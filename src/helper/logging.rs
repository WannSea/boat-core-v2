use log::{debug, warn};

use socketcan::EmbeddedFrame;
use crate::{can::{CanReceiver, get_can_id}, SETTINGS};

use super::MetricSender;

pub struct Logger {
    can_receiver: CanReceiver,
    metric_sender: MetricSender
}

impl Logger {
    pub async fn log_can(can_receiver: CanReceiver) {
        let mut can_receiver = can_receiver.subscribe();
        loop {
            let frame = can_receiver.recv().await.unwrap();
            debug!(target: "CAN", "ID: {:X?} LEN: {} DATA: {:X?}", get_can_id(frame.id()), frame.dlc(), frame.data());
        }
    }
    
    pub async fn log_metrics(metric_sender: MetricSender) {
        let mut metric_receiver = metric_sender.subscribe();
        loop { 
            match metric_receiver.recv().await {
                Ok(metric) => {
                    debug!(target: "Metric", "{} {:?}", metric.id().as_str_name(), metric.value.unwrap());
                },
                Err(err) => warn!("Error while receiving from Metric Bus: {:?}", err),
            }
        }
    }

    pub fn new(metric_sender: MetricSender, can_receiver: CanReceiver) -> Self {
        Logger { metric_sender, can_receiver }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("logging.can").unwrap() {
            tokio::spawn(Self::log_can(self.can_receiver.clone()));
        }
        if SETTINGS.get::<bool>("logging.metrics").unwrap() {
            tokio::spawn(Self::log_metrics(self.metric_sender.clone()));
        }

    }
}

