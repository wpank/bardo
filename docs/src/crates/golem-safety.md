# golem-safety

## What It Is

`golem-safety` is the Layer 3 boundary for capability and policy enforcement. The scaffold reserves the crate for safety controls without exposing permit or policy types yet.

## Features

- Reserved Layer 3 crate for capabilities, permits, and policy evaluation
- Crate root documents the intended safety scope
- Inherits shared workspace toolchain, dependency, and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-safety
```

## Configuration

No crate-local configuration surface exists yet. Shared build and lint behavior comes from the workspace root.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-safety` forms the dedicated layer between cognition and infrastructure. The scaffold keeps that layer separate from the start so later enforcement logic can land without changing dependency boundaries.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure` and `Crate Dependency DAG`
- `prd2/17-monorepo/03-conventions.md` sections `Workspace Dependency Inheritance` and `Lint Config`
