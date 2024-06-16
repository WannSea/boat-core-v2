

use futures::{StreamExt, SinkExt};
use log::{debug, info, warn};
use tokio_tungstenite::connect_async;
use wannsea_types::BoatCoreMessage;
use prost::Message;
use crate::{helper::MetricSender, SETTINGS};

use super::metric_queue::MetricQueue;

pub struct WebSocketClient {
    metric_sender: MetricSender,
    cached_messages: MetricQueue<BoatCoreMessage>
} 


impl WebSocketClient {
    pub fn new(metric_sender: MetricSender) -> Self {
        WebSocketClient { metric_sender: metric_sender.clone(), cached_messages: MetricQueue::new(metric_sender.clone()) }
    }

    async fn start_thread(metric_queue: MetricQueue<BoatCoreMessage>) {
        loop {        
            debug!("Trying to connect to ws...");
            let addr = SETTINGS.get::<String>("ws-client.address").unwrap().to_string();
            let retry_timeout = SETTINGS.get::<u64>("ws-client.retry_timeout").unwrap();
            let timeout_dur = tokio::time::Duration::from_millis(retry_timeout);
            let res = tokio::time::timeout(timeout_dur, connect_async(&addr)).await;
            if res.is_err() {
                debug!("Could not reach the WebSocket server at {}. Retrying in {} ms...", &addr, retry_timeout);
                tokio::time::sleep(timeout_dur).await;
                continue;
            }
            info!("WebSocket handshake has been successfully completed");

            let (mut write, _read) = res.unwrap().unwrap().0.split();
            
            loop {
                let msg: BoatCoreMessage = metric_queue.pop().await;
                let mut buf = Vec::new();
                buf.reserve(msg.encoded_len());
                msg.encode(&mut buf).unwrap();
                let send_result = write.send(tokio_tungstenite::tungstenite::Message::Binary(buf)).await;
                match send_result {
                    Ok(_res) => {},
                    Err(_err) => {
                        metric_queue.push(msg).await;
                        break;
                    }
                }
            }
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("ws-client.enabled").unwrap() {
            info!("WebSocket Client enabled!");

            tokio::spawn(Self::start_thread( self.cached_messages.clone()));
        
            let metric_sender = self.metric_sender.clone();
            let metric_queue = self.cached_messages.clone();
            tokio::spawn(async move {
                let mut receiver = metric_sender.subscribe();
                loop {
                    match receiver.recv().await {
                        Ok(msg) => {
                            metric_queue.push(msg).await;
                        },
                        Err(err) => warn!("Error while receiving from Metric Bus: {:?}", err),
                    }
                }
            });
        }


    }
}