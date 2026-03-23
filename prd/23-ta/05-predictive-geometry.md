<!-- prd2/23-ta/05-predictive-geometry.md -->
<!-- Source: tmp/research/witness-research/new/ta/05-predictive-geometry.md -->

> **Version**: 1.0 | **Status**: Active | **Section**: 23-ta
>
> **Crates**: `bardo-ta-predictive-geometry`
>
> **Cross-references**:
> - [01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- the 32-signal perception surface where `trajectory_confidence` and `topology_signal` are written for decision-making
> - [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- the 9-step decision cycle whose Theta ticks run full topological analysis and trajectory forecasting
> - [01-golem/17-prediction-engine.md](../01-golem/17-prediction-engine.md) -- the Oracle prediction pipeline that consumes topological features as inputs to trajectory forecasts
> - [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- HDC foundations for encoding trajectory predictions as hypervectors that support efficient similarity queries
> - [23-ta/00-witness-as-technical-analyst.md](00-witness-as-technical-analyst.md) -- Doc 0: system context including the existing TDA pipeline that this document extends into prediction
> - [23-ta/01-hyperdimensional-technical-analysis.md](01-hyperdimensional-technical-analysis.md) -- Doc 1: HDC pattern algebra providing the encoding substrate for topological features
> - [23-ta/02-spectral-liquidity-manifolds.md](02-spectral-liquidity-manifolds.md) -- Doc 2: Riemannian manifold and geodesics providing geometric constraints on predicted trajectories
> - [23-ta/04-causal-microstructure-discovery.md](04-causal-microstructure-discovery.md) -- Doc 4: causal graph edges that constrain which trajectory paths are physically possible

---

> **Reader orientation:** This document converts topological data analysis (TDA) into trajectory forecasting for the Golem (mortal autonomous DeFi agent) runtime. It belongs to the **TA research layer** (Doc 5 of 10) and covers persistence landscapes from DeFi state spaces, topological constraints on market trajectories, and how persistent homology features feed into the prediction engine. You should understand algebraic topology basics (Betti numbers, persistent homology) and DeFi execution mechanics. For Bardo-specific terms, see `prd2/shared/glossary.md`.

# Predictive geometry: from topological detection to trajectory forecasting

**Audience:** Researchers and engineers familiar with Rust, algebraic topology, and DeFi. Assumes familiarity with Doc 0 (system architecture), Doc 2 (spectral liquidity manifolds), and the existing TDA pipeline from innovations/01.

## Abstract

The Golem's existing TDA pipeline computes persistence diagrams from observation windows at Gamma frequency and classifies the current regime -- calm, trending, volatile, crisis -- by tracking Betti numbers and Wasserstein distances between consecutive diagrams. This is reactive. The Golem knows where it is but not where it is going.

Topology constrains dynamics. A manifold with two disconnected components cannot smoothly merge into one without passing through a specific topological event. A space with a persistent 1-cycle implies oscillatory behavior that will continue until the cycle collapses. The shape of the observation space limits what can happen next.

This document extends TDA from regime detection to trajectory forecasting. We lift persistence diagrams into Bubenik's persistence landscape representation -- a Banach space where statistical operations are well-defined -- and learn a mapping from landscape configurations to families of predicted trajectories. Topological constraints filter physically impossible paths. The result is a forecasting engine that does not predict a single future but narrows the space of possible futures based on the current topology. Integration with the heartbeat places landscape computation at Gamma frequency, trajectory forecasting at Theta, model refinement at Delta, and topological replay/counterfactual generation during dream cycles.

## The problem

### What the existing TDA pipeline does

The innovations/01 implementation builds a Vietoris-Rips complex from each observation window and extracts a persistence diagram: a multiset of (birth, death) pairs recording when topological features appear and disappear across filtration scales. The pipeline tracks four Betti numbers (β_0 through β_3) and computes the Wasserstein distance between consecutive diagrams. When that distance exceeds a threshold, the system flags a regime transition.

This works. It detects regime changes 10 to 100 ticks ahead of statistical methods because topological features shift before statistical moments do. A brewing crisis shows up as an increase in β_0 (the observation space fragmenting into disconnected clusters) before the variance of returns spikes.

But detection is not prediction.

### The gap

Knowing "we just entered a volatile regime" is better than not knowing. It is not enough. The Golem needs to know what happens *inside* the regime. What trajectories are possible? How long will this regime persist? What topological events would signal a transition to the next regime, and how far away are those events?

A persistence diagram is a snapshot. It tells you the topology at time t. It does not tell you the topology at time t+1, or what path the system will trace between now and then.

The gap is between topological state (what the shape is) and topological dynamics (how the shape evolves and what that evolution implies for observable quantities like price, liquidity, and volatility).

### Why topology constrains trajectories

Consider a persistence diagram with β_0 = 2 at time t. The observation space has two disconnected components. For the system to reach β_0 = 1 (a single connected component), something must bridge the gap between the components. In DeFi terms, if liquidity has fragmented into two separate clusters -- say concentrated around two distant tick ranges in a Uniswap v3 pool -- the price cannot smoothly transition from one cluster to the other without either new liquidity connecting the ranges or existing liquidity migrating.

This is not a statistical correlation. It is a topological constraint. The space of possible trajectories is bounded by the current topology.

Similarly, β_1 > 0 means a loop exists in the observation space. In fee-IL (impermanent loss) space for an LP position, a persistent loop means fee accrual and IL are cycling: IL increases, fees compensate, IL increases again, fees compensate again. The loop constrains the trajectory to oscillatory paths. If the loop collapses (β_1 drops to 0), the oscillation breaks and one-directional IL accumulation becomes possible.

The key insight is that these constraints are *qualitative*, not quantitative. A statistical model might predict that price will stay within a range with 70% probability. A topological constraint says that certain transitions are *impossible* without intermediate events. The constraint does not give you a probability; it gives you a logical boundary on the set of reachable states. This is a different kind of information, and it composes well with statistical predictions: the topology restricts the domain, and statistics operate within that restricted domain.

The existing TDA pipeline throws away this predictive information. It detects that β_0 changed or β_1 changed, but it does not ask: given the current β values and the full persistence diagram, what trajectories are topologically compatible?

### What prediction means here

This document does not attempt to predict the future price of ETH. That would be silly.

What we predict is a *family* of trajectories -- a set of possible paths through the observation space, each carrying a probability weight. The family is constrained by topology (impossible paths are removed) and shaped by historical precedent (paths that followed similar topological configurations in the past are weighted higher).

The utility is not point prediction. It is narrowing the space of possibilities. Instead of "anything could happen," the Golem gets "given the current topological structure, these are the families of behavior that are geometrically possible, and here is how they are weighted by historical analogy." That is enough to inform position sizing, risk management, and attention allocation -- which is what the Oracle actually needs.

A concrete example: the observation space has β_0 = 1 (connected), β_1 = 2 (two loops), and the landscape derivative is small (topology is stable). The trajectory family might contain: (a) continued oscillation within both loops, probability 0.6; (b) collapse of one loop with continued oscillation in the other, probability 0.25; (c) collapse of both loops, probability 0.1; (d) appearance of a new component (β_0 increase), probability 0.05. The Oracle sees this and knows the most likely future involves oscillation. It positions accordingly.

## Mathematical foundations [SPEC]

### The problem with persistence diagrams as a feature space

A persistence diagram is a multiset of points in the half-plane above the diagonal. It summarizes topology well, but it lives in an awkward mathematical space. The space of persistence diagrams -- equipped with the Wasserstein or bottleneck metric -- is not a vector space. You cannot add two diagrams. You cannot scale a diagram by a real number. You cannot compute a meaningful mean of two diagrams by averaging their points, because diagrams can have different numbers of points, and the correspondence between points across diagrams is not canonical.

This is a serious problem for prediction. All standard statistical methods (regression, PCA, hypothesis testing, time-series modeling) assume the data lives in a vector space, or at least in a space where arithmetic operations are well-defined. The Wasserstein distance gives us a metric, but a metric alone does not support the operations we need: averaging topological summaries over time windows, computing rates of change, or defining a regression target.

Several approaches exist for vectorizing persistence diagrams: persistence images (Adams et al., 2017), Betti curves, persistence entropy. We use persistence landscapes because they are the only representation that is simultaneously stable, invertible, and lives in a Banach space with a well-defined inner product.

### Persistence landscapes

Bubenik (2015) solved the vectorization problem with persistence landscapes: a stable, invertible transformation that maps persistence diagrams into a Banach space of functions where linear algebra works.

Given a persistence diagram D with features {(b_i, d_i)}, define a tent function for each feature:

```
Λ_i(t) = max(0, min(t - b_i, d_i - t))
```

This is a triangle centered at (b_i + d_i)/2 with height (d_i - b_i)/2. Long-lived features produce tall, wide tents. Short-lived features produce small tents. Features on the diagonal (b = d) produce zero tents.

The k-th persistence landscape function is:

```
λ_k(t) = k-max{Λ_i(t) : (b_i, d_i) ∈ D}
```

where k-max denotes the k-th largest value. So λ_1(t) is the upper envelope of all tent functions. λ_2(t) is the second-highest value at each t. And so on.

The full persistence landscape is the sequence (λ_1, λ_2, λ_3, ...). In practice, we truncate to the first K layers, where K is chosen based on the complexity of the diagrams we expect.

Why landscapes over diagrams? Four properties matter:

**Vector space structure.** Landscapes live in L^p function space. You can add them, scale them, compute norms and inner products. The mean of N landscapes is just (1/N) times the sum, computed pointwise. Try doing that with raw persistence diagrams.

**Stability.** Small perturbations in the input data produce small perturbations in the landscape (Bubenik 2015, Theorem 13). The L^p distance between landscapes is bounded by the p-Wasserstein distance between the underlying diagrams. Noisy data does not produce wild landscape oscillations.

**Invertibility.** The persistence diagram can be recovered from the landscape. No information is lost in the transformation, only reorganized into a more tractable form.

**Separability.** Different topological regimes produce geometrically distinct landscapes. A regime with one large persistent loop produces a landscape dominated by a single tall tent in dimension 1. A regime with many small loops produces a landscape with many small tents spread across the parameter range. These are easy to distinguish in L^p distance, even when the raw persistence diagrams look superficially similar.

### The landscape Banach space

The space of persistence landscapes inherits Banach space structure from L^p:

```
||λ||_p = (Σ_{k=1}^{K} ∫ |λ_k(t)|^p dt)^{1/p}
```

For p = 2, this is a Hilbert space with inner product:

```
⟨λ, μ⟩ = Σ_{k=1}^{K} ∫ λ_k(t) · μ_k(t) dt
```

This inner product enables:
- **Distance:** ||λ - μ||_2 measures how different two topological configurations are.
- **Mean:** λ̄ = (1/N) Σ_i λ_i computes an average topology over a window.
- **Variance:** Var(λ) = (1/N) Σ_i ||λ_i - λ̄||_2^2 measures topological stability.
- **Derivative:** dλ/dt ≈ (λ(t) - λ(t-1)) / Δt captures the rate of topological change.

The derivative is the critical one for prediction. A persistence landscape that is changing rapidly (large ||dλ/dt||) signals an impending regime transition. The direction of the change (which layers are growing, which are shrinking) constrains what transitions are possible.

### Discretization

For computation, we discretize landscapes. Sample each λ_k at N evenly spaced points t_0, t_1, ..., t_{N-1} over the range [t_min, t_max]. The landscape becomes a matrix L ∈ R^{K × N}, where L[k][i] = λ_k(t_i).

This matrix is the feature vector for all downstream computation. Distances become matrix norms. Means become element-wise averages. The landscape library stores these matrices for nearest-neighbor retrieval.

The choice of K and N matters. K = 5 layers capture the five most prominent topological features at each parameter value. N = 64 samples provide sufficient resolution for the filtration ranges we encounter in DeFi observation windows. These are configurable; the implementation accepts both as parameters.

### Topology-to-trajectory mapping

Define the prediction problem. Given a persistence landscape λ(t) at time t, predict a distribution over trajectories for the observation space over the horizon [t, t+H], where H is measured in Gamma ticks.

A trajectory here is a sequence of observation vectors: (x_{t+1}, x_{t+2}, ..., x_{t+H}). In DeFi context, each x is a vector of pool metrics: price, liquidity depth, volume, fee rate, utilization, and so on.

The mapping:

```
Φ: λ(t) → P(Trajectory | λ(t))
```

is learned from historical data. The learning procedure:

1. Compute persistence landscapes for each observation window in the historical record.
2. Record the actual trajectory that followed each landscape.
3. Build a library of (landscape, trajectory) pairs.
4. For a new landscape λ(t), find the K nearest neighbors in the library (by L^2 distance), and construct the predicted trajectory distribution from their associated trajectories.

This is deliberately non-parametric. We are not fitting a neural network or even a linear model to the landscape-trajectory relationship. The topology of DeFi observation spaces shifts qualitatively over time (new pool types, new protocol mechanics, new participant behaviors), and a non-parametric approach degrades gracefully when the current landscape has no close historical matches: the nearest-neighbor distances are large, the predicted trajectories are diverse, and the confidence is low. This uncertainty is honest rather than hidden inside a model's confidence calibration.

Why not a parametric model? Two reasons. First, the relationship between topology and trajectory is not stationary. A given topological configuration in March 2024 might produce different trajectories than the same configuration in September 2024, because the market participants, liquidity conditions, and protocol parameters have all changed. A parametric model trained on March data would need retraining for September data, and the Golem has no concept of "retraining" -- it has continuous experience. The non-parametric library naturally adapts: old entries decay in fitness, new entries enter, and the weighted average shifts toward recent experience.

Second, the landscape space is high-dimensional (K * N dimensions per homology degree, times the number of homology degrees). Parametric models in high dimensions need large training sets to avoid overfitting. The Golem's library grows one entry per observation window per Delta tick. That is slow. A non-parametric model with adaptive bandwidth handles small datasets gracefully -- it just reports low confidence when the library is sparse.

The kernel regression refinement weights neighbors by landscape similarity:

```
P̂(trajectory | λ) = Σ_i w_i · δ(trajectory_i)
```

where:

```
w_i = K_h(||λ - λ_i||_2) / Σ_j K_h(||λ - λ_j||_2)
```

and K_h is a Gaussian kernel with bandwidth h. Close landscape matches dominate the prediction. Distant matches contribute noise that washes out, leaving the prediction driven by topologically similar historical episodes.

### Topological constraints on trajectories

The kernel regression produces a distribution over trajectories. But not all trajectories in that distribution are topologically possible given the current state. Constraints filter the impossible ones.

**Component count constraints (β_0).**

When β_0 = 1 (single connected component), smooth transitions dominate. The observation space is connected, so prices, liquidity, and other metrics can move continuously without jumping.

When β_0 > 1 (multiple components), the system is fragmented. A trajectory that assumes smooth convergence to a single price level is topologically inconsistent: the components must first merge, which requires specific bridging events (new liquidity connecting the fragmented ranges, or removal of the boundary between clusters). Trajectories that ignore the component structure get filtered.

When β_0 is increasing over time, fragmentation is in progress. Predict continued divergence until the fragmenting process completes or reverses.

**Loop constraints (β_1).**

When β_1 > 0, loops exist. In the observation space, loops correspond to cyclic dynamics: quantities that return to previous values after excursions. Fee-IL cycles in LP positions, rate-utilization oscillations in lending markets, basis-funding loops in derivatives.

A persistent loop (high death - birth value in the persistence diagram) constrains trajectories to maintain the oscillatory pattern. The trajectory family should include cyclical paths with period roughly matching the loop's characteristic scale.

When β_1 is decreasing, loops are collapsing. The cyclic behavior is weakening. Predict that oscillation amplitude will decrease and mean-reversion will weaken. Trajectories that assume continued strong oscillation should be downweighted.

**Void constraints (β_2).**

Voids in the observation space (β_2 > 0) represent regions that are topologically enclosed but empty. In DeFi terms, these are combinations of observables that *used to be reachable* but are no longer accessible. A void in price-liquidity space means there is a price range where liquidity once existed but has been completely withdrawn.

Trajectories that pass through void regions should be filtered or heavily penalized. The topology says those states are unreachable under current conditions.

**Rate-of-change constraints.**

The Morse inequalities provide lower bounds on the number of critical points (transitions) required to change the Betti numbers by specified amounts. If the current β_0 is 3 and the predicted trajectory implies β_0 = 1, at least 2 merging events must occur. Each merging event corresponds to a concrete market action (new liquidity bridging two clusters). The time required for those events provides a minimum time-to-transition.

This is where the connection to Doc 2's manifold becomes load-bearing. The spectral liquidity manifold provides geodesic distances between states. A topological transition that requires merging two components corresponds to a geodesic connecting two points on the manifold. The geodesic length (measured in the Riemannian metric from Doc 2) gives a lower bound on the time-to-transition, because the system must traverse that geodesic distance in state space.

### DeFi-specific topological features

The abstract topological constraints become concrete when applied to specific DeFi primitives.

**LP positions in concentrated liquidity (Uniswap v3).**

The observation space for an LP position lives in (price, liquidity_depth, fee_rate, IL) coordinates. Topological features in this space have direct market interpretations:

β_0 increase in price-liquidity space: concentrated liquidity is fragmenting. Some LPs are abandoning the current price range and repositioning elsewhere. The pool's liquidity is splitting into disconnected clusters around different tick ranges. This predicts increased slippage for swaps that cross the gaps between clusters.

β_1 in fee-IL space: a persistent loop means fee accrual and IL are locked in a compensating cycle. Fees increase as volume passes through the position's range, IL increases as price moves away, fees compensate on the return. This loop is the healthy state for an active LP position. It constrains trajectories to oscillatory P&L patterns.

β_1 collapse in fee-IL space: the compensating cycle broke. Either volume dried up (fees stopped accruing) or price moved away permanently (IL became one-directional). The trajectory constraint shifts from oscillatory to monotonically deteriorating. This is an exit signal for the position.

**Lending markets (Aave, Compound).**

The observation space includes (utilization_rate, borrow_rate, supply_rate, health_factor_distribution).

β_0 in utilization-rate space: bimodal utilization means some lending markets are stressed (utilization near 100%, rates at the kink) while others are calm (utilization below 50%, rates at the base level). Two disconnected clusters in this space predict that the stressed and calm markets will evolve independently unless a common shock (ETH price move, stablecoin depeg) forces them to converge.

β_1 in rate-borrow space: oscillatory borrowing patterns. Borrowers enter when rates drop, driving utilization up, which pushes rates up, which drives borrowers out, which drops utilization and rates. A persistent loop here means the lending market is self-regulating. Trajectory prediction: rates will oscillate within the loop's characteristic amplitude and frequency.

β_2 (void) in utilization-rate-health space: a region of utilization-rate combinations that used to exist but no longer does. This signals a regime shift in the lending market. Trajectories that assume the market can return to that region should be filtered.

**Derivatives (perpetuals, options).**

The observation space includes (funding_rate, open_interest, basis, implied_volatility).

β_0 in funding-OI space: fragmented derivatives markets. Some contracts are heavily long-biased (positive funding, high OI) while others are neutral or short-biased. Predict that these fragments will evolve independently unless a liquidation cascade or large directional trade bridges them.

β_1 in basis-funding space: the basis trade cycle. Arbitrageurs go long spot / short perp when basis is positive, collecting funding. Their activity compresses basis, which reduces funding, which causes some arbs to exit, which widens basis again. A persistent loop constrains trajectories to continued basis oscillation within the loop's bounds.

β_1 collapse in basis-funding: the arbitrage loop broke. This happens when funding becomes too volatile for arbs to hold positions, or when basis widens beyond the arb's capital constraints. Predict one-directional basis movement.

## Architecture [SPEC]

### System overview

The `PredictiveTopology` engine wraps the existing `TdaAnalyzer` and extends it with four new capabilities:

1. **Landscape computation.** Transforms persistence diagrams into persistence landscape matrices.
2. **Landscape library.** An indexed historical collection of landscapes paired with the trajectories that followed them.
3. **Trajectory regression.** K-nearest-neighbor lookup in landscape space with kernel-weighted trajectory combination.
4. **Constraint engine.** Filters predicted trajectories against topological feasibility.

The engine implements the `TaAnalyzer` trait from Doc 0:

```
gamma_tick:  Compute persistence diagram (via TdaAnalyzer)
             → transform to landscape
             → look up trajectory family
             → write trajectory_confidence to TaCorticalExtension

theta_tick:  Aggregate Gamma-rate landscapes
             → produce TopologicalForecast for Oracle
             → report constraint-filtered trajectory families as TaInsights

delta_tick:  Update regression model with new (landscape, trajectory) pairs
             → prune stale library entries
             → recompute kernel bandwidth via cross-validation

dream_nrem:  Replay topological transitions
             → sequences of landscapes with outcomes
             → reinforce accurate predictions, decay inaccurate ones

dream_rem:   Deform landscapes into novel configurations
             → predict trajectories for never-seen topologies
             → expand library with synthetic entries
```

### Integration with Doc 2's manifold

The spectral liquidity manifold from Doc 2 provides a Riemannian metric on the DeFi state space. The predictive geometry engine uses this metric in two ways:

**Geodesic distance as transition time lower bound.** When the constraint engine determines that a topological transition requires merging two components, it queries the manifold for the geodesic distance between the components. The geodesic distance, divided by the historical rate of state-space traversal, gives a minimum time-to-transition. This is a hard lower bound: the system cannot traverse the state space faster than its dynamics allow, and the geodesic is the shortest possible path. If the geodesic distance between two liquidity clusters is large (they occupy distant regions of the manifold), merging will take many ticks even under optimistic assumptions.

**Curvature-aware trajectory weighting.** Trajectories that pass through high-curvature regions of the manifold are less likely than trajectories through flat regions, because high curvature means small perturbations cause large state changes -- the system is unstable there and tends to move through quickly. The trajectory regression weights neighbors by both landscape similarity and manifold curvature along the predicted path.

The connection between topology and geometry runs deeper than these two pragmatic uses. The manifold's curvature tensor determines the local topology via the Gauss-Bonnet theorem (for 2-manifolds) and its higher-dimensional generalizations. Regions of high positive curvature tend to be topologically simple (sphere-like). Regions of negative curvature tend to be topologically complex (hyperbolic, with potential for high Betti numbers). The predictive geometry engine can use curvature as a leading indicator of topological change: if the manifold's curvature is becoming more negative in a region, expect increased topological complexity (higher Betti numbers, more features in the persistence diagram) in upcoming observations.

### Integration with HDC encoding (Doc 1)

For fast similarity search in the landscape library, each discretized landscape matrix L ∈ R^{K × N} is encoded as a hyperdimensional binary vector. The encoding:

1. Quantize each element of L to one of Q levels (Q = 16 in practice).
2. Map each (layer_index, sample_index, quantized_value) triple to a random 10,240-bit seed vector.
3. XOR-bind the seed vectors for each row.
4. Majority-vote bundle across rows.

The resulting 10,240-bit hypervector captures the landscape's shape in a fixed-size representation. Hamming distance between hypervectors approximates L^2 distance between landscapes, enabling O(1) approximate nearest-neighbor lookups in the library. The full L^2 distance is computed only for the top-M candidates returned by the HDC filter.

## Rust implementation

### Core types

```rust
use std::collections::VecDeque;

/// Configuration for the predictive topology engine.
#[derive(Clone, Debug)]
pub struct PredictiveConfig {
    /// Number of landscape layers to compute (K).
    pub num_layers: usize,
    /// Number of sample points per layer (N).
    pub num_samples: usize,
    /// Filtration range: [t_min, t_max].
    pub t_min: f64,
    pub t_max: f64,
    /// Prediction horizon in Gamma ticks.
    pub horizon_ticks: u32,
    /// Number of nearest neighbors for trajectory lookup.
    pub k_neighbors: usize,
    /// Kernel bandwidth for trajectory weighting.
    pub kernel_bandwidth: f64,
    /// Maximum library size before pruning.
    pub max_library_size: usize,
    /// Minimum landscape distance to consider a match "close."
    pub close_match_threshold: f64,
    /// Dimensions to compute landscapes for (0 = components, 1 = loops, 2 = voids).
    pub homology_dimensions: Vec<u8>,
}

impl Default for PredictiveConfig {
    fn default() -> Self {
        Self {
            num_layers: 5,
            num_samples: 64,
            t_min: 0.0,
            t_max: 1.0,
            horizon_ticks: 30,
            k_neighbors: 10,
            kernel_bandwidth: 0.1,
            max_library_size: 10_000,
            close_match_threshold: 0.05,
            homology_dimensions: vec![0, 1, 2],
        }
    }
}
```

### Persistence landscape

```rust
/// A persistence landscape: a sequence of piecewise-linear functions
/// discretized on a regular grid.
///
/// The landscape for homology dimension d summarizes the persistence
/// diagram restricted to d-dimensional features. layers[k][i] holds
/// the value of the k-th landscape function at sample point i.
#[derive(Clone, Debug)]
pub struct PersistenceLandscape {
    /// λ_k functions sampled at regular t-intervals.
    /// layers[k][i] = λ_k(t_i).
    pub layers: Vec<Vec<f64>>,
    pub t_min: f64,
    pub t_max: f64,
    pub num_samples: usize,
    pub num_layers: usize,
    /// The homology dimension this landscape summarizes.
    pub dimension: u8,
}

impl PersistenceLandscape {
    /// Build a landscape from a persistence diagram, restricted to
    /// features of the given homology dimension.
    ///
    /// `diagram` contains (birth, death, dimension) triples.
    /// We filter to the target dimension, compute tent functions,
    /// then extract the top-K envelope functions.
    pub fn from_diagram(
        diagram: &PersistenceDiagram,
        dim: u8,
        num_layers: usize,
        num_samples: usize,
        t_min: f64,
        t_max: f64,
    ) -> Self {
        // Filter features to the target dimension.
        let features: Vec<(f64, f64)> = diagram
            .features
            .iter()
            .filter(|f| f.dimension == dim)
            .map(|f| (f.birth as f64, f.death as f64))
            .collect();

        let dt = (t_max - t_min) / (num_samples as f64 - 1.0);
        let mut layers = vec![vec![0.0; num_samples]; num_layers];

        for i in 0..num_samples {
            let t = t_min + (i as f64) * dt;

            // Evaluate all tent functions at this t.
            let mut values: Vec<f64> = features
                .iter()
                .map(|&(b, d)| {
                    let v = f64::min(t - b, d - t);
                    f64::max(0.0, v)
                })
                .collect();

            // Sort descending to extract the k-th largest values.
            values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

            for k in 0..num_layers {
                layers[k][i] = if k < values.len() { values[k] } else { 0.0 };
            }
        }

        Self {
            layers,
            t_min,
            t_max,
            num_samples,
            num_layers,
            dimension: dim,
        }
    }

    /// L^p distance between two landscapes.
    ///
    /// Both landscapes must share the same grid parameters.
    /// For p = 2, this is the standard Hilbert space distance.
    pub fn distance(&self, other: &Self, p: f64) -> f64 {
        debug_assert_eq!(self.num_samples, other.num_samples);
        debug_assert_eq!(self.num_layers, other.num_layers);

        let dt = (self.t_max - self.t_min) / (self.num_samples as f64 - 1.0);
        let mut total = 0.0;

        for k in 0..self.num_layers {
            for i in 0..self.num_samples {
                let diff = (self.layers[k][i] - other.layers[k][i]).abs();
                total += diff.powf(p) * dt;
            }
        }

        total.powf(1.0 / p)
    }

    /// Pointwise mean of multiple landscapes.
    ///
    /// Returns a landscape where each sample point is the
    /// arithmetic mean of the corresponding points across inputs.
    pub fn mean(landscapes: &[&Self]) -> Option<Self> {
        if landscapes.is_empty() {
            return None;
        }

        let first = landscapes[0];
        let n = landscapes.len() as f64;
        let mut layers = vec![vec![0.0; first.num_samples]; first.num_layers];

        for ls in landscapes {
            for k in 0..first.num_layers {
                for i in 0..first.num_samples {
                    layers[k][i] += ls.layers[k][i] / n;
                }
            }
        }

        Some(Self {
            layers,
            t_min: first.t_min,
            t_max: first.t_max,
            num_samples: first.num_samples,
            num_layers: first.num_layers,
            dimension: first.dimension,
        })
    }

    /// Finite-difference derivative: (self - other) / dt.
    ///
    /// Measures how the landscape changed between two time steps.
    /// Large derivatives signal topological transitions in progress.
    pub fn derivative(&self, previous: &Self, time_delta: f64) -> Self {
        debug_assert_eq!(self.num_samples, previous.num_samples);
        debug_assert_eq!(self.num_layers, previous.num_layers);

        let mut layers = vec![vec![0.0; self.num_samples]; self.num_layers];

        for k in 0..self.num_layers {
            for i in 0..self.num_samples {
                layers[k][i] = (self.layers[k][i] - previous.layers[k][i]) / time_delta;
            }
        }

        Self {
            layers,
            t_min: self.t_min,
            t_max: self.t_max,
            num_samples: self.num_samples,
            num_layers: self.num_layers,
            dimension: self.dimension,
        }
    }

    /// L^p norm of the landscape.
    pub fn norm(&self, p: f64) -> f64 {
        let dt = (self.t_max - self.t_min) / (self.num_samples as f64 - 1.0);
        let mut total = 0.0;

        for k in 0..self.num_layers {
            for i in 0..self.num_samples {
                total += self.layers[k][i].abs().powf(p) * dt;
            }
        }

        total.powf(1.0 / p)
    }

    /// Flatten the landscape into a single feature vector for
    /// regression and similarity search.
    ///
    /// Layout: [layer_0_sample_0, layer_0_sample_1, ..., layer_K_sample_N]
    pub fn to_feature_vector(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.num_layers * self.num_samples);
        for k in 0..self.num_layers {
            v.extend_from_slice(&self.layers[k]);
        }
        v
    }

    /// L^2 inner product between two landscapes.
    pub fn inner_product(&self, other: &Self) -> f64 {
        debug_assert_eq!(self.num_samples, other.num_samples);
        debug_assert_eq!(self.num_layers, other.num_layers);

        let dt = (self.t_max - self.t_min) / (self.num_samples as f64 - 1.0);
        let mut total = 0.0;

        for k in 0..self.num_layers {
            for i in 0..self.num_samples {
                total += self.layers[k][i] * other.layers[k][i] * dt;
            }
        }

        total
    }
}
```

### Multi-dimensional landscape bundle

A single persistence diagram produces multiple landscapes, one per homology dimension. The bundle groups them:

```rust
/// A bundle of persistence landscapes across multiple homology dimensions.
///
/// For a typical DeFi observation window, we compute landscapes for
/// dimensions 0 (components), 1 (loops), and 2 (voids). The bundle
/// is the full topological summary that feeds trajectory prediction.
#[derive(Clone, Debug)]
pub struct LandscapeBundle {
    /// Landscapes keyed by homology dimension.
    pub landscapes: Vec<PersistenceLandscape>,
    /// Tick at which this bundle was computed.
    pub tick: u64,
    /// Betti numbers extracted from the underlying diagram.
    pub betti: Vec<u16>,
}

impl LandscapeBundle {
    /// Build a bundle from a persistence diagram.
    pub fn from_diagram(
        diagram: &PersistenceDiagram,
        dimensions: &[u8],
        num_layers: usize,
        num_samples: usize,
        t_min: f64,
        t_max: f64,
    ) -> Self {
        let landscapes: Vec<PersistenceLandscape> = dimensions
            .iter()
            .map(|&dim| {
                PersistenceLandscape::from_diagram(
                    diagram, dim, num_layers, num_samples, t_min, t_max,
                )
            })
            .collect();

        // Extract Betti numbers: count features alive at the midpoint
        // of the filtration range.
        let mid = (t_min + t_max) / 2.0;
        let betti: Vec<u16> = dimensions
            .iter()
            .map(|&dim| {
                diagram
                    .features
                    .iter()
                    .filter(|f| {
                        f.dimension == dim
                            && (f.birth as f64) <= mid
                            && (f.death as f64) > mid
                    })
                    .count() as u16
            })
            .collect();

        Self {
            landscapes,
            tick: 0, // Set by caller.
            betti,
        }
    }

    /// Combined L^2 distance across all dimensions.
    ///
    /// Weights each dimension equally. A weighted variant could
    /// prioritize β_0 changes over β_1 changes, but uniform
    /// weighting works well in practice.
    pub fn distance(&self, other: &Self) -> f64 {
        let sum_sq: f64 = self
            .landscapes
            .iter()
            .zip(other.landscapes.iter())
            .map(|(a, b)| {
                let d = a.distance(b, 2.0);
                d * d
            })
            .sum();

        sum_sq.sqrt()
    }

    /// Flatten all landscapes into a single feature vector.
    pub fn to_feature_vector(&self) -> Vec<f64> {
        let mut v = Vec::new();
        for ls in &self.landscapes {
            v.extend(ls.to_feature_vector());
        }
        v
    }
}
```

### Trajectory types

```rust
/// A topological constraint on predicted trajectories.
///
/// Derived from the current persistence landscape and Betti numbers.
/// The constraint engine uses these to filter impossible paths.
#[derive(Clone, Debug)]
pub enum TopologicalConstraint {
    /// β_0 = 1: single connected component.
    /// Smooth transitions are the norm. Discontinuous jumps are unlikely
    /// without an external shock.
    SingleComponent,

    /// β_0 > 1: observation space has multiple disconnected clusters.
    /// The u16 field records the component count.
    /// Smooth convergence is impossible; merging events must occur first.
    Fragmented(u16),

    /// β_1 > 0: loops present in the observation space.
    /// The u16 field records the number of persistent loops.
    /// Oscillatory trajectories are topologically favored.
    Oscillatory(u16),

    /// Betti numbers are decreasing across dimensions.
    /// Topology is simplifying: features are dying, the space
    /// is consolidating. Predict convergent behavior.
    Collapsing,

    /// Betti numbers are increasing across dimensions.
    /// Topology is gaining features: new components, new loops.
    /// Predict divergent or complexifying behavior.
    Complexifying,
}

/// A single predicted trajectory: a path through observation space
/// with an associated probability and topological constraint.
#[derive(Clone, Debug)]
pub struct PredictedTrajectory {
    /// Sequence of observation vectors over the prediction horizon.
    /// path[t][d] = value of dimension d at horizon tick t.
    pub path: Vec<Vec<f64>>,
    /// Probability weight assigned to this trajectory.
    /// Derived from kernel regression weights and constraint filtering.
    pub probability: f64,
    /// The topological constraint that this trajectory satisfies.
    pub constraint: TopologicalConstraint,
    /// L^2 distance from the query landscape to the library entry
    /// that produced this trajectory. Lower = more confident match.
    pub landscape_distance: f64,
}

/// A family of predicted trajectories spanning the forecast horizon.
///
/// The family represents the space of topologically possible futures.
/// Individual trajectories carry probabilities; the family carries
/// aggregate statistics about confidence and constraint type.
#[derive(Clone, Debug)]
pub struct TrajectoryFamily {
    /// The predicted trajectories, sorted by probability (descending).
    pub trajectories: Vec<PredictedTrajectory>,
    /// Prediction horizon in Gamma ticks.
    pub horizon_ticks: u32,
    /// Confidence: 1.0 minus the mean landscape distance of neighbors.
    /// High confidence means the library had close matches.
    /// Low confidence means we are in uncharted topological territory.
    pub confidence: f64,
    /// The dominant topological constraint for this family.
    pub dominant_constraint: TopologicalConstraint,
    /// Betti numbers at the time of prediction.
    pub current_betti: Vec<u16>,
}

/// A topological forecast produced at Theta frequency.
///
/// Aggregates Gamma-rate trajectory predictions into a single
/// insight for the Oracle.
#[derive(Clone, Debug)]
pub struct TopologicalForecast {
    /// The trajectory family (already filtered by constraints).
    pub family: TrajectoryFamily,
    /// Expected topological transitions in the horizon.
    /// Each entry: (estimated tick, transition description).
    pub expected_transitions: Vec<(u32, String)>,
    /// Landscape derivative norm: rate of topological change.
    /// High values signal imminent regime transitions.
    pub landscape_velocity: f64,
    /// DeFi primitive type this forecast concerns.
    pub primitive_type: DeFiPrimitive,
}

/// Enumeration of DeFi primitive types for targeted interpretation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeFiPrimitive {
    ConcentratedLiquidity,
    LendingMarket,
    PerpetualDerivative,
    OptionsMarket,
    YieldToken,
    Vault,
    Staking,
}

/// A hypothetical topology generated during REM dreaming.
///
/// REM dreams deform existing landscapes into novel configurations
/// that the Golem has never observed, then predict what trajectories
/// those configurations would produce.
#[derive(Clone, Debug)]
pub struct HypotheticalTopology {
    /// The deformed landscape bundle.
    pub landscape: LandscapeBundle,
    /// The predicted trajectory family for this hypothetical topology.
    pub predicted_family: TrajectoryFamily,
    /// Description of the deformation applied.
    pub deformation: String,
    /// How different this hypothetical is from any historical landscape.
    pub novelty_score: f64,
}
```

### Landscape library

```rust
/// An indexed collection of historical landscape bundles paired
/// with the trajectories that followed.
///
/// The library supports two retrieval modes:
/// - HDC-accelerated approximate nearest neighbor (for Gamma-rate queries)
/// - Exact L^2 nearest neighbor (for Theta-rate refinement)
pub struct LandscapeLibrary {
    /// Historical entries: (tick, landscape_bundle, observed_trajectory).
    entries: Vec<LibraryEntry>,
    /// HDC index: hypervectors for approximate nearest neighbor.
    hdc_index: Vec<(usize, Vec<u64>)>,
    /// Configuration.
    config: PredictiveConfig,
    /// Running statistics for adaptive kernel bandwidth.
    distance_stats: DistanceStats,
}

struct LibraryEntry {
    tick: u64,
    bundle: LandscapeBundle,
    trajectory: Vec<Vec<f64>>,
    /// Fitness score: how well did predictions based on this
    /// entry perform? Entries with low fitness get pruned.
    fitness: f64,
    /// Number of times this entry was used as a neighbor.
    retrieval_count: u32,
}

/// Running mean and variance of nearest-neighbor distances,
/// used to adaptively set the kernel bandwidth.
struct DistanceStats {
    count: u64,
    mean: f64,
    m2: f64, // Welford's online variance accumulator.
}

impl DistanceStats {
    fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    fn update(&mut self, distance: f64) {
        self.count += 1;
        let delta = distance - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = distance - self.mean;
        self.m2 += delta * delta2;
    }

    fn variance(&self) -> f64 {
        if self.count < 2 {
            return 1.0;
        }
        self.m2 / (self.count as f64 - 1.0)
    }

    /// Adaptive kernel bandwidth: Silverman's rule of thumb
    /// applied to the distribution of nearest-neighbor distances.
    fn adaptive_bandwidth(&self) -> f64 {
        let sigma = self.variance().sqrt();
        let n = self.count as f64;
        if n < 2.0 {
            return 1.0;
        }
        // Silverman's rule: h = 1.06 * σ * n^(-1/5)
        1.06 * sigma * n.powf(-0.2)
    }
}

impl LandscapeLibrary {
    pub fn new(config: PredictiveConfig) -> Self {
        Self {
            entries: Vec::new(),
            hdc_index: Vec::new(),
            config,
            distance_stats: DistanceStats::new(),
        }
    }

    /// Add a new (landscape, trajectory) pair to the library.
    ///
    /// Called during Delta ticks when the trajectory following a
    /// landscape has been fully observed.
    pub fn insert(&mut self, bundle: LandscapeBundle, trajectory: Vec<Vec<f64>>) {
        let hdc = self.encode_hdc(&bundle);
        let idx = self.entries.len();

        self.entries.push(LibraryEntry {
            tick: bundle.tick,
            bundle,
            trajectory,
            fitness: 1.0, // Neutral initial fitness.
            retrieval_count: 0,
        });

        self.hdc_index.push((idx, hdc));

        // Prune if over capacity.
        if self.entries.len() > self.config.max_library_size {
            self.prune();
        }
    }

    /// Find the K nearest neighbors by L^2 distance.
    ///
    /// Two-stage retrieval:
    /// 1. HDC filter: compute Hamming distances to all entries, keep top 4K candidates.
    /// 2. Exact L^2: compute full landscape distance on candidates, keep top K.
    pub fn nearest_neighbors(
        &mut self,
        query: &LandscapeBundle,
        k: usize,
    ) -> Vec<(usize, f64)> {
        let query_hdc = self.encode_hdc(query);
        let candidate_count = (4 * k).min(self.entries.len());

        // Stage 1: HDC approximate filter.
        let mut hdc_distances: Vec<(usize, u32)> = self
            .hdc_index
            .iter()
            .map(|(idx, hdc)| (*idx, hamming_distance(&query_hdc, hdc)))
            .collect();

        hdc_distances.sort_by_key(|&(_, d)| d);
        let candidates: Vec<usize> = hdc_distances
            .iter()
            .take(candidate_count)
            .map(|&(idx, _)| idx)
            .collect();

        // Stage 2: exact L^2 distance on candidates.
        let mut exact: Vec<(usize, f64)> = candidates
            .iter()
            .map(|&idx| {
                let dist = query.distance(&self.entries[idx].bundle);
                (idx, dist)
            })
            .collect();

        exact.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        exact.truncate(k);

        // Update distance statistics for adaptive bandwidth.
        for &(_, dist) in &exact {
            self.distance_stats.update(dist);
        }

        // Update retrieval counts.
        for &(idx, _) in &exact {
            self.entries[idx].retrieval_count += 1;
        }

        exact
    }

    /// Encode a landscape bundle as a 10,240-bit HDC hypervector.
    ///
    /// The encoding quantizes landscape values, maps (layer, sample, level)
    /// triples to seed vectors via deterministic hashing, and bundles
    /// them with majority vote.
    fn encode_hdc(&self, bundle: &LandscapeBundle) -> Vec<u64> {
        let dim_words = 10_240 / 64; // 160 u64 words.
        let mut accumulator = vec![0i32; 10_240];

        for ls in &bundle.landscapes {
            for k in 0..ls.num_layers {
                for i in 0..ls.num_samples {
                    // Quantize to 16 levels.
                    let level = quantize(ls.layers[k][i], 16);

                    // Deterministic seed from (dimension, layer, sample, level).
                    let seed = hash_triple(ls.dimension, k as u32, i as u32, level);
                    let seed_vec = generate_seed_vector(seed, 10_240);

                    // XOR-bind within a layer row, then accumulate.
                    for bit in 0..10_240 {
                        if seed_vec[bit] {
                            accumulator[bit] += 1;
                        } else {
                            accumulator[bit] -= 1;
                        }
                    }
                }
            }
        }

        // Majority vote: positive -> 1, non-positive -> 0.
        let mut result = vec![0u64; dim_words];
        for bit in 0..10_240 {
            if accumulator[bit] > 0 {
                result[bit / 64] |= 1u64 << (bit % 64);
            }
        }

        result
    }

    /// Prune the library to max_library_size.
    ///
    /// Eviction priority (lowest fitness first):
    /// - Low fitness score (predictions based on this entry were wrong).
    /// - Low retrieval count (this entry was rarely the nearest neighbor).
    /// - Old tick (older entries are less relevant to current market conditions).
    fn prune(&mut self) {
        let target = self.config.max_library_size * 3 / 4;

        // Score each entry for eviction.
        let max_tick = self.entries.iter().map(|e| e.tick).max().unwrap_or(0);
        let mut scores: Vec<(usize, f64)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let recency = if max_tick > 0 {
                    entry.tick as f64 / max_tick as f64
                } else {
                    1.0
                };
                let retrieval = (entry.retrieval_count as f64).ln_1p();
                let score = entry.fitness * 0.5 + recency * 0.3 + retrieval * 0.2;
                (idx, score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let keep: std::collections::HashSet<usize> =
            scores.iter().take(target).map(|&(idx, _)| idx).collect();

        // Rebuild entries and HDC index.
        let mut new_entries = Vec::with_capacity(target);
        let mut new_hdc = Vec::with_capacity(target);
        let mut remap = std::collections::HashMap::new();

        for (old_idx, entry) in self.entries.drain(..).enumerate() {
            if keep.contains(&old_idx) {
                let new_idx = new_entries.len();
                remap.insert(old_idx, new_idx);
                new_entries.push(entry);
            }
        }

        for (old_idx, hdc) in self.hdc_index.drain(..) {
            if let Some(&new_idx) = remap.get(&old_idx) {
                new_hdc.push((new_idx, hdc));
            }
        }

        self.entries = new_entries;
        self.hdc_index = new_hdc;
    }

    /// Update fitness scores based on prediction outcomes.
    ///
    /// For each entry that was used as a neighbor, compare the
    /// predicted trajectory to the actual trajectory and adjust fitness.
    pub fn update_fitness(
        &mut self,
        neighbor_indices: &[usize],
        actual_trajectory: &[Vec<f64>],
    ) {
        for &idx in neighbor_indices {
            if idx >= self.entries.len() {
                continue;
            }
            let entry = &mut self.entries[idx];
            let predicted = &entry.trajectory;

            // Trajectory similarity: mean L^2 distance across time steps.
            let steps = predicted.len().min(actual_trajectory.len());
            if steps == 0 {
                continue;
            }

            let mut total_dist = 0.0;
            for t in 0..steps {
                let dims = predicted[t].len().min(actual_trajectory[t].len());
                let step_dist: f64 = (0..dims)
                    .map(|d| {
                        let diff = predicted[t][d] - actual_trajectory[t][d];
                        diff * diff
                    })
                    .sum::<f64>()
                    .sqrt();
                total_dist += step_dist;
            }
            let mean_dist = total_dist / steps as f64;

            // Fitness update: exponential moving average.
            // Accuracy = 1 / (1 + mean_dist) maps distance to [0, 1].
            let accuracy = 1.0 / (1.0 + mean_dist);
            entry.fitness = 0.9 * entry.fitness + 0.1 * accuracy;
        }
    }
}

/// Hamming distance between two HDC vectors (packed as u64 words).
fn hamming_distance(a: &[u64], b: &[u64]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

/// Quantize a floating-point value to one of `levels` discrete levels.
/// Assumes input is non-negative (landscape values are always >= 0).
fn quantize(value: f64, levels: u32) -> u32 {
    // Clamp to [0, max_expected] then map to [0, levels-1].
    // The max_expected is estimated from the data; 1.0 works for
    // normalized landscapes.
    let clamped = value.clamp(0.0, 1.0);
    let level = (clamped * (levels - 1) as f64).round() as u32;
    level.min(levels - 1)
}

/// Deterministic hash for (dimension, layer, sample, level) -> u64 seed.
fn hash_triple(dim: u8, layer: u32, sample: u32, level: u32) -> u64 {
    // FNV-1a hash for simplicity. Not cryptographic, but sufficient
    // for generating pseudo-random seed vectors.
    let mut hash: u64 = 0xcbf29ce484222325;
    let prime: u64 = 0x100000001b3;

    for byte in &[dim] {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(prime);
    }
    for byte in &layer.to_le_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(prime);
    }
    for byte in &sample.to_le_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(prime);
    }
    for byte in &level.to_le_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(prime);
    }

    hash
}

/// Generate a pseudo-random bit vector from a seed.
/// Uses a simple xorshift64 PRNG seeded from the input.
fn generate_seed_vector(seed: u64, bits: usize) -> Vec<bool> {
    let mut state = seed;
    let mut vec = Vec::with_capacity(bits);

    for _ in 0..bits {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        vec.push(state & 1 == 1);
    }

    vec
}
```

### Topological constraint engine

```rust
/// Determines topological constraints from a landscape bundle
/// and its recent history, then filters trajectory families
/// to remove topologically impossible paths.
pub struct TopologicalConstraintEngine {
    /// Recent Betti number history for trend detection.
    betti_history: VecDeque<(u64, Vec<u16>)>,
    /// Maximum history length.
    max_history: usize,
}

impl TopologicalConstraintEngine {
    pub fn new(max_history: usize) -> Self {
        Self {
            betti_history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Record the current Betti numbers.
    pub fn record(&mut self, tick: u64, betti: Vec<u16>) {
        self.betti_history.push_back((tick, betti));
        if self.betti_history.len() > self.max_history {
            self.betti_history.pop_front();
        }
    }

    /// Derive the current topological constraint from Betti numbers
    /// and their recent trend.
    pub fn current_constraint(&self) -> TopologicalConstraint {
        let current = match self.betti_history.back() {
            Some((_, betti)) => betti.clone(),
            None => return TopologicalConstraint::SingleComponent,
        };

        let beta_0 = current.first().copied().unwrap_or(1);
        let beta_1 = current.get(1).copied().unwrap_or(0);

        // Check for fragmentation.
        if beta_0 > 1 {
            return TopologicalConstraint::Fragmented(beta_0);
        }

        // Check for oscillatory constraints.
        if beta_1 > 0 {
            return TopologicalConstraint::Oscillatory(beta_1);
        }

        // Check trends over history.
        if self.betti_history.len() >= 3 {
            let trend = self.betti_trend();
            if trend < -0.5 {
                return TopologicalConstraint::Collapsing;
            }
            if trend > 0.5 {
                return TopologicalConstraint::Complexifying;
            }
        }

        TopologicalConstraint::SingleComponent
    }

    /// Compute a scalar trend over recent Betti number history.
    ///
    /// Positive = topology gaining features (complexifying).
    /// Negative = topology losing features (collapsing).
    /// Near zero = stable.
    fn betti_trend(&self) -> f64 {
        if self.betti_history.len() < 2 {
            return 0.0;
        }

        let n = self.betti_history.len();
        let half = n / 2;

        // Compare mean total Betti in the recent half vs. older half.
        let older_mean: f64 = self.betti_history
            .iter()
            .take(half)
            .map(|(_, b)| b.iter().map(|&x| x as f64).sum::<f64>())
            .sum::<f64>()
            / half as f64;

        let recent_mean: f64 = self.betti_history
            .iter()
            .skip(n - half)
            .map(|(_, b)| b.iter().map(|&x| x as f64).sum::<f64>())
            .sum::<f64>()
            / half as f64;

        recent_mean - older_mean
    }

    /// Filter a set of candidate trajectories against topological constraints.
    ///
    /// Trajectories incompatible with the current constraint are removed.
    /// Trajectories partially compatible have their probability downweighted.
    pub fn filter_trajectories(
        &self,
        candidates: Vec<PredictedTrajectory>,
        constraint: &TopologicalConstraint,
    ) -> Vec<PredictedTrajectory> {
        candidates
            .into_iter()
            .filter_map(|mut traj| {
                let weight = self.constraint_compatibility(&traj, constraint);
                if weight < 0.01 {
                    return None; // Incompatible: remove.
                }
                traj.probability *= weight;
                traj.constraint = constraint.clone();
                Some(traj)
            })
            .collect()
    }

    /// Score how compatible a trajectory is with a topological constraint.
    ///
    /// Returns a weight in [0, 1]. Zero means incompatible.
    fn constraint_compatibility(
        &self,
        trajectory: &PredictedTrajectory,
        constraint: &TopologicalConstraint,
    ) -> f64 {
        match constraint {
            TopologicalConstraint::SingleComponent => {
                // Penalize trajectories with large jumps (discontinuities).
                let max_jump = self.max_step_size(trajectory);
                if max_jump > 2.0 {
                    0.1 // Heavy penalty but not elimination.
                } else {
                    1.0
                }
            }
            TopologicalConstraint::Fragmented(components) => {
                // Penalize trajectories that assume smooth convergence.
                // The more components, the harder convergence is.
                let smoothness = self.trajectory_smoothness(trajectory);
                if smoothness > 0.8 && *components > 2 {
                    0.2 // Too smooth for a fragmented space.
                } else {
                    1.0
                }
            }
            TopologicalConstraint::Oscillatory(loops) => {
                // Favor trajectories that show oscillatory behavior.
                let osc = self.oscillation_score(trajectory);
                // More loops = stronger oscillation expected.
                let expected_osc = (*loops as f64 * 0.2).min(1.0);
                if osc < expected_osc * 0.5 {
                    0.3 // Insufficient oscillation for topology.
                } else {
                    1.0
                }
            }
            TopologicalConstraint::Collapsing => {
                // Favor trajectories that converge.
                let convergence = self.convergence_score(trajectory);
                if convergence < 0.3 {
                    0.4 // Not converging in a collapsing topology.
                } else {
                    1.0
                }
            }
            TopologicalConstraint::Complexifying => {
                // Favor trajectories that diverge or show increasing variance.
                let divergence = self.divergence_score(trajectory);
                if divergence < 0.3 {
                    0.4 // Not diverging in a complexifying topology.
                } else {
                    1.0
                }
            }
        }
    }

    /// Maximum Euclidean step size between consecutive points in a trajectory.
    fn max_step_size(&self, trajectory: &PredictedTrajectory) -> f64 {
        let path = &trajectory.path;
        let mut max_step = 0.0f64;

        for t in 1..path.len() {
            let step: f64 = path[t]
                .iter()
                .zip(path[t - 1].iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            max_step = max_step.max(step);
        }

        max_step
    }

    /// Smoothness: fraction of steps below the median step size.
    fn trajectory_smoothness(&self, trajectory: &PredictedTrajectory) -> f64 {
        let path = &trajectory.path;
        if path.len() < 2 {
            return 1.0;
        }

        let mut steps: Vec<f64> = (1..path.len())
            .map(|t| {
                path[t]
                    .iter()
                    .zip(path[t - 1].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .collect();

        steps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = steps[steps.len() / 2];
        let below = steps.iter().filter(|&&s| s <= median * 1.5).count();

        below as f64 / steps.len() as f64
    }

    /// Oscillation score: measures how much a trajectory reverses direction.
    fn oscillation_score(&self, trajectory: &PredictedTrajectory) -> f64 {
        let path = &trajectory.path;
        if path.len() < 3 {
            return 0.0;
        }

        // Count direction reversals in each dimension, average across dimensions.
        let dims = path[0].len();
        if dims == 0 {
            return 0.0;
        }

        let mut total_reversals = 0usize;
        let max_reversals = (path.len() - 2) * dims;

        for d in 0..dims {
            for t in 2..path.len() {
                let prev_delta = path[t - 1][d] - path[t - 2][d];
                let curr_delta = path[t][d] - path[t - 1][d];
                if prev_delta * curr_delta < 0.0 {
                    total_reversals += 1;
                }
            }
        }

        if max_reversals == 0 {
            return 0.0;
        }

        total_reversals as f64 / max_reversals as f64
    }

    /// Convergence score: do the trajectory dimensions narrow over time?
    fn convergence_score(&self, trajectory: &PredictedTrajectory) -> f64 {
        let path = &trajectory.path;
        if path.len() < 4 {
            return 0.5;
        }

        let half = path.len() / 2;
        let dims = path[0].len();
        if dims == 0 {
            return 0.5;
        }

        // Compare variance in the first half vs. second half.
        let first_var = self.window_variance(&path[..half]);
        let second_var = self.window_variance(&path[half..]);

        if first_var < 1e-10 {
            return 0.5;
        }

        // Ratio < 1 means converging, > 1 means diverging.
        let ratio = second_var / first_var;
        (1.0 - ratio).clamp(0.0, 1.0)
    }

    /// Divergence score: inverse of convergence.
    fn divergence_score(&self, trajectory: &PredictedTrajectory) -> f64 {
        1.0 - self.convergence_score(trajectory)
    }

    /// Variance of observation vectors in a window.
    fn window_variance(&self, window: &[Vec<f64>]) -> f64 {
        if window.len() < 2 {
            return 0.0;
        }

        let dims = window[0].len();
        let n = window.len() as f64;

        let mean: Vec<f64> = (0..dims)
            .map(|d| window.iter().map(|v| v[d]).sum::<f64>() / n)
            .collect();

        let var: f64 = window
            .iter()
            .map(|v| {
                v.iter()
                    .zip(mean.iter())
                    .map(|(x, m)| (x - m).powi(2))
                    .sum::<f64>()
            })
            .sum::<f64>()
            / n;

        var
    }
}
```

### Topology-trajectory regression

```rust
/// Kernel regression from persistence landscapes to trajectory families.
///
/// Given a query landscape, finds nearest neighbors in the library,
/// weights them by a Gaussian kernel, and constructs a trajectory family
/// from the weighted combination.
pub struct TopologyTrajectoryRegression {
    library: LandscapeLibrary,
    constraint_engine: TopologicalConstraintEngine,
    config: PredictiveConfig,
}

impl TopologyTrajectoryRegression {
    pub fn new(config: PredictiveConfig) -> Self {
        Self {
            library: LandscapeLibrary::new(config.clone()),
            constraint_engine: TopologicalConstraintEngine::new(100),
            config,
        }
    }

    /// Predict a trajectory family from a landscape bundle.
    ///
    /// Steps:
    /// 1. Find K nearest neighbors in the library.
    /// 2. Compute Gaussian kernel weights.
    /// 3. Build candidate trajectories from neighbors.
    /// 4. Apply topological constraints.
    /// 5. Normalize probabilities and return.
    pub fn predict(&mut self, query: &LandscapeBundle) -> TrajectoryFamily {
        // Record Betti numbers for constraint tracking.
        self.constraint_engine
            .record(query.tick, query.betti.clone());

        let neighbors = self
            .library
            .nearest_neighbors(query, self.config.k_neighbors);

        if neighbors.is_empty() {
            return self.empty_family(query);
        }

        // Kernel bandwidth: use adaptive bandwidth from library statistics,
        // falling back to the configured value if insufficient data.
        let bandwidth = if self.library.distance_stats.count > 10 {
            self.library.distance_stats.adaptive_bandwidth()
        } else {
            self.config.kernel_bandwidth
        };

        // Compute kernel weights.
        let weights: Vec<f64> = neighbors
            .iter()
            .map(|&(_, dist)| gaussian_kernel(dist, bandwidth))
            .collect();

        let weight_sum: f64 = weights.iter().sum();
        if weight_sum < 1e-15 {
            return self.empty_family(query);
        }

        // Build candidate trajectories.
        let mut candidates: Vec<PredictedTrajectory> = neighbors
            .iter()
            .zip(weights.iter())
            .map(|(&(idx, dist), &w)| {
                let entry_traj = &self.library.entries[idx].trajectory;
                PredictedTrajectory {
                    path: entry_traj.clone(),
                    probability: w / weight_sum,
                    constraint: TopologicalConstraint::SingleComponent, // Overwritten by filter.
                    landscape_distance: dist,
                }
            })
            .collect();

        // Apply topological constraints.
        let constraint = self.constraint_engine.current_constraint();
        candidates = self
            .constraint_engine
            .filter_trajectories(candidates, &constraint);

        // Re-normalize probabilities after filtering.
        let total_prob: f64 = candidates.iter().map(|t| t.probability).sum();
        if total_prob > 1e-15 {
            for traj in &mut candidates {
                traj.probability /= total_prob;
            }
        }

        // Sort by probability descending.
        candidates.sort_by(|a, b| {
            b.probability
                .partial_cmp(&a.probability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Compute confidence from mean landscape distance.
        let mean_dist: f64 = neighbors.iter().map(|&(_, d)| d).sum::<f64>()
            / neighbors.len() as f64;
        let confidence = 1.0 / (1.0 + mean_dist / self.config.close_match_threshold);

        TrajectoryFamily {
            trajectories: candidates,
            horizon_ticks: self.config.horizon_ticks,
            confidence,
            dominant_constraint: constraint,
            current_betti: query.betti.clone(),
        }
    }

    /// Record an observed trajectory for a past landscape.
    /// Called during Delta tick when the horizon has elapsed.
    pub fn record_observation(
        &mut self,
        bundle: LandscapeBundle,
        trajectory: Vec<Vec<f64>>,
    ) {
        self.library.insert(bundle, trajectory);
    }

    /// Update fitness scores based on prediction accuracy.
    pub fn update_fitness(
        &mut self,
        neighbor_indices: &[usize],
        actual: &[Vec<f64>],
    ) {
        self.library.update_fitness(neighbor_indices, actual);
    }

    /// Cross-validate kernel bandwidth on the library.
    ///
    /// Leave-one-out: for each entry, predict its trajectory using
    /// all other entries, measure error, find the bandwidth that
    /// minimizes mean error.
    pub fn cross_validate_bandwidth(&mut self) {
        if self.library.entries.len() < 20 {
            return; // Not enough data for meaningful CV.
        }

        let bandwidths = [0.01, 0.05, 0.1, 0.2, 0.5, 1.0];
        let mut best_bw = self.config.kernel_bandwidth;
        let mut best_error = f64::MAX;

        for &bw in &bandwidths {
            let error = self.loo_error(bw);
            if error < best_error {
                best_error = error;
                best_bw = bw;
            }
        }

        self.config.kernel_bandwidth = best_bw;
    }

    /// Leave-one-out prediction error for a given bandwidth.
    fn loo_error(&self, bandwidth: f64) -> f64 {
        let n = self.library.entries.len();
        let sample_size = n.min(50); // Subsample for speed.
        let step = n / sample_size;
        let mut total_error = 0.0;
        let mut count = 0;

        for i in (0..n).step_by(step.max(1)) {
            let query = &self.library.entries[i].bundle;
            let actual = &self.library.entries[i].trajectory;

            // Find neighbors excluding self.
            let mut weighted_traj = vec![vec![0.0; actual[0].len()]; actual.len()];
            let mut weight_sum = 0.0;

            for (j, entry) in self.library.entries.iter().enumerate() {
                if j == i {
                    continue;
                }
                let dist = query.distance(&entry.bundle);
                let w = gaussian_kernel(dist, bandwidth);
                weight_sum += w;

                let steps = entry.trajectory.len().min(actual.len());
                for t in 0..steps {
                    let dims = entry.trajectory[t].len().min(actual[0].len());
                    for d in 0..dims {
                        weighted_traj[t][d] += w * entry.trajectory[t][d];
                    }
                }
            }

            if weight_sum < 1e-15 {
                continue;
            }

            // Normalize and compute error.
            let mut error = 0.0;
            for t in 0..actual.len() {
                for d in 0..actual[t].len() {
                    let predicted = weighted_traj[t][d] / weight_sum;
                    error += (predicted - actual[t][d]).powi(2);
                }
            }

            total_error += error;
            count += 1;
        }

        if count == 0 {
            return f64::MAX;
        }

        total_error / count as f64
    }

    fn empty_family(&self, query: &LandscapeBundle) -> TrajectoryFamily {
        TrajectoryFamily {
            trajectories: Vec::new(),
            horizon_ticks: self.config.horizon_ticks,
            confidence: 0.0,
            dominant_constraint: TopologicalConstraint::SingleComponent,
            current_betti: query.betti.clone(),
        }
    }
}

/// Gaussian kernel: K_h(d) = exp(-d^2 / (2h^2)).
fn gaussian_kernel(distance: f64, bandwidth: f64) -> f64 {
    (-distance.powi(2) / (2.0 * bandwidth.powi(2))).exp()
}
```

### The main engine

```rust
/// The predictive topology engine.
///
/// Wraps TdaAnalyzer with persistence landscape computation,
/// trajectory prediction, and topological constraint filtering.
/// Implements the TaAnalyzer trait for integration with the heartbeat.
pub struct PredictiveTopology {
    /// The underlying TDA engine for persistence diagram computation.
    tda: TdaAnalyzer,
    /// Persistence landscape history for derivative computation.
    landscape_history: VecDeque<LandscapeBundle>,
    /// Observation history for recording trajectories.
    observation_history: VecDeque<(u64, Vec<Vec<f64>>)>,
    /// The trajectory regression model.
    regression: TopologyTrajectoryRegression,
    /// Configuration.
    config: PredictiveConfig,
    /// Most recent trajectory family (for CorticalState writing).
    latest_family: Option<TrajectoryFamily>,
    /// Pending landscape-trajectory pairs waiting for trajectory completion.
    /// (start_tick, landscape_bundle, partial_trajectory)
    pending: VecDeque<(u64, LandscapeBundle, Vec<Vec<f64>>)>,
    /// Neighbor indices from the most recent prediction (for fitness updates).
    last_neighbor_indices: Vec<usize>,
}

impl PredictiveTopology {
    pub fn new(tda: TdaAnalyzer, config: PredictiveConfig) -> Self {
        let regression = TopologyTrajectoryRegression::new(config.clone());
        Self {
            tda,
            landscape_history: VecDeque::with_capacity(100),
            observation_history: VecDeque::with_capacity(
                config.horizon_ticks as usize * 2,
            ),
            regression,
            config,
            latest_family: None,
            pending: VecDeque::new(),
            last_neighbor_indices: Vec::new(),
        }
    }

    /// Gamma tick: perception-rate update.
    ///
    /// 1. Feed observations to TDA and get a persistence diagram.
    /// 2. Transform diagram into a landscape bundle.
    /// 3. Predict trajectory family.
    /// 4. Write confidence to CorticalState.
    pub fn gamma_tick(
        &mut self,
        observations: &[Vec<f32>],
        tick: u64,
    ) -> Option<TrajectoryFamily> {
        // Convert f32 observations to f64 for internal computation.
        let obs_f64: Vec<Vec<f64>> = observations
            .iter()
            .map(|v| v.iter().map(|&x| x as f64).collect())
            .collect();

        // Record observations for trajectory tracking.
        self.observation_history.push_back((tick, obs_f64.clone()));
        if self.observation_history.len() > self.config.horizon_ticks as usize * 2 {
            self.observation_history.pop_front();
        }

        // Compute persistence diagram via existing TDA pipeline.
        let diagram = self.tda.compute_diagram(observations)?;

        // Transform to landscape bundle.
        let mut bundle = LandscapeBundle::from_diagram(
            &diagram,
            &self.config.homology_dimensions,
            self.config.num_layers,
            self.config.num_samples,
            self.config.t_min,
            self.config.t_max,
        );
        bundle.tick = tick;

        // Store for derivative computation.
        self.landscape_history.push_back(bundle.clone());
        if self.landscape_history.len() > 100 {
            self.landscape_history.pop_front();
        }

        // Check if any pending entries have complete trajectories.
        self.complete_pending_entries(tick);

        // Start tracking a new pending entry.
        self.pending.push_back((tick, bundle.clone(), Vec::new()));
        if self.pending.len() > 50 {
            self.pending.pop_front();
        }

        // Predict trajectory family.
        let family = self.regression.predict(&bundle);
        self.latest_family = Some(family.clone());

        Some(family)
    }

    /// Theta tick: cognition-rate analysis.
    ///
    /// Aggregates recent Gamma-rate landscapes and produces
    /// TopologicalForecast values for the Oracle.
    pub fn theta_tick(&mut self) -> Vec<TopologicalForecast> {
        let family = match &self.latest_family {
            Some(f) => f.clone(),
            None => return Vec::new(),
        };

        // Compute landscape velocity from history.
        let velocity = self.landscape_velocity();

        // Detect expected transitions from Betti trends and velocity.
        let transitions = self.expected_transitions(&family, velocity);

        // Produce forecasts per DeFi primitive.
        // In practice, the engine would maintain separate observation
        // streams per primitive type. Here we produce a single forecast.
        vec![TopologicalForecast {
            family,
            expected_transitions: transitions,
            landscape_velocity: velocity,
            primitive_type: DeFiPrimitive::ConcentratedLiquidity,
        }]
    }

    /// Delta tick: consolidation-rate update.
    ///
    /// 1. Complete all pending trajectory recordings.
    /// 2. Cross-validate kernel bandwidth.
    /// 3. Update fitness scores.
    pub fn delta_tick(&mut self) {
        // Force-complete any old pending entries.
        self.force_complete_pending();

        // Cross-validate bandwidth every Delta tick.
        self.regression.cross_validate_bandwidth();
    }

    /// NREM dreaming: replay topological transitions.
    ///
    /// Receives a sequence of (tick, landscape, trajectory) triples
    /// representing historical topological transitions. Replays them
    /// through the regression model to reinforce accurate predictions
    /// and decay inaccurate ones.
    pub fn dream_nrem(
        &mut self,
        replay: &[(u64, LandscapeBundle, Vec<Vec<f64>>)],
    ) {
        for (_, bundle, trajectory) in replay {
            // Re-predict using current model.
            let predicted_family = self.regression.predict(bundle);

            // Compare predicted trajectories to actual.
            if !predicted_family.trajectories.is_empty() {
                let neighbor_indices: Vec<usize> = (0..predicted_family.trajectories.len())
                    .collect();
                self.regression
                    .update_fitness(&neighbor_indices, trajectory);
            }

            // Re-insert with updated context.
            self.regression
                .record_observation(bundle.clone(), trajectory.clone());
        }
    }

    /// REM dreaming: generate novel topological configurations.
    ///
    /// Deforms existing landscapes to create configurations the Golem
    /// has never seen, predicts trajectories for them, and adds
    /// the hypotheticals to the library as synthetic entries.
    pub fn dream_rem(
        &mut self,
        rng: &mut impl rand::Rng,
    ) -> Vec<HypotheticalTopology> {
        let mut hypotheticals = Vec::new();

        // Sample random historical landscapes to deform.
        let history_len = self.landscape_history.len();
        if history_len < 2 {
            return hypotheticals;
        }

        let num_dreams = 5.min(history_len);

        for _ in 0..num_dreams {
            let idx = rng.gen_range(0..history_len);
            let base = &self.landscape_history[idx];

            // Deformation 1: scale a random layer.
            let mut deformed = base.clone();
            if let Some(ls) = deformed.landscapes.first_mut() {
                let layer = rng.gen_range(0..ls.num_layers);
                let scale = rng.gen_range(0.5..2.0);
                for i in 0..ls.num_samples {
                    ls.layers[layer][i] *= scale;
                }
            }

            let description = "layer amplitude scaling".to_string();
            let novelty = base.distance(&deformed);
            let predicted_family = self.regression.predict(&deformed);

            hypotheticals.push(HypotheticalTopology {
                landscape: deformed.clone(),
                predicted_family,
                deformation: description,
                novelty_score: novelty,
            });

            // Deformation 2: shift the landscape in parameter space.
            let mut shifted = base.clone();
            let shift_amount = rng.gen_range(-0.1..0.1);
            for ls in &mut shifted.landscapes {
                for k in 0..ls.num_layers {
                    let n = ls.num_samples;
                    if shift_amount > 0.0 {
                        // Shift right: drop rightmost samples, pad left with zero.
                        let shift_bins = ((shift_amount * n as f64) as usize).min(n - 1);
                        ls.layers[k].rotate_right(shift_bins);
                        for i in 0..shift_bins {
                            ls.layers[k][i] = 0.0;
                        }
                    } else {
                        // Shift left: drop leftmost samples, pad right with zero.
                        let shift_bins =
                            ((-shift_amount * n as f64) as usize).min(n - 1);
                        ls.layers[k].rotate_left(shift_bins);
                        for i in (n - shift_bins)..n {
                            ls.layers[k][i] = 0.0;
                        }
                    }
                }
            }

            let description = "parameter space translation".to_string();
            let novelty = base.distance(&shifted);
            let predicted_family = self.regression.predict(&shifted);

            hypotheticals.push(HypotheticalTopology {
                landscape: shifted,
                predicted_family,
                deformation: description,
                novelty_score: novelty,
            });

            // Deformation 3: interpolate between two historical landscapes.
            let other_idx = rng.gen_range(0..history_len);
            if other_idx != idx {
                let other = &self.landscape_history[other_idx];
                let alpha = rng.gen_range(0.0..1.0);

                let mut interpolated = base.clone();
                for (ls_a, ls_b) in interpolated
                    .landscapes
                    .iter_mut()
                    .zip(other.landscapes.iter())
                {
                    for k in 0..ls_a.num_layers.min(ls_b.num_layers) {
                        for i in 0..ls_a.num_samples.min(ls_b.num_samples) {
                            ls_a.layers[k][i] = (1.0 - alpha) * ls_a.layers[k][i]
                                + alpha * ls_b.layers[k][i];
                        }
                    }
                }

                let description = format!(
                    "interpolation (alpha={:.2}) between ticks {} and {}",
                    alpha, base.tick, other.tick
                );
                let novelty = base.distance(&interpolated);
                let predicted_family = self.regression.predict(&interpolated);

                hypotheticals.push(HypotheticalTopology {
                    landscape: interpolated,
                    predicted_family,
                    deformation: description,
                    novelty_score: novelty,
                });
            }
        }

        hypotheticals
    }

    /// Compute the rate of topological change from recent history.
    fn landscape_velocity(&self) -> f64 {
        if self.landscape_history.len() < 2 {
            return 0.0;
        }

        let n = self.landscape_history.len();
        let current = &self.landscape_history[n - 1];
        let previous = &self.landscape_history[n - 2];

        current.distance(previous)
    }

    /// Predict expected topological transitions based on current
    /// Betti numbers, their trend, and landscape velocity.
    fn expected_transitions(
        &self,
        family: &TrajectoryFamily,
        velocity: f64,
    ) -> Vec<(u32, String)> {
        let mut transitions = Vec::new();

        let betti = &family.current_betti;
        let beta_0 = betti.first().copied().unwrap_or(1);
        let beta_1 = betti.get(1).copied().unwrap_or(0);

        // If fragmented and velocity is high, predict merging.
        if beta_0 > 1 && velocity > 0.1 {
            let estimated_ticks = ((beta_0 as f64 - 1.0) / velocity * 10.0) as u32;
            transitions.push((
                estimated_ticks.min(self.config.horizon_ticks),
                format!("component merger: beta_0 {} -> 1", beta_0),
            ));
        }

        // If loops present and landscape is changing, predict loop collapse.
        if beta_1 > 0 && velocity > 0.05 {
            let estimated_ticks = (beta_1 as f64 / velocity * 5.0) as u32;
            transitions.push((
                estimated_ticks.min(self.config.horizon_ticks),
                format!("loop collapse: beta_1 {} -> 0", beta_1),
            ));
        }

        // If single component with low velocity, predict stability.
        if beta_0 == 1 && beta_1 == 0 && velocity < 0.02 {
            transitions.push((
                self.config.horizon_ticks,
                "topological stability: no transitions expected".to_string(),
            ));
        }

        transitions
    }

    /// Complete pending entries whose trajectories have been fully observed.
    fn complete_pending_entries(&mut self, current_tick: u64) {
        let horizon = self.config.horizon_ticks as u64;
        let mut completed = Vec::new();

        for (i, (start_tick, bundle, partial)) in self.pending.iter_mut().enumerate() {
            // Accumulate observations into the partial trajectory.
            for (obs_tick, obs) in &self.observation_history {
                if *obs_tick > *start_tick
                    && *obs_tick <= *start_tick + horizon
                    && partial.len() < self.config.horizon_ticks as usize
                {
                    // Use the mean observation vector across points.
                    if !obs.is_empty() {
                        let dims = obs[0].len();
                        let mean: Vec<f64> = (0..dims)
                            .map(|d| {
                                obs.iter().map(|v| v[d]).sum::<f64>() / obs.len() as f64
                            })
                            .collect();
                        partial.push(mean);
                    }
                }
            }

            // Check if trajectory is complete.
            if current_tick >= *start_tick + horizon
                && !partial.is_empty()
            {
                completed.push(i);
            }
        }

        // Record completed entries in the library.
        // Process in reverse order to maintain valid indices during removal.
        for &i in completed.iter().rev() {
            if let Some((_, bundle, trajectory)) = self.pending.remove(i) {
                self.regression.record_observation(bundle, trajectory);
            }
        }
    }

    /// Force-complete all pending entries (called during Delta tick).
    fn force_complete_pending(&mut self) {
        while let Some((_, bundle, trajectory)) = self.pending.pop_front() {
            if !trajectory.is_empty() {
                self.regression.record_observation(bundle, trajectory);
            }
        }
    }
}
```

### TaAnalyzer trait implementation

```rust
/// Integration with the Bardo heartbeat via the TaAnalyzer trait.
///
/// The PredictiveTopology engine maps cleanly onto the three-timescale
/// architecture defined in Doc 0.
impl TaAnalyzer for PredictiveTopology {
    fn gamma_tick(
        &mut self,
        cortical: &CorticalState,
        ta_ext: &TaCorticalExtension,
    ) {
        // Read current observations from CorticalState.
        // In practice, observations come from the triage pipeline
        // via the event fabric. Here we show the CorticalState integration.
        let arousal = f32::from_bits(cortical.arousal.load(Ordering::Relaxed));

        // Adaptive observation window: higher arousal = more data points.
        let window_size = if arousal > 0.7 {
            128 // High arousal: wide observation window.
        } else if arousal > 0.4 {
            64
        } else {
            32 // Low arousal: narrow window, conserve compute.
        };

        // The actual observations would come from the event fabric.
        // This is a placeholder showing the interface.
        // let observations = self.collect_observations(window_size);
        // if let Some(family) = self.gamma_tick(&observations, current_tick) {
        //     let confidence_bits = (family.confidence as f32).to_bits();
        //     ta_ext.trajectory_confidence.store(confidence_bits, Ordering::Relaxed);
        // }
    }

    fn theta_tick(
        &mut self,
        _cortical: &CorticalState,
        _ta_ext: &TaCorticalExtension,
    ) -> Vec<TaInsight> {
        let forecasts = self.theta_tick();

        forecasts
            .into_iter()
            .map(|forecast| {
                let confidence = forecast.family.confidence as f32;
                let action_bias = match &forecast.family.dominant_constraint {
                    TopologicalConstraint::Collapsing => -0.3, // Caution: simplifying topology.
                    TopologicalConstraint::Complexifying => 0.2, // Opportunity: new features emerging.
                    TopologicalConstraint::Fragmented(_) => -0.5, // Strong caution: fragmented space.
                    TopologicalConstraint::Oscillatory(_) => 0.0, // Neutral: follow the cycle.
                    TopologicalConstraint::SingleComponent => 0.1, // Mild positive: stable space.
                };

                TaInsight {
                    kind: InsightKind::TopologicalForecast,
                    confidence,
                    targets: Vec::new(), // Filled by the event fabric.
                    action_bias,
                    horizon: forecast.family.horizon_ticks,
                    source: TaSubsystem::PredictiveGeometry,
                }
            })
            .collect()
    }

    fn delta_tick(
        &mut self,
        _cortical: &CorticalState,
        _ta_ext: &TaCorticalExtension,
    ) {
        self.delta_tick();
    }

    fn dream_nrem(&mut self, replay_buffer: &[TaEpisode]) {
        // Extract landscape-trajectory triples from episodes.
        let replay: Vec<(u64, LandscapeBundle, Vec<Vec<f64>>)> = replay_buffer
            .iter()
            .filter_map(|episode| {
                // Episodes contain raw observation data.
                // We recompute landscapes from observations.
                let observations = &episode.observation;
                let tick = episode.block_range.0;

                // Recompute diagram and landscape from the episode's observations.
                let obs_f32: Vec<Vec<f32>> = observations.to_f32_vecs();
                let diagram = self.tda.compute_diagram(&obs_f32)?;

                let mut bundle = LandscapeBundle::from_diagram(
                    &diagram,
                    &self.config.homology_dimensions,
                    self.config.num_layers,
                    self.config.num_samples,
                    self.config.t_min,
                    self.config.t_max,
                );
                bundle.tick = tick;

                let trajectory = episode.outcome.as_ref()?.to_trajectory_vec();
                Some((tick, bundle, trajectory))
            })
            .collect();

        self.dream_nrem(&replay);
    }

    fn dream_rem(&mut self, rng: &mut impl rand::Rng) {
        let hypotheticals = self.dream_rem(rng);

        // Store novel hypotheticals with high novelty scores
        // as synthetic library entries.
        for hyp in hypotheticals {
            if hyp.novelty_score > 0.5 && !hyp.predicted_family.trajectories.is_empty() {
                // Use the most probable predicted trajectory as the
                // "observed" trajectory for the synthetic entry.
                if let Some(best) = hyp.predicted_family.trajectories.first() {
                    self.regression
                        .record_observation(hyp.landscape, best.path.clone());
                }
            }
        }
    }

    fn death_testament(&self) -> TaTestament {
        // The predictive geometry subsystem contributes:
        // - High-fitness library entries as semantic patterns.
        // - The constraint engine's Betti trend model.
        // - Kernel bandwidth (learned from cross-validation).
        //
        // The actual testament construction is handled by the
        // TaTestament framework. This method extracts the
        // subsystem-specific knowledge.
        TaTestament {
            patterns: Vec::new(), // Filled by the testament framework.
            signal_families: Vec::new(),
            causal_skeleton: CausalGraph::empty(),
            somatic_associations: Vec::new(),
            metadata: TestamentMetadata {
                subsystem: TaSubsystem::PredictiveGeometry,
                library_size: self.regression.library.entries.len(),
                mean_fitness: self
                    .regression
                    .library
                    .entries
                    .iter()
                    .map(|e| e.fitness)
                    .sum::<f64>()
                    / self.regression.library.entries.len().max(1) as f64,
                kernel_bandwidth: self.config.kernel_bandwidth,
            },
        }
    }
}
```

## Subsystem interactions [SPEC]

The predictive geometry engine does not operate alone. It reads from and writes to other TA subsystems through CorticalState, and its outputs feed the Golem's decision-making pipeline.

### CorticalState integration [SPEC]

**Writes:**
- `trajectory_confidence` (AtomicU32): the confidence score from the most recent trajectory family prediction. Stored as f32 bit pattern. Range [0.0, 1.0]. Updated every Gamma tick. Other subsystems read this to assess how well the Golem can forecast the near future.

**Reads:**
- `topology_signal` (AtomicU32): Wasserstein distance from the base TDA pipeline. When this spikes, the predictive geometry engine knows a regime transition is in progress and should weight recent library entries more heavily.
- `betti_0`, `betti_1` (AtomicU16): raw Betti numbers from TDA. The constraint engine uses these directly.
- `liquidity_curvature` (AtomicU32): from Doc 2's manifold analyzer. High curvature regions constrain geodesics, which constrains the physical realizability of predicted trajectories.
- `pattern_confidence` (AtomicU32): from Doc 1's HDC pattern matcher. When pattern confidence is high, the current market state matches a known pattern, and trajectory predictions conditioned on that pattern should receive a confidence boost.
- `causal_density` (AtomicU32): from Doc 4's causal discovery engine. A sudden drop in causal density weakens the predictive geometry engine's confidence, because the causal structure that trajectory predictions relied on is breaking down.

### Oracle interaction

The Oracle receives `TopologicalForecast` values from Theta ticks and uses them as priors in its prediction pipeline. The interaction:

1. Predictive geometry produces a trajectory family with probabilities and constraints.
2. The Oracle combines this with its own statistical predictions (from historical data, causal models, and other TA subsystems).
3. Where the topology-derived and statistics-derived predictions agree, the Oracle's confidence increases.
4. Where they disagree, the Oracle investigates the source of disagreement. A high-confidence topological forecast that contradicts a statistical prediction suggests the statistical model is missing structural information.

The Oracle feeds approximately 15,000 prediction corrections per day back through the pipeline. Topological forecasts that systematically disagree with reality (the Oracle's corrections show the topology-derived predictions were wrong) trigger fitness decay in the landscape library. The predictive geometry engine self-corrects through this feedback.

### Dream integration

**NREM:** The consolidation engine selects high-surprise episodes involving topological transitions. These are replayed through the predictive geometry engine, which recomputes landscapes and compares predicted trajectories to observed ones. Episodes where the topology predicted the trajectory correctly reinforce the library entries that supported that prediction. Episodes where the topology failed to predict the trajectory decay those entries.

This is especially useful for rare events. A liquidity fragmentation event (β_0 jumping from 1 to 3) might happen only a few times in a Golem's lifetime. Without replay, the library would have one or two entries for that topological configuration, producing low-confidence predictions. With NREM replay, each occurrence is processed multiple times from slightly different angles (different consolidation contexts, different combinations with other replayed episodes), producing multiple library entries with varied fitness scores. The library's coverage of rare topological configurations improves without requiring those configurations to recur in the live market.

**REM:** The counterfactual engine generates novel landscape configurations by deforming historical landscapes. Three deformation types (documented in the implementation above):

1. **Amplitude scaling:** amplify or dampen individual landscape layers. This simulates "what if the most prominent topological feature were twice as strong?"
2. **Parameter translation:** shift the landscape along the filtration axis. This simulates "what if the topological features occurred at different scales?"
3. **Interpolation:** blend two historical landscapes. This simulates topological configurations that are intermediate between observed states.

The Golem predicts trajectories for these hypothetical topologies and stores the results as synthetic library entries. When a real market configuration eventually resembles a previously dreamed hypothetical, the Golem has a head start on trajectory prediction.

### Attention auction interaction

The predictive geometry engine's forecast uncertainty feeds into the attention auction as a bid modifier. Specifically:

- **High confidence, low uncertainty:** the Golem already knows what to expect. Attention bids from the predictive geometry subsystem are low. Other subsystems can claim those attention slots.
- **Low confidence, high uncertainty:** the Golem is in uncharted topological territory. Attention bids spike. The Golem pays more attention to the items driving the unfamiliar topology, gathering more data to improve the prediction.
- **High landscape velocity:** a topological transition is in progress. The predictive geometry subsystem bids aggressively for attention on the items involved in the transition, because observing the transition closely provides the highest information gain.

### Mortality interaction

Forecast accuracy feeds the epistemic death clock. The predictive geometry engine maintains a rolling accuracy score: for each trajectory prediction, how close was the most probable trajectory to the actual outcome? A declining accuracy score means the engine's topological model of the market is drifting from reality. This is exactly the condition the epistemic clock is designed to detect.

If the predictive geometry engine's accuracy falls below a threshold for an extended period, the epistemic clock decays faster. The Golem is losing its ability to understand the topological structure of the market. If it cannot recover (by gathering new data, replaying relevant episodes, or dreaming about the gap), this contributes to death.

Conversely, a predictive geometry engine with consistently high accuracy slows epistemic decay. The Golem understands the topology of its environment and can forecast where it is heading. This is a survival advantage.

## DeFi primitive coverage

The mathematical framework applies uniformly across DeFi primitives. The observation vectors change, the topological features change their interpretation, but the landscape-to-trajectory machinery is identical.

### Concentrated liquidity (Uniswap v3, v4)

**Observation space:** (current_tick, liquidity_at_tick, fee_growth, swap_volume, tick_bitmap_density).

**Topological signatures:**
- β_0 = 1, strong λ_1 peak: single concentrated liquidity cluster. Trajectories predict continued price oscillation within the cluster's range.
- β_0 > 1: liquidity has fragmented. Two or more tick ranges with concentrated liquidity separated by dead zones. Trajectory prediction: price will oscillate within one cluster until a large swap pushes it across the dead zone.
- β_1 in (price, fee_accrual) space: the pool is in a healthy cycling state. Price moves generate fees, attracting liquidity, which dampens price moves, which reduces fees, which thins liquidity, which amplifies price moves. The loop predicts continued cycling at a characteristic frequency.

### Lending markets (Aave, Compound, Morpho)

**Observation space:** (utilization, borrow_rate, supply_rate, total_borrows, health_factor_distribution_quartiles).

**Topological signatures:**
- β_0 in utilization-rate space: bimodal markets. Some pools near the rate kink, others far below. Trajectory: the two modes will persist until a correlated shock (ETH price move, stablecoin depeg) forces convergence.
- β_1 in rate-utilization space: the self-regulating loop. Borrowers respond to rates, rates respond to utilization, utilization responds to borrowing. Persistent β_1 predicts continued rate oscillation. β_1 collapse predicts one-directional utilization movement (either a rush to borrow or a mass repayment).
- β_2 void near health_factor = 1.0: a liquidation-cleared zone. Positions that were near liquidation have been liquidated. The void constrains trajectories: re-entry into this zone requires new borrowers to deliberately take on risk.

### Perpetual derivatives

**Observation space:** (funding_rate, open_interest, mark_price, basis, liquidation_levels).

**Topological signatures:**
- β_0 in (funding, OI) space: market fragmentation. Some contracts carry positive funding with high OI (long-heavy), others carry negative funding (short-heavy). Predict independent evolution until a directional move forces convergence.
- β_1 in (basis, funding) space: the basis arbitrage loop. Arbs compress basis, reducing funding, causing arbs to exit, widening basis. Persistent β_1 predicts that the basis trade remains profitable and self-correcting. Collapse predicts a dislocation where the arb breaks down (capital constraints, exchange risk, or extreme directional pressure overwhelming the arb flow).

### Yield tokens (Pendle)

**Observation space:** (PT_price, YT_price, implied_yield, maturity_remaining, underlying_yield).

**Topological signatures:**
- β_0 in (implied_yield, maturity) space: a fragmented yield curve. Different maturities are pricing wildly different yield expectations. Predict that the curve will steepen or flatten depending on which fragment carries more volume.
- β_1 in (PT_price, underlying_yield) space: PT pricing is cycling around the underlying. The market is repricing yield expectations back and forth. Persistent β_1 predicts continued PT price volatility around the fair value implied by underlying yield.

### Vaults (ERC-4626)

**Observation space:** (share_price, total_assets, redemption_queue_depth, strategy_allocation_vector).

**Topological signatures:**
- β_0 > 1 in (share_price, total_assets) space: the vault's share price has decoupled from NAV. Two clusters: one around fair value, one at a discount. Predict a potential bank-run dynamic where the discount cluster attracts more redemptions.
- β_1 in (share_price, strategy_return) space: the vault is cycling between strategies. Persistent β_1 means the vault operator is actively rotating, and the rotation has a characteristic period. Trajectory prediction follows the rotation cycle.

### Staking and restaking

**Observation space:** (stake_amount, reward_rate, delegation_flow, queue_depth, operator_concentration).

**Topological signatures:**
- β_0 in (stake_amount, reward_rate) space: fragmented staking landscape. Validators or operators occupy distinct clusters of stake-reward combinations. High-stake/low-reward operators (established, trusted) and low-stake/high-reward operators (new, speculative) form separate components. Trajectory prediction: the components evolve independently unless a slashing event or protocol change forces convergence.
- β_1 in (delegation_flow, reward_rate) space: a delegation-reward cycle. High rewards attract delegation, diluting rewards, causing withdrawal, concentrating rewards, attracting delegation again. Persistent β_1 predicts continued oscillation in delegation flows.

### Cross-primitive topology

The most interesting topological features appear when we combine observation spaces across primitives. An LP position on Uniswap v3, a borrow on Aave against the LP token, and a hedge via a perpetual on the same asset create a three-primitive observation space. The topology of this combined space captures correlations that no single-primitive analysis can see.

For example, β_1 in the combined (LP_fee_rate, borrow_rate, funding_rate) space indicates a cross-primitive cycle: LP fees and borrow costs and perp funding are locked in a feedback loop. This happens in practice when arbitrageurs use all three primitives simultaneously. A persistent cross-primitive loop predicts stability in the strategy (the arb is self-sustaining). Collapse of the loop predicts that one leg of the strategy is breaking down.

The predictive geometry engine handles cross-primitive observation spaces the same way as single-primitive ones. The observation vectors are wider, the persistence diagrams are richer, and the landscape library needs more entries to cover the space. But the mathematical machinery is identical.

## Cybernetic feedback loop

The predictive geometry engine participates in a tight feedback loop:

```
Observe topology (Gamma: persistence diagram → landscape)
    |
    v
Predict trajectory (Gamma: landscape → trajectory family via KNN)
    |
    v
Constrain predictions (Gamma: topological constraint filtering)
    |
    v
Act on predictions (Theta: Oracle uses forecast as prior)
    |
    v
Observe outcome (Gamma: record actual trajectory)
    |
    v
Update model (Delta: insert (landscape, trajectory) into library)
    |
    v
Refine kernel (Delta: cross-validate bandwidth)
    |
    v
Replay transitions (NREM: reinforce accurate predictions)
    |
    v
Imagine novel topologies (REM: deform landscapes, predict trajectories)
    |
    v
(back to Observe, with expanded library, refined kernel,
 and dream-generated coverage of unseen topologies)
```

Each iteration tightens the topology-trajectory association. Early in the Golem's life, the library is sparse and predictions are low-confidence. The constraint engine does most of the work, eliminating impossible trajectories based on Betti numbers alone. As the library fills, kernel regression takes over, producing targeted predictions weighted by topological similarity.

The dream cycle is what separates this from a static lookup table. NREM replay means every topological transition the Golem experiences gets replayed multiple times, each replay refining the library entries that supported the prediction. REM dreaming means the Golem explores topological configurations it has never seen, building up library entries for hypothetical futures. When one of those hypothetical futures actually occurs, the Golem is not encountering it for the first time.

The dream cycle is what makes this system different from a static lookup table or a trained model deployed and left alone. Static systems degrade as markets shift. Trained models need periodic retraining. The predictive geometry engine, embedded in the dream cycle, continuously self-corrects. NREM sharpens the existing library. REM expands it into hypothetical territory. Together, they keep the engine calibrated to whatever the market is doing now, not what it was doing when the library was initialized.

There is a risk of overfitting through this process. If REM dreams generate synthetic entries that are too similar to existing entries, the library becomes biased toward the already-known topology space. The novelty score threshold (0.5 in the implementation) prevents this: only genuinely novel hypotheticals enter the library. But the threshold itself should be subject to cross-validation during Delta ticks.

The mortality pressure sharpens this loop. A Golem with low epistemic vitality (poor prediction accuracy) needs the predictive geometry engine to work well. If it does not, the Golem dies faster. The Golem that builds the best topology-trajectory library lives longest and transfers that library to its successors via the death testament. Across generations, the predictive geometry engine improves through evolutionary selection on its knowledge base, not on its code.

The inter-generational transfer of library entries is particularly interesting for the predictive geometry engine. Most TA subsystems transfer parameters or model weights. This subsystem transfers actual data points: historical topology-trajectory pairs that survived fitness selection over an entire Golem lifetime. A successor Golem boots with a library that already covers the topological configurations its predecessor encountered. It does not need to observe those configurations again to predict their trajectories. The successor's first topological regime transition might already have library coverage from the predecessor's death testament.

This is Lamarckian inheritance applied to topological knowledge. The Golem does not evolve its topology-trajectory mapping through random mutation and natural selection alone. It actively curates its library, selects the fittest entries, and passes them to the next generation. Acquired knowledge -- specifically, acquired topological knowledge -- is heritable.

## Evaluation protocol [SPEC]

### Hypothesis

Topology-derived trajectory predictions provide information that purely statistical methods miss. The topological constraints narrow the trajectory space in ways that improve forecast accuracy, especially around regime transitions.

### Metrics

**Trajectory prediction accuracy.**

For each prediction, compute the weighted mean trajectory (probability-weighted average of paths in the family) and compare to the actual trajectory via mean squared error (MSE) normalized by trajectory variance:

```
NMSE = MSE(predicted, actual) / Var(actual)
```

NMSE < 1.0 means the topology-derived prediction is better than predicting the mean. Compare against:
- Naive baseline: predict the most recent observation repeated forward.
- Statistical baseline: autoregressive model (AR(p)) fitted on the observation series.
- TDA-only baseline: use the existing regime detection (calm/trending/volatile/crisis) and pick the historical average trajectory for that regime.

**Topological constraint filtering effectiveness.**

Measure the fraction of filtered trajectories that would have been wrong:

```
Filter precision = |{filtered trajectories that were wrong}| / |{all filtered trajectories}|
```

High filter precision means the constraint engine is correctly removing bad predictions. Low filter precision means it is removing predictions that would have been fine, and the constraints are too strict.

Also measure filter recall:

```
Filter recall = |{filtered trajectories that were wrong}| / |{all trajectories that were wrong}|
```

High recall means the constraint engine catches most of the bad predictions. Low recall means bad predictions survive filtering.

**Lead time.**

Compare how early the predictive geometry engine detects an upcoming regime transition versus the base TDA pipeline (which detects transitions by Wasserstein distance threshold) and versus statistical change-point detection (CUSUM, BOCPD).

Measure lead time as the number of Gamma ticks between the first trajectory family that assigns > 50% probability to a regime transition and the actual transition.

**Library efficiency.**

Track the library size over time and the mean fitness of entries. An efficient library is compact (old or useless entries pruned) with high mean fitness (remaining entries produce good predictions).

Track the fraction of trajectory predictions that had at least one "close" neighbor (landscape distance below the threshold). A library that frequently returns only distant neighbors is not covering the topological space well.

### Experimental design

Two Golem populations on 6 months of historical mainnet data:
- **Control:** base TDA pipeline (regime detection only), no predictive geometry.
- **Treatment:** full predictive geometry engine with landscape library, KNN regression, constraint filtering, and dream integration.

Both populations share all other subsystems (HDC, manifold, causal discovery, signal metabolism).

Primary comparison: risk-adjusted returns (Sharpe ratio) and prediction accuracy (Brier score for regime transitions, NMSE for trajectories).

Secondary comparison: survival time (Kaplan-Meier curves) and epistemic clock trajectory (does the treatment population maintain higher epistemic vitality?).

### Expected outcomes

The strongest gains should appear during regime transitions. When the market shifts from calm to volatile, the base TDA pipeline detects the shift after it happens. The predictive geometry engine, tracking landscape derivatives and Betti trends, should forecast the shift before it completes. This lead time translates directly into better positioning (reducing exposure before volatility arrives, increasing exposure before a trending regime).

During stable regimes, the gains should be smaller. The topology is not changing, the constraint engine adds little value (all trajectories are compatible with a stable topology), and the KNN regression degenerates to local statistical forecasting.

The most informative scenarios for evaluation are the transitions that Gidea and Katz (2018) studied in equity markets: pre-crash topological signatures. Their work showed that persistence landscape norms spike 50-100 trading days before major market drawdowns. In DeFi, the timescales compress. A "crash" in a Uniswap pool (liquidity exodus after a large IL event) might develop over hours, not months. The question is whether landscape norm spikes in DeFi observation spaces provide comparable lead times, compressed to the Gamma/Theta timescale.

We expect that cross-primitive topological features will provide the strongest predictive signal. Single-primitive topology (one pool, one lending market) is useful but limited. The DeFi-specific value comes from topological features that span multiple primitives -- the cross-primitive loops described in the DeFi coverage section. These features capture systemic dependencies that no single-market analysis can detect.

An honest concern: the non-parametric approach might not scale. If the Golem monitors 50 pools, 20 lending markets, and 10 derivative contracts, the combined observation space is high-dimensional and the landscape library needs exponentially more entries to cover it. The HDC encoding helps with retrieval speed, but coverage remains a cold-start problem. The evaluation should track library coverage (fraction of queries with at least one close neighbor) as a function of Golem lifetime. If coverage plateaus below 80% after a month of operation, the approach needs dimensionality reduction (project the observation space onto its principal topological directions before computing landscapes).

### Ablation study

The predictive geometry engine has four distinct components. Each can be disabled independently to measure its marginal contribution:

1. **Landscape computation only** (no regression, no constraints, no dreams). The Golem computes landscapes and writes landscape velocity to CorticalState, but produces no trajectory predictions. Does landscape velocity alone improve Oracle decisions?

2. **Landscape + KNN regression** (no constraints, no dreams). The Golem predicts trajectories from the library but does not filter them by topological constraints. Does constraint filtering add value or mostly remove valid trajectories?

3. **Full engine without dreams** (landscapes, regression, constraints, but no NREM replay or REM counterfactuals). Does the dream cycle improve prediction accuracy, or does the library fill adequately from waking experience alone?

4. **Full engine** (all components). The hypothesis is that this outperforms all ablations, but the margin between (3) and (4) quantifies the dream cycle's contribution.

### Failure criteria

If NMSE does not improve over the AR(p) baseline by at least 10% during regime transitions after 6 months of simulation, the complexity of the landscape-trajectory machinery is not justified. A simpler approach (direct Betti number trend extrapolation) might suffice.

If filter precision falls below 60%, the constraint engine is too aggressive and should be loosened or removed.

If the library exceeds the maximum configured size repeatedly despite pruning, the kernel bandwidth is too wide (too many entries are considered "close") and cross-validation should adjust.

If the ablation study shows that component (1) alone (landscape velocity, no trajectory prediction) captures 80% or more of the full engine's benefit, the KNN regression machinery is dead weight and should be stripped. This is a real possibility: landscape velocity is cheap to compute and may carry most of the topological predictive information. The regression, constraint filtering, and dream integration might be second-order effects. The evaluation must be honest about this.

## References

1. Bubenik, P. (2015). "Statistical topological data analysis using persistence landscapes." *Journal of Machine Learning Research,* 16(1), 77-102. The foundational paper: defines persistence landscapes, proves stability and invertibility, introduces the Banach space framework.

2. Perea, J.A. and Harer, J. (2015). "Sliding windows and persistence: an application of topological methods to signal analysis." *Foundations of Computational Mathematics,* 15(3), 799-838. Time-delay embeddings + persistent homology for periodic signal detection. Relevant to the oscillatory trajectory constraints derived from β_1.

3. Gidea, M. and Katz, Y. (2018). "Topological data analysis of financial time series: landscapes of crashes." *Physica A,* 491, 820-834. Empirical validation: persistence landscapes applied to S&P 500, DJIA, NASDAQ, and Russell 2000 data detect critical transitions preceding crashes. Landscape norms spike before major drawdowns.

4. Chazal, F., Fasy, B.T., Lecci, F., Rinaldo, A., and Wasserman, L. (2014). "Stochastic convergence of persistence landscapes and silhouettes." *Proceedings of the 30th Symposium on Computational Geometry (SoCG),* 474-483. Proves that the mean persistence landscape converges to its population counterpart at rate O(1/sqrt(n)). Justifies the use of sample means in the landscape Banach space.

5. Cohen-Steiner, D., Edelsbrunner, H., and Harer, J. (2007). "Stability of persistence diagrams." *Discrete & Computational Geometry,* 37(1), 103-120. The foundational stability result for persistent homology. Small perturbations in the input produce small perturbations in the diagram (and therefore in the landscape, by composition with Bubenik's stability theorem).

6. Silverman, B.W. (1986). *Density Estimation for Statistics and Data Analysis.* Chapman and Hall. The source of the bandwidth selection rule used in the adaptive kernel regression.

7. Edelsbrunner, H. and Harer, J.L. (2010). *Computational Topology: An Introduction.* American Mathematical Society. Background on Betti numbers, persistence diagrams, and the algebraic topology underlying the constraint engine.

8. Carlsson, G. (2009). "Topology and data." *Bulletin of the American Mathematical Society,* 46(2), 255-308. Survey paper motivating the application of algebraic topology to data analysis. Establishes that topological features are coordinate-free invariants, meaning they capture shape properties that survive reparametrization of the observation space.

9. Kanatani, K. (1990). *Group-Theoretical Methods in Image Understanding.* Springer. The connection between topological invariants and trajectory constraints draws on the invariant-theoretic perspective: topological features constrain the group of allowable transformations of the observation space, and trajectories are orbits under those transformations.

10. Morse, M. (1934). *The Calculus of Variations in the Large.* American Mathematical Society. Morse theory: the relationship between critical points of a function and the topology of its level sets. The Morse inequalities used in the rate-of-change constraints are a direct application.
