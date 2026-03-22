//! NFT metadata generation from rack state.
//!
//! Stub: will mint on-chain representations of sonification patches.

/// Metadata for minting a rack configuration as an NFT.
pub struct NftMetadata {
    /// Serialized rack state used as the NFT payload.
    pub rack_state: serde_json::Value,
}

impl NftMetadata {
    /// Prepare NFT metadata from serialized rack state.
    pub fn mint_from_rack(rack_state: serde_json::Value) -> Self {
        Self { rack_state }
    }
}
