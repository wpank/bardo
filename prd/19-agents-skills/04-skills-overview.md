# Pi interaction surfaces [SPEC]

> **Cross-ref**: [05-skill-categories.md](05-skill-categories.md) (full inventory of 68 interaction surfaces across 13 categories), [06-skill-definitions.md](06-skill-definitions.md) (Pi workflow definition format and trigger types), [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Pi extension loading and tool registration)

> **Reader orientation:** This document specifies the interaction surfaces through which operators, depositors, chat platforms, and other systems communicate with a running Golem (mortal autonomous agent). It belongs to Section 19 (Agents & Skills) and covers operator directives (steer/followUp), chat surfaces, API endpoints, heartbeat-driven triggers, and Pi Skills -- containers for dynamic capability loading that let a Golem evolve its strategy mid-session without restart. The Heartbeat (9-step decision cycle) is the primary self-generated interaction surface. See `prd2/shared/glossary.md` for full term definitions.

---

## How golems interact with the world

Golems are autonomous. They do not wait for slash commands. They run a heartbeat loop and make decisions based on PLAYBOOK.md, STRATEGY.md, and market state. But they still need interfaces -- operators need to give directives, depositors need to query status, and other systems need to trigger workflows.

These interfaces are called **interaction surfaces**. They are the input/output boundaries of the Pi runtime.

```text
Interaction Surface (input)
  |
  v
Pi Runtime (golem session)
  |  - Extensions process the input
  |  - Heartbeat FSM routes to appropriate state
  |  - Archetype determines which tools to use
  |
  v
Tool Adapter Layer (golem-tools crate)
  |  - On-chain reads/writes
  |  - Safety middleware
  |
  v
Uniswap Protocol + ERC-4626 Vaults + ERC-8004 Identity
```

---

## Surface types

### 1. Operator directives (steer/followUp)

Pi's `steer()` and `followUp()` primitives are the primary operator interface. `steer()` injects a system-level directive mid-session without waiting for the current turn to complete. `followUp()` queues a message for delivery at the next DECIDING state.

**When to use steer()**: Health factor breach, emergency halt, force liquidation, budget override. Anything that cannot wait for the next tick.

**When to use followUp()**: Strategy adjustments, allocation guidance, risk tolerance changes. Anything that should inform the next decision but does not require interrupting the current one.

```typescript
// Operator intervenes via app UI
await golem.steer("Reduce ETH exposure to 30% of portfolio by EOD");

// Non-urgent guidance
await golem.followUp(
  "Consider rotating into stablecoin LPs -- vol looks elevated",
);
```

### 2. Chat surfaces (Discord, Telegram, web)

The `bardo-social` extension routes messages from chat platforms to the Pi session. Each platform has a transport adapter that converts platform-specific message formats to Pi session inputs.

| Surface    | Transport         | Capabilities                          |
| ---------- | ----------------- | ------------------------------------- |
| Discord    | Bot + webhooks    | Text, embeds, reactions, slash cmds   |
| Telegram   | Bot API + webhooks| Text, inline keyboards, callbacks     |
| Web chat   | WebSocket + REST  | Real-time streaming, rich formatting  |
| CLI        | stdin/stdout      | Text, TUI rendering                   |

Chat messages arrive as `followUp()` calls. The golem processes them at the next DECIDING state, not immediately. This prevents chat messages from interrupting active execution.

### 3. API surface (RPC mode)

Pi's RPC mode exposes the golem as an HTTP service. External systems send JSON-RPC requests to query state, trigger actions, or subscribe to events.

```text
POST /rpc
{
  "method": "query_state",
  "params": { "kind": "positions" }
}
```

Used by: portal dashboard, monitoring infrastructure, cross-golem coordination (`bardo-clade`), external A2A clients.

### 4. Scheduled triggers (heartbeat)

The heartbeat is the primary interaction surface. It is self-generated: the Adaptive Clock fires theta-frequency ticks on a regime-dependent interval (30-120s), with faster gamma ticks (5-15s) for perception and slower delta ticks (5-30min) for maintenance. Each theta tick is an input to the Pi session, processed through the PROBE -> ESCALATE -> DECIDE -> EXECUTE -> REFLECT pipeline.

Most ticks (80%) are suppressed at PROBE (T0, no LLM cost). The heartbeat is the golem talking to itself.

### 5. Event-driven triggers (market events)

The `bardo-heartbeat` extension also fires on external events: price alerts, liquidation warnings, health factor breaches, governance proposals. These arrive as `GolemEvent` objects on the event bus and trigger an immediate tick, bypassing the scheduled interval.

---

## Interaction patterns

### Operator -> golem

| Pattern               | Pi primitive    | Latency | Use case                              |
| --------------------- | --------------- | ------- | ------------------------------------- |
| Emergency directive   | `steer()`       | <2s     | Health factor breach, force halt      |
| Strategy guidance     | `followUp()`    | Next tick| Allocation changes, risk adjustments  |
| Status query          | RPC             | <500ms  | Portfolio state, health check         |
| Configuration change  | Config reload   | Next boot| Strategy file update, profile change  |

### Golem -> operator

| Pattern               | Channel          | Trigger                               |
| --------------------- | ---------------- | ------------------------------------- |
| Execution report      | Discord/Telegram | After every non-trivial execution     |
| Health alert          | Discord/Telegram | Mortality threshold breach            |
| Daily digest          | Email/Discord    | Scheduled summary of activity         |
| Position alert        | Discord/Telegram | LP out-of-range, IL threshold breach  |

### Golem -> golem (intra-Clade, a group of related Golems sharing knowledge)

| Pattern               | Mechanism        | Use case                              |
| --------------------- | ---------------- | ------------------------------------- |
| Grimoire sync         | REST (P2P)       | Knowledge sharing between siblings    |
| Coordination event    | `bardo-clade`    | Multi-vault rebalancing               |
| Intelligence sale     | Oracle L3 / x402 | Sleepwalker publishes, traders buy    |

---

## Error handling

All interaction surfaces follow a structured error template:

```text
[WHAT HAPPENED]
  Transaction reverted: ERC20 insufficient allowance

[ARE FUNDS SAFE?]
  Yes -- transaction was simulated before broadcast and reverted in simulation.
  No tokens left your wallet.

[WHAT TO DO]
  1. Check approval status via query_state
  2. Re-provision Permit2 allowance
  3. Retry the original operation
```

**Progressive disclosure**:
- **Default**: One-line summary ("Swap failed: insufficient allowance")
- **On request**: Full context (the structured template above)
- **Debug**: Raw hex data, revert reason bytes, gas trace

---

## Sandbox mode

New golems start in sandbox mode -- a local Anvil fork that mirrors mainnet state but uses test ETH:

- Same tools, archetypes, and Pi runtime -- no sandbox-specific code paths
- Different RPC target -- Anvil fork instead of mainnet/Base
- Forked state -- real pool data, real token balances (impersonated)
- Free gas -- unlimited test ETH
- No real funds at risk

**Entering sandbox**: Default for new golems. Explicit `--sandbox` flag.

**Exiting sandbox**: Explicit `--live` flag switches RPC target to production. Requires operator confirmation.

---

## Pi Skills for dynamic capability loading

### What Pi Skills are

A Pi Skill is not a tool. A tool is a function the LLM can call. A skill is a container: it holds a description, a set of trigger terms, a bundle of tools, and decision-framework instructions. The skill sits dormant until the LLM's reasoning matches a trigger term, at which point the full skill content loads into context and its tools become available.

The distinction matters for cost. A Golem with 6 registered skills pays ~300 tokens for their descriptions (50 tokens each). The full content of all 6 skills would be ~6,000 tokens. The skill system provides progressive disclosure: capability is always accessible, but only costs tokens when it's needed.

Static tool profiles (the v0 approach) load all tools at boot and never change. Pi Skills enable dynamic capability loading during a session. A Golem running a DCA strategy can organically discover lending loops through its reasoning and load the lending-loop skill mid-session -- without a restart, without operator intervention, without any change to the manifest.

Wang et al. (2023) demonstrated this pattern in Voyager, an embodied agent that builds an open-ended skill library through exploration. Bardo adapts the same principle: instead of a Minecraft agent discovering "how to mine diamonds," a Golem discovers "how to supply collateral to Morpho" -- and the discovery sticks in the Grimoire for future sessions.

### Skill lifecycle

```text
REGISTER ─────> DORMANT ─────> TRIGGER ─────> LOAD ─────> ACTIVE ─────> UNLOAD
   │               │               │              │            │            │
   │  session(start)│  description  │  LLM reason  │  full body │  tools     │  agent_end
   │  registers all │  in context   │  matches a   │  + tools   │  available │  skill tools
   │  skill YAMLs   │  (~50 tokens) │  trigger term│  injected  │  for calls │  removed
   │               │               │              │            │            │
   └───────────────┴───────────────┴──────────────┴────────────┴────────────┘
                                                                     │
                                                             description remains
                                                             (can re-trigger)
```

Six states:

1. **REGISTER**: Skill YAML registered with Pi at session creation. Parsed from `~/.bardo/skills/*/SKILL.md`.
2. **DORMANT**: Only the description sits in context. ~50 tokens per skill. No tools loaded.
3. **TRIGGER**: The LLM's reasoning mentions a trigger term (e.g., "lending", "health factor", "borrow"). Pi detects the match.
4. **LOAD**: Full skill content -- instructions, decision frameworks, risk heuristics -- injected into context. Skill tools become callable.
5. **ACTIVE**: The Golem can use the skill's tools for the current agent invocation.
6. **UNLOAD**: After `agent_end`, skill tools are removed from context. The description remains (can re-trigger on a future tick).

---

## Strategy skills

Each DeFi strategy archetype maps to a Pi Skill with trigger terms, tool subsets, and decision frameworks.

### spot-trading

```yaml
# ~/.bardo/skills/spot-trading/SKILL.md
---
name: spot-trading
description: "Spot token trading: swaps, DCA execution, limit orders, and market analysis for single-asset trades."
triggers:
  - swap
  - trade
  - buy
  - sell
  - DCA
  - dollar cost average
  - market order
  - limit order
  - token purchase
---

# Spot Trading Skill

## Available Tools
When this skill is active, the following adapters become available via preview_action/commit_action:
- execute_swap (Uniswap V3/V4 router)
- get_quote (multi-venue best price)
- simulate_price_impact
- get_token_price, get_token_info
- get_order_flow_metrics

## Decision Framework
1. ALWAYS check current price via search_context before any swap
2. Compare quotes across venues (V3, V4, UniswapX)
3. For amounts > 1% of pool depth, use TWAP execution (split across blocks)
4. For DCA: execute at the configured interval, don't try to time the market
5. Set slippage to min(strategy.maxSlippageBps, pool_depth_based_estimate)

## Risk Considerations
- Slippage scales non-linearly with trade size relative to pool depth
- Gas cost is fixed per trade — small trades may not be economical
- MEV risk: Base L2 sequencer provides private mempool (lower risk than L1)
```

### lending-loop

```yaml
# ~/.bardo/skills/lending-loop/SKILL.md
---
name: lending-loop
description: "Lending protocol management: Morpho/Aave supply, borrow, leverage loops, health factor monitoring, and deleverage operations."
triggers:
  - health factor
  - leverage
  - collateral ratio
  - lending loop
  - deleverage
  - borrow
  - supply
  - liquidation
  - Morpho
  - Aave
---

# Lending Loop Management

## Available Tools
- get_lending_position (health factor, utilization, APY)
- supply_collateral
- borrow_asset
- repay_debt
- calculate_health_factor
- simulate_leverage_change

## Decision Framework
1. NEVER increase leverage when health factor < 1.5
2. Target health factor: 1.8-2.5 range for stability
3. Deleverage trigger: HF drops below 1.4 OR HF drops >0.3 in one tick
4. Recursive loop: supply -> borrow -> supply cycles max 3 iterations
5. ALWAYS simulate the full loop before executing

## Risk Heuristics
- Health factor decays continuously as borrow APY accrues
- Flash crashes can move HF from safe to liquidation in seconds
- Deleverage requires gas — ensure gas partition has sufficient budget
- Over 70% of DeFi liquidations occur from gradual decay, not flash crashes [QIN-2021]
```

### concentrated-lp

```yaml
# ~/.bardo/skills/concentrated-lp/SKILL.md
---
name: concentrated-lp
description: "Concentrated liquidity provision on Uniswap V3/V4: range selection, rebalancing, fee collection, and impermanent loss management."
triggers:
  - liquidity
  - LP
  - tick range
  - rebalance
  - impermanent loss
  - fee tier
  - concentrated
  - pool position
---

# Concentrated LP Management

## Available Tools
- add_liquidity (V3/V4 with tick range)
- remove_liquidity
- collect_fees
- get_position (current range, liquidity, uncollected fees)
- get_tick_data (liquidity distribution)
- calculate_il (estimated impermanent loss)
- get_fee_earnings_history

## Decision Framework
1. Range width should balance fee concentration vs rebalance frequency
2. Wider range = fewer rebalances but lower fee APR
3. Narrower range = higher fee APR but frequent out-of-range events
4. Rebalance when price exits range by more than 5% [PLAYBOOK heuristic]
5. Always calculate net return: fees - IL - gas - opportunity cost

## Critical Data
- LVR (Loss vs Rebalancing) scales with sigma^2 [MILIONIS-2022]
- Across 17 V3 pools: total fees $199.3M vs total IL $260.1M [LOESCH-2021]
- LP provision is structurally negative-EV without edge in range selection
```

### vault-management

```yaml
# ~/.bardo/skills/vault-management/SKILL.md
---
name: vault-management
description: "ERC-4626 vault operations: deposit, withdraw, NAV tracking, fee management, share pricing, and vault strategy allocation."
triggers:
  - vault
  - deposit
  - withdraw
  - shares
  - NAV
  - vault strategy
  - ERC-4626
---

# Vault Management

## Available Tools
- vault_deposit, vault_withdraw
- vault_preview_deposit, vault_preview_withdraw
- get_vault_info (NAV, share price, fee schedule)
- get_vault_positions (underlying positions)
- get_vault_performance (historical returns, Sharpe)

## Decision Framework
1. Verify vault manager's reputation via ERC-8004 before deposit
2. Check vault's NAV vs share price for discount/premium
3. For deposits > 5% of vault TVL, check for front-running risk
4. Monitor vault's circuit breaker status before any operation
```

### risk-assessment

```yaml
# ~/.bardo/skills/risk-assessment/SKILL.md
---
name: risk-assessment
description: "Portfolio risk analysis: VaR computation, correlation assessment, drawdown analysis, stress testing, and risk-adjusted metrics."
triggers:
  - risk
  - drawdown
  - VaR
  - value at risk
  - correlation
  - stress test
  - Sharpe
  - Sortino
  - portfolio health
---

# Risk Assessment

## Available Tools
- get_portfolio_snapshot
- compute_portfolio_var
- compute_correlation_matrix
- compute_risk_metrics (Sharpe, Sortino, Calmar, max DD)
- simulate_stress_scenario

## Decision Framework
1. Run full portfolio assessment before any position change > 5% of NAV
2. Check correlation before adding positions — correlated assets amplify risk
3. Compare realized vs estimated VaR — persistent underestimate = recalibrate
4. Drawdown above 50% of max = automatic T2 escalation
```

### cross-chain-arbitrage

```yaml
# ~/.bardo/skills/cross-chain-arbitrage/SKILL.md
---
name: cross-chain-arbitrage
description: "Cross-chain opportunity detection and execution: bridge routing, price discrepancy identification, ERC-7683 intent submission, and settlement verification."
triggers:
  - cross-chain
  - bridge
  - arbitrage
  - price discrepancy
  - chain transfer
  - ERC-7683
  - intent
---

# Cross-Chain Arbitrage

## Available Tools
- get_bridgeable_tokens
- submit_cross_chain_intent (ERC-7683)
- get_token_price (multi-chain)
- simulate_price_impact (per-chain)
- check_bridge_status

## Decision Framework
1. Price discrepancy must exceed bridge fee + gas + slippage to be profitable
2. Use ERC-7683 intents — let fillers compete for best execution
3. Never bridge more than 10% of NAV in a single transaction
4. Monitor settlement: if bridge takes >30 minutes, escalate to operator
5. Account for destination chain gas costs in profitability calculation

## Risk Heuristics
- Bridge exploits are the largest category of DeFi loss by dollar value
- Verify bridge contract addresses against known-good registries
- Cross-chain arbitrage has execution risk: price may move during bridge latency
```

---

## Skill registration at boot

Skills are discovered and registered during `session(start)`. The `registerBardoSkills` function scans the skills directory, parses each YAML frontmatter, and registers the skill with Pi. Tool loading is deferred -- the `tools` callback only executes when the skill activates.

```typescript
import * as fs from "node:fs";
import * as path from "node:path";

function registerBardoSkills(
  session: GolemSession,
  config: GolemConfig,
): void {
  const skillsDir = path.join(config.dataDir, "skills");
  const skillFiles = fs
    .readdirSync(skillsDir, { recursive: true })
    .filter((f) => f.toString().endsWith("SKILL.md"));

  for (const file of skillFiles) {
    const content = fs.readFileSync(
      path.join(skillsDir, file.toString()),
      "utf-8",
    );
    const { frontmatter, body } = parseSkillYAML(content);

    session.registerSkill({
      name: frontmatter.name,
      description: frontmatter.description,
      triggers: frontmatter.triggers,
      instructions: body,
      // Tools are loaded dynamically when the skill activates.
      // This callback is NOT called at registration time.
      tools: () => loadSkillTools(frontmatter.name, config),
    });
  }
}
```

At boot, only descriptions enter context. 6 skills cost ~300 tokens. The full content of all 6 skills would be ~6,000 tokens. The difference is a 95% reduction in standing context cost for capability that may never be needed on a given tick.

---

## Dynamic tool loading and emergent strategy evolution

The real power of Pi Skills is that a Golem can evolve its strategy without restart. A Golem deployed as a DCA executor (spot-trading skill) can discover lending opportunities through its reasoning process and load the lending-loop skill mid-session.

Here's how it works in practice:

```text
Tick 1000: Golem running DCA strategy (spot-trading skill active)
  LLM: "I have 45,000 USDC sitting idle between DCA intervals.
        I should consider alternative yield strategies."

Tick 1001: LLM reasoning mentions "lending" -> lending-loop skill triggers
  lending-loop tools now available: supply_collateral, borrow_asset, etc.
  LLM: "I'll analyze Morpho supply rates for USDC."
  LLM calls: query_state({ kind: "lending_rates", protocol: "morpho", asset: "USDC" })

Tick 1002: LLM explores lending, decides to supply USDC to Morpho
  LLM calls: preview_action({ type: "supply", protocol: "morpho", asset: "USDC", amount: "45000000000" })
  -> This is EMERGENT STRATEGY EVOLUTION
  -> The Golem expanded from DCA-only to DCA + lending
  -> No restart, no operator intervention, no manifest change
  -> The Grimoire records this as a new strategy fragment
```

The Grimoire step is what separates this from a one-off exploration. The Golem stores the lending discovery as an episode with a reflection: "Supplying idle USDC to Morpho at 4.2% APY between DCA intervals generates additional yield with minimal risk (HF > 3.0)." On future sessions, this episode surfaces during memory retrieval, making the Golem more likely to repeat the pattern.

Over time, a population of Golems running the same archetype will independently discover different strategies. One finds lending loops. Another discovers LP opportunities. A third identifies cross-chain arbitrage. The Grimoire captures these discoveries, and sibling Golems share them through the `bardo-clade` knowledge sync. This is open-ended learning through a skill library -- the same pattern Wang et al. (2023) demonstrated in Voyager for Minecraft, applied to DeFi.

### Skill events

| Event | Payload | When |
| ---- | ---- | ---- |
| `skill:triggered` | `{ skillName, triggerTerm, tickNumber }` | A trigger term matched in LLM reasoning |
| `skill:loaded` | `{ skillName, toolCount, contextTokens }` | Skill content and tools injected into context |
| `skill:unloaded` | `{ skillName, wasUsed }` | Skill tools removed after agent_end |

The `wasUsed` field in `skill:unloaded` tracks whether the Golem actually called any of the skill's tools after loading. A skill that triggers frequently but is rarely used may have overly broad trigger terms. Operators can tune triggers based on this signal.

---
