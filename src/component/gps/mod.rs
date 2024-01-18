use std::str;

use futures::StreamExt;
use log::{error, debug, info};
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

    fn process_gprmc(line: &Vec<&str>, sender: &MetricSender) {
        let lat = line[3];
        let lon = line[5];
        let velocity = line[7];
        let course = line[8];

        if lat.len() > 2 && lon.len() > 3 {
            let dd = lat[..2].parse::<f32>().unwrap();
            let lat_rest = lat[2..].parse::<f32>().unwrap();
    
            let ddd = lon[..3].parse::<f32>().unwrap();
            let lon_rest = lon[3..].parse::<f32>().unwrap();
    
            let lat = dd + (lat_rest / 60.0);
            let lon = ddd + (lon_rest / 60.0);
            sender.send_now(MessageId::GpsPos, Value::Vector2(Vector2 { x: lat, y: lon })).unwrap();
        }
      
        if let Ok(val) = velocity.parse::<f32>() {
            sender.send_now(MessageId::GpsSpeed, Value::Float(val)).unwrap();
        }
        if let Ok(val) = course.parse::<f32>() {
            sender.send_now(MessageId::GpsCourse, Value::Float(val)).unwrap();
        }
    }

    fn process_pqxfi(line: &Vec<&str>, sender: &MetricSender) {
        let altitude = line[6];
        let hor_error = line[7];
        let vert_uncertainty = line[8];
        let velo_uncertainty = line[9];

        if let Ok(val) = altitude.parse::<f32>() {
            sender.send_now(MessageId::GpsAltitude, Value::Float(val)).unwrap();
        }

        if let Ok(val) = hor_error.parse::<f32>() {
            sender.send_now(MessageId::GpsHorError, Value::Float(val)).unwrap();
        }

        if let Ok(val) = vert_uncertainty.parse::<f32>() {
            sender.send_now(MessageId::GpsVertUncertainty, Value::Float(val)).unwrap();
        }

        if let Ok(val) = velo_uncertainty.parse::<f32>() {
            sender.send_now(MessageId::GpsVeloUncertainty, Value::Float(val)).unwrap();
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
                Ok(sentence) => {
                    match sentence  {
                        other => {
                            debug!("NMEA {:?}", other);
                        }
                    }
                },
                Err(err) => error!("NMEA parse err {:?}", err)
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

