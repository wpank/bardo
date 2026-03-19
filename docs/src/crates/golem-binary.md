# golem-binary

## What It Is

`golem-binary` is the Layer 7 binary entrypoint package for the workspace. Today it exposes a minimal Tokio startup surface that initializes tracing, emits a startup log line, and exits successfully.

## Features

- Binary crate with the `bardo-golem` executable target
- Async `main` function using Tokio
- Initializes `tracing_subscriber` on startup
- Inherits shared workspace package metadata and lint policy

## Getting Started

```bash
cargo run -p golem-binary
```

## Configuration

The current binary surface has no CLI arguments or crate-local configuration. It inherits shared build settings from the workspace root.

## API

The binary exposes one entrypoint:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

It initializes tracing with `tracing_subscriber::fmt::init()` and emits a startup banner.

## Architecture

`golem-binary` sits at the top of the workspace DAG above every library crate. The scaffold keeps the shipped executable target and package name stable before the full runtime startup sequence is introduced.

## References

- `prd2/17-monorepo/00-packages.md` section `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `Crate Dependency DAG`, and `Release Profile`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
