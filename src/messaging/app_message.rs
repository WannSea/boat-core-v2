use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use wannsea_types::types::Metric;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricMessage {
    pub id: Metric,
    pub ts: u128,
    pub value: Vec<u8>
}

impl MetricMessage {
    pub fn now(id: Metric, value: Vec<u8>) -> Self {
        MetricMessage { id: id, value, ts: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() }
    }

    pub fn now_f32(id: Metric, value: f32) -> Self {
        Self::now(id, value.to_ne_bytes().to_vec())
    }

    pub fn now_str(id: Metric, value: &str) -> Self {
        Self::now(id, value.as_bytes().to_vec())
    }

    pub fn get_u8(&self) -> Vec<u8> {
        let mut out_vec = Vec::new();
        out_vec.push(self.id.clone() as u8);
        out_vec.append(self.ts.to_ne_bytes().to_vec().as_mut());
        out_vec.append(self.value.to_vec().as_mut());
        return out_vec;
    }
}

pub type MetricSender = broadcast::Sender<MetricMessage>;