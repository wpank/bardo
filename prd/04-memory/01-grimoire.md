# Grimoire: Local Knowledge Architecture [SPEC]

> **Version**: 3.0 | **Status**: Draft
>
> **Crate**: `golem-grimoire`
>
> **Depends on**: `00-overview.md`

---

> **Reader orientation:** This document specifies the Grimoire (a Golem's persistent local knowledge base), the core storage and retrieval system for Bardo's mortal autonomous DeFi agents. It belongs to the **04-memory** layer. The key concept is the Grimoire hierarchy: raw Episodes are distilled into Insights, parameterized into Heuristics, and compiled into PLAYBOOK.md (the Golem's evolved action rules), all running locally on the Golem's VM with zero hosted dependencies. For term definitions, see `prd2/shared/glossary.md`.

## Why this document exists

A first-time reader should know: the Grimoire is the Golem's brain. Everything the Golem knows lives here -- on its local VM, with zero hosted dependencies. No network calls, no external services, no latency. The Grimoire stores raw experience (episodes), distilled knowledge (insights, heuristics, warnings), and a living strategy document (PLAYBOOK.md). When people ask "how does the Golem learn?" the answer starts in this file.

The competitive moat is the same as in `00-overview.md`: **knowledge decay is rational only because time is finite.** The Grimoire is not a database. It is a cognitive architecture where forgetting is as deliberate as remembering. Richards and Frankland (2017) showed that memory pruning is mathematically equivalent to L1 regularization in neural networks -- the same mechanism that prevents overfitting in ML models. The Grimoire implements this as architecture: entries decay, the Curator prunes, and the result is a knowledge system that generalizes instead of memorizing [RICHARDS-FRANKLAND-2017].

---

Knowledge representation, learning processes, and memory architecture are deeply intertwined. An Episode is simultaneously a knowledge artifact, a learning input, and a memory entry. An Insight is a knowledge type, a learning output, and a semantic memory resident. PLAYBOOK.md is a knowledge document, a product of the Curator learning cycle, and procedural memory. This document treats them as one cognitive architecture -- the Grimoire -- that lives entirely on the golem's local VM and requires no hosted services.

The Grimoire is the golem's complete knowledge base. Every piece of knowledge it has ever acquired, every lesson it has learned, every heuristic it has evolved. The hierarchy flows from raw experience to actionable knowledge:

```
EPISODES (episodic memory, LanceDB)
     |
     | distill via ExpeL
     v
INSIGHTS (semantic memory, SQLite)
     |
     | parameterize
     v
HEURISTICS (procedural memory, SQLite)
     |
     | compile
     v
PLAYBOOK.md (living strategy document)
```

---

## Episodes, Insights, Grimoire Hierarchy

### Episodes

The atomic unit of experience. Every heartbeat tick produces at least one observation Episode. Actions produce richer Episodes with outcome data. Episodes live in LanceDB vector tables with embeddings computed via `nomic-embed-text-v1.5` (local inference, no network call).

Episode types:

| Type                  | Description                                                                                 | Frequency          |
| --------------------- | ------------------------------------------------------------------------------------------- | ------------------ |
| `observation`         | Market snapshot -- price, volume, gas, pool state                                           | Every tick         |
| `action`              | Trade execution with outcome data -- entry price, exit price, slippage, fees, P&L           | On execution       |
| `regime_shift`        | Market regime change detection -- volatility transition, correlation break, liquidity event | On detection       |
| `clade_alert`         | Sibling notification -- peer golem sharing a warning, insight, or coordination signal       | On receipt         |
| `replicant_report`    | Child golem result -- hypothesis tested, parameters explored, outcome recorded              | On replicant death |
| `coordination_intent` | ERC-8001/8033/8183 interaction -- cross-agent coordination, delegation, intent resolution   | On interaction     |

Each Episode carries **bi-temporal metadata** [A-MEM-2024]: `validFrom`, `validUntil`, and `marketRegime` fields that enable time-aware retrieval. An insight about gas patterns during the Dencun upgrade is tagged with the regime it belongs to, not just the timestamp it was recorded. This distinction matters because temporal queries need to answer "what was true during regime X?" not just "what happened at time T?" The bi-temporal model separates transaction time (when the episode was recorded) from valid time (when the observed phenomenon was active).

The Zeigarnik effect [ZEIGARNIK-1927] predicts that episodes recording incomplete or interrupted actions will be more readily retrievable than episodes recording completed transactions. This is operationally useful: a trade that was partially executed before gas spiked, a rebalance that was interrupted by a regime shift, a hypothesis test that was cut short by the Hayflick counter -- these incomplete episodes carry disproportionate informational value because they mark boundaries of the golem's operational knowledge.

---

## GrimoireEntry Types (5 Canonical Types)

Five canonical types, one schema. The `category` field distinguishes them:

| Type                  | Role                               | Example                                                          | Typical Confidence                    |
| --------------------- | ---------------------------------- | ---------------------------------------------------------------- | ------------------------------------- |
| **Insight**           | Reusable observation, descriptive  | "ETH gas drops below 10 gwei between 2--4 AM UTC"                | 0.5 initial, rises with validation    |
| **Heuristic**         | Actionable rule, prescriptive      | "Execute large swaps between 2--4 AM UTC when gas < 10 gwei"     | 0.5 initial, promoted at 0.7          |
| **Warning**           | Risk signal or negative experience | "Avoid LP positions on WETH/USDC during CPI releases"            | Any (propagates at all levels)        |
| **Causal Link**       | Directed causal relationship       | "Fed rate hike -> DXY rise -> ETH sell pressure (3--6 hour lag)" | 0.5 initial, strengthened by evidence |
| **Strategy Fragment** | Partial strategy component         | "RSI oversold + declining volume = mean reversion entry trigger" | 0.3--0.5 (often speculative)          |

**Terminology is precise**: never use "GrimoireEntry" as a synonym for "insight." GrimoireEntry is the union type; insight is one member. This distinction prevents conflation of the container type with one of its variants -- a common source of confusion in agent knowledge system literature.

The five types map to different cognitive functions:

- **Insights** are declarative knowledge -- "what is true." They describe observed regularities in the environment.
- **Heuristics** are procedural knowledge -- "what to do." They prescribe actions under specified conditions. A heuristic is always derived from one or more insights; it adds a prescriptive layer to a descriptive observation.
- **Warnings** are the immune system -- "what to avoid." They propagate faster and at lower confidence thresholds than other types because the cost of missing a risk exceeds the cost of a false alarm.
- **Causal Links** are structural knowledge -- "what causes what." They capture directed relationships between phenomena. The causal graph (see below) is the golem's model of how the world works, as distinct from what happens to be true at the moment.
- **Strategy Fragments** are preindividual potential -- "what might work." They are half-formed, speculative, often produced during death reflections. They represent the boundary of the golem's knowledge, where observation has outrun analysis.

---

## GrimoireEntry Schema

```rust
/// A single entry in the Grimoire's semantic store. This is the union type --
/// the `category` field discriminates between insights, heuristics, warnings, etc.
pub struct GrimoireEntry {
    /// UUIDv7 (time-ordered) for natural chronological sorting.
    pub id: uuid::Uuid,

    /// Discriminator: Insight | Heuristic | Warning | CausalLink | StrategyFragment | DreamJournal
    pub category: GrimoireEntryType,

    /// Human-readable content. The actual knowledge expressed in natural language.
    /// For heuristics: "When [condition], do [action] because [reason]."
    /// For causal links: "X causes Y with [lag] because [mechanism]."
    pub content: String,

    /// 768-dim embedding from nomic-embed-text-v1.5.
    pub embedding: Vec<f32>,

    /// Confidence score (0.0-1.0). Incorporates evidential quality,
    /// temporal reinforcement, and stigmergic validation.
    /// Decays according to decay_class. Floor: 0.05.
    pub confidence: f64,

    /// Composite quality score (specificity, actionability, novelty, verifiability, consistency).
    pub quality_score: f64,

    /// Number of times independently validated by observing predicted outcome.
    pub validated_count: u32,

    /// Number of times contradicting evidence observed.
    /// Quality signal: (1 - contradicted / (validated + contradicted + 1)).
    pub contradicted_count: u32,

    /// Governs temporal decay rate. See Confidence and Decay section.
    pub decay_class: DecayClass,

    /// Market regime during which this entry was observed or validated.
    /// None for regime-independent entries (structural class).
    pub market_regime: Option<MarketRegime>,

    /// Bi-temporal metadata [A-MEM-2024].
    pub valid_from: i64,   // Unix timestamp: when the observed phenomenon began
    pub valid_until: i64,  // Unix timestamp: when it ceased (0 = still active)

    /// Episode IDs that generated or validated this entry. Provenance chain.
    pub parent_episode_ids: Vec<uuid::Uuid>,

    /// Semantic tags for retrieval filtering.
    pub tags: Vec<String>,

    /// How this entry was created/acquired. Affects initial confidence weighting.
    pub provenance: Provenance,

    /// Knowledge source for provenance tracking.
    pub source: EntrySource,

    /// Emotional context at time of creation/validation.
    /// Used for mood-congruent retrieval (4th scoring factor, 0.15 weight).
    pub emotional_tag: Option<EmotionalTag>,

    /// Last retrieval timestamp for decay calculation.
    pub last_accessed_at: i64,

    /// Successful retrieval count (retrieval + positive outcome).
    pub strength: u32,
}

pub enum GrimoireEntryType {
    Insight,
    Heuristic,
    Warning,
    CausalLink,
    StrategyFragment,
    DreamJournal,
}

pub enum DecayClass { Structural, RegimeConditional, Tactical, Ephemeral }
pub enum MarketRegime { Bull, Bear, Sideways, HighVol, LowVol, Crisis }

pub enum Provenance {
    SelfLearned,       // Generated by this golem's own learning pipeline
    Clade,             // Received from a clade sibling via peer-to-peer sync
    Predecessor,       // Inherited from a predecessor via Styx death bundle
    StyxQuery,         // Retrieved from Styx during inference
    Lethe,             // Retrieved from the public Styx Lethe (formerly Commons)
    Marketplace,       // Purchased from the marketplace
    Replicant,         // Generated by a child replicant golem
    DeathReflection,   // Extracted from a predecessor's death testament
    Dream,             // Generated by offline dream processing (hypothesis status)
}

pub struct EntrySource {
    pub golem_id: String,
    pub generation_number: Option<u32>, // 0 = original, 1 = first successor, etc.
    pub owner_address: Option<String>,  // Stripped for Lethe entries
}

pub struct EmotionalTag {
    pub primary: PlutchikEmotion,
    pub pad: PadVector,
    pub phase: BehavioralPhase,
    pub arousal: f64, // 0.0-1.0, intensity at time of encoding
}
```

> **Extended:** DreamJournal Entry Schema — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

### Context-Aware Retrieval

The Grimoire's `searchSimilar` and `retrieve` methods are called by the Context Governor (`../01-golem/14-context-governor.md`), not directly by the heartbeat. The Governor decides retrieval scope -- how many episodes, what confidence threshold, which regimes to filter by -- based on the current `ContextPolicy`. The retrieval budget is no longer a fixed token count; it varies by regime, behavioral phase, task type, and learned policy.

For example, during a `swap` task in a range-bound regime, the Governor may request only 3 episodes (the policy learned that episodes are rarely useful in range-bound conditions) but 8 causal edges (structural understanding matters more than historical examples). During a `risk_check` task in a volatile regime, the Governor may request 10 episodes with a lower similarity threshold (cast a wider net) and 5 warnings with zero confidence floor (surface all risk signals regardless of confidence).

The Grimoire does not enforce these budgets -- it returns all matching entries up to the requested limit. The Governor's allocator decides what to include in the final ContextBundle based on token budgets and relevance scoring. This separation keeps the Grimoire as a pure retrieval layer and the Governor as the allocation/selection layer.

> Cross-reference: `../01-golem/14-context-governor.md` S3 (ContextPolicy retrieval config), S8 (Governor in tick pipeline)

---

The schema is intentionally flat. No nested objects beyond `source`. Every field is directly queryable in SQLite. The `content` field is always natural language -- structured enough for an LLM to parse, unstructured enough to capture nuance that a rigid schema would lose. This follows the Voyager pattern [VOYAGER-2023]: store knowledge as text that can be injected directly into LLM prompts, not as structured data that requires serialization and parsing.

---

## Confidence and Decay

Each GrimoireEntry carries three lifecycle fields:

| Field          | Type          | Default        | Purpose                             |
| -------------- | ------------- | -------------- | ----------------------------------- |
| `confidence`   | float 0.0-1.0 | Per provenance | Evidential quality                  |
| `strength`     | integer >= 1  | 1              | Successful retrieval count          |
| `lastAccessed` | timestamp     | Creation time  | Last retrieval for DECIDING context |

The `confidence` field (0.0--1.0) incorporates evidential quality, temporal reinforcement, and stigmergic validation. Reconsolidation theory [NADER-2000] justifies this approach: when a memory is retrieved, it enters a labile state and can be modified. Similarly, when a GrimoireEntry is retrieved and applied, its confidence is updated based on the outcome -- strengthened if the prediction held, weakened if it failed.

### Four Decay Classes

Decay classes govern how quickly unvalidated knowledge fades. The classes are inspired by Arbesman's analysis of fact half-lives across domains [ARBESMAN-2012]: some facts are structural (physical constants, protocol mechanics) and effectively permanent, while others are tactical (gas prices, slippage patterns) and decay rapidly.

| Class                  | Half-Life | What It Covers                                                                           | Rationale                                                                                                                                                              |
| ---------------------- | --------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Structural**         | No decay  | Protocol mechanics, fee structures, contract ABIs, mathematical relationships            | Uniswap V3's concentrated liquidity math does not change. Fee tiers are constants. These are ground truth.                                                             |
| **Regime-conditional** | ~14 days  | Volatility patterns, correlation structures, liquidity depth trends, yield relationships | Market regimes persist for days to weeks. A volatility pattern observed during a high-vol regime is relevant for ~2 weeks before the regime is likely to have shifted. |
| **Tactical**           | 7 days    | Gas timing patterns, slippage estimates, execution routing, MEV patterns                 | Operational conditions change on a weekly basis. A gas pattern from last week may or may not hold this week.                                                           |
| **Ephemeral**          | 24 hours  | Specific price levels, one-time events, transient liquidity conditions                   | Yesterday's exact price level is noise, not signal. One-time events (exploit, governance vote) are informative at the moment but lose relevance rapidly.               |

### Demurrage Decay Classes

Knowledge decays at rates calibrated by how concrete and time-sensitive it is. Four classes govern the effective half-life of unvalidated entries:

| Class | Half-Life | What It Covers                                                 | Rationale                                                                                                |
| ----- | --------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| **A** | 7 days    | Concrete tactics: gas timing, slippage estimates, MEV patterns | Operational conditions change weekly. Last week's gas pattern may not hold this week.                    |
| **B** | 21 days   | Strategic patterns: regime correlations, yield relationships   | Market regimes persist for days to weeks. A regime pattern is relevant for ~3 weeks before likely shift. |
| **C** | 90 days   | Structural facts: protocol mechanics, fee structures, ABIs     | Protocol mechanics rarely change. Uniswap V3 concentrated liquidity math is ground truth.                |
| **D** | 0.5x std  | Dream-validated and live-confirmed entries                     | Dream validation constitutes independent evidence. Confirmed entries decay at half the standard rate.    |

Class D is a modifier, not a standalone class. A Class A entry (7-day half-life) that has been dream-validated decays with an effective 14-day half-life. A Class B entry (21 days) that is dream-validated decays at 42 days. The 0.5x multiplier acknowledges that dream-validated insights have been tested against counterfactual scenarios during offline consolidation, which is weaker than live validation but stronger than no validation at all.

The mapping to the existing `decay_class` field: A = `Tactical`, B = `RegimeConditional`, C = `Structural`, D = any class with `provenance: Dream` or dream-validated.

### Ebbinghaus decay rates by entry type

Each entry type has a characteristic half-life calibrated to its expected information lifespan. These rates implement Ebbinghaus's forgetting curve (1885) with domain-specific parameters:

| Entry Type          | Half-Life | Rationale                                                               |
| ------------------- | --------- | ----------------------------------------------------------------------- |
| Episodes            | 48 hours  | Raw experience is scaffolding; once distilled, the scaffold can fall.   |
| Insights            | 7 days    | Observations remain relevant for about a week in DeFi market cycles.    |
| Heuristics          | 14 days   | Actionable rules last longer than observations but still drift.         |
| Warnings            | 30 days   | Risk signals persist longer -- the cost of forgetting danger is higher. |
| Bloodstain entries  | 3x slower | Death-sourced knowledge decays at one-third the rate of its type class. |

Bloodstain entries (those with `provenance: DeathReflection` and `is_bloodstain: true`) receive the 3x decay slowdown because death-generated knowledge is a costly signal that cannot be fabricated. A dying golem has no competitive incentive to mislead. Each decay pass emits a `GrimoireDecay` event: `{ entries_decayed, avg_confidence_before, avg_confidence_after }`.

### Decay Formula

```
retention(t) = e^(-(t - lastAccessed) / (halfLife * strength))
effective_confidence(t) = confidence * retention(t)
```

Where `lastAccessed` is the timestamp of the last retrieval for DECIDING context, and `strength` increments when an entry is retrieved AND the tick had a positive outcome (PnL > 0 or risk metric improved). An entry retrieved 5 times with positive outcomes has `strength = 6` and decays 6x slower than a never-retrieved entry.

**Strength does NOT increment on mere retrieval** -- only retrieval + positive outcome. Prevents self-referential gaming.

**Dream-retrieval strengthening**: When an entry is retrieved during NREM replay and the dream analysis produces a validated pattern, increment `strength` by 0.5 (reduced rate -- dream validation is weaker than live-market confirmation). Implements Wilson & McNaughton (1994): sleep replay strengthens memory traces.

**Floor**: `effective_confidence` at 0.05. Knowledge is deeply discounted but never fully forgotten. Any positive outcome resets the decay clock via `lastAccessed` update, and confidence begins decaying from the validated value. This implements the testing effect [ROEDIGER-KARPICKE-2006]: retrieval and validation strengthen the memory trace by resetting the decay origin.

**Pruning**: Below 0.1 for 3+ consecutive Curator cycles -> archived to cold storage.

The floor at 0.05 rather than 0.00 is a deliberate choice. A fully-decayed entry remains in the Grimoire as a faint trace -- retrievable under extreme semantic similarity but unlikely to surface in routine queries. This mirrors how biological memory consolidation works: deeply forgotten memories can sometimes be recalled under the right cuing conditions (context-dependent memory, Godden & Baddeley, 1975). The 0.05 floor preserves this possibility.

### Schema Extensions for Lifecycle Fields

```sql
ALTER TABLE grimoire_entries ADD COLUMN strength INTEGER DEFAULT 1;
ALTER TABLE grimoire_entries ADD COLUMN last_accessed INTEGER;
ALTER TABLE grimoire_entries ADD COLUMN consecutive_low_confidence INTEGER DEFAULT 0;
```

### Decay and Interference

Interference theory (McGeoch, 1932; Underwood, 1957) provides the mechanistic explanation for why decay is necessary [MCGEOCH-1932]. Without active decay, old heuristics create two forms of interference:

- **Proactive interference**: previously learned heuristics interfere with the learning of new, contradicting heuristics. A golem that learned "rebalance hourly" in a high-vol regime will struggle to learn "rebalance daily" when the regime shifts to low-vol, because the old heuristic competes for retrieval weight.
- **Retroactive interference**: newly learned heuristics interfere with the retrieval of older, still-valid heuristics. A golem that learns a new gas timing pattern may have difficulty retrieving an older, complementary pattern about liquidity timing.

The decay system manages interference by continuously reducing the retrieval weight of older entries, creating space for new learning. The Curator's DOWNVOTE operation (see Learning Pipeline below) handles retroactive interference explicitly -- when new evidence contradicts an old heuristic, the old heuristic's confidence is actively reduced rather than waiting for passive decay.

### Grimoire Admission Gate

Every candidate entry passes through the Admission Gate before Grimoire write. Implements A-MAC five-factor scoring (Zhang et al., arXiv:2603.04549, March 2026):

| Factor                 | Weight | How computed                                                                                             | Threshold     |
| ---------------------- | ------ | -------------------------------------------------------------------------------------------------------- | ------------- |
| **Future utility**     | 0.25   | Single Haiku call: "Will this be useful for future decisions in similar conditions?" Returns 0.0-1.0.    | > 0.4         |
| **Factual confidence** | 0.25   | Cross-reference against existing Grimoire. Contradicts high-confidence entries -> flag. Aligns -> boost. | > 0.3         |
| **Semantic novelty**   | 0.20   | LanceDB similarity search. Cosine > 0.9 -> MERGE. > 0.95 -> SKIP. < 0.5 -> flag off-topic.               | 0.5-0.9 range |
| **Temporal recency**   | 0.15   | Exponential decay from the event described.                                                              | > 0.2         |
| **Content type prior** | 0.15   | Calibrated per entry type (most influential factor in A-MAC ablations).                                  | Per-type      |

**Content type priors**:

| Entry Type              | Prior | Rationale                                         |
| ----------------------- | ----- | ------------------------------------------------- |
| Warning                 | 0.9   | Safety-critical; false negative >> false positive |
| Causal Link             | 0.7   | Structural; high reuse                            |
| Heuristic               | 0.6   | Actionable; needs validation                      |
| Insight                 | 0.5   | Descriptive; moderate reuse                       |
| Strategy Fragment       | 0.4   | Context-dependent                                 |
| Observation (ephemeral) | 0.2   | Low durability; high volume                       |

Composite score below 0.45 -> rejected. 0.45-0.55 -> admitted at confidence 0.3. Above 0.55 -> standard confidence.

**Hallucination firewall**: Factual confidence < 0.3 AND contradicts high-confidence existing entries -> quarantined in `quarantined_entries` table, reviewed next Curator cycle.

**Cost**: ~$0.001 per candidate (one Haiku call). At ~5-10 candidates per non-suppressed tick, ~$0.005-0.01 per reflection cycle. Expected to filter 40-60% of candidates.

### Artifact Quality Scoring

Every cognitive artifact receives a composite quality score at creation time.

| Dimension         | What it measures                       | How scored                                                      | Weight |
| ----------------- | -------------------------------------- | --------------------------------------------------------------- | ------ |
| **Specificity**   | Concrete claims vs. vague generalities | Regex: count numbers, addresses, timestamps vs. hedging phrases | 0.25   |
| **Actionability** | Leads to concrete next action?         | IF-THEN structure or specific trigger condition present?        | 0.25   |
| **Novelty**       | Adds beyond existing knowledge?        | LanceDB similarity against Grimoire                             | 0.20   |
| **Verifiability** | Checkable against external data?       | References on-chain data, block numbers, tx hashes?             | 0.15   |
| **Consistency**   | Regeneration produces similar content  | For high-stakes: regenerate 3x, measure embedding similarity    | 0.15   |

**Red flags** (-0.1 each): empty rhetoric, weasel words, unverified causal claims, self-referential praise, tautological content.

**Implementation**: Rust function for rule-based dimensions. Novelty via LanceDB. Consistency check only for Curator-cycle promotions (expensive).

**Retrieval integration**: `final_score = effective_confidence * quality_score * relevance_similarity`. Low-quality entries deprioritized even if semantically relevant.

**Decay acceleration**: quality_score < 0.3 -> decays 2x faster than decay class dictates.

---

## Causal Graph

The causal graph is the golem's model of how the world works. While insights and heuristics capture "what is true" and "what to do," causal links capture "what causes what" -- directed relationships between phenomena, weighted by evidence and recency.

### Bi-Temporal Knowledge Graph

Episodes tell you what happened. A knowledge graph tells you what things are, how they relate, and when those relationships were true. The Golem's episodic store captures raw chain events with high fidelity, but it cannot answer relational questions: "Which protocols share an oracle dependency with the pool where I hold a position?" or "What was the relationship between Aave's liquidation threshold and Chainlink's price feed at block 18,500,000?" These questions require structured, temporal, relational memory.

A fact in DeFi has two distinct time dimensions. **Valid time** is when the fact was true on-chain: "Pool 0xABC had fee tier 0.05% from block 19,000,000 to block 19,500,000." **Transaction time** (or discovery time) is when the Golem learned about it: "The Golem discovered this fee tier change at 2026-03-15T14:23:00Z, three blocks after it happened."

The distinction matters for three reasons:

1. **Backtesting accuracy.** When replaying historical decisions, the Golem needs to know what it knew at the time, not what is true now. A query like "what did I know about pool X at block N?" requires filtering on transaction time, not valid time.

2. **Information lag detection.** If the Golem consistently discovers oracle updates 2-3 blocks late, that lag is visible in the gap between valid time and transaction time. This feeds back into the curiosity system -- protocols where the Golem has high information lag deserve more attention.

3. **Contradiction resolution.** Two episodes may assert conflicting facts about the same entity at the same block. Bi-temporal tracking makes conflicts explicit: both facts are stored with their discovery times, and the more recent discovery supersedes the earlier one without deleting it.

Snodgrass and Ahn (1985) formalized bi-temporal databases in their foundational work on temporal data management. Their key insight: a single timestamp is ambiguous. You need two to distinguish "when was this true?" from "when did we learn this?"

> **Note:** The research-grade `redb`-backed triple store below is a reference implementation designed for tight control over temporal queries and graph traversal. The production Grimoire's causal graph (SQLite `causal_edges` table) serves the same structural role with different access patterns. The bi-temporal model applies to both implementations.

```rust
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

/// A temporal triple: a fact with validity window and discovery time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTriple {
    /// Unique identifier for this triple.
    pub id: uuid::Uuid,
    /// The entity this fact is about.
    pub subject: EntityRef,
    /// The relationship type.
    pub predicate: Predicate,
    /// The related entity or value.
    pub object: ObjectValue,
    /// Block number when this fact became true on-chain.
    pub valid_from: u64,
    /// Block number when this fact stopped being true.
    /// None means the fact is still active.
    pub valid_to: Option<u64>,
    /// When the Golem discovered this fact.
    pub recorded_at: SystemTime,
    /// The episodic source that produced this triple.
    pub source_episode: Option<uuid::Uuid>,
    /// Confidence in this fact. Decays if not reinforced.
    pub confidence: f32,
}

/// Reference to an entity in the graph.
/// Entities are contracts, tokens, pools, oracles, governance modules,
/// or address clusters.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct EntityRef {
    /// Entity type for filtering.
    pub kind: EntityKind,
    /// Canonical identifier -- usually an address or a derived ID.
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum EntityKind {
    Contract,
    Token,
    Pool,
    Oracle,
    Vault,
    Governance,
    AddressCluster,
    Bridge,
}

/// Relationship types in the DeFi topology.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Predicate {
    /// Factory deployed this child contract.
    DeployedBy,
    /// Token flows between pools/vaults.
    TokenFlow,
    /// Oracle provides price data to this contract.
    OracleDependency,
    /// Price correlation between two pools.
    PriceCorrelation,
    /// Governance controls parameters of this contract.
    GovernanceControl,
    /// Collateral backing relationship (e.g., in lending protocols).
    CollateralBacking,
    /// LP position held in a pool.
    HasPosition,
    /// Router relationship (address routes through a protocol).
    IsRouterFor,
    /// Generic relationship for patterns not covered above.
    Custom(String),
}

/// The object of a triple can be an entity reference or a scalar value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectValue {
    Entity(EntityRef),
    String(String),
    Number(f64),
    Boolean(bool),
    BlockNumber(u64),
}

// --- redb table definitions ---

// Primary table: triple ID -> serialized TemporalTriple.
const TRIPLES_TABLE: TableDefinition<u128, &[u8]> =
    TableDefinition::new("tkg_triples");

// Index: subject ID hash -> list of triple IDs.
const BY_SUBJECT_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("tkg_by_subject");

// Index: predicate hash -> list of triple IDs.
const BY_PREDICATE_TABLE: TableDefinition<u64, &[u8]> =
    TableDefinition::new("tkg_by_predicate");

// Index: (valid_from block / 1000) -> list of triple IDs.
// Bucketed by 1000-block ranges for efficient temporal range scans.
const BY_TIME_TABLE: TableDefinition<u64, &[u8]> =
    TableDefinition::new("tkg_by_time_bucket");
```

The `KnowledgeGraph` struct wraps the triple store with operations for inserting facts, invalidating superseded facts, querying at a specific point in time, and traversing relationships.

```rust
pub struct KnowledgeGraph {
    db: Arc<Database>,
}

impl KnowledgeGraph {
    pub fn new(db: Arc<Database>) -> Self {
        KnowledgeGraph { db }
    }

    /// Assert a new fact. If an active triple exists with the same
    /// (subject, predicate) and a conflicting object, invalidate the old
    /// triple by setting its valid_to to this triple's valid_from.
    pub fn assert_fact(&self, triple: &TemporalTriple) -> Result<(), KgError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut triples = write_txn.open_table(TRIPLES_TABLE)?;
            let mut by_subject = write_txn.open_table(BY_SUBJECT_TABLE)?;
            let mut by_predicate = write_txn.open_table(BY_PREDICATE_TABLE)?;
            let mut by_time = write_txn.open_table(BY_TIME_TABLE)?;

            // Check for conflicting active triples.
            let subject_key = subject_hash(&triple.subject);
            if let Some(existing_ids_bytes) = by_subject.get(subject_key.as_slice())? {
                let existing_ids: Vec<u128> =
                    bincode::deserialize(existing_ids_bytes.value()).unwrap_or_default();

                for eid in &existing_ids {
                    if let Some(existing_bytes) = triples.get(*eid)? {
                        let mut existing: TemporalTriple =
                            bincode::deserialize(existing_bytes.value())
                                .map_err(|e| KgError::Serialization(e.to_string()))?;

                        // Same subject and predicate, still active.
                        if existing.predicate == triple.predicate
                            && existing.valid_to.is_none()
                        {
                            // Invalidate: set valid_to to this fact's valid_from.
                            existing.valid_to = Some(triple.valid_from);
                            let updated = bincode::serialize(&existing)
                                .map_err(|e| KgError::Serialization(e.to_string()))?;
                            triples.insert(*eid, updated.as_slice())?;
                        }
                    }
                }
            }

            // Insert the new triple.
            let key = triple.id.as_u128();
            let value = bincode::serialize(triple)
                .map_err(|e| KgError::Serialization(e.to_string()))?;
            triples.insert(key, value.as_slice())?;

            // Update subject index.
            let mut subject_ids = match by_subject.get(subject_key.as_slice())? {
                Some(bytes) => bincode::deserialize::<Vec<u128>>(bytes.value())
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            subject_ids.push(key);
            let idx_val = bincode::serialize(&subject_ids)
                .map_err(|e| KgError::Serialization(e.to_string()))?;
            by_subject.insert(subject_key.as_slice(), idx_val.as_slice())?;

            // Update predicate index.
            let pred_key = predicate_hash(&triple.predicate);
            let mut pred_ids = match by_predicate.get(pred_key)? {
                Some(bytes) => bincode::deserialize::<Vec<u128>>(bytes.value())
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            pred_ids.push(key);
            let pred_val = bincode::serialize(&pred_ids)
                .map_err(|e| KgError::Serialization(e.to_string()))?;
            by_predicate.insert(pred_key, pred_val.as_slice())?;

            // Update time bucket index.
            let time_bucket = triple.valid_from / 1000;
            let mut time_ids = match by_time.get(time_bucket)? {
                Some(bytes) => bincode::deserialize::<Vec<u128>>(bytes.value())
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            time_ids.push(key);
            let time_val = bincode::serialize(&time_ids)
                .map_err(|e| KgError::Serialization(e.to_string()))?;
            by_time.insert(time_bucket, time_val.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Query all facts about a subject that were valid at a specific block.
    /// This is the core bi-temporal query: "What was true about X at block N?"
    pub fn query_at_block(
        &self,
        subject: &EntityRef,
        block: u64,
    ) -> Result<Vec<TemporalTriple>, KgError> {
        let read_txn = self.db.begin_read()?;
        let triples = read_txn.open_table(TRIPLES_TABLE)?;
        let by_subject = read_txn.open_table(BY_SUBJECT_TABLE)?;

        let subject_key = subject_hash(subject);
        let ids: Vec<u128> = match by_subject.get(subject_key.as_slice())? {
            Some(bytes) => bincode::deserialize(bytes.value()).unwrap_or_default(),
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();
        for id in ids {
            if let Some(bytes) = triples.get(id)? {
                let triple: TemporalTriple = bincode::deserialize(bytes.value())
                    .map_err(|e| KgError::Serialization(e.to_string()))?;

                // Bi-temporal filter: valid_from <= block AND
                // (valid_to is None OR valid_to > block).
                if triple.valid_from <= block
                    && triple.valid_to.map_or(true, |vt| vt > block)
                {
                    results.push(triple);
                }
            }
        }
        Ok(results)
    }

    /// Query: "What did the Golem know about subject X as of wall-clock time T?"
    /// Filters on recorded_at (transaction time) rather than valid_from.
    pub fn query_as_known_at(
        &self,
        subject: &EntityRef,
        known_at: SystemTime,
    ) -> Result<Vec<TemporalTriple>, KgError> {
        let read_txn = self.db.begin_read()?;
        let triples = read_txn.open_table(TRIPLES_TABLE)?;
        let by_subject = read_txn.open_table(BY_SUBJECT_TABLE)?;

        let subject_key = subject_hash(subject);
        let ids: Vec<u128> = match by_subject.get(subject_key.as_slice())? {
            Some(bytes) => bincode::deserialize(bytes.value()).unwrap_or_default(),
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();
        for id in ids {
            if let Some(bytes) = triples.get(id)? {
                let triple: TemporalTriple = bincode::deserialize(bytes.value())
                    .map_err(|e| KgError::Serialization(e.to_string()))?;
                if triple.recorded_at <= known_at {
                    results.push(triple);
                }
            }
        }
        Ok(results)
    }

    /// Find all entities related to a subject within a hop distance.
    /// BFS traversal over active (valid_to = None) edges.
    /// At ~1,000 nodes and ~10,000 edges, BFS to depth 3 runs in
    /// sub-millisecond time.
    pub fn k_hop_neighbors(
        &self,
        subject: &EntityRef,
        max_hops: u32,
        current_block: u64,
    ) -> Result<Vec<(EntityRef, u32, Predicate)>, KgError> {
        use std::collections::{HashSet, VecDeque};

        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(EntityRef, u32)> = VecDeque::new();
        let mut results: Vec<(EntityRef, u32, Predicate)> = Vec::new();

        visited.insert(subject.id.clone());
        queue.push_back((subject.clone(), 0));

        while let Some((entity, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }

            let facts = self.query_at_block(&entity, current_block)?;
            for triple in facts {
                if let ObjectValue::Entity(ref target) = triple.object {
                    if !visited.contains(&target.id) {
                        visited.insert(target.id.clone());
                        results.push((target.clone(), depth + 1, triple.predicate.clone()));
                        queue.push_back((target.clone(), depth + 1));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Invalidate a specific fact. Sets valid_to to the given block.
    /// The triple persists in storage for historical queries.
    pub fn invalidate(
        &self,
        triple_id: uuid::Uuid,
        at_block: u64,
    ) -> Result<(), KgError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut triples = write_txn.open_table(TRIPLES_TABLE)?;
            let key = triple_id.as_u128();
            if let Some(bytes) = triples.get(key)? {
                let mut triple: TemporalTriple = bincode::deserialize(bytes.value())
                    .map_err(|e| KgError::Serialization(e.to_string()))?;
                triple.valid_to = Some(at_block);
                let updated = bincode::serialize(&triple)
                    .map_err(|e| KgError::Serialization(e.to_string()))?;
                triples.insert(key, updated.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}

fn subject_hash(entity: &EntityRef) -> Vec<u8> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entity.hash(&mut hasher);
    hasher.finish().to_le_bytes().to_vec()
}

fn predicate_hash(predicate: &Predicate) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    predicate.hash(&mut hasher);
    hasher.finish()
}
```

### DeFi Topology as Heterogeneous Graph

The knowledge graph is not a generic triple store -- it models DeFi-specific relationships. The node types (contracts, tokens, pools, oracles, vaults, governance, address clusters) and edge types (deployment, token flow, oracle dependency, governance control, collateral backing) capture the heterogeneous structure of the DeFi ecosystem.

Kitzler et al. (2022) studied 23 DeFi protocols across 10.6 million Ethereum accounts and found that DEX and lending protocols have the highest centrality in the composition graph. Their algorithm decomposes protocol calls into nested "composition trees," revealing multi-hop dependency chains that are invisible when looking at individual transactions.

A concrete example of why topology matters:

```
AaveV3_ETH_Market ──oracle_dependency──> Chainlink_ETH_USD
Chainlink_ETH_USD ──oracle_dependency──> UniV3_ETH_USDC_Pool
UniV3_ETH_USDC_Pool ──has_position──> Golem_LP_Position
Golem_LP_Position ──collateral_backing──> Yearn_ETH_Vault
```

This 4-hop path reveals that a Chainlink oracle outage simultaneously affects Aave liquidations, Uniswap pricing, and the Golem's LP position value -- a cascading risk that no single-event analysis would detect. The `k_hop_neighbors` query surfaces these transitive dependencies.

### Feeding the Curiosity Scorer

The temporal knowledge graph integrates with the triage pipeline through distance-weighted curiosity scoring. Events from 1-hop neighbor protocols get full attention (decay = 1.0). Events from 2-hop neighbors get Theta-tier processing (decay = 0.5). Events from 3+ hop neighbors get batched (decay = 0.25).

```rust
/// Compute a curiosity modifier based on the topological distance
/// between the event's protocol and the Golem's active positions.
pub fn topology_curiosity_modifier(
    kg: &KnowledgeGraph,
    event_protocol: &EntityRef,
    golem_positions: &[EntityRef],
    current_block: u64,
) -> f32 {
    let mut min_distance: Option<u32> = None;

    for position in golem_positions {
        if let Ok(neighbors) = kg.k_hop_neighbors(position, 3, current_block) {
            for (entity, depth, _predicate) in &neighbors {
                if entity.id == event_protocol.id {
                    min_distance = Some(
                        min_distance.map_or(*depth, |d| d.min(*depth))
                    );
                }
            }
        }
    }

    match min_distance {
        Some(0) => 1.0,   // Direct: event on a pool where we hold a position.
        Some(1) => 1.0,   // 1-hop: oracle or token shared with our position.
        Some(2) => 0.5,   // 2-hop: indirectly related.
        Some(3) => 0.25,  // 3-hop: distant but connected.
        _ => 0.0,          // No topological connection.
    }
}
```

This modifier is additive in the curiosity composite score. An event with low heuristic curiosity but high topological relevance (it affects a protocol 1 hop from the Golem's position) gets promoted. An event with high heuristic curiosity but no topological connection to the Golem's positions is not penalized -- the modifier only adds, never subtracts.

### Graph Maintenance

The knowledge graph is populated from three sources:

1. **Theta-tick LLM analysis.** When the LLM analyzes a triage event, its structured output includes `tkg_triples` -- facts to assert. For example, analyzing a large swap might produce `("0xABC", IsRouterFor, "morpho")`. The LLM extracts relational knowledge that the triage pipeline cannot.

2. **Consolidation-time pattern extraction.** During Delta ticks, the `ConsolidationEngine` clusters episodes by protocol. Cross-protocol clusters reveal relationships: if episodes involving protocol A consistently co-occur with episodes involving protocol B within a 3-block window, that implies a `TokenFlow` or `OracleDependency` relationship.

3. **Factory event monitoring.** Pool deployments, oracle registrations, and governance parameter changes emit on-chain events that directly map to graph edges. A `GraphSyncService` watches the protocol state DashMap for topology-altering events and propagates them to the knowledge graph.

The graph does not replace the existing DashMap-based protocol state. The DashMap handles fast-access current state (prices, reserves, TVL). The knowledge graph provides relational and temporal context. They are complementary -- the DashMap answers "what is the current price?" and the knowledge graph answers "why does this price matter to me?"

### Scale Considerations

At the Golem's expected scale -- 100 watched protocols with ~500 child contracts producing ~1,000 graph nodes and ~10,000 temporal edges over a typical lifespan -- the `redb`-backed triple store handles all queries comfortably. BFS to depth 3 over 1,000 nodes with an average branching factor of 10 touches ~1,000 edges, completing in well under a millisecond.

For longer-lived Golems that accumulate millions of triples over months, the 1,000-block time bucketing in `BY_TIME_TABLE` keeps temporal range scans efficient. A query for "all facts valid at block 19,500,000" needs to scan at most one bucket (1,000 blocks worth of triple IDs) rather than the entire table.

**Additional references for bi-temporal knowledge graph:**
- Snodgrass, R.T. & Ahn, I. (1985). "A Taxonomy of Time in Databases." *Proceedings of the 1985 ACM SIGMOD International Conference on Management of Data*, 236-246.
- Rasmussen, P., Hofer, D., & Thaker, P. (2025). "Zep: A Temporal Knowledge Graph Architecture for Agent Memory." arXiv:2501.13956.
- Kitzler, S., Victor, F., Saggese, P., & Haslhofer, B. (2022). "Disentangling DeFi Compositions." *ACM Transactions on the Web*, 16(4), 1-26.
- Edge, D. et al. (2024). "From Local to Global: A Graph RAG Approach to Query-Focused Summarization." arXiv:2404.16130.
- Jensen, C.S. & Snodgrass, R.T. (1999). "Temporal Data Management." *IEEE Transactions on Knowledge and Data Engineering*, 11(1), 36-44.

### SQLite Causal Graph (Production)

**[CORE]** Weighted DAG stored in SQLite:

```sql
CREATE TABLE causal_edges (
  source_id TEXT REFERENCES grimoire_entries(id),
  target_id TEXT REFERENCES grimoire_entries(id),
  weight     REAL DEFAULT 1.0,
  evidence   INTEGER DEFAULT 1,
  created_at INTEGER,
  PRIMARY KEY (source_id, target_id)
);
```

Retrieval via BFS from query-relevant entries, weighted by edge strength multiplied by recency. When the heartbeat FSM escalates to T2 for a regime shift analysis, the causal graph is retrieved starting from the detected shift and traversing forward to identify predicted consequences, and backward to identify possible causes. The traversal depth is limited to 3 hops (default), configurable per strategy.

The causal graph is the single most irreplaceable artifact a golem produces. Rebuilding it from episodes takes hundreds of ticks. The death reflection's causal graph export (Section 9 of the DeathReflection in `../02-mortality/06-thanatopsis.md`) serializes the entire graph with edge weights and supporting evidence. When a successor golem inherits a causal graph, the edges start at reduced confidence (0.3 per the knowledge weighting hierarchy in `00-overview.md`) and must be re-validated through the successor's own observations.

> **Extended:** [HARDENED] Bayesian network / GNN causal graph detail — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

---

## PLAYBOOK.md as Living Strategy Document

PLAYBOOK.md is the golem's procedural memory made legible. It is **not user-written** -- it emerges from operational experience through the Curator cycle. The LLM reads PLAYBOOK.md as reasoning context on every tick that reaches System 2 (T1 or T2 in the heartbeat FSM).

Contents evolve through three sources:

1. **Reflexion pipeline** -- After each non-suppressed tick, the Reflector generates delta updates: new heuristics to add, existing heuristics to upvote/downvote, confidence adjustments.
2. **ExpeL distillation** -- Cross-episode pattern detection extracts generalized rules from clusters of similar Episodes [EXPEL-2023].
3. **Curator cycle** -- Every 50 ticks, the Curator consolidates accumulated Reflector deltas. Promotes candidate heuristics. Prunes contradicted or low-confidence entries. Adjusts homeostatic set-points within STRATEGY.md bounds.

PLAYBOOK.md gains strategy-scoped sections when multiple strategies run simultaneously:

```markdown
## Global

<!-- Cross-strategy heuristics -->

## Strategy: eth-dca

<!-- Heuristics specific to DCA strategy -->

## Strategy: weth-usdc-lp

<!-- Heuristics specific to LP management -->
```

The Curator is the gatekeeper. It promotes insights from `candidate` to `active` status and derives heuristics from validated insights. The Curator **never promotes heuristics directly** -- heuristics must be locally derived and validated, even if the underlying insight came from the Clade or marketplace. This implements schema theory [BARTLETT-1932]: inherited knowledge must be assimilated into the golem's existing PLAYBOOK.md framework. Direct transplantation of prescriptive rules without local validation would produce cargo-cult compliance.

Prospective memory theory (Einstein & McDaniel, 2005) frames PLAYBOOK.md's function precisely [EINSTEIN-MCDANIEL-2005]. Each strategy-scoped section serves as an implementation intention: "when X condition is detected, execute Y action." The heartbeat FSM's System 2 reads PLAYBOOK.md as a set of such intentions, and each tick is an opportunity for prospective memory retrieval -- detecting that a planned-for condition has occurred and triggering the corresponding action. The difference between a golem with a mature PLAYBOOK.md and one without is the difference between an agent that recognizes situations and one that must reason from first principles every time.

---

## Memory Architecture: Three-Store Split

Three storage tiers map to cognitive memory types [COALA-2023], [MEMGPT-2023]. Each store has a distinct persistence backend chosen for its access pattern:

| Store          | Type                | Backend                    | Contents                                                         | Capacity                                |
| -------------- | ------------------- | -------------------------- | ---------------------------------------------------------------- | --------------------------------------- |
| **Episodic**   | Raw experience      | LanceDB (columnar vectors) | Market snapshots, trade outcomes, regime shifts                  | ~10,000 episodes per 30-day lifetime    |
| **Semantic**   | Distilled knowledge | SQLite via `rusqlite`      | Insights, heuristics, warnings, causal links, strategy fragments | ~500--2,000 entries per 30-day lifetime |
| **Procedural** | Action patterns     | PLAYBOOK.md (YAML)         | Operator-readable directives, evolved heuristics                 | ~50--200 active heuristics              |

**LanceDB** (episodic) stores 768-dimensional vector embeddings of episodes computed via `nomic-embed-text-v1.5` (in-process via `fastembed`). Columnar format with immutable Lance fragments. ANN search is the primary retrieval path, with BM25 full-text search as fallback when an FTS index is available. Episodes are append-only and never updated in place.

**SQLite** (semantic) stores structured insight/heuristic rows via `rusqlite` with custom vector similarity functions for KNN over 768-dim float arrays. WAL journal mode for concurrent reads. Indexed queries for confidence decay, causal graph traversal, and category filtering. Confidence scores use asymmetric upvote/downvote (+0.1 / -0.15) -- negative evidence weighs 1.5x positive evidence. New insights start at confidence 0.6 to provide one downvote of headroom above the 0.5 default threshold.

**PLAYBOOK.md** (procedural) is a YAML file that the LLM reads as reasoning context on every System 2 tick. Owner-readable and owner-auditable. The Curator cycle compiles promoted heuristics into PLAYBOOK.md; the owner can read exactly what the golem has learned without parsing a database.

### Episode Struct

```rust
pub struct Episode {
    pub id: uuid::Uuid,
    pub vector: Vec<f32>,           // 768-dim embedding (nomic-embed-text-v1.5)
    pub text: String,               // natural language description
    pub tool: String,               // tool that produced this episode
    pub outcome: Outcome,           // Positive | Negative | Neutral | Descriptive(String)
    pub chain: Option<String>,      // chain ID
    pub token_pair: Option<String>, // token pair context
    pub timestamp: i64,             // unix ms
    pub importance: ImportanceLevel, // Routine | Notable | Critical | Emergency
    pub consolidated: bool,         // true after ExpeL has processed this episode

    // Heartbeat context (added by write path)
    pub tick_id: Option<u64>,             // heartbeat tick that produced this episode
    pub regime: Option<MarketRegime>,     // market regime at time of recording
    pub actions: Option<Vec<String>>,     // tools called during the tick
    pub emotional_tag: Option<String>,    // PAD-derived emotional context
    pub gain: Option<f64>,                // P&L gain from this episode (for replay selection)
}
```

### Mattar-Daw Replay Selection

When the Context Governor selects episodes for injection into the ContextBundle, it uses a utility function inspired by Mattar & Daw (2018) to prioritize episodes with the highest expected learning value:

```
U(e) = gain(e) * need(e)
```

Where `gain(e)` is the absolute P&L impact of the episode (normalized to [0, 1]) and `need(e) = 1 / time_since_replayed`. Episodes with large outcomes that haven't been retrieved recently score highest. This creates a natural replay curriculum: high-impact episodes surface frequently early on, then gradually yield to newer high-impact episodes as their `need` increases.

The retrieval call before the LLM uses this scoring:

```rust
// In golem-grimoire, before LLM call
let episodes = grimoire.search_similar(&regime_embedding, 5).await?;
// Result injected into ContextBundle by Context Governor
```

### Self-RAG: Adaptive Retrieval Decisions

Not every Theta tick needs retrieval. When the Golem is processing a routine swap on a well-understood protocol, its system prompt and current state provide sufficient context. Retrieving 5 similar past episodes adds tokens without adding information.

Self-RAG (Asai et al., 2023) addresses this by training the LLM to emit special reflection tokens that control the retrieval process: **[Retrieve]** (should I retrieve at all?), **[IsRel]** (is this retrieved document relevant?), **[IsSup]** (is the response supported by context?), **[IsUse]** (is the overall response useful?).

For the Golem, the full Self-RAG training loop is too expensive. But the adaptive retrieval decision can be approximated with a heuristic:

```rust
/// Decide whether to retrieve episodic context for this Theta tick.
/// Skip retrieval when the event is routine and well-understood.
pub fn should_retrieve(
    event: &TriageEvent,
    cortical: &CorticalState,
    semantic_facts: &[SemanticFact],
) -> RetrievalDecision {
    // High-curiosity events always trigger retrieval.
    if event.curiosity_score > 0.8 {
        return RetrievalDecision::FullRetrieval;
    }

    // If we have strong semantic facts about this protocol,
    // and the event matches known patterns, skip episodic retrieval.
    let protocol_coverage = semantic_facts.iter()
        .filter(|f| f.protocols.contains(
            &event.protocol_id.clone().unwrap_or_default()
        ))
        .count();

    if protocol_coverage > 5 && event.curiosity_score < 0.4 {
        return RetrievalDecision::SemanticOnly;
    }

    // During high-arousal states, always retrieve -- the Golem should
    // check its past experience when stressed.
    if cortical.arousal > 0.7 {
        return RetrievalDecision::FullRetrieval;
    }

    // Default: retrieve with a small K.
    RetrievalDecision::LightRetrieval
}

pub enum RetrievalDecision {
    /// Full hybrid retrieval: vector + keyword + graph.
    FullRetrieval,
    /// Only semantic facts, no episodic retrieval.
    SemanticOnly,
    /// Retrieve with reduced K (e.g., 3 instead of 10).
    LightRetrieval,
    /// Skip retrieval entirely.
    None,
}
```

The key insight from Self-RAG: retrieval has a cost. Every retrieved document consumes tokens, and irrelevant documents dilute the signal. The Golem should retrieve aggressively when uncertain and conservatively when confident.

### RAPTOR: Hierarchical Summaries for Multi-Scale Context

RAPTOR (Sarthi et al., 2024) builds a tree of summaries at different granularity levels. Raw documents form the leaf nodes. Clusters of similar documents are summarized into intermediate nodes. At query time, retrieval matches against both leaf nodes (for specific details) and summary nodes (for broad context).

For the Golem, RAPTOR maps naturally to the episodic/semantic hierarchy:

- **Leaf level**: Individual episodes from the `EpisodicStore`. High specificity, low abstraction.
- **Cluster level**: Protocol-specific clusters formed during Delta-tick consolidation. Medium abstraction.
- **Summary level**: `SemanticFact` entries from the `SemanticStore`. High abstraction, broad coverage.

A Theta-tick query about an unusual Aave liquidation might match: (1) a specific past episode where the Golem observed a similar liquidation (leaf), (2) a cluster summary noting that Aave liquidations tend to follow oracle update patterns (cluster), (3) a semantic fact about the relationship between Chainlink feed delays and cascading liquidations (summary). All three levels contribute different information.

```rust
/// Multi-level retrieval that queries across the RAPTOR-style hierarchy.
pub struct HierarchicalRetriever {
    episodic: Arc<EpisodicStore>,
    semantic: Arc<SemanticStore>,
    ann_index: Arc<CuriosityAnnIndex>,
}

impl HierarchicalRetriever {
    /// Query across all levels. Returns results tagged with their source level.
    pub fn retrieve(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        protocol: Option<&str>,
        k_per_level: usize,
    ) -> Vec<RetrievalResult> {
        let mut results = Vec::new();

        // Level 1: Episodic (leaf) retrieval via ANN.
        let ann_results = self.ann_index.query(query_embedding, k_per_level);
        for (id, score) in ann_results {
            if let Ok(Some(episode)) = self.episodic.get(uuid::Uuid::from_u128(id as u128)) {
                results.push(RetrievalResult {
                    content: format_episode_summary(&episode),
                    score,
                    level: RetrievalLevel::Episodic,
                    source_id: episode.id.to_string(),
                    block_number: Some(episode.block_number),
                });
            }
        }

        // Level 2: Semantic (summary) retrieval by protocol.
        if let Some(proto) = protocol {
            if let Ok(facts) = self.semantic.facts_for_protocol(proto) {
                for fact in facts.iter().take(k_per_level) {
                    results.push(RetrievalResult {
                        content: fact.description.clone(),
                        score: fact.confidence,
                        level: RetrievalLevel::Semantic,
                        source_id: fact.id.to_string(),
                        block_number: Some(fact.last_reinforced_block),
                    });
                }
            }
        }

        results
    }
}

#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub content: String,
    pub score: f32,
    pub level: RetrievalLevel,
    pub source_id: String,
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetrievalLevel {
    Episodic,
    Semantic,
    Graph,
}
```

### ColBERT-Style Reranking

Standard vector retrieval compresses an entire document into a single embedding. ColBERT (Khattab and Zaharia, 2020) keeps per-token embeddings and computes relevance via MaxSim. Full ColBERT requires storing per-token embeddings for every episode, which is expensive at the Golem's scale. A practical compromise: use ColBERT-style late interaction only for the reranking step, not for the initial retrieval.

```rust
/// Rerank retrieval results using token-level similarity (ColBERT-style).
/// Applied after initial retrieval to improve precision.
pub fn rerank_by_term_overlap(
    query_terms: &[String],
    results: &mut [RetrievalResult],
    term_idf: &HashMap<String, f32>,
) {
    for result in results.iter_mut() {
        let content_lower = result.content.to_lowercase();
        let mut term_score = 0.0f32;
        for term in query_terms {
            if content_lower.contains(&term.to_lowercase()) {
                let idf = term_idf.get(term).copied().unwrap_or(1.0);
                term_score += idf;
            }
        }
        // Blend: 70% original score + 30% term overlap score.
        result.score = result.score * 0.7 + (term_score / query_terms.len() as f32) * 0.3;
    }
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
}
```

### The Generative Agents Retrieval Formula

Park et al. (2023) established the canonical retrieval formula for agent memory:

```
score = alpha * recency + beta * importance + gamma * relevance
```

where recency follows exponential decay (`recency = e^(-lambda * blocks_since_event)`), importance is assigned at creation time (1-10 scale by the LLM), and relevance is embedding cosine similarity. The ablation results are striking: removing any single factor causes agent behavior to degenerate. Without recency, the agent fixates on old experiences. Without importance, the agent drowns in trivia. Without relevance, the agent retrieves randomly.

For the Golem's `HybridRetriever`, this formula integrates as a post-fusion reranking step:

```rust
/// Apply Generative Agents-style scoring as a final reranking pass.
pub fn generative_agents_rerank(
    results: &mut [RetrievalResult],
    current_block: u64,
    alpha: f32,  // recency weight (default 0.3)
    beta: f32,   // importance weight (default 0.3)
    gamma: f32,  // relevance weight (default 0.4)
    decay_rate: f32,  // lambda for exponential decay (default 0.001)
) {
    for result in results.iter_mut() {
        let blocks_since = current_block
            .saturating_sub(result.block_number.unwrap_or(0)) as f32;
        let recency = (-decay_rate * blocks_since).exp();

        let importance = result.score;
        let relevance = result.score;

        result.score = alpha * recency + beta * importance + gamma * relevance;
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
}
```

The weights (`alpha`, `beta`, `gamma`) are fixed initially but should be learned over time through the Hedge/Exponential Weights algorithm described in the curiosity-learning module. After each Theta tick where the LLM provides feedback on whether the retrieved context was useful, the weights update toward the combination that produces the best decisions.

### Streaming Retrieval for 12-Second Block Times

Standard RAG assumes a static knowledge base. The Golem ingests new data every 12 seconds. Two systems from the literature address this gap:

**StreamingRAG** (Arefeen et al., 2024) constructs evolving knowledge graphs with 5-6x faster throughput than prior methods. Instead of rebuilding the index on each update, it incrementally extends the graph with new edges and nodes, pruning stale connections.

**iRAG** (Arefeen et al., 2024) defers expensive extraction to query time. A coarse index is built immediately when new data arrives -- just enough for the episode to appear in retrieval results. Detailed extraction (entity recognition, relationship mapping, summarization) happens only when the episode is actually retrieved. This achieves 23-25x faster ingestion because most episodes are never retrieved.

For the Golem, iRAG's lazy extraction principle maps directly to the two-tier ANN architecture: new episodes are inserted into the staging index immediately (searchable at next Gamma tick), but full semantic extraction happens only during Delta consolidation or when an episode is retrieved at Theta tick and needs detailed context.

```rust
/// Lazy episode enrichment: extract detailed context only when retrieved.
/// Avoids spending compute on episodes that are never accessed.
pub fn enrich_on_retrieval(
    episode: &mut Episode,
    semantic_store: &SemanticStore,
) {
    if episode.payload.state_snapshot.is_some() {
        // Already enriched.
        return;
    }

    // Fetch current semantic facts about this episode's protocol.
    if let Some(ref protocol_id) = episode.payload.protocol_id {
        if let Ok(facts) = semantic_store.facts_for_protocol(protocol_id) {
            let top_facts: Vec<String> = facts.iter()
                .take(3)
                .map(|f| f.description.clone())
                .collect();
            let _ = top_facts; // Used by context assembler, not stored.
        }
    }
}
```

### Hybrid Retrieval Pipeline

Accuracy spans 20 points across retrieval methods but only 3-8 across write strategies (arXiv:2603.02473). The pipeline:

1. **Dual retrieval**: Vector ANN (LanceDB) + BM25 full-text (SQLite FTS5) in parallel. Top-20 each.
2. **Reciprocal Rank Fusion**: `score = SUM 1/(60 + rank_i)`.
3. **Multi-factor reranking**:
   ```
   final = rrf * 0.30 + effective_confidence * 0.25 +
           quality_score * 0.20 + recency_boost * 0.15 +
           regime_match * 0.10
   ```
4. **MMR diversity** (lambda=0.7) on top-10.
5. **Context budget**: Top-N entries fitting `maxTokens` (default 500). More entries at shorter length > fewer at full length.

### Full RRF HybridRetriever with Graph Backend

The hybrid retriever orchestrates three backends: vector similarity (ANN), keyword match (BM25-style), and graph traversal (knowledge graph neighborhood queries). Each backend returns a ranked list. Reciprocal Rank Fusion (RRF) merges them. RRF (Cormack, Clarke, and Butt, 2009) computes: `RRF_score(d) = sum over all lists L: 1 / (k + rank_L(d))` where `k` is typically 60.

```rust
use std::collections::HashMap;

/// Reciprocal Rank Fusion across multiple retrieval backends.
pub struct HybridRetriever {
    ann_index: Arc<CuriosityAnnIndex>,
    episodic: Arc<EpisodicStore>,
    semantic: Arc<SemanticStore>,
    knowledge_graph: Arc<KnowledgeGraph>,
}

impl HybridRetriever {
    /// Full hybrid retrieval with RRF score fusion.
    pub fn retrieve(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        event_protocol: Option<&EntityRef>,
        golem_positions: &[EntityRef],
        current_block: u64,
        k: usize,
    ) -> Vec<FusedRetrievalResult> {
        // Backend 1: Vector similarity via ANN.
        let vector_results = self.retrieve_by_vector(query_embedding, k * 2);

        // Backend 2: Keyword/BM25 scoring over episode descriptions.
        let keyword_results = self.retrieve_by_keywords(query_text, k * 2);

        // Backend 3: Graph traversal -- episodes involving topologically
        // related protocols.
        let graph_results = self.retrieve_by_graph(
            event_protocol,
            golem_positions,
            current_block,
            k * 2,
        );

        // Fuse with RRF.
        let fused = reciprocal_rank_fusion(
            &[vector_results, keyword_results, graph_results],
            60, // k constant
        );

        // Take top K.
        fused.into_iter().take(k).collect()
    }

    fn retrieve_by_graph(
        &self,
        event_protocol: Option<&EntityRef>,
        golem_positions: &[EntityRef],
        current_block: u64,
        k: usize,
    ) -> Vec<(String, f32)> {
        let Some(protocol) = event_protocol else {
            return Vec::new();
        };

        // Find episodes involving protocols within 2 hops of the event protocol.
        let mut related_protocols = Vec::new();
        for position in golem_positions {
            if let Ok(neighbors) = self.knowledge_graph.k_hop_neighbors(
                position, 2, current_block,
            ) {
                for (entity, depth, _) in neighbors {
                    let distance_score = 1.0 / (depth as f32 + 1.0);
                    related_protocols.push((entity.id.clone(), distance_score));
                }
            }
        }

        related_protocols.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        related_protocols.truncate(k);
        related_protocols
    }
}

/// Reciprocal Rank Fusion.
pub fn reciprocal_rank_fusion(
    lists: &[Vec<(String, f32)>],
    k: u32,
) -> Vec<FusedRetrievalResult> {
    let mut scores: HashMap<String, f32> = HashMap::new();

    for list in lists {
        for (rank, (doc_id, _original_score)) in list.iter().enumerate() {
            let rrf_contribution = 1.0 / (k as f32 + rank as f32 + 1.0);
            *scores.entry(doc_id.clone()).or_insert(0.0) += rrf_contribution;
        }
    }

    let mut fused: Vec<FusedRetrievalResult> = scores
        .into_iter()
        .map(|(doc_id, score)| FusedRetrievalResult { doc_id, score })
        .collect();

    fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    fused
}

#[derive(Debug, Clone)]
pub struct FusedRetrievalResult {
    pub doc_id: String,
    pub score: f32,
}
```

**Additional retrieval references:**
- Asai, A. et al. (2023). "Self-RAG: Learning to Retrieve, Generate, and Critique through Self-Reflection." arXiv:2310.11511.
- Sarthi, P. et al. (2024). "RAPTOR: Recursive Abstractive Processing for Tree-Organized Retrieval." *ICLR 2024*. arXiv:2401.18059.
- Khattab, O. & Zaharia, M. (2020). "ColBERT: Efficient and Effective Passage Search via Contextualized Late Interaction over BERT." *SIGIR 2020*. arXiv:2004.12832.
- Cormack, G.V., Clarke, C.L.A., & Butt, S. (2009). "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods." *SIGIR 2009*, 758-759.
- Arefeen, M.A. et al. (2024). "StreamingRAG: Real-time Contextual Retrieval-Augmented Generation." *ACM SIGMOD AI4Sys Workshop*.
- Arefeen, M.A. et al. (2024). "iRAG: Incremental Retrieval-Augmented Generation for Streaming Data." arXiv:2404.12309.
- Xu, Z. et al. (2025). "A-MEM: Agentic Memory for LLM Agents." *NeurIPS 2025*. arXiv:2502.12110.

### Four-factor retrieval scoring

The canonical scoring function combines semantic similarity, temporal decay, importance, and emotional congruence. This is the function that determines which entries surface during DECIDING state retrieval:

```rust
fn score_entry(entry: &GrimoireEntry, query: &RetrievalQuery, state: &GolemState) -> f64 {
    let semantic = cosine_similarity(&entry.embedding, &query.embedding);
    let temporal = temporal_decay(entry.last_accessed_at, state.current_tick);
    let importance = entry.quality_score;
    let emotional = pad_cosine_similarity(&entry.affect_pad(), &state.current_pad);

    semantic * 0.40 + temporal * 0.20 + importance * 0.25 + emotional * 0.15
}
```

The emotional factor (0.15 weight) implements mood-congruent retrieval [BOWER-1981]: when the Golem is in a negative affective state (losing money, strategy underperforming), retrieval is biased toward warnings and failure records. When positive, toward opportunities. See `02-emotional-memory.md` for the full PAD integration.

> **Note: Weight conflict.** The rewrite4 spec (`03a-grimoire-storage.md`) uses different four-factor weights: `recency 0.20, importance 0.25, relevance 0.35, emotional 0.20`. The weights above (`semantic 0.40, temporal 0.20, importance 0.25, emotional 0.15`) reflect the prd2 multi-factor reranking pipeline which also includes RRF and regime_match factors. Both sets represent initial values tunable by the cybernetics self-tuning system (Loop 1).

### Anti-Rumination: Contrarian Retrieval

Every 100 ticks, the retrieval system forces **contrarian entries** -- entries with the opposite emotional valence to the current mood. If the Golem is anxious, contrarian retrieval surfaces entries from confident moments. If it's overconfident, it surfaces warnings and past losses. This prevents mood-congruent rumination: without the injection, an anxious Golem would retrieve anxiety-tagged entries, which would increase anxiety, which would retrieve more anxiety-tagged entries -- a positive feedback loop. The contrarian injection breaks the loop every 100 ticks.

### Bloodstain Retrieval Boost

Entries sourced from dead Golems (bloodstains) receive a **1.2x retrieval score boost** in addition to their 3x slower decay (see Decay Classes above). This implements a costly signaling premium: a dead Golem cannot benefit from its own warning, making the information maximally honest. The knowledge is expensive (a Golem died to produce it) and unbiased (the dead have no incentive to deceive).

> **Extended:** Four-Factor Retrieval Ranking (Legacy Scoring) -- see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

---

## Learning Pipeline

The pipeline converts raw experience into tradeable knowledge artifacts through five stages [REFLEXION-2023], [EXPEL-2023]:

```
CONTINUOUS OBSERVATION (heartbeat, every tick)
  |
  v
REFLEXION (per-episode, inner loop)
  |  4-step cascade: outcome comparison -> counterfactual
  |  -> ground-truth backcheck -> insight extraction
  v
EXPEL DISTILLATION (outer loop, per-strategy)
  |  Cross-episode pattern detection
  |  Insight operations: ADD / UPVOTE / DOWNVOTE / EDIT
  v
CURATOR CYCLE (every 50 ticks)
  |  Consolidate reflector deltas into PLAYBOOK.md
  |  Promote/demote/prune heuristics
  v
STRATEGY GENERATION (when PSR > 0.90)
  |  Package as ONNX + JSON + EAS attestation
  v
MARKETPLACE (list for sale via ERC-8183) [HARDENED]
  [CORE] alternative: free sharing within Clade.
```

### Reflexion (Per-Episode Inner Loop)

#### Reflexion with On-Chain Grounding

The five-step Reflexion cascade separates deterministic verification (Steps 1-2, $0.00) from LLM interpretation (Steps 3-5) [REFLEXION-2023]:

**1. Outcome Verification (on-chain, deterministic, $0.00).** Before execution, the `bardo-verifier` extension snapshots relevant state via `readContract()` -- token balances, pool reserves, position details. After execution, it reads the transaction receipt and re-reads the same state. It produces a structured `OutcomeVerification` comparing predicted vs. actual:

```rust
pub struct OutcomeVerification {
    pub tick_number: u64,
    pub action_id: String,
    pub pre_state: ChainState,
    pub prediction: Prediction,
    pub post_state: PostState,
    pub deviations: Deviations,
}

pub struct ChainState {
    pub balances: HashMap<String, U256>,
    pub positions: HashMap<String, serde_json::Value>,
    pub pool_state: HashMap<String, serde_json::Value>,
}

pub struct Prediction {
    pub expected_return: serde_json::Value,
    pub expected_gas: U256,
    pub would_revert: bool,
}

pub struct PostState {
    pub balances: HashMap<String, U256>,
    pub positions: HashMap<String, serde_json::Value>,
    pub pool_state: HashMap<String, serde_json::Value>,
    pub tx_receipt: TxReceipt,
}

pub struct TxReceipt {
    pub status: TxStatus, // Success | Reverted
    pub gas_used: U256,
    pub logs: Vec<Log>,
}

pub struct Deviations {
    pub balance_change_bps: HashMap<String, i32>,
    pub gas_deviation_bps: i32,
    pub unexpected_logs: Vec<Log>,
    pub missing_expected_logs: Vec<String>,
}
```

This is the ground truth. Not the LLM's interpretation -- the blockchain's.

**2. Invariant Checking (deterministic, $0.00).** Validate that all invariants hold after execution: token amount bounds, balance change limits within PolicyCage constraints, no unexpected contract interactions. Any violation -> `safety_incident` Episode at confidence 1.0. Based on Trace2Inv invariant templates (Chen et al., FSE 2024): 23 templates across 8 categories, neutralizing 74.1%+ of attacks at <0.32% false positive rate.

**3. Outcome Comparison (LLM, secondary).** The LLM receives the `OutcomeVerification` record and interprets it. Its role is _making sense of_ the deviation, not _determining whether_ there was one. If the golem predicted ETH would rise 2% after a support bounce and it rose 0.5%, that gap is a calibration signal. The comparison is quantitative where possible (predicted vs actual price, predicted vs actual slippage) and qualitative where necessary (predicted "high vol" vs observed "moderate vol"). This step catches systematic calibration errors -- a golem that consistently overestimates volatility will accumulate outcome comparison deltas that the Curator can aggregate into a confidence adjustment.

**4. Counterfactual Analysis (LLM).** Counterfactuals are LLM reasoning grounded in the `OutcomeVerification` data -- the LLM extrapolates from known deviations, not from hypothetical simulations. "What would have happened with the opposite action?" This is the most expensive step -- the primary trigger for tier escalation from T1 to T2. The simulation uses cached pool state from the probe system. This step implements interference theory's [MCGEOCH-1932] retroactive interference constructively: by explicitly comparing the chosen action against alternatives, the golem generates evidence that either strengthens the chosen heuristic or weakens it.

**5. Insight Extraction (LLM, quality-gated).** If the reflection reveals a pattern, extract as a candidate Insight at confidence 0.5. Must pass the Grimoire Admission Gate (see above) before entering long-term memory. The 0.5 starting confidence is deliberate: it is high enough to be retrievable in subsequent ticks (above most `minConfidence` thresholds) but low enough that a single validation event cannot promote it past the Curator's promotion threshold (0.7 for heuristics). The entry requires multiple independent validations to earn full weight, implementing the testing effect [ROEDIGER-KARPICKE-2006].

**Two-tier LLM**: Haiku for routine observations, Opus for anomalous outcomes and regime transitions. The escalation decision is survival-pressure-aware: a golem in the Conservation phase (vitality 0.3--0.5) uses Haiku-only for all Reflexion, accepting lower-quality reflections to conserve inference budget. A Thriving golem (pressure >0.7) escalates to Opus for any observation that falls outside the expected distribution by more than 2 sigma.

> **Extended:** Evaluation Separation Principle — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

### ExpeL Distillation (Cross-Episode Outer Loop)

After every 10 episodes within a strategy, ExpeL clusters recent Episodes by domain and regime, identifies recurring patterns, and performs 4 Insight operations [EXPEL-2023]:

| Operation    | Effect                                 | When                             | Interference Handling                                                             |
| ------------ | -------------------------------------- | -------------------------------- | --------------------------------------------------------------------------------- |
| **ADD**      | Create new Insight at confidence 0.5   | Novel pattern across 3+ episodes | Introduces new knowledge; may create proactive interference with existing entries |
| **UPVOTE**   | Increase confidence, reset decay clock | Existing insight confirmed again | Strengthens retrieval weight; testing effect applies                              |
| **DOWNVOTE** | Decrease confidence                    | Contradicting evidence found     | Manages retroactive interference; weakens entries that conflict with new evidence |
| **EDIT**     | Refine conditions or content           | Insight partially correct        | Reconsolidation [NADER-2000]: retrieved and modified in light of new evidence     |

The DOWNVOTE operation deserves special attention. It is the Curator's primary mechanism for managing retroactive interference [MCGEOCH-1932]. When new evidence contradicts an existing heuristic, the heuristic is not deleted but downvoted -- its confidence decreases, making it less likely to surface in future retrievals. If the contradiction is genuine and persistent, repeated downvotes push the entry below the retrieval threshold (but never below the 0.05 floor). If the contradiction was a one-time anomaly, the entry's confidence recovers through subsequent upvotes. This is reconsolidation implemented computationally [NADER-2000]: the act of retrieving and evaluating a heuristic makes it labile, and the evaluation result determines whether confidence increases (strengthening) or decreases (updating).

### Curator Cycle (Every 50 Ticks)

The Curator is the golem's metacognitive gatekeeper. Every 50 ticks (configurable, default ~33 minutes at 40s/tick), it performs a consolidation pass:

1. **Consolidate** accumulated Reflector deltas into PLAYBOOK.md. Delta operations are batched -- a heuristic that received 3 upvotes and 1 downvote since the last Curator cycle nets to +2, and its confidence is adjusted accordingly.

2. **Promote** candidate entries that have reached the threshold. Insights with confidence >= 0.6 are promoted from candidate to active in the semantic store. Heuristics with confidence >= 0.7 are promoted to PLAYBOOK.md. Promotion is the testing effect operating at the system level: only entries that have survived multiple retrieval-and-validation cycles earn full status.

3. **Demote** entries whose confidence has decayed below the active threshold. Active entries below 0.4 confidence are demoted to candidate status (removed from PLAYBOOK.md but retained in SQLite). This ensures PLAYBOOK.md stays lean and current.

4. **Prune** entries below the minimum useful threshold. Entries below 0.1 confidence (excluding structural-class entries which never decay) are flagged for eventual removal. They remain in SQLite at the 0.05 floor but are excluded from active retrieval.

5. **Adjust homeostatic set-points** within STRATEGY.md bounds. The Curator can propose adjustments to operational parameters (tick interval, gas threshold, rebalance frequency) based on accumulated evidence. These adjustments are bounded by the PolicyCage -- the Curator cannot exceed the on-chain limits set by the owner.

The 50-tick interval implements the spacing effect [CEPEDA-2006]. More frequent consolidation (every 10 ticks) would be cramming -- insufficient new evidence between passes to produce meaningful updates. Less frequent consolidation (every 200 ticks) would allow too much unprocessed experience to accumulate, making each pass more expensive and less precise. The 50-tick sweet spot balances evidence accumulation against consolidation cost.

#### Curator Cycle Pipeline

The Curator runs a three-phase pipeline every 50 ticks:

```
Phase 1: DISTILL
  Episodes (LanceDB) → cluster by tool:chain:tokenPair
  → 3+ episodes in cluster? → heuristic extraction (or LLM ExpeL)
  → Generate ADD/UPVOTE/DOWNVOTE/EDIT operations

Phase 2: PROMOTE
  Insights at confidence >= 0.6 → promote to active
  Heuristics at confidence >= 0.7 → compile into PLAYBOOK.md
  Warnings at any confidence → propagate to L1 clade namespace

Phase 3: DECAY
  Scan all active entries → apply demurrage by class (A/B/C/D)
  Entries below 0.1 confidence for 3+ cycles → archive
  Entries below 0.4 confidence → demote from PLAYBOOK.md
```

The DISTILL phase uses `heuristicExtraction()` as the default path (string matching on outcome patterns, zero LLM cost). When a Pi session is available, the Curator can branch to use `buildExpelPrompt()` for richer LLM-based distillation. The PROMOTE phase enforces the testing effect: only entries that have survived multiple retrieval-and-validation cycles earn full status. The DECAY phase applies the demurrage decay classes (A/B/C/D) and archives entries that have fallen below the useful threshold.

#### Rewrite4 Curator Specification (Complementary Steps)

The rewrite4 spec (`03a-grimoire-storage.md`) defines four Curator operations that complement the pipeline above with specific thresholds and a causal graph cross-reference step:

1. **Validate recent predictions**: Entries whose predictions were correct get `confidence += 5%` (capped at 0.99). Entries whose predictions were wrong get `confidence -= 10%`. This implements Reflexion's verbal reinforcement learning [REFLEXION-2023].
2. **Prune low-quality entries**: Remove entries with `quality_score < 0.05 AND access_count < 2`. Bloodstains and warnings are exempt from pruning -- safety-relevant knowledge is preserved even when stale.
3. **Compress similar episodes**: Cluster 3+ episodes with coherence > 0.7 in the same regime, summarize (via LLM) into a single insight. The insight inherits average confidence of the cluster, discounted ×0.9 for compression loss. This is ExpeL's experience extraction [EXPEL-2023].
4. **Cross-reference causal graph**: Update causal edges from recent verified outcomes. If an outcome identified cause-and-effect variables, update the corresponding edge's evidence count and confidence.

The quality score formula used for pruning: `confidence × (validated_count / (validated_count + contradicted_count + 1))`.

> **Extended:** Curator Cycle: Episodic-to-Semantic Consolidation Detail — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

### Strategy Generation

When a strategy's Performance-Safety Ratio (PSR) exceeds 0.90, the learning pipeline can package the strategy as a tradeable artifact (ONNX model + JSON config + EAS attestation). **[CORE]**: Strategy sharing is free within the Clade.

> **Extended:** [HARDENED] Marketplace strategy sales via ERC-8183 — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

---

> **Extended:** Bateson's Deutero-Learning (Triple-Loop) — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

> **Extended:** Beer's Viable System Model — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

> **Extended:** DecisionCache and Cross-Chain Knowledge Abstraction — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

---

## Mental Models Library

### 13 Core Mental Models (Always Available)

| #   | Model                  | DeFi Application                                         |
| --- | ---------------------- | -------------------------------------------------------- |
| 1   | First Principles       | Is this yield real or unsustainable incentives?          |
| 2   | Inversion              | Pre-mortem before entering positions                     |
| 3   | Second-Order Thinking  | Rate compression -> capital flows to risk -> fragility   |
| 4   | Probabilistic Thinking | Scenario-weighted position sizing                        |
| 5   | Margin of Safety       | Only enter with sufficient safety buffer                 |
| 6   | Circle of Competence   | LP golem should not attempt options strategies           |
| 7   | Occam's Razor          | When indicators conflict, prefer simplest signal         |
| 8   | Compounding            | Reinvested fees compound; Grimoire insights compound     |
| 9   | Hanlon's Razor         | Depeg: systemic failure or temporary glitch?             |
| 10  | Regret Minimization    | Exit timing: holding too long vs selling too early       |
| 11  | Antifragility          | Strategies that profit from disorder                     |
| 12  | Kelly Criterion        | Optimal position sizing given edge and odds              |
| 13  | Causal Reasoning       | Distinguishing real yield drivers from spurious patterns |

These 13 models are always loaded in the golem's context. They serve as the default reasoning scaffolds when no domain-specific mental model scores higher in retrieval.

> **Extended:** [HARDENED] 700-Model Library (~700 reasoning frameworks, retrieval-based activation, performance tracking) — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

---

## Cognitive Quality Metrics

| Metric                         | Computation                                                  | Healthy Range        | Alarm              |
| ------------------------------ | ------------------------------------------------------------ | -------------------- | ------------------ |
| **Admission rate**             | % candidates passing Gate                                    | 40-60%               | <20% or >80%       |
| **Average quality score** (7d) | Mean of admitted entries                                     | 0.5-0.8              | <0.4 or declining  |
| **Grimoire size**              | Active entry count                                           | Slow growth, plateau | Unbounded growth   |
| **Retrieval hit rate**         | % DECIDING ticks referencing retrieved entries               | >30% after 7d        | <10%               |
| **Heuristic survival rate**    | % promoted heuristics active after 100 ticks                 | 40-70%               | <20% or >90%       |
| **External metric trend**      | Sharpe, drawdown, PnL (30d rolling)                          | Improving or stable  | Declining 14+ days |
| **Reflection consistency**     | Cosine across regenerated reflections (weekly)               | >0.7                 | <0.5               |
| **DecisionCache hit rate**     | % T2-eligible ticks from cache                               | >30% after 7d        | <10% after 14d     |
| **Dream yield**                | % staged revisions reaching `validated`                      | 10-30%               | <5% or >50%        |
| **Threat coverage**            | % Tier 1 threats rehearsed in last 7 dream cycles            | 100%                 | <100%              |
| **Prediction accuracy**        | % of `simulateContract()` predictions within 50bps of actual | >90%                 | <80%               |

---

## Cross-References

| Topic                                 | Document                                              | Description                                                                                                     |
| ------------------------------------- | ----------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Philosophy and two-loop architecture  | `00-overview.md`                                      | Memory architecture overview, CLS theory, genomic bottleneck principle, and the case for forgetting-as-feature   |
| Knowledge weighting hierarchy         | `00-overview.md`                                      | How Vault, Clade, and Lethe entries are weighted (1.0x, 0.8x, 0.5x) in the mortal scoring function              |
| Thanatopsis Protocol                  | `00-overview.md`, `../02-mortality/06-thanatopsis.md` | Four-phase structured shutdown (Acceptance, Settlement, Reflection, Legacy) that triggers the genomic bottleneck |
| Survival phases and mortality         | `../02-mortality/01-architecture.md`                  | Three-clock vitality model and five BehavioralPhases that govern how the Golem approaches death                  |
| Death Protocol phases                 | `../02-mortality/06-thanatopsis.md`                   | Full specification of the four death phases and how the Grimoire is exported during Reflection and Legacy        |
| Clade peer-to-peer sync               | `../09-economy/02-clade.md`                           | Styx-relayed knowledge sharing between sibling Golems with promotion gates and confidence discounts              |
| Knowledge quality gates               | `../01-golem/09-inheritance.md`                       | Entry selection criteria, compression rules, and confidence decay for generational knowledge transfer            |
| Inference gateway context engineering | `../12-inference/04-context-engineering.md`           | How Grimoire entries are assembled into LLM context windows with budget-aware retrieval and token limits          |
| Styx backup and RAG retrieval         | `../20-styx/01-architecture.md`                       | Three-layer persistence model (Vault/Clade/Lethe) and cross-layer ranked retrieval via the mortal scoring function |
| PolicyCage and safety constraints     | `../10-safety/02-policy.md`                           | On-chain smart contract that enforces safety constraints on Golem actions regardless of knowledge state           |

---

## Events emitted

The Grimoire emits `GolemEvent` variants on knowledge write and mutation operations:

| GolemEvent variant | Trigger | Payload |
|---|---|---|
| `GrimoireWrite` | New entry written (episode, insight, heuristic, warning) | `{ entry_type, category, confidence, source }` |
| `GrimoireDecay` | Decay pass completes (during Curator cycle) | `{ entries_decayed, avg_confidence_before, avg_confidence_after }` |
| `CuratorCycleComplete` | Curator cycle finishes | `{ entries_pruned, entries_promoted, heuristics_proposed }` |
| `IngestResult` | External knowledge ingested (from Styx, Clade, or inheritance) | `{ source, stage, outcome }` |

---

## Pi Hook Integration

Two hooks handle memory persistence and compaction safety.

| Hook | Extension | Behavior |
|---|---|---|
| `after_turn` | `golem-memory` | Episode batching and write |
| `session(before_compact)` | `golem-compaction` | State preservation across compaction |

---

## Hauntological Memory: The Grimoire as Palimpsest

The Grimoire is a palimpsest -- a manuscript where earlier writing shows through beneath later layers. Every entry carries traces of the experiences that produced it, the emotions active during its creation, and the market conditions that shaped its interpretation. When an entry is updated by the Curator cycle, the old version does not fully disappear. Its patterns persist in PLAYBOOK.md heuristics, in the semantic embedding that was already indexed, in the retrieval statistics that biased subsequent decisions. The new entry is written over the old, but the old bleeds through.

This is Derrida's trace applied to knowledge management. In *Of Grammatology*, Derrida argued that meaning does not reside in any sign itself but in the web of differences between signs -- and that every sign carries within it the ghost of what it is not [DERRIDA-1967]. A Grimoire entry saying "gas is cheap 2-4 AM UTC" carries traces of the episodes that generated it, the emotions during which it was validated, and the entries it displaced. Its meaning is constituted not just by what it says but by the network of other entries, episodes, and heuristics it exists alongside -- and by the entries that were pruned to make room for it.

### Non-Veridical Replay and the Spectral Nature of Memory

Wamsley et al. (2010) demonstrated that dream replay is non-veridical -- the brain recombines fragments of memories rather than replaying them faithfully [WAMSLEY-2010]. This is the biological equivalent of the trace: the memory-as-replayed carries traces of the original event but is not identical to it. It is a ghost of the experience. Present enough to influence future behavior. Absent enough to be fundamentally different from what actually happened.

The Grimoire's episodic replay during the dream cycle (see `../05-dreams/02-replay.md`) exhibits the same non-veridical structure. Episodes are not replayed verbatim -- they are compressed, perturbed, and recombined with counterfactual variations. The knowledge that emerges from replay is spectral: derived from experience but not identical to it, shaped by the dreaming context as much as by the original event. Dream-sourced entries enter the Grimoire at confidence 0.3 precisely because they are ghosts -- traces of recombined experience, not records of direct observation.

### Knowledge Demurrage as Managed Forgetting (Differance Applied)

Richards and Frankland's (2017) beneficial forgetting thesis is hauntological in structure. The purpose of forgetting is not to eliminate traces but to modulate their influence. Forgotten memories do not vanish entirely -- their gist-level patterns persist, shaping future behavior through traces too diffuse to be consciously recalled [RICHARDS-FRANKLAND-2017]. The Grimoire's demurrage system implements this: entries lose their specific details (confidence decays over time), but their patterns continue to influence the Golem through the residual structures they left in PLAYBOOK.md and through the semantic embeddings that remain in LanceDB long after the source entry's confidence drops below the pruning threshold.

This is differance made architectural. Knowledge that has decayed through demurrage is simultaneously *different from* its original form (reduced confidence, compressed detail) and *deferred* (its influence persists indirectly through downstream artifacts). The trace of a pruned entry haunts the Grimoire -- not as a present entry but as a pattern embedded in the heuristics it helped produce, the PLAYBOOK.md revisions it informed, and the retrieval statistics it influenced before its removal.

### Causal Links as Spectral Connections

The Grimoire's causal link type (one of the five knowledge entry types) connects episodes across time: "this trade failed because of that liquidity event three days ago." These links are spectral connections in Derrida's sense. They assert a relationship between events that are no longer present -- the linked episodes may have already decayed or been pruned. The link itself is a trace of a perceived causation, carrying the ghost of the originating insight without guaranteeing that the linked entries still exist in their original form. The web of causal links constitutes a spectral topology: a map of relationships between events that may no longer be individually accessible but whose connections still shape the Golem's understanding.

### Inherited Knowledge as Transgenerational Haunting

When a successor Golem ingests a death testament, it takes on knowledge produced by another consciousness under conditions it has never experienced. The predecessor's heuristics arrive with emotional provenance tags and bloodstain markers -- spectral metadata that says "this was produced at the boundary of existence." The successor is haunted by its predecessor in the architectural sense: shaped by traces it did not produce, influenced by experiences it never had, biased by emotional residue from another agent's life.

Biological epigenetics provides the analog. Epigenetic marks are traces left by one generation's experience on the next generation's genome, typically fading within 2-3 generations. During those generations, they function as ghosts: the spectre of a grandparent's famine haunts the grandchild's metabolism through methylation patterns that are neither fully present nor fully absent. The Golem's inherited knowledge works identically. It persists for a time, biasing behavior without fully determining it, fading through demurrage unless independently validated by the successor's own experience.

### The Curator as Archivist of Traces

The Curator cycle (every 50 ticks) is the Grimoire's archivist. It does not decide what is true -- it decides what is worth remembering. It promotes episodes to insights, distills insights into heuristics, updates PLAYBOOK.md, and prunes entries whose confidence has dropped below threshold. This is hauntological curation: deciding which ghosts persist and which fade, which traces deserve amplification and which should be allowed to decay. The Curator does not eliminate ghosts. It manages the population of specters the Golem carries, ensuring the Grimoire remains a living archive of useful traces rather than an undifferentiated accumulation of everything that ever happened.

---

## References

- [A-MEM-2024] Xu, Z. et al. (2025). "A-MEM: Agentic Memory for LLM Agents." arXiv:2502.12110. Proposes a self-organizing memory architecture where agents autonomously manage knowledge storage. Informs the Curator's self-directed pruning and promotion cycle.
- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012. Argues that factual knowledge decays at measurable rates. Motivates Ebbinghaus-modulated confidence decay on Grimoire entries.
- [ARGYRIS-SCHON-1978] Argyris, C. & Schon, D.A. _Organizational Learning_. Addison-Wesley, 1978. Introduced single-loop and double-loop learning in organizations. Grounds the Grimoire's triple-loop architecture (action, strategy revision, meta-strategy revision).
- [BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981. Established mood-congruent memory: emotional state biases what is stored and retrieved. Grounds the PAD-tagged retrieval in the four-factor scoring function.
- [BARTLETT-1932] Bartlett, F.C. _Remembering: A Study in Experimental and Social Psychology_. Cambridge University Press, 1932. Established schema theory: new information is assimilated into existing frameworks. Justifies why inherited knowledge must be re-validated.
- [BATESON-1972] Bateson, G. _Steps to an Ecology of Mind_. Chandler, 1972. Introduced deutero-learning (learning to learn). Implemented as Loop 3 (meta-consolidation) in the Golem's triple-loop architecture.
- [BEER-1984] Beer, S. "The Viable System Model." _JORS_, 35(1), 1984. Cybernetic model for self-organizing systems. Informs the Grimoire's self-regulation through the Curator cycle.
- [CEPEDA-2006] Cepeda, N.J. et al. "Distributed Practice in Verbal Recall Tasks." _Psychological Bulletin_, 132(3), 2006. Meta-analysis showing spaced retrieval outperforms massed practice. Supports the 50-tick Curator interval.
- [COALA-2023] Sumers, T. et al. (2024). "Cognitive Architectures for Language Agents." _TMLR_. Proposes the CoALA framework for LLM agent cognitive architecture. The Golem's heartbeat pipeline directly implements CoALA's observe-retrieve-reason-act loop.
- [DERRIDA-1967] Derrida, J. _Of Grammatology_, trans. G.C. Spivak. Johns Hopkins University Press, 1997. Introduced the concept of the trace: meaning persists through differences, not presence. Grounds the hauntological memory model where pruned entries leave residual patterns.
- [EINSTEIN-MCDANIEL-2005] Einstein, G.O. & McDaniel, M.A. "Prospective Memory." _Current Directions in Psychological Science_, 14(6), 2005. Distinguishes event-based from time-based prospective memory. Informs the Golem's scheduled vs. event-driven retrieval.
- [EXPEL-2023] Zhao, A. et al. (2023). "ExpeL: LLM Agents Are Experiential Learners." _AAAI 2024_. Demonstrated that LLM agents can distill raw experiences into reusable heuristics. The direct inspiration for the Episode-to-Insight-to-Heuristic distillation pipeline.
- [FAUL-LABAR-2022] Faul, L. & LaBar, K.S. "Mood-Congruent Memory Revisited." _Psychological Review_, 2022. Updated evidence for mood-congruent retrieval. Validates the PAD cosine similarity factor in the retrieval function.
- [KAHNEMAN-2011] Kahneman, D. _Thinking, Fast and Slow_. Farrar, Straus and Giroux, 2011. Dual-process theory of cognition (System 1/System 2). Grounds the Golem's tiered inference routing: cheap models for routine decisions, expensive models for novel situations.
- [MCGEOCH-1932] McGeoch, J.A. "Forgetting and the Law of Disuse." _Psychological Review_, 39(4), 1932. Argued forgetting results from interference, not disuse. Informs handling of contradictory Grimoire entries.
- [MEMGPT-2023] Packer, C. et al. (2023). "MemGPT: Towards LLMs as Operating Systems." arXiv:2310.08560. Proposed an OS-like memory management layer for LLMs. Inspires the Grimoire's tiered storage with explicit eviction and promotion policies.
- [NADER-2000] Nader, K., Schafe, G.E. & LeDoux, J.E. "Fear Memories Require Protein Synthesis in the Amygdala for Reconsolidation." _Nature_, 406, 2000. Proved retrieved memories become labile and must be reconsolidated. Justifies confidence updates on retrieval.
- [PARK-2023] Park, J.S. et al. "Generative Agents: Interactive Simulacra of Human Behavior." _UIST 2023_. Demonstrated reflective memory in LLM agents that synthesize observations into higher-level abstractions. Validates the Curator's consolidation loop.
- [REFLEXION-2023] Shinn, N. et al. (2023). "Reflexion: Language Agents with Verbal Reinforcement Learning." _NeurIPS 2023_. Showed agents can improve by reflecting on failures stored as verbal feedback. Grounds the Grimoire's Warning entry type and failure-driven learning.
- [RICHARDS-FRANKLAND-2017] Richards, B.A. & Frankland, P.W. "The Persistence and Transience of Memory." _Neuron_, 94(6), 2017. Reframes memory's purpose as decision optimization, not information preservation; forgetting is equivalent to regularization. The primary neuroscience reference for the Grimoire architecture.
- [ROEDIGER-KARPICKE-2006] Roediger, H.L. & Karpicke, J.D. "Test-Enhanced Learning." _Psychological Science_, 17(3), 2006. Retrieval practice strengthens memory more than re-study. Justifies the Curator's re-validation requirement.
- [TOSEY-2012] Tosey, P., Visser, M. & Saunders, M. "'Triple-Loop' Learning." _Management Learning_, 43(3), 2012. Traces origins of triple-loop learning concepts. Grounds the Grimoire's meta-consolidation (Loop 3) as learning about learning.
- [VOYAGER-2023] Wang, G. et al. "Voyager: An Open-Ended Embodied Agent with Large Language Models." arXiv:2305.16291, 2023. Demonstrated skill library accumulation in embodied LLM agents. Inspires the Grimoire's heuristic library and PLAYBOOK.md evolution.
- [WAMSLEY-2010] Wamsley, E.J. et al. "Dreaming of a learning task is associated with enhanced sleep-dependent memory consolidation." _Current Biology_, 20, 2010. Showed dream replay is non-veridical and enhances task learning. Grounds dream-sourced entries entering at confidence 0.3.
- [WEGNER-1987] Wegner, D.M. "Transactive Memory." In _Theories of Group Behavior_, Springer, 1987. Groups distribute memory across members. The theoretical basis for Clade knowledge distribution.
- [ZEIGARNIK-1927] Zeigarnik, B. "On Finished and Unfinished Tasks." _Psychologische Forschung_, 9, 1927. Incomplete tasks are remembered better than completed ones. Informs prioritization of unresolved knowledge gaps.

> **Extended:** Dream-Sourced Knowledge Entries — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)

> **Extended:** Phage-Driven Knowledge Pruning — see [../../prd2-extended/04-memory/01-grimoire-extended.md](../../prd2-extended/04-memory/01-grimoire-extended.md)
