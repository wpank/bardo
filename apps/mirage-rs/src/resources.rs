//! Resource profiles and usage reporting for `mirage-rs`.

use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::{MirageError, Result};

const MICRO_MEMORY_BYTES: u64 = 256 * 1024 * 1024;
const STANDARD_MEMORY_BYTES: u64 = 512 * 1024 * 1024;
const POWER_MEMORY_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const SPAWN_HEADROOM_BYTES: u64 = 128 * 1024 * 1024;

fn ensure_spawn_budget_from_available_memory(required: u64, available: u64) -> Result<()> {
    if available == 0 || available < required {
        return Err(MirageError::Unsupported(format!(
            "insufficient memory: available={available} required={required}"
        )));
    }
    Ok(())
}

/// Runtime operating mode for the fork.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MirageMode {
    /// Live lazy-read mode with optional targeted follower support.
    Live,
    /// Historical pinned-block mode.
    Historical,
    /// Proxy-only mode with replay disabled due to pressure.
    Proxy,
}

/// Predefined resource envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Profile {
    /// 256 MB / 32 watched contracts / 5,000 cache entries.
    Micro,
    /// 512 MB / 64 watched contracts / 10,000 cache entries.
    #[default]
    Standard,
    /// 2 GB / 256 watched contracts / 50,000 cache entries.
    Power,
}

/// Static resource model used for startup checks and runtime pressure handling.
#[derive(Debug, Clone)]
pub struct ResourceModel {
    /// Selected resource profile.
    pub profile: Profile,
    /// Hard memory ceiling in bytes.
    pub max_memory_bytes: u64,
    /// Maximum number of watched contracts.
    pub max_watched_contracts: usize,
    /// Read-cache entry capacity.
    pub cache_capacity: usize,
    /// Read-cache TTL.
    pub cache_ttl: Duration,
}

impl ResourceModel {
    /// Creates a model from a named profile.
    #[must_use]
    pub fn for_profile(profile: Profile, cache_ttl: Duration) -> Self {
        match profile {
            Profile::Micro => Self {
                profile,
                max_memory_bytes: MICRO_MEMORY_BYTES,
                max_watched_contracts: 32,
                cache_capacity: 5_000,
                cache_ttl,
            },
            Profile::Standard => Self {
                profile,
                max_memory_bytes: STANDARD_MEMORY_BYTES,
                max_watched_contracts: 64,
                cache_capacity: 10_000,
                cache_ttl,
            },
            Profile::Power => Self {
                profile,
                max_memory_bytes: POWER_MEMORY_BYTES,
                max_watched_contracts: 256,
                cache_capacity: 50_000,
                cache_ttl,
            },
        }
    }

    /// Returns the derived bytecode cache capacity.
    #[must_use]
    pub fn bytecode_cache_capacity(&self) -> NonZeroUsize {
        NonZeroUsize::new((self.cache_capacity / 5).clamp(1, 10_000)).unwrap_or(NonZeroUsize::MIN)
    }

    /// Computes resource pressure from a memory sample.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn pressure_for_memory(&self, memory_bytes: u64) -> f64 {
        if self.max_memory_bytes == 0 {
            return 0.0;
        }
        (memory_bytes as f64 / self.max_memory_bytes as f64).clamp(0.0, 1.5)
    }

    /// Checks whether the process should be allowed to start.
    pub fn ensure_spawn_budget(&self) -> Result<()> {
        let mut system = System::new();
        system.refresh_memory();
        let available = std::env::var("BARDO_AVAILABLE_MEMORY_BYTES")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or_else(|| system.available_memory());
        // sysinfo returns 0 when it cannot read available memory (e.g. on macOS
        // when the Mach host_statistics64 call fails).  Treat 0 as unknown and
        // skip the check rather than blocking startup on a machine with plenty
        // of RAM.
        if available == 0 {
            return Ok(());
        }
        let required = self.max_memory_bytes.saturating_add(SPAWN_HEADROOM_BYTES);
        ensure_spawn_budget_from_available_memory(required, available)
    }

    /// Captures the current process memory footprint.
    #[must_use]
    pub fn current_process_memory_bytes() -> u64 {
        let mut system = System::new_all();
        let pid = Pid::from_u32(std::process::id());
        system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        system.process(pid).map_or(0, sysinfo::Process::memory)
    }
}

/// Runtime pressure response selected from the latest resource sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureAction {
    /// No runtime changes are needed.
    None,
    /// Trim caches and keep the fork live.
    EvictCache,
    /// Trim caches and demote auto-classified contracts to slot-only reads.
    Throttle,
    /// Trim caches and demote runtime mode to proxy.
    DemoteToProxy,
}

impl ResourceUsage {
    /// Returns the runtime action implied by this usage sample.
    #[must_use]
    pub fn pressure_action(&self) -> PressureAction {
        if self.is_emergency() {
            PressureAction::DemoteToProxy
        } else if self.is_throttled() {
            PressureAction::Throttle
        } else if self.is_warning() {
            PressureAction::EvictCache
        } else {
            PressureAction::None
        }
    }
}

/// Snapshot of the runtime's current resource usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUsage {
    /// Resident memory estimate for the current process.
    pub memory_bytes: u64,
    /// Configured memory limit for the current profile.
    pub memory_limit_bytes: u64,
    /// Pressure score in the inclusive range `[0.0, 1.0+]`.
    pub resource_pressure: f64,
    /// Read-cache hit rate across the process lifetime.
    pub cache_hit_rate: f64,
    /// Current read-cache entry count.
    pub cache_entries: usize,
    /// Configured read-cache capacity.
    pub cache_capacity: usize,
    /// Number of watched contracts.
    pub watch_list_size: usize,
    /// Number of locally dirty storage slots.
    pub dirty_slot_count: u64,
    /// Total upstream RPC calls.
    pub upstream_rpc_calls: u64,
    /// Total upstream RPC errors.
    pub upstream_rpc_errors: u64,
    /// Current operating mode.
    pub mode: MirageMode,
    /// Estimated disk usage for artifacts in bytes.
    pub disk_usage_bytes: u64,
}

impl ResourceUsage {
    /// Builds a usage snapshot.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        model: &ResourceModel,
        memory_bytes: u64,
        cache_hit_rate: f64,
        cache_entries: usize,
        watch_list_size: usize,
        dirty_slot_count: u64,
        upstream_rpc_calls: u64,
        upstream_rpc_errors: u64,
        mode: MirageMode,
        disk_usage_bytes: u64,
    ) -> Self {
        Self {
            memory_bytes,
            memory_limit_bytes: model.max_memory_bytes,
            resource_pressure: model.pressure_for_memory(memory_bytes),
            cache_hit_rate,
            cache_entries,
            cache_capacity: model.cache_capacity,
            watch_list_size,
            dirty_slot_count,
            upstream_rpc_calls,
            upstream_rpc_errors,
            mode,
            disk_usage_bytes,
        }
    }

    /// Returns whether the usage has entered the warning tier.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.resource_pressure >= 0.5
    }

    /// Returns whether the usage has entered the throttle tier.
    #[must_use]
    pub fn is_throttled(&self) -> bool {
        self.resource_pressure >= 0.7
    }

    /// Returns whether the usage has entered the emergency tier.
    #[must_use]
    pub fn is_emergency(&self) -> bool {
        self.resource_pressure >= 0.9
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use alloy_primitives::{U256, address};

    use super::{PressureAction, Profile, ResourceModel, ResourceUsage};
    use crate::resources::MirageMode;
    use crate::{AccountInfo, Bytecode, fork::ReadCache};

    #[test]
    fn pressure_tiers() {
        let model =
            ResourceModel::for_profile(Profile::Standard, std::time::Duration::from_secs(12));
        let idle = ResourceUsage::new(
            &model,
            64 * 1024 * 1024,
            0.8,
            10,
            0,
            0,
            0,
            0,
            MirageMode::Live,
            0,
        );
        assert!(!idle.is_warning());

        let warning = ResourceUsage::new(
            &model,
            300 * 1024 * 1024,
            0.8,
            10,
            0,
            0,
            0,
            0,
            MirageMode::Live,
            0,
        );
        assert!(warning.is_warning());
        assert!(!warning.is_throttled());

        let throttle = ResourceUsage::new(
            &model,
            380 * 1024 * 1024,
            0.8,
            10,
            0,
            0,
            0,
            0,
            MirageMode::Live,
            0,
        );
        assert!(throttle.is_throttled());

        let emergency = ResourceUsage::new(
            &model,
            470 * 1024 * 1024,
            0.8,
            10,
            0,
            0,
            0,
            0,
            MirageMode::Live,
            0,
        );
        assert!(emergency.is_emergency());
        assert_eq!(warning.pressure_action(), PressureAction::EvictCache);
        assert_eq!(throttle.pressure_action(), PressureAction::Throttle);
        assert_eq!(emergency.pressure_action(), PressureAction::DemoteToProxy);
    }

    #[test]
    fn lru_eviction_at_capacity() {
        let mut cache = ReadCache::new(2, Duration::from_secs(12));
        let info = AccountInfo {
            balance: U256::from(1_u64),
            nonce: 0,
            code_hash: Bytecode::default().hash_slow(),
            code: Some(Bytecode::default()),
        };

        cache.insert_account(
            address!("0x1000000000000000000000000000000000000001"),
            info.clone(),
        );
        cache.insert_account(
            address!("0x1000000000000000000000000000000000000002"),
            info.clone(),
        );
        cache.insert_account(address!("0x1000000000000000000000000000000000000003"), info);

        assert_eq!(cache.entry_count(), 2);
    }

    #[test]
    fn zero_available_memory_is_rejected() {
        let err = super::ensure_spawn_budget_from_available_memory(640 * 1024 * 1024, 0)
            .expect_err("zero available memory should fail");

        assert!(err.to_string().contains("available=0"));
    }
}
