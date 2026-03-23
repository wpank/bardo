# Bardo Tools -- Bridge and aggregator tools [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Cross-chain bridge tools (Stargate, Across, Hop) and DEX aggregator routing/comparison tools. 9 tools total: 6 bridge, 3 aggregator.
>
> Governance tools (veCRV, vlCVX, veAERO, veBAL) have been removed. Governance locks create time-locked positions that conflict with Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) mortality -- a 4-year veCRV lock outlives any Golem, creating orphaned state with no exit path. The locked tokens become permanently inaccessible when the Golem dies. For the same reason, `governance_extend_lock` is gone. If governance participation is needed, it should happen through a non-mortal owner, not through Golem-controlled wallets.

---

> **Reader orientation:** This document specifies 9 tools for cross-chain bridging (Stargate, Across, Hop) and DEX aggregator routing (1inch, CoW Protocol, ParaSwap, Odos). Bridge tools move assets between EVM chains; aggregator tools compare quotes across routing venues. You should be familiar with cross-chain messaging, bridge trust models, and DEX aggregator mechanics. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Cross-chain bridge tools (6)

### `bridge_get_quote`

Get a quote for bridging tokens between chains. Queries the specified bridge protocol's quoter contract or API for output amount, fees, and estimated delivery time.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token` | `String` | Yes | Token to bridge (0x address or symbol) |
| `amount` | `String` | Yes | Amount to bridge (human-readable) |
| `source_chain` | `u64` | Yes | Source chain ID |
| `dest_chain` | `u64` | Yes | Destination chain ID |
| `bridge` | `String` | Yes | `"stargate"`, `"across"`, or `"hop"` |

```rust
#[derive(Debug, Deserialize)]
pub struct BridgeGetQuoteParams {
    /// Token to bridge. Symbol (e.g., "USDC") or 0x-prefixed address.
    pub token: String,
    /// Amount to bridge in human-readable units.
    pub amount: String,
    /// Source chain ID (e.g., 1 for Ethereum, 8453 for Base).
    pub source_chain: u64,
    /// Destination chain ID.
    pub dest_chain: u64,
    /// Bridge protocol: "stargate", "across", or "hop".
    pub bridge: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeQuote {
    pub bridge: String,
    pub token: String,
    pub amount_in: String,
    /// Output amount on destination chain. May differ from amount_in due to fees.
    pub amount_out: String,
    pub bridge_fee: String,
    pub relayer_fee: String,
    pub total_fee_usd: f64,
    /// e.g., "1-3 minutes"
    pub estimated_time: String,
    pub destination_token: String,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Medium` (1-5s) -- bridge API or quoter contract call
**progress_steps**: `["Resolving token", "Querying bridge quoter", "Computing fees"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `BridgeQuoteCard` -- shows source -> destination chain badges, fee breakdown, and estimated time with a timeline indicator.

**promptSnippet**:
> Gets a bridge quote for transferring tokens between chains (Stargate, Across, Hop). Returns fees, estimated time, and output amount. Read-only.

**promptGuidelines**:
- Always call `bridge_compare` instead of this tool if you want the best route. Use this only when you already know which bridge to use.
- Check `bridge_get_limits` to verify the amount is within the bridge's transfer range before quoting.

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components):
- `tool:start` -> `{ tool_name: "bridge_get_quote", params: { bridge, token, source_chain, dest_chain } }`
- `tool:update` -> `{ step: "Querying bridge quoter" }`
- `tool:end` -> `{ success: true, result_summary: { amount_out, total_fee_usd, estimated_time } }`

**Pi hooks**: `tool_call` (none), `tool_result` (none)

---

### `bridge_execute`

Execute a cross-chain bridge transfer. Tokens leave the source chain and arrive on the destination chain after a delay. The handler builds calldata via Alloy (the standard Rust library for EVM interaction) `sol!` bindings for the bridge's router contract, simulates via Revm (Rust EVM implementation for local transaction simulation against forked chain state) fork, then signs and broadcasts.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `bridge` | `String` | Yes | `"stargate"`, `"across"`, or `"hop"` |
| `token` | `String` | Yes | Token to bridge |
| `amount` | `String` | Yes | Amount to bridge |
| `source_chain` | `u64` | Yes | Source chain ID |
| `dest_chain` | `u64` | Yes | Destination chain ID |
| `recipient` | `String` | No | Recipient on destination chain (default: same address) |

```rust
#[derive(Debug, Deserialize)]
pub struct BridgeExecuteParams {
    pub bridge: String,
    pub token: String,
    pub amount: String,
    pub source_chain: u64,
    pub dest_chain: u64,
    /// Recipient on destination chain. Defaults to the Golem's wallet.
    pub recipient: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BridgeExecuteResult {
    pub tx_hash: String,
    pub bridge_id: String,
    /// ISO 8601 estimated arrival time.
    pub estimated_arrival: String,
}
```

**capability**: `CapabilityTier::Write` -- requires `Capability<BridgeExecute>`
**risk_tier**: `RiskTier::Layer3` -- unbounded write (tokens leave current chain)
**tick_budget**: `TickBudget::Slow` (5-15s) -- simulation + approval + broadcast + confirmation
**progress_steps**: `["Checking bridge limits", "Simulating via Revm", "Approving token", "Building bridge calldata", "Signing transaction", "Broadcasting", "Confirming source receipt"]`
**sprite_trigger**: `SpriteTrigger::Executing`

**Custody implications**:
- **Delegation**: Session key signs via the bridge router contract. CaveatEnforcer must whitelist the bridge router address and token. Cross-chain recipient must match the Golem's own address (or owner-approved destinations).
- **Embedded**: Privy wallet signs. PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) validates bridge target, spending limit, and recipient.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed bridge routers and amounts.

**TUI rendering**: `BridgeProgress` -- multi-step progress bar with source/destination chain icons. After broadcast, transitions to a tracking view showing in-flight status with `bridge_get_status` polling.

**promptSnippet**:
> Executes a cross-chain bridge transfer. Tokens leave the source chain and arrive on the destination chain after a delay. Track status via bridge_get_status.

**promptGuidelines**:
- **thriving**: Bridge as needed for cross-chain strategy. Use `bridge_compare` first. Always specify recipient.
- **cautious**: Bridge only to consolidate assets on a single chain for risk reduction.
- **declining**: Bridge to consolidate assets for settlement.
- **terminal**: Bridge only for final asset consolidation.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "bridge_execute", params: { bridge, token, amount, source_chain, dest_chain } }`
- `tool:update` -> `{ step: "Simulating via Revm" }` | `{ step: "Signing transaction" }` | `{ step: "Broadcasting" }`
- `tool:end` -> `{ success: true, result_summary: { tx_hash, bridge_id, estimated_arrival } }`

**Pi hooks**: `tool_call` (balance check, bridge limit validation, spending limit), `tool_result` (track in-flight bridge in Grimoire (the Golem's persistent memory system with episodic, semantic, and strategy layers))

**Ground truth verification**:
- **expected_outcome**: `"Bridge {amount} {token} from chain {source_chain} to chain {dest_chain} via {bridge}"`
- **actual_outcome**: `"Source tx {tx_hash}, bridge ID {bridge_id}, status: {bridge_status}"` (from `bridge_get_status`)
- **ground_truth_source**: `"source_receipt + bridge_status_api + destination_balance_diff"` -- full verification requires destination chain balance check after arrival

---

### `bridge_get_status`

Check the status of an in-flight bridge transaction. Queries the bridge protocol's status API or destination chain for completion.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `bridge` | `String` | Yes | `"stargate"`, `"across"`, or `"hop"` |
| `bridge_id` | `String` | Yes | Bridge transfer ID from `bridge_execute` |
| `source_chain` | `u64` | Yes | Source chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct BridgeGetStatusParams {
    pub bridge: String,
    /// Bridge transfer ID returned by bridge_execute.
    pub bridge_id: String,
    pub source_chain: u64,
}

#[derive(Debug, Serialize)]
pub struct BridgeStatus {
    /// pending, in-transit, completed, or failed.
    pub status: String,
    pub tx_hash_source: String,
    pub tx_hash_dest: Option<String>,
    /// ISO 8601 estimated time remaining, null if completed.
    pub eta: Option<String>,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Fast` (<1s) -- single API call
**progress_steps**: `["Querying bridge status"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `BridgeStatusBadge` -- inline status badge: pending (yellow pulse), in-transit (blue spinner), completed (green check), failed (red X). If completed, shows destination tx hash as a clickable link.

**promptSnippet**:
> Checks the status of an in-flight bridge transfer. Returns pending/in-transit/completed/failed. Read-only.

**promptGuidelines**:
- Poll every 30 seconds for Stargate and Hop. Across fills are faster -- poll every 15 seconds.
- If status is "failed", check the error details and consider retrying via `bridge_execute`.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "bridge_get_status", params: { bridge, bridge_id } }`
- `tool:end` -> `{ success: true, result_summary: { status, eta } }`

**Pi hooks**: `tool_call` (none), `tool_result` (alert if status is "failed")

---

### `bridge_compare`

Compare all bridge options for a route and rank by net output after fees. Queries Stargate, Across, and Hop in parallel and returns a ranked list with a recommendation.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token` | `String` | Yes | Token to bridge |
| `amount` | `String` | Yes | Amount to bridge |
| `source_chain` | `u64` | Yes | Source chain ID |
| `dest_chain` | `u64` | Yes | Destination chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct BridgeCompareParams {
    pub token: String,
    pub amount: String,
    pub source_chain: u64,
    pub dest_chain: u64,
}

#[derive(Debug, Serialize)]
pub struct BridgeCompareRoute {
    pub bridge: String,
    pub amount_out: String,
    pub total_fee_usd: f64,
    pub estimated_time: String,
    pub rank: u32,
    pub notes: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeRecommendation {
    pub bridge: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeCompareResult {
    pub routes: Vec<BridgeCompareRoute>,
    pub recommendation: BridgeRecommendation,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Medium` (1-5s) -- parallel queries to 3 bridges
**progress_steps**: `["Querying Stargate", "Querying Across", "Querying Hop", "Ranking by net output"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `BridgeCompareTable` -- ranked table with bridge name, output amount, fees, estimated time, and a "Recommended" badge on the winner.

**promptSnippet**:
> Compares all bridge options for a route and ranks by net output after fees. Returns a recommendation. Read-only. Always use before bridging.

**promptGuidelines**:
- Always call this before `bridge_execute`. Picking the wrong bridge can cost 2-5% in fees on large transfers.
- The recommendation considers both fees and speed. For time-sensitive transfers, check the `estimated_time` column directly.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "bridge_compare", params: { token, amount, source_chain, dest_chain } }`
- `tool:update` -> `{ step: "Querying Stargate" }` | `{ step: "Querying Across" }` | `{ step: "Ranking by net output" }`
- `tool:end` -> `{ success: true, result_summary: { winner: recommendation.bridge, route_count } }`

**Pi hooks**: `tool_call` (none), `tool_result` (none)

---

### `bridge_get_limits`

Get minimum and maximum transfer amounts for a bridge route. Also returns available liquidity on the destination side, which determines the practical maximum.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `bridge` | `String` | Yes | `"stargate"`, `"across"`, or `"hop"` |
| `token` | `String` | Yes | Token address or symbol |
| `source_chain` | `u64` | Yes | Source chain ID |
| `dest_chain` | `u64` | Yes | Destination chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct BridgeGetLimitsParams {
    pub bridge: String,
    pub token: String,
    pub source_chain: u64,
    pub dest_chain: u64,
}

#[derive(Debug, Serialize)]
pub struct BridgeLimits {
    /// Minimum transfer amount in token units.
    pub min_amount: String,
    /// Maximum transfer amount in token units.
    pub max_amount: String,
    /// Available liquidity on destination side.
    pub available_liquidity: String,
    pub token: String,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Querying bridge limits"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `LimitsBadge` -- inline display showing min/max range and a utilization bar for available liquidity (green = plenty, red = near depleted).

**promptSnippet**:
> Returns min/max transfer amounts and available liquidity for a bridge route. Read-only. Check before bridging to avoid BRIDGE_LIMIT_EXCEEDED errors.

**promptGuidelines**:
- Always check limits before calling `bridge_execute`. Submitting an amount outside limits wastes gas on a reverted transaction.
- If `available_liquidity` is less than your transfer amount, the bridge will fail even if the amount is within min/max range.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "bridge_get_limits", params: { bridge, token, source_chain, dest_chain } }`
- `tool:end` -> `{ success: true, result_summary: { min_amount, max_amount, available_liquidity } }`

**Pi hooks**: `tool_call` (none), `tool_result` (none)

---

### `bridge_estimate_time`

Estimate bridge completion time based on current queue depth and network conditions. Returns p50 and p90 estimates for probabilistic planning.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `bridge` | `String` | Yes | `"stargate"`, `"across"`, or `"hop"` |
| `source_chain` | `u64` | Yes | Source chain ID |
| `dest_chain` | `u64` | Yes | Destination chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct BridgeEstimateTimeParams {
    pub bridge: String,
    pub source_chain: u64,
    pub dest_chain: u64,
}

#[derive(Debug, Serialize)]
pub struct BridgeTimeEstimate {
    /// Median completion time (e.g., "2 minutes").
    pub p50_time: String,
    /// 90th percentile completion time (e.g., "8 minutes").
    pub p90_time: String,
    /// Number of pending transfers in the bridge's queue.
    pub current_queue_depth: u32,
    /// Additional context (e.g., "Queue depth normal", "High congestion").
    pub notes: String,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Querying bridge queue"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `TimeEstimateBadge` -- shows p50 time as primary with p90 in parentheses. Color-coded: green (< 5 min), yellow (5-15 min), red (> 15 min).

**promptSnippet**:
> Estimates bridge completion time based on current queue depth. Returns p50 and p90 estimates. Read-only.

**promptGuidelines**:
- Use the p90 estimate for planning, not p50. Bridge delays can spike during network congestion.
- If `current_queue_depth` is unusually high, consider waiting or using a different bridge.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "bridge_estimate_time", params: { bridge, source_chain, dest_chain } }`
- `tool:end` -> `{ success: true, result_summary: { p50_time, p90_time, current_queue_depth } }`

**Pi hooks**: `tool_call` (none), `tool_result` (none)

---

## DEX aggregator routing tools (3)

Cross-aggregator comparison and utility tools. These tools query external aggregator APIs (1inch, CoW Protocol, ParaSwap, Odos) for quotes and gas estimates. They do not execute trades -- use the aggregator-specific swap tools in [03-tools-trading.md](03-tools-trading.md) (swap execution, UniswapX orders, cross-chain intents, and approval management) for execution.

### `aggregator_compare`

Get quotes from all supported aggregators in parallel and rank by net output after gas costs. The winner is the aggregator delivering the most tokens to the Golem after subtracting gas costs denominated in the output token.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_in` | `String` | Yes | Token to sell |
| `token_out` | `String` | Yes | Token to buy |
| `amount` | `String` | Yes | Amount to sell |
| `chain` | `u64` | Yes | Chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct AggregatorCompareParams {
    pub token_in: String,
    pub token_out: String,
    pub amount: String,
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AggregatorQuote {
    pub aggregator: String,
    pub amount_out: String,
    /// Output minus gas cost in output token terms.
    pub net_output_after_gas: String,
    pub gas_usd: f64,
    pub rank: u32,
    /// e.g., "MEV protected", "Best for large trades"
    pub notes: String,
}

#[derive(Debug, Serialize)]
pub struct AggregatorCompareResult {
    /// Aggregator name of the winner.
    pub winner: String,
    pub quotes: Vec<AggregatorQuote>,
    /// Human-readable recommendation explaining why the winner was chosen.
    pub recommendation: String,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Medium` (1-5s) -- parallel API calls to 4 aggregators
**progress_steps**: `["Querying 1inch", "Querying CoW", "Querying ParaSwap", "Querying Odos", "Ranking by net output"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `AggregatorCompareTable` -- ranked table with aggregator logo, output amount, gas cost, net output, and notes. Winner row highlighted.

**promptSnippet**:
> Gets quotes from all supported aggregators in parallel and ranks by net output after gas. Returns a winner and recommendation. Read-only. Always use before large trades to find the best execution venue.

**promptGuidelines**:
- Use for trades where the Uniswap router might not be optimal (e.g., exotic tokens, large size, multi-hop routes).
- CoW Protocol quotes are MEV-protected by default. Note this in the recommendation if CoW is competitive but not the cheapest.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "aggregator_compare", params: { token_in, token_out, amount, chain } }`
- `tool:update` -> `{ step: "Querying 1inch" }` | `{ step: "Querying CoW" }` | `{ step: "Ranking by net output" }`
- `tool:end` -> `{ success: true, result_summary: { winner, quote_count } }`

**Pi hooks**: `tool_call` (rate-limit), `tool_result` (none)

---

### `aggregator_get_gas_estimate`

Get gas cost estimate for a swap via a specific aggregator. Useful for comparing gas efficiency between aggregators before committing to one.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `aggregator` | `String` | Yes | `"1inch"`, `"cow"`, `"paraswap"`, or `"odos"` |
| `token_in` | `String` | Yes | Token to sell |
| `token_out` | `String` | Yes | Token to buy |
| `amount` | `String` | Yes | Amount to sell |
| `chain` | `u64` | Yes | Chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct AggregatorGetGasEstimateParams {
    /// Aggregator: "1inch", "cow", "paraswap", or "odos".
    pub aggregator: String,
    pub token_in: String,
    pub token_out: String,
    pub amount: String,
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct GasEstimate {
    pub gas_units: u64,
    pub gas_price: String,
    pub gas_cost_eth: String,
    pub gas_cost_usd: f64,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Querying aggregator gas estimate"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `GasBadge` -- inline display showing gas cost in USD with a color indicator based on current gas conditions (green = cheap, red = expensive relative to 7d average).

**promptSnippet**:
> Returns gas cost estimate for a swap via a specific aggregator. Read-only. Use aggregator_compare for a full ranking -- this tool is for single-aggregator gas checks.

**promptGuidelines**:
- CoW Protocol (cow) gas is paid by solvers, so `gas_cost_usd` will be 0 for CoW quotes. The cost is embedded in the spread instead.
- Compare gas estimates across aggregators to find the most gas-efficient route for small trades where gas is a significant fraction of trade value.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "aggregator_get_gas_estimate", params: { aggregator, token_in, token_out, chain } }`
- `tool:end` -> `{ success: true, result_summary: { gas_cost_usd } }`

**Pi hooks**: `tool_call` (rate-limit), `tool_result` (none)

---

### `aggregator_check_approval`

Check if a token approval is needed for a specific aggregator's router contract. Returns approval calldata if required. Does not broadcast -- the actual approval must go through `uniswap_approve_token` or a dedicated approve handler.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `aggregator` | `String` | Yes | `"1inch"`, `"cow"`, `"paraswap"`, `"odos"`, or `"bebop"` |
| `token` | `String` | Yes | Token address to check |
| `amount` | `String` | Yes | Amount that will be sold |
| `chain` | `u64` | Yes | Chain ID |

```rust
#[derive(Debug, Deserialize)]
pub struct AggregatorCheckApprovalParams {
    /// Aggregator: "1inch", "cow", "paraswap", "odos", or "bebop".
    pub aggregator: String,
    pub token: String,
    pub amount: String,
    pub chain: u64,
}

#[derive(Debug, Serialize)]
pub struct AggregatorApprovalResult {
    pub needs_approval: bool,
    /// Aggregator router contract address, null if no approval needed.
    pub spender: Option<String>,
    /// Pre-built approval tx calldata, null if no approval needed.
    pub approval_tx: Option<String>,
}
```

**capability**: `CapabilityTier::Read`
**risk_tier**: `RiskTier::Layer1`
**tick_budget**: `TickBudget::Fast` (<1s)
**progress_steps**: `["Resolving spender", "Reading allowance"]`
**sprite_trigger**: `SpriteTrigger::Thinking`

**TUI rendering**: `ApprovalChip` -- inline badge showing "Approved" (green) or "Needs approval" (yellow) with aggregator name.

**promptSnippet**:
> Checks if a token approval is needed for a specific aggregator. Returns approval tx calldata if required. Read-only check, no state change.

**promptGuidelines**:
- Check before any aggregator swap. Each aggregator uses a different router contract, so approvals from one don't carry over.
- If `needs_approval` is true, use the spender address with the appropriate approval tool before executing the swap.

**Event Fabric**:
- `tool:start` -> `{ tool_name: "aggregator_check_approval", params: { aggregator, token, chain } }`
- `tool:end` -> `{ success: true, result_summary: { needs_approval } }`

**Pi hooks**: `tool_call` (rate-limit), `tool_result` (none)

---

## ToolDef summary

| Tool | Category | Capability | Risk tier | Tick budget | Sprite |
| --- | --- | --- | --- | --- | --- |
| `bridge_get_quote` | `Trading` | `Read` | `Layer1` | `Medium` | `Thinking` |
| `bridge_execute` | `Trading` | `Write` | `Layer3` | `Slow` | `Executing` |
| `bridge_get_status` | `Trading` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `bridge_compare` | `Trading` | `Read` | `Layer1` | `Medium` | `Thinking` |
| `bridge_get_limits` | `Trading` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `bridge_estimate_time` | `Trading` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `aggregator_compare` | `Trading` | `Read` | `Layer1` | `Medium` | `Thinking` |
| `aggregator_get_gas_estimate` | `Trading` | `Read` | `Layer1` | `Fast` | `Thinking` |
| `aggregator_check_approval` | `Trading` | `Read` | `Layer1` | `Fast` | `Thinking` |

---

## Removed: governance and incentive tools

The following 8 tools from the v1 spec have been cut:

| Removed tool | Reason |
| --- | --- |
| `governance_lock_token` | Governance locks (1 week to 4 years) outlive Golems. Locked tokens become permanently inaccessible at Golem death. |
| `governance_vote_gauge` | Requires active voting power from locks. Without locks, no votes. |
| `governance_get_bribes` | Bribe markets are only actionable with voting power. Read-only bribe data can be added to a data tool if needed. |
| `governance_claim_bribes` | Requires locked positions that can't exist under mortality constraints. |
| `governance_get_voting_power` | No locks -> no voting power -> nothing to query. |
| `governance_get_gauge_apy` | Gauge APY is only relevant for locked position holders. |
| `governance_get_locked_positions` | No locks -> no positions -> nothing to list. |
| `governance_extend_lock` | Extending locks compounds the mortality conflict -- the extended duration could outlive the Golem by years. |

If a Golem needs to interact with governance protocols, the correct pattern is to delegate governance tokens to a non-mortal owner address, not to lock them in the Golem's own wallet.

---

## Error codes

| Code | Description |
| --- | --- |
| `BRIDGE_LIMIT_EXCEEDED` | Transfer amount exceeds bridge's min/max limits |
| `BRIDGE_CHAIN_NOT_SUPPORTED` | Bridge does not support this source/destination chain pair |
| `BRIDGE_TOKEN_NOT_SUPPORTED` | Bridge does not support this token on this route |
| `BRIDGE_LIQUIDITY_INSUFFICIENT` | Available liquidity on destination side is less than transfer amount |
| `BRIDGE_TRANSFER_FAILED` | Bridge transfer failed (check bridge_get_status for details) |
| `AGGREGATOR_UNAVAILABLE` | Aggregator API is down or returned an error |
| `QUOTE_EXPIRED` | Aggregator quote has expired |
| `AGGREGATOR_INSUFFICIENT_LIQUIDITY` | Aggregator cannot fill the trade at any price |
