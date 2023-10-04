use byteorder::{ReadBytesExt, BigEndian};
use num_traits::FromPrimitive;
use socketcan::EmbeddedFrame;

use crate::{can::{CanReceiver, get_can_id, ids::CanIds}, messaging::app_message::{MetricSender, MetricMessage}, SETTINGS};

pub struct PMU {
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

impl PMU {
    pub fn new(can_receiver: CanReceiver, metric_sender: MetricSender) -> Self {
        PMU { can_receiver, metric_sender }
    }

    fn parse_float(mut data: &[u8]) -> f32 {
        return data.read_f32::<BigEndian>().unwrap();
    }

    pub async fn listen_can(can_receiver: CanReceiver, metric_sender: MetricSender) {
        let mut receiver = can_receiver.subscribe();
        loop {
            let frame = receiver.recv().await.unwrap();
            let data = frame.data();
            let id = get_can_id(frame.id());
            let result = match FromPrimitive::from_u32(id) {
                Some(CanIds::CanIdApmuTemp) => metric_sender.send(MetricMessage::now(String::from("APMU_TEMP"), Self::parse_float(data))),
                Some(CanIds::CanIdMpmuTemp) => metric_sender.send(MetricMessage::now(String::from("MPMU_TEMP"), Self::parse_float(data))),
                Some(CanIds::CanIdMotorCurrent) => metric_sender.send(MetricMessage::now(String::from("MOTOR_I"), Self::parse_float(data))),
                Some(CanIds::CanIdBattVoltage) => metric_sender.send(MetricMessage::now(String::from("PMU_BAT_V"), Self::parse_float(data))),
                Some(CanIds::CanIdFan1Rpm) => metric_sender.send(MetricMessage::now(String::from("FAN_1_RPM"), Self::parse_float(data))),
                Some(CanIds::CanIdFan2Rpm) => metric_sender.send(MetricMessage::now(String::from("FAN_2_RPM"), Self::parse_float(data))),
                Some(CanIds::CanIdFan3Rpm) => metric_sender.send(MetricMessage::now(String::from("FAN_3_RPM"), Self::parse_float(data))),
                Some(CanIds::CanIdFan4Rpm) => metric_sender.send(MetricMessage::now(String::from("FAN_4_RPM"), Self::parse_float(data))),
                Some(CanIds::CanIdSolarPower) => metric_sender.send(MetricMessage::now(String::from("SOLAR_POWER"), Self::parse_float(data))),
                Some(CanIds::CanIdSolarTemp) => metric_sender.send(MetricMessage::now(String::from("SOLAR_TEMP"), Self::parse_float(data))),
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

