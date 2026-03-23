# Bardo Tools -- Liquid staking [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Lido and Rocket Pool liquid staking: stake, unstake, wrap, withdraw, yield tracking, oracle health, and mortality-aware exit routing. Category: `staking`. Total: 15 tools.

---

> **Reader orientation:** This document specifies 15 liquid staking tools in `bardo-tools` for Lido and Rocket Pool. These tools handle ETH staking, LST wrapping/unwrapping, withdrawal queue management, oracle health monitoring, and yield harvesting. You should be familiar with liquid staking mechanics (share-rate accounting, withdrawal queues, rebase vs. non-rebase tokens). Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Lido (10 tools)

Lido is the metabolic substrate for a mortal Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM). An idle ETH treasury burns days of life by sitting still. stETH and wstETH transform that idle capital into a yield stream -- earnings per block, compounded continuously, convertible to operating reserves on demand. The yield from a healthy wstETH allocation offsets a fraction of daily compute and inference costs. That fraction can mean the difference between a Golem that survives a bear market and one that terminates prematurely.

wstETH is the default agent-compatible form. It accumulates value without rebasing and works as DeFi collateral without triggering rebase-accounting bugs. The Golem holds wstETH by default and only unwraps to stETH when requesting protocol withdrawals.

### `lido_stake`

Stake ETH to receive stETH or wstETH. Checks `getCurrentStakeLimit()` first; if the limit is binding and stETH trades at a secondary discount, routes to secondary market acquisition instead.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoStakeParams {
    /// ETH amount to stake.
    pub amount: String,
    /// Output token. Default: "wstETH".
    #[serde(default = "default_output_token")]
    pub output_token: String,  // "stETH" | "wstETH"
    /// Referral address; overrides lido.referralAddress from config.
    #[serde(default)]
    pub referral: Option<String>,
    /// Chain ID (1 for Ethereum mainnet).
    pub chain: u64,
    /// Max slippage for secondary market route in bps (default: 50).
    #[serde(default = "default_slippage")]
    pub max_slippage_bps: u32,
}

fn default_output_token() -> String { "wstETH".into() }
fn default_slippage() -> u32 { 50 }

#[derive(Debug, Serialize)]
pub struct LidoStakeResult {
    pub tx_hash: String,
    pub eth_staked: String,
    pub token_received: String,
    pub token_symbol: String,
    pub share_rate: String,
    pub route: String,  // "direct" | "secondary_market"
    pub staking_limit_remaining: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking staking limit", "Approving", "Signing stake tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Stake ETH to receive stETH or wstETH. Default output is wstETH. Checks staking rate limit and routes to secondary market if limit is binding and stETH trades at a discount."

**promptGuidelines**:
- **thriving**: Stake idle ETH to earn yield. wstETH is the preferred form.
- **cautious**: Stake only if ETH is idle and not needed for gas. Keep gas reserve.
- **declining**: Do not stake new ETH. Focus on existing positions.
- **terminal**: Do not stake. Begin exit sequence.

**Events**: `tool:start` -> `tool:update` (checking limit) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (spending limit + staking limit check), `tool_result` (token received + route logged)

**Ground truth**: Compare `token_received` against wstETH/stETH balance diff. Source: on-chain balance read after confirmation.

**Error codes**: `LIDO_STAKE_LIMIT_EXCEEDED`, `LIDO_SLIPPAGE_EXCEEDED`, `LIDO_PHASE_GATE_BLOCKED`

### `lido_unstake`

Request unstaking via the Lido withdrawal queue. Accepts stETH or wstETH. Batches multiple amounts into a single call.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoUnstakeParams {
    /// Array of amounts to withdraw (each <= 1000 stETH per Lido limits).
    pub amounts: Vec<String>,
    /// Input token. Default: "stETH" (wstETH is unwrapped internally).
    #[serde(default = "default_unstake_token")]
    pub token: String,  // "stETH" | "wstETH"
    /// Chain ID.
    pub chain: u64,
}

fn default_unstake_token() -> String { "stETH".into() }

#[derive(Debug, Serialize)]
pub struct LidoUnstakeResult {
    pub tx_hash: String,
    pub request_ids: Vec<String>,
    pub total_st_eth_queued: String,
    pub estimated_wait_time: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Approving stETH", "Submitting withdrawal request", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Submit a withdrawal request to the Lido queue. Receives unstETH NFTs. Use `lido_claim_withdrawal` once finalized."

**promptGuidelines**:
- **thriving**: Unstake when reallocating capital away from staking.
- **cautious**: Unstake if yield drops significantly or oracle stalls.
- **declining**: Begin unstaking excess positions.
- **terminal**: Queue all stETH/wstETH for withdrawal. Check `lido_exit_decision` for route.

**Events**: `tool:start` -> `tool:update` (approving) -> `tool:update` (submitting) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (balance check), `tool_result` (request IDs + estimated wait logged)

**Ground truth**: Verify request IDs exist via `getWithdrawalStatus`. Source: on-chain WithdrawalQueue contract.

### `lido_wrap`

Wrap stETH to wstETH.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoWrapParams {
    /// stETH amount to wrap.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LidoWrapResult {
    pub tx_hash: String,
    pub st_eth_wrapped: String,
    pub wst_eth_received: String,
    pub share_rate: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Approving stETH", "Wrapping"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Wrap stETH to wstETH. wstETH is the preferred form -- it does not rebase and works as DeFi collateral without accounting bugs."

**Events**: `tool:start` -> `tool:update` (approving) -> `tool:update` (wrapping) -> `tool:end`

**Ground truth**: Verify wstETH balance increase. Source: on-chain balance read.

### `lido_unwrap`

Unwrap wstETH to stETH.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoUnwrapParams {
    /// wstETH amount to unwrap.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LidoUnwrapResult {
    pub tx_hash: String,
    pub wst_eth_unwrapped: String,
    pub st_eth_received: String,
    pub share_rate: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Unwrapping"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Unwrap wstETH to stETH. Only unwrap when requesting protocol withdrawals or when a specific operation requires stETH."

**Events**: `tool:start` -> `tool:update` (unwrapping) -> `tool:end`

**Ground truth**: Verify stETH balance increase. Source: on-chain balance read.

### `lido_get_position`

Get the Golem's current Lido position: stETH/wstETH balances, share accounting, realized yield since last harvest.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoGetPositionParams {
    /// Account address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LidoPosition {
    pub st_eth_balance: String,
    pub wst_eth_balance: String,
    pub shares: String,
    pub current_share_rate: String,
    pub share_rate_delta_7d: f64,
    pub share_rate_delta_30d: f64,
    pub realized_apy_7d: f64,
    pub yield_earned_since_last_harvest: String,
    pub eth_float: String,
    pub open_withdrawal_requests: u32,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching balances", "Computing yield"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Account's Lido position: stETH/wstETH balances, share rate, realized APY, yield since last harvest, pending withdrawals."

**promptGuidelines**:
- **thriving**: Monitor on heartbeat. Track share rate delta for yield accounting.
- **cautious**: Monitor daily. Check yield covers operating costs.
- **declining**: Check for pending withdrawals. Plan exit order.
- **terminal**: Final position inventory.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (no filtering)

### `lido_get_rates`

Get current Lido staking APY, share rate, and historical deltas.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoGetRatesParams {
    /// Chain ID.
    pub chain: u64,
    /// History window in days (default: 30).
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 { 30 }

#[derive(Debug, Serialize)]
pub struct LidoRates {
    pub share_rate: String,
    pub delta_7d: f64,
    pub delta_30d: f64,
    pub estimated_apy: f64,
    pub last_update_timestamp: u64,
    pub data_points: Vec<LidoRatePoint>,
    pub average_apy: f64,
    pub min_apy: f64,
    pub max_apy: f64,
}

#[derive(Debug, Serialize)]
pub struct LidoRatePoint {
    pub timestamp: u64,
    pub share_rate: String,
    pub apy_annualized: f64,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching rates"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Current Lido staking APY, share rate, and historical deltas. Use to evaluate whether staking yield covers operating costs."

**Events**: `tool:start` -> `tool:end`

### `lido_request_withdrawal`

Submit a withdrawal request via the Lido WithdrawalQueueERC721.

Same interface as `lido_unstake`. This tool exists as an alias for explicit withdrawal intent. The handler delegates to `lido_unstake` internally.

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`

### `lido_claim_withdrawal`

Claim finalized withdrawal requests. Burns the unstETH NFTs and returns ETH.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoClaimParams {
    /// Finalized unstETH token IDs to claim.
    pub request_ids: Vec<String>,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LidoClaimResult {
    pub tx_hash: String,
    pub requests_claimed: u32,
    pub eth_received: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Verifying finalization", "Claiming", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Claim finalized Lido withdrawal requests. Burns unstETH NFTs, returns ETH. Only callable on finalized requests."

**promptGuidelines**:
- **thriving**: Claim when finalized. Route ETH to restaking or operating reserves.
- **cautious**: Claim promptly. Route to operating reserves.
- **declining**: Claim all finalized requests immediately.
- **terminal**: Claim all finalized requests. ETH goes to final distribution.

**Events**: `tool:start` -> `tool:update` (verifying) -> `tool:update` (claiming) -> `tool:update` (confirming) -> `tool:end`

**Ground truth**: Verify ETH balance increase matches `eth_received`. Source: on-chain balance diff.

**Error codes**: `LIDO_WITHDRAWAL_NOT_FINALIZED`

### `lido_get_oracle_status`

Get AccountingOracle health: last finalized epoch, quorum status, estimated next rebase.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoOracleStatusParams {
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LidoOracleStatus {
    pub last_finalized_epoch: u64,
    pub expected_epoch: u64,
    pub epochs_overdue: u32,
    pub quorum_reached: bool,
    pub quorum_members_reporting: u32,
    pub quorum_size: u32,
    pub estimated_next_rebase: String,
    pub is_oracle_stalled: bool,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching oracle state"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Lido AccountingOracle health. If oracle is stalled (epochs_overdue > threshold), the Golem drops to observation mode: no high-tier inference, no new positions."

**promptGuidelines**:
- **thriving**: Check on heartbeat. Stalled oracle triggers observation mode.
- **cautious**: Check on heartbeat. Stalled oracle triggers conservation mode.
- **declining**: Check once. Oracle status does not change exit plan.
- **terminal**: Not needed.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (stalled oracle triggers behavioral engine alert)

**Error codes**: `LIDO_ORACLE_STALLED`

### `lido_harvest_yield`

Compute yield earned since the last harvest (share-rate delta x position size), swap the yield fraction to USDC, and route to operating reserves.

```rust
#[derive(Debug, Deserialize)]
pub struct LidoHarvestParams {
    /// Account address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
    /// Percentage of accumulated yield to harvest (default: 100).
    #[serde(default = "default_harvest_pct")]
    pub harvest_pct: u32,
    /// Minimum USDC out for the swap (default: 98% of fair value).
    #[serde(default)]
    pub min_usdc_out: Option<String>,
}

fn default_harvest_pct() -> u32 { 100 }

#[derive(Debug, Serialize)]
pub struct LidoHarvestResult {
    pub tx_hash: String,
    pub wst_eth_sold: String,
    pub usdc_received: String,
    pub share_rate_delta_used: String,
    pub harvest_date: String,
    pub cumulative_yield_since_last_harvest: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Slow`
**progress_steps**: `["Computing yield delta", "Unwrapping yield fraction", "Swapping to USDC", "Routing to reserves", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Harvest accumulated staking yield: compute share-rate delta, swap yield to USDC, route to operating reserves. The Golem's primary self-funding mechanism."

**promptGuidelines**:
- **thriving**: Harvest on a configurable schedule (default: weekly). Harvest 100% of yield.
- **cautious**: Harvest more frequently (daily). Ensure operating reserves stay funded.
- **declining**: Harvest all accumulated yield immediately.
- **terminal**: Harvest everything. Route to final reserves.

**Events**: `tool:start` -> `tool:update` (computing) -> `tool:update` (swapping) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (slippage check on USDC swap), `tool_result` (harvest logged as Grimoire (the Golem's persistent memory system with episodic, semantic, and strategy layers) episode)

**Ground truth**: Verify USDC received against swap output. Source: on-chain USDC balance diff.

### Lido safety invariants

All Lido tools enforce these invariants regardless of LLM instructions. Structural enforcement in the adapter layer, not prompting heuristics.

- **Address allowlist**: only canonical Lido deployed contracts are callable.
- **Asset allowlist**: ETH, stETH, wstETH, unstETH.
- **Max single-operation ETH stake**: configurable (`lido.maxStakePerTx`), default 10 ETH.
- **Phase gating**: `declining` and `terminal` block `lido_stake`. `conservation` blocks `lido_stake`.
- **Preview->commit lifecycle**: all write tools go through the standard `preview_action` -> `commit_action` lifecycle.

---

## Rocket Pool (5 tools)

Rocket Pool is a decentralized staking protocol. rETH is a non-rebasing receipt token (like wstETH). Rocket Pool has a higher decentralization score than Lido due to its permissionless minipool operator set, but lower TVL and liquidity.

### `rocketpool_stake`

Stake ETH to receive rETH via the Rocket Pool deposit pool.

```rust
#[derive(Debug, Deserialize)]
pub struct RocketPoolStakeParams {
    /// ETH amount to stake.
    pub amount: String,
    /// Chain ID (1 for Ethereum mainnet).
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct RocketPoolStakeResult {
    pub tx_hash: String,
    pub eth_staked: String,
    pub r_eth_received: String,
    pub exchange_rate: String,
    pub deposit_pool_balance: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking deposit pool capacity", "Signing stake tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Stake ETH to receive rETH via Rocket Pool. rETH is non-rebasing (like wstETH). Check deposit pool capacity before staking -- if full, you must wait or buy rETH on secondary market."

**promptGuidelines**:
- **thriving**: Stake for yield diversification alongside Lido. rETH is a good DeFi collateral option.
- **cautious**: Stake only if diversification from Lido is needed.
- **declining**: Do not stake new ETH.
- **terminal**: Do not stake.

**Events**: `tool:start` -> `tool:update` (checking pool) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (spending limit + deposit pool check), `tool_result` (rETH received + rate logged)

**Ground truth**: Verify rETH balance increase. Source: on-chain balance read.

**Error codes**: `ROCKETPOOL_DEPOSIT_POOL_FULL`, `ROCKETPOOL_DEPOSITS_PAUSED`

### `rocketpool_unstake`

Burn rETH to receive ETH. If the deposit pool has enough balance, this is instant. Otherwise the rETH must be sold on secondary market.

```rust
#[derive(Debug, Deserialize)]
pub struct RocketPoolUnstakeParams {
    /// rETH amount to burn.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
    /// Max slippage for secondary market fallback in bps (default: 50).
    #[serde(default = "default_rp_slippage")]
    pub max_slippage_bps: u32,
}

fn default_rp_slippage() -> u32 { 50 }

#[derive(Debug, Serialize)]
pub struct RocketPoolUnstakeResult {
    pub tx_hash: String,
    pub r_eth_burned: String,
    pub eth_received: String,
    pub route: String,  // "burn" | "secondary_market"
    pub exchange_rate: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking deposit pool", "Burning rETH / Swapping", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Burn rETH for ETH. Instant if deposit pool has balance; otherwise falls back to secondary market swap."

**promptGuidelines**:
- **thriving**: Unstake when reallocating capital.
- **cautious**: Unstake if yield drops or needs rebalancing.
- **declining**: Unstake to convert to operating reserves.
- **terminal**: Unstake all rETH immediately.

**Events**: `tool:start` -> `tool:update` (checking) -> `tool:update` (burning/swapping) -> `tool:update` (confirming) -> `tool:end`

**Ground truth**: Verify ETH balance increase. Source: on-chain balance read.

### `rocketpool_get_position`

Get an account's Rocket Pool position.

```rust
#[derive(Debug, Deserialize)]
pub struct RocketPoolGetPositionParams {
    /// Account address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct RocketPoolPosition {
    pub r_eth_balance: String,
    pub eth_equivalent: String,
    pub exchange_rate: String,
    pub exchange_rate_delta_7d: f64,
    pub exchange_rate_delta_30d: f64,
    pub realized_apy_7d: f64,
    pub realized_apy_30d: f64,
    pub yield_earned_eth: String,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching position"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Account's Rocket Pool position: rETH balance, ETH equivalent, exchange rate deltas, realized APY, yield earned."

**Events**: `tool:start` -> `tool:end`

### `rocketpool_get_rates`

Get current Rocket Pool staking APY and network metrics.

```rust
#[derive(Debug, Deserialize)]
pub struct RocketPoolGetRatesParams {
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct RocketPoolRates {
    pub exchange_rate: String,
    pub estimated_apy: f64,
    pub delta_7d: f64,
    pub delta_30d: f64,
    pub node_commission: f64,
    pub deposit_pool_balance: String,
    pub deposit_pool_capacity: String,
    pub total_staked_eth: String,
    pub active_minipools: u32,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching rates"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Rocket Pool staking APY, exchange rate, node commission, deposit pool status, and network stats."

**Events**: `tool:start` -> `tool:end`

### `rocketpool_get_operators`

List Rocket Pool node operators with performance metrics.

```rust
#[derive(Debug, Deserialize)]
pub struct RocketPoolGetOperatorsParams {
    /// Chain ID.
    pub chain: u64,
    /// Minimum ETH staked by operator.
    #[serde(default)]
    pub min_staked: Option<f64>,
    /// Filter by commission rate (e.g. max 15%).
    #[serde(default)]
    pub max_commission: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RocketPoolOperator {
    pub address: String,
    pub minipools: u32,
    pub staked_eth: String,
    pub commission_rate: f64,
    pub performance_score: f64,
    pub slashing_history: Vec<SlashingEvent>,
}

#[derive(Debug, Serialize)]
pub struct SlashingEvent {
    pub date: String,
    pub amount: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RocketPoolOperatorsResult {
    pub operators: Vec<RocketPoolOperator>,
    pub total_operators: u32,
    pub network_average_commission: f64,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Fetching operator data"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Rocket Pool node operators with performance, slashing history, commission rates. Use for decentralization assessment."

**Events**: `tool:start` -> `tool:end`

---

## Custody implications (staking write tools)

All staking write tools share the same custody behavior:

- **Delegation**: Session key signs via the staking protocol's contract (Lido stETH, Rocket Pool rETH deposit). CaveatEnforcer must whitelist canonical staking contract addresses. ETH staking amounts are bounded by the session key's spending limit.
- **Embedded**: Privy server wallet signs. PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) validates staking target and amount limits.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed staking targets and ETH amounts.

Unstaking and withdrawal tools are always permitted regardless of Golem lifecycle phase.

> **Withdrawal tools are never phase-gated.** A Golem in any vitality phase can withdraw staked assets. This is a safety invariant, not a default that can be overridden.

---

## Staking safety invariants

All staking tools enforce these invariants in the adapter layer:

- **Address allowlist**: only canonical Lido and Rocket Pool contract addresses.
- **Asset allowlist**: ETH, stETH, wstETH, rETH, unstETH.
- **Max single-operation ETH stake**: configurable per protocol, default 10 ETH.
- **Phase gating**: `declining` and `terminal` block stake operations. `conservation` blocks staking.
- **Preview->commit lifecycle**: all write tools go through `preview_action` -> `commit_action`.

### Error reference

| Error code | Category | Trigger |
| --- | --- | --- |
| `LIDO_STAKE_LIMIT_EXCEEDED` | `policy` | Stake amount > `getCurrentStakeLimit()` and no secondary route |
| `LIDO_WITHDRAWAL_NOT_FINALIZED` | `state` | Claim called on non-finalized request |
| `LIDO_ORACLE_STALLED` | `state` | `epochsOverdue > threshold` |
| `LIDO_PHASE_GATE_BLOCKED` | `policy` | Write tool called in a phase that forbids it |
| `LIDO_SLIPPAGE_EXCEEDED` | `execution` | DEX quote slippage exceeds `maxSlippageBps` |
| `ROCKETPOOL_DEPOSIT_POOL_FULL` | `state` | Deposit pool at capacity |
| `ROCKETPOOL_DEPOSITS_PAUSED` | `state` | Protocol has paused deposits |
