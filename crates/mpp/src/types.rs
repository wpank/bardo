//! Core MPP (Machine Payment Protocol) types.
//!
//! These types define the wire format for the 402 challenge/credential/receipt
//! flow. They serialize to JSON for HTTP headers and response bodies.

use alloy::primitives::{Address, FixedBytes, U256};
use serde::{Deserialize, Serialize};

/// Payment intent type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentIntent {
    /// One-shot per-request billing.
    Charge,
    /// Streaming pay-as-you-go session.
    Session,
}

/// 402 response payload sent in both the body and `X-Payment-Required` header.
///
/// The server returns this when a request arrives without a valid `X-Payment`
/// credential. The client reads it, signs an ERC-3009 authorization for the
/// quoted amount, and retries with the credential attached.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequired {
    /// Amount in token base units (USDC: 6 decimals, so 1_000_000 = $1.00).
    pub amount: U256,
    /// Token contract address (e.g. USDC on Base).
    pub asset: Address,
    /// Chain ID (e.g. 8453 = Base).
    pub chain_id: u64,
    /// Recipient address (server's wallet).
    pub recipient: Address,
    /// Unix timestamp after which this quote expires.
    pub expiry: u64,
    /// Payment intent type.
    pub intent: PaymentIntent,
    /// Unique nonce for this payment request (prevents replay).
    pub nonce: FixedBytes<32>,
    /// Session ID for Session intent operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Cost breakdown for transparency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<CostBreakdown>,
}

/// Transparent cost breakdown in the 402 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Estimated provider cost in token base units.
    pub provider_cost: U256,
    /// Spread amount in token base units.
    pub spread: U256,
    /// Spread percentage (e.g. 0.20 = 20%).
    pub spread_pct: f64,
}

/// Payment credential sent by the client in the `X-Payment` header.
///
/// Contains an ERC-3009 `transferWithAuthorization` signature that the server
/// can submit on-chain to pull funds from the client's wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCredential {
    /// Intent type.
    pub intent: PaymentIntent,
    /// ERC-3009 transferWithAuthorization signature.
    pub authorization: Erc3009Authorization,
    /// Session ID (required for Session draws/top-ups/close).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Session operation type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_op: Option<SessionOp>,
}

/// ERC-3009 `transferWithAuthorization` parameters.
///
/// The `from` address signs these parameters using EIP-712 typed data.
/// The server verifies the signature off-chain before accepting the payment,
/// then submits it on-chain for settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erc3009Authorization {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub valid_after: U256,
    pub valid_before: U256,
    pub nonce: FixedBytes<32>,
    pub v: u8,
    pub r: FixedBytes<32>,
    pub s: FixedBytes<32>,
}

/// Session lifecycle operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOp {
    Open,
    TopUp,
    Draw,
    Close,
}

/// Payment receipt returned in the `Payment-Receipt` response header.
///
/// Proves the server acknowledged the payment. Contains the actual amount
/// charged (which may differ from the quoted amount for session draws).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReceipt {
    /// Unique receipt ID.
    pub receipt_id: String,
    /// Amount actually charged in token base units.
    pub amount_charged: U256,
    /// Remaining balance (Session intent only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_remaining: Option<U256>,
    /// On-chain settlement tx hash (None until settled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    /// Session ID (Session intent only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Unix timestamp.
    pub timestamp: u64,
}

/// Attached to request extensions after credential validation.
///
/// Server middleware validates the credential and attaches this context
/// so downstream handlers know who is paying, how much was authorized,
/// and which session (if any) is active.
#[derive(Debug, Clone)]
pub struct PaymentContext {
    pub intent: PaymentIntent,
    pub payer: Address,
    pub authorized_amount: U256,
    pub session_id: Option<String>,
    pub credential: PaymentCredential,
}

/// Auth mode tag for request routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    /// Traditional API key authentication.
    ApiKey,
    /// Machine Payment Protocol.
    Mpp,
}
