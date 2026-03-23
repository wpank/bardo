# 03b -- Cognitive Mechanisms [SPEC]

**Attention Salience, Sleep Pressure, Habituation, Homeostasis, Compensation, Event Wakeup**

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-runtime` | **Layer**: 0 (FOUNDATION)
>
> Cross-references: [03-mind.md](./03-mind.md) (overview), [02-heartbeat.md](./02-heartbeat.md) (heartbeat pipeline), [18-cortical-state.md](./18-cortical-state.md) (CorticalState)
>
> Sources: `03-agent-runtime/01-attention-salience`, `03-agent-runtime/02-sleep-consolidation`, `03-agent-runtime/03-habituation`, `03-agent-runtime/04-homeostasis`, `03-agent-runtime/05-compensation-rollback`, `03-agent-runtime/06-event-wakeup`

> **Reader orientation:** This document specifies six cognitive mechanisms that modulate how the Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) Heartbeat (the 9-step decision cycle) operates: attention salience, sleep pressure, habituation, homeostasis, compensation/rollback, and event-driven wakeup. It belongs to the `01-golem` cognition layer, in the `golem-runtime` crate. These mechanisms are concurrent processes that run alongside the heartbeat pipeline, not heartbeat steps themselves. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## 1. Attention Salience

The AttentionSalience queue determines which observations get priority at Step 1 (OBSERVE). It is a `BinaryHeap` ordered by salience score, with exponential decay applied at each gamma tick so that stale stimuli lose priority.

### Theoretical grounding

Cherry's dichotic listening experiments (1953) established that selective attention operates on structural and acoustic features before semantic content. Subjects in a dual-channel setup filter one channel while suppressing the other. The filtering is pre-attentive, not deliberate. AttentionSalience operates on the same principle: stimuli are scored by kind and structural shape before any LLM-level content analysis.

Baars' Global Workspace Theory (1988) describes a central broadcast mechanism where competing specialized processors vie for access to a global resource (consciousness or focal attention). AttentionSalience is the competition mechanism; the theta tick's context prefix is the broadcast. Items that win the competition enter the workspace; losers silently expire.

The three-term scoring formula derives from Itti and Koch's computational saliency model (2001), which collapses bottom-up stimulus novelty and top-down task relevance into a scalar.

Corbetta and Shulman (2002) distinguish dorsal (top-down, goal-directed) from ventral (bottom-up, stimulus-driven) attention networks. The relevance term is dorsal; the novelty term is ventral. Both are needed. Pure bottom-up attention gets hijacked by noise; pure top-down attention misses regime changes.

### Salience computation

Each new observation receives a baseline salience from three factors:

```
baseline = novelty * 0.4 + relevance * 0.35 + urgency * 0.25
```

- **Novelty** (0.4 weight): How different is this observation from recent history? Multiplied by HabituationMask attenuation.
- **Relevance** (0.35 weight): How related is this to the Golem's current regime and task? Regime-dependent scoring.
- **Urgency** (0.25 weight): Is this time-sensitive? Derived from time-to-expiry in [0.0, 1.0].

### Stimulus types and patterns (source implementation)

```rust
// crates/golem-core/src/attention_salience.rs

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

use crate::cortical_state::CorticalState;
use crate::habituation_mask::HabituationMask;

/// Coarse hash of (StimulusKind + structural payload shape).
/// Blake3 over canonical CBOR of kind + payload schema version, truncated to 16 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StimulusPattern(pub [u8; 16]);

/// Unique monotonic ID for a single stimulus event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StimulusId(pub u64);

impl StimulusId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(1);
        StimulusId(CTR.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StimulusKind {
    PheromoneHit,
    PriceAnomaly,
    ChainConfirmation,
    OwnerMessage,
    DreamRequest,
    PolicyViolation,
}
```

### Stimulus payload

```rust
/// Opaque payload. Callers downcast to the concrete event type they expect.
#[derive(Debug, Clone)]
pub struct StimulusPayload {
    pub kind: StimulusKind,
    pub pattern: StimulusPattern,
    /// Raw urgency signal in [0.0, 1.0], derived from time-to-expiry.
    pub urgency: f32,
    pub bytes: Vec<u8>,
}

impl StimulusPayload {
    pub fn urgency(&self) -> f32 {
        self.urgency.clamp(0.0, 1.0)
    }
}
```

### Salience baseline with relevance scoring

The baseline maintains per-kind rolling statistics used to compute novelty and relevance. It reads `CorticalState.regime` to bias relevance scoring: a price anomaly is more relevant in a volatile regime than a calm one. `HabituationMask` feeds the novelty term.

```rust
pub struct SalienceBaseline {
    urgency_ema: std::collections::HashMap<StimulusKind, f32>,
    ema_alpha: f32,
    habituation: HabituationMask,
}

impl SalienceBaseline {
    pub fn new(ema_alpha: f32, habituation: HabituationMask) -> Self {
        SalienceBaseline {
            urgency_ema: std::collections::HashMap::new(),
            ema_alpha,
            habituation,
        }
    }

    /// Novelty in [0.0, 1.0]. Multiplied by HabituationMask attenuation.
    pub fn novelty_score(
        &mut self,
        kind: StimulusKind,
        payload: &StimulusPayload,
        tick: u64,
    ) -> f32 {
        let attenuation = self.habituation.observe(payload.pattern, tick);
        let ema = self.urgency_ema.entry(kind).or_insert(payload.urgency);
        let deviation = (payload.urgency - *ema).abs();
        *ema = self.ema_alpha * payload.urgency + (1.0 - self.ema_alpha) * *ema;
        (deviation * attenuation).clamp(0.0, 1.0)
    }

    /// Relevance in [0.0, 1.0]. Higher in regimes where this kind matters.
    pub fn relevance_score(
        &self, kind: StimulusKind, state: &CorticalState,
    ) -> f32 {
        use crate::cortical_state::MarketRegime;
        match (kind, state.regime()) {
            (StimulusKind::PolicyViolation, _)                   => 1.0,
            (StimulusKind::OwnerMessage, _)                      => 0.90,
            (StimulusKind::PriceAnomaly, MarketRegime::Volatile) => 0.95,
            (StimulusKind::PriceAnomaly, _)                      => 0.55,
            (StimulusKind::PheromoneHit, MarketRegime::Volatile) => 0.70,
            (StimulusKind::PheromoneHit, _)                      => 0.45,
            (StimulusKind::ChainConfirmation, _)                 => 0.65,
            (StimulusKind::DreamRequest, _)                      => 0.40,
        }
    }
}
```

### The salience queue (source implementation)

```rust
#[derive(Debug, Clone)]
pub struct SalienceItem {
    pub id: StimulusId,
    pub kind: StimulusKind,
    pub score: f32,
    pub payload: StimulusPayload,
    pub expires_at: Instant,
}

impl PartialEq for SalienceItem {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for SalienceItem {}

impl PartialOrd for SalienceItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SalienceItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(Ordering::Equal)
    }
}

pub struct AttentionSalience {
    queue: BinaryHeap<SalienceItem>,
    decay_alpha: f32,   // per-tick exponential decay, e.g. 0.85
    top_n: usize,       // max items injected per theta tick
}

impl AttentionSalience {
    pub fn new(decay_alpha: f32, top_n: usize) -> Self {
        AttentionSalience {
            queue: BinaryHeap::new(),
            decay_alpha,
            top_n,
        }
    }

    /// Score and enqueue a new stimulus. Called from Step 3 (Gate).
    pub fn push(
        &mut self,
        payload: StimulusPayload,
        baseline: &mut SalienceBaseline,
        state: &CorticalState,
        tick: u64,
    ) {
        let novelty   = baseline.novelty_score(payload.kind, &payload, tick);
        let relevance = baseline.relevance_score(payload.kind, state);
        let urgency   = payload.urgency();
        let score     = novelty * 0.4 + relevance * 0.35 + urgency * 0.25;
        self.queue.push(SalienceItem {
            id: StimulusId::new(),
            kind: payload.kind,
            score,
            payload,
            expires_at: Instant::now() + Duration::from_secs(300),
        });
    }

    /// Decay all scores, drop expired items, return top-N for context injection.
    /// Called once per tick at Step 3 (Gate).
    pub fn tick(&mut self) -> Vec<SalienceItem> {
        let now   = Instant::now();
        let alpha = self.decay_alpha;
        let items: Vec<SalienceItem> = self.queue.drain().collect();
        self.queue = items
            .into_iter()
            .filter(|i| i.expires_at > now)
            .map(|mut i| { i.score *= alpha; i })
            .collect();
        let mut top: Vec<SalienceItem> = self.queue.iter().cloned().collect();
        top.sort_unstable_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal)
        });
        top.truncate(self.top_n);
        top
    }

    /// Reset habit baselines on regime change so prior novelty
    /// distributions no longer apply.
    pub fn on_regime_change(&mut self, baseline: &mut SalienceBaseline) {
        for item in self.queue.iter() {
            baseline.habituation.reset_pattern(item.payload.pattern);
        }
    }
}
```

### Salience item expiry

Items have a 300-second hard cutoff that prevents extremely stale items from consuming heap memory.

### Decay dynamics

Each tick, every item's score is multiplied by `decay_alpha` (default 0.85). A stimulus that scored 1.0 on arrival decays to:

| Ticks | Score |
|-------|-------|
| 1 | 0.85 |
| 5 | 0.44 |
| 10 | 0.20 |
| 20 | 0.04 |

Items with high initial scores from urgency or relevance persist longer. Items that scored high only on novelty fade quickly as they become old news.

---

## 2. Sleep Pressure

Sleep pressure replaces the fixed delta-tick counter for triggering dream consolidation. It accumulates based on cognitive complexity, not wall-clock time. A Golem that processed 50 T2 deliberations builds sleep pressure faster than one that coasted on T0 suppressions, even if the same number of ticks elapsed.

### Theoretical grounding

Borbely's two-process model (1982) describes sleep regulation through the interaction of Process S (sleep pressure, accumulates during waking, dissipates during sleep) and Process C (circadian rhythm). `SleepPressure` implements Process S. The `AdaptiveClock`'s delta-tick schedule handles Process C.

Porkka-Heiskanen et al. (1997) identified adenosine as the molecular substrate of Process S, establishing that it accumulates proportionally to neural activity. The `context_pressure` term in `record_tick()` is the computational analog: higher context utilization equals more "neural activity" equals faster accumulation.

Tononi and Cirelli's Synaptic Homeostasis Hypothesis (2006) argues that wake activity net-increases synaptic strength while sleep downscales it to restore signal-to-noise ratio. Dream consolidation in Bardo does the same: it compresses the context window (downscaling) to improve the ratio of useful information to noise for the next waking cycle.

Stickgold (2005) demonstrated that sleep consolidates declarative memory into more efficient long-term representations. This is the functional motivation for dream consolidation: compacted context is more inference-efficient than raw accumulated history.

### Two-term accumulator formula (source implementation)

The accumulator has two input terms per tick. A flat elapsed-tick component (weighted `1.0 - complexity_weight`) ensures that even idle ticks contribute some pressure. A context-complexity component (weighted `complexity_weight`, default 0.6) scales with `tokens_used / context_budget`:

```
flat = 1.0 - complexity_weight
load = complexity_weight * pressure
increment = flat + load
```

```rust
// crates/golem-dreams/src/sleep_pressure.rs

/// Load-weighted accumulator that determines when dream consolidation
/// is warranted. Replaces DreamScheduler's simple tick counter.
pub struct SleepPressure {
    accumulator: f32,
    threshold: f32,
    /// Weight on context-complexity term vs. flat-elapsed-tick term. In [0, 1].
    complexity_weight: f32,
    /// Minimum ticks between consecutive consolidation triggers.
    /// Guards against a storm of expensive consolidations during a busy session.
    min_ticks_between: u32,
    ticks_since_last_reset: u32,
}

impl SleepPressure {
    pub fn new(threshold: f32) -> Self {
        SleepPressure {
            accumulator: 0.0,
            threshold,
            complexity_weight: 0.6,
            min_ticks_between: 5,
            ticks_since_last_reset: 0,
        }
    }

    pub fn with_complexity_weight(mut self, w: f32) -> Self {
        self.complexity_weight = w.clamp(0.0, 1.0);
        self
    }

    pub fn with_min_ticks_between(mut self, n: u32) -> Self {
        self.min_ticks_between = n;
        self
    }

    /// Call after each theta tick completes.
    ///
    /// `context_pressure` = tokens_used / context_budget, in [0, 1].
    /// Source: `ActiveContext::pressure()` in golem-context.
    pub fn record_tick(&mut self, context_pressure: f32) {
        let pressure = context_pressure.clamp(0.0, 1.0);
        let flat     = 1.0 - self.complexity_weight;
        let load     = self.complexity_weight * pressure;
        self.accumulator += flat + load;
        self.ticks_since_last_reset += 1;
    }

    /// Returns true when consolidation should be scheduled.
    /// Respects the min_ticks_between guard.
    pub fn needs_consolidation(&self) -> bool {
        self.accumulator >= self.threshold
            && self.ticks_since_last_reset >= self.min_ticks_between
    }

    /// Call when a dream cycle completes successfully.
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
        self.ticks_since_last_reset = 0;
    }

    /// Normalized pressure in [0.0, 1.0]. Exposed to TUI nooscopy view
    /// and to MetricsEmitter for operational dashboards.
    pub fn pressure(&self) -> f32 {
        (self.accumulator / self.threshold).min(1.0)
    }

    /// Raw accumulator value. Serialize this across process restarts
    /// so consolidation history survives crashes.
    pub fn raw_accumulator(&self) -> f32 {
        self.accumulator
    }

    pub fn restore_accumulator(&mut self, value: f32) {
        self.accumulator = value.max(0.0);
    }
}
```

### Accumulation dynamics

With default parameters (complexity_weight = 0.6, threshold = 30.0):

| Scenario | Per-tick increment | Ticks to trigger |
|----------|-------------------|-----------------|
| Low load (context at 20%) | `0.4 + 0.6 * 0.2 = 0.52` | ~58 ticks |
| High load (context at 90%) | `0.4 + 0.6 * 0.9 = 0.94` | ~32 ticks |
| Maximum load (context at 100%) | `0.4 + 0.6 * 1.0 = 1.0` | 30 ticks |

The `min_ticks_between` guard (default 5) prevents a pathological case where the Golem enters a high-load loop: consolidate, immediately re-fill context, consolidate again. Five ticks of breathing room lets the Golem act on the newly freed context before considering another dream cycle.

### Dream cycle contents

When `needs_consolidation()` returns true, `DreamScheduler` schedules the following at the next delta tick:

1. **Episodic-to-semantic compression**: Recent episodes in the Grimoire are compressed from full traces into summary entries.
2. **Garbage collection**: Expired salience items, fully-decayed habituation records, and stale context segments are purged.
3. **Priority rebalancing**: AttentionSalience baselines are recalculated against the post-consolidation context state.
4. **Context I-frame**: A full compaction pass resets the delta compressor's base.

---

## 3. Habituation

The HabituationMask attenuates the response to repeated patterns. A price oscillation that has fired the same probe 20 times in a row should not continue escalating to T2. The Golem should habituate to it, reducing its effective salience.

### Theoretical grounding

Thompson and Spencer (1966) published the foundational parametric account of habituation, listing nine characteristics: it is stimulus-specific, recovers spontaneously with rest, and dishabituates on introduction of a novel stimulus. The `reset_pattern()` call on regime change implements dishabituation directly -- a new regime makes old predictions stale.

Groves and Thompson (1970) separated habituation (response decrement) from sensitization (response enhancement). High-urgency novel stimuli sensitize rather than habituate. `HabituationMask` applies only to the novelty term; the urgency term in AttentionSalience's scoring formula is immune, preserving sensitization to genuinely time-critical events.

Rankin et al. (2009) updated the Thompson-Spencer framework with constraints on spontaneous recovery time-course. The `forgetting_ticks` parameter implements this: patterns not seen for `forgetting_ticks` ticks decay toward zero exposure count, enabling full novelty recovery.

Rao and Ballard (1999) framed perception as prediction error minimization. A stimulus that matches a learned prediction generates low error (low novelty). `HabituationMask` is a scalar implementation of this idea: high exposure count equals good prediction equals low novelty score.

### StimulusPattern hash construction

`StimulusPattern` is a coarse hash over (StimulusKind, structural shape of payload): `blake3(kind_byte || payload_schema_version_byte || key_fields)[..16]`. Key fields depend on kind: for `PheromoneHit`, the emitting GolemId; for `PriceAnomaly`, the token address; for `ChainConfirmation`, the contract address. The hash is coarse by design -- we want to habituate to "daily price update from WETH", not "this exact price level."

### The mask (source implementation)

The source implementation uses a continuously decayed f32 exposure count rather than an integer counter. This provides smooth spontaneous recovery dynamics.

```rust
// crates/golem-core/src/habituation_mask.rs

use std::collections::HashMap;

/// Coarse hash of (StimulusKind, structural payload shape).
/// 16 bytes, truncated Blake3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StimulusPattern(pub [u8; 16]);

#[derive(Debug, Clone)]
struct ExposureRecord {
    /// Effective exposure count, decayed over time.
    count: f32,
    last_seen_tick: u64,
}

/// Per-pattern exposure tracker. Attenuates novelty scores for recurring stimuli.
pub struct HabituationMask {
    exposures: HashMap<StimulusPattern, ExposureRecord>,
    /// Exposures at which attenuation reaches 0.5. Lower = faster habituation.
    half_life: f32,
    /// Ticks without seeing a pattern before its count decays to zero.
    /// Controls spontaneous recovery. Typical: 1000-5000 ticks.
    forgetting_ticks: u64,
}

impl HabituationMask {
    pub fn new(half_life: f32, forgetting_ticks: u64) -> Self {
        HabituationMask {
            exposures: HashMap::new(),
            half_life,
            forgetting_ticks,
        }
    }

    /// Observe a stimulus occurrence at `tick`.
    ///
    /// Returns attenuation factor in (0.05, 1.0].
    /// Caller multiplies novelty by this value before computing salience.
    ///
    /// First observation returns 1.0 (fully novel).
    /// At `half_life` exposures, returns ~0.5.
    /// Floor at 0.05: even fully habituated stimuli retain a small signal,
    /// preventing total blindness to any event class.
    pub fn observe(&mut self, pattern: StimulusPattern, tick: u64) -> f32 {
        let forgetting = self.forgetting_ticks as f32;
        let record = self.exposures.entry(pattern).or_insert(ExposureRecord {
            count: 0.0,
            last_seen_tick: tick,
        });

        // Continuous decay formula for spontaneous recovery:
        // decay = exp(-ticks_since / forgetting)
        // record.count = record.count * decay + 1.0
        let ticks_since = tick.saturating_sub(record.last_seen_tick) as f32;
        let decay = (-ticks_since / forgetting).exp();
        record.count = record.count * decay + 1.0;
        record.last_seen_tick = tick;

        // Hyperbolic attenuation: half_life / (half_life + count - 1.0)
        // with 0.05 floor
        let attenuation = self.half_life / (self.half_life + record.count - 1.0);
        attenuation.max(0.05)
    }

    /// Reset exposure for a pattern (dishabituation).
    /// Called on regime-change events from CorticalState.
    /// After reset, the next observation returns full novelty (1.0).
    pub fn reset_pattern(&mut self, pattern: StimulusPattern) {
        self.exposures.remove(&pattern);
    }

    /// Reset all patterns. Called on full regime resets or agent restarts.
    pub fn reset_all(&mut self) {
        self.exposures.clear();
    }

    /// Evict fully-decayed patterns to bound memory growth.
    /// Call periodically (e.g., every 100 ticks in gamma cleanup phase).
    pub fn gc(&mut self, current_tick: u64) {
        let forgetting = self.forgetting_ticks as f32;
        self.exposures.retain(|_, record| {
            let ticks_since =
                current_tick.saturating_sub(record.last_seen_tick) as f32;
            let decayed_count = record.count * (-ticks_since / forgetting).exp();
            decayed_count >= 0.01
        });
    }
}
```

### Attenuation curve

With `half_life = 10.0`:

| Exposures | Attenuation | Effect |
|-----------|-------------|--------|
| 1 | 1.00 | Fully novel |
| 5 | 0.71 | Somewhat familiar |
| 10 | 0.53 | Near half-strength |
| 25 | 0.29 | Background noise |
| 50 | 0.17 | Nearly muted |
| 100 | 0.09 | At the floor |

The 0.05 floor ensures no event class becomes completely invisible. A pheromone broadcast seen 10,000 times still contributes 5% of its face novelty.

### Spontaneous recovery dynamics

The `forgetting_ticks` parameter controls how quickly an absent pattern recovers its novelty. With `forgetting_ticks = 2000`:

| Ticks since last seen | Retained count fraction |
|----------------------|------------------------|
| 200 | ~90% |
| 1000 | ~61% |
| 2000 | ~37% |
| 5000 | ~8% |

This matches Thompson and Spencer's observation that habituation recovery is gradual, not binary.

### Dishabituation

Regime changes trigger `reset_pattern()` for all currently queued items in `AttentionSalience`. This is dishabituation: a context change invalidates prior predictions, restoring full novelty. When the market shifts from Stable to Volatile, the hourly WETH price update that scored 0.05 goes back to 1.0. The Golem pays attention again because the old baseline no longer applies.

The connection runs through `AttentionSalience::on_regime_change()`, which iterates the queue and resets each item's pattern in the `SalienceBaseline`'s `HabituationMask`.

### Memory management

The `gc()` method evicts patterns whose decayed count has fallen below 0.01. Without periodic garbage collection, the exposure map grows unboundedly as the Golem encounters new patterns over weeks of operation. A reasonable schedule is once per 100 gamma ticks, which keeps the map to a few thousand entries in typical operation.

---

## 4. Homeostasis

The HomeostasisRegulator applies proportional control to CorticalState signals, nudging them toward stable operating ranges. It does not override signal values. It detects when a signal has deviated from its running average for too long and applies corrective nudges to AgentConfig.

### Theoretical grounding

Cannon (1932) originally formulated homeostasis: biological systems maintain internal variables near setpoints through feedback control. His insight was that stability is actively produced, not passively maintained. The `HomeostaticRule.deviation_threshold` captures this: small deviations are tolerated; persistent large ones trigger corrective action.

Sterling and Eyer (1988) extended homeostasis with allostasis: setpoints themselves shift with context. A Golem in a volatile regime should run different baselines than one in a stable regime. The `HomeostaticRule` evaluates deviations against a rolling average rather than a fixed setpoint -- the average *is* the setpoint, and it shifts as the Golem's baseline state evolves.

Barrett and Simmons (2015) describe interoceptive predictive coding: the brain maintains predictions about body states and issues corrections when prediction error exceeds a threshold. The `persistence_ticks` field captures this logic: a single deviant reading may be noise; N consecutive deviant readings are a genuine state change warranting correction.

Astrom and Hagglund (1995) formalize PID control theory. `HomeostasisRegulator` implements proportional control only -- no integral or derivative terms. Full PID would overcorrect: agent configuration knobs do not have the predictable response times of physical actuators.

### Signals and actuators

The regulator connects specific signals to specific corrective actions:

| Signal | Condition | Actuator |
|--------|-----------|----------|
| `economic_vitality` | Below rolling average | Tighten tool trust thresholds (lower risk tolerance) |
| `aggregate_accuracy` | Declining trend | Bias inference tier toward higher-capability models |
| `pleasure` | Chronically low | Set `DreamMode::Intensive` (more frequent consolidation) |

The regulator does NOT override deliberation. It adjusts `AgentConfig` knobs. An operator can still override; the regulator does not fight back. Think of it as a bias, not a constraint.

### Types (source implementation)

```rust
// crates/golem-core/src/homeostasis_regulator.rs

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignalId(pub &'static str);

pub const SIGNAL_ECONOMIC_VITALITY: SignalId  = SignalId("economic_vitality");
pub const SIGNAL_AGGREGATE_ACCURACY: SignalId = SignalId("aggregate_accuracy");
pub const SIGNAL_PLEASURE: SignalId           = SignalId("pleasure");
pub const SIGNAL_AROUSAL: SignalId            = SignalId("arousal");

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigKey(pub &'static str);

#[derive(Debug, Clone, Copy)]
pub enum ToolClass { DeFiWrite, DeFiRead, SystemOp }

#[derive(Debug, Clone, Copy)]
pub enum DreamMode { Standard, Intensive }

#[derive(Debug, Clone)]
pub enum HomeostaticAction {
    NudgeConfig { key: ConfigKey, delta: f32 },
    AdjustToolTrust { tool: ToolClass, delta: f32 },
    SetDreamMode(DreamMode),
}

impl HomeostaticAction {
    /// Apply the action. `direction` is +1.0 if signal is above baseline,
    /// -1.0 if below. Corrective actions apply in the opposite direction.
    pub fn apply(&self, config: &mut AgentConfig, direction: f32) {
        match self {
            HomeostaticAction::NudgeConfig { key, delta } => {
                config.nudge(key, -direction * delta);
            }
            HomeostaticAction::AdjustToolTrust { tool, delta } => {
                config.adjust_tool_trust(*tool, -direction * delta);
            }
            HomeostaticAction::SetDreamMode(mode) => {
                config.set_dream_mode(*mode);
            }
        }
    }
}
```

### Rules, rolling averages, and the regulator

```rust
pub struct HomeostaticRule {
    pub signal: SignalId,
    pub deviation_threshold: f32,
    /// Consecutive ticks the deviation must persist before applying.
    pub persistence_ticks: u32,
    pub action: HomeostaticAction,
    consecutive_ticks: u32,
}

struct RollingAverage {
    alpha: f32,   // EMA smoothing factor
    value: f32,
    initialized: bool,
}

impl RollingAverage {
    fn new(alpha: f32) -> Self {
        RollingAverage { alpha, value: 0.0, initialized: false }
    }

    fn update(&mut self, sample: f32) -> f32 {
        if !self.initialized {
            self.value = sample;
            self.initialized = true;
        } else {
            self.value = self.alpha * sample
                + (1.0 - self.alpha) * self.value;
        }
        self.value
    }
}

pub struct HomeostasisRegulator {
    windows: HashMap<SignalId, RollingAverage>,
    rules: Vec<HomeostaticRule>,
}

impl HomeostasisRegulator {
    pub fn new() -> Self {
        HomeostasisRegulator {
            windows: HashMap::new(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: HomeostaticRule) {
        self.windows
            .entry(rule.signal.clone())
            .or_insert_with(|| RollingAverage::new(0.05));
        self.rules.push(rule);
    }

    /// Call every theta tick after Step 7 (Reflect).
    /// Applies soft nudges to AgentConfig when persistent deviations
    /// are detected.
    pub fn tick(
        &mut self,
        state: &dyn CorticalStateReader,
        config: &mut AgentConfig,
    ) {
        for rule in &mut self.rules {
            let current = state.read_signal(&rule.signal);
            let avg = self.windows
                .get_mut(&rule.signal)
                .map(|w| w.update(current))
                .unwrap_or(current);

            let deviation = current - avg;
            if deviation.abs() >= rule.deviation_threshold {
                rule.consecutive_ticks += 1;
            } else {
                rule.consecutive_ticks = 0;
            }

            if rule.consecutive_ticks >= rule.persistence_ticks {
                rule.action.apply(config, deviation.signum());
            }
        }
    }
}

pub trait CorticalStateReader {
    fn read_signal(&self, id: &SignalId) -> f32;
    fn regime(&self) -> MarketRegime;
}

#[derive(Debug, Clone, Copy)]
pub enum MarketRegime { Stable, Trending, Volatile, Crisis }
```

### Example rule configuration

Three concrete rules covering financial health, predictive quality, and emotional baseline:

```rust
let mut regulator = HomeostasisRegulator::new();

// When economic vitality drops persistently, tighten DeFi write trust.
regulator.add_rule(HomeostaticRule {
    signal: SIGNAL_ECONOMIC_VITALITY,
    deviation_threshold: 0.15,
    persistence_ticks: 10,
    action: HomeostaticAction::AdjustToolTrust {
        tool: ToolClass::DeFiWrite,
        delta: 0.1,
    },
    consecutive_ticks: 0,
});

// When accuracy declines persistently, nudge inference toward T2.
regulator.add_rule(HomeostaticRule {
    signal: SIGNAL_AGGREGATE_ACCURACY,
    deviation_threshold: 0.10,
    persistence_ticks: 15,
    action: HomeostaticAction::NudgeConfig {
        key: ConfigKey("inference_tier_bias"),
        delta: 0.2,
    },
    consecutive_ticks: 0,
});

// When pleasure stays low, schedule more dream consolidation.
regulator.add_rule(HomeostaticRule {
    signal: SIGNAL_PLEASURE,
    deviation_threshold: 0.20,
    persistence_ticks: 20,
    action: HomeostaticAction::SetDreamMode(DreamMode::Intensive),
    consecutive_ticks: 0,
});
```

### Why proportional control only

Full PID control requires tuning three gains (K_p, K_i, K_d) against a system with known response characteristics. Agent configuration knobs do not have predictable response curves. The integral term (accumulated error correction) would cause the regulator to keep nudging a knob harder the longer the deviation persists, even if the nudge has already maxed out the knob's effective range. The derivative term (rate-of-change dampening) would require smooth signal trajectories, which CorticalState signals do not provide. Proportional control alone is stable and predictable: the correction is directly proportional to the deviation, and it stops when the deviation stops. No windup, no overshoot.

---

## 5. Compensation and Rollback

Multi-step DeFi actions (approve -> swap -> provide liquidity) need saga-pattern compensation. If step 3 fails, steps 1 and 2 may need rollback. This section covers two complementary primitives: `CompensationChain` (the saga pattern for semantic rollback of multi-step operations) and `RollbackCheckpoint` (explicit state save points for full-state restoration).

### Theoretical grounding

Garcia-Molina and Salem (1987) defined the Saga pattern: a long-lived transaction is decomposed into a sequence of short-lived sub-transactions, each paired with a compensating transaction. If the saga fails at step N, compensations execute in reverse order from step N-1 to step 1. Compensating transactions must be idempotent and semantically reversible, not technically undoable.

Gray and Reuter (1992) distinguished compensation from rollback. Rollback undoes side effects at the storage level. Compensation produces forward-correcting state equivalent to never having started. On-chain transactions cannot be undone, only compensated.

Richardson (2018) treats sagas in the microservices context, establishing choreography vs. orchestration variants. `CompensationChain` uses orchestration: the chain itself is the coordinator.

### Part 1: CompensationChain (source implementation)

```rust
// crates/golem-runtime/src/compensation_chain.rs

use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub u64);

pub struct CompensationContext {
    pub chain_client: std::sync::Arc<dyn ChainClient>,
    pub current_nonce: u64,
    pub wallet: WalletAddress,
}

pub trait ChainClient: Send + Sync {
    fn execute_compensation(
        &self,
        ctx: &CompensationContext,
        idempotency_key: [u8; 32],
        payload: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), CompensationError>> + Send + '_>>;
}

#[derive(Debug)]
pub struct WalletAddress(pub [u8; 20]);

#[derive(Debug)]
pub enum CompensationError {
    ChainError(String),
    IdempotencyConflict,
    ContextUnavailable,
}

/// Compensation action. MUST be idempotent -- safe to call multiple times.
pub trait CompensationAction: Send + Sync {
    fn compensate<'a>(
        &'a self,
        ctx: &'a CompensationContext,
    ) -> Pin<Box<dyn Future<Output = Result<(), CompensationError>> + Send + 'a>>;
}

pub struct CompensationStep {
    pub step_name: &'static str,
    pub action: Box<dyn CompensationAction>,
    /// blake3(workflow_id_bytes || step_name_bytes).
    /// Guards against double-compensation after crash recovery.
    pub idempotency_key: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ChainState {
    Active,
    Completed,
    Compensating,
    Compensated,
    CompensationFailed,
}

pub struct CompensationChain {
    id: WorkflowId,
    compensations: Vec<CompensationStep>,
    pub state: ChainState,
}

fn derive_idempotency_key(
    workflow_id: WorkflowId, step_name: &'static str,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&workflow_id.0.to_le_bytes());
    hasher.update(step_name.as_bytes());
    hasher.finalize().into()
}

impl CompensationChain {
    pub fn new(id: WorkflowId) -> Self {
        CompensationChain {
            id,
            compensations: Vec::new(),
            state: ChainState::Active,
        }
    }

    /// Register a compensation for a completed step.
    /// MUST be called immediately after each step succeeds,
    /// before the next step begins.
    pub fn register<A: CompensationAction + 'static>(
        &mut self, name: &'static str, action: A,
    ) {
        debug_assert!(
            self.state == ChainState::Active,
            "registering compensation on non-Active chain"
        );
        self.compensations.push(CompensationStep {
            step_name: name,
            action: Box::new(action),
            idempotency_key: derive_idempotency_key(self.id, name),
        });
    }

    pub fn complete(&mut self) {
        self.state = ChainState::Completed;
    }

    /// Execute compensations in reverse registration order.
    /// Continues past individual step failures; records each error.
    pub async fn rollback(
        &mut self, ctx: &CompensationContext,
    ) -> ChainState {
        self.state = ChainState::Compensating;
        let mut all_ok = true;
        for step in self.compensations.iter().rev() {
            match step.action.compensate(ctx).await {
                Ok(()) => {
                    tracing::info!(
                        step = step.step_name,
                        "compensation step succeeded"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        step = step.step_name,
                        error = ?e,
                        "compensation step failed"
                    );
                    all_ok = false;
                }
            }
        }
        self.state = if all_ok {
            ChainState::Compensated
        } else {
            ChainState::CompensationFailed
        };
        self.state
    }
}
```

### Idempotency

The `idempotency_key` is derived deterministically from the workflow ID and step name: `blake3(workflow_id_bytes || step_name_bytes)[..32]`. If the agent crashes during compensation and restarts, re-executing the same compensation with the same key produces no additional on-chain effect. The `ChainClient` implementation is responsible for checking the key before submitting a transaction.

### Compensation failure semantics

When a compensation step fails, the chain continues executing remaining compensations in reverse order rather than stopping. A failed compensation at step 3 should not prevent the compensations at steps 2 and 1 from running -- those positions are independent. The final state `CompensationFailed` signals to the operator that manual intervention is needed for the specific failed step.

### Part 2: RollbackCheckpoint (source implementation)

```rust
// crates/golem-runtime/src/rollback_checkpoint.rs

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorticalSnapshot {
    pub pleasure: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub regime: u8,
    pub economic_vitality: f32,
    pub composite_vitality: f32,
    pub inference_budget_remaining: f32,
    pub aggregate_accuracy: f32,
    pub active_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextSnapshot {
    pub tokens_used: u32,
    pub context_budget: u32,
    pub messages_cbor: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PositionLedgerSnapshot {
    pub positions_cbor: Vec<u8>,
    pub snapshot_tick: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrimoireRef {
    pub content_hash: [u8; 32],
    pub entry_count: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RollbackCheckpoint {
    pub name: &'static str,
    pub created_at_unix_secs: u64,
    pub tick: u64,
    pub content_hash: [u8; 32],
    pub cortical: CorticalSnapshot,
    pub context: ContextSnapshot,
    pub ledger: PositionLedgerSnapshot,
    pub grimoire_ref: GrimoireRef,
}

impl RollbackCheckpoint {
    pub fn new(
        name: &'static str,
        tick: u64,
        cortical: CorticalSnapshot,
        context: ContextSnapshot,
        ledger: PositionLedgerSnapshot,
        grimoire_ref: GrimoireRef,
    ) -> Self {
        let payload = (&cortical, &context, &ledger, &grimoire_ref, tick);
        let bytes = ciborium_ser(&payload);
        let content_hash: [u8; 32] = blake3::hash(&bytes).into();

        let created_at_unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        RollbackCheckpoint {
            name, created_at_unix_secs, tick, content_hash,
            cortical, context, ledger, grimoire_ref,
        }
    }

    /// Verify stored hash matches payload. Call before restoring.
    pub fn verify_integrity(&self) -> bool {
        let payload = (
            &self.cortical, &self.context, &self.ledger,
            &self.grimoire_ref, self.tick,
        );
        let bytes = ciborium_ser(&payload);
        let computed: [u8; 32] = blake3::hash(&bytes).into();
        computed == self.content_hash
    }
}

pub struct CheckpointRegistry {
    checkpoints: HashMap<&'static str, RollbackCheckpoint>,
    insertion_order: Vec<&'static str>,
    max_retained: usize,
}

impl CheckpointRegistry {
    pub fn new(max_retained: usize) -> Self {
        CheckpointRegistry {
            checkpoints: HashMap::new(),
            insertion_order: Vec::new(),
            max_retained,
        }
    }

    pub fn save(&mut self, checkpoint: RollbackCheckpoint) {
        let name = checkpoint.name;
        if !self.checkpoints.contains_key(name) {
            self.insertion_order.push(name);
        }
        self.checkpoints.insert(name, checkpoint);
        self.evict_oldest_if_needed();
    }

    pub fn restore(
        &self, name: &'static str,
    ) -> Option<&RollbackCheckpoint> {
        self.checkpoints.get(name)
    }

    pub fn remove(
        &mut self, name: &'static str,
    ) -> Option<RollbackCheckpoint> {
        self.insertion_order.retain(|&n| n != name);
        self.checkpoints.remove(name)
    }

    fn evict_oldest_if_needed(&mut self) {
        while self.checkpoints.len() > self.max_retained {
            if let Some(oldest) = self.insertion_order.first().copied() {
                self.insertion_order.remove(0);
                self.checkpoints.remove(oldest);
            } else {
                break;
            }
        }
    }
}
```

### Difference from StateSnapshot

`RollbackCheckpoint` (this section) is named and intentionally placed at semantically meaningful moments: `"pre_defi_borrow"`, `"post_dream_consolidation"`. `StateSnapshot` (see `03c-state-management.md` Section 1) is periodic and content-addressed. Both use CBOR + Blake3, but with different retention and lookup semantics. Checkpoints are for recovery. Snapshots are for forensics.

### 8-step composition workflow

A typical DeFi workflow uses both primitives:

1. Save a `RollbackCheckpoint` named `"pre_workflow"`.
2. Create a `CompensationChain` with the workflow ID.
3. Execute step 1 (borrow). On success, register the compensation (repay borrow).
4. Execute step 2 (swap). On success, register the compensation (reverse swap).
5. Execute step 3 (deposit). On success, mark the chain `Completed`.
6. If step 3 fails: call `chain.rollback()` to execute compensations in reverse.
7. If rollback succeeds: state is semantically equivalent to pre-workflow.
8. If rollback fails (a compensation step failed): restore the `"pre_workflow"` checkpoint to get clean state, then alert the operator about the failed compensation.

The checkpoint is the backstop. The compensation chain handles normal failure paths. The checkpoint handles the case where compensation itself fails.

---

## 6. Event-Driven Wakeup

The adaptive clock normally fires on a timer. Event-driven wakeup allows external conditions to interrupt the normal cadence and force an immediate gamma or theta tick.

### Theoretical grounding

Schmidt (1995) formalized the Reactor pattern: a single event demultiplexer waits for events across multiple handles and dispatches to registered handlers. `EventDrivenWakeup` implements Reactor over Bardo's `tokio::broadcast` EventFabric.

The Observer pattern (Gamma et al., 1995) describes objects subscribing to events and receiving notifications when they fire. `WakeupCondition` is an Observer; `EventDrivenWakeup::on_event()` is the notification path.

Lampson and Redell (1980) described condition variables as the substrate for interrupt-driven synchronization in Mesa. `drain_pending()` is the condition-variable poll that `AdaptiveClock` calls between scheduled ticks.

The Reactive Manifesto (Boner et al., 2014) argues that message-driven systems should respond to events rather than blocking on resources. The wakeup mechanism is the reactive entry point into an otherwise scheduled system.

### Event and resolution types (source implementation)

```rust
// crates/golem-runtime/src/event_driven_wakeup.rs

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TriggerId(pub u32);

#[derive(Debug, Clone, Copy)]
pub enum WakeupResolution {
    /// Update CorticalState and run attention; skip full deliberation.
    GammaPass,
    /// Full cognitive cycle including deliberation and potential action.
    ThetaPass,
}

#[derive(Debug, Clone)]
pub struct WakeupEvent {
    pub trigger_id: TriggerId,
    pub resolution: WakeupResolution,
    pub fired_at: Instant,
}

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    PolicyTripwire { cage_id: u32 },
    OwnerMessage { size_tokens: u32 },
    ChainConfirmation { tx_hash: [u8; 32] },
    PheromoneSpike { intensity: f32 },
    DreamRequest,
    Shutdown,
}
```

### Condition trait and triggers

```rust
pub trait WakeupCondition: Send + Sync {
    fn evaluate(&self, event: &RuntimeEvent) -> bool;
}

pub struct WakeupTrigger {
    pub id: TriggerId,
    pub condition: Box<dyn WakeupCondition>,
    pub resolution: WakeupResolution,
    /// Guard against trigger storms. Per-trigger, not global.
    pub min_refire_interval: Duration,
    last_fired: Option<Instant>,
}
```

### The wakeup manager

```rust
pub struct EventDrivenWakeup {
    triggers: Vec<WakeupTrigger>,
    pending: Arc<Mutex<VecDeque<WakeupEvent>>>,
}

impl EventDrivenWakeup {
    pub fn new() -> Self {
        EventDrivenWakeup {
            triggers: Vec::new(),
            pending: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn register(&mut self, trigger: WakeupTrigger) {
        self.triggers.push(trigger);
    }

    /// Called by EventFabric subscriber for every incoming RuntimeEvent.
    pub fn on_event(&mut self, event: &RuntimeEvent) {
        let now = Instant::now();
        for trigger in &mut self.triggers {
            if !trigger.condition.evaluate(event) {
                continue;
            }
            let can_fire = trigger
                .last_fired
                .map(|t| now.duration_since(t) >= trigger.min_refire_interval)
                .unwrap_or(true);
            if can_fire {
                trigger.last_fired = Some(now);
                self.pending.lock().unwrap().push_back(WakeupEvent {
                    trigger_id: trigger.id,
                    resolution: trigger.resolution,
                    fired_at: now,
                });
            }
        }
    }

    /// AdaptiveClock calls this between scheduled tick sleeps.
    pub fn drain_pending(&self) -> Vec<WakeupEvent> {
        self.pending.lock().unwrap().drain(..).collect()
    }

    pub fn pending_handle(&self) -> Arc<Mutex<VecDeque<WakeupEvent>>> {
        Arc::clone(&self.pending)
    }
}
```

### Built-in conditions

```rust
pub struct OwnerMessageCondition {
    pub min_tokens: u32,
}

impl WakeupCondition for OwnerMessageCondition {
    fn evaluate(&self, event: &RuntimeEvent) -> bool {
        matches!(
            event,
            RuntimeEvent::OwnerMessage { size_tokens }
                if *size_tokens >= self.min_tokens
        )
    }
}

pub struct PolicyTripwireCondition;

impl WakeupCondition for PolicyTripwireCondition {
    fn evaluate(&self, event: &RuntimeEvent) -> bool {
        matches!(event, RuntimeEvent::PolicyTripwire { .. })
    }
}
```

### Deduplication with scheduled ticks

`AdaptiveClock` suppresses double-firing when a scheduled tick is due within a short window of a wakeup event. If a theta tick is scheduled to fire in less than 10 seconds and a `GammaPass` wakeup arrives, the wakeup is suppressed. If a `ThetaPass` wakeup arrives and the next scheduled theta tick is more than 10 seconds away, the wakeup fires immediately. The 10-second window is tunable in `AdaptiveClock`'s configuration.

### Per-trigger refire intervals

The `min_refire_interval` is per-trigger, not global. A `PolicyTripwire` trigger with a 5-second refire interval and a `PheromoneSpike` trigger with a 60-second refire interval operate independently. Without per-trigger intervals, a volatile market generating rapid-fire `PheromoneSpike` events could trigger a ThetaPass wakeup every few seconds, burning through the daily inference budget in minutes.

### AdaptiveClock integration

```rust
// Pseudocode for AdaptiveClock::run()
loop {
    let (scheduled_kind, sleep_duration) = self.next_tick();

    // Check for wakeups before sleeping the full interval
    let wakeups = self.wakeup.drain_pending();
    if !wakeups.is_empty() {
        // Process the highest-resolution wakeup
        let resolution = wakeups.iter()
            .map(|w| w.resolution)
            .max_by_key(|r| match r {
                WakeupResolution::ThetaPass => 1,
                WakeupResolution::GammaPass => 0,
            });
        if let Some(res) = resolution {
            self.fire_tick(res.into());
            continue;
        }
    }

    // No wakeups: sleep for the scheduled duration
    tokio::time::sleep(sleep_duration).await;
    self.fire_tick(scheduled_kind);
}
```

### Example wakeup conditions

| Condition | Fires when | Resolution |
|-----------|-----------|------------|
| Large price movement | > 5% in 30 seconds | ThetaPass |
| Position liquidation risk | Health factor < 1.2 | ThetaPass |
| Governance proposal ending | < 10 minutes to deadline | GammaPass |
| Gas price spike | > 3x recent average | GammaPass |
| Styx urgent message | Clade alert received | ThetaPass |

---

## 7. Cross-Primitive Relationships

The six mechanisms in this document form a tightly coupled subsystem. Each primitive reads from or writes to the others:

- **AttentionSalience + HabituationMask**: HabituationMask feeds the novelty term in the salience scoring formula. Without it, recurring background events score as fully novel every tick, flooding the queue with structurally predictable noise.
- **AttentionSalience + AdaptiveClock**: The AdaptiveClock (see `13-runtime-extensions.md`) determines *when* to think. AttentionSalience determines *what* to think about. They are complementary.
- **HomeostasisRegulator + SleepPressure**: When `SetDreamMode(Intensive)` fires, it lowers the SleepPressure threshold, causing consolidation to trigger more frequently.
- **HomeostasisRegulator + AttentionSalience**: The regulator does not directly modify attention, but inference tier bias changes affect how many context items the model can process effectively during deliberation.
- **HomeostasisRegulator + CorticalState**: The regulator reads but never writes CorticalState signals; it writes only to AgentConfig. CorticalState (see `18-cortical-state.md`) is the input source.
- **SleepPressure + AdaptiveClock**: The clock determines when delta ticks fire; SleepPressure determines whether consolidation is warranted at each delta tick. The clock is the schedule; SleepPressure is the readiness signal.
- **SleepPressure + MetricsEmitter**: `MetricsEmitter` (see `03c-state-management.md`) includes `sleep_pressure` in each `TickMetrics` record, making the accumulation curve visible in operational dashboards.
- **CompensationChain + RollbackCheckpoint**: Compensation handles normal failure paths (semantic rollback). The checkpoint handles the case where compensation itself fails (full state restoration).
- **EventDrivenWakeup + AdaptiveClock**: Wakeups preempt the normal tick schedule. The AdaptiveClock deduplicates wakeups against pending scheduled ticks within a 10-second window.

---

## References

### Attention salience
- [CHERRY-1953] Cherry, E. C. (1953). "Some experiments on the recognition of speech, with one and two ears." *Journal of the Acoustical Society of America*, 25(5), 975-979. — *Established that selective attention operates on structural features before semantics; the basis for salience scoring by kind and shape before LLM analysis.*
- [BAARS-1988] Baars, B. J. (1988). *A Cognitive Theory of Consciousness*. Cambridge University Press. — *Global Workspace Theory: competing processors vie for a central broadcast resource. AttentionSalience is the competition mechanism; the theta tick's context is the broadcast.*
- [ITTI-2001] Itti, L., & Koch, C. (2001). "Computational modelling of visual attention." *Nature Reviews Neuroscience*, 2(3), 194-203. — *Computational saliency maps with center-surround inhibition; informs the bottom-up salience component of the scoring formula.*
- [CORBETTA-2002] Corbetta, M., & Shulman, G. L. (2002). "Control of goal-directed and stimulus-driven attention in the brain." *Nature Reviews Neuroscience*, 3(3), 201-215. — *Distinguishes dorsal (goal-directed) from ventral (stimulus-driven) attention; maps to the top-down vs. bottom-up salience components.*

### Sleep consolidation
- [BORBELY-1982] Borbely, A. A. (1982). "A two process model of sleep regulation." *Human Neurobiology*, 1(3), 195-204. — *The two-process model (circadian + homeostatic sleep pressure); the biological basis for the SleepPressure accumulator.*
- [PORKKA-HEISKANEN-1997] Porkka-Heiskanen, T., et al. (1997). "Adenosine: A mediator of the sleep-inducing effects of prolonged wakefulness." *Science*, 276(5316), 1265-1268. — *Identifies adenosine as the molecular signal for sleep pressure; the biological analogue of the complexity accumulator.*
- [TONONI-2006] Tononi, G., & Cirelli, C. (2006). "Sleep function and synaptic homeostasis." *Sleep*, 29(2), 145-165. — *Synaptic homeostasis hypothesis: sleep downscales synaptic weights to restore signal-to-noise ratio. Informs the dream consolidation's pruning phase.*
- [STICKGOLD-2005] Stickgold, R. (2005). "Sleep-dependent memory consolidation." *Nature*, 437(7063), 1272-1278. — *Reviews evidence that sleep actively consolidates memories; supports the dream cycle's three-phase consolidation pipeline.*

### Habituation
- [THOMPSON-1966] Thompson, R. F., & Spencer, W. A. (1966). "Habituation: A model phenomenon for the study of neuronal substrates of behavior." *Psychological Review*, 73(1), 16-43. — *Defines the nine parametric characteristics of habituation; the behavioral spec for the HabituationMask's attenuation rules.*
- [GROVES-1970] Groves, P. M., & Thompson, R. F. (1970). "Habituation: A dual-process theory." *Psychological Review*, 77(5), 419-450. — *Dual-process theory of habituation: habituation (decremental) and sensitization (incremental) interact; supports the dishabituation mechanic.*
- [RANKIN-2009] Rankin, C. H., et al. (2009). "Habituation revisited: An updated and revised description of the behavioral characteristics of habituation." *Neurobiology of Learning and Memory*, 92(2), 135-138. — *Updated consensus on habituation characteristics across species; validates the exposure-count-based attenuation model.*
- [RAO-1999] Rao, R. P. N., & Ballard, D. H. (1999). "Predictive coding in the visual cortex." *Nature Neuroscience*, 2(1), 79-87. — *Predictive coding: the brain suppresses predicted stimuli and amplifies prediction errors. The computational principle behind habituation as prediction-match suppression.*

### Homeostasis
- [CANNON-1932] Cannon, W. B. (1932). *The Wisdom of the Body*. W. W. Norton. — *Coined "homeostasis": maintenance of internal stability through negative feedback. The direct inspiration for the HomeostasisRegulator's proportional control.*
- [STERLING-1988] Sterling, P., & Eyer, J. (1988). "Allostasis: A new paradigm to explain arousal pathology." In *Handbook of Life Stress, Cognition and Health*. Wiley. — *Extends homeostasis to allostasis: predictive regulation that adjusts setpoints in anticipation of demand. Informs the adaptive setpoint mechanism.*
- [BARRETT-2015] Barrett, L. F., & Simmons, W. K. (2015). "Interoceptive predictions in the brain." *Nature Reviews Neuroscience*, 16(7), 419-429. — *The brain's predictive model of internal body states; supports homeostatic regulation through prediction rather than just reactive correction.*
- [ASTROM-1995] Astrom, K. J., & Hagglund, T. (1995). *PID Controllers: Theory, Design, and Tuning*. ISA. — *The canonical PID control reference; the engineering basis for the HomeostasisRegulator's proportional correction implementation.*

### Compensation and rollback
- [GARCIA-MOLINA-1987] Garcia-Molina, H., & Salem, K. (1987). "Sagas." *ACM SIGMOD Record*, 16(3), 249-259. — *Introduces the saga pattern: long-lived transactions with compensating actions. The direct basis for the CompensationChain's rollback mechanism.*
- [GRAY-1992] Gray, J., & Reuter, A. (1992). *Transaction Processing: Concepts and Techniques*. Morgan Kaufmann. — *Canonical reference on ACID transactions and recovery; informs the checkpoint-based rollback design.*
- [MOHAN-1992] Mohan, C., et al. (1992). "ARIES: A transaction recovery method." *ACM Transactions on Database Systems*, 17(1), 94-162. — *Write-ahead logging and physiological redo/undo; informs the compensation chain's logging strategy.*
- [ELNOZAHY-2002] Elnozahy, E. N., et al. (2002). "A survey of rollback-recovery protocols in message-passing systems." *ACM Computing Surveys*, 34(3), 375-408. — *Comprehensive survey of rollback-recovery; validates the coordinated checkpoint approach.*
- [RICHARDSON-2018] Richardson, C. (2018). *Microservices Patterns*. Manning. — *Saga orchestration in microservices; the practical engineering pattern behind multi-step DeFi action rollback.*
- [HOHPE-2003] Hohpe, G., & Woolf, B. (2003). *Enterprise Integration Patterns*. Addison-Wesley. — *Compensating transaction patterns; informs the design of protocol-specific undo operations.*

### Event-driven wakeup
- [SCHMIDT-1995] Schmidt, D. C. (1995). "Reactor: An object behavioral pattern for demultiplexing and dispatching handles for synchronous events." In *Pattern Languages of Program Design*. Addison-Wesley. — *The Reactor pattern: event demultiplexing and dispatch. The architectural pattern behind EventDrivenWakeup's condition evaluation loop.*
- [GAMMA-1995] Gamma, E., Helm, R., Johnson, R., & Vlissides, J. (1995). *Design Patterns*. Addison-Wesley. — *Observer pattern: objects subscribe to events and are notified on state change. Foundational for the Event Fabric subscription model.*
- [LAMPSON-1980] Lampson, B. W., & Redell, D. D. (1980). "Experience with processes and monitors in Mesa." *Communications of the ACM*, 23(2), 105-117. — *Condition variables and monitors; informs the condition-based clock interrupt mechanism.*
- [REACTIVE-MANIFESTO-2014] Boner, J., et al. (2014). *The Reactive Manifesto*. reactivemanifesto.org. — *Principles for responsive, resilient, elastic, message-driven systems; validates the event-driven architecture choice.*

---
