# Dev Environment Overview

**Version:** 2.0.0
**Last Updated:** 2026-03-16
**Crate:** `mirage-rs/` (chain backend) + `packages/dev/ui/` (debug UI)

---

> **Reader orientation:** This document is the overview of Bardo's local development environment (section 15). It introduces mirage-rs, a single Rust binary that replaces Anvil for DeFi development by forking a live chain and replaying blocks through an in-process revm instance. The key concept is that your local transactions execute alongside real market activity, so LP positions accrue fees from real swaps and vaults track real NAV changes. This section also covers the debug UI, contract deployment pipeline, and scenario tooling. See `prd2/shared/glossary.md` for full term definitions.

## Why This Exists

Building on top of the Uniswap stack (V2, V3, V4, UniswapX, Universal Router, ERC-8004) means testing against the real thing. Mock stubs miss the protocol interactions that matter: V4 hook callbacks during swaps, concentrated liquidity tick crossings, UniswapX Dutch auction fills, ERC-8004 identity gates.

But Anvil's fork mode has a fundamental limitation: **forked chains are frozen snapshots.** You fork mainnet at block N, execute your trade, and then nothing. The rest of the market doesn't move. You can't see what happens to your LP position when ETH drops 5% an hour later. You can't watch how your vault's NAV tracks as the underlying assets fluctuate.

**mirage-rs** fixes this. It forks a live chain, replays incoming blocks through an in-process revm instance, and lets you trade alongside the real market. Your transactions execute locally; mainnet transactions continue flowing in. You see the combined state: your positions plus the market's actual movements.

This replaces Anvil, the TypeScript deployment tooling, and all prior mirage implementations. One Rust binary.

---

## Goals

| ID  | Goal |
| --- | ---- |
| G1  | Fork any EVM chain (Ethereum, Base, Arbitrum) and continue receiving live blocks |
| G2  | Trade on the fork and observe how the real market moves around your positions |
| G3  | Fresh testnet mode: deploy the full Uniswap stack (V2, V3, V4, Permit2, Universal Router, UniswapX, ERC-8004) from scratch |
| G4  | Single binary. No Anvil, no Docker, no external processes |
| G5  | Cheatcode-style state manipulation (setBalance, setCode, mintERC20, impersonate, snapshot/revert) |
| G6  | Transaction filtering: replay only transactions touching specific contracts or selectors |
| G7  | Divergence detection: flag when local replay diverges from mainnet outcomes |
| G8  | Debug UI on a web port for interactive chain exploration and protocol controls |
| G9  | Contract deployment pipeline in Rust (7-phase Uniswap deployment, mock tokens, liquidity seeding) |
| G10 | JSON-RPC compatibility so existing tools (viem, wagmi, debug UI) connect without changes |

## Non-Goals

| ID  | Non-Goal |
| --- | -------- |
| NG1 | Production deployment. Dev-only infrastructure |
| NG2 | Full Reth node. mirage-rs is a dev tool, not a node implementation |
| NG3 | Historical state replay. Fork starts at a specific block, not all history |
| NG4 | Browser-based EVM. The browser SPA is deprioritized in favor of the Rust backend |

---

## Two Modes

### Live Fork Mode

Fork any EVM chain at a specific block (or latest). The block follower subscribes to new blocks via WebSocket and replays them through the local revm instance. Your local transactions are interleaved with mainnet transactions.

Best for:
- Seeing how markets move after your trades
- Testing LP strategies against real price action
- Observing vault NAV changes under real market conditions
- Evaluating agent strategies against live data

```bash
mirage-rs --rpc-url wss://eth-mainnet.g.alchemy.com/v2/KEY --follow
```

### Fresh Testnet Mode

Start with a blank chain. Deploy all contracts from scratch. No external RPC needed.

Best for:
- Offline development
- CI environments
- Testing exact deployment sequences
- Deterministic, reproducible environments

```bash
mirage-rs --fresh --chain-id 31337
```

---

## Architecture

```
+-----------------------------------------------------+
|                    mirage-rs binary                   |
|                                                       |
|  +-----------+  +-------------+  +-----------+       |
|  | JSON-RPC  |  |   Block     |  | Divergence|       |
|  |  Server   |  |  Follower   |  | Detector  |       |
|  |(jsonrpsee)|  | (WS/HTTP)   |  |           |       |
|  +-----+-----+  +------+------+  +-----+-----+      |
|        |               |               |              |
|  +-----v---------------v---------------v--------+    |
|  |              EVM Executor (revm)              |    |
|  |  +------------------------------------------+|    |
|  |  | ForkState                                 ||    |
|  |  |  +--------------------------------------+ ||    |
|  |  |  | CacheDB (in-memory, mutable)         | ||    |
|  |  |  |  +--------------------------------+  | ||    |
|  |  |  |  | RemoteDB (lazy RPC fetch)      |  | ||    |
|  |  |  |  +--------------------------------+  | ||    |
|  |  |  +--------------------------------------+ ||    |
|  |  +------------------------------------------+|    |
|  +-----------------------------------------------+   |
+-----------------------------------------------------+
          ^                           |
          | JSON-RPC                  | RPC fetch on cache miss
          |                           v
   Debug UI / viem             Upstream RPC (mainnet)
```

Three layers:

1. **ForkState** -- In-memory EVM state. `CacheDB<RemoteDB>` provides two-layer caching: mutations stay in CacheDB, cache misses fetch from upstream RPC at the fork block.
2. **EVM Executor** -- revm configured for Cancun spec. Executes transactions, commits state changes, returns receipts.
3. **Surfaces** -- JSON-RPC server for external tools, block follower for live data, divergence detector for validation.

---

## Package Layout

```
mirage-rs/                    # Rust binary -- the chain backend
+-- src/
|   +-- main.rs               # Entry point, CLI, lifecycle
|   +-- cli.rs                # clap argument parsing
|   +-- config.rs             # MirageConfig, SyncMode
|   +-- error.rs              # Error types
|   +-- prefetch.rs           # Block state prefetching
|   +-- state/                # ForkState, RemoteDB, overlay, snapshots, manipulation
|   +-- executor/             # EvmExecutor, block/tx env construction
|   +-- follower/             # Block following, tx filtering
|   +-- rpc/                  # JSON-RPC server (eth_*, mirage_*, net_*, web3_*)
|   +-- divergence/           # Replay vs mainnet comparison
|   +-- simulators/           # Gas, lending (stub), oracle (stub)
+-- Cargo.toml
+-- rust-toolchain.toml

packages/dev/
+-- ui/                       # React + Vite debug UI (connects via JSON-RPC)
+-- browser/                  # Browser SPA (deprioritized)
```

---

## Tech Stack

| Tool | Version | Purpose |
| ---- | ------- | ------- |
| revm | v36 | In-process EVM execution engine |
| alloy | v1.7 | Ethereum types, RPC client, providers |
| jsonrpsee | v0.26 | JSON-RPC server |
| tokio | latest | Async runtime |
| clap | v4.5 | CLI argument parsing |
| tracing | v0.1 | Structured logging |
| parking_lot | v0.12 | Fast RwLock for shared state |
| React | ^19.0 | Debug UI framework |
| Vite | ^6.0 | Debug UI dev server |
| viem | ^2.0 | Debug UI Ethereum client |

---

## Ports

| Service | Port | Notes |
| ------- | ---- | ----- |
| mirage-rs | 8546 | JSON-RPC endpoint (configurable) |
| Debug UI | 3001 | Web interface |

---

## Cross-References

| Document | Relationship |
| -------- | ------------ |
| `prd2/15-dev/01-mirage-rs.md` | Core architecture: HybridDB, DirtyStore, targeted follower, CoW state layers, Block-STM parallel execution |
| `prd2/15-dev/01b-mirage-rpc.md` | Full JSON-RPC method reference: eth_*, mirage_*, evm_*, hardhat/anvil compatibility |
| `prd2/15-dev/01c-mirage-scenarios.md` | Scenario runner, historical replay mode, targeted follower classification rules |
| `prd2/15-dev/01d-mirage-integration.md` | Bardo TUI integration, F6 fork workflow, golem sidecar lifecycle, CorticalState signals |
| `prd2/15-dev/01e-mirage-tx-compatibility.md` | Transaction formats, signature verification, gas edge cases, DeFi-specific pitfalls |
| `prd2/15-dev/02-deployment.md` | 7-phase Uniswap contract deployment pipeline (V2/V3/V4/UniswapX/ERC-8004) |
| `prd2/15-dev/03-debug-ui.md` | React + Vite debug web UI for chain exploration and protocol controls |
| `prd2/15-dev/04-scenarios.md` | Scenario library: live fork scenarios, synthetic market conditions, composition |
| `prd2/15-dev/05-tooling.md` | Developer CLI, Rust build tooling (cargo, clippy, just), environment setup |
| `prd2/15-dev/06-indexer.md` | Local chain indexer via Ponder + PGlite, production protocol state engine spec |
