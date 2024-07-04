use std::collections::HashMap;

use log::{debug, error, trace};
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

            let mut data = Vec::new();
            data.extend(bat_pack.serial_number.to_be_bytes());
            data.extend(vec![0, 0, 0, function as u8]);
            
            let can_id = StandardId::new(EmsRequest::BmsIndividualRequest as u16).unwrap();
            let frame = CanFrame::new(can_id, &data).unwrap();
            match  self.can_sender.send(frame) {
                Ok(_data) => trace!("Sent individual request with function {:?} for pack {} !", function, bat_pack.id),
                Err(_err) => debug!("Error sending poll bat pack msg")
            }
        }
    }


    async fn aquire_serial_number(&self) {
        let id = StandardId::new(EmsRequest::BmsGeneralRequest as u16).unwrap();
        let frame = CanFrame::new(id, &[129, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        
        match self.can_sender.send(frame) {
            Ok(_data) => trace!("Sent serial number acquisition message"),
            Err(_err) => error!("Error sending ser number can frame")
        }
    }

    async fn start_bms_communication_run(&self, mut pack_receiver: BatteryPackReceiver) {
        let mut battery_packs: BatteryPacks = HashMap::new();
        let request_interval = SETTINGS.get::<u64>("bms.request_interval").unwrap();
        let bms_search_interval = SETTINGS.get::<u64>("bms.search_interval").unwrap();

        let mut last_searched: u128 = 0;
        loop {
            // Check if read thread has notified us about new packs
            while let Ok(pack) = pack_receiver.try_recv() {
                if !battery_packs.contains_key(&pack.id) {
                    debug!("Found new battery pack with id: {}, serial_number: {}, part_number: {}", pack.id, pack.serial_number, pack.part_number);
                    battery_packs.insert(pack.id, pack);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(request_interval / 2)).await;
            self.request_all_packs(&battery_packs, BmsIndividualRequestFunction::AllMeasurements);
            tokio::time::sleep(tokio::time::Duration::from_millis(request_interval / 2)).await;
            self.request_all_packs(&battery_packs, BmsIndividualRequestFunction::InternalStatus1);

            if get_ts_ms() - last_searched > bms_search_interval as u128 {
                self.aquire_serial_number().await;
                last_searched = get_ts_ms();
            }
         
        }
    }

    pub async fn start(can_sender: CanSender, pack_receiver: BatteryPackReceiver) {
        let thread = BmsMainThread { can_sender };
        thread.start_bms_communication_run(pack_receiver).await;
    }
}