use std::str;

use futures::StreamExt;
use log::{error, debug, info, warn};
use tokio::io::AsyncReadExt;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;
use wannsea_types::{MessageId, Vector2};
use wannsea_types::boat_core_message::Value;
use crate::{helper::{serial_ext::LineCodec, MetricSender, MetricSenderExt}, SETTINGS};
use nmea_parser::*;
pub struct GPS {
    metric_sender: MetricSender
}

// ToDo: Maybe use https://lib.rs/crates/nmea0183
impl GPS {
    pub fn new(metric_sender: MetricSender) -> Self {
        GPS { metric_sender }
    }


    pub fn handle_sentence(message: ParsedMessage, sender: &MetricSender) {
        match message {
            ParsedMessage::Incomplete => { /* Is okay, do nothing */ },
            ParsedMessage::Gsv(gsv) => {
                // Could use more data, satellite count probably only thing we need
                sender.send_now(MessageId::GpsSatelliteCount, Value::Uint32(gsv.len() as u32)).unwrap();
            },
            ParsedMessage::Gns(gns) => {
                debug!("GNS: {:?}", gns);
            },
            ParsedMessage::Vtg(vtg) => {
                debug!("vtg: {:?}", vtg);
                
            },
            ParsedMessage::Gga(gga) => {
                debug!("gga: {:?}", gga);
                // Lat lon quality sat count 
            },
            ParsedMessage::Rmc(rmc) => {
                debug!("rmc: {:?}", rmc);
                // Speed and Course
            },
            ParsedMessage::Gsa(gsa) => {
                /* Ignore */
            },
            unknown => warn!("Unknown NMEA sentence {:?}", unknown)
        }
    }

    pub async fn run_thread(metric_sender: MetricSender) {
        let mut port = match tokio_serial::new(SETTINGS.get::<String>("gps.port").unwrap(), 115_200)
        .open_native_async() {
            Ok(port) => port,
            Err(_e) => {
                error!("Could not open GPS port. Exiting thread!");
                return;
            }
        };
        
        let mut parser = NmeaParser::new();
        
        let mut reader = LineCodec.framed(port);
        while let Some(line_result) = reader.next().await {
            let line = line_result.unwrap();
            match parser.parse_sentence(line.as_str()) {
                Ok(sentence) => Self::handle_sentence(sentence, &metric_sender),
                Err(_err) => { /* Ignore, happens when unknown sentence arrive (only $XFI) */ }
            }
        }
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(core::time::Duration::from_secs(1)).await;
            }
        });
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("gps.enabled").unwrap() == true {
            info!("GPS enabled!");

            tokio::spawn(Self::run_thread(self.metric_sender.clone()));
        }
    }
}

