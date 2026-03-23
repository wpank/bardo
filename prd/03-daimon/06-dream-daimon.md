# Dream-Daimon Bridge: How Emotions and Dreaming Shape Each Other [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-daimon`, `golem-dreams`
>
> **Canonical sources**: `../05-dreams/` (overview, architecture, replay, imagination, consolidation)
>
> **Depends-on**: `00-overview.md`, `01-appraisal.md`, `02-emotion-memory.md`

---

> **Reader orientation:** This document specifies the bidirectional bridge between the Daimon (the affect engine) and the Dream Engine (offline intelligence system) within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers how emotional load drives dream urgency, how mood biases replay content selection, how REM depotentiation reduces arousal on high-charge memories, and how dream outcomes feed back into appraisal. Prerequisites: the Daimon overview (`00-overview.md`), the appraisal spec (`01-appraisal.md`), and the Dreams track (`../05-dreams/`). For a full glossary, see `prd2/shared/glossary.md`.

## Summary

The Dream Engine (`golem-dreams`) and the Daimon (`golem-daimon`) have a bidirectional relationship: the Daimon shapes what the Golem dreams about, and dreaming reshapes the Daimon's emotional state. This bridge document specifies the interaction points, shared constants, and data flows between the two systems. For full dream architecture, see the Dreams PRD suite. For full daimon architecture, see `00-overview.md`.

---

## 1. Daimon -> Dreams: How Emotions Shape Dreaming

### 1.1 Emotional Load in Dream Urgency

The DreamScheduler computes a composite urgency score that determines when the next dream cycle fires. Emotional load is one of five urgency components:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamUrgency {
    pub novelty: f64,
    pub consolidation_debt: f64,
    pub emotional_load: f64,
    pub threat_exposure: f64,
    pub time_since_last_dream: u64,
}

/// Compute emotional load: average absolute arousal over recent episodes.
///
/// High emotional load means many recent experiences haven't been
/// emotionally processed -- the Golem needs REM depotentiation.
pub fn compute_emotional_load(
    recent_episodes: &[Episode],
    window: usize,  // default: 50
) -> f64 {
    let windowed = if recent_episodes.len() > window {
        &recent_episodes[recent_episodes.len() - window..]
    } else {
        recent_episodes
    };

    if windowed.is_empty() {
        return 0.0;
    }

    let total_arousal: f64 = windowed
        .iter()
        .map(|ep| {
            ep.emotional_tag
                .as_ref()
                .map(|t| t.pad.arousal.abs() as f64)
                .unwrap_or(0.0)
        })
        .sum();

    total_arousal / windowed.len() as f64
}
```

An emotional load above 0.5 indicates sustained high-arousal experience that has not been processed through dreaming. This threshold interacts with the behavioral phase: a conservation-phase Golem with emotional load 0.6 will trigger a dream cycle sooner than a thriving-phase Golem with the same load, because conservation-phase Golems have less cognitive margin.

### 1.1b DreamScheduler Struct

The `DreamScheduler` gates dream cycle initiation using a minimum tick interval, an emotional load threshold, and recency tracking:

```rust
/// Controls when dream cycles fire based on emotional load and temporal spacing.
/// The scheduler prevents dream spam during sustained high-arousal periods
/// by enforcing a minimum interval between cycles.
pub struct DreamScheduler {
    /// Minimum ticks between dream cycles. Default: 500.
    /// Prevents the Golem from dreaming every time emotional load spikes.
    pub min_interval_ticks: u32,    // 500
    /// Emotional load must exceed this threshold to trigger a dream cycle.
    /// Below this, the Golem's emotional state is manageable without dreaming.
    pub emotional_load_threshold: f32, // 0.7
    /// Tick number of the most recent dream cycle completion.
    pub last_dream_tick: u32,
}
```

After dream completion, the emotional load is aggressively reset: `emotional_load *= 0.3`. This reflects the biological observation that REM processing discharges accumulated emotional pressure. The 0.3 multiplier means a Golem entering a dream cycle with emotional load 0.9 exits with load 0.27 -- well below the urgency threshold, giving it headroom before the next cycle becomes necessary.

### 1.2 Mood-Biased Replay Content

The Daimon's current PAD state biases which episodes the Dream Engine selects for NREM replay:

| PAD State               | Replay Bias                                             | Dream Content Character                      |
| ----------------------- | ------------------------------------------------------- | -------------------------------------------- |
| -P, +A, -D (Anxious)    | Loss episodes, near-misses, unresolved anomalies        | Defensive: process threats and failures      |
| +P, -A, +D (Confident)  | Successful patterns, novel recombinations               | Exploratory: build on strengths              |
| -P, -A, -D (Depleted)   | Consolidation-focused, existing knowledge reinforcement | Conservative: strengthen foundations         |
| -P, +A, +D (Hostile)    | Recent failures with anger attribution                  | Diagnostic: find root causes                 |
| -P, -A, +D (Disdainful) | Cross-regime episodes, historical parallels             | Corrective: break out of dismissive patterns |

The replay bias operates as a scoring modifier on the Dream Engine's utility-weighted selection (Mattar & Daw, 2018): `utility = gain x need`, where emotional need is one factor in the Need component. Mood shifts the Gain and Need weights without overriding the utility computation. The 20% minimum creative allocation is enforced regardless of mood state.

### 1.3 Emotional Tags Guide NREM Replay Priority

High-arousal episodes are replayed preferentially during NREM-like phases, matching the neurobiological pattern where amygdala-hippocampal interactions privilege emotionally significant experiences for consolidation during sleep (McGaugh, 2004). The emotional salience score enters the replay utility function as a multiplier on the Gain component.

### 1.4 Creative Dreaming Mood Modulation

During REM-like phases, mood and behavioral phase jointly determine the balance between Boden's three creativity modes:

- **Combinational** (recombine known patterns): Emphasized when the Golem is in conservation or declining phase
- **Exploratory** (test boundaries of known space): Balanced across all phases
- **Transformational** (challenge fundamental assumptions): Emphasized when thriving and confident

See `03-behavior.md` for the full phase x mood allocation table.

---

## 2. Dreams -> Daimon: How Dreaming Shapes Emotions

### 2.1 REM Depotentiation

The SFSR model (Walker & van der Helm, 2009) is the Dream Engine's primary mechanism for reshaping the Daimon's emotional state. During REM-like phases, high-arousal experiences are depotentiated.

**What is preserved:** Pleasure and dominance components of the PAD vector -- these encode the _lesson_ (what happened and whether the Golem had control). The informational content of the episode remains intact.

**What is reduced:** The arousal component decays toward baseline across dream cycles. The decay is **multiplicative, not additive**: `arousal_new = arousal_old * 0.70` per cycle for episodes with arousal > 0.5. This means each cycle removes 30% of the _remaining_ arousal, producing exponential convergence toward baseline rather than a fixed-step linear walk. The distinction matters: additive decay (subtract 0.3 each cycle) would drive arousal negative after enough cycles, while multiplicative decay asymptotically approaches zero without crossing it.

This implements the dream extension (#21 in the extension registry, see `01a-runtime-extensions.md`), which hooks into `after_turn` to check whether dreaming is warranted based on emotional load, novelty, and time since last dream.

**The depotentiation mechanism**: REM reduces the arousal component while preserving the informational content of the memory. This is the computational equivalent of what Walker & van der Helm describe: "sleep to forget the emotion, sleep to remember the content." The neurochemical environment during REM -- low norepinephrine, high acetylcholine -- allows the amygdala to reprocess emotional memories without the stress hormones that would reinforce the emotional response. The Golem's implementation mirrors this by selectively decaying arousal while leaving pleasure and dominance untouched.

**Example trajectory:**

```
Original encoding:  P:-0.8, A: 0.9, D:-0.5  (traumatic flash crash)
After 1 dream cycle: P:-0.6, A: 0.6, D:-0.4  (arousal reduced, lesson preserved)
After 2 dream cycles: P:-0.5, A: 0.4, D:-0.3  (further depotentiation)
After 3 dream cycles: P:-0.4, A: 0.2, D:-0.3  (calmly remembered warning)
```

**Functional purpose:** Depotentiation breaks mood-congruent retrieval loops. Without it, a negative memory with high arousal would be preferentially retrieved whenever the Golem is anxious, reinforcing the anxiety, which further biases retrieval toward negative memories. Depotentiation reduces the arousal "hook" that makes the memory hyper-retrievable while preserving its informational value.

### 2.2 Dream Outcome Appraisal

When a dream-validated hypothesis is confirmed or refuted in live trading, the appraisal engine fires a `dream_outcome` trigger:

| Outcome               | Emotion           | PAD                    | Interpretation                                     |
| --------------------- | ----------------- | ---------------------- | -------------------------------------------------- |
| Hypothesis validated  | Mild joy          | P:+0.3, A:+0.1, D:+0.4 | High dominance: "my offline cognition was correct" |
| Hypothesis refuted    | Moderate surprise | P:-0.2, A:+0.4, D:-0.1 | Low dominance: "my dream insight was wrong"        |
| Novel dream insight   | Mild anticipation | P:+0.2, A:+0.3, D:+0.2 | Forward-looking interest in a new connection       |
| Dream threat realized | Moderate fear     | P:-0.4, A:+0.5, D:-0.3 | The rehearsed threat actually occurred             |

Dream outcomes also fire for novel insights (`dream_insight`) and threat discoveries (`dream_threat`). See `01-appraisal.md` for the full appraisal functions.

### 2.2b Dream Outcome Emotional Feedback Loop

The dream-emotion cycle is closed: dream insights that validate in live trading produce pleasure (P: +0.3, D: +0.4 -- "my offline cognition was correct"), while contradictions produce arousal (A: +0.4, P: -0.2 -- surprise at being wrong). This emotional feedback biases future dream content selection. A Golem whose dream hypotheses keep validating enters a virtuous cycle: pleasure boosts confidence, confidence biases toward exploitation during dreams, and exploitation during dreams generates more targeted hypotheses. A Golem whose dream hypotheses keep failing enters a corrective cycle: surprise boosts arousal, arousal biases toward exploration during dreams, and exploration generates more diverse hypotheses. Neither cycle is permanent -- mood decay toward personality baseline ensures the Golem returns to its natural equilibrium.

### 2.3 Depotentiation as Mood-Congruent Loop Breaker

REM depotentiation is the primary mechanism for preventing depressive rumination in the Daimon system. The causal chain it interrupts:

```
Negative mood -> mood-congruent retrieval -> negative memories surface
-> negative memories reinforce negative mood -> (loop)
```

By reducing the arousal of negative memories during dreaming, depotentiation weakens the mood-congruent retrieval bias for those specific memories. The memories are still retrievable (their semantic content is unchanged), but they no longer have the elevated arousal that makes them preferentially surface during anxious states.

This mechanism complements the contrarian retrieval schedule (every 100 ticks, see `02-emotion-memory.md`) but operates at a different level: contrarian retrieval forces diversity in what is _retrieved_, while depotentiation changes the emotional charge of what _can be_ retrieved.

---

## 3. DreamJournal Emotional Metadata

Dream outputs that enter the Grimoire carry emotional metadata recording both the original encoding emotion and the dream-modified emotion:

```rust
/// Emotional metadata attached to dream-produced Grimoire entries.
/// Records the full emotional processing history: original encoding
/// state, post-dream-processing state, and the processing provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamEmotionalMetadata {
    /// PAD vector at the time of original episode encoding.
    pub original_pad: PADVector,

    /// PAD vector after dream processing (depotentiation applied).
    pub dream_processed_pad: PADVector,

    /// Number of depotentiation cycles this entry has undergone.
    pub depotentiation_cycles: u32,

    /// Which dream phase produced or processed this entry.
    pub dream_phase: DreamPhase,

    /// The type of dream processing applied.
    pub dream_type: DreamType,

    /// Emotional load at the time the dream cycle began.
    pub emotional_load_at_dream_entry: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DreamPhase {
    Nrem,
    Rem,
    Integration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DreamType {
    Replay,
    Counterfactual,
    Creative,
    Threat,
}
```

This dual provenance allows the Grimoire to track the full emotional processing history: "discovered under panic, depotentiated through 2 dream cycles, now held with calm confidence." Successors inheriting these entries receive not just the insight but the evidence that it was emotionally processed -- knowledge that has been "slept on" carries different weight than knowledge that was encoded under acute emotional pressure.

---

## 4. Shared Constants

| Constant                           | Value                                     | Used By                |
| ---------------------------------- | ----------------------------------------- | ---------------------- |
| Emotional load window              | 50 ticks                                  | DreamScheduler, Daimon |
| Emotional load urgency threshold   | 0.5                                       | DreamScheduler         |
| DreamScheduler min interval        | 500 ticks                                 | DreamScheduler         |
| DreamScheduler emotional threshold | 0.7                                       | DreamScheduler         |
| Post-dream emotional load reset    | `emotional_load *= 0.3`                   | DreamScheduler, Daimon |
| REM depotentiation rate            | 30% multiplicative per cycle (A > 0.5)    | Dream Engine, Daimon   |
| Minimum creative allocation        | 20% of REM budget                         | Dream Engine           |
| Dream outcome appraisal: validated | P:+0.3, A:+0.1, D:+0.4                    | Appraisal Engine       |
| Dream outcome appraisal: refuted   | P:-0.2, A:+0.4, D:-0.1                    | Appraisal Engine       |
| Dream hypothesis base confidence   | 0.3                                       | Dream Engine, Grimoire |
| Dream-validated entry decay rate   | 0.5x normal                               | Demurrage system       |

---

## 5. Events Emitted

The dream-daimon bridge emits events through the Event Fabric when dreaming modifies the mood state. These are typed `GolemEvent` enum variants, not SSE payloads.

| GolemEvent Variant | Trigger | Key Fields |
|---|---|---|
| `DreamStart` | Dream cycle begins | `emotional_load`, `dream_type_allocation` |
| `DreamPhase` | Dream phase transition | `phase`, `episode_count` |
| `DreamHypothesis` | Hypothesis generated during REM | `hypothesis`, `confidence`, `emotional_origin` |
| `DreamCounterfactual` | Counterfactual imagined during REM | `hypothesis`, `outcome` |
| `DreamComplete` | Dream cycle completes | `depotentiation_count`, `hypotheses_generated`, `mood_shift` |

When dream processing modifies the PAD vector by more than the 0.15 emission threshold, a `MoodUpdate` event is also emitted. See `14-events.md` (rewrite4) for the full GolemEvent enum and wire format.

---

## 6. Cross-References

| Topic                                | Document               |
| ------------------------------------ | ---------------------- |
| Dream architecture and phases        | `../05-dreams/`        |
| Replay prioritization                | `../05-dreams/`        |
| Creativity modes                     | `../05-dreams/`        |
| Dream consolidation and staging      | `../05-dreams/`        |
| Threat simulation                    | `../05-dreams/`        |
| Cross-track integration              | `../05-dreams/`        |
| Extension #21 (dream)               | `01a-runtime-extensions.md` |
| Daimon overview                      | `00-overview.md`       |
| Appraisal triggers                   | `01-appraisal.md`      |
| Emotional memory                     | `02-emotion-memory.md` |
| Behavioral modulation (dream budget) | `03-behavior.md`       |
| Death protocol (DreamJournal)        | `05-death-daimon.md`   |
| Immortal mode dream-emotion          | `../02-mortality/`     |

---
