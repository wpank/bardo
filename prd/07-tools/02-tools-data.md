# Bardo Tools -- Data, historical, token, and portfolio tools [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Data querying, historical analytics, token directory, and portfolio/P&L tools. All read-only (`CapabilityTier::Read`). No safety gates, no capability tokens, no ActionPermits. For trading tools see [03-tools-trading.md](03-tools-trading.md) (swap execution, UniswapX orders, cross-chain intents, and approval management).

---

> **Reader orientation:** This document specifies the 9 read-only data and analytics tools in `bardo-tools`. These tools query pool state, token prices, positions, tick distributions, and portfolio snapshots -- none of them modify on-chain state or require capability tokens. You should be familiar with Uniswap V3/V4 pool mechanics (ticks, sqrtPriceX96, concentrated liquidity) and basic DeFi analytics concepts. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Data and analytics tools (9)

All 9 tools share these properties:

- **capability**: `CapabilityTier::Read`
- **risk_tier**: `RiskTier::Layer1`
- **sprite_trigger**: `SpriteTrigger::Thinking`
- **ground_truth_source**: N/A (read-only -- no on-chain mutation to verify)

Read tools do not require a `Capability<Self>` token. They implement the `ReadTool` trait directly.

---

### `data_get_token_price`

Get the current USD price of a token on a specific chain. Reads pool `slot0` via Alloy (the standard Rust library for EVM interaction) `sol!` as primary source, falls back to Trading API quote.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token` | `String` | Yes | Token symbol or 0x-prefixed address |
| `chain` | `String` | Yes | Chain name or chain ID |
| `quote_currency` | `String` | No | Quote currency (default: `"USD"`) |

```rust
#[derive(Debug, Deserialize)]
pub struct GetTokenPriceParams {
    /// Token symbol (e.g., "WETH") or 0x-prefixed address.
    pub token: String,
    /// Chain name ("ethereum", "base") or chain ID ("1", "8453").
    pub chain: String,
    /// Quote currency. Defaults to USD.
    #[serde(default = "default_usd")]
    pub quote_currency: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPrice {
    pub token: String,
    pub chain: String,
    pub price: f64,
    /// "slot0" or "trading_api"
    pub source: String,
    pub price_change_24h: Option<f64>,
    pub volume_24h_usd: Option<f64>,
    pub timestamp: u64,
}
```

**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Resolving token", "Reading slot0", "Formatting price"]`

**TUI rendering**: `PriceWidget` -- single-line price display with 24h delta arrow (green up / red down).

**promptSnippet**:
> Returns current USD price of a token using on-chain slot0 reads. Does not execute any transaction. Safe to call freely.

**promptGuidelines**:
- Cache results for 15s. Don't call twice in the same tick for the same token.
- If the token isn't found on-chain, it falls back to the Trading API. Check the `source` field to know which path was used.
- **thriving**: Call freely for price discovery, watchlist monitoring, and pre-trade checks.
- **cautious**: Call freely. Cross-reference against `data_get_pool_info` for accuracy on low-liquidity tokens.
- **declining**: Prefer cached portfolio data from `data_get_portfolio_snapshot` over per-token price calls.
- **terminal**: Only call when required for exit trade pricing. Do not poll for monitoring.

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components):
- `tool:start` -> `{ tool_name: "data_get_token_price", params: { token, chain } }`
- `tool:update` -> `{ step: "Reading slot0" }`
- `tool:end` -> `{ success: true, result_summary: { price, source } }`

---

### `data_get_pool_info`

Get current state of a Uniswap V3 or V4 pool: price, liquidity, TVL, volume, fees. Use when the Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) needs pool depth, TVL, current price, or fee APY. Returns tick, sqrtPriceX96, liquidity, 24h volume, and fee tier.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `pool_address` | `String` | Yes | Pool contract address (0x...) |
| `chain_id` | `u64` | No | Chain ID (default: 1) |

```rust
#[derive(Debug, Deserialize)]
pub struct GetPoolInfoParams {
    /// Pool contract address (0x-prefixed). Use data_get_pools_by_token_pair to find it.
    pub pool_address: String,
    /// Chain ID. Default: 1 (Ethereum). Common: 8453 (Base), 42161 (Arbitrum).
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct TokenMeta {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Serialize)]
pub struct PoolInfo {
    pub pool_address: String,
    pub chain_id: u64,
    /// "v3" or "v4"
    pub version: String,
    pub token0: TokenMeta,
    pub token1: TokenMeta,
    pub fee_tier: u32,
    pub sqrt_price_x96: String,
    pub tick: i32,
    pub liquidity: String,
    pub tvl_usd: f64,
    pub volume_24h_usd: f64,
    pub fee_apy_24h: f64,
    pub price_token0_in_token1: f64,
    pub price_token1_in_token0: f64,
}
```

**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Fetching slot0", "Loading subgraph data", "Computing APY"]`

**TUI rendering**: `PoolCard` -- two-column layout showing token pair, fee tier, TVL, volume, and fee APY with a mini tick-density sparkline.

**promptSnippet**:
> Use data_get_pool_info for pool state. Call data_get_pools_by_token_pair first to get the pool address.

**promptGuidelines**:
- Prefer this over manual slot0 reads -- it normalizes V3/V4 differences.
- Cache results for 15s. Don't call twice in the same tick for the same pool.
- **thriving**: Use to scout new pools and compare fee tiers. Call before any LP operation.
- **cautious**: Check TVL and volume before any capital deployment. Skip pools with TVL < $100K.
- **declining**: Only query pools where you hold active positions or plan immediate exits.
- **terminal**: Only for exit routing. Prefer `data_get_pools_by_token_pair` for deepest liquidity.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_pool_info", params: { pool_address, chain_id } }`
- `tool:update` -> `{ step: "Fetching slot0" }` | `{ step: "Loading subgraph data" }` | `{ step: "Computing APY" }`
- `tool:end` -> `{ success: true, result_summary: { tvl_usd, fee_apy_24h, version } }`

---

### `data_get_pools_by_token_pair`

Find all Uniswap pools for a token pair across V2/V3/V4, sorted by TVL. This is the entry point for discovering pool addresses -- always call before `data_get_pool_info`.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token0` | `String` | Yes | First token symbol or address |
| `token1` | `String` | Yes | Second token symbol or address |
| `chain` | `String` | Yes | Chain name or chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct GetPoolsByTokenPairParams {
    /// First token symbol (e.g., "WETH") or 0x-prefixed address.
    pub token0: String,
    /// Second token symbol or address.
    pub token1: String,
    /// Chain name or chain ID.
    pub chain: String,
}

#[derive(Debug, Serialize)]
pub struct PoolSummary {
    pub pool_address: String,
    /// "v2", "v3", or "v4"
    pub version: String,
    pub fee_tier: u32,
    pub tvl_usd: f64,
    pub volume_24h_usd: f64,
    pub fee_apy_24h: f64,
    pub token0_symbol: String,
    pub token1_symbol: String,
}

#[derive(Debug, Serialize)]
pub struct PoolsByTokenPairResult {
    pub pools: Vec<PoolSummary>,
    pub chain: String,
}
```

**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Resolving tokens", "Querying subgraph", "Sorting by TVL"]`

**TUI rendering**: `PoolTable` -- ranked table with version badge, fee tier, TVL, and 24h volume columns.

**promptSnippet**:
> Finds all Uniswap pools for a token pair across V2/V3/V4, sorted by TVL. Returns pool addresses needed by data_get_pool_info and LP tools.

**promptGuidelines**:
- Always call this before data_get_pool_info if you don't already have a pool address.
- Results are sorted by TVL descending. The first result is the most liquid pool.
- **thriving**: Compare all available pools. Factor in fee APY and version differences.
- **cautious**: Prefer V3/V4 pools with TVL > $500K and stable volume.
- **declining**: Use only for exit routing. Pick highest-TVL pool for minimal slippage.
- **terminal**: Find the deepest liquidity pool for full position exit.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_pools_by_token_pair", params: { token0, token1, chain } }`
- `tool:update` -> `{ step: "Querying subgraph" }`
- `tool:end` -> `{ success: true, result_summary: { pool_count, top_tvl_usd } }`

---

### `data_get_pools_by_token`

Find all pools containing a specific token, ranked by TVL. Useful for discovering which pairs a token trades against.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token` | `String` | Yes | Token symbol or address |
| `chain` | `String` | Yes | Chain name or chain ID |
| `limit` | `u32` | No | Max results (default: 10) |

```rust
#[derive(Debug, Deserialize)]
pub struct GetPoolsByTokenParams {
    /// Token symbol or 0x-prefixed address.
    pub token: String,
    /// Chain name or chain ID.
    pub chain: String,
    /// Max results. Default: 10.
    #[serde(default = "default_limit_10")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct PoolsByTokenResult {
    pub token: String,
    pub chain: String,
    pub pools: Vec<PoolSummary>,
}
```

**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Resolving token", "Querying subgraph", "Ranking by TVL"]`

**TUI rendering**: `PoolTable` -- same as `data_get_pools_by_token_pair`.

**promptSnippet**:
> Finds all pools containing a specific token, ranked by TVL. Use to discover which pairs a token trades against.

**promptGuidelines**:
- Use `data_get_pools_by_token_pair` if you already know both tokens. This tool is for single-token discovery.
- **thriving**: Scan broadly. Discover LP opportunities across all pools for a token.
- **cautious**: Filter to top-5 by TVL. Ignore low-liquidity pools.
- **declining**: Only query for tokens you hold. Limit to 3 results.
- **terminal**: Find the pool with the most liquidity for each held token.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_pools_by_token", params: { token, chain } }`
- `tool:update` -> `{ step: "Querying subgraph" }`
- `tool:end` -> `{ success: true, result_summary: { pool_count } }`

---

### `data_get_new_pools`

Discover recently created pools on a chain. Reads factory PoolCreated events via Alloy's `sol!` event filtering.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `chain` | `String` | Yes | Chain name or chain ID |
| `hours` | `u64` | No | Lookback window in hours (default: 24) |
| `min_tvl` | `f64` | No | Minimum TVL in USD (default: 0) |
| `limit` | `u32` | No | Max results (default: 10) |

```rust
#[derive(Debug, Deserialize)]
pub struct GetNewPoolsParams {
    pub chain: String,
    #[serde(default = "default_24")]
    pub hours: u64,
    #[serde(default)]
    pub min_tvl: f64,
    #[serde(default = "default_limit_10")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct NewPool {
    pub pool_address: String,
    pub version: String,
    pub token0: TokenMeta,
    pub token1: TokenMeta,
    pub fee_tier: u32,
    pub tvl_usd: f64,
    pub created_at: u64,
    pub created_block: u64,
    pub tx_hash: String,
}

#[derive(Debug, Serialize)]
pub struct NewPoolsResult {
    pub chain: String,
    pub lookback_hours: u64,
    pub pools: Vec<NewPool>,
}
```

**tick_budget**: `TickBudget::Medium` (1-5s) -- scans recent blocks for factory events
**progress_steps**: `["Scanning factory events", "Filtering by TVL", "Enriching metadata"]`

**TUI rendering**: `NewPoolFeed` -- scrolling list with creation timestamp, token pair, and TVL badge. Pools with TVL > $100K highlighted.

**promptSnippet**:
> Discovers recently created Uniswap pools on a chain. Filters by minimum TVL and lookback window. Read-only. Always call safety_validate_token before interacting with new pools.

**promptGuidelines**:
- Set `min_tvl` to at least 10000 to filter out dust pools and potential honeypots.
- New pools with very high TVL relative to token market cap deserve extra scrutiny. Cross-reference with safety_validate_token.
- **thriving**: Monitor hourly. Flag pools with TVL > $50K and verified tokens for further analysis.
- **cautious**: Reduce scan frequency to daily. Increase min_tvl filter to $200K.
- **declining**: Do not scan for new pools. Focus on existing positions.
- **terminal**: Never call. No new positions allowed.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_new_pools", params: { chain, hours } }`
- `tool:update` -> `{ step: "Scanning factory events" }` | `{ step: "Enriching metadata" }`
- `tool:end` -> `{ success: true, result_summary: { pool_count, max_tvl } }`

---

### `data_get_position`

Get details of a specific LP position by NFT token ID (V3) or position key (V4). Reads position state from the NonfungiblePositionManager (V3) or PositionManager (V4) via Alloy `sol!` bindings.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_id` | `String` | Yes | Position NFT token ID (V3) or position key (V4) |
| `chain` | `String` | Yes | Chain name or chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct GetPositionParams {
    /// V3: NFT token ID. V4: position key (bytes32 hex).
    pub token_id: String,
    pub chain: String,
}

#[derive(Debug, Serialize)]
pub struct PositionInfo {
    pub token_id: String,
    pub pool_address: String,
    pub version: String,
    pub token0: TokenMeta,
    pub token1: TokenMeta,
    pub fee_tier: u32,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: String,
    pub amount0: String,
    pub amount1: String,
    pub unclaimed_fees0: String,
    pub unclaimed_fees1: String,
    pub in_range: bool,
    pub current_tick: i32,
}
```

**Alloy reads**:

```rust
sol! {
    /// NonfungiblePositionManager.positions()
    function positions(uint256 tokenId) external view returns (
        uint96 nonce,
        address operator,
        address token0,
        address token1,
        uint24 fee,
        int24 tickLower,
        int24 tickUpper,
        uint128 liquidity,
        uint256 feeGrowthInside0LastX128,
        uint256 feeGrowthInside1LastX128,
        uint128 tokensOwed0,
        uint128 tokensOwed1
    );
}
```

**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Reading position state", "Computing token amounts", "Checking range status"]`

**TUI rendering**: `PositionCard` -- shows tick range as a visual bar relative to current tick, with green/red in-range indicator, unclaimed fees in USD.

**promptSnippet**:
> Gets details of a specific LP position including liquidity, token amounts, unclaimed fees, and in-range status. Read-only.

**promptGuidelines**:
- Use `data_get_positions_by_owner` if you need all positions for a wallet. This is for inspecting a single known position.
- The `in_range` field tells you whether the position is actively earning fees.
- **thriving**: Check in-range status and fee accrual. Use to decide whether to compound or rebalance.
- **cautious**: Monitor all positions for out-of-range drift. Trigger rebalance at 20% out-of-range.
- **declining**: Check all positions. Prioritize fee collection over rebalancing.
- **terminal**: Use to calculate total recoverable value before `remove_liquidity`.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_position", params: { token_id, chain } }`
- `tool:update` -> `{ step: "Reading position state" }`
- `tool:end` -> `{ success: true, result_summary: { in_range, unclaimed_fees_usd } }`

---

### `data_get_positions_by_owner`

Get all LP positions for an owner address across V3 and V4. Queries the NonfungiblePositionManager's `tokenOfOwnerByIndex` (V3) and the PositionManager (V4) via Alloy.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `owner` | `String` | No | Owner address (defaults to configured wallet) |
| `chain` | `String` | Yes | Chain name or chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct GetPositionsByOwnerParams {
    /// Owner address. Defaults to the Golem's wallet if omitted.
    pub owner: Option<String>,
    pub chain: String,
}

#[derive(Debug, Serialize)]
pub struct PositionsByOwnerResult {
    pub owner: String,
    pub chain: String,
    pub positions: Vec<PositionInfo>,
    pub total_value_usd: f64,
    pub total_unclaimed_fees_usd: f64,
}
```

**tick_budget**: `TickBudget::Medium` (1-5s) -- iterates over NFTs
**progress_steps**: `["Enumerating V3 NFTs", "Reading V4 positions", "Computing values"]`

**TUI rendering**: `PositionTable` -- sortable table with pool, range, liquidity, value, unclaimed fees, and in-range status columns.

**promptSnippet**:
> Gets all LP positions for a wallet across V3 and V4. Returns token amounts, unclaimed fees, and in-range status for each position.

**promptGuidelines**:
- Defaults to the Golem's own wallet. Pass `owner` explicitly to inspect another address.
- Combine with data_get_pool_info on each position's pool to get fee APY context.
- **thriving**: Call on every heartbeat cycle. Monitor all positions for range and fee status.
- **cautious**: Same frequency. Flag any position with > 50% fee concentration in one token.
- **declining**: Call once per cycle. Rank positions by recoverable value for exit prioritization.
- **terminal**: Call once to inventory all positions for orderly wind-down.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_positions_by_owner", params: { owner, chain } }`
- `tool:update` -> `{ step: "Enumerating V3 NFTs" }` | `{ step: "Reading V4 positions" }`
- `tool:end` -> `{ success: true, result_summary: { position_count, total_value_usd } }`

---

### `data_get_tick_data`

Get tick-level liquidity distribution for a pool. Reads initialized tick bitmap via Alloy `sol!` bindings and computes net liquidity at each tick.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `pool` | `String` | Yes | Pool address |
| `chain` | `String` | Yes | Chain name or chain ID |
| `range` | `u32` | No | Number of ticks around current tick (default: 100) |

```rust
#[derive(Debug, Deserialize)]
pub struct GetTickDataParams {
    /// Pool address (0x-prefixed).
    pub pool: String,
    pub chain: String,
    /// Number of ticks around current tick to return. Default: 100.
    #[serde(default = "default_range_100")]
    pub range: u32,
}

#[derive(Debug, Serialize)]
pub struct TickDataPoint {
    pub tick: i32,
    pub liquidity_net: String,
    pub liquidity_gross: String,
    pub price: f64,
}

#[derive(Debug, Serialize)]
pub struct TickDataResult {
    pub pool: String,
    pub chain: String,
    pub current_tick: i32,
    pub tick_spacing: i32,
    pub ticks: Vec<TickDataPoint>,
}
```

**tick_budget**: `TickBudget::Medium` (1-5s) -- reads multiple tick bitmap words
**progress_steps**: `["Reading tick bitmap", "Fetching initialized ticks", "Computing liquidity deltas"]`

**TUI rendering**: `TickChart` -- horizontal bar chart of liquidity at each tick, current tick marked with a vertical line. Dense liquidity zones highlighted.

**promptSnippet**:
> Gets tick-level liquidity distribution around the current tick. Use for analyzing depth, finding concentration zones, and planning LP range placement.

**promptGuidelines**:
- Increase `range` to 500+ for wide analysis of tick utilization. Keep at 100 for quick depth checks.
- Combine with data_get_pool_info to contextualize the distribution against current price and fee APY.
- **thriving**: Analyze tick distribution to find optimal range placement relative to liquidity peaks.
- **cautious**: Check for liquidity cliffs near current tick that could cause rapid IL.
- **declining**: Only call when evaluating a rebalance. Not needed for position monitoring.
- **terminal**: Skip entirely. Not needed for exits.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_tick_data", params: { pool, chain, range } }`
- `tool:update` -> `{ step: "Reading tick bitmap" }` | `{ step: "Computing liquidity deltas" }`
- `tool:end` -> `{ success: true, result_summary: { tick_count, current_tick } }`

---

### `data_get_pool_health_score`

Composite health score (0-100) combining TVL stability, volume consistency, fee generation, liquidity distribution quality, and IL risk. Higher score means safer for LP entry.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `pool` | `String` | Yes | Pool address |
| `chain` | `String` | Yes | Chain name or chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct GetPoolHealthScoreParams {
    pub pool: String,
    pub chain: String,
}

#[derive(Debug, Serialize)]
pub struct HealthBreakdown {
    /// TVL stability over 7d (0-100).
    pub tvl_score: u8,
    /// Volume consistency over 7d (0-100).
    pub volume_score: u8,
    /// Fee generation quality (0-100).
    pub fee_score: u8,
    /// Liquidity distribution quality (0-100). Higher = more evenly distributed.
    pub liquidity_distribution: u8,
    /// IL risk score (0-100). Lower = less IL risk.
    pub il_risk: u8,
}

#[derive(Debug, Serialize)]
pub struct PoolHealthScore {
    /// Composite score (0-100).
    pub health_score: u8,
    pub breakdown: HealthBreakdown,
    /// Actionable one-line recommendation.
    pub recommendation: String,
}
```

**tick_budget**: `TickBudget::Medium` (1-5s) -- aggregates multiple data sources
**progress_steps**: `["Loading pool history", "Computing TVL stability", "Analyzing liquidity shape", "Scoring"]`

**TUI rendering**: `HealthGauge` -- circular gauge showing composite score with color coding (green > 70, yellow 40-70, red < 40). Breakdown shown as small horizontal bars below.

**promptSnippet**:
> Returns a composite health score (0-100) for a pool. Combines TVL, volume, fee generation, liquidity distribution, and IL risk. Use before LP entry to assess pool quality.

**promptGuidelines**:
- Score > 70: safe for LP entry. 40-70: proceed with caution, narrow ranges recommended. < 40: avoid or exit.
- The `il_risk` sub-score uses realized volatility. High IL risk doesn't mean the pool is bad -- it means you need a wider range or shorter horizon.
- **thriving**: Screen all candidate pools. Require score >= 60 for new positions.
- **cautious**: Require score >= 70. Weight IL risk component heavily.
- **declining**: Monitor existing position pools only. Exit if score drops below 30.
- **terminal**: Not needed. Focus on exit execution.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "data_get_pool_health_score", params: { pool, chain } }`
- `tool:update` -> `{ step: "Computing TVL stability" }` | `{ step: "Scoring" }`
- `tool:end` -> `{ success: true, result_summary: { health_score, recommendation } }`

---

## ToolDef summary

| Tool | Category | Capability | Risk tier | Tick budget | Sprite |
| --- | --- | --- | --- | --- | --- |
| `data_get_token_price` | `Data` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `data_get_pool_info` | `Data` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `data_get_pools_by_token_pair` | `Data` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `data_get_pools_by_token` | `Data` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `data_get_new_pools` | `Data` | `Read` | `Layer1` | `Medium` | `Thinking` |
| `data_get_position` | `Data` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `data_get_positions_by_owner` | `Data` | `Read` | `Layer1` | `Medium` | `Thinking` |
| `data_get_tick_data` | `Data` | `Read` | `Layer1` | `Medium` | `Thinking` |
| `data_get_pool_health_score` | `Data` | `Read` | `Layer1` | `Medium` | `Thinking` |

---

## Error codes

| Code | Description |
| --- | --- |
| `TOKEN_NOT_FOUND` | Token symbol or address not recognized on this chain |
| `POOL_NOT_FOUND` | Pool address does not exist or is not a recognized AMM pool |
| `POSITION_NOT_FOUND` | Position NFT token ID or key does not exist |
| `CHAIN_NOT_SUPPORTED` | Chain ID is not in the configured chain registry |
| `SUBGRAPH_UNAVAILABLE` | Subgraph endpoint is down or returned an error |
| `RPC_ERROR` | Alloy provider returned an RPC error |

---

## Chain provider pattern

All 9 tools follow the same provider acquisition pattern:

```rust
pub async fn handle(params: P, ctx: &ToolContext) -> Result<ToolResult> {
    let chain_id = resolve_chain(&params.chain)?;
    let provider = ctx.provider(chain_id)?;

    ctx.event_fabric.emit(Subsystem::Tools, EventPayload::ToolExecutionStart {
        tool_name: TOOL_DEF.name.into(),
        params_hash: hash_params(&params),
    });

    // ... Alloy sol! reads ...

    ctx.event_fabric.emit(Subsystem::Tools, EventPayload::ToolExecutionComplete {
        tool_name: TOOL_DEF.name.into(),
        success: true,
        duration_ms: elapsed.as_millis() as u64,
    });

    Ok(ToolResult::read(result))
}
```

On-chain reads use Alloy's `sol!` macro for type-safe contract bindings:

```rust
sol! {
    #[sol(rpc)]
    interface IUniswapV3Pool {
        function slot0() external view returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );
        function liquidity() external view returns (uint128);
        function fee() external view returns (uint24);
        function token0() external view returns (address);
        function token1() external view returns (address);
    }
}
```

Subgraph queries use the `SubgraphClient` from `ToolContext` for historical and aggregate data (TVL, volume, fee earnings) that cannot be efficiently computed from on-chain state alone.
