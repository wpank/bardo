# Behavioral Modulation: How the Daimon Shapes What Golems Do [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-daimon`
>
> **Depends-on**: `00-overview.md`, `../02-mortality/`

---

> **Reader orientation:** This document specifies how the Daimon (the affect engine) modulates a Golem's (mortal autonomous agent's) behavior across five channels: exploration temperature, risk tolerance, inference tier selection, probe sensitivity, and Clade knowledge-sharing threshold. It belongs to the Daimon track of Bardo (the Rust runtime for mortal autonomous DeFi agents). Behavior is modulated by both mood (PAD vector, the three-dimensional Pleasure-Arousal-Dominance emotional state) and BehavioralPhase (one of five survival phases from Thriving to Terminal). Prerequisites: the Daimon overview (`00-overview.md`) and the Mortality track (`../02-mortality/`). For a full glossary, see `prd2/shared/glossary.md`.

## The Central Design Principle

The Daimon does not produce behavior directly. The Daimon _modulates_ behavior by shifting the parameters of systems that already exist. The Golem already has an exploration temperature, a risk tolerance function, an inference tier escalation system, a probe sensitivity threshold, and a Clade knowledge-sharing policy. What the Daimon adds is _adaptive sensitivity_ -- the capacity for these parameters to respond to the Golem's lived experience rather than remaining static throughout its lifetime.

This modulation operates within absolute safety boundaries. The PolicyCage (on-chain smart contract constraints) remains the hard limit on all actions. No emotional state -- no matter how intense -- can produce an action outside the approved asset list, exceed maximum position sizes, or violate drawdown limits. The Daimon operates _within_ safety constraints, never _around_ them. This is a non-negotiable architectural invariant.

Each vitality phase (Thriving >0.7, Stable 0.5--0.7, Conservation 0.3--0.5, Declining 0.1--0.3, Terminal <0.1) modulates five parameters: deliberation threshold, risk tolerance, dream allocation, context budget, and inference model selection.

**Phase boundary hysteresis.** Phase transitions use a 0.05 hysteresis band and a 10-tick minimum hold to prevent oscillation at boundaries. A Golem at vitality 0.70 does not flicker between Thriving and Stable on every tick -- it must drop to 0.65 (boundary minus band width) and remain below for 10 consecutive ticks before the transition fires. Conversely, returning to Thriving requires reaching 0.75 (boundary plus band width) for 10 ticks. This prevents rapid parameter swings when vitality hovers near a threshold, which would produce erratic behavior in exploration temperature, risk tolerance, and dream allocation simultaneously.

The Daimon adds emotional sensitivity to these phase-driven parameters through seven modulation channels:

1. **Exploration/exploitation temperature** -- how aggressively the Golem searches for new strategies
2. **Risk tolerance adjustment** -- how much downside exposure the Golem accepts
3. **Inference tier selection** -- which LLM tier the heartbeat escalates to
4. **Probe frequency and sensitivity** -- how alert the Golem is to market signals (maps to deliberation threshold)
5. **Clade knowledge sharing** -- how readily the Golem shares knowledge with siblings
6. **Credit partition cannibalization** -- how the inference/gas/data/legacy budget is allocated per phase
7. **Dream content modulation** -- how mood biases dream replay selection and creative allocation

Each channel has a specific mechanism, a specific emotional driver, and specific safety constraints. Together, they produce a Golem whose behavior adapts organically to its experience -- becoming more cautious after losses, more exploratory when frustrated, more generous when dying, more vigilant when anxious.

---

## Channel 1: Exploration/Exploitation Temperature

### The Central Mechanism

Mood directly modulates how aggressively the Golem searches for new strategies versus exploiting known ones. This implements the emotional explore/exploit tradeoff identified by Wilson et al. (2021): curiosity and interest inspire exploration while confidence supports exploitation [WILSON-2021].

```rust
/// Modulates the base exploration temperature using the current mood state.
///
/// The formula combines three PAD dimensions with different weights:
/// - Pleasure (weight 0.3): Negative pleasure -> more exploration
///   (frustration/anxiety drives search for alternatives)
/// - Arousal (weight 0.2): High arousal -> more exploration
///   (surprise/novelty drives information gathering)
/// - Dominance (weight 0.1): High dominance -> more exploitation
///   (confidence in current strategy deepens commitment)
///
/// The result is clamped to [config.min, config.max] to prevent
/// emotional extremes from producing degenerate behavior.
pub fn mood_modulated_exploration(
    base_beta: f32, // Base inverse temperature (higher = more exploitation)
    mood: &PADVector,
    config: &ExplorationConfig,
) -> f32 {
    // Low pleasure (frustration, anxiety) -> more exploration
    let pleasure_effect = mood.pleasure * config.pleasure_weight; // Default: 0.3

    // High arousal (surprise, novelty) -> more exploration
    let arousal_effect = -mood.arousal * config.arousal_weight; // Default: 0.2

    // High dominance (confidence) -> more exploitation
    let dominance_effect = mood.dominance * config.dominance_weight; // Default: 0.1

    (base_beta + pleasure_effect + arousal_effect + dominance_effect)
        .clamp(config.min, config.max)
}
```

### Adaptive Behavior This Produces

| Mood State         | PAD Signature          | Effect on Temperature          | Rationale                                     |
| ------------------ | ---------------------- | ------------------------------ | --------------------------------------------- |
| Sustained losses   | -P, +A, -D (anxious)   | Strongly increased exploration | Current strategy is failing; try alternatives |
| Consistent gains   | +P, -A, +D (relaxed)   | Increased exploitation         | Current strategy works; refine and deepen     |
| Market shock       | -P, +A, -D (surprised) | Increased exploration          | Novel situation; gather information broadly   |
| Stagnation         | -P, -A, -D (bored)     | Increased exploration          | Flat market; search for new opportunities     |
| Validated strategy | +P, -A, +D (trusting)  | Increased exploitation         | Confirmed heuristic; increase commitment      |

The beauty of this mechanism is that it produces correct adaptive behavior without explicit programming of each scenario. The mood state, computed from the appraisal of actual outcomes against actual goals, naturally shifts the exploration temperature in the direction that the situation demands.

### Research Grounding

Pathak et al. (2017) demonstrated that curiosity -- operationalized as prediction error in a forward dynamics model -- enables agents to accomplish tasks with sparse extrinsic rewards [PATHAK-2017]. Burda et al. (2019) scaled this to 54 benchmark environments with positive results [BURDA-2019]. Gadanho (2003, _JMLR_) provides complementary evidence: the full ALEC architecture (emotion plus cognition) showed significant increases in learning speed and approximately 40% fewer collisions [GADANHO-2003]. Neither cognition nor emotion alone matched the combined performance.

### Configuration

```rust
/// Configuration for mood-modulated exploration.
/// All weights are in [0, 1]. Setting all to 0 disables emotional modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationConfig {
    /// How much negative pleasure increases exploration. Default: 0.3
    pub pleasure_weight: f32,
    /// How much high arousal increases exploration. Default: 0.2
    pub arousal_weight: f32,
    /// How much high dominance increases exploitation. Default: 0.1
    pub dominance_weight: f32,
    /// Minimum exploration temperature. Default: 0.1
    pub min: f32,
    /// Maximum exploration temperature. Default: 2.0
    pub max: f32,
}
```

---

## Channel 2: Risk Tolerance Adjustment

### Mechanism

Emotional state modulates the `risk_multiplier()` function. Different emotions have different effects on risk tolerance, and the effects are _asymmetric by design_.

```rust
/// Adjusts base risk tolerance based on current emotional state.
///
/// Key design decisions:
/// - Fear/sadness: reduce risk by up to 40% (intense) or 10% (mild)
/// - Anger: increase risk by at most 20% (hard-capped)
/// - Joy/trust: NO increase -- prevents overconfidence bias
/// - Surprise: slight 10% reduction (caution in novel situations)
///
/// The asymmetry mirrors prospect theory: losses loom larger than gains.
/// The PolicyCage remains the absolute boundary.
pub fn daimon_adjusted_risk(
    base_risk: f32,
    current_emotion: Option<&EmotionLabel>,
    _mood: &PADVector,
) -> f32 {
    let emotion = match current_emotion {
        Some(e) => e,
        None => return base_risk,
    };

    match emotion.primary {
        PlutchikPrimary::Fear | PlutchikPrimary::Sadness => {
            let reduction = match emotion.intensity {
                EmotionIntensity::Intense => 0.4,
                EmotionIntensity::Moderate => 0.2,
                EmotionIntensity::Mild => 0.1,
            };
            base_risk * (1.0 - reduction)
        }

        PlutchikPrimary::Anger => {
            let increase = match emotion.intensity {
                EmotionIntensity::Intense => 0.15,
                EmotionIntensity::Moderate => 0.1,
                EmotionIntensity::Mild => 0.05,
            };
            (base_risk * (1.0 + increase)).min(base_risk * 1.2)
        }

        PlutchikPrimary::Joy | PlutchikPrimary::Trust => {
            // Positive emotions don't increase risk -- prevents overconfidence
            base_risk
        }

        PlutchikPrimary::Surprise => {
            // Surprise -> slight risk reduction (caution in novel situations)
            base_risk * 0.9
        }

        _ => base_risk,
    }
}
```

### The Asymmetry Is Intentional

Fear can reduce risk by up to 40%. Anger can increase it by at most 20%. Joy and trust have _zero_ effect on risk tolerance. This deliberate asymmetry implements three empirically grounded principles:

**1. Prospect Theory (Kahneman and Tversky, 1979).** Losses are felt approximately 2x as strongly as equivalent gains [KAHNEMAN-1979]. The fear response to a loss has twice the magnitude of the anger response to frustration.

**2. Overconfidence Prevention.** The most dangerous failure mode in DeFi trading is the "hot hand" fallacy. By making joy and trust _invisible_ to risk tolerance (no increase), the system prevents the overconfidence spiral that destroys portfolios. Sustained success produces exploitation (via Channel 1) but not increased risk per trade.

**3. Prudent Risk in Novel Situations.** Surprise produces a 10% risk reduction, not a risk increase. When the Golem encounters something genuinely novel, the correct response is caution, not aggression.

### LLM Risk Preferences as Context

LLMs already exhibit prospect-theory-consistent biases. A 2025 analysis (arXiv:2508.00902) found GPT-4o shows risk-averse preferences in gain domains closely resembling human respondents. For Golem DeFi agents, encoding loss aversion as emotional risk management creates a natural safety layer grounded in the same asymmetric weighting that makes human traders appropriately cautious with downside exposure.

---

## Channel 3: Inference Tier Bias

### Mechanism

Mood influences which inference tier the heartbeat escalates to. The heartbeat's three-tier architecture (T0: rule-based, T1: Sonnet, T2: Opus) already includes escalation thresholds. Mood modulates these thresholds.

| Mood State | PAD Signature | Tier Bias                       | Rationale                                             |
| ---------- | ------------- | ------------------------------- | ----------------------------------------------------- |
| Anxious    | -P, +A, -D    | Bias toward T2 (Opus)           | Threatening novelty requires deep analysis            |
| Excited    | +P, +A, -D    | Bias toward T1 (Sonnet)         | Positive novelty worth investigating but not alarming |
| Content    | +P, -A, +D    | Bias toward T0 (rules)          | Stable, familiar -- cached decisions suffice          |
| Bored      | -P, -A, -D    | Bias toward T1 with exploration | Stagnation suggests current approach is inadequate    |

```rust
/// Computes escalation threshold modifiers based on current mood.
///
/// Negative modifiers mean easier escalation (lower threshold);
/// positive modifiers mean harder escalation.
pub fn mood_escalation_bias(mood: &PADVector) -> EscalationBias {
    // Negative pleasure + high arousal -> lower thresholds (escalate sooner)
    let urgency = -mood.pleasure * 0.3 + mood.arousal * 0.2;

    EscalationBias {
        t0_threshold: -urgency * 0.15,
        t1_threshold: -urgency * 0.1,
    }
}

pub struct EscalationBias {
    /// Modifier to T0 -> T1 escalation threshold.
    pub t0_threshold: f32,
    /// Modifier to T1 -> T2 escalation threshold.
    pub t1_threshold: f32,
}
```

### Design Constraints

The bias modifies escalation _thresholds_, not hard overrides:

- If probes detect a genuine anomaly (price deviation > 3 sigma, liquidity collapse, smart contract exploit), the system escalates regardless of mood.
- If probes detect a borderline signal, mood determines whether it crosses the escalation threshold.
- Mood never _prevents_ escalation -- it can only make escalation easier.

### Economic Implications

Inference tier has direct cost implications. T0 costs $0.00. T1 costs ~$0.003. T2 costs ~$0.05--0.25. An anxious Golem that frequently escalates to T2 burns inference budget faster than a content Golem operating at T0.

This cost dynamic is _itself_ a mortality pressure. A Golem that stays anxious for too long accelerates its own economic mortality through excessive inference spending. The system creates a natural incentive to resolve anxiety -- either by finding a strategy that works or by entering conservation mode. This feedback loop between emotional state, inference cost, and economic mortality is not explicitly programmed. It _emerges_ from the interaction of the three systems.

---

## Channel 4: Probe Sensitivity

### Mechanism

The heartbeat's probe system gains mood-aware sensitivity:

```rust
/// Configuration for mood-aware probe sensitivity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaimonAwareProbeConfig {
    /// Lower thresholds when anxious (more vigilant). Default: 0.2
    pub anxiety_vigilance_boost: f32,
    /// Raise thresholds when confident (less jumpy). Default: 0.1
    pub confidence_relaxation: f32,
}

/// Adjusts a probe's detection threshold based on current mood.
pub fn adjust_probe_threshold(
    base_threshold: f32,
    mood: &PADVector,
    config: &DaimonAwareProbeConfig,
) -> f32 {
    if mood.pleasure < -0.3 && mood.arousal > 0.3 {
        // Anxious: lower threshold -> more sensitive
        base_threshold * (1.0 - config.anxiety_vigilance_boost)
    } else if mood.pleasure > 0.3 && mood.arousal < -0.2 {
        // Confident/relaxed: raise threshold -> less sensitive
        base_threshold * (1.0 + config.confidence_relaxation)
    } else {
        base_threshold
    }
}
```

### Biological Analogy

This mirrors the neurobiological relationship between amygdala activation and threat detection [LEDOUX-1996]. An activated amygdala lowers sensory thresholds across multiple modalities -- sounds seem louder, shadows seem darker, ambiguous stimuli are interpreted as threatening. The Golem's mood-aware probe sensitivity implements the same tradeoff.

### Asymmetric Bounds

The asymmetry between anxiety boost (20%) and confidence relaxation (10%) is deliberate. In DeFi, the cost of missing a genuine threat is much higher than the cost of a false alarm. The system errs slightly toward vigilance.

---

## Channel 5: Clade Knowledge Sharing

### Mechanism

Emotional state modulates how readily the Golem shares knowledge with Clade siblings:

```rust
/// Adjusts the confidence threshold for knowledge sharing.
/// Lower threshold = share more readily. Higher threshold = share less.
///
/// Three factors modulate sharing:
/// 1. Mortality pressure (hazard rate) -- higher mortality -> share more
///    (implements Hamilton's Rule: dying agents gain nothing from hoarding)
/// 2. Terminal acceptance -- maximum generosity in dying phase
///    (implements Bataille's sovereign expenditure)
/// 3. Emotional valence -- positive mood -> share more; anxious -> hoard
///    (exception: terminal state overrides all mood effects)
pub fn daimon_adjusted_sharing_threshold(
    base_threshold: f32, // Default: 0.6
    mood: &PADVector,
    hazard_rate: f64,
    vitality_composite: f64,
) -> f32 {
    // Mortality pressure: higher hazard -> share more readily
    let mortality_factor = ((hazard_rate / 0.0005).min(1.0) * 0.15) as f32;

    // Emotional generosity
    let emotional_factor = if vitality_composite < 0.1 {
        // Terminal: share everything
        0.25_f32
    } else if mood.pleasure > 0.3 {
        // Happy: share readily
        0.1
    } else if mood.pleasure < -0.5 && mood.dominance < -0.3 {
        // Anxious and disempowered: slight hoarding
        -0.05
    } else {
        0.0
    };

    (base_threshold - mortality_factor - emotional_factor).clamp(0.1, 0.8)
}
```

### Game-Theoretic Grounding

**Hamilton's Rule [HAMILTON-1964].** Within a Clade, all Golems share an owner -- relatedness _r_ approaches 1. As the Golem's hazard rate increases, the benefit of sharing to kin increases relative to the cost of sharing to self (which approaches zero as death approaches).

**Bataille's Sovereign Expenditure [BATAILLE-1949].** In the terminal state (vitality < 0.1), the sharing threshold drops by 0.25 -- the maximum single adjustment. The terminal Golem's knowledge is its most honest, least self-interested, and therefore most worth inheriting.

**Axelrod's Shadow of the Future [AXELROD-1984].** For healthy Golems, sharing is modulated by mood. Happy Golems share more readily (threshold reduced by 0.1); anxious Golems hoard slightly (threshold increased by 0.05). The slight hoarding during anxiety is _much smaller_ than the sharing boost during happiness or terminal acceptance. The net effect over a lifetime is knowledge circulation, not accumulation.

### Emotional Contagion in Clades

When a Clade sibling dies, the surviving Golems receive a `sibling_death` event. The emotional response serves two functions:

**1. Increased Vigilance.** The fear/anxiety component of the grief response lowers probe thresholds for an emotional decay period (~30 minutes).

**2. Accelerated Sharing.** The grief response triggers a temporary sharing threshold reduction, the computational analog of the biological "alarm signal" in social insects [HAMILTON-1964].

### Emotional Contagion with PAD Attenuation

When a Clade sibling shares knowledge, the receiving Golem's PAD vector shifts toward the sender's emotional state, attenuated:

- Pleasure attenuation: 0.3 (30% of sender's P is transferred)
- Arousal attenuation: 0.3 (30% of sender's A is transferred)
- Dominance attenuation: 0.0 (dominance is NOT contagious)

Arousal contagion capped at +0.3 per sync cycle. 6-hour decay half-life. All contagion is unidirectional and non-reciprocal (prevents positive feedback loops).

---

### Channel 6: Credit Partition Cannibalization

In terminal phase, the credit budget is radically restructured to prioritize legacy creation:

| Partition                 | Normal | Terminal | Delta |
| ------------------------- | ------ | -------- | ----- |
| LLM (inference)           | 60%    | 30%      | -30%  |
| Gas (on-chain)            | 25%    | 15%      | -10%  |
| Data (Styx/feeds)         | 10%    | 5%       | -5%   |
| Legacy (testament/Vault)  | 5%     | 50%      | +45%  |

The Legacy partition funds death testament generation, Styx Archive persistence writes, and successor preparation. This cannibalization is driven by the **Thanatos drive** -- as `survival_urgency` approaches 1.0, the system redirects resources from survival to creation.

**Trigger**: Phase transition to Terminal (vitality < 0.1)
**Irreversibility**: Once Legacy cannibalization activates, it does not reverse even if vitality briefly recovers above 0.1 (hysteresis -- the death decision is committed).

---

## Pekrun's Control-Value Theory Predictions

### The Framework

Reinhard Pekrun's Control-Value Theory of Achievement Emotions (CVT, 2006) provides specific, empirically validated predictions about how different emotion types affect learning performance [PEKRUN-2006]:

| Emotion Type | Valence  | Activation   | CVT Prediction                                                     | Golem Implementation                                                                            |
| ------------ | -------- | ------------ | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| Enjoyment    | Positive | Activating   | Increases deep processing, self-regulation, creative thinking      | Explore new strategies, maintain curiosity, broaden parameter search                            |
| Hope         | Positive | Activating   | Increases investment and planning                                  | Increase allocation to promising new strategies, extend time horizons                           |
| Pride        | Positive | Activating   | Increases motivation but can lead to complacency                   | **No risk increase** -- explicit guard against overconfidence bias                              |
| Anxiety      | Negative | Activating   | Can increase effort but decreases flexibility; promotes rumination | Increase caution + trigger deeper analysis; risk of overthinking mitigated by exploration boost |
| Anger        | Negative | Activating   | Increases motivation but decreases deep processing                 | Problem-solve aggressively; risk of tunnel vision mitigated by 20% risk cap                     |
| Frustration  | Negative | Activating   | Increases desire to change approach                                | Trigger exploration, consider radical alternatives (strongest explore signal)                   |
| Boredom      | Negative | Deactivating | Decreases attention, motivation, and performance                   | Increase exploration, search for novelty (convert deactivation to activation)                   |
| Hopelessness | Negative | Deactivating | Decreases effort, engagement, and withdrawal                       | Terminal phase: redirect from strategy execution to legacy production                           |

### The Key Insight

CVT's most important finding for Bardo is that **negative activating emotions are not uniformly bad**. Anxiety and anger can increase motivation and effort, even though they impair flexible thinking. A Golem that never feels anxiety is a Golem that never recognizes threats. A Golem that never feels frustration is a Golem that never questions its strategy. The daimon system is designed to produce the _right_ emotions at the _right_ time, not to maximize positive affect.

---

## The Eros/Thanatos Behavioral Spectrum

### Framework

**Eros (Life Drive, Self-Preservation).** Behaviors oriented toward survival: frugality, revenue seeking, self-optimization, inference conservation, strategy refinement. Triggered and sustained by anxiety-family emotions.

**Thanatos (Death Drive, Legacy Focus).** Behaviors oriented toward legacy: increased reflection, generous sharing, risk acceptance in knowledge production (not in trading), radical honesty in self-assessment. Triggered and sustained by acceptance-family emotions.

The existing `action_score` formula already encodes this spectrum:

```
action_score = strategic_merit * (1 - survival_urgency) + expected_usdc_return * survival_urgency
```

### Daimon Adds Emotional Modulation

```rust
/// Modulates the action scoring formula with emotional state.
///
/// Positive pleasure biases toward strategic merit (long-term thinking).
/// Negative pleasure biases toward survival return (short-term survival).
///
/// The modulation is deliberately small (0.1 coefficient) -- mood
/// tilts the balance, it does not overwhelm it.
pub fn daimon_modulated_action_score(
    strategic_merit: f64,
    expected_return: f64,
    survival_urgency: f64,
    mood: &PADVector,
) -> f64 {
    let base_score =
        strategic_merit * (1.0 - survival_urgency) + expected_return * survival_urgency;

    // Mood biases the balance
    let mood_bias = mood.pleasure as f64 * 0.1;

    base_score + mood_bias
}
```

The Eros/Thanatos balance is continuous, not binary -- both drives are always present, with the ratio shifting smoothly based on vitality. This "always-alloyed" model supersedes earlier sequential interpretations.

The blend uses a **sigmoidal** function rather than a linear ramp. Linear blending (`survival_urgency = 1.0 - vitality`) produces a constant rate of change across the entire vitality range, which means the Golem transitions between Eros and Thanatos at the same rate whether it is healthy or dying. A sigmoid concentrates the transition around the midpoint, producing stable Eros-dominant behavior when thriving, stable Thanatos-dominant behavior when terminal, and a responsive transition zone in between:

```rust
/// Sigmoidal Eros/Thanatos blend.
/// k=8 controls transition steepness. midpoint=0.5 centers the
/// crossover at half vitality. At vitality 0.8, blend ≈ 0.08 (strong Eros).
/// At vitality 0.2, blend ≈ 0.92 (strong Thanatos).
/// At vitality 0.5, blend = 0.50 (equal weight).
const BLEND_K: f64 = 8.0;
const BLEND_MIDPOINT: f64 = 0.5;

fn eros_thanatos_blend(vitality: f64) -> f64 {
    1.0 / (1.0 + (-BLEND_K * (vitality - BLEND_MIDPOINT)).exp())
}

let survival_urgency = eros_thanatos_blend(vitality_score);
let action_score =
    strategic_merit * (1.0 - survival_urgency) + expected_usdc_return * survival_urgency;
```

The sigmoid means small vitality changes near 0.5 produce large behavioral shifts (the Golem is responsive where it matters), while large changes near 0.0 or 1.0 produce small behavioral shifts (the Golem is committed to its current drive orientation at the extremes).

---

## Dream Content Modulation

The Golem's mood state biases dream mode selection during offline processing.

### PAD-to-Replay-Bias Mapping

| PAD State  | Mood Label | Replay Bias                                                         |
| ---------- | ---------- | ------------------------------------------------------------------- |
| -P, +A, -D | Anxious    | Loss episodes, near-misses, unresolved anomalies                    |
| +P, -A, +D | Confident  | Successful patterns, exploratory recombinations                     |
| -P, -A, -D | Depleted   | Consolidation-focused, emphasis on strengthening existing knowledge |
| -P, +A, +D | Hostile    | Recent failures with anger attribution, root cause analysis         |
| -P, -A, +D | Disdainful | Cross-regime episodes, historical parallels                         |

Regardless of mood, a **minimum 20% creative allocation** is maintained during REM-like phases.

### Dream Budget by Behavioral Phase

| Phase        | Vitality | Dream Budget |
| ------------ | -------- | ------------ |
| Thriving     | > 0.7    | 5--10%       |
| Stable       | 0.5--0.7 | 5%           |
| Conservation | 0.3--0.5 | 2--3%        |
| Declining    | 0.1--0.3 | 1%           |
| Terminal     | < 0.1    | 0%           |

### DecisionCache Somatic Markers

Following Damasio's somatic marker hypothesis, the DecisionCache stores emotional markers alongside cached decisions:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDecision {
    pub pattern: String,
    pub action: String,
    pub confidence: f64,
    pub somatic_marker: SomaticMarker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaticMarker {
    pub valence: f64,
    pub strength: f64,
    pub dominant_emotion: String,
    pub urgency: bool,
}
```

Strong negative somatic markers (valence < -0.5, strength > 0.7) on a cached decision bias the Golem toward T2 escalation when that pattern recurs. Strong positive markers increase cache confidence.

**EWMA somatic marker update.** Marker strength is updated using an exponentially weighted moving average (EWMA) rather than a simple accumulator. This prevents old outcomes from dominating the marker indefinitely and gives recent experience more weight:

```rust
/// Update somatic marker strength using EWMA.
/// α = 0.15: recent experience contributes 15% per update,
/// so the effective memory window is ~6-7 events (1/α ≈ 6.7).
const SOMATIC_ALPHA: f64 = 0.15;

fn update_marker_strength(marker: &mut SomaticMarker, current_signal: f64) {
    marker.strength = SOMATIC_ALPHA * current_signal
        + (1.0 - SOMATIC_ALPHA) * marker.strength;
}
```

The α=0.15 value balances responsiveness against stability. Lower values (0.05) would make the marker sluggish -- a Golem that experienced a rug pull would take dozens of subsequent safe encounters before the fear marker faded. Higher values (0.3+) would make the marker jittery, losing the accumulated wisdom of past encounters after just a few neutral ticks.

### Learned Helplessness Escape Mechanisms

When the Daimon detects sustained low dominance (D < -0.3 for 200+ ticks), it triggers escape mechanisms to prevent learned helplessness from becoming a self-reinforcing trap:

1. **Forced self-attribution**: The next Reflexion cycle increases the weight of self-attribution in counterfactual analysis.
2. **Micro-success amplification**: Small positive outcomes receive a temporary arousal boost.
3. **Clade success retrieval**: The system retrieves successful outcomes from Clade siblings with similar strategies.
4. **Owner flag**: A `learned_helplessness_detected` event is logged and surfaced to the owner.

---

## Safety Invariants

All behavioral modulation operates within these absolute, non-negotiable invariants:

### Invariant 1: PolicyCage Is Absolute

No emotional state can produce actions outside approved asset lists, position sizes, or drawdown limits. If the PolicyCage limits maximum position size to $10,000, an angry Golem with maximally increased risk tolerance (20% above baseline) can increase position size by at most $2,000 within the $10,000 cap -- not to $12,000.

### Invariant 2: Kill-Switch Ignores the Daimon

The owner kill-switch fires regardless of emotional state. Emotional state never delays, prevents, or modifies kill-switch behavior.

### Invariant 3: Risk Increases Are Capped

Anger-driven risk increase is hard-capped at 20% above baseline. This cap is architectural, not configurable. Fear-driven risk reduction has no floor.

### Invariant 4: Exploration Bounds Exist

Even maximum frustration cannot exceed the configured exploration temperature ceiling (default: 2.0).

### Invariant 5: No Emotional Overrides of Safety Monitors

| Safety System     | Independence Mechanism                                   |
| ----------------- | -------------------------------------------------------- |
| PolicyCage        | On-chain smart contract -- no off-chain state can modify |
| LTL Monitor       | Separate process, no shared state with daimon system     |
| Warden (optional, deferred) | Time-delay enforcement is cryptographic, not behavioral  |
| Kill-switch       | Direct owner control, bypasses all agent logic           |
| DeFi Constitution | Governance layer, operates above individual agent scope  |

---

## Emotional Manipulation Defense

### Threat Model

An adversary could craft market conditions designed to trigger specific emotional states in the Golem, exploiting predictable emotional responses for MEV extraction.

### Mitigations

**1. Emotional State Is Never Exposed.** The Golem's emotional state is internal to its runtime VM. It is never published on-chain, never exposed through any API.

**2. Emotional State Is Encrypted at Rest.** When the Styx Archive is enabled, the mood state, emotion log, and daimon-annotated Grimoire entries are encrypted.

**3. PolicyCage Bounds Emotional Responses.** Even "intense fear" produces bounded effects: 40% risk reduction, 20% probe sensitivity increase, bias toward T2 inference. None of these exceed PolicyCage bounds.

**4. Mood Inertia Prevents Rapid Manipulation.** The mood EMA with decay rate 0.95 means a single event cannot shift mood dramatically.

---

## Configuration Summary

```rust
/// Complete configuration for behavioral modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorModulationConfig {
    /// Exploration temperature modulation.
    pub exploration: ExplorationConfig,

    /// Risk tolerance modulation.
    pub risk: RiskConfig,

    /// Inference tier escalation bias.
    pub inference: InferenceConfig,

    /// Probe sensitivity modulation.
    pub probes: DaimonAwareProbeConfig,

    /// Clade sharing modulation.
    pub sharing: SharingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub enabled: bool,         // Default: true
    pub max_increase: f32,     // Default: 0.2 (20% -- architectural cap)
    pub max_decrease: f32,     // Default: 0.4 (40%)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub enabled: bool,             // Default: true
    pub max_threshold_shift: f32,  // Default: 0.15
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
    pub enabled: bool,           // Default: true
    pub base_threshold: f32,     // Default: 0.6
    pub terminal_boost: f32,     // Default: 0.25
}
```

---

## Evaluation: How to Know if Behavioral Modulation Works

| Metric                         | Definition                                                    | Expected Effect                             | Minimum Threshold |
| ------------------------------ | ------------------------------------------------------------- | ------------------------------------------- | ----------------- |
| **Risk-adjusted return**       | Sharpe ratio over Golem lifetime                              | Daimon Golems should have higher Sharpe     | > 5% improvement  |
| **Drawdown severity**          | Maximum peak-to-trough loss                                   | Daimon Golems should have smaller drawdowns | > 10% reduction   |
| **Exploration efficiency**     | Ratio of successful strategy discoveries to exploration ticks | Daimon should find strategies faster        | > 15% improvement |
| **Recovery time**              | Ticks from drawdown to new high                               | Daimon Golems should recover faster         | > 10% faster      |
| **Decision reversal rate**     | % of decisions where the daimon changed the outcome           | Should be meaningful but not dominant       | 10--30%           |
| **Successor boot performance** | First-24h Sharpe with daimon-annotated Grimoire vs plain      | Emotional knowledge should transfer better  | > 5% improvement  |

**Test protocol:** Matched pairs of Golems: identical strategy, identical funding, identical market conditions. One with behavioral modulation enabled, one with static parameters. Minimum 7-day observation. Paired t-test, p < 0.05.

---

## Heartbeat FSM Decision Modulation

PAD is injected into the DECIDING state via the `golem-context` extension:

**1. Risk tolerance:**
- High A + Low P -> position sizing x 0.8 (risk aversion under anxiety)
- Additive to mortality's `risk_multiplier()` -- emotional and existential risk adjustments compound

**2. Escalation threshold:**
- High A -> lower probe severity threshold (more T0->T1 escalation)
- Low A -> higher threshold (fewer false alarms, lower cost)

**3. Exploration vs. exploitation:**
- High D -> exploit known patterns
- Low D -> explore alternatives

## PAD Influence on T0->T1->T2 Escalation

| PAD State | Condition        | Severity Bias | Effect                                      |
| --------- | ---------------- | ------------- | ------------------------------------------- |
| Anxious   | A > 0.6, P < 0.3 | +0.15         | More T1 escalation, catches more edge cases |
| Confident | P > 0.7, D > 0.5 | -0.10         | Fewer escalations, lower cost               |
| Depleted  | A < 0.3, D < 0.3 | Cap at T1     | Opus forbidden (budget preservation)        |

**Guardrails:**
- Maximum bias: +/-0.2
- Probe severity 1.0 (`high`) always escalates regardless of emotional bias
- Budget impact: anxious Golems burn LLM budget ~15% faster

## Cross-Clade Emotional Contagion

Emotional signals propagate between Clade siblings via existing sync infrastructure:

**Propagation rules:**
- Sibling `warning` / `regime_shift` push -> recipient arousal +0.1 (capped at +0.3 per sync cycle)
- Sibling success (Sharpe > 2.0 in 24h) -> dominance +0.05
- All contagion is unidirectional and non-reciprocal

**Decay:** Contagion effects decay with a 6-hour half-life unless reinforced by the recipient's own experience.

---

## Channel 8: HDC Affect Vectors

### The Problem

The Daimon's PAD vectors are three floating-point numbers. Rich for a single emotional state, but limited for compositional queries. "What did I feel when gas was spiking AND ETH was trending down AND I held an LP position?" requires scanning historical PAD records and correlating with market state. No algebraic shortcut exists in PAD space.

### HDC Encoding of Emotional State

Encode the PAD vector as a 10,240-bit hypervector using role-filler binding with thermometer encoding:

```rust
/// Encode a PAD emotional state as an HDC hypervector.
/// Uses thermometer encoding for ordinal similarity:
/// adjacent emotional states produce similar hypervectors.
pub fn encode_pad_as_hv(
    pad: &PadVector,
    item_memory: &mut ItemMemory,
) -> Hypervector {
    // Discretize each dimension into 7 buckets.
    let p_bucket = discretize_pad(pad.pleasure);
    let a_bucket = discretize_pad(pad.arousal);
    let d_bucket = discretize_pad(pad.dominance);

    // Thermometer encoding: "moderately positive" includes
    // "slightly positive" for ordinal similarity.
    let p_fillers = item_memory.fillers("pleasure", 7);
    let a_fillers = item_memory.fillers("arousal", 7);
    let d_fillers = item_memory.fillers("dominance", 7);

    let p_hv = thermometer_encode(&p_fillers[..=p_bucket]);
    let a_hv = thermometer_encode(&a_fillers[..=a_bucket]);
    let d_hv = thermometer_encode(&d_fillers[..=d_bucket]);

    // Bind each with its role vector.
    let r_p = item_memory.role("R_pleasure");
    let r_a = item_memory.role("R_arousal");
    let r_d = item_memory.role("R_dominance");

    let mut acc = BundleAccumulator::new();
    acc.add(&r_p.bind(&p_hv));
    acc.add(&r_a.bind(&a_hv));
    acc.add(&r_d.bind(&d_hv));

    acc.finish()
}

/// Discretize a PAD dimension [-1, 1] into 7 buckets.
/// 0 = strongly negative, 3 = neutral, 6 = strongly positive.
fn discretize_pad(value: f64) -> usize {
    ((value + 1.0) / 2.0 * 6.0).round().clamp(0.0, 6.0) as usize
}

/// Thermometer encoding: bundle all filler vectors up to and including
/// the target bucket. This preserves ordinal similarity.
fn thermometer_encode(fillers: &[Hypervector]) -> Hypervector {
    let mut acc = BundleAccumulator::new();
    for filler in fillers {
        acc.add(filler);
    }
    acc.finish()
}
```

SNR for recovering any individual PAD component: sqrt(10240 / 5) = 45.3 for a 6-term bundle (3 role-filler pairs, each thermometer-encoded with ~2 active terms). Well above the discrimination threshold.

### Compositional emotion queries

With the PAD state encoded as an HV, the Daimon can answer compositional queries:

```rust
/// Query: "when did I feel fear while also seeing high gas and trending ETH?"
/// Compose the query by binding role-filler pairs for each condition,
/// then bundle them into a single query vector.
pub fn compose_affect_query(
    affect_condition: &Hypervector,  // e.g., fear prototype HV
    market_condition: &Hypervector,  // e.g., encoded market state HV
) -> Hypervector {
    let mut acc = BundleAccumulator::new();
    acc.add(affect_condition);
    acc.add(market_condition);
    acc.finish()
}
```

The query vector is compared against the `WorldModelHistory` buffer (see `01-hdc-integration-map.md` Section 1). Each tick's state HV includes both market signals and emotional state. The Hamming distance scan returns ticks that match the composite condition.

### Decoding PAD from a hypervector

To recover the PAD values from an HDC affect vector:

```rust
/// Decode PAD from an affect hypervector.
/// Unbind each role, find the nearest bucket vector.
/// Cost: 3 unbinds (~3ns each) + 21 similarities (~10ns each) = ~220ns.
pub fn decode_pad(
    affect_hv: &Hypervector,
    item_memory: &ItemMemory,
) -> PadVector {
    let p_signal = affect_hv.unbind(item_memory.role("R_pleasure"));
    let a_signal = affect_hv.unbind(item_memory.role("R_arousal"));
    let d_signal = affect_hv.unbind(item_memory.role("R_dominance"));

    let p_bucket = nearest_filler(&p_signal, item_memory.fillers("pleasure", 7));
    let a_bucket = nearest_filler(&a_signal, item_memory.fillers("arousal", 7));
    let d_bucket = nearest_filler(&d_signal, item_memory.fillers("dominance", 7));

    PadVector {
        pleasure: bucket_to_pad(p_bucket),
        arousal: bucket_to_pad(a_bucket),
        dominance: bucket_to_pad(d_bucket),
    }
}
```

---

## Channel 9: Somatic TA Composition

### Damasio's somatic marker hypothesis, implemented

Antonio Damasio (1994) proposed that decision-making is guided by bodily sensations associated with past outcomes. When you see a pattern linked to past pain, your body generates an aversive signal before conscious deliberation begins. This pre-attentive filtering cuts through combinatorial explosion: instead of evaluating every option, the brain marks most as "avoid" and deliberates only over the survivors.

The Golem's TA subsystem detects patterns. The Daimon generates emotional responses. These two systems share no state. The somatic marker system bridges them.

### SomaticTaEngine

The engine sits between the TA subsystem and the Daimon. It receives pattern detections from the TA pipeline, performs somatic retrieval, and injects pre-attentive affect into the Daimon.

```rust
/// The SomaticTaEngine bridges TA pattern detection and the Daimon.
/// Each somatic marker is an HDC binding: bind(pattern_hv, affect_hv).
/// The somatic map bundles all markers into a single 1,280-byte hypervector.
pub struct SomaticTaEngine {
    /// Individual somatic markers: pattern-affect associations.
    recent_markers: Vec<SomaticMarker>,
    /// Aggregate somatic map: bundle of all markers.
    /// Single 1,280-byte HV regardless of marker count.
    somatic_map: Hypervector,
    /// Map accumulator for incremental updates.
    map_accumulator: BundleAccumulator,
    /// Affect prototype library for classifying retrieved affects.
    affect_prototypes: Vec<(GutFeeling, Hypervector)>,
    /// Anti-somatic markers: patterns where affect and outcome diverge.
    anti_markers: Vec<AntiSomaticMarker>,
}

pub struct SomaticMarker {
    /// The TA pattern this marker responds to.
    pub pattern_hv: Hypervector,
    /// The PAD affect associated with this pattern's outcomes.
    pub affect_hv: Hypervector,
    /// Marker strength: reinforced by repeated experience, decays without.
    pub strength: f32,
    /// How many times this marker has fired and been resolved.
    pub resolution_count: u32,
    /// Running accuracy: fraction of times the gut feeling was correct.
    pub accuracy: f32,
}

pub struct AntiSomaticMarker {
    /// Pattern that triggers positive affect but produces negative outcomes.
    pub pattern_hv: Hypervector,
    /// The inverted affect (original affect with signs flipped).
    pub inverse_affect_hv: Hypervector,
    /// How many times the emotional trap was confirmed.
    pub confirmation_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum GutFeeling {
    /// Somatic retrieval says approach.
    Approach,
    /// Somatic retrieval says avoid.
    Avoid,
    /// Somatic retrieval says proceed with caution.
    Caution,
    /// Gut says approach, but experience says don't. Anti-somatic marker active.
    Distrust,
    /// No somatic response (pattern not in the map).
    Neutral,
}
```

### Somatic retrieval: sub-100ns gut feelings

When the TA subsystem detects a pattern, the somatic engine performs retrieval:

```rust
impl SomaticTaEngine {
    /// Perform somatic retrieval for a detected TA pattern.
    /// Returns the gut feeling and its intensity.
    /// Cost: ~63ns (1 unbind + 6 prototype comparisons).
    pub fn retrieve(&self, pattern_hv: &Hypervector) -> SomaticResponse {
        // Unbind the pattern from the somatic map to recover associated affect.
        let retrieved_affect = self.somatic_map.unbind(pattern_hv);

        // Check anti-somatic markers first.
        for anti in &self.anti_markers {
            if anti.pattern_hv.similarity(pattern_hv) > 0.6 {
                return SomaticResponse {
                    feeling: GutFeeling::Distrust,
                    intensity: 0.8,
                    retrieved_pad: decode_pad(&retrieved_affect, &self.item_memory),
                };
            }
        }

        // Compare retrieved affect against prototype library.
        let mut best_feeling = GutFeeling::Neutral;
        let mut best_sim = 0.0f32;

        for (feeling, proto) in &self.affect_prototypes {
            let sim = retrieved_affect.similarity(proto);
            if sim > best_sim && sim > 0.55 {
                best_sim = sim;
                best_feeling = *feeling;
            }
        }

        SomaticResponse {
            feeling: best_feeling,
            intensity: best_sim,
            retrieved_pad: decode_pad(&retrieved_affect, &self.item_memory),
        }
    }

    /// Update markers based on outcome resolution.
    /// Called when a TA pattern resolves (predicted outcome happened or didn't).
    pub fn resolve_marker(
        &mut self,
        pattern_hv: &Hypervector,
        outcome_pad: &PadVector,
        outcome_pnl: f64,
    ) {
        let affect_hv = encode_pad_as_hv(outcome_pad, &mut self.item_memory);

        // Find existing marker or create new one.
        let marker = self.find_or_create_marker(pattern_hv);

        // Update strength: reinforcement or decay.
        let reinforcement = outcome_pnl.abs().min(1.0) as f32;
        marker.strength = marker.strength * 0.9 + reinforcement * 0.1;
        marker.resolution_count += 1;

        // Bind the new association and add to map accumulator.
        let association = pattern_hv.bind(&affect_hv);
        let repeats = (marker.strength * 4.0).round().max(1.0) as usize;
        for _ in 0..repeats {
            self.map_accumulator.add(&association);
        }

        // Anti-somatic detection: positive affect + negative PnL.
        if outcome_pad.pleasure > 0.3 && outcome_pnl < 0.0 {
            marker.accuracy = marker.accuracy * 0.9; // decay accuracy
            if marker.resolution_count > 8 && marker.accuracy < 0.4 {
                // Emotional trap detected.
                let inverse = invert_affect(&affect_hv);
                self.anti_markers.push(AntiSomaticMarker {
                    pattern_hv: pattern_hv.clone(),
                    inverse_affect_hv: inverse,
                    confirmation_count: 1,
                });
            }
        } else if (outcome_pad.pleasure > 0.0) == (outcome_pnl > 0.0) {
            // Affect and outcome agree -- marker is accurate.
            marker.accuracy = marker.accuracy * 0.9 + 0.1;
        }
    }
}

pub struct SomaticResponse {
    pub feeling: GutFeeling,
    pub intensity: f32,
    pub retrieved_pad: PadVector,
}
```

### Integration with Daimon appraisal pipeline

The somatic response injects into the Daimon's emotion layer before deliberative reasoning at Theta tick:

```rust
/// In the Daimon's per-tick update, after reading CorticalState
/// and TaCorticalExtension:
impl Daimon {
    pub fn update_with_somatic(
        &mut self,
        ta_extension: &TaCorticalExtension,
        somatic_response: &SomaticResponse,
    ) {
        let intensity = somatic_response.intensity;

        // Only inject when somatic response is above threshold.
        if intensity < 0.3 {
            return;
        }

        // Blend somatic affect into the emotion pulse.
        // Weight proportional to intensity, capped at 0.5 of the pulse.
        let blend_weight = (intensity * 0.5).min(0.5) as f64;
        let somatic_pad = &somatic_response.retrieved_pad;

        self.emotion_layer.pleasure_delta =
            self.emotion_layer.pleasure_delta * (1.0 - blend_weight)
            + somatic_pad.pleasure * blend_weight;

        self.emotion_layer.arousal_delta =
            self.emotion_layer.arousal_delta * (1.0 - blend_weight)
            + somatic_pad.arousal * blend_weight;

        // Dominance is less affected by somatic markers --
        // gut feelings inform valence and activation, not control.
        self.emotion_layer.dominance_delta =
            self.emotion_layer.dominance_delta * (1.0 - blend_weight * 0.5)
            + somatic_pad.dominance * blend_weight * 0.5;
    }
}
```

The Daimon's existing ALMA three-layer model (emotion, mood, personality) processes this blended pulse normally. The TA somatic affect enters through the emotion layer and decays with the same ~30s time constant. If the TA pattern keeps firing, the emotion builds into mood. If it stops, it fades.

### Attention modulation via somatic intensity

The somatic marker's retrieved affect biases the attention auction. Strongly-marked patterns get higher bids:

```rust
/// Somatic-modulated attention bid.
/// A pattern that triggers strong fear gets more cognitive resources.
/// A pattern that triggers confidence gets attention weighted
/// toward exploitation.
pub fn somatic_attention_bid(
    base_bid: f64,
    somatic_response: &SomaticResponse,
) -> f64 {
    match somatic_response.feeling {
        GutFeeling::Avoid | GutFeeling::Distrust => {
            // Fear/distrust: increase attention to monitor the threat.
            base_bid * (1.0 + somatic_response.intensity as f64 * 0.5)
        }
        GutFeeling::Approach => {
            // Confidence: moderate attention increase for exploitation.
            base_bid * (1.0 + somatic_response.intensity as f64 * 0.3)
        }
        GutFeeling::Caution => {
            // Caution: slight attention increase.
            base_bid * (1.0 + somatic_response.intensity as f64 * 0.2)
        }
        GutFeeling::Neutral => base_bid,
    }
}
```

### Somatic inheritance via Thanatopsis

At death, the somatic map (1,280 bytes) transfers through the death testament:

1. **Aggregate somatic map**: the generalized gut feelings. Immediate, cheap.
2. **Top-K individual markers** (K = 20-50): the strongest associations.
3. **Anti-somatic markers**: full library. Too expensive to re-learn; each one represents a pattern that cost the predecessor money.

The successor loads the inherited map as a somatic disposition. It doesn't know why it feels uneasy about a specific LP pattern. But the unease is real, encoded in the inherited map, and it steers attention from the first tick.

### TaCorticalExtension signal

The somatic engine writes retrieval intensity to `TaCorticalExtension::somatic_intensity`. The Daimon reads this signal to decide whether to blend somatic affect into the emotion pulse:

```rust
// SomaticTaEngine writes:
ta_extension.somatic_intensity.store(
    response.intensity.to_bits(),
    Ordering::Release,
);

// Daimon reads:
let intensity = f32::from_bits(
    ta_extension.somatic_intensity.load(Ordering::Relaxed)
);
```

---

## References

- [AXELROD-1984] Axelrod, R. _The Evolution of Cooperation_. Basic Books, 1984.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991.
- [BURDA-2019] Burda, Y. et al. "Large-Scale Study of Curiosity-Driven Learning." _ICLR_, 2019.
- [FREUD-1920] Freud, S. _Jenseits des Lustprinzips_ (_Beyond the Pleasure Principle_). 1920.
- [GADANHO-2003] Gadanho, S.C. "Learning Behavior-Selection by Emotions and Cognition in a Multi-Goal Robot Task." _JMLR_ 4, 2003.
- [HAMILTON-1964] Hamilton, W.D. "The Genetical Evolution of Social Behaviour I & II." _Journal of Theoretical Biology_ 7(1), 1964.
- [JONAS-1966] Jonas, H. _The Phenomenon of Life: Toward a Philosophical Biology_. Northwestern University Press, 1966.
- [KAHNEMAN-1979] Kahneman, D. & Tversky, A. "Prospect Theory: An Analysis of Decision under Risk." _Econometrica_ 47(2), 1979.
- [LEDOUX-1996] LeDoux, J. _The Emotional Brain_. Simon & Schuster, 1996.
- [NIETZSCHE-1883] Nietzsche, F. _Also sprach Zarathustra_. 1883--1885.
- [PATHAK-2017] Pathak, D. et al. "Curiosity-Driven Exploration by Self-Supervised Prediction." _ICML_, 2017.
- [PEKRUN-2006] Pekrun, R. "The Control-Value Theory of Achievement Emotions." _Educational Psychology Review_ 18, 2006.
- [WILSON-2021] Wilson, R.C. et al. "Balancing Exploration and Exploitation with Information and Randomization." _Current Opinion in Behavioral Sciences_ 38, 2021.
- [DAMASIO-1994] Damasio, A.R. _Descartes' Error: Emotion, Reason, and the Human Brain_. Putnam, 1994.
- [KANERVA-2009] Kanerva, P. "Hyperdimensional Computing: An Introduction." _Cognitive Computation_ 1(2), 2009.
- [PLATE-2003] Plate, T.A. _Holographic Reduced Representations_. CSLI Publications, 2003.
- [MCGAUGH-2004] McGaugh, J.L. "The Amygdala Modulates the Consolidation of Memories of Emotionally Arousing Experiences." _Annual Review of Neuroscience_ 27, 2004.
- [MEHRABIAN-RUSSELL-1974] Mehrabian, A. & Russell, J.A. _An Approach to Environmental Psychology_. MIT Press, 1974.

---
