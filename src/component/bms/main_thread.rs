use log::{debug, error};
use socketcan::{StandardId, CanFrame, EmbeddedFrame};
use byteorder::{WriteBytesExt, BigEndian};
use crate::{can::CanSender, component::bms::structs::EmsRequest};

use super::SharedBatteryPacks;


pub struct BmsMainThread {
    battery_packs: SharedBatteryPacks,
    can_sender: CanSender
}

// Write Methods
impl BmsMainThread {
    fn poll_battery_packs(&self) {
        let battery_packs = self.battery_packs.read().unwrap();
        for bat_pack in battery_packs.values() {
            let id = StandardId::new(EmsRequest::BmsIndividualRequest as u16).unwrap();
            let mut data = [0; 8];
            data.as_mut().write_i32::<BigEndian>(bat_pack.serial_number).unwrap();
            debug!("Poll data len: {}", data.len());
            data[7] = 3;

            let frame = CanFrame::new(id, &data).unwrap();
            let _result = self.can_sender.send(frame);
            match _result {
                Ok(_data) => {

                },
                Err(_err) => debug!("Error sending poll bat pack msg")
            }

            debug!("Sent poll bat pack msg");
        }
    }

    async fn aquire_serial_number(&self) {
        let id = StandardId::new(EmsRequest::BmsGeneralRequest as u16).unwrap();
        let frame = CanFrame::new(id, &[129, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        
        loop {
            let _result = self.can_sender.send(frame);
            match _result {
                Ok(_data) => {
                    debug!("Sent serial number acquisition message");
                    return;
                },
                Err(_err) => {
                    error!("Error sending ser number can frame");
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                }
            }
        }
    }

    async fn start_bms_communication_run(&self) {
        let mut ctr = 0;
        loop {
            if ctr == 0 {
                self.aquire_serial_number().await;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            self.poll_battery_packs();

            // Update serial numbers every 10 secs
            ctr = (ctr + 1) % 10;
        }
    }

    pub async fn start(can_sender: CanSender, battery_packs: SharedBatteryPacks) {
        let thread = BmsMainThread { can_sender, battery_packs };
        thread.start_bms_communication_run().await;
    }
}
