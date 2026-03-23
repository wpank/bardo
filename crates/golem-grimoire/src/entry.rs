//! Grimoire entry types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Decay class for knowledge entries.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DecayClass {
    /// Permanent structural knowledge (never decays).
    Structural,
    /// Knowledge conditional on current market regime.
    RegimeConditional,
    /// Short-lived tactical knowledge.
    Tactical,
    /// Ephemeral observations.
    Ephemeral,
}

/// Entry status in the grimoire lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EntryStatus {
    /// Active and retrievable.
    Active,
    /// Archived (low confidence, cold storage).
    Archived,
    /// Burned (confidence zeroed, permanently removed).
    Burned,
}

/// A single knowledge entry in the Grimoire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrimoireEntry {
    /// Unique identifier.
    pub id: Uuid,
    /// Content text.
    pub content: String,
    /// Confidence score in [0.0, 1.0].
    pub confidence: f64,
    /// Knowledge domain.
    pub domain: String,
    /// Knowledge type.
    pub entry_type: String,
    /// Decay classification.
    pub decay_class: DecayClass,
    /// Last tick at which this entry was validated.
    pub last_validated_tick: u64,
    /// Tick at which the entry was created.
    pub created_tick: u64,
    /// Generation (0 = original, 1+ = inherited).
    pub generation: u32,
    /// Current status.
    pub status: EntryStatus,
    /// Whether this is a bloodstain entry (decays at 1/3 rate).
    pub bloodstain: bool,
    /// Whether this entry was dream-validated (decays at 0.5x rate).
    pub dream_validated: bool,
    /// Last retrieval tick.
    pub last_retrieved_tick: u64,
}
