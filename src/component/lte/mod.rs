use futures::{StreamExt, SinkExt};
use log::{error, debug, warn, info};
use systemstat::Duration;
use tokio::time::sleep;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;
use wannsea_types::MetricId;
use wannsea_types::boat_core_message::Value;

use crate::{helper::{MetricSender, serial_ext::LineCodec, MetricSenderExt}, SETTINGS};

pub struct LTE {
    metric_sender: MetricSender
}

impl LTE {
    pub fn new(metric_sender: MetricSender) -> Self {
        LTE { metric_sender }
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
        
        tokio::spawn(async move {
            loop {
                
                let item = rx
                    .next()
                    .await
                    .expect("Error awaiting future in RX stream.")
                    .expect("Reading stream resulted in an error");
                let cmd = item.split(':').collect::<Vec<&str>>();
                match cmd[0] {
                    // +CPSI: GSM,Online,460-00,0x182d,12401,27 EGSM 900,-64,2110,42-42
                    "+CPSI" => {
                        let cmd_result = cmd[1].trim().split(',').collect::<Vec<&str>>();
                        let network_mode = cmd_result[0];
                        debug!("Network mode: {}", network_mode);
                        metric_sender.send_now(MetricId::CELLULAR_NETWORK_MODE, Value::String(network_mode.to_string())).unwrap();

                    },
                    // +CSQ: 22,0
                    "+CSQ" => {
                        let cmd_result = cmd[1].trim().split(',').collect::<Vec<&str>>();
                        let signal_quality = cmd_result[0].parse::<f32>().unwrap();
                        debug!("Signal Quality: {}", signal_quality);
                        metric_sender.send_now(MetricId::CELLULAR_SIGNAL_QUALITY, Value::Float(signal_quality)).unwrap();
                    },
                    d => warn!("Unknown cmd {}", d)
                }
            }
        });

        tokio::spawn(async move {
            loop {
                let write_result = tx
                    .send("AT+CPSI?\r".to_string())
                    .await;
                sleep(Duration::from_millis(1000)).await;
                debug!("Querying LTE Status");
                match write_result {
                    Ok(_) => (),
                    Err(err) => println!("{:?}", err),
                }
            }
        });


        // https://github.com/berkowski/tokio-serial/issues/20
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("lte.enabled").unwrap() == true {
            info!("LTE enabled!");

            tokio::spawn(Self::run_thread(self.metric_sender.clone()));
        }
    }
}

