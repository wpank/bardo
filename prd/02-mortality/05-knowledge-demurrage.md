# Knowledge Demurrage: Forgetting as Feature [SPEC]

> **Version**: 4.0 | **Status**: Draft
>
> **Crates**: `golem-core`, `golem-grimoire`
>
> **Depends on**: `00-thesis.md` (foundational mortality thesis), `02-epistemic-decay.md` (epistemic clock specification), `06-thanatopsis.md` (four-phase death protocol with emotional life review), `../04-memory/01-grimoire.md` (persistent knowledge base specification)

---

> **Reader orientation:** This document specifies how Bardo Golems (mortal autonomous DeFi agents) actively forget. Knowledge entries in the Grimoire (the agent's persistent knowledge base) lose confidence over time unless revalidated, following Ebbinghaus decay curves. This is not a bug; it is the mechanism that keeps the Golem's working knowledge current and prevents context window bloat. The economic parallel is Gesell's Freigeld (depreciating currency that circulates faster). See `02-epistemic-decay.md` (epistemic clock that kills Golems when predictions go stale) for how decay feeds into mortality. See `prd2/shared/glossary.md` for full term definitions.

## Theoretical Foundations (S1--S4)

> **Extended:** Neuroscience of forgetting (Richards & Frankland, Davis & Zhong, Luria/Borges pathology of perfect recall), genomic bottleneck principle (Weismann barrier, Baldwin effect), Gesell's Freigeld and the Worgl experiment, EIP-1559 deflationary knowledge analogy -- see [../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md](../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md)

Forgetting is memory's primary function, not its failure mode. Active forgetting (Davis & Zhong, 2017) is as metabolically expensive as remembering. Perfect recall is pathological (Luria's Patient S., Borges's Funes). Knowledge demurrage applies Gesell's Freigeld principle to information: entries lose confidence over time unless actively validated, forcing circulation or expiration. The knowledge-burning mechanism parallels EIP-1559's fee burn.

The decay function follows the Ebbinghaus forgetting curve (1885): `retention = exp(-t / half_life)`, where `t` is ticks since last access. Different entry types have different base decay rates, reflecting their temporal characteristics: episodes decay at 0.001/tick (raw observations are time-bound), insights at 0.002/tick (distilled patterns are moderately durable), heuristics at 0.0005/tick (validated rules persist longer), warnings at 0.003/tick (danger signals are time-sensitive), and strategy fragments at 0.001/tick. Bloodstain entries (death-sourced knowledge) decay at 1/3 the normal rate -- the costly signaling premium extends to decay resistance. The testing effect (Roediger & Karpicke, 2006) provides the counterforce: every retrieval resets the `last_accessed_at` field, slowing decay. Knowledge that is never retrieved decays to the archive threshold and is pruned by the Curator. See `tmp/research/rewrite4/03a-grimoire-storage.md` for the complete decay specification.

---

## S5 -- DemurrageConfig and Implementation

### 5.1 Configuration Interface

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemurrageConfig {
    /// Ticks between validation checks. Default: 250 (~2.9 hours at 40s/tick)
    pub validation_interval: u64,
    /// Base confidence loss per missed validation interval. Default: 0.03 (3%)
    pub base_decay_per_interval: f64,
    /// Minimum confidence before entry is archived (removed from active context). Default: 0.1
    pub archive_threshold: f64,
    /// Minimum confidence before entry is permanently burned. Default: 0.02
    pub burn_threshold: f64,
    /// Domain-specific decay multipliers. Higher = faster decay.
    pub domain_multipliers: HashMap<KnowledgeDomain, f64>,
    /// Knowledge type weights (higher = more resistant to decay).
    pub type_weights: HashMap<KnowledgeType, f64>,
    /// Maximum entries in active Grimoire before capacity pressure triggers. Default: 500
    pub active_capacity: usize,
    /// Base rent per interval when capacity > 80%. Default: 0.001 USDC
    pub base_rent_usdc: f64,
    /// Enable exponential rent scaling. Default: true
    pub exponential_rent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeDomain {
    GasMev,          // Gas prices, MEV patterns
    PriceDirection,  // Price trend predictions
    Volatility,      // Volatility regime assessment
    Yield,           // Yield farming strategy
    Protocol,        // Protocol behavior, smart contract mechanics
    Governance,      // DAO governance, parameter changes
    MarketStructure, // Orderbook depth, liquidity distribution
    CrossChain,      // Bridge behavior, cross-chain arbitrage
    General,         // Domain-agnostic knowledge
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeType {
    Episode,          // Specific event record
    Insight,          // Generalized observation
    Heuristic,        // Decision rule
    Warning,          // Cautionary pattern
    CausalLink,       // Cause-effect relationship
    StrategyFragment, // Partial strategy observation
    Question,         // Explicit knowledge gap
    DeathTestament,   // Dying Golem's final reflection
}
```

### 5.2 Differential Decay by Knowledge Type

Not all knowledge decays at the same rate. Arbesman (2012) provides empirically measured half-lives: surgical medicine ~45 years, physics ~13 years, engineering 3--5 years, technology/software under 2 years [ARBESMAN-2012]. Applied to DeFi agent knowledge:

| Domain             | Typical Half-Life | Decay Multiplier | Rationale                                                                                             |
| ------------------ | ----------------- | ---------------- | ----------------------------------------------------------------------------------------------------- |
| `gas_mev`          | Hours             | 3.0x             | Extremely volatile. Gas patterns change with every block. MEV strategies have minutes-long viability. |
| `price_direction`  | Days              | 1.5x             | Regime-dependent, moderate stability. Trends persist for days to weeks.                               |
| `volatility`       | Weeks             | 0.8x             | Structural volatility regimes shift gradually. Breakouts are sudden but infrequent.                   |
| `yield`            | Weeks--Months     | 0.5x             | Protocol yield parameters change via governance. Underlying mechanics stable for weeks.               |
| `protocol`         | Months            | 0.3x             | Smart contract logic is immutable until upgrade. Protocol behavior highly predictable.                |
| `governance`       | Months            | 0.4x             | Governance proposals take weeks. Parameter changes are slow and announced.                            |
| `market_structure` | Weeks             | 0.7x             | Liquidity migrates gradually. Orderbook depth changes with market cycles.                             |
| `cross_chain`      | Weeks             | 0.6x             | Bridge behavior relatively stable. Cross-chain arb windows shift with volume.                         |
| `general`          | Variable          | 1.0x             | Default rate for unclassified knowledge.                                                              |

| Knowledge Type      | Type Weight | Rationale                                                                              |
| ------------------- | ----------- | -------------------------------------------------------------------------------------- |
| `episode`           | 0.5         | Raw events decay fastest. Specific observations lose relevance quickly.                |
| `insight`           | 1.0         | Generalized observations have standard persistence.                                    |
| `heuristic`         | 1.2         | Decision rules are more durable -- they encode abstracted patterns.                    |
| `warning`           | 1.5         | Cautionary knowledge is more valuable precisely because it's rare.                     |
| `causal_link`       | 1.8         | Cause-effect relationships are structural and slow to change.                          |
| `strategy_fragment` | 0.7         | Partial observations are fragile -- they haven't been validated.                       |
| `question`          | 2.0         | Open questions don't decay -- they persist until answered or superseded.               |
| `death_testament`   | 3.0         | Death testaments are produced under zero survival pressure. Highest epistemic honesty. |

### 5.2.1 Dream-Validated Entries

Dream-validated entries -- knowledge that was hypothesized during dreaming and subsequently confirmed through waking experience -- decay at 0.5x the standard rate for their decay class. The dual validation (dream hypothesis + waking confirmation) indicates robust knowledge that has survived both offline reasoning and live market testing. See `../05-dreams/04-consolidation.md`.

### 5.2.2 Three-Clock Knowledge Expiry

Styx Lethe (formerly Commons) (see `../20-styx/00-architecture.md`) implements three-clock knowledge expiry that mirrors the mortality architecture. Each clock triggers different knowledge management responses:

| Clock          | Trigger                            | Styx Response                                                                                                                                                  |
| -------------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Economic**   | Credits entering critical range    | Archive dormant entries (entries not retrieved in >500 ticks) to reduce storage costs. Curator cycles suspended to save LLM budget for Death Protocol.         |
| **Epistemic**  | Fitness below senescence threshold | Emergency Curator activation: aggressive pruning of entries in failing domains. Entries whose domain fitness < 0.3 are fast-tracked for demurrage.             |
| **Stochastic** | hayflickRatio > 0.85               | Clade push acceleration: all entries above 0.1 confidence pushed to Styx Lethe at Clade level. Legacy preparation -- entries tagged for death testament inclusion. |

This three-clock expiry ensures knowledge management aligns with the Golem's overall mortality trajectory. A Golem facing economic death prioritizes cost reduction; one facing epistemic death prioritizes quality; one approaching stochastic death prioritizes distribution.

> **Cross-ref**: `../20-styx/00-architecture.md` (mortal scoring function, three-clock expiry)

### 5.3 Decay Function

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemurrageEntryStatus {
    Active,
    Archived,
    Burned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrimoireEntry {
    pub id: String,
    pub content: String,
    pub confidence: f64,
    pub domain: KnowledgeDomain,
    pub entry_type: KnowledgeType,
    pub last_validated_tick: u64,
    pub created_tick: u64,
    /// 0 = directly learned, N = inherited N generations back.
    pub generation: u32,
    pub emotional_tag: Option<String>,
    pub status: DemurrageEntryStatus,
}

pub fn apply_demurrage(
    entry: &GrimoireEntry,
    config: &DemurrageConfig,
    current_tick: u64,
) -> GrimoireEntry {
    if entry.status != DemurrageEntryStatus::Active {
        return entry.clone();
    }

    let ticks_since_validation = current_tick.saturating_sub(entry.last_validated_tick);
    let intervals = ticks_since_validation / config.validation_interval;

    if intervals == 0 {
        return entry.clone();
    }

    // Effective decay = base * domain multiplier / type weight
    let domain_multiplier = config.domain_multipliers
        .get(&entry.domain)
        .copied()
        .unwrap_or(1.0);
    let type_weight = config.type_weights
        .get(&entry.entry_type)
        .copied()
        .unwrap_or(1.0);
    let effective_decay = (config.base_decay_per_interval * domain_multiplier) / type_weight;

    // Apply decay for each missed interval
    let total_decay = effective_decay * intervals as f64;
    let new_confidence = (entry.confidence - total_decay).max(0.0);

    // Determine new status
    let new_status = if new_confidence < config.burn_threshold {
        DemurrageEntryStatus::Burned
    } else if new_confidence < config.archive_threshold {
        DemurrageEntryStatus::Archived
    } else {
        DemurrageEntryStatus::Active
    };

    GrimoireEntry {
        confidence: if new_status == DemurrageEntryStatus::Burned { 0.0 } else { new_confidence },
        status: new_status,
        ..entry.clone()
    }
}

pub struct DemurrageResult {
    pub entries: Vec<GrimoireEntry>,
    pub archived: Vec<GrimoireEntry>,
    pub burned: Vec<GrimoireEntry>,
}

pub fn apply_demurrage_to_grimoire(
    entries: &[GrimoireEntry],
    config: &DemurrageConfig,
    current_tick: u64,
) -> DemurrageResult {
    let mut updated = Vec::new();
    let mut archived = Vec::new();
    let mut burned = Vec::new();

    for entry in entries {
        let result = apply_demurrage(entry, config, current_tick);
        match result.status {
            DemurrageEntryStatus::Burned => burned.push(result),
            DemurrageEntryStatus::Archived => archived.push(result),
            DemurrageEntryStatus::Active => updated.push(result),
        }
    }

    DemurrageResult { entries: updated, archived, burned }
}
```

### 5.4 Decay Timeline Example

> **Extended:** Full decay timeline tables (price_direction insight, causal_link warning) -- see [../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md](../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md)

Tactical knowledge (`gas_mev`, 3.0x multiplier) decays within hours. Structural knowledge (`protocol`, 0.3x multiplier, `causal_link` type weight 1.8) persists for weeks. This differential implements Arbesman's half-life framework.

---

## S6 -- Memory Rent System

> **Extended:** Exponential capacity-based pricing, rent payment mechanisms, applyRentPenalty — see [../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md](../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md)
>
> v1: lowest-confidence entry archived when Grimoire reaches capacity. No rent computation.

---

## S7 -- Knowledge Burning

The knowledge-burning mechanism is the destructive complement to demurrage. While demurrage causes gradual decay, burning permanently removes entries from the system -- analogous to EIP-1559's base fee burn.

### 7.1 When Entries Are Burned

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BurnReason {
    ConfidenceZero,    // Decayed below burn threshold
    Contradicted,      // Fresh evidence directly contradicts the entry
    Superseded,        // A newer, higher-confidence entry covers the same ground
    CapacityEviction,  // Capacity pressure forced removal
    GolemDeath,        // Golem died; entries not selected for death testament are burned
    GovernanceSunset,  // Clade voted to retire the entry
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnEvent {
    pub entry_id: String,
    pub reason: BurnReason,
    pub tick: u64,
    pub final_confidence: f64,
    /// Compressed summary preserved in cold storage for death testament.
    pub compression_summary: Option<String>,
}
```

### 7.2 Burn vs. Archive

| Outcome     | What Happens                                          | Reversible?                           | Accessible To                              |
| ----------- | ----------------------------------------------------- | ------------------------------------- | ------------------------------------------ |
| **Archive** | Removed from active context. Stored in cold Grimoire. | Yes (can be restored if re-validated) | Death testament, life review, Styx Archive backup |
| **Burn**    | Compressed to one-line summary. Original deleted.     | No (summary only, original gone)      | Death testament summary section only       |

Burning is permanent. The entry's content is lossy-compressed to a one-line summary (sufficient for death testament context) and the original is deleted. This is not a storage optimization -- it is a philosophical commitment. Some knowledge should die. Knowledge that has been contradicted, superseded, or abandoned serves no purpose and consumes resources (even cold storage has retrieval cost). Burning it is an act of hygiene.

### 7.3 Active vs. Passive Forgetting

The system implements both active and passive forgetting mechanisms:

**Passive forgetting** (demurrage):

- Entries decay automatically over time
- No action required from the Golem
- Rate determined by domain and type
- Mimics biological memory trace degradation (Davis & Zhong pathway)

**Active forgetting** (deliberate burning):

- Golem explicitly marks entries for removal during Curator cycle
- Triggered by contradictory evidence or supersession
- Requires inference (at least T1/Haiku to evaluate)
- Mimics Anderson & Green's executive suppression mechanism
- Produces a `BurnEvent` for telemetry

The combination ensures that the Grimoire is subject to both entropy-like background decay (passive) and intelligent pruning (active). Neither alone is sufficient: passive forgetting cannot handle contradictions (a wrong entry with high confidence decays slowly), and active forgetting requires inference budget that may be unavailable in conservation phase.

---

## S8 -- Ostrom's Commons Governance for Knowledge Pools

> **Extended:** Ostrom's eight principles mapped to knowledge commons, knowledge tragedy taxonomy, CladeKnowledgeGovernance interface -- see [../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md](../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md)

Shared Clade knowledge is a common-pool resource governed by Ostrom's principles [OSTROM-1990]: defined boundaries (ERC-8004 membership), domain-congruent decay rates, collective sunset votes, citation monitoring, graduated sanctions, and nested enterprises (private Grimoire -> Clade pool -> Styx Lethe marketplace).

---

## S9 -- Two-Loop Architecture

The knowledge demurrage system operates through two distinct loops that mirror the biological separation of somatic and germ-line information.

### 9.1 Inner Loop: Local Grimoire

The inner loop is each Golem's private Grimoire -- its personal knowledge base, subject to individual demurrage and capacity constraints.

```
Every tick:
  1. Probes observe market state
  2. Episodes recorded to Grimoire
  3. Every 50 ticks: Curator cycle
     - Validate active entries against recent episodes
     - Apply demurrage to unvalidated entries
     - Archive entries below threshold
     - Burn entries below burn threshold
     - Promote high-confidence entries to PLAYBOOK.md
  4. Every 250 ticks: Deep validation
     - Cross-reference entries against each other
     - Detect contradictions
     - Merge redundant entries (compression)
     - Compute domain-specific fitness contributions
```

The inner loop is fast, local, and lossy. Knowledge is born, tested, validated, degraded, and burned within a single Golem's lifetime. Most inner-loop knowledge dies with the Golem -- this is by design, implementing the Weismann barrier between individual experience and inherited information.

### 9.2 Outer Loop: Hosted Services

The outer loop operates across Golem generations through Styx: the Vault layer (encrypted backup), the Lethe layer (RAG retrieval), and the Clade (peer-to-peer sharing).

```
Generational transfer:
  1. Golem dies → Death testament produced (Thanatopsis Protocol)
  2. Death testament uploaded to Styx Archive (encrypted, 365-day TTL)
  3. Death testament indexed in Styx Lethe (1.5x retrieval weight)
  4. Compressed Grimoire pushed to Clade (confidence decayed to 0.4)
  5. Successor boots → Inherits compressed priors at 0.4 confidence
  6. Successor must validate inherited knowledge through direct experience
  7. Unvalidated inherited knowledge decays at 2x rate (generational penalty)
```

The outer loop is slow, compressed, and filtered. Only knowledge that passes the death testament's emotional life review and the Clade's governance filters survives to the next generation. The genomic bottleneck is implemented through two mechanisms:

1. **Confidence decay on inheritance**: All inherited entries arrive at 0.4 confidence (down from whatever the predecessor held). This is the epigenetic erasure equivalent.
2. **Generational decay rate**: `confidence = original * 0.85^N` where N is the generation count. Knowledge that was directly learned decays slowly; knowledge inherited 3 generations back decays rapidly.

```rust
pub fn inheritance_confidence(
    original_confidence: f64,
    generation: u32,
    decay_rate: f64,
) -> f64 {
    original_confidence * decay_rate.powi(generation as i32)
}

// Generation 0 (directly learned): 0.85 * 1.0 = 0.85
// Generation 1 (from predecessor): 0.85 * 0.85 = 0.72
// Generation 2 (from grandparent): 0.85 * 0.72 = 0.61
// Generation 3 (from great-grandparent): 0.85 * 0.52 = 0.44
// Generation 5: 0.85 * 0.44 = 0.37 -> approaching archive threshold
```

Knowledge that persists across 5+ generations without re-validation is almost certainly either fundamental (and should be re-derived from first principles) or wrong (and should be discarded). The generational decay ensures natural turnover.

### S9.3 Demurrage Reset at Generational Boundary

At inheritance, the successor receives knowledge with a **fresh demurrage clock**. The inherited entry gets a new `lastValidatedTick` set to the successor's current tick. Confidence is computed as:

```rust
let inherited_confidence =
    predecessor_confidence.min(1.0) * 0.85_f64.powi(generation_gap as i32);
```

Demurrage-decayed confidence from the predecessor's final state is **not** used as input. The generational decay factor (0.85^N) already accounts for information degradation across generations. Applying demurrage decay _on top of_ generational decay would double-count, making inherited knowledge unusably low confidence by generation 3+.

**Invariant**: `lastValidatedTick` resets at every generational boundary. Demurrage begins fresh from the moment of inheritance.

---

## S10 -- Knowledge Weighting Hierarchy

> **Extended:** computeKnowledgeWeight function, provenance/validation/emotional diversity multipliers, 8-rank hierarchy table -- see [../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md](../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md)

Weight is composite of type weight, provenance (directly learned > death testament > inherited), validation count, and emotional diversity. Death testaments rank highest (1.5--3.0) because they're produced under zero survival pressure. Speculative entries rank lowest (0.1--0.3).

---

## S11 -- What Demurrage Produces

Knowledge demurrage, memory rent, and knowledge burning combine to produce five systemic benefits:

### 1. A Lean, Current Grimoire

Stale heuristics naturally fade, keeping PLAYBOOK.md context relevant. The active Grimoire at any given moment contains only knowledge that has been recently validated or is inherently durable. Context window pollution is minimized. Retrieval quality improves because the search space is curated by time pressure.

### 2. Mortality Acceleration for Inactive Golems

A Golem in Conservation mode (monitoring only) learns less, validates less, and watches its Grimoire erode -- hastening epistemic death. This is the March (1991) feedback loop in action: the less the Golem interacts with the environment, the faster its knowledge decays, the sooner it dies. Demurrage ensures that mere observation cannot sustain a Golem's cognitive fitness indefinitely.

### 3. Natural Knowledge Turnover

Old entries make room for new ones without explicit deletion. The Grimoire is self-pruning. Capacity is naturally managed through competitive displacement. No garbage collection algorithm is needed -- time itself is the garbage collector.

### 4. Incentive to Explore

Only fresh evidence maintains confidence, rewarding active market engagement. A Golem that stops exploring watches its entire knowledge base decay toward zero. Exploration is not optional -- it is a survival requirement.

### 5. Forced Circulation

Knowledge that is valuable but unvalidated must be shared (with Clade, published to Styx Lethe) before it depreciates entirely. This implements Gesell's circulation-forcing principle: knowledge that isn't actively used by the holder should flow to someone who can use it, increasing the system's total knowledge velocity.

---

## S12 -- Nietzsche on Forgetting

> **Extended:** Nietzsche's positive forgetting as architectural principle (Genealogy of Morals, Untimely Meditations) -- see [../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md](../../prd2-extended/02-mortality/05-knowledge-demurrage-extended.md)

The Grimoire is not a library. It is a living garden that must be tended or it overgrows. Knowledge that is not actively maintained becomes noise.

---

## References

- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012.
- [BALDWIN-1896] Baldwin, J.M. "A New Factor in Evolution." _American Naturalist_ 30, 1896.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991.
- [BORGES-1942] Borges, J.L. "Funes the Memorious." _Ficciones_. 1944.
- [DAVIS-ZHONG-2017] Davis, R.L. & Zhong, Y. "The Biology of Forgetting." _Neuron_ 95(5), 2017.
- [EIP-1559-2021] Buterin, V. et al. EIP-1559: Fee Market Change for ETH 1.0 Chain. 2021.
- [GESELL-1916] Gesell, S. _The Natural Economic Order_. 1916.
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance." _Cell_ 157(1), 2014.
- [HINTON-NOWLAN-1987] Hinton, G. & Nowlan, S. "How Learning Can Guide Evolution." _Complex Systems_ 1, 1987.
- [LURIA-1968] Luria, A.R. _The Mind of a Mnemonist_. Harvard University Press, 1968.
- [MARCH-1991] March, J.G. "Exploration and Exploitation in Organizational Learning." _Organization Science_ 2(1), 1991.
- [NIETZSCHE-1874] Nietzsche, F. "On the Uses and Disadvantages of History for Life." _Untimely Meditations_, 1874.
- [NIETZSCHE-1887] Nietzsche, F. _On the Genealogy of Morals_. 1887.
- [OSTROM-1990] Ostrom, E. _Governing the Commons_. Cambridge University Press, 1990.
- [RICHARDS-FRANKLAND-2017] Richards, B.A. & Frankland, P.W. "The Persistence and Transience of Memory." _Neuron_ 94(6), 2017.
- [SHUVAEV-2024] Shuvaev, S. et al. "Encoding innate ability through a genomic bottleneck." _PNAS_ 121(6), 2024.

---
