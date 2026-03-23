# Debug UI

**Version:** 2.0.0
**Last Updated:** 2026-03-16
**Package:** `packages/dev/ui/`

---

> **Reader orientation:** This document specifies the debug UI, a developer-facing web interface for Bardo's local development environment (section 15). It is a React + Vite SPA that connects to mirage-rs (the in-process EVM fork) via standard JSON-RPC. Nine views cover protocol exploration, interactive swaps, liquidity management, UniswapX order submission, ERC-8004 (on-chain agent identity standard) registry browsing, chain state manipulation, and real-time transaction logging. This is a dev-only tool with no authentication or production build. See `prd2/shared/glossary.md` for full term definitions.

## Purpose

Developer-facing web interface for exploring the deployed protocol, making test transactions, and controlling the local chain. Not a production frontend -- no authentication, no mainnet support. Connects to mirage-rs via standard JSON-RPC (viem + wagmi).

The debug UI is functional and stays as-is. It is not a development focus.

---

## Tech Stack

| Library | Purpose |
| ------- | ------- |
| React ^19.0 | UI framework |
| Vite ^6.0 | Dev server |
| TailwindCSS ^4.0 | Styling |
| wagmi ^2.0 | Ethereum wallet hooks |
| viem ^2.0 | RPC calls to mirage-rs |
| @tanstack/react-query ^5.0 | Data fetching with auto-refresh |
| recharts ^2.0 | Charts and liquidity visualization |

---

## Views

Nine views in the sidebar:

1. **Protocol Overview** -- Every contract address, deployment status, health checks. Addresses link to Otterscan if running.
2. **Pool Explorer** -- Browse V2/V3/V4 pools. V3 detail panel: tick, sqrtPriceX96, tick range visualizer, active positions, price chart. V4 pools display hook address and permission flags.
3. **Swap UI** -- Interactive swap form. Quote auto-fetch (debounced 300ms) via V4Quoter/QuoterV2. Multi-protocol routing. Account dropdown with balances. Slippage configurable.
4. **Liquidity UI** -- Add/remove LP positions across V3/V4. Position list with in-range/out-of-range indicators. Fee collection.
5. **UniswapX Orders** -- Dutch auction submission and monitoring. Fill simulation. Order history.
6. **Agent Identity** -- ERC-8004 registry browser. Registration form. Reputation feedback.
7. **Chain Controls** -- Time advancement, block mining, snapshots, balance manipulation, token minting, account impersonation.
8. **Key Management** -- Test account display with balances. Show/hide private keys. Fund/drain buttons.
9. **Transaction Log** -- Real-time ABI-decoded transaction feed via `watchBlocks`. Protocol filter (V2/V3/V4/UniswapX/ERC-8004).

---

## Chain Controls Mapping

The Chain Controls view maps to mirage-rs RPC methods:

| UI Control | RPC Method |
| ---------- | ---------- |
| Fast-Forward | `evm_increaseTime` |
| Mine N blocks | `evm_mine` |
| Set timestamp | `evm_setNextBlockTimestamp` |
| Snapshot | `evm_snapshot` |
| Revert | `evm_revert` |
| Set Balance | `mirage_setBalance` |
| Impersonate | `mirage_impersonateAccount` |
| Mint tokens | `mirage_mintERC20` |

---

## Connection

The debug UI connects to mirage-rs at whatever port it's running on (default 8546). Configured via environment variable or hardcoded in the Vite config. No special protocol -- standard JSON-RPC over HTTP with CORS enabled on the mirage-rs side.

---

## Non-Goals

- No authentication -- localhost only
- No production build for deployment
- No mainnet support
- No mobile responsive design
