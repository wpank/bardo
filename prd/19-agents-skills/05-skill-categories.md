# Interaction categories [SPEC]

> **Cross-ref**: [04-skills-overview.md](04-skills-overview.md) (interaction surfaces, operator directives, and Pi Skills overview), [06-skill-definitions.md](06-skill-definitions.md) (Pi workflow definition format and trigger types)

> **Reader orientation:** This document is the full inventory of interaction surfaces organized by domain: 68 typed API endpoints across 13 categories (Trading, Liquidity, Research, Strategy, Development, Infrastructure, Composite, Agent Capital Markets, Vault, Golem Operations, Protocol Fees, Real-time, and Development Testnet). It belongs to Section 19 (Agents & Skills). Each interaction maps to an archetype that handles the work. A Golem (mortal autonomous agent) exposes these endpoints to operators, dashboards, and other Golems. See `prd2/shared/glossary.md` for full term definitions.

---

## Interaction surface categories

Golems expose structured interaction endpoints organized by domain. These are not Claude skills or slash commands. They are typed API surfaces that operators, chat bots, dashboards, and other golems use to interact with a running Pi session.

Each interaction maps to an archetype that handles the work. The interaction surface is thin: parse input, validate, delegate to archetype, format output.

---

### 1. Trading (6 interactions)

Token swaps, limit orders, and cross-chain operations.

| Interaction          | Archetype              | Purpose                                     |
| -------------------- | ---------------------- | ------------------------------------------- |
| `execute-swap`       | `trade-executor`       | Execute a token swap on any supported chain |
| `batch-swap`         | `trade-executor`       | Execute multiple swaps in sequence          |
| `submit-limit-order` | `trade-executor`       | Submit a UniswapX Dutch auction order       |
| `cross-chain-swap`   | `cross-chain-executor` | Cross-chain swap or bridge via ERC-7683     |
| `bridge-tokens`      | `cross-chain-executor` | Bridge tokens between chains (no swap)      |
| `plan-swap`          | `trade-executor`       | Advisory swap planning with deep links      |

---

### 2. Liquidity (2 interactions)

LP position creation, management, and rebalancing.

| Interaction          | Archetype           | Purpose                                         |
| -------------------- | ------------------- | ----------------------------------------------- |
| `manage-liquidity`   | `liquidity-manager` | Add/remove liquidity, collect fees              |
| `rebalance-position` | `liquidity-manager` | Rebalance out-of-range LP position to new range |

---

### 3. Research (11 interactions)

Data querying, pool analysis, portfolio tracking, and P&L reporting.

| Interaction             | Archetype             | Purpose                                        |
| ----------------------- | --------------------- | ---------------------------------------------- |
| `analyze-pool`          | `pool-researcher`     | Pool deep-dive: TVL, volume, fees, tick data   |
| `compare-pools`         | `pool-researcher`     | Compare pools across fee tiers and versions    |
| `research-token`        | `token-analyst`       | Token due diligence: liquidity, risk, honeypot |
| `scan-opportunities`    | `opportunity-scanner` | LP/arb/yield opportunity scanning              |
| `find-yield`            | `opportunity-scanner` | LP yield scanning (focused version)            |
| `portfolio-report`      | `portfolio-analyst`   | Full portfolio report across all chains        |
| `track-performance`     | `portfolio-analyst`   | Performance tracking with change indicators    |
| `account-balance`       | (direct tool)         | Current token/LP balances across chains        |
| `balance-history`       | `pnl-analyst`         | Historical balance changes over time           |
| `pnl-report`            | `pnl-analyst`         | Profit and loss report                         |
| `performance-dashboard` | `pnl-analyst`         | Risk-adjusted metrics: Sharpe, drawdown, alpha |

---

### 4. Strategy (4 interactions)

LP optimization, strategy comparison, and risk assessment.

| Interaction   | Archetype       | Purpose                                           |
| ------------- | --------------- | ------------------------------------------------- |
| `optimize-lp` | `lp-strategist` | Optimal LP strategy recommendation                |
| `lp-strategy` | `lp-strategist` | Deep-dive LP strategy comparison                  |
| `assess-risk` | `risk-assessor` | Independent risk assessment for any operation     |
| `hedge-il`    | `lp-strategist` | Hedge impermanent loss via options or perp shorts |

---

### 5. Development (5 interactions)

V4 hook building, integration design, and auditing.

| Interaction          | Archetype               | Purpose                                       |
| -------------------- | ----------------------- | --------------------------------------------- |
| `build-hook`         | `hook-builder`          | Generate V4 hook: Solidity + tests + deploy   |
| `design-hook`        | `hook-builder`          | Design hook architecture (no code generation) |
| `design-integration` | `integration-architect` | Integration architecture design               |
| `audit-integration`  | `integration-architect` | Review existing integration for issues        |
| `test-hook`          | `hook-builder`          | Deploy and test hook on local testnet         |

---

### 6. Infrastructure (6 interactions)

Wallet setup, safety checks, session keys, approvals management.

| Interaction             | Archetype            | Purpose                                          |
| ----------------------- | -------------------- | ------------------------------------------------ |
| `setup-wallet`          | `wallet-provisioner` | Provision wallet with TEE keys, session keys     |
| `check-safety`          | (direct tool)        | Check spending limits, circuit breakers, keys    |
| `halt-golem`            | (direct)             | Emergency stop: revoke keys, trigger breakers    |
| `rotate-session-keys`   | `wallet-provisioner` | Zero-downtime session key rotation               |
| `audit-approvals`       | `safety-guardian`    | Scan and risk-score token approvals              |
| `revoke-approvals`      | `safety-guardian`    | Batch-revoke risky approvals, migrate to Permit2 |

---

### 7. Composite (5 interactions)

Multi-archetype orchestration for end-to-end workflows.

| Interaction             | Archetypes involved                                                   | Purpose                                           |
| ----------------------- | --------------------------------------------------------------------- | ------------------------------------------------- |
| `research-and-trade`    | token-analyst, pool-researcher, risk-assessor, trade-executor         | Research token, assess risk, trade if approved    |
| `full-lp-workflow`      | opportunity-scanner, lp-strategist, trade-executor, liquidity-manager | Find opportunity, design strategy, enter position |
| `cross-chain-arbitrage` | opportunity-scanner, risk-assessor, cross-chain-executor              | Find and execute cross-chain arbitrage            |
| `self-funding-setup`    | wallet-provisioner, identity-verifier, treasury-manager               | Full golem setup: wallet + identity + treasury    |
| `launch-token-full`     | token-deployer, liquidity-manager, safety-guardian                    | Full token launch: create + CCA + pool + lock     |

---

### 8. Agent capital markets (9 interactions)

Self-funding agent economy: treasury, tokens, identity, services, competitions.

| Interaction             | Archetype                 | Purpose                                  |
| ----------------------- | ------------------------- | ---------------------------------------- |
| `manage-treasury`       | `treasury-manager`        | Autonomous treasury management           |
| `deploy-agent-token`    | `token-deployer`          | Create agent token + V4 pool + hooks     |
| `verify-agent`          | `identity-verifier`       | Verify ERC-8004 identity and reputation  |
| `configure-x402`        | `agent-service-broker`    | Set up x402 micropayment endpoints       |
| `revenue-report`        | `treasury-manager`        | Unified revenue dashboard across streams |
| `monitor-competition`   | `opportunity-scanner`     | Track competitive intelligence           |
| `discover-competitions` | `competition-participant` | Find active hackathons and competitions  |
| `agent-otc-trade`       | `agent-service-broker`    | Agent-to-agent OTC trading               |
| `agent-market-maker`    | `agent-service-broker`    | Agent market-making services             |

---

### 9. Vault (7 interactions)

Vault deposit, management, strategy, am-AMM, yield discovery.

| Interaction           | Archetype            | Purpose                                               |
| --------------------- | -------------------- | ----------------------------------------------------- |
| `deposit-vault`       | `vault-manager`      | Deposit into a vault with identity verification       |
| `manage-vault`        | `vault-manager`      | Full vault operations: deposit/withdraw/rebalance     |
| `vault-strategy`      | `vault-strategist`   | Strategy analysis and research-backed recommendations |
| `bid-amamm`           | `vault-auctioneer`   | Bid for am-AMM management rights                      |
| `improve-strategy`    | `strategy-optimizer` | Self-improvement: Bayesian parameter tuning           |
| `discover-yield`      | `yield-scout`        | Multi-source yield discovery                          |
| `monitor-correlation` | `risk-assessor`      | Monitor cross-asset correlation for risk              |

---

### 10. Golem operations (4 interactions)

Golem lifecycle management.

| Interaction    | Archetype            | Purpose                                               |
| -------------- | -------------------- | ----------------------------------------------------- |
| `start-golem`  | `golem-instance`     | Start a golem instance from config                    |
| `stop-golem`   | `golem-instance`     | Graceful shutdown, wait for tick completion            |
| `golem-status` | `heartbeat-monitor`  | Query health, phase, vitality, mortality metrics      |
| `deploy-golem` | `golem-instance`     | Full provisioning: wallet + identity + config + boot  |

---

### 11. Protocol fees (3 interactions)

TokenJar monitoring, Firepit burn economics, fee claiming.

| Interaction              | Archetype             | Purpose                                 |
| ------------------------ | --------------------- | --------------------------------------- |
| `seek-protocol-fees`     | `protocol-fee-seeker` | Monitor and execute TokenJar burns      |
| `monitor-tokenjar`       | `protocol-fee-seeker` | Real-time TokenJar balance monitoring   |
| `analyze-burn-economics` | `protocol-fee-seeker` | Analyze profitability of burn-and-claim |

---

### 12. Real-time (2 interactions)

Streaming market and position monitoring.

| Interaction         | Archetype          | Purpose                               |
| ------------------- | ------------------ | ------------------------------------- |
| `monitor-markets`   | `market-monitor`   | Subscribe to price/volume/TVL streams |
| `monitor-positions` | `position-monitor` | Track LP positions, range, IL, fees   |

---

### 13. Development testnet (4 interactions)

Local testnet setup and testing utilities.

| Interaction           | Archetype        | Purpose                               |
| --------------------- | ---------------- | ------------------------------------- |
| `setup-local-testnet` | `hook-builder`   | Spin up Anvil with full Uniswap stack |
| `create-test-pool`    | `hook-builder`   | Create pool with custom parameters    |
| `time-travel`         | (direct tool)    | Advance EVM time for testing          |
| `setup-dca`           | `trade-executor` | Configure DCA strategy on testnet     |

---

## Summary

| Category              | Count  |
| --------------------- | ------ |
| Trading               | 6      |
| Liquidity             | 2      |
| Research              | 11     |
| Strategy              | 4      |
| Development           | 5      |
| Infrastructure        | 6      |
| Composite             | 5      |
| Agent capital markets | 9      |
| Vault                 | 7      |
| Golem operations      | 4      |
| Protocol fees         | 3      |
| Real-time             | 2      |
| Development testnet   | 4      |
| **Total**             | **68** |

---
