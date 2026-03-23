# Scenarios

**Version:** 2.0.0
**Last Updated:** 2026-03-16

---

> **Reader orientation:** This document is the scenario library for Bardo's local development environment (section 15). It covers two categories of scenarios run on mirage-rs (the in-process EVM fork): live fork scenarios where your action executes and then real mainnet blocks continue flowing through, and synthetic scenarios that use programmatic swaps and state manipulation to create specific market conditions. The key concept is that live fork scenarios let you observe real market reactions to your trades, unlike static Anvil forks where nothing happens after your transaction. Scenarios can be composed via snapshot checkpoints and branching. See `prd2/shared/glossary.md` for full term definitions.

## Overview

mirage-rs's live fork mode makes scenarios fundamentally different from static Anvil forks. Instead of simulating market conditions synthetically, you can observe real market conditions unfolding after your intervention.

Two categories:

1. **Live fork scenarios** -- Execute your action, then watch real mainnet blocks flow through. The market moves around you.
2. **Synthetic scenarios** -- Programmatic swaps and state manipulation to create specific conditions. Work in both fork and fresh testnet modes.

---

## Live Fork Scenarios

These require fork mode with `--follow` enabled.

### Trade and Watch

The simplest and most powerful scenario. Execute a swap on the fork, then observe how the pool evolves as real mainnet transactions continue flowing in.

```bash
# Start mirage-rs forking Ethereum mainnet with live following
mirage-rs --rpc-url wss://eth-mainnet.g.alchemy.com/v2/KEY --follow

# Execute your swap via cast or the debug UI
cast send $UNIVERSAL_ROUTER "execute(bytes,bytes[],uint256)" \
  ... --rpc-url http://localhost:8546

# Mainnet blocks continue replaying.
# Your swap changed the local pool state.
# Subsequent mainnet swaps execute against your modified state.
# Divergence detector flags which mainnet txs were affected.
```

### LP Position Performance

Add a concentrated liquidity position, then observe whether it stays in range as real price action unfolds.

1. Fork at current block.
2. Add a V3 position with tight range (+/-2% around current price).
3. Let blocks replay for N minutes or hours.
4. Query position: still in range? Fees accrued? Would a wider range have been better?

Compare strategies by reverting to a pre-position snapshot and trying a different range.

### Vault NAV Tracking

Deploy a vault on the fork, deposit assets into Uniswap pools via the vault's strategy. Watch how NAV changes as the market moves. See whether the strategy's rebalancing logic responds correctly to real price action.

### Agent Strategy Evaluation

Register an ERC-8004 (on-chain agent identity standard) agent on the fork, run its strategy loop, and observe outcomes against real market data. Compare against a control (no agent intervention) by reverting to a pre-intervention snapshot.

### Filtered Replay

Transaction filtering controls what mainnet activity is replayed. If you're testing an LP position on WETH/USDC, you don't need to replay every transaction on mainnet -- just the ones that touch your pool:

```bash
mirage-rs --rpc-url $WS_URL --follow \
  --filter-address 0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640 \  # WETH/USDC 0.05% pool
  --filter-selector 0x128acb08  # swap(address,bool,int256,uint160,bytes)
```

Reduces noise and speeds up replay.

---

## Synthetic Scenarios

Work in both modes. Driven by state manipulation + programmatic transactions.

### Crash (ETH Price Drop)

Large directional swaps across V2/V3/V4 to simulate a price decline.

| Parameter | Default | Range | Description |
| --------- | ------- | ----- | ----------- |
| `dropPercent` | 40 | 10-90 | Target price drop |
| `swapCount` | 20 | 5-100 | Number of sell swaps |

Use cases:
- Test vault drawdown tolerance
- Observe LP position health under stress
- Validate agent mortality triggers when portfolio value crashes

### Volume Spike

Burst of two-directional swaps across all pools.

| Parameter | Default | Range | Description |
| --------- | ------- | ----- | ----------- |
| `swapCount` | 100 | 10-500 | Number of swaps |
| `protocol` | `all` | v2/v3/v4/all | Target protocols |

Behavior: 2/3 buys, 1/3 sells (bullish spike). Varying sizes (0.1-5 ETH per swap). Spreads across multiple accounts.

Use cases:
- Test fee accumulation
- Observe MEV patterns in V4 hooks
- Stress-test under high-activity conditions

### New Pool

Deploy a new token, create pools across V2/V3/V4, add initial liquidity.

| Parameter | Default | Range | Description |
| --------- | ------- | ----- | ----------- |
| `symbol` | `"NEW"` | any | Token ticker |
| `decimals` | 18 | 6/8/18 | Token decimals |
| `initialPrice` | 1.0 | 0.001-10000 | Price in USDC |
| `tvl` | 500,000 | 10K-10M | Target TVL |

### Agent Onboard

Full ERC-8004 registration flow: fund wallet, register identity, set reputation.

| Parameter | Default | Range | Description |
| --------- | ------- | ----- | ----------- |
| `handle` | (required) | any | Agent handle |
| `fundEth` | 10 | 0-10000 | ETH funding |
| `fundUsdc` | 10,000 | 0-1M | USDC funding |
| `initialReputation` | 0 | 0-1000 | Starting reputation |

---

## Scenario Composition

Chain scenarios with snapshot checkpoints between them. `evm_snapshot` + `evm_revert` let you branch from any checkpoint:

1. Run volume-spike scenario, take snapshot ("after-volume").
2. Run crash scenario, take snapshot ("after-crash").
3. Revert to "after-volume" to test a different path from that state.

Snapshots are ephemeral (in-memory, tied to the mirage-rs process lifetime). For persistence across restarts, the state would need to be serialized -- not yet implemented.

---

## Live Fork + Synthetic Combinations

The most interesting scenarios combine both modes:

1. Fork mainnet at current block with `--follow`.
2. Run a synthetic scenario (e.g., deploy a new token and seed pools).
3. Let live blocks flow through. Watch how the market interacts with your new deployment.
4. Use divergence detection to see which mainnet transactions were affected.

This lets you test "what if I deployed this contract on mainnet right now?" with real subsequent market activity.
