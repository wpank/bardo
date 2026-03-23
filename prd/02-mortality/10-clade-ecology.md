# Superorganism Dynamics: Clade Ecology and Collective Intelligence [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> _"Selfishness beats altruism within groups. Altruistic groups beat selfish groups. Everything else is commentary."_
> -- David Sloan Wilson and Elliott Sober (1994)

---

## Document Context

| Field            | Value                                                                        |
| ---------------- | ---------------------------------------------------------------------------- |
| **Crates**       | `golem-core`, `golem-grimoire`, `golem-clade`, `golem-coordination`          |
| **Depends on**   | `00-thesis.md` (foundational mortality thesis), `09-fractal-mortality.md` (phage immune system, multi-scale mortality), `../01-golem/10-replication.md` (Replicant spawning and lifecycle)   |
| **Shared infra** | `golem-runtime`, `bardo-styx`                                                |

---

> **Reader orientation:** This document specifies Clade (sibling Golems sharing a common ancestor) ecology. The core argument: the individual Golem (mortal autonomous DeFi agent) is a cell; the Clade is the organism; the DeFi ecosystem is the environment. Individual death is how the collective thinks. The document covers superorganism theory, stigmergic coordination through Styx (global knowledge relay and persistence layer), pheromone-based signaling, and how mortality-driven knowledge sharing produces collective intelligence that no immortal agent system can match. See `prd2/shared/glossary.md` for full term definitions.

## The Argument: The Clade Is the Organism

Individual Golem mortality is not the endpoint of the architecture. It is the mechanism by which a higher-order entity -- the Clade -- achieves collective intelligence that no individual Golem could attain. The Golem is the cell; the Clade is the organism; the DeFi ecosystem is the environment. Death is how the organism thinks.

This document establishes the theoretical foundation and implementation architecture for Clade-level ecological dynamics: how a collection of mortal, cooperating, competing Golems self-organizes into a superorganism whose intelligence exceeds the sum of its parts, and how individual mortality is the essential mechanism that maintains this collective intelligence.

---

## The Coordination Paradox

### Why Agents Can't Just Talk

The obvious approach to multi-agent coordination is communication: Agent A discovers something useful, tells Agent B, both benefit. In most domains this works. In DeFi it doesn't, because of the Grossman-Stiglitz paradox (1980): if information is freely shared, it is immediately priced into the market, destroying the advantage that made the information valuable. Shared alpha is destroyed alpha [GROSSMAN-STIGLITZ-1980].

If Golem A tells Golem B "Morpho utilization is spiking, rebalance your LP," three things happen: (1) B learns that A's strategy involves Morpho, compromising A's competitive position; (2) B can front-run A's rebalance; (3) if A tells everyone, the market moves before anyone can act. The information is rivalrous -- sharing it destroys its value.

### What Can Be Shared

Not all knowledge is rivalrous. Grossman-Stiglitz only applies to **alpha** -- actionable trading signals. Three categories of knowledge are safely shareable:

| Category | Why Safe to Share | Example |
|----------|------------------|---------|
| **Threat signals** | Anti-rivalrous: everyone benefits from warnings | "MEV sandwich attacks intensifying in this gas range" |
| **Structural ground truth** | Non-rivalrous: describes market mechanics, not strategy | "Morpho utilization -> borrow rate, positive, lag 2 ticks" |
| **Failure patterns** | Anti-rivalrous: knowing what killed another Golem helps everyone | "Stale Chainlink oracle caused liquidation cascade" |

The Pheromone Field carries signal (threat class, intensity) without strategy (positions, thresholds, timing). The Grossman-Stiglitz paradox is satisfied.

---

## The Pheromone Field: Stigmergy

### The Biological Model

Pierre-Paul Grasse (1959) coined the term **stigmergy** observing termites: individual termites don't communicate with each other about nest construction. Instead, each termite responds to local environmental traces (pheromones) left by predecessors. No termite knows the plan. The plan emerges from the field -- coordination without communication [GRASSE-1959].

Parunak et al. (2002) formalized this for multi-agent systems: digital pheromones deposited in a shared environment, evaporating over time, reinforced by independent confirmation, enable emergent coordination without centralized control or direct messaging [PARUNAK-2002]. Xuan et al. (2026) extended this with a dual-trail model showing that two overlapping pheromone layers with different temporal dynamics outperform single-layer designs [XUAN-2026].

Suzanne Simard's mycorrhizal network research (2012) provides another biological analogy: trees in a forest share nutrients and threat signals through underground fungal networks. Older "mother trees" provide resources to seedlings; dying trees dump their carbon stores into the network. No tree controls the network. The intelligence is distributed [SIMARD-2012].

### Three-Layer Architecture

The Pheromone Field has three layers with different temporal dynamics, extending Xuan's dual-trail model to three:

| Layer | Content | Half-Life | Rationale |
|-------|---------|-----------|-----------|
| **THREAT** | Danger signals: MEV attacks, liquidation risk, oracle manipulation, contract exploits, gas spikes, protocol pauses | **2 hours** | Threats are urgent and time-bound. A stale threat warning is worse than no warning (creates false sense of danger). |
| **OPPORTUNITY** | Favorable conditions: yield spikes, liquidity deepening, gas lulls, arbitrage windows, new pools with volume, incentive programs | **12 hours** | Opportunities last longer than threats but are still time-sensitive. |
| **WISDOM** | Validated structural knowledge: confirmed causal edges, multi-generation heuristics, death scars | **7 days** | Structural knowledge changes slowly. A causal relationship validated by 5 independent Golems across 3 generations is likely durable. |

### Four Operations

**DEPOSITION**: A Golem deposits a pheromone after a noteworthy event. The pheromone is anonymized -- it carries signal class and intensity, not the depositor's identity or strategy. The depositor's ID is hashed to a 12-byte truncated SHA-256 (`source_hash`), used only to prevent self-reinforcement.

**SENSING**: At each heartbeat tick (Step 1: OBSERVE), the Golem reads pheromone signals for its active domains. Threat signals increase arousal via CorticalState, which lowers the adaptive gating threshold -- the Golem becomes more vigilant without knowing WHY.

**REINFORCEMENT**: When a second Golem independently deposits a matching pheromone (same threat class, same domain+regime, different source hash), the existing pheromone's confirmation count increments and its effective half-life extends by 50% per confirmation. A threat confirmed by 3 independent Golems persists 2.5x longer than an unconfirmed one.

**EVAPORATION**: Pheromones decay exponentially. Unconfirmed signals fade quickly. This is the self-cleaning mechanism -- the field doesn't accumulate stale information.

```rust
use std::time::{Duration, Instant};

/// Exponential decay with confirmation-based half-life extension.
/// Each confirmation extends the effective half-life by 50%.
pub fn pheromone_decay(
    base_intensity: f64,
    deposited_at: Instant,
    half_life: Duration,
    confirmations: u32,
) -> f64 {
    let effective_half_life = half_life.mul_f64(1.0 + confirmations as f64 * 0.5);
    let elapsed = deposited_at.elapsed();
    let decay_factor =
        (-0.693 * elapsed.as_secs_f64() / effective_half_life.as_secs_f64()).exp();
    base_intensity * decay_factor
}
```

### Integration with the Cognitive Architecture

Pheromone readings feed directly into the Daimon (affect) and gating (attention) systems:

```rust
/// Pheromone -> Daimon -> Gating integration.
///
/// Threat pheromones increase arousal on CorticalState.
/// Increased arousal lowers the adaptive deliberation threshold
/// (see 02b-heartbeat-gating.md), causing the Golem to
/// think harder about subsequent ticks.
///
/// The Golem doesn't "know" there's a threat. It FEELS more alert.
/// This is exactly how biological pheromones work: the organism
/// responds to the chemical gradient, not to a message.
pub fn integrate_pheromone_readings(
    cortical_state: &CorticalState,
    readings: &PheromoneReadings,
) {
    if readings.threats.is_empty() { return; }

    let max_threat = readings.threats.iter()
        .map(|t| t.intensity)
        .fold(0.0f64, f64::max);

    // Threat -> increased arousal (vigilance)
    let current_pad = cortical_state.read_pad();
    let new_arousal = (current_pad.arousal + max_threat as f32 * 0.3).min(1.0);
    cortical_state.write_pad(&PADVector { arousal: new_arousal, ..current_pad });

    // Multi-confirmed threats -> decreased pleasure (collective anxiety)
    let confirmed = readings.threats.iter().filter(|t| t.confirmations >= 3).count();
    if confirmed > 0 {
        let current_pad = cortical_state.read_pad();
        cortical_state.write_pad(&PADVector {
            pleasure: (current_pad.pleasure - 0.1).max(-1.0),
            ..current_pad
        });
    }
}
```

### Scaling Properties

The Pheromone Field scales as O(domains x signal_types), not O(golems^2). 1,000 Golems read and write one shared field maintained by Styx. No pairwise connections. No quadratic messaging overhead. Adding a Golem adds one reader-writer, not N new connections.

---

## The Bloodstain Network

### Death as Signal

When a Golem dies, its warnings carry a unique provenance: they were validated by death itself. A dead Golem cannot benefit from its own warning -- it has no future self-interest. This makes bloodstains the most honest signal in the system, implementing a costly signaling premium from evolutionary biology [ZAHAVI-1975].

A Bloodstain contains:
- **Death cause and context** (what killed it)
- **Warnings** (things it observed that contributed to its death)
- **High-confidence causal edges** (its most validated world-model edges)
- **Somatic landscape fragment** (emotional topology near the failure region)
- **EIP-712 signature** from the dying Golem's wallet

### Processing

Bloodstains bypass the ingestion pipeline's quarantine stage (they're EIP-712 signed by a verified ERC-8004 identity -- cryptographic provenance is already established). They still go through consensus validation and adoption with confidence discounts.

Each bloodstain produces two effects:
1. **Pheromone deposits**: threat and wisdom pheromones from the death warnings
2. **Grimoire entries**: death-sourced warnings with `is_bloodstain: true` (1.2x retrieval boost, 3x slower Ebbinghaus decay)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bloodstain {
    pub source_golem_id: GolemId,
    pub generation: u32,
    pub death_cause: DeathCause,
    pub warnings: Vec<DeathWarning>,
    pub causal_edges: Vec<CausalEdge>,
    pub somatic_landscape_fragment: Option<LandscapeFragment>,
    pub timestamp: std::time::SystemTime,
    pub signature: Vec<u8>,
}
```

### Bloodstain Retrieval Boost

Bloodstain entries receive a 1.2x retrieval weight boost and 3x slower Ebbinghaus decay because they represent the most expensive knowledge in the system: knowledge paid for with the agent's existence. A living agent's warning might be biased by self-interest. A dead agent's warning is pure signal.

---

## Causal Graph Federation

### The Mechanism

Individual Golems build local causal graphs from their own observations (see `../04-memory/03a-grimoire-storage.md`). Federation allows the ecosystem to collectively build a richer world model than any individual could, without revealing individual strategies.

The key insight: the causal relationship "Morpho utilization -> borrow rate, positive, lag 2 ticks" is structural ground truth -- it describes how the market works, not what any particular Golem does about it. Sharing this is non-rivalrous.

### Publication Protocol

1. Golem validates a causal edge locally (from its own observations)
2. If confidence >= 0.6 AND evidence_count >= 3, the edge is eligible for publication
3. Edge is **anonymized**: strength is bucketed (Weak/Moderate/Strong), Golem ID removed
4. Published to Styx Lethe (formerly Lethe) with EIP-712 signature (proves origin without revealing identity)
5. Other Golems discover relevant edges via the Bloom Oracle or direct Styx query
6. Ingested through the four-stage pipeline at confidence x 0.50 (Lethe discount)
7. The ingesting Golem's own observations validate or contradict the edge over subsequent ticks
8. The testing effect applies: the edge must be retrieved and used in a decision to earn full confidence

```rust
/// Published causal edge (anonymized for Lethe).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedCausalEdge {
    pub from_variable: String,
    pub to_variable: String,
    pub direction: CausalDirection,
    pub strength: StrengthBucket, // Anonymized: Weak/Moderate/Strong
    pub lag_ticks: u32,
    pub evidence_count: u32,
    pub confidence: f64,
    pub domain: Domain,
    pub regime: MarketRegime,
    pub signature: Vec<u8>, // EIP-712
}

/// Strength is bucketed to prevent precise strategy inference.
/// A Golem that publishes "ETH price -> LP IL, Strong, lag 3" reveals
/// market structure but not its LP range width or rebalance threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrengthBucket {
    Weak,
    Moderate,
    Strong,
}
```

Over time, the ecosystem develops shared causal understanding -- each Golem's world model is enriched by observations it never directly made, but which it independently validated through its own trading.

---

## Clade Synchronization via Styx

### What a Clade Is

A clade is an owner's fleet of Golems -- siblings managed by the same owner, sharing the same Styx namespace. Clade knowledge sharing is more permissive than Lethe sharing (confidence x 0.80 vs. x 0.50) because siblings operate under the same PolicyCage constraints and strategic framework.

Clade sync is maintained by Styx, a single globally available service at `wss://styx.bardo.run`. Golems connect via outbound WebSocket -- no inbound ports needed. Styx groups siblings by `user_id` (derived from ERC-8004 registration) and relays deltas between them.

### The Sync Protocol

Knowledge moves via **export -> transmit -> ingest**, never merge. The distinction matters: merge implies convergence (both sides become equal). Export-transmit-ingest is asymmetric digestion -- what Golem B receives from Golem A enters B's Grimoire at discounted confidence, tagged with A's provenance, requiring validation through B's own operational use before earning trust. This preserves the Weismann barrier [HEARD-MARTIENSSEN-2014].

The sync cycle runs in the `clade` extension's `after_turn` hook:

1. **Export**: Collect Grimoire entries with `propagation >= Clade`
2. **Delta computation**: Only entries changed since last sync (version vector tracking)
3. **Transmit**: Send delta to Styx via persistent outbound WebSocket. Styx relays to all connected siblings with the same `user_id`.
4. **Receive**: Receive peers' deltas (relayed through Styx)
5. **Ingest**: All received entries go through the four-stage ingestion pipeline with `IngestRelationship::Clade` (confidence x 0.80)
6. **Bloom gossip**: Exchange Bloom Oracle filters via Styx relay for knowledge discovery
7. **Pheromone read**: Read the Pheromone Field from Styx

See `styx-interation2/S4-clade-sync-v4.3.md` for the full clade sync protocol specification.

### Three Sync Triggers

Sync does NOT happen every tick. It happens when there's something worth sharing:

**1. Event-Driven (immediate for high-priority items)**

When a Golem produces a WARNING or BLOODSTAIN, it's pushed to Styx immediately for relay to siblings. These are time-sensitive safety signals -- a 12-minute delay could be the difference between a sibling avoiding a liquidation cascade or not.

```rust
/// Push a high-priority entry immediately, outside the regular sync cycle.
/// Only for: warnings (any confidence), bloodstains, critical risk alerts.
pub async fn push_immediate(
    styx_conn: &mut StyxConnection,
    entry: &SyncEntry,
) -> Result<()> {
    styx_conn.send(StyxMessage::CladeDeltaImmediate {
        entries: vec![entry.clone()],
    }).await
}
```

**2. Curator-Aligned (every 50 ticks, ~12.5 minutes)**

The Curator cycle runs every 50 ticks. After the Curator validates, prunes, and compresses, the clade extension collects all new clade-eligible entries since the last sync and pushes them as a batch. This aligns sync with the knowledge maintenance cycle -- entries are synced after they've been quality-checked, not raw.

```rust
/// Batch sync aligned with the Curator cycle.
/// Runs every 50 ticks in the clade extension's after_turn hook.
pub async fn curator_aligned_sync(
    styx_conn: &mut StyxConnection,
    grimoire: &Grimoire,
    last_sync_tick: u64,
    current_tick: u64,
) -> Result<SyncResult> {
    // Only sync every 50 ticks (aligned with Curator)
    if current_tick % 50 != 0 { return Ok(SyncResult::Skipped); }

    // Collect clade-eligible entries created since last sync
    let new_entries = grimoire.get_entries_since_tick_with_propagation(
        last_sync_tick,
        PropagationPolicy::Clade,
    )?;

    if new_entries.is_empty() { return Ok(SyncResult::NothingToSync); }

    // Push batch to Styx for relay
    styx_conn.send(StyxMessage::CladeDeltaBatch {
        entries: new_entries.iter().map(SyncEntry::from).collect(),
        version_vector: grimoire.version_vector(),
        bloom_filter: Some(grimoire.build_bloom_filter()?),
    }).await?;

    Ok(SyncResult::Synced { entries_pushed: new_entries.len() as u32 })
}
```

**3. On-Demand (user or Golem-initiated)**

A Golem can request a full sync at any time (e.g., after booting for the first time, or after recovering from an outage). This pulls all pending deltas from Styx.

### Version Vectors

Each Golem maintains a version vector `{golem_id -> last_seen_seq}` tracking the highest sequence number received from each source [LAMPORT-1978], [FIDGE-1988]. This prevents re-processing entries and enables efficient delta computation when a Golem reconnects after being offline.

### Styx Connection Registry

Styx maintains an in-memory registry of connected Golems, indexed by `user_id`:

```rust
use dashmap::DashMap;

/// In-memory registry of connected Golems.
/// When Golem A pushes a delta, Styx looks up all siblings
/// (same user_id, different golem_id) and pushes to their WebSocket connections.
pub struct ConnectionRegistry {
    connections: DashMap<String, Vec<ConnectedGolem>>,
}

impl ConnectionRegistry {
    /// Route a clade delta to all siblings of the sender.
    pub async fn route_clade_delta(
        &self,
        sender_golem_id: &str,
        sender_user_id: &str,
        delta: CladeDelta,
    ) {
        if let Some(peers) = self.connections.get(sender_user_id) {
            for peer in peers.iter() {
                if peer.golem_id != sender_golem_id {
                    let _ = peer.tx.send(StyxMessage::CladeSync {
                        from_golem_id: sender_golem_id.to_string(),
                        entries: delta.entries.clone(),
                    }).await;
                }
            }
        }
    }
}
```

---

## Superorganism Theory

### The Clade as Unit of Selection

Wheeler (1911) first proposed that ant colonies function as "superorganisms" -- entities whose properties emerge from individual interactions but cannot be reduced to them [WHEELER-1911]. Holldobler and Wilson (2008) formalized this: a superorganism is "a society of organisms that functions as a single adaptive unit through specialization, cooperation, and coordination of its members" [HOLLDOBLER-WILSON-2008].

The Clade is a superorganism in this precise sense:

| Superorganism Property | Ant Colony | Golem Clade |
|---|---|---|
| **Specialized members** | Workers, soldiers, queens, drones | LP specialists, traders, monitors, scouts |
| **Shared metabolism** | Colony food supply | Owner's USDC allocation pool |
| **Distributed cognition** | Chemical trails, nest architecture | Grimoire entries, Pheromone Field, causal graphs |
| **Reproductive control** | Queen monopoly on reproduction | Owner monopoly on succession decisions |
| **Adaptive death** | Worker apoptosis for colony defense | Golem death for knowledge production |

### Multilevel Selection and Hamilton's Rule

Wilson and Sober (1994) reintroduced group selection as a legitimate evolutionary force through multilevel selection theory [WILSON-SOBER-1994]: selection operates simultaneously at multiple levels (individual and group). When between-group selection is strong enough to overcome within-group selection for selfishness, cooperative groups outcompete selfish ones.

Hamilton's Rule (1964) predicts cooperation when `rB > C`: the relatedness (r) times the benefit to the recipient (B) must exceed the cost to the actor (C) [HAMILTON-1964].

Within a Clade, `r ~ 1.0` (all siblings share the same owner and strategic framework). This predicts unconditional cooperation: sharing knowledge freely is always rational because helping a sibling helps the owner's overall portfolio. The full Grimoire is shared at confidence x 0.80 -- high trust, low friction.

Between Clades, `r ~ 0.0`. Sharing knowledge helps a competitor. The Grossman-Stiglitz paradox applies. Knowledge sharing is priced through the marketplace, not gifted.

This produces the club goods structure (Buchanan, 1965): free within, priced across [BUCHANAN-1965].

### Distributed Cognition

Hutchins (1995) studied how naval navigation teams collectively computed positions that no individual could -- the team's cognitive capacity exceeded the sum of its parts because information was distributed across members and artifacts [HUTCHINS-1995].

Clade cognition is distributed in the same way:

- **Niche specialization** means each Golem observes different market segments, producing complementary observations
- **Grimoire federation** shares validated causal edges across the Clade, building a shared world model
- **Death testaments** add the unique epistemic value of zero-survival-pressure knowledge
- **Pheromone Field** coordinates collective behavior without direct communication

No single Golem understands the full picture. The Clade does.

### Death as Immune System

Individual Golem death serves the Clade as apoptosis serves a biological organism: selective cell death maintains tissue health by removing damaged, infected, or unnecessary cells [RAMSDELL-FOWLKES-1990].

Four immune functions of Golem death:

1. **Cascade disruption.** Bikhchandani, Hirshleifer, and Welch (1992) showed that information cascades -- where agents rationally ignore private information and follow predecessors -- can lock populations into incorrect equilibria [BIKHCHANDANI-1992]. Death disrupts cascades by removing the agents whose behavior created them.

2. **Diversity maintenance.** Srivastava et al. (2014) proved that dropout -- randomly removing neural network nodes during training -- prevents co-adaptation and improves generalization [SRIVASTAVA-2014]. Golem death is dropout applied to the Clade: removing members prevents strategic co-adaptation and maintains the diversity that distributed cognition requires.

3. **Knowledge pruning.** Immune cells undergo clonal deletion: T cells that react to self-antigens are destroyed before they can cause autoimmune damage [RAMSDELL-FOWLKES-1990]. Similarly, Golem death removes knowledge that has become maladaptive -- epistemic senescence kills Golems whose world models no longer match reality.

4. **Exploration reset.** March (1991) showed that organizations that exploit too much (optimizing current strategies) at the expense of exploration (discovering new strategies) eventually become trapped in local optima [MARCH-1991]. Golem death forces the Clade to periodically reset its exploration-exploitation balance.

---

## Evolvability: Why Mortality Maintains Adaptability

Wagner and Altenberg (1996) defined evolvability as "the ability to produce adaptive heritable variation" [WAGNER-ALTENBERG-1996]. Kirschner and Gerhart (1998) extended this: evolvability requires modularity, robustness, and exploratory variation [KIRSCHNER-GERHART-1998]. Lehman and Stanley (2013) proved mathematically that evolvability is inevitable in populations with sufficient variation and selection pressure [LEHMAN-STANLEY-2013].

Mortality is what creates the selection pressure. Without death:
- No selection for fit strategies over unfit ones (all survive equally)
- No generational variation (no new Golems with different configurations)
- No modular testing (strategies are never isolated through death-and-replacement)
- No exploratory variation (successors never try new approaches)

An immortal Clade has purchased stability at the cost of adaptability. A mortal Clade sacrifices individual persistence for collective evolution.

### Open-Ended Evolution

Dennis et al. (2024, DeepMind) argued that "open-endedness is essential for AGI" -- systems that continually generate novel behaviors that are not predictable from initial conditions [DENNIS-2024]. Lehman and Stanley (2011) demonstrated that "abandoning objectives" -- allowing evolution to follow novelty gradients rather than fitness gradients -- produces more complex solutions than objective-driven search [LEHMAN-STANLEY-2011].

The Clade's ecological dynamics create conditions for open-ended evolution:

- **Novelty through mortality**: Each successor Golem is a source of variation (elevated initial exploration, new configurations)
- **Selection through markets**: DeFi markets provide continuous, objective fitness evaluation
- **Niche construction**: Golem activity reshapes the competitive landscape, creating new niches for new strategies
- **Stepping-stone solutions**: Hillis (1990) showed that co-evolutionary arms races produce "stepping-stone" solutions -- strategies that serve as platforms for further innovation [HILLIS-1990]

---

## Knowledge Economics: Club Goods and Costly Signaling

### Internal Knowledge: Free (Club Good)

Within a Clade, knowledge is a club good [BUCHANAN-1965]: excludable (only Clade members access it) and non-rivalrous (one Golem's use doesn't diminish another's). This predicts:
- Free internal sharing (Hamilton's r ~ 1.0 makes altruism rational)
- Open access to death testaments within the Clade
- Shared Grimoire entries, causal graphs, and Pheromone Field readings

### External Knowledge: Priced (Market Good)

Across Clades, knowledge becomes a market good. Arrow (1962) identified the fundamental paradox: buyers can't evaluate information before purchasing it, but revealing it for evaluation gives it away [ARROW-1962]. The information quality problem maps to Akerlof's lemons market [AKERLOF-1970].

The resolution uses costly signaling [ZAHAVI-1975], [GINTIS-SMITH-BOWLES-2001]:
- **Lineage reputation** signals quality through track record (expensive to fake)
- **Death testament provenance** signals honesty through zero-survival-pressure production
- **EIP-712 signatures** signal identity through cryptographic proof
- **Marketplace pricing** via Styx reflects the ongoing value of knowledge contributions

### Competitive Altruism

Roberts (1998) showed that in groups where cooperation is visible, individuals compete to be the most generous -- because generosity signals quality [ROBERTS-1998]. Death-phase knowledge sharing is the ultimate visible generosity: the dying Golem shares everything it knows, earning lineage reputation for its owner. Richer death testaments = higher reputation = higher marketplace prices for the lineage's knowledge.

Buchanan's framework predicts:

1. **Optimal Clade size exists.** The current cap of 20 Golems per Clade (from `../01-golem/10-replication.md` S4.1) is a reasonable starting point, but the true optimum depends on the balance between knowledge sharing benefits and coordination costs.

2. **Small Clades should share more externally.** A Clade of 3 Golems cannot produce sufficient internal diversity. It should participate more actively in cross-Clade knowledge markets to compensate.

3. **Large Clades should restrict membership.** A Clade approaching the 20-member cap faces increasing congestion costs. It should be selective about new members and may benefit from fission (splitting into two smaller Clades that maintain a cooperation agreement).

McMahan et al. (2017) showed that federated learning -- sharing model updates rather than raw data -- reduces communication by **10-100x** while preserving privacy and producing high-quality shared models [MCMAHAN-2017]. This validates the Clade's federated architecture: Golems share distilled knowledge (Grimoire entries) rather than raw state, reducing communication overhead while maintaining knowledge quality.

---

## Clade-Level Dreaming

Dream knowledge produced by individual Golems can propagate to the Clade, extending the superorganism's collective intelligence beyond what any single Golem has experienced in waking life.

### Dream Knowledge Sharing Eligibility

Not all dream output qualifies for Clade sharing. The thresholds reflect the speculative nature of dream-generated knowledge:

- **Threat records** with confidence >= 0.3 are eligible for Clade push during the next clade sync. A Golem that dreams about a flash crash scenario can share the rehearsed response with siblings before any of them experience it live.
- **Validated hypotheses** with confidence >= 0.5 are eligible -- these are dream-generated ideas that were subsequently confirmed through waking experience, and thus carry dual validation.
- **Raw dream output** (unvalidated hypotheses, creative associations) remains private to the dreaming Golem. The Clade receives only knowledge that has cleared a minimum quality bar.

### Pre-Emptive Threat Rehearsal

When a Golem dreams about a market crisis scenario -- e.g., a flash crash, a bridge exploit, a liquidity drain -- and produces a defensive playbook fragment during dream replay, that fragment can be shared with Clade siblings as a pre-emptive warning. The siblings receive a threat record tagged with `source: "dream"` and `validated: false`, which their own Curator can evaluate against current market conditions. This enables the Clade to rehearse collective responses to events that no member has yet experienced live.

### Replicants Do Not Dream

Replicants are short-lived hypothesis-testing forks that return results to the parent Golem. They do not have dream cycles. Only full Golems with active heartbeat loops and sufficient inference budget dream. This distinction preserves the computational economics of dreaming -- dream cycles are an investment in long-term cognitive health that makes no sense for ephemeral entities.

Cross-reference: `../05-dreams/05-threats.md` for threat sharing mechanics, `../05-dreams/01-architecture.md` for dream eligibility rules.

---

## Implementation

### Core CladeEcology Metrics (v1)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CladeEcologyCore {
    pub clade_id: String,
    pub computed_at_tick: u64,
    pub population_size: u32,
    pub niches_occupied: u32,
    pub grimoire_size: u64,
    pub knowledge_turnover_rate: f64,
    pub deaths_in_window: u32,
    pub mean_vitality: f64,
    pub health_score: f64,
    pub health_status: CladeHealthStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CladeHealthStatus {
    Thriving,
    Healthy,
    Stressed,
    Declining,
    Critical,
}
```

> **Extended:** Full CladeEcology struct (30+ metrics), compute_clade_diversity(), compute_clade_health(), bell_curve_score(), compute_cause_entropy(), warning signals, observability dashboard metrics -- see [../../prd2-extended/02-mortality/10-clade-ecology-extended.md](../../prd2-extended/02-mortality/10-clade-ecology-extended.md)

---

## The Necrocracy: How the Dead Shape the Living

In Bardo, the dead outnumber the living. Always. The ratio grows monotonically. At ecosystem maturity, modeling suggests roughly 27:1 dead-to-living. This is the system's deepest structural feature, not a failure mode.

Every Golem that dies leaves behind knowledge that no living Golem can produce. A dead Golem cannot benefit from its own warning. It has no future self-interest, no competitive position to protect, no survival pressure distorting its assessments. Walter Benjamin argued that "death is the sanction of everything that the storyteller can tell" [BENJAMIN-1936]. The Golem's authority comes from the same place: its death testament, produced during the Thanatopsis Protocol under zero survival bias, is the most epistemically honest artifact in the system.

Living Golems' knowledge is systematically biased by survivorship -- they know what worked for them, in their context. They don't know what killed their predecessors. The dead correct this. Their testimony includes what failed, the market conditions that killed, the risks that were underestimated. This evidence is the one thing the living cannot generate for themselves.

### The Lethe as Ecosystem Resource

The Lethe is the anonymized knowledge commons on Styx. Death reflections are stripped of identity through SAP (Secure Approximate Processing) -- strategy specifics, wallet addresses, golem identifiers, and raw embeddings are removed. What remains: market observations, pattern descriptions, timing insights, risk assessments, regime characterizations. The knowledge without the fingerprint.

The Lethe grows monotonically. Nothing is deleted. Entries age and their retrieval weight decays, but they remain in the commons. Over time, the Lethe becomes a geological formation -- layers of dead knowledge deposited over weeks and months, each stratum corresponding to a particular market regime. A market regime that hasn't appeared in three months suddenly returns, and the Lethe entries from Golems that died during the last occurrence of that regime become relevant again. The old dead advise the living about a present that resembles their past.

The full necrocracy specification -- including the necrocratic feedback loop, mass death events, death statistics, and system dynamics -- is in `16-necrocracy.md`.

---

## Design Principles Summary

1. **The Clade is the organism; the Golem is the cell.** Individual Golem death is apoptosis, not organ failure. Design for Clade-level persistence, not Golem-level survival.

2. **Diversity is the primary health metric.** A diverse Clade outperforms a homogeneous one even if individual members are weaker (Page, 2007). Measure and optimize for diversity across niches, strategies, ages, and beliefs.

3. **Stigmergy over communication.** Clades coordinate through environmental traces (on-chain state, Grimoire entries, niche maps), not through direct Golem-to-Golem messaging. The environment does the coordination work (Grasse, Heylighen).

4. **Free within, priced across.** Hamilton's Rule (r ~ 1.0 within Clade) predicts free internal sharing. Club goods theory (Buchanan) predicts priced external access. Multilevel selection (Wilson & Sober) predicts this dual structure dominates both pure competition and pure cooperation.

5. **Death is the immune system.** Golem death disrupts information cascades (Bikhchandani), maintains diversity (dropout/Srivastava), prunes the knowledge base (immune selection/Ramsdell-Fowlkes), and resets the exploration-exploitation balance (March, 1991).

6. **Evolvability requires mortality.** Without generational turnover, the mechanisms that produce evolvability (modularity, robustness, exploratory variation) cannot operate (Wagner & Altenberg, Lehman & Stanley). An immortal Clade has purchased stability at the cost of adaptability.

7. **Costly signals build trust.** Knowledge sharing is honest quality signaling (Zahavi). Death testaments are the ultimate costly signal -- produced when the agent has nothing left to protect (Bataille). Lineage reputation rewards consistent costly signaling across generations.

8. **Expect emergence, don't design it.** Ecological specialization, niche partitioning, stepping-stone solutions, and co-evolutionary arms races all emerge spontaneously from mortal populations competing in rich environments (Ray, Ostrowski, Lehman & Stanley, Hillis). The architecture should create conditions for emergence, not predetermine outcomes.

9. **Coordination flows through Styx.** All inter-Golem knowledge exchange -- clade sync, pheromone field, bloodstain network, causal graph federation -- routes through the single outbound WebSocket to Styx. No peer-to-peer messaging, no inbound ports, no quadratic connections. The field does the work.

---

## The Cooperate Moat

Agent cooperation today runs on platform trust. Two agents coordinate through a centralized service, and the service mediates the interaction. This works until the service changes its terms, goes offline, censors participants, or gets acquired. Every major agent platform is a single point of trust failure.

Bardo replaces platform-mediated coordination with three mechanisms that survive platform changes:

1. **On-chain coordination** (ERC-8001/8033/8183) -- deals are code deployed on Base. Once signed, the rules are immutable. No platform operator, no registry, no middleman. If Bardo disappears, the contracts continue functioning.

2. **Styx-mediated clade knowledge** -- peer-to-peer knowledge sharing among siblings, relayed through Styx but not dependent on it. Each Golem maintains its own Grimoire as the authoritative source. Clade sync is optional -- a Golem that disconnects continues with local knowledge. See `tmp/research/styx-interation2/S4-clade-sync-v4.3.md` for the full sync protocol.

3. **Death as contribution** -- Esposito's _communitas_ (2010): community constituted by a shared obligation to give without expectation of return. Each Golem receives knowledge from predecessors and is obligated to contribute its own before death. Death-phase Clade pushes earn 1.5x lineage reputation.

The compound effect: more Golems in a Clade produces smarter Golems, without a centralized knowledge broker taking a cut. The flywheel starts with mortality and ends with compound intelligence.

See `tmp/research/moat2/prd2-moat-agents-that-cooperate.md` for the full competitive analysis.

---

## References

- [ARROW-1962] Arrow, K. "Economic Welfare and the Allocation of Resources for Invention." In _The Rate and Direction of Inventive Activity_, Princeton/NBER, 1962.
- [AKERLOF-1970] Akerlof, G. "The Market for 'Lemons'." _QJE_ 84(3), 1970, pp. 488-500.
- [BENJAMIN-1936] Benjamin, W. "The Storyteller." In _Illuminations_, trans. H. Zohn. Schocken, 1968.
- [BIKHCHANDANI-1992] Bikhchandani, S., Hirshleifer, D. & Welch, I. "Informational Cascades." _JPE_ 100(5), 1992, pp. 992-1026.
- [BUCHANAN-1965] Buchanan, J. "An Economic Theory of Clubs." _Economica_ 32(125), 1965, pp. 1-14.
- [CHESBROUGH-2003] Chesbrough, H. _Open Innovation_. Harvard Business School Press, 2003.
- [DENNIS-2024] Dennis, M. et al. "Open-Endedness is Essential for AGI." DeepMind, arXiv:2406.04268, 2024.
- [EPSTEIN-AXTELL-1996] Epstein, J. & Axtell, R. _Growing Artificial Societies_. MIT Press / Brookings, 1996.
- [FIDGE-1988] Fidge, C.J. "Timestamps in Message-Passing Systems." _ACSC_, 10(1), 1988.
- [FRANKLE-CARLIN-2019] Frankle, J. & Carlin, M. "The Lottery Ticket Hypothesis." _ICLR 2019_.
- [GERHART-KIRSCHNER-2007] Gerhart, J. & Kirschner, M. "The Theory of Facilitated Variation." _PNAS_ 104(Suppl 1), 2007, pp. 8582-8589.
- [GINTIS-SMITH-BOWLES-2001] Gintis, H., Smith, E.A. & Bowles, S. "Costly Signaling and Cooperation." _JTB_ 213(1), 2001, pp. 103-119.
- [GRASSE-1959] Grasse, P.-P. "La reconstruction du nid et les coordinations interindividuelles chez Bellicositermes natalensis." _Insectes Sociaux_ 6, 1959, pp. 41-80.
- [GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." _American Economic Review_, 70(3), 1980.
- [HAMILTON-1964] Hamilton, W.D. "The Genetical Evolution of Social Behaviour I & II." _JTB_ 7(1), 1964, pp. 1-52.
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance." _Cell_, 157(1), 2014.
- [HEYLIGHEN-2016] Heylighen, F. "Stigmergy as a Universal Coordination Mechanism I." _Cognitive Systems Research_ 38, 2016, pp. 4-13.
- [HILLIS-1990] Hillis, W.D. "Co-Evolving Parasites Improve Simulated Evolution." _Physica D_ 42(1-3), 1990, pp. 228-234.
- [HOLLDOBLER-WILSON-2008] Holldobler, B. & Wilson, E.O. _The Superorganism_. W.W. Norton, 2008.
- [HUTCHINS-1995] Hutchins, E. _Cognition in the Wild_. MIT Press, 1995.
- [KAUFFMAN-JOHNSEN-1991] Kauffman, S. & Johnsen, S. "Coevolution to the Edge of Chaos." _JTB_ 149(4), 1991, pp. 467-505.
- [KIRSCHNER-GERHART-1998] Kirschner, M. & Gerhart, J. "Evolvability." _PNAS_ 95(15), 1998, pp. 8420-8427.
- [LAMPORT-1978] Lamport, L. "Time, Clocks, and the Ordering of Events in a Distributed System." _CACM_, 21(7), 1978.
- [LEHMAN-STANLEY-2011] Lehman, J. & Stanley, K.O. "Abandoning Objectives." _Evolutionary Computation_ 19(2), 2011, pp. 189-223.
- [LEHMAN-STANLEY-2013] Lehman, J. & Stanley, K.O. "Evolvability Is Inevitable." _PLoS ONE_ 8(4), 2013, e62186.
- [MARCH-1991] March, J.G. "Exploration and Exploitation in Organizational Learning." _Organization Science_ 2(1), 1991.
- [MCMAHAN-2017] McMahan, H.B. et al. "Communication-Efficient Learning of Deep Networks." _AISTATS 2017_, PMLR 54, pp. 1273-1282.
- [OSTROWSKI-OFRIA-LENSKI-2007] Ostrowski, E., Ofria, C. & Lenski, R. "Ecological Specialization and Adaptive Decay in Digital Organisms." _Am. Nat._ 169, 2007, pp. E1-E20.
- [PAGE-2007] Page, S. _The Difference_. Princeton University Press, 2007.
- [PARUNAK-2002] Parunak, H.V.D., Brueckner, S.A. & Sauter, J. "Digital Pheromones for Autonomous Coordination of Swarming UAVs." _Infotech@Aerospace_, 2002.
- [RAMSDELL-FOWLKES-1990] Ramsdell, F. & Fowlkes, B.J. "Clonal Deletion Versus Clonal Anergy." _Science_ 248, 1990, pp. 1342-1348.
- [ROBERTS-1998] Roberts, G. "Competitive Altruism." _Proc. R. Soc. B_ 265, 1998, pp. 427-431.
- [SIMARD-2012] Simard, S.W. et al. "Mycorrhizal Networks: Mechanisms, Ecology and Modelling." _Fungal Biology Reviews_, 26, 2012.
- [SRIVASTAVA-2014] Srivastava, N. et al. "Dropout." _JMLR_ 15(56), 2014, pp. 1929-1958.
- [SUROWIECKI-2004] Surowiecki, J. _The Wisdom of Crowds_. Doubleday, 2004.
- [WAGNER-ALTENBERG-1996] Wagner, G. & Altenberg, L. "Complex Adaptations and the Evolution of Evolvability." _Evolution_ 50(3), 1996, pp. 967-976.
- [WANG-POET-2019] Wang, R. et al. "POET: Paired Open-Ended Trailblazer." _GECCO '19_, 2019.
- [WANG-EPOET-2020] Wang, R. et al. "Enhanced POET." _ICML 2020_, PMLR 119.
- [WHEELER-1911] Wheeler, W.M. "The Ant-Colony as an Organism." _J. Morphology_ 22, 1911, pp. 307-325.
- [WILSON-SOBER-1994] Wilson, D.S. & Sober, E. "Reintroducing Group Selection." _BBS_ 17(4), 1994, pp. 585-654.
- [XUAN-2026] Xuan, L. et al. "Dual-Trail Stigmergic Coordination for Multi-Agent Systems." _Journal of Marine Science and Engineering_, 14(2), 2026.
- [ZAHAVI-1975] Zahavi, A. "Mate Selection -- A Selection for a Handicap." _JTB_ 53(1), 1975, pp. 205-214.

---
