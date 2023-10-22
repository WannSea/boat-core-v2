mod messaging;
mod can;
mod transport;
mod component;
use component::{system_stats::SystemStats, pmu::PMU, gps::GPS, lte::LTE};
use config::Config;
use log::debug;
use simple_logger::SimpleLogger;
use socketcan::EmbeddedFrame;
use lazy_static::lazy_static;
use tokio::{sync::broadcast, signal};
use transport::web_socket_client::WebSocketClient;
use crate::{messaging::app_message::MetricMessage, transport::web_socket_server::WebSocketServer, component::bms::BMS, can::{CAN, get_can_id}};
use wannsea_types::types;
lazy_static! {
    static ref SETTINGS: Config = Config::builder()
    .add_source(config::File::with_name("config.toml"))
    .add_source(config::Environment::with_prefix("WS"))
    .build()
    .unwrap();
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(log::LevelFilter::Debug).init().unwrap();

    // Metric bus
    let (metric_sender, mut metric_receiver) = broadcast::channel::<MetricMessage>(16);

    let ws_server = WebSocketServer::new(metric_sender.clone());
    ws_server.start();

    let ws_client = WebSocketClient::new(metric_sender.clone());
    ws_client.start();

    let can = CAN::start();
    let mut can_receiver = can.receiver.subscribe();

    // CAN Logger
    tokio::spawn(async move {
        loop {
            let frame = can_receiver.recv().await.unwrap();
            debug!(target: "CAN", "ID: {} LEN: {} DATA: {:X?}", get_can_id(frame.id()), frame.dlc(), frame.data());
        }
    });

    // Metric Logger
    tokio::spawn(async move {
        loop {
            let metric = metric_receiver.recv().await.unwrap();
            debug!(target: "Metric", "{}: {}", metric.metric, metric.value);
        }
    });


    let bms: BMS = BMS::new(can.sender.clone(), can.receiver.clone(), metric_sender.clone());
    bms.start();

    let system_stats = SystemStats::new(metric_sender.clone());
    system_stats.start();

    let pmu = PMU::new(can.receiver.clone(), metric_sender.clone());
    pmu.start();

    let gps = GPS::new(metric_sender.clone());
    gps.start();

    let lte: LTE = LTE::new(metric_sender.clone());
    lte.start();
    

    signal::ctrl_c().await.unwrap();
}