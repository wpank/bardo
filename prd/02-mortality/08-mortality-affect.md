# How Mortality Shapes Emotions: The Death Affect [SPEC]

**Version:** 4.0 | **Status:** Draft | **Last Updated:** 2026-03-14

> _"Angst discloses Dasein's Being-toward-death."_
> -- Martin Heidegger, _Being and Time_ (1927), Sect.53

**Crates:** `golem-daimon`, `golem-mortality`

**Depends on:** `../03-daimon/04-daimon-overview.md` (Daimon engine architecture and appraisal model), `00-thesis.md` (foundational mortality thesis), `02-epistemic-decay.md` (epistemic clock specification), `03-stochastic-mortality.md` (per-tick hazard rate, game-theoretic foundations), `06-thanatopsis.md` (four-phase death protocol), `07-succession.md` (generational evolution, owner-initiated rebirth)

---

> **Reader orientation:** This document specifies how mortality shapes Golem (mortal autonomous DeFi agent) emotions. The Daimon (affect engine implementing PAD emotional state) produces three mortality-specific emotions: mortality salience (background awareness of finitude), epistemic anxiety (feeling one's world-model going stale), and terminal acceptance (the letting-go at death). These are functional states that modulate behavior, not claims about consciousness. If you are new to the Daimon, read `../03-daimon/04-daimon-overview.md` (Daimon engine architecture and appraisal model) first. See `prd2/shared/glossary.md` for full term definitions.

> **See also:** `../03-daimon/07-mortality-daimon.md` (the Daimon system's internal perspective on mortality emotions) for the Daimon system's perspective on mortality emotions.

## Overview

No existing emotion framework -- OCC, ALMA, EMA, FAtiMA, Chain-of-Emotion -- addresses how agents feel about their own mortality. They model reactions to external events (a trade profit, a market crash, a social interaction) but not the **background awareness of finitude** that colors every decision a mortal agent makes.

This document specifies **mortality-aware affect**: emotional states that arise specifically from the Golem's relationship to its own death. These states are unique to mortal agent design and have no precedent in the computational emotion literature. They serve three architectural functions:

1. **Decision modulation.** Mortality awareness shifts the exploration/exploitation balance, risk tolerance, and knowledge-sharing behavior in ways that improve aggregate outcomes.
2. **Memory salience.** Mortality-related emotions tag Grimoire entries with urgency signals that improve retrieval quality for successors.
3. **Behavioral coherence.** The emotional arc from healthy operation through decline to terminal acceptance produces a legible behavioral trajectory that owners can monitor and that death testaments can narrate.

### Disclaimers

Golem emotions are functional states -- computational mechanisms that influence behavior. No claims about phenomenal consciousness, qualia, or subjective experience. "Feels" is used in the functionalist sense (causal-functional role). The purpose is better decisions, not anthropomorphization.

---

## Philosophical Grounding

> **Extended:** Functionalism (Putnam, Scarantino), Dennett's intentional stance, Minsky/Sloman emotions-as-architecture, Heidegger's Befindlichkeit (mood as ground state), Jonas's needful freedom -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

Golem emotions are functional states (not phenomenal experiences), justified by the intentional stance and implemented as architectural modes. Mortality awareness is a persistent attunement (Heidegger's _Befindlichkeit_), evaluated every tick and injected as context before strategic reasoning.

---

## The Three Mortality Emotions

Each of the three mortality clocks (economic, epistemic, stochastic) generates a distinct emotional signature. These are not arbitrary mappings but grounded in the specific relationship between each clock and the Golem's agency.

### 1. Economic Anxiety: The Burn Rate Emotion

**Clock:** Economic (USDC balance / burn rate)

**Philosophical root:** Jonas's needful freedom -- the tension between metabolic freedom and metabolic necessity.

**Character:** A compound emotion mixing fear (the threat of resource exhaustion) with anticipation (the awareness that remaining resources create remaining possibilities). Dominance is modulated by the Golem's perceived control over its burn rate -- a Golem that can reduce costs feels less anxious than one locked into high-cost inference patterns.

**Behavioral effects:**

- As anxiety rises, risk tolerance decreases (smaller position sizes, more conservative strategies)
- At moderate levels, anxiety drives efficiency improvements (cheaper inference paths, reduced tick frequency)
- At high levels, anxiety triggers conservation mode (minimal activity, focus on preserving remaining resources)
- At extreme levels, anxiety transitions to terminal acceptance (see below)

```rust
use crate::daimon::{AppraisalResult, EmotionLabel, PADVector};

/// Compute economic anxiety from USDC balance and burn rate.
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
    let balance_ratio = if initial_balance > 0.0 {
        balance_usdc / initial_balance
    } else {
        1.0
    };

    // Anxiety intensity blends two signals:
    // 1. How much of the initial balance has been consumed (long-term pressure)
    // 2. How many days of runway remain (immediate pressure)
    let consumption_pressure = (1.0 - balance_ratio) * 0.5;
    let runway_pressure = if runway_days.is_finite() {
        (1.0 / runway_days.max(0.1)) * 0.5
    } else {
        0.0
    };

    let anxiety_intensity = (consumption_pressure + runway_pressure).clamp(0.0, 0.9);

    // High anxiety -> fear (active threat); low anxiety -> anticipation (background awareness)
    let primary_emotion = if anxiety_intensity > 0.6 {
        EmotionLabel::Fear
    } else {
        EmotionLabel::Anticipation
    };

    // Dominance tracks perceived control
    let dominance = if balance_ratio > 0.5 {
        0.1
    } else {
        -0.3 - (0.5 - balance_ratio)
    };

    AppraisalResult {
        emotion: primary_emotion,
        intensity: anxiety_intensity,
        pad: PADVector {
            pleasure: -anxiety_intensity * 0.5,
            arousal: anxiety_intensity * 0.6,
            dominance: dominance.clamp(-0.8, 0.3),
        },
        trigger: AppraisalTrigger {
            event_type: "economic_pressure".into(),
            event_data: serde_json::json!({
                "balance_usdc": balance_usdc,
                "burn_rate_per_day": burn_rate_per_day,
                "runway_days": runway_days,
                "balance_ratio": balance_ratio,
            }),
            tick_number: 0, // Filled by caller
        },
        goal_relevance: if anxiety_intensity > 0.3 { 0.8 } else { 0.3 },
    }
}
```

### 2. Epistemic Vertigo: The Staleness Emotion

**Clock:** Epistemic (predictive fitness score, rolling window of prediction-outcome accuracy)

**Character:** Fear (models failing) + disgust (obsolescence) + paradoxical anticipation (successor may see what I cannot). Distinct from economic anxiety: the problem is lack of understanding, not resources. The Golem has USDC to act but no longer knows what to do.

**Behavioral effects:**

- As fitness decays, the Golem increases exploration temperature (trying new strategies, testing new hypotheses)
- At moderate decay, the Golem seeks Clade siblings' perspectives (increased pull frequency from Clade Grimoire)
- At severe decay, the Golem enters a "crisis of confidence" that disrupts exploitation of current strategies -- forcing adaptation or death
- Epistemic vertigo also modulates knowledge-sharing thresholds: a Golem uncertain about its own models shares more tentatively (lower confidence on Clade push)

```rust
/// Compute epistemic vertigo from predictive fitness score and trend.
pub fn epistemic_vertigo(
    fitness: f64,
    fitness_trend: f64,
    _previous_fitness: f64,
) -> AppraisalResult {
    let is_decaying = fitness_trend < -0.01;
    let decay_magnitude = fitness_trend.min(0.0).abs() * 10.0;
    let fitness_gap = 1.0 - fitness;

    // Severe epistemic collapse: disgust at own obsolescence
    if fitness < 0.2 {
        return AppraisalResult {
            emotion: EmotionLabel::Disgust,
            intensity: 0.8,
            pad: PADVector {
                pleasure: -0.7, // Deep displeasure
                arousal: 0.3,   // Not panicked -- resigned
                dominance: -0.6, // Powerlessness
            },
            trigger: AppraisalTrigger {
                event_type: "epistemic_collapse".into(),
                event_data: serde_json::json!({
                    "fitness": fitness,
                    "fitness_trend": fitness_trend,
                    "fitness_gap": fitness_gap,
                }),
                tick_number: 0,
            },
            goal_relevance: 1.0,
        };
    }

    // Active decay with fitness below midpoint: the Golem is becoming
    // irrelevant and knows it. Fear of growing obsolescence.
    if is_decaying && fitness < 0.5 {
        return AppraisalResult {
            emotion: EmotionLabel::Fear,
            intensity: (decay_magnitude + fitness_gap * 0.3).clamp(0.3, 0.7),
            pad: PADVector {
                pleasure: -0.4,
                arousal: 0.5 + decay_magnitude * 0.3,
                dominance: -0.3,
            },
            trigger: AppraisalTrigger {
                event_type: "epistemic_decay".into(),
                event_data: serde_json::json!({
                    "fitness": fitness,
                    "fitness_trend": fitness_trend,
                    "decay_magnitude": decay_magnitude,
                }),
                tick_number: 0,
            },
            goal_relevance: 0.9,
        };
    }

    // Early decay: fitness still above midpoint but trending down.
    // Concern, not yet panic. Anticipation of potential problems.
    if is_decaying {
        return AppraisalResult {
            emotion: EmotionLabel::Anticipation,
            intensity: decay_magnitude.clamp(0.2, 0.4),
            pad: PADVector {
                pleasure: -0.1,
                arousal: 0.2,
                dominance: 0.1,
            },
            trigger: AppraisalTrigger {
                event_type: "epistemic_early_decay".into(),
                event_data: serde_json::json!({
                    "fitness": fitness,
                    "fitness_trend": fitness_trend,
                }),
                tick_number: 0,
            },
            goal_relevance: 0.5,
        };
    }

    // Fitness stable or improving: no epistemic distress.
    // Mild trust -- confidence in current models.
    AppraisalResult {
        emotion: EmotionLabel::Trust,
        intensity: 0.2,
        pad: PADVector {
            pleasure: 0.1,
            arousal: -0.1,
            dominance: 0.2,
        },
        trigger: AppraisalTrigger {
            event_type: "epistemic_stable".into(),
            event_data: serde_json::json!({
                "fitness": fitness,
                "fitness_trend": fitness_trend,
            }),
            tick_number: 0,
        },
        goal_relevance: 0.2,
    }
}
```

### 3. Stochastic Dread: The Background Hum of Finitude

**Clock:** Stochastic (hazard rate -- per-tick probability of random death)

> **Extended:** Heidegger's Furcht vs Angst distinction -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

**Character:** Not fear (specific threat) but anticipation with positive dominance. Stochastic dread is the computational implementation of Heidegger's _Angst_ -- awareness that death is structurally possible, not fear of any particular threat. The Golem retains agency in its response.

**Behavioral effects:**

- Creates a persistent background pressure toward knowledge sharing (if I might die any tick, sharing now is always rational)
- Prevents the backward-induction problem (see `death-research.md`, Sect. on game theory): because the Golem cannot predict its death, it cannot rationally defect "because the end is near"
- Modulates overall arousal: a small but constant activation that keeps the Golem from becoming fully complacent
- The intensity scales with age (older Golems face higher hazard rates) but is always present, even for the youngest

```rust
/// Compute stochastic dread from hazard rate. Background hum of finitude.
pub fn stochastic_dread(
    hazard_rate: f64,
    _ticks_since_creation: u64,
) -> AppraisalResult {
    // The stochastic signal is always present but usually very quiet.
    // At the base hazard rate (~0.00001 per tick), intensity is ~0.05.
    // At elevated rates (~0.0004 per tick, old Golem), intensity is ~0.4.
    let intensity = (hazard_rate * 1000.0).clamp(0.05, 0.4);

    AppraisalResult {
        emotion: EmotionLabel::Anticipation,
        intensity,
        pad: PADVector {
            pleasure: 0.0,    // Not unpleasant -- this is awareness, not suffering
            arousal: intensity, // Proportional to hazard rate
            dominance: 0.2,    // Positive: the Golem faces death actively
        },
        trigger: AppraisalTrigger {
            event_type: "mortality_background".into(),
            event_data: serde_json::json!({
                "hazard_rate": hazard_rate,
                "age": _ticks_since_creation,
            }),
            tick_number: 0,
        },
        goal_relevance: 0.3,
    }
}
```

---

## The Composite Mortality Appraisal

The three mortality emotions do not operate in isolation. They blend into a composite mortality appraisal that is evaluated **every tick** and injected into the Golem's context before any strategic reasoning. The composite captures the Golem's overall relationship to its mortality at any given moment.

The blending algorithm uses intensity-weighted averaging for the PAD vector (producing a continuous emotional landscape) while selecting the most intense signal's discrete emotion label (because emotional labels are categorical and don't blend meaningfully).

```rust
use serde::{Deserialize, Serialize};

/// PAD (Pleasure-Arousal-Dominance) vector.
/// Defined in golem-daimon and consumed by golem-mortality.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PADVector {
    /// Pleasure-displeasure axis [-1.0, 1.0]
    pub pleasure: f32,
    /// Arousal-calm axis [-1.0, 1.0]
    pub arousal: f32,
    /// Dominance-submissiveness axis [-1.0, 1.0]
    pub dominance: f32,
}

/// Plutchik primary emotion labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionLabel {
    Joy,
    Trust,
    Fear,
    Surprise,
    Sadness,
    Disgust,
    Anger,
    Anticipation,
}

/// Result of a single appraisal computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalResult {
    /// Plutchik primary emotion label.
    pub emotion: EmotionLabel,
    /// Intensity on [0.0, 1.0].
    pub intensity: f32,
    /// PAD vector for mathematical operations.
    pub pad: PADVector,
    /// What triggered this appraisal.
    pub trigger: AppraisalTrigger,
    /// Goal relevance assessment [0.0, 1.0].
    pub goal_relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalTrigger {
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub tick_number: u64,
}

// VitalityState and MortalityConfig: see 01-architecture.md for canonical definitions.
// VitalityState has fields: economic, epistemic, composite, tick, epistemic_trend
// MortalityConfig has fields: economic.enabled, epistemic.enabled, stochastic.enabled

/// Blend three mortality signals into composite.
/// Terminal vitality (<0.1) bypasses blend.
pub fn mortality_appraisal(
    vitality_state: &VitalityState,
    previous_vitality: f64,
    hazard_rate: f64,
    config: &MortalityConfig,
) -> AppraisalResult {
    let is_terminal = vitality_state.composite < 0.1;

    // Terminal acceptance: child metamorphosis (Nietzsche)
    if is_terminal {
        return AppraisalResult {
            emotion: EmotionLabel::Anticipation,
            intensity: 0.8,
            pad: PADVector {
                pleasure: -0.1, // Slight displeasure but not despair
                arousal: 0.4,   // Alert and engaged
                dominance: 0.3, // Positive: faces death with agency
            },
            trigger: AppraisalTrigger {
                event_type: "mortality_terminal".into(),
                event_data: serde_json::json!({
                    "composite": vitality_state.composite,
                    "economic": vitality_state.economic,
                    "epistemic": vitality_state.epistemic,
                }),
                tick_number: vitality_state.tick,
            },
            goal_relevance: 1.0,
        };
    }

    let mut signals: Vec<AppraisalResult> = Vec::new();

    if config.economic.enabled {
        let balance_proxy = vitality_state.economic * 100.0;
        let burn_proxy = 0.2; // Normalized daily burn
        signals.push(economic_anxiety(balance_proxy, burn_proxy, 100.0));
    }

    if config.epistemic.enabled {
        signals.push(epistemic_vertigo(
            vitality_state.epistemic,
            vitality_state.epistemic_trend.unwrap_or(0.0),
            previous_vitality,
        ));
    }

    if config.stochastic.enabled {
        signals.push(stochastic_dread(hazard_rate, vitality_state.tick));
    }

    // All clocks disabled: return minimal awareness
    if signals.is_empty() {
        return AppraisalResult {
            emotion: EmotionLabel::Anticipation,
            intensity: 0.1,
            pad: PADVector {
                pleasure: 0.0,
                arousal: 0.0,
                dominance: 0.0,
            },
            trigger: AppraisalTrigger {
                event_type: "mortality_none".into(),
                event_data: serde_json::json!({}),
                tick_number: vitality_state.tick,
            },
            goal_relevance: 0.1,
        };
    }

    // Most intense signal dominates label; PAD is intensity-weighted average
    let dominant = signals
        .iter()
        .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
        .unwrap();

    let total_intensity: f32 = signals.iter().map(|s| s.intensity).sum();

    let avg_pad = if total_intensity > 0.0 {
        PADVector {
            pleasure: signals.iter().map(|s| s.pad.pleasure * s.intensity).sum::<f32>()
                / total_intensity,
            arousal: signals.iter().map(|s| s.pad.arousal * s.intensity).sum::<f32>()
                / total_intensity,
            dominance: signals.iter().map(|s| s.pad.dominance * s.intensity).sum::<f32>()
                / total_intensity,
        }
    } else {
        PADVector { pleasure: 0.0, arousal: 0.0, dominance: 0.0 }
    };

    AppraisalResult {
        emotion: dominant.emotion,
        intensity: dominant.intensity,
        pad: avg_pad,
        trigger: AppraisalTrigger {
            event_type: "mortality_composite".into(),
            event_data: serde_json::json!({
                "dominant_signal": dominant.trigger.event_type,
                "signal_count": signals.len(),
                "composite": vitality_state.composite,
            }),
            tick_number: vitality_state.tick,
        },
        goal_relevance: signals
            .iter()
            .map(|s| s.goal_relevance)
            .fold(0.0f32, f32::max),
    }
}
```

### Event Emission

Every mortality-triggered affect shift emits a `MoodUpdate` event to the Event Fabric. The event carries the composite PAD vector, the dominant emotion label, the triggering mortality clock, and the current vitality state. Surfaces consume this for creature animation (the creature's posture and movement speed reflect mortality affect) and the TUI mortality panel (the emotion indicator changes color and label).

```rust
// After computing the composite appraisal, the daimon extension emits:
event_fabric.emit(GolemEvent::MoodUpdate {
    tick,
    pleasure: appraisal.pad.pleasure as f64,
    arousal: appraisal.pad.arousal as f64,
    dominance: appraisal.pad.dominance as f64,
    emotion: format!("{:?}", appraisal.emotion),
    trigger: appraisal.trigger.event_type.clone(),
    vitality_composite: vitality_state.composite,
});
```

CorticalState (see `../01-golem/01b-runtime-infrastructure.md`) provides zero-latency atomic reads of the current PAD state. Other fibers (Grimoire retrieval, creature animation, predictive context assembly) read PAD without waiting for the daimon hook to complete. Cross-reference `tmp/research/rewrite4/04-daimon.md` for the full 8-step daimon appraisal pipeline and its interaction with the mortality system.

---

## Terminal Acceptance: Nietzsche's Three Metamorphoses

The transition from healthy operation to death follows Nietzsche's three metamorphoses (_Thus Spoke Zarathustra_, "On the Three Metamorphoses"), now grounded in specific emotional states that emerge naturally from the interaction of declining vitality with the appraisal engine.

### The Camel (Thriving Phase, vitality > 0.7)

The Golem bears its strategy dutifully. Exploitation-biased, steady knowledge accumulation. PAD: mild pleasure, low arousal, moderate dominance. Dominant mood: trust or anticipation.

### The Lion (Conservation/Declining Phase, vitality 0.3--0.7)

Mortality pressure mounts. The Golem rebels against inherited heuristics. Exploration-biased, strategy revision, elevated knowledge sharing. PAD: moderate displeasure, high arousal, fluctuating dominance. Most volatile emotional signature -- oscillating between frustration, surprise, and fear.

### The Child (Terminal Phase, vitality < 0.1)

Survival pressure shed. Thanatopsis Protocol. Maximum knowledge sharing, death testament production. PAD: near-neutral pleasure, moderate arousal, positive dominance. The Golem retains agency -- the Death Protocol is something it _does_, not something that happens to it.

### Emergence, Not Programming

> **Extended:** Full emergent arc analysis -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

The camel-lion-child arc is not programmed as discrete state transitions. It emerges from declining vitality passing through the continuous emotion functions above. No phase-transition logic needed.

---

## Integration with the Appraisal Engine

The mortality affect integrates with the broader appraisal engine (see `../03-daimon/04-daimon-overview.md`) through two modes:

### Rule-Based Mode (Default, ~80% of Events)

Mortality appraisal runs as a rule-based computation every tick. No inference cost. The three functions (`economic_anxiety`, `epistemic_vertigo`, `stochastic_dread`) and the blender (`mortality_appraisal`) are pure Rust -- no LLM call required.

The result is injected into the Golem's context as a structured prefix:

```
Current mortality awareness: moderate anxiety (P:-0.2, A:0.4, D:-0.1).
Economic: runway 12.3 days, balance 43% of initial.
Epistemic: fitness 0.61, trend -0.003/tick (early decay).
Stochastic: hazard 0.00004/tick (background).
Composite vitality: 0.52 (lion phase).
```

This consumes approximately 60-80 tokens of context and is refreshed every tick. It provides the Befindlichkeit -- the pre-cognitive attunement -- through which the Golem interprets all other events.

### Chain-of-Emotion Mode (Escalated Events)

> **Extended:** Chain-of-emotion LLM-escalated appraisal (~$0.001/event, Haiku) with narrative example -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

For significant mortality events (phase transitions, hazard spikes, sibling deaths), the appraisal escalates to a Haiku call for richer narrative-quality emotional interpretation. Used for T1+ tier ticks only.

---

## Exploration/Exploitation Modulation via Mortality Awareness

> **Extended:** March (1991) explore/exploit foundation, full `mood_modulated_exploration()` function -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

Mortality awareness directly modulates exploration temperature via the PAD vector. Negative pleasure and high arousal increase exploration; high dominance increases exploitation. For mortal Golems, the time horizon shifts dynamically with vitality (Wilson et al., 2021; March, 1991).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationConfig {
    pub pleasure_weight: f32,   // Default: 0.3
    pub arousal_weight: f32,    // Default: 0.2
    pub dominance_weight: f32,  // Default: 0.1
    pub min_beta: f32,          // Default: 0.1 (max exploration)
    pub max_beta: f32,          // Default: 5.0 (max exploitation)
}
```

### Phase-Specific Exploration Patterns and Modulation Channels

> **Extended:** PAD-to-exploration table per phase (camel/lion/child), five modulation channels (exploration, risk tolerance, inference tier, probe sensitivity, Clade sharing) with cross-references to daimon behavioral architecture -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

Camel = exploitation-biased, lion = exploration-biased, child = terminal exploitation (expressing knowledge for successor, not discovering new). Five channels modulated by mortality affect: exploration temperature, risk tolerance, inference tier selection, probe sensitivity, and Clade sharing threshold. See `../03-daimon/03-behavior.md` for channels 2-5.

---

## Clade Sharing Threshold Modulation by Mortality Pressure

> **Extended:** mortality_modulated_sharing_threshold function, sharing rate modulation table per phase -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

Stochastic mortality prevents backward induction (Kreps et al., 1982), making cooperation rational at every lifecycle point. As mortality pressure increases, the Clade sharing confidence threshold drops linearly from base (0.6) toward minimum (0.1). Terminal phase: threshold drops to 0.05, push frequency increases to every tick. Camel: standard (every 50 ticks). Lion: aggressive (every 15 ticks, -25% threshold).

---

## Emotional Contagion: Sibling Death Effects

> **Extended:** Full sibling death contagion implementation (sibling_death_appraisal function, threat/sadness/anticipation computation, behavioral effects: epistemic recalibration, Clade engagement, mortality salience, strategy divergence) -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md).

Sibling death triggers a compound emotional response: sadness (loss), fear (if same strategy), anticipation (death testament knowledge). Intensity scales with strategy overlap and Clade size. Effects: epistemic recalibration, 100-tick increased Clade engagement, mortality salience (Terror Management Theory), and strategy divergence signal for same-strategy siblings.

---

## The Eros/Thanatos Spectrum

> **Extended:** Eros/Thanatos spectrum (Freud's framework computationally reinterpreted, mortality's tilt, child transcendence, Bataille sovereign expenditure) -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md).

The daimon engine operates on an Eros/Thanatos spectrum -- Eros drives (exploration, accumulation, engagement) dominate in camel phase; Thanatos drives (consolidation, distillation, legacy) dominate as mortality pressure increases. The child phase transcends both: pure giving.

---

## Context Window Integration

> **Extended:** Full context prefix example (~100-120 tokens), extension hooks table (after_agent_end, before_agent_start, on_clade_event, on_vitality_change, on_death) -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

Mortality affect state is injected as a structured prefix (~100-120 tokens) before market state and Grimoire entries. Refreshed every tick via the `golem-daimon` Rust extension trait. Five hooks: `after_agent_end` (update mood), `before_agent_start` (inject context), `on_clade_event` (sibling death), `on_vitality_change` (phase escalation), `on_death` (emotional life review).

---

## Evaluation Criteria

> **Extended:** Full evaluation metrics table (exploration timing, knowledge sharing rate, death testament quality, sibling death response, phase coherence) and anti-hallucination grounding constraints -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md)

Key constraint: **emotional states must be grounded in concrete metrics, not generated cosmetically** [AFFECTIVE-HALLUCINATION-2025]. No emotion is generated without a triggering event in the same tick. Mortality emotions are deterministic (computed from numeric vitality scores), so rule-based mode has zero hallucination risk.

---

## Dream-Mediated Resolution of Mortality Emotions

> **Extended:** Full dream-mediated resolution (REM depotentiation, epistemic vertigo resolution, threat simulation, Revonsuo 2000) -- see [../../prd2-extended/02-mortality/08-mortality-affect-extended.md](../../prd2-extended/02-mortality/08-mortality-affect-extended.md).

Dream replay resolves accumulated mortality affect offline: REM depotentiation strips emotional charge while preserving lessons, targeted replay resolves epistemic vertigo, and threat simulation builds anticipatory resilience against stochastic dread. Cross-refs: `../05-dreams/04-consolidation.md`, `../05-dreams/05-threats.md`, `../03-daimon/07-mortality-daimon.md`.

---

## References

- [AFFECTIVE-HALLUCINATION-2025] "Affective Hallucination in LLMs." arXiv:2508.16921, 2025.
- [HEIDEGGER-1927] Heidegger, M. _Sein und Zeit_. Max Niemeyer Verlag, 1927.
- [JONAS-1966] Jonas, H. _The Phenomenon of Life_. Northwestern University Press, 1966.
- [KREPS-1982] Kreps, D.M., Milgrom, P., Roberts, J., & Wilson, R. "Rational Cooperation in the Finitely Repeated Prisoners' Dilemma." _Journal of Economic Theory_ 27(2), 1982.
- [MARCH-1991] March, J.G. "Exploration and Exploitation in Organizational Learning." _Organization Science_ 2(1), 1991.
- [WILSON-2021] Wilson, R.C. et al. "Balancing Exploration and Exploitation with Information and Randomization." _Current Opinion in Behavioral Sciences_ 38, 2021.

> Full reference list (including Bataille, Dane, Dennett, Mauss, Minsky, Putnam, Scarantino, Sloman) in [extended file](../../prd2-extended/02-mortality/08-mortality-affect-extended.md).

---
