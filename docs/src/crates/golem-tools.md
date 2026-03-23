# golem-tools

## What It Is

`golem-tools` is the Layer 4 boundary for tool registration and sandboxed execution. The scaffold reserves the crate for tool traits, registries, and sidecar integration without exposing those APIs yet.

## Features

- Reserved Layer 4 crate for tool traits, registry, Wasmtime sandboxing, and JSON-RPC sidecars
- Crate root documents the intended scope: `ToolDef`, `ToolContext`, `ToolResult`, and tool traits
- Inherits shared workspace dependency and lint policy
- No public Rust items are exported yet

## Getting Started

```bash
cargo check -p golem-tools
```

## Configuration

The scaffold does not yet expose crate-local configuration. The related TypeScript sidecar uses `BARDO_SIDECAR_SOCKET` for its Unix-socket path.

## API

The crate does not expose a public Rust API yet.

```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]
```

## Architecture

`golem-tools` sits in the infrastructure layer so it can integrate with inference, safety, and chain-facing crates without living in any one of them. The scaffold preserves that integration boundary before the tool system is implemented.

## References

- `prd2/17-monorepo/00-packages.md` sections `Crate Inventory` and `TypeScript Sidecar`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `Crate Dependency DAG`, and `Key Dependencies`
- `prd2/shared/dependencies.md` section `8. Rust Workspace Dependencies (bardo-golem-rs)`
