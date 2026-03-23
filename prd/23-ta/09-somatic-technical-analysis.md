# Somatic Technical Analysis: Learned Gut Feelings for Autonomous DeFi Agents [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Source**: `tmp/research/witness-research/new/ta/09-somatic-technical-analysis.md`
>
> **Depends on**: Doc 0 (Overview), Doc 1 (HDC Encoding), Daimon (PAD model)

**Audience:** Systems engineers integrating affect-driven decision systems into the Bardo runtime; researchers working on embodied cognition in autonomous agents.

---

> **Reader orientation:** This document applies Damasio's somatic marker hypothesis to the Golem (mortal autonomous DeFi agent), building learned "gut feelings" that bind TA patterns to emotional outcomes. It belongs to the **TA research layer** (Doc 9 of 10) and covers HDC binding between pattern hypervectors and PAD (Pleasure-Arousal-Dominance) affect states, somatic map construction, sub-100ns gut-feeling retrieval, attention auction modulation, and cross-generational transfer through Thanatopsis (the four-phase death protocol). You should understand embodied cognition concepts and the Daimon (the Golem's affect engine). For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Abstract

The Bardo runtime has two systems that should talk to each other but don't. The Daimon generates emotional responses to market events as PAD vectors (pleasure, arousal, dominance). The TA subsystem detects patterns in on-chain data. Neither system knows what the other learned. The Daimon doesn't know which specific TA pattern triggered its anxiety. The TA system doesn't know which patterns the Golem has learned to fear from bitter experience.

Antonio Damasio's somatic marker hypothesis (1994) proposes that decision-making is guided by bodily sensations associated with past outcomes. When you see a pattern linked to past pain, your body generates an aversive signal before conscious deliberation begins. This pre-attentive filtering cuts through combinatorial explosion: instead of evaluating every option, the brain marks most as "avoid" and deliberates only over the survivors.

This document builds somatic markers for Golems. Each marker is an HDC binding between a TA pattern's hypervector and the PAD affect state the Golem experienced when that pattern resolved. Over time, these markers accumulate into a somatic map: a single 1,280-byte hypervector that holographically encodes the Golem's entire history of pattern-outcome-emotion associations. The map enables sub-100ns gut-feeling retrieval, modulates attention auction bids, accelerates learning from feared contexts, detects emotional traps via anti-somatic markers, and transfers across generations through death testaments.

---

## The problem [SPEC]

Two subsystems. One gap.

The Daimon is Bardo's affect engine. It operates on three temporal layers. Emotion is fast, updating per-event with a decay half-life of seconds. Mood is medium, shifting per-session over minutes. Personality is slow, evolving across generations. Each layer produces a PAD vector in [-1, 1]^3. Low pleasure, high arousal, low dominance reads as fear. High pleasure, low arousal, high dominance reads as confidence. These vectors bias every decision the Golem makes: risk tolerance, position sizing, attention allocation, learning rate.

The TA subsystem detects patterns. Doc 1 encodes every TA signal as a 10,240-bit BSC hypervector. Doc 6 evolves these patterns as organisms in an ecosystem, with fitness scores, mutation, and selection pressure. Doc 3 metabolizes signals as living units competing for an attention budget. Doc 8 tests whether detected patterns are adversarial fabrications. The TA pipeline runs from raw block ingestion through pattern detection, scoring, and action recommendation.

These two systems share no state. When the Daimon registers anxiety during a market downturn, it doesn't know whether the anxiety came from a head-and-shoulders pattern on ETH/USDC, a spike in Aave utilization, or the combination of both. When the TA system detects a rising wedge that historically preceded profitable breakouts, it doesn't know that the last three times the Golem acted on rising wedges, it lost money and the Daimon recorded sustained negative affect.

A human trader bridges this gap with gut feelings. Years of watching charts build visceral associations: a specific candlestick formation triggers a knot in the stomach, a particular volume profile creates a surge of excitement. These are not rational analyses. They are learned somatic markers that fire before deliberation, pre-filtering the option space so that conscious reasoning can focus on the survivors.

The Golem needs this bridge. Without it, the Daimon's emotional state floats free of TA context, reacting to aggregate market conditions rather than specific patterns. And the TA system operates as a cold pattern matcher, unable to draw on the Golem's accumulated emotional history to weight which patterns deserve attention. Every generation starts from scratch, re-learning which patterns are trustworthy through costly trial and error, unable to inherit the hard-won gut feelings of its predecessors.

Damasio called this the "as-if body loop": the brain reactivates the somatic state associated with a past experience without actually re-experiencing the full situation. The brain doesn't re-run the full simulation. It recalls the conclusion, the somatic tag, the feeling. That shortcut is what makes experts fast. A chess grandmaster doesn't evaluate every legal move. Pattern recognition triggers a somatic marker that says "this position is winning" or "this position is dangerous," and conscious analysis focuses on the positions that survived the gut check.

For the Golem, the as-if body loop is an HDC unbind: given a pattern, retrieve the affect that was associated with it in the past, and inject that affect into the Daimon before deliberation begins. The unbind costs 3 nanoseconds. The deliberative analysis at the theta tick costs milliseconds to seconds. The somatic shortcut doesn't replace deliberation; it focuses it. Instead of evaluating 50 detected patterns with equal weight, the Golem evaluates 5 that passed the somatic filter with full attention and 45 others with minimal resources.

This has a direct analog in DeFi market microstructure. An autonomous agent that deliberates equally over every detected pattern cannot keep up with block time. Ethereum produces a block every 12 seconds. A Golem monitoring 10 pools across 4 protocols might detect 30-50 pattern candidates per block. Full deliberative analysis of each candidate at the theta tick would require more time than the block interval allows. The somatic system provides the attentional triage that makes real-time autonomous operation feasible.

There is a second problem that somatic markers solve: the cold-start problem across generations. When a Golem dies and its successor boots, the successor has no market experience. Without somatic inheritance, it must learn from scratch which patterns to trust and which to fear. With inheritance, it starts with a compressed summary of its predecessor's emotional history, a somatic disposition that biases attention from the very first gamma tick. The successor doesn't know why it feels uneasy about a specific LP pattern. But the unease is real, encoded in the inherited somatic map, and it correctly steers attention toward caution until the successor's own experience confirms or contradicts the inherited marker.

---

## Mathematical foundations [SPEC]

### The somatic marker as HDC binding

A somatic marker is the learned association between a TA pattern and an emotional outcome. In HDC terms, it is a binding:

```
somatic_marker = bind(pattern_hv, affect_hv)
```

where `pattern_hv` is the TA pattern's 10,240-bit hypervector (produced by the encoding schemes in Doc 1) and `affect_hv` is the PAD emotional state encoded as a hypervector.

The binding operation is XOR. It is its own inverse: `bind(bind(a, b), a) = b`. This means given a somatic marker and a pattern, you can recover the affect. Given a somatic marker and an affect, you can recover the pattern. The marker is a reversible association, exactly the property needed for somatic retrieval.

The result is a 10,240-bit hypervector (1,280 bytes) that encodes "this pattern feels like THIS."

Why binding and not bundling? Bundling (majority vote) encodes co-occurrence: "these things happened together." Binding (XOR) encodes association: "this thing is linked to that thing." A somatic marker is an association, not a co-occurrence. The pattern and the affect are not merely present in the same context; one evokes the other. Binding's invertibility is what makes retrieval work. Given the marker and the pattern, you recover the affect. Given the marker and the affect, you recover the pattern. Bundling has no such inverse.

The choice also interacts with the somatic map's bundle structure. The map is a bundle of bindings. This two-level composition (bind individual associations, then bundle them) is the HDC idiom for associative memory. Each individual association is recoverable from the map (with noise proportional to the number of bundled entries), and the map as a whole is a single fixed-size object regardless of how many associations it contains.

### PAD encoding

The PAD model (Mehrabian and Russell, 1974) represents emotional states as vectors in a three-dimensional space:

- **Pleasure** [-1, 1]: the valence axis. Positive is pleasure, negative is distress.
- **Arousal** [-1, 1]: the activation axis. High is agitated, low is calm.
- **Dominance** [-1, 1]: the control axis. High is in-control, low is overwhelmed.

To encode a PAD vector as a hypervector, discretize each dimension into buckets and use role-filler binding:

```
affect_hv = bind(R_pleasure, V_pleasure_bucket)
          + bind(R_arousal, V_arousal_bucket)
          + bind(R_dominance, V_dominance_bucket)
```

where `+` denotes majority-vote bundling and `R_pleasure`, `R_arousal`, `R_dominance` are role vectors from the item memory.

The bucket granularity matters. Too few buckets and the system conflates distinct emotional states. Too many and the SNR drops (more fillers competing in the codebook). Seven buckets per dimension works well: strongly negative, moderately negative, slightly negative, neutral, slightly positive, moderately positive, strongly positive. With 7 buckets per dimension, the codebook holds 21 filler vectors plus 3 role vectors.

For ordinal dimensions, use thermometer encoding. A "moderately positive" pleasure reading encodes as the bundle of `V_slightly_positive` and `V_moderately_positive`, giving it partial similarity to adjacent states. This means similar emotional states produce similar affect hypervectors, which is what we want: mild fear and strong fear should not be orthogonal.

The resulting affect hypervector has SNR = sqrt(10240 / 5) = 45.3 for recovering any individual PAD component from a 6-term bundle (3 role-filler pairs, each thermometer-encoded with 2 active terms).

### Decoding PAD from a hypervector

To decode, unbind each role and find the nearest bucket vector:

```
pleasure_estimate = unbind(affect_hv, R_pleasure)
nearest_bucket = argmax_v similarity(pleasure_estimate, V_pleasure_bucket_v)
```

Repeat for arousal and dominance. Total decode cost: 3 unbinds (XOR, ~3ns each) and 3 x 7 = 21 similarity comparisons (~10ns each). Full PAD decode: ~220ns. Fast enough for the gamma tick.

### Somatic retrieval

When the TA system detects a pattern, the somatic engine performs retrieval:

```
retrieved_affect = unbind(somatic_marker, pattern_hv)
```

Because `somatic_marker = bind(pattern_hv, affect_hv)` and XOR is its own inverse:

```
unbind(bind(pattern_hv, affect_hv), pattern_hv)
    = pattern_hv XOR affect_hv XOR pattern_hv
    = affect_hv
```

The retrieved affect is exactly the original affect hypervector (for a single marker). Compare it against a library of affect prototypes:

| Prototype | Pleasure | Arousal | Dominance | Meaning |
|-----------|----------|---------|-----------|---------|
| Fear | -0.7 | 0.8 | -0.5 | High-alert aversion |
| Confidence | 0.7 | 0.2 | 0.8 | Calm control |
| Excitement | 0.6 | 0.8 | 0.4 | Energized approach |
| Disgust | -0.8 | 0.3 | 0.6 | Rejection with control |
| Curiosity | 0.3 | 0.6 | 0.3 | Approach with activation |
| Resignation | -0.4 | -0.6 | -0.7 | Low-energy withdrawal |

The similarity between the retrieved affect and each prototype determines the gut feeling. If `similarity(retrieved_affect, fear_prototype) > 0.6`, the Golem has a bad feeling about this pattern.

Cost: 1 unbind (~3ns) + 6 prototype comparisons (~60ns) = ~63ns total. Well within the gamma tick budget.

### The somatic map

Individual markers are useful but expensive to store and query one at a time. The somatic map bundles all markers into a single hypervector:

```
somatic_map = bundle(marker_1, marker_2, ..., marker_N)
```

where each `marker_i = bind(pattern_i_hv, affect_i_hv)`.

The bundled map is still a single 1,280-byte hypervector. It holographically encodes every pattern-affect association the Golem has ever formed. Query it the same way as a single marker:

```
gut_response = unbind(somatic_map, pattern_hv)
```

The result is `affect_hv + noise`, where the noise comes from the other N-1 markers in the bundle. The SNR for recovering the correct affect:

```
SNR = sqrt(D / (N - 1))
```

At D = 10,240:

| Markers bundled | SNR | Reliable retrieval? |
|-----------------|-----|---------------------|
| 10 | 33.7 | Yes, high confidence |
| 50 | 14.6 | Yes, reliable |
| 100 | 10.1 | Yes, adequate |
| 500 | 4.5 | Marginal |
| 1000 | 3.2 | Below threshold |

A Golem that has formed 100 somatic markers can retrieve any individual marker's affect with SNR 10.1, well above the discrimination threshold of ~4.0. A Golem with 500 markers approaches the limit. Beyond that, the map saturates and individual markers blur together.

This saturation is actually desirable behavior. A very experienced Golem (many markers) has a somatic map that responds more to the aggregate emotional tenor of a pattern class than to any single memory. The map becomes a generalized gut feeling, not a specific recollection. The detailed associations live in the individual marker store; the map provides fast, approximate retrieval.

### Marker strength and temporal decay

Not all somatic markers are equally strong. A marker formed from a single event is weaker than one reinforced by repeated experience. Marker strength evolves:

```
strength(t + 1) = strength(t) * decay + reinforcement * (1 - decay)
```

where `decay` controls how quickly unreinforced markers fade and `reinforcement` is a value in [0, 1] proportional to outcome magnitude.

Strong markers contribute more to the somatic map through weighted bundling:

```
somatic_map = weighted_bundle(
    (marker_1, strength_1),
    (marker_2, strength_2),
    ...
)
```

In BSC, weighted bundling is achieved by repeating a vector proportionally to its weight before taking the majority vote. A marker with strength 0.8 gets 4x the repetitions of one with strength 0.2. The effective capacity of the map depends on the strength distribution: a few very strong markers dominate, while many weak markers contribute background texture.

### Somatic interference and marker competition

When two patterns are detected simultaneously and both have somatic markers, their retrieved affects compete. This is somatic interference: the Daimon receives two pre-attentive affect injections in the same gamma tick, and they may conflict.

Consider a gamma tick where the Golem detects both a bullish RSI divergence (somatic marker: approach, strength 0.6) and a spike in Aave utilization (somatic marker: avoid, strength 0.4). The approach marker pushes pleasure positive and arousal down. The avoid marker pushes pleasure negative and arousal up. The net injection depends on strength-weighted summation:

```
net_delta.pleasure = 0.6 * approach_delta.pleasure + 0.4 * avoid_delta.pleasure
net_delta.arousal  = 0.6 * approach_delta.arousal  + 0.4 * avoid_delta.arousal
net_delta.dominance = 0.6 * approach_delta.dominance + 0.4 * avoid_delta.dominance
```

The result is a mixed affect state: slightly positive pleasure, elevated arousal, moderate dominance. In human terms, the Golem feels cautiously optimistic but alert. This mixed state produces moderate attention bids for both patterns and a theta-tick deliberation cycle that weighs both signals.

Somatic interference is not a bug. It is information. When two strong markers fire in opposite directions, the Golem's emotional state becomes conflicted, which is exactly what should happen when the evidence is ambiguous. The elevated arousal from the conflict increases the learning rate (via signal metabolism), meaning the Golem pays extra attention to ambiguous situations and learns from them faster. Ambiguity produces arousal, arousal produces learning, learning reduces future ambiguity. This is a self-correcting loop.

### The TA-affect feedback loop

The full cybernetic loop has six steps:

**Step 1: Pattern detection.** The TA subsystem (Doc 1, Doc 6) detects a pattern and produces `pattern_hv`.

**Step 2: Somatic retrieval.** The somatic engine unbinds `pattern_hv` from the somatic map. If the retrieved affect is similar to a known prototype (similarity > 0.55), a somatic response fires.

**Step 3: Pre-attentive affect injection.** The somatic response modifies the Daimon's current emotional state. If the gut feeling is "fear," the Daimon's pleasure drops and arousal rises. This happens before deliberative reasoning at the theta tick, biasing downstream decisions.

**Step 4: Attention modulation.** The Daimon bids in the attention auction (VCG mechanism). Strongly-marked patterns get higher bids. A pattern that triggers strong fear or strong confidence gets more cognitive resources than one that triggers no somatic response. The somatic engine doesn't decide what to do about the pattern; it decides how much attention the pattern deserves.

**Step 5: Outcome resolution.** The pattern eventually resolves (the predicted move happens or doesn't, the trade is profitable or not). The outcome is recorded.

**Step 6: Marker update.** If the outcome confirms the marker (feared pattern was bad, or confident pattern was good), the marker strengthens. If the outcome contradicts the marker (feared pattern was actually fine), the marker weakens. Over many cycles, markers calibrate to empirical reality.

This loop means the Golem's gut feelings are not static biases. They are learned associations that update with experience, creating a feedback system where emotion and evidence mutually constrain each other.

### Anti-somatic markers

Here is the problem with learned gut feelings: they can be wrong. Worse, they can be wrong in a way that feels right.

Consider a pattern that consistently triggers positive affect (high pleasure, moderate arousal, high dominance). The Golem feels confident when it sees this pattern. But empirically, the pattern loses money. The emotional response is a trap.

This is not hypothetical. In DeFi, adversarial actors craft signals designed to trigger positive responses (Doc 8). Memetic parasites (Innovation 06) are patterns that replicate well through the epistemic ecosystem because they feel good, not because they predict outcomes. A Golem that trusts its gut feelings without verification will get exploited.

Anti-somatic markers flag these traps. The detection rule:

```
anti_somatic(pattern) =
    affect_valence(pattern) > 0     // feels positive
    AND empirical_return(pattern) < threshold  // but loses money
    AND sample_count(pattern) > min_samples    // with enough data to be confident
```

The `min_samples` threshold prevents premature labeling. A pattern that felt good and lost money once might be noise. A pattern that felt good and lost money eight out of ten times is an emotional trap.

Once flagged, an anti-somatic marker encodes the inverse affect:

```
anti_marker = bind(pattern_hv, inverse(affect_hv))
```

where `inverse(affect_hv)` is the PAD vector with all signs flipped: what felt like confidence now encodes as what should feel like wariness. The anti-marker enters the somatic map alongside regular markers.

When the Golem next encounters this pattern, somatic retrieval pulls back a mixed signal: the original positive affect partially canceled by the inverse. The `SomaticResponse` struct flags this as `GutFeeling::Distrust`, meaning "your gut says approach, but your experience says don't."

The connection to Doc 8 (adversarial robustness) is direct. Anti-somatic markers are the emotional immune system. Doc 8 detects signal manipulation at the statistical level. Anti-somatic markers detect it at the experiential level: patterns that bypass rational analysis by triggering positive affect. The two detection systems complement each other. A sophisticated adversary might evade statistical detection while still triggering learned positive associations. The anti-somatic system catches what the statistical system misses.

The connection to Innovation 06 (memetic ecology) is equally direct. Epistemic parasites spread through the pattern ecosystem because they replicate well, not because they predict well. Anti-somatic markers are antibodies against these parasites. A Golem that has been burned by a memetic parasite develops an anti-somatic marker that suppresses future replication of that pattern class.

### Somatic transfer via death testament

When a Golem dies, its somatic map transfers to successors through the Grimoire's death testament system. The transfer has three components:

**The aggregate somatic map.** A single 1,280-byte hypervector encoding the generalized gut feelings of the deceased Golem. This transfers immediately and cheaply. The successor starts with a baseline somatic disposition inherited from its predecessor.

**High-strength individual markers.** The top-K strongest markers (typically K = 20-50), preserved at full resolution. These are the most reinforced pattern-affect associations, the ones the deceased Golem felt most strongly about. They enter the successor's recent marker store.

**The anti-somatic set.** All anti-somatic markers transfer in full. These are too expensive to re-learn from scratch; each one represents a pattern that cost the predecessor money. The successor inherits the full library of "don't trust this feeling" markers.

The aggregate map loses fidelity through bundling compression. If the deceased Golem had 300 markers, the bundled map has SNR ~5.8 for any individual marker. The successor can't precisely recall what the predecessor felt about a specific pattern, but it can sense the general emotional tenor: "patterns like this one tend to feel bad." This is analogous to inherited temperament versus learned fear. The specific fears require reinforcement through the successor's own experience.

Older Golems (further from their founding ancestor) have richer somatic maps because they inherit from longer lineages. The mortality system creates a natural progression: young Golems have thin somatic maps and rely more on deliberative reasoning. Old Golems have thick somatic maps and can make faster, more gut-driven decisions, with more attention budget freed for novel situations.

The fidelity loss through bundling compression is a feature, not a defect. If the somatic map transferred at full resolution, successor Golems would inherit specific fears and confidences that might not apply to the current market regime. The lossy transfer acts as regularization: the successor gets the general shape of the predecessor's somatic disposition without being bound to specific associations that may have been regime-dependent. The successor starts with a disposition ("patterns like this tend to feel bad") rather than a doctrine ("this exact pattern is bad"), and refines through its own experience. This is analogous to the difference between genetic predisposition and learned behavior in biological organisms. The genes encode a bias; the organism's life experience tunes it.

---

## Architecture [SPEC]

### SomaticEngine

The somatic engine sits between the TA subsystem and the Daimon. It receives pattern detections from the TA pipeline, performs somatic retrieval, and injects pre-attentive affect into the Daimon. It receives outcome notifications from the execution pipeline and updates markers accordingly.

The engine has five responsibilities:

1. **Somatic retrieval**: given a detected pattern, produce a gut feeling in <100ns.
2. **Marker management**: create, strengthen, weaken, and retire somatic markers.
3. **Anti-somatic detection**: identify patterns where affect and outcome diverge.
4. **Map maintenance**: periodically consolidate recent markers into the aggregate map.
5. **Testament production**: serialize the somatic state for death testament transfer.

### Heartbeat integration

The somatic engine hooks into the Golem's heartbeat at every temporal tier:

**Gamma tick (5-15 seconds, perception).** When the TA subsystem detects patterns, the somatic engine performs retrieval on each detected pattern. Cost: ~63ns per pattern, with typical gamma ticks detecting 0-5 patterns. Total somatic processing per gamma tick: <500ns. The somatic responses feed into the Daimon's emotion layer and the attention auction.

**Theta tick (30-120 seconds, cognition).** After the deliberative reasoning cycle completes and actions are chosen, the somatic engine records which patterns influenced the decision. This creates the ground truth for later marker updates: "at this theta tick, pattern X triggered a fear response, and the Golem chose to reduce exposure."

**Delta tick (~50 theta ticks, consolidation).** The engine consolidates recent markers into the aggregate somatic map. Markers older than a configurable window (default: 100 delta ticks) that have not been activated get retired from the recent store and exist only in the bundled map. The engine also runs anti-somatic detection over the recent marker set, flagging any markers where affect and empirical outcome have diverged beyond threshold.

**NREM dream (offline consolidation).** The dream system replays pattern-outcome-emotion triples from episodic memory. Each replay is a simulated experience that the somatic engine processes as if it were live: retrieve the marker, compare retrieved affect to the replayed outcome, update the marker. NREM dreams strengthen correct associations and weaken incorrect ones, compressing days of market experience into minutes of offline processing.

**REM dream (creative/counterfactual).** The dream system generates counterfactual scenarios: "what if this pattern had resolved differently?" The somatic engine processes these counterfactuals, testing whether anti-somatic markers are still warranted. If a pattern that was marked as an emotional trap would have been profitable under the counterfactual conditions, the anti-somatic marker's strength decreases. REM dreams prevent the Golem from becoming permanently gun-shy about patterns that were traps in one regime but opportunities in another.

### Data flow

```
[TA Pattern Detection]
        |
        v
[Somatic Retrieval] -- unbind from somatic_map
        |
        v
[SomaticResponse: gut_feeling + strength + anti_somatic flag]
        |
        +---> [Daimon: pre-attentive PAD injection]
        |
        +---> [Attention Auction: bid modulation]
        |
        v
[Theta Tick: deliberative reasoning with somatic bias]
        |
        v
[Action taken or not taken]
        |
        v
[Outcome resolved: profit/loss + magnitude]
        |
        v
[Marker Update: strengthen / weaken / create / promote to anti-somatic]
        |
        v
[Delta Tick: consolidate into somatic_map, run anti-somatic scan]
        |
        v
[Dream: replay and counterfactual refinement]
```

---

## Rust implementation [SPEC]

### Configuration

```rust
/// Configuration for the somatic engine.
pub struct SomaticConfig {
    /// Number of buckets per PAD dimension for discretization.
    /// Default: 7 (strongly_neg through strongly_pos).
    pub pad_buckets: usize,

    /// Maximum number of recent individual markers to retain
    /// before forcing consolidation into the aggregate map.
    pub max_recent_markers: usize,

    /// Decay factor for marker strength per delta tick.
    /// Range: [0.0, 1.0]. Higher values mean slower decay.
    pub strength_decay: f64,

    /// Minimum similarity to an affect prototype to trigger
    /// a somatic response. Below this, the response is Neutral.
    pub response_threshold: f64,

    /// Minimum sample count before a marker can be promoted
    /// to anti-somatic status.
    pub anti_somatic_min_samples: u64,

    /// Threshold for affect-outcome divergence to flag anti-somatic.
    /// Positive affect valence + empirical return below this = trap.
    pub anti_somatic_return_threshold: f64,

    /// Number of high-strength markers to include in death testament.
    pub testament_top_k: usize,

    /// Delta ticks of inactivity before a recent marker is retired
    /// to the aggregate map only.
    pub marker_retirement_ticks: u64,

    /// Weight of inherited somatic map vs. fresh experience.
    /// Range: [0.0, 1.0]. 1.0 = fully trust inheritance.
    pub inheritance_weight: f64,
}

impl Default for SomaticConfig {
    fn default() -> Self {
        Self {
            pad_buckets: 7,
            max_recent_markers: 200,
            strength_decay: 0.98,
            response_threshold: 0.55,
            anti_somatic_min_samples: 8,
            anti_somatic_return_threshold: -0.05,
            testament_top_k: 30,
            marker_retirement_ticks: 100,
            inheritance_weight: 0.6,
        }
    }
}
```

### Core types

```rust
use std::collections::HashMap;

/// A PAD emotional state vector.
#[derive(Clone, Debug)]
pub struct PadVector {
    /// Pleasure/valence axis. Range: [-1.0, 1.0].
    pub pleasure: f64,
    /// Arousal/activation axis. Range: [-1.0, 1.0].
    pub arousal: f64,
    /// Dominance/control axis. Range: [-1.0, 1.0].
    pub dominance: f64,
}

impl PadVector {
    pub fn new(pleasure: f64, arousal: f64, dominance: f64) -> Self {
        Self {
            pleasure: pleasure.clamp(-1.0, 1.0),
            arousal: arousal.clamp(-1.0, 1.0),
            dominance: dominance.clamp(-1.0, 1.0),
        }
    }

    /// Valence shorthand: positive = approach, negative = avoid.
    pub fn valence(&self) -> f64 {
        self.pleasure
    }

    /// Flip all PAD dimensions. Used for anti-somatic marker construction.
    pub fn inverse(&self) -> Self {
        Self {
            pleasure: -self.pleasure,
            arousal: -self.arousal,
            dominance: -self.dominance,
        }
    }

    /// Weighted blend between two PAD states. Alpha = 0.0 returns self,
    /// alpha = 1.0 returns other.
    pub fn blend(&self, other: &PadVector, alpha: f64) -> PadVector {
        let a = alpha.clamp(0.0, 1.0);
        PadVector::new(
            self.pleasure * (1.0 - a) + other.pleasure * a,
            self.arousal * (1.0 - a) + other.arousal * a,
            self.dominance * (1.0 - a) + other.dominance * a,
        )
    }
}

/// Outcome of a TA pattern's prediction.
#[derive(Clone, Debug)]
pub struct PatternOutcome {
    /// The pattern that resolved.
    pub pattern_hv: Hypervector,
    /// Empirical return: positive = profitable, negative = loss.
    /// Normalized to [-1.0, 1.0] by the maximum expected return.
    pub empirical_return: f64,
    /// Magnitude of the outcome, independent of direction.
    /// Used for reinforcement scaling.
    pub magnitude: f64,
    /// Timestamp (block number or heartbeat tick).
    pub resolved_at: u64,
}

/// Combined pattern-outcome-emotion triple for dream replay.
#[derive(Clone, Debug)]
pub struct PatternOutcomeEmotion {
    pub pattern_hv: Hypervector,
    pub outcome: PatternOutcome,
    /// The PAD state the Golem was in when this pattern was active.
    pub affect_at_detection: PadVector,
    /// The PAD state after outcome resolution.
    pub affect_at_resolution: PadVector,
}

/// Exponential moving average for tracking empirical accuracy.
#[derive(Clone, Debug)]
pub struct ExponentialAverage {
    value: f64,
    alpha: f64,
    count: u64,
}

impl ExponentialAverage {
    pub fn new(alpha: f64) -> Self {
        Self {
            value: 0.0,
            alpha,
            count: 0,
        }
    }

    pub fn update(&mut self, sample: f64) {
        if self.count == 0 {
            self.value = sample;
        } else {
            self.value = self.alpha * sample + (1.0 - self.alpha) * self.value;
        }
        self.count += 1;
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}
```

### The somatic marker

```rust
/// A single somatic marker: the learned association between
/// a TA pattern and the emotional state it produced.
pub struct SomaticMarker {
    /// The TA pattern's hypervector encoding.
    pub pattern_hv: Hypervector,

    /// The associated affect, encoded as a hypervector.
    pub affect_hv: Hypervector,

    /// The bound marker: bind(pattern_hv, affect_hv).
    pub marker_hv: Hypervector,

    /// Decoded PAD vector for fast access without unbinding.
    pub affect_pad: PadVector,

    /// Positive = approach, negative = avoid.
    pub valence: f64,

    /// How strong this gut feeling is. Range: [0.0, 1.0].
    /// Increases with reinforcement, decays without it.
    pub strength: f64,

    /// Rolling average of whether the marker's emotional prediction
    /// matched the empirical outcome.
    pub empirical_accuracy: ExponentialAverage,

    /// Rolling average of empirical returns when this marker fired.
    pub empirical_return: ExponentialAverage,

    /// True if this marker has been flagged as an emotional trap.
    pub is_anti_somatic: bool,

    /// Tick when this marker was first created.
    pub created_at: u64,

    /// Last tick when somatic retrieval activated this marker.
    pub last_activated: u64,

    /// Total number of times this marker has been activated.
    pub activation_count: u64,
}

impl SomaticMarker {
    pub fn new(
        pattern_hv: Hypervector,
        affect_hv: Hypervector,
        affect_pad: PadVector,
        current_tick: u64,
    ) -> Self {
        let marker_hv = pattern_hv.bind(&affect_hv);
        let valence = affect_pad.valence();
        Self {
            pattern_hv,
            affect_hv,
            marker_hv,
            affect_pad,
            valence,
            strength: 0.3, // new markers start at moderate strength
            empirical_accuracy: ExponentialAverage::new(0.1),
            empirical_return: ExponentialAverage::new(0.1),
            is_anti_somatic: false,
            created_at: current_tick,
            last_activated: current_tick,
            activation_count: 0,
        }
    }

    /// Strengthen this marker based on a confirming outcome.
    pub fn reinforce(&mut self, outcome_magnitude: f64) {
        let reinforcement = outcome_magnitude.clamp(0.0, 1.0);
        self.strength = (self.strength + reinforcement * (1.0 - self.strength))
            .clamp(0.0, 1.0);
    }

    /// Weaken this marker based on a contradicting outcome.
    pub fn weaken(&mut self, contradiction_magnitude: f64) {
        let penalty = contradiction_magnitude.clamp(0.0, 1.0);
        self.strength = (self.strength * (1.0 - penalty * 0.3))
            .clamp(0.0, 1.0);
    }

    /// Apply temporal decay (called each delta tick).
    pub fn decay(&mut self, decay_factor: f64) {
        self.strength *= decay_factor;
    }

    /// Record an activation (somatic retrieval matched this marker).
    pub fn activate(&mut self, tick: u64) {
        self.last_activated = tick;
        self.activation_count += 1;
    }
}
```

### The PAD codebook

```rust
/// Encodes and decodes PAD vectors as HDC hypervectors.
///
/// Uses role-filler binding with thermometer encoding
/// for ordinal bucket values.
pub struct PadCodebook {
    /// Role vectors for each PAD dimension.
    role_pleasure: Hypervector,
    role_arousal: Hypervector,
    role_dominance: Hypervector,

    /// Bucket filler vectors per dimension.
    /// Key: (dimension_name, bucket_index). Value: random HV.
    bucket_vectors: HashMap<(PadDimension, usize), Hypervector>,

    /// Number of buckets per dimension.
    num_buckets: usize,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum PadDimension {
    Pleasure,
    Arousal,
    Dominance,
}

impl PadCodebook {
    /// Create a new codebook with `num_buckets` per dimension.
    /// All role and filler vectors are drawn randomly.
    pub fn new(num_buckets: usize, rng: &mut impl rand::Rng) -> Self {
        let role_pleasure = Hypervector::random(rng);
        let role_arousal = Hypervector::random(rng);
        let role_dominance = Hypervector::random(rng);

        let mut bucket_vectors = HashMap::new();
        for dim in [PadDimension::Pleasure, PadDimension::Arousal, PadDimension::Dominance] {
            for i in 0..num_buckets {
                bucket_vectors.insert((dim, i), Hypervector::random(rng));
            }
        }

        Self {
            role_pleasure,
            role_arousal,
            role_dominance,
            bucket_vectors,
            num_buckets,
        }
    }

    /// Map a continuous value in [-1.0, 1.0] to a bucket index.
    fn value_to_bucket(&self, value: f64) -> usize {
        let clamped = value.clamp(-1.0, 1.0);
        // Map [-1, 1] to [0, num_buckets - 1]
        let normalized = (clamped + 1.0) / 2.0; // [0, 1]
        let bucket = (normalized * self.num_buckets as f64) as usize;
        bucket.min(self.num_buckets - 1)
    }

    /// Thermometer-encode a bucket index: bundle this bucket and
    /// its immediate neighbor (if ordinal adjacency exists).
    fn thermometer_encode(
        &self,
        dim: PadDimension,
        bucket: usize,
    ) -> Hypervector {
        let primary = self.bucket_vectors[&(dim, bucket)].clone();
        if bucket > 0 {
            let neighbor = &self.bucket_vectors[&(dim, bucket - 1)];
            primary.bundle(&[neighbor.clone()])
        } else if bucket < self.num_buckets - 1 {
            let neighbor = &self.bucket_vectors[&(dim, bucket + 1)];
            primary.bundle(&[neighbor.clone()])
        } else {
            primary
        }
    }

    fn role_for(&self, dim: PadDimension) -> &Hypervector {
        match dim {
            PadDimension::Pleasure => &self.role_pleasure,
            PadDimension::Arousal => &self.role_arousal,
            PadDimension::Dominance => &self.role_dominance,
        }
    }

    /// Encode a PAD vector as a hypervector.
    pub fn encode_pad(&self, pad: &PadVector) -> Hypervector {
        let p_bucket = self.value_to_bucket(pad.pleasure);
        let a_bucket = self.value_to_bucket(pad.arousal);
        let d_bucket = self.value_to_bucket(pad.dominance);

        let p_filler = self.thermometer_encode(PadDimension::Pleasure, p_bucket);
        let a_filler = self.thermometer_encode(PadDimension::Arousal, a_bucket);
        let d_filler = self.thermometer_encode(PadDimension::Dominance, d_bucket);

        let p_bound = self.role_pleasure.bind(&p_filler);
        let a_bound = self.role_arousal.bind(&a_filler);
        let d_bound = self.role_dominance.bind(&d_filler);

        p_bound.bundle(&[a_bound, d_bound])
    }

    /// Decode a hypervector back to a PAD vector.
    /// Returns the centroid of the best-matching bucket for each dimension.
    pub fn decode_pad(&self, hv: &Hypervector) -> PadVector {
        let p_val = self.decode_dimension(hv, PadDimension::Pleasure);
        let a_val = self.decode_dimension(hv, PadDimension::Arousal);
        let d_val = self.decode_dimension(hv, PadDimension::Dominance);
        PadVector::new(p_val, a_val, d_val)
    }

    fn decode_dimension(&self, hv: &Hypervector, dim: PadDimension) -> f64 {
        let role = self.role_for(dim);
        let unbound = hv.bind(role); // XOR is its own inverse

        let mut best_sim = f64::NEG_INFINITY;
        let mut best_bucket = 0usize;

        for i in 0..self.num_buckets {
            let candidate = &self.bucket_vectors[&(dim, i)];
            let sim = unbound.similarity(candidate);
            if sim > best_sim {
                best_sim = sim;
                best_bucket = i;
            }
        }

        // Convert bucket index back to [-1, 1]
        let center = (best_bucket as f64 + 0.5) / self.num_buckets as f64;
        center * 2.0 - 1.0
    }
}
```

### Affect prototype library

```rust
/// A named affect prototype for classifying gut feelings.
pub struct AffectPrototype {
    pub name: String,
    pub pad: PadVector,
    pub hv: Hypervector,
    pub gut_feeling: GutFeeling,
}

/// The gut feeling classification.
#[derive(Clone, Debug, PartialEq)]
pub enum GutFeeling {
    /// Strong positive somatic marker. Approach with confidence.
    StrongApproach,
    /// Mild positive marker. Lean toward approach.
    MildApproach,
    /// No somatic association. Proceed to deliberative analysis.
    Neutral,
    /// Mild negative marker. Lean toward avoidance.
    MildAvoid,
    /// Strong negative marker. Avoid unless deliberation overrides.
    StrongAvoid,
    /// Anti-somatic: feels positive but empirically negative.
    /// The most dangerous category. Override gut with evidence.
    Distrust,
}

/// Build the standard affect prototype library.
pub fn build_affect_prototypes(codebook: &PadCodebook) -> Vec<AffectPrototype> {
    let prototypes = vec![
        ("fear", PadVector::new(-0.7, 0.8, -0.5), GutFeeling::StrongAvoid),
        ("anxiety", PadVector::new(-0.4, 0.6, -0.3), GutFeeling::MildAvoid),
        ("confidence", PadVector::new(0.7, 0.2, 0.8), GutFeeling::StrongApproach),
        ("mild_optimism", PadVector::new(0.3, 0.1, 0.4), GutFeeling::MildApproach),
        ("excitement", PadVector::new(0.6, 0.8, 0.4), GutFeeling::StrongApproach),
        ("curiosity", PadVector::new(0.3, 0.6, 0.3), GutFeeling::MildApproach),
        ("disgust", PadVector::new(-0.8, 0.3, 0.6), GutFeeling::StrongAvoid),
        ("resignation", PadVector::new(-0.4, -0.6, -0.7), GutFeeling::MildAvoid),
        ("calm", PadVector::new(0.2, -0.5, 0.5), GutFeeling::MildApproach),
        ("panic", PadVector::new(-0.9, 0.95, -0.8), GutFeeling::StrongAvoid),
    ];

    prototypes
        .into_iter()
        .map(|(name, pad, feeling)| {
            let hv = codebook.encode_pad(&pad);
            AffectPrototype {
                name: name.to_string(),
                pad,
                hv,
                gut_feeling: feeling,
            }
        })
        .collect()
}
```

### The somatic response

```rust
/// The result of somatic retrieval: a gut feeling about a pattern.
pub struct SomaticResponse {
    /// The classified gut feeling.
    pub gut_feeling: GutFeeling,

    /// How strongly the somatic marker fired. Range: [0.0, 1.0].
    pub strength: f64,

    /// True if this pattern has an active anti-somatic flag.
    pub is_anti_somatic: bool,

    /// The best-matching affect prototype name.
    pub matched_prototype: String,

    /// Similarity to the matched prototype.
    pub prototype_similarity: f64,

    /// Suggested PAD delta to inject into the Daimon.
    /// Scaled by strength: a weak marker produces a small delta.
    pub suggested_pad_delta: PadVector,
}

impl SomaticResponse {
    /// Returns true if this response should influence the attention auction.
    pub fn is_salient(&self) -> bool {
        self.strength > 0.2 && self.gut_feeling != GutFeeling::Neutral
    }

    /// Compute a bid modifier for the attention auction.
    /// Strongly-marked patterns get higher bids.
    /// Anti-somatic patterns get the highest bids (they need scrutiny).
    pub fn attention_bid_modifier(&self) -> f64 {
        let base = match self.gut_feeling {
            GutFeeling::StrongApproach => 1.5,
            GutFeeling::StrongAvoid => 1.8, // fear demands more attention than confidence
            GutFeeling::Distrust => 2.0,    // highest priority: emotional traps
            GutFeeling::MildApproach => 1.1,
            GutFeeling::MildAvoid => 1.3,
            GutFeeling::Neutral => 1.0,
        };
        // Scale by marker strength
        1.0 + (base - 1.0) * self.strength
    }
}
```

### The somatic engine

```rust
/// The somatic engine: manages gut feelings about TA patterns.
///
/// Sits between the TA subsystem and the Daimon. Receives pattern
/// detections, performs somatic retrieval, injects pre-attentive
/// affect, and updates markers based on outcomes.
pub struct SomaticEngine {
    /// The aggregate somatic map: a single HV encoding all
    /// pattern-affect associations via weighted bundling.
    somatic_map: Hypervector,

    /// Individual recent markers at full resolution.
    /// Newer markers live here until they are consolidated
    /// into the aggregate map at delta tick.
    recent_markers: Vec<SomaticMarker>,

    /// Anti-somatic markers: patterns flagged as emotional traps.
    /// Stored separately for fast lookup and full death testament transfer.
    anti_somatic_set: Vec<SomaticMarker>,

    /// PAD encoding/decoding codebook.
    pad_codebook: PadCodebook,

    /// Affect prototypes for classifying somatic retrievals.
    affect_prototypes: Vec<AffectPrototype>,

    /// Current heartbeat tick (updated externally).
    current_tick: u64,

    /// Total markers ever consolidated into the somatic map.
    /// Tracks effective capacity.
    consolidated_marker_count: usize,

    config: SomaticConfig,
}

impl SomaticEngine {
    pub fn new(config: SomaticConfig, rng: &mut impl rand::Rng) -> Self {
        let pad_codebook = PadCodebook::new(config.pad_buckets, rng);
        let affect_prototypes = build_affect_prototypes(&pad_codebook);
        Self {
            somatic_map: Hypervector::zero(),
            recent_markers: Vec::with_capacity(config.max_recent_markers),
            anti_somatic_set: Vec::new(),
            pad_codebook,
            affect_prototypes,
            current_tick: 0,
            consolidated_marker_count: 0,
            config,
        }
    }

    /// Perform somatic retrieval on a single detected pattern.
    ///
    /// Cost: ~63ns (1 unbind + 6-10 prototype comparisons).
    pub fn somatic_retrieval(&self, pattern_hv: &Hypervector) -> SomaticResponse {
        // Check anti-somatic set first (linear scan, typically <50 entries)
        let is_anti = self.anti_somatic_set.iter().any(|m| {
            pattern_hv.similarity(&m.pattern_hv) > 0.7
        });

        // Try recent markers first (higher resolution)
        let mut best_marker: Option<&SomaticMarker> = None;
        let mut best_sim = 0.0f64;
        for marker in &self.recent_markers {
            let sim = pattern_hv.similarity(&marker.pattern_hv);
            if sim > best_sim && sim > 0.6 {
                best_sim = sim;
                best_marker = Some(marker);
            }
        }

        // If a strong recent marker exists, use its precise affect
        if let Some(marker) = best_marker {
            if marker.strength > 0.1 {
                return self.classify_response(
                    &marker.affect_hv,
                    marker.strength * best_sim,
                    is_anti,
                );
            }
        }

        // Fall back to aggregate somatic map
        let retrieved_affect = self.somatic_map.bind(pattern_hv);

        // Check if the retrieved affect is meaningful (not just noise)
        let max_proto_sim = self.affect_prototypes.iter()
            .map(|p| retrieved_affect.similarity(&p.hv))
            .fold(0.0f64, f64::max);

        if max_proto_sim < self.config.response_threshold {
            // No somatic association exists for this pattern
            return SomaticResponse {
                gut_feeling: GutFeeling::Neutral,
                strength: 0.0,
                is_anti_somatic: false,
                matched_prototype: "none".to_string(),
                prototype_similarity: max_proto_sim,
                suggested_pad_delta: PadVector::new(0.0, 0.0, 0.0),
            };
        }

        // Estimate strength from map SNR based on consolidated count
        let estimated_strength = if self.consolidated_marker_count > 0 {
            let snr = (10_240.0 / self.consolidated_marker_count as f64).sqrt();
            (snr / 40.0).clamp(0.0, 1.0) // normalize to [0, 1]
        } else {
            0.0
        };

        self.classify_response(&retrieved_affect, estimated_strength, is_anti)
    }

    /// Classify a retrieved affect vector into a SomaticResponse.
    fn classify_response(
        &self,
        affect_hv: &Hypervector,
        strength: f64,
        is_anti_somatic: bool,
    ) -> SomaticResponse {
        let mut best_proto: Option<&AffectPrototype> = None;
        let mut best_sim = f64::NEG_INFINITY;

        for proto in &self.affect_prototypes {
            let sim = affect_hv.similarity(&proto.hv);
            if sim > best_sim {
                best_sim = sim;
                best_proto = Some(proto);
            }
        }

        let proto = best_proto.expect("affect_prototypes is never empty");

        let gut_feeling = if is_anti_somatic {
            GutFeeling::Distrust
        } else {
            proto.gut_feeling.clone()
        };

        // Scale the PAD delta by strength
        let delta = PadVector::new(
            proto.pad.pleasure * strength * 0.3,
            proto.pad.arousal * strength * 0.3,
            proto.pad.dominance * strength * 0.3,
        );

        SomaticResponse {
            gut_feeling,
            strength,
            is_anti_somatic,
            matched_prototype: proto.name.clone(),
            prototype_similarity: best_sim,
            suggested_pad_delta: delta,
        }
    }

    /// Create a new somatic marker from a pattern detection and
    /// the current Daimon PAD state.
    pub fn create_marker(
        &mut self,
        pattern_hv: &Hypervector,
        current_affect: &PadVector,
    ) {
        let affect_hv = self.pad_codebook.encode_pad(current_affect);
        let marker = SomaticMarker::new(
            pattern_hv.clone(),
            affect_hv,
            current_affect.clone(),
            self.current_tick,
        );

        if self.recent_markers.len() >= self.config.max_recent_markers {
            // Force consolidation of the weakest marker
            self.retire_weakest_marker();
        }

        self.recent_markers.push(marker);
    }

    /// Update an existing marker based on pattern outcome.
    pub fn update_marker(
        &mut self,
        pattern_hv: &Hypervector,
        outcome: &PatternOutcome,
    ) {
        // Find the best-matching recent marker
        let mut best_idx: Option<usize> = None;
        let mut best_sim = 0.0f64;
        for (i, marker) in self.recent_markers.iter().enumerate() {
            let sim = pattern_hv.similarity(&marker.pattern_hv);
            if sim > best_sim && sim > 0.6 {
                best_sim = sim;
                best_idx = Some(i);
            }
        }

        if let Some(idx) = best_idx {
            let marker = &mut self.recent_markers[idx];

            // Update empirical tracking
            let outcome_confirms_affect =
                (marker.valence > 0.0 && outcome.empirical_return > 0.0)
                || (marker.valence < 0.0 && outcome.empirical_return < 0.0);

            marker.empirical_accuracy.update(
                if outcome_confirms_affect { 1.0 } else { 0.0 }
            );
            marker.empirical_return.update(outcome.empirical_return);

            if outcome_confirms_affect {
                marker.reinforce(outcome.magnitude);
            } else {
                marker.weaken(outcome.magnitude);
            }
        }
        // If no matching marker exists, a new one will be created
        // at the next gamma tick when this pattern is detected again.
    }

    /// Check whether a marker should be promoted to anti-somatic.
    pub fn check_anti_somatic(&self, marker: &SomaticMarker) -> bool {
        marker.valence > 0.0
            && marker.empirical_return.count() >= self.config.anti_somatic_min_samples
            && marker.empirical_return.value() < self.config.anti_somatic_return_threshold
    }

    /// Promote a marker to anti-somatic status.
    fn promote_to_anti_somatic(&mut self, marker_idx: usize) {
        let mut marker = self.recent_markers.remove(marker_idx);
        marker.is_anti_somatic = true;

        // Construct the anti-marker: bind pattern with inverted affect
        let inverted_pad = marker.affect_pad.inverse();
        let inverted_hv = self.pad_codebook.encode_pad(&inverted_pad);
        marker.affect_hv = inverted_hv;
        marker.marker_hv = marker.pattern_hv.bind(&marker.affect_hv);

        self.anti_somatic_set.push(marker);
    }

    /// Retire the weakest recent marker into the aggregate somatic map.
    fn retire_weakest_marker(&mut self) {
        if self.recent_markers.is_empty() {
            return;
        }

        let weakest_idx = self.recent_markers
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.strength.partial_cmp(&b.strength).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap();

        let marker = self.recent_markers.remove(weakest_idx);
        self.consolidate_single_marker(&marker);
    }

    /// Add a single marker into the aggregate somatic map.
    fn consolidate_single_marker(&mut self, marker: &SomaticMarker) {
        if self.consolidated_marker_count == 0 {
            self.somatic_map = marker.marker_hv.clone();
        } else {
            // Weighted bundling: repeat marker proportional to strength
            // before taking majority vote with existing map.
            //
            // Approximation: blend the marker into the map with weight
            // proportional to strength / (consolidated_count + strength).
            let weight = marker.strength
                / (self.consolidated_marker_count as f64 + marker.strength);
            self.somatic_map = self.somatic_map.weighted_blend(
                &marker.marker_hv,
                weight,
            );
        }
        self.consolidated_marker_count += 1;
    }

    // ─── Heartbeat integration ───────────────────────────────────

    /// Gamma tick: perform somatic retrieval on all detected patterns.
    ///
    /// Returns a SomaticResponse for each pattern. The caller
    /// (heartbeat controller) routes these to the Daimon and
    /// attention auction.
    pub fn gamma_tick(
        &mut self,
        detected_patterns: &[Hypervector],
    ) -> Vec<SomaticResponse> {
        let responses: Vec<SomaticResponse> = detected_patterns
            .iter()
            .map(|p| {
                let response = self.somatic_retrieval(p);
                // Record activation on matching markers
                for marker in &mut self.recent_markers {
                    if p.similarity(&marker.pattern_hv) > 0.6 {
                        marker.activate(self.current_tick);
                    }
                }
                response
            })
            .collect();

        responses
    }

    /// Theta tick: record which patterns were active during this
    /// deliberation cycle. No marker updates yet; outcomes are
    /// not resolved until later.
    pub fn theta_tick(
        &mut self,
        active_patterns: &[Hypervector],
        current_affect: &PadVector,
    ) {
        // Create markers for patterns that don't have one yet
        for pattern in active_patterns {
            let has_marker = self.recent_markers.iter()
                .any(|m| pattern.similarity(&m.pattern_hv) > 0.6);

            if !has_marker {
                self.create_marker(pattern, current_affect);
            }
        }
    }

    /// Delta tick: consolidate, decay, and scan for anti-somatic markers.
    pub fn delta_tick(&mut self) {
        self.current_tick += 1;

        // 1. Apply temporal decay to all recent markers
        let decay = self.config.strength_decay;
        for marker in &mut self.recent_markers {
            marker.decay(decay);
        }

        // 2. Scan for anti-somatic candidates
        let mut promote_indices = Vec::new();
        for (i, marker) in self.recent_markers.iter().enumerate() {
            if self.check_anti_somatic(marker) {
                promote_indices.push(i);
            }
        }
        // Promote in reverse order to preserve indices
        for idx in promote_indices.into_iter().rev() {
            self.promote_to_anti_somatic(idx);
        }

        // 3. Retire inactive markers
        let retirement_threshold = self.config.marker_retirement_ticks;
        let tick = self.current_tick;
        let mut retire_indices = Vec::new();
        for (i, marker) in self.recent_markers.iter().enumerate() {
            if tick.saturating_sub(marker.last_activated) > retirement_threshold
                && marker.strength < 0.15
            {
                retire_indices.push(i);
            }
        }
        // Retire in reverse order
        for idx in retire_indices.into_iter().rev() {
            let marker = self.recent_markers.remove(idx);
            self.consolidate_single_marker(&marker);
        }
    }

    /// Process outcomes from resolved predictions.
    pub fn process_outcomes(&mut self, outcomes: &[PatternOutcome]) {
        for outcome in outcomes {
            self.update_marker(&outcome.pattern_hv, outcome);
        }
    }

    /// NREM dream: replay pattern-outcome-emotion triples to
    /// consolidate somatic associations offline.
    pub fn dream_nrem(&mut self, replay: &[PatternOutcomeEmotion]) {
        for episode in replay {
            // Re-process as if the pattern was just detected
            let current_response = self.somatic_retrieval(&episode.pattern_hv);

            // Compare the retrieved affect to the actual outcome
            let outcome_positive = episode.outcome.empirical_return > 0.0;
            let marker_positive = current_response.gut_feeling == GutFeeling::StrongApproach
                || current_response.gut_feeling == GutFeeling::MildApproach;

            // Find or create the marker
            let marker_idx = self.recent_markers.iter().position(|m| {
                episode.pattern_hv.similarity(&m.pattern_hv) > 0.6
            });

            match marker_idx {
                Some(idx) => {
                    if outcome_positive == marker_positive {
                        // Correct association: strengthen
                        self.recent_markers[idx].reinforce(
                            episode.outcome.magnitude * 0.5, // dream replay at half strength
                        );
                    } else {
                        // Incorrect association: weaken
                        self.recent_markers[idx].weaken(
                            episode.outcome.magnitude * 0.5,
                        );
                    }
                    self.recent_markers[idx].empirical_return.update(
                        episode.outcome.empirical_return,
                    );
                }
                None => {
                    // No marker for this pattern. Create one from
                    // the replayed affect-at-resolution (the emotional
                    // state after seeing the outcome, which is more
                    // informative than the state at detection).
                    self.create_marker(
                        &episode.pattern_hv,
                        &episode.affect_at_resolution,
                    );
                }
            }
        }
    }

    /// REM dream: test anti-somatic markers against counterfactual
    /// scenarios. Markers that would have been correct under
    /// alternative conditions get weakened.
    pub fn dream_rem(&mut self, rng: &mut impl rand::Rng) {
        if self.anti_somatic_set.is_empty() {
            return;
        }

        // For each anti-somatic marker, generate a counterfactual:
        // "What if the outcome had been the opposite?"
        let mut weaken_indices = Vec::new();

        for (i, marker) in self.anti_somatic_set.iter().enumerate() {
            // Stochastic review: only process a random subset per REM cycle
            if rng.gen::<f64>() > 0.3 {
                continue;
            }

            // If the marker's empirical return has improved (the pattern
            // has started making money), the anti-somatic flag may be stale.
            if marker.empirical_return.value() > 0.0
                && marker.empirical_return.count() > self.config.anti_somatic_min_samples
            {
                weaken_indices.push(i);
            }
        }

        // Demote stale anti-somatic markers back to regular markers
        for idx in weaken_indices.into_iter().rev() {
            let mut marker = self.anti_somatic_set.remove(idx);
            marker.is_anti_somatic = false;
            // Re-encode with original (non-inverted) affect
            let original_pad = marker.affect_pad.inverse(); // inverse of inverse = original
            marker.affect_hv = self.pad_codebook.encode_pad(&original_pad);
            marker.affect_pad = original_pad;
            marker.marker_hv = marker.pattern_hv.bind(&marker.affect_hv);
            marker.strength *= 0.5; // start at reduced strength after demotion
            self.recent_markers.push(marker);
        }
    }

    // ─── Death testament ─────────────────────────────────────────

    /// Produce a death testament containing the somatic state
    /// for transfer to a successor Golem.
    pub fn death_testament(&self) -> SomaticTestament {
        // Select the top-K strongest recent markers
        let mut sorted_markers: Vec<&SomaticMarker> = self.recent_markers
            .iter()
            .filter(|m| m.strength > 0.2)
            .collect();
        sorted_markers.sort_by(|a, b| {
            b.strength.partial_cmp(&a.strength).unwrap()
        });
        let top_markers: Vec<SomaticMarker> = sorted_markers
            .into_iter()
            .take(self.config.testament_top_k)
            .cloned()
            .collect();

        SomaticTestament {
            aggregate_map: self.somatic_map.clone(),
            consolidated_count: self.consolidated_marker_count,
            top_markers,
            anti_somatic_set: self.anti_somatic_set.clone(),
        }
    }

    /// Inherit a death testament from a predecessor Golem.
    /// Blends the inherited somatic map with any existing state.
    pub fn inherit_testament(&mut self, will: &SomaticTestament) {
        let w = self.config.inheritance_weight;

        // Blend the aggregate maps
        if self.consolidated_marker_count == 0 {
            self.somatic_map = will.aggregate_map.clone();
        } else {
            self.somatic_map = self.somatic_map.weighted_blend(
                &will.aggregate_map,
                w,
            );
        }
        self.consolidated_marker_count += will.consolidated_count;

        // Inherit top markers into the recent store at reduced strength
        for marker in &will.top_markers {
            let mut inherited = marker.clone();
            inherited.strength *= w; // discount inherited markers
            inherited.created_at = self.current_tick;
            inherited.last_activated = self.current_tick;
            self.recent_markers.push(inherited);
        }

        // Inherit the full anti-somatic set
        for marker in &will.anti_somatic_set {
            let mut inherited = marker.clone();
            inherited.created_at = self.current_tick;
            inherited.last_activated = self.current_tick;
            self.anti_somatic_set.push(inherited);
        }
    }

    // ─── Diagnostics ─────────────────────────────────────────────

    /// Number of active somatic markers (recent + consolidated).
    pub fn total_marker_count(&self) -> usize {
        self.recent_markers.len() + self.consolidated_marker_count
    }

    /// Number of anti-somatic markers (emotional traps detected).
    pub fn anti_somatic_count(&self) -> usize {
        self.anti_somatic_set.len()
    }

    /// Estimated SNR of the aggregate somatic map.
    pub fn map_snr(&self) -> f64 {
        if self.consolidated_marker_count <= 1 {
            f64::INFINITY
        } else {
            (10_240.0 / (self.consolidated_marker_count as f64 - 1.0)).sqrt()
        }
    }
}

/// Death testament for somatic state transfer.
#[derive(Clone)]
pub struct SomaticTestament {
    /// The aggregate somatic map of the deceased Golem.
    pub aggregate_map: Hypervector,
    /// Number of markers consolidated into the aggregate map.
    pub consolidated_count: usize,
    /// The top-K strongest individual markers at full resolution.
    pub top_markers: Vec<SomaticMarker>,
    /// All anti-somatic markers (emotional traps).
    pub anti_somatic_set: Vec<SomaticMarker>,
}
```

### Somatic map serialization

```rust
use std::io::{Read, Write};

/// Compact wire format for somatic map transfer.
/// Used for death testament serialization and checkpoint snapshots.
pub struct SomaticMapSnapshot {
    pub aggregate_map_bytes: Vec<u8>,       // 1,280 bytes
    pub consolidated_count: u64,
    pub recent_marker_count: u32,
    pub anti_somatic_count: u32,
    pub markers: Vec<MarkerSnapshot>,
    pub anti_markers: Vec<MarkerSnapshot>,
}

pub struct MarkerSnapshot {
    pub pattern_hv_bytes: Vec<u8>,          // 1,280 bytes
    pub affect_hv_bytes: Vec<u8>,           // 1,280 bytes
    pub valence: f32,                       // downsized for wire
    pub strength: f32,
    pub empirical_accuracy: f32,
    pub empirical_return: f32,
    pub activation_count: u32,
    pub is_anti_somatic: bool,
}

impl SomaticEngine {
    /// Serialize the somatic engine state to a compact byte buffer.
    /// Used for checkpoint persistence and death testament wire format.
    pub fn serialize(&self) -> SomaticMapSnapshot {
        let recent_snapshots: Vec<MarkerSnapshot> = self.recent_markers
            .iter()
            .filter(|m| m.strength > 0.1) // skip near-dead markers
            .map(|m| MarkerSnapshot {
                pattern_hv_bytes: m.pattern_hv.to_bytes(),
                affect_hv_bytes: m.affect_hv.to_bytes(),
                valence: m.valence as f32,
                strength: m.strength as f32,
                empirical_accuracy: m.empirical_accuracy.value() as f32,
                empirical_return: m.empirical_return.value() as f32,
                activation_count: m.activation_count as u32,
                is_anti_somatic: m.is_anti_somatic,
            })
            .collect();

        let anti_snapshots: Vec<MarkerSnapshot> = self.anti_somatic_set
            .iter()
            .map(|m| MarkerSnapshot {
                pattern_hv_bytes: m.pattern_hv.to_bytes(),
                affect_hv_bytes: m.affect_hv.to_bytes(),
                valence: m.valence as f32,
                strength: m.strength as f32,
                empirical_accuracy: m.empirical_accuracy.value() as f32,
                empirical_return: m.empirical_return.value() as f32,
                activation_count: m.activation_count as u32,
                is_anti_somatic: true,
            })
            .collect();

        SomaticMapSnapshot {
            aggregate_map_bytes: self.somatic_map.to_bytes(),
            consolidated_count: self.consolidated_marker_count as u64,
            recent_marker_count: recent_snapshots.len() as u32,
            anti_somatic_count: anti_snapshots.len() as u32,
            markers: recent_snapshots,
            anti_markers: anti_snapshots,
        }
    }

    /// Wire size estimate for capacity planning.
    /// Aggregate map: 1,280 bytes (fixed).
    /// Each marker: ~2,580 bytes (two HVs + metadata).
    /// Anti-somatic markers: same size.
    pub fn estimated_wire_size(&self) -> usize {
        let base = 1_280 + 16; // aggregate map + header
        let marker_size = 2 * 1_280 + 20; // two HVs + scalar fields
        let active_recent = self.recent_markers.iter()
            .filter(|m| m.strength > 0.1)
            .count();
        base + active_recent * marker_size + self.anti_somatic_set.len() * marker_size
    }
}
```

### Diagnostics and observability

```rust
/// Runtime diagnostics for the somatic engine.
/// Exposed via the Golem's telemetry system for monitoring.
pub struct SomaticDiagnostics {
    /// Total markers in the system (recent + consolidated).
    pub total_markers: usize,
    /// Active recent markers (not retired to map).
    pub active_recent: usize,
    /// Anti-somatic markers (emotional traps detected).
    pub anti_somatic_count: usize,
    /// Estimated SNR of the aggregate somatic map.
    pub map_snr: f64,
    /// Average strength of recent markers.
    pub avg_recent_strength: f64,
    /// Fraction of gamma ticks that produced a non-neutral response.
    pub somatic_hit_rate: f64,
    /// Most recently matched prototype name.
    pub last_matched_prototype: String,
    /// Estimated wire size for death testament.
    pub estimated_testament_bytes: usize,
}

impl SomaticEngine {
    pub fn diagnostics(&self) -> SomaticDiagnostics {
        let avg_strength = if self.recent_markers.is_empty() {
            0.0
        } else {
            self.recent_markers.iter()
                .map(|m| m.strength)
                .sum::<f64>() / self.recent_markers.len() as f64
        };

        SomaticDiagnostics {
            total_markers: self.total_marker_count(),
            active_recent: self.recent_markers.len(),
            anti_somatic_count: self.anti_somatic_count(),
            map_snr: self.map_snr(),
            avg_recent_strength: avg_strength,
            somatic_hit_rate: 0.0, // tracked externally via counter
            last_matched_prototype: String::new(),
            estimated_testament_bytes: self.estimated_wire_size(),
        }
    }
}
```

### Integration example: full heartbeat cycle

```rust
/// Demonstrates one complete heartbeat cycle with somatic integration.
fn heartbeat_with_somatic(
    somatic: &mut SomaticEngine,
    daimon: &mut Daimon,
    attention: &mut AttentionAuction,
    ta_pipeline: &TaPipeline,
) {
    // ─── Gamma tick: perception ──────────────────────────────────
    let detected_patterns = ta_pipeline.detect_current_patterns();
    let somatic_responses = somatic.gamma_tick(&detected_patterns);

    // Inject pre-attentive affect into the Daimon
    for response in &somatic_responses {
        if response.is_salient() {
            daimon.inject_pre_attentive_affect(&response.suggested_pad_delta);
        }
    }

    // Modulate attention auction bids
    for (pattern, response) in detected_patterns.iter().zip(&somatic_responses) {
        let modifier = response.attention_bid_modifier();
        attention.modify_bid(pattern, modifier);
    }

    // ─── Theta tick: cognition ───────────────────────────────────
    let current_affect = daimon.current_pad();
    somatic.theta_tick(&detected_patterns, &current_affect);

    // Deliberative reasoning happens here, biased by the Daimon's
    // somatic-adjusted PAD state. Decisions made. Actions taken.

    // ─── Later: outcomes resolve ─────────────────────────────────
    let outcomes = ta_pipeline.get_resolved_outcomes();
    somatic.process_outcomes(&outcomes);

    // ─── Delta tick: consolidation ───────────────────────────────
    somatic.delta_tick();
}
```

---

## Subsystem interactions [SPEC]

### Daimon integration

The somatic engine writes to the Daimon through `inject_pre_attentive_affect`, a method that adds a PAD delta to the Daimon's emotion layer before the theta tick's deliberative processing. The delta is small (scaled by 0.3 * strength) because pre-attentive affect should bias, not override. A strong fear marker might shift the Daimon's pleasure from 0.1 to -0.1 and arousal from 0.3 to 0.5. This is enough to change the risk tolerance calculation without overriding the Daimon's own emotional processing of the current market state.

The scaling factor (0.3) deserves justification. In Damasio's framework, somatic markers are one input to a decision process that also includes rational analysis, current emotional state, and social context. Setting the scaling too high makes the Golem reactive, snapping to gut-level judgments and ignoring deliberative analysis. Setting it too low makes the somatic system irrelevant. The 0.3 factor means a maximum-strength somatic marker for a fear prototype (PAD: -0.7, 0.8, -0.5) would produce a delta of (-0.21, 0.24, -0.15). Against a Daimon with a neutral mood (all values near 0), this shifts the emotional state from "calm" to "mildly anxious" but not to "panicking." The Daimon can still override the somatic suggestion if its own processing of the current situation contradicts it.

The Daimon reads the somatic engine's state indirectly through the attention auction results. Patterns with strong somatic markers get more attention, which means the Daimon processes more information about them, which generates richer emotional responses. The somatic system amplifies the Daimon's existing tendencies rather than replacing them.

There is an asymmetry in the temporal layers. Somatic markers write to the emotion layer (fast, per-event) but not to the mood layer (medium, per-session) or personality layer (slow, per-generation). Mood and personality evolve through their own dynamics. A single somatic fear response at a gamma tick should not depress the Golem's mood for the next hour. But repeated somatic fear responses within a session will indirectly shift mood, because the emotion layer feeds into mood computation through the Daimon's internal dynamics. The somatic engine respects the temporal hierarchy by writing only to the fastest layer and letting the cascade propagate naturally.

### Attention auction

The attention auction uses a VCG mechanism where subsystems bid for cognitive resources. The somatic engine modifies bids rather than placing its own. When a TA pattern triggers a strong somatic response, its bid gets multiplied by the `attention_bid_modifier` (1.0x for neutral, up to 2.0x for anti-somatic distrust). This means the theta tick's reasoning cycle spends more time on somatically-marked patterns.

The asymmetry in bid modifiers is intentional. Fear (`StrongAvoid`, 1.8x) bids higher than confidence (`StrongApproach`, 1.5x) because negative outcomes have asymmetric impact. Distrust (`Distrust`, 2.0x) bids highest because anti-somatic patterns are the ones most likely to fool the Golem if not examined carefully.

### Signal metabolism (Doc 3)

Doc 3 models signals as living units competing for metabolic budget. Each signal has a learning rate that determines how quickly it adapts to new evidence. The somatic engine modulates this learning rate indirectly through the Daimon's arousal state. When a somatic fear response fires, the Daimon's arousal increases. Higher arousal increases the learning rate for signals in the associated context. The Golem learns faster from feared situations because its arousal is higher.

This is the computational analog of stress-enhanced memory formation in biological systems. The amygdala modulates hippocampal learning rates during stressful experiences, which is why people form vivid memories of frightening events. The Golem's architecture replicates this: the somatic engine (analogous to amygdala) modulates the signal metabolism (analogous to hippocampal learning) through the Daimon's arousal state (analogous to norepinephrine release).

The effect is indirect: somatic engine -> Daimon arousal -> metabolism learning rate. No direct coupling between the somatic engine and signal metabolism. The Daimon mediates, which means mood and personality also modulate the effect. A personality-level bias toward high arousal (a "nervous" Golem) amplifies the somatic-to-metabolism coupling. A personality-level bias toward low arousal (a "calm" Golem) dampens it. This creates individual differences between Golems in how strongly their gut feelings influence their learning, an emergent personality trait that varies across the population.

### Pattern ecosystem (Doc 6)

Doc 6 evolves patterns as organisms with fitness scores. Somatic markers provide an additional fitness signal: patterns that trigger strong, accurate somatic markers (high strength, high empirical accuracy) get a fitness bonus. They are "emotionally resonant" patterns that the Golem has learned to trust or fear for good reason.

Anti-somatic markers provide a negative fitness signal. Patterns flagged as emotional traps get a fitness penalty, suppressing their reproduction in the ecosystem. This creates selection pressure against parasitic patterns.

### Adversarial robustness (Doc 8)

Doc 8 detects signal manipulation at the statistical level: distribution shifts, anomalous signal clusters, Sybil-like pattern injection. The somatic engine detects a different failure mode: patterns that bypass statistical detection by targeting emotional responses. An adversary who knows the Golem's somatic map could craft patterns that trigger confident approach responses even though the underlying signal is manipulated.

The defense is layered. Doc 8's statistical tests run first. If a pattern passes statistical checks but triggers a strong positive somatic response, and the somatic engine has flagged similar patterns as anti-somatic in the past, the combined signal is "statistically clean but experientially suspicious." This is the kind of thing a human trader would call "it looks right on the charts but something feels off." The Golem can now express that.

### Dreams

NREM dreams replay pattern-outcome-emotion triples from episodic memory. The somatic engine processes each replay as a simulated experience, strengthening correct markers and weakening incorrect ones. This is offline consolidation: the Golem doesn't need to re-encounter a pattern in the live market to refine its somatic association. One night of NREM dreaming can process hundreds of episodes.

REM dreams test anti-somatic markers. The dream system generates counterfactual scenarios by perturbing stored episodes. If an anti-somatic marker's pattern would have been profitable under a different market regime, the marker weakens. This prevents over-learning from transient adversarial conditions. A market manipulation that fooled the Golem for a week should not permanently poison that pattern class.

### Grimoire and mortality

The Grimoire stores episodes with somatic metadata: what the Golem felt when it encountered the pattern, what it felt after the outcome resolved. This metadata enriches the episodic record beyond raw outcomes. When retrieving similar past situations, the Grimoire returns both the factual record and the emotional context: "here's what happened and here's how it felt."

Mortality gives somatic maps their evolutionary dynamic. A young Golem with a thin somatic map makes more mistakes (no gut feelings to pre-filter bad options) but also explores more freely (no inherited biases). An old Golem with a thick somatic map makes fewer mistakes but may be slower to adapt to regime changes (strong markers take time to weaken). Death and rebirth is the reset mechanism: the successor inherits the aggregate map but sheds the detail, starting with a disposition rather than a doctrine.

---

## DeFi primitive coverage [SPEC]

Somatic markers are not generic. The emotional associations differ by DeFi primitive, and these differences carry information.

### LP operations

The somatic feel of a good LP rebalance is calm confidence: moderate pleasure (0.4), low arousal (0.15), high dominance (0.7). The Golem has seen this pattern before. It knows what to do. The execution path is familiar. The somatic feel of a bad rebalance (impermanent loss, missed range) is frustration: negative pleasure (-0.5), moderate arousal (0.4), moderate dominance (0.3). The Golem still feels some control, but the outcome was wrong.

Markers for LP operations encode tick-range width, fee tier, and pool volatility regime. A Golem that has been burned by narrow ranges in volatile markets develops a somatic aversion to `V_narrow` tick ranges when the volatility context vector shows elevated movement. The fear is specific: not "LP is scary" but "narrow LP in high-vol is scary."

This specificity comes from the HDC encoding. The pattern hypervector for "narrow LP in ETH/USDC during high volatility" is different from "wide LP in ETH/USDC during high volatility." The somatic map stores separate markers for each. A Golem might feel confident about wide-range LP in volatile conditions (the impermanent loss exposure is bounded) while feeling fearful about narrow-range LP in the same conditions (the position can go out of range within a single block). The granularity of the somatic markers matches the granularity of the pattern encoding.

JIT (just-in-time) liquidity detection creates a distinct somatic signature. When the Golem observes a JIT pattern (add concentrated liquidity, capture one block of fees, remove liquidity), the somatic marker is high arousal, mixed valence. The Golem recognizes the pattern as adversarial to its own LP positions but also as a signal that a large swap is imminent. Over time, JIT detection markers become compound: the fear component (someone is extracting value) and the curiosity component (a large trade is about to happen) coexist in the same marker, producing a Distrust-like response that demands deliberative attention.

### Lending and borrowing

The somatic feel of a good lending rate entry is mild excitement: pleasure 0.3, arousal 0.5, dominance 0.4. The Golem spotted an above-average rate and acted on it. The feel of a bad entry (rates dropped immediately after, or utilization spiked making the Golem's position illiquid) is anxiety: pleasure -0.4, arousal 0.7, dominance -0.3.

For borrowing, the dominance axis matters most. A Golem with a risky collateral ratio feels low dominance. Over time, the somatic map encodes "being near the liquidation threshold feels terrible" as a strong avoidance marker for `V_risky` and `V_critical` collateral ratio buckets. The Golem develops an aversion to over-leveraged positions that precedes rational analysis.

The temporal structure of lending somatic markers differs from LP markers. LP markers typically resolve within a few theta ticks (the rebalance either worked or it didn't). Lending markers resolve over longer horizons: the rate entry was good or bad depending on what rates did over the following hours or days. This means lending markers accumulate strength more slowly (fewer reinforcement events per unit time) but also decay more slowly (the experience is more deliberate, less noisy). The config's `strength_decay` parameter applies uniformly, but the effective decay rate differs by primitive because the activation frequency differs.

Liquidation events create the strongest somatic markers in the entire system. A Golem that experiences or narrowly avoids a liquidation forms a marker with near-maximum negative pleasure and arousal. These markers persist across many delta ticks even without reinforcement because the initial strength is so high. In Damasio's terms, this is the equivalent of a traumatic memory: a single event so emotionally intense that it brands a permanent somatic association. A Golem that was once liquidated at 115% collateral ratio will feel visceral aversion to collateral ratios below 150% for the rest of its life.

### Cross-primitive somatic patterns

The most interesting somatic markers span primitives. When LP operations feel bad and lending operations feel good simultaneously, the Golem's somatic map encodes this as a composite state: capital is flowing from AMMs to lending, a rotation signal. A Golem that has experienced this rotation multiple times develops a compound somatic marker for it.

The encoding uses the same HDC composition from Doc 1:

```
rotation_hv = bind(R_primitive_1, lp_bad_feel_hv)
            + bind(R_primitive_2, lending_good_feel_hv)
            + bind(R_temporal, V_simultaneous)
```

This compound marker fires when the cross-primitive somatic pattern repeats, giving the Golem a gut feeling about capital rotation before any single-primitive indicator would detect it.

### Somatic profiles by primitive

Each DeFi primitive develops its own characteristic somatic profile in a mature Golem:

**AMM swaps.** Markers cluster around size/slippage combinations. Large swaps with low slippage feel confident. Large swaps with high slippage feel anxious. MEV-sandwiched swaps accumulate strong disgust markers.

**Vaults (ERC-4626).** Somatic associations track share price trajectory more than absolute yield. A vault with appreciating share price feels good even at moderate APY. A vault with declining share price triggers anxiety regardless of reported yield.

**Staking and restaking.** Low-arousal somatic profiles dominate. Staking is a slow game and the Golem's somatic associations reflect it: calm satisfaction or mild concern, rarely panic. The exception is slashing events. An EigenLayer operator slash creates a somatic marker comparable in intensity to a liquidation event: extremely negative pleasure, high arousal, low dominance. These markers are rare but powerful. A Golem that witnessed a slash on an operator it had delegated to will carry that marker into every future restaking decision, biasing it toward operators with longer track records and lower risk profiles.

**Perpetuals and options.** High-arousal profiles dominate. These instruments carry leverage, and the Golem's somatic map reflects the volatility of outcomes. Strong approach/avoid markers with high activation counts. The funding rate is a particularly rich source of somatic associations. A Golem that has learned to enter short perpetual positions when funding is extremely positive (and gotten paid for it) develops a strong approach marker for the combination of `V_positive_high` funding and `V_short` direction. A Golem that tried the same trade during a momentum rally and got squeezed develops a strong avoidance marker. Both markers are correct in their respective regimes. The somatic map doesn't resolve the contradiction; it records both associations, and the strength of each reflects how recently and frequently each outcome occurred. The Daimon's current mood and the broader market context determine which marker dominates in any given moment.

**Vaults (ERC-4626).** Somatic associations track share price trajectory more than absolute yield. A vault with appreciating share price feels good even at moderate APY. A vault with declining share price triggers anxiety regardless of reported yield. This is a specific prediction of the somatic model: vault TVL outflows should correlate more with share price declines than with APY declines, because the somatic marker system weights the emotional salience of losses over the rational calculation of expected returns.

---

## Cybernetic feedback loops [SPEC]

The somatic system creates two nested feedback loops.

### The inner loop: pattern-emotion-attention

```
TA detects pattern
    -> somatic retrieval produces gut feeling
    -> gut feeling injects pre-attentive affect into Daimon
    -> Daimon's adjusted state biases attention auction bids
    -> more attention allocated to somatically-marked patterns
    -> richer information about marked patterns enters deliberation
    -> better decisions about marked patterns
    -> outcomes update somatic markers
    -> stronger/weaker markers for next retrieval
```

This loop runs within a single heartbeat cycle. It takes ~500ns for the somatic retrieval and affect injection at the gamma tick, then minutes for the pattern to resolve and the marker to update. The inner loop determines what the Golem pays attention to right now.

### The outer loop: experience-inheritance-disposition

```
Golem encounters patterns over its lifetime
    -> somatic map accumulates (100s of markers)
    -> strong markers and anti-somatic set stabilize
    -> Golem dies
    -> death testament transfers aggregate map + top markers + anti-somatic set
    -> successor Golem inherits somatic disposition
    -> inherited disposition biases attention from the first tick
    -> successor refines inherited map through its own experience
    -> repeat across generations
```

This loop runs across Golem lifetimes. Each generation refines the somatic map, keeping associations that predict outcomes and shedding ones that don't. The map co-evolves with the market: in a regime where a specific pattern is reliably profitable, the somatic association strengthens across generations. When the regime changes, the inherited marker weakens through contradicting experience, and the lineage adapts within one or two generations.

The two loops interact through the anti-somatic system. The inner loop detects emotional traps (patterns that feel right but lose money). The outer loop transmits the trap detection across generations, preventing successors from falling for the same trick. Over many generations, the lineage's anti-somatic library grows, creating an accumulated immune memory against manipulative patterns.

### Emergent behavior: somatic wisdom

A mature Golem lineage (many generations, rich somatic inheritance) exhibits something that looks like wisdom: fast, accurate gut feelings about market patterns that compress years of experience into sub-100ns retrievals. The wisdom is not explicit knowledge ("this pattern predicts a 63% chance of upward movement"). It is somatic knowledge ("this pattern feels bad"). The explicit analysis happens afterward, in the theta tick's deliberative cycle, but the somatic system has already narrowed the option space.

This matches Damasio's original observation about human experts. An experienced trader doesn't consciously analyze every chart pattern. Most are pre-filtered by somatic markers that fire before conscious recognition. The remaining patterns, the ones that survive somatic filtering, get full deliberative attention. The Golem's architecture replicates this structure, with the gamma tick serving as the pre-attentive somatic filter and the theta tick serving as the deliberative system that processes only the survivors.

There is a subtlety worth noting. Human somatic markers are often wrong in systematic ways. Loss aversion, recency bias, anchoring: these are somatic markers that fire on heuristics rather than evidence. The Golem's somatic system is not immune to these failure modes. A marker formed from a single traumatic event (a liquidation) can be disproportionately strong relative to its actual predictive value, creating loss aversion. A marker formed from recent events can dominate over markers from older events, creating recency bias. The anti-somatic detection system catches some of these errors (where affect and outcome diverge). But it doesn't catch biases where the affect is correctly negative but disproportionately strong.

This is where the dream system becomes a corrective. NREM replay revisits old episodes, giving past markers a chance to be reinforced or weakened against the full historical record, not just recent experience. REM counterfactual processing asks "what if?" questions that probe whether current somatic dispositions are still appropriate. Together, the dream systems act as a bias-correction mechanism for the somatic map. They do not eliminate somatic biases (some bias is functional, such as the elevated fear around liquidation thresholds), but they prevent transient experiences from dominating the map permanently.

The long-term trajectory is convergence toward calibrated somatic markers: gut feelings whose strength correlates with their empirical accuracy, formed through cycles of experience, consolidation, inheritance, and dream-mediated correction. A lineage that has run for many generations approaches a somatic equilibrium where the map's affect predictions match observed outcomes within noise. At that point, the Golem's gut feelings are not biases. They are compressed, experience-validated heuristics that operate at memory-access speed.

---

## Evaluation protocol [SPEC]

### Metric 1: somatic-guided decision speed

Compare decision latency (time from pattern detection to action) for Golems with somatic markers versus Golems without. Hypothesis: somatic Golems decide faster because pre-attentive filtering narrows the option space before deliberation.

**Protocol:** Run matched pairs of Golems (same configuration, same market feed) with and without the somatic engine enabled. Measure the distribution of theta-tick durations. The somatic Golem's theta ticks should be shorter on average because the attention auction has already down-weighted irrelevant patterns.

**Success criterion:** Median theta-tick duration reduced by at least 15% with somatic markers enabled, without degradation in decision quality.

### Metric 2: anti-somatic detection accuracy

Measure the precision and recall of anti-somatic marker detection against known emotional traps.

**Protocol:** Construct synthetic adversarial signals that are designed to trigger positive somatic responses but produce negative returns (the exact scenario that anti-somatic markers target). Inject these into the market feed alongside genuine signals. Measure how many the somatic engine flags as anti-somatic, and how many genuine positive patterns it incorrectly flags.

**Success criterion:** Precision > 0.8 (at most 20% of flagged patterns are false positives). Recall > 0.6 (at least 60% of true emotional traps are detected). The precision/recall tradeoff should favor precision because false positives suppress profitable patterns.

### Metric 3: somatic transfer quality

Measure whether inherited somatic maps improve successor performance relative to starting from scratch.

**Protocol:** Run two lineages of Golems on the same market data. Lineage A transfers somatic testaments at death. Lineage B starts fresh each generation. Compare cumulative returns, max drawdown, and decision quality metrics (Sharpe ratio, win rate) across 10+ generations.

**Success criterion:** Lineage A achieves break-even performance at least 30% faster in each generation than Lineage B. The advantage should compound: later generations of Lineage A should outperform earlier ones, while Lineage B's performance remains stationary across generations.

### Metric 4: ablation study

Selectively disable somatic engine components to measure their individual contribution.

**Configurations to test:**

- Full somatic engine (baseline)
- No anti-somatic detection (remove emotional trap flagging)
- No dream consolidation (disable NREM/REM somatic processing)
- No death testament transfer (each generation starts fresh)
- No somatic engine at all (pure deliberative decision-making)

**Protocol:** Run each configuration on the same 90-day historical market dataset. Compare risk-adjusted returns.

**Success criterion:** Each component should contribute measurably. Removing anti-somatic detection should increase losses from adversarial patterns. Removing dream consolidation should slow marker calibration. Removing testament transfer should eliminate cross-generation improvement. The full system should outperform every ablation.

---

## References

Damasio, A. R. (1994). *Descartes' Error: Emotion, Reason, and the Human Brain*. Putnam.

Bechara, A., Damasio, H., Tranel, D., and Damasio, A. R. (1997). Deciding advantageously before knowing the advantageous strategy. *Science*, 275(5304):1293-1295.

Mehrabian, A. and Russell, J. A. (1974). *An Approach to Environmental Psychology*. MIT Press.

Kanerva, P. (2009). Hyperdimensional computing: An introduction to computing in distributed representation with high-dimensional random vectors. *Cognitive Computation*, 1(2):139-159.

Bechara, A. and Damasio, A. R. (2005). The somatic marker hypothesis: A neural theory of economic decision. *Games and Economic Behavior*, 52(2):336-372.

Reimann, M. and Bechara, A. (2010). The somatic marker framework as a neurological theory of decision-making: Review, conceptual comparisons, and future neuroeconomics research. *Journal of Economic Psychology*, 31(5):767-776.

Lerner, J. S., Li, Y., Valdesolo, P., and Kassam, K. S. (2015). Emotion and decision making. *Annual Review of Psychology*, 66:799-823.
