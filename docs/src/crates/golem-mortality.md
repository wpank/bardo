# golem-mortality

## What It Is

`golem-mortality` is the Layer 2 boundary for mortality and teardown logic. The scaffold establishes the crate boundary for death clocks, vitality tracking, and Thanatopsis without exposing those systems yet.

## Features

- Reserved Layer 2 crate for economic, epistemic, and stochastic mortality
- Crate root documents the intended scope: death clocks, vitality state, and Thanatopsis
- Inherits shared workspace package metadata and lint policy
- No public mortality API is available yet

## Getting Started

```bash
cargo check -p golem-mortality
```

## Configuration

No crate-specific configuration surface is exposed yet. Shared build and lint behavior comes from the workspace scaffold.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-mortality` sits in the cognition layer so later lifecycle and legacy work can depend on a fixed crate boundary. The scaffold keeps that dependency edge explicit without adding placeholder domain types.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
