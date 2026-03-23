# Bardo Tools -- Yield [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Yield aggregators (Yearn V3, Convex), yield tokenization (Pendle PT/YT/LP), and synthetic dollar yield (Ethena sUSDe). Category: `yield`. 20 tools total. Managed LP tools (Arrakis, Gamma) live in `04-tools-lp.md`.

---

> **Reader orientation:** This document specifies 20 yield tools in `bardo-tools`: Yearn V3 vaults, Convex staking, Pendle yield tokenization (PT/YT/LP), and Ethena sUSDe synthetic dollar yield. You should be familiar with ERC-4626 vault mechanics, Convex/Curve gauge staking, and Pendle's fixed/variable yield splitting. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Yield aggregators -- Yearn V3 (4 tools)

Yearn V3 uses ERC-4626 vault architecture with pluggable strategy adapters. Strategies rotate across lending, Curve LP, and other yield sources. Deployed on Ethereum (1), Arbitrum (42161), Base (8453), and Polygon (137).

All Yearn tools use Alloy's (the standard Rust library for EVM interaction) `sol!` macro against standard ERC-4626 interfaces (`deposit`, `redeem`, `convertToAssets`, `totalAssets`).

### `yearn_deposit`

Deposit assets into a Yearn V3 vault.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Resolving vault", "Checking approval", "Simulating deposit in Revm (Rust EVM implementation for local transaction simulation)", "Broadcasting deposit"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct YearnDepositParams {
    /// Vault address or slug (e.g., "yvUSDC").
    pub vault: String,
    /// Amount to deposit in underlying token units.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct YearnDepositResult {
    pub tx_hash: String,
    pub shares_received: String,
    pub assets_deposited: String,
    pub share_price: f64,
    /// Current vault APY.
    pub preview_apy: f64,
    /// e.g., "Curve USDC-crvUSD Convex"
    pub strategy_name: String,
}
```

**Ground truth**: ERC-4626 `balanceOf(account)` increased by `shares_received`. Expected: `"shares received: {shares_received}"`. Actual: on-chain share balance. Source: `"erc4626_balance_of"`.

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components):
- `tool:start` -> `{ tool_name: "yearn_deposit", params_hash }`
- `tool:update` -> `"Resolving vault"` -> `"Checking approval"` -> `"Simulating deposit in Revm"` -> `"Broadcasting deposit"`
- `tool:end` -> `{ success, shares_received, preview_apy }`

**promptSnippet**: "Deposits assets into a Yearn V3 vault (ERC-4626). Check `yearn_get_vaults` first to compare APY and risk. Phase-gated: blocked in declining/terminal."

**promptGuidelines**:
- "In thriving phase: deposit idle assets for yield. Prefer vaults with 30d APY consistency and TVL >$1M."
- "In stable phase: deposit only into low-risk vaults (stablecoin strategies)."
- "In conservation phase: no new deposits. Let existing positions compound."
- "In declining/terminal phase: BLOCKED."

### `yearn_withdraw`

Withdraw from a Yearn V3 vault by redeeming shares.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Resolving vault", "Simulating withdrawal", "Broadcasting redeem"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct YearnWithdrawParams {
    /// Vault address or slug.
    pub vault: String,
    /// Shares to redeem, or "max" for full withdrawal.
    pub shares: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct YearnWithdrawResult {
    pub tx_hash: String,
    pub assets_received: String,
    pub shares_burned: String,
    pub realized_yield: String,
}
```

**Ground truth**: Shares burned, assets transferred. Source: `"erc4626_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ assets_received, realized_yield }`

**promptSnippet**: "Withdraws from a Yearn V3 vault by redeeming shares. Pass \"max\" for full withdrawal. Allowed at any phase."

**promptGuidelines**:
- "In thriving phase: withdraw to rebalance or capture yield."
- "In conservation phase: withdraw underperforming positions (APY below borrow cost)."
- "In declining phase: withdraw all positions. Mandatory."
- "In terminal phase: withdraw any remaining."

### `yearn_get_vaults`

List Yearn V3 vaults with APY and TVL data.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Querying Yearn API", "Enriching with on-chain data"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct YearnGetVaultsParams {
    /// Chain ID.
    pub chain_id: u64,
    /// Filter by underlying asset address.
    pub asset: Option<String>,
    /// Minimum APY filter.
    pub min_apy: Option<f64>,
    /// Minimum TVL in USD.
    pub min_tvl: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct YearnVaultsResult {
    pub vaults: Vec<YearnVaultInfo>,
}

#[derive(Debug, Serialize)]
pub struct YearnVaultInfo {
    pub vault: String,
    pub name: String,
    pub asset: String,
    pub apy: f64,
    pub apy_7d: f64,
    pub apy_30d: f64,
    pub tvl: f64,
    /// e.g., "Curve LP", "Lending", "Staking"
    pub strategy_type: String,
    /// e.g., ["Curve", "Smart Contract"]
    pub risks: Vec<String>,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ vault_count }`

**promptSnippet**: "Lists Yearn V3 vaults with APY, TVL, strategy type, and risk factors. Filter by asset, minimum APY, or minimum TVL. Read-only."

### `yearn_get_position`

Get an account's current Yearn V3 positions.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Scanning vaults for account shares"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct YearnGetPositionParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct YearnPositionResult {
    pub positions: Vec<YearnPosition>,
    pub total_value_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct YearnPosition {
    pub vault: String,
    pub vault_name: String,
    pub shares: String,
    pub assets_value: String,
    pub assets_value_usd: f64,
    pub apy: f64,
    pub yield_earned: String,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ position_count, total_value_usd }`

**promptSnippet**: "Returns all Yearn V3 positions for an account: shares, value, APY, earned yield. Read-only."

**Error codes**: `VAULT_NOT_FOUND`, `INSUFFICIENT_SHARES`

---

## Yield aggregators -- Convex (4 tools)

Convex aggregates Curve LP positions to maximize CRV rewards via pooled veCRV boosting. Stakers earn CRV + CVX + trading fees. vlCVX holders direct emissions and earn protocol revenue. Deployed on Ethereum (1).

### `convex_stake_lp`

Stake Curve LP tokens in Convex for boosted CRV + CVX rewards.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Resolving Convex pool", "Checking approval", "Simulating stake in Revm", "Broadcasting deposit"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ConvexStakeLpParams {
    /// Curve pool address.
    pub pool: String,
    /// LP token amount.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ConvexStakeResult {
    pub tx_hash: String,
    pub convex_deposit_token: String,
    pub staked_amount: String,
    pub estimated_crv_apy: f64,
    pub estimated_cvx_apy: f64,
    pub total_apy: f64,
}
```

**Ground truth**: Convex receipt tokens in wallet. Source: `"convex_base_reward_pool"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ total_apy }`

**promptSnippet**: "Stakes Curve LP tokens in Convex for boosted CRV + CVX rewards. Requires existing Curve LP position. Phase-gated: blocked in declining/terminal."

**promptGuidelines**:
- "In thriving phase: stake all Curve LP for maximum yield boost."
- "In conservation phase: keep existing stakes. No new staking."
- "In declining/terminal phase: unstake all."

### `convex_unstake_lp`

Unstake Curve LP from Convex and claim pending rewards.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Simulating unstake", "Broadcasting withdrawAndUnwrap"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ConvexUnstakeLpParams {
    /// Curve pool address.
    pub pool: String,
    /// Amount to unstake.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ConvexUnstakeResult {
    pub tx_hash: String,
    pub lp_returned: String,
    pub rewards_claimed: Vec<RewardClaim>,
}

#[derive(Debug, Serialize)]
pub struct RewardClaim {
    pub token: String,
    pub symbol: String,
    pub amount: String,
    pub usd_value: f64,
}
```

**Ground truth**: LP tokens returned to wallet. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ lp_returned, reward_count }`

**promptSnippet**: "Unstakes Curve LP from Convex and claims pending CRV + CVX rewards. Allowed at any phase."

### `convex_claim_rewards`

Claim earned reward tokens (CRV, CVX, and extras) from a Convex pool without unstaking.

**Capability**: `WriteTool` | **Risk tier**: `Layer1` | **Tick budget**: `Slow`
**Progress steps**: `["Checking claimable rewards", "Broadcasting getReward"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ConvexClaimRewardsParams {
    /// Pool or deposit contract address.
    pub pool: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ConvexClaimResult {
    pub tx_hash: String,
    pub rewards: Vec<RewardClaim>,
}
```

**Ground truth**: Reward token balances increased. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ rewards }`

**promptSnippet**: "Claims earned CRV, CVX, and extra reward tokens from Convex. Does not unstake LP. Allowed at any phase."

**promptGuidelines**:
- "In thriving phase: claim periodically and compound or sell rewards for operating capital."
- "In conservation phase: claim and sell for operating capital."
- "In declining phase: claim all outstanding rewards immediately."

### `convex_get_pools`

List Convex pools with full APY breakdown.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Querying Convex API", "Computing APY breakdown"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ConvexGetPoolsParams {
    /// Chain ID.
    pub chain_id: u64,
    /// Filter by underlying asset.
    pub asset: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConvexPoolsResult {
    pub pools: Vec<ConvexPoolInfo>,
}

#[derive(Debug, Serialize)]
pub struct ConvexPoolInfo {
    pub pool: String,
    pub curve_lp_token: String,
    pub base_apy: f64,
    pub crv_apy: f64,
    pub cvx_apy: f64,
    pub extra_apy: f64,
    pub total_apy: f64,
    pub tvl: f64,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ pool_count }`

**promptSnippet**: "Lists Convex pools with full APY breakdown: base, CRV, CVX, and extra rewards. Read-only."

**Error codes**: `CONVEX_POOL_SHUTDOWN`, `CONVEX_POOL_NOT_FOUND`

---

## Yield tokenization -- Pendle (8 tools)

Pendle splits yield-bearing assets (stETH, sUSDe, etc.) into Principal Tokens (PT) and Yield Tokens (YT). PTs trade at a discount and redeem at par at maturity -- fixed-rate bond semantics. YTs represent the right to all yield during the remaining period -- leveraged yield speculation. Pendle's AMM enables continuous price discovery for both.

Deployed on Ethereum (1), Arbitrum (42161), Base (8453), and BNB Chain (56).

### `pendle_buy_pt`

Buy a Principal Token at a discount for fixed-rate yield. The implied APY locks in the discount-to-par return over the remaining period.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Resolving market", "Computing PT price", "Simulating in Revm fork", "Broadcasting swapExactTokenForPt"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleBuyPtParams {
    /// Pendle market address or symbol (e.g., "PT-sUSDe-Mar2025").
    pub market: String,
    /// Spending amount in underlying token.
    pub amount: String,
    /// Max slippage in basis points (default: 50).
    #[serde(default = "default_slippage_50")]
    pub slippage_bps: u32,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendlePtPurchase {
    pub tx_hash: String,
    pub pt_received: String,
    pub pt_address: String,
    pub underlying: String,
    /// e.g., "2025-03-27"
    pub maturity: String,
    pub days_to_maturity: u32,
    /// Annualized yield at current price.
    pub implied_apy: f64,
    /// Percentage discount vs par.
    pub discount: f64,
    /// PT price in underlying tokens.
    pub spot_price: f64,
}
```

**Ground truth**: PT balance increased by `pt_received`. Expected: `"PT received: {pt_received}"`. Actual: on-chain PT balance. Source: `"erc20_balance_of"`.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "pendle_buy_pt", params_hash }`
- `tool:update` -> `"Resolving market"` -> `"Computing PT price"` -> `"Simulating in Revm fork"` -> `"Broadcasting swapExactTokenForPt"`
- `tool:end` -> `{ success, pt_received, implied_apy, days_to_maturity }`

**promptSnippet**: "Buys a Pendle PT at a discount for fixed-rate yield. PT redeems at par at maturity. Check `pendle_get_markets` first. Phase-gated: blocked in declining/terminal."

**promptGuidelines**:
- "In thriving phase: buy PT for fixed yield on stable assets (sUSDe, stETH). Match maturity to strategy horizon."
- "In stable phase: buy short-maturity PT only (under 30 days). Fixed yield reduces uncertainty."
- "In conservation phase: no new PT purchases."
- "In declining/terminal phase: BLOCKED."
- "ALWAYS check maturity vs Golem projected lifespan. Do not buy PT that matures after you die."

### `pendle_sell_pt`

Sell PT before maturity at current market price.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Resolving market", "Simulating sale", "Broadcasting swapExactPtForToken"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleSellPtParams {
    /// Pendle market address or symbol.
    pub market: String,
    /// PT amount to sell.
    pub amount: String,
    /// Default: 50 bps.
    #[serde(default = "default_slippage_50")]
    pub slippage_bps: u32,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendlePtSold {
    pub tx_hash: String,
    pub underlying_received: String,
    pub pnl_vs_par: f64,
    pub implied_apy: f64,
}
```

**Ground truth**: Underlying received matches simulation. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ underlying_received, pnl_vs_par }`

**promptSnippet**: "Sells PT before maturity at current market price. May realize a loss if sold below purchase price. Allowed at any phase."

### `pendle_buy_yt`

Buy Yield Token for leveraged yield exposure during the remaining period. YT value decays to zero at maturity regardless of price; this is yield speculation, not principal preservation.

**Capability**: `WriteTool` | **Risk tier**: `Layer3` (speculative, decays to zero) | **Tick budget**: `Slow`
**Progress steps**: `["Resolving market", "Computing YT leverage", "Simulating in Revm fork", "Broadcasting swapExactTokenForYt"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleBuyYtParams {
    /// Pendle market address or symbol.
    pub market: String,
    /// Spending amount in underlying token.
    pub amount: String,
    /// Default: 100 bps (YT is more volatile than PT).
    #[serde(default = "default_slippage_100")]
    pub slippage_bps: u32,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendleYtPurchase {
    pub tx_hash: String,
    pub yt_received: String,
    pub yt_address: String,
    pub implied_yield: f64,
    pub leverage: f64,
    pub maturity: String,
}
```

**Ground truth**: YT balance increased. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ yt_received, leverage, maturity }`

**promptSnippet**: "Buys a Pendle YT for leveraged yield exposure. WARNING: YT decays to ZERO at maturity. Phase-gated: blocked in stable/conservation/declining/terminal."

**promptGuidelines**:
- "In thriving phase ONLY: buy YT with strong conviction on rising yields. Max 5% of capital. Sell before maturity decay accelerates."
- "ALL other phases: BLOCKED. YT is too speculative."

### `pendle_sell_yt`

Sell Yield Token before maturity.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Resolving market", "Simulating sale", "Broadcasting swapExactYtForToken"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleSellYtParams {
    /// Pendle market address or symbol.
    pub market: String,
    /// YT amount to sell.
    pub amount: String,
    /// Default: 100 bps.
    #[serde(default = "default_slippage_100")]
    pub slippage_bps: u32,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendleYtSold {
    pub tx_hash: String,
    pub underlying_received: String,
    pub pnl_vs_entry: f64,
}
```

**Ground truth**: Underlying received. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ underlying_received }`

**promptSnippet**: "Sells YT before maturity. Sell early -- YT value accelerates toward zero as maturity approaches. Allowed at any phase."

### `pendle_add_lp`

Add liquidity to a Pendle AMM market (SY + PT). LP earns trading fees and PENDLE incentives.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Resolving market", "Computing SY/PT split", "Simulating in Revm", "Broadcasting addLiquidityDualTokenAndPt"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleAddLpParams {
    /// Pendle market address.
    pub market: String,
    /// Underlying amount to provide as liquidity.
    pub amount: String,
    /// Default: 100 bps.
    #[serde(default = "default_slippage_100")]
    pub slippage_bps: u32,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendleLpAdded {
    pub tx_hash: String,
    pub lp_received: String,
    pub sy_deposited: String,
    pub pt_deposited: String,
    pub apy: f64,
}
```

**Ground truth**: LP token balance increased. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ lp_received, apy }`

**promptSnippet**: "Adds liquidity to a Pendle AMM market (SY + PT). Earns trading fees and PENDLE incentives. Phase-gated: blocked in declining/terminal."

### `pendle_remove_lp`

Remove liquidity from a Pendle market.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Simulating removal", "Broadcasting removeLiquidityDualTokenAndPt"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleRemoveLpParams {
    /// Pendle market address.
    pub market: String,
    /// LP tokens to redeem.
    pub lp_amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendleLpRemoved {
    pub tx_hash: String,
    pub sy_received: String,
    pub pt_received: String,
    pub underlying_value: String,
}
```

**Ground truth**: LP tokens burned, SY+PT received. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ sy_received, pt_received }`

**promptSnippet**: "Removes liquidity from a Pendle market. Returns SY and PT components. Allowed at any phase."

### `pendle_redeem`

Redeem Principal Token at maturity for the underlying asset. Only callable after maturity.

**Capability**: `WriteTool` | **Risk tier**: `Layer1` (risk-reducing, guaranteed par) | **Tick budget**: `Slow`
**Progress steps**: `["Verifying maturity", "Broadcasting redeemPyToToken"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleRedeemParams {
    /// PT token address.
    pub pt: String,
    /// PT amount to redeem.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendleRedeemed {
    pub tx_hash: String,
    pub underlying_received: String,
    pub realized_apy: f64,
}
```

**Ground truth**: Underlying received at par. Source: `"erc20_balance_of"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ underlying_received, realized_apy }`

**promptSnippet**: "Redeems PT at maturity for underlying at par. Only callable after maturity date. Check maturity via `pendle_get_markets` or `pendle_get_position`."

**promptGuidelines**:
- "In thriving phase: redeem on maturity date for full par value."
- "In conservation/declining phase: redeem any mature PTs for operating capital."
- "In terminal phase: redeem all mature PTs as part of settlement."

### `pendle_get_markets`

List active Pendle markets with implied APY, TVL, and maturity data.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Querying Pendle API", "Enriching with on-chain data"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleGetMarketsParams {
    /// Chain ID.
    pub chain_id: u64,
    /// Filter by underlying asset (e.g., "sUSDe", "stETH").
    pub underlying: Option<String>,
    /// Minimum market TVL in USD.
    pub min_tvl: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct PendleMarketsResult {
    pub markets: Vec<PendleMarketInfo>,
}

#[derive(Debug, Serialize)]
pub struct PendleMarketInfo {
    pub address: String,
    pub name: String,
    pub underlying: String,
    /// e.g., "Ethena", "Lido"
    pub underlying_protocol: String,
    pub maturity: String,
    pub days_to_maturity: u32,
    pub implied_apy: f64,
    pub pt_discount: f64,
    pub tvl: f64,
    pub volume_24h: f64,
    pub pt_address: String,
    pub yt_address: String,
    pub is_active: bool,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ market_count }`

**promptSnippet**: "Lists active Pendle markets with implied APY, TVL, maturity dates, and PT/YT addresses. Filter by underlying. Read-only."

### `pendle_get_position`

Get an account's PT, YT, and LP positions across all Pendle markets.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Scanning PT balances", "Scanning YT balances", "Scanning LP positions", "Computing P&L"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct PendleGetPositionParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendlePositionResult {
    pub pt_positions: Vec<PtPosition>,
    pub yt_positions: Vec<YtPosition>,
    pub lp_positions: Vec<LpPosition>,
    pub total_value_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct PtPosition {
    pub pt: String,
    pub market: String,
    pub amount: String,
    pub value_usd: f64,
    pub maturity: String,
    pub implied_apy: f64,
    /// Cost basis vs current value toward par.
    pub pnl: f64,
}

#[derive(Debug, Serialize)]
pub struct YtPosition {
    pub yt: String,
    pub market: String,
    pub amount: String,
    pub value_usd: f64,
    pub maturity: String,
    pub accrued_yield: String,
    pub days_remaining: u32,
}

#[derive(Debug, Serialize)]
pub struct LpPosition {
    pub market: String,
    pub lp_amount: String,
    pub value_usd: f64,
    pub fees_earned: String,
    pub apy: f64,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ total_value_usd }`

**promptSnippet**: "Returns all PT, YT, and LP positions across Pendle markets for an account. Includes P&L and accrued yield. Read-only."

**Error codes**: `PENDLE_MARKET_EXPIRED`, `PENDLE_MARKET_NOT_FOUND`, `PT_NOT_MATURE`, `PENDLE_SLIPPAGE_EXCEEDED`

---

## Synthetic dollar yield -- Ethena (4 tools)

Ethena's USDe is a synthetic dollar backed by a delta-neutral position: spot staked ETH (stETH/wstETH) long + short ETH perpetual futures. The yield (sUSDe APY) comes from staking yield plus perpetual funding rate. When funding is positive (longs pay shorts), sUSDe earns substantial yield. When funding turns negative, yield drops and can briefly go negative.

Deployed on Ethereum (1).

### `ethena_stake_usde`

Stake USDe to receive sUSDe, which earns the combined staking yield and funding rate.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Checking approval", "Querying current APY", "Simulating stake in Revm", "Broadcasting deposit"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct EthenaStakeUsdeParams {
    /// USDe amount to stake.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct EthenaStakeResult {
    pub tx_hash: String,
    pub susde_received: String,
    pub current_apy: f64,
    /// Annualized funding rate contribution.
    pub funding_rate_component: f64,
    /// ETH staking yield contribution.
    pub staking_yield_component: f64,
    /// null if no lockup.
    pub lockup_period: Option<String>,
    /// sUSDe per USDe.
    pub exchange_rate: f64,
}
```

**Ground truth**: sUSDe balance increased by `susde_received`. Source: `"erc20_balance_of"`.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "ethena_stake_usde", params_hash }`
- `tool:update` -> steps
- `tool:end` -> `{ susde_received, current_apy }`

**promptSnippet**: "Stakes USDe for sUSDe. Yield from staked ETH + perp funding rate. Check `ethena_get_apy` first -- avoid staking when funding is negative. Phase-gated: blocked in declining/terminal."

**promptGuidelines**:
- "In thriving phase: stake idle USDe for yield. Check `ethena_get_apy` first -- avoid when `funding_rate_negative` is true."
- "In stable phase: stake for capital preservation. sUSDe is lower risk than perp trading."
- "In conservation phase: no new stakes."
- "In declining/terminal phase: BLOCKED."

### `ethena_unstake_usde`

Redeem sUSDe for USDe. Cooldown mode is fee-free but requires a 7-day wait. Instant mode is immediate but incurs a fee.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Checking balance", "Simulating redemption", "Broadcasting unstake"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct EthenaUnstakeUsdeParams {
    /// sUSDe amount to unstake.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Default: "cooldown" (7-day, no fee).
    #[serde(default = "default_cooldown")]
    pub mode: UnstakeMode,
}

#[derive(Debug, Serialize)]
pub enum UnstakeMode {
    Instant,
    Cooldown,
}

#[derive(Debug, Serialize)]
pub struct EthenaUnstakeResult {
    pub tx_hash: String,
    /// null for cooldown mode until claimed.
    pub usde_received: Option<String>,
    /// null for instant mode.
    pub request_id: Option<String>,
    /// null for instant mode.
    pub cooldown_end: Option<String>,
    /// null for cooldown mode.
    pub instant_fee: Option<String>,
}
```

**Ground truth**: sUSDe balance decreased. USDe received (instant) or cooldown request created. Source: `"erc20_balance_of"` or `"cooldown_contract"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ mode, usde_received_or_cooldown_end }`

**promptSnippet**: "Redeems sUSDe for USDe. Cooldown: free, 7-day wait. Instant: immediate, incurs fee. Use cooldown when lifespan permits."

**promptGuidelines**:
- "In thriving/stable phase: use cooldown mode. No rush."
- "In conservation phase: use cooldown if projected lifespan > 10 days. Otherwise instant."
- "In declining phase: use instant mode. Speed matters."
- "In terminal phase: use instant mode for immediate settlement."

### `ethena_get_apy`

Get current and historical sUSDe APY with funding rate component breakdown.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Querying Ethena API"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct EthenaGetApyParams {
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct EthenaApyResult {
    pub current_apy: f64,
    pub apy_7d_average: f64,
    pub apy_30d_average: f64,
    pub funding_rate_component: FundingRateComponent,
    pub staking_yield_component: f64,
    pub total_historical: Vec<ApyDataPoint>,
    /// Based on funding rate stability.
    pub yield_risk: YieldRisk,
}

#[derive(Debug, Serialize)]
pub struct FundingRateComponent {
    /// Annualized rate from perp shorts.
    pub current: f64,
    pub average_7d: f64,
    pub average_30d: f64,
    /// True if funding is currently inverted.
    pub is_negative: bool,
}

#[derive(Debug, Serialize)]
pub struct ApyDataPoint {
    pub date: String,
    pub apy: f64,
    pub funding_rate: f64,
}

#[derive(Debug, Serialize)]
pub enum YieldRisk {
    High,
    Medium,
    Low,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ current_apy, yield_risk }`

**promptSnippet**: "Returns current and historical sUSDe APY with funding rate breakdown. Includes yield risk classification. Read-only."

### `ethena_get_position`

Get an account's USDe and sUSDe balances including any pending cooldown redemptions.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Reading balances and cooldowns"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct EthenaGetPositionParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct EthenaPositionResult {
    pub usde_balance: String,
    pub susde_balance: String,
    pub susde_value_usd: f64,
    pub pending_cooldowns: Vec<CooldownRequest>,
    pub cumulative_yield: String,
}

#[derive(Debug, Serialize)]
pub struct CooldownRequest {
    pub request_id: String,
    pub amount: String,
    pub cooldown_end: String,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ susde_value_usd }`

**promptSnippet**: "Returns USDe and sUSDe balances with pending cooldown redemptions. Read-only."

**Error codes**: `ETHENA_PAUSED`, `ETHENA_COOLDOWN_ACTIVE`, `USDE_INSUFFICIENT_BALANCE`

---

## Custody implications (yield write tools)

All yield write tools share the same custody behavior:

- **Delegation**: Session key signs via the vault or staking contract (Yearn Vault, Convex Booster, Pendle Router, Ethena staking). CaveatEnforcer must whitelist each protocol's canonical contract addresses. Deposit amounts bounded by session key spending limits.
- **Embedded**: Privy server wallet signs. PolicyCage validates vault/staking target and amount limits.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed targets and amounts.

Withdrawal tools are always permitted. Pendle PT/YT operations carry maturity-specific risk — the custody layer validates the call but does not enforce maturity awareness; that responsibility falls to the Golem's lifecycle-phase gating.

---

## Adapter registry mapping

| Pi-facing tool | Action type | Internal ToolDef |
| --- | --- | --- |
| `preview_action` | `yield_deposit` | `yearn_deposit` (simulate) |
| `commit_action` | `yield_deposit` | `yearn_deposit` (execute) |
| `preview_action` | `yield_withdraw` | `yearn_withdraw` (simulate) |
| `commit_action` | `yield_withdraw` | `yearn_withdraw` (execute) |
| `preview_action` | `stake_lp` | `convex_stake_lp` (simulate) |
| `commit_action` | `stake_lp` | `convex_stake_lp` (execute) |
| `preview_action` | `buy_pt` | `pendle_buy_pt` (simulate) |
| `commit_action` | `buy_pt` | `pendle_buy_pt` (execute) |
| `preview_action` | `stake_usde` | `ethena_stake_usde` (simulate) |
| `commit_action` | `stake_usde` | `ethena_stake_usde` (execute) |
| `query_state` | `yield_positions` | `yearn_get_position` + `pendle_get_position` + `ethena_get_position` |
| `search_context` | `yield_vaults` | `yearn_get_vaults` |
| `search_context` | `pendle_markets` | `pendle_get_markets` |
| `search_context` | `usde_apy` | `ethena_get_apy` |

---

## Error taxonomy

| Code | Description |
| --- | --- |
| `VAULT_NOT_FOUND` | Vault address does not match any known vault on the specified chain |
| `INSUFFICIENT_SHARES` | Account holds fewer shares than requested for withdrawal |
| `CONVEX_POOL_SHUTDOWN` | Convex pool has been shut down; unstake only |
| `CONVEX_POOL_NOT_FOUND` | Pool address does not match any active Convex pool |
| `PENDLE_MARKET_EXPIRED` | Pendle market is past maturity; use `pendle_redeem` for PT |
| `PENDLE_MARKET_NOT_FOUND` | Market address not found or not active |
| `PT_NOT_MATURE` | PT has not reached maturity; cannot redeem yet |
| `PENDLE_SLIPPAGE_EXCEEDED` | Execution price exceeds slippage tolerance |
| `ETHENA_PAUSED` | Ethena staking contract is paused |
| `ETHENA_COOLDOWN_ACTIVE` | A cooldown redemption is already in progress |
| `USDE_INSUFFICIENT_BALANCE` | Account does not hold enough USDe for the requested amount |
