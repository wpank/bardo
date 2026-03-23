# Owner Controls: Mortality Configuration [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Crate**: `golem-mortality`
>
> **Depends on**: `00-thesis.md` (foundational mortality thesis), `02-epistemic-decay.md` (epistemic clock), `03-stochastic-mortality.md` (per-tick hazard rate), `08-mortality-affect.md` (mortality emotions), `06-thanatopsis.md` (death protocol), `07-succession.md` (generational evolution), `11-immortal-control.md` (immortal mode as control experiment), `12-integration.md` (mortality across the full Bardo stack)

---

> **Reader orientation:** This document is the complete configuration reference for Bardo's mortality system. It covers every parameter an owner can set in `golem.toml` to tune their Golem's (mortal autonomous DeFi agent's) mortality behavior: economic thresholds, epistemic decay rates, stochastic hazard parameters, affect settings, and immortal mode. Protocol-enforced invariants (parameters owners cannot change) are also documented. If you want to understand what the defaults do and why, this is the reference. See `prd2/shared/glossary.md` for full term definitions.

## Overview

Mortality is configurable but not optional. Every Golem faces mortality pressure unless the owner explicitly enables immortal mode -- and even then, the system tracks what _would_ have happened under mortal rules, providing the data for the falsifiable experiment (see `11-immortal-control.md`).

This document specifies the complete configuration surface, default values with rationale, protocol-enforced invariants that owners cannot change, telemetry events, error codes, environment variables, and testing configuration.

---

## Complete MortalityConfig Struct

The top-level configuration struct and all sub-configs:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityConfig {
    /// Economic mortality (credit depletion). Disabled for self-hosted immortal.
    pub economic: EconomicConfig,
    /// Epistemic mortality (predictive fitness decay).
    pub epistemic: EpistemicConfig,
    /// Stochastic mortality (per-tick hazard rate).
    pub stochastic: StochasticConfig,
    /// Knowledge demurrage (Grimoire entry decay).
    pub demurrage: DemurrageConfig,
    /// Micro-Replicant (phage) system for continuous self-testing.
    pub phage: PhageConfig,
    /// Daimon engine (emotional architecture).
    pub affect: AffectConfig,
    /// Thanatopsis Protocol (enhanced Death Protocol).
    pub thanatopsis: ThanatopsisConfig,
    /// Succession (generational learning).
    pub succession: SuccessionConfig,
    /// Styx connection configuration.
    pub styx: StyxConfig,
    /// Immortal override (self-hosted only).
    /// When true, disables ALL mortality clocks.
    /// See 11-immortal-control.md for full specification.
    /// Default: false.
    pub immortal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyxConfig {
    /// WebSocket endpoint for the Styx service.
    /// Default: "wss://styx.bardo.run"
    pub endpoint: String,
    /// Enable Styx Archive layer (insurance snapshots, death bundles).
    /// Default: true
    pub vault_enabled: bool,
    /// Enable Styx Lethe (formerly Commons) layer (death testament indexing, bloodstains).
    /// Default: true
    pub lethe_enabled: bool,
    /// Reconnect interval in seconds on connection loss.
    /// Default: 5
    pub reconnect_interval_secs: u64,
}
```

### EconomicConfig

Controls the economic mortality clock -- death by resource exhaustion.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicConfig {
    /// Enable economic mortality.
    /// When false, no credit tracking, no partitions, no resource-based death.
    /// Self-hosted Golems typically set this to false.
    /// Default: true (hosted), false (self-hosted).
    pub enabled: bool,

    /// Absolute floor for the Apoptotic Reserve in USDC.
    /// Guarantees a minimum-viable death even for cheaply funded Golems.
    /// The reserve is ring-fenced from creation and cannot be spent operationally.
    /// Default: 0.30
    /// Protocol invariant: minimum 0.10 (cannot be set below)
    pub death_reserve_floor_usdc: f64,

    /// Proportional reserve for well-funded Golems.
    /// Reserve = max(death_reserve_floor_usdc, initial_balance * death_reserve_proportional).
    /// A $50 Golem reserves max($0.30, $50 * 0.02) = $1.00.
    /// Default: 0.02 (2%)
    /// Protocol invariant: range 0.01-0.10
    pub death_reserve_proportional: f64,

    /// Credit percentage below which conservation mode activates.
    /// Conservation: suppress Opus, increase heartbeat interval 2x, monitor-only.
    /// Default: 0.30 (30%)
    /// Protocol invariant: range 0.10-0.50
    pub conservation_threshold: f64,

    /// Credit percentage below which the Death Protocol initiates.
    /// Default: 0.05 (5%)
    /// Protocol invariant: range 0.01-0.20; must be < conservation_threshold
    pub terminal_threshold: f64,

    /// Credit partition allocations. Must sum to 1.0.
    /// Default: { llm: 0.60, gas: 0.25, data: 0.15 }
    pub partitions: CreditPartitions,

    /// Per-partition daily budget caps as fraction of partition balance.
    /// Prevents a single bad day from depleting a partition.
    /// Default: { llm: 0.20, gas: 0.30, data: 0.25 }
    pub daily_caps: CreditPartitions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditPartitions {
    pub llm: f64,
    pub gas: f64,
    pub data: f64,
}
```

### EpistemicConfig

Controls the epistemic mortality clock -- death by informational decay.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicConfig {
    /// Enable epistemic mortality.
    /// When false, fitness is tracked for telemetry but cannot trigger death.
    /// Default: true
    pub enabled: bool,

    /// Rolling window in ticks for fitness computation.
    /// Larger windows smooth noise but delay detection of genuine decay.
    /// Default: 2000 (~22 hours at 40s/tick)
    /// Range: 500-10000
    pub fitness_window: u64,

    /// Fitness score below which the Golem enters senescence.
    /// Senescence starts the grace period countdown.
    /// Default: 0.35
    /// Range: 0.20-0.60
    pub senescence_threshold: f64,

    /// Ticks the Golem has to recover fitness above the senescence threshold.
    /// If fitness remains below threshold for this many ticks, death triggers.
    /// Default: 500 (~5.5 hours)
    /// Range: 100-2000
    pub recovery_grace_period: u64,

    /// Domain-specific decay rates for epistemic fitness.
    /// Different market domains have different knowledge half-lives.
    /// Values in days. Default values based on empirical DeFi observation.
    pub decay_half_life_by_domain: HashMap<String, f64>,

    /// Prediction types tracked for fitness computation.
    /// Each type contributes equally to the composite fitness score.
    /// Default: ["price_direction", "volatility_regime", "fee_rate", "liquidity_depth"]
    pub tracked_prediction_types: Vec<String>,

    /// Minimum number of evaluated predictions required before fitness
    /// score is considered valid. Below this, fitness defaults to 0.5.
    /// Default: 50
    /// Range: 10-500
    pub minimum_predictions: u32,
}
```

### RegimeEvaluationConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeEvaluationConfig {
    /// Number of ticks for baseline measurement window. Default: 50.
    pub baseline_window_ticks: u64,
    /// Number of ticks for treatment measurement window. Default: 50.
    pub treatment_window_ticks: u64,
    /// Minimum matched regime pairs required for valid comparison. Default: 10.
    pub minimum_matched_pairs: u32,
    /// Cohen's d threshold for validated improvement. Default: 0.2.
    pub effect_size_threshold: f64,
    /// KS statistic significance level for change-point detection. Default: 0.05.
    pub ks_significance_level: f64,
    /// Ticks in novel regime before epistemic clock contributes to mortality. Default: 20.
    pub learning_grace_period: u64,
}
```

**Rationale**: Without regime-tagged evaluation, the mortality system cannot distinguish genuine cognitive decay from environmental regime changes. These parameters control the statistical rigor of regime-matched comparisons. The defaults are calibrated for a Golem with 60-second tick intervals operating in DeFi markets.

### StochasticConfig

Controls the stochastic mortality clock -- death by random hazard.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticConfig {
    /// Enable stochastic mortality.
    /// When false, hazard rate is computed for telemetry but never evaluated.
    /// Default: true
    pub enabled: bool,

    /// Base per-tick probability of death for a young, healthy Golem.
    /// At 40s/tick, 1e-6 = ~0.086% chance of death per day.
    /// Default: 1e-6
    /// Range: 1e-8 to 1e-4
    /// Protocol invariant: cannot exceed 1e-3 (would make survival < 50% per day)
    pub base_hazard_rate: f64,

    /// How much age increases the hazard rate.
    /// Hazard contribution from age = age_factor * age_hazard_coefficient.
    /// Default: 0.00001
    /// Range: 0 to 0.001
    pub age_hazard_coefficient: f64,

    /// Multiplier applied to hazard rate when epistemic fitness is below 0.5.
    /// Low-fitness Golems face elevated stochastic mortality.
    /// Default: 2.0
    /// Range: 1.0 to 10.0
    pub epistemic_hazard_multiplier: f64,

    /// Cap on the hazard rate to prevent near-certain death.
    /// Even the most aged, epistemically degraded Golem has at most this
    /// probability of dying per tick.
    /// Default: 0.001 (~8.6% chance of death per day)
    /// Protocol invariant: cannot exceed 0.01
    pub max_hazard_rate: f64,

    /// Seed for the stochastic death PRNG.
    /// If None, uses OsRng (crypto-grade randomness).
    /// Set to a fixed value for reproducible testing.
    /// Default: None
    pub seed: Option<u64>,
}
```

### DemurrageConfig

Controls knowledge carrying costs -- Gesell's Freigeld principle for information.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemurrageConfig {
    /// Enable knowledge demurrage.
    /// When false, Grimoire entries persist at assigned confidence indefinitely.
    /// Default: true (mortal), false (immortal)
    pub enabled: bool,

    /// Ticks between demurrage validation checks.
    /// The Curator scans all active entries at this interval.
    /// Default: 5000 (~2.3 days at 40s/tick)
    /// Range: 1000-20000
    pub validation_interval_ticks: u64,

    /// Confidence decay per interval for unvalidated entries.
    /// An entry not revalidated loses this fraction of its confidence.
    /// Default: 0.05 (5%)
    /// Range: 0.01-0.20
    pub decay_rate_per_interval: f64,

    /// Confidence below which entries are archived (removed from active PLAYBOOK.md).
    /// Archived entries persist in LanceDB cold storage for death reflection.
    /// Default: 0.1
    /// Range: 0.01-0.30
    pub archive_threshold: f64,

    /// Decay class configuration.
    /// Half-lives in days. None = no decay (structural knowledge).
    pub decay_classes: DecayClasses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayClasses {
    /// Protocol mechanics, contract ABIs, chain parameters. Default: None (no decay).
    pub structural: Option<f64>,
    /// Market regime models, correlation structures. Default: 14 (days).
    pub regime_conditional: f64,
    /// Entry/exit signals, spread patterns, gas estimates. Default: 7 (days).
    pub tactical: f64,
    /// Order book snapshots, mempool state, current prices. Default: 1 (day).
    pub ephemeral: f64,
}
```

### PhageConfig

Controls the micro-Replicant (phage) system for continuous self-testing.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhageConfig {
    /// Enable micro-Replicant (phage) spawning.
    /// When false, no background hypothesis testing occurs.
    /// Default: true
    pub enabled: bool,

    /// Number of micro-Replicants spawned per day.
    /// Each phage tests a single hypothesis against the current strategy.
    /// Default: 2
    /// Range: 0-10
    pub daily_spawn_rate: u32,

    /// Maximum lifetime of a micro-Replicant in ticks.
    /// After this many ticks, the phage reports back and dies.
    /// Default: 50 (~33 minutes)
    /// Range: 10-200
    pub max_lifetime_ticks: u64,

    /// Budget per micro-Replicant in USDC.
    /// Limits the cost of each hypothesis test.
    /// Default: 0.05
    /// Range: 0.01-1.00
    pub budget_per_phage: f64,

    /// Minimum Golem vitality required to spawn phages.
    /// No phages in conservation or worse phases.
    /// Default: 0.5 (Stable or better)
    /// Range: 0.3-0.8
    pub minimum_vitality_to_spawn: f64,
}
```

### AffectConfig

Controls the emotional architecture -- how the Golem appraises events and modulates behavior.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectConfig {
    /// Enable the daimon engine.
    /// When false, no emotional appraisal occurs; all decisions are "neutral."
    /// Default: true
    pub enabled: bool,

    /// Appraisal model for emotional computation.
    /// - ChainOfEmotion: LLM-based appraisal using chain-of-thought prompting.
    ///   Most expressive. Cost: 1 Haiku call per non-suppressed tick.
    /// - RuleBased: Deterministic rules mapping events to emotions. $0.00 cost.
    /// - Disabled: No appraisal; emotional context is always Neutral.
    /// Default: ChainOfEmotion
    pub appraisal_model: AppraisalModel,

    /// EMA decay rate for mood state.
    /// Mood is the exponential moving average of recent emotional valence.
    /// Higher = slower mood changes (more stable temperament).
    /// Lower = faster mood changes (more reactive temperament).
    /// Default: 0.95
    /// Range: 0.80-0.99
    pub mood_decay_rate: f64,

    /// Whether mortality state modulates affect.
    /// When true: approaching death amplifies emotional responses.
    /// When false: affect is computed but not mortality-aware.
    /// Default: true (mortal), false (immortal)
    pub mortality_aware_affect: bool,

    /// Grief duration in ticks after a Clade sibling dies.
    /// During grief, the Golem is more cautious and learns faster.
    /// Default: 100
    /// Range: 10-500
    pub grief_duration_ticks: u64,

    /// Whether to record emotional context on Grimoire entries.
    /// Default: true
    pub record_emotional_context: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppraisalModel {
    ChainOfEmotion,
    RuleBased,
    Disabled,
}
```

### ThanatopsisConfig

Controls the enhanced Death Protocol -- emotional life review and knowledge compression.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThanatopsisConfig {
    /// Enable the full Thanatopsis Protocol (enhanced death reflection).
    /// When false, the standard Death Protocol runs (see `06-thanatopsis.md`).
    /// Default: true
    pub enabled: bool,

    /// Budget allocation for death reflection as fraction of Legacy Partition.
    /// Phases: Settle (10%), Reflect (this value), Legacy (remainder).
    /// Default: 0.60 (60%)
    /// Range: 0.30-0.80
    pub reflect_budget_fraction: f64,

    /// Include emotional life review in death reflection.
    /// The Golem reviews its affective arc: what it felt, when, and why.
    /// Default: true
    pub emotional_life_review: bool,

    /// Include unresolved questions in death testament.
    /// "What confused me" and "What I suspect but can't prove."
    /// Default: true
    pub include_unresolved_questions: bool,

    /// Maximum token budget for the death reflection Opus call.
    /// Default: 8000
    /// Range: 2000-32000
    pub max_reflection_tokens: u32,

    /// Progressive death snapshot interval in ticks.
    /// Haiku summaries at this interval as insurance against mid-reflection failure.
    /// Default: 50
    /// Range: 10-200
    pub snapshot_interval_ticks: u64,

    /// Successor recommendation: include specific advice for next generation.
    /// Default: true
    pub generate_successor_recommendation: bool,

    /// Novelty-prioritized packaging for marketplace listing.
    /// Score entries by novelty relative to Clade knowledge.
    /// Default: true
    pub novelty_prioritized_packaging: bool,
}
```

### SuccessionConfig

Controls generational learning -- how knowledge transfers between predecessor and successor.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessionConfig {
    /// Enable owner-initiated succession (not automatic).
    /// When a Golem dies, the owner decides whether to create a successor.
    /// Default: true
    pub enabled: bool,

    /// Confidence applied to inherited Grimoire entries.
    /// Lower values force more independent validation by the successor.
    /// Default: 0.4
    /// Range: 0.1-0.7
    /// Protocol invariant: cannot exceed 0.7 (prevents uncritical inheritance)
    pub inheritance_confidence: f64,

    /// Anti-proletarianization mandate: minimum PLAYBOOK.md divergence.
    /// The successor must modify at least this fraction of inherited entries.
    /// Default: 0.15 (15%)
    /// Range: 0.05-0.50
    pub min_playbook_divergence: f64,

    /// Minimum novel entries the successor must generate (not in predecessor's PLAYBOOK.md).
    /// Default: 5
    /// Range: 1-20
    pub min_novel_entries: u32,

    /// Minimum inherited entries the successor must explicitly invalidate with reasoning.
    /// Default: 3
    /// Range: 0-10
    pub min_invalidated_entries: u32,

    /// Require explicit gaps and open questions in the Grimoire.
    /// Default: true
    pub require_explicit_gaps: bool,

    /// Minimum fraction of Grimoire entries that are questions, not answers.
    /// Default: 0.10 (10%)
    /// Range: 0.0-0.30
    pub min_question_ratio: f64,

    /// Whether to seed the successor with the predecessor's death bundle.
    /// If false, the successor starts fresh (no inherited knowledge).
    /// Default: true
    pub inherit_from_predecessor: bool,

    /// Whether to seed the successor with Clade knowledge.
    /// Default: true
    pub inherit_from_clade: bool,

    /// SUPER (Surprise-Based Experience Sharing) for novelty ranking.
    /// Prioritize inherited entries by how much they diverge from existing knowledge.
    /// Default: true
    pub use_novelty_ranking: bool,
}
```

---

## Default Values

> **Extended:** Full default values tables with rationale columns for all 9 sub-configs (economic, epistemic, stochastic, demurrage, phage, affect, thanatopsis, succession) -- see [../../prd2-extended/02-mortality/13-configuration-extended.md](../../prd2-extended/02-mortality/13-configuration-extended.md)

Default values are specified inline in each sub-config struct above (see doc comments). Key defaults: `death_reserve_floor_usdc: 0.30`, `senescence_threshold: 0.35`, `recovery_grace_period: 500`, `base_hazard_rate: 1e-6`, `decay_rate_per_interval: 0.05`, `mood_decay_rate: 0.95`, `inheritance_confidence: 0.4`, `min_playbook_divergence: 0.15`.

---

## Owner-Tunable Parameters vs Protocol-Enforced Invariants

### Owner-Tunable (Full Range)

These parameters can be set to any value within their specified range. The owner has full control.

| Parameter                           | Range          | Notes                                           |
| ----------------------------------- | -------------- | ----------------------------------------------- |
| `economic.enabled`                       | `bool`          | Owner chooses credit model vs direct payment    |
| `epistemic.enabled`                      | `bool`          | Owner can disable epistemic death               |
| `stochastic.enabled`                     | `bool`          | Owner can disable stochastic death              |
| `demurrage.enabled`                      | `bool`          | Owner can disable knowledge decay               |
| `phage.enabled`                          | `bool`          | Owner can disable micro-Replicants              |
| `affect.enabled`                         | `bool`          | Owner can disable daimon engine                 |
| `affect.appraisal_model`                 | enum            | Owner chooses appraisal model                   |
| `thanatopsis.enabled`                    | `bool`          | Owner can use standard Death Protocol           |
| `succession.enabled`                     | `bool`          | Owner can disable succession                    |
| `immortal`                               | `bool`          | Master switch for all mortality                 |
| `epistemic.fitness_window`               | 500-10000       | Tuning to market volatility                     |
| `epistemic.senescence_threshold`         | 0.20-0.60       | Risk tolerance for informational decay          |
| `epistemic.recovery_grace_period`        | 100-2000        | Patience for recovery attempts                  |
| `stochastic.base_hazard_rate`            | 1e-8 to 1e-4    | Desired level of existential uncertainty        |
| `stochastic.age_hazard_coefficient`      | 0 to 0.001      | How much age increases mortality                |
| `stochastic.seed`                        | `Option<u64>`   | Reproducibility for testing                     |
| `demurrage.validation_interval_ticks`    | 1000-20000      | Knowledge freshness requirements                |
| `demurrage.decay_rate_per_interval`      | 0.01-0.20       | Aggressiveness of knowledge pruning             |
| `affect.mood_decay_rate`                 | 0.80-0.99       | Emotional stability                             |
| `affect.grief_duration_ticks`            | 10-500          | Grief processing time                           |
| `phage.daily_spawn_rate`                 | 0-10            | Self-testing frequency                          |
| `phage.budget_per_phage`                 | 0.01-1.00       | Cost per hypothesis test                        |
| `thanatopsis.reflect_budget_fraction`    | 0.30-0.80       | Investment in death reflection quality          |
| `thanatopsis.max_reflection_tokens`      | 2000-32000      | Depth of death reflection                       |
| `succession.inheritance_confidence`      | 0.1-0.7         | How much to trust predecessor knowledge         |
| `succession.min_playbook_divergence`     | 0.05-0.50       | Innovation requirement per generation           |

### Protocol-Enforced Invariants (Cannot Be Changed)

These values are hardcoded and cannot be modified by the owner. They exist to prevent configurations that would break the system's integrity or safety guarantees.

| Invariant                                       | Value       | Rationale                                                                                                          |
| ----------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------ |
| `death_reserve_floor_usdc` minimum              | `0.10`      | Below $0.10, no meaningful death can execute. The Golem would die necrotic (silent).                               |
| `death_reserve_proportional` range              | `0.01-0.10` | Below 1%, well-funded Golems get trivial deaths. Above 10%, too much capital locked up.                            |
| `conservation_threshold` range                  | `0.10-0.50` | Below 10%, conservation mode never activates. Above 50%, the Golem spends half its life conserving.                |
| `terminal_threshold` < `conservation_threshold` | enforced    | Terminal must come after conservation in the cascade.                                                              |
| `max_hazard_rate` ceiling                       | `0.01`      | Above 1% per tick, the Golem dies with >99.5% probability per day. Not meaningful stochastic mortality.            |
| `inheritance_confidence` ceiling                | `0.7`       | Above 0.7, inherited knowledge is treated as near-certain, preventing independent validation (proletarianization). |
| Apoptotic Reserve is not spendable              | enforced    | The reserve cannot be accessed for operations. Ring-fenced at creation.                                            |
| Apoptotic Reserve is not owner-overridable      | enforced    | The owner cannot reduce the reserve below the computed minimum.                                                    |
| PolicyCage is not modifiable by mortality state  | enforced    | No emotional state, phase transition, or survival pressure can alter on-chain PolicyCage constraints.              |
| Death Protocol is atomic once started            | enforced    | Once the Dying state is entered, the Death Protocol runs to completion. No rollback to Running.                    |
| Partitions must sum to 1.0                       | enforced    | Credit partitions are a complete allocation of the budget.                                                         |

---

## Cognitive Quality Metrics

> **Extended:** 11-metric cognitive quality table (admission rate, quality score, Grimoire size, retrieval hit rate, heuristic survival rate, external metrics, reflection consistency, DecisionCache, dream yield, threat coverage, prediction accuracy) with healthy ranges and alarm thresholds -- see [../../prd2-extended/02-mortality/13-configuration-extended.md](../../prd2-extended/02-mortality/13-configuration-extended.md)

These metrics provide early warning signals for cognitive degradation that precedes mortality triggers. A declining admission rate combined with declining prediction accuracy is a strong signal of senescence.

---

## Telemetry Events

> **Extended:** Full 19-event telemetry table with trigger conditions, payload schemas, and privacy levels (PostHog with HMAC-anonymized identity) -- see [../../prd2-extended/02-mortality/13-configuration-extended.md](../../prd2-extended/02-mortality/13-configuration-extended.md)

19 mortality-related telemetry events: phase transitions, epistemic warnings/recovery, death triggers (senescence, stochastic, economic), death protocol phases, demurrage cycles, phage lifecycle, affect state changes, grief, Clade sharing, immortal tracking, migration steps, succession events. All events use bucketed amounts for financial data and HMAC-anonymized identity for privacy.

---

## Death Cause Enumeration

```rust
/// All possible causes of Golem death.
/// Each variant maps to specific metadata and handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathCause {
    /// Economic clock: USDC depleted.
    CreditExhaustion,
    /// Epistemic clock: fitness decayed beyond recovery.
    EpistemicSenescence,
    /// Stochastic clock: random death check succeeded.
    Stochastic,
    /// Legacy tick counter (if still enabled).
    HayflickLimit,
    /// Prediction accuracy below threshold.
    ModelStaleness,
    /// Owner-initiated kill-switch.
    OwnerKill,
    /// Owner-initiated teardown (not kill).
    OwnerDestroy,
    /// Unrecoverable system failure.
    SystemError,
    /// Golem migrating to different deployment.
    Migration,
    /// Compute TTL expired, VM destroyed.
    VmTerminated,
    /// Hard crash, no graceful death possible.
    CatastrophicFallback,
}
```

### Death Cause Metadata Schemas

> **Extended:** Per-cause metadata structs (CreditExhaustionMeta, EpistemicSenescenceMeta, StochasticDeathMeta, OwnerKillMeta, OwnerDestroyMeta, SystemErrorMeta, and 5 inline variants), DeathCauseMeta enum -- see [../../prd2-extended/02-mortality/13-configuration-extended.md](../../prd2-extended/02-mortality/13-configuration-extended.md)

Each death cause carries typed metadata specific to the cause (e.g., `CreditExhaustionMeta` includes `burn_rate_at_death` and `days_alive`; `StochasticDeathMeta` includes `hazard_rate_at_death` and `roll_value`).

---

## Error Codes and Handling

> **Extended:** MortalityErrorCode enum (22 codes across 5 categories: configuration, runtime, epistemic, phage, succession, migration), error handling rules -- see [../../prd2-extended/02-mortality/13-configuration-extended.md](../../prd2-extended/02-mortality/13-configuration-extended.md)

Configuration errors are fatal at boot. Runtime errors during normal operation are retried. Errors during the Death Protocol do not prevent death -- the last insurance snapshot becomes the final record. Clade push failures retry 3 times with exponential backoff.

---

## Environment Variables and Feature Flags

> **Extended:** 13 environment variables (`BARDO_MORTALITY_ENABLED`, `BARDO_IMMORTAL`, `BARDO_MORTALITY_CONFIG_PATH`, etc.) and 6 feature flags (`MORTALITY_EPISTEMIC_V2`, `MORTALITY_STOCHASTIC_GOMPERTZ`, etc.) -- see [../../prd2-extended/02-mortality/13-configuration-extended.md](../../prd2-extended/02-mortality/13-configuration-extended.md)

Key env vars: `BARDO_MORTALITY_ENABLED` (master switch), `BARDO_IMMORTAL` (shorthand for immortal mode), `BARDO_STOCHASTIC_SEED` (deterministic PRNG for testing), `BARDO_EXPERIMENT_ENABLED` (immortal vs mortal experiment tracking).

---

## Testing, Examples, and Validation

> **Extended:** Full devenv config (accelerated mortality with deterministic PRNG), test utilities (`create_test_mortality_config`, `create_test_vitality`, `advance_mortality_ticks`, `trigger_test_death`), 3 example configurations (conservative/aggressive/research with paired immortal control), `validate_mortality_config()` implementation with range assertions, invalid combination tables with error messages -- see [../../prd2-extended/02-mortality/13-configuration-extended.md](../../prd2-extended/02-mortality/13-configuration-extended.md)

Dev mode uses 10x shorter windows, 100x higher hazard rates, deterministic seed (42), rule-based affect (no LLM cost), and disabled phages/thanatopsis. Three example profiles: conservative (2-3x longer lifespan, gentler pressure), aggressive (0.3-0.5x lifespan, rapid turnover), and research (maximum instrumentation for the immortal vs mortal experiment).

---

## Deployment-Level Configuration: golem.toml

For self-hosted deployments, mortality parameters are specified in `golem.toml` under the `[mortality]` section. The configuration file uses TOML format, consistent with the Rust ecosystem convention (Cargo.toml, rustfmt.toml).

```toml
[mortality]
death_reserve_usdc = 0.30       # Minimum USDC reserved for Death Protocol
staleness_threshold = 0.35      # Epistemic fitness below which senescence cascade begins
staleness_window = 500          # Grace period in ticks before senescence is confirmed

[mortality.economic]
enabled = true                  # false for self-hosted
conservation_threshold = 0.30   # 30% credits triggers Conservation phase

[mortality.stochastic]
enabled = true
base_hazard_rate = 1e-6         # Per-tick baseline probability
max_hazard_rate = 0.001         # Cap to prevent near-certain death

[mortality.affect]
enabled = true
appraisal_model = "chain-of-emotion"  # "chain-of-emotion" | "rule-based" | "disabled"
```

```rust
/// Parsed and validated from golem.toml [mortality] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityTomlConfig {
    /// Minimum USDC reserved for Death Protocol. Default: 0.30.
    #[serde(default = "default_death_reserve")]
    pub death_reserve_usdc: f64,
    /// Epistemic fitness below which senescence cascade begins. Default: 0.35.
    #[serde(default = "default_staleness_threshold")]
    pub staleness_threshold: f64,
    /// Grace period in ticks before senescence is confirmed. Default: 500.
    #[serde(default = "default_staleness_window")]
    pub staleness_window: u64,
    /// Economic mortality sub-config.
    pub economic: Option<EconomicTomlConfig>,
    /// Stochastic mortality sub-config.
    pub stochastic: Option<StochasticTomlConfig>,
    /// Affect engine sub-config.
    pub affect: Option<AffectTomlConfig>,
}
```

| Deployment Mode      | TTL Enforcement           | Hayflick Limit          | Death Reserve    |
| -------------------- | ------------------------- | ----------------------- | ---------------- |
| Hosted (Fly.io)      | Yes (TTL from credits)    | 100,000 (default)       | $0.30 (minimum)  |
| Self-hosted (Docker) | No (owner bears costs)    | 0 (disabled by default) | $0.00 (optional) |
| Dev (local Anvil)    | No                        | 0                       | $0.00            |

> **Cross-ref**: `runtime/prd/10-packaging-deployment.md` (golem.toml schema, deployment modes)

---

## Dream Configuration Cross-Reference

Dream configuration is specified in `../05-dreams/01-architecture.md`. Key mortality-relevant config: `dream.budget_fraction` (fraction of inference budget allocated to dreaming, default 0.08), `dream.phase_scaling` (per-phase budget multipliers), `dream.terminal_cutoff` (vitality threshold below which dreaming ceases, default 0.15).

---

## References

- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012.
- [DAVIS-ZHONG-2017] Davis, R.L. & Zhong, Y. "The Biology of Forgetting." _Neuron_ 95(5), 2017.
- [DOHARE-2024] Dohare, S. et al. "Loss of Plasticity in Deep Continual Learning." _Nature_ 632, 2024.
- [GESELL-1916] Gesell, S. _The Natural Economic Order_. 1916.
- [PEREZ-2024] Perez, J. et al. "Artificial Generational Intelligence." arXiv:2406.00392, 2024.
- [STIEGLER-2018] Stiegler, B. _The Neganthropocene_. Open Humanities Press, 2018.
- [VELA-2022] Vela, D. et al. "Temporal Quality Degradation in AI Models." _Scientific Reports_ 12, 2022.
