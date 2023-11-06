use tokio::sync::broadcast;
use wannsea_types::MetricMessage;

pub mod serial_ext;
pub type MetricSender = broadcast::Sender<MetricMessage>;