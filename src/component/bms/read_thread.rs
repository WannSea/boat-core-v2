use log::warn;
use socketcan::EmbeddedFrame;
use wannsea_types::MessageId;
use wannsea_types::boat_core_message::Value;

use crate::helper::MetricSenderExt;
use crate::{can::{CanReceiver, get_can_id}, helper::MetricSender};

use super::{structs::{BmsFunction, BatteryPack}, BatteryPackNotifier};

pub struct BmsReadThread {
    can_receiver: CanReceiver,
    metric_sender: MetricSender,
    pack_notifier: BatteryPackNotifier
}

// Read Methods
impl BmsReadThread {
    pub async fn start(can_receiver: CanReceiver, metric_sender: MetricSender, notifier: BatteryPackNotifier) {
        let thread = BmsReadThread { can_receiver, metric_sender, pack_notifier: notifier };
        thread.start_receiving().await;
    }

    async fn parse_voltage_data(&self, metrics: Vec<MessageId>, data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            let base_idx = idx * 2;
            self.metric_sender.send_now(*metric, Value::Uint32(u16::from_be_bytes(data[base_idx..(base_idx + 2)].try_into().unwrap()) as u32)).unwrap();
        }
    }

    async fn parse_temp_data(&self, metrics: &[MessageId; 3], data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            self.metric_sender.send_now(*metric, Value::Uint32((data[idx] - 40) as u32)).unwrap();
        }
    }

    async fn parse_bms_id_v_21_24(&self, metrics: &[MessageId; 5], data: &[u8]) {
        
        let ah_discharged_in_life = u16::from_be_bytes(data[0..2].try_into().unwrap());
        let remaining_capacity = u16::from_be_bytes(data[2..4].try_into().unwrap());
        let soh = data[4];
        let soc = data[5];
        let i_batt_i = u16::from_be_bytes(data[6..8].try_into().unwrap());

        self.metric_sender.send_now(metrics[0], Value::Uint32(ah_discharged_in_life.into())).unwrap();
        self.metric_sender.send_now(metrics[1], Value::Uint32(remaining_capacity.into())).unwrap();
        self.metric_sender.send_now(metrics[2], Value::Uint32(soh.into())).unwrap();
        self.metric_sender.send_now(metrics[3], Value::Uint32(soc.into())).unwrap();
        self.metric_sender.send_now(metrics[4], Value::Uint32(i_batt_i.into())).unwrap();
    }

    async fn parse_internal_status_1(&self, metrics: &[MessageId; 4], data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            self.metric_sender.send_now(*metric, Value::Uint32(data[idx].into())).unwrap();
        }
    }

    async fn start_receiving(&self) {
        let mut receiver = self.can_receiver.subscribe();
        loop {
            let frame = receiver.recv().await.unwrap();

            if frame.dlc() != 8 {
                continue;
            }

            let id = get_can_id(frame.id());

            let bms_id = (id >> 12) as u8;
           
            let data = frame.data();
            if bms_id < 1 || bms_id >9 {
                self.parse_bms_master_message(id, data);
                continue
            }
            
            let bms_requested_fun = id & 0x0FFF;
            if bms_requested_fun == BmsFunction::BmsIdSerialNumberAnswer as u32 {
                let pack = BatteryPack {
                    id: bms_id,
                    serial_number: u32::from_be_bytes(data[0..4].try_into().unwrap()),
                    part_number: u32::from_be_bytes(data[4..8].try_into().unwrap()),
                };
                // Got bat pack serial number answer, notify main/write thread
                self.pack_notifier.send(pack).await.unwrap();
            } 
            else if bms_requested_fun == BmsFunction::BmsIdV01_04 as u32 {
                match bms_id {
                    1 => self.parse_voltage_data(vec![MessageId::Bat1U1, MessageId::Bat1U2, MessageId::Bat1U3, MessageId::Bat1U4], data).await,
                    2 => self.parse_voltage_data(vec![MessageId::Bat2U1, MessageId::Bat2U2, MessageId::Bat2U3, MessageId::Bat2U4], data).await,
                    3 => self.parse_voltage_data(vec![MessageId::Bat3U1, MessageId::Bat3U2, MessageId::Bat3U3, MessageId::Bat3U4], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV05_08 as u32 {
                match bms_id {
                    1 => self.parse_voltage_data(vec![MessageId::Bat1U5, MessageId::Bat1U6, MessageId::Bat1U7, MessageId::Bat1U8], data).await,
                    2 => self.parse_voltage_data(vec![MessageId::Bat2U5, MessageId::Bat2U6, MessageId::Bat2U7, MessageId::Bat2U8], data).await,
                    3 => self.parse_voltage_data(vec![MessageId::Bat3U5, MessageId::Bat3U6, MessageId::Bat3U7, MessageId::Bat3U8], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV09_12 as u32 {
                match bms_id {
                    1 => self.parse_voltage_data(vec![MessageId::Bat1U9, MessageId::Bat1U10, MessageId::Bat1U11, MessageId::Bat1U12], data).await,
                    2 => self.parse_voltage_data(vec![MessageId::Bat2U9, MessageId::Bat2U10, MessageId::Bat2U11, MessageId::Bat2U12], data).await,
                    3 => self.parse_voltage_data(vec![MessageId::Bat3U9, MessageId::Bat3U10, MessageId::Bat3U11, MessageId::Bat3U12], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV13_16 as u32 {
                match bms_id {
                    1 => self.parse_voltage_data(vec![MessageId::Bat1U13, MessageId::Bat1U14], data).await,
                    2 => self.parse_voltage_data(vec![MessageId::Bat2U13, MessageId::Bat2U14], data).await,
                    3 => self.parse_voltage_data(vec![MessageId::Bat3U13, MessageId::Bat3U14], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV21_24 as u32 {
                match bms_id {
                    1 => self.parse_bms_id_v_21_24(&[MessageId::Bat1AhDischarged, MessageId::Bat1AhDischarged, MessageId::Bat1Soh, MessageId::Bat1Soc, MessageId::Bat1IBatI], data).await,
                    2 => self.parse_bms_id_v_21_24(&[MessageId::Bat2AhDischarged, MessageId::Bat2AhDischarged, MessageId::Bat2Soh, MessageId::Bat2Soc, MessageId::Bat2IBatI], data).await,
                    3 => self.parse_bms_id_v_21_24(&[MessageId::Bat3AhDischarged, MessageId::Bat3AhDischarged, MessageId::Bat3Soh, MessageId::Bat3Soc, MessageId::Bat3IBatI], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
        
            else if bms_requested_fun == BmsFunction::BmsIdT01_06 as u32 {
                match bms_id {
                    1 => self.parse_temp_data(&[MessageId::Bat1T0, MessageId::Bat1T1, MessageId::Bat1T2], data).await,
                    2 => self.parse_temp_data(&[MessageId::Bat2T0, MessageId::Bat2T1, MessageId::Bat2T2], data).await,
                    3 => self.parse_temp_data(&[MessageId::Bat3T0, MessageId::Bat3T1, MessageId::Bat3T2], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdInternalStatus1 as u32 {
                match bms_id {
                    1 => self.parse_internal_status_1(&[MessageId::Bat1MajorAlert1, MessageId::Bat1MajorAlert2, MessageId::Bat1MajorAlert3, MessageId::Bat1MinorAlert], data).await,
                    2 => self.parse_internal_status_1(&[MessageId::Bat1MajorAlert1, MessageId::Bat2MajorAlert2, MessageId::Bat2MajorAlert3, MessageId::Bat2MinorAlert], data).await,
                    3 => self.parse_internal_status_1(&[MessageId::Bat1MajorAlert1, MessageId::Bat3MajorAlert2, MessageId::Bat3MajorAlert3, MessageId::Bat3MinorAlert], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
        }
    }

    fn parse_bms_master_message(&self, id: u32, data: &[u8]) {
        let requested_function = id & 0x0FFF;

        if requested_function == BmsFunction::EmsControl as u32 {
            self.metric_sender.send_now(MessageId::MaxBatteryDischargeCurrent, Value::Uint32(u16::from_be_bytes(data[0..2].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MessageId::MaxBatteryRechargeCurrent, Value::Uint32(u16::from_be_bytes(data[2..4].try_into().unwrap()).into())).unwrap();
        }
        else if requested_function == BmsFunction::GlobalStatus3 as u32 {
            self.metric_sender.send_now(MessageId::GlobalSoc, Value::Uint32(data[0].into())).unwrap();
            self.metric_sender.send_now(MessageId::IdGlobalSoc, Value::Uint32(((data[1] >> 4) as u8).into())).unwrap();
            self.metric_sender.send_now(MessageId::GlobalIbmsAlarmState, Value::Uint32(((data[2] & 0x03) as u8).into())).unwrap();
            self.metric_sender.send_now(MessageId::NumberOfConnectedBms, Value::Uint32(((data[2] >> 4) as u8).into())).unwrap();
            self.metric_sender.send_now(MessageId::PowerbusInformation, Value::Uint32((data[3] as u8).into())).unwrap();
        }
        else if requested_function == BmsFunction::GlobalStatus4 as u32 {
            self.metric_sender.send_now(MessageId::BatTmin, Value::Uint32((data[0] as u8 - 40).into())).unwrap();
            self.metric_sender.send_now(MessageId::BatTmax, Value::Uint32((data[1] as u8 - 40).into())).unwrap();
            self.metric_sender.send_now(MessageId::BatIdTmin, Value::Uint32(((data[2] & 0x0F) as u8).into())).unwrap();
            self.metric_sender.send_now(MessageId::BatIdTmax, Value::Uint32(((data[2] >> 4) as u8).into())).unwrap();

            self.metric_sender.send_now(MessageId::BatVmin, Value::Uint32(u16::from_be_bytes(data[3 .. 5].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MessageId::BatVmax, Value::Uint32(u16::from_be_bytes(data[5 .. 7].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MessageId::BatIdVmin, Value::Uint32(((data[7] & 0x0F) as u8).into())).unwrap();
            self.metric_sender.send_now(MessageId::BatIdVmax, Value::Uint32(((data[7] >> 4) as u8).into())).unwrap();
        }
        else if requested_function == BmsFunction::GlobalStatus5 as u32 {
            let bat_current: i16 = i16::from_be_bytes(data[0 .. 2].try_into().unwrap()).into();

            self.metric_sender.send_now(MessageId::GlobalBatCurrent, Value::Float(bat_current as f32 * 0.1f32)).unwrap();
            self.metric_sender.send_now(MessageId::GlobalCellVMin, Value::Sint32(i16::from_be_bytes(data[2 .. 4].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MessageId::GlobalCellVMax, Value::Sint32(i16::from_be_bytes(data[4 .. 6].try_into().unwrap()).into())).unwrap();

            self.metric_sender.send_now(MessageId::GlobalCellVMinId, Value::Uint32(((data[6] & 0x0F) as u8).into())).unwrap();
            self.metric_sender.send_now(MessageId::GlobalCellVMaxId, Value::Uint32(((data[6] >> 4) as u8).into())).unwrap();
        }
    }
}