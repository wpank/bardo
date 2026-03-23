//! ERC-3009 off-chain signature verification.
//!
//! Verifies `transferWithAuthorization` signatures using EIP-712 typed data
//! recovery. No RPC calls needed -- pure cryptographic verification.

use alloy::primitives::{Address, B256, Signature, U256, address, keccak256};

use crate::error::MppError;
use crate::types::Erc3009Authorization;

/// EIP-712 domain parameters for an ERC-3009-compatible token.
///
/// Use [`Erc3009Domain::usdc_base`] for USDC on Base, or construct your own
/// for other tokens/chains.
#[derive(Debug, Clone)]
pub struct Erc3009Domain {
    /// Token name (e.g. "USD Coin").
    pub name: String,
    /// Token version (e.g. "2").
    pub version: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Token contract address.
    pub verifying_contract: Address,
}

/// USDC contract address on Base.
pub const USDC_BASE: Address = address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");

/// Base chain ID.
pub const BASE_CHAIN_ID: u64 = 8453;

impl Erc3009Domain {
    /// USDC on Base (chain 8453).
    pub fn usdc_base() -> Self {
        Self {
            name: "USD Coin".to_string(),
            version: "2".to_string(),
            chain_id: BASE_CHAIN_ID,
            verifying_contract: USDC_BASE,
        }
    }

    /// Compute the EIP-712 domain separator for this token.
    pub fn domain_separator(&self) -> B256 {
        let domain_type_hash = keccak256(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        );
        let name_hash = keccak256(self.name.as_bytes());
        let version_hash = keccak256(self.version.as_bytes());

        let mut buf = Vec::with_capacity(5 * 32);
        buf.extend_from_slice(domain_type_hash.as_slice());
        buf.extend_from_slice(name_hash.as_slice());
        buf.extend_from_slice(version_hash.as_slice());
        buf.extend_from_slice(&U256::from(self.chain_id).to_be_bytes::<32>());
        buf.extend_from_slice(self.verifying_contract.into_word().as_slice());

        keccak256(&buf)
    }
}

/// ERC-3009 TransferWithAuthorization type hash.
fn transfer_with_auth_type_hash() -> B256 {
    keccak256(b"TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)")
}

/// Compute the EIP-712 struct hash for a TransferWithAuthorization.
fn struct_hash(auth: &Erc3009Authorization) -> B256 {
    let type_hash = transfer_with_auth_type_hash();
    let mut buf = Vec::with_capacity(7 * 32);
    buf.extend_from_slice(type_hash.as_slice());
    buf.extend_from_slice(auth.from.into_word().as_slice());
    buf.extend_from_slice(auth.to.into_word().as_slice());
    buf.extend_from_slice(&auth.value.to_be_bytes::<32>());
    buf.extend_from_slice(&auth.valid_after.to_be_bytes::<32>());
    buf.extend_from_slice(&auth.valid_before.to_be_bytes::<32>());
    buf.extend_from_slice(auth.nonce.as_slice());
    keccak256(&buf)
}

/// Compute the full EIP-712 digest: `keccak256(0x1901 || domainSeparator || structHash)`.
pub fn eip712_digest(domain: &Erc3009Domain, auth: &Erc3009Authorization) -> B256 {
    let domain_sep = domain.domain_separator();
    let s_hash = struct_hash(auth);

    let mut buf = Vec::with_capacity(2 + 32 + 32);
    buf.extend_from_slice(&[0x19, 0x01]);
    buf.extend_from_slice(domain_sep.as_slice());
    buf.extend_from_slice(s_hash.as_slice());

    keccak256(&buf)
}

/// Verify an ERC-3009 authorization off-chain using ecrecover.
///
/// Checks:
/// 1. `to` matches the expected recipient
/// 2. `value` >= minimum required amount
/// 3. Current time is between `valid_after` and `valid_before`
/// 4. Recovered signer matches `from`
///
/// Uses USDC on Base domain by default. For other tokens, use
/// [`verify_authorization_with_domain`].
pub fn verify_authorization(
    auth: &Erc3009Authorization,
    expected_recipient: Address,
    min_amount: U256,
) -> Result<(), MppError> {
    verify_authorization_with_domain(
        auth,
        expected_recipient,
        min_amount,
        &Erc3009Domain::usdc_base(),
    )
}

/// Verify an ERC-3009 authorization against a specific token domain.
pub fn verify_authorization_with_domain(
    auth: &Erc3009Authorization,
    expected_recipient: Address,
    min_amount: U256,
    domain: &Erc3009Domain,
) -> Result<(), MppError> {
    // Check recipient.
    if auth.to != expected_recipient {
        return Err(MppError::InvalidRecipient);
    }

    // Check amount.
    if auth.value < min_amount {
        return Err(MppError::InsufficientAmount {
            needed: min_amount.to_string(),
            got: auth.value.to_string(),
        });
    }

    // Check time window.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let valid_after: u64 = auth.valid_after.try_into().unwrap_or(0);
    let valid_before: u64 = auth.valid_before.try_into().unwrap_or(0);

    if now < valid_after || (valid_before > 0 && now > valid_before) {
        return Err(MppError::ExpiredAuthorization);
    }

    // Recover signer from EIP-712 typed data digest.
    let digest = eip712_digest(domain, auth);
    let signature = Signature::new(
        U256::from_be_bytes::<32>(auth.r.into()),
        U256::from_be_bytes::<32>(auth.s.into()),
        auth.v != 0 && auth.v != 27, // normalize: 0/27 = false, 1/28 = true
    );

    let recovered = signature
        .recover_address_from_prehash(&digest)
        .map_err(|e| MppError::RecoveryFailed(e.to_string()))?;

    if recovered != auth.from {
        return Err(MppError::InvalidSignature(format!(
            "expected {}, recovered {}",
            auth.from, recovered
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_separator_is_deterministic() {
        let domain = Erc3009Domain::usdc_base();
        let d1 = domain.domain_separator();
        let d2 = domain.domain_separator();
        assert_eq!(d1, d2);
    }

    #[test]
    fn type_hash_is_correct() {
        let th = transfer_with_auth_type_hash();
        assert_ne!(th, B256::ZERO);
    }

    #[test]
    fn custom_domain() {
        let domain = Erc3009Domain {
            name: "Test Token".to_string(),
            version: "1".to_string(),
            chain_id: 1,
            verifying_contract: Address::ZERO,
        };
        let sep = domain.domain_separator();
        assert_ne!(sep, B256::ZERO);
        // Different from USDC Base.
        assert_ne!(sep, Erc3009Domain::usdc_base().domain_separator());
    }
}
