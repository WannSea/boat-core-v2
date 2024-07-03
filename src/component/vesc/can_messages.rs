
use std::fmt::Debug;

use log::debug;
use num_derive::FromPrimitive;
use num_traits::ToPrimitive;
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

pub trait CanMessage: Sized + Debug {
    fn from_can_data(data: &[u8]) -> Self;
    async fn send_metrics(&self, metric_sender: &MetricSender);
    async fn parse_and_send(data: &[u8], metric_sender: &MetricSender) {
        let metrics = Self::from_can_data(data);
        let _ = metrics.send_metrics(metric_sender).await;

    }
}

#[derive(Debug)]
pub struct SetDutyMsg {
    duty_cycle: i32
}

impl CanMessage for SetDutyMsg {
    fn from_can_data(data: &[u8]) -> Self {
        SetDutyMsg {
            duty_cycle: i32::from_be_bytes(data[0..4].try_into().unwrap()),
        }
    }

    async fn send_metrics(&self, metric_sender: &MetricSender) {
        let _ = metric_sender.send_now(MessageId::ThrottlePos, Value::Float(self.duty_cycle.to_f32().unwrap() / 100000.0f32)).unwrap();
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
        let _ = metric_sender.send_now(MessageId::EscRpm, Value::Int32(self.rpm)).unwrap();
        let _ = metric_sender.send_now(MessageId::EscTotalCurrent, Value::Float(self.total_current)).unwrap();
        let _ = metric_sender.send_now(MessageId::EscDutyCycle, Value::Float(self.duty_cycle)).unwrap();
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
        let _ = metric_sender.send_now(MessageId::EscAmpHours, Value::Float(self.amp_hours)).unwrap();
        let _ = metric_sender.send_now(MessageId::EscAmpHoursCharged, Value::Float(self.amp_hours_charged)).unwrap();
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
        let _ = metric_sender.send_now(MessageId::EscWattHours, Value::Float(self.watt_hours)).unwrap();
        let _ = metric_sender.send_now(MessageId::EscWattHoursCharged, Value::Float(self.watt_hours_charged)).unwrap();
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
        let _ = metric_sender.send_now(MessageId::EscMosfetTemp, Value::Float(self.mosfet_temp)).unwrap();
        let _ = metric_sender.send_now(MessageId::EscMotorTemp, Value::Float(self.motor_temp)).unwrap();
        let _ = metric_sender.send_now(MessageId::EscTotalInCurrent, Value::Float(self.total_in_cur)).unwrap();
        let _ = metric_sender.send_now(MessageId::EscPidPos, Value::Float(self.pid_pos)).unwrap();
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