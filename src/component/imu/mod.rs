use crate::{
    helper::{MetricSender, MetricSenderExt},
    SETTINGS,
};
use bno085::{bno_driver::BnoDriver, bno_packet::ChannelCommandData};
use bno085::{
    bno_constants::{
        SENSOR_REPORTID_ACCEL, SENSOR_REPORTID_GYRO_CALIBRATED, SENSOR_REPORTID_ROTATION_VECTOR,
    },
    bno_packet::{BnoPacket::ChannelExec, ChannelExecutableData},
    interface::i2c::I2CInterface,
};
use log::{info, warn};
use tokio::time::{sleep, Duration};
use wannsea_types::boat_core_message::Value;
use wannsea_types::{Floats, MessageId};
pub struct IMU {
    metric_sender: MetricSender,
}

impl IMU {
    pub fn new(metric_sender: MetricSender) -> Self {
        IMU { metric_sender }
    }

    pub async fn run_thread(metric_sender: MetricSender) {
        let rpi_interface = rppal::i2c::I2c::new().unwrap();
        let interface = I2CInterface::new(rpi_interface);

        let accel_interval = SETTINGS.get::<u16>("imu.accel_report_interval").unwrap();
        let gyro_interval = SETTINGS.get::<u16>("imu.gyro_report_interval").unwrap();
        let rotation_interval = SETTINGS.get::<u16>("imu.rotation_report_interval").unwrap();

        let query_interval = SETTINGS.get::<u64>("imu.metric_update_interval").unwrap();

        let mut driver = BnoDriver::new(interface);
        driver.setup();
        driver.soft_reset().unwrap();

        loop {
            match driver.receive_packet() {
                Ok(res) => match res {
                    ChannelExec(ce) => match ce {
                        ChannelExecutableData::ResetComplete => {
                            info!("Reset Complete, enabling Reports!");
                            // Enable reports after reset
                            driver
                                .enable_report(
                                    SENSOR_REPORTID_ACCEL,
                                    accel_interval,
                                    accel_interval - 1,
                                )
                                .unwrap();
                            driver
                                .enable_report(
                                    SENSOR_REPORTID_ROTATION_VECTOR,
                                    rotation_interval,
                                    rotation_interval - 1,
                                )
                                .unwrap();
                            driver
                                .enable_report(
                                    SENSOR_REPORTID_GYRO_CALIBRATED,
                                    gyro_interval,
                                    gyro_interval - 1,
                                )
                                .unwrap();
                        }
                        ChannelExecutableData::Unknown(ced) => { 
                            println!("CED {:?}", ced);
                        }
                    },
                    bno085::bno_packet::BnoPacket::SensorReports(reports) => {
                        for report in reports {
                            match report {
                                bno085::bno_packet::SensorReportData::Acceleration(d) => metric_sender.send_now(MessageId::Acceleration, Value::Floats(Floats{ values: d.get_vec() })).unwrap(),
                                bno085::bno_packet::SensorReportData::Rotation(d) => metric_sender.send_now(MessageId::Rotation, Value::Floats(Floats{ values: d.get_vec() })).unwrap(),
                                bno085::bno_packet::SensorReportData::GyroCalibrated(d) => metric_sender.send_now(MessageId::Gyro, Value::Floats(Floats{ values: d.get_vec() })).unwrap(),
                                d => {
                                    warn!("Unknown Sensor Data {:?}", d);
                                    0 as usize
                                },
                            };
                        }
                    }
                    d => { 
                        println!("CED: {:?}", d);
                    }
                },
                Err(err) => {
                    match err {
                        bno085::bno_driver::DriverError::NoDataAvailable => { /* Nothing to do, can happen due to sleep/clock drift */ },
                        e => {
                            warn!("BNO Driver Error {:?}", e);
                        }
                    }
                },
            }
            sleep(Duration::from_millis(query_interval)).await;
        }

        // loop {
        //     let _msg_count = imu_driver.handle_all_messages(&mut delay_source, 1);
        //     if _msg_count > 0 {
        //         metric_sender.send_now(MessageId::Rotation, Value::Floats(Floats{ values: imu_driver.rotation_quaternion().unwrap().to_vec() })).unwrap();
        //         metric_sender.send_now(MessageId::Acceleration, Value::Floats(Floats{ values: imu_driver.accel().unwrap().to_vec() })).unwrap();
        //         metric_sender.send_now(MessageId::Gyro, Value::Floats(Floats{ values: imu_driver.gyro().unwrap().to_vec() })).unwrap();

        //     }
        //     tokio::time::sleep(tokio::time::Duration::from_millis(SETTINGS.get::<u64>("imu.metric_update_interval").unwrap())).await;
        // }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("imu.enabled").unwrap() == true {
            info!("IMU enabled!");

            tokio::spawn(Self::run_thread(self.metric_sender.clone()));
        }
    }
}
