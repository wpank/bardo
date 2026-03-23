//! Cognitive tiering for inference routing.
//!
//! Re-exports `InferenceTier` from `bardo-primitives` under the platform-local
//! name `CognitiveTier`. All downstream Golem crates that import
//! `golem_core::CognitiveTier` continue to work unchanged; the type is
//! identical to `bardo_primitives::InferenceTier`.

pub use bardo_primitives::InferenceTier as CognitiveTier;

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
