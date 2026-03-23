# 03c -- State Management [SPEC]

**Snapshots, Context Delta Compression, Episodic Replay, Metrics and Tracing**

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-state` | **Layer**: 0 (FOUNDATION)
>
> Cross-references: [03-mind.md](./03-mind.md) (overview), [02-heartbeat.md](./02-heartbeat.md) (heartbeat pipeline), [18-cortical-state.md](./18-cortical-state.md) (CorticalState), [14-context-governor.md](./14-context-governor.md) (context assembly)
>
> Sources: `03-agent-runtime/07-state-snapshot`, `03-agent-runtime/08-metrics-tracing`, `03-agent-runtime/09-context-delta`, `03-agent-runtime/10-episodic-replay`

> **Reader orientation:** This document specifies four state management mechanisms for the Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM): content-addressed state snapshots, context delta compression (I-frame/P-frame), episodic replay for case-based reasoning, and metrics/tracing. It belongs to the `01-golem` cognition layer, in the `golem-state` crate. These are the persistence and observability infrastructure that make the Heartbeat (the 9-step decision cycle) auditable and recoverable. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## 1. State Snapshots

Content-addressed, full agent state snapshots for forensic comparison. Distinct from RollbackCheckpoints (forensics vs. recovery): snapshots are for understanding what happened, checkpoints are for undoing it.

### Theoretical grounding

Merkle (1987) established content-addressed storage using cryptographic hashing. A Blake3 hash over the serialized snapshot content serves as both the snapshot ID and its integrity check. Any tampering changes the ID.

Fowler (2005) described Event Sourcing: state is derived by replaying an event log from a known snapshot. `StateSnapshot` bounds how far back the event log must be read to reconstruct agent state.

Kleppmann (2017, Chapter 11) covers the snapshot-plus-event-log duality: snapshots provide fast startup; event logs provide full auditability.

### The StateSnapshot struct (source implementation)

The source implementation uses content-addressed IDs (the `id` field IS the Blake3 hash of all content fields excluding `created_at_unix_secs`) and includes a `CapabilitySetSnapshot` with capability bitmask.

```rust
// crates/golem-runtime/src/state_snapshot.rs

use std::collections::BTreeMap;
use crate::rollback_checkpoint::{
    CorticalSnapshot, ContextSnapshot,
    PositionLedgerSnapshot, GrimoireRef,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilitySetSnapshot {
    /// Bitmask of active capability flags at snapshot time.
    pub capability_flags: u64,
}

/// Content-addressed full agent state capture.
/// The `id` field IS the Blake3 hash of all content fields
/// (excluding `created_at_unix_secs`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateSnapshot {
    pub id: [u8; 32],
    pub tick: u64,
    pub created_at_unix_secs: u64,
    pub context: ContextSnapshot,
    pub cortical: CorticalSnapshot,
    pub ledger: PositionLedgerSnapshot,
    pub grimoire_ref: GrimoireRef,
    pub capability_set: CapabilitySetSnapshot,
}

impl StateSnapshot {
    pub fn capture(
        tick: u64,
        context: ContextSnapshot,
        cortical: CorticalSnapshot,
        ledger: PositionLedgerSnapshot,
        grimoire_ref: GrimoireRef,
        capability_set: CapabilitySetSnapshot,
    ) -> Self {
        let content = (
            &context, &cortical, &ledger,
            &grimoire_ref, &capability_set, tick,
        );
        let bytes = ciborium_ser(&content);
        let id: [u8; 32] = blake3::hash(&bytes).into();

        let created_at_unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        StateSnapshot {
            id, tick, created_at_unix_secs,
            context, cortical, ledger, grimoire_ref, capability_set,
        }
    }

    pub fn verify_integrity(&self) -> bool {
        let content = (
            &self.context, &self.cortical, &self.ledger,
            &self.grimoire_ref, &self.capability_set, self.tick,
        );
        let bytes = ciborium_ser(&content);
        let computed: [u8; 32] = blake3::hash(&bytes).into();
        computed == self.id
    }
}
```

### State diff (source implementation)

```rust
pub struct StateDiff {
    pub tick_delta: i64,
    pub pleasure_delta: Option<f32>,
    pub arousal_delta: Option<f32>,
    pub tokens_used_delta: Option<i32>,
    pub grimoire_changed: bool,
}

impl StateDiff {
    pub fn compute(
        before: &StateSnapshot, after: &StateSnapshot,
    ) -> Self {
        let thresh = 0.001_f32;
        StateDiff {
            tick_delta: after.tick as i64 - before.tick as i64,
            pleasure_delta: {
                let d = after.cortical.pleasure - before.cortical.pleasure;
                if d.abs() > thresh { Some(d) } else { None }
            },
            arousal_delta: {
                let d = after.cortical.arousal - before.cortical.arousal;
                if d.abs() > thresh { Some(d) } else { None }
            },
            tokens_used_delta: {
                let d = after.context.tokens_used as i32
                    - before.context.tokens_used as i32;
                if d != 0 { Some(d) } else { None }
            },
            grimoire_changed: before.grimoire_ref.content_hash
                != after.grimoire_ref.content_hash,
        }
    }
}
```

### Content-addressed SnapshotStore

The source uses a `HashMap<[u8;32], Vec<u8>>` for content-addressed storage with `get_by_hash()` lookup and nearest-tick queries.

```rust
pub struct SnapshotStore {
    store: std::collections::HashMap<[u8; 32], Vec<u8>>,
    tick_index: BTreeMap<u64, [u8; 32]>,
    max_retained: usize,
}

impl SnapshotStore {
    pub fn new(max_retained: usize) -> Self {
        SnapshotStore {
            store: Default::default(),
            tick_index: BTreeMap::new(),
            max_retained,
        }
    }

    pub fn insert(&mut self, snapshot: StateSnapshot) {
        let hash = snapshot.id;
        let tick = snapshot.tick;
        self.store.insert(hash, ciborium_ser(&snapshot));
        self.tick_index.insert(tick, hash);
        self.evict_oldest_if_needed();
    }

    pub fn get_by_hash(
        &self, hash: &[u8; 32],
    ) -> Option<StateSnapshot> {
        let bytes = self.store.get(hash)?;
        ciborium_de(bytes).ok()
    }

    pub fn get_at_tick(&self, tick: u64) -> Option<StateSnapshot> {
        let hash = self.tick_index.get(&tick)?;
        self.get_by_hash(hash)
    }

    /// Nearest snapshot at or before `tick`.
    pub fn get_nearest_before(
        &self, tick: u64,
    ) -> Option<StateSnapshot> {
        let (_, hash) = self.tick_index.range(..=tick).next_back()?;
        self.get_by_hash(hash)
    }

    fn evict_oldest_if_needed(&mut self) {
        while self.store.len() > self.max_retained {
            if let Some((&oldest_tick, _)) = self.tick_index.iter().next()
            {
                let hash = self.tick_index.remove(&oldest_tick).unwrap();
                self.store.remove(&hash);
            } else {
                break;
            }
        }
    }
}
```

### Use cases

- **Replay debugging**: Load the snapshot from before the bad tick, replace a parameter, and re-run the cognitive cycle to see what would have happened.
- **Pre/post compaction diffing**: Take a snapshot before and after `compact()`. `StateDiff::compute()` reveals exactly what the compaction changed.
- **On-chain attestation**: Anchor the Blake3 hash via Styx. Anyone can later verify: "at tick 4820, the agent's full state hashed to X."
- **Cross-session comparison**: Two Golems running different strategies can compare their state snapshots at matching ticks to identify where their cognitive states diverged.

Snapshots capture at delta frequency (every ~50 theta ticks). At 200 retained snapshots and ~2 KB each, the store uses ~400 KB.

### Relationship to HomeostasisRegulator (from source 04-homeostasis)

The `HomeostasisRegulator` (see [03b-cognitive-mechanisms.md](./03b-cognitive-mechanisms.md) Section 4) reads CorticalState signals that are also captured in `CorticalSnapshot`. When the regulator detects persistent deviations (e.g., `economic_vitality` below rolling average for 10+ ticks), it nudges AgentConfig knobs. The `StateDiff::compute()` method can reveal the CorticalState changes that triggered homeostatic corrections by comparing pre/post snapshots around the correction point.

The regulator's `SetDreamMode(Intensive)` action lowers SleepPressure thresholds, which indirectly affects snapshot frequency: more frequent consolidation means more delta-frequency snapshots capturing the Golem's state transitions.

---

## 2. Context Delta Compression

The Context Governor assembles a CognitiveWorkspace each theta tick for LLM calls. Most of the workspace is stable between consecutive ticks. Context delta compression uses an I-frame/P-frame scheme (borrowed from video compression) to reduce re-tokenization cost and make workspace changes explicit.

### Theoretical grounding

The I-frame/P-frame architecture comes from ITU-T H.261 (1990; documented in Richardson, 2002). The invariant: every P-frame is valid only relative to a specific base I-frame. `ContextDelta.base_hash` plays this role -- the delta is applicable only to the specific compacted context identified by that hash.

Myers (1986) described the O(ND) difference algorithm underlying `diff` and `git diff`. `DeltaCompressor::compute_delta()` computes which context segments changed. The O(ND) complexity bound confirms that per-tick delta computation is feasible even for contexts with many segments.

Cai et al. (2014) formalized incremental computation in the lambda calculus: given changes to a function's inputs, compute the function's output change without re-deriving everything from scratch.

Mu et al. (2024) demonstrated context compression with minimal task-performance loss in their GIST work. The token savings from `ContextDelta` -- avoiding re-summarization of stable segments -- reduce inference cost in the same way GIST tokens do.

### Segment types (source implementation)

Context is divided into named segments. Each segment has an ID, a token count, a content hash, and the actual text.

```rust
// crates/golem-core/src/context_delta.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SegmentId(pub &'static str);

pub const SEG_POSITION_SUMMARY: SegmentId = SegmentId("position_summary");
pub const SEG_RECENT_EPISODES: SegmentId  = SegmentId("recent_episodes");
pub const SEG_STRATEGY_HEADER: SegmentId  = SegmentId("strategy_header");
pub const SEG_MARKET_STATE: SegmentId     = SegmentId("market_state");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaKind { Added, Modified, Removed }

#[derive(Debug, Clone)]
pub struct ContextSegment {
    pub id: SegmentId,
    pub token_count: u32,
    pub content_hash: [u8; 16],
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ContextSegmentDelta {
    pub segment_id: SegmentId,
    pub kind: DeltaKind,
    /// Present for Added and Modified; absent for Removed.
    pub content: Option<ContextSegment>,
}
```

### The delta structure (source implementation)

```rust
#[derive(Debug, Clone)]
pub struct ContextDelta {
    /// Blake3 hash of the base compacted context.
    pub base_hash: [u8; 32],
    pub base_tick: u64,
    pub changed_segments: Vec<ContextSegmentDelta>,
    /// Total tokens changed since base.
    /// Used to trigger forced full recompact.
    pub tokens_delta: u32,
}
```

### CompactionRecord diff() implementation

```rust
pub struct CompactionRecord {
    pub tick: u64,
    pub content_hash: [u8; 32],
    pub segments: Vec<ContextSegment>,
}

impl CompactionRecord {
    pub fn diff(&self, current: &ActiveContext) -> ContextDelta {
        let mut changed = Vec::new();
        let mut tokens_delta = 0u32;

        // Check each base segment against current state
        for base_seg in &self.segments {
            match current.find_segment(&base_seg.id) {
                Some(cur) if cur.content_hash != base_seg.content_hash => {
                    tokens_delta +=
                        cur.token_count.abs_diff(base_seg.token_count);
                    changed.push(ContextSegmentDelta {
                        segment_id: base_seg.id.clone(),
                        kind: DeltaKind::Modified,
                        content: Some(cur.clone()),
                    });
                }
                None => {
                    tokens_delta += base_seg.token_count;
                    changed.push(ContextSegmentDelta {
                        segment_id: base_seg.id.clone(),
                        kind: DeltaKind::Removed,
                        content: None,
                    });
                }
                _ => {} // Unchanged
            }
        }

        // Check for segments added since last compaction
        for cur in current.segments() {
            if !self.segments.iter().any(|s| s.id == cur.id) {
                tokens_delta += cur.token_count;
                changed.push(ContextSegmentDelta {
                    segment_id: cur.id.clone(),
                    kind: DeltaKind::Added,
                    content: Some(cur.clone()),
                });
            }
        }

        ContextDelta {
            base_hash: self.content_hash,
            base_tick: self.tick,
            changed_segments: changed,
            tokens_delta,
        }
    }
}
```

### The delta compressor (source implementation)

The source uses `delta_threshold` as a fraction of the token budget (not cosine similarity). When delta tokens exceed this fraction, a full I-frame is forced.

```rust
pub struct DeltaCompressor {
    last_compaction: Option<CompactionRecord>,
    delta_run_count: u32,
    /// Force full recompact after this many consecutive delta runs.
    max_delta_runs: u32,
    /// Force full recompact when delta tokens exceed this fraction of budget.
    delta_threshold: f32,
}

impl DeltaCompressor {
    pub fn new(max_delta_runs: u32, delta_threshold: f32) -> Self {
        DeltaCompressor {
            last_compaction: None,
            delta_run_count: 0,
            max_delta_runs,
            delta_threshold,
        }
    }

    /// Returns None when a full recompact is required.
    /// Called from ContextCompaction before each consolidation run.
    pub fn compute_delta(
        &mut self, current: &ActiveContext, budget: u32,
    ) -> Option<ContextDelta> {
        let last = self.last_compaction.as_ref()?;
        if self.delta_run_count >= self.max_delta_runs {
            self.delta_run_count = 0;
            return None;
        }
        let delta = last.diff(current);
        let fraction = delta.tokens_delta as f32 / budget.max(1) as f32;
        if fraction >= self.delta_threshold {
            return None;
        }
        self.delta_run_count += 1;
        Some(delta)
    }

    /// Apply a delta on top of the last compacted representation.
    pub fn apply_delta(
        &self, delta: &ContextDelta, base: &CompactedContext,
    ) -> ActiveContext {
        base.apply(delta)
    }

    /// Record a completed full compaction as the new base.
    pub fn record_compaction(&mut self, record: CompactionRecord) {
        self.delta_run_count = 0;
        self.last_compaction = Some(record);
    }

    /// Called on CorticalRegimeChange events.
    /// Forces the next compaction to be a full I-frame.
    pub fn on_regime_change(&mut self) {
        self.delta_run_count = self.max_delta_runs;
    }
}
```

### Decision flow

```
SleepPressure.needs_consolidation() == true
    |
    v
DeltaCompressor.compute_delta(current_context, budget)
    |
    +-- Some(delta) --> apply_delta(delta, base) --> done (cheap P-frame)
    |
    +-- None ---------> run full compact() --------> record_compaction()
                                                     (expensive I-frame)
```

### Three forced I-frame cases

`compute_delta()` returns `None` (forcing a full I-frame) in three cases:

1. **No base compaction exists yet** -- first run is always a full I-frame.
2. **Too many consecutive delta runs** -- exceeds `max_delta_runs` (default 10). This prevents error accumulation from drift against the same base.
3. **Delta tokens exceed `delta_threshold` fraction of budget** (default 0.3). If 30%+ of the context budget worth of tokens changed, a delta is barely cheaper than a full recompact.

### Token savings

In a stable regime with a 32,000-token context budget:

| Scenario | Full compact cost | Delta cost | Savings |
|----------|------------------|------------|---------|
| 5% of context changed | 32,000 tokens input | ~1,600 tokens | 95% |
| 15% of context changed | 32,000 tokens input | ~4,800 tokens | 85% |
| 30% of context changed | 32,000 tokens input | Full recompact triggered | 0% |

Over 10 consolidation cycles in a stable regime, 9 may be deltas and 1 a forced I-frame. That is roughly 90% fewer compaction tokens than 10 full recompacts.

### Regime change handling

When `CorticalState` emits a regime change event, `DeltaCompressor::on_regime_change()` sets `delta_run_count` to `max_delta_runs`, which forces the next `compute_delta()` call to return `None`. A regime change invalidates assumptions about which segments are "stable." Starting from a fresh I-frame gives subsequent deltas a clean, accurate base.

---

## 3. Episodic Replay

Case-based reasoning for the deliberation step (Step 5, DELIBERATE). When the Golem faces a decision, episodic replay searches for past episodes with similar conditions and injects their narrative into the LLM's context.

### Theoretical grounding

Tulving (1972) drew the foundational distinction between semantic memory (facts) and episodic memory (personally experienced events in spatio-temporal context). Grimoire retrieval is semantic; `EpisodicReplay` constructs the episodic frame -- the "I was there" wrapper that places retrieved facts in the agent's own experience timeline.

Schank (1982) introduced Scripts and Memory Organization Packets (MOPs): past experiences are indexed by abstract situation structures. When a situation matches a known script, the script guides expectation and action. `EpisodeQuery.regime x affect` is the situation index; `NarrativeTemplate` renders the matched script.

Wilson and McNaughton (1994) demonstrated neural replay during sleep: hippocampal ensemble patterns active during waking behavior are reactivated during subsequent sleep. Dream consolidation in Bardo performs the computational analog. `EpisodicReplay` is the waking counterpart -- replaying past episodes into working context before deliberation.

Aamodt and Plaza (1994) formalized the CBR (Case-Based Reasoning) cycle: Retrieve, Reuse, Revise, Retain. `EpisodicReplay` implements Retrieve and Reuse. The Golem performs Revise during deliberation; Retain is handled by Grimoire storage of the completed episode after the tick.

Lewis et al. (2020) established that grounding LLM generation in retrieved context (RAG) significantly improves factual accuracy. `EpisodicReplay` is a domain-specific RAG implementation: retrieved episodes ground the Golem's deliberation in its own operational history.

### Episode types (source implementation)

```rust
// crates/golem-core/src/episodic_replay.rs

#[derive(Debug, Clone, Copy)]
pub enum MarketRegime { Stable, Trending, Volatile, Crisis }

#[derive(Debug, Clone)]
pub struct ActionSummary {
    pub action_type: &'static str,
    /// +1 = positive, 0 = neutral, -1 = negative
    pub outcome_direction: i8,
}

#[derive(Debug, Clone)]
pub struct OutcomeSummary {
    pub pnl_direction: i8,
    pub avoided_risk: bool,
}

/// Compressed representation of one past episode.
/// 5-8 words when rendered.
#[derive(Debug, Clone)]
pub struct EpisodeSummary {
    pub tick: u64,
    pub regime: MarketRegime,
    pub top_action: ActionSummary,
    pub outcome: OutcomeSummary,
    pub accuracy_at_time: f32,
}

impl EpisodeSummary {
    /// Compress a raw Episode into a lean summary.
    /// Only deliberation-relevant fields retained.
    pub fn from_episode(
        episode: &Episode, _query: &EpisodeQuery,
    ) -> Self {
        let top_action = episode.actions.iter()
            .max_by_key(|a| a.significance)
            .map(|a| ActionSummary {
                action_type: a.action_type,
                outcome_direction: a.outcome_direction,
            })
            .unwrap_or(ActionSummary {
                action_type: "observe",
                outcome_direction: 0,
            });

        EpisodeSummary {
            tick: episode.tick,
            regime: episode.regime,
            top_action,
            outcome: OutcomeSummary {
                pnl_direction: episode.pnl_direction,
                avoided_risk: episode.avoided_risk,
            },
            accuracy_at_time: episode.accuracy,
        }
    }
}
```

### Query and template (source implementation)

```rust
/// Current situation for Grimoire episode retrieval.
pub struct EpisodeQuery {
    pub regime: MarketRegime,
    /// Affect triple from CorticalState.
    pub affect: (f32, f32, f32),
    /// Task embedding from golem-inference for semantic similarity.
    pub task_embedding: Vec<f32>,
    pub current_tick: u64,
}

/// Static template for rendering episode summaries.
/// No heap allocation until render().
pub struct NarrativeTemplate {
    /// Slots: {count}, {regime}, {lines}.
    pub template_str: &'static str,
}

impl NarrativeTemplate {
    pub fn render(
        &self,
        summaries: &[EpisodeSummary],
        query: &EpisodeQuery,
    ) -> String {
        let regime_str = match query.regime {
            MarketRegime::Stable   => "stable",
            MarketRegime::Trending => "trending",
            MarketRegime::Volatile => "volatile",
            MarketRegime::Crisis   => "crisis",
        };

        let lines: String = summaries.iter().map(|s| {
            let outcome = match s.outcome.pnl_direction {
                d if d > 0 => "gain",
                d if d < 0 => "loss",
                _          => "flat",
            };
            format!(
                "  - tick {}: {}, {} -> {}\n",
                s.tick,
                format!("{:?}", s.regime).to_lowercase(),
                s.top_action.action_type,
                outcome,
            )
        }).collect();

        self.template_str
            .replace("{count}", &summaries.len().to_string())
            .replace("{regime}", regime_str)
            .replace("{lines}", &lines)
    }
}
```

### Token budget enforcement

```rust
/// Token-counted narrative prefix ready for context injection.
pub struct NarrativePrefix {
    pub text: String,
    pub tokens: u32,
    pub episode_count: usize,
}
```

### The replay engine (source implementation)

```rust
pub struct EpisodicReplay {
    top_k: usize,
    /// Maximum tokens the narrative prefix may consume.
    token_budget: u32,
    template: NarrativeTemplate,
}

impl EpisodicReplay {
    pub fn new(
        top_k: usize,
        token_budget: u32,
        template: NarrativeTemplate,
    ) -> Self {
        EpisodicReplay { top_k, token_budget, template }
    }

    /// Step 4 (Retrieve) integration point.
    /// Returns a prefix ready for prepending to the deliberation
    /// context assembled in Step 5.
    pub async fn build_prefix(
        &self,
        query: EpisodeQuery,
        grimoire: &dyn Grimoire,
    ) -> NarrativePrefix {
        let episodes = grimoire
            .retrieve_similar_episodes(&query, self.top_k)
            .await;

        if episodes.is_empty() {
            return NarrativePrefix {
                text: String::new(),
                tokens: 0,
                episode_count: 0,
            };
        }

        let summaries: Vec<EpisodeSummary> = episodes.iter()
            .map(|e| EpisodeSummary::from_episode(e, &query))
            .collect();

        let text = self.template.render(&summaries, &query);
        let tokens = count_tokens(&text);

        let (text, tokens) = if tokens > self.token_budget {
            let trimmed = trim_to_token_budget(&text, self.token_budget);
            let t = count_tokens(&trimmed);
            (trimmed, t)
        } else {
            (text, tokens)
        };

        NarrativePrefix {
            text,
            tokens,
            episode_count: summaries.len(),
        }
    }
}
```

### Grimoire trait and supporting types

```rust
pub struct Episode {
    pub tick: u64,
    pub regime: MarketRegime,
    pub actions: Vec<EpisodeAction>,
    pub pnl_direction: i8,
    pub avoided_risk: bool,
    pub accuracy: f32,
}

pub struct EpisodeAction {
    pub action_type: &'static str,
    pub outcome_direction: i8,
    pub significance: u32,
}

#[async_trait::async_trait]
pub trait Grimoire: Send + Sync {
    async fn retrieve_similar_episodes(
        &self, query: &EpisodeQuery, top_k: usize,
    ) -> Vec<Episode>;
}

fn count_tokens(text: &str) -> u32 {
    (text.len() / 4) as u32
}

fn trim_to_token_budget(text: &str, budget: u32) -> String {
    let approx_chars = (budget * 4) as usize;
    if text.len() <= approx_chars {
        text.to_owned()
    } else {
        text[..approx_chars].to_owned()
    }
}
```

### Example rendered prefix

Given a query with regime=Volatile and three matching episodes, the template `"## Past experience\nIn {count} similar sessions ({regime}):\n{lines}"` renders:

```
## Past experience
In 3 similar sessions (volatile):
  - tick 4812: volatile, reduce_exposure -> gain
  - tick 3901: volatile, hold -> loss
  - tick 2744: volatile, reduce_exposure -> gain
```

This occupies roughly 30-40 tokens. The Golem sees the pattern immediately: reducing exposure in volatile conditions succeeded twice; holding once led to a loss.

### Relevance scoring (source implementation)

Grimoire retrieval uses a composite similarity measure across three dimensions with configurable weights:

- **Regime match** (weight 0.4): Episodes from the same regime score highest.
- **Affect distance** (weight 0.3): Euclidean distance in the PAD (pleasure, arousal, dominance) space. Episodes where the Golem was in a similar emotional state are ranked higher.
- **Task embedding similarity** (weight 0.3): Cosine similarity between the current task embedding and the episode's task embedding.

### Recency weighting formula

Among episodes with similar relevance scores, more recent episodes are preferred:

```
recency_weight = 1.0 / (1.0 + (current_tick - episode_tick) as f32 / 10000.0)
```

A 5000-tick-old episode scores at 67% of an identical current-tick episode. The decay is intentionally slow -- old episodes with high relevance should still surface.

Episodic replay runs only on T1+ ticks (when the LLM is invoked). On T0 ticks, no context assembly happens, so no replay is needed.

---

## 4. Metrics and Tracing

### Theoretical grounding

The SRE handbook (Beyer et al., 2016) defines the Four Golden Signals: latency, traffic, errors, saturation. `PhaseLatencies` and `wall_time_ms` cover latency; `tokens_used / tokens_budget` covers saturation. These four signals inform which fields are mandatory in `TickMetrics`.

Prometheus (Volz et al., 2015) established pull-based scraping with label-keyed time series. The optional `PrometheusSink` implements an OpenMetrics-compatible text format for direct Prometheus scraping.

Majors, Fong-Jones, and Miranda (2022) describe wide-event observability: emit all contextual fields in a single event per request. `TickMetrics` is a wide event -- one structured record per tick containing all operational fields.

Sigelman et al. (2010) established the tree-of-spans model with Google's Dapper: a trace is a tree of timestamped spans, each representing one unit of work.

The W3C Trace Context specification (2021) defines `traceparent` (version, trace-id, parent-id, flags) and `tracestate` (vendor-specific).

The OpenTelemetry specification (CNCF, 2019-present) provides the SDK-agnostic standard for traces, metrics, and logs.

### Part 1: MetricsEmitter

#### Phase timer (source implementation)

The source uses named stop methods for each heartbeat phase, producing a `PhaseLatencies` struct with 7 named u16 fields.

```rust
// crates/golem-runtime/src/metrics_emitter.rs

use std::time::Instant;

/// Measures wall time at each phase boundary in HeartbeatFSM.
pub struct PhaseTimer {
    started: Option<Instant>,
    pub predict_ms: u16,
    pub appraise_ms: u16,
    pub gate_ms: u16,
    pub retrieve_ms: u16,
    pub deliberate_ms: u16,
    pub act_ms: u16,
    pub reflect_ms: u16,
}

impl PhaseTimer {
    pub fn new() -> Self {
        PhaseTimer {
            started: None,
            predict_ms: 0, appraise_ms: 0, gate_ms: 0,
            retrieve_ms: 0, deliberate_ms: 0, act_ms: 0, reflect_ms: 0,
        }
    }

    pub fn start(&mut self) { self.started = Some(Instant::now()); }

    fn elapsed_ms(&mut self) -> u16 {
        let ms = self.started.take()
            .map(|s| s.elapsed().as_millis())
            .unwrap_or(0);
        ms.min(u16::MAX as u128) as u16
    }

    pub fn stop_predict(&mut self)    { self.predict_ms    = self.elapsed_ms(); }
    pub fn stop_appraise(&mut self)   { self.appraise_ms   = self.elapsed_ms(); }
    pub fn stop_gate(&mut self)       { self.gate_ms       = self.elapsed_ms(); }
    pub fn stop_retrieve(&mut self)   { self.retrieve_ms   = self.elapsed_ms(); }
    pub fn stop_deliberate(&mut self) { self.deliberate_ms = self.elapsed_ms(); }
    pub fn stop_act(&mut self)        { self.act_ms        = self.elapsed_ms(); }
    pub fn stop_reflect(&mut self)    { self.reflect_ms    = self.elapsed_ms(); }

    pub fn into_latencies(self) -> PhaseLatencies {
        PhaseLatencies {
            predict_ms: self.predict_ms,
            appraise_ms: self.appraise_ms,
            gate_ms: self.gate_ms,
            retrieve_ms: self.retrieve_ms,
            deliberate_ms: self.deliberate_ms,
            act_ms: self.act_ms,
            reflect_ms: self.reflect_ms,
        }
    }
}
```

#### PhaseLatencies and TickMetrics (source implementation)

```rust
#[derive(Debug, Clone, Copy)]
pub enum TickKind { Theta, Gamma, Delta, WakeupGamma, WakeupTheta }

#[derive(Debug, Clone, Copy)]
pub enum InferenceTier { Haiku, Sonnet, Opus }

#[derive(Debug, Clone, Copy)]
pub enum MarketRegime { Stable, Trending, Volatile, Crisis }

#[derive(Debug, Clone)]
pub struct PhaseLatencies {
    pub predict_ms: u16,
    pub appraise_ms: u16,
    pub gate_ms: u16,
    pub retrieve_ms: u16,
    pub deliberate_ms: u16,
    pub act_ms: u16,
    pub reflect_ms: u16,
}

impl PhaseLatencies {
    pub fn total_ms(&self) -> u32 {
        self.predict_ms as u32 + self.appraise_ms as u32
            + self.gate_ms as u32 + self.retrieve_ms as u32
            + self.deliberate_ms as u32 + self.act_ms as u32
            + self.reflect_ms as u32
    }
}

/// One record per tick. All operational fields in one struct.
#[derive(Debug, Clone)]
pub struct TickMetrics {
    pub tick: u64,
    pub tick_kind: TickKind,
    pub wall_time_ms: u32,
    pub phase_latencies: PhaseLatencies,
    pub inference_cost_usd: f64,
    pub inference_tier: InferenceTier,
    pub tokens_used: u32,
    pub tokens_budget: u32,
    pub prediction_accuracy_delta: f32,
    pub budget_fraction_used: f32,
    pub regime: MarketRegime,
    pub actions_taken: u8,
    pub grimoire_hits: u8,
    pub grimoire_misses: u8,
    pub sleep_pressure: f32,
}
```

#### Generic RingBuffer and emitter (source implementation)

```rust
/// Fixed-capacity ring buffer. `push` overwrites oldest entry when full.
pub struct RingBuffer<T> {
    buf: Vec<Option<T>>,
    head: usize,
    len: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        RingBuffer { buf: vec![None; capacity], head: 0, len: 0 }
    }

    pub fn push(&mut self, item: T) {
        let cap = self.buf.len();
        self.buf[self.head % cap] = Some(item);
        self.head = (self.head + 1) % cap;
        self.len = (self.len + 1).min(cap);
    }

    pub fn last_n(&self, n: usize) -> Vec<T> {
        let cap = self.buf.len();
        let take = n.min(self.len);
        let mut result = Vec::with_capacity(take);
        for i in (0..take).rev() {
            let idx = (self.head + cap - 1 - i) % cap;
            if let Some(ref item) = self.buf[idx] {
                result.push(item.clone());
            }
        }
        result
    }
}

pub struct MetricsEmitter {
    buffer: RingBuffer<TickMetrics>,
    prometheus_sink: Option<std::sync::Arc<std::sync::Mutex<PrometheusSink>>>,
}

impl MetricsEmitter {
    pub fn new(ring_capacity: usize) -> Self {
        MetricsEmitter {
            buffer: RingBuffer::new(ring_capacity),
            prometheus_sink: None,
        }
    }

    pub fn with_prometheus_sink(
        mut self,
        sink: std::sync::Arc<std::sync::Mutex<PrometheusSink>>,
    ) -> Self {
        self.prometheus_sink = Some(sink);
        self
    }

    /// Called at end of Step 7 (Reflect) in HeartbeatFSM.
    pub fn emit(&mut self, metrics: TickMetrics) {
        if let Some(sink) = &self.prometheus_sink {
            sink.lock().unwrap().write(&metrics);
        }
        self.buffer.push(metrics);
    }

    pub fn recent(&self, n: usize) -> Vec<TickMetrics> {
        self.buffer.last_n(n)
    }

    pub fn rolling_cost(&self, n: usize) -> f64 {
        let recent = self.recent(n);
        if recent.is_empty() { return 0.0; }
        recent.iter().map(|m| m.inference_cost_usd).sum::<f64>()
            / recent.len() as f64
    }

    /// p99 deliberate-phase latency over the last `n` ticks.
    pub fn p99_deliberate_ms(&self, n: usize) -> u16 {
        let mut latencies: Vec<u16> = self.recent(n)
            .iter()
            .map(|m| m.phase_latencies.deliberate_ms)
            .collect();
        latencies.sort_unstable();
        let idx = ((latencies.len() as f64 * 0.99) as usize)
            .min(latencies.len().saturating_sub(1));
        latencies.get(idx).copied().unwrap_or(0)
    }
}
```

### Part 2: TraceContext (source implementation)

```rust
// crates/golem-runtime/src/trace_context.rs

use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceId(pub [u8; 16]);

impl TraceId {
    pub fn new() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        Instant::now().hash(&mut h);
        let v = h.finish();
        let mut b = [0u8; 16];
        b[..8].copy_from_slice(&v.to_le_bytes());
        b[8..].copy_from_slice(&(!v).to_le_bytes());
        TraceId(b)
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanId(pub u64);

impl SpanId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(1);
        SpanId(CTR.fetch_add(1, Ordering::Relaxed))
    }

    pub fn to_hex(&self) -> String { format!("{:016x}", self.0) }
}

#[derive(Debug, Clone)]
pub enum SpanName {
    Tick(u64),
    Inference { model: &'static str },
    ToolExecution { tool: &'static str },
    GrimoireQuery,
    StyxWrite,
    Custom(&'static str),
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self { AttributeValue::Str(s.to_owned()) }
}
impl From<u64> for AttributeValue {
    fn from(n: u64) -> Self { AttributeValue::Int(n as i64) }
}
impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self { AttributeValue::Float(f) }
}

#[derive(Debug, Clone)]
pub struct SpanRecord {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub name: SpanName,
    pub duration_ns: u64,
    pub attributes: Vec<(&'static str, AttributeValue)>,
}

pub struct Span {
    pub trace_id: TraceId,
    pub id: SpanId,
    parent_id: Option<SpanId>,
    name: SpanName,
    start: Instant,
    attributes: Vec<(&'static str, AttributeValue)>,
}

impl Span {
    pub fn start(
        trace_id: TraceId,
        parent_id: Option<SpanId>,
        name: SpanName,
    ) -> Self {
        Span {
            trace_id,
            id: SpanId::new(),
            parent_id,
            name,
            start: Instant::now(),
            attributes: Vec::new(),
        }
    }

    pub fn set_attribute(
        &mut self, key: &'static str, value: impl Into<AttributeValue>,
    ) {
        self.attributes.push((key, value.into()));
    }

    pub fn finish(self) -> SpanRecord {
        SpanRecord {
            trace_id: self.trace_id,
            span_id: self.id,
            parent_span_id: self.parent_id,
            name: self.name,
            duration_ns: self.start.elapsed().as_nanos() as u64,
            attributes: self.attributes,
        }
    }
}

pub trait SpanExporter: Send + Sync {
    fn export(&self, spans: Vec<SpanRecord>);
}

/// Per-tick distributed trace root. One per theta tick.
pub struct TraceContext {
    pub trace_id: TraceId,
    root_span: Span,
    completed_spans: Vec<SpanRecord>,
    exporter: Option<Arc<dyn SpanExporter>>,
}

impl TraceContext {
    pub fn start(
        tick: u64, exporter: Option<Arc<dyn SpanExporter>>,
    ) -> Self {
        let trace_id = TraceId::new();
        let root_span = Span::start(
            trace_id, None, SpanName::Tick(tick),
        );
        TraceContext {
            trace_id, root_span,
            completed_spans: Vec::new(), exporter,
        }
    }

    /// Create a child span for an outbound call.
    pub fn child_span(&self, name: SpanName) -> ChildSpan {
        let span = Span::start(
            self.trace_id, Some(self.root_span.id), name,
        );
        let traceparent = format!(
            "00-{}-{}-01",
            self.trace_id.to_hex(),
            span.id.to_hex(),
        );
        ChildSpan { traceparent, span }
    }

    pub fn finish_child(&mut self, child: ChildSpan) {
        self.completed_spans.push(child.span.finish());
    }

    /// Finalize and export. Zero-cost when exporter is None.
    pub fn finish(mut self) {
        let root_record = self.root_span.finish();
        if let Some(exporter) = &self.exporter {
            let mut all_spans = self.completed_spans;
            all_spans.push(root_record);
            exporter.export(all_spans);
        }
    }
}

pub struct ChildSpan {
    /// W3C traceparent header for injection into outbound HTTP requests.
    pub traceparent: String,
    span: Span,
}

impl ChildSpan {
    pub fn set_attr(
        &mut self, key: &'static str, value: impl Into<AttributeValue>,
    ) {
        self.span.set_attribute(key, value.into());
    }
}

/// Inject traceparent into an HTTP header map.
pub fn inject_trace_header(
    child: &ChildSpan, headers: &mut http::HeaderMap,
) {
    if let Ok(value) = child.traceparent.parse() {
        headers.insert("traceparent", value);
    }
}
```

### Investigation composition flow

`MetricsEmitter` produces one record per tick with aggregate numbers. `TraceContext` produces one tree of spans per tick with per-call breakdowns. A typical investigation:

1. MetricsEmitter's rolling cost shows a spike at ticks 4800-4850.
2. Pull `TickMetrics` for that range: `deliberate_ms` is the outlier.
3. Load the TraceContext export for tick 4812: the inference call to Opus took 3200ms (normally 800ms).
4. The child span's attributes show the model was overloaded (queue depth attribute from the provider).

MetricsEmitter tells you *where* to look. TraceContext tells you *why*.

---

## References

### State snapshots
- [MERKLE-1987] Merkle, R. C. (1987). "A digital signature based on a conventional encryption function." In *Advances in Cryptology -- CRYPTO '87*, LNCS 293. Springer. — *Introduces Merkle trees for tamper-evident data structures; used here as the integrity backbone for Golem state snapshots.*
- [FOWLER-2005] Fowler, M. (2005). "Event Sourcing." martinfowler.com. — *Defines the event-sourcing pattern where state is reconstructed from an append-only log of changes; the conceptual model for Golem state persistence.*
- [KLEPPMANN-2017] Kleppmann, M. (2017). *Designing Data-Intensive Applications*. O'Reilly. — *Comprehensive treatment of replication, consistency, and storage engine design; informs the durability guarantees for Golem snapshots.*

### Context delta compression
- [RICHARDSON-2002] Richardson, I. E. G. (2002). *Video Codec Design*. Wiley. (ITU-T H.261, 1990.) — *Explains I-frame/P-frame delta coding in video compression; the same keyframe + delta pattern is adapted for Golem context snapshots.*
- [MYERS-1986] Myers, E. W. (1986). "An O(ND) difference algorithm and its variations." *Algorithmica*, 1(1), 251-266. — *The foundational shortest-edit-script algorithm (used in `diff`); underlies the delta computation between context states.*
- [CAI-2014] Cai, Y., et al. (2014). "A theory of changes for higher-order languages." *ACM PLDI*, 145-155. — *Formalizes incremental computation via self-maintainable derivatives; theoretical grounding for computing context deltas efficiently.*
- [MU-2024] Mu, Y., et al. (2024). "GIST: Efficient inference of long-context LLMs via attention compression." arXiv:2402.14827. — *Demonstrates attention-based token compression for LLMs; informs how Golem context deltas reduce prompt size without losing signal.*

### Episodic replay
- [TULVING-1972] Tulving, E. (1972). "Episodic and semantic memory." In *Organization of Memory*. Academic Press. — *Distinguishes episodic from semantic memory; the foundational taxonomy behind Grimoire's separation of episodes from insights.*
- [SCHANK-1982] Schank, R. C. (1982). *Dynamic Memory*. Cambridge University Press. — *Introduces scripts, plans, and dynamic memory organization for story understanding; inspires the Grimoire's episode schema and retrieval structure.*
- [WILSON-1994] Wilson, M. A., & McNaughton, B. L. (1994). "Reactivation of hippocampal ensemble memories during sleep." *Science*, 265(5172), 676-679. — *Shows that hippocampal neurons replay waking experiences during sleep; biological basis for the Golem's sleep-phase memory consolidation.*
- [LEWIS-2020] Lewis, P., et al. (2020). "Retrieval-augmented generation for knowledge-intensive NLP tasks." *NeurIPS*, 33. — *Introduces RAG, combining a retrieval index with a generator; the pattern Golem uses to inject Grimoire episodes into LLM context.*
- [AAMODT-1994] Aamodt, A. & Plaza, E. (1994). "Case-Based Reasoning: Foundational Issues." *AI Communications*, 7(1), 39-59. — *Formalizes case-based reasoning (retrieve, reuse, revise, retain); maps directly to how Golem replays past episodes to inform current decisions.*

### Metrics and tracing
- [BEYER-2016] Beyer, B., Jones, C., Petoff, J., & Murphy, N. R. (2016). *Site Reliability Engineering*. O'Reilly. — *Google's SRE bible defining SLIs, SLOs, and error budgets; the operational framework adapted for Golem health monitoring.*
- [VOLZ-2015] Volz, B., et al. (2015). "Prometheus: Monitoring at SoundCloud." *USENIX LISA*. — *Describes the Prometheus pull-based metrics architecture; Golem's internal metrics emitter follows a similar dimensional model.*
- [MAJORS-2022] Majors, C., Fong-Jones, L., & Miranda, G. (2022). *Observability Engineering*. O'Reilly. — *Argues for high-cardinality, wide-event observability over traditional metrics; informs Golem's trace-centric approach to runtime visibility.*
- [DAPPER-2010] Sigelman, B., et al. (2010). "Dapper, a large-scale distributed systems tracing infrastructure." Google Technical Report. — *Pioneering distributed tracing system introducing trace/span models; the conceptual ancestor of Golem's TraceContext implementation.*
- [W3C-TRACE-2021] W3C Trace Context Specification, Level 1 (2021). — *Standardizes trace propagation headers (traceparent, tracestate); Golem's trace IDs follow this format for interoperability.*
- [OTEL] OpenTelemetry Specification. CNCF, 2019-present. — *Vendor-neutral observability standard unifying traces, metrics, and logs; the target export format for Golem telemetry.*
- [MATTAR-2018] Mattar, M.G. & Daw, N.D. "Prioritized Memory Access Explains Planning and Hippocampal Replay." *Nature Neuroscience*, 21, 2018. — *Shows that replay priority correlates with planning utility, not recency; justifies Golem's relevance-weighted episode replay over FIFO ordering.*

---
