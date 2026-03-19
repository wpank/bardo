# What Is Bardo?

Bardo is permissionless infrastructure for mortal autonomous agents in DeFi.

A **golem** is a finite-lived Rust process that executes a 9-step cognitive loop - observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect - once per tick. It holds USDC as metabolic substrate. When the USDC balance reaches zero, or when epistemic fitness decays below threshold, or when a stochastic mortality draw fires, the golem dies. At death it runs the Thanatopsis protocol: compress its Grimoire to at most 2048 entries, push to the clade, and leave a death mask on-chain.

The successor golem inherits this compressed knowledge. Across generations, the population accumulates judgment that no immortal agent can develop: knowledge distilled under survival pressure.

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

```bash
# Check the workspace compiles
just build

# Run tests
just test

# Build the docs
just mdbook

# Start a dev fork (requires an upstream RPC URL)
just mirage rpc_url=https://mainnet.example/rpc
```

## Workspace Scaffold

The repository root is a Cargo workspace with 26 members: 21 library crates and 5 app binaries.
The workspace manifest pins edition 2024, Rust 1.85, shared dependency versions, workspace-wide
lints, and release settings. The mdBook site lives under `docs/`, and the TypeScript sidecar lives
under `sidecar/tools-ts/` outside the Cargo workspace.

## References

- `prd2/17-monorepo/00-packages.md` sections `Workspace Layout`, `Root Cargo.toml`, `Crate Inventory`, and `Dependency Rules`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `DX Tooling`, `Workspace Dependency Inheritance`, and `Workspace Lints`
- `prd2/17-monorepo/02-build.md` sections `Rust Workspace`, `Testing`, `Linting`, and `Formatting`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
- `prd2/shared/dependencies.md` section `8. Rust Workspace Dependencies (bardo-golem-rs)`
