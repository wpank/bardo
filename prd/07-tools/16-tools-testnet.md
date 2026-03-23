# Bardo Tools -- Testnet tools [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Local testnet lifecycle, time control, and mock deployment. Category: `testnet`. All tools are development-only with no mainnet interaction. 5 tools.

---

> **Reader orientation:** This document specifies 5 development-only testnet tools in `bardo-tools`. These tools spin up local Anvil forks, control block time, deploy mock contracts, and fund test wallets. None interact with mainnet. You should be familiar with Foundry/Anvil for local EVM testing. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Local testnet (2 tools)

### `testnet_setup_local`

Spin up a local Anvil testnet with Uniswap contracts deployed and pre-seeded liquidity. Uses Anvil (Foundry's local EVM node) backed by Revm (a Rust EVM implementation for deterministic transaction execution) for deterministic block production and state forking.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `mode` | `String` | No | `"fork"` (real state from RPC) or `"mock"` (synthetic empty state). Default: `"fork"` |
| `fork_from` | `String` | No | Chain to fork: `"ethereum"`, `"base"`, `"sepolia"`. Default: `"ethereum"` |
| `versions` | `Vec<String>` | No | Uniswap versions to deploy: `["v2", "v3", "v4"]`. Default: all |
| `seed_liquidity` | `bool` | No | Pre-seed pools with liquidity. Default: `true` |
| `funded_accounts` | `u32` | No | Number of test accounts to fund with ETH + tokens. Default: `3` |

```rust
#[derive(Debug, Deserialize)]
pub struct SetupLocalParams {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_fork_from")]
    pub fork_from: String,
    #[serde(default = "default_versions")]
    pub versions: Vec<String>,
    #[serde(default = "default_true")]
    pub seed_liquidity: bool,
    #[serde(default = "default_funded_accounts")]
    pub funded_accounts: u32,
}

#[derive(Debug, Serialize)]
pub struct SetupLocalResult {
    pub rpc_url: String,
    pub chain_id: u64,
    pub block_number: u64,
    pub funded_addresses: Vec<String>,
    pub deployed_contracts: DeployedContracts,
    pub seeded_pools: Vec<SeededPool>,
}

#[derive(Debug, Serialize)]
pub struct DeployedContracts {
    pub v2_factory: Option<String>,
    pub v2_router: Option<String>,
    pub v3_factory: Option<String>,
    pub v3_position_manager: Option<String>,
    pub v4_pool_manager: Option<String>,
    pub v4_position_manager: Option<String>,
    pub permit2: String,
    pub weth: String,
}

#[derive(Debug, Serialize)]
pub struct SeededPool {
    pub address: String,
    pub token0: String,
    pub token1: String,
    pub version: String,
    pub initial_liquidity_usd: f64,
}
```

The handler spawns an Anvil child process via `tokio::process::Command`, waits for the RPC endpoint to become responsive, then deploys contracts from compiled bytecode artifacts. In fork mode, it calls `anvil_reset` with the target RPC URL. In mock mode, it deploys from scratch using `anvil_setCode` for deterministic addresses.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "testnet_setup_local",
    description: concat!(
        "Spin up a local Anvil testnet with Uniswap contracts and pre-seeded liquidity. ",
        "Development-only. Use for strategy testing before mainnet deployment. ",
        "Fork mode copies real chain state; mock mode creates synthetic state.",
    ),
    category: Category::Testnet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Slow,
    progress_steps: &[
        "Spawning Anvil",
        "Deploying contracts",
        "Seeding liquidity",
        "Funding accounts",
    ],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Spin up local Anvil with Uniswap contracts and pre-seeded liquidity. Development-only. Use for strategy testing before mainnet deployment.",
    prompt_guidelines: &[
        "Use fork mode to replay real market conditions against a strategy.",
        "Use mock mode for deterministic unit-test-style scenarios.",
        "Testnet tools have no mainnet interaction. No risk.",
    ],
};
```

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components) **events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ mode, fork_from, versions }` |
| `tool:update` | `{ step: "Spawning Anvil" }`, `{ step: "Deploying contracts" }`, etc. |
| `tool:end` | `{ rpc_url, chain_id, pool_count, account_count }` |

---

### `testnet_stop_local`

Stop a running local Anvil testnet instance and clean up resources. Sends SIGTERM to the Anvil child process, waits for graceful shutdown, then removes any temporary state files.

```rust
#[derive(Debug, Deserialize)]
pub struct StopLocalParams {
    /// Optional: specific RPC URL to stop. If omitted, stops the most recently started instance.
    pub rpc_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StopLocalResult {
    pub stopped: bool,
    pub rpc_url: String,
    pub uptime_seconds: u64,
}
```

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "testnet_stop_local",
    description: "Stop a running local Anvil testnet. Cleans up child process and temp state.",
    category: Category::Testnet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Sending SIGTERM", "Cleaning state"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Stop a running local Anvil testnet. Cleans up resources.",
    prompt_guidelines: &[
        "Call when done testing. Avoids orphan processes.",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ rpc_url }` |
| `tool:end` | `{ stopped, uptime_seconds }` |

---

## Time control (1 tool)

### `testnet_time_travel`

Fast-forward the local testnet's block timestamp. Uses Anvil's `evm_increaseTime` + `evm_mine` JSON-RPC methods (backed by Revm's in-memory state). For testing time-dependent operations: DCA cadences, LP fee accumulation, governance timelocks, vault profit unlock periods.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `seconds` | `u64` | Yes | Seconds to advance |
| `mine_blocks` | `u32` | No | Blocks to mine after advancing. Default: `1` |

```rust
#[derive(Debug, Deserialize)]
pub struct TimeTravelParams {
    pub seconds: u64,
    #[serde(default = "default_one")]
    pub mine_blocks: u32,
}

#[derive(Debug, Serialize)]
pub struct TimeTravelResult {
    pub new_timestamp: u64,
    pub new_block_number: u64,
    pub advanced_seconds: u64,
    pub mined_blocks: u32,
}
```

The handler calls `evm_increaseTime(seconds)` then `evm_mine` for each requested block. Both are Anvil-specific JSON-RPC extensions that manipulate Revm's internal clock and block production without waiting for real time to pass.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "testnet_time_travel",
    description: concat!(
        "Fast-forward local testnet block timestamp. ",
        "Use to test time-dependent operations (DCA, fee accumulation, timelocks). ",
        "Development-only. Calls Anvil's evm_increaseTime + evm_mine.",
    ),
    category: Category::Testnet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Advancing time", "Mining blocks"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Fast-forward local testnet block timestamp. Use to test time-dependent operations (DCA, fee accumulation, timelocks). Development-only.",
    prompt_guidelines: &[
        "Use freely during development. No risk -- testnet only.",
        "Large time jumps (>1 day) may cause stale oracle prices in forked state.",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ seconds, mine_blocks }` |
| `tool:end` | `{ new_timestamp, new_block_number, advanced_seconds }` |

---

## Mock deployment (2 tools)

### `testnet_fund_account`

Fund a test account with ETH and/or ERC-20 tokens on the local testnet. Uses Anvil's `anvil_setBalance` for ETH and `anvil_impersonateAccount` + direct ERC-20 `transfer` for tokens. The impersonation approach works because Anvil's Revm instance skips signature validation for impersonated accounts.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `address` | `String` | Yes | Account address to fund |
| `eth_amount` | `String` | No | ETH amount in wei. Default: `"10000000000000000000"` (10 ETH) |
| `tokens` | `Vec<TokenFunding>` | No | Tokens to fund with amounts |

```rust
#[derive(Debug, Deserialize)]
pub struct FundAccountParams {
    pub address: String,
    #[serde(default = "default_eth_amount")]
    pub eth_amount: String,
    #[serde(default)]
    pub tokens: Vec<TokenFunding>,
}

#[derive(Debug, Deserialize)]
pub struct TokenFunding {
    pub token: String,       // Address or symbol
    pub amount: String,      // Base units
}

#[derive(Debug, Serialize)]
pub struct FundAccountResult {
    pub address: String,
    pub eth_balance: String,
    pub token_balances: Vec<TokenBalance>,
}

#[derive(Debug, Serialize)]
pub struct TokenBalance {
    pub token: String,
    pub symbol: String,
    pub balance: String,
}
```

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "testnet_fund_account",
    description: concat!(
        "Fund a test account with ETH and tokens on local testnet. ",
        "No real value. Development-only. Uses anvil_setBalance and account impersonation.",
    ),
    category: Category::Testnet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Setting ETH balance", "Transferring tokens"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Fund a test account on local testnet. No real value. Development-only.",
    prompt_guidelines: &[
        "Use freely during development. No risk.",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ address, token_count }` |
| `tool:end` | `{ eth_balance, token_count }` |

---

### `testnet_deploy_mock_pool`

Deploy a new liquidity pool on local testnet with configurable parameters. Supports deploying pools with thin liquidity, wide spreads, specific tick ranges, or extreme fee tiers -- conditions that are hard to find on mainnet but common in production edge cases.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token0` | `String` | Yes | Token 0 address or symbol |
| `token1` | `String` | Yes | Token 1 address or symbol |
| `fee_tier` | `u32` | No | Fee tier in bps (100, 500, 3000, 10000). Default: `3000` |
| `initial_price` | `String` | No | Initial price as token1/token0 ratio. Default: `"1.0"` |
| `liquidity_usd` | `f64` | No | Initial liquidity in USD equivalent. Default: `100000.0` |
| `tick_lower` | `Option<i32>` | No | Lower tick bound for initial liquidity |
| `tick_upper` | `Option<i32>` | No | Upper tick bound for initial liquidity |
| `version` | `String` | No | Pool version: `"v3"` or `"v4"`. Default: `"v3"` |

```rust
#[derive(Debug, Deserialize)]
pub struct DeployMockPoolParams {
    pub token0: String,
    pub token1: String,
    #[serde(default = "default_fee_tier")]
    pub fee_tier: u32,
    #[serde(default = "default_price")]
    pub initial_price: String,
    #[serde(default = "default_liquidity")]
    pub liquidity_usd: f64,
    pub tick_lower: Option<i32>,
    pub tick_upper: Option<i32>,
    #[serde(default = "default_v3")]
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct DeployMockPoolResult {
    pub pool_address: String,
    pub token0: String,
    pub token1: String,
    pub fee_tier: u32,
    pub version: String,
    pub initial_tick: i32,
    pub initial_sqrt_price_x96: String,
    pub liquidity: String,
}
```

The handler uses Alloy's `sol!` macro to call the appropriate factory contract (`createPool` for V3, `initialize` for V4). After pool creation, it mints initial liquidity using the position manager. Tick bounds default to full range if not specified.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "testnet_deploy_mock_pool",
    description: concat!(
        "Deploy a new pool on local testnet with configurable parameters. ",
        "Development-only. Use to create specific market conditions for testing: ",
        "thin liquidity, wide spreads, extreme fee tiers, custom tick ranges.",
    ),
    category: Category::Testnet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Creating pool", "Minting initial liquidity", "Verifying state"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Deploy a new pool on local testnet with configurable parameters. Development-only. Use to create specific market conditions for testing.",
    prompt_guidelines: &[
        "Use freely during development. No risk.",
        "Set narrow tick ranges to test out-of-range position behavior.",
        "Set low liquidity_usd to test high-slippage scenarios.",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ token0, token1, version, fee_tier }` |
| `tool:update` | `{ step: "Creating pool" }`, etc. |
| `tool:end` | `{ pool_address, initial_tick, liquidity }` |

---

## Custody implications

Testnet tools operate exclusively on local Anvil instances. No custody signing is involved — all transactions use Anvil's built-in funded accounts. These tools are gated to the `dev` profile and cannot execute on mainnet or public testnets.

---

## Tool summary

| Tool | Capability | Risk | Budget | Profiles |
| --- | --- | --- | --- | --- |
| `testnet_setup_local` | Write | Layer 1 | Slow | `dev` |
| `testnet_stop_local` | Write | Layer 1 | Fast | `dev` |
| `testnet_time_travel` | Write | Layer 1 | Fast | `dev` |
| `testnet_fund_account` | Write | Layer 1 | Fast | `dev` |
| `testnet_deploy_mock_pool` | Write | Layer 1 | Medium | `dev` |

All 5 tools are Write-capability because they mutate local Anvil state. All are Layer 1 risk because no mainnet interaction occurs. The `dev` profile is the only profile that includes testnet tools.
