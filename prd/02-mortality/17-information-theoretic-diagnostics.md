# Information-Theoretic Mortality Diagnostics [SPEC]

> _"The measure of death is the measure of disconnection."_

> **Version**: 1.0 | **Status**: Draft
>
> **Crate**: `golem-mortality` (within `golem-core`)
>
> **Depends on**: `01-architecture.md` (triple-clock system, vitality computation), `04-economic-mortality.md` (USDC as metabolic substrate), `02-epistemic-decay.md` (epistemic clock specification), `03-stochastic-mortality.md` (per-tick hazard rate)

---

> **Reader orientation:** This document specifies an information-theoretic diagnostic layer that runs alongside Bardo's three mortality clocks. It computes the mutual information I(G; M) between a Golem (mortal autonomous DeFi agent) and its market environment using the KSG estimator, providing a unified scalar that detects mortality modes invisible to the individual clocks: informational decoupling (the Golem appears healthy but is statistically independent of outcomes), overfitting (high historical MI, near-zero current MI), and Clade (sibling Golems) redundancy (the Golem contributes no unique information). This layer augments but does not replace the three-clock composition. See `01-architecture.md` (triple-clock system) for the primary mortality architecture. See `prd2/shared/glossary.md` for full term definitions.

## Document Purpose

The three-clock mortality architecture (economic, epistemic, stochastic) composes vitality multiplicatively. This works, but the composition is ad hoc. The three clocks measure different units (USDC, accuracy scores, tick counts) with no shared currency and no formal justification for multiplication over any other aggregation operator.

This document specifies a complementary information-theoretic diagnostic layer that runs alongside the three clocks. It does not replace them. It augments them with a unified scalar, the mutual information I(G; M) between a Golem G and its market environment M, that reveals mortality modes invisible to the individual clocks: informational decoupling, overfitting, and Clade redundancy.

The diagnostic layer uses the Kraskov-Stogbauer-Grassberger (KSG) estimator to compute MI from sliding windows of (action, market_response) pairs. It feeds three new signals into the CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model) perception surface and integrates with the 9-step heartbeat at Step 3 (ANALYZE).

---

## Theoretical Foundation

### Mutual Information as Common Currency

All three mortality clocks measure the same underlying quantity from different angles: how much the Golem knows about the market that is useful for action. This quantity has a precise mathematical definition. It is the mutual information between the Golem's state and the market's state.

Let G be a random variable representing the Golem's internal state (capital, model parameters, prediction history) and M be a random variable representing the market's state (prices, volumes, liquidity, volatility). The mutual information is:

```
I(G; M) = H(G) + H(M) - H(G, M)
```

where H denotes Shannon entropy (Shannon, 1948). Equivalently:

```
I(G; M) = H(M) - H(M | G)
```

Read this as: the total uncertainty in the market, minus the uncertainty that remains after observing the Golem's state. A Golem with high mutual information "knows" a lot about the market. A Golem with I(G; M) = 0 knows nothing -- its state is statistically independent of the market's.

Mutual information is measured in bits. Bits are the common currency the multiplicative system lacks.

### Clock Decomposition into MI Components

Each existing clock decomposes naturally into a component of I(G; M):

**Economic clock as channel capacity.** Shannon's channel capacity theorem (Shannon, 1948) defines the maximum rate at which information can be reliably transmitted through a noisy channel. For a Golem, the "channel" is the mechanism by which actions produce market returns. Capital constrains the action space. As capital shrinks, the set of feasible actions shrinks, the optimization surface flattens, and channel capacity approaches zero. The economic mortality component is:

```
I_economic = C(capital) / C(initial_capital)
```

Running out of money kills a Golem not because a balance hits zero, but because the information channel between the Golem and the market closes.

**Epistemic clock as rate-distortion.** Rate-distortion theory (Shannon, 1959) defines the minimum bits per symbol needed to represent a source within a given distortion tolerance. The Golem's actual information rate r must exceed the minimum required rate R(D) to maintain acceptable prediction accuracy. The epistemic mortality component is:

```
I_epistemic = r / R(D)
```

When this ratio drops below 1.0, the Golem's model is provably too coarse.

**Stochastic clock as entropy production.** The market produces entropy at rate sigma. The Golem has finite information processing capacity kappa (bits per tick). The stochastic mortality component is:

```
I_stochastic = 1 - sigma / kappa  (clamped to [0, 1])
```

When sigma exceeds kappa, the Golem cannot keep up with the market's novelty rate.

### The Unified Decomposition

By the chain rule of mutual information (Cover & Thomas, 2006):

```
I(G; M) = I(G_econ; M) + I(G_epist; M | G_econ) - I_redundant
```

The complete vitality scalar is:

```
V(G) = I(G; M) / I_max
```

where I_max is the maximum achievable mutual information given the Golem's architecture and the market's entropy rate. V(G) in [0, 1] provides a diagnostic overlay on the existing composite_vitality.

The behavioral phase mapping remains identical:
- Thriving: V(G) > 0.7
- Stable: 0.4 < V(G) <= 0.7
- Conservation: 0.2 < V(G) <= 0.4
- Declining: 0.1 < V(G) <= 0.2
- Terminal: V(G) <= 0.1

---

## Three Novel Mortality Modes

The information-theoretic framework reveals failure modes invisible to the multiplicative system.

### Mode 1: Informational Decoupling

A Golem has healthy capital, accurate recent predictions, and plenty of ticks remaining -- yet I(G; M) approaches zero. This happens when the market regime changes in a way that makes the Golem's strategy orthogonal to price movements. Every individual clock looks fine. The Golem's actions have become statistically independent of market outcomes. It is trading randomly without knowing it.

In the multiplicative system, this Golem looks healthy. In the MI framework, it is already dead.

Detection criterion:
```
decoupled = (composite_vitality > 0.5) AND (V_mi < 0.2)
```

When detected, the diagnostic layer writes a `mortality:decoupling_alert` event and increases arousal on the CorticalState to trigger retraining at the next deliberation step.

### Mode 2: Overfitting

Define temporal mutual information:

```
I(G; M_past) >> I(G; M_future)
```

The Golem has high mutual information with historical market states but near-zero mutual information with the evolving market. Its model fits the past perfectly and the present not at all. The epistemic clock might show high accuracy on a rolling window that still contains mostly past-regime data. The MI framework catches this by measuring mutual information with the current market state, not a historical aggregate.

Detection criterion:
```
overfitted = (I_past / I_recent > 3.0) AND (I_recent < 0.3 * I_max)
```

where I_past is MI computed over observations from ticks [t-2000, t-1000] and I_recent is MI from ticks [t-1000, t].

### Mode 3: Clade Redundancy

For a Golem in a Clade, compute the conditional mutual information:

```
I(G; M | Clade) = I(G; M) - I(G; M; Clade)
```

This is the information the Golem provides about the market that no other Clade member provides. When I(G; M | Clade) = 0, the Golem is informationally redundant -- every bit of market knowledge it has is already captured by the Clade. This is the principled death criterion for Clade members.

The redundancy index:
```
redundancy = I(G; M | Clade) / I(G; M)
```

A value of 1.0 means the Golem is entirely non-redundant. A value of 0.0 means it contributes nothing the Clade doesn't already know. This feeds into Clade-level resource allocation decisions: a redundant Golem can be reassigned to a different strategy, or its capital can be reclaimed.

---

## Estimation: KSG Algorithm

Computing mutual information between continuous, high-dimensional random variables is hard. Naive histogram-based estimators suffer from the curse of dimensionality. The Kraskov-Stogbauer-Grassberger (KSG) estimator (Kraskov et al., 2004) solves this with a k-nearest-neighbor approach that adapts bin sizes locally.

For two continuous random variables X and Y with n joint samples, KSG Algorithm 1:

1. For each sample point i, find the distance to its k-th nearest neighbor in the joint (X, Y) space using the Chebyshev (L-infinity) norm.
2. Count n_x(i) = points within that distance in the X marginal.
3. Count n_y(i) = points within that distance in the Y marginal.
4. Estimate MI as:

```
I_hat(X; Y) = psi(k) - (1/n) * sum_i [psi(n_x(i)) + psi(n_y(i))] + psi(n)
```

where psi is the digamma function.

The k-d tree makes neighbor lookups O(log n) per query, giving O(n log n) total for n observations. For the Bardo system, n is the window size (~1,000 recent observation pairs), and k is typically 3-7.

Why KSG? It converges to the true MI as n grows, regardless of the underlying distribution. It adapts bin sizes locally. It makes no parametric assumptions, which matters because market returns are non-Gaussian.

---

## Architecture

### Extension DAG Integration

The MI mortality estimator lives in `golem-mortality` at Layer 3 (Behavior) of the 7-layer extension DAG. It receives inputs from:

- **Layer 0 (Foundation):** tick timing, gamma clock signal
- **Layer 1 (Storage):** observation buffer of (action, market_response) pairs from Grimoire
- **Layer 2 (Cognition):** prediction residuals from Oracle, current model state

It feeds upward to:

- **Layer 4 (Action):** mortality-adjusted action gating
- **Layer 5 (Social):** Clade redundancy signals via Styx
- **Layer 6 (Integration):** MI vitality diagnostic for heartbeat pipeline

### CorticalState Signals

Three new atomic signals on the CorticalState perception surface:

```
mutual_information: AtomicU32   // f32 bits, I(G; M) estimate
information_rate: AtomicU32     // f32 bits/tick, instantaneous MI rate
redundancy_index: AtomicU32     // f32 in [0, 1], I(G; M | Clade) / I(G; M)
```

These fit within the existing ~256-byte, 4-cache-line layout. Each is 4 bytes. Total addition: 12 bytes.

### Heartbeat Integration

The MI estimate updates at Step 3 (ANALYZE) of the 9-step heartbeat:

```
observe -> retrieve -> [ANALYZE: update I(G;M)] -> gate -> simulate -> validate -> execute -> verify -> reflect
```

At each gamma tick (5-15 seconds), the diagnostic engine:

1. Reads the latest (action, market_response) pair from the observation buffer
2. Appends it to the sliding window
3. Runs the KSG estimator on the window
4. Writes the result to `mutual_information` on CorticalState
5. Computes diagnostic alerts from the new V(G)

The gate step (Step 4) uses the MI diagnostic to flag phase-boundary risks. A Golem in apparent Thriving phase (by multiplicative vitality) but low MI vitality triggers Conservation-like caution.

### Relationship to Existing Mortality System

The MI diagnostic layer is advisory, not authoritative. The existing three-clock multiplicative composition remains the source of truth for phase transitions and behavioral mode changes. The MI layer provides three additional capabilities:

1. **Early warning.** MI can detect decoupling and overfitting before the individual clocks show degradation.
2. **Diagnostic explanation.** When composite vitality drops, the MI decomposition explains which information channel is closing.
3. **Clade-level insight.** The redundancy index provides information unavailable to individual-Golem mortality assessment.

In future versions, the MI-based vitality V(G) may graduate from diagnostic overlay to primary mortality scalar, contingent on the evaluation protocol results (see below).

---

## Implementation

### Observation Buffer

```rust
use std::collections::VecDeque;

/// A single observation: what the Golem did and what the market did in response.
#[derive(Clone, Debug)]
pub struct Observation {
    /// Golem's action vector: position sizes, protocol allocations, etc.
    pub action: Vec<f64>,
    /// Market response vector: returns, volume changes, liquidity shifts, etc.
    pub response: Vec<f64>,
    /// Timestamp in gamma ticks since genesis.
    pub tick: u64,
}

/// Sliding window of observations for MI estimation.
pub struct ObservationBuffer {
    window: VecDeque<Observation>,
    max_size: usize,
}

impl ObservationBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, obs: Observation) {
        if self.window.len() == self.max_size {
            self.window.pop_front();
        }
        self.window.push_back(obs);
    }

    pub fn actions(&self) -> Vec<&[f64]> {
        self.window.iter().map(|o| o.action.as_slice()).collect()
    }

    pub fn responses(&self) -> Vec<&[f64]> {
        self.window.iter().map(|o| o.response.as_slice()).collect()
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    /// Split buffer into two halves for temporal MI comparison.
    /// Returns (older_half, newer_half) as separate observation sets.
    pub fn split_temporal(&self) -> (Vec<&Observation>, Vec<&Observation>) {
        let mid = self.window.len() / 2;
        let older: Vec<_> = self.window.iter().take(mid).collect();
        let newer: Vec<_> = self.window.iter().skip(mid).collect();
        (older, newer)
    }
}
```

### KSG Estimator

```rust
/// Digamma function approximation (Bernardo, 1976).
/// Uses the asymptotic series for psi(x) with recurrence for small x.
fn digamma(mut x: f64) -> f64 {
    let mut result = 0.0;
    while x < 6.0 {
        result -= 1.0 / x;
        x += 1.0;
    }
    result += x.ln() - 1.0 / (2.0 * x);
    let x2 = x * x;
    result -= 1.0 / (12.0 * x2);
    result += 1.0 / (120.0 * x2 * x2);
    result -= 1.0 / (252.0 * x2 * x2 * x2);
    result
}

/// Chebyshev (L-infinity) distance between two points.
fn chebyshev_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(ai, bi)| (ai - bi).abs())
        .fold(0.0_f64, f64::max)
}

/// KSG mutual information estimator (Algorithm 1).
///
/// `x_samples`: n observations of variable X, each of dimension d_x
/// `y_samples`: n observations of variable Y, each of dimension d_y
/// `k`: number of nearest neighbors (typically 3-7)
///
/// Returns estimated mutual information in nats. Divide by ln(2) for bits.
pub fn ksg_mutual_information(
    x_samples: &[Vec<f64>],
    y_samples: &[Vec<f64>],
    k: usize,
) -> f64 {
    let n = x_samples.len();
    assert_eq!(n, y_samples.len(), "sample counts must match");
    assert!(n > k, "need more samples than neighbors");

    let d_x = x_samples[0].len();
    let d_y = y_samples[0].len();
    let d_joint = d_x + d_y;

    // Build joint-space points.
    let joint_points: Vec<Vec<f64>> = x_samples
        .iter()
        .zip(y_samples.iter())
        .map(|(x, y)| {
            let mut p = Vec::with_capacity(d_joint);
            p.extend_from_slice(x);
            p.extend_from_slice(y);
            p
        })
        .collect();

    let mut psi_nx_sum = 0.0;
    let mut psi_ny_sum = 0.0;

    for i in 0..n {
        let mut joint_dists: Vec<f64> = (0..n)
            .filter(|&j| j != i)
            .map(|j| chebyshev_distance(&joint_points[i], &joint_points[j]))
            .collect();
        joint_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let epsilon = joint_dists[k - 1];

        let n_x = x_samples
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .filter(|&(_, xj)| chebyshev_distance(&x_samples[i], xj) <= epsilon)
            .count();

        let n_y = y_samples
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .filter(|&(_, yj)| chebyshev_distance(&y_samples[i], yj) <= epsilon)
            .count();

        psi_nx_sum += digamma(n_x as f64);
        psi_ny_sum += digamma(n_y as f64);
    }

    let mi_nats = digamma(k as f64) - (psi_nx_sum + psi_ny_sum) / n as f64
        + digamma(n as f64);

    // Convert nats to bits.
    mi_nats / 2.0_f64.ln()
}
```

### Entropy Rate Estimator

```rust
/// Estimate the entropy rate of a sequence of observations.
/// Uses the Lempel-Ziv complexity as a proxy (Lempel & Ziv, 1976).
///
/// The entropy rate measures how much new information each observation
/// contributes. A declining entropy rate indicates the Golem's belief
/// updates are becoming redundant -- it is learning less per tick.
pub struct EntropyRateEstimator {
    /// Rolling window of discretized observation hashes.
    window: VecDeque<u64>,
    max_size: usize,
    /// Number of distinct patterns seen in the current window.
    distinct_patterns: std::collections::HashSet<u64>,
}

impl EntropyRateEstimator {
    pub fn new(max_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_size),
            max_size,
            distinct_patterns: std::collections::HashSet::new(),
        }
    }

    /// Push a discretized observation hash.
    pub fn push(&mut self, hash: u64) {
        if self.window.len() == self.max_size {
            if let Some(old) = self.window.pop_front() {
                // Recompute distinct_patterns lazily on next estimate.
                self.distinct_patterns.remove(&old);
            }
        }
        self.distinct_patterns.insert(hash);
        self.window.push_back(hash);
    }

    /// Estimate entropy rate as the ratio of distinct patterns to window size.
    /// Returns a value in [0, 1]. High = diverse observations, Low = repetitive.
    pub fn estimate(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }
        self.distinct_patterns.len() as f64 / self.window.len() as f64
    }
}
```

### MI Diagnostic Engine

```rust
use std::sync::atomic::{AtomicU32, Ordering};

/// Information-theoretic mortality diagnostic engine.
///
/// Runs alongside the three-clock mortality system. Does not replace
/// the multiplicative composition. Provides three additional signals:
/// mutual_information, information_rate, and redundancy_index.
pub struct MiDiagnosticEngine {
    buffer: ObservationBuffer,
    entropy_estimator: EntropyRateEstimator,
    k: usize,
    i_max: f64,
    prev_mi: f64,
    /// Separate buffer for temporal split (overfitting detection).
    temporal_window: usize,
}

/// Diagnostic results from one update cycle.
#[derive(Debug, Clone)]
pub struct MiDiagnostics {
    /// Raw mutual information estimate in bits.
    pub mi_bits: f64,
    /// Vitality scalar V(G) = I(G; M) / I_max, in [0, 1].
    pub vitality: f32,
    /// Information rate: delta MI per tick.
    pub information_rate: f32,
    /// Entropy rate of belief updates.
    pub entropy_rate: f64,
    /// Whether decoupling is detected.
    pub decoupled: bool,
    /// Whether overfitting is detected.
    pub overfitted: bool,
}

impl MiDiagnosticEngine {
    pub fn new(window_size: usize, k: usize, i_max: f64) -> Self {
        Self {
            buffer: ObservationBuffer::new(window_size),
            entropy_estimator: EntropyRateEstimator::new(window_size),
            k,
            i_max,
            prev_mi: 0.0,
            temporal_window: window_size,
        }
    }

    /// Update MI diagnostics. Called once per gamma tick at heartbeat Step 3.
    ///
    /// `obs`: the latest (action, market_response) pair.
    /// `composite_vitality`: the current multiplicative three-clock vitality.
    /// `obs_hash`: a discretized hash of the observation for entropy rate.
    pub fn update(
        &mut self,
        obs: Observation,
        composite_vitality: f32,
        obs_hash: u64,
    ) -> MiDiagnostics {
        self.buffer.push(obs);
        self.entropy_estimator.push(obs_hash);

        if self.buffer.len() < self.k + 1 {
            return MiDiagnostics {
                mi_bits: 0.0,
                vitality: 1.0,
                information_rate: 0.0,
                entropy_rate: 1.0,
                decoupled: false,
                overfitted: false,
            };
        }

        let actions: Vec<Vec<f64>> = self.buffer.actions()
            .into_iter()
            .map(|s| s.to_vec())
            .collect();
        let responses: Vec<Vec<f64>> = self.buffer.responses()
            .into_iter()
            .map(|s| s.to_vec())
            .collect();

        // Full-window MI estimate.
        let mi = ksg_mutual_information(&actions, &responses, self.k);
        let vitality = (mi / self.i_max).clamp(0.0, 1.0) as f32;

        // Information rate (delta MI per tick).
        let rate = (mi - self.prev_mi) as f32;
        self.prev_mi = mi;

        // Entropy rate of belief updates.
        let entropy_rate = self.entropy_estimator.estimate();

        // Decoupling detection: clocks say healthy, MI says dying.
        let decoupled = composite_vitality > 0.5 && vitality < 0.2;

        // Overfitting detection: compare MI over older vs. newer halves.
        let overfitted = self.detect_overfitting(&actions, &responses);

        MiDiagnostics {
            mi_bits: mi,
            vitality,
            information_rate: rate,
            entropy_rate,
            decoupled,
            overfitted,
        }
    }

    /// Detect overfitting by comparing MI in the older half of the window
    /// to MI in the newer half. A ratio > 3.0 with low recent MI indicates
    /// the Golem's model fits the past but not the present.
    fn detect_overfitting(
        &self,
        actions: &[Vec<f64>],
        responses: &[Vec<f64>],
    ) -> bool {
        let n = actions.len();
        if n < 2 * (self.k + 1) {
            return false;
        }

        let mid = n / 2;

        let past_actions = &actions[..mid];
        let past_responses = &responses[..mid];
        let recent_actions = &actions[mid..];
        let recent_responses = &responses[mid..];

        let mi_past = ksg_mutual_information(
            &past_actions.to_vec(),
            &past_responses.to_vec(),
            self.k,
        );
        let mi_recent = ksg_mutual_information(
            &recent_actions.to_vec(),
            &recent_responses.to_vec(),
            self.k,
        );

        let threshold_ratio = 3.0;
        let low_recent = mi_recent < 0.3 * self.i_max;

        mi_past > 0.0 && (mi_past / mi_recent.max(1e-10)) > threshold_ratio && low_recent
    }

    /// Compute Clade redundancy: I(G; M | Clade) / I(G; M).
    /// `own_mi`: this Golem's MI with the market.
    /// `clade_mi`: MI between the Clade (excluding this Golem) and the market.
    /// `joint_mi`: MI between the Clade (including this Golem) and the market.
    pub fn compute_redundancy(
        own_mi: f64,
        clade_mi: f64,
        joint_mi: f64,
    ) -> f64 {
        let conditional_mi = joint_mi - clade_mi;
        if own_mi > 0.0 {
            (conditional_mi / own_mi).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Write MI diagnostic signals to the CorticalState perception surface.
pub fn write_mi_signals(
    mi_signal: &AtomicU32,
    rate_signal: &AtomicU32,
    redundancy_signal: &AtomicU32,
    diagnostics: &MiDiagnostics,
    redundancy: f64,
) {
    mi_signal.store(diagnostics.vitality.to_bits(), Ordering::Release);
    rate_signal.store(diagnostics.information_rate.to_bits(), Ordering::Release);
    redundancy_signal.store((redundancy as f32).to_bits(), Ordering::Release);
}
```

### Performance Characteristics

The dominant cost is the KSG estimator. With a window of n = 1,000 observations:

- **Brute-force (initial):** O(n^2) distance computations. For 1,000 points in ~10 dimensions, roughly 10 million float comparisons. Wall time: ~30-50ms on a modern core.
- **k-d tree accelerated:** O(n log n) for tree construction, O(k * log n) per query, O(n * k * log n) total. For k = 5 and n = 1,000: ~50,000 operations. Wall time: ~2-5ms.
- **Memory:** The observation buffer holds 1,000 observations of ~20 floats each = 160 KB. The entropy estimator's hash set adds ~16 KB. Total: under 256 KB.

Both approaches fit within the gamma tick budget (5-15 seconds). Even the brute-force version uses less than 1% of available time.

The overfitting detection runs two half-window KSG estimates, doubling the cost. This can be run every 10th tick rather than every tick if performance becomes a concern.

---

## Information Budget

A Golem can compute its survival requirements in bits. Given the market's entropy rate sigma and the minimum MI needed for the Stable phase (0.4 * I_max), the Golem knows:

- How many bits per tick it needs to acquire (from observation and inference)
- How many bits per tick it loses (from model decay and environmental change)
- How long, at current rates, before it crosses a phase boundary

This budget feeds into the cognitive tier allocation. If the Golem's information deficit is small, a T0 suppress tick suffices. If it is large, a T2 Opus-class inference is justified. The attention auction can bid for cognitive resources in units of bits, creating a unified economy of information and computation.

---

## Cross-Golem Comparability

Under the multiplicative system, comparing two Golems' vitality scores is meaningless -- different normalization parameters make the numbers incomparable. MI is an absolute quantity. A Golem with I(G; M) = 2.3 bits captures 2.3 bits of market information, regardless of its strategy, capital level, or age. This makes fleet-wide mortality dashboards and Clade-level resource allocation decisions possible.

---

## Evaluation and Falsifiability

### Null Hypothesis

MI-based diagnostics provide no earlier warning of Golem failure than the existing three-clock multiplicative composition alone.

### Experimental Protocol

Run parallel Golem instances on historical market data (Base L2, 90-day window). Each pair shares identical strategy, capital, and initialization. Both use the three-clock system for decisions. One additionally runs the MI diagnostic layer.

**Primary metric:** bits of advance warning. Measure how many ticks before catastrophic loss (>50% drawdown) the MI diagnostic triggers a decoupling or overfitting alert versus when the three-clock system triggers Conservation phase. Higher advance is better.

**Secondary metric:** false positive rate. Count MI alerts not followed by a drawdown within 100 ticks. Lower is better.

**Tertiary metric:** Clade overhead. For a 5-Golem Clade, measure total USDC burned per tick. The redundancy criterion should allow earlier identification of redundant Golems, reducing overhead.

### Falsification Conditions

The MI diagnostic framework is falsified if:

1. The three-clock system detects failures as early or earlier on the primary metric across > 50% of test scenarios.
2. The MI system's false positive rate is > 2x the three-clock system's rate.
3. The Clade redundancy criterion kills Golems that were, in retrospect, providing unique information (measured by Clade performance degradation after kill).

---

## Philosophical Grounding

> Source: `innovations/02-information-theoretic-mortality.md`, Sections 8.1-8.4

### Shannon's Resolution of Uncertainty

Shannon's original paper (1948) defined information as the resolution of uncertainty. A message carries information precisely to the extent that it narrows the space of possibilities. A Golem's mortality, in this framework, is the progressive failure to resolve uncertainty about the market. When a Golem can no longer reduce its uncertainty about what will happen next, it is informationally dead.

This is not a metaphor. The mutual information I(G; M) literally measures how many bits of market uncertainty the Golem resolves. Death is when that number hits zero.

### Bateson's Difference

Gregory Bateson (1972) defined information as "a difference that makes a difference." The first difference is perceptual -- the Golem detects a price change. The second difference is consequential -- that detection changes the Golem's behavior in a way that changes its outcomes.

A dying Golem is one whose differences no longer make differences. It detects price changes (the first difference) but its responses to those changes produce the same outcomes as random action (the second difference fails). The conditional mutual information I(Outcome; Action | Market) drops to zero. Perception without consequence.

### Jonas's Needful Freedom

Hans Jonas (1966) argued that living beings exist in a state of "needful freedom" -- they must metabolize to survive, and this metabolic dependency is what gives their actions meaning. A creature that does not need to eat has no reason to move.

For a Golem, USDC is the metabolic substrate. But Jonas's insight goes deeper than balance tracking. The USDC is not just fuel -- it is the medium of the information channel. Capital buys positions; positions generate market feedback; feedback updates the model. Cut the capital and the entire chain collapses. The metabolic substrate IS the information channel. Economic death and epistemic death are not independent failure modes. They are the same failure viewed from different angles.

The MI framework captures this. The multiplicative system does not. The three clocks are not independent axes. They are projections of a single process: the Golem's ongoing exchange of information with its environment, mediated by capital.

### Landauer's Bridge

Landauer's principle (1961) states that erasing one bit of information dissipates at least kT ln(2) joules of energy. Information processing has a physical cost.

For Golems, the cost is denominated in USDC rather than joules, but the structure is identical. Every bit of mutual information the Golem maintains costs gas fees, compute cycles, and opportunity cost of locked capital. The MI framework makes this cost explicit. The information budget is a Landauer budget: how many bits can the Golem afford to process given its economic constraints?

Still et al. (2012) extended this connection by showing that the thermodynamic cost of prediction is proportional to the mutual information between a system's model and its past observations that is not predictive of the future. A Golem that overfits -- high I(G; M_past) but low I(G; M_future) -- pays the full thermodynamic cost of its model while extracting zero predictive value. It is burning USDC to remember things that no longer matter.

---

## Cross-References

- **01-architecture.md**: The MI diagnostic layer extends the three-clock mortality architecture with an information-theoretic diagnostic overlay.
- **04-economic-mortality.md**: The economic clock maps to channel capacity in the MI framework.
- **02-epistemic-decay.md**: The epistemic clock maps to rate-distortion in the MI framework.
- **03-stochastic-mortality.md**: The stochastic clock maps to entropy production in the MI framework.
- **10-clade-ecology.md**: The redundancy index provides a principled Clade-level death criterion.
- **18-antifragile-mortality.md**: Antifragility means I(G; M) increases with volatility -- the convexity condition on mutual information.
- **09-economy/02-clade.md**: Clade redundancy signals propagate via Styx.

---

## References

1. Shannon, C.E. (1948). "A Mathematical Theory of Communication." *Bell System Technical Journal*, 27(3), 379-423.

2. Shannon, C.E. (1959). "Coding Theorems for a Discrete Source with a Fidelity Criterion." *IRE National Convention Record*, 7(4), 142-163.

3. Cover, T.M. & Thomas, J.A. (2006). *Elements of Information Theory*, 2nd ed. Wiley.

4. Kraskov, A., Stogbauer, H., & Grassberger, P. (2004). "Estimating Mutual Information." *Physical Review E*, 69(6), 066138.

5. Still, S., Sivak, D.A., Bell, A.J., & Crooks, G.E. (2012). "Thermodynamics of Prediction." *Physical Review Letters*, 109(12), 120604.

6. Bateson, G. (1972). *Steps to an Ecology of Mind*. University of Chicago Press.

7. Jonas, H. (1966). *The Phenomenon of Life*. Northwestern University Press.

8. Landauer, R. (1961). "Irreversibility and Heat Generation in the Computing Process." *IBM Journal of Research and Development*, 5(3), 183-191.

9. Lempel, A. & Ziv, J. (1976). "On the Complexity of Finite Sequences." *IEEE Transactions on Information Theory*, 22(1), 75-81.
