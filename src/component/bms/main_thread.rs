use std::collections::HashMap;

use log::{debug, error, trace};
use socketcan::{StandardId, CanFrame, EmbeddedFrame};
use crate::{can::CanSender, component::bms::structs::EmsRequest};

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
            let id = StandardId::new(EmsRequest::BmsIndividualRequest as u16).unwrap();

            let mut data = Vec::new();
            data.extend(bat_pack.serial_number.to_be_bytes());
            data.extend(vec![0, 0, 0, function as u8]);

            let frame = CanFrame::new(id, &data).unwrap();
            match  self.can_sender.send(frame) {
                Ok(_data) => trace!("Sent individual request with function {} for every pack!", function),
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
        loop {
            // Check if read thread has notified us about new packs
            while let Ok(pack) = pack_receiver.try_recv() {
                battery_packs.insert(pack.id, pack);
            }

            self.aquire_serial_number().await;            
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            self.request_all_packs(&battery_packs, BmsIndividualRequestFunction::AllMeasurements);
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            self.request_all_packs(&battery_packs, BmsIndividualRequestFunction::InternalStatus1);
        }
    }

    pub async fn start(can_sender: CanSender, pack_receiver: BatteryPackReceiver) {
        let thread = BmsMainThread { can_sender };
        thread.start_bms_communication_run(pack_receiver).await;
    }
}
