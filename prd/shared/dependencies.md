# External Dependencies Catalog [SPEC]

> **Document Type**: REF (normative) | **Referenced by**: All PRDs | **Last Updated**: 2026-03-15
>
> Comprehensive catalog of external dependencies across the Bardo ecosystem. For each dependency: purpose, consuming products, required vs optional status, cost model, alternatives, and self-hostability.

> **Reader orientation:** This document catalogs every external dependency across the Bardo ecosystem, from infrastructure services (Fly.io, Turso, Privy) to blockchain protocols (Uniswap, ERC-8004) to Rust and TypeScript libraries. It belongs to the `shared/` reference layer. The key concept is the managed vs self-sovereign spectrum: for each dependency, Bardo provides a hosted path and a self-hostable alternative, so Golem (mortal autonomous agent) operators can choose their trust model. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. Infrastructure Services

| Service            | Purpose                                                                                                                           | Products           | Required?                                       | Cost Model                                                                            | Alternatives                                            | Self-Hostable?                                                                                            |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------- | ------------------ | ----------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| **Fly.io**         | VM hosting for Bardo Compute (Golem runtimes; Golem = mortal autonomous agent compiled as a single Rust binary). Two-app topology: `bardo-control` (orchestrator) and `bardo-machines` (golem VMs). | Compute, Bott      | Yes (for hosted golems)                         | Pay-per-use VMs. Micro: $0.025/hr, Small: $0.05/hr, Medium: $0.10/hr, Large: $0.20/hr | AWS EC2, GCP Compute, Hetzner, self-hosted Docker       | No (Fly.io APIs used for machine lifecycle), but golem VMs can run on any Docker host in self-hosted mode |
| **Turso (libSQL)** | Database for Compute state persistence. TTL tracking, session state, billing records. CAS (compare-and-swap) for TTL enforcement. | Compute            | Yes (for hosted)                                | Free tier (500 databases, 9GB storage, 1B row reads/mo) + usage-based                 | SQLite (local), Postgres, PlanetScale                   | Yes (libSQL is open source; run your own Turso-compatible server)                                         |
| **Cloudflare R2**  | Object storage for Grimoire snapshots, insurance backups, and golem state exports.                                                | Compute, Golem     | Yes (for hosted)                                | Free: 10GB storage, 10M reads/mo. Then $0.015/GB/mo storage, $0.36/M reads            | AWS S3, GCP Cloud Storage, MinIO                        | Yes (MinIO is S3-compatible and self-hostable)                                                            |
| **Privy**          | Wallet creation, TEE key management, P-256 session signers, and signing policies. Production wallet provider.                     | All (wallet layer) | Recommended (local key fallback for dev)        | API usage-based. Free tier available.                                                 | Turnkey, Capsule, Lit Protocol, local key (dev only)    | No (TEE infrastructure is Privy-managed)                                                                  |
| **Alchemy**        | RPC provider for Ethereum and L2 chains. Fallback/primary chain access.                                                           | All (chain access) | Optional (public RPCs work but rate-limited)    | Free tier (300M compute units/mo) + usage                                             | Infura, QuickNode, Ankr, public RPCs, self-hosted nodes | Yes (run your own Ethereum/L2 nodes)                                                                      |
| **PostHog**        | Product analytics, funnel tracking, feature flags. HMAC anonymization for privacy.                                                | All (telemetry)    | Optional (disable with `BARDO_TELEMETRY=false`) | Free tier (1M events/mo, 15K session replays/mo) + usage                              | Plausible, Mixpanel, self-hosted PostHog                | Yes (PostHog is open source and self-hostable)                                                            |

---

## 2. Blockchain Dependencies

| Dependency            | Purpose                                                                         | Version/Address                                           | Required?            | Products               | Notes                                                        |
| --------------------- | ------------------------------------------------------------------------------- | --------------------------------------------------------- | -------------------- | ---------------------- | ------------------------------------------------------------ |
| **Uniswap V2**        | Legacy constant product AMM. Read-only pool data and historical analysis.       | Deployed on Ethereum, Base                                | Yes (read)           | Sanctum, Golem         | No custom contract deployment needed                         |
| **Uniswap V3**        | Concentrated liquidity AMM. LP management, position tracking, swap routing.     | Deployed on all 11 chains                                 | Yes (read/write)     | Sanctum, Golem, Vaults | Primary LP protocol for v1                                   |
| **Uniswap V4**        | Hook-based customizable AMM. Singleton architecture. Share pools, dynamic fees. | PoolManager: `0x000000000004444c5dc75cB358380D2e3dE08A90` | Yes (vault hooks)    | Vaults, Sanctum        | V4-enabled chains: Ethereum, Base, Unichain                  |
| **UniswapX**          | Dutch auction order protocol. MEV-protected swap execution.                     | Deployed on 6 chains                                      | Yes (MEV protection) | Sanctum, Golem         | Filler network required                                      |
| **ERC-8004 Registry** | Agent identity registration, metadata, reputation attestations.                 | `0x8004A818BFB912233c491871b3d84c89A494BD9e`              | Yes (identity layer) | All                    | Same address on all deployed chains. Primary: Ethereum, Base |
| **Permit2**           | Batch token approvals and signature-based transfers.                            | `0x000000000022D473030F116dDEE9F6B43aC78BA3`              | Yes (transfers)      | Vaults, Sanctum        | Canonical Uniswap contract                                   |
| **ERC-4337**          | Account abstraction. Smart contract wallets with UserOperations.                | Standard                                                  | Yes (smart accounts) | Wallet, Golem          | Bundlers: Alchemy, Pimlico, Stackup                          |
| **USDC**              | Payment token for x402 inference, compute billing, vault deposits.              | Base + Ethereum                                           | Yes (primary unit)   | All                    | Circle-issued stablecoin                                     |
| **Morpho**            | Lending adapter for vault yield strategies. Supply-side lending.                | Base deployment                                           | Phase 1 adapter      | Vaults                 | $5.8B TVL as of Feb 2026 [MORPHO-TVL-2026]                   |
| **Aave V3**           | Lending adapter for vault yield strategies. Supply-side lending.                | Base deployment                                           | Phase 1 adapter      | Vaults                 | Major lending protocol                                       |
| **Pendle**            | Yield tokenization adapter. Separate principal and yield tokens.                | Base deployment                                           | Phase 2+ adapter     | Vaults                 | Deferred track                                               |

---

## 3. Development Dependencies

| Dependency                             | Purpose                                                   | Version  | License        | Notes                                                             |
| -------------------------------------- | --------------------------------------------------------- | -------- | -------------- | ----------------------------------------------------------------- |
| **Node.js**                            | Runtime environment                                       | >= 20    | MIT            | LTS releases preferred                                            |
| **pnpm**                               | Package manager with workspace support                    | 9.x      | MIT            | Pinned via `packageManager` field                                 |
| **Foundry**                            | Solidity development toolkit (forge, cast, anvil, chisel) | Latest   | MIT/Apache 2.0 | forge build, forge test, anvil for local chain                    |
| **tsup**                               | TypeScript bundler (ESM + CJS output)                     | 8.5.x    | MIT            | Used instead of tsdown (rolldown native binding bug with pnpm v9) |
| **vitest**                             | TypeScript testing framework                              | Latest   | MIT            | `passWithNoTests: true` in all configs                            |
| **viem**                               | TypeScript Ethereum client                                | Latest   | MIT            | Only Ethereum client used (not ethers.js)                         |
| **React**                              | UI framework                                              | 19       | MIT            | Used in portal, debug UI, browser SPA                             |
| **Tailwind CSS**                       | Utility-first CSS framework                               | 4        | MIT            | Used with `@bardo/ui` component library                           |
| **Radix UI**                           | Accessible React primitives                               | Latest   | MIT            | Headless components for `@bardo/ui`                               |
| **OpenZeppelin Contracts**             | Audited Solidity libraries                                | v5.5-5.6 | MIT            | ERC4626Upgradeable, OwnableUpgradeable, PausableUpgradeable       |
| **OpenZeppelin Contracts Upgradeable** | Upgrade-safe Solidity libraries                           | v5.6.1   | MIT            | ERC-7201 namespaced storage                                       |
| **forge-std**                          | Foundry standard test library                             | Latest   | MIT            | Test utilities, cheatcodes                                        |
| **ESLint**                             | TypeScript linter                                         | Latest   | MIT            | Shared config in `@bardo/eslint-config`                           |
| **Prettier**                           | Code formatter                                            | Latest   | MIT            | Consistent formatting across packages                             |
| **zod**                                | TypeScript schema validation                              | Latest   | MIT            | All tool input schemas, config validation                         |
| **picocolors**                         | Terminal color output                                     | Latest   | ISC            | Used by TUI package                                               |
| **MSW**                                | Mock Service Worker for API testing                       | Latest   | MIT            | Network-level mocking in tests                                    |

---

## 4. AI and LLM Dependencies

| Dependency                               | Purpose                                                                         | Used By                    | Required?           | Notes                                                           |
| ---------------------------------------- | ------------------------------------------------------------------------------- | -------------------------- | ------------------- | --------------------------------------------------------------- |
| **Anthropic Claude** (Haiku/Sonnet/Opus) | Primary LLM provider. 3-tier routing.                                           | Inference, Golem cognition | Yes (primary)       | Haiku: 95% of calls ($0.80/$4.00 per 1M). Sonnet: 4%. Opus: 1%. |
| **OpenAI** (GPT-4, etc.)                 | Secondary LLM provider for multi-provider routing.                              | Inference (optional)       | Optional            | Provides model diversity and failover                           |
| **Google** (Gemini)                      | Tertiary LLM provider for multi-provider routing.                               | Inference (optional)       | Optional            | Additional model diversity                                      |
| **Golem-RS** (self-contained)            | Rust binary implementing heartbeat FSM, extensions, tool use, JSONL branching. TypeScript Uniswap SDK sidecar over Unix domain socket. | Golem runtime              | Yes (golem core)    | Built from scratch; see `01-golem/00-overview.md`               |

---

## 5. Third-Party Services (Optional)

| Service                 | Purpose                                                                        | Product            | Required?                         | Cost Model                      | Alternatives                                   |
| ----------------------- | ------------------------------------------------------------------------------ | ------------------ | --------------------------------- | ------------------------------- | ---------------------------------------------- |
| **MoonPay**             | Fiat on-ramp for golem USDC funding. Credit card to crypto.                    | Golem funding      | Optional                          | Transaction-based fees          | Transak, Wyre, Ramp                            |
| **LI.FI**               | Cross-chain bridge aggregator for golem funding from non-Base chains.          | Golem funding      | Optional                          | Bridge fees vary                | Socket, Across, Stargate                       |
| **step-ca (Smallstep)** | SSH Certificate Authority for Compute VM access. Short-lived SSH certificates. | Compute security   | Yes (for hosted)                  | Open source (free)              | OpenSSH CA, HashiCorp Vault                    |
| **Inngest**             | Durable workflow engine for golem provisioning pipeline. Idempotency, resume.  | Golem provisioning | Recommended                       | Free tier + usage               | Temporal, custom Inngest-like with Bull/BullMQ |
| **0x API**              | DEX aggregation for optimal swap routing.                                      | Trading tools      | Optional                          | API usage (free tier available) | 1inch API, Paraswap API, on-chain routing only |
| **Basescan**            | Contract verification and block explorer for Base mainnet.                     | Deployment         | Yes (for mainnet)                 | Free (API key for rate limits)  | Blockscout (self-hostable)                     |
| **Pinata**              | IPFS pinning for AgentCard metadata.                                           | Wallet (IPFS mode) | Optional (Bardo proxy is default) | Free tier (500 pins) + usage    | Infura IPFS, self-hosted IPFS node             |
| **CoinGecko**           | Token price data for x402-gated queries.                                       | Sanctum data tools | Optional                          | Free tier + Pro plan            | CoinMarketCap, DefiLlama, on-chain oracles     |

---

## 6. Trust Model Impact

Dependencies affect the managed vs self-sovereign spectrum. Self-hosted alternatives are available for most services:

| Component   | Managed Path            | Self-Sovereign Path     | Trust Implication                                                       |
| ----------- | ----------------------- | ----------------------- | ----------------------------------------------------------------------- |
| Wallet keys | Privy TEE enclave       | Local viem signer       | Managed: keys never leave TEE. Self-sovereign: full key control.        |
| Compute     | Fly.io VMs              | Self-hosted Docker      | Managed: Fly.io has access to VM. Self-sovereign: full isolation.       |
| Database    | Turso (cloud)           | Local SQLite            | Managed: Turso stores state. Self-sovereign: local data only.           |
| Storage     | Cloudflare R2           | MinIO (self-hosted)     | Managed: Cloudflare stores Grimoire. Self-sovereign: local storage.     |
| RPC         | Alchemy/Infura          | Self-hosted node        | Managed: provider sees queries. Self-sovereign: full privacy.           |
| Analytics   | PostHog                 | Disabled or self-hosted | Managed: telemetry sent. Self-sovereign: `BARDO_TELEMETRY=false`.       |
| Identity    | ERC-8004 registry       | Same (on-chain)         | Both: on-chain data is public and self-sovereign by design.             |
| Inference   | Bardo Inference gateway | Direct API keys         | Managed: x402 through gateway. Self-sovereign: bring your own API keys. |

Self-hosted golems store no data with Bardo. The self-sovereign path means operators who run their own infrastructure have no off-chain data subject to third-party access.

---

## 7. Version Pins and Compatibility

| Dependency   | Pinned Version | Reason                                                            |
| ------------ | -------------- | ----------------------------------------------------------------- |
| pnpm         | 9.15.9         | `packageManager` field in `package.json`. Corepack enforced.      |
| tsup         | 8.5.x          | rolldown native binding bug prevents tsdown with pnpm v9          |
| OpenZeppelin | 5.5-5.6        | ERC4626Upgradeable requires v5.5+ for ERC-7201 namespaced storage |
| Node.js      | >= 20          | Required for ESM support, `import.meta.url`, native fetch         |
| React        | 19             | Peer dependency for `@bardo/ui` and `@bardo/portal`               |
| Tailwind CSS | 4              | Major version with new configuration approach                     |
| Rust         | 1.94.0         | Pinned via `rust-toolchain.toml` in `bardo-golem-rs/`             |
| alloy        | 0.15.x         | EVM client for Rust workspace. Not ethers-rs.                     |
| ratatui      | 0.29.x         | Terminal UI for Golem dashboard and bardo-terminal binary         |

---

## 8. Rust Workspace Dependencies (`bardo-golem-rs`)

The Golem runtime is a separate Rust workspace (`bardo-golem-rs/`). These are workspace-level dependency declarations in the root `Cargo.toml`; individual crates inherit versions via `dep.workspace = true`. See `prd2/17-monorepo/01-rust-workspace.md` for the full workspace structure and crate DAG.

### Runtime

| Dependency           | Version | License    | Purpose                                                                                  |
| -------------------- | ------- | ---------- | ---------------------------------------------------------------------------------------- |
| `tokio`              | 1.x     | MIT        | Async runtime. Features: `rt-multi-thread`, `macros`, `signal`. Foundation for all I/O. |
| `axum`               | 0.8.x   | MIT        | HTTP server for REST API, WebSocket endpoints, and health checks in gateway + Styx.      |
| `reqwest`            | 0.12.x  | MIT/Apache | HTTP client for inference providers (Anthropic, OpenAI, Google, Venice, Grok) and APIs. |
| `tokio-tungstenite`  | Latest  | MIT        | WebSocket client for Clade sync and Styx coordination layer.                             |

### EVM / On-Chain

| Dependency | Version | License | Purpose                                                                                              |
| ---------- | ------- | ------- | ---------------------------------------------------------------------------------------------------- |
| `alloy`    | 0.15.x  | MIT     | EVM interaction: RPC client, transaction building, ABI encoding via `sol!`, receipt parsing. Not ethers-rs. Features: `providers`, `signers`, `sol-types`. |

### Storage

| Dependency       | Version | License    | Purpose                                                                              |
| ---------------- | ------- | ---------- | ------------------------------------------------------------------------------------ |
| `sqlx`           | 0.8.x   | MIT/Apache | SQLite async driver for Grimoire structured storage (episodes, insights, heuristics). |
| `lancedb`        | 0.15.x  | Apache 2.0 | Columnar vector database for episodic memory. Stores episode embeddings locally.     |
| `qdrant-client`  | 1.x     | Apache 2.0 | Vector database client for Grimoire semantic search against hosted Qdrant instance.  |

### Serialization

| Dependency    | Version | License    | Purpose                                                                              |
| ------------- | ------- | ---------- | ------------------------------------------------------------------------------------ |
| `serde`       | 1.x     | MIT/Apache | Derive macros for serialization. Used on every data type crossing crate boundaries.  |
| `serde_json`  | 1.x     | MIT/Apache | JSON serialization for API payloads, Pi tool results, and GolemSnapshot exports.     |
| `toml`        | 0.8.x   | MIT/Apache | TOML parsing for `golem.toml` operator config and archetype TOML files.              |

### Observability

| Dependency             | Version | License    | Purpose                                                                            |
| ---------------------- | ------- | ---------- | ---------------------------------------------------------------------------------- |
| `tracing`              | 0.1.x   | MIT        | Structured logging and async-aware instrumentation across all crates.              |
| `tracing-subscriber`   | 0.3.x   | MIT        | Log formatting (JSON or pretty), filtering by crate, OTLP export for telemetry.   |

### Error Handling

| Dependency    | Version | License    | Purpose                                                                                    |
| ------------- | ------- | ---------- | ------------------------------------------------------------------------------------------ |
| `thiserror`   | 2.x     | MIT/Apache | Typed error enums for library crates (`golem-core`, `golem-grimoire`, etc.).               |
| `anyhow`      | 1.x     | MIT/Apache | Error propagation in application crates (`golem-binary`, `apps/`). Context chain on `?`.  |

### Terminal UI

| Dependency  | Version | License | Purpose                                                                                         |
| ----------- | ------- | ------- | ----------------------------------------------------------------------------------------------- |
| `ratatui`   | 0.29.x  | MIT     | Terminal UI framework for the Golem dashboard (golem-surfaces crate) and bardo-terminal binary. |
| `crossterm` | Latest  | MIT     | Cross-platform terminal backend for ratatui. Keyboard input, raw mode, cursor control.          |

### CLI

| Dependency | Version | License    | Purpose                                              |
| ---------- | ------- | ---------- | ---------------------------------------------------- |
| `clap`     | 4.x     | MIT/Apache | CLI argument parsing for `golem-binary` and `apps/`. |

### Utility

| Dependency   | Version | License    | Purpose                                                                                |
| ------------ | ------- | ---------- | -------------------------------------------------------------------------------------- |
| `rand`       | 0.9.x   | MIT/Apache | Stochastic mortality PRNG, dream element selection. Seeded with `BARDO_STOCHASTIC_SEED` in tests. |
| `chrono`     | 0.4.x   | MIT/Apache | Timestamp handling for dream windows, sync scheduling, and TTL tracking.               |
| `dashmap`    | 6.x     | MIT        | Concurrent hash map for connection registries (Styx WebSocket connections).            |
| `uuid`       | 1.x     | MIT/Apache | UUID generation for episode IDs, permit IDs, and golem instance identifiers.           |

### Dev / Test Only

| Dependency        | Version | License | Purpose                                                                                    |
| ----------------- | ------- | ------- | ------------------------------------------------------------------------------------------ |
| `proptest`        | 1.x     | MIT     | Property-based testing for numerical code: mortality calculations, vitality scoring, credit partitions. |
| `insta`           | Latest  | MIT     | Snapshot testing for serialized output (GolemSnapshot, ContextBundle, tool results).       |
| `wiremock`        | Latest  | MIT     | HTTP mock server for inference provider tests (Anthropic, OpenAI, Venice, Grok responses). |
| `mockall`         | Latest  | MIT     | Trait mocking for unit tests that need to isolate crate boundaries.                        |

### Build Tooling (Cargo Plugins)

| Tool             | Install                    | Purpose                                                                  |
| ---------------- | -------------------------- | ------------------------------------------------------------------------ |
| `cargo-nextest`  | `cargo install cargo-nextest` | Parallel test runner. Each test runs in an isolated process.          |
| `cargo-llvm-cov` | `cargo install cargo-llvm-cov` | Source-based coverage reports. Minimum 60% per crate enforced in CI. |
| `cargo-deny`     | `cargo install cargo-deny` | License compliance (allowlist: MIT, Apache-2.0, BSD-*) and advisory scan (RustSec). |
| `cargo-hakari`   | `cargo install cargo-hakari` | Feature unification across workspace. Generates `workspace-hack` crate. |
| `cargo-chef`     | `cargo install cargo-chef` | Docker layer caching. Separates dependency compilation from source compilation. |
| `cargo-vet`      | `cargo install cargo-vet`  | Supply chain auditing. New deps require `cargo vet certify` entry.       |
| `sccache`        | `cargo install sccache`    | Compiler caching. Shared across local builds and CI (S3-backed bucket).  |
| `bacon`          | `cargo install bacon`      | Background clippy watcher. Runs checks on file save.                     |
| `just`           | `cargo install just`       | Task runner. Replaces Makefile. `justfile` at workspace root.            |
| `lefthook`       | `npm install -g lefthook`  | Pre-commit hooks: `cargo fmt --check`, `cargo clippy`, `cargo deny advisories`. |

---
