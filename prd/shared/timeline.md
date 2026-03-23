# Implementation Timeline [SPEC]

> **Document Type**: SPEC (normative) | **Referenced by**: All domain PRDs | **Last Updated**: 2026-03-08
>
> Unified implementation timeline synthesizing phase schedules from all Bardo products. Week numbers are relative to the start of development (Week 0 = project kickoff). Dependency annotations indicate when one domain unblocks another.

> **Reader orientation:** This document is the unified implementation timeline for the entire Bardo ecosystem, synthesizing phase schedules from all products: Dev Environment, Bardo Sanctum (tool library), Bardo Vaults (ERC-4626), Agents, Skills, Interfaces, Golem (mortal autonomous agent runtime), Compute, and Inference. Week numbers are relative to project kickoff (Week 0). The key concept is the cross-domain dependency chain: for example, the Sanctum tool library must ship read tools before Agents can begin, and PolicyCage (on-chain smart contract enforcing safety constraints) must be functional before mainnet launch. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. Domain Summary

| Domain          | Phases                          | Week Range | Source              |
| --------------- | ------------------------------- | ---------- | ------------------- |
| Dev Environment | 1 phase (precondition)          | 0-2        | `15-dev/`           |
| Bardo Sanctum (unified Uniswap-native tool library)   | 7 phases                        | 1-28       | `07-tools/`       |
| Bardo Vaults    | 5 phases (+ Track A-G deferred) | 0-22       | `08-vault/`         |
| Agents          | 3 phases                        | 1-20       | `19-agents-skills/` |
| Skills          | 3 phases                        | 5-22       | `19-agents-skills/` |
| Interfaces      | 3 phases                        | 1-24       | `18-interfaces/`    |
| Golem           | 3 phases                        | 5-25       | `01-golem/`         |
| Compute         | 2 phases                        | 8-18       | `11-compute/`       |
| Bardo Inference (x402-gated LLM gateway routing between Anthropic/OpenAI/Google/Venice/Grok)       | 2 phases                        | 6-16       | `12-inference/`     |
| Testing (includes Gauntlet -- swarm simulation environment running batches of Golems against forked Base chain state)         | 3 phases                        | TBD        | `16-testing/`       |

---

## 2. Week-by-Week Phase Map

| Week  | Dev          | Sanctum              | Vaults              | Agents             | Skills             | Interfaces      | Golem                 | Compute        | Inference            |
| ----- | ------------ | -------------------- | ------------------- | ------------------ | ------------------ | --------------- | --------------------- | -------------- | -------------------- |
| 0     | **Setup**    | --                   | **P0: Baseline**    | --                 | --                 | --              | --                    | --             | --                   |
| 1-2   | **Complete** | **P1: Foundation**   | **P1: Factory**     | **A1: Foundation** | --                 | **W1: Landing** | --                    | --             | --                   |
| 3-4   | done         | P1                   | P1                  | A1                 | --                 | W1              | --                    | --             | --                   |
| 5-8   | done         | **P2: Trading**      | P1->**P2: SDK**     | A1                 | **S1: Trading/LP** | W1              | **G1: Core**          | --             | **I1: Gateway**      |
| 9-12  | done         | **P3: Liquidity**    | P2->**P3: Safety**  | **A2: Core**       | S1                 | **W2: App**     | G1                    | **C1: Infra**  | I1                   |
| 13-15 | done         | **P4: Advanced**     | P3                  | A2                 | **S2: Advanced**   | W2              | G1->**G2: Lifecycle** | C1             | I1->**I2: Optimize** |
| 16-20 | done         | **P5: Hardening**    | **P4: Reputation**  | A2->**A3: Vault**  | S2                 | **W3: Portal**  | G2                    | C1->**C2: HA** | I2                   |
| 18-22 | done         | P5                   | **P5: Mainnet**     | A3                 | S2                 | W3              | G2                    | C2             | done                 |
| 21-24 | done         | **P6: CCA/am-AMM**   | done (mainnet)      | done               | **S3: Vault+Econ** | W3              | G2->**G3: Clade**     | done           | done                 |
| 25-28 | done         | **P7: Intelligence** | Track A-G (post-v1) | done               | S3                 | done            | G3                    | done           | done                 |

**Legend:** done = phase complete; -- = not started; bold = phase starting.

---

## 3. Phase Details

### 3.1 Dev Environment (Weeks 0-2)

Precondition for all other development. Must be available before Sanctum P1 and Vault P1.

| Phase | Weeks | Goal                                                                                   |
| ----- | ----- | -------------------------------------------------------------------------------------- |
| Setup | 0-2   | Full Uniswap stack (V2/V3/V4/UniswapX/ERC-8004) on local Anvil; debug UI; seed scripts |

**Unblocks**: Sanctum P1 (forked mainnet tests), Vault P1 (testnet deployment).

### 3.2 Bardo Sanctum (Weeks 1-28)

| Phase            | Weeks | Goal                                          | Key Deliverables                                         |
| ---------------- | ----- | --------------------------------------------- | -------------------------------------------------------- |
| P1: Foundation   | 1-4   | Read-only data tools, caching, eval framework | `get_token_price`, `get_pool_info`, chain provider layer |
| P2: Trading      | 5-8   | Swap execution, safety pipeline               | `get_quote`, `execute_swap`, safety middleware           |
| P3: Liquidity    | 9-12  | LP management, V4 PositionManager             | `add_liquidity`, `remove_liquidity`, `calculate_il`      |
| P4: Advanced     | 13-16 | UniswapX, cross-chain, x402, wallets          | `submit_uniswapx_order`, x402 providers                  |
| P5: Hardening    | 17-20 | Prompt injection defense, reputation gates    | CaMeL dual-LLM, ERC-8004 tier enforcement                |
| P6: CCA/am-AMM   | 21-24 | CCA tools, Liquidity Launcher                 | `get_cca_state`, `submit_cca_bid`, `launch_token`        |
| P7: Intelligence | 25-28 | Intelligence tools, smart accounts            | `simulate_pool_scenario`, all 158+ tools                 |

### 3.3 Bardo Vaults (Weeks 0-22)

| Phase                     | Weeks | Goal                              | Key Deliverables                                            |
| ------------------------- | ----- | --------------------------------- | ----------------------------------------------------------- |
| P0: Baseline Lock         | 0-1   | PRD alignment, count freeze       | DRIFT-MATRIX, IMPLEMENTATION-STATE baseline                 |
| P1: Factory + Core        | 1-5   | Permissionless vault creation     | AgentVaultFactory, AgentVaultCore, NAVAwareHook             |
| P2: Core SDK + Tools      | 6-9   | External agent access             | 24 core vault tools, VaultClient, OnboardRouter             |
| P3: Safety + am-AMM       | 10-15 | Safety hardening, am-AMM auctions | PolicyCage hardening, am-AMM tools, ERC-7265 circuit breakers, optional Warden (deferred) |
| P4: Reputation + Adapters | 16-20 | Milestone lifecycle, adapters     | Reputation engine, MorphoSupplyAdapter, AaveV3SupplyAdapter |
| P5: Mainnet               | 18-22 | Launch on Base                    | Audit, Base deployment, monitoring, SIWE auth               |

**Mainnet-blocking gate (P3)**: PolicyCage and safety hardening MUST be functional before P5. Warden is optional/deferred.

### 3.4 Golem (Weeks 5-25)

| Phase                   | Weeks | Goal                                      | Key Deliverables                                              |
| ----------------------- | ----- | ----------------------------------------- | ------------------------------------------------------------- |
| G1: Core                | 5-14  | Heartbeat FSM, mortality engine, Grimoire | Pi-mono integration, SurvivalState, PLAYBOOK.md, triple-loop  |
| G2: Lifecycle           | 15-22 | Creation, provisioning, death protocol    | Manifest, 8-step pipeline, Death Protocol, inheritance        |
| G3: Clade + Replication | 23-28 | Peer-to-peer knowledge sharing, evolution | Clade REST API, MAP-Elites replication, stigmergic validation |

### 3.5 Compute (Weeks 8-18)

| Phase                 | Weeks | Goal                                                         |
| --------------------- | ----- | ------------------------------------------------------------ |
| C1: Infrastructure    | 8-14  | Fly.io two-app topology, x402 billing, warm pool, supervisor |
| C2: High Availability | 15-18 | TTL worker, Turso CAS, monitoring, alerting, multi-region    |

### 3.6 Inference (Weeks 6-16)

| Phase            | Weeks | Goal                                                       |
| ---------------- | ----- | ---------------------------------------------------------- |
| I1: Gateway      | 6-12  | Multi-provider routing, x402 billing, Facilitator contract |
| I2: Optimization | 13-16 | Prompt caching, semantic caching, cost reduction (40-85%)  |

---

## 4. Cross-Domain Dependencies

| Dependency                    | Blocks                                   |
| ----------------------------- | ---------------------------------------- |
| Dev (W0-2)                    | Sanctum P1, Vault P1                     |
| Sanctum P1 (read tools)       | Agents A1, Interfaces W2                 |
| Sanctum P2 (write tools)      | Agents A2, Skills S1                     |
| Sanctum P5 (safety hardening) | Agents A3                                |
| Vault P1 (factory deployed)   | Interfaces W2, Agents A3                 |
| Vault P2 (tools live)         | Skills S3, Interfaces W3                 |
| Vault P3 (Safety gate)        | Vault P5 (mainnet launch)                |
| Vault P4 (reputation engine)  | Skills S3                                |
| Golem G1 (core runtime)       | Compute C1 (needs golem to host)         |
| Inference I1 (gateway)        | Golem G1 (needs inference for cognition) |
| Compute C1 (VM hosting)       | Golem G2 (needs hosting for lifecycle)   |

---

## 5. Product Roadmap Phases

### Phase 1 -- Lending Strategies (Vault P1-P3, Sanctum P1-P3)

Prove the full loop: deploy vault, earn yield, generate performance card, copy-deploy. Vaults + Morpho/Aave adapters + Console + Golem + x402-compute.

### Phase 2 -- Composability (Vault P4-P5, Sanctum P4-P5)

PendleAdapter, recursive lending, strategy marketplace, copy-deploy flow, keeper economy.

### Phase 3 -- Ecosystem Expansion (Post-v1 Tracks A-D)

New yield sources (Aerodrome, Ondo, Ethena), agent-to-agent economy, A2A Agent Cards, Gauntlet swarm demo.

### Phase 4 -- Flywheel (Tracks E-G and beyond)

Self-sustaining agents, cross-chain expansion, custom strategy compilation, institutional features.

---

## 6. Unit Economics (Projections)

> **Moved to [../revenue-model.md](../revenue-model.md) Section 2 (Unit Economics).**

---

## 7. Deferred Tracks (Post-v1)

Post-mainnet expansion tracks activate based on entrance criteria:

| Track                        | Scope                                                    | Entrance Criteria                                 |
| ---------------------------- | -------------------------------------------------------- | ------------------------------------------------- |
| Track A: CCA Expansion       | CCA lifecycle tools                                      | Vault P3+ stable, CCA risk model validated        |
| Track B: Viral Growth        | Referrals, multiplier points, streaks                    | Core retention targets met; Sybil model validated |
| Track C: Advanced V4         | JIT liquidity, PolicyCage, reputation-tiered LP          | Vault P6 V4 modules stable                        |
| Track D: Cross-Vault         | ExecutionMarket, LiquidityRouter, intent solver          | 50+ active vaults; Track A stable                 |
| Track E: Composability       | Pendle integration, Morpho collateral, recursive lending | Vault P5 stable 4+ weeks; adapters audited        |
| Track F: Structured Products | Tranches (PYT), BondMM fixed-rate                        | Track E lending adapters operational              |
| Track G: ve-Token Governance | Optional veVAULT module                                  | 100+ active vaults; community demand              |
