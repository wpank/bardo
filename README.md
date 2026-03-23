# Bardo

Permissionless infrastructure for mortal autonomous agents in DeFi.

## What is this

A golem is a finite-lived Rust process that manages a DeFi portfolio autonomously. Each tick it runs a 9-step cognitive loop: observe the market, retrieve relevant memory, analyze conditions, gate the decision (heuristic vs. LLM), simulate the outcome, validate safety constraints, execute on-chain, verify the result, reflect on what happened. It burns USDC as metabolic substrate. When the balance hits zero, when epistemic fitness decays past threshold, or when a stochastic mortality draw fires, it dies.

Death is not failure — it's the mechanism. At death the golem runs Thanatopsis: compress its Grimoire (episodic memory + learned heuristics), push the inheritance package to its clade, leave a death mask on-chain. The successor inherits this compressed knowledge. Across generations, the population accumulates judgment that no immortal agent can develop: knowledge distilled under survival pressure.

The workspace is a Rust monorepo. 27 library crates organized into 7 dependency layers, 7 application binaries, and a build orchestration system called Mori.

## Architecture

```
golem-binary  (single VM binary)
  └── golem-runtime  (extension registry, lifecycle FSM)
        ├── golem-heartbeat  (9-step tick pipeline)
        │     ├── golem-context  (CognitiveWorkspace assembly)
        │     │     ├── golem-grimoire  (LanceDB + SQLite + PLAYBOOK.md)
        │     │     ├── golem-daimon  (PAD affect engine)
        │     │     └── golem-core  [foundation]
        │     ├── golem-safety  (Capability<T>, PolicyCage, audit log)
        │     ├── golem-tools  (tool registry, Wasmtime sandbox)
        │     ├── golem-inference  (T0/T1/T2 routing, x402)
        │     └── golem-core
        ├── golem-mortality  (three clocks, thanatopsis)
        ├── golem-dreams  (NREM/REM/consolidation)
        ├── golem-coordination  (pheromone field, clade sync)
        ├── golem-chain  (Alloy, ERC-8004, Warden, revm)
        ├── golem-chain-intelligence  (bardo-witness, PVS)
        ├── golem-triage  (Bayesian surprise)
        ├── golem-ta  (TDA, regime detection)
        ├── golem-surfaces  (WebSocket, SSE, Telegram)
        ├── golem-creature  (visual identity engine)
        ├── golem-engagement  (achievements, graveyard)
        └── golem-core  [zero workspace deps]
```

## Workspace layout

### Foundation

- **`bardo-primitives`** — Pure compute primitives with zero internal dependencies
- **`bardo-inference`** — Inference protocol types shared between gateway and golem-inference

### Golem runtime (layers 0–7)

**Layer 0** — `golem-core`: foundational types (`GolemId`, `CognitiveTier`, `PadVector`, `BehavioralPhase`, `PlutchikEmotion`), `GolemConfig`, `TickArena` allocator, error types

**Layer 1** — `golem-runtime`: extension registry, hook dispatch, lifecycle FSM (Provisioning → Active → Dreaming → Terminal → Dead)

**Layer 2** — Subsystem implementations:
- `golem-heartbeat` — 9-step autonomous decision cycle
- `golem-grimoire` — Persistent knowledge store (LanceDB episodic + SQLite semantic + BM25 full-text)
- `golem-daimon` — Affect engine mapping market events to PAD vectors and Plutchik emotions
- `golem-mortality` — Three mortality clocks (economic, epistemic, stochastic) and Thanatopsis death protocol
- `golem-dreams` — Offline knowledge consolidation (NREM replay, REM imagination, integration)
- `golem-context` — CognitiveWorkspace assembly for each tick

**Layer 3** — `golem-safety`: capability-based authorization, `PolicyCage` sandbox, audit logging

**Layer 4** — Decision engines and data sources:
- `golem-inference` — Three-tier inference routing (T0 heuristics, T1 lightweight LLM, T2 full workspace)
- `golem-chain` — Ethereum interaction via Alloy, ERC-8004 identity, revm simulation
- `golem-chain-intelligence` — On-chain event monitoring and proof verification
- `golem-triage` — Bayesian surprise detection for event prioritization
- `golem-ta` — Topological data analysis and market regime detection
- `golem-oneirography` — Dream content analysis and pattern extraction
- `golem-tools` — Tool registry with Wasmtime sandboxed execution

**Layer 5** — `golem-coordination`: pheromone field (stigmergy-based inter-agent signaling), clade knowledge sync

**Layer 6** — Presentation:
- `golem-surfaces` — WebSocket, SSE, and messaging connector layer
- `golem-creature` — Procedural visual identity tied to emotional state
- `golem-engagement` — Achievement tracking and graveyard (death records)
- `golem-sonification` — Audio representation of golem state (optional `sound` feature)

**Layer 7** — `golem-binary`: the compiled golem executable

### Applications

- **`bardo-gateway`** — HTTP API server: MPP payment protocol, inference proxy, embedding service. Axum on port 4000.
- **`bardo-terminal`** — TUI for golem management built with ratatui. 60fps rendering, WebSocket event subscription, sprite system, optional sound.
- **`bardo-styx`** — Global knowledge relay and persistence. Three privacy layers: Vault (private), Clade (shared), Lethe (public-anonymized).
- **`bardo-compute`** — Batch compute orchestration.
- **`mirage-rs`** — EVM simulator for local development. Drop-in Anvil replacement with fork support.

### Mori (build orchestration)

- **`mori-index`** — Codebase indexing (tree-sitter AST, tantivy full-text, memmap'd indexes)
- **`mori-context`** — Context assembly for agent-driven builds
- **`mori-mcp`** — MCP server exposing mori capabilities
- **`mori`** — Main orchestrator binary
- **`mori-service`** — Background service daemon

### Test infrastructure

- **`tests/harness`** — Shared test fixtures and integration test utilities

## Prerequisites

- **Rust 1.85+** — Pinned via `rust-toolchain.toml`. The toolchain file also pulls `rustfmt`, `clippy`, and `llvm-tools-preview`.
- **[just](https://github.com/casey/just)** — Task runner. Install with your system package manager.
- **Foundry** — For local EVM development (`anvil`). Only needed if you're working on chain-facing code without setting `MIRAGE_RPC_URL`.

Optional but recommended:
- `cargo-nextest` — Parallel test runner (required by `just test`)
- `sccache` — Compile cache
- `mdbook` — For building the documentation site

## Getting started

```bash
git clone <repo-url> && cd bardo

# Install git hooks
git config core.hooksPath .githooks

# Set up environment
cp .env.example .env
# Fill in at minimum ANTHROPIC_API_KEY

# Install dev tools (macOS — runs cargo binstall + brew)
just setup

# Build the workspace
just build

# Run tests
just test
```

## Development

```bash
just build              # Debug build, full workspace
just build-release      # Release build of golem-binary
just test               # Run all tests with nextest
just lint               # Clippy on all crates, all features
just fmt                # Format all code
just watch              # Continuous clippy via bacon
just docs               # Build and open rustdoc
just mdbook             # Build mdbook site (docs/)
just ci                 # Full CI sequence: fmt-check → lint → test → deny
just audit              # Deep audit: fmt, lint, deny, unused deps, TOML check
just mirage rpc_url=URL # Start local EVM fork
just run-debug          # Run golem-binary with RUST_LOG=debug
just unused-deps        # Find unused dependencies (cargo-machete)
just coverage           # Generate HTML coverage report
```

## Configuration

Copy `.env.example` to `.env`. The supervisor and mori scripts auto-source it at startup.

**LLM keys** — At minimum set `ANTHROPIC_API_KEY`. Optional backup keys (`_2`, `_3`) enable rate-limit failover. `OPENAI_API_KEY` and `OPENROUTER_API_KEY` are optional.

**Gateway** — `BARDO_GATEWAY_URL` defaults to `http://127.0.0.1:4000`. The gateway generates a random API key on startup if `BARDO_GATEWAY_API_KEY` is unset.

**RPC / Mirage** — Leave `MIRAGE_RPC_URL` unset to auto-start a local Anvil instance. Set it to a mainnet RPC endpoint (e.g. Alchemy) for fork-mode development. `MIRAGE_FORK_BLOCK` pins to a specific block for deterministic replay.

**Timeouts** — `CODEX_TIMEOUT` (default 3600s) and `CARGO_TEST_TIMEOUT` (default 900s) bound long-running operations.

## Key concepts

**Golem** — A mortal autonomous DeFi agent. Single Rust binary deployed to a micro VM. Three cognitive tiers: T0 (heuristics), T1 (lightweight LLM), T2 (full workspace reasoning).

**Heartbeat** — The golem's fundamental tick cycle. 9 steps from market observation through on-chain execution to reflection. Adaptive gating decides which cognitive tier fires.

**Grimoire** — Persistent knowledge store. Episodic memory in LanceDB (vector search), semantic knowledge in SQLite, procedural knowledge in PLAYBOOK.md, full-text search via tantivy.

**Daimon** — The affect engine. Computes a PAD (Pleasure-Arousal-Dominance) vector from episode outcomes, survival pressure, and market conditions. Maps to Plutchik emotions (Joy, Fear, Anger, etc.) that gate decision-making.

**Clade** — A cooperative group of golems sharing knowledge through Styx. Quality gates prevent noise: shared insights start at 0.3 confidence and must be validated independently.

**Thanatopsis** — The four-phase death protocol: Acceptance (acknowledge terminal state), Settlement (close positions), Reflection (compress Grimoire), Legacy (push inheritance to clade, write death mask on-chain).

**Bardo** — The transitional state between death and rebirth. The system's philosophical grounding: every golem exists in passage.

## Toolchain and lint policy

The workspace uses Rust edition 2024 with MSRV 1.85. Key policies:

- `unsafe_code = "deny"` — No unsafe anywhere in the workspace
- Clippy pedantic and nursery lints enabled as warnings
- `unwrap_used = "deny"` — Must handle errors explicitly
- Release profile: thin LTO, single codegen unit, symbols stripped, panic = abort
- Dependency auditing via `cargo-deny` (license and advisory checks)

Formatting is enforced by `rustfmt.toml`, TOML consistency by `taplo.toml`.

## Documentation

The `docs/` directory contains an mdbook site covering architecture, individual crate documentation, and application guides.

```bash
just mdbook   # Build the mdbook site
just docs     # Build and open rustdoc for all crates
```
