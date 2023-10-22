use log::{error, trace};
use systemstat::Duration;

use crate::{messaging::{app_message::MetricSender}, SETTINGS};

pub struct LTE {
    metric_sender: MetricSender
}

impl LTE {
    pub fn new(metric_sender: MetricSender) -> Self {
        LTE { metric_sender }
    }

    pub async fn run_thread(metric_sender: MetricSender) {
        let port = match tokio_serial::new(SETTINGS.get::<String>("lte.port").unwrap(), 115_200)
            .timeout(Duration::from_millis(10))
            .open() {
                Ok(port) => port,
                Err(_e) => {
                    error!("Could not open LTE port. Exiting thread!");
                    return;
                }
            };

        // let line = read_line(port);
        // trace!("Read LTE line {}", line);
    }

    pub fn start(&self) {
        //tokio::spawn(Self::run_thread(self.metric_sender.clone()));
    }
}

