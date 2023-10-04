use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricMessage {
    pub name: String,
    pub value: f32,
    pub ts: u128
}

impl MetricMessage {
    pub fn _new(name: String, value: f32, ts: u128) -> Self {
        MetricMessage { name, value, ts }
    }

    pub fn now(name: String, value: f32) -> Self {
        MetricMessage { name, value, ts: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() }
    }
}

pub type MetricSender = broadcast::Sender<MetricMessage>;