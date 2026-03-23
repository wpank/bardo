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
