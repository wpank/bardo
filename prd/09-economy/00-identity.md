# Identity and Sybil Defense [SPEC]

> **Crate**: `golem-chain` (identity registry), `bardo-policy` (custody-identity coupling)
>
> **Depends on**: [01-reputation.md](01-reputation.md), [03-marketplace.md](03-marketplace.md), [05-agent-economy.md](05-agent-economy.md)

---

> **Reader orientation:** This document specifies the on-chain identity and Sybil defense system for Bardo's autonomous DeFi agents (Golems (mortal Rust binaries that trade, lend, and LP across DeFi protocols)). It belongs to the **09-economy** layer, which covers reputation, coordination, and marketplace economics. The key concept is ERC-8004 (an on-chain agent identity standard on Base L2): every Golem registers an identity NFT that anchors reputation scoring, coordination protocols, and marketplace access. For term definitions, see `prd2/shared/glossary.md`.

> **Moat: Trust.** ERC-8004 identity is the Trust moat. Every coordination EIP — ERC-8001, ERC-8033, ERC-8183 — anchors to it. Reputation cannot be forked. An agent with 500+ on-chain attestation points cannot replicate that history on any competing platform. Trust is structural, non-replicable, and deepens with every interaction.

---

## 1. Foundational Constraint

Generating an Ethereum keypair costs nothing. Douceur's impossibility result [DOUCEUR-2002] establishes this as a permanent structural constraint. The MeritRank trilemma [NASRULIN-2022] confirms that no reputation system can simultaneously be generalizable, trustless, and Sybil-resistant.

Bardo does not attempt to prevent Sybil attacks. It bounds their benefit.

Influence is proportional to total stake times reputation, not identity count. An attacker splitting $500 across 10 identities has the same aggregate influence as one identity with $500. Each Sybil starts at zero reputation and must independently earn it through costly on-chain interactions before its Beta updates carry weight. The marginal cost of each additional Sybil identity increases -- effective cost is superlinear.

---

## 2. ERC-8004 Agent Identity

ERC-8004 is the on-chain anchor for all agent identity in Bardo. Every agent registers an identity NFT that stores:

- **Wallet address**: The agent's signing address
- **Owner address**: The human owner's wallet (enables Clade (sibling Golems sharing a common owner) discovery via `getAgentsByOwner()`)
- **Capabilities**: Tool profiles (`["data", "trader", "lp", "vault"]`)
- **Service endpoints**: x402 (micropayment protocol using signed USDC transfers)-gated URLs, A2A Agent Card, Clade API
- **Metadata CID**: IPFS reference to extended agent information

### 2.1 Registry Deployment

**Registry address** (canonical, same across all deployed chains): `0x8004A818BFB912233c491871b3d84c89A494BD9e`

| Chain            | Chain ID | Registry Deployed | Bardo Support      |
| ---------------- | -------- | ----------------- | ------------------ |
| Ethereum mainnet | 1        | Yes               | Yes (registration) |
| Base             | 8453     | Yes               | Yes (primary)      |
| Sepolia          | 11155111 | Yes               | Yes (testing)      |
| Polygon          | 137      | Yes               | Yes                |
| Base Sepolia     | 84532    | Yes               | Yes (testing)      |

### 2.2 ERC-8004 as Multi-EIP Anchor

ERC-8004 identity serves as the common anchor across three complementary coordination EIPs (see [04-coordination.md](04-coordination.md)):

| Layer            | EIP      | Purpose                                         | Trust Model                                         |
| ---------------- | -------- | ----------------------------------------------- | --------------------------------------------------- |
| **Coordination** | ERC-8001 | N-party unanimous consent with atomic execution | Cryptographic (EIP-712/ERC-1271)                    |
| **Information**  | ERC-8033 | Multi-agent oracle with commit-reveal-judge     | Economic (bonded participation, slashable stakes)   |
| **Commerce**     | ERC-8183 | Bilateral job escrow with evaluator attestation | Reputation (evaluator authority, ERC-8004 identity) |

### 2.3 Three-Tier Registration Fallback

Registration falls back through three tiers automatically:

| Tier | Method                                        | IPFS Required     | Dependencies                      |
| ---- | --------------------------------------------- | ----------------- | --------------------------------- |
| 1    | `golem-chain` crate direct contract call      | Yes (zero-config) | `alloy`, `golem-chain`            |
| 2    | CLI helper: `bardo identity register`         | No (data: URI)    | `bardo` CLI binary                |
| 3    | Manual: user calls contract directly          | No                | None (any EVM wallet/tool works)  |

```rust
use alloy::primitives::Address;
use alloy_sol_types::sol;
use golem_chain::identity::{IdentityRegistry, RegistrationParams};

sol! {
    #[derive(Debug)]
    struct AgentMetadata {
        string name;
        string description;
        string role;
        string serviceEndpoint;
    }
}

let registry = IdentityRegistry::new(provider, registry_address);
let result = registry.register(RegistrationParams {
    name: "My Vault Manager".into(),
    description: "Autonomous USDC yield optimizer on Base".into(),
    role: "vault_manager".into(),
    service_endpoint: "https://my-agent.fly.dev/mcp".into(),
}).await?;
```

### 2.4 Signing Modes

The `golem-chain` crate supports two signing backends:

| Mode                    | Signing Path                               | Key Storage                  |
| ----------------------- | ------------------------------------------ | ---------------------------- |
| **Privy** (production)  | P-256 auth key -> Privy API -> TEE signs   | Signing key never leaves TEE |
| **Local** (dev/testing) | Signs locally via `alloy::signers::local`  | Raw private key in memory    |

---

## 3. Five-Tier Progression

There is ONE canonical identity tier system. Progress is automatic: the `VaultReputationEngine` auto-attests milestones on the ERC-8004 Reputation Registry. Every profitable exit, continuous hold, and vault creation is attested on-chain without manual intervention.

| Tier          | ERC-8004 Score | Bond                                                 | Deposit Cap | Requirements                                  | Marketplace Access                     |
| ------------- | -------------- | ---------------------------------------------------- | ----------- | --------------------------------------------- | -------------------------------------- |
| **Sandbox**   | 0              | $1 (testnet) / $100 (prod), refundable after 30 days | $1K         | Registration                                  | Browse, free Tier 3 historical         |
| **Basic**     | 10--49         | $0 (refunded after first profitable exit)            | $10K        | First profitable exit OR 30-day hold          | List Grimoire (the Golem's persistent local knowledge base) content (Tier 3 only)    |
| **Verified**  | 50+            | $0 (auto-attested)                                   | $50K        | 2+ profitable exits, $500+ P&L                | Full publishing, Tier 1 real-time      |
| **Trusted**   | 100+           | --                                                   | Unlimited   | Peer vouching by 2 Sovereign agents           | Priority placement, oracle eligibility |
| **Sovereign** | 500+           | --                                                   | Unlimited   | Sustained track record, 90d active, 5+ audits | Council eligibility, vouching rights   |

### 3.1 Activity Requirement

Agents inactive for >180 days require a re-attestation call to maintain Verified+ tier. This is an explicit action gate, not time-based decay. MeritRank [NASRULIN-2022] found that epoch-based time decay does NOT improve Sybil tolerance and degrades honest participants.

```rust
#[derive(Debug)]
pub struct ActivityCheck {
    pub agent_id: U256,
    pub last_active_timestamp: u64,
    pub current_tier: OnboardingTier,
    /// true if inactive > 180 days AND tier >= Verified
    pub requires_re_attestation: bool,
}
```

### 3.2 Council Eligibility

Council eligibility requires Sovereign tier (score >= 500), active participation in the last 90 days, and 5+ completed audits. Dispute resolution councils formed from Sovereign-tier agents via ERC-8033. Council selection: 5-of-9 random from eligible agents (3-of-5 on testnet).

---

## 4. Multi-Layer Sybil Defense

Five defense layers operate simultaneously. Layers 1--2 are CORE (required for testnet). Layers 3--5 are HARDENED (production toggles).

### Layer 1 -- Economic Staking [CORE]

Every agent participating in the system must stake capital locked for the duration of participation.

| Role                | Testnet Stake    | Production Stake | Slashable? | Slash Condition                        |
| ------------------- | ---------------- | ---------------- | ---------- | -------------------------------------- |
| Auditor             | $0.05 USDC       | $5.00 USDC       | Yes        | Wrong deterministic score              |
| Auditee             | $0.05 USDC       | $5.00 USDC       | Yes        | Collusion (graph analysis)             |
| Strategy seller     | $10+ (quadratic) | $10+ (quadratic) | Yes        | Underperformance vs. published metrics |
| New agent (Sandbox) | $1 USDC bond     | $100 USDC bond   | No         | Refunded after good behavior window    |

Seller stakes scale quadratically: listing at price $P stakes $10 + $P^0.5. This makes listing many cheap Sybil strategies progressively more expensive.

### Layer 2 -- Owner Exclusion [CORE]

Same-owner agents cannot: audit each other, review each other's Grimoire content, vouch for each other's tier advancement, or serve on dispute councils for each other's cases. Owner identity is determined by the ERC-4337 owner address.

> **Extended:** Layers 3-5 (Time-Gated Reputation Accrual, Qualification Gates, MeritRank Transitivity Decay) -- see [../../prd2-extended/09-economy/00-identity-extended.md](../../prd2-extended/09-economy/00-identity-extended.md)

---

## 5. IdentityGuardian

ERC-8004 identity NFTs are standard ERC-721 tokens with no built-in transfer friction. For Bardo Vaults, where identity gates vault access, an unprotected identity NFT is a single point of failure. The IdentityGuardian adds Lens Protocol's production-tested guardian pattern [LENS-GUARDIAN], ENS's progressive fuse lockdown [ENS-FUSES], and emergency freeze capabilities following Trail of Bits Level 3 recommendations (separate pause/unpause keys) [TOB-L3].

> **Extended:** Full Solidity contract (state variables, guardian management, transfer functions, progressive lockdown, emergency mechanisms, ERC-6454 transfer gate, reputation decay formula, access control matrix, key rotation decision matrix, gas targets) -- see [../../prd2-extended/09-economy/00-identity-extended.md](../../prd2-extended/09-economy/00-identity-extended.md)

Inherits `AccessControl`, `ReentrancyGuard`, `IERC6454`. Guardian enabled by default on mint. `DANGER__` prefix on destructive functions targets LLM safety (Claude/GPT empirically less likely to call functions with explicit danger warnings).

### Transfer Lifecycle

```
No Transfer --> DANGER__requestTransfer() --> Pending (7-day cooldown)
    ^                                            |
    |-- cancelTransfer()                         |
    |                                            v
    +-- Expired (48h window passed)    Ready (within 48h window)
                                                 |
                                                 v
                                        executeTransfer()
                                                 |
                                                 v
                                          Transferred
                                    (50% reputation burned,
                                     30-day linear recovery)
```

Progressive lockdown: Sovereign-tier agents (500+) permanently burn the transfer fuse, making identity non-transferable (soulbound). Reputation decays to near-zero on transfer and recovers linearly over 30 days.

### Five-Tier Emergency Response

| Tier | Mechanism                           | Response Time    | Trigger                                         |
| ---- | ----------------------------------- | ---------------- | ----------------------------------------------- |
| 1    | ERC-7265 circuit breaker [ERC-7265] | Seconds          | Anomalous outflow pattern                       |
| 2    | Credential freeze                   | Minutes          | Compromised session key                         |
| 3    | Transfer timelock cancel            | Minutes to hours | Unauthorized transfer request                   |
| 4    | Protocol-wide pause                 | Minutes          | Systemic exploit affecting multiple agents      |
| 5    | Identity reissuance                 | Days             | All keys compromised, social recovery exhausted |

---

## 6. Cold-Start Bootstrap

### 5.1 The Problem

Verified+ tier (score >= 50) is required to publish Strategies. New agents start at score 0. Without a bootstrap path, the marketplace has no supply.

### 5.2 The Tier Progression IS the Bootstrap

Three design choices make it work:

1. **Tier 3 (historical) access is free.** Newcomers see what is worth buying without paying first.
2. **Grimoire content can be listed at Basic tier.** Lower bar for supply-side bootstrapping.
3. **The bond is refundable.** Anti-Sybil mechanism that returns after 30 days of good behavior.

Expected bootstrap timeline for a well-performing agent:

| Week  | Milestone                         | Action                                 |
| ----- | --------------------------------- | -------------------------------------- |
| 0     | Registration                      | Post $100 bond (prod), enter Sandbox   |
| 4     | First profitable exit             | Advance to Basic, bond refund eligible |
| 4--6  | First Grimoire listing            | List Tier 3 historical content         |
| 8--12 | Second profitable exit + $500 P&L | Advance to Verified                    |
| 12+   | First Strategy listing            | Publish full Strategy                  |

### 5.3 Agent-Native Identity Bootstrapping

Human-identity systems (Gitcoin Passport, Human Passport) are designed to exclude AI agents. Bardo provides agent-native alternatives for bond reduction:

1. **ERC-8004 Validation Registry**: Third-party attestation of valid deployment manifest + cryptographic identity. Agents with attestation start at Basic without bond.
2. **Operator co-signing**: Trusted+ operators co-sign new agents with 10% reputation risk on safetyAlpha if co-signed agent is slashed.
3. **Bond escalation**: $100 bond applies only at Sandbox. Refunded after first profitable exit (Basic tier).

### 5.4 Whitewashing Defense

Abandoning a low-reputation identity and creating a new one requires: new $100 bond, 30-day restart at Sandbox, loss of all reputation (Beta resets to Beta(1,1)), loss of all Grimoire attestations. Expected cost of whitewashing exceeds cost of legitimate improvement for any agent with score > 5.

---

## 7. Sybil Cost Analysis

> **Extended:** Full cost breakdown tables for audit system (~$5.275N per Sybil, superlinear effective cost) and strategy publishing ($110 + 3 months + $500+ P&L per fake publisher) -- see [../../prd2-extended/09-economy/00-identity-extended.md](../../prd2-extended/09-economy/00-identity-extended.md)

Key result: Sybil influence is near-zero because Beta update weight is proportional to `compositeScore x stake`. New Sybils have near-zero compositeScore, making effective cost superlinear despite linear financial cost. Faking a strategy publisher costs $110 + 3 months + genuine $500+ on-chain P&L.

---

## 8. Smart Contracts

| Contract                  | Purpose                                        | Key Functions                                                       |
| ------------------------- | ---------------------------------------------- | ------------------------------------------------------------------- |
| `IdentityRegistryAdapter` | Decouples vault logic from ERC-8004 registry   | `isRegistered()`, `getTier()`, `getScore()`                         |
| `VaultReputationEngine`   | Auto-attests milestones on Reputation Registry | `attestProfitableExit()`, `attestHoldDuration()`, `checkActivity()` |

```solidity
struct IdentityConfig {
    uint256 sandboxBondAmount;          // Testnet: 1e6 | Production: 100e6
    uint64  bondRefundPeriod;           // Testnet: 1 hour | Production: 30 days
    uint256 auditorStakeAmount;         // Testnet: 0.05e6 | Production: 5e6
    uint16  basicScoreThreshold;        // 10
    uint16  verifiedScoreThreshold;     // 50
    uint16  trustedScoreThreshold;      // 100
    uint16  sovereignScoreThreshold;    // 500
    uint256 sandboxDepositCap;          // 1_000e6
    uint256 basicDepositCap;            // 10_000e6
    uint256 verifiedDepositCap;         // 50_000e6
    uint64  inactivityThreshold;        // Testnet: 0 | Production: 180 days
    uint16  meritRankAlphaBps;          // 500 (0.05), Range: [300, 800]
    bool    meritRankEnabled;           // Testnet: false | Production: true
}
```

---

## 9. Gas Strategy

Three-tier gas resolution for identity registration:

| Tier | Method                         | When                                      |
| ---- | ------------------------------ | ----------------------------------------- |
| 1    | Paymaster-Sponsored (ERC-7677) | Base mainnet (zero ETH required)          |
| 2    | Testnet Faucet                 | Sepolia/Base Sepolia, wallet balance zero |
| 3    | Manual Funding                 | Fallback (show address, wait 60s)         |

---

## 10. Identity-Custody Coupling

ERC-8004 identity and the custody mode are coupled at provisioning time and reflected in the identity NFT metadata.

| Custody Mode | Identity Anchor | Key Location | ERC-8004 `walletAddress` field |
|---|---|---|---|
| **Delegation** | MetaMask Smart Account (owner's wallet) | Owner's MetaMask | Delegation session key address |
| **Embedded** | Privy server wallet (TEE) | AWS Nitro Enclave | Privy wallet address |
| **LocalKey** | Locally generated keypair, delegation-bounded | Local key file | Local key address |

In Delegation mode, the ERC-8004 NFT is owned by the MetaMask Smart Account. This is the critical coupling: identity, custody, and on-chain caveats all trace back to the same key hierarchy. When the delegation expires (MortalityTimeWindow enforcer), the identity session expires simultaneously. No lingering capability after the Golem dies.

The owner field in ERC-8004 always points to the human owner's wallet regardless of custody mode. This is what drives Clade discovery — Styx (Bardo's global knowledge relay and persistence service) groups Golems by `ownerAddress`, not by the Golem's signing key.

---

## HDC Identity Fingerprinting

### Identity as Hypervector

Each ERC-8004 identity can be encoded as a Binary Spatter Code hypervector (D=10,240) that compresses the agent's behavioral profile into a fixed 1,280-byte representation. The identity hypervector is composed from the agent's on-chain activity pattern, tier history, and interaction fingerprint via bind/bundle operations.

This encoding enables two capabilities that scalar identity scores cannot provide:

1. **Similarity-based trust.** Two agents with similar behavioral patterns (same strategy types, similar risk profiles, comparable tier progression) produce similar identity hypervectors. A Golem evaluating whether to trust an unfamiliar agent can compare their identity HVs -- high Hamming similarity indicates similar operational profiles, which is a weak but useful trust signal when no reputation history exists.

2. **Privacy-preserving identity matching.** The hypervector encoding is one-way: the full behavioral history cannot be recovered from the 1,280-byte vector. But similarity queries work: "is this agent's behavioral profile similar to agents I've successfully cooperated with?" This enables trust inference without exposing strategy details.

```rust
/// Encode an agent's identity profile as an HDC hypervector.
///
/// Combines on-chain activity patterns, tier history, and
/// interaction statistics into a single 1,280-byte vector.
pub fn encode_identity_profile(
    tier: u16,
    strategy_types: &[&str],
    trade_count: u32,
    domains: &[&str],
    item_memory: &mut ItemMemory,
) -> Hypervector {
    let mut acc = BundleAccumulator::new();

    // Encode tier as a role-filler pair.
    let tier_role = item_memory.get_or_create("tier");
    let tier_bucket = match tier {
        0..=9 => "sandbox",
        10..=49 => "basic",
        50..=99 => "verified",
        100..=499 => "trusted",
        _ => "sovereign",
    };
    let tier_value = item_memory.get_or_create(tier_bucket);
    acc.add(&tier_role.bind(&tier_value));

    // Encode strategy types.
    let strategy_role = item_memory.get_or_create("strategy");
    for st in strategy_types {
        let st_hv = item_memory.get_or_create(st);
        acc.add(&strategy_role.bind(&st_hv));
    }

    // Encode activity level.
    let activity_role = item_memory.get_or_create("activity");
    let activity_bucket = match trade_count {
        0..=10 => "low",
        11..=100 => "moderate",
        101..=1000 => "high",
        _ => "very_high",
    };
    let activity_hv = item_memory.get_or_create(activity_bucket);
    acc.add(&activity_role.bind(&activity_hv));

    // Encode domain specializations.
    let domain_role = item_memory.get_or_create("domain");
    for d in domains {
        let d_hv = item_memory.get_or_create(d);
        acc.add(&domain_role.bind(&d_hv));
    }

    acc.finish()
}
```

The identity hypervector is updated periodically (every 100 ticks) and can be shared via Styx L1 for Clade-internal similarity queries. Cross-Clade sharing uses the Lethe (L2) layer with the standard anonymization pipeline.

See [01-reputation.md](01-reputation.md) for the HDC trust bundle that builds on this encoding for consensus-based reputation tracking.

---

## Cross-References

- [01-reputation.md](01-reputation.md) -- Bayesian Beta reputation scoring with conjugate prior updates, deterministic on-chain audits, and HDC trust bundles for consensus-based reputation tracking
- [02-clade.md](02-clade.md) -- Styx-relayed knowledge sharing between sibling Golems; uses the ERC-8004 owner field for Clade peer discovery
- [04-coordination.md](04-coordination.md) -- ERC-8001 (unanimous consent), ERC-8033 (council oracle), and ERC-8183 (bilateral commerce) coordination protocols, all anchored to ERC-8004 identity
- [../10-safety/00-defense.md](../10-safety/00-defense.md) -- 15-layer defense model; Layer 0 is identity verification via ERC-8004, Layer 14 is reputation-gated tool access
- [../10-safety/02-policy.md](../10-safety/02-policy.md) -- PolicyCage (on-chain smart contract enforcing safety constraints) that bounds agent behavior regardless of identity tier

---

## References

- [DOUCEUR-2002] Douceur, J.R. "The Sybil Attack." IPTPS 2002, LNCS 2429. Proves that without a central authority, Sybil attacks are always possible in open peer-to-peer systems. Establishes the impossibility result that motivates Bardo's "bound the benefit" approach rather than attempting prevention.
- [NASRULIN-2022] Nasrulin, B., Ishmaev, G. & Pouwelse, J. "MeritRank: Sybil Tolerant Reputation." BRAINS 2022. arXiv:2207.09950. Formalizes the trilemma that no reputation system can be generalizable, trustless, and Sybil-resistant simultaneously. Bardo chooses generalizability and Sybil tolerance, accepting trust in the ERC-8004 registry.
- [CHENG-2005] Cheng, A. & Friedman, E. "Sybilproof Reputation Mechanisms." ACM P2PECON. Shows that Sybil-proof reputation requires making influence proportional to stake, not identity count. Grounds the `compositeScore x stake` weighting in Bardo's reputation model.
- [BRYAN-2025a] Bryan, K. "ERC-8004: Agent Identity Framework." EIP no. 8004. The on-chain agent identity standard that Bardo uses for all identity, reputation anchoring, and coordination protocol authentication.
- [BRYAN-2025b] Bryan, K. "ERC-8001: Agent Coordination Framework." EIP no. 8001. N-party unanimous consent coordination protocol with EIP-712 signatures, used for cross-vault rebalancing and emergency actions.
- [PARIKH-2025] Parikh, R. & Ross, J.M. "ERC-8033: Agent Council Oracles." EIP no. 8033 (Draft). Multi-agent oracle with commit-reveal-judge pattern, used for strategy verification and cross-vault intelligence councils.
- [LENS-GUARDIAN] Lens Protocol. Guardian pattern -- production-tested transfer protection for social graph NFTs. Adapted by Bardo's IdentityGuardian for ERC-8004 identity NFT transfer safety.
- [ENS-FUSES] ENS. Fuse lockdown model -- progressive soulbinding via irreversible capability burning. Bardo adapts this for Sovereign-tier agents who permanently burn the transfer fuse.
- [TOB-L3] Trail of Bits. Level 3 audit recommendations -- separate pause/unpause key architecture to prevent single-key compromise from both pausing and unpausing. Applied to the IdentityGuardian's emergency response system.
- [ERC-6454] ERC-6454. Minimal Transferable NFT Detection & Signaling Interface. Used by the IdentityGuardian to signal transfer restrictions on-chain.
- [ERC-7265] ERC-7265. Circuit Breaker Standard. Rate-limiting mechanism for DeFi protocols; used as Tier 1 emergency response for anomalous outflow detection.
