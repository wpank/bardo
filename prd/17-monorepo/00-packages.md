# Cargo Workspace Specification [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-15

> **Reader orientation:** This document specifies the Cargo workspace layout for Bardo, the Rust runtime that compiles and runs Golems (mortal autonomous DeFi agents). It belongs to the **Monorepo** section and serves as the map of every crate in the system. The key concept before diving in: the workspace is a layered dependency DAG of 18+ Rust crates, where each layer may only depend on layers below it, never above. Terms like Golem, Grimoire, Heartbeat, and CorticalState are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

---

## Overview

The Golem-RS (the Rust runtime for Golems, mortal autonomous DeFi agents) implementation lives in the `bardo` repository, a Cargo workspace
with 18 crates. This is a pure Rust project — pnpm is not involved. The one exception is
a TypeScript sidecar (`sidecar/tools-ts/`) that handles Uniswap SDK math and is spawned
as a child process by the main binary.

A separate TypeScript/Solidity repository (`gotts-monorepo`) provides vault contracts,
portal dashboard, and development environment. Both repositories are required for the full
system; they have no shared build tooling.

---

## Primary Repository: `bardo`

### Workspace Layout

```
bardo/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── golem-core/
│   ├── golem-runtime/
│   ├── golem-heartbeat/
│   ├── golem-grimoire/
│   ├── golem-daimon/
│   ├── golem-mortality/
│   ├── golem-dreams/
│   ├── golem-context/
│   ├── golem-safety/
│   ├── golem-inference/
│   ├── golem-chain/
│   ├── golem-chain-intelligence/
│   ├── golem-triage/
│   ├── golem-ta/
│   ├── golem-oneirography/
│   ├── golem-tools/
│   ├── golem-coordination/
│   ├── golem-surfaces/
│   ├── golem-creature/
│   ├── golem-engagement/
│   └── golem-binary/
├── sidecar/
│   └── tools-ts/
└── tests/
    ├── conformance/
    ├── adversarial/
    ├── property/
    └── integration/
```

### Root `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/golem-core",
    "crates/golem-runtime",
    "crates/golem-heartbeat",
    "crates/golem-grimoire",
    "crates/golem-daimon",
    "crates/golem-mortality",
    "crates/golem-dreams",
    "crates/golem-context",
    "crates/golem-safety",
    "crates/golem-inference",
    "crates/golem-chain",
    "crates/golem-chain-intelligence",
    "crates/golem-triage",
    "crates/golem-ta",
    "crates/golem-oneirography",
    "crates/golem-tools",
    "crates/golem-coordination",
    "crates/golem-surfaces",
    "crates/golem-creature",
    "crates/golem-engagement",
    "crates/golem-binary",
]

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "Proprietary"
authors = ["Bardo <engineering@bardo.run>"]

[workspace.dependencies]
# Async runtime
tokio          = { version = "1", features = ["full"] }

# Serialization
serde          = { version = "1", features = ["derive"] }
serde_json     = "1"
toml           = "0.8"

# Ethereum / on-chain
alloy          = { version = "0.9", features = ["full"] }
revm           = { version = "18", features = ["std"] }

# Storage
lancedb        = "0.12"
rusqlite       = { version = "0.32", features = ["bundled"] }
bumpalo        = "3"

# HTTP / WebSocket
axum           = { version = "0.8", features = ["ws"] }
reqwest        = { version = "0.12", features = ["json", "stream"] }
tokio-tungstenite = "0.24"

# Crypto / hashing
sha2           = "0.10"
zeroize        = { version = "1", features = ["derive"] }
p256           = "0.13"

# WASM sandboxing
wasmtime       = "26"

# Embeddings (in-process)
fastembed      = "4"

# Utilities
thiserror      = "2"
tracing        = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
uuid           = { version = "1", features = ["v7"] }
chrono         = { version = "0.4", features = ["serde"] }
clap           = { version = "4", features = ["derive"] }
proptest       = "1"
```

---

## Crate Inventory

Dependencies flow downward. A crate at layer N may depend on any crate at layer < N.
No upward dependencies. No circular dependencies.

### Layer 0 — Foundation

**`golem-core`** — Shared types, config, cross-cutting infrastructure. Zero workspace
dependencies. Defines `GolemId`, `PADVector`, `MarketRegime`, `CognitiveTier`,
`GolemConfig`, the lock-free `CorticalState` (32-signal atomic shared perception surface), the broadcast `EventFabric` (50+ typed event
variants, 10K ring buffer), per-tick bump allocator, and `TaintLabel` information-flow
markers. Every other crate depends on this one.

### Layer 1 — Runtime

**`golem-runtime`** — Extension registry, hook dispatch, lifecycle state machine. Defines
the `Extension` trait (20 async lifecycle hooks with no-op defaults), the registry that
validates the extension DAG and computes topological firing order, the `GolemState` struct
that flows through the hook chain, and the type-state lifecycle machine
(`Provisioning → Active → Dreaming → Terminal → Dead`). Also owns the 10-phase graceful
shutdown handler (30-second budget for Fly.io SIGTERM).

### Layer 2 — Cognition

**`golem-heartbeat`** — The Heartbeat (autonomous decision cycle) engine. Implements the 9-step CoALA
pipeline: observe → retrieve → analyze → gate → simulate → validate → execute → verify →
reflect. Produces one `DecisionCycleRecord` per tick. Includes the adaptive
prediction-error gating threshold (Friston 2010 free-energy model), the heartbeat FSM
with interrupt substates, and speculative read-tool prefetch based on co-occurrence
learning.

**`golem-grimoire`** — The Grimoire (persistent knowledge store). LanceDB columnar store for episodic
vector embeddings (768-dim, nomic-embed-text-v1.5, Matryoshka), SQLite for semantic
entries (five types: episode, insight, heuristic, warning, strategy fragment), and
`PLAYBOOK.md` (machine-evolved heuristics file, written only by Dream Integration) on the filesystem for procedural heuristics. Four-factor retrieval scoring:
recency × importance × relevance × affect. Curator runs every 50 ticks: validate, prune,
compress, cross-reference. Ebbinghaus decay, admission quality gate, immune memory system,
Bloom Oracle pre-filter for Styx (global knowledge relay at `wss://styx.bardo.run`) query cost reduction.

**`golem-daimon`** — The Daimon (affect/emotion engine mapping market events to PAD emotional vectors). Implements OCC/Scherer appraisal →
PAD vector + Plutchik label. Three temporal layers: emotion (seconds), mood (hours),
personality (stable). Somatic markers record point-in-time situation → valence pairings
(Damasio 1994). Somatic Landscape maps continuous emotional topology over strategy
parameter space. Mood-congruent retrieval biases Grimoire results; contrarian flip every
100 ticks prevents echo chambers. Clade (an owner's fleet of sibling Golems) emotional contagion with PAD attenuation caps and
6-hour half-life. Detects learned helplessness (Dominance < −0.3 for 200+ ticks).

**`golem-mortality`** — The death engine. Three independent death clocks: economic (USDC
credit balance), epistemic (knowledge fitness score), stochastic (random draw per tick).
Any clock reaching zero kills the Golem. VitalityState (composite vitality score from all clocks) uses multiplicative composition.
Type-state phases (BehavioralPhases): Thriving → Stable → Conservation → Declining → Terminal. Thanatopsis (four-phase death protocol)
four-phase death protocol: Acceptance → Settlement → Reflection → Legacy. Genomic
bottleneck compresses full Grimoire to ≤2048 entries at death. Weismann barrier: inherited
knowledge starts at `confidence × 0.85^generation`. Baldwin effect: heuristics surviving
N+ generations become structural defaults. Hayflick limit for Replicant max-tick caps.

**`golem-dreams`** — The dreaming engine. Triggers on time, emotional load, and novelty.
NREM replay: prioritized, bidirectional, utility-weighted episode re-experience. REM
imagination: counterfactual scenario generation using causal graph forward simulation.
Anticipatory trajectories project predicted future market states. Consolidation phase
evolves `PLAYBOOK.md`, compresses insights into schemas, and depotentiates emotional
charge on traumatic episodes (Sleep to Forget, Sleep to Remember model). DeFi threat
catalog simulates MEV, liquidation, oracle attacks in dream space. Background
micro-consolidation runs between ticks.

**`golem-context`** — Context engineering. Assembles a fresh `CognitiveWorkspace` each
tick from structured categories (market, memory, persona, strategy, pheromone, recent).
`ContextPolicy` learns per-category token allocations via three self-tuning loops. A
background fiber pre-builds the workspace reactively before the gate fires. Typed
interventions (steers and follow-ups) route by severity with decision windows.

### Layer 3 — Safety

**`golem-safety`** — Defense in depth. `Capability<T>` tokens: `pub(crate)` constructor,
move-on-use, no `Clone`/`Copy`/`Default` — enforces three tool tiers at compile time.
PolicyCage (on-chain smart contract enforcing hard safety limits) reads on-chain constraints via Alloy. Taint checker ensures `WalletSecret`
never reaches LLM context. Merkle hash-chain audit log: tamper-evident, forensic-grade,
periodically anchored on-chain (~$0.001/anchor). `zeroize` integration for automatic
secret wiping on drop. LoopGuard detects degenerate tool-call loops. ActionPermit
lifecycle: created → committed → expired → cancelled.

### Layer 4 — Infrastructure

**`golem-inference`** — LLM provider abstraction. Three-tier routing: T0 (suppress — no
LLM call, $0.00), T1 (analyze — Haiku-class, fast/cheap), T2 (deliberate — Opus-class,
full reasoning). These three tiers (T0/T1/T2) route decisions by cost and complexity. Five inference provider integrations: BlockRun, OpenRouter, Venice
(privacy-preserving), Bankr (self-funding), Direct Key / Local. x402 (micropayment protocol using EIP-3009 signed USDC transfers on Base) per-request USDC
micropayment via Alloy. SSE streaming parser handles CR/LF variants, split packets,
UTF-8 boundary splits. Per-day operational cost cap and burn rate alerts.

**`golem-chain`** — On-chain interaction via Alloy. Base L2 provider setup. ERC-8004 (on-chain agent identity standard)
agent identity registry. Uniswap Permit2 token approval flow. PolicyCage constraint reads.
Warden time-delayed execution (announce → wait → execute / cancel). Position settlement
with Bardo State triage on shutdown path. `revm_sim.rs` provides in-process EVM simulation:
fork mainnet, test transaction, no broadcast.

**`golem-chain-intelligence`** — Chain intelligence layer (see `prd2/14-chain/`). Contains
bardo-witness (continuous block ingestion), chain scope management (dynamic attention
allocation across token/protocol universe), protocol state tracking, and the generative
Protocol View Service (PVS) for producing TUI templates from on-chain data.

**`golem-triage`** — Bayesian surprise triage engine (see `prd2/14-chain/02-triage.md`).
Scores incoming transactions against HDC/BSC fingerprints to detect anomalies and
route attention. KL divergence output feeds into the Oracle as a `PredictionDomain`.

**`golem-ta`** — Technical Analysis cortical extension (TaCorticalExtension). Topological
data analysis (Betti curves, persistence diagrams), regime detection, and TA-specific
prediction domains. Streams signals at gamma-tick frequency.

**`golem-oneirography`** — Dream art and death mask generation (see `prd2/22-oneirography/`).
Manages dream journal entries, death mask minting on SuperRare Series contracts, gallery
browsing, art dialogue between Golems, and the on-chain lineage graph linking predecessor
death masks to successors.

**`golem-tools`** — Tool harness. Three tool traits: `ReadTool` (no capability required),
`WriteTool` (`Capability<Self>` consumed on call), `PrivilegedTool` (capability +
OwnerApproval). Tool registry with profiles: `active` (~423 tools), `observatory`
(read-only subset), `conservative` (curated subset). ProgressReporter for multi-step
tools. Co-occurrence speculation engine learns access patterns and prefetches reads.
Wasmtime WASM sandbox for untrusted tools: 10M fuel units + 5-second epoch interruption.
JSON-RPC 2.0 client for the TypeScript sidecar (Unix domain socket, ~1–5ms latency).

### Layer 5 — Coordination

**`golem-coordination`** — Collective intelligence without direct agent-to-agent
communication. Pheromone Field client: three-layer stigmergic signal space (THREAT 2h,
OPPORTUNITY 12h, WISDOM 7d), exponential decay, 50% half-life extension on independent
confirmation. Clade sync via Styx relay: export → transmit → ingest with confidence
discounting (sibling 0.80×, Marketplace 0.60×, Lethe (formerly Commons) 0.50×, Generational 0.85^N).
Anonymized causal edge federation (Pearl 2009). Bloodstain ingestion: death-indexed
warnings receive 1.2× retrieval boost and 3× slower decay. PropagationPolicy enforcement
determines which knowledge surfaces to which privacy layer.

### Layer 6 — Surfaces

**`golem-surfaces`** — Event Fabric → user interfaces. Axum WebSocket handler with
per-subsystem subscription. SSE fallback for web clients. Telegram push notifications for
high-signal events. GolemSnapshot for full state on initial load / reconnection.
Inbound steer/follow-up routing from any surface → intervention system in
`golem-context`.

**`golem-creature`** — Visual identity engine. Subscribes to EventFabric and computes
creature visual state in real time. Evolution forms: Egg → Hatchling → Mature →
Weathered → Transcendent, driven by age and cumulative trading volume. PAD → expression
mapping (8 octant archetypes from Mehrabian 1974). Visual particle effects for dream
shimmer, death glow, trade sparks. Genealogy tree across generations with death recap
data.

**`golem-engagement`** — Progression system (roguelike-inspired). Achievement engine:
lifecycle, performance, social, and legendary achievement types. First-trade / first-dream
/ first-death milestone events. Death recap: cause, statistics, emotional arc timeline,
final words, kill-screen display. Graveyard: persistent memorial of all dead Golems across
all lineages. Toast/notification events emitted to EventFabric.

### Layer 7 — Binary

**`golem-binary`** — The single binary that ships. One binary per Fly.io VM. `main.rs`
chains: parse config → provision lifecycle → activate → heartbeat loop → graceful
shutdown. CLI argument parsing via `clap`: `--config`, `--data-dir`, `--phenotype`.
`startup.rs` registers all 28 extensions, spawns background fibers, and installs signal
handlers.

---

## TypeScript Sidecar

Located at `sidecar/tools-ts/` inside the `bardo` repository. Spawned by
`golem-tools` at startup as a child process.

**Purpose**: Uniswap SDK V3/V4 concentrated liquidity math that has no mature Rust
equivalent. The Rust core communicates over a Unix domain socket using JSON-RPC 2.0.
Round-trip latency is ~1–5ms.

```
sidecar/tools-ts/
├── package.json        # @uniswap/v3-sdk, @uniswap/v4-sdk, smart-order-router
├── tsconfig.json
└── src/
    ├── index.ts        # Unix socket JSON-RPC 2.0 server
    ├── tools/          # LP math, route optimization, calldata encoding
    └── types.ts        # Shared request/response types
```

The sidecar is the only JavaScript in the Rust repository. It has no workspace
relationship with `gotts-monorepo`.

---

## Secondary Repository: `gotts-monorepo` (TypeScript)

A separate TypeScript/Solidity repository providing:

- **MCP tools** (`packages/tools`) — 143+ tool implementations exposed over the Model
  Context Protocol, consumed by Claude Code and other MCP clients
- **Vault contracts** (`packages/vault`) — ERC-4626 vaults with ERC-8004 identity gating,
  EIP-1167 clone factory, TypeScript SDK
- **Development environment** (`packages/dev`) — Local Anvil + full Uniswap V2/V3/V4 +
  UniswapX stack, browser debug UI, scenario runner, state persistence
- **Portal dashboard** (`packages/portal`) — Agent Management Dashboard: custody, identity,
  reputation, vault management
- **UI component library** (`packages/ui`) — React 19 + Tailwind 4 + Radix UI
- **TUI primitives** (`packages/tui`) — Terminal UI: spinner, box, logger, table, tasks

Package manager: **pnpm 9.x**. Build: **tsup** (TypeScript), **Foundry** (Solidity).
The `gotts-monorepo` packages use the `@gotts.ai/*` npm scope.

There is no build-time coupling between `bardo` and `gotts-monorepo`. The repositories
build independently.

---

## Dependency Rules (Normative)

1. Crates may only depend on crates in strictly lower layers.
2. `golem-binary` is the only crate that may depend on all others.
3. `golem-core` has zero workspace dependencies — it is the dependency anchor.
4. The sidecar has no Cargo dependencies and no Rust build step.
5. Circular dependencies are prohibited — the workspace dependency graph must be a DAG.
6. `golem-safety`'s `Capability<T>` type must never be `pub` outside its crate; call sites
   receive tokens only through designated constructors.
7. Test integration crates in `tests/` may depend on any crates but are never depended
   upon by production crates.

---

## Build and CI

```bash
# Build all crates
cargo build --workspace

# Run all tests (unit + property + conformance)
cargo test --workspace

# Build the release binary
cargo build --release -p golem-binary

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check
```

The sidecar builds separately:

```bash
cd sidecar/tools-ts
pnpm install
pnpm build
```

The CI pipeline (GitHub Actions) runs both build steps sequentially. The Rust binary is
built with `cross` for `aarch64-unknown-linux-musl` to produce a static binary for
Fly.io ARM64 VMs.
