use eskf;
use log::{info, warn};
use nalgebra::{Vector3, Point3};
use wannsea_types::{boat_core_message::Value, Floats, MessageId};
use std::time::Duration;

use crate::{helper::{MetricSender, MetricSenderExt}, SETTINGS};

pub struct SensorFusion {
    metric_sender: MetricSender
}

impl SensorFusion {
    pub fn new(metric_sender: MetricSender) -> Self {
        SensorFusion { metric_sender }
    }

    pub async fn run(metric_sender: MetricSender) {
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
                            Point3::new(floats.values[0], floats.values[1], 0.0),
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
                match metric.value.as_ref().unwrap() {
                    Value::Floats(floats) => {
                        imu_rotation[0] = floats.values[0];
                        imu_rotation[1] = -floats.values[1];
                        imu_rotation[2] = floats.values[2];
                    },
                    _ => warn!("Rotation unexpected metric format")
                }                
                filter.predict(imu_acceleration, imu_rotation, Duration::from_nanos((metric.get_ts_ns() - last_update_ns) as u64));
                last_update_ns = metric.get_ts_ns();

                let pos_uncertainty = filter.position_uncertainty();
                let ori_uncertainty = filter.orientation_uncertainty();
                let velo_uncertainty = filter.velocity_uncertainty();
                metric_sender.send_now(MessageId::FusedPosition, Value::Floats(Floats{ values: vec![filter.position.x, filter.position.y, filter.position.z] })).unwrap();
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
            // filter.position or filter.velxocity...
            // ...
            // After some time we get an observation of the actual state
            
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("sensor_fusion.enabled").unwrap() {
            info!("Sensor Fusion enabled!");
            tokio::spawn(Self::run(self.metric_sender.clone()));
        }
    }
}