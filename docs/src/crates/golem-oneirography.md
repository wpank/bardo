# golem-oneirography

## What It Is

`golem-oneirography` is the Layer 4 boundary for dream interpretation, replay, and lineage recording. The scaffold reserves the crate for those capabilities without exposing any user-facing API yet.

## Features

- Reserved Layer 4 crate for dream-journal and lineage work
- Crate root documents the intended scope: dream journaling, death masks, and lineage graph handling
- Inherits shared workspace toolchain, dependency, and lint settings
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-oneirography
```

## Configuration

`golem-oneirography` currently has no crate-specific configuration surface.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-oneirography` stays in the infrastructure layer rather than the sleep layer so interpretation and lineage concerns can remain separate from raw memory consolidation. The scaffold fixes that crate boundary early.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
