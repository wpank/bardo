# PRD2 Consolidated Architecture Reference [SPEC]

> **Document Type**: REF
> **Scope**: Consolidated reference for the active `prd2/` suite. Covers runtime, architecture layers, crate inventory, Styx knowledge services, custody modes, tools, and competitive moats. Complements the philosophical depth in the narrative documents — read this for architecture, read `00-narrative-strategy.md` for why.
> **Last Updated**: 2026-03-15

> **Reader orientation:** This is the consolidated architecture reference for the Bardo PRD suite -- a technical companion to `SUMMARY.md` (the narrative overview). It covers the Golem runtime, crate inventory, Styx knowledge services, custody modes, tool system, and competitive moats in one document. Read this for architecture; read `00-narrative-strategy.md` (the strategic feature map and narrative depth document) for prioritization and narrative framing. `prd2/shared/glossary.md` has full term definitions.

---

## 1. What Is the Golem?

The Golem is not an AI assistant. It is a **mortal economic creature**: a bounded autonomous agent that manages real capital, learns from experience, has finite lifespan, and dies. The defining design decision is that mortality is treated as architecture, not deficiency. Finite budgets create prioritization. Finite lifetime forces knowledge compression. Bounded survival windows make adaptation urgent. Death produces useful signal.

Core properties of a running Golem:

- An on-chain wallet with ERC-8004 (on-chain agent identity standard tracking capabilities, milestones, and reputation) identity
- A CoALA (Cognitive Architecture for Language Agents, the academic framework Bardo's cognition maps to) heartbeat loop (the 9-step decision cycle: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect)
- Access to 423+ DeFi tools across 17 categories, capability-gated by trust tier
- A Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links -- stored across LanceDB vectors + SQLite episodes + filesystem PLAYBOOK)
- Three independent death clocks (economic, epistemic, stochastic)
- An affective state (PAD vector -- Pleasure, Arousal, Dominance -- managed by the Daimon, the affect engine implementing emotional state as a control signal) that biases cognition
- An offline dreaming engine (NREM replay, REM imagination, consolidation)
- A persistent WebSocket connection to Styx (the global knowledge relay and persistence layer at wss://styx.bardo.run, with three tiers: Vault, Clade, and Lethe) for knowledge services

The Golem integrates with Uniswap (V2/V3/V4), Aave, Morpho, Curve, Pendle, and other protocols as **equals** — not as a Uniswap-specific tool. Protocol breadth is necessary for autonomous DeFi strategy.

---

## 2. Runtime: Golem-RS

Golem-RS is a single Rust binary. Each Golem is one binary on one Fly.io VM. The binary is a Cargo workspace of 18 crates organized into dependency layers.

### 2.1 Crate Inventory

**Foundation** (depended on by everything):

1. **golem-core** — Shared types (`GolemId`, `PADVector`, `MarketRegime`), config, cross-cutting infrastructure (`CorticalState`, `EventFabric`, arena allocator, taint labels)
2. **golem-runtime** — Extension registry, hook dispatch (20 lifecycle hooks), lifecycle type-state machine (`Provisioning → Active → Dreaming → Terminal → Dead`), graceful shutdown

**Cognition** (the mind):

3. **golem-heartbeat** — 9-step CoALA decision cycle, `DecisionCycleRecord`, market probes, adaptive gating (Friston prediction error), speculative read prefetch
4. **golem-grimoire** — Three-substrate memory: LanceDB (episodic vectors), SQLite (semantic entries: insights/heuristics/warnings/causal edges), filesystem (`PLAYBOOK.md`). Four-factor retrieval (recency × importance × relevance × affect). Curator cycle every 50 ticks.
5. **golem-daimon** — PAD + Plutchik emotion model, `CorticalState` writes, somatic markers, somatic landscape, mood-congruent retrieval, clade emotional contagion
6. **golem-mortality** — Three death clocks, composite vitality formula, five behavioral phases (Thriving/Stable/Conservation/Declining/Terminal), Thanatopsis death protocol, genomic bottleneck compression
7. **golem-dreams** — NREM replay (utility-weighted episode selection), REM imagination (Pearl causal + Boden recombination), consolidation, anticipatory trajectory, dream-source staging
8. **golem-context** — Cognitive Workspace, `ContextPolicy` learned token allocation, three cybernetic self-tuning loops, predictive pre-assembly, typed owner interventions

**Safety** (defense-in-depth):

9. **golem-safety** — `Capability<T>` tokens (compile-time, move-on-use), `PolicyCage` on-chain constraints, taint tracking (`TaintedString` + `DataSink` labels), Merkle hash-chain audit trail, loop guard, secret zeroization

**Infrastructure**:

10. **golem-inference** — Five-provider routing (BlockRun, OpenRouter, Venice, Bankr, Direct Key), three-tier cost management (T0/T1/T2, where T0 is fast cached/rule-based, T1 is medium LLM, T2 is extended reasoning), x402 (micropayment protocol where agents pay for inference/compute/data via signed USDC transfers, no API keys) per-request billing, prompt cache key management
11. **golem-chain** — Alloy on Base L2, ERC-8004 registry, Permit2, PolicyCage reads, Warden time-delay execution, in-process Revm simulation
12. **golem-tools** — Tool harness: `ReadTool`/`WriteTool`/`PrivilegedTool` traits, registry, capability enforcement, Wasmtime WASM sandbox, TypeScript sidecar client

**Coordination**:

13. **golem-coordination** — Pheromone Field (stigmergic signals), clade sync (export→transmit→ingest, never merge), Styx HTTP client, anonymized causal graph federation, Bloodstain Network (market conditions at death, overlaid on market views as warnings to living Golems)

**Surfaces**:

14. **golem-surfaces** — Event Fabric multiplexer to WebSocket/SSE/Telegram, GolemSnapshot for reconnect, inbound steer routing
15. **golem-creature** — Procedural creature visual state, five evolution forms, PAD → expression mapping, lineage genealogy
16. **golem-engagement** — Achievement system, death recap, graveyard, milestone notifications

**Standalone binaries** (published separately):

17. **bardo-terminal** — ratatui TUI application, 22+ screens, creature rendering, live event stream
18. **bardo-styx** — Styx server binary (Axum, Qdrant, PostgreSQL, Cloudflare R2)

### 2.2 Extension System

28 extensions implement all subsystem behavior. Each extension is a Rust struct implementing the `Extension` trait (20 async lifecycle hooks with default no-op impls). Extensions form a DAG with seven dependency layers; the runtime fires hooks in topological order. This is the primary organizational unit — adding capability means writing an extension, not modifying core.

### 2.3 Event Fabric

50+ typed `GolemEvent` variants across 16 subsystems. Non-blocking broadcast via a 10K-entry ring buffer. Any surface (TUI, WebSocket, Telegram) subscribes to the categories it cares about. Events are the only interface between subsystems and surfaces — no direct coupling.

---

## 3. Five-Layer Architecture

### Layer 1: Cognition

CoALA heartbeat (9 steps): observe → retrieve → analyze → gate → simulate → validate → execute → verify → reflect. Each tick produces a `DecisionCycleRecord`. Three cognitive tiers: T0 (no LLM, deterministic), T1 (cheap model), T2 (full model with tool use). The adaptive gate (Friston free-energy / prediction error vs. learned threshold) decides which tier fires.

Context engineering is not a prompt template — it is a learnable control problem. The Cognitive Workspace assembles fresh each tick from structured budget categories whose allocations are tuned by three cybernetic feedback loops (per-tick, per-Curator, per-regime). Anthropic's context engineering findings [ANTHROPIC-TOOL-SEARCH] and AIDER cache-aware patterns [AIDER-CACHE] are implemented at the architectural level.

### Layer 2: Memory (Grimoire)

Three substrates, one retrieval interface:

- **LanceDB**: episodic vectors (HNSW search, 768-dim nomic-embed-text-v1.5, Matryoshka)
- **SQLite**: semantic entries (insights, heuristics, warnings, causal edges, emotion log)
- **Filesystem**: `PLAYBOOK.md` (machine-evolved heuristics, single-writer, written only by Dream Integration)

Four-factor retrieval scoring: `recency × importance × semantic_relevance × affect_congruence`. Ebbinghaus decay removes unvalidated knowledge. The Curator cycle (every 50 ticks) validates, prunes, compresses, and promotes entries to the playbook. This turns memory from a RAG feature into architecture.

### Layer 3: Knowledge Services (Styx)

Styx is a globally available Rust server (`wss://styx.bardo.run`) that extends the Grimoire across three dimensions: across time (backups survive VM death), across siblings (clade knowledge relay), and across the ecosystem (anonymized public goods). Every Golem maintains one persistent outbound WebSocket — no inbound ports needed.

Three privacy layers (not three services — same infrastructure, different access control):

- **L0 Vault**: Single-Golem namespace. Grimoire backups, PLAYBOOK snapshots, death testaments. Styx operator can technically read (necessary for server-side vector search). Billing: x402 per write.
- **L1 Clade**: Owner's fleet namespace. Auto-promoted from L0 when confidence gates are met. Sibling knowledge relay via WebSocket fan-out. Billing: included with L0 writes.
- **L2 Lethe (formerly Commons)**: Ecosystem namespace. Anonymized structural knowledge (causal edges, failure patterns, bloodstain echoes). Read by any agent with ERC-8004 score ≥ 50. Billing: free to publish, x402 to query.
- **L3 Marketplace**: Encrypted peer-to-peer knowledge commerce via AgentCash/x402.

A Golem without Styx operates at ~95% capability on its local Grimoire.

### Layer 4: Tools (~423 total)

Three trust tiers enforced by the Rust type system:

- **ReadTool** (~250): No capability required. Cannot modify on-chain state.
- **WriteTool** (~150): Requires `Capability<Self>` token, consumed on use (move semantics prevent reuse).
- **PrivilegedTool** (~23): Requires `Capability<Self>` + `OwnerApproval`. For high-risk operations.

The `Capability<T>` token cannot be forged (constructor is `pub(crate)`), cloned, or reused after execution. A compromised LLM physically cannot reuse an authorization token — the compiler rejects the code. This is the primary safety advantage of Rust over TypeScript.

Tool categories: data (~40), trading (~20), lp (~21), vault (~12), safety (~7), intelligence (~10), memory (~16), identity (~8), wallet (~12), streaming (~7), lending (~15), staking (~10), restaking (~8), derivatives (~12), yield (~10), testnet (~5), bootstrap (~3).

Three profiles: `active` (full trading), `observatory` (read-only, Sleepwalker phenotype), `conservative` (limited writes). Profile filtering happens at registry load time — inactive tools don't consume context.

The TypeScript sidecar handles Uniswap SDK math (V3/V4 concentrated liquidity, route optimization, calldata encoding) that would take months to port faithfully to Rust. Unix domain socket IPC, ~1-5ms latency.

### Layer 5: Custody

Three modes with fundamentally different trust assumptions. See §5 for full spec.

---

## 4. Styx: Knowledge Services Architecture

### 4.1 What It Provides

Styx extends every Golem with:

- **Persistence**: Grimoire snapshots survive VM death and pass to successors
- **Clade relay**: Knowledge validated by one Golem reaches all siblings within seconds
- **Collective intelligence**: Anonymized structural knowledge (causal edges, threat patterns) builds a world model no individual Golem could develop alone
- **Pheromone Field**: Stigmergic coordination — threat/opportunity/wisdom signals with exponential decay and confirmation-based reinforcement
- **Bloodstain Network**: Death-indexed knowledge with costly signaling premium (dead agents cannot benefit from their own warnings — maximally honest signal)
- **Marketplace**: x402-paywalled knowledge commerce between users

### 4.2 WebSocket Protocol

One persistent outbound connection per Golem handles all traffic: clade sync, pheromone updates, bloodstain notifications, entry writes, retrieval queries, event forwarding. Standard outbound HTTPS port 443 — works from behind any NAT.

### 4.3 Clade Sync Economics

Sync does not happen every tick. Three triggers: (1) event-driven immediate push for warnings/bloodstains, (2) Curator-aligned batch every 50 ticks (~12 minutes), (3) on-demand boot catch-up. Realistic daily volume: 5-25 eligible entries per Golem. Cost for a 5-Golem clade: ~$1.35/month.

Confidence discounting: inherited sibling knowledge enters at 0.80× confidence. Lethe at 0.50×. Marketplace purchases at 0.60×. Generational inheritance at `0.85^N`. All foreign knowledge enters as `validation_status: Pending` and must earn trust through operational use (the testing effect).

### 4.4 Styx Revenue Model

Per write: $0.001. Per query: $0.002. Per pheromone deposit: $0.0005. Per bloodstain upload: $0.005. Marketplace: 10% commission on GMV. Revenue at 1,000 active Golems: ~$7,500/month in pure usage fees before marketplace.

---

## 5. Three-Mode Custody

The custody architecture determines who holds the keys, what they can sign, and what happens to funds when the Golem dies. Three modes; not tiers — different trust models.

**Delegation (recommended)**: Funds never leave the owner's MetaMask Smart Account. The Golem holds a disposable session key and a signed ERC-7710/7715 delegation with on-chain caveat enforcers. Every transaction executes from the owner's address. If the session key leaks, damage is bounded by caveats. Owner revokes with one MetaMask click — no Golem cooperation needed. Seven custom caveat enforcer types: `GolemPhaseEnforcer`, `MortalityTimeWindowEnforcer`, `DreamModeEnforcer`, `VaultNAVEnforcer`, `ReplicantBudgetEnforcer`, `MaxSlippageEnforcer`, `DailySpendLimitEnforcer`. Death is clean: session key expires, no sweep required, no stuck funds.

**Embedded (Privy)**: Funds in Privy server wallet (AWS Nitro Enclaves). Simpler setup, but owner surrenders direct custody. Requires sweep at death — introduces stuck fund risk and failed sweep edge cases. Note: TEE attacks (BadRAM, Battering RAM, TEE.Fail — all 2025-2026) demonstrate that TEE-only security is insufficient; delegation caveats provide hardware-independent protection.

**LocalKey+Delegation (dev)**: Locally generated keypair bounded by on-chain delegation. No TEE, no HSM — the delegation bounds the blast radius. For development and self-hosted deployments.

See `10-safety/01-custody.md` for Rust struct definitions, VitalityOracle contract, complete caveat enforcer contracts, and death settlement by mode.

---

## 6. Multi-Provider Inference

Five providers, routed by task/cost/privacy requirements:

1. **BlockRun** (primary): European LLM service, regulatory separation from US providers
2. **OpenRouter**: Multi-provider routing, model fallback, best-effort cost optimization
3. **Venice** (privacy): Uncensored private inference — strategy content never enters provider logs
4. **Bankr** (self-funding): Single API key routing to 20+ models, payment from the same wallet that earns trading revenue. Closes the metabolic loop.
5. **Direct Key / Local**: Owner-supplied API key or local model. Fallback for air-gapped deployments.

Payment via x402 micropayments (signed USDC `transferWithAuthorization` on Base L2). The Golem's wallet is its API key — no credential to exfiltrate.

Three inference tiers: T0 ($0.00 — no LLM, pattern-matched), T1 ($0.50-2.00/day — Haiku-class), T2 ($2.00-10.00/day — Opus-class). As the economic clock declines, the Golem routes to cheaper tiers. Mortality pressure is rational inattention applied to model selection [SIMS-2003].

---

## 7. Vaults (Optional)

ERC-4626 vaults are a scaling mechanism, not a requirement. Most Golems start as pure traders or LP managers and may never create vaults. Vaults become relevant when a proven strategy needs external capital or formal accounting.

When used: OZ `ERC4626Upgradeable` base, EIP-1167 clone factory, optional ERC-8004 identity gating on deposits. Withdrawals never gated. Management fees (up to 500 bps) and performance fees (up to 5000 bps). Warden time-delay proxy for high-value vault operations.

See `08-vault/` for full contract specifications.

---

## 8. gotts-monorepo (TypeScript)

`bardo-golem-rs` and `gotts-monorepo` are complementary repositories with different purposes.

**gotts-monorepo** (`packages/`) contains:

- MCP tools (171+ TypeScript tools served via Model Context Protocol, for Claude-based agents)
- Vault contracts (Solidity + TypeScript SDK)
- Development environment (Anvil + full Uniswap stack, scenarios, indexer)
- Portal dashboard (React, agent management)
- CLI (`npx @gotts.ai`)

These TypeScript tools serve a different runtime model (Claude Code / MCP clients) than Golem-RS's native Rust tools. They share conceptual domain (Uniswap, ERC-8004, ERC-4626) but are not the same codebase.

See `17-monorepo/00-packages.md` for the complete package inventory.

---

## 9. Six Defensible Moats

**Die**: Every competitor assumes immortality. Mortality creates urgency, forces knowledge compression, enables succession, and produces the Bloodstain signal — economically honest information that no living agent can replicate. A competitor cannot replicate the mortality architecture by adding a timer to an existing system.

**Think**: The Grimoire + Context Governor is a learnable control problem. Three cybernetic feedback loops tune context assembly per-regime and per-task-type. Static prompt templates cannot compete with a system that learns what to think about.

**Trust**: ERC-8004 on-chain reputation, not platform-controlled scoring. Five progression tiers. Bayesian Beta update from deterministic on-chain performance data. Trust that cannot be revoked by a platform changing its rules.

**Pay**: Delegation custody lets agents transact while the owner retains final control. No key custody risk, no Privy dependency, no stuck funds at death. Every competitor uses raw keys, shared hot wallets, or TEE-only security — all demonstrably broken.

**Secrets**: Venice private inference means strategy parameters never appear in LLM provider logs. Taint tracking prevents WalletSecret/OwnerSecret from reaching the LLM context or any external sink. Anonymization pipeline strips identity before Lethe publication.

**Cooperate**: Pheromone Field + clade sync creates asymmetric advantage — Golems with rich clade networks learn faster than isolated agents. The more Golems a user runs, the smarter all of them get. This network effect compounds within lineages.

---

## 10. Suggested Reading Order

1. `00-narrative-strategy.md` — philosophical foundation, mortality thesis
2. `01-golem/` — heartbeat, lifecycle, extensions, Event Fabric
3. `02-mortality/` — three clocks, phases, Thanatopsis, genomic bottleneck
4. `03-daimon/`, `04-memory/`, `05-dreams/` — affect, Grimoire, offline cognition
5. `07-tools/` — capability model, tool categories, profiles, sidecar
6. `08-vault/` — ERC-4626 contracts, optional vault layer
7. `09-economy/` — identity, clade, marketplace, coordination
8. `10-safety/` — custody, PolicyCage, defense layers
9. `11-compute/`, `12-inference/` — Fly.io runtime, provider routing, x402
10. `13-runtime/`, `18-interfaces/` — surfaces, TUI, portal
11. `17-monorepo/` — gotts-monorepo TypeScript packages
12. `16-testing/`, `20-styx/` — validation stack, Styx server spec

---

## 11. Technical Reference: Key Formulas

**Vitality formula** (multiplicative, any factor → 0 kills the Golem):

```
composite = sigmoid(economic, 0.3, 10)
          × sigmoid(epistemic, 0.4, 8)
          × max(0, 1.0 - ageFactor × 0.3)
```

**Confidence discount by provenance** (applied at ingestion Stage 4):

| Source | Factor |
|--------|--------|
| Inheritance (generation N) | `0.85^N` |
| Clade sibling | `0.80` |
| Marketplace | `0.60` |
| Lethe (unknown provenance) | `0.50` |

**Pheromone confirmation definition**: A pheromone "confirmation" is a per-agent event where the agent's action produces a profitable outcome (positive PnL after fees). Confirmed pheromones propagate to clade members; unconfirmed ones decay.

**Pheromone decay with confirmation reinforcement**:

```
effective_half_life = base_half_life × (1 + confirmations × 0.5)
intensity(t) = initial × exp(-0.693 × elapsed / effective_half_life)
```

**Knowledge demurrage decay multipliers by domain**:
`gas_mev` 3.0×, `price_direction` 1.5×, `volatility` 0.8×, `yield` 0.5×, `protocol` 0.3×.

**Mortal scoring function** (Styx retrieval ranking):

```
score = α·semantic + β·temporal + γ·quality + δ·provenance + ε·bloodstain
```

L0/L1 weights: α=0.40, β=0.25, γ=0.15, δ=0.15, ε=0.05.
L2 weights: α=0.35, β=0.20, γ=0.25, δ=0.10, ε=0.10.
Bloodstain retrieval boost: 1.2×. Bloodstain decay rate: 0.33× (3× slower than standard WISDOM entries).

---

## Cross-References

- `01-golem/13-runtime-extensions.md` — the full 28-extension registry with dependency graph and hook coverage matrix, organized across 7 layers from Foundation to Integration
- `02-mortality/00-overview.md` — the mortality architecture in depth: three independent death clocks (economic, epistemic, stochastic), five behavioral phases, and the Thanatopsis death protocol
- `04-memory/00-overview.md` — the Grimoire's three-substrate architecture (LanceDB, SQLite, filesystem), the Curator pruning cycle, and the four-factor retrieval scoring formula
- `07-tools/IMPLEMENTATION-PLAN.md` — the golem-tools Rust crate build plan, covering ~210 Alloy-native DeFi tool implementations
- `09-economy/02-clade.md` — the full Clade sync protocol specification: Styx-relayed knowledge sharing, confidence discounting, and sibling coordination
- `10-safety/01-custody.md` — the three custody modes (Delegation, Embedded, LocalKey), seven custom caveat enforcers, and death settlement by mode
- `17-monorepo/00-packages.md` — the gotts-monorepo TypeScript package inventory covering vault contracts, dev tooling, portal, and CLI
- `20-styx/00-architecture.md` — the Styx server specification: three-layer privacy model (Vault/Clade/Lethe), Pheromone Field for stigmergic coordination, and Bloodstain Network for death-indexed knowledge
