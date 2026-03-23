# What Is Bardo?

Bardo is permissionless infrastructure for mortal autonomous agents in DeFi.

A golem is a finite-lived Rust process that executes a 9-step cognitive loop once per tick: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect. It holds USDC as metabolic substrate. When the balance reaches zero, when epistemic fitness decays below threshold, or when a stochastic mortality draw fires, the golem dies. At death it runs the Thanatopsis protocol: compress its Grimoire, push the inheritance package to the clade, and leave a death mask on-chain.

The successor golem inherits this compressed knowledge. Across generations, the population accumulates judgment that no immortal agent can develop: knowledge distilled under survival pressure.

## Features

- Single Rust workspace with 26 members: 21 library crates and 5 application binaries
- Layered architecture from `golem-core` through `golem-binary`
- Shared Rust 1.85 / edition 2024 toolchain, dependency pins, lint policy, and release profile
- mdBook documentation site under `docs/`
- TypeScript JSON-RPC sidecar for concentrated-liquidity tooling under `sidecar/tools-ts/`
- Local EVM development through `mirage-rs`

## Key Concepts

| Term | Definition |
|---|---|
| Golem | A mortal autonomous DeFi agent compiled as a single Rust binary |
| Grimoire | Persistent knowledge store: LanceDB episodic + SQLite semantic + PLAYBOOK.md procedural |
| Heartbeat | The 9-step autonomous decision cycle |
| Clade | A fleet of sibling golems sharing knowledge via the Styx relay |
| Daimon | The affect engine that maps market events to PAD (Pleasure-Arousal-Dominance) vectors |
| Thanatopsis | The four-phase death protocol: Acceptance, Settlement, Reflection, Legacy |
| Bardo | The transitional state between death and rebirth - the system's philosophical grounding |

## Architecture

```
golem-binary  (single Fly.io VM binary)
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

## Getting Started

Before running the workspace commands below, install Rust 1.85+, `just`, `cargo-nextest`,
and `mdbook`.

- Install `just` with your system package manager so the task runner is available.
- `just setup` is a macOS-oriented convenience recipe: it installs Rust CLI tools and then
  shells out to `brew install sccache lld`.
- Install `mdbook` separately before running `just mdbook`; the current `just setup` recipe
  does not install it.
- If you are provisioning tools manually instead of using `just setup`, install
  `cargo-nextest` before `just test`.

```bash
# Build the workspace
just build

# Run tests
just test

# Build the docs
just mdbook

# Start a local fork for chain-facing development
just mirage rpc_url=https://mainnet.example/rpc
```

## Configuration

The repository root provides the shared configuration contract for every crate:

- `Cargo.toml` defines workspace members, package metadata, dependency pins, lint policy, and build profiles
- `rust-toolchain.toml` pins Rust 1.85 and the required components
- `rustfmt.toml` and `clippy.toml` define formatting and lint behavior
- `.cargo/config.toml` configures linker and compiler-wrapper behavior
- `nextest.toml`, `deny.toml`, and `justfile` define testing, policy, and developer workflows
- `BARDO_SIDECAR_SOCKET` controls the Unix socket used by the TypeScript sidecar and defaults to `/tmp/bardo-tools.sock`

## API

At the workspace level, the current scaffold exposes operational commands rather than shared Rust types:

```text
just build
just test
just lint
just fmt-check
just deny
just mdbook
just mirage rpc_url="<RPC_URL>"
```

The scaffold also standardizes the minimal async entrypoint used by the shell binaries:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

## References

- `prd2/17-monorepo/00-packages.md` sections `Workspace Layout`, `Root Cargo.toml`, `Crate Inventory`, and `Dependency Rules`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `DX Tooling`, `Workspace Dependency Inheritance`, and `Workspace Lints`
- `prd2/17-monorepo/02-build.md` sections `Rust Workspace`, `Testing`, `Linting`, and `Formatting`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
- `prd2/shared/dependencies.md` section `8. Rust Workspace Dependencies (bardo-golem-rs)`
