# golem-heartbeat

## What It Is

`golem-heartbeat` is the Layer 2 boundary for the golem's decision-cycle engine. At the scaffold stage it defines the crate slot for the heartbeat pipeline and its crate-level documentation, but it does not yet expose heartbeat types or execution logic.

## Features

- Reserved Layer 2 crate for the 9-step heartbeat pipeline
- Crate root documents the intended surface: `CoALA`, `DecisionCycleRecord`, and the heartbeat FSM
- Inherits workspace-wide dependency, toolchain, and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-heartbeat
```

## Configuration

`golem-heartbeat` currently inherits all of its configuration from the workspace scaffold. There are no crate-local settings or environment variables yet.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-heartbeat` occupies the cognition layer in the workspace DAG. The scaffold preserves the crate boundary for the future tick engine without introducing premature dependencies or placeholder domain types.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` sections `Workspace Dependency Inheritance` and `Lint Config`
