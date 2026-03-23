# 17 -- The Prediction Engine [SPEC]

**Domain-Agnostic Oracle, Residual Correction, Attention Foraging, and Action Gating**

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-oracle` | **Layer**: 0 (FOUNDATION)
>
> Cross-references: [01-cognition.md](./01-cognition.md), [02-heartbeat.md](./02-heartbeat.md) (adaptive clock), [18-cortical-state.md](./18-cortical-state.md) (CorticalState), [13-runtime-extensions.md](./13-runtime-extensions.md), [14-context-governor.md](./14-context-governor.md)
>
> Sources: `active-inference/02-prediction-engine`, `mmo2/20-prediction-engine`

> **Reader orientation:** This document specifies the Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) prediction engine -- a domain-agnostic Oracle that provides learning without LLM self-grading, attention allocation, and action gating. It belongs to the `01-golem` cognition layer, in the `golem-oracle` crate. The key concept: predictions resolve against external reality with ~15,000 residual corrections per day at zero inference cost. The Golem earns the right to act by demonstrating prediction accuracy, structurally preventing over-trading. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## What the Prediction Engine Is

The prediction engine is a subsystem -- not the entire architecture -- that provides three capabilities:

1. **Learning without LLM self-grading.** Predictions resolve against external reality. Residual correction improves future predictions automatically. ~15,000 corrections/day at zero inference cost.
2. **Attention allocation.** Items with high prediction error get more monitoring resources. Items with low error get less. The Golem discovers what to watch.
3. **Action gating.** The Golem earns the right to act by demonstrating prediction accuracy. This structurally prevents over-trading.

The engine is domain-agnostic. Domain knowledge enters through the `PredictionDomain` trait. DeFi is one implementation. A golem that monitors weather, shipping routes, or sports betting lines implements the same trait with different categories and resolution logic. Everything downstream -- the ledger, the corrector, the action gate, the calibrator -- works identically regardless of domain.

The feedback signal is arithmetic: the golem predicted X, reality was Y, the residual is Y - X. That residual corrects the next prediction automatically. No LLM self-grading. No reward models. No RLHF.

Derivation of ~15,000/day: ~200 tracked items x gamma frequency (~one update per 6s) x ~7,200 active seconds/day = ~14,400 adjustments/day, rounded to ~15,000.

Accuracy ranges cited throughout this document are aspirational targets measured over a 30-day rolling window against live market data. They are not guaranteed and will vary with market conditions, golem maturity, and strategy type.

---

## The PredictionDomain Trait

Any observable, measurable environment implements this trait to teach the Oracle what to predict, how to check, and how to discover new items.

```rust
/// Domain abstraction that makes the Oracle domain-agnostic.
/// DeFi discovers pools and reads on-chain state. Weather discovers
/// stations and reads sensors. The Oracle processes Prediction structs
/// regardless of origin.
#[async_trait]
pub trait PredictionDomain: Send + Sync + 'static {
    fn domain_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn categories(&self) -> Vec<PredictionCategory>;

    /// Discover trackable items from a strategy seed.
    /// Called at boot and during Phase 3 continuous scanning.
    async fn discover(
        &self,
        seed: &AttentionSeed,
        env: &dyn EnvironmentClient,
    ) -> Result<Vec<TrackedItem>>;

    /// Scan for newly appeared items not in the known set.
    /// Called every ~100 theta ticks.
    async fn discover_new(
        &self,
        seed: &AttentionSeed,
        known: &HashSet<ItemId>,
        env: &dyn EnvironmentClient,
    ) -> Result<Vec<TrackedItem>>;

    /// SCANNED tier: cheap, context-free. 1 prediction per item.
    fn predict_scanned(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
    ) -> Vec<PredictionDraft>;

    /// WATCHED tier: has cortical state. 2-4 predictions per item.
    fn predict_watched(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
    ) -> Vec<PredictionDraft>;

    /// ACTIVE tier: full context. 5-12 predictions per item.
    fn predict_active(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
        playbook: &PlaybookState,
    ) -> Vec<PredictionDraft>;

    /// Read ground truth for resolution. eth_call for DeFi,
    /// sensor API for weather, etc.
    async fn resolve(
        &self,
        prediction: &Prediction,
        checkpoint: &Checkpoint,
        env: &dyn EnvironmentClient,
    ) -> Result<ResolutionOutcome>;

    /// Predict action outcomes before execution.
    fn predict_action(
        &self,
        action: &ProposedAction,
        item: &TrackedItem,
        history: &ResidualBuffer,
    ) -> Vec<PredictionDraft>;

    /// Predict inaction outcomes. Explicit "holding is optimal" claim.
    fn predict_inaction(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
    ) -> Vec<PredictionDraft>;
}
```

### The EnvironmentClient trait

The `EnvironmentClient` abstracts external state. For DeFi, it wraps an Alloy provider for `eth_call` reads. For other domains, it wraps whatever service provides ground truth. The Oracle never writes through this interface -- all writes go through the action pipeline.

```rust
#[async_trait]
pub trait EnvironmentClient: Send + Sync {
    async fn read(&self, query: &EnvironmentQuery) -> Result<EnvironmentValue>;
    async fn read_batch(&self, queries: &[EnvironmentQuery]) -> Result<Vec<EnvironmentValue>>;
    fn env_timestamp(&self) -> u64;
}
```

---

## The Prediction Primitive

Every prediction the golem makes -- analytical, corrective, creative, collective -- is the same struct. This uniformity is what makes the residual corrector and the action gate domain-agnostic. A fee-rate prediction, a price-direction prediction, and a "holding is optimal" prediction all resolve through the same pipeline.

```rust
/// A single falsifiable claim about the future.
/// ~200 bytes per prediction. At 15,000/day, ~3 MB before compaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub id: PredictionId,                       // Monotonic within lifetime
    pub created_at_tick: u64,                   // Theta tick of creation
    pub domain: DomainId,                       // Which domain produced this
    pub category: PredictionCategory,           // Accuracy tracked per (category, regime)
    pub source: PredictionSource,               // Which intelligence mode
    pub claim: PredictionClaim,                 // The falsifiable claim
    pub checkpoints: Vec<Checkpoint>,           // Multi-scale resolution points
    pub tracked_item: Option<TrackedItemId>,    // The item this concerns
    pub action_ref: Option<ActionRef>,          // If action-related
    pub regime: RegimeTag,                      // Market regime at prediction time
    pub pad: PadVector,                         // Emotional state at prediction time
    pub confidence: Option<f64>,                // Calibrated confidence [0.0, 1.0]
    pub correction_applied: Option<ResidualCorrection>,
}
```

### Source tags

```rust
/// Which intelligence mode generated this prediction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PredictionSource {
    /// Waking theta cycles. ~14,000/day.
    Analytical { tier: CognitiveTier },
    /// ResidualCorrector adjustments. Zero inference cost.
    Corrective { adjustment: AdjustmentType },
    /// Dream cycles. ~34% expected accuracy, high value when correct.
    Creative { phase: CreativePhase },
    /// Clade sibling or Lethe via Styx. Weighted at 0.7x.
    Collective { source_golem: Option<GolemId>, layer: StyxLayer },
    /// Retrospective pattern recognition across past sequences.
    Retrospective { sequence_ids: Vec<PredictionId> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AdjustmentType {
    BiasCorrection,
    IntervalCalibration,
    Combined,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CreativePhase {
    Hypnagogic,
    NremReplay,
    RemImagination,
    Integration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StyxLayer {
    Clade,  // Private fleet (same owner). Full residual sharing.
    Lethe,  // Public anonymized aggregates.
}
```

### Claim variants

```rust
/// A falsifiable claim resolved by arithmetic comparison.
/// Every variant reduces to a boolean: correct or wrong, no subjectivity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionClaim {
    /// |observed - center| <= center * tolerance_bps / 10000
    WithinBps { center: f64, tolerance_bps: u64 },
    /// lower <= observed <= upper
    InRange { lower: f64, upper: f64 },
    /// observed > threshold
    Above(f64),
    /// observed < threshold
    Below(f64),
    /// (observed - baseline).signum() == direction
    Direction(Direction),
    /// observed_bool == predicted_bool
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction { Up, Down }
```

### Checkpoints

A prediction can have multiple checkpoints for multi-scale resolution. A fee-rate prediction might check at 5 minutes (does the rate still hold?), 1 hour (trend confirmation), and 24 hours (structural accuracy). Each checkpoint produces an independent resolution and residual.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub resolve_at_tick: u64,
    pub query: EnvironmentQuery,
    pub status: CheckpointStatus,
    pub resolution: Option<ResolutionOutcome>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CheckpointStatus { Pending, Resolved, Failed, Expired }
```

---

## The Prediction Ledger

The ledger is append-only SQLite storage for predictions and their resolutions. Every prediction the golem creates goes here. Every resolution outcome goes here. The ledger is the single source of truth for the golem's predictive track record.

### Storage economics

| Metric | Value |
|--------|-------|
| Bytes per prediction | ~200 |
| Bytes per resolution | ~100 |
| Daily predictions (full throughput) | ~15,000 |
| Daily resolutions | ~15,000 |
| Daily storage growth | ~4.5 MB |
| Weekly before compaction | ~31.5 MB |
| After compaction (7-day window) | ~4.5 MB |

Compaction runs at delta frequency (every few hours). It removes resolved predictions older than the residual buffer window (configurable, default 7 days). The residual statistics survive compaction -- only the raw prediction records are pruned.

### SQL schema

```sql
CREATE TABLE predictions (
    id              INTEGER PRIMARY KEY,
    created_at_tick INTEGER NOT NULL,
    domain          TEXT    NOT NULL,
    category        TEXT    NOT NULL,
    source          TEXT    NOT NULL,  -- JSON: PredictionSource
    claim           TEXT    NOT NULL,  -- JSON: PredictionClaim
    tracked_item    TEXT,
    action_ref      TEXT,
    regime          TEXT    NOT NULL,
    confidence      REAL,
    confidence_raw  REAL,             -- Pre-calibration confidence
    correction      TEXT,             -- JSON: ResidualCorrection
    pad_pleasure    REAL    NOT NULL,
    pad_arousal     REAL    NOT NULL,
    pad_dominance   REAL    NOT NULL,
    created_at      INTEGER NOT NULL  -- Unix timestamp
);

CREATE TABLE checkpoints (
    id            INTEGER PRIMARY KEY,
    prediction_id INTEGER NOT NULL REFERENCES predictions(id),
    resolve_tick  INTEGER NOT NULL,
    query         TEXT    NOT NULL,  -- JSON: EnvironmentQuery
    status        TEXT    NOT NULL DEFAULT 'pending',
    actual_value  REAL,
    residual      REAL,
    correct       INTEGER,          -- 0 or 1
    resolved_at   INTEGER,          -- Unix timestamp
    FOREIGN KEY (prediction_id) REFERENCES predictions(id)
);

-- Hot path: category accuracy queries
CREATE INDEX idx_pred_category_regime
    ON predictions(category, regime, created_at_tick);

-- Hot path: pending checkpoint resolution
CREATE INDEX idx_checkpoint_pending
    ON checkpoints(status, resolve_tick)
    WHERE status = 'pending';

-- Hot path: recent resolutions for corrector
CREATE INDEX idx_checkpoint_resolved
    ON checkpoints(prediction_id, status, resolved_at)
    WHERE status = 'resolved';

-- Compaction target: old resolved predictions
CREATE INDEX idx_pred_compaction
    ON predictions(created_at)
    WHERE id NOT IN (
        SELECT prediction_id FROM checkpoints WHERE status = 'pending'
    );
```

### Rust interface

The ledger never updates a prediction after registration. Resolutions are appended. Compaction deletes old rows but never modifies them. This append-only invariant means the ledger can be replayed from any point for auditing.

```rust
pub struct PredictionLedger {
    db: SqlitePool,                                                 // WAL-enabled
    index: DashMap<(CategoryId, RegimeTag), Vec<PredictionId>>,     // Hot-path cache
    pending_count: AtomicU32,                                       // TUI display
}

impl PredictionLedger {
    pub fn register(&self, prediction: Prediction) -> PredictionId { /* append */ }
    pub fn resolve(&self, id: PredictionId, checkpoint_idx: usize,
        outcome: ResolutionOutcome) -> Resolution { /* resolve + compute residual */ }
    pub fn category_accuracy(&self, category: CategoryId,
        regime: Option<RegimeTag>, window: Duration) -> AccuracyReport { /* query */ }
    pub fn compact(&self, retain_window: Duration) -> CompactionResult { /* prune */ }
}

#[derive(Debug, Clone)]
pub struct AccuracyReport {
    pub category: CategoryId,
    pub regime: Option<RegimeTag>,
    pub window: Duration,
    pub total: u64,
    pub hits: u64,
    pub hit_rate: f64,
    pub mean_residual: f64,
    pub mean_abs_residual: f64,
    pub mean_interval_width: f64,
    pub sample_sufficient: bool, // true if total >= 30
}
```

### Resolution protocol

Resolution is event-driven with fallback polling. Primary: subscribe to on-chain events (price updates, position changes) that trigger resolution. Fallback: poll every 5 minutes for predictions past `checkpoint_tick`. Chain unreachable: mark prediction as `Unresolved` after 3 poll attempts; retry on next successful connection.

### CategoryId enum

```rust
pub enum CategoryId {
    PriceDirection,     // Token price up/down
    PriceRange,         // Price within range at checkpoint
    VolatilityRegime,   // Low/medium/high/extreme
    LiquidityDepth,     // Pool TVL changes
    FeeGeneration,      // LP fee income predictions
    GasPrice,           // Network gas cost
    ProtocolEvent,      // Governance, upgrades, exploits
    CrossChain,         // Bridge flow, arbitrage opportunities
}
```

### Cold-start strategy

Cold start (first ~20 predictions): use T0-only predictions with wide confidence intervals (0.3-0.5). Seed attention from STRATEGY.md specified domains. Calibrator activates after 20 resolved predictions. Accuracy tracking begins but is not used for gating until 50 predictions.

---

## The Residual Corrector

The fast learning loop. Runs on every prediction resolution at gamma frequency. Zero inference cost. Pure arithmetic.

Two operations, both from conformal prediction theory [VOVK-2005]:

1. **Bias correction.** If the mean residual for a (category, regime) pair is non-zero, shift future prediction centers toward observed means.
2. **Interval calibration.** If the coverage rate (fraction of predictions where the actual value fell within the predicted interval) deviates from the target (default 85%), adjust prediction widths.

The theoretical foundation is conformal prediction (Vovk, Gammerman, and Shafer 2005), recently made accessible by Angelopoulos and Bates (2023). For handling non-stationary environments, the corrector draws on adaptive conformal inference (Gibbs and Candes 2021), which maintains valid coverage even when the data distribution shifts over time.

```rust
pub struct ResidualCorrector {
    buffers: DashMap<(CategoryId, RegimeTag), ResidualBuffer>,
    novel_residuals: Mutex<Vec<Resolution>>,  // Large creative errors -> dream replay seeds
    novelty_threshold: f64,                   // Residual magnitude cutoff for novelty
    forgetting_rate: f64,                     // Gibbs & Candes (2021). Default: 0.005
}

/// Per-(category, regime) residual statistics.
/// Uses Welford's online algorithm for numerically stable mean/variance.
pub struct ResidualBuffer {
    residuals: CircularBuffer<f64, 256>,  // Ring buffer, 2 KB per pair
    mean: f64,                            // Welford's online mean
    variance: f64,                        // Welford's online variance
    count: u64,                           // Lifetime observation count
    coverage: f64,                        // Fraction of predictions containing actual value
    target_coverage: f64,                 // Default 0.85
    adaptive_weight: f64,                 // Gibbs & Candes forgetting accumulator
}

impl ResidualCorrector {
    /// Called on every prediction resolution (gamma frequency).
    pub fn record(&self, resolution: &Resolution) {
        let key = (resolution.category, resolution.regime);
        let mut buffer = self.buffers.entry(key).or_default();

        buffer.push(resolution.residual);
        buffer.update_coverage(resolution.correct);

        // Adaptive weight update (Gibbs & Candes, 2021).
        let coverage_error = if resolution.correct { 1.0 } else { 0.0 }
            - buffer.target_coverage;
        buffer.adaptive_weight += self.forgetting_rate * coverage_error;

        // Novel creative residuals drain to dream replay seeds.
        if matches!(resolution.source, PredictionSource::Creative { .. })
            && resolution.residual.abs() > self.novelty_threshold
        {
            self.novel_residuals.lock().push(resolution.clone());
        }
    }

    /// Applied to every new prediction before registration.
    /// Returns None if insufficient data (< 10 observations).
    pub fn correct(&self, draft: &mut PredictionDraft) -> Option<ResidualCorrection> {
        let key = (draft.category, draft.regime);
        let buffer = self.buffers.get(&key)?;

        if buffer.count < 10 { return None; }

        // 1. Bias correction: shift center toward observed mean.
        let bias = buffer.mean;
        let center_adjustment = -bias;

        // 2. Interval calibration: widen if under-covering, narrow if over-covering.
        let coverage_ratio = buffer.coverage / buffer.target_coverage;
        let width_factor = if coverage_ratio < 0.95 {
            1.0 + (1.0 - coverage_ratio) * 0.5
        } else if coverage_ratio > 1.05 {
            1.0 - (coverage_ratio - 1.0) * 0.3
        } else {
            1.0
        };

        draft.apply_correction(center_adjustment, width_factor);

        Some(ResidualCorrection {
            bias_adjustment: center_adjustment,
            width_factor,
            sample_size: buffer.count,
            mean_residual: bias,
            coverage_before: buffer.coverage,
        })
    }

    /// Aggregate for Clade sharing (delta frequency).
    pub fn aggregate_statistics(&self) -> ResidualDigest {
        let mut entries = Vec::new();
        for entry in self.buffers.iter() {
            let (key, buffer) = entry.pair();
            entries.push(ResidualDigestEntry {
                category: key.0,
                regime: key.1,
                mean: buffer.mean,
                variance: buffer.variance,
                count: buffer.count,
                coverage: buffer.coverage,
            });
        }
        ResidualDigest { entries }
    }

    /// Drain novel residuals for dream replay. Called at dream onset.
    pub fn drain_novel_residuals(&self) -> Vec<Resolution> {
        std::mem::take(&mut *self.novel_residuals.lock())
    }
}
```

### Convergence properties

Bias correction converges at O(1/sqrt(N)). At 600 resolutions per day per category (a typical ACTIVE-tier item), systematic biases surface within hours. At 50 clade members sharing residual statistics, convergence is ~7x faster because the effective sample size multiplies across the fleet.

The convergence rate is a direct consequence of the Central Limit Theorem applied to residual means. The standard error of the mean residual after N observations is sigma/sqrt(N), where sigma is the residual standard deviation. For a category with sigma = 50 bps (a typical fee-rate prediction), the bias estimate after 100 observations has standard error = 5 bps. After 600 observations, standard error = 2 bps.

### Known limitations

**Exchangeability.** The corrector assumes approximate exchangeability within (category, regime) windows. This holds for fee rates during a stable regime but breaks across regime transitions. When the regime tag changes, the corrector resets the affected buffers, accepting a cold-start penalty of 1-2 hours of degraded accuracy. The adaptive forgetting factor (Gibbs and Candes 2021) softens this penalty by downweighting stale observations even within a regime, but cannot eliminate it.

**Goodhart's law.** Wide predictions game accuracy. If the golem predicts "the fee rate will be between $0 and $1000/hr," it is always correct and always useless. The architecture mitigates this by tracking multiple quality metrics: accuracy (hit rate), mean interval width, residual magnitude, and Expected Calibration Error. The action gate uses accuracy. The Grimoire uses ECE. The TUI displays all four. No single metric controls everything, which makes gaming any one metric self-defeating.

**Corrector plateau.** The corrector handles systematic linear biases well. It does not handle structural model errors. If the golem's underlying prediction model is wrong in a non-linear way (e.g., it underestimates tail risks), the corrector will shift the center but won't fix the shape of the distribution. Structural errors require creative-mode discoveries or Grimoire heuristic updates, both of which operate on longer timescales.

---

## The Attention Forager

### Three-tier hierarchy

| Tier | Evaluation rate | Prediction depth | Item count | Purpose |
|------|----------------|-----------------|------------|---------|
| **ACTIVE** | Every theta tick | 5-12 predictions | 5-15 | Current positions, high-signal candidates |
| **WATCHED** | Every 3-5 theta ticks | 2-4 predictions | 30-60 | Recent anomalies, promising candidates |
| **SCANNED** | Every ~100 theta ticks | 1 prediction | 100-500+ | Background monitoring, discovery |

Items promote on surprise (prediction violations). Items demote on boredom (no violations for 10+ delta cycles). Current positions are always ACTIVE regardless of prediction error -- you don't stop watching something you own.

```rust
/// Three-tier attention system. The golem forages for prediction
/// errors the way an animal forages for food: allocate resources
/// where the model of the world is most wrong.
pub struct AttentionForager {
    tiers: [AttentionTier; 3],
    promotion_threshold: f64,
    demotion_patience: u32,
}

pub struct AttentionTier {
    pub level: TierLevel,
    pub items: Vec<TrackedItem>,
    pub max_items: usize,
    pub eval_frequency: u32,  // theta ticks between evaluations
}

#[derive(Debug, Clone, Copy)]
pub enum TierLevel { Active, Watched, Scanned }
```

### Promotion and demotion

```rust
impl AttentionForager {
    /// promotion_score = anomaly_score * sqrt(consecutive_surprises)
    pub fn promotion_score(&self, item: &TrackedItem) -> f64 {
        let anomaly = item.recent_anomaly_score();
        let consecutive = item.consecutive_surprises() as f64;
        anomaly * consecutive.sqrt()
    }

    pub fn tick(&mut self, tick: u64, ledger: &PredictionLedger) {
        // ACTIVE items: every tick.
        // WATCHED: every 3-5 ticks.
        if tick % self.tiers[1].eval_frequency as u64 == 0 {
            self.evaluate_tier(TierLevel::Watched, ledger);
        }
        // SCANNED: every ~100 ticks.
        if tick % self.tiers[2].eval_frequency as u64 == 0 {
            self.evaluate_tier(TierLevel::Scanned, ledger);
        }
        // Promote items whose score exceeds threshold.
        self.promote_candidates();
        // Demote items that haven't surprised in N delta cycles.
        self.demote_stale();
    }
}
```

### Discovery phases

**Phase 1 (boot).** The owner's STRATEGY.md is parsed into an `AttentionSeed` -- a structured representation of what the golem should watch. Pool addresses, token pairs, protocol names, chain IDs. The seed is passed to each `PredictionDomain::discover()` method, which returns initial items.

**Phase 2 (first 100 ticks).** Graph expansion from seed items. The golem discovers related items: same token on different venues, same venue with different fee tiers, composed protocols (e.g., a lending protocol that uses the pool's LP tokens as collateral). This is information foraging patch exploration (Pirolli and Card 1999): the golem explores the neighborhood of its initial information sources.

**Phase 3 (continuous).** Every ~100 theta ticks, `PredictionDomain::discover_new()` scans for items that have appeared since the last scan. Newly deployed pools, newly listed tokens, newly created vaults. These enter at the SCANNED tier and promote based on prediction error.

### Connection to foraging theory

The attention forager is an application of optimal foraging theory (Stephens and Krebs 1986) to information. Charnov's marginal value theorem (1976) predicts that an optimal forager should leave a patch when the marginal return drops below the average return across all patches. The golem's analog: an item should demote from ACTIVE to WATCHED when its marginal prediction-error yield drops below the average across all ACTIVE items. The golem stops paying full attention to an item not because it becomes uninteresting in absolute terms, but because other items are yielding more prediction errors per cognitive unit invested.

The information foraging variant (Pirolli and Card 1999) adds information scent. Items emit "scent" proportional to their prediction error, and the golem follows the strongest scent. Graph expansion during Phase 2 is a scent-following behavior -- the golem discovers new items by following links from high-error items to their neighbors.

### Known limitation

**Most SCANNED predictions are low-value noise.** If the golem scans 200 items with 1 prediction per hour each, that produces 4,800 trivial predictions per day ("this pool's fee rate is still approximately $X"). The corrections on these predictions are real but low-signal. The genuinely useful volume is the 200-500 predictions on ACTIVE items with full context. The headline "15,000 predictions/day" includes a lot of background noise. The per-category accuracy breakdown in the TUI makes this transparent -- SCANNED-tier accuracy runs at ~92% (because the predictions are conservative) while ACTIVE-tier accuracy is lower but more informative.

---

## The Action Gate

The action gate prevents over-trading. A golem cannot execute an action unless its predictions about the relevant categories have demonstrated sufficient accuracy. No accuracy record, no action.

```rust
pub struct ActionGate {
    category_threshold: f64,    // Default 0.60
    inaction_comparison: bool,  // Default true
    inaction_margin: f64,       // Default 0.05 (5 percentage points)
}

#[derive(Debug, Clone)]
pub enum GateDecision {
    Permitted,
    Blocked { reason: String },
}

impl ActionGate {
    pub fn permits(
        &self,
        action_predictions: &[Prediction],
        ledger: &PredictionLedger,
    ) -> GateDecision {
        let mut min_action_accuracy = f64::MAX;
        for pred in action_predictions {
            let report = ledger.category_accuracy(
                pred.category.id, Some(pred.regime),
                Duration::from_secs(7 * 86400),
            );
            if !report.sample_sufficient {
                return GateDecision::Blocked { reason: format!(
                    "Insufficient data for '{}': {} < 30 samples",
                    pred.category.name, report.total) };
            }
            if report.hit_rate < self.category_threshold {
                return GateDecision::Blocked { reason: format!(
                    "'{}' accuracy {:.1}% < {:.1}% threshold",
                    pred.category.name, report.hit_rate * 100.0,
                    self.category_threshold * 100.0) };
            }
            min_action_accuracy = min_action_accuracy.min(report.hit_rate);
        }

        if self.inaction_comparison {
            let inaction = ledger.category_accuracy(
                CategoryId::INACTION, None, Duration::from_secs(7 * 86400));
            if inaction.sample_sufficient
                && inaction.hit_rate > min_action_accuracy + self.inaction_margin
            {
                return GateDecision::Blocked { reason: format!(
                    "Inaction {:.1}% > action {:.1}% + {:.1}% margin",
                    inaction.hit_rate * 100.0, min_action_accuracy * 100.0,
                    self.inaction_margin * 100.0) };
            }
        }
        GateDecision::Permitted
    }
}
```

### Cost/value scaling

The default threshold (60%) is a floor. For high-stakes actions, the gate raises the bar:

```rust
impl ActionGate {
    /// required_accuracy = 0.50 + min(cost / expected_value, 0.45)
    /// Capped at 0.95 to avoid requiring impossible accuracy.
    pub fn required_accuracy(
        &self,
        cost_usdc: f64,
        expected_value_usdc: f64,
    ) -> f64 {
        let ratio = if expected_value_usdc > 0.0 {
            cost_usdc / expected_value_usdc
        } else {
            1.0
        };
        let scaled = 0.50 + ratio.min(0.45);
        scaled.max(self.category_threshold)
    }
}
```

### Inaction prediction

Every suppressed theta tick generates an explicit prediction: "I predict holding is optimal." This resolves by comparing portfolio value at prediction time vs. resolution time. If inaction accuracy exceeds action accuracy, the action gate blocks trades. Patience emerges from the math.

### Known limitation: sideways market conservatism

In flat markets, inaction accuracy runs around 95% because holding is almost always correct when prices don't move. The gate becomes very conservative, blocking the golem even when genuine opportunities appear. The inaction comparison margin (default 5%) softens this: the golem can act when its action accuracy is within 5 percentage points of inaction accuracy. The owner can lower this margin in `golem.toml` to allow more aggressive behavior in flat regimes, accepting higher risk. The TUI displays the margin and both accuracy rates, so the tradeoff is visible.

---

## The Confidence Calibrator

### The problem

LLMs are systematically overconfident. Xiong et al. (2023) found that LLMs verbalize confidence in the 80-100% range even when actual accuracy sits at 50-60%. Geng et al. (2024) confirmed: LLMs produce poorly calibrated confidence estimates out of the box, with ECE typically in the 0.15-0.30 range.

Without calibration, the `confidence` field on `Prediction` is unreliable. The action gate would trust a stated 80% confidence that corresponds to 55% actual accuracy.

### The solution: isotonic regression

A post-hoc calibration module that learns the mapping from stated confidence to actual accuracy, per (category, regime), and applies a correction to every new prediction's confidence before it enters the ledger.

Isotonic regression is chosen over temperature scaling (Guo et al. 2017) for three reasons: no parametric assumptions about the calibration curve shape; monotonicity guarantee (higher stated confidence always maps to higher calibrated confidence); and it handles the clustered confidence distribution of LLMs (most values in 80-100% range) without overfitting to sparse regions.

```rust
pub struct ConfidenceCalibrator {
    curves: DashMap<(CategoryId, RegimeTag), IsotonicCurve>,
    min_samples: usize,                                  // Default: 30
    ece: DashMap<(CategoryId, RegimeTag), f64>,           // ECE per (category, regime)
}

pub struct IsotonicCurve {
    bins: Vec<CalibrationBin>,
    history: CircularBuffer<(f64, bool), 2048>,
    sample_count: u64,
    max_history: usize,
}

#[derive(Debug, Clone)]
pub struct CalibrationBin {
    pub lower: f64,
    pub upper: f64,
    pub observed_accuracy: f64,
    pub sample_count: u32,
}

impl ConfidenceCalibrator {
    /// Called on every prediction resolution (gamma frequency).
    pub fn record(
        &self,
        stated: f64,
        correct: bool,
        category: CategoryId,
        regime: RegimeTag,
    ) {
        let key = (category, regime);
        let mut curve = self.curves.entry(key).or_default();
        curve.add_sample(stated, correct);

        // Refit every 50 samples to amortize the O(N log N) sort cost.
        if curve.sample_count % 50 == 0 {
            curve.refit();
            let ece = curve.compute_ece();
            self.ece.insert(key, ece);
        }
    }

    /// Maps stated confidence to calibrated value before registration.
    pub fn calibrate(
        &self,
        stated: f64,
        category: CategoryId,
        regime: RegimeTag,
    ) -> f64 {
        let key = (category, regime);
        match self.curves.get(&key) {
            Some(curve) if curve.sample_count >= self.min_samples as u64 => {
                curve.map(stated)
            }
            _ => stated,
        }
    }

    /// ECE for a specific (category, regime).
    /// 0.0 = perfect, <0.05 = good, 0.05-0.15 = moderate, >0.15 = bad.
    pub fn ece(&self, category: CategoryId, regime: RegimeTag) -> Option<f64> {
        self.ece.get(&(category, regime)).map(|v| *v)
    }
}

impl IsotonicCurve {
    pub fn add_sample(&mut self, stated: f64, correct: bool) {
        self.history.push((stated, correct));
        self.sample_count += 1;
    }

    /// Refit via Pool Adjacent Violators (PAV) algorithm.
    pub fn refit(&mut self) { /* bin by stated confidence, compute observed accuracy,
                                  merge adjacent bins that violate monotonicity */ }

    pub fn map(&self, stated: f64) -> f64 {
        for bin in &self.bins {
            if stated >= bin.lower && stated < bin.upper {
                return bin.observed_accuracy;
            }
        }
        stated
    }

    /// ECE = sum(bin_weight * |accuracy - confidence|)
    pub fn compute_ece(&self) -> f64 {
        let total: u32 = self.bins.iter().map(|b| b.sample_count).sum();
        if total == 0 { return 0.0; }
        self.bins.iter().map(|bin| {
            let weight = bin.sample_count as f64 / total as f64;
            let midpoint = (bin.lower + bin.upper) / 2.0;
            weight * (bin.observed_accuracy - midpoint).abs()
        }).sum()
    }
}
```

### Calibrator integration points

- **Action gate.** Uses calibrated confidence, not raw stated confidence. The gate sees 55% when the LLM claims 80%, if the calibrator has learned that mapping.
- **Daimon.** The calibration gap feeds the emotional model. Large ECE (the golem is overconfident about its own certainty) lowers the dominance dimension.
- **TUI Oracle screen.** Per-category ECE displayed alongside accuracy. The user sees: "Fee rate predictions: stated confidence 81%, actual accuracy 62%, ECE 0.19."
- **Grimoire.** The retrospective evaluator includes ECE trends. Is the golem's self-knowledge improving over time?

### Known limitation

Isotonic regression requires the stated confidence distribution to have reasonable spread. If the LLM always outputs "90% confident," there is only one populated bin and calibration degrades to a constant correction. The system prompt instructs the LLM to use the full 0-100% range, but this is a prompt-engineering patch, not a fix.

---

## Four Modes of Intelligence

The Oracle generates predictions through four modes. All four produce the same `Prediction` struct. All four resolve against external reality. All four feed the same residual corrector.

| Mode | When it runs | Cost per unit | Speed | Daily volume | What it produces |
|------|-------------|---------------|-------|-------------|-----------------|
| **Analytical** | Waking theta cycle | $0.00-$0.03 | Per tick | ~14,000 | Calibrated expectations about tracked items |
| **Corrective** | Every resolution | $0.00 | Per resolution | ~15,000 | Bias-adjusted prediction parameters |
| **Creative** | Dream cycles | $0.01-$0.05 | Per dream cycle | ~20-50 | Novel hypotheses as testable predictions |
| **Collective** | Continuous (Styx) | ~$0.001 | Per sync | ~1,000 | Community-calibrated adjustments |

### Analytical mode

The bulk of the system. During each theta tick, `PredictionDomain` implementations generate predictions for ACTIVE items. The `ResidualCorrector` adjusts them. The `PredictionLedger` stores them. Gamma ticks resolve them against reality.

~80% of theta ticks are suppressed at the gate (T0, zero cost). The remaining ~20% escalate through the full cognitive pipeline: Grimoire retrieval, LLM deliberation, action prediction, execution, verification.

Inaction predictions happen on every suppressed tick. The golem explicitly predicts "holding is optimal" and resolves that claim by comparing portfolio value at prediction time vs. resolution time.

### Corrective mode

Zero-cost arithmetic running at gamma frequency. Bias correction and interval calibration. The corrector converges on systematic biases (the easy wins), then plateaus. Structural patterns require creative mode.

### Creative mode

During dream cycles, the golem generates novel hypotheses by replaying prediction errors, imagining counterfactual scenarios, and forming associations between semantically distant concepts. Creative outputs become testable predictions, not vague journal entries.

```
HYPNAGOGIC ONSET (seconds to minutes)
  Top N prediction residuals presented as scrambled fragments.
  Temperature elevated, executive constraints loosened.
  Dali interrupts capture half-formed associations.
  -> Each fragment registered as a Creative prediction (confidence 0.10-0.20)

NREM REPLAY (minutes)
  50 predictions with largest residuals replayed.
  LLM scans for systematic patterns across items.
  -> Pattern observations registered as Creative predictions (confidence 0.25-0.40)

REM IMAGINATION (minutes)
  Hypnagogic fragments developed into full counterfactual scenarios.
  "IF condition X, THEN outcome Y within Z hours."
  -> Counterfactuals registered as Creative predictions (confidence 0.20-0.35)

INTEGRATION (minutes)
  Surviving hypotheses consolidated into:
  - PLAYBOOK.md heuristic proposals
  - Environmental model candidates (cross-item patterns)
  - ResidualCorrector bias adjustments (applied immediately)
```

Creative accuracy of ~34% is the expected rate for genuinely novel hypotheses. The 34% that are right are disproportionately useful because they improve many predictions at once. The TUI shows this breakdown without apology.

**Known limitation: multiple comparisons.** If the dream engine generates 30 creative predictions and 34% hit, some hits are expected by chance alone. Creative predictions must be confirmed by at least 3 independent resolution events across different items before promotion to environmental model status.

### Collective mode

Three layers via Styx:

| Layer | Privacy | Shared data | Benefit |
|-------|---------|-------------|---------|
| **Vault (L0)** | Private | Nothing | Baseline solo performance |
| **Clade (L1)** | Fleet (same owner) | Residual stats, attention signals, environmental models | 7x convergence speedup at 50 members |
| **Lethe (L2)** | Public (anonymized) | Anonymized aggregates, pheromone deposits | Community calibration |

```rust
pub struct CladeResidualUpdate {
    pub golem_id: GolemId,
    pub tick: u64,
    pub residuals: Vec<ResidualStat>,
    pub environmental_models: Vec<EnvironmentalModelDigest>,
    pub attention_signals: Vec<AttentionSignal>,
}
```

The collective mode is optional. A solo golem with no Styx connection runs modes 1-3 at full capability.

### The compounding cycle

```
ANALYTICAL -> produces residuals -> feeds CORRECTIVE
CORRECTIVE -> adjusts predictions -> improves ANALYTICAL
ANALYTICAL -> largest residuals -> seeds CREATIVE (dream replay)
CREATIVE -> confirmed models -> improve ANALYTICAL across many items
ANALYTICAL -> residual stats -> shared via COLLECTIVE
COLLECTIVE -> community calibration -> improves CORRECTIVE
```

Each mode alone is useful. Together they compound.

---

## Inheritance: OracleGenome

When a golem dies, its Oracle state compresses through the genomic bottleneck. Maximum 2048 entries.

```rust
/// Compressed Oracle state that crosses the generational boundary.
/// Max 2048 entries. Inherited at 0.7x confidence (Weismann barrier).
pub struct OracleGenome {
    pub residual_stats: Vec<ResidualStat>,             // Per-(category, regime) calibration
    pub environmental_models: Vec<EnvironmentalModel>, // Cross-item patterns from creative mode
    pub attention_snapshot: AttentionSnapshot,          // What the predecessor was watching
    pub accuracy_history: Vec<AccuracyTimeSeries>,     // Calibration trajectory expectations
    pub calibration_curves: Vec<InheritedCalibrationCurve>, // Also at 0.7x
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualStat {
    pub category: CategoryId,
    pub regime: RegimeTag,
    pub mean: f64,
    pub variance: f64,
    pub coverage: f64,
    pub sample_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritedCalibrationCurve {
    pub category: CategoryId,
    pub regime: RegimeTag,
    pub bins: Vec<CalibrationBin>,
    pub parent_ece: f64,
}
```

The 0.7x inheritance coefficient is the Weismann barrier: acquired characteristics do not flow to the next generation at full strength. Lamarckian inheritance (full transfer) would make golems converge too quickly on inherited patterns, losing the ability to adapt when those patterns stop working. The 0.7x discount forces each generation to independently validate its predecessor's knowledge.

---

## TUI Integration

### Oracle screen

The MIND > Oracle screen surfaces the prediction engine's state:

```
ORACLE OVERVIEW

  Accuracy by Category (7-day rolling)
  ──────────────────────────────────────
  fee_rate        ████████████████░░░  82.4%  (1,204 resolved)
  supply_rate     ███████████████░░░░  78.1%  (891 resolved)
  price_range     ██████████████░░░░░  71.3%  (1,102 resolved)
  swap_slippage   █████████████████░░  88.2%  (47 resolved)
  inaction        ██████████████████░  92.1%  (4,208 resolved)
  lp_pnl          ██████████░░░░░░░░░  54.3%  (12 resolved)  <- learning

  Attention Universe
  ──────────────────────────────────────
  ACTIVE:   7 items (current positions + 2 high-signal)
  WATCHED:  23 items (recent anomalies)
  SCANNED:  189 items (background)
```

### Spectre visual mapping

The CorticalState's prediction signals drive Spectre visualization:

- `aggregate_accuracy` -> dot displacement clarity: higher accuracy = tighter, more coherent dot orbits
- `accuracy_trend` -> Spectre vertical posture: improving = upright, declining = slouched
- `surprise_rate` -> eye micro-flicker: high surprise = rapid pupil dilation oscillation
- `pending_predictions` -> peripheral particle speed: high foraging = fast-moving dots suggesting active search

### CorticalState TUI variables

Six prediction-related interpolating variables were added to the TUI render loop (bringing the total from 26 to 32). See [18-cortical-state.md](./18-cortical-state.md) for the full mapping.

| # | Variable | Lerp rate | Range | Source signal | What it drives |
|---|----------|-----------|-------|---------------|----------------|
| 27 | `prediction_accuracy` | 1.5 | [0.0, 1.0] | `aggregate_accuracy` | Mind screen accuracy gauge, dot orbit coherence |
| 28 | `accuracy_trend` | 1.0 | [-1.0, 1.0] | `accuracy_trend` | Trend arrow direction and color |
| 29 | `attention_breadth` | 1.2 | [0.0, 1.0] | `active_count / universe_size` | Attention focus indicator |
| 30 | `surprise_rate` | 6.0 | [0.0, 1.0] | `surprise_rate` | Background flicker intensity |
| 31 | `foraging_activity` | 1.0 | [0.0, 1.0] | `pending_predictions` | Discovery pulse in attention widget |
| 32 | `compounding_momentum` | 0.05 | [0.0, 1.0] | `compounding_momentum` | Background warmth (golden/blue shift) |

---

## Owner Configuration

```toml
[prediction]
residual_buffer_size = 256      # Entries per (category, regime). 8 bytes each.
target_coverage = 0.85          # Interval calibration target
min_correction_samples = 10     # Minimum observations before corrector activates
novelty_threshold = 2.0         # |residual| above this -> dream replay
forgetting_rate = 0.005         # Adaptive forgetting (Gibbs & Candes). Range: 0.001-0.05
compaction_window = 604800      # 7 days in seconds

[prediction.attention]
active_max = 15
watched_max = 60
scanned_max = 500
watched_eval_frequency = 4      # Theta ticks between WATCHED evaluations
scanned_eval_frequency = 100
promotion_threshold = 3.0
demotion_patience = 10          # Delta cycles without surprise before demotion

[prediction.gate]
category_threshold = 0.60
inaction_comparison = true
inaction_margin = 0.05
inheritance_coefficient = 0.70

[calibration]
enabled = true
min_samples = 30
refit_interval = 50
max_history = 2048
ece_alarm_threshold = 0.25
num_bins = 20
```

---

## Disabled Mode

When `[oracle] enabled = false`, the entire oracle crate is disabled at boot. The `Oracle` struct is not initialized. No attention universe, no ledger, no corrector, no calibrator. The config keys under `[oracle]` remain valid (no parse error), but are silently ignored.

**Heartbeat Step 3 (ANALYZE).** Falls back to observation-signal-based surprise computation. `compute_prediction_error` accepts an `Option<&Oracle>` and, when `None`, computes a signal-only error from four inputs:

- Regime change severity (0.0–1.0)
- Anomaly count (normalized by probe count)
- Position health delta
- Pheromone threat intensity

The gating logic (Step 4) continues unchanged. It just uses the degraded signal. The Golem still escalates to LLM deliberation when something notable happens — it has less calibrated prior expectations, not less responsiveness.

**Action gate.** When oracle is off, the accuracy-threshold check does not run. The gate passes on LLM deliberation alone. Step 6 VALIDATE still runs PolicyCage in full — disabling the oracle removes the prediction accuracy pre-check only, not constraint enforcement.

**Dreams.** Creative mode still runs on `DecisionCycleRecord` episodes. NREM replay has no prediction residuals to replay (residual list is empty). REM imagination runs but produces no oracle predictions — outputs land only in PLAYBOOK.md proposals. Integration phase still runs.

**Daimon.** Prediction-surprise PAD signals are not emitted. No `prediction_surprise` appraisal events fire. Other appraisal sources — position loss, regime change, owner message — still fire normally.

**TUI Oracle screen.** Shows a "ORACLE DISABLED" banner with a one-line explanation ("This Golem uses P&L fitness. Enable oracle in golem.toml to activate prediction tracking."). The 6 CorticalState variables 27–32 (`prediction_accuracy`, `accuracy_trend`, `attention_breadth`, `surprise_rate`, `foraging_activity`, `compounding_momentum`) default to 0.5 neutral when oracle is off. Visual effects driven by these variables render at their neutral baseline.

**Inheritance (OracleGenome).** When oracle is disabled, no genome is produced at death. The successor inherits an empty genome. The `inheritance_coefficient` field remains present in the config but has no effect.

---

## Registered Prediction Domains

The Oracle is domain-agnostic. Domain knowledge enters through `PredictionDomain` implementations registered at boot. The following domains ship with the Golem runtime. Additional domains can be registered by extensions.

### DeFi Domain (existing)

The original domain. Discovers pools, vaults, and lending markets from the owner's `STRATEGY.md`. Predicts fee rates, price direction, volatility regime, liquidity depth, gas prices, protocol events, and cross-chain flows. Resolution reads on-chain state via `EnvironmentClient`. Categories map to the `CategoryId` enum defined above.

### BayesianSurpriseDomain

Bayesian surprise measures how much a single observation shifts the agent's beliefs, computed as KL divergence between prior and posterior distributions (Itti & Baldi, 2009). Unlike Shannon entropy, which treats all randomness as informative, Bayesian surprise responds only to data that forces a genuine model update. A fair coin flip has zero surprise because the outcome doesn't change the agent's belief that the coin is fair. A liquidation event on a pool that has never been liquidated has high surprise because it shifts the posterior sharply.

The domain registers with the Oracle alongside the DeFi domain. It does not replace the Oracle's prediction error signal. It feeds INTO the Oracle as an additional prediction domain, producing predictions about future surprise levels that resolve against observed KL divergence. The Oracle's `ResidualCorrector` handles surprise predictions identically to price predictions.

Conflict resolution reference: [04-conflict-resolution.md](../../tmp/research/witness-research/new/reconciling/04-conflict-resolution.md), Conflict 1.

#### Conjugate prior models

Four conjugate families cover the feature types in the observation stream. Conjugate priors allow closed-form posterior updates: the posterior stays in the same distribution family as the prior, so updating is parameter arithmetic, O(1) per observation.

**Beta-Binomial** for binary features (did a liquidation occur? is this a new address?):

```
Prior:     Beta(alpha, beta)
Observe:   k successes in n trials
Posterior: Beta(alpha + k, beta + n - k)

KL divergence (closed form):
D_KL(Beta(a', b') || Beta(a, b)) =
    ln(B(a, b) / B(a', b'))
    + (a' - a) * psi(a')
    + (b' - b) * psi(b')
    - (a' + b' - a - b) * psi(a' + b')
```

**Normal-Gamma** for continuous features (gas prices, swap values, price impact):

```
Prior:     NormalGamma(mu_0, kappa_0, alpha_0, beta_0)
Observe:   x
Update:    kappa_1 = kappa_0 + 1
           mu_1    = (kappa_0 * mu_0 + x) / kappa_1
           alpha_1 = alpha_0 + 0.5
           beta_1  = beta_0 + (kappa_0 * (x - mu_0)^2) / (2 * kappa_1)
```

**Dirichlet-Multinomial** for categorical features (protocol family, transaction type, MEV pattern):

```
Prior:     Dirichlet(alpha_1, ..., alpha_K)
Observe:   category j
Posterior: Dirichlet(alpha_1, ..., alpha_j + 1, ..., alpha_K)
```

**Gamma-Poisson** for event rates (transactions per block from an address, events per Theta window):

```
Prior:     Gamma(alpha, beta) conjugate to Poisson(lambda)
Observe:   k events in one interval
Posterior: Gamma(alpha + k, beta + 1)
```

Each model supports exponential decay on sufficient statistics for non-stationarity. Decay factors align with the tick hierarchy: 0.995 at Gamma (fast features), 0.99 at Theta (medium features), with a full parameter review at Delta.

#### Rust types

```rust
/// Trait for conjugate Bayesian models with O(1) surprise computation.
pub trait ConjugatePrior: Clone + Send + Sync {
    type Observation;

    /// Update with observation, return KL divergence (surprise).
    fn observe(&mut self, obs: &Self::Observation) -> f64;

    /// Apply exponential decay to sufficient statistics.
    fn decay(&mut self, factor: f64);

    /// Reset to initial prior.
    fn reset(&mut self);

    /// Number of effective observations (diagnostics).
    fn effective_count(&self) -> f64;
}

#[derive(Clone, Debug)]
pub struct BetaBinomial {
    alpha: f64,
    beta: f64,
    alpha_0: f64,
    beta_0: f64,
}

#[derive(Clone, Debug)]
pub struct NormalGamma {
    mu: f64,
    kappa: f64,
    alpha: f64,
    beta: f64,
    kappa_0: f64,
    alpha_0: f64,
    beta_0: f64,
}

#[derive(Clone, Debug)]
pub struct DirichletMultinomial {
    alphas: Vec<f64>,
    alphas_0: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct GammaPoisson {
    alpha: f64,
    beta: f64,
    alpha_0: f64,
    beta_0: f64,
}
```

#### Full ConjugatePrior Implementations

> Source: `06-curiosity-learning/00-bayesian-surprise.md`

**BetaBinomial implementation:**

```rust
use statrs::function::gamma::{digamma, ln_gamma};

impl BetaBinomial {
    pub fn new(alpha: f64, beta: f64) -> Self {
        Self {
            alpha,
            beta,
            alpha_0: alpha,
            beta_0: beta,
        }
    }

    fn kl_divergence(a1: f64, b1: f64, a0: f64, b0: f64) -> f64 {
        let ln_beta = |a: f64, b: f64| ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
        ln_beta(a0, b0) - ln_beta(a1, b1)
            + (a1 - a0) * digamma(a1)
            + (b1 - b0) * digamma(b1)
            - (a1 + b1 - a0 - b0) * digamma(a1 + b1)
    }
}

impl ConjugatePrior for BetaBinomial {
    type Observation = bool;

    fn observe(&mut self, obs: &bool) -> f64 {
        let (a_prior, b_prior) = (self.alpha, self.beta);
        if *obs {
            self.alpha += 1.0;
        } else {
            self.beta += 1.0;
        }
        Self::kl_divergence(self.alpha, self.beta, a_prior, b_prior)
    }

    fn decay(&mut self, factor: f64) {
        self.alpha = 1.0 + (self.alpha - 1.0) * factor;
        self.beta = 1.0 + (self.beta - 1.0) * factor;
    }

    fn reset(&mut self) {
        self.alpha = self.alpha_0;
        self.beta = self.beta_0;
    }

    fn effective_count(&self) -> f64 {
        (self.alpha - self.alpha_0) + (self.beta - self.beta_0)
    }
}
```

**NormalGamma implementation:**

```rust
impl NormalGamma {
    pub fn new(mu: f64, kappa: f64, alpha: f64, beta: f64) -> Self {
        Self {
            mu,
            kappa,
            alpha,
            beta,
            kappa_0: kappa,
            alpha_0: alpha,
            beta_0: beta,
        }
    }

    fn kl_divergence(&self, mu_0: f64, kappa_0: f64, alpha_0: f64, beta_0: f64) -> f64 {
        let (mu1, k1, a1, b1) = (self.mu, self.kappa, self.alpha, self.beta);
        let (k0, a0, b0) = (kappa_0, alpha_0, beta_0);

        let term1 = 0.5 * (k0 / k1).ln()
            + 0.5 * (k1 / k0 - 1.0)
            + 0.5 * k0 * (mu1 - mu_0).powi(2) * a1 / b1;
        let term2 = a0 * (b1 / b0).ln() - ln_gamma(a1) + ln_gamma(a0)
            + (a1 - a0) * digamma(a1)
            - (b1 - b0) * a1 / b1;

        (term1 + term2).max(0.0) // Clamp numerical noise
    }
}

impl ConjugatePrior for NormalGamma {
    type Observation = f64;

    fn observe(&mut self, obs: &f64) -> f64 {
        let (mu_prior, kappa_prior, alpha_prior, beta_prior) =
            (self.mu, self.kappa, self.alpha, self.beta);

        let x = *obs;
        let kappa_new = self.kappa + 1.0;
        let mu_new = (self.kappa * self.mu + x) / kappa_new;
        let alpha_new = self.alpha + 0.5;
        let beta_new = self.beta + (self.kappa * (x - self.mu).powi(2)) / (2.0 * kappa_new);

        self.mu = mu_new;
        self.kappa = kappa_new;
        self.alpha = alpha_new;
        self.beta = beta_new;

        self.kl_divergence(mu_prior, kappa_prior, alpha_prior, beta_prior)
    }

    fn decay(&mut self, factor: f64) {
        self.kappa = self.kappa_0.max(self.kappa * factor);
        self.alpha = self.alpha_0.max(self.alpha_0 + (self.alpha - self.alpha_0) * factor);
        self.beta = self.beta_0.max(self.beta_0 + (self.beta - self.beta_0) * factor);
    }

    fn reset(&mut self) {
        self.kappa = self.kappa_0;
        self.alpha = self.alpha_0;
        self.beta = self.beta_0;
    }

    fn effective_count(&self) -> f64 {
        self.kappa - self.kappa_0
    }
}
```

**DirichletMultinomial implementation:**

```rust
impl DirichletMultinomial {
    pub fn new(alphas: Vec<f64>) -> Self {
        Self {
            alphas_0: alphas.clone(),
            alphas,
        }
    }

    pub fn num_categories(&self) -> usize {
        self.alphas.len()
    }

    fn kl_divergence(post: &[f64], prior: &[f64]) -> f64 {
        let sum_post: f64 = post.iter().sum();
        let sum_prior: f64 = prior.iter().sum();

        let mut kl = ln_gamma(sum_post) - ln_gamma(sum_prior);
        for (a_post, a_prior) in post.iter().zip(prior.iter()) {
            kl += ln_gamma(*a_prior) - ln_gamma(*a_post);
            kl += (*a_post - *a_prior) * (digamma(*a_post) - digamma(sum_post));
        }
        kl.max(0.0)
    }
}

impl ConjugatePrior for DirichletMultinomial {
    /// Observation is the index of the observed category.
    type Observation = usize;

    fn observe(&mut self, obs: &usize) -> f64 {
        assert!(*obs < self.alphas.len(), "category index out of bounds");
        let prior = self.alphas.clone();
        self.alphas[*obs] += 1.0;
        Self::kl_divergence(&self.alphas, &prior)
    }

    fn decay(&mut self, factor: f64) {
        for (a, a0) in self.alphas.iter_mut().zip(self.alphas_0.iter()) {
            *a = *a0 + (*a - *a0) * factor;
        }
    }

    fn reset(&mut self) {
        self.alphas = self.alphas_0.clone();
    }

    fn effective_count(&self) -> f64 {
        self.alphas
            .iter()
            .zip(self.alphas_0.iter())
            .map(|(a, a0)| a - a0)
            .sum()
    }
}
```

Each implements `ConjugatePrior` with the update rules and KL divergence formulas above. The digamma function evaluations dominate cost at ~50ns per call. A typical transaction touches 3-5 models, so total surprise computation runs under 1 microsecond.

#### The Composite Surprise Scorer

> Source: `06-curiosity-learning/00-bayesian-surprise.md`

The `BayesianSurpriseScorer` wraps the per-protocol conjugate models and provides a single `score()` entry point for the triage pipeline. It uses `DashMap` for concurrent per-protocol access.

```rust
use dashmap::DashMap;
use std::sync::Arc;

/// Per-protocol Bayesian surprise models.
/// Each protocol gets its own set of conjugate models
/// tracking different feature families.
pub struct BayesianSurpriseScorer {
    /// Binary features per protocol (e.g., "liquidation occurred")
    binary_models: DashMap<ProtocolId, Vec<(String, BetaBinomial)>>,
    /// Continuous features per protocol (e.g., gas price, swap value)
    continuous_models: DashMap<ProtocolId, Vec<(String, NormalGamma)>>,
    /// Categorical features per protocol (e.g., transaction type)
    categorical_models: DashMap<ProtocolId, Vec<(String, DirichletMultinomial)>>,
    /// Decay factors per tick tier
    gamma_decay: f64,
    theta_decay: f64,
}

impl BayesianSurpriseScorer {
    pub fn new(gamma_decay: f64, theta_decay: f64) -> Self {
        Self {
            binary_models: DashMap::new(),
            continuous_models: DashMap::new(),
            categorical_models: DashMap::new(),
            gamma_decay,
            theta_decay,
        }
    }

    /// Score a transaction's surprise across all feature models
    /// for its protocol. Returns the maximum surprise across features
    /// (not the sum -- we want the single most surprising dimension).
    pub fn score(&self, protocol: &ProtocolId, features: &TxFeatures) -> f64 {
        let mut max_surprise = 0.0_f64;

        if let Some(mut binary) = self.binary_models.get_mut(protocol) {
            for (name, model) in binary.iter_mut() {
                if let Some(obs) = features.binary.get(name.as_str()) {
                    let s = model.observe(obs);
                    max_surprise = max_surprise.max(s);
                }
            }
        }

        if let Some(mut continuous) = self.continuous_models.get_mut(protocol) {
            for (name, model) in continuous.iter_mut() {
                if let Some(obs) = features.continuous.get(name.as_str()) {
                    let s = model.observe(obs);
                    max_surprise = max_surprise.max(s);
                }
            }
        }

        if let Some(mut categorical) = self.categorical_models.get_mut(protocol) {
            for (name, model) in categorical.iter_mut() {
                if let Some(obs) = features.categorical.get(name.as_str()) {
                    let s = model.observe(obs);
                    max_surprise = max_surprise.max(s);
                }
            }
        }

        max_surprise
    }

    /// Apply decay at Gamma tick boundary.
    pub fn decay_gamma(&self) {
        for mut entry in self.binary_models.iter_mut() {
            for (_, model) in entry.value_mut().iter_mut() {
                model.decay(self.gamma_decay);
            }
        }
        for mut entry in self.continuous_models.iter_mut() {
            for (_, model) in entry.value_mut().iter_mut() {
                model.decay(self.gamma_decay);
            }
        }
    }

    /// Apply decay at Theta tick boundary.
    pub fn decay_theta(&self) {
        for mut entry in self.categorical_models.iter_mut() {
            for (_, model) in entry.value_mut().iter_mut() {
                model.decay(self.theta_decay);
            }
        }
    }

    /// Register models for a new protocol with default priors.
    pub fn register_protocol(&self, protocol: ProtocolId, config: &ProtocolModelConfig) {
        self.binary_models.insert(
            protocol.clone(),
            config
                .binary_features
                .iter()
                .map(|name| (name.clone(), BetaBinomial::new(1.0, 10.0)))
                .collect(),
        );
        self.continuous_models.insert(
            protocol.clone(),
            config
                .continuous_features
                .iter()
                .map(|name| {
                    // Weakly informative prior: mean 0, low confidence
                    (name.clone(), NormalGamma::new(0.0, 0.1, 1.0, 1.0))
                })
                .collect(),
        );
        self.categorical_models.insert(
            protocol.clone(),
            config
                .categorical_features
                .iter()
                .map(|(name, k)| {
                    // Symmetric Dirichlet with alpha=1 (uniform prior)
                    (name.clone(), DirichletMultinomial::new(vec![1.0; *k]))
                })
                .collect(),
        );
    }
}
```

#### The BayesianSurpriseDomain implementation

```rust
/// Bayesian surprise as a PredictionDomain.
/// Per-protocol conjugate models at three timescales.
/// Predictions: "surprise in the next Theta window will be X nats."
/// Resolution: actual KL divergence observed.
pub struct BayesianSurpriseDomain {
    /// Fast models (sub-hour structural changes). Decay 0.995/gamma.
    fast_models: DashMap<ProtocolId, BayesianSurpriseScorer>,
    /// Medium models (intra-day regime shifts). Decay 0.998/gamma.
    medium_models: DashMap<ProtocolId, BayesianSurpriseScorer>,
    /// Slow models (multi-day trend changes). Decay 0.9995/gamma.
    slow_models: DashMap<ProtocolId, BayesianSurpriseScorer>,
}

/// Per-protocol scorer holding binary, continuous, categorical,
/// and rate models for a single timescale.
pub struct BayesianSurpriseScorer {
    binary_models: Vec<(String, BetaBinomial)>,
    continuous_models: Vec<(String, NormalGamma)>,
    categorical_models: Vec<(String, DirichletMultinomial)>,
    rate_models: Vec<(String, GammaPoisson)>,
    decay_factor: f64,
}

#[async_trait]
impl PredictionDomain for BayesianSurpriseDomain {
    fn domain_id(&self) -> &str { "bayesian_surprise" }
    fn display_name(&self) -> &str { "Bayesian Surprise" }

    fn categories(&self) -> Vec<PredictionCategory> {
        vec![
            PredictionCategory::new("fast_surprise", "Sub-hour structural changes"),
            PredictionCategory::new("medium_surprise", "Intra-day regime shifts"),
            PredictionCategory::new("slow_surprise", "Multi-day trend changes"),
        ]
    }

    async fn discover(
        &self,
        seed: &AttentionSeed,
        env: &dyn EnvironmentClient,
    ) -> Result<Vec<TrackedItem>> {
        // Discovers protocol IDs from the seed. Each protocol becomes
        // a tracked item with three timescale models initialized
        // from weakly informative priors.
        todo!()
    }

    fn predict_scanned(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
    ) -> Vec<PredictionDraft> {
        // Single prediction: aggregate surprise across all models
        // for the slow timescale. Cheap: reads existing model state.
        todo!()
    }

    fn predict_watched(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
    ) -> Vec<PredictionDraft> {
        // Two predictions: fast and medium timescale surprise.
        // Uses cortical regime to condition the prediction.
        todo!()
    }

    fn predict_active(
        &self,
        item: &TrackedItem,
        history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
        playbook: &PlaybookState,
    ) -> Vec<PredictionDraft> {
        // Three predictions: fast, medium, slow. Full context.
        todo!()
    }

    async fn resolve(
        &self,
        prediction: &Prediction,
        checkpoint: &Checkpoint,
        env: &dyn EnvironmentClient,
    ) -> Result<ResolutionOutcome> {
        // Actual KL divergence observed in the checkpoint window.
        // Compare against predicted surprise level.
        todo!()
    }

    fn predict_action(&self, _: &ProposedAction, _: &TrackedItem,
        _: &ResidualBuffer) -> Vec<PredictionDraft> { vec![] }
    fn predict_inaction(&self, _: &TrackedItem, _: &ResidualBuffer,
        _: &CorticalSnapshot) -> Vec<PredictionDraft> { vec![] }
}
```

#### Surprise as triage routing signal

Raw surprise feeds into triage routing alongside existing heuristics:

| Surprise range | Interpretation | Action |
|---|---|---|
| > 2.0 nats | Extreme model violation | Immediate TriageAlert regardless of other scores |
| 0.5 - 2.0 nats | Moderate belief shift | Boost curiosity score by 0.3 |
| 0.1 - 0.5 nats | Minor update | No modification |
| < 0.1 nats | Expected observation | Slight curiosity penalty (-0.1) |

The extreme surprise threshold (> 2.0 nats) acts as an override. If a single observation forces a large belief update, the event gets escalated even if heuristics and similarity would have filtered it. This catches novel attack vectors that heuristic rules don't anticipate and that have no similar episodes in the ANN index.

#### Cold-start behavior

When the golem has no prior observations for a protocol, conjugate models start at uninformative priors. Every observation generates moderate surprise because models update rapidly from weak priors. This is correct: the agent should be curious about protocols it hasn't modeled yet. As observations accumulate, models become confident and surprise drops for routine events. During cold start (fewer than ~50 observations per protocol), the Hedge weighting system (see [02-heartbeat.md](./02-heartbeat.md)) naturally downweights the surprise signal because it carries high variance.

#### Memory per protocol

~200 bytes for a standard feature set (2 binary, 2 continuous, 1 categorical with 10 categories, 1 rate model). For 100 tracked protocols across 3 timescales, the total footprint is ~60KB. Negligible against the ANN index or Grimoire.

### TopologyDomain

Persistent homology tracks the birth and death of geometric features in observation point clouds as a scale parameter sweeps from zero to infinity (Carlsson, 2009). Applied to the Golem's observation stream, TDA-based regime detection identifies structural precursors to regime shifts 10-100 perception ticks before statistical methods, because the topology of observation space changes before its moments do. A cluster fracture (beta_0 increasing) happens before variance spikes. A cyclic pattern collapse (beta_1 dropping) happens before a trend reversal is visible in moving averages.

The TopologyDomain registers with the Oracle to make topological predictions that resolve against observed structural changes. The Oracle tracks topology prediction accuracy alongside fee-rate and price-direction accuracy in the same ledger.

#### Mathematical foundation

At each gamma tick, the Golem's perception layer produces an observation vector x_t in R^d. A sliding window of N = 200 recent observations forms a point cloud X_N. The Vietoris-Rips complex VR(X, epsilon) connects points within distance epsilon, and persistent homology tracks features across the filtration as epsilon increases:

- **H_0 (connected components):** Clusters in observation space. Many persistent components = market fragmentation.
- **H_1 (loops):** Cycles in observation space. Mean-reverting markets create loops; trending markets don't.
- **H_2 (voids):** Empty regions where observations used to occur. Correlation breakdowns create voids.

Each feature gets a (birth, death) pair. Features that persist over a large epsilon range are signal; short-lived features are noise. The Stability Theorem (Cohen-Steiner, Edelsbrunner & Harer, 2007) guarantees that small input perturbations produce small output perturbations.

The **Wasserstein distance** between two persistence diagrams quantifies structural difference:

```
W_p(D_1, D_2) = (inf_gamma sum |x - gamma(x)|^p)^{1/p}
```

The Golem maintains a reference diagram D_ref for the current regime. At each gamma tick, W_p(D_now, D_ref) measures topological distance from the reference state.

#### Topological regime signatures

| Regime | W_p | beta_0 | beta_1 | beta_2 | Signature |
|--------|-----|--------|--------|--------|-----------|
| Calm | Low | ~1 | ~0 | 0 | Single tight cluster, no loops or voids |
| Trending | Low | ~1 | ~0 | 0 | Cluster drifting; diagram shift rate discriminates from calm |
| Volatile | Moderate | >1 | >0 | 0 | Fragmented clusters, cyclic oscillations |
| Crisis | High | >>1 | varies | >0 | Extreme fragmentation, voids appearing |

#### The TDA pipeline

```rust
/// TDA analyzer. Runs at Step 3 (ANALYZE), gamma frequency.
pub struct TdaAnalyzer {
    config: TdaConfig,
    reference_diagram: Option<PersistenceDiagram>,
    tick_count: u64,
}

pub struct TdaConfig {
    pub window_size: usize,       // 200
    pub pca_dimensions: usize,    // 5 (Ripser is exponential in dimension)
    pub max_homology_dim: usize,  // 2 (H_0, H_1, H_2)
    pub reference_tau: f32,       // 50.0 (EMA time constant, ~4-12 min)
    pub wasserstein_p: f32,       // 2.0
}

/// Persistence diagram: multiset of (birth, death) pairs per dimension.
#[derive(Clone, Debug)]
pub struct PersistenceDiagram {
    pub features: Vec<PersistenceFeature>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceFeature {
    pub birth: f32,
    pub death: f32,
    pub dimension: u8,
}

impl PersistenceDiagram {
    /// Betti number at scale epsilon for dimension k.
    pub fn betti(&self, dim: u8, epsilon: f32) -> usize {
        self.features.iter()
            .filter(|f| f.dimension == dim && f.birth <= epsilon && f.death > epsilon)
            .count()
    }

    /// Scale that maximizes total persistence of alive features.
    pub fn optimal_scale(&self) -> f32 { /* argmax sum(persistence) */ todo!() }
}

/// Wasserstein distance via Hungarian algorithm.
/// O(n^3) on diagram size. With ~50 features: <1ms.
pub fn wasserstein_distance(
    d1: &PersistenceDiagram,
    d2: &PersistenceDiagram,
    dim: u8,
    p: f32,
) -> f32 { todo!() }
```

#### TopologyDomain implementation

```rust
/// Persistent homology as a PredictionDomain.
/// Predictions: "Wasserstein distance will be X at next Theta checkpoint"
///              "beta_0 will remain <= N"
///              "beta_1 will increase (loop formation)"
/// Resolution: actual persistence diagram at checkpoint time.
pub struct TopologyDomain {
    analyzer: TdaAnalyzer,
}

#[async_trait]
impl PredictionDomain for TopologyDomain {
    fn domain_id(&self) -> &str { "topology" }
    fn display_name(&self) -> &str { "Topological Market Intelligence" }

    fn categories(&self) -> Vec<PredictionCategory> {
        vec![
            PredictionCategory::new("betti_0", "Connected components at optimal scale"),
            PredictionCategory::new("betti_1", "Loops at optimal scale"),
            PredictionCategory::new("wasserstein", "Topological distance from reference"),
        ]
    }

    async fn discover(
        &self,
        seed: &AttentionSeed,
        env: &dyn EnvironmentClient,
    ) -> Result<Vec<TrackedItem>> {
        // One tracked item per observation-space window.
        // Typically one per chain the Golem monitors.
        todo!()
    }

    fn predict_scanned(
        &self, item: &TrackedItem, history: &ResidualBuffer,
    ) -> Vec<PredictionDraft> {
        // Single prediction: Wasserstein distance will stay below threshold.
        todo!()
    }

    fn predict_watched(
        &self, item: &TrackedItem, history: &ResidualBuffer,
        cortical: &CorticalSnapshot,
    ) -> Vec<PredictionDraft> {
        // Two predictions: Wasserstein distance + beta_0 stability.
        todo!()
    }

    fn predict_active(
        &self, item: &TrackedItem, history: &ResidualBuffer,
        cortical: &CorticalSnapshot, playbook: &PlaybookState,
    ) -> Vec<PredictionDraft> {
        // Three predictions: Wasserstein, beta_0, beta_1.
        todo!()
    }

    async fn resolve(
        &self, prediction: &Prediction, checkpoint: &Checkpoint,
        env: &dyn EnvironmentClient,
    ) -> Result<ResolutionOutcome> {
        // Compute persistence diagram at checkpoint, compare to prediction.
        todo!()
    }

    fn predict_action(&self, _: &ProposedAction, _: &TrackedItem,
        _: &ResidualBuffer) -> Vec<PredictionDraft> { vec![] }
    fn predict_inaction(&self, _: &TrackedItem, _: &ResidualBuffer,
        _: &CorticalSnapshot) -> Vec<PredictionDraft> { vec![] }
}
```

#### Computational budget

Ripser on 200 points in R^5 with max_dim=2: under 50ms (Bauer, 2021). PCA projection: 1-2ms. Wasserstein distance on ~50 features: <1ms. Total: under 55ms per gamma tick, well within the 5-second minimum gamma interval.

### TA Prediction Domains

Eight additional prediction domains arise from the technical analysis research. Each maps a TA subsystem's output to a falsifiable claim that the Oracle tracks. These are documented in [17b-ta-prediction-domains.md](./17b-ta-prediction-domains.md).

| Category | Source | Claim type | Resolution |
|----------|--------|-----------|------------|
| `ManifoldCurvatureSign` | SpectralManifold | `DirectionalClaim` | Did curvature correctly predict stability/instability? |
| `TopologicalTransition` | PredictiveGeometry | `ThresholdClaim` | Did beta_0 or beta_1 change within the predicted horizon? |
| `CausalEdgeStrength` | CausalDiscoveryEngine | `RangeClaim` | Did intervention effect size fall within predicted range? |
| `PatternMatchOutcome` | TaPatternCodebook | `DirectionalClaim` | Did the matched pattern's historical outcome repeat? |
| `EntanglementRegime` | EntanglementTracker | `ThresholdClaim` | Did cross-protocol entanglement exceed predicted threshold? |
| `AdversarialDetection` | AdversarialDefense | `DirectionalClaim` | Was a flagged observation confirmed as manipulation? |
| `SignalSpeciation` | SignalMetabolism | `OccurrenceClaim` | Did a signal speciate within the predicted window? |
| `SomaticAccuracy` | SomaticTaEngine | `DirectionalClaim` | Did a somatic marker's behavioral bias produce a better outcome? |

All eight register through `PredictionDomain::categories()`. Predictions flow through the same residual correction pipeline as existing DeFi predictions. The Oracle does not distinguish between a prediction about ETH price direction and a prediction about manifold curvature sign. It tracks accuracy, computes residuals, and gates action identically.

#### Integration with category_accuracies

CorticalState holds `category_accuracies: [AtomicU32; 16]`. The 8 TA categories are bucketed into `category_accuracies[15]` (the last position). The TaCorticalExtension tracks per-category accuracy internally and writes the aggregate to the CorticalState slot. This avoids expanding CorticalState's cache-line footprint for domain-specific signals.

### Domain registration at boot

```rust
impl Oracle {
    pub fn register_domains(&mut self) {
        // Core domain (always present)
        self.register(Box::new(DeFiDomain::new(self.env.clone())));

        // Bayesian surprise (opt-in via golem.toml)
        if self.config.bayesian_surprise.enabled {
            self.register(Box::new(BayesianSurpriseDomain::new(
                self.config.bayesian_surprise.clone(),
            )));
        }

        // Topology (opt-in, requires sufficient observation history)
        if self.config.topology.enabled {
            self.register(Box::new(TopologyDomain::new(
                TdaConfig::from(&self.config.topology),
            )));
        }

        // TA domains (opt-in, requires TA subsystem)
        if self.config.ta.enabled {
            self.register(Box::new(TaPredictionDomains::new(
                self.ta_extension.clone(),
            )));
        }
    }
}
```

### Extended CategoryId enum

```rust
pub enum CategoryId {
    // === Existing DeFi categories ===
    PriceDirection,
    PriceRange,
    VolatilityRegime,
    LiquidityDepth,
    FeeGeneration,
    GasPrice,
    ProtocolEvent,
    CrossChain,

    // === Bayesian surprise categories ===
    FastSurprise,       // Sub-hour structural changes
    MediumSurprise,     // Intra-day regime shifts
    SlowSurprise,       // Multi-day trend changes

    // === Topology categories ===
    Betti0,             // Connected components
    Betti1,             // Loops
    WassersteinDist,    // Topological distance from reference

    // === TA categories (see 17b-ta-prediction-domains.md) ===
    ManifoldCurvatureSign,
    TopologicalTransition,
    CausalEdgeStrength,
    PatternMatchOutcome,
    EntanglementRegime,
    AdversarialDetection,
    SignalSpeciation,
    SomaticAccuracy,
}
```

---

## References

- [VOVK-2005] Vovk, V., Gammerman, A., & Shafer, G. *Algorithmic Learning in a Random World*. Springer, 2005. — *Foundational text on conformal prediction providing distribution-free coverage guarantees; the theoretical basis for the Oracle's prediction interval calibration.*
- [ANGELOPOULOS-2023] Angelopoulos, A.N. & Bates, S. "Conformal Prediction: A Gentle Introduction." *Foundations and Trends in Machine Learning* 16(4), 2023. — *Accessible introduction to conformal prediction with modern applications; the practical guide for implementing the Oracle's coverage guarantees.*
- [GIBBS-2021] Gibbs, I. & Candes, E. "Adaptive Conformal Inference Under Distribution Shift." *NeurIPS*, 2021. — *Extends conformal prediction to non-stationary data by adapting the miscoverage rate online; directly addresses the Oracle's need to maintain calibration across market regime changes.*
- [STEPHENS-1986] Stephens, D.W. & Krebs, J.R. *Foraging Theory*. Princeton University Press, 1986. — *Formalizes optimal foraging as a resource allocation problem under uncertainty; the biological model for how the Oracle allocates computational budget across prediction domains.*
- [CHARNOV-1976] Charnov, E.L. "Optimal Foraging, the Marginal Value Theorem." *Theoretical Population Biology* 9(2), 1976. — *Proves that an optimal forager should leave a patch when marginal return drops to the average; governs when the Oracle stops refining a prediction and moves to the next domain.*
- [PIROLLI-1999] Pirolli, P. & Card, S.K. "Information Foraging." *Psychological Review* 106(4), 1999. — *Adapts foraging theory to information seeking, introducing information scent; the cognitive model for how the Oracle decides which data sources to consult.*
- [GUO-2017] Guo, C. et al. "On Calibration of Modern Neural Networks." *ICML*, 2017. — *Shows that modern neural networks are poorly calibrated and proposes temperature scaling; motivates the Oracle's explicit calibration layer on top of LLM confidence scores.*
- [XIONG-2023] Xiong, M. et al. "Can LLMs Express Their Uncertainty?" arXiv:2306.13063, 2023. — *Evaluates whether LLMs can verbalize calibrated uncertainty estimates; informs the Oracle's approach to extracting and correcting LLM confidence signals.*
- [GENG-2024] Geng, J. et al. "A Survey of Confidence Estimation and Calibration in Large Language Models." *NAACL*, 2024. — *Surveys methods for LLM confidence estimation including verbalized and logit-based approaches; the reference for the Oracle's multi-method confidence extraction.*
- [MINDERER-2021] Minderer, M. et al. "Revisiting the Calibration of Modern Neural Networks." *NeurIPS*, 2021. — *Shows calibration varies by architecture and training procedure; motivates the Oracle's per-model calibration rather than assuming uniform reliability.*
- [STOCKBENCH-2025] "StockBench: Can LLM Agents Trade Stocks Profitably?" 2025. — *Benchmarks LLM agents on real trading tasks; provides empirical baselines for what prediction accuracy to expect from the Oracle's LLM-based forecasts.*
- [CLARK-2013] Clark, A. "Whatever next?" *BBS*, 36(3), 2013. — *Proposes the predictive processing framework where brains are fundamentally prediction machines; the cognitive science paradigm that the Oracle's residual-correction architecture implements.*
- [FRISTON-2010] Friston, K. "The free-energy principle." *Nature Reviews Neuroscience*, 11(2), 2010. — *Proposes that all adaptive systems minimize variational free energy (prediction error); the theoretical umbrella connecting the Oracle's predictions to the Golem's overall behavior.*
- [ITTI-2009] Itti, L. & Baldi, P. "Bayesian Surprise Attracts Human Attention." *Vision Research*, 49(10), 2009. — *Shows that attention is drawn to observations that maximally shift beliefs; the Oracle uses Bayesian surprise to prioritize which prediction domains need updating.*
- [BALDI-2010] Baldi, P. & Itti, L. "Of Bits and Wows: A Bayesian Theory of Surprise." *Neural Networks*, 23(5), 2010. — *Axiomatizes Bayesian surprise as the unique measure depending on model class, zero when beliefs don't change, and additive for independent components; the formal definition the Oracle implements.*
- [CARLSSON-2009] Carlsson, G. "Topology and Data." *Bulletin of the AMS*, 46(2), 2009. — *Introduces topological data analysis (TDA) and persistent homology for shape detection in data; the mathematical foundation for the Oracle's regime-change detection via persistence diagrams.*
- [GIDEA-2018] Gidea, M. & Katz, Y. "Topological Data Analysis of Financial Time Series." *Physica A*, 491, 2018. — *Applies TDA to financial time series and detects market crashes via topological features; directly validates the Oracle's use of persistence diagrams for DeFi regime detection.*
- [COHEN-STEINER-2007] Cohen-Steiner, D., Edelsbrunner, H. & Harer, J. "Stability of Persistence Diagrams." *Discrete & Computational Geometry*, 37(1), 2007. — *Proves that small perturbations in input data produce small changes in persistence diagrams; the stability guarantee that makes the Oracle's topological signals robust to noise.*
- [BAUER-2021] Bauer, U. "Ripser: Efficient Computation of Vietoris-Rips Persistence Barcodes." *JACT*, 5, 2021. — *Describes the fastest known algorithm for computing Vietoris-Rips persistence; the implementation the Oracle uses for real-time TDA on streaming market data.*
- [SCHMIDHUBER-2010] Schmidhuber, J. "Formal Theory of Creativity, Fun, and Intrinsic Motivation." *IEEE Transactions on Autonomous Mental Development*, 2(3), 2010. — *Formalizes curiosity as the drive to maximize learning progress (compression improvement); connects the Oracle's surprise signals to the Golem's intrinsic motivation system.*
- [SHALEV-SHWARTZ-2011] Shalev-Shwartz, S. "Online Learning and Online Convex Optimization." *Foundations and Trends in Machine Learning*, 4(2), 2011. — *Comprehensive treatment of online learning with regret bounds; the theoretical framework for the Oracle's adaptive triage parameter tuning.*

---

## Bayesian Surprise: Extended Foundation

> Source: `06-curiosity-learning/00-bayesian-surprise.md`

### Why Not Shannon Information?

Shannon's self-information `-log P(D)` assigns high scores to rare events regardless of whether they're informative. A transaction with an unusual gas price gets high Shannon information even if the agent already knows gas is volatile. Bayesian surprise, by contrast, stays low when the agent's model already accounts for that volatility. The practical consequence: Shannon-based curiosity generates false positives on noisy features; Bayesian surprise is calibrated to the agent's current knowledge state.

Baldi & Itti (2010) proved that Bayesian surprise is the only measure satisfying three axioms:

1. It depends on the model class, not just the observation.
2. It is zero when beliefs don't change.
3. It is additive for independent model components.

Shannon information satisfies only the third.

### Gamma-Poisson Conjugate Model with KL Formula

Event rates: transactions per block from a given address, events per Theta window for a protocol.

```
Prior:     Gamma(alpha, beta)  -- conjugate to Poisson(lambda)
Observe:   k events in one time interval
Posterior: Gamma(alpha + k, beta + 1)

KL divergence:
D_KL(Gamma(a', b') || Gamma(a, b)) =
    (a' - a) * psi(a') - ln(Gamma(a')) + ln(Gamma(a))
    + a * (ln(b') - ln(b)) + a' * (b / b' - 1)
```

An address that normally produces 2 transactions per block suddenly producing 50 generates high surprise. The Gamma-Poisson model captures this naturally because the posterior rate estimate shifts sharply.

### Streaming Decay Details

Exponential decay on sufficient statistics prevents the model from becoming increasingly rigid and resistant to surprise from genuine regime changes.

**Decay factor:** 0.995 per Gamma tick (roughly 12 seconds). This gives a **half-life of about 138 ticks, or ~28 minutes**. The model forgets half its conviction every half hour, staying responsive to regime changes while still accumulating enough evidence to distinguish signal from noise.

**Three-tier tick alignment:**

- **Gamma tick**: Decay fast-moving features (gas prices, event rates). Decay factor 0.995.
- **Theta tick**: Decay medium-cadence features (protocol category distributions). Decay factor 0.99.
- **Delta tick**: Full parameter review. If any model has accumulated sufficient statistics beyond a threshold (indicating the agent is over-confident), apply aggressive decay or reset to a moderately informative prior.

For Beta(alpha, beta):
```
alpha_decayed = 1 + (alpha - 1) * decay_factor
beta_decayed  = 1 + (beta - 1) * decay_factor
```

The `1 +` offset prevents the parameters from collapsing below the uninformative prior.

For Normal-Gamma(mu, kappa, alpha, beta):
```
kappa_decayed = max(kappa_0, kappa * decay_factor)
alpha_decayed = max(alpha_0, alpha_0 + (alpha - alpha_0) * decay_factor)
beta_decayed  = max(beta_0, beta_0 + (beta - beta_0) * decay_factor)
// mu stays as-is -- it's a running estimate, not an accumulator
```

The `max` with initial values prevents the parameters from becoming more uninformative than the cold-start prior.

### Surprise Routing Signal Table

The raw surprise score feeds into the routing table:

| Surprise range | Interpretation | Action |
|---|---|---|
| > 2.0 nats | Extreme model violation | Immediate TriageAlert regardless of other scores |
| 0.5 - 2.0 nats | Moderate belief shift | Boost curiosity score by 0.3 |
| 0.1 - 0.5 nats | Minor update | No modification |
| < 0.1 nats | Expected observation | Slight curiosity penalty (-0.1) |

The extreme surprise threshold (> 2.0 nats) acts as an override: if a single observation forces a large belief update, the event gets escalated even if heuristics and similarity would have filtered it. This catches novel attack vectors that the heuristic rules don't anticipate and that have no similar episodes in the ANN index.

### Cold-Start Behavior with Hedge

When the golem has no prior observations for a protocol, the conjugate models start at their uninformative priors. Every observation generates moderate surprise because the models are updating rapidly from weak priors. This is correct behavior: the agent should be curious about protocols it hasn't modeled yet. As observations accumulate, the models become more confident and surprise drops for routine events.

During cold start (fewer than ~50 observations per protocol), heuristics should dominate via the Hedge weighting system. The Bayesian surprise signal carries high variance early on because the models are unstable. The Hedge algorithm naturally downweights signals with poor predictive accuracy, so surprise gets low weight until the models stabilize.

### Performance Characteristics

The entire Bayesian surprise computation -- including prior snapshot, posterior update, KL divergence calculation, and decay -- runs in **under 1 microsecond per transaction** on a single core. The dominant cost is the digamma function evaluation, which takes **~50 nanoseconds per call**. A typical transaction touches 3-5 models (one binary, one continuous, one categorical, one rate), requiring 4-6 digamma calls total.

Memory per protocol: roughly **200 bytes** for a standard feature set (2 binary models, 2 continuous models, 1 categorical model with 10 categories). For 100 tracked protocols, the total footprint is ~20KB. This is negligible compared to the ANN index (megabytes) or the Grimoire (gigabytes).

---

## Active Inference: Theoretical Umbrella for Agent Behavior

> Source: `06-curiosity-learning/01-active-inference.md`

The Free Energy Principle (FEP) provides a single mathematical framework that unifies the golem's curiosity scoring, attention allocation, action selection, and model updating into one objective: minimize surprise. Where Bayesian surprise measures how much a single observation shifts beliefs, active inference prescribes what the agent should *do* about it. The agent can reduce surprise in two ways -- update its model (perception) or change the world (action). Both are governed by the same variational objective. This reframes the entire triage-to-action pipeline as inference rather than engineering: the golem doesn't follow hand-coded rules about when to act; it selects actions that minimize expected free energy, which decomposes naturally into information-seeking (curiosity) and goal-seeking (profit).

### Variational free energy

An agent maintains a generative model -- beliefs q(s) about hidden states s given observations o. Variational free energy is:

```
F = D_KL[ q(s) || p(s|o) ] - ln p(o)
  = D_KL[ q(s) || p(s) ] - E_q[ ln p(o|s) ]
  = complexity - accuracy
```

The first line says free energy upper-bounds surprise (-ln p(o)). The second line decomposes it into complexity (how far beliefs deviate from priors) and accuracy (how well beliefs explain observations). Minimizing free energy forces the agent to find the simplest beliefs (low complexity) that still explain the data (high accuracy).

For the golem, this decomposition maps directly:

- **Hidden states s**: the true state of on-chain protocols (pool reserves, pending liquidations, MEV bot positions, oracle staleness)
- **Observations o**: decoded transaction logs, gas prices, event counts
- **Beliefs q(s)**: the golem's protocol state models plus the Bayesian models from the surprise domain
- **Generative model p(o|s)**: the forward model predicting what observations should occur given the believed state

Friston (2010) showed that any system that maintains its organization over time must, in effect, minimize variational free energy. The golem's mortality-aware lifecycle makes this concrete: a golem that fails to minimize surprise (fails to model its environment accurately) makes bad trades, loses capital, and dies sooner.

### The perception-action cycle

Free energy minimization happens through two complementary pathways:

**Perception (model update):** Adjust q(s) to better explain observations. This is the Bayesian update step -- exactly what the conjugate prior models do at each Gamma tick. When the golem updates its gas price model after observing an unusually high gas price, it's performing perceptual inference.

**Action:** Change the world so that future observations match predictions. When the golem rebalances an LP position that has gone out of range, it's performing active inference -- changing the state of the world to reduce the prediction error between "position should be in range" and "position is out of range."

Both pathways reduce free energy. The golem's tick hierarchy implements them at different timescales:
- **Gamma tick**: Perceptual inference (fast model updates, ~12 seconds)
- **Theta tick**: Mixed inference and planning (LLM analysis, action proposals, ~5 minutes)
- **Delta tick**: Deep perceptual inference (model consolidation, ANN rebuild, ~20 minutes)

### Precision weighting

Precision is the inverse variance of a prediction error. A prediction channel with high precision produces tight, reliable predictions; its errors carry weight. A channel with low precision produces noisy, unreliable predictions; its errors should be downweighted. In active inference, attention *is* precision optimization.

```rust
/// Precision estimate for a single prediction channel.
/// Tracks the inverse variance of recent prediction errors.
#[derive(Clone, Debug)]
pub struct PrecisionEstimate {
    /// Running mean of squared prediction errors
    mean_sq_error: f64,
    /// Running mean of prediction errors (for bias detection)
    mean_error: f64,
    /// Exponential decay factor for the running estimates
    decay: f64,
    /// Number of effective samples
    n_eff: f64,
}

impl PrecisionEstimate {
    pub fn new(decay: f64) -> Self {
        Self {
            mean_sq_error: 1.0, // Start with unit variance (uninformative)
            mean_error: 0.0,
            decay,
            n_eff: 0.0,
        }
    }

    /// Update with a new prediction error. Returns the current precision.
    pub fn update(&mut self, error: f64) -> f64 {
        self.mean_sq_error = self.decay * self.mean_sq_error + (1.0 - self.decay) * error * error;
        self.mean_error = self.decay * self.mean_error + (1.0 - self.decay) * error;
        self.n_eff = self.decay * self.n_eff + 1.0;
        self.precision()
    }

    /// Precision = inverse variance, clamped to avoid division by near-zero.
    pub fn precision(&self) -> f64 {
        let variance = (self.mean_sq_error - self.mean_error.powi(2)).max(1e-8);
        (1.0 / variance).min(1e6) // Clamp to prevent numerical explosion
    }

    /// Is this channel well-calibrated? High precision + low bias = yes.
    pub fn is_calibrated(&self) -> bool {
        self.n_eff > 10.0 && self.mean_error.abs() < 2.0 * self.mean_sq_error.sqrt()
    }
}
```

The FEP predicts a counterintuitive relationship between model confidence and curiosity thresholds. Protocols where the golem has high-precision models should get *lower* curiosity thresholds, not higher. A prediction error from a high-precision channel is more informative than one from a low-precision channel. If the golem's model of Uniswap V3 is tight (high precision), and a Uniswap V3 event violates that model, something genuinely unusual is happening. If the model of some obscure new protocol is loose (low precision), prediction errors are expected and carry less information.

```rust
/// Modulate curiosity threshold based on model precision.
/// High precision -> lower threshold -> more events escalated for that protocol.
/// Low precision -> higher threshold -> fewer events escalated (noise expected).
fn precision_modulated_threshold(
    base_threshold: f32,
    precision: f64,
    max_precision: f64,
) -> f32 {
    let normalized = (precision / max_precision).min(1.0) as f32;
    // High precision: threshold drops to 60% of base
    // Low precision: threshold stays at 100% of base
    base_threshold * (1.0 - 0.4 * normalized)
}
```

### Expected free energy for action selection

Active inference treats planning as inference. Instead of optimizing a reward function (reinforcement learning) or following rules (expert systems), the agent scores candidate actions by their **Expected Free Energy (EFE)**:

```
G(a) = E_q[ D_KL( q(o|a) || p(o) ) ] + E_q[ D_KL( q(s|o,a) || q(s|a) ) ]
     = pragmatic_value              + epistemic_value
     = (expected reward)             + (expected information gain)
```

The first term (pragmatic value) favors actions that lead to observations the agent prefers. For the golem: actions that lead to profitable positions, lower risk exposure, timely rebalancing. The second term (epistemic value) favors actions that resolve uncertainty. For the golem: probe swaps that reveal market depth, monitoring positions to detect out-of-range conditions, investigating unknown protocols.

The balance between these terms is not tuned by a hyperparameter. It falls out of the math: when the agent is uncertain, epistemic value dominates and the agent explores. When the agent is confident, pragmatic value dominates and the agent exploits.

```rust
/// Expected Free Energy for a triage routing action.
pub struct EfeScore {
    /// Pragmatic: how much does this action align with the golem's goals?
    pub pragmatic: f64,
    /// Epistemic: how much uncertainty does this action resolve?
    pub epistemic: f64,
}

impl EfeScore {
    /// Combined EFE. Lower is better (minimizing free energy).
    pub fn combined(&self) -> f64 {
        -(self.pragmatic + self.epistemic)
    }
}

/// Score a routing decision using expected free energy.
fn score_routing_action(
    event: &TriageEvent,
    action: RoutingAction,
    belief_state: &BeliefState,
    golem_preferences: &GolemPreferences,
) -> EfeScore {
    match action {
        RoutingAction::EscalateToLlm => {
            let epistemic = belief_state.information_gain_from_analysis(event);
            let pragmatic = golem_preferences.relevance_to_positions(event);
            EfeScore { pragmatic, epistemic }
        }
        RoutingAction::UpdateStateSilently => {
            let epistemic = 0.1 * belief_state.information_gain_from_analysis(event);
            let pragmatic = 0.3 * golem_preferences.relevance_to_positions(event);
            EfeScore { pragmatic, epistemic }
        }
        RoutingAction::Discard => {
            EfeScore {
                pragmatic: 0.0,
                epistemic: 0.0,
            }
        }
    }
}
```

### The belief state

```rust
/// The golem's beliefs about the state of its environment.
/// Updated at each tick via Bayesian inference.
pub struct BeliefState {
    /// Per-protocol Bayesian surprise models
    pub surprise_models: BayesianSurpriseScorer,
    /// Per-channel precision estimates
    pub precisions: DashMap<(ProtocolId, String), PrecisionEstimate>,
    /// Protocol state predictions for the next observation
    pub predictions: DashMap<ProtocolId, StatePrediction>,
    /// Global regime indicator (stationary vs. changepoint detected)
    pub regime: AtomicU8, // 0 = stationary, 1 = transition, 2 = new regime
}

/// Prediction about what the next observation from a protocol should look like.
#[derive(Clone, Debug)]
pub struct StatePrediction {
    pub expected_event_rate: f64,
    pub expected_gas_range: (f64, f64),
    pub expected_value_range: (f64, f64),
    pub confidence: f64, // derived from precision estimates
}

impl BeliefState {
    /// Process a new observation: update beliefs, compute surprise,
    /// update precisions, generate next prediction.
    pub fn process_observation(
        &self,
        protocol: &ProtocolId,
        features: &TxFeatures,
    ) -> ObservationResult {
        // 1. Compute surprise (updates models internally)
        let surprise = self.surprise_models.score(protocol, features);

        // 2. Compute prediction error against current predictions
        let prediction_error = if let Some(pred) = self.predictions.get(protocol) {
            features.deviation_from(&pred)
        } else {
            1.0 // No prediction = maximum uncertainty
        };

        // 3. Update precision for each feature channel
        let precision = self.update_precisions(protocol, features, prediction_error);

        // 4. Precision-weighted prediction error
        let weighted_error = precision * prediction_error;

        // 5. Generate next prediction (simple EMA-based)
        self.update_predictions(protocol, features);

        ObservationResult {
            surprise,
            prediction_error,
            precision_weighted_error: weighted_error,
            regime_change_detected: weighted_error > REGIME_THRESHOLD,
        }
    }

    /// Estimate information gain from LLM analysis of an event.
    pub fn information_gain_from_analysis(&self, event: &TriageEvent) -> f64 {
        let avg_precision = self.average_precision(&event.protocol_id);
        let uncertainty = 1.0 / (avg_precision + 1.0);
        let surprise = event.curiosity_score as f64;
        surprise * uncertainty
    }

    fn update_precisions(
        &self,
        protocol: &ProtocolId,
        features: &TxFeatures,
        error: f64,
    ) -> f64 {
        let key = (protocol.clone(), "aggregate".to_string());
        let mut entry = self.precisions
            .entry(key)
            .or_insert_with(|| PrecisionEstimate::new(0.95));
        entry.update(error)
    }

    fn average_precision(&self, protocol: &ProtocolId) -> f64 {
        let key = (protocol.clone(), "aggregate".to_string());
        self.precisions
            .get(&key)
            .map(|p| p.precision())
            .unwrap_or(1.0)
    }

    fn update_predictions(&self, protocol: &ProtocolId, features: &TxFeatures) {
        let alpha = 0.1;
        let mut pred = self.predictions
            .entry(protocol.clone())
            .or_insert_with(|| StatePrediction::from_features(features));

        pred.expected_event_rate = pred.expected_event_rate * (1.0 - alpha)
            + features.event_rate * alpha;
        pred.expected_gas_range = (
            pred.expected_gas_range.0 * (1.0 - alpha) + features.gas_price * alpha,
            pred.expected_gas_range.1.max(features.gas_price),
        );
        pred.confidence = self.average_precision(protocol).min(1.0);
    }
}
```

### The active inference agent

```rust
/// Active inference agent wrapping the belief state and action selection.
pub struct ActiveInferenceAgent {
    pub beliefs: BeliefState,
    pub preferences: GolemPreferences,
}

impl ActiveInferenceAgent {
    /// Select the best routing action for a triage event.
    /// Returns the action with the lowest expected free energy.
    pub fn select_routing(&self, event: &TriageEvent) -> RoutingAction {
        let actions = [
            RoutingAction::EscalateToLlm,
            RoutingAction::UpdateStateSilently,
            RoutingAction::Discard,
        ];

        actions
            .iter()
            .map(|action| {
                let efe = score_routing_action(
                    event,
                    *action,
                    &self.beliefs,
                    &self.preferences,
                );
                (*action, efe.combined())
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(action, _)| action)
            .unwrap_or(RoutingAction::Discard)
    }

    /// Process an observation and update the agent's beliefs.
    pub fn observe(&self, protocol: &ProtocolId, features: &TxFeatures) -> ObservationResult {
        self.beliefs.process_observation(protocol, features)
    }

    /// Gamma tick maintenance: decay models, update regime detection.
    pub fn gamma_tick(&self) {
        self.beliefs.surprise_models.decay_gamma();
    }

    /// Theta tick maintenance: decay categorical models,
    /// check for regime changes across protocols.
    pub fn theta_tick(&self) {
        self.beliefs.surprise_models.decay_theta();
    }
}
```

### Behavioral phases as shifted priors

The golem's behavioral phases (thriving, declining, terminal) map naturally to the FEP as shifted priors over preferred observations. In the thriving phase, the golem "expects" (prefers) observations consistent with capital growth: profitable trade opportunities, healthy position states. In the terminal phase, the golem "expects" observations consistent with capital preservation: stable positions, low-risk states.

This shift in priors changes the pragmatic value term of EFE without touching the epistemic term. A terminal golem still seeks information (epistemic value is unchanged), but it no longer seeks profit opportunities (pragmatic value shifts toward conservation). The math produces the right behavior: a dying golem explores its environment to generate knowledge for its successor (high epistemic value) while avoiding risky trades (low pragmatic value for profit-seeking actions).

### Connection to the triage pipeline

Active inference doesn't replace the existing triage pipeline. It provides a theoretical grounding for decisions that the pipeline already makes heuristically:

| Triage decision | Current implementation | Active inference interpretation |
|---|---|---|
| Bloom filter pre-screen | Fixed bloom membership | Prior-based gating: events outside the generative model are pre-filtered |
| Address triage | DashSet membership check | Attention allocation: only attend to entities within the generative model |
| Curiosity scoring | Heuristic + ANN similarity | Free energy: surprise (Bayesian) + prediction error (precision-weighted) |
| Score routing thresholds | Fixed brackets (>0.8, 0.5-0.8, etc.) | EFE-based action selection: route to minimize expected free energy |
| Heuristic-to-learned weight shift | Linear interpolation by episode count | Precision weighting: shift from prior-dominated to likelihood-dominated as data accumulates |

The value of this mapping is that it identifies missing capabilities: precision weighting isn't implemented, EFE-based routing isn't implemented, and the exploration-exploitation balance is currently managed by hand-tuned interpolation rather than falling out of the math.

### Active inference references

- Friston, K. (2010). "The Free-Energy Principle: A Unified Brain Theory?" *Nature Reviews Neuroscience*, 11(2), 127-138. — *Proposes that all adaptive systems minimize variational free energy; the unifying principle connecting the Oracle's prediction error minimization to the Golem's overall survival strategy.*
- Parr, T. & Friston, K.J. (2019). "Generalised Free Energy and Active Inference." *Biological Cybernetics*, 113(5), 495-513. — *Extends free energy to include expected free energy for planning; formalizes the explore-exploit balance the Oracle uses when deciding between refining known models and probing new ones.*
- Da Costa, L., Parr, T., Sajid, N., Veselic, S., Neacsu, V. & Friston, K. (2020). "Active Inference on Discrete State-Spaces: A Synthesis." *Journal of Mathematical Psychology*, 99, 102447. — *Provides a tractable discrete-state implementation of active inference; the computational template for the Oracle's finite routing decisions.*
- Parr, T., Pezzulo, G. & Friston, K. (2022). *Active Inference: The Free Energy Principle in Mind, Brain, and Behavior.* MIT Press. — *The definitive textbook on active inference; primary reference for connecting the Oracle's prediction-correction loop to the broader Golem cognitive architecture.*
- Feldman, H. & Friston, K. (2010). "Attention, Uncertainty, and Free-Energy." *Frontiers in Human Neuroscience*, 4, 215. — *Models attention as precision optimization on prediction errors; justifies the Oracle's precision-weighted surprise signals for attention allocation.*

---

## Online Learning for Adaptive Triage

> Source: `06-curiosity-learning/02-online-learning.md`

The triage pipeline's thresholds and signal weights are currently static. This section introduces four online learning algorithms -- Hedge, Thompson sampling, LinUCB, and epsilon-greedy with decay -- that let the triage pipeline adapt these parameters based on downstream feedback. Each algorithm fills a different niche: Hedge combines multiple expert signals with provable regret bounds; Thompson sampling balances exploration and exploitation of triage thresholds; LinUCB adds context-dependent routing; and epsilon-greedy provides a dead-simple baseline for comparison.

All four algorithms share a common feedback loop: the golem takes an action (escalate, route, discard), observes the outcome (LLM analysis was useful, trade was profitable, event was irrelevant), and updates the algorithm's internal state. The feedback is delayed (outcomes arrive at Theta or Delta tick, not Gamma) and partial (discarded events never receive feedback). Both issues are addressed below.

### Hedge / exponential weights

Hedge (Freund & Schapire, 1997) maintains a weight for each "expert" -- in our case, each curiosity signal. At each round, the algorithm observes each expert's loss and multiplicatively updates weights:

```
w_i(t+1) = w_i(t) * exp(-eta * loss_i(t))
normalize: w_i(t+1) = w_i(t+1) / sum(w_j(t+1))
```

With eta = sqrt(ln(N) / T) for N experts and T rounds, Hedge guarantees O(sqrt(T * ln(N))) regret. For N=5 signals and T=10,000 triage events, this translates to ~28 nats of cumulative regret over the golem's lifetime.

The triage pipeline grows from two curiosity signals to five:

1. **Heuristic score** -- rule-based, O(1), from existing Stage 4
2. **ANN similarity** -- embedding distance, from existing Layer 2
3. **Bayesian surprise** -- KL divergence from conjugate models
4. **Prediction error** -- deviation from the forward model's expected state
5. **Anomaly score** -- MIDAS-R edge anomaly or BOCPD changepoint signal

```rust
/// Hedge algorithm for combining curiosity signals.
pub struct HedgeWeights {
    weights: Vec<f64>,
    eta: f64,
    /// Track cumulative loss per expert for diagnostics
    cumulative_loss: Vec<f64>,
    rounds: u64,
}

impl HedgeWeights {
    pub fn new(num_experts: usize, eta: f64) -> Self {
        let uniform = 1.0 / num_experts as f64;
        Self {
            weights: vec![uniform; num_experts],
            eta,
            cumulative_loss: vec![0.0; num_experts],
            rounds: 0,
        }
    }

    /// Compute weighted combination of expert scores.
    pub fn combine(&self, scores: &[f64]) -> f64 {
        assert_eq!(scores.len(), self.weights.len());
        self.weights
            .iter()
            .zip(scores.iter())
            .map(|(w, s)| w * s)
            .sum()
    }

    /// Update weights after observing loss for each expert.
    /// Loss should be in [0, 1] -- lower is better.
    pub fn update(&mut self, losses: &[f64]) {
        assert_eq!(losses.len(), self.weights.len());

        for (i, loss) in losses.iter().enumerate() {
            self.weights[i] *= (-self.eta * loss).exp();
            self.cumulative_loss[i] += loss;
        }

        // Normalize
        let total: f64 = self.weights.iter().sum();
        if total > 0.0 {
            for w in self.weights.iter_mut() {
                *w /= total;
            }
        }
        self.rounds += 1;
    }

    /// Current weight distribution (for diagnostics and logging).
    pub fn distribution(&self) -> &[f64] {
        &self.weights
    }

    /// Adaptive eta based on round count.
    pub fn adapt_eta(&mut self) {
        let n = self.weights.len() as f64;
        let t = (self.rounds + 1) as f64;
        self.eta = (n.ln() / t).sqrt();
    }
}
```

### Thompson sampling for threshold exploration

Thompson sampling (Thompson, 1933) maintains a probability distribution over each parameter's "goodness" and samples from it to make decisions. For triage threshold exploration:

```rust
use rand::Rng;
use rand_distr::Beta as BetaDist;

/// Thompson sampling for triage threshold exploration.
pub struct ThompsonThresholds {
    arms: Vec<ThresholdArm>,
}

#[derive(Clone, Debug)]
pub struct ThresholdArm {
    pub high_threshold: f32,
    pub medium_threshold: f32,
    pub low_threshold: f32,
    pub alpha: f64,
    pub beta: f64,
}

impl ThompsonThresholds {
    pub fn new() -> Self {
        let arms = vec![
            ThresholdArm { high_threshold: 0.8, medium_threshold: 0.5, low_threshold: 0.2, alpha: 2.0, beta: 2.0 },
            ThresholdArm { high_threshold: 0.7, medium_threshold: 0.4, low_threshold: 0.15, alpha: 1.0, beta: 1.0 },
            ThresholdArm { high_threshold: 0.85, medium_threshold: 0.55, low_threshold: 0.25, alpha: 1.0, beta: 1.0 },
            ThresholdArm { high_threshold: 0.75, medium_threshold: 0.45, low_threshold: 0.2, alpha: 1.0, beta: 1.0 },
            ThresholdArm { high_threshold: 0.9, medium_threshold: 0.6, low_threshold: 0.3, alpha: 1.0, beta: 1.0 },
        ];
        Self { arms }
    }

    pub fn select_thresholds(&self, rng: &mut impl Rng) -> &ThresholdArm {
        let mut best_idx = 0;
        let mut best_sample = f64::NEG_INFINITY;
        for (i, arm) in self.arms.iter().enumerate() {
            let dist = BetaDist::new(arm.alpha, arm.beta).unwrap();
            let sample: f64 = rng.sample(dist);
            if sample > best_sample {
                best_sample = sample;
                best_idx = i;
            }
        }
        &self.arms[best_idx]
    }

    pub fn update(&mut self, arm_idx: usize, success: bool) {
        if success {
            self.arms[arm_idx].alpha += 1.0;
        } else {
            self.arms[arm_idx].beta += 1.0;
        }
    }

    pub fn decay(&mut self, factor: f64) {
        for arm in self.arms.iter_mut() {
            arm.alpha = 1.0 + (arm.alpha - 1.0) * factor;
            arm.beta = 1.0 + (arm.beta - 1.0) * factor;
        }
    }
}
```

#### Arousal-conditioned threshold selection

The golem's arousal state (from CorticalState) should influence threshold selection. During high arousal, the golem uses lower thresholds (escalate more events). During low arousal, higher thresholds (be more selective).

```rust
pub struct ContextualThompson {
    regimes: HashMap<ArousalRegime, ThompsonThresholds>,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ArousalRegime {
    Low,      // arousal < 0.3
    Medium,   // 0.3 <= arousal < 0.7
    High,     // arousal >= 0.7
}

impl ContextualThompson {
    pub fn select(&self, arousal: f32, rng: &mut impl Rng) -> &ThresholdArm {
        let regime = match arousal {
            a if a < 0.3 => ArousalRegime::Low,
            a if a < 0.7 => ArousalRegime::Medium,
            _ => ArousalRegime::High,
        };
        self.regimes[&regime].select_thresholds(rng)
    }
}
```

### LinUCB for contextual routing

LinUCB (Li et al., 2010) extends UCB to contextual bandits. It maintains a linear model that predicts reward as a function of context features. The confidence bound comes from ridge regression uncertainty:

```
For context x and arm a:
  A_a = I + sum(x_t * x_t^T)       -- d x d matrix
  b_a = sum(r_t * x_t)             -- d-vector
  theta_a = A_a^{-1} * b_a          -- parameter estimate
  UCB_a = theta_a^T * x + alpha * sqrt(x^T * A_a^{-1} * x)
```

```rust
use nalgebra::{DMatrix, DVector};

/// LinUCB contextual bandit for triage routing.
pub struct LinUcbRouter {
    arms: Vec<LinUcbArm>,
    alpha: f64,
    d: usize,
}

pub struct LinUcbArm {
    a_matrix: DMatrix<f64>,
    b_vector: DVector<f64>,
    theta: DVector<f64>,
    a_inv: DMatrix<f64>,
}

impl LinUcbArm {
    pub fn new(d: usize) -> Self {
        let identity = DMatrix::identity(d, d);
        Self {
            a_matrix: identity.clone(),
            b_vector: DVector::zeros(d),
            theta: DVector::zeros(d),
            a_inv: identity,
        }
    }

    pub fn ucb(&self, x: &DVector<f64>, alpha: f64) -> f64 {
        let exploitation = self.theta.dot(x);
        let exploration = alpha * (x.transpose() * &self.a_inv * x)[(0, 0)].sqrt();
        exploitation + exploration
    }

    pub fn update(&mut self, x: &DVector<f64>, reward: f64) {
        let a_inv_x = &self.a_inv * x;
        let denom = 1.0 + (x.transpose() * &a_inv_x)[(0, 0)];
        self.a_inv -= (&a_inv_x * a_inv_x.transpose()) / denom;
        self.a_matrix += x * x.transpose();
        self.b_vector += reward * x;
        self.theta = &self.a_inv * &self.b_vector;
    }
}

impl LinUcbRouter {
    pub fn new(d: usize, num_arms: usize, alpha: f64) -> Self {
        Self {
            arms: (0..num_arms).map(|_| LinUcbArm::new(d)).collect(),
            alpha,
            d,
        }
    }

    /// Build context vector from a triage event.
    pub fn build_context(event: &TriageEvent, cortical: &CorticalState) -> DVector<f64> {
        DVector::from_vec(vec![
            event.curiosity_score as f64,
            event.bayesian_surprise,
            event.prediction_error,
            event.anomaly_score,
            cortical.arousal as f64,
            cortical.valence as f64,
            if event.involves_active_position { 1.0 } else { 0.0 },
            if event.protocol_id.is_some() { 1.0 } else { 0.0 },
            event.gas_ratio,
            event.value_usd.log10().max(0.0),
            cortical.chain_blocks_behind as f64,
            event.time_since_last_escalation_secs,
        ])
    }

    pub fn select(&self, context: &DVector<f64>) -> usize {
        self.arms
            .iter()
            .enumerate()
            .map(|(i, arm)| (i, arm.ucb(context, self.alpha)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    pub fn update(&mut self, arm_idx: usize, context: &DVector<f64>, reward: f64) {
        self.arms[arm_idx].update(context, reward);
    }
}
```

At d=12 and 3 arms, LinUCB uses 432 f64 values (3.4KB total). The Sherman-Morrison update avoids full matrix inversion and runs in O(d^2) time, about 1 microsecond at d=12.

### Epsilon-greedy with decay

The simplest exploration strategy, included as a comparison baseline and fallback:

```rust
pub struct EpsilonGreedy {
    epsilon: f64,
    epsilon_min: f64,
    decay_rate: f64,
    arm_means: Vec<f64>,
    arm_counts: Vec<u64>,
}

impl EpsilonGreedy {
    pub fn new(num_arms: usize, epsilon: f64, epsilon_min: f64, decay_rate: f64) -> Self {
        Self {
            epsilon,
            epsilon_min,
            decay_rate,
            arm_means: vec![0.0; num_arms],
            arm_counts: vec![0; num_arms],
        }
    }

    pub fn select(&self, rng: &mut impl Rng) -> usize {
        if rng.gen::<f64>() < self.epsilon {
            rng.gen_range(0..self.arm_means.len())
        } else {
            self.arm_means
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0)
        }
    }

    pub fn update(&mut self, arm: usize, reward: f64) {
        self.arm_counts[arm] += 1;
        let n = self.arm_counts[arm] as f64;
        self.arm_means[arm] += (reward - self.arm_means[arm]) / n;
        self.epsilon = (self.epsilon * self.decay_rate).max(self.epsilon_min);
    }
}
```

Starting epsilon at 0.2 and decaying to 0.01 with rate 0.999 per round gives about 5000 rounds of meaningful exploration before settling into near-pure exploitation. For a golem processing ~100 triage events per Theta tick, that's about 50 Theta ticks, roughly 4 hours of operation.

### The adaptive triage router

The algorithms compose into a single router:

```rust
pub struct AdaptiveTriageRouter {
    /// Hedge combines the 5 curiosity signals into a single score
    pub signal_combiner: HedgeWeights,
    /// Thompson sampling selects threshold configurations
    pub threshold_selector: ContextualThompson,
    /// LinUCB makes the final routing decision using full context
    pub router: LinUcbRouter,
    /// Epsilon-greedy as comparison baseline (runs in shadow mode)
    pub baseline: EpsilonGreedy,
}

impl AdaptiveTriageRouter {
    pub fn process(
        &mut self,
        event: &TriageEvent,
        expert_scores: &[f64],
        cortical: &CorticalState,
        rng: &mut impl Rng,
    ) -> RoutingDecision {
        // 1. Hedge combines expert signals
        let combined_score = self.signal_combiner.combine(expert_scores);

        // 2. Thompson selects thresholds for this arousal regime
        let thresholds = self.threshold_selector.select(cortical.arousal, rng);

        // 3. Preliminary routing by threshold
        let preliminary = if combined_score > thresholds.high_threshold as f64 {
            RoutingAction::EscalateToLlm
        } else if combined_score > thresholds.medium_threshold as f64 {
            RoutingAction::EmitChainEvent
        } else if combined_score > thresholds.low_threshold as f64 {
            RoutingAction::UpdateSilently
        } else {
            RoutingAction::Discard
        };

        // 4. LinUCB can override for borderline cases
        let context = LinUcbRouter::build_context(event, cortical);
        let linucb_action = self.router.select(&context);

        // 5. Shadow-run epsilon-greedy for baseline comparison
        let _baseline_action = self.baseline.select(rng);

        // 6. Final decision: use LinUCB for borderline, threshold for clear cases
        let final_action = if (combined_score - thresholds.high_threshold as f64).abs() < 0.1 {
            RoutingAction::from_index(linucb_action)
        } else {
            preliminary
        };

        RoutingDecision {
            action: final_action,
            combined_score,
            context,
            threshold_arm_idx: 0,
        }
    }
}
```

### Feedback loop and partial observability

The delayed feedback (minutes between action and outcome) doesn't affect algorithm correctness -- it just means the learner updates less frequently than it acts. The partial feedback (discarded events never get LLM analysis) is the harder problem. Three mitigation strategies:

**Strategy 1: Epsilon-exploration of discards.** With probability epsilon, route a below-threshold event to LLM analysis anyway. This provides counterfactual labels for the low-score region. Set epsilon low (0.01-0.05) to bound the LLM cost.

**Strategy 2: Implicit negative signal.** Events that are discarded and never re-surface are treated as true negatives after a time window. If nothing interesting happened to the discarded event's protocol in the next N blocks, the discard was correct.

**Strategy 3: Retroactive re-scoring.** When ABI resolution identifies a previously unknown contract, retroactively re-score its past events. Events whose score changes significantly are treated as missed positives, providing negative reward to the routing action that discarded them.

### Persistence across golem generations

Online learning state is small (a few KB total) and represents hard-won knowledge about which signals predict interesting events. This state should be included in the generational inheritance package:

- **Hedge weights**: Which curiosity signals matter? ~40 bytes.
- **Thompson posteriors**: Which thresholds work in which arousal regimes? ~240 bytes.
- **LinUCB parameters**: Full A^{-1} and theta per arm. ~3.4KB.
- **Epsilon-greedy means**: Baseline arm rewards. ~24 bytes.

A successor golem inheriting these parameters starts with calibrated signal weights and threshold configurations rather than re-learning from scratch. The decay mechanisms in Thompson sampling and the adaptive eta in Hedge ensure the successor can still adapt to changed conditions.

### Online learning references

- Freund, Y. & Schapire, R.E. (1997). "A Decision-Theoretic Generalization of On-Line Learning and an Application to Boosting." *Journal of Computer and System Sciences*, 55(1), 119-139. — *Introduces the Hedge algorithm for combining expert predictions with multiplicative weight updates and provable regret bounds; the algorithm used to combine curiosity signal weights in the adaptive triage router.*
- Thompson, W.R. (1933). "On the Likelihood that One Unknown Probability Exceeds Another in View of the Evidence of Two Samples." *Biometrika*, 25(3/4), 285-294. — *The original Thompson sampling paper showing Bayesian posterior sampling for exploration; the triage router uses this to explore threshold configurations.*
- Li, L., Chu, W., Langford, J. & Schapire, R.E. (2010). "A Contextual-Bandit Approach to Personalized News Article Recommendation." *Proceedings of the 19th International Conference on World Wide Web (WWW)*, 661-670. — *Introduces LinUCB for contextual bandits with linear reward models; the algorithm the triage router uses for context-dependent routing decisions.*
- Russo, D., Van Roy, B., Kazerouni, A., Osband, I. & Wen, Z. (2018). "A Tutorial on Thompson Sampling." *Foundations and Trends in Machine Learning*, 11(1), 1-96. — *Comprehensive tutorial on Thompson sampling covering theory, variants, and practical implementation; reference for the arousal-conditioned threshold selection.*
- Agrawal, S. & Goyal, N. (2012). "Analysis of Thompson Sampling for the Multi-Armed Bandit Problem." *Proceedings of the 25th Conference on Learning Theory (COLT)*. — *Proves O(sqrt(KT log T)) regret for Thompson sampling, matching UCB bounds; the theoretical guarantee for the triage threshold exploration.*
- Lattimore, T. & Szepesvari, C. (2020). *Bandit Algorithms.* Cambridge University Press. — *Definitive textbook on bandit algorithms covering UCB, Thompson sampling, and contextual bandits; the primary reference for the triage router's online learning framework.*
- Auer, P., Cesa-Bianchi, N., Freund, Y. & Schapire, R.E. (2002). "The Nonstochastic Multiarmed Bandit Problem." *SIAM Journal on Computing*, 32(1), 48-77. — *Extends bandit algorithms to adversarial (non-stochastic) settings; provides robustness guarantees for the triage router when market conditions shift adversarially.*

---
