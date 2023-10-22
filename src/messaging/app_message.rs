use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use wannsea_types::types::Metric;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricMessage {
    pub metric: u32,
    pub value: f32,
    pub ts: u128
}

impl MetricMessage {
    pub fn _new(metric: u32, value: f32, ts: u128) -> Self {
        MetricMessage { metric, value, ts }
    }

    pub fn now(metric: Metric, value: f32) -> Self {
        MetricMessage { metric: metric as u32, value, ts: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() }
    }
}

pub type MetricSender = broadcast::Sender<MetricMessage>;