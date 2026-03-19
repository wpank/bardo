# bardo-styx

## What It Is

`bardo-styx` is an application workspace member reserved for the Styx relay process. Today it exposes a minimal Tokio entrypoint that initializes tracing, logs startup, and exits successfully.

## Features

- Binary target named `bardo-styx`
- Async `main` function using Tokio
- Initializes `tracing_subscriber` on startup
- Reserved app boundary for the future clade relay and knowledge exchange service
- Reserved scaffold port: `8443` for the future local Styx WebSocket listener

## Getting Started

```bash
cargo run -p bardo-styx
```

## Configuration

The current binary surface has no CLI flags or runtime configuration beyond standard process
logging.

The normative port map reserves port `8443` for the Styx relay's local TLS WebSocket surface.
The current scaffold binary does not bind that port yet; it remains a reserved/default
allocation for the later implementation.

## API

The binary exposes one entrypoint:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

It initializes tracing with `tracing_subscriber::fmt::init()` and emits a startup banner.

## Architecture

`bardo-styx` lives under `apps/` as a process boundary rather than a reusable library boundary. The scaffold keeps the executable target stable before relay, persistence, and sync logic are introduced.

## References

- `prd2/17-monorepo/00-packages.md` sections `Workspace Layout` and `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` section `Workspace Structure`
- `prd2/shared/port-allocation.md` section `Port Map (Normative)`
