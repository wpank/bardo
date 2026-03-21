//! Periodic system-resource collection backed by sysinfo.
//!
//! DEPRECATED: Enhanced sys_metrics have been consolidated into bardo-ctl
//! (tmp/bardo-ctl/src/sys_metrics.rs). This module remains for bardo-terminal's
//! standalone monitoring use case but should not be extended further.
//!
//! [`SysStats`] spawns a background thread that owns the sysinfo collectors
//! and emits [`SysMetrics`] snapshots once per second. The render thread reads
//! via a non-blocking `try_recv` so sysinfo I/O never stalls a frame.

use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::Duration;

use sysinfo::{Disks, Networks, System};

use crate::state::{SYS_HISTORY_LEN, SysMetrics};

/// Refresh interval for the background sysinfo thread.
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

/// Owns the channel receiver. The sysinfo work runs in a background thread.
pub(crate) struct SysStats {
    rx: mpsc::Receiver<SysMetrics>,
}

impl SysStats {
    /// Spawns the background sysinfo thread and returns a `SysStats` receiver.
    pub(crate) fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || run_worker(tx));
        Self { rx }
    }

    /// Returns the latest metrics if the background thread has emitted a new snapshot.
    /// Returns `None` on all frames where no new snapshot is available.
    pub(crate) fn tick(&mut self) -> Option<SysMetrics> {
        // Drain to latest; discard older snapshots if we fell behind.
        let mut latest = None;
        loop {
            match self.rx.try_recv() {
                Ok(m) => latest = Some(m),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
        latest
    }
}

fn push(history: &mut VecDeque<f64>, value: f64) {
    if history.len() >= SYS_HISTORY_LEN {
        history.pop_front();
    }
    history.push_back(value);
}

fn run_worker(tx: mpsc::Sender<SysMetrics>) {
    let mut system = System::new_all();
    system.refresh_all();

    let mut networks = Networks::new_with_refreshed_list();
    let mut disks = Disks::new_with_refreshed_list();

    let mut cpu_history: VecDeque<f64> = VecDeque::with_capacity(SYS_HISTORY_LEN);
    let mut mem_history: VecDeque<f64> = VecDeque::with_capacity(SYS_HISTORY_LEN);
    let mut net_rx_history: VecDeque<f64> = VecDeque::with_capacity(SYS_HISTORY_LEN);
    let mut net_tx_history: VecDeque<f64> = VecDeque::with_capacity(SYS_HISTORY_LEN);
    let mut disk_read_history: VecDeque<f64> = VecDeque::with_capacity(SYS_HISTORY_LEN);
    let mut disk_write_history: VecDeque<f64> = VecDeque::with_capacity(SYS_HISTORY_LEN);

    loop {
        std::thread::sleep(REFRESH_INTERVAL);

        system.refresh_cpu_usage();
        system.refresh_memory();
        networks.refresh(false);
        disks.refresh(false);

        // ── CPU ─────────────────────────────────────────────────────────────
        let cpu_pct = system.global_cpu_usage();
        push(&mut cpu_history, cpu_pct as f64);

        // ── Memory ──────────────────────────────────────────────────────────
        let mem_used = system.used_memory();
        let mem_total = system.total_memory();
        let mem_frac = if mem_total > 0 {
            mem_used as f64 / mem_total as f64
        } else {
            0.0
        };
        push(&mut mem_history, mem_frac);

        // ── Network (sum all interfaces) ─────────────────────────────────────
        let mut rx_bytes: u64 = 0;
        let mut tx_bytes: u64 = 0;
        for (_, data) in &networks {
            rx_bytes += data.received();
            tx_bytes += data.transmitted();
        }
        let interval = REFRESH_INTERVAL.as_secs_f64();
        let rx_bps = rx_bytes as f64 / interval;
        let tx_bps = tx_bytes as f64 / interval;
        push(&mut net_rx_history, rx_bps);
        push(&mut net_tx_history, tx_bps);

        // ── Disk I/O (sum all disks) ─────────────────────────────────────────
        let mut read_bytes: u64 = 0;
        let mut write_bytes: u64 = 0;
        let mut disk_used: u64 = 0;
        let mut disk_total: u64 = 0;
        for disk in &disks {
            let usage = disk.usage();
            read_bytes += usage.read_bytes;
            write_bytes += usage.written_bytes;
            disk_total += disk.total_space();
            disk_used += disk.total_space().saturating_sub(disk.available_space());
        }
        let read_bps = read_bytes as f64 / interval;
        let write_bps = write_bytes as f64 / interval;
        push(&mut disk_read_history, read_bps);
        push(&mut disk_write_history, write_bps);

        let metrics = SysMetrics {
            cpu_pct,
            cpu_history: cpu_history.iter().copied().collect(),
            mem_used_bytes: mem_used,
            mem_total_bytes: mem_total,
            mem_history: mem_history.iter().copied().collect(),
            net_rx_bps: rx_bps,
            net_tx_bps: tx_bps,
            net_rx_history: net_rx_history.iter().copied().collect(),
            net_tx_history: net_tx_history.iter().copied().collect(),
            disk_read_bps: read_bps,
            disk_write_bps: write_bps,
            disk_read_history: disk_read_history.iter().copied().collect(),
            disk_write_history: disk_write_history.iter().copied().collect(),
            disk_used_bytes: disk_used,
            disk_total_bytes: disk_total,
        };

        if tx.send(metrics).is_err() {
            // Receiver dropped — app is shutting down.
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_stays_bounded_after_many_pushes() {
        let mut history: VecDeque<f64> = VecDeque::with_capacity(SYS_HISTORY_LEN);
        for i in 0..(SYS_HISTORY_LEN + 8) {
            push(&mut history, i as f64);
        }
        assert_eq!(history.len(), SYS_HISTORY_LEN);
    }

    #[test]
    fn tick_returns_none_before_first_refresh() {
        let mut stats = SysStats::new();
        // Background thread sleeps 1s before first send — try_recv should see nothing.
        assert!(stats.tick().is_none());
    }
}
