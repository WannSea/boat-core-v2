use futures::{StreamExt, SinkExt};
use log::{info, error};
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncRead, AsyncWrite}};
use tokio_tungstenite::{tungstenite::{Message, self, handshake::server::{Request, Response, ErrorResponse}}, WebSocketStream};
use crate::{SETTINGS, helper::MetricSender};

pub struct WebSocketServer {
    message_bus: MetricSender
} 
async fn handle_raw_socket<T: AsyncRead + AsyncWrite + Unpin>(
    socket: T
) -> (Result<WebSocketStream<T>, tungstenite::Error>, Option<String>) {
    let mut path = None;
    let callback = |req: &Request, res: Response| -> Result<Response, ErrorResponse> {
        path = Some(req.uri().path().to_string());
        Ok(res)
    };
    (tokio_tungstenite::accept_hdr_async(socket, callback).await, path)
}


async fn handle_client(path: String, stream: WebSocketStream<TcpStream>, metric_bus: MetricSender) {   
    tokio::spawn(async move {
        info!("WebSocket connection established: {}", path);

        let (mut out, _inc) = stream.split();

        // Message bus to ws
        let mut receiver = metric_bus.subscribe();
        loop {
            let msg: wannsea_types::BoatCoreMessage = receiver.recv().await.unwrap();

            if path.to_lowercase() == "/" || msg.id().as_str_name() == &path[1..] {
                let json = serde_json::to_string(&msg).unwrap();
                out.send(Message::Text(json)).await.unwrap();
            }

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
                while let Ok((stream, _addr)) = listener.accept().await {       
                    let ws = handle_raw_socket(stream).await;
                    match ws {
                        (Ok(ws), Some(path)) =>  {
                            println!("WS CONNECT {}", path);
                            handle_client(path, ws, message_bus.clone()).await;
                        },
                        _ => error!("Ws Connect error")
                    }
                }
            });
        }
        
    }
}