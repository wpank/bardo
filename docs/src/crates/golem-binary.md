# golem-binary

`golem-binary` packages the entire Bardo golem stack into a single deployable binary for Fly.io. Rather than running multiple processes, a golem VM runs one binary that includes all crates: runtime, heartbeat, cognition, chain, inference, surfaces, and everything else.

## Features

- Single binary deployment for Fly.io VMs
- Links all golem workspace crates
- Reads configuration from environment variables and `golem.toml`
- Starts the runtime, registers extensions, and begins the heartbeat loop

## Getting Started

```bash
# Build the release binary
cargo build -p golem-binary --release

# Run locally (requires config)
./target/release/golem
```

For Fly.io deployment, use the provided `fly.toml` and `Dockerfile` at the workspace root.

## Architecture

`golem-binary` is a thin entrypoint. Its `main` function initializes `GolemConfig`, constructs `ExtensionRegistry`, registers all built-in extensions in dependency order, and hands control to the `golem-runtime` lifecycle FSM. From that point, `golem-runtime` drives the tick loop until the golem dies.
