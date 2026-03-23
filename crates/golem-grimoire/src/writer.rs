//! Grimoire writer trait.

use crate::entry::GrimoireEntry;
use crate::error::GrimoireError;
use uuid::Uuid;

/// Trait for writing to the Grimoire.
pub trait GrimoireWriter: Send + Sync {
    /// Archive an entry by ID.
    fn archive(&self, id: Uuid) -> Result<(), GrimoireError>;
    /// Burn (permanently remove) an entry by ID.
    fn burn(&self, id: Uuid) -> Result<(), GrimoireError>;
    /// Update an entry's confidence.
    fn update_confidence(&self, id: Uuid, confidence: f64) -> Result<(), GrimoireError>;
    /// Retrieve all active entries.
    fn active_entries(&self) -> Result<Vec<GrimoireEntry>, GrimoireError>;
}
