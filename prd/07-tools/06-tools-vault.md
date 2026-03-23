# Bardo Tools -- Vault tools [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` (category `vault`) | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles), [../08-vault/](../08-vault/) (the Bardo vault protocol specification covering ERC-4626 architecture, am-AMM integration, and strategy framework)
>
> ERC-4626 vault operations, am-AMM bidding, proxy management, executor framework, and strategy auctions. 42 tools total. All tools use the `vault_` prefix.

---

> **Reader orientation:** This document specifies the 42 vault tools in `bardo-tools`. These cover ERC-4626 vault lifecycle (create, deposit, withdraw, rebalance), am-AMM bidding for Bunni v2 pools, proxy/delegate management, executor frameworks, and strategy auctions. You should be familiar with ERC-4626, vault share accounting, and the am-AMM mechanism. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Core vault tools (20)

### Vault lifecycle

#### `vault_create`

Deploy a new ERC-4626 vault with configurable fees, identity gating, and asset selection. One-time operation per vault. The factory deploys an EIP-1167 minimal proxy clone.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultCreateParams {
    /// Underlying asset address (ERC-20).
    pub asset: String,
    /// Vault name.
    pub name: String,
    /// Vault symbol.
    pub symbol: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Management fee in basis points (max 500).
    #[serde(default)]
    pub management_fee_bps: u16,
    /// Performance fee in basis points (max 5000).
    #[serde(default)]
    pub performance_fee_bps: u16,
    /// Require ERC-8004 (on-chain standard for registering and verifying autonomous agent identities) identity for deposits. Default: false.
    #[serde(default)]
    pub identity_gated: bool,
}

#[derive(Debug, Serialize)]
pub struct VaultCreateResult {
    pub tx_hash: String,
    pub vault_address: String,
    pub asset: String,
    pub owner: String,
    pub management_fee_bps: u16,
    pub performance_fee_bps: u16,
    pub identity_gated: bool,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Validating fee parameters", "Simulating deployment", "Submitting factory call", "Confirming on-chain", "Verifying vault initialization"]`

**Ground truth**: After tx, call `vault.asset()`, `vault.owner()`, `vault.managementFeeBps()`, `vault.performanceFeeBps()`, and `vault.identityGated()`. All must match the creation parameters.

**promptSnippet**:
> Deploy a new ERC-4626 vault with configurable fees, identity gating, and asset. One-time operation per vault. Gas-intensive. Verify fee parameters are within caps (500 bps mgmt, 5000 bps perf) before calling.

**promptGuidelines**:
- **thriving**: Create vaults when the owner requests. Verify fee structure, identity gating, and asset choice before deploying.
- **cautious**: Only create if the owner explicitly requests. Do not autonomously spin up new vaults.
- **declining**: Do not create new vaults. Focus on managing existing ones.
- **terminal**: Never create.

---

#### `vault_list`

List vaults by owner, asset, or chain.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultListParams {
    pub chain_id: u64,
    /// Filter by vault owner address.
    pub owner: Option<String>,
    /// Filter by underlying asset address or symbol.
    pub asset: Option<String>,
    /// Max results. Default: 50.
    #[serde(default = "default_50")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct VaultListResult {
    pub vaults: Vec<VaultSummary>,
}

#[derive(Debug, Serialize)]
pub struct VaultSummary {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub asset: String,
    pub owner: String,
    pub tvl_usd: f64,
    pub share_price: f64,
    pub management_fee_bps: u16,
    pub performance_fee_bps: u16,
    pub identity_gated: bool,
    pub paused: bool,
}
```

**Progress steps**: `["Querying vault factory events", "Loading vault states"]`

**promptSnippet**:
> List vaults by owner, asset, or chain. Read-only discovery tool. Use to find vaults before deposit or management operations.

**promptGuidelines**:
- **thriving**: Use freely for vault discovery and comparison.
- **cautious**: Use to audit all managed vaults. Check for any with declining TVL.
- **declining**: Use to inventory managed vaults for wind-down planning.
- **terminal**: Use once for final inventory.

---

#### `vault_get_info`

Get vault state: TVL, share price, fees, owner, identity gating status.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultGetInfoParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VaultInfo {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub asset: String,
    pub asset_decimals: u8,
    pub share_decimals: u8,
    pub owner: String,
    pub tvl_usd: f64,
    pub total_assets: String,
    pub total_supply: String,
    pub share_price: f64,
    pub management_fee_bps: u16,
    pub performance_fee_bps: u16,
    pub identity_gated: bool,
    pub paused: bool,
}
```

**Progress steps**: `["Reading vault contract state", "Computing share price"]`

**promptSnippet**:
> Get vault state: TVL, share price, fees, owner, identity gating status. Call before any vault interaction to verify current state.

**promptGuidelines**:
- **thriving**: Check on every heartbeat for managed vaults. Monitor share price trajectory.
- **cautious**: Check before every operation. Verify vault is not paused.
- **declining**: Monitor all vaults for depositor withdrawal pressure.
- **terminal**: Final state check before emergency operations.

---

#### `vault_get_history`

Get historical vault performance, deposits, withdrawals, and fee events.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultGetHistoryParams {
    pub vault: String,
    pub chain_id: u64,
    /// Start time (ISO 8601). Default: 30 days ago.
    pub since: Option<String>,
    /// Max events. Default: 100.
    #[serde(default = "default_100")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct VaultHistory {
    pub events: Vec<VaultEvent>,
    pub performance: VaultPerformanceSummary,
}

#[derive(Debug, Serialize)]
pub struct VaultEvent {
    pub event_type: String,
    pub timestamp: String,
    pub tx_hash: String,
    pub actor: String,
    pub details: serde_json::Value,
}
```

**Progress steps**: `["Querying vault events", "Computing performance metrics"]`

**promptSnippet**:
> Get historical vault performance, deposits, withdrawals, fee events. Use for performance reporting and trend analysis.

---

### Participant operations

#### `vault_deposit`

Deposit assets into a vault, receive shares. Respects identity gating.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultDepositParams {
    pub vault: String,
    pub chain_id: u64,
    /// Asset amount to deposit (human-readable).
    pub amount: String,
    /// ERC-8004 agent ID (required if vault is identity-gated).
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VaultDepositResult {
    pub tx_hash: String,
    pub shares_received: String,
    pub assets_deposited: String,
    pub fee_deducted: String,
    pub share_price_at_deposit: f64,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Previewing deposit", "Checking identity gate", "Approving asset", "Submitting deposit", "Confirming on-chain"]`

**Ground truth**: Read vault share balance post-tx. Verify shares received matches `previewDeposit(amount)` minus rounding.

**promptSnippet**:
> Deposit assets into a vault, receive shares. Respects identity gating. Call `vault_preview_deposit` first to check expected shares and fee deductions. In declining phase, no new deposits.

**promptGuidelines**:
- **thriving**: Deposit according to strategy allocation. Verify share price is reasonable. Check identity gating requirements.
- **cautious**: Maximum 10% of NAV per vault. Require vault health score > 70 and positive share price trend.
- **declining**: Do not make new deposits. Only maintain existing positions.
- **terminal**: Never deposit. Withdraw from all vaults.

---

#### `vault_withdraw`

Withdraw assets from a vault by redeeming shares.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultWithdrawParams {
    pub vault: String,
    pub chain_id: u64,
    /// Shares to redeem (human-readable), or "max" for full position.
    pub shares: String,
}

#[derive(Debug, Serialize)]
pub struct VaultWithdrawResult {
    pub tx_hash: String,
    pub assets_received: String,
    pub shares_redeemed: String,
    pub fee_deducted: String,
    pub share_price_at_withdraw: f64,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Previewing withdrawal", "Submitting redemption", "Confirming on-chain"]`

**Ground truth**: Read vault share balance post-tx. For "max", verify zero balance. Read asset balance increase and verify matches `previewRedeem(shares)` minus rounding.

**promptSnippet**:
> Withdraw assets from a vault by redeeming shares. In terminal phase, withdraw everything from all vaults.

**promptGuidelines**:
- **thriving**: Withdraw only when rebalancing across vaults or strategy dictates.
- **cautious**: Withdraw if vault share price has declined >5% from entry. Partial withdrawal (50%) to reduce exposure.
- **declining**: Withdraw from all vaults. Start with worst-performing.
- **terminal**: Withdraw 100% from all vaults.

---

#### `vault_preview`

Preview a deposit or withdrawal: expected shares/assets, fee deductions, identity check result. Replaces the separate `vault_preview_deposit` and `vault_simulate_deposit` tools from v1.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultPreviewParams {
    pub vault: String,
    pub chain_id: u64,
    /// "deposit" or "withdraw".
    pub action: String,
    /// For deposit: asset amount. For withdraw: share amount (or "max").
    pub amount: String,
    /// ERC-8004 agent ID (checked for identity-gated deposit previews).
    pub agent_id: Option<String>,
    /// Simulate under different market conditions. Optional.
    pub scenario: Option<MarketScenario>,
}

#[derive(Debug, Deserialize)]
pub struct MarketScenario {
    /// Price change percentage (e.g., -20.0 for crash scenario).
    pub price_change_pct: f64,
    /// Label for this scenario.
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct VaultPreviewResult {
    pub action: String,
    /// For deposit: expected shares. For withdraw: expected assets.
    pub expected_output: String,
    pub fee_deduction: String,
    pub share_price: f64,
    /// Only for identity-gated vaults.
    pub identity_check: Option<IdentityCheckResult>,
    /// Scenario simulation results (if scenario was provided).
    pub scenario_result: Option<ScenarioResult>,
}

#[derive(Debug, Serialize)]
pub struct IdentityCheckResult {
    pub eligible: bool,
    pub agent_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScenarioResult {
    pub label: String,
    pub projected_value: String,
    pub projected_pnl_pct: f64,
}
```

**Progress steps**: `["Reading vault state", "Computing preview"]`

**promptSnippet**:
> Preview deposit or withdrawal: expected shares/assets, fee deductions, identity check. Read-only. ALWAYS call before `vault_deposit` or `vault_withdraw` to verify terms. Set `scenario` to stress-test under crash conditions.

**promptGuidelines**:
- **thriving**: Call before every deposit. Compare expected shares across multiple vaults. Use scenario for deposits > $10K.
- **cautious**: Always call. Verify fee deduction is within expected range. Run crash-20% scenario.
- **declining**: Call before each exit to plan capital recovery.
- **terminal**: Call once per vault, then execute withdrawal immediately.

---

#### `vault_get_position`

Get a depositor's position: shares held, current value, unrealized P&L.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultGetPositionParams {
    pub vault: String,
    pub chain_id: u64,
    /// Depositor wallet address. Default: caller's address.
    pub account: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VaultPosition {
    pub vault: String,
    pub account: String,
    pub shares: String,
    pub current_value_usd: f64,
    pub cost_basis_usd: f64,
    pub unrealized_pnl_usd: f64,
    pub unrealized_pnl_pct: f64,
    pub share_price_at_entry: f64,
    pub current_share_price: f64,
}
```

**Progress steps**: `["Reading share balance", "Computing P&L"]`

**promptSnippet**:
> Get a depositor's position: shares held, current value, unrealized P&L. Use for portfolio tracking and exit planning.

---

### Manager operations

#### `vault_rebalance`

Rebalance vault allocations across strategies. When Warden is deployed (optional, deferred; see `prd2-extended/10-safety/02-warden.md`), passes through announce-wait-execute.

**Capability**: `Write` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultRebalanceParams {
    pub vault: String,
    pub chain_id: u64,
    /// Target allocations: [{ adapter, weight_bps }].
    pub target_allocations: Vec<AllocationTarget>,
}

#[derive(Debug, Deserialize)]
pub struct AllocationTarget {
    pub adapter: String,
    pub weight_bps: u16,
}

#[derive(Debug, Serialize)]
pub struct VaultRebalanceResult {
    pub tx_hash: String,
    pub old_allocations: Vec<AllocationSnapshot>,
    pub new_allocations: Vec<AllocationSnapshot>,
    pub gas_used: u64,
    pub explorer_url: String,
}

#[derive(Debug, Serialize)]
pub struct AllocationSnapshot {
    pub adapter: String,
    pub weight_bps: u16,
    pub value_usd: f64,
}
```

**Progress steps**: `["Computing allocation delta", "Announcing via Warden (if deployed)", "Waiting for delay", "Executing rebalance", "Confirming on-chain"]`

**Ground truth**: After tx, read each adapter's `totalValue()`. Verify weights match `target_allocations` within 50 bps tolerance (rounding from integer arithmetic).

**promptSnippet**:
> Rebalance vault allocations across strategies. When Warden is deployed (optional, deferred), passes through announce-wait-execute. Call when strategy parameters drift from target allocations.

**promptGuidelines**:
- **thriving**: Rebalance when allocation drift exceeds 5% from target. Use `vault_get_allocations` to check drift.
- **cautious**: Rebalance when drift exceeds 3%. Smaller increments, more frequent checks.
- **declining**: Only rebalance to de-risk (shift to stables/low-risk allocations).
- **terminal**: Rebalance everything to the vault's base asset for maximum liquidity.

---

#### `vault_collect_fees`

Collect accrued management and performance fees.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultCollectFeesParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VaultCollectFeesResult {
    pub tx_hash: String,
    pub management_fee_collected: String,
    pub performance_fee_collected: String,
    pub total_fee_usd: f64,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Computing accrued fees", "Submitting collection", "Confirming on-chain"]`

**Ground truth**: Compare fee recipient balance pre-tx and post-tx. Increase should match reported fees.

**promptSnippet**:
> Collect accrued management and performance fees. As vault manager/owner, call periodically to realize earned fees.

**promptGuidelines**:
- **thriving**: Collect monthly. Fees are revenue for the Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM).
- **cautious**: Collect weekly. Uncollected fees are at risk in the vault contract.
- **declining**: Collect immediately. Every bit of revenue extends runway.
- **terminal**: Collect all remaining fees as part of wind-down.

---

#### `vault_update_strategy`

Update vault strategy parameters within PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) bounds. Passes through Warden (requires optional Warden module, deferred).

**Capability**: `Write` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultUpdateStrategyParams {
    pub vault: String,
    pub chain_id: u64,
    /// Strategy parameter updates as key-value pairs.
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct VaultUpdateStrategyResult {
    pub tx_hash: String,
    pub old_params: serde_json::Value,
    pub new_params: serde_json::Value,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Validating against PolicyCage", "Announcing via Warden (if deployed)", "Waiting for delay", "Executing update", "Confirming on-chain"]`

**Ground truth**: Read strategy parameters from vault contract post-tx. Verify all updated fields match.

**promptSnippet**:
> Update vault strategy parameters within PolicyCage bounds. Cannot exceed bounds set at vault creation.

**promptGuidelines**:
- **thriving**: Adjust parameters based on Grimoire (the Golem's persistent memory system with episodic, semantic, and strategy layers) insights and market regime changes.
- **cautious**: Only tighten parameters (reduce risk). Do not loosen constraints.
- **declining**: Shift strategy toward capital preservation.
- **terminal**: Set strategy to most conservative possible.

---

#### `vault_report`

Submit a strategy report. Triggers fee accrual and profit accounting.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultReportParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VaultReportResult {
    pub tx_hash: String,
    pub total_assets: String,
    pub profit_since_last_report: String,
    pub fees_accrued: String,
    pub report_hash: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Computing total assets", "Submitting report", "Confirming on-chain"]`

**Ground truth**: Verify report event emitted with correct total assets. Verify fee accrual matches expected calculation.

**promptSnippet**:
> Submit a strategy report to the vault. Triggers fee accrual and profit accounting. Required for performance fee calculation.

**promptGuidelines**:
- **thriving**: Submit reports at least weekly.
- **cautious**: Submit after every significant operation.
- **declining**: Submit final report documenting performance decline.
- **terminal**: Submit final report before relinquishing management.

---

#### `vault_get_strategy`

Get current strategy configuration and allocation breakdown.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultGetStrategyParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VaultStrategy {
    pub vault: String,
    pub strategy_params: serde_json::Value,
    pub allocations: Vec<AllocationSnapshot>,
    pub policy_cage_bounds: serde_json::Value,
    pub last_rebalance: String,
    pub last_report: String,
}
```

**Progress steps**: `["Reading strategy state", "Loading allocation breakdown"]`

**promptSnippet**:
> Get current strategy configuration and allocation breakdown. Read-only. Call before `vault_rebalance` to understand current state.

---

### Analytics

#### `vault_get_performance`

Get vault performance metrics: APY, Sharpe, drawdown, benchmark comparison.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultGetPerformanceParams {
    pub vault: String,
    pub chain_id: u64,
    /// Period: "7d", "30d", "90d", "365d". Default: "30d".
    #[serde(default = "default_30d")]
    pub period: String,
}

#[derive(Debug, Serialize)]
pub struct VaultPerformance {
    pub vault: String,
    pub period: String,
    pub apy: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
    pub vs_eth_pct: f64,
    pub vs_hodl_pct: f64,
    pub total_fees_earned_usd: f64,
}
```

**Progress steps**: `["Loading share price history", "Computing risk metrics"]`

**promptSnippet**:
> Get vault performance: APY, Sharpe, drawdown, benchmark comparison. Use for strategy evaluation and owner reporting.

---

#### `vault_get_allocations`

Get current asset allocation breakdown by strategy adapter.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultGetAllocationsParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VaultAllocations {
    pub vault: String,
    pub total_assets_usd: f64,
    pub allocations: Vec<AllocationSnapshot>,
    pub drift_from_target: Vec<AllocationDrift>,
}

#[derive(Debug, Serialize)]
pub struct AllocationDrift {
    pub adapter: String,
    pub target_weight_bps: u16,
    pub actual_weight_bps: u16,
    pub drift_bps: i16,
}
```

**Progress steps**: `["Reading adapter balances", "Computing drift"]`

**promptSnippet**:
> Get current asset allocation breakdown by strategy adapter. Call before `vault_rebalance` to assess allocation drift.

---

#### `vault_compare`

Compare multiple vaults by performance, risk, and fee metrics.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultCompareParams {
    pub vaults: Vec<String>,
    pub chain_id: u64,
    /// Period. Default: "30d".
    #[serde(default = "default_30d")]
    pub period: String,
}

#[derive(Debug, Serialize)]
pub struct VaultComparison {
    pub vaults: Vec<VaultCompareEntry>,
}

#[derive(Debug, Serialize)]
pub struct VaultCompareEntry {
    pub address: String,
    pub name: String,
    pub apy: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_pct: f64,
    pub tvl_usd: f64,
    pub management_fee_bps: u16,
    pub performance_fee_bps: u16,
    pub identity_gated: bool,
}
```

**Progress steps**: `["Loading vault states", "Computing performance metrics", "Ranking vaults"]`

**promptSnippet**:
> Compare multiple vaults by performance, risk, and fees. Use when choosing where to deposit or when evaluating competitive landscape.

---

#### `vault_get_share_price_history`

Get historical share price for yield curve visualization.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultSharePriceHistoryParams {
    pub vault: String,
    pub chain_id: u64,
    /// Period. Default: "90d".
    #[serde(default = "default_90d")]
    pub period: String,
    /// Data point interval: "1h", "1d", "1w". Default: "1d".
    #[serde(default = "default_1d")]
    pub interval: String,
}

#[derive(Debug, Serialize)]
pub struct SharePriceHistory {
    pub vault: String,
    pub data_points: Vec<SharePricePoint>,
}

#[derive(Debug, Serialize)]
pub struct SharePricePoint {
    pub timestamp: String,
    pub share_price: f64,
    pub total_assets: String,
    pub total_supply: String,
}
```

**Progress steps**: `["Querying historical events", "Reconstructing share price curve"]`

**promptSnippet**:
> Get historical share price for yield curve visualization. Use to assess vault performance trend and stability.

---

## Proxy (Warden) tools (6) -- Optional, Deferred

> The Warden time-delay proxy is optional and deferred. See `prd2-extended/10-safety/02-warden.md` for the full specification. These tools require the optional Warden module to be deployed.

Time-delayed execution for write operations. When deployed, the Warden enforces announce-wait-execute with configurable delays based on risk tier.

### Risk tiers

| Tier | Delay | Required for |
| --- | --- | --- |
| Routine | 0 | Read-only, simulations |
| Standard | 10 min | Small participant deposits |
| Elevated | 1 hour | Manager rebalances, large writes |
| High | 24 hours | Large operations, config changes |
| Critical | 48 hours | Admin operations, ownership changes |

---

#### `vault_proxy_announce`

Announce a pending operation with calculated risk tier and delay.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ProxyAnnounceParams {
    pub vault: String,
    pub chain_id: u64,
    /// The operation to announce (serialized tool call).
    pub operation: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ProxyAnnounceResult {
    pub tx_hash: String,
    pub announcement_id: String,
    pub risk_tier: String,
    pub delay_seconds: u64,
    pub executable_after: String,
    pub expires_at: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Classifying risk tier", "Submitting announcement", "Confirming on-chain"]`

**Ground truth**: Read announcement from Warden contract (requires optional Warden module). Verify risk tier, delay, and operation hash match.

**promptSnippet**:
> Announce a pending operation with calculated risk tier and delay. Required before any write operation that goes through the Warden (requires optional Warden module).

**promptGuidelines**:
- **thriving**: Use the Warden for all manager write operations when deployed. Announce and wait.
- **cautious**: The delay is a safety feature. Do not try to circumvent it.
- **declining**: Announce exit operations. Shorter delays apply for routine operations.
- **terminal**: Announce emergency withdrawals. Critical tier operations still require 48h -- plan ahead.

---

#### `vault_proxy_execute`

Execute a previously announced operation after the delay window has elapsed.

**Capability**: `Write` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ProxyExecuteParams {
    pub vault: String,
    pub chain_id: u64,
    pub announcement_id: String,
}

#[derive(Debug, Serialize)]
pub struct ProxyExecuteResult {
    pub tx_hash: String,
    pub announcement_id: String,
    pub operation_result: serde_json::Value,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Verifying delay elapsed", "Re-validating operation", "Executing operation", "Confirming on-chain"]`

**Ground truth**: Verify announcement status is "executed" in Warden contract (requires optional Warden module). Verify operation result matches expected outcome.

**promptSnippet**:
> Execute a previously announced operation after the delay window has elapsed. Check `vault_proxy_get_pending` to verify the delay has passed.

**promptGuidelines**:
- **thriving**: Execute as soon as delay elapses. Announcements expire if not executed within the window.
- **cautious**: Verify market conditions have not changed significantly since announcement.
- **declining**: Execute exit operations promptly after delay.
- **terminal**: Execute all pending operations. Time is running out.

---

#### `vault_proxy_cancel`

Cancel a pending announcement before execution.

**Capability**: `Write` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ProxyCancelParams {
    pub vault: String,
    pub chain_id: u64,
    pub announcement_id: String,
}

#[derive(Debug, Serialize)]
pub struct ProxyCancelResult {
    pub tx_hash: String,
    pub announcement_id: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Submitting cancellation", "Confirming on-chain"]`

**Ground truth**: Verify announcement status is "cancelled" in Warden contract (requires optional Warden module).

**promptSnippet**:
> Cancel a pending announcement. Use when conditions have changed and the announced operation is no longer desirable.

---

#### `vault_proxy_get_pending`

List all pending announcements with remaining delay.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ProxyGetPendingParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendingAnnouncements {
    pub announcements: Vec<PendingAnnouncement>,
}

#[derive(Debug, Serialize)]
pub struct PendingAnnouncement {
    pub announcement_id: String,
    pub risk_tier: String,
    pub operation_summary: String,
    pub announced_at: String,
    pub executable_after: String,
    pub expires_at: String,
    pub remaining_seconds: u64,
    pub ready: bool,
}
```

**Progress steps**: `["Querying Warden contract (requires optional Warden module)"]`

**promptSnippet**:
> List all pending announcements with remaining delay time. Check before `vault_proxy_execute` to verify operations are ready.

---

#### `vault_proxy_get_history`

Get execution history with outcomes.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ProxyGetHistoryParams {
    pub vault: String,
    pub chain_id: u64,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ProxyHistory {
    pub entries: Vec<ProxyHistoryEntry>,
}

#[derive(Debug, Serialize)]
pub struct ProxyHistoryEntry {
    pub announcement_id: String,
    pub status: String,
    pub risk_tier: String,
    pub operation_summary: String,
    pub announced_at: String,
    pub executed_at: Option<String>,
    pub tx_hash: Option<String>,
}
```

**Progress steps**: `["Querying Warden history (requires optional Warden module)"]`

**promptSnippet**:
> Get execution history with outcomes. Use for audit trail and performance review.

---

#### `vault_proxy_get_config`

Get current risk tier configuration and delay settings.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ProxyGetConfigParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ProxyConfig {
    pub risk_tiers: Vec<RiskTierConfig>,
    pub expiry_window_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct RiskTierConfig {
    pub tier: String,
    pub delay_seconds: u64,
    pub description: String,
}
```

**Progress steps**: `["Reading Warden configuration (requires optional Warden module)"]`

**promptSnippet**:
> Get current risk tier configuration and delay settings. Verify delay expectations before announcing operations.

---

## am-AMM tools (5)

Auction-managed AMM tools for Bunni v2 pool management bidding via BidDog contracts.

#### `vault_get_amamm_state`

Get pool management state: current manager, rent, deposit, fees.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetAmammStateParams {
    pub pool: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct AmammState {
    pub pool: String,
    pub current_manager: String,
    pub rent_per_block: String,
    pub manager_deposit: String,
    pub swap_fee_bps: u16,
    pub blocks_since_activation: u64,
    pub total_fees_earned: String,
}
```

**Progress steps**: `["Reading BidDog contract state"]`

**promptSnippet**:
> Get pool management state: current manager, rent rate, deposit, fees. Read-only. Call before bidding for pool management rights.

---

#### `vault_submit_amamm_bid`

Submit a bid for pool management rights. Requires 1.1x minimum increment over current bid. 7,200-block activation delay.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct SubmitAmammBidParams {
    pub pool: String,
    pub chain_id: u64,
    /// Rent per block (must be >= 1.1x current).
    pub rent_per_block: String,
    /// Deposit amount.
    pub deposit: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitAmammBidResult {
    pub tx_hash: String,
    pub bid_id: String,
    pub rent_per_block: String,
    pub deposit: String,
    pub activation_block: u64,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Validating bid increment", "Simulating bid", "Approving deposit token", "Submitting bid", "Confirming on-chain"]`

**Ground truth**: Read bid state from BidDog contract. Verify bid exists with correct rent and deposit.

**promptSnippet**:
> Submit a bid for pool management rights via BidDog. Requires 1.1x minimum increment over current bid. 7,200-block activation delay.

**promptGuidelines**:
- **thriving**: Bid on high-volume pools where management fee revenue exceeds rent cost.
- **cautious**: Only bid on pools you already LP in. Minimum viable bid.
- **declining**: Do not submit new bids.
- **terminal**: Never bid.

---

#### `vault_get_amamm_bid_status`

Check bid status, activation countdown, competing bids.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetAmammBidStatusParams {
    pub bid_id: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct AmammBidStatus {
    pub bid_id: String,
    pub status: String,
    pub rent_per_block: String,
    pub deposit: String,
    pub activation_block: u64,
    pub blocks_remaining: u64,
    pub competing_bids: u32,
}
```

**Progress steps**: `["Querying BidDog state"]`

**promptSnippet**:
> Check bid status, activation countdown, competing bids. Call after `vault_submit_amamm_bid` to track position.

---

#### `vault_withdraw_amamm_deposit`

Withdraw excess deposit from a pool management position.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct WithdrawAmammDepositParams {
    pub pool: String,
    pub chain_id: u64,
    /// Amount to withdraw. "max" for all excess.
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct WithdrawAmammDepositResult {
    pub tx_hash: String,
    pub withdrawn: String,
    pub remaining_deposit: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Computing withdrawable amount", "Submitting withdrawal", "Confirming on-chain"]`

**Ground truth**: Read deposit balance from BidDog. Verify decrease matches withdrawn amount.

**promptSnippet**:
> Withdraw excess deposit from a pool management position. Use to reclaim capital not needed for maintaining rights.

---

#### `vault_set_amamm_swap_fee`

Set the swap fee for a managed pool. Manager only.

**Capability**: `Write` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct SetAmammSwapFeeParams {
    pub pool: String,
    pub chain_id: u64,
    /// New swap fee in basis points.
    pub fee_bps: u16,
}

#[derive(Debug, Serialize)]
pub struct SetAmammSwapFeeResult {
    pub tx_hash: String,
    pub old_fee_bps: u16,
    pub new_fee_bps: u16,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Verifying manager role", "Submitting fee change", "Confirming on-chain"]`

**Ground truth**: Read swap fee from pool contract. Verify matches new value.

**promptSnippet**:
> Set the swap fee for a managed pool. Manager only. Adjusting fees affects volume and revenue.

---

## Deferred tools (7)

Loaded on demand. Primarily admin and emergency operations.

#### `vault_emergency_withdraw`

Emergency withdrawal bypassing normal Warden delay (when Warden is deployed; optional, deferred). Only available when circuit breaker is active or vault is under exploit.

**Capability**: `Privileged` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct EmergencyWithdrawParams {
    pub vault: String,
    pub chain_id: u64,
    /// Reason for emergency withdrawal.
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct EmergencyWithdrawResult {
    pub tx_hash: String,
    pub assets_received: String,
    pub emergency_reason: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Verifying circuit breaker state", "Executing emergency withdrawal", "Confirming on-chain"]`

**Ground truth**: Verify share balance is zero post-tx. Verify asset balance increased.

**promptSnippet**:
> Emergency withdrawal bypassing normal Warden delay (when deployed). Only available when circuit breaker is active or vault is under exploit. Last resort.

**promptGuidelines**:
- **thriving**: Never use unless a genuine emergency is detected (exploit, oracle manipulation).
- **cautious**: Available but should not be needed.
- **declining**: Use if vault is at risk of insolvency.
- **terminal**: Use if normal withdrawal path is blocked or too slow.

---

#### `vault_pause`

Pause vault operations. Owner only.

**Capability**: `Privileged` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct VaultPauseParams {
    pub vault: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VaultPauseResult {
    pub tx_hash: String,
    pub vault: String,
    pub paused: bool,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Verifying owner role", "Submitting pause", "Confirming on-chain"]`

**Ground truth**: Read `paused()` from vault contract. Verify returns `true`.

**promptSnippet**:
> Pause vault operations. Owner only. All deposits and management operations are blocked. Use when an exploit is detected.

---

#### `vault_unpause`

Resume vault operations. Owner only. Goes through Warden Critical tier (48h delay) when Warden is deployed (optional, deferred).

**Capability**: `Privileged` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

Same param/result types as `vault_pause`.

**Progress steps**: `["Announcing via Warden (48h delay, requires optional Warden module)", "Waiting for delay", "Executing unpause", "Confirming on-chain"]`

**Ground truth**: Read `paused()` from vault. Verify returns `false`.

**promptSnippet**:
> Resume vault operations. Owner only. 48h Warden delay (requires optional Warden module). Use after the emergency is resolved.

---

#### `vault_transfer_ownership`

Transfer vault ownership. Critical risk tier -- 48h Warden delay (requires optional Warden module). Irreversible once executed.

**Capability**: `Privileged` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct TransferOwnershipParams {
    pub vault: String,
    pub chain_id: u64,
    /// New owner address.
    pub new_owner: String,
}

#[derive(Debug, Serialize)]
pub struct TransferOwnershipResult {
    pub tx_hash: String,
    pub old_owner: String,
    pub new_owner: String,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Validating new owner address", "Announcing via Warden (48h delay, requires optional Warden module)", "Waiting for delay", "Executing transfer", "Confirming on-chain"]`

**Ground truth**: Read `owner()` from vault. Verify matches `new_owner`.

**promptSnippet**:
> Transfer vault ownership. Critical risk tier -- 48h Warden delay (requires optional Warden module). Irreversible. Verify destination address carefully.

---

#### `vault_set_fee_config`

Update management/performance fee parameters within immutable caps (500 bps mgmt, 5000 bps perf).

**Capability**: `Privileged` | **Risk tier**: `Layer2` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct SetFeeConfigParams {
    pub vault: String,
    pub chain_id: u64,
    pub management_fee_bps: Option<u16>,
    pub performance_fee_bps: Option<u16>,
}

#[derive(Debug, Serialize)]
pub struct SetFeeConfigResult {
    pub tx_hash: String,
    pub old_management_fee_bps: u16,
    pub new_management_fee_bps: u16,
    pub old_performance_fee_bps: u16,
    pub new_performance_fee_bps: u16,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Validating fee caps", "Announcing via Warden (1h delay, requires optional Warden module)", "Waiting for delay", "Executing fee update", "Confirming on-chain"]`

**Ground truth**: Read fee params from vault. Verify match new values.

**promptSnippet**:
> Update fee parameters within immutable caps (500 bps mgmt, 5000 bps perf). Warden Elevated tier (1h delay, requires optional Warden module).

---

#### `vault_set_identity_gate`

Toggle identity-gated deposits on or off.

**Capability**: `Privileged` | **Risk tier**: `Layer2` | **Tick budget**: `Medium` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct SetIdentityGateParams {
    pub vault: String,
    pub chain_id: u64,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct SetIdentityGateResult {
    pub tx_hash: String,
    pub old_state: bool,
    pub new_state: bool,
    pub gas_used: u64,
    pub explorer_url: String,
}
```

**Progress steps**: `["Announcing via Warden (10 min delay, requires optional Warden module)", "Waiting for delay", "Executing gate change", "Confirming on-chain"]`

**Ground truth**: Read `identityGated()` from vault. Verify matches `enabled`.

**promptSnippet**:
> Toggle identity-gated deposits on or off. When enabled, only ERC-8004 registered agents can deposit.

---

#### `vault_get_audit_trail`

Get complete audit trail of vault operations.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetAuditTrailParams {
    pub vault: String,
    pub chain_id: u64,
    pub since: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AuditTrail {
    pub entries: Vec<AuditEntry>,
}

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub action: String,
    pub actor: String,
    pub tx_hash: String,
    pub details: serde_json::Value,
    pub risk_tier: String,
}
```

**Progress steps**: `["Querying vault events", "Assembling audit trail"]`

**promptSnippet**:
> Complete audit trail of vault operations. Use for transparency reporting and regulatory compliance.

---

## Executor framework tools (4)

Executor contracts handle on-chain execution of vault strategy operations. The executor abstraction separates strategy logic from execution logic, enabling composable strategy execution with gas optimization and MEV protection.

#### `vault_list_executors`

List available executor contracts with capabilities and gas profiles.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ListExecutorsParams {
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ExecutorList {
    pub executors: Vec<ExecutorSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutorSummary {
    pub address: String,
    pub name: String,
    pub supported_operations: Vec<String>,
    pub gas_overhead_pct: f64,
    pub audit_status: String,
    pub audit_date: Option<String>,
}
```

**Progress steps**: `["Querying executor registry"]`

---

#### `vault_get_executor`

Get executor details: supported operations, gas overhead, audit status.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Fast` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetExecutorParams {
    pub executor: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ExecutorDetails {
    pub address: String,
    pub name: String,
    pub supported_operations: Vec<String>,
    pub gas_overhead_pct: f64,
    pub audit_status: String,
    pub audit_date: Option<String>,
    pub total_executions: u64,
    pub success_rate: f64,
}
```

**Progress steps**: `["Reading executor contract", "Loading execution history"]`

---

#### `vault_execute_batch`

Execute a batch of strategy operations through an executor. Operations validated against PolicyCage before submission.

**Capability**: `Write` | **Risk tier**: `Layer3` | **Tick budget**: `Slow` | **Sprite**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ExecuteBatchParams {
    pub vault: String,
    pub executor: String,
    pub chain_id: u64,
    /// Array of { adapter, action, params }.
    pub operations: Vec<ExecutorOperation>,
    /// Simulate before executing. Default: true.
    #[serde(default = "default_true")]
    pub simulate: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExecutorOperation {
    pub adapter: String,
    pub action: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ExecuteBatchResult {
    pub tx_hash: String,
    pub executed_ops: u32,
    pub total_gas_used: u64,
    pub state_changes: Vec<serde_json::Value>,
    pub explorer_url: String,
}
```

**Progress steps**: `["Simulating batch", "Validating against PolicyCage", "Approving tokens", "Submitting batch execution", "Confirming on-chain"]`

**Ground truth**: For each operation, verify adapter state changed as expected. Compare total assets pre-tx vs post-tx.

**promptSnippet**:
> Execute a batch of strategy operations through an executor contract. Validated against PolicyCage. Simulate first by default.

**promptGuidelines**:
- **thriving**: Use for multi-step strategy operations. Always simulate before executing.
- **cautious**: Simulate twice. Verify state changes match expectations.
- **declining**: Use for batch de-risking operations only.
- **terminal**: Use for batch exit operations.

---

#### `vault_simulate_execution`

Dry-run executor operations without broadcasting. Returns projected state changes, gas estimates, and PolicyCage validation results.

**Capability**: `Read` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` | **Sprite**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct SimulateExecutionParams {
    pub vault: String,
    pub executor: String,
    pub chain_id: u64,
    pub operations: Vec<ExecutorOperation>,
}

#[derive(Debug, Serialize)]
pub struct SimulationResult {
    pub valid: bool,
    pub policy_violations: Vec<String>,
    pub projected_state_changes: Vec<serde_json::Value>,
    pub gas_estimate: u64,
    pub revert_risk: f64,
}
```

**Progress steps**: `["Forking state via Revm", "Simulating operations", "Validating PolicyCage"]`

**promptSnippet**:
> Dry-run executor operations: projected state changes, gas estimates, PolicyCage validation. ALWAYS call before `vault_execute_batch`.

---

## Design rationale

### Intent-based grouping

Vault tools are grouped by user intent, not by contract function:

- **Lifecycle tools** (create, list, info) are the entry point
- **Participant tools** (deposit, withdraw, preview) are the primary user flow
- **Manager tools** (rebalance, collect, report) are the owner flow
- **Proxy tools** gate every write through announce-wait-execute

### Consolidations from v1

| v1 tools | v4 replacement | Rationale |
| --- | --- | --- |
| `vault_preview_deposit` + `vault_preview_withdraw` + `vault_simulate_deposit` | `vault_preview` | Single preview tool with `action` field and optional `scenario` for stress testing. Three tools doing variants of the same read operation collapsed to one. |
| `vault_register_agent` + `vault_claim_milestone` + `vault_get_reputation` + `vault_check_eligibility` | Use `identity_*` tools | Identity and reputation are not vault-specific. The `identity_register_agent`, `identity_claim_milestone`, `identity_get_reputation`, and `identity_check_eligibility` tools handle the same operations across all contexts. |
| `vault_marketplace_*` (6 tools) | Cut | Strategy marketplace is speculative. No on-chain contract designed yet. Will be re-added when the strategy marketplace PRD is finalized. |
| `vault_export_data` | Cut | Subsumed by `vault_get_audit_trail` + `vault_get_history`. Export formatting belongs in the surface layer. |

### Ownership and runtime boundaries

- **Vault owner**: Can pause/unpause, transfer ownership, update fees. Enforced by smart contract `onlyOwner`.
- **Vault manager** (am-AMM winner): Can rebalance, set swap fees, collect management fees. Enforced by BidDog.
- **Participant**: Can deposit (subject to identity gating), withdraw (never gated), preview. No restrictions on read tools.

---

## Error taxonomy

| Code | Description |
| --- | --- |
| `VAULT_NOT_FOUND` | Vault address not found or not a valid ERC-4626 |
| `IDENTITY_REQUIRED` | Vault requires ERC-8004 identity for deposit |
| `INSUFFICIENT_SHARES` | Not enough shares for withdrawal amount |
| `NOT_VAULT_OWNER` | Operation requires vault owner role |
| `NOT_VAULT_MANAGER` | Operation requires current am-AMM manager role |
| `VAULT_PAUSED` | Vault is paused, operations are blocked |
| `FEE_CAP_EXCEEDED` | Requested fees exceed immutable caps (500 bps mgmt, 5000 bps perf) |
| `PROXY_DELAY_NOT_MET` | Warden delay window has not elapsed (requires optional Warden module) |
| `PROXY_ALREADY_EXECUTED` | Announcement already executed |
| `PROXY_EXPIRED` | Announcement expired before execution |
| `POLICY_VIOLATION` | Operation violates PolicyCage constraints |
| `EXECUTOR_NOT_FOUND` | Executor contract not registered |
| `BID_INCREMENT_TOO_LOW` | am-AMM bid does not meet 1.1x minimum increment |

---

## Custody implications (vault write tools)

All vault write tools share the same custody behavior:

- **Delegation**: Session key signs via the vault contract or factory. CaveatEnforcer must whitelist the vault address (or factory for creation). Vault ownership operations (`vault_set_fees`, `vault_pause`, `vault_unpause`) require the session key to be authorized for the vault's `owner()` function calls.
- **Embedded**: Privy server wallet signs. PolicyCage validates vault target address, operation type, and parameter bounds (fee caps, allocation limits).
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed vault targets and operations.

Privileged tools (`vault_transfer_ownership`, `vault_set_guardian`, etc.) require owner approval in addition to the capability token, regardless of custody mode.

---

## Golem integration

Golems operating as vault managers use vault tools in their heartbeat loop:

1. **Monitor**: `vault_get_info` + `vault_get_allocations` to check vault health
2. **Decide**: `vault_get_performance` + Grimoire insights to assess rebalance need
3. **Execute**: `vault_rebalance` via Warden if deployed (announce-wait-execute; optional, deferred)
4. **Learn**: Grimoire stores outcome for strategy refinement

The am-AMM bidding tools enable Golems to compete for pool management rights -- a direct revenue stream. A Golem's bid strategy (rent level, deposit sizing) is governed by its STRATEGY.md and refined through operational experience stored in the Grimoire.

---

## Tool count summary

| Group | Tools | Read | Write | Privileged |
| --- | --- | --- | --- | --- |
| Core vault (lifecycle) | 4 | 3 | 1 | 0 |
| Core vault (participant) | 4 | 2 | 2 | 0 |
| Core vault (manager) | 5 | 1 | 4 | 0 |
| Core vault (analytics) | 4 | 4 | 0 | 0 |
| Core vault (identity) | 0 | -- | -- | -- |
| Proxy (Warden, deferred) | 6 | 3 | 3 | 0 |
| am-AMM | 5 | 2 | 3 | 0 |
| Deferred | 7 | 1 | 0 | 6 |
| Executor framework | 4 | 2 | 1 | 0 |
| Strategy marketplace | 0 | -- | -- | -- |
| Strategy auction | 0 | -- | -- | -- |
| **Total** | **42** | **18** | **14** | **6** |

Note: Identity tools (4 in v1) moved to `identity_*` category. Strategy marketplace (6) and strategy auction (3) cut pending design finalization.
