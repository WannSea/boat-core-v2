use futures::{StreamExt, SinkExt};
use log::info;
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast};
use tokio_tungstenite::tungstenite::Message;
use wannsea_types::BoatCoreMessage;

use crate::{SETTINGS, helper::MetricSender};

pub struct WebSocketServer {
    message_bus: MetricSender
} 

fn handle_client(stream: TcpStream, addr: String, metric_bus: MetricSender) {
    tokio::spawn(async move {
        let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
        info!("WebSocket connection established: {}", addr);

        let (mut out, _inc) = ws_stream.split();

        // Message bus to ws
        let mut receiver = metric_bus.subscribe();
        loop {
            let msg = receiver.recv().await.unwrap();
            // if let Ok(data) = msg.get_json_repr() {
            //     out.send(Message::text(data)).await.unwrap();
            // }
        }
    });
}

impl WebSocketServer {
    pub fn new(message_bus: MetricSender) -> Self {
        WebSocketServer { message_bus }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("ws-server.enabled").unwrap() {
            info!("WebSocket Server enabled!");

            let message_bus = self.message_bus.clone();
            tokio::spawn(async move {
                let addr = SETTINGS.get::<String>("ws-server.address").unwrap();
                let try_socket = TcpListener::bind(&addr).await;
                let listener = try_socket.expect("Failed to bind");
                info!("Listening on: {}", addr);
    
                // Let's spawn the handling of each connection in a separate task.
                while let Ok((stream, addr)) = listener.accept().await {       
                   handle_client(stream, addr.to_string(), message_bus.clone());
                }
            });
        }
        
    }
}