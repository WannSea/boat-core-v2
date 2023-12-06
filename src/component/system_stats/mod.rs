use log::{info, error};
use systemstat::{saturating_sub_bytes, Duration, System, Platform};
use wannsea_types::{MetricMessage, MetricId};

use crate::{helper::MetricSender, SETTINGS};

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
                        metric_sender.send(MetricMessage::now(MetricId::MEM_USED, (saturating_sub_bytes(mem.total, mem.free).as_u64()).into())).unwrap();
                        metric_sender.send(MetricMessage::now(MetricId::MEM_TOTAL, (mem.total.as_u64()).into())).unwrap();
                    },
                    Err(x) => error!("Memory: error: {}", x)
                }
            }
           
            if SETTINGS.get::<bool>("system.swap").unwrap() {
                match sys.swap() {
                    Ok(swap) => {
                        metric_sender.send(MetricMessage::now(MetricId::SWAP_USED, (saturating_sub_bytes(swap.total, swap.free).as_u64()).into())).unwrap();
                        metric_sender.send(MetricMessage::now(MetricId::SWAP_TOTAL, (swap.total.as_u64()).into())).unwrap();
                    },
                    Err(x) => error!("Swap: error: {}", x)
                }
            }
        
            if SETTINGS.get::<bool>("system.network").unwrap() {
                let network_if = SETTINGS.get::<String>("system.network_if").unwrap();
                match sys.network_stats(&network_if) {
                    Ok(stats) => {
                       // ToDo: Report Network Traffic

                       metric_sender.send(MetricMessage::now(MetricId::NET_RX_BYTES, stats.rx_bytes.as_u64().into())).unwrap();
                       metric_sender.send(MetricMessage::now(MetricId::NET_TX_BYTES, stats.tx_bytes.as_u64().into())).unwrap();
                       metric_sender.send(MetricMessage::now(MetricId::NET_RX_PACKETS, stats.rx_packets.into())).unwrap();
                       metric_sender.send(MetricMessage::now(MetricId::NET_TX_PACKETS, stats.tx_packets.into())).unwrap();
                       metric_sender.send(MetricMessage::now(MetricId::NET_RX_ERORRS, stats.rx_errors.into())).unwrap();
                       metric_sender.send(MetricMessage::now(MetricId::NET_TX_ERORRS, stats.tx_errors.into())).unwrap();
                    },
                    Err(x) => error!("Network: error: {}", x)
                }
            }

            if SETTINGS.get::<bool>("system.uptime").unwrap() {
                match sys.uptime() {
                    Ok(uptime) => {
                        metric_sender.send(MetricMessage::now(MetricId::SYSTEM_UPTIME, (uptime.as_secs_f32()).into())).unwrap();
                    },
                    Err(x) => error!("Uptime: error: {}", x)
                }
            }
            
            if SETTINGS.get::<bool>("system.cpu_temp").unwrap() {
                match sys.cpu_temp() {
                    Ok(cpu_temp) => {
                        metric_sender.send(MetricMessage::now(MetricId::CPU_TEMP, (cpu_temp).into())).unwrap();
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

                        metric_sender.send(MetricMessage::now(MetricId::CPU_USAGE_USER, (cpu.user * 100.0).into())).unwrap();
                        metric_sender.send(MetricMessage::now(MetricId::CPU_USAGE_SYSTEM, (cpu.system * 100.0).into())).unwrap();
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
        if SETTINGS.get::<bool>("system.enabled").unwrap() {
            info!("System Stats enabled!");

            tokio::spawn(Self::collect_stats(self.metric_sender.clone()));
        }
    }
}

