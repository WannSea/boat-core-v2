use log::info;
use num_traits::FromPrimitive;
use socketcan::EmbeddedFrame;
use wannsea_types::MetricId;
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
                Some(CanIds::CanIdApmuTemp) => metric_sender.send_now(MetricId::APMU_TEMP, Value::Bytes(data)),
                Some(CanIds::CanIdMpmuTemp) => metric_sender.send_now(MetricId::MPMU_TEMP, Value::Bytes(data)),
                Some(CanIds::CanIdMotorCurrent) => metric_sender.send_now(MetricId::MOTOR_CURRENT, Value::Bytes(data)),
                Some(CanIds::CanIdBattVoltage) => metric_sender.send_now(MetricId::BATTERY_VOLTAGE, Value::Bytes(data)),
                Some(CanIds::CanIdFan1Rpm) => metric_sender.send_now(MetricId::FAN_1, Value::Bytes(data)),
                Some(CanIds::CanIdFan2Rpm) => metric_sender.send_now(MetricId::FAN_2, Value::Bytes(data)),
                Some(CanIds::CanIdFan3Rpm) => metric_sender.send_now(MetricId::FAN_3, Value::Bytes(data)),
                Some(CanIds::CanIdFan4Rpm) => metric_sender.send_now(MetricId::FAN_4, Value::Bytes(data)),
                Some(CanIds::CanIdSolarPower) => metric_sender.send_now(MetricId::SOLAR_POWER, Value::Bytes(data)),
                Some(CanIds::CanIdSolarTemp) => metric_sender.send_now(MetricId::SOLAR_TEMP, Value::Bytes(data)),
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

