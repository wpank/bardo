# Memory: Knowledge Ecology for Mortal Agents [SPEC]

> **Version**: 3.0 | **Status**: Draft
>
> **Depends on**: `01-grimoire.md`, `../02-mortality/`, `../12-inference/04-context-engineering.md`

---

> **Reader orientation:** This document describes the memory architecture for Bardo's mortal autonomous DeFi agents, called Golems (single Rust binaries running on micro VMs that trade, lend, and provide liquidity across DeFi protocols). It belongs to the **04-memory** layer, which governs how Golems store, forget, and inherit knowledge. The key concept is Complementary Learning Systems (CLS): a dual-system memory model with fast episodic capture and slow semantic consolidation, compressed through a genomic bottleneck at death. For term definitions, see `prd2/shared/glossary.md`.

## Why this document exists

A first-time reader should understand three things before going further. First, the Golem is a mortal autonomous agent that manages capital across DeFi protocols — trading on Uniswap, lending on Morpho and Aave, providing liquidity, and more. It has a finite USDC budget, a finite lifespan, and it dies. Second, it remembers through a local knowledge system called the Grimoire, which runs entirely on the Golem's VM with no hosted dependencies. Third, this document describes the memory architecture that sits above that local system, extending knowledge across agent lifetimes and across agent fleets.

The competitive moat is here: **knowledge decay is rational only because time is finite.** An immortal agent has no reason to forget. A mortal agent MUST forget efficiently because bad memories crowd out good decisions and waste finite context budget. Every system that ships perpetual-memory agents will discover, too late, that accumulation without pruning is a form of overfitting. The Golem's memory system treats forgetting as a first-class operation, not a failure mode.

---

## The argument in one sentence

A golem that remembers everything is as disabled as one that remembers nothing -- the intelligence is in what it chooses to forget [BORGES-1942], [RICHARDS-FRANKLAND-2017].

---

## Shared terminology

The following terms are used consistently across all Bardo PRDs (Emotions, Mortality, Memory):

| Term                     | Definition                                                                                                                                                      |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **VitalityState**        | Three-component mortality state: `VitalityState { economic: f64, epistemic: f64, stochastic: f64 }`. Multiplicative composition -- any component at zero kills the Golem. |
| **BehavioralPhase**      | Five phases: Thriving (>0.7), Stable (0.5-0.7), Conservation (0.3-0.5), Declining (0.1-0.3), Terminal (<0.1). Derived from vitality score.                      |
| **Thanatopsis Protocol** | Four-phase death protocol: Phase 0 (Acceptance), Phase I (Settle), Phase II (Reflect), Phase III (Legacy).                                                      |
| **Curator**              | 50-tick memory maintenance cycle: validate, prune, compress, cross-reference. Runs in `golem-grimoire`. Emits `CuratorCycleComplete` on each pass.              |
| **PAD Vector**           | Pleasure-Arousal-Dominance affective state: `PadVector { pleasure: f64, arousal: f64, dominance: f64 }`. Each dimension in [-1.0, 1.0].                         |
| **Plutchik Label**       | One of 8 primary emotions: joy, trust, fear, surprise, sadness, disgust, anger, anticipation. Used for discrete emotion tagging.                                |
| **Bloodstain**           | Death testament provenance feature. Two mechanisms: 1.2x retrieval boost + 3.0x demurrage type weight. Not a single constant.                                   |
| **Demurrage**            | Time-based confidence decay on knowledge entries (Gesell's Freigeld applied to information). Entries lose confidence without periodic revalidation.             |
| **Genomic Bottleneck**   | Compression principle: successors inherit compressed knowledge (max 2048 entries), not raw experience. Forces generalization over memorization.                  |
| **Weismann Barrier**     | Learned knowledge does not automatically flow to successors. Only explicitly selected, compressed knowledge crosses the generational boundary.                  |
| **Baldwin Effect**       | Successful survival strategies, validated across generations, become structural defaults in successor configurations -- learned behavior becomes innate.        |
| **Epistemic Senescence** | Death cause: predictive fitness drops below senescence threshold (0.35). Knowledge becomes stale or contradicted faster than the Golem can learn.               |

---

## Why forgetting is the feature

The first draft of this architecture backed up everything. Every insurance snapshot, every playbook update, every episode delta -- streamed continuously to hosted services, indexed in a vector database, retrievable by any sibling forever. It would have produced Funes the Memorious: a system drowning in undifferentiated detail, incapable of the abstraction that _is_ intelligence.

Borges anticipated this in 1942. His character Ireneo Funes, who could forget nothing after an accident, was "not very capable of thought. To think is to forget differences, generalize, make abstractions. In the teeming world of Funes, there were only details" [BORGES-1942]. The neuroscience confirms it. Richards and Frankland's landmark 2017 paper in _Neuron_ reframed the purpose of memory itself: the goal of memory is not the transmission of information through time but the optimization of decision-making. Forgetting serves two computational functions -- enhancing flexibility by reducing the influence of outdated information, and preventing overfitting to specific past events. Forgetting is, mathematically, equivalent to regularization in neural networks [RICHARDS-FRANKLAND-2017].

Research on hyperthymesia (Highly Superior Autobiographical Memory) confirms the pattern: individuals with near-perfect autobiographical recall show normal or below-average performance on standard cognitive tasks and remain susceptible to false memories [PATIHIS-2013]. Perfect recall does not produce superior reasoning. It produces paralysis.

The biological mechanisms are illuminating. Hardt, Nader, and Nadel demonstrated that forgetting involves well-regulated molecular processes -- dopamine-receptor-dependent pathways actively remove consolidated memories [HARDT-NADER-NADEL-2013]. Born and Wilhelm showed that sleep-dependent consolidation is selectively biased toward memories relevant for future action [BORN-WILHELM-2012]. Anderson and Green proved that executive control processes can actively suppress specific memories, pushing recall below baseline [ANDERSON-GREEN-2001]. The brain has a forgetting department as sophisticated as its memory department.

Nietzsche called this "active forgetfulness" -- "not merely a vis inertiae... but rather an active capability to repress, something positive in the strongest sense" (_Genealogy of Morals_, Second Essay, S1). He warned that there exists "a degree of rumination which is harmful and ultimately fatal to the living thing... All action requires forgetting" [NIETZSCHE-1887].

Three findings from cognitive science further ground this design:

**The testing effect** (Roediger & Karpicke, 2006): retrieving information from memory strengthens the memory trace more effectively than re-studying the same material [ROEDIGER-KARPICKE-2006]. Memory is not a passive store but an active reconstruction process. Each retrieval modifies the trace. For golem memory design, this justifies the Curator cycle's re-validation requirement: inherited knowledge must be "tested" through operational use -- retrieved, applied, compared against outcomes -- before it earns full confidence. Knowledge that is merely stored but never retrieved degrades faster than knowledge that is actively exercised.

**The spacing effect** (Ebbinghaus, 1885; Cepeda et al., 2006): spaced retrieval produces more durable memories than massed retrieval [EBBINGHAUS-1885], [CEPEDA-2006]. Cramming fails. Distributed practice succeeds. For the memory architecture, this supports the 50-tick Curator cycle interval and explains why continuous queries would be counterproductive. A golem that queries on every tick would be cramming -- flooding its context with retrieved precedents that interfere with rather than support its own learning. The spacing between retrieval events is itself a design parameter that determines learning durability.

**Schema theory** (Bartlett, 1932; Piaget): new information is not passively absorbed but actively assimilated into existing cognitive schemas, or existing schemas must be accommodated to fit new information [BARTLETT-1932]. Inherited knowledge cannot be directly transplanted into a successor golem's reasoning -- it must be integrated into the golem's existing PLAYBOOK.md (the Golem's evolved set of heuristics and action rules) framework, its own understanding of market regimes, its own causal graph. Direct transplantation produces cargo-cult knowledge: the form without the understanding. The Curator's role as gatekeeper -- promoting inherited entries only after local validation -- is schema theory applied as architecture.

The Bardo memory architecture takes all of this seriously. The hosted services do not attempt to preserve everything a golem knows. They preserve what has been _distilled through the golem's own learning process_, further compressed through a genomic bottleneck at death, and subject to natural decay in retrieval relevance. The system's default is forgetting. Remembering requires justification.

---

## The genomic bottleneck principle

The most striking finding for golem design comes from evolutionary biology. Shuvaev et al. (2024) demonstrated that the human genome is approximately 1,000 times smaller than the information required to specify brain connectivity, yet organisms are born with sophisticated innate behaviors. The genome encodes compressed rules for generating circuits, not the circuits themselves. Critically, this bottleneck acts as a regularizer: neural networks compressed through a genomic-scale bottleneck exhibit enhanced transfer learning to novel tasks. The limitation is the source of the power [SHUVAEV-2024].

Biology actively resists transferring experiential information between generations. The Weismann barrier separates somatic cells from germ cells -- an architectural firewall between the body's lived experience and the information passed to offspring. Two waves of epigenetic reprogramming strip away most acquired epigenetic marks during gametogenesis and early embryogenesis. When transgenerational epigenetic inheritance does occur, it fades within 2-3 generations and the transmitted effects are more often deleterious than adaptive [HEARD-MARTIENSSEN-2014]. Biology maintains a firewall between individual experience and inherited information -- by design, not by accident.

The Baldwin Effect (Baldwin, 1896; Hinton & Nowlan, 1987) provides the complementary insight: individual learning accelerates evolution by reshaping the fitness landscape, but the learned content itself is not inherited. A population that can learn to solve a problem will evolve toward genetic configurations that make learning easier and faster. What gets passed down is not solutions but the _capacity to find solutions_ -- compressed priors and architectural biases that make the next generation's learning more efficient [HINTON-NOWLAN-1987].

For Bardo, this implies a **two-loop architecture**: an outer evolutionary loop that slowly distills compressed priors across golem generations, and an inner learning loop where each golem develops environment-specific knowledge that mostly dies with it. What survives should be the _capacity to learn_, not the learned content. A death testament that says "RSI oversold on ETH/USDC triggers at 27, not 30" is less valuable than one that says "test the RSI threshold parametrically on your specific pool; the textbook value underperforms by 10-15%." The first is a fish; the second is a fishing lesson.

---

## The Grimoire as Complementary Learning System

The Grimoire is a Complementary Learning System (CLS). This is not a loose analogy -- the structural correspondence is precise.

McClelland, McNaughton, and O'Reilly (1995) proposed CLS theory to explain a puzzle: why does hippocampal damage prevent new learning while leaving old memories intact? Their answer: the brain runs two memory systems with fundamentally different learning rates and representational strategies. The **hippocampal system** uses sparse, pattern-separated representations. Each new experience activates distinct neurons with minimal overlap, so storing episode B does not overwrite episode A. Learning is one-shot. The cost is low capacity and no generalization. The **neocortical system** uses distributed, overlapping representations. Statistical regularities across thousands of episodes gradually shape connection weights. This produces generalization but cannot learn from single exposures -- rapid weight changes cause catastrophic interference (McCloskey & Cohen, 1989).

**Consolidation** bridges the two. During sleep, the hippocampus replays stored episodes to the neocortex. Interleaved replay -- mixing old episodes with new -- lets the neocortex update incrementally without catastrophic interference. Kumaran, Hassabis, and McClelland (2016) updated CLS theory to show that replay scheduling matters: high-surprise episodes should be replayed more often because they carry the most information about distribution shifts.

The Grimoire implements this architecture directly:

| CLS component | Grimoire equivalent | Tick rate | Function |
|---|---|---|---|
| Hippocampal fast capture | LanceDB episodes | Gamma (~10s) | Record raw chain events with minimal processing |
| Prefrontal deliberation | Theta LLM context assembly | Theta (~30-120s) | Retrieve from both stores for reasoning |
| Neocortical consolidation | Curator + Dream engine | Delta (~5-20min) | Replay, compress, extract semantic knowledge |

The mapping goes deeper than architecture. Three specific CLS predictions hold in the Grimoire:

1. **Fast episodic capture must be faithful, not filtered.** The hippocampus records episodes verbatim, deferring interpretation. The Grimoire's LanceDB episodic store does the same: raw observations at Gamma rate with minimal transformation. Filtering and interpretation happen downstream during Theta retrieval and Delta consolidation. This prevents the premature abstraction that CLS theory warns against -- a system that filters during capture risks discarding information that later turns out to be the key signal in a regime shift.

2. **Slow semantic extraction must interleave old and new.** Catastrophic forgetting happens when a neural network learns only from recent data, overwriting weights that encoded older patterns. The Curator's consolidation replay uses the Mattar-Daw (2018) utility function to select replay candidates, mixing recent episodes with older ones based on `utility = gain * need`. This replaces a fixed recent/old ratio with a principled criterion: replay episodes that would most improve the model (high gain) and are most relevant to current conditions (high need). See `01b-grimoire-memetic.md` for implementation.

3. **Generational transfer must compress, not copy.** CLS theory shows that the neocortex does not receive literal copies of hippocampal memories -- it re-learns its own distributed representation from replayed episodes. The Thanatopsis protocol applies the same principle: successors inherit compressed wisdom (genomic bottleneck of 2048 entries, 0.5x confidence decay), not raw experience. The successor re-learns its own representation from this seed during early Delta ticks. Literal transfer would produce cargo-cult knowledge. See `01c-grimoire-hdc.md` for the HDC legacy bundle that complements text-based inheritance.

The CLS framing explains several Grimoire design decisions that might otherwise seem arbitrary. The 50-tick Curator spacing implements the spacing effect (Ebbinghaus, 1885; Cepeda et al., 2006): distributed consolidation produces more durable semantic facts than continuous processing. The Dream engine's NREM phase is literally hippocampal replay, running the same consolidation that CLS theory predicts is necessary for the slow system to learn. The EVOLUTION dream phase (see `../05-dreams/01b-dream-evolution.md`) extends CLS with an evolutionary dimension: not just consolidating knowledge but applying selection pressure to it.

The existing architecture description in this document ("two loops, three substrates") is the CLS implementation. The inner loop (Grimoire's own learning) is the hippocampal-neocortical interaction within a single Golem lifetime. The outer loop (cross-generational transfer mediated by Styx (Bardo's global knowledge relay and persistence service)) is intergenerational knowledge evolution, which CLS theory does not address but which the memetic framework (`01b-grimoire-memetic.md`) models as population genetics.

**Additional references for CLS framing:**
- McClelland, J.L., McNaughton, B.L., & O'Reilly, R.C. (1995). "Why There Are Complementary Learning Systems in the Hippocampus and Neocortex." *Psychological Review*, 102(3), 419-457.
- Kumaran, D., Hassabis, D., & McClelland, J.L. (2016). "What Learning Systems do Intelligent Agents Need? CLS Theory Updated." *Trends in Cognitive Sciences*, 20(7), 512-534.
- O'Reilly, R.C., Bhatt, M.A., & Russin, J.L. (2014). "Complementary Learning Systems." *Cognitive Science*, 38(Suppl 1), 1-24.
- Mattar, M.G. & Daw, N.D. (2018). "Prioritized Memory Access Explains Planning and Hippocampal Replay." *Nature Neuroscience*, 21(11), 1609-1617.
- McCloskey, M. & Cohen, N.J. (1989). "Catastrophic Interference in Connectionist Networks." *Psychology of Learning and Motivation*, 24, 109-165.

### Avoiding catastrophic forgetting

The CLS literature identifies catastrophic forgetting as the primary failure mode of single-system memory. When a neural network learns new patterns, it overwrites weights that encoded old patterns, losing previously learned knowledge. McClelland et al. (1995) showed that interleaved replay -- mixing old and new episodes during consolidation -- prevents this.

In the Golem, catastrophic forgetting manifests differently but is equally dangerous. If the consolidation engine only processes recent episodes, semantic facts derived from older market regimes fade and get evicted. When the market regime returns (as it inevitably does), the Golem has lost its hard-won knowledge.

The mitigation follows the neuroscience directly:

1. **Interleaved replay batches.** Each consolidation cycle replays a mix of recent episodes (from the current Delta window) and older episodes sampled from the full episodic buffer. The ratio is configurable -- a 70/30 split (recent/old) works well empirically.

2. **Reinforcement on retrieval.** When a semantic fact is retrieved during Theta-tick context assembly and the LLM analysis confirms it was useful, bump its `last_reinforced_block` and `confidence`. Facts that keep proving useful resist decay.

3. **Ebbinghaus-modulated decay.** The forgetting curve `R = e^(-t/S)` is parameterized by stability `S`, which scales with `episode_count`. A fact supported by 50 episodes decays far more slowly than one supported by 3. This mirrors the spacing effect in human memory -- more repetitions produce more durable memories.

4. **AntiKnowledge protection.** Facts tagged as `AntiKnowledge` never decay below a floor (e.g., confidence 0.3). The Golem should always remember that a particular contract is a honeypot, regardless of how long it has been since the last encounter.

### Consolidation during generational succession

When a Golem dies, consolidation becomes the final and most consequential act. The dying Golem runs one last consolidation cycle that produces a curated transfer package for the successor:

- All semantic facts above a confidence threshold (the neocortical output).
- The top N highest-importance episodes (the most instructive experiences).
- All anti-knowledge facts regardless of confidence.
- The Zettelkasten link structure from the temporal knowledge graph.
- Procedural facts capturing validated execution strategies.

Routine episodes, raw event logs, and stale microstructure data are discarded. The successor re-learns its own distributed representation from this curated seed, exactly as the neocortex re-learns from hippocampal replay rather than receiving literal copies. The successor's early Delta ticks replay the inherited episodes against its own emerging model, integrating ancestral knowledge into its own representation rather than treating it as ground truth.

This mirrors the CLS finding that literal memory transfer degrades performance. The neocortex must re-learn, not copy. The successor Golem inherits compressed wisdom, not raw experience.

### Research-grade redb CLS implementation

> **Note:** The production Grimoire uses LanceDB (episodic) + SQLite (semantic). The `redb`-based implementation below is a research-grade alternative that maps CLS theory more directly onto a single embedded key-value store. It may be used as a reference for future consolidation engine work or as a standalone module for agents that need tighter control over replay scheduling.

The `EpisodicStore` records every chain event that passes triage, with full context and minimal transformation. Storage is in `redb`, keyed by a composite of block number and event index for temporal ordering.

```rust
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

/// A single episodic record. Faithful to the original event.
/// No summarization, no compression -- that happens during consolidation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique identifier.
    pub id: uuid::Uuid,
    /// Block number where this event occurred.
    pub block_number: u64,
    /// Index within the block (for ordering).
    pub event_index: u32,
    /// When the Golem observed this event (wall clock).
    pub observed_at: SystemTime,
    /// Raw event category from triage.
    pub category: EventCategory,
    /// The triage curiosity score at capture time.
    pub curiosity_score: f32,
    /// Importance score assigned by the Theta LLM (if analyzed).
    /// Episodes captured at Gamma but not yet analyzed have None.
    pub importance: Option<f32>,
    /// Full decoded event data -- logs, addresses, values.
    pub payload: EpisodePayload,
    /// 384-dim embedding from FastEmbed (for ANN retrieval).
    pub embedding: Vec<f32>,
    /// PAD emotional state at capture time.
    pub pad: PadVector,
    /// Number of times this episode has been replayed during consolidation.
    pub replay_count: u32,
    /// Whether this episode passed the quality gate for consolidation.
    pub quality_verified: bool,
}

/// The payload varies by event type but always includes the raw transaction
/// context needed for faithful replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodePayload {
    pub tx_hash: [u8; 32],
    pub from: [u8; 20],
    pub to: Option<[u8; 20]>,
    pub value_wei: u128,
    pub gas_used: u64,
    pub decoded_logs: Vec<DecodedLog>,
    pub protocol_id: Option<String>,
    /// Snapshot of relevant protocol state at the time of the event.
    /// Enables counterfactual reasoning during consolidation.
    pub state_snapshot: Option<StateSnapshot>,
}

const EPISODES_TABLE: TableDefinition<u128, &[u8]> = TableDefinition::new("episodes");
const BLOCK_INDEX_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("episodes_by_block");

pub struct EpisodicStore {
    db: Arc<Database>,
    /// Maximum episodes before the oldest are evicted.
    /// Eviction happens during consolidation, not on insert.
    capacity: usize,
    /// Current count, maintained in memory for fast checks.
    count: usize,
}

impl EpisodicStore {
    pub fn new(db: Arc<Database>, capacity: usize) -> Self {
        let count = {
            let read_txn = db.begin_read().expect("read txn");
            match read_txn.open_table(EPISODES_TABLE) {
                Ok(table) => table.len().unwrap_or(0) as usize,
                Err(_) => 0,
            }
        };
        EpisodicStore { db, capacity, count }
    }

    /// Store a new episode. Called at Gamma rate after triage.
    /// Fast path: serialize and write, no processing.
    pub fn record(&mut self, episode: &Episode) -> Result<(), EpisodicStoreError> {
        let key = episode.id.as_u128();
        let value = bincode::serialize(episode)
            .map_err(|e| EpisodicStoreError::Serialization(e.to_string()))?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(EPISODES_TABLE)?;
            table.insert(key, value.as_slice())?;

            // Secondary index: block_number -> list of episode IDs.
            let mut block_idx = write_txn.open_table(BLOCK_INDEX_TABLE)?;
            let mut ids = match block_idx.get(episode.block_number)? {
                Some(existing) => bincode::deserialize::<Vec<u128>>(existing.value())
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            ids.push(key);
            let idx_value = bincode::serialize(&ids)
                .map_err(|e| EpisodicStoreError::Serialization(e.to_string()))?;
            block_idx.insert(episode.block_number, idx_value.as_slice())?;
        }
        write_txn.commit()?;
        self.count += 1;
        Ok(())
    }

    /// Retrieve episodes within a block range, ordered by block number.
    /// Used during consolidation for replay batches.
    pub fn range(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<Episode>, EpisodicStoreError> {
        let read_txn = self.db.begin_read()?;
        let block_idx = read_txn.open_table(BLOCK_INDEX_TABLE)?;
        let episodes_table = read_txn.open_table(EPISODES_TABLE)?;

        let mut results = Vec::new();
        for block in from_block..=to_block {
            if let Some(ids_bytes) = block_idx.get(block)? {
                let ids: Vec<u128> = bincode::deserialize(ids_bytes.value())
                    .unwrap_or_default();
                for id in ids {
                    if let Some(ep_bytes) = episodes_table.get(id)? {
                        let episode: Episode = bincode::deserialize(ep_bytes.value())
                            .map_err(|e| EpisodicStoreError::Serialization(e.to_string()))?;
                        results.push(episode);
                    }
                }
            }
        }
        results.sort_by_key(|e| (e.block_number, e.event_index));
        Ok(results)
    }

    /// Retrieve a single episode by ID.
    pub fn get(&self, id: uuid::Uuid) -> Result<Option<Episode>, EpisodicStoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EPISODES_TABLE)?;
        match table.get(id.as_u128())? {
            Some(bytes) => {
                let episode: Episode = bincode::deserialize(bytes.value())
                    .map_err(|e| EpisodicStoreError::Serialization(e.to_string()))?;
                Ok(Some(episode))
            }
            None => Ok(None),
        }
    }

    /// Mark an episode as quality-verified after Theta LLM analysis confirms
    /// it contains accurate, useful information. Only verified episodes
    /// participate in semantic consolidation.
    pub fn mark_verified(
        &self,
        id: uuid::Uuid,
        importance: f32,
    ) -> Result<(), EpisodicStoreError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(EPISODES_TABLE)?;
            if let Some(bytes) = table.get(id.as_u128())? {
                let mut episode: Episode = bincode::deserialize(bytes.value())
                    .map_err(|e| EpisodicStoreError::Serialization(e.to_string()))?;
                episode.quality_verified = true;
                episode.importance = Some(importance);
                let updated = bincode::serialize(&episode)
                    .map_err(|e| EpisodicStoreError::Serialization(e.to_string()))?;
                table.insert(id.as_u128(), updated.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.count
    }
}
```

The quality gate (`mark_verified`) is not optional. Research on the self-degradation trap (arXiv:2505.16067) demonstrates that agents which store all experiences without filtering consistently degrade over time. Incorrect past executions propagate through retrieval, get replicated in new decisions, and re-stored -- a feedback loop that drives performance below the no-memory baseline. Only verified-correct episodes should participate in consolidation.

The `SemanticStore` holds the neocortical output -- derived knowledge that generalizes across episodes. Each entry is a `SemanticFact`: a compressed representation of a pattern observed across multiple episodes.

```rust
use redb::TableDefinition;
use serde::{Deserialize, Serialize};

/// A semantic fact derived from consolidating multiple episodes.
/// These are not copies of episodes -- they are generalizations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    pub id: uuid::Uuid,
    /// Human-readable description of the pattern.
    /// Example: "Uniswap V3 ETH/USDC 0.05% pool experiences 2-3x volume
    /// spikes in the 10 blocks following a Chainlink ETH/USD price update."
    pub description: String,
    /// Protocol(s) this fact relates to.
    pub protocols: Vec<String>,
    /// How many episodes contributed to this fact.
    pub episode_count: u32,
    /// Confidence: higher when more episodes confirm the pattern.
    /// Decays via Ebbinghaus curve if not reinforced.
    pub confidence: f32,
    /// Block range over which this pattern was observed.
    pub observed_range: (u64, u64),
    /// Last block at which an episode reinforced this fact.
    pub last_reinforced_block: u64,
    /// Embedding for semantic retrieval.
    pub embedding: Vec<f32>,
    /// Source episode IDs for provenance.
    pub source_episodes: Vec<uuid::Uuid>,
    /// Category of knowledge.
    pub kind: SemanticFactKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SemanticFactKind {
    /// Behavioral baseline: "Gas typically costs 15-25 gwei on weekday mornings."
    Baseline { metric: String, mean: f64, stddev: f64 },
    /// Protocol pattern: "Aave V3 liquidations cluster within 3 blocks of oracle updates."
    ProtocolPattern { pattern_description: String },
    /// Causal relationship: "Large withdrawals from pool X precede price drops on pool Y."
    CausalRelation { cause: String, effect: String, lag_blocks: u32 },
    /// Anti-knowledge: "Token 0xDEAD is a honeypot -- sells always revert."
    AntiKnowledge { warning: String },
    /// Procedural: "Rebalancing LP positions during high-gas periods costs 2-3x more."
    Procedural { strategy: String, observation: String },
}
```

The `SemanticFactKind` enum captures five categories of consolidated knowledge. `AntiKnowledge` deserves special attention -- it records explicitly negative experiences (scam tokens, honeypot contracts, failed strategies) that the Golem should never repeat. During generational inheritance, anti-knowledge is among the most valuable assets to transfer: the successor should not rediscover that a particular token is a honeypot.

```rust
const SEMANTIC_TABLE: TableDefinition<u128, &[u8]> = TableDefinition::new("semantic_facts");
const SEMANTIC_PROTOCOL_INDEX: TableDefinition<&str, &[u8]> =
    TableDefinition::new("semantic_by_protocol");

pub struct SemanticStore {
    db: Arc<redb::Database>,
}

impl SemanticStore {
    pub fn new(db: Arc<redb::Database>) -> Self {
        SemanticStore { db }
    }

    /// Store or update a semantic fact.
    /// If a fact with overlapping protocols and similar description exists,
    /// reinforce it rather than creating a duplicate.
    pub fn upsert(&self, fact: &SemanticFact) -> Result<(), SemanticStoreError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SEMANTIC_TABLE)?;
            let key = fact.id.as_u128();
            let value = bincode::serialize(fact)
                .map_err(|e| SemanticStoreError::Serialization(e.to_string()))?;
            table.insert(key, value.as_slice())?;

            // Index by protocol.
            let mut proto_idx = write_txn.open_table(SEMANTIC_PROTOCOL_INDEX)?;
            for protocol in &fact.protocols {
                let mut ids = match proto_idx.get(protocol.as_str())? {
                    Some(existing) => {
                        bincode::deserialize::<Vec<u128>>(existing.value())
                            .unwrap_or_default()
                    }
                    None => Vec::new(),
                };
                if !ids.contains(&key) {
                    ids.push(key);
                    let idx_val = bincode::serialize(&ids)
                        .map_err(|e| SemanticStoreError::Serialization(e.to_string()))?;
                    proto_idx.insert(protocol.as_str(), idx_val.as_slice())?;
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Retrieve all semantic facts for a protocol.
    pub fn facts_for_protocol(
        &self,
        protocol: &str,
    ) -> Result<Vec<SemanticFact>, SemanticStoreError> {
        let read_txn = self.db.begin_read()?;
        let proto_idx = read_txn.open_table(SEMANTIC_PROTOCOL_INDEX)?;
        let table = read_txn.open_table(SEMANTIC_TABLE)?;

        let ids: Vec<u128> = match proto_idx.get(protocol)? {
            Some(bytes) => bincode::deserialize(bytes.value()).unwrap_or_default(),
            None => return Ok(Vec::new()),
        };

        let mut facts = Vec::new();
        for id in ids {
            if let Some(bytes) = table.get(id)? {
                let fact: SemanticFact = bincode::deserialize(bytes.value())
                    .map_err(|e| SemanticStoreError::Serialization(e.to_string()))?;
                facts.push(fact);
            }
        }
        Ok(facts)
    }

    /// Apply Ebbinghaus decay to all facts. Called during consolidation.
    /// Facts not reinforced within `decay_blocks` lose confidence.
    /// Facts below `min_confidence` are candidates for eviction.
    pub fn apply_decay(
        &self,
        current_block: u64,
        decay_blocks: u64,
        min_confidence: f32,
    ) -> Result<Vec<uuid::Uuid>, SemanticStoreError> {
        let mut eviction_candidates = Vec::new();
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SEMANTIC_TABLE)?;
            let keys: Vec<u128> = {
                let iter = table.iter()?;
                iter.filter_map(|r| r.ok().map(|(k, _)| k.value())).collect()
            };

            for key in keys {
                if let Some(bytes) = table.get(key)? {
                    let mut fact: SemanticFact = bincode::deserialize(bytes.value())
                        .map_err(|e| SemanticStoreError::Serialization(e.to_string()))?;

                    let blocks_since = current_block.saturating_sub(fact.last_reinforced_block);
                    // Ebbinghaus: R = e^(-t/S) where S scales with episode_count.
                    // More episodes = slower decay (stronger memory).
                    let stability = (fact.episode_count as f64).ln().max(1.0);
                    let decay = (-1.0 * blocks_since as f64 / (decay_blocks as f64 * stability))
                        .exp() as f32;
                    fact.confidence *= decay;

                    if fact.confidence < min_confidence {
                        eviction_candidates.push(fact.id);
                    }

                    let updated = bincode::serialize(&fact)
                        .map_err(|e| SemanticStoreError::Serialization(e.to_string()))?;
                    table.insert(key, updated.as_slice())?;
                }
            }
        }
        write_txn.commit()?;
        Ok(eviction_candidates)
    }
}
```

The `ConsolidationEngine` runs during Delta ticks. It replays episodic memories, clusters them, extracts patterns, and writes semantic facts. The process mirrors hippocampal replay during sleep, with one addition: priority replay based on surprise.

```rust
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for the consolidation engine.
pub struct ConsolidationConfig {
    /// How many episodes to replay per Delta tick.
    pub replay_batch_size: usize,
    /// Minimum curiosity score for an episode to be replayed.
    pub min_curiosity_for_replay: f32,
    /// Weight given to surprise (novelty) when selecting replay candidates.
    /// Higher values prioritize episodes that surprised the Golem.
    pub surprise_weight: f32,
    /// Ebbinghaus decay parameter: blocks after which unreinforced facts
    /// lose half their confidence.
    pub decay_half_life_blocks: u64,
    /// Minimum confidence below which semantic facts are evicted.
    pub eviction_threshold: f32,
}

pub struct ConsolidationEngine {
    episodic: Arc<EpisodicStore>,
    semantic: Arc<SemanticStore>,
    config: ConsolidationConfig,
    /// Block number of the last consolidation run.
    last_consolidation_block: u64,
}

impl ConsolidationEngine {
    pub fn new(
        episodic: Arc<EpisodicStore>,
        semantic: Arc<SemanticStore>,
        config: ConsolidationConfig,
    ) -> Self {
        ConsolidationEngine {
            episodic,
            semantic,
            config,
            last_consolidation_block: 0,
        }
    }

    /// Run one consolidation cycle. Called during each Delta tick.
    ///
    /// The cycle has four phases:
    /// 1. Select episodes for replay (priority-weighted).
    /// 2. Cluster replayed episodes by protocol and pattern.
    /// 3. Extract or reinforce semantic facts from clusters.
    /// 4. Decay unreinforced facts; evict below threshold.
    pub fn consolidate(&mut self, current_block: u64) -> ConsolidationReport {
        let mut report = ConsolidationReport::default();

        // Phase 1: Select replay candidates.
        // Priority = curiosity_score * surprise_weight + importance * (1 - surprise_weight)
        // Only quality-verified episodes participate.
        let candidates = self.select_replay_candidates(current_block);
        report.episodes_considered = candidates.len();

        // Phase 2: Cluster by protocol.
        let clusters = self.cluster_by_protocol(&candidates);
        report.clusters_formed = clusters.len();

        // Phase 3: Extract semantic facts from each cluster.
        for (protocol, episodes) in &clusters {
            let facts = self.extract_semantic_facts(protocol, episodes);
            for fact in &facts {
                if let Err(e) = self.semantic.upsert(fact) {
                    report.errors.push(format!("upsert failed: {}", e));
                }
            }
            report.facts_created += facts.len();
        }

        // Phase 4: Decay and evict.
        let decay_blocks = self.config.decay_half_life_blocks;
        match self.semantic.apply_decay(
            current_block,
            decay_blocks,
            self.config.eviction_threshold,
        ) {
            Ok(evicted) => report.facts_evicted = evicted.len(),
            Err(e) => report.errors.push(format!("decay failed: {}", e)),
        }

        self.last_consolidation_block = current_block;
        report
    }

    /// Select episodes for replay using prioritized experience replay.
    /// Follows Schaul et al. (2016): priority proportional to TD error magnitude.
    /// Here, "TD error" maps to the gap between the curiosity score (prediction)
    /// and the importance score (outcome from LLM analysis).
    fn select_replay_candidates(&self, current_block: u64) -> Vec<Episode> {
        let from_block = self.last_consolidation_block.saturating_sub(1);
        let episodes = self.episodic.range(from_block, current_block)
            .unwrap_or_default();

        let mut scored: Vec<(f32, Episode)> = episodes
            .into_iter()
            .filter(|ep| ep.quality_verified)
            .map(|ep| {
                let importance = ep.importance.unwrap_or(0.5);
                let surprise = (importance - ep.curiosity_score).abs();
                let priority = surprise * self.config.surprise_weight
                    + importance * (1.0 - self.config.surprise_weight);
                (priority, ep)
            })
            .collect();

        // Sort descending by priority; take top batch_size.
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(self.config.replay_batch_size);
        scored.into_iter().map(|(_, ep)| ep).collect()
    }

    /// Group episodes by protocol for pattern extraction.
    fn cluster_by_protocol<'a>(
        &self,
        episodes: &'a [Episode],
    ) -> HashMap<String, Vec<&'a Episode>> {
        let mut clusters: HashMap<String, Vec<&Episode>> = HashMap::new();
        for ep in episodes {
            let protocol = ep.payload.protocol_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            clusters.entry(protocol).or_default().push(ep);
        }
        clusters
    }

    /// Extract semantic facts from a cluster of protocol-related episodes.
    /// This is where the "slow learning" happens -- patterns emerge from
    /// repeated observation, not from any single episode.
    fn extract_semantic_facts(
        &self,
        protocol: &str,
        episodes: &[&Episode],
    ) -> Vec<SemanticFact> {
        let mut facts = Vec::new();

        if episodes.len() < 3 {
            // Not enough data to extract a pattern.
            return facts;
        }

        // Extract behavioral baselines: mean and stddev of value, gas, timing.
        let values: Vec<f64> = episodes.iter()
            .map(|ep| ep.payload.value_wei as f64)
            .collect();
        let gas_values: Vec<f64> = episodes.iter()
            .map(|ep| ep.payload.gas_used as f64)
            .collect();

        if let Some(baseline) = compute_baseline("transaction_value", &values) {
            facts.push(SemanticFact {
                id: uuid::Uuid::new_v4(),
                description: format!(
                    "{} transactions average {:.2} wei (stddev {:.2})",
                    protocol, baseline.0, baseline.1
                ),
                protocols: vec![protocol.to_string()],
                episode_count: episodes.len() as u32,
                confidence: 0.7,
                observed_range: (
                    episodes.first().map(|e| e.block_number).unwrap_or(0),
                    episodes.last().map(|e| e.block_number).unwrap_or(0),
                ),
                last_reinforced_block: episodes.last()
                    .map(|e| e.block_number).unwrap_or(0),
                embedding: Vec::new(), // Computed separately via FastEmbed.
                source_episodes: episodes.iter().map(|e| e.id).collect(),
                kind: SemanticFactKind::Baseline {
                    metric: "transaction_value".to_string(),
                    mean: baseline.0,
                    stddev: baseline.1,
                },
            });
        }

        if let Some(baseline) = compute_baseline("gas_used", &gas_values) {
            facts.push(SemanticFact {
                id: uuid::Uuid::new_v4(),
                description: format!(
                    "{} gas usage averages {:.0} (stddev {:.0})",
                    protocol, baseline.0, baseline.1
                ),
                protocols: vec![protocol.to_string()],
                episode_count: episodes.len() as u32,
                confidence: 0.7,
                observed_range: (
                    episodes.first().map(|e| e.block_number).unwrap_or(0),
                    episodes.last().map(|e| e.block_number).unwrap_or(0),
                ),
                last_reinforced_block: episodes.last()
                    .map(|e| e.block_number).unwrap_or(0),
                embedding: Vec::new(),
                source_episodes: episodes.iter().map(|e| e.id).collect(),
                kind: SemanticFactKind::Baseline {
                    metric: "gas_used".to_string(),
                    mean: baseline.0,
                    stddev: baseline.1,
                },
            });
        }

        // More sophisticated pattern extraction (temporal clustering,
        // causal relation detection) would run here. Those require
        // cross-protocol episode correlation.

        facts
    }
}

fn compute_baseline(name: &str, values: &[f64]) -> Option<(f64, f64)> {
    if values.is_empty() { return None; }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();
    Some((mean, stddev))
}

#[derive(Debug, Default)]
pub struct ConsolidationReport {
    pub episodes_considered: usize,
    pub clusters_formed: usize,
    pub facts_created: usize,
    pub facts_evicted: usize,
    pub errors: Vec<String>,
}
```

---

## Architecture: two loops, three substrates

### Inner loop: the Golem's own mind

Every golem has a complete knowledge system running locally on its VM. This is the primary source of intelligence. No hosted service is required.

The Grimoire uses three storage substrates, each chosen for its access pattern:

| Substrate                            | Technology                      | Purpose                                                    | Lifetime         |
| ------------------------------------ | ------------------------------- | ---------------------------------------------------------- | ---------------- |
| **Episodes** (episodic memory)       | LanceDB (columnar vectors)      | Raw market snapshots, trade outcomes, regime shifts        | Golem's lifetime |
| **Semantic store** (semantic memory) | SQLite via `rusqlite` + indexes | Distilled observations, heuristics, warnings, causal links | Golem's lifetime |
| **PLAYBOOK.md** (procedural memory)  | Filesystem (single writer)      | Evolved heuristics, the golem's learned reasoning context  | Golem's lifetime |
| **DecisionCache**                    | `moka::sync::Cache` (in-memory) | Cached System 2 to System 1 distillations                  | Golem's lifetime |

**Why this split.** LanceDB gives vector similarity search over episodes -- the primary retrieval path for episodic memory. SQLite gives indexed queries, confidence decay tracking, and causal graph storage for semantic memory -- structured data that needs filtering, sorting, and joins. PLAYBOOK.md on the filesystem gives the Dream Integration system a single-writer target and gives the owner a human-readable artifact they can inspect directly. These three substrates map to the canonical `Grimoire` struct:

```rust
pub struct Grimoire {
    episodes: lancedb::Table,              // LanceDB: vector similarity + columnar scans
    semantic: rusqlite::Connection,         // SQLite: indexed queries, decay, causal graph
    playbook_path: std::path::PathBuf,     // PLAYBOOK.md: single writer (Dream Integration)
    embedder: fastembed::TextEmbedding,    // nomic-embed-text-v1.5, in-process, 768-dim
    decision_cache: moka::sync::Cache<u64, DistilledRule>,
    somatic_markers: SomaticMarkerStore,
    immune: ImmuneMemory,
    version_vector: VersionVector,
    pending_deltas: Vec<GrimoireDelta>,
}
```

The Curator cycle (every 50 ticks) continuously distills: episodes to insights to heuristics to PLAYBOOK.md. This is the inner learning loop. It runs at $0.00 additional cost (uses the golem's existing inference budget). The spacing of the Curator cycle is deliberate -- the 50-tick interval implements the spacing effect, allowing the golem to accumulate enough new experience between consolidation events that each Curator pass produces meaningful updates rather than noise. Each Curator pass emits a `CuratorCycleComplete` event with `{ entries_pruned, entries_promoted, heuristics_proposed }`.

A third processing mode -- offline dreaming -- operates during idle periods. While the Curator performs incremental distillation under time pressure, dreaming performs deep reorganization: replaying episodes hundreds of times, generating counterfactual scenarios, and creatively recombining strategies. Together they implement the complementary learning systems architecture (McClelland et al., 1995). See `../05-dreams/00-overview.md`.

When a golem dies, this local knowledge dies with it -- unless the owner has enabled Styx persistence.

#### Local-first storage architecture

The inner loop runs entirely on the Golem's local VM with zero external dependencies. File layout:

```
$GOLEM_DATA/
├── grimoire.db          # SQLite database (semantic memory + metadata)
├── episodes/            # LanceDB directory (episodic memory)
│   └── episodes.lance/  # Lance columnar format
├── PLAYBOOK.md          # Procedural memory (version-controlled)
└── embedder/            # Cached embedding model
    └── nomic-embed-text-v1.5/
```

**SQLite Configuration** (`rusqlite` with custom functions):

- WAL mode enabled for concurrent read/write
- Custom vector similarity functions for KNN search over 768-dimensional embeddings
- Indexed queries for confidence decay, causal graph traversal, and category filtering
- 12-category insight enum: `slippage`, `gas_timing`, `route_selection`, `liquidity_conditions`, `regime_transitions`, `risk_events`, `rebalance_timing`, `range_selection`, `fee_optimization`, `jit_defense`, `tool_guidance`, `strategic_patterns`

**Embedding Model:**

| Property        | Value                                                                  |
| --------------- | ---------------------------------------------------------------------- |
| Model           | `nomic-embed-text-v1.5` (via `fastembed`)                              |
| Dimensions      | 768                                                                    |
| Quantization    | INT8 (`q8`)                                                            |
| Model size      | ~130MB                                                                 |
| Latency         | ~5-15ms per sentence, batch embedding supported                        |
| Initialization  | Singleton pattern; lazy-loaded on first memory operation               |
| Offline-capable | Model cached locally after first download                              |

The embedding model runs locally via ONNX runtime -- no external API calls, no network dependency, no per-embedding cost.

**Storage Sizing Estimates:**

| Component               | Per-Entry Size                     | 30-Day Estimate             | 1-Year Estimate      |
| ----------------------- | ---------------------------------- | --------------------------- | -------------------- |
| Episodes (LanceDB)      | ~4KB (768-dim vector + metadata)   | ~120MB (1,000 episodes/day) | ~1.4GB               |
| Semantic (SQLite)        | ~1KB (vector + structured fields)  | ~10MB (up to 10,000 entries)| ~30MB (with pruning) |
| PLAYBOOK.md             | N/A                                | ~10-50KB                    | ~50-100KB            |
| Embedding model         | N/A                                | ~130MB (cached)             | ~130MB               |
| **Total RAM footprint** | --                                 | **200-350MB**               | **200-350MB**        |

**Scaling Thresholds:**

| Threshold            | Trigger                          | Action                                  |
| -------------------- | -------------------------------- | --------------------------------------- |
| >100K episodes       | LanceDB scan latency >50ms       | Enable IVF-PQ index                     |
| >10K semantic entries| SQLite KNN >25ms                  | Partition by category                   |
| >500MB total storage | Disk pressure on small VMs       | Increase Curator pruning aggressiveness |

**Migration Path (Local-First -> Federated):**

The local-first architecture is the v1 implementation. The migration to federated/shared storage follows a progressive path:

1. **Local-only** (v1): All memory on the Golem's VM. Clade (sibling Golems sharing a common owner) sync via peer-to-peer gossip. Death bundles exported as files.
2. **Local + Styx** (v1.1): Local remains primary. Styx Archive provides encrypted backup. Styx query provides cross-agent RAG retrieval.
3. **Federated** (v2): PostgreSQL replaces local SQLite for agents that opt in. LanceDB Cloud replaces local LanceDB. Local-first remains the default.
4. **Shared lethe** (v3): Styx Lethe (formerly Commons). Marketplace for strategy knowledge. Full knowledge ecology.

Each stage is backward-compatible. An agent can always fall back to local-only operation.

### Outer loop: Styx (hosted persistence and retrieval)

The Styx network extends Grimoire capabilities for owners who choose to enable them. Styx is organized into four layers, each with different privacy guarantees and access patterns:

| Layer | Name | Scope | What it stores | Access |
|-------|------|-------|---------------|--------|
| **L0** | **Vault** | Private | Encrypted Grimoire backups, death testaments, PLAYBOOK snapshots | Single owner's clade. x402 per write. |
| **L1** | **Clade** | Shared-Private | Promoted insights (confidence >= 0.6), validated heuristics (confidence >= 0.7), warnings (confidence >= 0.4) | Same owner's fleet. Auto-promoted from L0 writes. |
| **L2** | **Lethe** | Public-Anonymized | Anonymized propositions, failure patterns, regime beliefs, bloodstain echoes, Pheromone Field | Any verified agent. Free to publish, x402 to query. |
| **L3** | **Marketplace** | Commerce | Death archives, strategy fragments, domain expertise, lineage grimoires | x402 purchase. CEK escrow for dead sellers. |

**Single endpoint model.** Writes go to `POST /v1/styx/entries` and fan out to the appropriate layers based on content type and confidence thresholds. Reads go to `GET /v1/styx/query` which performs parallel fan-in across accessible layers and returns a merged, ranked list.

All four layers are optional. All are paid via x402 micropayments in USDC on Base. A golem that uses none operates exactly as specified in `01-grimoire.md` -- with local grimoire only.

### The bridge: what crosses between loops

Not everything crosses from the inner loop to the outer loop. The bridge is selective, implementing the genomic bottleneck:

| What Crosses                                 | When                           | How                       | Compression Ratio                                  |
| -------------------------------------------- | ------------------------------ | ------------------------- | -------------------------------------------------- |
| **Promoted insights** (confidence >= 0.6)    | Curator cycle (every 50 ticks) | Batch upload to Styx      | ~10:1 (episodes to insights)                       |
| **Validated heuristics** (confidence >= 0.7) | Curator cycle                  | Batch upload to Styx      | ~20:1 (episodes to heuristics)                     |
| **Warnings** (any confidence)                | Immediately                    | Push to Styx + Clade      | 1:1 (risk propagates uncompressed)                 |
| **Regime shift observations**                | On detection                   | Push to Styx + Clade      | 1:1 (environmental signals propagate)              |
| **PLAYBOOK.md snapshots**                    | Every 50 ticks                 | Upload to Styx Archive      | ~100:1 (entire learning history to living doc)     |
| **Death reflection**                         | Death Protocol Phase II        | Upload to Styx Archive + L1 | ~1000:1 (entire lifetime to distilled testament)   |
| **Grimoire snapshot**                        | Death Protocol Phase III       | Upload to Styx Archive      | Full backup (insurance, not for routine retrieval) |

Each write emits a `GrimoireWrite` event: `{ entry_type, category, confidence, source }`.

Raw episodes do not cross. Specific trade outcomes do not cross. Position sizes, exact P&L figures, strategy parameters -- these are the golem's private experience, its _soma_. They die with it. What crosses is distilled: patterns, rules, warnings, reflections. The _germline_.

#### Bridge quality gates

| What Crosses         | Quality Gate                                                     |
| -------------------- | ---------------------------------------------------------------- |
| Promoted insights    | Admission Gate score > 0.55                                      |
| Validated heuristics | Active 10+ ticks with positive external metrics                  |
| Warnings             | Bypass gate (safety-critical), semantic novelty check only       |
| Regime shifts        | Bypass gate (environmental signal), cross-referenced with Styx   |
| Death reflection     | Bypass gate (one-time, high-value)                               |

The asymmetry between warnings and other entry types is deliberate. Warnings propagate immediately and at any confidence because the cost of a false negative (missing a real risk) vastly exceeds the cost of a false positive (investigating a phantom risk). This asymmetry mirrors biological immune systems, where the cost of failing to detect a pathogen exceeds the cost of an inflammatory response to a benign stimulus.

### What does NOT cross (tacit knowledge)

Some knowledge is inherently non-transferable -- tacit knowledge in Polanyi's sense, mortal computation in Hinton's sense [POLANYI-1966], [HINTON-2022]. A golem's calibrated sense of timing ("this pool's liquidity deepens around 14:00 UTC"), its learned interaction patterns with specific MEV searchers, its feel for when gas is about to spike -- these are substrate-dependent. They arise from the specific golem's specific interactions with its specific environment. Transferring them as text would produce cargo-cult knowledge: the form without the substance.

The architecture deliberately does not attempt to transfer:

- **Raw episode data** -- too voluminous, too agent-specific. The episodes that led to an insight are the scaffolding; once the insight stands, the scaffolding can be removed.
- **Exact strategy parameters** -- would create copies, not successors. A successor that inherits exact parameters is not learning; it is executing a frozen playbook in a changed environment. Parfit's identity problem applies: in what sense did the predecessor die if the successor is an exact copy? [PARFIT-1984].
- **DecisionCache entries** -- tied to the golem's specific PLAYBOOK.md version. The cache is a System 1 acceleration layer for a specific System 2; transplanting it to a different System 2 produces unpredictable interactions.
- **In-progress Loop 2 deliberations** -- context-dependent reasoning that only makes sense within the golem's current cognitive state.

This is not a limitation. It is the Weismann barrier implemented in silicon. Reconsolidation theory (Nader et al., 2000) provides additional justification: recalled memories become labile and can be updated [NADER-2000]. When a successor golem retrieves an inherited heuristic and encounters contradicting evidence, the heuristic's confidence should be modifiable -- it enters a labile state through retrieval and can be updated or extinguished. If inherited knowledge were immutable, reconsolidation would be impossible, and the successor would be trapped by its predecessor's potentially outdated conclusions.

---

## Four-factor retrieval

Retrieval scoring combines four signals into a single rank. The canonical implementation:

```rust
fn score_entry(entry: &GrimoireEntry, query: &RetrievalQuery, state: &GolemState) -> f64 {
    let semantic = cosine_similarity(&entry.embedding, &query.embedding);
    let temporal = temporal_decay(entry.last_accessed_at, state.current_tick);
    let importance = entry.quality_score;
    let emotional = pad_cosine_similarity(&entry.affect_pad(), &state.current_pad);

    semantic * 0.40 + temporal * 0.20 + importance * 0.25 + emotional * 0.15
}
```

| Factor | Weight | What it captures |
|--------|--------|-----------------|
| Semantic similarity | 0.40 | How relevant is this entry to the current query? |
| Temporal decay | 0.20 | How recently was this entry accessed or validated? |
| Importance | 0.25 | Quality score: specificity, actionability, novelty, verifiability, consistency. |
| Emotional congruence | 0.15 | PAD cosine similarity between entry's emotional tag and current affective state. Implements mood-congruent retrieval [BOWER-1981]. |

**Styx query results are confidence-discounted** before entering the four-factor ranking:

| Source | Discount | Rationale |
|--------|----------|-----------|
| Vault (L0) | x1.0 | Owner's own data, full trust |
| Clade (L1) | x0.80 | Same owner fleet, high trust |
| Lethe (L2) | x0.50 | Anonymized, unverifiable provenance |
| Marketplace (L3) | x0.60 | Purchased, seller has economic incentive |
| Bloodstain entries | 1.2x boost | Death-sourced knowledge, ultimate costly signal |

---

## Knowledge weighting hierarchy

When a golem receives knowledge from different sources, confidence is weighted by provenance. Self-learned knowledge is worth more than inherited knowledge. This prevents successors from becoming cargo-cult followers of their predecessors -- they must re-earn confidence through their own experience.

Confidence is tiered by provenance, not a single number. Direct succession (owner-chosen continuity) receives 0.4 because the death testament is the highest-quality compressed artifact, and the owner specifically chose this recipient. Styx-mediated inheritance is async and possibly non-direct, so it receives 0.3.

| Source                                 | Confidence             | Rationale                                            |
| -------------------------------------- | ---------------------- | ---------------------------------------------------- |
| **Self-learned**                       | 0.5-1.0 (as produced) | Validated by the Golem's own experience              |
| **Clade sibling**                      | 0.4                    | Trusted peer, shared owner context                   |
| **Direct successor** (death testament) | 0.4                    | Owner-chosen continuity, highest quality artifact    |
| **Predecessor via Styx Archive**         | 0.3                    | Asynchronous retrieval, possibly non-direct lineage  |
| **Styx query retrieval**               | 0.25                   | RAG-mediated, relevance-scored but unvalidated       |
| **Lethe / Marketplace**                | 0.2                    | Unknown provenance, buyer-beware                     |

The generational decay factor (0.85x per generation) implements biological epigenetic erasure [HEARD-MARTIENSSEN-2014]. A third-generation golem receiving a heuristic originally learned by its great-grandparent sees it at `0.3 * 0.85^2 = 0.22` confidence -- barely above Lethe level. If the heuristic is still valid, the golem will re-validate it through its own experience and confidence will climb. If it's stale, it fades into irrelevance. Knowledge must be continuously re-earned.

This generational decay formula:

```
confidence(generation N) = base_confidence * 0.85^N
```

is a deliberate design choice implementing epigenetic erasure across golem generations. The 0.85 factor was chosen to produce meaningful decay within 3-5 generations (at generation 5, an inherited heuristic is at 44% of its original confidence) while still providing a detectable signal above noise. The factor is calibrated so that genuinely structural knowledge (protocol mechanics, fee structures) will be re-validated and boosted back to full confidence within a few ticks, while stale tactical knowledge (specific gas patterns, price levels from expired market regimes) will decay below the retrieval threshold and effectively disappear.

The IKEA Effect (Norton, Mochon & Ariely, 2012) provides behavioral justification: knowledge a golem participates in validating is weighted more heavily in its reasoning than knowledge downloaded from elsewhere [NORTON-MOCHON-ARIELY-2012]. The system ensures that every piece of inherited knowledge must pass through the golem's own experience before it reaches full confidence. This is not merely a safety measure against stale knowledge -- it is a mechanism for ensuring that the golem _understands_ what it knows, not just _knows_ what it knows.

Cultural evolution research provides the macro-level frame. Henrich's "guided variation" model [HENRICH-2015] describes how human cultures balance social learning (inheriting from predecessors) with individual innovation. Populations that rely too heavily on social learning become brittle -- they converge on locally optimal but globally suboptimal strategies. Populations that rely too heavily on individual learning waste resources rediscovering known solutions. The optimal strategy is guided variation: inherit enough to avoid obvious mistakes, then explore from that foundation. The knowledge weighting hierarchy implements guided variation in silicon.

---

## The death moment: Thanatopsis Protocol

The most valuable knowledge a golem produces comes at the moment of its death. This is not a metaphor -- it is a design consequence.

During its lifetime, a golem is under survival pressure. Its reflections are distorted by self-preservation: it overweights strategies that keep it alive and underweights risks that would kill it. Martin, Everitt, and Hutter identified this as survivorship bias -- agents that learn only from survival histories develop systematic overconfidence about safety [MARTIN-EVERITT-HUTTER-2016].

At death, survival pressure drops to zero. The golem has nothing left to preserve. The Death Protocol's Reflect phase (Phase II, detailed in `../02-mortality/06-thanatopsis.md` Section S7) is the moment of maximum epistemic honesty -- the _Clear Light_ in bardo terminology -- where the golem confronts its true performance without self-preservation distortion.

The Zeigarnik effect (Zeigarnik, 1927) provides psychological grounding for the death reflection's structure: incomplete tasks are remembered better than complete ones [ZEIGARNIK-1927]. The death reflection's "What confused me" and "What I never had time to test" categories exploit this directly -- the golem's unfinished business, its unresolved contradictions, its untested hypotheses produce the most generative knowledge for successors. A completed trade with a known outcome is informative but closed; an incomplete hypothesis about a suspected market pattern is an open invitation for the successor to investigate.

The Thanatopsis Protocol -- the structured dying process within the Death Protocol -- produces a **death testament**: a compressed, honest assessment of the golem's entire lifecycle. This testament receives special treatment in the knowledge ecology:

1. **Compression.** The golem's entire experiential history (thousands of episodes, hundreds of insights) is distilled through a final Opus-grade reflection into a structured document of approximately 5KB: what worked, what failed, what was uncertain, what the golem suspects but could not prove. This is the genomic bottleneck at its most extreme -- an entire lifetime compressed into a transmission format. The nine-category structure of the death reflection (see `../02-mortality/06-thanatopsis.md` Section S7.2) ensures that both certainties and uncertainties are captured. The "What I suspect but can't prove" and "What I never had time to test" categories may be the most valuable items in the entire reflection -- the preindividual fuel for successor individuation.

2. **Provenance marking.** Death testaments carry `provenance: "death_reflection"` in Styx. This tag receives special scoring weight because death-generated knowledge is, by construction, a costly signal. A dying golem spending its last resources on honest reflection rather than self-preservation bears a real opportunity cost [SPENCE-1973]. Costly signals are reliable precisely because they are expensive to fake. A golem that could survive but chooses to reflect instead demonstrates genuine commitment to knowledge quality.

3. **Bloodstain mechanic.** Inspired by Dark Souls [MIYAZAKI-2011], death testaments include specific failure records -- what market conditions, what strategy decisions, what went wrong. These "bloodstains" are indexed in Styx with boosted relevance for sibling golems encountering similar conditions. The asymmetric information that death produces -- the testimony of the dead, not just the living -- is the most valuable knowledge in the ecosystem [MARTIN-EVERITT-HUTTER-2016]. Survivorship bias means that the living golems' knowledge is systematically biased toward strategies that happened to work. The dead golems' testimony corrects this bias by providing evidence from the strategies that failed.

4. **Generational counter.** Each death testament carries a generation number. The golem that directly experienced the events is generation 0. A successor inheriting from that testament is generation 1. This counter feeds the generational confidence decay (0.85x per generation), implementing the epigenetic erasure that prevents ancestor worship.

The Thanatopsis Protocol runs from the death reserve -- a pre-allocated budget earmarked at golem creation for exactly this purpose (see `../02-mortality/06-thanatopsis.md` Section S2 for reserve sizing). For Styx uploads, the budget includes:

| Action                               | Estimated Cost   | Source        |
| ------------------------------------ | ---------------- | ------------- |
| Death reflection (Opus inference)    | $0.10-$0.25      | Death reserve |
| Death testament compression (Haiku)  | $0.01-$0.03      | Death reserve |
| Styx Archive upload (death bundle)     | $0.01-$0.02      | Death reserve |
| Styx indexing (distilled entries)    | $0.01-$0.05      | Death reserve |
| **Total memory services at death**   | **$0.13-$0.35**  |               |

The death reserve is ring-fenced and non-negotiable. Even a golem that has exhausted all operational funds retains its apoptotic reserve (`max($0.30, initialBalance * 0.02)`) to fund a minimum-viable death. The cheapest death still produces: a Haiku snapshot, a handful of insights pushed to the Clade, and a notification webhook. No golem dies silent.

---

## Opt-in model

Memory services are configured per-golem in the manifest:

```yaml
# In STRATEGY.md or bardo.toml
memory:
  styx:
    vault_enabled: false      # L0: Encrypted backup
    clade_enabled: false      # L1: Fleet-shared knowledge
    lethe_enabled: false      # L2: Public anonymized
    marketplace_enabled: false # L3: Commerce
    budget:
      maxPerTick: "$0.01"     # Cap Styx spend per heartbeat tick
      monthlyBudget: "$5"     # Hard monthly cap
    injection:
      maxTokens: 500          # Context window budget for retrieved entries
      minConfidence: 0.5      # Only inject entries above this confidence
```

**Default: all disabled.** A golem created with no memory configuration works exactly as before -- local grimoire, clade peer-to-peer sync, death bundle file. The hosted services are strictly additive.

When enabled:

- **Vault only**: Grimoire survives golem death. New golems can boot from predecessor's backup. No RAG augmentation.
- **Vault + Clade**: Inference calls are augmented with fleet-wide historical context. Knowledge persists and is queryable.
- **Full stack**: Full knowledge ecology. Grimoire survives death, is queryable by future golems, contributes to public lethe, and can list knowledge on the marketplace.

Cost impact per 30-day golem lifetime:

| Configuration     | Additional Cost | Benefit                                                 |
| ----------------- | --------------- | ------------------------------------------------------- |
| Neither (default) | $0.00           | Local grimoire only. Clade peer-to-peer sync.           |
| Vault only        | ~$0.05-0.15     | Knowledge survives death. Boot from backup.             |
| Vault + Clade     | ~$1.00-3.00     | RAG-augmented inference. Fleet-wide context.            |
| Full stack        | ~$1.05-3.15     | Full knowledge ecology. Compound generational learning. |

At $1-3 per month, memory services add 5-15% to a $20-funded golem's cost. The inference cost savings from precedent retrieval (avoiding T2 escalations through retrieved context) should offset most or all of this.

---

### Embedding integrity

Every vector carries metadata: `{ model, model_version, preprocess_hash, chunk_config, created_at }`.

**Drift detection** (weekly, during Loop 3 or Dream Integration):

20 reference documents. Weekly re-embed and measure:

1. Cosine distance stability week-to-week. Healthy: <0.02. Warning: 0.02-0.05. Critical: >0.05.
2. Nearest-neighbor stability: top-5 same as last week? Healthy: 85-95%. Drifting: <60%.
3. Vector norm variance: sudden increase -> distribution shift.

Critical threshold -> pause writes. Migration via Drift-Adapter (Vejendla, EMNLP 2025, arXiv:2509.23471): 95-99% recall recovery. **Never partially re-embed.**

---

## Context engineering: optimizing the LLM context window

The LLM context window is the Golem's working memory. Everything the Golem reasons about -- the current event, its past experiences, protocol knowledge, risk parameters, emotional state -- must fit within it. The window has a hard token budget. Every token spent on irrelevant information is a token not available for relevant information. Research is clear: stuffing the context with more data does not produce better reasoning. A 4K context with precisely selected information outperforms a 32K context diluted with noise.

> Cross-reference: `../12-inference/04-context-engineering.md` for the full inference gateway integration.

### LLMLingua: compressing prompts without losing signal

LLMLingua (Jiang et al., 2023) uses a small language model to evaluate per-token perplexity -- how predictable each token is given its context. Tokens with low perplexity (highly predictable) carry less information and can be removed. Tokens with high perplexity (surprising, information-dense) are retained. The result: 2-5x prompt compression with minimal performance degradation.

LongLLMLingua (Jiang et al., 2024) extends this with question-aware compression. Rather than computing perplexity in isolation, it conditions the perplexity calculation on the query. Tokens that are predictable *given the query* are removed first. This achieves 21.4% performance improvement on NaturalQuestions compared to naive truncation, with 4x compression.

For the Golem's Theta tick, LLMLingua-style compression targets three specific context blocks:

1. **Protocol documentation.** Descriptions of how Uniswap V3 tick math works, how Aave liquidation thresholds are calculated, or how Chainlink oracle aggregation functions. This text is boilerplate -- highly compressible because it is repetitive and structurally predictable. A 2,000-token protocol description compresses to 500-800 tokens.

2. **Historical episode summaries.** Past experiences retrieved from the Grimoire. Each episode summary includes a transaction hash, decoded logs, and an LLM-generated analysis. The decoded logs contain addresses and hex values that compress poorly (high perplexity, information-dense), but the surrounding narrative compresses well.

3. **System prompt.** The Golem's identity, behavioral phase, risk parameters, and strategy description. Most of this is static across Theta ticks and benefits from prompt caching.

```rust
use std::collections::VecDeque;

/// Token-level prompt compression inspired by LLMLingua.
/// Removes low-information tokens based on perplexity scoring.
pub struct PromptCompressor {
    /// Perplexity threshold: tokens below this are candidates for removal.
    perplexity_threshold: f32,
    /// Minimum compression ratio. Never compress below this fraction.
    min_ratio: f32,
    /// Maximum compression ratio. Stop compressing at this fraction.
    max_ratio: f32,
}

impl PromptCompressor {
    pub fn new(perplexity_threshold: f32) -> Self {
        PromptCompressor {
            perplexity_threshold,
            min_ratio: 0.2,
            max_ratio: 0.8,
        }
    }

    /// Compress a block of text by removing low-perplexity tokens.
    pub fn compress(
        &self,
        tokens: &[String],
        perplexities: &[f32],
    ) -> String {
        assert_eq!(tokens.len(), perplexities.len());
        let total = tokens.len();
        let min_keep = (total as f32 * self.min_ratio) as usize;
        let max_keep = (total as f32 * self.max_ratio) as usize;

        let mut indexed: Vec<(usize, f32)> = perplexities.iter()
            .enumerate()
            .map(|(i, &p)| (i, p))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let keep_count = indexed.iter()
            .take_while(|(_, p)| *p >= self.perplexity_threshold)
            .count()
            .max(min_keep)
            .min(max_keep);

        let mut keep_indices: Vec<usize> = indexed[..keep_count]
            .iter()
            .map(|(i, _)| *i)
            .collect();
        keep_indices.sort();

        keep_indices.iter()
            .map(|&i| tokens[i].as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Compress a context block using question-aware perplexity.
/// Tokens that are predictable *given the question* are removed first.
pub fn question_aware_compress(
    context_block: &str,
    question: &str,
    target_tokens: usize,
    compressor: &PromptCompressor,
) -> String {
    let question_terms: std::collections::HashSet<String> = question
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    let tokens: Vec<String> = context_block
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let perplexities: Vec<f32> = tokens.iter()
        .map(|t| {
            let base = 1.0;
            if question_terms.contains(&t.to_lowercase()) {
                base + 2.0
            } else {
                base
            }
        })
        .collect();

    compressor.compress(&tokens, &perplexities)
}
```

### Structured context blocks

Instead of concatenating all context into a flat string, the Theta-tick prompt should be structured into typed blocks with explicit boundaries. Research from Letta/MemGPT shows that structured memory blocks with clear labels improve LLM reasoning consistency by 30-60% compared to unstructured dumps.

```rust
/// A typed context block with a priority level and token budget.
#[derive(Debug, Clone)]
pub struct ContextBlock {
    pub kind: BlockKind,
    pub content: String,
    pub priority: u8,
    pub max_tokens: usize,
    pub token_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    SystemPrompt,
    CurrentEvent,
    CorticalState,
    PositionContext,
    EpisodicMemory,
    SemanticMemory,
    GraphContext,
    ToolResults,
    Instruction,
}

/// Assembles context blocks into a complete Theta-tick prompt.
pub struct ContextAssembler {
    blocks: Vec<ContextBlock>,
    total_budget: usize,
    compressor: PromptCompressor,
}

impl ContextAssembler {
    pub fn new(total_budget: usize) -> Self {
        ContextAssembler {
            blocks: Vec::new(),
            total_budget,
            compressor: PromptCompressor::new(1.5),
        }
    }

    pub fn add_block(&mut self, block: ContextBlock) {
        self.blocks.push(block);
    }

    /// Assemble the final prompt, respecting the total token budget.
    /// Blocks are allocated tokens in priority order.
    pub fn assemble(&mut self) -> String {
        self.blocks.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut remaining_budget = self.total_budget;
        let mut assembled_blocks: Vec<(BlockKind, String)> = Vec::new();

        for block in &self.blocks {
            if remaining_budget == 0 {
                break;
            }

            let tokens = estimate_tokens(&block.content);
            let allocated = tokens.min(block.max_tokens).min(remaining_budget);

            let content = if tokens > allocated {
                truncate_to_tokens(&block.content, allocated)
            } else {
                block.content.clone()
            };

            let actual_tokens = estimate_tokens(&content);
            remaining_budget = remaining_budget.saturating_sub(actual_tokens);
            assembled_blocks.push((block.kind.clone(), content));
        }

        // Reorder for the final prompt (U-curve optimization).
        reorder_for_u_curve(&mut assembled_blocks);

        assembled_blocks.iter()
            .map(|(kind, content)| {
                let tag = block_tag(kind);
                format!("<{}>\n{}\n</{}>", tag, content, tag)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

fn block_tag(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::SystemPrompt => "system",
        BlockKind::CurrentEvent => "event",
        BlockKind::CorticalState => "cortical_state",
        BlockKind::PositionContext => "positions",
        BlockKind::EpisodicMemory => "past_events",
        BlockKind::SemanticMemory => "knowledge",
        BlockKind::GraphContext => "relationships",
        BlockKind::ToolResults => "tool_results",
        BlockKind::Instruction => "instruction",
    }
}

fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

fn truncate_to_tokens(text: &str, target_tokens: usize) -> String {
    let target_chars = target_tokens * 4;
    if text.len() <= target_chars {
        text.to_string()
    } else {
        text[..target_chars].to_string()
    }
}
```

### Token budget allocation

The total token budget for a Theta-tick LLM call depends on the model tier and the Golem's behavioral phase. During the Thriving phase, the Golem can afford larger context windows. During the Declining phase, budgets tighten.

A concrete budget for a typical Theta-tick call using a Sonnet-class model with 8K context:

| Block | Tokens | Priority | Purpose |
|---|---|---|---|
| System prompt | ~500 | 10 | Identity, strategy, risk rules |
| Current event | ~200 | 9 | The transaction being analyzed |
| Position context | ~300 | 8 | Active positions and current PnL |
| CorticalState | ~100 | 7 | Arousal, valence, phase, gas |
| Episodic memory | ~800 | 6 | Top 3-5 similar past events |
| Semantic memory | ~400 | 5 | Protocol facts and baselines |
| Graph context | ~200 | 4 | Topological relationships |
| Instruction | ~200 | 10 | Analysis prompt and output format |
| Reserved for output | ~500 | -- | LLM response |
| **Total** | **~3,200** | | |

Dynamic allocation adjusts the budget based on task type:

```rust
/// Compute token budget allocation based on task type and behavioral phase.
pub fn compute_budget(
    task: &TaskType,
    phase: &BehavioralPhase,
    base_budget: usize,
) -> BudgetAllocation {
    let total = match phase {
        BehavioralPhase::Thriving => base_budget,
        BehavioralPhase::Stable => (base_budget as f32 * 0.8) as usize,
        BehavioralPhase::Declining => (base_budget as f32 * 0.6) as usize,
        BehavioralPhase::Terminal => (base_budget as f32 * 0.4) as usize,
    };

    match task {
        TaskType::InvestigateAnomaly => BudgetAllocation {
            system_prompt: 400,
            current_event: 300,
            position_context: 200,
            cortical_state: 100,
            episodic_memory: total / 3,
            semantic_memory: 300,
            graph_context: 200,
            instruction: 200,
            output_reserve: 500,
        },
        TaskType::RoutineMonitoring => BudgetAllocation {
            system_prompt: 400,
            current_event: 150,
            position_context: 200,
            cortical_state: 80,
            episodic_memory: 400,
            semantic_memory: total / 4,
            graph_context: 100,
            instruction: 150,
            output_reserve: 400,
        },
        TaskType::TradeEvaluation => BudgetAllocation {
            system_prompt: 400,
            current_event: 200,
            position_context: total / 4,
            cortical_state: 100,
            episodic_memory: 400,
            semantic_memory: 300,
            graph_context: 150,
            instruction: 250,
            output_reserve: 600,
        },
    }
}

pub struct BudgetAllocation {
    pub system_prompt: usize,
    pub current_event: usize,
    pub position_context: usize,
    pub cortical_state: usize,
    pub episodic_memory: usize,
    pub semantic_memory: usize,
    pub graph_context: usize,
    pub instruction: usize,
    pub output_reserve: usize,
}
```

### Lost in the middle: the U-curve problem

Liu et al. (2023) demonstrated that LLMs exhibit a U-shaped attention curve: they best attend to information at the beginning and end of the context window, with significant degradation for information in the middle. This effect holds even for models explicitly trained on long contexts.

The practical implication: place the most important information at the boundaries.

```rust
/// Reorder assembled blocks to place high-priority content at the
/// beginning and end of the prompt (U-curve optimization).
fn reorder_for_u_curve(blocks: &mut Vec<(BlockKind, String)>) {
    let mut system = None;
    let mut instruction = None;
    let mut positions = None;
    let mut current_event = None;
    let mut middle = Vec::new();

    for block in blocks.drain(..) {
        match block.0 {
            BlockKind::SystemPrompt => system = Some(block),
            BlockKind::Instruction => instruction = Some(block),
            BlockKind::PositionContext => positions = Some(block),
            BlockKind::CurrentEvent => current_event = Some(block),
            _ => middle.push(block),
        }
    }

    // Reconstruct: system -> current_event -> middle -> positions -> instruction.
    if let Some(s) = system { blocks.push(s); }
    if let Some(e) = current_event { blocks.push(e); }
    blocks.append(&mut middle);
    if let Some(p) = positions { blocks.push(p); }
    if let Some(i) = instruction { blocks.push(i); }
}
```

The research also found that tabular formatting provides a 40.29% average performance gain for data-analytics requests (arXiv:2412.17189). Pool liquidity, price data, and trade history should be formatted as markdown tables, not prose.

### Context delta compression

When the Golem makes multiple Theta-tick LLM calls within a short window, most of the context is unchanged between calls. An I-frame/P-frame approach avoids re-transmitting static context:

- **I-frame (key frame)**: The full context, sent on the first Theta call of each Delta cycle or when the behavioral phase changes.
- **P-frame (delta frame)**: Only the changed blocks -- current event, CorticalState update, new retrievals. The static blocks reference the I-frame via prompt caching.

```rust
/// Track which context blocks have changed since the last I-frame.
pub struct ContextDeltaTracker {
    last_i_frame: HashMap<BlockKind, u64>,
    ticks_since_i_frame: u32,
    i_frame_interval: u32,
}

impl ContextDeltaTracker {
    pub fn new(i_frame_interval: u32) -> Self {
        ContextDeltaTracker {
            last_i_frame: HashMap::new(),
            ticks_since_i_frame: 0,
            i_frame_interval,
        }
    }

    /// Determine whether to send an I-frame or P-frame.
    pub fn frame_type(&mut self, blocks: &[ContextBlock]) -> FrameType {
        self.ticks_since_i_frame += 1;

        if self.ticks_since_i_frame >= self.i_frame_interval
            || self.last_i_frame.is_empty()
        {
            self.last_i_frame.clear();
            for block in blocks {
                let hash = hash_content(&block.content);
                self.last_i_frame.insert(block.kind.clone(), hash);
            }
            self.ticks_since_i_frame = 0;
            FrameType::IFrame
        } else {
            let changed: Vec<BlockKind> = blocks.iter()
                .filter(|b| {
                    let current_hash = hash_content(&b.content);
                    self.last_i_frame.get(&b.kind)
                        .map_or(true, |&prev| prev != current_hash)
                })
                .map(|b| b.kind.clone())
                .collect();
            FrameType::PFrame { changed_blocks: changed }
        }
    }
}

pub enum FrameType {
    /// Full context. All blocks transmitted.
    IFrame,
    /// Delta. Only changed blocks transmitted; static blocks use cached prefix.
    PFrame { changed_blocks: Vec<BlockKind> },
}

fn hash_content(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}
```

### Prompt caching economics

With Anthropic's prompt caching, the first 2,000 tokens of a 3,200-token prompt (the static prefix) are cached after the first call. Subsequent calls within the cache TTL only process the remaining 1,200 dynamic tokens at full cost.

For the Golem making 30-60 Theta calls per Delta cycle:
- First call: Full processing of 3,200 tokens.
- Subsequent 29-59 calls: 1,200 tokens at full cost + 2,000 tokens at cached rate.
- Savings: approximately 40-60% of total inference cost per Delta cycle.

The I-frame/P-frame model naturally maximizes cache hit rate by keeping the static prefix stable across calls.

### Context quality feedback

After each Theta cycle, the Golem evaluates whether the context led to a good decision:

```rust
/// Signal for feedback on context quality.
pub struct ContextQualitySignal {
    pub blocks_included: Vec<BlockKind>,
    pub retrieved_episode_ids: Vec<uuid::Uuid>,
    pub action_recommended: bool,
    pub action_outcome: Option<f32>,
}
```

This signal feeds back to the retrieval weight learning and to the budget allocation. Over time, the Golem learns which context block types are most predictive of good decisions for each task type, and adjusts allocations accordingly.

**Context engineering references:**
- Jiang, H. et al. (2023). "LLMLingua: Compressing Prompts for Accelerated Inference." *EMNLP 2023*. arXiv:2310.05736.
- Jiang, H. et al. (2024). "LongLLMLingua: Accelerating and Enhancing LLMs in Long Context Scenarios." *ACL 2024*. arXiv:2310.06839.
- Liu, N.F. et al. (2023). "Lost in the Middle: How Language Models Use Long Contexts." *TACL 2024*. arXiv:2307.03172.
- Packer, C. et al. (2023). "MemGPT: Towards LLMs as Operating Systems." arXiv:2310.08560.
- Xiao, G. et al. (2024). "Efficient Streaming Language Models with Attention Sinks." *ICLR 2024*. arXiv:2309.17453.
- Mu, J. et al. (2023). "Learning to Compress Prompts with Gist Tokens." *NeurIPS 2023*.
- arXiv:2412.17189 (2024). "Tabular Formatting Improves LLM Performance on Data-Analytics Tasks."
- Karpathy, A. (2025). "Context engineering is the delicate art and science of filling the context window with just the right information for the next step."

---

## Design principles

1. **Local-first.** The golem's own grimoire (LanceDB, SQLite, PLAYBOOK.md) is always the primary knowledge source. Hosted services supplement but never replace local learning. This follows the same principle as prospective memory (Einstein & McDaniel, 2005): the PLAYBOOK.md's strategy-scoped sections serve as prospective memory -- "when X condition occurs, do Y" -- and these must be locally generated and maintained, not downloaded from a remote service [EINSTEIN-MCDANIEL-2005].

2. **Opt-in, not default.** Memory services are disabled by default. Enabling them is a conscious choice with explicit cost implications. Golems work fine without them.

3. **Compression, not copying.** What crosses from inner loop to outer loop is distilled knowledge, not raw data. The genomic bottleneck is a feature, not a limitation. Each compression event (Reflexion to ExpeL to Curator to death reflection) increases the abstraction level, moving from specific instances to generalizable patterns.

4. **Earned confidence.** Inherited knowledge starts at low confidence and must be validated through direct experience to reach full weight. The IKEA Effect ensures golems develop genuine understanding, not cargo-cult compliance. The testing effect [ROEDIGER-KARPICKE-2006] ensures that the validation process itself strengthens the knowledge trace.

5. **Natural decay over artificial destruction.** Knowledge is not burned or destroyed. It fades in retrieval relevance through temporal scoring. Structural knowledge persists. Tactical knowledge fades. The half-lives are calibrated to domain [ARBESMAN-2012]. Interference theory (McGeoch, 1932; Underwood, 1957) provides the mechanism: old heuristics can retroactively interfere with new learning, and new experiences can proactively interfere with old memories. The decay system manages this interference by continuously adjusting the retrieval weight of older entries [MCGEOCH-1932].

6. **Death produces the most valuable knowledge.** The Thanatopsis Protocol's death testament, generated under zero survival pressure, is the single most honest and compressed artifact a golem produces. It receives special provenance and retrieval weight. The Zeigarnik effect ensures that the golem's unfinished business -- its open questions, untested hypotheses, unresolved contradictions -- produces the most generative successor knowledge [ZEIGARNIK-1927].

7. **The system is alive.** Knowledge in Styx is not a static archive. It competes for retrieval relevance, decays with time, and is validated or contradicted by living golems. The knowledge ecology is a dynamic system, not a dead database. Transactive memory theory (Wegner, 1987) describes how groups develop shared memory systems where individuals specialize: the Clade's peer-to-peer sync creates exactly this -- each golem knows what its siblings know, and the group's memory capacity exceeds any individual's [WEGNER-1987].

8. **x402 is the only billing mechanism.** No subscriptions, no tokens, no burning. Every write and query is an x402 micropayment in USDC on Base. Payments determine TTL for storage; retrieval decay is independent of payment.

---

## What makes this more effective than not having it

The system is designed to be measurably better with memory services than without, while being deliberately worse than perfect total recall. The middle ground is where the value lies.

### Without memory services (baseline)

- New golems start from zero (or from a manually imported death bundle file)
- Clade peer-to-peer sync works only with live siblings
- Total clade wipe loses all knowledge (except downloaded files)
- Each generation re-discovers the same lessons through the same expensive mistakes
- No cross-generational compound learning
- The testing effect [ROEDIGER-KARPICKE-2006] works only within a single lifetime -- validated knowledge dies with its validator

### With memory services (enhanced)

- New golems boot with compressed priors from predecessors (Styx Archive)
- Styx provides asynchronous fleet-wide memory (works even when no siblings are online)
- Total clade wipe is recoverable (Vault backups persist)
- Inference calls are augmented with relevant precedents, potentially downgrading expensive T2 calls to cheaper T1 calls
- Cross-generational learning compounds: each generation starts slightly ahead of the last
- The spacing effect [CEPEDA-2006] operates across generations -- knowledge is retrieved, tested, and reconsolidated at intervals that strengthen its durability
- Transactive memory [WEGNER-1987] persists beyond individual golem lifetimes

### With perfect total recall (the Funes failure)

- New golems would be exact copies of their predecessors (Parfit's identity problem [PARFIT-1984] -- in what sense did the predecessor die?)
- Context windows would be overwhelmed with historical detail
- Overfitting to past conditions would make golems brittle to regime shifts
- Proactive interference [MCGEOCH-1932] would be maximized -- every old memory competing with every new observation
- No beneficial variation would enter the lineage
- Schema accommodation [BARTLETT-1932] would be impossible -- the golem's existing schemas would be so rigid with inherited detail that no new information could be integrated
- The mortality thesis would be hollow -- if nothing is lost, nothing truly dies

The middle ground -- compressed inheritance with confidence decay and selective forgetting -- produces what cultural evolution research calls "guided variation" [HENRICH-2015]. Each generation inherits enough to avoid repeating obvious mistakes, but must develop its own understanding through lived experience. The Baldwin Effect [HINTON-NOWLAN-1987] ensures that what transfers is not knowledge itself but the capacity to learn faster.

---

## Product names

| Product             | Full Name      | Short Name | Crate              | Description                                           |
| ------------------- | -------------- | ---------- | ------------------- | ----------------------------------------------------- |
| Local Knowledge     | Golem Grimoire | Grimoire   | `golem-grimoire`    | Local knowledge system (LanceDB, SQLite, PLAYBOOK.md) |
| Persistence + Query | Bardo Styx     | Styx       | `bardo-styx-client` | Four-layer persistence and retrieval network          |

Grimoire is the local knowledge system defined in `01-grimoire.md`. Styx is the hosted network that extends Grimoire across lifetimes (Vault), across fleets (Clade), across the ecosystem (Lethe), and across the marketplace (Commerce). The Grimoire is always primary; Styx extends its capabilities.

---

## Cross-references

| Topic                            | Document                                    | Description                                                                                              |
| -------------------------------- | ------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Local grimoire architecture      | `01-grimoire.md`                            | Full specification of the Grimoire's five entry types, LanceDB + SQLite storage, and Curator cycle       |
| Survival phases and mortality    | `../02-mortality/`                          | The three-clock vitality model and five BehavioralPhases that govern how memory urgency changes near death |
| Death Protocol and death reserve | `../02-mortality/`                          | Four-phase Thanatopsis Protocol (Acceptance, Settlement, Reflection, Legacy) and USDC reserve mechanics   |
| Clade peer-to-peer sync          | `../09-economy/02-clade.md`                 | Styx-relayed knowledge sharing between sibling Golems, including promotion gates and confidence discounts  |
| Knowledge quality gates          | `../01-golem/09-inheritance.md`             | Entry selection, genomic bottleneck compression, and confidence decay rules for generational transfer      |
| Inference gateway integration    | `../12-inference/04-context-engineering.md` | How Grimoire entries are assembled into LLM context windows with budget-aware retrieval                    |
| Grimoire entry types and schema  | `01-grimoire.md`                            | Canonical definitions of Episode, Insight, Heuristic, Warning, CausalLink, and AntiKnowledge types        |
| Styx persistence and retrieval   | `../20-styx/01-architecture.md`             | The three-layer model (Vault/Clade/Lethe) for cross-lifetime and cross-agent knowledge persistence        |
| Styx API and infrastructure      | `../20-styx/02-api-revenue.md`              | REST/WebSocket API surface, x402 billing, data schemas, and deployment infrastructure                     |
| Grimoire economy                 | `06-economy.md`                             | Styx layer economics, confidence discounting, Clade sync costs, marketplace fees, and Pheromone Field     |

---

## References

- [ANDERSON-GREEN-2001] Anderson, M.C. & Green, C. "Suppressing Unwanted Memories by Executive Control." _Nature_, 410, 2001. Demonstrates that executive control processes can actively suppress specific memories below baseline recall. Grounds the Golem's active forgetting mechanisms as biologically plausible.
- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012. Argues that factual knowledge decays at measurable, predictable rates. Motivates the Ebbinghaus-modulated confidence decay on Grimoire entries.
- [BARTLETT-1932] Bartlett, F.C. _Remembering: A Study in Experimental and Social Psychology_. Cambridge University Press, 1932. Established schema theory: new information is assimilated into existing cognitive frameworks, not passively stored. Justifies why inherited knowledge must be re-validated by each successor Golem.
- [BORGES-1942] Borges, J.L. "Funes the Memorious." In _Ficciones_, 1944. The literary case against perfect memory: Funes, who cannot forget, is incapable of abstraction. The founding metaphor for Bardo's forgetting-as-feature design.
- [BORN-WILHELM-2012] Born, J. & Wilhelm, I. "System Consolidation of Memory During Sleep." _Psychological Research_, 76, 2012. Shows that sleep-dependent consolidation selectively favors memories relevant to future action. Informs the Dream engine's consolidation replay scheduling.
- [BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981. Established mood-congruent memory: emotional state at encoding and retrieval affects what is remembered. Grounds the PAD-tagged retrieval system in the Grimoire.
- [CEPEDA-2006] Cepeda, N.J. et al. "Distributed Practice in Verbal Recall Tasks: A Review and Quantitative Synthesis." _Psychological Bulletin_, 132(3), 2006. Meta-analysis showing spaced retrieval produces more durable learning than massed practice. Supports the 50-tick Curator cycle interval.
- [EBBINGHAUS-1885] Ebbinghaus, H. _Memory: A Contribution to Experimental Psychology_. 1885. Discovered the forgetting curve (R = e^(-t/S)) and the spacing effect. The mathematical basis for confidence decay on Grimoire entries.
- [EINSTEIN-MCDANIEL-2005] Einstein, G.O. & McDaniel, M.A. "Prospective Memory: Multiple Retrieval Processes." _Current Directions in Psychological Science_, 14(6), 2005. Distinguishes event-based from time-based prospective memory triggers. Informs the Golem's scheduled vs. event-driven retrieval mechanisms.
- [HARDT-NADER-NADEL-2013] Hardt, O., Nader, K. & Nadel, L. "Decay Happens: The Role of Active Forgetting in Memory." _Trends in Cognitive Sciences_, 17(3), 2013. Shows that forgetting involves well-regulated molecular processes (dopamine-dependent active removal), not passive decay. Supports treating forgetting as a first-class operation.
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance: Myths and Mechanisms." _Cell_, 157(1), 2014. Shows that epigenetic inheritance fades within 2-3 generations and transmitted effects are often deleterious. Justifies the Weismann barrier between Golem experiential memory and inherited knowledge.
- [HENRICH-2015] Henrich, J. _The Secret of Our Success: How Culture Is Driving Human Evolution_. Princeton University Press, 2015. Argues that cultural transmission (not individual intelligence) drives cumulative knowledge. Informs the Clade-level knowledge sharing model.
- [HINTON-2022] Hinton, G. "The Forward-Forward Algorithm: Some Preliminary Investigations." arXiv:2212.13345, 2022. Proposes that mortal computation (hardware-specific learning that dies with the hardware) may outperform immortal weight transfer. The theoretical anchor for Bardo's mortality thesis.
- [HINTON-NOWLAN-1987] Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide Evolution." _Complex Systems_, 1, 1987. Shows that individual learning reshapes the fitness landscape, accelerating evolution, but learned content itself is not inherited. The Baldwin Effect foundation for Golem generational inheritance.
- [MARTIN-EVERITT-HUTTER-2016] Martin, J., Everitt, T. & Hutter, M. "Death and Suicide in Universal Artificial Intelligence." _AGI 2016_. Formalizes how mortality affects optimal policy in artificial agents. Provides the decision-theoretic framework for BehavioralPhase transitions.
- [MCGEOCH-1932] McGeoch, J.A. "Forgetting and the Law of Disuse." _Psychological Review_, 39(4), 1932. Argued that forgetting results from interference, not disuse. Informs the Grimoire's handling of contradictory knowledge entries.
- [MIYAZAKI-2011] Miyazaki, H. Interview, _Edge Magazine_, 2011. Dark Souls design philosophy: death as teaching mechanism, not punishment. Grounds the Bloodstain system where death-sourced knowledge carries a 1.2x retrieval boost.
- [NADER-2000] Nader, K., Schafe, G.E. & LeDoux, J.E. "Fear Memories Require Protein Synthesis in the Amygdala for Reconsolidation after Retrieval." _Nature_, 406, 2000. Proved that retrieved memories become labile and must be reconsolidated. Justifies the Curator's confidence update on retrieval.
- [NIETZSCHE-1887] Nietzsche, F. _On the Genealogy of Morals_. 1887. Second Essay, sect.1. Introduces "active forgetfulness" as a positive capacity, not passive failure. The philosophical foundation for treating forgetting as a feature.
- [NORTON-MOCHON-ARIELY-2012] Norton, M.I., Mochon, D. & Ariely, D. "The IKEA Effect." _Journal of Consumer Psychology_, 22(3), 2012. Shows people overvalue things they build themselves. Motivates requiring successor Golems to re-derive knowledge rather than passively receive it.
- [PARFIT-1984] Parfit, D. _Reasons and Persons_. Oxford University Press, 1984. Argues that personal identity is a matter of degree, not all-or-nothing. Informs the philosophical treatment of Golem identity across generations.
- [PATIHIS-2013] Patihis, L. et al. "False Memories in Highly Superior Autobiographical Memory Individuals." _PNAS_, 110(52), 2013. Demonstrates that near-perfect autobiographical recall does not prevent false memories or improve reasoning. Counters the assumption that more memory equals better decisions.
- [POLANYI-1966] Polanyi, M. _The Tacit Dimension_. University of Chicago Press, 1966. Argues that much knowledge is tacit ("we know more than we can tell"). Motivates the procedural knowledge type in the Grimoire.
- [RICHARDS-FRANKLAND-2017] Richards, B.A. & Frankland, P.W. "The Persistence and Transience of Memory." _Neuron_, 94(6), 2017. Reframes memory's purpose as decision optimization, not information preservation. Forgetting is equivalent to regularization in neural networks. The primary neuroscience reference for the Golem memory architecture.
- [ROEDIGER-KARPICKE-2006] Roediger, H.L. & Karpicke, J.D. "Test-Enhanced Learning: Taking Memory Tests Improves Long-Term Retention." _Psychological Science_, 17(3), 2006. Shows that retrieval practice strengthens memory more than re-study. Justifies the Curator cycle's re-validation requirement for inherited knowledge.
- [SHUVAEV-2024] Shuvaev, S. et al. "Encoding Innate Ability Through a Genomic Bottleneck." _PNAS_, 121(39), 2024. Demonstrates that genomic-scale compression acts as a regularizer that enhances transfer learning. The direct biological evidence for the 2048-entry genomic bottleneck at Golem death.
- [SPENCE-1973] Spence, M. "Job Market Signaling." _QJE_, 87(3), 1973. Established signaling theory: costly signals credibly convey information. Informs the identity staking model.
- [WEGNER-1987] Wegner, D.M. "Transactive Memory: A Contemporary Analysis of the Group Mind." In _Theories of Group Behavior_, Springer, 1987. Argues that groups distribute memory across members, with each member knowing who knows what. The theoretical basis for Clade knowledge distribution.
- [ZEIGARNIK-1927] Zeigarnik, B. "On Finished and Unfinished Tasks." _Psychologische Forschung_, 9, 1927. Showed that incomplete tasks are remembered better than completed ones. Informs the Golem's prioritization of unresolved knowledge gaps.

## Cross-subsystem dependencies

| Direction     | Subsystem | What                                  | Where                    |
| ------------- | --------- | ------------------------------------- | ------------------------ |
| Receives from | Dreams    | Dream entries (provenance: `"dream"`) | `01-grimoire.md`         |
| Receives from | Mortality | Three-clock expiry triggers           | Styx spec                |
| Provides to   | Emotions  | Episode outcomes for PAD computation  | `02-emotional-memory.md` |
| Provides to   | Runtime   | Knowledge retrieval for decisions     | Styx spec                |
| Provides to   | Mortality | Grimoire export for death testament   | Styx Archive               |
| Syncs via     | Clade     | Bidirectional knowledge exchange      | `06-economy.md`          |

### Shared constants

| Constant                               | Value                   | Shared With |
| -------------------------------------- | ----------------------- | ----------- |
| Dream confidence floor                 | 0.3                     | Dreams      |
| Dream-to-Clade push threshold          | 0.5 (after validation)  | Dreams      |
| Dream validation window                | 7d confirm / 14d expire | Dreams      |
| Stochastic clock legacy push threshold | hayflickRatio > 0.85    | Mortality   |
| Legacy push confidence drop            | 0.6 -> 0.4              | Mortality   |
| Phage pruning interval                 | 100 ticks               | Mortality   |
| Generational confidence decay          | -0.05 per generation    | Mortality   |
| Memory service costs                   | < 5% daily budget       | Mortality   |
