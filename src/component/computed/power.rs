use log::{debug, info, warn};
use num_traits::FromPrimitive;
use socketcan::EmbeddedFrame;
use wannsea_types::boat_core_message::Value;
use wannsea_types::MessageId;

use crate::helper::MetricSenderExt;
use crate::SETTINGS;
use crate::{can::{CanReceiver, get_can_id}, helper::MetricSender};


pub struct MotorPower {
    metric_sender: MetricSender
}

impl MotorPower {

    pub fn start(&self) {
        if SETTINGS.get::<bool>("motor_power.enabled").unwrap() == true {
            info!("Computed Motor Power enabled!");

            tokio::spawn(Self::start_receiving(self.metric_sender.clone()));
        }
    }

    pub fn new(metric_sender: MetricSender) -> MotorPower {
        MotorPower { metric_sender }
    }

    async fn start_receiving(sender: MetricSender) {
        let mut receiver = sender.subscribe();

        loop {
            let metric = receiver.recv().await.unwrap();

            let mut last_current: f32 = 0.0f32;
            let mut last_voltage: f32 = 0.0f32;

            if metric.id() == MessageId::EscTotalInCurrent {
                match metric.value.unwrap() {
                    Value::Float(x) => {
                        last_current = x;
                    },
                    _ => warn!("Unexpected ESC Current format")
                }
            }
            else if metric.id() == MessageId::EscInVoltage {
                match metric.value.unwrap() {
                    Value::Float(x) => {
                        last_voltage = x;
                        sender.send_now(MessageId::EscTotalInPower, Value::Float(last_voltage * last_current)).unwrap();
                    },
                    _ => warn!("Unexpected ESC Current format")
                }
            }
        }
    }
}