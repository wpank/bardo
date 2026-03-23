# Rust Workspace: bardo-golem-rs [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-14

> **Reader orientation:** This document details the Rust workspace (`bardo-golem-rs`) that compiles into the Golem (mortal autonomous DeFi agent) binary. It belongs to the **Monorepo** section of the Bardo system documentation. The key concept before diving in: the workspace produces a single static binary containing the full cognitive architecture (Heartbeat, Grimoire, Daimon, mortality, dreams, inference, chain interaction, and TUI), and the same artifact deploys identically whether on Fly.io, a VPS, or a Raspberry Pi. Terms like Grimoire, Heartbeat, and PolicyCage are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

---

The Rust workspace contains all Golem runtime code. It compiles to a single static binary (`bardo-golem`) that runs the autonomous agent: Heartbeat (9-step decision cycle) loop, mortality clocks, Grimoire (persistent knowledge store) storage, Daimon (affect/emotion engine) affect engine, dream cycle, inference routing, on-chain interaction, and TUI rendering. The binary is the same artifact whether deployed on Bardo Compute (Fly.io), a self-hosted VPS, or a Raspberry Pi.

The TypeScript monorepo (`prd2/17-monorepo/00-packages.md`) handles the web-facing layer: Sanctum tools, CLI, portal dashboard, vault contracts, dev environment. The Rust workspace handles the agent runtime. They are separate repositories with a shared data contract (Grimoire schema, golem.toml config, event types).

---

## Workspace Structure

```
bardo-golem-rs/
├── Cargo.toml              # Virtual workspace manifest
├── rust-toolchain.toml     # Pinned Rust 1.82.0, edition 2021
├── deny.toml               # cargo-deny: license + advisory policy
├── hakari.toml             # cargo-hakari: feature unification
├── justfile                # Task runner aliases
├── .cargo/
│   └── config.toml         # Linker config (mold on Linux)
├── crates/
│   ├── golem-core/          # Manifest, identity, config types, GolemState
│   ├── golem-runtime/       # Top-level orchestrator: boot, tick loop, shutdown
│   ├── golem-heartbeat/     # 9-step tick pipeline, FSM, gating, speculation
│   ├── golem-grimoire/      # Knowledge graph: LanceDB + SQLite + vector search
│   ├── golem-daimon/        # PAD vector, appraisal engine, mood EMA
│   ├── golem-mortality/     # Three clocks, vitality score, behavioral phases
│   ├── golem-dreams/        # NREM/REM/Integration cycle, DreamScheduler
│   ├── golem-context/       # ContextBundle assembly, token budgeting, policy
│   ├── golem-safety/        # PolicyCage enforcement, pre-flight simulation
│   ├── golem-inference/     # Multi-provider routing, x402 payment, budget tracking
│   ├── golem-chain/         # alloy EVM client, transaction building, receipt parsing
│   ├── golem-chain-intelligence/  # bardo-witness, chain scope, protocol state, PVS
│   ├── golem-triage/        # Bayesian surprise triage, BSC fingerprinting
│   ├── golem-ta/            # TA cortical extension, TDA, regime detection
│   ├── golem-oneirography/  # Dream art, death masks, gallery, SuperRare contracts
│   ├── golem-tools/         # Tool invocation, capability tokens, tool registry
│   ├── golem-coordination/  # Clade sync, Styx WebSocket, pheromone field
│   ├── golem-surfaces/      # Ratatui TUI, event subscription, widget library
│   ├── golem-creature/      # Creature state machine, sprite rendering, animations
│   ├── golem-engagement/    # Loop cadence, retention mechanics, notification scheduling
│   └── golem-binary/        # main(), CLI arg parsing, signal handling
├── apps/
│   ├── bardo-gateway/       # Inference gateway: x402 verification + provider routing
│   ├── bardo-terminal/      # Standalone TUI binary (connects to running Golem)
│   └── bardo-styx/          # Styx relay server: clade sync, lethe, pheromone
├── mirage-rs/               # Fork simulation engine (library + binary)
└── xtask/                   # Build automation (coverage reports, schema generation)
```

---

## Crate Dependency DAG

```
golem-binary
  └── golem-runtime
        ├── golem-heartbeat
        │     ├── golem-context
        │     │     ├── golem-grimoire
        │     │     ├── golem-daimon
        │     │     └── golem-core
        │     ├── golem-safety
        │     │     ├── golem-chain
        │     │     └── golem-core
        │     ├── golem-tools
        │     │     ├── golem-chain
        │     │     └── golem-core
        │     ├── golem-inference
        │     │     └── golem-core
        │     └── golem-core
        ├── golem-mortality
        │     ├── golem-grimoire
        │     ├── golem-daimon
        │     └── golem-core
        ├── golem-dreams
        │     ├── golem-grimoire
        │     ├── golem-inference
        │     ├── golem-daimon
        │     └── golem-core
        ├── golem-coordination
        │     ├── golem-grimoire
        │     ├── golem-chain
        │     └── golem-core
        ├── golem-surfaces
        │     └── golem-core
        ├── golem-creature
        │     ├── golem-daimon
        │     └── golem-core
        ├── golem-engagement
        │     └── golem-core
        └── golem-core
```

Rules:

- `golem-core` has zero workspace dependencies. Everything depends on it.
- `golem-chain` depends only on `golem-core`. No upward dependencies.
- `golem-grimoire` depends on `golem-core`. It does not depend on `golem-daimon` or `golem-inference` (those depend on it, not the other way).
- Circular dependencies are prohibited. The graph is a strict DAG.
- `apps/` crates depend on workspace crates but workspace crates never depend on `apps/`.

---

## Key Dependencies

Workspace-level dependency declarations in the root `Cargo.toml`. Crates inherit versions via `dep.workspace = true`.

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `tokio` | 1.x | Async runtime. `rt-multi-thread`, `macros`, `signal` features. |
| `axum` | 0.8.x | HTTP framework for REST API, WebSocket, health endpoints. |
| `alloy` | 0.15.x | EVM interaction: RPC, transaction building, ABI encoding, receipt parsing. Not ethers-rs. |
| `ratatui` | 0.29.x | Terminal UI framework for the Golem dashboard. |
| `qdrant-client` | 1.x | Vector database client for Grimoire semantic search. |
| `sqlx` | 0.8.x | SQLite async driver for Grimoire structured storage. |
| `lancedb` | 0.15.x | Columnar vector DB for episodic memory (LanceDB). |
| `serde` | 1.x | Serialization with derive macros. Used everywhere. |
| `serde_json` | 1.x | JSON serialization for API payloads and config. |
| `toml` | 0.8.x | TOML parsing for `golem.toml`. |
| `thiserror` | 2.x | Typed error enums for library crates. |
| `anyhow` | 1.x | Error propagation in application crates (`golem-binary`, `apps/`). |
| `tracing` | 0.1.x | Structured logging and instrumentation. |
| `tracing-subscriber` | 0.3.x | Log formatting, filtering, OTLP export. |
| `reqwest` | 0.12.x | HTTP client for inference providers and external APIs. |
| `clap` | 4.x | CLI argument parsing for `golem-binary`. |
| `rand` | 0.9.x | Stochastic mortality PRNG, dream element selection. |
| `chrono` | 0.4.x | Timestamp handling for dream windows and sync scheduling. |
| `dashmap` | 6.x | Concurrent hash map for connection registry (Styx). |
| `proptest` | 1.x | Property-based testing (dev dependency). |
| `x402-rs` | 0.1.x | x402 HTTP payment protocol client for per-request micropayments. |

---

## DX Tooling

### rust-toolchain.toml

```toml
[toolchain]
channel = "1.82.0"
edition = "2021"
components = ["rustfmt", "clippy", "llvm-tools-preview"]
targets = ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
```

Pin the exact Rust version. No `nightly`. `llvm-tools-preview` enables `cargo-llvm-cov`.

### cargo-hakari

Feature unification across the workspace. Without it, different crates pull the same dependency with different feature flags, causing redundant compilations. `hakari` generates a synthetic crate (`workspace-hack`) that forces all features to unify.

```bash
cargo hakari generate          # Regenerate workspace-hack
cargo hakari manage-deps       # Ensure all crates depend on workspace-hack
```

### Linker Configuration

`.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# macOS uses the default linker (no mold)
```

mold cuts link times by 5-10x on Linux. On macOS, the default Apple linker is used (mold doesn't support Mach-O).

### sccache

Compiler caching. Set via env var or `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "sccache"
```

Shared cache across local builds and CI. Reduces incremental rebuild times to seconds for unchanged crates. CI uses an S3-backed sccache bucket.

### Task Runner (just)

`justfile` at workspace root:

```just
# Build all crates
build:
    cargo build --workspace

# Run all tests with nextest
test:
    cargo nextest run --workspace

# Run clippy on all crates
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format check
fmt-check:
    cargo fmt --all -- --check

# Format fix
fmt:
    cargo fmt --all

# Run cargo-deny checks (license + advisory)
deny:
    cargo deny check

# Coverage report
cov:
    cargo llvm-cov nextest --workspace --html

# Build release binary (musl static)
release-linux-amd64:
    cargo build --release --target x86_64-unknown-linux-musl -p golem-binary

release-linux-arm64:
    cargo build --release --target aarch64-unknown-linux-musl -p golem-binary

# Development mode: watch and check
watch:
    bacon clippy
```

### bacon

Background code checker. Runs `cargo clippy` on file save, shows errors inline. Replaces `cargo watch` with a better TUI.

```bash
bacon clippy    # Watch mode with clippy
bacon test      # Watch mode with tests
bacon nextest   # Watch mode with nextest
```

### lefthook

Pre-commit hooks (`.lefthook.yml`):

```yaml
pre-commit:
  parallel: true
  commands:
    fmt:
      run: cargo fmt --all -- --check
    clippy:
      run: cargo clippy --workspace --all-targets -- -D warnings
    deny:
      run: cargo deny check advisories
```

Fast fail on format and lint before pushing. `deny` checks advisories only (license check is CI-only since it's slower).

---

## Rust Coding Conventions

### Error Handling

- **Library crates** (`golem-core`, `golem-grimoire`, etc.): Use `thiserror` with typed error enums. Each crate defines its own error type. Errors carry enough context to diagnose without a stack trace.
- **Application crates** (`golem-binary`, `apps/`): Use `anyhow` for error propagation. Convert typed errors at crate boundaries.
- No `unwrap()` in production code. `expect()` only where the invariant is documented and provably holds.

```rust
// Library crate error pattern
#[derive(Debug, thiserror::Error)]
pub enum GrimoireError {
    #[error("episode not found: {id}")]
    EpisodeNotFound { id: String },

    #[error("vector search failed: {0}")]
    VectorSearch(#[from] lancedb::Error),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] sqlx::Error),
}
```

### Async

Tokio runtime everywhere. All I/O-bound operations are async. CPU-bound work (vector similarity, compression) uses `tokio::task::spawn_blocking` to avoid blocking the runtime.

### Serialization

`serde` with derive macros for all data types that cross crate boundaries or hit disk/network. `#[serde(rename_all = "snake_case")]` on enums. `#[serde(default)]` on optional config fields.

### EVM Interaction

`alloy` for everything on-chain. Not ethers-rs. alloy provides typed ABI bindings via `sol!` macro, transaction building, receipt parsing, and RPC client. The `golem-chain` crate wraps alloy's `Provider` with retry logic, chain-specific configuration, and gas estimation.

### Testing

- `cargo nextest` for parallel test execution. Nextest runs each test in its own process, preventing shared state contamination.
- `proptest` for property-based testing on numerical code (mortality calculations, credit partitions, vitality scoring).
- Integration tests in `tests/` directories within each crate. Unit tests in `#[cfg(test)]` modules.
- `BARDO_STOCHASTIC_SEED=42` in test env for deterministic mortality checks.

### Workspace Dependency Inheritance

All shared dependencies declared once in the workspace root `Cargo.toml`:

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tracing = "0.1"
alloy = { version = "0.15", features = ["providers", "signers", "sol-types"] }
# ... etc
```

Crates reference them with `workspace = true`:

```toml
[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
```

### Workspace Lints

Shared clippy and rustc lint configuration:

```toml
[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "warn"
nursery = { level = "warn", priority = -1 }
# Allow pedantic lints that produce false positives
module_name_repetitions = "allow"
must_use_candidate = "allow"

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"
```

Crates inherit via:

```toml
[lints]
workspace = true
```

---

## CI Pipeline

GitHub Actions. Five stages, fail-fast:

```
cargo check --workspace
    ↓
cargo nextest run --workspace
    ↓
cargo clippy --workspace --all-targets -- -D warnings
    ↓
cargo deny check
    ↓
cargo llvm-cov nextest --workspace --lcov --output-path lcov.info
```

- `cargo check`: Type checking without codegen. Catches compilation errors in seconds.
- `cargo nextest`: Parallel test execution. Each test isolated in its own process.
- `cargo clippy`: Lint pass. `-D warnings` makes all warnings fatal in CI.
- `cargo deny`: License compliance (allowlist: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0) + advisory database scan (RustSec).
- `cargo llvm-cov`: Source-based coverage. Uploaded to Codecov. Minimum threshold: 60% per crate.

sccache is shared across CI runs via an S3 bucket. Warm builds complete in under 3 minutes.

### cargo-vet

Supply chain auditing. Each new dependency or version bump requires a `cargo vet certify` entry from a trusted reviewer. The `supply-chain/` directory tracks audit records. CI runs `cargo vet check` and fails if any unaudited dependency is detected.

---

## Versioning Policy

All workspace crates follow semver. Pre-1.0 crates use 0.x versioning. Breaking changes bump the minor version (0.x → 0.x+1). Patch versions are for bug fixes and documentation changes only. The workspace does not enforce lockstep versioning -- each crate advances independently.

---

## Build Output

### Static Binary

The release build targets `x86_64-unknown-linux-musl` for fully static linking. No glibc dependency, no dynamic libraries, single file deployment.

```bash
cargo build --release --target x86_64-unknown-linux-musl -p golem-binary
# Output: target/x86_64-unknown-linux-musl/release/bardo-golem
```

ARM64 builds for Graviton instances and Raspberry Pi:

```bash
cargo build --release --target aarch64-unknown-linux-musl -p golem-binary
```

Binary size target: <50 MB (stripped). `strip` and `opt-level = "z"` in the release profile if needed.

### Release Profile

```toml
[profile.release]
lto = "thin"          # Link-time optimization (thin for reasonable build times)
codegen-units = 1     # Single codegen unit for better optimization
strip = true          # Strip debug symbols
panic = "abort"       # No unwinding overhead
```

---

## Docker Build

Multi-stage build with cargo-chef for layer caching:

```dockerfile
# Stage 1: Plan dependencies
FROM rust:1.82.0-slim AS planner
WORKDIR /app
RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build dependencies (cached layer)
FROM rust:1.82.0-slim AS builder
WORKDIR /app
RUN cargo install cargo-chef --locked
RUN apt-get update && apt-get install -y musl-tools
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Stage 3: Build application
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl -p golem-binary

# Stage 4: Runtime (scratch = no OS, just the binary)
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bardo-golem /bardo-golem
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENTRYPOINT ["/bardo-golem"]
```

cargo-chef separates the dependency graph from source code. When only source files change (not `Cargo.toml` or `Cargo.lock`), the dependency layer is cached and the rebuild only compiles application code. This cuts Docker build times from 10+ minutes to under 2 minutes for source-only changes.

The final image is `scratch` (no shell, no OS, no attack surface). The binary is statically linked against musl. CA certificates are copied for TLS. Image size: ~50 MB.

---

## Cross-References

- `prd2/17-monorepo/00-packages.md` -- TypeScript workspace structure
- `prd2/shared/config-reference.md` -- `golem.toml` schema (consumed by `golem-core`)
- `prd2/01-golem/02-heartbeat.md` -- Tick pipeline (implemented in `golem-heartbeat`)
- `prd2/01-golem/01-cognition.md` -- Inference routing (implemented in `golem-inference`)
- `prd2/02-mortality/01-architecture.md` -- Three clocks (implemented in `golem-mortality`)
- `prd2/05-dreams/01-architecture.md` -- Dream cycle (implemented in `golem-dreams`)
- `prd2/03-daimon/` -- Affect engine (implemented in `golem-daimon`)
- `prd2/04-memory/` -- Grimoire architecture (implemented in `golem-grimoire`)
- `prd2/11-compute/00-overview.md` -- Fly.io hosting (deploys the Docker image)
- `prd2/14-chain/` -- Chain intelligence (implemented in `golem-chain-intelligence`, `golem-triage`, `golem-ta`)
- `prd2/22-oneirography/` -- Oneirography/SuperRare (implemented in `golem-oneirography`)
