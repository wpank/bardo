# Golem Tools -- Architecture [SPEC]

**Version:** 5.0.0
**Last Updated:** 2026-03-14

> **Crate**: `golem-tools` | **Workspace**: `crates/tools/` | **Prerequisites**: [00-overview.md](00-overview.md) (goals, design philosophy, personas, and the ToolDef pattern introduction)
>
> Tool architecture, ToolDef pattern, handler signatures, three trust tiers (ReadTool/WriteTool/PrivilegedTool), Capability&lt;T&gt; flow, safety hook chain, Revm simulation, WASM sandbox, Event Fabric integration, memory layer, profiles, TypeScript sidecar, and the two-layer tool model.

---

> **Reader orientation:** This is the architecture spec for `golem-tools`, the DeFi tool crate that ships with the Bardo runtime. It covers the `ToolDef` pattern, the three trust tiers (`ReadTool`/`WriteTool`/`PrivilegedTool`), the `Capability<T>` authorization flow, the safety hook chain, Revm simulation, WASM sandboxing, profile-based tool loading, and the TypeScript sidecar. You should already be familiar with Uniswap V3/V4 pool mechanics and basic Rust ownership semantics. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Rust-native tool library

`golem-tools` is a tool library, not a server. It exports `ToolDef` constants and handler functions that callers invoke directly. No transport layer, no protocol framing, no server process. The crate is `#![no_std]`-compatible for the core types; handler implementations require `alloc` + `tokio`.

The crate contains 423+ DeFi tools covering Uniswap (V2/V3/V4/UniswapX), Aave, Morpho, Pendle, Lido, EigenLayer, GMX, Panoptic, and other protocols. Tools are organized into three trust tiers enforced by the Rust type system:

- **ReadTool** (~250 tools): No capability token required. Cannot modify on-chain state. Examples: read pool state, check balance, get gas price, query health factor.
- **WriteTool** (~150 tools): Require a `Capability<WriteTool>` token that is consumed (moved) on use. Rust's ownership system prevents reuse at compile time. Examples: swap tokens, add liquidity, deposit into vault, stake ETH.
- **PrivilegedTool** (~23 tools): Require a capability token plus owner approval. Examples: change PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) parameters, modify strategy, adjust risk limits.

Two access paths:

- **Golems** (mortal autonomous agents, each compiled as a single Rust binary on a micro-VM) consume tools through Pi extensions. The `golem-tools` extension registers 8 Pi-facing tools that internally resolve to tool implementations via the Tool Adapter Registry. The Golem's LLM never sees 423 definitions; it sees 8 tools (`preview_action`, `commit_action`, etc.) that route to the right handler based on action type. Profile filtering happens at adapter registration time.
- **External agents** consume tools through the A2A interface (JSON-RPC 2.0 task lifecycle). A2A is a separate binary -- it imports handler functions from `golem-tools` but runs its own process.

The tool implementations, safety hook chain, memory layer, and chain provider layer are identical regardless of caller. The adapter layer is a thin mapping between Pi tool schemas (or A2A tasks) and ToolDef handlers.

> Cross-reference: `../01-golem/13-runtime-extensions.md` (runtime extension loading, activation, and lifecycle management within a running Golem) S3 (two-layer tool model), S4 (ActionPermit flow)

---

## ToolDef pattern

Every tool is a module in `crates/tools/src/tools/` exporting a `TOOL_DEF: ToolDef` static:

```rust
use golem_tools::{ToolDef, ToolContext, ToolResult, Category, CapabilityTier, RiskTier, TickBudget};
use serde::{Deserialize, Serialize};

/// Input parameters for uniswap_get_pool_info.
#[derive(Debug, Deserialize)]
pub struct GetPoolInfoParams {
    /// Pool contract address (0x...).
    /// Use uniswap_get_pools_by_token_pair to find the address first.
    pub pool_address: String,
    /// Chain ID (default: 1 for Ethereum). Common: 8453 for Base, 42161 for Arbitrum.
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

fn default_chain_id() -> u64 { 1 }

/// Pool state returned by uniswap_get_pool_info.
#[derive(Debug, Serialize)]
pub struct PoolInfo {
    pub pool_address: String,
    pub chain_id: u64,
    pub version: String,           // "v3" | "v4"
    pub token0: TokenMeta,
    pub token1: TokenMeta,
    pub fee_tier: u32,
    pub sqrt_price_x96: String,
    pub tick: i32,
    pub liquidity: String,
    pub tvl_usd: f64,
    pub volume_24h_usd: f64,
    pub fee_apy_24h: f64,
}

pub static TOOL_DEF: ToolDef = ToolDef {
    name: "uniswap_get_pool_info",
    description: concat!(
        "Get current state of a Uniswap V3 or V4 pool: price, liquidity, TVL, volume, fees. ",
        "Use when the Golem needs pool depth, TVL, current price, or fee APY. ",
        "Returns tick, sqrtPriceX96, liquidity (as token amounts), 24h volume, and fee tier.",
    ),
    category: Category::Data,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,         // <1s
    progress_steps: &["Fetching slot0", "Loading subgraph data", "Computing APY"],
    sprite_trigger: SpriteTrigger::Thinking,

    prompt_snippet: "Use uniswap_get_pool_info for pool state. Call uniswap_get_pools_by_token_pair first to get the pool address.",

    prompt_guidelines: &[
        "Prefer this over manual slot0 reads -- it normalizes V3/V4 differences.",
        "Cache results for 15s. Don't call twice in the same tick for the same pool.",
    ],
};

/// Handler implementation.
pub async fn handle(params: GetPoolInfoParams, ctx: &ToolContext) -> Result<ToolResult> {
    ctx.event_fabric.emit_tool_start("uniswap_get_pool_info", &params);

    let provider = ctx.provider(params.chain_id)?;

    // Step 1: Read slot0
    ctx.event_fabric.emit_tool_update("uniswap_get_pool_info", "Fetching slot0");
    let slot0 = read_slot0(&provider, params.pool_address.parse()?).await?;

    // Step 2: Subgraph data
    ctx.event_fabric.emit_tool_update("uniswap_get_pool_info", "Loading subgraph data");
    let subgraph = ctx.subgraph_client.query_pool(params.pool_address.parse()?).await?;

    // Step 3: Compute APY
    ctx.event_fabric.emit_tool_update("uniswap_get_pool_info", "Computing APY");
    let apy = compute_fee_apy(&slot0, &subgraph);

    let result = PoolInfo { /* ... */ };
    ctx.event_fabric.emit_tool_end("uniswap_get_pool_info", true);

    Ok(ToolResult::read(result))
}
```

### ToolDef fields

| Field | Type | Purpose |
| --- | --- | --- |
| `name` | `&'static str` | Tool name following `<prefix>_<action>_<subject>` convention |
| `description` | `&'static str` | LLM-facing description: when to call, what it returns, what it does NOT do |
| `category` | `Category` | Drives profile filtering (17 categories) |
| `capability` | `CapabilityTier` | `Read`, `Write`, or `Privileged` -- determines handler trait |
| `risk_tier` | `RiskTier` | `Layer1` (read), `Layer2` (bounded write), `Layer3` (unbounded write) |
| `tick_budget` | `TickBudget` | `Fast` (<1s), `Medium` (1-5s), `Slow` (5-15s) |
| `progress_steps` | `&[&str]` | Named execution steps for TUI progress bar rendering |
| `sprite_trigger` | `SpriteTrigger` | Animation state: `Thinking`, `Executing`, `Success`, `Failure` |
| `prompt_snippet` | `&'static str` | Injected near system prompt, cached. 20-50 tokens. |
| `prompt_guidelines` | `&[&str]` | Phase-conditional usage hints injected with tool schema |

### Handler traits: three trust tiers

Tools implement one of three traits based on their trust tier. The Rust type system enforces that write tools cannot execute without a capability token -- this is a compile-time guarantee, not a runtime check. If the safety extension doesn't mint a `Capability<T>`, the code that calls `execute_write` cannot compile.

```rust
/// Tier 1: Read tools -- no capability token required.
/// Cannot modify on-chain state. ~250 tools (~60% of total).
/// Examples: check price, read balance, query pool state, get gas price, read health factor.
#[async_trait]
pub trait ReadTool: Send + Sync {
    fn id(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn execute_read(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult>;
}

/// Tier 2: Write tools -- require a Capability<Self> token, consumed on use.
/// Can broadcast transactions. ~150 tools (~35% of total).
/// The capability is CONSUMED (ownership transferred) on execution.
/// After one use, the capability no longer exists -- Rust's move
/// semantics prevent reuse at compile time.
/// Examples: swap tokens, rebalance LP, deposit, withdraw, stake.
#[async_trait]
pub trait WriteTool: Send + Sync {
    fn id(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn execute_write(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
        capability: Capability<Self>,  // Moved (consumed) on use -- cannot reuse
    ) -> Result<ToolResult>
    where Self: Sized;
}

/// Tier 3: Privileged tools -- capability + owner approval.
/// Admin operations, strategy changes. ~23 tools (~5% of total).
/// Almost never called autonomously -- requires explicit owner steer
/// or multi-sig approval.
/// Examples: change PolicyCage parameters, modify strategy, adjust risk limits.
#[async_trait]
pub trait PrivilegedTool: Send + Sync {
    fn id(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn execute_privileged(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
        capability: Capability<Self>,
        owner_approval: OwnerApproval,
    ) -> Result<ToolResult>
    where Self: Sized;
}
```

### Capability<T> flow

The `Capability<T>` token is the core safety mechanism. It proves that the PolicyCage was checked, the risk engine approved, and an ActionPermit was created -- all before the tool can execute. Even if every other safety mechanism fails, the tool physically cannot run without this token.

The safety extension is the only code that can mint capability tokens (`pub(crate)` constructor). The token flows through a strict pipeline:

```text
LLM proposes action
       |
       v
Safety extension: check PolicyCage, check phase, check spending limits
       |
       v
Risk engine: assess risk tier, check allowlist, simulate via Revm
       |
       v
ActionPermit created (permit_id links to audit chain)
       |
       v
Capability<T> minted with value_limit, expiry, policy_hash
       |
       v
Tool handler receives Capability<T> -- consumed by move semantics
       |
       v
Audit chain records: PermitCreated, ToolCall, PermitConsumed
```

Properties enforced at compile time:
1. **Cannot be created** outside the safety extension (`pub(crate)` constructor)
2. **Cannot be reused** (moved on use -- Rust's ownership system)
3. **Cannot be forged** (no `Default`, no `Clone`, no `Copy`)
4. **Cannot be used after expiry** (checked at execution time)

```rust
use std::marker::PhantomData;

/// Unforgeable, single-use, scoped capability token.
/// The safety extension mints it. The tool handler consumes it.
/// No other path exists.
pub struct Capability<T> {
    pub value_limit: f64,           // Max USD authorized
    pub expires_at: u64,            // Tick expiry
    pub policy_hash: [u8; 32],      // SHA-256 of PolicyCage state at check time
    pub permit_id: String,          // Links to audit trail
    _marker: PhantomData<T>,        // Ties token to specific tool type
}

impl<T> Capability<T> {
    /// Only the safety extension can create capability tokens.
    /// No other code in the system can mint one.
    pub(crate) fn new(
        value_limit: f64,
        expires_at: u64,
        policy_hash: [u8; 32],
        permit_id: String,
    ) -> Self {
        Self { value_limit, expires_at, policy_hash, permit_id, _marker: PhantomData }
    }

    pub fn is_valid(&self, current_tick: u64) -> bool {
        self.expires_at > current_tick
    }
}
```

### Speculative execution and capability tokens

The speculative tool execution engine can only speculate on `ReadTool` types. Speculating on a `WriteTool` is not "checked at runtime and rejected" -- it is impossible to write the code, because `execute_write` requires a `Capability<Self>` parameter that no speculative code path can produce:

```rust
// This compiles -- read tools don't need capabilities:
async fn speculate_read(tool: &dyn ReadTool) {
    tool.execute_read(serde_json::Value::Null, &ctx).await;
}

// This does NOT compile -- no way to construct the Capability:
// async fn speculate_write(tool: &dyn WriteTool) {
//     tool.execute_write(serde_json::Value::Null, &ctx, ???).await;
//     //                                               ^^^ No capability to pass
// }
```

### ToolResult format

All tools return `ToolResult`, which includes expected/actual fields for ground truth verification on write tools:

```rust
#[derive(Debug, Serialize)]
pub struct ToolResult {
    /// The tool output data, serialized as JSON.
    pub data: serde_json::Value,
    /// Whether the tool execution failed.
    pub is_error: bool,
    /// Schema version for response format evolution.
    pub schema_version: u32,
    /// For write tools: what the tool expected to happen.
    pub expected_outcome: Option<String>,
    /// For write tools: what actually happened (from receipt/balance check).
    pub actual_outcome: Option<String>,
    /// Ground truth verification source.
    pub ground_truth_source: Option<String>,
}

impl ToolResult {
    /// Convenience for read-only results.
    pub fn read<T: Serialize>(data: T) -> Self {
        Self {
            data: serde_json::to_value(data).unwrap(),
            is_error: false,
            schema_version: 1,
            expected_outcome: None,
            actual_outcome: None,
            ground_truth_source: None,
        }
    }

    /// Convenience for write results with ground truth.
    pub fn write<T: Serialize>(
        data: T,
        expected: impl Into<String>,
        actual: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            data: serde_json::to_value(data).unwrap(),
            is_error: false,
            schema_version: 1,
            expected_outcome: Some(expected.into()),
            actual_outcome: Some(actual.into()),
            ground_truth_source: Some(source.into()),
        }
    }
}
```

### ToolContext interface

The runtime injects a `ToolContext` providing chain access, safety pipeline, event bus, and memory:

```rust
pub struct ToolContext {
    /// Alloy provider for the specified chain.
    pub fn provider(&self, chain_id: u64) -> Result<Arc<dyn Provider>>;
    /// Alloy signer for write operations.
    pub fn signer(&self, chain_id: u64) -> Result<Arc<dyn Signer>>;
    /// Revm fork for pre-flight simulation.
    pub fn revm_fork(&self, chain_id: u64) -> Result<RevmFork>;
    /// Event Fabric for TUI/surface rendering.
    pub event_fabric: Arc<EventFabric>,
    /// Grimoire memory (optional, active when learning profile on).
    pub grimoire: Option<Arc<Grimoire>>,
    /// Subgraph client for historical data.
    pub subgraph_client: Arc<SubgraphClient>,
    /// Current session config.
    pub config: Arc<ToolConfig>,
    /// Uniswap Trading API client (optional, requires API key).
    pub trading_api: Option<Arc<TradingApiClient>>,
    /// TypeScript sidecar for Uniswap SDK math.
    pub sidecar: Arc<SidecarClient>,
}
```

### Tool annotation semantics

| Annotation | Meaning | Safety effect |
| --- | --- | --- |
| `CapabilityTier::Read` | No on-chain state modification | Safety skips simulation, no capability needed |
| `CapabilityTier::Write` | Broadcasts transactions | Requires `Capability<Self>`, full simulation |
| `CapabilityTier::Privileged` | Admin/ownership operations | Requires capability + owner approval |
| `RiskTier::Layer1` | Read-only | No ActionPermit |
| `RiskTier::Layer2` | Bounded write (value < limit) | Standard ActionPermit |
| `RiskTier::Layer3` | Unbounded write | Elevated ActionPermit, Warden pipeline (optional, deferred) |

### LLM-optimized tool descriptions

Tool descriptions serve two audiences: the LLM selecting which tool to call, and the LLM filling in parameters.

**Selection guidance** (the `description` field): answers "when should I call this tool?"

- Starts with the tool's purpose in a single phrase
- Lists specific intents that map to this tool
- States what it does NOT do (disambiguates from similar tools)
- Mentions prerequisites ("get pool address first via uniswap_get_pools_by_token_pair")

**Parameter documentation** (serde `#[serde(rename)]` + doc comments): answers "how do I fill this in?"

- Format requirements ("0x-prefixed hex address")
- Common values or examples
- Defaults and when to omit

**Anti-patterns** that degrade LLM tool selection accuracy:

- Generic descriptions: "Interact with Uniswap" (does not help selection)
- No parameter docs: missing doc comments cause hallucinated values
- Ambiguous scope: two similar tools with indistinguishable descriptions
- Response payloads exceeding ~25,000 tokens -- implement pagination

---

## 17 tool categories

Every `ToolDef` has a `category: Category` field. Categories drive profile filtering.

| Category | Prefix | Description |
| --- | --- | --- |
| `data` | `data_` | On-chain data reads, pool state, token info, portfolio, P&L |
| `trading` | `uniswap_` | Swap execution, quotes, approvals, order management |
| `lending` | `aave_`, `morpho_` | Supply, borrow, repay, health factor monitoring |
| `staking` | `lido_`, `rocketpool_` | Liquid staking deposits, withdrawals, reward tracking |
| `restaking` | `eigenlayer_` | Restaking, AVS delegation, LRT management |
| `derivatives` | `gmx_`, `panoptic_` | Perpetuals, options, hedging strategies |
| `yield` | `yearn_`, `pendle_`, `ethena_` | Yield aggregators, PT/YT tokenization |
| `lp` | `uniswap_` | Liquidity provision and position management |
| `vault` | `vault_` | ERC-4626 vault operations |
| `safety` | `safety_` | Simulation, risk assessment, circuit breakers |
| `intelligence` | `intel_` | MEV scoring, IL calc, venue comparison, regime classification |
| `memory` | `memory_` | Grimoire episodic and semantic memory |
| `identity` | `identity_` | ERC-8004 agent identity, reputation |
| `wallet` | `wallet_` | Wallet policy, funding, session keys |
| `streaming` | `stream_` | Event Fabric live data subscriptions |
| `testnet` | `testnet_` | Local Anvil testnet management |
| `bootstrap` | `bootstrap_` | First-run setup and provisioning |

### Chain support by tool category

Not every tool category is available on every chain. This matrix shows which chains each category supports for write operations (read operations are available on all chains via RPC).

| Category | Ethereum | Base | Unichain | Arbitrum | Optimism | Polygon | BNB | Avalanche |
|----------|----------|------|----------|----------|----------|---------|-----|-----------|
| trading (Uniswap) | V2/V3/V4 | V2/V3/V4 | V3/V4 | V3 | V3 | V3 | V3 | V3 |
| lending (Aave) | Yes | Yes | No | Yes | Yes | Yes | No | Yes |
| lending (Morpho) | Yes | Yes | No | No | No | No | No | No |
| staking (Lido) | Yes | No | No | No | No | No | No | No |
| restaking (EigenLayer) | Yes | No | No | No | No | No | No | No |
| derivatives (GMX) | No | No | No | Yes | No | No | No | Yes |
| derivatives (Panoptic) | Yes | Yes | Yes | No | No | No | No | No |
| yield (Pendle) | Yes | No | No | Yes | No | No | No | No |
| vault (ERC-4626) | Yes | Yes | Yes | No | No | No | No | No |

Chain support is declared in each tool's `ToolDef` via `supported_chains: &[u64]`. The adapter layer rejects calls targeting unsupported chains with `CHAIN_NOT_SUPPORTED` before any on-chain interaction.

### Risk tier scale

Tools use a three-layer risk classification that gates phase behavior and custody constraints:

- **Layer 1 (low risk)**: Reads, quotes, balance checks. No on-chain state mutation. No capability token required.
- **Layer 2 (medium risk)**: Swaps, standard DeFi operations (supply, withdraw, stake). Require `Capability<WriteTool>`. Subject to spending limits and PolicyCage validation.
- **Layer 3 (high risk)**: Leveraged positions, novel protocols, flash loans, options. Require `Capability<WriteTool>` plus additional simulation in Revm fork before execution. Phase-gated: blocked in conservation/declining/terminal phases.

Risk tier is declared in `ToolDef.risk_tier` and enforced by the safety hook chain before any handler runs.

---

## Prefix convention

All tool names follow `<prefix>_<action>_<subject>`. The prefix identifies the protocol or subsystem.

### Protocol prefixes

| Prefix | Protocol | Example |
| --- | --- | --- |
| `uniswap_` | Uniswap V2/V3/V4/UniswapX | `uniswap_execute_swap` |
| `aave_` | Aave V3 | `aave_supply_collateral` |
| `morpho_` | Morpho Blue | `morpho_supply_market` |
| `curve_` | Curve Finance | `curve_get_pool_info` |
| `lido_` | Lido | `lido_stake_eth` |
| `rocketpool_` | Rocket Pool | `rocketpool_deposit_eth` |
| `eigenlayer_` | EigenLayer | `eigenlayer_delegate_avs` |
| `pendle_` | Pendle | `pendle_buy_pt` |
| `yearn_` | Yearn V3 | `yearn_deposit_vault` |
| `ethena_` | Ethena | `ethena_stake_usde` |
| `gmx_` | GMX V2 | `gmx_open_position` |
| `panoptic_` | Panoptic | `panoptic_buy_option` |

### Subsystem prefixes

| Prefix | Subsystem | Example |
| --- | --- | --- |
| `data_` | On-chain data reads | `data_get_token_price` |
| `safety_` | Safety and simulation | `safety_simulate_transaction` |
| `intel_` | Intelligence/analytics | `intel_assess_mev_risk` |
| `memory_` | Grimoire memory | `memory_store_episode` |
| `identity_` | ERC-8004 identity | `identity_verify_agent` |
| `wallet_` | Wallet management | `wallet_get_status` |
| `vault_` | ERC-4626 vault ops | `vault_deposit` |
| `stream_` | Live data streams | `stream_subscribe_price` |
| `testnet_` | Local testnet | `testnet_time_travel` |
| `bootstrap_` | First-run setup | `bootstrap_setup_wallet` |

---

## Profiles

Set `TOOL_PROFILE` (or `GOLEM_PROFILE`) to control which tool handlers load at boot. Profiles compose -- `TOOL_PROFILE=trader,vault` activates both.

| Profile | Read Tools | Write Tools | Use case |
| --- | --- | --- | --- |
| `active` | All ~250 | All ~150 | Standard active trading Golem -- full read + write access |
| `observatory` | All ~250 | None | Sleepwalker phenotype -- observes, dreams, publishes, never trades |
| `conservative` | All ~250 | ~40 (no leverage, no complex LP, no flashloans) | Risk-averse owner configuration |
| `data` | ~40 | None | Read-only analytics, monitoring, portfolio tracking |
| `trader` | ~60 | ~20 | Swap execution, quotes, approvals, MEV assessment |
| `lp` | ~65 | ~25 | Liquidity provision, position management, fee collection |
| `vault` | ~75 | ~35 | ERC-4626 vault operations, am-AMM bidding |
| `intelligence` | ~58 | None | MEV scoring, IL calculation, venue comparison |
| `learning` | ~52 | ~12 | Memory management, self-improvement |
| `identity` | ~60 | ~20 | ERC-8004 identity, reputation, wallet |
| `golem` | All ~250 | ~150 | Full Golem lifecycle: all categories except testnet |
| `full` | All | All | All tools registered |
| `dev` | All | All + testnet | Full + local testnet tools |

The `observatory` profile is particularly interesting. A Sleepwalker Golem loads only read tools, meaning the code path for executing trades doesn't exist at runtime -- not blocked by a policy check, but structurally absent. The Sleepwalker watches the market, dreams about what it observes, publishes structural insights to the Lethe (formerly Commons), and burns capital at 0.3x the rate of an active Golem (no gas costs, reduced inference).

### Profile-to-category mapping

| Profile | data | trading | lending | staking | restaking | derivatives | yield | lp | vault | safety | intelligence | memory | identity | wallet | streaming | testnet | bootstrap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| active | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | -- | Yes |
| observatory | Yes | -- | -- | -- | -- | -- | -- | -- | -- | -- | Yes | -- | -- | -- | -- | -- | -- |
| conservative | Yes | Yes\* | -- | -- | -- | -- | -- | Yes\* | -- | Yes | -- | -- | -- | -- | -- | -- | -- |
| data | Yes | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- |
| trader | Yes | Yes | -- | -- | -- | -- | -- | -- | -- | Yes | -- | -- | -- | -- | -- | -- | -- |
| lp | Yes | -- | -- | -- | -- | -- | -- | Yes | -- | Yes | -- | -- | -- | -- | -- | -- | -- |
| vault | Yes | -- | -- | -- | -- | -- | -- | -- | Yes | Yes | -- | -- | -- | -- | -- | -- | -- |
| intelligence | Yes | -- | -- | -- | -- | -- | -- | -- | -- | -- | Yes | -- | -- | -- | -- | -- | -- |
| learning | Yes | -- | -- | -- | -- | -- | -- | -- | -- | -- | Yes | Yes | -- | -- | -- | -- | -- |
| identity | Yes | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | -- | Yes | -- | -- | -- | -- |
| golem | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | -- | Yes |
| full | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | -- | Yes |
| dev | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |

\* `conservative` includes a restricted subset of trading and LP write tools -- no leverage, no flashloans, no complex multi-hop strategies.

The `data` category is implicitly included in all profiles. `observatory` includes data + intelligence (read-only). `active` and `golem` are equivalent for tool access. `full` includes everything except testnet. `dev` extends `full` with testnet tools.

### Profile filtering

Profile filtering uses the `ToolDef.category` field. Filtering happens once at extension initialization:

```rust
let allowed = resolve_profile_categories(profile);
let tools: Vec<&ToolDef> = ALL_TOOL_DEFS
    .iter()
    .filter(|t| allowed.contains(&t.category))
    .collect();
```

### Fine-grained overrides

The config file supports per-tool enable/disable that takes precedence over profiles:

```toml
[tools]
profile = "trader"
enable = ["intel_compute_vpin", "intel_compute_lvr"]
disable = ["uniswap_submit_uniswapx_order"]
```

---

## Capability gating

Three capabilities gate tool registration. A tool requiring a capability that isn't present is silently skipped during registration.

| Capability | Required by | How it's satisfied |
| --- | --- | --- |
| `wallet` | All write tools (trading, LP, vault, safety) | A signer is configured (`GOLEM_WALLET_*` env or `golem.toml [wallet]`) |
| `uniswap_api` | API-backed tools | `GOLEM_UNISWAP_API_KEY` is set |
| `memory` | Memory and self-improvement tools | `GOLEM_MEMORY_ENABLED=true` and `learning` profile active |

Capability checking happens once at boot. A `data` profile with no wallet loads all read tools without error. A `trader` profile without a wallet logs a warning and skips write tools.

---

## Event Fabric integration

Every tool emits typed events through the Event Fabric (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components) for TUI rendering, telemetry, and surface updates.

### Tool lifecycle events

| Event | Payload | When |
| --- | --- | --- |
| `tool:start` | `{ tool_name, params_hash, tick }` | Handler entry |
| `tool:update` | `{ tool_name, step_name, step_index, total_steps }` | Each progress step |
| `tool:end` | `{ tool_name, success, duration_ms, result_summary }` | Handler exit |
| `tool:error` | `{ tool_name, error_code, error_message }` | Handler failure |

### TUI rendering contract

The TUI subscribes to `tool:*` events and renders them according to the tool's metadata:

- **`progress_steps`**: Drives a step-by-step progress bar. Each `tool:update` event advances the bar.
- **`sprite_trigger`**: Sets the Golem sprite animation state (`Thinking` for reads, `Executing` for writes, `Success`/`Failure` on completion).
- **`tick_budget`**: The TUI uses this to estimate expected duration and show appropriate loading states.

### Event emission pattern

```rust
// Every handler follows this pattern:
pub async fn handle(params: P, ctx: &ToolContext) -> Result<ToolResult> {
    ctx.event_fabric.emit(Subsystem::Tools, EventPayload::ToolExecutionStart {
        tool_name: TOOL_DEF.name.into(),
        params_hash: hash_params(&params),
    });

    // Step 1
    ctx.event_fabric.emit_tool_update(TOOL_DEF.name, "Fetching data");
    let data = fetch(&ctx.provider(chain_id)?).await?;

    // Step 2
    ctx.event_fabric.emit_tool_update(TOOL_DEF.name, "Processing");
    let result = process(data)?;

    ctx.event_fabric.emit(Subsystem::Tools, EventPayload::ToolExecutionComplete {
        tool_name: TOOL_DEF.name.into(),
        success: true,
        duration_ms: elapsed.as_millis() as u64,
    });

    Ok(ToolResult::read(result))
}
```

---

## High-level architecture

### Primary path: Golem via Pi extensions

```text
Golem (Pi Session)
       |
       v
+--- Pi Extension Layer ----------------------------------+
|  golem-tools: 8 Pi-facing tools                         |
|  golem-safety: PolicyCage + phase enforcement            |
|  golem-permits: ActionPermit lifecycle                   |
|  All 19 extensions in dependency order                   |
+---------+------------------------------------------------+
          |
          v
+--- Tool Adapter Registry --------------------------------+
|  Pi-facing tool -> ToolDef handler resolution             |
|  Profile-filtered: trader, vault, lp, etc.                |
+---------+------------------------------------------------+
          |
          v
+--- golem-tools crate ------------------------------------+
|  Data (~40) | Trading (~20) | Lending (~27) | LP (~28)   |
|  Staking (~16) | Restaking (~16) | Derivatives (~16)     |
|  Yield (~20) | Vault (~40) | Safety (~16) | Intel (~18)  |
|  Memory (~13) | Identity (~24) | Wallet (~8) | Stream (6)|
|  423+ tools, all ToolDef + handler functions              |
+---------+------------------------------------------------+
          | (unchanged from here down)
          v
   [Grimoire -> Safety Hook Chain -> Alloy Provider -> Signer -> Revm]
```

### Secondary path: External agents via A2A

```text
External Agent
       |
       v
+--- A2A Interface ----------------------------------------+
|  JSON-RPC 2.0 task lifecycle                              |
|  Agent Card at /.well-known/agent.json                    |
|  Imports handlers from golem-tools                        |
+---------+------------------------------------------------+
          |
          v
   [Same golem-tools handler layer]
```

### Shared lower stack

```text
+--- Grimoire (Memory Layer) ----------------------------------+
|  LanceDB (episodic) | SQLite (semantic) | Filesystem (strat) |
|  Reflexion | ExpeL | Ebbinghaus decay | Dream hooks          |
|  Optional: active when `learning` profile is on              |
+---------+----------------------------------------------------+
          |
          v
+--- Safety Hook Chain ----------------------------------------+
|  on_tool_call chain: safety -> permits -> risk -> filter     |
|  Capability token minting + consumption                      |
|  Revm simulation (pre-flight fork, not eth_call)             |
|  PolicyCage enforcement + phase gating                       |
+---------+----------------------------------------------------+
          |
          v
+--- Alloy Provider Layer -------------------------------------+
|  sol! macro for type-safe contract bindings                   |
|  11 chains | RPC pool | retry | block caching                |
+---------+----------------------------------------------------+
          |
          v
+--- Signer Abstraction ----------------------------------------+
|  Local key | Privy (HTTP) | Safe | ZeroDev | generic Alloy    |
|  All normalized to Alloy's Signer trait                       |
+---------+-----------------------------------------------------+
          |
          v
+--- TypeScript Sidecar -----------------------------------------+
|  Unix socket IPC (~1-5ms)                                      |
|  Uniswap SDK math (smart-order-router, v3-sdk, v4-sdk)         |
|  Called only for routing/position math that hasn't been ported  |
+----------------------------------------------------------------+
```

---

## Tool modules

| Module | Count | Category | Capability | Write ops |
| --- | --- | --- | --- | --- |
| On-chain data reads | 9 | data | Read | 0 |
| Trading (Uniswap) | 5 | trading | Write | 4 |
| Uniswap API | ~20 | trading | Write | ~12 |
| Lending (Aave, Morpho, MakerDAO) | ~27 | lending | Write | ~18 |
| LP management | ~28 | lp | Write | ~18 |
| Vault core | ~40 | vault | Write | ~25 |
| Staking (Lido, Rocket Pool) | ~16 | staking | Write | ~8 |
| Restaking (EigenLayer, LRTs) | ~16 | restaking | Write | ~10 |
| Derivatives (GMX, Panoptic) | ~16 | derivatives | Write | ~10 |
| Yield (Yearn, Pendle, Convex, Ethena) | ~20 | yield | Write | ~12 |
| Bridge + Aggregator | 9 | trading | Write | 4 |
| Safety and simulation | ~16 | safety | Read/Write | ~4 |
| Intelligence and analytics | ~18 | intelligence | Read | 0 |
| Memory | ~13 | memory | Write | ~5 |
| Identity + Wallet | ~24 | identity/wallet | Read/Write | ~8 |
| Streaming | 6 | streaming | Read | 0 |
| Testnet | 5 | testnet | Write | 4 |
| Bootstrap | 3 | bootstrap | Write | 2 |
| **Total** | **423+** | | | **~150** |

All write operations pass through the full safety hook chain.

---

## Two-layer tool model

### DeFi adapter pattern

The Golem's LLM never calls protocol-specific tools directly. It calls 8 Pi-facing tools (`preview_action`, `commit_action`, `cancel_action`, `emergency_halt`, `query_state`, `search_context`, `query_grimoire`, `update_directive`). Each call resolves to a specific ToolDef handler through the Tool Adapter Registry maintained by the `golem-tools` extension.

When the LLM calls `preview_action({ action_type: "deposit", venue: "morpho", asset: "USDC", amount: "50000000000" })`, the adapter layer resolves the venue to a protocol-specific handler, constructs the calldata via Alloy's `sol!` macro, classifies the risk tier, and returns an `AdapterResolution` that routes to the internal handler.

```rust
pub struct AdapterResolution {
    /// The internal ToolDef to invoke.
    pub internal_tool: &'static ToolDef,
    /// Transformed parameters (Pi-facing schema -> internal schema).
    pub transformed_params: serde_json::Value,
    /// Risk tier for ActionPermit routing.
    pub risk_tier: RiskTier,
}

#[derive(Debug, Clone, Copy)]
pub enum RiskTier {
    Routine,     // Read-only, informational
    Standard,    // Bounded value write
    Elevated,    // Large value or complex operation
    High,        // Cross-chain, V4 hooks, leverage
    Critical,    // Ownership/admin operations
}
```

### Adapter registry

Each protocol has a typed adapter that wraps tool handlers with regime-aware parameter defaults. Adapter methods call specific tool handlers and return structured results used by heartbeat probes.

The `golem-tools` extension loads appropriate adapters based on the GolemManifest's `allowed_protocols` field. Only adapters for declared protocols are registered. An adapter for a protocol not in `allowed_protocols` is never instantiated.

### Risk tier classification

| Category | Default risk tier | Escalation conditions |
| --- | --- | --- |
| Lending (supply/withdraw) | Standard | > 100K USD: Elevated |
| LP (add/remove liquidity) | Elevated | V4 hooks: High |
| Swaps | Standard | > 50K USD: Elevated, cross-chain: High |
| Staking | Standard | > 100K USD: Elevated |
| Ownership/admin operations | Critical | Always |

Risk tiers feed into the ActionPermit system. Standard-tier permits execute immediately. Elevated and above route through the Warden's announce-wait-execute pipeline (requires optional Warden module, deferred; see `prd2-extended/10-safety/02-warden.md`).

### Tool pruning

The task classifier in `golem-model-router` determines which 12 or fewer tools to expose per tick. The rest are deferred -- present in the adapter registry but not included in the LLM's context window for that inference call. The classifier reads current probe results and regime to select the most relevant tool subset.

### Profile-specific adapter sets

Each profile gets a different set of adapters. The `data` profile is structurally unable to trade -- it has no `preview_action` or `commit_action` adapters. Not gated by a flag. Not blocked by a policy check. The routing entries don't exist.

```rust
fn register_adapters(profile: &ToolProfile) -> AdapterRegistry {
    let mut registry = AdapterRegistry::new();

    // All profiles get query adapters
    registry.add("query_state", "portfolio", &GET_PORTFOLIO_SNAPSHOT);
    registry.add("search_context", "price", &GET_TOKEN_PRICE);
    registry.add("search_context", "pool", &GET_POOL_INFO);

    match profile {
        ToolProfile::Trader | ToolProfile::Golem | ToolProfile::Full | ToolProfile::Dev => {
            registry.add("preview_action", "swap", &SIMULATE_SWAP);
            registry.add("commit_action", "swap", &EXECUTE_SWAP);
        }
        ToolProfile::Lp | ToolProfile::Golem | ToolProfile::Full | ToolProfile::Dev => {
            registry.add("preview_action", "add_liquidity", &SIMULATE_ADD_LIQUIDITY);
            registry.add("commit_action", "add_liquidity", &EXECUTE_ADD_LIQUIDITY);
            registry.add("preview_action", "remove_liquidity", &SIMULATE_REMOVE_LIQUIDITY);
            registry.add("commit_action", "remove_liquidity", &EXECUTE_REMOVE_LIQUIDITY);
        }
        ToolProfile::Vault | ToolProfile::Golem | ToolProfile::Full | ToolProfile::Dev => {
            registry.add("preview_action", "deposit", &VAULT_PREVIEW_DEPOSIT);
            registry.add("commit_action", "deposit", &VAULT_DEPOSIT);
            registry.add("preview_action", "withdraw", &VAULT_PREVIEW_WITHDRAW);
            registry.add("commit_action", "withdraw", &VAULT_WITHDRAW);
        }
        ToolProfile::Data => {
            // Read-only. NO preview_action or commit_action adapters.
        }
        _ => {}
    }

    registry
}
```

---

## Alloy integration

All on-chain interaction uses Alloy (Paradigm's Rust Ethereum toolkit). The `sol!` macro generates type-safe Rust bindings from Solidity function signatures at compile time.

```rust
use alloy::{sol, providers::Provider, primitives::*};

sol! {
    /// Uniswap V3 Pool slot0 read.
    function slot0() external view returns (
        uint160 sqrtPriceX96,
        int24 tick,
        uint16 observationIndex,
        uint16 observationCardinality,
        uint16 observationCardinalityNext,
        uint8 feeProtocol,
        bool unlocked
    );

    /// ERC-20 balance check.
    function balanceOf(address owner) external view returns (uint256);

    /// ERC-4626 vault deposit.
    function deposit(uint256 assets, address receiver) external returns (uint256 shares);
}

/// Read pool state using type-safe bindings.
async fn read_slot0(provider: &dyn Provider, pool: Address) -> Result<Slot0Return> {
    let call = slot0Call {};
    let result = provider.call(call.abi_encode(), pool).await?;
    Ok(slot0Call::abi_decode_returns(&result)?)
}
```

Advantages over the previous viem approach:

- **Type-safe at compile time**: Solidity signatures compile to Rust types. No ABI JSON, no runtime decode errors.
- **No codegen step**: The `sol!` macro runs at compile time as a procedural macro.
- **60% faster arithmetic**: Alloy's `U256` operations in native Rust vs JavaScript BigInt.
- **Zero-copy decoding**: ABI decoding reads directly from the response buffer.

### TypeScript sidecar

Uniswap SDKs are 50,000+ lines of TypeScript. Porting them would take months. Instead, a co-located Node.js process handles SDK math via Unix domain socket (~1-5ms latency):

```rust
pub struct SidecarClient {
    socket_path: PathBuf,
}

impl SidecarClient {
    pub async fn find_best_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount: U256,
        chain_id: u64,
    ) -> Result<SwapRoute> {
        let params = serde_json::json!({
            "tokenIn": token_in.to_string(),
            "tokenOut": token_out.to_string(),
            "amount": amount.to_string(),
            "chainId": chain_id,
        });
        let result = self.call("findBestRoute", params).await?;
        Ok(serde_json::from_value(result)?)
    }
}
```

The sidecar runs `@uniswap/smart-order-router`, `v3-sdk`, `v4-sdk`, `permit2-sdk`, and `uniswapx-sdk`. It starts automatically with the Golem and restarts on crash.

---

## Revm simulation

Pre-flight simulation uses Revm (Rust EVM implementation) instead of `eth_call`. Revm provides a local fork of chain state that supports multi-step simulation, state inspection, and gas profiling.

```rust
pub async fn simulate_swap(
    ctx: &ToolContext,
    chain_id: u64,
    calldata: &[u8],
    to: Address,
    value: U256,
) -> Result<SimulationResult> {
    let mut fork = ctx.revm_fork(chain_id)?;

    // Execute the transaction in the fork
    let result = fork.transact(calldata, to, value)?;

    // Inspect state changes
    let balance_before = fork.balance_of(ctx.signer_address(), token)?;
    let balance_after = fork.balance_of_post(ctx.signer_address(), token)?;

    Ok(SimulationResult {
        success: result.is_success(),
        gas_used: result.gas_used(),
        output: result.output().to_vec(),
        state_changes: fork.diff(),
        balance_delta: balance_after - balance_before,
    })
}
```

Advantages over `eth_call`:
- **Multi-step simulation**: Execute approve + swap + verify in one fork
- **State inspection**: Read balances before and after without separate calls
- **No RPC round-trips**: One fork creation, then all simulation is local
- **Deterministic gas**: Fork snapshot at specific block, not racing against mempool

---

## WASM sandbox for untrusted tools

The 423+ native DeFi tools run unsandboxed at full Rust speed -- they're part of the reviewed, compiled codebase. But untrusted tools (user-provided, marketplace-purchased, third-party MCP tools) run inside a WASM sandbox using Wasmtime.

The sandbox applies two resource limits:

**Fuel metering**: Each WASM instruction consumes "fuel." When fuel runs out, execution halts. This prevents infinite loops and runaway computation. Default: 10 million fuel units (~100ms of computation).

**Epoch interruption**: A wall-clock timeout enforced by a background tokio task that increments the Wasmtime engine epoch. This catches cases where fuel metering alone doesn't prevent long execution (tight loops consuming little fuel per iteration). Default: 5 seconds.

```rust
pub struct WasmSandbox {
    engine: wasmtime::Engine,
    fuel_limit: u64,         // default: 10_000_000
    timeout: Duration,       // default: 5s
    memory_limit: usize,     // default: 256MB
}

impl WasmSandbox {
    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        let mut store = wasmtime::Store::new(&self.engine, ());
        store.set_fuel(self.fuel_limit)?;

        // Epoch-based timeout
        let engine = self.engine.clone();
        let timeout = self.timeout;
        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;
            engine.increment_epoch();
        });

        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;
        let execute_fn = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "execute")?;

        // Marshal params -> WASM memory, call, unmarshal result
        // ...
        Ok(result)
    }
}
```

Sandboxed tools receive a restricted interface -- read operations only. Any write operation must be returned as a request that the host validates through the normal safety hook chain before executing. The sandbox has no filesystem access, no network access, and no access to the Golem's wallet keys.

Every sandbox execution emits `GolemEvent::WasmToolStart` and `GolemEvent::WasmToolComplete` events for audit trail and TUI rendering.

---

## Safety hook chain

All write operations pass through the safety hook chain before execution. The chain is implemented as Pi extension lifecycle hooks (`on_tool_call`) that fire in dependency order.

```rust
/// Safety hook chain -- each hook can approve, reject, or modify the tool call.
pub trait SafetyHook: Send + Sync {
    async fn on_tool_call(
        &self,
        tool: &ToolDef,
        params: &serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<HookDecision>;
}

pub enum HookDecision {
    /// Allow the tool call to proceed.
    Allow,
    /// Allow with modified parameters.
    AllowModified(serde_json::Value),
    /// Reject the tool call with a reason.
    Reject(String),
}
```

Default hook chain order:

1. **PolicyCage**: Phase enforcement (conservation = unwind only, terminal = blocked)
2. **AllowlistGuard**: Token and contract allowlist check
3. **SpendingLimiter**: Per-tick and per-day USD spending limits
4. **RateLimiter**: Max operations per time window
5. **RevmSimulator**: Pre-flight simulation in Revm fork
6. **HallucinationDetector**: Verify addresses and amounts against known state
7. **ResultFilter**: Sanitize output (strip sensitive data, cap response size)

Each hook emits events to the Event Fabric for TUI rendering of safety check progress.

---

## TaintedString: sensitive data handling

Sensitive data (private keys, API keys, session tokens) is wrapped in `TaintedString`, which provides automatic zeroization on drop and information flow control:

```rust
pub struct TaintedString {
    value: zeroize::Zeroizing<String>,
    labels: HashSet<TaintLabel>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintLabel {
    WalletSecret,          // Never leaves process
    OwnerSecret,        // Never to LLM context or Styx
    StrategyConfidential,  // Never to lethe
    UserPII,               // Never to lethe without anon
    UntrustedExternal,     // Must validate before use
}

impl TaintedString {
    pub fn can_flow_to(&self, sink: DataSink) -> bool {
        match sink {
            DataSink::LlmContext => {
                !self.labels.contains(&TaintLabel::WalletSecret)
                && !self.labels.contains(&TaintLabel::OwnerSecret)
            }
            DataSink::EventFabric => {
                !self.labels.contains(&TaintLabel::WalletSecret)
            }
            DataSink::StyxLethe => {
                !self.labels.contains(&TaintLabel::StrategyConfidential)
                && !self.labels.contains(&TaintLabel::UserPII)
                && !self.labels.contains(&TaintLabel::WalletSecret)
            }
        }
    }
}
```

---

## DecisionCycleRecord integration

Tools contribute data to the Golem's 9-step heartbeat decision cycle via the `DecisionCycleRecord`. Every tool execution produces an `ActionRecord` that becomes part of the tick's permanent record:

```rust
pub struct ActionRecord {
    pub action_type: String,           // "swap", "rebalance", "deposit"
    pub tool_name: String,             // The ToolDef.name that executed
    pub permit_id: Option<String>,     // Links to capability token
    pub tx_hash: Option<String>,       // On-chain transaction hash
    pub status: ActionStatus,          // Executed, Blocked, Deferred
    pub block_reason: Option<String>,  // Why it was blocked (if blocked)
    pub gas_cost: f64,                 // Gas cost in USD
}

pub struct OutcomeRecord {
    pub verified: bool,
    pub expected: String,              // From ToolResult.expected_outcome
    pub actual: String,                // From ToolResult.actual_outcome
    pub pnl_impact: Option<f64>,       // P&L change from this action
    pub ground_truth_source: String,   // "receipt", "balance_check", "log_comparison"
}
```

The `OutcomeRecord` feeds back into the Grimoire -- if expected and actual diverge, the episode is tagged for Dream replay and heuristic revision.

---

## promptSnippet and promptGuidelines

Pi tool definitions accept two fields for zero-cost context engineering:

- **`prompt_snippet`**: Short string injected near the system prompt. Always present when the tool is loaded. ~20-50 tokens, cached by the provider's prompt caching system.
- **`prompt_guidelines`**: Array of usage hints injected as part of the tool schema. Also cached alongside tool definitions.

These fields replace stuffing tool usage instructions into the system prompt. Each tool carries its own instructions, present only when the tool is loaded.

### Phase-conditional guidelines

Guidelines reference behavioral phases directly. The LLM reads them and self-enforces -- no runtime branching needed:

```rust
pub static COMMIT_ACTION_DEF: ToolDef = ToolDef {
    name: "commit_action",
    // ...
    prompt_snippet: "Executes a previewed action. Requires a valid, unexpired permit ID. After committing, ALWAYS verify the outcome with query_state.",

    prompt_guidelines: &[
        "NEVER call commit_action without a valid permitId from preview_action.",
        "If more than 3 minutes have passed since the preview, re-preview first.",
        "After committing, call query_state to verify the state change occurred.",
        "If commit fails, DO NOT retry immediately -- check query_state first.",
        // Phase-conditional (LLM reads and self-enforces):
        "In conservation phase: only commit close/unwind actions. New positions will be blocked.",
        "In declining phase: only commit settlement actions. The system will block all other commits.",
        "In terminal phase: DO NOT attempt commits. The Death Protocol handles all remaining actions.",
    ],
};
```

### Token savings: full stack analysis

| Configuration | Tool tokens per turn | Context savings vs baseline |
| --- | --- | --- |
| All 423+ tools directly exposed (baseline) | ~38,000 | -- |
| 8 Pi-facing tools (two-layer model) | ~1,200 | **94% reduction** |
| 8 Pi-facing + 5 skill descriptions (dormant) | ~1,450 | **92% reduction** |
| 8 Pi-facing + 2 skills active (loaded) | ~2,800 | **85% reduction** |

The savings compound across a Golem's lifetime. 19,000 tokens saved per turn, roughly 20 T1 turns per day, at $0.001/1K tokens = ~$0.38/day. Over 30 days that's $11.40 -- enough to extend a Golem burning $0.20/day by 57 days.

---

## Speculation engine

Read tools support prefetching via co-occurrence patterns. When a tool is called, the speculation engine checks historical co-occurrence data and prefetches likely-next-read tools in parallel:

```rust
pub struct SpeculationEngine {
    /// Tool co-occurrence matrix (tool_a, tool_b) -> probability.
    co_occurrence: HashMap<(&'static str, &'static str), f64>,
    /// Minimum probability to trigger prefetch.
    threshold: f64, // default: 0.7
}

impl SpeculationEngine {
    pub fn on_tool_call(&self, tool_name: &str) -> Vec<PrefetchTask> {
        self.co_occurrence
            .iter()
            .filter(|((a, _), prob)| *a == tool_name && **prob >= self.threshold)
            .map(|((_, b), _)| PrefetchTask { tool_name: b })
            .collect()
    }
}
```

Example co-occurrences:
- `uniswap_get_pool_info` -> `data_get_token_price` (0.85)
- `uniswap_get_quote` -> `safety_simulate_transaction` (0.92)
- `aave_get_health_factor` -> `data_get_token_price` (0.78)

Prefetched results are cached for the duration of the tick. If the Golem doesn't use them, they're discarded at tick end.

---

## Memory layer architecture (Grimoire)

The memory layer is an optional, fully embedded self-improving system that augments tool execution with historical context. Active when `TOOL_PROFILE` includes `learning`. Zero external service dependencies.

### Triple-store design

| Store | Technology | Purpose | Query pattern |
| --- | --- | --- | --- |
| **Episodic** | LanceDB (Lance columnar) | Raw trade outcomes, reflections, snapshots | Hybrid BM25 + vector similarity via RRF. Sub-25ms. |
| **Semantic** | SQLite + `sqlite-vec` | Distilled insights, confidence, decay | SQL + optional KNN. Sub-millisecond. |
| **Strategic** | Filesystem (TOML) | Heuristics, strategies, causal links | File reads + pattern matching. |

### Embedding pipeline

Local embeddings via `fastembed-rs` with `all-MiniLM-L6-v2`:

- **Dimensions**: 384
- **Quantization**: INT8 -- ~23MB model
- **Latency**: ~5-15ms per sentence (faster than JS Transformers.js)
- **Offline-capable**: Model cached locally after first download

### Memory lifecycle

```text
Tool Call Received
       |
       v
  RETRIEVE: Embed params -> search LanceDB (top-k) -> query SQLite (insights >= threshold)
       |
       v
  AUGMENT: Adjust SOFT parameters only (slippage, timing, route)
           Memory CANNOT override safety limits, allowlist, spending caps, simulation
       |
       v
  [Normal Tool Execution via Safety Hook Chain]
       |
       v
  REFLECT: Reflexion pattern -- structured self-reflection: predicted vs actual
       |
       v
  STORE: Episode in LanceDB + tags. Periodically: ExpeL consolidation -> SQLite insights.
```

### Ebbinghaus decay

Insights decay: `R = e^(-t/S)` where `R` is retention (0-1), `t` is time since last access, `S` is stability (days).

- **Base stability**: 7 days
- **Access reinforcement**: Each retrieval increases stability by +50%
- **Importance modulation**: MEV insights get 90-day stability; emergency conditions get 180+
- **Minimum retention**: 0.1 -- insights are never fully forgotten

### Dream integration

During Dream cycles (NREM/REM), the Grimoire replays high-prediction-error episodes for heuristic revision. Tools that produced surprising outcomes (expected != actual in ToolResult) are prioritized for replay.

---

## Data flow: execute swap

```text
Golem calls: preview_action({ type: "swap", token_in: "USDC", token_out: "WETH", amount: "500" })
         |
         v
  Adapter resolves: preview_action:swap -> SIMULATE_SWAP tool
         |
         v
  Handler: Validate params, resolve chain
         |
         v
  Grimoire Retrieve (if learning profile active):
    Embed "swap USDC->WETH 500" -> search similar episodes + relevant insights
         |
         v
  Alloy: sol!{ function slot0() } read for current price
  Sidecar: findBestRoute(USDC, WETH, 500e6, 1) for optimal route
         |
         v
  Revm Simulation:
    1. Fork chain state at current block
    2. Execute approve (if needed) in fork
    3. Execute swap in fork
    4. Read token balances before/after
    5. Compute price impact, gas cost
         |
         v
  Safety Hook Chain:
    1. PolicyCage: phase allows trading?
    2. AllowlistGuard: USDC, WETH in allowlist?
    3. SpendingLimiter: $500 < daily limit?
    4. RateLimiter: < max swaps per hour?
    5. HallucinationDetector: addresses match token list?
    ANY hook rejects -> return structured error, mint no capability
         |
         v
  Return ActionPermit (simulation result, gas estimate, risk tier, permit_id)
         |
         v
  Golem calls: commit_action({ permit_id: "..." })
         |
         v
  Safety mints Capability<ExecuteSwap> -> consumed by execute_write
         |
         v
  Alloy Signer: Sign and broadcast transaction
         |
         v
  Chain: Wait for receipt, verify success
         |
         v
  ToolResult::write(result, expected_output, actual_output, "receipt")
         |
         v  (if learning profile active)
  Grimoire Store: Episode with reflection + outcome tags
```

---

## Tick budget enforcement

Tools must complete within their declared `tick_budget`. The tool runner enforces this with `tokio::time::timeout`:

| Budget | Max duration | Use case |
| --- | --- | --- |
| `Fast` | 1 second | Data reads, cached lookups |
| `Medium` | 5 seconds | Subgraph queries, API calls, simulation |
| `Slow` | 15 seconds | Transaction broadcast + confirmation |

Tools exceeding their budget are cancelled. Long-running tools must yield intermediate results via `tool:update` events and checkpoint their state so they can resume if retried.

The heartbeat theta-tick budget (adaptive, 30-120s depending on regime) is the outer bound. If a tool consumes most of a tick's budget, only the remainder is available for other tool calls in that tick.

---

## Schema versioning

- Every tool MUST declare `schema_version: u32` in its `ToolResult` (starting at 1).
- Breaking changes to parameter or response schema MUST ship as a new version. Old version remains available for >= 90 days.
- Non-breaking additions (new optional params, new response fields) do NOT require a version bump but MUST increment `schema_version`.

---

## A2A external interface

`golem-tools` handlers are consumed by a separate A2A service for external agents. A2A handles multi-turn dialogue with a task lifecycle (`submitted -> working -> input_required -> completed`):

- `/a2a` -- JSON-RPC 2.0 (task management, multi-turn dialogue)
- `/.well-known/agent.json` -- A2A Agent Card (public, no auth)

The A2A Agent Card URL is embedded in the ERC-8004 registration's `services` array: on-chain identity -> agentURI -> registration file -> services -> Agent Card.

---

## ERC-8004 entity registration

The Golem tool infrastructure registers as an ERC-8004 agent with `role: "infrastructure"` metadata:

1. **Trust anchor**: Investors verify that a vault manager is backed by the Golem protocol
2. **Infrastructure reputation**: Uptime and reliability tracked via the infrastructure feedback track
3. **Discoverability**: Other agents find Golem infrastructure through standard ERC-8004 search
4. **Validation chain**: Infrastructure can submit `validationRequest()` for agents it has onboarded
