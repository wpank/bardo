//! Alloy-backed chain provider with moka caching layer.

/// Cache key for provider-level result caching.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    /// Chain ID this request targets.
    pub chain_id: u64,
    /// Method-specific discriminator bytes.
    pub discriminator: Vec<u8>,
}

/// A cached RPC result with expiry metadata.
#[derive(Debug, Clone)]
pub struct CachedValue {
    /// Raw bytes of the cached response.
    pub data: Vec<u8>,
    /// Block number at which this value was fetched.
    pub fetched_at_block: u64,
}

/// Per-chain alloy provider with an integrated moka cache.
pub struct ChainProvider {
    _private: (),
}

impl ChainProvider {
    /// Execute a read-only `eth_call` against the given chain and contract.
    ///
    /// Stub implementation — returns empty bytes until real RPC transport is wired.
    pub async fn eth_call(
        &self,
        _chain_id: u64,
        _to: alloy::primitives::Address,
        _calldata: alloy::primitives::Bytes,
    ) -> std::result::Result<alloy::primitives::Bytes, crate::error::ChainError> {
        Ok(alloy::primitives::Bytes::new())
    }
}
