# golem-engagement

## What It Is

`golem-engagement` is the Layer 6 boundary for achievements, recaps, and graveyard-facing events. The scaffold reserves the crate for those engagement surfaces without exposing their APIs yet.

## Features

- Reserved Layer 6 crate for achievements, recap events, and graveyard records
- Crate root documents the intended engagement scope
- Inherits shared workspace package metadata and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-engagement
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

`golem-engagement` lives in the surfaces layer alongside outward transports and visual identity. The scaffold keeps user-facing recap and achievement work separate from runtime and cognition crates.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
