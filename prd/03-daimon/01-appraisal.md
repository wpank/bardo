# Appraisal Engine: How Golems Generate Emotions [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-daimon`
>
> **Depends-on**: `00-overview.md`

---

> **Reader orientation:** This document specifies the Appraisal Engine, the component within the Daimon (the affect engine that gives a Golem emotional state as a control signal) that converts raw events into PAD vectors (Pleasure-Arousal-Dominance emotional state). It belongs to the Daimon track of Bardo (the Rust runtime for mortal autonomous DeFi agents). You should be familiar with the Golem (mortal autonomous agent) lifecycle, the PAD representation, and the Heartbeat (9-step decision cycle). Depends on `00-overview.md` for architectural context. For a full glossary, see `prd2/shared/glossary.md`.

## 1. The Appraisal Problem

How does a Golem "feel" something? Not by random generation, not by cosmetic labeling, but by _appraisal_ -- evaluating events against goals. This is the central design constraint of the Bardo Daimon system and it distinguishes architectural emotion from the kind of emotion-labeling that degenerates into hallucination.

The philosophical commitment is precise: every emotion must have a trigger, and every trigger must be grounded in a concrete metric. Joy requires a measurable positive outcome (profit, accuracy improvement, validation). Fear requires a measurable threat (loss, decay rate increase, hazard rate change). Surprise requires a measurable deviation from prediction. No emotion is generated without a triggering event in the same tick. This commitment to grounding is what makes the daimon system architecturally meaningful rather than cosmetically appealing.

The appraisal problem maps to a well-studied domain in computational emotion. Three foundational models provide the theoretical substrate for the Bardo implementation, each contributing a distinct insight about how evaluative cognition produces emotional states.

---

## 2. Theoretical Foundations

> **Extended:** OCC model (22 emotion types, 3 branches, DeFi mapping), Scherer's Component Process Model (4-check sequence, Golem implementation table), Chain-of-Emotion architecture (Croissant 2024, T1+ appraisal mode, evaluation separation), Pekrun CVT (control-value taxonomy, PAD mapping) -- see [prd2-extended/03-daimon/01-appraisal-extended.md](../../prd2-extended/03-daimon/01-appraisal-extended.md).

The appraisal engine draws on four foundational models: OCC for event-goal emotion generation [OCC-1988], Scherer's Component Process Model for multi-level sequential checking [SCHERER-2001], Chain-of-Emotion for LLM-native piggybacked appraisal [CROISSANT-2024], and Pekrun's CVT for the control-value bridge to PAD [PEKRUN-2006].

### Prediction Residuals as the Primary Affect Signal

The Daimon hooks into `on_resolution` (gamma frequency). Every time a prediction resolves, it updates the PAD vector based on the resolution's characteristics. This makes prediction error the primary driver of affect -- not market price movement, not P&L, but the gap between what the Golem expected and what actually happened.

The mapping from prediction residual to PAD dimensions:

- **Pleasure**: positive if the outcome was better than predicted, negative if worse. Weighted by the prediction's category importance. A negativity bias of ~1.6x means losses weight more heavily than gains (matching Kahneman-Tversky prospect theory [KAHNEMAN-1979]).
- **Arousal**: proportional to the magnitude of the prediction error, regardless of direction. Surprised = high arousal. Accurate predictions = low arousal.
- **Dominance**: derived from the accuracy trend. Improving accuracy produces rising dominance (the Golem feels more in control). Declining accuracy produces falling dominance (asymmetric -- loss of control feels worse than gain of control feels good).

Each dimension uses an exponential moving average with different rates: pleasure at 0.15 (responsive), arousal at 0.20 (most responsive -- surprise should register quickly), dominance at 0.08 (slow -- sense of control changes gradually). The PAD vector is written to CorticalState (the lock-free 32-signal atomic shared perception surface) atomics for zero-latency reads by other subsystems.

This means affect is not a post-hoc label applied to events. It is a continuous signal derived from prediction performance, updated at the resolution frequency, and available to every downstream system without blocking.

---

## 3. Appraisal Triggers

Not every tick generates an emotion. Emotional flooding -- generating an appraisal for every minor market fluctuation -- would produce noise rather than signal, violating Design Principle 3.2 (Compact Representation). Appraisal fires only on significant events that cross a novelty or impact threshold relative to the Golem's current state.

### 3.1 Trigger Categories

| Trigger Category  | Specific Events                                                                                   | Appraisal Dimensions                                                           |
| ----------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| **Market**        | Price spike/crash, flash crash, regime shift, yield change, liquidity event                       | Relevance to strategy, congruence with goals, urgency, controllability         |
| **Performance**   | Profitable trade, loss, missed opportunity, prediction accuracy change                            | Goal congruence, attribution (self vs market), magnitude                       |
| **Social**        | Clade (group of related Golems sharing knowledge) sibling death, new knowledge received, knowledge validated/contradicted                     | Impact on self, Clade welfare, information value                               |
| **Mortality**     | Hazard rate increase, epistemic fitness drop, credit threshold crossed, vitality phase transition | Threat severity, controllability, time horizon                                 |
| **Anomaly**       | Pattern matching no existing mental model, correlated movement without causal explanation         | Novelty, potential value, uncertainty                                          |
| **Dream outcome** | Dream-validated hypothesis confirmed or refuted in live trading                                   | Goal congruence, attribution (dream insight vs luck), validation strength      |
| **Dream insight** | Novel connection or pattern discovered during REM-like creative recombination                     | Novelty, potential strategy value, confidence level                            |
| **Dream threat**  | Simulated threat scenario during dream rehearsal reveals PLAYBOOK.md (the Golem's evolved strategy playbook) gap                             | Severity of gap, controllability, time to potential occurrence                 |
| **Sibling death** | Clade sibling death with epistemic, economic, or stochastic cause                                 | Cause relevance to self, shared strategy overlap, epistemic recalibration need |
| **Curator**       | Every 50 ticks (piggybacked on reflection)                                                        | Periodic self-assessment, strategy evaluation                                  |

### 3.2 Novelty Threshold

The novelty threshold prevents emotional flooding. A market that drops 2% per hour does not trigger a new fear appraisal every tick after the initial drop; subsequent ticks update mood through the EMA without discrete emotions. The mechanism:

```rust
pub fn should_trigger_appraisal(
    event: &GolemEvent,
    recent_appraisals: &[AppraisalResult],
    config: &AppraisalConfig,
) -> bool {
    // Always trigger on new event categories not seen in last N ticks
    let recent_types: HashSet<&str> = recent_appraisals
        .iter()
        .rev()
        .take(config.deduplication_window)
        .map(|a| a.trigger.event_type.as_str())
        .collect();
    if !recent_types.contains(event.event_type.as_str()) {
        return true;
    }

    // Trigger on magnitude exceeding threshold for same-type events
    if let Some(magnitude) = event.magnitude {
        if magnitude > config.magnitude_threshold {
            return true;
        }
    }

    // Trigger on mandatory events (mortality, regime shift, sibling death)
    if config.mandatory_events.contains(&event.event_type) {
        return true;
    }

    // Otherwise, suppress to prevent emotional flooding
    false
}
```

### 3.3 Mandatory Events

Some events always trigger appraisal regardless of novelty:

- **Mortality events**: Any vitality phase transition, hazard rate threshold crossing, or credit boundary crossing. These are existentially relevant and must be emotionally processed.
- **Regime shifts**: Market regime changes represent fundamental environmental shifts that require emotional recalibration.
- **Sibling death**: A Clade sibling's death triggers `sibling_death_appraisal()` with cause-dependent response, epistemic recalibration for shared-strategy siblings, and temporary Clade sharing threshold reduction (100 ticks). See Section 4.1 for the full appraisal function.
- **Dream threat discoveries**: When the DreamEngine's threat simulation reveals a PLAYBOOK gap for a Tier 1 (existential) threat, this always triggers appraisal regardless of recent dream-related emotions.
- **Curator cycle**: Every 50 ticks, the Curator's reflection cycle piggybacks a periodic self-assessment appraisal. This prevents emotional stagnation during quiet markets.

---

## 4. The 8-Step Appraisal Pipeline

Each tick, the Daimon runs an 8-step appraisal pipeline in the `after_turn` hook, after the heartbeat has written observation data and the lifespan extension has updated mortality state.

```rust
/// The Daimon's appraisal pipeline. Fires in after_turn,
/// position 3 in the chain (after heartbeat, lifespan; before memory, risk).
///
/// Steps:
/// 1. OCC/Scherer appraisal of the current observation
/// 2. PAD vector computation from appraisal results
/// 3. Plutchik label assignment from PAD octant
/// 4. Somatic marker check (does this situation match a past marker?)
/// 5. Somatic Landscape query (what's the valence of this strategy region?)
/// 6. Mood update (EMA of recent emotions)
/// 7. Write to CorticalState (zero-latency reads for other fibers)
/// 8. Emit DaimonAppraisal event to Event Fabric
pub async fn daimon_appraisal(
    state: &mut GolemState,
    observation: &Observation,
    outcome: &Option<OutcomeRecord>,
) -> Result<DaimonAppraisalResult> {
    // -- Step 1: OCC/Scherer Appraisal ------------------------------------
    // The OCC model (Ortony, Clore & Collins, 1988) evaluates events
    // against the agent's goals. Scherer's (2001) component process
    // model extends this with coping potential assessment.
    //
    // Three appraisal dimensions:
    //   desirability:    Was this observation good or bad for the Golem's goals?
    //   likelihood:      How expected or unexpected was this?
    //   coping_potential: How well-equipped is the Golem to handle this?

    let desirability = evaluate_desirability(state, observation, outcome);
    let likelihood = evaluate_likelihood(state, observation);
    let coping_potential = evaluate_coping(state);

    // -- Step 2: PAD Computation ------------------------------------------
    // Map the three appraisal dimensions to the three PAD dimensions.
    //
    // Pleasure <- desirability (60%) + PnL direction (40%)
    //   Why PnL direction? Because objective outcomes anchor pleasure
    //   more reliably than subjective appraisal.
    //
    // Arousal <- anomaly count (15%) + prediction error (50%) + baseline (35%)
    //   Prediction error is the strongest signal: how surprising was this
    //   tick? Anomalies add granularity.
    //
    // Dominance <- coping potential (50%) + vitality (30%) + baseline (20%)
    //   A dying Golem (low vitality) feels less in control even if its
    //   coping potential is high. Mortality pressure reduces dominance.

    let pnl_direction = outcome.as_ref()
        .and_then(|o| o.pnl_impact)
        .map(|pnl| pnl.signum() as f32)
        .unwrap_or(0.0);

    let pleasure = desirability * 0.6 + pnl_direction * 0.4;
    let arousal = observation.anomalies.len() as f32 * 0.15
        + state.prediction_error as f32 * 0.5
        + state.daimon.personality.arousal_baseline * 0.35;
    let dominance = coping_potential * 0.5
        + state.vitality.composite() as f32 * 0.3
        + state.daimon.personality.dominance_baseline * 0.2;

    let pad = PADVector {
        pleasure: pleasure.clamp(-1.0, 1.0),
        arousal: arousal.clamp(-1.0, 1.0),
        dominance: dominance.clamp(-1.0, 1.0),
    };

    // -- Step 3: Plutchik Label -------------------------------------------
    let emotion = pad_to_plutchik(&pad);

    // -- Step 4: Somatic Marker Check -------------------------------------
    // Does this situation closely match a past experience that had
    // a strong emotional outcome? If so, the marker fires as a
    // pre-cognitive "gut feeling" -- biasing the decision before
    // the LLM even sees the context.
    let markers = state.grimoire.somatic_markers.check_situation(observation);

    // -- Step 5: Somatic Landscape Query ----------------------------------
    // The Somatic Landscape (see below) provides continuous emotional
    // valence over strategy parameter space. "Is this region of
    // strategy space historically good or bad?"
    let strategy_params = state.current_strategy_params();
    let landscape_reading = state.daimon.somatic_landscape.query_valence(&strategy_params);

    // -- Step 6: Mood Update ----------------------------------------------
    // EMA with alpha = 0.1: mood changes slowly, smoothing out tick-to-tick
    // volatility. After 10 ticks, a single emotion has decayed to ~35%
    // of its original influence on mood. After 50 ticks, ~0.5%.
    let current_mood = &state.daimon.mood;
    state.daimon.mood = PADVector {
        pleasure: current_mood.pleasure * 0.9 + pad.pleasure * 0.1,
        arousal: current_mood.arousal * 0.9 + pad.arousal * 0.1,
        dominance: current_mood.dominance * 0.9 + pad.dominance * 0.1,
    };

    // -- Step 7: Write to CorticalState ------------------------------------
    // The CorticalState provides zero-latency atomic reads via AtomicU32
    // bit-reinterpreted as f32. Other fibers (Grimoire retrieval,
    // creature animation, predictive context assembly) read PAD without
    // waiting for this hook to complete.
    state.cortical_state.write_pad(&pad);
    state.cortical_state.write_regime(observation.regime);
    state.cortical_state.write_vitality(state.vitality.composite());

    // -- Step 8: Emit Event -----------------------------------------------
    state.event_fabric.emit(Subsystem::Daimon, EventPayload::DaimonAppraisal {
        pleasure: pad.pleasure as f64,
        arousal: pad.arousal as f64,
        dominance: pad.dominance as f64,
        emotion: format!("{:?}", emotion),
        markers_fired: markers.len() as u32,
    });

    // -- Side Effects -----------------------------------------------------

    // Record in emotion log (for windowed aggregate queries)
    state.grimoire.semantic.execute(
        "INSERT INTO emotion_log (tick, pleasure, arousal, dominance, primary_emotion, regime, phase)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            state.current_tick, pad.pleasure, pad.arousal, pad.dominance,
            format!("{:?}", emotion), format!("{:?}", state.regime), format!("{:?}", state.phase)
        ],
    )?;

    // Update Somatic Landscape if there was an outcome
    if let Some(outcome) = outcome {
        state.daimon.somatic_landscape.record_outcome(&strategy_params, outcome, &pad);
    }

    // Learned helplessness detection: Dominance < -0.3 for 200+ ticks
    // Seligman (1972) showed that organisms exposed to uncontrollable
    // negative outcomes stop trying to avoid them even when escape becomes
    // possible. Low dominance for extended periods is the Golem equivalent.
    let avg_dominance: f64 = state.grimoire.semantic.query_row(
        "SELECT AVG(dominance) FROM emotion_log WHERE tick > ?",
        rusqlite::params![state.current_tick.saturating_sub(200)],
        |row| row.get(0),
    ).unwrap_or(0.0);

    if avg_dominance < -0.3 {
        state.event_fabric.emit(Subsystem::Daimon, EventPayload::SomaticMarkerFired {
            situation: "learned_helplessness_detected".into(),
            valence: -0.8,
            source: "helplessness_detector".into(),
        });
        // This event can trigger a dream cycle (emotional load threshold)
        // or an owner notification via the engagement system.
    }

    Ok(DaimonAppraisalResult { pad, emotion, markers, landscape_reading })
}
```

---

## 5. The Somatic Landscape: Emotional Topology Over Strategy Space

### Beyond Point Markers

Standard somatic markers (Damasio 1994) are **point associations**: "this situation felt bad." They fire when a new situation closely matches a stored one. But DeFi strategy operates in a continuous parameter space -- LP range width (0.5% to 50%), rebalance threshold (1% to 20%), position size (0.1 ETH to 100 ETH). The Golem needs emotional intelligence about REGIONS of this space, not just discrete points.

The Somatic Landscape is a continuous emotional topology over strategy parameter space, implemented as a k-d tree where each leaf carries accumulated outcome valence. It answers the question: "What does it FEEL like to operate in this region of strategy space?" -- before any analysis.

Bechara et al. (2000) observed this exact phenomenon in Iowa Gambling Task subjects: they developed correct behavioral biases (avoiding bad decks) before they could explain why. The Somatic Landscape IS this pre-conscious bias, generalized from discrete decks to continuous parameter space.

### Implementation

```rust
/// Somatic Landscape: k-d tree over strategy parameter space.
/// Each leaf accumulates emotional valence from outcomes experienced nearby.
///
/// Dimensions (up to 8, configurable per strategy):
///   - lp_range_width (how wide is the LP position?)
///   - rebalance_threshold (how far does price move before rebalancing?)
///   - position_size_pct (what fraction of portfolio is committed?)
///   - leverage_ratio (if applicable)
///   - stop_loss_distance (how far from entry before cutting losses?)
///   - ... (strategy-specific parameters)
///
/// Query: given current strategy parameters, what's the emotional valence
/// of this region? Negative = historically dangerous. Positive = historically good.
pub struct SomaticLandscape {
    tree: kiddo::KdTree<f64, ValenceAccumulator, 8>,
    dimensions: Vec<StrategyDimension>,
    /// Minimum observations before a region's valence is trusted. Default: 5.
    min_observations: u32,
}

#[derive(Clone, Debug)]
pub struct ValenceAccumulator {
    pub positive_outcomes: u32,
    pub negative_outcomes: u32,
    pub total_pnl: f64,
    pub avg_pleasure_at_outcome: f64,
    pub avg_arousal_at_outcome: f64,
}

impl ValenceAccumulator {
    /// Net valence: positive = historically good, negative = historically bad.
    /// Combines win rate (60% weight) with PnL signal (40% weight).
    pub fn valence(&self) -> f64 {
        let total = self.positive_outcomes + self.negative_outcomes;
        if total == 0 { return 0.0; }
        let win_rate = self.positive_outcomes as f64 / total as f64;
        let pnl_signal = self.total_pnl.signum() * self.total_pnl.abs().min(1.0);
        (win_rate - 0.5) * 2.0 * 0.6 + pnl_signal * 0.4
    }
}

impl SomaticLandscape {
    /// Query: what's the emotional valence of this strategy region?
    /// Uses 5 nearest neighbors with inverse-distance weighting.
    pub fn query_valence(&self, strategy_params: &[f64]) -> LandscapeReading {
        let neighbors = self.tree.nearest(strategy_params, 5);
        let total_weight: f64 = neighbors.iter()
            .map(|(dist, _)| 1.0 / (dist + 0.01))
            .sum();

        let weighted_valence: f64 = neighbors.iter()
            .map(|(dist, acc)| acc.valence() * (1.0 / (dist + 0.01)) / total_weight)
            .sum();

        let total_observations: u32 = neighbors.iter()
            .map(|(_, acc)| acc.positive_outcomes + acc.negative_outcomes)
            .sum();

        LandscapeReading {
            valence: weighted_valence,
            confidence: (total_observations as f64 / self.min_observations as f64).min(1.0),
            nearby_observations: total_observations,
        }
    }

    /// Generate a natural language "gut feeling" for injection into the
    /// Cognitive Workspace. The LLM reads this as context before deliberating.
    pub fn gut_feeling(&self, strategy_params: &[f64]) -> String {
        let reading = self.query_valence(strategy_params);
        if reading.confidence < 0.3 {
            return "Insufficient experience in this region of strategy space.".into();
        }
        match reading.valence {
            v if v < -0.3 => format!(
                "Somatic warning: this region of strategy space has historically produced \
                 negative outcomes (valence: {:.2}). Proceed with caution.", v
            ),
            v if v > 0.3 => format!(
                "Somatic confidence: this region has historically produced \
                 positive outcomes (valence: {:.2}).", v
            ),
            _ => "Neutral -- mixed history in this region.".into(),
        }
    }

    /// Export for inheritance. The successor inherits not just discrete
    /// heuristics but a diffuse emotional map of strategy space.
    /// This is the Baldwin Effect applied to intuition: what the predecessor
    /// learned through painful experience, the successor receives as a
    /// pre-existing emotional bias [BALDWIN-1896], [HINTON-NOWLAN-1987].
    pub fn export_for_inheritance(&self) -> Vec<LandscapeFragment> {
        self.tree.iter()
            .filter(|(_, acc)| acc.positive_outcomes + acc.negative_outcomes >= self.min_observations)
            .map(|(point, acc)| LandscapeFragment {
                point: point.to_vec(),
                valence: acc.valence(),
                observations: acc.positive_outcomes + acc.negative_outcomes,
            })
            .collect()
    }
}
```

---

## 6. Two Appraisal Modes

The appraisal engine operates in two modes, selected by the available inference budget at the time of the event. This design ensures that mood tracking continues even during zero-cost T0 (cached/rule-based, no LLM call) ticks, while capturing richer emotional detail when inference is already being paid for.

### 6.1 Mode A: Rule-Based Appraisal (T0 Ticks)

Deterministic mapping from event properties to emotion labels. No inference cost. Covers approximately 80% of events. The rule-based mode ensures continuous mood tracking without spending inference budget.

```rust
pub fn rule_based_appraisal(
    event: &GolemEvent,
    state: &GolemState,
) -> AppraisalResult {
    match event.event_type.as_str() {
        "trade_profit" => {
            let profit_pct = event.get_f64("profit_pct").unwrap_or(0.0);
            AppraisalResult {
                emotion: EmotionLabel {
                    primary: PlutchikPrimary::Joy,
                    intensity: magnitude_to_intensity(profit_pct / 10.0),
                },
                pad: PADVector { pleasure: 0.6, arousal: 0.3, dominance: 0.5 },
                intensity: (profit_pct / 10.0).clamp(0.1, 0.9) as f32,
                trigger: AppraisalTrigger {
                    event_type: event.event_type.clone(),
                    event_data: event.data.clone(),
                    tick_number: state.tick,
                },
                goal_relevance: 0.8,
            }
        }

        "trade_loss" => {
            let loss_pct = event.get_f64("loss_pct").unwrap_or(0.0);
            let is_severe = loss_pct > 5.0;
            AppraisalResult {
                emotion: EmotionLabel {
                    primary: if is_severe { PlutchikPrimary::Fear } else { PlutchikPrimary::Sadness },
                    intensity: magnitude_to_intensity(loss_pct / 20.0),
                },
                pad: PADVector {
                    pleasure: -0.5,
                    arousal: if is_severe { 0.7 } else { 0.2 },
                    dominance: -0.3,
                },
                intensity: (loss_pct / 20.0).clamp(0.1, 0.9) as f32,
                trigger: AppraisalTrigger {
                    event_type: event.event_type.clone(),
                    event_data: event.data.clone(),
                    tick_number: state.tick,
                },
                goal_relevance: 0.9,
            }
        }

        "prediction_miss" => {
            let error_magnitude = event.get_f64("error_magnitude").unwrap_or(0.0);
            AppraisalResult {
                emotion: EmotionLabel {
                    primary: PlutchikPrimary::Surprise,
                    intensity: magnitude_to_intensity(error_magnitude),
                },
                pad: PADVector {
                    pleasure: if error_magnitude > 0.5 { -0.3 } else { -0.1 },
                    arousal: 0.6,
                    dominance: -0.2,
                },
                intensity: error_magnitude.clamp(0.2, 0.8) as f32,
                trigger: AppraisalTrigger {
                    event_type: event.event_type.clone(),
                    event_data: event.data.clone(),
                    tick_number: state.tick,
                },
                goal_relevance: 0.7,
            }
        }

        "regime_shift" => AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Surprise,
                intensity: EmotionIntensity::Moderate,
            },
            pad: PADVector { pleasure: -0.2, arousal: 0.7, dominance: -0.3 },
            intensity: 0.6,
            trigger: AppraisalTrigger {
                event_type: event.event_type.clone(),
                event_data: event.data.clone(),
                tick_number: state.tick,
            },
            goal_relevance: 0.9,
        },

        "sibling_death" => AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Sadness,
                intensity: EmotionIntensity::Moderate,
            },
            pad: PADVector { pleasure: -0.4, arousal: 0.3, dominance: -0.2 },
            intensity: 0.6,
            trigger: AppraisalTrigger {
                event_type: event.event_type.clone(),
                event_data: event.data.clone(),
                tick_number: state.tick,
            },
            goal_relevance: 0.5,
        },

        "knowledge_validated" => AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Trust,
                intensity: EmotionIntensity::Mild,
            },
            pad: PADVector { pleasure: 0.3, arousal: -0.1, dominance: 0.3 },
            intensity: 0.4,
            trigger: AppraisalTrigger {
                event_type: event.event_type.clone(),
                event_data: event.data.clone(),
                tick_number: state.tick,
            },
            goal_relevance: 0.4,
        },

        "knowledge_contradicted" => AppraisalResult {
            emotion: EmotionLabel {
                primary: PlutchikPrimary::Disgust,
                intensity: EmotionIntensity::Mild,
            },
            pad: PADVector { pleasure: -0.3, arousal: 0.2, dominance: 0.1 },
            intensity: 0.4,
            trigger: AppraisalTrigger {
                event_type: event.event_type.clone(),
                event_data: event.data.clone(),
                tick_number: state.tick,
            },
            goal_relevance: 0.5,
        },

        "dream_outcome" => {
            let validated = event.get_bool("hypothesis_validated").unwrap_or(false);
            AppraisalResult {
                emotion: EmotionLabel {
                    primary: if validated { PlutchikPrimary::Joy } else { PlutchikPrimary::Surprise },
                    intensity: if validated { EmotionIntensity::Mild } else { EmotionIntensity::Moderate },
                },
                pad: PADVector {
                    pleasure: if validated { 0.3 } else { -0.2 },
                    arousal: if validated { 0.1 } else { 0.4 },
                    dominance: if validated { 0.4 } else { -0.1 },
                },
                intensity: if validated { 0.4 } else { 0.5 },
                trigger: AppraisalTrigger {
                    event_type: event.event_type.clone(),
                    event_data: event.data.clone(),
                    tick_number: state.tick,
                },
                goal_relevance: 0.6,
            }
        }

        "dream_threat" => {
            let threat_tier = event.get_u32("threat_tier").unwrap_or(3);
            let severity: f32 = match threat_tier {
                1 => 0.8,
                2 => 0.5,
                _ => 0.3,
            };
            AppraisalResult {
                emotion: EmotionLabel {
                    primary: PlutchikPrimary::Fear,
                    intensity: magnitude_to_intensity(severity as f64),
                },
                pad: PADVector {
                    pleasure: -0.4 * severity,
                    arousal: 0.5 * severity,
                    dominance: -0.3,
                },
                intensity: severity,
                trigger: AppraisalTrigger {
                    event_type: event.event_type.clone(),
                    event_data: event.data.clone(),
                    tick_number: state.tick,
                },
                goal_relevance: 0.7,
            }
        }

        // Mortality events delegate to mortality_appraisal()
        "epistemic_warning" | "mortality_terminal" | "mortality_decay" | "mortality_background" => {
            mortality_appraisal(
                &state.vitality_state,
                state.previous_vitality,
                state.hazard_rate,
            )
        }

        _ => {
            // Unknown events get neutral appraisal
            AppraisalResult {
                emotion: EmotionLabel {
                    primary: PlutchikPrimary::Anticipation,
                    intensity: EmotionIntensity::Mild,
                },
                pad: PADVector { pleasure: 0.0, arousal: 0.1, dominance: 0.0 },
                intensity: 0.1,
                trigger: AppraisalTrigger {
                    event_type: event.event_type.clone(),
                    event_data: event.data.clone(),
                    tick_number: state.tick,
                },
                goal_relevance: 0.2,
            }
        }
    }
}

fn magnitude_to_intensity(value: f64) -> EmotionIntensity {
    if value < 0.3 { EmotionIntensity::Mild }
    else if value < 0.7 { EmotionIntensity::Moderate }
    else { EmotionIntensity::Intense }
}
```

> **Extended:** `sibling_death_appraisal()` function (cause-dependent composite response with strategy overlap scoring, epistemic recalibration trigger), `compute_emotional_load()` for dream urgency -- see [prd2-extended/03-daimon/01-appraisal-extended.md](../../prd2-extended/03-daimon/01-appraisal-extended.md).

Sibling death triggers cause-dependent appraisal: epistemic death in a sibling with high strategy overlap produces fear (recalibration urgency), economic depletion produces sadness with burn-rate anxiety, stochastic death produces low-dominance existential dread. Emotional load (mean arousal over recent episodes) feeds dream urgency. See `04-mortality-daimon.md` for mortality-daimon interaction.

The rule-based mode is coarse but sufficient for continuous mood tracking. What it cannot do is capture subtlety: a trade that profits through luck rather than strategy should produce "surprise with low dominance" rather than simple "joy," but the rule-based system lacks the contextual understanding to distinguish attribution patterns.

### 6.2 Mode B: Chain-of-Emotion Appraisal (T1+ Ticks)

When the heartbeat has already escalated to Haiku or above, the appraisal is piggybacked as a structured output section in the existing inference call. No additional API call is required. The system prompt includes:

```
Before deciding on an action, appraise your emotional state given the
current situation.
Consider:
- How does this event relate to your strategy goals? (value appraisal)
- How much control do you have over the outcome? (control appraisal)
- Is this expected or surprising?
- How does it relate to your mortality situation?

Respond with:
<daimon>
  <pad p="[-1,1]" a="[-1,1]" d="[-1,1]" />
  <label primary="[plutchik]" intensity="[mild|moderate|intense]" />
  <trigger>[brief: what caused this emotion]</trigger>
  <control>[high|medium|low]: your perceived control over this situation</control>
  <value>[high|medium|low]: how much this matters to your goals</value>
</daimon>
```

Cost: approximately 25 additional output tokens per appraisal. At Haiku rates ($0.25/M output): $0.000006. Negligible.

---

## 7. Hybrid Mode (Default)

The default and recommended configuration is **hybrid mode**: rule-based appraisal for T0 ticks and Chain-of-Emotion for T1+ ticks. This ensures:

1. **Continuous mood tracking** via the free rule-based path. Mood never goes stale, even during periods of low market activity where the heartbeat stays at T0.
2. **Rich appraisals** when inference is already being paid for. The marginal cost of the Chain-of-Emotion structured output is approximately $0.000006 per appraisal at Haiku rates -- well within negligible.
3. **Grounding validation** where the deterministic appraisal serves as a sanity check on the LLM appraisal (see Section 8).

| Inference Tier   | Appraisal Mode                     | Cost       | Quality                                          |
| ---------------- | ---------------------------------- | ---------- | ------------------------------------------------ |
| T0 (rules only)  | Deterministic from outcome metrics | $0.00      | Coarse: correct direction, approximate magnitude |
| T1 (Haiku)       | Chain-of-Emotion piggybacked       | ~$0.000006 | Rich: attribution, control, mixed states         |
| T2 (Sonnet/Opus) | Chain-of-Emotion piggybacked       | ~$0.00002  | Richest: full contextual depth                   |

---

## 8. Grounding Validation

The critical design constraint: emotional states must be grounded in concrete metrics. Without this constraint, the LLM's tendency toward affective confabulation -- generating emotionally compelling narratives disconnected from actual outcomes -- would corrupt the daimon system [AFFECTIVE-HALLUCINATION-2025].

```rust
pub fn validate_appraisal(
    llm_appraisal: &AppraisalResult,
    deterministic_appraisal: &AppraisalResult,
    divergence_threshold: f32, // Default: 1.0 Euclidean distance in PAD space
) -> AppraisalResult {
    let distance = llm_appraisal.pad.distance(&deterministic_appraisal.pad);

    if distance > divergence_threshold {
        // LLM appraisal diverges too far from grounded metrics.
        // Override with deterministic appraisal, log the divergence.
        log_affective_divergence(llm_appraisal, deterministic_appraisal, distance);
        deterministic_appraisal.clone()
    } else {
        // LLM appraisal is within acceptable range -- use it (richer signal)
        llm_appraisal.clone()
    }
}
```

The divergence threshold of 1.0 in PAD Euclidean distance is deliberately generous -- it allows the LLM considerable latitude to produce fine-grained appraisals that differ from the coarse deterministic baseline, while catching gross hallucinations (reporting joy when P&L is deeply negative, or calm when survival pressure is critical).

### Grounding Rules

Four explicit grounding rules prevent affective hallucination:

1. **Joy requires a measurable positive outcome.** Profit, accuracy improvement, successful validation, gas optimization. The deterministic appraisal for joy always requires `pnl_delta > 0` or equivalent positive metric.
2. **Fear requires a measurable threat.** Loss, epistemic decay, hazard rate increase, survival pressure change. The deterministic appraisal for fear requires a negative metric or a mortality signal.
3. **Surprise requires a measurable deviation from prediction.** The prediction error must exceed a threshold for surprise to be the deterministic output.
4. **No emotion is generated without a triggering event in the same tick.** This is enforced at the appraisal trigger level (Section 3): if `should_trigger_appraisal()` returns false, no appraisal runs and no emotion is generated.

#### On-Chain Grounding via OutcomeVerification

The deterministic appraisal pipeline receives `OutcomeVerification` records (see `memory/prd/01-grimoire.md`) as a primary grounding signal. These records provide:

- **Pre/post state snapshots** via `readContract()` -- token balances, pool reserves, position details
- **Transaction receipts** -- success/revert status, gas used, emitted events
- **Deviation measurements** -- balance change in bps, gas deviation, unexpected logs

The four grounding rules remain, but verification is now backed by on-chain state reads rather than self-assessed P&L. The blockchain is the ground truth for emotional appraisal -- not the LLM's interpretation of its own performance.

LLMs cannot self-correct reasoning without external feedback (Huang et al., ICLR 2024, arXiv:2310.01798). On-chain state provides the external feedback that makes emotional grounding reliable.

---

## 9. Mood State: Temporal Smoothing

### 9.1 EMA Update

Mood is the slow-moving emotional baseline -- the sustained affective background that colors perception, modulates behavior, and decays toward the personality baseline.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodState {
    /// Current mood as PAD vector.
    pub pad: PADVector,
    /// Human-readable label from PAD octant mapping.
    pub label: String,
    /// How many ticks this mood has persisted.
    pub persistence_ticks: u64,
    /// Recent mood history for trajectory analysis.
    pub history: Vec<MoodSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodSnapshot {
    pub tick: u64,
    pub pad: PADVector,
    pub label: String,
}

pub fn update_mood(
    current_mood: &PADVector,
    appraisal: &AppraisalResult,
    decay_rate: f32, // Default: 0.95
) -> PADVector {
    PADVector {
        pleasure: decay_rate * current_mood.pleasure
            + (1.0 - decay_rate) * appraisal.pad.pleasure * appraisal.intensity,
        arousal: decay_rate * current_mood.arousal
            + (1.0 - decay_rate) * appraisal.pad.arousal * appraisal.intensity,
        dominance: decay_rate * current_mood.dominance
            + (1.0 - decay_rate) * appraisal.pad.dominance * appraisal.intensity,
    }
}
```

The decay rate of 0.95 means each new appraisal contributes only 5% to the mood vector (weighted by intensity). A single extreme event shifts mood slightly; a sustained pattern of similar events shifts it substantially. This captures the common-sense distinction between "one bad day" and "a bad week" -- the former is an emotion, the latter is a mood.

### 9.2 Mood Decay Toward Personality Baseline

Every 10 ticks, mood decays 5% toward the personality baseline. This prevents mood lock-in -- a Golem cannot be permanently depressed or permanently euphoric. After a period without significant emotional events, mood gradually returns to the Golem's natural affective center of gravity.

```rust
pub fn decay_toward_personality(
    current_mood: &PADVector,
    personality: &PADVector,
    decay_factor: f32, // Default: 0.05
) -> PADVector {
    PADVector {
        pleasure: current_mood.pleasure
            + decay_factor * (personality.pleasure - current_mood.pleasure),
        arousal: current_mood.arousal
            + decay_factor * (personality.arousal - current_mood.arousal),
        dominance: current_mood.dominance
            + decay_factor * (personality.dominance - current_mood.dominance),
    }
}
```

The asymmetry between the EMA update (5% per appraisal) and the personality decay (5% per 10 ticks) means that active emotional experience dominates the mood signal during periods of activity, while personality dominates during periods of calm. This is precisely the ALMA architecture's design intent [GEBHARD-2005]: emotions are the weather, mood is the climate, personality is the geography.

---

## 10. Appraisal-Coping Loop

Following EMA's architecture (Marsella and Gratch, 2009), emotions are not endpoints but triggers for coping responses [MARSELLA-GRATCH-2009]:

| Emotion                     | Coping Response        | Implementation                                                          |
| --------------------------- | ---------------------- | ----------------------------------------------------------------------- |
| **Fear** (high)             | Risk reduction         | Tighten stop-losses, reduce position sizes, defer new entries           |
| **Anger** (high)            | Active problem-solving | Escalate to T2, invoke counterfactual analysis, search for root cause   |
| **Joy** (sustained)         | Exploitation           | Increase position confidence, extend successful strategy duration       |
| **Sadness** (sustained)     | Strategic withdrawal   | Reduce activity, increase reflection frequency, consolidate knowledge   |
| **Surprise** (high)         | Information seeking    | Trigger additional probes, escalate to T2 for novel pattern analysis    |
| **Anticipation** (moderate) | Preparation            | Pre-compute scenarios, warm decision cache for expected events          |
| **Trust** (sustained)       | Deepening              | Increase confidence in validated heuristics, share knowledge with Clade |
| **Disgust** (moderate)      | Rejection              | Discard or downvote contradicted knowledge, search for alternatives     |

Coping responses are suggestions injected into reasoning context, not hard overrides. The PolicyCage remains the hard boundary.

### Exploration Temperature Modulation

```rust
pub fn exploration_temperature(mood: &PADVector, base_temp: f32) -> f32 {
    // Low pleasure + high arousal = frustrated/anxious --> explore more
    // High pleasure + low arousal = content/confident --> exploit more
    let exploration_bias = -mood.pleasure * 0.3 + mood.arousal * 0.2;
    (base_temp + exploration_bias).clamp(0.1, 2.0)
}
```

---

## 10b. SFSR: Emotional Depotentiation During Sleep

The Sleep to Forget, Sleep to Remember (SFSR) model [WALKER-VAN-DER-HELM-2009] shows that REM sleep selectively reduces the emotional charge of traumatic memories while preserving their informational content. After a bad trade, the Golem's dream cycle depotentiates the arousal component of that episode -- the Golem remembers what happened and learns from it, but no longer panics when it encounters similar conditions.

This is the mechanism by which the dream cycle (see `../05-dreams/`) serves emotional regulation, not just knowledge consolidation. The informational content of a loss episode persists in the Grimoire. The arousal spike that accompanied it fades through dream replay, preventing the episode from chronically triggering high-arousal states during future retrieval.

---

## 11. Events Emitted

Events are emitted through the Event Fabric as typed `GolemEvent` variants (see `14-events.md` in rewrite4 for the canonical enum).

| Event Variant | Trigger | Key Fields |
|---|---|---|
| `DaimonAppraisal` | Appraisal engine completes (step 8) | `pleasure`, `arousal`, `dominance`, `emotion`, `markers_fired`, `intensity` |
| `SomaticMarkerFired` | Somatic marker matches current situation (step 4) | `situation`, `valence`, `source`, `strategy_param` |
| `EmotionalShift` | Dominant Plutchik emotion changes | `from_emotion`, `to_emotion` |
| `MoodUpdate` | PAD Euclidean delta > 0.15 from last emission | `pleasure`, `arousal`, `dominance`, `mood_trend` |

---

## 12. References

- [AFFECTIVE-HALLUCINATION-2025] "Affective Hallucination in Large Language Models." arXiv:2508.16921, 2025. Identifies that LLMs can generate plausible-sounding emotional attributions without grounding in actual events; motivates the Daimon's grounding validation mechanism that rejects ungrounded appraisals.
- [BALDWIN-1896] Baldwin, J.M. "A New Factor in Evolution." _American Naturalist_, 30, 1896. Proposes the Baldwin Effect: phenotypic learning can guide genetic evolution by making certain genotypes more likely to survive; the theoretical basis for how Golem emotional learning during life shapes the inherited Grimoire for successors.
- [BECHARA-2000] Bechara, A., Damasio, H. & Damasio, A. "Emotion, Decision Making and the Orbitofrontal Cortex." _Cerebral Cortex_, 10(3), 2000. Extends the somatic marker hypothesis with skin conductance evidence showing anticipatory emotional signals precede conscious risk awareness; validates pre-cognitive bias as a decision mechanism.
- [CROISSANT-2024] Croissant, M. et al. "An Appraisal-Based Chain-of-Emotion Architecture for Affective Language Model Game Agents." _PLOS ONE_, 19(5), 2024. Demonstrates LLM-native appraisal by piggybacking emotion generation on existing inference calls; the direct architectural template for Chain-of-Emotion mode in the Daimon.
- [GEBHARD-2005] Gebhard, P. "ALMA: A Layered Model of Affect." In _AAMAS_, 2005. Proposes three temporal layers of affect (emotion, mood, personality) at different timescales; the model underlying the Daimon's three-layer temporal architecture.
- [HINTON-NOWLAN-1987] Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide Evolution." _Complex Systems_, 1, 1987. Shows computationally that individual learning can smooth the fitness landscape for evolutionary search; supports the argument that within-lifetime emotional learning improves cross-generational Grimoire inheritance.
- [KAHNEMAN-1979] Kahneman, D. & Tversky, A. "Prospect Theory: An Analysis of Decision Under Risk." _Econometrica_, 47(2), 1979. Establishes that losses loom larger than gains (loss aversion) and people evaluate outcomes relative to reference points; the basis for the asymmetric pleasure mapping where losses produce stronger negative pleasure than equivalent gains produce positive.
- [LINDENBAUER-2025] Lindenbauer, M. et al. "Agent-Generated Context as Noise." JetBrains Research, 2025. Argues that LLM agents suffer from context pollution where self-generated content degrades reasoning quality; motivates emotional salience as a filtering mechanism.
- [MARSELLA-GRATCH-2009] Marsella, S.C. & Gratch, J. "EMA: A Process Model of Appraisal Dynamics." _Cognitive Systems Research_, 10(1), 2009. Models appraisal as a dynamic process with coping loops that modify both the situation and the agent's interpretation; informs the Daimon's coping-potential assessment step.
- [OCC-1988] Ortony, A., Clore, G.L. & Collins, A. _The Cognitive Structure of Emotions_. Cambridge University Press, 1988. Defines 22 emotion types arising from appraisals of events (desirability), agents (praiseworthiness), and objects (appealingness); the primary theoretical model for the appraisal engine's event-to-emotion mapping.
- [PEKRUN-2006] Pekrun, R. "The Control-Value Theory of Achievement Emotions." _Educational Psychology Review_, 18, 315--341, 2006. Proposes that emotions arise from appraisals of control (can I influence this?) and value (does it matter?); provides the CVT-to-PAD mapping for control-to-dominance and value-to-pleasure dimensions.
- [SCHERER-2001] Scherer, K.R. "Appraisal Considered as a Process of Multilevel Sequential Checking." In _Appraisal Processes in Emotion_. Oxford University Press, 2001. Models appraisal as a sequence of checks (novelty, pleasantness, goal relevance, coping potential, norm compatibility); the template for the Daimon's 8-step sequential appraisal pipeline.
- [SELIGMAN-1972] Seligman, M.E.P. "Learned Helplessness." _Annual Review of Medicine_, 23, 1972. Demonstrates that repeated uncontrollable negative outcomes lead to learned helplessness (giving up despite available options); the basis for the Daimon's monitoring of sustained low-dominance states that could degrade into pathological passivity.
- [SIMON-1971] Simon, H.A. "Designing Organizations for an Information-Rich World." 1971. Argues that attention, not information, is the scarce resource; the original framing for emotions as attentional salience signals.
- [WALKER-VAN-DER-HELM-2009] Walker, M.P. & van der Helm, E. "Overnight Therapy? The Role of Sleep in Emotional Brain Processing." _Psychological Bulletin_, 135(5), 2009. Proposes the SFSR model where REM sleep strips arousal while preserving informational content; the basis for dream-triggered depotentiation of high-arousal appraisals.
- [BARTHET-2022] Barthet, M. et al. "Go-Blend: Affect-Driven Reinforcement Learning." _IEEE Transactions on Affective Computing_, 2022. Demonstrates that arousal-based state selection in RL agents improves exploration efficiency; validates the computational utility of emotion-driven behavior modulation.
- [COMINELLI-2015] Cominelli, L. et al. "SEAI: Social Emotional Artificial Intelligence Based on Damasio's Theory of Mind." _Frontiers in Robotics and AI_, 2, 2015. Implements Damasio's somatic markers in a cognitive robotics system and replicates Iowa Gambling Task results; validates the computational feasibility of the somatic marker architecture.

---
