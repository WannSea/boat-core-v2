use futures::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use systemstat::Duration;
use tokio::{join, time::sleep};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};
use wannsea_types::MessageId;
use wannsea_types::boat_core_message::Value;

use crate::{helper::{MetricSender, serial_ext::LineCodec, MetricSenderExt}, SETTINGS};

pub struct LTE {
    metric_sender: MetricSender
}

impl LTE {
    pub fn new(metric_sender: MetricSender) -> Self {
        LTE { metric_sender }
    }

    pub async fn send_serial_msg(tx: &mut SplitSink<Framed<SerialStream, LineCodec>, String>, msg: &str) -> Result<(), std::io::Error> {
        tx
        .send(msg.to_string())
        .await
    }

    async fn run_read_thread(mut rx: SplitStream<Framed<SerialStream, LineCodec>>, metric_sender: MetricSender) {
        loop {
            let item = rx
                .next()
                .await
                .expect("Error awaiting future in RX stream.")
                .expect("Reading stream resulted in an error");
            let cmds = item.split(':').map(|x| x.trim().to_string()).collect::<Vec<String>>();
            
            if cmds[0].is_empty() || cmds[0] == "OK" || cmds[0].starts_with("AT+") {
                continue;
            }

            match cmds[0].as_str() {
                // +CPSI: GSM,Online,460-00,0x182d,12401,27 EGSM 900,-64,2110,42-42
                "+CPSI" => {
                    let cmd_result = cmds[1].split(',').collect::<Vec<&str>>();
                    let network_mode = cmd_result[0];
                    metric_sender.send_now(MessageId::CellularNetworkMode, Value::String(network_mode.to_string())).unwrap();
                },
                // +CSQ: 22,0
                "+CSQ" => {
                    let cmd_result = cmds[1].split(',').collect::<Vec<&str>>();
                    let signal_quality = cmd_result[0].parse::<f32>().unwrap();
                    metric_sender.send_now(MessageId::CellularSignalQuality, Value::Float(signal_quality)).unwrap();
                },
                "+CPMUTEMP" => {
                    let temp = cmds[1].parse::<f32>().unwrap();
                    metric_sender.send_now(MessageId::CellularModuleTemp, Value::Float(temp)).unwrap();
                },
                "+CBC" => {
                    let voltage = cmds[1].replace("V", "").parse::<f32>().unwrap();
                    metric_sender.send_now(MessageId::CellularModuleVoltage, Value::Float(voltage)).unwrap();
                },
                // https://support.micromedia-int.com/hc/en-us/articles/360010426299-GSM-modem-CME-ERRORS-Equipment-Related-GSM-Errors-
                // +CME ERROR: X (where X is an individual Error Code)
                _d => {
                    warn!("Unknown LTE module sentence: {:?}", cmds);
                    metric_sender.send_now(MessageId::CellularModuleLogMsg, Value::String(item)).unwrap();
                }
            }
        }
    }

    async fn run_write_thread(mut tx: SplitSink<Framed<SerialStream, LineCodec>, String>) {
        loop {
            trace!("Querying LTE Network Mode");
            
            Self::send_serial_msg(&mut tx, "AT+CPSI?\r").await.unwrap();
            sleep(Duration::from_millis(200)).await;

            Self::send_serial_msg(&mut tx, "AT+CSQ\r").await.unwrap();
            sleep(Duration::from_millis(200)).await;

            Self::send_serial_msg(&mut tx, "AT+CPMUTEMP\r").await.unwrap();
            sleep(Duration::from_millis(200)).await;

            Self::send_serial_msg(&mut tx, "AT+CBC\r").await.unwrap();
            sleep(Duration::from_millis(200)).await;
        }
    }
    
    pub async fn run_thread(metric_sender: MetricSender) {
        loop{
            let port = match tokio_serial::new(SETTINGS.get::<String>("lte.port").unwrap(), 115_200)
                .open_native_async() {
                    Ok(port) => port,
                    Err(_e) => {
                        error!("Could not open LTE port. Retrying...");
                        sleep(Duration::from_millis(1000)).await;
                        continue;
                    }
                };
            let stream = LineCodec.framed(port);
            let (mut tx, rx) = stream.split();

            // Start GPS
            Self::send_serial_msg(&mut tx, "AT+CGPS=1\r").await.unwrap();

            let write_thread = tokio::spawn(Self::run_write_thread(tx));
            let read_thread = tokio::spawn(Self::run_read_thread(rx, metric_sender.clone()));
            let thread_results = join!(write_thread, read_thread);
            if thread_results.0.is_err() {
                warn!("Write thread errored. Trying to reconnect serial port...");
            }
            if thread_results.1.is_err() {
                warn!("Read thread errored. Trying to reconnect serial port...");
            }
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("lte.enabled").unwrap() == true {
            info!("LTE enabled!");

            tokio::spawn(Self::run_thread(self.metric_sender.clone()));
        }
    }
}

