# Pi-native composition patterns [SPEC]

> **Cross-ref**: [00-agents-overview.md](00-agents-overview.md) (architecture overview for archetypes and tool profiles), [03-delegation.md](03-delegation.md) (tool resolution pipeline and delegation DAG with max depth 3), [09-golem-agents.md](09-golem-agents.md) (three autonomous core archetypes that originate all delegations)

> **Reader orientation:** This document specifies all 18 composition patterns through which Golem (mortal autonomous agent) archetypes combine to perform complex DeFi operations: autonomous LP management, research-to-trade pipelines, cross-chain arbitrage, vault management, self-improvement loops, yield discovery, delta-neutral hedging, lending health management, and more. It belongs to Section 19 (Agents & Skills). The Heartbeat (9-step decision cycle) loop is the master pattern from which all archetype delegations originate. The cybernetic triple loop (single-tick, double-loop Reflexion, and meta-loop PLAYBOOK.md evolution) is the architectural backbone. See `prd2/shared/glossary.md` for full term definitions.

---

## The heartbeat as master composition pattern

The golem's heartbeat loop is the root of all composition. Every pattern below runs inside the heartbeat's EXECUTE phase. The golem reads PLAYBOOK.md, evaluates active strategies, and delegates to specialist archetypes as needed.

```text
golem-instance heartbeat
  |
  +-- Tick N: DCA strategy triggers -> trade-executor -> safety-guardian -> execute
  |
  +-- Tick N+1: Delta drift detected -> liquidity-manager -> safety-guardian -> rebalance
  |
  +-- Tick N+5: RSI oversold -> lp-strategist -> risk-assessor -> liquidity-manager -> enter
  |
  +-- Tick N+50: Curator trigger -> memory-consolidator -> PLAYBOOK.md evolution
```

All delegations flow through the two-layer tool model: the Pi-facing tools (`preview_action`, `commit_action`) backed by concrete `golem-tools` implementations. The `bardo-permits` extension governs the preview-to-commit lifecycle. The `bardo-safety` extension enforces phase-based and PolicyCage constraints.

---

## Pattern 1: Autonomous LP management

```text
Golem heartbeat -> LP strategy triggers

1. lp-strategist
   - Analyzes token pair characteristics
   - Delegates to pool-researcher for pool data
   - Delegates to risk-assessor for IL risk evaluation
   - Returns: recommended range, fee tier, rebalance strategy

2. liquidity-manager
   - Checks current position status
   - If rebalance needed:
     a. Collect accrued fees -> preview_action -> commit_action
     b. Remove old position -> preview_action -> commit_action
     c. Add new position at recommended range -> preview_action -> commit_action

3. golem-instance
   - Stores outcome as episode via update_directive
   - Records P&L impact in Grimoire
```

---

## Pattern 2: Research-to-trade pipeline

```text
Golem heartbeat -> opportunity detected

1. token-analyst -> Researches target: liquidity, volume, risk factors
2. pool-researcher -> Finds best pool, analyzes depth
3. risk-assessor -> Evaluates: slippage risk, token risk, pool risk
4. trade-executor (only if risk-assessor approves)
   -> preview_action -> commit_action
5. If risk-assessor VETOES: pipeline halts, episode recorded with veto reason
```

---

## Pattern 3: Cross-chain arbitrage

```text
Golem heartbeat -> price discrepancy detected

1. opportunity-scanner -> Compares prices across chains
2. risk-assessor -> Evaluates bridge risk, execution speed, slippage
3. cross-chain-executor -> Buy cheap, bridge, sell expensive
   (each tx through preview_action -> commit_action)
4. golem-instance -> Records profit/loss as episode
```

---

## Pattern 4: Agent-managed vault

```text
Golem heartbeat -> vault rebalance trigger

1. vault-strategist
   - Queries vault rankings and performance metrics
   - Evaluates risk scores and APY
   - Delegates to risk-assessor for independent validation

2. vault-manager
   - Verifies ERC-8004 identity
   - Checks tier limits (deposit caps based on reputation)
   - Simulates deposit via preview_action
   - Commits via commit_action

3. vault-strategist (post-execution)
   - Reports vault share balance, current APY, risk score
```

**Academic frameworks** integrated into vault decisions:

| Framework                           | Application                                                          |
| ----------------------------------- | -------------------------------------------------------------------- |
| Campbell-Baggiani optimal fees      | Dynamic fee tier selection based on volatility regime                |
| Singh LVR-theta floor               | Minimum fee threshold to remain profitable against adverse selection |
| Hasbrouck-Rivera-Saleh execution cost| Covered call LP analysis for range order profitability               |
| Bergault-Bieber-Sanchez-Betancourt  | Optimal exit timing under mean-reverting spread dynamics             |
| Devorsetz-Herlihy convex optimization| Defensive rebalancing across correlated positions                    |
| Trotti et al. JIT liquidity         | Just-in-time provision around predicted large swaps                  |
| Aqsha et al. Stackelberg equilibrium| Strategic fee setting in competitive LP environments                 |

**Hierarchical planning cadences**:

| Cadence       | Archetype         | Action                                                 |
| ------------- | ----------------- | ------------------------------------------------------ |
| Weekly        | vault-strategist  | Sets allocation targets across adapters                |
| Daily         | vault-allocator   | Adjusts positions toward targets based on market drift |
| Hourly        | liquidity-manager | Monitors ranges, fine-tunes tick boundaries            |
| Per-operation | trade-executor    | Executes individual trades through safety pipeline     |

---

## Pattern 5: Self-improvement loop

```text
(Autonomous -- runs as part of golem heartbeat meta loop)

1. pnl-analyst -> Computes performance metrics for vault positions
2. strategy-optimizer
   -> Compares predicted vs actual for recent executions
   -> Clusters underperforming executions by common features
   -> Runs Bayesian optimization on strategy parameters
   -> Bounds changes to 10% max per iteration (Maxwell governor damping)
   -> Delegates to risk-assessor to validate parameter changes
3. vault-strategist -> Receives tuned parameters, adjusts recommendations
4. vault-manager (next rebalance) -> Executes with tuned parameters
5. golem-instance -> Stores outcome as episode
6. memory-consolidator (every 50 ticks) -> Integrates into PLAYBOOK.md
```

Verbal reinforcement signals (FinCon-style) preserve causal reasoning more effectively than scalar rewards, maintaining the chain of strategic logic across optimization cycles.

---

## Pattern 6: Yield discovery and rotation

```text
Golem heartbeat -> yield scan trigger

1. yield-scout
   - Aggregates yields from DefiLlama, Morpho, Pendle, Aavescan, Lido
   - Computes risk-adjusted scores
   - Delegates to risk-assessor for strategy risk evaluation

2. lp-strategist (for Uniswap LP opportunities)
   - Designs range, fee tier, rebalance plan
   - Uses classify_regime to detect market state

3. trade-executor (if token conversion needed)
   -> preview_action -> commit_action

4. liquidity-manager (for LP entry)
   -> preview_action -> commit_action

5. strategy-optimizer (background, post-execution)
   -> Records outcome, stores successful strategy for retrieval
```

---

## Pattern 7: Real-time monitoring and alert response

```text
(Event-driven -- triggered by GolemEvent on event bus)

1. bardo-heartbeat extension detects anomaly via deterministic probes
   -> Emits GolemEvent with severity

2. On anomaly detection:
   a. Price deviation on existing position:
      -> vault-strategist for rebalance recommendation
      -> vault-manager executes via ActionPermit pipeline

   b. Large trade detected (whale alert):
      -> risk-assessor for impact evaluation
      -> portfolio-analyst for exposure check

   c. TVL drop on pool with open position:
      -> risk-assessor for liquidity risk evaluation
      -> If critical: liquidity-manager removes position

3. vault-watchdog (active for Warden-enhanced vaults; Warden is optional, deferred)
   -> Monitors pending proxy operations
   -> Exercises cancel authority on anomalous announcements
```

---

## Pattern 8: Token launch pipeline

```text
Operator triggers via steer() or deploy-agent-token workflow

1. token-deployer -> Creates token contract, configures CCA supply schedule
   -> Deploys anti-snipe hooks and am-AMM configuration
   -> All through ActionPermit pipeline
2. token-deployer (CCA phase) -> Deploys CCA auction, monitors bids
   -> Block-by-block bid processing at uniform clearing prices
3. token-deployer (pool seeding)
   -> LiquidityLauncher migrates tokens to V4 pool
   -> LaunchFeeHook provides MEV protection during critical early trading
   -> Delegates to liquidity-manager for LP operations
4. golem-instance -> Records outcome, reports post-launch metrics
```

---

## Pattern 9: Order flow intelligence

```text
Golem heartbeat -> LP profitability check trigger

1. pool-researcher -> VPIN, LVR, fee/LVR ratio, markout profile
2. risk-assessor -> Order flow toxicity risk evaluation
3. lp-strategist -> Recommends exit/stay/widen based on fee/LVR ratio
4. portfolio-analyst -> Reports P&L with LVR-adjusted IL
```

**VPIN classification thresholds**: low (<0.3), medium (0.3-0.7), high (>0.7).

**fee/LVR action thresholds**:

| fee/LVR | Assessment                        | Action                                |
| ------- | --------------------------------- | ------------------------------------- |
| < 1.0   | Losing money to adverse selection | Widen range, decrease lambda, or exit |
| 1.0-1.5 | Marginally profitable             | Maintain current parameters           |
| > 1.5   | Comfortably profitable            | Tighten range for more fee income     |

---

## Pattern 10: Yield compounding

```text
(Autonomous -- runs on configurable schedule within heartbeat)

1. liquidity-manager (periodic check)
   - Enumerates all LP positions with accrued fees
   - For each: accumulated_fees x (1 - swap_slippage) vs gas_cost x multiplier

2. liquidity-manager (compound execution)
   a. V4 pool with auto-compound hook: native hook compounding (zero gas)
   b. V4 pool without hook: batch via flash accounting (single settlement)
   c. V3 pool: collect -> swap -> add liquidity (3 txs)
   - Each through ActionPermit pipeline

3. treasury-manager -> Records each compound as taxable event
4. strategy-optimizer -> Optimizes compound_multiplier parameter
```

---

## Pattern 11: Tax-loss harvesting

```text
Golem heartbeat -> quarterly tax harvest trigger

1. pnl-analyst -> Computes unrealized P&L, identifies losing positions, YTD gains
2. treasury-manager -> Ranks losses by harvest benefit, plans re-entry
3. liquidity-manager -> Removes losing positions via ActionPermit
   -> Re-enters with different parameters for economic substance
4. pnl-analyst -> Reports: capital losses harvested, estimated tax savings
```

---

## Pattern 12: Cross-vault coordination

```text
(Autonomous -- runs when golem manages multiple vaults)

1. portfolio-analyst -> Aggregates positions across all managed vaults
2. vault-strategist -> Runs Devorsetz-Herlihy convex optimization
   -> Subject to: per-vault PolicyCage, total TVL conservation,
      rebalance frequency limits, adapter exposure caps
3. vault-manager -> Executes in priority order:
   a. Sell operations first (reducing exposure)
   b. Lending adjustments second
   c. Buy operations last (new positions)
   -> Ordering minimizes cross-vault MEV
4. portfolio-analyst -> Verifies aggregate state
```

During market stress, switches from optimization to preservation mode.

---

## Pattern 13: Delta-neutral hedging

```text
(Event-driven -- triggered by delta drift on LP positions)

1. position-monitor -> Delta calculations for all LP positions
2. risk-assessor -> Evaluates hedge cost vs delta risk when threshold exceeded
3. trade-executor -> Hedges via perpetuals or spot through ActionPermit
4. portfolio-analyst -> Reports net delta, hedge cost, position P&L
```

**Regime-adaptive thresholds**:

| Regime          | Delta threshold | Trigger                                        |
| --------------- | --------------- | ---------------------------------------------- |
| High-volatility | +-8%            | Wider band reduces hedge churn during whipsaws |
| Default         | +-5%            | Standard operating range                       |
| Low-volatility  | +-3%            | Tighter band captures drift in calm markets    |

---

## Pattern 14: Lending health management

```text
Golem heartbeat -> health factor drift detected

1. lending-monitor -> Reads health factors across Aave, Compound, Morpho positions
2. lending-manager -> Evaluates collateral ratios and rebalance options
3. risk-assessor -> Validates rebalance plan (liquidation risk, gas cost)
4. lending-manager -> Executes collateral top-up or debt repayment via ActionPermit
5. golem-instance -> Records health factor history and rebalance outcome
```

---

## Pattern 15: Staking lifecycle management

```text
Golem heartbeat -> staking event triggers (reward claim, unstake window, validator change)

1. staking-manager -> Monitors validator performance, reward accrual, unbonding queues
2. yield-scout -> Compares restaking yields across EigenLayer, Symbiotic, Karak
3. risk-assessor -> Evaluates slashing risk, lock-up cost, opportunity cost
4. staking-manager -> Claims rewards, rotates validators, or initiates unstaking via ActionPermit
```

---

## Pattern 16: Derivatives hedging with TA signals

```text
Golem heartbeat -> technical signal fires

1. technical-analyst -> Generates signals: RSI divergence, volatility compression, support/resistance
2. derivatives-trader -> Constructs options spread or perp position matching signal
3. risk-assessor -> Validates max notional, margin requirements, liquidation price
4. derivatives-trader -> Executes via ActionPermit (GMX, Aevo, Lyra)
5. portfolio-analyst -> Reports net Greeks, margin utilization, P&L attribution
```

---

## Pattern 17: Full-stack DeFi position management

```text
Golem heartbeat -> portfolio rebalance trigger

1. portfolio-analyst -> Aggregates all positions: LP, lending, staking, derivatives
2. lending-monitor -> Flags any health factors below threshold
3. technical-analyst -> Provides regime context (trending vs ranging, vol regime)
4. lp-strategist + lending-manager + staking-manager -> Coordinate rebalance
5. risk-assessor -> Validates combined portfolio risk (correlation, concentration)
6. Respective executors -> Execute via ActionPermits
```

---

## Pattern 18: Lending-to-LP capital rotation

```text
(Event-driven -- triggered by yield differential exceeding threshold)

1. lending-monitor -> Detects lending yield falling below LP fee APR
2. yield-scout -> Confirms LP opportunity with superior risk-adjusted yield
3. lending-manager -> Withdraws collateral from lending protocol via ActionPermit
4. liquidity-manager -> Deploys capital to LP position via ActionPermit
5. position-monitor -> Begins tracking new LP position
```

---

## The cybernetic triple loop

The golem-instance composes archetypes at three timescales:

```text
SINGLE LOOP (per-tick)
  Scheduler -> Probe -> Escalation Gate -> LLM Decision -> Archetype Delegation
  Memory middleware adjusts tool parameters. Wiener feedback.

DOUBLE LOOP (per-execution)
  After each non-suppressed tick: Reflexion pattern
  Update PLAYBOOK.md heuristics. Argyris double-loop learning.

META LOOP (every 50 ticks)
  memory-consolidator restructures PLAYBOOK.md itself
  Second-order cybernetics: the system observes itself
```

This triple-loop structure is what makes golems different from static agent orchestration. The composition patterns above operate within the single loop. The double loop evolves which patterns to use and when. The meta loop evolves the strategic framework that guides all decisions.

---
