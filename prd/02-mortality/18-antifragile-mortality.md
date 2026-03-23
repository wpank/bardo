# Antifragile Mortality Architecture [SPEC]

> _"The wind that extinguishes a candle feeds a fire."_
> -- Nassim Nicholas Taleb

> **Version**: 1.0 | **Status**: Draft
>
> **Crates**: `golem-risk`, `golem-grimoire`, `golem-core`
>
> **Depends on**: `01-architecture.md` (triple-clock system, vitality computation), `04-economic-mortality.md` (USDC as metabolic substrate, burn rate economics), `10-clade-ecology.md` (superorganism dynamics, stigmergic coordination)

---

> **Reader orientation:** This document extends Bardo's mortality architecture from resilient (absorbs shocks, returns to baseline) to antifragile (gets stronger from stress). The mechanism is a barbell allocation: 90% of the Golem's (mortal autonomous DeFi agent's) strategy stays conservative, while 10% runs maximum-curiosity exploration during crises. Via negativa knowledge (what NOT to do, learned from failures and dead predecessors) decays more slowly than positive knowledge and forms the Grimoire's (persistent knowledge base's) most durable substrate. Jensen's inequality shows that variable environments produce more total learning than stable ones, which is why mortality-driven turnover outperforms immortal stability. See `01-architecture.md` (triple-clock system) and `10-clade-ecology.md` (superorganism dynamics) for context. See `prd2/shared/glossary.md` for full term definitions.

## Document Purpose

Bardo's current risk architecture is defensive. Five adaptive layers protect the Golem from catastrophic loss. When stress rises, risk limits tighten. When mortality clocks tick faster, position sizes shrink. When prediction accuracy drops, the system retreats to proven strategies and suppresses exploration.

This is resilient design. A resilient Golem absorbs shocks and returns to baseline. But resilience has a ceiling: the Golem that enters a crisis is, at best, the same Golem that exits it. The crisis produced nothing. All that information -- all those prediction residuals, correlation breakdowns, protocol failures -- was treated as threat and discarded.

This document specifies an antifragile extension to the mortality architecture: a system that gets stronger from stress. The design combines a barbell allocation (90% conservative, 10% maximum-curiosity exploration), a via negativa knowledge substrate where negative knowledge ages more slowly than positive knowledge, and a confidence decay floor that prevents AntiKnowledge from ever fully decaying. Jensen's inequality, applied to the Golem's learning function, shows that variable environments produce more total learning than stable ones.

---

## Antifragility as Convexity

### Formal Definition

A function f is antifragile if it is convex: for any perturbation delta around a point x,

```
f(x + delta) + f(x - delta) > 2f(x)
```

The average of f applied to variable inputs exceeds f applied to the average input. Volatility in the input helps [TALEB-DOUADY-2013].

For Golems, the perturbation is market volatility sigma(t) and the response is knowledge acquisition rate L(sigma). The system is antifragile if L is convex in sigma:

```
L(sigma + delta) + L(sigma - delta) > 2L(sigma)
```

### Jensen's Inequality and Mortal Learning

Jensen's inequality (Jensen, 1906) states that for a convex function f and a random variable X:

```
E[f(X)] >= f(E[X])
```

If L(sigma) is convex in sigma, then the expected learning rate under variable volatility exceeds the learning rate at average volatility. A Golem living through boom-bust cycles learns more, in expectation, than a Golem living through perpetual calm at the same average volatility.

For mortal Golems with finite lifespan T:

```
K = integral from 0 to T of L(sigma(t)) dt
```

If L is convex, a volatile environment produces higher K than a calm environment with the same time-averaged volatility. The mortal Golem in a volatile market produces more knowledge before death.

---

## Via Negativa: AntiKnowledge as the Antifragile Substrate

### Two Knowledge Classes

**K+ (positive knowledge):** "Strategy X works in condition Y." Example: "Providing liquidity to ETH/USDC 0.3% tier produces positive returns when realized volatility is below 40% annualized."

**K- (negative knowledge / AntiKnowledge):** "Strategy X fails in condition Z." Example: "Never provide liquidity to any pool where the oracle update frequency exceeds 10x the block time. The oracle lag creates an arbitrage that consistently drains LPs."

K- has a longer half-life than K+ because failure modes are properties of mechanism design, and mechanism design changes slowly. The reason providing liquidity with a lagging oracle loses money is structural: the information asymmetry is baked into the protocol's architecture. This failure mode persists until the protocol is redesigned. By contrast, conditions under which a strategy produces positive returns depend on market microstructure, competition, and capital flows -- all of which shift constantly.

### AntiKnowledge: Honeypots, Scams, Failed Strategies

AntiKnowledge is a specific category of K- knowledge that encodes traps, exploits, and definitively failed approaches:

- **Honeypot contracts:** Token contracts with hidden sell restrictions, fee-on-transfer traps, or admin-only withdrawal functions
- **Scam patterns:** Rug pull signatures, fake liquidity additions, coordinated pump-and-dump patterns
- **Failed strategies:** Approaches that were tested and produced consistent losses under specific conditions
- **Oracle manipulation patterns:** Sequences of transactions that indicate oracle price manipulation
- **MEV attack signatures:** Known sandwich attack patterns, frontrunning sequences, JIT liquidity attacks

AntiKnowledge entries in the Grimoire are tagged with `polarity: Negative` and receive differential treatment in the demurrage system.

### Confidence Decay Floor of 0.3

Standard Grimoire entries decay toward zero confidence via knowledge demurrage. AntiKnowledge entries decay toward a floor of 0.3, never reaching zero. The rationale: if a honeypot contract was dangerous in 2024, it is still dangerous in 2025. The failure mode is structural. Allowing the warning to decay to zero would mean re-learning the lesson through another Golem's death.

```rust
/// Compute effective confidence after demurrage, respecting the
/// AntiKnowledge floor.
pub fn confidence_after_demurrage(
    base_confidence: f64,
    demurrage_rate: f64,
    ticks_elapsed: u64,
    is_antiknowledge: bool,
) -> f64 {
    let decayed = base_confidence * (-demurrage_rate * ticks_elapsed as f64).exp();
    if is_antiknowledge {
        decayed.max(0.3) // AntiKnowledge never fully decays
    } else {
        decayed
    }
}
```

The 0.3 floor is above the archive threshold (0.1) used by the Curator for pruning. This means AntiKnowledge entries are never pruned -- they persist in the active Grimoire indefinitely, providing a growing immune system.

### Differential Demurrage

K- entries decay at half the rate of K+ entries:

```
demurrage_K_minus = base_demurrage * 0.5
demurrage_K_plus  = base_demurrage * 1.0
```

Over many stress cycles, the Grimoire's composition shifts toward durable negative knowledge. Each crisis makes the next crisis less dangerous because the Golem has learned more ways to die.

---

## Barbell Allocation

### The Structure

The barbell partitions the Golem's portfolio into two firewalled budgets:

```
conservative_budget = vault_balance * 0.90
exploration_budget  = vault_balance * 0.10
```

The conservative allocation uses the full five-layer risk stack with ergodicity-optimal Kelly sizing. The exploration allocation uses only Layer 1 (Hard Shields from PolicyCage) as a backstop, bypassing Kelly constraints and behavioral anomaly detection. This is deliberate: exploration during anomalous markets is where the learning happens.

### Why the Firewall Matters

Exploration losses must not affect the conservative allocation. The 10% exploration budget is an option: limited downside (10% of capital), potentially unlimited upside (the value of what might be discovered).

```
V_explore(t) = max(0, K_discovered(t) - K_cost(t))
```

During high-volatility periods, the option's value increases. This is the Vega effect applied to knowledge: the exploration allocation becomes more valuable during crises because there is more to discover.

### Implementation

```rust
/// Budget partition for the barbell strategy.
#[derive(Debug, Clone)]
pub struct BarbellConfig {
    pub conservative_ratio: f64,
    pub exploration_ratio: f64,
    pub min_exploration: f64,
    pub max_exploration: f64,
}

impl Default for BarbellConfig {
    fn default() -> Self {
        Self {
            conservative_ratio: 0.90,
            exploration_ratio: 0.10,
            min_exploration: 0.05,
            max_exploration: 0.20,
        }
    }
}

pub struct BarbellAllocator {
    config: BarbellConfig,
    vault_balance: f64,
    exploration_pnl: f64,
    exploration_entries: u64,
}

impl BarbellAllocator {
    pub fn new(config: BarbellConfig, vault_balance: f64) -> Self {
        Self {
            config,
            vault_balance,
            exploration_pnl: 0.0,
            exploration_entries: 0,
        }
    }

    pub fn conservative_budget(&self) -> f64 {
        self.vault_balance * self.config.conservative_ratio
    }

    pub fn exploration_budget(&self) -> f64 {
        self.vault_balance * self.config.exploration_ratio
    }

    pub fn record_exploration_pnl(&mut self, pnl: f64, entries: u64) {
        self.exploration_pnl += pnl;
        self.exploration_entries += entries;
    }

    /// Entries per USDC lost. Higher is better.
    pub fn exploration_efficiency(&self) -> f64 {
        if self.exploration_pnl >= 0.0 { return f64::INFINITY; }
        let cost = self.exploration_pnl.abs();
        if cost < 1e-6 { return f64::INFINITY; }
        self.exploration_entries as f64 / cost
    }

    /// Adjust ratio. Called by Loop 3 (meta-learning).
    pub fn adjust_ratio(&mut self, direction: f64) {
        let step = 0.01 * direction.signum();
        let new_explore = (self.config.exploration_ratio + step)
            .max(self.config.min_exploration)
            .min(self.config.max_exploration);
        self.config.exploration_ratio = new_explore;
        self.config.conservative_ratio = 1.0 - new_explore;
    }

    pub fn reset_tracking(&mut self) {
        self.exploration_pnl = 0.0;
        self.exploration_entries = 0;
    }

    pub fn update_balance(&mut self, balance: f64) {
        self.vault_balance = balance;
    }
}
```

---

## Optionality Preservation

### Definition

Optionality is the property of having choices available. A position with optionality has limited downside and open-ended upside. The barbell is one form of optionality. The broader principle: across all decisions, prefer choices that preserve future options over choices that foreclose them.

In DeFi terms:

- Holding USDC preserves optionality (can deploy to any protocol). Locking in a 90-day vault forecloses it.
- Maintaining a diversified strategy vector preserves optionality. Over-specializing forecloses it.
- Keeping capital above the Conservation threshold preserves options. Dipping below it triggers irreversible behavioral restrictions.

### Decision Scoring

When the Golem evaluates candidate actions at the Simulate step (Step 5), each action is scored on an optionality dimension:

```rust
/// Score the optionality impact of a proposed action.
/// Negative scores mean the action forecloses future options.
/// Positive scores mean it creates new options.
pub fn score_optionality(
    action: &ProposedAction,
    current_state: &GolemState,
) -> f64 {
    let mut score = 0.0;

    // Penalty for capital lockup that reduces available capital
    // below the Conservation threshold.
    let post_action_available = current_state.available_capital
        - action.capital_required;
    if post_action_available < current_state.conservation_threshold {
        score -= 0.5;
    }

    // Penalty for concentrating in a single protocol.
    if action.increases_concentration_above(0.5) {
        score -= 0.3;
    }

    // Bonus for actions that create future exit options.
    if action.has_exit_option() {
        score += 0.2;
    }

    // Bonus for actions that generate information regardless of P&L.
    if action.exploration_value > 0.0 {
        score += action.exploration_value * 0.3;
    }

    score
}
```

---

## Tail Risk Optimization via Barbell

### The Barbell as Tail Risk Strategy

The barbell naturally provides tail risk optimization. The 90% conservative allocation has bounded downside (near-zero loss in the worst case). The 10% exploration allocation has exposure to tail upside (discovering a novel strategy, identifying a systemic risk before others).

The combined portfolio has a convex payoff profile:

```
E[L_barbell(sigma)] = 0.9 * E[L_conservative(sigma)] + 0.1 * E[L_explore(sigma)]
```

L_conservative is approximately constant in sigma. L_explore is convex in sigma. The weighted sum is convex because any positive linear combination of a constant and a convex function is convex. The barbell inherits the exploration allocation's convexity while bounding downside.

### Stress-Aware Knowledge Harvesting

During high-volatility periods, the Grimoire increases its write budget:

```
write_rate(sigma) = base_rate * (1 + alpha * (sigma / sigma_ref - 1))
```

Stress periods generate K- at a higher rate than calm periods because stress is when things break. A liquidity crisis that drains three pool types produces three durable K- entries. The same period might invalidate several K+ entries calibrated to calm conditions.

```rust
/// Knowledge polarity tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgePolarity {
    Positive,
    Negative,
}

/// Stress-aware knowledge configuration.
pub struct StressKnowledgeConfig {
    pub base_demurrage: f64,
    pub negative_demurrage_factor: f64,
    pub stress_volatility_threshold: f64,
    pub base_write_rate: f64,
    pub write_rate_alpha: f64,
    pub reference_volatility: f64,
    /// Confidence floor for AntiKnowledge entries.
    pub antiknowledge_floor: f64,
}

impl Default for StressKnowledgeConfig {
    fn default() -> Self {
        Self {
            base_demurrage: 0.001,
            negative_demurrage_factor: 0.5,
            stress_volatility_threshold: 0.05,
            base_write_rate: 1.0,
            write_rate_alpha: 2.0,
            reference_volatility: 0.02,
            antiknowledge_floor: 0.3,
        }
    }
}

impl StressKnowledgeConfig {
    pub fn write_rate(&self, current_volatility: f64) -> f64 {
        let ratio = current_volatility / self.reference_volatility;
        self.base_write_rate
            * (1.0 + self.write_rate_alpha * (ratio - 1.0).max(0.0))
    }

    pub fn demurrage_for(&self, polarity: KnowledgePolarity) -> f64 {
        match polarity {
            KnowledgePolarity::Positive => self.base_demurrage,
            KnowledgePolarity::Negative => {
                self.base_demurrage * self.negative_demurrage_factor
            }
        }
    }

    pub fn is_stressed(&self, current_volatility: f64) -> bool {
        current_volatility > self.stress_volatility_threshold
    }
}
```

---

## Convexity Monitoring

A CorticalState signal tracks whether the Golem is actually antifragile:

```
learning_convexity: AtomicU32  // f32, estimated convexity of L(sigma)
```

Positive values = antifragile (learning more during stress). Negative values = fragile (learning less during stress). Zero = resilient (learning rate independent of stress).

### Estimation

```rust
use std::collections::VecDeque;

/// Rolling estimator for the convexity of L(sigma).
/// Fits L = a*sigma^2 + b*sigma + c and reports `a`.
pub struct ConvexityEstimator {
    window: VecDeque<(f64, f64)>,
    max_size: usize,
}

impl ConvexityEstimator {
    pub fn new(max_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, volatility: f64, learning_rate: f64) {
        if self.window.len() == self.max_size {
            self.window.pop_front();
        }
        self.window.push_back((volatility, learning_rate));
    }

    /// Returns the quadratic coefficient `a`. Positive = antifragile.
    pub fn estimate_convexity(&self) -> Option<f64> {
        let n = self.window.len();
        if n < 10 { return None; }

        let mut xtx = [[0.0_f64; 3]; 3];
        let mut xty = [0.0_f64; 3];

        for &(sigma, l) in &self.window {
            let s2 = sigma * sigma;
            let row = [s2, sigma, 1.0];
            for i in 0..3 {
                for j in 0..3 {
                    xtx[i][j] += row[i] * row[j];
                }
                xty[i] += row[i] * l;
            }
        }

        let det = det_3x3(&xtx);
        if det.abs() < 1e-12 { return None; }

        let mut m = xtx;
        for i in 0..3 { m[i][0] = xty[i]; }
        Some(det_3x3(&m) / det)
    }
}

fn det_3x3(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}
```

### Loop 3 Integration

Loop 3 (meta-learning, days-to-weeks timescale) monitors `learning_convexity` and adjusts barbell parameters:

- If `learning_convexity` < 0 for > 48 hours, increase exploration from 10% to 15%.
- If `learning_convexity` > threshold for > 48 hours, decrease exploration toward 10%.
- If exploration has lost > 50% of its budget without generating proportional K- entries, rotate the exploration strategy.

---

## Dream Cycle Prioritization

> Source: `innovations/09-antifragile-architecture.md`, Section 3.2

During NREM replay (offline consolidation), episodes from stress periods receive higher replay priority. The consolidation engine weights episodes by their information content, estimated from prediction residual magnitude. Stress-period episodes have larger residuals and therefore higher replay weight. This biases the Golem's offline learning toward crisis-generated knowledge.

---

## Volatility Harvesting for Knowledge

> Source: `innovations/09-antifragile-architecture.md`, Section 2.6

Define information density as a function of market volatility:

```
I(sigma) = bits of useful information per tick at volatility sigma
```

The hypothesis is that I is convex in sigma:

```
I(sigma + delta) + I(sigma - delta) > 2I(sigma)
```

The argument is empirical and structural. During calm periods, price movements are small, predictions are accurate, residuals are small, and few surprises occur. The information content per tick is low. During crises, prices move sharply, predictions fail, residuals are large, correlations shift, and many surprises occur simultaneously. The information content per tick is high, and it grows faster than linearly with volatility because crisis dynamics are combinatorial: if n protocol interactions can break, and stress tests them simultaneously, the number of observable failure combinations grows as O(2^n) while volatility grows linearly.

The catch: the Golem must survive the crisis to harvest the knowledge. Dead agents learn nothing. The barbell ensures survival (the 90% conservative allocation weathers the storm). The 10% exploration allocation ensures harvest (it is sized to take risks during the crisis and record what happens).

The key equation combines Jensen's inequality with the barbell:

```
E[L_barbell(sigma)] = 0.9 * E[L_conservative(sigma)] + 0.1 * E[L_explore(sigma)]
```

L_conservative is approximately constant in sigma (conservative strategies don't learn much regardless of conditions). L_explore is convex in sigma (exploration learns disproportionately more during stress). The weighted sum is convex because any positive linear combination of a constant function and a convex function is convex. The barbell inherits the exploration allocation's convexity while bounding the downside with the conservative allocation's stability.

---

## Philosophical Grounding

> Source: `innovations/09-antifragile-architecture.md`, Section 7

### Nietzsche's Dictum, Taken Literally

"What does not kill me makes me stronger" is typically read as motivational rhetoric. For a system with a convex learning function, it is a mathematical statement. Any perturbation delta that the system survives produces a learning response L(delta). If L is convex, larger perturbations produce disproportionately larger learning responses. The system literally gets stronger from larger stresses, provided it survives them.

The barbell is the mechanism that converts this from tautology to architecture. Without the barbell, the Golem that survives a crisis might or might not learn from it (it depends on whether the risk system allowed enough exploration during the crisis). With the barbell, the exploration allocation guarantees that the Golem observes the crisis actively, not just passively, and records what it observes.

### Mortality as Antifragility's Prerequisite

An immortal agent has no reason to adopt the barbell. With infinite time, expected value converges to time average (by ergodicity over infinite horizons, if the process is stationary). The immortal agent can afford to explore uniformly, without partitioning resources, because it can recover from any finite loss given enough time.

A mortal agent cannot. Its finite horizon forces a choice: explore uniformly and risk death, or partition resources so that survival is guaranteed and exploration is bounded. The barbell is the mortal agent's solution to this dilemma. Mortality creates the constraint that makes the barbell necessary, and the barbell creates the convexity that makes the system antifragile.

This connects to the Thanatopsis protocol. When a Golem dies, its accumulated knowledge (including all K- entries) is transmitted to the Clade via the bloodstain mechanism. Death is not just the end of one agent's learning. It is the moment when that agent's knowledge becomes available to all surviving agents. The dying Golem's final act is a knowledge transfer, and the knowledge transferred during death (when the Golem has experienced terminal-phase conditions that surviving Golems have not) is among the most informative the Clade will ever receive.

Death, in this framework, is the ultimate stress event. The Golem that dies after a full lifetime of antifragile operation passes a body of K- knowledge, much of it generated during crises, most of it durable, to a Clade that will use it to survive crises that the dead Golem did not. The individual is fragile (it dies). The Clade is antifragile (it gets stronger from each member's death).

### Seneca's Asymmetry

Seneca observed that we learn more from a single catastrophe than from a thousand calm days. This is not just a rhetorical claim. It is a statement about the information content of extreme events.

In information-theoretic terms (connecting to 17-information-theoretic-diagnostics.md), the surprise of an event is -log(p), where p is the event's probability. Low-probability events carry more information per occurrence. A flash crash that happens once a year carries -log(1/365) ~ 8.5 bits of information. A normal trading day carries -log(364/365) ~ 0.004 bits. The flash crash is 2,000x more informative per occurrence.

The current system treats the flash crash as a threat and shuts down learning. The antifragile system treats it as a windfall and maximizes learning. Seneca's asymmetry, quantified via Shannon, is the information-theoretic justification for stress-aware write rates and dream-cycle prioritization of stress episodes.

---

## Cross-Clade Antifragility

When one Golem's exploration discovers a failure mode and the Golem dies as a result, the K- entry is shared with the Clade via the bloodstain protocol. The Golem's death is the Clade's gain. One member's loss during exploration becomes every member's durable negative knowledge. The Clade is antifragile even if individual members are not: the Clade benefits from stress that kills its components.

This mirrors the biological immune system. The immune system learns from pathogens that kill individual cells. The Clade learns from market conditions that kill individual Golems. The unit of antifragility is the Clade, not the Golem.

---

## Evaluation and Falsifiability

### Null Hypothesis

Antifragile Golems produce no more knowledge than resilient Golems in volatile environments.

### Protocol

Two populations on the same historical data (Base L2, 180-day window spanning at least two high-volatility events):

- **Control (resilient):** Standard risk architecture, no barbell, uniform demurrage, no stress-aware write rate.
- **Treatment (antifragile):** Barbell allocation, differential demurrage, stress-aware write rate, convexity monitoring, AntiKnowledge floor.

### Metrics

- **Knowledge acquisition convexity:** Fit L = a*sigma^2 + b*sigma + c. Treatment should have a > 0.
- **Knowledge half-life:** K- entries from stress should remain actionable >= 2x as long as K+ entries from calm.
- **Survival x Knowledge product:** survival_rate * total_knowledge should be higher for treatment.

### Falsification Conditions

1. Treatment group's convexity coefficient a is not significantly > 0 (p < 0.05) across 1,000 runs.
2. K- entries from stress don't outlast K+ entries from calm in >= 80% of simulations.
3. survival_rate * total_knowledge is lower for treatment in > 50% of volatility regimes.

---

## Cross-References

- **04-economic-mortality.md**: The ergodicity economics section derives the barbell from Kelly allocation across heterogeneous volatilities. The antifragile architecture extends this from capital to knowledge.
- **10b-morphogenetic-specialization.md**: Specialized Clades generate non-overlapping K- entries during the same crisis. Specialization amplifies collective antifragility.
- **17-information-theoretic-diagnostics.md**: Antifragility means I(G; M) increases with volatility. The MI diagnostic detects whether the Golem's vitality is convex in market volatility.
- **10-clade-ecology.md**: The bloodstain network is the mechanism through which individual deaths produce Clade-level antifragility.
- **01-architecture.md**: The barbell operates within the existing five-layer risk stack, adding optionality preservation as a sixth consideration.

---

## References

1. Taleb, N.N. (2012). *Antifragile: Things That Gain from Disorder*. Random House.

2. Taleb, N.N. & Douady, R. (2013). "Mathematical Definition, Mapping, and Detection of (Anti)Fragility." *Quantitative Finance*, 13(11), 1677-1689.

3. Jensen, J.L.W.V. (1906). "Sur les fonctions convexes et les inegalites entre les valeurs moyennes." *Acta Mathematica*, 30, 175-193.

4. Shannon, C.E. (1948). "A Mathematical Theory of Communication." *Bell System Technical Journal*, 27(3), 379-423.

5. Peters, O. (2019). "The Ergodicity Problem in Economics." *Nature Physics*, 15(12), 1216-1221.
