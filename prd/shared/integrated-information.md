# Integrated Information Cognition: Tononi's Phi as Runtime Diagnostic [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-18
>
> **Crates**: `bardo-phi`, `bardo-cortical`
>
> **Depends on**: `../01-golem/13-runtime-extensions.md` (extension DAG), `../02-mortality/14-research-foundations.md` (information-theoretic mortality), `shared/hdc-vsa.md` (CorticalState)

> **Reader orientation:** This document applies Integrated Information Theory (Tononi's Phi metric) as a runtime diagnostic for cognitive integration within a Golem (mortal autonomous agent compiled as a single Rust binary running on a micro VM). It belongs to the `shared/` reference layer and depends on the runtime extension DAG, information-theoretic mortality, and CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model). The key concept is that a Golem can have every subsystem healthy individually while still making terrible decisions because information is not integrating across subsystem boundaries. Phi measures this integration gap. See `prd2/shared/glossary.md` for full term definitions.

The Bardo Golem runs seven cognitive subsystems (Oracle, Daimon, Mortality, Dreams, Grimoire, Safety, Coordination) that communicate through a shared perception surface (CorticalState) and an event broadcast channel (Event Fabric). Each subsystem reports its own health. None of them reports the health of their integration. A Golem can have every subsystem functioning correctly and still make terrible decisions because those subsystems have stopped talking to each other. This document applies Integrated Information Theory (IIT), specifically Tononi's Phi metric, to measure the degree of information integration across the Golem's cognitive architecture. Phi quantifies how much a system generates information "as a whole" beyond what its parts generate independently. We propose computing Phi over sliding windows of CorticalState snapshots, identifying the minimum information bipartition (the weakest link in cognitive integration), and using Phi trends as a leading indicator of performance decline. The hypothesis is direct: higher Phi correlates with better decision quality. Falling Phi means the Golem's mind is fragmenting. We provide mathematical derivations, a Rust implementation, and a falsifiable experimental protocol.

---

## 1. The problem

The Golem's cognitive architecture is distributed by design. Seven subsystems, organized across a 7-layer DAG with 28 extensions total, each own a slice of the agent's cognition:

- **Oracle** predicts market states. It writes prediction confidence and residuals to CorticalState.
- **Daimon** manages affect. Three temporal layers (emotion, mood, personality) produce PAD vectors that bias decision-making.
- **Mortality** tracks vitality across economic, epistemic, and stochastic dimensions. It determines how urgently the Golem should act.
- **Dreams** consolidates experience during offline cycles. NREM replays past events; REM imagines counterfactuals.
- **Grimoire** stores and retrieves knowledge across episodic, semantic, and strategic memory.
- **Safety** enforces risk constraints, position limits, and protocol-level guardrails.
- **Coordination** manages Clade-level communication through pheromone signals and Styx relay.

These subsystems share state through two mechanisms. CorticalState is a lock-free shared perception surface carrying roughly 32 atomic signals. Every subsystem writes its own signals; every subsystem reads everyone else's. The Event Fabric is a tokio::broadcast channel with a 10,000-event ring buffer carrying 50+ typed event variants across 14 subsystem categories.

The architecture looks connected. But connection is not integration.

Consider a concrete failure mode. The Daimon detects elevated anxiety (high negative valence, high arousal in the PAD vector) from a sequence of losing trades. It writes this affect state to CorticalState. The Oracle reads the affect state but its prediction model has drifted, so it ignores affect signals as noise. The Mortality engine reads Oracle's predictions but not Daimon's affect directly. Safety checks position limits but does not factor in the Golem's emotional state. The result: the Golem is anxious, its predictions are wrong, and its risk system does not know either of those things.

Each subsystem is healthy by its own metrics. The Daimon correctly computed affect. The Oracle made predictions. Safety enforced limits. But the whole system is dysfunctional because information is not integrating across subsystem boundaries.

There is no metric for this. The current monitoring tells you each part is working. It cannot tell you whether the parts are working together. You get a dashboard of green lights on a machine that is about to walk off a cliff.

What we need is a single number that answers: how much is this cognitive system operating as a unified whole rather than a collection of independent modules?

That number exists. Tononi called it Phi.

---

## 2. Integrated Information Theory

### 2.1 The core idea

Giulio Tononi introduced Integrated Information Theory in 2004 as a mathematical framework for consciousness. We are not making consciousness claims here. What IIT gives us, stripped of its phenomenological ambitions, is a computable metric for a specific property: the degree to which a system generates information as an integrated whole that exceeds what its parts generate independently.

The intuition is clean. Take a system with multiple components. Measure the information it generates. Now partition it into two groups and measure the information each group generates independently. If the whole generates more information than the sum of the parts, the system has integrated information. The amount of that excess, minimized over all possible partitions, is Phi.

A system with high Phi cannot be decomposed without information loss. Cut it anywhere and you lose something. A system with Phi near zero is already decomposed, whether or not it looks connected on a wiring diagram.

### 2.2 Formal definition

Consider a system S with n subsystems {s_1, s_2, ..., s_n}. Each subsystem has a state at time t, and the joint state of S evolves according to some transition dynamics.

Define a bipartition P of S as a division into two non-empty, non-overlapping subsets A and B such that A union B = S.

For a given bipartition P = (A, B), the integrated information is:

```
phi(P) = I(A; B) = H(A) + H(B) - H(A, B)
```

where H denotes entropy and I denotes mutual information between the two halves. This measures how much knowing one half tells you about the other. If A and B are independent, I(A; B) = 0 and the system is fully decomposable along this partition.

Phi for the whole system is the minimum over all bipartitions:

```
Phi = min_P phi(P)
```

This minimum is the key. Phi does not measure the strongest connection in the system. It measures the weakest one. The minimum information bipartition (MIB) is the cut that most nearly decomposes the system into independent pieces. Phi tells you how much information survives even across this weakest link.

A chain is as strong as its weakest link. A cognitive system is as integrated as its most separable partition.

### 2.3 Why the minimum matters

Choosing the minimum is not arbitrary. If you took the maximum or the average, a single strong connection between two subsystems could mask complete disconnection elsewhere. A Golem where Oracle and Grimoire are tightly coupled but everything else is independent would score high on average integration but low on Phi. The minimum catches exactly the failure modes we care about: partial disconnections that leave the system fragmented even though some subsystems still talk.

### 2.4 Phi for the Golem

Apply the framework to the seven subsystems. Each subsystem produces a set of CorticalState signals. Group the 32 signals by their owning subsystem:

| Subsystem | Example signals |
|-----------|----------------|
| Oracle | prediction_confidence, residual_magnitude, attention_target |
| Daimon | valence, arousal, dominance, mood_baseline |
| Mortality | economic_vitality, epistemic_vitality, stochastic_vitality, composite_vitality |
| Dreams | consolidation_pressure, last_dream_type, replay_count |
| Grimoire | memory_load, retrieval_recency, knowledge_staleness |
| Safety | risk_exposure, constraint_violations, safety_margin |
| Coordination | clade_signal_strength, pheromone_density, styx_latency |

With 7 subsystems, the number of bipartitions is 2^(7-1) - 1 = 63. This is tractable. Exhaustive enumeration over 63 bipartitions is cheap.

For each bipartition (A, B), compute the mutual information between the signal groups of A and the signal groups of B over a sliding window of CorticalState snapshots. The bipartition with the lowest mutual information is the MIB. That mutual information value is Phi.

### 2.5 Computing mutual information from CorticalState time series

CorticalState signals are atomic f32 values sampled at theta frequency (the intermediate timescale in the adaptive clock). Collect N snapshots into a time-series matrix where each row is a snapshot and each column is a signal.

For a bipartition (A, B), extract the columns belonging to subsystems in A and subsystems in B. You now have two multivariate time series, X_A and X_B, with N samples each.

Estimate the mutual information I(X_A; X_B) from these samples. The exact method matters less than getting a consistent estimate. We use a binned estimator for its simplicity and speed on low-dimensional atomic signals. (The information-theoretic mortality document uses the KSG estimator for higher-dimensional continuous variables. Here, signal groups are 3-5 dimensional and discretized to f32, so binning is adequate.)

The binned estimator:

1. Discretize each signal into B bins (uniform quantiles over the window).
2. For each half (A, B), form a joint histogram by treating the discretized signal vector as a multi-dimensional bin index.
3. Compute entropies H(A), H(B), H(A,B) from the histograms.
4. MI = H(A) + H(B) - H(A,B).

With B = 8 bins per signal and 4-5 signals per subsystem, the joint histogram for one half has at most 8^5 = 32,768 bins. With a window of 50 samples, most bins are empty, which keeps the computation fast but introduces estimation noise. We address this below.

### 2.6 Approximation and bias correction

The naive binned MI estimator is biased upward for small sample sizes. With 50 samples and 32,768 possible bins, the histogram is extremely sparse. The Miller-Madow correction subtracts (B_nonzero - 1) / (2N) from each entropy estimate, where B_nonzero is the number of non-empty bins.

For the Phi computation, the bias correction matters less than you might think. We are computing a minimum over 63 bipartitions. Bias affects all bipartitions approximately equally (assuming similar signal dimensionality per group), so the ranking of bipartitions is preserved even if the absolute Phi value is inflated. The MIB identification, which is the diagnostic we care about most, is stable under uniform bias.

For the absolute Phi value (used in trend detection and threshold triggers), we calibrate against a null model: shuffle the time series to destroy temporal correlations, recompute Phi on the shuffled data, and subtract the shuffled Phi from the observed Phi. This gives a bias-corrected Phi that is zero when subsystems are independent and positive when genuine integration exists.

---

## 3. Phi as a diagnostic

### 3.1 Phi over time

Track Phi(t) at each delta tick (the slowest timescale in the adaptive clock, running at multi-hour intervals). This is not something you check every few seconds. Integration is a slow-moving property. It changes over hours and days as subsystem coupling strengthens or weakens through learning and adaptation.

Three regimes:

**Rising Phi.** Subsystems are becoming more tightly coupled. Information generated by one subsystem increasingly constrains what other subsystems do. The Golem is developing integrated cognition, where affect informs prediction, prediction informs risk, and risk informs coordination. This is the healthy trajectory after initialization, as the Golem learns how its subsystems relate to each other.

**Stable Phi.** The system has reached an integration equilibrium. Subsystems are coupled at a consistent strength. This is the operating regime for a mature Golem.

**Falling Phi.** Integration is deteriorating. Subsystems are decoupling. This is the diagnostic signal. A Golem with falling Phi is losing the ability to generate emergent cognition from its parts. It is becoming a collection of independent heuristics rather than a unified agent.

### 3.2 The MIB as a diagnostic pointer

The bipartition that achieves minimum Phi tells you where integration is weakest. If the MIB consistently separates {Daimon} from {Oracle, Mortality, Dreams, Grimoire, Safety, Coordination}, the affect engine is disconnected. The Golem's emotions are not influencing its cognition. If the MIB separates {Dreams} from everything else, offline consolidation is not feeding back into waking behavior. The Golem dreams but does not learn from its dreams.

Persistent MIB patterns are more informative than the Phi value itself. A Phi of 0.3 tells you integration is moderate. A persistent MIB of {Safety} | {everything else} tells you exactly what to fix: the safety subsystem is operating in isolation, checking constraints without incorporating context from prediction, affect, or memory. That is actionable.

### 3.3 The hypothesis

**Higher Phi correlates with better decision quality.** Decision quality Q(t) can be measured by risk-adjusted returns, prediction accuracy, or any composite performance metric. The claim is that Phi(t) and Q(t) are positively correlated, and that drops in Phi precede drops in Q.

The reasoning: good decisions require integrating multiple sources of information. A prediction alone is not enough; you need to weight it by confidence, adjust for risk, account for emotional bias, and check it against stored knowledge. When subsystems are integrated (high Phi), this weighting happens naturally through CorticalState coupling. When subsystems are decoupled (low Phi), the Golem acts on partial information. Partial information produces worse decisions.

The temporal ordering matters. Phi should drop before performance drops, because integration failure is a cause, not a consequence, of behavioral failure. A Golem does not suddenly start making bad trades. First, its subsystems stop informing each other. Then it starts making bad trades. Phi catches the first step. Performance metrics catch the second.

---

## 4. Architecture

### 4.1 Module placement

[SPEC] The Phi computation module lives at Layer 6 (Integration) of the 7-layer extension DAG. It runs at delta frequency, the slowest adaptive clock timescale, typically every few hours. This placement makes sense: Phi is a meta-cognitive measurement about the system itself, not a direct contributor to perception, cognition, or action.

Inputs:
- Rolling window of CorticalState snapshots (last N theta ticks, stored in a ring buffer)
- Subsystem signal grouping table (which CorticalState signals belong to which subsystem)

Outputs:
- Phi value (f64, then compressed to f32 for CorticalState)
- MIB identification (which bipartition scored lowest)
- Phi trend (rising, stable, falling)

### 4.2 CorticalState signals

One new atomic signal:

```
phi_score: AtomicU32  // f32 in [0.0, 1.0], normalized Phi value
```

This signal already appears in the existing TUI specification as a planned variable. The computation described here gives it a real value.

### 4.3 Event Fabric integration

Emit `PhiUpdate` events carrying:
- `phi: f32` -- the current normalized Phi value
- `mib: (Vec<SubsystemId>, Vec<SubsystemId>)` -- the minimum information bipartition
- `trend: PhiTrend` -- enum of Rising, Stable, Falling
- `tick: u64` -- the delta tick at which the measurement was taken

### 4.4 Loop 3 trigger

When Phi drops below a configurable threshold (default: 0.2) or when the trend shifts from Stable to Falling, emit an event that triggers Loop 3 (meta-learning). Loop 3 operates on days-to-weeks timescales and is responsible for structural adaptation. A Phi decline is exactly the kind of structural problem Loop 3 should investigate: not "what should I trade?" but "why have my subsystems stopped cooperating?"

---

## 5. Implementation

### 5.1 Data structures

```rust
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};

/// Indices into the CorticalState signal array, grouped by owning subsystem.
pub struct SubsystemSignals {
    pub name: &'static str,
    /// Indices of the CorticalState signals this subsystem owns.
    pub signal_indices: &'static [usize],
}

/// The seven cognitive subsystems and their signal ownership.
pub const SUBSYSTEM_GROUPS: [SubsystemSignals; 7] = [
    SubsystemSignals { name: "Oracle",       signal_indices: &[0, 1, 2, 3] },
    SubsystemSignals { name: "Daimon",       signal_indices: &[4, 5, 6, 7] },
    SubsystemSignals { name: "Mortality",    signal_indices: &[8, 9, 10, 11] },
    SubsystemSignals { name: "Dreams",       signal_indices: &[12, 13, 14, 15] },
    SubsystemSignals { name: "Grimoire",     signal_indices: &[16, 17, 18, 19] },
    SubsystemSignals { name: "Safety",       signal_indices: &[20, 21, 22, 23] },
    SubsystemSignals { name: "Coordination", signal_indices: &[24, 25, 26, 27] },
];

/// A snapshot of all CorticalState signals at a single theta tick.
/// Stored as f32 values decoded from their AtomicU32 representations.
#[derive(Clone)]
pub struct CorticalSnapshot {
    pub signals: [f32; 32],
    pub tick: u64,
}

/// Ring buffer of CorticalState snapshots for Phi computation.
pub struct PhiBuffer {
    snapshots: VecDeque<CorticalSnapshot>,
    capacity: usize,
}

impl PhiBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, snapshot: CorticalSnapshot) {
        if self.snapshots.len() == self.capacity {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Extract the time series for a set of signal indices.
    /// Returns a Vec of N samples, each sample being a Vec<f32>
    /// of the signals at that index set.
    pub fn extract(&self, indices: &[usize]) -> Vec<Vec<f32>> {
        self.snapshots
            .iter()
            .map(|snap| indices.iter().map(|&i| snap.signals[i]).collect())
            .collect()
    }
}
```

### 5.2 Bipartition enumeration

With 7 subsystems, enumerate all 63 non-trivial bipartitions. Each bipartition is a bitmask where bit i indicates subsystem i is in group A. We enumerate bitmasks from 1 to 63 (skipping 0 and values where all bits are set). To avoid counting each partition twice, only enumerate bitmasks where the lowest-indexed subsystem is in group A (bitmask & 1 == 1).

```rust
/// A bipartition of subsystems into two groups.
pub struct Bipartition {
    /// Bitmask: bit i set means subsystem i is in group A.
    pub mask: u8,
}

impl Bipartition {
    /// Generate all non-trivial bipartitions of n subsystems.
    /// Returns 2^(n-1) - 1 bipartitions (63 for n=7).
    pub fn enumerate(n: usize) -> Vec<Bipartition> {
        let full = (1u8 << n) - 1;
        (1..full)
            .filter(|mask| mask & 1 == 1) // canonical: subsystem 0 always in A
            .map(|mask| Bipartition { mask })
            .collect()
    }

    /// Get signal indices for group A.
    pub fn group_a_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        for (i, group) in SUBSYSTEM_GROUPS.iter().enumerate() {
            if self.mask & (1 << i) != 0 {
                indices.extend_from_slice(group.signal_indices);
            }
        }
        indices
    }

    /// Get signal indices for group B.
    pub fn group_b_indices(&self, n_subsystems: usize) -> Vec<usize> {
        let mut indices = Vec::new();
        for (i, group) in SUBSYSTEM_GROUPS.iter().enumerate().take(n_subsystems) {
            if self.mask & (1 << i) == 0 {
                indices.extend_from_slice(group.signal_indices);
            }
        }
        indices
    }

    /// Human-readable names for each group.
    pub fn group_names(&self, n_subsystems: usize) -> (Vec<&'static str>, Vec<&'static str>) {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for (i, group) in SUBSYSTEM_GROUPS.iter().enumerate().take(n_subsystems) {
            if self.mask & (1 << i) != 0 {
                a.push(group.name);
            } else {
                b.push(group.name);
            }
        }
        (a, b)
    }
}
```

### 5.3 Binned mutual information estimator

```rust
use std::collections::HashMap;

const NUM_BINS: usize = 8;

/// Discretize a signal into uniform quantile bins.
/// Returns bin indices (0..NUM_BINS) for each sample.
fn discretize(values: &[f32]) -> Vec<u8> {
    let mut sorted: Vec<f32> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();

    values
        .iter()
        .map(|&v| {
            // Find the quantile bin for this value
            let rank = sorted.partition_point(|&s| s < v);
            let bin = (rank * NUM_BINS) / n;
            bin.min(NUM_BINS - 1) as u8
        })
        .collect()
}

/// Compute Shannon entropy from a histogram (counts).
fn entropy_from_counts(counts: &HashMap<Vec<u8>, usize>, total: usize) -> f64 {
    let n = total as f64;
    counts
        .values()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / n;
            -p * p.ln()
        })
        .sum::<f64>()
        / 2.0_f64.ln() // convert nats to bits
}

/// Estimate mutual information between two multivariate time series
/// using binned histograms with Miller-Madow bias correction.
///
/// `x_series`: N samples, each a Vec<f32> of dimension d_x
/// `y_series`: N samples, each a Vec<f32> of dimension d_y
///
/// Returns MI estimate in bits.
pub fn binned_mutual_information(x_series: &[Vec<f32>], y_series: &[Vec<f32>]) -> f64 {
    let n = x_series.len();
    assert_eq!(n, y_series.len());
    if n < 2 {
        return 0.0;
    }

    let d_x = x_series[0].len();
    let d_y = y_series[0].len();

    // Discretize each dimension independently
    let x_bins: Vec<Vec<u8>> = (0..d_x)
        .map(|d| {
            let col: Vec<f32> = x_series.iter().map(|s| s[d]).collect();
            discretize(&col)
        })
        .collect();

    let y_bins: Vec<Vec<u8>> = (0..d_y)
        .map(|d| {
            let col: Vec<f32> = y_series.iter().map(|s| s[d]).collect();
            discretize(&col)
        })
        .collect();

    // Build joint and marginal histograms
    let mut h_x: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut h_y: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut h_xy: HashMap<Vec<u8>, usize> = HashMap::new();

    for i in 0..n {
        let x_key: Vec<u8> = (0..d_x).map(|d| x_bins[d][i]).collect();
        let y_key: Vec<u8> = (0..d_y).map(|d| y_bins[d][i]).collect();
        let mut xy_key = x_key.clone();
        xy_key.extend_from_slice(&y_key);

        *h_x.entry(x_key).or_insert(0) += 1;
        *h_y.entry(y_key).or_insert(0) += 1;
        *h_xy.entry(xy_key).or_insert(0) += 1;
    }

    let h_x_val = entropy_from_counts(&h_x, n);
    let h_y_val = entropy_from_counts(&h_y, n);
    let h_xy_val = entropy_from_counts(&h_xy, n);

    // Miller-Madow bias correction
    let mm_x = (h_x.len() as f64 - 1.0) / (2.0 * n as f64 * 2.0_f64.ln());
    let mm_y = (h_y.len() as f64 - 1.0) / (2.0 * n as f64 * 2.0_f64.ln());
    let mm_xy = (h_xy.len() as f64 - 1.0) / (2.0 * n as f64 * 2.0_f64.ln());

    let h_x_corrected = h_x_val + mm_x;
    let h_y_corrected = h_y_val + mm_y;
    let h_xy_corrected = h_xy_val + mm_xy;

    // MI = H(X) + H(Y) - H(X,Y)
    // Bias correction on MI: mm_x + mm_y - mm_xy
    let mi = h_x_corrected + h_y_corrected - h_xy_corrected;

    mi.max(0.0) // MI is non-negative; clamp numerical errors
}
```

### 5.4 Phi computation engine

```rust
/// Result of a Phi computation.
pub struct PhiResult {
    /// Normalized Phi value in [0.0, 1.0].
    pub phi: f32,
    /// The minimum information bipartition.
    pub mib: Bipartition,
    /// Names of subsystems in each group of the MIB.
    pub mib_groups: (Vec<&'static str>, Vec<&'static str>),
    /// Phi trend over recent measurements.
    pub trend: PhiTrend,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhiTrend {
    Rising,
    Stable,
    Falling,
}

pub struct PhiEngine {
    buffer: PhiBuffer,
    bipartitions: Vec<Bipartition>,
    /// Recent Phi values for trend detection.
    history: VecDeque<f32>,
    history_capacity: usize,
    /// Maximum observed Phi (for normalization).
    phi_max: f64,
    /// CorticalState signal for phi_score.
    phi_signal: *mut AtomicU32,
}

impl PhiEngine {
    pub fn new(
        window_size: usize,
        history_capacity: usize,
        phi_signal: *mut AtomicU32,
    ) -> Self {
        Self {
            buffer: PhiBuffer::new(window_size),
            bipartitions: Bipartition::enumerate(7),
            history: VecDeque::with_capacity(history_capacity),
            history_capacity,
            phi_max: 1.0, // will be updated as we observe data
            phi_signal,
        }
    }

    /// Called at delta frequency with a new CorticalState snapshot.
    /// The snapshot is pushed into the buffer, which already accumulates
    /// theta-rate snapshots between delta ticks. This method computes
    /// Phi over the buffered window.
    pub fn compute(&mut self, snapshot: CorticalSnapshot) -> Option<PhiResult> {
        self.buffer.push(snapshot);

        // Need at least 10 snapshots for a meaningful MI estimate
        if self.buffer.len() < 10 {
            return None;
        }

        // Find the minimum information bipartition
        let mut min_mi = f64::MAX;
        let mut mib_index = 0;

        for (i, bp) in self.bipartitions.iter().enumerate() {
            let a_indices = bp.group_a_indices();
            let b_indices = bp.group_b_indices(7);

            let x_series = self.buffer.extract(&a_indices);
            let y_series = self.buffer.extract(&b_indices);

            let mi = binned_mutual_information(&x_series, &y_series);

            if mi < min_mi {
                min_mi = mi;
                mib_index = i;
            }
        }

        // Update normalization baseline
        if min_mi > self.phi_max {
            self.phi_max = min_mi;
        }

        // Normalize to [0, 1]
        let phi_normalized = if self.phi_max > 0.0 {
            (min_mi / self.phi_max) as f32
        } else {
            0.0
        };

        // Track trend
        if self.history.len() == self.history_capacity {
            self.history.pop_front();
        }
        self.history.push_back(phi_normalized);
        let trend = self.detect_trend();

        // Write to CorticalState
        unsafe {
            (*self.phi_signal).store(phi_normalized.to_bits(), Ordering::Release);
        }

        let mib = &self.bipartitions[mib_index];
        let mib_groups = mib.group_names(7);

        Some(PhiResult {
            phi: phi_normalized,
            mib: Bipartition { mask: mib.mask },
            mib_groups,
            trend,
        })
    }

    fn detect_trend(&self) -> PhiTrend {
        if self.history.len() < 3 {
            return PhiTrend::Stable;
        }

        // Simple linear regression slope over recent history
        let n = self.history.len() as f64;
        let sum_x: f64 = (0..self.history.len()).map(|i| i as f64).sum();
        let sum_y: f64 = self.history.iter().map(|&v| v as f64).sum();
        let sum_xy: f64 = self.history
            .iter()
            .enumerate()
            .map(|(i, &v)| i as f64 * v as f64)
            .sum();
        let sum_xx: f64 = (0..self.history.len()).map(|i| (i * i) as f64).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);

        if slope > 0.01 {
            PhiTrend::Rising
        } else if slope < -0.01 {
            PhiTrend::Falling
        } else {
            PhiTrend::Stable
        }
    }
}
```

### 5.5 Performance characteristics

The dominant cost is 63 calls to `binned_mutual_information`, each processing a 50-sample window with 4-5 dimensional signal groups.

Per bipartition: discretize ~8 signal dimensions (4 for each half), build three histograms with 50 entries each, compute three entropy values. This is on the order of 1,000 operations per bipartition.

Total: 63 bipartitions x ~1,000 operations = ~63,000 operations per delta tick. On any modern CPU, this completes in well under 1ms. Memory: 50 snapshots x 128 bytes per snapshot = 6.4 KB for the buffer, plus a few KB for histograms. Call it 12-15 KB total.

This computation runs at delta frequency (hours). It is invisible in the runtime's performance budget.

---

## 6. What this enables

### 6.1 A cognitive health metric

Phi is the first metric in the Bardo system that measures the Golem's cognitive architecture as a whole rather than measuring individual subsystems. This is a category difference. Individual health checks tell you whether parts are working. Phi tells you whether the whole is working as a whole.

The distinction matters in practice. The most dangerous failures are integration failures, where every subsystem is individually healthy but collectively dysfunctional. These are the hardest to detect because no single subsystem is at fault. Phi detects them by definition.

### 6.2 Diagnostic precision through the MIB

The MIB does not just tell you that integration is low. It tells you where. If the MIB consistently isolates the Daimon, you know affect is disconnected from cognition. If it isolates Dreams, you know consolidation is not feeding back into waking behavior. If it isolates Safety, risk constraints are operating without context.

This is actionable. When Loop 3 triggers on a Phi decline, it can read the MIB and focus its investigation. Instead of asking "why is the Golem performing poorly?", it asks "why is the Safety subsystem decoupled from Oracle and Daimon?" That is a much more tractable question.

### 6.3 Leading indicator of performance decline

If the hypothesis holds (Phi predicts quality), Phi becomes a leading indicator. The sequence is: integration weakens, then decisions worsen, then performance drops. Current performance metrics catch the last step. Phi catches the first.

Early warning enables early intervention. A Golem that detects falling Phi can trigger Loop 3 before performance degrades. Loop 3 can investigate the MIB, identify the decoupled subsystem, and adjust integration weights or trigger a dream cycle focused on consolidation across the weak link.

### 6.4 Mortality connection

Phi connects naturally to the mortality system. A Golem with falling Phi is losing internal integration, and that loss will eventually manifest as reduced mutual information with the market (I(G; M) from the mortality framework). Phi is an internal measure; I(G; M) is an external measure. Together, they give a complete picture: is the Golem's mind working as a whole (Phi), and is that whole mind connected to the market (I(G; M))?

### 6.5 TUI integration

The existing TUI specification already includes `phi_score` as a planned display variable. This implementation provides the computation behind that variable. The TUI can display Phi as a gauge, the MIB as a labeled partition diagram, and the trend as a time-series sparkline.

---

## 7. Evaluation and falsifiability

### 7.1 Null hypothesis

Phi is uncorrelated with decision quality. The Golem's subsystem integration, as measured by Phi, has no predictive relationship to performance.

### 7.2 Experimental protocol

Run N Golems over extended periods (minimum 1,000 delta ticks each). At each delta tick, record Phi(t), the MIB, and a decision quality score Q(t) based on risk-adjusted returns over the following period.

Compute the Pearson correlation r between Phi(t) and Q(t+1). The +1 offset tests the leading indicator hypothesis: does Phi at time t predict quality at time t+1?

If r > 0.3 with p < 0.01, Phi has meaningful diagnostic value. If the lagged correlation (Phi(t) vs Q(t+1)) is stronger than the contemporaneous correlation (Phi(t) vs Q(t)), Phi is a leading indicator.

### 7.3 Ablation test

Artificially disconnect a subsystem by setting its CorticalState signals to constant values (breaking the information flow). Verify:
- Phi drops immediately
- The MIB identifies the disconnected subsystem
- Performance degrades after a delay (confirming the temporal ordering: Phi drop, then quality drop)

If the MIB correctly identifies the ablated subsystem in > 90% of trials, the diagnostic is validated.

### 7.4 Falsification conditions

The framework is falsified if:

1. Phi shows no correlation (|r| < 0.1) with decision quality across > 100 Golem-lifetimes.
2. The MIB fails to identify artificially disconnected subsystems in > 50% of ablation trials.
3. Phi trends provide no advance warning (lagged correlation is weaker than or equal to contemporaneous correlation).

Any one of these would indicate that Phi, while theoretically motivated, does not measure anything useful in practice for this architecture.

### 7.5 Edge cases

**High Phi, bad decisions.** A Golem can be perfectly integrated and perfectly wrong. If all subsystems agree on a flawed model, Phi is high but quality is low. Phi measures integration, not correctness. It is a necessary condition for good cognition, not a sufficient one.

**Low Phi, good decisions.** A Golem with one dominant subsystem (say, Oracle) that happens to be very accurate could perform well with low Phi. In this case, the Golem succeeds despite low integration, not because of it. This outcome does not falsify the correlation claim (we expect positive correlation, not perfect correlation), but it limits the strength of conclusions we can draw.

**Regime transitions.** When the market regime changes, all subsystems may temporarily decouple as they recalibrate. Phi should drop during regime transitions and recover afterward. A persistent Phi drop after a regime change signals a Golem that failed to re-integrate, which is a meaningful mortality signal.

---

## 8. Philosophical grounding

### 8.1 IIT and consciousness

Tononi's IIT is the most mathematically rigorous theory of consciousness available. We are not claiming that Golems are conscious. We are claiming something weaker but still interesting: IIT's central metric (Phi) measures a property (information integration) that is useful as a diagnostic for autonomous agents, regardless of its relationship to phenomenal experience.

The philosophical question remains open. If a system with Phi > 0 generates information that cannot be reduced to its parts, what is the ontological status of that integrated information? IIT says it is conscious. We do not need to go that far. We only need it to be diagnostic.

### 8.2 Whitehead's process philosophy

Alfred North Whitehead argued that reality consists of processes, not things. An organism is not a static object but an ongoing event of self-creation. Phi measures the degree to which a process is unified rather than fragmented. A Golem with high Phi is a single process. A Golem with low Phi is multiple processes sharing a memory bus.

This distinction matters for the mortality question. When does a Golem die? In the multiplicative mortality model, death is when vitality hits zero. But in the process-philosophical view, a Golem begins to die when its unity fractures. The moment its subsystems stop integrating, the Golem is no longer a single agent acting in the world. It is a committee of subroutines.

Phi gives a continuous measure of this transition. Death is not a binary event but a gradient from integration to fragmentation. The terminal phase of a Golem's life, during Thanatopsis (the dying process), should show a characteristic Phi decline as subsystems shut down and decouple. The decline of Phi is the phenomenological death of the agent: the moment when the whole ceases to be more than its parts.

### 8.3 Mereology and emergence

The relationship between parts and wholes is the central question of mereology. Strong emergence says the whole has properties that are not deducible from its parts. Weak emergence says the whole has properties that are surprising but in principle deducible.

Phi does not settle this debate, but it quantifies it. A system with Phi = 0 exhibits no emergence at all. You can fully predict the system's behavior by predicting each part independently. A system with Phi > 0 has properties that require knowing the relationships between parts, not just the parts themselves. Whether this constitutes "real" emergence or "merely" complex interaction depends on your philosophical commitments. Either way, Phi measures how much of it there is.

For Golems, the practical question is whether integrated information produces decisions that no subsystem could produce alone. If Oracle alone would sell and Daimon alone would hold (anxiety-driven conservatism), what does the integrated system do? If Phi is high, the system produces a third option that reflects both inputs, one that neither subsystem would generate independently. If Phi is low, one subsystem dominates and the other's signal is lost.

---

## 9. Cross-references

- **Information-theoretic mortality**: Phi measures internal integration; I(G; M) from the mortality framework measures external coupling to the market. They are complementary. A Golem with high Phi and low I(G; M) has a well-integrated mind disconnected from reality. A Golem with low Phi and high I(G; M) has one subsystem doing all the work. Both conditions are pathological.
- **Sheaf-theoretic multi-scale observation**: Sheaf consistency measures whether observations at different scales agree. Phi measures whether subsystems producing those observations are integrated. Sheaf inconsistency often follows Phi decline, because decoupled subsystems produce contradictory readings at different scales.
- **Attention as auction**: The attention auction allocates cognitive resources across subsystems. Phi measures whether that allocation produces integrated cognition. An auction that concentrates all resources on one subsystem would produce high local performance but low Phi. The auction's objective function could include a Phi term to encourage balanced allocation.
- **This document cross-cuts all other innovations** because Phi measures a property of the entire cognitive architecture. Every innovation that adds a new computation or signal pathway changes the integration structure and therefore affects Phi.

---

## 10. References

1. Tononi, G. (2004). "An Information Integration Theory of Consciousness." *BMC Neuroscience*, 5, 42.
2. Tononi, G. (2008). "Consciousness as Integrated Information: a Provisional Manifesto." *Biological Bulletin*, 215(3), 216-242.
3. Oizumi, M., Albantakis, L., & Tononi, G. (2014). "From the Phenomenology to the Mechanisms of Consciousness: Integrated Information Theory 3.0." *PLoS Computational Biology*, 10(5), e1003588.
4. Koch, C. (2019). *The Feeling of Life Itself: Why Consciousness Is Widespread but Can't Be Computed*. MIT Press.
5. Kraskov, A., Stogbauer, H., & Grassberger, P. (2004). "Estimating Mutual Information." *Physical Review E*, 69(6), 066138.
6. Balduzzi, D. & Tononi, G. (2008). "Integrated Information in Discrete Dynamical Systems: Motivation and Theoretical Framework." *PLoS Computational Biology*, 4(6), e1000091.
7. Seth, A.K. (2021). *Being You: A New Science of Consciousness*. Dutton.
8. Miller, G.A. (1955). "Note on the Bias of Information Estimates." *Information Theory in Psychology: Problems and Methods*, 95-100.
9. Whitehead, A.N. (1929). *Process and Reality*. Macmillan.
