use std::time::{UNIX_EPOCH, SystemTime};

use tokio::sync::broadcast;

use wannsea_types::MessageId;
use wannsea_types::BoatCoreMessage;
use wannsea_types::boat_core_message::Value;

pub mod logging;
pub mod serial_ext;
pub type MetricSender = broadcast::Sender<BoatCoreMessage>;

pub trait MetricSenderExt {
    fn send_now(&self, id: MessageId, value: Value) -> Result<usize, tokio::sync::broadcast::error::SendError<BoatCoreMessage>>;
}

impl MetricSenderExt for MetricSender {
    fn send_now(&self, id: MessageId, value: Value) -> Result<usize, tokio::sync::broadcast::error::SendError<BoatCoreMessage>> {
        let mut msg = wannsea_types::BoatCoreMessage::default();
        msg.set_id(id);
        msg.timestamp = get_ts_ms().try_into().unwrap();
        msg.value = Some(value);
        self.send(msg)
    }
}

pub fn get_ts_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

