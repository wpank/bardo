# golem-inference

## What It Is

`golem-inference` is the Layer 4 boundary for model routing and provider integration. The scaffold reserves the crate for inference clients, tier routing, and budget enforcement without exposing those APIs yet.

## Features

- Reserved Layer 4 crate for inference client abstractions and routing
- Crate root documents the intended role: T0/T1/T2 routing, providers, x402, and SSE parsing
- Inherits shared workspace policy for dependencies, lints, and toolchain
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-inference
```

## Configuration

`golem-inference` currently has no crate-specific configuration surface.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-inference` lives in the infrastructure layer with chain access, tools, and higher-level analysis crates. The scaffold keeps the inference boundary explicit before provider integrations are added.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `Crate Dependency DAG`, and `Key Dependencies`
- `prd2/shared/dependencies.md` section `8. Rust Workspace Dependencies (bardo-golem-rs)`
