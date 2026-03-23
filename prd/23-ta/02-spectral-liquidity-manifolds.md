<!-- prd2/23-ta/02-spectral-liquidity-manifolds.md -->
<!-- Source: tmp/research/witness-research/new/ta/02-spectral-liquidity-manifolds.md -->

> **Version**: 1.0 | **Status**: Active | **Section**: 23-ta
>
> **Crates**: `bardo-ta-manifold`
>
> **Cross-references**:
> - [01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- the 32-signal perception surface where manifold signals (`liquidity_curvature`, `geodesic_cost`, `manifold_stability`) are written for other subsystems
> - [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- the 9-step decision cycle whose Gamma ticks update manifold state and Theta ticks run full geodesic computation
> - [14-chain/03-protocol-state.md](../14-chain/03-protocol-state.md) -- live protocol state (pool reserves, tick bitmaps, position data) providing the coordinate source for manifold construction
> - [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- HDC foundations enabling parallel transport of pattern hypervectors across manifold charts
> - [23-ta/00-witness-as-technical-analyst.md](00-witness-as-technical-analyst.md) -- Doc 0: prerequisite system context mapping the full data pipeline
> - [23-ta/01-hyperdimensional-technical-analysis.md](01-hyperdimensional-technical-analysis.md) -- Doc 1: HDC pattern algebra whose hypervectors are transported across the manifold
> - [23-ta/04-causal-microstructure-discovery.md](04-causal-microstructure-discovery.md) -- Doc 4: causal inference engine that discovers directed edges between manifold curvature events
> - [23-ta/05-predictive-geometry.md](05-predictive-geometry.md) -- Doc 5: topology-to-trajectory forecasting that uses manifold geodesics as prediction constraints

---

> **Reader orientation:** This document applies Riemannian geometry to concentrated liquidity, modeling DeFi execution as geodesics on curved manifolds within the Golem (mortal autonomous DeFi agent) runtime. It belongs to the **TA research layer** (Doc 2 of 10) and covers metric tensor construction from pool state, geodesic computation for optimal execution paths, curvature detection for liquidity crises, and parallel transport for cross-pool pattern comparison. You should have differential geometry background and understand Uniswap V3 tick mechanics. For Bardo-specific terms, see `prd2/shared/glossary.md`.

# Spectral Liquidity Manifolds [SPEC]: Riemannian geometry for DeFi execution

**Audience:** Researchers and engineers with differential geometry background and DeFi familiarity, but not necessarily familiar with the Bardo runtime.

## Abstract

Every DeFi operation has a cost that depends on where you are and where you are going. Swap slippage depends on pool reserves. Gas depends on network congestion. Bridge fees depend on destination chain liquidity. These costs are not independent: adjusting an LP position while rebalancing a lending position produces correlated costs that neither operation's marginal cost predicts. This paper treats the space of DeFi protocol states as a Riemannian manifold, where the metric tensor encodes the local cost of moving between states. The formulation yields geodesics (minimum-cost execution paths), curvature (instability detection before price moves), and parallel transport (principled translation of learned patterns across protocol contexts). We describe the mathematical foundations, integrate the manifold with an autonomous agent runtime's heartbeat cycle, provide a near-complete Rust implementation, and propose falsifiable evaluation criteria. The central bet: geometry warps before prices move, and an agent that reads the curvature has a structural advantage over one that reads the price.

## The problem

Bardo is a Rust runtime for autonomous DeFi agents called Golems. Each Golem runs a 9-step heartbeat cycle (observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect) on an adaptive clock with three frequency tiers: gamma ticks (5-15 seconds, perception), theta ticks (30-120 seconds, cognition), and delta ticks (~50 theta ticks, consolidation). At each gamma tick, the Golem writes atomic signals to a shared perception surface called `CorticalState`, roughly 256 bytes of lock-free atomic values that downstream subsystems read without contention.

The Golem operates across many DeFi protocols simultaneously. It holds LP positions on Uniswap, lends on Aave, deposits into yield vaults, stakes on restaking protocols, and watches derivatives markets for hedging. Each of these protocols has its own state: reserve ratios, utilization curves, TVL trajectories, funding rates. The Golem needs to answer questions like:

- "What is the cheapest way to move from my current position set to this target allocation?"
- "Is the current cost structure stable, or is it about to shift?"
- "I learned that this LP rebalancing pattern works well on Uniswap V3. How does it translate to Curve?"

These are geometric questions. The first is a shortest-path problem. The second asks about curvature. The third is parallel transport. But the Golem currently answers them with ad hoc heuristics: simulate a few candidate paths via Mirage-rs, pick the cheapest one, and hope the cost structure doesn't change during execution.

This paper formalizes the cost structure as a Riemannian metric and builds the machinery to answer all three questions with computation rather than guesswork.

## Mathematical foundations [SPEC]

### The state manifold

Define the DeFi state manifold *M* as an *N*-dimensional smooth manifold. Each coordinate corresponds to a protocol state variable:

$$\mathbf{x} = (x_1, x_2, \ldots, x_N) \in M$$

The coordinates decompose by protocol type:

- $x_1 \ldots x_k$: AMM pool states (reserve ratios, concentrated liquidity tick distributions, accumulated fee revenue per unit liquidity)
- $x_{k+1} \ldots x_m$: lending protocol states (utilization ratios, borrow/supply rates, aggregate collateral factors)
- $x_{m+1} \ldots x_p$: vault states (total value locked, share price relative to underlying, withdrawal queue depth)
- $x_{p+1} \ldots x_q$: staking states (total staked, validator/operator delegation distribution, slashing history)
- $x_{q+1} \ldots x_r$: derivatives states (open interest, funding rates, implied volatility surfaces, Greeks)
- $x_{r+1} \ldots x_N$: auxiliary states (gas price, cross-chain bridge liquidity, token streaming rates)

In practice, *N* ranges from 50 to 500 depending on how many protocols the Golem tracks. A Golem monitoring 5 Uniswap V3 pools, 3 Aave markets, 2 vaults, and a staking position has roughly N = 80 state variables. A Golem tracking the full Ethereum DeFi surface could reach N = 500.

The manifold is not globally Euclidean. Protocol states have boundaries (utilization cannot exceed 100%, reserves cannot go negative), nonlinear constraints (constant product curves, bonding curves), and discrete jumps (governance parameter changes, liquidation cascades). We work with the interior of the feasible region and treat boundaries as high-curvature zones where the metric tensor diverges, an accurate model of reality: operations near protocol limits cost much more than operations in the interior.

### The metric tensor

The metric tensor $g_{ij}(\mathbf{x})$ at point $\mathbf{x}$ defines the infinitesimal cost of moving from $\mathbf{x}$ in direction $d\mathbf{x}$:

$$ds^2 = \sum_{i,j} g_{ij}(\mathbf{x}) \, dx_i \, dx_j$$

This is a quadratic form on the tangent space $T_\mathbf{x}M$. The value $ds$ is the cost of the infinitesimal state change $d\mathbf{x}$.

The metric tensor components encode different cost sources:

**Diagonal terms** $g_{ii}$: the self-cost of changing state variable $i$ in isolation. Changing AMM reserves incurs slippage. Changing vault TVL incurs deposit/withdrawal fees. Changing a lending position incurs gas. These costs depend on the current state: slippage is higher when reserves are imbalanced, gas is higher during congestion.

**Off-diagonal terms** $g_{ij}$: the interaction cost of changing variables $i$ and $j$ simultaneously. Adjusting an LP position while the lending utilization on the same asset is changing produces correlated costs: the LP adjustment affects the lending rate (through supply changes), and the lending rate affects the optimal LP range. These cross terms capture the cost coupling that per-protocol analysis misses.

The metric tensor has three properties that constrain its estimation:

**Positive definiteness.** All movements cost something. Free state transitions don't exist in DeFi -- even a no-op costs gas. Mathematically, $g_{ij}(\mathbf{x})$ must be a positive-definite matrix at every point.

**State dependence.** Costs change with market conditions. The metric at point $\mathbf{x}$ differs from the metric at point $\mathbf{y}$. A swap when pools are balanced costs less than the same swap when pools are depleted on one side.

**Time variation.** The manifold deforms as markets evolve. Gas prices change. Liquidity migrates between venues. Protocol parameters get updated through governance. The metric tensor is a function of both position and time: $g_{ij}(\mathbf{x}, t)$.

**Asymmetry.** Buying costs different from selling. Depositing costs different from withdrawing. Strictly, this makes the cost structure Finslerian rather than Riemannian (Finsler metrics allow direction-dependent costs). We handle this by symmetrizing: $\hat{g}_{ij} = \frac{1}{2}(g_{ij}^{+} + g_{ij}^{-})$ where $g^{+}$ and $g^{-}$ are the forward and reverse costs. The asymmetry is stored separately as a skew tensor $a_{ij} = \frac{1}{2}(g_{ij}^{+} - g_{ij}^{-})$ that enters directional cost estimates.

**Concrete metric construction.** The metric decomposes into four weighted components:

$$g_{ij}(\mathbf{x}) = \alpha \cdot S_{ij}(\mathbf{x}) + \beta \cdot G_{ij}(\mathbf{x}) + \gamma \cdot T_{ij}(\mathbf{x}) + \delta \cdot O_{ij}(\mathbf{x})$$

where:

- $S_{ij}$: slippage cost matrix, estimated from AMM curve shapes, order book depth, and recent execution fills
- $G_{ij}$: gas cost matrix, estimated from recent gas prices and per-operation gas consumption
- $T_{ij}$: time cost matrix, encoding time-to-finality for cross-chain operations and settlement delays
- $O_{ij}$: opportunity cost matrix, the cost of capital locked during the state transition

The weights $\alpha, \beta, \gamma, \delta$ are configurable per Golem and can be adjusted based on the agent's current priorities. A Golem in a hurry weights time cost high. A Golem optimizing for cost efficiency weights slippage high. The weights sum to 1 for normalization.

Each component matrix is estimated from recent execution data (the Golem's own trades, observed on-chain transactions) and simulation results from Mirage-rs (fork simulation of hypothetical operations). The metric tensor is therefore empirical, not theoretical: it comes from the data, and it updates as the data changes.

### Christoffel symbols

The Christoffel symbols of the second kind encode how the coordinate basis vectors change as you move along the manifold. They are computed from the metric tensor and its first derivatives:

$$\Gamma^k_{ij} = \frac{1}{2} \sum_l g^{kl} \left( \frac{\partial g_{jl}}{\partial x_i} + \frac{\partial g_{il}}{\partial x_j} - \frac{\partial g_{ij}}{\partial x_l} \right)$$

where $g^{kl}$ are the components of the inverse metric tensor.

In DeFi terms, the Christoffel symbols encode how cost structures change as you traverse the state space. A large $\Gamma^k_{ij}$ means that moving in the $i$ and $j$ directions simultaneously produces an acceleration in the $k$ direction that you would not predict from either direction alone. This is the mathematical signature of DeFi composability: protocols interact, and those interactions create non-trivial geometry.

Computing Christoffel symbols requires the metric derivative field. Since the metric is sampled at discrete points (not given as a closed-form expression), we estimate derivatives via finite differences between neighboring grid points.

### Geodesics: optimal execution paths

A geodesic $\gamma(t)$ from state $\mathbf{x}_0$ to state $\mathbf{x}_1$ is the curve that minimizes total cost:

$$L[\gamma] = \int_0^1 \sqrt{\sum_{i,j} g_{ij}(\gamma(t)) \, \dot{\gamma}_i(t) \, \dot{\gamma}_j(t)} \, dt$$

The minimizing curve satisfies the geodesic equation:

$$\frac{d^2 x^k}{dt^2} + \sum_{i,j} \Gamma^k_{ij} \frac{dx^i}{dt} \frac{dx^j}{dt} = 0$$

This is a system of *N* coupled second-order ODEs. The Christoffel symbols act as "forces" that deflect the path from a straight line in coordinate space toward the minimum-cost route.

In DeFi terms: "What sequence of operations (swaps, LP adjustments, vault deposits, borrow changes) minimizes total execution cost to reach a target portfolio state?" The geodesic is not necessarily a single transaction. It might be a multi-step path: first swap token A for token B on venue 1, then deposit B into a vault, then use the vault shares as collateral on a lending protocol. The geodesic equation finds the optimal sequencing and sizing.

The geodesic might also be non-obvious. Direct swaps between two tokens might cost more than routing through a third token, not because of a simple arbitrage, but because the metric tensor is shaped such that the indirect path traverses lower-cost regions of the manifold. Existing DEX aggregators do a version of this for single swaps; the manifold approach generalizes it to multi-protocol, multi-step execution paths.

**Boundary conditions.** The geodesic is a boundary value problem: we know the start and end points but not the path. Numerical solution via the shooting method (guess initial velocity, integrate the ODE, adjust until the endpoint matches) or relaxation methods (discretize the path, iteratively minimize the cost functional). For real-time execution, we use a hybrid: precompute geodesics for common routes during delta ticks, then refine them at gamma frequency using the latest metric tensor.

### Curvature: instability detection

The Riemann curvature tensor $R^l{}_{ijk}$ measures how much parallel transport around an infinitesimal loop changes a vector. It is computed from the Christoffel symbols and their derivatives:

$$R^l{}_{ijk} = \frac{\partial \Gamma^l_{jk}}{\partial x_i} - \frac{\partial \Gamma^l_{ik}}{\partial x_j} + \sum_m \left( \Gamma^l_{im} \Gamma^m_{jk} - \Gamma^l_{jm} \Gamma^m_{ik} \right)$$

Contracting the Riemann tensor yields the Ricci tensor:

$$R_{ij} = \sum_k R^k{}_{ikj}$$

And contracting again with the inverse metric gives the Ricci scalar:

$$R = \sum_{i,j} g^{ij} R_{ij}$$

The Ricci scalar is a single number at each point that summarizes the average curvature. Its sign has physical meaning on the liquidity manifold:

**High positive curvature.** The cost surface is locally convex. Moving away from the current state in any direction costs more the further you go, and returning is cheap. This is a stable configuration: the current state is a cost basin. Markets with balanced liquidity, low volatility, and predictable gas prices produce positive curvature.

**High negative curvature.** The cost surface is saddle-shaped. Some perturbations are cheap to make but expensive to reverse. This is unstable: the system can slide into a new state at low cost but cannot easily return. Markets approaching liquidation thresholds, protocols with utilization near 100%, or pools with severely imbalanced reserves produce negative curvature.

**Near-zero curvature.** The cost surface is locally flat. Costs are approximately uniform in all directions. This is the unremarkable case: no strong stability or instability signal.

**Curvature as a leading indicator.** The geometry of the cost surface changes before prices change. When a lending protocol's utilization creeps toward its kink point, the curvature around that region of the manifold turns negative (the cost of small further increases in utilization explodes, while the cost of decreases is moderate). This curvature shift is detectable from the metric tensor alone, before the interest rate spike that utilization triggers. Similarly, when AMM liquidity thins on one side of a pool, the curvature in the direction of further depletion goes negative. The pool is "leaning" in a geometrically precise sense.

This connects to the topological regime detection described in the companion paper (Doc 1 in this series). TDA detects structural fractures in observation space. Curvature detects instability in cost space. They are complementary: TDA asks "is the shape of the data changing?" while curvature asks "is the cost of moving through the data changing?" Both can fire before price indicators, for different geometric reasons.

**Sectional curvature.** The Ricci scalar averages over all directions. For directional information, sectional curvature $K(\sigma)$ for a 2-plane $\sigma$ spanned by vectors $u$ and $v$ is:

$$K(\sigma) = \frac{R(u, v, v, u)}{g(u, u) g(v, v) - g(u, v)^2}$$

Sectional curvature tells you the instability of a specific pair of protocol interactions. Negative sectional curvature in the plane spanned by "Uniswap V3 ETH/USDC reserves" and "Aave ETH utilization" means that simultaneous movements in these two variables produce non-linear cost amplification. This is actionable: the Golem knows which specific protocol pair is becoming unstable.

### Parallel transport: cross-protocol pattern transfer

Given a tangent vector $\mathbf{v}$ at point $\mathbf{x}_A$ (representing a learned pattern in protocol A's neighborhood of the manifold), parallel transport along a curve $\gamma$ from $\mathbf{x}_A$ to $\mathbf{x}_B$ yields $\mathbf{v}'$ at $\mathbf{x}_B$:

$$\frac{dv^k}{dt} + \sum_{i,j} \Gamma^k_{ij} v^i \frac{d\gamma^j}{dt} = 0$$

This is a system of *N* first-order linear ODEs along the curve $\gamma$.

The transported vector $\mathbf{v}'$ is the "same" pattern as $\mathbf{v}$, adjusted for the curvature of the cost surface between the two protocols. If the manifold were flat (zero curvature), parallel transport would be trivial: the pattern transfers unchanged. Non-zero curvature means the pattern rotates and stretches as it crosses regions where cost structures differ.

Concrete example: the Golem has learned that when Uniswap V3 ETH/USDC pool reserves become imbalanced by more than 15%, rebalancing the LP range by 3 ticks in the heavy direction and simultaneously reducing the LP size by 10% minimizes total cost over the next 50 gamma ticks. This "knowledge" is a tangent vector at the Golem's current manifold position in the Uniswap coordinate subspace. The Golem now wants to apply the same insight to a Curve stETH/ETH pool that is experiencing a similar imbalance.

Without parallel transport, the Golem would either: (a) try the exact same parameters (wrong, because Curve's bonding curve has different cost geometry), or (b) learn the Curve pattern from scratch (expensive, takes many observations). With parallel transport, the Golem transports the pattern vector along a geodesic from the Uniswap neighborhood to the Curve neighborhood. The Christoffel symbols along the path encode how the cost geometry changes between AMM implementations. The transported vector arrives at the Curve position already adjusted for those differences.

This is not analogy. It is a computation. The result depends on the actual metric tensor (measured from execution data), the actual geodesic (computed from that metric), and the actual Christoffel symbols (derived from the metric's gradient). If the underlying geometry says the protocols are structurally similar, the transport will be nearly identity (the pattern transfers unchanged). If the geometry says they are radically different, the transport will significantly rotate the pattern, or the geodesic will be very long (high transport cost, suggesting the analogy is weak).

### Exponential map and logarithmic map

Two additional operations are useful for manifold-based computation:

The **exponential map** $\text{Exp}_\mathbf{x}: T_\mathbf{x}M \to M$ takes a tangent vector at $\mathbf{x}$ and follows the geodesic in that direction for unit time, returning the endpoint. This converts "a direction and magnitude of change" into "the state you reach by executing that change optimally."

The **logarithmic map** $\text{Log}_\mathbf{x}: M \to T_\mathbf{x}M$ is the inverse: given two points, it returns the initial velocity of the geodesic connecting them. This converts "where I am and where I want to be" into "the optimal direction and magnitude of change."

Together, these maps allow the Golem to reason in tangent space (where linear algebra works) and project results back to the manifold (where DeFi operations happen). Planning occurs in tangent space; execution occurs on the manifold.

The exponential map has a radius of injectivity: beyond a certain distance, geodesics from a point may cross, and the map is no longer one-to-one. On the liquidity manifold, this has a natural interpretation. The injectivity radius is small in high-curvature regions (near protocol limits, during crises) and large in low-curvature regions (balanced liquidity, calm markets). When the injectivity radius shrinks, the Golem's ability to plan reliably using tangent-space reasoning degrades, and it should switch to direct simulation (Mirage-rs) for validation. The manifold tracks its own reliability.

### Geodesic completeness and singularities

A Riemannian manifold is geodesically complete if every geodesic can be extended indefinitely. The liquidity manifold is not complete. Protocol state boundaries (zero reserves, 100% utilization, zero TVL) are metric singularities where the cost of approaching the boundary diverges. In Riemannian terms, the manifold has a boundary at finite coordinate distance but infinite geodesic distance. This is physically correct: you cannot drain a pool to zero because the cost of the last unit of liquidity approaches infinity.

Singularities appear concretely as eigenvalues of the metric tensor diverging. When one eigenvalue grows much larger than the others, the condition number spikes, the manifold becomes "pinched" in that direction, and geodesics curve away from the singularity. The `manifold_stability` signal in CorticalState (the condition number) tracks proximity to singularities. A Golem approaching a metric singularity should reduce position size in the corresponding protocol dimension.

Liquidation cascades create transient singularities. When a lending protocol mass-liquidates, the metric tensor in the collateral dimension spikes (selling pressure makes liquidation cheap for the liquidator but extremely expensive for the liquidated position), creating a temporary singularity that resolves as prices stabilize. The manifold records these transient events as curvature spikes, and the dream system can simulate their recurrence.

### The Frechet mean on the manifold

Given a collection of manifold points $\{p_1, \ldots, p_K\}$, their Frechet mean is:

$$\bar{p} = \arg\min_{p \in M} \sum_{k=1}^K d(p, p_k)^2$$

where $d(p, q)$ is the geodesic distance. This generalizes the Euclidean centroid to curved spaces. On the liquidity manifold, the Frechet mean of the Golem's recent positions is the "center of mass" of its operational neighborhood, accounting for the fact that some directions are more expensive to traverse than others.

The Frechet mean is useful for defining the Golem's "home" position on the manifold, which serves as the reference point for curvature monitoring and geodesic precomputation. It updates slowly (exponential moving average of positions, computed in tangent space via the logarithmic map and projected back via the exponential map) and provides a stable anchor even as the Golem's tick-by-tick position fluctuates.

Computing the Frechet mean on a Riemannian manifold requires iterative optimization (gradient descent on the sum-of-squared-distances objective, where each gradient step uses the logarithmic map). We use 10 iterations of Riemannian gradient descent at each delta tick, which converges well for the mild curvatures typical of normal market conditions.

## Architecture [SPEC]

### Heartbeat integration [SPEC]

The manifold computation distributes across the Golem's heartbeat tiers:

**Gamma tick (5-15 seconds).** Update the metric tensor at the Golem's current position using the latest execution costs, gas prices, and protocol state changes. Compute local Ricci scalar and write it to CorticalState. Check if nearby curvature has crossed sign thresholds. This is a lightweight operation: updating a single metric tensor and computing one curvature value. Budget: <10ms.

**Theta tick (30-120 seconds).** Compute or refine geodesics for planned operations. The Golem's strategy layer proposes target states; the manifold layer computes optimal paths. Curvature data feeds the Oracle's prediction engine as a feature. Parallel transport translates patterns between protocols when the strategy layer requests cross-protocol inference. Budget: <200ms.

**Delta tick (consolidation).** Full manifold maintenance. Recompute the metric field at all sampled grid points from accumulated execution data. Rebuild the Christoffel symbol cache. Prune stale grid points where the Golem has not operated recently. Pre-compute geodesics for common execution routes (the "geodesic atlas"). Imagine manifold deformations during REM dreams: "what if Uniswap V3 ETH/USDC liquidity halved?" and pre-compute geodesics for the hypothetical manifold. Budget: seconds-scale, runs during dream consolidation.

**Dreams (REM).** The Golem's dream cycle imagines counterfactual manifold states. By perturbing the metric tensor (halving certain liquidity values, doubling gas prices, removing a protocol entirely), the dream system generates hypothetical manifolds and pre-computes geodesic atlases for them. When the actual manifold shifts toward a dreamed state, the Golem already has precomputed execution paths. Dreams trade compute now for reaction speed later.

### CorticalState integration [SPEC]

Three atomic values on the perception surface:

```rust
// In CorticalState
pub liquidity_curvature: AtomicU32,  // f32 as bits: Ricci scalar at current position
pub geodesic_cost: AtomicU32,        // f32 as bits: cost of optimal path to nearest opportunity
pub manifold_stability: AtomicU32,   // f32 as bits: condition number of metric tensor
```

`liquidity_curvature` is the Ricci scalar. Negative values signal instability. The magnitude indicates severity.

`geodesic_cost` is the integrated cost along the shortest geodesic from the Golem's current position to the nearest identified opportunity. High values mean good opportunities are expensive to reach. Low values mean cheap execution paths exist.

`manifold_stability` is the condition number of the metric tensor (ratio of largest to smallest eigenvalue). A high condition number means the metric is ill-conditioned: costs vary wildly by direction, making the manifold "thin" in some dimensions. This correlates with execution risk, because small errors in direction produce large cost differences.

### Coordinate map

The bridge between DeFi protocol states and manifold coordinates requires a mapping layer. Raw protocol state variables have different units, scales, and ranges. The coordinate map normalizes them into a common manifold coordinate system.

```
Protocol state                    Manifold coordinate
─────────────────────────────────────────────────────
Uniswap V3 reserve0: 1.5e18      x_1 = 0.73 (normalized by pool TVL)
Aave utilization: 0.82            x_14 = 0.82 (already a ratio)
Vault TVL: $50M                   x_28 = 0.65 (normalized by historical range)
Gas price: 35 gwei                x_71 = 0.44 (normalized by 30-day range)
```

Normalization matters because the metric tensor operates on coordinate differentials. If one coordinate ranges from 0 to 1 and another from 0 to 1e18, the metric tensor will be dominated by the large-scale coordinate regardless of its actual cost significance. Normalization by protocol-appropriate scales puts all coordinates on comparable footing.

## Rust implementation [SPEC]

### Manifold point

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// A point on the DeFi liquidity manifold.
/// Each coordinate is a normalized protocol state variable in [0, 1].
#[derive(Clone, Debug)]
pub struct ManifoldPoint {
    pub coordinates: Vec<f64>,
}

impl ManifoldPoint {
    pub fn new(dim: usize) -> Self {
        Self {
            coordinates: vec![0.0; dim],
        }
    }

    pub fn dimension(&self) -> usize {
        self.coordinates.len()
    }

    /// Euclidean distance (in coordinate space, not geodesic distance).
    /// Useful for grid lookups, not for cost estimation.
    pub fn coord_distance(&self, other: &ManifoldPoint) -> f64 {
        assert_eq!(self.coordinates.len(), other.coordinates.len());
        self.coordinates
            .iter()
            .zip(other.coordinates.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt()
    }
}
```

### Tangent vector

```rust
/// A vector in the tangent space at a manifold point.
/// Represents an infinitesimal state change direction.
#[derive(Clone, Debug)]
pub struct TangentVector {
    pub components: Vec<f64>,
}

impl TangentVector {
    pub fn new(dim: usize) -> Self {
        Self {
            components: vec![0.0; dim],
        }
    }

    pub fn dot(&self, other: &TangentVector) -> f64 {
        self.components
            .iter()
            .zip(other.components.iter())
            .map(|(a, b)| a * b)
            .sum()
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            components: self.components.iter().map(|c| c * s).collect(),
        }
    }

    pub fn add(&self, other: &TangentVector) -> Self {
        Self {
            components: self
                .components
                .iter()
                .zip(other.components.iter())
                .map(|(a, b)| a + b)
                .collect(),
        }
    }

    pub fn norm_squared(&self) -> f64 {
        self.dot(self)
    }
}
```

### Metric tensor

```rust
/// The Riemannian metric tensor at a point.
/// Stored as a symmetric positive-definite matrix in row-major order.
/// For an N-dimensional manifold, this is an N x N matrix with N*(N+1)/2
/// independent components due to symmetry.
#[derive(Clone, Debug)]
pub struct MetricTensor {
    /// N x N matrix in row-major order
    components: Vec<f64>,
    dimension: usize,
}

impl MetricTensor {
    pub fn new(dim: usize) -> Self {
        // Initialize as identity (flat Euclidean metric)
        let mut components = vec![0.0; dim * dim];
        for i in 0..dim {
            components[i * dim + i] = 1.0;
        }
        Self {
            components,
            dimension: dim,
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.components[i * self.dimension + j]
    }

    pub fn set(&mut self, i: usize, j: usize, val: f64) {
        // Enforce symmetry
        self.components[i * self.dimension + j] = val;
        self.components[j * self.dimension + i] = val;
    }

    /// Compute ds^2 = g_ij dx^i dx^j for a tangent vector dx.
    pub fn quadratic_form(&self, dx: &TangentVector) -> f64 {
        let n = self.dimension;
        let mut result = 0.0;
        for i in 0..n {
            for j in 0..n {
                result += self.get(i, j) * dx.components[i] * dx.components[j];
            }
        }
        result
    }

    /// Geodesic distance for an infinitesimal displacement dx.
    pub fn infinitesimal_distance(&self, dx: &TangentVector) -> f64 {
        self.quadratic_form(dx).sqrt()
    }

    /// Compute the inverse metric tensor g^{ij} via Cholesky decomposition.
    /// Returns None if the matrix is not positive definite.
    pub fn inverse(&self) -> Option<MetricTensor> {
        let n = self.dimension;
        // Cholesky: g = L L^T
        let mut l = vec![0.0_f64; n * n];

        for i in 0..n {
            for j in 0..=i {
                let mut sum = 0.0;
                for k in 0..j {
                    sum += l[i * n + k] * l[j * n + k];
                }
                if i == j {
                    let diag = self.get(i, i) - sum;
                    if diag <= 0.0 {
                        return None; // not positive definite
                    }
                    l[i * n + j] = diag.sqrt();
                } else {
                    l[i * n + j] = (self.get(i, j) - sum) / l[j * n + j];
                }
            }
        }

        // Invert L (forward substitution)
        let mut l_inv = vec![0.0_f64; n * n];
        for i in 0..n {
            l_inv[i * n + i] = 1.0 / l[i * n + i];
            for j in (0..i).rev() {
                let mut sum = 0.0;
                for k in (j + 1)..=i {
                    sum += l[i * n + k] * l_inv[k * n + j];
                }
                l_inv[i * n + j] = -sum / l[i * n + i]; // fix: should use l[j*n+j]?
                // Correct: l_inv[i*n+j] = -sum * l_inv[i*n+i]
                // but the above is equivalent since l_inv[i*n+i] = 1/l[i*n+i]
            }
        }

        // g^{-1} = (L^T)^{-1} L^{-1} = L_inv^T L_inv
        let mut inv = vec![0.0_f64; n * n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in i.max(j)..n {
                    sum += l_inv[k * n + i] * l_inv[k * n + j];
                }
                inv[i * n + j] = sum;
            }
        }

        Some(MetricTensor {
            components: inv,
            dimension: n,
        })
    }

    /// Eigenvalues of the metric tensor, sorted ascending.
    /// Uses the Jacobi eigenvalue algorithm for symmetric matrices.
    pub fn eigenvalues(&self) -> Vec<f64> {
        let n = self.dimension;
        let mut a = self.components.clone();

        // Jacobi iteration: rotate to diagonalize
        let max_iter = 100 * n * n;
        let tol = 1e-12;

        for _ in 0..max_iter {
            // Find largest off-diagonal element
            let mut max_val = 0.0_f64;
            let mut p = 0;
            let mut q = 1;
            for i in 0..n {
                for j in (i + 1)..n {
                    let val = a[i * n + j].abs();
                    if val > max_val {
                        max_val = val;
                        p = i;
                        q = j;
                    }
                }
            }
            if max_val < tol {
                break;
            }

            // Compute rotation angle
            let app = a[p * n + p];
            let aqq = a[q * n + q];
            let apq = a[p * n + q];
            let theta = 0.5 * (aqq - app).atan2(2.0 * apq);
            let c = theta.cos();
            let s = theta.sin();

            // Apply Jacobi rotation
            let mut new_a = a.clone();
            for i in 0..n {
                new_a[i * n + p] = c * a[i * n + p] - s * a[i * n + q];
                new_a[i * n + q] = s * a[i * n + p] + c * a[i * n + q];
                new_a[p * n + i] = new_a[i * n + p]; // symmetry
                new_a[q * n + i] = new_a[i * n + q];
            }
            new_a[p * n + p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
            new_a[q * n + q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
            new_a[p * n + q] = 0.0;
            new_a[q * n + p] = 0.0;
            a = new_a;
        }

        let mut eigenvalues: Vec<f64> = (0..n).map(|i| a[i * n + i]).collect();
        eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap());
        eigenvalues
    }

    /// Condition number: ratio of largest to smallest eigenvalue.
    /// High condition number = anisotropic cost structure = execution risk.
    pub fn condition_number(&self) -> f64 {
        let eigs = self.eigenvalues();
        if eigs.is_empty() || eigs[0].abs() < 1e-15 {
            return f64::INFINITY;
        }
        eigs.last().unwrap() / eigs[0]
    }
}
```

### Metric derivatives and Christoffel symbols

```rust
/// First derivatives of the metric tensor with respect to coordinates.
/// dg[k][i][j] = ∂g_ij / ∂x_k
pub struct MetricDerivatives {
    /// Indexed as [k][i * dim + j]
    pub dg: Vec<Vec<f64>>,
    pub dimension: usize,
}

impl MetricDerivatives {
    /// Estimate derivatives via central finite differences between
    /// neighboring grid points.
    pub fn from_metric_field(
        field: &MetricField,
        point: &ManifoldPoint,
        step: f64,
    ) -> Self {
        let dim = point.dimension();
        let mut dg = vec![vec![0.0; dim * dim]; dim];

        for k in 0..dim {
            // Perturb coordinate k forward and backward
            let mut forward = point.clone();
            let mut backward = point.clone();
            forward.coordinates[k] += step;
            backward.coordinates[k] -= step;

            let g_forward = field.interpolate(&forward);
            let g_backward = field.interpolate(&backward);

            for i in 0..dim {
                for j in 0..dim {
                    dg[k][i * dim + j] =
                        (g_forward.get(i, j) - g_backward.get(i, j)) / (2.0 * step);
                }
            }
        }

        Self { dg, dimension: dim }
    }
}

/// Christoffel symbols of the second kind: Γ^k_ij.
/// Encodes how the cost geometry changes as you move through state space.
pub struct ChristoffelSymbols {
    /// Γ^k_ij stored flat: index = k * dim * dim + i * dim + j
    pub components: Vec<f64>,
    pub dimension: usize,
}

impl ChristoffelSymbols {
    /// Compute from metric tensor and its derivatives.
    /// Γ^k_ij = (1/2) g^{kl} (∂g_{jl}/∂x_i + ∂g_{il}/∂x_j - ∂g_{ij}/∂x_l)
    pub fn from_metric(
        metric: &MetricTensor,
        derivatives: &MetricDerivatives,
    ) -> Option<Self> {
        let dim = metric.dimension();
        let g_inv = metric.inverse()?;
        let mut components = vec![0.0; dim * dim * dim];

        for k in 0..dim {
            for i in 0..dim {
                for j in 0..dim {
                    let mut val = 0.0;
                    for l in 0..dim {
                        let dg_jl_di = derivatives.dg[i][j * dim + l];
                        let dg_il_dj = derivatives.dg[j][i * dim + l];
                        let dg_ij_dl = derivatives.dg[l][i * dim + j];
                        val += g_inv.get(k, l) * (dg_jl_di + dg_il_dj - dg_ij_dl);
                    }
                    components[k * dim * dim + i * dim + j] = 0.5 * val;
                }
            }
        }

        Some(Self { components, dimension: dim })
    }

    pub fn get(&self, k: usize, i: usize, j: usize) -> f64 {
        self.components[k * self.dimension * self.dimension + i * self.dimension + j]
    }
}
```

### Curvature computation [SPEC]

```rust
/// Riemann curvature tensor R^l_{ijk} and derived quantities.
pub struct CurvatureComputer;

impl CurvatureComputer {
    /// Compute the Ricci scalar R = g^{ij} R_{ij} at a point.
    /// This is the single-number curvature summary written to CorticalState.
    pub fn ricci_scalar(
        metric: &MetricTensor,
        christoffel: &ChristoffelSymbols,
        christoffel_derivatives: &ChristoffelDerivatives,
    ) -> Option<f64> {
        let dim = metric.dimension();
        let g_inv = metric.inverse()?;

        // Ricci tensor R_{ij} = R^k_{ikj}
        // R^l_{ijk} = ∂Γ^l_{jk}/∂x_i - ∂Γ^l_{ik}/∂x_j
        //           + Σ_m (Γ^l_{im} Γ^m_{jk} - Γ^l_{jm} Γ^m_{ik})
        let mut ricci_scalar = 0.0;

        for i in 0..dim {
            for j in 0..dim {
                // R_{ij} = Σ_k R^k_{ikj}
                let mut r_ij = 0.0;
                for k in 0..dim {
                    // R^k_{ikj}
                    let d_gamma_kj_di = christoffel_derivatives.get(k, i, k, j);
                    let d_gamma_ki_dj = christoffel_derivatives.get(k, j, k, i);

                    let mut contraction = 0.0;
                    for m in 0..dim {
                        contraction += christoffel.get(k, i, m) * christoffel.get(m, k, j)
                            - christoffel.get(k, k, m) * christoffel.get(m, i, j);
                    }

                    r_ij += d_gamma_kj_di - d_gamma_ki_dj + contraction;
                }
                ricci_scalar += g_inv.get(i, j) * r_ij;
            }
        }

        Some(ricci_scalar)
    }

    /// Sectional curvature K(u, v) for a 2-plane spanned by tangent vectors u, v.
    /// Returns the curvature of the specific protocol-pair interaction.
    pub fn sectional_curvature(
        metric: &MetricTensor,
        christoffel: &ChristoffelSymbols,
        christoffel_derivatives: &ChristoffelDerivatives,
        u: &TangentVector,
        v: &TangentVector,
    ) -> Option<f64> {
        let dim = metric.dimension();

        // R(u, v, v, u) = R^l_{ijk} u^i v^j v^k g_{lm} u^m
        // (using index lowering)
        let mut numerator = 0.0;
        for i in 0..dim {
            for j in 0..dim {
                for k in 0..dim {
                    for l in 0..dim {
                        let r_lijk = riemann_component(
                            l, i, j, k, christoffel, christoffel_derivatives,
                        );
                        for m in 0..dim {
                            numerator +=
                                r_lijk * metric.get(l, m) * u.components[i]
                                * v.components[j] * v.components[k] * u.components[m];
                        }
                    }
                }
            }
        }

        // Denominator: g(u,u) g(v,v) - g(u,v)^2
        let guu = metric.quadratic_form(u);
        let gvv = metric.quadratic_form(v);
        let guv = {
            let mut val = 0.0;
            for i in 0..dim {
                for j in 0..dim {
                    val += metric.get(i, j) * u.components[i] * v.components[j];
                }
            }
            val
        };
        let denom = guu * gvv - guv * guv;

        if denom.abs() < 1e-15 {
            return None; // degenerate plane
        }

        Some(numerator / denom)
    }
}

/// Single component of the Riemann tensor.
fn riemann_component(
    l: usize,
    i: usize,
    j: usize,
    k: usize,
    christoffel: &ChristoffelSymbols,
    christoffel_derivatives: &ChristoffelDerivatives,
) -> f64 {
    let dim = christoffel.dimension;

    let d1 = christoffel_derivatives.get(l, i, j, k);
    let d2 = christoffel_derivatives.get(l, j, i, k);

    let mut contraction = 0.0;
    for m in 0..dim {
        contraction += christoffel.get(l, i, m) * christoffel.get(m, j, k)
            - christoffel.get(l, j, m) * christoffel.get(m, i, k);
    }

    d1 - d2 + contraction
}

/// Derivatives of Christoffel symbols.
/// d_christoffel[l][a][i][j] = ∂Γ^l_{ij} / ∂x_a
pub struct ChristoffelDerivatives {
    /// Flat storage: l * dim^3 + a * dim^2 + i * dim + j
    data: Vec<f64>,
    dimension: usize,
}

impl ChristoffelDerivatives {
    pub fn from_christoffel_field(
        field: &ChristoffelField,
        point: &ManifoldPoint,
        step: f64,
    ) -> Self {
        let dim = point.dimension();
        let mut data = vec![0.0; dim * dim * dim * dim];

        for a in 0..dim {
            let mut fwd = point.clone();
            let mut bwd = point.clone();
            fwd.coordinates[a] += step;
            bwd.coordinates[a] -= step;

            let gamma_fwd = field.interpolate(&fwd);
            let gamma_bwd = field.interpolate(&bwd);

            for l in 0..dim {
                for i in 0..dim {
                    for j in 0..dim {
                        let deriv = (gamma_fwd.get(l, i, j) - gamma_bwd.get(l, i, j))
                            / (2.0 * step);
                        data[l * dim * dim * dim + a * dim * dim + i * dim + j] = deriv;
                    }
                }
            }
        }

        Self { data, dimension: dim }
    }

    /// Get ∂Γ^l_{ij} / ∂x_a
    pub fn get(&self, l: usize, a: usize, i: usize, j: usize) -> f64 {
        let dim = self.dimension;
        self.data[l * dim * dim * dim + a * dim * dim + i * dim + j]
    }
}
```

### Geodesic computation [SPEC]

```rust
/// A discretized geodesic path on the manifold.
#[derive(Clone, Debug)]
pub struct GeodesicPath {
    pub points: Vec<ManifoldPoint>,
    pub total_cost: f64,
    pub num_steps: usize,
}

/// Geodesic solver using the shooting method.
/// Finds the minimum-cost path between two points on the manifold.
pub struct GeodesicSolver {
    step_size: f64,
    max_iterations: usize,
    convergence_tol: f64,
    num_path_steps: usize,
}

impl GeodesicSolver {
    pub fn new() -> Self {
        Self {
            step_size: 0.01,
            max_iterations: 200,
            convergence_tol: 1e-6,
            num_path_steps: 100,
        }
    }

    /// Compute the geodesic from `start` to `end` on the given manifold.
    /// Uses the shooting method: guess initial velocity, integrate the
    /// geodesic ODE, measure endpoint error, adjust velocity via Newton's
    /// method, repeat.
    pub fn solve(
        &self,
        manifold: &LiquidityManifold,
        start: &ManifoldPoint,
        end: &ManifoldPoint,
    ) -> Option<GeodesicPath> {
        let dim = start.dimension();

        // Initial velocity guess: straight line in coordinate space
        let mut velocity = TangentVector {
            components: start
                .coordinates
                .iter()
                .zip(end.coordinates.iter())
                .map(|(s, e)| e - s)
                .collect(),
        };

        for _iter in 0..self.max_iterations {
            // Integrate geodesic ODE with current initial velocity
            let path = self.integrate_geodesic(manifold, start, &velocity);

            // Measure endpoint error
            let endpoint = path.points.last()?;
            let error: Vec<f64> = endpoint
                .coordinates
                .iter()
                .zip(end.coordinates.iter())
                .map(|(a, b)| b - a)
                .collect();

            let error_norm: f64 = error.iter().map(|e| e * e).sum::<f64>().sqrt();
            if error_norm < self.convergence_tol {
                return Some(path);
            }

            // Compute Jacobian of endpoint with respect to initial velocity
            // via finite differences, then apply Newton correction.
            let jacobian = self.endpoint_jacobian(manifold, start, &velocity);
            let correction = solve_linear_system(&jacobian, &error)?;

            for i in 0..dim {
                velocity.components[i] += correction[i];
            }
        }

        // Did not converge; return best path so far
        let path = self.integrate_geodesic(manifold, start, &velocity);
        Some(path)
    }

    /// Integrate the geodesic ODE:
    ///   d²x^k/dt² + Γ^k_{ij} (dx^i/dt)(dx^j/dt) = 0
    ///
    /// Rewritten as a first-order system:
    ///   dx^k/dt = v^k
    ///   dv^k/dt = -Γ^k_{ij} v^i v^j
    ///
    /// Integrated via 4th-order Runge-Kutta.
    fn integrate_geodesic(
        &self,
        manifold: &LiquidityManifold,
        start: &ManifoldPoint,
        initial_velocity: &TangentVector,
    ) -> GeodesicPath {
        let dim = start.dimension();
        let dt = 1.0 / self.num_path_steps as f64;
        let mut points = Vec::with_capacity(self.num_path_steps + 1);

        let mut x = start.coordinates.clone();
        let mut v = initial_velocity.components.clone();
        points.push(ManifoldPoint {
            coordinates: x.clone(),
        });

        let mut total_cost = 0.0;

        for _ in 0..self.num_path_steps {
            // RK4 integration
            let (k1x, k1v) = self.geodesic_derivatives(manifold, &x, &v);
            let (x2, v2) = step_state(&x, &v, &k1x, &k1v, dt * 0.5);
            let (k2x, k2v) = self.geodesic_derivatives(manifold, &x2, &v2);
            let (x3, v3) = step_state(&x, &v, &k2x, &k2v, dt * 0.5);
            let (k3x, k3v) = self.geodesic_derivatives(manifold, &x3, &v3);
            let (x4, v4) = step_state(&x, &v, &k3x, &k3v, dt);
            let (k4x, k4v) = self.geodesic_derivatives(manifold, &x4, &v4);

            for i in 0..dim {
                let dx = (k1x[i] + 2.0 * k2x[i] + 2.0 * k3x[i] + k4x[i]) / 6.0;
                let dv = (k1v[i] + 2.0 * k2v[i] + 2.0 * k3v[i] + k4v[i]) / 6.0;
                x[i] += dx * dt;
                v[i] += dv * dt;
            }

            // Accumulate cost: ds = sqrt(g_ij v^i v^j) * dt
            let metric = manifold.metric_at(&ManifoldPoint {
                coordinates: x.clone(),
            });
            let tv = TangentVector {
                components: v.clone(),
            };
            total_cost += metric.infinitesimal_distance(&tv) * dt;

            points.push(ManifoldPoint {
                coordinates: x.clone(),
            });
        }

        GeodesicPath {
            points,
            total_cost,
            num_steps: self.num_path_steps,
        }
    }

    /// Compute derivatives for the geodesic ODE.
    /// dx/dt = v
    /// dv^k/dt = -Γ^k_{ij} v^i v^j
    fn geodesic_derivatives(
        &self,
        manifold: &LiquidityManifold,
        x: &[f64],
        v: &[f64],
    ) -> (Vec<f64>, Vec<f64>) {
        let dim = x.len();
        let point = ManifoldPoint {
            coordinates: x.to_vec(),
        };
        let christoffel = manifold.christoffel_at(&point);

        let dx: Vec<f64> = v.to_vec();
        let mut dv = vec![0.0; dim];

        for k in 0..dim {
            let mut acc = 0.0;
            for i in 0..dim {
                for j in 0..dim {
                    acc += christoffel.get(k, i, j) * v[i] * v[j];
                }
            }
            dv[k] = -acc;
        }

        (dx, dv)
    }

    /// Compute Jacobian of geodesic endpoint with respect to initial velocity.
    /// Used by the shooting method for Newton corrections.
    fn endpoint_jacobian(
        &self,
        manifold: &LiquidityManifold,
        start: &ManifoldPoint,
        velocity: &TangentVector,
    ) -> Vec<Vec<f64>> {
        let dim = start.dimension();
        let eps = 1e-6;
        let baseline = self.integrate_geodesic(manifold, start, velocity);
        let baseline_end = baseline.points.last().unwrap();

        let mut jacobian = vec![vec![0.0; dim]; dim];
        for j in 0..dim {
            let mut perturbed_v = velocity.clone();
            perturbed_v.components[j] += eps;
            let perturbed = self.integrate_geodesic(manifold, start, &perturbed_v);
            let perturbed_end = perturbed.points.last().unwrap();

            for i in 0..dim {
                jacobian[i][j] = (perturbed_end.coordinates[i]
                    - baseline_end.coordinates[i])
                    / eps;
            }
        }
        jacobian
    }
}

/// Helper: advance state by dt * derivatives.
fn step_state(
    x: &[f64],
    v: &[f64],
    dx: &[f64],
    dv: &[f64],
    dt: f64,
) -> (Vec<f64>, Vec<f64>) {
    let x_new: Vec<f64> = x.iter().zip(dx.iter()).map(|(xi, dxi)| xi + dxi * dt).collect();
    let v_new: Vec<f64> = v.iter().zip(dv.iter()).map(|(vi, dvi)| vi + dvi * dt).collect();
    (x_new, v_new)
}

/// Solve Ax = b via LU decomposition. Returns None if singular.
fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = b.len();
    let mut aug: Vec<Vec<f64>> = a
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut r = row.clone();
            r.push(b[i]);
            r
        })
        .collect();

    // Gaussian elimination with partial pivoting
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-15 {
            return None; // singular
        }
        aug.swap(col, max_row);

        // Eliminate below
        for row in (col + 1)..n {
            let factor = aug[row][col] / aug[col][col];
            for j in col..=n {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = aug[i][n];
        for j in (i + 1)..n {
            sum -= aug[i][j] * x[j];
        }
        x[i] = sum / aug[i][i];
    }
    Some(x)
}
```

### Parallel transport [SPEC]

```rust
/// Parallel transport a tangent vector along a geodesic path.
///
/// Solves: dv^k/dt + Γ^k_{ij} v^i (dγ^j/dt) = 0
/// along the discretized path using RK4.
pub struct ParallelTransporter;

impl ParallelTransporter {
    pub fn transport(
        manifold: &LiquidityManifold,
        vector: &TangentVector,
        path: &GeodesicPath,
    ) -> TangentVector {
        let dim = vector.components.len();
        let mut v = vector.components.clone();

        for step in 0..(path.points.len() - 1) {
            let p0 = &path.points[step];
            let p1 = &path.points[step + 1];

            // Path tangent (velocity along the geodesic)
            let gamma_dot: Vec<f64> = p0
                .coordinates
                .iter()
                .zip(p1.coordinates.iter())
                .map(|(a, b)| b - a)
                .collect();

            let christoffel = manifold.christoffel_at(p0);

            // RK4 step for parallel transport equation
            let dt = 1.0; // unit step between path points
            let k1 = transport_deriv(&christoffel, &v, &gamma_dot, dim);
            let v_mid1: Vec<f64> = v
                .iter()
                .zip(k1.iter())
                .map(|(vi, ki)| vi + 0.5 * dt * ki)
                .collect();
            let k2 = transport_deriv(&christoffel, &v_mid1, &gamma_dot, dim);
            let v_mid2: Vec<f64> = v
                .iter()
                .zip(k2.iter())
                .map(|(vi, ki)| vi + 0.5 * dt * ki)
                .collect();
            let k3 = transport_deriv(&christoffel, &v_mid2, &gamma_dot, dim);
            let v_end: Vec<f64> = v
                .iter()
                .zip(k3.iter())
                .map(|(vi, ki)| vi + dt * ki)
                .collect();
            let k4 = transport_deriv(&christoffel, &v_end, &gamma_dot, dim);

            for i in 0..dim {
                v[i] += dt * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0;
            }
        }

        TangentVector { components: v }
    }
}

/// Compute dv^k/dt = -Γ^k_{ij} v^i γ_dot^j
fn transport_deriv(
    christoffel: &ChristoffelSymbols,
    v: &[f64],
    gamma_dot: &[f64],
    dim: usize,
) -> Vec<f64> {
    let mut dv = vec![0.0; dim];
    for k in 0..dim {
        let mut acc = 0.0;
        for i in 0..dim {
            for j in 0..dim {
                acc += christoffel.get(k, i, j) * v[i] * gamma_dot[j];
            }
        }
        dv[k] = -acc;
    }
    dv
}
```

### Coordinate map

```rust
/// Maps protocol state variables to manifold coordinates and back.
/// Handles normalization so all coordinates live in [0, 1].
pub struct CoordinateMap {
    entries: Vec<CoordinateEntry>,
    /// Reverse lookup: coordinate index -> entry index
    index_to_entry: HashMap<usize, usize>,
}

pub struct CoordinateEntry {
    pub protocol: String,
    pub state_variable: String,
    pub coordinate_index: usize,
    /// Normalize raw protocol value to [0, 1].
    pub normalizer: Box<dyn Fn(f64) -> f64 + Send + Sync>,
    /// Denormalize from [0, 1] back to raw protocol value.
    pub denormalizer: Box<dyn Fn(f64) -> f64 + Send + Sync>,
}

impl CoordinateMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index_to_entry: HashMap::new(),
        }
    }

    pub fn dimension(&self) -> usize {
        self.entries.len()
    }

    pub fn add_entry(&mut self, entry: CoordinateEntry) {
        let idx = self.entries.len();
        self.index_to_entry.insert(entry.coordinate_index, idx);
        self.entries.push(entry);
    }

    /// Convert raw protocol states into a ManifoldPoint.
    pub fn to_manifold_point(&self, raw_states: &HashMap<String, f64>) -> ManifoldPoint {
        let dim = self.entries.len();
        let mut coords = vec![0.0; dim];
        for entry in &self.entries {
            let key = format!("{}:{}", entry.protocol, entry.state_variable);
            if let Some(&raw) = raw_states.get(&key) {
                coords[entry.coordinate_index] = (entry.normalizer)(raw);
            }
        }
        ManifoldPoint { coordinates: coords }
    }

    /// Convert a ManifoldPoint back to raw protocol states.
    pub fn from_manifold_point(&self, point: &ManifoldPoint) -> HashMap<String, f64> {
        let mut raw = HashMap::new();
        for entry in &self.entries {
            let key = format!("{}:{}", entry.protocol, entry.state_variable);
            let coord_val = point.coordinates[entry.coordinate_index];
            raw.insert(key, (entry.denormalizer)(coord_val));
        }
        raw
    }
}

/// Standard normalizers for common protocol state types.
pub mod normalizers {
    /// Normalize a ratio already in [0, 1] (e.g., utilization).
    pub fn ratio() -> (Box<dyn Fn(f64) -> f64 + Send + Sync>, Box<dyn Fn(f64) -> f64 + Send + Sync>) {
        (
            Box::new(|x| x.clamp(0.0, 1.0)),
            Box::new(|x| x.clamp(0.0, 1.0)),
        )
    }

    /// Normalize by a reference value. E.g., TVL / historical_max_tvl.
    pub fn by_reference(
        reference: f64,
    ) -> (Box<dyn Fn(f64) -> f64 + Send + Sync>, Box<dyn Fn(f64) -> f64 + Send + Sync>) {
        let r = reference;
        (
            Box::new(move |x| (x / r).clamp(0.0, 2.0) / 2.0),
            Box::new(move |x| x * 2.0 * r),
        )
    }

    /// Normalize gas price by a rolling range [min, max].
    pub fn range_normalize(
        min: f64,
        max: f64,
    ) -> (Box<dyn Fn(f64) -> f64 + Send + Sync>, Box<dyn Fn(f64) -> f64 + Send + Sync>) {
        let range = max - min;
        let mn = min;
        (
            Box::new(move |x| ((x - mn) / range).clamp(0.0, 1.0)),
            Box::new(move |x| x * range + mn),
        )
    }

    /// Log-normalize for values spanning orders of magnitude (e.g., TVL).
    pub fn log_normalize(
        min_log: f64,
        max_log: f64,
    ) -> (Box<dyn Fn(f64) -> f64 + Send + Sync>, Box<dyn Fn(f64) -> f64 + Send + Sync>) {
        let range = max_log - min_log;
        let ml = min_log;
        (
            Box::new(move |x| {
                let log_x = (x.max(1e-18)).ln();
                ((log_x - ml) / range).clamp(0.0, 1.0)
            }),
            Box::new(move |x| (x * range + ml).exp()),
        )
    }
}
```

### Metric estimation from execution data

```rust
/// Execution cost record from a realized or simulated operation.
#[derive(Clone, Debug)]
pub struct ExecutionCost {
    /// Manifold position before execution
    pub position: ManifoldPoint,
    /// State change vector (dx)
    pub state_delta: TangentVector,
    /// Decomposed costs
    pub slippage: f64,
    pub gas_cost: f64,
    pub time_cost: f64,
    pub opportunity_cost: f64,
    /// Timestamp (gamma tick number)
    pub tick: u64,
}

/// Estimates the metric tensor from accumulated execution cost data.
/// Uses weighted least squares to fit g_ij(x) from observed (dx, cost) pairs
/// in the neighborhood of each grid point.
pub struct MetricEstimator {
    /// Exponential decay for weighting older observations.
    decay_rate: f64,
    /// Spatial kernel bandwidth for weighting distant observations.
    kernel_bandwidth: f64,
    /// Weights for combining cost components.
    alpha: f64, // slippage
    beta: f64,  // gas
    gamma: f64, // time
    delta: f64, // opportunity
}

impl MetricEstimator {
    pub fn new(alpha: f64, beta: f64, gamma: f64, delta: f64) -> Self {
        let total = alpha + beta + gamma + delta;
        Self {
            decay_rate: 0.01,
            kernel_bandwidth: 0.1,
            alpha: alpha / total,
            beta: beta / total,
            gamma: gamma / total,
            delta: delta / total,
        }
    }

    /// Estimate the metric tensor at `point` from a buffer of recent
    /// execution cost observations.
    pub fn estimate(
        &self,
        point: &ManifoldPoint,
        observations: &[ExecutionCost],
        current_tick: u64,
    ) -> MetricTensor {
        let dim = point.dimension();
        let mut metric = MetricTensor::new(dim);

        if observations.is_empty() {
            return metric; // identity metric as fallback
        }

        // Weighted sum of outer products: g_ij ≈ Σ_n w_n * cost_n * dx_i * dx_j / |dx|^4
        // This follows from ds^2 = g_ij dx^i dx^j, rearranged.
        let mut numerator = vec![0.0_f64; dim * dim];
        let mut weight_sum = 0.0_f64;

        for obs in observations {
            // Temporal weight: exponential decay
            let age = (current_tick - obs.tick) as f64;
            let temporal_weight = (-self.decay_rate * age).exp();

            // Spatial weight: Gaussian kernel
            let dist = point.coord_distance(&obs.position);
            let spatial_weight = (-dist * dist / (2.0 * self.kernel_bandwidth * self.kernel_bandwidth)).exp();

            let w = temporal_weight * spatial_weight;
            if w < 1e-10 {
                continue;
            }

            // Total cost for this observation
            let total_cost = self.alpha * obs.slippage
                + self.beta * obs.gas_cost
                + self.gamma * obs.time_cost
                + self.delta * obs.opportunity_cost;

            let dx = &obs.state_delta.components;
            let dx_norm_sq: f64 = dx.iter().map(|d| d * d).sum();
            if dx_norm_sq < 1e-15 {
                continue;
            }

            // From ds^2 = g_ij dx^i dx^j and ds^2 ≈ total_cost^2,
            // each observation contributes to the metric estimate.
            let cost_sq = total_cost * total_cost;

            for i in 0..dim {
                for j in 0..dim {
                    numerator[i * dim + j] += w * cost_sq * dx[i] * dx[j] / (dx_norm_sq * dx_norm_sq);
                }
            }
            weight_sum += w;
        }

        if weight_sum > 0.0 {
            for i in 0..dim {
                for j in 0..dim {
                    let val = numerator[i * dim + j] / weight_sum;
                    metric.set(i, j, val);
                }
            }
        }

        // Regularize: add small identity component for positive definiteness
        let reg = 1e-6;
        for i in 0..dim {
            let current = metric.get(i, i);
            metric.set(i, i, current + reg);
        }

        metric
    }
}
```

### The metric field and manifold

```rust
/// Grid point identifier for discretized manifold sampling.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GridPoint {
    /// Quantized coordinate indices
    pub indices: Vec<i32>,
}

impl GridPoint {
    pub fn from_manifold_point(point: &ManifoldPoint, resolution: f64) -> Self {
        Self {
            indices: point
                .coordinates
                .iter()
                .map(|c| (c / resolution).round() as i32)
                .collect(),
        }
    }

    pub fn to_manifold_point(&self, resolution: f64) -> ManifoldPoint {
        ManifoldPoint {
            coordinates: self.indices.iter().map(|i| *i as f64 * resolution).collect(),
        }
    }
}

/// Sampled metric tensor field on the manifold.
pub struct MetricField {
    grid: HashMap<GridPoint, MetricTensor>,
    resolution: f64,
    default_dim: usize,
}

impl MetricField {
    pub fn new(dim: usize, resolution: f64) -> Self {
        Self {
            grid: HashMap::new(),
            resolution,
            default_dim: dim,
        }
    }

    pub fn insert(&mut self, point: &ManifoldPoint, metric: MetricTensor) {
        let gp = GridPoint::from_manifold_point(point, self.resolution);
        self.grid.insert(gp, metric);
    }

    /// Interpolate the metric at an arbitrary point using inverse-distance
    /// weighting from nearby grid points.
    pub fn interpolate(&self, point: &ManifoldPoint) -> MetricTensor {
        let gp = GridPoint::from_manifold_point(point, self.resolution);
        if let Some(metric) = self.grid.get(&gp) {
            return metric.clone();
        }

        // Find neighboring grid points and interpolate
        let dim = self.default_dim;
        let mut result = MetricTensor::new(dim);
        let mut total_weight = 0.0;
        let mut components = vec![0.0; dim * dim];

        for (grid_pt, metric) in &self.grid {
            let neighbor = grid_pt.to_manifold_point(self.resolution);
            let dist = point.coord_distance(&neighbor);
            if dist < self.resolution * 3.0 {
                let w = 1.0 / (dist + 1e-10);
                for i in 0..dim {
                    for j in 0..dim {
                        components[i * dim + j] += w * metric.get(i, j);
                    }
                }
                total_weight += w;
            }
        }

        if total_weight > 0.0 {
            for i in 0..dim {
                for j in 0..dim {
                    result.set(i, j, components[i * dim + j] / total_weight);
                }
            }
        }

        result
    }
}

/// Field of Christoffel symbols (cached from metric field).
pub struct ChristoffelField {
    grid: HashMap<GridPoint, ChristoffelSymbols>,
    resolution: f64,
    default_dim: usize,
}

impl ChristoffelField {
    pub fn new(dim: usize, resolution: f64) -> Self {
        Self {
            grid: HashMap::new(),
            resolution,
            default_dim: dim,
        }
    }

    pub fn insert(&mut self, point: &ManifoldPoint, symbols: ChristoffelSymbols) {
        let gp = GridPoint::from_manifold_point(point, self.resolution);
        self.grid.insert(gp, symbols);
    }

    /// Interpolate Christoffel symbols at an arbitrary point.
    pub fn interpolate(&self, point: &ManifoldPoint) -> ChristoffelSymbols {
        let gp = GridPoint::from_manifold_point(point, self.resolution);
        if let Some(symbols) = self.grid.get(&gp) {
            return symbols.clone();
        }

        let dim = self.default_dim;
        let size = dim * dim * dim;
        let mut components = vec![0.0; size];
        let mut total_weight = 0.0;

        for (grid_pt, symbols) in &self.grid {
            let neighbor = grid_pt.to_manifold_point(self.resolution);
            let dist = point.coord_distance(&neighbor);
            if dist < self.resolution * 3.0 {
                let w = 1.0 / (dist + 1e-10);
                for idx in 0..size {
                    components[idx] += w * symbols.components[idx];
                }
                total_weight += w;
            }
        }

        if total_weight > 0.0 {
            for idx in 0..size {
                components[idx] /= total_weight;
            }
        }

        ChristoffelSymbols {
            components,
            dimension: dim,
        }
    }
}
```

### The liquidity manifold

```rust
/// The complete liquidity manifold.
/// Holds the metric field, Christoffel cache, coordinate map,
/// metric estimator, and the Golem's current position.
pub struct LiquidityManifold {
    dimension: usize,
    metric_field: MetricField,
    christoffel_field: ChristoffelField,
    coordinate_map: CoordinateMap,
    metric_estimator: MetricEstimator,
    current_position: ManifoldPoint,
    execution_buffer: Vec<ExecutionCost>,
    /// Maximum execution records to retain
    buffer_capacity: usize,
    /// Current gamma tick
    current_tick: u64,
    /// Grid resolution for metric sampling
    resolution: f64,
    /// Finite difference step for derivative estimation
    deriv_step: f64,
}

impl LiquidityManifold {
    pub fn new(
        coordinate_map: CoordinateMap,
        estimator: MetricEstimator,
        resolution: f64,
    ) -> Self {
        let dim = coordinate_map.dimension();
        Self {
            dimension: dim,
            metric_field: MetricField::new(dim, resolution),
            christoffel_field: ChristoffelField::new(dim, resolution),
            coordinate_map,
            metric_estimator: estimator,
            current_position: ManifoldPoint::new(dim),
            execution_buffer: Vec::new(),
            buffer_capacity: 10_000,
            current_tick: 0,
            resolution,
            deriv_step: resolution * 0.5,
        }
    }

    /// Called at each gamma tick with the latest protocol state.
    /// Updates the Golem's position and the local metric tensor.
    pub fn gamma_tick(
        &mut self,
        raw_states: &HashMap<String, f64>,
        tick: u64,
    ) -> ManifoldTickResult {
        self.current_tick = tick;
        self.current_position = self.coordinate_map.to_manifold_point(raw_states);

        // Re-estimate metric at current position from execution buffer
        let metric = self.metric_estimator.estimate(
            &self.current_position,
            &self.execution_buffer,
            tick,
        );
        self.metric_field.insert(&self.current_position, metric.clone());

        // Compute curvature (Ricci scalar) at current position
        let ricci = self.ricci_scalar_at(&self.current_position);
        let condition = metric.condition_number();

        ManifoldTickResult {
            ricci_scalar: ricci.unwrap_or(0.0),
            condition_number: condition,
            position: self.current_position.clone(),
        }
    }

    /// Record an execution cost observation for metric estimation.
    pub fn record_execution(&mut self, cost: ExecutionCost) {
        self.execution_buffer.push(cost);
        if self.execution_buffer.len() > self.buffer_capacity {
            // Remove oldest entries
            let drain_count = self.buffer_capacity / 10;
            self.execution_buffer.drain(0..drain_count);
        }
    }

    /// Get the metric tensor at a point (interpolated from field).
    pub fn metric_at(&self, point: &ManifoldPoint) -> MetricTensor {
        self.metric_field.interpolate(point)
    }

    /// Get Christoffel symbols at a point (interpolated from field).
    pub fn christoffel_at(&self, point: &ManifoldPoint) -> ChristoffelSymbols {
        self.christoffel_field.interpolate(point)
    }

    /// Compute Ricci scalar at a point.
    pub fn ricci_scalar_at(&self, point: &ManifoldPoint) -> Option<f64> {
        let metric = self.metric_field.interpolate(point);
        let derivatives = MetricDerivatives::from_metric_field(
            &self.metric_field,
            point,
            self.deriv_step,
        );
        let christoffel = ChristoffelSymbols::from_metric(&metric, &derivatives)?;
        let christoffel_derivs = ChristoffelDerivatives::from_christoffel_field(
            &self.christoffel_field,
            point,
            self.deriv_step,
        );
        CurvatureComputer::ricci_scalar(&metric, &christoffel, &christoffel_derivs)
    }

    /// Compute geodesic between two points.
    pub fn geodesic(
        &self,
        from: &ManifoldPoint,
        to: &ManifoldPoint,
    ) -> Option<GeodesicPath> {
        let solver = GeodesicSolver::new();
        solver.solve(self, from, to)
    }

    /// Parallel transport a pattern vector along a geodesic.
    pub fn parallel_transport(
        &self,
        vector: &TangentVector,
        along: &GeodesicPath,
    ) -> TangentVector {
        ParallelTransporter::transport(self, vector, along)
    }

    /// Exponential map: follow geodesic from point in direction v for unit time.
    pub fn exp_map(
        &self,
        point: &ManifoldPoint,
        v: &TangentVector,
    ) -> ManifoldPoint {
        let solver = GeodesicSolver::new();
        let path = solver.integrate_geodesic(self, point, v);
        path.points.last().cloned().unwrap_or_else(|| point.clone())
    }

    /// Logarithmic map: compute initial velocity of geodesic from p to q.
    pub fn log_map(
        &self,
        p: &ManifoldPoint,
        q: &ManifoldPoint,
    ) -> Option<TangentVector> {
        let path = self.geodesic(p, q)?;
        if path.points.len() < 2 {
            return None;
        }
        let p0 = &path.points[0];
        let p1 = &path.points[1];
        let n = path.num_steps as f64;
        Some(TangentVector {
            components: p0
                .coordinates
                .iter()
                .zip(p1.coordinates.iter())
                .map(|(a, b)| (b - a) * n)
                .collect(),
        })
    }

    /// Delta tick: full manifold maintenance.
    /// Recompute metric field and Christoffel cache across all active grid points.
    pub fn delta_tick(&mut self) {
        // Collect all grid points that have been visited recently
        let grid_points: Vec<GridPoint> = self.metric_field.grid.keys().cloned().collect();

        for gp in &grid_points {
            let point = gp.to_manifold_point(self.resolution);
            let metric = self.metric_estimator.estimate(
                &point,
                &self.execution_buffer,
                self.current_tick,
            );

            let derivatives = MetricDerivatives::from_metric_field(
                &self.metric_field,
                &point,
                self.deriv_step,
            );

            if let Some(christoffel) = ChristoffelSymbols::from_metric(&metric, &derivatives) {
                self.christoffel_field.insert(&point, christoffel);
            }

            self.metric_field.insert(&point, metric);
        }
    }

    /// Dream: imagine a manifold deformation and precompute geodesics.
    /// `perturbation` maps coordinate indices to scale factors.
    /// E.g., {3: 0.5} means "halve the value at coordinate 3."
    pub fn dream_deformation(
        &self,
        perturbation: &HashMap<usize, f64>,
        targets: &[ManifoldPoint],
    ) -> Vec<Option<GeodesicPath>> {
        // Create a perturbed manifold by scaling metric components
        // corresponding to the perturbed coordinates.
        // This is an approximation: we scale the diagonal of the metric
        // to simulate changed liquidity conditions.
        let mut dreamed = self.clone_with_perturbation(perturbation);
        dreamed.delta_tick();

        targets
            .iter()
            .map(|target| dreamed.geodesic(&self.current_position, target))
            .collect()
    }

    fn clone_with_perturbation(
        &self,
        perturbation: &HashMap<usize, f64>,
    ) -> LiquidityManifold {
        let mut cloned = LiquidityManifold::new(
            CoordinateMap::new(), // simplified; production would clone the map
            MetricEstimator::new(0.4, 0.3, 0.1, 0.2),
            self.resolution,
        );
        cloned.dimension = self.dimension;

        // Copy and perturb the metric field
        for (gp, metric) in &self.metric_field.grid {
            let mut perturbed = metric.clone();
            for (&coord, &scale) in perturbation {
                // Scale the diagonal element for the perturbed coordinate.
                // A lower scale factor means "less liquidity" which means
                // higher cost (inverse relationship).
                let current = perturbed.get(coord, coord);
                perturbed.set(coord, coord, current / scale.max(0.01));
            }
            cloned.metric_field.grid.insert(gp.clone(), perturbed);
        }

        cloned
    }
}

pub struct ManifoldTickResult {
    pub ricci_scalar: f64,
    pub condition_number: f64,
    pub position: ManifoldPoint,
}
```

### Frechet mean computation

```rust
/// Compute the Frechet mean of a set of manifold points.
/// Uses Riemannian gradient descent: iteratively project points to
/// tangent space via log map, average, and project back via exp map.
pub struct FrechetMean;

impl FrechetMean {
    pub fn compute(
        manifold: &LiquidityManifold,
        points: &[ManifoldPoint],
        max_iterations: usize,
    ) -> Option<ManifoldPoint> {
        if points.is_empty() {
            return None;
        }
        if points.len() == 1 {
            return Some(points[0].clone());
        }

        let dim = points[0].dimension();
        // Initialize with the first point (or could use Euclidean centroid)
        let mut mean = points[0].clone();

        for _iter in 0..max_iterations {
            // Project all points to tangent space at current mean
            let mut tangent_sum = TangentVector::new(dim);
            let mut count = 0;

            for p in points {
                if let Some(log_v) = manifold.log_map(&mean, p) {
                    tangent_sum = tangent_sum.add(&log_v);
                    count += 1;
                }
            }

            if count == 0 {
                break;
            }

            // Average tangent vector
            let step = tangent_sum.scale(1.0 / count as f64);

            // Check convergence: if the average tangent vector is small,
            // the mean is stable.
            if step.norm_squared() < 1e-12 {
                break;
            }

            // Gradient descent step: move mean in the direction of the
            // average tangent vector (with step size < 1 for stability)
            let dampened = step.scale(0.5);
            mean = manifold.exp_map(&mean, &dampened);
        }

        Some(mean)
    }
}
```

### Curvature field snapshot

```rust
/// A snapshot of the curvature field across the manifold,
/// computed during delta ticks for spatial curvature analysis.
#[derive(Clone, Debug)]
pub struct CurvatureField {
    /// Ricci scalar at each sampled grid point
    pub scalars: HashMap<GridPoint, f64>,
    /// Sectional curvatures for tracked protocol pairs
    pub sectional: HashMap<(usize, usize), f64>,
    /// Tick at which this snapshot was computed
    pub computed_at: u64,
}

impl CurvatureField {
    /// Compute the full curvature field during a delta tick.
    pub fn compute(manifold: &LiquidityManifold) -> Self {
        let mut scalars = HashMap::new();
        let mut sectional = HashMap::new();

        for (gp, _metric) in &manifold.metric_field.grid {
            let point = gp.to_manifold_point(manifold.resolution);
            if let Some(r) = manifold.ricci_scalar_at(&point) {
                scalars.insert(gp.clone(), r);
            }
        }

        // Compute sectional curvatures for interesting protocol pairs.
        // In production, the list of pairs comes from the Golem's
        // current holdings and strategy.
        let dim = manifold.dimension;
        let current = &manifold.current_position;
        let metric = manifold.metric_at(current);
        let derivatives = MetricDerivatives::from_metric_field(
            &manifold.metric_field,
            current,
            manifold.deriv_step,
        );

        if let Some(christoffel) = ChristoffelSymbols::from_metric(&metric, &derivatives) {
            let christoffel_derivs = ChristoffelDerivatives::from_christoffel_field(
                &manifold.christoffel_field,
                current,
                manifold.deriv_step,
            );

            // Check all pairs of "active" coordinates
            // (those with non-trivial metric values)
            for i in 0..dim {
                for j in (i + 1)..dim {
                    if metric.get(i, i) > 1e-6 && metric.get(j, j) > 1e-6 {
                        let mut u = TangentVector::new(dim);
                        let mut v = TangentVector::new(dim);
                        u.components[i] = 1.0;
                        v.components[j] = 1.0;

                        if let Some(k) = CurvatureComputer::sectional_curvature(
                            &metric,
                            &christoffel,
                            &christoffel_derivs,
                            &u,
                            &v,
                        ) {
                            if k.abs() > 1e-8 {
                                sectional.insert((i, j), k);
                            }
                        }
                    }
                }
            }
        }

        CurvatureField {
            scalars,
            sectional,
            computed_at: manifold.current_tick,
        }
    }

    /// Find the grid point with the most negative curvature.
    /// This is the most unstable region of the manifold.
    pub fn most_unstable(&self) -> Option<(&GridPoint, f64)> {
        self.scalars
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(gp, &r)| (gp, r))
    }

    /// Find protocol pairs with the most negative sectional curvature.
    /// These are the protocol interactions most likely to produce
    /// non-linear cost amplification.
    pub fn unstable_pairs(&self, threshold: f64) -> Vec<((usize, usize), f64)> {
        let mut pairs: Vec<_> = self
            .sectional
            .iter()
            .filter(|(_, &k)| k < threshold)
            .map(|(&pair, &k)| (pair, k))
            .collect();
        pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        pairs
    }
}
```

### CorticalState writer

```rust
/// Writes manifold-derived signals to the Golem's CorticalState.
pub struct ManifoldCorticalWriter {
    pub liquidity_curvature: AtomicU32,
    pub geodesic_cost: AtomicU32,
    pub manifold_stability: AtomicU32,
}

impl ManifoldCorticalWriter {
    pub fn new() -> Self {
        Self {
            liquidity_curvature: AtomicU32::new(0),
            geodesic_cost: AtomicU32::new(0),
            manifold_stability: AtomicU32::new(0),
        }
    }

    pub fn update(&self, tick_result: &ManifoldTickResult, geodesic_cost: f64) {
        self.liquidity_curvature.store(
            (tick_result.ricci_scalar as f32).to_bits(),
            Ordering::Relaxed,
        );
        self.geodesic_cost.store(
            (geodesic_cost as f32).to_bits(),
            Ordering::Relaxed,
        );
        self.manifold_stability.store(
            (tick_result.condition_number as f32).to_bits(),
            Ordering::Relaxed,
        );
    }

    pub fn read_curvature(&self) -> f32 {
        f32::from_bits(self.liquidity_curvature.load(Ordering::Relaxed))
    }

    pub fn read_geodesic_cost(&self) -> f32 {
        f32::from_bits(self.geodesic_cost.load(Ordering::Relaxed))
    }

    pub fn read_stability(&self) -> f32 {
        f32::from_bits(self.manifold_stability.load(Ordering::Relaxed))
    }
}
```

## Subsystem interactions [SPEC]

### CorticalState

Three new atomic signals join the perception surface: `liquidity_curvature`, `geodesic_cost`, and `manifold_stability`. Other subsystems read these without locks. The attention system uses `liquidity_curvature` sign changes to trigger high-priority analysis. The risk daemon reads `manifold_stability` and tightens execution constraints when the condition number exceeds a threshold (anisotropic cost structures increase slippage risk).

### Mortality

The Golem's mortality engine tracks `economic_vitality`, a scalar representing the agent's overall financial health. The manifold provides a geometric interpretation: economic vitality correlates with the Golem's position relative to high-liquidity, low-curvature regions of the manifold. A Golem far from liquidity (high geodesic cost to reach executable states) is economically fragile. A Golem in a high-negative-curvature region is in an unstable cost environment where small errors compound. Both conditions should lower vitality.

The `geodesic_cost` signal feeds directly into the vitality calculation. If the cheapest path to any identified opportunity costs more than the expected return, the Golem is in a dead zone. Mortality should rise.

### Dreams (REM consolidation)

During delta-tick dream cycles, the manifold system generates counterfactual deformations. The dream engine picks perturbation scenarios based on current risk factors: if the Golem holds large LP positions on Uniswap, one dream scenario halves Uniswap liquidity. If the Golem has lending exposure on Aave, another scenario pushes utilization to 95%.

For each dreamed manifold, the system pre-computes geodesics from the current position to all target states in the Golem's opportunity set. These precomputed paths are stored in a geodesic atlas indexed by perturbation type. When the real manifold shifts toward a dreamed state, the Golem can look up the pre-computed path instead of computing a geodesic from scratch, cutting reaction time from hundreds of milliseconds to a lookup.

### Styx (multi-agent coordination)

Each Golem samples execution costs in its own neighborhood of the manifold. One Golem trading ETH/USDC on Uniswap V3 knows the metric tensor well in that region but has no data for Curve or Balancer. Another Golem focused on Curve stablecoins has the opposite view.

Through Styx (the inter-Golem communication layer), Golems share local manifold patches. Each Golem broadcasts its recently estimated metric tensors tagged with grid coordinates. Other Golems in the clade incorporate these patches into their own metric field, weighted by the reporting Golem's credibility score (based on historical execution accuracy).

The clade assembles a more complete manifold than any individual Golem could build alone. Cross-protocol geodesics become possible only when multiple Golems contribute metric data from their respective protocol neighborhoods.

### Mirage-rs (fork simulation)

Geodesic predictions are hypotheses about optimal execution paths. Mirage-rs validates them. Before executing a geodesic-recommended sequence of operations, the Golem simulates the full sequence on a forked state via Mirage-rs and compares the simulated cost to the geodesic-predicted cost.

If the simulation cost matches the prediction (within tolerance), the geodesic is reliable and the metric tensor is well-calibrated in that region. If the simulation cost diverges, the metric tensor needs recalibration: the execution cost observation from simulation feeds back into the metric estimator.

This creates a validation loop: manifold predicts cost, simulation measures cost, discrepancy updates the manifold.

## DeFi primitive coverage

Every DeFi primitive type has coordinates on the manifold. The coordinate assignments below are concrete; a production system would discover them automatically from protocol ABIs and state inspection.

### AMM pools

Coordinates: reserve ratio ($x_0 / x_1$ normalized by initial ratio), concentrated liquidity active tick density (fraction of ticks with non-zero liquidity in the active range), fee accrual rate (fees earned per unit liquidity per block, normalized by 30-day average).

The metric tensor in AMM coordinates encodes slippage as a function of reserve imbalance. Near the center of the pool (balanced reserves), the metric is nearly flat: small swaps cost little. Near the edges (depleted reserves on one side), the metric diverges: the cost of further depletion is very high, while the cost of rebalancing toward center is moderate. This asymmetry appears in the skew tensor $a_{ij}$.

For a Uniswap V3 concentrated liquidity pool, the metric in the reserve-ratio direction has a shape determined by the tick distribution. Dense liquidity around the current price produces a flat metric (low slippage per unit of reserve change). Sparse liquidity produces a steep metric. The tick density coordinate interacts with the reserve ratio: removing liquidity (decreasing tick density) while the reserve ratio is already imbalanced is much more expensive than removing liquidity from a balanced pool, because you are widening the slippage surface in a region that is already steep. This cross-term is where the manifold approach adds value over per-coordinate analysis.

A concrete example: the 2x2 metric submatrix for a Uniswap V3 pool might look like:

$$g_{\text{AMM}} = \begin{pmatrix} \frac{k}{L^2(1-|r|)^2} & \frac{kr}{L^2(1-|r|)} \\ \frac{kr}{L^2(1-|r|)} & \frac{c}{d^2} \end{pmatrix}$$

where $r$ is the reserve imbalance ratio, $L$ is total liquidity, $k$ is a slippage constant derived from the bonding curve, $c$ is the gas cost of LP operations, and $d$ is the tick density. The $(1-|r|)^2$ term in the upper left makes the metric diverge as the pool approaches full depletion on one side, which is exactly what happens to slippage costs. The off-diagonal term couples reserve changes with liquidity changes through the imbalance ratio.

### Lending protocols

Coordinates: utilization ratio (borrows / total deposits), supply rate, borrow rate, aggregate collateral factor (weighted average of collateral types).

The metric has a distinctive shape near utilization kink points. Aave and Compound use piecewise-linear interest rate curves with a kink at ~80% utilization where rates jump sharply. On the manifold, this kink appears as a region of extremely high curvature: the cost of pushing utilization past the kink is discontinuously higher than the cost of staying below it. The curvature signal at the kink is a strong warning of impending rate spikes.

The metric in the utilization direction is approximately:

$$g_{uu} \approx \begin{cases} a + bu & \text{if } u < u_{\text{kink}} \\ a + bu_{\text{kink}} + c(u - u_{\text{kink}})^2 & \text{if } u \geq u_{\text{kink}} \end{cases}$$

where $u$ is utilization, $u_{\text{kink}} \approx 0.8$ for most Aave markets, and $c \gg b$ reflects the rate curve's steep slope above the kink. The quadratic term above the kink creates positive curvature (a cost basin that penalizes upward deviation), which is why utilization tends to stay near or below the kink in practice. But when external demand pushes utilization past the kink, the Ricci scalar in the lending subspace goes negative because the system transitions from a single basin to a saddle: the cost of further increases is extreme, but the cost of decreases (repayment) depends on borrowers' willingness to repay, which is not guaranteed. The geometry of the cost surface becomes asymmetric in a way that the symmetrized metric cannot fully capture, and the skew tensor norm spikes, which is itself a signal.

The collateral factor coordinate interacts with utilization through liquidation risk. As utilization rises and collateral factors tighten, the off-diagonal metric term $g_{u,\text{cf}}$ grows: changing utilization while the collateral factor is shifting costs more than either change in isolation, because the combined movement can trigger liquidation cascades that produce discontinuous cost jumps.

### Vaults

Coordinates: TVL (log-normalized), share price ratio (share price / underlying value), withdrawal queue depth (number of pending withdrawals normalized by daily capacity).

Vault coordinates interact with lending coordinates through a common underlying asset. When a vault deposits into a lending protocol, the vault TVL coordinate and the lending supply coordinate are coupled. The off-diagonal metric terms between vault and lending coordinates capture this coupling.

### Staking and restaking

Coordinates: total staked (fraction of circulating supply), delegation concentration (Herfindahl index of validator/operator shares), slashing event recency (exponentially decayed count of recent slashing events).

For restaking protocols like EigenLayer, additional coordinates track AVS (Actively Validated Service) allocations and operator performance scores. The manifold captures the cost of reallocating stake between AVSes, which depends on unbonding periods (time cost), operator reliability (opportunity cost), and AVS demand (slippage in allocation markets).

### Derivatives

Coordinates: open interest (normalized by underlying market cap), funding rate, implied volatility (at-the-money, 30-day), delta exposure.

The derivatives submanifold has particularly rich curvature structure because derivatives are non-linear functions of their underlyings. Implied volatility surfaces create complex geometry: the metric tensor near volatility smile extremes (deep OTM puts, deep OTM calls) differs substantially from the at-the-money region.

### Yield protocols

Coordinates: implied yield rate, time to maturity (normalized), liquidity depth at current yield level.

Yield protocol coordinates are naturally temporal: as maturity approaches, the geometry of the yield submanifold deforms. The metric tensor in the time-to-maturity direction increases as maturity nears (adjustments become more expensive), creating a funnel-shaped geometry.

### Cross-chain

Coordinates: bridge liquidity per destination chain (log-normalized), message latency (block confirmations, normalized), bridge fee rate.

Cross-chain coordinates introduce the time cost component $T_{ij}$ most strongly. A geodesic that routes through a bridge must account for the settlement delay, which is not a financial cost but a real cost in lost opportunity. The metric tensor's time component makes bridges "expensive" in a way that pure financial cost ignores, correctly penalizing routes that require waiting.

The bridge liquidity coordinate has an asymmetry that is particularly pronounced. Bridging from Ethereum to Arbitrum when Arbitrum-side bridge liquidity is deep costs much less than bridging from Arbitrum to Ethereum when Ethereum-side liquidity is thin. The skew tensor for bridge coordinates is often larger in magnitude than the symmetric part, meaning the Finsler correction matters more here than in any other protocol type. A practical compromise: use the full Finsler metric for bridge dimensions and the Riemannian approximation for everything else, since bridge operations are discrete events that don't require continuous geodesic computation.

Cross-chain geodesics are inherently multi-step: bridge, wait for finality, execute on destination chain. The waiting step is represented as a segment of the geodesic where the spatial coordinates are constant but time advances. On the manifold, this looks like a geodesic that "pauses" in spatial dimensions while accumulating cost in the time dimension. The metric tensor during the pause has large $T_{ii}$ components (time is passing) and zero $S_{ij}$ components (no slippage, because nothing is being traded). The geodesic equation naturally handles this: the Christoffel symbols during the pause deflect the path away from further time-costly operations, biasing the post-bridge execution toward fast, gas-efficient routes.

### Cross-primitive navigation

The core use case: "What is the cheapest way to move from an ETH LP position on Uniswap V3 to a stablecoin lending position on Aave?"

This path crosses multiple protocol subspaces on the manifold. The geodesic might route through: withdraw LP (Uniswap coordinates), swap ETH for USDC (AMM coordinates, possibly routing through a different pool for lower slippage), deposit USDC (Aave lending coordinates). The metric tensor at each point along the path determines the optimal sizing and timing of each step.

A naive execution would perform these steps sequentially, each optimized in isolation. The geodesic optimizes them jointly. It might find that withdrawing the LP in two steps (partial withdrawal, swap, partial withdrawal, swap) costs less than one large withdrawal because the first withdrawal moves the pool's reserves, making the second withdrawal cheaper. The Christoffel symbols encode exactly this kind of inter-step cost coupling.

## Cybernetic feedback loop

The manifold is not a static structure. It learns.

**Navigate.** The Golem computes a geodesic from its current position to a target state using the current metric tensor.

**Execute.** The Golem follows the geodesic, executing operations step by step. At each step, it records the actual execution cost (slippage, gas, time, opportunity).

**Measure.** The actual cost is compared to the geodesic-predicted cost. The discrepancy is the metric error at that point.

**Update.** The execution cost observation feeds into the metric estimator. The metric tensor at the executed point is re-estimated, pulling it toward the observed cost structure. The Christoffel symbols at that point are invalidated and recomputed at the next delta tick.

**Navigate better.** The next geodesic computation uses the updated metric. If the previous prediction was too optimistic (underestimated cost), the updated metric will route around that region. If it was too pessimistic, the metric relaxes and allows cheaper paths.

This is a cybernetic loop in Wiener's sense: the system's output (execution cost) feeds back to modify its internal model (metric tensor), which modifies its future output (geodesic routing). The manifold geometry self-corrects from experience.

The convergence properties depend on the metric estimator's learning rate (controlled by `decay_rate` and `kernel_bandwidth`). Too fast, and the metric oscillates with each new observation. Too slow, and the manifold fails to track market changes. The delta-tick maintenance step provides a regularization checkpoint: at each delta tick, the full metric field is re-estimated from the complete execution buffer, smoothing out per-tick noise.

Over many execution cycles, the metric tensor converges to an accurate representation of the true cost structure in regions where the Golem actively operates. Regions it hasn't visited remain at the identity metric (flat geometry), which is conservative: it assumes uniform costs and does not claim knowledge it doesn't have. When a Golem receives manifold patches from other Golems via Styx, unvisited regions fill in, expanding the Golem's navigable manifold without requiring it to explore everywhere itself.

### The geodesic atlas

Pre-computed geodesics for common routes are stored in a geodesic atlas, indexed by (start region, end region) pairs. The atlas is populated during delta ticks and dream cycles. Each entry stores the discretized path, the total cost, the metric tensor condition number along the path (indicating reliability), and the tick at which it was computed (for staleness detection).

At theta tick time, when the strategy layer requests an execution path, the manifold first checks the atlas for a cached geodesic. If one exists and its staleness (current tick minus computation tick) is below a threshold, the cached geodesic is used directly, saving the 200ms shooting-method computation. If no cached geodesic exists, or the cached one is stale, a fresh geodesic is computed and the atlas is updated.

The atlas also serves the dream system. During REM, the Golem generates hypothetical manifold deformations and computes geodesics on the deformed manifolds. These dream-geodesics are stored in a separate "dream atlas" keyed by (deformation type, start region, end region). When real market conditions shift toward a dreamed scenario, the Golem consults the dream atlas. The dream-geodesic won't be exact (the actual deformation will differ from the imagined one), but it provides a far better initial guess for the shooting method than a straight-line initialization, reducing convergence time from ~20 iterations to ~5.

The atlas has a capacity limit. When full, it evicts the least-recently-used entries. High-value routes (those the Golem executes frequently, or those with high curvature indicating instability) are pinned and not evicted. The atlas size is a tunable parameter; a reasonable default is 1000 entries, occupying roughly 10 MB for an 80-dimensional manifold with 100-step paths.

## Evaluation protocol [SPEC]

### Geodesic accuracy

**Claim:** Geodesic-predicted execution costs match actual execution costs within 15% after 1000 execution cycles in a stable market, and within 25% in volatile markets.

**Test protocol:** Run a Golem in simulation (Mirage-rs fork mode) on 6 months of historical mainnet data. At each theta tick, the Golem proposes a target state and computes a geodesic. It then executes the geodesic and records the predicted vs. actual cost. After a warm-up period of 1000 gamma ticks (to populate the execution buffer), compute the mean and median prediction error.

**Primary metric:** Mean absolute percentage error (MAPE) of predicted vs. actual execution cost.

**Baseline:** Compare against the current approach of simulating 5 candidate paths via Mirage-rs and selecting the cheapest. The geodesic approach should produce lower-cost paths on average because it searches the full continuous path space rather than a discrete set of candidates.

### Curvature as leading indicator

**Claim:** Negative Ricci scalar predicts cost instability (defined as >50% change in execution costs within the next 100 gamma ticks) with a lead time of 10-50 ticks and a false positive rate below 5%.

**Test protocol:** On historical data, compute the Ricci scalar at each gamma tick. Identify ground-truth cost instability events (periods where execution costs changed by >50% within 100 ticks). For each event, record the tick at which the Ricci scalar first went negative. Compute lead time and false positive rate.

**Failure mode:** If curvature does not lead cost instability by a statistically significant margin, the curvature computation is overhead without value and should be removed. The metric tensor and geodesics are still useful even if curvature is not predictive.

### Parallel transport accuracy

**Claim:** Patterns transported from one protocol region to another via parallel transport produce lower execution costs than patterns re-learned from scratch in the target region, for the first 200 ticks after transport. After 200 ticks, re-learned patterns should converge to transported patterns.

**Test protocol:** Train a Golem on Uniswap V3 LP management for 5000 ticks. Record its best LP rebalancing pattern (encoded as a tangent vector). Transport the pattern to a Curve pool. Simultaneously, train a second Golem on the Curve pool from scratch. Compare execution costs for the first 500 ticks.

**Baseline:** The transported-pattern Golem should outperform the from-scratch Golem in ticks 1-200 (geometric transfer advantage) and converge to similar performance by tick 500 (both have enough local data).

### Computational budget

The manifold computation must fit within the heartbeat's timing constraints.

Gamma tick operations (metric update + curvature at one point): target <10ms. For an 80-dimensional manifold, this is an 80x80 matrix estimation, one eigenvalue computation, and one Ricci scalar computation. The matrix operations are O(N^2) to O(N^3); at N=80 this is ~500K floating-point operations, well within budget.

Theta tick operations (one geodesic computation): target <200ms. The shooting method requires ~20 geodesic integrations (iterations), each with ~100 RK4 steps, each step requiring O(N^2) Christoffel symbol lookups. Total: ~20 * 100 * 80^2 = ~13M operations. Fits within budget at modern CPU speeds.

Delta tick operations (full field recomputation): target <5 seconds. Recomputing metrics and Christoffel symbols at ~500 grid points, each requiring O(N^3) for Cholesky decomposition. Total: ~500 * 80^3 = ~256M operations. Comfortable within a 5-second budget, especially since delta ticks happen during dream consolidation when the Golem is not time-constrained.

## Discussion

The manifold formulation has an opinionated stance on a question that DEX aggregators and execution algorithms have been circling: is optimal execution a graph search problem or a continuous optimization problem?

Current DEX aggregators treat it as graph search. Enumerate routes (A -> B, A -> C -> B, A -> D -> C -> B), simulate each, pick the cheapest. This works for single-step swaps. It breaks down for multi-protocol, multi-step execution because the graph explodes combinatorially, and the cost of each edge depends on all previous edges (path-dependent costs from state changes).

The manifold formulation treats it as continuous optimization. The geodesic equation finds the optimal path through a continuous cost field, accounting for path dependence through the Christoffel symbols (which encode how costs change as you move). The solution is a smooth curve, not a discrete sequence of hops. Discretizing this curve into executable operations is straightforward; the hard part (finding the optimal direction and timing at each point) is handled by the geometry.

The weakness of the manifold approach is that it requires a well-calibrated metric tensor, which requires execution data, which requires the Golem to have already traded in that region. The cold-start problem is real: a Golem entering a new protocol has identity metric (no geometric knowledge) and falls back to brute-force simulation until it accumulates enough data. Styx mitigates this by allowing Golems to share metric patches, but the fundamental dependency on empirical data remains.

The Finsler generalization (for direction-dependent costs) is theoretically cleaner than our symmetrize-and-store-skew approach. A full Finsler implementation would compute geodesics in a direction-dependent metric, which is more accurate for the buy/sell asymmetry that pervades DeFi. We chose the Riemannian approximation for tractability: Riemannian geodesics are standard second-order ODEs, while Finsler geodesics involve a spray geometry that complicates the numerics. If the symmetrization error (estimated from the skew tensor norm) exceeds 20% of the metric magnitude, upgrading to Finsler geometry would be warranted.

The relationship to information geometry (Amari, 1985) is worth noting. Information geometry treats the space of probability distributions as a Riemannian manifold with the Fisher information metric. The DeFi liquidity manifold is not an information manifold (the metric comes from execution costs, not statistical divergences), but the mathematical machinery is identical. Patterns and algorithms from information geometry (natural gradient descent, geodesic optimization, alpha-connections) may transfer directly. We have not explored this connection in depth, but it is a promising direction.

There is a deeper question about whether the metric should be learned or derived from first principles. The current approach is empirical: the metric estimator fits $g_{ij}$ from observed execution costs. An alternative is to derive the metric analytically from protocol mechanics. For a constant-product AMM, the slippage cost is a known function of reserves and trade size; the corresponding metric component has a closed-form expression. For lending protocols with known interest rate curves, the metric in the utilization direction can be derived from the rate function's curvature.

The analytical approach has the advantage of zero warm-up time (no execution data needed) and exact accuracy for the protocols it covers. The empirical approach handles protocol interactions, unknown protocols, and real-world effects (MEV, gas auction dynamics, front-running) that no analytical model captures. A hybrid is likely optimal: use analytical metrics for well-understood protocol dimensions (AMM slippage, lending rate curves) and empirical estimation for cross-protocol interactions and auxiliary dimensions. The implementation supports this: the metric estimator accepts a "prior" metric that is overridden by data as it accumulates. The analytical metric is the prior; the empirical estimate is the posterior.

The dimensionality of the manifold (N = 50 to 500) raises practical concerns about the O(N^3) cost of matrix operations (inversion, eigenvalue decomposition). For N = 500, Cholesky decomposition alone costs ~42 million operations per point. Two mitigations apply. First, the metric tensor is sparse in practice: most protocol coordinates do not interact with most other protocol coordinates. A Uniswap pool and an Aave lending market on different assets have near-zero off-diagonal terms. Sparse matrix algorithms reduce the effective dimensionality. Second, the manifold decomposes into approximately independent submanifolds (one per protocol cluster), connected by weak interactions (cross-protocol cost coupling). Block-diagonal approximation of the metric tensor reduces each N^3 operation to a sum of smaller blocks, providing significant speedup with bounded approximation error.

The connection to optimal transport theory is also relevant. Optimal transport asks: given two distributions $\mu$ and $\nu$ on a space, what is the cheapest way to move mass from $\mu$ to $\nu$? The Wasserstein distance is the cost of the optimal transport plan. On the liquidity manifold, the Golem's portfolio is a distribution of capital across protocols, and rebalancing is a transport problem. The geodesic from one portfolio state to another is closely related to the optimal transport map between the corresponding capital distributions, with the manifold metric playing the role of the ground cost. This connection suggests that results from computational optimal transport (Cuturi's Sinkhorn algorithm, entropic regularization) could accelerate geodesic computation for high-dimensional manifolds.

One limitation we should be honest about: the manifold formulation assumes that the cost structure is smooth (continuously differentiable metric tensor). DeFi costs are not always smooth. Governance votes can change protocol parameters discontinuously. Flash crashes create discontinuous cost jumps. Liquidation cascades produce singular cost spikes. The manifold handles mild non-smoothness through the metric estimator's kernel smoothing (which effectively regularizes discontinuities), but severe discontinuities (a protocol upgrade that changes the fee structure overnight) require manifold re-initialization. The delta tick maintenance step handles this: if the metric error at any grid point exceeds a threshold after re-estimation, the Christoffel cache for that region is discarded and rebuilt from scratch.

## References

1. do Carmo, M.P. (1992). *Riemannian Geometry*. Birkhauser.

2. Lee, J.M. (2018). *Introduction to Riemannian Manifolds*. 2nd ed. Springer.

3. Amari, S. & Nagaoka, H. (2000). *Methods of Information Geometry*. American Mathematical Society.

4. Bao, D., Chern, S.S., & Shen, Z. (2000). *An Introduction to Riemann-Finsler Geometry*. Springer.

5. Lee, J.M. (2012). *Introduction to Smooth Manifolds*. 2nd ed. Springer.

6. Absil, P.A., Mahony, R., & Sepulchre, R. (2008). *Optimization Algorithms on Matrix Manifolds*. Princeton University Press.

7. Pennec, X. (2006). "Intrinsic Statistics on Riemannian Manifolds: Basic Tools for Geometric Measurements." *Journal of Mathematical Imaging and Vision*, 25(1), 127-154.

8. Bronstein, M.M., Bruna, J., Cohen, T., & Velickovic, P. (2021). "Geometric Deep Learning: Grids, Groups, Graphs, Geodesics, and Gauges." *arXiv:2104.13478*.

9. Adams, R.P. & Pennington, J. (2018). "Estimating the Spectral Density of Large Implicit Matrices." *arXiv:1802.03451*.

10. Nakahara, M. (2003). *Geometry, Topology and Physics*. 2nd ed. CRC Press.
