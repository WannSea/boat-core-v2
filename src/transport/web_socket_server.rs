use futures::{StreamExt, SinkExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use wannsea_types::{BoatCoreMessage, MetricId};

use crate::{SETTINGS, helper::MetricSender};

pub struct WebSocketServer {
    message_bus: MetricSender
} 

fn get_json_value(msg: &BoatCoreMessage) -> String {
    match msg.clone().value.unwrap() {
        wannsea_types::boat_core_message::Value::Double(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Float(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Int32(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Int64(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Uint32(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Uint64(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Sint32(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Sint64(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Fixed32(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Fixed64(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Sfixed32(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Sfixed64(x) => x.to_string(),
        wannsea_types::boat_core_message::Value::Bool(x) => if x { "\"true\"".to_string() } else { "\"false\"".to_string() },
        wannsea_types::boat_core_message::Value::String(x) => x,
        wannsea_types::boat_core_message::Value::Bytes(_) => "Bytes unsupported!".to_string(),
    }
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
            let json = format!("{{ \"ts\": {}, \"cat\": \"{}\", \"value\": {} }}", msg.timestamp, MetricId::from_repr(msg.cat as usize).unwrap(), get_json_value(&msg));
            out.send(Message::Text(json)).await.unwrap();
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