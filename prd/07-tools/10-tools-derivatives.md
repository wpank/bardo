# Bardo Tools -- Derivatives [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Perpetual futures (GMX v2) and on-chain options (Panoptic). 16 tools total. **Safety constraint**: write tools are restricted to Thriving and Stable vitality phases. A Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) in Conservation phase (one of five BehavioralPhases that govern risk appetite based on remaining budget) may close existing positions but cannot open new ones. Phase gate runs in the handler preamble before any state-mutating call.

---

> **Reader orientation:** This document specifies 16 derivatives tools in `bardo-tools`: 8 for GMX v2 perpetual futures and 8 for Panoptic on-chain options. Write tools are phase-gated to Thriving and Stable phases only. You should be familiar with perpetual futures (funding rates, leverage, liquidation prices) and options basics (puts, calls, spreads). Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Perpetuals -- GMX v2 (8 tools)

GMX v2 uses a synthetic asset model where a Global LP pool (GM tokens) provides liquidity for all positions. Pricing uses Chainlink + Pyth oracles; there is no order book. Deployed on Arbitrum (chain 42161) and Avalanche (chain 43114).

All GMX tools use Alloy's (the standard Rust library for EVM interaction) `sol!` macro for type-safe contract bindings against GMX v2's `ExchangeRouter`, `Reader`, and `DataStore` contracts.

### `gmx_open_position`

Open a leveraged perpetual position on GMX v2.

**Capability**: `WriteTool` | **Risk tier**: `Layer3` (unbounded leverage exposure) | **Tick budget**: `Slow` (15s -- keeper execution)
**Progress steps**: `["Validating market", "Simulating in Revm (Rust EVM implementation for local transaction simulation) fork", "Broadcasting createOrder", "Awaiting keeper execution"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct GmxOpenPositionParams {
    /// Market address or symbol (e.g., "ETH/USD", "BTC/USD").
    pub market: String,
    /// Position direction.
    pub side: Side,
    /// Collateral token address or symbol.
    pub collateral: String,
    /// Collateral amount (human-readable, e.g., "1000").
    pub collateral_amount: String,
    /// Position size in USD notional.
    pub size_delta: String,
    /// Chain ID (42161 for Arbitrum, 43114 for Avalanche).
    pub chain_id: u64,
    /// Max acceptable entry price for slippage control.
    pub acceptable_price: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GmxPositionOpened {
    pub tx_hash: String,
    /// Unique on-chain position identifier.
    pub position_key: String,
    pub market: String,
    pub side: Side,
    pub entry_price: f64,
    pub leverage: f64,
    pub collateral_usd: f64,
    pub size_usd: f64,
    pub liquidation_price: f64,
    pub fees: GmxFees,
}

#[derive(Debug, Serialize)]
pub struct GmxFees {
    /// Position fee in USD.
    pub open_fee: String,
    /// Accrued funding at open.
    pub funding_fee: String,
    /// Keeper execution fee in ETH.
    pub execution_fee: String,
}
```

**Ground truth**: Receipt confirmation of `createOrder` + position existence check via `Reader.getPosition()` post-execution. Expected: position key exists with matching `sizeUsd`. Actual: on-chain position state. Source: `"gmx_reader_contract"`.

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components):
- `tool:start` -> `{ tool_name: "gmx_open_position", params_hash }`
- `tool:update` -> `"Validating market"` -> `"Simulating in Revm fork"` -> `"Broadcasting createOrder"` -> `"Awaiting keeper execution"`
- `tool:end` -> `{ success, duration_ms, result_summary: { position_key, leverage, size_usd } }`

**promptSnippet**: "Opens a leveraged perp on GMX v2 (Arbitrum/Avalanche). Uses oracle pricing, no order book. Check `get_gmx_markets` and `get_gmx_funding_rate` before opening. Phase-gated: blocked in conservation/declining/terminal."

**promptGuidelines**:
- "In thriving phase: open positions with clear directional conviction. Set stop-loss via `gmx_set_stop_loss_take_profit` immediately after."
- "In stable phase: open smaller positions only. Max 3x leverage. Always set stop-loss."
- "In conservation phase: BLOCKED for new positions. Use `gmx_close_position` to unwind."
- "In declining/terminal phase: BLOCKED."
- "ALWAYS check `get_gmx_funding_rate` before opening -- unfavorable funding erodes P&L."

### `gmx_close_position`

Close or partially reduce a GMX v2 position. Pass the position's full size in `size_delta` for a full close.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` (bounded -- closing reduces exposure) | **Tick budget**: `Slow`
**Progress steps**: `["Loading position", "Simulating close in Revm", "Broadcasting decreaseOrder", "Awaiting keeper execution"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct GmxClosePositionParams {
    /// From `gmx_open_position` or `get_gmx_position`.
    pub position_key: String,
    /// USD amount to close.
    pub size_delta: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Acceptable execution price.
    pub acceptable_price: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GmxPositionClosed {
    pub tx_hash: String,
    pub pnl_usd: f64,
    pub fees_usd: f64,
    pub collateral_returned: String,
}
```

**Ground truth**: Position no longer exists (full close) or `sizeUsd` decreased by `size_delta`. Expected: `"position closed"` or `"size reduced by {size_delta}"`. Actual: `Reader.getPosition()` state. Source: `"gmx_reader_contract"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ pnl_usd, fees_usd }`

**promptSnippet**: "Closes or partially reduces a GMX v2 position. Allowed at any phase. Pass full size for complete close."

**promptGuidelines**:
- "In thriving/stable phase: close to take profit or cut losses per strategy."
- "In conservation phase: close positions to reduce exposure. Mandatory."
- "In declining phase: close ALL positions."
- "In terminal phase: close any remaining positions for settlement."

### `gmx_adjust_collateral`

Add or remove collateral from a live GMX position to change leverage.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Loading position", "Simulating adjustment", "Broadcasting updateOrder"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct GmxAdjustCollateralParams {
    /// Position identifier.
    pub position_key: String,
    /// Positive to add collateral, negative to remove.
    pub collateral_delta: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GmxCollateralAdjusted {
    pub tx_hash: String,
    pub new_collateral_usd: f64,
    pub new_leverage: f64,
    pub new_liquidation_price: f64,
}
```

**Ground truth**: `Reader.getPosition()` shows updated collateral. Expected: collateral changed by `collateral_delta`. Actual: new collateral value. Source: `"gmx_reader_contract"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ new_leverage, new_liquidation_price }`

**promptSnippet**: "Adjusts collateral on a live GMX position to change leverage. Adding collateral reduces leverage and lowers liquidation risk. Removing increases leverage."

**promptGuidelines**:
- "In thriving phase: adjust to optimize leverage for strategy."
- "In conservation phase: only ADD collateral to reduce leverage. Removing is blocked."
- "In declining/terminal phase: close positions instead of adjusting."

### `gmx_set_stop_loss_take_profit`

Attach stop-loss and take-profit trigger orders to a GMX position. Either field may be omitted to set only one order type.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Validating trigger prices", "Creating trigger orders"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct GmxSetStopLossTakeProfitParams {
    /// Position identifier.
    pub position_key: String,
    /// Oracle trigger price for stop-loss.
    pub stop_loss_price: Option<String>,
    /// Oracle trigger price for take-profit.
    pub take_profit_price: Option<String>,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GmxStopLossTakeProfitSet {
    pub stop_loss_order_key: Option<String>,
    pub take_profit_order_key: Option<String>,
}
```

**Ground truth**: Order keys exist in GMX `DataStore`. Source: `"gmx_datastore_contract"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ stop_loss_order_key, take_profit_order_key }`

**promptSnippet**: "Sets stop-loss and/or take-profit trigger orders on a GMX position. ALWAYS call immediately after opening a new position."

**promptGuidelines**:
- "In thriving phase: set stop-loss at max acceptable loss. Take-profit at target."
- "In conservation phase: tighten stop-losses to reduce max drawdown."
- "NEVER leave a position without a stop-loss."

### `get_gmx_position`

Read the current state of a GMX v2 position.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Reading position from Reader contract"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetGmxPositionParams {
    /// Position identifier.
    pub position_key: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GmxPosition {
    pub position_key: String,
    pub market: String,
    pub side: Side,
    pub account: String,
    pub collateral_token: String,
    pub collateral_usd: f64,
    pub size_usd: f64,
    pub leverage: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl_usd: f64,
    pub liquidation_price: f64,
    pub funding_fee_accrued: String,
    pub is_liquidatable: bool,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ unrealized_pnl_usd, leverage }`

**promptSnippet**: "Reads current state of a GMX v2 position: P&L, leverage, liquidation price, funding fees. Read-only."

### `get_gmx_markets`

List all available GMX v2 markets on a chain.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Querying GMX Reader for markets"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetGmxMarketsParams {
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GmxMarketsResult {
    pub markets: Vec<GmxMarketInfo>,
}

#[derive(Debug, Serialize)]
pub struct GmxMarketInfo {
    pub address: String,
    pub index_token: String,
    pub long_token: String,
    pub short_token: String,
    pub max_leverage: f64,
    pub funding_rate: f64,
    pub open_interest_long: f64,
    pub open_interest_short: f64,
    pub borrowing_factor: f64,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ market_count }`

**promptSnippet**: "Lists all GMX v2 markets on a chain with funding rates, open interest, and max leverage. Read-only discovery."

### `get_gmx_funding_rate`

Get the current funding rate for a GMX market. Positive means longs pay shorts; negative means shorts pay longs.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Reading funding rate from DataStore"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetGmxFundingRateParams {
    /// Market address or symbol.
    pub market: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GmxFundingRateResult {
    pub market: String,
    pub funding_rate_per_hour: f64,
    pub long_funding: f64,
    pub short_funding: f64,
    pub next_funding_time: String,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ funding_rate_per_hour }`

**promptSnippet**: "Gets current funding rate for a GMX market. Positive = longs pay shorts. Check before opening a position to avoid unfavorable funding."

### `get_gmx_fees`

Estimate the fee breakdown for opening or closing a GMX position of a given size.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Calculating fee estimate"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetGmxFeesParams {
    /// Market address or symbol.
    pub market: String,
    /// Position size in USD.
    pub size_delta: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GmxFeesResult {
    pub position_fee: String,
    pub funding_fee: String,
    pub execution_fee: String,
    pub total_fee_usd: f64,
    pub fee_pct: f64,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ total_fee_usd, fee_pct }`

**promptSnippet**: "Estimates fee breakdown for a GMX position of a given size: position fee, funding, execution fee. Use to compare cost before opening."

**Error codes**: `GMX_MARKET_PAUSED`, `GMX_BELOW_MIN_COLLATERAL`, `GMX_ABOVE_MAX_LEVERAGE`, `POSITION_NOT_FOUND`, `GMX_PRICE_IMPACT_TOO_HIGH`

---

## Options -- Panoptic (8 tools)

Panoptic implements options as Uniswap V4 LP positions. A call option is a long single-sided LP position above spot; a put option is below spot. Options are perpetual (no fixed expiry) and priced by LP fees rather than Black-Scholes. This architecture makes Panoptic positions fully composable with V4.

All Panoptic tools use Alloy's `sol!` macro against the `SemiFungiblePositionManager` and `PanopticPool` contracts.

### `panoptic_buy_option`

Buy an options contract on Panoptic.

**Capability**: `WriteTool` | **Risk tier**: `Layer3` (options are high-risk derivatives) | **Tick budget**: `Slow`
**Progress steps**: `["Validating pool and strike", "Computing collateral requirement", "Simulating in Revm fork", "Broadcasting mintTokenizedPosition"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PanopticBuyOptionParams {
    /// Uniswap V4 pool address.
    pub pool: String,
    /// Strike price as a V4 tick.
    pub strike: i32,
    /// Tick range width in tick spacings (1 = minimum).
    pub width: u32,
    /// Option type.
    pub option_type: OptionType,
    /// Notional amount in quote token.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticOptionOpened {
    pub tx_hash: String,
    pub position_id: String,
    pub pool: String,
    pub option_type: OptionType,
    pub strike: i32,
    /// Strike tick converted to USD price.
    pub strike_price: f64,
    pub width: u32,
    pub notional: String,
    /// Stream-priced, not paid upfront.
    pub premium: String,
    pub collateral_required: String,
    pub greeks: Greeks,
    pub max_loss: String,
}

#[derive(Debug, Serialize)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    /// Negative for buyers -- time decay.
    pub theta: f64,
    pub vega: f64,
}
```

**Ground truth**: Position exists in `SemiFungiblePositionManager` with matching token ID. Expected: `"option position created with notional {amount}"`. Actual: on-chain position state. Source: `"panoptic_sfpm_contract"`.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "panoptic_buy_option", params_hash }`
- `tool:update` -> `"Validating pool and strike"` -> `"Computing collateral requirement"` -> `"Simulating in Revm fork"` -> `"Broadcasting mintTokenizedPosition"`
- `tool:end` -> `{ success, duration_ms, result_summary: { position_id, option_type, strike_price, max_loss } }`

**promptSnippet**: "Buys a perpetual option on Panoptic (built on Uniswap V4 LP positions). Premium is stream-priced via LP fees, not paid upfront. Phase-gated: blocked in conservation/declining/terminal."

**promptGuidelines**:
- "In thriving phase: buy options for directional bets or hedging. Size to max 5% of portfolio per position."
- "In stable phase: buy only for hedging existing exposure. No speculative positions."
- "In conservation/declining/terminal phase: BLOCKED for new buys. Use `panoptic_close_option` to unwind."
- "ALWAYS check `get_panoptic_option_chain` for available strikes and premiums first."
- "Set a mental stop -- if unrealized loss exceeds 50% of max_loss, close early."

### `panoptic_sell_option`

Sell (write) an options contract on Panoptic. The seller earns LP fees as premium but faces increasing collateral requirements as the option moves in-the-money.

**Capability**: `WriteTool` | **Risk tier**: `Layer3` | **Tick budget**: `Slow`
**Progress steps**: `["Validating pool and strike", "Computing collateral requirement", "Simulating in Revm fork", "Broadcasting mintTokenizedPosition"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PanopticSellOptionParams {
    /// Uniswap V4 pool address.
    pub pool: String,
    /// Strike price as a V4 tick.
    pub strike: i32,
    /// Tick range width in tick spacings.
    pub width: u32,
    /// Option type.
    pub option_type: OptionType,
    /// Notional to sell.
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticOptionSold {
    pub tx_hash: String,
    pub position_id: String,
    pub premium: String,
    pub collateral_required: String,
    pub greeks: Greeks,
    /// Maximum gain for the seller (capped).
    pub max_gain: String,
}
```

**Ground truth**: Short position exists in SFPM. Source: `"panoptic_sfpm_contract"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ position_id, premium, collateral_required }`

**promptSnippet**: "Sells (writes) a Panoptic option. Earns streaming premium but faces unlimited loss on naked positions. Phase-gated: blocked in conservation/declining/terminal."

**promptGuidelines**:
- "In thriving phase: sell options for income generation. Always hedge with the underlying or pair with a bought option (spread)."
- "In stable phase: sell only covered options. Naked selling blocked by risk policy."
- "In conservation/declining/terminal phase: BLOCKED."
- "WARNING: Naked sold options have unlimited loss potential. Monitor collateral utilization."

### `panoptic_close_option`

Close an existing Panoptic options position.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Loading position", "Simulating close", "Broadcasting burnTokenizedPosition"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PanopticCloseOptionParams {
    /// From `panoptic_buy_option` or `panoptic_sell_option`.
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticOptionClosed {
    pub tx_hash: String,
    pub realized_pnl: f64,
    pub premium_earned: String,
    pub collateral_returned: String,
}
```

**Ground truth**: Position no longer exists in SFPM. Source: `"panoptic_sfpm_contract"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ realized_pnl, premium_earned }`

**promptSnippet**: "Closes a Panoptic options position. Allowed at any phase."

**promptGuidelines**:
- "In conservation phase: close speculative positions first, hedges last."
- "In declining phase: close ALL options positions."

### `panoptic_exercise_option`

Force-exercise an in-the-money Panoptic position. This is the perpetual analog of expiry exercise: it forces early liquidation of an ITM position. Only available to buyers.

**Capability**: `WriteTool` | **Risk tier**: `Layer2` | **Tick budget**: `Slow`
**Progress steps**: `["Verifying ITM status", "Simulating exercise", "Broadcasting forceExercise"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct PanopticExerciseOptionParams {
    /// Position to exercise.
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticOptionExercised {
    pub tx_hash: String,
    pub payout: String,
    pub intrinsic_value: String,
}
```

**Ground truth**: Position burned, payout transferred. Source: `"panoptic_pool_contract"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ payout, intrinsic_value }`

**promptSnippet**: "Force-exercises an ITM Panoptic option. Only for buyers. Reverts if the option is OTM."

### `get_panoptic_option_chain`

Get the available strike prices and premiums for a Panoptic pool's option chain.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Reading pool state", "Computing strike premiums"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetPanopticOptionChainParams {
    /// Uniswap V4 pool address.
    pub pool: String,
    /// Filter by type (default: "both").
    pub option_type: Option<OptionTypeFilter>,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticOptionChain {
    pub pool: String,
    pub current_price: f64,
    pub strikes: Vec<StrikeInfo>,
}

#[derive(Debug, Serialize)]
pub struct StrikeInfo {
    pub tick: i32,
    pub strike_price: f64,
    /// Annualized premium as LP fee rate.
    pub call_premium_bps: f64,
    pub put_premium_bps: f64,
    pub delta: f64,
    pub open_interest: String,
    pub moneyness: Moneyness,
}

#[derive(Debug, Serialize)]
pub enum Moneyness {
    DeepItm,
    Itm,
    Atm,
    Otm,
    DeepOtm,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ strike_count }`

**promptSnippet**: "Lists available strikes with premiums and Greeks for a Panoptic pool. Read-only. Call before buying or selling options."

### `get_panoptic_greeks`

Get Greeks for an existing Panoptic position.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Computing Greeks from position state"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetPanopticGreeksParams {
    /// Position identifier.
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticGreeksResult {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
    pub collateral_utilization: f64,
    pub unrealized_pnl: f64,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ delta, unrealized_pnl }`

**promptSnippet**: "Gets Greeks for a Panoptic position: delta, gamma, theta, vega, rho plus collateral utilization. Read-only."

### `get_panoptic_iv`

Get implied volatility for a Panoptic pool, derived from LP fee rates rather than Black-Scholes inversion.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Reading fee accumulation data", "Computing IV surface"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetPanopticIvParams {
    /// Uniswap V4 pool address.
    pub pool: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticIvResult {
    pub implied_volatility: f64,
    pub iv_surface: Vec<IvPoint>,
    /// Comparison to 30-day realized vol.
    pub vs_30d_realized_vol: f64,
    /// Current IV relative to historical range (0-100).
    pub iv_percentile: f64,
}

#[derive(Debug, Serialize)]
pub struct IvPoint {
    pub strike: i32,
    pub width: u32,
    pub iv: f64,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ implied_volatility, iv_percentile }`

**promptSnippet**: "Gets implied volatility for a Panoptic pool derived from LP fee rates. Includes IV surface and comparison to realized vol. Read-only."

### `get_panoptic_positions`

Get all Panoptic options positions for an account, with aggregate Greeks.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Scanning SFPM for positions", "Computing aggregate Greeks"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetPanopticPositionsParams {
    /// Account address.
    pub account: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PanopticPositionsResult {
    pub positions: Vec<PanopticPositionInfo>,
    pub total_delta: f64,
    pub total_vega: f64,
}

#[derive(Debug, Serialize)]
pub struct PanopticPositionInfo {
    pub position_id: String,
    pub pool: String,
    pub option_type: OptionType,
    pub strike: i32,
    pub width: u32,
    pub notional: String,
    pub unrealized_pnl: f64,
    pub greeks: Greeks,
    pub collateral_utilization: f64,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ position_count, total_delta }`

**promptSnippet**: "Returns all Panoptic positions for an account with per-position and aggregate Greeks. Read-only."

**Error codes**: `PANOPTIC_POOL_NOT_FOUND`, `STRIKE_OUT_OF_RANGE`, `INSUFFICIENT_PREMIUM_COLLATERAL`, `OPTION_NOT_EXERCISABLE`

---

## Cross-derivatives tools (2 tools)

These tools span both GMX and Panoptic, providing cross-protocol views.

### `get_perp_liquidation_risk`

Check liquidation risk across all open perpetual positions for an account.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Scanning GMX positions", "Computing liquidation distances"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetPerpLiquidationRiskParams {
    /// Account address.
    pub account: String,
    /// Omit to check across all supported chains.
    pub chain_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PerpLiquidationRiskResult {
    pub positions: Vec<PerpRiskPosition>,
    pub highest_risk: Option<HighestRisk>,
}

#[derive(Debug, Serialize)]
pub struct PerpRiskPosition {
    pub venue: String,
    pub market: String,
    pub side: Side,
    pub leverage: f64,
    pub liquidation_price: f64,
    /// Percentage distance from current price to liquidation.
    pub distance_to_liquidation: f64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Serialize)]
pub struct HighestRisk {
    pub venue: String,
    pub market: String,
    pub risk_level: RiskLevel,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ position_count, highest_risk_level }`

**promptSnippet**: "Checks liquidation risk across all GMX perp positions. Returns distance to liquidation and risk level per position. Read-only."

### `estimate_carry_trade`

Estimate net APY for a delta-neutral carry trade: hold spot, short via perp to collect funding.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Fetching spot price", "Querying GMX funding rate", "Computing carry economics"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct EstimateCarryTradeParams {
    /// Asset to hold spot (e.g., "ETH").
    pub asset: String,
    /// Notional position size in USD.
    pub amount: String,
    /// Chain ID for spot leg.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct CarryTradeEstimate {
    pub asset: String,
    pub perp_venue: String,
    pub spot_position: String,
    pub perp_position: String,
    pub funding_rate_annualized: f64,
    /// Cost of holding spot if sourced via lending.
    pub borrow_cost_annualized: f64,
    pub estimated_net_apy_pct: f64,
    pub break_even_funding_rate: f64,
    pub risk_factors: Vec<String>,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ estimated_net_apy_pct }`

**promptSnippet**: "Estimates net APY for a delta-neutral carry trade (spot long + perp short). Factors in funding rate and borrow cost. Read-only."

---

## Cross-protocol risk normalization

GMX uses a leverage-based health metric. The safety tools normalize it to a unified risk scale alongside lending protocols.

| Protocol | Native metric | Safe | Warning | Danger |
| --- | --- | --- | --- | --- |
| GMX v2 | Leverage vs max | < 70% max | 70-85% max | > 85% max |
| Panoptic | Collateral utilization | < 60% | 60-80% | > 80% |

---

## Adapter registry mapping

The two-layer tool model maps GMX and Panoptic tools to Pi-facing tools:

| Pi-facing tool | Action type | Internal ToolDef |
| --- | --- | --- |
| `preview_action` | `open_perp` | `gmx_open_position` (simulate) |
| `commit_action` | `open_perp` | `gmx_open_position` (execute) |
| `preview_action` | `close_perp` | `gmx_close_position` (simulate) |
| `commit_action` | `close_perp` | `gmx_close_position` (execute) |
| `preview_action` | `buy_option` | `panoptic_buy_option` (simulate) |
| `commit_action` | `buy_option` | `panoptic_buy_option` (execute) |
| `query_state` | `perp_position` | `get_gmx_position` |
| `query_state` | `option_positions` | `get_panoptic_positions` |
| `query_state` | `option_greeks` | `get_panoptic_greeks` |
| `search_context` | `funding_rate` | `get_gmx_funding_rate` |
| `search_context` | `option_chain` | `get_panoptic_option_chain` |
| `search_context` | `implied_vol` | `get_panoptic_iv` |

---

## Custody implications (derivatives write tools)

All derivatives write tools share the same custody behavior:

- **Delegation**: Session key signs via the protocol router (GMX Router, Panoptic SemiFungiblePositionManager). CaveatEnforcer must whitelist each protocol's canonical contract addresses. Position size is bounded by session key spending limits.
- **Embedded**: Privy server wallet signs. PolicyCage validates protocol target, position size, and leverage limits.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed targets and amounts.

Perpetual positions carry ongoing liquidation risk. The custody layer validates opening and modification calls but cannot prevent on-chain liquidation by the protocol. Close position tools are always permitted regardless of lifecycle phase.

---

## Chain-specific contracts

For chain-specific V4 contract addresses, see `shared/chains.md`.

---

## Error taxonomy

| Code | Description |
| --- | --- |
| `GMX_MARKET_PAUSED` | GMX market is paused for maintenance or emergency |
| `GMX_BELOW_MIN_COLLATERAL` | Collateral amount below GMX's minimum requirement |
| `GMX_ABOVE_MAX_LEVERAGE` | Requested leverage exceeds market's maximum |
| `GMX_PRICE_IMPACT_TOO_HIGH` | Position size creates excessive price impact |
| `POSITION_NOT_FOUND` | Position key does not match any open position |
| `PANOPTIC_POOL_NOT_FOUND` | V4 pool does not have a Panoptic deployment |
| `STRIKE_OUT_OF_RANGE` | Strike tick is outside the pool's active range |
| `INSUFFICIENT_PREMIUM_COLLATERAL` | Not enough collateral to cover premium stream |
| `OPTION_NOT_EXERCISABLE` | Option is OTM and cannot be force-exercised |
