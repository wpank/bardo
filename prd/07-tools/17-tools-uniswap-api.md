# Bardo Tools -- Uniswap API tools [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles), [20-config.md](20-config.md) (environment variables, TOML config schema, API key tiers)
>
> Tools backed by the Uniswap Trading API. Category: `trading` (swap tools), `lp` (LP tools), `data` (reference tools). 21 tools. All require `BARDO_UNISWAP_API_KEY`.
>
> **Protocol framing note:** This document covers Uniswap API integration in depth. Equivalent depth of specification exists or will exist for other major DeFi protocols: Aave/Morpho lending tools (`07-tools-lending.md`), Curve/Balancer AMM tools, Pendle yield tools (`11-tools-yield.md`), bridging aggregators (`05-tools-bridge-aggregator.md`). The Uniswap API is one integration among many — not the center of the tool library.

---

> **Reader orientation:** This document specifies 21 tools backed by the Uniswap Trading API (`trade-api.gateway.uniswap.org/v1`). These tools provide limit orders, batch execution, EIP-7702 delegation, LP operations via API, and multi-step cross-chain plans. All require a `BARDO_UNISWAP_API_KEY`. You should be familiar with Uniswap's swap routing, Permit2, and UniswapX order types. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Why the Uniswap API

The Uniswap Trading API is not a convenience wrapper around on-chain calls. It unlocks a different class of DeFi operation that is not achievable by reading chain state or calling smart contracts directly.

### vs on-chain reads + direct contract calls

Direct on-chain routing requires implementing a routing algorithm across thousands of pools, with no access to off-chain or private liquidity. The practical gaps:

- **No private liquidity**: UniswapX RFQ fillers hold significant off-chain inventory. A direct on-chain swap never touches this liquidity, always trading at worse prices.
- **No MEV protection**: Submitting a signed transaction to the public mempool exposes the order to frontrunning and sandwich attacks. UniswapX intents go to a private filler network, bypassing the mempool entirely.
- **No Permit2 optimization**: Permit2 batches token approval + transfer into a single signature, saving a full approval transaction. Constructing valid Permit2 calldata from scratch requires replicating protocol-specific logic that the API generates correctly.
- **No cross-chain orchestration**: A bridge + swap flow requires coordinating separate protocols (bridge + DEX) with no shared state. The API generates sequenced calldata and tracks proof submission across steps.
- **No gas calibration**: Estimating gas accurately requires simulating transactions against current chain state. The API does this server-side with live state, returning calibrated estimates. Client-side estimation is unreliable under congestion.

### vs Uniswap SDK (@uniswap/v3-sdk, @uniswap/router-sdk, @uniswap/smart-order-router)

The SDK runs routing client-side using on-chain pool state fetched via RPC. This was the right approach before UniswapX. Now:

- **No RFQ access**: The SDK cannot query UniswapX RFQ fillers. It routes through on-chain pools only.
- **No intent submission**: There is no SDK path for submitting Dutch v2/v3 or Priority orders. These require the API.
- **No gasless execution**: The SDK produces signed transactions. UniswapX gasless orders are a different primitive (signed intents) that only the API supports.
- **No LP migration**: The SDK has no concept of migrating a position between protocol versions. `/lp/migrate` generates the calldata to atomically exit a V3 position and enter a V4 position.
- **No multi-step planning**: The SDK does not track multi-step execution state.

### vs other aggregators (0x, 1inch, Paraswap)

- **Exclusive RFQ access**: UniswapX RFQ fillers are not accessible via other aggregators. Private liquidity from Uniswap's filler network only routes through the Uniswap API.
- **V4 hooks support**: No other aggregator routes through V4 pools with hooks. The `hooksOptions` parameter enables targeted routing exclusive to this API.
- **Deepest Permit2 integration**: Permit2 is a Uniswap-built primitive. The API generates Permit2 calldata with full correctness guarantees.
- **EIP-5792/7702 support**: Batch and delegated execution modes are specific to the Uniswap API.

### Capabilities only available via API

| Capability | Endpoint | Alternative |
| --- | --- | --- |
| Limit orders (off-chain, on-chain fill) | `/limit_order_quote` + `/order` | None |
| Gasless swaps (filler pays gas) | `/quote` with UniswapX routing + `/order` | None |
| LP migration (V2->V3->V4 calldata) | `/lp/migrate` | Manual: 2 separate txs, no atomicity |
| LP rewards claiming | `/lp/claim_rewards` | Protocol-specific, non-standard |
| EIP-5792 batch execution | `/swap_5792` | None (EOA cannot batch) |
| EIP-7702 smart delegation | `/swap_7702` + `/wallet/encode_7702` | None (requires delegation setup) |
| Bridge token enumeration | `/swappable_tokens` | Manual: query each bridge protocol separately |

---

## Paradigms enabled

### Gasless agent operation

An agent can operate indefinitely without holding native tokens for gas. UniswapX Dutch v2/v3 orders are signed intents: the agent specifies what it wants, fillers compete to fill it, and the winning filler pays the gas. The agent's gas cost is priced into the output amount.

Flow: `uniswap_api_get_quote` with `routing_preference: "DUTCH_V2"` -> sign the returned quote -> `uniswap_api_submit_order`. Monitor via `uniswap_api_get_orders`. No ETH required.

### V4-hooks-targeted routing

V4 pools can have hooks -- custom contract logic that executes before/after every swap or LP operation. An agent deploying a custom hook strategy needs liquidity to route through that hook's pool.

`uniswap_api_get_quote` with `hooks_options: "V4_HOOKS_ONLY"` routes exclusively through V4 pools with hooks. `V4_HOOKS_INCLUSIVE` prefers them. `V4_NO_HOOKS` avoids them. No other way to enforce this constraint exists outside the API.

### Limit order portfolios

An agent can maintain an active book of limit orders. Flow: `uniswap_api_get_limit_order_quote` -> sign -> `uniswap_api_submit_order` with routing `LIMIT_ORDER` -> monitor via `uniswap_api_get_orders`. Unfilled orders expire at `order_deadline`. Enables DCA, grid trading, and yield-seeking entries without on-chain interaction until fill.

### Approval-first safety pipeline

Call `uniswap_api_check_approval` before any swap or LP operation. It returns whether approval exists and, if not, the approval transaction calldata. This converts on-chain reverts (gas spent, tx failed) into pre-flight errors (no gas spent, no tx submitted).

With EIP-5792, the approval calldata and the swap calldata can be submitted as a single atomic batch.

---

## Configuration

### Per-Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) config block

```toml
[uniswap_api]
api_key = "your-api-key-from-developers.uniswap.org"
base_url = "https://trade-api.gateway.uniswap.org/v1"
rate_limit = 3
quote_max_age_ms = 30000
gas_buffer_pct = 20
universal_router_version = "2.0"
permit2_enabled = true
enable_lp = true
enable_limit_orders = true
enable_smart_wallet = false
chain_ids = [1, 8453, 42161, 10, 137]
```

Config precedence: Golem config file > env vars > defaults.

API keys are obtained from [developers.uniswap.org](https://developers.uniswap.org/dashboard/).

### Environment variables

| Variable | Default | Description |
| --- | --- | --- |
| `BARDO_UNISWAP_API_KEY` | -- | API key (required to activate API tools) |
| `BARDO_UNISWAP_API_BASE_URL` | `https://trade-api.gateway.uniswap.org/v1` | Base URL override |
| `BARDO_UNISWAP_API_RATE_LIMIT` | `3` | Requests per second |
| `BARDO_UNISWAP_API_QUOTE_MAX_AGE_MS` | `30000` | Quote cache TTL (ms) |
| `BARDO_UNISWAP_API_GAS_BUFFER_PCT` | `20` | Gas estimate buffer percent |
| `BARDO_UNISWAP_ROUTER_VERSION` | `2.0` | Universal Router version (`1.2` or `2.0`) |

### Feature flags

| Flag | Effect when false |
| --- | --- |
| `enable_lp` | LP tools fall back to SDK; `uniswap_api_migrate_liquidity`, `uniswap_api_get_lp_pool_info` disabled |
| `enable_limit_orders` | `uniswap_api_get_limit_order_quote` disabled; `uniswap_api_submit_order` still works for Dutch/Priority orders |
| `enable_smart_wallet` | `uniswap_api_batch_swap`, `uniswap_api_create_delegated_swap` disabled |

---

## HTTP client

All API tools share a `TradingApiClient` built on `reqwest`:

```rust
pub struct TradingApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: TaintedString,
    rate_limiter: Arc<TokenBucket>,
}

impl TradingApiClient {
    pub async fn get<T: DeserializeOwned>(&self, path: &str, params: &impl Serialize) -> Result<T> {
        self.rate_limiter.acquire().await;
        let resp = self.client
            .get(format!("{}{}", self.base_url, path))
            .header("x-api-key", self.api_key.as_ref())
            .query(params)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T> {
        self.rate_limiter.acquire().await;
        let resp = self.client
            .post(format!("{}{}", self.base_url, path))
            .header("x-api-key", self.api_key.as_ref())
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    fn is_fallback_eligible(status: reqwest::StatusCode) -> bool {
        matches!(status.as_u16(), 429 | 503 | 504)
    }
}
```

Rate limiting: token bucket (default 3 req/sec). The bucket is shared across all API tools within a Golem session.

---

## Swap tools (category: `trading`)

### `uniswap_api_get_quote`

Requests a quote for a swap, bridge, or wrap/unwrap. Routes across V2/V3/V4 and UniswapX.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_in` | `String` | Yes | Input token symbol or address |
| `token_out` | `String` | Yes | Output token symbol or address |
| `token_in_chain_id` | `u64` | Yes | Chain ID for input token |
| `token_out_chain_id` | `Option<u64>` | No | Chain ID for output token (cross-chain if different) |
| `amount` | `String` | Yes | Amount in token base units |
| `trade_type` | `String` | Yes | `"exactIn"` or `"exactOut"` |
| `swapper` | `Option<String>` | No | Wallet address (required for UniswapX routing) |
| `routing_preference` | `Option<String>` | No | `"AUTO"`, `"CLASSIC"`, `"DUTCH_V2"`, `"DUTCH_V3"`, `"PRIORITY"`, `"FASTEST"`. Default: `"AUTO"` |
| `protocols` | `Option<Vec<String>>` | No | Restrict to specific protocols (e.g., `["V4"]`) |
| `slippage_tolerance` | `Option<u32>` | No | Slippage in basis points (1-5000) |
| `auto_slippage` | `Option<bool>` | No | Let API compute optimal slippage |
| `hooks_options` | `Option<String>` | No | `"V4_HOOKS_INCLUSIVE"`, `"V4_HOOKS_ONLY"`, `"V4_NO_HOOKS"` |
| `spread_optimization` | `Option<bool>` | No | Spread execution to minimize price impact |

```rust
#[derive(Debug, Serialize)]
pub struct QuoteResult {
    pub quote_id: String,
    pub token_in: String,
    pub token_out: String,
    pub token_in_chain_id: u64,
    pub token_out_chain_id: u64,
    pub amount_in: String,
    pub amount_out: String,
    pub price_impact: f64,
    pub gas_estimate: String,
    pub gas_estimate_usd: f64,
    pub routing_type: String,
    pub route: serde_json::Value,
    pub permit_data: Option<serde_json::Value>,
    pub valid_until: u64,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer1` |
| tick_budget | `Medium` |
| progress_steps | `["Requesting quote", "Parsing route"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Gets a swap/bridge/wrap quote from the Uniswap Trading API. Routes across V2/V3/V4 and UniswapX. Prefer over manual routing -- exclusive access to RFQ fillers and V4 hooks. Read-only. |

**promptGuidelines:**
- **thriving**: Use AUTO routing for most trades. Use DUTCH_V2 for gasless execution. Set hooks_options for hook-specific strategies.
- **cautious**: Prefer DUTCH_V2 (gasless) to conserve ETH. Enable auto_slippage.
- **declining**: Quote only settlement swaps. Use FASTEST routing.
- **terminal**: Quote only final settlement. Use FASTEST with urgency "urgent".

**Event Fabric events:** `tool:start` -> `{ token_in, token_out, amount, routing_preference }` -> `tool:end` -> `{ quote_id, amount_out, routing_type, price_impact }`

---

### `uniswap_api_check_approval`

Checks if a wallet has required token spend approval for a swap or LP operation. Returns approval transaction calldata if needed. Consolidates the previously separate `check_swap_approval` and `check_lp_approval` into a single tool -- the API endpoint is the same, the context (swap vs LP) determines which tokens to check.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address to check |
| `tokens` | `Vec<String>` | Yes | Token addresses to check approval for |
| `amounts` | `Vec<String>` | Yes | Amounts in token base units (parallel with tokens) |
| `chain_id` | `u64` | Yes | Chain ID |
| `permit2_enabled` | `Option<bool>` | No | Check Permit2 approval. Default: `true` |

```rust
#[derive(Debug, Serialize)]
pub struct ApprovalCheckResult {
    pub all_approved: bool,
    pub token_approvals: Vec<TokenApproval>,
}

#[derive(Debug, Serialize)]
pub struct TokenApproval {
    pub token: String,
    pub approved: bool,
    pub approval_tx: Option<TransactionCalldata>,
    pub permit_data: Option<serde_json::Value>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer1` |
| tick_budget | `Fast` |
| progress_steps | `["Checking approvals"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Checks wallet token approvals for swap or LP operations. Returns approval tx calldata if needed. ALWAYS call before any write operation. Read-only. |

**promptGuidelines:**
- All phases: call before every swap or LP operation. Batch approval txs with EIP-5792 if smart wallet is available.

**Event Fabric events:** `tool:start` -> `{ wallet_address, token_count }` -> `tool:end` -> `{ all_approved, needs_approval_count }`

---

### `uniswap_api_create_swap_calldata`

Converts a quote into executable swap transaction calldata. Call `uniswap_api_get_quote` first, then pass the quote object here.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `quote` | `serde_json::Value` | Yes | Quote object from `uniswap_api_get_quote` |
| `signature` | `Option<String>` | No | Permit2 signature |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas info |
| `safety_mode` | `Option<bool>` | No | Enable additional on-chain safety checks |
| `deadline` | `Option<u64>` | No | Unix timestamp deadline |

```rust
#[derive(Debug, Serialize)]
pub struct SwapCalldataResult {
    pub calldata: String,
    pub to: String,
    pub value: String,
    pub chain_id: u64,
    pub gas_estimate: Option<String>,
    pub simulation_result: Option<serde_json::Value>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer3` |
| tick_budget | `Medium` |
| progress_steps | `["Building calldata", "Simulating"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Converts a quote into executable swap transaction calldata. Call uniswap_api_get_quote first. Optional: simulate for gas accuracy and revert detection. |

**promptGuidelines:**
- **thriving**: Always set simulate_transaction: true for trades >$1000. Use safety_mode: true.
- **cautious**: Always simulate. Always use safety_mode.
- **declining**: Simulate and execute settlement trades only.
- **terminal**: Simulate. Set urgency urgent.

**Ground truth:** `expected_outcome` = quote amount_out. `actual_outcome` = simulation output. `ground_truth_source` = `"api_simulation"`.

**Event Fabric events:** `tool:start` -> `{ quote_id, simulate_transaction }` -> `tool:end` -> `{ calldata_length, gas_estimate }`

---

### `uniswap_api_batch_swap`

Generates EIP-5792 `wallet_sendCalls` payload for a swap. Requires a smart wallet supporting EIP-5792. Bundles approval + swap into a single atomic batch.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `quote` | `serde_json::Value` | Yes | Quote object from `uniswap_api_get_quote` |
| `permit_data` | `Option<serde_json::Value>` | No | Permit data for approval batching |
| `deadline` | `Option<u64>` | No | Unix timestamp deadline |

```rust
#[derive(Debug, Serialize)]
pub struct BatchSwapResult {
    pub calls: Vec<TransactionCalldata>,
    pub chain_id: u64,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer3` |
| tick_budget | `Medium` |
| progress_steps | `["Building batch"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Generates EIP-5792 wallet_sendCalls payload for a swap. Requires smart wallet. Bundles approval + swap atomically. |

**Capability gate:** `enable_smart_wallet = true`.

**Event Fabric events:** `tool:start` -> `{ quote_id }` -> `tool:end` -> `{ call_count, chain_id }`

---

### `uniswap_api_create_delegated_swap`

Generates EIP-7702 calldata for smart contract wallet delegation execution.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `quote` | `serde_json::Value` | Yes | Quote object from `uniswap_api_get_quote` |
| `delegation_address` | `String` | Yes | Address of the delegation target contract |
| `permit_data` | `Option<serde_json::Value>` | No | Permit data |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas info |

```rust
#[derive(Debug, Serialize)]
pub struct DelegatedSwapResult {
    pub calldata: String,
    pub to: String,
    pub value: String,
    pub chain_id: u64,
    pub gas_info: Option<serde_json::Value>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer3` |
| tick_budget | `Medium` |
| progress_steps | `["Encoding delegation"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Generates EIP-7702 calldata for smart contract wallet delegation execution. Requires delegation setup. |

**Capability gate:** `enable_smart_wallet = true`.

**Event Fabric events:** `tool:start` -> `{ quote_id }` -> `tool:end` -> `{ calldata_length, chain_id }`

---

### `uniswap_api_submit_order`

Submits a signed UniswapX intent or limit order to the filler network. Gasless -- filler pays gas. Consolidates the previously separate `submit_uniswapx_order` naming.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `signature` | `String` | Yes | EIP-712 signature of the quote |
| `quote` | `serde_json::Value` | Yes | The signed quote object |
| `routing` | `String` | Yes | `"DUTCH_V2"`, `"DUTCH_V3"`, `"PRIORITY"`, `"LIMIT_ORDER"` |

```rust
#[derive(Debug, Serialize)]
pub struct SubmitOrderResult {
    pub order_id: String,
    pub order_hash: String,
    pub status: String,
    pub created_at: u64,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer3` |
| tick_budget | `Medium` |
| progress_steps | `["Validating signature", "Submitting to filler network"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Submits a signed UniswapX intent to the filler network. Gasless -- filler pays gas. Requires EIP-712 signature. Supports Dutch v2/v3, Priority, and Limit orders. |

**promptGuidelines:**
- **thriving**: Prefer for trades >$1000 (MEV protection + gasless). DUTCH_V2 for market orders, LIMIT_ORDER for DCA.
- **cautious**: Prefer for all trades (gasless conserves ETH). Monitor fill via `uniswap_api_get_orders`.
- **declining**: Use for settlement orders (gasless is critical when ETH is scarce).
- **terminal**: Use for final settlement orders only.

**Ground truth:** `expected_outcome` = order submitted with status "open". `actual_outcome` = API response status. `ground_truth_source` = `"uniswapx_api"`.

**Event Fabric events:** `tool:start` -> `{ routing, order_hash }` -> `tool:end` -> `{ order_id, status }`

---

### `uniswap_api_get_orders`

Retrieves and filters UniswapX orders. At least one filter parameter is required.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `swapper` | `Option<String>` | No | Filter by swapper address |
| `order_ids` | `Option<Vec<String>>` | No | Filter by specific order IDs |
| `order_status` | `Option<String>` | No | `"open"`, `"expired"`, `"error"`, `"cancelled"`, `"filled"`, `"insufficient-funds"` |
| `order_type` | `Option<String>` | No | Filter by order type |
| `limit` | `Option<u32>` | No | Max results. Default: `10` |
| `cursor` | `Option<String>` | No | Pagination cursor |

```rust
#[derive(Debug, Serialize)]
pub struct GetOrdersResult {
    pub orders: Vec<OrderEntry>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderEntry {
    pub order_id: String,
    pub order_hash: String,
    pub status: String,
    pub swapper: String,
    pub input: TokenAmount,
    pub output: TokenAmount,
    pub deadline: u64,
    pub created_at: u64,
    pub filled_at: Option<u64>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer1` |
| tick_budget | `Fast` |
| progress_steps | `["Fetching orders"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Retrieves UniswapX orders. Filter by swapper, order ID, status. Use to monitor pending gasless orders. Read-only. |

**Event Fabric events:** `tool:start` -> `{ filter_count }` -> `tool:end` -> `{ order_count }`

---

### `uniswap_api_check_swap_status`

Gets the status of swap or bridge transactions by hash.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `transaction_hashes` | `Vec<String>` | Yes | Transaction hashes to check |
| `chain_id` | `u64` | Yes | Chain ID where transactions were submitted |

```rust
#[derive(Debug, Serialize)]
pub struct SwapStatusResult {
    pub swaps: Vec<SwapStatus>,
}

#[derive(Debug, Serialize)]
pub struct SwapStatus {
    pub tx_hash: String,
    pub status: String,   // "PENDING", "SUCCESS", "FAILED", "EXPIRED", "NOT_FOUND"
    pub details: Option<serde_json::Value>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer1` |
| tick_budget | `Fast` |
| progress_steps | `["Checking status"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Gets status of swap or bridge transactions by hash. Supports batch checking. Read-only. |

**Event Fabric events:** `tool:start` -> `{ hash_count }` -> `tool:end` -> `{ statuses }`

---

### `uniswap_api_get_limit_order_quote`

Gets a quote for a limit order at a specified price.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `swapper` | `String` | Yes | Wallet address placing the order |
| `token_in` | `String` | Yes | Input token address |
| `token_out` | `String` | Yes | Output token address |
| `chain_id` | `u64` | Yes | Chain ID |
| `amount` | `String` | Yes | Input amount in token base units |
| `limit_price` | `String` | Yes | Minimum output amount (limit price as token ratio) |
| `trade_type` | `String` | Yes | `"exactIn"` or `"exactOut"` |
| `order_deadline` | `u64` | Yes | Unix timestamp when order expires |

```rust
#[derive(Debug, Serialize)]
pub struct LimitOrderQuoteResult {
    pub quote: serde_json::Value,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer2` |
| tick_budget | `Medium` |
| progress_steps | `["Requesting limit quote"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Gets a quote for a limit order at a specified price. Sign the quote, then submit via uniswap_api_submit_order with routing LIMIT_ORDER. No gas until fill. |

**promptGuidelines:**
- **thriving**: Use for DCA strategies, grid trading, yield-seeking entries (e.g., limit buy 2% below market).
- **cautious**: Conservative entries only. Set tight expiry.
- **declining**: Not applicable for new orders.
- **terminal**: Not applicable.

**Capability gate:** `enable_limit_orders = true`.

**Event Fabric events:** `tool:start` -> `{ token_in, token_out, limit_price }` -> `tool:end` -> `{ quote_valid }`

---

## LP tools (category: `lp`)

LP tools follow the **API-first / SDK-fallback** pattern. When the API key is set and `enable_lp` is true, these tools route through the Uniswap API. If the API returns a fallback-eligible error (429, 503, 504, network timeout), they fall back to the TypeScript sidecar running the Uniswap SDK.

```rust
async fn api_first_with_fallback<T>(
    api: &TradingApiClient,
    sidecar: &SidecarClient,
    api_call: impl Future<Output = Result<T>>,
    sdk_call: impl Future<Output = Result<T>>,
) -> Result<T> {
    match api_call.await {
        Ok(result) => Ok(result),
        Err(e) if TradingApiClient::is_fallback_eligible_error(&e) => sdk_call.await,
        Err(e) => Err(e),
    }
}
```

---

### `uniswap_api_create_lp_position`

Creates a new LP position.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address creating the position |
| `token0` | `String` | Yes | Token 0 address or symbol |
| `token1` | `String` | Yes | Token 1 address or symbol |
| `chain_id` | `u64` | Yes | Chain ID |
| `fee_tier` | `u32` | Yes | Fee tier in bps (100, 500, 3000, 10000) |
| `amount0` | `String` | Yes | Amount of token0 in base units |
| `amount1` | `String` | Yes | Amount of token1 in base units |
| `tick_lower` | `Option<i32>` | No | Lower tick (V3/V4 only) |
| `tick_upper` | `Option<i32>` | No | Upper tick (V3/V4 only) |
| `protocol` | `Option<String>` | No | `"V2"`, `"V3"`, `"V4"`. Default: `"V4"` |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas estimate |

```rust
#[derive(Debug, Serialize)]
pub struct CreateLpResult {
    pub txs: Vec<TransactionCalldata>,
    pub position_id: Option<String>,
    pub pool_address: String,
    pub gas_estimate: Option<String>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer3` |
| tick_budget | `Medium` |
| progress_steps | `["Checking approvals", "Building position calldata", "Simulating"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Creates a new LP position via the Uniswap API. API-first with SDK fallback. Supports V2, V3, V4. Default: V4. |

**promptGuidelines:**
- **thriving**: Create positions in V4 pools for best efficiency. Set tick ranges based on strategy.
- **cautious**: No new LP positions.
- **declining**: Blocked.
- **terminal**: Blocked.

**Ground truth:** `expected_outcome` = position created at specified tick range. `actual_outcome` = simulation result. `ground_truth_source` = `"api_simulation"`.

**Event Fabric events:** `tool:start` -> `{ token0, token1, protocol, fee_tier }` -> `tool:end` -> `{ position_id, pool_address }`

---

### `uniswap_api_increase_lp_position`

Adds liquidity to an existing LP position.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address |
| `position_id` | `String` | Yes | Position token ID or NFT ID |
| `chain_id` | `u64` | Yes | Chain ID |
| `amount0` | `Option<String>` | No | Additional amount of token0 |
| `amount1` | `Option<String>` | No | Additional amount of token1 |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas estimate |

```rust
#[derive(Debug, Serialize)]
pub struct IncreaseLpResult {
    pub txs: Vec<TransactionCalldata>,
    pub new_liquidity: String,
    pub gas_estimate: Option<String>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer2` |
| tick_budget | `Medium` |
| progress_steps | `["Building increase calldata"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Adds liquidity to an existing LP position. API-first with SDK fallback. Phase-gated: blocked in declining/terminal. |

**Event Fabric events:** `tool:start` -> `{ position_id }` -> `tool:end` -> `{ new_liquidity }`

---

### `uniswap_api_decrease_lp_position`

Removes liquidity from an existing LP position.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address |
| `position_id` | `String` | Yes | Position token ID or NFT ID |
| `chain_id` | `u64` | Yes | Chain ID |
| `liquidity_pct` | `Option<u32>` | No | Percent of liquidity to remove (1-100). Default: `100` |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas estimate |

```rust
#[derive(Debug, Serialize)]
pub struct DecreaseLpResult {
    pub txs: Vec<TransactionCalldata>,
    pub amount0_out: String,
    pub amount1_out: String,
    pub gas_estimate: Option<String>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer2` |
| tick_budget | `Medium` |
| progress_steps | `["Building decrease calldata"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Removes liquidity from an LP position. Default: remove 100%. Allowed at any phase. |

**promptGuidelines:**
- **thriving**: Remove to rebalance or capture fees.
- **cautious**: Remove from out-of-range positions.
- **declining**: Remove all liquidity. Mandatory.
- **terminal**: Remove any remaining.

**Ground truth:** `expected_outcome` = liquidity_pct% removed. `actual_outcome` = amounts returned. `ground_truth_source` = `"api_simulation"`.

**Event Fabric events:** `tool:start` -> `{ position_id, liquidity_pct }` -> `tool:end` -> `{ amount0_out, amount1_out }`

---

### `uniswap_api_claim_lp_fees`

Claims accrued trading fees from an LP position.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address |
| `position_id` | `String` | Yes | Position token ID or NFT ID |
| `chain_id` | `u64` | Yes | Chain ID |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas estimate |

```rust
#[derive(Debug, Serialize)]
pub struct ClaimFeesResult {
    pub txs: Vec<TransactionCalldata>,
    pub fee0_amount: String,
    pub fee1_amount: String,
    pub gas_estimate: Option<String>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer1` |
| tick_budget | `Medium` |
| progress_steps | `["Building claim calldata"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Claims accrued trading fees from an LP position. API-first with SDK fallback. Allowed at any phase. |

**Event Fabric events:** `tool:start` -> `{ position_id }` -> `tool:end` -> `{ fee0_amount, fee1_amount }`

---

### `uniswap_api_migrate_liquidity`

Generates calldata to migrate an LP position between protocol versions. Supports V2->V3, V3->V4. The migration is atomic -- single transaction, no position downtime.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address |
| `position_id` | `String` | Yes | Current position ID |
| `from_protocol` | `String` | Yes | `"V2"` or `"V3"` |
| `to_protocol` | `String` | Yes | `"V3"` or `"V4"` |
| `chain_id` | `u64` | Yes | Chain ID |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas estimate |

```rust
#[derive(Debug, Serialize)]
pub struct MigrateLiquidityResult {
    pub txs: Vec<TransactionCalldata>,
    pub new_position_id: Option<String>,
    pub gas_estimate: Option<String>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer3` |
| tick_budget | `Slow` |
| progress_steps | `["Checking migration path", "Building exit calldata", "Building enter calldata", "Simulating atomic migration"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Atomically migrates an LP position between protocol versions (V2->V3, V3->V4). Single transaction, no position downtime. Requires API. |

**promptGuidelines:**
- **thriving**: Migrate V3->V4 when V4 pool shows better fee yield. Use uniswap_api_get_lp_pool_info to compare.
- **cautious**: Only migrate if V4 fee yield exceeds V3 by >20%.
- **declining**: Do not migrate. Focus on removing liquidity.
- **terminal**: Do not migrate.

**Ground truth:** `expected_outcome` = position migrated to target protocol. `actual_outcome` = new position ID. `ground_truth_source` = `"api_simulation"`.

**Event Fabric events:** `tool:start` -> `{ position_id, from_protocol, to_protocol }` -> `tool:end` -> `{ new_position_id }`

---

### `uniswap_api_claim_rewards`

Claims protocol incentive rewards for an LP position.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address |
| `position_id` | `String` | Yes | Position ID |
| `chain_id` | `u64` | Yes | Chain ID |
| `simulate_transaction` | `Option<bool>` | No | Simulate and return gas estimate |

```rust
#[derive(Debug, Serialize)]
pub struct ClaimRewardsResult {
    pub txs: Vec<TransactionCalldata>,
    pub reward_amounts: Vec<TokenAmount>,
    pub gas_estimate: Option<String>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer1` |
| tick_budget | `Medium` |
| progress_steps | `["Building claim calldata"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Claims protocol incentive rewards for an LP position. Allowed at any phase. |

**Event Fabric events:** `tool:start` -> `{ position_id }` -> `tool:end` -> `{ reward_amounts }`

---

### `uniswap_api_get_lp_pool_info`

Returns detailed pool information. More complete than `uniswap_get_pool_info`: includes fee income rates and reward program info.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `chain_id` | `u64` | Yes | Chain ID |
| `pool_address` | `Option<String>` | No | Pool contract address |
| `pair_address` | `Option<String>` | No | V2 pair address |
| `pool_id` | `Option<String>` | No | V4 pool ID |

At least one of `pool_address`, `pair_address`, or `pool_id` is required.

```rust
#[derive(Debug, Serialize)]
pub struct LpPoolInfoResult {
    pub pool_id: String,
    pub token0: TokenMeta,
    pub token1: TokenMeta,
    pub fee_tier: u32,
    pub protocol: String,
    pub tvl: f64,
    pub volume_24h: f64,
    pub fees_24h: f64,
    pub current_tick: i32,
    pub sqrt_price_x96: String,
    pub liquidity: String,
    pub reward_programs: Option<Vec<serde_json::Value>>,
    pub hooks: Option<Vec<String>>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer1` |
| tick_budget | `Fast` |
| progress_steps | `["Fetching pool info"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Returns detailed pool info: TVL, volume, fee rates, current tick, reward programs, hooks. More complete than uniswap_get_pool_info. Read-only. |

**Event Fabric events:** `tool:start` -> `{ pool_address }` -> `tool:end` -> `{ tvl, fees_24h }`

---

## Reference tools (category: `data`)

### `uniswap_api_get_bridgeable_tokens`

Returns all destination chains where a given token can be bridged, with bridge protocol for each route.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_in` | `String` | Yes | Token address on source chain |
| `chain_id` | `u64` | Yes | Source chain ID |

```rust
#[derive(Debug, Serialize)]
pub struct BridgeableTokensResult {
    pub destinations: Vec<BridgeDestination>,
}

#[derive(Debug, Serialize)]
pub struct BridgeDestination {
    pub chain_id: u64,
    pub bridge_token: String,
    pub bridge_token_address: String,
    pub bridge_protocol: String,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer1` |
| tick_budget | `Fast` |
| progress_steps | `["Querying bridge routes"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Returns all destination chains where a token can be bridged, with bridge protocol for each route. Use for cross-chain arbitrage discovery. Read-only. |

**Event Fabric events:** `tool:start` -> `{ token_in, chain_id }` -> `tool:end` -> `{ destination_count }`

---

## Wallet utility tools (category: `wallet`)

### `uniswap_api_create_token_transfer`

Generates calldata for an ERC-20 token transfer with server-side gas estimation.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `sender` | `String` | Yes | Sending wallet address |
| `recipient` | `String` | Yes | Recipient address |
| `token` | `String` | Yes | ERC-20 token address |
| `amount` | `String` | Yes | Amount in token base units |
| `chain_id` | `u64` | Yes | Chain ID |

```rust
#[derive(Debug, Serialize)]
pub struct TokenTransferResult {
    pub calldata: String,
    pub to: String,
    pub value: String,
    pub chain_id: u64,
    pub gas_estimate: String,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Write` |
| risk_tier | `Layer3` |
| tick_budget | `Fast` |
| progress_steps | `["Building transfer calldata"]` |
| sprite_trigger | `Executing` |
| prompt_snippet | Generates ERC-20 transfer calldata with server-side gas estimation. Does not execute -- returns calldata for commit_action. |

**promptGuidelines:**
- **thriving**: Transfer as needed for strategy execution.
- **cautious**: Transfer only for operating needs. Smaller amounts.
- **declining**: Transfer only to owner wallet for settlement.
- **terminal**: Transfer only for Death Protocol asset sweep.

**Ground truth:** `expected_outcome` = transfer of amount to recipient. `actual_outcome` = calldata generated. `ground_truth_source` = `"api_gas_simulation"`.

**Event Fabric events:** `tool:start` -> `{ token, amount, recipient }` -> `tool:end` -> `{ calldata_length, gas_estimate }`

---

### `uniswap_api_check_wallet_delegation`

Gets current EIP-7702 delegation status for a wallet across multiple chains.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Smart wallet address to check |
| `chain_ids` | `Vec<u64>` | Yes | Chain IDs to check |

```rust
#[derive(Debug, Serialize)]
pub struct DelegationStatusResult {
    pub delegations: Vec<DelegationEntry>,
}

#[derive(Debug, Serialize)]
pub struct DelegationEntry {
    pub chain_id: u64,
    pub delegated: bool,
    pub delegation_address: Option<String>,
    pub message: Option<String>,
}
```

**ToolDef:**

| Field | Value |
| --- | --- |
| capability | `Read` |
| risk_tier | `Layer1` |
| tick_budget | `Fast` |
| progress_steps | `["Checking delegations"]` |
| sprite_trigger | `Thinking` |
| prompt_snippet | Gets EIP-7702 delegation status for a wallet across multiple chains. Returns re-delegation info if any chain is stale. Read-only. |

**Event Fabric events:** `tool:start` -> `{ wallet_address, chain_count }` -> `tool:end` -> `{ delegations }`

---

## Shared types

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionCalldata {
    pub to: String,
    pub data: String,
    pub value: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAmount {
    pub token: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMeta {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}
```

---

## Custody implications (API write tools)

Write tools that submit orders or execute transactions via the Uniswap Trading API:

- **Delegation**: Session key signs Permit2 messages or order structs. The API handles routing and gas. CaveatEnforcer must whitelist the Permit2 contract and Universal Router. Spending limits apply to order amounts.
- **Embedded**: Privy server wallet signs. PolicyCage validates order parameters and spending limits.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain Permit2 allowances and router targets.

Read-only API tools (quotes, order status, pool queries) require no signing — only the `BARDO_UNISWAP_API_KEY` environment variable.

---

## Capability gates

| Capability | Tools gated | Config flag |
| --- | --- | --- |
| `uniswap_api` | All 21 tools in this file | `api_key` present |
| `smart_wallet` | `uniswap_api_batch_swap`, `uniswap_api_create_delegated_swap` | `enable_smart_wallet = true` |
| `wallet` | `uniswap_api_submit_order`, `uniswap_api_migrate_liquidity` | Wallet configured |
| `limit_orders` | `uniswap_api_get_limit_order_quote` | `enable_limit_orders = true` |

---

## Tool summary

| Tool | Category | Capability | Risk | Budget |
| --- | --- | --- | --- | --- |
| `uniswap_api_get_quote` | trading | Read | Layer 1 | Medium |
| `uniswap_api_check_approval` | trading | Read | Layer 1 | Fast |
| `uniswap_api_create_swap_calldata` | trading | Write | Layer 3 | Medium |
| `uniswap_api_batch_swap` | trading | Write | Layer 3 | Medium |
| `uniswap_api_create_delegated_swap` | trading | Write | Layer 3 | Medium |
| `uniswap_api_submit_order` | trading | Write | Layer 3 | Medium |
| `uniswap_api_get_orders` | trading | Read | Layer 1 | Fast |
| `uniswap_api_check_swap_status` | trading | Read | Layer 1 | Fast |
| `uniswap_api_get_limit_order_quote` | trading | Read | Layer 2 | Medium |
| `uniswap_api_create_lp_position` | lp | Write | Layer 3 | Medium |
| `uniswap_api_increase_lp_position` | lp | Write | Layer 2 | Medium |
| `uniswap_api_decrease_lp_position` | lp | Write | Layer 2 | Medium |
| `uniswap_api_claim_lp_fees` | lp | Write | Layer 1 | Medium |
| `uniswap_api_migrate_liquidity` | lp | Write | Layer 3 | Slow |
| `uniswap_api_claim_rewards` | lp | Write | Layer 1 | Medium |
| `uniswap_api_get_lp_pool_info` | lp | Read | Layer 1 | Fast |
| `uniswap_api_get_bridgeable_tokens` | data | Read | Layer 1 | Fast |
| `uniswap_api_create_token_transfer` | wallet | Write | Layer 3 | Fast |
| `uniswap_api_check_wallet_delegation` | wallet | Read | Layer 1 | Fast |

**Consolidation notes (24 -> 21):**
- `uniswap_api_check_swap_approval` + `uniswap_api_check_lp_approval` merged into `uniswap_api_check_approval`. Both hit the same API endpoint; the only difference was which tokens to check. Now accepts a `tokens` array.
- `uniswap_api_submit_uniswapx_order` renamed to `uniswap_api_submit_order`. Cleaner name, same semantics.
- `uniswap_api_get_uniswapx_orders` renamed to `uniswap_api_get_orders`. Cleaner name, same semantics.
- `uniswap_api_encode_eip7702` removed. It was a thin wrapper around transaction encoding that `uniswap_api_create_delegated_swap` already handles internally. Exposing raw EIP-7702 encoding as a separate tool added no value -- the Golem never needs to encode arbitrary transaction batches outside the swap flow.

**Relationship to core trading/LP tools:** The `uniswap_` prefix tools in [02-tools-onchain-data.md](02-tools-onchain-data.md) and [04-tools-trading.md](04-tools-trading.md) read on-chain state directly via Alloy. The `uniswap_api_` prefix tools in this file call the Uniswap Trading API via `reqwest`. Both coexist. The API tools provide routing, gasless execution, and cross-chain orchestration that on-chain reads cannot. The on-chain tools provide raw data access that doesn't require an API key.
