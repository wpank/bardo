# Emotion-Weighted Memory: How the Daimon Shapes What Golems Remember [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-daimon`, `golem-grimoire`
>
> **Depends-on**: `00-overview.md`

---

> **Reader orientation:** This document specifies how the Daimon (the affect engine) integrates with the Grimoire (the agent's persistent knowledge base) to create emotion-weighted memory within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers emotional tagging of episodes, four-factor retrieval scoring with mood-congruent bias, and how emotional provenance flows through knowledge inheritance. Prerequisites: the Daimon overview (`00-overview.md`) and the Grimoire architecture (`../04-memory/`). For a full glossary, see `prd2/shared/glossary.md`.

## 1. Why Emotional Memory Matters

> **Extended:** Neural correlate grounding (amygdala-hippocampal interactions, McGaugh 2004), neurobiological pathway detail -- see [prd2-extended/03-daimon/02-emotion-memory-extended.md](../../prd2-extended/03-daimon/02-emotion-memory-extended.md).

Emotional experiences receive preferential encoding, consolidation, and retrieval. High-arousal events are remembered more vividly and retained longer than neutral events. For Golems, a flash crash that cost 15% should be more salient than a routine +0.1% tick. The Grimoire's existing three-factor retrieval score (`recency x importance x relevance`) does not capture affective salience. Emotional tagging adds a fourth dimension, producing the four-factor retrieval model: (1) semantic similarity, (2) temporal recency, (3) importance/quality_score, (4) emotional congruence via PAD cosine similarity. See Section 3 for the full scoring function.

Empirical validation: Emotional RAG (2024, arXiv:2410.23041) demonstrated that emotion-weighted retrieval significantly outperformed purely semantic retrieval across ChatGLM, Qwen, and GPT-3.5 [EMOTIONAL-RAG-2024]. Theoretical foundation: Bower's (1981) associative network theory, with 5--30% accuracy boost for congruent versus incongruent retrieval [BOWER-1981], [FAUL-LABAR-2022].

---

## 2. Emotional Tags on Episodes

Every Episode in the Grimoire gains an optional emotional tag. This tag is populated by the appraisal engine (see `01-appraisal.md`) whenever an emotion is generated in the same tick as the Episode. Episodes created before the daimon extension is enabled, or during T0 ticks that skip appraisal, have `emotional_tag: None` and are retrieved normally -- the emotional component defaults to a neutral factor in the retrieval score.

### 2.1 EmotionalTag Struct

```rust
/// Emotional context attached to a Grimoire entry at creation time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTag {
    /// PAD vector at the time the Episode was created.
    /// This is the discrete emotion that the appraisal engine
    /// generated for the event that produced this Episode.
    pub pad: PADVector,

    /// Plutchik label for LLM-compatible annotation.
    /// Stored as the combined string, e.g., "moderate_fear", "mild_joy".
    pub emotion: String,

    /// Emotional intensity at encoding time [0.0, 1.0].
    /// Higher intensity means the Episode was encoded under
    /// stronger emotional conditions and should receive
    /// correspondingly higher consolidation priority.
    pub intensity: f32,

    /// Brief description of what triggered the appraisal.
    /// Used for narrative reconstruction during life review
    /// and for debugging the daimon system.
    pub trigger: String,

    /// Mood snapshot at the time of Episode creation.
    /// Captures the broader affective context beyond the
    /// discrete emotion. Used for mood-congruent retrieval:
    /// the mood at encoding is compared with the mood at
    /// retrieval to compute congruence.
    pub mood_snapshot: PADVector,
}
```

### 2.2 Extension to GrimoireEntry

The existing `GrimoireEntry` struct gains the optional emotional tag:

```rust
// Extension to existing GrimoireEntry struct
pub struct GrimoireEntry {
    // ... existing fields (id, content, embedding, importance,
    //     novelty, tick_number, decay_half_life, etc.) ...

    /// Emotional context at creation time.
    /// None for entries created before the daimon was enabled
    /// or during ticks without appraisal events.
    pub emotional_tag: Option<EmotionalTag>,
}
```

The optional nature of the field ensures backward compatibility. The Grimoire operates identically with or without emotional tags -- the daimon system is strictly additive. This follows Design Principle 3.4 (Opt-In, Not Default) from `00-overview.md`.

### 2.3 LanceDB Schema Extension

Episodes stored in LanceDB (the vector database for episodic memory) gain additional columns for the emotional tag fields. The affect fields are stored as separate columns rather than a nested object because LanceDB's columnar format makes individual field filtering and sorting efficient. Querying for "all episodes where arousal > 0.5" is a column scan, not a parse.

---

## 3. Four-Factor Retrieval Scoring

Every knowledge retrieval scores candidates using four factors, each grounded in a different research tradition.

### 3.1 The Four Factors

**Factor 1: Recency** (Ebbinghaus, 1885). Memories fade with time. The forgetting curve is exponential: `recency = exp(-t / half_life)`, where `t` is ticks since last access. Recently accessed entries score higher. This implements Ebbinghaus's finding that memory retention follows a negative exponential decay [EBBINGHAUS-1885].

**Factor 2: Importance** (Reflexion, Shinn et al. 2023). Entries that have been validated through operational use (predicted an outcome that came true) earn higher confidence. The `quality_score` computed column combines raw confidence with the validation ratio: `confidence * (validated / (validated + contradicted + 1))`. This implements Reflexion's core idea: self-reflection on past performance improves future decisions [SHINN-2023].

**Factor 3: Relevance** (standard RAG). Cosine similarity between the query embedding and the entry embedding. This is the baseline retrieval signal -- how semantically close is this entry to what the Golem is currently thinking about?

**Factor 4: Emotional congruence** (Bower, 1981). The PAD cosine similarity between the Golem's current emotional state and the entry's affect provenance (the emotion the Golem was feeling when it created the entry). An anxious Golem retrieves entries created during anxious moments; a confident Golem retrieves entries created during confident moments. Bower's (1981) associative network theory explains why: emotional states activate linked memories, creating feedback loops between affect and cognition [BOWER-1981]. Emotional RAG (2024) validated this computationally: LLM agents with emotion-tagged retrieval significantly outperformed non-emotional retrieval across three datasets using ChatGLM, Qwen, and GPT-3.5 [EMOTIONAL-RAG-2024].

### 3.2 Implementation

```rust
/// Four-factor retrieval scoring.
///
/// score = w_recency    * recency(Ebbinghaus)
///       + w_importance * quality(Reflexion)
///       + w_relevance  * cosine(query, entry)
///       + w_emotional  * PAD_cosine(current_mood, entry_affect)
///
/// Weights are LEARNED by the cybernetics self-tuning system (Loop 1,
/// see 08-cognition.md). The initial weights are:
///   recency: 0.20, importance: 0.25, relevance: 0.35, emotional: 0.20
///
/// After several hundred ticks, Loop 1 adjusts these based on which
/// factor combinations correlate with positive outcomes.
pub fn score_entry(
    entry: &GrimoireEntry,
    query_embedding: &[f32],
    current_pad: &PADVector,
    current_tick: u64,
    weights: &RetrievalWeights,
) -> f64 {
    // Factor 1: Recency (Ebbinghaus forgetting curve)
    let ticks_since_access = (current_tick - entry.last_accessed_at) as f64;
    let recency = (-ticks_since_access / weights.recency_half_life).exp();

    // Factor 2: Importance (confidence x validation ratio)
    let importance = entry.quality_score();

    // Factor 3: Relevance (cosine similarity of embeddings)
    let relevance = cosine_similarity(query_embedding, &entry.embedding);

    // Factor 4: Emotional congruence (PAD cosine similarity)
    let entry_pad = PADVector {
        pleasure: entry.affect_pleasure,
        arousal: entry.affect_arousal,
        dominance: entry.affect_dominance,
    };
    let emotional_congruence = pad_cosine_similarity(current_pad, &entry_pad);

    // Weighted sum
    weights.recency * recency
        + weights.importance * importance
        + weights.relevance * relevance
        + weights.emotional * emotional_congruence
}
```

### 3.3 PAD Similarity Function

```rust
/// PAD cosine similarity, mapped to [0, 1].
/// Cosine similarity captures the _direction_ of emotional state
/// (the quality of emotion) rather than its _magnitude_ (the intensity).
/// Two anxiety states at different intensities are recognized as similar.
pub fn pad_cosine_similarity(a: &PADVector, b: &PADVector) -> f64 {
    let dot = a.pleasure as f64 * b.pleasure as f64
        + a.arousal as f64 * b.arousal as f64
        + a.dominance as f64 * b.dominance as f64;
    let mag_a = ((a.pleasure as f64).powi(2)
        + (a.arousal as f64).powi(2)
        + (a.dominance as f64).powi(2))
    .sqrt();
    let mag_b = ((b.pleasure as f64).powi(2)
        + (b.arousal as f64).powi(2)
        + (b.dominance as f64).powi(2))
    .sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.5; // Neutral mood
    }
    (dot / (mag_a * mag_b) + 1.0) / 2.0 // Map [-1, 1] --> [0, 1]
}
```

---

## 4. Mood-Congruent Memory: Bower's Theory Operationalized

### 4.1 The Theory

> **Extended:** Bower's associative network theory full detail, Faul and LaBar (2022) evidence review -- see [prd2-extended/03-daimon/02-emotion-memory-extended.md](../../prd2-extended/03-daimon/02-emotion-memory-extended.md).

Mood acts as a retrieval cue: when a Golem is anxious, memories encoded during previous anxiety states become more accessible -- not because they are semantically similar, but because they share an emotional signature. Mood-congruent recall boosts accuracy by 5--30% [FAUL-LABAR-2022], is strongest for high-arousal autobiographical memories, and is bidirectional (positive mood retrieves positive memories, negative retrieves negative).

### 4.2 The Full Retrieval Pipeline

```rust
/// Retrieve relevant knowledge from the Grimoire.
///
/// Five phases:
/// 1. Candidate generation: HNSW approximate nearest neighbors (LanceDB)
/// 2. Four-factor re-ranking: score each candidate
/// 3. Contrarian injection: 15% min across rolling 200-tick window
/// 4. Bloodstain boost: 1.2x for death-sourced entries
/// 5. Testing effect: mark retrieved entries as accessed (strengthens memory trace)
pub fn retrieve(
    grimoire: &Grimoire,
    query: &str,
    current_pad: &PADVector,
    current_tick: u64,
    weights: &RetrievalWeights,
    limit: usize,
) -> Result<Vec<ScoredEntry>> {
    let query_embedding = grimoire.embedder.embed(query)?;

    // Phase 1: Candidate generation
    // Over-fetch 3x for re-ranking headroom
    let candidates = grimoire.episodes.search(&query_embedding)
        .limit(limit * 3)
        .execute()?;

    // Phase 2: Four-factor re-ranking
    let mut scored: Vec<ScoredEntry> = candidates.iter()
        .map(|entry| ScoredEntry {
            score: score_entry(entry, &query_embedding, current_pad, current_tick, weights),
            entry: entry.clone(),
        })
        .collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Phase 3: Contrarian injection (rolling window schedule)
    // Prevents depressive rumination by forcing mood-opposite entries.
    // Without it, an anxious Golem would retrieve anxiety-tagged entries,
    // which would increase anxiety, which would retrieve more anxiety-tagged
    // entries -- a positive feedback loop that Nietzsche warned against:
    // "there exists a degree of rumination which is harmful and ultimately
    // fatal to the living thing" [NIETZSCHE-1887].
    //
    // The schedule maintains a minimum 15% contrarian retrieval rate
    // across any rolling 200-tick window, rather than firing on a
    // fixed tick modulus.
    if contrarian_tracker.should_inject(current_tick) {
        let inverted_pad = PADVector {
            pleasure: -current_pad.pleasure,
            arousal: current_pad.arousal,    // Keep arousal (salience still matters)
            dominance: -current_pad.dominance,
        };
        let contrarian = grimoire.episodes.search(&query_embedding)
            .limit(3)
            .filter(format!("affect_pleasure * {} < 0", current_pad.pleasure.signum()))
            .execute()?;
        for entry in contrarian {
            scored.push(ScoredEntry {
                score: score_entry(&entry, &query_embedding, &inverted_pad, current_tick, weights),
                entry,
            });
        }
        contrarian_tracker.record(current_tick);
    }

    // Phase 4: Bloodstain boost
    // Death-sourced entries receive a 1.2x retrieval score boost.
    // A dead Golem cannot benefit from its own warning, making the
    // information maximally honest. Death-sourced entries also decay
    // 3x slower than normal entries.
    for scored_entry in &mut scored {
        if scored_entry.entry.is_bloodstain {
            scored_entry.score *= 1.2;
        }
    }

    // Phase 5: Testing effect -- mark as accessed
    // Roediger & Karpicke (2006): retrieval strengthens the memory trace.
    // Entries that are never retrieved decay to zero and get pruned.
    for scored_entry in scored.iter().take(limit) {
        grimoire.semantic.execute(
            "UPDATE grimoire_entries SET last_accessed_at = ?, access_count = access_count + 1
             WHERE id = ?",
            rusqlite::params![current_tick, scored_entry.entry.id],
        )?;
    }

    Ok(scored.into_iter().take(limit).collect())
}
```

### 4.3 Cross-Emotional Retrieval

The system also supports _cross-emotional_ retrieval -- deliberately searching for memories encoded under a different emotional state. Useful when anxious (retrieve coping memories), overconfident (retrieve past failures), or stuck (inject emotional diversity). Available as an explicit retrieval mode when the default produces tunnel vision.

### 4.4 Contrarian Retrieval Schedule

To prevent depressive rumination, the system enforces periodic contrarian retrieval: minimum 15% contrarian retrieval is maintained across any rolling 200-tick window. The schedule is adaptive, not fixed-interval.

#### Retrieval Strengthening (Testing Effect)

Retrieved emotional memories with positive outcomes increment `strength`, making them decay slower. Live retrieval: +1 on positive outcome. Dream-retrieval: +0.5 on validated pattern. No increment on mere retrieval.

```
retention(t) = e^(-(t - lastAccessed) / (halfLife * strength))
effective_confidence(t) = confidence * retention(t)
```

---

## 5. Emotional Consolidation Bias

### 5.1 Neuroscience and Implementation

> **Extended:** Neuroscience of emotional consolidation (McGaugh 2004, Born & Wilhelm 2012), somatic marker consolidation mechanism -- see [prd2-extended/03-daimon/02-emotion-memory-extended.md](../../prd2-extended/03-daimon/02-emotion-memory-extended.md).

When the daimon is enabled, the Curator applies an emotional salience boost during consolidation:

```rust
pub fn consolidation_priority(episode: &Episode) -> f64 {
    let base_priority = episode.importance * episode.novelty;

    let emotional_tag = match &episode.emotional_tag {
        Some(tag) => tag,
        None => return base_priority,
    };

    // Emotional arousal increases consolidation priority.
    // High-arousal experiences (fear, surprise, anger, joy)
    // are preferentially consolidated, matching the
    // neurobiological pattern (McGaugh, 2004).
    let arousal_boost = emotional_tag.pad.arousal.abs() as f64 * 0.3;
    base_priority * (1.0 + arousal_boost)
}
```

The 0.3 scaling factor ensures emotional salience is _additive_ to the existing importance-novelty ranking, not dominant over it. Among Episodes of comparable importance and novelty, emotional intensity breaks the tie in favor of the emotionally salient one.

> **Extended:** Flashbulb memory (Brown & Kulik 1977, crisis-encoded knowledge, 4 triggers, rate limits), emotional modulation of learning (high-arousal reflexion bias, low-dominance learned helplessness, rumination detection, Curator distribution checks), Weismann barrier for emotional tags (inheritance constraints, Baldwin Effect) -- see [prd2-extended/03-daimon/02-emotion-memory-extended.md](../../prd2-extended/03-daimon/02-emotion-memory-extended.md).

---

## 6. Emotional Provenance Tracking

### 6.1 From Episodes to Insights: Emotional Context Transfers

When the Curator consolidates Episodes into Insights and Heuristics, the emotional provenance transfers. An Insight is not just a distilled observation; it carries the emotional context of its discovery and validation:

1. **Retrieval enrichment**: Insights with emotional provenance participate in mood-congruent retrieval.
2. **Confidence signaling**: Insights validated across diverse emotional states are more reliable.
3. **Knowledge transfer**: When the Golem dies and its Grimoire is inherited, emotional provenance carries contextual information that bare confidence scores cannot.

### 6.2 EmotionalProvenance Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalProvenance {
    /// Average PAD vector during evidence accumulation.
    /// Computed as the mean of all supporting Episodes'
    /// emotional tags.
    pub average_pad: PADVector,

    /// Emotion at first discovery.
    pub discovery_emotion: EmotionLabel,

    /// Narrative arc of validation.
    /// Follows McAdams' (2001) narrative identity categories.
    pub validation_arc: Option<ValidationArc>,

    /// Emotional diversity: how many different emotional
    /// states contributed evidence. Higher = more reliable.
    /// Computed as normalized Shannon entropy.
    pub emotional_diversity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationArc {
    /// Adversity to positive outcome -- most transferable knowledge.
    Redemptive,
    /// Initial success to failure -- cautionary.
    Contaminating,
    /// Consistent tone -- reliable but less narratively rich.
    Stable,
    /// Gradual improvement -- successful learning trajectory.
    Progressive,
}
```

### 6.3 Emotional Diversity as Quality Signal

An Insight validated across diverse emotional states is more reliable than one validated only during a single mood. Emotional diversity is computed as the normalized Shannon entropy of emotional labels across supporting Episodes:

```rust
pub fn emotional_diversity(supporting_episodes: &[Episode]) -> f64 {
    let mut emotion_counts: HashMap<String, u32> = HashMap::new();
    for ep in supporting_episodes {
        if let Some(ref tag) = ep.emotional_tag {
            *emotion_counts.entry(tag.emotion.clone()).or_insert(0) += 1;
        }
    }

    let total: u32 = emotion_counts.values().sum();
    if total == 0 { return 0.0; }

    let mut entropy = 0.0_f64;
    for &count in emotion_counts.values() {
        let p = count as f64 / total as f64;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }

    let max_entropy = (emotion_counts.len() as f64).log2();
    if max_entropy > 0.0 { entropy / max_entropy } else { 0.0 }
}
```

An `emotional_diversity` of 1.0 means every supporting Episode was tagged with a different emotion (maximum diversity). A value of 0.0 means all supporting Episodes had the same emotional tag. The Curator uses this score as a supplementary quality signal: among Insights of comparable confidence, higher emotional diversity suggests greater robustness.

---

## 7. Mind Wandering

> **Extended:** Mind wandering mechanism (default mode network analog, spontaneous retrieval every ~200 ticks, re-appraisal logic, emotional recalibration, variability injection, narrative continuity) -- see [prd2-extended/03-daimon/02-emotion-memory-extended.md](../../prd2-extended/03-daimon/02-emotion-memory-extended.md).

Approximately every 200 ticks, the system retrieves a random high-arousal Episode and re-appraises it in the current context. This prevents emotional degeneration and injects variability. Cost: zero (local query + deterministic appraisal).

---

## 8. Styx Integration

The emotion-weighted memory system integrates with Styx's three privacy layers. These integrations are active only when the respective layers are enabled by the owner (all are opt-in).

### 8.1 Styx Archive: Playbook Mood Summary

When the Styx Archive layer is enabled, PLAYBOOK.md uploads gain a compact mood summary. The Vault stores encrypted snapshots of the Golem's knowledge, including emotional context:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookWithMood {
    /// The PLAYBOOK.md text content.
    pub content: String,

    /// Compact mood summary appended to each upload.
    pub mood_summary: MoodSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodSummary {
    /// Average mood over this Curator cycle.
    pub average_mood: PADVector,
    /// Top 3 emotional events since last upload.
    pub emotional_highlights: Vec<String>,
    /// Mood trajectory classification.
    pub mood_trajectory: MoodTrajectory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoodTrajectory {
    Improving,
    Stable,
    Declining,
}
```

The mood summary serves two purposes:

1. **Successor boot context**: When a new Golem boots from a predecessor's Vault backup, the mood summaries attached to PLAYBOOK.md snapshots provide emotional context for the inherited knowledge.
2. **Trajectory analysis**: The sequence of mood summaries across multiple uploads creates a mood trajectory that can be analyzed after the Golem's death.

### 8.2 Styx Lethe (formerly Commons): Emotional Retrieval Parameter

When the Styx Lethe layer is enabled, queries include the current mood as metadata (`mood: { p, a, d }`). The Lethe indexes include affect fields, applying emotional congruence as a secondary ranking factor (15% weight). This enables fleet-wide mood-congruent recall: an anxious Golem retrieves peers' fear memories from similar conditions, regardless of asset or strategy differences.

### 8.3 Death Testament: Emotional Life Review

The Death Protocol's Phase II gains an Emotional Life Review: peak experiences, turning points, narrative arc classification, and unresolved tensions. See `05-death-daimon.md` for the full `EmotionalLifeReview` struct and the `EmotionalDeathTestament` type.

### 8.4 Hot Cognition Transfer

Emotionally annotated knowledge is hot cognition [ABELSON-1963]. A successor receiving an insight annotated with emotional provenance gains context that bare confidence scores cannot provide: _when_ the insight is most relevant (conditions matching the emotional state at discovery), _how_ it was validated (emotional diversity of confirming episodes), and _how strongly_ to weight it (redemptive arcs indicate hard-won knowledge with high transfer value).

---

## 9. Data Model

### 9.1 SQLite Schema (Grimoire Semantic Store)

The emotion log and grimoire entry schemas are defined in `03a-grimoire-storage.md`. The Daimon writes to the `emotion_log` table every tick and reads from it for windowed aggregate queries (learned helplessness detection, emotional load computation).

```sql
-- Emotion log (append-only, written by the Daimon every tick)
-- See 03a-grimoire-storage.md for the canonical schema.
CREATE TABLE emotion_log (
    tick INTEGER PRIMARY KEY,
    pleasure REAL NOT NULL,
    arousal REAL NOT NULL,
    dominance REAL NOT NULL,
    primary_emotion TEXT NOT NULL,
    regime TEXT,
    phase TEXT
);

CREATE INDEX idx_emotion_arousal ON emotion_log(arousal);
CREATE INDEX idx_emotion_tick    ON emotion_log(tick DESC);
```

### 9.2 Grimoire Entry Affect Fields

The `grimoire_entries` table (see `03a-grimoire-storage.md`) includes affect provenance columns:

- `affect_pleasure`, `affect_arousal`, `affect_dominance` -- PAD at discovery
- `discovery_emotion` -- Plutchik label at time of creation
- These columns enable the four-factor retrieval scoring described in Section 3.

---

## 9b. Dream-Memory-Emotion Triangle

> **Extended:** Three-way interaction between emotional tagging, offline consolidation, and memory reorganization (SFSR depotentiation, mood-congruent dream generation, dream consolidation feedback) -- see [prd2-extended/03-daimon/02-emotion-memory-extended.md](../../prd2-extended/03-daimon/02-emotion-memory-extended.md).

---

## 10. Cross-References

- `00-overview.md` -- Why agents need emotions; design principles
- `01-appraisal.md` -- Appraisal engine, OCC, Scherer, Chain-of-Emotion
- `../04-memory/` -- Grimoire architecture, Styx Archive, Styx Lethe, knowledge ecology
- `../02-mortality/` -- Death protocol, life review, succession, emotional arc
- `../05-dreams/` -- NREM replay, REM depotentiation, dream-daimon interaction
- `prd2/09-economy/02-clade.md` -- Sibling knowledge sharing

---

## 11. References

- [ABELSON-1963] Abelson, R.P. "Computer Simulation of 'Hot Cognition'." In Tomkins, S. & Messick, S. (Eds.), _Computer Simulation of Personality_. Wiley, 1963. Introduces the concept of "hot cognition" where emotionally charged beliefs resist revision more strongly than neutral ones; the theoretical basis for why emotionally tagged Grimoire entries have different retrieval and decay dynamics.
- [BORN-WILHELM-2012] Born, J. & Wilhelm, I. "System Consolidation of Memory During Sleep." _Psychological Research_, 76, 2012. Shows that sleep consolidation is selective, preferentially preserving memories with relevance for future plans; supports the Daimon's consolidation bias toward high-emotional-salience entries.
- [BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981. Establishes mood-congruent memory: emotional states bias which memories are retrieved; the direct theoretical basis for the emotional congruence factor in the four-factor retrieval scoring model.
- [EBBINGHAUS-1885] Ebbinghaus, H. _Memory: A Contribution to Experimental Psychology._ 1885. Establishes the forgetting curve: memories decay exponentially without rehearsal; the foundation for the Grimoire's time-based demurrage system, which the Daimon's emotional tagging modulates.
- [EMOTIONAL-RAG-2024] "Emotional RAG: Enhancing Role-Playing Agents through Emotional Retrieval." arXiv:2410.23041, 2024. Demonstrates that emotion-weighted retrieval in LLM agents outperforms purely semantic retrieval across multiple models; direct empirical validation for the Daimon's emotional memory integration approach.
- [FAUL-LABAR-2022] Faul, L. & LaBar, K.S. "Mood-Congruent Memory Revisited." _Psychological Review_, 2022. Updates Bower's theory with modern neuroscience evidence confirming that emotional state systematically biases memory retrieval; validates the continued relevance of mood-congruent retrieval for computational systems.
- [MCADAMS-2001] McAdams, D.P. "The Psychology of Life Stories." _Review of General Psychology_, 5(2), 2001. Argues that identity is constituted through emotionally structured narrative; informs the emotional provenance tracking that makes inherited knowledge transfer more effective.
- [MCGAUGH-2004] McGaugh, J.L. "The Amygdala Modulates the Consolidation of Memories of Emotionally Arousing Experiences." _Annual Review of Neuroscience_, 27, 2004. Reviews evidence that emotional arousal enhances memory consolidation via amygdala-hippocampal interaction; the neurobiological basis for prioritizing high-arousal episodes in the Grimoire.
- [NIETZSCHE-1887] Nietzsche, F. _On the Genealogy of Morals._ Second Essay, 1887. Argues that pain creates memory more effectively than pleasure ("man had to be given a memory"); frames the asymmetric encoding where losses produce stronger emotional tags than equivalent gains.
- [SHINN-2023] Shinn, N. et al. "Reflexion: Language Agents with Verbal Reinforcement Learning." _NeurIPS_, 2023. Demonstrates that LLM agents improve through verbal self-reflection on past failures; validates the general mechanism of learning from emotionally tagged past experience, though Reflexion lacks the Daimon's continuous affect state.

---
