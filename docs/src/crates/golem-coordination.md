# golem-coordination

## What It Is

`golem-coordination` is the Layer 5 boundary for clade synchronization and pheromone propagation. The scaffold reserves the crate for cross-golem coordination without exposing those systems yet.

## Features

- Reserved Layer 5 crate for clade sync and pheromone handling
- Crate root documents the intended scope: field client, bloodstain ingestion, and propagation policy
- Inherits shared workspace toolchain, dependency, and lint settings
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-coordination
```

## Configuration

The scaffold does not define crate-local configuration yet.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-coordination` sits above the infrastructure layer and below the outward-facing surfaces. The scaffold keeps that coordination boundary stable before any relay or sync protocol is introduced.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
