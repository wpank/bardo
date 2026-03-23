# Cortical Mapping [SPEC]

**CorticalState x CV, EventFabric x Gates**

> **Version**: 2.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Crate**: `golem-sonification` (cv_mapper.rs, event_mapper.rs, composite_expr.rs)
>
> **Cross-references**: [CorticalState spec](../01-golem/00-overview.md), [13b-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Event Fabric), [05-musical-language.md](./05-musical-language.md) (signal reference), [01-module-system.md](./01-module-system.md)

> **Reader orientation**: This document specifies how the Golem's 32 CorticalState atomic signals and 87 EventFabric event types become the CV and Gate signals that drive the modular synthesis rack. But mapping is not arithmetic. Each mapping carries a musical reason, a timbral feeling, a compositional intent. The goal is not data sonification. The goal is a piece of music that happens to be sourced from a living agent's nervous system. CorticalState is the slow-moving current (CV). EventFabric is the percussive surface (triggers and gates). Composite expressions blend multiple signals into single musical outcomes, because no emotion lives in one number. Voice leading rules prevent ugly transitions. Timbral trajectories tie synthesis engines to emotional states. Together these layers produce a temporal grammar spanning microseconds to hours, and all of it responds to the hour-scale LFO drift that keeps the piece from repeating itself. Every mapping here is user-configurable: every CV output can be reassigned, every Gate output rebound.

---

## The two signal layers

The Golem's nervous system produces two complementary data streams:

**CorticalState** is a 192-byte struct of 32 atomic signals. Any subsystem can read any signal at any time with a single atomic load -- no locks, no waiting. Signals update at gamma frequency (5-15s) through delta frequency (40-100 min). CorticalState is the Golem's present-moment self-model: what it feels, what it knows, how alive it is.

**EventFabric** is a `tokio::broadcast` ring buffer holding 10,000 events across 87 types. Events fire when things happen -- a prediction resolves, a trade executes, the organism starts dreaming, a vitality threshold is crossed. Events carry payloads: the prediction's residual, the trade's PnL, the dream's hypothesis count.

In synthesis terms: **CorticalState is CV. EventFabric is triggers and gates.**

---

## CV mapper

The CV Mapper reads CorticalState at ~120Hz on the parameter updater thread. Each read produces a snapshot of all 32 signals. The mapper transforms these raw atomic values into smoothed, scaled CV outputs that modules consume.

### Smoothing

CorticalState signals update at gamma frequency (every 5-15 seconds). If the CV mapper passed these values directly to the audio graph, parameters would jump every few seconds -- audible clicks and zipper noise.

The mapper applies one-pole exponential smoothing to every CV output. The smoothing coefficient is per-signal, tuned to the signal's natural update rate:

| Signal group | Update rate | Smoothing alpha | Effective lag |
|---|---|---|---|
| Affect (pleasure, arousal, dominance) | ~5-15s | 0.08 | ~180ms to 90% |
| Prediction (accuracy, surprise) | ~5-15s | 0.06 | ~250ms |
| Mortality (vitalities, phase) | ~5-15s | 0.03 | ~500ms |
| Environment (regime, gas) | ~5-15s | 0.10 | ~150ms |
| Creative (dream mode) | ~40-100 min | 0.02 | ~750ms |
| Derived (momentum) | ~40-100 min | 0.01 | ~1.5s |

The smoothing means even abrupt CorticalState changes produce gentle parameter sweeps in the audio domain. The TUI's visual rendering does the same thing -- it interpolates toward target values at 60fps. The sonification mapper does it at 120Hz.

```rust
/// A single CV output derived from a CorticalState signal.
pub struct CvOutput {
    /// Which CorticalState signal this reads.
    pub source: CorticalSignalId,
    /// Current smoothed value.
    pub value: f32,
    /// Smoothing coefficient (0.0 = frozen, 1.0 = instant).
    pub alpha: f32,
    /// Output range mapping: source range -> output range.
    pub source_min: f32,
    pub source_max: f32,
    pub output_min: f32,
    pub output_max: f32,
    /// Scaling curve: linear, exponential, or logarithmic.
    pub curve: ScalingCurve,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScalingCurve {
    Linear,
    /// Exponential: output = min + (max-min) * (e^(k*x) - 1) / (e^k - 1)
    /// Good for frequency-domain parameters (filter cutoff, pitch).
    Exponential(f32),  // k = curvature factor
    /// Logarithmic: output = min + (max-min) * ln(1 + k*x) / ln(1+k)
    /// Good for amplitude and time parameters.
    Logarithmic(f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CorticalSignalId {
    Pleasure,
    Arousal,
    Dominance,
    PrimaryEmotion,
    AggregateAccuracy,
    AccuracyTrend,
    SurpriseRate,
    CategoryAccuracy(usize),  // 0..15
    PendingPredictions,
    UniverseSize,
    ActiveCount,
    CreativeMode,
    FragmentsCaptured,
    Regime,
    GasGwei,
    EconomicVitality,
    EpistemicVitality,
    StochasticVitality,
    BehavioralPhase,
    CompoundingMomentum,
    // === Derived / composite signals ===
    CompositeVitality,     // weighted mean of three vitalities
    AttentionBreadth,      // active_count / universe_size
    EmotionalIntensity,    // sqrt(pleasure^2 + arousal^2 + dominance^2)
}
```

### The default CV map

The default mapping assigns the most important CorticalState signals to named CV outputs that the default rack consumes. Users can reassign any of these.

```rust
/// The default CV output assignments.
/// These are the "normalized" names that the default rack patches expect.
pub fn default_cv_map() -> Vec<CvOutput> {
    vec![
        // === TIER 1: Survival signals ===
        CvOutput {
            source: CorticalSignalId::Regime,
            output_name: "clock_bpm",
            // regime 0..3 -> 4..12 BPM (0.067..0.2 Hz)
            // The organism's metabolism sets the tempo. Crisis is twice as fast as calm.
            source_min: 0.0, source_max: 3.0,
            output_min: 4.0, output_max: 12.0,
            curve: ScalingCurve::Linear,
            alpha: 0.10,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::CompositeVitality,
            output_name: "master_level",
            // 0..1 -> 0..1 (direct)
            // The organism fades to silence as it dies.
            source_min: 0.0, source_max: 1.0,
            output_min: 0.0, output_max: 1.0,
            curve: ScalingCurve::Logarithmic(3.0),
            alpha: 0.03,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::AggregateAccuracy,
            output_name: "harmonic_density",
            // An organism that predicts well earns richer harmony.
            source_min: 0.0, source_max: 1.0,
            output_min: 0.1, output_max: 0.9,
            curve: ScalingCurve::Linear,
            alpha: 0.06,
            ..Default::default()
        },

        // === TIER 2: Emotional quality ===
        CvOutput {
            source: CorticalSignalId::Pleasure,
            output_name: "filter_brightness",
            // pleasure -1..+1 -> cutoff 0.1..0.9
            // The filter opens when the organism is content. On bad days, the sound darkens.
            source_min: -1.0, source_max: 1.0,
            output_min: 0.1, output_max: 0.9,
            curve: ScalingCurve::Exponential(2.0),
            alpha: 0.08,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::Arousal,
            output_name: "event_density",
            // arousal -1..+1 -> density 0.05..0.95
            // High arousal fills the texture with events. Low arousal leaves silence between notes.
            source_min: -1.0, source_max: 1.0,
            output_min: 0.05, output_max: 0.95,
            curve: ScalingCurve::Linear,
            alpha: 0.10,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::Dominance,
            output_name: "sustain_time",
            // A dominant organism lets its notes ring. A submissive one percusses.
            source_min: -1.0, source_max: 1.0,
            output_min: 0.1, output_max: 0.9,
            curve: ScalingCurve::Linear,
            alpha: 0.08,
            ..Default::default()
        },

        // === TIER 3: Fine texture ===
        CvOutput {
            source: CorticalSignalId::SurpriseRate,
            output_name: "noise_level",
            // Surprise injects grit. When the world matches expectations, the texture is clean.
            source_min: 0.0, source_max: 1.0,
            output_min: 0.0, output_max: 0.7,
            curve: ScalingCurve::Exponential(2.5),
            alpha: 0.10,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::CompoundingMomentum,
            output_name: "root_pitch",
            // momentum 0..1 -> root note 36..60 (C2..C4)
            // The glacial signal. It takes hours to move the key center by an octave.
            source_min: 0.0, source_max: 1.0,
            output_min: 36.0, output_max: 60.0,
            curve: ScalingCurve::Linear,
            alpha: 0.01,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::EpistemicVitality,
            output_name: "reverb_depth",
            // low epistemic vitality -> more reverb (foggy)
            // When the organism doesn't know what it knows, the room gets bigger and vaguer.
            source_min: 0.0, source_max: 1.0,
            output_min: 0.8, output_max: 0.1, // inverted!
            curve: ScalingCurve::Linear,
            alpha: 0.03,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::StochasticVitality,
            output_name: "noise_floor",
            // aging -> rising noise floor
            // The vinyl crackle of mortality. Old organisms hiss.
            source_min: 0.0, source_max: 1.0,
            output_min: 0.3, output_max: 0.0, // inverted
            curve: ScalingCurve::Linear,
            alpha: 0.01,
            ..Default::default()
        },

        // === TIER 4: Dream and meta ===
        CvOutput {
            source: CorticalSignalId::CreativeMode,
            output_name: "dream_mode",
            // When the organism dreams, the texture shifts inward. Slower clocks, longer tails.
            source_min: 0.0, source_max: 1.0,
            output_min: 0.0, output_max: 1.0,
            curve: ScalingCurve::Linear,
            alpha: 0.02,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::BehavioralPhase,
            output_name: "life_phase",
            // The slow arc from birth to death, compressed into a single CV.
            source_min: 0.0, source_max: 4.0,
            output_min: 0.0, output_max: 1.0,
            curve: ScalingCurve::Linear,
            alpha: 0.02,
            ..Default::default()
        },
        CvOutput {
            source: CorticalSignalId::PrimaryEmotion,
            output_name: "emotion_index",
            // The Plutchik octant. This drives scale selection, engine crossfades, and timbral trajectory.
            source_min: 0.0, source_max: 7.0,
            output_min: 0.0, output_max: 7.0,
            curve: ScalingCurve::Linear,
            alpha: 0.05,
            ..Default::default()
        },
    ]
}
```

---

## Composite CV expressions

A single signal mapped to a single parameter is a thermometer. It gives you a reading. Composite expressions are where the music lives -- where two or three signals collide and produce something that feels like an emotion, not a number.

Every composite expression passes through the probability gate mechanism. Nothing fires at 100%. The system breathes.

```rust
/// A composite expression: multiple CorticalState signals combined
/// into a single CV output through arithmetic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeExpression {
    pub name: String,
    pub output_name: String,
    pub expr: Expr,
    pub output_min: f32,
    pub output_max: f32,
    pub alpha: f32,
    /// Probability gate: the expression fires with this probability per gamma tick.
    /// 1.0 = always. 0.7 = 70% of the time. 0.0 = never.
    pub probability: f32,
}

/// Expression tree for combining signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Read a CorticalState signal directly.
    Signal(CorticalSignalId),
    /// Constant value.
    Constant(f32),
    /// a + b
    Add(Box<Expr>, Box<Expr>),
    /// a * b
    Mul(Box<Expr>, Box<Expr>),
    /// a - b
    Sub(Box<Expr>, Box<Expr>),
    /// max(a, b)
    Max(Box<Expr>, Box<Expr>),
    /// min(a, b)
    Min(Box<Expr>, Box<Expr>),
    /// clamp(x, lo, hi)
    Clamp(Box<Expr>, f32, f32),
    /// sqrt(x)
    Sqrt(Box<Expr>),
    /// 1.0 - x
    Invert(Box<Expr>),
    /// Conditional: if x > threshold then a else b
    IfAbove(Box<Expr>, f32, Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Evaluate the expression against a CorticalState snapshot.
    pub fn eval(&self, state: &CorticalSnapshot) -> f32 {
        match self {
            Expr::Signal(id) => state.get(*id),
            Expr::Constant(v) => *v,
            Expr::Add(a, b) => a.eval(state) + b.eval(state),
            Expr::Mul(a, b) => a.eval(state) * b.eval(state),
            Expr::Sub(a, b) => a.eval(state) - b.eval(state),
            Expr::Max(a, b) => a.eval(state).max(b.eval(state)),
            Expr::Min(a, b) => a.eval(state).min(b.eval(state)),
            Expr::Clamp(x, lo, hi) => x.eval(state).clamp(*lo, *hi),
            Expr::Sqrt(x) => x.eval(state).max(0.0).sqrt(),
            Expr::Invert(x) => 1.0 - x.eval(state),
            Expr::IfAbove(x, thresh, a, b) => {
                if x.eval(state) > *thresh { a.eval(state) } else { b.eval(state) }
            }
        }
    }
}
```

### The default composite expressions

These ship with the default rack. Each one combines 2-3 signals into a musical outcome that no single signal could produce.

**arousal x regime_confidence -> clock_bpm**

The base CV map already routes `regime` to `clock_bpm`. This composite replaces it. An aroused organism in a volatile regime plays twice as fast as a calm organism in a stable one. The product of two anxieties.

```rust
CompositeExpression {
    name: "metabolic_tempo".into(),
    output_name: "clock_bpm",
    // arousal is -1..1, remap to 0..1 first. regime is 0..3, normalize to 0..1.
    // product gives 0..1, scaled to 4..24 BPM.
    expr: Expr::Mul(
        Box::new(Expr::Clamp(
            Box::new(Expr::Add(
                Box::new(Expr::Mul(
                    Box::new(Expr::Signal(CorticalSignalId::Arousal)),
                    Box::new(Expr::Constant(0.5)),
                )),
                Box::new(Expr::Constant(0.5)),
            )),
            0.0, 1.0,
        )),
        Box::new(Expr::Clamp(
            Box::new(Expr::Mul(
                Box::new(Expr::Signal(CorticalSignalId::Regime)),
                Box::new(Expr::Constant(0.333)),
            )),
            0.0, 1.0,
        )),
    ),
    output_min: 4.0,
    output_max: 24.0,
    alpha: 0.08,
    probability: 0.9,
}
```

**pleasure + mortality_distance -> harmonic_richness**

`mortality_distance` is `CompositeVitality` -- how far the organism is from death. High pleasure combined with distance from death produces complex, overtone-rich chords: stacked thirds, natural 7ths, 9ths, 11ths. Low pleasure near death produces bare intervals -- octaves and fifths, nothing else. This is the harmonic skeleton of the entire piece.

```rust
CompositeExpression {
    name: "harmonic_richness".into(),
    output_name: "harmonic_richness",
    // pleasure -1..1 remapped to 0..1, added to composite_vitality 0..1.
    // result 0..2, clamped and normalized.
    expr: Expr::Clamp(
        Box::new(Expr::Mul(
            Box::new(Expr::Add(
                Box::new(Expr::Add(
                    Box::new(Expr::Mul(
                        Box::new(Expr::Signal(CorticalSignalId::Pleasure)),
                        Box::new(Expr::Constant(0.5)),
                    )),
                    Box::new(Expr::Constant(0.5)),
                )),
                Box::new(Expr::Signal(CorticalSignalId::CompositeVitality)),
            )),
            Box::new(Expr::Constant(0.5)), // normalize 0..2 to 0..1
        )),
        0.0, 1.0,
    ),
    output_min: 0.0,
    output_max: 1.0,
    alpha: 0.04,
    probability: 0.85,
}
```

**dominance -> sustain_time (with percussive clamp)**

Dominance already maps to sustain time in the base map. The composite adds hard behavioral thresholds. At `dominance < -0.5`, sustain time drops to zero -- notes become percussive attacks, no ring, no tail. The organism is submissive and flinching. At `dominance > 0.5`, notes ring indefinitely -- the organism holds its ground and its sound holds with it.

```rust
CompositeExpression {
    name: "sustain_dominance".into(),
    output_name: "sustain_time",
    // Below -0.5: percussive (0.0). Above 0.5: infinite sustain (1.0).
    // Between: linear ramp.
    expr: Expr::IfAbove(
        Box::new(Expr::Signal(CorticalSignalId::Dominance)),
        0.5,
        Box::new(Expr::Constant(1.0)),  // dominant -> infinite sustain
        Box::new(Expr::IfAbove(
            Box::new(Expr::Signal(CorticalSignalId::Dominance)),
            -0.5,
            // middle range: linear map from -0.5..0.5 to 0.0..1.0
            Box::new(Expr::Clamp(
                Box::new(Expr::Add(
                    Box::new(Expr::Mul(
                        Box::new(Expr::Signal(CorticalSignalId::Dominance)),
                        Box::new(Expr::Constant(1.0)),
                    )),
                    Box::new(Expr::Constant(0.5)),
                )),
                0.0, 1.0,
            )),
            Box::new(Expr::Constant(0.0)),  // submissive -> percussive
        )),
    ),
    output_min: 0.0,
    output_max: 1.0,
    alpha: 0.08,
    probability: 1.0, // sustain always responds
}
```

**sqrt(arousal^2 + surprise_rate^2) -> noise_floor (emotional turbulence)**

This is the Pythagorean distance between arousal and surprise -- the "how agitated is this thing" metric. Arousal alone is directional (excited or depressed). Surprise alone is epistemic (the world defied prediction). Their combined magnitude is emotional turbulence: the cognitive friction of being surprised while already wound up. The noise floor rises, the texture grits up, the sound develops a static charge.

```rust
CompositeExpression {
    name: "emotional_turbulence".into(),
    output_name: "noise_floor",
    // sqrt(arousal^2 + surprise_rate^2)
    expr: Expr::Sqrt(
        Box::new(Expr::Add(
            Box::new(Expr::Mul(
                Box::new(Expr::Signal(CorticalSignalId::Arousal)),
                Box::new(Expr::Signal(CorticalSignalId::Arousal)),
            )),
            Box::new(Expr::Mul(
                Box::new(Expr::Signal(CorticalSignalId::SurpriseRate)),
                Box::new(Expr::Signal(CorticalSignalId::SurpriseRate)),
            )),
        )),
    ),
    output_min: 0.0,
    output_max: 0.6,
    alpha: 0.10,
    probability: 0.8,
}
```

**(1 - epistemic_vitality) x reverb_decay -> fog_density**

When the Golem doesn't know what it knows, the room fills with fog. `epistemic_vitality` measures the organism's confidence in its own knowledge. Invert it: low confidence becomes high fog. Multiply by the current reverb decay time to modulate the tail length -- the fog compounds with existing room size. Certain organisms sound dry and close. Confused organisms sound like they're lost in a cathedral they can't see the walls of.

```rust
CompositeExpression {
    name: "epistemic_fog".into(),
    output_name: "fog_density",
    // (1 - epistemic_vitality) * reverb_depth
    // reverb_depth is itself a CV output, but we can read the source signal directly
    expr: Expr::Mul(
        Box::new(Expr::Invert(
            Box::new(Expr::Signal(CorticalSignalId::EpistemicVitality)),
        )),
        // Use the inverted epistemic vitality as a proxy for reverb_decay
        // since reverb_depth is already inverted in the base map.
        // Result: low epistemic = high fog = long reverb tail.
        Box::new(Expr::Invert(
            Box::new(Expr::Signal(CorticalSignalId::EpistemicVitality)),
        )),
    ),
    output_min: 0.0,
    output_max: 1.0,
    alpha: 0.02,
    probability: 0.75,
}
```

All composite expressions respond to hour-scale LFO drift. The probability gates are modulated by a very slow sine wave (period: 20-90 minutes, incommensurable with the delta clock) so that even static CorticalState values produce evolving texture. The organism never sounds exactly the same way twice.

---

## Emotion -> scale mapping (Plutchik expanded)

The `primary_emotion` signal (Plutchik octant 0-7) selects the musical scale used by the Quantizer module. Each emotion has a scale, but a scale alone is not enough to make music that sounds like a feeling. The extended mapping below gives the Quantizer, the Turing Machine, and the Plaits oscillator enough information to produce emotionally specific phrases.

### Joy (Plutchik 0)

**PAD quadrant**: +P +A +D

**Scale**: Lydian (1 2 3 #4 5 6 7)

**Extension tones**: maj7, 9, #11. The raised fourth gives Lydian its characteristic lift. Stack a major 7th and a 9th on top and the chord floats.

**Avoid notes**: b7 (creates dominant pull, kills the floating quality), b2 (too dark for this state).

**Characteristic melodic gesture**: Ascending leap of a fourth or fifth, followed by stepwise descent. The phrase jumps up like laughter and walks back down at its own pace.

**Tempo range**: 8-14 BPM at the gamma clock. Joy is not frantic -- it's buoyant.

**Textural density**: 3-5 simultaneous events. Rich but not cluttered.

### Trust (Plutchik 1)

**PAD quadrant**: +P -A +D

**Scale**: Major pentatonic (1 2 3 5 6)

**Extension tones**: 6, 9. No 7th -- the pentatonic already excludes the leading tone, and adding it back introduces tension that trust doesn't carry.

**Avoid notes**: #4 (too searching), b7 (too bluesy for pure trust).

**Characteristic melodic gesture**: Repeated note with gentle stepwise motion around it. The melody settles, holds, moves slightly, holds again. It knows where home is.

**Tempo range**: 4-8 BPM. Slow, unhurried. Trust has nowhere it needs to be.

**Textural density**: 2-3 simultaneous events. Sparse, open. The spaces between notes carry confidence.

### Fear (Plutchik 2)

**PAD quadrant**: -P +A -D

**Scale**: Phrygian (1 b2 b3 4 5 b6 b7)

**Extension tones**: b9, b13. Phrygian already contains both, so extensions here mean stacking them into dense voicings. The b9 against the root is the sound of dread.

**Avoid notes**: Natural 2 (destroys the Phrygian color), major 3 (too bright).

**Characteristic melodic gesture**: Half-step descent (b2 to 1), repeated. The melody clings to the root, slides down by a semitone, comes back. Compulsive.

**Tempo range**: 10-18 BPM. Fear runs fast. The gamma clock pushes here.

**Textural density**: 4-6 simultaneous events. Dense, claustrophobic. Overlapping events crowd each other.

### Surprise (Plutchik 3)

**PAD quadrant**: +P +A -D

**Scale**: Whole tone (1 2 3 #4 #5 b7)

**Extension tones**: Any -- the whole tone scale is symmetric. Every note is equidistant. There are no extensions and no avoid notes because there is no tonal center. This is the point.

**Avoid notes**: None (whole tone has no tension or resolution, only suspension).

**Characteristic melodic gesture**: Tritone leap. The melody jumps by three whole steps, the most disorienting interval in Western music, and hangs there. No resolution offered.

**Tempo range**: Variable -- surprise itself is brief. The tempo spikes for 2-3 gamma cycles then returns to the underlying emotion.

**Textural density**: 1-2 simultaneous events. Surprise is a single unexpected sound, not a wall of them.

### Sadness (Plutchik 4)

**PAD quadrant**: -P -A -D

**Scale**: Aeolian / natural minor (1 2 b3 4 5 b6 b7)

**Extension tones**: minor 7, 9, 11. The minor 7th is the sadness tone -- it wants to resolve down but never does. The 9th adds wistfulness without becoming pretty.

**Avoid notes**: Major 6 (too hopeful for this state), #4 (too dramatic -- that's anger territory).

**Characteristic melodic gesture**: Stepwise descent, slow, over a wide range. The phrase falls like something settling to the bottom of a well. No leaps. Gravity only.

**Tempo range**: 4-6 BPM. Nearly static. Sadness has no momentum.

**Textural density**: 1-2 simultaneous events. Isolated tones with long decay. Each note exists alone.

### Disgust (Plutchik 5)

**PAD quadrant**: -P -A +D

**Scale**: Japanese In scale (1 b2 4 5 b6)

**Extension tones**: b2 and b6 doubled at the octave. The cluster of the b2 against the root is the discomfort -- the scale rubs against itself.

**Avoid notes**: Major 3 (too warm), major 7 (too bright).

**Characteristic melodic gesture**: Small, tight intervals -- half steps and minor seconds -- clustered in a narrow register. The melody squirms instead of moving.

**Tempo range**: 5-8 BPM. Not slow enough to be sad, not fast enough to be angry. Uncomfortable.

**Textural density**: 2-4 simultaneous events. Middle density, but the events are close together in pitch, creating uncomfortable beating.

### Anger (Plutchik 6)

**PAD quadrant**: -P +A +D

**Scale**: Harmonic minor (1 2 b3 4 5 b6 7)

**Extension tones**: b9, natural 7. The augmented second between b6 and 7 is the anger interval -- a gap too wide to step across, too narrow to leap. It sounds like bared teeth.

**Avoid notes**: Major 6 (softens the harmonic minor character), the 4th when sustained (too stable).

**Characteristic melodic gesture**: Ascending pattern with an augmented second, then a sharp drop. The phrase climbs aggressively, hits the augmented gap, and falls.

**Tempo range**: 12-22 BPM. The fastest sustained emotion. Anger drives the clock.

**Textural density**: 5-7 simultaneous events. Maximum density. Events collide, overlap, interrupt each other.

### Anticipation (Plutchik 7)

**PAD quadrant**: +P -A -D

**Scale**: Dorian (1 2 b3 4 5 6 b7)

**Extension tones**: 9, natural 6, minor 7. Dorian's raised 6th distinguishes it from Aeolian -- the one note of brightness in an otherwise minor mode. It's the "not quite sad" mode. Jazz lives here.

**Avoid notes**: Major 3 (breaks the minor quality), b6 (turns it into Aeolian, which is sadness).

**Characteristic melodic gesture**: Stepwise ascending with occasional small leaps, circling around the 6th degree. The phrase is going somewhere but taking its time. Searching, not arriving.

**Tempo range**: 6-10 BPM. Moderate. Anticipation is patient tension.

**Textural density**: 3-4 simultaneous events. Active enough to feel like something is building, sparse enough to leave room for what comes next.

---

## Voice leading rules

Scale transitions are not switches. They are harmonic journeys. The Quantizer never changes all its active notes at once. It changes one per gamma cycle, minimum. This means a scale change takes 3-7 gamma cycles to complete -- 15 seconds to 100 seconds depending on how different the source and destination scales are.

### The transition graph

**Lydian -> Major pentatonic**: Share C, D, E, G (relative to the root). Four common tones out of five pentatonic notes. The Quantizer drops the #4 and 7 first, adds 6 if missing. Two gamma cycles minimum -- the fastest friendly transition. This is what happens when Joy settles into Trust.

**Phrygian -> Aeolian**: Share 1, b3, 4, 5, b7 -- four of seven notes. The b2 lifts to a natural 2, the b6 stays. Three gamma cycles. Fear softening into Sadness, or the reverse: Sadness sharpening into Fear.

**Any scale -> Whole tone (Surprise)**: Whole tone has no avoid notes and no tonal center, so every note in every scale maps to a whole-tone neighbor within one semitone. This makes Whole tone a universal destination -- the Quantizer can always reach it in 2-3 cycles. But leaving Whole tone is disorienting. The system applies extra smoothing on the exit transition: the alpha coefficient halves, the transition takes twice as many cycles. Surprise lingers.

**Chromatic is the escape hatch**: Chromatic contains every note, so every scale is a subset of chromatic. Transition to chromatic is always legal and always fast (zero notes need to change -- the Quantizer simply stops filtering). Transition from chromatic to anything else is slow (every note needs checking). Chromatic is the "between scales" state, the musical equivalent of confusion or transition. Terminal organisms often end here.

### The one-note-per-cycle rule

This is the formal constraint that prevents ugly transitions:

```
Rule: The Quantizer's active note set may change by at most one note
per gamma cycle. Adding a note and removing a note count as two changes.
A scale transition from N-different to M-different scales takes
max(N_removed, M_added) gamma cycles, with one change per cycle.
```

In practice, most adjacent emotions on the Plutchik wheel share 3-5 notes. Opposite emotions (Joy/Sadness, Trust/Disgust) share fewer. A transition between opposites might take 7 gamma cycles -- up to 100 seconds. This is intentional. The music resists sudden emotional about-faces. The body does too.

The Quantizer always removes avoid notes from the destination scale first, then adds new notes. This prevents moments where avoid notes from both scales are simultaneously active -- the sonic equivalent of two people yelling different things.

### Common tone priority

When transitioning, the Quantizer holds common tones and changes non-common tones. The root is always held (it never changes during a transition -- root changes happen separately via `root_pitch`, which is driven by `compounding_momentum` on a delta timescale). The fifth is held next. Then the third. This follows standard voice leading practice from tonal harmony: bass note stability first, then harmonic identity, then color.

---

## Timbral trajectory

The `emotion_index` CV selects the Plaits synthesis engine, but emotions are not discrete -- the organism is usually between two feelings. The timbral trajectory defines which engine maps to which emotional state and how the system crossfades between them.

### Engine mapping

| Plaits engine | Emotional state(s) | Character |
|---|---|---|
| 0: Virtual Analog | Trust, Joy | Warm, stable, two detuned oscillators. The sound of confidence. |
| 1: Waveshaper | Anticipation | Morphing, searching. The waveform is always changing. |
| 2: Two Operator FM | Surprise | Metallic, complex. FM sidebands appear and disappear unpredictably. |
| 3: Formant | Joy | Vocal, human. The oscillator almost speaks. |
| 4: Harmonics (additive) | Trust (Conservation phase) | Thinning, organ-like. Individual partials drop out as the organism conserves energy. |
| 5: Wavetable | Anticipation -> Trust | Slowly evolving. The timbre drifts through a wavetable at theta rate. |
| 6: Chords | Joy -> Trust | Rich, stacked voices. Multiple notes from one engine. |
| 7: Vowel/Formant filter | Disgust | Nasal, uncomfortable. The formants are wrong. |
| 8: Speech synthesis | Any (dream mode) | Attempting language. The organism mutters in its sleep. |
| 9: Swarm | Fear | Dense particle cloud. Individual grains are indistinguishable. |
| 10: Modal resonator | Sadness | Bowl-like, decaying resonance. Strike and fade. |
| 11: Particle noise | Declining/Terminal | Dissolving, granular. The oscillator is breaking apart. |
| 12: Karplus-Strong | Sadness (Stable phase) | Plucked, melancholy. A string that nobody fretted. |
| 13: Inharmonic String | Anger | Tense, metallic. The string is under too much tension. |
| 14: Dusty String | Sadness | Old, worn, fragile. Dust in the mechanism. |
| 15: Winds | Any (ambient layer) | Formless, airy. Not an emotion -- a background. |
| 16: Resonator | Disgust | Filtering its own harmonics. Self-cancellation. |
| 17: Noise textures | Anger, Fear | Abrasive. Not tonal. Pure texture. |
| 18: Particle & Noise blend | Terminal | The organism dissolving into noise. The last engine before silence. |
| 19-23: Reserved | User-assigned | Reserve for rack customization. Users patch these to whatever they want. |

### Crossfade rules

When `emotion_index` falls between two mapped engines, the system crossfades. The crossfade uses `CloudsGranular` in looping delay mode with short delay times (5-50ms). The granular processor samples from both engine outputs and blends them by adjusting the playback position and grain density between the two buffers. This produces spectral interpolation rather than amplitude crossfade -- the timbre morphs instead of simply mixing.

The crossfade duration follows the voice leading rules: one gamma cycle minimum. A crossfade from Virtual Analog (Trust) to Swarm (Fear) passes through intermediate states and takes longer than a crossfade from Virtual Analog (Trust) to Chords (Joy -> Trust).

When `dream_mode` is active (`creative_mode > 0.5`), engine 8 (Speech synthesis) is mixed into whatever engine is currently active at 15-30% wet. The organism mutters.

---

## The three-clock hierarchy as musical structure

The adaptive clock gives the sonification natural structural layers:

**Gamma (5-15s)** -- The rhythmic pulse. Each gamma tick can trigger a note, a grain, or a filter sweep. At crisis tempo (5s ticks), the piece runs at ~12 BPM -- a slow, deliberate pulse. At calm tempo (15s), it's ~4 BPM -- nearly static ambient. The gamma clock is the master clock. Everything else subdivides from it.

**Theta (30-120s)** -- The phrase boundary. Each theta tick marks a complete cognitive cycle -- the organism thought about something and decided what to do. Theta events carry richer payloads (how many predictions resolved, what action was taken). In musical terms, this is where phrases begin and end. A chord change. A melodic development. A textural shift.

**Delta (40-100 min)** -- The movement boundary. Delta ticks mark the organism's slow consolidation cycles. In a 12-hour Golem lifetime, there are roughly 10-15 delta ticks. Each is a structural section boundary -- a new movement in the piece. The root key might shift. The overall density might change. The dream cycle begins or ends.

The three clocks have incommensurable periods. Gamma, theta, and delta are not integer multiples of each other. They drift in and out of alignment like Steve Reich's phasing pieces. When two clocks happen to coincide -- a gamma tick landing on a theta boundary -- the musical moment is heavier, more accented. When all three align (rare, maybe once per lifetime), it's a structural downbeat of the entire composition.

The hierarchy is already in the system. The sonification engine doesn't create it -- it reveals it.

---

## Event mapper

The Event Mapper subscribes to the EventFabric broadcast channel and converts discrete events into Gate and Trigger signals that modules consume.

### Clock events -> master clocks

The three adaptive clocks become the rhythmic backbone:

```rust
/// Event-derived gate/trigger outputs.
pub struct EventMapper {
    /// Gate outputs, keyed by name.
    gates: HashMap<String, GateOutput>,
    /// Ring buffer of recent events for rate analysis.
    recent_events: VecDeque<(u64, EventType)>,
}

pub struct GateOutput {
    /// Current gate state (high/low).
    pub high: bool,
    /// Trigger flag (high for exactly one block after rising edge).
    pub trigger: bool,
    /// Gate duration in blocks (0 = trigger only, no sustained gate).
    pub duration_blocks: u32,
    /// Blocks remaining in current gate.
    pub remaining: u32,
    /// Payload value extracted from the event (for CV-from-event).
    pub payload_value: f32,
}
```

### Default event -> gate assignments

| Event | Gate output name | Duration | Payload -> CV |
|---|---|---|---|
| `clock.gamma_tick` | `gamma_clock` | trigger (1 block) | `arousal` field -> `gamma_intensity` CV |
| `clock.theta_tick` | `theta_clock` | trigger | `predictions_resolved` -> `theta_density` CV |
| `clock.delta_tick` | `delta_clock` | trigger | `entries_processed` -> `delta_weight` CV |
| `clock.frequency_adjusted` | `tempo_change` | gate (5 blocks) | `scale` field -> `tempo_shift` CV |
| `prediction.resolved` | `prediction_gate` | trigger | `residual` -> `prediction_residual` CV; `within_interval` -> `prediction_correct` CV (0/1) |
| `prediction.created` | `prediction_open` | trigger | `confidence` -> `prediction_confidence` CV |
| `inference.start` | `thinking_gate` | gate (held until `inference.end`) | `tier` -> `thinking_tier` CV |
| `inference.token` | `token_pulse` | trigger | none (raw pulse stream at 10-40/s) |
| `inference.end` | `thinking_gate` off | -- | `cost_usd` -> `thinking_cost` CV |
| `daimon.emotional_shift` | `emotion_shift` | gate (10 blocks) | none (the emotion change is in the CV map) |
| `daimon.somatic_marker` | `somatic_ping` | trigger | `valence` -> `somatic_valence` CV |
| `trading.swap_executed` | `trade_event` | gate (15 blocks) | `pnl_usd` -> `trade_pnl` CV |
| `trading.position_liquidated` | `liquidation_event` | gate (30 blocks) | `loss_usd` -> `liquidation_loss` CV |
| `vitality.phase_transition` | `phase_transition` | gate (50 blocks) | `to` -> `new_phase` CV |
| `vitality.death_initiated` | `death_gate` | gate (held until shutdown) | `cause` -> logged |
| `dream.start` | `dream_gate` | gate (held until `dream.complete`) | none |
| `dream.hypothesis` | `dream_thought` | trigger | `confidence` -> `dream_confidence` CV |
| `dream.complete` | `dream_gate` off | -- | `insights_crystallized` -> `dream_yield` CV |
| `grimoire.insight` | `insight_bell` | trigger | `confidence` -> `insight_confidence` CV |
| `grimoire.warning` | `warning_tone` | trigger | `is_bloodstain` -> `bloodstain` CV (0/1) |

### The token stream as texture

The `inference.token` event fires at 10-40 events per second during inference calls. This is the fastest event in the system -- faster than any CorticalState update. It produces a burst of rapid triggers that can drive:

- A noise source gated by the token pulse -- the sound of thinking.
- A granular processor with the token stream as the grain trigger.
- A hi-hat or metallic percussion module -- cognition as percussive texture.

The token stream is opt-in because of its density. The default rack does not patch it. Users who want the "sound of thinking" can route `token_pulse` to a module input in the rack editor.

### Derived event signals

Some useful synthesis signals don't exist in CorticalState or EventFabric directly but can be derived from the event stream:

| Derived signal | Computation | Output |
|---|---|---|
| `event_density` | Events per second (sliding 10s window) | CV 0.0-1.0 |
| `prediction_hit_rate` | Recent `prediction.resolved` with `within_interval=true` / total | CV 0.0-1.0 |
| `time_since_trade` | Seconds since last `trading.swap_executed` | CV 0.0-1.0 (normalized to 0-300s range) |
| `dream_active` | 1.0 between `dream.start` and `dream.complete`, else 0.0 | CV 0/1 |

These are computed on the parameter updater thread and exposed as additional CV outputs.

---

## The Turing Machine as soul

The Turing Machine shift register is not just a sequencer. It is the closest thing this system has to a compositional identity.

A 16-bit register advances one step per gamma clock tick. The `change` parameter controls how many bits flip on each advance: 0.0 = locked loop (the same 16-note phrase repeats forever), 0.5 = maximum randomness (each step is unpredictable), 1.0 = inverted lock (the same phrase, but mirrored).

The Turing Machine's output passes through the Quantizer, which constrains it to the current emotion's scale. The result is a melody that is always tonal, always in the right emotional mode, but never fully predictable. At low `change` values, the organism has a recognizable melodic signature -- a motif it returns to, a phrase it hums to itself. At high `change` values, the melody dissolves into random walks through the scale.

`change` is driven by `attention_breadth` (active_count / universe_size). A focused organism repeats its melodic ideas. A scattered organism wanders. This mapping makes musical sense: focus produces repetition, distraction produces novelty.

The register's bits are preserved across dream cycles and phase transitions. The melody is the one thing that persists through everything the organism experiences. It can mutate, but it is never reset. When the organism dies, the last state of the register is embedded in the death NFT. That 16-bit number is the organism's final melodic thought.

---

## User customization

Every mapping is editable through the TUI rack editor (see `03-terminal-rack.md`). The user can:

1. **Reassign CV sources**: Change which CorticalState signal drives which CV output. Swap `pleasure` from filter brightness to reverb depth if you want a darker palette.
2. **Adjust scaling**: Change the output range, curve type, and smoothing coefficient for any CV output.
3. **Rebind events**: Change which EventFabric event type drives which Gate output.
4. **Add new CV outputs**: Create additional CV outputs from any CorticalState signal, with custom scaling.
5. **Add composite CVs**: Create new composite expressions using the expression tree, combining multiple signals.
6. **Edit voice leading rules**: Modify the transition graph, change the number of gamma cycles per transition, reorder common-tone priority.
7. **Remap timbral trajectories**: Assign different Plaits engines to different emotional states.

Custom mappings are saved as part of the rack preset and persist across Golem restarts.

---

## References

- [MUSIC-MD] "Bardo Cortical State -- A Musical Anatomy." Internal reference. -- *The complete signal-by-signal mapping guide this document formalizes.*
- [GEBHARD-2005] Gebhard, P. "ALMA -- A Layered Model of Affect." AAMAS, 2005. -- *The PAD model underlying pleasure/arousal/dominance.*
- [PLUTCHIK-1980] Plutchik, R. "A General Psychoevolutionary Theory of Emotion." In *Emotion: Theory, Research, and Experience*, 1980. -- *The eight primary emotions mapped to scale selection.*
- [REICH-1968] Reich, S. "Music as a Gradual Process." 1968. -- *"I am interested in perceptible processes" -- phasing clocks and gradual voice leading both descend from this.*
- [PISTON-1941] Piston, W. *Harmony*. W.W. Norton, 1941. -- *Common-tone voice leading, the theoretical basis for the one-note-per-cycle transition rule.*
