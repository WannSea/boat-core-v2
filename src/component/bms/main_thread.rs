use std::collections::HashMap;

use log::{debug, error, trace, warn};
use num_traits::ToBytes;
use socketcan::{StandardId, CanFrame, EmbeddedFrame, ExtendedId};
use crate::{can::CanSender, component::bms::structs::EmsRequest, SETTINGS, helper::get_ts_ms};

use super::{structs::{BmsIndividualRequestFunction, BatteryPack}, BatteryPackReceiver};


pub type BatteryPacks = HashMap<u8, BatteryPack>;

pub struct BmsMainThread {
    can_sender: CanSender,
}

// Write Methods
impl BmsMainThread {
    // Request specific individual request for all packs
    fn request_all_packs(&self, battery_packs: &BatteryPacks, function: BmsIndividualRequestFunction) {
        for bat_pack in battery_packs.values() {
            let id = ((bat_pack.id as u32) << 12) | (EmsRequest::BmsIndividualRequest as u32);

            let mut data = Vec::new();
            data.extend(bat_pack.serial_number.to_be_bytes());
            data.extend(vec![0, 0, 0, function as u8]);

            let frame = CanFrame::new(ExtendedId::new(id).unwrap(), &data).unwrap();
            match  self.can_sender.send(frame) {
                Ok(_data) => trace!("Sent individual request with function {:?} for every pack!", function),
                Err(_err) => debug!("Error sending poll bat pack msg")
            }
        }
    }


    async fn aquire_serial_number(&self) {
        let id = StandardId::new(EmsRequest::BmsGeneralRequest as u16).unwrap();
        let frame = CanFrame::new(id, &[129, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        
        match self.can_sender.send(frame) {
            Ok(_data) => debug!("Sent serial number acquisition message"),
            Err(_err) => error!("Error sending ser number can frame")
        }
    }

    async fn start_bms_communication_run(&self, mut pack_receiver: BatteryPackReceiver) {
        let mut battery_packs: BatteryPacks = HashMap::new();
        let request_interval = SETTINGS.get::<u64>("bms.request_interval").unwrap();
        let bms_search_interval = SETTINGS.get::<u64>("bms.search_interval").unwrap();

        let mut last_searched: u128 = 0;
        let mut number_of_packs: u8 = 0;
        let mut reconfigurations = 0;
        loop {
            let mut count_packs: u8 = 0;    
            // Check if read thread has notified us about new packs
            while let Ok(pack) = pack_receiver.try_recv() {
                count_packs += 1;
                if !battery_packs.contains_key(&pack.id) {
                    // battery_packs does not contain id: add it!
                    debug!("Found new battery pack with id: {}, serial_number: {}, part_number: {}", pack.id, pack.serial_number, pack.part_number);
                    battery_packs.insert(pack.id, pack);
                }
            }
            // check if the number of packs has changed
            if number_of_packs != count_packs
            {   // configuration changed
                self.configure_all_packs(&battery_packs);
                reconfigurations += 1;
                if reconfigurations >= 2
                {
                    warn!("Reconfigured battery packs {} times, something might be wrong!", reconfigurations)
                }
            }
            number_of_packs = count_packs;

            tokio::time::sleep(tokio::time::Duration::from_millis(request_interval / 3)).await;
            self.request_all_packs(&battery_packs, BmsIndividualRequestFunction::AllMeasurements);
            tokio::time::sleep(tokio::time::Duration::from_millis(request_interval / 3)).await;
            self.request_all_packs(&battery_packs, BmsIndividualRequestFunction::InternalStatus1);

            if get_ts_ms() - last_searched > bms_search_interval as u128 {
                self.aquire_serial_number().await;
                last_searched = get_ts_ms();
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(request_interval / 3)).await;         
        }
    }

    fn configure_all_packs(&self, battery_packs: &BatteryPacks )
    {
        let mut id_to_set: u8 = 1;
        let total_bms_count: u8 = battery_packs.len().try_into().unwrap();
        for bat_pack in battery_packs.values()
        {
            self.configure_pack(bat_pack.serial_number, id_to_set, total_bms_count);
            id_to_set += 1;
        }
    }

    fn configure_pack(&self, serial_number: u32, id_to_set: u8, total_bms_count: u8)
    {
        let id = StandardId::new(EmsRequest::BmsIndividualRequest as u16).unwrap();
        let sn_bytes = serial_number.to_le_bytes();
        let frame = CanFrame::new(id, &[sn_bytes[0], sn_bytes[1], sn_bytes[2], sn_bytes[3], id_to_set, total_bms_count, 0, 0x11]).unwrap();
        match self.can_sender.send(frame){
            Ok(_data) => debug!("Sent configuration to bms with serial_number {}", serial_number),
            Err(_err) => error!("Failed to send config to bms")
        }
    }

    pub async fn start(can_sender: CanSender, pack_receiver: BatteryPackReceiver) {
        let thread = BmsMainThread { can_sender };
        thread.start_bms_communication_run(pack_receiver).await;
    }
}
