<!-- prd2/23-ta/04-causal-microstructure-discovery.md -->
<!-- Source: tmp/research/witness-research/new/ta/04-causal-microstructure-discovery.md -->

> **Version**: 1.0 | **Status**: Active | **Section**: 23-ta
>
> **Crates**: `bardo-ta-causal`
>
> **Cross-references**:
> - [01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- the 32-signal perception surface where `causal_density` tracks the richness of the discovered causal graph
> - [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- the 9-step decision cycle whose Gamma ticks update causal variables and Theta ticks run structural learning
> - [14-chain/03-protocol-state.md](../14-chain/03-protocol-state.md) -- live protocol state providing the causal variable source (pool reserves, rates, positions) for structural discovery
> - [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- HDC foundations for encoding causal edges as hypervector bindings that support efficient graph queries
> - [23-ta/00-witness-as-technical-analyst.md](00-witness-as-technical-analyst.md) -- Doc 0: system context including mirage-rs fork simulation used for interventional causal testing
> - [23-ta/01-hyperdimensional-technical-analysis.md](01-hyperdimensional-technical-analysis.md) -- Doc 1: HDC pattern algebra providing the encoding substrate for causal graph representation
> - [23-ta/02-spectral-liquidity-manifolds.md](02-spectral-liquidity-manifolds.md) -- Doc 2: manifold curvature events as candidate causal signals in the structural learning graph
> - [23-ta/03-adaptive-signal-metabolism.md](03-adaptive-signal-metabolism.md) -- Doc 3: evolutionary signal system where causal signals serve as fitness inputs for signal selection
> - [23-ta/05-predictive-geometry.md](05-predictive-geometry.md) -- Doc 5: topology-to-trajectory forecasting that uses causal constraints to prune impossible prediction paths

---

> **Reader orientation:** This document specifies how the Golem (mortal autonomous DeFi agent) discovers causal relationships in DeFi microstructure, climbing Pearl's causal hierarchy from observation to intervention to counterfactual reasoning. It belongs to the **TA research layer** (Doc 4 of 10) and covers the PC algorithm for structural learning, Granger causality for temporal edges, mirage-rs fork simulation for interventional testing, and HDC encoding of causal graphs. You should understand causal inference basics and on-chain transaction ordering. For Bardo-specific terms, see `prd2/shared/glossary.md`.

# Causal Microstructure Discovery [SPEC]

Audience: Systems engineers building autonomous DeFi agents. Assumes familiarity with Bardo's heartbeat cycle, the Witness DAG, Mirage-rs fork simulation, and the HDC/BSC encoding from Doc 1 of this series. Prior exposure to causal inference helps but is not required; we build the theory from scratch.

---

## Abstract

DeFi protocols interact through shared liquidity, oracle dependencies, and arbitrage pathways, but the causal structure of these interactions is hidden. Correlation-based technical analysis cannot distinguish "X predicts Y" from "X causes Y," and the difference matters when a Golem must decide whether intervening on X will move Y. This document presents a causal discovery engine for the Bardo runtime that operates at all three levels of Pearl's causal hierarchy: observational discovery via the PC algorithm on streaming protocol data, interventional discovery via dream-based do(X) experiments on Mirage-rs fork state, and counterfactual reasoning via structural equation inversion. The engine maintains a live directed acyclic graph of causal relationships across DeFi primitives, with typed edges carrying strength coefficients, confidence scores subject to Ebbinghaus decay, and cryptographic provenance through the Witness DAG. We provide complete Rust implementations of the causal graph, the PC algorithm with conditional independence testing, Granger causality for irregular event-time series, an interventional dream designer, and the orchestrating CausalDiscoveryEngine that integrates with Bardo's Gamma/Theta/Delta heartbeat and NREM/REM dream cycles. Cross-primitive causal hypotheses (large swaps causing lending rate shifts, vault outflows predicting LP removal, gas spikes triggering liquidation cascades) are mapped to specific discovery protocols. The system compounds causal knowledge across dreams and generations, giving each Golem a progressively richer model of DeFi's hidden wiring.

---

## The problem

A Golem watches Uniswap V3. A whale dumps 5,000 ETH into the WETH/USDC pool. Three blocks later, the Aave ETH lending rate spikes 40 basis points. The Golem's correlation-based models see the co-movement. They note the pattern. Next time a large swap appears, they predict a rate spike.

But the prediction fails. It fails because the correlation was not a direct causal link. The swap moved the ETH spot price, which pushed several Aave borrowers closer to their liquidation thresholds, which increased utilization as borrowers scrambled to repay, which triggered the rate model's utilization kink. The causal chain has four steps, two confounders (gas price affecting transaction ordering, MEV bots front-running the liquidations), and a feedback loop (higher rates incentivize more supply, which dampens the rate). A system that only measures P(rate_spike | large_swap) will get the direction right sometimes and the magnitude wrong almost always.

This is the core limitation of Level 1 reasoning in Pearl's causal hierarchy. The Golem sees correlations. It cannot answer: "If I inject a large swap, will the rate move?" (Level 2, intervention). It cannot answer: "Given that the rate did move, would it have stayed flat if the swap hadn't happened?" (Level 3, counterfactual). And without those answers, it cannot reason about strategy. Should it front-run the rate spike by depositing ETH on Aave before executing a large swap on Uniswap? That question requires causal knowledge, not statistical pattern matching.

The problem has several dimensions that make DeFi harder than the textbook causal inference setting.

**Non-stationarity.** The causal graph changes. When Aave governance modifies the interest rate model parameters, the edge from utilization to borrow rate changes its functional form. When a new Chainlink oracle feed goes live, new edges appear. When a lending protocol adds a new collateral type, the graph topology shifts. Any discovery algorithm must treat the graph as a living object, not a one-time computation.

**Irregular timestamps.** Ethereum blocks arrive every 12 seconds on average, but actual block times vary, and the events within a block have no meaningful sub-block ordering (they are ordered by the builder, not by causation). Cross-chain interactions through bridges add variable delays from minutes to hours. Standard time-series causal methods assume regular sampling intervals. DeFi gives us event streams with irregular spacing and batch-processed state transitions.

**Confounders everywhere.** Gas prices affect every protocol simultaneously. MEV searcher activity creates correlated behavior across pools. Macro events (rate decisions, regulatory news) move everything at once. The Golem cannot observe these exogenous forces directly; they appear as latent confounders that create spurious edges in any naive discovery algorithm.

**Feedback loops.** DeFi protocols are cybernetic systems. Lending rates affect borrowing demand, which affects utilization, which affects rates. AMM prices affect arbitrage, which affects prices. Vault yields affect TVL flows, which affect yields. These feedback loops create cycles in the causal graph, which violates the DAG assumption that most discovery algorithms require. The engine must handle cycles explicitly, either by time-indexing (the graph is a DAG when you condition on time) or by identifying and modeling the feedback structure.

**Scale.** A Golem tracking 10 protocols with 5 metrics each has 50 variables. The number of possible directed edges is 50 * 49 = 2,450. The number of possible DAGs over 50 nodes is superexponential. Even with efficient algorithms, discovery over the full variable set is expensive, and the Golem must do it incrementally at heartbeat timescales.

Pearl's causal hierarchy gives us the framework. Three levels, each strictly more powerful than the last:

**Level 1: Association.** P(Y | X). What is the probability of Y given that we observe X? This is correlation, mutual information, regression. Every standard TA indicator operates here. You see a pattern, you predict from it. No claim about what happens if you intervene.

**Level 2: Intervention.** P(Y | do(X)). What is the probability of Y if we force X to take a specific value? The do-operator is Pearl's formalization of intervention: it severs all incoming edges to X in the causal graph (because we set X; its causes no longer matter) and propagates the effect downstream. This is the level that answers strategy questions. If I execute this swap, what happens to rates?

**Level 3: Counterfactual.** P(Y_x | X', Y'). Given that we observed X' and Y', what would Y have been if X had been x instead? This requires a full structural causal model with specified functional forms and noise terms. Counterfactuals answer the hardest questions: "Was this liquidation cascade caused by the oracle update, or would it have happened anyway?"

Most trading systems, including most autonomous agents, operate at Level 1. Some use Granger causality or transfer entropy, which gets partway toward Level 2 but remains observational. This document gives the Golem all three levels, using a combination of classical causal discovery algorithms, Bardo's dream system for simulated interventions, and structural equation models for counterfactual reasoning.

---

## Mathematical foundations [SPEC]

### Structural causal models

A structural causal model (SCM) is a triple M = (U, V, F) where:

- U is a set of exogenous (external) variables. In DeFi: whale behavior, macro events, gas market dynamics, MEV searcher strategies. These are variables the Golem cannot observe directly or can only observe through proxies.
- V = {V_1, ..., V_n} is a set of endogenous variables. Each represents an observable DeFi quantity: swap volume on a specific pool, lending rate on a specific market, TVL of a specific vault, funding rate on a perpetual contract, and so on.
- F = {f_1, ..., f_n} is a set of structural equations. Each equation specifies V_i as a deterministic function of its parents and its exogenous noise term: V_i = f_i(pa(V_i), U_i), where pa(V_i) is the set of endogenous variables that directly cause V_i.

The causal graph G is the directed graph induced by F: draw an edge from V_j to V_i whenever V_j appears in f_i. If G is acyclic, M is a recursive SCM and the standard do-calculus applies directly. If G has cycles (feedback loops), we need the cyclic SCM extensions from Bongers et al. (2021) or we time-index the variables to break the cycles.

An example SCM for a Uniswap-Aave interaction:

```
V_1 = swap_volume       = f_1(U_whale, U_arb)
V_2 = spot_price        = f_2(V_1, U_market)
V_3 = collateral_value  = f_3(V_2)
V_4 = utilization       = f_4(V_3, V_6, U_demand)
V_5 = borrow_rate       = f_5(V_4)                    // Aave rate model
V_6 = borrow_demand     = f_6(V_5, U_demand)          // feedback loop
```

The graph has a cycle: V_4 -> V_5 -> V_6 -> V_4. To make this a DAG, we time-index: V_4(t) depends on V_6(t-1), not V_6(t). The feedback loop becomes a temporal chain. This is the standard trick, and it matches DeFi reality: the Aave rate model updates are discrete (once per transaction), so the feedback genuinely operates across time steps.

The Golem's task: discover the graph G from data, estimate the structural equations F, and use them for intervention and counterfactual queries.

### The do-operator and interventional distributions

Pearl's do-operator formalizes the difference between observation and intervention. Observing X = x gives us P(Y | X = x), which includes information from all paths between X and Y, including backdoor paths through confounders. Intervening to set X = x gives us P(Y | do(X = x)), which cuts all incoming edges to X and measures only the downstream causal effect.

The adjustment formula (Pearl's backdoor criterion) computes interventional distributions from observational data when a suitable adjustment set Z exists:

```
P(Y | do(X = x)) = sum_z P(Y | X = x, Z = z) * P(Z = z)
```

This works when Z blocks all backdoor paths from X to Y. Finding such a Z requires knowing the causal graph, which is what we are trying to discover. This circular dependency is why the Golem needs multiple discovery methods: observational algorithms give a candidate graph, then interventional experiments (via dreams) confirm or refute specific edges.

When no valid adjustment set exists (because of unobserved confounders), do-calculus provides three inference rules that can sometimes identify the causal effect through chains of conditional independencies and interventions. The rules are mechanical and can be automated; Tian and Pearl (2002) showed that a complete identification algorithm exists for any SCM with the graph structure known.

For the Golem, the practical implication: once it has a candidate causal graph from the PC algorithm, it can check whether specific causal effects are identifiable from observational data alone. When they are not, it queues an interventional dream experiment. This is the bridge between Level 1 and Level 2.

### Observational discovery: the PC algorithm

The PC (Peter-Clark) algorithm, introduced by Spirtes, Glymour, and Scheines (2000), discovers the causal graph skeleton from conditional independence (CI) tests on observational data. It operates in two phases.

**Phase 1: Skeleton discovery.** Start with a complete undirected graph over all n variables. For each pair (X, Y), test conditional independence X _||_ Y | Z for conditioning sets Z of increasing size:

1. Size 0: Test X _||_ Y (marginal independence). If independent, remove the edge X -- Y.
2. Size 1: For each remaining edge X -- Y, test X _||_ Y | Z for each Z in adj(X) \ {Y}. If any Z renders them independent, remove the edge and record Z as the separating set sep(X, Y).
3. Size k: Continue with conditioning sets of size k drawn from the current adjacency set.
4. Stop when the maximum conditioning set size exceeds the maximum adjacency degree.

The computational cost is dominated by the CI tests. In the worst case (dense graph), the number of tests is O(n^2 * 2^n), but for sparse graphs (which DeFi causal structures tend to be, since most protocols interact with only a few others), the cost is O(n^2 * d^k) where d is the maximum degree and k is the maximum conditioning set size.

**Phase 2: Edge orientation.** Given the skeleton and separating sets, orient edges using three rules:

Rule 1 (v-structure detection): If X -- Z -- Y, X and Y are not adjacent, and Z is not in sep(X, Y), then orient as X -> Z <- Y. This is a collider: Z is a common effect of X and Y.

Rule 2 (acyclicity): If X -> Z -- Y and X and Y are not adjacent, orient as Z -> Y (otherwise we create a new v-structure or a cycle).

Rule 3 (no new colliders): If X -- Z, X -- Y, Y -> Z, and X and Y are not adjacent, orient as X -> Z.

These rules do not orient all edges. The result is a completed partially directed acyclic graph (CPDAG) representing the Markov equivalence class of DAGs consistent with the observed independencies. Some edges remain undirected, meaning the data cannot distinguish between the two possible orientations. This is where interventional experiments become necessary: a targeted do(X) experiment can orient an ambiguous edge by checking whether X -> Y or Y -> X.

**Conditional independence testing.** The CI test is the engine's most critical component. Two options:

*Partial correlation test (Fisher's z-transform).* For continuous Gaussian variables, X _||_ Y | Z iff the partial correlation rho_{XY|Z} = 0. The test statistic is:

```
z = 0.5 * sqrt(n - |Z| - 3) * ln((1 + rho) / (1 - rho))
```

which is approximately N(0, 1) under the null hypothesis. This is fast (O(|Z|^3) for the matrix inversion) but assumes linearity and Gaussianity. DeFi data is neither linear nor Gaussian, but the test is a reasonable starting point and runs within the Theta tick budget.

*Conditional mutual information (CMI).* For non-linear relationships:

```
I(X; Y | Z) = sum_{x,y,z} p(x, y, z) * log(p(x, y | z) / (p(x | z) * p(y | z)))
```

CMI = 0 iff X _||_ Y | Z, regardless of the functional form. Estimation requires density estimation (kernel density, k-NN, or binning), which is more expensive but handles non-linear dependencies. Runge (2018) introduced the PCMCI algorithm, which combines the PC algorithm with momentary conditional independence (MCI) testing for time series, using CMI estimated via the Kraskov-Stogbauer-Grassberger (KSG) k-nearest-neighbor estimator.

For the Golem, we implement both. The partial correlation test runs at Theta tick for fast incremental updates. The CMI test runs at Delta tick for a thorough audit of the graph structure.

### Extended Granger causality for DeFi

Standard Granger causality (Granger, 1969): X Granger-causes Y if past values of X improve prediction of Y beyond what Y's own past provides. Formally:

```
X ->_G Y  iff  P(Y_t | Y_{t-1..t-k}, X_{t-1..t-k}) != P(Y_t | Y_{t-1..t-k})
```

In practice, this is tested by fitting two VAR models (one restricted, one unrestricted) and comparing residual variances with an F-test.

Granger causality is not true causality. It measures predictive precedence, not structural causation. A confounded pair (X <- Z -> Y, with Z unobserved) can show Granger causality in both directions. But Granger causality has practical value as a pre-filter: edges that fail the Granger test are unlikely to be causal, so the PC algorithm can skip them.

DeFi requires four extensions to the standard test:

**1. Irregular timestamps.** Ethereum events do not arrive at fixed intervals. Two approaches: (a) resample to a regular grid (e.g., per-block or per-Gamma-tick) with forward-fill for missing values, or (b) use event-time Granger causality where the lag is measured in events, not clock time. Option (b) is more natural for DeFi but requires a custom test implementation. We implement (b) by maintaining per-variable event queues and computing lagged regressions over event indices.

**2. Multivariate conditioning.** Bivariate Granger causality (testing X -> Y while ignoring all other variables) produces false positives when a confounder Z causes both. Multivariate Granger causality conditions on all other observed variables:

```
X ->_G Y | Z  iff  P(Y_t | Y_{<t}, X_{<t}, Z_{<t}) != P(Y_t | Y_{<t}, Z_{<t})
```

This is the VAR formulation. The practical limitation is that with 50 variables, the VAR model has 50 * 50 * k parameters (for lag order k), which requires substantial data to estimate reliably. We mitigate this with L1-regularized (Lasso) VAR estimation, which enforces sparsity.

**3. Non-linear extensions.** Linear VAR misses non-linear causal relationships (e.g., the Aave rate model's utilization kink, where rates are flat below 80% utilization and spike above it). Two options: (a) kernel Granger causality using reproducing kernel Hilbert spaces (Marinazzo et al., 2008), or (b) transfer entropy, which is the information-theoretic analogue of Granger causality:

```
TE(X -> Y) = I(Y_t; X_{<t} | Y_{<t})
```

Transfer entropy equals Granger causality for Gaussian processes but captures non-linear dependencies in general. We implement transfer entropy using the KSG estimator, the same as for CMI in the PC algorithm.

**4. Time-varying structure.** Causal relationships in DeFi form and dissolve. A new lending market creates new edges. A governance parameter change alters edge strengths. A bridge exploit severs cross-chain edges. We use sliding-window Granger tests: compute the test statistic over a rolling window of the most recent W observations (where W is configurable, defaulting to 500 events). When an edge's Granger p-value drops below the significance threshold in the current window, it is marked as newly active. When it rises above the threshold, the edge is flagged as potentially stale.

### Interventional discovery: dream-based do(X) experiments

The PC algorithm and Granger causality operate on observational data. They can discover the graph skeleton and orient some edges, but they cannot distinguish between Markov-equivalent DAGs without interventional data. The Golem cannot intervene on mainnet: executing a large swap to see if it moves lending rates would cost real capital and real slippage.

But the Golem dreams.

During REM sleep (the creative, counterfactual dream phase), the Golem has access to Mirage-rs, the fork simulation engine. Mirage-rs forks current mainnet state, and the Golem can execute arbitrary transactions against the fork. This is a perfect simulation of do(X): force a variable to a specific value by executing the transaction that would produce that value, then observe downstream effects.

The do(X) experiment protocol:

1. **Select a hypothesis.** Choose an edge X -> Y from the candidate graph that is either (a) unoriented in the CPDAG, (b) low-confidence, or (c) newly discovered and unconfirmed.

2. **Design the intervention.** Determine the transaction(s) that would set X to a specific value. For "large_swap -> rate_change," the intervention is a simulated swap of a specified size on the target pool.

3. **Fork the state.** Create a Mirage-rs CoW branch from the current mainnet state. This costs ~12.8 KB of memory, not a full state clone.

4. **Execute the intervention.** Run the transaction on the fork. This sets V_X to the post-intervention value while keeping all other state at its pre-intervention mainnet values. This is exactly do(X = x): we set X, so its causes (whatever they were on mainnet) do not matter.

5. **Simulate downstream blocks.** Replay the next N blocks (typically 5-20) on the fork, using either actual pending mempool transactions (for near-future simulation) or synthetic transaction sequences sampled from historical distributions.

6. **Measure effects.** Compare Y (and all other tracked variables) between the intervention fork and a control fork (same block replay, no intervention). The difference is the estimated causal effect of X on Y.

7. **Record evidence.** Create a Witness DAG vertex linking the experiment design, the fork state, the intervention transaction, the observed outcomes, and the inferred causal edge. This provides cryptographic provenance for the causal claim.

The power of dream-based intervention is that it breaks confounders. On mainnet, a large swap and a lending rate change might both be caused by the same macro event. In the dream, we force the swap and observe whether the rate moves independently. If it does, the edge X -> Y has interventional support. If it does not (the rate stays flat despite the swap), the observational correlation was confounded and the edge should be weakened or removed.

**Counterfactual queries (Level 3).** Given an SCM with estimated structural equations, counterfactual reasoning proceeds in three steps (Pearl's "abduction-action-prediction" method):

1. **Abduction:** Given the observed evidence (X = x', Y = y'), compute the posterior distribution over the exogenous noise terms U.
2. **Action:** In the modified model, set X = x (the counterfactual intervention).
3. **Prediction:** Propagate through the structural equations with the abduced noise terms to compute Y_x.

This requires parametric structural equations, which the Golem estimates from its observational and interventional data. The estimates are imprecise, so counterfactual queries carry wider confidence intervals than interventional ones. But they answer questions that nothing else can: "Would this liquidation have happened if the oracle hadn't updated?" The Golem uses counterfactual reasoning primarily for post-hoc analysis during NREM dreams (retrospective replay), not for real-time decision-making.

### Causal graph as a living structure

The causal graph is not computed once and frozen. Relationships form, strengthen, weaken, and break as DeFi protocols evolve. Each edge carries metadata that governs its lifecycle:

**Confidence score.** Initialized when the edge is first discovered, updated with each confirming or disconfirming observation. Decays over time following the Ebbinghaus forgetting curve from the Grimoire memory system:

```
confidence(t) = confidence(t_0) * exp(-lambda * (t - t_last_confirmed))
```

where lambda is the decay rate and t_last_confirmed is the tick when the edge was last reinforced by new evidence. Edges whose confidence drops below a pruning threshold are removed from the active graph (but retained in Grimoire as semantic knowledge, in case the relationship re-emerges).

**Evidence type.** Each edge records how it was discovered: observational (PC algorithm), temporal (Granger causality), interventional (dream experiment), or hybrid (multiple sources). When evidence types disagree, interventional evidence takes priority: it directly tests the causal mechanism, while observational evidence can be confounded.

**Strength coefficient.** The estimated magnitude of the causal effect. For linear relationships, this is the regression coefficient from the structural equation. For non-linear relationships, it is the average treatment effect estimated from interventional experiments. Strength is directional and can be positive or negative.

**Lag.** The typical delay from cause to effect, measured in Gamma ticks. A swap causing a price change has lag ~0 (same block). A price change causing a liquidation might have lag 1-5 (depending on oracle update frequency and liquidation bot response time). A TVL outflow causing yield changes might have lag 10-50 (slow rebalancing). The lag informs the Golem about the time horizon of causal effects.

---

## Architecture [SPEC]

The causal discovery engine sits between the chain intelligence layer and the agent runtime. It consumes observations from the protocol state system, runs discovery algorithms at heartbeat-appropriate timescales, maintains the causal graph, and exports causal knowledge to the Grimoire and attention systems.

### Integration with the heartbeat

**Gamma tick (5-15 seconds).** The fastest timescale. The engine receives new observations (protocol state changes from the current block) and appends them to its observation buffer. It updates running statistics (means, variances, covariances) used by the CI tests. It does not run any discovery algorithms; the budget is too tight. But it does check recently confirmed causal edges for real-time predictions: if edge X -> Y has high confidence and X just changed, the engine emits a causal prediction for Y's expected change. This feeds the attention auction with causal relevance signals.

**Theta tick (30-120 seconds).** The incremental discovery timescale. The engine runs an incremental PC algorithm pass over the most recent observation window. "Incremental" means it does not start from scratch; it tests edges that are (a) newly possible (new variables added), (b) recently weakened (confidence dropped below a threshold), or (c) adjacent to variables that changed significantly in the last window. Theta also runs pairwise Granger tests for newly observed variable pairs. Results update the causal graph: new edges are added with observational evidence, existing edges have their confidence refreshed.

**Delta tick (~50 Theta ticks).** The full audit timescale. The engine runs a complete PC algorithm pass over all variables, using the CMI test (more expensive but handles non-linearity). It reconciles the observational graph with accumulated interventional evidence from dreams. Stale edges (confidence below threshold) are pruned. The entire graph is checked for DAG consistency (no cycles, or cycles are properly time-indexed). Updated causal knowledge is written to the Grimoire as semantic memory. The death testament is updated with the current graph state.

**NREM dream.** Retrospective replay. The engine receives sequences of past observations and replays them through the causal model. When the model's causal predictions match the observed outcomes, edge confidence is reinforced. When predictions fail, the edge is penalized. This is the consolidation mechanism: causal knowledge that correctly explains past experience is strengthened; knowledge that fails is weakened.

**REM dream.** Interventional experimentation. The engine's InterventionalDreamDesigner selects the highest-value experiments from the dream queue (prioritized by information gain: how much would confirming or disconfirming this edge change the graph?) and runs them on Mirage-rs. Each experiment forks state, executes an intervention, simulates downstream blocks, measures effects, and records results as interventional evidence in the causal graph with full Witness DAG provenance.

### Integration with the Witness DAG

Every causal claim is a vertex in the Witness DAG. The vertex links to:

- The observations that triggered the discovery (for observational edges)
- The CI test results (test statistic, p-value, conditioning set)
- The dream experiment record (for interventional edges): intervention design, fork state hash, outcome measurements
- The Granger test results (for temporal edges): window, lag, F-statistic

This gives cryptographic provenance for "why do we believe X causes Y?" If a downstream decision goes wrong, the Golem (or its operator) can trace the causal reasoning back to the specific observations and experiments that supported it.

### Integration with other subsystems

**Grimoire.** Causal edges are semantic knowledge. They are stored with the same Ebbinghaus decay as other Grimoire entries. When a Golem dies and its knowledge is inherited by a new generation (via Styx), the causal graph transfers as part of the semantic memory. The new Golem starts with its predecessor's causal model, subject to confidence decay during the inheritance gap.

**Mortality clocks.** The epistemic clock penalizes causal graph degradation. A graph with many stale edges (low confidence, old last-confirmed timestamps) represents dying knowledge. The epistemic mortality pressure incentivizes the Golem to invest dream cycles in maintaining its causal model.

**Attention auction.** Causal relevance is a bid factor. If the Golem's causal graph says X -> Y with high confidence, and X just changed, then Y becomes causally relevant. The attention system should bid high on monitoring Y. This implements proactive attention: instead of waiting to see Y change, the Golem anticipates the change based on causal knowledge and allocates cognitive resources accordingly.

**HDC encoding.** Causal edges are encoded as hypervectors for fast similarity search. A causal edge (X, Y) is encoded as bind(hv_X, hv_Y) in BSC space. A query "what does X cause?" is answered by binding the query with hv_X and searching the causal codebook for similar vectors. This runs in O(n) time with POPCNT hardware, where n is the number of stored causal edges, fast enough for Gamma-tick queries.

**Styx (clade knowledge sharing).** When one Golem discovers a causal edge, it can share the evidence with its clade through Styx. The receiving Golem does not blindly accept the edge; it adds it to its dream queue for independent confirmation. But the prior from the shared evidence accelerates discovery: instead of testing all 2,450 possible edges, the Golem focuses on the ones its siblings found promising.

---

## Rust implementation

### Core types

```rust
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// Unique identifier for a variable in the causal graph.
/// Cheap to copy, hash, and compare.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VariableId(pub u64);

impl fmt::Display for VariableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V{}", self.0)
    }
}

/// A variable in the causal graph. Each represents a single observable
/// DeFi quantity: one metric on one protocol.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CausalVariable {
    /// Protocol name, e.g. "uniswap_v3", "aave_v3", "lido"
    pub protocol: String,
    /// Metric name, e.g. "swap_volume_eth", "borrow_rate_usdc", "tvl_steth"
    pub metric: String,
    /// Unique identifier
    pub id: VariableId,
}

impl CausalVariable {
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.protocol, self.metric)
    }
}

/// How a causal edge was discovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CausalEvidenceType {
    /// Discovered by the PC algorithm from conditional independence tests
    Observational,
    /// Confirmed by a dream-based do(X) experiment on Mirage-rs
    Interventional,
    /// Discovered by Granger causality / transfer entropy test
    GrangerTemporal,
    /// Supported by multiple evidence types
    Hybrid,
}

impl CausalEvidenceType {
    /// Interventional evidence is strongest. Hybrid next. Then observational
    /// and Granger (which are roughly equivalent in trustworthiness).
    pub fn priority(&self) -> u8 {
        match self {
            CausalEvidenceType::Interventional => 3,
            CausalEvidenceType::Hybrid => 2,
            CausalEvidenceType::Observational => 1,
            CausalEvidenceType::GrangerTemporal => 1,
        }
    }
}

/// A directed causal edge: source causes target.
pub struct CausalEdge {
    pub source: VariableId,
    pub target: VariableId,
    /// How this edge was discovered
    pub evidence_type: CausalEvidenceType,
    /// Estimated causal coefficient (effect size).
    /// Positive = source increase causes target increase.
    /// Negative = source increase causes target decrease.
    pub strength: f64,
    /// Confidence in [0.0, 1.0]. Decays over time via Ebbinghaus curve.
    pub confidence: f64,
    /// Typical delay from cause to effect, in Gamma ticks
    pub lag_ticks: u32,
    /// Tick when this edge was last confirmed by new evidence
    pub last_confirmed: u64,
    /// Number of observations or experiments supporting this edge
    pub sample_count: u64,
    /// Link to the Witness DAG for cryptographic provenance
    pub witness_dag_id: Option<WitnessDagNodeId>,
}

/// Placeholder for the Witness DAG node reference.
/// In the full system, this is a content-addressed hash linking
/// to the observation/experiment that supports this edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WitnessDagNodeId(pub [u8; 32]);

/// Snapshot of observed variable values at a single point in time.
#[derive(Clone, Debug)]
pub struct Observation {
    pub tick: u64,
    pub values: HashMap<VariableId, f64>,
}

/// A timestamped observation stored in the buffer for causal analysis.
#[derive(Clone, Debug)]
pub struct ObservationRecord {
    pub tick: u64,
    pub block_number: u64,
    pub timestamp_secs: u64,
    pub values: HashMap<VariableId, f64>,
}
```

### The causal graph

```rust
/// The causal graph: a directed graph with typed, weighted edges.
/// Nodes are DeFi state variables. Edges are causal relationships.
pub struct CausalGraph {
    /// Variable metadata, keyed by ID
    variables: HashMap<VariableId, CausalVariable>,
    /// Edges keyed by (source, target) pair
    edges: HashMap<(VariableId, VariableId), CausalEdge>,
    /// Forward adjacency: variable -> list of children (variables it causes)
    adjacency: HashMap<VariableId, Vec<VariableId>>,
    /// Reverse adjacency: variable -> list of parents (variables that cause it)
    reverse_adjacency: HashMap<VariableId, Vec<VariableId>>,
    /// Counter for generating unique variable IDs
    next_var_id: u64,
}

impl CausalGraph {
    pub fn new() -> Self {
        CausalGraph {
            variables: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            next_var_id: 0,
        }
    }

    /// Register a new variable in the graph. Returns its unique ID.
    pub fn add_variable(&mut self, protocol: &str, metric: &str) -> VariableId {
        let id = VariableId(self.next_var_id);
        self.next_var_id += 1;
        self.variables.insert(id, CausalVariable {
            protocol: protocol.to_string(),
            metric: metric.to_string(),
            id,
        });
        self.adjacency.entry(id).or_default();
        self.reverse_adjacency.entry(id).or_default();
        id
    }

    /// Add or update a directed causal edge.
    /// If the edge already exists, the new evidence is merged:
    /// - If the new evidence type has higher priority, it replaces the old type
    /// - Confidence is max(old, new)
    /// - Sample count accumulates
    /// - Strength is updated via exponential moving average
    pub fn add_edge(&mut self, edge: CausalEdge) {
        let key = (edge.source, edge.target);
        if let Some(existing) = self.edges.get_mut(&key) {
            // Merge evidence
            if edge.evidence_type.priority() > existing.evidence_type.priority() {
                existing.evidence_type = edge.evidence_type;
            } else if edge.evidence_type != existing.evidence_type
                && existing.evidence_type != CausalEvidenceType::Hybrid
            {
                existing.evidence_type = CausalEvidenceType::Hybrid;
            }
            // Exponential moving average on strength
            let alpha = 0.3;
            existing.strength = (1.0 - alpha) * existing.strength + alpha * edge.strength;
            existing.confidence = existing.confidence.max(edge.confidence);
            existing.last_confirmed = existing.last_confirmed.max(edge.last_confirmed);
            existing.sample_count += edge.sample_count;
            if edge.witness_dag_id.is_some() {
                existing.witness_dag_id = edge.witness_dag_id;
            }
        } else {
            // New edge: update adjacency lists
            self.adjacency.entry(edge.source).or_default().push(edge.target);
            self.reverse_adjacency.entry(edge.target).or_default().push(edge.source);
            self.edges.insert(key, edge);
        }
    }

    /// Remove a causal edge.
    pub fn remove_edge(&mut self, source: VariableId, target: VariableId) {
        if self.edges.remove(&(source, target)).is_some() {
            if let Some(children) = self.adjacency.get_mut(&source) {
                children.retain(|&v| v != target);
            }
            if let Some(parents) = self.reverse_adjacency.get_mut(&target) {
                parents.retain(|&v| v != source);
            }
        }
    }

    /// Get the parents (direct causes) of a variable.
    pub fn parents(&self, var: VariableId) -> &[VariableId] {
        self.reverse_adjacency
            .get(&var)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get the children (direct effects) of a variable.
    pub fn children(&self, var: VariableId) -> &[VariableId] {
        self.adjacency
            .get(&var)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get edge metadata, if the edge exists.
    pub fn edge(&self, source: VariableId, target: VariableId) -> Option<&CausalEdge> {
        self.edges.get(&(source, target))
    }

    /// Edge density: ratio of actual edges to maximum possible edges.
    pub fn edge_density(&self) -> f64 {
        let n = self.variables.len();
        if n <= 1 {
            return 0.0;
        }
        self.edges.len() as f64 / (n * (n - 1)) as f64
    }

    /// Number of variables in the graph.
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// Number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check whether the graph is a DAG (no directed cycles).
    /// Uses Kahn's algorithm: iteratively remove nodes with in-degree 0.
    /// If all nodes are removed, the graph is a DAG.
    pub fn is_dag(&self) -> bool {
        self.topological_sort().is_some()
    }

    /// Topological sort via Kahn's algorithm. Returns None if the graph
    /// contains a cycle.
    pub fn topological_sort(&self) -> Option<Vec<VariableId>> {
        let mut in_degree: HashMap<VariableId, usize> = HashMap::new();
        for &var in self.variables.keys() {
            in_degree.insert(var, 0);
        }
        for edge in self.edges.values() {
            *in_degree.entry(edge.target).or_insert(0) += 1;
        }

        let mut queue: VecDeque<VariableId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&var, _)| var)
            .collect();

        let mut sorted = Vec::with_capacity(self.variables.len());

        while let Some(var) = queue.pop_front() {
            sorted.push(var);
            for &child in self.children(var) {
                if let Some(deg) = in_degree.get_mut(&child) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(child);
                    }
                }
            }
        }

        if sorted.len() == self.variables.len() {
            Some(sorted)
        } else {
            None // Cycle detected
        }
    }

    /// Estimate the total causal effect of an intervention on `source`
    /// on the `outcome` variable, by summing over all directed paths.
    /// For linear SCMs, this is the product of edge strengths along each path.
    ///
    /// Returns None if no directed path exists.
    pub fn causal_effect(
        &self,
        intervention: VariableId,
        outcome: VariableId,
    ) -> Option<f64> {
        if intervention == outcome {
            return Some(1.0);
        }

        // BFS to find all directed paths and sum their contributions.
        // For a DAG with reasonable size (< 100 nodes), this is fast.
        let mut total_effect = 0.0;
        let mut found_path = false;

        // Stack: (current_node, accumulated_effect)
        let mut stack: Vec<(VariableId, f64)> = vec![(intervention, 1.0)];

        // Track visited to avoid infinite loops in case of cycles
        // (though ideally the graph is a DAG)
        let mut visited_paths: HashSet<(VariableId, u64)> = HashSet::new();
        let mut path_counter: u64 = 0;

        while let Some((current, acc_effect)) = stack.pop() {
            for &child in self.children(current) {
                if let Some(edge) = self.edge(current, child) {
                    let path_effect = acc_effect * edge.strength;

                    if child == outcome {
                        total_effect += path_effect;
                        found_path = true;
                    } else {
                        path_counter += 1;
                        if visited_paths.insert((child, path_counter)) {
                            stack.push((child, path_effect));
                        }
                    }
                }
            }
        }

        if found_path {
            Some(total_effect)
        } else {
            None
        }
    }

    /// Test d-separation: are X and Y d-separated by the set Z?
    /// Uses the Bayes-Ball algorithm (Shachter, 1998).
    ///
    /// d-separation means: conditioning on Z blocks all paths between X and Y.
    /// If d-separated, X _||_ Y | Z in any distribution faithful to this graph.
    pub fn d_separation(
        &self,
        x: VariableId,
        y: VariableId,
        z: &[VariableId],
    ) -> bool {
        let z_set: HashSet<VariableId> = z.iter().copied().collect();

        // Bayes-Ball: start from X, try to reach Y.
        // Track (node, direction) where direction is "going up" (toward parents)
        // or "going down" (toward children).
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        enum Direction {
            Up,
            Down,
        }

        let mut visited: HashSet<(VariableId, Direction)> = HashSet::new();
        let mut queue: VecDeque<(VariableId, Direction)> = VecDeque::new();

        // Start by going both up and down from X
        queue.push_back((x, Direction::Up));
        queue.push_back((x, Direction::Down));

        while let Some((node, dir)) = queue.pop_front() {
            if node == y {
                return false; // Reachable => not d-separated
            }

            if !visited.insert((node, dir)) {
                continue;
            }

            let is_conditioned = z_set.contains(&node);

            match dir {
                Direction::Up => {
                    // Arrived going up (from a child). We can continue
                    // through this node if it is NOT conditioned on.
                    if !is_conditioned {
                        // Pass through to parents (continue up)
                        for &parent in self.parents(node) {
                            queue.push_back((parent, Direction::Up));
                        }
                        // Pass through to other children (go back down)
                        for &child in self.children(node) {
                            queue.push_back((child, Direction::Down));
                        }
                    }
                }
                Direction::Down => {
                    // Arrived going down (from a parent).
                    if !is_conditioned {
                        // Non-collider, not conditioned: pass through to children
                        for &child in self.children(node) {
                            queue.push_back((child, Direction::Down));
                        }
                    }
                    // Collider or conditioned: can go up to parents
                    // (conditioning on a collider opens the path)
                    if is_conditioned {
                        for &parent in self.parents(node) {
                            queue.push_back((parent, Direction::Up));
                        }
                    }
                }
            }
        }

        true // Y not reachable => d-separated
    }

    /// Compute the Markov blanket of a variable: its parents, its children,
    /// and the other parents of its children.
    /// The Markov blanket is the minimal set that makes a variable
    /// conditionally independent of all other variables.
    pub fn markov_blanket(&self, var: VariableId) -> Vec<VariableId> {
        let mut blanket: HashSet<VariableId> = HashSet::new();

        // Parents
        for &parent in self.parents(var) {
            blanket.insert(parent);
        }

        // Children and co-parents of children
        for &child in self.children(var) {
            blanket.insert(child);
            for &co_parent in self.parents(child) {
                if co_parent != var {
                    blanket.insert(co_parent);
                }
            }
        }

        blanket.into_iter().collect()
    }

    /// Decay all edge confidences by the Ebbinghaus forgetting curve.
    /// Called at Delta tick.
    ///
    /// decay_rate: lambda in exp(-lambda * dt). Typical value: 0.001 per tick.
    /// current_tick: the current Gamma tick number.
    pub fn decay_confidences(&mut self, decay_rate: f64, current_tick: u64) {
        for edge in self.edges.values_mut() {
            let dt = current_tick.saturating_sub(edge.last_confirmed) as f64;
            edge.confidence *= (-decay_rate * dt).exp();
        }
    }

    /// Remove all edges with confidence below the threshold.
    /// Returns the removed edges for logging/audit.
    pub fn prune_stale_edges(&mut self, min_confidence: f64) -> Vec<(VariableId, VariableId)> {
        let stale: Vec<(VariableId, VariableId)> = self
            .edges
            .iter()
            .filter(|(_, edge)| edge.confidence < min_confidence)
            .map(|(&key, _)| key)
            .collect();

        for (source, target) in &stale {
            self.remove_edge(*source, *target);
        }

        stale
    }

    /// Get all variable IDs.
    pub fn variable_ids(&self) -> Vec<VariableId> {
        self.variables.keys().copied().collect()
    }

    /// Get variable metadata by ID.
    pub fn variable(&self, id: VariableId) -> Option<&CausalVariable> {
        self.variables.get(&id)
    }

    /// Get all edges as an iterator.
    pub fn all_edges(&self) -> impl Iterator<Item = &CausalEdge> {
        self.edges.values()
    }
}
```

### Conditional independence testing

```rust
/// Trait for conditional independence tests.
/// Implementations must provide a test for X _||_ Y | Z
/// given observation data.
pub trait ConditionalIndependenceTest: Send + Sync {
    /// Test whether X and Y are conditionally independent given Z.
    ///
    /// Returns (independent, p_value, test_statistic).
    /// `independent` is true if p_value > significance_level.
    fn test(
        &self,
        x: VariableId,
        y: VariableId,
        z: &[VariableId],
        data: &[ObservationRecord],
        significance_level: f64,
    ) -> CiTestResult;
}

#[derive(Clone, Debug)]
pub struct CiTestResult {
    pub independent: bool,
    pub p_value: f64,
    pub test_statistic: f64,
    /// The conditioning set that was used
    pub conditioning_set: Vec<VariableId>,
}

/// Partial correlation test using Fisher's z-transform.
/// Assumes linear Gaussian relationships. Fast (O(|Z|^3) for matrix
/// inversion) but misses non-linear dependencies.
///
/// Used at Theta tick for fast incremental updates.
pub struct PartialCorrelationTest;

impl PartialCorrelationTest {
    /// Compute the partial correlation of X and Y given Z from data.
    /// Uses the formula: rho_{XY|Z} = -P^{-1}_{XY} / sqrt(P^{-1}_{XX} * P^{-1}_{YY})
    /// where P is the precision matrix (inverse covariance) of [X, Y, Z].
    fn partial_correlation(
        &self,
        x: VariableId,
        y: VariableId,
        z: &[VariableId],
        data: &[ObservationRecord],
    ) -> Option<f64> {
        let vars: Vec<VariableId> = std::iter::once(x)
            .chain(std::iter::once(y))
            .chain(z.iter().copied())
            .collect();
        let n = data.len();
        let p = vars.len();

        if n <= p + 3 {
            return None; // Not enough data for reliable estimation
        }

        // Build the data matrix: n x p
        let mut matrix: Vec<Vec<f64>> = Vec::with_capacity(n);
        for obs in data {
            let row: Vec<f64> = vars
                .iter()
                .map(|&v| obs.values.get(&v).copied().unwrap_or(0.0))
                .collect();
            matrix.push(row);
        }

        // Compute means
        let mut means = vec![0.0; p];
        for row in &matrix {
            for (j, val) in row.iter().enumerate() {
                means[j] += val;
            }
        }
        for mean in &mut means {
            *mean /= n as f64;
        }

        // Compute covariance matrix (p x p)
        let mut cov = vec![vec![0.0; p]; p];
        for row in &matrix {
            for i in 0..p {
                for j in i..p {
                    let d = (row[i] - means[i]) * (row[j] - means[j]);
                    cov[i][j] += d;
                    if i != j {
                        cov[j][i] += d;
                    }
                }
            }
        }
        let denom = (n - 1) as f64;
        for i in 0..p {
            for j in 0..p {
                cov[i][j] /= denom;
            }
        }

        // Invert the covariance matrix to get the precision matrix.
        // For small p (typically < 20), direct inversion is fine.
        let precision = invert_matrix(&cov)?;

        // Partial correlation from precision matrix:
        // rho_{XY|Z} = -P_{01} / sqrt(P_{00} * P_{11})
        // where 0 = X index, 1 = Y index in the variable list.
        let p_xy = precision[0][1];
        let p_xx = precision[0][0];
        let p_yy = precision[1][1];

        if p_xx <= 0.0 || p_yy <= 0.0 {
            return None;
        }

        let rho = -p_xy / (p_xx * p_yy).sqrt();
        Some(rho.clamp(-1.0, 1.0))
    }
}

impl ConditionalIndependenceTest for PartialCorrelationTest {
    fn test(
        &self,
        x: VariableId,
        y: VariableId,
        z: &[VariableId],
        data: &[ObservationRecord],
        significance_level: f64,
    ) -> CiTestResult {
        let n = data.len();
        let k = z.len();

        // Need at least k + 4 observations for the Fisher z-test
        if n < k + 4 {
            return CiTestResult {
                independent: false,
                p_value: 0.0,
                test_statistic: f64::NAN,
                conditioning_set: z.to_vec(),
            };
        }

        let rho = match self.partial_correlation(x, y, z, data) {
            Some(r) => r,
            None => {
                return CiTestResult {
                    independent: false,
                    p_value: 0.0,
                    test_statistic: f64::NAN,
                    conditioning_set: z.to_vec(),
                };
            }
        };

        // Fisher's z-transform
        // z_stat = 0.5 * sqrt(n - |Z| - 3) * ln((1 + rho) / (1 - rho))
        // Under H0 (independence), z_stat ~ N(0, 1)
        let dof = (n as f64) - (k as f64) - 3.0;
        if dof <= 0.0 {
            return CiTestResult {
                independent: false,
                p_value: 0.0,
                test_statistic: f64::NAN,
                conditioning_set: z.to_vec(),
            };
        }

        let z_stat = 0.5 * dof.sqrt() * ((1.0 + rho) / (1.0 - rho)).abs().ln();

        // Two-sided p-value from standard normal
        let p_value = 2.0 * standard_normal_survival(z_stat.abs());

        CiTestResult {
            independent: p_value > significance_level,
            p_value,
            test_statistic: z_stat,
            conditioning_set: z.to_vec(),
        }
    }
}

/// Conditional mutual information test using k-NN (KSG) estimation.
/// Handles non-linear dependencies. More expensive than partial correlation.
///
/// Used at Delta tick for thorough graph audits.
pub struct MutualInformationTest {
    /// Number of nearest neighbors for the KSG estimator
    pub k_neighbors: usize,
}

impl MutualInformationTest {
    pub fn new(k_neighbors: usize) -> Self {
        MutualInformationTest { k_neighbors }
    }

    /// Estimate conditional mutual information I(X; Y | Z) using the
    /// KSG (Kraskov-Stogbauer-Grassberger) estimator.
    ///
    /// The KSG estimator works by:
    /// 1. Find the k-th nearest neighbor distance in the joint space (X, Y, Z)
    /// 2. Count neighbors within that distance in marginal spaces (X,Z) and (Y,Z) and Z
    /// 3. CMI = psi(k) - <psi(n_xz + 1) + psi(n_yz + 1) - psi(n_z + 1)>
    ///
    /// where psi is the digamma function and <> denotes averaging over data points.
    fn estimate_cmi(
        &self,
        x: VariableId,
        y: VariableId,
        z: &[VariableId],
        data: &[ObservationRecord],
    ) -> f64 {
        let n = data.len();
        if n < self.k_neighbors + 1 {
            return 0.0;
        }

        // Extract data columns
        let x_vals: Vec<f64> = data
            .iter()
            .map(|obs| obs.values.get(&x).copied().unwrap_or(0.0))
            .collect();
        let y_vals: Vec<f64> = data
            .iter()
            .map(|obs| obs.values.get(&y).copied().unwrap_or(0.0))
            .collect();
        let z_vals: Vec<Vec<f64>> = data
            .iter()
            .map(|obs| {
                z.iter()
                    .map(|&v| obs.values.get(&v).copied().unwrap_or(0.0))
                    .collect()
            })
            .collect();

        // For each point i, find the k-th nearest neighbor distance
        // in the joint (X, Y, Z) space using Chebyshev (max) distance.
        let k = self.k_neighbors;
        let mut cmi_sum = 0.0;

        for i in 0..n {
            // Compute distances from point i to all other points in joint space
            let mut joint_dists: Vec<(usize, f64)> = (0..n)
                .filter(|&j| j != i)
                .map(|j| {
                    let dx = (x_vals[i] - x_vals[j]).abs();
                    let dy = (y_vals[i] - y_vals[j]).abs();
                    let dz: f64 = z_vals[i]
                        .iter()
                        .zip(z_vals[j].iter())
                        .map(|(a, b)| (a - b).abs())
                        .fold(0.0_f64, f64::max);
                    let d = dx.max(dy).max(dz);
                    (j, d)
                })
                .collect();

            joint_dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            if joint_dists.len() < k {
                continue;
            }

            let epsilon = joint_dists[k - 1].1;
            if epsilon == 0.0 {
                continue; // Degenerate case: k-th neighbor at distance 0
            }

            // Count neighbors within epsilon in marginal spaces
            let n_xz = (0..n)
                .filter(|&j| {
                    if j == i {
                        return false;
                    }
                    let dx = (x_vals[i] - x_vals[j]).abs();
                    let dz: f64 = z_vals[i]
                        .iter()
                        .zip(z_vals[j].iter())
                        .map(|(a, b)| (a - b).abs())
                        .fold(0.0_f64, f64::max);
                    dx.max(dz) <= epsilon
                })
                .count();

            let n_yz = (0..n)
                .filter(|&j| {
                    if j == i {
                        return false;
                    }
                    let dy = (y_vals[i] - y_vals[j]).abs();
                    let dz: f64 = z_vals[i]
                        .iter()
                        .zip(z_vals[j].iter())
                        .map(|(a, b)| (a - b).abs())
                        .fold(0.0_f64, f64::max);
                    dy.max(dz) <= epsilon
                })
                .count();

            let n_z = (0..n)
                .filter(|&j| {
                    if j == i {
                        return false;
                    }
                    let dz: f64 = z_vals[i]
                        .iter()
                        .zip(z_vals[j].iter())
                        .map(|(a, b)| (a - b).abs())
                        .fold(0.0_f64, f64::max);
                    dz <= epsilon
                })
                .count();

            // KSG formula: psi(k) - [psi(n_xz + 1) + psi(n_yz + 1) - psi(n_z + 1)]
            // For unconditional MI (Z is empty), the formula simplifies.
            if z.is_empty() {
                // Unconditional MI: psi(k) + psi(n) - psi(n_x + 1) - psi(n_y + 1)
                // where n_x, n_y are counts in marginal X, Y within epsilon
                let n_x = (0..n)
                    .filter(|&j| j != i && (x_vals[i] - x_vals[j]).abs() <= epsilon)
                    .count();
                let n_y = (0..n)
                    .filter(|&j| j != i && (y_vals[i] - y_vals[j]).abs() <= epsilon)
                    .count();
                cmi_sum += digamma(k as f64) + digamma(n as f64)
                    - digamma(n_x as f64 + 1.0)
                    - digamma(n_y as f64 + 1.0);
            } else {
                cmi_sum += digamma(k as f64)
                    - digamma(n_xz as f64 + 1.0)
                    - digamma(n_yz as f64 + 1.0)
                    + digamma(n_z as f64 + 1.0);
            }
        }

        let cmi = cmi_sum / n as f64;
        cmi.max(0.0) // CMI is non-negative; clamp numerical errors
    }
}

impl ConditionalIndependenceTest for MutualInformationTest {
    fn test(
        &self,
        x: VariableId,
        y: VariableId,
        z: &[VariableId],
        data: &[ObservationRecord],
        significance_level: f64,
    ) -> CiTestResult {
        let n = data.len();
        let cmi = self.estimate_cmi(x, y, z, data);

        // Permutation test for significance:
        // Shuffle X values (breaking X-Y dependence) and re-estimate CMI.
        // The p-value is the fraction of permuted CMI values >= observed CMI.
        let n_permutations = 100;
        let mut rng_state: u64 = n as u64 ^ 0xDEADBEEF;
        let mut count_above = 0u32;

        let mut permuted_data: Vec<ObservationRecord> = data.to_vec();
        for _ in 0..n_permutations {
            // Fisher-Yates shuffle on the X values only
            let mut x_vals: Vec<f64> = permuted_data
                .iter()
                .map(|obs| obs.values.get(&x).copied().unwrap_or(0.0))
                .collect();

            for i in (1..x_vals.len()).rev() {
                // Simple LCG for deterministic permutation testing
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let j = (rng_state >> 33) as usize % (i + 1);
                x_vals.swap(i, j);
            }

            // Write shuffled X values back
            for (obs, &val) in permuted_data.iter_mut().zip(x_vals.iter()) {
                obs.values.insert(x, val);
            }

            let perm_cmi = self.estimate_cmi(x, y, z, &permuted_data);
            if perm_cmi >= cmi {
                count_above += 1;
            }
        }

        let p_value = count_above as f64 / n_permutations as f64;

        CiTestResult {
            independent: p_value > significance_level,
            p_value,
            test_statistic: cmi,
            conditioning_set: z.to_vec(),
        }
    }
}

// --- Utility functions ---

/// Approximate the survival function of the standard normal distribution.
/// P(Z > z) for Z ~ N(0, 1). Uses the Abramowitz and Stegun approximation.
fn standard_normal_survival(z: f64) -> f64 {
    if z < 0.0 {
        return 1.0 - standard_normal_survival(-z);
    }
    let t = 1.0 / (1.0 + 0.2316419 * z);
    let d = 0.3989422804014327; // 1 / sqrt(2 * pi)
    let p = d * (-0.5 * z * z).exp();
    let poly = t * (0.319381530
        + t * (-0.356563782
            + t * (1.781477937
                + t * (-1.821255978
                    + t * 1.330274429))));
    p * poly
}

/// Digamma function (psi). Approximation using the asymptotic series
/// with recurrence for small arguments.
fn digamma(mut x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NEG_INFINITY;
    }
    let mut result = 0.0;
    // Use recurrence psi(x) = psi(x+1) - 1/x to shift x into the
    // asymptotic region (x >= 6)
    while x < 6.0 {
        result -= 1.0 / x;
        x += 1.0;
    }
    // Asymptotic expansion
    result += x.ln() - 0.5 / x;
    let x2 = x * x;
    result -= 1.0 / (12.0 * x2);
    result += 1.0 / (120.0 * x2 * x2);
    result -= 1.0 / (252.0 * x2 * x2 * x2);
    result
}

/// Invert a square matrix using Gauss-Jordan elimination.
/// Returns None if the matrix is singular.
fn invert_matrix(mat: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = mat.len();
    // Augmented matrix [mat | I]
    let mut aug: Vec<Vec<f64>> = Vec::with_capacity(n);
    for i in 0..n {
        let mut row = mat[i].clone();
        row.resize(2 * n, 0.0);
        row[n + i] = 1.0;
        aug.push(row);
    }

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
        if max_val < 1e-12 {
            return None; // Singular
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        for j in 0..(2 * n) {
            aug[col][j] /= pivot;
        }

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            for j in 0..(2 * n) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Extract the inverse (right half of augmented matrix)
    let inv: Vec<Vec<f64>> = aug
        .iter()
        .map(|row| row[n..].to_vec())
        .collect();
    Some(inv)
}
```

### PC algorithm

```rust
/// Configuration for the PC algorithm.
pub struct PcConfig {
    /// Significance level for conditional independence tests.
    /// Lower = fewer edges (more conservative). Default: 0.05.
    pub significance_level: f64,
    /// Maximum conditioning set size. Limits computation.
    /// Default: 4 (sufficient for most DeFi causal structures).
    pub max_conditioning_size: usize,
    /// Maximum number of CI tests per Theta tick (budget control).
    pub max_tests_per_tick: usize,
}

impl Default for PcConfig {
    fn default() -> Self {
        PcConfig {
            significance_level: 0.05,
            max_conditioning_size: 4,
            max_tests_per_tick: 500,
        }
    }
}

/// Undirected edge in the skeleton, with its separating set.
#[derive(Clone, Debug)]
struct SkeletonEdge {
    a: VariableId,
    b: VariableId,
    /// The conditioning set that rendered this pair independent (if removed)
    separating_set: Option<Vec<VariableId>>,
    /// Whether the edge has been removed
    removed: bool,
}

/// The PC algorithm for causal skeleton discovery.
///
/// Operates incrementally: maintains state across Theta ticks,
/// resuming from where it left off rather than starting over.
pub struct PcAlgorithm {
    config: PcConfig,
    /// Current adjacency structure (undirected skeleton)
    skeleton: HashMap<VariableId, HashSet<VariableId>>,
    /// Separating sets: sep(X, Y) = the conditioning set that made X _||_ Y
    separating_sets: HashMap<(VariableId, VariableId), Vec<VariableId>>,
    /// Current conditioning set size being tested
    current_cond_size: usize,
    /// Queue of edge pairs to test at the current conditioning size
    test_queue: VecDeque<(VariableId, VariableId)>,
    /// Whether the skeleton phase is complete
    skeleton_complete: bool,
}

impl PcAlgorithm {
    pub fn new(config: PcConfig) -> Self {
        PcAlgorithm {
            config,
            skeleton: HashMap::new(),
            separating_sets: HashMap::new(),
            current_cond_size: 0,
            test_queue: VecDeque::new(),
            skeleton_complete: false,
        }
    }

    /// Initialize with a complete undirected graph over the given variables.
    pub fn initialize(&mut self, variables: &[VariableId]) {
        self.skeleton.clear();
        self.separating_sets.clear();
        self.current_cond_size = 0;
        self.skeleton_complete = false;

        // Start with complete graph
        for &v in variables {
            let mut neighbors: HashSet<VariableId> = variables.iter().copied().collect();
            neighbors.remove(&v);
            self.skeleton.insert(v, neighbors);
        }

        // Initialize test queue with all pairs
        self.rebuild_test_queue();
    }

    fn rebuild_test_queue(&mut self) {
        self.test_queue.clear();
        let vars: Vec<VariableId> = self.skeleton.keys().copied().collect();
        for i in 0..vars.len() {
            for j in (i + 1)..vars.len() {
                let a = vars[i];
                let b = vars[j];
                if self.skeleton.get(&a).map_or(false, |s| s.contains(&b)) {
                    self.test_queue.push_back((a, b));
                }
            }
        }
    }

    /// Run one batch of CI tests (up to max_tests_per_tick).
    /// Returns true if the skeleton phase is complete.
    pub fn run_batch(
        &mut self,
        ci_test: &dyn ConditionalIndependenceTest,
        data: &[ObservationRecord],
    ) -> bool {
        if self.skeleton_complete {
            return true;
        }

        let mut tests_run = 0;

        while tests_run < self.config.max_tests_per_tick {
            if let Some((a, b)) = self.test_queue.pop_front() {
                // Check if edge still exists (might have been removed by a previous test)
                if !self.skeleton.get(&a).map_or(false, |s| s.contains(&b)) {
                    continue;
                }

                // Get conditioning set candidates: adj(a) \ {b}
                let adj_a: Vec<VariableId> = self
                    .skeleton
                    .get(&a)
                    .map(|s| s.iter().copied().filter(|&v| v != b).collect())
                    .unwrap_or_default();

                if adj_a.len() < self.current_cond_size {
                    continue; // Not enough neighbors for this conditioning size
                }

                // Test all subsets of adj(a) of size current_cond_size
                let subsets = combinations(&adj_a, self.current_cond_size);
                let mut found_independence = false;

                for subset in subsets {
                    let result = ci_test.test(
                        a,
                        b,
                        &subset,
                        data,
                        self.config.significance_level,
                    );
                    tests_run += 1;

                    if result.independent {
                        // Remove edge and record separating set
                        self.skeleton.get_mut(&a).map(|s| s.remove(&b));
                        self.skeleton.get_mut(&b).map(|s| s.remove(&a));
                        let key = if a.0 < b.0 { (a, b) } else { (b, a) };
                        self.separating_sets.insert(key, subset);
                        found_independence = true;
                        break;
                    }

                    if tests_run >= self.config.max_tests_per_tick {
                        // Budget exhausted. Re-queue this pair for next tick.
                        if !found_independence {
                            self.test_queue.push_back((a, b));
                        }
                        return false;
                    }
                }
            } else {
                // Queue exhausted at this conditioning size. Move to next size.
                self.current_cond_size += 1;
                if self.current_cond_size > self.config.max_conditioning_size {
                    self.skeleton_complete = true;
                    return true;
                }
                self.rebuild_test_queue();

                if self.test_queue.is_empty() {
                    self.skeleton_complete = true;
                    return true;
                }
            }
        }

        false
    }

    /// Phase 2: Orient edges using v-structure detection and orientation rules.
    /// Call after skeleton discovery is complete.
    ///
    /// Returns a directed graph as a set of directed edges.
    pub fn orient_edges(&self) -> Vec<(VariableId, VariableId)> {
        // Start with undirected edges
        let mut undirected: HashSet<(VariableId, VariableId)> = HashSet::new();
        let mut directed: HashSet<(VariableId, VariableId)> = HashSet::new();

        for (&a, neighbors) in &self.skeleton {
            for &b in neighbors {
                if a.0 < b.0 {
                    undirected.insert((a, b));
                }
            }
        }

        // Rule 1: v-structure detection
        // For each triple X -- Z -- Y where X and Y are not adjacent:
        // If Z is NOT in sep(X, Y), orient as X -> Z <- Y (collider)
        let all_vars: Vec<VariableId> = self.skeleton.keys().copied().collect();
        for &z in &all_vars {
            let z_neighbors: Vec<VariableId> = self
                .skeleton
                .get(&z)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();

            for i in 0..z_neighbors.len() {
                for j in (i + 1)..z_neighbors.len() {
                    let x = z_neighbors[i];
                    let y = z_neighbors[j];

                    // Check X and Y are not adjacent
                    if self.skeleton.get(&x).map_or(false, |s| s.contains(&y)) {
                        continue;
                    }

                    // Check if Z is in sep(X, Y)
                    let key = if x.0 < y.0 { (x, y) } else { (y, x) };
                    let z_in_sep = self
                        .separating_sets
                        .get(&key)
                        .map_or(false, |sep| sep.contains(&z));

                    if !z_in_sep {
                        // Orient X -> Z <- Y
                        directed.insert((x, z));
                        directed.insert((y, z));
                        let edge_xz = if x.0 < z.0 { (x, z) } else { (z, x) };
                        let edge_yz = if y.0 < z.0 { (y, z) } else { (z, y) };
                        undirected.remove(&edge_xz);
                        undirected.remove(&edge_yz);
                    }
                }
            }
        }

        // Rules 2 and 3: propagate orientations to avoid new v-structures and cycles.
        // Iterate until no more orientations can be made.
        let mut changed = true;
        while changed {
            changed = false;

            for &(a, b) in &undirected.clone() {
                // Check both possible orientations of the undirected edge
                for (x, y) in [(a, b), (b, a)] {
                    // Rule 2: If there exists Z such that Z -> X and Z -- Y are
                    // not adjacent, orient X -> Y
                    let x_parents: Vec<VariableId> = directed
                        .iter()
                        .filter(|(_, t)| *t == x)
                        .map(|(s, _)| *s)
                        .collect();

                    for &z in &x_parents {
                        if !self.skeleton.get(&z).map_or(false, |s| s.contains(&y)) {
                            directed.insert((x, y));
                            let edge = if a.0 < b.0 { (a, b) } else { (b, a) };
                            undirected.remove(&edge);
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }

        // Convert remaining undirected edges to both directions (ambiguous)
        // The caller should queue these for interventional experiments.
        let mut result: Vec<(VariableId, VariableId)> =
            directed.into_iter().collect();

        // Mark undirected edges -- we include them in both directions
        // with a convention that the caller can identify them
        for (a, b) in undirected {
            result.push((a, b));
            result.push((b, a));
        }

        result
    }

    /// Get the separating set for a pair of variables, if known.
    pub fn separating_set(
        &self,
        a: VariableId,
        b: VariableId,
    ) -> Option<&Vec<VariableId>> {
        let key = if a.0 < b.0 { (a, b) } else { (b, a) };
        self.separating_sets.get(&key)
    }
}

/// Generate all combinations of `k` elements from `items`.
fn combinations(items: &[VariableId], k: usize) -> Vec<Vec<VariableId>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.len() < k {
        return vec![];
    }

    let mut result = Vec::new();
    let mut stack: Vec<(usize, Vec<VariableId>)> = vec![(0, Vec::new())];

    while let Some((start, current)) = stack.pop() {
        if current.len() == k {
            result.push(current);
            continue;
        }
        let remaining = k - current.len();
        for i in start..=(items.len() - remaining) {
            let mut next = current.clone();
            next.push(items[i]);
            stack.push((i + 1, next));
        }
    }

    result
}
```

### Granger causality

```rust
/// Configuration for Granger causality tests.
pub struct GrangerConfig {
    /// Maximum lag order (in events, not clock time)
    pub max_lag: usize,
    /// Significance level for the F-test
    pub significance_level: f64,
    /// Sliding window size (number of events)
    pub window_size: usize,
    /// L1 regularization strength for Lasso VAR (0.0 = no regularization)
    pub l1_penalty: f64,
}

impl Default for GrangerConfig {
    fn default() -> Self {
        GrangerConfig {
            max_lag: 10,
            significance_level: 0.05,
            window_size: 500,
            l1_penalty: 0.01,
        }
    }
}

/// Result of a Granger causality test.
#[derive(Clone, Debug)]
pub struct GrangerResult {
    pub source: VariableId,
    pub target: VariableId,
    /// Whether source Granger-causes target at the given significance level
    pub granger_causes: bool,
    /// F-statistic for the test
    pub f_statistic: f64,
    /// p-value from the F-distribution
    pub p_value: f64,
    /// Optimal lag order (the lag that gives the strongest signal)
    pub optimal_lag: usize,
    /// Transfer entropy estimate (non-linear alternative to F-test)
    pub transfer_entropy: f64,
}

/// Granger causality test with extensions for DeFi event streams.
///
/// Supports irregular timestamps (event-time indexing), multivariate
/// conditioning, and sliding windows for time-varying structure.
pub struct GrangerCausalityTest {
    config: GrangerConfig,
    /// Per-variable event queues: ordered (event_index, value) pairs
    event_queues: HashMap<VariableId, VecDeque<(u64, f64)>>,
    /// Global event counter
    event_counter: u64,
}

impl GrangerCausalityTest {
    pub fn new(config: GrangerConfig) -> Self {
        GrangerCausalityTest {
            config,
            event_queues: HashMap::new(),
            event_counter: 0,
        }
    }

    /// Ingest a new observation. Each variable's value is appended to
    /// its event queue with the current event index.
    pub fn observe(&mut self, observation: &Observation) {
        self.event_counter += 1;
        for (&var, &value) in &observation.values {
            let queue = self.event_queues.entry(var).or_default();
            queue.push_back((self.event_counter, value));

            // Trim to window size
            while queue.len() > self.config.window_size {
                queue.pop_front();
            }
        }
    }

    /// Test whether `source` Granger-causes `target`, conditioning on
    /// all other observed variables.
    ///
    /// Uses a bivariate VAR(k) model comparison:
    /// Restricted:   Y_t = a_0 + sum_{i=1}^{k} a_i * Y_{t-i} + e_t
    /// Unrestricted: Y_t = a_0 + sum_{i=1}^{k} a_i * Y_{t-i}
    ///                          + sum_{i=1}^{k} b_i * X_{t-i} + e_t
    ///
    /// F = ((RSS_r - RSS_u) / k) / (RSS_u / (n - 2k - 1))
    pub fn test(
        &self,
        source: VariableId,
        target: VariableId,
    ) -> GrangerResult {
        let x_queue = match self.event_queues.get(&source) {
            Some(q) if q.len() >= self.config.max_lag + 10 => q,
            _ => {
                return GrangerResult {
                    source,
                    target,
                    granger_causes: false,
                    f_statistic: 0.0,
                    p_value: 1.0,
                    optimal_lag: 0,
                    transfer_entropy: 0.0,
                };
            }
        };

        let y_queue = match self.event_queues.get(&target) {
            Some(q) if q.len() >= self.config.max_lag + 10 => q,
            _ => {
                return GrangerResult {
                    source,
                    target,
                    granger_causes: false,
                    f_statistic: 0.0,
                    p_value: 1.0,
                    optimal_lag: 0,
                    transfer_entropy: 0.0,
                };
            }
        };

        // Align event-time series: find common event indices
        let x_vals: Vec<f64> = x_queue.iter().map(|(_, v)| *v).collect();
        let y_vals: Vec<f64> = y_queue.iter().map(|(_, v)| *v).collect();

        // Use the shorter series as the effective length
        let n = x_vals.len().min(y_vals.len());
        let x = &x_vals[x_vals.len() - n..];
        let y = &y_vals[y_vals.len() - n..];

        let mut best_lag = 1;
        let mut best_f = 0.0_f64;
        let mut best_p = 1.0_f64;

        // Test each lag order and pick the one with the strongest signal
        for lag in 1..=self.config.max_lag {
            if n <= 2 * lag + 1 {
                break;
            }

            let effective_n = n - lag;

            // Build restricted model (Y ~ Y_lagged)
            // Build unrestricted model (Y ~ Y_lagged + X_lagged)
            let mut rss_restricted = 0.0;
            let mut rss_unrestricted = 0.0;

            // For simplicity, use OLS via normal equations.
            // Restricted: Y_t = c + sum a_i * Y_{t-i}
            // Unrestricted: Y_t = c + sum a_i * Y_{t-i} + sum b_i * X_{t-i}

            // Build design matrices
            let mut y_target = Vec::with_capacity(effective_n);
            let mut x_restricted = Vec::with_capacity(effective_n);
            let mut x_unrestricted = Vec::with_capacity(effective_n);

            for t in lag..n {
                y_target.push(y[t]);

                // Restricted features: [1, y_{t-1}, ..., y_{t-lag}]
                let mut row_r = vec![1.0];
                for i in 1..=lag {
                    row_r.push(y[t - i]);
                }
                x_restricted.push(row_r);

                // Unrestricted features: [1, y_{t-1}, ..., y_{t-lag}, x_{t-1}, ..., x_{t-lag}]
                let mut row_u = vec![1.0];
                for i in 1..=lag {
                    row_u.push(y[t - i]);
                }
                for i in 1..=lag {
                    row_u.push(x[t - i]);
                }
                x_unrestricted.push(row_u);
            }

            // Solve OLS: beta = (X'X)^{-1} X'y
            rss_restricted = ols_rss(&x_restricted, &y_target);
            rss_unrestricted = ols_rss(&x_unrestricted, &y_target);

            // F-statistic
            let df1 = lag as f64; // number of added parameters
            let df2 = effective_n as f64 - 2.0 * lag as f64 - 1.0;
            if df2 <= 0.0 || rss_unrestricted <= 0.0 {
                continue;
            }

            let f_stat =
                ((rss_restricted - rss_unrestricted) / df1) / (rss_unrestricted / df2);

            // Approximate p-value from F-distribution using the
            // incomplete beta function relationship:
            // p = 1 - I_x(df1/2, df2/2) where x = df1*F / (df1*F + df2)
            let p_value = f_distribution_survival(f_stat, df1, df2);

            if f_stat > best_f {
                best_f = f_stat;
                best_p = p_value;
                best_lag = lag;
            }
        }

        // Transfer entropy estimate (non-linear check)
        // TE(X -> Y) = I(Y_t; X_{<t} | Y_{<t})
        // We approximate using the bivariate time series with lag 1
        let te = estimate_transfer_entropy(x, y, best_lag);

        GrangerResult {
            source,
            target,
            granger_causes: best_p < self.config.significance_level,
            f_statistic: best_f,
            p_value: best_p,
            optimal_lag: best_lag,
            transfer_entropy: te,
        }
    }

    /// Get all variable IDs with sufficient data for testing.
    pub fn testable_variables(&self) -> Vec<VariableId> {
        self.event_queues
            .iter()
            .filter(|(_, q)| q.len() >= self.config.max_lag + 10)
            .map(|(&v, _)| v)
            .collect()
    }
}

/// Compute residual sum of squares for OLS regression.
/// X: n x p design matrix (rows are observations)
/// y: n x 1 target vector
fn ols_rss(x_matrix: &[Vec<f64>], y: &[f64]) -> f64 {
    let n = x_matrix.len();
    let p = x_matrix[0].len();

    // X'X (p x p)
    let mut xtx = vec![vec![0.0; p]; p];
    for row in x_matrix {
        for i in 0..p {
            for j in i..p {
                let v = row[i] * row[j];
                xtx[i][j] += v;
                if i != j {
                    xtx[j][i] += v;
                }
            }
        }
    }

    // X'y (p x 1)
    let mut xty = vec![0.0; p];
    for (row, &yi) in x_matrix.iter().zip(y.iter()) {
        for j in 0..p {
            xty[j] += row[j] * yi;
        }
    }

    // Solve beta = (X'X)^{-1} X'y
    let beta = match solve_linear_system(&xtx, &xty) {
        Some(b) => b,
        None => return f64::MAX, // Singular: return large RSS
    };

    // RSS = sum (y_i - x_i' * beta)^2
    let mut rss = 0.0;
    for (row, &yi) in x_matrix.iter().zip(y.iter()) {
        let predicted: f64 = row.iter().zip(beta.iter()).map(|(x, b)| x * b).sum();
        let residual = yi - predicted;
        rss += residual * residual;
    }
    rss
}

/// Solve a linear system Ax = b using Gaussian elimination with partial pivoting.
fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = a.len();
    let mut aug: Vec<Vec<f64>> = a
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut r = row.clone();
            r.push(b[i]);
            r
        })
        .collect();

    for col in 0..n {
        // Partial pivoting
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None;
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        for j in col..=n {
            aug[col][j] /= pivot;
        }

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            for j in col..=n {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    Some(aug.iter().map(|row| row[n]).collect())
}

/// Approximate the survival function of the F-distribution.
/// Uses the relationship between F and the Beta distribution.
fn f_distribution_survival(f: f64, df1: f64, df2: f64) -> f64 {
    if f <= 0.0 {
        return 1.0;
    }
    let x = df1 * f / (df1 * f + df2);
    1.0 - regularized_incomplete_beta(x, df1 / 2.0, df2 / 2.0)
}

/// Regularized incomplete beta function I_x(a, b).
/// Uses a continued fraction approximation (Lentz's method).
fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use the symmetry relation when x > (a+1)/(a+b+2) for better convergence
    if x > (a + 1.0) / (a + b + 2.0) {
        return 1.0 - regularized_incomplete_beta(1.0 - x, b, a);
    }

    let ln_prefix = a * x.ln() + b * (1.0 - x).ln()
        - a.ln()
        - ln_beta(a, b);
    let prefix = ln_prefix.exp();

    // Continued fraction (Lentz's modified method)
    let max_iter = 200;
    let epsilon = 1e-10;
    let tiny = 1e-30;

    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=max_iter {
        let m_f64 = m as f64;

        // Even step
        let numerator = m_f64 * (b - m_f64) * x / ((a + 2.0 * m_f64 - 1.0) * (a + 2.0 * m_f64));
        d = 1.0 + numerator * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + numerator / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        h *= d * c;

        // Odd step
        let numerator = -((a + m_f64) * (a + b + m_f64) * x)
            / ((a + 2.0 * m_f64) * (a + 2.0 * m_f64 + 1.0));
        d = 1.0 + numerator * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + numerator / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;

        if (delta - 1.0).abs() < epsilon {
            break;
        }
    }

    prefix * h / a
}

/// Natural log of the Beta function: ln(B(a, b)) = ln(Gamma(a)) + ln(Gamma(b)) - ln(Gamma(a+b))
fn ln_beta(a: f64, b: f64) -> f64 {
    ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)
}

/// Stirling's approximation for ln(Gamma(x)).
fn ln_gamma(x: f64) -> f64 {
    // Lanczos approximation
    let g = 7.0;
    let coefficients = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    if x < 0.5 {
        // Reflection formula
        let pi = std::f64::consts::PI;
        return pi.ln() - (pi * x).sin().ln() - ln_gamma(1.0 - x);
    }

    let x = x - 1.0;
    let mut ag = coefficients[0];
    for (i, &c) in coefficients.iter().enumerate().skip(1) {
        ag += c / (x + i as f64);
    }

    let tmp = x + g + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * tmp.ln() - tmp + ag.ln()
}

/// Estimate transfer entropy TE(X -> Y) at a given lag.
/// Uses a binning estimator for speed (adequate for screening).
fn estimate_transfer_entropy(x: &[f64], y: &[f64], lag: usize) -> f64 {
    let n = x.len().min(y.len());
    if n <= lag + 1 {
        return 0.0;
    }

    // Bin continuous values into discrete states
    let n_bins = 8;
    let x_bins = bin_series(x, n_bins);
    let y_bins = bin_series(y, n_bins);

    // TE(X -> Y) = H(Y_t | Y_{t-lag}) - H(Y_t | Y_{t-lag}, X_{t-lag})
    // Estimate conditional entropies from joint frequency tables
    let effective_n = n - lag;

    // Count (y_t, y_{t-lag}) pairs
    let mut count_yy: HashMap<(u8, u8), f64> = HashMap::new();
    let mut count_y_lag: HashMap<u8, f64> = HashMap::new();

    // Count (y_t, y_{t-lag}, x_{t-lag}) triples
    let mut count_yyx: HashMap<(u8, u8, u8), f64> = HashMap::new();
    let mut count_yx_lag: HashMap<(u8, u8), f64> = HashMap::new();

    for t in lag..n {
        let yt = y_bins[t];
        let yt_lag = y_bins[t - lag];
        let xt_lag = x_bins[t - lag];

        *count_yy.entry((yt, yt_lag)).or_default() += 1.0;
        *count_y_lag.entry(yt_lag).or_default() += 1.0;
        *count_yyx.entry((yt, yt_lag, xt_lag)).or_default() += 1.0;
        *count_yx_lag.entry((yt_lag, xt_lag)).or_default() += 1.0;
    }

    let n_f64 = effective_n as f64;

    // H(Y_t | Y_{t-lag}) = -sum p(y_t, y_lag) * log(p(y_t | y_lag))
    let mut h_y_given_ylag = 0.0;
    for (&(yt, yl), &c) in &count_yy {
        let p_joint = c / n_f64;
        let p_cond = c / count_y_lag[&yl];
        h_y_given_ylag -= p_joint * p_cond.ln();
    }

    // H(Y_t | Y_{t-lag}, X_{t-lag})
    let mut h_y_given_yxlag = 0.0;
    for (&(yt, yl, xl), &c) in &count_yyx {
        let p_joint = c / n_f64;
        let p_cond = c / count_yx_lag[&(yl, xl)];
        h_y_given_yxlag -= p_joint * p_cond.ln();
    }

    (h_y_given_ylag - h_y_given_yxlag).max(0.0)
}

/// Bin a continuous series into discrete bins using equal-frequency binning.
fn bin_series(values: &[f64], n_bins: usize) -> Vec<u8> {
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Compute bin edges at quantiles
    let mut edges = Vec::with_capacity(n_bins - 1);
    for i in 1..n_bins {
        let idx = (i * sorted.len()) / n_bins;
        edges.push(sorted[idx.min(sorted.len() - 1)]);
    }

    values
        .iter()
        .map(|&v| {
            let bin = edges.iter().position(|&e| v < e).unwrap_or(edges.len());
            bin as u8
        })
        .collect()
}
```

### Dream experiment design

```rust
/// A planned interventional experiment to be run during REM dream.
#[derive(Clone, Debug)]
pub struct DreamExperiment {
    /// Unique experiment ID
    pub id: u64,
    /// The causal edge being tested: source -> target
    pub hypothesis_source: VariableId,
    pub hypothesis_target: VariableId,
    /// The intervention: what value to force the source variable to.
    /// Expressed as a delta from current value (e.g., +1000.0 for a 1000 ETH swap).
    pub intervention_delta: f64,
    /// Number of blocks to simulate after intervention
    pub simulation_blocks: u32,
    /// Expected information gain from this experiment.
    /// Higher = higher priority to run.
    pub information_gain: f64,
    /// The variables to measure in the simulation output
    pub measurement_targets: Vec<VariableId>,
    /// Current status
    pub status: ExperimentStatus,
    /// Results (populated after execution)
    pub result: Option<ExperimentResult>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExperimentStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct ExperimentResult {
    /// Observed change in the target variable (intervention fork vs. control fork)
    pub target_delta: f64,
    /// Observed changes in all measured variables
    pub all_deltas: HashMap<VariableId, f64>,
    /// Statistical significance of the observed effect
    /// (from comparing multiple simulation runs with noise)
    pub p_value: f64,
    /// The estimated causal coefficient: target_delta / intervention_delta
    pub estimated_coefficient: f64,
    /// Fork state hash for provenance
    pub fork_state_hash: [u8; 32],
    /// Number of simulation runs that were aggregated
    pub n_runs: u32,
}

/// Designs interventional experiments for REM dreams.
/// Prioritizes experiments by expected information gain:
/// how much would confirming or disconfirming this edge change
/// the causal graph?
pub struct InterventionalDreamDesigner {
    /// Counter for experiment IDs
    next_id: u64,
    /// Default number of blocks to simulate
    default_sim_blocks: u32,
    /// Maximum experiments to queue per REM cycle
    max_queue_size: usize,
}

impl InterventionalDreamDesigner {
    pub fn new() -> Self {
        InterventionalDreamDesigner {
            next_id: 0,
            default_sim_blocks: 10,
            max_queue_size: 20,
        }
    }

    /// Design experiments for edges that need interventional confirmation.
    /// Prioritizes:
    /// 1. Undirected edges in the CPDAG (orientation ambiguity)
    /// 2. Low-confidence edges (need more evidence)
    /// 3. High-impact edges (in terms of downstream causal effects)
    pub fn design_experiments(
        &mut self,
        graph: &CausalGraph,
        undirected_edges: &[(VariableId, VariableId)],
    ) -> Vec<DreamExperiment> {
        let mut candidates: Vec<DreamExperiment> = Vec::new();

        // Category 1: Undirected edges need orientation
        for &(a, b) in undirected_edges {
            let info_gain = self.compute_orientation_info_gain(graph, a, b);
            candidates.push(DreamExperiment {
                id: self.next_id(),
                hypothesis_source: a,
                hypothesis_target: b,
                intervention_delta: self.default_intervention_size(graph, a),
                simulation_blocks: self.default_sim_blocks,
                information_gain: info_gain,
                measurement_targets: vec![b],
                status: ExperimentStatus::Queued,
                result: None,
            });
        }

        // Category 2: Low-confidence directed edges need confirmation
        for edge in graph.all_edges() {
            if edge.confidence < 0.5 && edge.evidence_type != CausalEvidenceType::Interventional {
                let info_gain = self.compute_confirmation_info_gain(edge);
                candidates.push(DreamExperiment {
                    id: self.next_id(),
                    hypothesis_source: edge.source,
                    hypothesis_target: edge.target,
                    intervention_delta: self.default_intervention_size(graph, edge.source),
                    simulation_blocks: self.default_sim_blocks,
                    information_gain: info_gain,
                    measurement_targets: graph.children(edge.source).to_vec(),
                    status: ExperimentStatus::Queued,
                    result: None,
                });
            }
        }

        // Sort by information gain (descending) and take top N
        candidates.sort_by(|a, b| {
            b.information_gain
                .partial_cmp(&a.information_gain)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(self.max_queue_size);

        candidates
    }

    /// Process the result of a completed experiment.
    /// Returns an updated CausalEdge to be added to the graph.
    pub fn process_result(
        &self,
        experiment: &DreamExperiment,
        current_tick: u64,
    ) -> Option<CausalEdge> {
        let result = experiment.result.as_ref()?;

        // If the target moved significantly in response to the intervention,
        // we have interventional evidence for the edge.
        if result.p_value < 0.05 && result.estimated_coefficient.abs() > 1e-6 {
            Some(CausalEdge {
                source: experiment.hypothesis_source,
                target: experiment.hypothesis_target,
                evidence_type: CausalEvidenceType::Interventional,
                strength: result.estimated_coefficient,
                confidence: 1.0 - result.p_value, // Higher significance = higher confidence
                lag_ticks: 1, // Refine with actual measurement
                last_confirmed: current_tick,
                sample_count: result.n_runs as u64,
                witness_dag_id: Some(WitnessDagNodeId(result.fork_state_hash)),
            })
        } else {
            // Intervention had no effect: weaken or remove the edge
            None
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Information gain from orienting an undirected edge.
    /// Higher when the edge connects to many other nodes (more downstream impact).
    fn compute_orientation_info_gain(
        &self,
        graph: &CausalGraph,
        a: VariableId,
        b: VariableId,
    ) -> f64 {
        let a_degree = graph.children(a).len() + graph.parents(a).len();
        let b_degree = graph.children(b).len() + graph.parents(b).len();
        // More connected nodes produce higher information gain when oriented
        (a_degree + b_degree) as f64 + 1.0
    }

    /// Information gain from confirming a low-confidence edge.
    /// Higher when confidence is lower (more uncertainty to resolve).
    fn compute_confirmation_info_gain(&self, edge: &CausalEdge) -> f64 {
        // Binary entropy of the confidence: maximum at confidence = 0.5
        let p = edge.confidence;
        if p <= 0.0 || p >= 1.0 {
            return 0.0;
        }
        -(p * p.ln() + (1.0 - p) * (1.0 - p).ln())
    }

    /// Determine a reasonable intervention size for a variable.
    /// Based on the variable's observed range in the graph.
    fn default_intervention_size(&self, _graph: &CausalGraph, _var: VariableId) -> f64 {
        // In a full implementation, this would look at the variable's
        // historical distribution and pick an intervention at the 90th percentile.
        // For now, return a placeholder.
        1.0
    }
}
```

### HDC encoding of causal edges

```rust
/// HDC encoding for causal relationships.
/// Each causal edge (source -> target) is encoded as a hypervector
/// using BSC bind, enabling fast similarity queries.
pub struct CausalHdcCodebook {
    /// Dimension of the hypervectors (matches the system-wide BSC dimension)
    dim: usize,
    /// Base hypervectors for each variable (randomly generated on registration)
    variable_hvs: HashMap<VariableId, Vec<u64>>,
    /// Encoded causal edges: bind(source_hv, target_hv)
    edge_hvs: Vec<(VariableId, VariableId, Vec<u64>)>,
    /// Number of u64 words per hypervector
    words: usize,
}

impl CausalHdcCodebook {
    /// Create a new codebook with the given hypervector dimension.
    /// Default for Bardo: 10,240 bits.
    pub fn new(dim: usize) -> Self {
        CausalHdcCodebook {
            dim,
            variable_hvs: HashMap::new(),
            edge_hvs: Vec::new(),
            words: (dim + 63) / 64,
        }
    }

    /// Register a variable with a random base hypervector.
    /// Uses a simple xorshift PRNG seeded from the variable ID.
    pub fn register_variable(&mut self, var: VariableId) {
        if self.variable_hvs.contains_key(&var) {
            return;
        }
        let mut hv = vec![0u64; self.words];
        let mut state = var.0 ^ 0x517cc1b727220a95;
        for word in &mut hv {
            // xorshift64
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *word = state;
        }
        self.variable_hvs.insert(var, hv);
    }

    /// Encode a causal edge as bind(source_hv, target_hv).
    /// The result is quasi-orthogonal to both source and target HVs,
    /// representing the relationship itself as a first-class object.
    pub fn encode_edge(&mut self, source: VariableId, target: VariableId) {
        self.register_variable(source);
        self.register_variable(target);

        let source_hv = &self.variable_hvs[&source];
        let target_hv = &self.variable_hvs[&target];

        // BSC bind = XOR
        let edge_hv: Vec<u64> = source_hv
            .iter()
            .zip(target_hv.iter())
            .map(|(&a, &b)| a ^ b)
            .collect();

        self.edge_hvs.push((source, target, edge_hv));
    }

    /// Query: "What does variable X cause?"
    /// Bind query_hv with X's base HV, then find the closest edge HV.
    /// The closest match's target is the answer.
    pub fn query_effects(&self, source: VariableId) -> Vec<(VariableId, f64)> {
        let source_hv = match self.variable_hvs.get(&source) {
            Some(hv) => hv,
            None => return vec![],
        };

        let mut results: Vec<(VariableId, f64)> = self
            .edge_hvs
            .iter()
            .filter(|(s, _, _)| *s == source)
            .map(|(_, target, edge_hv)| {
                let sim = self.cosine_similarity(source_hv, edge_hv);
                (*target, sim)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Query: "What causes variable Y?"
    /// Find all edges whose target matches Y.
    pub fn query_causes(&self, target: VariableId) -> Vec<(VariableId, f64)> {
        let target_hv = match self.variable_hvs.get(&target) {
            Some(hv) => hv,
            None => return vec![],
        };

        let mut results: Vec<(VariableId, f64)> = self
            .edge_hvs
            .iter()
            .filter(|(_, t, _)| *t == target)
            .map(|(source, _, edge_hv)| {
                let sim = self.cosine_similarity(target_hv, edge_hv);
                (*source, sim)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Query: "Is this new edge similar to any known causal relationship?"
    /// Encode the candidate edge and find nearest neighbors in the codebook.
    pub fn query_similar_edges(
        &self,
        source: VariableId,
        target: VariableId,
        top_k: usize,
    ) -> Vec<(VariableId, VariableId, f64)> {
        let source_hv = match self.variable_hvs.get(&source) {
            Some(hv) => hv,
            None => return vec![],
        };
        let target_hv = match self.variable_hvs.get(&target) {
            Some(hv) => hv,
            None => return vec![],
        };

        // Encode the query edge
        let query_hv: Vec<u64> = source_hv
            .iter()
            .zip(target_hv.iter())
            .map(|(&a, &b)| a ^ b)
            .collect();

        let mut results: Vec<(VariableId, VariableId, f64)> = self
            .edge_hvs
            .iter()
            .map(|(s, t, hv)| {
                let sim = self.hamming_similarity(&query_hv, hv);
                (*s, *t, sim)
            })
            .collect();

        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// Hamming similarity: fraction of matching bits.
    /// 0.5 = random (quasi-orthogonal), 1.0 = identical, 0.0 = complementary.
    fn hamming_similarity(&self, a: &[u64], b: &[u64]) -> f64 {
        let total_bits = self.dim;
        let matching: u32 = a
            .iter()
            .zip(b.iter())
            .map(|(&x, &y)| (!(x ^ y)).count_ones())
            .sum();
        matching as f64 / total_bits as f64
    }

    /// Cosine-like similarity in BSC space.
    /// Maps Hamming similarity to [-1, 1] range: 2 * hamming_sim - 1.
    fn cosine_similarity(&self, a: &[u64], b: &[u64]) -> f64 {
        2.0 * self.hamming_similarity(a, b) - 1.0
    }
}
```

### The orchestrating engine

```rust
/// Configuration for the causal discovery engine.
pub struct CausalConfig {
    /// PC algorithm configuration
    pub pc: PcConfig,
    /// Granger test configuration
    pub granger: GrangerConfig,
    /// Maximum observations to buffer before oldest are evicted
    pub max_observation_buffer: usize,
    /// Confidence decay rate (lambda in the Ebbinghaus curve)
    pub confidence_decay_rate: f64,
    /// Minimum confidence to keep an edge (below this, prune)
    pub min_edge_confidence: f64,
    /// Whether to run CMI tests at Delta tick (expensive but thorough)
    pub use_cmi_at_delta: bool,
}

impl Default for CausalConfig {
    fn default() -> Self {
        CausalConfig {
            pc: PcConfig::default(),
            granger: GrangerConfig::default(),
            max_observation_buffer: 10_000,
            confidence_decay_rate: 0.001,
            min_edge_confidence: 0.1,
            use_cmi_at_delta: true,
        }
    }
}

/// An insight produced by the causal discovery engine.
/// Emitted at Theta tick when new causal knowledge is found.
#[derive(Clone, Debug)]
pub struct CausalInsight {
    /// What changed
    pub kind: InsightKind,
    /// The edge involved
    pub source: VariableId,
    pub target: VariableId,
    /// Confidence in the insight
    pub confidence: f64,
}

#[derive(Clone, Debug)]
pub enum InsightKind {
    /// A new causal edge was discovered
    NewEdge { evidence: CausalEvidenceType },
    /// An existing edge was strengthened by new evidence
    EdgeStrengthened { old_confidence: f64 },
    /// An existing edge was weakened or removed
    EdgeWeakened { new_confidence: f64 },
    /// An edge was confirmed by interventional experiment
    EdgeConfirmedByDream,
    /// An edge was refuted by interventional experiment
    EdgeRefutedByDream,
    /// A causal prediction was made (X changed, Y should follow)
    CausalPrediction { predicted_delta: f64, lag_ticks: u32 },
}

/// Death testament: the causal graph state to be inherited by the next generation.
pub struct CausalTestament {
    /// Serialized causal graph
    pub graph_snapshot: CausalGraph,
    /// Variable metadata for deserialization
    pub variable_registry: Vec<CausalVariable>,
    /// Summary statistics
    pub total_edges: usize,
    pub interventionally_confirmed: usize,
    pub average_confidence: f64,
    /// Tick when the testament was created
    pub created_at: u64,
}

/// The main causal discovery engine. Orchestrates the PC algorithm,
/// Granger tests, dream experiments, and graph maintenance.
///
/// Integrates with the heartbeat via gamma_tick, theta_tick, delta_tick,
/// and the dream system via dream_nrem and dream_rem.
pub struct CausalDiscoveryEngine {
    /// The live causal graph
    graph: CausalGraph,
    /// PC algorithm state (incremental)
    pc: PcAlgorithm,
    /// Granger causality tester
    granger: GrangerCausalityTest,
    /// Observation buffer for the sliding window
    observation_buffer: VecDeque<ObservationRecord>,
    /// Queue of dream experiments to run
    dream_queue: VecDeque<DreamExperiment>,
    /// Dream experiment designer
    dream_designer: InterventionalDreamDesigner,
    /// HDC codebook for fast causal queries
    hdc_codebook: CausalHdcCodebook,
    /// Configuration
    config: CausalConfig,
    /// Current tick (updated each gamma_tick)
    current_tick: u64,
    /// Running statistics for variables (mean, variance, covariance)
    running_stats: RunningStats,
    /// Pending causal predictions (emitted at gamma, checked later)
    pending_predictions: Vec<CausalPrediction>,
}

/// Tracks running mean and variance for streaming computation.
struct RunningStats {
    means: HashMap<VariableId, f64>,
    variances: HashMap<VariableId, f64>,
    counts: HashMap<VariableId, u64>,
}

impl RunningStats {
    fn new() -> Self {
        RunningStats {
            means: HashMap::new(),
            variances: HashMap::new(),
            counts: HashMap::new(),
        }
    }

    /// Welford's online algorithm for updating mean and variance.
    fn update(&mut self, var: VariableId, value: f64) {
        let count = self.counts.entry(var).or_insert(0);
        *count += 1;
        let n = *count as f64;

        let mean = self.means.entry(var).or_insert(0.0);
        let delta = value - *mean;
        *mean += delta / n;

        let variance = self.variances.entry(var).or_insert(0.0);
        let delta2 = value - *mean;
        *variance += delta * delta2;
    }

    fn current_variance(&self, var: VariableId) -> f64 {
        let count = self.counts.get(&var).copied().unwrap_or(0);
        if count < 2 {
            return 0.0;
        }
        self.variances.get(&var).copied().unwrap_or(0.0) / (count - 1) as f64
    }
}

struct CausalPrediction {
    source: VariableId,
    target: VariableId,
    predicted_delta: f64,
    expected_at_tick: u64,
    edge_confidence: f64,
}

impl CausalDiscoveryEngine {
    pub fn new(config: CausalConfig) -> Self {
        let pc_config = config.pc.clone();
        let granger_config = config.granger.clone();

        CausalDiscoveryEngine {
            graph: CausalGraph::new(),
            pc: PcAlgorithm::new(pc_config),
            granger: GrangerCausalityTest::new(granger_config),
            observation_buffer: VecDeque::with_capacity(config.max_observation_buffer),
            dream_queue: VecDeque::new(),
            dream_designer: InterventionalDreamDesigner::new(),
            hdc_codebook: CausalHdcCodebook::new(10_240),
            config,
            current_tick: 0,
            running_stats: RunningStats::new(),
            pending_predictions: Vec::new(),
        }
    }

    /// Register a new DeFi variable to track.
    pub fn register_variable(&mut self, protocol: &str, metric: &str) -> VariableId {
        let id = self.graph.add_variable(protocol, metric);
        self.hdc_codebook.register_variable(id);
        id
    }

    /// Gamma tick: ingest observations and emit causal predictions.
    ///
    /// Budget: < 1ms. Only bookkeeping and prediction lookup.
    pub fn gamma_tick(&mut self, observations: &[Observation]) -> Vec<CausalInsight> {
        let mut insights = Vec::new();

        for obs in observations {
            self.current_tick = obs.tick;

            // Update running statistics
            for (&var, &value) in &obs.values {
                self.running_stats.update(var, value);
            }

            // Append to observation buffer
            self.observation_buffer.push_back(ObservationRecord {
                tick: obs.tick,
                block_number: 0, // Filled by caller
                timestamp_secs: 0,
                values: obs.values.clone(),
            });

            // Evict oldest if over capacity
            while self.observation_buffer.len() > self.config.max_observation_buffer {
                self.observation_buffer.pop_front();
            }

            // Feed Granger test
            self.granger.observe(obs);

            // Check for causal predictions: if a variable with outgoing edges
            // changed significantly, predict downstream effects
            for (&var, &value) in &obs.values {
                let variance = self.running_stats.current_variance(var);
                if variance <= 0.0 {
                    continue;
                }
                let mean = self.running_stats.means.get(&var).copied().unwrap_or(0.0);
                let z_score = (value - mean) / variance.sqrt();

                // Only predict for significant deviations (|z| > 2)
                if z_score.abs() > 2.0 {
                    for &child in self.graph.children(var) {
                        if let Some(edge) = self.graph.edge(var, child) {
                            if edge.confidence > 0.3 {
                                let predicted_delta = (value - mean) * edge.strength;
                                insights.push(CausalInsight {
                                    kind: InsightKind::CausalPrediction {
                                        predicted_delta,
                                        lag_ticks: edge.lag_ticks,
                                    },
                                    source: var,
                                    target: child,
                                    confidence: edge.confidence,
                                });

                                self.pending_predictions.push(CausalPrediction {
                                    source: var,
                                    target: child,
                                    predicted_delta,
                                    expected_at_tick: obs.tick + edge.lag_ticks as u64,
                                    edge_confidence: edge.confidence,
                                });
                            }
                        }
                    }
                }
            }

            // Check pending predictions against actual observations
            self.pending_predictions.retain(|pred| {
                if obs.tick >= pred.expected_at_tick {
                    if let Some(&actual) = obs.values.get(&pred.target) {
                        let mean = self
                            .running_stats
                            .means
                            .get(&pred.target)
                            .copied()
                            .unwrap_or(0.0);
                        let actual_delta = actual - mean;

                        // If prediction direction matches, reinforce the edge
                        let direction_match =
                            pred.predicted_delta.signum() == actual_delta.signum();

                        if let Some(edge) = self
                            .graph
                            .edges
                            .get_mut(&(pred.source, pred.target))
                        {
                            if direction_match {
                                edge.confidence =
                                    (edge.confidence + 0.02).min(1.0);
                                edge.last_confirmed = obs.tick;
                            } else {
                                edge.confidence =
                                    (edge.confidence - 0.05).max(0.0);
                            }
                        }
                    }
                    false // Remove prediction from pending list
                } else {
                    true // Keep waiting
                }
            });
        }

        insights
    }

    /// Theta tick: run incremental causal discovery.
    ///
    /// Budget: < 500ms. Runs a batch of PC algorithm CI tests and
    /// pairwise Granger tests.
    pub fn theta_tick(&mut self) -> Vec<CausalInsight> {
        let mut insights = Vec::new();
        let data: Vec<ObservationRecord> = self.observation_buffer.iter().cloned().collect();

        if data.len() < 50 {
            return insights; // Not enough data yet
        }

        // Initialize PC if needed (new variables added since last init)
        let current_vars = self.graph.variable_ids();
        if !current_vars.is_empty() {
            // Run a batch of PC algorithm tests
            let ci_test = PartialCorrelationTest;
            let complete = self.pc.run_batch(&ci_test, &data);

            if complete {
                // Orient edges and update the causal graph
                let directed = self.pc.orient_edges();
                for (source, target) in &directed {
                    if self.graph.edge(*source, *target).is_none() {
                        let edge = CausalEdge {
                            source: *source,
                            target: *target,
                            evidence_type: CausalEvidenceType::Observational,
                            strength: 0.0, // Estimated later
                            confidence: 0.6, // Moderate initial confidence
                            lag_ticks: 1,
                            last_confirmed: self.current_tick,
                            sample_count: data.len() as u64,
                            witness_dag_id: None,
                        };
                        self.graph.add_edge(edge);
                        self.hdc_codebook.encode_edge(*source, *target);

                        insights.push(CausalInsight {
                            kind: InsightKind::NewEdge {
                                evidence: CausalEvidenceType::Observational,
                            },
                            source: *source,
                            target: *target,
                            confidence: 0.6,
                        });
                    }
                }

                // Re-initialize PC for the next cycle
                self.pc.initialize(&current_vars);
            }
        }

        // Run pairwise Granger tests on testable variables
        let testable = self.granger.testable_variables();
        for i in 0..testable.len() {
            for j in 0..testable.len() {
                if i == j {
                    continue;
                }
                let result = self.granger.test(testable[i], testable[j]);
                if result.granger_causes {
                    let existing = self.graph.edge(testable[i], testable[j]);
                    if existing.is_none() {
                        let edge = CausalEdge {
                            source: testable[i],
                            target: testable[j],
                            evidence_type: CausalEvidenceType::GrangerTemporal,
                            strength: result.transfer_entropy,
                            confidence: 0.4, // Lower than observational (Granger != causation)
                            lag_ticks: result.optimal_lag as u32,
                            last_confirmed: self.current_tick,
                            sample_count: self.config.granger.window_size as u64,
                            witness_dag_id: None,
                        };
                        self.graph.add_edge(edge);
                        self.hdc_codebook.encode_edge(testable[i], testable[j]);

                        insights.push(CausalInsight {
                            kind: InsightKind::NewEdge {
                                evidence: CausalEvidenceType::GrangerTemporal,
                            },
                            source: testable[i],
                            target: testable[j],
                            confidence: 0.4,
                        });
                    } else if let Some(existing) = existing {
                        if existing.confidence < 0.4 {
                            // Granger evidence reinforces a weak edge
                            insights.push(CausalInsight {
                                kind: InsightKind::EdgeStrengthened {
                                    old_confidence: existing.confidence,
                                },
                                source: testable[i],
                                target: testable[j],
                                confidence: 0.4,
                            });
                        }
                    }
                }
            }
        }

        insights
    }

    /// Delta tick: full graph audit, CMI-based testing, pruning, grimoire update.
    ///
    /// Budget: < 30 seconds. Runs expensive operations.
    pub fn delta_tick(&mut self) {
        let data: Vec<ObservationRecord> = self.observation_buffer.iter().cloned().collect();

        // Decay all edge confidences
        self.graph
            .decay_confidences(self.config.confidence_decay_rate, self.current_tick);

        // Prune stale edges
        let pruned = self
            .graph
            .prune_stale_edges(self.config.min_edge_confidence);
        for (source, target) in &pruned {
            // Log pruned edges for audit
            let _ = (source, target); // In production: emit to audit log
        }

        // Full CMI-based PC algorithm pass (if enabled)
        if self.config.use_cmi_at_delta && data.len() >= 100 {
            let cmi_test = MutualInformationTest::new(5);
            let vars = self.graph.variable_ids();
            let mut full_pc = PcAlgorithm::new(PcConfig {
                significance_level: 0.01, // Stricter for Delta audit
                max_conditioning_size: 3,
                max_tests_per_tick: 5000, // Larger budget at Delta
            });
            full_pc.initialize(&vars);

            // Run to completion
            loop {
                if full_pc.run_batch(&cmi_test, &data) {
                    break;
                }
            }

            // Compare with current graph for inconsistencies
            let cmi_directed = full_pc.orient_edges();
            for (source, target) in &cmi_directed {
                if let Some(edge) = self.graph.edge(*source, *target) {
                    // Edge confirmed by CMI: reinforce
                    let key = (*source, *target);
                    if let Some(e) = self.graph.edges.get_mut(&key) {
                        e.last_confirmed = self.current_tick;
                        e.confidence = (e.confidence + 0.1).min(1.0);
                    }
                }
            }
        }

        // DAG consistency check
        if !self.graph.is_dag() {
            // Graph has cycles. This can happen when observational and Granger
            // evidence create conflicting edges. Resolve by removing the
            // weakest edge in each cycle.
            self.break_cycles();
        }

        // Design dream experiments for the next REM cycle
        let undirected: Vec<(VariableId, VariableId)> = Vec::new(); // From PC's unoriented edges
        let experiments = self.dream_designer.design_experiments(&self.graph, &undirected);
        for exp in experiments {
            self.dream_queue.push_back(exp);
        }
    }

    /// NREM dream: replay past observation sequences and validate
    /// causal predictions.
    pub fn dream_nrem(&mut self, replay: &[ObservationRecord]) {
        // Replay the sequence through the causal model.
        // For each observation, check if the causal graph's predictions
        // match the actual outcome.
        for window in replay.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            // For each edge, check if the predicted direction matches
            for edge_key in self.graph.edges.keys().cloned().collect::<Vec<_>>() {
                let (source, target) = edge_key;
                let source_prev = prev.values.get(&source).copied();
                let source_curr = curr.values.get(&source).copied();
                let target_prev = prev.values.get(&target).copied();
                let target_curr = curr.values.get(&target).copied();

                if let (Some(sp), Some(sc), Some(tp), Some(tc)) =
                    (source_prev, source_curr, target_prev, target_curr)
                {
                    let source_delta = sc - sp;
                    let target_delta = tc - tp;

                    if source_delta.abs() > 1e-8 && target_delta.abs() > 1e-8 {
                        if let Some(edge) = self.graph.edges.get_mut(&edge_key) {
                            let predicted_sign = (source_delta * edge.strength).signum();
                            let actual_sign = target_delta.signum();

                            if predicted_sign == actual_sign {
                                // Prediction confirmed: reinforce
                                edge.confidence = (edge.confidence + 0.01).min(1.0);
                                edge.last_confirmed = curr.tick;
                            } else {
                                // Prediction failed: penalize
                                edge.confidence = (edge.confidence - 0.02).max(0.0);
                            }
                        }
                    }
                }
            }
        }
    }

    /// REM dream: run interventional experiments via Mirage-rs.
    ///
    /// The `mirage` parameter is a handle to the fork simulation engine.
    /// Returns the experiments that were run (with results populated).
    ///
    /// In a full implementation, MirageSimulator would be the mirage-rs
    /// EvmExecutor. Here we define the interface.
    pub fn dream_rem(
        &mut self,
        mirage: &mut dyn MirageSimulator,
    ) -> Vec<DreamExperiment> {
        let mut completed = Vec::new();
        let budget = 5; // Max experiments per REM cycle

        for _ in 0..budget {
            let mut experiment = match self.dream_queue.pop_front() {
                Some(e) => e,
                None => break,
            };

            experiment.status = ExperimentStatus::Running;

            // Run the experiment on Mirage-rs
            match mirage.run_experiment(&experiment) {
                Ok(result) => {
                    experiment.result = Some(result);
                    experiment.status = ExperimentStatus::Completed;

                    // Process the result and update the causal graph
                    if let Some(edge) =
                        self.dream_designer.process_result(&experiment, self.current_tick)
                    {
                        self.graph.add_edge(edge);
                        self.hdc_codebook
                            .encode_edge(experiment.hypothesis_source, experiment.hypothesis_target);
                    } else {
                        // Experiment showed no effect: weaken the edge
                        if let Some(e) = self.graph.edges.get_mut(&(
                            experiment.hypothesis_source,
                            experiment.hypothesis_target,
                        )) {
                            e.confidence *= 0.3; // Steep penalty for interventional disconfirmation
                        }
                    }
                }
                Err(msg) => {
                    experiment.status = ExperimentStatus::Failed(msg);
                }
            }

            completed.push(experiment);
        }

        completed
    }

    /// Generate a death testament: the causal graph state to be inherited.
    pub fn death_testament(&self) -> CausalTestament {
        let total = self.graph.edge_count();
        let interventional = self
            .graph
            .all_edges()
            .filter(|e| {
                e.evidence_type == CausalEvidenceType::Interventional
                    || e.evidence_type == CausalEvidenceType::Hybrid
            })
            .count();

        let avg_conf = if total > 0 {
            self.graph.all_edges().map(|e| e.confidence).sum::<f64>() / total as f64
        } else {
            0.0
        };

        CausalTestament {
            graph_snapshot: CausalGraph::new(), // In production: deep clone
            variable_registry: self
                .graph
                .variable_ids()
                .iter()
                .filter_map(|&id| self.graph.variable(id).cloned())
                .collect(),
            total_edges: total,
            interventionally_confirmed: interventional,
            average_confidence: avg_conf,
            created_at: self.current_tick,
        }
    }

    /// Break cycles in the graph by removing the weakest edge in each cycle.
    fn break_cycles(&mut self) {
        // Repeatedly find and break cycles until the graph is a DAG.
        // Detect a cycle via DFS, then remove the lowest-confidence edge in it.
        loop {
            if let Some(cycle) = self.find_cycle() {
                // Find the weakest edge in the cycle
                let mut weakest: Option<(VariableId, VariableId, f64)> = None;
                for window in cycle.windows(2) {
                    let (s, t) = (window[0], window[1]);
                    if let Some(edge) = self.graph.edge(s, t) {
                        match &weakest {
                            None => weakest = Some((s, t, edge.confidence)),
                            Some((_, _, conf)) if edge.confidence < *conf => {
                                weakest = Some((s, t, edge.confidence));
                            }
                            _ => {}
                        }
                    }
                }

                if let Some((s, t, _)) = weakest {
                    self.graph.remove_edge(s, t);
                } else {
                    break;
                }
            } else {
                break; // No more cycles
            }
        }
    }

    /// Find a cycle in the graph using DFS. Returns the cycle as a path,
    /// or None if the graph is acyclic.
    fn find_cycle(&self) -> Option<Vec<VariableId>> {
        let vars = self.graph.variable_ids();
        let mut visited: HashSet<VariableId> = HashSet::new();
        let mut in_stack: HashSet<VariableId> = HashSet::new();
        let mut parent: HashMap<VariableId, VariableId> = HashMap::new();

        for &start in &vars {
            if visited.contains(&start) {
                continue;
            }

            let mut stack: Vec<(VariableId, bool)> = vec![(start, false)];

            while let Some((node, processed)) = stack.pop() {
                if processed {
                    in_stack.remove(&node);
                    continue;
                }

                if in_stack.contains(&node) {
                    // Found a cycle. Reconstruct it.
                    let mut cycle = vec![node];
                    let mut current = *parent.get(&node).unwrap_or(&node);
                    while current != node {
                        cycle.push(current);
                        current = *parent.get(&current).unwrap_or(&node);
                    }
                    cycle.push(node);
                    cycle.reverse();
                    return Some(cycle);
                }

                if visited.contains(&node) {
                    continue;
                }

                visited.insert(node);
                in_stack.insert(node);
                stack.push((node, true)); // Mark for post-processing

                for &child in self.graph.children(node) {
                    parent.insert(child, node);
                    stack.push((child, false));
                }
            }
        }

        None
    }
}

/// Interface to the Mirage-rs fork simulation engine.
/// The causal discovery engine uses this to run do(X) experiments.
pub trait MirageSimulator {
    /// Fork current mainnet state, execute the intervention, simulate
    /// subsequent blocks, and return the measured effects.
    fn run_experiment(
        &mut self,
        experiment: &DreamExperiment,
    ) -> Result<ExperimentResult, String>;
}
```

---

## DeFi primitive coverage: the cross-primitive causal graph

The causal discovery engine does not start from scratch. It begins with a set of causal hypotheses derived from known DeFi mechanisms. These are priors, not certainties. Each hypothesis enters the graph as a low-confidence edge that the engine must confirm, refute, or refine through observation and experimentation.

### DEX to lending: the arbitrage pressure channel

**Hypothesis:** A large swap on a Uniswap V3 pool causes a lending rate change on Aave within 1-5 blocks.

**Mechanism:** The swap moves the spot price. Borrowers on Aave whose collateral is the swapped token see their collateral-to-debt ratio shift. If the price moves against them, they approach liquidation thresholds, increasing effective utilization. Aave's interest rate model responds to utilization with a piecewise linear function that steepens sharply above the optimal utilization threshold (typically 80%). MEV bots may front-run liquidations, compressing the lag.

**Causal chain:** swap_volume -> spot_price -> collateral_value -> utilization -> borrow_rate

**Variables to track:**
- uniswap_v3.swap_volume_eth (source)
- uniswap_v3.sqrt_price_x96 (intermediate)
- aave_v3.collateral_value_eth (intermediate)
- aave_v3.utilization_usdc (intermediate)
- aave_v3.borrow_rate_usdc (target)

**Dream experiment:** Simulate a 1,000 ETH swap on the WETH/USDC pool. Fork state, execute swap, replay 10 blocks. Measure the Aave borrow rate oracle update, borrower health factor changes, and any liquidation events.

### Vault outflows to LP removal: capital rotation

**Hypothesis:** Outflow from yield vaults (e.g., Yearn, Convex) precedes removal of LP positions on the underlying DEX pools.

**Mechanism:** Vault depositors withdrawing capital often need to unwind their positions in the underlying protocols. A vault holding Curve LP tokens must burn those tokens (removing liquidity) to return the underlying assets. The lag depends on the vault's withdrawal queue and any lockup periods.

**Variables to track:**
- yearn.tvl_delta (source)
- curve.lp_removal_volume (target)
- curve.pool_depth (downstream)

### Funding rate deviation to spot price movement

**Hypothesis:** Persistent funding rate deviation on perpetual exchanges predicts spot price movement as arbitrageurs close the basis.

**Mechanism:** When the perpetual funding rate diverges from neutral, arbitrageurs enter basis trades: long spot / short perp (when funding is positive) or short spot / long perp (when negative). This puts directional pressure on the spot market. The lag depends on arbitrageur capital availability and gas costs.

**Variables to track:**
- perp_exchange.funding_rate (source)
- uniswap_v3.spot_price (target)

### Gas spike to liquidation cascade

**Hypothesis:** Sudden gas price spikes trigger liquidation cascades by preventing borrowers from topping up collateral or repaying debt.

**Mechanism:** Borrowers near liquidation thresholds rely on being able to submit collateral top-up transactions before liquidators act. A gas spike raises the cost of these transactions, causing some borrowers to delay or abandon the attempt. Liquidators, who operate with dedicated gas budgets and priority fee strategies, continue executing. The result is a burst of liquidations concentrated in the blocks immediately following the gas spike.

**Variables to track:**
- gas.base_fee_gwei (source)
- aave_v3.liquidation_count (target)
- aave_v3.utilization (downstream)

### Bridge inflow to DEX volume

**Hypothesis:** Token bridge inflows (from L2s or other chains) predict DEX volume spikes on the receiving chain within 10-60 minutes.

**Mechanism:** Capital bridged to Ethereum mainnet typically needs to be swapped on arrival. Bridge users either swap immediately or deposit into DeFi protocols, both of which generate on-chain activity. The lag depends on the bridge finality time and the user's intention.

**Variables to track:**
- bridge.eth_inflow_volume (source)
- uniswap_v3.total_volume (target)

### Restaking delegation to staking yield

**Hypothesis:** Shifts in restaking delegation (EigenLayer) predict changes in base staking yield.

**Mechanism:** Restaking locks ETH that would otherwise be available for native staking or other yield sources. Large delegation shifts change the effective supply of staking capital, which affects the base staking yield through Ethereum's validator reward curve (yield decreases as total staked ETH increases).

**Variables to track:**
- eigenlayer.delegation_delta (source)
- ethereum.staking_yield (target)

### Prediction market outcome to DeFi position movement

**Hypothesis:** Resolution of prediction market events (Polymarket, Augur) on topics correlated with DeFi asset prices precedes correlated DeFi position adjustments.

**Mechanism:** Prediction market outcomes provide early information signals. A market resolving "yes" on a regulatory question affecting stablecoins could trigger stablecoin de-pegging hedges, lending rate adjustments, and LP position changes. The causal link here is informational: the prediction market reveals information faster than the DeFi protocols price it in.

**Variables to track:**
- polymarket.event_resolution (source)
- various.position_changes (target)

### Intent-based batch settlement to price impact

**Hypothesis:** Batch settlement by intent-based protocols (CoW Swap, UniswapX) creates characteristic price impact patterns distinguishable from continuous AMM trading.

**Mechanism:** Intent-based systems batch multiple orders and settle them simultaneously, creating a point-in-time price impact that differs from the continuous small impacts of individual AMM swaps. The settlement transaction concentrates price movement into a single block, potentially creating arbitrage opportunities in adjacent blocks.

**Variables to track:**
- cowswap.batch_settlement_volume (source)
- uniswap_v3.price_impact_per_block (target)

---

## The cybernetic loop

The causal discovery system is a cybernetic feedback loop. Each cycle refines the Golem's causal model, and the model in turn directs the next cycle's attention and experimentation.

**Observe.** At Gamma tick, the Golem ingests protocol state changes. The observation buffer grows. Running statistics update. If a tracked variable changes significantly and the causal graph predicts downstream effects, the Golem emits a causal prediction and raises its attention bid on the predicted target variable.

**Hypothesize.** At Theta tick, the PC algorithm and Granger tests discover candidate causal edges. These are hypotheses, not facts. The engine records them in the graph with moderate confidence and queues them for confirmation.

**Test.** During REM dreams, the dream designer selects the highest-information-gain hypotheses and runs do(X) experiments on Mirage-rs forks. The intervention breaks confounders: if the target variable moves in the fork despite no confounding macro event, the causal link is real.

**Update.** The dream results update the graph. Confirmed edges gain interventional evidence and higher confidence. Refuted edges lose confidence steeply (interventional disconfirmation is strong evidence). The graph's structure changes.

**Consolidate.** During NREM dreams, the engine replays past observation sequences through the updated causal model. Edges that correctly explain past observations are reinforced. Edges that fail are weakened. This is the Ebbinghaus decay in action: causal knowledge must be periodically re-confirmed to survive.

**Predict.** The updated graph feeds back into Gamma tick predictions. Better causal models produce better predictions. Better predictions produce better attention allocation (via the attention auction). Better attention produces more relevant observations. The loop tightens.

**Inherit.** When a Golem dies, its causal graph transfers to the next generation through the death testament and Grimoire. The new Golem starts with its predecessor's graph, subject to confidence decay during the inheritance gap. It does not need to rediscover known causal structures from scratch. Causal knowledge compounds across generations.

This compounding is the system's primary value. A first-generation Golem discovers that large swaps cause rate changes. A second-generation Golem inherits that knowledge and discovers the intermediate mechanism (collateral value -> utilization). A third-generation Golem discovers the gas price confound and learns to distinguish real causal effects from confounded correlations. Each generation's causal model is richer than the last.

---

## Evaluation protocol [SPEC]

### Graph accuracy against known mechanisms

DeFi provides ground truth for certain causal relationships because the protocol code is public. The Aave interest rate model is a deterministic function of utilization: we know the structural equation. The Uniswap V3 price impact formula is specified in the contract. These known mechanisms serve as benchmarks.

**Test:** Initialize the engine on a period of historical data. Let it discover the causal graph. Compare discovered edges against the known protocol mechanics. Metrics:

- True positive rate: fraction of known causal edges that the engine discovers
- False positive rate: fraction of discovered edges that do not correspond to known mechanisms
- Edge strength accuracy: for known edges, how close is the estimated causal coefficient to the true value derived from the protocol math?

Target: TPR > 0.8, FPR < 0.2, within 500 Theta ticks of initialization.

### Interventional experiment success rate

For dream-based do(X) experiments, the success metric is prediction accuracy: does the Mirage-rs fork simulation produce the effect the causal graph predicts?

**Test:** Run 100 dream experiments on known causal edges (where we can independently verify the effect). Measure:

- Directional accuracy: fraction of experiments where the predicted effect direction matches the observed direction
- Magnitude accuracy: mean absolute error of predicted vs. observed effect size
- Null detection rate: fraction of experiments on non-causal edges that correctly return no effect

Target: directional accuracy > 0.85, null detection rate > 0.90.

### Causal prediction vs. correlation-based prediction

The core question: do causal-informed predictions outperform correlation-based ones?

**Test:** Run two prediction systems in parallel on live data:

1. Correlation-based: predict Y from X using the raw correlation coefficient
2. Causal-based: predict Y from X using the causal graph's estimated causal effect (which accounts for confounders via the adjustment formula)

Measure prediction quality (RMSE, directional accuracy) over a rolling window. The causal system should outperform in regimes where confounders are active (market stress, gas spikes, governance events) and perform comparably in stable regimes where correlations approximate causal effects.

Target: causal predictions reduce RMSE by > 15% during high-volatility periods.

### Graph adaptation speed

When the DeFi environment changes (new protocol launch, governance parameter update, market regime shift), how quickly does the causal graph adapt?

**Test:** Introduce a structural change (e.g., modify the Aave rate model parameters in a historical replay) and measure:

- Detection latency: how many Theta ticks until the engine detects the edge strength change?
- Convergence time: how many Theta ticks until the updated edge strength stabilizes within 10% of the new true value?
- False alarm rate: how often does the engine incorrectly report structural changes that didn't happen?

Target: detection within 10 Theta ticks, convergence within 50, false alarm rate < 0.05.

---

## References

Bongers, S., Forre, P., Peters, J., & Mooij, J. M. (2021). Foundations of structural causal models with cycles and latent variables. *Annals of Statistics*, 49(5), 2885-2915.

Granger, C. W. J. (1969). Investigating causal relations by econometric models and cross-spectral methods. *Econometrica*, 37(3), 424-438.

Kraskov, A., Stogbauer, H., & Grassberger, P. (2004). Estimating mutual information. *Physical Review E*, 69(6), 066138.

Marinazzo, D., Pellicoro, M., & Stramaglia, S. (2008). Kernel method for nonlinear Granger causality. *Physical Review Letters*, 100(14), 144103.

Pearl, J. (2009). *Causality: Models, Reasoning, and Inference* (2nd ed.). Cambridge University Press.

Peters, J., Janzing, D., & Scholkopf, B. (2017). *Elements of Causal Inference: Foundations and Learning Algorithms*. MIT Press.

Runge, J. (2018). Causal network reconstruction from time series: From theoretical assumptions to practical estimation. *Chaos*, 28(7), 075310.

Runge, J., Nowack, P., Kretschmer, M., Flaxman, S., & Sejdinovic, D. (2019). Detecting and quantifying causal associations in large nonlinear time series datasets. *Science Advances*, 5(11), eaau4996.

Shachter, R. D. (1998). Bayes-Ball: The rational pastime. *Proceedings of the 14th Conference on Uncertainty in Artificial Intelligence*, 480-487.

Spirtes, P., Glymour, C., & Scheines, R. (2000). *Causation, Prediction, and Search* (2nd ed.). MIT Press.

Tian, J., & Pearl, J. (2002). On the identification of causal effects. Technical Report R-290, UCLA Cognitive Systems Laboratory.
