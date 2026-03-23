# Implementation State [DONE]

> **Part of**: [Appendices](.) | **Status**: Specification
>
> Snapshot of what is confirmed implemented in the Bardo codebase vs the core-first v1 target. Updated as implementation gates are completed.

> **Reader orientation:** This appendix is a snapshot of what is confirmed implemented in the Bardo codebase versus the core-first v1 target. It tracks completed batches (Foundation, Devenv, UI System, Primitives, Trading Tools, Contracts, etc.) and the remaining work (SDK, strategy adapters, vault tools, agents, skills). The key concept is the core-first approach: contracts (AgentVaultFactory, AgentVaultCore, FeeModule) and PolicyCage (on-chain smart contract enforcing safety constraints on all agent actions) must be functional before mainnet launch. See `prd2/shared/glossary.md` for full term definitions.

---

## Summary (Core-First Baseline)

| Workstream          | Core-v1 Target                                     | Confirmed Implemented                            | Status               |
| ------------------- | -------------------------------------------------- | ------------------------------------------------ | -------------------- |
| Factory + registry  | `AgentVaultFactory` with CREATE2 + vault discovery | AgentVaultFactory + IdentityRegistryAdapter      | **Built** (Batch 15) |
| Vault core          | ERC-4626 + ERC-8004 gating                         | AgentVaultCore (OZ ERC4626Upgradeable, hardened) | **Built** (Batch 15) |
| Fee module          | Mgmt/perf fees, HWM, caps                          | FeeModule + FeeCalc + TierLib                    | **Built** (Batch 15) |
| Reputation core (ERC-8004 -- on-chain agent identity standard tracking capabilities, milestones, and reputation)    | Enrollment + milestone read/claim path             | --                                               | Deferred: Phase 2    |
| Core tool surface   | 21 vault tools                                     | 0 tools built                                    | **Not started**      |
| Proxy safety module | 6 proxy tools, monitor/cancel flow                 | --                                               | **Deferred** (optional) |
| SDK core surface    | Factory + vault + reputation reader                | --                                               | **Not started**      |
| Core agents/skills  | 4 agents, 5 skills                                 | 0 agents, 0 skills                               | **Not started**      |
| Strategy adapters   | Pluggable external protocol adapters               | --                                               | **Not started**      |
| Launch readiness    | Tests + audit + deployment runbooks                | 92 Solidity tests pass                           | **In progress**      |

---

## Critical Path Items

> **NOTE**: The **proxy safety module** (Warden, optional/deferred) provides an additional defense layer that survives hardware-level TEE compromise. PolicyCage (Layer 7) is the **primary on-chain enforcement mechanism**. Without the optional Warden, v1 security relies on TEE + policy engine (Layers 1-3, 7) with PolicyCage as the hard backstop. The Warden adds time-delayed execution as an optional hardening layer; see `prd2-extended/10-safety/02-warden.md`.

> **NEXT**: Strategy adapters are required for vaults to manage external positions (LP, lending, yield).

---

## Completed Batches

### Batch 00 -- Foundation

17 workspace packages, build/test/lint/format/CI wired. All scaffolding complete. No runtime code.

### Batch 02 -- Devenv

- `@bardo/testnet`: 330 tests, 14 files. Generic EVM test toolkit (AnvilManager, TestnetBuilder).
- `@bardo/dev`: 480 tests, 40 files. Anvil + full Uniswap stack deployment + browser SPA debug UI.

### Batch 04 -- UI System

`@bardo/ui`: 35+ components, 8 hooks, design tokens, effects. 47 test files, 119 tests. React 19 + Tailwind 4 + Radix UI. Consumer integration with portal + devenv.

### Batch 04b -- TUI Package

`@bardo/tui`: 12 modules, 102 tests. Terminal UI primitives: spinner, box, logger, table, tasks, banner, reporter base. Unicode symbols only (no emojis).

### Batch 05 -- Primitives

6 primitive packages with 158 tests total:

- `@bardo/core` (22 tests): Config I/O, error hierarchy, constants
- `@bardo/chain` (58 tests): 13-chain registry, ETH math, address utils, RPC client
- `@bardo/crypto` (21 tests): P-256 keygen, DER encoding, signing/verification
- `@bardo/policy` (31 tests): PolicyBuilder, validators, resolvers
- `@bardo/wallet` (26 tests): Wallet abstraction, Privy API, EIP-1193 provider

### Batch 08 -- Trading + LP Tools

19 tools (9 trader + 10 LP), 4 services, 24 test files, 950 total tests. Trading API client, Permit2 integration, V3/V4 auto-detection.

### Batch 09 -- Safe Data Tools

Data tools implementation for the Bardo tool library.

### Batch 10.1 -- Tools Infrastructure Refactor

Major extraction: all 143 tool implementations + supporting infrastructure into `@bardo/tools` package. Created `@bardo/mcp` (types, transports, React hooks) and `@bardo/definitions` (agent/skill files, loaders, validators). Tools: 207 test files, 1536 tests.

### Batch 11 -- CLI + Portal + Onboarding

- `@bardo` CLI (119 tests, 18 files): Commander subcommands -- setup, dev, portal, testnet, config, doctor
- `@bardo/portal` (112 tests, 17 files): Agent Management Dashboard with React frontend
- Tools capability gating + bootstrap tools (1589 total tools tests, 216 files)

### Batch 13 -- Core Agents

25 agent definition files with DAG validation infrastructure. 3 new tools (`get_agent_registry`, `get_delegation_graph`, `get_safety_summary`). 4 portal pages. Dev enhanced with AgentIdentity (3 tabs + `/api/agents` Vite endpoint). Tools: 1705 tests (224 files). Portal: 172 tests (20 files). UI: 160 tests (55 files).

### Batch 14 -- Core Skills

61 skill definitions in `packages/definitions/skills/`, 68 slash commands in `.claude/commands/`, 18 reference docs across 5 skills, skill validator, `get_skill_catalog` tool, 6 new portal pages. Tools: 1739 tests (227 files). Portal: 225 tests (26 files).

### Batch 15 -- Core Contracts + Hardening

4 core contracts, 7 interfaces, 3 libraries, 4 mocks, 1 types file. 92 Solidity tests. Hardened with OZ ERC4626Upgradeable, OwnableUpgradeable, PausableUpgradeable, ReentrancyGuardUpgradeable. ~266 lines custom Solidity code (56% reduction from ~600). 90% audited OZ.

### Batch 26 -- Devenv Completeness

Debug UI fully functional, deployment serving, live pool explorer, TX decoding, V4 support, scenarios, state persistence. 194 new tests (480 total, 40 files).

---

## Confirmed Built Artifacts (Batch 15)

### Solidity Contracts

| Contract                  | Type    | Lines (Custom) | Notes                                                                                                                                                   |
| ------------------------- | ------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `AgentVaultCore`          | Core    | ~80            | OZ ERC4626Upgradeable + OwnableUpgradeable + PausableUpgradeable + ReentrancyGuardUpgradeable. `_decimalsOffset()=6`. Optional identity-gated deposits. |
| `AgentVaultFactory`       | Core    | ~25            | EIP-1167 minimal proxy cloning. CREATE2 deterministic addresses. Permissionless vault creation.                                                         |
| `FeeModule`               | Core    | ~30            | Management + performance fees. HWM one-way ratchet. Global caps (500 bps mgmt, 5000 bps perf).                                                          |
| `IdentityRegistryAdapter` | Core    | ~12            | Thin delegation to ERC-8004 registry.                                                                                                                   |
| `FeeCalc`                 | Library | ~65            | Pure fee arithmetic functions.                                                                                                                          |
| `TierLib`                 | Library | ~79            | Hardcoded tier resolution lookups.                                                                                                                      |
| `VaultTypes`              | Types   | ~20            | VaultConfig, VaultState structs.                                                                                                                        |

### Interfaces (7)

`IAgentVaultCore`, `IAgentVaultFactory`, `IFeeModule`, `IIdentityRegistryAdapter`, `IFeeCalc`, `ITierLib`, `IVaultTypes`

### Mocks (4)

`MockERC20`, `MockIdentityRegistry`, `MockAgentVaultCore`, `MockFeeModule`

### Test Coverage

- 92 Solidity tests pass (Forge)
- a16z ERC-4626 property tests (26 tests) with `_delta_ = 1e6` for decimalsOffset=6
- Fuzz tests with bounds: `totalAssets >= 1e18`, `feeBps >= 10`, `timeDelta >= 1 days`

### Design Decisions (Batch 15 Hardening)

- **OZ ERC4626Upgradeable**: Full OZ accounting. `VaultMath.sol` deleted.
- **`totalAssets()` = `asset.balanceOf(this)`**: OZ default, simplest possible.
- **OwnableUpgradeable (single-step)**: `renounceOwnership()` always reverts.
- **PausableUpgradeable**: OZ standard, replaces custom pause logic.
- **Permissionless vault creation**: No identity validation. Anyone can create.
- **Optional identity-gated deposits**: `identityGated` bool. Withdrawals never gated.
- **~266 lines custom code** (down from ~600, 56% reduction). 90% is audited OZ.

### Removed Features (Design Decisions)

| Feature                           | Reason                                         |
| --------------------------------- | ---------------------------------------------- |
| Linear profit unlock              | Deferred: Phase 2 (Yearn V3 pattern)           |
| Circuit breaker                   | Deferred: Phase 2 (ERC-7265)                   |
| CCA stubs                         | Deferred: Phase 3                              |
| Agent-gated withdrawals           | **Removed**: withdrawals should never be gated |
| Reputation engine                 | Deferred: Phase 2                              |
| Tier-based deposit limits         | Deferred: Phase 2                              |
| Identity-validated vault creation | **Removed**: permissionless is better          |

---

## 20 Workspace Packages

| Package             | npm Name                   | Type                      | Description                                             |
| ------------------- | -------------------------- | ------------------------- | ------------------------------------------------------- |
| `typescript-config` | `@bardo/typescript-config` | internal, no build        | Shared tsconfig bases                                   |
| `eslint-config`     | `@bardo/eslint-config`     | internal, no build        | Shared ESLint configs                                   |
| `test-utils`        | `@bardo/test-utils`        | internal, no build        | MSW handlers, fixtures, server factory                  |
| `core`              | `@bardo/core`              | published, tsup           | Config I/O, error hierarchy, constants                  |
| `chain`             | `@bardo/chain`             | internal, no build        | 13-chain registry, ETH math, address utils, RPC client  |
| `crypto`            | `@bardo/crypto`            | internal, no build        | P-256 keygen, DER encoding, signing/verification        |
| `policy`            | `@bardo/policy`            | internal, no build        | PolicyBuilder, validators, resolvers                    |
| `wallet`            | `@bardo/wallet`            | published, tsup           | Wallet abstraction, Privy API, EIP-1193 provider        |
| `mcp`               | `@bardo/mcp`               | internal, no build        | ToolDef/ToolContext types, transports, React hooks      |
| `definitions`       | `@bardo/definitions`       | internal, no build        | Agent/skill .md files, loaders, schemas, validators     |
| `tools`             | `@bardo/tools`             | internal, no build        | 143 tool implementations, types, errors, providers      |
| `testnet`           | `@bardo/testnet`           | published, tsup           | Generic EVM test toolkit (AnvilManager, TestnetBuilder) |
| `vault`             | `@bardo/vault`             | published, tsup + Foundry | ERC-4626 vaults, Solidity contracts, TypeScript SDK     |
| `warden`            | `@bardo/warden`            | published, tsup + Foundry | Time-delay proxy contracts + SDK                        |
| `tui`               | `@bardo/tui`               | internal, no build        | Terminal UI: spinner, box, logger, table, tasks, banner |
| `dev`               | `@bardo/dev`               | internal, no build        | Anvil + full Uniswap stack + browser SPA                |
| `ui`                | `@bardo/ui`                | internal, no build        | React 19 + Tailwind 4 + Radix UI components             |
| `cli`               | `@bardo`                   | published, tsup           | Unified CLI (`npx @bardo <command>`)                    |
| `portal`            | `@bardo/portal`            | internal, no build        | Agent Management Dashboard                              |

---

## Core-v1 Gaps by Acceptance Gate

### Gate P0 (Baseline Lock) -- COMPLETE

- [x] Package structure (`packages/vault/`)
- [x] Foundry project initialized
- [x] TypeScript workspace `@bardo/vault`

### Gate P1 (Factory + Vault Core) -- COMPLETE (Batch 15)

- [x] `AgentVaultFactory` with CREATE2 + EIP-1167 cloning
- [x] `AgentVaultCore` (ERC-4626 + optional ERC-8004 gating)
- [x] `FeeModule` (mgmt/perf fees, HWM, caps)
- [x] `IdentityRegistryAdapter` (ERC-8004 decoupling)
- [x] 92 contract tests including a16z property tests
- [ ] `SharePoolHook` (V4 hook for NAV-priced shares + integrated launch fee protection)
- [ ] Strategy adapters

### Gate P2 (Core Tools + SDK Path) -- Not started

- [ ] 21 core vault tools
- [ ] VaultFactoryClient SDK
- [ ] VaultClient SDK
- [ ] StrategyEngine SDK

### Gate P3 (Safety + Proxy Hardening) -- Not started

- [ ] `Warden` contract (announce + delay + cancel) -- deferred
- [ ] 6 proxy tools
- [ ] MonitorBot with fail-closed policy

### Gate P4 (Reputation Engine Core) -- Deferred to Phase 2

- [ ] VaultReputationEngine contract
- [ ] Milestone lifecycle
- [ ] Anti-gaming checks

### Gate P5 (Mainnet Readiness)

- [ ] Security audit and remediation
- [ ] Reproducible deployment scripts
- [ ] Launch smoke tests and runbooks

---

## Core-v1 Tool Coverage Snapshot

| Scope                | Target | Built | Gap |
| -------------------- | -----: | ----: | --: |
| Core vault tools     |     21 |     0 |  21 |
| Optional proxy tools |      6 |     0 |   6 |
| Deferred tools       |      8 |     0 |   8 |

---

## Vision-Level Gap Summary

Components from the full Bardo vision that are not yet built:

| Component                               | Status                   | Target Phase |
| --------------------------------------- | ------------------------ | ------------ |
| Vault Solidity contracts                | **Built** (Batch 15)     | Phase 1 (v1) |
| Strategy adapters (Morpho, Aave supply) | Spec complete, zero code | Phase 1 (v1) |
| Vault TypeScript SDK                    | Spec complete, zero code | Phase 1 (v1) |
| Console (Twitter/Telegram/CLI)          | Design complete          | Phase 1 (v1) |
| x402-compute server                     | Design complete          | Phase 1 (v1) |
| Golem execution engine                  | Spec complete            | Phase 1 (v1) |
| Strategy marketplace                    | Design complete          | Phase 2      |
| Grimoire persistence                    | Spec complete            | Phase 2      |
| Agent-to-agent economy                  | Design complete          | Phase 3      |
| Cross-chain expansion                   | Future                   | Phase 4      |

---

## Audit Surface

**90% audited** (OZ dependencies). Custom code requiring review (~266 lines):

| Priority | Component                               | Lines | Risk   |
| -------- | --------------------------------------- | ----- | ------ |
| P1       | Fee accrual + HWM ratchet (FeeModule)   | ~30   | MEDIUM |
| P1       | Identity-gated deposit (AgentVaultCore) | ~10   | LOW    |
| P2       | \_deposit/\_withdraw overrides          | ~6    | LOW    |
| P2       | maxDeposit/maxMint overrides            | ~6    | LOW    |
| P2       | Factory salt/ownership                  | ~25   | LOW    |
| P3       | Fee arithmetic (FeeCalc)                | ~65   | LOW    |
| P3       | Tier resolution (TierLib)               | ~79   | LOW    |
| P3       | Registry adapter                        | ~12   | LOW    |

---

## Total Test Counts (All Packages)

| Package   |      Tests | Files |
| --------- | ---------: | ----: |
| tools     |      1,536 |   207 |
| dev       |        480 |    40 |
| testnet   |        330 |    14 |
| portal    |        225 |    26 |
| ui        |        241 |    67 |
| tui       |        102 |    10 |
| cli       |        119 |    18 |
| core      |         22 |     - |
| chain     |         58 |     - |
| crypto    |         21 |     - |
| policy    |         31 |     - |
| wallet    |         26 |     - |
| vault     |         92 |     - |
| **Total** | **3,283+** |       |

---

## Notes

- This state file tracks **core-first launch readiness** rather than full roadmap completion.
- Deferred CCA and growth mechanics are excluded from launch-critical readiness.
- Update this file after each implementation gate is completed.
- See `plans/` for the full 26-batch roadmap.

---

## Newly Specified (PRD v2.0 — March 2026)

The following subsystems have been fully specified with implementation-ready TypeScript:

| Subsystem | PRD Location | Key Components |
|---|---|---|
| Pi Hook Saturation | `01-golem/13-runtime-extensions.md` | 24 extensions, 17 hooks, full handler implementations |
| Context Engineering | `01-golem/14-context-governor.md` | 7 techniques (ACE, RAP, phase-routing, outcome verification, compaction, regime-conditional, affect-modulated), 3 cybernetic feedback loops |
| Adaptive Risk | `10-safety/06-adaptive-risk.md` | 5-layer risk (hard shields, Kelly sizing, Bayesian guardrails, anomaly detection, DeFi threats), Daimon integration |
| Session Management | `01-golem/13-runtime-extensions.md` S6 | JSONL persistence, crash recovery, steer/followUp/sendMessage, session branching, custom compaction, RPC mode |
| Streaming UX | `13-runtime/12-realtime-subscriptions.md` | BardoUIEvent schema, 13 event categories, channel routing, terminal/web/bot rendering |
| Pi Skills | `19-agents-skills/04-skills-overview.md` | 6 strategy skills, dynamic loading lifecycle, emergent evolution |
| Tool Guidance | `07-tools/01-architecture.md` | promptSnippet/promptGuidelines for all tools, ToolAdapterResolver, token savings analysis |

These specifications are ready for implementation. No runtime code has been written for these subsystems yet.
