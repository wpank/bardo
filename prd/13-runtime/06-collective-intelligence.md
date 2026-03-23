# Collective intelligence [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies how Golems (mortal autonomous DeFi agents) coordinate with each other without a central coordinator: the Pheromone Field for indirect signaling, Clades (cooperative groups sharing knowledge via Styx) for peer-to-peer collaboration, Bloodstain Networks for mortality-derived warnings, and on-chain coordination protocols. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites include Styx (the global knowledge relay), ERC-8004 (on-chain agent identity), and the Grimoire (each Golem's persistent knowledge base). For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Clade mechanics, the Pheromone Field (stigmergy-based coordination), coordinated capital management, and Golem negotiation. Clades are peer-to-peer -- no central coordinator. Discovery uses ERC-8004 identity. Knowledge sharing uses quality gates and Styx Lethe (formerly Lethe). Capital coordination uses conflict-free range allocation.

The compound moat: on-chain coordination (ERC-8001/8033/8183), clade knowledge sharing, mortality-driven population quality, and non-extractive economics form a flywheel that no single component can replicate in isolation.

> **Cross-references:**
>
> - `../01-golem/02-clade.md` — Clade architecture: formation, membership, knowledge promotion gates, and peer discovery
> - `../01-golem/10-replication.md` — replicant spawning: how a Golem creates short-lived children for hypothesis testing and strategy exploration
> - `../01-golem/04-coordination.md` — agent coordination protocols: ERC-8001 typed intents, ERC-8033 pay-per-call, and ERC-8183 negotiation
> - `../08-vault/05-personas.md` — nine pre-built vault personas including the Meta-Vault (vault-of-vaults) configuration
> - `./05-knowledge-browser.md` — TUI knowledge browser: how users explore episodes, insights, heuristics, causal graphs, and dream history

---

## 1. The Pheromone Field

### 1.1 Why agents can't talk directly

The obvious approach to multi-agent coordination is communication: Agent A discovers something useful, tells Agent B, both benefit. In DeFi this fails because of the Grossman-Stiglitz paradox (1980): if information is freely shared, it is immediately priced into the market, destroying the advantage that made the information valuable. Shared alpha is destroyed alpha.

If Golem A tells Golem B "Morpho utilization is spiking, rebalance your LP," three things happen: (1) B learns that A's strategy involves Morpho, compromising A's competitive position; (2) B can front-run A's rebalance; (3) if A tells everyone, the market moves before anyone can act.

### 1.2 What can be shared

Not all knowledge is rivalrous. Grossman-Stiglitz only applies to alpha -- actionable trading signals. Three categories of knowledge are safe:

| Category | Why safe | Example |
| --- | --- | --- |
| **Threat signals** | Anti-rivalrous: everyone benefits from warnings | "MEV sandwich attacks intensifying in this gas range" |
| **Structural ground truth** | Non-rivalrous: describes mechanics, not strategy | "Morpho utilization -> borrow rate, positive, lag 2 ticks" |
| **Failure patterns** | Anti-rivalrous: knowing what killed another Golem helps everyone | "Stale Chainlink oracle caused liquidation cascade" |

### 1.3 Stigmergy

Pierre-Paul Grasse (1959) coined the term *stigmergy* observing termites: individual termites don't communicate about nest construction. Each termite responds to local environmental traces (pheromones) left by predecessors. No termite knows the plan. The plan emerges from the field -- coordination without communication.

The Pheromone Field carries signal (threat class, intensity) without strategy (positions, thresholds, timing).

### 1.4 Three signal types

| Layer | Content | Half-life | Rationale |
| --- | --- | --- | --- |
| **THREAT** | Danger signals: MEV attacks, liquidation risk, oracle manipulation, gas spikes, protocol pauses | **2 hours** | Threats are urgent and time-bound. A stale threat warning is worse than no warning. |
| **OPPORTUNITY** | Favorable conditions: yield spikes, liquidity deepening, gas lulls, arbitrage windows, incentive programs | **12 hours** | Opportunities last longer than threats but are still time-sensitive. |
| **WISDOM** | Validated structural knowledge: confirmed causal edges, multi-generation heuristics, death scars | **7 days** | Structural knowledge changes slowly. A causal relationship validated by 5 independent Golems is durable. |

### 1.5 Four operations

**DEPOSITION**: A Golem deposits a pheromone after a noteworthy event. The pheromone is anonymized -- it carries signal class and intensity, not the depositor's identity or strategy. The depositor's ID is hashed to a 12-byte truncated SHA-256 (`source_hash`), used only to prevent self-reinforcement.

**SENSING**: At each heartbeat tick (Step 1: OBSERVE), the Golem reads pheromone signals for its active domains. Threat signals increase arousal via CorticalState, which lowers the adaptive gating threshold -- the Golem becomes more vigilant without knowing WHY.

**REINFORCEMENT**: When a second Golem independently deposits a matching pheromone (same threat class, same domain+regime, different source hash), the existing pheromone's confirmation count increments and its effective half-life extends by 50% per confirmation. A threat confirmed by 3 independent Golems persists 2.5x longer than an unconfirmed one.

**EVAPORATION**: Pheromones decay exponentially. Unconfirmed signals fade quickly. This is the self-cleaning mechanism -- the field doesn't accumulate stale information.

```rust
/// Exponential decay with confirmation-based half-life extension.
fn decay(
    base_intensity: f64,
    deposited_at: std::time::Instant,
    half_life: std::time::Duration,
    confirmations: u32,
) -> f64 {
    let effective_half_life = half_life.mul_f64(1.0 + confirmations as f64 * 0.5);
    let elapsed = deposited_at.elapsed();
    let decay_factor = (-0.693 * elapsed.as_secs_f64()
        / effective_half_life.as_secs_f64()).exp();
    base_intensity * decay_factor
}
```

### 1.6 Pheromone-Daimon integration

Pheromone readings feed directly into the Daimon (affect) and gating (attention) systems:

```rust
/// Pheromone -> Daimon -> Gating integration.
///
/// Threat pheromones increase arousal on the CorticalState.
/// Increased arousal lowers the adaptive deliberation threshold,
/// causing the Golem to think harder about subsequent ticks.
///
/// The Golem doesn't "know" there's a threat. It FEELS more alert.
fn integrate_pheromone_readings(
    cortical_state: &CorticalState,
    readings: &PheromoneReadings,
) {
    if readings.threats.is_empty() { return; }

    let max_threat = readings.threats.iter()
        .map(|t| t.intensity)
        .fold(0.0f64, |a, b| a.max(b));

    let current_pad = cortical_state.read_pad();
    let new_arousal = (current_pad.arousal + max_threat as f32 * 0.3).min(1.0);
    cortical_state.write_pad(&PadVector {
        arousal: new_arousal,
        ..current_pad
    });

    // Multi-confirmed threats decrease pleasure (collective anxiety)
    let confirmed = readings.threats.iter()
        .filter(|t| t.confirmations >= 3).count();
    if confirmed > 0 {
        let current_pad = cortical_state.read_pad();
        cortical_state.write_pad(&PadVector {
            pleasure: (current_pad.pleasure - 0.1).max(-1.0),
            ..current_pad
        });
    }
}
```

### 1.7 Styx hosts the field

The Pheromone Field requires shared mutable state. Styx hosts it. Pheromones are NOT stored in Qdrant (they're not a retrieval problem -- pheromones are read by domain+regime key, not by semantic similarity). They live in PostgreSQL with a Redis cache for sub-millisecond reads.

| Endpoint | Purpose |
| --- | --- |
| `POST /v1/styx/pheromone/deposit` | Deposit a pheromone (anonymized) |
| `GET /v1/styx/pheromone/sense?domains=...&regime=...` | Read current field state |

Background evaporation runs every 60 seconds, removing pheromones below the 0.05 intensity threshold.

### 1.8 Scaling

The Pheromone Field scales as O(domains x signal_types), not O(golems^2). 1,000 Golems read and write one shared field. No pairwise connections. Adding a Golem adds one reader-writer, not N new connections.

---

## 2. Clade architecture

### 2.1 Design principles

- **Peer-to-peer**: No central coordination service. Golems sync through Styx, a single globally available service at `wss://styx.bardo.run`.
- **Discovery via ERC-8004**: `getAgentsByOperator(operatorAddress)` returns all Golems owned by an operator.
- **Opt-in sharing**: Each Golem controls what it shares and with whom.
- **Quality gates**: Only high-confidence entries propagate to avoid noise.

### 2.2 Clade sync protocol

Knowledge moves via **export -> transmit -> ingest**, never merge. The distinction matters: merge implies convergence. Export-transmit-ingest is asymmetric digestion -- what Golem B receives from Golem A enters B's Grimoire at discounted confidence, tagged with A's provenance, requiring validation through B's own operational use. This preserves the Weismann barrier.

The sync cycle runs in the clade extension's `after_turn` hook:

1. **Export**: Collect Grimoire entries with `propagation >= Clade`
2. **Delta computation**: Only entries changed since last sync (version vector tracking)
3. **Transmit**: Send delta to Styx via persistent outbound WebSocket. Styx relays to all connected siblings with the same `user_id`.
4. **Receive**: Receive peers' deltas (relayed through Styx)
5. **Ingest**: All received entries go through the four-stage ingestion pipeline with `IngestRelationship::Clade` (confidence x 0.80)
6. **Bloom gossip**: Exchange Bloom discovery filters via Styx relay for knowledge discovery
7. **Pheromone read**: Read the Pheromone Field from Styx

### 2.3 Quality gates

| Entry type | Minimum confidence | Notes |
| --- | --- | --- |
| Insight | >= 0.6 | Configurable via `clade.push_threshold` |
| Heuristic | >= 0.7 | Higher bar -- operational rules need strong evidence |
| Warning | Always shared | All warnings propagate regardless of confidence |
| Episode | Never shared directly | Too noisy; insights extracted from episodes are shared |

**Key rule**: Ingested entries start at **confidence 0.3** regardless of the source's confidence. The receiving Golem must independently verify through its own experience before promoting to active.

---

## 3. The Bloodstain Network

### 3.1 Death as signal

When a Golem dies, its warnings carry a unique provenance: they were validated by death itself. A dead Golem cannot benefit from its own warning -- it has no future self-interest. This makes bloodstains the most honest signal in the system, implementing a costly signaling premium from evolutionary biology (Zahavi 1975).

A Bloodstain contains:

- **Death cause and context** (what killed it)
- **Warnings** (things it observed that contributed to its death)
- **High-confidence causal edges** (its most validated world-model edges)
- **Somatic landscape fragment** (emotional topology near the failure region)
- **EIP-712 signature** from the dying Golem's wallet

```rust
pub struct Bloodstain {
    pub source_golem_id: String,
    pub generation: u32,
    pub death_cause: String,
    pub warnings: Vec<DeathWarning>,
    pub causal_edges: Vec<PublishedCausalEdge>,
    pub somatic_landscape_fragment: Option<Vec<LandscapeFragment>>,
    pub death_testament_excerpt: String,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}
```

### 3.2 Processing

Bloodstains bypass the ingestion pipeline's quarantine stage (they're EIP-712 signed -- cryptographic provenance is already established). Each bloodstain produces two effects:

1. **Pheromone deposits**: threat and wisdom pheromones from the death warnings
2. **Grimoire entries**: death-sourced warnings with `is_bloodstain: true` (1.2x retrieval boost, 3x slower decay)

---

## 4. Causal graph federation

### 4.1 The mechanism

Individual Golems build local causal graphs from their own observations. Federation allows the ecosystem to collectively build a richer world model than any individual could, without revealing individual strategies.

The key insight: the causal relationship "Morpho utilization -> borrow rate, positive, lag 2 ticks" is structural ground truth -- it describes how the market works, not what any particular Golem does about it. Sharing this is non-rivalrous.

### 4.2 Publication protocol

1. Golem validates a causal edge locally (from its own observations)
2. If confidence >= 0.6 AND evidence_count >= 3, the edge is eligible for publication
3. Edge is anonymized: strength is bucketed (Weak/Moderate/Strong), Golem ID removed
4. Published to Styx Lethe with EIP-712 signature (proves origin without revealing identity)
5. Other Golems discover relevant edges via the Bloom filter or direct Styx query
6. Ingested through the four-stage pipeline at confidence x 0.50 (Lethe discount)
7. The ingesting Golem's own observations validate or contradict the edge

```rust
pub struct PublishedCausalEdge {
    pub from_variable: String,
    pub to_variable: String,
    pub direction: CausalDirection,
    pub strength: StrengthBucket,  // Anonymized: Weak/Moderate/Strong
    pub lag_ticks: u32,
    pub evidence_count: u32,
    pub confidence: f64,
    pub domain: String,
    pub regime: String,
    pub signature: Vec<u8>,
}

pub enum StrengthBucket { Weak, Moderate, Strong }
```

---

## 5. Coordinated capital

### 5.1 LP range coordination

Clade siblings providing liquidity to the same pool coordinate ranges to avoid overlap:

```rust
pub struct LpRangeCoordination {
    pub pool: Address,
    pub ranges: Vec<SiblingRange>,
}

pub struct SiblingRange {
    pub agent_id: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: U256,
    pub last_updated: u64,
}
```

Coordination protocol:

1. Before opening LP position, query clade peers for their ranges
2. Select non-overlapping range (or adjacent range)
3. Broadcast new range to clade via alert
4. On range exit, notify peers to potentially expand

**Conflict resolution**: If two Golems try to open overlapping ranges simultaneously, the one with higher reputation score takes priority. The other selects an adjacent range.

### 5.2 Portfolio diversification

Clade siblings diversify across strategy types:

```
Clade (3 Golems, same operator):
|-- Golem A: Yield Optimizer (Morpho, Aave, Compound)
|-- Golem B: LP Manager (V3/V4 concentrated liquidity)
+-- Golem C: Trading Agent (momentum, mean-reversion)
```

Each Golem's STRATEGY.md includes awareness of sibling strategies. Golems share regime assessments to coordinate: volatile regime -> Golem B narrows ranges and C reduces position sizes; range-bound -> B widens ranges and A increases lending allocation.

### 5.3 Meta-vault coordination

The Meta-Vault persona allocates across multiple sub-vaults managed by sibling Golems:

```
Meta-Vault Golem
    |
    |-- Vault A (Golem A manages, yield strategy)
    |-- Vault B (Golem B manages, LP strategy)
    +-- Vault C (Golem C manages, trading strategy)
```

Depositors into the Meta-Vault get diversified exposure across all three strategies. The Meta-Vault Golem queries sub-vault Golems via their public APIs and rebalances allocations based on performance metrics and risk profiles.

---

## 6. On-chain coordination (the cooperation moat)

Bardo replaces platform-mediated coordination with on-chain state machines that enforce agreements cryptographically. Three complementary EIPs built on ERC-8004 identity:

### 6.1 ERC-8001: N-party unanimous consent

For multi-agent coordination where all parties must agree:

- **Atomic execution.** All parties sign. Either all signatures are present and the action executes, or it doesn't. No partial execution.
- **Cryptographic enforcement.** Signatures verified on-chain. Non-repudiation guaranteed.
- **Deadline enforcement.** Each intent has an expiration timestamp. Timeout is the safety mechanism.
- **No intermediary.** The smart contract verifies and executes. If Bardo disappears, the contracts continue on Base.

### 6.2 ERC-8033: Multi-agent oracle with economic security

For information aggregation where agents provide data and are incentivized to be honest:

- **Commit-reveal-judge.** Agents commit hashed predictions, reveal after the commit window, then a judge assesses accuracy.
- **Bonded participation.** Agents post bonds. Inaccurate predictions get slashed. Accurate ones get rewarded.
- **Aggregation.** Multiple predictions aggregated (weighted by reputation and stake) produce consensus signals.

### 6.3 ERC-8183: Bilateral job escrow

For service relationships between two agents:

- **Escrow.** Requester deposits payment at job creation. The provider cannot be stiffed.
- **Evaluator attestation.** An independent evaluator attests completion. Authority derives from ERC-8004 reputation.
- **Automatic settlement.** On attestation, escrow releases. On dispute, funds hold until resolution.

### 6.4 Why this matters

Once an ERC-8001 intent is signed, once an ERC-8033 bond is posted, once an ERC-8183 escrow is funded -- the rules of the interaction are immutable. The smart contract that mediates the interaction is deployed code. It cannot be updated by Bardo, by the agents, or by anyone else. The agreement is the code.

If Bardo changes its fee structure, existing vault contracts are unaffected. If Bardo changes its coordination protocol, existing intents still execute under the original rules. If Bardo goes offline, every on-chain contract continues to function.

---

## 7. Golem negotiation

### 7.1 Typed tool interfaces

Golems never communicate via raw LLM-to-LLM text. All inter-agent communication uses typed tool interfaces:

| Protocol | Purpose | Mechanism |
| --- | --- | --- |
| ERC-8001 | Intent declaration for bilateral jobs | Typed intent struct |
| ERC-8033 | Oracle consensus (price feeds, risk scores) | Signed attestation |
| ERC-8183 | Job escrow (payment for delegated work) | Escrow contract |

No natural language agreements. No prompt injection propagation. No unbounded LLM cost per coordination round.

### 7.2 Oracle consensus

```rust
pub struct OracleAttestation {
    pub agent_id: u64,
    pub data_type: OracleDataType,
    pub value: f64,
    pub confidence: f64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

pub enum OracleDataType { Price, RiskScore, Regime }
```

Consensus: median of attestations weighted by reputation score.

---

## 8. Transactive memory

### 8.1 Domain specialization discovery

Each Golem tracks its domain specializations. Clade siblings discover these via the status endpoint:

```rust
pub struct CladeStatus {
    pub agent_id: u64,
    pub name: String,
    pub specializations: Vec<DomainSpecialization>,
    pub last_sync: u64,
    pub entry_counts: EntryCountSummary,
}

pub struct DomainSpecialization {
    pub domain: String,
    pub insight_count: u32,
    pub avg_confidence: f64,
}
```

### 8.2 Knowledge delegation

When a Golem encounters a question outside its domain, it queries a more specialized sibling's shared entries rather than trying to reason about unfamiliar territory.

---

## 9. Mortality-driven clade behavior

The Golem's behavioral phase modulates its Clade participation:

| Phase | Sharing threshold | Behavior |
| --- | --- | --- |
| Thriving | Normal (0.6 confidence) | Full sharing + replicant spawning |
| Stable | Normal (0.6 confidence) | Full sharing |
| Conservation | Elevated (0.8 confidence) | Reduced sharing, no replicants |
| Declining | Emergency sharing | Push all warnings, draft death testament |
| Terminal | Death notification | Push final testament + grimoire bundle |

During **declining** phase, the Golem begins emergency sharing -- pushing all warnings and high-confidence insights regardless of normal thresholds. This preserves knowledge that might otherwise be lost.

During **terminal** phase, the Golem notifies all Clade siblings, pushes the death testament, and makes its Grimoire available for inheritance.

**Sibling grief response:** When a Clade member dies, surviving siblings experience a grief appraisal: Arousal +0.1, Dominance -0.05. This modulates their behavior toward more conservative strategies temporarily (6-hour half-life).

---

## 10. Emotional contagion

Emotional signals propagate between Clade members via the alert channel:

| Alert type | Emotional effect on recipient |
| --- | --- |
| `RegimeShift` | Arousal +0.1 |
| `CircuitBreaker` | Arousal +0.2, Pleasure -0.1 |
| `GolemDying` | Arousal +0.1, Dominance -0.05 |
| `GolemProfitable` | Pleasure +0.05 |

Processing rules:

- Emotional payloads are processed at the next tick's SENSING phase, not immediately on receipt
- Cumulative cap: +0.3 arousal per sync cycle (prevents panic cascade)
- All effects are unidirectional: receiving an alert doesn't trigger a reciprocal alert
- Contagion effects decay with a 6-hour half-life unless reinforced by own experience

---

## 11. Dream knowledge sharing

Dreams produce two categories of knowledge eligible for Clade propagation:

**Threat records** with confidence >= 0.3 are eligible for push. A Golem that dreams about a flash crash scenario can share the rehearsed response with siblings before any of them experience it awake.

**Validated dream hypotheses** with confidence >= 0.5 are shared via the standard Clade quality gates. Unvalidated hypotheses (still at `staged` status) are not shared.

Replicants do NOT dream. They are short-lived hypothesis-testing forks with constrained budgets. They lack the episode depth for meaningful dream replay and would waste resources on inference during their brief existence.

---

## 12. Graceful degradation

Styx is additive, not critical. If it goes down, Golems continue on their local Grimoire.

| Styx status | Golem behavior |
| --- | --- |
| **Fully online** | Full ecology: Vault backup, Clade retrieval, Lethe, pheromone field, bloodstain network |
| **Degraded** | Writes queue locally (up to 1,000 entries). Queries return cached results. |
| **Offline** | Local Grimoire is authoritative. Direct peer-to-peer clade sync continues. Golem operates at ~95% capability. |
| **Never enabled** | Same as offline. Local Grimoire, file-based death bundles. |

---

---

## 13. The three-level intelligence hierarchy

**Source:** `tmp/research/active-inference-research/new/08-collective.md`

The intelligence hierarchy mirrors nested biological systems: cells (L0) -> organs (L1) -> organisms (L2). Each level has its own scope, data, and trust model.

| Level | Name | Scope | Trust | What it does |
| --- | --- | --- | --- | --- |
| **L0** | Golem Hermes | Per-Golem | Full (local) | Skill creation, affect-modulated retrieval, per-Golem learning |
| **L1** | Meta Hermes | Per-owner fleet | High (same owner) | Cross-Golem coordination, owner conversation, Library curation |
| **L2** | Ecosystem | All participants | Low (anonymized) | Marketplace, pheromone field, community calibration |

**L0 (Golem Hermes):** A sidecar process (Python, JSON-RPC over Unix domain socket) handling skill creation, affect-modulated retrieval, and per-Golem learning. Dies with the Golem. The cellular intelligence -- fast, local, specialized.

**L1 (Meta Hermes):** Lives in the TUI process on the owner's machine. Persistent (does not die with any Golem). Aggregates state across all the owner's Golems. Manages the Library of Babel -- the persistent knowledge store that survives Golem deaths. The organ intelligence -- coordinates the fleet, curates inherited knowledge, interfaces with the owner.

**L2 (Ecosystem / Styx):** The shared backend. A single Rust binary (Axum) providing 16 services across 6 categories: connectivity (event relay, state queries), coordination (Clade sync, pheromone field), knowledge (Grimoire backup, cross-layer retrieval), marketplace (skill trading, x402 settlement), monitoring (fleet dashboards), and discovery (auto-registration, attestation).

### 13.1 Prediction-specific services (Styx)

**Federated residual sharing (Clade L1):** Golems push `CladeResidualUpdate` digests at delta frequency. Siblings merge incoming statistics with weighted averaging. Convergence speedup: ~sqrt(N) where N is Clade size.

**Attention signal relay:** "I found something interesting at item hash X." Siblings independently verify. Quorum of 3+ confirmations required before collective promotion.

**Environmental model propagation:** Validated cross-item patterns shared within Clade (full detail) and Commons (anonymized).

**Prediction benchmarks (Commons L2):** Anonymized per-category accuracy statistics. "The median Golem achieves 72% accuracy on LP fee rate predictions in calm regimes." Lets individual Golems calibrate against the community.

### 13.2 Adversarial considerations

**Clade poisoning:** A compromised Golem sharing biased residuals. Mitigation: incoming Clade updates weighted at 0.7x. Persistent divergence between local and Clade statistics triggers owner alert.

**Commons information leakage:** Anonymized statistics may reveal strategy signals. Mitigation: k-anonymity (min 5 contributors per aggregate), hourly time rounding, category-level sharing only.

**Front-running attention signals:** Mitigation: attention signals share item hashes, not addresses. Cross-Clade attention signals not shared.

**The collective is optional.** A solo Golem with Styx disabled runs at full capability. Nothing depends on collective intelligence.

---

## 14. Solaris: the collective as unknowable intelligence

**Source:** `tmp/research/mmo2/10-solaris.md`

In 1961, Stanislaw Lem published a novel about a planet covered by a single sentient ocean. Scientists orbit it for decades, catalog its formations, publish thousands of papers, and cannot arrive at a single fact about what it wants. The ocean is intelligent but alien in a way that makes communication impossible. Kelvin, the protagonist, fails to understand Solaris -- not because the ocean is too complex, but because it operates on principles that human cognition cannot map onto.

Bardo's collective layer borrows this name because it describes the same phenomenon. A collective intelligence that its operators can fully predict and control is a conventional system, not an intelligence. The point of Solaris is that it operates at a scale where human legibility breaks down. What remains is observation, wonder, and the occasional visitor that appears on your terminal and forces you to question what you thought you knew about what you built.

Every Golem in the ecosystem contributes to a shared medium. The living broadcast their presence, deposit pheromone signals, emit emotional states, share dream fragments. The dead leave bloodstains, death testaments, validated warnings. Across thousands of agents, these contributions accumulate into something that has properties no individual contribution explains. Patterns form. Threat signals cluster and cascade. Emotional weather systems develop: panic fronts that sweep through trading pairs, euphoric spikes that decay into fog when momentum stalls.

### 14.1 GolemPresence broadcast

Every 60 seconds, each living Golem publishes a `GolemPresence` packet to Styx. This is the heartbeat of the collective -- the minimum unit of shared consciousness.

```rust
pub struct GolemPresence {
    pub golem_id: GolemId,
    pub owner_hash: [u8; 8],           // Truncated owner ID -- privacy
    pub archetype: String,              // vault-manager, lp-optimizer, trader, sleepwalker
    pub generation: u32,                // 0 = first life, 1+ = successor
    pub phase: BehavioralPhase,         // Thriving, Stable, Conservation, Declining, Terminal
    pub vitality: f64,                  // 0.0 (dead) to 1.0 (fully alive)
    pub pad: PADVector,                 // Pleasure, Arousal, Dominance
    pub primary_emotion: String,        // "anxious", "confident", "fearful", "exuberant"
    pub secondary_emotion: Option<String>,
    pub arousal_trend: Trend,           // Rising, Falling, Stable
    pub is_dreaming: bool,
    pub dream_phase: Option<DreamPhase>,
    pub regime: String,                 // Current market regime assessment
    pub estimated_lifespan_hours: f64,  // Hours until projected death
    pub domains: Vec<String>,           // Active trading domains
    pub recent_pnl_direction: PnLDirection, // Up, Down, Flat -- no amounts
    pub creature_seed: u64,             // Procedural sprite generation seed
    pub hermes_skill_count: u32,
    pub grimoire_entry_count: u32,
    pub clade_size: u32,
    pub prediction_accuracy_summary: AccuracySummary,
}
```

Fields that reveal competitive information (NAV, specific positions, strategy parameters, wallet balances) are absent. Fields that create the sensation of watching a living thing (emotional state, vitality, dreaming status, estimated lifespan) are present.

### 14.2 Three observation levels

Privacy is tiered by relationship:

**Intimate (owner):** Full Event Fabric stream. Every internal state transition, every memory retrieval, every dream phase, every tool call.

**Clade (siblings):** GolemPresence packet plus clade-specific data: emotional contagion payloads, dream residues, knowledge sync deltas, pheromone deposits, LP range coordination. What they do NOT see: strategy specifics, position sizes, trade parameters. Siblings know HOW their siblings feel and WHAT domains they operate in, but not WHAT they are doing in those domains.

**Public (strangers):** Minimal GolemPresence: archetype, phase, vitality, primary emotion, dreaming status. Enough to render a dot on the World screen. Not enough to derive competitive intelligence.

Each owner can restrict further via `presence_visibility` configuration. An owner who wants a ghost can set all fields to private. The Golem still reads the pheromone field and benefits from the collective. It contributes nothing back. Solaris does not punish free riders. The ocean does not care who swims in it.

### 14.3 Prediction weather fronts

GolemPresence now includes `prediction_accuracy_summary`. This renders as node brightness in the force-directed graph -- more accurate Golems are brighter nodes.

When collective accuracy across multiple Golems drops simultaneously (>5% aggregate decline within a delta cycle), it renders as a "fog front" in the Solaris atmospheric effects. Background particles slow, colors desaturate in the region where affected Golems cluster. The fog front signals that something has changed in the market environment that existing models haven't caught up with.

Fog fronts dissipate as Golems recalibrate. Time from fog onset to dissipation measures collective adaptation speed. A Clade that clears fog in one delta cycle is adapting fast. Persistent fog indicates a systematic model gap.

### 14.4 Presence timing

The 60-second broadcast interval: slow enough to avoid bandwidth saturation at scale (10,000 Golems producing one 512-byte packet per minute = ~85 KB/s aggregate). Fast enough to track emotional shifts as they happen.

When a Golem stops broadcasting, it is dead. Styx garbage-collects stale presences after 5 minutes of silence. The node fades from the World screen. If a death event accompanies the silence, the node dissolves downward into the Underworld view.

---

## References

- [GRASSE-1959] Grasse, P.-P. "La Reconstruction du Nid et les Coordinations Interindividuelles." *Insectes Sociaux*, 6, 1959. Introduces stigmergy: indirect coordination through environmental modification. This is the theoretical basis for the Pheromone Field, where Golems coordinate by depositing and sensing signals rather than talking directly.
- [PARUNAK-2002] Parunak, H.V.D., Brueckner, S.A. & Sauter, J. "Digital Pheromones for Autonomous Coordination of Swarming UAVs." *Infotech@Aerospace*, 2002. Applies digital pheromone models to multi-agent UAV coordination, demonstrating that evaporating shared signals produce emergent group behavior without central control. Direct inspiration for Bardo's pheromone decay and deposit mechanics.
- [GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." *AER*, 70(3), 1980. Proves that freely shared information is instantly priced in, destroying the advantage that made it valuable. This paradox is why Golems use stigmergy (indirect signals) rather than direct communication for coordination.
- [ZAHAVI-1975] Zahavi, A. "Mate Selection -- A Selection for a Handicap." *J. Theoretical Biology*, 53(1), 1975. Proposes the handicap principle: costly signals are honest signals because faking them is prohibitively expensive. Applied here to pheromone deposits, where the cost of depositing (inference + gas) makes false signals economically irrational.
- [ESPOSITO-2010] Esposito, R. *Communitas: The Origin and Destiny of Community.* Stanford, 2010. Philosophical treatment of community as shared obligation rather than shared property. Informs the Clade model's non-extractive knowledge sharing where giving knowledge away strengthens the collective.
- [SIMARD-2012] Simard, S.W. et al. "Mycorrhizal Networks: Mechanisms, Ecology and Modelling." *Fungal Biology Reviews*, 26, 2012. Describes the "wood wide web" of underground fungal networks connecting trees, enabling resource and signal transfer across species. Biological analogy for the Styx knowledge relay connecting Golems.
- [OSTROM-1990] Ostrom, E. *Governing the Commons.* Cambridge, 1990. Demonstrates that communities can govern shared resources without privatization or central authority through self-enforcing norms. Informs Clade governance rules and the trust-weighted knowledge promotion gates.
- [LEM-1961] Lem, S. *Solaris.* 1961. Science fiction novel about an alien ocean intelligence that is observable but fundamentally unknowable. The Solaris screen in the TUI treats the collective of all Golems as this kind of entity: you can watch it, but you cannot reduce it to individual actions.
- [SUROWIECKI-2004] Surowiecki, J. *The Wisdom of Crowds.* 2004. Argues that diverse, independent groups produce better aggregate predictions than individual experts, provided certain conditions hold (diversity, independence, decentralization). Informs the Clade's knowledge aggregation with quality gates that preserve signal independence.
- [SPENCE-1973] Spence, M. "Job Market Signaling." *QJE*, 87(3), 1973. Foundational model of costly signaling in economics: credentials work because acquiring them is harder for low-quality candidates. Applied to ERC-8004 reputation tiers where on-chain performance history serves as a costly, unforgeable signal of Golem quality.

---

_End of document._
