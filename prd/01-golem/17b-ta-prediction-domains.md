# 17b -- TA Prediction Domains [SPEC]

**Eight Technical Analysis Categories for the Oracle**

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-oracle` | **Layer**: 0 (FOUNDATION)
>
> Cross-references: [17-prediction-engine.md](./17-prediction-engine.md) (Oracle architecture), [18-cortical-state.md](./18-cortical-state.md) (TaCorticalExtension), [02-heartbeat.md](./02-heartbeat.md) (heartbeat pipeline)
>
> Sources: `ta/01` through `ta/09`, [03-ta-integration.md](../../tmp/research/witness-research/new/reconciling/03-ta-integration.md)

> **Reader orientation:** This document specifies eight technical analysis prediction categories that extend the Oracle's (the Golem's prediction engine) domain-agnostic `PredictionDomain` trait with TA-specific prediction capabilities. It belongs to the `01-golem` cognition layer, in the `golem-oracle` crate. The key concept: each TA subsystem's output maps to a falsifiable claim the Oracle can track, correct, and gate against -- manifold curvature, pattern codebook matches, causal discoveries, signal metabolism, adversarial detection, predictive geometry, entanglement, and somatic TA signals. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## What This File Covers

The Oracle's `PredictionDomain` trait is domain-agnostic. The TA research (papers ta/01 through ta/09) generates prediction capabilities that don't exist in the base DeFi domain. This file specifies eight new `PredictionCategory` values, each mapping a TA subsystem's output to a falsifiable claim the Oracle can track, correct, and gate against.

All eight categories register through a single `TaPredictionDomains` struct that implements `PredictionDomain`. Predictions in these categories flow through the same residual correction pipeline as fee-rate or price-direction predictions. The Oracle treats them identically.

---

## The TaPredictionDomains Implementation

```rust
/// Unified PredictionDomain for all TA-originated prediction categories.
/// Wraps the TaCorticalExtension and individual TA engine references
/// to produce predictions and resolve them against observed outcomes.
pub struct TaPredictionDomains {
    ta_extension: Arc<TaCorticalExtension>,
    pattern_codebook: Arc<TaPatternCodebook>,
    manifold: Arc<SpectralManifold>,
    causal_engine: Arc<CausalDiscoveryEngine>,
    signal_metabolism: Arc<SignalMetabolism>,
    adversarial_defense: Arc<AdversarialDefense>,
    predictive_geometry: Arc<PredictiveGeometry>,
    entanglement_tracker: Arc<EntanglementTracker>,
    somatic_engine: Arc<SomaticTaEngine>,
}

#[async_trait]
impl PredictionDomain for TaPredictionDomains {
    fn domain_id(&self) -> &str { "ta" }
    fn display_name(&self) -> &str { "Technical Analysis" }

    fn categories(&self) -> Vec<PredictionCategory> {
        vec![
            PredictionCategory::new(
                "manifold_curvature_sign",
                "Ricci scalar curvature predicts stability/instability",
            ),
            PredictionCategory::new(
                "topological_transition",
                "Betti number changes within predicted horizon",
            ),
            PredictionCategory::new(
                "causal_edge_strength",
                "Intervention effect size within predicted range",
            ),
            PredictionCategory::new(
                "pattern_match_outcome",
                "Matched pattern's historical outcome repeats",
            ),
            PredictionCategory::new(
                "entanglement_regime",
                "Cross-protocol entanglement exceeds threshold",
            ),
            PredictionCategory::new(
                "adversarial_detection",
                "Flagged observation confirmed as manipulation",
            ),
            PredictionCategory::new(
                "signal_speciation",
                "Signal speciates within predicted window",
            ),
            PredictionCategory::new(
                "somatic_accuracy",
                "Somatic marker bias produces better outcome",
            ),
        ]
    }

    async fn discover(
        &self,
        seed: &AttentionSeed,
        env: &dyn EnvironmentClient,
    ) -> Result<Vec<TrackedItem>> {
        // TA predictions are not item-specific in the same way DeFi
        // predictions are. A ManifoldCurvatureSign prediction concerns
        // the manifold at a particular position, not a particular pool.
        // Discovery produces one tracked item per TA subsystem that
        // has accumulated enough state to predict.
        todo!()
    }

    fn predict_scanned(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
    ) -> Vec<PredictionDraft> {
        // SCANNED tier: one prediction from the cheapest TA signal.
        // PatternMatchOutcome if the codebook has a match above 0.5,
        // ManifoldCurvatureSign otherwise (curvature is always available).
        todo!()
    }

    fn predict_watched(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
    ) -> Vec<PredictionDraft> {
        // WATCHED tier: 2-4 predictions from the most active TA signals.
        // Selection based on which subsystems have the highest recent
        // accuracy in the current regime.
        todo!()
    }

    fn predict_active(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
        playbook: &PlaybookState,
    ) -> Vec<PredictionDraft> {
        // ACTIVE tier: up to 8 predictions, one per category.
        // All categories produce predictions when the TA subsystem
        // has sufficient state.
        todo!()
    }

    async fn resolve(
        &self,
        prediction: &Prediction,
        checkpoint: &Checkpoint,
        env: &dyn EnvironmentClient,
    ) -> Result<ResolutionOutcome> {
        // Dispatch to the appropriate TA subsystem based on category.
        match prediction.category.id.as_str() {
            "manifold_curvature_sign" => self.resolve_curvature(prediction, checkpoint).await,
            "topological_transition" => self.resolve_topology(prediction, checkpoint).await,
            "causal_edge_strength" => self.resolve_causal(prediction, checkpoint).await,
            "pattern_match_outcome" => self.resolve_pattern(prediction, checkpoint).await,
            "entanglement_regime" => self.resolve_entanglement(prediction, checkpoint).await,
            "adversarial_detection" => self.resolve_adversarial(prediction, checkpoint).await,
            "signal_speciation" => self.resolve_speciation(prediction, checkpoint).await,
            "somatic_accuracy" => self.resolve_somatic(prediction, checkpoint).await,
            _ => Err(anyhow!("Unknown TA category: {}", prediction.category.id)),
        }
    }

    fn predict_action(
        &self, _: &ProposedAction, _: &TrackedItem, _: &ResidualBuffer,
    ) -> Vec<PredictionDraft> { vec![] }

    fn predict_inaction(
        &self, _: &TrackedItem, _: &ResidualBuffer, _: &CorticalSnapshot,
    ) -> Vec<PredictionDraft> { vec![] }
}
```

---

## Category Specifications

### 1. ManifoldCurvatureSign

The Riemannian liquidity manifold (ta/02) assigns a Ricci scalar curvature value to each point in the Golem's execution space. Positive curvature indicates a stable basin (nearby geodesics converge). Negative curvature indicates a saddle point or instability (geodesics diverge). The sign of curvature is a leading indicator: geometry warps before prices move.

**Claim type:** `PredictionClaim::Direction`

**Prediction:** "Curvature at the current manifold position will remain positive (stable) / flip negative (unstable) within N gamma ticks."

**Resolution:** Read `TaCorticalExtension::manifold_curvature` at checkpoint time. Compare the sign to the predicted sign.

```rust
impl TaPredictionDomains {
    fn predict_curvature(
        &self,
        cortical: &CorticalSnapshot,
        history: &ResidualBuffer,
    ) -> PredictionDraft {
        let current_curvature = f32::from_bits(
            self.ta_extension.manifold_curvature.load(Ordering::Relaxed)
        );
        // Predict sign persistence or flip based on curvature trend.
        // Positive and increasing -> predict stays positive.
        // Positive but decreasing toward zero -> predict flip.
        let trend = history.recent_trend("manifold_curvature_sign");
        let predicted_direction = if current_curvature > 0.0 && trend > -0.1 {
            Direction::Up // stable
        } else {
            Direction::Down // instability
        };

        PredictionDraft {
            category: CategoryId::ManifoldCurvatureSign,
            claim: PredictionClaim::Direction(predicted_direction),
            checkpoints: vec![
                Checkpoint::at_ticks_ahead(10),  // short-horizon
                Checkpoint::at_ticks_ahead(50),  // medium-horizon
            ],
            confidence: None, // calibrator fills this
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }

    async fn resolve_curvature(
        &self,
        prediction: &Prediction,
        checkpoint: &Checkpoint,
    ) -> Result<ResolutionOutcome> {
        let actual_curvature = f32::from_bits(
            self.ta_extension.manifold_curvature.load(Ordering::Relaxed)
        );
        let actual_sign = if actual_curvature > 0.0 {
            Direction::Up
        } else {
            Direction::Down
        };

        let correct = match &prediction.claim {
            PredictionClaim::Direction(predicted) => *predicted == actual_sign,
            _ => return Err(anyhow!("Wrong claim type for curvature")),
        };

        Ok(ResolutionOutcome {
            correct,
            actual_value: actual_curvature as f64,
            residual: actual_curvature as f64,
        })
    }
}
```

**Expected accuracy:** 65-75% after 200 observations. Curvature sign persistence is moderately predictable because geometric state is autocorrelated.

**CorticalState signal:** Reads `TaCorticalExtension::manifold_curvature`. Writer: `SpectralManifold::update_curvature()` at gamma frequency.

### 2. TopologicalTransition

The predictive geometry subsystem (ta/05) computes persistence landscapes and their derivatives. A topological transition occurs when Betti numbers change: beta_0 increasing means a cluster split, beta_1 increasing means a new cyclic pattern forming, either decreasing means a structural merger or collapse.

**Claim type:** `PredictionClaim::Boolean` (transition will / will not occur)

**Prediction:** "A topological transition (change in beta_0 or beta_1 at optimal scale) will / will not occur within N gamma ticks."

**Resolution:** Compare persistence diagrams at prediction time and checkpoint time. If the Betti numbers at optimal scale differ, a transition occurred.

```rust
impl TaPredictionDomains {
    fn predict_topology_transition(
        &self,
        cortical: &CorticalSnapshot,
    ) -> PredictionDraft {
        let change_rate = f32::from_bits(
            self.ta_extension.topology_change_rate.load(Ordering::Relaxed)
        );
        // High landscape derivative norm -> predict transition.
        let will_transition = change_rate > 0.3;

        PredictionDraft {
            category: CategoryId::TopologicalTransition,
            claim: PredictionClaim::Boolean(will_transition),
            checkpoints: vec![Checkpoint::at_ticks_ahead(20)],
            confidence: None,
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }
}
```

**Expected accuracy:** 55-65%. Topological transitions are inherently harder to predict than curvature persistence. The value is in the asymmetry: correct transition predictions are disproportionately valuable because they signal regime changes.

**CorticalState signal:** Reads `TaCorticalExtension::topology_change_rate`. Writer: `PredictiveGeometry::landscape_delta()` at gamma frequency.

### 3. CausalEdgeStrength

The causal discovery engine (ta/04) maintains a directed acyclic graph of causal relationships between protocol variables. An edge from variable A to variable B with strength s means "a 1-unit change in A causes an s-unit change in B, with lag L blocks." The prediction tests whether the discovered causal structure holds under intervention.

**Claim type:** `PredictionClaim::InRange`

**Prediction:** "If variable A changes by delta_A, variable B will change by [s * delta_A * lower_bound, s * delta_A * upper_bound] within L +/- tolerance blocks."

**Resolution:** Observe the actual change in variable B after the trigger event. Compare to predicted range.

```rust
impl TaPredictionDomains {
    fn predict_causal_effect(
        &self,
        trigger: &CausalTrigger,
        history: &ResidualBuffer,
    ) -> PredictionDraft {
        let edge = self.causal_engine.get_edge(
            &trigger.source_var, &trigger.target_var,
        );
        let predicted_effect = edge.strength * trigger.delta_source;
        let uncertainty = history.std_dev("causal_edge_strength")
            .unwrap_or(0.3);

        PredictionDraft {
            category: CategoryId::CausalEdgeStrength,
            claim: PredictionClaim::InRange {
                lower: predicted_effect * (1.0 - uncertainty),
                upper: predicted_effect * (1.0 + uncertainty),
            },
            checkpoints: vec![
                Checkpoint::at_blocks_ahead(edge.lag_blocks),
            ],
            confidence: None,
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }
}
```

**Expected accuracy:** 50-60%. Causal relationships in DeFi are noisy and regime-dependent. The prediction is valuable when the causal graph is sparse and edges are strong. Dense graphs with many weak edges produce low-confidence predictions that the calibrator correctly discounts.

**CorticalState signal:** Reads `TaCorticalExtension::causal_edge_count`. Writer: `CausalDiscoveryEngine::pc_update()` at theta frequency.

**Causal graph update cadence:** Partial correlation tests (Fisher's z-transform) run at theta frequency for fast edge discovery. Conditional mutual information tests run at delta frequency for thorough audits. The PC algorithm's incremental updates add or remove edges without recomputing the full graph.

### 4. PatternMatchOutcome

The HDC pattern codebook (ta/01, ta/06) matches the current market state against a library of stored patterns. Each pattern has a historical outcome distribution. The prediction tests whether the matched pattern's historical outcome repeats.

**Claim type:** `PredictionClaim::Direction`

**Prediction:** "The outcome following the best-matching pattern will be [Up/Down] within N gamma ticks, consistent with the pattern's historical outcome distribution."

**Resolution:** Observe actual price direction (or other relevant outcome) at checkpoint time.

```rust
impl TaPredictionDomains {
    fn predict_pattern_outcome(
        &self,
        cortical: &CorticalSnapshot,
    ) -> PredictionDraft {
        let match_score = f32::from_bits(
            self.ta_extension.pattern_match_score.load(Ordering::Relaxed)
        );

        if match_score < 0.6 {
            // No strong pattern match; skip prediction.
            return PredictionDraft::skip();
        }

        let best_match = self.pattern_codebook.best_match();
        let historical_direction = best_match.dominant_outcome();

        PredictionDraft {
            category: CategoryId::PatternMatchOutcome,
            claim: PredictionClaim::Direction(historical_direction),
            checkpoints: vec![
                Checkpoint::at_ticks_ahead(best_match.typical_horizon()),
            ],
            confidence: None,
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }
}
```

**Expected accuracy:** 55-65%. Pattern matching in financial markets is noisy. The HDC encoding provides fast retrieval (~10ns per comparison via XOR + POPCNT over 160 u64 words), and the pattern library self-curates through the signal ecosystem's replicator dynamics (ta/03, ta/06). Patterns with low accuracy are eliminated; survivors earn higher prediction weight.

**CorticalState signal:** Reads `TaCorticalExtension::pattern_match_score`. Writer: `TaPatternCodebook::match_patterns()` at gamma frequency.

### 5. EntanglementRegime

The entanglement tracker (ta/01) monitors cross-protocol correlation using HDC binding operations. When two previously independent protocols begin correlating (entanglement drift increases), it signals systemic risk. When correlated protocols decorrelate, it signals market fragmentation.

**Claim type:** `PredictionClaim::Above` (threshold claim)

**Prediction:** "Cross-protocol entanglement will exceed threshold T within N gamma ticks."

**Resolution:** Read `TaCorticalExtension::entanglement_drift` at checkpoint time. Compare to threshold.

```rust
impl TaPredictionDomains {
    fn predict_entanglement(
        &self,
        cortical: &CorticalSnapshot,
    ) -> PredictionDraft {
        let current_drift = f32::from_bits(
            self.ta_extension.entanglement_drift.load(Ordering::Relaxed)
        );
        // Predict whether drift will cross the systemic risk threshold.
        let threshold = 0.15; // calibrated from ta/01
        let will_cross = current_drift > 0.10
            && self.entanglement_tracker.drift_velocity() > 0.0;

        PredictionDraft {
            category: CategoryId::EntanglementRegime,
            claim: PredictionClaim::Above(threshold as f64),
            checkpoints: vec![Checkpoint::at_ticks_ahead(30)],
            confidence: None,
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }
}
```

**Expected accuracy:** 60-70%. Entanglement tends to be persistent once established (correlation clustering is autocorrelated), making positive predictions reliable. The harder prediction is the onset of entanglement from a decorrelated state.

**CorticalState signal:** Reads `TaCorticalExtension::entanglement_drift`. Writer: `EntanglementTracker::drift()` at gamma frequency.

### 6. AdversarialDetection

The adversarial defense (ta/08) screens observations against a manipulation prototype library using HDC matching. The prediction tests whether flagged observations are confirmed as manipulation by subsequent on-chain evidence (e.g., sandwich attack resolution, oracle manipulation unwinding).

**Claim type:** `PredictionClaim::Boolean`

**Prediction:** "This flagged observation is / is not adversarial manipulation."

**Resolution:** Check for on-chain evidence of manipulation within L blocks (sandwich resolution, MEV extraction, oracle attack unwinding).

```rust
impl TaPredictionDomains {
    fn predict_adversarial(
        &self,
        flagged_event: &FlaggedObservation,
    ) -> PredictionDraft {
        let fraction = f32::from_bits(
            self.ta_extension.adversarial_fraction.load(Ordering::Relaxed)
        );

        PredictionDraft {
            category: CategoryId::AdversarialDetection,
            claim: PredictionClaim::Boolean(true), // "this is adversarial"
            checkpoints: vec![
                // Check for confirmation within 10 blocks
                Checkpoint::at_blocks_ahead(10),
            ],
            confidence: None,
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }
}
```

**Expected accuracy:** 70-85%. Adversarial detection has a natural ground truth: sandwich attacks resolve within blocks, and the on-chain evidence is unambiguous. False positive rate (flagging benign activity) should stay below 10% for the system to be useful. The adversarial defense runs red-team dreaming during REM cycles (ta/08) to self-harden.

**CorticalState signal:** Reads `TaCorticalExtension::adversarial_fraction`. Writer: `AdversarialDefense::scan()` at gamma frequency.

### 7. SignalSpeciation

The signal metabolism (ta/03) tracks fitness of individual TA signals across DeFi context types. When a signal's accuracy diverges by more than 0.25 across two context types, it speciates: the parent forks into two children specialized for different contexts. The prediction tests whether a speciation event will occur.

**Claim type:** `PredictionClaim::Boolean` (occurrence claim)

**Prediction:** "Signal S will speciate within N theta ticks based on accuracy divergence trend."

**Resolution:** Check the signal metabolism's speciation log at checkpoint time.

```rust
impl TaPredictionDomains {
    fn predict_speciation(
        &self,
        signal_id: &SignalId,
        history: &ResidualBuffer,
    ) -> PredictionDraft {
        let divergence = self.signal_metabolism
            .accuracy_divergence(signal_id);
        // Predict speciation if divergence is trending toward 0.25 threshold.
        let will_speciate = divergence > 0.18
            && self.signal_metabolism.divergence_velocity(signal_id) > 0.0;

        PredictionDraft {
            category: CategoryId::SignalSpeciation,
            claim: PredictionClaim::Boolean(will_speciate),
            checkpoints: vec![Checkpoint::at_theta_ticks_ahead(10)],
            confidence: None,
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }
}
```

**Expected accuracy:** 60-70%. Speciation events are infrequent (a few per day at most) but predictable because the divergence trend is observable before the threshold is crossed. The value lies in anticipating population changes that affect the signal ecosystem's compute budget allocation.

**CorticalState signal:** Reads `TaCorticalExtension::signal_ecosystem_fitness`. Writer: `SignalMetabolism::replicator_step()` at theta frequency.

### 8. SomaticAccuracy

The somatic TA engine (ta/09) retrieves affect from the somatic map when a pattern fires. The retrieved affect biases the Daimon's attention and risk tolerance. The prediction tests whether following the somatic bias produces a better outcome than ignoring it.

**Claim type:** `PredictionClaim::Direction`

**Prediction:** "The somatic marker's directional bias (approach / avoid) will produce a better outcome than the neutral alternative."

**Resolution:** Compare the outcome of the action taken under somatic bias to a counterfactual estimate of the outcome without bias. The counterfactual is computed from the Oracle's inaction predictions for the same item.

```rust
impl TaPredictionDomains {
    fn predict_somatic(
        &self,
        marker_retrieval: &SomaticRetrieval,
        cortical: &CorticalSnapshot,
    ) -> PredictionDraft {
        let intensity = f32::from_bits(
            self.ta_extension.somatic_intensity.load(Ordering::Relaxed)
        );

        if intensity < 0.3 {
            return PredictionDraft::skip();
        }

        let bias_direction = marker_retrieval.dominant_valence();

        PredictionDraft {
            category: CategoryId::SomaticAccuracy,
            claim: PredictionClaim::Direction(bias_direction),
            checkpoints: vec![
                Checkpoint::at_ticks_ahead(20),
                Checkpoint::at_ticks_ahead(100),
            ],
            confidence: None,
            source: PredictionSource::Analytical {
                tier: CognitiveTier::T0,
            },
        }
    }
}
```

**Expected accuracy:** 50-60%. Somatic markers encode noisy associations between patterns and outcomes. The value is not in high accuracy but in the accumulation effect: over hundreds of decisions, a slight bias toward historically correct gut feelings compounds into measurably better performance. Damasio (1994) demonstrated this for human somatic markers; the mechanism translates to the artificial case.

**CorticalState signal:** Reads `TaCorticalExtension::somatic_intensity`. Writer: `SomaticTaEngine::retrieve()` at gamma frequency.

---

## Interaction with Signal Metabolism

The signal metabolism (ta/03) tracks accuracy for all TA signals through the Oracle's prediction ledger. The replicator dynamics equation governs budget share:

```
dx_i/dt = x_i * (W_i - W_bar) * selection_pressure
```

Where `W_i = alpha * accuracy_i + beta * info_gain_i - gamma * normalized_cost_i`. The accuracy term comes from the Oracle's per-category accuracy reports. This closes the loop: TA signals produce predictions, the Oracle tracks their accuracy, the metabolism adjusts compute budget based on accuracy, and better-funded signals produce better predictions.

The fitness function uses the Oracle's accuracy reports rather than maintaining its own accuracy tracking. Per conflict resolution (Conflict 1): the Oracle is canonical for prediction accuracy.

---

## Hebbian Weight Updates and the Daimon

Each TA signal has a weight vector across DeFi context types. Weights update on every prediction resolution:

```
delta_w_ij = eta_effective * activation_i * (2 * outcome_j - 1)
```

The learning rate `eta_effective` modulates through the Daimon's PAD vector:

| Affect state | PAD region | eta_effective | Rationale |
|---|---|---|---|
| Fear | Low P, high A, low D | `eta_base * 3.0` | Rapid downweighting of losing signals (McGaugh, 2004) |
| Calm confidence | Moderate P, low A, moderate D | `eta_base * 0.5` | Slow learning prevents overreaction |
| Surprise | Neutral P, high A, low D | `eta_base * 2.0` | Accelerated learning, less aggressive than fear |

The Daimon reads the PAD vector from CorticalState. The metabolism engine reads PAD through the same path. No new coupling is needed.

---

## Inheritance at Death

At death, Thanatopsis exports the top-fitness TA signal configurations:

```rust
/// TA prediction state that crosses the generational boundary.
/// Included in OracleGenome alongside DeFi residual stats.
pub struct TaGenome {
    /// Top-fitness signal function identifiers and parameters.
    pub signals: Vec<InheritedSignal>,
    /// Context weight vectors (which DeFi primitives each signal predicts).
    pub context_weights: Vec<Vec<f64>>,
    /// Per-category accuracy from the Oracle's ledger.
    pub category_accuracies: HashMap<CategoryId, f64>,
    /// Lineage: parent signal IDs, generation depth.
    pub lineage: Vec<SignalLineage>,
}

pub struct InheritedSignal {
    pub evaluator_type: String,
    pub parameters: Vec<f64>,
    pub fitness: f64,        // inherited at 0.5x
    pub generation: u32,
}
```

The successor loads inherited signals at 0.5x fitness. Signals that were dominant in the predecessor start with an advantage but must re-earn their position. This is the Baldwin Effect: behavioral adaptations become structural defaults in the next generation (Baldwin, 1896).

---

## References

- [ITTI-2009] Itti, L. & Baldi, P. "Bayesian Surprise Attracts Human Attention." *Vision Research*, 49(10), 2009. — *Shows that attention is drawn to observations that maximally shift beliefs (KL divergence between posterior and prior); the surprise metric used by TA prediction domains to flag unusual market structure.*
- [BALDI-2010] Baldi, P. & Itti, L. "Of Bits and Wows: A Bayesian Theory of Surprise." *Neural Networks*, 23(5), 2010. — *Axiomatizes Bayesian surprise as the unique attention-worthy measure depending on the agent's model class; provides the formal definition the SomaticAccuracy domain uses for prediction scoring.*
- [GIDEA-2018] Gidea, M. & Katz, Y. "Topological Data Analysis of Financial Time Series." *Physica A*, 491, 2018. — *Applies persistent homology to detect topological regime changes in financial markets; validates the TopologicalTransition domain's use of persistence diagrams for DeFi regime detection.*
- [MCGAUGH-2004] McGaugh, J.L. "The Amygdala Modulates the Consolidation of Memories." *Annual Review of Neuroscience*, 27, 2004. — *Shows that emotionally arousing events are remembered better due to amygdala-mediated consolidation; the biological model for why high-arousal market events receive stronger Grimoire encoding via the Daimon affect engine.*
- [DAMASIO-1994] Damasio, A.R. *Descartes' Error: Emotion, Reason, and the Human Brain*. Putnam, 1994. — *Argues that somatic markers (body-state signals) are necessary for rational decision-making; the neuroscience basis for the SomaticAccuracy domain, which tracks whether the Daimon's affect signals actually predict outcomes.*
- [BALDWIN-1896] Baldwin, J.M. "A New Factor in Evolution." *American Naturalist*, 30(354), 1896. — *Proposes the Baldwin Effect where learned behaviors become inherited defaults over generations; the evolutionary mechanism that the PatternMatchOutcome domain exploits as successful prediction heuristics solidify into structural defaults across Golem generations.*

---
