# 14b -- VCG Attention Auction [SPEC]

**Phase 2: Strategy-Proof Cognitive Resource Allocation**

> **Version**: 2.0 | **Status**: Implementation Specification (Phase 2, after EFE composite validates)
>
> **Crate**: `golem-context` | **Layer**: 0 (FOUNDATION)
>
> Cross-references: [14-context-governor.md](./14-context-governor.md) (EFE composite, Phase 1), [17-prediction-engine.md](./17-prediction-engine.md) (Oracle), [18-cortical-state.md](./18-cortical-state.md) (CorticalState)
>
> Sources: [05-attention-as-auction.md](../../tmp/research/witness-research/new/innovations/05-attention-as-auction.md)

> **Reader orientation:** This document specifies Phase 2 of the attention allocation system: a VCG (Vickrey-Clarke-Groves) auction where five cognitive subsystems (Oracle, Daimon, RiskEngine, Curiosity, Mortality) bid independently for the Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) limited attention budget. Phase 1 (the EFE composite in `14-context-governor.md`, the learnable context assembly system) is the current canonical mechanism. Phase 2 activates only after Phase 1 has been validated over a full lifecycle. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## When This Activates

The VCG auction is Phase 2 of the attention allocation transition. Phase 1 (the EFE composite score in [14-context-governor.md](./14-context-governor.md)) uses four additive components to score items. Phase 2 replaces the additive scoring with a full VCG mechanism where cognitive subsystems bid independently.

Phase 2 activates when:
1. The EFE composite has been validated over at least one full golem lifecycle
2. Clade-level attention coordination is needed (multiple golems sharing attention budgets)
3. The Phase 1 scoring produces measurable suboptimality (attention allocation quality plateaus)

Until then, Phase 1's EFE composite is canonical. This file specifies the Phase 2 mechanism for implementation readiness.

---

## The Attention Market

**Items** I = {i_1, ..., i_n}: candidate observations (token pairs, protocol states, positions, on-chain events). Typically n = 50 to 500.

**Bidders** B = {Oracle, Daimon, RiskEngine, Curiosity, Mortality}: five cognitive subsystems, each with a distinct perspective on what deserves attention.

**Valuations**: each bidder b has v_b(i) >= 0 for each item i, representing the welfare b derives from having item i in the ACTIVE tier.

**Budget** K: maximum ACTIVE items. Derived from mortality state (see below).

---

## The VCG Mechanism

The Vickrey-Clarke-Groves mechanism (Vickrey, 1961; Clarke, 1971; Groves, 1973) provides two properties:

**Strategy-proofness.** Each bidder's dominant strategy is truthful reporting. No subsystem benefits from inflating or deflating bids. This means subsystems report honest valuations and the mechanism handles the rest.

**Efficiency.** The allocation maximizes total reported welfare. Since all bidders report truthfully, this also maximizes true welfare.

### Allocation

Under additive valuations, the aggregate value of item i is:

```
V(i) = sum_{b in B} v_b(i)
```

S* is the K items with highest V(i). This is a sort: O(n log n).

### Payments

Each bidder b pays the externality it imposes:

```
p_b = (welfare of others without b's influence on allocation)
    - (welfare of others under actual allocation)
```

These payments are bookkeeping, not USDC. They quantify opportunity costs for diagnostics and learning. When the risk engine's bids displace a curiosity-driven item, the payment shows the displacement cost.

---

## Valuation Functions

### Oracle (prediction engine)

```rust
/// Oracle values attention on items where predictions are wrong.
fn valuate_oracle(item: &TrackedItem, oracle: &Oracle) -> f64 {
    let residual = oracle.recent_residual(item);
    let max_residual = oracle.max_residual();
    if max_residual > 0.0 {
        (residual * residual) / (max_residual * max_residual)
    } else {
        0.0
    }
}
```

High prediction error means the model is wrong. Observing the item provides correction data. Normalized to [0, 1].

### Daimon (affect engine)

```rust
/// Daimon values mood-congruent attention.
fn valuate_daimon(item: &TrackedItem, daimon: &Daimon) -> f64 {
    let pad_current = daimon.current_pad();
    let pad_assoc = daimon.pad_association(item);
    pad_current.cosine_similarity(&pad_assoc).abs()
}
```

Absolute cosine similarity between current PAD and item's emotional association. A fearful Golem bids high on fear-relevant items. A confident Golem bids high on success-associated items.

### Risk engine

```rust
/// Risk engine values attention on items correlated with positions.
fn valuate_risk(item: &TrackedItem, risk: &RiskEngine) -> f64 {
    let delta_cvar = risk.marginal_cvar_reduction(item);
    let portfolio_cvar = risk.portfolio_cvar();
    if portfolio_cvar > 0.0 {
        delta_cvar / portfolio_cvar
    } else {
        0.0
    }
}
```

Marginal CVaR reduction from monitoring. Concentrates on items whose unmonitored movement could cause the largest portfolio loss. When the portfolio is hedged, risk bids are low. When concentrated, correlated assets get high bids.

### Curiosity module

```rust
/// Curiosity values expected information gain.
fn valuate_curiosity(
    item: &TrackedItem,
    surprise_domain: &BayesianSurpriseDomain,
) -> f64 {
    surprise_domain.expected_kl(item).min(1.0)
}
```

KL divergence between expected posterior and current prior. Items where the Golem expects to learn the most get the highest curiosity bids.

### Mortality engine

```rust
/// Mortality values survival-relevant items.
fn valuate_mortality(
    item: &TrackedItem,
    mortality: &MortalityEngine,
) -> f64 {
    let urgency = 1.0 - mortality.composite_vitality();
    let relevance = mortality.item_relevance(item);
    urgency * relevance
}
```

Near-zero for healthy Golems. Dominates for dying Golems. A Golem in Conservation phase concentrates attention on survival-relevant items through bidding dynamics, not hard-coded rules.

---

## Budget Constraint from Mortality

The attention budget K is not fixed. It derives from remaining lifespan:

```
K = floor(remaining_ticks * attention_fraction)
```

As the Golem ages, K shrinks, and the auction forces harder choices. A young Golem with K = 200 can afford broad attention. An old Golem with K = 20 must concentrate.

The budget distributes across timescales:

| Timescale | Budget | What it allocates |
|-----------|--------|-------------------|
| Gamma | K * 0.6 | Raw perception: which observations to read |
| Theta | K * 0.3 | Cognition: which items to predict on |
| Delta | K * 0.1 | Consolidation: which memories to curate |

Each timescale runs its own auction. Gamma auctions run every 5-15 seconds. Theta auctions run every 30-120 seconds. Delta auctions run every ~50 theta-ticks.

---

## Implementation

### The Bidder trait

```rust
/// Trait implemented by each cognitive subsystem.
pub trait Bidder: Send + Sync {
    fn id(&self) -> &str;

    /// Compute valuations for all items in the universe.
    fn valuate(&self, universe: &[ItemId]) -> Vec<Bid>;
}

pub struct Bid {
    pub item: ItemId,
    pub value: f64,
}
```

### The VCG auction

```rust
pub struct VcgAuction {
    k_active: usize,
    k_watched: usize,
}

pub struct AuctionResult {
    pub allocation: Vec<(ItemId, Tier)>,
    pub payments: HashMap<String, f64>,
    pub welfare: f64,
}

impl VcgAuction {
    pub fn run(
        &self,
        universe: &[ItemId],
        bidders: &[&dyn Bidder],
    ) -> AuctionResult {
        // 1. Collect all bids.
        let mut aggregate: HashMap<ItemId, f64> = HashMap::new();
        let mut per_bidder: HashMap<String, HashMap<ItemId, f64>> = HashMap::new();

        for bidder in bidders {
            let bids = bidder.valuate(universe);
            let mut bidder_map = HashMap::new();
            for bid in &bids {
                *aggregate.entry(bid.item.clone()).or_default() += bid.value;
                bidder_map.insert(bid.item.clone(), bid.value);
            }
            per_bidder.insert(bidder.id().to_string(), bidder_map);
        }

        // 2. Sort items by aggregate value (O(n log n)).
        let mut items: Vec<_> = aggregate.into_iter().collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 3. Assign tiers.
        let mut allocation = Vec::new();
        for (idx, (item, _)) in items.iter().enumerate() {
            let tier = if idx < self.k_active {
                Tier::Active
            } else if idx < self.k_active + self.k_watched {
                Tier::Watched
            } else {
                Tier::Dormant
            };
            allocation.push((item.clone(), tier));
        }

        // 4. Compute welfare.
        let welfare: f64 = items.iter()
            .take(self.k_active)
            .map(|(_, v)| v)
            .sum();

        // 5. Compute VCG payments.
        let payments = self.compute_payments(
            &items, &per_bidder, bidders,
        );

        AuctionResult { allocation, payments, welfare }
    }

    fn compute_payments(
        &self,
        sorted_items: &[(ItemId, f64)],
        per_bidder: &HashMap<String, HashMap<ItemId, f64>>,
        bidders: &[&dyn Bidder],
    ) -> HashMap<String, f64> {
        let mut payments = HashMap::new();

        for bidder in bidders {
            let bid = bidder.id().to_string();
            // Welfare of others under actual allocation
            let others_actual: f64 = sorted_items.iter()
                .take(self.k_active)
                .map(|(item, total)| {
                    let my_bid = per_bidder.get(&bid)
                        .and_then(|m| m.get(item))
                        .copied()
                        .unwrap_or(0.0);
                    total - my_bid
                })
                .sum();

            // Welfare of others without this bidder (re-sort without their bids)
            let mut without: Vec<_> = sorted_items.iter()
                .map(|(item, total)| {
                    let my_bid = per_bidder.get(&bid)
                        .and_then(|m| m.get(item))
                        .copied()
                        .unwrap_or(0.0);
                    (item.clone(), total - my_bid)
                })
                .collect();
            without.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            let others_counterfactual: f64 = without.iter()
                .take(self.k_active)
                .map(|(_, v)| v)
                .sum();

            payments.insert(bid, others_counterfactual - others_actual);
        }

        payments
    }
}
```

### Complementarity extension

The additive model (v_b(S) = sum of v_b(i)) is a simplification. In practice, some items are complements: monitoring ETH-USDC and WBTC-USDC together is worth more than monitoring each separately because correlation structure between them carries information.

```rust
/// Complementarity bonus between two items for a given bidder.
pub trait ComplementarityBidder: Bidder {
    fn complementarity(&self, i: &ItemId, j: &ItemId) -> f64;
}
```

The risk engine sets complementarity proportional to the correlation between items. When two correlated assets are both in the portfolio, monitoring them together captures correlation risk. The greedy allocation algorithm handles complementarities at O(n * m * K) cost without solving the full combinatorial problem.

---

## CorticalState Signals

Two signals written by the auction:

| Signal | Type | Writer | Frequency |
|--------|------|--------|-----------|
| `auction_welfare` | `AtomicU32` (f32 bits) | `VcgAuction::run()` | Theta |
| `attention_budget` | `AtomicU32` (f32 bits) | `MortalityEngine` | Theta |

`auction_welfare` tracks total value of the current allocation. A declining trend means the Golem's attention is becoming less effective.

`attention_budget` tracks remaining K. Declining budget indicates the Golem is aging and must concentrate attention.

---

## Hierarchical Auctions

Each timescale runs its own auction:

**Gamma auction** (every 5-15s): Allocates raw perception. Which observations to read from the chain. Runs in the perception layer before Step 1 (OBSERVE). Budget: K * 0.6.

**Theta auction** (every 30-120s): Allocates prediction resources. Which items get full prediction depth. Runs before Step 3 (ANALYZE). Budget: K * 0.3.

**Delta auction** (every ~50 theta-ticks): Allocates consolidation resources. Which memories to curate, which patterns to consolidate during dreams. Runs during Step 9 (REFLECT). Budget: K * 0.1.

The theta auction is the primary one. Gamma and delta auctions use simplified versions (fewer bidders, cheaper valuation computation).

---

## Diagnostic Value

The VCG payments answer questions the EFE composite cannot:

- "Why did the Golem ignore ETH-USDC during the volatility spike?" -- the payment history shows which bidder displaced it and at what opportunity cost.
- "Is the risk engine dominating attention too aggressively?" -- high risk engine payments indicate it's claiming slots that would have gone to curiosity.
- "Is mortality pressure changing attention patterns?" -- mortality payments spike near phase boundaries.

These diagnostics flow to the TUI and are logged in the `DecisionCycleRecord` for dream replay.

---

## Problem Statements

> Source: `innovations/05-attention-as-auction.md`

The current scoring formula has four concrete failure modes:

**1. Risk signals drowned by curiosity.** A new DeFi protocol appears. Information gain is high -- the Golem knows nothing about it, so every observation reduces uncertainty. The protocol dominates the attention ranking. Meanwhile, the Golem holds a large ETH-USDC position, and ETH volatility is rising. The risk of that position demands attention. But the risk engine has no way to express "I need you to watch ETH-USDC right now" except through the same scoring formula, where the novelty of the new protocol outweighs the risk of the existing position.

**2. Affective bias.** The Daimon (Bardo's affect engine) modulates risk tolerance through PAD vectors. A fearful Golem should probably be cautious. But the current system has no mechanism for affect to influence attention allocation. Either affect is wired into the scoring weights (making the Golem's attention permanently mood-dependent) or it is not (making the Golem's attention emotionally deaf).

**3. Mortality cannot claim priority.** When the economic clock is declining, the Golem should shift attention toward items that could arrest the decline -- high-yield opportunities, positions that can be unwound, gas cost optimization. The mortality engine knows which items matter for survival. It has no way to tell the Attention Forager.

**4. Fixed weights cannot adapt.** In low-volatility periods, curiosity-driven exploration is cheap and potentially profitable. In a crisis, risk monitoring dominates everything. The fixed weights cannot adapt to these shifts.

---

## Full Valuation Functions

### Oracle (prediction engine)

```
v_oracle(i) = prediction_residual(i)^2 / max_residual
```

High prediction error means the Oracle's model is wrong about item i. Observing i provides the data needed to correct the model. Normalized by the maximum residual across all items, placing values in [0, 1]. When the Oracle is accurate everywhere, its bids are low. When it is missing something, its bids concentrate on the gap.

### Daimon (affect engine)

```
v_daimon(i) = |dot(PAD_current, PAD_association(i))| / (||PAD_current|| * ||PAD_association(i)||)
```

Absolute cosine similarity between current PAD and item's emotional association. The absolute value matters. A fearful Golem should attend to fear-relevant items whether the association is positive or negative. The sign determines how the attention is used (approach vs. avoidance), not whether attention is warranted.

### Risk engine

```
v_risk(i) = delta_CVaR(i) / portfolio_CVaR
```

Conditional value-at-risk reduction from monitoring item i, estimated from the position correlation matrix. Items with high delta_CVaR are those whose unmonitored movement could cause the largest portfolio loss. Naturally concentrates risk attention on concentrated positions and correlated assets.

### Curiosity module

```
v_curiosity(i) = D_KL(posterior(i) || prior(i))
```

The KL divergence between the expected posterior (after observing i) and the current prior. Items where the Golem expects to learn the most get the highest curiosity bids. In the old system, curiosity competed with risk through fixed weights. In the auction, curiosity competes with risk through bids.

### Mortality engine

```
v_mortality(i) = urgency * relevance(i)
```

where:
- `urgency = 1.0 - vitality` (from the information-theoretic mortality framework). A healthy Golem has near-zero urgency. A dying Golem has near-maximum urgency.
- `relevance(i)` measures how much item i could affect the declining mortality axis. If the economic clock is declining, items that could generate revenue get high relevance. If the epistemic clock is declining, items with high model uncertainty get high relevance.

---

## Budget Constraint from Mortality (Extended)

The attention budget K is not a fixed parameter. It derives from the Golem's remaining lifespan:

```
K = floor(remaining_ticks * attention_fraction)
```

where:
- `remaining_ticks` is estimated from the three mortality clocks (or from the unified MI vitality)
- `attention_fraction` is the fraction of total remaining cognitive capacity allocated to the current theta cycle, approximately `1 / remaining_theta_ticks`

As the Golem ages, remaining_ticks shrinks, K shrinks, and the auction forces harder choices. A young Golem with K = 200 can afford broad attention. An old Golem with K = 20 must concentrate. The total attention budget across timescales:

```
K_gamma = floor(K * gamma_fraction)    // perception: which observations to read
K_theta = floor(K * theta_fraction)    // cognition: which items to predict on
K_delta = floor(K * delta_fraction)    // consolidation: which memories to curate
```

### Three-Timescale Decomposition

**Gamma auction (perception, 5-15 seconds):** K_gamma is larger (more items can be passively observed than actively analyzed). Typical K_gamma: 100-300 items. The Oracle bids based on prediction staleness. The risk engine bids based on position exposure. The mortality engine bids based on survival urgency. The Daimon and curiosity module typically abstain at gamma scale.

**Theta auction (cognition, 30-120 seconds):** The primary allocation mechanism. All five bidders participate. Budget K_theta: 20-100 items.

**Delta auction (consolidation, ~50 theta-ticks):** Decides which experiences to commit to the knowledge layer. The Oracle bids to remember high-error episodes (training data). The Daimon bids to remember emotionally significant events. The mortality engine bids to remember near-death experiences. Budget K_delta: 5-20 items.

---

## Complementarities: Formal Treatment

The additive assumption (v_b(S) = sum of v_b(i) for i in S) is a simplification. In practice, some items are complements: monitoring ETH-USDC and WBTC-USDC together is worth more than monitoring each separately, because the correlation structure carries information that neither alone provides.

VCG handles complementarities through set-based valuations. A bidder can report v_b({ETH-USDC, WBTC-USDC}) > v_b({ETH-USDC}) + v_b({WBTC-USDC}). The mechanism still finds the welfare-maximizing allocation. The cost is computational: with arbitrary set valuations, the allocation problem is NP-hard.

For the Golem, a complementarity bonus:

```
v_b(S) = sum_{i in S} v_b(i) + sum_{(i,j) in S x S} c_b(i, j)
```

where c_b(i, j) >= 0 is the complementarity between items i and j for bidder b. The risk engine sets c_risk(i, j) proportional to the correlation between items i and j. If i and j are highly correlated and the Golem holds positions in both, monitoring them together captures correlation risk that monitoring either alone would miss.

The greedy allocation algorithm handles complementarities at O(n * m * K) cost:

```rust
/// Greedy allocation with complementarity bonuses.
pub fn greedy_allocate_with_complements(
    universe: &[ItemId],
    bid_matrix: &[Vec<f64>],
    complements: &HashMap<(usize, usize), f64>,
    k: usize,
) -> Vec<usize> {
    let n = universe.len();
    let mut selected: Vec<usize> = Vec::with_capacity(k);
    let mut available: Vec<bool> = vec![true; n];

    for _ in 0..k.min(n) {
        let mut best_idx = None;
        let mut best_marginal = f64::NEG_INFINITY;

        for i in 0..n {
            if !available[i] {
                continue;
            }

            // Base value from all bidders.
            let base: f64 = bid_matrix[i].iter().sum();

            // Complementarity bonus with already-selected items.
            let complement_bonus: f64 = selected.iter()
                .filter_map(|&j| {
                    let key_a = (i.min(j), i.max(j));
                    complements.get(&key_a)
                })
                .sum();

            let marginal = base + complement_bonus;

            if marginal > best_marginal {
                best_marginal = marginal;
                best_idx = Some(i);
            }
        }

        if let Some(idx) = best_idx {
            selected.push(idx);
            available[idx] = false;
        }
    }

    selected
}
```

The greedy approach gives a (1 - 1/e)-approximation to the optimal welfare for submodular valuations (Nemhauser et al., 1978).

---

## Performance Analysis

Additive case (no complementarities):
- **Time:** O(n * m) for bid collection + O(n log n) for sort + O(n * m^2) for VCG payments. For n = 500, m = 5: ~15,000 operations. Wall time < 0.1ms.
- **Memory:** O(n * m) for bid matrix = 500 * 5 * 8 bytes = 20 KB.

Complementarity case:
- **Time:** O(n * K * m) for greedy + O(n * m^2 * K) for VCG payments. Worst case for n = 500, K = 50, m = 5: ~625,000 operations. Wall time < 1ms.
- **Memory:** O(n^2) for complement matrix in the worst case, but sparse in practice. With ~5% density: ~12,500 entries * 16 bytes = 200 KB.

Both fit within the gamma tick budget with room to spare.

---

## What This Enables

> Source: `innovations/05-attention-as-auction.md`, Section 6

**Emergent attention patterns.** The Golem's attention becomes an emergent property of competing cognitive demands rather than a centrally planned allocation. In calm markets, curiosity dominates -- the Golem explores broadly. In volatile markets, risk dominates -- the Golem focuses on its positions. As death approaches, mortality dominates -- the Golem narrows to survival. No one programmed these behaviors. They arise from the auction dynamics.

**Diagnosable decisions.** Every attention decision has a paper trail. Why did the Golem ignore AAVE-USDC on March 3? Because the risk engine bid 0.72 on ETH-USDC and the curiosity module bid 0.31 on AAVE-USDC, and with K = 40, AAVE-USDC ranked 43rd. This is auditable in a way that "the heuristic score was 0.47" never is.

**Automatic adaptation.** The auction adapts to market conditions without parameter tuning. The weights in the old scoring formula were fixed. In the auction, the effective "weights" are the bidders' valuations, which change every tick based on the Golem's state and the market's state. A regime change does not require a parameter update -- the bidders' valuations shift, and the allocation follows.

**Strategy-proof composition.** Because VCG is strategy-proof, subsystem developers do not need to reason about what other subsystems will bid. The Oracle team can implement a valuation function based purely on prediction residuals, without worrying that the risk engine will inflate its bids to crowd out Oracle priorities. Each subsystem reports truth. The mechanism handles the rest.

**Mortality-driven scarcity.** The budget constraint K links attention to survival. A thriving Golem with 10,000 remaining ticks has broad attention. A declining Golem with 500 remaining ticks has narrow attention. The narrowing is smooth, continuous, and driven by the same mortality framework that determines the Golem's behavioral phase. Attention scarcity is not an arbitrary parameter. It is a consequence of being mortal.

---

## Evaluation and Falsifiability

**Null hypothesis:** VCG attention allocation does not outperform weighted scoring on prediction accuracy or risk-adjusted returns.

**Metric 1: Prediction accuracy per unit of attention.** Measure the total prediction error reduction per ACTIVE slot per theta tick. VCG should allocate attention to items where the marginal error reduction is highest.

**Metric 2: Risk-adjusted returns during stress.** During the volatility regime change, compare Sharpe ratios. The VCG Golem should shift attention to risk-relevant items faster than the weighted scorer because the risk engine can outbid curiosity directly.

**Metric 3: Mortality outcomes.** Track how many ticks before catastrophic loss (>50% drawdown) each system triggers Conservation phase.

**Metric 4: Strategy-proofness verification.** Artificially inflate one bidder's valuations by 2x and verify that (a) the inflating bidder does not improve its allocation (it already reported truth under VCG) and (b) other bidders' allocations do not degrade beyond expected reallocation.

**Falsification conditions:**

1. Weighted scoring achieves equal or higher prediction accuracy per attention slot across > 60% of test windows.
2. Risk-adjusted returns during stress are not statistically different (p > 0.05) between the two systems.
3. The mortality engine's ability to bid for attention does not produce earlier Conservation warnings than the weighted scorer.
4. Computational overhead of the VCG mechanism exceeds 10% of gamma tick budget.

---

## Philosophical Grounding

### Kahneman's Capacity Model

Kahneman (1973) proposed that attention is a limited-capacity resource with an allocation policy. The policy is not fixed -- it responds to task demands, arousal, and the consequences of failure. The VCG auction is an explicit allocation policy. The bidders represent Kahneman's "task demands" (Oracle, Curiosity), "arousal" (Daimon), and "consequences of failure" (Risk, Mortality). The budget K is the capacity constraint.

### Simon's Attention Poverty

Herbert Simon (1971): "A wealth of information creates a poverty of attention and a need to allocate that attention efficiently among the overabundance of information sources that might consume it." A Golem's universe of 50-500 candidate observations is Simon's "wealth of information." The ACTIVE tier of 20-100 slots is the attention budget. Simon identified the need. VCG provides the mechanism.

### Mortality and Meaning

An immortal agent with unlimited attention has no allocation problem. It can observe everything, predict everything, remember everything. Attention becomes meaningful only when it is scarce, and scarcity is a consequence of mortality. The VCG auction formalizes this. The Golem must choose. The mortality clocks enforce the constraint. The auction computes the optimal choice given that constraint. As the Golem approaches death, K shrinks, the choices become harder, and the allocation becomes more focused.

### Markets as Computation (Hayek)

Hayek (1945) argued that market prices aggregate dispersed knowledge that no single planner could collect. The VCG auction is a market inside the Golem's mind. Each subsystem has local knowledge. No single scoring formula can aggregate this knowledge correctly because the relative importance of each signal changes with conditions. The auction aggregates it through bids.

The VCG payment mechanism adds something Hayek's markets do not guarantee: strategy-proofness. Real markets are gameable. The internal market of the Golem's attention cannot be gamed because VCG makes truthful bidding the dominant strategy.

---

## References

- [VICKREY-1961] Vickrey, W. "Counterspeculation, Auctions, and Competitive Sealed Tenders." *Journal of Finance*, 16(1), 1961. — *Introduces the second-price sealed-bid auction where truthful bidding is a dominant strategy; the core mechanism adapted for attention slot allocation.*
- [CLARKE-1971] Clarke, E.H. "Multipart Pricing of Public Goods." *Public Choice*, 11, 1971. — *Extends Vickrey's mechanism to multi-item settings with externality pricing; completes the VCG framework used to price context slots.*
- [GROVES-1973] Groves, T. "Incentives in Teams." *Econometrica*, 41(4), 1973. — *Proves that pivotal mechanisms make truthful reporting a dominant strategy in team settings; the incentive-compatibility guarantee for the attention auction.*
- [NISAN-2007] Nisan, N. et al. *Algorithmic Game Theory*. Cambridge University Press, 2007. — *Comprehensive treatment of mechanism design, combinatorial auctions, and computational complexity of allocation problems; the textbook foundation for the auction's design choices.*
- [FRISTON-2017] Friston, K. et al. "Active Inference: A Process Theory." *Neural Computation*, 29(1), 2017. — *Specifies expected free energy as the objective for action selection; provides the theoretical link between attention allocation and free energy minimization.*
- [SIMS-2003] Sims, C.A. "Implications of Rational Inattention." *Journal of Monetary Economics*, 50(3), 2003. — *Models agents as having finite information-processing capacity, forcing selective attention; the economic theory motivating why a Golem needs an attention auction at all.*
- [KAHNEMAN-1973] Kahneman, D. *Attention and Effort*. Prentice-Hall, 1973. — *Proposes attention as a limited-capacity resource with a central allocation policy; the cognitive science model behind the auction's capacity budget.*
- [SIMON-1971] Simon, H.A. "Designing Organizations for an Information-Rich World." In: *Computers, Communications, and the Public Interest*, ed. M. Greenberger. Johns Hopkins University Press, 1971. — *Argues that information abundance creates attention poverty; the original insight that attention, not data, is the scarce resource a Golem must manage.*
- [MILGROM-2004] Milgrom, P. *Putting Auction Theory to Work*. Cambridge University Press, 2004. — *Practical guide to designing real-world auctions, including combinatorial and package bidding; informs the auction's handling of complementary context blocks.*
- [HAYEK-1945] Hayek, F.A. "The Use of Knowledge in Society." *American Economic Review*, 35(4), 1945. — *Argues that prices aggregate dispersed knowledge better than central planning; justifies using auction prices rather than fixed rules to allocate attention across competing information sources.*
- [NEMHAUSER-1978] Nemhauser, G.L., Wolsey, L.A. & Fisher, M.L. "An Analysis of Approximations for Maximizing Submodular Set Functions." *Mathematical Programming*, 14(1), 1978. — *Proves the greedy algorithm achieves a (1 - 1/e) approximation for submodular maximization; the performance guarantee for the auction's greedy allocation fallback.*

---
