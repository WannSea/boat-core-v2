use futures::StreamExt;
use log::info;
use tokio_tungstenite::connect_async;

use crate::{messaging::app_message::{MetricMessage, MetricSender}, SETTINGS};

pub struct WebSocketClient {
    metric_sender: MetricSender
} 


impl WebSocketClient {
    pub fn new(metric_sender: MetricSender) -> Self {
        WebSocketClient { metric_sender }
    }

    async fn start_thread(metric_sender: MetricSender) {
        let addr = SETTINGS.get::<String>("ws-client.address").unwrap().to_string();
        let (ws_stream, _) = connect_async(addr).await.expect("Failed to connect");
        info!("WebSocket handshake has been successfully completed");
        let (write, read) = ws_stream.split();
        
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("ws-client.enabled").unwrap() {
            tokio::spawn(Self::start_thread(self.metric_sender.clone()));
        }
    }
}