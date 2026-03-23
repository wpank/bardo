# Slow Feedback Loops: Evaluation That Requires Time and Thought [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Crates**: `golem-oracle` (shadow, meta-learning), `golem-grimoire` (reasoning review), `golem-dreams` (reasoning replay)
>
> **Depends on**: `./05-evaluation-lifecycle.md` (four-phase evaluation lifecycle from dev through production), `./07-fast-feedback-loops.md` (five machine-speed evaluation loops that feed data into the slow loops), `./10-retrospective-evaluation.md` (daily/weekly/epoch hindsight evaluation of decisions and heuristics)
>
> **Source**: `active-inference-research/new/16-slow-feedback-loops.md`

> **Reader orientation:** This document specifies three slow evaluation loops that require either significant time horizons or LLM-assisted reasoning: shadow strategy testing, reasoning quality review, and meta-learning evaluation. It belongs to Section 16 (Testing) and complements the fast feedback loops in `./07-fast-feedback-loops.md`. These loops evaluate whether the Golem's (mortal autonomous agent's) learning processes themselves are improving, not just domain performance. See `prd2/shared/glossary.md` for full term definitions.

---

## Purpose

The fast feedback loops (see [07-fast-feedback-loops.md](07-fast-feedback-loops.md)) run at machine speed -- per-resolution, per-tick, near-zero cost. They handle things that can be evaluated with arithmetic.

This document covers three evaluation loops that are inherently slow because they require either significant time horizons or LLM-assisted reasoning:

| Loop | What It Evaluates | Timescale | Cost | Why It's Slow |
|------|------------------|-----------|------|--------------|
| **Shadow Strategy Testing** | Would different parameters produce better outcomes? | Weekly | Near-zero | Needs a full week of counterfactual data |
| **Reasoning Quality Review** | Is the golem's reasoning consistent and sound? | Per-dream-cycle | T1 ($0.005-0.01) | Requires LLM to review reasoning traces |
| **Meta-Learning Evaluation** | Is the learning process itself improving? | Weekly + generational | Near-zero | Convergence rates need weeks to be meaningful |

---

## Loop 6: Shadow Strategy Testing

### The Problem

The user tweaks parameters but has no way to know if the new configuration is better without waiting days for real results. Shadow testing eliminates the wait by running counterfactual configurations alongside the real one.

### How It Works

On every theta tick, after real execution completes, the shadow system re-runs the gating decision with a perturbed parameter set. It does not execute anything -- it records the counterfactual decision and registers shadow predictions.

```rust
/// Shadow strategy tester. Runs counterfactual parameter sets
/// alongside the real configuration.
pub struct ShadowTester {
    /// Maximum 3 concurrent shadows, each varying one parameter.
    shadows: Vec<ShadowConfig>,
    results: Vec<ShadowResult>,
}

pub struct ShadowConfig {
    pub id: ShadowId,
    pub name: String,              // e.g., "Lower action gate (0.50)"
    pub parameter: String,         // e.g., "prediction.action_gate_threshold"
    pub real_value: f64,
    pub shadow_value: f64,
    pub started_at_tick: u64,
}

pub struct ShadowResult {
    pub shadow_id: ShadowId,
    pub shadow_acted_real_didnt: Vec<ShadowAction>,
    pub real_acted_shadow_didnt: Vec<ShadowAction>,
    pub shadow_accuracy: AccuracyReport,
    pub shadow_pnl: f64,
}

pub struct ShadowAction {
    pub tick: u64,
    pub action: ProposedAction,
    pub simulated_outcome: SimulatedOutcome,
}

impl ShadowTester {
    pub fn evaluate_tick(&mut self, tick_ctx: &ThetaContext) {
        for shadow in &mut self.shadows {
            let shadow_gate = tick_ctx.oracle.gate_with_override(
                tick_ctx.cortical,
                &tick_ctx.appraisal,
                &shadow.parameter,
                shadow.shadow_value,
            );

            match (tick_ctx.real_gate, shadow_gate) {
                (GateDecision::Suppress, GateDecision::Permit) => {
                    let sim = tick_ctx.simulate_action(&tick_ctx.deliberation);
                    shadow.results.shadow_acted_real_didnt.push(ShadowAction {
                        tick: tick_ctx.tick,
                        action: tick_ctx.deliberation.proposed_action.clone(),
                        simulated_outcome: sim,
                    });
                }
                (GateDecision::Permit, GateDecision::Suppress) => {
                    shadow.results.real_acted_shadow_didnt.push(ShadowAction {
                        tick: tick_ctx.tick,
                        action: tick_ctx.execution.action.clone(),
                        simulated_outcome: SimulatedOutcome::Held,
                    });
                }
                _ => {}
            }
        }
    }
}
```

### Resolution and Reporting

Shadow predictions resolve on the same schedule as real predictions. After one week, the weekly retrospective includes a shadow comparison:

```
SHADOW STRATEGY COMPARISON (7-day)

  REAL CONFIG: action_gate_threshold = 0.60
  SHADOW A:    action_gate_threshold = 0.50

  Real: 4 trades executed, net PnL +$3.20, accuracy 74%
  Shadow A: 9 trades would have executed, simulated PnL +$1.80, accuracy 68%

  Analysis: Shadow A would have traded more but earned less.
  The higher threshold filtered out 5 marginal trades that
  would have been net negative. Current threshold is better.
```

### Owner Interaction

The user sets up shadow tests through the COMMAND > Settings > Experiments screen:

```
SHADOW EXPERIMENTS (max 3 concurrent)

  [+] New experiment

  Name: "More aggressive gating"
  Parameter: prediction.action_gate_threshold
  Current value: 0.60
  Shadow value: 0.50
  Duration: 7 days

  [Enter] Start  [Esc] Cancel
```

Results appear in the FATE > Reviews screen after the experiment duration elapses. The user sees concrete data: "If I had used these parameters, here's what would have happened."

### Known Limitations

**Simulation is not execution.** Shadow PnL is simulated from read-only quotes. Real execution has slippage, gas costs, MEV impact, and timing effects that simulation cannot capture.

**Only one parameter at a time.** Testing combinations requires exponential shadow configurations. The architecture limits to 3 concurrent single-parameter shadows.

**Confirmation bias.** Users may set up shadows that confirm what they already want to do. Mitigation: the system could suggest shadow experiments based on areas where the golem is underperforming, but this is a v2 feature.

---

## NREM Residual Replay (Consolidation-Speed Loop)

During NREM sleep, the dream engine replays the 50 largest residuals from the current waking period. "Largest" means the predictions where the Golem was most wrong -- the biggest gap between predicted and actual. This is not the same as per-resolution correction (which corrects the mean bias). NREM replay examines the outliers -- the specific predictions that were far from the mean, looking for patterns in what made them outliers.

```rust
/// NREM replay: examine the largest prediction errors from the
/// current waking period, looking for systematic patterns.
///
/// Runs during each NREM phase of the dream cycle.
/// Produces pattern extractions that feed the Grimoire and
/// the corrector's regime-specific buffers.
pub struct NremReplayEngine {
    /// How many residuals to replay per NREM phase.
    top_n: usize, // default: 50

    /// Minimum residual magnitude to consider for replay.
    /// Prevents replaying noise.
    min_residual: f64, // default: 0.5 (USDC)
}

pub struct NremReplayResult {
    /// Residuals replayed during this NREM phase.
    pub replayed: Vec<ReplayedResidual>,

    /// Patterns detected across the replayed residuals.
    /// Generated by the T1 LLM from the assembled residual data.
    pub patterns: Vec<DetectedPattern>,

    /// Proposed corrections: regime-specific bias adjustments
    /// that the corrector should apply.
    pub proposed_corrections: Vec<ProposedCorrection>,
}

pub struct ReplayedResidual {
    pub prediction_id: PredictionId,
    pub category: CategoryId,
    pub regime: RegimeTag,
    pub predicted: f64,
    pub actual: f64,
    pub residual: f64,
    /// The context that was assembled for this prediction.
    pub context_summary: String,
    /// The reasoning trace that produced this prediction.
    pub reasoning_summary: String,
}

pub struct DetectedPattern {
    /// Human-readable description of the pattern.
    pub description: String,
    /// Which residuals exhibited this pattern.
    pub evidence: Vec<PredictionId>,
    /// Confidence that this is a real pattern (not noise).
    pub confidence: f64,
    /// Proposed Grimoire entry if confidence is high enough.
    pub proposed_entry: Option<GrimoireEntry>,
}

pub struct ProposedCorrection {
    pub category: CategoryId,
    pub regime: RegimeTag,
    /// The bias to apply: add this to future predictions.
    pub bias_adjustment: f64,
    /// The evidence supporting this correction.
    pub evidence_count: usize,
    pub confidence: f64,
}
```

The replay looks for clusters in the outliers. If 8 of the 50 largest residuals are fee_rate predictions in a bull regime, and all 8 over-predicted, there is a systematic pattern: the Golem overestimates fees in bull markets. This signal the per-resolution corrector might miss because the mean residual across all fee_rate predictions could be near zero (the bull-market over-predictions average out with bear-market under-predictions).

The LLM's role during NREM replay is pattern description, not pattern detection. The Rust runtime identifies the clusters (8/50 residuals share a category and regime and have the same sign). The LLM explains why the pattern might exist: "Fee predictions in bull markets are too high because the model doesn't account for the fee compression that occurs when many LPs enter the same pools during bull runs."

NREM replay prompt template: "You are reviewing predictions from the past [N] theta ticks. Here are [K] prediction-outcome pairs with the largest residuals: [data]. Identify patterns: are there systematic biases? Recurring error types? Suggest one concrete adjustment to the prediction approach." System instruction: respond in structured JSON with fields `patterns`, `bias_direction`, `suggested_adjustment`.

### REM Counterfactual Generation

During REM sleep, the dream engine generates creative predictions -- novel hypotheses about the environment that the Golem has not yet tested. The false discovery rate (FDR) problem is severe. LLMs are confident confabulators. The FDR control mechanism requires 3 independent confirmations before promoting a creative prediction to the environmental model:

1. The creative prediction must resolve correctly at its checkpoint.
2. The residual corrector must show reduced error in the relevant category after the creative prediction is incorporated.
3. A second, independent dream cycle must produce a compatible hypothesis (not the same wording, but a hypothesis that implies the same directional claim).

If all three conditions are met, the creative prediction is promoted from `PredictionSource::Creative` to a Grimoire entry tagged `provenance: Confirmed_Creative`. If any fails, the prediction is discarded.

---

## Loop 7: Reasoning Quality Review

### The Problem

Every T1/T2 theta tick produces a reasoning trace. But reasoning traces are never evaluated for quality independent of outcome. A prediction can be correct by luck (fragile), or incorrect while the reasoning was sound (valuable).

LLMs cannot reliably grade their own reasoning [CHEN-2025], so self-grading is not an option. But specific quality signals can be detected without self-grading.

### Three Reasoning Quality Signals

All three are evaluated during dream cycles, not in real time.

#### Signal 1: Outcome-Reasoning Alignment

```rust
/// Evaluated during NREM replay.
pub struct ReasoningAlignmentCheck {
    pub trace_id: ReasoningTraceId,
    pub prediction_id: PredictionId,
    pub outcome: Resolution,
    pub alignment: ReasoningAlignment,
}

pub enum ReasoningAlignment {
    /// Correct prediction, reasoning consistent. Most valuable.
    AlignedCorrect,

    /// Incorrect prediction, reasoning wrong too. Standard learning case.
    AlignedIncorrect,

    /// Correct prediction, reasoning inconsistent. FRAGILE.
    /// Flag for dream review -- the reasoning should not be trusted.
    MisalignedCorrect { explanation: String },

    /// Incorrect prediction, reasoning was sound but world was noisy. VALUABLE.
    /// Reduce confidence penalty -- do not overtrain on noise.
    MisalignedIncorrect { explanation: String },
}
```

#### Reasoning Trace Format

```json
{
  "trace_id": "uuid",
  "tick": 12345,
  "category": "PriceDirection",
  "prediction_id": "uuid",
  "reasoning": "ETH/USDC 4h MA crossing above 20h MA suggests...",
  "confidence": 0.72,
  "inference_tier": "T1",
  "duration_ms": 1250,
  "token_count": 340
}
```

The MisalignedCorrect cell is the most dangerous. A Golem that is right for wrong reasons accumulates false confidence. Its stated confidence rises (the predictions were correct), its accuracy metrics look good, but the underlying reasoning is fragile. The next time the luck runs out, the Golem makes the same flawed inference and loses money. The reasoning review catches this by examining why the prediction was correct, not just whether it was correct.

Chen et al. (2025) showed that reasoning models do not always say what they think -- the stated reasoning trace can diverge from the internal computation that produced the output [CHEN-2025]. Huang et al. (2024) established that LLMs cannot reliably self-correct their own reasoning [HUANG-2024]. Both findings constrain the reasoning review: the alignment classification is itself produced by an LLM (T1, during the dream cycle) and is therefore subject to the same faithfulness problems. Mitigation: the alignment classifications are tracked over time. If the LLM consistently classifies traces as AlignedCorrect but the Golem's actual accuracy is mediocre, the classifications are poorly calibrated. The confidence calibrator can be applied to alignment classifications themselves, creating a meta-calibration layer.

#### Signal 2: Cross-Tick Consistency

Pure arithmetic, no LLM needed. Extract directional claims from recent reasoning traces and check for contradictions:

```rust
/// Check whether recent reasoning traces contradict each other.
pub struct ConsistencyChecker {
    claims: DashMap<TrackedItemId, Vec<(u64, DirectionalClaim)>>,
}

impl ConsistencyChecker {
    pub fn record(&self, item: TrackedItemId, tick: u64, trace: &str) {
        let claim = extract_directional_claim(trace);
        self.claims.entry(item).or_default().push((tick, claim));
    }

    pub fn contradictions(&self, item: &TrackedItemId, window: usize) -> u32 {
        let claims = self.claims.get(item)?;
        let recent = &claims[claims.len().saturating_sub(window)..];
        let mut contradictions = 0;
        for i in 1..recent.len() {
            if recent[i].1.contradicts(&recent[i-1].1)
                && recent[i].0 - recent[i-1].0 < 5
            {
                contradictions += 1;
            }
        }
        contradictions
    }
}
```

High contradiction rates signal unstable reasoning. Feeds the Daimon (lowers dominance) and the dream engine (contradiction density is a scheduling trigger).

#### Signal 3: User Annotations

The owner reads a reasoning trace and annotates: "The fee spike was actually from the new emissions schedule change, not staking." Owner annotations are the most reliable quality signal and are stored as high-value Grimoire entries with elevated retrieval boost.

---

## Loop 8: Meta-Learning Evaluation

### The Problem

The system has many learning loops. Nothing evaluates whether the *loops themselves* are working. Liu and Hernandez-Lobato (ICML 2025) call this intrinsic metacognitive learning [LIU-METACOGNITION-2025].

### The Solution: Learning Process Metrics

```rust
/// Meta-learning dashboard. Computed weekly.
pub struct MetaLearningReport {
    pub period: DateRange,

    // Corrector convergence.
    pub corrector_convergence_rates: Vec<(CategoryId, f64)>,
    pub corrector_convergence_trend: f64,

    // Dream yield.
    pub dream_yield: f64,
    pub dream_yield_trend: f64,
    pub dream_hypothesis_resolution_time: Duration,

    // Attention precision.
    pub attention_promotion_to_stability: Duration,
    pub attention_precision: f64,

    // Heuristic durability.
    pub heuristic_half_life: Duration,
    pub heuristic_inheritance_rate: f64,

    // Generational progress.
    pub time_to_competence: Vec<(u32, Duration)>,
    pub inheritance_validation_rate: f64,

    // Aggregate.
    pub meta_learning_score: f64,
}
```

### Integration

- **Dream scheduling**: If `dream_yield` is declining, reduce dream frequency or change replay strategy.
- **Attention forager**: If `attention_precision` is low, raise the promotion threshold.
- **Mortality engine**: If `meta_learning_score` is negative for two consecutive weekly reviews, the golem's learning processes are degrading. Accelerate the Hayflick clock.
- **Generational transfer**: `time_to_competence` across generations is the final metric. If each generation reaches competence faster, inheritance works.

### TUI: The Meta-Learning Screen

```
META-LEARNING (7-day evaluation)

  CORRECTOR CONVERGENCE
  ---------------------
  fee_rate:     47 samples to halve SE (was 62 last week, improving)
  supply_rate:  55 samples to halve SE (was 41 last week, degrading)
  price_range:  39 samples to halve SE (was 44 last week, improving)

  DREAM YIELD
  -----------
  Creative predictions confirmed: 8/24 = 33%
  Trend: up from 28% last week
  Avg resolution time: 14.2 hours

  ATTENTION QUALITY
  -----------------
  Promotions to stable: avg 3.2 days (was 4.1, improving)
  Useful promotions: 67% (was 58%, improving)

  HEURISTIC DURABILITY
  --------------------
  Avg heuristic lifespan: 12.4 days
  Inheritance survival: 72% of inherited heuristics validated

  GENERATIONAL PROGRESS
  ---------------------
  Gen 1: 847 ticks to 70% accuracy
  Gen 2: 612 ticks to 70% accuracy (28% faster)
  Gen 3: 491 ticks to 70% accuracy (20% faster)

  META SCORE: +0.24 (learning processes are improving)
```

This screen answers: "Is my golem not just getting smarter, but getting smarter *faster*?"

### Known Limitations

**Small sample sizes.** Most meta-metrics need weeks to be statistically meaningful. The system reports "insufficient data" when sample size is too small.

**Confounding variables.** If `corrector_convergence_rate` improved, is it the corrector or a more predictable market? Shadow testing (Loop 6) provides counterfactuals, but confounding is inherent.

**Meta-meta-learning oscillation.** Meta-learning adjustments are rate-limited (one per weekly review) with asymmetric bias (reducing > increasing).

---

## How the Slow Loops Compose with the Fast Loops

| Evaluation | Speed | Feeds Into | Fed By |
|-----------|-------|-----------|--------|
| Confidence calibration (Loop 1) | Per-resolution | Action gate, Daimon | Every prediction resolution |
| Context attribution (Loop 2) | Per-theta-tick | Grimoire retrieval ranking | Every escalated tick outcome |
| Cost-effectiveness (Loop 3) | Per-theta-tick | Inference tier routing | Every inference call |
| Tool selection (Loop 4) | Per-action | Tool routing preferences | Every executed action |
| Adversarial awareness (Loop 5) | Per-action | Safety layer, gas strategy | Every on-chain transaction |
| Shadow strategies (Loop 6) | Weekly | Owner parameter decisions | Every theta tick (counterfactual) |
| Reasoning quality (Loop 7) | Per-dream-cycle | Dream replay priority, Grimoire | Reasoning traces + outcomes |
| Meta-learning (Loop 8) | Weekly | Dream scheduling, attention, mortality | All other loops' metrics |

The fast loops produce the data. The slow loops evaluate the data producers. The meta-learning loop evaluates the evaluators. Three levels of feedback at their natural timescales.

---

## References

- [KARPATHY-2026] Karpathy, A. (2026). "autoresearch." GitHub. — *Articulates the one-metric/one-arena/one-gate pattern shared by all evaluation loops.*
- [HUANG-2024] Huang, J. et al. (2024). "Large Language Models Cannot Self-Correct Reasoning Yet." ICLR. — *Demonstrates that intrinsic self-correction degrades LLM performance, constraining what the reasoning quality review can trust.*
- [CHEN-2025] Chen, A. et al. (2025). "Reasoning Models Don't Always Say What They Think." arXiv:2505.05410. — *Shows stated reasoning traces can diverge from internal computation, motivating external alignment checks.*
- [LIU-METACOGNITION-2025] Liu, T. & Hernandez-Lobato, J.M. (2025). "Truly Self-Improving Agents Require Intrinsic Metacognitive Learning." ICML. — *Defines the meta-learning evaluation loop: evaluating whether the learning processes themselves are improving.*
- [ROESE-1997] Roese, N.J. (1997). "Counterfactual Thinking." *Psychological Bulletin*, 121(1). — *Establishes the psychological basis for shadow strategy testing and counterfactual reasoning about alternative parameter choices.*
- [GAO-SELF-EVOLVING-2025] Gao, H. et al. (2025). "A Comprehensive Survey of Self-Evolving AI Agents." arXiv:2508.07407. — *Surveys the landscape of autonomous agent self-improvement, providing context for the meta-learning approach.*
- [SHINN-REFLEXION-2023] Shinn, N. et al. (2023). "Reflexion: Language Agents with Verbal Reinforcement Learning." NeurIPS. — *Introduces the Reflexion pattern used in the double-loop learning architecture, showing gains require external verification signals.*
