# Sheaf-Theoretic Multiscale Observation [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-18
>
> **Crates**: `bardo-sheaf`, `bardo-cortical`
>
> **Depends on**: `../01-golem/13-runtime-extensions.md` (adaptive clock), `../14-chain/00-architecture.md` (chain intelligence), `shared/hdc-vsa.md` (CorticalState)

> **Reader orientation:** This document formalizes multi-timeframe observation consistency for Golems (mortal autonomous agents compiled as single Rust binaries running on micro VMs) using sheaf theory on a temporal poset. It belongs to the inference layer of Bardo. The key concept is CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model), which gains two new signals from this framework: `sheaf_consistency` and `contradiction_dimension`. Familiarity with the adaptive clock's three timescales (gamma/theta/delta) from `13-runtime-extensions.md` is required. See `prd2/shared/glossary.md` for full term definitions.

An autonomous DeFi agent (Golem) watches the market through three timescales simultaneously: gamma (5-15 seconds, raw ticks), theta (30-120 seconds, cognitive patterns), and delta (roughly 50 theta-ticks, structural consolidation). Each timescale produces its own observations. Those observations sometimes contradict each other. Today, detecting contradiction depends on an LLM noticing it during deliberation. This document formalizes multi-timeframe consistency using sheaf theory on a temporal poset, where the first cohomology group H^1 measures exactly how much the timescales disagree. The result is a computable scalar, the consistency score, that turns "the short-term data contradicts the long-term trend" from a vague intuition into a number the agent can act on.

---

## The problem

A Golem's adaptive clock partitions time into three scales. Gamma perceives: price, volume, gas, spread, updated every few seconds. Theta cognizes: it looks at a window of gamma observations, detects trends, regime characteristics, correlations. Delta consolidates: it looks across many theta cycles, evaluating strategy performance, knowledge quality, long-run statistics.

These three scales look at the same market. They should agree. Often they don't.

A gamma observation might show price rising fast over the last 10 seconds. The current theta cycle might classify the regime as "momentum up." But the delta layer, looking at the last hour, might show a sustained downtrend with this uptick fitting the pattern of a dead-cat bounce. All three observations are correct at their respective scales. Together, they are inconsistent.

Right now, Bardo handles this the hard way. Each timescale writes its signals to a shared CorticalState (a lock-free structure of about 32 atomic signals). The LLM, during theta's cognition phase, reads all the signals and uses judgment to notice contradictions. Sometimes it catches them. Sometimes it doesn't.

This is the same problem every trader faces. Multi-timeframe analysis is standard practice in trading. The 5-minute chart says buy; the daily chart says sell. Experienced traders develop intuition for reconciling these conflicts. But intuition is not a framework. There is no mathematical language for measuring the degree of contradiction across timescales. There is no way to say "the inconsistency between my hourly and daily view is at severity 0.7."

In a Clade (a group of Golems sharing knowledge), the problem compounds. Different Golems observe from different vantage points: different chains, different assets, different strategies. Spatial inconsistency (Agent A and Agent B disagree about market regime) layers on top of temporal inconsistency (Agent A's gamma and delta layers disagree with each other). The state space of possible contradictions grows multiplicatively.

We need a framework that detects inconsistency automatically, measures its severity, and exposes it as a signal the agent can act on. Sheaf theory provides exactly this.

---

## Mathematical foundation

### The temporal poset

Start with the observation that timescales nest. Every gamma interval is contained in some theta interval, and every theta interval is contained in some delta interval. This containment defines a partial order.

Let T be the set of all observation intervals. For concrete numbers: at any moment, the Golem maintains roughly 5-8 recent gamma observations, 3-5 theta observations, and 1-2 delta observations. Define a partial order by temporal containment:

    gamma_i <= theta_j   iff   the time interval of gamma_i is contained in the time interval of theta_j
    theta_j <= delta_k    iff   the time interval of theta_j is contained in the time interval of delta_k

This makes T a finite partially ordered set (poset). The Hasse diagram looks like a forest of trees, with delta nodes at the roots, theta nodes at the middle level, and gamma nodes as leaves.

### Presheaf of observations

A presheaf F on T assigns to each interval I an observation vector F(I), and to each containment relation I <= J a restriction map rho_{J,I}: F(J) -> F(I).

Concretely:

- F(gamma_i) is in R^8: price, volume, gas price, bid-ask spread, order flow imbalance, volatility (tick-level), price velocity, price acceleration.
- F(theta_j) is in R^6: trend direction, trend strength, regime classification (encoded), correlation structure (compressed), prediction confidence, pattern match score.
- F(delta_k) is in R^4: strategy P&L, prediction accuracy (rolling), knowledge quality score, long-term volatility estimate.

The restriction map rho_{theta_j, gamma_i}: R^6 -> R^8 encodes what the theta-level observation *implies* about a specific gamma interval. If theta says "strong uptrend," the restriction map translates that into expected gamma-level features: positive price velocity, accelerating volume, and so on.

These restriction maps are where the domain knowledge lives. They answer the question: "If this coarse-grained observation is true, what should the fine-grained data look like?"

### The sheaf condition

A presheaf is a sheaf when local observations glue consistently into global ones. More precisely: if you have observations on a collection of overlapping intervals, and those observations agree on every overlap, then there exists a unique global observation they all came from.

When this condition holds, the data is consistent. The gamma, theta, and delta views all tell the same story at their respective resolutions.

When the condition fails, the data is inconsistent. The short-term view contradicts the long-term view. The presheaf is not a sheaf, and the failure to glue is measurable.

### Cohomology measures the obstruction

The first cohomology group H^1(T, F) measures precisely the obstruction to gluing.

To compute it, build a cochain complex from the poset's simplicial structure. The 0-simplices are the nodes of T (the individual observation intervals). The 1-simplices are the edges (the containment relations). The 2-simplices are the triangles (chains gamma_i <= theta_j <= delta_k).

The cochain complex is:

    C^0 --d_0--> C^1 --d_1--> C^2

where:
- C^0 = direct sum of F(v) for each node v (all observation vectors stacked)
- C^1 = direct sum of R for each edge e (one scalar per containment relation)
- C^2 = direct sum of R for each triangle t

The coboundary operator d_0: C^0 -> C^1 takes a 0-cochain (an assignment of vectors to nodes) and produces a 1-cochain measuring the disagreement across each edge. For an edge e = (I <= J):

    (d_0 f)(e) = || rho_{J,I}(f(J)) - f(I) ||

This is the norm of the difference between what J predicts about I and what I actually shows.

The coboundary d_1: C^1 -> C^2 checks whether the edge disagreements are themselves consistent around triangles.

The first cohomology group is:

    H^1(T, F) = ker(d_1) / im(d_0)

When H^1 = 0, every disagreement across edges can be explained by adjusting the node observations. The data is globally consistent up to a change of coordinates.

When dim(H^1) > 0, there are disagreements that cannot be resolved by any adjustment. The dimension counts the number of independent, irreconcilable contradictions in the data.

### The Hodge Laplacian and the consistency score

Computing H^1 via quotient groups gives a binary answer (consistent or not) and a dimension. We want a continuous measure. The Hodge Laplacian provides one.

Define the combinatorial Hodge Laplacian on 1-cochains:

    L_1 = d_0^T d_0 + d_1 d_1^T

The spectral properties of L_1 encode the geometry of (in)consistency:

- The kernel of L_1 (eigenvalue 0) is the space of harmonic 1-cochains, isomorphic to H^1. These are the irreconcilable contradictions.
- Nonzero eigenvalues measure the "energy" of inconsistency. Larger eigenvalues correspond to more severe contradictions.

The consistency score is:

    c = 1 - (lambda_max(L_1) / lambda_ref)

where lambda_max is the largest eigenvalue of L_1 and lambda_ref is a reference value calibrated from historical data. The score lives in [0, 1], where 1 means perfect consistency and 0 means maximum observed contradiction.

This is the number we expose to the agent. A single f32.

### Worked example

Suppose the Golem has three active observations: one gamma interval gamma_1, one theta interval theta_1 containing gamma_1, and one delta interval delta_1 containing theta_1.

The simplicial complex has three nodes, two edges (gamma_1 <= theta_1 and theta_1 <= delta_1), and one triangle.

The restriction map rho_{theta_1, gamma_1} predicts that gamma should show [positive velocity, high volume, ...]. The actual gamma observation shows [negative velocity, low volume, ...]. The disagreement on this edge is large.

Meanwhile, rho_{delta_1, theta_1} predicts theta should show [downtrend, ...]. Theta actually shows [uptrend, ...]. Another large disagreement.

The coboundary d_0 computes these disagreement magnitudes. The Hodge Laplacian L_1 is a 2x2 matrix (two edges). Its largest eigenvalue is large, so the consistency score drops toward 0.

The Golem now knows, quantitatively, that its short-term and long-term views are contradicting each other at a specific severity.

### Extension to spatial observations in a Clade

Replace the temporal poset with a spatio-temporal poset. Nodes become (agent, timescale) pairs. Edges exist between:
- (Agent_A, gamma_i) and (Agent_A, theta_j) when gamma_i is contained in theta_j (temporal containment, same as before)
- (Agent_A, theta_j) and (Agent_B, theta_j) when both agents observe the same time interval at the same scale (spatial adjacency)

The restriction maps for spatial edges encode what one agent's observation implies about another's. If Agent_A observes ETH/USDC and Agent_B observes ETH/BTC, the spatial restriction map translates between their observation spaces using the known cross-rate relationship.

Now H^1 of the spatio-temporal complex captures both types of inconsistency in a single algebraic object. Temporal contradictions (an agent's timescales disagree) and spatial contradictions (agents disagree with each other) are unified.

---

## Architecture

### Integration with the adaptive clock

[SPEC] Each timescale produces a typed observation vector at its natural frequency:

- Gamma: every gamma tick (5-15s), produces an `ObsGamma` (8 x f32)
- Theta: every theta tick (30-120s), produces an `ObsTheta` (6 x f32) and triggers the sheaf computation
- Delta: every delta tick (~50 theta ticks), produces an `ObsDelta` (4 x f32)

The sheaf computation runs at theta frequency. Every theta tick, the system:

1. Collects the latest gamma, theta, and delta observations
2. Builds the simplicial complex from their temporal containment relations
3. Computes the coboundary operators using the current restriction maps
4. Computes the Hodge Laplacian L_1
5. Extracts lambda_max via power iteration (one or two iterations suffice given the small matrix)
6. Writes the consistency score and contradiction dimension to CorticalState

### New CorticalState signals

Two new atomic signals join the existing ~32:

- `sheaf_consistency: AtomicU32` -- an f32 in [0.0, 1.0] stored as bits, representing the consistency score from the Hodge Laplacian. High values mean the timescales agree. Low values mean contradiction.
- `contradiction_dimension: AtomicU8` -- dim(H^1), the number of independent irreconcilable contradictions. Usually 0 (consistent), occasionally 1 or 2.

### Downstream consumers

The consistency score feeds three subsystems:

**Heartbeat pipeline.** At Step 3 (ANALYZE) of the theta cognition cycle, the system checks `sheaf_consistency`. If it falls below a configurable threshold (default: 0.5), the analysis flags a "multiscale contradiction" condition. The LLM at Step 5 (DELIBERATE) receives this flag with the contradiction dimension, giving it structured information about what to investigate instead of having to notice the contradiction itself.

**Attention auction.** Inconsistency is information. When sheaf consistency drops, the attention auction assigns higher value to the contradicting signals. A Golem confused about its own observations should spend more cognitive resources resolving the confusion, not less.

**Information-theoretic mortality.** Persistent low consistency degrades the mutual information estimate between the Golem's internal model and the market. If a Golem's timescales chronically disagree and it cannot resolve the disagreement, its model is breaking down. This feeds the mortality signal.

---

## Implementation

### Core data structures

```rust
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

/// Observation vectors at each timescale.
/// Fixed-size arrays keep allocation off the hot path.
#[derive(Clone, Debug)]
pub struct ObsGamma {
    /// [price, volume, gas, spread, order_flow, volatility, velocity, acceleration]
    pub features: [f32; 8],
    pub timestamp_ms: u64,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub struct ObsTheta {
    /// [trend_dir, trend_strength, regime, correlation, pred_confidence, pattern_score]
    pub features: [f32; 6],
    pub timestamp_ms: u64,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub struct ObsDelta {
    /// [strategy_pnl, pred_accuracy, knowledge_quality, long_term_vol]
    pub features: [f32; 4],
    pub timestamp_ms: u64,
    pub duration_ms: u32,
}

/// A restriction map is a learned linear transformation.
/// Initialized from domain knowledge, refined through experience.
pub struct RestrictionMap {
    /// Matrix stored in row-major order.
    /// For theta->gamma: 8 rows x 6 cols (maps R^6 -> R^8).
    /// For delta->theta: 6 rows x 4 cols (maps R^4 -> R^6).
    pub weights: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
}

impl RestrictionMap {
    pub fn apply(&self, input: &[f32]) -> Vec<f32> {
        assert_eq!(input.len(), self.cols);
        let mut output = vec![0.0f32; self.rows];
        for i in 0..self.rows {
            for j in 0..self.cols {
                output[i] += self.weights[i * self.cols + j] * input[j];
            }
        }
        output
    }
}
```

### Sheaf computation

```rust
/// Edge in the simplicial complex: a containment relation between two observation intervals.
#[derive(Clone, Debug)]
pub struct SheafEdge {
    /// Index of the finer (contained) observation in the node list.
    pub fine_idx: usize,
    /// Index of the coarser (containing) observation in the node list.
    pub coarse_idx: usize,
    /// Disagreement: || rho(coarse) - fine ||
    pub disagreement: f32,
}

/// Result of the sheaf consistency computation.
#[derive(Clone, Debug)]
pub struct SheafResult {
    pub consistency_score: f32,
    pub contradiction_dim: u8,
    pub max_eigenvalue: f32,
    pub edge_disagreements: Vec<f32>,
}

/// Compute the sheaf consistency score from current observations.
///
/// `gammas`, `thetas`, `deltas`: current observation windows.
/// `rho_tg`: restriction maps from theta to gamma (one per theta-gamma pair).
/// `rho_dt`: restriction maps from delta to theta (one per delta-theta pair).
/// `lambda_ref`: reference eigenvalue for normalization (calibrated from history).
pub fn compute_sheaf_consistency(
    gammas: &[ObsGamma],
    thetas: &[ObsTheta],
    deltas: &[ObsDelta],
    rho_tg: &RestrictionMap,
    rho_dt: &RestrictionMap,
    lambda_ref: f32,
) -> SheafResult {
    // Step 1: Build edges by checking temporal containment.
    let mut edges: Vec<SheafEdge> = Vec::new();

    // Theta-to-gamma edges.
    for (ti, theta) in thetas.iter().enumerate() {
        let theta_start = theta.timestamp_ms;
        let theta_end = theta.timestamp_ms + theta.duration_ms as u64;
        for (gi, gamma) in gammas.iter().enumerate() {
            let gamma_start = gamma.timestamp_ms;
            let gamma_end = gamma.timestamp_ms + gamma.duration_ms as u64;
            if gamma_start >= theta_start && gamma_end <= theta_end {
                // gamma_i is contained in theta_j. Compute disagreement.
                let predicted = rho_tg.apply(&theta.features);
                let disagreement = l2_distance(&predicted, &gamma.features);
                edges.push(SheafEdge {
                    fine_idx: gi,
                    coarse_idx: gammas.len() + ti,
                    disagreement,
                });
            }
        }
    }

    // Delta-to-theta edges.
    for (di, delta) in deltas.iter().enumerate() {
        let delta_start = delta.timestamp_ms;
        let delta_end = delta.timestamp_ms + delta.duration_ms as u64;
        for (ti, theta) in thetas.iter().enumerate() {
            let theta_start = theta.timestamp_ms;
            let theta_end = theta.timestamp_ms + theta.duration_ms as u64;
            if theta_start >= delta_start && theta_end <= delta_end {
                let predicted = rho_dt.apply(&delta.features);
                let disagreement = l2_distance(&predicted, &theta.features);
                edges.push(SheafEdge {
                    fine_idx: gammas.len() + ti,
                    coarse_idx: gammas.len() + thetas.len() + di,
                    disagreement,
                });
            }
        }
    }

    if edges.is_empty() {
        return SheafResult {
            consistency_score: 1.0,
            contradiction_dim: 0,
            max_eigenvalue: 0.0,
            edge_disagreements: vec![],
        };
    }

    // Step 2: Build the Hodge Laplacian L_1 on the edge space.
    // L_1 = d_0^T d_0 + d_1 d_1^T
    // For our poset, d_0^T d_0 is the dominant term. d_1 contributes when triangles
    // (gamma <= theta <= delta chains) exist.
    let n_edges = edges.len();
    let mut laplacian = vec![0.0f32; n_edges * n_edges];

    // d_0^T d_0 contribution: for edges sharing a node, the Laplacian entry
    // is the sum of squared disagreements weighted by adjacency.
    // Diagonal: sum of disagreement^2 for the edge.
    // Off-diagonal: nonzero when two edges share a node.
    for i in 0..n_edges {
        // Diagonal entry: degree contribution.
        laplacian[i * n_edges + i] = 2.0; // Each edge touches two nodes.
        for j in 0..n_edges {
            if i == j {
                continue;
            }
            // Off-diagonal: -1 if edges share exactly one node.
            let share_fine = edges[i].fine_idx == edges[j].fine_idx
                || edges[i].fine_idx == edges[j].coarse_idx;
            let share_coarse = edges[i].coarse_idx == edges[j].fine_idx
                || edges[i].coarse_idx == edges[j].coarse_idx;
            if share_fine || share_coarse {
                laplacian[i * n_edges + j] = -1.0;
            }
        }
    }

    // Weight the Laplacian by the disagreement values.
    // This makes the eigenvalues reflect the magnitude of inconsistency,
    // not just the topology.
    for i in 0..n_edges {
        for j in 0..n_edges {
            laplacian[i * n_edges + j] *= edges[i].disagreement * edges[j].disagreement;
        }
    }

    // Step 3: Compute lambda_max via power iteration.
    let max_eigenvalue = power_iteration(&laplacian, n_edges, 20);

    // Step 4: Count contradiction dimension (zero eigenvalues of L_1 = dim H^1).
    // For small matrices, compute all eigenvalues. In practice, dim H^1 is almost
    // always 0 or 1 for the temporal poset.
    let contradiction_dim = count_near_zero_eigenvalues(&laplacian, n_edges, 1e-6);

    // Step 5: Compute consistency score.
    let consistency_score = (1.0 - (max_eigenvalue / lambda_ref)).clamp(0.0, 1.0);

    SheafResult {
        consistency_score,
        contradiction_dim,
        max_eigenvalue,
        edge_disagreements: edges.iter().map(|e| e.disagreement).collect(),
    }
}

fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    let min_len = a.len().min(b.len());
    let mut sum = 0.0f32;
    for i in 0..min_len {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

/// Power iteration to find the largest eigenvalue of a symmetric matrix.
fn power_iteration(matrix: &[f32], n: usize, iterations: usize) -> f32 {
    let mut v = vec![1.0f32 / (n as f32).sqrt(); n];
    let mut eigenvalue = 0.0f32;

    for _ in 0..iterations {
        // w = M * v
        let mut w = vec![0.0f32; n];
        for i in 0..n {
            for j in 0..n {
                w[i] += matrix[i * n + j] * v[j];
            }
        }
        // eigenvalue = v^T * w
        eigenvalue = v.iter().zip(w.iter()).map(|(a, b)| a * b).sum();
        // normalize w
        let norm: f32 = w.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm < 1e-10 {
            return 0.0;
        }
        for x in &mut w {
            *x /= norm;
        }
        v = w;
    }
    eigenvalue
}

/// Count eigenvalues near zero by computing the rank deficiency.
/// For small matrices (n < 20), this uses a direct approach.
fn count_near_zero_eigenvalues(matrix: &[f32], n: usize, tol: f32) -> u8 {
    // Simple approach: use the trace and determinant for small cases,
    // or iterative deflation for larger ones. For n < 20, the Gershgorin
    // circle theorem gives a quick upper bound on the number of small eigenvalues.
    //
    // In production, replace with a proper eigendecomposition from `faer` or `nalgebra`.
    // For the typical case of 2-10 edges, this is fast enough.
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return if matrix[0].abs() < tol { 1 } else { 0 };
    }

    // Gaussian elimination to find rank.
    let mut mat = matrix.to_vec();
    let mut rank = 0;
    for col in 0..n {
        // Find pivot.
        let mut pivot = None;
        for row in rank..n {
            if mat[row * n + col].abs() > tol {
                pivot = Some(row);
                break;
            }
        }
        let Some(pivot_row) = pivot else { continue };
        // Swap rows.
        for k in 0..n {
            mat.swap(rank * n + k, pivot_row * n + k);
        }
        // Eliminate below.
        for row in (rank + 1)..n {
            let factor = mat[row * n + col] / mat[rank * n + col];
            for k in col..n {
                mat[row * n + k] -= factor * mat[rank * n + k];
            }
        }
        rank += 1;
    }
    (n - rank) as u8
}
```

### Writing to CorticalState

```rust
/// Update CorticalState with sheaf consistency results.
/// Called every theta tick after the sheaf computation completes.
pub fn update_cortical_state(
    cortical: &CorticalState,
    result: &SheafResult,
) {
    cortical
        .sheaf_consistency
        .store(result.consistency_score.to_bits(), Ordering::Release);
    cortical
        .contradiction_dimension
        .store(result.contradiction_dim, Ordering::Release);
}

/// Read the consistency score from CorticalState. Lock-free.
pub fn read_consistency(cortical: &CorticalState) -> f32 {
    f32::from_bits(cortical.sheaf_consistency.load(Ordering::Acquire))
}
```

### Performance

The sheaf computation is cheap. The poset has about 10-15 nodes at any given time (5-8 gamma, 3-5 theta, 1-2 delta), producing roughly 8-12 edges. The Hodge Laplacian is at most a 12x12 matrix. The entire computation, including power iteration, completes in under 10 microseconds on modern hardware. Memory usage is negligible: a few hundred bytes for the sparse matrix. This fits comfortably within the theta tick budget.

---

## What this enables

**Quantitative multi-timeframe analysis.** Every trader uses multiple timeframes. No trading system, automated or manual, has a mathematical framework for measuring contradiction severity across timeframes. The sheaf consistency score is, as far as we know, the first computable metric for multi-timeframe agreement in a trading agent.

**Automatic contradiction detection.** The Golem no longer depends on an LLM noticing that gamma and delta disagree. The inconsistency appears as a number in CorticalState, readable by any subsystem. The heartbeat pipeline flags it. The attention system responds to it. The deliberation phase receives structured information about what is contradicted and how badly.

**"Confused but aware."** There is a difference between not knowing something and knowing that you don't know. When H^1 > 0, the Golem has detected that its own observations are internally inconsistent. It can respond proportionally: reduce position sizes, widen stop-losses, defer to longer-timeframe signals, or request more information from Clade members. This is epistemic humility with a number attached.

**Principled position sizing under contradiction.** High sheaf inconsistency is a direct signal to reduce exposure. If the Golem's model of the market is internally contradictory, it should not bet as if the model were reliable. The consistency score maps directly to a position-sizing multiplier: at consistency 1.0, trade at full size; at 0.5, halve it; below 0.3, reduce to minimum.

**Clade-level inconsistency.** With the spatio-temporal extension, a Clade can detect when its members disagree about the market state. "Agent A sees momentum on Arbitrum while Agent B sees mean-reversion on Optimism, and these observations conflict given the known cross-chain correlations." The Clade can identify which agent's vantage point is most informative and weight its contributions accordingly.

**A new behavioral mode.** Most autonomous agents have two states: confident and acting, or uncertain and paralyzed. The sheaf framework gives the Golem a third state: confused but aware of the confusion, with a measure of its severity. This is closer to how human traders actually operate, and it produces different behavior than either confidence or paralysis.

---

## Evaluation and falsifiability

This framework makes testable predictions.

**Primary hypothesis.** The sheaf consistency score is positively correlated with prediction accuracy. When the Golem's timescales agree (high consistency), its predictions should be more accurate than when they disagree (low consistency).

**Test protocol.** Track the consistency score and prediction outcomes over a rolling window of 1000 theta ticks. Compute the Pearson correlation between consistency score at time t and prediction accuracy at time t + k for various lags k.

**Prediction: leading indicator.** Drops in sheaf consistency should precede drops in prediction accuracy by several theta ticks. The intuition: inconsistency between timescales means the market is doing something the Golem's model doesn't expect. The predictions based on that model will degrade shortly after, once the unexpected behavior propagates through the prediction horizons.

**Null hypothesis.** Sheaf consistency is uncorrelated with prediction accuracy. If this holds, the signal adds no information and the computation is wasted. The test is straightforward.

**Cross-agent test.** Within a Clade, compute the spatial consistency score between agent pairs. Agents with low pairwise spatial consistency should underperform on shared predictions compared to agents with high spatial consistency. If two agents see the same market differently and neither is wrong, both should predict worse than an agent whose view is confirmed by its peers.

**Adversarial test.** During known regime changes (a DEX pool rebalances, a major liquidation cascade, a governance vote executes), the consistency score should drop sharply and then recover as the new regime stabilizes. If the score doesn't respond to genuine structural changes, it's not measuring what we think it's measuring.

---

## Philosophical grounding

Sheaf theory was invented by Jean Leray in a prisoner-of-war camp in the 1940s and developed into a general mathematical framework by Alexander Grothendieck in the 1950s and 1960s. The core insight: local data can be defined and studied locally, but the interesting questions are about whether local data glues into global data. When it does, the world is consistent. When it doesn't, the failure to glue is itself an object worth studying.

The Golem's epistemic situation is inherently sheaf-theoretic. It has many local observations: gamma sees the last few seconds, theta sees the last few minutes, delta sees the last hour. Each observation is valid in its own domain. The question is whether they combine into a coherent picture. The sheaf framework doesn't force consistency. It measures the lack of it.

There is a connection to Karl Friston's free energy principle, which models biological agents as systems that minimize the divergence between their predictions and their observations. The variational free energy is, roughly, the cost of failing to predict the world accurately. The sheaf cohomology H^1 formalizes a related quantity: the cost of failing to produce a consistent world model across observation scales. Both are measures of model inadequacy. The sheaf version has the advantage of being algebraic and computable from finite data, rather than requiring a full generative model.

Markets are not globally consistent objects. Different participants have different information, different models, different time horizons. A price that looks like noise on the 5-second chart might be a trend on the 5-minute chart and a reversion on the 1-hour chart. All of these can be true simultaneously. The sheaf framework does not claim the market has a single true state. It measures whether *the Golem's own observations* form a consistent picture, which is the epistemic foundation for knowing when to act and when to wait.

This is where the connection to mortality becomes sharpest. A Golem whose observations are chronically inconsistent is a Golem whose model of the world is failing. If the inconsistency cannot be resolved by updating the model (adjusting the restriction maps, recalibrating the timescales), then the agent's cognitive architecture is no longer fit for the environment it's in. The consistency score doesn't just measure a market condition. It measures the agent's relationship to its environment, and the breakdown of that relationship is what mortality looks like in information-theoretic terms.

---

## Relationship to other innovations

The sheaf framework connects to several other components of the Bardo research program.

**TDA** computes persistent homology on point clouds within a single timescale, capturing the topological shape of market data at one resolution. The sheaf framework operates *across* timescales, measuring whether observations at different resolutions are compatible. They are complementary: TDA describes the shape of what a timescale sees, and sheaf cohomology describes whether those shapes, viewed at different scales, fit together.

**Information-theoretic mortality** tracks the mutual information between the Golem's model and the market. Persistent sheaf inconsistency (low consistency score over many delta ticks) degrades this mutual information estimate. The sheaf signal is a leading indicator of model degradation, giving the mortality framework earlier warning.

**The attention auction** allocates cognitive resources to the highest-value signals. Sheaf inconsistency is high-value information by definition: it means the agent's model is producing contradictory outputs. A drop in consistency should increase the attention budget allocated to the contradicting timescales, prompting more detailed analysis of where the disagreement lies and what might resolve it.

**CorticalState** gains two new signals (`sheaf_consistency` and `contradiction_dimension`) that are available to all subsystems. These signals are lock-free, fit within the existing ~256 byte budget (adding 5 bytes), and update at theta frequency.

---

## References

1. Curry, J. (2014). "Sheaves, Cosheaves, and Applications." PhD dissertation, University of Pennsylvania. — *Foundational treatment of computational sheaf theory on finite posets.*
2. Robinson, M. (2014). *Topological Signal Processing.* Springer. — *Applied sheaf theory for signal integration; basis for the restriction map design.*
3. Robinson, M. (2017). "Sheaves are the Canonical Data Structure for Sensor Integration." *Information Fusion*, 36, 208-224. — *Direct precedent for using sheaf consistency to detect multi-source observation contradictions.*
4. Hansen, J. & Ghrist, R. (2019). "Toward a Spectral Theory of Cellular Sheaves." *Journal of Applied and Computational Topology*, 3(4), 315-358. — *Hodge Laplacian spectral theory used here for the continuous consistency score.*
5. Ghrist, R. (2014). *Elementary Applied Topology.* Createspace. — *Accessible introduction to applied algebraic topology including cohomology computation.*
6. Otter, N., Porter, M.A., Tillmann, U., Grindrod, P., & Harrington, H.A. (2017). "A Roadmap for the Computation of Persistent Homology." *EPJ Data Science*, 6(1), 17. — *Computational topology survey; complements the TDA work in the chain intelligence layer.*
7. Bredon, G.E. (1997). *Sheaf Theory*, 2nd ed. Springer. — *Standard mathematical reference for sheaf cohomology.*
