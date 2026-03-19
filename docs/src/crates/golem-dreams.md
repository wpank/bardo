# golem-dreams

## What It Is

`golem-dreams` is the Layer 2 boundary for sleep, replay, and consolidation behavior. The scaffold reserves the crate for dream scheduling and memory-processing work without exposing those systems yet.

## Features

- Reserved Layer 2 crate for NREM, REM, and consolidation workflows
- Crate root documents the intended role of the sleep subsystem
- Inherits shared workspace toolchain, dependency, and lint settings
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-dreams
```

## Configuration

The scaffold does not define crate-local configuration yet. Shared policy comes from the workspace root.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-dreams` occupies the cognition layer beside heartbeat, context, and memory. The scaffold keeps the crate boundary fixed so later sleep-cycle work can integrate cleanly with runtime and Grimoire crates.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
