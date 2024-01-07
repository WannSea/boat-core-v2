use std::str;

use futures::StreamExt;
use log::{error, debug, info};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;
use wannsea_types::MessageId;
use wannsea_types::boat_core_message::Value;
use crate::{helper::{serial_ext::LineCodec, MetricSender, MetricSenderExt}, SETTINGS};

pub struct GPS {
    metric_sender: MetricSender
}

impl GPS {
    pub fn new(metric_sender: MetricSender) -> Self {
        GPS { metric_sender }
    }

    fn process_gprmc(line: &Vec<&str>, sender: &MetricSender) {
        let lat = line[3];
        let lon = line[5];
        let velocity = line[7];
        let course = line[8];

        debug!("{:?}", line);

        let dd = lat[..2].parse::<f32>().unwrap();
        let lat_rest = lat[2..].parse::<f32>().unwrap();

        let ddd = lon[..3].parse::<f32>().unwrap();
        let lon_rest = lon[3..].parse::<f32>().unwrap();

        sender.send_now(MessageId::GpsLat, Value::Float(dd + (lat_rest / 60.0))).unwrap();
        sender.send_now(MessageId::GpsLon, Value::Float(ddd + (lon_rest / 60.0))).unwrap();
        sender.send_now(MessageId::GpsSpeed, Value::Float(velocity.parse::<f32>().unwrap())).unwrap();
        sender.send_now(MessageId::GpsCourse, Value::Float(course.parse::<f32>().unwrap())).unwrap();
    }

    fn process_pqxfi(line: &Vec<&str>, sender: &MetricSender) {
        let altitude = line[6];
        let hor_error = line[7];
        let vert_uncertainty = line[8];
        let velo_uncertainty = line[9];

        sender.send_now(MessageId::GpsAltitude, Value::Float(altitude.parse::<f32>().unwrap())).unwrap();
        sender.send_now(MessageId::GpsHorError, Value::Float(hor_error.parse::<f32>().unwrap())).unwrap();
        sender.send_now(MessageId::GpsVertUncertainty, Value::Float(vert_uncertainty.parse::<f32>().unwrap())).unwrap();
        sender.send_now(MessageId::GpsVeloUncertainty, Value::Float(velo_uncertainty.parse::<f32>().unwrap())).unwrap();
    }

    pub async fn run_thread(metric_sender: MetricSender) {
        let port = match tokio_serial::new(SETTINGS.get::<String>("gps.port").unwrap(), 115_200)
        .open_native_async() {
            Ok(port) => port,
            Err(_e) => {
                error!("Could not open GPS port. Exiting thread!");
                return;
            }
        };

        let mut reader = LineCodec.framed(port);
        while let Some(line_result) = reader.next().await {
            let line = line_result.unwrap();
            
            let input = line.trim().split('*').collect::<Vec<&str>>()[0].split(',').collect::<Vec<&str>>();
            match input[0] {
                "$PQXFI" => Self::process_pqxfi(&input, &metric_sender),
                "$GPRMC" => Self::process_gprmc(&input, &metric_sender),
                _ => ()
            }
        }
        
        
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("gps.enabled").unwrap() == true {
            info!("GPS enabled!");

            tokio::spawn(Self::run_thread(self.metric_sender.clone()));
        }
    }
}

