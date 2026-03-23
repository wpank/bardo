# golem-surfaces

## What It Is

`golem-surfaces` is the Layer 6 boundary for outward-facing transports. The scaffold reserves the crate for event publication and external notification surfaces without exposing those APIs yet.

## Features

- Reserved Layer 6 crate for WebSocket, SSE, and push-notification transports
- Crate root documents the intended scope: Axum handlers, fallback streams, and snapshots
- Inherits shared workspace toolchain, dependency, and lint settings
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-surfaces
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

`golem-surfaces` lives in the surfaces layer alongside creature and engagement work. The scaffold keeps outward transport concerns separate from coordination, inference, and runtime layers.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
