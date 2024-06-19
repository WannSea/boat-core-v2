use eskf;
use log::{info, warn};
use nalgebra::{Point3, Vector3};
use wannsea_types::{boat_core_message::Value, Floats, MessageId};
use std::time::Duration;
use map_3d::{geodetic2ned, ned2geodetic, Ellipsoid::WGS84};

use crate::{helper::{MetricSender, MetricSenderExt}, SETTINGS};

pub struct SensorFusion {
    metric_sender: MetricSender
}

impl SensorFusion {
    pub fn new(metric_sender: MetricSender) -> Self {
        SensorFusion { metric_sender }
    }

    pub async fn run(metric_sender: MetricSender, gpsConverter: CoordinatesConverter) {
        let mut metric_receiver = metric_sender.subscribe();

        // https://www.ceva-ip.com/wp-content/uploads/2019/10/BNO080_085-Datasheet.pdf
        // Chapter 6.7
        let mut filter = eskf::Builder::new()
        // .rotation_variance(0.541052)
        // .acceleration_variance(0.3)
        //.initial_covariance(1e-1)
        .build();

        let mut last_update_ns: u128 = 0;
        let mut imu_acceleration = Vector3::new(0.0, 0.0, -9.81);
        let mut imu_rotation = Vector3::zeros();
        loop { 
            let metric: wannsea_types::BoatCoreMessage = metric_receiver.recv().await.unwrap();

            // if let Shape::Circle(_, radius) = my_shape {
            //     println!("value: {}", radius);
            // }
            if metric.id == MessageId::GpsPos {
                match metric.value.unwrap() {
                    Value::Floats(floats) => {
                        filter.observe_position(
                            gpsConverter.gps_to_ned(floats.values[0], floats.values[1]),
                            eskf::ESKF::variance_from_element(0.1))
                                .expect("Filter update failed");
                    },
                    _ => warn!("GPS unexpected Data format")
                }
            }
            else if metric.id == MessageId::ImuAcceleration {
                match metric.value.unwrap() {
                    Value::Floats(floats) => {
                        imu_acceleration[0] = floats.values[0];
                        imu_acceleration[1] = -floats.values[1];
                        imu_acceleration[2] = floats.values[2];
                    },
                    _ => warn!("Acceleration unexpected metric format")
                }
            }
            else if metric.id == MessageId::ImuGyro {
                let new_ts = metric.get_ts_ns();
                match metric.value.as_ref().unwrap() {
                    Value::Floats(floats) => {
                        imu_rotation[0] = floats.values[0];
                        imu_rotation[1] = -floats.values[1];
                        imu_rotation[2] = floats.values[2];
                    },
                    _ => warn!("Rotation unexpected metric format")
                }                
                filter.predict(imu_acceleration, imu_rotation, Duration::from_nanos((new_ts - last_update_ns) as u64));
                last_update_ns = new_ts;

                let (lat, lon) = gpsConverter.ned_to_gps(filter.position);
                let pos_uncertainty = filter.position_uncertainty();
                let ori_uncertainty = filter.orientation_uncertainty();
                let velo_uncertainty = filter.velocity_uncertainty();
                metric_sender.send_now(MessageId::FusedPositionRelative, Value::Floats(Floats{ values: vec![filter.position.x, filter.position.y, filter.position.z] })).unwrap();
                metric_sender.send_now(MessageId::FusedPosition, Value::Floats(Floats{ values: vec![lat, lon, 0.0] })).unwrap();
                metric_sender.send_now(MessageId::FusedPositionUncertainty, Value::Floats(Floats{ values: vec![pos_uncertainty.x, pos_uncertainty.y, pos_uncertainty.z] })).unwrap();

                metric_sender.send_now(MessageId::FusedOrientation, Value::Floats(Floats{ values: vec![filter.orientation.i, filter.orientation.j, filter.orientation.k, filter.orientation.w] })).unwrap();
                metric_sender.send_now(MessageId::FusedOrientationUncertainty, Value::Floats(Floats{ values: vec![ori_uncertainty.x, ori_uncertainty.y, ori_uncertainty.z] })).unwrap();

                metric_sender.send_now(MessageId::FusedVelocity, Value::Floats(Floats{ values: vec![filter.velocity.x, filter.velocity.y, filter.velocity.z] })).unwrap();
                metric_sender.send_now(MessageId::FusedVelocityUncertainty, Value::Floats(Floats{ values: vec![velo_uncertainty.x, velo_uncertainty.y, velo_uncertainty.z] })).unwrap();
            }
            

            // Create a default filter, modelling a perfect IMU without drift
            // Read measurements from IMU
            // Flip Y because IMU coordinates are left handed



            // Tell the filter what we just measured
            // Check the new state of the filter
            // filter.position or filter.velocity...
            // ...
            // After some time we get an observation of the actual state
            
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("sensor_fusion.enabled").unwrap() {
            info!("Sensor Fusion enabled!");
            tokio::spawn(Self::run(self.metric_sender.clone(), CoordinatesConverter::default()));
        }
    }
}

pub struct CoordinatesConverter {
    reference_lat: f64,
    reference_lon: f64,
    reference_alt: f64,
}

impl CoordinatesConverter {
    pub fn default() -> Self {
        let reference_lat = SETTINGS.get::<f64>("sensor_fusion.reference_lat").unwrap();
        let reference_lon = SETTINGS.get::<f64>("sensor_fusion.reference_lon").unwrap();
        CoordinatesConverter { reference_lat, reference_lon, reference_alt: 0.0}
    }

    fn gps_to_ned(&self, lat: f32, lon: f32) -> Point3<f32> {
        let ned = geodetic2ned(lat as f64, lon as f64, 0.0, self.reference_lat, self.reference_lon, 0.0, WGS84);
        Point3::new(ned.0 as f32, ned.1 as f32, ned.2 as f32)
    }

    fn ned_to_gps(&self, ned: Point3<f32>) -> (f32, f32){
        let (lat, lon, _) = ned2geodetic(ned.x as f64, ned.y as f64, ned.z as f64, self.reference_lat, self.reference_lon, self.reference_alt, WGS84);
        return (lat as f32, lon as f32);
    }

}