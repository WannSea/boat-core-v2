
use std::{sync::{Mutex, Arc}, time::{UNIX_EPOCH, SystemTime}};

use futures::{StreamExt, SinkExt};
use log::{info, debug};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{messaging::app_message::{MetricMessage, MetricSender}, SETTINGS};

use super::metric_queue::MetricQueue;

pub struct WebSocketClient {
    metric_sender: MetricSender,
    cached_messages: MetricQueue<MetricMessage>
} 


impl WebSocketClient {
    pub fn new(metric_sender: MetricSender) -> Self {
        WebSocketClient { metric_sender, cached_messages: MetricQueue::new() }
    }

    async fn start_thread(cached_messages: MetricQueue<MetricMessage>) {
        loop {        
            debug!("Trying to connect to ws...");
            let addr = SETTINGS.get::<String>("ws-client.address").unwrap().to_string();
            let res = connect_async(addr).await;
            if res.is_err() {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                continue;
            }
            info!("WebSocket handshake has been successfully completed");
            let (mut write, _read) = res.unwrap().0.split();
            
            loop {
                let msg: MetricMessage = cached_messages.pop().await;
                
                let send_result = write.send(Message::Binary(msg.get_u8())).await;
                match send_result {
                    Ok(_res) => {},
                    Err(_err) => {
                        cached_messages.push(msg).await;
                        break;
                    }
                }
            }
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("ws-client.enabled").unwrap() {
            tokio::spawn(Self::start_thread( self.cached_messages.clone()));
        
            let mut receiver = self.metric_sender.subscribe();
            let sender = self.cached_messages.clone();
            tokio::spawn(async move {
                loop {
                    let msg = receiver.recv().await.unwrap();
                    let stats = sender.stats().await;
                    sender.push(msg).await;
                }
            });
        }


    }
}