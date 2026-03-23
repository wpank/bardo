# golem-chain

## What It Is

`golem-chain` is the Layer 4 boundary for on-chain connectivity and transaction execution. The scaffold reserves the crate for Alloy-based providers, protocol bindings, and transaction policy integration without exposing those APIs yet.

## Features

- Reserved Layer 4 crate for chain providers, protocol bindings, and simulation helpers
- Crate root documents the intended scope: Alloy, ERC-8004, Permit2, Warden, and `revm`
- Inherits shared workspace toolchain, dependency, and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-chain
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

`golem-chain` sits in the infrastructure layer beside inference, triage, tools, and chain-intelligence work. The scaffold keeps the chain boundary explicit before RPC clients or contract bindings are added.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `Crate Dependency DAG`, and `Key Dependencies`
- `prd2/shared/dependencies.md` section `8. Rust Workspace Dependencies (bardo-golem-rs)`
