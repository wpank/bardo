# golem-runtime

## What It Is

`golem-runtime` is the Layer 1 boundary for lifecycle orchestration in the Bardo workspace. At the scaffold stage it establishes the crate boundary, crate-level documentation, and workspace policy inheritance for the runtime layer without exposing runtime types yet.

## Features

- Reserved Layer 1 crate for lifecycle orchestration
- Crate root declares the intended runtime scope: extension registry, hook dispatch, `GolemState`, lifecycle FSM, and shutdown handling
- Inherits the workspace edition, rust-version, dependency policy, and lint configuration
- No public modules, types, or functions are exported yet

## Getting Started

```bash
cargo check -p golem-runtime
```

## Configuration

`golem-runtime` currently has no crate-specific configuration surface. It inherits the shared workspace configuration from the root manifest and tooling files.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-runtime` sits directly above `golem-core` in the dependency DAG. The scaffold keeps that layer boundary explicit from the beginning so later runtime work can land without changing workspace membership or layering rules.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` sections `Workspace Dependency Inheritance` and `Lint Config`
