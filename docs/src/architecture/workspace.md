# Workspace Overview

Bardo is organized as a single Cargo workspace plus a small TypeScript sidecar. The workspace root is the operational contract for the repository: it fixes crate membership, dependency versions, lint policy, build profiles, local toolchain behavior, and the mdBook chapter structure that later crate documentation fills in.

## Features

- 26 workspace members: 21 library crates and 5 app binaries
- Layered crate graph from `golem-core` at Layer 0 through `golem-binary` at Layer 7
- Shared dependency pins and workspace lint policy inherited by every Rust crate
- Reproducible Rust 1.85 / edition 2024 toolchain configuration
- `just` task aliases for build, test, lint, docs, coverage, and release packaging
- mdBook documentation scaffold under `docs/`
- TypeScript sidecar scaffold under `sidecar/tools-ts/`
- Minimal async startup surface for the scaffolded binaries

## Getting Started

Before using the task aliases below, install Rust 1.85+, `just`, `cargo-nextest`, and
`mdbook`.

- Install `just` first so the repository task runner is available.
- `just setup` is a convenience recipe for the current macOS/Homebrew flow: it installs the
  Rust CLI tools and then runs `brew install sccache lld`.
- Install `mdbook` separately for `just mdbook`; the current `just setup` recipe does not
  install it.
- If you are provisioning tools manually, install `cargo-nextest` for `just test`.

```bash
# Verify the workspace compiles
just build

# Run the Rust test suite
just test

# Check formatting, linting, and release tooling
just fmt-check
just lint
just deny

# Build the GitBook-style docs
just mdbook

# Run the local mirage fork sidecar
just mirage rpc_url=https://mainnet.example/rpc

# Start one of the scaffolded binaries
cargo run -p bardo-gateway
```

## Configuration

| File | Purpose |
|------|---------|
| `Cargo.toml` | Defines workspace members, shared dependencies, package metadata, lint policy, and release profile |
| `rust-toolchain.toml` | Pins Rust 1.85 and the required components (`rustfmt`, `clippy`, `llvm-tools-preview`) |
| `rustfmt.toml` | Sets edition 2024 formatting defaults |
| `clippy.toml` | Holds tool-specific config; workspace lint policy lives in the root manifest |
| `.cargo/config.toml` | Sets the Linux linker and optional `sccache` wrapper hook |
| `nextest.toml` | Configures workspace test execution |
| `deny.toml` | Declares the baseline license and advisory policy |
| `justfile` | Provides the build, test, lint, docs, and release task aliases |
| `docs/book.toml` | Configures the mdBook site |
| `sidecar/tools-ts/package.json` | Declares the TypeScript sidecar package and scripts |
| `sidecar/tools-ts/tsconfig.json` | Defines the sidecar compiler settings |
| `sidecar/tools-ts/src/types.ts` | Defines the JSON-RPC request/response types |
| `sidecar/tools-ts/src/index.ts` | Starts the Unix-socket JSON-RPC server stub |

`BARDO_SIDECAR_SOCKET` controls the Unix socket path used by the TypeScript sidecar. It defaults
to `/tmp/bardo-tools.sock`.

## API

The scaffold's public surface is operational rather than domain-specific.

```text
build() -> cargo build --workspace
build-release() -> cargo build --release -p golem-binary
test() -> cargo nextest run --workspace
test-ci() -> cargo nextest run --workspace --profile ci
lint() -> cargo clippy --workspace --all-features -- -D warnings
fmt() -> cargo fmt --all
fmt-check() -> cargo fmt --all -- --check
deny() -> cargo deny check
coverage() -> cargo llvm-cov nextest --workspace --html
docs() -> cargo doc --workspace --no-deps --open
mdbook() -> cd docs && mdbook build
release-linux-amd64() -> cargo build --release --target x86_64-unknown-linux-musl -p golem-binary
release-linux-arm64() -> cargo build --release --target aarch64-unknown-linux-musl -p golem-binary
mirage(rpc_url="") -> cargo run -p mirage-rs -- --rpc-url <rpc_url>
ci() -> fmt-check + lint + test + deny
```

The scaffolded Rust binaries share the same entrypoint signature:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

The TypeScript sidecar currently exposes a minimal JSON-RPC 2.0 contract:

```typescript
export interface JsonRpcRequest {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params: unknown;
}

export interface JsonRpcResponse<T = unknown> {
  jsonrpc: "2.0";
  id: number | string;
  result?: T;
  error?: JsonRpcError;
}

export interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

export type SidecarMethod =
  | "uniswap_v3_quote"
  | "uniswap_v3_position_amounts"
  | "uniswap_v4_quote"
  | "uniswap_route_optimal";
```

## Architecture

```
Cargo.toml
├── crates/ (21 library crates)
├── apps/ (5 app binaries)
├── docs/ (mdBook site)
└── sidecar/tools-ts/ (npm package; not a Cargo member)
```

**Dependency layers**

| Layer | Crates |
|-------|--------|
| 0 - Foundation | golem-core |
| 1 - Runtime | golem-runtime |
| 2 - Cognition | golem-heartbeat, golem-grimoire, golem-daimon, golem-mortality, golem-dreams, golem-context |
| 3 - Safety | golem-safety |
| 4 - Infrastructure | golem-inference, golem-chain, golem-chain-intelligence, golem-triage, golem-ta, golem-oneirography, golem-tools |
| 5 - Coordination | golem-coordination |
| 6 - Surfaces | golem-surfaces, golem-creature, golem-engagement |
| 7 - Binary | golem-binary |
| Apps | bardo-gateway, bardo-terminal, bardo-styx, bardo-compute, mirage-rs |

## References

- `prd2/17-monorepo/00-packages.md` sections `Workspace Layout`, `Root Cargo.toml`, `Crate Inventory`, and `Dependency Rules`
- `prd2/17-monorepo/01-rust-workspace.md` sections `Workspace Structure`, `DX Tooling`, `Workspace Dependency Inheritance`, `Workspace Lints`, and `Release Profile`
- `prd2/17-monorepo/02-build.md` sections `Rust Workspace`, `Testing`, `Linting`, and `Formatting`
- `prd2/17-monorepo/03-conventions.md` section `Rust Conventions`
- `prd2/shared/dependencies.md` section `8. Rust Workspace Dependencies (bardo-golem-rs)`
- `prd2/shared/port-allocation.md` section `Port Map (Normative)`
- `prd2/15-dev/05-tooling.md` sections `mirage-rs CLI` and `Rust Build Tooling`
