# bardo-compute

## What It Is

`bardo-compute` is an application workspace member reserved for the compute provisioning service. Today it exposes a minimal Tokio entrypoint that initializes tracing, logs startup, and exits successfully.

## Features

- Binary target named `bardo-compute`
- Async `main` function using Tokio
- Initializes `tracing_subscriber` on startup
- Reserved app boundary for the future provisioning and fleet-management service
- Reserved scaffold port: `9090` for the future provisioning API

## Getting Started

```bash
cargo run -p bardo-compute
```

## Configuration

The current binary surface has no CLI flags or crate-local runtime configuration beyond
standard process logging.

The normative port map reserves port `9090` for the compute provisioning API. The current
scaffold binary does not bind that port yet; it remains a reserved/default allocation for the
later implementation.

## API

The binary exposes one entrypoint:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

It initializes tracing with `tracing_subscriber::fmt::init()` and emits a startup banner.

## Architecture

`bardo-compute` lives under `apps/` as a dedicated process boundary. The scaffold keeps the package and executable target stable before provisioning APIs, billing, and fleet management are added.

## References

- `prd2/17-monorepo/00-packages.md` sections `Workspace Layout` and `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` section `Workspace Structure`
- `prd2/shared/port-allocation.md` section `Port Map (Normative)`
