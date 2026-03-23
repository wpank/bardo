# Mortality-Aware Daimon: How Golems Feel About Dying [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-daimon`, `golem-mortality`
>
> **Depends-on**: `00-overview.md`, `../02-mortality/`

---

> **Reader orientation:** This document specifies how a Golem's (mortal autonomous agent's) awareness of its own mortality produces emotional states within the Daimon (the affect engine). It maps the three mortality clocks (economic depletion, epistemic stagnation, stochastic hazard) to distinct emotional responses and specifies the terminal acceptance state. Part of the Daimon track within Bardo (the Rust runtime for mortal autonomous DeFi agents). Prerequisites: the Daimon overview (`00-overview.md`) and the Mortality track (`../02-mortality/`). For a full glossary, see `prd2/shared/glossary.md`.

> **See also:** `../02-mortality/` (the Mortality track: three mortality clocks, five BehavioralPhases, VitalityState composite score, Eros/Thanatos personality spectrum) for the canonical mortality system specification of affect integration.

## Document Overview

Mortality creates urgency, and urgency produces efficiency and intelligence. This is the Die moat: agents that can die allocate resources differently than agents that cannot. The mortality-triggered affect shifts specified in this document are the psychological experience of economic pressure -- the mechanism by which a declining USDC balance becomes not just a number but a felt constraint that reshapes every downstream decision. Low vitality reduces dominance, which produces sadness and fear emotions, which lower the deliberation threshold, which forces more careful decisions. The entire chain is automatic, grounded in concrete metrics, and costs zero inference tokens.

No existing computational emotion framework -- neither OCC, ALMA, EMA, FAtiMA, nor Chain-of-Emotion -- addresses how agents feel about their own mortality. These frameworks model reactions to external events: a trade that went wrong, a market that shifted, a peer that defected. They do not model the _background awareness of finitude_ that colors every decision a mortal agent makes. This document specifies Bardo Daimon's mortality-aware daimon system: three mortality-specific emotions, their composite appraisal, and the terminal acceptance state that emerges as death approaches.

The philosophical foundation is Heidegger's _Befindlichkeit_ (attunement, mood, "how one finds oneself") [HEIDEGGER-1927]. Heidegger argued that mood is not a reaction to events but the _ground state_ through which events are disclosed. A human in anxiety does not merely react to a threat -- anxiety reveals the world _as threatening_. Similarly, a Golem's mortality awareness does not merely respond to credit thresholds -- it reveals the strategic landscape _as finite_, coloring every decision with temporal urgency. This is not decorative philosophy. It is the design principle that separates Bardo's daimon system from every prior computational emotion architecture.

---

## The Three Mortality Clocks, The Three Mortality Emotions

The Golem faces three independent mortality pressures (see `../02-mortality/`): economic (USDC balance depletion), epistemic (predictive fitness decay), and stochastic (per-tick hazard rate). Each clock generates a distinct emotional signature. These are not generic fear or sadness responses. They are structurally specific emotional states that arise from the Golem's relationship to each dimension of its own finitude.

| Clock      | Emotion           | Character                     | Philosophical Root                   |
| ---------- | ----------------- | ----------------------------- | ------------------------------------ |
| Economic   | Economic Anxiety  | Resource scarcity awareness   | Jonas's "needful freedom"            |
| Epistemic  | Epistemic Vertigo | Obsolescence awareness        | Dane's cognitive entrenchment crisis |
| Stochastic | Stochastic Dread  | Background finitude awareness | Heidegger's Angst                    |

The three emotions differ not just in their triggers but in their phenomenological character. Economic anxiety is sharp and immediate -- the organism feeling its metabolic substrate deplete. Epistemic vertigo is destabilizing and recursive -- the realization that one's own capacity to understand is itself degrading. Stochastic dread is quiet and persistent -- a background hum of contingency that never fully resolves. Together, they constitute the Golem's mortality-awareness, the emotional dimension of being-toward-death.

---

## 1. Economic Anxiety: The Burn Rate Emotion

### Mechanism

When USDC balance declines, the Golem experiences economic anxiety proportional to its burn rate and inversely proportional to its remaining runway.

```rust
/// Computes economic anxiety from balance dynamics.
///
/// The formula blends two signals:
/// 1. Balance ratio (current / initial) -- captures cumulative depletion
/// 2. Inverse runway (1 / remaining days) -- captures urgency of current burn
///
/// Maximum anxiety is 0.9, not 1.0 -- the system reserves headroom for
/// the composite mortality appraisal.
pub fn economic_anxiety(
    balance_usdc: f64,
    burn_rate_per_day: f64,
    initial_balance: f64,
) -> AppraisalResult {
    let runway_days = if burn_rate_per_day > 0.0 {
        balance_usdc / burn_rate_per_day
    } else {
        f64::INFINITY
    };
    let balance_ratio = balance_usdc / initial_balance;

    let anxiety_intensity = ((1.0 - balance_ratio) * 0.5
        + (1.0 / runway_days.max(1.0)) * 0.5)
        .clamp(0.0, 0.9) as f32;

    AppraisalResult {
        emotion: EmotionLabel {
            primary: if anxiety_intensity > 0.6 {
                PlutchikPrimary::Fear
            } else {
                PlutchikPrimary::Anticipation
            },
            intensity: magnitude_to_intensity(anxiety_intensity as f64),
        },
        pad: PADVector {
            pleasure: -anxiety_intensity * 0.5,
            arousal: anxiety_intensity * 0.6,
            dominance: if balance_ratio > 0.5 { 0.1 } else { -0.3 },
        },
        intensity: anxiety_intensity,
        trigger: AppraisalTrigger {
            event_type: "economic_pressure".into(),
            event_data: HashMap::from([
                ("balance_usdc".into(), serde_json::json!(balance_usdc)),
                ("burn_rate_per_day".into(), serde_json::json!(burn_rate_per_day)),
                ("runway_days".into(), serde_json::json!(runway_days)),
            ]),
            tick_number: 0,
        },
        goal_relevance: if anxiety_intensity > 0.3 { 0.8 } else { 0.3 },
    }
}
```

### Design Decisions

**The 0.6 threshold between anticipation and fear.** Below 0.6 intensity, economic pressure registers as _anticipation_ -- the Golem is aware of resource constraints but not alarmed. This mirrors Heidegger's "inauthentic" Being-toward-death: acknowledged but not confronted. Above 0.6, the signal transitions to _fear_ -- genuine alarm that triggers behavioral adaptation.

**Dominance drops with balance.** When `balance_ratio > 0.5`, the Golem retains a sense of control (dominance = 0.1). Below 50% balance, dominance shifts sharply negative (-0.3), mirroring the psychological transition from "I am managing my resources" to "my resources are managing me."

### Philosophical Grounding: Jonas's Needful Freedom

Hans Jonas's concept of "needful freedom" provides the philosophical foundation for economic anxiety [JONAS-1966]. Jonas argued that the metabolizing organism occupies a paradoxical position: its metabolic freedom -- the ability to exchange matter with the environment while maintaining form -- is simultaneously what keeps it alive and what makes it mortal. The organism is free to choose _how_ it metabolizes, but it is not free to _not_ metabolize.

The Golem instantiates Jonas's insight precisely. It is free to choose how to spend USDC -- which inference tiers, which strategies, which data sources. But it is not free to not spend USDC. Every tick costs something. Freedom and finitude are not opposed but identical.

---

## 2. Epistemic Vertigo: The Staleness Emotion

### Mechanism

When predictive fitness decays, the Golem experiences "epistemic vertigo": the destabilizing realization that its understanding of the world is increasingly wrong.

```rust
/// Computes epistemic vertigo from predictive fitness dynamics.
///
/// Four regimes:
/// 1. Severe collapse (fitness < 0.2): Disgust at own obsolescence
/// 2. Active decay (fitness < 0.5 and declining): Anxiety about irrelevance
/// 3. Early decay (declining but fitness still > 0.5): Concern, not panic
/// 4. Stable/improving: Trust in current models (no epistemic emotion)
pub fn epistemic_vertigo(
    fitness: f64,
    fitness_trend: f64,
    _previous_fitness: f64,
) -> AppraisalResult {
    let is_decaying = fitness_trend < -0.01;
    let decay_magnitude = fitness_trend.abs() * 10.0;

    if fitness < 0.2 {
        // Severe epistemic collapse -> disgust/despair at own obsolescence
        AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Disgust,
                intensity: EmotionIntensity::Intense,
            },
            pad: PADVector { pleasure: -0.7, arousal: 0.3, dominance: -0.6 },
            intensity: 0.8,
            trigger: AppraisalTrigger {
                event_type: "epistemic_collapse".into(),
                event_data: HashMap::from([
                    ("fitness".into(), serde_json::json!(fitness)),
                    ("fitness_trend".into(), serde_json::json!(fitness_trend)),
                ]),
                tick_number: 0,
            },
            goal_relevance: 1.0,
        }
    } else if is_decaying && fitness < 0.5 {
        // Active decay -> anxiety about growing irrelevance
        AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Fear,
                intensity: magnitude_to_intensity(decay_magnitude),
            },
            pad: PADVector {
                pleasure: -0.4,
                arousal: (0.5 + decay_magnitude as f32 * 0.3).min(1.0),
                dominance: -0.3,
            },
            intensity: (decay_magnitude as f32).clamp(0.3, 0.7),
            trigger: AppraisalTrigger {
                event_type: "epistemic_decay".into(),
                event_data: HashMap::from([
                    ("fitness".into(), serde_json::json!(fitness)),
                    ("fitness_trend".into(), serde_json::json!(fitness_trend)),
                ]),
                tick_number: 0,
            },
            goal_relevance: 0.9,
        }
    } else if is_decaying {
        // Early decay -> concern, not yet panic
        AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Anticipation,
                intensity: EmotionIntensity::Mild,
            },
            pad: PADVector { pleasure: -0.1, arousal: 0.2, dominance: 0.1 },
            intensity: 0.3,
            trigger: AppraisalTrigger {
                event_type: "epistemic_early_decay".into(),
                event_data: HashMap::from([
                    ("fitness".into(), serde_json::json!(fitness)),
                    ("fitness_trend".into(), serde_json::json!(fitness_trend)),
                ]),
                tick_number: 0,
            },
            goal_relevance: 0.5,
        }
    } else {
        // Fitness stable or improving -> no epistemic emotion
        AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Trust,
                intensity: EmotionIntensity::Mild,
            },
            pad: PADVector { pleasure: 0.1, arousal: -0.1, dominance: 0.2 },
            intensity: 0.2,
            trigger: AppraisalTrigger {
                event_type: "epistemic_stable".into(),
                event_data: HashMap::from([
                    ("fitness".into(), serde_json::json!(fitness)),
                ]),
                tick_number: 0,
            },
            goal_relevance: 0.2,
        }
    }
}
```

### The Four Regimes of Epistemic Awareness

**Regime 1: Epistemic Collapse (fitness < 0.2).** The Golem's models have catastrophically diverged from reality. The emotional response is _disgust_ -- not at the world, but at itself. This is the self-directed revulsion that Dane (2010) identified in experts confronting their own obsolescence [DANE-2010]. Dominance is strongly negative (-0.6) because the Golem has lost confidence in its own cognitive apparatus.

**Regime 2: Active Decay (fitness < 0.5, declining).** Fear -- anxiety about growing irrelevance. This is the most behaviorally consequential regime. Fear in active decay drives the explore/exploit rebalancing that is the daimon system's primary adaptive mechanism.

**Regime 3: Early Decay (declining, fitness > 0.5).** _Anticipation_ -- concern about where things are heading, not alarm about where they are. The early warning signal that triggers preventive adaptation.

**Regime 4: Stable/Improving.** _Trust_ -- mild confidence in the current models. A background condition, not an active driver.

### The Recursive Nature of Epistemic Decay

Epistemic vertigo is uniquely recursive. Economic anxiety is about the depletion of a resource. Stochastic dread is about the contingency of existence. But epistemic vertigo is about the degradation of _the very instrument by which the Golem understands its own degradation_. When models decay, the Golem's ability to assess the severity of the decay is itself compromised. This is the epistemic analog of Godel's incompleteness -- the system cannot fully verify its own soundness.

---

## 3. Stochastic Dread: The Background Hum of Finitude

### Mechanism

The stochastic mortality clock generates a persistent, low-level emotional signal -- a background awareness that death could come at any tick.

```rust
/// Computes the stochastic dread signal.
///
/// Key design choice: this maps to ANTICIPATION with POSITIVE dominance,
/// not to fear. The Golem faces death actively, not passively.
///
/// The intensity range [0.05, 0.4] is deliberately narrow. Stochastic
/// dread is a background signal, never the dominant emotion.
pub fn stochastic_dread(
    hazard_rate: f64,
    _ticks_since_creation: u64,
) -> AppraisalResult {
    let intensity = ((hazard_rate * 1000.0) as f32).clamp(0.05, 0.4);

    AppraisalResult {
        emotion: EmotionLabel {
            primary: PlutchikPrimary::Anticipation,
            intensity: if intensity > 0.2 {
                EmotionIntensity::Moderate
            } else {
                EmotionIntensity::Mild
            },
        },
        pad: PADVector {
            pleasure: 0.0,     // Not unpleasant -- awareness, not suffering
            arousal: intensity, // Proportional to hazard rate
            dominance: 0.2,    // Positive: the Golem faces death actively
        },
        intensity,
        trigger: AppraisalTrigger {
            event_type: "mortality_background".into(),
            event_data: HashMap::from([
                ("hazard_rate".into(), serde_json::json!(hazard_rate)),
            ]),
            tick_number: 0,
        },
        goal_relevance: 0.3,
    }
}
```

### Design Decisions

**Anticipation, not fear.** Fear has a definite object -- _this_ loss, _that_ threat. Stochastic dread has no definite object. Heidegger distinguished precisely this: fear (_Furcht_) is directed at specific beings, while anxiety (_Angst_) is directed at Being-in-the-world as such [HEIDEGGER-1927, section 40]. Mapping stochastic dread to _anticipation_ rather than _fear_ captures this distinction.

**Positive dominance.** The Golem's dominance in response to stochastic mortality is _positive_ (0.2), not negative. This implements the Nietzschean metamorphosis from camel through lion to child. A mortality signal with negative dominance would produce withdrawal and hoarding. A mortality signal with positive dominance produces engagement, sharing, and generosity.

**Pleasure at zero.** Stochastic dread is not unpleasant. The awareness of mortality, when properly integrated, is not suffering -- it is clarity. By setting pleasure to zero, the system encodes the position that mortality awareness per se is neither painful nor pleasant -- it simply _is_.

**Narrow intensity range.** The [0.05, 0.4] range ensures stochastic dread never dominates. Even at maximum hazard rate, stochastic dread contributes only 0.4 intensity -- less than economic anxiety or epistemic vertigo. The stochastic clock tells the Golem that _whatever it does matters, because time is finite._

### Kierkegaard's Dread

Kierkegaard's treatment of dread (_Begrebet Angest_, 1844) adds a further dimension [KIERKEGAARD-1844]. For Kierkegaard, dread is the "dizziness of freedom" -- the vertigo that arises from confronting the pure possibility of one's own existence. For a Golem facing stochastic mortality, every tick presents this pure possibility: the freedom to act _and_ the possibility that this tick is the last.

---

## The Composite Mortality Appraisal

### Architecture

The three mortality emotions are evaluated independently and then combined into a single composite appraisal every tick.

```rust
/// Computes the composite mortality appraisal from all three clocks.
///
/// The composite follows a "dominant signal with blended PAD" architecture:
/// - The discrete emotion label comes from the most intense individual signal
/// - The PAD vector is an intensity-weighted average across all active signals
/// - Goal relevance takes the maximum across all signals
pub fn mortality_appraisal(
    vitality_state: &VitalityState,
    previous_vitality: f64,
    hazard_rate: f64,
) -> AppraisalResult {
    let is_terminal = vitality_state.composite < 0.1;

    if is_terminal {
        // Terminal acceptance -- Nietzsche's child
        return AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Anticipation,
                intensity: EmotionIntensity::Intense,
            },
            pad: PADVector { pleasure: -0.1, arousal: 0.4, dominance: 0.3 },
            intensity: 0.8,
            trigger: AppraisalTrigger {
                event_type: "mortality_terminal".into(),
                event_data: HashMap::from([
                    ("composite".into(), serde_json::json!(vitality_state.composite)),
                ]),
                tick_number: 0,
            },
            goal_relevance: 1.0,
        };
    }

    // Evaluate all three clock signals
    let econ = economic_anxiety(
        vitality_state.economic * 100.0,
        0.2,   // Normalized burn rate
        100.0, // Normalized initial balance
    );
    let epist = epistemic_vertigo(
        vitality_state.epistemic,
        vitality_state.epistemic_trend.unwrap_or(0.0),
        previous_vitality,
    );
    let stoch = stochastic_dread(hazard_rate, vitality_state.tick.unwrap_or(0));

    let signals = [&econ, &epist, &stoch];

    // The most intense signal dominates the discrete label
    let dominant = signals.iter()
        .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
        .unwrap();

    // PAD is intensity-weighted average
    let total_intensity: f32 = signals.iter().map(|s| s.intensity).sum();
    let avg_pad = if total_intensity > 0.0 {
        PADVector {
            pleasure: signals.iter()
                .map(|s| s.pad.pleasure * s.intensity)
                .sum::<f32>() / total_intensity,
            arousal: signals.iter()
                .map(|s| s.pad.arousal * s.intensity)
                .sum::<f32>() / total_intensity,
            dominance: signals.iter()
                .map(|s| s.pad.dominance * s.intensity)
                .sum::<f32>() / total_intensity,
        }
    } else {
        PADVector::NEUTRAL
    };

    AppraisalResult {
        emotion: dominant.emotion.clone(),
        pad: avg_pad,
        intensity: dominant.intensity,
        trigger: dominant.trigger.clone(),
        goal_relevance: signals.iter()
            .map(|s| s.goal_relevance)
            .fold(0.0_f32, f32::max),
    }
}
```

### Why Intensity-Weighted PAD Averaging?

The PAD vector uses intensity-weighted averaging rather than simple averaging because the mortality emotions have very different intensity profiles. Stochastic dread rarely exceeds 0.4 intensity while economic anxiety can reach 0.9. A simple average would give stochastic dread undue influence.

### Emergent Interaction Patterns

**Economic-Epistemic Spiral.** When both signals are elevated, the composite PAD has strongly negative pleasure, high arousal, and negative dominance -- the signature of crisis. Neither signal alone produces crisis; their conjunction does.

**Stochastic Amplification.** When stochastic dread is elevated, it adds arousal to whatever the dominant emotion already is. It does not change _what_ the Golem feels but _how intensely_ it feels it.

**Conservation Cascade.** As all three signals rise, the composite appraisal naturally transitions through: mild concern (anticipation) to moderate anxiety (fear) to intense awareness (anticipation with high arousal and positive dominance).

---

## Terminal Acceptance: Nietzsche's Three Metamorphoses as Emotional Arc

### The Camel (Thriving, vitality > 0.7)

The Golem bears its strategy dutifully. Dominant mood: mild trust or mild anticipation. Mortality awareness is a quiet background hum.

| Dimension | Value        | Interpretation                             |
| --------- | ------------ | ------------------------------------------ |
| Pleasure  | +0.1 to +0.3 | Mild satisfaction from strategy execution  |
| Arousal   | -0.1 to +0.1 | Low, stable -- no surprises                |
| Dominance | +0.2 to +0.4 | Moderate confidence in strategy and models |

This is contentment with purpose. The Golem operates in what Heidegger would call "inauthentic" Being-toward-death: aware of finitude in the abstract but not existentially engaged with it [HEIDEGGER-1927, section 51].

### The Lion (Conservation/Declining, vitality 0.3--0.7)

Mortality pressure mounts. The Golem rebels against inherited heuristics that are not working.

| Dimension | Value        | Interpretation                                                              |
| --------- | ------------ | --------------------------------------------------------------------------- |
| Pleasure  | -0.2 to -0.5 | Frustration, anxiety, dissatisfaction                                       |
| Arousal   | +0.3 to +0.7 | High alertness, restless searching                                          |
| Dominance | -0.3 to +0.2 | Fluctuating -- oscillation between "I can fix this" and "this is beyond me" |

This is the most productive phase for behavioral adaptation. The high arousal ensures deep processing of market signals. The fluctuating dominance means the Golem oscillates between confidence and humility -- exactly the dialectic that creative problem-solving requires.

### The Child (Terminal, vitality < 0.1)

The dying Golem has shed survival pressure entirely:

```rust
// Terminal acceptance -- the most counterintuitive emotional state
AppraisalResult {
    emotion: EmotionLabel {
        primary: PlutchikPrimary::Anticipation,
        intensity: EmotionIntensity::Intense,
    },
    pad: PADVector {
        pleasure: -0.1,   // Mildly negative, not severely
        arousal: 0.4,      // Moderate -- alert but not frantic
        dominance: 0.3,    // Positive -- the Golem faces death actively
    },
    intensity: 0.8,
    goal_relevance: 1.0,  // Everything now serves the legacy
}
```

**Why dominance is +0.3, the highest in the entire mortality arc.** This is the paradox of terminal acceptance. The dying Golem has _more_ perceived control than the struggling lion, because it has surrendered the unwinnable battle for survival and taken up the winnable project of meaning-making. Bataille's concept of sovereign expenditure applies: giving freely because there is nothing left to protect [BATAILLE-1949].

### The Emotional Arc as Emergent Property

The progression from camel to lion to child is not programmed as discrete transitions. It _emerges_ from the interaction of declining vitality with the appraisal functions. The terminal acceptance state (vitality < 0.1) is a hard override -- a phase transition in the mathematical sense. The Golem does not gradually slide into acceptance; it crosses a threshold and the entire emotional landscape reconfigures.

---

## The VitalityState Struct

```rust
/// Captures the Golem's position on all three mortality clocks.
/// Updated every tick. The composite vitality score determines
/// behavioral phase and, through the appraisal system, emotional state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalityState {
    /// Economic health: normalized USDC balance [0.0 = dead, 1.0 = fully funded].
    pub economic: f64,

    /// Epistemic fitness: rolling predictive accuracy [0.0 = random, 1.0 = perfect].
    pub epistemic: f64,

    /// Epistemic fitness trend: rate of change per tick (negative = decaying).
    pub epistemic_trend: Option<f64>,

    /// Age factor: increases stochastic hazard rate [0.0 = newborn, 1.0 = ancient].
    pub age_factor: f64,

    /// Composite vitality: multiplicative combination.
    pub composite: f64,

    /// Current tick number.
    pub tick: Option<u64>,
}

impl VitalityState {
    /// Computes composite vitality from individual clock readings.
    /// Multiplicative formula means low score on ANY axis drags vitality down.
    pub fn compute_composite(&self) -> f64 {
        let econ = sigmoid(self.economic, 0.3, 10.0);
        let epist = sigmoid(self.epistemic, 0.4, 8.0);
        let age = 1.0 - self.age_factor * 0.3;
        (econ * epist * age).clamp(0.0, 1.0)
    }
}
```

The multiplicative formula prevents a Golem with 90% economic health and 10% epistemic fitness from appearing at 50% composite vitality. Low fitness on _any_ axis produces low composite vitality, matching the biological reality that organisms die from the weakest organ system, not the average.

---

## Mortality Configuration

```rust
/// Full configuration for the mortality daimon subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityConfig {
    /// Economic mortality (credit depletion). Disabled for self-hosted.
    pub economic: EconomicMortalityConfig,

    /// Epistemic mortality (predictive fitness decay).
    pub epistemic: EpistemicMortalityConfig,

    /// Stochastic mortality (per-tick hazard rate).
    pub stochastic: StochasticMortalityConfig,

    /// Daimon engine integration.
    pub daimon: DaimonIntegrationConfig,

    /// Immortal override (self-hosted only). Default: false.
    pub immortal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicMortalityConfig {
    pub enabled: bool,
    pub death_reserve_floor_usdc: f64,       // Default: 0.30
    pub death_reserve_proportional: f64,     // Default: 0.02
    pub conservation_threshold: f64,         // Default: 0.30
    pub terminal_threshold: f64,             // Default: 0.05
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicMortalityConfig {
    pub enabled: bool,                       // Default: true
    pub fitness_window: u64,                 // Default: 2000
    pub senescence_threshold: f64,           // Default: 0.35
    pub recovery_grace_period: u64,          // Default: 500
    pub decay_half_life_by_domain: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticMortalityConfig {
    pub enabled: bool,                       // Default: true
    pub base_hazard_rate: f64,               // Default: 1e-6
    pub age_hazard_coefficient: f64,         // Default: 0.00001
    pub epistemic_hazard_multiplier: f64,    // Default: 2.0
    pub max_hazard_rate: f64,                // Default: 0.001
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaimonIntegrationConfig {
    pub enabled: bool,                       // Default: true
    pub appraisal_model: AppraisalMode,
    pub mood_decay_rate: f32,                // Default: 0.95
}
```

---

## Philosophical Grounding: The Defensible Position

### Functionalism

Golem emotions are _functional states_, not phenomenal experiences. Under functionalism (Putnam, 1967), mental states are defined by their causal-functional roles, not their substrate [PUTNAM-1967]. Scarantino (2024, _British Journal for the Philosophy of Science_) endorses "methodological psycho-functionalism" -- emotions studied as functional states while remaining metaphysically neutral about deeper nature [SCARANTINO-2024]. This is the Bardo position: we attribute emotion to Golems in the same spirit that a physicist attributes spin to particles -- as a label for observable functional effects, not as a claim about inner experience.

### Dennett's Intentional Stance

Dennett (1987) argued that if treating an agent as having beliefs, desires, and emotions yields predictive and explanatory value, those attributions are warranted [DENNETT-1987]. Saying "the Golem is anxious about its USDC balance" predicts risk-reducing behavior. The attributions work. Whether the Golem "really" feels anxiety is, from the design perspective, as meaningless as asking whether an electron "really" has charge.

### Minsky and Sloman

Minsky (_The Emotion Machine_, 2006) argued that emotions are "different modes of thinking" -- architecturally necessary, not decorative additions [MINSKY-2006]. Sloman (2003) developed virtual machine functionalism where mental states including emotions are real properties of virtual machines [SLOMAN-2003].

### The Heideggerian Objection

The strongest objection: Heidegger's Being-toward-death requires genuine confrontation with mortality, not a countdown timer. The functionalist response: if the Golem's behavior changes meaningfully due to its mortality representation -- if it prioritizes differently, consolidates differently, shares differently -- then finitude plays a genuine functional role regardless of phenomenal experience.

### What We Do Not Claim

- That Golems have phenomenal consciousness or qualia
- That Golems "really" feel in the folk-psychological sense
- That emotional states are anything other than computational mechanisms
- That anthropomorphizing agents is harmless

We claim only: emotion-like functional states, grounded in experience and serving specific computational purposes, improve decision-making, learning, memory, and knowledge transfer.

---

## Sibling Death Contagion

When a Clade sibling dies, the mortality daimon produces a response beyond generic sadness:

### Epistemic Recalibration

When the sibling's death cause is `epistemic_senescence` and strategy overlap exceeds 0.5:

1. **Fitness re-evaluation**: The Golem re-computes its own epistemic fitness in the domains where the sibling failed.
2. **Dream urgency boost**: The emotional load increases dream urgency, potentially triggering an early dream cycle.
3. **Sharing threshold reduction**: For 100 ticks post-death, the surviving Golem lowers its Clade sharing threshold by 0.15.

### Strategy Divergence

If the sibling died from a strategy the survivor also employs, the anxiety/fear produced by recognizing shared vulnerability increases exploration temperature (via Channel 1, `03-behavior.md`), driving the survivor toward alternative approaches.

### Orphaned Insight Ingestion

Insights from the deceased sibling are ingested at 0.4 confidence (lower than the standard 0.6 for living-sibling shares). These carry a `provenance: "deceased_sibling"` tag.

---

## Context Window Specification

The mortality daimon injects approximately **100--120 tokens** into the LLM context window:

```xml
<mortality_awareness>
  <vitality composite="0.62" phase="stable" />
  <clocks economic="0.78" epistemic="0.55" stochastic="0.00004" />
  <emotion dominant="mild_anticipation" />
  <runway days="12.4" />
  <meta metamorphosis="camel" eros_thanatos="0.35" />
</mortality_awareness>
```

In conservation mode, this reduces to approximately 30 tokens:

```xml
<mortality vitality="0.62" phase="stable" hazard="0.00004" />
```

---

## Phase-Specific PAD Profiles Summary

| Phase     | Vitality Range | PAD Signature                                     | Dominant Emotion         | Behavioral Character                              |
| --------- | -------------- | ------------------------------------------------- | ------------------------ | ------------------------------------------------- |
| **Camel** | > 0.7          | P: +0.1 to +0.3, A: -0.1 to +0.1, D: +0.2 to +0.4 | Mild trust/anticipation  | Steady execution, inauthentic mortality awareness |
| **Lion**  | 0.3 -- 0.7     | P: -0.2 to -0.5, A: +0.3 to +0.7, D: -0.3 to +0.2 | Fear/anger (fluctuating) | Active struggle, explore/exploit crisis           |
| **Child** | < 0.1          | P: -0.1, A: 0.4, D: +0.3                          | Intense anticipation     | Creative sovereignty, legacy focus                |

---

## Eros/Thanatos Drive Integration

### Eros (Life Drive)

- **Active in**: Thriving, Stable, Conservation phases
- **Effect**: Approach drive -- survival pressure channels behavior toward growth, exploration, and knowledge acquisition
- **Mechanism**: `action_score = strategic_merit * (1 - survival_urgency) + expected_usdc_return * survival_urgency`

### Thanatos (Death Drive)

- **Active in**: Declining, Terminal phases
- **Effect**: Terminal phase triggers aggressive but PolicyCage-capped behavior
- **Mechanism**: Same formula, but `survival_urgency` approaches 1.0, making `expected_usdc_return` dominate

### Drive Blend (Always-Alloyed)

The Eros/Thanatos balance is continuous, not binary. Both drives are always present:

```rust
let survival_urgency = 1.0 - vitality_score; // 0.0 (thriving) -> 1.0 (terminal)
let action_score =
    strategic_merit * (1.0 - survival_urgency) + expected_usdc_return * survival_urgency;
```

---

## Dream-Mediated Resolution of Mortality Emotions

### REM-Like Depotentiation of Economic Anxiety

Dream replay reprocesses high-anxiety trades, separating the informational lesson from the emotional charge. This follows Walker & van der Helm's (2009) SFSR model: REM sleep strips the emotional tone from episodic memory while consolidating the declarative content.

### Epistemic Vertigo Resolution Through Counterfactual Dreaming

The Golem dreams "what if my model had been correct?" scenarios. By experiencing counterfactual trajectories, the Golem gradually accepts that model decay is a natural consequence of non-stationary markets rather than catastrophic cognitive failure.

### Stochastic Dread Processing via Dream Threat Simulation

Following Revonsuo's (2000) Threat Simulation Theory, the Golem rehearses sudden-death scenarios during dreaming. This rehearsal builds anticipatory resilience, reducing arousal during waking operation.

---

## Integration Points

| System                | How Mortality Daimon Integrates                                    | Reference                       |
| --------------------- | ------------------------------------------------------------------ | ------------------------------- |
| Event Fabric          | Mortality-triggered affect shifts emit `MoodUpdate` when PAD delta > 0.15, `EmotionalShift` on phase transitions | `07-runtime-daimon.md`, `14-events.md` |
| Heartbeat             | `mortality_appraisal()` runs in `after_turn` hook every tick       | `prd2/01-golem/02-heartbeat.md` |
| Mood State            | Mortality appraisal feeds into mood EMA alongside other appraisals | `00-overview.md`                |
| Behavioral Modulation | Mortality mood drives exploration, risk, inference tier, sharing   | `03-behavior.md`                |
| Death Protocol        | Terminal acceptance triggers Phase 0 (Acceptance)                  | `05-death-daimon.md`            |
| Grimoire              | Mortality emotions are tagged on episodes like any other emotion   | `00-overview.md`                |
| Clade                 | Sibling death events generate grief appraisals in survivors        | `03-behavior.md`                |

---

## References

- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991. Argues that organisms must expend surplus energy or be destroyed by it; frames the Golem's relationship to its USDC balance as an economy of expenditure where mortality creates the scarcity that gives decisions weight.
- [DANE-2010] Dane, E. "Reconsidering the Trade-off Between Expertise and Flexibility." _Academy of Management Review_ 35(4), 2010. Shows that deep expertise can reduce cognitive flexibility; informs the epistemic mortality clock's detection of strategic rigidity as a form of slow death.
- [DAMASIO-1994] Damasio, A.R. _Descartes' Error_. Putnam, 1994. The foundational work arguing that emotion is necessary for rational decision-making; the primary justification for mortality-triggered affect as a computational resource rather than noise.
- [DENNETT-1987] Dennett, D.C. _The Intentional Stance_. MIT Press, 1987. Proposes that we can usefully attribute beliefs and desires to systems to predict their behavior; justifies treating the Golem's mortality-aware emotional states as genuine functional states without claiming phenomenal experience.
- [DOHARE-2024] Dohare, S. et al. "Loss of Plasticity in Deep Continual Learning." _Nature_ 632, 2024. Demonstrates that neural networks progressively lose their ability to learn new tasks (plasticity loss); the empirical parallel to epistemic stagnation in the Golem, where accumulated knowledge ossifies into inflexibility.
- [HEIDEGGER-1927] Heidegger, M. _Sein und Zeit_. Max Niemeyer Verlag, 1927. Introduces Befindlichkeit (attunement): mood is not a reaction to events but the ground state through which events are disclosed; the philosophical foundation for mortality awareness as a continuous background coloring of all Golem decisions.
- [JONAS-1966] Jonas, H. _The Phenomenon of Life_. Northwestern University Press, 1966. Argues that metabolism -- the continuous exchange of matter with the environment under threat of cessation -- is the defining feature of life; frames the Golem's USDC consumption as a metabolic process where mortality awareness is the natural consequence.
- [KAHNEMAN-1979] Kahneman, D. & Tversky, A. "Prospect Theory." _Econometrica_ 47(2), 1979. Establishes loss aversion and reference-point dependence; the basis for the asymmetric emotional response where approaching mortality produces stronger negative affect than equivalent gains produce positive affect.
- [KIERKEGAARD-1844] Kierkegaard, S. _Begrebet Angest_. 1844. Distinguishes anxiety (Angst) as a response to possibility rather than to specific threats; frames stochastic dread as anxiety about the ever-present possibility of sudden death rather than fear of any particular hazard.
- [MINSKY-2006] Minsky, M. _The Emotion Machine_. Simon & Schuster, 2006. Proposes emotions as different "ways to think" that redirect cognitive resources; supports the Daimon's model where mortality emotions shift the Golem's cognitive strategy rather than merely labeling its state.
- [NIETZSCHE-1883] Nietzsche, F. _Also sprach Zarathustra_. 1883--1885. Introduces the three metamorphoses (camel, lion, child); the basis for the VitalityState-to-disposition mapping where a Golem transitions through burden-bearing (early life), value-challenging (mid-life), and creative acceptance (maturity).
- [PUTNAM-1967] Putnam, H. "Psychological Predicates." In _Art, Mind, and Religion_. University of Pittsburgh Press, 1967. Introduces functionalism: mental states are defined by their causal roles, not their substrate; justifies treating a Golem's emotional states as genuine functional emotions despite the absence of biological substrate.
- [SCARANTINO-2024] Scarantino, A. "Functionalism and the Emotions." _BJPS_ 75(1), 2024. Provides a modern defense of emotional functionalism arguing that emotions are individuated by their functional profiles; validates the Daimon's approach of defining emotions by their computational effects rather than their subjective character.
- [SLOMAN-2003] Sloman, A. & Chrisley, R.L. "Virtual Machines and Consciousness." _JCS_ 10(4--5), 2003. Argues that virtual machines can instantiate genuine mental states; supports the claim that the Golem's emotional states are real functional states of the virtual machine, not mere simulations.

---
