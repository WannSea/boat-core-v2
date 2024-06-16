use log::{warn};
use num_traits::FromPrimitive;
use socketcan::EmbeddedFrame;

use crate::SETTINGS;
use crate::{can::{CanReceiver, get_can_id}, helper::MetricSender};

use super::can_messages::*;

pub struct VescReadThread {
    can_receiver: CanReceiver,
    metric_sender: MetricSender,
    vesc_id: u32,
}

// Read Methods
impl VescReadThread {
    pub async fn start(can_receiver: CanReceiver, metric_sender: MetricSender) {
        let vesc_id = SETTINGS.get::<u32>("vesc.id").unwrap();
        let thread = VescReadThread { can_receiver, metric_sender, vesc_id };
        thread.start_receiving().await;
    }

    async fn start_receiving(&self) {
        let mut receiver = self.can_receiver.subscribe();
        loop {
            let frame = receiver.recv().await.unwrap();

            if frame.dlc() != 8 {
                // VESC Messages will always have 8 bytes
                // therefore we can skip to the next frame
                continue;
            }

            let can_id = get_can_id(frame.id());
            let is_vesc_id = can_id & 0xFF == self.vesc_id;
            if !is_vesc_id {
                continue;
            }
            
            let data = frame.data();
            let msg_id = can_id >> 8;

            match VescMessageIds ::from_u32(msg_id) {
                Some(VescMessageIds::Status1) => StatusMsg1::parse_and_send(data, &self.metric_sender),
                Some(VescMessageIds::Status2) => StatusMsg2::parse_and_send(data, &self.metric_sender),
                Some(VescMessageIds::Status3) => StatusMsg3::parse_and_send(data, &self.metric_sender),
                Some(VescMessageIds::Status4) => StatusMsg4::parse_and_send(data, &self.metric_sender),
                Some(VescMessageIds::Status5) => StatusMsg5::parse_and_send(data, &self.metric_sender),
                _ => {
                    warn!("Unknown VESC Message ID: {}", msg_id);
                }
            }
        }
    }
}