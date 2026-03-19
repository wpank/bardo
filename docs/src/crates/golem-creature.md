# golem-creature

## What It Is

`golem-creature` is the Layer 6 boundary for visual identity and expression mapping. The scaffold reserves the crate for creature rendering and form evolution without exposing those APIs yet.

## Features

- Reserved Layer 6 crate for visual identity and evolution forms
- Crate root documents the intended scope: creature state, forms, and PAD-to-expression mapping
- Inherits shared workspace toolchain, dependency, and lint settings
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-creature
```

## Configuration

This scaffolded crate currently relies on workspace-level configuration only.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-creature` occupies the surfaces layer beside engagement and external transports. The scaffold keeps visual identity concerns separate from runtime and cognition crates before any rendering model is added.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
