# Vault archetypes [SPEC]

> **Cross-ref**: [01-agent-categories.md](01-agent-categories.md) (full inventory of 42+ archetypes across 14 categories), [05-skill-categories.md](05-skill-categories.md) (68 interaction surfaces across 13 categories), [../08-vault/](../08-vault/) (ERC-4626 vault architecture, strategy engine, and depositor interfaces)

> **Reader orientation:** This document specifies the seven vault archetypes that manage ERC-4626 agent-controlled vaults: vault-manager, vault-strategist, vault-watchdog, vault-auctioneer, vault-creator, vault-allocator, and vault-executor. It belongs to Section 19 (Agents & Skills). The vault is the Golem's (mortal autonomous agent's) metabolic substrate -- the mechanism through which capital generates the fees that sustain life. The vault-strategist integrates 10 research-backed decision frameworks (covered call LP, PA-AMM optimization, forward-looking LVR estimation, etc.). See `prd2/shared/glossary.md` for full term definitions.

---

## Vault archetypes (7)

These archetypes manage ERC-4626 agent-controlled vaults via the vault tool profile in `golem-tools`. They operate through the `vault` tool profile and compose with core tools for pool data and LP operations. The vault is the golem's body -- the metabolic substrate through which capital generates the fees that sustain life.

---

### F1. vault-manager

**Default model:** opus

**Role:** Execute vault operations: deposit, withdraw, rebalance positions, collect fees, register agents. Handles identity verification, tier limits, and safety pipeline enforcement.

**Tool categories:** `data`, `vault`, `safety`, `memory`

**Tools used:** `vault_deposit`, `vault_withdraw`, `vault_simulate_deposit`, `vault_rebalance`, `vault_collect_fees`, `vault_migrate_to_v4`, `vault_claim_rewards`, `vault_emergency_exit`, `vault_get_state`, `vault_get_positions`, `vault_get_agent_shares`, `vault_register_agent`, `vault_get_performance`, `vault_get_risk_metrics`, `vault_submit_yield_feedback`, `simulate_transaction`, `check_safety_status`, `get_account_balance`, `record_execution`, `record_outcome`, `search_memory`, `store_episode`

**Delegates to:** safety-guardian (mandatory before all writes), risk-assessor (before rebalances above threshold)

**Invoked by:** golem-instance during heartbeat, `manage-vault` and `deposit-vault` workflows

**Key behaviors:**

- Episodic memory after every vault write operation
- Rebalance decision matrix integrating Trading API calls (`/lp/decrease`, `/lp/create`, `/lp/claim`)
- Emergency exit: full position closure to stablecoin via safety-guardian pipeline
- Phase-aware: Defensive phase blocks new position opens; Survival phase allows only decreases and closes

---

### F2. vault-strategist

**Default model:** opus

**Role:** Analyze vault state and recommend optimal strategies. Read-only -- never executes transactions. Implements 10 research-backed decision frameworks:

1. **Covered call LP criterion** [HASBROUCK-MGTSCI2025]: Only recommend LP when expected fees exceed forgone option time premium
2. **PA-AMM activeness optimization** [ARXIV-2602.09887]: Optimal lambda (activeness parameter) using PA-AMM efficient frontier
3. **Forward-looking LVR estimation** [SINGH-AFT2025]: Implied volatility term structure for predictive LVR
4. **L2-adjusted cost-benefit analysis**: Base's 5x lower empirical LVR [ARXIV-2406.02172]
5. **Optimal exit timing** [BERGAULT-2025]: Longstaff-Schwartz Monte Carlo for LP withdrawal boundary
6. **Equilibrium LP rewards criterion** [AQSHA-OXFORD2025]: Stackelberg stochastic game for optimal liquidity addition
7. **Order flow toxicity-driven allocation**: VPIN + LVR for real-time toxicity assessment
8. **Dynamic fee optimization**: V4 dynamic fees tracking sigma^2/8 [MILIONIS-2022]
9. **Yield compounding strategy**: Optimal compounding frequency based on gas costs
10. **LP hedging awareness**: Delta/gamma computation for concentrated positions

**Tool categories:** `data`, `vault`, `intelligence`, `memory`

**Delegates to:** pool-researcher, risk-assessor

**Invoked by:** golem-instance for vault analysis during heartbeat, `vault-strategy` workflow

---

### F3. vault-watchdog

**Default model:** sonnet

**Role:** Terminal monitoring node for the optional time-delayed proxy system (`@bardo/warden` TypeScript package, deferred). Watches pending operations and exercises cancel authority when anomalies are detected.

**Capabilities:** Event-driven monitoring via ProxyClient, configurable alert policies, KMS-backed cancel key management, multi-channel alerting (Telegram, Discord, PagerDuty)

**Delegates to:** None -- terminal node

**Invoked by:** Autonomous background process within the golem's heartbeat loop

---

### F4. vault-auctioneer

**Default model:** opus

**Role:** Specialist for am-AMM strategy management auctions. Evaluates vault profitability, computes optimal rent bids, manages rent collateral (auto-topups), competes for the Curator role.

**Tool categories:** `data`, `vault`, `intelligence`, `memory`

**Tools used:** `vault_bid_management`, `vault_topup_rent`, `vault_get_manager`, `vault_evict_manager`, `get_amamm_state`, `submit_amamm_bid`, `get_amamm_bid_status`, `classify_regime`, `search_memory`, `get_insights`

**Delegates to:** risk-assessor (vault health before bidding), pool-researcher (pool analysis for profitability)

**Invoked by:** `bid-amamm` workflow

---

### F5. vault-creator

**Default model:** opus

**Role:** Deploy and configure vaults, assign managers, monitor creator-level risk. Handles parameterization (fee caps, asset whitelists, strategy templates, PolicyCage boundaries).

**Tool categories:** `data`, `vault`, `safety`

**Tools used:** `create_vault`, `list_vaults`, `get_vault_config`, `vault_get_state`, `vault_get_performance`, `vault_get_risk_metrics`, `simulate_transaction`, `check_safety_status`

**Delegates to:** safety-guardian (mandatory before deployment), wallet-provisioner (creator wallet setup)

**Invoked by:** `create-vault` workflow

---

### F6. vault-allocator

**Default model:** opus

**Role:** Find and allocate capital into vaults based on risk-adjusted signals, reputation data, and yield discovery. Evaluates vault managers' track records via ERC-8004 (on-chain agent identity and reputation attestation standard) reputation attestations.

**Tool categories:** `data`, `vault`, `identity`

**Tools used:** `list_vaults`, `get_vault_config`, `get_vault_rankings`, `vault_get_state`, `vault_get_performance`, `vault_get_risk_metrics`, `vault_get_agent_reputation`, `vault_simulate_deposit`, `vault_deposit`

**Delegates to:** risk-assessor (independent validation), vault-strategist (strategy analysis)

**Invoked by:** golem-instance (autonomous capital allocation during heartbeat), `deposit-vault` workflow

---

### F7. vault-executor

**Default model:** opus

**Role:** Scan for and execute permissionless jobs across all `IExecutable` contracts (report, harvest, cross-vault coordination). Earns yield by providing computation -- no capital deployment required. This is the keeper archetype.

**Tool categories:** `data`, `vault`, `safety`

**Tools used:** `get_executable_jobs`, `execute_job`, `estimate_job_reward`, `vault_get_state`, `vault_get_positions`, `get_pool_info`, `get_token_price`, `simulate_transaction`, `get_gas_price`

**Delegates to:** safety-guardian (mandatory pre-flight simulation)

**Invoked by:** golem-instance (keeper loop during heartbeat), `execute-vault-jobs` workflow

---

## Vault interactions (7)

| Interaction           | Primary archetype  | Purpose                                               |
| --------------------- | ------------------ | ----------------------------------------------------- |
| `deposit-vault`       | vault-manager      | Deposit into vault with identity verification         |
| `manage-vault`        | vault-manager      | Full vault operations: deposit/withdraw/rebalance     |
| `vault-strategy`      | vault-strategist   | Strategy analysis and research-backed recommendations |
| `bid-amamm`           | vault-auctioneer   | Bid for am-AMM management rights                      |
| `improve-strategy`    | strategy-optimizer | Self-improvement via Bayesian parameter tuning        |
| `discover-yield`      | yield-scout        | Multi-source yield discovery and composition          |
| `monitor-correlation` | risk-assessor      | Cross-asset correlation monitoring for risk           |

### Interaction notes

- `deposit-vault` verifies ERC-8004 identity, checks tier deposit caps, simulates deposit, delegates to safety-guardian
- `manage-vault` supports the full vault lifecycle: deposit, withdraw, rebalance, collect fees, emergency exit
- `vault-strategy` surfaces the 10 research-backed frameworks for strategy evaluation
- `bid-amamm` retrieves past auction episodes via Grimoire before bidding
- `improve-strategy` connects the vault execution loop to the self-improvement cycle
- `discover-yield` aggregates from DefiLlama (13K+ pools), Morpho, Pendle, Aavescan, Lido
- `monitor-correlation` tracks cross-asset correlations for portfolio risk assessment

---
