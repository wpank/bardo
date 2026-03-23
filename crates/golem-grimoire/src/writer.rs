//! Grimoire writer trait.

use crate::entry::GrimoireEntry;
use uuid::Uuid;

/// Trait for writing to the Grimoire.
pub trait GrimoireWriter: Send + Sync {
    /// Archive an entry by ID.
    fn archive(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Burn (permanently remove) an entry by ID.
    fn burn(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Update an entry's confidence.
    fn update_confidence(
        &self,
        id: Uuid,
        confidence: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Retrieve all active entries.
    fn active_entries(
        &self,
    ) -> Result<Vec<GrimoireEntry>, Box<dyn std::error::Error + Send + Sync>>;
}
