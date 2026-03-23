# golem-daimon

## What It Is

`golem-daimon` is the Layer 2 boundary for affect and appraisal logic. The scaffold establishes the crate slot for PAD-based emotional state, mood processing, and related runtime hooks without exposing those types yet.

## Features

- Reserved Layer 2 crate for PAD affect modeling
- Crate root documents the intended scope: appraisal, somatic markers, mood, and contagion
- Inherits workspace-wide edition, dependency, and lint settings
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-daimon
```

## Configuration

The scaffold provides no crate-local configuration surface yet. Shared policy comes from the workspace root.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-daimon` occupies the cognition layer next to the heartbeat, memory, and context crates. The scaffold exists so emotional-state work can land in a dedicated crate without changing the workspace DAG later.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
