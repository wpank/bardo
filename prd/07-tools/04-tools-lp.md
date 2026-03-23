# Bardo Tools -- LP management tools [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` (category `lp`) | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Uniswap LP position management, fee collection, optimization, TWAMM, migration, managed LP vaults (Arrakis, Gamma), and other-protocol LP tools (Curve, Balancer/Aerodrome). 30 tools total. For trading tools see [03-tools-trading.md](03-tools-trading.md) (swap execution, UniswapX orders, cross-chain intents, and approval management).

---

> **Reader orientation:** This document specifies the 30 LP management tools in `bardo-tools`. Coverage spans Uniswap V3/V4 concentrated liquidity (add, remove, rebalance, collect fees), TWAMM gradual LP, position migration, managed LP vaults (Arrakis, Gamma, Steer), and other-protocol LP (Curve, Balancer, Aerodrome). You should be comfortable with concentrated liquidity mechanics, tick math, and impermanent loss. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Uniswap LP management tools (10)

### `uniswap_lp_add_liquidity`

Add concentrated liquidity to a Uniswap V3 or V4 pool. Creates a new position NFT (V3) or position key (V4).

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct AddLiquidityParams {
    /// First token symbol or 0x-prefixed address.
    pub token0: String,
    /// Second token symbol or 0x-prefixed address.
    pub token1: String,
    /// Chain ID. Common: 1 (Ethereum), 8453 (Base), 42161 (Arbitrum).
    pub chain_id: u64,
    /// Fee tier in hundredths of a bip (100, 500, 3000, 10000). Auto-detected if omitted.
    pub fee: Option<u32>,
    /// Lower price bound (token1/token0).
    pub price_lower: f64,
    /// Upper price bound (token1/token0).
    pub price_upper: f64,
    /// Amount of token0 to deposit (human-readable decimal string).
    pub amount0: Option<String>,
    /// Amount of token1 to deposit (human-readable decimal string).
    pub amount1: Option<String>,
    /// "v3" or "v4". Auto-detected from chain support if omitted.
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddLiquidityResult {
    pub tx_hash: String,
    /// V3: NFT token ID. V4: position key hex.
    pub position_id: String,
    pub pool: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: String,
    pub amount0_deposited: String,
    pub amount1_deposited: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Computing tick range", "Simulating deposit", "Approving token0", "Approving token1", "Submitting transaction", "Confirming on-chain"]`

**Ground truth**: After tx confirmation, read position from `NonfungiblePositionManager.positions(tokenId)` (V3) or `PositionManager.getPositionInfo(positionKey)` (V4). Compare `liquidity` and deposited amounts against expected values.

**promptSnippet**:
> Adds concentrated liquidity to a Uniswap V3/V4 pool. Requires range selection (price_lower/price_upper). ALWAYS call `uniswap_lp_optimize_range` for range recommendation and `data_get_pool_info` for current tick before calling this tool.

**promptGuidelines**:
- **thriving**: Deploy up to 20% of NAV per position. Use Monte Carlo-optimized ranges from `uniswap_lp_optimize_range`. Prefer moderate risk tolerance.
- **cautious**: Maximum 10% of NAV per position. Use conservative risk tolerance (95% in-range probability). Require pool health score >= 70.
- **declining**: Do not add new liquidity. Focus on managing and exiting existing positions.
- **terminal**: Never call. Remove all existing liquidity instead.

**Events**: `tool:start` -> `tool:update` (Computing tick range) -> `tool:update` (Simulating deposit) -> `tool:update` (Approving token0) -> `tool:update` (Approving token1) -> `tool:update` (Submitting transaction) -> `tool:update` (Confirming on-chain) -> `tool:end`

---

### `uniswap_lp_remove_liquidity`

Remove liquidity from an existing position. Supports partial removal.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct RemoveLiquidityParams {
    /// Position NFT token ID (V3) or position key hex (V4).
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Percentage to remove (1-100). Default: 100.
    #[serde(default = "default_100")]
    pub percent_to_remove: u8,
    /// Collect accrued fees simultaneously. Default: true.
    #[serde(default = "default_true")]
    pub collect_fees: bool,
}

#[derive(Debug, Serialize)]
pub struct RemoveLiquidityResult {
    pub tx_hash: String,
    pub amount0_received: String,
    pub amount1_received: String,
    pub fees_collected0: String,
    pub fees_collected1: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Verifying position ownership", "Simulating removal", "Submitting transaction", "Confirming on-chain"]`

**Ground truth**: After tx, read position liquidity. For 100% removal, verify `liquidity == 0`. For partial, verify remaining liquidity matches `(100 - percent_to_remove)%` of original.

**promptSnippet**:
> Removes liquidity from an existing position. Supports partial removal (1-100%). In terminal phase, always remove 100%. Optionally collects accrued fees simultaneously.

**promptGuidelines**:
- **thriving**: Remove only when rebalancing. Prefer partial removal (50%) over full exit to maintain presence.
- **cautious**: Remove if pool health score drops below 50 or position is >80% out of range for 24h.
- **declining**: Remove positions in priority order: worst-performing first. Full removal (100%).
- **terminal**: Remove 100% from all positions. Collect fees. This is the primary exit tool.

---

### `uniswap_lp_collect_fees`

Collect accrued trading fees from an LP position without removing liquidity.

**Capability**: `Write` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct CollectFeesParams {
    /// Position NFT token ID (V3) or position key hex (V4).
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct CollectFeesResult {
    pub tx_hash: String,
    pub fees_collected0: String,
    pub fees_collected1: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Querying accrued fees", "Submitting collection", "Confirming on-chain"]`

**Ground truth**: Compare pre-tx and post-tx `tokensOwed0`/`tokensOwed1` on the position. Post-tx values should be zero.

V4 has no standalone `collect()`. Fees are auto-collected during any position modification. To collect without changing a position, a zero-delta modification (increase liquidity by 0) is used internally.

**promptSnippet**:
> Claims accrued trading fees from an LP position without removing liquidity. No risk, always safe to call. V4 uses zero-delta modification internally.

**promptGuidelines**:
- **thriving**: Collect when fees > $50 or monthly, whichever comes first. Consider `uniswap_lp_compound_fees` for auto-reinvestment.
- **cautious**: Collect weekly. Uncollected fees are at risk if pool is exploited.
- **declining**: Collect all fees immediately. Convert to stables or ETH.
- **terminal**: Collect all fees from all positions before `uniswap_lp_remove_liquidity`.

---

### `uniswap_lp_increase_liquidity`

Add more liquidity to an existing position without changing the price range.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct IncreaseLiquidityParams {
    /// Position NFT token ID or position key.
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Additional amount of token0 (human-readable).
    pub amount0: Option<String>,
    /// Additional amount of token1 (human-readable).
    pub amount1: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IncreaseLiquidityResult {
    pub tx_hash: String,
    pub liquidity_added: String,
    pub amount0_deposited: String,
    pub amount1_deposited: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Validating position range", "Simulating increase", "Approving tokens", "Submitting transaction", "Confirming on-chain"]`

**Ground truth**: Compare pre-tx and post-tx position liquidity. Delta should match `liquidity_added`.

**promptSnippet**:
> Add more liquidity to an existing position without changing the price range. Use when you have additional capital and the current range is still optimal.

**promptGuidelines**:
- **thriving**: Increase when position is in-range and `uniswap_lp_optimize_range` confirms the range is still good.
- **cautious**: Only increase if position has been consistently in-range for >7d.
- **declining**: Do not increase. Preserve capital.
- **terminal**: Never call.

---

### `uniswap_lp_rebalance_position`

Close an existing position and open a new one centered on the current price. Atomic: collect fees + remove + add in a single multicall.

**Capability**: `Write` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct RebalancePositionParams {
    /// Position to close.
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
    /// New lower price bound (token1/token0).
    pub new_price_lower: f64,
    /// New upper price bound (token1/token0).
    pub new_price_upper: f64,
    /// New fee tier. Default: same as original.
    pub fee: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct RebalancePositionResult {
    pub tx_hash: String,
    /// The old position ID (now closed).
    pub old_position_id: String,
    /// The new position ID.
    pub new_position_id: String,
    pub new_tick_lower: i32,
    pub new_tick_upper: i32,
    pub liquidity: String,
    pub fees_collected0: String,
    pub fees_collected1: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Reading old position", "Computing new tick range", "Simulating multicall", "Approving tokens", "Submitting multicall", "Confirming on-chain"]`

**Ground truth**: Verify old position liquidity is 0. Verify new position exists with expected tick range and liquidity. Compare total capital in old + fees vs new position value.

**promptSnippet**:
> Atomically removes and re-adds liquidity at a new range. Higher gas but avoids temporary capital exposure. ALWAYS call `uniswap_lp_optimize_range` for the new range recommendation.

**promptGuidelines**:
- **thriving**: Rebalance when position has been out-of-range for >12h. Use `uniswap_lp_optimize_range` with current market data for new range.
- **cautious**: Rebalance when out-of-range for >6h AND `uniswap_lp_optimize_range` confirms net APR improvement > 2%.
- **declining**: Do not rebalance. Remove liquidity instead.
- **terminal**: Never rebalance. Remove all positions outright.

---

### `uniswap_lp_migrate_position`

Migrate an LP position from V3 to V4, or between fee tiers.

**Capability**: `Write` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct MigratePositionParams {
    /// V3 position NFT token ID.
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Target version. Default: "v4".
    #[serde(default = "default_v4")]
    pub target_version: String,
    /// New fee tier. Default: same as original.
    pub target_fee: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct MigratePositionResult {
    pub tx_hash: String,
    pub old_position_id: String,
    pub new_position_id: String,
    pub target_version: String,
    pub liquidity: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Reading source position", "Simulating migration", "Removing from V3", "Adding to V4", "Confirming on-chain"]`

**Ground truth**: Verify V3 position liquidity is 0. Verify V4 position exists with matching tick range and comparable liquidity.

**promptSnippet**:
> Migrate an LP position from V3 to V4 or between fee tiers. Use when V4 offers better fee revenue or hook-based improvements for your pair.

**promptGuidelines**:
- **thriving**: Migrate to V4 when available and beneficial. Compare fee APY between versions first.
- **cautious**: Only migrate if V4 shows >20% fee APR improvement. Verify V4 hook safety.
- **declining**: Do not migrate. Remove from current version.
- **terminal**: Do not migrate. Remove only.

---

### `uniswap_lp_compound_fees`

Collect accrued fees and reinvest them into the same position to compound returns. Optionally swaps accumulated fees to match the position's current token ratio before re-depositing.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct CompoundFeesParams {
    /// Position NFT token ID (V3) or position key (V4).
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
    /// "v3" or "v4". Auto-detected if omitted.
    pub version: Option<String>,
    /// Swap fees to match the position's token ratio. Default: true.
    #[serde(default = "default_true")]
    pub swap_to_optimal_ratio: bool,
}

#[derive(Debug, Serialize)]
pub struct CompoundFeesResult {
    pub tx_hash: String,
    pub fees_collected0: String,
    pub fees_collected1: String,
    pub liquidity_added: String,
    pub swap_amount: Option<String>,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Querying accrued fees", "Computing optimal swap", "Simulating compound", "Submitting multicall", "Confirming on-chain"]`

**Ground truth**: Compare pre-tx and post-tx position liquidity. Increase should correspond to collected fee value minus swap slippage and gas.

**promptSnippet**:
> Collect fees and reinvest into the same position. Optionally swaps fees to match position's token ratio. Best for positions you plan to keep long-term.

**promptGuidelines**:
- **thriving**: Compound monthly if position is in-range and fees > $100.
- **cautious**: Compound only if position is solidly in-range (current tick within middle 50% of range).
- **declining**: Do not compound. Collect fees and hold as stables.
- **terminal**: Do not compound. Collect fees separately for maximum flexibility.

---

### `uniswap_lp_batch_operations`

Execute multiple LP operations in a single multicall transaction. Supports collecting fees from multiple positions, rebalancing, compounding, and adding/removing liquidity atomically. ~60% gas savings vs separate transactions.

**Capability**: `Write` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct BatchLpParams {
    /// Array of operations: { op_type, position_id, params }.
    /// op_type: "collect" | "compound" | "remove" | "add" | "rebalance".
    pub operations: Vec<LpOperation>,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct LpOperation {
    pub op_type: String,
    pub position_id: Option<String>,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct BatchLpResult {
    pub tx_hash: String,
    pub results: Vec<BatchOpResult>,
    pub total_gas_used: u64,
    pub gas_saved_vs_separate: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Validating operations", "Simulating multicall", "Approving tokens", "Submitting multicall", "Confirming on-chain"]`

**Ground truth**: Each operation in the batch is verified independently using its single-operation ground truth method. Batch fails atomically if any operation fails simulation.

**promptSnippet**:
> Execute multiple LP operations in a single multicall: collect, compound, remove, add, rebalance. ~60% gas savings vs separate transactions.

**promptGuidelines**:
- **thriving**: Batch fee collection across all positions monthly. Batch rebalances when multiple positions go out of range.
- **cautious**: Batch carefully. Review each operation individually. Simulation is mandatory.
- **declining**: Batch all remaining fee collections. Batch position removals for gas efficiency.
- **terminal**: Batch all `remove` + `collect` operations for fastest and cheapest exit.

---

### `uniswap_lp_claim_rewards`

Claim external rewards (liquidity mining, incentive programs) for LP positions.

**Capability**: `Write` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ClaimRewardsParams {
    /// Position NFT token ID.
    pub position_id: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Reward contract address. Auto-detected if omitted.
    pub reward_contract: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClaimRewardsResult {
    pub tx_hash: String,
    pub rewards: Vec<RewardClaim>,
    pub gas_used: u64,
    pub explorer_url: String,
}

#[derive(Debug, Serialize)]
pub struct RewardClaim {
    pub token: String,
    pub amount: String,
    pub value_usd: f64,
}
```

**Progress steps**: `["Querying pending rewards", "Submitting claim", "Confirming on-chain"]`

**Ground truth**: Compare pre-tx and post-tx `earned()` or `claimable()` on the reward contract. Post-tx should be zero or near-zero.

**promptSnippet**:
> Claim external rewards (liquidity mining, incentives) for LP positions. No risk beyond gas cost. Check reward amount vs gas cost before claiming.

**promptGuidelines**:
- **thriving**: Claim when reward value > 5x gas cost.
- **cautious**: Claim when reward value > 3x gas cost.
- **declining**: Claim all rewards immediately. Convert to stables.
- **terminal**: Claim everything.

---

### `uniswap_lp_harvest_rewards`

Harvest all pending rewards across multiple positions in a single transaction.

**Capability**: `Write` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct HarvestRewardsParams {
    /// Position IDs. Default: all positions with pending rewards.
    pub positions: Option<Vec<String>>,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct HarvestRewardsResult {
    pub tx_hash: String,
    pub total_rewards: Vec<RewardClaim>,
    pub positions_harvested: u32,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Scanning positions for rewards", "Submitting batch harvest", "Confirming on-chain"]`

**Ground truth**: Verify all harvested positions show zero claimable rewards post-tx.

**promptSnippet**:
> Harvest all pending rewards across multiple positions in one transaction. More gas-efficient than individual claims.

**promptGuidelines**:
- **thriving**: Harvest monthly. Let rewards accumulate for gas efficiency.
- **cautious**: Harvest weekly. Reduce smart contract exposure time.
- **declining**: Harvest immediately. Convert all to stables.
- **terminal**: Harvest everything in one call.

---

## Uniswap LP optimization tools (3)

### `uniswap_lp_optimize_range`

Compute optimal tick range for a new LP position using Monte Carlo simulation with Geometric Brownian Motion (GBM) and EWMA-weighted historical volatility (lambda=0.94). Models fee revenue, impermanent loss, and in-range probability across thousands of price paths.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Slow` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct OptimizeRangeParams {
    /// First token symbol or address.
    pub token0: String,
    /// Second token symbol or address.
    pub token1: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Fee tier in bps. Uses highest-liquidity tier if omitted.
    pub fee: Option<u32>,
    /// Capital to deploy in USD.
    pub amount_usd: String,
    /// Time horizon in days. Default: 30.
    #[serde(default = "default_30")]
    pub horizon_days: u32,
    /// "conservative" (95% in-range), "moderate" (80%), "aggressive" (60%). Default: "moderate".
    #[serde(default = "default_moderate")]
    pub risk_tolerance: String,
    /// Monte Carlo paths. Default: 5000, max: 50000.
    #[serde(default = "default_5000")]
    pub simulations: u32,
    /// "historical" (EWMA) or "implied". Default: "historical".
    #[serde(default = "default_historical")]
    pub volatility_source: String,
}

#[derive(Debug, Serialize)]
pub struct OptimizeRangeResult {
    pub optimal_range: RangeRecommendation,
    pub projections: RangeProjections,
    /// Always 4 alternatives: narrow (+/-5%), optimal, wide (+/-25%), full-range.
    pub candidate_ranges: Vec<CandidateRange>,
    pub methodology: String,
}

#[derive(Debug, Serialize)]
pub struct RangeRecommendation {
    pub price_lower: f64,
    pub price_upper: f64,
    pub tick_lower: i32,
    pub tick_upper: i32,
}

#[derive(Debug, Serialize)]
pub struct RangeProjections {
    pub expected_fee_apr: f64,
    pub expected_il_pct: f64,
    pub expected_net_apr: f64,
    pub in_range_probability: f64,
}

#[derive(Debug, Serialize)]
pub struct CandidateRange {
    pub label: String,
    pub width_pct: f64,
    pub fee_apr: f64,
    pub il_pct: f64,
    pub net_apr: f64,
    pub in_range_probability: f64,
}
```

**Progress steps**: `["Loading historical prices", "Computing volatility", "Running Monte Carlo simulation", "Ranking candidate ranges"]`

**promptSnippet**:
> Monte Carlo simulation (GBM + EWMA volatility) to find optimal LP range. Returns 4 candidate ranges (narrow/optimal/wide/full-range) with projected fee APR, IL, and in-range probability. ALWAYS call before `uniswap_lp_add_liquidity` or `uniswap_lp_rebalance_position`.

**promptGuidelines**:
- **thriving**: Run with 5000 simulations, moderate risk tolerance. Compare all 4 candidate ranges before deciding.
- **cautious**: Run with conservative risk tolerance (95% in-range). Prefer wider ranges even at lower APR.
- **declining**: Only run if considering a rebalance (which you probably should not be doing).
- **terminal**: Not needed. No new LP positions.

---

### `uniswap_lp_recommend_fee_tier`

Recommend the optimal fee tier for a token pair based on historical volatility and volume distribution.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct RecommendFeeTierParams {
    pub token0: String,
    pub token1: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct RecommendFeeTierResult {
    pub recommended: u32,
    pub tiers: Vec<FeeTierAnalysis>,
}

#[derive(Debug, Serialize)]
pub struct FeeTierAnalysis {
    pub fee: u32,
    pub volume_24h_usd: f64,
    pub tvl_usd: f64,
    pub fee_apy: f64,
    pub rationale: String,
}
```

**Progress steps**: `["Querying pools by fee tier", "Analyzing volume distribution", "Computing recommendation"]`

**promptSnippet**:
> Recommend optimal fee tier for a token pair based on historical volatility and volume distribution. Call before choosing a pool for LP entry.

---

### `uniswap_lp_backtest_strategy`

Backtest LP strategies against historical data. Simulates fee collection, impermanent loss, gas costs, and optional rebalancing. Supports comparing multiple strategies simultaneously -- when `strategies` is omitted, four default configurations are tested in parallel.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Slow` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct BacktestStrategyParams {
    pub token0: String,
    pub token1: String,
    pub chain_id: u64,
    pub fee: Option<u32>,
    /// Initial deposit in USD.
    pub amount_usd: String,
    /// Backtest start date (ISO 8601).
    pub start_time: String,
    /// End date. Default: now.
    pub end_time: Option<String>,
    /// JSON array of strategy configs. Default: narrow/medium/wide/full-range.
    pub strategies: Option<Vec<serde_json::Value>>,
    /// Rebalance when position is X% out of range (applies to default strategies).
    pub rebalance_threshold: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct BacktestResult {
    pub results: Vec<StrategyBacktest>,
}

#[derive(Debug, Serialize)]
pub struct StrategyBacktest {
    pub strategy: String,
    pub fees_earned_usd: f64,
    pub il_loss_usd: f64,
    pub rebalance_count: u32,
    pub rebalance_gas_usd: f64,
    pub end_value_usd: f64,
    pub net_return_pct: f64,
    pub vs_hodl_pct: f64,
    pub in_range_pct: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
}
```

**Progress steps**: `["Loading historical tick data", "Running strategy simulations", "Computing risk metrics", "Comparing strategies"]`

**promptSnippet**:
> Backtest LP strategies against historical data: fee collection, IL, gas, rebalancing. Compares 4 strategies by default. Use for strategy validation before deploying capital.

**promptGuidelines**:
- **thriving**: Run before any LP deployment > $5K. Require net return > 0% across all default strategies.
- **cautious**: Run before any LP deployment. Require net return > 3% and Sharpe > 0.5.
- **declining**: Run only if the Reflector extension is evaluating whether to adjust an existing strategy.
- **terminal**: Not needed.

---

## TWAMM tools (2)

### `uniswap_lp_submit_twamm_order`

Submit a Time-Weighted Average Market Maker order for gradual execution over a time period. Requires a V4 pool with the TWAMM hook deployed.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct SubmitTwammParams {
    /// Input token symbol or address.
    pub token_in: String,
    /// Output token symbol or address.
    pub token_out: String,
    /// Total input amount (human-readable).
    pub amount: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Duration: "1h", "4h", "24h", "7d".
    pub duration: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitTwammResult {
    pub tx_hash: String,
    pub order_id: String,
    pub estimated_completion: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Resolving TWAMM hook", "Simulating order", "Approving tokens", "Submitting order", "Confirming on-chain"]`

**Ground truth**: After tx, read the TWAMM order state from the hook contract. Verify order exists with correct amount and duration.

**promptSnippet**:
> Submit a TWAMM order for gradual execution over a time period. Use for large trades where instant execution would cause unacceptable price impact. Requires V4 pool with TWAMM hook.

**promptGuidelines**:
- **thriving**: Use for trades > 1% of pool liquidity. Choose duration based on urgency.
- **cautious**: Prefer TWAMM over instant swaps for trades > 0.5% of pool liquidity.
- **declining**: Use for large exit trades only. Shorter durations (4h).
- **terminal**: Use 1h duration for fastest TWAMM execution.

---

### `uniswap_lp_get_twamm_order_status`

Check the status of an active TWAMM order.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetTwammStatusParams {
    pub order_id: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct TwammOrderStatus {
    pub order_id: String,
    pub status: String,
    pub percent_complete: f64,
    pub amount_filled: String,
    pub average_price: f64,
    pub remaining_amount: String,
    pub estimated_completion: String,
}
```

**Progress steps**: `["Querying TWAMM hook state"]`

**promptSnippet**:
> Check TWAMM order status: percent complete, amount filled, average price, estimated completion. Call periodically after `uniswap_lp_submit_twamm_order`.

---

## Managed LP vault tools (7)

Managed LP protocols automate concentrated liquidity rebalancing on Uniswap V3/V4. Arrakis V2 uses off-chain manager strategies with on-chain vault contracts. Gamma uses a hypervisor pattern with base and limit ranges.

### `arrakis_deposit`

Deposit tokens into an Arrakis V2 managed LP vault.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ArrakisDepositParams {
    /// Vault contract address.
    pub vault: String,
    /// Token0 amount (or "0" for single-sided).
    pub amount0: String,
    /// Token1 amount (or "0" for single-sided).
    pub amount1: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ArrakisDepositResult {
    pub tx_hash: String,
    pub shares: String,
    pub token0_deposited: String,
    pub token1_deposited: String,
    pub token0_refunded: String,
    pub token1_refunded: String,
    pub current_range: TickRange,
    pub apy: f64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Querying vault state", "Simulating deposit", "Approving tokens", "Submitting deposit", "Confirming on-chain"]`

**Ground truth**: Read vault share balance post-tx. Verify shares received matches expected from `previewDeposit`.

---

### `arrakis_withdraw`

Withdraw from an Arrakis V2 managed LP vault by burning shares.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ArrakisWithdrawParams {
    pub vault: String,
    /// Shares to redeem, or "max" for full position.
    pub shares: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ArrakisWithdrawResult {
    pub tx_hash: String,
    pub token0_received: String,
    pub token1_received: String,
    pub fees_earned_token0: String,
    pub fees_earned_token1: String,
    pub explorer_url: String,
}
```

**Progress steps**: `["Previewing withdrawal", "Submitting redemption", "Confirming on-chain"]`

**Ground truth**: Verify vault share balance is zero (for "max") or reduced by the specified amount post-tx.

---

### `arrakis_get_vault_info`

Get rebalance history, manager fees, and strategy details for an Arrakis vault.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ArrakisVaultInfoParams {
    pub vault: String,
    pub chain_id: u64,
    /// Rebalance history limit.
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ArrakisVaultInfo {
    pub vault: String,
    pub strategy_type: String,
    pub rebalance_triggers: Vec<String>,
    pub last_rebalance: String,
    pub rebalances: Vec<RebalanceEvent>,
    pub manager_fees: f64,
    pub apy: f64,
    pub tvl_usd: f64,
}
```

**Progress steps**: `["Fetching vault contract state", "Loading rebalance history"]`

---

### `gamma_deposit`

Deposit tokens into a Gamma Strategies hypervisor vault.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct GammaDepositParams {
    pub vault: String,
    pub amount0: String,
    pub amount1: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GammaDepositResult {
    pub tx_hash: String,
    pub shares: String,
    pub token0_deposited: String,
    pub token1_deposited: String,
    pub token0_refunded: String,
    pub token1_refunded: String,
    pub apy: f64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Querying hypervisor state", "Simulating deposit", "Approving tokens", "Submitting deposit", "Confirming on-chain"]`

**Ground truth**: Read hypervisor share balance post-tx. Verify matches expected.

---

### `gamma_withdraw`

Withdraw from a Gamma hypervisor by burning shares.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct GammaWithdrawParams {
    pub vault: String,
    pub shares: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GammaWithdrawResult {
    pub tx_hash: String,
    pub token0_received: String,
    pub token1_received: String,
    pub fees_earned_token0: String,
    pub fees_earned_token1: String,
    pub explorer_url: String,
}
```

**Progress steps**: `["Previewing withdrawal", "Submitting redemption", "Confirming on-chain"]`

**Ground truth**: Verify share balance decreased by expected amount post-tx.

---

### `gamma_get_vault_info`

Get Gamma hypervisor metrics including current ratio, ranges, and fee revenue.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GammaVaultInfoParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GammaVaultInfo {
    pub hypervisor: String,
    pub pool: String,
    pub current_ratio: f64,
    pub base_range: TickRange,
    pub limit_range: TickRange,
    pub total_supply: String,
    pub tvl_usd: f64,
    pub fee_revenue_7d_usd: f64,
    pub fee_revenue_30d_usd: f64,
    pub manager_address: String,
}
```

**Progress steps**: `["Fetching hypervisor state", "Loading fee revenue data"]`

---

### `managed_lp_list_vaults`

List managed LP vaults across Arrakis and Gamma by chain and underlying pool.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ListManagedLpParams {
    /// "arrakis", "gamma", or "all". Default: "all".
    #[serde(default = "default_all")]
    pub protocol: String,
    pub chain_id: u64,
    /// Filter by underlying Uniswap pool address.
    pub pool: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ManagedLpVaultList {
    pub vaults: Vec<ManagedLpVaultSummary>,
}

#[derive(Debug, Serialize)]
pub struct ManagedLpVaultSummary {
    pub protocol: String,
    pub vault: String,
    pub name: String,
    pub pool: String,
    pub token0: String,
    pub token1: String,
    pub fee_tier: u32,
    pub current_range: TickRange,
    pub apy: f64,
    pub tvl_usd: f64,
    pub rebalance_count_30d: u32,
    pub management_fee: f64,
}
```

**Progress steps**: `["Querying Arrakis vaults", "Querying Gamma hypervisors", "Aggregating results"]`

---

## Curve LP tools (4)

### `curve_add_liquidity`

Add liquidity to a Curve pool. Deposits can be single-sided or balanced across all coins.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct CurveAddLiquidityParams {
    /// Curve pool address.
    pub pool: String,
    /// Amount per coin in pool order. Use "0" to skip a coin.
    pub amounts: Vec<String>,
    pub chain_id: u64,
    /// Minimum LP tokens to receive (slippage protection).
    pub min_lp_tokens: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CurveAddLiquidityResult {
    pub tx_hash: String,
    pub lp_received: String,
    pub share_of_pool: f64,
    /// Slippage/bonus vs balanced deposit in bps.
    pub bonus_bps: i32,
    pub explorer_url: String,
}
```

**Progress steps**: `["Computing deposit amounts", "Simulating deposit", "Approving tokens", "Submitting deposit", "Confirming on-chain"]`

**Ground truth**: Read LP token balance post-tx. Verify matches expected from `calc_token_amount`.

**promptSnippet**:
> Adds liquidity to a Curve pool. Single-sided or balanced. Use "0" for coins to skip.

---

### `curve_remove_liquidity`

Remove liquidity from a Curve pool.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct CurveRemoveLiquidityParams {
    pub pool: String,
    /// LP token amount to burn, or "max" for full position.
    pub lp_amount: String,
    pub chain_id: u64,
    /// Withdraw in a single coin (address or index). Default: proportional.
    pub single_coin: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CurveRemoveLiquidityResult {
    pub tx_hash: String,
    pub received: Vec<TokenAmount>,
    pub explorer_url: String,
}
```

**Progress steps**: `["Previewing withdrawal", "Submitting removal", "Confirming on-chain"]`

**Ground truth**: Verify LP token balance is zero (for "max") or reduced by specified amount post-tx.

**promptSnippet**:
> Removes liquidity from a Curve pool. "max" for full withdrawal. Optional single-coin exit.

---

### `curve_stake_gauge`

Stake Curve LP tokens in the gauge to earn CRV rewards. Also handles unstaking and claiming when `action` is set.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct CurveGaugeParams {
    pub pool: String,
    /// "stake", "unstake", or "claim". Default: "stake".
    #[serde(default = "default_stake")]
    pub action: String,
    /// LP token amount (required for stake/unstake).
    pub amount: Option<String>,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct CurveGaugeResult {
    pub tx_hash: String,
    pub action: String,
    pub amount: String,
    pub gauge: String,
    pub crv_apy_base: Option<f64>,
    pub crv_apy_boosted: Option<f64>,
    pub rewards_claimed: Option<Vec<TokenAmount>>,
    pub explorer_url: String,
}
```

**Progress steps**: `["Resolving gauge", "Simulating gauge operation", "Submitting transaction", "Confirming on-chain"]`

**Ground truth**: For stake, verify gauge balance increased. For unstake, verify gauge balance decreased and LP returned. For claim, verify reward tokens received.

---

### `curve_get_pools`

List Curve pools filtered by asset type or chain.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetCurvePoolsParams {
    pub chain_id: u64,
    pub asset: Option<String>,
    /// "stable", "crypto", "tricrypto", or "all". Default: "all".
    #[serde(default = "default_all")]
    pub pool_type: String,
}

#[derive(Debug, Serialize)]
pub struct CurvePoolList {
    pub pools: Vec<CurvePoolSummary>,
}

#[derive(Debug, Serialize)]
pub struct CurvePoolSummary {
    pub address: String,
    pub name: String,
    pub coins: Vec<String>,
    pub base_apy: f64,
    pub crv_apy: f64,
    pub total_apy: f64,
    pub tvl_usd: f64,
    pub volume_24h_usd: f64,
    pub pool_type: String,
    pub gauge: Option<String>,
    pub amplification_coef: Option<f64>,
}
```

**Progress steps**: `["Querying Curve registry", "Loading APY data"]`

**Error codes**: `CURVE_POOL_NOT_FOUND`, `CURVE_SLIPPAGE_TOO_HIGH`, `CURVE_GAUGE_NOT_FOUND`

---

## Balancer / Aerodrome LP tools (4)

### `balancer_join_pool`

Join (add liquidity to) a Balancer V3 pool. Also used for Aerodrome pools on Base/Optimism through the same interface.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct BalancerJoinParams {
    /// Pool ID (bytes32) or pool address.
    pub pool_id: String,
    /// Token addresses in pool order.
    pub assets: Vec<String>,
    /// Maximum amounts per asset.
    pub max_amounts_in: Vec<String>,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct BalancerJoinResult {
    pub tx_hash: String,
    pub bpt_received: String,
    pub join_type: String,
    pub explorer_url: String,
}
```

**Progress steps**: `["Computing join amounts", "Simulating join", "Approving tokens", "Submitting join", "Confirming on-chain"]`

**Ground truth**: Read BPT balance post-tx. Verify matches expected from vault query.

---

### `balancer_exit_pool`

Exit (remove liquidity from) a Balancer V3 pool.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct BalancerExitParams {
    pub pool_id: String,
    /// BPT amount to burn.
    pub bpt_amount: String,
    pub assets: Vec<String>,
    /// Exit in a single asset (address). Default: proportional.
    pub single_asset_out: Option<String>,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct BalancerExitResult {
    pub tx_hash: String,
    pub received: Vec<TokenAmount>,
    pub explorer_url: String,
}
```

**Progress steps**: `["Previewing exit", "Submitting exit", "Confirming on-chain"]`

**Ground truth**: Verify BPT balance decreased by specified amount post-tx.

---

### `balancer_gauge_operation`

Stake or unstake BPT tokens in a Balancer gauge to earn BAL rewards.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct BalancerGaugeParams {
    pub pool_id: String,
    /// "stake" or "unstake".
    pub action: String,
    /// BPT amount.
    pub amount: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct BalancerGaugeResult {
    pub tx_hash: String,
    pub action: String,
    pub amount: String,
    pub gauge: String,
    pub bal_apy: Option<f64>,
    pub rewards_claimed: Option<Vec<TokenAmount>>,
    pub explorer_url: String,
}
```

**Progress steps**: `["Resolving gauge", "Submitting gauge operation", "Confirming on-chain"]`

**Ground truth**: Verify gauge balance changed by expected amount post-tx.

---

### `balancer_get_pools`

List Balancer V3 pools with TVL, volume, and APY. On Base/Optimism, this returns Aerodrome pools through the same interface.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetBalancerPoolsParams {
    pub chain_id: u64,
    pub asset: Option<String>,
    /// "weighted", "stable", "composable", or "all". Default: "all".
    #[serde(default = "default_all")]
    pub pool_type: String,
}

#[derive(Debug, Serialize)]
pub struct BalancerPoolList {
    pub pools: Vec<BalancerPoolSummary>,
}

#[derive(Debug, Serialize)]
pub struct BalancerPoolSummary {
    pub id: String,
    pub address: String,
    pub pool_type: String,
    pub tokens: Vec<BalancerTokenWeight>,
    pub swap_fee: f64,
    pub swap_fee_apy: f64,
    pub reward_apy: f64,
    pub total_apy: f64,
    pub tvl_usd: f64,
    pub volume_24h_usd: f64,
}
```

**Progress steps**: `["Querying Balancer subgraph", "Loading APY data"]`

**Error codes**: `BALANCER_POOL_NOT_FOUND`, `BALANCER_SWAP_LIMIT_EXCEEDED`, `BPT_NOT_FOUND`

---

## Shared types

```rust
#[derive(Debug, Serialize)]
pub struct TickRange {
    pub tick_lower: i32,
    pub tick_upper: i32,
}

#[derive(Debug, Serialize)]
pub struct TokenAmount {
    pub token: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct BatchOpResult {
    pub op_type: String,
    pub position_id: String,
    pub status: String,
    pub gas_used: u64,
}

#[derive(Debug, Serialize)]
pub struct RebalanceEvent {
    pub timestamp: String,
    pub old_range: TickRange,
    pub new_range: TickRange,
    pub reason: String,
    pub gas_cost_usd: f64,
}
```

---

## Custody implications (all LP write tools)

All 20 write tools in this file share the same custody behavior:

- **Delegation**: Session key signs via Universal Router or protocol-specific router. CaveatEnforcer must whitelist the router address and token pair. On-chain enforcement bounds LP operations to approved pools and amounts.
- **Embedded**: Privy server wallet signs. PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) validates router target, position size against NAV percentage limits, and per-operation spending caps.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed router targets and amounts identically to Delegation mode.

For managed LP vaults (Arrakis, Gamma), the target contract is the vault address rather than the Uniswap router. CaveatEnforcers and PolicyCage must whitelist each vault contract separately.

---

## Golem integration

Golems (mortal autonomous agents, each compiled as a single Rust binary on a micro-VM) operating as LP managers use these tools in their heartbeat (the Golem's periodic 9-step decision cycle, running every 30-120 seconds) loop:

1. **Monitor**: `data_get_positions_by_owner` + `data_get_pool_info` to check position health
2. **Decide**: `uniswap_lp_optimize_range` + `intel_compute_lvr` to assess rebalance need
3. **Execute**: `uniswap_lp_rebalance_position` or `uniswap_lp_compound_fees` when conditions trigger
4. **Learn**: Grimoire (the Golem's persistent memory system with episodic, semantic, and strategy layers) stores the episode with outcome metrics for future optimization

The `uniswap_lp_backtest_strategy` tool is valuable during Golem Loop 2 (strategic reflection), where the Reflector extension evaluates whether current LP strategy parameters should be revised based on historical performance.

---

## Error codes

| Code | Category | Description |
| --- | --- | --- |
| `INVALID_RANGE` | validation | price_lower >= price_upper |
| `POSITION_NOT_FOUND` | data | Position NFT or key not found |
| `INSUFFICIENT_BALANCE` | wallet | Not enough tokens for the deposit |
| `POOL_NOT_FOUND` | data | No pool found for pair and fee tier |
| `SAFETY_SIMULATION_FAILED` | safety | Pre-flight simulation reverted |
| `INSUFFICIENT_HISTORY` | data | Not enough data for backtest period |
| `MANAGED_LP_VAULT_NOT_FOUND` | data | Managed LP vault not found |
| `VAULT_CAPACITY_REACHED` | validation | Managed LP vault is full |

---

## Tool count summary

| Group | Tools | Capability |
| --- | --- | --- |
| Uniswap LP management | 10 | 9 Write, 1 Read (collect_fees is Write at Layer1) |
| Uniswap LP optimization | 3 | 3 Read |
| TWAMM | 2 | 1 Write, 1 Read |
| Managed LP (Arrakis + Gamma) | 7 | 4 Write, 3 Read |
| Curve LP | 4 | 3 Write, 1 Read |
| Balancer/Aerodrome LP | 4 | 3 Write, 1 Read |
| **Total** | **30** | **20 Write, 10 Read** |
