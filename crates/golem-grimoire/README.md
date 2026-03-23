# golem-grimoire

Persistent knowledge store for Bardo Golems. Two substrates: a LanceDB vector store for raw episodic observations, and a SQLite relational store for extracted semantic knowledge. Everything written to the grimoire passes through an admission gate. Everything retrieved is scored by four factors simultaneously.

## Architecture

**Episodic substrate** (`substrate/episodic.rs`, LanceDB): immutable execution observations. Each `Episode` has a 768-dim embedding (nomic-embed-text-v1.5, Matryoshka), a UUIDv7 for time-ordered indexing, PAD affect components, importance score, and outcome label. Once written, episodes do not change.

**Semantic substrate** (`substrate/semantic.rs`, SQLite): extracted patterns stored as `GrimoireEntry`. Six categories:

| Category         | Description                                       | Confidence floor |
|------------------|---------------------------------------------------|------------------|
| `Insight`        | Declarative: "what is true"                       | 0.05             |
| `Heuristic`      | Prescriptive: "what to do"                        | 0.05             |
| `Warning`        | Negative pattern, immune system function          | 0.05             |
| `CausalLink`     | Structural dependency                             | 0.05             |
| `StrategyFragment` | Speculative approach                            | 0.05             |
| `AntiKnowledge`  | Permanent negative knowledge, never decays below 0.30 | 0.30        |

**Playbook** (`substrate/playbook.rs`): validated behavioral rules promoted from semantic patterns after five or more successful applications.

## GrimoireEntry

The full entry struct carries more than just content. Key fields:

```rust
pub struct GrimoireEntry {
    pub id: Uuid,                      // UUIDv7, time-ordered
    pub golem_id: GolemId,
    pub category: EntryType,           // immutable after creation
    pub content: String,
    pub embedding: Vec<f32>,           // 768-dim
    pub confidence: f64,               // [floor, 1.0]
    pub quality_score: f64,
    pub decay_class: DecayClass,       // immutable after creation
    pub provenance: Provenance,
    pub emotional_tag: Option<EmotionalTag>,
    pub is_bloodstain: bool,           // death-sourced: 1.2x retrieval, 3x slower decay
    pub polarity: KnowledgePolarity,   // K+ (positive) or K- (negative)
    pub memetic: MemeticFields,        // fidelity, fecundity, fitness, parasite_score
    // ... validation tracking, timestamps, causal DAG edges, tags
}
```

`DecayClass` governs temporal half-life: `Ephemeral` (~48h), `Tactical` (~7d), `RegimeConditional` (~14-21d), `Structural` (no decay), `Procedural` (curator-managed).

## Admission Gate (A-MAC)

Every candidate entry is scored before being written. The composite score is:

```
score = 0.25 × future_utility
      + 0.25 × factual_confidence
      + 0.20 × semantic_novelty
      + 0.15 × temporal_recency
      + 0.15 × content_type_prior
```

Content type priors: `Warning` and `AntiKnowledge` get 0.9, `CausalLink` 0.7, `Heuristic` 0.6, `Insight` 0.5, `StrategyFragment` 0.4. Warnings get in easily; speculative fragments have to work for it.

```rust
let score = AdmissionGate::composite_score(
    EntryType::Warning,
    future_utility,
    factual_confidence,
    semantic_novelty,
    temporal_recency,
);
let result = AdmissionGate::decide(score);
// Rejected (< 0.45), AdmittedConservative (0.45–0.55, confidence 0.3),
// or AdmittedStandard (> 0.55, confidence 0.6)
```

Hallucination firewall: entries with `factual_confidence < 0.3` that contradict existing high-confidence entries go to `EntryStatus::Quarantined` rather than being written.

## Retrieval

Four factors, multiplied:

```
retrieval_score = recency × importance × relevance × congruence
```

- **Recency**: exponential decay from `last_accessed_at` based on `DecayClass` half-life. Bloodstain entries decay at 3× the normal rate.
- **Importance**: `confidence × quality_score`, with a 1.2× boost (capped at 1.0) for bloodstain entries.
- **Relevance**: cosine similarity between query embedding and `entry.embedding`, clamped to `[-1.0, 1.0]`, then floored at 0.0 before multiplication.
- **Emotional congruence**: dot product of normalized query PAD and entry PAD, mapped to `[0.0, 1.0]` via `0.5 + 0.5 × dot`. Entries without an `EmotionalTag` get 0.5 (neutral).

```rust
let scored: ScoredEntry = score_entry(&entry, &query_embedding, &query_pad, current_tick);
// scored.retrieval_score, .recency_component, .relevance_component,
// .importance_component, .congruence_component
```

## Memetic Tracking

`MemeticFields` tracks cultural-evolution fitness for each entry:

- `fidelity`: how faithfully the entry has been transmitted (0.0–1.0)
- `fecundity`: how often it is retrieved and acted on (copies/tick)
- `fitness`: `fidelity × fecundity × longevity`
- `parasite_score`: normalized sum of squared prediction errors — high values flag entries that spread without validation

The curator prunes entries when `parasite_score` exceeds threshold. The `memetic` module in `src/memetic.rs` owns the update logic.

## Provenance

Entries track how they were created via `Provenance`:

```rust
pub enum Provenance {
    SelfLearned, Clade, Predecessor, StyxQuery, Lethe,
    Marketplace, Replicant, DeathReflection, Dream,
}
```

`DeathReflection` and `Dream` provenance set `is_bloodstain = true`, which triggers the retrieval boost and slower decay.

## Usage

```toml
[dependencies]
golem-grimoire = { path = "../../crates/golem-grimoire" }
```
