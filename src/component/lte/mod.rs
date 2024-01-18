use futures::{StreamExt, SinkExt, stream::SplitSink};
use log::{error, debug, warn, info};
use systemstat::Duration;
use tokio::time::sleep;
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

    pub async fn send_serial_msg(tx: &mut SplitSink<Framed<SerialStream, LineCodec>, String>, msg: &str) {
        let write_result = tx
        .send(msg.to_string())
        .await;
        match write_result {
            Ok(_) => (),
            Err(err) => println!("{:?}", err),
        }
    }

    pub async fn run_thread(metric_sender: MetricSender) {
        let port = match tokio_serial::new(SETTINGS.get::<String>("lte.port").unwrap(), 115_200)
            .open_native_async() {
                Ok(port) => port,
                Err(_e) => {
                    error!("Could not open LTE port. Exiting thread!");
                    return;
                }
            };
        let stream = LineCodec.framed(port);
        let (mut tx, mut rx) = stream.split();

        // Start GPS
        Self::send_serial_msg(&mut tx, "AT+CGPS=1\r").await;

        tokio::spawn(async move {
            loop {
                
                let item = rx
                    .next()
                    .await
                    .expect("Error awaiting future in RX stream.")
                    .expect("Reading stream resulted in an error");
                let cmds = item.split(':').map(|x| x.replace("\r\n", "")).collect::<Vec<String>>();

                if cmds[0].is_empty() || cmds[0] == "OK" {
                    continue;
                }

                match cmds[0].as_str() {
                    // +CPSI: GSM,Online,460-00,0x182d,12401,27 EGSM 900,-64,2110,42-42
                    "+CPSI" => {
                        let cmd_result = cmds[1].trim().split(',').collect::<Vec<&str>>();
                        let network_mode = cmd_result[0];
                        metric_sender.send_now(MessageId::CellularNetworkMode, Value::String(network_mode.to_string())).unwrap();
                    },
                    // +CSQ: 22,0
                    "+CSQ" => {
                        let cmd_result = cmds[1].trim().split(',').collect::<Vec<&str>>();
                        let signal_quality = cmd_result[0].parse::<f32>().unwrap();
                        metric_sender.send_now(MessageId::CellularSignalQuality, Value::Float(signal_quality)).unwrap();
                    },
                    // https://support.micromedia-int.com/hc/en-us/articles/360010426299-GSM-modem-CME-ERRORS-Equipment-Related-GSM-Errors-
                    // +CME ERROR: X (where X is an individual Error Code)
                    "+CME ERROR" => {
                        warn!("CME ERROR: {}", cmds[1]);
                    },
                    d => warn!("Unknown cmd {:?}", d)
                }
            }
        });
        tokio::spawn(async move {
            loop {
                debug!("Querying LTE Network Mode");
                
                Self::send_serial_msg(&mut tx, "AT+CPSI?\r").await;
                sleep(Duration::from_millis(500)).await;

                Self::send_serial_msg(&mut tx, "AT+CSQ\r").await;
                sleep(Duration::from_millis(500)).await;
            }
        });

    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("lte.enabled").unwrap() == true {
            info!("LTE enabled!");

            tokio::spawn(Self::run_thread(self.metric_sender.clone()));
        }
    }
}

