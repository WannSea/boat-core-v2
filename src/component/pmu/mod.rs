use num_traits::FromPrimitive;
use socketcan::EmbeddedFrame;
use wannsea_types::types::Metric;

use crate::{can::{CanReceiver, get_can_id, ids::CanIds}, messaging::app_message::{MetricSender, MetricMessage}, SETTINGS};

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
                Some(CanIds::CanIdApmuTemp) => metric_sender.send(MetricMessage::now(Metric::ApmuTemp, data)),
                Some(CanIds::CanIdMpmuTemp) => metric_sender.send(MetricMessage::now(Metric::MpmuTemp, data)),
                Some(CanIds::CanIdMotorCurrent) => metric_sender.send(MetricMessage::now(Metric::MotorCurrent, data)),
                Some(CanIds::CanIdBattVoltage) => metric_sender.send(MetricMessage::now(Metric::BatteryVoltage, data)),
                Some(CanIds::CanIdFan1Rpm) => metric_sender.send(MetricMessage::now(Metric::Fan1, data)),
                Some(CanIds::CanIdFan2Rpm) => metric_sender.send(MetricMessage::now(Metric::Fan2, data)),
                Some(CanIds::CanIdFan3Rpm) => metric_sender.send(MetricMessage::now(Metric::Fan3, data)),
                Some(CanIds::CanIdFan4Rpm) => metric_sender.send(MetricMessage::now(Metric::Fan4, data)),
                Some(CanIds::CanIdSolarPower) => metric_sender.send(MetricMessage::now(Metric::SolarPower, data)),
                Some(CanIds::CanIdSolarTemp) => metric_sender.send(MetricMessage::now(Metric::SolarTemp, data)),
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

