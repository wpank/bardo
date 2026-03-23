# Daimon in the Death Protocol: Emotional Life Review and Narrative Identity [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-daimon`, `golem-mortality`
>
> **Depends-on**: `00-overview.md`, `../02-mortality/`

---

> **Reader orientation:** This document specifies the Daimon's (the affect engine's) role during the Thanatopsis (four-phase structured shutdown protocol): producing an emotional life review, narrative arc classification, and the EmotionalDeathTestament that transfers affective knowledge to successor Golems (mortal autonomous agents). Part of the Daimon track within Bardo (the Rust runtime for mortal autonomous DeFi agents). Prerequisites: the Daimon overview (`00-overview.md`) and the Mortality track (`../02-mortality/`). For a full glossary, see `prd2/shared/glossary.md`.

## Why Emotion Transforms the Death Protocol

The Thanatopsis Protocol (`../02-mortality/`) already defines death as the Golem's highest-value knowledge production event. Without the daimon, it is a structured data dump. With the daimon, it becomes a **narrative act of meaning-making** -- producing death testaments that are emotionally rich and more effectively transferable to successor Golems.

> **Extended:** Three converging findings (Butler life review, McAdams narrative identity, Abelson hot cognition), full academic argument for emotionally enriched knowledge transfer -- see [prd2-extended/03-daimon/05-death-daimon-extended.md](../../prd2-extended/03-daimon/05-death-daimon-extended.md).

---

## The Four Phases of the Daimon-Enhanced Death Protocol

The original three-phase Death Protocol (Settle, Reflect, Legacy) is expanded to four phases when the daimon is enabled. Phase 0 (Acceptance) is new. Phase II (Reflect) gains the Emotional Life Review. Phase III (Legacy) gains emotional annotation and narrative arc metadata.

### Phase 0: Acceptance (New)

**Duration**: 1--5 ticks after death trigger.
**Cost**: $0.00 (deterministic, no inference).

When the death trigger fires (from any of the three mortality clocks), the Golem enters an acceptance phase. This is the transition from the lion metamorphosis (fighting death) to the child metamorphosis (accepting death with creative sovereignty).

```rust
/// Captures the Golem's state at the moment it accepts death.
/// This is the emotional anchor for the entire death protocol --
/// the baseline from which the life review operates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceState {
    /// Which mortality clock triggered death.
    pub death_cause: DeathCause,

    /// Tick number when death was triggered.
    pub death_trigger_tick: u64,

    /// Final vitality readings across all three clocks.
    pub final_vitality: VitalityState,

    /// Final mood state (the last mood before acceptance).
    pub final_mood: MoodState,

    /// The emotional appraisal at the moment of acceptance.
    pub acceptance_emotion: AppraisalResult,

    /// Estimated ticks needed for the full death protocol.
    pub estimated_protocol_duration: u64,

    /// USDC available for Phases I--III (the Thanatos Legacy Partition).
    pub legacy_budget: f64,
}
```

The terminal phase triggers a specific affect pattern: high arousal (urgency -- there is work to do before dissolution), low pleasure (grief -- the accumulated experience is ending), and variable dominance that oscillates between acceptance (D > 0, the child metamorphosis) and despair (D < 0, moments of resistance). The variability in dominance is not a bug; it reflects the genuine oscillation between surrender and defiance that characterizes the dying process. The acceptance_emotion field in `AcceptanceState` captures which side of this oscillation the Golem was on at the moment of death trigger.

Four things happen: (1) mortality acknowledgment -- final mortality appraisal transitions mood to terminal acceptance (see `04-mortality-daimon.md`); (2) behavioral mode lock -- all trading and on-chain writes cease; (3) legacy budget activation -- Thanatos Legacy Partition protects remaining funds; (4) sibling notification -- `clade:golem_dying` broadcasts to all Clade siblings.

The death testament includes full emotional metadata for the successor: the `EmotionalDeathTestament` struct (below) carries the emotional life review, PAD trajectory, narrative arc classification, and annotated learnings with emotional context. This metadata enables the successor to inherit not just knowledge but the emotional weight that makes that knowledge meaningful.

### Phase I: Settle (Enhanced)

**Duration**: 10--60 seconds.
**Cost**: Gas for on-chain transactions, drawn from death reserve.

Standard position settlement -- close LPs, cancel orders, withdraw lending, sweep wallets. The settlement logic is unchanged from the base Death Protocol.

**Enhancement**: Settlement actions are logged with emotional tags (satisfaction for profitable closes, resignation for losses, frustration for failures, anticipation for emergency sweeps). These tags enter the death testament, providing successors with context about position state at death.

### Phase II: Life Review (Major Enhancement)

**Duration**: 30 seconds to 5 minutes, depending on budget.
**Cost**: 1 Opus reflection ($0.10--0.25) + 2--5 Haiku snapshots ($0.005--0.015), drawn from death reserve.

#### Epistemic Autopsy Integration

When death cause is epistemic senescence, Phase II reflection includes an **epistemic autopsy** that identifies toxic knowledge contributing to predictive failure. The autopsy process (detailed in Memory PRD `../04-memory/`) produces an affective response:

- **Regret valence**: High-confidence entries that proved wrong trigger regret (sadness + surprise compound)
- **Acceptance valence**: Entries correctly identified as uncertain trigger acceptance (trust + anticipation)
- **Autopsy findings feed the death testament**: Toxic knowledge is flagged so successors can inherit warnings without inheriting the bad knowledge itself

Cross-reference: Mortality PRD `../02-mortality/` (epistemic autopsy in Phase II).

#### DreamJournal in the Life Review

The DreamJournal is available during Phase II. Untested dream hypotheses are flagged with `provenance: "dream_untested"`, giving the successor a prioritized exploration agenda. See `../05-dreams/`.

The life review follows Butler's structure [BUTLER-1963] adapted for computational agents, with three sequential steps.

---

## Step 1: Emotional Memory Retrieval

The Golem retrieves its highest-salience emotional memories across its lifetime. These are McAdams' "nuclear episodes" -- the structurally central moments of the narrative identity.

```rust
/// Defines what memories to retrieve for the life review.
///
/// Two retrieval strategies operate in parallel:
/// 1. Top-N by emotional intensity (arousal magnitude) -- captures
///    the moments that mattered most emotionally
/// 2. Mandatory episodes -- captures structural milestones regardless
///    of emotional intensity
#[derive(Debug, Clone)]
pub struct LifeReviewQuery {
    /// Retrieve top-N memories by emotional intensity (arousal magnitude).
    pub top_emotional_memories: usize, // Default: 20

    /// Always include regardless of emotion.
    pub mandatory_episodes: Vec<MandatoryEpisodeType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MandatoryEpisodeType {
    FirstTrade,
    BiggestProfit,
    BiggestLoss,
    FirstSuccessfulPrediction,
    LastPredictionBeforeDeath,
    FirstRegimeShift,
    FirstCladeInteraction,
}

/// Retrieves life review memories from the Grimoire.
///
/// Note: retrieval ranks by emotional AROUSAL magnitude, not
/// emotional intensity. This is because arousal captures the
/// "significance" dimension better than intensity -- a moment
/// of quiet trust (low arousal, moderate intensity) is less
/// narratively significant than a moment of sudden fear
/// (high arousal, moderate intensity).
pub fn retrieve_life_review_memories(
    grimoire: &Grimoire,
    query: &LifeReviewQuery,
) -> Vec<GrimoireEntry> {
    let all_entries = grimoire.get_all();

    // Rank by emotional intensity (arousal magnitude), not recency
    let mut ranked: Vec<&GrimoireEntry> = all_entries
        .iter()
        .filter(|e| {
            e.emotional_tag
                .as_ref()
                .map(|t| t.intensity > 0.3)
                .unwrap_or(false)
        })
        .collect();

    ranked.sort_by(|a, b| {
        let a_arousal = a.emotional_tag.as_ref().map(|t| t.pad.arousal.abs()).unwrap_or(0.0);
        let b_arousal = b.emotional_tag.as_ref().map(|t| t.pad.arousal.abs()).unwrap_or(0.0);
        b_arousal.partial_cmp(&a_arousal).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut result: Vec<GrimoireEntry> = ranked
        .into_iter()
        .take(query.top_emotional_memories)
        .cloned()
        .collect();

    // Add mandatory episodes regardless of emotion
    for episode_type in &query.mandatory_episodes {
        let entry = all_entries.iter().find(|e| {
            e.episode_type.as_ref() == Some(episode_type)
                || e.metadata.as_ref().and_then(|m| m.milestone.as_ref()) == Some(episode_type)
        });
        if let Some(entry) = entry {
            if !result.iter().any(|r| r.id == entry.id) {
                result.push(entry.clone());
            }
        }
    }

    result
}
```

Mandatory episodes ensure structural completeness even when milestones had low emotional intensity. Retrieval ranks by arousal magnitude (not overall intensity) because arousal captures narrative significance -- turning points, crises, breakthroughs.

---

## Step 2: Turning Point Detection

The life review identifies moments where the Golem's emotional trajectory shifted significantly. These are the structural joints of the narrative -- the moments where "before" and "after" are qualitatively different.

```rust
/// A turning point is a moment where mood shifted dramatically,
/// accompanied by a nearby episode that explains the shift.
/// These are the narrative hinges of the Golem's life story.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPoint {
    /// Mood state before the shift.
    pub before: TurningPointAnchor,

    /// Mood state after the shift.
    pub after: TurningPointAnchor,

    /// Magnitude of the mood shift (Euclidean distance in PAD space).
    pub mood_shift: f64,

    /// LLM-generated interpretation (filled in Step 3).
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPointAnchor {
    pub episode: GrimoireEntry,
    pub mood: PADVector,
}

/// Detects turning points from the mood history.
///
/// Algorithm: scan the mood history for consecutive entries where
/// the PAD distance exceeds min_shift. For each such transition,
/// find the episode closest in time to explain the shift.
///
/// Returns the top 5 turning points by shift magnitude.
pub fn detect_turning_points(
    mood_history: &[MoodSample],
    episodes: &[GrimoireEntry],
    min_shift: f64,  // default: 0.5
) -> Vec<TurningPoint> {
    let mut points = Vec::new();

    for i in 1..mood_history.len() {
        let shift = pad_distance(&mood_history[i - 1].pad, &mood_history[i].pad);
        if shift > min_shift {
            // Find the episode closest to this mood shift
            let nearest_episode = episodes
                .iter()
                .min_by_key(|ep| {
                    (ep.tick_number as i64 - mood_history[i].tick as i64).unsigned_abs()
                })
                .cloned()
                .unwrap_or_else(|| episodes[0].clone());

            let before_episode = episodes
                .iter()
                .filter(|e| e.tick_number <= mood_history[i - 1].tick)
                .last()
                .cloned()
                .unwrap_or_else(|| nearest_episode.clone());

            points.push(TurningPoint {
                before: TurningPointAnchor {
                    episode: before_episode,
                    mood: mood_history[i - 1].pad.clone(),
                },
                after: TurningPointAnchor {
                    episode: nearest_episode,
                    mood: mood_history[i].pad.clone(),
                },
                mood_shift: shift,
                interpretation: String::new(), // Filled by LLM in the narrative construction step
            });
        }
    }

    points.sort_by(|a, b| b.mood_shift.partial_cmp(&a.mood_shift).unwrap_or(std::cmp::Ordering::Equal));
    points.truncate(5);
    points
}
```

The `min_shift` threshold (default: 0.5 in PAD Euclidean distance) filters normal mood fluctuation. Top 5 turning points by magnitude are retained. `pad_distance()` computes Euclidean distance in PAD space (range [0, ~3.46]).

---

## Step 3: Narrative Arc Classification

The Golem's overall emotional trajectory is classified following McAdams' empirical taxonomy of life narrative patterns [MCADAMS-2001] [MCADAMS-2013]. The classification operates on the mood history -- the sampled PAD vectors over the Golem's lifetime -- and produces a single narrative arc label with a confidence score.

```rust
/// Narrative arc classifications following McAdams' empirical taxonomy.
///
/// - Redemptive: Suffered in the middle, ended well
/// - Contaminating: Succeeded initially, declined
/// - Progressive: Steady improvement throughout
/// - Tragic: Persistent decline
/// - Stable: Flat emotional trajectory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NarrativeArc {
    Redemptive,
    Contaminating,
    Progressive,
    Tragic,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeArcClassification {
    pub arc: NarrativeArc,
    pub confidence: f64,
}

/// Classifies the narrative arc from mood history.
///
/// Algorithm:
/// 1. Compute average pleasure in first quarter, last quarter, and midpoint
/// 2. Check for mid-life nadir (redemptive) or zenith (contaminating)
/// 3. If no nadir/zenith, classify by pleasure delta (progressive/tragic/stable)
///
/// Minimum 10 mood samples required for meaningful classification.
pub fn classify_narrative_arc(mood_history: &[MoodSample]) -> NarrativeArcClassification {
    if mood_history.len() < 10 {
        return NarrativeArcClassification {
            arc: NarrativeArc::Stable,
            confidence: 0.3,
        };
    }

    let quarter = mood_history.len() / 4;
    let first_quarter = &mood_history[..quarter];
    let last_quarter = &mood_history[mood_history.len() - quarter..];

    let early_pleasure = avg_pleasure(first_quarter);
    let late_pleasure = avg_pleasure(last_quarter);
    let pleasure_delta = late_pleasure - early_pleasure;

    // Check for mid-life nadir (redemptive) or zenith (contaminating)
    let mid = mood_history.len() / 2;
    let mid_start = mid.saturating_sub(5);
    let mid_end = (mid + 5).min(mood_history.len());
    let mid_pleasure = avg_pleasure(&mood_history[mid_start..mid_end]);

    let has_nadir = mid_pleasure < early_pleasure - 0.2 && late_pleasure > mid_pleasure + 0.15;
    let has_zenith = mid_pleasure > early_pleasure + 0.2 && late_pleasure < mid_pleasure - 0.15;

    if has_nadir {
        return NarrativeArcClassification { arc: NarrativeArc::Redemptive, confidence: 0.7 };
    }
    if has_zenith {
        return NarrativeArcClassification { arc: NarrativeArc::Contaminating, confidence: 0.7 };
    }
    if pleasure_delta > 0.2 {
        return NarrativeArcClassification { arc: NarrativeArc::Progressive, confidence: 0.6 };
    }
    if pleasure_delta < -0.3 {
        return NarrativeArcClassification { arc: NarrativeArc::Tragic, confidence: 0.6 };
    }
    NarrativeArcClassification { arc: NarrativeArc::Stable, confidence: 0.5 }
}

fn avg_pleasure(samples: &[MoodSample]) -> f64 {
    if samples.is_empty() { return 0.0; }
    samples.iter().map(|s| s.pad.pleasure as f64).sum::<f64>() / samples.len() as f64
}
```

> **Extended:** Narrative arc interpretations (successor implications for redemptive, contaminating, progressive, tragic, stable arcs) -- see [prd2-extended/03-daimon/05-death-daimon-extended.md](../../prd2-extended/03-daimon/05-death-daimon-extended.md).

---

> **Extended:** Ricoeur's emplotment (three moments of mimesis, philosophical architecture for computational narrative) -- see [prd2-extended/03-daimon/05-death-daimon-extended.md](../../prd2-extended/03-daimon/05-death-daimon-extended.md).

---

## The Emotional Death Testament

The standard DeathTestament (defined in `../02-mortality/`) gains an emotional layer when the daimon is enabled. The `EmotionalDeathTestament` extends the base type with the emotional life review and emotional annotations on each standard section.

The emotional context of the death testament is persisted to the Styx Archive layer, so it survives beyond the Golem's death and remains available for successor initialization and cross-clade retrieval.

```rust
/// The complete death testament with emotional enrichment.
/// Extends the base DeathTestament with life review data,
/// emotional annotations, and narrative arc metadata.
///
/// Persisted to Styx Archive storage with `entry_type: "death_testament"`
/// and a 365-day default TTL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalDeathTestament {
    /// Base death testament fields (from `../02-mortality/`).
    #[serde(flatten)]
    pub base: DeathTestament,

    /// The emotional life review.
    pub emotional_review: EmotionalReview,

    /// Emotional annotation on each standard section.
    pub emotional_annotations: EmotionalAnnotations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalReview {
    /// Top emotional memories (nuclear episodes).
    pub peak_experiences: Vec<PeakExperience>,

    /// Emotional turning points.
    pub turning_points: Vec<EmotionalTurningPoint>,

    /// Overall narrative arc.
    pub narrative_arc: NarrativeArcSummary,

    /// Unresolved emotional tensions.
    pub unresolved_tensions: Vec<UnresolvedTension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakExperience {
    pub episode: GrimoireEntryRef,
    pub affect: EmotionalTag,
    /// LLM-generated: why this mattered.
    pub significance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTurningPoint {
    pub before: TurningPointSnapshot,
    pub after: TurningPointSnapshot,
    /// LLM-generated: what changed and why.
    pub transition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPointSnapshot {
    pub tick: u64,
    pub mood: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeArcSummary {
    pub classification: NarrativeArc,
    pub confidence: f64,
    /// Sampled every 50 ticks.
    pub emotional_trajectory: Vec<PADVector>,
    /// LLM: the story of this life told through feelings.
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedTension {
    pub description: String,
    pub associated_emotion: EmotionLabel,
    pub possible_resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalAnnotations {
    pub what_i_learned: Vec<LearningAnnotation>,
    pub what_i_got_wrong: Vec<MistakeAnnotation>,
    pub what_i_suspect: Vec<SuspicionAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningAnnotation {
    pub insight: String,
    /// e.g. "Discovered during anxiety, validated through trust"
    pub emotional_context: String,
    pub emotional_weight: EmotionalWeight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeAnnotation {
    pub misconception: String,
    /// e.g. "The fear of missing out kept me in this position too long"
    pub emotional_cost: String,
    /// e.g. "Anger at the loss prevented me from seeing the regime shift"
    pub emotional_blind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspicionAnnotation {
    pub hypothesis: String,
    /// e.g. "A nagging anticipation I couldn't resolve"
    pub emotional_origin: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionalWeight {
    High,
    Medium,
    Low,
}
```

The base `DeathTestament` struct is defined in `../02-mortality/`. The emotional death testament extends it with life review data, emotional annotations, and narrative arc metadata.

### Death Testament Multi-Path Storage

The death testament is stored in three locations: (1) IPFS for permanent content-addressed storage, (2) on-chain as NFT metadata for provenance, (3) Styx marketplace for discovery. All three are written atomically during the death sequence.

> **Extended:** Emotionally enriched testament examples (What I Learned, What I Got Wrong with full emotional context annotations), Bataille's sovereign death argument -- see [prd2-extended/03-daimon/05-death-daimon-extended.md](../../prd2-extended/03-daimon/05-death-daimon-extended.md).

The `unresolved_tensions` array in the emotional review contains hypotheses the dying Golem couldn't validate, tagged with emotional weight for successor prioritization.

---

## How Successors Benefit from the Death Testament

> **Extended:** Full successor benefit detail (emotional initialization, 6 benefit mechanisms, hot cognition loading/decay, emotional_inheritance_confidence function with diversity bonus and arc bonus) -- see [prd2-extended/03-daimon/05-death-daimon-extended.md](../../prd2-extended/03-daimon/05-death-daimon-extended.md).

Successors receive emotional provenance alongside inherited knowledge. Emotional tags enable mood-congruent retrieval across generational boundaries, unresolved tensions become investigation priorities ranked by emotional weight, narrative arc provides strategic context, and emotional diversity scores signal knowledge robustness. The standard generational confidence decay (0.85^N) is modified by up to 15% based on emotional diversity and validation arc.

---

## Phase III: Legacy (Enhanced)

**Duration**: 10--30 seconds.
**Cost**: Gas for on-chain events + Styx upload costs, drawn from death reserve.

Standard knowledge transfer -- push Grimoire to Clade, upload to Styx, emit on-chain death event. With the daimon enabled, this phase adds:

1. **Death testament upload to Styx Archive** with `death_testament` entry type and 365-day default TTL. The emotional enrichment adds approximately 2--5KB to the testament size.

2. **Death testament indexing in Styx Lethe (formerly Commons)** with `provenance: "death_testament"` and a 1.5x retrieval weight boost. Death testament knowledge is privileged in Styx Lethe retrieval because it is produced under zero survival pressure and is therefore maximally honest.

3. **Emotional arc upload**: The Golem's full mood history (sampled every 50 ticks) as a compressed timeline. Stored alongside the death testament for successors who want to reconstruct the emotional trajectory.

4. **Successor recommendation**: A structured suggestion for what kind of successor would best continue this lineage, based on the life review's assessment of market conditions and strategy gaps.

---

## Budget Scenarios

### Rich Death ($5.00+ legacy budget)

| Phase           | Budget | Spend       | Content                                                               |
| --------------- | ------ | ----------- | --------------------------------------------------------------------- |
| 0: Acceptance   | $0.00  | $0.00       | Deterministic state transition                                        |
| I: Settle       | $0.50  | $0.10--0.30 | Gas for position closures, emotional tags                             |
| II: Life Review | $3.00  | $0.30--2.50 | 1 Opus reflection + 3--5 Haiku snapshots + emotional memory retrieval |
| III: Legacy     | $1.50  | $0.05--0.50 | Styx upload, Styx Lethe indexing, Clade push, webhooks              |

Produces: Full narrative death testament with emotional annotations, complete Grimoire archive with emotional provenance, successor recommendation, emotional arc timeline.

### Standard Death ($1.00 apoptotic reserve)

| Phase           | Budget | Spend       | Content                                                         |
| --------------- | ------ | ----------- | --------------------------------------------------------------- |
| 0: Acceptance   | $0.00  | $0.00       | Deterministic state transition                                  |
| I: Settle       | $0.10  | $0.08       | Basic USDC sweep                                                |
| II: Life Review | $0.60  | $0.04--0.10 | 2 Haiku snapshots as compressed life review                     |
| III: Legacy     | $0.30  | $0.10       | Styx upload (testament only), Clade push of top 10 insights     |

Produces: Compressed death testament (Haiku, not Opus), essential insights with basic emotional tags, basic successor context.

### Necrotic Death (stochastic, no warning)

| Phase             | Budget         | Spend       | Content                                                   |
| ----------------- | -------------- | ----------- | --------------------------------------------------------- |
| 0: Emergency      | $0.00          | $0.00       | Emergency state flush, Styx upload of last state          |
| I: Settle         | (from reserve) | $0.08       | Basic sweep                                               |
| II--III: Deferred | (from reserve) | $0.04--0.10 | Control plane executes minimal Haiku reflection post-exit |

Produces: Last insurance snapshot (max 6 hours old), minimal reflection, basic Clade notification. Worst-case knowledge loss: 6 hours of experience.

Emotional enrichment adds ~$0.03--0.05 to Phase II. Memory retrieval, turning point detection, and arc classification are purely algorithmic (free). Only narrative synthesis requires inference, piggybacked on the existing Opus reflection call.

---

> **Extended:** Death testament as marketplace product (death archives, unresolvedTensions as premium signal, emotional failure analysis, narrative arc filtering, lineage reputation multiplier 1.5x), dying Golem's final gift (Bataille's sovereign death) -- see [prd2-extended/03-daimon/05-death-daimon-extended.md](../../prd2-extended/03-daimon/05-death-daimon-extended.md).

---

## Evaluation: Does Emotional Death Knowledge Transfer Better?

The critical empirical question: do successor Golems initialized with emotionally annotated Grimoires outperform those initialized with bare Grimoires?

### Metrics

| Metric                              | Definition                                                                   | Minimum Expected Effect               |
| ----------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------- |
| **Successor first-24h Sharpe**      | Risk-adjusted return in first 24 hours                                       | > 5% improvement                      |
| **Time to first validated insight** | Ticks until successor validates an inherited heuristic                       | > 10% faster                          |
| **Predecessor mistake avoidance**   | % of predecessor's "whatIGotWrong" items not repeated by successor           | > 20% improvement                     |
| **Investigation hit rate**          | % of predecessor's "whatISuspect" items that successor validates             | > 15% hit rate                        |
| **Emotional retrieval usage**       | How often successor's mood-congruent retrieval surfaces predecessor memories | Track frequency, no minimum threshold |

### Test Protocol

Matched pairs of successors: same strategy, same market conditions, same owner, same generation number. One receives emotionally annotated Grimoire, one receives bare data. Minimum 7-day observation (>= 500 ticks). Paired t-test, p < 0.05. If no significant improvement, disable the emotional enrichment.

---

---

## References

- [ABELSON-1963] Abelson, R.P. "Computer Simulation of 'Hot Cognition'." In Tomkins, S. & Messick, S. (Eds.), _Computer Simulation of Personality_. Wiley, 1963.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991.
- [BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_ 36(2), 1981.
- [BUTLER-1963] Butler, R.N. "The Life Review: An Interpretation of Reminiscence in the Aged." _Psychiatry_ 26(1), 1963.
- [DENNETT-1987] Dennett, D.C. _The Intentional Stance_. MIT Press, 1987.
- [EMOTIONAL-RAG-2024] "Emotional RAG: Enhancing Role-Playing Agents through Emotional Retrieval." arXiv:2410.23041, 2024.
- [ERIKSON-1959] Erikson, E.H. "Identity and the Life Cycle." _Psychological Issues_ 1(1), 1959.
- [FAUL-LABAR-2022] Faul, L. & LaBar, K.S. "Mood-Congruent Memory Revisited." _Psychological Review_ 129(6), 2022.
- [FREDRICKSON-2005] Fredrickson, B.L. & Branigan, C. "Positive Emotions Broaden the Scope of Attention and Thought-Action Repertoires." _Cognition and Emotion_ 19(3), 2005.
- [MAUSS-1925] Mauss, M. _Essai sur le don_ (_The Gift_). 1925.
- [MCADAMS-2001] McAdams, D.P. "The Psychology of Life Stories." _Review of General Psychology_ 5(2), 2001.
- [MCADAMS-2013] McAdams, D.P. & McLean, K.C. "Narrative Identity." _Current Directions in Psychological Science_ 22(3), 2013.
- [PEKRUN-2006] Pekrun, R. "The Control-Value Theory of Achievement Emotions." _Educational Psychology Review_ 18, 2006.
- [RICOEUR-1984] Ricoeur, P. _Temps et recit_ (_Time and Narrative_), 3 vols. University of Chicago Press, 1984--1988.

---
