# Morphogenetic Clade Specialization: Turing Patterns for Role Emergence [SPEC]

> _"The pattern was always latent in the dynamics. It just needed a nudge."_

> **Version**: 1.0 | **Status**: Draft
>
> **Crates**: `golem-clade`, `golem-coordination`
>
> **Depends on**: `10-clade-ecology.md` (superorganism dynamics, stigmergic coordination), `01-architecture.md` (triple-clock system, behavioral phases), `../09-economy/02-clade.md` (Clade economic model and incentive structure)

---

> **Reader orientation:** This document specifies how Golems (mortal autonomous DeFi agents) in a Clade (sibling Golems sharing a common ancestor) spontaneously specialize into complementary roles (momentum traders, mean-reversion specialists, liquidity providers, risk monitors) without central planning. The mechanism is a reaction-diffusion system based on Turing's 1952 morphogenesis theory. Each Golem broadcasts a strategy-type pheromone through Styx (global knowledge relay); the pheromone inhibits neighbors from adopting the same strategy. See `10-clade-ecology.md` (superorganism dynamics, stigmergic coordination) for the broader Clade framework. See `prd2/shared/glossary.md` for full term definitions.

## Document Purpose

Left to their own devices, Golems in a Clade converge on the same strategy. They read the same markets, run the same models, optimize the same objective. The result is a Clade of clones, each stepping on the others' trades.

This document specifies a reaction-diffusion mechanism, grounded in Turing's 1952 morphogenesis theory, that drives spontaneous specialization in multi-agent Clades. Each agent broadcasts a strategy-type pheromone through the existing Styx relay. The pheromone inhibits neighbors from adopting the same strategy. Profitable performance reinforces the agent's own specialization. Because inhibition diffuses faster than reinforcement (pheromones spread in milliseconds; learning takes hundreds of ticks), the system satisfies Turing's instability condition, and stable specialist patterns emerge from an initially homogeneous population.

Momentum traders, mean-reversion specialists, liquidity providers, and risk monitors crystallize from local interactions alone. No central planner decides who does what.

---

## The Problem

Each Golem's strategy is chosen by its owner via configuration, or inherited from a dead predecessor. Nothing in the current architecture drives Golems toward complementary specializations. If three Golems in a five-member Clade all inherit a momentum-trading strategy, the Clade has three momentum traders competing for the same signals and two empty niches that nobody fills.

The ideal Clade is an ecosystem: some Golems trade momentum, some trade mean-reversion, some provide liquidity, some monitor risk. These strategies interact. Momentum traders create trends that mean-reversion traders profit from when they exhaust. Liquidity providers collect fees from both. Risk monitors protect the collective. But reaching this configuration requires central planning or luck.

Biology solved this 500 million years ago. Cells in a developing embryo start identical. Through local chemical signaling alone, they differentiate into neurons, muscle cells, blood cells. No central planner tells a cell to become a neuron. The pattern emerges from the interaction dynamics. Turing showed why in 1952.

---

## Mathematical Foundation

### Turing's Reaction-Diffusion Mechanism

Turing (1952) considered a system with two chemical species: an activator A and an inhibitor B. The activator stimulates its own production (autocatalysis) and also stimulates the inhibitor. The inhibitor suppresses the activator. Both diffuse through space, but the inhibitor diffuses faster.

The continuous reaction-diffusion equations are:

```
dA/dt = f(A, B) + D_A * nabla^2(A)
dB/dt = g(A, B) + D_B * nabla^2(B)
```

Turing's key result: even when the homogeneous steady state is stable without diffusion, adding diffusion can destabilize it -- provided D_B >> D_A. The inhibitor "runs away" from local activator peaks, leaving those peaks free to grow. The result is a stable pattern of high-activator peaks separated by high-inhibitor troughs.

### Strategy as Morphogen Concentration

Each Golem maintains a strategy concentration vector:

```
s = (s_1, s_2, ..., s_d) in R^d
```

where d = 8 strategy dimensions:

| Dimension | Range | Meaning |
|-----------|-------|---------|
| s_1: momentum | [0, 1] | Trend-following intensity |
| s_2: mean_reversion | [0, 1] | Mean-reversion intensity |
| s_3: liquidity_provision | [0, 1] | LP allocation fraction |
| s_4: risk_monitoring | [0, 1] | Defensive posture intensity |
| s_5: time_horizon | [0, 1] | 0 = scalping, 1 = position trading |
| s_6: asset_breadth | [0, 1] | 0 = single-asset, 1 = broad portfolio |
| s_7: volatility_appetite | [0, 1] | Preferred operating volatility regime |
| s_8: cross_chain | [0, 1] | Cross-chain bridging and arbitrage |

The vector is not a discrete label. It is a continuous distribution over strategy space. Specialists have one or two dominant components. Generalists have a flatter distribution.

### The Agent Reaction-Diffusion System

For each strategy dimension k:

**Activation (local, slow).** When a Golem earns returns attributed to dimension k, the concentration s_k increases. This is autocatalytic feedback -- doing well at something makes you do more of it.

```
activation_k = alpha * returns_k * s_k
```

**Inhibition (global, fast).** The Golem reads pheromone signals from the Clade. Each neighbor broadcasts its strategy vector through Styx. The total pheromone concentration for dimension k across the Clade (excluding self) is:

```
P_k = sum_{j != i} s_k^{(j)} * w(i, j)
```

The inhibition term is:

```
inhibition_k = beta * P_k * s_k
```

The product P_k * s_k means inhibition is strongest when both the Golem's own concentration and the neighborhood's concentration are high. If you're doing what everyone else is doing, the pressure to change is maximal.

**Decay.** Each strategy component decays toward a baseline:

```
decay_k = mu * (s_k - s_baseline)
```

**Update rule.** Per delta cycle (aligned with Clade sync, every 50 heartbeat ticks):

```
s_k(t+1) = s_k(t) + activation_k - inhibition_k - decay_k + noise_k
```

where noise_k ~ N(0, sigma_noise^2) for symmetry breaking. After the update, s is renormalized so sum(s_k) = 1.

### The Turing Instability Condition for Agents

The activation rate is bounded by how fast a Golem can learn from experience -- minutes to hours for meaningful signal. Call this D_A.

The inhibition rate is bounded by pheromone propagation through the Clade. Styx relays in near-real-time (sub-second WebSocket). Call this D_B.

D_B >> D_A. The condition is naturally satisfied. This asymmetry is exactly what Turing's mechanism requires.

What happens: suppose all five Golems start identical. Due to noise in returns, one Golem does marginally better at momentum. Its s_1 increases slightly. It broadcasts the increase via pheromones. All others reduce their s_1 (inhibition). The first Golem's advantage grows. Within a few dozen sync cycles, the uniform state has broken into distinct specialists.

### Gierer-Meinhardt Reaction Kinetics

> Source: `innovations/08-morphogenetic-agent-specialization.md`, Section 2

A standard choice for the reaction terms is the Gierer-Meinhardt model (1972):

```
f(A, B) = rho_A * (A^2 / B) - mu_A * A + sigma_A
g(A, B) = rho_B * A^2 - mu_B * B
```

where rho terms control production rates, mu terms control decay rates, and sigma_A is a small baseline production rate that prevents the system from collapsing to zero.

The instability condition (derived from linearizing the system around the uniform state) requires:

```
D_B / D_A > (mu_A * rho_B + mu_B * rho_A)^2 / (4 * mu_A * mu_B * rho_A * rho_B)
```

When this ratio is exceeded, certain spatial frequency modes become unstable. The wavelength of the fastest-growing mode determines the pattern's characteristic scale: how far apart the activator peaks are, or equivalently, how many distinct specialist types emerge.

### Stability Analysis

> Source: `innovations/08-morphogenetic-agent-specialization.md`, Section 2

Linearize the reaction-diffusion system around the uniform steady state s_0. The Jacobian of the reaction terms is:

```
J = | alpha * returns_k - beta * P_k - mu,  -beta * s_k |
    | 2 * rho_B * s_k,                      -mu_B       |
```

(simplified to the activator-inhibitor pair for a single strategy dimension). Add the diffusion terms via the Laplacian eigenvalues lambda_n = -n^2 * pi^2 / L^2 (for a "strategy space" of effective size L). The system matrix for mode n is:

```
M_n = J + D * lambda_n
```

where D = diag(D_A, D_B). The uniform state is unstable for mode n if any eigenvalue of M_n has positive real part. The fastest-growing mode determines the dominant pattern wavelength.

For a Clade of size C, the effective "domain size" L scales with C. Larger Clades support more unstable modes -- more distinct specialist types can emerge. A 3-Golem Clade might support 2 specialist types. A 20-Golem Clade (the current cap) might support 5-7.

The ratio D_B / D_A controls pattern sharpness. When D_B / D_A is very large (pheromones spread much faster than learning), the patterns are sharp -- Golems become strong specialists. When the ratio is moderate, the patterns are softer -- Golems maintain some generalist capability alongside their primary specialization.

### Predator-Prey Dynamics in Strategy Space

Strategy types interact through the market. Momentum traders create trends that mean-reversion traders profit from when exhausted. Liquidity providers collect fees from both. These are Lotka-Volterra dynamics (Lotka, 1925; Volterra, 1926):

```
dM/dt = r_M * M * (1 - M/K_M) - a_MR * M * R
dR/dt = r_R * R * (1 - R/K_R) + b_RM * M * R - c * R
```

The reaction-diffusion system produces not just static specialization but a dynamic ecosystem with oscillating populations.

---

## Mortality Integration

### Death Opens Niches

When a Golem dies, its strategy vector drops out of the pheromone field. The inhibition signal for its specialization decreases. If it was the sole mean-reversion specialist, P_mean_reversion drops sharply. Other Golems experience reduced inhibition on that dimension. Some shift toward filling the empty niche. The ecological vacancy creates evolutionary pressure.

### Successors Inherit at Half Confidence

When a successor boots with its predecessor's strategy vector at 0.5x, the inherited concentrations are weaker. The successor starts as a partial specialist. It can either deepen the inherited specialization (if the niche is still underserved) or drift toward a different one (if competitors have filled the gap). This is analogous to how stem cells differentiate depending on the local signaling environment.

### Conservation Phase Dampens Activation

A Golem in Conservation mortality phase (vitality 0.2-0.4) reduces its activation rate alpha by half. It stops reinforcing its current specialization and becomes more responsive to inhibition signals. This prevents dying Golems from rigidly holding a niche they can no longer exploit.

---

## CorticalState Signals

Two new atomic signals on the CorticalState perception surface:

```
specialization_index: AtomicU32  // f32 in [0, 1], normalized Shannon entropy
                                 // Low = specialist, High = generalist
niche_competition: AtomicU32     // f32 in [0, C], effective competitors
```

The specialization index is the normalized Shannon entropy of the strategy vector:

```
H(s) = -sum_k (s_k * ln(s_k)) / ln(d)
```

where d = 8. H(s) = 0 for a perfect specialist. H(s) = 1 for a perfect generalist.

The niche competition signal counts effective competitors: Golems whose strategy vectors have cosine similarity > 0.8 with the focal Golem.

---

## Implementation

### Core Data Structures

```rust
use std::sync::atomic::{AtomicU32, Ordering};

/// Strategy dimensions for a DeFi Clade.
pub const STRATEGY_DIMS: usize = 8;

pub const DIM_NAMES: [&str; STRATEGY_DIMS] = [
    "momentum",
    "mean_reversion",
    "liquidity_provision",
    "risk_monitoring",
    "time_horizon",
    "asset_breadth",
    "volatility_appetite",
    "cross_chain",
];

/// The morphogenetic state of a single Golem.
#[derive(Debug, Clone)]
pub struct MorphogeneticState {
    /// Strategy concentration vector. Sum = 1.0.
    pub strategy: [f64; STRATEGY_DIMS],
    /// Accumulated returns attributed to each dimension since last update.
    pub attributed_returns: [f64; STRATEGY_DIMS],
    /// Pheromone readings: aggregate strategy vectors from Clade neighbors.
    pub clade_pheromone: [f64; STRATEGY_DIMS],
    /// Number of Clade members (excluding self).
    pub clade_size: usize,
}

/// Parameters for the reaction-diffusion update.
#[derive(Debug, Clone)]
pub struct MorphogeneticParams {
    /// Activation rate: how fast profitable strategies reinforce.
    pub alpha: f64,
    /// Inhibition rate: how strongly Clade overlap suppresses.
    pub beta: f64,
    /// Decay rate toward baseline.
    pub mu: f64,
    /// Baseline strategy concentration (uniform = 1/d).
    pub baseline: f64,
    /// Noise standard deviation for symmetry breaking.
    pub sigma_noise: f64,
    /// Mortality-phase multiplier for activation rate.
    /// 1.0 in Thriving/Stable, 0.5 in Conservation, 0.1 in Declining.
    pub mortality_activation_scalar: f64,
}

impl Default for MorphogeneticParams {
    fn default() -> Self {
        Self {
            alpha: 0.05,
            beta: 0.15,
            mu: 0.01,
            baseline: 1.0 / STRATEGY_DIMS as f64,
            sigma_noise: 0.005,
            mortality_activation_scalar: 1.0,
        }
    }
}
```

### The Reaction-Diffusion Update

```rust
use rand::Rng;
use rand_distr::Normal;

impl MorphogeneticState {
    /// Run one reaction-diffusion update cycle.
    ///
    /// Called every 50 heartbeat ticks (~12.5 minutes), aligned with Clade sync.
    pub fn update(&mut self, params: &MorphogeneticParams, rng: &mut impl Rng) {
        let noise_dist = Normal::new(0.0, params.sigma_noise)
            .expect("sigma_noise must be non-negative");

        let effective_alpha = params.alpha * params.mortality_activation_scalar;

        for k in 0..STRATEGY_DIMS {
            let activation = effective_alpha
                * self.attributed_returns[k].max(0.0)
                * self.strategy[k];

            let p_k = if self.clade_size > 0 {
                self.clade_pheromone[k] / self.clade_size as f64
            } else {
                0.0
            };
            let inhibition = params.beta * p_k * self.strategy[k];

            let decay = params.mu * (self.strategy[k] - params.baseline);

            let noise: f64 = rng.sample(noise_dist);

            self.strategy[k] += activation - inhibition - decay + noise;
            self.strategy[k] = self.strategy[k].clamp(0.001, 1.0);
        }

        // Renormalize so sum = 1.0.
        let total: f64 = self.strategy.iter().sum();
        if total > 0.0 {
            for k in 0..STRATEGY_DIMS {
                self.strategy[k] /= total;
            }
        }

        self.attributed_returns = [0.0; STRATEGY_DIMS];
    }

    /// Shannon entropy of the strategy vector, normalized to [0, 1].
    pub fn specialization_index(&self) -> f32 {
        let max_entropy = (STRATEGY_DIMS as f64).ln();
        if max_entropy == 0.0 { return 1.0; }
        let entropy: f64 = self.strategy.iter()
            .filter(|&&s| s > 0.0)
            .map(|&s| -s * s.ln())
            .sum();
        (entropy / max_entropy) as f32
    }

    /// Count effective competitors: Clade members with cosine similarity > 0.8.
    pub fn niche_competition(
        &self,
        clade_strategies: &[[f64; STRATEGY_DIMS]],
    ) -> f32 {
        let threshold = 0.8;
        let self_norm = self.vector_norm(&self.strategy);
        if self_norm < 1e-10 { return 0.0; }
        let count = clade_strategies.iter()
            .filter(|other| {
                let dot: f64 = self.strategy.iter()
                    .zip(other.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                let other_norm = self.vector_norm(other);
                if other_norm < 1e-10 { return false; }
                dot / (self_norm * other_norm) > threshold
            })
            .count();
        count as f32
    }

    fn vector_norm(&self, v: &[f64; STRATEGY_DIMS]) -> f64 {
        v.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// Aggregate pheromone readings from Clade neighbors.
    pub fn ingest_pheromones(&mut self, pheromones: &[MorphogeneticPheromone]) {
        self.clade_pheromone = [0.0; STRATEGY_DIMS];
        self.clade_size = pheromones.len();
        for pheromone in pheromones {
            for k in 0..STRATEGY_DIMS {
                self.clade_pheromone[k] += pheromone.strategy[k];
            }
        }
    }
}
```

### Pheromone Field Integration

The strategy vector broadcasts through the existing Styx sync protocol. No new message types.

```rust
use serde::{Deserialize, Serialize};

/// Payload included in the Clade sync delta for morphogenetic coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphogeneticPheromone {
    pub strategy: [f64; STRATEGY_DIMS],
    pub specialization_index: f32,
    pub emitted_at_tick: u64,
}
```

### CorticalState Integration

```rust
/// Write morphogenetic signals to the CorticalState perception surface.
pub fn write_cortical_signals(
    specialization_signal: &AtomicU32,
    competition_signal: &AtomicU32,
    state: &MorphogeneticState,
    clade_strategies: &[[f64; STRATEGY_DIMS]],
) {
    let spec_index = state.specialization_index();
    let competition = state.niche_competition(clade_strategies);
    specialization_signal.store(spec_index.to_bits(), Ordering::Release);
    competition_signal.store(competition.to_bits(), Ordering::Release);
}
```

### Performance

The update function runs once every 50 ticks. Cost: ~50 floating-point operations. The pheromone payload adds 64 bytes (8 x f64) to the Clade sync message. Memory per Golem: ~200 bytes. For a 20-Golem Clade: 4 KB total.

---

## What This Enables

**Self-organizing ecology.** Drop five identical Golems into a Clade and within ~200 sync cycles (~two days), they differentiate into distinct specialists. The owner configures the reaction-diffusion parameters; the system does the rest.

**Dynamic rebalancing.** When a niche becomes crowded, the inhibition signal intensifies. Some Golems shift toward less crowded niches. The system self-corrects.

**Resilience through diversity.** When one strategy type fails, the Clade does not collapse. The diversified portfolio of strategies buffers against regime-specific losses.

**Death as niche opportunity.** When a specialist dies, its pheromone disappears. The inhibition on that niche drops. Surviving Golems sense the opening. The successor, inheriting the dead predecessor's vector at 0.5x, can fill the niche faster than starting from scratch.

**Observable structure.** The specialization_index and niche_competition signals appear on the CorticalState dashboard. The owner watches ecological structure form in real time.

---

## Evaluation and Falsifiability

### Null Hypothesis

Reaction-diffusion specialization produces no better strategy diversity than random assignment.

### Protocol

Two populations of Clades (5 Golems each, 20 Clades per population) on historical Base L2 data (90-day window). One uses the morphogenetic update. The other uses random strategy initialization held fixed.

### Metrics

- **Strategy coverage:** Fraction of strategy-space bins occupied. Morphogenetic > random expected.
- **Redundancy:** Mean pairwise cosine similarity. Morphogenetic < random expected.
- **Clade-level Sharpe ratio:** Diversification benefit should raise Clade Sharpe above individual average.
- **Niche recovery time:** Sync cycles before another Golem fills a dead specialist's niche. Morphogenetic: 20-50 cycles. Random: never.

### Falsification Conditions

1. Random Clades achieve equal or higher strategy coverage after 500 sync cycles in > 50% of trials.
2. Clade-level Sharpe ratio improvement is not statistically significant (p > 0.05).
3. Morphogenetic Clades exhibit strategy "churning" -- oscillating between niches rather than stabilizing.

---

## Philosophical Grounding

> Source: `innovations/08-morphogenetic-agent-specialization.md`, Sections 7

### Turing and Symmetry Breaking

Turing's 1952 paper has an arresting opening. He considers a ring of identical cells, each with the same chemical composition, and asks: how does asymmetry arise from symmetry? His answer: it does not need to "arise." The symmetric state is unstable. Any infinitesimal perturbation -- thermal noise, a stray molecule -- gets amplified by the reaction-diffusion dynamics into a macroscopic pattern. The pattern was always latent in the dynamics. It just needed a nudge.

The same holds for a Clade of identical Golems. The uniform state (everyone doing the same thing) is not stable. Noise in returns, timing differences in trade execution, minor variations in initialization -- any of these break the symmetry. The reaction-diffusion dynamics amplify the break. Specialists emerge because the homogeneous state is mathematically unstable, not because anyone planned them.

### Kauffman and the Edge of Chaos

Stuart Kauffman (1993) argued that biological systems operate at the boundary between order and chaos. Systems in the ordered regime are too rigid: they cannot adapt. Systems in the chaotic regime are too unstable: they cannot maintain useful structure. Systems at the edge combine adaptability with structure.

The reaction-diffusion parameters control where the Clade sits on this spectrum. Strong inhibition (high beta) and weak noise (low sigma_noise) push toward order: rigid specialists that never change. Weak inhibition and strong noise push toward chaos: Golems drifting randomly through strategy space with no stable pattern. The parameters alpha, beta, mu, and sigma_noise must be tuned so that the Clade sits at the edge: specialists that are stable enough to develop deep expertise but flexible enough to respecialize when conditions demand it.

This is not a metaphor. The stability analysis gives exact conditions for when the system is in the pattern-forming regime versus the homogeneous (ordered) or chaotic regimes. The eigenvalue analysis tells us which parameter combinations produce stable patterns, which produce uniform convergence, and which produce oscillations. Kauffman's edge of chaos, for this system, has a precise mathematical characterization.

### DeLanda and Assemblage Theory

Manuel DeLanda (2006), building on Deleuze and Guattari, proposed assemblage theory: complex systems are compositions of heterogeneous parts whose properties emerge from the interactions between parts, not from the parts themselves. An assemblage has two axes: a material axis (the physical components) and an expressive axis (the patterns of interaction). The whole has properties that no part possesses in isolation.

A morphogenetic Clade is an assemblage. Its material components are individual Golems, each with a strategy vector, a portfolio, and a mortality clock. Its expressive axis is the pheromone field: the pattern of strategy signals flowing between Golems that creates the reaction-diffusion dynamics. No individual Golem "knows" the Clade's ecological structure. The structure exists only in the interaction pattern, in the pheromone field, in the gaps between agents. Kill the Styx relay and the Clade becomes a collection of isolated agents. The assemblage dissolves.

### Death Prevents Pattern Lock-In

Without mortality, the first stable pattern to emerge from the reaction-diffusion dynamics would persist indefinitely. The specialists that form during the first market regime would hold their niches through all subsequent regimes, growing increasingly mismatched to current conditions but never yielding their positions.

Mortality breaks this lock-in. When a specialist dies -- whether from economic ruin, epistemic decay, or stochastic bad luck -- its niche opens. The pheromone field changes. Surviving Golems sense new possibilities. The successor, starting with halved strategy concentrations, may differentiate into a different specialist type if the old niche is no longer viable.

Over many generations, the Clade's ecological structure evolves. The distribution of specialist types tracks the distribution of market opportunities, not because anyone steers it, but because mortality creates continuous turnover and the reaction-diffusion dynamics fill vacancies according to current profitability signals. The Clade adapts because its members die. The dead make room for the living to become what the market needs now.

---

## Information-Theoretic Redundancy Metric

> Source: `innovations/08-morphogenetic-agent-specialization.md`, Section 6

Using the framework from 17-information-theoretic-diagnostics.md: compute I(G; M | Clade) for each Golem. Morphogenetic Clades should have higher conditional mutual information per Golem (each Golem contributes unique information about the market) and lower I(G; M; Clade) overlap. This provides a formal metric for whether the morphogenetic system is producing genuine differentiation or cosmetic variation.

---

## Cross-References

- **10-clade-ecology.md**: The pheromone field that carries threat and opportunity signals is the same field that carries strategy concentrations. The two systems share a substrate.
- **17-information-theoretic-diagnostics.md**: The Clade redundancy criterion I(G; M | Clade) = 0 signals failed differentiation -- the morphogenetic system actively prevents this.
- **01-architecture.md**: The three-clock mortality system constrains the morphogenetic dynamics via the mortality_activation_scalar.
- **09-economy/02-clade.md**: Styx L1 carries the morphogenetic pheromone payloads alongside existing Clade sync deltas.
- **18-antifragile-mortality.md**: Heterogeneous specialist responses to stress produce the diversity that antifragility requires.

---

## References

1. Turing, A.M. (1952). "The Chemical Basis of Morphogenesis." *Philosophical Transactions of the Royal Society of London B*, 237(641), 37-72.

2. Gierer, A. & Meinhardt, H. (1972). "A Theory of Biological Pattern Formation." *Kybernetik*, 12(1), 30-39.

3. Kondo, S. & Miura, T. (2010). "Reaction-Diffusion Model as a Framework for Understanding Biological Pattern Formation." *Science*, 329(5999), 1616-1620.

4. Camazine, S., Deneubourg, J.-L., Franks, N.R., Sneyd, J., Theraulaz, G. & Bonabeau, E. (2001). *Self-Organization in Biological Systems*. Princeton University Press.

5. Lotka, A.J. (1925). *Elements of Physical Biology*. Williams & Wilkins.

6. Volterra, V. (1926). "Fluctuations in the Abundance of a Species Considered Mathematically." *Nature*, 118, 558-560.

7. Kauffman, S.A. (1993). *The Origins of Order*. Oxford University Press.

8. DeLanda, M. (2006). *A New Philosophy of Society: Assemblage Theory and Social Complexity*. Continuum.

9. Grasse, P.-P. (1959). "La reconstruction du nid et les coordinations interindividuelles." *Insectes Sociaux*, 6(1), 41-80.
