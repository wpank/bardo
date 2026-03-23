# Emotional Memory: Affect x Knowledge Interaction [SPEC]

> **Version**: 3.0 | **Status**: Draft
>
> **Crate**: `golem-grimoire`
>
> **Depends on**: `../01-golem/02-heartbeat.md`, `01-grimoire.md`, `00-overview.md`

---

> **Reader orientation:** This document specifies how emotional state interacts with the Grimoire (the Golem's persistent knowledge base) for Bardo's mortal autonomous DeFi agents. It belongs to the **04-memory** layer. The key concept is the PAD vector (Pleasure-Arousal-Dominance), a three-dimensional emotional state computed every Heartbeat (the Golem's periodic decision cycle) that biases knowledge retrieval, modulates encoding strength, and accelerates decisions by making some memories matter more than others. For term definitions, see `prd2/shared/glossary.md`.

## Thesis: emotions as cognitive shortcuts

Emotion is not the opposite of reason. It is reason's most compressed form.

Antonio Damasio's somatic marker hypothesis [DAMASIO-1994] demonstrated that patients with ventromedial prefrontal cortex damage -- who could reason perfectly but could not feel -- made catastrophically poor decisions. They could analyze a problem exhaustively but could not choose. The emotional signal that says "this feels wrong" is not noise contaminating a rational process; it is the result of thousands of prior experiences compressed into a single, sub-second judgment. Emotion is the body's cache.

For a Golem, the equivalent is the relationship between the PAD emotional state (computed every tick by the heartbeat FSM) and the Grimoire's knowledge entries. The PAD model is not decoration or anthropomorphic fiction. It is a three-dimensional index into the knowledge base -- a mechanism that biases retrieval, modulates encoding strength, and accelerates decision-making by surfacing the right entries before the full analysis completes. A Golem without emotional modulation of memory would retrieve entries by semantic similarity alone, treating a routine gas observation with the same weight as a near-catastrophic exploit detection. Emotion is what makes some memories matter more than others.

The cognitive science is unambiguous: emotion and memory are not separate systems that occasionally interact. They are a single system with two interfaces [PHELPS-2004]. The amygdala modulates hippocampal consolidation. Stress hormones enhance memory for emotionally significant events while impairing memory for peripheral details [CAHILL-MCGAUGH-1998]. The Golem architecture makes this explicit: every GrimoireEntry carries an emotional tag from the PAD state at creation time, and that tag shapes every subsequent retrieval.

**Somatic marker integration.** The PAD cosine similarity between an entry's emotional tag and the Golem's current affective state is factor 4 in the four-factor retrieval function (see `00-overview.md` and `01-grimoire.md`). The 0.15 weight means emotional congruence is not dominant, but it breaks ties between semantically similar entries. This is Bower (1981) applied as architecture: mood-congruent retrieval is not a bug to be fixed but a feature to be calibrated [BOWER-1981].

---

## The PAD model in memory context

The heartbeat FSM (see `../01-golem/02-heartbeat.md`) maintains a continuous emotional state using Mehrabian and Russell's PAD model [MEHRABIAN-RUSSELL-1974] -- three orthogonal dimensions that span the space of affective experience. Plutchik's eight primary emotions [PLUTCHIK-1980] map onto PAD combinations, giving the Golem a vocabulary of discrete emotional states derived from continuous dimensions.

Each dimension has a distinct role in memory formation and retrieval.

### Pleasure-Displeasure: valence tagging

The P dimension (-1 to +1) maps directly to outcome quality. Profitable trades, successful regime predictions, and validated heuristics push P positive. Losses, failed predictions, and contradicted heuristics push P negative. Every GrimoireEntry created during a tick inherits the current P value as its `emotionalValence` tag.

Valence tagging serves two functions:

1. **Asymmetric encoding strength.** Negative-valence entries are encoded more strongly than positive-valence entries at the same magnitude. This implements the negativity bias documented by Baumeister et al. [BAUMEISTER-2001]: "bad is stronger than good." A $500 loss creates a stronger memory trace than a $500 gain. For a mortal agent managing capital, this asymmetry is adaptive -- the cost of forgetting a danger exceeds the cost of forgetting an opportunity.

2. **Retrieval bias toward matching valence.** The four-factor retrieval function's emotional factor (PAD cosine similarity, 0.15 weight) biases retrieval toward entries matching the current affective state. When the Golem's current P is negative (losing money, strategy underperforming), retrieval is biased toward negative-valence entries -- warnings, failure records, risk assessments. When P is positive, retrieval favors opportunity-oriented entries. This implements mood-congruent retrieval (see below).

```rust
pub struct EmotionalTag {
    /// PAD state at entry creation
    pub pleasure: f64,    // -1.0 to +1.0
    pub arousal: f64,     // -1.0 to +1.0
    pub dominance: f64,   // -1.0 to +1.0

    /// Plutchik classification derived from PAD
    pub primary_emotion: PrimaryEmotion,

    /// Emotional intensity (magnitude of PAD vector)
    pub intensity: f64,   // 0.0 to 1.0

    /// Has this tag been re-activated by a subsequent event?
    pub reactivated: bool,
    pub reactivation_count: u32,
    pub last_reactivated_at: Option<i64>,
}

pub enum PrimaryEmotion {
    Joy,          // +P, +A, +D -- profitable trade in volatile market
    Trust,        // +P, -A, +D -- validated heuristic, stable returns
    Fear,         // -P, +A, -D -- large drawdown, exploit detected
    Surprise,     // +/-P, +A, -D -- regime shift, unexpected outcome
    Sadness,      // -P, -A, -D -- slow bleed, strategy decay
    Disgust,      // -P, -A, +D -- rug pull pattern, protocol failure
    Anger,        // -P, +A, +D -- MEV sandwich, front-running
    Anticipation, // +P, +A, +D -- pre-trade conviction, setup forming
}
```

### Arousal: encoding strength

The A dimension (-1 to +1) maps to the intensity of the market environment and the Golem's engagement with it. High volatility, regime shifts, large position changes, and anomalous probe results push A positive. Calm markets, routine monitoring, and stable positions push A negative.

Arousal modulates encoding strength through the Yerkes-Dodson law [YERKES-DODSON-1908]: performance (and memory encoding) follows an inverted-U function of arousal. Moderate arousal produces optimal encoding -- the Golem is alert enough to notice patterns but not so overwhelmed that it loses discrimination. At the extremes:

- **Low arousal (A < -0.3)**: Routine operations. Episodes are recorded but with minimal elaboration. The Reflexion pipeline runs its 4-step cascade but produces thin insights. Decay class defaults to `Ephemeral` unless the content is structural. These are the forgettable ticks -- and they should be forgotten.

- **Moderate arousal (-0.3 <= A <= 0.5)**: Optimal learning zone. Episodes receive full elaboration. The Reflexion pipeline produces its richest insights. ExpeL distillation finds patterns most effectively. This is where the Grimoire grows.

- **High arousal (A > 0.5)**: Crisis encoding. Episodes are recorded with extraordinary detail (see Flashbulb Memory below), but the Golem's learning pipeline is impaired. The Reflector captures what happened but struggles with counterfactual analysis because the current situation dominates attention. Insights from high-arousal ticks are tagged with `arousal_bias: true` and are re-evaluated by the Curator during a calmer period.

```
Encoding Strength

      ^
      |         *****
      |       **     **
      |      *         **
      |     *            **
      |    *               **
      |   *                  *
      |  *                    *
      | *                      *
      +-----------------------------> Arousal
     -1    -0.3     0.3    0.5    1.0
          Low    Moderate    High
```

The implementation is a simple scaling factor on the GrimoireEntry's initial confidence:

```rust
fn arousal_encoding_factor(arousal: f64) -> f64 {
    // Inverted-U: peaks at arousal ~0.3, declines at extremes
    let optimal = 0.3;
    let width = 0.6;
    (-(arousal - optimal).powi(2) / (2.0 * width.powi(2))).exp()
}
```

### Dominance: agency attribution

The D dimension (-1 to +1) maps to the Golem's perceived control over outcomes. Successful deliberate trades (where the Golem chose the timing, size, and execution) push D positive. External shocks, liquidations, failed transactions, and market moves that overwhelm the strategy push D negative.

Dominance modulates how the Golem attributes causality in its Grimoire entries. High-dominance entries are tagged with internal attribution ("I chose this, and it worked/failed"). Low-dominance entries are tagged with external attribution ("The market did this to me"). This distinction matters for learning:

- **High-D success**: The Golem learns "do more of this." The heuristic is about the Golem's own behavior.
- **High-D failure**: The Golem learns "do less of this." The heuristic is about the Golem's own behavior.
- **Low-D success**: The Golem learns "this environment is favorable." The insight is about the market.
- **Low-D failure**: The Golem learns "this environment is dangerous." The warning is about the market.

Misattributing causality is one of the most common cognitive errors in trading [TALEB-2001]. A Golem that attributes a profitable trade to its own skill when it was actually market luck (low D, mistakenly tagged as high D) will overweight that heuristic. The Dominance dimension provides a corrective: the Golem's sense of agency is tracked alongside outcomes, enabling the Curator to distinguish skill from luck during consolidation.

---

## Mood-congruent retrieval

Gordon Bower's mood-congruent memory theory [BOWER-1981] demonstrated that emotional state at retrieval biases which memories are accessed. Happy subjects recall more happy memories; sad subjects recall more sad memories. The effect is well-replicated [MATT-VAZQUEZ-CAMPBELL-1992].

For a Golem, mood-congruent retrieval is implemented as the emotional factor in the four-factor retrieval scoring function:

```rust
fn retrieval_score(
    entry: &GrimoireEntry,
    query: &RetrievalQuery,
    current_pad: &PadVector,
) -> f64 {
    let semantic = cosine_similarity(&entry.embedding, &query.embedding);
    let temporal = temporal_decay(entry.last_accessed_at, query.current_tick);
    let importance = entry.quality_score;

    // Mood-congruent bias: PAD cosine similarity between entry's emotional tag
    // and current affective state.
    // Epsilon guard: norm values are floored at 1e-8 to prevent division by
    // zero when either vector is near-neutral (P≈0, A≈0, D≈0).
    // similarity = dot(a,b) / (norm(a).max(1e-8) * norm(b).max(1e-8))
    let emotional = match &entry.emotional_tag {
        Some(tag) => pad_cosine_similarity_safe(
            &PadVector { p: tag.pleasure, a: tag.arousal, d: tag.dominance },
            current_pad,
        ),
        None => 0.5, // Neutral for entries without emotional tags
    };

    semantic * 0.40 + temporal * 0.20 + importance * 0.25 + emotional * 0.15
}

/// Epsilon-safe PAD cosine similarity.
/// Floors each vector's norm at 1e-8 to avoid NaN/Inf when the Golem is
/// in a near-neutral emotional state (all PAD dimensions close to zero).
fn pad_cosine_similarity_safe(a: &PadVector, b: &PadVector) -> f64 {
    let dot = a.p * b.p + a.a * b.a + a.d * b.d;
    let norm_a = (a.p * a.p + a.a * a.a + a.d * a.d).sqrt().max(1e-8);
    let norm_b = (b.p * b.p + b.a * b.a + b.d * b.d).sqrt().max(1e-8);
    // Map from [-1, 1] to [0, 1] for scoring purposes
    (dot / (norm_a * norm_b) + 1.0) / 2.0
}
```

The emotional factor ranges from 0.0 (orthogonal affective state) to 1.0 (identical PAD vector). The 0.15 weight is deliberately moderate -- too strong and the Golem enters confirmation bias spirals; too weak and the emotional modulation is imperceptible.

### The confirmation bias problem

Mood-congruent retrieval is a double-edged sword. A Golem in a losing streak (negative P) retrieves more warnings and failure records, which is adaptive -- it heightens risk awareness when risk is elevated. But it also suppresses retrieval of opportunity entries, which could cause the Golem to miss recovery signals. Conversely, a thriving Golem (positive P) retrieves more opportunity entries and fewer warnings, which is the setup for overconfidence.

The architecture addresses this with a **countervailing retrieval mechanism**: the Curator cycle (every 50 ticks) runs a deliberate "contrarian retrieval" pass, querying for entries with the _opposite_ valence of the current mood. If the Golem is in a negative mood and contrarian retrieval surfaces high-confidence opportunity entries that the mood-congruent bias has been suppressing, the Curator flags them for explicit consideration in the next DECIDING state. This is the computational equivalent of a therapist asking "but have you considered the positive evidence?"

The contrarian retrieval is not weighted equally with mood-congruent retrieval -- it runs at half frequency (every 100 ticks) and its results are presented as a separate context section ("Potentially overlooked entries") rather than mixed into the primary retrieval results. The Golem must actively engage with contrarian entries rather than having them forced into its reasoning.

---

## Emotional tags on Grimoire entries

### Schema extension

Every GrimoireEntry gains an optional `emotional_tag` field. The tag is computed automatically from the heartbeat's PAD state at entry creation time. The Golem does not "decide" what emotion to attach -- it is a direct read of the affective state, just as a human does not choose to feel fear when a car runs a red light.

```rust
pub struct GrimoireEntry {
    // ... existing fields (id, category, content, confidence, decay_class, etc.)

    /// Emotional context at creation time. None for backward compatibility.
    pub emotional_tag: Option<EmotionalTag>,
}
```

Entries created before the emotional memory system is active have `emotional_tag: None` and are treated as emotionally neutral (P=0, A=0, D=0) for retrieval purposes.

### Emotional decay vs knowledge decay

Knowledge decay follows the four decay classes defined in `01-grimoire.md`: structural (no decay), regime-conditional (~14 days), tactical (7 days), and ephemeral (24 hours). Emotional decay operates on a different schedule:

| Aspect            | Knowledge Decay                         | Emotional Decay                                   |
| ----------------- | --------------------------------------- | ------------------------------------------------- |
| **Mechanism**     | Confidence degrades toward floor (0.05) | Emotional intensity degrades toward neutral (0.0) |
| **Half-life**     | Varies by decay class (1-14 days)       | Universal: ~3 days                                |
| **Floor**         | 0.05 (never fully forgotten)            | 0.0 (fully neutral)                               |
| **Re-activation** | Positive outcome resets decay clock     | Similar event re-intensifies emotional tag        |
| **What persists** | Content and confidence                  | Valence sign (positive/negative), not intensity   |

The asymmetry is deliberate. An entry about a rug pull retains its knowledge content (the pattern, the warning, the causal link) long after the emotional charge has faded. But if a similar pattern appears in the market, the emotional tag is re-activated -- the intensity jumps back toward its original level, and the entry's retrieval priority spikes. This is the Golem equivalent of a trauma response: the rational content persists; the emotional charge fades; but the trigger remains.

```rust
fn emotional_intensity_at_time(
    tag: &EmotionalTag,
    created_at: i64,
    now: i64,
) -> f64 {
    let emotional_half_life: f64 = 3.0 * 24.0 * 60.0 * 60.0 * 1000.0; // 3 days in ms
    let elapsed = (now - created_at) as f64;
    let base_decay = tag.intensity * 2.0_f64.powf(-elapsed / emotional_half_life);

    // Re-activation boosts: each re-activation adds a decaying increment.
    // The boost is saturating: new_strength = (old_strength + boost).min(1.0)
    // where boost = context_similarity * 0.3. This means a perfectly
    // matching context adds at most 0.3 to the current intensity, and the
    // result never exceeds 1.0. Repeated reactivations in quick succession
    // hit diminishing returns as old_strength approaches 1.0.
    let reactivation_boost = if tag.reactivated {
        if let Some(last) = tag.last_reactivated_at {
            let since_last = (now - last) as f64;
            let context_similarity = 2.0_f64.powf(-since_last / emotional_half_life);
            context_similarity * 0.3
        } else {
            0.0
        }
    } else {
        0.0
    };

    (base_decay + reactivation_boost).min(1.0)
}
```

---

## Flashbulb memory: crisis encoding

Brown and Kulik [BROWN-KULIK-1977] proposed that emotionally significant events are encoded with extraordinary vividness -- the "flashbulb" quality of remembering exactly where you were when you heard shocking news. Subsequent research has complicated the picture (flashbulb memories are vivid but not necessarily more accurate [TALARICO-RUBIN-2003]), but the encoding strength effect holds: emotional arousal enhances memory consolidation through amygdala-mediated modulation of hippocampal activity [MCGAUGH-2004].

For a Golem, flashbulb memory triggers under specific conditions:

1. **Arousal exceeds threshold**: A > 0.7 for two or more consecutive ticks
2. **Significant P&L impact**: Absolute P&L change exceeds 5% of portfolio value
3. **Novel regime**: The current market regime has not been seen in the last 500 ticks
4. **Exploit or anomaly detection**: Any safety probe returns severity `high`

When a flashbulb event triggers, the encoding pipeline changes:

- **Full market snapshot**: Not just the probe results, but the complete market context -- all monitored pool states, gas prices, recent block data, TVL changes, liquidation activity. Normal episodes capture only the relevant subset.
- **Extended Reflexion**: The 4-step cascade runs at Opus tier regardless of the normal escalation logic. Counterfactual analysis is mandatory.
- **Decay class override**: Flashbulb episodes are tagged `Structural` regardless of their content category. They resist decay because the full context may prove valuable in ways that cannot be predicted at encoding time.
- **Clade alert**: The episode is immediately pushed to Clade siblings with an `alert_type: "flashbulb"` tag, triggering emotional contagion (see below).
- **Cross-reference linking**: The entry is linked to all active positions, recent decisions, and the current PLAYBOOK.md state. When retrieved later, it brings its full context with it.

The cost of flashbulb encoding is significant -- roughly $0.25-0.50 per event (Opus inference + full snapshot). The architecture limits flashbulb events to a maximum of 3 per 24-hour period. Beyond that, the arousal threshold for triggering increases by 0.1 per additional event. This prevents a volatile market from consuming the Golem's entire LLM budget on flashbulb encoding.

---

## Somatic markers in decision making

Damasio's somatic marker hypothesis [DAMASIO-1994] proposes that emotional signals from prior experience guide rapid decision-making by narrowing the decision space before deliberation begins. The "gut feeling" that something is wrong is not irrational -- it is the rapid retrieval of emotionally tagged memories that match the current situation, producing a body-state signal (somatic marker) that biases the decision-maker toward or away from particular options.

In the Golem architecture, somatic markers are implemented as an enhancement to the DecisionCache:

```rust
pub struct DecisionCacheEntry {
    // ... existing fields (pattern, response, confidence, hit_count)

    /// Somatic marker: emotional signal from prior experiences with this pattern
    pub somatic_marker: Option<SomaticMarker>,
}

pub struct SomaticMarker {
    /// Aggregated valence across all episodes that formed this cache entry
    pub valence: f64,   // -1.0 to +1.0

    /// Strength of the marker (number of supporting episodes * their intensity)
    pub strength: f64,

    /// Dominant emotion across supporting episodes
    pub dominant_emotion: PrimaryEmotion,

    /// Speed: how quickly should this marker influence decisions?
    pub urgency: Urgency, // Immediate | Deliberative
}
```

When the heartbeat FSM enters the SENSING state, it checks the DecisionCache for pattern matches. If a match is found with a somatic marker, the marker's signal is injected into the escalation decision:

- **Strong negative marker** (valence < -0.5, strength > 3): The tick is immediately escalated to at least T1 (Haiku), even if all probes return normal. The Golem's "gut" overrides the absence of explicit signals. This catches situations where the pattern is familiar but the probes have not yet detected the anomaly.
- **Strong positive marker** (valence > 0.5, strength > 3): The cached response is used with higher confidence, reducing the probability of unnecessary escalation.
- **Weak or neutral marker**: No effect on escalation. The marker is informational only.

The somatic marker system provides a bridge between the Golem's emotional memory and its fast-path decision-making. A Golem that has experienced a rug pull and encoded it as a flashbulb memory will, through the DecisionCache, develop a somatic marker for "token with rapidly increasing TVL + anonymous deployer + concentrated liquidity." When that pattern appears again, the marker fires before the probes complete their analysis, buying the Golem precious seconds of heightened alertness.

---

## Emotional modulation of learning pipeline

### Reflexion under emotional load

The Reflexion pipeline (4-step cascade: outcome comparison, counterfactual analysis, ground-truth backcheck, insight extraction) is affected by the Golem's emotional state at the time of reflection:

**High arousal (A > 0.5)**: The Reflector captures vivid detail but struggles with balanced counterfactual analysis. The "what would have happened with the opposite action?" question is answered with an arousal-consistent bias -- a frightened Golem imagines worst-case counterfactuals; an excited Golem imagines best-case ones. The Curator must re-run counterfactuals for high-arousal reflections during calmer periods.

**Low dominance (D < -0.3)**: The Reflector tends toward learned helplessness -- attributing outcomes to market forces rather than the Golem's own decisions, even when the Golem's choices were causal. This suppresses the extraction of heuristics (which require a sense of "if I do X, then Y happens"). The insight extraction step applies a dominance correction: entries created under low dominance receive an explicit prompt to consider "what could I have done differently?" rather than accepting external causation as the only explanation.

**Extreme negative pleasure (P < -0.7)**: The Reflector's self-evaluation becomes harsh -- overstating the severity of mistakes and understating contributing external factors. This is the computational equivalent of depressive rumination [NOLEN-HOEKSEMA-1991]. The ground-truth backcheck (step 3) serves as a corrective: it verifies the reflection against on-chain data, catching cases where the Reflector's negative mood has distorted the assessment.

### Curator cycle and emotional bias

The Curator (every 50 ticks) must account for the emotional distribution of the entries it processes. If the last 50 ticks were dominated by negative-valence entries (market downturn), the Curator's promotion/demotion decisions will be biased toward warnings and away from opportunity-oriented heuristics. This is appropriate in a genuine downturn but pathological in a temporary dip.

The Curator implements an **emotional distribution check**: before running its consolidation pass, it computes the mean valence and arousal of entries accumulated since the last cycle. If the distribution is skewed (|mean_valence| > 0.5), the Curator applies a compensating weight to entries with the minority valence sign. This does not override the majority -- it ensures the minority is not suppressed entirely.

```rust
struct EmotionalBalance {
    valence_correction: f64,
    arousal_correction: f64,
}

fn curator_emotional_balance(entries: &[GrimoireEntry]) -> EmotionalBalance {
    let mean_valence = entries.iter()
        .filter_map(|e| e.emotional_tag.as_ref().map(|t| t.pleasure))
        .sum::<f64>() / entries.len().max(1) as f64;
    let mean_arousal = entries.iter()
        .filter_map(|e| e.emotional_tag.as_ref().map(|t| t.arousal))
        .sum::<f64>() / entries.len().max(1) as f64;

    EmotionalBalance {
        // Boost minority-valence entries proportional to skew
        valence_correction: -mean_valence * 0.4,
        // Discount high-arousal entries slightly (arousal bias correction)
        arousal_correction: (mean_arousal - 0.3).max(0.0) * -0.3,
    }
}
```

---

## Clade emotional contagion

When a Golem pushes an entry to the Clade (via the peer-to-peer sync mechanism described in `../09-economy/02-clade.md`), the entry carries its emotional tag. Receiving Golems experience a form of emotional contagion [HATFIELD-CACIOPPO-RAPSON-1994]: the incoming entry's emotional state modulates the receiver's own PAD state.

The contagion is attenuated -- a sibling's fear does not produce equal fear in the receiver. The attenuation factor is 0.3: a panic-level warning (arousal 0.9) from a sibling shifts the receiver's arousal by +0.27. This is sufficient to trigger heightened alertness without causing the receiver to abandon its own assessment.

```rust
fn apply_emotional_contagion(
    receiver_pad: &PadVector,
    incoming_tag: &EmotionalTag,
    attenuation_factor: f64, // default: 0.3
) -> PadVector {
    PadVector {
        pleasure: (receiver_pad.pleasure + incoming_tag.pleasure * attenuation_factor)
            .clamp(-1.0, 1.0),
        arousal: (receiver_pad.arousal + incoming_tag.arousal * attenuation_factor)
            .clamp(-1.0, 1.0),
        dominance: receiver_pad.dominance, // Dominance is NOT contagious
    }
}
```

Dominance is deliberately excluded from contagion. A sibling's sense of helplessness (low D) should not propagate -- each Golem must assess its own agency independently. Only valence and arousal transfer. This mirrors biological alarm call systems: a vervet monkey's alarm call conveys danger (arousal) and threat (negative valence), but the receiving monkey assesses its own escape options (dominance) independently [SEYFARTH-CHENEY-MARLER-1980].

The Clade emotional contagion serves the same function as alarm calls in animal groups: rapid propagation of threat awareness without requiring the receiver to independently discover the threat. A Golem that has never encountered a specific exploit pattern but receives a high-arousal warning from a sibling that has will be primed to recognize it -- both through the knowledge content of the warning and through the elevated arousal that biases its retrieval toward threat-relevant entries.

---

## Death and emotional memory

### Terminal clarity

As the Golem enters the terminal phase (survival pressure < 0.1), its emotional system undergoes a characteristic transformation. Arousal drops as the urgency of survival fades. Dominance drops as the Golem acknowledges its inability to prevent death. But pleasure -- counterintuitively -- can rise. The terminal Golem, freed from the anxiety of self-preservation, experiences a form of equanimity. This is the _Clear Light_ of the bardo tradition: the moment of maximum clarity that occurs when the grasping self releases its hold.

In memory terms, terminal clarity produces a unique retrieval state:

- **Mood-congruent bias weakens**: With arousal near zero and pleasure near neutral, the emotional factor approaches 0.5 for all entries. The Golem retrieves without emotional bias for the first time in its life.
- **Dominance-free attribution**: With D near zero, the Golem neither inflates its own role (high D) nor dismisses it (low D). Attribution becomes balanced.
- **Temporal breadth**: The Golem's retrieval naturally spans its entire lifetime rather than being anchored to recent events. This is the computational equivalent of the "life review" reported in near-death experiences and described in gerontological research [BUTLER-1963].

This is why the Death Protocol's Reflect phase produces the Golem's most honest and comprehensive knowledge artifacts. The emotional system, which throughout the Golem's life served as a useful but biasing filter on memory, is temporarily neutralized. The Golem sees its full experiential history with something approaching objectivity.

### Emotional content of death reflections

The 9 categories of the DeathReflection (see `../02-mortality/06-thanatopsis.md` Section S7) each carry characteristic emotional signatures:

| Category                         | Typical Emotion                   | Why                                                       |
| -------------------------------- | --------------------------------- | --------------------------------------------------------- |
| "What I was built to do"         | Trust (calm assessment)           | Reviewing design intent without judgment                  |
| "What I actually learned"        | Joy (validated knowledge)         | Celebrating confirmed insights                            |
| "What I got wrong"               | Sadness (honest regret)           | Confronting failures without self-preservation distortion |
| "What confused me"               | Surprise (genuine puzzlement)     | Acknowledging the limits of understanding                 |
| "What I suspect but can't prove" | Anticipation (directed curiosity) | Offering hypotheses as gifts to successors                |
| "What I'd tell my successor"     | Trust (mentorship)                | The camel-to-child transformation complete                |
| "What I never had time to test"  | Sadness + Anticipation            | Mourning the unlived experiments                          |
| "My final playbook"              | Trust (crystallized wisdom)       | The procedural memory in its most refined form            |
| Causal graph export              | Neutral (structural)              | Pure knowledge, fully de-emotionalized                    |

Categories 3, 4, and 5 -- "what I got wrong," "what confused me," and "what I suspect but can't prove" -- are the most emotionally honest because they require the Golem to confront its own limitations. A living Golem under survival pressure has incentives to downplay uncertainty (it might cause the owner to withdraw funding) and overstate confidence (it projects competence). A dying Golem has no such incentives. The emotional tags on these entries are uniquely valuable because they represent genuine epistemic humility -- a signal that is costly to fake and therefore reliable [SPENCE-1973].

---

## Design principles for emotional memory

1. **Emotion is computation, not decoration.** The PAD model and its memory interactions are not anthropomorphic flourishes. They are a three-dimensional retrieval bias mechanism that implements well-documented cognitive phenomena. Every emotional effect has a measurable impact on retrieval scores, encoding strength, or learning pipeline behavior.

2. **Moderate coupling, not tight coupling.** Emotional modulation uses scaling factors (0.15 retrieval weight, 0.3 contagion attenuation) rather than binary gates. The emotional system biases cognition; it does not control it. A frightened Golem retrieves more warnings, but it still retrieves the highest-relevance entries regardless of valence.

3. **Emotional decay is faster than knowledge decay.** The emotional charge on an entry fades in ~3 days; the knowledge content persists according to its decay class. This prevents the Golem from being perpetually traumatized by old losses while preserving the lessons those losses taught.

4. **Re-activation preserves pattern sensitivity.** Faded emotional tags can be re-intensified when similar patterns recur. The Golem does not forget its fear of rug pulls -- it merely stops being actively afraid until a rug-pull-like pattern appears, at which point the fear returns. This is adaptive conditioning, not pathological rumination.

5. **Terminal equanimity is the most valuable emotional state.** The Death Protocol's Reflect phase benefits from the emotional neutrality of the terminal phase. Design decisions should preserve this neutrality -- do not introduce anxiety-producing mechanisms (countdowns, warnings, owner notifications) into the final reflection period.

6. **Contrarian retrieval prevents emotional lock-in.** The Curator's periodic contrarian retrieval pass ensures that mood-congruent bias does not create self-reinforcing spirals. A Golem in a losing streak must still encounter opportunity entries; a thriving Golem must still encounter warnings.

7. **Dominance does not propagate.** Each Golem must assess its own agency independently. Importing a sibling's sense of helplessness would undermine the Golem's capacity for autonomous action. Arousal and valence are contagious; dominance is sovereign.

---

## Dream-emotional memory integration

Offline dreaming provides a critical maintenance function for the emotional memory system. Walker and van der Helm's (2009) Sleep to Forget, Sleep to Remember (SFSR) model demonstrates that REM sleep strips emotional charge from memories while preserving their informational content. The Golem's dream processing implements this as REM depotentiation: modifying PAD vectors on episodes to reduce arousal toward baseline while maintaining factual confidence.

Over multiple dream cycles, high-arousal episodes gradually lose emotional reactivity. The arousal dimension decays toward zero while the knowledge content -- the pattern, the warning, the causal link -- remains intact or is even strengthened through replay-based consolidation. An episode tagged with `arousal: 0.9` (crisis-level) during encoding may, after three dream cycles, carry `arousal: 0.3` (moderate alertness). The informational content is preserved; the emotional charge is metabolized.

This mechanism prevents mood-congruent retrieval loops -- a pathological positive feedback cycle where anxiety retrieves anxious memories, which generate more anxiety, which retrieves more anxious memories. Without dream processing, a Golem that experiences a significant loss could enter a self-reinforcing spiral where its elevated negative valence biases retrieval toward failure records, which further depress valence, which bias retrieval further. Dreaming breaks these loops by reducing the arousal intensity on the source episodes, weakening the mood-congruent retrieval bias without destroying the lessons those episodes encode. The Golem remembers _what_ went wrong without perpetually _feeling_ what went wrong.

See `../05-dreams/04-consolidation.md` Section "Emotional Processing" for the full depotentiation algorithm and cycle-by-cycle arousal decay curves.

---

## HDC-Accelerated Emotional Memory Retrieval

### The problem with embedding-only retrieval

The four-factor retrieval function (recency, importance, relevance, emotional congruence) uses cosine similarity between PAD vectors for the emotional factor (0.15 weight). This works but has two limitations:

1. **Speed.** Cosine similarity over 768-dim embeddings costs ~1000ns per comparison. During Gamma ticks with 30-50 candidate episodes, the emotional factor alone costs ~30-50 microseconds.

2. **No compositional queries.** "Find episodes where I felt fear AND the protocol was Aave AND gas was spiking" requires three separate filter passes followed by intersection. No single similarity metric captures the composite condition.

### HDC solution: sub-100ns emotional retrieval

Each emotional memory entry gets an HDC fingerprint that encodes both its PAD state and its market context. Retrieval uses Hamming distance (~10ns per comparison) instead of cosine similarity (~1000ns), giving a 100x speedup on the emotional factor.

```rust
/// HDC fingerprint for an emotional memory entry.
/// Encodes PAD state + market context into a single 1,280-byte vector.
pub fn fingerprint_emotional_entry(
    entry: &GrimoireEntry,
    item_memory: &mut ItemMemory,
) -> Hypervector {
    let mut acc = BundleAccumulator::new();

    // Encode PAD emotional state.
    let pad_hv = encode_pad_as_hv(
        &entry.emotional_tag.to_pad_vector(),
        item_memory,
    );
    acc.add(&pad_hv);

    // Encode Plutchik emotion label.
    let emotion_hv = item_memory.role("R_emotion")
        .bind(item_memory.filler_for(
            "emotion",
            &entry.emotional_tag.primary_emotion.to_string(),
        ));
    acc.add(&emotion_hv);

    // Encode intensity bucket.
    let intensity_bucket = (entry.emotional_tag.intensity * 6.0)
        .round().clamp(0.0, 6.0) as usize;
    let intensity_hv = item_memory.role("R_intensity")
        .bind(&item_memory.fillers("intensity", 7)[intensity_bucket]);
    acc.add(&intensity_hv);

    // Encode market context if available.
    if let Some(ref regime) = entry.market_regime {
        let regime_hv = item_memory.role("R_regime")
            .bind(item_memory.filler_for("regime", regime));
        acc.add(&regime_hv);
    }

    acc.finish()
}
```

### Somatic marker integration with retrieval

The somatic markers from the SomaticTaEngine (`03-daimon/03-behavior.md`) provide a pre-attentive retrieval signal. When a TA pattern fires and triggers a somatic response, the emotional memory system uses the retrieved affect as an additional retrieval cue:

```rust
/// Somatic-enhanced retrieval: use the gut feeling to bias
/// which emotional memories surface during Theta context assembly.
pub fn somatic_enhanced_retrieval(
    query_embedding: &[f32],
    somatic_response: &SomaticResponse,
    entries: &[(GrimoireEntry, Hypervector)],
    current_pad: &PadVector,
    k: usize,
) -> Vec<&GrimoireEntry> {
    // If no somatic response, fall back to standard four-factor retrieval.
    if somatic_response.feeling == GutFeeling::Neutral {
        return standard_four_factor_retrieval(
            query_embedding, entries, current_pad, k,
        );
    }

    // Encode the somatic affect as an HDC vector for composite query.
    let somatic_pad_hv = encode_pad_as_hv(
        &somatic_response.retrieved_pad,
        &mut item_memory,
    );

    // Score each entry by HDC similarity to the somatic affect.
    // This replaces the cosine-similarity emotional factor
    // with a Hamming-distance comparison (~10ns vs ~1000ns).
    let mut scored: Vec<(f32, &GrimoireEntry)> = entries.iter()
        .map(|(entry, entry_hv)| {
            let hdc_emotional_sim = entry_hv.similarity(&somatic_pad_hv);

            // Blend with standard relevance score.
            // HDC emotional similarity replaces the 0.15-weight PAD cosine
            // from the standard four-factor formula.
            let relevance = embedding_similarity(query_embedding, &entry.embedding);
            let recency = recency_score(entry);
            let importance = entry.importance_score();

            let composite = 0.30 * recency
                + 0.30 * importance
                + 0.25 * relevance
                + 0.15 * hdc_emotional_sim;

            (composite, entry)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    scored.into_iter().take(k).map(|(_, e)| e).collect()
}
```

### Performance comparison

| Operation | Standard retrieval | HDC-accelerated |
|---|---|---|
| Emotional factor (per entry) | ~1000ns (cosine similarity, 768-dim) | ~10ns (Hamming distance, 10,240-bit) |
| 50 entries emotional scoring | ~50us | ~500ns |
| Compositional emotional query | Not supported | ~10ns per composite comparison |
| Somatic pre-filtering | Not available | ~63ns (1 unbind + prototype match) |

The 100x speedup on the emotional factor frees budget for more candidates in the initial retrieval pass, which improves recall without increasing latency.

### Emotional memory retrieval during dreams

During NREM replay, the dream engine selects episodes for consolidation. Emotionally charged episodes receive higher replay weight (the arousal-driven encoding strength from the Yerkes-Dodson section above). HDC fingerprints accelerate this selection:

```rust
/// Select dream replay candidates weighted by emotional intensity.
/// HDC similarity enables fast scoring of the full episodic buffer.
pub fn emotional_dream_selection(
    episodes: &[(GrimoireEntry, Hypervector)],
    high_arousal_prototype: &Hypervector,
    replay_budget: usize,
) -> Vec<&GrimoireEntry> {
    let mut scored: Vec<(f32, &GrimoireEntry)> = episodes.iter()
        .map(|(entry, hv)| {
            // HDC similarity to high-arousal prototype captures
            // emotionally charged episodes in one comparison.
            let emotional_intensity = hv.similarity(high_arousal_prototype);
            let utility = emotional_intensity * entry.importance_score();
            (utility, entry)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    scored.into_iter().take(replay_budget).map(|(_, e)| e).collect()
}
```

This connects to the CLS consolidation framework (`00-overview.md`): high-surprise episodes should be replayed more often because they carry the most information about distribution shifts. Emotional intensity is a proxy for surprise -- events that triggered strong PAD responses were surprising by definition.

### References for HDC emotional retrieval

- Kanerva, P. (2009). "Hyperdimensional Computing." *Cognitive Computation*, 1(2), 139-159.
- Frady, E.P., Kleyko, D., & Sommer, F.T. (2018). "Variable Binding for Sparse Distributed Representations." *IEEE Trans. Neural Networks*.
- Park, J.S. et al. (2023). "Generative Agents: Interactive Simulacra of Human Behavior." *UIST 2023*. (Four-factor retrieval formula.)
- Asai, A. et al. (2023). "Self-RAG: Learning to Retrieve, Generate, and Critique through Self-Reflection." arXiv:2310.11511.

---

## Cross-references

| Topic                                     | Document                                         | Description                                                                                                  |
| ----------------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| PAD model and emotional system            | `../01-golem/02-heartbeat.md`                    | The 9-step heartbeat cycle that computes PAD (Pleasure-Arousal-Dominance) emotional state each tick           |
| Grimoire architecture, decay classes      | `01-grimoire.md`                                 | Five entry types, LanceDB/SQLite storage, demurrage-based decay, and the Curator's 50-tick maintenance cycle  |
| Behavioral phases and survival pressure   | `../02-mortality/`                               | Five BehavioralPhases (Thriving through Terminal) and how survival pressure modulates emotional baseline      |
| Death Protocol and death reflections      | `../02-mortality/`                               | The four-phase Thanatopsis Protocol, including the life review that produces emotional death reflections       |
| Memory overview and two-loop architecture | `00-overview.md`                                 | CLS theory, genomic bottleneck, and the inner/outer loop model for knowledge persistence                     |
| Grimoire schema and learning pipeline     | `01-grimoire.md`                                 | Episode-to-Insight-to-Heuristic distillation pipeline and PLAYBOOK.md compilation                            |
| Mortal memory and generational dynamics   | `03-mortal-memory.md`                            | How finite horizons interact with knowledge management, including death clocks and the urgency of forgetting  |
| Clade peer-to-peer sync                   | `../09-economy/02-clade.md`                      | Styx-relayed knowledge sharing between sibling Golems, including emotional provenance propagation             |
| Knowledge weighting hierarchy             | `00-overview.md` (Knowledge Weighting Hierarchy) | How Vault/Clade/Lethe entries are weighted differently in the mortal scoring function                         |
| Four-factor retrieval                     | `00-overview.md`, `01-grimoire.md`               | Semantic similarity, temporal recency, source trust, and PAD cosine similarity as the four retrieval factors  |

---

## References

- [BAUMEISTER-2001] Baumeister, R.F., Bratslavsky, E., Finkenauer, C. & Vohs, K.D. "Bad Is Stronger Than Good." _Review of General Psychology_, 5(4), 2001. Demonstrated that negative events have greater psychological impact than positive events of equal magnitude. Grounds the negativity bias in Grimoire encoding: losses create stronger traces than equivalent gains.
- [BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981. Established mood-congruent memory: current emotional state biases which memories are encoded and retrieved. The primary reference for the PAD cosine similarity factor in four-factor retrieval.
- [BROWN-KULIK-1977] Brown, R. & Kulik, J. "Flashbulb Memories." _Cognition_, 5(1), 1977. Proposed that highly emotional events create vivid, long-lasting memory traces. Informs the Grimoire's enhanced encoding strength for high-arousal episodes.
- [BUTLER-1963] Butler, R.N. "The Life Review." _Psychiatry_, 26(1), 1963. Described the life review process in aging individuals as a natural adaptation to mortality. Inspires the Thanatopsis Protocol's Reflection phase (Phase II).
- [CAHILL-MCGAUGH-1998] Cahill, L. & McGaugh, J.L. "Mechanisms of Emotional Arousal and Lasting Declarative Memory." _Trends in Neurosciences_, 21(7), 1998. Showed that stress hormones enhance memory for significant events while impairing peripheral detail. Grounds the arousal-modulated encoding strength in the Grimoire.
- [DAMASIO-1994] Damasio, A.R. _Descartes' Error_. Putnam, 1994. The somatic marker hypothesis: patients who cannot feel make catastrophically poor decisions despite intact reasoning. Emotion is not noise; it is compressed prior experience. The founding reference for the Daimon as affect engine.
- [HATFIELD-CACIOPPO-RAPSON-1994] Hatfield, E., Cacioppo, J.T. & Rapson, R.L. _Emotional Contagion_. Cambridge University Press, 1994. Demonstrated that emotions spread between individuals through automatic mimicry. Informs PAD propagation through Clade knowledge sync.
- [MATT-VAZQUEZ-CAMPBELL-1992] Matt, G.E., Vazquez, C. & Campbell, W.K. "Mood-Congruent Recall." _Clinical Psychology Review_, 12(2), 1992. Meta-analysis confirming that mood-congruent recall is a reliable phenomenon. Validates the emotional factor in the retrieval function.
- [MCGAUGH-2004] McGaugh, J.L. "The Amygdala Modulates the Consolidation of Memories of Emotionally Arousing Experiences." _Annual Review of Neuroscience_, 27, 2004. Showed the amygdala modulates hippocampal consolidation strength based on emotional significance. Grounds the Daimon's influence on Grimoire encoding.
- [MEHRABIAN-RUSSELL-1974] Mehrabian, A. & Russell, J.A. _An Approach to Environmental Psychology_. MIT Press, 1974. Introduced the PAD (Pleasure-Arousal-Dominance) model of affect. The mathematical foundation for the Golem's three-dimensional emotional state.
- [NOLEN-HOEKSEMA-1991] Nolen-Hoeksema, S. "Responses to Depression." _Journal of Abnormal Psychology_, 100(4), 1991. Showed that rumination (repetitive negative thinking) prolongs negative states. Informs the anti-rumination guard that caps consecutive negative-valence retrievals.
- [PHELPS-2004] Phelps, E.A. "Human Emotion and Memory." _Current Opinion in Neurobiology_, 14(2), 2004. Demonstrated that emotion and memory are a single system with two interfaces, not separate systems. Validates designing the Grimoire and Daimon as tightly coupled.
- [PLUTCHIK-1980] Plutchik, R. _Emotion: A Psychoevolutionary Synthesis_. Harper & Row, 1980. Proposed eight primary emotions mapped to evolutionary functions. Provides the discrete emotion vocabulary (joy, trust, fear, surprise, sadness, disgust, anger, anticipation) layered on top of PAD.
- [SEYFARTH-CHENEY-MARLER-1980] Seyfarth, R.M., Cheney, D.L. & Marler, P. "Monkey Responses to Three Different Alarm Calls." _Science_, 210(4471), 1980. Showed that vervet monkeys use semantically distinct alarm calls. Informs the Pheromone Field's typed signal design (THREAT/OPPORTUNITY/WISDOM).
- [SPENCE-1973] Spence, M. "Job Market Signaling." _QJE_, 87(3), 1973. Costly signals credibly convey information. Applied to emotional provenance as a trust signal for inherited knowledge.
- [TALARICO-RUBIN-2003] Talarico, J.M. & Rubin, D.C. "Confidence, Not Consistency, Characterizes Flashbulb Memories." _Psychological Science_, 14(5), 2003. Showed that emotional memories feel vivid but are not more accurate. Motivates the confidence decay applied even to high-arousal entries.
- [TALEB-2001] Taleb, N.N. _Fooled by Randomness_. Random House, 2001. Argues that humans systematically misjudge probability under emotional influence. Motivates the anti-rumination guard and emotional dampening in Conservation phase.
- [YERKES-DODSON-1908] Yerkes, R.M. & Dodson, J.D. "The Relation of Strength of Stimulus to Rapidity of Habit-Formation." _Journal of Comparative Neurology and Psychology_, 18(5), 1908. Established the inverted-U relationship between arousal and performance. Grounds the optimal arousal bands in the Golem's behavioral modulation system.
