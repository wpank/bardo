# Bardo Tools -- Trading and approval tools [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Swap execution, UniswapX orders, cross-chain intents, and approval management. 5 core tools. For LP tools see [04-tools-lp.md](04-tools-lp.md) (concentrated liquidity position management, fee collection, migration, and optimization).

---

> **Reader orientation:** This document specifies the 5 core trading tools in `bardo-tools`: swap execution, UniswapX Dutch auction orders, cross-chain intents (ERC-7683), and token approval management. All write tools pass through the full safety hook chain (allowlist, spending limits, Revm simulation) before execution. You should be familiar with Uniswap swap mechanics, Permit2, and UniswapX order types. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Trading tools (5)

### `uniswap_get_quote`

Get a price quote for a swap without executing. Returns expected output, price impact, gas estimate, and recommended route. Does not modify state.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_in` | `String` | Yes | Input token symbol or 0x-prefixed address |
| `token_out` | `String` | Yes | Output token symbol or 0x-prefixed address |
| `amount` | `String` | Yes | Input amount (human-readable, e.g., "500") |
| `chain` | `String` | Yes | Chain name or chain ID |
| `slippage_bps` | `u32` | No | Slippage tolerance in basis points (default: 50) |
| `prefer_uniswapx` | `bool` | No | Prefer UniswapX Dutch auction routing (default: true) |
| `exact_output` | `bool` | No | Quote for exact output amount instead of exact input (default: false) |

```rust
#[derive(Debug, Deserialize)]
pub struct GetQuoteParams {
    /// Input token symbol (e.g., "WETH") or 0x-prefixed address.
    pub token_in: String,
    /// Output token symbol or address.
    pub token_out: String,
    /// Human-readable amount (e.g., "500"). Converted to raw units internally.
    pub amount: String,
    /// Chain name ("ethereum", "base") or chain ID ("1", "8453").
    pub chain: String,
    /// Slippage tolerance in basis points. Default: 50 (0.5%).
    #[serde(default = "default_slippage_50")]
    pub slippage_bps: u32,
    /// Prefer UniswapX Dutch auction routing. Default: true.
    #[serde(default = "default_true")]
    pub prefer_uniswapx: bool,
    /// Quote for exact output amount. Default: false (exact input).
    #[serde(default)]
    pub exact_output: bool,
}

fn default_slippage_50() -> u32 { 50 }
fn default_true() -> bool { true }

#[derive(Debug, Serialize)]
pub struct RouteHop {
    pub pool: String,
    pub token_in: String,
    pub token_out: String,
    pub fee_tier: u32,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct QuoteResult {
    pub quote_id: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: String,
    pub amount_out: String,
    pub price_impact_pct: f64,
    pub gas_estimate_usd: f64,
    pub route: Vec<RouteHop>,
    /// CLASSIC, DUTCH_V2, DUTCH_V3, or PRIORITY.
    pub route_type: String,
    pub deadline: u64,
    pub permit_data: Option<serde_json::Value>,
}
```

**Data source**: Uniswap Trading API (if `BARDO_UNISWAP_API_KEY` set), otherwise TypeScript sidecar running smart-order-router via Unix socket IPC.

| Field | Value |
| --- | --- |
| `capability` | `CapabilityTier::Read` |
| `risk_tier` | `RiskTier::Layer1` |
| `tick_budget` | `TickBudget::Medium` (1-5s) |
| `progress_steps` | `["Resolving tokens", "Computing route", "Estimating gas"]` |
| `sprite_trigger` | `SpriteTrigger::Thinking` |
| `category` | `Category::Trading` |

**TUI rendering**: `QuoteCard` -- shows input/output amounts, price impact badge (green < 0.5%, yellow 0.5-2%, red > 2%), route visualization as hop chain, and route type badge.

**promptSnippet**:
> Returns expected output, price impact, gas estimate, and recommended route. Does NOT execute. Safe to call freely for price discovery and pre-trade planning.

**promptGuidelines**:
- **thriving**: Call freely. Use to compare routes and evaluate trade timing.
- **cautious**: Call before every planned trade. Reject quotes with price impact > 2%.
- **declining**: Call only for exit routes. Compare UniswapX vs Classic for best exit pricing.
- **terminal**: Call for exit pricing only. Accept any reasonable quote for immediate exits.

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components):
- `tool:start` -> `{ tool_name: "uniswap_get_quote", params: { token_in, token_out, amount, chain } }`
- `tool:update` -> `{ step: "Computing route" }`
- `tool:end` -> `{ success: true, result_summary: { amount_out, route_type, price_impact_pct } }`

**Pi hooks**: `tool_call` (rate-limit check), `tool_result` (no filtering)

---

### `uniswap_execute_swap`

Execute a token swap on Uniswap. Passes through the full safety hook chain: token allowlist, spending limit, rate limit, balance check, slippage guard, Revm (Rust EVM implementation for local transaction simulation against forked chain state) fork simulation, nonce management. Any check fails -> structured error, no signature.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_in` | `String` | Yes | Input token symbol or 0x-prefixed address |
| `token_out` | `String` | Yes | Output token symbol or 0x-prefixed address |
| `amount` | `String` | Yes | Input amount (human-readable) |
| `chain` | `String` | Yes | Chain name or chain ID |
| `slippage_bps` | `u32` | No | Slippage tolerance in bps (default: 50) |
| `deadline` | `u64` | No | Transaction deadline in seconds (default: 300) |
| `prefer_uniswapx` | `bool` | No | Prefer UniswapX (default: true) |

```rust
#[derive(Debug, Deserialize)]
pub struct ExecuteSwapParams {
    pub token_in: String,
    pub token_out: String,
    pub amount: String,
    pub chain: String,
    #[serde(default = "default_slippage_50")]
    pub slippage_bps: u32,
    #[serde(default = "default_deadline_300")]
    pub deadline: u64,
    #[serde(default = "default_true")]
    pub prefer_uniswapx: bool,
}

fn default_deadline_300() -> u64 { 300 }

#[derive(Debug, Serialize)]
pub struct SwapResult {
    pub tx_hash: String,
    pub block_number: u64,
    pub gas_used: u64,
    pub amount_in: String,
    pub amount_out: String,
    pub effective_price: f64,
    pub price_impact_pct: f64,
    pub slippage_actual_bps: i32,
    pub explorer_url: String,
}
```

| Field | Value |
| --- | --- |
| `capability` | `CapabilityTier::Write` -- requires `Capability<ExecuteSwap>` token |
| `risk_tier` | `RiskTier::Layer3` -- unbounded write, full ActionPermit pipeline |
| `tick_budget` | `TickBudget::Slow` (5-15s) |
| `progress_steps` | `["Simulating via Revm", "Approving token", "Building calldata", "Signing transaction", "Broadcasting", "Confirming receipt"]` |
| `sprite_trigger` | `SpriteTrigger::Executing` (transitions to `Success` or `Failure` on completion) |
| `category` | `Category::Trading` |

**Safety pipeline**: Token allowlist check -> spending limit check -> rate limit check -> balance circuit breaker -> slippage guard -> pre-flight Revm fork simulation -> nonce check. ANY check fails -> return structured error, do not sign.

**Pre-flight simulation**: Uses `ctx.revm_fork(chain_id)` to fork current chain state and simulate the swap transaction locally. The Revm fork catches reverts, estimates gas, and validates output amounts before signing. Revm fork is deterministic and does not leak transaction details to the RPC provider.

**Custody implications**:
- **Delegation**: Signs via session key scoped to swap targets. CaveatEnforcer validates swap router + spending limit on-chain before the tx executes. Session key expiry blocks post-mortem swaps.
- **Embedded**: Privy server wallet signs. Spending limits enforced by PolicyCage (the on-chain smart contract that enforces an owner's safety constraints; here checked off-chain) only. Owner retains Privy recovery.
- **Local Key**: Signs with local keypair. On-chain delegation bounds constrain allowed targets and amounts identically to Delegation mode.

**Memory-augmented execution**: When the `learning` profile is active, the handler retrieves similar past swaps from Grimoire's (the Golem's persistent memory system with episodic, semantic, and strategy layers) episodic memory and relevant insights from semantic memory. Memory adjusts SOFT parameters only (slippage tolerance, timing recommendations, route preferences). Memory CANNOT override safety limits, token allowlist, spending caps, or simulation requirements. After execution, a Reflexion self-reflection is stored as a new episode.

**TUI rendering**: `SwapProgress` -- multi-step progress bar showing each safety check and execution step. On completion, shows before/after balance delta and slippage vs. quoted comparison.

**promptSnippet**:
> Executes an atomic token swap via Uniswap. ALWAYS call uniswap_get_quote first. In cautious/declining phase, reduce size by 50%. In terminal phase, only exit swaps (selling to stables or ETH) are allowed.

**promptGuidelines**:
- **thriving**: Execute at full planned size. Use UniswapX for MEV protection on trades > $1K. Call intel_assess_mev_risk for trades > $10K.
- **cautious**: Reduce trade size by 50% from planned amount. Tighten slippage to 30 bps. Require price impact < 1%.
- **declining**: Only exit swaps (sell risky tokens for stables/ETH). Maximum 25% of position per trade. No new entries.
- **terminal**: Exit-only. Sell everything to USDC or ETH. Accept up to 200 bps slippage for immediate execution. No position-building swaps.
- **memory**: When learning profile is active, similar past swap outcomes adjust soft parameters. Memory cannot override safety limits.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "uniswap_execute_swap", params: { token_in, token_out, amount, chain } }`
- `tool:update` -> `{ step: "Simulating via Revm" }` | `{ step: "Signing transaction" }` | `{ step: "Broadcasting" }` | `{ step: "Confirming receipt" }`
- `tool:end` -> `{ success: true, result_summary: { tx_hash, amount_out, slippage_actual_bps } }`
- `tool:error` -> `{ error_code, error_message }` (on any safety check failure or revert)

**Pi hooks**: `tool_call` (full safety pipeline: allowlist -> spending limit -> rate limit -> balance check -> slippage guard -> simulation), `tool_result` (execution quality metrics logged, Grimoire episode stored if learning active)

**Ground truth verification**:
- **expected_outcome**: `"Swap {amount_in} {token_in} for >= {min_amount_out} {token_out}"` (computed from quote minus slippage)
- **actual_outcome**: `"Received {amount_out} {token_out}, tx {tx_hash}"` (from transaction receipt)
- **ground_truth_source**: `"transaction_receipt + balance_diff"` -- post-swap balance delta compared against pre-swap balance to detect discrepancies

---

### `uniswap_submit_uniswapx_order`

Submit a UniswapX Dutch auction order for MEV-protected execution. The order decays from a start amount to an end amount over the decay window. Fillers compete to fill at the best price. Gasless for the Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM).

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_in` | `String` | Yes | Input token symbol or address |
| `token_out` | `String` | Yes | Output token symbol or address |
| `amount` | `String` | Yes | Input amount |
| `chain` | `String` | Yes | Chain name or chain ID |
| `decay_start_time` | `u64` | No | Seconds until Dutch auction decay starts (default: 10) |
| `decay_end_time` | `u64` | No | Seconds until order expires (default: 120) |

```rust
#[derive(Debug, Deserialize)]
pub struct SubmitUniswapXOrderParams {
    pub token_in: String,
    pub token_out: String,
    pub amount: String,
    pub chain: String,
    #[serde(default = "default_10")]
    pub decay_start_time: u64,
    #[serde(default = "default_120")]
    pub decay_end_time: u64,
}

fn default_10() -> u64 { 10 }
fn default_120() -> u64 { 120 }

#[derive(Debug, Serialize)]
pub struct UniswapXOrderResult {
    pub order_hash: String,
    /// DUTCH_V2, DUTCH_V3, or PRIORITY.
    pub order_type: String,
    /// pending, open, filled, expired, cancelled.
    pub status: String,
    pub deadline: u64,
    pub start_amount: String,
    pub end_amount: String,
}
```

| Field | Value |
| --- | --- |
| `capability` | `CapabilityTier::Write` -- requires `Capability<SubmitUniswapXOrder>` |
| `risk_tier` | `RiskTier::Layer2` -- bounded write (order can expire unfilled, no on-chain cost) |
| `tick_budget` | `TickBudget::Medium` (1-5s) |
| `progress_steps` | `["Validating order", "Signing Permit2", "Submitting to filler network"]` |
| `sprite_trigger` | `SpriteTrigger::Executing` |
| `category` | `Category::Trading` |

**Custody implications**:
- **Delegation**: Signs Permit2 via session key. CaveatEnforcer validates token + amount. Gasless -- no ETH required in the session account.
- **Embedded**: Privy wallet signs Permit2. PolicyCage enforces spending limits.
- **Local Key**: Local keypair signs Permit2. On-chain delegation bounds apply.

**TUI rendering**: `OrderCard` -- shows order hash, decay curve visualization (start amount -> end amount over time), and status badge. Updates in real-time via `uniswap_get_uniswapx_order_status`.

**promptSnippet**:
> Submit a UniswapX Dutch auction order for MEV-protected off-chain execution. Gasless. Preferred for trades > $1K on mainnet. Monitor fill status with uniswap_get_uniswapx_order_status.

**promptGuidelines**:
- **thriving**: Prefer over Classic routing for trades > $1K. Dutch auction typically yields better execution.
- **cautious**: Same preference, but reduce decay window (shorter auction = less price risk).
- **declining**: Use for exit swaps only. Set aggressive decay_end_time (60s) for fast fills.
- **terminal**: Use if fill speed is acceptable. Otherwise fall back to uniswap_execute_swap for immediate on-chain execution.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "uniswap_submit_uniswapx_order", params: { token_in, token_out, amount, chain } }`
- `tool:update` -> `{ step: "Signing Permit2" }` | `{ step: "Submitting to filler network" }`
- `tool:end` -> `{ success: true, result_summary: { order_hash, order_type, status } }`

**Pi hooks**: `tool_call` (allowlist + spending limit + rate limit), `tool_result` (order hash logged for tracking)

**Ground truth verification**:
- **expected_outcome**: `"UniswapX order submitted for {amount} {token_in}, decay {start_amount} -> {end_amount}"`
- **actual_outcome**: `"Order {order_hash} status: {status}"` (queried via `uniswap_get_uniswapx_order_status`)
- **ground_truth_source**: `"uniswapx_api_order_status"` -- poll until terminal state

---

### `uniswap_check_allowance`

Check token approval allowance for the Universal Router or Permit2. Read-only -- does not modify state. Call before any swap or LP operation to determine if an approval is needed.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token` | `String` | Yes | Token symbol or address |
| `spender` | `String` | No | Spender address (default: Universal Router) |
| `chain` | `String` | Yes | Chain name or chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct CheckAllowanceParams {
    /// Token symbol or 0x-prefixed address.
    pub token: String,
    /// Spender address. Defaults to Universal Router for the chain.
    pub spender: Option<String>,
    pub chain: String,
}

#[derive(Debug, Serialize)]
pub struct AllowanceResult {
    pub token: String,
    pub spender: String,
    /// Current allowance in raw token units.
    pub allowance: String,
    /// Whether the allowance is sufficient for common operations.
    pub sufficient: bool,
    /// Whether an approval transaction is needed.
    pub needs_approval: bool,
}
```

**Alloy reads**:

```rust
sol! {
    function allowance(address owner, address spender) external view returns (uint256);
}
```

| Field | Value |
| --- | --- |
| `capability` | `CapabilityTier::Read` |
| `risk_tier` | `RiskTier::Layer1` |
| `tick_budget` | `TickBudget::Fast` (<1s) |
| `progress_steps` | `["Resolving token", "Reading allowance"]` |
| `sprite_trigger` | `SpriteTrigger::Thinking` |
| `category` | `Category::Trading` |

**TUI rendering**: `AllowanceChip` -- inline badge showing "Approved" (green) or "Needs approval" (yellow) with spender label.

**promptSnippet**:
> Check token approval allowance for Universal Router or Permit2. Call before any swap or LP operation to determine if an approval transaction is needed. Read-only.

**promptGuidelines**:
- **thriving**: Check before trades. Batch approvals with Permit2 where possible.
- **cautious**: Always check. Verify the spender address matches expected contracts.
- **declining**: Check before exit operations only.
- **terminal**: Check before exits. Approve only what is needed for the immediate operation.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "uniswap_check_allowance", params: { token, spender, chain } }`
- `tool:end` -> `{ success: true, result_summary: { needs_approval, allowance } }`

**Pi hooks**: `tool_call` (rate-limit), `tool_result` (no filtering)

---

### `uniswap_submit_cross_chain_intent`

Submit an ERC-7683 cross-chain intent for cross-chain swaps via Uniswap's filler network. Assets leave the source chain and arrive on the destination chain after fillers compete to fill the intent.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_in` | `String` | Yes | Input token symbol or address |
| `token_out` | `String` | Yes | Output token symbol or address |
| `amount` | `String` | Yes | Input amount |
| `source_chain` | `String` | Yes | Source chain name or ID |
| `destination_chain` | `String` | Yes | Destination chain name or ID |
| `deadline` | `u64` | No | Intent deadline in seconds (default: 3600) |

```rust
#[derive(Debug, Deserialize)]
pub struct SubmitCrossChainIntentParams {
    pub token_in: String,
    pub token_out: String,
    pub amount: String,
    pub source_chain: String,
    pub destination_chain: String,
    #[serde(default = "default_deadline_3600")]
    pub deadline: u64,
}

fn default_deadline_3600() -> u64 { 3600 }

#[derive(Debug, Serialize)]
pub struct FillerBid {
    pub filler: String,
    pub output_amount: String,
    pub estimated_time: String,
}

#[derive(Debug, Serialize)]
pub struct CrossChainIntentResult {
    pub intent_id: String,
    /// pending, filling, filled, expired.
    pub status: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub estimated_output: String,
    pub filler_bids: Vec<FillerBid>,
}
```

**Alloy writes**: Signs ERC-7683 `GaslessCrossChainOrder` struct via Alloy's `sol!` ABI encoding, submitted to the filler network API.

```rust
sol! {
    struct GaslessCrossChainOrder {
        address originSettler;
        address user;
        uint256 nonce;
        uint256 originChainId;
        uint32 openDeadline;
        uint32 fillDeadline;
        bytes32 orderDataType;
        bytes orderData;
    }
}
```

| Field | Value |
| --- | --- |
| `capability` | `CapabilityTier::Write` -- requires `Capability<SubmitCrossChainIntent>` |
| `risk_tier` | `RiskTier::Layer3` -- unbounded write, cross-chain risk |
| `tick_budget` | `TickBudget::Slow` (5-15s) |
| `progress_steps` | `["Validating chains", "Signing intent", "Submitting to filler network", "Awaiting initial bids"]` |
| `sprite_trigger` | `SpriteTrigger::Executing` |
| `category` | `Category::Trading` |

**Custody implications**:
- **Delegation**: Signs ERC-7683 order via session key. CaveatEnforcer must whitelist the origin settler contract on the source chain. Cross-chain fills are gasless for the signer.
- **Embedded**: Privy wallet signs. PolicyCage validates source chain allowlist + spending limit.
- **Local Key**: Local keypair signs. On-chain delegation bounds must include the origin settler in allowed targets.

**TUI rendering**: `CrossChainProgress` -- two-chain bridge visualization showing source chain lock, filler competition, and destination chain delivery status. Status badge updates from pending through filling to filled.

**promptSnippet**:
> Submit an ERC-7683 cross-chain intent for cross-chain swaps via filler network. Use for moving assets between chains. Always verify destination chain health with get_chain_status first.

**promptGuidelines**:
- **thriving**: Use for cross-chain rebalancing. Verify filler bids before committing. Prefer chains with high filler competition.
- **cautious**: Use only for moving assets to safer chains. Set conservative deadlines (1800s). Verify destination chain is not degraded.
- **declining**: Use only for consolidating assets to a single chain for simpler exit management.
- **terminal**: Use if assets are stranded on non-primary chains. Accept wider spreads for speed.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "uniswap_submit_cross_chain_intent", params: { token_in, token_out, amount, source_chain, destination_chain } }`
- `tool:update` -> `{ step: "Signing intent" }` | `{ step: "Submitting to filler network" }` | `{ step: "Awaiting initial bids" }`
- `tool:end` -> `{ success: true, result_summary: { intent_id, status, filler_count } }`

**Pi hooks**: `tool_call` (allowlist + spending limit + chain health check), `tool_result` (intent ID logged for cross-chain tracking)

**Ground truth verification**:
- **expected_outcome**: `"Cross-chain intent: {amount} {token_in} on {source_chain} -> >= {min_output} {token_out} on {destination_chain}"`
- **actual_outcome**: `"Intent {intent_id} status: {status}, received: {actual_output}"` (tracked via filler network status)
- **ground_truth_source**: `"erc7683_intent_status + destination_balance_diff"` -- confirmed by destination chain balance check after fill

---

## ToolDef summary

| Tool | Category | Capability | Risk tier | Tick budget | Sprite |
| --- | --- | --- | --- | --- | --- |
| `uniswap_get_quote` | `Trading` | `Read` | `Layer1` | `Medium` | `Thinking` |
| `uniswap_execute_swap` | `Trading` | `Write` | `Layer3` | `Slow` | `Executing` |
| `uniswap_submit_uniswapx_order` | `Trading` | `Write` | `Layer2` | `Medium` | `Executing` |
| `uniswap_check_allowance` | `Trading` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `uniswap_submit_cross_chain_intent` | `Trading` | `Write` | `Layer3` | `Slow` | `Executing` |

---

## MEV protection strategies

The trading tools automatically apply MEV mitigation based on trade characteristics:

| Strategy | When applied | Mechanism |
| --- | --- | --- |
| UniswapX | Default for all swaps when enabled | Dutch auction with off-chain fillers. Gasless for the Golem. |
| Private mempool | Large trades on mainnet | Submit via Flashbots Protect RPC |
| Slippage guards | All swaps | Minimum output enforced in calldata |
| Deadline enforcement | All swaps | Transaction expires if not included within deadline |
| Trade splitting | Very large trades (>1% of pool liquidity) | Recommended via `intel_assess_mev_risk` |

---

## Revm fork simulation

Write tools use `ctx.revm_fork(chain_id)` instead of `eth_call` for pre-flight simulation. The Revm fork:

1. Snapshots current chain state from the Alloy provider
2. Executes the transaction locally in an isolated EVM instance
3. Returns gas estimate, output amounts, and any revert data
4. Does not leak transaction details to the RPC provider (privacy advantage over `eth_call`)
5. Runs deterministically -- no mempool interference

```rust
let fork = ctx.revm_fork(chain_id)?;
let sim_result = fork.simulate_tx(calldata, from, to, value).await?;

if sim_result.reverted {
    return Err(ToolError::new(
        "SAFETY_SIMULATION_FAILED",
        format!("Revm simulation reverted: {}", sim_result.revert_reason),
    ));
}

// Inspect state changes
let balance_before = fork.balance_of(signer, token_out)?;
let balance_after = fork.balance_of_post(signer, token_out)?;
let expected_delta = balance_after - balance_before;
```

---

## Error codes

| Code | Category | Description |
| --- | --- | --- |
| `TOKEN_NOT_FOUND` | data | Token not recognized on this chain |
| `ROUTING_NO_ROUTE` | routing | No viable swap route found |
| `ROUTING_INSUFFICIENT_LIQUIDITY` | routing | Insufficient liquidity for trade size |
| `SAFETY_TOKEN_NOT_ALLOWED` | safety | Token not on allowlist |
| `SAFETY_SPENDING_LIMIT_EXCEEDED` | safety | Trade exceeds configured spending limit |
| `SAFETY_SIMULATION_FAILED` | safety | Revm fork simulation reverted |
| `EXECUTION_TX_REVERTED` | execution | Transaction reverted on-chain |
| `EXECUTION_TX_TIMEOUT` | execution | Transaction not included within deadline |
| `WALLET_NOT_CONFIGURED` | wallet | No signer configured for write operations |
| `CROSS_CHAIN_DESTINATION_DEGRADED` | chain | Destination chain health check failed |
| `UNISWAPX_ORDER_EXPIRED` | order | UniswapX order expired without being filled |
