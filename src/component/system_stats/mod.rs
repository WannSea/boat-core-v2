use crate::{
    helper::{MetricSender, MetricSenderExt},
    SETTINGS,
};
use log::{error, info, warn};
use systemstat::{saturating_sub_bytes, Platform};
use std::{borrow::BorrowMut, collections::HashMap};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind};
use wannsea_types::{boat_core_message::Value, Floats};
use wannsea_types::{MessageId, StringFloatMap};
use std::time::Duration;


pub struct SystemStats {
    metric_sender: MetricSender,
}

impl SystemStats {
    pub fn new(metric_sender: MetricSender) -> Self {
        SystemStats { metric_sender }
    }

    // Use sysinfo for process specific stuff
    async fn collect_sysinfo_stats(metric_sender: MetricSender) {
        let report_interval = Duration::from_millis(SETTINGS.get::<u64>("system.interval").unwrap());
        let prk_specific = ProcessRefreshKind::new().with_cpu().with_memory();
        let mut sys = sysinfo::System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));

        let process_names = SETTINGS.get::<Vec<String>>("system.processes").unwrap();
        if process_names.len() == 0 {
            info!("No processes to watch");
            return;
        }
        
        let mut pids: HashMap<String, u32> = HashMap::new();
        let mut process_cpus: HashMap<String, f32> = HashMap::new();
        let mut process_memory: HashMap<String, f32> = HashMap::new();

        loop {
            for p in &process_names {
                let proc = sys.borrow_mut().processes_by_exact_name(&p).next();
                match proc {
                    Some(proc) => {
                        // Insert
                        if pids.insert(p.to_owned(), proc.pid().as_u32()).is_none() {
                            info!("Watching new process: {}", p);
                            process_cpus.insert(p.to_owned(), 0.0);
                            process_memory.insert(p.to_owned(), 0.0);
                        }
                        *process_cpus.get_mut(p).unwrap() = proc.cpu_usage();
                        *process_memory.get_mut(p).unwrap() = ((proc.memory() as f64) / 1000_000.0) as f32;
                    }
                    None => {
                        // Remove
                        if let Some(pid) = pids.remove(p) {
                            info!("Remove watched process: {} - {}", p, pid);
                            process_cpus.remove(p).unwrap();
                            process_memory.remove(p).unwrap();
                        }
                    }
                }
            }
            metric_sender
                .send_now(MessageId::ProcCpuUsage, Value::StringFloatMap { 0: StringFloatMap { items: process_cpus.clone() }})
                .unwrap();

            metric_sender
                .send_now(MessageId::ProcMemUsed, Value::StringFloatMap { 0: StringFloatMap { items: process_memory.clone() }})
                .unwrap();

            tokio::time::sleep(report_interval).await;

            for pid in pids.values() {
                sys.refresh_process_specifics(Pid::from_u32(*pid), prk_specific);
            }
        }
    }

    async fn collect_systemstat_stats(metric_sender: MetricSender) {
        let report_interval = Duration::from_millis(SETTINGS.get::<u64>("system.interval").unwrap());
        let sys = systemstat::System::new();
        let mut network_if = SETTINGS.get::<String>("system.network_if").unwrap();
        if network_if == "auto" {
            match sys.networks() {
                Ok(networks) => {
                    // Prefer usb0 (LTE Module)
                    if let Some(_net) = networks.get("usb0") {
                        network_if = String::from("usb0");
                    }
                    // If not present, just choose first interface available
                    else if let Some((if_name, _net)) = networks.first_key_value() {
                        network_if = String::from(if_name);
                    }
                    info!("Chose Network {} automatically!", network_if);
                },
                Err(e) => warn!("Could not automatically define network interface: {:?}", e),
            }
        }

        loop {

            if SETTINGS.get::<bool>("system.memory").unwrap() {
                match sys.memory() {
                    Ok(mem) => {
                        metric_sender.send_now(MessageId::MemUsed, Value::Uint64(saturating_sub_bytes(mem.total, mem.free).as_u64())).unwrap();
                        metric_sender.send_now(MessageId::MemTotal, Value::Uint64(mem.total.as_u64())).unwrap();
                    },
                    Err(x) => error!("Memory: error: {}", x)
                }
            }

            if SETTINGS.get::<bool>("system.swap").unwrap() {
                match sys.swap() {
                    Ok(swap) => {
                        metric_sender.send_now(MessageId::SwapUsed, Value::Uint64(saturating_sub_bytes(swap.total, swap.free).as_u64())).unwrap();
                        metric_sender.send_now(MessageId::SwapTotal, Value::Uint64(swap.total.as_u64())).unwrap();
                    },
                    Err(x) => error!("Swap: error: {}", x)
                }
            }

            if SETTINGS.get::<bool>("system.network").unwrap() {
                match sys.network_stats(&network_if) {
                    Ok(stats) => {
                       // ToDo: Report Network Traffic
                       metric_sender.send_now(MessageId::NetRxBytes, Value::Uint64(stats.rx_bytes.as_u64())).unwrap();
                       metric_sender.send_now(MessageId::NetTxBytes, Value::Uint64(stats.tx_bytes.as_u64())).unwrap();
                       metric_sender.send_now(MessageId::NetRxPackets, Value::Uint64(stats.rx_packets)).unwrap();
                       metric_sender.send_now(MessageId::NetTxPackets, Value::Uint64(stats.tx_packets)).unwrap();
                       metric_sender.send_now(MessageId::NetRxErorrs, Value::Uint64(stats.rx_errors)).unwrap();
                       metric_sender.send_now(MessageId::NetTxErorrs, Value::Uint64(stats.tx_errors)).unwrap();
                    },
                    Err(x) => error!("Network: error: {}", x)
                }
            }

            if SETTINGS.get::<bool>("system.uptime").unwrap() {
                match sys.uptime() {
                    Ok(uptime) => {
                        metric_sender.send_now(MessageId::SystemUptime, Value::Float(uptime.as_secs_f32())).unwrap();
                    },
                    Err(x) => error!("Uptime: error: {}", x)
                }
            }

            if SETTINGS.get::<bool>("system.cpu_temp").unwrap() {
                match sys.cpu_temp() {
                    Ok(cpu_temp) => {
                        metric_sender.send_now(MessageId::CpuTemp, Value::Float(cpu_temp)).unwrap();
                    },
                    Err(x) => error!("CPU temp: {}", x)
                }
            }
            if SETTINGS.get::<bool>("system.cpu_freq").unwrap() {
                let freqs = cpu_freq::get();
                metric_sender.send_now(MessageId::CpuFreqs, Value::Floats(Floats { values: freqs.iter().map(|f| f.cur.unwrap_or(-1.0)).collect() })).unwrap();
            }

            if SETTINGS.get::<bool>("system.cpu_usage").unwrap() {
                match sys.cpu_load_aggregate() {
                    Ok(cpu)=> {
                        tokio::time::sleep(report_interval).await;
                        let cpu = cpu.done().unwrap();

                        metric_sender.send_now(MessageId::CpuUsageUser, Value::Float(cpu.user * 100.0)).unwrap();
                        metric_sender.send_now(MessageId::CpuUsageSystem, Value::Float(cpu.system * 100.0)).unwrap();
                    },
                    Err(x) => {
                        error!("CPU load: error: {}", x);
                        tokio::time::sleep(report_interval).await;
                    }
                }
            }
            else {
        
            }
        }
    }

    pub fn start(&self) {
        if SETTINGS.get::<bool>("system.enabled").unwrap() {
            info!("System Stats enabled!");

            tokio::spawn(Self::collect_sysinfo_stats(self.metric_sender.clone()));
            tokio::spawn(Self::collect_systemstat_stats(self.metric_sender.clone()));

        }
    }
}
