mod helper;
mod can;
mod transport;
mod component;
use component::{computed::sensor_fusion::SensorFusion, gps::GPS, imu::IMU, lte::LTE, pmu::PMU, system_stats::SystemStats, vesc::{self, VESC}};
use config::Config;


use helper::logging::Logger;
use simple_logger::SimpleLogger;
use lazy_static::lazy_static;
use tokio::{sync::broadcast, signal};
use transport::web_socket_client::WebSocketClient;
use wannsea_types::BoatCoreMessage;
use crate::{transport::web_socket_server::WebSocketServer, component::bms::BMS, can::CAN};
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

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("Starting Boat Core v{}", VERSION);

    // Metric bus
    let (metric_sender, _metric_receiver) = broadcast::channel::<BoatCoreMessage>(64);

    let can = CAN::start();

    let logger = Logger::new(metric_sender.clone(), can.receiver.clone());
    logger.start();

    let ws_server = WebSocketServer::new(metric_sender.clone());
    ws_server.start();

    let ws_client = WebSocketClient::new(metric_sender.clone());
    ws_client.start();

    let sensor_fusion = SensorFusion::new(metric_sender.clone());
    sensor_fusion.start();

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

    let imu: IMU = IMU::new(metric_sender.clone());
    imu.start();

    let sensor_fusion: SensorFusion = SensorFusion::new(metric_sender.clone());
    sensor_fusion.start();

    let vesc: VESC = VESC::new(can.sender.clone(), can.receiver.clone(), metric_sender.clone());
    vesc.start();

    signal::ctrl_c().await.unwrap();
}