//! Chain configuration types and static registry.

use serde::{Deserialize, Serialize};

/// Numeric chain identifier.
pub type ChainId = u64;

/// Static configuration for a single chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// EIP-155 chain ID.
    pub chain_id: ChainId,
    /// Short lowercase name (e.g. "ethereum", "base").
    pub name: &'static str,
    /// Native gas token symbol.
    pub native_token: &'static str,
    /// Average block time in seconds.
    pub block_time_secs: f64,
    /// Block explorer base URL.
    pub explorer_url: &'static str,
    /// Env var name holding the RPC URL for this chain.
    pub rpc_url_env: &'static str,
    /// Whether this chain is a Bardo v1 deployment target.
    pub bardo_v1: bool,
    /// Whether Uniswap v4 PoolManager is deployed.
    pub v4_deployed: bool,
    /// Whether UniswapX order system is active.
    pub uniswap_x: bool,
    /// Whether ERC-7683 cross-chain intents are supported.
    pub erc_7683: bool,
}

/// Known contract addresses for a chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    /// Uniswap v4 PoolManager (None if not deployed).
    pub v4_pool_manager: Option<[u8; 20]>,
    /// Permit2 canonical address.
    pub permit2: [u8; 20],
    /// ERC-8004 agent registry (L1 only).
    pub erc8004_registry: Option<[u8; 20]>,
    /// UNI governance token.
    pub uni_token: Option<[u8; 20]>,
    /// Token jar contract.
    pub token_jar: Option<[u8; 20]>,
    /// Firepit burn contract.
    pub firepit: Option<[u8; 20]>,
}

/// Registry holding all configured chains and their contract addresses.
pub struct ChainRegistry {
    chains: std::collections::HashMap<ChainId, ChainConfig>,
    addresses: std::collections::HashMap<ChainId, ContractAddresses>,
}

impl ChainRegistry {
    /// Look up a chain config by ID.
    pub fn get(&self, chain_id: ChainId) -> Option<&ChainConfig> {
        self.chains.get(&chain_id)
    }

    /// Look up contract addresses for a chain.
    pub fn addresses(&self, chain_id: ChainId) -> Option<&ContractAddresses> {
        self.addresses.get(&chain_id)
    }
}
