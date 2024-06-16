mod can_messages;
mod read_thread;

use log::info;

use crate::{can::{CanSender, CanReceiver}, helper::MetricSender, SETTINGS};

use self::read_thread::VescReadThread;


pub struct VESC {
    can_sender: CanSender,
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

impl VESC {
    pub fn new(can_sender: CanSender, can_receiver: CanReceiver, metric_sender: MetricSender) -> Self {
        VESC { can_sender, can_receiver, metric_sender }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("vesc.enabled").unwrap() {
            info!("VESC enabled!");
            
            tokio::spawn(VescReadThread::start(self.can_receiver.clone(), self.metric_sender.clone()));
        }
    }
}

