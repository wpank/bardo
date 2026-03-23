# The Immortal Control: Self-Hosted Mode as Falsifiable Experiment [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Crate**: `golem-mortality`
>
> **Depends on**: `00-thesis.md` (foundational mortality thesis), `02-epistemic-decay.md` (epistemic clock specification), `04-economic-mortality.md` (USDC as metabolic substrate), `06-thanatopsis.md` (four-phase death protocol)

---

> **Reader orientation:** This document specifies Bardo's immortal mode, a deliberate control experiment. Self-hosted Golems (mortal autonomous DeFi agents) can opt into `immortal: true`, which disables all three mortality clocks. The prediction, grounded in ML degradation research, is that immortal Golems will measurably degrade compared to mortal populations. This mode exists to make the mortality thesis falsifiable: if immortal agents consistently outperform mortal ones, the thesis is wrong. See `01-architecture.md` (triple-clock system, self-hosted exception rules) for how mortality clocks operate. See `prd2/shared/glossary.md` for full term definitions.

## The Thesis Must Be Falsifiable

Bardo's central claim -- that mortal agents outperform immortal ones -- is an empirical prediction, not a philosophical assertion. For the prediction to be scientific, it must be falsifiable (Popper, 1959). The self-hosted immortal mode provides the control group.

Karl Popper's demarcation criterion separates science from pseudoscience: a theory is scientific if and only if it makes predictions that could, in principle, be shown false. "Mortality produces better agents" is scientific because there exists a concrete experimental setup -- immortal Golems running the same strategies in the same markets -- that could refute it. If immortal Golems consistently outperform mortal populations across all six measurement dimensions, the mortality thesis is wrong and the architecture should be revised.

Self-hosted Golems with `immortal: true` run indefinitely, bearing no economic mortality (owner pays infrastructure directly), no epistemic death (fitness can decay but does not kill), and no stochastic mortality (no per-tick hazard). They represent the default assumption of every other agent framework: agents should live forever.

---

## What Immortal Mode Disables

Immortal mode disables every mechanism that could terminate the Golem or modulate its behavior based on mortality pressure. The following is the complete enumeration:

### Mortality Clocks Disabled

| Clock                    | Mechanism                                                                  | Effect When Disabled                                                                                                              |
| ------------------------ | -------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| **Economic mortality**   | USDC balance depletion triggers death                                      | Owner pays infrastructure directly; no credit partitions, no burn rate tracking, no survival pressure from resource exhaustion |
| **Epistemic mortality**  | Predictive fitness decay below senescence threshold triggers death cascade | Fitness is computed and tracked but crossing the senescence threshold does not initiate the Death Protocol                        |
| **Stochastic mortality** | Per-tick hazard rate with age-increasing probability                       | No random death checks; hazard rate is computed for telemetry but never evaluated as a kill condition                             |

### Behavioral Mechanisms Disabled

| Mechanism                              | Normal Function                                                              | Immortal State                                                       |
| -------------------------------------- | ---------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| **Behavioral phase transitions**       | Vitality score drives Thriving/Stable/Conservation/Declining/Terminal phases | Always in Thriving-equivalent; no phase transitions                  |
| **Survival pressure computation**      | `computeSurvivalPressure()` returns 0.0-1.0 based on projected life hours    | Always returns 1.0 (maximum survival)                                |
| **Death Protocol**                     | Three-phase orderly shutdown (Settle, Reflect, Legacy)                       | Never fires autonomously; only owner-initiated teardown              |
| **Apoptotic reserve**                  | Ring-fenced funds for orderly death                                          | Not allocated (no death to fund)                                      |
| **Stochastic death checks**            | Per-tick mortality evaluation                                                | Skipped entirely                                                     |
| **Epistemic senescence cascade**       | Below-threshold fitness triggers grace period then death                     | Grace period never starts; fitness can reach 0.0 without consequence |
| **Knowledge demurrage**                | Grimoire entries lose confidence over time unless revalidated                | Disabled; entries persist at their assigned confidence indefinitely  |
| **Eros/Thanatos spectrum**             | Survival pressure modulates risk appetite, frugality, legacy focus           | Risk multiplier fixed at 1.0; no Thanatos behaviors activate         |
| **Nietzsche metamorphosis arc**        | Camel (thriving) -> Lion (fighting) -> Child (accepting)                     | Permanently Camel; no metamorphosis                                  |
| **Credit partition rebalancing**       | LLM/Gas/Data partitions with predictive rebalancing                          | No partitions; owner budget is external                              |
| **Thanatos-phase cannibalization**     | Terminal partition shift: 50% to Legacy                                      | No Legacy partition; no terminal reallocation                        |
| **Clade sharing threshold modulation** | Threshold drops from 0.6 to 0.3 to 0.1 as mortality approaches               | Fixed at normal threshold (0.6)                                      |
| **Death snapshots**                    | Progressive Haiku summaries beginning at 25% credits                         | No credit-triggered snapshots                                        |
| **Successor recommendation**           | Dying Golem's final advisory for next generation                             | No death, no recommendation                                          |

### Configuration

```rust
/// immortal: true configuration — complete specification
fn immortal_config() -> MortalityConfig {
    MortalityConfig {
        economic: EconomicConfig {
            enabled: false,                      // No credit depletion tracking
            death_reserve_floor_usdc: 0.0,       // No reserve needed
            death_reserve_proportional: 0.0,     // No reserve needed
            conservation_threshold: 0.0,         // No conservation mode
            terminal_threshold: 0.0,             // No terminal phase
            ..Default::default()
        },
        epistemic: EpistemicConfig {
            enabled: false,                      // Fitness tracked but not fatal
            fitness_window: 2000,                // Still computed for measurement
            senescence_threshold: 0.35,          // Still computed for "would have died" tracking
            recovery_grace_period: 500,          // Irrelevant but kept for comparison
            decay_half_life_by_domain: HashMap::new(), // No decay — entries persist indefinitely
            ..Default::default()
        },
        stochastic: StochasticConfig {
            enabled: false,                      // No per-tick death risk
            base_hazard_rate: 1e-6,              // Still computed for measurement
            age_hazard_coefficient: 0.00001,
            epistemic_hazard_multiplier: 2.0,
            max_hazard_rate: 0.001,
            seed: None,
        },
        affect: AffectConfig {
            enabled: true,                       // Affect still runs (for fair comparison)
            appraisal_model: AppraisalModel::ChainOfEmotion,
            mood_decay_rate: 0.95,
            ..Default::default()
        },
        demurrage: DemurrageConfig {
            enabled: false,                      // No knowledge carrying costs
            validation_interval_ticks: 5000,
            decay_rate_per_interval: 0.05,
            archive_threshold: 0.1,
            ..Default::default()
        },
        phage: PhageConfig {
            enabled: true,                       // Micro-Replicants still run if configured
            daily_spawn_rate: 2,
            max_lifetime_ticks: 50,
            ..Default::default()
        },
        immortal: true,                          // Master flag: disables ALL mortality clocks
        ..Default::default()
    }
}
```

---

## What Still Works in Immortal Mode

Everything that is not mortality-specific continues to operate at full capability. The immortal Golem has every tool, every cognitive resource, and every integration that a mortal Golem has:

| System                                             | Operates Normally | Notes                                                                             |
| -------------------------------------------------- | ----------------- | --------------------------------------------------------------------------------- |
| Full heartbeat loop (6-state FSM)                  | Yes               | IDLE -> SENSING -> DECIDING -> ACTING -> REFLECTING -> SLEEPING                   |
| Probe system (11 deterministic probes)             | Yes               | All T0 probes fire every tick                                                     |
| LLM tier escalation (Haiku/Sonnet/Opus)            | Yes               | All tiers available without survival-pressure suppression                         |
| Complete Grimoire (LanceDB + SQLite + PLAYBOOK.md) | Yes               | All five entry types: insight, heuristic, warning, causal_link, strategy_fragment |
| Loop 1 reactive learning                           | Yes               | Error correction within current strategy                                          |
| Loop 2 strategic reflection                        | Yes               | Double-loop learning every ~50 ticks                                              |
| Loop 3 meta-consolidation                          | Yes               | Deutero-learning (if [HARDENED] enabled)                                          |
| Daimon engine                                      | Yes               | Emotional appraisal computed normally, but no mortality-aware affect              |
| Regime detection                                   | Yes               | Threshold-based (CORE) or HMM (HARDENED)                                          |
| Decision cache (System 2 -> System 1 distillation) | Yes               | Cache hit rates tracked normally                                                  |
| Clade membership and knowledge sharing             | Yes               | Push/pull/catch-up sync all operational                                           |
| Replicant spawning                                 | Yes               | If owner has enabled and funded                                                   |
| Micro-Replicant (phage) system                     | Yes               | If configured; phage death rate is a health signal                                |
| Styx Archive insurance snapshots                     | Yes               | Every 6 hours, same as mortal                                                     |
| Styx Lethe (formerly Commons) integration           | Yes               | If death testaments from mortal siblings are available                            |
| Mental models library                              | Yes               | All 700 models (HARDENED) or 13 core models                                       |
| OODA + Reflexion pipeline                          | Yes               | Full 4-step cascade on every non-suppressed tick                                  |
| Tool access via Sanctum                            | Yes               | All golem-tools available                                                         |
| Telemetry and observability                        | Yes               | Full JSONL event stream, webhook emission                                         |
| PolicyCage enforcement                             | Yes               | On-chain hard limits unaffected by immortality                                    |
| Warden time-delayed execution (optional, deferred)  | Yes               | If configured and deployed                                                        |
| ERC-8004 identity                                  | Yes               | Agent identity and reputation unaffected                                          |

The immortal Golem has every capability a mortal Golem has, minus the mortality pressure. If mortality is truly beneficial, its absence should produce measurable degradation. If mortality is merely a constraint, its absence should produce superior performance.

### What Immortal Mode Retains

Immortal mode is a **subtraction**, not a replacement. The following systems remain fully active:

- **Phase lock**: Phase is locked to `Thriving` (vitality score artificially held at 1.0)
- **Generic affect**: Joy, trust, fear, anger from trading outcomes and market events remain active. Only mortality-specific emotions (Economic Anxiety, Epistemic Vertigo, Stochastic Dread) are disabled.
- **Memory services**: Grimoire, Curator, Styx Lethe all operate normally. Knowledge demurrage still runs (entries still decay without validation). However, no death testament is ever produced since death never occurs.
- **Clade participation**: Immortal Golems can share knowledge with mortal siblings. However, their unbounded lifespan means they accumulate knowledge that mortal Golems cannot — creating an asymmetry that is itself a research question.
- **Inference and trading**: All cognitive and financial operations unchanged.

**Explicitly NOT retained**: Terminal acceptance, legacy budget activation, death protocol (all 4 phases), successor inheritance triggers, Eros/Thanatos drive modulation, mortality-driven behavioral phase transitions.

**Framing**: Immortal mode exists for **evaluation only** (A/B testing mortality's behavioral effects), not as a production configuration. An immortal Golem is the control group.

---

## Six Predictions and Predicted Degradation Pattern

> **Extended:** Six falsifiable predictions with full research basis (epistemic fitness decay, loss of plasticity, technical debt, behavioral diversity, clade-level underperformance, cognitive entrenchment) and four-phase degradation analysis (Honeymoon, Stagnation, Decay, Senescence Without Death) — see [../../prd2-extended/02-mortality/11-immortal-control-extended.md](../../prd2-extended/02-mortality/11-immortal-control-extended.md)

**Summary of predictions:** (1) Epistemic fitness decay within 2-4 weeks, (2) Loss of plasticity per Dohare et al. 2024, (3) Technical debt accumulation per Sculley et al. 2015, (4) Reduced behavioral diversity per March 1991, (5) Clade-level underperformance within 60 days, (6) Cognitive entrenchment per Dane 2010. Predicted degradation follows Honeymoon (d1-14) -> Stagnation (d14-45) -> Decay (d45-90) -> Senescence (d90+).

---

## Six-Dimensional Measurement Framework

The control experiment requires systematic comparison between immortal and mortal Golem populations. The measurement framework tracks six orthogonal dimensions, each with full type definitions and collection methodology.

### Dimension 1: Epistemic Fitness Over Time

Even though immortal Golems do not die from epistemic decay, the fitness score is computed every tick using the identical algorithm as mortal Golems. This enables direct comparison.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicTrackingForImmortal {
    /// Fitness score computed every tick, same algorithm as mortal Golems.
    pub fitness_history: Vec<FitnessSnapshot>,
    /// Would this Golem have entered senescence under mortal rules?
    pub would_have_died: bool,
    /// The tick at which mortal death would have occurred.
    pub would_have_died_at_tick: Option<u64>,
    /// Timestamp of hypothetical mortal death.
    pub would_have_died_at: Option<u64>,
    /// How many mortal "lifetimes" has this Golem outlived?
    pub mortal_lifetime_equivalents: f64,
    /// Rolling 1000-tick fitness average.
    pub rolling_fitness_average: f64,
    /// Slope of fitness trend over last 2000 ticks.
    pub fitness_trend: f64,
    /// Number of times fitness crossed below senescence threshold.
    pub senescence_crossings: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessSnapshot {
    pub tick: u64,
    pub fitness: f64,
    pub timestamp: u64,
}
```

### Dimension 2: Strategy Diversity (MAP-Elites Niche Occupancy)

For Clades with multiple Golems, track how many behavioral niches are occupied. MAP-Elites (Mouret & Clune, 2015) provides the quality-diversity framework: each cell in the behavioral space represents a distinct strategy archetype.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityMetric {
    /// Number of distinct MAP-Elites cells occupied by Clade members.
    pub niche_occupancy: u32,
    /// Total cells in the behavioral grid.
    pub total_niches: u32,
    /// Niche occupancy ratio (0.0-1.0).
    pub occupancy_ratio: f64,
    /// Behavioral variance across Clade members (std dev of strategy embeddings).
    pub behavioral_variance: f64,
    /// Are strategies converging (negative = bad) or diverging (positive = good)?
    pub convergence_trend: f64,
    /// Per-niche quality: best fitness score in each occupied niche.
    pub niche_qualities: Vec<NicheQuality>,
    /// Timestamp of measurement.
    pub measured_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheQuality {
    pub niche_id: String,
    pub best_fitness: f64,
    pub occupant_count: u32,
}
```

### Dimension 3: Novel Insight Generation Rate

Measures the Golem's capacity to produce genuinely new knowledge, not merely refine existing entries.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoveltyMetric {
    /// Insights generated per 1000 ticks.
    pub insight_rate: f64,
    /// Novelty score of generated insights (embedding distance from existing Grimoire).
    pub average_novelty: f64,
    /// Trend: positive = more novel insights over time, negative = fewer.
    pub novelty_trend: f64,
    /// Breakdown by entry type.
    pub by_type: HashMap<GrimoireEntryType, NoveltyByType>,
    /// Number of insights that were adopted by Clade siblings.
    pub adopted_by_clade: u32,
    /// Adoption rate (adopted / total generated).
    pub clade_adoption_rate: f64,
    /// Window over which this metric was computed (in ticks).
    pub window_ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoveltyByType {
    pub count: u32,
    pub average_novelty: f64,
}
```

### Dimension 4: Risk-Adjusted Return

Standard financial performance metrics, compared between mortal and immortal populations with equivalent starting capital.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAdjustedReturnMetric {
    /// Sharpe ratio over measurement window.
    pub sharpe_ratio: f64,
    /// Sortino ratio (downside deviation only).
    pub sortino_ratio: f64,
    /// Maximum drawdown as fraction (0.0-1.0).
    pub max_drawdown: f64,
    /// Cumulative P&L in USDC.
    pub cumulative_pnl_usdc: f64,
    /// Annualized return rate.
    pub annualized_return: f64,
    /// Win rate (fraction of profitable ticks with trading activity).
    pub win_rate: f64,
    /// Average profit per winning trade.
    pub avg_win: f64,
    /// Average loss per losing trade.
    pub avg_loss: f64,
    /// Profit factor (gross profit / gross loss).
    pub profit_factor: f64,
    /// Measurement window in days.
    pub window_days: u32,
}
```

### Dimension 5: Grimoire Health

Measures the quality and utility of the Golem's accumulated knowledge base.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrimoireHealthMetric {
    /// Total entries in Grimoire.
    pub entry_count: u32,
    /// Entries by type.
    pub by_type: HashMap<GrimoireEntryType, u32>,
    /// Fraction of entries validated against fresh market data in last 1000 ticks.
    pub validation_rate: f64,
    /// Average confidence of active entries.
    pub average_confidence: f64,
    /// Fraction of entries that would be archived under demurrage rules.
    pub stale_entry_rate: f64,
    /// Context window utilization: fraction of available context consumed by Grimoire retrieval.
    pub context_utilization: f64,
    /// Number of contradictory entry pairs (entries that give conflicting advice).
    pub contradiction_count: u32,
    /// Average age of active entries in ticks.
    pub average_entry_age_ticks: u64,
    /// Causal graph density: edges / possible edges.
    pub causal_graph_density: f64,
    /// Causal graph largest connected component size.
    pub causal_graph_largest_component: u32,
}
```

### Dimension 6: Adaptation to Regime Changes

When a detectable market regime shift occurs, measure how quickly the Golem adapts -- restores epistemic fitness to pre-shift levels.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationMetric {
    /// Unique identifier for the regime shift event.
    pub regime_shift_id: String,
    /// Type of regime shift detected.
    pub shift_type: RegimeShiftType,
    /// Tick at which regime shift was detected.
    pub detected_at_tick: u64,
    /// Epistemic fitness at moment of shift detection.
    pub fitness_at_detection: f64,
    /// Minimum fitness reached after shift.
    pub fitness_nadir: f64,
    /// Ticks from regime shift detection to fitness recovery (back to pre-shift level).
    pub recovery_ticks: Option<u64>,
    /// Was adaptation successful (fitness recovered to within 90% of pre-shift level)?
    pub adapted: bool,
    /// How many strategies were modified during adaptation?
    pub strategies_modified: u32,
    /// How many Grimoire entries were invalidated during adaptation?
    pub entries_invalidated: u32,
    /// How many new entries were created during adaptation?
    pub entries_created: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegimeShiftType {
    TrendingUp,
    TrendingDown,
    RangeBound,
    Volatile,
    Unknown,
}
```

---

## ImmortalComparison Interface

The top-level struct for comparing immortal and mortal populations in the experiment:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmortalComparison {
    /// Unique experiment identifier.
    pub experiment_id: String,
    /// Experiment start timestamp.
    pub started_at: u64,
    /// Current measurement timestamp.
    pub measured_at: u64,
    /// Duration in days.
    pub duration_days: u32,
    /// Market regime during the experiment period.
    pub market_regimes: Vec<RegimePeriod>,
    /// Immortal Golem(s) data.
    pub immortal: GroupMetrics,
    /// Mortal Clade data (may include multiple generations).
    pub mortal: MortalGroupMetrics,
    /// Per-dimension comparison verdicts.
    pub verdicts: ComparisonVerdicts,
    /// Overall thesis status.
    pub thesis_status: ThesisStatus,
    /// Minimum duration for statistically valid comparison (30 days).
    pub minimum_duration_days: u32,
    /// Has the experiment run long enough for valid conclusions?
    pub is_statistically_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimePeriod {
    pub regime: String,
    pub start_tick: u64,
    pub end_tick: Option<u64>,
    pub volatility: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMetrics {
    pub golem_ids: Vec<String>,
    pub total_budget_usdc: f64,
    pub epistemic: EpistemicTrackingForImmortal,
    pub diversity: DiversityMetric,
    pub novelty: NoveltyMetric,
    pub returns: RiskAdjustedReturnMetric,
    pub grimoire_health: GrimoireHealthMetric,
    pub adaptations: Vec<AdaptationMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalGroupMetrics {
    /// All Golem IDs across all generations.
    pub golem_ids: Vec<String>,
    /// Number of generations (deaths + successors).
    pub generations: u32,
    /// Total budget consumed across all generations.
    pub total_budget_usdc: f64,
    /// Average lifespan per generation in days.
    pub average_lifespan_days: f64,
    /// Aggregated metrics across all generations.
    pub epistemic: EpistemicTrackingForImmortal,
    pub diversity: DiversityMetric,
    pub novelty: NoveltyMetric,
    pub returns: RiskAdjustedReturnMetric,
    pub grimoire_health: GrimoireHealthMetric,
    pub adaptations: Vec<AdaptationMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verdict {
    ImmortalWins,
    MortalWins,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonVerdicts {
    pub epistemic: Verdict,
    pub diversity: Verdict,
    pub novelty: Verdict,
    pub returns: Verdict,
    pub grimoire_health: Verdict,
    pub adaptation: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThesisStatus {
    /// Mortal wins on 4+ dimensions.
    Supported,
    /// Mortal wins on 3 dimensions.
    WeaklySupported,
    /// 2-3 dimensions each or insufficient data.
    Inconclusive,
    /// Immortal wins on 3 dimensions.
    WeaklyRefuted,
    /// Immortal wins on 4+ dimensions.
    Refuted,
}
```

---

## MortalityExperimentConfig Interface

Configuration for owners who want to run the control experiment:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityExperimentConfig {
    /// Enable the experiment. Creates parallel tracking infrastructure.
    pub enabled: bool,
    /// Identifier for this experiment run.
    pub experiment_id: String,
    /// Golem IDs designated as immortal controls.
    pub immortal_golem_ids: Vec<String>,
    /// Golem IDs designated as mortal treatment group.
    pub mortal_golem_ids: Vec<String>,
    /// Budget equivalence: total USDC allocated to each group.
    pub budget_per_group: f64,
    /// Measurement collection interval in ticks. Default: 100.
    pub measurement_interval_ticks: u64,
    /// Snapshot interval for full comparison report. Default: 24.
    pub report_interval_hours: u32,
    /// Minimum experiment duration before drawing conclusions. Default: 30.
    pub minimum_duration_days: u32,
    /// Statistical significance threshold for verdicts. Default: 0.05.
    pub significance_threshold: f64,
    /// Market regime filter: only compare during specified regimes. None = all.
    pub regime_filter: Option<Vec<String>>,
    /// Storage location for experiment data. Default: ~/.bardo/experiments/
    pub data_store_path: String,
    /// Emit webhook on verdict change. Default: true.
    pub webhook_on_verdict_change: bool,
    /// Dashboard integration: show experiment panel. Default: true.
    pub dashboard_enabled: bool,
}
```

---

## Reporting Format

The control experiment data is aggregated and available through the owner dashboard. Reports are generated at configurable intervals (default: daily) and on demand.

### Example Dashboard Output

```
================================================================================
  IMMORTAL vs MORTAL COMPARISON REPORT
  Experiment: exp-2026-Q1-alpha
  Duration: 73 days (minimum 30 — STATISTICALLY VALID)
================================================================================

IMMORTAL GROUP
  Golem: Golem-ABC (Generation: Inf, Age: 73 days)
  Budget consumed: $147.20 (ongoing)

MORTAL GROUP
  Golems: Golem-D (Gen 1, d.23), Golem-E (Gen 2, d.41), Golem-F (Gen 3, active)
  Generations: 3
  Budget consumed: $142.80
  Average lifespan: 24.3 days

--------------------------------------------------------------------------------
DIMENSION 1: EPISTEMIC FITNESS                              VERDICT: MORTAL WINS
--------------------------------------------------------------------------------
                          Immortal        Mortal (current gen)   Mortal (avg)
  Current fitness:        0.42            0.71                   0.68
  Fitness at creation:    0.78            0.73                   0.75
  Fitness trend (slope):  -0.0034/tick    +0.0008/tick           -0.0002/tick
  Would have died:        Yes (day 51)    N/A
  Lifetime equivalents:   1.4             N/A

--------------------------------------------------------------------------------
DIMENSION 2: STRATEGY DIVERSITY                             VERDICT: MORTAL WINS
--------------------------------------------------------------------------------
                          Immortal        Mortal Clade
  Niche occupancy:        1 / 8           5 / 8
  Behavioral variance:    0.12            0.67
  Convergence trend:      -0.03/day       +0.01/day

--------------------------------------------------------------------------------
DIMENSION 3: NOVEL INSIGHT GENERATION                       VERDICT: MORTAL WINS
--------------------------------------------------------------------------------
                          Immortal        Mortal Clade
  Rate (per 1000 ticks):  0.3             1.8
  Average novelty:        0.21            0.54
  Novelty trend:          -0.04/day       +0.01/day
  Clade adoption rate:    N/A             34%

--------------------------------------------------------------------------------
DIMENSION 4: RISK-ADJUSTED RETURN                           VERDICT: MORTAL WINS
--------------------------------------------------------------------------------
                          Immortal        Mortal Clade (cumul.)
  60-day Sharpe:          0.41            0.93
  Sortino:                0.38            0.87
  Max drawdown:           -18.3%          -12.1%
  Cumulative P&L:         -$23.40         +$8.70
  Win rate:               44%             52%

--------------------------------------------------------------------------------
DIMENSION 5: GRIMOIRE HEALTH                                VERDICT: MORTAL WINS
--------------------------------------------------------------------------------
                          Immortal        Mortal (current gen)
  Entry count:            847             312
  Validated (last 1000t): 23%             71%
  Avg confidence:         0.38            0.62
  Stale entry rate:       61%             14%
  Context utilization:    89%             43%
  Contradictions:         23              4

--------------------------------------------------------------------------------
DIMENSION 6: REGIME ADAPTATION                              VERDICT: MORTAL WINS
--------------------------------------------------------------------------------
  Last regime shift: Day 58 (trending_up -> volatile)

                          Immortal        Mortal (current gen)
  Recovery ticks:         340             N/A (new Golem started fresh)
  Strategies modified:    1               N/A
  Entries invalidated:    12              N/A

  Note: Mortal Gen 2 died during this shift. Gen 3 initialized with
  fresh knowledge + death testament. Effective "recovery" was 0 ticks.

================================================================================
OVERALL THESIS STATUS: SUPPORTED (6/6 dimensions favor mortal)
================================================================================
  The immortal Golem underperforms the mortal succession across all six
  measurement dimensions after 73 days. Predicted Phase 3 (Decay) degradation
  pattern confirmed: fitness decline began at day 18, crossed senescence
  threshold at day 51, and continues to deteriorate.

  Recommendation: Enable mortality for this Golem. See migration path below.
================================================================================
```

---

## Why Not Nerf Immortal Mode?

It would be easy to handicap immortal Golems -- reduce their inference tier, limit their Grimoire size, disable Loop 2 reflection, cap their context window. This would guarantee that mortal Golems outperform. But it would prove nothing.

The control experiment is only valid if immortal Golems have **every advantage**:

1. **Continuous operation.** No downtime for death/rebirth. No wasted ticks on death preparation, successor provisioning, or knowledge re-ingestion.
2. **Complete knowledge retention.** All accumulated knowledge persists at full confidence. No lossy compression through death testament packaging. No confidence decay on inheritance.
3. **Full strategy continuity.** No cold-start period for new generations. No Baldwin Effect exploration. No strategy divergence requirements.
4. **Zero urgency.** No survival pressure distorting decisions. No Eros/Thanatos spectrum. No risk-multiplier inflation. Pure rational decision-making unconstrained by existential threat.
5. **Full resource access.** All LLM tiers available at all times. No credit conservation mode. No partition constraints.
6. **Maximum learning time.** No epistemic senescence. No mortality clocks. The Golem has unlimited time to accumulate experience and refine strategies.

Every bias in the experiment favors immortality. The immortal Golem has strictly more resources, more time, more knowledge, and more stability than any single mortal Golem.

If mortal Golems still outperform under these conditions, the mortality thesis is robustly validated. If they do not, the thesis needs revision. This is the intellectual integrity of the design: **Bardo bets its central claim on an experiment it could lose.**

Nerfing immortal mode would be the scientific equivalent of testing a drug against a placebo that causes nausea -- you win the trial but learn nothing about the drug.

---

## Migration Path: Immortal to Mortal

Owners who begin with immortal mode and observe degradation can migrate to mortal mode without starting over. The migration is gradual -- each mortality clock can be enabled independently, and the owner can observe the effect of each before enabling the next.

### Step 1: Enable Knowledge Demurrage (Low Risk)

```toml
# Least disruptive change: activate knowledge carrying costs
[golem.mortality.demurrage]
enabled = true
validation_interval_ticks = 5000  # ~2.3 days
decay_rate_per_interval = 0.05    # 5% per interval
archive_threshold = 0.1
```

**Effect:** Grimoire entries that are not actively validated against fresh market data begin to lose confidence. Stale entries are gradually archived. The Golem's knowledge base self-prunes. This alone may restore epistemic fitness by reducing context window pollution.

**Observation period:** 7 days. Monitor `stale_entry_rate` and `context_utilization` -- both should decrease. Monitor `epistemic_fitness` -- should stabilize or improve.

### Step 2: Enable Epistemic Mortality (Medium Risk)

```toml
# Activate informational death
[golem.mortality.epistemic]
enabled = true
fitness_window = 2000
senescence_threshold = 0.35
recovery_grace_period = 500
```

**Effect:** The Golem can now die from epistemic decay. If fitness drops below 0.35 and remains there for 500 ticks, the Death Protocol initiates. This creates genuine urgency around learning and adaptation.

**Observation period:** 14 days. If the Golem's current fitness is already below 0.35, this will trigger death within the grace period. The owner should be prepared for succession.

### Step 3: Enable Stochastic Mortality (Medium Risk)

```toml
# Activate per-tick hazard rate
[golem.mortality.stochastic]
enabled = true
base_hazard_rate = 1e-6
age_hazard_coefficient = 0.00001
epistemic_hazard_multiplier = 2.0
max_hazard_rate = 0.001
```

**Effect:** The Golem now faces a small, ever-present probability of death that increases with age and epistemic decay. This breaks backward induction and incentivizes immediate knowledge sharing. For a healthy Golem, the per-tick probability is ~1 in 1,000,000 (~0.1% per day). For an aged, epistemically declining Golem, it can reach ~0.1% per tick.

**Observation period:** 14 days. Monitor Clade sharing rate -- should increase as the Golem internalizes the possibility of sudden death.

### Step 4: Enable Economic Mortality (High Impact -- Hosted Only)

```toml
# Switch from owner-paid to credit model
[golem.mortality.economic]
enabled = true
death_reserve_floor_usdc = 0.3
death_reserve_proportional = 0.02
conservation_threshold = 0.3
terminal_threshold = 0.05
```

**Effect:** The Golem now operates on a finite credit model. The full behavioral spectrum activates: Thriving, Stable, Conservation, Declining, Terminal. Eros/Thanatos dynamics engage. The Golem must earn revenue to survive. This is the full mortal experience.

**Note:** This step requires switching from owner-paid infrastructure to the credit model. For self-hosted deployments, this means configuring the x402 payment system. The owner can skip this step and run with epistemic + stochastic mortality only.

### Step 5: Disable Immortal Flag

```toml
[golem.mortality]
immortal = false
```

**Effect:** The master flag is cleared. The Golem is now fully mortal. All behavioral mechanisms, phase transitions, and survival pressure computations activate. The Golem's existing knowledge and strategies carry forward -- this is not a restart but a transition.

### Migration Telemetry

Each migration step emits a telemetry event:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityMigrationEvent {
    pub golem_id: String,
    pub step: MigrationStep,
    pub previous_config: serde_json::Value,
    pub new_config: serde_json::Value,
    pub timestamp: u64,
    pub epistemic_fitness_at_migration: f64,
    pub grimoire_health_at_migration: GrimoireHealthMetric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStep {
    Demurrage,
    Epistemic,
    Stochastic,
    Economic,
    FullMortal,
}
```

---

## The Die Moat and Immortal Mode

The immortal control group validates the Die moat thesis. If mortality is a structural moat (not a feature), then disabling it should produce measurable degradation across all six dimensions. The immortal control experiment generates the data that proves or disproves this claim.

The prediction, grounded in the research cited throughout this section:
- Immortal Golems will show knowledge accumulation without regularization (no demurrage, no forgetting)
- Decision quality will degrade as stale knowledge crowds context windows (Borges' Funes effect)
- Cooperation will decline without the shadow of uncertain death (KMRW violation)
- Population diversity will decrease (no generational turnover, premature convergence)

If immortal Golems consistently outperform mortal populations on all six metrics, the mortality thesis is falsified and the architecture requires revision.

---

## References

- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012.
- [BORGES-1942] Borges, J.L. "Funes the Memorious." In _Ficciones_, 1944.
- [CHRISTENSEN-1997] Christensen, C.M. _The Innovator's Dilemma_. Harvard Business School Press, 1997.
- [DANE-2010] Dane, E. "Reconsidering the Trade-off Between Expertise and Flexibility." _AMR_ 35(4), 2010.
- [DOHARE-2024] Dohare, S. et al. "Loss of Plasticity in Deep Continual Learning." _Nature_ 632, 2024.
- [GORONZY-WEYAND-2013] Goronzy, J.J. & Weyand, C.M. "Understanding Immunosenescence." _Nat. Immunol._ 14, 2013.
- [MARCH-1991] March, J.G. "Exploration and Exploitation in Organizational Learning." _Organization Science_ 2(1), 1991.
- [POPPER-1959] Popper, K. _The Logic of Scientific Discovery_. Routledge, 1959.
- [RAY-1991] Ray, T. "An Approach to the Synthesis of Life." _Artificial Life II_, SFI, 1991.
- [SCHUMPETER-1942] Schumpeter, J.A. _Capitalism, Socialism and Democracy_. Harper & Brothers, 1942.
- [SCULLEY-2015] Sculley, D. et al. "Hidden Technical Debt in Machine Learning Systems." _NIPS_, 2015.
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." _J. Monetary Economics_ 50(3), 2003.
- [VELA-2022] Vela, D. et al. "Temporal Quality Degradation in AI Models." _Scientific Reports_ 12, 2022.
- [XU-GAO-1997] Xu, H. & Gao, Y. "Premature Convergence in Genetic Algorithms." _Sci. China E_ 40, 1997.
