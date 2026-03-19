//! Cognitive tiering for inference routing.

use core::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::error::GolemError;

/// Cognitive tier used to gate inference spend and latency.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CognitiveTier {
    /// Suppress: heuristics only.
    T0 = 0,
    /// Analyze: light LLM use.
    T1 = 1,
    /// Deliberate: full LLM workspace.
    T2 = 2,
}

impl TryFrom<u8> for CognitiveTier {
    type Error = GolemError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::T0),
            1 => Ok(Self::T1),
            2 => Ok(Self::T2),
            _ => Err(GolemError::Config(format!(
                "invalid cognitive tier: {value}"
            ))),
        }
    }
}

impl From<CognitiveTier> for u8 {
    fn from(value: CognitiveTier) -> Self {
        value as Self
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::CognitiveTier;

    #[test]
    fn cognitive_tier_try_from() {
        assert_eq!(
            CognitiveTier::try_from(0).expect("tier 0"),
            CognitiveTier::T0
        );
        assert_eq!(
            CognitiveTier::try_from(1).expect("tier 1"),
            CognitiveTier::T1
        );
        assert_eq!(
            CognitiveTier::try_from(2).expect("tier 2"),
            CognitiveTier::T2
        );
        assert!(CognitiveTier::try_from(3).is_err());
    }
}
