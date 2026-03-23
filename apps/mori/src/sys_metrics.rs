use std::collections::VecDeque;
use sysinfo::{Disks, Networks, System};

const HISTORY_LEN: usize = 60;
const REFRESH_TICKS: u32 = 30; // 1 s at 30 fps

#[derive(Debug, Clone, Default)]
pub struct SysSnapshot {
    pub cpu_pct: f32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub net_rx_bytes_sec: f64,
    pub net_tx_bytes_sec: f64,
    pub disk_read_bytes_sec: f64,
    pub disk_write_bytes_sec: f64,
    pub cpu_history: VecDeque<f32>,
    pub mem_history: VecDeque<f32>,    // fraction 0..1
    pub net_rx_history: VecDeque<f64>, // bytes/sec
    pub net_tx_history: VecDeque<f64>,
    pub disk_r_history: VecDeque<f64>, // bytes/sec
    pub disk_w_history: VecDeque<f64>,
}

pub struct SysCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    tick_count: u32,
}

impl SysCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        Self {
            system,
            networks,
            disks,
            tick_count: 0,
        }
    }

    /// Call once per render tick. Updates `snap` every REFRESH_TICKS frames.
    pub fn poll(&mut self, snap: &mut SysSnapshot) {
        self.tick_count += 1;
        if self.tick_count < REFRESH_TICKS {
            return;
        }
        self.tick_count = 0;

        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(false);
        self.disks.refresh(false);

        // CPU
        snap.cpu_pct = self.system.global_cpu_usage();
        push_f32(&mut snap.cpu_history, snap.cpu_pct);

        // Memory
        snap.mem_used_bytes = self.system.used_memory();
        snap.mem_total_bytes = self.system.total_memory();
        let mem_frac = if snap.mem_total_bytes > 0 {
            snap.mem_used_bytes as f32 / snap.mem_total_bytes as f32
        } else {
            0.0
        };
        push_f32(&mut snap.mem_history, mem_frac);

        // Network (sum all interfaces, bytes received/transmitted since last refresh)
        let mut rx_bytes: u64 = 0;
        let mut tx_bytes: u64 = 0;
        for (_, data) in &self.networks {
            rx_bytes += data.received();
            tx_bytes += data.transmitted();
        }
        // sysinfo returns bytes since last refresh; our interval is ~1s
        snap.net_rx_bytes_sec = rx_bytes as f64;
        snap.net_tx_bytes_sec = tx_bytes as f64;
        push_f64(&mut snap.net_rx_history, snap.net_rx_bytes_sec);
        push_f64(&mut snap.net_tx_history, snap.net_tx_bytes_sec);

        // Disk I/O + capacity
        let mut read_bytes: u64 = 0;
        let mut write_bytes: u64 = 0;
        let mut disk_used: u64 = 0;
        let mut disk_total: u64 = 0;
        for disk in &self.disks {
            let usage = disk.usage();
            read_bytes += usage.read_bytes;
            write_bytes += usage.written_bytes;
            disk_total += disk.total_space();
            disk_used += disk.total_space().saturating_sub(disk.available_space());
        }
        snap.disk_read_bytes_sec = read_bytes as f64;
        snap.disk_write_bytes_sec = write_bytes as f64;
        snap.disk_used_bytes = disk_used;
        snap.disk_total_bytes = disk_total;
        push_f64(&mut snap.disk_r_history, snap.disk_read_bytes_sec);
        push_f64(&mut snap.disk_w_history, snap.disk_write_bytes_sec);
    }
}

fn push_f32(h: &mut VecDeque<f32>, v: f32) {
    if h.len() >= HISTORY_LEN {
        h.pop_front();
    }
    h.push_back(v);
}

fn push_f64(h: &mut VecDeque<f64>, v: f64) {
    if h.len() >= HISTORY_LEN {
        h.pop_front();
    }
    h.push_back(v);
}
