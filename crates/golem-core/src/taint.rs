//! Taint markers for information-flow tracking.

use serde::{Deserialize, Serialize};

/// Labels that describe how data crossed trust boundaries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaintLabel {
    /// Trusted and unmodified data.
    Clean,
    /// Data whose provenance requires explicit handling.
    Tainted,
    /// Wallet or secret material.
    WalletSecret,
    /// LLM-generated output.
    LlmOutput,
    /// User-originated input.
    UserInput,
    /// Chain-derived data.
    ChainData,
}

/// String value paired with a taint label.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaintedString {
    /// Underlying string value.
    pub value: String,
    /// Information-flow label.
    pub label: TaintLabel,
}

impl TaintedString {
    /// Creates a new tainted string.
    #[must_use]
    pub const fn new(value: String, label: TaintLabel) -> Self {
        Self { value, label }
    }

    /// Creates a clean string.
    pub const fn clean(value: String) -> Self {
        Self::new(value, TaintLabel::Clean)
    }

    /// Returns `true` if the value is not clean.
    pub fn is_tainted(&self) -> bool {
        self.label != TaintLabel::Clean
    }
}

#[cfg(test)]
mod tests {
    use super::{TaintLabel, TaintedString};

    #[test]
    fn tainted_string_is_tainted() {
        let clean = TaintedString::clean("hello".to_owned());
        let tainted = TaintedString::new("secret".to_owned(), TaintLabel::WalletSecret);
        assert!(!clean.is_tainted());
        assert!(tainted.is_tainted());
    }
}
