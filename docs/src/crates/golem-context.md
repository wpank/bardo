# golem-context

## What It Is

`golem-context` is the Layer 2 boundary for cognitive workspace assembly. In the scaffold it reserves the crate slot for context-policy and workspace-construction logic without exposing those types yet.

## Features

- Reserved Layer 2 crate for `CognitiveWorkspace` and context policy
- Crate root documents the intended assembly role for knowledge, affect, and event context
- Inherits workspace-wide toolchain, dependency, and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-context
```

## Configuration

This scaffolded crate currently relies entirely on workspace-level configuration.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-context` stays in the cognition layer alongside heartbeat, Grimoire, Daimon, dreams, and mortality. The scaffold keeps the dependency boundary stable before the first context types land.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
