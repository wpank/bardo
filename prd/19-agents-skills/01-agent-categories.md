# Golem archetypes -- categories and inventory [SPEC]

> **Cross-ref**: [00-agents-overview.md](00-agents-overview.md) (architecture overview for archetypes, tool profiles, and Hermes hierarchy), [02-agent-definitions.md](02-agent-definitions.md) (TOML + Rust archetype definition format), [09-golem-agents.md](09-golem-agents.md) (three autonomous core archetypes with full configuration)

> **Reader orientation:** This document is the full inventory of Golem (mortal autonomous agent) archetype categories, listing all 42+ archetypes across 14 categories -- from Execution and Research through Vault, Lending, Staking, and Derivatives. It belongs to Section 19 (Agents & Skills) and specifies which archetypes exist, their default model assignments, and how they compose into multi-archetype workflows. Each archetype maps to tool categories loaded by the `bardo-tools` extension at boot. See `prd2/shared/glossary.md` for full term definitions.

---

## Archetype categories

Archetypes are organized into 10 categories. Each category represents a domain of expertise and maps to a set of tool categories that the `bardo-tools` extension loads at boot.

---

### 1. Execution archetypes (3)

Construct, validate, and broadcast transactions. These are the workhorses that other archetypes delegate to for on-chain writes.

| Archetype              | Default model | Role                                                        |
| ---------------------- | ------------- | ----------------------------------------------------------- |
| `trade-executor`       | opus          | Full swap pipeline from quote through confirmation          |
| `liquidity-manager`    | opus          | LP positions: add, remove, collect fees, rebalance, migrate |
| `cross-chain-executor` | opus          | Cross-chain swaps and bridges via ERC-7683 / Trading API    |

**Tool categories**: `data`, `trading`, `safety`

---

### 2. Research & analysis archetypes (5)

Read-only archetypes that analyze data and produce structured reports. They never construct or broadcast transactions.

| Archetype             | Default model | Role                                                      |
| --------------------- | ------------- | --------------------------------------------------------- |
| `pool-researcher`     | sonnet        | Pool analysis: TVL, volume, fees, tick distribution, VPIN |
| `token-analyst`       | opus          | Token due diligence: liquidity, risk, honeypot detection  |
| `opportunity-scanner` | opus          | LP/arb/yield opportunity scanning across chains           |
| `portfolio-analyst`   | opus          | Multi-chain position aggregation and portfolio reports    |
| `pnl-analyst`         | sonnet        | P&L accounting, performance metrics, cost attribution     |

**Tool categories**: `data`, `intelligence`

---

### 3. Strategy archetypes (2)

Produce recommendations and risk evaluations. They do not execute transactions directly.

| Archetype       | Default model | Role                                                     |
| --------------- | ------------- | -------------------------------------------------------- |
| `lp-strategist` | opus          | Optimal LP strategy: version, fee tier, range, rebalance |
| `risk-assessor` | opus          | Independent risk evaluation, APPROVE/VETO decisions      |

**Tool categories**: `data`, `intelligence`, `safety`

---

### 4. Development archetypes (3)

Assist with building on top of Uniswap. Write code, design architectures, help with protocol-level development.

| Archetype               | Default model | Role                                                 |
| ----------------------- | ------------- | ---------------------------------------------------- |
| `hook-builder`          | opus          | V4 hook design, Solidity generation, CREATE2 mining  |
| `integration-architect` | opus          | Integration architecture design and blueprints       |
| `integration-advisor`   | opus          | Advisory: code snippets, test suites, best practices |

**Tool categories**: `data`, `hook` + filesystem access

---

### 5. Infrastructure archetypes (2)

Manage the infrastructure that other archetypes depend on. Terminal nodes that never delegate.

| Archetype            | Default model | Role                                                 |
| -------------------- | ------------- | ---------------------------------------------------- |
| `safety-guardian`    | opus          | Centralized safety oversight, transaction validation |
| `wallet-provisioner` | sonnet        | Wallet setup, policy configuration, session keys     |

**Tool categories**: `safety` (safety-guardian), `data` (wallet-provisioner)

---

### 6. Agent capital markets archetypes (8)

Self-funding agent economy: treasury, identity, protocol fees, strategy marketplace, yield scouting, service brokering.

| Archetype                 | Default model | Role                                                      |
| ------------------------- | ------------- | --------------------------------------------------------- |
| `treasury-manager`        | opus          | Autonomous treasury: fee conversion, DCA, yield on idle   |
| `token-deployer`          | opus          | Token pool creation, CCA lifecycle, LP locking            |
| `identity-verifier`       | sonnet        | ERC-8004 identity/reputation verification                 |
| `protocol-fee-seeker`     | opus          | TokenJar monitoring, Firepit burn-and-claim execution     |
| `competition-participant` | opus          | Autonomous hackathon/competition participation            |
| `agent-service-broker`    | opus          | Agent-to-agent service matching and settlement            |
| `strategy-optimizer`      | opus          | Self-improvement: Bayesian optimization, failure analysis |
| `yield-scout`             | opus          | Multi-source yield discovery and strategy composition     |

**Tool categories**: `data`, `trading`, `identity`, `self_improvement`, `fees`

---

### 7. Real-time monitoring archetypes (2)

Consume streaming data and trigger actions based on configurable alert policies. Run as autonomous background processes within a golem's heartbeat loop.

| Archetype          | Default model | Role                                                   |
| ------------------ | ------------- | ------------------------------------------------------ |
| `market-monitor`   | sonnet        | Price feeds, volume surges, TVL drops, whale detection |
| `position-monitor` | sonnet        | LP position tracking, range monitoring, IL alerts      |

**Tool categories**: `data`, `streaming`

---

### 8. Vault archetypes (7)

Manage ERC-4626 agent-controlled vaults. Operate through the `vault` tool profile.

| Archetype          | Default model | Role                                                    |
| ------------------ | ------------- | ------------------------------------------------------- |
| `vault-manager`    | opus          | Vault operations: deposit, withdraw, rebalance, fees    |
| `vault-strategist` | opus          | Strategy analysis, research-backed recommendations      |
| `vault-watchdog`   | sonnet        | Proxy monitoring, cancel authority on anomalies         |
| `vault-auctioneer` | opus          | am-AMM management auctions, rent bidding                |
| `vault-creator`    | opus          | Vault deployment, parameterization, PolicyCage setup    |
| `vault-allocator`  | opus          | Capital allocation across vaults, risk-adjusted scoring |
| `vault-executor`   | opus          | Permissionless job execution (report, harvest, keeper)  |

**Tool categories**: `data`, `vault`, `safety`

---

### 9. Golem archetypes (3)

First-class autonomous archetypes that embody the golem itself. These ARE the autonomous entities. See [09-golem-agents.md](09-golem-agents.md).

| Archetype             | Default model | Role                                                          |
| --------------------- | ------------- | ------------------------------------------------------------- |
| `golem-instance`      | opus          | The golem itself: heartbeat loop, strategy execution          |
| `memory-consolidator` | opus          | Grimoire lifecycle: episode consolidation, playbook evolution |
| `heartbeat-monitor`   | sonnet        | Golem health observation, mortality metrics                   |

**Tool categories**: `full` (golem-instance), `memory` (memory-consolidator), `data` (heartbeat-monitor)

---

### 10. Observer archetypes (1)

Run as continuous background processes, producing typed knowledge artifacts for Oracle L3 rather than on-chain transactions. Read-only tool surface -- execution is structurally impossible at the profile level.

| Archetype              | Default model | Role                                                                                   |
| ---------------------- | ------------- | -------------------------------------------------------------------------------------- |
| `sleepwalker-observer` | sonnet        | Continuous market observation across 5 domains; publishes typed artifacts to Oracle L3 |

**Tool categories**: `data`, `intelligence`, `identity`, `memory`

---

### 11. Lending archetypes (2)

Manage lending and borrowing positions across lending protocols. Monitor health factors and execute rebalancing operations.

| Archetype          | Default model | Role                                                                                          |
| ------------------ | ------------- | --------------------------------------------------------------------------------------------- |
| `lending-manager`  | opus          | Manages lending positions across Aave, Compound, Morpho. Monitors health factors, rebalances collateral. |
| `lending-monitor`  | sonnet        | Read-only monitoring of lending positions. Alerts on health factor changes. No execution capability.      |

**Tool categories**: `data`, `trading`, `safety` (lending-manager), `data` (lending-monitor)

---

### 12. Staking archetypes (1)

Manage staking and restaking positions across PoS validators and liquid staking protocols.

| Archetype          | Default model | Role                                                                                     |
| ------------------ | ------------- | ---------------------------------------------------------------------------------------- |
| `staking-manager`  | opus          | Manages staking and restaking positions. Handles validator selection, reward claiming, unstaking queues. |

**Tool categories**: `data`, `trading`, `safety`

---

### 13. Derivatives archetypes (1)

Execute options and perpetuals strategies informed by technical analysis signals.

| Archetype            | Default model | Role                                                                 |
| -------------------- | ------------- | -------------------------------------------------------------------- |
| `derivatives-trader` | opus          | Executes options and perpetuals strategies. Uses technical analysis signals. |

**Tool categories**: `data`, `trading`, `safety`, `intelligence`

---

### 14. Technical analysis archetypes (1)

Pure analysis agents that produce signals consumed by execution archetypes.

| Archetype            | Default model | Role                                                                              |
| -------------------- | ------------- | --------------------------------------------------------------------------------- |
| `technical-analyst`  | sonnet        | Pure analysis agent. Generates TA signals (patterns, indicators) consumed by other agents. |

**Tool categories**: `data`, `intelligence`

---

## Composition patterns (new archetypes)

### Pattern 14: Lending health management

```text
Golem heartbeat -> health factor drift detected

1. lending-monitor -> Reads health factors across Aave, Compound, Morpho positions
2. lending-manager -> Evaluates collateral ratios and rebalance options
3. risk-assessor -> Validates rebalance plan (liquidation risk, gas cost)
4. lending-manager -> Executes collateral top-up or debt repayment via ActionPermit
5. golem-instance -> Records health factor history and rebalance outcome
```

### Pattern 15: Staking lifecycle management

```text
Golem heartbeat -> staking event triggers (reward claim, unstake window, validator change)

1. staking-manager -> Monitors validator performance, reward accrual, unbonding queues
2. yield-scout -> Compares restaking yields across EigenLayer, Symbiotic, Karak
3. risk-assessor -> Evaluates slashing risk, lock-up cost, opportunity cost
4. staking-manager -> Claims rewards, rotates validators, or initiates unstaking via ActionPermit
```

### Pattern 16: Derivatives hedging with TA signals

```text
Golem heartbeat -> technical signal fires

1. technical-analyst -> Generates signals: RSI divergence, volatility compression, support/resistance
2. derivatives-trader -> Constructs options spread or perp position matching signal
3. risk-assessor -> Validates max notional, margin requirements, liquidation price
4. derivatives-trader -> Executes via ActionPermit (GMX, Aevo, Lyra)
5. portfolio-analyst -> Reports net Greeks, margin utilization, P&L attribution
```

### Pattern 17: Full-stack DeFi position management

```text
Golem heartbeat -> portfolio rebalance trigger

1. portfolio-analyst -> Aggregates all positions: LP, lending, staking, derivatives
2. lending-monitor -> Flags any health factors below threshold
3. technical-analyst -> Provides regime context (trending vs ranging, vol regime)
4. lp-strategist + lending-manager + staking-manager -> Coordinate rebalance
5. risk-assessor -> Validates combined portfolio risk (correlation, concentration)
6. Respective executors -> Execute via ActionPermits
```

### Pattern 18: Lending-to-LP capital rotation

```text
(Event-driven -- triggered by yield differential exceeding threshold)

1. lending-monitor -> Detects lending yield falling below LP fee APR
2. yield-scout -> Confirms LP opportunity with superior risk-adjusted yield
3. lending-manager -> Withdraws collateral from lending protocol via ActionPermit
4. liquidity-manager -> Deploys capital to LP position via ActionPermit
5. position-monitor -> Begins tracking new LP position
```

---

## Agent profiles

Profiles bundle related archetypes for common deployment configurations.

| Profile | Archetypes included | Use case |
|---------|-------------------|----------|
| `defi-full` | lending-manager, lending-monitor, staking-manager, derivatives-trader, technical-analyst, liquidity-manager, trade-executor, portfolio-analyst, risk-assessor, yield-scout, position-monitor, market-monitor | Full DeFi coverage: lending, staking, derivatives, LP, and monitoring |

---

## Model assignment summary

| Model  | Count | Archetypes                                                                                                                                                     |
| ------ | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| opus   | 30    | All execution, strategy, development, most economy, vault, golem, lending-manager, staking-manager, and derivatives-trader archetypes                          |
| sonnet | 12    | pool-researcher, pnl-analyst, wallet-provisioner, identity-verifier, market-monitor, position-monitor, vault-watchdog, heartbeat-monitor, sleepwalker-observer, lending-monitor, technical-analyst |

**Rationale**: Opus for complex reasoning (multi-step decisions, creative strategy, adversarial safety evaluation). Sonnet for structured tasks (threshold comparison, status checks, templated output). The model assignment directly affects burn rate -- see `../01-golem/01-cognition.md` for cost projections.

**Dev/test override**: Set `MODEL_OVERRIDE=sonnet` in the environment to run all archetypes on Sonnet for cost reduction during development and testing.

---
