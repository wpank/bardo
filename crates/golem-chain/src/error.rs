//! Chain-layer error types.

use alloy::primitives::Address;
use thiserror::Error;
use uuid::Uuid;

/// Unified error type for all chain-layer failures.
#[derive(Debug, Error)]
pub enum ChainError {
    /// The requested chain ID is not in the registry.
    #[error("chain {0} not configured")]
    UnknownChain(u64),

    /// An RPC call failed.
    #[error("RPC error on chain {chain_id}: {message}")]
    RpcError {
        /// Which chain the error occurred on.
        chain_id: u64,
        /// Human-readable error description.
        message: String,
    },

    /// A contract call reverted.
    #[error("call reverted on chain {chain_id}: {reason}")]
    Reverted {
        /// Which chain the revert occurred on.
        chain_id: u64,
        /// Revert reason string.
        reason: String,
    },

    /// A revm simulation failed.
    #[error("simulation failed: {0}")]
    SimulationFailed(String),

    /// A warden action could not be found by its UUID.
    #[error("warden action {0} not found")]
    WardenActionNotFound(Uuid),

    /// No on-chain identity record for the given address.
    #[error("identity not found for address {0}")]
    IdentityNotFound(Address),

    /// Failed to build an alloy provider for the given chain.
    #[error("provider build failed for chain {chain_id}: {source}")]
    ProviderBuild {
        /// Which chain the provider was being built for.
        chain_id: u64,
        /// Underlying error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
