use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use wannsea_types::types::Metric;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricMessage {
    pub id: u32,
    pub value: f32,
    pub ts: u128
}

impl MetricMessage {
    pub fn _new(id: u32, value: f32, ts: u128) -> Self {
        MetricMessage { id, value, ts }
    }

    pub fn now(id: Metric, value: f32) -> Self {
        MetricMessage { id: id as u32, value, ts: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() }
    }

    pub fn get_u8(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

pub type MetricSender = broadcast::Sender<MetricMessage>;