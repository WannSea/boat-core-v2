use std::time::{UNIX_EPOCH, SystemTime};

use tokio::sync::broadcast;
use wannsea_types::MetricMessage;


pub mod logging;
pub mod serial_ext;
pub type MetricSender = broadcast::Sender<MetricMessage>;

pub fn get_ts_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

