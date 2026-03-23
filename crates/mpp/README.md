# MPP — Machine Payment Protocol

Rust primitives for HTTP 402-based machine-to-machine payments. Sign an ERC-3009 authorization, attach it to your API request, get a receipt back. No accounts, no API keys, no invoices -- just a wallet and a signature.

MPP defines the types, verification logic, and session management needed to build both sides of a pay-per-request API.

## The protocol

Three steps:

```
Client                                Server
  |                                     |
  |  POST /v1/chat (no credential)      |
  | ----------------------------------> |
  |                                     |
  |  402 + PaymentRequired              |
  |  (amount, asset, recipient, nonce)  |
  | <---------------------------------- |
  |                                     |
  |  POST /v1/chat                      |
  |  X-Payment: {PaymentCredential}     |
  |  (ERC-3009 signed authorization)    |
  | ----------------------------------> |
  |                                     |
  |  200 + Payment-Receipt              |
  |  (amount_charged, tx_hash)          |
  | <---------------------------------- |
```

1. **Challenge.** Client sends a request without payment. Server estimates the cost and returns `402 Payment Required` with a `PaymentRequired` body: the amount in USDC base units, the token address, chain ID, recipient wallet, expiry, and a nonce.

2. **Credential.** Client reads the quote, constructs an ERC-3009 `transferWithAuthorization` for the quoted amount (or more), signs it with their wallet, and retries with the signature in the `X-Payment` header as a `PaymentCredential`.

3. **Receipt.** Server verifies the signature off-chain (no RPC call needed), serves the request, and returns a `PaymentReceipt` in the `Payment-Receipt` header with the actual amount charged and an optional on-chain tx hash.

## Two payment intents

**Charge** -- one-shot. Every request carries its own ERC-3009 signature. Simple but adds a signature round-trip per request.

**Session** -- pre-funded balance. The client opens a session with a single large authorization, then draws against that balance per-request without re-signing. When the session closes, unused funds are refunded.

```
Open session:     ERC-3009 for $5.00  →  session_id
Request 1:        session_id          →  drew $0.03, $4.97 remaining
Request 2:        session_id          →  drew $0.08, $4.89 remaining
...
Close session:    session_id          →  refund $4.12
```

Sessions are the right choice for streaming workloads, multi-turn conversations, or any workflow where you'd rather not sign on every call.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
mpp = { path = "../crates/mpp" }  # or from your registry
```

### Types only (lightest dependency)

If you just need the wire format types for serializing/deserializing MPP headers:

```toml
[dependencies]
mpp = { path = "../crates/mpp", default-features = false }
```

This gives you `PaymentRequired`, `PaymentCredential`, `PaymentReceipt`, `Erc3009Authorization`, and the rest of the protocol types. No session store, no database.

### Feature flags

| Feature   | Default | What it adds |
|-----------|---------|-------------|
| `session` | yes     | `SessionStore`, `MppSession`, `SessionSettlement` -- in-memory session management |
| `db`      | yes     | `MppDb` -- SQLite persistence for payment records and sessions |

## Building a server

### Verify a payment credential

The core server-side operation: a request arrives with an `X-Payment` header. Parse it, verify the ERC-3009 signature, and extract the payer address.

```rust
use mpp::{PaymentCredential, verify_authorization};
use alloy::primitives::{Address, U256};

// Parse the X-Payment header.
let credential: PaymentCredential = serde_json::from_str(&header_value)?;

// Your server's wallet address.
let my_wallet: Address = "0x...".parse()?;

// Verify: checks recipient, amount, time window, and recovers the signer.
verify_authorization(
    &credential.authorization,
    my_wallet,       // expected recipient
    U256::ZERO,      // minimum amount (0 = accept any)
)?;

// Signature is valid. credential.authorization.from is the payer.
let payer = credential.authorization.from;
let authorized_amount = credential.authorization.value;
```

This does pure cryptographic verification -- no RPC calls, no network requests. It recovers the signer from the EIP-712 typed data digest and checks it matches the claimed `from` address.

### Return a 402 challenge

When a request arrives without payment, build a `PaymentRequired` response:

```rust
use mpp::{PaymentRequired, PaymentIntent, CostBreakdown};
use mpp::currency::usdc_from_usd;
use mpp::verifier::USDC_BASE;
use alloy::primitives::FixedBytes;

let provider_cost = usdc_from_usd(0.03);  // estimated provider cost
let spread = usdc_from_usd(0.006);         // 20% markup
let total = provider_cost + spread;

let challenge = PaymentRequired {
    amount: total,
    asset: USDC_BASE,
    chain_id: 8453,  // Base
    recipient: my_wallet,
    expiry: now_secs() + 300,  // 5 minute validity
    intent: PaymentIntent::Charge,
    nonce: FixedBytes::random(),
    session_id: None,
    breakdown: Some(CostBreakdown {
        provider_cost,
        spread,
        spread_pct: 0.20,
    }),
};

// Serialize to JSON for the response body and X-Payment-Required header.
let body = serde_json::to_vec(&challenge)?;
```

### Manage sessions

```rust
use mpp::session::SessionStore;
use alloy::primitives::{Address, U256};

let store = SessionStore::new();

// Client opens a session with $5.00 pre-funded.
let session = store.open(
    "session-uuid".into(),
    payer_address,
    U256::from(5_000_000u64),  // $5.00 in USDC base units
    3600,                       // 1 hour TTL
);

// Per-request draws.
let remaining = store.draw("session-uuid", U256::from(30_000u64), "req-1", "claude-sonnet-4")?;
// remaining = $4.97

// Client or TTL closes the session.
let settlement = store.close("session-uuid").unwrap();
// settlement.refund_amount = whatever wasn't drawn
```

The `SessionStore` is thread-safe (`DashMap` internally). For persistence across restarts, pair it with `MppDb`.

### Issue receipts

After serving a request, return a receipt so the client has proof of payment:

```rust
use mpp::PaymentReceipt;

let receipt = PaymentReceipt {
    receipt_id: uuid::Uuid::new_v4().to_string(),
    amount_charged: actual_cost,
    amount_remaining: session_balance,  // None for Charge intent
    tx_hash: None,                      // filled after on-chain settlement
    session_id: None,
    timestamp: now_secs(),
};

// Serialize to the Payment-Receipt response header.
let header = serde_json::to_string(&receipt)?;
```

### Persist payment records

```rust
use mpp::db::MppDb;

let db = MppDb::open(&"payments.db".into())?;

db.record_payment(
    "receipt-id",
    "0xpayer...",
    "charge",
    None,           // session_id
    36_000,         // amount_quoted (USDC micro)
    33_000,         // amount_charged
    27_500,         // provider_cost_micro
    5_500,          // spread_micro
    0.20,           // spread_pct
    "claude-sonnet-4",
).await?;
```

Creates two tables: `mpp_payments` (every transaction) and `mpp_sessions` (session lifecycle). WAL mode, indexed on payer and session_id.

## Building a client

### Parse a 402 response

```rust
use mpp::PaymentRequired;

// From the response body or X-Payment-Required header.
let challenge: PaymentRequired = serde_json::from_slice(&body)?;

println!("Pay {} to {} on chain {}", challenge.amount, challenge.recipient, challenge.chain_id);
println!("Expires at {}", challenge.expiry);

if let Some(breakdown) = &challenge.breakdown {
    println!("Provider cost: {}, spread: {} ({}%)",
        breakdown.provider_cost, breakdown.spread, breakdown.spread_pct * 100.0);
}
```

### Construct a credential

```rust
use mpp::{PaymentCredential, PaymentIntent, Erc3009Authorization};
use alloy::primitives::{FixedBytes, U256};

// Sign an ERC-3009 transferWithAuthorization with your wallet.
// (The actual signing uses your wallet's private key via alloy/ethers.)
let authorization = Erc3009Authorization {
    from: my_address,
    to: challenge.recipient,
    value: challenge.amount,
    valid_after: U256::ZERO,
    valid_before: U256::from(challenge.expiry),
    nonce: challenge.nonce,
    v, r, s,  // from your EIP-712 signature
};

let credential = PaymentCredential {
    intent: PaymentIntent::Charge,
    authorization,
    session_id: None,
    session_op: None,
};

// Serialize and attach as X-Payment header.
let header = serde_json::to_string(&credential)?;
```

### Parse a receipt

```rust
use mpp::PaymentReceipt;

let receipt: PaymentReceipt = serde_json::from_str(&header_value)?;
println!("Charged: {}", receipt.amount_charged);

if let Some(remaining) = receipt.amount_remaining {
    println!("Session balance: {}", remaining);
}
```

## Custom tokens and chains

The verifier defaults to USDC on Base (chain 8453). For other ERC-3009-compatible tokens:

```rust
use mpp::verifier::{Erc3009Domain, verify_authorization_with_domain};

let domain = Erc3009Domain {
    name: "USD Coin".into(),
    version: "2".into(),
    chain_id: 1,  // Ethereum mainnet
    verifying_contract: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?,
};

verify_authorization_with_domain(
    &credential.authorization,
    my_wallet,
    U256::ZERO,
    &domain,
)?;
```

Any token that implements ERC-3009 `transferWithAuthorization` works. The verification is pure EIP-712 signature recovery -- it doesn't call the token contract.

## Reputation-based pricing

The `spread` module defines reputation tiers that map to spread percentages. Higher reputation means lower markup:

```rust
use mpp::spread::ReputationTier;

let tier = ReputationTier::for_address(payer_address).await;
let spread = tier.spread();  // 0.20 for None, 0.08 for Sovereign
```

| Tier | Spread | How you get it |
|------|--------|---------------|
| None | 20% | Default, no on-chain identity |
| Basic | 18% | Basic attestation |
| Verified | 15% | KYC or verified identity |
| Trusted | 12% | Established usage history |
| Sovereign | 8% | Full on-chain reputation |

The tier lookup is currently a stub that returns `None`. Production implementations would query an on-chain registry (e.g. ERC-8004).

## Workflows this enables

**Pay-per-request inference.** An AI agent calls an API, pays per token with USDC. No API key provisioning, no billing cycles, no credit card on file. The agent's wallet IS its identity.

**Metered tool access.** An MCP server charges per tool call. The client opens a session at the start of a task, tools draw against the balance, and unused funds refund on close.

**Multi-agent billing.** A swarm of agents each hold their own wallet. The orchestrator funds sessions for each agent. Per-agent cost tracking comes free from the session draw records.

**Streaming with budget caps.** Open a session with your budget ceiling. The server draws per-chunk. When the session balance hits zero, the stream stops. No surprise bills.

**Provider-agnostic payment.** Any API server can add MPP. The types are generic -- not tied to any specific inference provider, model, or pricing structure. The `PaymentRequired` response tells the client exactly what it needs to pay.

## Currency utilities

```rust
use mpp::currency::{usdc_from_usd, usd_from_usdc};
use alloy::primitives::U256;

let amount = usdc_from_usd(1.50);           // U256: 1_500_000
let usd = usd_from_usdc(U256::from(2_500_000u64));  // f64: 2.50
```

USDC uses 6 decimal places. These helpers handle the conversion so you don't have to think about it.
