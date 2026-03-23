# bardo-gateway

## What It Is

`bardo-gateway` is an application workspace member reserved for the external gateway process. Today it exposes a minimal Tokio entrypoint that initializes tracing, logs startup, and exits successfully.

## Features

- Binary target named `bardo-gateway`
- Async `main` function using Tokio
- Initializes `tracing_subscriber` on startup
- Reserved app boundary for the future inference gateway and provider router
- Reserved scaffold ports: `8080` for HTTP+SSE traffic and `8081` for WebSocket traffic

## Getting Started

```bash
cargo run -p bardo-gateway
```

## Configuration

The current binary surface has no CLI flags or environment-driven behavior beyond standard
process logging.

The normative port map reserves these ports for the future gateway service:

- `8080` for the HTTP+SSE listener
- `8081` for the WebSocket listener

The current scaffold binary does not bind either port yet; these are reserved/default
allocations for the full implementation.

## API

The binary exposes one entrypoint:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

It initializes tracing with `tracing_subscriber::fmt::init()` and emits a startup banner.

## Architecture

`bardo-gateway` sits in the `apps/` layer rather than the reusable crate graph. The scaffold keeps its package name and executable target stable before the actual gateway transport and routing logic is introduced.

## References

- `prd2/17-monorepo/00-packages.md` sections `Workspace Layout` and `Crate Inventory`
- `prd2/17-monorepo/01-rust-workspace.md` section `Workspace Structure`
- `prd2/shared/port-allocation.md` section `Port Map (Normative)`
