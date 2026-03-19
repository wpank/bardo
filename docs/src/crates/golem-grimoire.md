# golem-grimoire

## What It Is

`golem-grimoire` is the Layer 2 boundary for persistent knowledge storage. In the current scaffold it reserves the crate name, workspace membership, and documentation surface for the Grimoire subsystem without exposing storage types yet.

## Features

- Reserved Layer 2 crate for episodic, semantic, and procedural memory
- Crate root documents the intended storage role: LanceDB, SQLite, and `PLAYBOOK.md`
- Inherits workspace policy for edition, dependencies, and lints
- No public storage API is available yet

## Getting Started

```bash
cargo check -p golem-grimoire
```

## Configuration

The scaffold does not define crate-local configuration yet. Shared toolchain, formatting, and lint behavior come from the workspace root.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-grimoire` sits in the cognition layer beside `golem-heartbeat`, `golem-daimon`, `golem-dreams`, and `golem-context`. The scaffold keeps the crate boundary stable so later storage work can attach to the existing workspace graph.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/shared/dependencies.md` section `8. Rust Workspace Dependencies (bardo-golem-rs)`
