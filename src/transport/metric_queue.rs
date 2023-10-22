use std::sync::{Arc, RwLock};
use std::time::{UNIX_EPOCH, SystemTime};

use log::debug;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex};

#[derive(Clone)]
pub struct MetricStats {
    pub len: usize,
    pub last_ts: u128,
    pub metrics_in_per_sec: f32,
    pub metrics_out_per_sec: f32,
    pub metrics_in: usize,
    pub metrics_out: usize
}

pub struct MetricQueue<T> {
    sender: UnboundedSender<T>,
    receiver: Arc<Mutex<UnboundedReceiver<T>>>,
    stats: Arc<RwLock<MetricStats>>
}

impl<T> MetricQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            sender: sender,
            receiver: Arc::new(Mutex::new(receiver)),
            stats: Arc::new(RwLock::new(MetricStats { len: 0, last_ts: 0, metrics_in_per_sec: 0.0, metrics_out_per_sec: 0.0, metrics_in: 0, metrics_out: 0 }))
        }
    }

    fn calc_stats(&self, mut stats: std::sync::RwLockWriteGuard<'_, MetricStats, >) {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if ts - stats.last_ts > 1000 {
            stats.metrics_in_per_sec = stats.metrics_in as f32;
            stats.metrics_out_per_sec = stats.metrics_out as f32;
            stats.metrics_in = 0;
            stats.metrics_out = 0;
            stats.last_ts = ts;
            debug!("Q Size: {}    In/s: {}    Out/s: {}", stats.len, stats.metrics_in_per_sec, stats.metrics_out_per_sec);
        }
    }

    pub async fn push(&self, e: T) {
        self.sender.send(e).unwrap();
        let mut stats = self.stats.write().unwrap();
        stats.len += 1;
        stats.metrics_in += 1;
        self.calc_stats(stats);
    }

    pub async fn pop(&self) -> T {
        let result = self.receiver.lock().await.recv().await.unwrap();
        let mut stats = self.stats.write().unwrap();
        stats.len -= 1;
        stats.metrics_out += 1;
        self.calc_stats(stats);
        return result;
    }

    pub async fn stats(&self) -> MetricStats {
        self.stats.read().unwrap().clone()
    }
}

impl<T> Clone for MetricQueue<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            stats: self.stats.clone()
        }
    }
}