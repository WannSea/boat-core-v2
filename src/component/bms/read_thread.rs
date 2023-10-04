use byteorder::{BigEndian, ReadBytesExt};
use socketcan::EmbeddedFrame;

use crate::{can::{CanReceiver, get_can_id}, messaging::app_message::{MetricSender, MetricMessage}};

use super::{SharedBatteryPacks, structs::{BmsFunction, BatteryPack}};

pub struct BmsReadThread {
    battery_packs: SharedBatteryPacks,
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

// Read Methods
impl BmsReadThread {
    pub async fn start(can_receiver: CanReceiver, battery_packs: SharedBatteryPacks, metric_sender: MetricSender) {
        let thread = BmsReadThread { can_receiver, battery_packs, metric_sender };
        thread.start_receiving().await;
    }

    fn send_metric(&self, name: String, data: f32) {
         self.metric_sender.send(MetricMessage::now(name, data)).expect("Could not report metric");
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
            
            let mut batt_packs = self.battery_packs.write().unwrap();
            let bms_requested_fun = id & 0x0FFF;

            if bms_requested_fun == BmsFunction::BmsIdSerialNumberAnswer as u32 {
                let pack = BatteryPack {
                    id: bms_id,
                    serial_number: (&data[0..4]).read_i32::<BigEndian>().unwrap(),
                    part_number: (&data[4..8]).read_i32::<BigEndian>().unwrap()
                };
                batt_packs.insert(bms_id, pack);
            } 
            else if bms_requested_fun >= BmsFunction::BmsIdV01_04 as u32 && bms_requested_fun <= BmsFunction::BmsIdV17_20 as u32 {
                let base_index = (bms_requested_fun as usize - 2) * 4;

                for idx in 0..4 {
                    let data_start_idx = idx * 4;
                    let data = (&data[data_start_idx..(data_start_idx + 4)]).read_u32::<BigEndian>().unwrap();
                    self.send_metric(format!("BAT_{}_V_{}", bms_id, base_index + idx), data as f32);
                }
            }
            else if bms_requested_fun == BmsFunction::BmsIdV21_24 as u32 {
                self.send_metric(format!("BAT_{}_AH_DISCHARGED", bms_id), (&data[0..2]).read_u16::<BigEndian>().unwrap() as f32);

                self.send_metric(format!("BAT_{}_REMAINING_CAPACITY", bms_id),  (&data[2..4]).read_u16::<BigEndian>().unwrap() as f32);
                 self.send_metric(format!("BAT_{}_SOH", bms_id), data[4] as f32);
                 self.send_metric(format!("BAT_{}_SOC", bms_id),  data[5] as f32);
                 self.send_metric(format!("BAT_{}_I", bms_id), ((&data[6..8]).read_u16::<BigEndian>().unwrap() as f32) * 0.1);
            }
            else if bms_requested_fun == BmsFunction::BmsIdT01_06 as u32 {
                 self.send_metric(format!("BAT_{}_T_CELL_0", bms_id), data[0] as f32 - 40.0);
                 self.send_metric(format!("BAT_{}_T_CELL_1", bms_id), data[1] as f32 - 40.0);
                 self.send_metric(format!("BAT_{}_T_CELL_2", bms_id), data[2] as f32 - 40.0);
            }
            else if bms_requested_fun == BmsFunction::BmsIdInternalStatus1 as u32 {
                 self.send_metric(format!("BAT_{}_MAJOR_ALERT_1", bms_id), (&data[0..1]).read_u8().unwrap() as f32);
                 self.send_metric(format!("BAT_{}_MAJOR_ALERT_2", bms_id), (&data[1..2]).read_u8().unwrap() as f32);
                 self.send_metric(format!("BAT_{}_MAJOR_ALERT_3", bms_id), (&data[2..3]).read_u8().unwrap() as f32);
                 self.send_metric(format!("BAT_{}_MINOR_ALERT", bms_id), (&data[3..4]).read_u8().unwrap() as f32);
            }
        }
    }

    fn parse_bms_master_message(&self, id: u32, data: &[u8]) {
        let requested_function = id & 0x0FFF;

        if requested_function == BmsFunction::EmsControl as u32 {
             self.send_metric(format!("MAX_BAT_DISCHARGE_I"), (&data[0 .. 2]).read_u16::<BigEndian>().unwrap() as f32);
             self.send_metric(format!("MAX_BAT_RECHARGE_I"), (&data[2 .. 4]).read_u16::<BigEndian>().unwrap() as f32);
        }
        else if requested_function == BmsFunction::GlobalStatus3 as u32 {
             self.send_metric(format!("GLOBAL_SOC"), data[0] as f32);
             self.send_metric(format!("ID_GLOBAL_SOC"), (data[1] >> 4) as f32);
             self.send_metric(format!("GLOBAL_I_BMS_ALARM_STATE"), (data[2] & 0x03) as f32);
             self.send_metric(format!("NUMBER_OF_CONNECTED_BMS"), (data[2] >> 4) as f32);
             self.send_metric(format!("POWERBUS_INFORMATION"), data[3] as f32);
        }
        else if requested_function == BmsFunction::GlobalStatus4 as u32 {
             self.send_metric(format!("BAT_T_MIN"), data[0] as f32 - 40.0);
             self.send_metric(format!("BAT_T_MAX"), data[1] as f32 - 40.0);
             self.send_metric(format!("BAT_ID_T_MIN"), (data[2] & 0x0F) as f32);
             self.send_metric(format!("BAT_ID_T_MAX"), (data[2] >> 4) as f32);

             self.send_metric(format!("BAT_V_MIN"), (&data[3 .. 5]).read_u16::<BigEndian>().unwrap() as f32 * 0.01);
             self.send_metric(format!("BAT_V_MAX"), (&data[5 .. 7]).read_u16::<BigEndian>().unwrap() as f32 * 0.01);
             self.send_metric(format!("BAT_ID_V_MIN"), (data[7] & 0x0F) as f32);
             self.send_metric(format!("BAT_ID_V_MAX"), (data[7] >> 4) as f32);
        }
        else if requested_function == BmsFunction::GlobalStatus5 as u32 {
             self.send_metric(format!("GLOBAL_BAT_CURRENT"), (&data[0..2]).read_i16::<BigEndian>().unwrap() as f32 * 0.01);
             self.send_metric(format!("GLOBAL_CELL_MIN"), (&data[2..4]).read_i16::<BigEndian>().unwrap() as f32 * 1e-3);
             self.send_metric(format!("GLOBAL_CELL_MAX"), (&data[4..6]).read_i16::<BigEndian>().unwrap() as f32 * 1e-3);
             self.send_metric(format!("GLOBAL_CELL_MIN_ID"), (data[6] & 0x0F) as f32);
             self.send_metric(format!("GLOBAL_CELL_MAX_ID"), (data[6] >> 4) as f32);
        }
    }
}