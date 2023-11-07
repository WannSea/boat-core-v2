use log::{trace, warn, debug};
use socketcan::EmbeddedFrame;
use wannsea_types::{MetricId, MetricMessage};

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

    fn send_metric(&self, message: MetricMessage) {
        self.metric_sender.send(message).unwrap();
    }

    async fn parse_voltage_data(&self, metrics: Vec<MetricId>, data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            let base_idx = idx * 2;
            self.send_metric(MetricMessage::now(*metric, u16::from_be_bytes(data[base_idx..(base_idx + 2)].try_into().unwrap()).into()));
        }
    }

    async fn parse_temp_data(&self, metrics: &[MetricId; 3], data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            self.send_metric(MetricMessage::now(*metric, (data[idx] - 40).into()));
        }
    }

    async fn parse_bms_id_v_21_24(&self, metrics: &[MetricId; 5], data: &[u8]) {
        
        let ah_discharged_in_life = u16::from_be_bytes(data[0..2].try_into().unwrap());
        let remaining_capacity = u16::from_be_bytes(data[2..4].try_into().unwrap());
        let soh = data[4];
        let soc = data[5];
        let i_batt_i = u16::from_be_bytes(data[6..8].try_into().unwrap());

        self.send_metric(MetricMessage::now(metrics[0], ah_discharged_in_life.into()));
        self.send_metric(MetricMessage::now(metrics[1], remaining_capacity.into()));
        self.send_metric(MetricMessage::now(metrics[2], soh.into()));
        self.send_metric(MetricMessage::now(metrics[3], soc.into()));
        self.send_metric(MetricMessage::now(metrics[4], i_batt_i.into()));
    }

    async fn parse_internal_status_1(&self, metrics: &[MetricId; 4], data: &[u8]) {
        for (idx, metric) in metrics.iter().enumerate() {
            self.send_metric(MetricMessage::now(*metric, data[idx].into()));
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
            self.send_metric(MetricMessage::now(MetricId::MAX_BATTERY_DISCHARGE_CURRENT, u16::from_be_bytes(data[0..2].try_into().unwrap()).into()));
            self.send_metric(MetricMessage::now(MetricId::MAX_BATTERY_RECHARGE_CURRENT, u16::from_be_bytes(data[2..4].try_into().unwrap()).into()));
        }
        else if requested_function == BmsFunction::GlobalStatus3 as u32 {
            self.send_metric(MetricMessage::now(MetricId::GLOBAL_SOC, data[0].into()));
            self.send_metric(MetricMessage::now(MetricId::ID_GLOBAL_SOC, ((data[1] >> 4) as u8).into()));
            self.send_metric(MetricMessage::now(MetricId::GLOBAL_IBMS_ALARM_STATE, ((data[2] & 0x03) as u8).into()));
            self.send_metric(MetricMessage::now(MetricId::NUMBER_OF_CONNECTED_BMS, ((data[2] >> 4) as u8).into()));
            self.send_metric(MetricMessage::now(MetricId::POWERBUS_INFORMATION, (data[3] as u8).into()));
        }
        else if requested_function == BmsFunction::GlobalStatus4 as u32 {
            self.send_metric(MetricMessage::now(MetricId::BAT_TMIN, (data[0] as u8 - 40).into()));
            self.send_metric(MetricMessage::now(MetricId::BAT_TMAX, (data[1] as u8 - 40).into()));
            self.send_metric(MetricMessage::now(MetricId::BAT_ID_TMIN, ((data[2] & 0x0F) as u8).into()));
            self.send_metric(MetricMessage::now(MetricId::BAT_ID_TMAX, ((data[2] >> 4) as u8).into()));

            self.send_metric(MetricMessage::now(MetricId::BAT_VMIN, u16::from_be_bytes(data[3 .. 5].try_into().unwrap()).into()));
            self.send_metric(MetricMessage::now(MetricId::BAT_VMAX, u16::from_be_bytes(data[5 .. 7].try_into().unwrap()).into()));
            self.send_metric(MetricMessage::now(MetricId::BAT_ID_VMIN, ((data[7] & 0x0F) as u8).into()));
            self.send_metric(MetricMessage::now(MetricId::BAT_ID_VMAX, ((data[7] >> 4) as u8).into()));
        }
        else if requested_function == BmsFunction::GlobalStatus5 as u32 {
            self.send_metric(MetricMessage::now(MetricId::GLOBAL_BAT_CURRENT, i16::from_be_bytes(data[0 .. 2].try_into().unwrap()).into()));
            self.send_metric(MetricMessage::now(MetricId::GLOBAL_CELL_V_MIN, i16::from_be_bytes(data[2 .. 4].try_into().unwrap()).into()));
            self.send_metric(MetricMessage::now(MetricId::GLOBAL_CELL_V_MAX, i16::from_be_bytes(data[4 .. 6].try_into().unwrap()).into()));

            self.send_metric(MetricMessage::now(MetricId::GLOBAL_CELL_V_MIN_ID, ((data[6] & 0x0F) as u8).into()));
            self.send_metric(MetricMessage::now(MetricId::GLOBAL_CELL_V_MAX_ID, ((data[6] >> 4) as u8).into()));
        }
    }
}