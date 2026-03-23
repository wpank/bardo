//! Mortality clock trait and shared types.

use serde::{Deserialize, Serialize};

/// Context passed to each clock on every tick.
pub struct ClockContext {
    /// Current tick number.
    pub tick: u64,
    /// Current epistemic fitness (used by stochastic clock's frailty multiplier).
    pub epistemic_fitness: f64,
    /// Golem identifier (used for deterministic death check seeding).
    pub golem_id: String,
}

/// Outcome of a single clock tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClockEvent {
    /// The clock reports the entity is alive with this vitality level.
    Alive {
        /// Current vitality in [0.0, 1.0].
        vitality: f64,
    },
    /// The clock reports the entity has died.
    Dead {
        /// Which clock caused death.
        cause: DeathCause,
    },
}

/// Which of the three clocks caused death.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DeathCause {
    /// Ran out of USDC credits.
    Economic,
    /// Epistemic fitness collapsed (world model stale).
    Epistemic,
    /// Stochastic hazard check failed.
    Stochastic,
}

/// A mortality clock that tracks one axis of death.
pub trait MortalityClock: Send + Sync {
    /// Current vitality for this clock in [0.0, 1.0].
    fn vitality(&self) -> f64;
    /// Advance this clock by one tick and return the outcome.
    fn tick(&mut self, ctx: &ClockContext) -> ClockEvent;
    /// Whether this clock considers the entity dead.
    fn is_dead(&self) -> bool {
        self.vitality() <= 0.0
    }
}
