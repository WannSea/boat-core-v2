use byteorder::{BigEndian, ReadBytesExt};
use socketcan::EmbeddedFrame;
use wannsea_types::types::Metric;

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

    fn send_metric(&self, metric: Metric, data: f32) {
         self.metric_sender.send(MetricMessage::now_f32(metric, data)).expect("Could not report metric");
    }

    fn send_metric_dynamic_name(&self, name: String, data: f32) {
        self.send_metric(name.parse().unwrap(), data);
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
                    self.send_metric_dynamic_name(format!("BAT_{}_V_{}", bms_id, base_index + idx), data as f32);
                }
            }
            else if bms_requested_fun == BmsFunction::BmsIdV21_24 as u32 {
                self.send_metric_dynamic_name(format!("Bat{}AhDischarged", bms_id), (&data[0..2]).read_u16::<BigEndian>().unwrap() as f32);

                self.send_metric_dynamic_name(format!("Bat{}RemainingCapacity", bms_id),  (&data[2..4]).read_u16::<BigEndian>().unwrap() as f32);
                self.send_metric_dynamic_name(format!("Bat{}Soh", bms_id), data[4] as f32);
                self.send_metric_dynamic_name(format!("Bat{}Soc", bms_id),  data[5] as f32);
                self.send_metric_dynamic_name(format!("Bat{}IBatI", bms_id), ((&data[6..8]).read_u16::<BigEndian>().unwrap() as f32) * 0.1);
            }
            else if bms_requested_fun == BmsFunction::BmsIdT01_06 as u32 {
                self.send_metric_dynamic_name(format!("Bat{}T0", bms_id), data[0] as f32 - 40.0);
                self.send_metric_dynamic_name(format!("Bat{}T1", bms_id), data[1] as f32 - 40.0);
                self.send_metric_dynamic_name(format!("Bat{}T2", bms_id), data[2] as f32 - 40.0);
            }
            else if bms_requested_fun == BmsFunction::BmsIdInternalStatus1 as u32 {
                self.send_metric_dynamic_name(format!("Bat{}MajorAlert1", bms_id), (&data[0..1]).read_u8().unwrap() as f32);
                self.send_metric_dynamic_name(format!("Bat{}MajorAlert2", bms_id), (&data[1..2]).read_u8().unwrap() as f32);
                self.send_metric_dynamic_name(format!("Bat{}MajorAlert3", bms_id), (&data[2..3]).read_u8().unwrap() as f32);
                self.send_metric_dynamic_name(format!("Bat{}MinorAlert", bms_id), (&data[3..4]).read_u8().unwrap() as f32);
            }
        }
    }

    fn parse_bms_master_message(&self, id: u32, data: &[u8]) {
        let requested_function = id & 0x0FFF;

        if requested_function == BmsFunction::EmsControl as u32 {
            self.send_metric(Metric::MaxBatteryDischargeCurrent, (&data[0 .. 2]).read_u16::<BigEndian>().unwrap() as f32);
            self.send_metric(Metric::MaxBatteryRechargeCurrent, (&data[2 .. 4]).read_u16::<BigEndian>().unwrap() as f32);
        }
        else if requested_function == BmsFunction::GlobalStatus3 as u32 {
            self.send_metric(Metric::GlobalSoc, data[0] as f32);
            self.send_metric(Metric::IdGlobalSoc, (data[1] >> 4) as f32);
            self.send_metric(Metric::GlobalIbmsAlarmState, (data[2] & 0x03) as f32);
            self.send_metric(Metric::NumberOfConnectedBms, (data[2] >> 4) as f32);
            self.send_metric(Metric::PowerbusInformation, data[3] as f32);
        }
        else if requested_function == BmsFunction::GlobalStatus4 as u32 {
            self.send_metric(Metric::BatTmin, data[0] as f32 - 40.0);
            self.send_metric(Metric::BatTmax, data[1] as f32 - 40.0);
            self.send_metric(Metric::BatIdTmin, (data[2] & 0x0F) as f32);
            self.send_metric(Metric::BatIdTmax, (data[2] >> 4) as f32);

            self.send_metric(Metric::BatVmin, (&data[3 .. 5]).read_u16::<BigEndian>().unwrap() as f32 * 0.01);
            self.send_metric(Metric::BatVmax, (&data[5 .. 7]).read_u16::<BigEndian>().unwrap() as f32 * 0.01);
            self.send_metric(Metric::BatIdVmin, (data[7] & 0x0F) as f32);
            self.send_metric(Metric::BatIdVmax, (data[7] >> 4) as f32);
        }
        else if requested_function == BmsFunction::GlobalStatus5 as u32 {
            self.send_metric(Metric::GlobalBatCurrent, (&data[0..2]).read_i16::<BigEndian>().unwrap() as f32 * 0.01);
            self.send_metric(Metric::GlobalCellMin, (&data[2..4]).read_i16::<BigEndian>().unwrap() as f32 * 1e-3);
            self.send_metric(Metric::GlobalCellMax, (&data[4..6]).read_i16::<BigEndian>().unwrap() as f32 * 1e-3);
            self.send_metric(Metric::GlobalCellMinId, (data[6] & 0x0F) as f32);
            self.send_metric(Metric::GlobalCellMaxId, (data[6] >> 4) as f32);
        }
    }
}