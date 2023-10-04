use std::str;

use log::{error, trace};
use systemstat::Duration;

use crate::{messaging::{app_message::{MetricSender, MetricMessage}, serial_ext::read_line}, SETTINGS};

pub struct GPS {
    metric_sender: MetricSender
}

impl GPS {
    pub fn new(metric_sender: MetricSender) -> Self {
        GPS { metric_sender }
    }

    fn process_gprmc(line: &Vec<&str>, sender: MetricSender) {
        let lat = line[3];
        let lon = line[5];
        let velocity = line[7];
        let course = line[8];

        let dd = lat[..2].parse::<f32>().unwrap();
        let lat_rest = lat[2..].parse::<f32>().unwrap();

        let ddd = lon[..3].parse::<f32>().unwrap();
        let lon_rest = lon[3..].parse::<f32>().unwrap();

        sender.send(MetricMessage::now(format!("GPS_LAT"), dd + (lat_rest / 60.0))).unwrap();
        sender.send(MetricMessage::now(format!("GPS_LON"), ddd + (lon_rest / 60.0))).unwrap();
        sender.send(MetricMessage::now(format!("GPS_VELOCITY"), velocity.parse().unwrap())).unwrap();
        sender.send(MetricMessage::now(format!("GPS_COURSE"), course.parse().unwrap())).unwrap();
    }

    fn process_pqxfi(line: &Vec<&str>, sender: MetricSender) {
        let altitude = line[6];
        let hor_error = line[7];
        let vert_uncertainty = line[8];
        let velo_uncertainty = line[9];

        sender.send(MetricMessage::now(format!("GPS_ALTITUDE"), altitude.parse().unwrap())).unwrap();
        sender.send(MetricMessage::now(format!("GPS_HOR_ERROR"), hor_error.parse().unwrap())).unwrap();
        sender.send(MetricMessage::now(format!("GPS_VERT_UNCERTAINTY"), vert_uncertainty.parse().unwrap())).unwrap();
        sender.send(MetricMessage::now(format!("GPS_VELO_UNCERTAINTY"), velo_uncertainty.parse().unwrap())).unwrap();
    }

    pub async fn run_thread(metric_sender: MetricSender) {
        let port = match serialport::new(SETTINGS.get::<String>("gps.port").unwrap(), 115_200)
            .timeout(Duration::from_millis(10))
            .open() {
                Ok(port) => port,
                Err(_e) => {
                    error!("Could not open GPS port. Exiting thread!");
                    return;
                }
            };

        let line = read_line(port);
        trace!("Read GPS line {}", line);
        
        let input = line.split('*').collect::<Vec<&str>>()[0].split(',').collect::<Vec<&str>>();
        match input[0] {
            "$PQXFI" => Self::process_pqxfi(&input, metric_sender),
            "$GPRMC" => Self::process_gprmc(&input, metric_sender),
            _ => ()
        }
    }

    pub fn start(&self) {
        tokio::spawn(Self::run_thread(self.metric_sender.clone()));
    }
}

