# Bardo Tools -- Wallet Architecture [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md)
>
> Wallet provider architecture, Alloy Signer trait, owner/agent responsibilities, Privy integration, capability token flow, and identity custody.

---

> **Reader orientation:** This document specifies the wallet architecture for the `bardo-tools` crate, covering custody modes, the Alloy Signer abstraction, capability token flow, and identity custody. It belongs to Bardo's DeFi tool layer. A Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) interacts with on-chain protocols through the wallet system defined here. Familiarity with `01-architecture.md` and the PolicyCage (on-chain smart contract enforcing safety constraints on all agent actions) concept is assumed. See `prd2/shared/glossary.md` for full term definitions.

## Owner vs. agent responsibilities

| Responsibility        | Owner                                                | Agent                                   |
| --------------------- | ---------------------------------------------------- | --------------------------------------- |
| Custody mode          | Selects Delegation, Embedded, or LocalKey            | Operates within chosen mode             |
| Wallet provisioning   | Creates/configures wallet or signs delegation        | Receives wallet handle                  |
| Policy setting        | Sets spending limits, allowlists, chain restrictions | Operates within policy                  |
| Key custody           | Chooses provider, manages recovery                   | Never sees private keys                 |
| Identity registration | Provides ERC-8004 (on-chain agent identity standard tracking capabilities, milestones, and reputation) metadata                           | Signs registration tx                   |
| Funding               | Signs delegation (Delegation mode) or seeds wallet   | Manages operating capital within budget |

---

## Three custody modes

The system supports three custody modes. They are not graduated tiers -- they are fundamentally different trust models. An owner selects one at provisioning time.

| Mode | Trust Model | Key Location | Fund Location | Death Settlement |
| --- | --- | --- | --- | --- |
| **Delegation** | On-chain caveats (ERC-7710/7715) | Disposable session key in process memory | Owner's MetaMask Smart Account | No sweep -- delegation expires |
| **Embedded** (Privy) | Off-chain TEE policy | AWS Nitro Enclaves | Privy server wallet | Sweep required |
| **LocalKey** | On-chain delegation bounds | Local disk or memory | Owner's Smart Account or local EOA | Delegation expires |

**Delegation** is recommended. Funds never leave the owner's wallet. The Golem holds a disposable session key and a signed delegation. Every transaction executes FROM the owner's address. If the session key leaks, the attacker is bounded by caveats. The owner revokes from MetaMask directly -- no Golem cooperation needed.

**Embedded** (Privy) is the legacy mode. Funds transfer to a Privy server wallet running in AWS Nitro Enclaves. Policy enforcement is off-chain (inside the TEE). Simpler to set up, but the owner surrenders direct custody.

**LocalKey** is for developers. A locally generated keypair bounded by an on-chain delegation. The key is insecure in the traditional sense -- no TEE, no HSM. The paradigm shift: instead of "keep the key secret," the system says "bound the damage if the key leaks."

## Wallet abstraction

All custody modes are normalized to Alloy's `Signer` trait. The `WalletHandle` wraps the mode-specific signer and exposes a uniform interface.

```rust
use alloy::signers::Signer;

/// Bardo wallet abstraction: wraps any custody mode behind Alloy's Signer trait.
/// Delegation mode: signs UserOps with session key, redeems against owner's Smart Account.
/// Embedded mode: signs via Privy TEE API call.
/// LocalKey mode: signs with local keypair.
pub struct WalletHandle {
    signer: Arc<dyn Signer + Send + Sync>,
    address: Address,
    chain_ids: Vec<u64>,
    custody_mode: CustodyMode,
}

/// The three custody modes as a Rust enum.
#[derive(Debug, Clone)]
pub enum CustodyMode {
    /// Recommended. Funds stay in the owner's MetaMask Smart Account.
    Delegation {
        smart_account: Address,
        delegation: SignedDelegation,
        caveat_enforcers: Vec<CaveatEnforcer>,
    },
    /// Legacy. Funds transferred to Privy server wallet.
    Embedded {
        privy_app_id: String,
        server_wallet_id: String,
    },
    /// Dev/self-hosted. Local key with on-chain delegation bounds.
    LocalKey {
        private_key_path: PathBuf,
        delegation_bounds: DelegationBounds,
    },
}

impl WalletHandle {
    pub fn address(&self) -> Address {
        self.address
    }

    /// Sign a transaction. In Delegation mode, this builds a UserOp
    /// that redeems the delegation from the owner's Smart Account.
    pub async fn sign_transaction(
        &self,
        tx: &mut TransactionRequest,
    ) -> Result<TxEnvelope> {
        let signature = self.signer.sign_transaction(tx).await?;
        Ok(tx.build(signature))
    }

    pub async fn sign_message(&self, message: &[u8]) -> Result<Signature> {
        self.signer.sign_message(message).await
    }

    pub async fn sign_typed_data<T: SolStruct>(
        &self,
        data: &T,
        domain: &Eip712Domain,
    ) -> Result<Signature> {
        self.signer.sign_typed_data(data, domain).await
    }
}
```

The `ToolContext` holds a `WalletHandle`. Write tool handlers access it via `ctx.signer(chain_id)`, which returns an `Arc<dyn Signer>` configured for the specified chain.

---

## Capability token flow

Write tools cannot execute without a `Capability<T>` token. The wallet participates in capability token creation because spending limits and policy enforcement are wallet-aware.

```text
1. Golem requests write action via preview_action
2. Safety extension evaluates action:
   a. Check token allowlist
   b. Simulate via revm fork
   c. Verify USD value against capability limits
   d. Verify rate limits
3. Safety extension mints Capability<T> with:
   - value_limit: min(requested_value, wallet_policy_max)
   - expires_at: current_tick + BARDO_CAP_EXPIRY_TICKS
   - policy_hash: SHA-256 of current PolicyCage state
   - permit_id: unique audit trail identifier
4. Golem confirms via commit_action, passing the Capability<T>
5. Write tool handler receives Capability<T> by move -- consumed on use
6. Handler calls ctx.signer(chain_id) to sign the transaction
7. Wallet provider (Privy TEE, Safe, etc.) enforces its own policy layer
8. Transaction broadcast
```

The `Capability<T>` token is unforgeable -- its constructor is `pub(crate)`, so only the safety extension within `bardo-tools` can mint one. The token is consumed by Rust's move semantics: calling `execute_write()` transfers ownership, making reuse a compile error.

For Privy wallets, step 7 adds a second enforcement boundary: even if the capability token is valid, Privy's TEE policy can independently reject the transaction. This defense-in-depth means no single compromised layer can authorize fund loss.

---

## Supported wallet providers

### 1. MetaMask Smart Account (Delegation mode -- recommended)

**Config**: `custody.mode = "delegation"` in `golem.toml`

The owner's MetaMask Smart Account (ERC-7710/7715). The Golem holds a disposable session key that signs UserOperations. The DelegationManager validates caveat enforcers before execution. See [18-tools-metamask.md](18-tools-metamask.md) for the full delegation architecture, seven caveat enforcers, and execution flow.

### 2. Local key (development only)

**Config**: `BARDO_WALLET_PRIVATE_KEY=0x...`

Alloy `LocalSigner::from_str()`. No policy enforcement, no TEE protection. For development and testing only. Can be combined with on-chain delegation bounds (LocalKey custody mode) for bounded insecurity. The library logs a warning on startup when a local key is detected.

### 3. Privy server wallet (Embedded mode)

**Config**: `BARDO_PRIVY_APP_ID`, `BARDO_PRIVY_APP_SECRET`

Privy server wallets provide TEE-backed key management with a programmable policy engine. The private key never leaves Privy's TEE -- the library receives only signed transactions.

**Privy integration via reqwest**: The library communicates with Privy's API using `reqwest` as the HTTP client. Request signing uses P-256 (secp256r1) keypairs for authentication.

```rust
use reqwest::Client;

pub struct PrivySigner {
    client: Client,
    app_id: String,
    wallet_id: String,
    auth_key: p256::ecdsa::SigningKey,
}

impl PrivySigner {
    /// Sign a transaction via Privy's TEE.
    pub async fn sign_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<Signature> {
        let body = serde_json::json!({
            "walletId": self.wallet_id,
            "chainType": "ethereum",
            "method": "eth_signTransaction",
            "params": { "transaction": tx },
        });
        let auth_sig = self.sign_request(&body)?;
        let resp = self.client
            .post(format!("https://auth.privy.io/api/v1/wallets/{}/rpc", self.wallet_id))
            .header("privy-app-id", &self.app_id)
            .header("authorization", format!("Bearer {}", auth_sig))
            .json(&body)
            .send()
            .await?;
        // Parse signature from response
        Ok(parse_signature(resp.json().await?)?)
    }
}
```

**Privy policy engine**: Policies are enforced by Privy's infrastructure (not by the tool library), creating a cryptographic enforcement boundary that cannot be bypassed even by a fully compromised LLM.

Policy rules:

- **Allowed contracts**: Restrict signing to specific contract addresses (Universal Router, Permit2, V3/V4 Position Managers)
- **Method restrictions**: Restrict to specific function selectors
- **Value limits**: Per-transaction ETH value caps
- **Chain restrictions**: Restrict to specific chain IDs
- **Rate limiting**: Transactions per time window

**Authorization key best practices**:

- Separate keys for transaction signing vs policy management
- Rotation every 90-180 days (add new key before removing old)
- Store in secrets manager for remote deployments

### 3. Safe (multi-sig / smart account)

**Config**: `BARDO_SAFE_ADDRESS`, `BARDO_SAFE_SIGNER_KEY`

Safe smart accounts enable multi-signature workflows. The tool library proposes transactions; other signers approve via the Safe transaction service. Supports ERC-4337 UserOperations via bundlers.

### 4. ZeroDev (ERC-4337 account abstraction)

**Config**: `BARDO_ZERODEV_PROJECT_ID`

ZeroDev kernel accounts with session keys, gas sponsorship, and batched transactions. Enables gasless operations for agents.

### 5. Lit Protocol (Vincent)

**Config**: `BARDO_LIT_AUTH_TOKEN`

Decentralized key management via Lit Protocol's Vincent framework. Keys are managed by a decentralized network of nodes -- no single point of compromise.

### 6. BardoWallet

**Config**: `BARDO_WALLET_ID`

Bardo's native wallet abstraction. Wraps Privy server wallets with Bardo-specific policy templates, automatic ERC-8004 identity registration, and integrated spending limit tracking.

### 7. Generic Alloy signer

**Config**: Custom `impl Signer` object via programmatic API

Any type implementing `alloy::signers::Signer`. For custom signers, HSMs, or novel wallet types. Pass the signer directly when constructing the `ToolContext` programmatically.

---

## Verified contract addresses

The following Uniswap contract addresses are hardcoded (never user-provided) for wallet policy allowlists:

| Contract                      | Base Mainnet Address                         | Required for         |
| ----------------------------- | -------------------------------------------- | -------------------- |
| SwapRouter02 (V3)             | `0x2626664c2603336E57B271c5C0b26F421741e481` | V3 swaps             |
| Universal Router              | `0x198EF79F1F515F02dFE9e3115eD9fC07A3a63800` | All Uniswap versions |
| V4 PoolManager                | `0x498581ff718922c3f8e6a244956af099b2652b2b` | V4 pool interactions |
| V4 PositionManager            | `0x7C5f5A4bBd8fD42D3ACEC2747a3236742C5A6BA6` | V4 LP positions      |
| Permit2                       | `0x000000000022D473030F116dDEE9F6B43aC78BA3` | Token approvals      |
| V3 NonfungiblePositionManager | `0x03a520b32C04BF3bEEf7BEb72E919cf822Ed34f1` | V3 LP positions      |
| WETH                          | `0x4200000000000000000000000000000000000006` | Wrapped ETH on Base  |

**Minimum allowlist** (5 addresses): SwapRouter02, Universal Router, PoolManager, Permit2, and the owner's cold wallet. All other contract interactions are denied by default.

---

## ERC-4626 function selectors

For vault integration, wallet policies must allowlist these function selectors:

| Function                            | Selector     | Operation       |
| ----------------------------------- | ------------ | --------------- |
| `deposit(uint256,address)`          | `0x6e553f65` | Deposit assets  |
| `withdraw(uint256,address,address)` | `0xb460af94` | Withdraw assets |
| `redeem(uint256,address,address)`   | `0xba087652` | Redeem shares   |

---

## Session key management (ERC-7715)

Session keys provide scoped, time-limited signing permissions without exposing the root wallet key. Requires a smart account (ERC-4337).

### `wallet_provision_session_key`

Create a new session key with scoped permissions and a time-limited expiry.

| Parameter     | Type     | Required | Description                                  |
| ------------- | -------- | -------- | -------------------------------------------- |
| `permissions` | `array`  | Yes      | `[{ contract, methods, maxAmount, chains }]` |
| `expiry`      | `number` | Yes      | Seconds from now (max: 30 days = 2,592,000)  |
| `label`       | `string` | No       | Human-readable label for identification      |

**Returns**: `{ keyId, publicKey, permissions, expiresAt }`

### `wallet_revoke_session_key`

Revoke one or more active session keys. Omitting `keyIds` revokes all active keys.

| Parameter | Type     | Required | Description                               |
| --------- | -------- | -------- | ----------------------------------------- |
| `keyIds`  | `array`  | No       | Specific key IDs to revoke (omit for ALL) |
| `reason`  | `string` | No       | Reason for audit trail                    |

### `wallet_list_session_keys`

List all active session keys with their permissions, expiry timestamps, and usage statistics.

**Returns**: `{ keys: [{ keyId, label, permissions, expiresAt, usageCount, lastUsed }] }`

---

## Wallet management tools (4)

### `wallet_configure_policy`

Configure the wallet's policy engine (Privy TEE or smart account guards). Policy templates map common agent roles to appropriate restriction sets.

| Parameter        | Type     | Required | Description                                                                           |
| ---------------- | -------- | -------- | ------------------------------------------------------------------------------------- |
| `policyTemplate` | `string` | No       | "vault-participant", "vault-manager", "defi-trading", "read-only"                     |
| `customPolicy`   | `object` | No       | Custom: `{ allowedContracts, spendingLimits, chainRestrictions, methodRestrictions }` |

### `wallet_fund`

Fund the agent wallet with gas tokens and operating capital from an owner-controlled source.

| Parameter | Type     | Required | Description                                   |
| --------- | -------- | -------- | --------------------------------------------- |
| `token`   | `string` | Yes      | Token symbol or address (or "ETH" for native) |
| `amount`  | `string` | Yes      | Amount to transfer                            |
| `chain`   | `string` | Yes      | Chain name or chain ID                        |
| `from`    | `string` | No       | Source address (default: owner wallet)     |

### `wallet_migrate`

Migrate all assets from one wallet to another. Execution order: ERC-20 transfers -> NFT `safeTransferFrom` -> native ETH last. Risk Tier: Layer 3 (Warden delay, requires optional Warden module).

| Parameter           | Type      | Required | Description                                        |
| ------------------- | --------- | -------- | -------------------------------------------------- |
| `sourceWallet`      | `string`  | No       | Source address (default: agent wallet)             |
| `destinationWallet` | `string`  | Yes      | Target wallet address                              |
| `chain`             | `string`  | Yes      | Chain name or chain ID                             |
| `gasReserveEth`     | `string`  | No       | Reserve for gas (default: "0.01")                  |
| `includeNfts`       | `boolean` | No       | Include LP NFTs (default: true)                    |
| `includeTokens`     | `boolean` | No       | Include ERC-20s (default: true)                    |
| `tokens`            | `array`   | No       | Specific tokens to migrate (default: all non-zero) |
| `minBalanceUsd`     | `number`  | No       | Dust filter threshold (default: 0.01)              |
| `dryRun`            | `boolean` | No       | Preview only, no execution (default: false)        |

### `wallet_get_status`

Get comprehensive wallet status: balances, active policies, session keys, pending operations, and health checks.

| Parameter | Type     | Required | Description  |
| --------- | -------- | -------- | ------------ |
| `chain`   | `string` | No       | Chain filter |

**Returns**: `{ address, provider, chains[], balances[], activePolicy, sessionKeys[], pendingOperations[], healthChecks }`

---

## Identity NFT custody

When the tool library registers the agent's ERC-8004 identity:

1. The identity NFT is minted to the agent's wallet address
2. The wallet becomes both the `owner` and `agent` of the identity
3. Guardian protection (IdentityGuardian contract) adds a time-delay to ownership transfers
4. Wallet rotation requires identity migration via `IdentityGuardian.transferWithDelay()`

### Provider comparison for identity custody

| Provider      | Key Rotation     | Identity NFT Custody     | Recommendation                                      |
| ------------- | ---------------- | ------------------------ | --------------------------------------------------- |
| Local Key     | No               | Not recommended          | No transfer protection                              |
| **Privy**     | Via ERC-4337     | Recommended (smart acct) | Layer ZeroDev on top for session key separation     |
| ZeroDev       | Yes (native)     | Recommended              | sudo + session key separation                       |
| Safe          | Yes (native)     | Recommended              | `swapOwner()` + Guards block unauthorized transfers |
| Lit (Vincent) | Via MPC rotation | Suitable                 | Censorship resistance at the cost of latency        |

**Critical**: EOA agents must migrate identity NFTs to a smart contract wallet before production. EOA key rotation triggers 30-day reputation decay.

---

## Credential lifecycle

```text
1. Owner configures wallet provider credentials in env / config
2. Startup: validate credentials, connect to provider
3. If BardoWallet: auto-create Privy server wallet if none exists
4. Generate/load P-256 authorization key (Privy)
5. Register authorization key with provider
6. Ready: all tool operations use the configured wallet
7. Runtime: every signing request authenticated via P-256 key
8. Rotation: bardo-tools setup --rotate-keys rotates auth keys
```

### Provider-specific credential rotation

| Provider  | Credential       | Rotation Method                        | Impact                               |
| --------- | ---------------- | -------------------------------------- | ------------------------------------ |
| Privy     | App Secret       | Regenerate via console.privy.io        | Wallet unaffected                    |
| Privy     | Auth key (P-256) | Add new key to quorum, remove old      | Wallet unaffected                    |
| Safe      | Owner key        | `swapOwner()` on-chain                 | Address unchanged                    |
| Local Key | Private key      | Cannot rotate without changing address | New address, 30-day reputation decay |

---

## Golem integration

Every Golem receives a wallet at creation time. The wallet is the Golem's identity -- its ERC-8004 agent identity NFT is minted to the wallet address, and all x402 (micropayment protocol; agents pay for inference/compute/data via signed USDC transfers, no API keys) micropayments flow through it.

When a Golem dies (budget reaches zero), the wallet's remaining dust is swept to the Clade's (sibling Golems sharing a common ancestor, exchanging knowledge through Styx) treasury address. The identity NFT is not burned -- it remains as a historical record of the Golem's existence, with its reputation score preserved on-chain.

When a successor Golem is created, it receives a fresh wallet. It does NOT inherit the predecessor's wallet or identity. It inherits knowledge (via the Grimoire, the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links), not assets or identity.

---

## Pi integration: promptSnippet and promptGuidelines

### Session key tools

#### `wallet_provision_session_key`

**promptSnippet** (injected near system prompt, cached):
> Creates a scoped, time-limited signing key for the agent's smart account. One-time setup per session. Requires ERC-4337 smart account.

**promptGuidelines** (phase-conditional):
- **thriving**: Create session keys with full configured permissions. Set expiry up to 30 days for long-running strategies.
- **cautious**: Reduce session key scope to close/withdraw only. Set expiry to 7 days max.
- **declining**: Do not provision new session keys. Existing keys remain valid until expiry.
- **terminal**: Revoke all session keys via `wallet_revoke_session_key` instead.

**Risk tier**: Layer 2. Triggers: PolicyCage permission validation, smart account ownership check.
**Events**: `tool:start` -> `{ keyType: "session", permissions }` -> `tool:end` -> `{ keyId, expiresAt }`
**Pi hooks**: `tool_call` (verify smart account type, validate permission scope against PolicyCage), `tool_result` (log key creation to Grimoire)

#### `wallet_revoke_session_key`

**promptSnippet** (injected near system prompt, cached):
> Revokes active session keys. Omit keyIds to revoke all. Safe to call at any phase. Use proactively when reducing risk posture.

**promptGuidelines** (phase-conditional):
- **thriving**: Revoke only specific compromised or expired keys.
- **cautious**: Revoke keys with broad permissions; keep narrow-scope keys active.
- **declining**: Revoke all keys except the one used for settlement operations.
- **terminal**: Revoke all keys immediately. This is part of the Death Protocol sequence.

**Risk tier**: Layer 1. Triggers: ownership verification only.
**Events**: `tool:start` -> `{ keyIds, revokeAll: boolean }` -> `tool:end` -> `{ revokedCount }`
**Pi hooks**: `tool_call` (verify caller owns the keys), `tool_result` (emit `session_key_revoked` event)

#### `wallet_list_session_keys`

**promptSnippet** (injected near system prompt, cached):
> Lists active session keys with permissions, expiry, and usage stats. Read-only. Call to audit key hygiene before provisioning new keys.

**promptGuidelines** (phase-conditional):
- **thriving**: Periodic audit -- check for stale keys with zero usage.
- **cautious**: Check key expiry dates against projected lifespan.
- **declining**: Verify only settlement-scoped keys remain active.
- **terminal**: Confirm all keys revoked (list should be empty).

**Risk tier**: Layer 1. Read-only, no state change.
**Events**: `tool:start` -> `tool:end` -> `{ activeKeyCount }`
**Pi hooks**: `tool_call` (none), `tool_result` (none)

### Wallet management tools

#### `wallet_configure_policy`

**promptSnippet** (injected near system prompt, cached):
> Sets the wallet's policy engine rules (Privy TEE or smart account guards). Use policy templates for standard setups. Custom policies for advanced configurations.

**promptGuidelines** (phase-conditional):
- **thriving**: Apply "defi-trading" or "vault-manager" templates based on strategy.
- **cautious**: Tighten to "vault-participant" template. Reduce spending limits by 50%.
- **declining**: Switch to "read-only" template. Block all outbound transfers except to operator cold wallet.
- **terminal**: Lock to "read-only". No policy changes permitted during Death Protocol.

**Risk tier**: Layer 2. Triggers: PolicyCage validation, owner ownership check.
**Events**: `tool:start` -> `{ policyTemplate, customPolicy }` -> `tool:end` -> `{ appliedPolicy }`
**Pi hooks**: `tool_call` (verify owner authorization, validate policy against phase), `tool_result` (log policy change to Grimoire audit trail)

#### `wallet_fund`

**promptSnippet** (injected near system prompt, cached):
> Transfers gas tokens or operating capital from an owner source to the agent wallet. Requires owner authorization for amounts above threshold.

**promptGuidelines** (phase-conditional):
- **thriving**: Fund as needed for strategy execution. Standard amounts.
- **cautious**: Request only minimum required for current operations. Flag large funding requests for owner review.
- **declining**: Fund only gas reserves. No new operating capital.
- **terminal**: Do not request funding. The Death Protocol handles remaining assets.

**Risk tier**: Layer 2. Triggers: owner authorization, spending limit check.
**Events**: `tool:start` -> `{ token, amount, chain }` -> `tool:end` -> `{ txHash, funded }`
**Pi hooks**: `tool_call` (verify owner source, check funding amount against daily limit), `tool_result` (update wallet balance cache)

#### `wallet_migrate`

**promptSnippet** (injected near system prompt, cached):
> Moves all assets from one wallet to another. Elevated risk: Warden delay (requires optional Warden module). Use for wallet rotation or provider migration. Order: ERC-20s first, NFTs second, native ETH last.

**promptGuidelines** (phase-conditional):
- **thriving**: Use for planned wallet upgrades or provider migration. Always use dryRun first.
- **cautious**: Only migrate if current wallet provider has a security concern. Double-check destination address.
- **declining**: Migrate to owner cold wallet as part of wind-down. Skip NFTs unless they are LP positions.
- **terminal**: Death Protocol handles asset sweep. Do not call wallet_migrate directly.

**Risk tier**: Layer 3. Triggers: Warden delay (requires optional Warden module), PolicyCage full authorization, identity NFT custody check.
**Events**: `tool:start` -> `{ source, destination, assetCount }` -> `tool:progress` -> `{ phase: "erc20" | "nft" | "eth", completed, remaining }` -> `tool:end` -> `{ txHashes, totalMigrated }`
**Pi hooks**: `tool_call` (Warden announce if deployed, PolicyCage elevated authorization, verify destination is not a contract without receive/fallback), `tool_result` (update identity NFT ownership records, log migration to Grimoire)

#### `wallet_get_status`

**promptSnippet** (injected near system prompt, cached):
> Returns wallet balances, active policies, session keys, pending operations, and health checks. Read-only. Call on every heartbeat tick to maintain wallet awareness.

**promptGuidelines** (phase-conditional):
- **thriving**: Check periodically. Focus on balance sufficiency for planned operations.
- **cautious**: Check every tick. Flag balances below 2x daily burn rate.
- **declining**: Check every tick. Report remaining balances to heartbeat for runway calculation.
- **terminal**: Final status check before Death Protocol asset sweep.

**Risk tier**: Layer 1. Read-only, no state change.
**Events**: `tool:start` -> `tool:end` -> `{ address, provider, balanceCount }`
**Pi hooks**: `tool_call` (none), `tool_result` (cache balances for heartbeat consumption)
