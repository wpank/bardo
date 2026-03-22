//! Parameter declarations and the lock-free atomic parameter bridge.

use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Deserialize, Serialize};

/// Named indices into the `AtomicParameterBridge`.
pub mod cv_index {
    /// Master output level (0.0-1.0), driven by composite vitality.
    pub const MASTER_LEVEL: usize = 0;
    /// Event density (0.0-1.0), driven by arousal.
    pub const EVENT_DENSITY: usize = 1;
    /// Total number of bridge slots.
    pub const COUNT: usize = 16;
}

/// Declares a module parameter's metadata and current value.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParamDeclaration {
    /// Internal name (used in `set_param`/`get_param`).
    pub name: String,
    /// Human-readable label.
    pub display_name: String,
    /// Current value.
    pub value: f32,
    /// Minimum allowed value.
    pub min: f32,
    /// Maximum allowed value.
    pub max: f32,
    /// Default value on reset.
    pub default: f32,
    /// Unit label (e.g. "Hz", "dB").
    pub unit: String,
    /// Human-readable description.
    pub description: String,
}

/// Lock-free bridge for passing parameter values between threads.
///
/// Thread 2 (parameter updater) writes f32 values as atomic u32 bit patterns.
/// Thread 3 (rack processor) reads them without locking.
pub struct AtomicParameterBridge {
    slots: [AtomicU32; cv_index::COUNT],
}

impl AtomicParameterBridge {
    /// Creates a new bridge with all slots zeroed.
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| AtomicU32::new(0)),
        }
    }

    /// Write a parameter value (called from Thread 2).
    pub fn write(&self, index: usize, value: f32) {
        if let Some(slot) = self.slots.get(index) {
            slot.store(value.to_bits(), Ordering::Release);
        }
    }

    /// Read a parameter value (called from Thread 3). Wait-free.
    pub fn read(&self, index: usize) -> f32 {
        self.slots
            .get(index)
            .map(|slot| f32::from_bits(slot.load(Ordering::Relaxed)))
            .unwrap_or(0.0)
    }
}

impl Default for AtomicParameterBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_roundtrip() {
        let bridge = AtomicParameterBridge::new();
        bridge.write(cv_index::MASTER_LEVEL, 0.75);
        let v = bridge.read(cv_index::MASTER_LEVEL);
        assert!((v - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_out_of_bounds_returns_zero() {
        let bridge = AtomicParameterBridge::new();
        assert!((bridge.read(999)).abs() < f32::EPSILON);
    }
}
