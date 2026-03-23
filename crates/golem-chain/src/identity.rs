//! ERC-8004 agent identity and on-chain registry client.

use serde::{Deserialize, Serialize};

/// An agent's on-chain identity record (ERC-8004).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// The agent's Ethereum address.
    pub address: [u8; 20],
    /// Chain where the identity is registered (always L1 = 1).
    pub chain_id: u64,
    /// Granted capabilities.
    pub capabilities: Vec<Capability8004>,
    /// Published service endpoints.
    pub service_endpoints: Vec<ServiceEndpoint>,
    /// IPFS CID of the metadata document.
    pub metadata_cid: Option<String>,
    /// Block number of last on-chain update.
    pub last_updated_block: u64,
}

/// A capability declared in an ERC-8004 identity record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability8004 {
    /// Can execute swaps.
    Swap,
    /// Can provide liquidity.
    ProvideLiquidity,
    /// Can manage positions.
    ManagePositions,
    /// Can read oracle data.
    OracleRead,
    /// Can participate in governance.
    Governance,
}

/// A service endpoint advertised through the identity registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Protocol identifier (e.g. "https", "wss").
    pub protocol: String,
    /// Endpoint URL.
    pub url: String,
}

/// Read-only client for the ERC-8004 agent registry on L1.
pub struct Erc8004Registry {
    _private: (),
}
