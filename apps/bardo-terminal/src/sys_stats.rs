//! Periodic system-resource collection backed by sysinfo.
//!
//! DEPRECATED: Enhanced sys_metrics have been consolidated into bardo-ctl
//! (tmp/bardo-ctl/src/sys_metrics.rs). This module remains for bardo-terminal's
//! standalone monitoring use case but should not be extended further.
//!
//! [`SysStats`] owns the sysinfo collectors and rolls history ring-buffers.
//! It is intentionally not `Clone` or `Debug` because the underlying
//! sysinfo types don't implement those traits. Only the derived
//! [`SysMetrics`] snapshot stored in [`AppState`] is shared with screens.

use std::collections::VecDeque;

use sysinfo::{Disks, Networks, System};

use crate::state::{SYS_HISTORY_LEN, SysMetrics};

/// Minimum seconds between sysinfo refreshes.
const REFRESH_SECS: f64 = 1.0;

/// Owns sysinfo collectors and maintains rolling history for sparklines.
pub(crate) struct SysStats {
    system: System,
    networks: Networks,
    disks: Disks,
    cpu_history: VecDeque<f64>,
    mem_history: VecDeque<f64>,
    net_rx_history: VecDeque<f64>,
    net_tx_history: VecDeque<f64>,
    disk_read_history: VecDeque<f64>,
    disk_write_history: VecDeque<f64>,
    /// Accumulated dt waiting for the next refresh window.
    pending_secs: f64,
    /// Elapsed wall time since the last emitted snapshot.
    elapsed_since_refresh_secs: f64,
}

impl SysStats {
    /// Creates and performs an initial refresh so the first snapshot is non-zero.
    pub(crate) fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        Self {
            system,
            networks,
            disks,
            cpu_history: VecDeque::with_capacity(SYS_HISTORY_LEN),
            mem_history: VecDeque::with_capacity(SYS_HISTORY_LEN),
            net_rx_history: VecDeque::with_capacity(SYS_HISTORY_LEN),
            net_tx_history: VecDeque::with_capacity(SYS_HISTORY_LEN),
            disk_read_history: VecDeque::with_capacity(SYS_HISTORY_LEN),
            disk_write_history: VecDeque::with_capacity(SYS_HISTORY_LEN),
            pending_secs: 0.0,
            elapsed_since_refresh_secs: 0.0,
        }
    }

    /// Advance time by `dt` seconds.
    ///
    /// Returns `Some(metrics)` once per second when a refresh fires,
    /// `None` on all other frames. The caller should update `AppState::sys`
    /// on `Some`.
    pub(crate) fn tick(&mut self, dt: f64) -> Option<SysMetrics> {
        if !dt.is_finite() || dt <= 0.0 {
            return None;
        }

        self.pending_secs += dt;
        self.elapsed_since_refresh_secs += dt;
        if self.pending_secs < REFRESH_SECS {
            return None;
        }

        let interval = self.elapsed_since_refresh_secs;
        self.pending_secs %= REFRESH_SECS;
        self.elapsed_since_refresh_secs = 0.0;

        Some(self.refresh(interval))
    }

    fn refresh(&mut self, interval: f64) -> SysMetrics {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(false);
        self.disks.refresh(false);

        // ── CPU ─────────────────────────────────────────────────────
        let cpu_pct = self.system.global_cpu_usage();
        push(&mut self.cpu_history, cpu_pct as f64);

        // ── Memory ──────────────────────────────────────────────────
        let mem_used = self.system.used_memory();
        let mem_total = self.system.total_memory();
        let mem_frac = if mem_total > 0 {
            mem_used as f64 / mem_total as f64
        } else {
            0.0
        };
        push(&mut self.mem_history, mem_frac);

        // ── Network (sum all interfaces) ─────────────────────────────
        let mut rx_bytes: u64 = 0;
        let mut tx_bytes: u64 = 0;
        for (_, data) in &self.networks {
            rx_bytes += data.received();
            tx_bytes += data.transmitted();
        }
        let rx_bps = rx_bytes as f64 / interval;
        let tx_bps = tx_bytes as f64 / interval;
        push(&mut self.net_rx_history, rx_bps);
        push(&mut self.net_tx_history, tx_bps);

        // ── Disk I/O (sum all disks) ─────────────────────────────────
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
        let read_bps = read_bytes as f64 / interval;
        let write_bps = write_bytes as f64 / interval;
        push(&mut self.disk_read_history, read_bps);
        push(&mut self.disk_write_history, write_bps);

        SysMetrics {
            cpu_pct,
            cpu_history: self.cpu_history.iter().copied().collect(),
            mem_used_bytes: mem_used,
            mem_total_bytes: mem_total,
            mem_history: self.mem_history.iter().copied().collect(),
            net_rx_bps: rx_bps,
            net_tx_bps: tx_bps,
            net_rx_history: self.net_rx_history.iter().copied().collect(),
            net_tx_history: self.net_tx_history.iter().copied().collect(),
            disk_read_bps: read_bps,
            disk_write_bps: write_bps,
            disk_read_history: self.disk_read_history.iter().copied().collect(),
            disk_write_history: self.disk_write_history.iter().copied().collect(),
            disk_used_bytes: disk_used,
            disk_total_bytes: disk_total,
        }
    }
}

fn push(history: &mut VecDeque<f64>, value: f64) {
    if history.len() >= SYS_HISTORY_LEN {
        history.pop_front();
    }
    history.push_back(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_refreshes_once_per_second_without_dropping_remainder() {
        let mut stats = SysStats::new();

        assert!(stats.tick(0.75).is_none());
        assert!(stats.tick(0.75).is_some());
        assert!(stats.tick(0.5).is_some());
    }

    #[test]
    fn tick_ignores_non_positive_or_non_finite_durations() {
        let mut stats = SysStats::new();

        assert!(stats.tick(0.0).is_none());
        assert!(stats.tick(-1.0).is_none());
        assert!(stats.tick(f64::NAN).is_none());
        assert!(stats.tick(f64::INFINITY).is_none());
        assert!(stats.tick(1.0).is_some());
    }

    #[test]
    fn history_buffers_stay_bounded_to_sys_history_len() {
        let mut stats = SysStats::new();
        let mut latest = None;

        for _ in 0..(SYS_HISTORY_LEN + 8) {
            latest = stats.tick(1.0);
        }

        let metrics = latest.expect("expected a metrics snapshot after repeated refreshes");

        assert_eq!(metrics.cpu_history.len(), SYS_HISTORY_LEN);
        assert_eq!(metrics.mem_history.len(), SYS_HISTORY_LEN);
        assert_eq!(metrics.net_rx_history.len(), SYS_HISTORY_LEN);
        assert_eq!(metrics.net_tx_history.len(), SYS_HISTORY_LEN);
        assert_eq!(metrics.disk_read_history.len(), SYS_HISTORY_LEN);
        assert_eq!(metrics.disk_write_history.len(), SYS_HISTORY_LEN);
    }
}
