# Contract Deployment Pipeline

**Version:** 2.0.0
**Last Updated:** 2026-03-16

---

> **Reader orientation:** This document specifies the 7-phase contract deployment pipeline for Bardo's local development environment (section 15). In fresh testnet mode, the pipeline deploys the full Uniswap stack (V2, V3, V4, Permit2, Universal Router, UniswapX reactors, ERC-8004 agent registries) from scratch onto a mirage-rs chain. In live fork mode, most contracts already exist at canonical addresses; the deployer only adds test-specific contracts. The pipeline also seeds realistic liquidity across V2/V3/V4 pools with mainnet-matching token decimals. See `prd2/shared/glossary.md` for full term definitions.

## Overview

mirage-rs provides the chain. Contract deployment -- the 7-phase Uniswap stack, mock tokens, and liquidity seeding -- runs on top of it.

In live fork mode, most contracts already exist at canonical addresses. The deployer only adds test-specific contracts (mock tokens, test agents). In fresh testnet mode, everything deploys from scratch.

The deployment pipeline is being rewritten in Rust. Currently, deployment can run via external tooling (Forge scripts, the existing TypeScript deployer) against mirage-rs's JSON-RPC endpoint. The target is a built-in Rust deployer that uses revm directly.

---

## 7-Phase Deployment (Fresh Testnet)

Same 7 phases as before, targeting mirage-rs instead of Anvil.

### Phase 1: Foundation (WETH9 + Permit2)

WETH9 from canonical bytecode. Permit2 from pre-compiled bytecode (requires `viaIR` compilation). In fork mode, Permit2 exists at `0x000000000022D473030F116dDEE9F6B43aC78BA3` -- skip.

### Phase 2: Uniswap V2

V2 Factory (`_feeToSetter = deployer`) and Router02 (`factory + WETH9`). Pre-compiled bytecodes from `@uniswap/v2-core` to preserve `INIT_CODE_PAIR_HASH`.

### Phase 3: Uniswap V3

12+ contracts across Solidity 0.7.6: Factory, Multicall2, ProxyAdmin, TickLens, NFTDescriptor, NonfungibleTokenPositionDescriptor, TransparentUpgradeableProxy, NonfungiblePositionManager, V3Migrator, UniswapV3Staker, QuoterV2, SwapRouter02.

NFTDescriptor requires library linking -- `NonfungibleTokenPositionDescriptor` bytecode contains Hardhat-style `__$...$__` placeholders replaced with the deployed library address.

### Phase 4: Uniswap V4

Singleton PoolManager (requires Cancun -- revm is configured for this), PositionDescriptor, PositionManager, V4Quoter, StateView. Test helpers (PoolSwapTest, PoolModifyLiquidityTest) deployed in fresh mode for seed scripts.

### Phase 5: Universal Router

Ties V2, V3, and V4 together via `RouterParameters` struct containing all prior phase addresses. Unused addresses set to `address(0)` -- the router reverts only when those commands are called.

### Phase 6: UniswapX Reactors

ExclusiveDutchReactor, V2DutchReactor, PriorityReactor, LimitReactor. Each extends `BaseReactor(IPermit2, address protocolFeeOwner)`.

### Phase 7: ERC-8004 Agent Registries

ERC-8004 (on-chain agent identity standard) Identity + Reputation registries. In fork mode, use canonical Base addresses.

---

## Mock Tokens

Five ERC-20 tokens with mainnet-matching decimals:

| Token | Symbol | Decimals | Initial Supply |
| ----- | ------ | -------- | -------------- |
| Wrapped Ether | WETH | 18 | via WETH9 deposit() |
| USD Coin | USDC | 6 | 100,000,000 |
| Tether USD | USDT | 6 | 100,000,000 |
| Dai Stablecoin | DAI | 18 | 100,000,000 |
| Wrapped Bitcoin | WBTC | 8 | 1,000 |

In fork mode, use `mirage_mintERC20` for funding. mirage-rs knows the balance storage slots for these 5 tokens on Ethereum mainnet and Base.

---

## Token Funding (Fork Mode)

mirage-rs provides native ERC-20 minting via `mirage_mintERC20`:

```bash
# Fund account with 100,000 USDC
cast rpc mirage_mintERC20 \
  "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" \
  "0xYourAddress" \
  "0x5F5E100" \
  null  # Use known balance slot

# Set ETH balance
cast rpc mirage_setBalance "0xYourAddress" "0x56BC75E2D63100000"
```

For tokens not in the known list, pass the storage slot explicitly or use `mirage_setStorageAt`.

---

## Liquidity Seeding

After deployment, realistic liquidity across V2, V3, and V4.

### V2 Pools

| Pair | Initial Price |
| ---- | ------------- |
| WETH/USDC | 1 WETH = 2,500 USDC |
| WETH/USDT | 1 WETH = 2,500 USDT |
| USDC/DAI | 1:1 |
| WBTC/WETH | 1 WBTC = 14 WETH |

### V3 Pools

| Pool | Fee Tier | Tick Spacing | Initial Price |
| ---- | -------- | ------------ | ------------- |
| USDC/DAI | 0.01% (100) | 1 | 1:1 |
| WETH/USDC | 0.05% (500) | 10 | 2,500 |
| WETH/USDC | 0.30% (3000) | 60 | 2,500 |
| WETH/USDT | 0.30% (3000) | 60 | 2,500 |
| WBTC/WETH | 1.00% (10000) | 200 | 14 ETH |

Position distribution per pool:
- 1 tight-range: tickCurrent +/- 2 * tickSpacing
- 2 medium-range: tickCurrent +/- 10 * tickSpacing
- 1 full-range: [MIN_TICK, MAX_TICK]

### V4 Pools

No-hook pools first, then one hook-enabled pool. ETH as native currency (ADDRESS_ZERO for currency0).

---

## Deployment State

Deployment produces a manifest at `.mirage/deployment.json`:

```json
{
  "mode": "fork",
  "chainId": 1,
  "forkBlock": 21000000,
  "addresses": {
    "weth9": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    "permit2": "0x000000000022D473030F116dDEE9F6B43aC78BA3",
    "v3Factory": "0x1F98431c8aD98523631AE4a59f267346ea31F984",
    "v3PositionManager": "0xC36442b4a4522E871399CD717aBDD847Ab11FE88",
    "v4PoolManager": "0x...",
    "universalRouter": "0x...",
    "erc8004IdentityRegistry": "0x...",
    "usdc": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "usdt": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
    "dai": "0x6B175474E89094C44Da98b954EedeAC495271d0F",
    "wbtc": "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"
  }
}
```

In fork mode, canonical addresses are detected automatically. In fresh mode, addresses are deterministic from the deployer's nonce sequence. The manifest is consumed by the debug UI, scenarios, and the indexer.

---

## Rust Deployer (Target)

The target is a built-in Rust deployment module within mirage-rs:

- **Speed**: Direct revm execution, no JSON-RPC overhead. Full 7-phase deployment in <5 seconds.
- **Single binary**: No Forge or Node.js runtime needed.
- **State manipulation**: Direct CacheDB writes for seeding instead of EVM transactions.
- **Integrated CLI**: `mirage-rs --fresh --deploy --seed` does everything.

The Rust deployer would load pre-compiled contract artifacts (ABI + bytecode as embedded resources), execute deployment transactions through revm, and produce the deployment manifest.

### Not Yet Built

The deployment pipeline is the largest gap. mirage-rs provides a functional chain backend; what's missing is the Rust code that deploys Uniswap contracts and seeds liquidity. In the interim, the existing TypeScript deployer or Forge scripts can target mirage-rs's JSON-RPC endpoint -- it speaks the same protocol as Anvil.

---

## Mode Comparison

| Aspect | Fresh Testnet | Live Fork |
| ------ | ------------- | --------- |
| Contracts | Full 7-phase deployment | Pre-existing at canonical addresses |
| Tokens | Minted via mock ERC-20s | `mirage_mintERC20` or `mirage_setBalance` |
| Chain ID | 31337 (configurable) | Inherited from source chain |
| Block time | Instant (mine on demand) | Inherited from source + live following |
| External RPC | Not required | Required (HTTP or WSS) |
| Deployment time | <5s (Rust) / ~45s (external) | <1s (test contracts only) |

---

## Compiler Versions

Pre-compiled artifacts span multiple Solidity versions:

| Phase | Contracts | Solidity |
| ----- | --------- | -------- |
| V2 | Factory, Router02 | 0.5.16 / 0.6.6 |
| V3 | Factory + periphery | 0.7.6 |
| V4 | PoolManager + periphery | 0.8.26 |
| UniswapX | All reactors | 0.8.x |
| ERC-8004 | Identity + Reputation | 0.8.x |

Bytecodes stored as embedded resources in the Rust binary (or loaded from JSON artifacts at runtime).
