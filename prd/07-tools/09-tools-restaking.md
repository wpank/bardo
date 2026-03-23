# Bardo Tools -- Restaking [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles), [08-tools-staking.md](08-tools-staking.md) (Lido and Rocket Pool liquid staking tools)
>
> EigenLayer restaking, liquid restaking tokens (ether.fi, Kelp), and points tracking. Category: `restaking`. Total: 16 tools.

---

> **Reader orientation:** This document specifies 16 restaking tools in `bardo-tools` for EigenLayer, liquid restaking tokens (ether.fi, Kelp, Puffer), and points tracking. You should be familiar with EigenLayer's delegation/operator/AVS model and the liquid restaking token ecosystem. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## EigenLayer (8 tools)

EigenLayer allows LST holders to restake by depositing into strategies and delegating to operators who run Actively Validated Services (AVSs). Withdrawal has a 7-day delay. Slashing risk comes from operator misbehavior on specific AVSs.

### `eigenlayer_deposit_lst`

Deposit a liquid staking token into an EigenLayer strategy.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenDepositParams {
    /// LST address or symbol (stETH, rETH, cbETH, etc.).
    pub token: String,
    /// Amount to restake.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct EigenDepositResult {
    pub tx_hash: String,
    pub shares_received: String,
    pub strategy: String,
    pub delegated_to: Option<String>,
    pub slashing_risk: String,  // "none" | "low" | "medium"
}
```

**capability**: `Write` | **risk_tier**: `Layer3` | **tick_budget**: `Medium`
**progress_steps**: `["Approving LST", "Signing deposit", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Deposit LST into EigenLayer strategy for restaking. Adds slashing risk from operator AVS participation. Check operator slashing history before depositing. 7-day withdrawal delay applies."

**promptGuidelines**:
- **thriving**: Restake for additional yield + points. Choose operators with clean slashing history and diverse AVS participation.
- **cautious**: Only restake if yield premium > 2% over base staking. Verify operator reputation.
- **declining**: Do not restake. Initiate EigenLayer withdrawals (7-day delay).
- **terminal**: Do not restake. Queue all withdrawals immediately.

**Events**: `tool:start` -> `tool:update` (approving LST) -> `tool:update` (signing deposit) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (spending limit + operator slashing check), `tool_result` (shares + delegation logged)

**Ground truth**: Verify `shares_received` against strategy shares delta. Source: on-chain `stakerStrategyShares` read.

### `eigenlayer_delegate`

Delegate restaked assets to an EigenLayer operator.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenDelegateParams {
    /// Operator address.
    pub operator: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct EigenDelegateResult {
    pub tx_hash: String,
    pub operator: String,
    pub delegated_shares: String,
    pub avs_count: u32,
    pub slashing_history: Vec<SlashingEvent>,
}

#[derive(Debug, Serialize)]
pub struct SlashingEvent {
    pub date: String,
    pub amount_slashed: String,
    pub reason: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Validating operator", "Signing delegation", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Delegate restaked assets to an EigenLayer operator. Delegation determines which AVSs your capital validates and what slashing risk you carry. Call `eigenlayer_get_operators` first."

**promptGuidelines**:
- **thriving**: Delegate to operators with 0 slashing incidents, > 100 delegators, and diverse AVS mix.
- **cautious**: Only delegate to top-10 operators by delegated ETH. Avoid operators with slashing history.
- **declining**: Undelegate from all operators.
- **terminal**: Undelegate immediately.

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (operator slashing check), `tool_result` (delegation logged)

**Ground truth**: Verify delegation via `delegatedTo(staker)`. Source: on-chain DelegationManager.

### `eigenlayer_undelegate`

Undelegate from an operator. Initiates the withdrawal process with a 7-day delay.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenUndelegateParams {
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct EigenUndelegateResult {
    pub tx_hash: String,
    pub withdrawal_root: String,
    pub estimated_completion: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Signing undelegation", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Undelegate from operator. Initiates 7-day withdrawal. Assets remain at slashing risk during this period. Start undelegation early if you plan to exit."

**promptGuidelines**:
- **thriving**: Undelegate when changing operators or exiting EigenLayer.
- **cautious**: Undelegate from operators with any new slashing events.
- **declining**: Undelegate from all operators. Start the 7-day clock now.
- **terminal**: Undelegate immediately if not already done. You need 7 days.

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (withdrawal root + completion time logged)

**Ground truth**: Verify undelegation via `delegatedTo(staker) == address(0)`. Source: on-chain state read.

### `eigenlayer_queue_withdrawal`

Queue a withdrawal from EigenLayer strategies.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenQueueWithdrawalParams {
    /// Array of strategy addresses to withdraw from.
    pub strategies: Vec<String>,
    /// Corresponding shares amounts.
    pub shares: Vec<String>,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct EigenQueueWithdrawalResult {
    pub tx_hash: String,
    pub withdrawal_root: String,
    pub queued_at: String,
    pub completion_time: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Signing queue withdrawal", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Queue withdrawal from EigenLayer strategies. 7-day delay. Specify which strategies and shares to withdraw."

**promptGuidelines**:
- **thriving**: Queue withdrawals when rotating strategies. Complete promptly after delay.
- **cautious**: Queue all non-essential EigenLayer positions for withdrawal.
- **declining**: Queue everything. Complete as soon as delay expires.
- **terminal**: Queue everything immediately. Set reminders for completion 7 days out.

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (withdrawal root logged)

**Ground truth**: Verify withdrawal queued via `pendingWithdrawals`. Source: on-chain DelegationManager.

### `eigenlayer_complete_withdrawal`

Complete a queued withdrawal after the 7-day delay.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenCompleteWithdrawalParams {
    /// Withdrawal root from `eigenlayer_queue_withdrawal`.
    pub withdrawal_root: String,
    /// Receive underlying tokens rather than shares. Default: true.
    #[serde(default = "default_receive_tokens")]
    pub receive_as_tokens: bool,
    /// Chain ID.
    pub chain: u64,
}

fn default_receive_tokens() -> bool { true }

#[derive(Debug, Serialize)]
pub struct EigenCompleteWithdrawalResult {
    pub tx_hash: String,
    pub tokens_received: Vec<TokenAmount>,
}

#[derive(Debug, Serialize)]
pub struct TokenAmount {
    pub token: String,
    pub amount: String,
}
```

**capability**: `Write` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Verifying delay elapsed", "Signing completion", "Receiving tokens", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Complete a queued withdrawal after 7-day delay. Set receive_as_tokens=true to get underlying tokens (not shares)."

**Events**: `tool:start` -> `tool:update` (signing) -> `tool:update` (receiving tokens) -> `tool:end`
**Pi hooks**: `tool_call` (delay elapsed check), `tool_result` (withdrawal logged)

**Ground truth**: Verify token balance increases match `tokens_received`. Source: on-chain balance diffs.

### `eigenlayer_get_operators`

List EigenLayer operators with performance and risk metrics.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenGetOperatorsParams {
    /// Chain ID.
    pub chain: u64,
    /// Minimum ETH delegated.
    #[serde(default)]
    pub min_delegated_eth: Option<f64>,
    /// Filter by AVS participation.
    #[serde(default)]
    pub avs_filter: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct EigenOperator {
    pub address: String,
    pub name: String,
    pub total_delegated_eth: f64,
    pub delegator_count: u32,
    pub avs_participation: Vec<AvsParticipation>,
    pub slashing_history: Vec<SlashingEvent>,
    pub commission_rate: f64,
    pub reputation_score: f64,
}

#[derive(Debug, Serialize)]
pub struct AvsParticipation {
    pub avs: String,
    pub avs_name: String,
    pub slashing_risk: String,  // "low" | "medium" | "high"
}

#[derive(Debug, Serialize)]
pub struct EigenOperatorsResult {
    pub operators: Vec<EigenOperator>,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Fetching operator data"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "List operators with performance, slashing history, AVS participation, and commission rates. Use before delegating."

**promptGuidelines**:
- **thriving**: Research operators and AVSs before delegation. Monitor position on heartbeat.
- **cautious**: Monitor position for any slashing events.
- **declining**: Check position for pending withdrawals.
- **terminal**: Final position check.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (slashing events trigger alert)

### `eigenlayer_get_position`

Get an account's EigenLayer restaking position.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenGetPositionParams {
    /// Account address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct EigenPosition {
    pub strategies: Vec<StrategyPosition>,
    pub delegated_to: Option<String>,
    pub operator: Option<EigenOperator>,
    pub pending_withdrawals: Vec<PendingWithdrawal>,
    pub points_earned: f64,
}

#[derive(Debug, Serialize)]
pub struct StrategyPosition {
    pub token: String,
    pub shares: String,
    pub eth_value: String,
}

#[derive(Debug, Serialize)]
pub struct PendingWithdrawal {
    pub withdrawal_root: String,
    pub completion_time: String,
    pub tokens: Vec<TokenAmount>,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching position"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Account's EigenLayer position: strategies, delegation, pending withdrawals, points earned."

**Events**: `tool:start` -> `tool:end`

### `eigenlayer_get_avs_list`

List Actively Validated Services and their requirements.

```rust
#[derive(Debug, Deserialize)]
pub struct EigenGetAvsParams {
    /// Chain ID.
    pub chain: u64,
    /// Filter by category (bridge, oracle, rollup, etc.).
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AvsService {
    pub address: String,
    pub name: String,
    pub category: String,
    pub tvl: f64,
    pub operator_count: u32,
    pub slash_conditions: String,
    pub restaker_rewards: String,
}

#[derive(Debug, Serialize)]
pub struct AvsListResult {
    pub avs_services: Vec<AvsService>,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Fetching AVS data"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "List Actively Validated Services with requirements, TVL, slashing conditions, and restaker rewards."

**Events**: `tool:start` -> `tool:end`

**Error codes**: `OPERATOR_NOT_REGISTERED`, `STRATEGY_NOT_SUPPORTED`, `WITHDRAWAL_DELAY_PENDING`, `UNDELEGATION_REQUIRED`

---

## Liquid restaking tokens (6 tools)

LRTs represent restaked ETH across multiple EigenLayer operators and AVSs. Each protocol has its own strategy for operator selection and point accumulation. The April 2024 Renzo ezETH depeg (peak -50% vs ETH) and withdrawal freeze demonstrated that LRT concentration and withdrawal mechanics are critical risk factors.

This integration covers ether.fi (eETH/weETH) and Kelp (rsETH) -- the two LRTs with the highest TVL and best withdrawal support.

### `lrt_deposit`

Deposit ETH or LST to receive a liquid restaking token.

```rust
#[derive(Debug, Deserialize)]
pub struct LrtDepositParams {
    /// Amount to deposit.
    pub amount: String,
    /// Input token (ETH or LST address; default: ETH).
    #[serde(default = "default_eth")]
    pub input_token: String,
    /// LRT protocol.
    pub protocol: String,  // "etherfi" | "kelp"
    /// Chain ID.
    pub chain: u64,
}

fn default_eth() -> String { "ETH".into() }

#[derive(Debug, Serialize)]
pub struct LrtDepositResult {
    pub tx_hash: String,
    pub lrt_received: String,
    pub lrt_token: String,
    pub exchange_rate: String,
    pub current_apy: f64,
}
```

**capability**: `Write` | **risk_tier**: `Layer3` | **tick_budget**: `Medium`
**progress_steps**: `["Checking depeg risk", "Approving input token", "Signing deposit", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Deposit ETH or LST to receive an LRT (eETH, rsETH). Adds restaking exposure on top of base staking yield. LRT depeg risk is real -- check `lrt_get_depeg_risk` first."

**promptGuidelines**:
- **thriving**: Deposit for combined staking + restaking + points yield. ALWAYS check depeg risk first. Avoid protocols with peg_deviation_7d > 2%.
- **cautious**: Only deposit into ether.fi (eETH) -- it has the best instant redemption support. Maximum 10% of portfolio in LRTs.
- **declining**: Do not deposit. Withdraw from LRT positions.
- **terminal**: Never deposit. Initiate all LRT withdrawals.

**Events**: `tool:start` -> `tool:update` (checking depeg) -> `tool:update` (approving) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (spending limit + depeg risk pre-check), `tool_result` (LRT received + exchange rate logged)

**Ground truth**: Verify LRT balance increase. Source: on-chain balance read.

### `lrt_withdraw`

Withdraw from an LRT protocol. Instant where supported (ether.fi has instant via eETH liquidity pool); queued otherwise.

```rust
#[derive(Debug, Deserialize)]
pub struct LrtWithdrawParams {
    /// LRT amount.
    pub amount: String,
    /// LRT protocol.
    pub protocol: String,  // "etherfi" | "kelp"
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LrtWithdrawResult {
    pub tx_hash: String,
    pub eth_received: Option<String>,     // Some if instant
    pub request_id: Option<String>,       // Some if queued
    pub estimated_completion: Option<String>,
}
```

**capability**: `Write` | **risk_tier**: `Layer2` | **tick_budget**: `Medium`
**progress_steps**: `["Checking withdrawal path", "Signing withdrawal", "Confirming"]`
**sprite_trigger**: `Executing`

**promptSnippet**: "Withdraw from LRT. Instant via ether.fi eETH pool if available. Other protocols may require queued withdrawal."

**promptGuidelines**:
- **thriving**: Use instant withdrawal when available. Queue for protocols without instant support.
- **cautious**: Prefer instant withdrawal. If not available, queue early.
- **declining**: Withdraw from all LRT positions. Use instant where possible.
- **terminal**: Instant for everything possible. Queue the rest immediately.

**Events**: `tool:start` -> `tool:update` (checking path) -> `tool:update` (signing) -> `tool:update` (confirming) -> `tool:end`
**Pi hooks**: `tool_call` (balance check), `tool_result` (withdrawal receipt or queue position logged)

**Ground truth**: For instant: verify ETH balance increase. For queued: verify request ID exists. Source: on-chain reads.

### `lrt_get_apy`

Get current APY breakdown for LRT protocols.

```rust
#[derive(Debug, Deserialize)]
pub struct LrtGetApyParams {
    /// Protocol filter. Default: "all".
    #[serde(default = "default_all")]
    pub protocol: String,  // "etherfi" | "kelp" | "all"
    /// Chain ID.
    pub chain: u64,
}

fn default_all() -> String { "all".into() }

#[derive(Debug, Serialize)]
pub struct LrtApyBreakdown {
    pub protocol: String,
    pub lrt_symbol: String,
    pub base_staking_apy: f64,
    pub restaking_apy: f64,
    pub protocol_rewards: f64,
    pub total_apy: f64,
    pub points_program: PointsProgram,
}

#[derive(Debug, Serialize)]
pub struct PointsProgram {
    pub name: String,
    pub accumulation_rate: f64,
    pub estimated_usd_value: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct LrtApyResult {
    pub apys: Vec<LrtApyBreakdown>,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching APY data"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "LRT APY breakdown: base staking + restaking + protocol rewards + points. Use to evaluate whether LRT exposure justifies the additional risk layer."

**promptGuidelines**:
- **thriving**: Compare across protocols. Factor in points value estimates (speculative).
- **cautious**: Focus on base + restaking APY only. Ignore speculative points value.
- **declining**: Check if current yield justifies maintaining the position.
- **terminal**: Not needed.

**Events**: `tool:start` -> `tool:end`

### `lrt_get_position`

Get account's LRT holdings and pending withdrawals across ether.fi and Kelp.

```rust
#[derive(Debug, Deserialize)]
pub struct LrtGetPositionParams {
    /// Account address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LrtPosition {
    pub holdings: Vec<LrtHolding>,
    pub pending_withdrawals: Vec<LrtPendingWithdrawal>,
}

#[derive(Debug, Serialize)]
pub struct LrtHolding {
    pub protocol: String,
    pub lrt_symbol: String,
    pub balance: String,
    pub balance_eth: String,
    pub balance_usd: f64,
    pub apy: f64,
    pub points_earned: f64,
}

#[derive(Debug, Serialize)]
pub struct LrtPendingWithdrawal {
    pub protocol: String,
    pub request_id: String,
    pub amount: String,
    pub status: String,
    pub estimated_completion: String,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching holdings"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Account's LRT holdings and pending withdrawals across all supported protocols. Portfolio overview for restaking exposure."

**promptGuidelines**:
- **thriving**: Review on heartbeat.
- **cautious**: Review daily. Verify all positions are expected.
- **declining**: Plan exit order.
- **terminal**: Final inventory.

**Events**: `tool:start` -> `tool:end`

### `lrt_get_depeg_risk`

Assess depeg risk for an LRT.

```rust
#[derive(Debug, Deserialize)]
pub struct LrtDepegRiskParams {
    /// LRT protocol.
    pub protocol: String,  // "etherfi" | "kelp"
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct LrtDepegRisk {
    pub lrt_symbol: String,
    pub current_premium_discount: f64,
    pub max_deviation_7d: f64,
    pub max_deviation_30d: f64,
    pub withdrawal_queue_depth: u32,
    pub instant_redemption_available: bool,
    pub slashing_exposure: Vec<SlashingExposure>,
    pub risk_level: String,      // "low" | "medium" | "high"
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SlashingExposure {
    pub avs: String,
    pub exposure_pct: f64,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching peg data", "Analyzing risk"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Assess LRT depeg risk: current discount, max deviations (7d/30d), withdrawal queue, instant redemption, slashing exposure, and risk level. ALWAYS call before `lrt_deposit`. The Renzo ezETH -50% depeg is the reference case."

**promptGuidelines**:
- **thriving**: Check before every LRT deposit. Require risk level "low". Flag any risk factors.
- **cautious**: Check daily for all held LRTs. Exit at risk level "medium" or higher.
- **declining**: Check for held LRTs. Exit any at "medium" or higher immediately.
- **terminal**: Not needed. Exit all positions regardless.

**Events**: `tool:start` -> `tool:end`
**Pi hooks**: `tool_call` (rate-limit), `tool_result` (high risk triggers hard block on LRT deposits)

**Error codes**: `LRT_PROTOCOL_PAUSED`, `LRT_WITHDRAWAL_SUSPENDED`, `LRT_MINIMUM_DEPOSIT`, `LRT_DEPEG_WARNING`

---

## Points tracking (2 tools)

EigenLayer points and LRT-specific points programs do not have official USD values -- they represent claims on future airdrops. `estimated_usd_value` is derived from secondary market trading of point tokens and futures where available.

### `lrt_get_restaking_points`

Get all restaking and LRT points accumulated by an account.

```rust
#[derive(Debug, Deserialize)]
pub struct GetRestakingPointsParams {
    /// Account address.
    pub account: String,
    /// Chain ID.
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct RestakingPoints {
    pub account: String,
    pub eigen_layer_points: f64,
    pub lrt_points: Vec<LrtPointsEntry>,
    pub total_estimated_usd_value: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct LrtPointsEntry {
    pub protocol: String,
    pub program_name: String,
    pub points: f64,
    pub accumulation_rate: f64,
    pub estimated_usd_value: Option<f64>,
    pub source: String,  // "official" | "secondary-market" | "estimated"
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Fast`
**progress_steps**: `["Fetching points data"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "All restaking and LRT points for an account: EigenLayer points, protocol-specific points, estimated USD values. Points have no official value -- estimates are from secondary markets."

**promptGuidelines**:
- **thriving**: Track monthly. Points inform position sizing but should not drive decisions alone.
- **cautious**: Track but do not factor into risk decisions. Points are speculative.
- **declining**: Check for any claimable value. Otherwise ignore.
- **terminal**: Note total points in death testament. May have future value for successor.

**Events**: `tool:start` -> `tool:end`

### `lrt_get_points_strategies`

Compare protocols by points accumulation efficiency.

```rust
#[derive(Debug, Deserialize)]
pub struct GetPointsStrategiesParams {
    /// ETH amount for comparison.
    pub amount: String,
    /// Chain ID.
    pub chain: u64,
    /// Days to project (default: 30).
    #[serde(default = "default_duration")]
    pub duration: u32,
}

fn default_duration() -> u32 { 30 }

#[derive(Debug, Serialize)]
pub struct PointsProjection {
    pub protocol: String,
    pub strategy: String,
    pub points_accumulated: Vec<PointsAccumulation>,
    pub total_estimated_usd_value: Option<f64>,
    pub staking_apy: f64,
    pub total_estimated_return: Option<f64>,
    pub risk: String,
}

#[derive(Debug, Serialize)]
pub struct PointsAccumulation {
    pub program: String,
    pub points: f64,
}

#[derive(Debug, Serialize)]
pub struct PointsStrategiesResult {
    pub projections: Vec<PointsProjection>,
    pub recommendation: String,
}
```

**capability**: `Read` | **risk_tier**: `Layer1` | **tick_budget**: `Medium`
**progress_steps**: `["Computing projections"]`
**sprite_trigger**: `Thinking`

**promptSnippet**: "Compare protocols by points accumulation efficiency: projected points per ETH per day, estimated USD value, staking APY, and total return. Use for points-optimized positioning."

**promptGuidelines**:
- **thriving**: Use to optimize points accumulation alongside yield. Balance points vs exit flexibility.
- **cautious**: Do not chase points at the expense of exit liquidity. Points are speculative.
- **declining**: Not actionable.
- **terminal**: Not needed.

**Events**: `tool:start` -> `tool:end`

---

## Custody implications (restaking write tools)

All restaking write tools share the same custody behavior:

- **Delegation**: Session key signs via EigenLayer StrategyManager, DelegationManager, or LRT protocol contracts. CaveatEnforcer must whitelist canonical EigenLayer and LRT contract addresses. Deposit amounts bounded by session key spending limits.
- **Embedded**: Privy server wallet signs. PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) validates protocol target, deposit amount, and operator delegation target.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed targets and amounts.

Withdrawal tools (`eigenlayer_queue_withdrawal`, `eigenlayer_complete_withdrawal`, `lrt_withdraw`) are always permitted regardless of lifecycle phase. The 7-day EigenLayer withdrawal delay is enforced on-chain, not at the custody layer.

---

## Restaking safety invariants

All restaking tools enforce these invariants in the adapter layer:

- **Address allowlist**: only canonical EigenLayer, ether.fi, and Kelp contract addresses.
- **Operator validation**: `eigenlayer_delegate` checks operator slashing history before execution.
- **Depeg gate**: `lrt_deposit` internally calls `lrt_get_depeg_risk` and blocks if risk_level is "high".
- **Phase gating**: `declining` and `terminal` block all deposit/delegate operations. Withdrawal operations are always allowed.
- **7-day withdrawal delay**: All EigenLayer withdrawal tools surface the delay prominently in results and promptSnippets.
- **Preview->commit lifecycle**: all write tools go through `preview_action` -> `commit_action`.

### Error reference

| Error code | Category | Trigger |
| --- | --- | --- |
| `OPERATOR_NOT_REGISTERED` | `state` | Operator address not registered in EigenLayer |
| `STRATEGY_NOT_SUPPORTED` | `state` | LST not supported by any EigenLayer strategy |
| `WITHDRAWAL_DELAY_PENDING` | `state` | Attempting to complete withdrawal before 7-day delay |
| `UNDELEGATION_REQUIRED` | `state` | Must undelegate before queueing withdrawal |
| `LRT_PROTOCOL_PAUSED` | `state` | LRT protocol has paused operations |
| `LRT_WITHDRAWAL_SUSPENDED` | `state` | LRT withdrawals temporarily suspended |
| `LRT_MINIMUM_DEPOSIT` | `policy` | Amount below minimum deposit threshold |
| `LRT_DEPEG_WARNING` | `safety` | LRT peg deviation exceeds 2% threshold |
