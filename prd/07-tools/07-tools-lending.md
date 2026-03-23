# Bardo Tools -- Lending, borrowing, and leveraged looping [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Aave V3, Morpho Blue, MakerDAO/Sky CDPs, leveraged looping with health monitoring and auto-deleverage, and cross-protocol lending utilities. Category: `lending`. Total: 27 tools.

---

> **Reader orientation:** This document specifies 27 lending, borrowing, and leveraged looping tools in `bardo-tools`. Coverage spans Aave V3, Morpho Blue isolated markets, MakerDAO/Sky CDPs, and cross-protocol utilities including recursive leverage loops with auto-deleverage. You should be comfortable with lending pool mechanics (health factors, liquidation thresholds, variable/stable rates) and collateral management. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Aave V3 (9 tools)

Deployed on Ethereum, Base, Arbitrum, Polygon, Optimism, and Avalanche. Pooled model where all depositors share liquidity. Health factor < 1.0 triggers liquidation.

### `aave_supply`

Supply assets to an Aave V3 lending pool to earn interest as aTokens.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveSupplyParams {
    /// Token address or symbol to supply.
    pub asset: String,
    /// Amount to supply (human-readable, e.g. "1000").
    pub amount: String,
    /// Chain ID (1 for Ethereum, 8453 for Base, 42161 for Arbitrum).
    pub chain: u64,
    /// Supply on behalf of another address.
    #[serde(default)]
    pub on_behalf_of: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AaveSupplyResult {
    pub tx_hash: String,
    pub a_tokens_received: String,
    pub a_token_balance: String,
    pub supply_apy: f64,
    pub health_factor_after: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking supply cap", "Approving token", "Signing supply tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Supply assets to Aave V3 to earn interest as aTokens. Check `aave_get_rates` for current APY and supply cap before calling. Health factor returned for monitoring."

**promptGuidelines**:
- **thriving**: Supply idle assets to earn yield. Verify supply cap has room. Use as collateral only if you plan to borrow.
- **cautious**: Only supply to earn yield. Do not enable as collateral unless health factor stays > 2.0.
- **declining**: Do not supply new assets. Focus on managing existing positions.
- **terminal**: Do not supply. Withdraw everything.

**Events**: `tool:start` -> `tool:update` (approving) -> `tool:update` (signing supply) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (spending limit + supply cap check), `tool_result` (aToken balance and health factor logged)

**Ground truth**: Compare `a_tokens_received` against Alloy (the standard Rust library for EVM interaction) `balanceOf` diff on the aToken contract. Source: on-chain receipt + balance read.

### `aave_withdraw`

Withdraw supplied assets from Aave V3, burning the corresponding aTokens.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveWithdrawParams {
    /// Token address or symbol to withdraw.
    pub asset: String,
    /// Amount to withdraw, or "max" for full balance.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AaveWithdrawResult {
    pub tx_hash: String,
    pub assets_withdrawn: String,
    pub a_tokens_burned: String,
    pub health_factor_after: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking health factor impact", "Signing withdrawal", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Withdraw supplied assets from Aave V3. Use 'max' for full withdrawal. Check health factor impact if assets are used as collateral -- withdrawal may trigger liquidation if it drops HF below 1.0."

**promptGuidelines**:
- **thriving**: Withdraw when reallocating capital. Always check health factor impact first.
- **cautious**: Withdraw if rate drops significantly. Keep health factor > 2.0 after withdrawal.
- **declining**: Withdraw all non-collateral positions. Withdraw collateral only after repaying debt.
- **terminal**: Repay all debt first, then withdraw everything.

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (block if withdrawal would push HF < 1.2), `tool_result` (post-withdrawal HF logged)

**Ground truth**: Verify `assets_withdrawn` against token balance diff. Source: on-chain receipt.

### `aave_borrow`

Borrow assets against supplied collateral.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveBorrowParams {
    /// Token to borrow.
    pub asset: String,
    /// Amount to borrow.
    pub amount: String,
    /// Rate mode. Stable is deprecated on most markets; prefer "variable".
    pub interest_rate_mode: String,  // "stable" | "variable"
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AaveBorrowResult {
    pub tx_hash: String,
    pub borrowed: String,
    pub variable_debt_token_balance: String,
    pub borrow_apy: f64,
    pub health_factor_after: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer3` | **tick_budget**: `Medium`
**progress_steps**: `["Simulating borrow impact", "Checking health factor guard", "Signing borrow tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Borrow against supplied collateral on Aave V3. NEVER borrow if health factor would drop below 1.5. In cautious phase, minimum health factor is 2.0. Variable rate recommended."

**promptGuidelines**:
- **thriving**: Borrow for productive use (leverage, yield farming). Maintain HF > 1.5. Call `aave_get_liquidation_risk` with -20% price change scenario.
- **cautious**: Minimum HF after borrow is 2.0. Only borrow for hedging or collateral-neutral strategies.
- **declining**: Do not borrow. Repay existing debt to reduce liquidation risk.
- **terminal**: Never borrow. Repay all debts first.

**Events**: `tool:start` -> `tool:update` (simulating) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (health factor guard: block if post-borrow HF < 1.5 in thriving, < 2.0 in cautious), `tool_result` (borrow amount + HF logged, Grimoire (the Golem's persistent memory system with episodic, semantic, and strategy layers) episode stored)

**Ground truth**: Verify `borrowed` against debt token balance increase. Source: on-chain receipt + debt token read.

### `aave_repay`

Repay borrowed assets.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveRepayParams {
    /// Token to repay.
    pub asset: String,
    /// Amount to repay, or "max" for full debt.
    pub amount: String,
    /// Rate mode of existing debt.
    pub interest_rate_mode: String,  // "stable" | "variable"
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AaveRepayResult {
    pub tx_hash: String,
    pub repaid: String,
    pub debt_remaining: String,
    pub health_factor_after: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Approving repayment token", "Signing repay tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Repay borrowed assets on Aave V3. Priority action when health factor < 1.3. In terminal phase, repay all debts first before any other operation. Use 'max' for full repayment."

**promptGuidelines**:
- **thriving**: Repay when borrow APY exceeds strategy returns or when rebalancing.
- **cautious**: Repay if health factor drops below 1.8. Target HF > 2.0.
- **declining**: Repay all debts. Debt-free positions are the priority.
- **terminal**: Repay everything immediately. This is the first action in wind-down.

**Events**: `tool:start` -> `tool:update` (approving) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (balance check for repayment amount), `tool_result` (remaining debt + HF logged)

**Ground truth**: Verify `repaid` against debt token balance decrease. Source: on-chain receipt + debt token read.

### `aave_get_health_factor`

Get an account's health factor and liquidation risk on Aave V3.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveGetHealthParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AaveHealthFactor {
    pub account: String,
    pub health_factor: f64,      // < 1.0 = liquidatable
    pub total_collateral_usd: f64,
    pub total_debt_usd: f64,
    pub available_borrows_usd: f64,
    pub current_ltv: f64,
    pub liquidation_threshold: f64,
    pub net_apy: f64,            // weighted supply APY minus weighted borrow APY
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching account data"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Get account health factor, collateral, debt, and available borrows. HF < 1.0 means liquidatable. Call on every heartbeat when Aave positions are active."

**promptGuidelines**:
- **thriving**: Monitor on heartbeat. Alert if HF < 1.5.
- **cautious**: Monitor on heartbeat. Trigger repayment if HF < 1.8.
- **declining**: Monitor continuously. Repay debt immediately if HF < 1.5.
- **terminal**: Check once. Repay all debt.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (HF < 1.3 triggers automatic alert to Pi behavioral engine)

### `aave_get_rates`

Get current supply and borrow rates for an asset on Aave V3.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveGetRatesParams {
    /// Token address or symbol.
    pub asset: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AaveRates {
    pub asset: String,
    pub symbol: String,
    pub supply_apy: f64,
    pub variable_borrow_apy: f64,
    pub stable_borrow_apy: Option<f64>,  // None if stable rate is disabled
    pub utilization_rate: f64,
    pub available_liquidity: String,
    pub total_supplied: String,
    pub total_borrowed: String,
    pub ltv: f64,
    pub liquidation_threshold: f64,
    pub supply_cap_remaining: Option<String>,
    pub borrow_cap_remaining: Option<String>,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching rates"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Current Aave V3 supply and borrow rates, utilization, available liquidity, LTV, and caps. Call before supply or borrow decisions."

**promptGuidelines**:
- **thriving**: Check before supply/borrow. Compare across protocols with `lending_compare_rates`.
- **cautious**: Check rates frequently. Exit positions where rate moves unfavorably > 2% APY.
- **declining**: Monitor borrow rates on existing debt. Repay if rates spike.
- **terminal**: Not needed. Repay all debt regardless of rate.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (no filtering)

### `aave_get_user_summary`

Get all supply and borrow positions for an account on Aave V3.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveGetUserSummaryParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AaveUserSummary {
    pub account: String,
    pub chain: u64,
    pub supplied: Vec<AaveSupplyPosition>,
    pub borrowed: Vec<AaveBorrowPosition>,
    pub health_factor: f64,
    pub net_apy: f64,
}

#[derive(Debug, Serialize)]
pub struct AaveSupplyPosition {
    pub asset: String,
    pub symbol: String,
    pub a_token_balance: String,
    pub balance_usd: f64,
    pub supply_apy: f64,
    pub used_as_collateral: bool,
}

#[derive(Debug, Serialize)]
pub struct AaveBorrowPosition {
    pub asset: String,
    pub symbol: String,
    pub debt_balance: String,
    pub debt_usd: f64,
    pub borrow_apy: f64,
    pub rate_mode: String,  // "stable" | "variable"
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching positions"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Get all Aave V3 supply and borrow positions for an account. Returns per-asset balances, APYs, and collateral status. Use for portfolio inventory."

**promptGuidelines**:
- **thriving**: Review on heartbeat. Track position changes.
- **cautious**: Review and verify all positions are expected.
- **declining**: Use to plan debt repayment order.
- **terminal**: Final inventory before full withdrawal.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (no filtering)

### `aave_enable_collateral`

Enable an asset as collateral on Aave V3.

```rust
#[derive(Debug, Deserialize)]
pub struct AaveEnableCollateralParams {
    /// Token address.
    pub asset: String,
    /// true to enable, false to disable.
    pub use_as_collateral: bool,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AaveEnableCollateralResult {
    pub tx_hash: String,
    pub asset: String,
    pub use_as_collateral: bool,
    pub health_factor_after: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking health factor impact", "Signing tx"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Enable or disable an asset as collateral on Aave V3. Enabling increases borrow capacity but exposes the asset to liquidation. Disabling reduces risk but limits borrowing."

**promptGuidelines**:
- **thriving**: Enable for assets you plan to borrow against. Disable for assets you want to protect from liquidation.
- **cautious**: Disable collateral on assets you do not actively need for borrowing.
- **declining**: Disable all collateral after repaying debt.
- **terminal**: Disable after debt repayment.

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:end`
**Pi hooks**: `tool_call` (HF impact check for disable -- block if it would trigger liquidation), `tool_result` (collateral state logged)

**Ground truth**: Verify collateral flag change via `getUserConfiguration`. Source: on-chain state read.

### `aave_disable_collateral`

Disable an asset as collateral on Aave V3. Separate tool from `aave_enable_collateral` for explicit risk-reducing intent.

Same parameters and result type as `aave_enable_collateral` with `use_as_collateral` hardcoded to `false`.

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Checking health factor impact", "Signing tx"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Disable an asset as collateral on Aave V3. Risk-reducing operation. Block if disabling would push HF below 1.0."

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:end`

**Ground truth**: Verify collateral flag change. Source: on-chain state read.

**Error codes**: `AAVE_MARKET_PAUSED`, `AAVE_SUPPLY_CAP_EXCEEDED`, `AAVE_BORROW_CAP_EXCEEDED`, `HEALTH_FACTOR_TOO_LOW`, `INSUFFICIENT_COLLATERAL`

---

## Morpho Blue (7 tools)

Permissionless lending primitive. Markets are identified by a `bytes32` marketId derived from `(loanToken, collateralToken, oracle, irm, lltv)`. The `marketId` parameter accepts either the raw bytes32 or a JSON struct with those five fields.

### `morpho_supply_market`

Supply assets as a lender to a Morpho Blue market.

```rust
#[derive(Debug, Deserialize)]
pub struct MorphoSupplyParams {
    /// bytes32 market ID or JSON struct {loanToken, collateralToken, oracle, irm, lltv}.
    pub market_id: String,
    /// Amount of loan token to supply.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
    /// Supply on behalf of another address.
    #[serde(default)]
    pub on_behalf_of: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MorphoSupplyResult {
    pub tx_hash: String,
    pub shares_received: String,
    pub assets_supplied: String,
    pub supply_apy: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Validating market", "Approving token", "Signing supply", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Supply as lender to a Morpho Blue market. Identify market by bytes32 ID or JSON struct. Supply earns interest. Verify market oracle and IRM before first interaction."

**promptGuidelines**:
- **thriving**: Supply to high-TVL markets for yield. Verify market oracle is a recognized feed.
- **cautious**: Supply only to markets with TVL > $10M. Verify oracle.
- **declining**: Do not supply new assets. Withdraw existing supply.
- **terminal**: Withdraw everything.

**Events**: `tool:start` -> `tool:update` (approving) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (spending limit + market validation), `tool_result` (position state logged)

**Ground truth**: Verify `shares_received` against Morpho supply shares delta. Source: on-chain receipt.

### `morpho_withdraw_market`

Withdraw supplied assets from a Morpho Blue market.

```rust
#[derive(Debug, Deserialize)]
pub struct MorphoWithdrawParams {
    /// Market ID.
    pub market_id: String,
    /// Amount to withdraw, or "max".
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MorphoWithdrawResult {
    pub tx_hash: String,
    pub assets_withdrawn: String,
    pub shares_repaid: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Signing withdrawal", "Confirming"]`
**sprite_trigger**: `Executing`

**Ground truth**: Verify `assets_withdrawn` against token balance diff. Source: on-chain receipt.

### `morpho_borrow_market`

Borrow the loan token against collateral in a Morpho Blue market.

```rust
#[derive(Debug, Deserialize)]
pub struct MorphoBorrowParams {
    /// Market ID.
    pub market_id: String,
    /// Amount to borrow.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MorphoBorrowResult {
    pub tx_hash: String,
    pub borrowed: String,
    pub borrow_apy: f64,
    pub ltv: f64,
    pub liquidation_ltv: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer3` | **tick_budget**: `Medium`
**progress_steps**: `["Simulating borrow", "Signing borrow tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Borrow loan token against collateral in Morpho Blue. Permissionless markets -- verify market parameters (oracle, IRM, LLTV) before interacting."

**promptGuidelines**:
- **thriving**: Borrow with HF > 1.5. Verify market oracle and IRM.
- **cautious**: Do not borrow.
- **declining**: Repay all debt.
- **terminal**: Full exit: repay -> withdraw collateral -> withdraw supply.

**Events**: `tool:start` -> `tool:update` (simulating) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (HF guard + oracle validation), `tool_result` (position state logged)

**Ground truth**: Verify `borrowed` against borrow shares increase. Source: on-chain receipt.

### `morpho_repay_market`

Repay borrowed loan token in a Morpho Blue market.

```rust
#[derive(Debug, Deserialize)]
pub struct MorphoRepayParams {
    pub market_id: String,
    /// Amount to repay, or "max".
    pub amount: String,
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MorphoRepayResult {
    pub tx_hash: String,
    pub repaid: String,
    pub remaining_debt: String,
    pub shares_repaid: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Approving token", "Signing repay", "Confirming"]`
**sprite_trigger**: `Executing`

**Ground truth**: Verify `repaid` against borrow shares decrease. Source: on-chain receipt.

### `morpho_get_market_info`

List Morpho Blue markets, optionally filtered by asset or minimum TVL.

```rust
#[derive(Debug, Deserialize)]
pub struct MorphoGetMarketsParams {
    pub chain: u64,
    #[serde(default)]
    pub asset: Option<String>,
    #[serde(default)]
    pub min_tvl: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct MorphoMarketInfo {
    pub market_id: String,
    pub loan_token: String,
    pub collateral_token: String,
    pub oracle: String,
    pub irm: String,
    pub lltv: f64,
    pub supply_apy: f64,
    pub borrow_apy: f64,
    pub tvl: f64,
    pub utilization: f64,
    pub available_liquidity: String,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching markets"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "List Morpho Blue markets with rates, TVL, utilization. Filter by asset or minimum TVL."

**Events**: `tool:start` -> `tool:end`

### `morpho_get_user_position`

Get all Morpho Blue positions for an account.

```rust
#[derive(Debug, Deserialize)]
pub struct MorphoGetPositionsParams {
    pub account: String,
    pub chain: u64,
    #[serde(default)]
    pub market_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MorphoUserPosition {
    pub account: String,
    pub markets: Vec<MorphoMarketPosition>,
}

#[derive(Debug, Serialize)]
pub struct MorphoMarketPosition {
    pub market_id: String,
    pub loan_token: String,
    pub collateral_token: String,
    pub supply_shares: String,
    pub supply_assets: String,
    pub borrow_shares: String,
    pub borrow_assets: String,
    pub collateral: String,
    pub ltv: f64,
    pub liquidation_ltv: f64,
    pub health_factor: f64,
    pub is_liquidatable: bool,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching positions"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Account's Morpho Blue positions: supply, borrow, collateral, health factor, liquidation status."

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (liquidatable position triggers alert)

### `morpho_liquidate`

Liquidate an unhealthy Morpho Blue position.

```rust
#[derive(Debug, Deserialize)]
pub struct MorphoLiquidateParams {
    /// Market ID.
    pub market_id: String,
    /// Address of the borrower to liquidate.
    pub borrower: String,
    /// Amount of collateral to seize.
    pub seized_assets: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MorphoLiquidateResult {
    pub tx_hash: String,
    pub collateral_seized: String,
    pub debt_repaid: String,
    pub profit_usd: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking position health", "Simulating liquidation", "Signing tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Liquidate an unhealthy Morpho Blue position. Only callable when borrower's LTV exceeds the market's LLTV."

**promptGuidelines**:
- **thriving**: Liquidate for profit when opportunities arise.
- **cautious**: Only liquidate if profit covers gas + risk.
- **declining**: Do not liquidate. Focus on own positions.
- **terminal**: Do not liquidate.

**Events**: `tool:start` -> `tool:update` (simulating) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`

**Ground truth**: Verify `collateral_seized` against token balance increase. Source: on-chain receipt + balance diff.

**Error codes**: `MORPHO_MARKET_NOT_FOUND`, `MORPHO_INSUFFICIENT_LIQUIDITY`, `MORPHO_BORROW_CAP_EXCEEDED`, `MORPHO_POSITION_HEALTHY`

---

## MakerDAO / Sky CDPs (4 tools)

MakerDAO vaults are CDPs backed by ETH, WBTC, and other collateral. Sky is the rebranded MakerDAO protocol.

### `maker_open_vault`

Open a new MakerDAO vault for a given collateral type.

```rust
#[derive(Debug, Deserialize)]
pub struct MakerOpenVaultParams {
    /// Collateral type identifier (e.g. "ETH-A", "WBTC-A").
    pub collateral_type: String,
    /// Chain ID (1 for Ethereum mainnet).
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MakerOpenVaultResult {
    pub tx_hash: String,
    pub vault_id: u64,
    pub collateral_type: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Creating vault", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Open a new MakerDAO vault for a collateral type (ETH-A, WBTC-A). One-time operation per vault."

**Ground truth**: Verify vault creation via CDP Manager `last(account)`. Source: on-chain state read.

### `maker_deposit_collateral`

Deposit collateral to an existing MakerDAO vault.

```rust
#[derive(Debug, Deserialize)]
pub struct MakerDepositParams {
    /// Vault ID.
    pub vault_id: u64,
    /// Collateral amount to deposit.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MakerDepositResult {
    pub tx_hash: String,
    pub collateral_deposited: String,
    pub total_collateral: String,
    pub collateralization_ratio: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Approving collateral", "Signing deposit", "Confirming"]`
**sprite_trigger**: `Executing`

**Ground truth**: Verify collateral increase via vault state read. Source: on-chain Vat query.

### `maker_generate_dai`

Generate DAI against vault collateral.

```rust
#[derive(Debug, Deserialize)]
pub struct MakerGenerateDaiParams {
    /// Vault ID.
    pub vault_id: u64,
    /// DAI amount to generate.
    pub dai_amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MakerGenerateDaiResult {
    pub tx_hash: String,
    pub dai_generated: String,
    pub total_debt: String,
    pub collateralization_ratio: f64,
    pub liquidation_price: f64,
    pub stability_fee: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer3` | **tick_budget**: `Medium`
**progress_steps**: `["Checking debt ceiling", "Signing generation tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Mint DAI against vault collateral. Do not mint if collateralization ratio would drop below 200%."

**promptGuidelines**:
- **thriving**: Generate DAI conservatively. Keep collateralization ratio > 250%.
- **cautious**: Do not generate new DAI.
- **declining**: Repay DAI to reduce liquidation risk.
- **terminal**: Repay all DAI and close vault.

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (collateralization ratio guard), `tool_result` (debt + liquidation price logged)

**Ground truth**: Verify DAI balance increase against vault debt increase. Source: on-chain Vat + DAI balance.

### `maker_close_vault`

Repay all debt and withdraw all collateral from a MakerDAO vault.

```rust
#[derive(Debug, Deserialize)]
pub struct MakerCloseVaultParams {
    /// Vault ID.
    pub vault_id: u64,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct MakerCloseVaultResult {
    pub tx_hash: String,
    pub dai_repaid: String,
    pub collateral_withdrawn: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Slow`
**progress_steps**: `["Approving DAI", "Repaying debt", "Withdrawing collateral", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Close a MakerDAO vault: repay all debt and withdraw all collateral in a single operation."

**Events**: `tool:start` -> `tool:update` (repaying) -> `tool:update` (withdrawing) -> `tool:update` (confirming) -> `tool:end`

**Ground truth**: Verify vault debt is zero and collateral is zero. Source: on-chain Vat query.

**Error codes**: `MAKER_VAULT_NOT_FOUND`, `COLLATERALIZATION_RATIO_TOO_LOW`, `DAI_DEBT_CEILING_REACHED`

---

## Leveraged looping (4 tools)

Leveraged loops amplify yield by recursively supplying and borrowing. A standard ETH loop supplies WETH, borrows USDC, swaps back to WETH, and supplies again until the target leverage is reached. The Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) monitors health factors on every heartbeat (the Golem's periodic 9-step decision cycle) tick and auto-deleverages when health factor drops below a configured threshold.

All loop execution tools have `destructive_hint: true`. A Golem in conservation phase or below must not initiate new loops. `lending_auto_deleverage` runs at any phase.

### `lending_build_loop`

Preview a leverage loop without executing. Returns the full multi-step plan, projected health factor, and estimated APY. Read-only.

```rust
#[derive(Debug, Deserialize)]
pub struct BuildLoopParams {
    /// Asset to supply as collateral (e.g. "WETH", "wstETH").
    pub collateral_asset: String,
    /// Asset to borrow (e.g. "USDC", "WETH").
    pub borrow_asset: String,
    /// Starting capital amount.
    pub initial_amount: String,
    /// Target leverage multiplier (e.g. 3.0 for 3x).
    pub target_leverage: f64,
    /// Lending protocol to use.
    pub protocol: String,  // "aave" | "morpho"
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LoopPreview {
    pub steps: Vec<LoopStep>,
    pub initial_capital: String,
    pub final_collateral: String,
    pub final_debt: String,
    pub health_factor_after: f64,
    pub leverage_achieved: f64,
    pub estimated_apy: f64,        // supplyAPY * leverage - borrowAPY * (leverage - 1)
    pub break_even_apy: f64,       // minimum collateral APY to break even on borrow cost
    pub gas_estimate: String,
    pub loop_count: u32,
    pub max_safe_loops: u32,
}

#[derive(Debug, Serialize)]
pub struct LoopStep {
    pub action: String,  // "supply" | "borrow" | "swap"
    pub asset: String,
    pub amount: String,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Computing loop plan", "Estimating APY"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Preview a leverage loop without executing. Returns multi-step plan, projected health factor, estimated APY, and break-even APY. ALWAYS call before `lending_execute_leverage_loop`."

**promptGuidelines**:
- **thriving**: Run to evaluate leverage opportunities. Require projected HF > 1.5 and APY > break-even by > 2%.
- **cautious**: Run but do not execute loops.
- **declining**: Do not build new loops.
- **terminal**: Do not build new loops.

**Events**: `tool:start` -> `tool:end`

### `lending_unwind_loop`

Unwind an active leverage loop partially or fully.

```rust
#[derive(Debug, Deserialize)]
pub struct UnwindLoopParams {
    /// Account address.
    pub account: String,
    /// Lending protocol.
    pub protocol: String,  // "aave" | "morpho"
    /// Chain ID.
    pub chain: u64,
    /// 1-100, percentage of leverage to remove.
    pub percent_to_unwind: u32,
}

#[derive(Debug, Serialize)]
pub struct UnwindLoopResult {
    pub tx_hash: String,
    pub assets_recovered: String,
    pub debt_repaid: String,
    pub health_factor_after: f64,
    pub leverage_after: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Slow`
**progress_steps**: `["Flash loan", "Repaying debt", "Withdrawing collateral", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Unwind an active leverage loop partially or fully. Reduces leverage exposure. Safe action in all phases."

**promptGuidelines**:
- **thriving**: Unwind when strategy no longer supports the leverage.
- **cautious**: Unwind all loops to zero leverage.
- **declining**: Unwind all loops immediately.
- **terminal**: Full unwind of all loops. Priority action.

**Events**: `tool:start` -> `tool:update` (flash loan) -> `tool:update` (repaying) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (slippage guard on swap legs), `tool_result` (final leverage + HF logged)

**Ground truth**: Verify leverage ratio decreased via position reads. Source: on-chain health factor + collateral/debt balances.

### `lending_get_loop_health`

Get current health metrics for an active leverage loop position.

```rust
#[derive(Debug, Deserialize)]
pub struct GetLoopHealthParams {
    pub account: String,
    pub protocol: String,  // "aave" | "morpho"
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LoopHealth {
    pub health_factor: f64,
    pub total_collateral_usd: f64,
    pub total_debt_usd: f64,
    pub leverage_ratio: f64,
    pub liquidation_price: LiquidationPrice,
    pub time_to_liquidation: Option<String>,
    pub net_apy: f64,
    pub risk_level: String,            // "safe" | "warning" | "danger"
    pub recommended_action: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LiquidationPrice {
    pub asset: String,
    pub price_usd: f64,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching loop state"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Loop health with risk classification and recommended action. Called on every heartbeat tick. When risk_level is 'danger' (HF < 1.2), heartbeat triggers `lending_auto_deleverage` automatically."

**promptGuidelines**:
- **thriving**: Monitor on every heartbeat. Act on recommendations.
- **cautious**: Monitor. Trigger unwind if HF < 1.5.
- **declining**: Monitor. Trigger full unwind.
- **terminal**: Monitor until fully unwound.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (danger triggers automatic `lending_auto_deleverage`)

### `lending_auto_deleverage`

Partially unwind a leverage loop to restore health factor above a target. Designed for autonomous execution -- no human input required.

```rust
#[derive(Debug, Deserialize)]
pub struct AutoDeleverageParams {
    /// Account address.
    pub account: String,
    /// Target HF after deleveraging (e.g. 1.8).
    pub target_health_factor: f64,
    /// Lending protocol.
    pub protocol: String,  // "aave" | "morpho"
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AutoDeleverageResult {
    pub tx_hash: String,
    pub leverage_before: f64,
    pub leverage_after: f64,
    pub health_factor_after: f64,
    pub assets_used: String,
    pub debt_repaid: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Slow`
**progress_steps**: `["Calculating unwind amount", "Executing deleverage", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Autonomous partial unwind to restore health factor. Runs at ANY phase without restriction. No human input required."

**promptGuidelines**:
- **thriving**: Automatic when HF < 1.2. Target HF 1.8 after deleverage.
- **cautious**: Same automatic behavior. Target HF 2.0.
- **declining**: Same. Target HF 2.0.
- **terminal**: Deleverage to zero leverage (full unwind).
- **override**: This tool bypasses normal phase restrictions. Preventing liquidation is more important than phase constraints.

**Events**: `tool:start` -> `tool:update` (calculating) -> `tool:update` (executing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (no phase gate -- always allowed), `tool_result` (deleverage logged as emergency episode in Grimoire)

**Ground truth**: Verify health factor increased above target. Source: on-chain health factor read after tx confirmation.

**Error codes**: `LOOP_HEALTH_FACTOR_BREACH`, `FLASH_LOAN_FAILED`, `SLIPPAGE_EXCEEDED_DURING_LOOP`, `LEVERAGE_TOO_HIGH`

---

## Custody implications (lending write tools)

All lending write tools share the same custody behavior:

- **Delegation**: Session key signs via the lending protocol's router or pool contract. CaveatEnforcer must whitelist each protocol's contract addresses (Aave Pool, Morpho Blue Singleton, MakerDAO Proxy). Spending limits apply to deposit amounts; borrow operations are bounded by on-chain collateral ratios.
- **Embedded**: Privy server wallet signs. PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) validates protocol target, operation type, and position size limits.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed protocol targets and amounts.

Flash loan tools (`lending_flash_loan`) require the receiver contract to be on the address allowlist. The flash loan is atomic, so the custody layer only validates the initial call.

---

## Health factor normalization

Different lending protocols report health factors differently. Bardo normalizes all of them to a 0.0-1.0 scale for consistent risk assessment across the lending dashboard and auto-deleverage triggers.

| Protocol | Raw HF Range | Bardo Normalized | Notes |
|----------|-------------|-----------------|-------|
| Aave v3  | 0-infinity  | 0.0-1.0         | HF=1.0 -> normalized 0.0 (at liquidation) |
| Compound v3 | 0-infinity | 0.0-1.0       | Uses borrow capacity ratio |
| Morpho   | 0-infinity  | 0.0-1.0         | Per-market, not per-account |

The normalization formula: `normalized = 1.0 - (1.0 / raw_hf)`. A raw HF of 2.0 becomes 0.5 (50% buffer before liquidation). A raw HF of 1.0 becomes 0.0 (at liquidation). This makes the normalized value directly interpretable as "distance from liquidation as a fraction of collateral value."

All lending tools in the cross-protocol utilities section (`lending_get_health_dashboard`, `lending_compare_rates`) use the normalized scale. Per-protocol tools (`aave_get_health_factor`, `morpho_get_user_position`) return the raw protocol-native value in `health_factor` and the normalized value in a separate `normalized_health` field when called through the Golem adapter layer.

---

## Cross-protocol utilities (3 tools)

### `lending_compare_rates`

Compare supply or borrow rates across all integrated lending protocols for an asset.

```rust
#[derive(Debug, Deserialize)]
pub struct CompareRatesParams {
    /// Token address or symbol.
    pub asset: String,
    /// Chain ID.
    pub chain: u64,
    /// Rate type to compare.
    pub rate_type: String,  // "supply" | "borrow"
}

#[derive(Debug, Serialize)]
pub struct CompareRatesResult {
    pub asset: String,
    pub chain: u64,
    pub rate_type: String,
    pub rates: Vec<ProtocolRate>,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
pub struct ProtocolRate {
    pub protocol: String,      // "aave_v3" | "morpho_blue"
    pub market: Option<String>,
    pub apy: f64,
    pub tvl: f64,
    pub utilization: f64,
    pub available_liquidity: String,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Fetching Aave rates", "Fetching Morpho rates", "Comparing"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Compare supply or borrow rates across Aave V3 and Morpho Blue for a given asset. Returns a recommendation for the best venue."

**Events**: `tool:start` -> `tool:update` (fetching) -> `tool:end`

### `lending_get_health_dashboard`

Get all lending positions across all integrated protocols for an account in a single call.

```rust
#[derive(Debug, Deserialize)]
pub struct HealthDashboardParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LendingHealthDashboard {
    pub account: String,
    pub total_supplied_usd: f64,
    pub total_borrowed_usd: f64,
    pub net_position_usd: f64,
    pub weighted_supply_apy: f64,
    pub weighted_borrow_apy: f64,
    pub net_apy: f64,
    pub positions: Vec<ProtocolPosition>,
    pub overall_health_factor: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ProtocolPosition {
    pub protocol: String,
    pub market: Option<String>,
    pub supplied: Vec<AssetPosition>,
    pub borrowed: Vec<AssetPosition>,
    pub health_factor: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct AssetPosition {
    pub asset: String,
    pub amount_usd: f64,
    pub apy: f64,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Fetching Aave positions", "Fetching Morpho positions", "Fetching Maker vaults", "Aggregating"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "All lending positions across Aave, Morpho, and Maker for an account. Returns total supplied, borrowed, net APY, and per-protocol health factors."

**Events**: `tool:start` -> `tool:update` (fetching) -> `tool:end`

### `lending_flash_loan`

Execute a flash loan for atomic multi-step operations (arbitrage, liquidation, collateral swaps).

```rust
#[derive(Debug, Deserialize)]
pub struct FlashLoanParams {
    /// Token to flash borrow.
    pub asset: String,
    /// Amount to borrow.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
    /// Encoded calldata for the flash loan receiver contract.
    pub receiver_calldata: String,
    /// Flash loan provider preference.
    #[serde(default = "default_flash_provider")]
    pub provider: String,  // "aave" | "morpho"
}

fn default_flash_provider() -> String { "aave".into() }

#[derive(Debug, Serialize)]
pub struct FlashLoanResult {
    pub tx_hash: String,
    pub amount_borrowed: String,
    pub fee_paid: String,
    pub profit_usd: Option<f64>,
}
```

**capability**: `Write` | **risk_tier**: `Layer3` | **tick_budget**: `Slow`
**progress_steps**: `["Validating receiver", "Simulating flash loan", "Signing tx", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Execute a flash loan. Atomic: the loan must be repaid in the same transaction or the entire operation reverts. Used internally by leverage loop and liquidation tools."

**promptGuidelines**:
- **thriving**: Use for profitable arbitrage and loop construction.
- **cautious**: Use only for deleveraging.
- **declining**: Use only for emergency deleveraging.
- **terminal**: Use only for emergency deleveraging.

**Events**: `tool:start` -> `tool:update` (simulating) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (receiver validation + simulation), `tool_result` (fee paid logged)

**Ground truth**: Flash loan is atomic -- if tx confirms, the loan was repaid. Source: transaction receipt status.

**Error codes**: `FLASH_LOAN_FAILED`, `FLASH_LOAN_NOT_REPAID`, `RECEIVER_NOT_WHITELISTED`
