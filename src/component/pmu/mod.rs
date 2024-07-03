use log::info;
use num_traits::{FromPrimitive, ToPrimitive};
use socketcan::EmbeddedFrame;
use wannsea_types::MessageId;
use wannsea_types::boat_core_message::Value;

use crate::{can::{CanReceiver, get_can_id, ids::CanIds}, helper::{MetricSender, MetricSenderExt}, SETTINGS};

pub struct PMU {
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

impl PMU {
    pub fn new(can_receiver: CanReceiver, metric_sender: MetricSender) -> Self {
        PMU { can_receiver, metric_sender }
    }

    pub async fn listen_can(can_receiver: CanReceiver, metric_sender: MetricSender) {
        let mut receiver = can_receiver.subscribe();
        loop {
            let frame = receiver.recv().await.unwrap();
            let data = frame.data().to_vec();
            let id = get_can_id(frame.id());
            let result = match FromPrimitive::from_u32(id) {
                Some(CanIds::CanIdApmuTemp) => metric_sender.send_now(MessageId::ApmuTemp, Value::Bytes(data)),
                Some(CanIds::CanIdMpmuTemp) => metric_sender.send_now(MessageId::MpmuTemp, Value::Bytes(data)),
                Some(CanIds::CanIdMotorCurrent) => metric_sender.send_now(MessageId::MotorCurrent, Value::Bytes(data)),
                Some(CanIds::CanIdBattVoltage) => metric_sender.send_now(MessageId::BatteryVoltage, Value::Bytes(data)),
                Some(CanIds::CanIdFan1Rpm) => metric_sender.send_now(MessageId::Fan1, Value::Bytes(data)),
                Some(CanIds::CanIdFan2Rpm) => metric_sender.send_now(MessageId::Fan2, Value::Bytes(data)),
                Some(CanIds::CanIdFan3Rpm) => metric_sender.send_now(MessageId::Fan3, Value::Bytes(data)),
                Some(CanIds::CanIdFan4Rpm) => metric_sender.send_now(MessageId::Fan4, Value::Bytes(data)),
                Some(CanIds::CanIdSolarPower) => metric_sender.send_now(MessageId::SolarPower, Value::Uint32(u32::from_be_bytes(data[0..4].try_into().unwrap()))),
                Some(CanIds::CanIdSolarTemp) => metric_sender.send_now(MessageId::SolarTemp, Value::Float(i16::from_be_bytes(data[0..2].try_into().unwrap()).to_f32().unwrap() * 0.01)),
                Some(CanIds::CanIdPCSTemp) => metric_sender.send_now(MessageId::PcsTemp, Value::Float(i16::from_be_bytes(data[0..2].try_into().unwrap()).to_f32().unwrap() * 0.01)),
                Some(CanIds::CanIdLPMainPower) => metric_sender.send_now(MessageId::LpMainPower, Value::Uint32(u16::from_be_bytes(data[0..2].try_into().unwrap()).to_u32().unwrap())),
                
                _x => Ok(0)
            };

            result.unwrap();
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("pmu.enabled").unwrap() == true {
            info!("PMU enabled!");

            tokio::spawn(Self::listen_can(self.can_receiver.clone(), self.metric_sender.clone()));
        }
    }
}

