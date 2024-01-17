use eskf;
use log::{info, warn};
use nalgebra::{Vector3, Point3};
use wannsea_types::{MessageId, Vector2, boat_core_message::Value};
use std::time::Duration;

use crate::{helper::MetricSender, SETTINGS};

pub struct SensorFusion {
    metric_sender: MetricSender
}

impl SensorFusion {
    pub fn new(metric_sender: MetricSender) -> Self {
        SensorFusion { metric_sender }
    }

    pub async fn run(metric_sender: MetricSender) {
        let mut metric_receiver = metric_sender.subscribe();
        let mut filter = eskf::Builder::new().build();
        loop { 
            let metric: wannsea_types::BoatCoreMessage = metric_receiver.recv().await.unwrap();

            // if let Shape::Circle(_, radius) = my_shape {
            //     println!("value: {}", radius);
            // }
            if metric.id == MessageId::GpsPos {
                match metric.value.unwrap() {
                    Value::Vector2(vec2) => {
                        filter.observe_position(
                            Point3::new(vec2.x, vec2.y, 0.0),
                            eskf::ESKF::variance_from_element(0.1))
                                .expect("Filter update failed");
                    },
                    _ => warn!("GPS unexpected Data format")
                }
            }
            else if metric.id == MessageId::Acceleration {
                
            }
            
            // Create a default filter, modelling a perfect IMU without drift
            // Read measurements from IMU
            let imu_acceleration = Vector3::new(0.0, 0.0, -9.81);
            let imu_rotation = Vector3::zeros();
            // Tell the filter what we just measured
            filter.predict(imu_acceleration, imu_rotation, Duration::from_millis(1000));
            // Check the new state of the filter
            // filter.position or filter.velxocity...
            // ...
            // After some time we get an observation of the actual state
            
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("system.enabled").unwrap() {
            info!("System Stats enabled!");
            tokio::spawn(Self::run(self.metric_sender.clone()));
        }
    }
}