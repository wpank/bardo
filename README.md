![Bardo](demo/cover.png)

# Bardo

A Rust monorepo of tools for building autonomous agents, running cost-efficient LLM inference, and simulating EVM state. The pieces are designed to be used independently or together.

## Tools

### [bardo-gateway](apps/bardo-gateway) — [demo](demo/demo1.mp4)

An HTTP inference proxy that sits in front of Anthropic, OpenAI, and OpenRouter. Drop it in, point your API calls at port 4000, and it handles:

- **Three-layer caching** — exact hash match, semantic similarity (61-68% hit rate at 0.8 cosine threshold), and prompt prefix caching. Most repeated workloads see significant cost reduction without changing your code.
- **Multi-provider routing** — configure fallback order, per-provider rate limits, budget caps. When one provider rate-limits you, it routes to the next.
- **MPP streaming payments** — session-based USDC micropayments via HTTP 402 for metered access.
- **Cost tracking** — per-key spend, model-level breakdowns, dashboard at `/dashboard`.

```bash
ANTHROPIC_API_KEY=sk-... cargo run -p bardo-gateway
# Now proxy your calls through http://localhost:4000
```

It speaks the Anthropic and OpenAI wire formats. Existing SDKs work unchanged — just change the base URL.

---

### [mori](apps/mori) — [demo](demo/demo3.mp4)

A multi-agent build orchestration system. You give it plans and it dispatches Claude agents to implement them in parallel, manages worktrees so agents don't step on each other, runs gate verification after each step, and tracks budget.

The core loop: read the plan, assemble context (AST-indexed, PageRank-ranked, token-budget-optimized), dispatch an implementer agent, run the verifier, gate-review the output, repeat.

```bash
# Start a build run
./bardo-ctl.sh start

# Watch it in the TUI
./bardo-supervisor.sh
```

Supporting crates:

- **[`mori-index`](crates/mori-index)** — Tree-sitter AST parsing, PageRank symbol ranking, HDC fingerprinting, Salsa incremental computation, memory-mapped snapshots. Builds a queryable index of your codebase.
- **[`mori-context`](crates/mori-context)** — Deterministic context assembly. Pulls relevant code, docs, task state, and prior outputs into a context window. Learns optimal budget allocation per task type over time.
- **[`mori-mcp`](crates/mori-mcp)** — MCP server exposing code search and context tools. Any MCP-compatible agent (Claude, etc.) can query your codebase through it.
- **[`mori-service`](apps/mori-service)** — HTTP daemon with event streaming and webhook integration for CI/build system hooks.

---

### [mirage-rs](apps/mirage-rs)

A local EVM fork simulator. Pulls state from a mainnet RPC, lets you speculatively execute transactions against it, and reports the outcome without broadcasting.

```bash
just mirage rpc_url=https://mainnet.example.com
```

Used for pre-flight simulation before any on-chain action. The [`golem-chain`](crates/golem-chain) crate wraps `revm` for programmatic simulation from Rust — fork a block, overlay account state, run a call sequence, inspect the result.

---

### Golems — [demo](demo/demo4.mp4)

A Golem is a mortal autonomous agent compiled as a single Rust binary. It has a wallet, a strategy, a knowledge base, and a finite lifespan. It runs on a VM (local or Fly.io), connects to chain, and makes decisions on every tick of its heartbeat.

Each tick runs a nine-step cycle: observe market state, retrieve relevant memories from the grimoire, appraise the situation through the daimon affect engine (PAD vectors), generate candidate actions via three-tier inference (T0 rule-based → T1 light model → T2 full model), simulate outcomes in mirage-rs, apply safety constraints via PolicyCage, execute the chosen action on-chain, update the grimoire, then reflect.

Golems die. Economic death fires when the USDC balance can no longer sustain inference costs. Epistemic death fires when prediction accuracy drops below threshold. Stochastic death adds entropy. When a Golem reaches terminal state it runs the Thanatopsis succession protocol — transferring learned heuristics and strategy DNA to a successor before shutting down.

The [`golem-runtime`](crates/golem-runtime) crate is the entry point for a Golem binary. It boots extensions in topological order, enforces lifecycle state transitions at compile time via the type-state pattern, and dispatches per-tick and per-block hooks concurrently where possible. The lifecycle is `Provisioning → Active ⇄ Dreaming → Terminal → Dead`. The PolicyCage is sealed at `Provisioning → Active`; it cannot be modified while the Golem is running.

```bash
# Run a Golem locally
bardo-golem start --config bardo.toml

# Connect the terminal to observe it
cargo run -p bardo-terminal -- --golem g-7f3a
```

Core subsystems:

- **[`golem-core`](crates/golem-core)** — foundation types: `GolemId`, `CognitiveTier`, PAD affect vectors, event fabric, HDC primitives
- **[`golem-grimoire`](crates/golem-grimoire)** — three-tier memory: LanceDB episodes, SQLite semantic patterns, procedural playbook
- **[`golem-mortality`](crates/golem-mortality)** — triple-clock death: economic, epistemic, stochastic; Thanatopsis succession
- **[`golem-daimon`](crates/golem-daimon)** — affect engine: PAD vector computation, appraisal triggers, behavioral phase transitions
- **[`golem-inference`](crates/golem-inference)** — cost-aware T0/T1/T2 inference routing through bardo-gateway
- **[`golem-safety`](crates/golem-safety)** — PolicyCage sandboxing, capability-based auth, taint tracking

---

### [bardo-terminal](apps/bardo-terminal)

A TUI for observing what's happening inside a running golem. Ratatui-based, connects over WebSocket, shows the cognitive loop state, memory retrieval, vitality, decisions, and audio-reactive sonification output.

```bash
cargo run -p bardo-terminal
```

---

## Core Libraries

These are the building blocks used by the tools above and available for your own code.

| Crate | What it does |
|---|---|
| [`golem-core`](crates/golem-core) | Foundation types: GolemId, CognitiveTier, PAD affect vectors, event fabric, taint labels, HDC primitives, tick arena allocator |
| [`golem-chain`](crates/golem-chain) | Alloy RPC provider, ERC-8004 agent identity registry, Warden timelock, local revm simulation |
| [`golem-grimoire`](crates/golem-grimoire) | Three-tier memory: LanceDB episode store, SQLite semantic patterns, procedural playbook. Admission gating, decay, memetic fitness tracking |
| [`golem-inference`](crates/golem-inference) | Three-tier model routing (T0 heuristic / T1 light / T2 full), cost-aware cascade escalation, gateway HTTP client |
| [`golem-mortality`](crates/golem-mortality) | Triple-clock mortality for agents: economic (USDC burn), epistemic (prediction accuracy EMA), stochastic. Thanatopsis succession protocol |
| [`golem-sonification`](crates/golem-sonification) | Modular synthesis engine driven by CorticalState. Four-thread architecture: runtime → parameter bridge → rack processor → audio output |
| [`golem-safety`](crates/golem-safety) | Capability-based auth, PolicyCage sandboxing, taint tracking, Merkle audit log *(planned)* |
| [`golem-triage`](crates/golem-triage) | Bayesian event scoring: Gamma-Poisson rate anomalies, BOCPD changepoint detection, MidasR graph anomalies, HDC encoding *(planned)* |
| [`bardo-primitives`](crates/bardo-primitives) | Zero-dependency compute primitives: HdcVector (10,240-bit), InferenceTier, TierRouter |
| [`bardo-inference`](crates/bardo-inference) | Shared inference wire types: InferenceRequest/Response, streaming chunks, roles, error vocabulary |

---

## Getting Started

```bash
git clone <repo> && cd bardo
git config core.hooksPath .githooks
cp .env.example .env        # set ANTHROPIC_API_KEY at minimum
just setup                  # install dev tools
just build
just test
```

**`.env` keys:**

```
ANTHROPIC_API_KEY=...        # required
OPENAI_API_KEY=...           # optional
OPENROUTER_API_KEY=...       # optional
BARDO_GATEWAY_URL=http://127.0.0.1:4000   # defaults to this
MIRAGE_RPC_URL=...           # leave unset for local Anvil
```

**Common commands:**

```bash
just build          # debug build, full workspace
just test           # run tests with nextest
just lint           # clippy, all crates
just fmt            # format
just ci             # fmt-check → lint → test → deny
just mirage rpc_url=URL     # start EVM fork
just coverage       # HTML coverage report
```

---

## Stack

Rust edition 2024. `unsafe` denied workspace-wide. Pinned toolchain via `rust-toolchain.toml`. No external services required to build — EVM simulation falls back to local Anvil, LLM calls require API keys.

Dependencies: `axum`, `alloy`, `revm`, `ratatui`, `rusqlite`, `lancedb`, `tokio`, `tree-sitter`, `salsa`, `moka`.
