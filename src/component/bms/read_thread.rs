use log::warn;
use socketcan::EmbeddedFrame;
use wannsea_types::MetricId;
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

    async fn parse_voltage_data(&self, metrics: Vec<MetricId>, data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            let base_idx = idx * 2;
            self.metric_sender.send_now(*metric, Value::Uint32(u16::from_be_bytes(data[base_idx..(base_idx + 2)].try_into().unwrap()) as u32)).unwrap();
        }
    }

    async fn parse_temp_data(&self, metrics: &[MetricId; 3], data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            self.metric_sender.send_now(*metric, Value::Uint32((data[idx] - 40) as u32)).unwrap();
        }
    }

    async fn parse_bms_id_v_21_24(&self, metrics: &[MetricId; 5], data: &[u8]) {
        
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

    async fn parse_internal_status_1(&self, metrics: &[MetricId; 4], data: &[u8]) {
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
                    1 => self.parse_voltage_data(vec![MetricId::BAT_1_U_1, MetricId::BAT_1_U_2, MetricId::BAT_1_U_3, MetricId::BAT_1_U_4], data).await,
                    2 => self.parse_voltage_data(vec![MetricId::BAT_2_U_1, MetricId::BAT_2_U_2, MetricId::BAT_2_U_3, MetricId::BAT_2_U_4], data).await,
                    3 => self.parse_voltage_data(vec![MetricId::BAT_3_U_1, MetricId::BAT_3_U_2, MetricId::BAT_3_U_3, MetricId::BAT_3_U_4], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV05_08 as u32 {
                match bms_id {
                    1 => self.parse_voltage_data(vec![MetricId::BAT_1_U_5, MetricId::BAT_1_U_6, MetricId::BAT_1_U_7, MetricId::BAT_1_U_8], data).await,
                    2 => self.parse_voltage_data(vec![MetricId::BAT_2_U_5, MetricId::BAT_2_U_6, MetricId::BAT_2_U_7, MetricId::BAT_2_U_8], data).await,
                    3 => self.parse_voltage_data(vec![MetricId::BAT_3_U_5, MetricId::BAT_3_U_6, MetricId::BAT_3_U_7, MetricId::BAT_3_U_8], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV09_12 as u32 {
                match bms_id {
                    1 => self.parse_voltage_data(vec![MetricId::BAT_1_U_9, MetricId::BAT_1_U_10, MetricId::BAT_1_U_11, MetricId::BAT_1_U_12], data).await,
                    2 => self.parse_voltage_data(vec![MetricId::BAT_2_U_9, MetricId::BAT_2_U_10, MetricId::BAT_2_U_11, MetricId::BAT_2_U_12], data).await,
                    3 => self.parse_voltage_data(vec![MetricId::BAT_3_U_9, MetricId::BAT_3_U_10, MetricId::BAT_3_U_11, MetricId::BAT_3_U_12], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV13_16 as u32 {
                match bms_id {
                    1 => self.parse_voltage_data(vec![MetricId::BAT_1_U_13, MetricId::BAT_1_U_14], data).await,
                    2 => self.parse_voltage_data(vec![MetricId::BAT_2_U_13, MetricId::BAT_2_U_14], data).await,
                    3 => self.parse_voltage_data(vec![MetricId::BAT_3_U_13, MetricId::BAT_3_U_14], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdV21_24 as u32 {
                match bms_id {
                    1 => self.parse_bms_id_v_21_24(&[MetricId::BAT_1_AH_DISCHARGED, MetricId::BAT_1_REMAINING_CAPACITY, MetricId::BAT_1_SOH, MetricId::BAT_1_SOC, MetricId::BAT_1_I_BAT_I], data).await,
                    2 => self.parse_bms_id_v_21_24(&[MetricId::BAT_2_AH_DISCHARGED, MetricId::BAT_2_REMAINING_CAPACITY, MetricId::BAT_2_SOH, MetricId::BAT_2_SOC, MetricId::BAT_2_I_BAT_I], data).await,
                    3 => self.parse_bms_id_v_21_24(&[MetricId::BAT_3_AH_DISCHARGED, MetricId::BAT_3_REMAINING_CAPACITY, MetricId::BAT_3_SOH, MetricId::BAT_3_SOC, MetricId::BAT_3_I_BAT_I], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
        
            else if bms_requested_fun == BmsFunction::BmsIdT01_06 as u32 {
                match bms_id {
                    1 => self.parse_temp_data(&[MetricId::BAT_1_T0, MetricId::BAT_1_T1, MetricId::BAT_1_T2], data).await,
                    2 => self.parse_temp_data(&[MetricId::BAT_2_T0, MetricId::BAT_2_T1, MetricId::BAT_2_T2], data).await,
                    3 => self.parse_temp_data(&[MetricId::BAT_3_T0, MetricId::BAT_3_T1, MetricId::BAT_3_T2], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
            else if bms_requested_fun == BmsFunction::BmsIdInternalStatus1 as u32 {
                match bms_id {
                    1 => self.parse_internal_status_1(&[MetricId::BAT_1_MAJOR_ALERT_1, MetricId::BAT_1_MAJOR_ALERT_2, MetricId::BAT_1_MAJOR_ALERT_3, MetricId::BAT_1_MINOR_ALERT], data).await,
                    2 => self.parse_internal_status_1(&[MetricId::BAT_2_MAJOR_ALERT_1, MetricId::BAT_2_MAJOR_ALERT_2, MetricId::BAT_2_MAJOR_ALERT_3, MetricId::BAT_2_MINOR_ALERT], data).await,
                    3 => self.parse_internal_status_1(&[MetricId::BAT_3_MAJOR_ALERT_1, MetricId::BAT_3_MAJOR_ALERT_2, MetricId::BAT_3_MAJOR_ALERT_3, MetricId::BAT_3_MINOR_ALERT], data).await,
                    x => warn!("Invalid BMS ID {}", x)
                };
            }
        }
    }

    fn parse_bms_master_message(&self, id: u32, data: &[u8]) {
        let requested_function = id & 0x0FFF;

        if requested_function == BmsFunction::EmsControl as u32 {
            self.metric_sender.send_now(MetricId::MAX_BATTERY_DISCHARGE_CURRENT, Value::Uint32(u16::from_be_bytes(data[0..2].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MetricId::MAX_BATTERY_RECHARGE_CURRENT, Value::Uint32(u16::from_be_bytes(data[2..4].try_into().unwrap()).into())).unwrap();
        }
        else if requested_function == BmsFunction::GlobalStatus3 as u32 {
            self.metric_sender.send_now(MetricId::GLOBAL_SOC, Value::Uint32(data[0].into())).unwrap();
            self.metric_sender.send_now(MetricId::ID_GLOBAL_SOC, Value::Uint32(((data[1] >> 4) as u8).into())).unwrap();
            self.metric_sender.send_now(MetricId::GLOBAL_IBMS_ALARM_STATE, Value::Uint32(((data[2] & 0x03) as u8).into())).unwrap();
            self.metric_sender.send_now(MetricId::NUMBER_OF_CONNECTED_BMS, Value::Uint32(((data[2] >> 4) as u8).into())).unwrap();
            self.metric_sender.send_now(MetricId::POWERBUS_INFORMATION, Value::Uint32((data[3] as u8).into())).unwrap();
        }
        else if requested_function == BmsFunction::GlobalStatus4 as u32 {
            self.metric_sender.send_now(MetricId::BAT_TMIN, Value::Uint32((data[0] as u8 - 40).into())).unwrap();
            self.metric_sender.send_now(MetricId::BAT_TMAX, Value::Uint32((data[1] as u8 - 40).into())).unwrap();
            self.metric_sender.send_now(MetricId::BAT_ID_TMIN, Value::Uint32(((data[2] & 0x0F) as u8).into())).unwrap();
            self.metric_sender.send_now(MetricId::BAT_ID_TMAX, Value::Uint32(((data[2] >> 4) as u8).into())).unwrap();

            self.metric_sender.send_now(MetricId::BAT_VMIN, Value::Uint32(u16::from_be_bytes(data[3 .. 5].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MetricId::BAT_VMAX, Value::Uint32(u16::from_be_bytes(data[5 .. 7].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MetricId::BAT_ID_VMIN, Value::Uint32(((data[7] & 0x0F) as u8).into())).unwrap();
            self.metric_sender.send_now(MetricId::BAT_ID_VMAX, Value::Uint32(((data[7] >> 4) as u8).into())).unwrap();
        }
        else if requested_function == BmsFunction::GlobalStatus5 as u32 {
            self.metric_sender.send_now(MetricId::GLOBAL_BAT_CURRENT, Value::Sint32(i16::from_be_bytes(data[0 .. 2].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MetricId::GLOBAL_CELL_V_MIN, Value::Sint32(i16::from_be_bytes(data[2 .. 4].try_into().unwrap()).into())).unwrap();
            self.metric_sender.send_now(MetricId::GLOBAL_CELL_V_MAX, Value::Sint32(i16::from_be_bytes(data[4 .. 6].try_into().unwrap()).into())).unwrap();

            self.metric_sender.send_now(MetricId::GLOBAL_CELL_V_MIN_ID, Value::Uint32(((data[6] & 0x0F) as u8).into())).unwrap();
            self.metric_sender.send_now(MetricId::GLOBAL_CELL_V_MAX_ID, Value::Uint32(((data[6] >> 4) as u8).into())).unwrap();
        }
    }
}