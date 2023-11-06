use num_traits::FromPrimitive;
use socketcan::EmbeddedFrame;
use wannsea_types::{MetricId, MetricMessage};

use crate::{can::{CanReceiver, get_can_id, ids::CanIds}, messaging::MetricSender, SETTINGS};

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
                Some(CanIds::CanIdApmuTemp) => metric_sender.send(MetricMessage::now(MetricId::APMU_TEMP, data.into())),
                Some(CanIds::CanIdMpmuTemp) => metric_sender.send(MetricMessage::now(MetricId::MPMU_TEMP, data.into())),
                Some(CanIds::CanIdMotorCurrent) => metric_sender.send(MetricMessage::now(MetricId::MOTOR_CURRENT, data.into())),
                Some(CanIds::CanIdBattVoltage) => metric_sender.send(MetricMessage::now(MetricId::BATTERY_VOLTAGE, data.into())),
                Some(CanIds::CanIdFan1Rpm) => metric_sender.send(MetricMessage::now(MetricId::FAN_1, data.into())),
                Some(CanIds::CanIdFan2Rpm) => metric_sender.send(MetricMessage::now(MetricId::FAN_2, data.into())),
                Some(CanIds::CanIdFan3Rpm) => metric_sender.send(MetricMessage::now(MetricId::FAN_3, data.into())),
                Some(CanIds::CanIdFan4Rpm) => metric_sender.send(MetricMessage::now(MetricId::FAN_4, data.into())),
                Some(CanIds::CanIdSolarPower) => metric_sender.send(MetricMessage::now(MetricId::SOLAR_POWER, data.into())),
                Some(CanIds::CanIdSolarTemp) => metric_sender.send(MetricMessage::now(MetricId::SOLAR_TEMP, data.into())),
                _x => Ok(0)
            };

            result.unwrap();
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("pmu.enabled").unwrap() == true {
            tokio::spawn(Self::listen_can(self.can_receiver.clone(), self.metric_sender.clone()));
        }
    }
}

