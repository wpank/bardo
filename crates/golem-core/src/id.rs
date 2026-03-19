//! Golem identity types.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ephemeral runtime identifier for a Golem process.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GolemId(Uuid);

impl GolemId {
    /// Creates a new random Golem identifier.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wraps an existing UUID as a Golem identifier.
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<Uuid> for GolemId {
    fn from(value: Uuid) -> Self {
        Self::from_uuid(value)
    }
}

impl From<GolemId> for Uuid {
    fn from(value: GolemId) -> Self {
        value.0
    }
}

impl fmt::Display for GolemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Default for GolemId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::GolemId;
    use uuid::Uuid;

    #[test]
    fn golem_id_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = GolemId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
        assert_eq!(Uuid::from(id), uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }
}
