
use std::fmt::Debug;

use log::debug;
use num_derive::FromPrimitive;
use wannsea_types::{boat_core_message::Value, MessageId};

use crate::helper::{MetricSender, MetricSenderExt};

#[derive(FromPrimitive)]
pub enum VescMessageIds {
    SetDuty = 0,
    SetCurrent = 1,
    SetCurrentBrake = 2,
    SetRpm = 3,
    SetPos = 4,
    FillRxBuffer = 5,
    FillRxBufferLong = 6,
    ProcessRxBuffer = 7,
    ProcessShortBuffer = 8,
    Status1 = 9,
    SetCurrentRel = 10,
    SetCurrentBrakeRel = 11,
    SetCurrentHandbrake = 12,
    SetCurrentHandbrakeRel = 13,
    Status2 = 14,
    Status3 = 15,
    Status4 = 16,
    Ping = 17,
    Pong = 18,
    DetectApplyAllFoc = 19,
    DetectApplyAllFocRes = 20,
    ConfCurrentLimits = 21,
    ConfStoreCurrentLimits = 22,
    ConfCurrentLimitsIn = 23,
    ConfStoreCurrentLimitsIn = 24,
    ConfFocRems = 25,
    ConfStoreFocErpms = 26,
    Status5 = 27,
}

/// A trait representing a CAN message.
pub trait CanMessage: Sized + Debug {
    /// Converts the CAN data into an instance of the implementing type.
    ///
    /// # Arguments
    ///
    /// * `data` - The CAN data as a slice of bytes.
    ///
    /// # Returns
    ///
    /// An instance of the implementing type.
    fn from_can_data(data: &[u8]) -> Self;
    
    /// Sends the metrics to the specified metric sender asynchronously.
    ///
    /// # Arguments
    ///
    /// * `metric_sender` - The metric sender to send the metrics to.
    async fn send_metrics(&self, metric_sender: &MetricSender);
    
    /// Parses the CAN data, sends the metrics, and logs debug information.
    ///
    /// # Arguments
    ///
    /// * `data` - The CAN data as a slice of bytes.
    /// * `metric_sender` - The metric sender to send the metrics to.
    fn parse_and_send(data: &[u8], metric_sender: &MetricSender) {
        let metrics = Self::from_can_data(data);
        debug!("Received VESC Metric: {:?}", metrics);
        let _ = metrics.send_metrics(metric_sender);
    }
}

#[derive(Debug)]
pub struct StatusMsg1 {
    rpm: i32,
    total_current: f32,
    duty_cycle: f32,
}

impl CanMessage for StatusMsg1 {
    fn from_can_data(data: &[u8]) -> Self {
        StatusMsg1 {
            rpm: i32::from_be_bytes(data[0..4].try_into().unwrap()),
            total_current: f32::from(i16::from_be_bytes(data[4..6].try_into().unwrap())) / 10.0,
            duty_cycle: f32::from(i16::from_be_bytes(data[6..8].try_into().unwrap())) / 10.0,
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscRpm, Value::Int32(self.rpm));
        let _ = metric_sender.send_now(MessageId::EscTotalCurrent, Value::Float(self.total_current));
        let _ = metric_sender.send_now(MessageId::EscDutyCycle, Value::Float(self.duty_cycle));
    }
}

#[derive(Debug)]
pub struct StatusMsg2 {
    amp_hours: f32,
    amp_hours_charged: f32,
}

impl CanMessage for StatusMsg2 {
    fn from_can_data(data: &[u8]) -> Self {
        StatusMsg2 {
            amp_hours: i32::from_be_bytes(data[0..4].try_into().unwrap()) as f32 / 10_000.0 ,
            amp_hours_charged: i32::from_be_bytes(data[4..8].try_into().unwrap()) as f32 / 10_000.0,
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscAmpHours, Value::Float(self.amp_hours));
        let _ = metric_sender.send_now(MessageId::EscAmpHoursCharged, Value::Float(self.amp_hours_charged));
    }
}

#[derive(Debug)]
pub struct StatusMsg3 {
    watt_hours: f32,
    watt_hours_charged: f32,
}

impl CanMessage for StatusMsg3 {
    fn from_can_data(data: &[u8]) -> Self {
        StatusMsg3 {
            watt_hours: i32::from_be_bytes(data[0..4].try_into().unwrap()) as f32 / 10_000.0,
            watt_hours_charged: i32::from_be_bytes(data[4..8].try_into().unwrap()) as f32 / 10_000.0,
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscWattHours, Value::Float(self.watt_hours));
        let _ = metric_sender.send_now(MessageId::EscWattHoursCharged, Value::Float(self.watt_hours_charged));
    }
}

#[derive(Debug)]
pub struct StatusMsg4 {
    mosfet_temp: f32,
    motor_temp: f32,
    total_in_cur: f32,
    pid_pos: f32,
}

impl CanMessage for StatusMsg4 {
    fn from_can_data(data: &[u8]) -> Self {
        StatusMsg4 {
            mosfet_temp: i16::from_be_bytes(data[0..2].try_into().unwrap()) as f32 / 10.0,
            motor_temp: i16::from_be_bytes(data[2..4].try_into().unwrap()) as f32 / 10.0,
            total_in_cur: i16::from_be_bytes(data[4..6].try_into().unwrap()) as f32 / 10.0,
            pid_pos: i16::from_be_bytes(data[6..8].try_into().unwrap()) as f32 / 50.0,
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscMosfetTemp, Value::Float(self.mosfet_temp));
        let _ = metric_sender.send_now(MessageId::EscMotorTemp, Value::Float(self.motor_temp));
        let _ = metric_sender.send_now(MessageId::EscTotalInCurrent, Value::Float(self.total_in_cur));
        let _ = metric_sender.send_now(MessageId::EscPidPos, Value::Float(self.pid_pos));
    }
}

#[derive(Debug)]
pub struct StatusMsg5 {
    tachometer: i32,
    in_voltage: f32,
}

impl CanMessage for StatusMsg5 {
    fn from_can_data(data: &[u8]) -> Self {
        StatusMsg5 {
            tachometer: i32::from_be_bytes(data[0..4].try_into().unwrap()),
            in_voltage: i16::from_be_bytes(data[4..6].try_into().unwrap()) as f32 / 10.0,
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscTachometer, Value::Int32(self.tachometer));
        let _ = metric_sender.send_now(MessageId::EscInVoltage, Value::Float(self.in_voltage));
    }
}

#[derive(Debug)]
/// Command duty cycle of the VESC
/// - This is direct command of MOSFET PWM modulation
    pub struct SetDuty {
    /// - Range -1 to 1
    duty: f32,
}

impl CanMessage for SetDuty {
    fn from_can_data(data: &[u8]) -> Self {
        SetDuty {
            duty: (i32::from_be_bytes(data[0..4].try_into().unwrap())) as f32 / 100000.0,
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscSetDuty, Value::Float(self.duty * 100.0));
    }
}


#[derive(Debug)]
/// Command a Current in Milliamps
/// - This is direct command of the current control loops
pub struct SetCurrent {
    current: i32,
}

impl CanMessage for SetCurrent {
    fn from_can_data(data: &[u8]) -> Self {
        SetCurrent {
            current: i32::from_be_bytes(data[0..4].try_into().unwrap()),
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscSetCurrent, Value::Int32(self.current));
    }
}


#[derive(Debug)]
/// Command angular velocity in rpm
/// - This is command of the closed loop PID angular velocity
pub struct SetRpm {
    rpm: i32,
}

impl CanMessage for SetRpm {
    fn from_can_data(data: &[u8]) -> Self {
        SetRpm {
            rpm: i32::from_be_bytes(data[0..4].try_into().unwrap()),
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscSetRpm, Value::Int32(self.rpm));
    }
    
}

#[derive(Debug)]
/// Command relative current 
pub struct SetRelCurrent {
    /// - Range -1 (lower Bound)to 1 (upper Bound)
    /// - NOTE that if the upper and lower current limits are not symmetric, sending 0 will NOT result in 0 current.
    current: f32,
}

impl CanMessage for SetRelCurrent {
    fn from_can_data(data: &[u8]) -> Self {
        SetRelCurrent {
            current: (i32::from_be_bytes(data[0..4].try_into().unwrap())) as f32 / 100000.0,
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::EscSetCurrentRel, Value::Float(self.current * 100.0));
    }
}