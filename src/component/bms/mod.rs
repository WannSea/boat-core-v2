mod structs;
mod read_thread;
mod main_thread;
use std::{sync::{Arc, RwLock}, collections::HashMap};
use crate::{can::{CanSender, CanReceiver}, messaging::app_message::MetricSender};

use self::{structs::BatteryPack, main_thread::BmsMainThread, read_thread::BmsReadThread};

type SharedBatteryPacks = Arc<RwLock<HashMap<u8, BatteryPack>>>;

pub struct BMS {
    can_sender: CanSender,
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

impl BMS {
    pub fn new(can_sender: CanSender, can_receiver: CanReceiver, metric_sender: MetricSender) -> Self {
        BMS { can_sender, can_receiver, metric_sender }
    }

    pub fn start(&self) {
        let battery_packs: SharedBatteryPacks = Arc::new(RwLock::new(HashMap::new()));
        tokio::spawn(BmsMainThread::start(self.can_sender.clone(), battery_packs.clone()));
        tokio::spawn(BmsReadThread::start(self.can_receiver.clone(), battery_packs.clone(), self.metric_sender.clone()));
    }
}

