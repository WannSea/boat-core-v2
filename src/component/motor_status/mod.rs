use log::info;
use num_traits::FromPrimitive;
use socketcan::EmbeddedFrame;
use wannsea_types::{boat_core_message::Value, MessageId};


use crate::{can::{CanReceiver, get_can_id, ids::CanIds}, helper::{MetricSender, MetricSenderExt}, SETTINGS};

pub struct Motor {
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

impl Motor {
    pub fn new(can_receiver: CanReceiver, metric_sender: MetricSender) -> Self {
        Motor { can_receiver, metric_sender }
    }

    pub async fn listen_can(can_receiver: CanReceiver, metric_sender: MetricSender) {
        let mut receiver = can_receiver.subscribe();
        loop {
            let frame = receiver.recv().await.unwrap();
            let data = frame.data().to_vec();
            let id = get_can_id(frame.id());
            let result = match FromPrimitive::from_u32(id) {
                Some(CanIds::CanIdMotorTemp) => metric_sender.send_now(
                    MessageId::MotorTemperature,
                    Value::Float(f32::from(i16::from_be_bytes(data[0..2].try_into().unwrap())) / 100.0)
                ),
                _x => Ok(0)
            };

            result.unwrap();
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("motor_status.enabled").unwrap() == true {
            info!("Motor enabled!");

            tokio::spawn(Self::listen_can(self.can_receiver.clone(), self.metric_sender.clone()));
        }
    }
}

