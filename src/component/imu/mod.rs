use log::info;
use crate::{helper::MetricSender, SETTINGS};


pub struct IMU {
    metric_sender: MetricSender
}

impl IMU {
    pub fn new(metric_sender: MetricSender) -> Self {
        IMU { metric_sender }
    }

    pub async fn run_thread(metric_sender: MetricSender) {

    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("imu.enabled").unwrap() == true {
            info!("IMU enabled!");

            tokio::spawn(Self::run_thread(self.metric_sender.clone()));
        }
    }
}

