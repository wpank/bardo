# Bardo Tools -- Safety [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Transaction safety, portfolio health monitoring, liquidation risk tracking, and alerting. Category: `safety`. 16 tools total. The Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) heartbeat (the Golem's periodic 9-step decision cycle) calls `safety_get_risk_dashboard` on every tick when any lending, looping, or derivatives position is active. Health alerts persist in the Grimoire (the Golem's persistent memory system with episodic, semantic, and strategy layers) under the `health_alerts` namespace.

---

> **Reader orientation:** This document specifies 16 safety tools in `bardo-tools`: transaction simulation, token verification, portfolio health monitoring, liquidation risk tracking, and alerting. These tools form the defense-in-depth layer that protects agents from fund loss. You should be familiar with Revm (Rust EVM) simulation concepts, health factor mechanics, and DeFi liquidation triggers. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Transaction safety (6 tools)

### `safety_simulate_transaction`

Pre-flight simulation of a transaction via Revm (Rust EVM implementation for local transaction simulation against forked chain state) fork. Returns balance changes, gas estimate, and revert detection without broadcasting. Replaces the old `eth_call`-based simulation with a full Revm fork that supports multi-step execution, state inspection, and deterministic gas profiling.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium` (1-5s -- fork creation + execution)
**Progress steps**: `["Creating Revm fork at current block", "Executing transaction in fork", "Inspecting state changes", "Computing balance deltas"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct SimulateTransactionParams {
    /// Target contract address (0x-prefixed).
    pub to: String,
    /// Encoded calldata (hex).
    pub data: String,
    /// ETH value in wei (default: "0").
    #[serde(default)]
    pub value: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct SimulationResult {
    pub success: bool,
    pub gas_estimate: u64,
    pub balance_changes: Vec<BalanceChange>,
    pub logs: Vec<DecodedLog>,
    pub decoded_result: Option<String>,
    pub revert_reason: Option<String>,
    /// State diff from the Revm fork.
    pub state_changes: Vec<StorageChange>,
}

#[derive(Debug, Serialize)]
pub struct BalanceChange {
    pub token: String,
    pub symbol: String,
    pub before: String,
    pub after: String,
    pub delta: String,
    pub usd_value: f64,
}

#[derive(Debug, Serialize)]
pub struct DecodedLog {
    pub address: String,
    pub event_name: Option<String>,
    pub topics: Vec<String>,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct StorageChange {
    pub contract: String,
    pub slot: String,
    pub before: String,
    pub after: String,
}
```

**Event Fabric**:
- `tool:start` -> `{ tool_name: "safety_simulate_transaction", params_hash }`
- `tool:update` -> `"Creating Revm fork at current block"` -> `"Executing transaction in fork"` -> `"Inspecting state changes"` -> `"Computing balance deltas"`
- `tool:end` -> `{ success, gas_estimate, balance_change_count }`

**promptSnippet**: "Pre-flight simulation via Revm fork (not eth_call). Returns balance changes, gas estimate, state diff, revert detection. ALWAYS run before any write transaction you haven't simulated through another tool's built-in simulation."

**promptGuidelines**:
- "In thriving phase: run before any manual calldata execution. Built-in tool simulations (e.g., `commit_action`) already include this."
- "In stable phase: run before all manual calls. Compare results against expected outcome."
- "In conservation phase: run twice. Compare results. If they differ, do not proceed."
- "In declining phase: run before every exit transaction."
- "In terminal phase: run before every transaction. Revert costs gas you cannot afford to waste."

### `safety_validate_token`

Verify a token is legitimate: bytecode check, ERC-20 interface compliance, token list presence, liquidity presence, transfer tax detection. Uses a Revm fork to simulate buy+sell and detect tax/honeypot behavior without risking funds.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Checking bytecode", "Verifying ERC-20 interface", "Querying token lists", "Simulating buy+sell in Revm fork", "Computing transfer tax"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct ValidateTokenParams {
    /// Token address (0x-prefixed).
    pub token: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct TokenValidationResult {
    pub token: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub is_erc20_compliant: bool,
    pub on_token_list: bool,
    pub token_list_source: Option<String>,
    pub has_liquidity: bool,
    pub liquidity_usd: f64,
    pub transfer_tax_buy_bps: u32,
    pub transfer_tax_sell_bps: u32,
    pub is_honeypot: bool,
    pub risk_level: TokenRisk,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub enum TokenRisk {
    Safe,
    Caution,
    Dangerous,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ risk_level, is_honeypot, transfer_tax_buy_bps }`

**promptSnippet**: "Verifies token legitimacy: bytecode, ERC-20, token list, liquidity, transfer tax, honeypot detection. ALWAYS call before first interaction with any unverified token."

**promptGuidelines**:
- "In thriving phase: call for every new token. Reject tokens with transfer tax > 5% or no liquidity."
- "In stable/conservation phase: call for every token, even previously verified if > 7d since last check. Zero tolerance for tax."
- "In declining phase: verify exit route tokens only."
- "In terminal phase: verify destination tokens for exits."

### `safety_audit_approvals`

Audit all token approvals for the configured wallet. Identifies risky approvals (unlimited amounts, unrecognized spenders).

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Scanning Approval events", "Checking current allowances", "Classifying spender risk"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct AuditApprovalsParams {
    /// Chain ID.
    pub chain_id: u64,
    /// Include previously revoked approvals (default: false).
    #[serde(default)]
    pub include_revoked: bool,
}

#[derive(Debug, Serialize)]
pub struct ApprovalAuditResult {
    pub total_approvals: u32,
    pub risky_approvals: u32,
    pub approvals: Vec<ApprovalInfo>,
}

#[derive(Debug, Serialize)]
pub struct ApprovalInfo {
    pub token: String,
    pub token_symbol: String,
    pub spender: String,
    pub spender_label: Option<String>,
    pub allowance: String,
    pub is_unlimited: bool,
    pub risk_level: ApprovalRisk,
    pub last_used: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum ApprovalRisk {
    Safe,
    Elevated,
    Risky,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ total_approvals, risky_approvals }`

**promptSnippet**: "Audits all token approvals for the wallet. Identifies risky approvals: unlimited amounts, unrecognized spenders. Run periodically to reduce attack surface."

**promptGuidelines**:
- "In thriving phase: run weekly. Revoke any approval to an unrecognized spender."
- "In stable phase: run daily. Revoke all unlimited approvals."
- "In conservation phase: run once. Revoke everything except what is needed for immediate exits."
- "In terminal phase: revoke all remaining approvals after final exits complete."

### `safety_batch_revoke_approvals`

Revoke multiple risky approvals in a single multicall transaction.

**Capability**: `WriteTool` | **Risk tier**: `Layer1` (risk-reducing operation) | **Tick budget**: `Slow`
**Progress steps**: `["Building multicall", "Simulating in Revm fork", "Broadcasting revoke multicall"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct BatchRevokeApprovalsParams {
    /// Array of approvals to revoke.
    pub approvals: Vec<ApprovalTarget>,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalTarget {
    pub token: String,
    pub spender: String,
}

#[derive(Debug, Serialize)]
pub struct BatchRevokeResult {
    pub tx_hash: String,
    pub revoked_count: u32,
    pub failed: Vec<RevokeFailure>,
}

#[derive(Debug, Serialize)]
pub struct RevokeFailure {
    pub token: String,
    pub spender: String,
    pub reason: String,
}
```

**Ground truth**: Allowances set to zero for all targets. Source: `"erc20_allowance"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ revoked_count }`

**promptSnippet**: "Revokes multiple risky approvals in a single multicall. Use after `safety_audit_approvals` identifies risky approvals. Gas-efficient batch operation."

### `safety_detect_honeypot`

Test if a token is a honeypot by simulating buy and sell transactions in a Revm fork. Detects tokens that can be bought but not sold.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Forking chain state", "Simulating buy", "Simulating sell from bought balance", "Analyzing results"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct DetectHoneypotParams {
    /// Token address.
    pub token: String,
    /// Chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct HoneypotResult {
    pub token: String,
    pub is_honeypot: bool,
    pub buy_success: bool,
    pub sell_success: bool,
    pub buy_tax_bps: u32,
    pub sell_tax_bps: u32,
    pub max_sell_pct: f64,
    pub has_blacklist: bool,
    pub has_pause: bool,
    pub risk_summary: String,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ is_honeypot, sell_tax_bps }`

**promptSnippet**: "Tests if a token is a honeypot by simulating buy and sell in Revm fork. Detects tokens that can be bought but not sold. ALWAYS run before trading any unverified token."

### `safety_set_circuit_breaker`

Manually trigger or reset the circuit breaker. Triggering halts all write operations. Used by the `halt-agents` skill for emergency stops.

**Capability**: `WriteTool` | **Risk tier**: `Layer1` (the circuit breaker is itself a safety mechanism) | **Tick budget**: `Fast`
**Progress steps**: `["Updating circuit breaker state"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct SetCircuitBreakerParams {
    /// "trigger" (halt) or "reset" (resume).
    pub action: CircuitBreakerAction,
    /// "all", "chain:<chainId>", or "agent:<agentId>".
    pub scope: Option<String>,
    /// Reason for trigger/reset (audit trail).
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum CircuitBreakerAction {
    Trigger,
    Reset,
}

#[derive(Debug, Serialize)]
pub struct CircuitBreakerResult {
    pub action: String,
    pub scope: String,
    pub active: bool,
    pub reason: Option<String>,
    pub timestamp: String,
}
```

**Ground truth**: Circuit breaker state matches requested action. Source: `"circuit_breaker_store"`.

**Event Fabric**: `tool:start` -> `tool:end` -> `{ action, scope, active }`

**promptSnippet**: "Triggers or resets the circuit breaker. Trigger halts all write operations. Reset resumes. Audit-trailed."

**promptGuidelines**:
- "In thriving phase: trigger only in genuine emergencies: exploit detected, oracle manipulation, unexpected large losses."
- "In stable phase: lower threshold for triggering. Trigger if any safety check fails unexpectedly."
- "In conservation phase: keep circuit breaker ready. Trigger if losses accelerate."
- "In terminal phase: do not trigger unless necessary. You need to execute exit operations."

---

## Position health and monitoring (6 tools)

### `safety_get_risk_dashboard`

Single-call risk summary optimized for LLM consumption in heartbeat loops. Aggregates all risk metrics into a compact, actionable format. Primary entry point for the Golem heartbeat.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Scanning positions across protocols", "Computing health factors", "Ranking critical positions", "Generating action recommendations"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetRiskDashboardParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID, or 0 to scan all chains.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct RiskDashboard {
    pub account: String,
    pub timestamp: String,
    pub net_worth_usd: f64,
    pub total_debt_usd: f64,
    pub overall_risk: OverallRisk,
    pub critical_positions: Vec<CriticalPosition>,
    pub pending_actions: Vec<PendingAction>,
    /// One-line summary for LLM consumption.
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub enum OverallRisk {
    Safe,
    Warning,
    Danger,
}

#[derive(Debug, Serialize)]
pub struct CriticalPosition {
    pub protocol: String,
    pub health_factor: f64,
    pub urgency: Urgency,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub enum Urgency {
    Critical,
    High,
}

#[derive(Debug, Serialize)]
pub struct PendingAction {
    pub priority: u32,
    pub action: String,
    pub tool: String,
    pub estimated_cost: String,
}
```

**Heartbeat integration**: Called on every tick. If `overall_risk == Danger`, the heartbeat escalates to T2 (Opus) for autonomous action planning. If `critical_positions` contains any entry with `urgency: Critical`, the heartbeat triggers `auto_deleverage` or a full position close before returning to normal cadence.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "safety_get_risk_dashboard", params_hash }`
- `tool:update` -> `"Scanning positions across protocols"` -> `"Computing health factors"` -> `"Ranking critical positions"` -> `"Generating action recommendations"`
- `tool:end` -> `{ overall_risk, critical_count, net_worth_usd }`

**promptSnippet**: "Single-call risk summary for heartbeat loops. Returns critical positions, pending actions sorted by priority, one-line summary. Call on EVERY heartbeat tick when any leveraged position is active."

**promptGuidelines**:
- "In thriving phase: log summary and continue if overall_risk is Safe. Investigate if Warning."
- "In stable phase: act on any pending_action with priority 1 or 2. Increase monitoring."
- "In conservation phase: execute ALL pending_actions immediately. Close all critical positions."
- "In declining phase: execute remaining pending_actions."
- "In terminal phase: should be clean (no positions). If not, execute remaining actions."

### `safety_get_portfolio_health`

Get all DeFi positions and overall portfolio health for an account across all integrated protocols.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Scanning lending positions", "Scanning LP positions", "Scanning derivatives", "Computing portfolio risk"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetPortfolioHealthParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID, or 0 to scan all chains.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PortfolioHealth {
    pub account: String,
    pub timestamp: String,
    pub total_value_usd: f64,
    pub collateral_usd: f64,
    pub debt_usd: f64,
    pub net_worth_usd: f64,
    pub positions: Vec<PositionHealth>,
    pub overall_risk: OverallRisk,
    pub top_risks: Vec<RiskItem>,
}

#[derive(Debug, Serialize)]
pub struct PositionHealth {
    pub protocol: String,
    pub position_type: PositionType,
    pub value_usd: f64,
    pub debt_usd: Option<f64>,
    pub health_factor: Option<f64>,
    pub risk_level: PositionRisk,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub enum PositionType {
    Lending,
    Loop,
    Cdp,
    Perp,
    Option,
    Lp,
    Staking,
    Yield,
}

#[derive(Debug, Serialize)]
pub enum PositionRisk {
    Safe,
    Warning,
    Danger,
    None,
}

#[derive(Debug, Serialize)]
pub struct RiskItem {
    pub protocol: String,
    pub description: String,
    pub urgency: RiskUrgency,
}

#[derive(Debug, Serialize)]
pub enum RiskUrgency {
    Low,
    Medium,
    High,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ overall_risk, position_count, top_risk_count }`

**promptSnippet**: "Returns all DeFi positions and overall portfolio health across integrated protocols. Pass chain_id: 0 to scan all chains. Read-only aggregate view."

**promptGuidelines**:
- "In thriving phase: check periodically. Use top_risks to prioritize which positions need attention."
- "In stable phase: check every tick. Act on any position with risk_level Warning or worse."
- "In conservation/declining phase: check every tick. Feed into unwind planning."
- "In terminal phase: final portfolio snapshot for Death Protocol."

### `safety_get_liquidation_risks`

Get all positions across protocols that are at risk of liquidation, sorted by urgency.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Scanning lending positions", "Computing liquidation distances", "Estimating time to liquidation"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetLiquidationRisksParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID, or 0 to scan all chains.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct LiquidationRisks {
    pub account: String,
    pub risky_positions: Vec<LiquidationRiskPosition>,
    pub total_at_risk_usd: f64,
    pub most_urgent: Option<MostUrgent>,
}

#[derive(Debug, Serialize)]
pub struct LiquidationRiskPosition {
    pub protocol: String,
    pub position_type: String,
    pub health_factor: f64,
    pub time_to_liquidation: Option<String>,
    pub collateral_usd: f64,
    pub debt_usd: f64,
    pub liquidation_price: Option<f64>,
    pub urgency: LiquidationUrgency,
    pub recommended_action: String,
}

#[derive(Debug, Serialize)]
pub enum LiquidationUrgency {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize)]
pub struct MostUrgent {
    pub protocol: String,
    pub urgency: LiquidationUrgency,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ risky_position_count, total_at_risk_usd }`

**promptSnippet**: "Returns all positions at risk of liquidation, sorted by urgency. Critical = HF < 1.1; High = 1.1-1.3; Medium = 1.3-1.5. Includes recommended actions. Read-only."

**promptGuidelines**:
- "In thriving phase: check after large market moves. Act on High and Critical."
- "In stable phase: check every tick. Act on Medium and above."
- "In conservation/declining phase: check every tick. Close all positions with any urgency level."
- "In terminal phase: should return empty (all positions closed)."

### `safety_get_protocol_exposure`

Get portfolio concentration risk by protocol, chain, and asset type.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Aggregating positions by protocol", "Computing chain distribution", "Classifying asset types"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetProtocolExposureParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID, or 0 to scan all chains.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ProtocolExposureResult {
    pub by_protocol: Vec<ProtocolExposure>,
    pub by_chain: Vec<ChainExposure>,
    pub by_asset_type: Vec<AssetTypeExposure>,
    pub concentration_risk: ConcentrationRisk,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProtocolExposure {
    pub protocol: String,
    pub value_usd: f64,
    pub pct: f64,
    pub debt_usd: f64,
    pub net_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct ChainExposure {
    pub chain: String,
    pub value_usd: f64,
    pub pct: f64,
}

#[derive(Debug, Serialize)]
pub struct AssetTypeExposure {
    pub asset_type: AssetType,
    pub value_usd: f64,
    pub pct: f64,
}

#[derive(Debug, Serialize)]
pub enum AssetType {
    Stablecoin,
    Eth,
    Btc,
    Lst,
    Lrt,
    DefiToken,
    Other,
}

#[derive(Debug, Serialize)]
pub enum ConcentrationRisk {
    Low,
    Medium,
    High,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ concentration_risk, protocol_count }`

**promptSnippet**: "Returns portfolio concentration risk by protocol, chain, and asset type. Flags high concentration. Read-only."

**promptGuidelines**:
- "In thriving phase: review periodically. Diversify if any single protocol exceeds 40% of portfolio."
- "In stable phase: check before any new position. Avoid increasing concentration."
- "In conservation phase: use to prioritize which protocols to unwind first (most concentrated first)."

### `safety_simulate_price_stress`

Model portfolio impact under hypothetical ETH price scenarios. Returns the maximum drawdown the portfolio survives before the first liquidation.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Loading portfolio positions", "Running price scenarios in Revm fork", "Computing liquidation thresholds", "Generating hedge recommendations"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct SimulatePriceStressParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID.
    pub chain_id: u64,
    /// Price change percentages to model (default: [-20, -40, -60]).
    pub price_scenarios: Option<Vec<f64>>,
}

#[derive(Debug, Serialize)]
pub struct PriceStressResult {
    pub scenarios: Vec<StressScenario>,
    /// Maximum drawdown percentage the portfolio survives.
    pub max_drawdown_survivable: f64,
    pub hedge_recommendation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StressScenario {
    pub price_change_pct: f64,
    pub portfolio_value_usd: f64,
    pub debt_usd: f64,
    pub net_worth_usd: f64,
    pub liquidated_positions: Vec<LiquidatedPosition>,
    pub total_loss_usd: f64,
    pub survives: bool,
}

#[derive(Debug, Serialize)]
pub struct LiquidatedPosition {
    pub protocol: String,
    pub health_factor_at_scenario: f64,
    pub loss_usd: f64,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ max_drawdown_survivable, scenario_count }`

**promptSnippet**: "Models portfolio impact under ETH price drops (-20%, -40%, -60% by default). Returns the max drawdown the portfolio survives before first liquidation. Read-only."

**promptGuidelines**:
- "In thriving phase: run before opening leveraged positions. Ensure max_drawdown_survivable > 30%."
- "In stable phase: run every tick. If max_drawdown_survivable < 20%, begin deleveraging."
- "In conservation phase: run to confirm all positions are safe during unwind."

### `safety_estimate_unwind_cost`

Estimate the total cost (gas + slippage) to close all positions in a category or protocol. Returns the optimal unwind order with dependency resolution.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Loading positions by category", "Simulating unwind in Revm fork", "Computing gas and slippage", "Determining optimal order"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct EstimateUnwindCostParams {
    /// Wallet address.
    pub account: String,
    /// Position category to unwind.
    pub category: UnwindCategory,
    /// Chain ID.
    pub chain_id: u64,
    /// Urgent uses a higher gas price estimate (default: Normal).
    #[serde(default)]
    pub urgency: UnwindUrgency,
}

#[derive(Debug, Serialize)]
pub enum UnwindCategory {
    Lending,
    Loops,
    Perps,
    Options,
    All,
}

#[derive(Debug, Serialize)]
pub enum UnwindUrgency {
    Normal,
    Urgent,
}

#[derive(Debug, Serialize)]
pub struct UnwindCostResult {
    pub category: String,
    pub positions: Vec<UnwindPosition>,
    pub total_unwind_cost_usd: f64,
    pub estimated_total_time: String,
    pub optimal_order: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UnwindPosition {
    pub protocol: String,
    pub unwind_cost_usd: f64,
    pub steps: u32,
    pub estimated_time: String,
}
```

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ total_unwind_cost_usd, position_count }`

**promptSnippet**: "Estimates total cost (gas + slippage) to close all positions in a category. Returns optimal unwind order. Read-only. Use before risk-driven exits."

**promptGuidelines**:
- "In thriving phase: use to evaluate cost of exiting underperforming strategies."
- "In conservation phase: run for all categories to understand total exit cost budget."
- "In declining phase: run with urgency: Urgent for accurate gas estimates. Execute optimal order."
- "In terminal phase: run for All category. Execute full unwind."

---

## Alerting (4 tools)

Alert configurations persist in the Golem's Grimoire under the `health_alerts` namespace, surviving heartbeat ticks and Golem restarts. The heartbeat calls `safety_check_pending_alerts` on every tick regardless of the `safety_get_risk_dashboard` result.

### `safety_set_health_alert`

Configure a health factor alert for a lending position.

**Capability**: `WriteTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Validating protocol", "Storing alert in Grimoire"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct SetHealthAlertParams {
    /// Protocol to monitor (e.g., "aave_v3", "morpho_blue").
    pub protocol: String,
    /// Alert trigger threshold (e.g., 1.5).
    pub min_health_factor: f64,
    /// Chain ID.
    pub chain_id: u64,
    /// Store alert in Grimoire for heartbeat pickup (default: true).
    #[serde(default = "default_true")]
    pub notify_on_breach: bool,
    /// Automated response on breach (default: None).
    #[serde(default)]
    pub auto_action: AutoAction,
    /// Custom ID for idempotent updates; auto-generated if omitted.
    pub alert_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum AutoAction {
    None,
    Deleverage,
    Close,
}

#[derive(Debug, Serialize)]
pub struct HealthAlertSet {
    pub alert_id: String,
    pub protocol: String,
    pub min_health_factor: f64,
    pub chain_id: u64,
    pub active: bool,
    pub created_at: String,
}
```

**Ground truth**: Alert stored in Grimoire's `health_alerts` namespace. Source: `"grimoire_health_alerts"`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ alert_id, min_health_factor }`

**promptSnippet**: "Configures a health factor alert for a lending position. Persists in Grimoire. Optional auto_action: Deleverage or Close on breach. Phase-gated: new alerts blocked in terminal."

**promptGuidelines**:
- "In thriving phase: set alerts at HF 1.5 for lending positions. Use auto_action: None -- let the Golem decide."
- "In stable phase: tighten thresholds to HF 1.8. Set auto_action: Deleverage."
- "In conservation phase: set auto_action: Close for all remaining positions."
- "In terminal phase: do not create new alerts. Existing alerts continue."

### `safety_get_health_alerts`

List active alert configurations for an account.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Reading alerts from Grimoire"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct GetHealthAlertsParams {
    /// Wallet address.
    pub account: String,
    /// Filter by chain.
    pub chain_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct HealthAlertsResult {
    pub alerts: Vec<HealthAlert>,
}

#[derive(Debug, Serialize)]
pub struct HealthAlert {
    pub alert_id: String,
    pub protocol: String,
    pub min_health_factor: f64,
    pub chain_id: u64,
    pub notify_on_breach: bool,
    pub auto_action: AutoAction,
    pub last_triggered: Option<String>,
}
```

**Event Fabric**: `tool:start` -> `tool:end` -> `{ alert_count }`

**promptSnippet**: "Lists active alert configurations. Read-only. Check to avoid creating duplicate alerts."

### `safety_check_pending_alerts`

Check whether any configured alerts have been breached. Called by the heartbeat on every tick.

**Capability**: `ReadTool` | **Risk tier**: `Layer1` | **Tick budget**: `Medium`
**Progress steps**: `["Loading alert configs", "Querying protocol health factors", "Evaluating thresholds"]`
**Sprite trigger**: `Thinking`

```rust
#[derive(Debug, Deserialize)]
pub struct CheckPendingAlertsParams {
    /// Wallet address.
    pub account: String,
    /// Chain ID, or 0 to check all chains.
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct PendingAlertsResult {
    pub breached_alerts: Vec<BreachedAlert>,
    pub all_clear: bool,
}

#[derive(Debug, Serialize)]
pub struct BreachedAlert {
    pub alert_id: String,
    pub protocol: String,
    pub current_health_factor: f64,
    pub threshold: f64,
    /// How far below threshold (absolute).
    pub breach: f64,
    pub auto_action: AutoAction,
    pub recommended_tool: String,
    pub recommended_params: serde_json::Value,
}
```

If `all_clear: false`, the heartbeat logs the breach and executes `auto_action` if it is not `None`.

**Event Fabric**: `tool:start` -> steps -> `tool:end` -> `{ all_clear, breached_count }`

**promptSnippet**: "Checks whether configured alerts have been breached. Called by heartbeat on EVERY tick regardless of risk dashboard result. Returns breached alerts with recommended tools and params."

**promptGuidelines**:
- "In thriving phase: log breaches, evaluate recommended actions."
- "In stable phase: execute auto_action if set. Otherwise, act on all breaches immediately."
- "In conservation/declining phase: execute all auto_actions. Close breached positions."
- "In terminal phase: execute remaining auto_actions for settlement."

### `safety_clear_health_alert`

Remove a health alert configuration.

**Capability**: `WriteTool` | **Risk tier**: `Layer1` | **Tick budget**: `Fast`
**Progress steps**: `["Removing alert from Grimoire"]`
**Sprite trigger**: `Executing`

```rust
#[derive(Debug, Deserialize)]
pub struct ClearHealthAlertParams {
    /// Alert ID to clear.
    pub alert_id: String,
}

#[derive(Debug, Serialize)]
pub struct ClearAlertResult {
    pub success: bool,
    pub alert_id: String,
}
```

**Ground truth**: Alert removed from Grimoire. Source: `"grimoire_health_alerts"`.

**Event Fabric**: `tool:start` -> `tool:end` -> `{ alert_id }`

**promptSnippet**: "Removes a health alert configuration. Use when the underlying position is closed."

---

## Cross-protocol risk normalization

Each protocol exposes a different health metric. Safety tools normalize all of them to a unified risk scale so the Golem can reason about a mixed portfolio without protocol-specific logic in its action planner.

| Protocol | Native metric | Safe | Warning | Danger |
| --- | --- | --- | --- | --- |
| Aave V3 | Health factor | HF > 1.5 | HF 1.2-1.5 | HF < 1.2 |
| Compound V3 | Collateralization | > 120% MCR | 105-120% MCR | < 105% MCR |
| Morpho Blue | LTV vs LLTV | LTV < 80% LLTV | LTV 80-90% LLTV | LTV > 90% LLTV |
| GMX v2 | Leverage vs max | < 70% max | 70-85% max | > 85% max |
| crvUSD LLAMMA | LLAMMA health | > 0.2 | 0.0-0.2 | < 0.0 (soft liq) |
| Leverage loop | Reconstructed HF | HF > 1.5 | HF 1.2-1.5 | HF < 1.2 |

Loop positions do not have a native health factor. The tools reconstruct one by treating the full supply position as collateral and the full borrow as debt, applying the protocol's liquidation threshold. The result is comparable to Aave's health factor scale.

---

## Heartbeat integration pattern

```text
Every tick:
  1. Call safety_get_risk_dashboard(account, chain_id: 0)
  2. If overall_risk == Safe: log summary and continue
  3. If overall_risk == Warning: log, increase monitoring frequency
  4. If overall_risk == Danger or critical_positions is non-empty:
     a. Escalate to T2 (Opus)
     b. Execute recommended actions from pending_actions
     c. Call safety_check_pending_alerts for auto_action execution
     d. Record Episode in Grimoire with reasoning chain
  5. Call safety_check_pending_alerts(account, chain_id: 0)
     -- runs regardless of step 2-4 outcome
```

---

## Adapter registry mapping

| Pi-facing tool | Action type | Internal ToolDef |
| --- | --- | --- |
| `preview_action` | `simulate_tx` | `safety_simulate_transaction` |
| `preview_action` | `revoke_approvals` | `safety_batch_revoke_approvals` (simulate) |
| `commit_action` | `revoke_approvals` | `safety_batch_revoke_approvals` (execute) |
| `commit_action` | `circuit_breaker` | `safety_set_circuit_breaker` |
| `query_state` | `risk_dashboard` | `safety_get_risk_dashboard` |
| `query_state` | `portfolio_health` | `safety_get_portfolio_health` |
| `query_state` | `liquidation_risks` | `safety_get_liquidation_risks` |
| `query_state` | `pending_alerts` | `safety_check_pending_alerts` |
| `search_context` | `token_validation` | `safety_validate_token` |
| `search_context` | `honeypot_check` | `safety_detect_honeypot` |
| `search_context` | `approval_audit` | `safety_audit_approvals` |
| `search_context` | `price_stress` | `safety_simulate_price_stress` |
| `search_context` | `unwind_cost` | `safety_estimate_unwind_cost` |

---

## Custody implications (safety write tools)

The 4 write tools in this file (`safety_set_alert`, `safety_delete_alert`, `safety_acknowledge_alert`, `safety_set_circuit_breaker`) modify off-chain safety configuration, not on-chain state. Custody mode affects them as follows:

- **Delegation**: No on-chain signing needed. Safety configuration is stored in the Grimoire (local) and validated by PolicyCage.
- **Embedded**: Same — off-chain config stored locally.
- **Local Key**: Same behavior. The `safety_set_circuit_breaker` tool is Privileged and requires owner approval regardless of custody mode.

---

## Error taxonomy

| Code | Description |
| --- | --- |
| `ACCOUNT_NOT_FOUND` | No positions found for the given account on the specified chain |
| `PROTOCOL_NOT_SUPPORTED` | Protocol is not integrated for health monitoring |
| `ALERT_NOT_FOUND` | `alert_id` does not match any active alert configuration |
| `ALERT_ALREADY_EXISTS` | An alert for the same protocol and chain already exists; use `alert_id` to update |
| `CHAIN_REQUIRED` | `chain_id: 0` (all chains) is not supported for this tool; specify a chain ID |
| `SUBGRAPH_UNAVAILABLE` | Historical data unavailable; position history requires subgraph access |
| `STRESS_SCENARIO_INVALID` | Price scenario out of range; values must be between -99 and +500 |
| `REVM_FORK_FAILED` | Could not create Revm fork at current block; RPC may be unavailable |
| `SIMULATION_TIMEOUT` | Revm simulation exceeded tick budget; transaction may be too complex |
