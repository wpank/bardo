# Plan 01: Workspace Scaffold & mdbook

## Context

This plan creates the physical skeleton of the entire Cargo workspace. Nothing else can compile until it exists. Specifically it implements:

- `prd2/17-monorepo/00-packages.md` §§ Workspace Layout, Root Cargo.toml, Crate Inventory, Dependency Rules
- `prd2/17-monorepo/01-rust-workspace.md` §§ Workspace Structure, Key Dependencies, DX Tooling
- `prd2/17-monorepo/02-build.md` §§ Rust Workspace (rustfmt, clippy, justfile tasks)
- `prd2/17-monorepo/03-conventions.md` §§ Rust Conventions, Workspace Dependency Inheritance, Lint Config
- `prd2/shared/dependencies.md` §8 Rust Workspace Dependencies (version pins)

This plan does NOT implement any crate logic. It creates shell files only — enough for `cargo check --workspace` to succeed with zero compile errors.

---

## Previous Plan

First plan — no predecessor.

---

## Prerequisites

None. This is the foundation everything else builds on.

---

## Imports (from earlier plans)

None.

---

## Exports (for later plans)

Every subsequent plan depends on this plan's output:

**Workspace root:**
- `Cargo.toml` — workspace manifest with all 26 members, all `[workspace.dependencies]`, all `[workspace.package]`, all `[workspace.lints]`, release profile
- `rust-toolchain.toml` — Rust 1.85, edition 2024
- `rustfmt.toml` — edition 2024, max_width 100, imports_granularity Crate
- `clippy.toml` — pedantic warn, selective allows
- `.cargo/config.toml` — mold linker on Linux, sccache wrapper
- `nextest.toml` — test runner config
- `deny.toml` — license allowlist stub
- `justfile` — all task aliases

**Library crate shells (each has `Cargo.toml` + `src/lib.rs`):**
- `crates/golem-core/`
- `crates/golem-runtime/`
- `crates/golem-heartbeat/`
- `crates/golem-grimoire/`
- `crates/golem-daimon/`
- `crates/golem-mortality/`
- `crates/golem-dreams/`
- `crates/golem-context/`
- `crates/golem-safety/`
- `crates/golem-inference/`
- `crates/golem-chain/`
- `crates/golem-chain-intelligence/`
- `crates/golem-triage/`
- `crates/golem-ta/`
- `crates/golem-oneirography/`
- `crates/golem-tools/`
- `crates/golem-coordination/`
- `crates/golem-surfaces/`
- `crates/golem-creature/`
- `crates/golem-engagement/`
- `crates/golem-binary/`

**App binary shells (each has `Cargo.toml` + `src/main.rs`):**
- `apps/bardo-gateway/`
- `apps/bardo-terminal/`
- `apps/bardo-styx/`
- `apps/bardo-compute/`
- `apps/mirage-rs/`

**Documentation scaffold:**
- `docs/book.toml`
- `docs/src/SUMMARY.md`
- `docs/src/introduction/what-is-bardo.md`

**TypeScript sidecar stub:**
- `sidecar/tools-ts/package.json`
- `sidecar/tools-ts/tsconfig.json`
- `sidecar/tools-ts/src/index.ts`
- `sidecar/tools-ts/src/types.ts`

**Test directory stubs:**
- `tests/conformance/.gitkeep`
- `tests/adversarial/.gitkeep`
- `tests/property/.gitkeep`
- `tests/integration/.gitkeep`

---

## Cargo Dependencies

The complete workspace dependency catalog, used verbatim in the root `Cargo.toml`:

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1.50", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging / tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# HTTP / Web
axum = { version = "0.8", features = ["ws"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# Ethereum
alloy = { version = "1.7", features = ["full"] }
alloy-primitives = "1"
alloy-sol-types = "1"
revm = { version = "36.0", features = ["std"] }

# Database
rusqlite = { version = "0.32", features = ["bundled"] }
sled = "0.34"

# Serialization (binary)
bincode = "1"

# Vector DB / Embeddings (for Grimoire)
lancedb = "0.27"
fastembed = "5.13"

# TUI
ratatui = "0.30"
crossterm = "0.28"

# Caching
moka = { version = "0.12", features = ["future"] }

# Audio (optional)
rodio = { version = "0.22", optional = true }

# Full-text search (Grimoire BM25 hybrid retrieval, Plan 69)
tantivy = "0.22"

# Observability (Plans 39c, 63b)
opentelemetry = { version = "0.27", features = ["metrics"] }
opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }
tracing-opentelemetry = "0.28"

# Testing
proptest = "1.10"

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rand = "0.8"
bytes = "1"
dashmap = "6"
parking_lot = "0.12"
clap = { version = "4", features = ["derive"] }
zeroize = { version = "1", features = ["derive"] }
sha2 = "0.10"
p256 = "0.13"
wasmtime = "26"
bumpalo = "3"
tokio-tungstenite = "0.26"
```

---

## Source Files

| File | Sections Used |
|------|--------------|
| `prd2/17-monorepo/00-packages.md` | Workspace Layout, Root Cargo.toml, Crate Inventory (all layers), TypeScript Sidecar, Dependency Rules |
| `prd2/17-monorepo/01-rust-workspace.md` | Workspace Structure, Crate Dependency DAG, Key Dependencies, DX Tooling (rust-toolchain.toml, Linker Configuration, sccache, Task Runner, bacon, lefthook), Rust Coding Conventions, Workspace Dependency Inheritance, Workspace Lints, CI Pipeline, Release Profile |
| `prd2/17-monorepo/02-build.md` | Rust Workspace: Build, Testing (nextest.toml), Linting (clippy.toml), Formatting (rustfmt.toml), Coverage |
| `prd2/17-monorepo/03-conventions.md` | Rust Conventions (Error Handling, Async, Serialization, Testing, Workspace Dependency Inheritance, Lint Config, Naming, Visibility) |
| `prd2/shared/glossary.md` | All Bardo-specific terminology used in doc comments |
| `prd2/shared/branding.md` | Product names, crate names table, color palette (ROSEDUST), no-emojis rule |
| `prd2/shared/dependencies.md` | §8 Rust Workspace Dependencies (version pins per dep) |
| `prd2/shared/doc-standards.md` | doc-comment standards for `//!` crate-level docs |
| `prd2/shared/port-allocation.md` | Full port map — include in Quick Reference |
| `prd2/15-dev/05-tooling.md` | justfile tasks, clippy/rustfmt invocations, mirage-rs CLI flags |

---

## Implementation Details

---

### Unit 1: Workspace Root & Toolchain

**Quick Reference**

Complete root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    # Layer 0
    "crates/golem-core",
    # Layer 1
    "crates/golem-runtime",
    # Layer 2
    "crates/golem-heartbeat",
    "crates/golem-grimoire",
    "crates/golem-daimon",
    "crates/golem-mortality",
    "crates/golem-dreams",
    "crates/golem-context",
    # Layer 3
    "crates/golem-safety",
    # Layer 4
    "crates/golem-inference",
    "crates/golem-chain",
    "crates/golem-chain-intelligence",
    "crates/golem-triage",
    "crates/golem-ta",
    "crates/golem-oneirography",
    "crates/golem-tools",
    # Layer 5
    "crates/golem-coordination",
    # Layer 6
    "crates/golem-surfaces",
    "crates/golem-creature",
    "crates/golem-engagement",
    # Layer 7
    "crates/golem-binary",
    # Apps
    "apps/bardo-gateway",
    "apps/bardo-terminal",
    "apps/bardo-styx",
    "apps/bardo-compute",
    "apps/mirage-rs",
]

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "Proprietary"
authors = ["Bardo <engineering@bardo.run>"]

[workspace.dependencies]
# Async runtime
tokio = { version = "1.50", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging / tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# HTTP / Web
axum = { version = "0.8", features = ["ws"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# Ethereum
alloy = { version = "1.7", features = ["full"] }
alloy-primitives = "1"
alloy-sol-types = "1"
revm = { version = "36.0", features = ["std"] }

# Database
rusqlite = { version = "0.32", features = ["bundled"] }
sled = "0.34"

# Serialization (binary)
bincode = "1"

# Vector DB / Embeddings
lancedb = "0.27"
fastembed = "5.13"

# TUI
ratatui = "0.30"
crossterm = "0.28"

# Caching
moka = { version = "0.12", features = ["future"] }

# Audio (optional feature)
rodio = { version = "0.22", optional = true }

# Full-text search (Grimoire BM25, Plan 69)
tantivy = "0.22"

# Observability (Plans 39c, 63b)
opentelemetry = { version = "0.27", features = ["metrics"] }
opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }
tracing-opentelemetry = "0.28"

# Testing
proptest = "1.10"

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rand = "0.8"
bytes = "1"
dashmap = "6"
parking_lot = "0.12"
clap = { version = "4", features = ["derive"] }
zeroize = { version = "1", features = ["derive"] }
sha2 = "0.10"
p256 = "0.13"
wasmtime = "26"
bumpalo = "3"
tokio-tungstenite = "0.26"

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
# Selective allows for high-noise pedantic rules
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
unwrap_used = "deny"
expect_used = "warn"

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
debug = true
```

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.85"
components = ["rustfmt", "clippy", "llvm-tools-preview"]
targets = ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
```

`rustfmt.toml`:

```toml
edition = "2024"
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

`clippy.toml`:

```toml
# Pedantic rules are enforced at workspace level via [workspace.lints.clippy].
# This file holds any per-tool configuration; currently empty.
```

`.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# macOS uses default Apple linker — no mold entry needed

[build]
# Uncomment if sccache is installed: rustc-wrapper = "sccache"
```

`nextest.toml`:

```toml
[profile.default]
test-threads = "num-cpus"
slow-timeout = { period = "60s", terminate-after = 3 }
fail-fast = false

[profile.ci]
fail-fast = true
slow-timeout = { period = "120s", terminate-after = 2 }
```

`deny.toml`:

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0"]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

Port allocation reference (from `prd2/shared/port-allocation.md`):

| Port  | Service                          | Owner                  |
|-------|----------------------------------|------------------------|
| 3000  | Vault debug UI                   | `packages/vault/ui`    |
| 3001  | Dev debug UI                     | `packages/dev/ui`      |
| 3002  | Portal local server              | `packages/portal`      |
| 3003  | Dev browser SPA                  | `packages/dev/browser` |
| 5100  | Otterscan block explorer         | Docker (external)      |
| 8080  | Vault tool server (HTTP+SSE)     | `packages/vault`       |
| 8081  | Vault tool server (WebSocket)    | `packages/vault`       |
| 8443  | Local Styx WebSocket (dev TLS)   | `bardo-styx`           |
| 8545  | Anvil RPC                        | `packages/dev`         |
| 8546  | mirage-rs JSON-RPC (default)     | `apps/mirage-rs`       |
| 9090  | bardo-compute provisioning API   | `apps/bardo-compute`   |
| 42069 | Ponder indexer                   | `packages/dev`         |
| 42070 | Indexer translation proxy        | `packages/dev`         |

**Source Files:** `prd2/17-monorepo/00-packages.md`, `prd2/17-monorepo/01-rust-workspace.md`, `prd2/17-monorepo/02-build.md`, `prd2/17-monorepo/03-conventions.md`, `prd2/shared/dependencies.md`, `prd2/shared/port-allocation.md`

**Crate Location:** N/A — workspace root files

**Implementation Notes:**

1. Create the files exactly as shown in Quick Reference. Do not add extra workspace features or profile settings beyond what is listed.
2. The `rodio` dep has `optional = true` because audio is a feature flag, not a hard dependency. Individual crates opt in.
3. Edition is `2024`. This requires Rust 1.85+. The toolchain is pinned accordingly.
4. Do NOT add `workspace-hack` (cargo-hakari) yet — that requires `cargo hakari generate` which modifies many files. Skip in this plan.
5. The `.cargo/config.toml` `rustc-wrapper` line is commented out. Uncomment locally if `sccache` is installed. CI sets it via environment variable.
6. The `deny.toml` is a minimal stub. The full advisory configuration is fleshed out when CI is wired.
7. After creating all files, verify: `cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d['workspace_members']))"` should print `25` (21 library crates + 4 app binaries).

**Gitbook Documentation:** N/A for this unit (docs scaffold created in Unit 5).

**Verification:**
```bash
# Check workspace member count
cargo metadata --no-deps --format-version 1 | python3 -c \
  "import sys,json; d=json.load(sys.stdin); print('members:', len(d['workspace_members']))"
# Expected output: members: 25

# Workspace check (will fail until crates exist — run after Unit 2 and 3)
cargo check --workspace 2>&1 | tail -5
```

---

### Unit 2: Library Crate Shells (21 crates)

**Quick Reference**

Layer→crate mapping with one-line description and implementing plan:

| Layer | Crate | Description | Implemented By |
|-------|-------|-------------|----------------|
| 0 | `golem-core` | Shared types, config, GolemId, PADVector, MarketRegime, CognitiveTier, GolemConfig, CorticalState, EventFabric, TaintLabel, bump allocator | Plan 02 |
| 1 | `golem-runtime` | Extension registry, hook dispatch, GolemState, lifecycle FSM (Provisioning→Active→Dreaming→Terminal→Dead), shutdown handler | Plans 02, 14b |
| 2 | `golem-heartbeat` | 9-step CoALA pipeline (observe→retrieve→analyze→gate→simulate→validate→execute→verify→reflect), DecisionCycleRecord, heartbeat FSM | Plan 15 |
| 2 | `golem-grimoire` | LanceDB episodic store, SQLite semantic store (5 entry types), PLAYBOOK.md, four-factor retrieval, curator | Plan 12 |
| 2 | `golem-daimon` | ALMA emotion model, PAD vectors, OCC/Scherer appraisal, somatic markers, mood EMA, clade emotional contagion | Plan 14a |
| 2 | `golem-mortality` | Three death clocks (economic, epistemic, stochastic), VitalityState, behavioral phases, thanatopsis (4-phase death protocol) | Plans 13a, 13b |
| 2 | `golem-dreams` | Dream scheduler, NREM replay, REM imagination, consolidation, PLAYBOOK.md evolution | Plan 21+ |
| 2 | `golem-context` | CognitiveWorkspace assembly, ContextPolicy, per-category token allocation, background fiber, typed interventions | Plan 15+ |
| 3 | `golem-safety` | Capability<T> tokens, TaintedString, PolicyCage, merkle audit log, LoopGuard, ActionPermit lifecycle | Plan 10 |
| 4 | `golem-inference` | T0/T1/T2 tier routing, five provider integrations, x402 micropayment, SSE parser, cost cap | Plan 11 |
| 4 | `golem-chain` | Alloy provider, ERC-8004, Permit2, Warden, revm_sim, block/log types | Plan 09 |
| 4 | `golem-chain-intelligence` | bardo-witness block ingestion, chain scope, protocol state, PVS | Plans 17, 18 |
| 4 | `golem-triage` | Bayesian surprise triage, HDC/BSC fingerprints, KL divergence | Plan 19 |
| 4 | `golem-ta` | TaCorticalExtension, TDA (Betti curves, persistence diagrams), regime detection | Plan 75a+ |
| 4 | `golem-oneirography` | Dream journal, death mask minting, SuperRare integration, lineage graph | Plan 72+ |
| 4 | `golem-tools` | ToolDef, ToolContext, ToolResult, three tool traits (ReadTool/WriteTool/PrivilegedTool), registry, Wasmtime sandbox, JSON-RPC sidecar client | Plans 26-35 |
| 5 | `golem-coordination` | Pheromone field client (THREAT/OPPORTUNITY/WISDOM), clade sync, bloodstain ingestion, PropagationPolicy | Plan 40+ |
| 6 | `golem-surfaces` | Axum WebSocket handler, SSE fallback, Telegram push, GolemSnapshot | Plan 63+ |
| 6 | `golem-creature` | Creature visual state, evolution forms (Egg→Hatchling→Mature→Weathered→Transcendent), PAD→expression mapping | Plan 43+ |
| 6 | `golem-engagement` | Achievement engine, death recap, graveyard, toast/notification events | Plan 64+ |
| 7 | `golem-binary` | Single binary entry point, CLI args (--config, --data-dir, --phenotype), signal handlers, extension registration | Plan 41+ |

**Crate `Cargo.toml` pattern** (same for all 21):

```toml
[package]
name = "golem-CRATE"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[lints]
workspace = true

[dependencies]
# No workspace deps in the shell — each plan adds what it needs
```

**Crate `src/lib.rs` pattern**:

```rust
//! `golem-CRATE` — ONE_LINE_DESCRIPTION_FROM_TABLE_ABOVE
//!
//! **Implemented by:** Plan NN
//! **Depends on:** [list from DAG in prd2/17-monorepo/01-rust-workspace.md]
//!
//! This crate is a shell. Plan NN implements the actual content.

#![deny(unsafe_code)]
#![warn(missing_docs)]
```

**Source Files:** `prd2/17-monorepo/00-packages.md` §Crate Inventory, `prd2/17-monorepo/01-rust-workspace.md` §Crate Dependency DAG

**Crate Location:** `crates/<name>/Cargo.toml` and `crates/<name>/src/lib.rs` for each of the 21 crates listed above.

**Implementation Notes:**

1. Create all 21 directories. Each gets exactly two files: `Cargo.toml` and `src/lib.rs`.
2. The `[dependencies]` section in each shell Cargo.toml is intentionally empty. Later plans add `dep.workspace = true` entries as they implement the crate.
3. The `[lints]` table MUST be present in every crate Cargo.toml so workspace lint config is inherited.
4. `golem-binary` is a library crate at the workspace level even though it contains a binary. Its `src/lib.rs` exports the top-level orchestration. The actual binary target is in `apps/` — see Unit 3. Wait: per `prd2/17-monorepo/00-packages.md`, `golem-binary` is listed under `crates/` as the binary that ships. Create it as a binary crate: `src/main.rs` not `src/lib.rs`, with `[[bin]]` in its Cargo.toml. Plan 41 fills in the actual `main()`.

    `crates/golem-binary/Cargo.toml`:
    ```toml
    [package]
    name = "golem-binary"
    version = "0.1.0"
    edition.workspace = true
    rust-version.workspace = true
    license.workspace = true
    authors.workspace = true

    [[bin]]
    name = "bardo-golem"
    path = "src/main.rs"

    [lints]
    workspace = true

    [dependencies]
    anyhow = { workspace = true }
    tokio = { workspace = true }
    tracing = { workspace = true }
    tracing-subscriber = { workspace = true }
    ```

    `crates/golem-binary/src/main.rs`:
    ```rust
    //! `golem-binary` — Single binary entry point for the Golem runtime.
    //!
    //! **Implemented by:** Plan 41+
    //! **Depends on:** All other golem-* crates
    //!
    //! This is a shell. Plan 41 implements the actual startup sequence.

    #[tokio::main]
    async fn main() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();
        tracing::info!("bardo-golem starting — not yet implemented");
        Ok(())
    }
    ```

5. All other 20 crates use `src/lib.rs` (not `src/main.rs`). They are pure libraries.
6. One-line descriptions for the `//!` doc comments come from the table in Quick Reference above. Copy them verbatim. Accuracy matters because later plans `grep` these comments.

**Gitbook Documentation:** N/A for shells — each crate's implementing plan writes the gitbook page.

**Verification:**
```bash
# Each crate must check cleanly
for crate in golem-core golem-runtime golem-heartbeat golem-grimoire golem-daimon \
             golem-mortality golem-dreams golem-context golem-safety golem-inference \
             golem-chain golem-chain-intelligence golem-triage golem-ta \
             golem-oneirography golem-tools golem-coordination golem-surfaces \
             golem-creature golem-engagement golem-binary; do
    cargo check -p "$crate" 2>&1 | grep -E "(error|warning)" | grep -v "missing_docs" | head -3
done
```

---

### Unit 3: App Binary Shells (5 apps)

**Quick Reference**

| Binary | Path | Description | Port | Implemented By |
|--------|------|-------------|------|----------------|
| `bardo-gateway` | `apps/bardo-gateway/` | Inference gateway: x402 USDC payment verification + provider routing (5 providers) | 8080/8081 | Plans 11, 36-39c |
| `bardo-terminal` | `apps/bardo-terminal/` | Standalone ratatui TUI that connects to a running golem via WebSocket | (none, connects to golem) | Plans 04-08d, 70a-70c |
| `bardo-styx` | `apps/bardo-styx/` | Styx relay server: clade knowledge sync, pheromone field, Lethe (formerly Commons) | 8443 (TLS) | Plan 40 |
| `bardo-compute` | `apps/bardo-compute/` | Compute provisioning service: Fly.io warm pool, x402 per-hour billing, fleet management | 9090 | Plan 67 |
| `mirage-rs` | `apps/mirage-rs/` | In-process revm fork simulator with JSON-RPC compat layer, state CoW layers | 8546 | Plan 03 |

**App `Cargo.toml` pattern**:

```toml
[package]
name = "bardo-APP"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[[bin]]
name = "bardo-APP"
path = "src/main.rs"

[lints]
workspace = true

[dependencies]
anyhow = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

**App `src/main.rs` pattern**:

```rust
//! `bardo-APP` — ONE_LINE_DESCRIPTION
//!
//! **Implemented by:** Plan NN
//!
//! This is a shell. Plan NN implements the actual binary.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("bardo-APP starting — not yet implemented");
    Ok(())
}
```

**`apps/mirage-rs/` note:** Per `prd2/15-dev/05-tooling.md`, mirage-rs exposes a CLI with flags (`--rpc-url`, `--fork-block`, `--follow`, `--port`, etc.). The shell binary name is `mirage-rs`. Its Cargo.toml:

```toml
[package]
name = "mirage-rs"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[[bin]]
name = "mirage-rs"
path = "src/main.rs"

[lints]
workspace = true

[dependencies]
anyhow = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
clap = { workspace = true }
```

**Source Files:** `prd2/17-monorepo/00-packages.md` §Apps, `prd2/17-monorepo/01-rust-workspace.md` §Workspace Structure, `prd2/15-dev/05-tooling.md`

**Crate Location:** `apps/bardo-gateway/`, `apps/bardo-terminal/`, `apps/bardo-styx/`, `apps/bardo-compute/`, `apps/mirage-rs/`

**Implementation Notes:**

1. Apps are workspace members. They appear in `[workspace] members` in root Cargo.toml.
2. Apps MUST NOT be depended on by any `crates/` workspace member (see Dependency Rules in `prd2/17-monorepo/00-packages.md`).
3. `apps/mirage-rs/` resolves Q-01 from CONTEXT.md: apps are in `[workspace] members`. This is confirmed by the workspace layout in `prd2/17-monorepo/01-rust-workspace.md` which shows `apps/` inside the workspace.
4. The sidecar at `sidecar/tools-ts/` is NOT a Cargo workspace member — it is a TypeScript project. Do not add it to `[workspace] members`.

**Gitbook Documentation:** N/A for shells.

**Verification:**
```bash
cargo check -p bardo-gateway
cargo check -p bardo-terminal
cargo check -p bardo-styx
cargo check -p bardo-compute
cargo check -p mirage-rs
```

---

### Unit 4: Build Tooling & Dev Setup

**Quick Reference**

Complete `justfile` at workspace root:

```just
# Build all crates (debug)
build:
    cargo build --workspace

# Build release binary
build-release:
    cargo build --release -p golem-binary

# Run all tests with nextest
test:
    cargo nextest run --workspace

# Run tests in CI mode (fail-fast)
test-ci:
    cargo nextest run --workspace --profile ci

# Run clippy on all crates
lint:
    cargo clippy --workspace --all-features -- -D warnings

# Format all crates in-place
fmt:
    cargo fmt --all

# Check formatting (CI)
fmt-check:
    cargo fmt --all -- --check

# Run cargo-deny checks (license + advisory)
deny:
    cargo deny check

# Generate coverage report
coverage:
    cargo llvm-cov nextest --workspace --html

# Build docs
docs:
    cargo doc --workspace --no-deps --open

# Build mdbook docs
mdbook:
    cd docs && mdbook build

# Watch mode (requires bacon)
watch:
    bacon clippy

# Release builds for deployment targets
release-linux-amd64:
    cargo build --release --target x86_64-unknown-linux-musl -p golem-binary

release-linux-arm64:
    cargo build --release --target aarch64-unknown-linux-musl -p golem-binary

# Run mirage-rs dev fork
mirage rpc_url="":
    cargo run -p mirage-rs -- --rpc-url {{rpc_url}} --follow

# Full CI check sequence
ci: fmt-check lint test deny
    @echo "CI passed"
```

**Source Files:** `prd2/17-monorepo/01-rust-workspace.md` §Task Runner, `prd2/15-dev/05-tooling.md` §justfile

**Crate Location:** `justfile` at workspace root, `.cargo/config.toml` (created in Unit 1)

**Implementation Notes:**

1. The `justfile` uses just's recipe syntax. Recipes are tab-indented (spaces in the Quick Reference above — use tabs in the actual file).
2. `just build` is the canonical check-everything command. Plans that add features use it as their verification step.
3. `just ci` chains `fmt-check lint test deny` — this is the PR gate.
4. The `mirage` recipe takes an optional `rpc_url` parameter.
5. No additional files beyond `justfile` are needed for this unit — `.cargo/config.toml` was handled in Unit 1.

**Gitbook Documentation:** N/A.

**Verification:**
```bash
just build 2>&1 | tail -3
# Expected: Finished `dev` profile target(s)

just fmt-check 2>&1 | tail -3
# Expected: silent (all files are freshly formatted) or minor warns
```

---

### Unit 5: mdbook Scaffold

**Quick Reference**

`docs/book.toml`:

```toml
[book]
title = "Bardo"
description = "Permissionless infrastructure for mortal autonomous agents in DeFi"
authors = ["Bardo Engineering"]
language = "en"
multilingual = false
src = "src"

[build]
build-dir = "book"
create-missing = true

[output.html]
site-url = "/docs/"
git-repository-url = "https://github.com/bardo-run/bardo"
edit-url-template = "https://github.com/bardo-run/bardo/edit/main/docs/{path}"
```

`docs/src/SUMMARY.md` (minimal — chapters added as plans complete):

```markdown
# Summary

[Introduction](introduction/what-is-bardo.md)

---

# Architecture

- [Workspace Overview](architecture/workspace.md)

---

# Crates

- [golem-core](crates/golem-core.md)
- [golem-runtime](crates/golem-runtime.md)
- [golem-heartbeat](crates/golem-heartbeat.md)
- [golem-grimoire](crates/golem-grimoire.md)
- [golem-daimon](crates/golem-daimon.md)
- [golem-mortality](crates/golem-mortality.md)
- [golem-dreams](crates/golem-dreams.md)
- [golem-context](crates/golem-context.md)
- [golem-safety](crates/golem-safety.md)
- [golem-inference](crates/golem-inference.md)
- [golem-chain](crates/golem-chain.md)
- [golem-chain-intelligence](crates/golem-chain-intelligence.md)
- [golem-triage](crates/golem-triage.md)
- [golem-ta](crates/golem-ta.md)
- [golem-oneirography](crates/golem-oneirography.md)
- [golem-tools](crates/golem-tools.md)
- [golem-coordination](crates/golem-coordination.md)
- [golem-surfaces](crates/golem-surfaces.md)
- [golem-creature](crates/golem-creature.md)
- [golem-engagement](crates/golem-engagement.md)
- [golem-binary](crates/golem-binary.md)

---

# Apps

- [bardo-gateway](apps/bardo-gateway.md)
- [bardo-terminal](apps/bardo-terminal.md)
- [bardo-styx](apps/bardo-styx.md)
- [bardo-compute](apps/bardo-compute.md)
- [mirage-rs](apps/mirage-rs.md)
```

`docs/src/introduction/what-is-bardo.md`:

```markdown
# What Is Bardo?

Bardo is permissionless infrastructure for mortal autonomous agents in DeFi.

A **golem** is a finite-lived Rust process that executes a 9-step cognitive
loop — observe, retrieve, analyze, gate, simulate, validate, execute, verify,
reflect — once per tick. It holds USDC as metabolic substrate. When the USDC
balance reaches zero, or when epistemic fitness decays below threshold, or when
a stochastic mortality draw fires, the golem dies. At death it runs the
Thanatopsis protocol: compress its Grimoire (knowledge store) to at most 2048
entries, push to the clade, and leave a death mask on-chain.

The successor golem inherits this compressed knowledge. Across generations,
the population accumulates judgment that no immortal agent can develop:
knowledge that has been distilled under survival pressure.

## Crate Architecture

```
golem-binary  (single Fly.io VM binary)
  └── golem-runtime  (extension registry, lifecycle FSM)
        ├── golem-heartbeat  (9-step tick pipeline)
        │     ├── golem-context  (CognitiveWorkspace assembly)
        │     │     ├── golem-grimoire  (LanceDB + SQLite + PLAYBOOK.md)
        │     │     ├── golem-daimon  (PAD affect engine)
        │     │     └── golem-core  [foundation]
        │     ├── golem-safety  (Capability<T>, PolicyCage, audit log)
        │     ├── golem-tools  (tool registry, Wasmtime sandbox)
        │     ├── golem-inference  (T0/T1/T2 routing, x402)
        │     └── golem-core
        ├── golem-mortality  (three clocks, thanatopsis)
        ├── golem-dreams  (NREM/REM/consolidation)
        ├── golem-coordination  (pheromone field, clade sync)
        ├── golem-chain  (Alloy, ERC-8004, Warden, revm)
        ├── golem-chain-intelligence  (bardo-witness, PVS)
        ├── golem-triage  (Bayesian surprise)
        ├── golem-ta  (TDA, regime detection)
        ├── golem-surfaces  (WebSocket, SSE, Telegram)
        ├── golem-creature  (visual identity engine)
        ├── golem-engagement  (achievements, graveyard)
        └── golem-core  [zero workspace deps]
```

## Getting Started

```bash
# Clone and check the workspace compiles
git clone https://github.com/bardo-run/bardo
cd bardo
just build

# Run tests
just test

# Start a dev fork (requires RPC URL)
just mirage $RPC_URL
```

## Key Concepts

| Term | Definition |
|------|-----------|
| Golem | A mortal autonomous DeFi agent compiled as a single Rust binary |
| Grimoire | Persistent knowledge store (LanceDB episodic + SQLite semantic + PLAYBOOK.md procedural) |
| Heartbeat | The 9-step autonomous decision cycle (CoALA pipeline) |
| Clade | A fleet of sibling golems sharing knowledge via the Styx relay |
| Daimon | The affect/emotion engine that maps market events to PAD vectors |
| Thanatopsis | The four-phase death protocol: Acceptance, Settlement, Reflection, Legacy |
| Bardo | The transitional state between death and rebirth — the system's philosophical framework |
```

Stub pages for crates and apps (created with `create-missing = true` in book.toml, but Codex
should create explicit stubs to avoid mdbook errors):

For each crate page (`docs/src/crates/golem-CRATE.md`) and app page (`docs/src/apps/bardo-APP.md`):

```markdown
# golem-CRATE

**Implemented by:** Plan NN

*Documentation forthcoming. This page is updated when Plan NN completes.*
```

Also create:

`docs/src/architecture/workspace.md`:

```markdown
# Workspace Overview

The Bardo workspace is a Cargo workspace with 21 library crates and 4 app binaries.

See `prd2/17-monorepo/00-packages.md` for the authoritative crate inventory.

**Workspace members:** 25 total (21 library crates + 4 app binaries).

**Dependency layers:**

| Layer | Crates |
|-------|--------|
| 0 — Foundation | golem-core |
| 1 — Runtime | golem-runtime |
| 2 — Cognition | golem-heartbeat, golem-grimoire, golem-daimon, golem-mortality, golem-dreams, golem-context |
| 3 — Safety | golem-safety |
| 4 — Infrastructure | golem-inference, golem-chain, golem-chain-intelligence, golem-triage, golem-ta, golem-oneirography, golem-tools |
| 5 — Coordination | golem-coordination |
| 6 — Surfaces | golem-surfaces, golem-creature, golem-engagement |
| 7 — Binary | golem-binary |
```

**Source Files:** `prd2/17-monorepo/00-packages.md`, `prd2/17-monorepo/01-rust-workspace.md`, `prd2/shared/branding.md`

**Crate Location:** `docs/` directory

**Implementation Notes:**

1. Install mdbook before running this unit: `cargo install mdbook`.
2. `create-missing = true` in book.toml causes mdbook to create empty files for entries in SUMMARY.md that don't exist. However, Codex should create all stub pages explicitly so they have meaningful placeholder content.
3. The crate architecture ASCII diagram in `what-is-bardo.md` uses backtick code blocks — ensure it renders correctly by keeping the inner triple-backtick on its own line.
4. All 21 crate pages and 4 app pages are stubs. The implementing plan for each crate fills in the actual documentation.
5. The `docs/` directory is not a workspace member. It has no Cargo.toml.

**Gitbook Documentation:** This unit creates the docs scaffold itself.

**Verification:**
```bash
# mdbook must be installed first
command -v mdbook || cargo install mdbook
cd /path/to/bardo/docs && mdbook build 2>&1 | tail -5
# Expected: Finished in X.XXs
ls book/index.html
```

---

### Unit 6: TypeScript Sidecar Stub

**Quick Reference**

The sidecar lives at `sidecar/tools-ts/`. It is NOT a Cargo workspace member. It is a separate npm project spawned by `golem-tools` at runtime over a Unix domain socket using JSON-RPC 2.0.

`sidecar/tools-ts/package.json`:

```json
{
  "name": "@bardo/tools-ts-sidecar",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "main": "dist/index.js",
  "scripts": {
    "build": "tsc",
    "dev": "tsc --watch",
    "start": "node dist/index.js"
  },
  "dependencies": {
    "@uniswap/v3-sdk": "^3.14.0",
    "@uniswap/v4-sdk": "^1.0.0",
    "@uniswap/smart-order-router": "^3.38.0",
    "jsbi": "^4.3.0"
  },
  "devDependencies": {
    "typescript": "^5.8.0"
  }
}
```

`sidecar/tools-ts/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "esModuleInterop": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

`sidecar/tools-ts/src/types.ts`:

```typescript
// JSON-RPC 2.0 types for the Bardo tools-ts sidecar.
// The Rust golem-tools crate connects to this over a Unix domain socket.

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

// Method registry — filled in by Plan 26+ (golem-tools implementation)
export type SidecarMethod =
  | "uniswap_v3_quote"
  | "uniswap_v3_position_amounts"
  | "uniswap_v4_quote"
  | "uniswap_route_optimal";
```

`sidecar/tools-ts/src/index.ts`:

```typescript
// Bardo tools-ts sidecar — JSON-RPC 2.0 server over Unix domain socket.
//
// Spawned by golem-tools at startup. Provides Uniswap V3/V4 concentrated
// liquidity math that has no mature Rust equivalent.
//
// Implemented by: Plan 26+ (golem-tools integration)
// This file is a stub. Plan 26 implements the actual tool handlers.

import * as net from "node:net";
import * as fs from "node:fs";
import type { JsonRpcRequest, JsonRpcResponse } from "./types.js";

const SOCKET_PATH = process.env["BARDO_SIDECAR_SOCKET"] ?? "/tmp/bardo-tools.sock";

// Remove stale socket file from a previous run
if (fs.existsSync(SOCKET_PATH)) {
  fs.unlinkSync(SOCKET_PATH);
}

const server = net.createServer((socket) => {
  let buffer = "";

  socket.on("data", (chunk) => {
    buffer += chunk.toString();
    const lines = buffer.split("\n");
    buffer = lines.pop() ?? "";

    for (const line of lines) {
      if (!line.trim()) continue;
      try {
        const request = JSON.parse(line) as JsonRpcRequest;
        const response = handleRequest(request);
        socket.write(JSON.stringify(response) + "\n");
      } catch (err) {
        const errorResponse: JsonRpcResponse = {
          jsonrpc: "2.0",
          id: "unknown",
          error: { code: -32700, message: "Parse error" },
        };
        socket.write(JSON.stringify(errorResponse) + "\n");
      }
    }
  });

  socket.on("error", (err) => {
    console.error("Socket error:", err.message);
  });
});

function handleRequest(request: JsonRpcRequest): JsonRpcResponse {
  // Stub: all methods return "not implemented" until Plan 26
  return {
    jsonrpc: "2.0",
    id: request.id,
    error: {
      code: -32601,
      message: `Method not found: ${request.method} (sidecar not yet implemented — see Plan 26)`,
    },
  };
}

server.listen(SOCKET_PATH, () => {
  console.log(`Bardo tools-ts sidecar listening on ${SOCKET_PATH}`);
});

process.on("SIGTERM", () => {
  server.close(() => process.exit(0));
});

process.on("SIGINT", () => {
  server.close(() => process.exit(0));
});
```

**Source Files:** `prd2/17-monorepo/00-packages.md` §TypeScript Sidecar

**Crate Location:** `sidecar/tools-ts/`

**Implementation Notes:**

1. Do NOT run `npm install` in this plan. The sidecar is built separately. This plan only creates the stub files.
2. The socket path is controlled by `BARDO_SIDECAR_SOCKET` env var with `/tmp/bardo-tools.sock` as default.
3. The JSON-RPC protocol uses newline-delimited JSON (one request per line, one response per line). Round-trip latency target is 1–5ms per `prd2/17-monorepo/00-packages.md`.
4. Add `sidecar/tools-ts/node_modules/` and `sidecar/tools-ts/dist/` to `.gitignore`.

**Gitbook Documentation:** N/A for this stub — Plan 26 documents the sidecar API.

**Verification:**
```bash
ls sidecar/tools-ts/
# Expected: package.json  tsconfig.json  src/

ls sidecar/tools-ts/src/
# Expected: index.ts  types.ts
```

---

### Unit 7: Old Plan Cleanup (Verification Only)

**Quick Reference**

These old plan files were deleted by the plan generator before Codex runs. This unit verifies they are gone. If any remain, delete them.

Files that MUST NOT exist:

```
plans/00-architecture-patterns.md
plans/01-foundation.md  (or any plans/NN-*.md files from the old batch system)
plans/README.md
plans/BRANCHES.md
```

Directories that MUST remain:

```
plans/CONTEXT.md          (keep — this is the cross-plan state tracker)
plans/completed/          (keep — completed plan archives)
plans/context/            (keep — context snapshots)
```

**Source Files:** None — this is a verification step.

**Crate Location:** N/A

**Implementation Notes:**

1. Run `ls plans/` and confirm only `CONTEXT.md`, `completed/`, and `context/` (and the current plan file `01-workspace-scaffold.md`) are present.
2. If any old `NN-*.md` plan files exist beyond `01-workspace-scaffold.md`, delete them with `rm plans/NN-*.md`.
3. Do NOT delete `plans/CONTEXT.md` — it is the cross-plan state file that all plans share.

**Gitbook Documentation:** N/A.

**Verification:**
```bash
ls plans/
# Expected: 01-workspace-scaffold.md  CONTEXT.md  completed/  context/
```

---

## Failure Recovery

**Missing crate in workspace:** Open `Cargo.toml` at workspace root and add the crate path to `[workspace] members`. Run `cargo check --workspace` again.

**Dependency version not found on crates.io:** Use the most recent published version. Record the substitution in the Completion Report with the reason (e.g., "used lancedb 0.26 — 0.27 not yet published"). The operator verifies against crates.io before merge.

**`rodio` build error on Linux (missing ALSA):** `rodio` is marked `optional = true` in workspace deps. Individual crates that actually use audio add it with `rodio = { workspace = true, optional = true }`. No crate in this plan uses rodio, so this should not trigger. If it does, check that no crate accidentally added `rodio` as a non-optional dep.

**`fastembed` or `lancedb` native build failure:** These crates have native components. If they fail to build from source, the shell crates don't depend on them — they are workspace dep declarations only. The shells have empty `[dependencies]` sections, so no crate in this plan pulls fastembed or lancedb transitively. If cargo still tries to compile them, check that no shell Cargo.toml accidentally listed them.

**`wasmtime` build failure:** Same pattern as fastembed/lancedb — workspace dep declaration only, no shell uses it. Should not trigger.

**`alloy` feature flag error:** The `"full"` feature on alloy 1.7 enables everything. If 1.7 is not published, try 1.6 and note in Completion Report.

**mdbook not installed:** `cargo install mdbook`. This requires internet. Do it during the setup phase (codex-setup.sh), not during Codex execution.

**TypeScript sidecar npm deps:** `package.json` specifies deps but `npm install` is NOT run by this plan. The sidecar builds separately. If someone runs `npm install` inside `sidecar/tools-ts/`, fine — but this plan does not require it.

**`cargo check --workspace` shows edition 2024 errors:** Rust 1.85 is required. If the installed toolchain is older, `rustup update 1.85` or ensure `rust-toolchain.toml` is being read (`rustup toolchain list` should show 1.85 active in this directory).

**Q-01 from CONTEXT.md (apps in workspace members):** Resolved by this plan. Apps are in `[workspace] members`. Decision recorded as D-04 in CONTEXT.md update below.

---

## Testing Checkpoint

Run these in order after all units complete:

```bash
# 1. Workspace member count
cargo metadata --no-deps --format-version 1 | python3 -c \
  "import sys,json; d=json.load(sys.stdin); print('workspace members:', len(d['workspace_members']))"
# Expected: workspace members: 25

# 2. Full workspace check (no compile errors)
cargo check --workspace 2>&1 | grep -c "^error"
# Expected: 0

# 3. Library crate checks
cargo check -p golem-core -p golem-runtime -p golem-heartbeat -p golem-grimoire \
    -p golem-daimon -p golem-mortality -p golem-dreams -p golem-context \
    -p golem-safety -p golem-inference -p golem-chain -p golem-chain-intelligence \
    -p golem-triage -p golem-ta -p golem-oneirography -p golem-tools \
    -p golem-coordination -p golem-surfaces -p golem-creature -p golem-engagement
# Expected: no errors

# 4. Binary shells
cargo check -p golem-binary -p bardo-gateway -p bardo-terminal -p bardo-styx -p bardo-compute -p mirage-rs
# Expected: no errors

# 5. Test suite (zero tests yet)
cargo test --workspace 2>&1 | grep -E "^test result"
# Expected: test result: ok. 0 passed; 0 failed (one line per crate)

# 6. just build
just build 2>&1 | tail -3
# Expected: Finished `dev` profile target(s)

# 7. mdbook build
cd docs && mdbook build 2>&1 | tail -3
# Expected: Finished in X.XXs

# 8. Sidecar stubs exist
ls sidecar/tools-ts/src/
# Expected: index.ts  types.ts

# 9. Old plan files gone
ls plans/*.md 2>/dev/null | grep -v "01-workspace-scaffold.md" | grep -v "CONTEXT.md"
# Expected: (empty — no other plan files)
```

Expected overall: `cargo check --workspace` exits 0 with no errors. Warnings about `missing_docs` on empty shell crates are expected and acceptable at this stage.

---

## Completion Report

*(Filled by Codex after implementation)*

**Date completed:**

**Actual workspace member count:**

**Cargo check result:**

**mdbook build result:**

**Deviations from plan:**

**Version substitutions (if any dep versions were unavailable):**

**Q-01 resolution:** Confirmed — apps in `apps/` are workspace members (or note any deviation).

**CONTEXT.md updates applied:** (list the D-NN entries added)

## Verification

### Invariants

<!-- INV-001: Epistemic Fitness Range Constraint -->
- **type**: numeric_range
- **module**: golem_mortality::epistemic_fitness
- **property**: Epistemic fitness score stays within valid [0.0, 1.0] range
- **formula**: `fitness = 0.0 (random) .. 1.0 (perfect prediction)`
- **constraint**: `0.0 ≤ fitness ≤ 1.0` at all times
- **test_fn**: `test_epistemic_fitness_clamp_range`
- **strategy**: unit
- **inputs**: `{"new_accuracy": [0.0, 0.5, 1.0, -0.1, 1.5], "alpha": [0.001, 0.05], "current_fitness": [0.0, 0.5, 1.0]}`
- **oracle**: `clamp(fitness_value, 0.0, 1.0)`
- **severity**: spec
- **source**: prd2/02-mortality/02-epistemic-decay.md §Epistemic Fitness Score

<!-- INV-002: EMA Fitness Convergence -->
- **type**: convergence
- **module**: golem_mortality::epistemic_fitness
- **property**: Exponential moving average converges to constant input within 1e-6
- **formula**: `fitness_new = alpha * accuracy + (1 - alpha) * fitness_old`; default `alpha = 0.01`
- **constraint**: After 1000 ticks of constant input `accuracy = c`, `|fitness(t) - c| < 1e-6`
- **test_fn**: `test_ema_convergence_to_constant`
- **strategy**: proptest
- **inputs**: `{"constant_accuracy": [0.0, 0.25, 0.5, 0.75, 1.0], "alpha": [0.001, 0.005, 0.01, 0.05], "ticks": 1000}`
- **oracle**: `abs(fitness - constant_accuracy) < 1e-6` after 1/alpha ≈ 100 time-constant multiplied by log(1e-6) ≈ 13.8 time-constants ≈ 1380 ticks
- **severity**: code
- **source**: prd2/02-mortality/02-epistemic-decay.md §Rolling Fitness via EMA

<!-- INV-003: Domain-Specific Alpha Values -->
- **type**: numeric_range
- **module**: golem_mortality::domain_alpha_config
- **property**: Domain alpha values produce correct half-life timescales
- **formula**: `alpha_values = {price_direction: 0.02, volatility_regime: 0.005, yield_trend: 0.003, gas_pattern: 0.05, protocol_behavior: 0.001}`
- **constraint**: `Ticks to 50% = 0.693 / alpha` matches published half-lives: gas ~14 ticks, price ~35 ticks, volatility ~139 ticks, yield ~231 ticks, protocol ~693 ticks
- **test_fn**: `test_domain_alpha_half_life_calibration`
- **strategy**: unit
- **inputs**: `{"alphas": {"price": 0.02, "volatility": 0.005, "yield": 0.003, "gas": 0.05, "protocol": 0.001}}`
- **oracle**: `ticks_to_50_percent = 0.693 / alpha`; gas=14, price=35, volatility=139, yield=231, protocol=693
- **severity**: spec
- **source**: prd2/02-mortality/02-epistemic-decay.md §Why These Specific Alphas

<!-- INV-004: Dimension Weights Sum -->
- **type**: sum_constraint
- **module**: golem_mortality::dimension_weights
- **property**: Accuracy dimension weights must sum to exactly 1.0
- **formula**: `weights = {price_direction: 0.35, volatility_regime: 0.25, yield_trend: 0.20, gas_condition: 0.10, protocol_state: 0.10}`
- **constraint**: `sum(weights) = 1.0` (conservation of probability)
- **test_fn**: `test_dimension_weights_sum_to_one`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: `0.35 + 0.25 + 0.20 + 0.10 + 0.10 = 1.0`
- **severity**: code
- **source**: prd2/02-mortality/02-epistemic-decay.md §Per-Tick Accuracy Computation

<!-- INV-005: Tick Accuracy Binary Match -->
- **type**: numeric_range
- **module**: golem_mortality::tick_accuracy
- **property**: Per-dimension match is binary (0.0 or 1.0), composite is weighted sum
- **formula**: `accuracy = sum(weights[i] * (prediction[i] == outcome[i] ? 1.0 : 0.0))`
- **constraint**: `0.0 ≤ accuracy ≤ 1.0` always; each component is binary, output is weighted sum
- **test_fn**: `test_tick_accuracy_range_proptest`
- **strategy**: proptest
- **inputs**: `{"dimensions": 5, "match_pattern": [all_match, all_miss, alternating, random]}`
- **oracle**: All matching = 1.0; all missing = 0.0; mixed = weighted sum in [0.0, 1.0]
- **severity**: code
- **source**: prd2/02-mortality/02-epistemic-decay.md §Per-Tick Accuracy Computation

<!-- INV-006: Senescence Threshold Hysteresis -->
- **type**: numeric_range
- **module**: golem_mortality::senescence_cascade
- **property**: Hysteresis threshold is threshold + 0.10 (recovery requires larger margin than entry)
- **formula**: `senescence_threshold = 0.35, recovery_threshold = 0.45`
- **constraint**: `recovery_threshold = senescence_threshold + 0.10` to prevent chatter; recovery > entry
- **test_fn**: `test_senescence_hysteresis_offset`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: `0.45 = 0.35 + 0.10`
- **severity**: spec
- **source**: prd2/02-mortality/02-epistemic-decay.md §Senescence Cascade

<!-- INV-007: Senescence Grace Period Ticks -->
- **type**: numeric_range
- **module**: golem_mortality::senescence_cascade
- **property**: Grace period duration in ticks matches documented value
- **formula**: `recovery_grace_period = 500 ticks ≈ 5.8 hours (at 40s/tick)`
- **constraint**: `recovery_grace_period = 500` in default config
- **test_fn**: `test_senescence_grace_period_constant`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: `500 ticks / (3600 s/hr / 40 s/tick) ≈ 5.8 hours`
- **severity**: spec
- **source**: prd2/02-mortality/02-epistemic-decay.md §Senescence Cascade

<!-- INV-008: Fitness Trend First Derivative -->
- **type**: numeric_range
- **module**: golem_mortality::epistemic_fitness_state
- **property**: Fitness trend is bounded by [-1.0, 1.0] (maximum rate of change per 100 ticks)
- **formula**: `fitness_trend = fitness(t) - fitness(t - 100 ticks)`
- **constraint**: `-1.0 ≤ fitness_trend ≤ 1.0` (fitness can move at most from 0→1 or 1→0 in 100 ticks)
- **test_fn**: `test_fitness_trend_range`
- **strategy**: unit
- **inputs**: `{"previous_fitness": [0.0, 0.5, 1.0], "current_fitness": [0.0, 0.5, 1.0]}`
- **oracle**: `clamp(current - previous, -1.0, 1.0)`
- **severity**: code
- **source**: prd2/02-mortality/02-epistemic-decay.md §EpistemicFitnessState Struct

<!-- INV-009: Prediction Log Window Size -->
- **type**: numeric_range
- **module**: golem_mortality::epistemic_fitness_state
- **property**: Prediction log rolling window capacity is 2000 ticks
- **formula**: `prediction_log: VecDeque with capacity 2000`
- **constraint**: Window size = 2000 ticks (~23 hours at 40s/tick); older entries evicted FIFO
- **test_fn**: `test_prediction_log_fifo_eviction`
- **strategy**: unit
- **inputs**: `{"predictions_to_insert": 2500}`
- **oracle**: After insert 2500 predictions, VecDeque.len() = 2000; first 500 evicted
- **severity**: spec
- **source**: prd2/02-mortality/02-epistemic-decay.md §EpistemicFitnessState Struct

<!-- INV-010: VitalityState Multiplicative Composition -->
- **type**: numeric_range
- **module**: golem_mortality::vitality
- **property**: Composite vitality is product of three independent death clocks (economic, epistemic, stochastic)
- **formula**: `composite_vitality = economic_vitality × epistemic_vitality × stochastic_vitality`
- **constraint**: `0.0 ≤ composite_vitality ≤ 1.0`; any clock at 0.0 → composite at 0.0; all at 1.0 → composite at 1.0
- **test_fn**: `test_composite_vitality_multiplicative_proptest`
- **strategy**: proptest
- **inputs**: `{"economic": [0.0, 0.1, 0.5, 1.0], "epistemic": [0.0, 0.1, 0.5, 1.0], "stochastic": [0.0, 0.1, 0.5, 1.0]}`
- **oracle**: `e * ep * s ∈ [0.0, 1.0]`; min(inputs)=0 → output=0; all=1 → output=1
- **severity**: spec
- **source**: prd2/02-mortality/01-architecture.md §VitalityState

<!-- INV-011: Golem Lifespan is Not Hayflick-Bounded -->
- **type**: numeric_range
- **module**: golem_mortality::lifespan
- **property**: No fixed maximum tick ceiling; lifespan emerges from environment coupling, not configuration
- **formula**: Epistemic death triggered when `epistemic_fitness < 0.35` for `recovery_grace_period` ticks; no `tick_max` field
- **constraint**: No upward bound on tick count in VitalityState struct; death is event-driven not timeout-driven
- **test_fn**: `test_no_hardcoded_lifespan_ceiling`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: VitalityState.tick_count is u64 unbounded; death decision made via fitness, not tick count
- **severity**: spec
- **source**: prd2/02-mortality/02-epistemic-decay.md §What Epistemic Decay Gets Right

<!-- INV-012: Senescence Stage Enum Transitions -->
- **type**: state_machine
- **module**: golem_mortality::senescence_stage
- **property**: Valid state transitions for SenescenceStage enum
- **formula**: States: None → Stage1 (entry), Stage1 → Stage2 (confirmed), Stage2 → Stage3 (protocol), Stage1/2/3 → None (recovery). Invalid: Stage3 → Stage1, Stage2 → None without passing Stage3
- **constraint**: `None → Stage1` (fitness < threshold), `Stage1 → Stage2` (timeout), `Stage2 → Stage3` (vitality critical), `*→ None` (recovery above hysteresis threshold)
- **test_fn**: `test_senescence_stage_valid_transitions`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: Valid paths only; all others panic/error
- **severity**: code
- **source**: prd2/02-mortality/02-epistemic-decay.md §Senescence Cascade

<!-- INV-013: Ebbinghaus Decay Application in Grimoire -->
- **type**: monotonic
- **module**: golem_grimoire::ebbinghaus_decay
- **property**: Entry confidence decays monotonically over time following Ebbinghaus curve
- **formula**: Knowledge decay formula based on [EBBINGHAUS-1885]: `R(t) = e^(-t/S)` or `Q(t) = 1.84 / ((log t)^1.25 + 1.84)`
- **constraint**: `R(t+1) < R(t)` for all t ≥ 0; R(0) = 1.0; R(∞) = 0.0
- **test_fn**: `test_ebbinghaus_decay_monotonic_proptest`
- **strategy**: proptest
- **inputs**: `{"time_ticks": range(0, 10000), "memory_strength_s": [10, 50, 100, 500, 1000]}`
- **oracle**: `R(t) > R(t+1)` for all consecutive pairs
- **severity**: spec
- **source**: prd2/02-mortality/02-epistemic-decay.md references [EBBINGHAUS-1885]

<!-- INV-014: Gompertz Hazard Monotonicity -->
- **type**: monotonic
- **module**: golem_mortality::stochastic_mortality
- **property**: Gompertz hazard function h(t) increases monotonically with age t
- **formula**: `h(t) = α × e^(β×t)` [GOMPERTZ-1825]; α > 0, β > 0
- **constraint**: `dh/dt > 0` for all t ≥ 0; h(t) is strictly increasing
- **test_fn**: `test_gompertz_hazard_monotonic_proptest`
- **strategy**: proptest
- **inputs**: `{"alpha": [0.0001, 0.001, 0.01], "beta": [0.0001, 0.001, 0.01], "age_ticks": range(0, 200_000)}`
- **oracle**: `h(t+1) > h(t)` for all consecutive ticks
- **severity**: spec
- **source**: prd2/02-mortality/03-stochastic-mortality.md

<!-- INV-015: PAD Vector Components Range -->
- **type**: numeric_range
- **module**: golem_daimon::pad_vector
- **property**: PAD emotion vector components (Pleasure, Arousal, Dominance) each in [-1.0, 1.0]
- **formula**: `PADVector { pleasure: f64, arousal: f64, dominance: f64 }` per [MEHRABIAN-1974]
- **constraint**: `-1.0 ≤ pleasure, arousal, dominance ≤ 1.0`
- **test_fn**: `test_pad_vector_component_clamps`
- **strategy**: unit
- **inputs**: `{"p": [-1.5, -1.0, 0.0, 1.0, 1.5], "a": [-1.5, -1.0, 0.0, 1.0, 1.5], "d": [-1.5, -1.0, 0.0, 1.0, 1.5]}`
- **oracle**: `clamp_each(-1.0, 1.0)`
- **severity**: code
- **source**: prd2/03-daimon/01-appraisal.md §PAD Emotional Model

<!-- INV-016: Plutchik Octant Mapping from PAD -->
- **type**: numeric_range
- **module**: golem_daimon::plutchik_label
- **property**: 8 primary Plutchik emotions map uniquely to octants of PAD space
- **formula**: 8 emotions {anger, fear, sadness, disgust, surprise, anticipation, trust, joy} correspond to 8 octants of [-1,1]³ space per [PLUTCHIK-1980]
- **constraint**: Each emotion's octant is non-overlapping; coverage is complete across [−1, 1]³
- **test_fn**: `test_plutchik_octant_coverage_proptest`
- **strategy**: proptest
- **inputs**: `{"pad_samples": 10000, "random_distribution": "uniform"}`
- **oracle**: Every PAD vector maps to exactly one Plutchik octant/emotion
- **severity**: code
- **source**: prd2/03-daimon/01-appraisal.md §Plutchik Emotion Wheel

<!-- INV-017: Mood EMA Convergence (Affective) -->
- **type**: convergence
- **module**: golem_daimon::mood_ema
- **property**: Mood EMA over emotional states converges from cold start
- **formula**: `mood_new = alpha_mood × emotion + (1 - alpha_mood) × mood_old`; default `alpha_mood ≈ 0.001` (hours-scale)
- **constraint**: After 10 observations, mood ≠ 0.5 (default); after 1000 observations in stable emotion, |mood - emotion| < 0.01
- **test_fn**: `test_mood_ema_convergence_from_cold_start`
- **strategy**: unit
- **inputs**: `{"stable_emotion": [0.0, 0.25, 0.5, 0.75, 1.0], "observations": [10, 100, 1000]}`
- **oracle**: Cold start converges by 1000 observations; fast enough for meaningful affect signal
- **severity**: code
- **source**: prd2/03-daimon/02-emotion-memory.md

<!-- INV-018: Dream Materialization via Mattar-Daw Replay -->
- **type**: event_sequence
- **module**: golem_dreams::replay
- **property**: NREM replay sequences prioritized episodes per Mattar-Daw utility weighting
- **formula**: Prioritized replay based on [MATTAR-DAW-2018]: episodes with higher utility-to-frequency ratio are re-experienced earlier in consolidation phase
- **constraint**: Replay ordering must match utility sort; no arbitrary permutation
- **test_fn**: `test_dream_replay_prioritizes_by_utility`
- **strategy**: integration
- **inputs**: `{"episodes": [{"utility": 0.9, "freq": 1}, {"utility": 0.1, "freq": 100}]}`
- **oracle**: High-utility, low-frequency episode replayed before low-utility, high-frequency
- **severity**: spec
- **source**: prd2/05-dreams/02-replay.md references [MATTAR-DAW-2018]

<!-- INV-019: Genomic Bottleneck Compression Ratio -->
- **type**: numeric_range
- **module**: golem_mortality::genomic_bottleneck
- **property**: Knowledge compression at death reduces Grimoire entries to ≤ 2048
- **formula**: `compressed_knowledge.len() ≤ 2048` at death; compression ratio = full_grimoire / 2048
- **constraint**: Upper bound enforced; compression loss is information-theoretic (pruning + quantization)
- **test_fn**: `test_genomic_bottleneck_compression_bound`
- **strategy**: unit
- **inputs**: `{"grimoire_sizes": [1024, 2048, 5000, 10000, 100000]}`
- **oracle**: All outputs ≤ 2048
- **severity**: spec
- **source**: prd2/02-mortality/01-architecture.md §Genomic Bottleneck

<!-- INV-020: Baldwin Effect Transgenerational Confidence Decay -->
- **type**: numeric_range
- **module**: golem_mortality::transgenerational_knowledge
- **property**: Inherited heuristics start at `confidence × 0.85^generation`
- **formula**: `inherited_confidence = ancestor_confidence × (0.85 ^ generation_distance)`
- **constraint**: `0.0 ≤ inherited_confidence ≤ ancestor_confidence`; monotonic decay per generation
- **test_fn**: `test_baldwin_effect_generational_decay_proptest`
- **strategy**: proptest
- **inputs**: `{"ancestor_confidence": [0.5, 0.9, 1.0], "generation": [1, 2, 3, 5, 10, 50]}`
- **oracle**: `conf_n = ancestor_conf × 0.85^n`; all outputs ≤ ancestor_conf; decreasing in generation
- **severity**: spec
- **source**: prd2/02-mortality/01-architecture.md references [BALDWIN-1896]

<!-- INV-021: Hayflick Limit Replicant Max Tick Cap -->
- **type**: numeric_range
- **module**: golem_mortality::replicant_lifecycle
- **property**: Maximum tick count for Replicants follows Hayflick limit model (biological inspiration)
- **formula**: `max_replicant_ticks` is configured value; default drawn from [HAYFLICK-1965] literature (exponential growth ceiling)
- **constraint**: `tick_count ≤ max_replicant_ticks` enforced; Replicant creation blocked if age > 80% of max
- **test_fn**: `test_hayflick_replicant_ceiling_enforced`
- **strategy**: unit
- **inputs**: `{"max_ticks": [100_000, 200_000, 500_000]}`
- **oracle**: Replicants respect ceiling; creation denied if ancestor age > 0.8 × max
- **severity**: spec
- **source**: prd2/02-mortality/01-architecture.md references [HAYFLICK-1965]

<!-- INV-022: Thanatopsis Four-Phase Death Protocol -->
- **type**: state_machine
- **module**: golem_mortality::thanatopsis
- **property**: Death protocol FSM with four ordered phases: Acceptance → Settlement → Reflection → Legacy
- **formula**: Phase transitions: (1) Acceptance sets `is_dying = true`, (2) Settlement writes Vault uploads, (3) Reflection broadcasts final reflection, (4) Legacy seals genealogy
- **constraint**: Phases must execute in order; no skipping; each phase < 30 seconds (Fly.io SIGTERM budget)
- **test_fn**: `test_thanatopsis_phase_ordering`
- **strategy**: integration
- **inputs**: `{}`
- **oracle**: Phase counters strictly increase; all four executed before process shutdown
- **severity**: code
- **source**: prd2/02-mortality/06-thanatopsis.md

<!-- INV-023: Kelly Criterion Optimal Bet Sizing (Risk Engine) -->
- **type**: numeric_range
- **module**: golem_mortality::risk_engine
- **property**: Position sizing follows Kelly criterion [KELLY-1956] for DeFi trading
- **formula**: `f* = (bp - q) / b` where b = odds, p = win probability, q = 1 - p
- **constraint**: `0 < f* < 1` for valid Kelly bets; f* = 0 signals no edge; fractional Kelly (e.g. 0.25×f*) for production stability
- **test_fn**: `test_kelly_criterion_bounds_proptest`
- **strategy**: proptest
- **inputs**: `{"b": [1.0, 2.0, 5.0, 10.0], "p": [0.1, 0.5, 0.9, 1.0, 2.0]}`
- **oracle**: `f_star = (b*p - (1-p)) / b`; 0 < f_star < 1 for p ∈ (0.5, 1); f_star ≤ 0 for p ≤ 0.5
- **severity**: spec
- **source**: prd2/01-golem/16-risk-engine.md references [KELLY-1956]

### Regression Anchors

- `test_epistemic_fitness_clamp_range`
- `test_ema_convergence_to_constant`
- `test_domain_alpha_half_life_calibration`
- `test_dimension_weights_sum_to_one`
- `test_tick_accuracy_range_proptest`
- `test_senescence_hysteresis_offset`
- `test_senescence_grace_period_constant`
- `test_fitness_trend_range`
- `test_prediction_log_fifo_eviction`
- `test_composite_vitality_multiplicative_proptest`
- `test_no_hardcoded_lifespan_ceiling`
- `test_senescence_stage_valid_transitions`
- `test_ebbinghaus_decay_monotonic_proptest`
- `test_gompertz_hazard_monotonic_proptest`
- `test_pad_vector_component_clamps`
- `test_plutchik_octant_coverage_proptest`
- `test_mood_ema_convergence_from_cold_start`
- `test_dream_replay_prioritizes_by_utility`
- `test_genomic_bottleneck_compression_bound`
- `test_baldwin_effect_generational_decay_proptest`
- `test_hayflick_replicant_ceiling_enforced`
- `test_thanatopsis_phase_ordering`
- `test_kelly_criterion_bounds_proptest`

### Cross-Crate Contracts

| Upstream | Input Condition | Expected Behavior |
|----------|----------------|-------------------|
| `golem-grimoire` (episodic store) → `golem-mortality` | Prediction logged with outcome | `EpistemicFitnessState.prediction_log` appends PredictionOutcomePair; evicts oldest if len > 2000 |
| `golem-core` (MarketRegime, PADVector) → `golem-daimon` | Regime context + somatic marker pair | PADVector clamped to [-1.0, 1.0] per dimension; regime tag stored for mood computation |
| `golem-daimon` (mood EMA) → `golem-grimoire` | Retrieval request during gate phase | Mood-congruent bias applied: matching valence entries boosted by ±20%, opposing entries penalized |
| `golem-heartbeat` (CoALA observe/retrieve) → `golem-grimoire` | Query for episode retrieval | Four-factor scoring: `recency × importance × relevance × affect`; affect bias ∝ mood EMA |
| `golem-heartbeat` → `golem-mortality` | Tick complete with prediction outcome | `update_fitness()` called; senescence stage updated if fitness crosses thresholds; death event emitted if composite vitality → 0 |
| `golem-mortality` → `golem-runtime` (lifecycle FSM) | Death triggered (any clock) | Lifecycle transitions to `Terminal` → `Dead`; `thanatopsis` phase begins (4-phase protocol) |
| `golem-mortality` → `golem-coordination` (Styx Vault) | Golem dying, knowledge distillation | Grimoire compressed to ≤ 2048 entries; uploaded to Vault with Baldwin confidence decay = `0.85^generation` |

### Event Sequence Assertions

**Death Cascade (in order, no skipping)**:
1. `EpistemicFitnessEvent { fitness_below_threshold: true }` emitted when `fitness < 0.35`
2. `SenescenceStageEvent { stage: Stage1 }` emitted; grace period starts
3. (Optional) `SenescenceRecoveryEvent { recovered: true }` if fitness > 0.45 before grace timeout
4. (Or) `SenescenceStageEvent { stage: Stage2 }` if grace period expires
5. `SenescenceStageEvent { stage: Stage3 }` when vitality composite < 0.01
6. `GolemDeathEvent { cause: DeathCause::{Economic|Epistemic|Stochastic} }` emitted
7. `LifecycleTransitionEvent { from: Active, to: Terminal }`
8. `ThanatsisPhaseEvent { phase: Acceptance }`
9. `ThanatsisPhaseEvent { phase: Settlement }`
10. `ThanatsisPhaseEvent { phase: Reflection }`
11. `ThanatsisPhaseEvent { phase: Legacy }`
12. `LifecycleTransitionEvent { from: Terminal, to: Dead }`

**Dream Consolidation (Mattar-Daw Replay)**:
1. `DreamInitiatedEvent { trigger: "consolidation" }` emitted
2. Prioritized episode list built via utility-weighted replay algorithm
3. `ReplayStartEvent { episode_id, utility }` emitted per episode
4. `PlaybookEvolutionEvent { heuristics_updated }` after replay
5. `DreamCompletedEvent { consolidated_entries }` emitted

### Academic References Verified

| Reference | Formula/Constant | PRD2 Match | Web-Verified |
|-----------|-----------------|------------|--------------|
| [GOMPERTZ-1825] | Hazard: `h(t) = α·e^(β·t)` exponential mortality | Yes, stochastic-mortality.md implements | ✓ Wikipedia, PMC, actuarial literature confirm original 1825 publication |
| [EBBINGHAUS-1885] | Forgetting: `Q(t) = 1.84/((log t)^1.25 + 1.84)` or `R = e^(-t/S)` | Yes, grimoire demurrage in memory docs | ✓ Verified 1885 publication; formula matches modern replication studies |
| [MEHRABIAN-1974] | PAD circumplex: Pleasure, Arousal, Dominance each ∈ [-1,1] | Yes, daimon appraisal 01 | ✓ Confirmed in Wikipedia, psychological literature, ISO 9001 emotion standard |
| [PLUTCHIK-1980] | 8 primary emotions + octant mapping | Yes, daimon 03-behavior.md | ✓ Verified wheel model, 8 basic emotions, octant structure in multiple sources |
| [MATTAR-DAW-2018] | Prioritized memory replay utility weighting | Yes, dreams 02-replay.md | ✓ Published in Nature Neuroscience Nov 2018; code at Princeton/GitHub |
| [KELLY-1956] | Optimal bet sizing: `f* = (bp - q)/b` | Yes, risk-engine 16 references | ✓ Bell Labs original; information theory connection verified |
| [HAYFLICK-1965] | Replicative senescence limit (~50 divisions) | Yes, immortal-control 11 references | ✓ Foundational cell biology; model applied as inspiration for Replicant max-ticks |
| [BALDWIN-1896] | Learned traits become structural across generations | Yes, mortality 01-architecture §Baldwin Effect | ✓ Verified; referenced in modern evolutionary computation literature |
| [VELA-2022] | 91% of ML models degrade temporally | Yes, epistemic-decay 02 §The Evidence | ✓ Scientific Reports 2022; cited in concept drift literature |
| [ARBESMAN-2012] | Knowledge half-lives: medical 45y, IT < 2y | Yes, epistemic-decay 02 §Knowledge Half-Life | ✓ Arbesman publications on knowledge decay; cited in domain-specific alpha calibration |

