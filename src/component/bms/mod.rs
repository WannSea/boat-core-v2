mod structs;
mod read_thread;
mod main_thread;

use tokio::sync::mpsc;

use crate::{can::{CanSender, CanReceiver}, helper::MetricSender};

use self::{main_thread::BmsMainThread, read_thread::BmsReadThread, structs::BatteryPack};


pub struct BMS {
    can_sender: CanSender,
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

pub type BatteryPackNotifier = mpsc::Sender<BatteryPack>;
pub type BatteryPackReceiver = mpsc::Receiver<BatteryPack>;

impl BMS {
    pub fn new(can_sender: CanSender, can_receiver: CanReceiver, metric_sender: MetricSender) -> Self {
        BMS { can_sender, can_receiver, metric_sender }
    }

    pub fn start(&self) {
        let (notifier, receiver) = mpsc::channel::<BatteryPack>(16);
        tokio::spawn(BmsMainThread::start(self.can_sender.clone(), receiver));
        tokio::spawn(BmsReadThread::start(self.can_receiver.clone(), self.metric_sender.clone(), notifier));
    }
}

