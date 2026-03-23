# 18 -- CorticalState and the Daimon: Runtime Affect [SPEC]

**Shared-Memory Perception Surface, ALMA Affect Engine, and TUI Visual Mapping**

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-core` (cortical_state.rs), `golem-daimon` (appraisal.rs, mood.rs, somatic_markers.rs)
>
> Cross-references: [01-cognition.md](./01-cognition.md), [02-heartbeat.md](./02-heartbeat.md) (adaptive clock), [17-prediction-engine.md](./17-prediction-engine.md) (Oracle), [13-runtime-extensions.md](./13-runtime-extensions.md)
>
> Sources: `active-inference/01-runtime-core` (CorticalState section), `mmo2/23-cortical-state-and-affect`

> **Reader orientation:** This is the canonical reference for the CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model) and the Daimon (the affect engine implementing PAD -- Pleasure-Arousal-Dominance -- emotional state as a control signal). It belongs to the `01-golem` cognition layer, spanning the `golem-core` and `golem-daimon` crates. The key concept: CorticalState is a lock-free ~256-byte struct where any subsystem can read any signal with a single atomic load -- no locks, no waiting. The Daimon writes affect signals; the Oracle writes prediction signals; the mortality engine writes vitality signals. The TUI reads them all at 60fps. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## What This Document Covers

This is the canonical reference for the CorticalState struct, the Daimon affect engine, and the mapping between CorticalState signals and TUI interpolating variables. It reconciles three sources: the 26 interpolating variables from the TUI spec, the ~32 atomic signals from the active inference runtime core, and the Daimon's ALMA affect model.

The full CorticalState struct (32 atomic signals, cache-line aligned), how those signals map to TUI interpolating variables (26 original + 6 new), the adaptive clock (gamma/theta/delta), the Daimon's affect engine (PAD computation, ALMA layers, somatic markers, Plutchik labels, affect-modulated retrieval), and owner interaction with affect.

---

## CorticalState

### The problem it solves

A Golem has many subsystems that need to read each other's current state. The Daimon needs prediction accuracy to compute affect. The mortality engine needs accuracy trends to update the epistemic death clock. The TUI needs everything -- affect, prediction, mortality, attention -- to render the creature. The action gate needs accuracy and affect to decide whether to permit trades.

The alternative is function calls through the extension system (waiting for hooks to fire), or mutex-guarded reads, or waiting for the next tick. All of those add latency to reads that should be free.

CorticalState solves this with shared-memory atomics. Any fiber can read any signal at any time with a single atomic load. No locks, no waiting, no contention.

### The struct

```rust
use std::sync::atomic::{AtomicU32, AtomicU16, AtomicU8, AtomicI8, Ordering};

/// Zero-latency shared perception surface.
///
/// Every subsystem writes its own signals; every subsystem reads everyone
/// else's. ~256 bytes total. Fits in 4 cache lines. Cache-line aligned
/// to avoid false sharing between signal groups.
///
/// Convention: f32 values stored via f32::to_bits() / f32::from_bits()
/// because Rust stable has no floating-point atomics.
#[repr(C, align(64))]
pub struct CorticalState {
    // ═══ AFFECT — written by Daimon ═══
    pub(crate) pleasure: AtomicU32,        // f32 [-1.0, 1.0] PAD pleasure
    pub(crate) arousal: AtomicU32,         // f32 [-1.0, 1.0] PAD arousal
    pub(crate) dominance: AtomicU32,       // f32 [-1.0, 1.0] PAD dominance
    pub(crate) primary_emotion: AtomicU8,  // Plutchik label (0-7)

    // ═══ PREDICTION — written by Oracle ═══
    pub(crate) aggregate_accuracy: AtomicU32,       // f32 [0.0, 1.0]
    pub(crate) accuracy_trend: AtomicI8,            // -1, 0, +1
    pub(crate) category_accuracies: [AtomicU32; 16], // f32 per category
    pub(crate) surprise_rate: AtomicU32,            // f32 [0.0, 1.0]

    // ═══ ATTENTION — written by Oracle/AttentionForager ═══
    pub(crate) universe_size: AtomicU32,     // total tracked items
    pub(crate) active_count: AtomicU16,      // ACTIVE tier items
    pub(crate) pending_predictions: AtomicU32, // awaiting resolution

    // ═══ CREATIVE — written by Dream engine ═══
    pub(crate) creative_mode: AtomicU8,               // bool as 0/1
    pub(crate) fragments_captured: AtomicU32,          // dream fragments
    pub(crate) last_novel_prediction_tick: AtomicU32,  // lower 32 bits
    pub(crate) last_novel_prediction_tick_hi: AtomicU32, // upper 32 bits

    // ═══ ENVIRONMENT — written by domain probes ═══
    pub(crate) regime: AtomicU8,     // 0=calm, 1=trending, 2=volatile, 3=crisis
    pub(crate) gas_gwei: AtomicU32,  // f32

    // ═══ MORTALITY — written by mortality engine ═══
    pub(crate) economic_vitality: AtomicU32,   // f32 [0.0, 1.0]
    pub(crate) epistemic_vitality: AtomicU32,  // f32 [0.0, 1.0]
    pub(crate) stochastic_vitality: AtomicU32, // f32 [0.0, 1.0]
    pub(crate) behavioral_phase: AtomicU8,     // 0-4 (Thriving..Terminal)

    // ═══ DERIVED — written by runtime per-tick ═══
    pub(crate) compounding_momentum: AtomicU32, // f32 [0.0, 1.0] glacial
}
```

That is 32 atomic signals. 4 affect, 20 prediction-related (aggregate + trend + 16 categories + surprise_rate + pending_predictions), 2 attention (universe_size + active_count), 4 creative, 2 environment, 4 mortality, 1 derived. The `category_accuracies` array counts as 16 individual signals but occupies one logical slot in the struct layout.

### Size verification

The core fields sum to 27 AtomicU32 (108 bytes) + 1 [AtomicU32; 16] (64 bytes) + 1 AtomicU16 (2 bytes) + 3 AtomicU8 (3 bytes) + 1 AtomicI8 (1 byte) = 178 bytes of payload. With `#[repr(C, align(64))]`, the struct is padded to the next 64-byte boundary = 192 bytes. The "~256 bytes" in the doc comment is a conservative upper bound that accounts for future fields and platform-specific alignment overhead. Actual `std::mem::size_of::<CorticalState>()` on x86-64 and aarch64 will be 192 bytes.

### Design properties

**No locks.** Writes use `Ordering::Release`, reads use `Ordering::Acquire`. This ensures that when a reader observes a new value, all preceding writes by that writer are also visible -- preventing a stale `pleasure` from pairing with a fresh `accuracy_trend` within the same signal group. A snapshot where `pleasure` is from tick N and `accuracy` is from tick N+1 is still acceptable across groups. The TUI's 60fps render loop interpolates toward targets anyway, smoothing any micro-inconsistency across frames.

**Clear ownership.** Each signal group has exactly one writer:

| Signal group | Writer | Frequency |
|-------------|--------|-----------|
| Affect | Daimon | Every prediction resolution (gamma) |
| Prediction | Oracle | Every prediction resolution (gamma) |
| Attention | Oracle / AttentionForager | Per gamma tick |
| Creative | Dream engine | Per dream cycle + on novel predictions |
| Environment | Domain probes | Per gamma tick |
| Mortality | Mortality engine | Per gamma tick |
| Derived | Runtime | Per delta tick |

No signal has two writers. This eliminates write contention entirely.

**Eventual consistency.** The CorticalState is not transactionally consistent. A safety-critical decision should never rely on multiple CorticalState signals being from the same tick. Safety constraints (PolicyCage, Capability tokens) operate on their own strongly-consistent state. The CorticalState is for heuristic decisions: attention allocation, inference tier selection, TUI rendering. Slight staleness is fine for all of those.

### Reading and writing

```rust
impl CorticalState {
    /// Read the full PAD vector as f64 for downstream math.
    pub fn pad(&self) -> PadVector {
        PadVector {
            pleasure: f32::from_bits(self.pleasure.load(Ordering::Acquire)) as f64,
            arousal: f32::from_bits(self.arousal.load(Ordering::Acquire)) as f64,
            dominance: f32::from_bits(self.dominance.load(Ordering::Acquire)) as f64,
        }
    }

    /// Aggregate prediction accuracy across all categories.
    pub fn prediction_accuracy(&self) -> f32 {
        f32::from_bits(self.aggregate_accuracy.load(Ordering::Acquire))
    }

    /// Current behavioral phase (Thriving through Terminal).
    pub fn phase(&self) -> BehavioralPhase {
        BehavioralPhase::from_u8(self.behavioral_phase.load(Ordering::Acquire))
    }

    /// Full snapshot for context assembly or TUI rendering.
    /// Reads all 32 signals. Not atomic across signals --
    /// individual signals are atomic, but the snapshot may
    /// span two ticks.
    pub fn snapshot(&self) -> CorticalSnapshot { /* reads all fields */ }
}
```

### Initialization

All 32 channels start at 0.0 (neutral). The PAD vector initializes to the personality baseline from `DaimonConfig.personality_preset`:

- **Cautious**: P=-0.1, A=0.1, D=-0.2
- **Balanced**: P=0.0, A=0.0, D=0.0
- **Aggressive**: P=0.1, A=0.3, D=0.2

Primary emotion starts as Anticipation (enum value 7), the "waiting to observe" state. Mood initializes to the personality baseline PAD.

If the Golem has a predecessor, the personality layer inherits from the predecessor's final mood at 0.5x decay (see `AlmaLayers::inherit`), and the initial CorticalState PAD reflects that inherited baseline instead.

### Serialization

CorticalState is serialized to `~/.bardo/golems/<name>/cortical.bin` using bincode format every 10 theta ticks (~5-20 minutes depending on theta interval). On restart, the runtime loads from the last checkpoint. Maximum data loss: 10 theta ticks of affect state, which is acceptable because the ALMA mood layer (4h decay) dominates and emotion-layer transients are inherently ephemeral.

AtomicI8 and AtomicU8 fields are converted to their plain integer types (i8, u8) for serialization. AtomicU32 fields storing f32 values are serialized as the raw u32 bits, not as f32, to avoid platform-dependent float serialization.

---

## Relationship to TUI Interpolating Variables

The TUI was originally specified with 26 interpolating variables. CorticalState has 32 atomic signals. These are not the same thing, and the distinction matters.

**CorticalState** is the fast atomic truth. Subsystems write signals here as they compute them. Reads are instantaneous. Values change discretely when a subsystem writes.

**TUI interpolating variables** are the smooth visual representation. Each variable has a current value that approaches a target via exponential decay at a variable lerp rate. The targets come from CorticalState snapshots and Event Fabric events. The current values are what the renderer actually reads.

The relationship: CorticalState signals set TUI targets. The TUI never reads CorticalState directly during rendering. It reads its own interpolated channels, which are always in motion toward the latest CorticalState values.

### The original 26

These are the variables from doc 01, now mapped to their CorticalState counterparts:

| TUI variable | CorticalState signal | Mapping |
|-------------|---------------------|---------|
| `pleasure` | `pleasure` | 1:1 |
| `arousal` | `arousal` | 1:1 |
| `dominance` | `dominance` | 1:1 |
| `emotion_label` | `primary_emotion` | 1:1 |
| `vitality_composite` | `economic_vitality + epistemic_vitality + stochastic_vitality` | Derived: weighted mean |
| `economic_clock` | `economic_vitality` | 1:1 |
| `epistemic_clock` | `epistemic_vitality` | 1:1 |
| `market_regime` | `regime` | 1:1 |
| `phase_density` | `behavioral_phase` | Derived: lookup table per phase |
| `phase_dimming` | `behavioral_phase` | Derived: lookup table per phase |
| `age_factor` | `stochastic_vitality` | Derived: `1.0 - stochastic_vitality` |

The remaining 15 of the original 26 (`fsm_phase`, `probe_severity`, `inference_glow`, `mouth_alpha`, `context_utilization`, `phi_score`, `dream_alpha`, `clade_connectivity`, `credit_balance`, `grimoire_density`, `burn_rate`, `heartbeat_phase`, `noise_floor`, `scanline_intensity`, `corruption_rate`) are not CorticalState signals. They come from Event Fabric events, local computation, or free-running oscillators. The CorticalState does not try to be exhaustive. It contains only the signals that multiple subsystems need to read at zero latency.

### The 6 new variables

Prediction subsystem signals need TUI representation. Adding 6 new interpolating variables brings the TUI to 32 total:

| # | Variable | Category | Lerp rate | Range | Source signal | What it drives |
|---|----------|----------|-----------|-------|---------------|----------------|
| 27 | `prediction_accuracy` | medium | 1.5 | [0.0, 1.0] | `aggregate_accuracy` | Mind screen accuracy gauge. Drives Oracle confidence indicator. Spectre dot orbit coherence: higher accuracy = tighter, more coherent dot orbits. |
| 28 | `accuracy_trend` | medium | 1.0 | [-1.0, 1.0] | `accuracy_trend` | Trend arrow direction and color. Green rising, amber flat, red falling. Spectre vertical posture offset: improving trend = upright posture, declining = slouched. |
| 29 | `attention_breadth` | medium | 1.2 | [0.0, 1.0] | `active_count / universe_size` | Attention focus indicator. Narrow = focused, wide = scanning. Peripheral particle density: narrow attention = fewer peripheral particles, wide = more. |
| 30 | `surprise_rate` | fast | 6.0 | [0.0, 1.0] | `surprise_rate` | Background flicker intensity. High surprise = unstable visual field. Eye micro-flicker: high surprise = rapid pupil dilation oscillation. |
| 31 | `foraging_activity` | medium | 1.0 | [0.0, 1.0] | `pending_predictions` | Discovery pulse in attention widget. Peripheral particle speed: high foraging = fast-moving peripheral dots suggesting active search. |
| 32 | `compounding_momentum` | glacial | 0.05 | [0.0, 1.0] | `compounding_momentum` | Background warmth. High momentum = warm golden undertone in the aura. Low = cool blue shift. |

These follow the same pattern as the original 26: each has a lerp rate, a range, a source, and a rendering consequence. The `surprise_rate` is fast because surprises demand immediate visual response. The `compounding_momentum` is glacial because it represents a long-term trajectory that should never jerk.

---

## The Adaptive Clock

The fixed ~60-second heartbeat from the previous architecture served all purposes at one rate. Wrong. A swap prediction resolves in seconds. An LP fee prediction resolves over hours. A regime prediction resolves over days. Biology solves this with oscillatory hierarchies [BUZSAKI-2006]. The adaptive clock borrows the structure.

### Three concurrent scales

| Scale | Interval | Role | Cost |
|-------|----------|------|------|
| **Gamma** | 5-15s | Perception: market data, prediction resolution, CorticalState updates, attention promotions | Near-zero (reads + arithmetic) |
| **Theta** | 30-120s | Cognition: predict, appraise, gate, [retrieve, deliberate, act], reflect. ~80% suppressed at gate | T0 $0.00, T1 $0.005, T2 $0.03 |
| **Delta** | ~50 theta-ticks | Consolidation: Grimoire curator, residual aggregation, attention rebalancing, dream scheduling | T0-T1 $0.00-$0.01 |

Each frequency adapts within bounds. Under volatility, gamma accelerates toward 5s and theta toward 30s. During flat markets, gamma slows to 15s and theta stretches to 120s:

```rust
// More violations -> faster gamma
self.interval = Duration::from_secs(15)
    .mul_f64(1.0 / (1.0 + violations.len() as f64 * 0.3))
    .max(Duration::from_secs(5));
```

Delta stays fixed at ~50 theta-ticks. Consolidation should not be rushed by market conditions.

The three-scale model draws from Friston's free energy principle [FRISTON-2010], which frames perception as hierarchical prediction at different temporal grains. Clark [CLARK-2013] extends this into the "predictive brain" framework: biological cognition is nested prediction loops at multiple timescales. Gamma is the sensory layer (fast, cheap, reactive). Theta is deliberative (slower, expensive, selective). Delta is consolidation (slow, maintenance).

Adaptive rates mean variable costs. At peak (5s gamma, 30s theta during a volatile period), the golem makes ~24 RPC calls per minute. Calm periods drop to ~5/min. The runtime tracks daily cost and throttles rates when approaching the budget ceiling in `golem.toml`.

---

## The Daimon Affect Engine

### PAD from prediction residuals

The Daimon computes the PAD (Pleasure, Arousal, Dominance) vector from prediction outcomes. This grounds emotion in something concrete: how well is the Golem predicting its environment?

Barrett's theory of constructed emotion [BARRETT-2017] reframes emotions as the brain's summary statistics of prediction errors -- not a separate evaluative system but cognition's self-assessment. The Golem's PAD vector follows this logic:

- **Pleasure** = `accuracy - baseline`. Predictions landing = positive. Predictions failing = negative.
- **Arousal** = `residual_magnitude`. Large errors in either direction = high arousal. The Golem is surprised.
- **Dominance** = `trend_direction`. Improving accuracy = high dominance. Declining = low.

### PAD update function

```rust
impl Daimon {
    /// Called on every prediction resolution (gamma frequency).
    pub fn update_pad_from_resolution(
        &mut self,
        res: &Resolution,
        cortical: &CorticalState,
    ) {
        // Pleasure: negativity bias (1.6x). Failures hurt more.
        let pleasure_delta = if res.correct { 0.05 } else { -0.08 };

        // Arousal: proportional to |residual|, clamped.
        let arousal_delta = (res.residual.abs() / self.expected_residual_magnitude)
            .min(1.0) * 0.1;

        // Dominance: asymmetric. Loss of control feels worse.
        let dominance_delta = match cortical.accuracy_trend.load(Ordering::Acquire) {
            1 => 0.03,   // improving
            -1 => -0.05, // declining
            _ => 0.0,
        };

        // EMA with per-dimension rates: pleasure 0.15, arousal 0.20, dominance 0.08
        self.state.emotion.pleasure = ema(self.state.emotion.pleasure, pleasure_delta, 0.15);
        self.state.emotion.arousal = ema(self.state.emotion.arousal, arousal_delta, 0.20);
        self.state.emotion.dominance = ema(self.state.emotion.dominance, dominance_delta, 0.08);

        // Compute effective PAD from all three ALMA layers.
        let effective = self.state.effective_pad();

        // Write to CorticalState for zero-latency reads by other subsystems.
        cortical.pleasure.store(
            (effective.pleasure as f32).to_bits(), Ordering::Release
        );
        cortical.arousal.store(
            (effective.arousal as f32).to_bits(), Ordering::Release
        );
        cortical.dominance.store(
            (effective.dominance as f32).to_bits(), Ordering::Release
        );

        // Update Plutchik label.
        let emotion = PlutchikEmotion::from_pad(&effective);
        cortical.primary_emotion.store(emotion as u8, Ordering::Release);
    }
}

// Note: this formula simplifies to `current + delta * alpha`, a leaky integrator
// with stimulus-proportional steps. Not a true EMA (which would be
// `current * (1-alpha) + target * alpha`). Here, each stimulus nudges the value
// by `alpha * delta`, then the value drifts toward zero through subsequent
// zero-delta ticks.
fn ema(current: f64, delta: f64, alpha: f64) -> f64 {
    current * (1.0 - alpha) + (current + delta) * alpha
}
```

The 1.6x negativity bias means failures degrade pleasure more than successes improve it. A Golem alternating between hits and misses drifts negative over time. This is intentional -- the asymmetry makes Golems cautious by default, the right bias for systems handling real capital. The 1.6x ratio is conservative relative to Baumeister et al. 2001 ("Bad is Stronger Than Good"), which reports negativity ratios of 2.0-5.0 across psychological domains, and Kahneman and Tversky's ~2.25 loss aversion coefficient from prospect theory. 1.6 was chosen because the Golem operates in financial markets where excessive negativity bias would cause premature risk aversion and missed opportunities. A 2.0+ ratio in testing caused Golems to enter near-permanent low-pleasure states after normal losing streaks.

---

## ALMA Three-Layer Model

Gebhard's ALMA model [GEBHARD-2005] decomposes affect into three temporal layers, each feeding the next.

**Emotion layer** (per-tick, ~30s decay). Each prediction resolution triggers an emotion pulse. Correct = joy. Large error = surprise or fear. The pulse decays exponentially. This layer is reactive, jittery, immediate. The 30-second decay constant is a middle ground between transient physiological arousal (~5-10s, Scherer 2005 component process model) and subjective feeling persistence (~60-120s). Shorter than biological feeling duration because the Golem resolves predictions faster than a human resolves emotional episodes.

**Mood layer** (rolling hours, ~4h decay). Exponential moving average of the emotion layer. Smooths per-tick noise into sustained affect. A string of successes produces positive mood even if individual pulses were modest. Sustained failures drag mood negative. The 4-hour time constant aligns with Gebhard 2005 ALMA model's "medium-term affect" layer, which uses 2-8 hour time constants for mood-level persistence. Four hours was chosen as a trading-day half-life: a bad morning's mood is half-decayed by the afternoon session.

**Personality layer** (lifetime baseline). Set at birth, barely drifts during the Golem's life. Inherited from the predecessor's final mood at 0.5x decay -- the Baldwin Effect. Behavioral patterns that proved adaptive become structural defaults in the next generation.

### Layer composition

```rust
pub struct AlmaLayers {
    pub emotion: PadVector,    // per-tick, ~30s decay
    pub mood: PadVector,       // rolling hours, ~4h decay
    pub personality: PadVector, // lifetime baseline, inherited at 0.5x
}

impl AlmaLayers {
    /// Weights: personality 0.25, mood 0.50, emotion 0.25.
    /// Mood dominates -- the sustained trajectory matters more
    /// than per-tick jitter or fixed personality.
    pub fn effective_pad(&self) -> PadVector {
        PadVector {
            pleasure: self.personality.pleasure * 0.25
                + self.mood.pleasure * 0.50
                + self.emotion.pleasure * 0.25,
            arousal: self.personality.arousal * 0.25
                + self.mood.arousal * 0.50
                + self.emotion.arousal * 0.25,
            dominance: self.personality.dominance * 0.25
                + self.mood.dominance * 0.50
                + self.emotion.dominance * 0.25,
        }
    }

    pub fn apply_emotion_pulse(&mut self, pulse: &PadVector, dt_secs: f64) {
        let decay = (-dt_secs / 30.0).exp(); // 30s time constant
        self.emotion.pleasure = self.emotion.pleasure * decay + pulse.pleasure;
        self.emotion.arousal = self.emotion.arousal * decay + pulse.arousal;
        self.emotion.dominance = self.emotion.dominance * decay + pulse.dominance;

        // Clamp to [-1, 1].
        self.emotion = self.emotion.clamp(-1.0, 1.0);
    }

    pub fn update_mood(&mut self, dt_secs: f64) {
        let alpha = 1.0 - (-dt_secs / (4.0 * 3600.0)).exp(); // 4h time constant
        self.mood.pleasure = lerp(self.mood.pleasure, self.emotion.pleasure, alpha);
        self.mood.arousal = lerp(self.mood.arousal, self.emotion.arousal, alpha);
        self.mood.dominance = lerp(self.mood.dominance, self.emotion.dominance, alpha);
    }

    pub fn inherit(predecessor_final_mood: &PadVector) -> Self {
        AlmaLayers {
            emotion: PadVector::ZERO,
            mood: PadVector::ZERO,
            personality: PadVector {
                pleasure: predecessor_final_mood.pleasure * 0.5,
                arousal: predecessor_final_mood.arousal * 0.5,
                dominance: predecessor_final_mood.dominance * 0.5,
            },
        }
    }
}
```

---

## Somatic Markers

Damasio [DAMASIO-1994, DAMASIO-1996] showed that patients who lost emotional signaling but retained full cognition made consistently worse decisions under uncertainty. They could reason about risks but could not feel them. Pure reasoning, without the somatic marker (a learned association between situation and bodily state), performed worse than reasoning guided by feeling.

For the Golem, somatic markers are learned PAD-context-outcome associations stored as Grimoire entries.

### The struct

```rust
/// Learned PAD-context-outcome association. Stored as Grimoire entry.
/// Fires automatically when the Golem enters a matching PAD region
/// under matching environmental conditions.
pub struct SomaticMarker {
    pub trigger_pad: PadRegion,          // PAD subspace that triggers
    pub trigger_context: SomaticContext, // environmental conditions
    pub associated_outcome: OutcomeType, // what historically followed
    pub confidence: f64,                 // strength, pruned below 0.1
    pub behavioral_bias: BehavioralBias, // caution / neutral / aggression
    pub activation_count: u32,
    pub confirmation_count: u32,
    pub created_tick: u64,
    pub last_activated_tick: u64,
}

pub struct PadRegion {
    pub center: PadVector,
    pub radius: f64, // Euclidean distance threshold
}

pub struct SomaticContext {
    pub regime: Option<MarketRegime>,
    pub category: Option<PredictionCategory>,
    pub creative_mode: Option<bool>,
}

#[derive(Clone, Copy)]
pub enum BehavioralBias { Caution, Neutral, Aggression }

#[derive(Clone, Copy)]
pub enum OutcomeType { Loss, Gain, MissedOpportunity, AvoidedLoss }
```

### Marker search

Somatic marker search uses a `BTreeMap<PadRegion, Vec<SomaticMarker>>` keyed by quantized PAD octant (the sign of each PAD dimension gives 8 octants). Lookup: O(log 8) = O(1) to find the octant, then linear scan within that octant's vector (typically <20 markers). Euclidean distance threshold for activation: 0.3 in PAD space (matching the `PadRegion.radius` field). For >100 markers per octant, spatial indexing (e.g., a k-d tree) would help, but this is unlikely in practice: golems prune markers below 0.1 confidence, and the 8-octant partition keeps each bucket small.

### How markers form

A marker forms when a PAD region co-occurs with an outcome at least twice. First occurrence: candidate at `confidence: 0.3`. Second: active at `confidence: 0.5`. Each confirmation increases confidence; each disconfirmation decreases it.

Example: the Golem enters `pleasure: -0.6, arousal: 0.8, dominance: -0.3` during volatile conditions, trades, and loses. This happens twice more. A marker forms: "anxious + volatile = losses." Next time that PAD region is entered during volatility, the marker fires before the LLM deliberates. The action gate receives `Caution`, reducing permitted position sizes. The Golem feels the danger before it thinks about it.

### Inheritance

When a Golem dies, Thanatopsis exports confirmed markers (confidence > 0.7) to the death testament. The successor imports them at 0.5x confidence -- gut feelings inherited but weakened enough to be overridden by fresh observation.

---

## Plutchik Emotion Labels

The PAD vector is continuous and three-dimensional. For display, logging, and episode tagging, the Daimon maps PAD to one of Plutchik's 8 primary emotions. The mapping divides PAD space into regions.

### PAD-to-emotion mapping

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PlutchikEmotion {
    Joy         = 0, // High P, moderate A, moderate D
    Trust       = 1, // Moderate P, low A, moderate D
    Fear        = 2, // Low P, high A, low D
    Surprise    = 3, // Neutral P, high A, low D
    Sadness     = 4, // Low P, low A, low D
    Disgust     = 5, // Low P, moderate A, moderate D
    Anger       = 6, // Low P, high A, high D
    Anticipation = 7, // Moderate P, moderate A, high D
}

impl PlutchikEmotion {
    /// Map a continuous PAD vector to the nearest Plutchik label.
    ///
    /// Uses squared Euclidean distance to 8 prototype points.
    /// The prototypes are empirically derived from Russell & Mehrabian (1977)
    /// and adapted for the Golem's domain.
    pub fn from_pad(pad: &PadVector) -> Self {
        let prototypes: [(PlutchikEmotion, [f64; 3]); 8] = [
            (Self::Joy,          [ 0.7,  0.3,  0.4]),
            (Self::Trust,        [ 0.4, -0.2,  0.3]),
            (Self::Fear,         [-0.6,  0.8, -0.5]),
            (Self::Surprise,     [ 0.0,  0.9, -0.3]),
            (Self::Sadness,      [-0.6, -0.4, -0.5]),
            (Self::Disgust,      [-0.5,  0.3,  0.2]),
            (Self::Anger,        [-0.5,  0.8,  0.5]),
            (Self::Anticipation, [ 0.3,  0.4,  0.6]),
        ];

        prototypes.iter()
            .min_by(|a, b| {
                let dist_a = sq_dist(pad, &a.1);
                let dist_b = sq_dist(pad, &b.1);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(emotion, _)| *emotion)
            .unwrap_or(Self::Surprise) // fallback
    }

    pub fn from_u8(v: u8) -> Self { /* match 0-7, default Surprise */ }
}
```

### What each emotion means for the Golem

| Emotion | Behavioral consequence |
|---------|----------------------|
| **Joy** | Predictions landing. Normal operation, moderate risk tolerance. |
| **Trust** | Calm confidence after sustained accuracy. Increased position sizes permitted. |
| **Fear** | Predictions failing, cause unclear. Maximum caution. Position sizes minimized. |
| **Surprise** | Unexpected event, not yet categorized. Bumps inference tier, holds positions steady. |
| **Sadness** | Extended failure, no longer surprising. Conservation behavior. Reduced activity. |
| **Disgust** | Poor performance but capacity to act. Drives avoidance of recognized failure patterns. |
| **Anger** | Failing but believes it can fix it. Higher inference budget, aggressive strategy revision. |
| **Anticipation** | Expects something and feels equipped. Pre-positions for predicted events. |

The Spectre's eye expression in the TUI sidebar maps directly to these labels. Joy = rounded eyes, slight upward tilt. Fear = wide, contracted. Sadness = drooped, narrowed. The visual gives the owner an instant read on affective state without checking numbers.

---

## Affect-Modulated Retrieval

Bower's mood-congruent memory [BOWER-1981]: people in negative moods recall negative memories more readily. This is not a bias to correct -- it is adaptive. The anxiety is information about the current situation. The Daimon modulates Grimoire retrieval by computing a relevance multiplier based on PAD alignment between the Golem's current state and the state when the memory formed.

### Retrieval scoring

```rust
impl Daimon {
    /// Returns [0.5, 2.0] multiplier on base relevance score.
    /// Aligned PAD = boosted. Opposite PAD = suppressed, never zeroed.
    pub fn affective_relevance(
        &self,
        entry_pad: &PadVector,
        current_pad: &PadVector,
    ) -> f64 {
        let alignment = pad_cosine_similarity(current_pad, entry_pad);

        // alignment is in [-1.0, 1.0].
        // Map to [0.5, 2.0]:
        //   alignment  1.0 -> multiplier 2.0 (strong match)
        //   alignment  0.0 -> multiplier 1.25 (neutral)
        //   alignment -1.0 -> multiplier 0.5 (opposite mood)
        0.5 + 0.75 * (alignment + 1.0)
    }
}

fn pad_cosine_similarity(a: &PadVector, b: &PadVector) -> f64 {
    let dot = a.pleasure * b.pleasure + a.arousal * b.arousal + a.dominance * b.dominance;
    let mag_a = (a.pleasure.powi(2) + a.arousal.powi(2) + a.dominance.powi(2)).sqrt();
    let mag_b = (b.pleasure.powi(2) + b.arousal.powi(2) + b.dominance.powi(2)).sqrt();

    if mag_a < 1e-8 || mag_b < 1e-8 {
        return 0.0; // near-zero PAD, no meaningful alignment
    }
    dot / (mag_a * mag_b)
}
```

### What this means in practice

An anxious Golem (`pleasure: -0.5, arousal: 0.7`) retrieves cautionary memories: loss episodes, predecessor warnings, somatic markers tagged `Caution`. The LLM's context window tilts toward bad outcomes under similar conditions, producing conservative recommendations. A confident Golem (`pleasure: 0.5, dominance: 0.5`) retrieves optimization knowledge and successful trades, producing aggressive recommendations. A neutral Golem retrieves balanced memories with no affective weighting.

The multiplier range of [0.5, 2.0] is intentionally bounded. Affect biases retrieval but cannot suppress relevant memories entirely or promote irrelevant ones past semantic relevance. High semantic relevance with opposite affect still scores well. Low semantic relevance with matching affect cannot be boosted past better matches.

---

## The Full Daimon State

Putting it all together: the Daimon maintains a `DaimonState` that holds the ALMA layers, the somatic marker index, and the Plutchik label.

```rust
pub struct DaimonState {
    pub layers: AlmaLayers,
    pub somatic_markers: Vec<SomaticMarker>,  // cached from Grimoire
    pub expected_residual_magnitude: f64,      // normalizes arousal
    pub accuracy_baselines: [f64; 16],         // pleasure is relative to these
    pub config: DaimonConfig,
}

pub struct DaimonConfig {
    pub negativity_bias: f64,        // default 1.6
    pub arousal_alpha: f64,          // EMA decay, default 0.20
    pub pleasure_alpha: f64,         // EMA decay, default 0.15
    pub dominance_alpha: f64,        // EMA decay, default 0.08
    pub personality_preset: PersonalityPreset, // first boot only
    pub marker_prune_threshold: f64, // default 0.1
}

/// Personality presets for first-boot (no predecessor).
/// Cautious:   P=-0.1, A=0.2, D=0.0  (slightly pessimistic, alert)
/// Balanced:   P= 0.0, A=0.1, D=0.1  (neutral baseline)
/// Aggressive: P= 0.2, A=0.1, D=0.3  (optimistic, confident)
#[derive(Clone, Copy)]
pub enum PersonalityPreset { Cautious, Balanced, Aggressive }
```

---

## Owner Interaction with Affect

Affect is not directly settable. You cannot type `set pleasure 0.8` and make your Golem happy. The PAD vector is downstream of prediction accuracy, market conditions, and experience. Setting it directly would be lying to the cognitive system -- somatic markers would fire incorrectly, retrieval would surface wrong memories, the action gate would permit unfounded trades. Direct affect manipulation is sabotage.

What the owner can do:

**Adjust strategy parameters.** A narrower strategy (fewer approved assets, lower max position size) reduces arousal by limiting the Golem's exposure to surprising events. A wider strategy increases arousal. The Golem's affect responds to the strategy over time because the strategy changes what predictions the Golem makes and how often they succeed.

**Adjust risk bounds.** Tighter risk limits (lower `risk_ceiling`, lower max slippage) increase dominance because the Golem "feels" more in control -- it has clearer boundaries, which means its predictions about its own behavior are more accurate. Loose limits decrease dominance.

**Change personality preset.** At first boot (no predecessor), the owner can choose Cautious, Balanced, or Aggressive. This sets the personality layer baseline. After boot, personality is fixed for the Golem's lifetime. The next Golem inherits from this one's final mood, not from the preset.

**Kill or pause.** Absolute controls that bypass affect. Pause stops action but lets the Golem keep observing and predicting. When unpaused, affect reflects the observation period.

**Read the Spectre.** The primary interaction with affect is observation. The Spectre's expression, color, breathing, and density all reflect PAD. An experienced owner learns to read the Spectre the way a trader reads a chart: not as a number to optimize but as a signal to interpret. If the Spectre looks fearful and you think the market is fine, maybe the Golem knows something you don't. Or maybe it's wrong and you should adjust its strategy. That judgment is the owner's job.

---

## TaCorticalExtension: Satellite Perception Surface

The TA research (papers ta/01 through ta/09) generates eight new atomic signals. These cannot go into CorticalState proper without pushing it past the 4-cache-line / 256-byte invariant. Instead, they live in a satellite struct that follows the same design principles: lock-free atomics, single-writer ownership, relaxed ordering.

Conflict resolution reference: [04-conflict-resolution.md](../../tmp/research/witness-research/new/reconciling/04-conflict-resolution.md), Conflict 5.

### Why not expand CorticalState directly

Three reasons. First, CorticalState is Layer 0. Every subsystem reads it. Adding TA-specific fields creates a dependency from the core perception surface to a domain-specific analysis module. Second, the TA subsystem is optional. A Golem running without TA should not carry 32 extra bytes of TA state. Third, the satellite pattern allows the TA extension to be replaced or extended without touching CorticalState's ABI.

### The struct

```rust
/// TA perception surface. Satellite to CorticalState.
/// Written exclusively by the TA analysis pipeline.
/// Read by Oracle, Daimon, TUI, and Dream engine.
///
/// 8 atomic signals, 32 bytes payload, padded to 64 bytes
/// for cache-line alignment. One cache line. One writer per signal.
#[repr(C, align(64))]
pub struct TaCorticalExtension {
    // ═══ HDC PATTERN STATE — written by TaPatternCodebook ═══
    /// Best pattern match similarity from the current tick's state vector.
    /// f32 via to_bits(). Range [0.5, 1.0]. 0.5 = no match above threshold.
    pub pattern_match_score: AtomicU32,

    // ═══ MANIFOLD STATE — written by SpectralManifold ═══
    /// Ricci scalar curvature at the Golem's current manifold position.
    /// f32 via to_bits(). Positive = stable basin. Negative = saddle/instability.
    pub manifold_curvature: AtomicU32,

    // ═══ CAUSAL STATE — written by CausalDiscoveryEngine ═══
    /// Number of active causal edges in the live DAG.
    /// u16 packed into u32. Typical range 10-200.
    pub causal_edge_count: AtomicU32,

    // ═══ SIGNAL ECOSYSTEM — written by SignalMetabolism ═══
    /// Population fitness of the signal ecosystem. EWMA of mean signal fitness.
    /// f32 via to_bits(). Range [0.0, 1.0].
    pub signal_ecosystem_fitness: AtomicU32,

    // ═══ ADVERSARIAL STATE — written by AdversarialDefense ═══
    /// Fraction of recent observations flagged as adversarial.
    /// f32 via to_bits(). Range [0.0, 1.0]. Above 0.3 = heavy manipulation.
    pub adversarial_fraction: AtomicU32,

    // ═══ TOPOLOGICAL STATE — written by PredictiveGeometry ═══
    /// Persistence landscape derivative norm. Rate of topological change.
    /// f32 via to_bits(). High values signal impending regime transition.
    pub topology_change_rate: AtomicU32,

    // ═══ CROSS-PROTOCOL — written by EntanglementTracker ═══
    /// Maximum entanglement drift across all tracked protocol pairs.
    /// f32 via to_bits(). Range [0.0, 1.0]. High = protocols correlating.
    pub entanglement_drift: AtomicU32,

    // ═══ SOMATIC — written by SomaticTaEngine ═══
    /// Somatic marker intensity. Absolute affect retrieval strength.
    /// f32 via to_bits(). Range [0.0, 1.0]. High = strong gut feeling active.
    pub somatic_intensity: AtomicU32,
}

impl TaCorticalExtension {
    /// Zero-initialized. All signals default to neutral.
    pub fn new() -> Self {
        Self {
            pattern_match_score: AtomicU32::new(0.5_f32.to_bits()),
            manifold_curvature: AtomicU32::new(0.0_f32.to_bits()),
            causal_edge_count: AtomicU32::new(0),
            signal_ecosystem_fitness: AtomicU32::new(0.5_f32.to_bits()),
            adversarial_fraction: AtomicU32::new(0.0_f32.to_bits()),
            topology_change_rate: AtomicU32::new(0.0_f32.to_bits()),
            entanglement_drift: AtomicU32::new(0.0_f32.to_bits()),
            somatic_intensity: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

    pub fn snapshot(&self) -> TaCorticalSnapshot {
        TaCorticalSnapshot {
            pattern_match_score: f32::from_bits(
                self.pattern_match_score.load(Ordering::Acquire)),
            manifold_curvature: f32::from_bits(
                self.manifold_curvature.load(Ordering::Acquire)),
            causal_edge_count: self.causal_edge_count.load(Ordering::Acquire) as u16,
            signal_ecosystem_fitness: f32::from_bits(
                self.signal_ecosystem_fitness.load(Ordering::Acquire)),
            adversarial_fraction: f32::from_bits(
                self.adversarial_fraction.load(Ordering::Acquire)),
            topology_change_rate: f32::from_bits(
                self.topology_change_rate.load(Ordering::Acquire)),
            entanglement_drift: f32::from_bits(
                self.entanglement_drift.load(Ordering::Acquire)),
            somatic_intensity: f32::from_bits(
                self.somatic_intensity.load(Ordering::Acquire)),
        }
    }
}
```

### Writer ownership table

| Signal | Writer | Frequency | Source |
|--------|--------|-----------|--------|
| `pattern_match_score` | `TaPatternCodebook::match_patterns()` | Gamma | ta/01 |
| `manifold_curvature` | `SpectralManifold::update_curvature()` | Gamma | ta/02 |
| `causal_edge_count` | `CausalDiscoveryEngine::pc_update()` | Theta | ta/04 |
| `signal_ecosystem_fitness` | `SignalMetabolism::replicator_step()` | Theta | ta/03 |
| `adversarial_fraction` | `AdversarialDefense::scan()` | Gamma | ta/08 |
| `topology_change_rate` | `PredictiveGeometry::landscape_delta()` | Gamma | ta/05 |
| `entanglement_drift` | `EntanglementTracker::drift()` | Gamma | ta/01 |
| `somatic_intensity` | `SomaticTaEngine::retrieve()` | Gamma | ta/09 |

Every signal has exactly one writer. The satellite struct is registered with the runtime alongside CorticalState and accessible through `Arc<TaCorticalExtension>` passed during initialization.

### Registration

```rust
impl GolemRuntime {
    pub fn init_perception(&mut self) {
        self.cortical_state = Arc::new(CorticalState::new());

        // TA extension: only if TA subsystem is enabled
        if self.config.ta.enabled {
            self.ta_extension = Some(Arc::new(TaCorticalExtension::new()));
        }
    }
}
```

---

## HDC Encoding of CorticalState

CorticalState's 32 atomic signals can be encoded as a single hyperdimensional binary vector for compositional queries and fast similarity search. The encoding uses Binary Spatter Codes (Kanerva, 1996) with dimensionality D = 10,240 bits (1,280 bytes).

Source: [01-hdc-integration-map.md](../../tmp/research/witness-research/new/reconciling/01-hdc-integration-map.md).

### Why encode CorticalState as HDC

Three use cases:

1. **WorldModelHistory queries.** "What was the agent's state when it last saw this pattern?" requires comparing the current CorticalState against stored historical states. HDC encoding reduces this to Hamming distance computation: ~1 microsecond per comparison.

2. **Grimoire retrieval augmentation.** Somatic markers and episodic memories can be indexed by the CorticalState at formation time. HDC encoding gives a structural fingerprint that complements text embedding similarity.

3. **Clade state comparison.** Two Golems in the same Clade can compare their CorticalState encodings to detect whether they're in similar cognitive states. This enables coordinated behavior without sharing raw state.

### Encoding scheme

Each atomic signal gets a random, fixed **role vector** R_i (D = 10,240 bits, ~50% ones). The signal's value is quantized to one of K levels, each with a random **filler vector** F_k. The role-filler pair is bound via XOR:

```
bound_i = R_i XOR F_{quantize(signal_i)}
```

The full CorticalState encoding is the bundled (majority-vote) superposition of all bound pairs:

```
H = bundle(bound_1, bound_2, ..., bound_32)
```

This is a single 10,240-bit vector that holographically encodes the entire CorticalState. The encoding preserves compositional structure: given H and a role vector R_i, unbinding (XOR) recovers an approximation of the filler F_k, which gives the quantized signal value.

```rust
/// HDC encoder for CorticalState.
pub struct CorticalHdcEncoder {
    /// Role vectors: one per signal (32 total).
    role_vectors: Vec<BitVector>,
    /// Filler vectors: K levels per signal.
    filler_vectors: Vec<Vec<BitVector>>,
    /// Quantization levels per signal.
    levels: usize,
    /// D = 10,240
    dimensionality: usize,
}

/// A binary vector of D = 10,240 bits, stored as 160 u64 words.
#[derive(Clone)]
pub struct BitVector {
    words: [u64; 160],
}

impl BitVector {
    /// XOR binding. Its own inverse.
    pub fn bind(&self, other: &BitVector) -> BitVector {
        let mut result = BitVector { words: [0; 160] };
        for i in 0..160 {
            result.words[i] = self.words[i] ^ other.words[i];
        }
        result
    }

    /// Hamming distance. Number of differing bits.
    pub fn hamming_distance(&self, other: &BitVector) -> u32 {
        let mut dist = 0u32;
        for i in 0..160 {
            dist += (self.words[i] ^ other.words[i]).count_ones();
        }
        dist
    }

    /// Normalized similarity: 1.0 - hamming/D.
    pub fn similarity(&self, other: &BitVector) -> f64 {
        1.0 - self.hamming_distance(other) as f64 / 10240.0
    }
}

impl CorticalHdcEncoder {
    /// Encode a CorticalSnapshot into a single HDC vector.
    pub fn encode(&self, snapshot: &CorticalSnapshot) -> BitVector {
        let signals = snapshot.to_f32_array(); // 32 f32 values
        let mut accumulator = vec![0i32; self.dimensionality];

        for (i, &signal) in signals.iter().enumerate() {
            let level = self.quantize(signal, i);
            let bound = self.role_vectors[i].bind(&self.filler_vectors[i][level]);
            // Accumulate for majority vote
            for (j, word) in bound.words.iter().enumerate() {
                for bit in 0..64 {
                    if (word >> bit) & 1 == 1 {
                        accumulator[j * 64 + bit] += 1;
                    } else {
                        accumulator[j * 64 + bit] -= 1;
                    }
                }
            }
        }

        // Threshold to binary
        let mut result = BitVector { words: [0; 160] };
        for (j, word) in result.words.iter_mut().enumerate() {
            for bit in 0..64 {
                if accumulator[j * 64 + bit] > 0 {
                    *word |= 1u64 << bit;
                }
            }
        }
        result
    }

    fn quantize(&self, value: f32, signal_idx: usize) -> usize {
        // Linear quantization into self.levels buckets
        let clamped = value.clamp(0.0, 1.0);
        ((clamped * (self.levels - 1) as f32) as usize).min(self.levels - 1)
    }
}
```

### WorldModelHistory

The encoded CorticalState at each theta tick is stored in a ring buffer for compositional queries:

```rust
/// Rolling history of HDC-encoded CorticalState snapshots.
/// Supports compositional queries: "What was the state when X happened?"
pub struct WorldModelHistory {
    buffer: CircularBuffer<(u64, BitVector), 512>, // (tick, encoding)
}

impl WorldModelHistory {
    /// Push a new snapshot.
    pub fn push(&mut self, tick: u64, encoding: BitVector) {
        self.buffer.push((tick, encoding));
    }

    /// Find the stored state most similar to the query.
    pub fn query(&self, query: &BitVector) -> Option<(u64, f64)> {
        self.buffer.iter()
            .map(|(tick, stored)| (*tick, stored.similarity(query)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    /// Average encoding over a window for smoothed comparison.
    pub fn windowed_average(&self, window: usize) -> Option<BitVector> {
        let recent: Vec<_> = self.buffer.iter()
            .rev()
            .take(window)
            .map(|(_, bv)| bv)
            .collect();
        if recent.is_empty() { return None; }

        // Majority vote across the window
        let mut accumulator = vec![0i32; 10240];
        for bv in &recent {
            for (j, word) in bv.words.iter().enumerate() {
                for bit in 0..64 {
                    if (word >> bit) & 1 == 1 {
                        accumulator[j * 64 + bit] += 1;
                    } else {
                        accumulator[j * 64 + bit] -= 1;
                    }
                }
            }
        }

        let mut result = BitVector { words: [0; 160] };
        for (j, word) in result.words.iter_mut().enumerate() {
            for bit in 0..64 {
                if accumulator[j * 64 + bit] > 0 {
                    *word |= 1u64 << bit;
                }
            }
        }
        Some(result)
    }
}
```

### Performance

Encoding a 32-signal CorticalSnapshot: ~5 microseconds (32 XOR operations on 160 u64 words + one majority vote pass). Hamming distance comparison: ~200 nanoseconds (160 XOR + POPCNT operations). Memory per stored snapshot: 1,280 bytes. A 512-entry ring buffer uses 640 KB.

---

## Complete Writer Ownership Table

All CorticalState and TaCorticalExtension signals with their exclusive writers:

### CorticalState (32 signals, 192 bytes)

| Signal | Type | Writer | Frequency |
|--------|------|--------|-----------|
| `pleasure` | AtomicU32 | Daimon | Gamma |
| `arousal` | AtomicU32 | Daimon | Gamma |
| `dominance` | AtomicU32 | Daimon | Gamma |
| `primary_emotion` | AtomicU8 | Daimon | Gamma |
| `aggregate_accuracy` | AtomicU32 | Oracle | Gamma |
| `accuracy_trend` | AtomicI8 | Oracle | Gamma |
| `category_accuracies[0..15]` | [AtomicU32; 16] | Oracle | Gamma |
| `surprise_rate` | AtomicU32 | Oracle | Gamma |
| `universe_size` | AtomicU32 | AttentionForager | Gamma |
| `active_count` | AtomicU16 | AttentionForager | Gamma |
| `pending_predictions` | AtomicU32 | Oracle | Gamma |
| `creative_mode` | AtomicU8 | Dream engine | Delta |
| `fragments_captured` | AtomicU32 | Dream engine | Delta |
| `last_novel_prediction_tick` | AtomicU32 | Oracle | Gamma |
| `last_novel_prediction_tick_hi` | AtomicU32 | Oracle | Gamma |
| `regime` | AtomicU8 | Domain probes | Gamma |
| `gas_gwei` | AtomicU32 | Domain probes | Gamma |
| `economic_vitality` | AtomicU32 | Mortality engine | Gamma |
| `epistemic_vitality` | AtomicU32 | Mortality engine | Gamma |
| `stochastic_vitality` | AtomicU32 | Mortality engine | Gamma |
| `behavioral_phase` | AtomicU8 | Mortality engine | Gamma |
| `compounding_momentum` | AtomicU32 | Runtime | Delta |

### TaCorticalExtension (8 signals, 64 bytes)

| Signal | Type | Writer | Frequency |
|--------|------|--------|-----------|
| `pattern_match_score` | AtomicU32 | TaPatternCodebook | Gamma |
| `manifold_curvature` | AtomicU32 | SpectralManifold | Gamma |
| `causal_edge_count` | AtomicU32 | CausalDiscoveryEngine | Theta |
| `signal_ecosystem_fitness` | AtomicU32 | SignalMetabolism | Theta |
| `adversarial_fraction` | AtomicU32 | AdversarialDefense | Gamma |
| `topology_change_rate` | AtomicU32 | PredictiveGeometry | Gamma |
| `entanglement_drift` | AtomicU32 | EntanglementTracker | Gamma |
| `somatic_intensity` | AtomicU32 | SomaticTaEngine | Gamma |

Total: 40 signals across both structs. 256 bytes (CorticalState, 4 cache lines) + 64 bytes (TaCorticalExtension, 1 cache line) = 320 bytes, 5 cache lines. The two structs are at separate, independently aligned addresses. No false sharing between them.

---

## HomeostasisRegulator: CorticalState Consumer (from source 04-homeostasis)

The `HomeostasisRegulator` is the primary closed-loop controller that reads CorticalState signals and nudges AgentConfig in response to persistent deviations. It reads but never writes CorticalState signals; it writes only to AgentConfig.

The regulator implements proportional control (no integral or derivative terms). Full PID would overcorrect: agent configuration knobs do not have the predictable response times of physical actuators. Proportional control is stable and predictable.

### CorticalStateReader trait

```rust
pub trait CorticalStateReader {
    fn read_signal(&self, id: &SignalId) -> f32;
    fn regime(&self) -> MarketRegime;
}
```

CorticalState implements this trait, exposing its atomic signals as f32 reads for the regulator. The regulator connects specific signals to specific corrective actions:

| Signal | Condition | Actuator |
|--------|-----------|----------|
| `economic_vitality` | Below rolling average for 10+ ticks | Tighten tool trust thresholds |
| `aggregate_accuracy` | Declining trend for 15+ ticks | Bias inference tier toward T2 |
| `pleasure` | Chronically low for 20+ ticks | Set `DreamMode::Intensive` |

### Allostatic adaptation

Sterling and Eyer (1988) extended homeostasis with allostasis: setpoints shift with context. The `HomeostaticRule` evaluates deviations against a rolling EMA average (alpha 0.05) rather than a fixed setpoint. The average *is* the setpoint, and it shifts as the Golem's baseline CorticalState evolves. A Golem in a volatile regime naturally runs different baselines than one in a stable regime.

Barrett and Simmons (2015) describe interoceptive predictive coding: the brain maintains predictions about body states and issues corrections when prediction error exceeds a threshold. The `persistence_ticks` field captures this: a single deviant CorticalState reading may be noise; N consecutive deviant readings warrant correction.

See [03b-cognitive-mechanisms.md](./03b-cognitive-mechanisms.md) Section 4 for the full HomeostasisRegulator implementation.

---

## References

- [BARRETT-2017] Barrett, L.F. "The Theory of Constructed Emotion: An Active Inference Account of Interoception and Categorization." *Social Cognitive and Affective Neuroscience*, 12(1), 2017. — *Proposes that emotions are constructed from interoceptive predictions rather than triggered by fixed circuits; the theoretical model behind Daimon's construction of affect states from market signals rather than hard-coded sentiment rules.*
- [BOWER-1981] Bower, G.H. "Mood and Memory." *American Psychologist*, 36(2), 1981. — *Demonstrates mood-congruent memory retrieval where emotional state biases which memories are recalled; the basis for Daimon's affect-weighted Grimoire retrieval scoring.*
- [BUZSAKI-2006] Buzsaki, G. *Rhythms of the Brain*. Oxford University Press, 2006. — *Comprehensive treatment of neural oscillations and their role in coordinating brain function; the neuroscience model for CorticalState's tick-synchronized signal propagation.*
- [CLARK-2013] Clark, A. "Whatever Next? Predictive Brains, Situated Agents, and the Future of Cognitive Science." *Behavioral and Brain Sciences*, 36(3), 2013. — *Proposes the predictive processing framework where brains are fundamentally prediction machines minimizing prediction error; the cognitive paradigm that CorticalState implements as a shared prediction-error surface.*
- [DAMASIO-1994] Damasio, A. *Descartes' Error: Emotion, Reason, and the Human Brain*. Putnam, 1994. — *Argues that emotional signals are necessary for rational decision-making, not opposed to it; the foundational justification for integrating Daimon affect into CorticalState alongside market data signals.*
- [DAMASIO-1996] Damasio, A. "The Somatic Marker Hypothesis and the Possible Functions of the Prefrontal Cortex." *Philosophical Transactions of the Royal Society B*, 351(1346), 1996. — *Formalizes the somatic marker hypothesis where body-state signals tag decision options with emotional valence; the specific mechanism Daimon implements via somatic markers that bias the Golem's action selection.*
- [FRISTON-2010] Friston, K. "The Free-Energy Principle: A Unified Brain Theory?" *Nature Reviews Neuroscience*, 11(2), 2010. — *Proposes that all adaptive systems minimize variational free energy; the theoretical umbrella connecting CorticalState's prediction signals to the Golem's overall survival objective.*
- [GEBHARD-2005] Gebhard, P. "ALMA -- A Layered Model of Affect." *Proceedings of the Fourth International Joint Conference on Autonomous Agents and Multiagent Systems (AAMAS)*, 2005. — *Introduces a layered affect model mapping events to emotions to moods via PAD (Pleasure-Arousal-Dominance) space; the direct architectural model for Daimon's three-dimensional affect representation.*
- [KANERVA-1996] Kanerva, P. "Binary Spatter-Coding of Ordered K-tuples." *Artificial Neural Networks (ICANN)*. Springer, 1996. — *Introduces binary spatter codes for representing structured data in high-dimensional binary vectors; the encoding scheme used for CorticalState's hyperdimensional computing representation.*
- [KANERVA-2009] Kanerva, P. "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation." *Cognitive Computation*, 1(2), 2009. — *Overview of hyperdimensional computing where concepts are represented as high-dimensional vectors with algebraic operations; the framework for encoding CorticalState snapshots into compact similarity-searchable vectors for the WorldModelHistory.*

---
