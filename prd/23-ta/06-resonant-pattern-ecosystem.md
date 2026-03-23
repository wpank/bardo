# Resonant Pattern Ecosystem: Evolutionary Dynamics for Living TA Patterns [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Source**: `tmp/research/witness-research/new/ta/06-resonant-pattern-ecosystem.md`
>
> **Depends on**: Doc 0 (Overview), Doc 1 (HDC Encoding), Doc 3 (Signal Metabolism)

**Audience:** Researchers and engineers with background in Rust, evolutionary computation, and DeFi protocol mechanics. Assumes familiarity with the Bardo runtime architecture (Doc 0), HDC encoding (Doc 1), and signal metabolism (Doc 3).

---

> **Reader orientation:** This document reframes TA patterns as living organisms in an evolutionary ecosystem within the Golem (mortal autonomous DeFi agent) runtime. It belongs to the **TA research layer** (Doc 6 of 10) and covers HDC genome encoding for patterns, VCG auction selection for cognitive resources, Lotka-Volterra predator-prey dynamics for edge depletion, pattern reproduction via dream-cycle mutation, and generational knowledge transfer through Thanatopsis (the four-phase death protocol). You should understand evolutionary game theory, auction mechanisms, and DeFi market microstructure. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Abstract

Technical analysis patterns are treated as static artifacts: a human designs a head-and-shoulders detector, codes it, and deploys it. The pattern runs unchanged until someone notices it stopped working. Markets, meanwhile, adapt continuously. Crowded patterns lose their edge. Adversaries learn to detect and front-run them. The shelf life of any fixed pattern set is short and getting shorter.

This document reframes TA patterns as living organisms in an evolutionary ecosystem. Each pattern is encoded as a 10,240-bit hypervector (its genome), competes for cognitive resources via VCG auction (natural selection), reproduces through HDC bundle operations (sexual crossover at XOR speed), mutates during REM dream cycles (random perturbation), and dies when its fitness drops below the population floor. Golem death acts as a generational bottleneck: the Thanatopsis death testament preserves the fittest patterns and transfers them to successor Golems. Pattern migration across a Clade via Styx relay is horizontal gene transfer.

The mathematical framework combines replicator dynamics from evolutionary game theory, Lotka-Volterra predator-prey equations for edge depletion, Fisher's fundamental theorem for fitness variance tracking, and niche specialization per DeFi primitive type. The Rust implementation provides a complete `PatternEcosystem` that integrates with the Golem's heartbeat at all three timescales: Gamma (evaluation), Theta (reproduction and selection), and Delta (census and ecosystem health). Patterns that work propagate. Patterns that don't get replaced by their mutant offspring. The ecosystem self-regulates through exploitation dynamics that maintain diversity without central coordination.

## The problem [SPEC]

A Uniswap v3 concentrated liquidity pool has been exhibiting a specific behavior for the past week: large liquidity additions below the current price, followed by aggressive selling that pushes the price into the new liquidity, followed by removal. Call it the "liquidity trap" pattern. A Golem that detects this pattern early can front-run the removal or avoid being caught as passive liquidity.

The pattern works for about three days. Then other participants notice. Some build their own detectors. Some start placing decoy liquidity. The original pattern's accuracy drops from 0.78 to 0.41. A static pattern library has no mechanism to respond. The Golem keeps running the degraded detector, wasting attention budget and acting on stale signal, until a human operator notices and updates the code.

This failure mode is not specific to one pattern. It is structural. Every profitable pattern in a transparent market creates incentives for adaptation by other participants. The only question is how fast the adaptation happens. On Ethereum mainnet, where every transaction is visible in the mempool and every strategy is inspectable on-chain, the adaptation can happen within hours.

The problem has four dimensions.

**Pattern decay.** Market edges erode as participants adapt. A pattern that returned 0.78 accuracy last week returns 0.41 this week and will return baseline (0.50) by next week. The decay rate depends on how many participants exploit the same edge and how easy the pattern is to reverse-engineer from on-chain behavior.

**Pattern rigidity.** Static pattern libraries cannot adapt to new market conditions. When a new DeFi primitive launches (a new vault strategy, a new perpetual design, a new lending mechanism), the existing patterns have no way to generalize. Someone must write new detectors by hand.

**Attention waste.** A Golem with 30 patterns in its library runs all 30 on every Theta tick, even when 20 of them have decayed to baseline accuracy. This wastes compute budget and, more critically, wastes attention budget in the VCG auction. Dead patterns still bid, crowding out live ones.

**Adversarial exploitation.** If a Golem's pattern library is fixed and its behavior is on-chain, an adversary can infer which patterns the Golem runs by observing its actions. Once the adversary knows the pattern set, they can construct inputs designed to trigger false positives, causing the Golem to act on fabricated signals.

The fix is to stop treating patterns as static code and start treating them as populations of evolving organisms. A living pattern ecosystem solves all four problems simultaneously: decayed patterns die and get replaced by mutants, new niches get colonized by offspring of existing patterns that happened to generalize, dead patterns stop bidding because they are dead, and adversaries face a moving target because the pattern population changes every generation.

## Mathematical foundations [SPEC]

### Patterns as organisms

A pattern organism P consists of five components:

```
P = (hv, W, niche, lineage, generation)
```

The hypervector `hv` is the pattern's genome: a 10,240-bit binary spatter code (BSC) vector from the HDC framework established in Doc 1. Every pattern, regardless of what market behavior it detects, is encoded in the same 1,280-byte representation. This uniform encoding makes crossover and mutation trivial bitwise operations.

Fitness W combines three measurements:

```
W(P) = accuracy(P) * information_gain(P) / cost(P)
```

Accuracy is the exponential moving average of the pattern's prediction correctness over the last N evaluations. Information gain measures how much the pattern's predictions reduce uncertainty beyond baseline (a pattern that predicts what you already know has zero information gain). Cost is the compute time consumed per evaluation, normalized against the Golem's current budget.

Niche specifies which DeFi primitive type the pattern has specialized for. Lineage tracks the pattern's ancestry (parent IDs, grandparent IDs). Generation counts how many reproduction events separate this pattern from its founding ancestor.

The organism metaphor is not decorative. It determines the system's dynamics. Organisms with higher fitness reproduce more often. Organisms consume resources (attention budget). When resources are scarce (the Golem approaches death and its budget shrinks), competition intensifies and only the fittest survive. These are the conditions for natural selection.

### Reproductive algebra in HDC space

Two parent patterns produce offspring through the HDC bundle operation, which is majority vote applied bitwise. For two parents, bundle degenerates to a coin flip at each disagreeing bit position, so we use weighted bundling:

```
child_hv[i] = if rand() < w_a / (w_a + w_b) { P_a.hv[i] } else { P_b.hv[i] }
```

where w_a and w_b are proportional to parent fitness. The fitter parent contributes more bits. At the limit where one parent has all the weight, the child is a clone. When parents have equal fitness, each contributes 50%.

The resulting child hypervector has calculable similarity to each parent:

```
sim(child, P_a) = w_a / (w_a + w_b)
sim(child, P_b) = w_b / (w_a + w_b)
```

This is sexual reproduction at hardware speed. The crossover operation is a single pass over 1,280 bytes, comparing random values against the weight ratio. On modern x86 with AVX-512, the entire operation completes in under 200 nanoseconds for 10,240 bits. Compare this to crossover in genetic programming, where parsing tree structures and recombining subtrees takes microseconds to milliseconds.

The crossover preserves semantic locality. If parent A detects liquidity trap patterns and parent B detects momentum divergence, the child inherits partial sensitivity to both. Whether this hybrid is useful depends on whether the market rewards that combination. Selection will tell.

### Mutation as XOR perturbation

Mutation flips a fraction of bits in a pattern's hypervector:

```
mutant_hv = P.hv XOR noise_mask
```

The noise mask has `f * D` randomly set bits, where `f` is the mutation rate and `D = 10,240` is the vector dimension. The relationship between mutation rate and semantic distance is direct:

| Mutation rate f | Bits flipped | Hamming similarity | Interpretation |
|----------------|-------------|-------------------|---------------|
| 0.001 | ~10 | 0.999 | Micro-perturbation, nearly identical |
| 0.01 | ~102 | 0.980 | Minor variant, same pattern family |
| 0.05 | ~512 | 0.900 | Significant divergence, new subspecies |
| 0.10 | ~1,024 | 0.800 | Major variant, different detection behavior |
| 0.25 | ~2,560 | 0.500 | Quasi-orthogonal, effectively new pattern |
| 0.50 | ~5,120 | ~0.000 | Random, no relation to parent |

The default mutation rate is 0.01 (1%), applied during REM dream cycles. When the ecosystem's diversity index (Shannon entropy) drops below a threshold, the mutation rate increases adaptively up to 0.10, injecting more variation to prevent population collapse.

The mutation operation is a single XOR over 1,280 bytes. With AVX-512, that is two 512-bit operations per cache line, completing in approximately 2 nanoseconds for the full vector. Millions of mutants can be generated per second.

### Fitness dynamics: replicator equations

The population evolves according to the replicator equation from evolutionary game theory (Taylor and Jonker, 1978):

```
dx_i/dt = x_i * (W_i - W_bar)
```

where `x_i` is the frequency of pattern i in the population, `W_i` is pattern i's fitness, and `W_bar` is the mean population fitness. Patterns with above-average fitness increase in frequency. Patterns with below-average fitness decrease.

Discretized for the Golem's tick-based execution:

```
x_i(t+1) = x_i(t) * (1 + alpha * (W_i - W_bar))
```

where alpha is the selection pressure parameter. High alpha (aggressive selection) means the population converges quickly to the fittest patterns but risks premature convergence. Low alpha (gentle selection) maintains diversity but slows adaptation. The system adapts alpha based on the fitness variance (see Fisher's theorem below).

Fisher's fundamental theorem of natural selection (Fisher, 1930) states that the rate of fitness increase equals the additive genetic variance in fitness:

```
dW_bar/dt = Var(W)
```

In plain terms: the more diverse the population's fitness values, the faster the population improves. A population where all patterns have similar fitness is stagnant. A population with high fitness variance is evolving rapidly.

This gives us a direct diagnostic. If Var(W) is near zero, the ecosystem has converged (or collapsed). If Var(W) is high, the ecosystem is actively evolving. We track this metric in the `EcosystemCensus` and use it to modulate mutation rate: low variance triggers higher mutation to inject diversity.

### The Price equation: tracking knowledge improvement

Innovation 06 uses the Price equation to track whether the Grimoire's memetic population is improving over time. The same equation applies to the pattern ecosystem, giving us a decomposition of fitness change into selection and transmission components.

The Price equation (Price, 1970):

```
delta(W_bar) = Cov(W_i, x_i) / W_bar + E(W_i * delta_i) / W_bar
```

The first term, `Cov(W_i, x_i) / W_bar`, captures the selection effect: how much of the fitness change comes from fitter patterns increasing in frequency. This is always non-negative (selection always improves mean fitness, all else equal).

The second term, `E(W_i * delta_i) / W_bar`, captures the transmission effect: how much fitness changes within individual pattern lineages due to mutation and crossover. This can be positive (beneficial mutations) or negative (deleterious mutations). During REM dream cycles with high mutation rates, this term swings negative as mutations scramble existing patterns. During calm evolution with low mutation, this term is near zero.

The decomposition tells us whether the ecosystem is improving through selection (good patterns winning) or through innovation (patterns getting better through mutation). A healthy ecosystem shows both terms contributing. An ecosystem that improves only through selection is running out of genetic variation. An ecosystem that improves only through transmission has too little competitive pressure.

In practice, we compute the Price equation decomposition at every Delta tick and store it in the census. If the selection term dominates for more than 5 consecutive Delta cycles, the adaptive mutation rate increases to inject variation. If the transmission term is chronically negative (mutations are consistently harmful), the mutation rate decreases.

### Competition: VCG auction integration

Patterns compete for attention budget through the same VCG (Vickrey-Clarke-Groves) mechanism that allocates resources across all Golem subsystems. Each pattern submits a bid for evaluation slots based on its expected marginal contribution to decision quality:

```
bid(P) = W(P) * confidence(P, current_context) * budget_remaining_fraction
```

The VCG mechanism allocates evaluation slots to maximize total welfare (sum of bids), and each pattern pays the externality it imposes on others (the welfare loss caused by its inclusion). Patterns that win evaluation slots get to observe and predict, which updates their fitness. Patterns that lose evaluation slots get no observations and their fitness decays toward a prior.

Budget constraint from mortality sharpens selection. As a Golem approaches death, its total attention budget shrinks (the mortality engine reduces `inference_budget_remaining` in CorticalState). Fewer evaluation slots means more competition. Only the patterns with highest expected contribution survive the auction. This creates the "ecological bottleneck before death" effect: dying Golems run lean, high-fitness pattern populations.

### Predator-prey dynamics: edge depletion

Markets adapt to exploited patterns. When many participants use the same pattern, the underlying edge erodes. This is the predator-prey dynamic: the pattern population is the predator, the market edge is the prey.

Lotka-Volterra equations model this interaction:

```
dP/dt = alpha * P * E - beta * P
dE/dt = gamma * E * (1 - E/K) - delta * P * E
```

Where:
- P is the effective pattern population (weighted by the number of Golems and external participants using similar strategies)
- E is the edge magnitude (excess return above baseline from exploiting the pattern)
- alpha is the growth rate of pattern population when edge exists
- beta is the natural death rate of patterns (from attention budget competition)
- gamma is the edge regeneration rate (how fast the market "forgets" and the opportunity re-emerges)
- K is the carrying capacity of the edge (maximum exploitable return)
- delta is the edge consumption rate per pattern

The logistic term `E * (1 - E/K)` means edges regenerate toward a carrying capacity, not indefinitely. A funding rate arbitrage opportunity has a maximum yield set by the rate differential. A liquidity trap pattern has a maximum profit set by the size of the trap.

This system produces oscillations. A rare pattern exploiting a fat edge grows rapidly. The growing population depletes the edge. The depleted edge kills the less-fit pattern variants. The reduced population allows the edge to regenerate. The cycle repeats. The oscillation period depends on the parameters, but typical DeFi edges cycle on the order of days to weeks.

For the Golem, these equations run as a background model estimating the remaining edge for each pattern niche. When estimated edge drops below a threshold, the ecosystem increases mutation pressure on that niche, forcing the population to explore variants that might find a new angle on the diminished opportunity.

### Niche specialization

Each DeFi primitive type is an ecological niche with distinct selection pressures:

**Swap niche.** Dominated by momentum and mean-reversion patterns. High turnover rate: patterns that detect short-term price trends in AMM pools lose their edge within days as arbitrage bots adapt. Carrying capacity is moderate (there is always some momentum signal in noisy markets). Favors fast, cheap patterns over sophisticated ones.

**LP niche.** Patterns here detect optimal rebalancing timing and fee/impermanent-loss tradeoffs. Slower turnover: LP strategies operate on longer timescales and the edge erodes more slowly. Higher carrying capacity because the LP decision space is wider (tick range selection, rebalancing frequency, fee tier choice). Favors patterns that integrate multiple signals.

**Lending niche.** Utilization rate prediction and rate arbitrage. Very slow turnover because lending rates change gradually. Low carrying capacity for rate arbitrage (small spread), but high carrying capacity for liquidation risk patterns (large tail events create opportunity). Favors patterns with long memory.

**Vault niche.** Harvest timing and strategy rotation detection. Medium turnover tied to vault strategy rebalancing frequency. Patterns that detect when a yield aggregator is about to rotate its strategy can front-run the rotation.

**Perpetual niche.** Funding rate mean-reversion patterns dominate. Moderate turnover: funding rates are partially predictable from basis and open interest, but the predictability varies with market conditions. High carrying capacity during volatile periods, low during calm ones.

Niche affinity is a learned quantity:

```
fitness(P, niche) = W(P) * affinity(P, niche)
```

where `affinity(P, niche)` is the exponential moving average of the pattern's accuracy specifically within that niche context. A pattern can have high global fitness but low affinity for a particular niche, or vice versa. Reproduction within a niche preferentially selects for niche-adapted parents, driving specialization.

This connects to Doc 3's signal speciation mechanism. When a signal finds context-dependent performance (works in the swap niche, fails in the lending niche), it forks into niche-specific variants. Pattern organisms undergo the same speciation process, but the forking happens through reproduction and selection rather than explicit splitting.

### Death as reproduction

When a Golem dies, its Thanatopsis death testament extracts the top-k patterns by fitness. This extraction is a population bottleneck: from a population of potentially hundreds of patterns, only the fittest 20 or 30 survive into the testament.

Bottleneck effects from population genetics apply directly. The surviving patterns are more fit on average than the pre-death population, but they carry less genetic diversity. If the same niche conditions persist, this is fine: the high-fitness survivors reproduce and quickly refill the population. If conditions have changed (the successor Golem operates in a different market regime), the reduced diversity is a problem.

The fix is REM dream mutation in the successor Golem. After inheriting a testament, the successor immediately runs an elevated-mutation dream cycle, generating variants of the inherited patterns. This restores diversity without discarding the fitness gained during the previous Golem's lifetime.

The cycle across Golem generations:

```
Golem N: ecosystem evolves, patterns improve via selection
    |
    v
Golem N dies: bottleneck extracts top-k patterns
    |
    v
Golem N+1 inherits: testament patterns seed the new population
    |
    v
REM dream cycle: elevated mutation restores diversity
    |
    v
Golem N+1: ecosystem evolves from a higher fitness floor
```

Each generation starts better than the last. The fitness floor ratchets upward across Golem lifetimes, bounded by the rate at which markets adapt to the pattern population.

## Architecture [SPEC]

### System overview

The `PatternEcosystem` sits between the signal metabolism layer (Doc 3) and the VCG attention auction. Patterns receive observations from the signal pipeline, make predictions, accumulate fitness, and compete for evaluation slots. The ecosystem manager handles the lifecycle: reproduction, mutation, death, and migration.

```
DeFi Observations (from triage + protocol state)
    |
    v
Signal Metabolism (Doc 3) --- competition for attention budget
    |
    v
Pattern Ecosystem (this doc)
    |--- Gamma: evaluate patterns against observations
    |--- Theta: update fitness, select for reproduction, kill unfit
    |--- Delta: population census, niche analysis, ecosystem health
    |--- NREM: consolidate fitness histories into long-term averages
    |--- REM: primary mutation mechanism (XOR perturbation)
    |
    v
VCG Auction --- patterns bid for evaluation slots alongside other subsystems
    |
    v
Oracle decision-making --- pattern predictions influence action selection
```

### Heartbeat integration

**Gamma tick (5-15s).** Each pattern that won an evaluation slot in the previous auction gets to observe the current `DeFiObservation` and produce a `PatternMatch`: a prediction about what will happen next, with confidence. The prediction is recorded. No fitness updates yet (outcomes are not known).

**Theta tick (30-120s).** Three operations run in sequence:

1. *Fitness update.* Resolve pending predictions against observed outcomes. Update each pattern's exponential moving averages for accuracy and information gain. Recompute composite fitness.

2. *Reproduction.* For each niche with population below carrying capacity, select two parents via tournament selection (k=4) and produce one offspring via weighted crossover. Insert the offspring into the population.

3. *Mortality.* Kill patterns with fitness more than one standard deviation below the niche mean, subject to the minimum population constraint. Dead patterns are logged with their lineage for post-mortem analysis.

**Delta tick (~50 Theta ticks).** Run the full ecosystem census:

- Population count per niche
- Shannon diversity index (H = -sum(p_i * ln(p_i)) where p_i is frequency of pattern i)
- Fitness variance (Fisher's theorem diagnostic)
- Niche coverage map (which niches are well-populated, which are underpopulated)
- Predator-prey dynamics update (Lotka-Volterra step for edge estimation)
- Ecosystem health score (composite of diversity, coverage, and fitness improvement rate)

**NREM dream.** Consolidate short-term fitness histories into long-term exponential averages. Merge staging fitness data with persistent fitness records. This is the pattern analog of memory consolidation: recent experiences get folded into long-term knowledge.

**REM dream.** The primary mutation mechanism. For each pattern in the population:

1. Generate a random mutation with rate sampled from the current adaptive mutation distribution
2. Evaluate the mutant against a replay buffer of recent observations (counterfactual backtesting)
3. If the mutant's counterfactual fitness exceeds the parent's current fitness by a threshold, insert the mutant into the population
4. If not, discard the mutant (most mutations are deleterious, as in biology)

The REM dream cycle can generate and evaluate thousands of mutants in seconds because both mutation (XOR) and evaluation (Hamming distance comparison) are bitwise operations.

### Styx migration (horizontal gene transfer)

Patterns migrate between Clade members via the Styx relay. A Golem with a high-fitness pattern in the swap niche can broadcast it to siblings, who integrate it into their own ecosystems without going through the normal reproduction cycle. This is horizontal gene transfer: the acquisition of genetic material from an unrelated organism.

Migration rules:

- *Emigration threshold.* Only patterns with fitness above the 90th percentile of their niche are eligible for emigration.
- *Immigration quarantine.* Incoming patterns enter a quarantine buffer and are evaluated against local observations for one Delta cycle before being admitted to the general population. This prevents a single bad pattern from contaminating the entire Clade.
- *Frequency cap.* A Golem can emit at most 5 patterns per Delta cycle to prevent flooding.
- *Diversity bonus.* Incoming patterns that are dissimilar to the existing population (Hamming distance > 0.3 from the nearest existing pattern) get a fitness bonus during quarantine evaluation, encouraging adoption of genuinely novel patterns.

### CorticalState integration

The pattern ecosystem writes two signals to the CorticalState perception surface. These signals are readable by every other subsystem.

```rust
use std::sync::atomic::{AtomicU32, AtomicU16};

/// Pattern ecosystem signals on the CorticalState bus.
///
/// Written exclusively by the PatternEcosystem at Delta ticks.
/// Read by the Oracle (ecosystem health informs risk tolerance),
/// the Daimon (low diversity triggers anxiety-like arousal),
/// and the mortality engine (ecosystem collapse accelerates death).
#[repr(C, align(64))]
pub struct PatternEcoCorticalSignals {
    /// Shannon diversity index of the pattern population, packed as f32.
    /// Range roughly [0.0, 3.0] for typical populations.
    /// The Oracle reads this to modulate confidence in pattern-derived
    /// predictions: low diversity means less trustworthy predictions
    /// because the population may have converged on a local optimum.
    pub ecosystem_diversity: AtomicU32,

    /// Fraction of DeFi niches with adequate pattern coverage.
    /// Range [0.0, 1.0]. A value of 1.0 means every niche has at least
    /// min_population_per_niche viable patterns. The Daimon reads this
    /// to detect "blind spots" -- niches where the Golem cannot detect
    /// any patterns, which should trigger elevated arousal.
    pub niche_coverage_fraction: AtomicU32,

    /// Number of active pattern organisms, packed as u16.
    /// Quick read for dashboards and the mortality engine.
    pub active_patterns: AtomicU16,

    /// Dominant niche ID (maps to DeFiContext enum discriminant).
    /// Tells other subsystems what the pattern population is
    /// currently best adapted for.
    pub dominant_niche: AtomicU16,
}

impl PatternEcoCorticalSignals {
    pub fn write_from_census(&self, census: &EcosystemCensus) {
        self.ecosystem_diversity.store(
            f32::to_bits(census.diversity_index as f32),
            std::sync::atomic::Ordering::Relaxed,
        );

        let coverage_frac: f32 = census
            .niche_coverage
            .values()
            .filter(|&&v| v > 0.5)
            .count() as f32
            / census.niche_coverage.len().max(1) as f32;

        self.niche_coverage_fraction.store(
            f32::to_bits(coverage_frac),
            std::sync::atomic::Ordering::Relaxed,
        );

        self.active_patterns.store(
            census.total_population as u16,
            std::sync::atomic::Ordering::Relaxed,
        );

        // Map DeFiContext to a discriminant. In the real implementation,
        // this would use the enum's repr value.
        let dominant_id: u16 = match census
            .niche_populations
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(niche, _)| *niche)
            .unwrap_or(DeFiContext::Swap)
        {
            DeFiContext::Swap => 0,
            DeFiContext::LiquidityProvision => 1,
            DeFiContext::Lending => 2,
            DeFiContext::Vault => 3,
            DeFiContext::Perpetual => 4,
        };

        self.dominant_niche.store(
            dominant_id,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}
```

## Rust implementation [SPEC]

### Core types

```rust
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// Unique identifier for a pattern organism within an ecosystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PatternId(pub u64);

/// DeFi primitive types that define ecological niches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeFiContext {
    Swap,
    LiquidityProvision,
    Lending,
    Vault,
    Perpetual,
}

/// Exponential moving average with configurable decay.
/// Uses Welford's online algorithm for numerical stability.
#[derive(Clone, Debug)]
pub struct ExponentialAverage {
    value: f64,
    alpha: f64,
    count: u64,
}

impl ExponentialAverage {
    pub fn new(alpha: f64) -> Self {
        Self {
            value: 0.5, // prior: uninformative
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

### The pattern organism

```rust
/// A living TA pattern encoded as a hypervector with evolutionary metadata.
///
/// The hypervector is the pattern's genome. Everything else is phenotype
/// (expressed through interaction with the environment). Two patterns can
/// have identical genomes but different fitness values if they operate
/// in different niches or were evaluated against different market conditions.
#[derive(Clone, Debug)]
pub struct PatternOrganism {
    pub id: PatternId,

    /// The pattern's genome: 10,240-bit BSC hypervector stored as 160 u64 words.
    /// All evolutionary operations (crossover, mutation, similarity) operate
    /// directly on this representation.
    pub hv: Vec<u64>,

    /// Fitness components, tracked separately for diagnostic purposes.
    pub fitness: PatternFitness,

    /// The ecological niche this pattern has specialized for.
    /// Set at birth (inherited from the dominant parent) but can shift
    /// if the pattern's niche affinity changes through evaluation.
    pub niche: DeFiContext,

    /// Parent pattern IDs. Empty for founder patterns (seeded at ecosystem init).
    /// Two entries for sexually reproduced patterns. One entry for clones.
    pub lineage: Vec<PatternId>,

    /// Reproduction events since the founding ancestor.
    pub generation: u32,

    /// Heartbeat tick when this pattern was created.
    pub born_at_tick: u64,

    /// Total number of observations this pattern has evaluated.
    pub total_evaluations: u64,

    /// Number of offspring this pattern has produced.
    pub reproduction_count: u32,
}

/// Fitness breakdown for a single pattern organism.
#[derive(Clone, Debug)]
pub struct PatternFitness {
    /// Prediction correctness averaged over recent evaluations.
    pub accuracy: ExponentialAverage,

    /// Bits of uncertainty reduced per prediction, relative to a uniform prior.
    pub information_gain: ExponentialAverage,

    /// Normalized compute cost per evaluation (lower is better).
    pub compute_cost: f64,

    /// Per-niche affinity scores: how well this pattern performs in each context.
    pub niche_affinities: HashMap<DeFiContext, f64>,

    /// The final fitness value: accuracy * information_gain / cost.
    /// Recomputed at every Theta tick.
    pub composite_fitness: f64,

    /// Estimated remaining edge in this pattern's niche.
    /// Derived from the Lotka-Volterra model.
    pub edge_estimate: f64,
}

impl PatternFitness {
    pub fn new() -> Self {
        Self {
            accuracy: ExponentialAverage::new(0.05),
            information_gain: ExponentialAverage::new(0.05),
            compute_cost: 1.0,
            niche_affinities: HashMap::new(),
            composite_fitness: 0.0,
            edge_estimate: 1.0,
        }
    }

    /// Recompute composite fitness from components.
    /// Guard against division by zero on compute cost.
    pub fn recompute(&mut self) {
        let acc = self.accuracy.value();
        let ig = self.information_gain.value();
        let cost = self.compute_cost.max(0.001);
        self.composite_fitness = acc * ig / cost;
    }
}
```

### Ecosystem configuration

```rust
/// All tunable evolutionary parameters, set at ecosystem creation
/// and adjustable via the Delta-tick feedback loop.
#[derive(Clone, Debug)]
pub struct EcosystemConfig {
    /// Maximum number of patterns across all niches.
    pub max_population: usize,

    /// Minimum population per niche (prevents extinction).
    pub min_population_per_niche: usize,

    /// Base mutation rate (fraction of bits flipped). Default: 0.01.
    pub base_mutation_rate: f64,

    /// Maximum mutation rate when diversity is critically low. Default: 0.10.
    pub max_mutation_rate: f64,

    /// Crossover probability per Theta tick per eligible niche. Default: 0.3.
    pub crossover_rate: f64,

    /// Selection pressure parameter (alpha in replicator equation). Default: 0.1.
    pub selection_pressure: f64,

    /// Tournament size for parent selection. Default: 4.
    pub tournament_size: usize,

    /// Kill threshold: patterns with fitness this many standard deviations below
    /// the niche mean are eligible for removal. Default: 1.0.
    pub kill_threshold_sigma: f64,

    /// Carrying capacity per niche (max patterns before competition intensifies).
    pub niche_carrying_capacities: HashMap<DeFiContext, usize>,

    /// Shannon entropy threshold below which mutation rate escalates.
    pub min_diversity_threshold: f64,

    /// Number of patterns extracted in a death testament. Default: 25.
    pub testament_size: usize,

    /// Mutation rate multiplier applied during post-inheritance REM. Default: 5.0.
    pub post_inheritance_mutation_boost: f64,

    /// Minimum Hamming distance for immigration diversity bonus. Default: 0.3.
    pub immigration_novelty_threshold: f64,

    /// Maximum patterns emitted per Delta cycle via Styx. Default: 5.
    pub max_emigration_per_cycle: usize,

    /// Fitness percentile threshold for emigration eligibility. Default: 0.90.
    pub emigration_fitness_percentile: f64,

    /// Lotka-Volterra parameters per niche.
    pub predator_prey_params: HashMap<DeFiContext, LotkaVolterraParams>,

    /// Dimension of hypervectors in bits. Default: 10_240.
    pub hv_dimension: usize,
}

/// Lotka-Volterra parameters for a single niche.
#[derive(Clone, Debug)]
pub struct LotkaVolterraParams {
    /// Pattern growth rate when edge exists (alpha).
    pub growth_rate: f64,
    /// Natural pattern death rate (beta).
    pub death_rate: f64,
    /// Edge regeneration rate (gamma).
    pub edge_regen_rate: f64,
    /// Edge carrying capacity (K).
    pub edge_carrying_capacity: f64,
    /// Edge consumption rate per pattern (delta).
    pub edge_consumption_rate: f64,
}

impl Default for EcosystemConfig {
    fn default() -> Self {
        let mut niche_caps = HashMap::new();
        niche_caps.insert(DeFiContext::Swap, 40);
        niche_caps.insert(DeFiContext::LiquidityProvision, 30);
        niche_caps.insert(DeFiContext::Lending, 25);
        niche_caps.insert(DeFiContext::Vault, 20);
        niche_caps.insert(DeFiContext::Perpetual, 25);

        let default_lv = LotkaVolterraParams {
            growth_rate: 0.1,
            death_rate: 0.02,
            edge_regen_rate: 0.05,
            edge_carrying_capacity: 1.0,
            edge_consumption_rate: 0.01,
        };

        let mut lv_params = HashMap::new();
        for ctx in &[
            DeFiContext::Swap,
            DeFiContext::LiquidityProvision,
            DeFiContext::Lending,
            DeFiContext::Vault,
            DeFiContext::Perpetual,
        ] {
            lv_params.insert(*ctx, default_lv.clone());
        }

        Self {
            max_population: 200,
            min_population_per_niche: 5,
            base_mutation_rate: 0.01,
            max_mutation_rate: 0.10,
            crossover_rate: 0.3,
            selection_pressure: 0.1,
            tournament_size: 4,
            kill_threshold_sigma: 1.0,
            niche_carrying_capacities: niche_caps,
            min_diversity_threshold: 2.0,
            testament_size: 25,
            post_inheritance_mutation_boost: 5.0,
            immigration_novelty_threshold: 0.3,
            max_emigration_per_cycle: 5,
            emigration_fitness_percentile: 0.90,
            predator_prey_params: lv_params,
            hv_dimension: 10_240,
        }
    }
}
```

### Pattern matching output

```rust
/// Result of evaluating a pattern against a DeFi observation.
/// This is what the pattern "sees" and "predicts" during a Gamma tick.
#[derive(Clone, Debug)]
pub struct PatternMatch {
    /// Which pattern produced this match.
    pub pattern_id: PatternId,

    /// Hamming similarity between the observation's HDC encoding
    /// and the pattern's hypervector. Range [0.0, 1.0].
    pub similarity: f64,

    /// The pattern's predicted outcome, encoded as a direction
    /// (positive = price/metric increases, negative = decreases)
    /// and magnitude (expected size of the move).
    pub prediction: Prediction,

    /// Confidence in the prediction, derived from the pattern's
    /// historical accuracy and the similarity score.
    pub confidence: f64,

    /// The DeFi context in which this match occurred.
    pub context: DeFiContext,

    /// Tick at which the prediction was made (for later resolution).
    pub tick: u64,
}

#[derive(Clone, Debug)]
pub struct Prediction {
    pub direction: f64,
    pub magnitude: f64,
    pub horizon_ticks: u32,
}

/// Death testament: the fittest patterns extracted from a dying Golem.
#[derive(Clone, Debug)]
pub struct PatternTestament {
    pub patterns: Vec<PatternOrganism>,
    pub ecosystem_metadata: TestamentMetadata,
}

/// Metadata about the ecosystem state at the time of death.
/// Helps successor Golems contextualize the inherited patterns.
#[derive(Clone, Debug)]
pub struct TestamentMetadata {
    pub total_generations_evolved: u64,
    pub final_diversity_index: f64,
    pub final_mean_fitness: f64,
    pub niche_coverage: HashMap<DeFiContext, f64>,
    pub dominant_niche: DeFiContext,
    pub golem_lifetime_ticks: u64,
}
```

### Ecosystem census

```rust
/// Population snapshot computed at every Delta tick.
/// This is the ecosystem's self-portrait: diversity, fitness distribution,
/// niche coverage, and evolutionary velocity.
#[derive(Clone, Debug)]
pub struct EcosystemCensus {
    /// Total living patterns.
    pub total_population: usize,

    /// Population per niche.
    pub niche_populations: HashMap<DeFiContext, usize>,

    /// Shannon diversity index across all patterns.
    /// H = -sum(p_i * ln(p_i)) where p_i = 1/N for each unique pattern.
    /// Higher H means more diverse population.
    pub diversity_index: f64,

    /// Fitness variance across the population (Fisher's theorem diagnostic).
    /// High variance = fast evolution. Near-zero variance = stagnation.
    pub fitness_variance: f64,

    /// Mean fitness across the population.
    pub mean_fitness: f64,

    /// Per-niche coverage score: fraction of the niche's carrying capacity
    /// that is filled with patterns above the fitness threshold.
    pub niche_coverage: HashMap<DeFiContext, f64>,

    /// Estimated remaining edge per niche from Lotka-Volterra model.
    pub edge_estimates: HashMap<DeFiContext, f64>,

    /// Current adaptive mutation rate.
    pub current_mutation_rate: f64,

    /// Births and deaths since last census.
    pub births_since_last: u64,
    pub deaths_since_last: u64,

    /// Immigrants received and emigrants sent since last census.
    pub immigrants_since_last: u64,
    pub emigrants_since_last: u64,

    /// Price equation decomposition.
    pub price_selection_term: f64,
    pub price_transmission_term: f64,

    /// Generation statistics.
    pub max_generation: u32,
    pub mean_generation: f64,
}
```

### The ecosystem itself

```rust
/// The living ecosystem of TA patterns.
///
/// This struct owns all pattern organisms, manages their lifecycle,
/// integrates with the Golem's heartbeat, and communicates with
/// the Styx relay for inter-Golem pattern migration.
pub struct PatternEcosystem {
    /// All living patterns, keyed by ID.
    patterns: HashMap<PatternId, PatternOrganism>,

    /// Monotonic ID counter.
    next_id: u64,

    /// Configuration (evolutionary parameters).
    config: EcosystemConfig,

    /// Patterns indexed by niche for fast niche-local operations.
    niche_index: HashMap<DeFiContext, Vec<PatternId>>,

    /// Pending predictions awaiting resolution.
    pending_predictions: VecDeque<PatternMatch>,

    /// Lotka-Volterra edge estimates per niche, updated at Delta ticks.
    edge_state: HashMap<DeFiContext, EdgeState>,

    /// Immigration quarantine buffer.
    quarantine: VecDeque<QuarantinedPattern>,

    /// Emigration buffer: patterns selected for Styx broadcast.
    emigration_buffer: Vec<PatternOrganism>,

    /// Current adaptive mutation rate (adjusted based on diversity).
    current_mutation_rate: f64,

    /// Census history for trend analysis.
    census_history: VecDeque<EcosystemCensus>,

    /// Tick counters for lifecycle tracking.
    births_since_census: u64,
    deaths_since_census: u64,
    immigrants_since_census: u64,
    emigrants_since_census: u64,
}

/// Per-niche edge tracking for Lotka-Volterra dynamics.
#[derive(Clone, Debug)]
struct EdgeState {
    /// Current estimated edge magnitude.
    edge: f64,
    /// Current effective pattern population exploiting this edge.
    population: f64,
    /// Parameters for this niche.
    params: LotkaVolterraParams,
}

/// A pattern in quarantine awaiting local evaluation before admission.
#[derive(Clone, Debug)]
struct QuarantinedPattern {
    pattern: PatternOrganism,
    evaluations: Vec<f64>,
    ticks_in_quarantine: u32,
    source_golem: Option<u64>,
}

impl PatternEcosystem {
    pub fn new(config: EcosystemConfig) -> Self {
        let mut edge_state = HashMap::new();
        for (ctx, params) in &config.predator_prey_params {
            edge_state.insert(
                *ctx,
                EdgeState {
                    edge: params.edge_carrying_capacity,
                    population: 0.0,
                    params: params.clone(),
                },
            );
        }

        Self {
            patterns: HashMap::new(),
            next_id: 0,
            config,
            niche_index: HashMap::new(),
            pending_predictions: VecDeque::new(),
            edge_state,
            quarantine: VecDeque::new(),
            emigration_buffer: Vec::new(),
            current_mutation_rate: 0.01,
            census_history: VecDeque::new(),
            births_since_census: 0,
            deaths_since_census: 0,
            immigrants_since_census: 0,
            emigrants_since_census: 0,
        }
    }

    /// Allocate a new unique pattern ID.
    fn next_id(&mut self) -> PatternId {
        let id = PatternId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Number of u64 words needed for the configured hypervector dimension.
    fn hv_words(&self) -> usize {
        (self.config.hv_dimension + 63) / 64
    }

    // ---------------------------------------------------------------
    // Heartbeat integration
    // ---------------------------------------------------------------

    /// Gamma tick: evaluate patterns against the current observation.
    ///
    /// Each pattern that won an evaluation slot computes its similarity
    /// to the observation's HDC encoding and produces a prediction.
    /// Predictions are stored for later resolution at the Theta tick.
    pub fn gamma_tick(
        &mut self,
        obs_hv: &[u64],
        context: DeFiContext,
        current_tick: u64,
        evaluation_slots: &[PatternId],
    ) -> Vec<PatternMatch> {
        let mut matches = Vec::with_capacity(evaluation_slots.len());

        for &pid in evaluation_slots {
            let pattern = match self.patterns.get_mut(&pid) {
                Some(p) => p,
                None => continue,
            };

            let similarity = hamming_similarity(&pattern.hv, obs_hv);
            pattern.total_evaluations += 1;

            // Prediction: direction and magnitude derived from similarity
            // and the pattern's historical behavior in this context.
            let niche_affinity = pattern
                .fitness
                .niche_affinities
                .get(&context)
                .copied()
                .unwrap_or(0.5);

            let confidence = similarity * niche_affinity;

            let prediction = Prediction {
                // Direction: positive similarity above threshold suggests
                // the pattern's historical outcome direction.
                direction: if similarity > 0.6 { 1.0 } else { -1.0 },
                magnitude: (similarity - 0.5).abs(),
                horizon_ticks: 3, // resolve after 3 Theta ticks
            };

            let pm = PatternMatch {
                pattern_id: pid,
                similarity,
                prediction,
                confidence,
                context,
                tick: current_tick,
            };

            matches.push(pm.clone());
            self.pending_predictions.push_back(pm);
        }

        matches
    }

    /// Theta tick: resolve predictions, update fitness, reproduce, kill.
    pub fn theta_tick(&mut self, current_tick: u64, rng: &mut impl rand::Rng) {
        // Phase 1: Resolve pending predictions whose horizon has elapsed.
        self.resolve_predictions(current_tick);

        // Phase 2: Recompute composite fitness for all patterns.
        let ids: Vec<PatternId> = self.patterns.keys().copied().collect();
        for id in &ids {
            if let Some(p) = self.patterns.get_mut(id) {
                p.fitness.recompute();
            }
        }

        // Phase 3: Reproduction in each niche.
        let niches: Vec<DeFiContext> = self.niche_index.keys().copied().collect();
        for niche in niches {
            self.reproduce_in_niche(niche, rng);
        }

        // Phase 4: Kill unfit patterns.
        self.kill_unfit();

        // Phase 5: Process quarantine buffer.
        self.process_quarantine();
    }

    /// Delta tick: full ecosystem census and Lotka-Volterra update.
    pub fn delta_tick(&mut self) -> EcosystemCensus {
        // Update predator-prey dynamics.
        self.update_lotka_volterra();

        // Adapt mutation rate based on diversity.
        self.adapt_mutation_rate();

        // Select emigrants for Styx broadcast.
        self.select_emigrants();

        // Build census.
        let census = self.build_census();

        // Store census in history (keep last 20).
        self.census_history.push_back(census.clone());
        if self.census_history.len() > 20 {
            self.census_history.pop_front();
        }

        // Reset counters.
        self.births_since_census = 0;
        self.deaths_since_census = 0;
        self.immigrants_since_census = 0;
        self.emigrants_since_census = 0;

        census
    }

    // ---------------------------------------------------------------
    // Prediction resolution
    // ---------------------------------------------------------------

    fn resolve_predictions(&mut self, current_tick: u64) {
        let mut resolved = Vec::new();

        while let Some(pred) = self.pending_predictions.front() {
            if current_tick < pred.tick + pred.prediction.horizon_ticks as u64 {
                break; // not yet due
            }
            resolved.push(self.pending_predictions.pop_front().unwrap());
        }

        for pred in resolved {
            // In a real implementation, the outcome would come from
            // observed market data. Here we model it as a callback
            // that the Theta tick provides.
            //
            // For now: accuracy = 1.0 if prediction direction matched
            // observed direction, 0.0 otherwise. The actual implementation
            // would use a continuous accuracy measure.
            if let Some(pattern) = self.patterns.get_mut(&pred.pattern_id) {
                // Placeholder: actual outcome resolution happens via
                // the observation pipeline. The accuracy value would be
                // computed from comparing pred.prediction against what
                // actually happened in the market.
                let outcome_accuracy = 0.5; // placeholder
                pattern.fitness.accuracy.update(outcome_accuracy);

                let info_gain = (pred.confidence - 0.5).abs() * 2.0;
                pattern.fitness.information_gain.update(info_gain);

                // Update niche affinity.
                let current = pattern
                    .fitness
                    .niche_affinities
                    .entry(pred.context)
                    .or_insert(0.5);
                *current = 0.95 * *current + 0.05 * outcome_accuracy;
            }
        }
    }

    // ---------------------------------------------------------------
    // Reproduction
    // ---------------------------------------------------------------

    /// Attempt reproduction within a niche if population is below carrying capacity.
    fn reproduce_in_niche(&mut self, niche: DeFiContext, rng: &mut impl rand::Rng) {
        let carrying_capacity = self
            .config
            .niche_carrying_capacities
            .get(&niche)
            .copied()
            .unwrap_or(20);

        let current_pop = self
            .niche_index
            .get(&niche)
            .map(|v| v.len())
            .unwrap_or(0);

        if current_pop >= carrying_capacity {
            return;
        }

        // Probability check: reproduction doesn't happen every tick.
        let roll: f64 = rng.gen();
        if roll > self.config.crossover_rate {
            return;
        }

        // Select two parents via tournament selection.
        let parent_a = match self.tournament_selection(niche, rng) {
            Some(id) => id,
            None => return,
        };
        let parent_b = match self.tournament_selection(niche, rng) {
            Some(id) => id,
            None => return,
        };

        if parent_a == parent_b {
            return; // no self-fertilization
        }

        self.crossover(parent_a, parent_b, niche, rng);
    }

    /// Tournament selection: pick tournament_size random patterns from
    /// the niche, return the fittest.
    fn tournament_selection(
        &self,
        niche: DeFiContext,
        rng: &mut impl rand::Rng,
    ) -> Option<PatternId> {
        let niche_patterns = self.niche_index.get(&niche)?;
        if niche_patterns.len() < 2 {
            return None;
        }

        let k = self.config.tournament_size.min(niche_patterns.len());
        let mut best_id = None;
        let mut best_fitness = f64::NEG_INFINITY;

        for _ in 0..k {
            let idx = rng.gen_range(0..niche_patterns.len());
            let pid = niche_patterns[idx];
            if let Some(p) = self.patterns.get(&pid) {
                if p.fitness.composite_fitness > best_fitness {
                    best_fitness = p.fitness.composite_fitness;
                    best_id = Some(pid);
                }
            }
        }

        best_id
    }

    /// Produce offspring from two parents via weighted HDC crossover.
    fn crossover(
        &mut self,
        parent_a: PatternId,
        parent_b: PatternId,
        niche: DeFiContext,
        rng: &mut impl rand::Rng,
    ) -> Option<PatternId> {
        let (hv_a, fitness_a, gen_a) = {
            let p = self.patterns.get(&parent_a)?;
            (p.hv.clone(), p.fitness.composite_fitness, p.generation)
        };
        let (hv_b, fitness_b, gen_b) = {
            let p = self.patterns.get(&parent_b)?;
            (p.hv.clone(), p.fitness.composite_fitness, p.generation)
        };

        // Weight by fitness. Guard against both being zero.
        let total_fitness = (fitness_a + fitness_b).max(0.001);
        let weight_a = fitness_a / total_fitness;

        let words = self.hv_words();
        let mut child_hv = vec![0u64; words];

        for i in 0..words {
            // For each bit, choose from parent A with probability weight_a.
            // Implementation: generate a random mask where each bit is 1
            // with probability weight_a, then blend.
            let mask = weighted_random_mask(weight_a, rng);
            child_hv[i] = (hv_a[i] & mask) | (hv_b[i] & !mask);
        }

        let child_id = self.next_id();
        let child = PatternOrganism {
            id: child_id,
            hv: child_hv,
            fitness: PatternFitness::new(),
            niche,
            lineage: vec![parent_a, parent_b],
            generation: gen_a.max(gen_b) + 1,
            born_at_tick: 0, // set by caller
            total_evaluations: 0,
            reproduction_count: 0,
        };

        self.insert_pattern(child);

        // Increment parents' reproduction count.
        if let Some(p) = self.patterns.get_mut(&parent_a) {
            p.reproduction_count += 1;
        }
        if let Some(p) = self.patterns.get_mut(&parent_b) {
            p.reproduction_count += 1;
        }

        self.births_since_census += 1;
        Some(child_id)
    }

    /// Insert a pattern into the ecosystem and update niche index.
    fn insert_pattern(&mut self, pattern: PatternOrganism) {
        let niche = pattern.niche;
        let id = pattern.id;
        self.patterns.insert(id, pattern);
        self.niche_index.entry(niche).or_default().push(id);
    }

    /// Remove a pattern from the ecosystem and niche index.
    fn remove_pattern(&mut self, id: PatternId) {
        if let Some(pattern) = self.patterns.remove(&id) {
            if let Some(niche_vec) = self.niche_index.get_mut(&pattern.niche) {
                niche_vec.retain(|&pid| pid != id);
            }
        }
    }

    // ---------------------------------------------------------------
    // Mutation
    // ---------------------------------------------------------------

    /// Mutate a pattern by XOR-ing with a random noise mask.
    /// The mutation rate determines what fraction of bits get flipped.
    fn mutate(&mut self, pattern_id: PatternId, rate: f64, rng: &mut impl rand::Rng) {
        let words = self.hv_words();
        let bits_to_flip = (rate * self.config.hv_dimension as f64) as usize;

        if let Some(pattern) = self.patterns.get_mut(&pattern_id) {
            let noise = generate_sparse_noise(words, bits_to_flip, rng);
            for i in 0..words.min(pattern.hv.len()) {
                pattern.hv[i] ^= noise[i];
            }
        }
    }

    // ---------------------------------------------------------------
    // Selection (killing unfit patterns)
    // ---------------------------------------------------------------

    /// Remove patterns with fitness below niche_mean - kill_threshold * niche_std.
    /// Respects the minimum population constraint per niche.
    fn kill_unfit(&mut self) {
        let niches: Vec<DeFiContext> = self.niche_index.keys().copied().collect();

        for niche in niches {
            let niche_ids: Vec<PatternId> = self
                .niche_index
                .get(&niche)
                .cloned()
                .unwrap_or_default();

            if niche_ids.len() <= self.config.min_population_per_niche {
                continue;
            }

            // Compute niche fitness statistics.
            let fitnesses: Vec<f64> = niche_ids
                .iter()
                .filter_map(|id| self.patterns.get(id))
                .map(|p| p.fitness.composite_fitness)
                .collect();

            if fitnesses.is_empty() {
                continue;
            }

            let mean = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
            let variance = fitnesses.iter().map(|f| (f - mean).powi(2)).sum::<f64>()
                / fitnesses.len() as f64;
            let std_dev = variance.sqrt();

            let threshold = mean - self.config.kill_threshold_sigma * std_dev;

            // Collect patterns below threshold, but keep minimum population.
            let mut to_kill: Vec<(PatternId, f64)> = niche_ids
                .iter()
                .filter_map(|&id| {
                    self.patterns.get(&id).and_then(|p| {
                        if p.fitness.composite_fitness < threshold {
                            Some((id, p.fitness.composite_fitness))
                        } else {
                            None
                        }
                    })
                })
                .collect();

            // Sort by fitness ascending (kill the worst first).
            to_kill.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            let max_kills =
                niche_ids.len().saturating_sub(self.config.min_population_per_niche);
            let kill_count = to_kill.len().min(max_kills);

            for (id, _) in to_kill.into_iter().take(kill_count) {
                self.remove_pattern(id);
                self.deaths_since_census += 1;
            }
        }
    }

    // ---------------------------------------------------------------
    // Dream integration
    // ---------------------------------------------------------------

    /// NREM dream: consolidate short-term fitness into long-term averages.
    /// This is passive memory consolidation -- no new patterns are created.
    pub fn dream_nrem(&mut self) {
        // During NREM, we replay recent fitness history and decay
        // the exponential averages toward their long-term values.
        // The effect is smoothing: short-term noise gets averaged out.
        for pattern in self.patterns.values_mut() {
            // Reduce the alpha (learning rate) temporarily to smooth values.
            // This simulates the consolidation effect of NREM sleep.
            let smoothed_accuracy = pattern.fitness.accuracy.value();
            let smoothed_ig = pattern.fitness.information_gain.value();

            // Nothing changes the values here -- NREM just "locks in"
            // the current state by allowing the exponential averages
            // to settle without new input.
            pattern.fitness.accuracy.update(smoothed_accuracy);
            pattern.fitness.information_gain.update(smoothed_ig);
            pattern.fitness.recompute();
        }
    }

    /// REM dream: primary mutation mechanism.
    ///
    /// For each pattern, generate a mutant and evaluate it counterfactually
    /// against a replay buffer. If the mutant outperforms the parent,
    /// it enters the population. Most mutants are discarded.
    pub fn dream_rem(
        &mut self,
        replay_buffer: &[ReplayEntry],
        rng: &mut impl rand::Rng,
    ) {
        let rate = self.current_mutation_rate;
        let pattern_ids: Vec<PatternId> = self.patterns.keys().copied().collect();
        let words = self.hv_words();

        let mut new_patterns = Vec::new();

        for pid in pattern_ids {
            let parent = match self.patterns.get(&pid) {
                Some(p) => p,
                None => continue,
            };

            // Generate mutant hypervector.
            let bits_to_flip = (rate * self.config.hv_dimension as f64) as usize;
            let noise = generate_sparse_noise(words, bits_to_flip, rng);
            let mut mutant_hv = parent.hv.clone();
            for i in 0..words.min(mutant_hv.len()) {
                mutant_hv[i] ^= noise[i];
            }

            // Counterfactual evaluation against replay buffer.
            let parent_score = counterfactual_fitness(&parent.hv, replay_buffer);
            let mutant_score = counterfactual_fitness(&mutant_hv, replay_buffer);

            // Mutant must beat parent by a margin to be admitted.
            // This prevents neutral drift from inflating population.
            if mutant_score > parent_score * 1.05 {
                let mutant_id = self.next_id();
                let mutant = PatternOrganism {
                    id: mutant_id,
                    hv: mutant_hv,
                    fitness: PatternFitness::new(),
                    niche: parent.niche,
                    lineage: vec![pid],
                    generation: parent.generation + 1,
                    born_at_tick: 0,
                    total_evaluations: 0,
                    reproduction_count: 0,
                };
                new_patterns.push(mutant);
            }
        }

        for mutant in new_patterns {
            // Check population cap before inserting.
            if self.patterns.len() < self.config.max_population {
                self.births_since_census += 1;
                self.insert_pattern(mutant);
            }
        }
    }

    // ---------------------------------------------------------------
    // Styx migration (horizontal gene transfer)
    // ---------------------------------------------------------------

    /// Receive immigrant patterns from Clade siblings via Styx relay.
    /// Immigrants enter quarantine for evaluation before admission.
    pub fn receive_immigrants(
        &mut self,
        patterns: Vec<PatternOrganism>,
        source_golem: Option<u64>,
    ) {
        for pattern in patterns {
            self.quarantine.push_back(QuarantinedPattern {
                pattern,
                evaluations: Vec::new(),
                ticks_in_quarantine: 0,
                source_golem,
            });
        }
    }

    /// Select top patterns for emigration via Styx broadcast.
    fn select_emigrants(&mut self) {
        self.emigration_buffer.clear();

        // Gather all patterns, sort by fitness descending.
        let mut candidates: Vec<(PatternId, f64)> = self
            .patterns
            .iter()
            .map(|(id, p)| (*id, p.fitness.composite_fitness))
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Find the fitness threshold at the configured percentile.
        if candidates.is_empty() {
            return;
        }

        let percentile_idx =
            ((self.config.emigration_fitness_percentile * candidates.len() as f64) as usize)
                .min(candidates.len() - 1);
        let threshold = candidates[percentile_idx].1;

        let mut count = 0;
        for (id, fitness) in &candidates {
            if count >= self.config.max_emigration_per_cycle {
                break;
            }
            if *fitness >= threshold {
                if let Some(p) = self.patterns.get(id) {
                    self.emigration_buffer.push(p.clone());
                    count += 1;
                }
            }
        }

        self.emigrants_since_census += count as u64;
    }

    /// Get the current emigration buffer for Styx transmission.
    pub fn take_emigrants(&mut self) -> Vec<PatternOrganism> {
        std::mem::take(&mut self.emigration_buffer)
    }

    /// Process quarantine: evaluate quarantined patterns and admit or reject.
    fn process_quarantine(&mut self) {
        let mut admitted = Vec::new();
        let mut remaining = VecDeque::new();

        while let Some(mut qp) = self.quarantine.pop_front() {
            qp.ticks_in_quarantine += 1;

            // Need at least 10 evaluation ticks before deciding.
            if qp.ticks_in_quarantine < 10 {
                remaining.push_back(qp);
                continue;
            }

            // Compute mean evaluation score.
            if qp.evaluations.is_empty() {
                remaining.push_back(qp);
                continue;
            }

            let mean_eval =
                qp.evaluations.iter().sum::<f64>() / qp.evaluations.len() as f64;

            // Check novelty: is this pattern sufficiently different from
            // existing population?
            let min_distance = self
                .patterns
                .values()
                .map(|p| 1.0 - hamming_similarity(&p.hv, &qp.pattern.hv))
                .fold(f64::INFINITY, f64::min);

            let novelty_bonus = if min_distance > self.config.immigration_novelty_threshold {
                0.1
            } else {
                0.0
            };

            // Admit if evaluation score + novelty bonus exceeds threshold.
            if mean_eval + novelty_bonus > 0.5 {
                admitted.push(qp.pattern);
            }
            // Otherwise: silently rejected (pattern dies in quarantine).
        }

        self.quarantine = remaining;

        for pattern in admitted {
            if self.patterns.len() < self.config.max_population {
                self.immigrants_since_census += 1;
                self.insert_pattern(pattern);
            }
        }
    }

    // ---------------------------------------------------------------
    // Death testament
    // ---------------------------------------------------------------

    /// Extract the fittest patterns for the Golem's death testament.
    /// Called by the Thanatopsis subsystem when the Golem is dying.
    pub fn death_testament(&self) -> PatternTestament {
        let mut all_patterns: Vec<&PatternOrganism> = self.patterns.values().collect();
        all_patterns.sort_by(|a, b| {
            b.fitness
                .composite_fitness
                .partial_cmp(&a.fitness.composite_fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let n = self.config.testament_size.min(all_patterns.len());
        let selected: Vec<PatternOrganism> =
            all_patterns[..n].iter().map(|p| (*p).clone()).collect();

        let metadata = TestamentMetadata {
            total_generations_evolved: all_patterns
                .iter()
                .map(|p| p.generation as u64)
                .max()
                .unwrap_or(0),
            final_diversity_index: self.compute_diversity_index(),
            final_mean_fitness: self.compute_mean_fitness(),
            niche_coverage: self.compute_niche_coverage(),
            dominant_niche: self.dominant_niche(),
            golem_lifetime_ticks: 0, // set by caller
        };

        PatternTestament {
            patterns: selected,
            ecosystem_metadata: metadata,
        }
    }

    /// Inherit patterns from a predecessor Golem's death testament.
    /// Seeds the population and triggers an elevated-mutation REM cycle.
    pub fn inherit_testament(
        &mut self,
        testament: &PatternTestament,
        rng: &mut impl rand::Rng,
    ) {
        for pattern in &testament.patterns {
            let mut inherited = pattern.clone();
            inherited.id = self.next_id();
            self.insert_pattern(inherited);
        }

        // Elevated mutation to restore diversity lost in the bottleneck.
        let boosted_rate =
            self.config.base_mutation_rate * self.config.post_inheritance_mutation_boost;
        let pattern_ids: Vec<PatternId> = self.patterns.keys().copied().collect();
        for pid in pattern_ids {
            self.mutate(pid, boosted_rate, rng);
        }
    }

    // ---------------------------------------------------------------
    // Lotka-Volterra dynamics
    // ---------------------------------------------------------------

    /// Update edge estimates using discretized Lotka-Volterra equations.
    fn update_lotka_volterra(&mut self) {
        let dt = 1.0; // one Delta tick

        for (niche, state) in self.edge_state.iter_mut() {
            let p = self
                .niche_index
                .get(niche)
                .map(|v| v.len() as f64)
                .unwrap_or(0.0);

            let e = state.edge;
            let params = &state.params;

            // dP/dt = alpha * P * E - beta * P
            // We don't actually change P here (that's governed by reproduction/death).
            // We only update the population estimate for the model.
            state.population = p;

            // dE/dt = gamma * E * (1 - E/K) - delta * P * E
            let de = params.edge_regen_rate * e * (1.0 - e / params.edge_carrying_capacity)
                - params.edge_consumption_rate * p * e;
            state.edge = (e + de * dt).clamp(0.0, params.edge_carrying_capacity);
        }

        // Push edge estimates back into pattern fitness.
        for (niche, state) in &self.edge_state {
            if let Some(niche_ids) = self.niche_index.get(niche) {
                for pid in niche_ids {
                    if let Some(pattern) = self.patterns.get_mut(pid) {
                        pattern.fitness.edge_estimate = state.edge;
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------
    // Adaptive mutation rate
    // ---------------------------------------------------------------

    /// Adjust mutation rate based on population diversity.
    /// Low diversity -> higher mutation. High diversity -> baseline mutation.
    fn adapt_mutation_rate(&mut self) {
        let diversity = self.compute_diversity_index();

        if diversity < self.config.min_diversity_threshold {
            // Scale mutation rate inversely with diversity.
            let deficit_ratio = 1.0 - (diversity / self.config.min_diversity_threshold);
            self.current_mutation_rate = self.config.base_mutation_rate
                + deficit_ratio
                    * (self.config.max_mutation_rate - self.config.base_mutation_rate);
        } else {
            self.current_mutation_rate = self.config.base_mutation_rate;
        }
    }

    // ---------------------------------------------------------------
    // Ecosystem metrics
    // ---------------------------------------------------------------

    /// Shannon diversity index: H = -sum(p_i * ln(p_i)).
    /// Uses niche frequencies as the probability distribution.
    pub fn compute_diversity_index(&self) -> f64 {
        let total = self.patterns.len() as f64;
        if total <= 1.0 {
            return 0.0;
        }

        // Compute frequency of each unique pattern based on niche + fitness bin.
        // For simplicity, we use niche distribution as the diversity proxy.
        let mut h = 0.0;
        for ids in self.niche_index.values() {
            let p_i = ids.len() as f64 / total;
            if p_i > 0.0 {
                h -= p_i * p_i.ln();
            }
        }
        h
    }

    /// Fitness variance across all patterns (Fisher's theorem diagnostic).
    pub fn compute_fitness_variance(&self) -> f64 {
        let fitnesses: Vec<f64> = self
            .patterns
            .values()
            .map(|p| p.fitness.composite_fitness)
            .collect();

        if fitnesses.len() < 2 {
            return 0.0;
        }

        let mean = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        fitnesses.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / fitnesses.len() as f64
    }

    /// Mean fitness across the population.
    fn compute_mean_fitness(&self) -> f64 {
        if self.patterns.is_empty() {
            return 0.0;
        }
        self.patterns
            .values()
            .map(|p| p.fitness.composite_fitness)
            .sum::<f64>()
            / self.patterns.len() as f64
    }

    /// Per-niche coverage: fraction of carrying capacity filled by
    /// patterns above the minimum fitness threshold.
    fn compute_niche_coverage(&self) -> HashMap<DeFiContext, f64> {
        let mut coverage = HashMap::new();
        let mean = self.compute_mean_fitness();

        for (niche, cap) in &self.config.niche_carrying_capacities {
            let viable_count = self
                .niche_index
                .get(niche)
                .map(|ids| {
                    ids.iter()
                        .filter(|id| {
                            self.patterns
                                .get(id)
                                .map(|p| p.fitness.composite_fitness > mean * 0.5)
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0);

            coverage.insert(*niche, viable_count as f64 / *cap as f64);
        }

        coverage
    }

    /// Which niche has the most patterns?
    fn dominant_niche(&self) -> DeFiContext {
        self.niche_index
            .iter()
            .max_by_key(|(_, ids)| ids.len())
            .map(|(niche, _)| *niche)
            .unwrap_or(DeFiContext::Swap)
    }

    // ---------------------------------------------------------------
    // Price equation decomposition
    // ---------------------------------------------------------------

    /// Selection term of the Price equation: Cov(W_i, x_i) / W_bar.
    ///
    /// Measures how much of the fitness improvement comes from fitter
    /// patterns increasing in frequency. Always non-negative under
    /// natural selection. Tracks whether selection is "working" --
    /// whether the population structure is shifting toward higher fitness.
    fn compute_price_selection_term(&self) -> f64 {
        let n = self.patterns.len() as f64;
        if n < 2.0 {
            return 0.0;
        }

        let mean_fitness = self.compute_mean_fitness();
        if mean_fitness.abs() < 1e-12 {
            return 0.0;
        }

        // x_i = 1/N for each pattern (uniform frequency, since each pattern
        // is a distinct individual). Covariance of fitness with frequency
        // reduces to Var(W)/N in the uniform case.
        let var_w = self.compute_fitness_variance();
        var_w / mean_fitness
    }

    /// Transmission term of the Price equation: E(W_i * delta_i) / W_bar.
    ///
    /// Measures fitness change within lineages due to mutation and crossover.
    /// Computed by comparing each pattern's fitness to its parents' fitness
    /// at the same generation. Positive means mutations are improving fitness.
    /// Negative means mutations are degrading it.
    fn compute_price_transmission_term(&self) -> f64 {
        let mean_fitness = self.compute_mean_fitness();
        if mean_fitness.abs() < 1e-12 || self.patterns.is_empty() {
            return 0.0;
        }

        let mut weighted_delta_sum = 0.0;
        let mut count = 0;

        for pattern in self.patterns.values() {
            if pattern.lineage.is_empty() {
                continue; // founder, no parent to compare against
            }

            // Find the best parent's current fitness (if the parent still lives).
            let parent_fitness: Option<f64> = pattern
                .lineage
                .iter()
                .filter_map(|pid| self.patterns.get(pid))
                .map(|p| p.fitness.composite_fitness)
                .reduce(f64::max);

            if let Some(pf) = parent_fitness {
                let delta = pattern.fitness.composite_fitness - pf;
                weighted_delta_sum += pattern.fitness.composite_fitness * delta;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        (weighted_delta_sum / count as f64) / mean_fitness
    }

    /// Build a full census from current ecosystem state.
    fn build_census(&self) -> EcosystemCensus {
        let niche_populations: HashMap<DeFiContext, usize> = self
            .niche_index
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect();

        let fitnesses: Vec<f64> = self
            .patterns
            .values()
            .map(|p| p.fitness.composite_fitness)
            .collect();

        let mean_fitness = if fitnesses.is_empty() {
            0.0
        } else {
            fitnesses.iter().sum::<f64>() / fitnesses.len() as f64
        };

        let generations: Vec<u32> = self.patterns.values().map(|p| p.generation).collect();

        EcosystemCensus {
            total_population: self.patterns.len(),
            niche_populations,
            diversity_index: self.compute_diversity_index(),
            fitness_variance: self.compute_fitness_variance(),
            mean_fitness,
            niche_coverage: self.compute_niche_coverage(),
            edge_estimates: self
                .edge_state
                .iter()
                .map(|(k, v)| (*k, v.edge))
                .collect(),
            current_mutation_rate: self.current_mutation_rate,
            births_since_last: self.births_since_census,
            deaths_since_last: self.deaths_since_census,
            immigrants_since_last: self.immigrants_since_census,
            emigrants_since_last: self.emigrants_since_census,
            price_selection_term: self.compute_price_selection_term(),
            price_transmission_term: self.compute_price_transmission_term(),
            max_generation: generations.iter().copied().max().unwrap_or(0),
            mean_generation: if generations.is_empty() {
                0.0
            } else {
                generations.iter().map(|g| *g as f64).sum::<f64>()
                    / generations.len() as f64
            },
        }
    }
}
```

### HDC utility functions

```rust
/// Replay buffer entry for counterfactual evaluation during REM dreams.
#[derive(Clone, Debug)]
pub struct ReplayEntry {
    pub observation_hv: Vec<u64>,
    pub context: DeFiContext,
    pub outcome_direction: f64,
}

/// Hamming similarity between two hypervectors.
/// Returns a value in [0.0, 1.0] where 1.0 means identical.
///
/// Uses POPCNT intrinsic for hardware-accelerated bit counting.
/// For 10,240-bit vectors (160 u64 words), this completes in ~10ns.
fn hamming_similarity(a: &[u64], b: &[u64]) -> f64 {
    let total_bits = a.len() * 64;
    let matching_bits: u32 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (!(x ^ y)).count_ones())
        .sum();
    matching_bits as f64 / total_bits as f64
}

/// Generate a weighted random mask where each bit is 1 with probability `weight`.
///
/// For crossover: bits set to 1 select from parent A, bits set to 0 from parent B.
fn weighted_random_mask(weight: f64, rng: &mut impl rand::Rng) -> u64 {
    let mut mask = 0u64;
    for bit in 0..64 {
        let r: f64 = rng.gen();
        if r < weight {
            mask |= 1u64 << bit;
        }
    }
    mask
}

/// Generate a sparse noise mask with exactly `bits_to_flip` bits set.
///
/// Uses Fisher-Yates partial shuffle to select bit positions uniformly
/// without replacement. More efficient than setting each bit independently
/// when the mutation rate is low.
fn generate_sparse_noise(words: usize, bits_to_flip: usize, rng: &mut impl rand::Rng) -> Vec<u64> {
    let total_bits = words * 64;
    let mut noise = vec![0u64; words];

    // For low mutation rates, pick positions directly.
    // For high mutation rates (>25%), generate full random words and threshold.
    if bits_to_flip * 4 < total_bits {
        // Sparse case: pick individual positions.
        let mut positions = Vec::with_capacity(bits_to_flip);
        // Reservoir-style selection for large total_bits.
        let mut selected = std::collections::HashSet::with_capacity(bits_to_flip);
        while selected.len() < bits_to_flip.min(total_bits) {
            let pos = rng.gen_range(0..total_bits);
            if selected.insert(pos) {
                positions.push(pos);
            }
        }
        for pos in positions {
            let word = pos / 64;
            let bit = pos % 64;
            noise[word] |= 1u64 << bit;
        }
    } else {
        // Dense case: generate full random words.
        let prob = bits_to_flip as f64 / total_bits as f64;
        for word in noise.iter_mut() {
            for bit in 0..64 {
                let r: f64 = rng.gen();
                if r < prob {
                    *word |= 1u64 << bit;
                }
            }
        }
    }

    noise
}

/// Counterfactual fitness evaluation against a replay buffer.
///
/// Computes the average similarity between a pattern hypervector
/// and replay observations where the observation direction matches
/// the pattern's predicted direction. Higher score = pattern would
/// have predicted correctly more often.
fn counterfactual_fitness(hv: &[u64], replay: &[ReplayEntry]) -> f64 {
    if replay.is_empty() {
        return 0.0;
    }

    let mut total_score = 0.0;
    for entry in replay {
        let sim = hamming_similarity(hv, &entry.observation_hv);
        // Score: how well does high similarity correlate with positive outcomes?
        if sim > 0.6 && entry.outcome_direction > 0.0 {
            total_score += sim;
        } else if sim < 0.4 && entry.outcome_direction < 0.0 {
            total_score += 1.0 - sim;
        }
    }

    total_score / replay.len() as f64
}
```

### Seeding the initial population

```rust
impl PatternEcosystem {
    /// Seed the ecosystem with founder patterns.
    ///
    /// Founder patterns are generated by encoding known TA concepts
    /// (momentum, mean-reversion, liquidity events, etc.) as hypervectors
    /// using the HDC codebook from Doc 1. These are the "primordial soup"
    /// from which all future patterns descend.
    pub fn seed_founders(
        &mut self,
        founders: Vec<(Vec<u64>, DeFiContext)>,
        current_tick: u64,
    ) {
        for (hv, niche) in founders {
            let id = self.next_id();
            let pattern = PatternOrganism {
                id,
                hv,
                fitness: PatternFitness::new(),
                niche,
                lineage: Vec::new(), // no parents -- this is a founder
                generation: 0,
                born_at_tick: current_tick,
                total_evaluations: 0,
                reproduction_count: 0,
            };
            self.insert_pattern(pattern);
        }
    }
}
```

### Phylogenetic tracker

The ecosystem maintains a lightweight record of every pattern that has ever lived, even after death. This enables lineage analysis: tracing the ancestry of a successful pattern back to its founders, identifying which mutations were beneficial, and detecting lineage bottlenecks.

```rust
/// A record of a pattern's existence, kept after the pattern dies.
/// Used for phylogenetic analysis and lineage tracking.
#[derive(Clone, Debug)]
pub struct PatternFossil {
    pub id: PatternId,
    pub lineage: Vec<PatternId>,
    pub generation: u32,
    pub niche: DeFiContext,
    pub peak_fitness: f64,
    pub born_at_tick: u64,
    pub died_at_tick: Option<u64>,
    pub total_evaluations: u64,
    pub offspring_count: u32,
    pub cause_of_death: DeathCause,
}

#[derive(Clone, Debug)]
pub enum DeathCause {
    /// Fitness dropped below the kill threshold.
    Selection,
    /// Golem died, pattern was not in the testament.
    GolemDeath,
    /// Population cap exceeded, least-fit culled.
    Overcrowding,
    /// Pattern rejected during quarantine.
    QuarantineRejection,
    /// Still alive (for living patterns queried as fossils).
    Alive,
}

/// Tracks the evolutionary history of the entire ecosystem.
/// Kept in memory as a ring buffer (most recent N fossils)
/// and persisted to redb for long-term analysis.
pub struct PhylogeneticTracker {
    fossils: VecDeque<PatternFossil>,
    max_fossils: usize,
}

impl PhylogeneticTracker {
    pub fn new(max_fossils: usize) -> Self {
        Self {
            fossils: VecDeque::with_capacity(max_fossils),
            max_fossils,
        }
    }

    /// Record a pattern's death.
    pub fn record_death(&mut self, fossil: PatternFossil) {
        if self.fossils.len() >= self.max_fossils {
            self.fossils.pop_front();
        }
        self.fossils.push_back(fossil);
    }

    /// Trace a living pattern's ancestry back to its founder.
    /// Returns the chain of ancestor IDs from the founder to the pattern.
    pub fn trace_lineage(&self, pattern_id: PatternId) -> Vec<PatternId> {
        let mut chain = vec![pattern_id];
        let mut current = pattern_id;

        // Walk up the lineage tree through fossils.
        loop {
            let parent = self
                .fossils
                .iter()
                .find(|f| f.id == current)
                .and_then(|f| f.lineage.first().copied());

            match parent {
                Some(pid) => {
                    chain.push(pid);
                    current = pid;
                }
                None => break,
            }
        }

        chain.reverse();
        chain
    }

    /// Find the most reproductively successful patterns (most offspring).
    /// These are the "Genghis Khans" of the ecosystem -- patterns whose
    /// genetic material dominates the current population.
    pub fn most_prolific(&self, top_n: usize) -> Vec<&PatternFossil> {
        let mut sorted: Vec<&PatternFossil> = self.fossils.iter().collect();
        sorted.sort_by(|a, b| b.offspring_count.cmp(&a.offspring_count));
        sorted.truncate(top_n);
        sorted
    }

    /// Compute the average lifespan of patterns in a given niche.
    /// Returns None if no dead patterns exist for that niche.
    pub fn avg_lifespan(&self, niche: DeFiContext) -> Option<f64> {
        let lifespans: Vec<f64> = self
            .fossils
            .iter()
            .filter(|f| f.niche == niche)
            .filter_map(|f| {
                f.died_at_tick
                    .map(|d| (d - f.born_at_tick) as f64)
            })
            .collect();

        if lifespans.is_empty() {
            None
        } else {
            Some(lifespans.iter().sum::<f64>() / lifespans.len() as f64)
        }
    }
}
```

The phylogenetic tracker feeds into two downstream analyses. First, the Grimoire stores lineage data as part of each episode's context, enabling the Golem to reason about which pattern families have been historically successful. Second, the Delta tick census includes lineage statistics (average generation depth, number of active lineages) that the CorticalState extension can expose to other subsystems.

## Subsystem interactions [SPEC]

### Grimoire (episodic memory)

Patterns store their evolutionary metadata in the Grimoire alongside the standard episode fields. Each pattern's lineage, generation count, and fitness history become Grimoire entries tagged with the `EvolutionaryMetadata` type. When a Golem retrieves memories of a past market situation, it also retrieves the patterns that were active at that time, including how fit they were and what offspring they produced. This gives the Golem a temporal view of pattern evolution: "Last time the market looked like this, pattern P-47 was dominant. Now its offspring P-293 holds that niche."

The Grimoire's existing memetic evolution framework (Innovation 06) provides the conceptual substrate. Grimoire entries already behave as Dawkinsian replicators with fitness W = f * r * L. Pattern organisms are a specialized class of Grimoire replicator where the "entry" is a hypervector and the replication mechanism is HDC crossover rather than Styx broadcast of text.

The epistemic parasite detection from Innovation 06 applies directly. A pattern that spreads well through the Clade via Styx but consistently reduces decision quality in the Golems that adopt it is an epistemic parasite. The quarantine mechanism described above is the first line of defense. The second is the Grimoire's memetic fitness tracking: patterns that reduce their host Golem's aggregate accuracy get flagged and suppressed.

### Dream cycles (NREM and REM)

The mapping between dream cycles and ecosystem operations is precise.

NREM consolidation corresponds to fitness stabilization. During waking operation, pattern fitness fluctuates with each observation and prediction resolution. NREM smooths these fluctuations by replaying the fitness history and letting the exponential averages converge. The result is a cleaner fitness signal for selection to act on.

REM exploration corresponds to mutation and counterfactual evaluation. REM dreams already generate novel combinations by randomly perturbing existing knowledge (Innovation 06 describes this as "creative exploration"). In the pattern ecosystem, REM dreams flip bits in pattern hypervectors and evaluate the resulting mutants against replayed observations. Most mutants fail. The rare ones that outperform their parents enter the population. This is where genuine novelty emerges in the ecosystem.

The dream-frequency coupling matters. A Golem that dreams frequently (high arousal, many novel situations) generates more mutants per unit time. A Golem in a calm market dreams less and explores less. The mutation rate tracks the Golem's subjective experience of market volatility, creating an adaptive exploration schedule.

### Mortality and Thanatopsis

The Golem's three-clock mortality system creates progressive selection pressure. As economic vitality declines, the total attention budget shrinks. Fewer evaluation slots mean patterns must compete harder for the right to observe and predict. The weakest patterns lose their slots, receive no observations, and their fitness decays. By the time the Golem approaches death, its pattern population has been winnowed to a lean, high-fitness core.

The death testament preserves this core. The Thanatopsis subsystem calls `death_testament()`, extracts the top-k patterns, and packages them for transfer to successor Golems. The testament also carries metadata about the ecosystem state at death: diversity index, mean fitness, niche coverage, dominant niche. Successor Golems can use this metadata to understand what kind of market their predecessor operated in.

Across multiple Golem generations, the pattern population undergoes a ratchet effect. Each generation starts from the fitness floor established by the previous generation's best patterns, then evolves further. The ratchet slips only when market conditions change so much that the inherited patterns become irrelevant, forcing the ecosystem to explore from a lower baseline.

### Styx relay (Clade-level dynamics)

Pattern migration via Styx creates Clade-level evolutionary dynamics that operate on a different timescale than within-Golem evolution.

Within a single Golem, evolution happens at the speed of Theta ticks (minutes). A pattern can reproduce, mutate, and be selected against within a single waking cycle. This is fast, but it is limited by the Golem's own experience: patterns can only adapt to conditions the Golem has actually observed.

Across a Clade, migration happens at the speed of Delta ticks (hours). A high-fitness pattern discovered by one Clade member takes hours to propagate to siblings. But it carries information about market conditions that the receiving Golem may not have encountered. A swap-niche specialist Golem discovers a funding-rate pattern that happens to work well because swap and perp markets are correlated. It emigrates the pattern to a perp-specialist sibling, which tests it in quarantine and adopts it. The Clade benefits from distributed exploration.

The Clade topology affects migration dynamics. A fully-connected Clade (every Golem talks to every other) propagates patterns fast but risks homogenizing the population. A sparsely-connected Clade (Golems only talk to nearest neighbors) propagates patterns slowly but maintains more diversity. Innovation 08's morphogenetic specialization provides the topology: Turing patterns on the Clade graph determine which Golems specialize in which niches, and migration follows the specialization gradient.

There is a subtlety to cross-niche migration. A swap-specialist Golem might discover a pattern that works in the LP niche by accident (mutations sometimes produce cross-niche sensitivity). If that Golem has no LP observations to evaluate the pattern against, it cannot tell the pattern is useful. But a Clade sibling that specializes in LP can. The emigration mechanism sends patterns based on fitness in the source Golem's niche, but the quarantine mechanism in the destination Golem evaluates them against the destination's niche. A pattern that is mediocre in swaps but excellent in LP gets emigrated (barely) and then adopted (enthusiastically) in the right home. This is how Clade-level migration solves the niche-colonization problem faster than any single Golem could through mutation alone.

The quarantine mechanism also defends against pattern parasites. A pattern that spreads well (high fitness in its source Golem) but actually hurts decision quality in the receiving Golem (a false positive generator, or a pattern that correlates with but does not cause the signal it appears to detect) will fail quarantine evaluation. The receiving Golem evaluates the pattern against its own independent observations. If the pattern's predictions do not hold up, it is rejected. This is the evolutionary analog of the immune system rejecting incompatible tissue.

### Morphogenetic specialization (Innovation 08)

Innovation 08 uses reaction-diffusion dynamics (Turing patterns) to drive spontaneous role specialization within a Clade. Activator and inhibitor signals propagate across the Clade graph, creating stable spatial patterns where neighboring Golems specialize in complementary roles.

The pattern ecosystem interacts with morphogenetic specialization through the niche dimension. When a Golem's morphogenetic role tilts toward a particular DeFi primitive (say, LP provision), its pattern ecosystem receives more LP observations, its LP patterns get more evaluation slots, and the LP niche grows while other niches shrink. The morphogenetic signal does not directly manipulate the pattern population. It shapes the ecosystem indirectly by controlling the flow of observations and attention budget.

This creates a clean separation of concerns. Morphogenetic specialization decides what a Golem focuses on. The pattern ecosystem decides how to detect patterns within that focus. A Golem that has specialized in lending will evolve a population dominated by lending patterns, not because anyone configured it that way, but because lending observations are the primary selection pressure acting on its population.

The feedback from ecosystem to morphogenesis runs through the CorticalState extension. The `niche_coverage_fraction` signal tells the morphogenetic system which niches are well-covered and which are neglected. A Golem with excellent swap patterns but no lending patterns broadcasts low lending coverage, which the Clade's reaction-diffusion dynamics can use to recruit a neighboring Golem toward lending specialization. The ecosystem's self-reported blind spots shape the Clade's division of labor.

### Signal metabolism (Doc 3)

Patterns and signals interact at two levels.

First, patterns compete alongside signals for attention budget in the VCG auction. A pattern detecting liquidity traps competes for the same evaluation slots as a Hebbian signal tracking gas prices. The auction allocates based on expected marginal contribution, so patterns and signals coexist only if they provide complementary information.

Second, patterns consume signals as inputs. A pattern's hypervector encoding includes the raw observation data and the signal values active at the time of encoding. A pattern might encode "swap volume is above the 90th percentile AND the gas price signal is firing AND the liquidity curvature signal is not firing." The signal metabolism determines which signals exist; the pattern ecosystem determines which combinations of signals form useful patterns.

Signal speciation (Doc 3) and pattern speciation are parallel processes. When a signal forks into niche-specific variants, the patterns that consume that signal may also fork, tracking the specialized signal in each niche. The two speciation processes are coupled but not locked: a pattern can use the general version of a signal even after the signal has speciated, and vice versa.

## DeFi primitive coverage [SPEC]

Each niche has characteristic evolutionary dynamics shaped by the market structure of its primitive type.

### Swap niche

The swap niche has the highest turnover and the most competitive dynamics. Patterns here detect momentum (short-term price trends in AMM pools), mean-reversion (price returning to a moving average after displacement), liquidity traps (the pattern described in the problem section), and sandwich attack signatures.

Carrying capacity: ~40 patterns. Turnover rate: high (patterns have a half-life of roughly 2-3 Delta cycles). Edge depletion: fast (arbitrage bots adapt within hours). Lotka-Volterra parameters: high growth rate (alpha), high consumption rate (delta), moderate regeneration (gamma).

Selection pressure in the swap niche favors cheap, fast patterns. A swap pattern that takes 5ms to evaluate loses to one that takes 0.5ms, all else equal, because the cost term in the fitness function penalizes expensive patterns. The swap niche also has the strongest predator-prey oscillations: a momentum pattern that works this week attracts imitators (MEV bots adapting to the same signal) and decays next week. The patterns that survive long-term in the swap niche tend to be metamorphic: they detect not the original signal but the adaptation to the signal, staying one step ahead of the competitive response.

### LP niche

The LP niche has slower dynamics because LP strategies operate on longer timescales. Patterns here detect rebalancing opportunities (when a concentrated liquidity position should adjust its tick range), fee accumulation rates (predicting when fee income will offset impermanent loss), and just-in-time (JIT) liquidity events (large JIT positions appearing before anticipated high-volume swaps).

Carrying capacity: ~30 patterns. Turnover rate: moderate (half-life of 5-7 Delta cycles). Edge depletion: slow (LP decisions are complex enough that crowding takes longer to develop). Lotka-Volterra parameters: moderate growth, low consumption, slow regeneration.

LP patterns tend to be more complex (higher information content in their hypervectors) because LP decisions depend on more variables than swap decisions. The ecosystem naturally selects for this complexity by rewarding patterns with high information gain even when their raw accuracy is moderate.

### Lending niche

Lending patterns detect utilization rate trajectories (predicting when a market will approach full utilization), rate arbitrage opportunities (discrepancies between variable and fixed rates across protocols), and liquidation cascades (detecting conditions that will trigger mass liquidations).

Carrying capacity: ~25 patterns. Turnover rate: slow (half-life of 10+ Delta cycles). Edge depletion: very slow (lending markets change gradually). Lotka-Volterra parameters: low growth, low consumption, low regeneration, but high carrying capacity for liquidation patterns during volatile periods.

The lending niche is unusual because its carrying capacity is dynamic. During calm markets, few lending patterns justify their compute cost. During volatile markets, liquidation prediction patterns become extremely fit and the niche population explodes. The ecosystem handles this through the Lotka-Volterra model's edge term: the edge for liquidation patterns spikes during volatility and collapses during calm.

### Vault niche

Vault patterns detect harvest timing (when a yield aggregator should compound its rewards), strategy rotation (when a vault is about to switch its underlying strategy), and yield depression (when a vault's APY is declining due to inflows diluting the yield).

Carrying capacity: ~20 patterns. Turnover rate: moderate (tied to vault rebalancing frequency, typically daily to weekly). Edge depletion: moderate (vault strategies are somewhat opaque, slowing adaptation).

Vault patterns have a unique characteristic: they are often meta-patterns, detecting the behavior of other automated systems (the vault's own strategy logic) rather than organic market activity. A pattern that detects when a Yearn vault is about to rebalance is, in effect, predicting the output of another algorithm. This makes vault patterns relatively stable once they lock onto the target strategy, but brittle when the vault's strategy code is updated. A vault contract upgrade invalidates the entire pattern family overnight.

### Perpetual niche

Perpetual patterns focus on funding rate dynamics. The dominant pattern family is funding rate mean-reversion: when the funding rate deviates far from zero, it tends to revert, creating predictable cash flows for basis traders. Secondary patterns detect open interest shifts, liquidation levels, and cross-exchange basis arbitrage.

Carrying capacity: ~25 patterns. Turnover rate: moderate (funding rates cycle on 8-hour periods, but the mean-reversion dynamic is persistent). Edge depletion: moderate (many participants trade funding, but the mechanism is somewhat self-reinforcing).

The perpetual niche has the most predictable edge lifecycle because funding rate mechanics are well-understood and transparent. Every participant can see the current funding rate, the open interest, and the basis. The edge comes not from the prediction itself (everyone knows funding rates revert) but from timing: when will it revert, and how much will it overshoot? Patterns in this niche compete on precision rather than novelty. The population tends toward a small number of highly refined variants rather than a diverse population of exploratory ones.

## Cybernetic feedback loops [SPEC]

The pattern ecosystem participates in three nested feedback loops.

### Loop 1: within-Golem evolution (minutes)

```
Pattern evaluates observation
    -> prediction recorded
    -> outcome observed
    -> fitness updated
    -> reproduction/death at Theta tick
    -> new patterns enter population
    -> new patterns evaluate next observation
```

This is the fastest loop. It runs every Theta tick. The timescale is minutes. The loop drives adaptation to the current market conditions as the Golem experiences them.

### Loop 2: cross-generation inheritance (hours to days)

```
Golem N's ecosystem evolves
    -> Golem N dies, testament extracted
    -> Golem N+1 inherits testament
    -> elevated mutation restores diversity
    -> Golem N+1 evolves from higher fitness floor
```

This loop runs at the timescale of Golem lifetimes. Each generation inherits the best of the previous generation and evolves further. The ratchet effect accumulates fitness across generations.

### Loop 3: Clade-level ecology (hours)

```
Golem A discovers high-fitness pattern
    -> pattern emigrates via Styx
    -> Golem B receives, quarantines, tests
    -> if useful: Golem B adopts
    -> pattern spreads through Clade
    -> Clade-wide fitness improves
```

This loop runs at the timescale of Delta ticks. It provides distributed exploration: the Clade collectively explores the pattern space faster than any single Golem could.

The three loops interact. A pattern discovered through Clade migration (Loop 3) might be further refined through within-Golem evolution (Loop 1) and then propagated to the next generation via death testament (Loop 2). The loops are not hierarchical; they run concurrently and feed into each other.

### Adaptive mutation as homeostasis

The adaptive mutation rate creates a homeostatic mechanism. When the ecosystem is diverse (high Shannon entropy), mutation stays at baseline, allowing selection to refine existing patterns. When diversity drops (entropy falls below threshold), mutation increases, injecting noise to prevent population collapse. When diversity recovers, mutation drops back to baseline.

This is a negative feedback loop: diversity depletion triggers its own correction. The time constant depends on the mutation rate range and the reproduction rate, but typical recovery from a diversity crisis takes 5-10 Theta ticks, roughly 10-20 minutes.

## A worked example: the liquidity trap lifecycle [SPEC]

To ground the abstract evolutionary machinery in concrete behavior, here is a complete lifecycle of a pattern through the ecosystem.

**Tick 0: founding.** The ecosystem is seeded with 15 founder patterns per niche, 75 total. One founder in the swap niche, P-3, encodes a generic "large liquidity addition near current price" detector. Its hypervector was generated by binding together the HDC atoms for large_liquidity_event, below_spot_price, and uniswap_v3_pool from the codebook (Doc 1). Its initial fitness is 0.5 (the uninformative prior).

**Ticks 1-50: evaluation.** Over the first 50 Theta ticks, P-3 evaluates 38 swap observations that won evaluation slots in the VCG auction. It correctly predicts the direction of 26 of them (68% accuracy). Its fitness climbs to 0.72, placing it in the top quartile of the swap niche.

**Tick 52: first reproduction.** P-3 is selected as a parent alongside P-7 (a momentum pattern with fitness 0.65) via tournament selection. Weighted crossover with w_3 = 0.72/(0.72+0.65) = 0.53 produces offspring P-89. P-89 inherits a slight majority of its bits from P-3, giving it a liquidity-detection bias but with momentum sensitivity mixed in.

**Ticks 53-100: speciation.** P-89 evaluates observations and achieves 0.74 accuracy in the swap niche, slightly beating its parent. P-3 itself reproduces twice more, producing P-102 and P-115. The P-3 lineage now has four members. P-89 reproduces with P-12 (a mean-reversion pattern), producing P-134, which turns out to be the first pattern in the lineage that detects the full liquidity trap sequence: addition, price push, removal. P-134 reaches 0.81 accuracy.

**Ticks 100-150: dominance.** P-134 and its descendants dominate the swap niche. The Lotka-Volterra model registers increasing population pressure on the swap edge. Edge estimate for the swap niche begins declining from 0.9 toward 0.6.

**Ticks 150-200: edge depletion.** Other market participants (or other Clades) have learned to detect the same pattern. The edge drops to 0.35. P-134's accuracy falls from 0.81 to 0.58. Its offspring, still tuned to the same pattern, also decline. The kill_unfit pass at Theta tick 180 removes 4 patterns from the P-3 lineage that have dropped below the niche mean minus one standard deviation.

**Tick 201: REM dream mutation.** During a REM cycle, the ecosystem generates mutants of all surviving patterns. A mutant of P-134 (call it P-201) has 1% of its bits flipped. By chance, the flipped bits alter its sensitivity: it now responds more strongly to the "decoy liquidity" signal that competitors have started planting. P-201's counterfactual fitness against the replay buffer is 8% higher than P-134's.

**Ticks 210-250: adaptation.** P-201 enters the population and achieves 0.71 accuracy: lower than P-134's peak, but adapted to the new conditions where decoys are common. The P-3 lineage has adapted without human intervention. The original pattern P-3 itself died at tick 195 (low fitness after 195 evaluations), but its genetic material lives on in P-201, three generations downstream.

**Tick 260: Styx emigration.** P-201, now at 0.73 fitness and above the 90th percentile in the swap niche, is selected for Styx emigration. A sibling Golem in the Clade receives it, quarantines it for 10 ticks, evaluates it at 0.68 accuracy in its own observations, and admits it. The liquidity-trap-with-decoy-detection pattern has spread across the Clade through horizontal gene transfer.

This lifecycle took about 4 hours of wall time. No human wrote the decoy-aware variant. It emerged from crossover, selection, and mutation operating on hypervector genomes at bitwise speed.

## Evaluation protocol [SPEC]

The evaluation protocol measures whether the ecosystem behaves as predicted by the mathematical framework. Each metric targets a specific theoretical claim.

### Metric 1: diversity maintenance

**Claim:** the adaptive mutation mechanism prevents population collapse while selection prevents random drift.

**Method:** Run the ecosystem for 1,000 Theta ticks with a synthetic replay buffer containing five distinct pattern types (one per niche). Measure Shannon entropy H at every Delta tick. Record minimum H, maximum H, and the number of consecutive Delta cycles where H drops below 0.5 * H_initial.

**Pass criteria:** H never drops below 0.5 * H_initial for more than 2 consecutive Delta cycles. The mean H over the full run is within 30% of H_initial.

**Failure diagnosis:** If H collapses, check the adaptive mutation rate. Is the min_diversity_threshold set too low? Is the max_mutation_rate too conservative? If H trends upward without bound, selection pressure is too weak: increase alpha or reduce carrying capacity.

### Metric 2: fitness improvement across generations

**Claim:** the death-testament-inheritance mechanism ratchets fitness upward across Golem generations.

**Method:** Run 5 sequential Golem lifetimes (each 200 Theta ticks) with testament inheritance between them. Measure mean fitness at three points per lifetime: at birth (after inheritance + elevated mutation), at midlife (tick 100), and at death (tick 200).

**Pass criteria:** Birth fitness of generation N+1 exceeds birth fitness of generation N for at least 4 of 5 transitions. Death fitness exceeds birth fitness within the same generation for all 5 lifetimes.

**Failure diagnosis:** If birth fitness fails to ratchet, the testament is losing too much information in the bottleneck. Increase testament_size. If within-lifetime improvement stalls, check selection pressure and crossover rate.

### Metric 3: niche colonization

**Claim:** mutation and crossover allow the ecosystem to colonize niches it was not explicitly seeded for.

**Method:** Seed the ecosystem with swap-niche-only founders (15 patterns, all with niche = Swap). Run for 500 Theta ticks with a replay buffer containing observations from all five niches.

**Pass criteria:** After 500 ticks, all five niches have at least min_population_per_niche viable patterns (fitness above 50% of the global mean). Coverage > 0.3 for every niche.

**Failure diagnosis:** If colonization fails, mutation is not creating enough inter-niche variation. Increase base mutation rate or reduce the niche affinity penalty for cross-niche evaluation.

### Metric 4: edge depletion response

**Claim:** the Lotka-Volterra model correctly predicts that overcrowded niches lose edge and subsequently contract.

**Method:** Run the ecosystem with 10 patterns per niche for 100 Theta ticks to reach steady state. At tick 100, inject 50 identical patterns into the swap niche. Continue for 200 more ticks.

**Pass criteria:** Swap edge estimate drops below 50% of its pre-injection value within 20 Theta ticks. Swap niche population returns to within 20% of its pre-injection count within 100 Theta ticks after injection.

### Metric 5: evolving vs. fixed comparison

**Claim:** an evolving ecosystem outperforms a static pattern library, especially as market conditions shift.

**Method:** Run two configurations against the same observation stream for 1,000 Theta ticks:

- Configuration A: full evolution (crossover, mutation, selection, migration all active)
- Configuration B: identical initial founders, all evolution disabled (fixed population)

Introduce a regime change at tick 500 (swap the replay buffer to a different market period).

**Pass criteria:** Cumulative prediction accuracy of A exceeds B by at least 5% over the full 1,000 ticks. After the regime change (ticks 500-1,000), the gap widens to at least 10%.

### Metric 6: Price equation decomposition

**Claim:** the Price equation correctly decomposes fitness improvement into selection and transmission terms.

**Method:** Run the ecosystem for 500 Theta ticks, recording the Price equation decomposition at every Delta tick. Compare the sum of selection and transmission terms against the observed change in mean fitness between consecutive Delta ticks.

**Pass criteria:** The Price equation prediction matches the observed delta W_bar to within 15% at every Delta tick (accounting for discretization error). The selection term is positive on average. The transmission term is negative during high-mutation periods and near-zero during low-mutation periods.

### Metric 7: Styx migration benefit

**Claim:** Clade-level pattern migration improves collective performance relative to isolated Golems.

**Method:** Run two Clade configurations, each with 3 Golems:

- Configuration A: Styx migration active (patterns can emigrate and immigrate)
- Configuration B: Styx disabled (each Golem evolves in isolation)

All Golems see overlapping but not identical observation streams (simulating different positions in the same market).

**Pass criteria:** Mean fitness across the Clade is at least 10% higher in configuration A than B after 500 Theta ticks. The diversity index is also higher in A (migration introduces novel genetic material).

## References

Dawkins, R. (1976). *The Selfish Gene*. Oxford University Press. The conceptual framework for viewing replicating information (genes, memes, and in our case, TA patterns) as the unit of selection rather than the organism.

Fisher, R.A. (1930). *The Genetical Theory of Natural Selection*. Oxford University Press. Fisher's fundamental theorem: the rate of fitness increase equals the additive genetic variance in fitness. We use this as a real-time diagnostic for ecosystem health.

Holland, J.H. (1975). *Adaptation in Natural and Artificial Systems*. University of Michigan Press. Foundational work on genetic algorithms. The tournament selection and crossover operators used here descend directly from Holland's framework.

Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors." *Cognitive Computation*, 1(2), 139-159. The theoretical basis for using high-dimensional binary vectors as a substrate for evolutionary operations.

Lotka, A.J. (1925). *Elements of Physical Biology*. Williams & Wilkins. The predator-prey model we adapt for edge depletion dynamics.

Maynard Smith, J. (1982). *Evolution and the Theory of Games*. Cambridge University Press. Evolutionary game theory and the replicator equation that governs our population dynamics.

Price, G.R. (1970). "Selection and Covariance." *Nature*, 227, 520-521. The Price equation decomposes evolutionary change into selection and transmission components. We use it to diagnose whether the ecosystem is improving through competition or through innovation.

Taylor, P.D. and Jonker, L.B. (1978). "Evolutionary stable strategies and game dynamics." *Mathematical Biosciences*, 40(1-2), 145-156. Original formulation of the replicator equation.

Volterra, V. (1926). "Fluctuations in the Abundance of a Species considered Mathematically." *Nature*, 118, 558-560. The prey side of the predator-prey equations.

Innovation 06: Memetic Knowledge Evolution. Grimoire entries as Dawkinsian replicators with fitness W = f * r * L. Replicator dynamics, Price equation for knowledge improvement. Epistemic parasite detection.

Innovation 08: Morphogenetic Specialization. Turing patterns for clade ecology. Reaction-diffusion drives spontaneous role specialization. Provides the Clade topology that governs pattern migration paths.

Doc 1: Hyperdimensional Technical Analysis. 10,240-bit BSC hypervectors. Bind=XOR, bundle=majority vote, similarity=Hamming. The encoding substrate for all pattern genomes.

Doc 3: Adaptive Signal Metabolism. Hebbian learning + economic selection for signals. Signal speciation. Patterns consume signals as inputs and compete alongside them for attention budget.
