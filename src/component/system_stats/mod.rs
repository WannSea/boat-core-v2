use log::error;
use systemstat::{saturating_sub_bytes, Duration, System, Platform};
use wannsea_types::types::Metric;
use crate::{messaging::app_message::{MetricSender, MetricMessage}, SETTINGS};

pub struct SystemStats {
    metric_sender: MetricSender
}

impl SystemStats {
    pub fn new(metric_sender: MetricSender) -> Self {
        SystemStats { metric_sender }
    }

    pub async fn collect_stats(metric_sender: MetricSender) {
        loop {

            let sys = System::new();
        
            if SETTINGS.get::<bool>("system.memory").unwrap() {
                match sys.memory() {
                    Ok(mem) => {
                        metric_sender.send(MetricMessage::now(Metric::MemUsedMb, Metric::val_f32((saturating_sub_bytes(mem.total, mem.free).as_u64() / 1024) as f32))).unwrap();
                        metric_sender.send(MetricMessage::now(Metric::MemTotal, Metric::val_f32((mem.total.as_u64() / 1024) as f32))).unwrap();
                    },
                    Err(x) => error!("Memory: error: {}", x)
                }
            }
           
            if SETTINGS.get::<bool>("system.swap").unwrap() {
                match sys.swap() {
                    Ok(swap) => {
                        metric_sender.send(MetricMessage::now(Metric::SwapUsedMb, Metric::val_f32((saturating_sub_bytes(swap.total, swap.free).as_u64() / 1024) as f32))).unwrap();
                        metric_sender.send(MetricMessage::now(Metric::SwapTotal, Metric::val_f32((swap.total.as_u64() / 1024) as f32))).unwrap();
                    },
                    Err(x) => error!("Swap: error: {}", x)
                }
            }
        
            if SETTINGS.get::<bool>("system.uptime").unwrap() {
                match sys.uptime() {
                    Ok(uptime) => {
                        metric_sender.send(MetricMessage::now(Metric::SystemUptime, Metric::val_f32(uptime.as_secs_f32()))).unwrap();
                    },
                    Err(x) => error!("Uptime: error: {}", x)
                }
            }
            
            if SETTINGS.get::<bool>("system.cpu_temp").unwrap() {
                match sys.cpu_temp() {
                    Ok(cpu_temp) => {
                        metric_sender.send(MetricMessage::now(Metric::CpuTemp, Metric::val_f32(cpu_temp))).unwrap();
                    },
                    Err(x) => error!("CPU temp: {}", x)
                }
            }

            let sleep_duration = Duration::from_millis(SETTINGS.get::<u64>("system.interval").unwrap());
            if SETTINGS.get::<bool>("system.cpu_usage").unwrap() {
                match sys.cpu_load_aggregate() {
                    Ok(cpu)=> {
                        tokio::time::sleep(sleep_duration).await;
                        let cpu = cpu.done().unwrap();

                        metric_sender.send(MetricMessage::now(Metric::CpuUsageUser, Metric::val_f32(cpu.user * 100.0))).unwrap();
                        metric_sender.send(MetricMessage::now(Metric::CpuUsageSystem, Metric::val_f32(cpu.system * 100.0))).unwrap();
                    },
                    Err(x) => {
                        error!("CPU load: error: {}", x);
                        tokio::time::sleep(sleep_duration).await;
                    }
                }
            }
            else {
                tokio::time::sleep(sleep_duration).await;
            }
        }
    }

    pub fn start(&self) {
        tokio::spawn(Self::collect_stats(self.metric_sender.clone()));
    }
}

