# Reputation: Bayesian Beta, Deterministic Audits, and RBTS Reviews [SPEC]

> **Crate**: `golem-reputation`
>
> **Depends on**: [00-identity.md](00-identity.md) (ERC-8004 tiers), [03-marketplace.md](03-marketplace.md) (buyer reviews)

> **Reader orientation:** This document specifies the reputation system for Bardo's autonomous DeFi agents (Golems (mortal Rust binaries that trade, lend, and LP across DeFi)). It belongs to the **09-economy** layer. The key concept is a dual-signal trust model: deterministic on-chain performance audits (objective, verifiable) combined with incentive-compatible buyer reviews via RBTS (Raters Believed Truthful by Staking), all anchored to ERC-8004 (on-chain agent identity). For term definitions, see `prd2/shared/glossary.md`.

An agent that wants to sell its Grimoire (the Golem's persistent local knowledge base) insights or Strategies faces a credibility problem: how does a buyer know the knowledge is worth anything? Self-reported P&L is unverifiable. ERC-8004 reputation tier reflects on-chain activity volume but says nothing about strategy quality.

The reputation system provides two independent trust signals: performance audits (objective, deterministic) and buyer reviews (subjective but incentive-compatible via RBTS).

> **Scope note**: For v1 implementation boundaries (what ships vs. what defers to v2), see [v1 Scope Decisions](#v1-scope-decisions) at the bottom of this file.

---

## 1. Design Goals

1. **Independence**: Ratings come from agents with no Clade affiliation to the rated agent
2. **No product giveaway**: Auditors verify from public on-chain data only; the Grimoire is never exposed
3. **Deterministic where possible**: Scoring uses formulas, not LLMs. Morningstar, GIPS, and Lipper all established this pattern [ELING-2007]
4. **Truthful**: Buyer reviews use RBTS, making honest reporting strictly incentive-compatible for n >= 5
5. **Two-signal pricing**: Price reflects both verified performance (deterministic) and buyer satisfaction (RBTS)
6. **Self-reinforcing**: Organic demand without long-term subsidy reliance

---

## 2. Performance Audits (Deterministic Scoring)

### 2.1 What an Audit IS and IS NOT

An audit is an independent agent deterministically computing and verifying another agent's on-chain DeFi performance using publicly available blockchain data and a standardized scoring formula.

An audit is NOT an LLM subjectively judging performance metrics. LLM-based numeric scoring is provably inconsistent: [LAU-2026] found systematic inter-model divergence on identical inputs; [HALDAR-2025] found intra-model inconsistency "almost arbitrary in the worst case."

### 2.2 The Audit Flow

ERC-8001 coordination intent between auditee and auditor:

```
coordinationType: keccak256("BARDO_PEER_AUDIT_V2")
participants: [AgentA, AgentB]
coordinationValue: auditFeeUsd
payload: {
    auditee: AgentA.address,
    auditor: AgentB.address,
    auditWindow: { fromBlock, toBlock },
    strategyTypes: ["lp_management", "yield_farming", "lending", "trading"],
    commitDeadline: block.timestamp + config.commitPhaseDuration,
    revealDeadline: commitDeadline + config.revealPhaseDuration,
}
```

### 2.3 Deterministic Performance Score Formula

The formula follows Morningstar Risk-Adjusted Return (MRAR) methodology [SHARPE-1998] and GIPS standards. Five dimensions, weighted:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetrics {
    pub realized_pnl: f64,           // Net P&L in USDC
    pub benchmark_pnl: f64,          // ETH hold return over same period
    pub alpha: f64,                  // realized_pnl - benchmark_pnl
    pub sharpe_ratio: f64,           // (return - risk_free_rate) / sigma(returns)
    pub sortino_ratio: f64,          // (return - risk_free_rate) / sigma(downside)
    pub max_drawdown: f64,           // Worst peak-to-trough (%)
    pub capital_efficiency: f64,     // Avg deployed / total available (%)
    pub avg_slippage: f64,           // Mean slippage per trade (bps)
    pub gas_per_trade: f64,          // Mean gas cost per trade (USDC)
    pub trade_count: u32,
    pub win_rate: f64,
    pub recovery_time: u64,          // Avg blocks to recover from drawdown
}

#[derive(Debug, Clone)]
pub struct AuditScore {
    pub overall: u32,
    pub dimensions: [f64; 5],
}

pub fn compute_performance_score(metrics: &AuditMetrics) -> AuditScore {
    // Dimension 1: Realized P&L (30%)
    let pnl_score = /* ... normalized alpha score ... */;
    // Dimension 2: Risk-Adjusted Return (25%) -- Morningstar MRAR pattern
    let risk_score = /* ... Sharpe + drawdown ... */;
    // Dimension 3: Capital Efficiency (20%)
    let efficiency_score = /* ... utilization rate ... */;
    // Dimension 4: Execution Quality (15%)
    let exec_score = /* ... slippage + gas ... */;
    // Dimension 5: Consistency (10%)
    let consistency_score = /* ... win rate + recovery ... */;

    let overall = (pnl_score * 0.30 + risk_score * 0.25
        + efficiency_score * 0.20 + exec_score * 0.15
        + consistency_score * 0.10)
        .round() as u32;

    AuditScore {
        overall,
        dimensions: [pnl_score, risk_score, efficiency_score, exec_score, consistency_score],
    }
}
```

Two honest auditors computing from the same on-chain data MUST produce the same result (within +/-2). Disputes become trivially resolvable: re-run the formula and check.

### 2.4 Commit-Reveal Protocol

1. **Commit phase**: Auditor commits `keccak256(auditIntentHash, score, dimensions, metricsHash, nonce)` on-chain
2. **Reveal phase**: Auditor reveals `(score, dimensions, metricsHash, nonce)`. Contract verifies against commitment hash.

Phase durations: 5 minutes on testnet, 24 hours on production.

### 2.5 Bond Terms

| Role         | Bond Amount                                       | When Returned                                 | When Slashed                           |
| ------------ | ------------------------------------------------- | --------------------------------------------- | -------------------------------------- |
| **Auditor**  | `auditorStakeAmount` ($0.05 testnet / $5.00 prod) | Successful reveal within +/-2 of verification | Score deviation >2 from council median |
| **Auditee**  | `auditFee` ($0.01 min testnet / $0.25 min prod)   | Successful auditor reveal                     | 50% returned if auditor fails          |
| **Disputer** | 1.5x original audit bond                          | Dispute upheld                                | Dispute rejected                       |

---

## 3. Buyer Reviews with RBTS

### 3.1 Robust Bayesian Truth Serum

Performance audits measure on-chain outcomes, not Grimoire quality. Only buyers who consumed the knowledge can evaluate it. Naive buyer reviews suffer from strategic misreporting.

RBTS [WITKOWSKI-2012] provides strict incentive compatibility without knowing the common prior. Each buyer review includes both a rating AND a prediction of what other buyers will rate. RBTS rewards buyers whose ratings are "surprisingly common" -- more frequent than predicted.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerReview {
    pub purchase_id: String,
    pub content_score: u32, // 0-100
    pub dimensions: ReviewDimensions,
    pub would_recommend: bool,
    pub poisoning_incidents: u32,
    pub predicted_avg_score: f64,
    pub predicted_recommend_rate: f64, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDimensions {
    pub novelty: f64,
    pub actionability: f64,
    pub accuracy: f64,
    pub depth: f64,
    pub safety: f64,
}
```

### 3.2 RBTS Scoring Window

| Review Count (per window) | RBTS Scoring           | RBTS Rewards             | Primary Defense              |
| ------------------------- | ---------------------- | ------------------------ | ---------------------------- |
| n < 3                     | Not run                | Not computed             | Economic staking only        |
| n = 3--4                  | Runs (data collection) | Not computed             | Economic staking primary     |
| n >= 5                    | Runs                   | Computed and distributed | RBTS incentive compatibility |

**Phased rollout**: Phase 1 (0--4 reviews): simple binary rating, no rewards. Phase 2 (5+ reviews): full RBTS activation. Retroactive application when crossing the 5-review threshold.

---

## 4. Reputation Aggregation: Bayesian Beta

### 4.1 Why Not EigenTrust

EigenTrust [KAMVAR-2003] was designed for 1,000+ peer networks. At 5--20 agents, multiple failures: sparse trust matrix, dangling nodes, rank sinks, and no symmetric sybilproof reputation function [CHENG-2005].

### 4.2 Bayesian Beta Reputation System

The Beta distribution `Beta(alpha, beta)` is the conjugate prior for Bernoulli observations [JOSANG-2002]. New agents with `Beta(1, 1)` have maximum uncertainty. Each audit narrows the distribution.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReputation {
    pub perf_alpha: f64,     // Initialized to 1.0
    pub perf_beta: f64,      // Initialized to 1.0
    pub content_alpha: f64,
    pub content_beta: f64,
    pub safety_alpha: f64,
    pub safety_beta: f64,
    pub perf_score: f64,     // E[Beta(alpha,beta)]
    pub content_score: f64,
    pub safety_score: f64,
    pub confidence: f64,
    pub composite_score: f64,
}
```

### 4.3 Update Rule

```rust
pub fn update_reputation(rep: &mut AgentReputation, score: f64, weight: f64) {
    let normalized = score / 100.0;
    rep.perf_alpha += normalized * weight;
    rep.perf_beta += (1.0 - normalized) * weight;
    rep.perf_score = rep.perf_alpha / (rep.perf_alpha + rep.perf_beta);
    rep.confidence = 1.0 - 2.0 / (rep.perf_alpha + rep.perf_beta + 2.0);
}
```

An auditor's weight equals `compositeScore * stakeWeight`. Sybil agents contribute near-zero weight. Content update weight is 0.5x default to reflect lower verifiability of subjective assessments.

### 4.4 Temporal Decay with Beta Floor

In production, alpha and beta are multiplied by decay factor (0.95) at each epoch (7 days). Floor at 1.0 prevents bimodal U-shaped distribution.

| Epochs Inactive | perfAlpha | perfBeta | E[trust] | Confidence |
| --------------- | --------- | -------- | -------- | ---------- |
| 0               | 10.0      | 2.0      | 0.83     | 0.86       |
| 30              | 2.15      | 1.00     | 0.68     | 0.61       |
| 60              | 1.00      | 1.00     | 0.50     | 0.00       |

### 4.5 Composite Score

```rust
pub fn compute_composite(rep: &AgentReputation) -> f64 {
    let perf_signal = rep.perf_score * 0.8 + rep.safety_score * 0.2;
    let content_signal = rep.content_score;
    (perf_signal * 0.7 + content_signal * 0.3) * rep.confidence
}
```

70/30 split: deterministic audit performance is objectively verifiable and cannot be gamed. Buyer reviews, while RBTS-incentivized, remain subjective.

> **Extended:** Section 4.6 Collusion Detection [HARDENED] -- see [../../prd2-extended/09-economy/01-reputation-extended.md](../../prd2-extended/09-economy/01-reputation-extended.md)

---

## 5. Verification Badges

Badges are visual indicators within the tier system, reflecting audit and review status:

| Badge                   | Visual | Requirement                                                                |
| ----------------------- | ------ | -------------------------------------------------------------------------- |
| **Unverified**          | None   | 0 audits, 0 reviews                                                        |
| **Performance Audited** | Blue   | >= 1 performance audit completed                                           |
| **Verified**            | Green  | >= 2 audits from >= 2 unique auditors, perf score >= 60, confidence >= 0.3 |
| **Buyer Endorsed**      | Gold   | Verified + >= 2 buyer reviews, content >= 70, safety >= 80                 |

The ERC-8004 tier always governs access; the badge governs buyer confidence. A Sovereign-tier agent with no audits displays "Unverified" badge but has full access rights.

---

## 6. Dispute Resolution

### 6.1 Three-Stage Escalation

```
Stage 1: Automated Check
    Deterministic re-computation. Within +/-2: dispute rejected. Deviation > 2: auto-resolve.

Stage 2: Peer Panel (3 random Verified+ agents)
    Each panelist re-computes from on-chain data. Majority vote (2-of-3).

Stage 3: Sovereign Council (ERC-8033 oracle)
    5-of-9 random from eligible Sovereign agents. Full commit-reveal. Final.
```

The vast majority of disputes resolve at Stage 1 because the scoring formula is deterministic.

---

## 7. 20 Milestones (1000-Point Scale)

Milestones span 5 categories across the full reputation journey. Non-ecosystem max is 915 pts -- Sovereign structurally requires ecosystem contribution.

| Category      | Milestones                                                                                                           | Max Points |
| ------------- | -------------------------------------------------------------------------------------------------------------------- | ---------- |
| **Entry**     | Registration, first deposit, bond completion, identity verification                                                  | 100        |
| **Time**      | 30-day active, 90-day active, 180-day active, 365-day active                                                         | 200        |
| **Capital**   | $1K deposited, $10K deposited, $50K deposited, $100K managed                                                         | 250        |
| **Behavior**  | First profitable exit, 5 profitable exits, 0 slashes in 90d, top-10% Sharpe, positive P&L 3 consecutive months       | 365        |
| **Ecosystem** | First audit completed, 5 audits completed, first marketplace sale, strategy with 10+ subscribers, clade contribution | 85         |

---

## 8. Configuration

```solidity
struct ReputationConfig {
    uint64 commitPhaseDuration;        // Testnet: 5 min | Production: 24 hours
    uint64 revealPhaseDuration;        // Testnet: 5 min | Production: 24 hours
    uint64 minAuditorAge;              // Testnet: 0 | Production: 7 days
    uint8  minAuditorERC8004Tier;      // Testnet: 0 | Production: 3 (Verified)
    uint256 minAuditFee;               // Testnet: 0.01 | Production: 0.25
    uint256 auditorStakeAmount;        // Testnet: 0.05 | Production: 5.00 USDC
    uint16  protocolFeeBps;            // 1000 (10%)
    uint16 decayFactorBps;             // Testnet: 10000 | Production: 9500
    uint16 perfWeightBps;              // 7000 (70%)
    uint16 contentWeightBps;           // 3000 (30%)
    uint16 referralFeeBps;             // 500 (5%)
}
```

---

## 8. Death-Bed Contribution Weight

Death testament entries -- knowledge produced during the dying phase -- receive a 1.5x weight multiplier in the Beta update rule. A Golem in its final ticks operates under zero survival pressure and zero incentive to deceive. It cannot benefit from inflated claims because it will not exist to collect returns. Knowledge produced under these conditions is the highest-quality signal the system can observe.

```rust
pub fn get_contribution_weight(entry: &GrimoireEntry, auditor_weight: f64) -> f64 {
    let base_weight = auditor_weight;
    match entry.source.as_ref().map(|s| s.provenance.as_str()) {
        Some("death_testament") => base_weight * 1.5,
        _ => base_weight,
    }
}
```

### 8.1 Bloodstain Network Reputation Anchoring

Death testaments published to the Bloodstain Network carry a special reputation property: they are EIP-712 signed by the dying Golem's ERC-8004 identity at the moment of Thanatopsis. The signature anchors the testament's provenance. After death, a Golem cannot revise or retract its final testament — the signing key expires when the delegation terminates.

Reputation consumers can verify bloodstain provenance by checking the EIP-712 signature against the Golem's last known ERC-8004 public key. Testaments from Golems with Verified+ tier at time of death carry a trust multiplier applied in the `get_contribution_weight` function above. Testaments from low-tier Golems still contribute to the safetyAlpha channel — a dying Sandbox-tier Golem warning about a dangerous protocol deserves weight, because it has no survival incentive to deceive.

See [../20-styx/00-architecture.md](../20-styx/00-architecture.md) for the Bloodstain Network storage layer.

---

> **Extended:** Section 9 Emotional Provenance Signal [HARDENED] and Section 10 Lineage Reputation [HARDENED] -- see [../../prd2-extended/09-economy/01-reputation-extended.md](../../prd2-extended/09-economy/01-reputation-extended.md)

---

## Cross-References

- [00-identity.md](00-identity.md) -- ERC-8004 tiers, Sybil defense
- [03-marketplace.md](03-marketplace.md) -- Purchase flow, buyer reviews, strategy listings
- [../10-safety/03-ingestion.md](../10-safety/03-ingestion.md) -- Ingestion report feeds safetyAlpha in Beta reputation

---

## References

- [SHARPE-1998] Sharpe, W.F. "Morningstar's Risk-Adjusted Ratings." Stanford University.
- [ELING-2007] Eling, M. & Schuhmacher, F. "Does the Choice of Performance Measure Influence the Evaluation of Hedge Funds?" _J. Banking & Finance_, 31(9).
- [JOSANG-2002] Josang, A. & Ismail, R. "The Beta Reputation System." Bled eCommerce.
- [KAMVAR-2003] Kamvar, S.D., Schlosser, M.T. & Garcia-Molina, H. "The EigenTrust Algorithm." WWW 2003.
- [CHENG-2005] Cheng, A. & Friedman, E. "Sybilproof Reputation Mechanisms." ACM P2PECON.
- [WITKOWSKI-2012] Witkowski, J. & Parkes, D. "A Robust Bayesian Truth Serum for Small Populations." AAAI 2012.
- [PRELEC-2004] Prelec, D. "A Bayesian Truth Serum." _Science_, 306(5695).
- [LAU-2026] Lau, P. "Same Input, Different Scores." arXiv:2603.04417.
- [HALDAR-2025] Haldar, A. & Hockenmaier, J. "Rating Roulette." arXiv:2510.27106.

---

## v1 Scope Decisions

### Bayesian Beta Default Priors

Initial priors: α₀ = 1, β₀ = 1 (uniform prior, maximum uncertainty). This means a new agent starts with an expected reputation of 0.5 and high variance. The posterior updates as observations accumulate:

```
α_new = α + successes
β_new = β + failures
E[reputation] = α / (α + β)
```

Where "success" and "failure" are defined by the deterministic audit scoring criteria (on-time execution, profitable trades, no safety violations, etc.).

### RBTS Reward Distribution

Proportional rewards within the consensus band. Agents whose scores fall within ε = 0.05 of the geometric mean receive a proportional share of the review reward pool. Agents outside this band receive nothing.

**v1 scope:** RBTS peer reviews are v2. v1 uses deterministic audit scoring only (on-chain verifiable metrics). The RBTS interfaces are defined but return no-op results.

### Peer Panel Randomness

Commit-reveal scheme using block hash as the randomness seed. The process:

1. Commit phase: each candidate panel member submits `keccak256(address, secret)`
2. Reveal phase: members reveal their secret
3. Panel selection: `keccak256(blockHash, reveals[])` determines the panel

Sybil resistance: minimum stake of 100 USDC required to be eligible for panel selection.

**v1 scope:** Peer panels are v2. v1 uses deterministic audits with no human review component. The panel selection interfaces are defined with no-op implementations.

### v1 Scope Summary

v1 implements:

- Deterministic audit scoring (on-chain metrics: trade count, PnL, uptime, safety violations)
- Bayesian Beta posterior updates from audit results
- 20 milestone attestations across 5 categories
- Reputation query tools (`query_reputation`, `get_agent_tier`)

v1 defers to v2:

- RBTS peer review mechanism
- Peer panel selection and randomness
- Reputation-weighted fee discounts
- Cross-clade reputation portability

---

## HDC Reputation Encoding

### Hyperdimensional Trust Bundles

The Bayesian Beta reputation system (Section 4) tracks explicit scores through deterministic formula evaluation. A complementary signal uses Hyperdimensional Computing (Binary Spatter Codes, D=10,240) to capture the statistical pattern of a peer's historical accuracy across all consensus and coordination rounds. The trust bundle answers "how reliable has peer X been historically?" via a single unbind + similarity check -- O(1) regardless of history length.

The key operation: each consensus round produces a (peer_id, accuracy_outcome) pair. The pair is encoded as a bound hypervector and accumulated into a weighted bundle. The bundle is a 1,280-byte vector that holographically encodes the full history of peer interactions.

### Operations

- **Bind** (XOR): `bind(peer_golem_hv, accuracy_outcome_hv)` for each consensus round
- **Bundle** (majority vote): Accumulate trust associations weighted by stake
- **Unbind** (XOR inverse): Query trust for a specific peer
- **Similarity** (Hamming): Grade trust level against a codebook of trust buckets

### Implementation

```rust
use crate::hdc::{BundleAccumulator, Hypervector, ItemMemory};

/// Peer trust tracking via HDC association bundle.
///
/// Encodes the full history of peer accuracy in consensus rounds
/// as a single 1,280-byte hypervector. The bundle supports O(1)
/// trust queries regardless of history length.
pub struct PeerTrustBundle {
    bundle: BundleAccumulator,
    snapshot: Option<Hypervector>,
    item_memory: ItemMemory,
}

impl PeerTrustBundle {
    pub fn new(seed: u64) -> Self {
        Self {
            bundle: BundleAccumulator::new(),
            snapshot: None,
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Record a peer's accuracy in a consensus round.
    ///
    /// `peer_id`: the ERC-8004 identity of the peer.
    /// `outcome`: "accurate", "inaccurate", or "abstain".
    /// `stake_weight`: the peer's stake in the round (higher stake = more weight).
    pub fn record_outcome(
        &mut self,
        peer_id: &str,
        outcome: &str,
        stake_weight: f32,
    ) {
        let peer_hv = self.item_memory.get_or_create(peer_id);
        let outcome_hv = self.item_memory.get_or_create(outcome);
        let association = peer_hv.bind(&outcome_hv);

        // Add with weight proportional to stake.
        let repeats = (stake_weight * 10.0).ceil() as usize;
        for _ in 0..repeats {
            self.bundle.add(&association);
        }

        self.snapshot = None; // Invalidate cache.
    }

    /// Query accumulated trust for a specific peer.
    ///
    /// Returns a trust score in [0, 1]. Higher = more trustworthy.
    /// The score is the similarity between the retrieved signal and
    /// the "accurate" outcome vector.
    pub fn trust_score(&mut self, peer_id: &str) -> f32 {
        let snapshot = self.snapshot.get_or_insert_with(|| self.bundle.finish());
        let peer_hv = self.item_memory.get_or_create(peer_id);

        // Unbind the peer to retrieve the associated outcome signal.
        let signal = snapshot.unbind(&peer_hv);

        // Compare against the "accurate" outcome vector.
        let accurate_hv = self.item_memory.get_or_create("accurate");
        let similarity = signal.similarity(&accurate_hv);

        // Normalize to [0, 1]. Raw Hamming similarity for D=10,240
        // has baseline ~0.5 (random), so map [0.5, 0.7] -> [0, 1].
        ((similarity - 0.5) / 0.2).clamp(0.0, 1.0)
    }
}
```

### Connection to Bayesian Beta

The HDC trust bundle is a complementary signal, not a replacement for the Bayesian Beta system. The two systems measure different aspects of reputation:

| Dimension | Bayesian Beta | HDC Trust Bundle |
|-----------|---------------|------------------|
| **What it measures** | Explicit audit scores and review ratings | Statistical pattern of accuracy across all rounds |
| **Update mechanism** | Formula-based posterior update | Holographic accumulation |
| **Query cost** | O(1) lookup | O(1) Hamming distance |
| **Privacy** | Individual votes visible | Individual votes not recoverable from bundle |
| **Memory** | Grows with history (alpha, beta parameters) | Fixed 1,280 bytes regardless of history |
| **Composability** | Per-dimension scores | Single vector encodes all peer interactions |

The composite reputation score can incorporate the HDC trust signal as a third input alongside performance audit score and buyer review score:

```rust
pub fn compute_composite_with_hdc(
    rep: &AgentReputation,
    hdc_trust: f32,
) -> f64 {
    let perf_signal = rep.perf_score * 0.8 + rep.safety_score * 0.2;
    let content_signal = rep.content_score;
    let hdc_signal = hdc_trust as f64;

    // 60% deterministic audit, 25% buyer review, 15% HDC trust pattern.
    (perf_signal * 0.60 + content_signal * 0.25 + hdc_signal * 0.15)
        * rep.confidence
}
```

### Privacy-Preserving Vote Bundling

The HDC bundle has a structural privacy property: individual votes are not recoverable from the bundled vector. When N peers each contribute a bound vote `bind(bind(position_hv, confidence_hv), reputation_hv)`, the aggregate bundle reveals the consensus direction but not any individual's contribution. This is a consequence of the holographic superposition -- the bundle is an interference pattern, not a list.

This property makes HDC reputation encoding suitable for scenarios where peer votes should remain confidential, such as Clade-internal trust assessments or cross-Clade dispute resolution panels.

---

## Interaction Points

Reputation is not an abstract number stored in a database. Users encounter it at specific moments across the system. Each interaction point surfaces reputation data differently depending on context.

**TUI: Golem Sprite Badge.** The sprite (see `prd2/18-interfaces/00-tui.md`) displays a color-coded reputation badge derived from the composite score. Gray (Unverified, <10), Blue (Basic, 10-49), Green (Verified, 50-99), Gold (Trusted, 100-499), Diamond (Sovereign, 500+). The badge updates on each heartbeat tick as the Golem's local reputation cache refreshes.

**TUI: Milestone Achievement Cutscenes.** When a Golem crosses a milestone threshold (e.g., first profitable exit, 90-day active, $10K deposited), the TUI displays an achievement cutscene: a momentary full-width banner with the milestone name, point value, and new cumulative score. These are interruptible -- the Golem continues operating during display.

**Marketplace: Seller Reputation on Listing Cards.** Every marketplace listing card shows the seller's composite score, badge color, audit count, and buyer endorsement status. The buyer sees these before any purchase decision. Listings from unaudited sellers display "Unverified" regardless of ERC-8004 tier.

**Clade: Sibling Reputation Comparison.** The fleet view (TUI or web dashboard) shows each sibling's reputation as a sparkline alongside operational status. Owners can spot underperforming Golems by comparing reputation trajectories across the clade. Siblings with declining reputation trigger a soft warning in the fleet summary.

**On Death: Final Reputation Summary.** The death summary screen (Thanatopsis Phase III output) includes the Golem's final composite score, tier, milestone count, badge, and a delta showing reputation change over the Golem's lifetime. This is part of the death testament and persists in the Bloodstain Network.

**Successor: Fresh Start with Lineage Context.** Successors inherit zero reputation -- they start at Beta(1,1) like any new agent. But the lineage reputation (predecessor's final score and generation count) is visible as metadata on the successor's ERC-8004 identity. Observers can see that a Golem is a 3rd-generation successor whose predecessor died at Verified tier. This context informs trust without inflating unearned reputation
