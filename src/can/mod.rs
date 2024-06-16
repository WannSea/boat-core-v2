pub mod ids;

use futures::StreamExt;
use log::{error, info};
use tokio::sync::broadcast;
use socketcan::{tokio::CanSocket, CanFrame, Id};

use crate::SETTINGS;

pub type CanSender = broadcast::Sender<CanFrame>;
pub type CanReceiver = broadcast::Sender<CanFrame>;

pub struct CAN {
    pub sender: CanSender,
    pub receiver: CanReceiver
}

pub fn get_can_id(id: Id) -> u32 {
    match id {
        socketcan::Id::Standard(id) => id.as_raw() as u32,
        socketcan::Id::Extended(id) => id.as_raw(),
    }
}

impl CAN {
    async fn can_to_rx(interface: String, receiver_tx: CanReceiver) {
        let mut sock_rx = match CanSocket::open(&interface) {
            Ok(port) => port,
            Err(_err) => {
                error!("CAN Device not accessible");
                return;
            }
        };
        while let Some(Ok(frame)) = sock_rx.next().await {
            receiver_tx.send(frame).unwrap();
        }
    }

    async fn tx_to_can(interface: String, sender_rx: CanSender) {
        let sock_tx =  match CanSocket::open(&interface) {
            Ok(port) => port,
            Err(_err) => {
                error!("CAN Device not accessible");
                return;
            }
        };
        let mut rx = sender_rx.subscribe();
        loop {
            let msg = rx.recv().await.unwrap();
            let _res = sock_tx.write_frame(msg).unwrap().await;
        }
    }

    pub fn start() -> Self {
        let (receiver, _receiver_rx) = broadcast::channel::<CanFrame>(64);
        let (sender, _sender_rx) = broadcast::channel::<CanFrame>(64);
        
        if SETTINGS.get::<bool>("can.enabled").unwrap() {
            info!("CAN enabled!");
            let interface = SETTINGS.get::<String>("can.interface").unwrap();
            tokio::spawn(Self::can_to_rx(interface.clone(), receiver.clone()));
            tokio::spawn(Self::tx_to_can(interface, sender.clone()));
        }

        CAN { sender, receiver }
    }

}