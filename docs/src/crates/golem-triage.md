# golem-triage

## What It Is

`golem-triage` is the Layer 4 boundary for novelty and surprise scoring. The scaffold reserves the crate for Bayesian surprise and routing logic without exposing those APIs yet.

## Features

- Reserved Layer 4 crate for surprise scoring and escalation
- Crate root documents the intended scope: Bayesian surprise, HDC/BSC fingerprints, and KL divergence
- Inherits shared workspace toolchain, dependency, and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-triage
```

## Configuration

`golem-triage` currently has no crate-specific configuration surface.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-triage` lives in the infrastructure layer alongside inference and chain-aware analysis crates. The scaffold preserves that separation before any routing or scoring implementation is added.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
