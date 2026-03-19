# golem-chain-intelligence

## What It Is

`golem-chain-intelligence` is the Layer 4 boundary for chain-observation and protocol-state intelligence. The scaffold reserves the crate for block ingestion and validation logic without exposing those systems yet.

## Features

- Reserved Layer 4 crate for witness and protocol-state analysis
- Crate root documents the intended scope: block ingestion, chain scope, protocol state, and PVS
- Inherits workspace-wide edition, dependency, and lint settings
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-chain-intelligence
```

## Configuration

The scaffold defines no crate-local configuration surface yet.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-chain-intelligence` shares the infrastructure layer with chain access, inference, and analysis crates. The scaffold keeps the observation boundary separate from raw chain access from the start.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
