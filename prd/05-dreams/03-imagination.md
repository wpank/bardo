# Dream Imagination: Counterfactual Reasoning and Creative Recombination [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `00-overview.md`, `01-architecture.md`, `golem-inference`

---

> **Reader orientation:** This document specifies the Imagination Engine, the REM-phase component of dreaming within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers counterfactual reasoning via Pearl's structural causal models, Hindsight Experience Replay (HER), Counterfactual Regret minimization (CFR), Boden's three creativity modes (combinational, exploratory, transformational), scenario generation, and anticipatory trajectory generation. A Golem (mortal autonomous agent) uses imagination to generate scenarios it has never experienced. Prerequisites: the Dreams overview (`00-overview.md`) and architecture (`01-architecture.md`). For a full glossary, see `prd2/shared/glossary.md`.

## From Replay to Imagination

Replay processes what happened. Imagination generates what _could_, _should_, or _might_ happen. This distinction — between consolidation and creation — maps directly onto the NREM/REM division in biological sleep. NREM stabilizes. REM innovates. Without imagination, dreaming is mere bookkeeping.

Three research traditions converge on the need for imagination in an autonomous agent: game-theoretic regret minimization, causal counterfactual reasoning, and computational creativity. Each provides a distinct capability that pure replay cannot.

---

## 1. Counterfactual Reasoning

### The Causal Framework

Pearl's (2000/2009) structural causal model (SCM) framework formalizes three levels of causal reasoning [PEARL-2000]:

1. **Association** — P(Y|X): "When gas spiked, my trade failed." Observational correlation.
2. **Intervention** — P(Y|do(X)): "If I had set a higher gas limit, would the trade have succeeded?" Causal intervention.
3. **Counterfactual** — P(Y_x|X', Y'): "Given that the trade actually failed with gas=50 gwei, would it have succeeded if gas had been 30 gwei?" Requires a full structural model.

Most trading agents — including the Golem's waking mode — operate at Level 1. They detect correlations between market features and outcomes. Counterfactual dreaming elevates the Golem to Level 2 and, where possible, Level 3. Instead of merely noting that gas spikes correlate with losses, the Golem constructs a causal DAG (gas → delay → slippage → loss) and evaluates interventions on specific variables.

**LLM-native causal reasoning**: The Golem does not formally compute do-calculus. Instead, it leverages the LLM's implicit causal reasoning to evaluate interventions through structured prompting:

```
Counterfactual Analysis: Trade #4,872

Actual outcome:
- Action: Entered ETH-USDC LP position at tick 5,200
- Context: Gas=45 gwei, ETH momentum +2.3% over 4h, pool TVL $12M
- Result: -2.3% loss. Exit forced by gas spike to 180 gwei at tick 5,847.

Construct a causal graph for this outcome. Then evaluate these
counterfactual interventions:

1. What if entry had been delayed by 50 ticks (waiting for gas to stabilize)?
2. What if position size had been 50% smaller?
3. What if a gas price ceiling of 100 gwei had been set as an exit trigger?
4. What if the pool had been ETH-DAI instead of ETH-USDC?

For each counterfactual:
- Estimate the probability of a better outcome
- Identify which causal link in the graph is being modified
- State the assumption required for the counterfactual to be valid
- Propose a PLAYBOOK.md guard that would implement this intervention
```

### Hindsight Experience Replay

Andrychowicz et al. (2017) introduced Hindsight Experience Replay (HER): even failed trajectories contain useful information. HER re-labels failed trajectories with the goal that was actually achieved, creating a successful "virtual" experience [ANDRYCHOWICZ-2017]. This functions as an implicit curriculum — the agent first learns to achieve easily reachable goals, then progressively harder ones.

**LLM-native HER**:

A trade targeting 5% profit but achieving 1.2% is a failure under the original goal. But HER-style dreaming re-labels:

```
Hindsight Re-labeling: Trade #4,913

Original goal: 5% profit on momentum trade
Actual outcome: 1.2% profit

Re-label: "What strategy would have been optimal if the goal had been 1% profit?"
- The entry was correct.
- The hold duration was excessive — the 1% target was hit at tick 200.
- A tighter take-profit at 1.5% would have captured this consistently.

New hypothesis: For this market regime (low volatility, weak momentum),
a 1.5% target with tight stops outperforms a 5% target with wide stops.
Confidence: 0.2 (dream-generated, requires live validation).
```

This extracts value from every trade, including "failures." The implicit curriculum effect is important: the Golem learns achievable strategies before attempting ambitious ones, building a foundation of reliable small wins.

### Counterfactual Regret Minimization

Zinkevich et al. (2007) developed Counterfactual Regret Minimization (CFR), which powered superhuman poker AIs Libratus and Pluribus [ZINKEVICH-2007]. CFR decomposes overall regret into counterfactual regret at each information set. By minimizing iteratively, the average strategy converges to Nash equilibrium.

For the Golem, the full CFR algorithm is not implementable (the game tree for DeFi markets is intractable). But the principle is directly applicable: **dream by simulating counterfactual market scenarios and minimizing worst-case regret across diverse conditions**:

```
Regret Minimization Dream Scenario:

Consider your current strategy portfolio. Simulate these adversarial
market conditions:
1. MEV sandwich attack during your largest pending swap
2. Oracle price delayed by 30 seconds during high volatility
3. Counterparty rug pull on your largest LP position
4. Simultaneous gas spike + liquidity withdrawal

For each scenario:
- What is the worst-case loss under your current strategy?
- What strategy modification would minimize the worst case?
- What is the cost of that modification under normal conditions?
- Is the regret reduction worth the normal-case cost?
```

### Byrne's Systematic Fault Lines

Byrne (2005) demonstrated that humans exhibit systematic "fault lines" in counterfactual thinking — they preferentially alter actions over inactions, controllable events over uncontrollable ones, and the most recent events in a sequence [BYRNE-2005]. This is not a bias but an efficient heuristic: the counterfactual space is infinite, and focusing on controllable, recent, active decisions produces the highest-value counterfactuals.

**Implementation**: The counterfactual engine prioritizes:

1. The Golem's own actions (not external market movements)
2. Controllable parameters (entry timing, position size, slippage tolerance, gas limits)
3. The most recent decisions in each trade's decision chain
4. Decisions where the Golem had multiple options and chose one

This dramatically reduces the counterfactual search space. Rather than asking "what if ETH had gone up instead of down?" (uncontrollable, uninstructive), the Golem asks "what if I had set my slippage tolerance to 0.5% instead of 1%?" (controllable, actionable).

### Epstude and Roese's Functional Theory

Epstude and Roese (2008) showed that counterfactual thinking serves behavior regulation through two pathways [EPSTUDE-ROESE-2008]:

1. **Content-specific**: Causal inference → behavioral intention. "If I had executed 200ms earlier, I would have captured the spread" → adjust latency thresholds.
2. **Content-neutral**: Counterfactual sessions as calibration warmups that sharpen risk sensitivity across all strategies, regardless of specific content.

Both pathways are implemented. The content-specific pathway produces concrete PLAYBOOK.md revisions. The content-neutral pathway means that even counterfactual sessions that produce no specific insight still improve the Golem's overall decision quality — the act of counterfactual reasoning itself sharpens calibration.

---

## 2. Creative Recombination

### Boden's Three Modes of Creativity

Margaret Boden (2004) identified three types of computational creativity [BODEN-2004]:

**Combinational creativity**: Novel juxtapositions of existing ideas. The lowest bar but often the most practically useful. A momentum strategy combined with a mean-reversion exit creates a hybrid that neither component alone could produce.

**Exploratory creativity**: Mapping the boundaries of a conceptual space. Systematically varying parameters to discover where strategies break. What happens to a yield farming strategy when APY drops below gas costs? When does a hedging strategy's cost exceed the risk it mitigates?

**Transformational creativity**: Changing the rules of the conceptual space itself. Generating ideas that were previously impossible within the old framework. "What if impermanent loss is a premium for providing liquidity information?" reframes a cost as revenue. This is the rarest and most valuable mode.

**Implementation**:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreativityMode {
    Combinational,
    Exploratory,
    Transformational,
}

pub struct CreativeSession {
    pub mode: CreativityMode,
    pub inputs: Vec<String>,
    pub temperature: f64,
    pub output: Vec<StrategyHypothesis>,
    pub deduplication_check: bool,
}
```

### Combinational Mode

**Prompt structure**:

```
Creative Recombination Session (Combinational)

You have access to these strategy components from PLAYBOOK.md:
- Strategy A: [momentum entry: enter when 4h RSI crosses above 60]
- Strategy B: [mean reversion exit: exit when price returns to 20-period SMA]
- Strategy C: [gas-aware sizing: reduce position size when gas > 50 gwei]

Generate 3 novel strategy hybrids by combining elements from different
strategies. For each:
1. Describe the hybrid strategy
2. What emergent property does the combination produce that neither
   component has alone?
3. Under what market conditions would this hybrid outperform its components?
4. What is the minimum viable test for this hypothesis?
```

### Exploratory Mode

**Prompt structure**:

```
Creative Exploration Session (Exploratory)

Strategy under test: [yield farming on ETH-USDC pool with 2% position limit]

Push this strategy to its boundaries:
1. At what APY does this strategy become unprofitable after gas costs?
2. At what pool TVL does slippage make this strategy unviable?
3. What happens if the correlation between ETH and USDC breaks?
4. What is the maximum drawdown this strategy can produce?
5. At what point does the position limit (2%) become the binding constraint?

For each boundary:
- Estimate the threshold value
- Is the current PLAYBOOK.md guard sufficient?
- What early warning signal would detect approach to this boundary?
```

### Transformational Mode

**Prompt structure**:

```
Creative Transformation Session (Transformational)

Challenge a fundamental assumption in PLAYBOOK.md:

Current assumption: "Impermanent loss is a cost to be minimized."

What if this assumption is wrong? Explore:
1. Under what framing could impermanent loss be a *benefit*?
2. What strategy would deliberately seek impermanent loss exposure?
3. What market conditions would make this strategy profitable?
4. What existing financial instrument does this most resemble?
5. How would you test whether this reframing is valid?

Note: This session deliberately generates contrarian hypotheses.
All outputs enter at confidence 0.1 and require extensive validation.
```

### Fauconnier and Turner's Conceptual Blending (v2)

Conceptual blending merges elements from diverse mental spaces into a blended space with emergent structure not present in either input [FAUCONNIER-TURNER-2002]. The four-space model:

```
Input Space 1: Momentum-following strategy
  - Elements: trend detection, breakout signals, position scaling

Input Space 2: Liquidity provision strategy
  - Elements: LP position management, rebalancing, fee collection

Generic Space: Both involve timing entries/exits relative to price movement

Blended Space: Use momentum signals to time LP entries/exits
  - Emergent property: momentum reversal signals predict optimal
    rebalancing windows — a connection neither input space contains
```

**Deferred to v2** because it requires formalizing the four-space model for market strategies, which is research-grade work. The v1 combinational mode achieves most of the practical benefit with simpler implementation.

### Koestler's Bisociation

Arthur Koestler (1964) described creativity as the collision of two normally incompatible "matrices" (frames of thought). He explicitly noted that bisociative breakthroughs often occur during dreams and states of relaxation [KOESTLER-1964]. The Golem's analog: deliberately colliding strategies from incompatible market frames.

```
Bisociation Session:

Frame 1: Bear market defensive hedging
  - Elements: protective puts, correlated short positions, capital preservation

Frame 2: Bull market leveraged yield farming
  - Elements: recursive lending, concentrated LP positions, yield compounding

These frames are normally incompatible. Collide them:
1. What strategy element from Frame 1 could protect Frame 2's downside?
2. What strategy element from Frame 2 could enhance Frame 1's upside?
3. What entirely new strategy emerges from the collision?
```

### Deduplication and Diversity

Si, Yang, and Hashimoto (2024) found that LLM-generated research ideas were statistically more novel than human expert ideas (p < 0.05), but ~95% were duplicates of each other [SI-2024]. The Golem must actively manage creative diversity.

**Deduplication**: Before accepting a creative output, compare it against all existing strategy hypotheses in the Grimoire using semantic similarity (embedding cosine similarity > 0.85 = duplicate). Duplicates are discarded.

**Diversity promotion**: Franceschelli and Musolesi (2024) showed that RLHF reduces LLM output diversity through attractor states [FRANCESCHELLI-MUSOLESI-2024]. Counter this by:

- Varying prompts across creative sessions (different seed elements, different framing)
- Using higher LLM temperature for creative sessions (0.8–1.0 vs. 0.3–0.5 for analytical)
- Explicitly requesting "ideas that contradict your previous suggestions"
- Rotating between Boden's three modes across dream cycles

---

## Mortality-Phase Creativity Modulation

Behavioral phase shifts affect which Boden creativity mode is emphasized during the REM-like phase. As the Golem ages and mortality pressure increases, creative energy redirects from exploration toward consolidation and legacy:

| Behavioral Phase | Primary Mode     | Secondary Mode | Creative Emphasis                                                                                                 |
| ---------------- | ---------------- | -------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Thriving**     | Transformational | Exploratory    | Bold hypothesis generation — challenge assumptions, reframe costs as opportunities, seek novel strategy paradigms |
| **Stable**       | Exploratory      | Combinational  | Boundary mapping — where do current strategies break? What parameter ranges are untested?                         |
| **Conservation** | Combinational    | Exploratory    | Defensive synthesis — combine proven strategies into more robust composites, find redundancies to prune           |
| **Declining**    | Combinational    | —              | Legacy-oriented — synthesize validated patterns into transferable heuristics for the death testament              |
| **Terminal**     | —                | —              | No imagination (all resources to Thanatopsis Protocol)                                                            |

The creative mode selection is not exclusive — all modes run in every dream cycle (at [HARDENED] tier). The table specifies _allocation weights_: a Thriving Golem spends ~50% of creative budget on transformational, ~30% exploratory, ~20% combinational. A Conservation Golem inverts this.

> **Cross-ref**: `../02-mortality/01-architecture.md` (behavioral phases), `../02-mortality/04-economic-mortality.md` (dream compute in burn rate)

## Daimon-Mood Creativity Modulation

The Golem's current mood state (PAD vector) biases dream content during the REM-like phase. This implements mood-congruent dream generation — the emotional analog of what biological research shows for REM sleep content.

| Mood State         | PAD Profile     | Dream Mode Bias                                                                  |
| ------------------ | --------------- | -------------------------------------------------------------------------------- |
| Positive/confident | High P + High D | More creative exploration, higher exploration temperature (τ), bolder hypotheses |
| Anxious/threatened | High A + Low P  | More threat simulation, more adversarial scenarios, higher severity assumptions  |
| Depleted/withdrawn | Low A + Low D   | Conservative consolidation, replay of known-good strategies, lower temperature   |
| Neutral/stable     | Mid-range       | Balanced allocation across modes                                                 |

**Minimum creative floor**: Regardless of mood state, at least 20% of the REM-like budget is allocated to creative/exploratory operations. Without this floor, a Golem in sustained negative affect loses its ability to generate novel strategies precisely when escape strategies are most needed. This constraint overrides the mood-driven bias.

> **Cross-ref**: `../03-daimon/03-behavior.md` (mood-modulated dream content, minimum creative allocation), `../03-daimon/00-overview.md` (Dream Engine as interaction partner)

---

## 3. Scenario Generation

Beyond counterfactuals on past trades and creative recombination of strategies, the imagination engine generates full synthetic market scenarios — coherent narratives of market events that have not occurred but plausibly could.

### Scenario Structure

```rust
pub struct SyntheticScenario {
    pub id: String,
    pub source: ScenarioSource,
    pub narrative: String,
    pub market_conditions: ScenarioMarketConditions,
    pub event_sequence: Vec<ScenarioEvent>,
    pub challenged_strategy: String,
    pub expected_outcome: String,
    pub proposed_adaptation: String,
    pub plausibility: f64,
    pub confidence: f64,
}

pub enum ScenarioSource {
    Counterfactual,
    Creative,
    ThreatSim,
    Stochastic,
}

pub struct ScenarioMarketConditions {
    pub regime: MarketRegime,
    pub volatility: VolatilityLevel,   // Low, Medium, High, Extreme
    pub liquidity: LiquidityLevel,     // Deep, Normal, Thin, Crisis
    pub gas_price: GasPriceLevel,      // Low, Normal, Elevated, Spike
    pub correlations: CorrelationState, // Normal, Stressed, Broken
}
```

### Generation Methods

**Method 1: Historical recombination** — Select elements from different historical episodes and combine them into a novel scenario:

```
Construct a plausible market scenario by combining:
- The liquidity conditions from epoch A [low TVL, wide spreads]
- The volatility pattern from epoch B [sudden spike after 48h calm]
- The gas dynamics from epoch C [sustained high gas, 3 days]

Requirements:
- The combination must be internally consistent
- Identify what real-world event could trigger this combination
- Assess how the current PLAYBOOK.md would perform
```

**Method 2: Extrapolation** — Take a current trend and extrapolate to extreme conditions:

```
Current observation: Gas fees have been rising 5% per day for 7 days.

Extrapolate:
- What happens if this continues for 30 days?
- At what point does the Golem's strategy become unprofitable?
- What early warning threshold should be added to PLAYBOOK.md?
- What alternative strategy becomes optimal at that threshold?
```

**Method 3: Stochastic activation** — Randomly combine memory elements (Hobson-McCarley activation-synthesis):

```
Three randomly selected memory elements:
1. [Oracle price deviation incident from 47 days ago]
2. [Successful arbitrage on a newly launched pool from 12 days ago]
3. [PLAYBOOK.md heuristic about gas estimation from initial configuration]

Synthesize a coherent scenario connecting these three elements.
What does this scenario teach about a risk the Golem has not considered?
```

---

## Novel Hypotheses as Testable Predictions

Every imagination output that proposes a causal claim or strategy hypothesis is registered as a formal Creative prediction in the prediction subsystem. This is the prediction-engine integration: dream-generated ideas are not free-floating suggestions -- they are testable claims with specific resolution checkpoints, resolved against on-chain reality on the same schedule as any waking prediction.

For example, the counterfactual "IF staking withdrawal queue > 1000, THEN Aerodrome ETH pool fees spike 200bps within 4h" becomes a Creative prediction with:
- A resolution condition (withdrawal queue exceeds 1000)
- A predicted outcome (fee spike of 200bps)
- A time horizon (4 hours)
- A confidence level (0.10-0.20, reflecting dream origin)

When the resolution condition triggers in live markets, the prediction resolves like any other. If confirmed, confidence increments. If refuted, it decrements. After 3+ independent confirmations, the hypothesis can be promoted to an environmental model. This is the same validation pipeline used for waking predictions, applied to dream-generated hypotheses. The prediction engine does not distinguish between waking and dreaming origins -- it only cares whether the prediction is confirmed by reality.

This means dream quality has a measurable, objective metric: the confirmation rate of dream-sourced predictions over time. A healthy rate is 3-10% of hypnagogia-sourced hypotheses confirmed in live trading. Below 1%, hypnagogia is producing noise. Above 15%, the creative temperature is probably too conservative (not exploring far enough from known patterns).

---

## Output: Strategy Hypotheses

All imagination outputs that propose new strategies are formalized as Strategy Hypotheses:

```rust
pub struct StrategyHypothesis {
    pub id: String,
    pub source: HypothesisSource,
    pub description: String,
    pub rationale: String,
    pub failure_conditions: String,
    pub validation_criterion: String,
    pub minimum_viable_test: String,
    pub confidence: f64,
    pub dream_cycle_origin: String,
    pub related_episodes: Vec<String>,
    pub deduplication_hash: String,

    pub validation_result: Option<ValidationResult>,
    pub validation_episodes: Option<Vec<String>>,
    pub promoted_to_playbook: Option<bool>,
}

pub enum HypothesisSource {
    Creativity(CreativityMode),
    Counterfactual,
    ThreatResponse,
}

pub enum ValidationResult {
    Confirmed,
    Refuted,
    Inconclusive,
}
```

**Confidence ladder for hypotheses**:

| Stage                   | Confidence | Trigger                                      |
| ----------------------- | ---------- | -------------------------------------------- |
| Dream-generated         | 0.1–0.2    | Output of imagination engine                 |
| Staged in Grimoire      | 0.2        | Passed deduplication and plausibility check  |
| Partially validated     | 0.3–0.5    | 1–3 live episodes consistent with hypothesis |
| Validated               | 0.5–0.7    | 5+ live episodes, statistically significant  |
| Promoted to PLAYBOOK.md | 0.7+       | Meets promotion criteria                     |

The conservative confidence ladder (Kumar et al.'s CQL principle [KUMAR-CQL-2020]) ensures that dream-generated strategies never override validated waking knowledge. A dream hypothesis at 0.2 cannot displace a live-validated heuristic at 0.7. Dreams suggest; the market validates.

---

## Anticipatory trajectories

Beyond counterfactuals on past trades and creative recombination of existing strategies, the imagination engine generates forward-looking trajectory predictions using causal graph traversal. This extends the forward replay concept from `02-replay.md` into structured multi-hop prediction.

The Grimoire's causal link store contains edges of the form "A causes B with confidence C and lag L ticks." The anticipatory trajectory generator performs 5-hop breadth-first search from the current market state, following the highest-confidence causal edges at each hop. At each node, it records the predicted state delta and the edge confidence.

Three scenario types are generated from this traversal:

1. **Regime continuation**: The current market regime persists for 20 ticks. The trajectory follows the most-traveled causal paths from similar historical states. This produces a baseline "what happens if nothing changes" prediction.

2. **Regime switch**: At the root node, force a transition to each alternative regime the Golem has experienced. Follow the causal graph from each alternative starting point. This produces "what if the market shifts" predictions that expose position vulnerabilities.

3. **Lagged edge fire**: Identify the highest-confidence causal edge in the graph that has not fired within its expected lag window. Assume it fires now. Follow the cascade forward. This catches "overdue" causal relationships that the market has not yet resolved.

```rust
pub struct AnticipatorTrajectory {
    pub hypothesis: String,
    pub steps: Vec<TrajectoryStep>,
    pub terminal_state: PredictedMarketState,
    pub strategy_fitness: f64,
    pub confidence: f64,
}

pub struct TrajectoryStep {
    pub causal_edge_id: String,
    pub predicted_state_delta: String,
    pub hop_confidence: f64,
}

pub struct PredictedMarketState {
    pub regime: String,
    pub price_deltas: Vec<(String, f64)>,
    pub liquidity_condition: String,
    pub gas_estimate: f64,
}
```

Trajectory outputs feed into three downstream consumers:

- **NREM Phase 1**: Trajectories identify which open positions need attention during credit assignment.
- **REM Phase 2**: Trajectories with low `strategy_fitness` become seeds for counterfactual exploration ("what strategy would survive this trajectory?").
- **Threat simulation**: Trajectories terminating in adverse states feed the ThreatSimulator's scenario generator.

> **Cross-ref**: `01-architecture.md` (anticipatory trajectories overview), `../04-memory/01-grimoire.md` (causal link store), Diba-Buzsaki 2007 (forward replay)

---

## Citation Summary

| Citation Key                  | Source                                                                                      |
| ----------------------------- | ------------------------------------------------------------------------------------------- |
| [PEARL-2000]                  | Pearl. _Causality._ Cambridge, 2000/2009.                                                   |
| [ANDRYCHOWICZ-2017]           | Andrychowicz et al. "Hindsight Experience Replay." _NeurIPS_, 2017.                         |
| [ZINKEVICH-2007]              | Zinkevich et al. "Regret Minimization in Games." _NeurIPS_, 2007.                           |
| [BYRNE-2005]                  | Byrne. _The Rational Imagination._ MIT Press, 2005.                                         |
| [EPSTUDE-ROESE-2008]          | Epstude & Roese. "Functional Theory of Counterfactual Thinking." _PSPB_, 2008.              |
| [BODEN-2004]                  | Boden. _The Creative Mind._ Routledge, 2004.                                                |
| [FAUCONNIER-TURNER-2002]      | Fauconnier & Turner. _The Way We Think._ Basic Books, 2002.                                 |
| [KOESTLER-1964]               | Koestler. _The Act of Creation._ Hutchinson, 1964.                                          |
| [SI-2024]                     | Si, Yang, & Hashimoto. "Can LLMs Generate Novel Research Ideas?" _ICLR_, 2025.              |
| [FRANCESCHELLI-MUSOLESI-2024] | Franceschelli & Musolesi. "Creativity and Machine Learning." _ACM Computing Surveys_, 2024. |
| [KUMAR-CQL-2020]              | Kumar et al. "Conservative Q-Learning for Offline RL." _NeurIPS_, 2020.                     |
