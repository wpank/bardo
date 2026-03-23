# Supported Chains Reference [SPEC]

> **Document Type**: REF (normative) | **Referenced by**: All PRDs | **Last Updated**: 2026-03-08

> **Reader orientation:** This document is the canonical chain support reference for the Bardo ecosystem. It specifies which EVM chains are supported, what features are available on each, and how Uniswap V4 deployment status maps to Bardo vault readiness. The key concept is the two-layer support model: Layer A tracks Uniswap's own V4 deployments, Layer B tracks Bardo's readiness on each chain. Golems (mortal autonomous agents compiled as single Rust binaries running on micro VMs) and vaults operate within these chain constraints. See `prd2/shared/glossary.md` for full term definitions.

---

## Chain Registry

| Chain         | Chain ID | Native Currency | Uniswap Versions | UniswapX | ERC-7683 | Explorer                |
| ------------- | -------- | --------------- | ---------------- | -------- | -------- | ----------------------- |
| Ethereum      | 1        | ETH             | V2, V3, V4       | Yes      | Yes      | etherscan.io            |
| Optimism      | 10       | ETH             | V3               | Yes      | Yes      | optimistic.etherscan.io |
| Polygon       | 137      | POL             | V3               | Yes      | Yes      | polygonscan.com         |
| Arbitrum      | 42161    | ETH             | V3               | Yes      | Yes      | arbiscan.io             |
| Base          | 8453     | ETH             | V2, V3, V4       | Yes      | Yes      | basescan.org            |
| BNB Chain     | 56       | BNB             | V3               | No       | No       | bscscan.com             |
| Avalanche     | 43114    | AVAX            | V3               | No       | No       | snowtrace.io            |
| Celo          | 42220    | CELO            | V3               | No       | No       | celoscan.io             |
| Blast         | 81457    | ETH             | V3               | No       | No       | blastscan.io            |
| Unichain      | 130      | ETH             | V3, V4           | Yes      | Yes      | uniscan.xyz             |
| zkSync Era    | 324      | ETH             | V3               | No       | No       | explorer.zksync.io      |
| Anvil (local) | 31337    | ETH             | V2, V3, V4       | Mock     | Mock     | Otterscan (port 5100)   |

---

## Two-Layer Chain Support Model (Normative)

Chain support requires distinguishing between Uniswap's own deployments and Bardo's readiness on each chain:

- **Layer A -- Uniswap V4 Deployed**: Uniswap has deployed V4 contracts on this chain. Source of truth: [Uniswap V4 Deployments](https://docs.uniswap.org/contracts/v4/deployments).
- **Layer B -- Bardo Vaults Supported**: Bardo's contracts, SDK, monitoring, and audits are ready for this chain.

### Client Display Rules (Normative)

| Layer A (V4 deployed) | Layer B (Bardo supported) | Client Behavior                                                                 |
| --------------------- | ------------------------- | ------------------------------------------------------------------------------- |
| No                    | N/A                       | V4-based features MUST be disabled. Show V3-only tool surface.                  |
| Yes                   | No                        | V4-based features MUST be marked "Not supported by Bardo yet." V3 paths remain. |
| Yes                   | Yes                       | Full feature set enabled.                                                       |

### Current Layer A/B Status

| Chain      | Layer A (V4 Deployed) | Layer B (Bardo Supported) |
| ---------- | --------------------- | ------------------------- |
| Ethereum   | Yes                   | Yes (v1 target)           |
| Unichain   | Yes                   | Yes (v1 target)           |
| Base       | Yes                   | Yes (v1 target)           |
| All others | No                    | No (core tools only)      |

### Address Mapping Sourcing (Normative)

Address mappings for Uniswap contracts MUST be sourced from the [Uniswap V4 Deployments](https://docs.uniswap.org/contracts/v4/deployments) page and pinned by commit hash in the repository configuration file (`packages/tools/src/constants/`). Integrators MUST NOT assume identical deployments across chains.

On every deployment or release:

1. The build pipeline MUST verify that pinned addresses match the latest Uniswap Deployments page
2. Any address mismatch MUST fail the build and require explicit acknowledgment
3. New chain support MUST include address verification as part of the deployment checklist

---

## Chain Capabilities (Normative)

Every feature in this specification MUST declare which chains support it. Clients MUST read `ChainCapabilities` and dynamically enable/disable UI and tool paths. Features unavailable on the selected chain MUST be disabled with an explanatory message, never silently omitted.

### Feature Matrix

| Feature                        | Capability Key    | Required Contracts/Data                | Chains Available                                      | Fallback Behavior                           |
| ------------------------------ | ----------------- | -------------------------------------- | ----------------------------------------------------- | ------------------------------------------- |
| V4 share pools (hook market)   | `v4SharePools`    | V4 PoolManager + VaultHook deployment  | Ethereum, Base, Unichain                              | Disable share market; deposit/withdraw only |
| Gas sponsorship                | `paymaster`       | Paymaster endpoint + AA bundle support | Base (ERC-7677 paymaster)                             | User-paid gas path                          |
| V3 Subgraph analytics          | `subgraphV3`      | Deployed subgraph endpoint             | All 11 chains                                         | Degrade to RPC-only reads or hide analytics |
| V4 Subgraph analytics          | `subgraphV4`      | Deployed subgraph endpoint             | Ethereum, Base, Unichain (others TBD)                 | Degrade to RPC-only or hide V4 analytics    |
| Async redemption (ERC-7540)    | `erc7540`         | Vault supports request/claim lifecycle | v1 Critical                                           | Use synchronous withdraw only               |
| UniswapX order routing         | `uniswapX`        | UniswapX deployment + filler network   | Ethereum, Optimism, Polygon, Arbitrum, Base, Unichain | Use on-chain swap via Universal Router      |
| Cross-chain intents (ERC-7683) | `erc7683`         | ERC-7683 filler network                | Ethereum, Optimism, Polygon, Arbitrum, Base, Unichain | Single-chain operation only                 |
| ERC-8004 identity registration | `erc8004Registry` | Identity Registry deployment           | Ethereum (L1), readable on all L2s                    | Cannot onboard without L1 registration      |

### Per-Chain Summary

| Chain      | v4SharePools | paymaster | subgraphV3 | subgraphV4 | uniswapX | erc7683 | erc8004Registry |
| ---------- | ------------ | --------- | ---------- | ---------- | -------- | ------- | --------------- |
| Ethereum   | Yes          | No        | Yes        | Yes        | Yes      | Yes     | Yes (primary)   |
| Optimism   | No           | No        | Yes        | No         | Yes      | Yes     | Read-only       |
| Polygon    | No           | No        | Yes        | No         | Yes      | Yes     | Read-only       |
| Arbitrum   | No           | No        | Yes        | No         | Yes      | Yes     | Read-only       |
| Base       | Yes          | Yes       | Yes        | Yes        | Yes      | Yes     | Read-only       |
| BNB Chain  | No           | No        | Yes        | No         | No       | No      | Read-only       |
| Avalanche  | No           | No        | Yes        | No         | No       | No      | Read-only       |
| Celo       | No           | No        | Yes        | No         | No       | No      | Read-only       |
| Blast      | No           | No        | Yes        | No         | No       | No      | Read-only       |
| Unichain   | Yes          | No        | Yes        | TBD        | Yes      | Yes     | Read-only       |
| zkSync Era | No           | No        | Yes        | No         | No       | No      | Read-only       |

> **Note**: "TBD" entries indicate infrastructure expected but not yet confirmed. Clients MUST treat TBD as `false` and degrade gracefully.

---

## V4 PoolManager Deployments

V4 uses a deterministic CREATE2 deployment. The PoolManager address is identical across all supported chains.

| Chain | Chain ID | V4 PoolManager | Status |
|-------|----------|----------------|--------|
| Ethereum | 1 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |
| Arbitrum | 42161 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |
| Base | 8453 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |
| Optimism | 10 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |
| Polygon | 137 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |
| BNB Chain | 56 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |
| Avalanche | 43114 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |
| Unichain | 130 | `0x000000000004444c5dc75cB358380D2e3dE08A90` | Live |

---

## Key Contracts (Mainnet)

| Contract                   | Address                                      |
| -------------------------- | -------------------------------------------- |
| ERC-8004 Identity Registry | `0x8004A818BFB912233c491871b3d84c89A494BD9e` |
| TokenJar                   | `0xf38521f130fcCF29dB1961597bc5d2B60F995f85` |
| Firepit                    | `0x0D5Cd355e2aBEB8fb1552F56c965B867346d6721` |
| UNI Token                  | `0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984` |
| V4 PoolManager             | `0x000000000004444c5dc75cB358380D2e3dE08A90` |
| Permit2                    | `0x000000000022D473030F116dDEE9F6B43aC78BA3` |

---

## Base L2 Calibration (Normative)

Base-specific parameters that differ from Ethereum mainnet due to L2 characteristics:

| Parameter              | Ethereum Mainnet | Base L2               | Rationale                                                  |
| ---------------------- | ---------------- | --------------------- | ---------------------------------------------------------- |
| LVR magnitude          | Baseline         | ~5x lower             | Faster block times reduce arbitrage window [BASE-LVR-2024] |
| LP strategy viability  | Moderate         | Higher                | Lower LVR makes active LP more profitable on L2            |
| Dynamic fee sigma_low  | 10%              | 20%                   | Wider thresholds account for L2 fee dynamics               |
| Dynamic fee sigma_high | 40%              | 80%                   | Wider thresholds account for L2 fee dynamics               |
| am-AMM lookahead       | 7,200 blocks     | 36,000 blocks         | Equivalent time window adjusted for 2s Base block time     |
| Gas costs              | Baseline         | 50-500x cheaper       | Enables frequent rebalancing strategies                    |
| Rebalance frequency    | Limited by gas   | High-frequency viable | Cost reduction enables more granular position management   |

---

## Scope Clarification

Bardo tools support data queries across all 11 Uniswap-deployed chains. Vault operations (deploy, manage, trade) are limited to V4-enabled chains (Ethereum, Base, Unichain) for v1. Bardo Bott v1 deployment targets Base only for execution, with Ethereum L1 for ERC-8004 identity registration.
