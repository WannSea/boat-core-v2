use std::str;

use futures::StreamExt;
use log::{error, debug};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;
use wannsea_types::types::Metric;
use crate::{messaging::{app_message::{MetricSender, MetricMessage}, serial_ext::LineCodec}, SETTINGS};

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

        sender.send(MetricMessage::now(Metric::GpsLat, Metric::val_f32(dd + (lat_rest / 60.0)))).unwrap();
        sender.send(MetricMessage::now(Metric::GpsLon, Metric::val_f32(ddd + (lon_rest / 60.0)))).unwrap();
        sender.send(MetricMessage::now(Metric::GpsSpeed, Metric::val_f32(velocity.parse().unwrap()))).unwrap();
        sender.send(MetricMessage::now(Metric::GpsCourse, Metric::val_f32(course.parse().unwrap()))).unwrap();
    }

    fn process_pqxfi(line: &Vec<&str>, sender: &MetricSender) {
        let altitude = line[6];
        let hor_error = line[7];
        let vert_uncertainty = line[8];
        let velo_uncertainty = line[9];

        sender.send(MetricMessage::now(Metric::GpsAltitude, Metric::val_f32(altitude.parse().unwrap()))).unwrap();
        sender.send(MetricMessage::now(Metric::GpsHorError, Metric::val_f32(hor_error.parse().unwrap()))).unwrap();
        sender.send(MetricMessage::now(Metric::GpsVertUncertainty, Metric::val_f32(vert_uncertainty.parse().unwrap()))).unwrap();
        sender.send(MetricMessage::now(Metric::GpsVeloUncertainty, Metric::val_f32(velo_uncertainty.parse().unwrap()))).unwrap();
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
        tokio::spawn(Self::run_thread(self.metric_sender.clone()));
    }
}

