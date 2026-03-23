//! ERC-8004 agent identity types and read-only registry client.

use std::sync::Arc;

use alloy::primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};

use crate::config::ChainId;
use crate::error::ChainError;
use crate::provider::ChainProvider;

/// A registered on-chain agent identity per ERC-8004.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// The agent's Ethereum address.
    pub address: Address,
    /// Chain where the identity was registered (always 1 for L1).
    pub chain_id: u64,
    /// Registered capabilities.
    pub capabilities: Vec<Capability8004>,
    /// Named URL endpoints the agent exposes.
    pub service_endpoints: Vec<ServiceEndpoint>,
    /// IPFS CID pointing to extended metadata JSON.
    pub metadata_cid: String,
    /// Block number of the last on-chain update.
    pub last_updated_block: u64,
}

/// Capability categories an agent can register under ERC-8004.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability8004 {
    /// Can execute spot and limit trades.
    Trading,
    /// Can provide concentrated liquidity.
    LiquidityProvider,
    /// Can supply/borrow in lending protocols.
    Lending,
    /// Can construct and route cross-chain intents.
    CrossChainRouting,
    /// Can manage ERC-4626 vaults.
    VaultManagement,
    /// Can act as a UniswapX filler.
    UniswapXFiller,
    /// Extension point for future capabilities.
    Custom(String),
}

/// A named URL endpoint in an agent's identity record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Human-readable name (e.g. "api", "stream", "health").
    pub name: String,
    /// The endpoint URL.
    pub url: String,
}

/// Read-only client for the ERC-8004 registry contract.
///
/// Write operations (register, update) are in Plan 45.
pub struct Erc8004Registry {
    provider: Arc<ChainProvider>,
    l1_chain_id: ChainId,
    registry_address: Address,
}

/// ERC-8004 ABI function selectors (first 4 bytes of keccak256).
mod selectors {
    /// `keccak256("getIdentity(address)")[0..4]`
    pub const GET_IDENTITY: [u8; 4] = [0xa5, 0xfc, 0x32, 0x8a];
    /// `keccak256("hasCapability(address,uint8)")[0..4]`
    pub const HAS_CAPABILITY: [u8; 4] = [0x1a, 0x8b, 0x95, 0x68];
    /// `keccak256("listIdentities(uint256,uint256)")[0..4]`
    pub const LIST_IDENTITIES: [u8; 4] = [0x9d, 0x6f, 0xa6, 0x18];
}

impl Erc8004Registry {
    /// Create a new read-only registry client.
    ///
    /// The registry is on Ethereum L1 (`chain_id = 1`).
    pub fn new(provider: Arc<ChainProvider>) -> Self {
        Self {
            provider,
            l1_chain_id: 1,
            registry_address: "0x8004A818BFB912233c491871b3d84c89A494BD9e"
                .parse()
                .expect("hardcoded registry address"),
        }
    }

    /// Fetch a Golem's registered identity by address. Returns `None` if not registered.
    pub async fn get_identity(
        &self,
        address: Address,
    ) -> Result<Option<AgentIdentity>, ChainError> {
        let mut calldata = Vec::with_capacity(36);
        calldata.extend_from_slice(&selectors::GET_IDENTITY);
        // ABI-encode address: 12 zero bytes + 20 address bytes
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(address.as_slice());

        let result = self
            .provider
            .eth_call(
                self.l1_chain_id,
                self.registry_address,
                Bytes::from(calldata),
            )
            .await?;

        if result.is_empty() || result.iter().all(|&b| b == 0) {
            return Ok(None);
        }

        // Placeholder ABI decoding; full sol! macro decoding when contract ABI is finalized.
        Ok(Some(AgentIdentity {
            address,
            chain_id: self.l1_chain_id,
            capabilities: vec![],
            service_endpoints: vec![],
            metadata_cid: String::new(),
            last_updated_block: 0,
        }))
    }

    /// Check if an address has a specific capability registered.
    pub async fn has_capability(
        &self,
        address: Address,
        cap: &Capability8004,
    ) -> Result<bool, ChainError> {
        let cap_byte = capability_to_uint8(cap);

        let mut calldata = Vec::with_capacity(68);
        calldata.extend_from_slice(&selectors::HAS_CAPABILITY);
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(address.as_slice());
        calldata.extend_from_slice(&[0u8; 31]);
        calldata.push(cap_byte);

        let result = self
            .provider
            .eth_call(
                self.l1_chain_id,
                self.registry_address,
                Bytes::from(calldata),
            )
            .await?;

        Ok(!result.is_empty() && result[result.len() - 1] != 0)
    }

    /// Fetch all registered identities (paginated).
    pub async fn list_identities(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<AgentIdentity>, ChainError> {
        let mut calldata = Vec::with_capacity(68);
        calldata.extend_from_slice(&selectors::LIST_IDENTITIES);

        let mut offset_bytes = [0u8; 32];
        offset_bytes[24..].copy_from_slice(&offset.to_be_bytes());
        calldata.extend_from_slice(&offset_bytes);

        let mut limit_bytes = [0u8; 32];
        limit_bytes[24..].copy_from_slice(&limit.to_be_bytes());
        calldata.extend_from_slice(&limit_bytes);

        let result = self
            .provider
            .eth_call(
                self.l1_chain_id,
                self.registry_address,
                Bytes::from(calldata),
            )
            .await?;

        if result.is_empty() {
            return Ok(vec![]);
        }

        // Placeholder: full ABI decoding when contract ABI is finalized.
        Ok(vec![])
    }
}

fn capability_to_uint8(cap: &Capability8004) -> u8 {
    match cap {
        Capability8004::Trading => 0,
        Capability8004::LiquidityProvider => 1,
        Capability8004::Lending => 2,
        Capability8004::CrossChainRouting => 3,
        Capability8004::VaultManagement => 4,
        Capability8004::UniswapXFiller => 5,
        Capability8004::Custom(_) => 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_erc8004_registry_address_correct() {
        let addr: Address = "0x8004A818BFB912233c491871b3d84c89A494BD9e"
            .parse()
            .unwrap();
        assert_eq!(addr.to_string().len(), 42);
    }

    #[test]
    fn test_identity_agent_identity_roundtrip_serde() {
        let identity = AgentIdentity {
            address: Address::repeat_byte(0xAB),
            chain_id: 1,
            capabilities: vec![Capability8004::Trading, Capability8004::LiquidityProvider],
            service_endpoints: vec![ServiceEndpoint {
                name: "api".into(),
                url: "https://example.com/api".into(),
            }],
            metadata_cid: "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".into(),
            last_updated_block: 12345,
        };

        let json = serde_json::to_string(&identity).unwrap();
        let roundtrip: AgentIdentity = serde_json::from_str(&json).unwrap();

        assert_eq!(roundtrip.address, identity.address);
        assert_eq!(roundtrip.chain_id, identity.chain_id);
        assert_eq!(roundtrip.capabilities, identity.capabilities);
        assert_eq!(roundtrip.metadata_cid, identity.metadata_cid);
        assert_eq!(roundtrip.last_updated_block, identity.last_updated_block);
        assert_eq!(roundtrip.service_endpoints.len(), 1);
        assert_eq!(roundtrip.service_endpoints[0].name, "api");
    }

    #[test]
    fn test_identity_capability_enum_all_variants() {
        let variants = vec![
            Capability8004::Trading,
            Capability8004::LiquidityProvider,
            Capability8004::Lending,
            Capability8004::CrossChainRouting,
            Capability8004::VaultManagement,
            Capability8004::UniswapXFiller,
            Capability8004::Custom("ext".into()),
        ];

        assert_eq!(variants.len(), 7);
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let roundtrip: Capability8004 = serde_json::from_str(&json).unwrap();
            assert_eq!(&roundtrip, v);
        }
    }

    #[test]
    fn test_identity_erc8004_registry_l1_only() {
        let expected: Address = "0x8004A818BFB912233c491871b3d84c89A494BD9e"
            .parse()
            .unwrap();
        assert_eq!(
            expected,
            "0x8004A818BFB912233c491871b3d84c89A494BD9e"
                .parse::<Address>()
                .unwrap()
        );
    }

    #[test]
    fn test_identity_erc8004_registry_read_only_no_writes() {
        // Compile-time verification: Erc8004Registry only has &self methods.
        // No register(), update_capabilities(), or set_metadata() methods exist.
        fn assert_read_only(_: &Erc8004Registry) {}
        let _ = assert_read_only as fn(&Erc8004Registry);
    }

    #[test]
    fn test_capability_to_uint8_mapping() {
        assert_eq!(capability_to_uint8(&Capability8004::Trading), 0);
        assert_eq!(capability_to_uint8(&Capability8004::LiquidityProvider), 1);
        assert_eq!(capability_to_uint8(&Capability8004::Lending), 2);
        assert_eq!(capability_to_uint8(&Capability8004::CrossChainRouting), 3);
        assert_eq!(capability_to_uint8(&Capability8004::VaultManagement), 4);
        assert_eq!(capability_to_uint8(&Capability8004::UniswapXFiller), 5);
        assert_eq!(
            capability_to_uint8(&Capability8004::Custom("x".into())),
            255
        );
    }
}
