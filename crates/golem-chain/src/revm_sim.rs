//! Local EVM simulation via revm.

use serde::{Deserialize, Serialize};

/// A request for a local EVM simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimRequest {
    /// Sender address.
    pub from: [u8; 20],
    /// Target address (None for contract deployment).
    pub to: Option<[u8; 20]>,
    /// Calldata.
    #[serde(with = "serde_bytes_hex")]
    pub data: Vec<u8>,
    /// ETH value in wei (as u128 serialized as string).
    pub value: u128,
    /// Gas limit.
    pub gas_limit: u64,
    /// Chain to simulate against.
    pub chain_id: u64,
    /// Block number to fork from.
    pub fork_block: Option<u64>,
}

/// Result of a local EVM simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimResult {
    /// Whether execution succeeded.
    pub success: bool,
    /// Gas consumed.
    pub gas_used: u64,
    /// Return data bytes.
    #[serde(with = "serde_bytes_hex")]
    pub output: Vec<u8>,
    /// Revert reason if the call reverted.
    pub revert_reason: Option<String>,
}

/// Local EVM simulator backed by revm.
pub struct RevmSimulator {
    _private: (),
}

/// Hex serialization helper for byte vectors in serde contexts.
mod serde_bytes_hex {
    use alloy::hex;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_str = format!("0x{}", hex::encode(bytes));
        serializer.serialize_str(&hex_str)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        hex::decode(s).map_err(serde::de::Error::custom)
    }
}
