# Retrospective Evaluation: The Slow Mirror [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Crate**: `golem-oracle` (retrospective module), `golem-grimoire`
>
> **Depends on**: `../01-golem/01-runtime.md` (Golem micro VM container and process model), `../05-oracle/01-prediction-engine.md` (prediction engine and T0/T1/T2 inference tier routing), `../04-memory/01-grimoire.md` (persistent knowledge base with episodes, insights, and heuristics), `../01-golem/05-mortality.md` (mortality clocks, behavioral phases, and epistemic vitality)
>
> **Source**: `active-inference-research/new/14-retrospective-evaluation.md`

> **Reader orientation:** This document specifies the retrospective evaluation system -- the "slow mirror" that evaluates decisions, positions, and heuristics in hindsight at daily/weekly/epoch horizons. It belongs to Section 16 (Testing) and answers questions the fast prediction engine cannot: "Was this decision good?" "Which PLAYBOOK.md (the agent's evolving strategy document) heuristics actually made money?" PnL attribution, position lifecycle analysis, and heuristic auditing are all defined here. See `prd2/shared/glossary.md` for full term definitions.

---

## The Gap This Fills

The prediction engine evaluates predictions at their scheduled checkpoints: "Was the fee rate above $1.50/hr at the 4-hour mark?" This answers whether a *specific claim* was accurate at a *specific moment*. It does not answer:

- **Was the decision good?** An LP entry that earned $6 in fees over 4 hours but suffered $14 in impermanent loss over 7 days was a bad decision -- even though the 4-hour fee prediction was correct.
- **Was the strategy working?** Prediction accuracy might be 78% while the portfolio is down 5% because the 22% of wrong predictions happened to be the expensive ones.
- **What should I have done differently?** Knowing what actually happened over the past week, what was the optimal set of actions? How far was the golem from optimal?
- **Which heuristics were actually load-bearing?** The PLAYBOOK.md has 30 heuristics. Which ones drove the decisions that made money? Which ones drove the decisions that lost money?

These are questions that can only be answered in hindsight, at time horizons much longer than any individual prediction checkpoint. Biology has an analog: the brain does not only learn from immediate feedback. During sleep, it re-evaluates the emotional valence of past events, reconsolidates memories with updated significance, and -- in humans -- engages in deliberate retrospective evaluation ("what would I have done differently?"). Robert Butler's life review [BUTLER-1963] and the psychological research on counterfactual thinking [ROESE-1997] establish that retrospective evaluation is a distinct cognitive function, not just delayed feedback.

The prediction engine is the fast mirror -- it shows you your reflection in real time, tick by tick. The retrospective evaluation system is the slow mirror -- it shows you your reflection after enough time has passed for the full consequences of your decisions to play out.

---

## Two Complementary Systems

| System | What It Evaluates | Timescale | Frequency | Cost |
|--------|------------------|-----------|-----------|------|
| **Prediction Engine** | Was this specific claim correct at this checkpoint? | Seconds to hours | Per-resolution (~15,000/day) | Near-zero (arithmetic) |
| **Retrospective Evaluator** | Was this decision/position/strategy good in hindsight? | Days to weeks | Per-review (~3-7/day) | T1 inference ($0.005-$0.01 per review) |

The two systems feed each other:
- The prediction engine produces the raw data (prediction outcomes, residuals, action records)
- The retrospective evaluator consumes that data at longer horizons and produces **retrospective insights** -- a new Grimoire entry type that the prediction engine cannot generate because it operates at the wrong timescale
- Retrospective insights feed back into the Daimon (emotional re-evaluation of past decisions), the Grimoire (updated episode significance), PLAYBOOK.md (heuristic validation/invalidation), and the prediction engine itself (long-horizon calibration corrections)

---

## The Retrospective Review

### What Triggers a Review

Reviews are triggered by time, not by events. Three review horizons run on independent schedules:

| Horizon | Name | When It Runs | What It Evaluates |
|---------|------|-------------|------------------|
| **Short** | Daily Review | Every 24 hours | Positions entered/exited yesterday. Actions taken. Predictions resolved. |
| **Medium** | Weekly Review | Every 7 days | Strategy effectiveness over the week. PnL attribution. Heuristic performance. |
| **Long** | Epoch Review | Every 30 days (or at death) | Full strategic assessment. Regime analysis. Generational learnings. |

Reviews also trigger on specific events:
- **Position close**: When a position is fully exited, a position-specific retrospective runs immediately, evaluating the position's entire lifecycle from entry to exit.
- **Significant loss**: When a single action's realized loss exceeds a configurable threshold (default: 5% of current balance), an immediate retrospective runs to understand what went wrong while the context is fresh.
- **Death**: The Thanatopsis (structured death protocol that generates a death testament for successor Golems) Protocol's Phase 2 (Reflect) is effectively a comprehensive epoch review. The retrospective evaluator feeds its accumulated findings into the death testament.

### What a Review Contains

Each review produces a `RetrospectiveReport` -- a structured evaluation that the LLM generates from assembled data. The LLM's role here is synthesis and explanation, not grading. The numerical grading is done by the Rust runtime from actual PnL data. The LLM explains *why* the numbers are what they are.

```rust
/// A retrospective evaluation at a specific time horizon.
/// Generated by LLM (T1/T2) from assembled performance data.
/// The numerical fields are computed by Rust from on-chain state.
/// The narrative fields are generated by the LLM to explain the numbers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrospectiveReport {
    pub id: ReportId,
    pub horizon: ReviewHorizon,
    pub period_start: u64,  // tick
    pub period_end: u64,    // tick
    pub generated_at: DateTime<Utc>,

    // ═══ HARD NUMBERS (computed by Rust, unfakeable) ═══

    /// Total PnL for the period, broken down by source.
    pub pnl: PnlAttribution,

    /// Prediction accuracy for the period (from the Ledger).
    pub prediction_accuracy: AccuracyReport,

    /// Actions taken during the period.
    pub actions_taken: Vec<ActionSummary>,

    /// Positions held during the period with per-position PnL.
    pub positions: Vec<PositionRetrospective>,

    /// Heuristics from PLAYBOOK.md that were cited in reasoning
    /// traces during this period, with their associated outcomes.
    pub heuristic_performance: Vec<HeuristicReport>,

    /// Cost breakdown: inference, gas, data.
    pub costs: CostBreakdown,

    // ═══ LLM-GENERATED NARRATIVE (synthesis, may not be perfectly faithful) ═══

    /// What went well and why.
    pub what_worked: String,

    /// What went wrong and why.
    pub what_failed: String,

    /// What the golem would do differently with hindsight.
    pub hindsight: String,

    /// Specific adjustments proposed for PLAYBOOK.md.
    pub playbook_proposals: Vec<PlaybookProposal>,

    /// Confidence in this review's conclusions.
    /// Lower for short horizons (less data), higher for long horizons.
    pub confidence: f64,
}
```

### PnL Attribution: Where the Money Went

The most important numerical component. PnL attribution breaks down returns by source over the review period, answering "why am I up/down?"

```rust
/// PnL broken down by source. All values in USDC.
/// Computed by the Rust runtime from on-chain position snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlAttribution {
    /// Total PnL for the period.
    pub total: f64,

    // ═══ REVENUE SOURCES ═══
    /// Fees earned from LP positions.
    pub lp_fees_earned: f64,
    /// Interest earned from lending.
    pub lending_interest_earned: f64,
    /// Staking rewards earned.
    pub staking_rewards: f64,
    /// Trading gains (buy low, sell high).
    pub trading_gains: f64,

    // ═══ COST SOURCES ═══
    /// Impermanent loss on LP positions.
    pub impermanent_loss: f64,
    /// Trading losses.
    pub trading_losses: f64,
    /// Gas costs for all on-chain transactions.
    pub gas_costs: f64,
    /// Inference costs (LLM calls).
    pub inference_costs: f64,
    /// Data costs (RPC calls, indexer queries).
    pub data_costs: f64,

    // ═══ UNREALIZED ═══
    /// Unrealized PnL on open positions (mark-to-market).
    pub unrealized_pnl: f64,

    // ═══ DERIVED ═══
    /// Net operational cost: gas + inference + data.
    /// This is the "cost of being alive."
    pub operational_cost: f64,
    /// Net yield: fees + interest + rewards - IL.
    /// This is what the strategy actually produced.
    pub net_yield: f64,
    /// Sharpe-like ratio: net_yield / volatility of daily returns.
    pub risk_adjusted_return: f64,
}
```

**Why this matters**: The prediction engine might report 78% accuracy while the portfolio is losing money. PnL attribution explains why: "Your fee rate predictions are accurate but your IL predictions underestimate losses in trending markets. The fees you earned ($12.40) were less than the IL you suffered ($18.20). Your net yield is -$5.80 despite 78% prediction accuracy."

This is the insight that prediction accuracy alone cannot provide. Accuracy treats all predictions equally. PnL attribution weights them by economic consequence.

### Position Retrospectives: The Full Lifecycle

When a position is closed, the retrospective evaluator computes its complete lifecycle assessment:

```rust
pub struct PositionRetrospective {
    pub position_id: PositionId,
    pub protocol: String,
    pub item: TrackedItemId,

    /// When the position was entered and exited.
    pub entry_tick: u64,
    pub exit_tick: u64,
    pub duration: Duration,

    /// The reasoning trace that justified entry.
    pub entry_reasoning_id: ReasoningTraceId,

    /// The predictions that justified entry, and their outcomes.
    pub entry_predictions: Vec<(PredictionId, Resolution)>,

    /// PnL at multiple checkpoints during the position's life.
    /// This is the "was this decision good?" time series.
    pub pnl_trajectory: Vec<(Duration, f64)>,
    // e.g., [(1h, +$0.80), (4h, +$2.10), (1d, +$1.40), (3d, -$0.90), (7d, -$4.20)]

    /// Final PnL at exit.
    pub final_pnl: f64,

    /// Attribution of the final PnL.
    pub attribution: PnlAttribution,

    /// Was the entry decision vindicated?
    /// Compares actual PnL trajectory against the inaction counterfactual.
    pub vs_inaction: f64,  // positive = position beat holding; negative = should have held

    /// The "regret score": how much better could optimal timing have done?
    /// max_pnl_during_lifetime - final_pnl.
    pub regret: f64,

    /// Computed optimal exit point (hindsight).
    pub optimal_exit_tick: Option<u64>,
    pub optimal_exit_pnl: Option<f64>,
}
```

The `pnl_trajectory` is the key data structure. It shows how the position's value evolved over time:

```
Position: Uniswap V3 ETH/USDC 0.3% LP (tight range 2380-2420)
Entry: Tick 1,204 | Exit: Tick 8,891 (7.2 days)

PnL Trajectory:
  +$3 ┤          ·····
  +$2 ┤     ····      ·····
  +$1 ┤····                 ···
   $0 ┤─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─·── ─ ─ ─ ─ ─ ─ ─
  -$1 ┤                           ···
  -$2 ┤                              ····
  -$3 ┤                                  ···
  -$4 ┤                                     ····  ← EXIT
      └───────────────────────────────────────────
      0h    1d      2d      3d     4d     5d    7d

  Entry prediction: +$1.60/hr fees, -$0.40/hr IL = +$1.20/hr net
  4h prediction accuracy: CORRECT (fees were $1.89/hr)
  7d actual: -$4.20 (IL dominated after ETH moved 8% out of range on day 4)

  vs. inaction: -$4.20 vs $0.00 = Should have held. Inaction was better.
  Regret: Peak was +$2.80 at 2.5 days. Optimal exit was day 2.5.
```

This retrospective is stored in the Grimoire as a high-value episode with a specific tag: `provenance: Retrospective`. It carries elevated retrieval weight because it contains hindsight knowledge -- the kind of information that can only be generated after the fact.

### Heuristic Performance: Which Rules Actually Helped?

Each PLAYBOOK.md heuristic has a name. When the LLM cites a heuristic in its reasoning trace (e.g., "Per heuristic H-7: concentrate LP when 7-day vol < 40%"), the retrospective evaluator tracks which heuristics were cited in decisions that led to good outcomes and which led to bad outcomes.

```rust
pub struct HeuristicReport {
    pub heuristic_id: String,
    pub heuristic_text: String,

    /// How many times this heuristic was cited in reasoning traces during the period.
    pub citation_count: u32,

    /// PnL of decisions that cited this heuristic.
    pub associated_pnl: f64,

    /// Win rate of decisions that cited this heuristic.
    pub win_rate: f64,

    /// Average PnL per citation.
    pub avg_pnl_per_citation: f64,

    /// Recommendation: keep, demote, or investigate.
    pub recommendation: HeuristicRecommendation,
}

pub enum HeuristicRecommendation {
    /// Heuristic is load-bearing: high citation count + positive PnL.
    Keep { reason: String },
    /// Heuristic is actively harmful: cited in losing decisions.
    Demote { reason: String },
    /// Heuristic is unused or inconclusive: low citations or mixed results.
    Investigate { reason: String },
    /// Not enough data to evaluate.
    InsufficientData,
}
```

This is how the golem evaluates its own learned rules at a strategic level. The prediction engine tells it whether individual predictions were accurate. The retrospective evaluator tells it whether the *heuristics that generated those predictions* were actually good strategy.

**Example output in the weekly review:**

```
HEURISTIC PERFORMANCE (7-day review)

  H-3: "Concentrate LP when 7d vol < 40%"
       Citations: 8 │ Win rate: 75% │ Avg PnL: +$1.40
       → KEEP: Load-bearing heuristic. Consistently profitable.

  H-7: "Increase position size when accuracy > 80%"
       Citations: 3 │ Win rate: 33% │ Avg PnL: -$2.10
       → DEMOTE: The 2 losses were large (-$4.80, -$3.60) and
         wiped out the 1 win (+$2.10). Overconfidence after
         high accuracy led to oversized positions.

  H-12: "Avoid Aerodrome LP when staking queue > 500"
       Citations: 0 │ Condition never triggered.
       → INSUFFICIENT DATA: Keep but cannot evaluate.

  H-15: "Exit LP when price moves >3% from range center"
       Citations: 5 │ Win rate: 60% │ Avg PnL: +$0.30
       → INVESTIGATE: Marginal. The exits that worked saved
         $2-3 each. The exits that were premature cost $1-2
         in missed fees. Tighten the threshold to 4%?
```

---

## Integration with Existing Systems

### Prediction Engine

The retrospective evaluator generates a new prediction type: **hindsight predictions**. After computing the optimal actions for the review period, the evaluator asks: "If the golem had followed the optimal strategy, what would the predictions have looked like?" The gap between actual predictions and optimal-hindsight predictions is a calibration signal for the residual corrector at long time horizons.

```rust
pub enum PredictionSource {
    Analytical { tier: CognitiveTier },
    Corrective { adjustment: AdjustmentType },
    Creative { phase: CreativePhase },
    Collective { source_golem: Option<GolemId>, layer: StyxLayer },
    Retrospective { horizon: ReviewHorizon },  // ← NEW
}
```

Retrospective predictions don't resolve against future reality -- they resolve against *past* reality that is already known. They exist to calibrate long-horizon prediction biases. "My 7-day LP PnL predictions are systematically 30% too optimistic because I underweight IL in trending markets." This is a bias the fast residual corrector can't catch because it operates on per-checkpoint residuals, not on full-lifecycle outcomes.

### Daimon (Emotional Re-Evaluation)

When a position retrospective reveals that a decision was worse than expected (negative `vs_inaction`), the Daimon receives a delayed negative signal. This updates the somatic marker for that type of decision: "LP positions in trending markets *feel* worse than they predicted" becomes an emotional association that biases future retrieval and gating.

This is the computational analog of what psychologists call **counterfactual emotion** [ROESE-1997]: the regret or relief you feel when you learn what would have happened if you'd made a different choice. The Daimon doesn't just respond to immediate outcomes -- it responds to retrospective evaluations of what *should* have happened.

```rust
impl Daimon {
    pub fn update_from_retrospective(&mut self, retro: &PositionRetrospective) {
        if retro.vs_inaction < -1.0 {
            // This position was significantly worse than holding.
            // Register a negative somatic marker for this type of decision.
            self.register_somatic_marker(SomaticMarker {
                trigger_context: retro.entry_context(),
                associated_outcome: OutcomeType::Regret,
                confidence: (retro.vs_inaction.abs() / 10.0).min(1.0),
                behavioral_bias: BehavioralBias::IncreaseCaution,
            });
        }

        if retro.regret > 2.0 {
            // Significant regret: could have exited much better.
            // This trains timing awareness, not entry avoidance.
            self.register_somatic_marker(SomaticMarker {
                trigger_context: retro.peak_context(),
                associated_outcome: OutcomeType::MissedExit,
                confidence: (retro.regret / 10.0).min(1.0),
                behavioral_bias: BehavioralBias::TightenExits,
            });
        }
    }
}
```

### Grimoire (Updated Episode Significance)

When the retrospective evaluator computes a position's full lifecycle PnL, it updates the original entry episode in the Grimoire with the retrospective outcome. An episode that was tagged as "successful trade" at the 4-hour checkpoint might be re-tagged as "ultimately harmful position" after the 7-day review.

This re-tagging changes the episode's retrieval weight. The Grimoire's affect-modulated retrieval will now surface this episode when the golem encounters similar conditions -- but with the *retrospective* assessment, not the *immediate* assessment. The golem remembers the full story, not just the beginning.

```rust
impl Grimoire {
    pub fn update_episode_from_retrospective(
        &mut self,
        episode_id: EpisodeId,
        retro: &PositionRetrospective,
    ) {
        let mut episode = self.get_episode(episode_id);

        // Update outcome with hindsight.
        episode.retrospective_pnl = Some(retro.final_pnl);
        episode.retrospective_vs_inaction = Some(retro.vs_inaction);
        episode.retrospective_regret = Some(retro.regret);

        // Adjust emotional valence based on hindsight.
        // An episode that felt positive at creation but was
        // retrospectively harmful gets its valence corrected.
        if retro.vs_inaction < -1.0 && episode.valence > 0.0 {
            episode.valence = -0.3; // Corrected to negative
            episode.tags.push("retrospective_reversal".into());
        }

        // Boost retrieval weight -- retrospective episodes are more
        // informative than immediate episodes because they contain
        // the full consequence of the decision.
        episode.retrieval_boost = 1.5;

        self.update_episode(episode);
    }
}
```

### Dreams (Retrospective Replay)

Retrospective reports feed the dream engine as high-value replay material. During NREM, the dream engine prioritizes replaying episodes with large retrospective reversals -- decisions that looked good at the time but turned out badly (or vice versa). These are the most instructive experiences because they reveal where the golem's real-time judgment was systematically wrong.

The dream engine also uses retrospective data for REM imagination: "If I encounter similar conditions again, what would the optimal strategy look like based on what I now know?" This generates counterfactual predictions grounded in actual hindsight, not in LLM speculation.

### Mortality (Strategic Death Clock)

The weekly and epoch reviews feed a new input to the epistemic vitality clock: **strategic effectiveness**. If the retrospective evaluator consistently finds that the golem's net yield is negative (the strategy is losing money after accounting for all costs), this is a signal that the golem's model of the world is wrong at the strategic level -- even if prediction accuracy is high.

A golem can have 80% prediction accuracy and still die of strategic failure if its *strategy* is wrong. Prediction accuracy measures whether the golem's model matches reality. Strategic effectiveness measures whether the golem's *goals* (as expressed in STRATEGY.md) are achievable in the current environment.

```rust
/// Strategic effectiveness score, computed from retrospective reviews.
/// Feeds into the epistemic vitality clock.
pub struct StrategicEffectiveness {
    /// Rolling 7-day net yield (fees + interest + rewards - IL - costs).
    pub net_yield_7d: f64,
    /// Rolling 7-day risk-adjusted return.
    pub sharpe_7d: f64,
    /// Fraction of positions where inaction would have been better.
    pub inaction_superiority_rate: f64,
    /// Composite score: higher = strategy is working.
    pub score: f64,
}
```

If `inaction_superiority_rate` exceeds 70% for two consecutive weekly reviews, the golem's strategic effectiveness drops sharply -- it would literally be better off doing nothing. This should either trigger the user to revise the strategy or, if ignored, accelerate epistemic death.

---

## Owner Visibility

### The Review Screen

A new tab in the FATE window: `FATE > Reviews`.

```
FATE > REVIEWS

  DAILY REVIEW (yesterday)
  ────────────────────────
  Period: Ticks 4,800-6,240 (24h)
  Actions: 3 entries, 1 exit, 47 holds
  Net PnL: +$1.82
    Fees earned:    +$4.20
    IL suffered:    -$1.40
    Gas costs:      -$0.62
    Inference:      -$0.36
  Prediction accuracy: 76.4%
  vs. Inaction: +$1.82 vs +$0.00 (strategy beat holding)

  [Enter] Full report  [D] Diff from last review

  WEEKLY REVIEW (7 days ago → today)
  ──────────────────────────────────
  Period: Ticks 1-6,240
  Actions: 12 entries, 4 exits, 289 holds
  Net PnL: +$8.41
    Best position:  Aave ETH supply (+$6.20, 6.2d held)
    Worst position: Aerodrome LP (-$4.20, 7.2d held, exited at loss)
  Strategy effectiveness: 0.72 (good)
  Inaction superiority: 23% (most actions were worthwhile)

  HEURISTIC AUDIT
  ▸ H-3: KEEP (8 citations, +$1.40 avg)
  ▸ H-7: DEMOTE (3 citations, -$2.10 avg)
  ▸ H-15: INVESTIGATE (5 citations, +$0.30 avg, marginal)

  [Enter] Full report  [H] Heuristic detail  [P] Position detail
```

### Position Lifecycle View

In SOMA > Trades, each closed position has a `[R]` key for its retrospective:

```
POSITION RETROSPECTIVE: Aerodrome ETH/USDC LP

  Lifecycle: 7.2 days (Tick 1,204 → 8,891)
  Entry reason: Fee rate anomaly (2.3x 7-day avg)
  Entry predictions: 3 registered, 2 correct at 4h checkpoint

  PnL TRAJECTORY
  +$3 ┤          ·····
  +$2 ┤     ····      ·····
  +$1 ┤····                 ···
   $0 ┤─────────────────────────────────────
  -$1 ┤                           ···
  -$2 ┤                              ····
  -$3 ┤                                  ···
  -$4 ┤                                     ···· EXIT
      └──────────────────────────────────────────
      0     1d     2d     3d    4d    5d    6d  7d

  HINDSIGHT ANALYSIS
  ──────────────────
  Peak PnL: +$2.80 at day 2.5
  Optimal exit: Day 2.5 (would have captured +$2.80)
  Actual exit: Day 7.2 at -$4.20
  Regret: $7.00 (difference between optimal and actual)

  What went wrong: ETH moved 8% out of range on day 4.
  The fee prediction was accurate for the first 3 days.
  IL prediction underestimated directional risk.

  Lesson: "In trending markets, tight-range LP positions
  should have a time-based exit at 48-72h even if fees
  are still accruing. IL risk compounds faster than fees."

  → This lesson has been proposed as PLAYBOOK.md candidate H-22.
    Confidence: 0.35. Needs validation from future positions.

  [Enter] View entry reasoning  [P] View predictions  [Esc] Back
```

---

## Configuration

```toml
[retrospective]
# Review schedules.
daily_review_enabled = true
weekly_review_enabled = true
epoch_review_enabled = true       # 30-day reviews.
epoch_interval_days = 30

# Position-close review.
position_close_review = true

# Immediate review on significant loss.
loss_review_threshold_pct = 5.0   # % of current balance.

# Inference tier for reviews.
# T1 by default. T2 for epoch reviews (deeper synthesis).
review_inference_tier = "T1"
epoch_inference_tier = "T2"

# PnL trajectory checkpoint intervals.
trajectory_checkpoint_intervals = ["1h", "4h", "1d", "3d", "7d", "14d", "30d"]

# Heuristic audit thresholds.
heuristic_min_citations = 3       # Min citations before evaluating.
heuristic_demote_threshold = -1.0 # Avg PnL below this -> DEMOTE.
heuristic_investigate_threshold = 0.5 # Avg PnL below this -> INVESTIGATE.

# Context attribution decay.
attribution_decay = 0.995          # Per-tick decay factor.

# Residual correction.
residual_buffer_size = 200         # Circular buffer capacity.
residual_min_samples = 30          # Min samples before correction activates.
conformal_quantile = 0.90          # Quantile for interval width.

# Confidence calibration.
calibration_min_samples = 30
calibration_refit_interval = 50    # Refit isotonic curve every N samples.
ece_alarm_threshold = 0.25         # Alert owner if ECE exceeds this.

# Cost-effectiveness.
tier_comparison_min_samples = 20   # Min samples per tier before comparing.
tier_switch_threshold = 1.5        # T1/T2 effectiveness ratio to trigger shift.

# Meta-learning.
meta_learning_enabled = true
meta_negative_weeks_before_hayflick = 2  # Weeks of negative score before acceleration.
meta_hayflick_acceleration = 0.10        # Per-week acceleration rate.

# Shadow strategies.
max_shadow_strategies = 3
shadow_min_duration_days = 7
shadow_promote_threshold = 0.10    # Shadow must outperform real by this fraction.
```

### Evaluator Garbage Collection

All evaluators use circular buffers (capacity 10K entries). When full, oldest entries are overwritten. For persistent analysis (meta-learning, shadow strategy), periodic snapshots are saved to the Grimoire as `EvaluationSummary` entries every delta cycle. This ensures that long-running evaluations (heuristic half-life, generational progress) survive the circular buffer's finite capacity while keeping memory usage bounded.

---

## Known Limitations

**Hindsight bias in narratives.** The LLM generates the "what went wrong" and "what I'd do differently" narratives. These are subject to hindsight bias -- the LLM knows the outcome and constructs a narrative that makes the outcome seem inevitable. The hard numbers (PnL trajectory, regret score, vs-inaction) are computed by Rust and are reliable. The narratives are useful for human readability but should not be treated as ground truth about causation.

**Attribution is approximate.** PnL attribution separates fees from IL from gas from inference. But some of these interact: gas cost determines when a position can be exited, which affects IL, which affects net yield. The attribution treats each component independently, which is a simplification.

**Optimal exit is only known in hindsight.** The "regret" metric compares actual PnL against the best possible exit during the position's lifetime. In real time, the golem couldn't have known the peak was the peak. The regret metric is useful for calibrating *exit timing heuristics* but should not be interpreted as "the golem should have known to exit here." The owner needs to understand this distinction.

**Weekly reviews cost inference.** Each weekly review is a T1 LLM call (~$0.005-$0.01) to synthesize the numerical data into a narrative. For a golem running 4 weekly reviews per month, this is $0.02-$0.04/month -- negligible. Epoch reviews use T2 (~$0.03) for deeper synthesis.

**Not all strategies have enough data.** A golem running a low-frequency strategy (1-2 trades per week) may not have enough position retrospectives for meaningful heuristic performance analysis until week 3-4. The system handles this by reporting "INSUFFICIENT DATA" until the minimum citation count is met.

---

## References

- [BUTLER-1963] Butler, R.N. "The life review: an interpretation of reminiscence in the aged." *Psychiatry*, 26(1), 65-76, 1963. — *Establishes that structured retrospective evaluation of past decisions is a distinct cognitive function, not just delayed feedback; the psychological basis for the epoch review and death testament.*
- [ROESE-1997] Roese, N.J. "Counterfactual thinking." *Psychological Bulletin*, 121(1), 133-148, 1997. — *Shows that "what if?" reasoning produces regret and relief emotions that improve future decisions; the basis for the vs-inaction comparison and the Daimon's counterfactual emotion updates.*
- [KAHNEMAN-TVERSKY-1979] Kahneman, D. & Tversky, A. "Prospect theory." *Econometrica*, 47(2), 263-292, 1979. — *Demonstrates that losses are weighted more heavily than equivalent gains; informs the asymmetric somatic marker registration where negative retrospectives produce stronger behavioral bias.*
- [RICHARDS-FRANKLAND-2017] Richards, B.A. & Frankland, P.W. "The persistence and transience of memory." *Neuron*, 94(6), 2017. — *Argues that forgetting is a feature, not a bug, of memory systems; supports the design where retrospective episode re-tagging can downweight initially positive episodes that proved harmful.*

---
