# golem-ta

## What It Is

`golem-ta` is the Layer 4 boundary for technical-analysis and regime-detection logic. The scaffold reserves the crate for statistical and topological market analysis without exposing those APIs yet.

## Features

- Reserved Layer 4 crate for regime detection and TDA
- Crate root documents the intended scope: Betti curves, persistence diagrams, and market regime analysis
- Inherits shared workspace package metadata and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-ta
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

`golem-ta` lives in the infrastructure layer beside inference, tools, and chain-aware analysis crates. The scaffold keeps the analytical boundary stable before the first indicators or topological features are added.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
