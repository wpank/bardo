# Fast Feedback Loops: Five Evaluation Systems at Machine Speed [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Crate**: `golem-oracle` (calibration, cost-effectiveness, adversarial modules), `golem-grimoire` (context attribution), `golem-tools` (tool selection)
>
> **Depends on**: `./05-evaluation-lifecycle.md` (four-phase evaluation lifecycle from dev through production), `../03-daimon/09-evaluation.md` (Daimon affect engine evaluation and its integration with feedback loops)
>
> **Source**: `active-inference-research/new/15-fast-feedback-loops.md`

> **Reader orientation:** This document specifies five machine-speed evaluation loops that run inside each Golem (mortal autonomous agent): confidence calibration, context attribution, cost-effectiveness tracking, tool selection evaluation, and adversarial awareness. It belongs to Section 16 (Testing) and fills the gap between per-resolution residual correction and slow retrospective evaluation. All five loops are arithmetic, not LLM-based, and run at gamma or theta tick frequency with near-zero inference cost. See `prd2/shared/glossary.md` for full term definitions.

---

## Purpose

The prediction engine provides one fast feedback loop: residual correction at ~15,000 adjustments/day. The retrospective evaluator (see [10-retrospective-evaluation.md](10-retrospective-evaluation.md)) provides slow feedback at daily/weekly horizons. Between these, five evaluation gaps exist -- things the system should be learning from continuously but is not.

All five loops share the Karpathy property: **one metric, one arena, one gate (keep/discard), running fast**. They operate at gamma or theta frequency with near-zero inference cost. They are arithmetic, not LLM-based. They compound automatically.

| Loop | What It Evaluates | Metric | Frequency | Cost |
|------|------------------|--------|-----------|------|
| **Confidence Calibration** | Does the LLM's stated confidence match reality? | Expected Calibration Error (ECE) | Per-resolution | ~Zero |
| **Context Attribution** | Which Grimoire entries actually helped? | Attribution score per context piece | Per-theta-tick | ~Zero |
| **Cost-Effectiveness** | Is expensive inference worth the accuracy gain? | delta-accuracy per dollar | Per-theta-tick | ~Zero |
| **Tool Selection** | Did we use the right tool for the job? | Execution quality vs. alternatives | Per-action | Low (read-only simulation) |
| **Adversarial Awareness** | Is someone exploiting the golem's behavior? | Slippage excess, timing anomalies | Per-action | ~Zero |

---

## Loop 1: Confidence Calibration

### The Problem

When the LLM generates a prediction, it often states a confidence: "I'm 82% confident that the fee rate will stay above $1.50/hr." LLMs are systematically overconfident. Xiong et al. (2023) found that LLMs verbalize confidence in the 80-100% range even when accuracy is 50-60% [XIONG-2023]. This is consistent across model families. A 2026 study found that LLMs display a Dunning-Kruger pattern: overconfidence is worst precisely in the domains where the model performs worst [DUNNING-KRUGER-LLM-2026].

Without calibration, the action gate trusts a stated 80% confidence that corresponds to 55% actual accuracy.

### The Solution: Post-Hoc Confidence Calibration

A calibration module that learns the mapping from stated confidence to actual accuracy, per (category, regime), and applies a correction to every new prediction's confidence before it enters the Ledger.

```rust
/// Confidence calibration using isotonic regression.
/// Maps LLM-stated confidence to empirically calibrated confidence.
///
/// Isotonic regression is chosen over Platt scaling because:
/// 1. It makes no parametric assumptions about the calibration curve
/// 2. It is monotonic (higher stated -> higher calibrated, always)
/// 3. It handles the non-uniform distribution of LLM confidence
///    (clustered in 80-100%) gracefully
///
/// See [GUO-2017] for temperature scaling as an alternative.
/// See [DABAH-2025] for the interplay between calibration and
/// conformal prediction.
pub struct ConfidenceCalibrator {
    /// Per-(category, regime) calibration curves.
    curves: DashMap<(CategoryId, RegimeTag), IsotonicCurve>,

    /// Minimum sample size before calibration activates.
    min_samples: usize, // default: 30

    /// ECE per (category, regime).
    ece: DashMap<(CategoryId, RegimeTag), f64>,
}

/// An isotonic regression curve: a piecewise-constant monotonic function.
pub struct IsotonicCurve {
    bins: Vec<(f64, f64, f64)>,
    sample_count: u64,
}

impl ConfidenceCalibrator {
    /// Called on every prediction resolution (gamma frequency).
    pub fn record(&self, stated: f64, correct: bool, category: CategoryId, regime: RegimeTag) {
        let key = (category, regime);
        let mut curve = self.curves.entry(key).or_default();
        curve.add_sample(stated, if correct { 1.0 } else { 0.0 });

        if curve.sample_count % 50 == 0 {
            curve.refit();
            let ece = curve.compute_ece();
            self.ece.insert(key, ece);
        }
    }

    /// Called before prediction registration (theta frequency).
    pub fn calibrate(&self, stated: f64, category: CategoryId, regime: RegimeTag) -> f64 {
        let key = (category, regime);
        match self.curves.get(&key) {
            Some(curve) if curve.sample_count >= self.min_samples as u64 => {
                curve.map(stated)
            }
            _ => stated,
        }
    }

    /// ECE for a specific (category, regime). Lower is better.
    pub fn ece(&self, category: CategoryId, regime: RegimeTag) -> Option<f64> {
        self.ece.get(&(category, regime)).map(|v| *v)
    }
}
```

### Expected Calibration Error (ECE)

ECE is the standard metric for calibration quality [NAEINI-2015]:

$$\text{ECE} = \sum_{b=1}^{B} \frac{n_b}{N} |accuracy(b) - confidence(b)|$$

ECE of 0.0 means perfect calibration. ECE of 0.20 means the average confidence-accuracy gap is 20 percentage points. Most LLMs have ECE in the 0.15-0.30 range on factual QA [XIONG-2023].

### Integration Points

- **Action gate**: Uses calibrated confidence, not raw stated confidence.
- **Daimon**: A large ECE lowers the dominance dimension -- the golem "feels" less in control when it cannot assess its own certainty.
- **TUI**: The Oracle screen shows per-category ECE alongside accuracy.
- **Retrospective evaluator**: Weekly reviews include an ECE trend.

### Owner Configuration

```toml
[calibration]
enabled = true
min_samples = 30
refit_interval = 50
ece_alarm_threshold = 0.25
```

### Known Limitation

Isotonic regression requires the stated confidence distribution to have reasonable spread. If the LLM always says "90% confident," calibration degrades to a constant correction. Mitigation: the system prompts the LLM with explicit instructions to use the full 0-100% range.

---

## Loop 2: Context Attribution

### The Problem

Every escalated theta tick assembles a context window from Grimoire entries, PLAYBOOK.md (the agent's evolving strategy document) heuristics, predictions, and CorticalState (32-signal atomic shared perception surface) data. The system tracks whether the *prediction* was correct, but not whether the *context* was useful.

### The Solution: Lightweight Context Value Scoring

```rust
/// Tracks which context pieces co-occur with successful vs.
/// failed theta ticks. Builds a per-entry "context value" score.
///
/// Inspired by SHAP but drastically simplified for real-time operation.
pub struct ContextAttributor {
    entry_scores: DashMap<GrimoireEntryId, ContextScore>,
    heuristic_scores: DashMap<HeuristicId, ContextScore>,
    decay: f64, // Default: 0.995 per tick
}

pub struct ContextScore {
    pub credit: f64,
    pub debit: f64,
    pub appearances: u32,
    pub value: f64, // credit - debit, normalized by appearances
}

impl ContextAttributor {
    /// Called after every escalated theta tick resolution.
    pub fn attribute(
        &self,
        context_ids: &[ContextPieceId],
        outcome: TickOutcome,
    ) {
        let signal = match outcome {
            TickOutcome::Correct => 1.0,
            TickOutcome::Incorrect => -1.0,
            TickOutcome::Mixed(accuracy) => accuracy * 2.0 - 1.0,
        };

        for id in context_ids {
            let mut score = self.entry_scores.entry(*id).or_default();
            if signal > 0.0 {
                score.credit += signal;
            } else {
                score.debit += signal.abs();
            }
            score.appearances += 1;
            score.value = (score.credit - score.debit) / score.appearances as f64;
        }
    }

    /// Used by the Grimoire's retrieval ranking.
    pub fn retrieval_boost(&self, entry_id: &GrimoireEntryId) -> f64 {
        self.entry_scores.get(entry_id)
            .map(|s| 1.0 + s.value.clamp(-0.5, 0.5))
            .unwrap_or(1.0)
    }
}
```

### Integration

- **Grimoire retrieval**: `retrieval_boost` modifies ranking during context assembly.
- **Heuristic audit**: Feeds the retrospective evaluator's heuristic performance analysis.
- **TUI**: Grimoire screen can show entries sorted by context value.

### Known Limitation

Credit assignment problem: all context pieces get credit when a prediction succeeds, even if only a subset was relevant. Reliable for separating consistently helpful entries from consistently unhelpful ones over many ticks.

---

## Loop 3: Cost-Effectiveness

### The Problem

A T2 inference call costs $0.03. A T1 call costs $0.005. A T0 tick costs $0.00. The system tracks costs and accuracy independently but does not connect them.

### The Solution

```rust
/// Tracks accuracy gain per dollar of inference, per tier, per category.
pub struct CostEffectivenessTracker {
    tiers: DashMap<(CognitiveTier, CategoryId), TierEffectiveness>,
}

pub struct TierEffectiveness {
    pub accuracy_gained: f64,
    pub cost_usdc: f64,
    pub tick_count: u32,
    pub effectiveness: f64, // accuracy_gained / cost_usdc
}

impl CostEffectivenessTracker {
    pub fn record(&self, tier: CognitiveTier, category: CategoryId,
                  accuracy: f64, baseline_accuracy: f64, cost: f64) {
        let mut entry = self.tiers.entry((tier, category)).or_default();
        let delta = accuracy - baseline_accuracy;
        entry.accuracy_gained += delta.max(0.0);
        entry.cost_usdc += cost;
        entry.tick_count += 1;
        entry.effectiveness = if entry.cost_usdc > 0.0 {
            entry.accuracy_gained / entry.cost_usdc
        } else {
            0.0
        };
    }
}
```

### Integration

- **Inference routing**: If T2 is not proportionally better than T1 for a category, shift ticks to T1.
- **Mortality engine**: Cost-effectiveness feeds economic vitality.
- **TUI**: MIND > Inference screen shows cost-effectiveness per tier.

---

## Loop 4: Tool Selection Evaluation

### The Problem

The golem has 423+ DeFi tool adapters. When it swaps on Uniswap V3 instead of Aerodrome, was that the right venue?

### The Solution

```rust
/// After every swap execution, compare actual slippage against
/// what alternative venues would have quoted (read-only simulation).
pub struct ToolSelectionTracker {
    comparisons: DashMap<(ActionType, VenuePair), VenueComparison>,
}

pub struct VenueComparison {
    pub chosen_venue: String,
    pub chosen_metric: f64,
    pub alternative_metrics: Vec<(String, f64)>,
    pub was_optimal: bool,
}

impl ToolSelectionTracker {
    pub async fn evaluate_swap(
        &self,
        executed: &SwapResult,
        amount: U256,
        env: &dyn EnvironmentClient,
    ) {
        let alternatives = vec![
            ("aerodrome", self.quote_aerodrome(amount, env).await),
            ("uniswap_v3_0.3%", self.quote_uniswap(amount, 3000, env).await),
            ("uniswap_v3_0.05%", self.quote_uniswap(amount, 500, env).await),
        ];

        let was_optimal = alternatives.iter()
            .all(|(_, quote)| executed.slippage_bps <= quote.slippage_bps + 5);

        self.comparisons.entry((ActionType::Swap, executed.pair()))
            .or_default()
            .record(executed.venue.clone(), executed.slippage_bps as f64,
                    alternatives, was_optimal);
    }
}
```

### Detailed Evaluation Struct

```rust
pub struct VenueComparison {
    pub tick: u64,
    pub trade_size_usdc: f64,

    /// The venue that was chosen.
    pub chosen_venue: String,
    pub chosen_slippage_bps: f64,
    pub chosen_gas_cost_usdc: f64,

    /// Alternative venue quotes (read-only simulation).
    pub alternatives: Vec<AlternativeQuote>,

    /// Was the chosen venue optimal (within 5 bps tolerance)?
    pub was_optimal: bool,

    /// The best alternative, if the chosen venue was not optimal.
    pub best_alternative: Option<String>,
    pub savings_bps: f64,
}

pub struct AlternativeQuote {
    pub venue: String,
    pub slippage_bps: f64,
    pub gas_cost_usdc: f64,
    /// Total cost = slippage_bps + (gas_cost / trade_size * 10000).
    pub total_cost_bps: f64,
}
```

Over time, the evaluator builds a venue preference model per (pair, size_range):

```rust
/// Over time, build a preference model per (pair, size_range).
pub fn venue_preference(
    &self,
    pair: &TokenPair,
    size_range: (f64, f64),
) -> Option<String> {
    let comparisons = self.comparisons.get(&(ActionType::Swap, pair.clone()))?;
    let relevant: Vec<_> = comparisons.iter()
        .filter(|c| c.trade_size_usdc >= size_range.0 && c.trade_size_usdc <= size_range.1)
        .collect();

    if relevant.len() < 10 {
        return None; // Insufficient data.
    }

    // Count wins per venue.
    let mut wins: HashMap<String, u32> = HashMap::new();
    for c in &relevant {
        if c.was_optimal {
            *wins.entry(c.chosen_venue.clone()).or_default() += 1;
        }
    }

    wins.into_iter()
        .max_by_key(|(_venue, count)| *count)
        .map(|(venue, _count)| venue)
}
```

### Integration

- **Tool routing**: Builds preference model per (action_type, pair, size_range).
- **Prediction engine**: Feeds swap execution predictions.
- **TUI**: SOMA > Trades shows execution quality vs best alternative.

### Known Limitation

Alternative quotes are point-in-time reads and do not account for MEV on the alternative venue. The simulation does not account for MEV on the alternative venue -- a venue might have quoted 3 bps lower but a sandwich there would have added 15 bps. The comparison detects systematic routing suboptimality ("you consistently use Uniswap V3 0.3% when 0.05% is cheaper for this pair"), not individual trade mistakes.

---

## Loop 5: Adversarial Awareness

### The Problem

DeFi is adversarial. MEV bots, sandwich attackers, and oracle manipulators actively exploit predictable agent behavior.

### The Solution

```rust
/// Monitors for patterns that suggest adversarial exploitation.
pub struct AdversarialMonitor {
    slippage_excess: CircularBuffer<f64, 100>,
    sandwich_detector: SandwichDetector,
    resolution_integrity: ResolutionIntegrityChecker,
    slippage_excess_threshold_bps: f64, // default: 20 bps
    sandwich_frequency_threshold: f64,  // default: 0.15
}

impl AdversarialMonitor {
    pub async fn evaluate_action(
        &self,
        action: &ExecutedAction,
        prediction: &Prediction,
        env: &dyn EnvironmentClient,
    ) {
        // 1. Slippage excess: actual - predicted.
        if let Some(predicted_slip) = prediction.claim.center() {
            let actual_slip = action.actual_slippage_bps();
            let excess = actual_slip - predicted_slip;
            self.slippage_excess.push(excess);

            if self.slippage_excess.mean() > self.slippage_excess_threshold_bps {
                self.emit_alert(AdversarialAlert::PersistentSlippageExcess {
                    mean_excess_bps: self.slippage_excess.mean(),
                    sample_size: self.slippage_excess.len(),
                });
            }
        }

        // 2. Check for sandwich patterns in the block.
        let block = env.read_block(action.block_number).await;
        if self.sandwich_detector.check(action.tx_hash, &block) {
            self.emit_alert(AdversarialAlert::SandwichDetected {
                block: action.block_number,
                tx: action.tx_hash,
            });
        }
    }

    pub async fn check_resolution_integrity(
        &self,
        resolution: &Resolution,
        env: &dyn EnvironmentClient,
    ) {
        let twap = env.read_twap(resolution.item, Duration::from_secs(300)).await;
        let point = resolution.actual_value;
        let deviation = (point - twap).abs() / twap;

        if deviation > 0.02 {
            self.resolution_integrity.flag(resolution.id, deviation);
        }
    }
}
```

### Integration

- **Safety layer**: Persistent slippage excess triggers escalation: private mempool, increased gas priority, reduced trade sizes.
- **Daimon**: Adversarial detection raises arousal and lowers dominance.
- **TUI**: Persistent warning banner when adversarial patterns are detected.

### Owner Configuration

```toml
[adversarial]
enabled = true

# Slippage excess monitoring.
slippage_excess_threshold_bps = 20
slippage_buffer_size = 100

# Sandwich detection.
sandwich_frequency_threshold = 0.15
sandwich_sample_rate = 1.0         # Fraction of trades to check (1.0 = all).

# Resolution integrity.
resolution_twap_duration_secs = 300  # 5-minute TWAP.
resolution_twap_deviation_threshold = 0.02  # 2% deviation flags resolution.

# Auto-escalation.
auto_private_mempool = false
```

### Escalation Path

When slippage excess persists, the escalation path is:

1. Alert the owner via toast notification.
2. If `auto_private_mempool` is enabled, switch to Flashbots Protect or equivalent private transaction submission.
3. If excess persists after switching to private mempool, increase gas priority to reduce time in the mempool.
4. If still exploited, reduce trade sizes to make sandwiching unprofitable.

The Daimon receives adversarial alerts as arousal spikes. A Golem that is being consistently sandwiched "feels" threatened -- arousal rises, dominance drops. This biases future retrieval toward cautionary knowledge about execution risk.

### Known Limitation

Sandwich detection requires reading full block contents per transaction, adding RPC cost. Can be configured to sample (check every Nth trade via `sandwich_sample_rate`).

---

## References

- [XIONG-2023] Xiong, M. et al. (2023). "Can LLMs Express Their Uncertainty? An Empirical Evaluation of Confidence Elicitation in LLMs." arXiv:2306.13063. — *Shows LLMs are systematically overconfident (80-100% stated vs 50-60% actual accuracy), motivating the post-hoc confidence calibration loop.*
- [GUO-2017] Guo, C. et al. (2017). "On Calibration of Modern Neural Networks." ICML. — *Introduces temperature scaling for neural network calibration; referenced as an alternative to isotonic regression.*
- [NAEINI-2015] Naeini, M.P. et al. (2015). "Obtaining Well Calibrated Probabilities Using Bayesian Binning into Quantiles." AAAI. — *Defines Expected Calibration Error (ECE), the standard metric used throughout the calibration loop.*
- [DABAH-2025] Dabah, L. & Tirer, T. (2025). "On Temperature Scaling and Conformal Prediction of Deep Classifiers." ICML. arXiv:2402.05806. — *Explores the interplay between calibration and conformal prediction for reliable uncertainty quantification.*
- [KARPATHY-2026] Karpathy, A. (2026). "autoresearch." GitHub. — *Articulates the "one metric, one arena, one gate" property that all five fast feedback loops share.*
- [DUNNING-KRUGER-LLM-2026] (2026). "The Dunning-Kruger Effect in Large Language Models." arXiv:2603.09985. — *Finds LLMs display worst overconfidence precisely in their weakest domains, compounding the calibration problem.*
- [VOVK-2005] Vovk, V. et al. (2005). *Algorithmic Learning in a Random World*. Springer. — *Foundational work on conformal prediction that provides theoretical grounding for prediction interval construction.*
