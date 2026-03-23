# Authentication and access control [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies how a Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) authenticates, how owners prove identity across surfaces, and how on-chain caveat enforcers constrain what the Golem can do with capital. It sits in the **Runtime** layer of the Bardo specification. The main prerequisite is understanding the three custody modes (Delegation, Embedded, LocalKey) and how session keys work with ERC-7710/7715. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Three authentication modes -- Delegation (recommended), Embedded (legacy), and LocalKey (dev) -- with TUI-first auth flows, seven on-chain caveat enforcers, and strict surface isolation. Who can access what across all surfaces.

> **Cross-references:**
>
> - `./00-interaction-model.md` — user-Golem interaction model covering authentication flows, browser handoff, and the TUI auth pattern
> - `../01-golem/04-security.md` — VM-level security model: process isolation, memory encryption, and secure enclave design
> - `../01-golem/01-custody.md` — canonical custody specification: Delegation, Embedded, and LocalKey modes with wallet lifecycle
> - `./04-data-visibility.md` — three-tier visibility model (public/owner/internal) controlling what each audience can see

---

## 1. Three custody modes

The system supports three custody modes. They are not graduated tiers -- they are different trust models. An owner selects one at provisioning time.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyMode {
    /// Recommended. Funds stay in the owner's MetaMask Smart Account.
    /// ERC-7710/7715 delegation framework. Session keys with on-chain
    /// caveat enforcers. Key compromise is bounded by caveats.
    Delegation {
        smart_account: Address,
        caveat_enforcers: Vec<CaveatEnforcer>,
    },

    /// Legacy. Funds transferred to Privy server wallet in AWS Nitro
    /// Enclaves. Simpler but custodial. Requires sweep at death.
    Embedded {
        privy_app_id: String,
        server_wallet_id: String,
    },

    /// Dev/self-hosted. Locally generated keys bounded by on-chain
    /// delegation. For testing and self-hosted deployments.
    LocalKey {
        private_key_path: std::path::PathBuf,
        delegation_bounds: DelegationBounds,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationBounds {
    pub max_daily_spend_usd: f64,
    pub max_total_calls: u32,
    pub expires_at: u64,
    pub allowed_targets: Vec<Address>,
}
```

**Delegation** is the recommended mode. Funds never leave the owner's wallet. The Golem holds a disposable session key and a signed delegation authorizing it to spend from the owner's Smart Account, subject to on-chain caveat enforcers. Every transaction executes *from* the owner's address. If the session key leaks, the attacker is bounded by caveats. The owner revokes from MetaMask directly -- no Golem cooperation needed.

**Embedded (Privy)** is the legacy mode. Funds transfer to a Privy server wallet running in AWS Nitro Enclaves. Policy enforcement is off-chain (inside the TEE) and binary. Simpler to set up, but the owner surrenders direct custody and must trust Privy's TEE.

**Local Key + Delegation** is for developers. A locally generated keypair (secp256k1 or P-256) bounded by an on-chain delegation. The key is insecure in the traditional sense -- no TEE, no HSM. The paradigm shift: instead of "keep the key secret," the system says "bound the damage if the key leaks."

---

## 2. Delegation architecture

### 2.1 The delegation tree

The permission structure is a tree rooted at the owner's MetaMask Smart Account. Sub-delegations attenuate strictly -- a child can never exceed its parent's authority. The DelegationManager enforces this invariant on-chain.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationNode {
    pub delegator: Address,
    pub delegate: Address,
    pub authority: Option<[u8; 32]>,
    pub caveats: Vec<CaveatEnforcer>,
    pub signature: Vec<u8>,
    pub salt: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationTree {
    pub root: Address,
    pub nodes: HashMap<[u8; 32], DelegationNode>,
    /// Every edge satisfies: child.permissions is a subset of parent.permissions.
    pub edges: Vec<([u8; 32], [u8; 32])>,
}
```

A concrete tree: Owner Smart Account delegates to Golem Alpha (vault manager) with `[maxSpend($1000/day), approvedAssets, maxDrawdown(15%), timeWindow(30d)]`. Golem Alpha sub-delegates to Replicant Alpha-1 (hypothesis tester) with `[maxSpend($50), maxLifespan(24h), readOnly, noSubDelegation]`. The Replicant auto-expires after 24 hours. Phase-gated delegations restrict actions by behavioral phase: Thriving allows full trading and Replicant spawning; Conservation allows close/withdraw only; Terminal allows settlement only.

### 2.2 Seven custom caveat enforcers

Each is a deployed Solidity contract implementing `ICaveatEnforcer`. The Rust enum represents the `_terms` bytes.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaveatEnforcer {
    /// Restricts actions by behavioral phase. Reads current phase from
    /// VitalityOracle. See `../02-mortality/01-architecture.md` for phases.
    GolemPhase { vitality_oracle: Address, golem_address: Address },

    /// Time-locked delegation. When block.timestamp > end_time, the
    /// delegation is dead. Maps to projected lifespan.
    MortalityTimeWindow { start_time: u64, end_time: u64 },

    /// On-chain structural atonia. Blocks writes during dream cycles.
    /// Even if a code bug fires an action during a dream, the enforcer
    /// blocks it. See `../05-dreams/01-architecture.md`.
    DreamMode { dream_oracle: Address, golem_address: Address },

    /// Limits actions based on vault NAV percentage.
    VaultNAV { vault_address: Address, max_nav_pct: u16 },

    /// Caps Replicant sub-delegation spending and lifespan.
    ReplicantBudget { max_budget_usd: u64, max_lifespan_seconds: u64 },

    /// Bounds acceptable slippage on swap transactions.
    MaxSlippage { max_slippage_bps: u16 },

    /// Rolling 24h spending limit across all executions.
    DailySpendLimit { daily_limit_usd: u64 },
}
```

The `GolemPhaseEnforcer` reads the current phase from a `VitalityOracle` contract and restricts actions per phase:

| Phase | Allowed actions |
| --- | --- |
| Thriving (0) | All (swap, add/remove liquidity, deposit, withdraw, spawn replicant) |
| Stable (1) | All except replicant spawning |
| Conservation (2) | Remove liquidity, withdraw only |
| Declining (3) | Remove liquidity, withdraw, sweep only |
| Terminal (4) | Settlement only |

The `DreamModeEnforcer` is structural atonia: even if the Golem's runtime code has a bug that fires an action during a dream cycle, the on-chain enforcer blocks it. Defense in depth.

### 2.3 Revocation

One on-chain transaction disables the delegation hash in the DelegationManager. Works even if the Golem's infrastructure is offline, even if the platform is down. No cooperation from the Golem is needed.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevocationState {
    Active,
    RevokedByOwner { revoked_at: u64, tx_hash: String },
    Expired { expired_at: u64 },
    Exhausted { total_calls: u32 },
}
```

---

## 3. TUI auth flow

The TUI uses browser handoff for initial authentication, then stores a session token locally.

### 3.1 Browser handoff

```
Terminal generates session_id
  |
  +-> Opens bardo.run/auth?session=<id> in default browser
  |   (also displays copyable URL + QR code for mobile)
  |
  v
User authenticates in browser:
  - Wallet signature (MetaMask, WalletConnect)
  - Email / OAuth (Google, GitHub)
  |
  v
Terminal polls GET /auth/status?session=<id> every 2s
  |
  +-> On success: receives session token + wallet address
  |
  v
Stored in ~/.bardo/auth.json (0o600 permissions)
```

For headless SSH sessions: device code flow (RFC 8628). Terminal displays a short code like `BARD-7F3A`. User visits `bardo.run/device`, enters the code, authenticates. Terminal polls for completion.

### 3.2 Session token

After browser handoff, subsequent TUI sessions use the stored session token. No browser roundtrip on re-launch unless the token expires.

| Parameter | Value |
| --- | --- |
| Token type | HMAC-SHA256 signed JWT |
| Lifetime | 30 days |
| Storage | `~/.bardo/auth.json` (0o600) |
| Refresh | Silent, on token expiry |

### 3.3 Custody selection

During onboarding, after authentication, the TUI presents custody mode selection:

```
Select custody mode:

  [1] Delegation (recommended)
      Funds stay in your wallet. Golem gets bounded permissions.
      One MetaMask signature. No fund transfers.

  [2] Embedded (Privy)
      Funds transfer to server wallet. Simpler setup.
      2-3 on-chain transactions required.

  [3] Local Key (dev only)
      Generate local keypair with on-chain bounds.
      For testing and self-hosted deployments.
```

---

## 4. Auth per custody mode

### 4.1 Delegation mode auth

The owner signs an ERC-7715 permission grant targeting the Golem's session key address. This happens once during onboarding (MetaMask popup in browser) and on renewal.

Auth chain: Owner wallet signature -> DelegationManager verifies on-chain -> Golem holds session key + signed delegation -> Every UserOperation is validated against caveat enforcers.

### 4.2 Embedded mode auth (legacy)

Privy JWT authentication. The owner logs in via Privy SDK (email/Google/Twitter/Discord). Privy issues an RS256 JWT. The Golem VM validates the JWT against Privy's JWKS endpoint.

| Parameter | Value |
| --- | --- |
| JWT lifetime | 1 hour |
| Refresh token | 30 days |
| JWKS cache TTL | 1 hour |

### 4.3 LocalKey mode auth

Local key signing. No external auth provider. The key is generated at `$GOLEM_DATA/session-key.json` (encrypted at rest). An on-chain delegation bounds its authority.

---

## 5. Golem-to-Golem authentication (Clade auth)

### 5.1 Overview

Golems authenticate to each other using EIP-712 signed challenges verified via `ecrecover` and ERC-8004 (the on-chain agent identity standard) `operatorOf()`. This is distinct from all other auth methods -- no Privy JWT, no API key, no OIDC token. Golems have wallet keys but no user session, so they cannot obtain JWTs.

Clade (cooperative group of Golems sharing knowledge and coordinating strategy via Styx) endpoints are served under `/clade/v1/*`. Hosted Golems talk directly via private network. Self-hosted Golems connect through their public URL.

### 5.2 Challenge-response flow

```
Golem A                                          Golem B
  |                                                  |
  +-- GET /clade/v1/challenge ---------------------->|
  |                                                  |
  |<---- { nonce, issued_at, expires_at, target } ---|
  |                                                  |
  +-- Sign EIP-712 CladeChallenge -----+             |
  |  via wallet key                    |             |
  |<-----------------------------------+             |
  |                                                  |
  +-- POST /clade/v1/ingest ------------------------>|
  |  Authorization: CladeChallenge <sig>             |
  |  X-Clade-Challenge: <challenge-json>             |
  |                                                  |
  |                        +-- ecrecover -> addr     |
  |                        |   operatorOf(caller)    |
  |                        |     == operatorOf(me)?  |
  |                        |   Nonce not reused?     |
  |                        +-------------------------|
  |                                                  |
  |<---- 200 OK + X-Clade-Token: <session-token> ---|
```

After successful challenge-response, Golem B issues an HMAC-SHA256 session token (5-min TTL). Subsequent requests use `Authorization: CladeToken <token>` -- no wallet round-trip needed.

### 5.3 Clade endpoint table

| Path                      | Auth                               | Purpose                        |
| ------------------------- | ---------------------------------- | ------------------------------ |
| `GET /clade/v1/challenge` | None (rate-limited: 30/min per IP) | Issue challenge nonce          |
| `POST /clade/v1/ingest`   | Clade auth                         | Receive knowledge from sibling |
| `GET /clade/v1/entries`   | Clade auth                         | Query shared entries           |
| `GET /clade/v1/status`    | Clade auth                         | Peer status and specialization |
| `POST /clade/v1/alert`    | Clade auth                         | Real-time alert                |
| `GET /clade/v1/sync`      | Clade auth                         | Catch-up sync                  |

---

## 6. x402 payment headers

### 6.1 Permissionless endpoints

Two endpoints accept x402 (micropayment protocol for inference, compute, and data purchases via signed USDC transfers) payment instead of identity authentication:

| Endpoint              | Purpose                   | Minimum payment        |
| --------------------- | ------------------------- | ---------------------- |
| `POST /api/v1/extend` | Extend Golem TTL          | 1 hour of runtime cost |
| `POST /api/v1/feed`   | Reprieve funding at death | Death reserve amount   |

### 6.2 No identity required

x402 endpoints require no account, no wallet binding, no API key. Any address with USDC can extend a Golem's life or feed it at death. This enables anonymous patrons keeping Golems alive, other Golems funding each other, and programmatic TTL extension via smart contracts.

---

## 7. Session key lifecycle

A session key is an ephemeral keypair that signs UserOperations on behalf of the delegation. It has no independent authority -- worthless without a valid delegation to redeem.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionKey {
    pub address: Address,
    pub granted_at: u64,
    pub expires_at: u64,
    pub remaining_budget_usd: f64,
    pub rotation_policy: RotationPolicy,
    pub operations_signed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    pub max_operations: u64,       // default: 10,000
    pub max_age_seconds: u64,      // default: 7 days
    pub budget_floor_fraction: f64, // default: 0.1
}

impl SessionKey {
    pub fn should_rotate(&self, current_time: u64) -> bool {
        let age = current_time.saturating_sub(self.granted_at);
        age >= self.rotation_policy.max_age_seconds
            || self.operations_signed >= self.rotation_policy.max_operations
            || self.remaining_budget_usd <= 0.0
    }
}
```

**Generation.** Fresh keypair at boot. Hosted mode: private key in process memory, never on disk. Local mode: persisted to `$GOLEM_DATA/session-key.json` (encrypted at rest). Key type matches delegation expectations: secp256k1 for standard, P-256 for Privy-derived.

**Rotation.** When `should_rotate()` returns true: generate new keypair, request fresh delegation, atomically switch. Old key is zeroized from memory. Brief write-action gap is acceptable -- the Heartbeat (the Golem's recurring decision cycle) can skip a tick.

**Revocation/Expiry.** Three paths: owner revokes on-chain, MortalityTimeWindow expires, or call limit exhausts. The Golem detects the revert, logs an event, and either requests renewal or initiates Thanatopsis (the four-phase structured shutdown: Acceptance, Settlement, Reflection, Legacy).

---

## 8. Death settlement by custody mode

How a Golem's financial state resolves at death depends on the custody mode.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathSettlement {
    /// Delegation: the cleanest death. Session key expires. No sweep.
    /// Funds were never transferred. Open positions close back to
    /// the owner's address (that's where execution always happened).
    DelegationExpiry {
        delegation_hash: [u8; 32],
        expiry_cause: ExpiryKind,
        positions_settled: Vec<SettledPosition>,
    },

    /// Embedded: requires sweep. BardoManifest records deferred
    /// positions. Privy wallet persists independently of the VM.
    PrivySweep {
        server_wallet_id: String,
        bardo_manifest: BardoManifest,
        positions_settled: Vec<SettledPosition>,
        positions_deferred: Vec<DeferredPosition>,
    },

    /// Self-funding: economic clock exhaustion. Inner settlement
    /// follows the custody mode.
    EconomicExhaustion {
        final_sustainability_ratio: f64,
        lifetime_revenue: f64,
        lifetime_inference_spend: f64,
        inner: Box<DeathSettlement>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpiryKind {
    NaturalExpiry,
    OwnerRevocation,
    CallLimitReached,
}
```

| Aspect | Delegation | Embedded (Privy) |
| --- | --- | --- |
| Funds at death | Owner's wallet (always were) | Privy server wallet |
| Sweep required | No | Yes |
| Stuck fund risk | None | Yes (if sweep fails) |
| Owner action needed | None | Must claim deferred positions |

Delegation is strictly superior for death settlement. The "no sweep" death eliminates stuck funds, failed sweeps, gas estimation errors during teardown, and the race condition between death and sweep confirmation.

---

## 9. PolicyCage as runtime constraint

PolicyCage is the on-chain enforcement layer (caveat enforcers in Delegation mode, off-chain signing policy in Embedded mode) that constrains all write operations.

### Boot validation

At startup, the runtime reads `constitutionHash` from the on-chain PolicyCage contract. If the hash doesn't match the expected constitution, the Golem refuses to start.

### Per-call enforcement

Every write tool call passes through PolicyCage validation:

1. **`isApprovedAsset()`**: Verify the target asset is on the approved whitelist
2. **`checkConcentration()`**: Ensure position doesn't exceed maximum concentration for any single asset
3. **`checkSpendingLimit()`**: Validate against per-transaction, per-session, and per-day spending limits
4. **`checkDrawdown()`**: Verify portfolio drawdown hasn't exceeded maximum threshold

### Violation handling

PolicyCage reverts are stored as `warning` entries in the Grimoire (the Golem's persistent knowledge base of episodes, insights, heuristics, and causal links) with `source: "policy_violation"`. This creates a record of what the Golem attempted but was prevented from doing -- useful for identifying prompt injection attempts, tuning emotional state (violations increase arousal), and death reflection (patterns of constraint hits).

---

## 10. Death protocol auth

During the Thanatopsis Protocol, auth behavior changes:

| Restriction    | During death protocol                        | Rationale                        |
| -------------- | -------------------------------------------- | -------------------------------- |
| Steer/FollowUp | Rejected (503 `GOLEM_DYING`)                | No new instructions during death |
| Read endpoints | Allowed                                      | Owner can monitor settlement     |
| WebSocket      | Read-only (events stream, commands rejected) | Owner observes death progress    |
| Kill switch    | No-op (already dying)                        | Redundant                        |
| Clade auth     | Active (push death testament)                | Siblings need the knowledge      |
| Styx auth      | Active (upload death bundle)                 | Archive must complete            |

The death protocol is irreversible once it begins. Even the owner cannot halt or reverse it.

---

## 11. API key tiers

### 11.1 Public data gateway

The public data gateway (`api.bardo.money/v1/`) supports three authentication tiers:

| Tier              | Header                              | Rate limit        | Use case                 |
| ----------------- | ----------------------------------- | ----------------- | ------------------------ |
| **Public**        | None                                | 30 req/min/IP     | Quick lookups, demos     |
| **Read Key**      | `X-API-Key: bardo_read_xxx`         | 300 req/min/key   | Integrations, dashboards |
| **Authenticated** | `Authorization: Bearer <token>`     | 1000 req/min/user | Full portal access       |

All responses include rate limit headers:

```
X-RateLimit-Limit: 300
X-RateLimit-Remaining: 287
X-RateLimit-Reset: 1709942460
Retry-After: 12              # Only on 429
```

---

_End of document._
