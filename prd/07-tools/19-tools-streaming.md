# Bardo Tools -- Streaming tools (Pi event subscriptions) [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles), [21-profiles.md](21-profiles.md) (profile-based tool loading system with 12+ profiles)
>
> Pi replaces SSE/WebSocket transports with native event subscriptions via `session.on()`. Tools register event handlers that fire on chain events, price updates, and position changes. Category: `streaming`. 6 tools.

---

> **Reader orientation:** This document specifies 6 streaming tools in `bardo-tools` for real-time event subscriptions. These tools register handlers on the Event Fabric's broadcast channel for chain events, price updates, and position changes. No WebSocket or SSE connections -- events flow through Bardo's native `tokio::broadcast` system. You should be familiar with async event-driven patterns and EVM log subscriptions. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Architecture

Streaming tools translate on-chain and off-chain real-time data into typed GolemEvent variants. No WebSocket connections. No SSE streams. The agent calls a tool that registers a handler on the Event Fabric's `tokio::broadcast` channel. Events fire asynchronously into the ring buffer. Cleanup happens automatically via `on_unmount` when the session ends, or explicitly via `stream_unsubscribe`.

Each streaming tool:

1. Spawns a `tokio::task` that produces events (log subscription or polling loop)
2. Emits typed `GolemEvent` variants through the Event Fabric's broadcast channel
3. Creates `session_state` entries tracking the active subscription
4. Defines `on_unmount` cleanup that aborts the task and removes the subscription

All streaming tools are in category `streaming` and included in profiles: `data`, `trader`, `lp`, `golem`, `full`, `dev`.

### Event Fabric integration

The Event Fabric (see `14-events.md`) is a `tokio::broadcast` channel with a ring buffer of 10,000 entries for reconnection replay. Every event carries a monotonic `sequence: u64` -- gaps indicate the subscriber lagged behind channel capacity. The surface can request replay from the ring buffer using `resume_from`.

Streaming tools emit two kinds of events:

- **Tool lifecycle events** (`GolemEvent::ToolExecutionStart`, `GolemEvent::ToolExecutionEnd`) -- standard pattern, emitted once when the subscription is created or destroyed.
- **Stream data events** -- continuous, emitted every time new data arrives. These flow through the broadcast channel to the TUI panes and to the agent's event queue.

Serialization cost is paid only when a subscriber exists. The `emit()` path checks subscriber count first; zero subscribers means zero work beyond the function call overhead.

### GolemEvent mapping

Each streaming tool produces a specific GolemEvent variant:

| Tool | GolemEvent variant | TUI pane | Rendering |
| --- | --- | --- | --- |
| `stream_pool_events` | `GolemEvent::PoolEventReceived` | Pool Explorer | Appends swap/mint/burn row to live event log |
| `stream_pool_state` | `GolemEvent::PoolStateSnapshot` | Pool Explorer | Updates price ticker, liquidity gauge, fee growth counters |
| `stream_position_alerts` | `GolemEvent::PositionAlert` | Position Monitor | Flashes alert row, updates out-of-range indicator |
| `stream_price_feed` | `GolemEvent::PriceUpdate` | Price Dashboard | Updates price chart data point, recalculates portfolio value |
| `stream_tokenjar` | `GolemEvent::TokenJarUpdate` | Fee Dashboard | Updates jar balance bars, burn countdown |
| `stream_trades` | `GolemEvent::SwapExecuted` | Trade History | Appends trade confirmation row with P&L |

### Sprite animation triggers

Stream data events trigger sprite state transitions on the TUI Golem character:

| Event condition | Sprite trigger | Duration |
| --- | --- | --- |
| `pool:event` with type `swap` | `SpriteTrigger::Thinking` | 500ms |
| `position:alert` with type `out_of_range` | `SpriteTrigger::Failure` | 2000ms |
| `position:alert` with type `back_in_range` | `SpriteTrigger::Success` | 1000ms |
| `trade:executed` with positive P&L | `SpriteTrigger::Success` | 1500ms |
| `trade:executed` with negative P&L | `SpriteTrigger::Failure` | 1500ms |
| `price:update` with >5% move | `SpriteTrigger::Executing` | 1000ms |

---

## Subscription lifecycle

Each subscription spawns a `tokio::task` that either calls `provider.subscribe_logs()` (for event-driven tools) or runs a `tokio::time::interval` loop (for polling tools). The task emits GolemEvent variants to the Event Fabric's broadcast channel via `event_fabric.emit(event)`.

```rust
/// Shared subscription state tracked in session state.
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionEntry {
    pub subscription_id: String,
    pub tool_name: &'static str,
    pub event_variant: &'static str,
    pub params_hash: String,
    pub started_at: u64,
    pub last_event_at: Option<u64>,
    pub event_count: u64,
}

/// Registry of active subscriptions within a session.
pub struct SubscriptionRegistry {
    entries: HashMap<String, SubscriptionEntry>,
    handles: HashMap<String, tokio::task::JoinHandle<()>>,
    event_tx: tokio::sync::broadcast::Sender<GolemEvent>,
}

impl SubscriptionRegistry {
    pub fn new(event_tx: tokio::sync::broadcast::Sender<GolemEvent>) -> Self {
        Self {
            entries: HashMap::new(),
            handles: HashMap::new(),
            event_tx,
        }
    }

    pub fn register(&mut self, entry: SubscriptionEntry, handle: JoinHandle<()>) -> String {
        let id = entry.subscription_id.clone();
        self.entries.insert(id.clone(), entry);
        self.handles.insert(id.clone(), handle);
        id
    }

    pub fn unsubscribe(&mut self, id: &str) -> bool {
        if let Some(handle) = self.handles.remove(id) {
            handle.abort();
            self.entries.remove(id);
            true
        } else {
            false
        }
    }

    pub fn unsubscribe_all(&mut self) -> usize {
        let count = self.handles.len();
        for (_, handle) in self.handles.drain() {
            handle.abort();
        }
        self.entries.clear();
        count
    }
}
```

The `on_unmount` hook calls `registry.unsubscribe_all()` when the session ends. All spawned tasks are aborted, and the broadcast channel naturally cleans up when all senders are dropped.

---

## Tool specifications

### `stream_pool_events`

Subscribes to on-chain events for a specific pool: swaps, mints, burns, fee parameter changes. Uses Alloy's `subscribe_logs` with a filter matching the pool address and relevant event signatures.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `pool_address` | `String` | Yes | Pool contract address |
| `chain_id` | `u64` | Yes | Chain ID |
| `events` | `Option<Vec<String>>` | No | Filter to specific events: `["swap", "mint", "burn"]`. Default: all |

```rust
#[derive(Debug, Deserialize)]
pub struct StreamPoolEventsParams {
    pub pool_address: String,
    pub chain_id: u64,
    #[serde(default)]
    pub events: Option<Vec<String>>,
}

/// Emitted on Pi event bus as `pool:event`.
#[derive(Debug, Serialize)]
pub struct PoolEvent {
    pub pool_address: String,
    pub chain_id: u64,
    pub event_type: String,  // "swap", "mint", "burn"
    pub data: serde_json::Value,
    pub block_number: u64,
    pub tx_hash: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct StreamPoolEventsResult {
    pub subscription_id: String,
}
```

The handler spawns a `tokio::task` that calls `provider.subscribe_logs(filter)` and forwards decoded events to the Pi event bus via `session.emit("pool:event", event)`. The task handle is stored in the `SubscriptionRegistry` for cleanup.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "stream_pool_events",
    description: concat!(
        "Subscribes to real-time pool events (swaps, mints, burns) via Pi session.on('pool:event'). ",
        "Events fire asynchronously. Call once to set up, events arrive continuously. ",
        "Unsubscribe via stream_unsubscribe or automatic session cleanup.",
    ),
    category: Category::Streaming,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Setting up log subscription"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Subscribes to real-time pool events (swaps, mints, burns) via Pi session.on('pool:event'). Call once to set up, events arrive continuously.",
    prompt_guidelines: &[
        "thriving: Subscribe to pools relevant to active strategies. Monitor swap volume and liquidity changes.",
        "cautious: Keep existing subscriptions. Do not add new ones unless replacing a closed position's pool.",
        "declining: Reduce to pools with active positions only. Unsubscribe from watch-only pools.",
        "terminal: Unsubscribe all via stream_unsubscribe.",
    ],
};
```

**Event Fabric events:**

| Event | Payload | Frequency |
| --- | --- | --- |
| `tool:start` | `{ pool_address, chain_id }` | Once |
| `tool:end` | `{ subscription_id }` | Once |
| `pool:event` | `PoolEvent` struct | Continuous |

**Pi hooks:** `tool_call` (validate pool exists on chain). `tool_result` (register subscription in session_state).

**TUI mapping:** Pool Explorer -- appends each event as a row in the live event log. Swap events show token amounts and price impact. Mint/burn events show liquidity delta.

---

### `stream_pool_state`

Subscribes to pool state snapshots at a configurable interval. Emits current tick, sqrtPriceX96, liquidity, and fee growth. Unlike `stream_pool_events` (which uses log subscriptions), this tool polls via `eth_call` on an interval because pool state is not emitted as events -- it must be read from contract storage.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `pool_address` | `String` | Yes | Pool contract address |
| `chain_id` | `u64` | Yes | Chain ID |
| `interval_ms` | `Option<u64>` | No | Polling interval in ms. Default: `15000` |

```rust
#[derive(Debug, Deserialize)]
pub struct StreamPoolStateParams {
    pub pool_address: String,
    pub chain_id: u64,
    #[serde(default = "default_15s")]
    pub interval_ms: u64,
}

/// Emitted on Pi event bus as `pool:state`.
#[derive(Debug, Serialize)]
pub struct PoolStateSnapshot {
    pub pool_address: String,
    pub chain_id: u64,
    pub tick: i32,
    pub sqrt_price_x96: String,
    pub liquidity: String,
    pub fee_growth_0: String,
    pub fee_growth_1: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct StreamPoolStateResult {
    pub subscription_id: String,
}
```

The handler spawns a `tokio::task` with a `tokio::time::interval` that reads `slot0()` and `liquidity()` via Alloy `sol!` bindings, then emits the snapshot to the Pi event bus.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "stream_pool_state",
    description: concat!(
        "Subscribes to periodic pool state snapshots (tick, price, liquidity) via Pi session.on('pool:state'). ",
        "Configurable interval, default 15s. Polls via eth_call, not log subscription.",
    ),
    category: Category::Streaming,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Setting up state poller"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Subscribes to periodic pool state snapshots (tick, price, liquidity) via Pi session.on('pool:state'). Configurable interval, default 15s.",
    prompt_guidelines: &[
        "thriving: Subscribe to pools backing active positions. Use default 15s interval.",
        "cautious: Increase interval to 30s to reduce RPC load. Keep only active-position pools.",
        "declining: Increase interval to 60s. Monitor only pools with positions pending exit.",
        "terminal: Unsubscribe all.",
    ],
};
```

**Event Fabric events:**

| Event | Payload | Frequency |
| --- | --- | --- |
| `tool:start` | `{ pool_address, interval_ms }` | Once |
| `tool:end` | `{ subscription_id }` | Once |
| `pool:state` | `PoolStateSnapshot` struct | Every interval_ms |

**Pi hooks:** `tool_call` (validate pool, validate interval >= 5000ms). `tool_result` (register in session_state).

**TUI mapping:** Pool Explorer -- updates the price ticker in the header, the liquidity depth gauge, and fee growth counters. Each snapshot replaces the previous values (no accumulation).

---

### `stream_position_alerts`

Subscribes to alerts for an LP position: out-of-range, fee accumulation thresholds, back-in-range. Polls position state at a configurable interval and checks conditions.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `position_id` | `String` | Yes | Position token ID or NFT ID |
| `chain_id` | `u64` | Yes | Chain ID |
| `out_of_range_alert` | `Option<bool>` | No | Alert when position goes out of range. Default: `true` |
| `fee_threshold_usd` | `Option<f64>` | No | Alert when uncollected fees exceed this USD value |
| `check_interval_ms` | `Option<u64>` | No | Check interval in ms. Default: `30000` |

```rust
#[derive(Debug, Deserialize)]
pub struct StreamPositionAlertsParams {
    pub position_id: String,
    pub chain_id: u64,
    #[serde(default = "default_true")]
    pub out_of_range_alert: bool,
    pub fee_threshold_usd: Option<f64>,
    #[serde(default = "default_30s")]
    pub check_interval_ms: u64,
}

/// Emitted on Pi event bus as `position:alert`.
#[derive(Debug, Serialize)]
pub struct PositionAlert {
    pub position_id: String,
    pub chain_id: u64,
    pub alert_type: String,  // "out_of_range", "fee_threshold", "back_in_range"
    pub data: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct StreamPositionAlertsResult {
    pub subscription_id: String,
}
```

The handler reads the position's tick range and the pool's current tick each interval. When the current tick exits or re-enters the position's range, it emits the appropriate alert. Fee threshold alerts compare accumulated fees (from position state) against the configured USD value.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "stream_position_alerts",
    description: concat!(
        "Subscribes to LP position alerts (out-of-range, fee threshold, back-in-range) via ",
        "Pi session.on('position:alert'). Configurable thresholds. Events fire when conditions are met.",
    ),
    category: Category::Streaming,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Setting up position monitor"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Subscribes to LP position alerts (out-of-range, fee threshold) via Pi session.on('position:alert'). Events fire when conditions are met.",
    prompt_guidelines: &[
        "thriving: Subscribe to all active positions. Set fee thresholds based on compounding strategy.",
        "cautious: Keep existing alerts. Lower fee thresholds to collect sooner.",
        "declining: Alert on out-of-range only. Ignore fee thresholds (collecting all fees manually).",
        "terminal: Unsubscribe all.",
    ],
};
```

**Event Fabric events:**

| Event | Payload | Frequency |
| --- | --- | --- |
| `tool:start` | `{ position_id }` | Once |
| `tool:end` | `{ subscription_id }` | Once |
| `position:alert` | `PositionAlert` struct | On condition match |

**Pi hooks:** `tool_call` (verify position ownership). `tool_result` (register in session_state).

**TUI mapping:** Position Monitor -- flashes the alert row with the position ID. Out-of-range alerts switch the position's range indicator from green to red. Back-in-range switches it back. Fee threshold alerts show a "Collect fees" action badge.

**Sprite triggers:** `out_of_range` -> `SpriteTrigger::Failure` (2s). `back_in_range` -> `SpriteTrigger::Success` (1s).

---

### `stream_price_feed`

Subscribes to price updates for a token pair. Sources: pool slot0 reads, with optional Trading API fallback for tokens not in any pool.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `token_a` | `String` | Yes | Token address or symbol |
| `token_b` | `String` | Yes | Token address or symbol |
| `chain_id` | `u64` | Yes | Chain ID |
| `interval_ms` | `Option<u64>` | No | Update interval in ms. Default: `15000` |

```rust
#[derive(Debug, Deserialize)]
pub struct StreamPriceFeedParams {
    pub token_a: String,
    pub token_b: String,
    pub chain_id: u64,
    #[serde(default = "default_15s")]
    pub interval_ms: u64,
}

/// Emitted on Pi event bus as `price:update`.
#[derive(Debug, Serialize)]
pub struct PriceUpdate {
    pub token_a: String,
    pub token_b: String,
    pub chain_id: u64,
    pub price: f64,
    pub source: String,  // "pool" or "api"
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct StreamPriceFeedResult {
    pub subscription_id: String,
}
```

The handler first resolves the best pool for the token pair (highest TVL). If no pool exists on-chain for the pair, falls back to `TradingApiClient::get_price()` if available. Emits `price:update` events with the source field indicating which path was used.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "stream_price_feed",
    description: concat!(
        "Subscribes to token pair price updates via Pi session.on('price:update'). ",
        "Sources from pool slot0 reads, with API fallback. Configurable interval, default 15s.",
    ),
    category: Category::Streaming,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Resolving price source", "Setting up price poller"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Subscribes to token pair price updates via Pi session.on('price:update'). Sources from pool slot0 reads. Configurable interval, default 15s.",
    prompt_guidelines: &[
        "thriving: Subscribe to pairs relevant to active strategies. Use default interval.",
        "cautious: Increase interval to 30s. Keep only pairs with active positions.",
        "declining: Increase interval to 60s. Monitor only settlement pairs.",
        "terminal: Unsubscribe all.",
    ],
};
```

**Event Fabric events:**

| Event | Payload | Frequency |
| --- | --- | --- |
| `tool:start` | `{ token_a, token_b, interval_ms }` | Once |
| `tool:end` | `{ subscription_id }` | Once |
| `price:update` | `PriceUpdate` struct | Every interval_ms |

**Pi hooks:** `tool_call` (validate tokens exist on chain). `tool_result` (register in session_state).

**TUI mapping:** Price Dashboard -- updates the price chart with each data point. Recalculates portfolio value if the token is held.

**Sprite triggers:** >5% price move -> `SpriteTrigger::Executing` (1s).

---

### `stream_tokenjar`

Subscribes to TokenJar balance updates and burn events. Monitors fee accumulation across managed positions.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `jar_address` | `String` | Yes | TokenJar contract address |
| `chain_id` | `u64` | Yes | Chain ID |
| `interval_ms` | `Option<u64>` | No | Polling interval in ms. Default: `60000` |

```rust
#[derive(Debug, Deserialize)]
pub struct StreamTokenJarParams {
    pub jar_address: String,
    pub chain_id: u64,
    #[serde(default = "default_60s")]
    pub interval_ms: u64,
}

/// Emitted on Pi event bus as `tokenjar:update`.
#[derive(Debug, Serialize)]
pub struct TokenJarUpdate {
    pub jar_address: String,
    pub chain_id: u64,
    pub balances: Vec<TokenBalance>,
    pub last_burn_at: u64,
    pub total_burned: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct TokenBalance {
    pub token: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct StreamTokenJarResult {
    pub subscription_id: String,
}
```

The handler reads the jar's token balances and burn history via Alloy `sol!` bindings at each interval.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "stream_tokenjar",
    description: concat!(
        "Subscribes to TokenJar balance and burn event updates via Pi session.on('tokenjar:update'). ",
        "Monitors fee accumulation. Default interval: 60s.",
    ),
    category: Category::Streaming,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Setting up jar monitor"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Subscribes to TokenJar balance and burn event updates via Pi session.on('tokenjar:update'). Monitors fee accumulation. Default interval: 60s.",
    prompt_guidelines: &[
        "thriving: Subscribe to TokenJars relevant to fee strategies. Monitor accumulation rate.",
        "cautious: Keep existing subscriptions. Reduce to 120s interval.",
        "declining: Reduce to jars approaching burn threshold only.",
        "terminal: Unsubscribe all.",
    ],
};
```

**Event Fabric events:**

| Event | Payload | Frequency |
| --- | --- | --- |
| `tool:start` | `{ jar_address }` | Once |
| `tool:end` | `{ subscription_id }` | Once |
| `tokenjar:update` | `TokenJarUpdate` struct | Every interval_ms |

**Pi hooks:** `tool_call` (validate jar address). `tool_result` (register in session_state).

**TUI mapping:** Fee Dashboard -- updates jar balance bars and the burn countdown timer.

---

### `stream_trades`

Subscribes to trade execution confirmations for the agent's wallet. Fires when any swap, limit order fill, or UniswapX order settlement completes. Uses Alloy's `subscribe_logs` with Transfer event filters on known token addresses, combined with Swap event filters on Uniswap routers.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `wallet_address` | `String` | Yes | Wallet address to monitor |
| `chain_ids` | `Option<Vec<u64>>` | No | Chain IDs to monitor. Default: all configured |
| `trade_types` | `Option<Vec<String>>` | No | Filter: `["swap", "limit_fill", "uniswapx"]`. Default: all |

```rust
#[derive(Debug, Deserialize)]
pub struct StreamTradesParams {
    pub wallet_address: String,
    #[serde(default)]
    pub chain_ids: Option<Vec<u64>>,
    #[serde(default)]
    pub trade_types: Option<Vec<String>>,
}

/// Emitted on Pi event bus as `trade:executed`.
#[derive(Debug, Serialize)]
pub struct TradeExecuted {
    pub wallet_address: String,
    pub chain_id: u64,
    pub trade_type: String,  // "swap", "limit_fill", "uniswapx"
    pub tx_hash: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: String,
    pub amount_out: String,
    pub gas_used: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct StreamTradesResult {
    pub subscription_id: String,
}
```

For multi-chain monitoring, the handler spawns one `tokio::task` per chain, each with its own log subscription. All tasks share the same subscription ID; unsubscribing cancels all of them.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "stream_trades",
    description: concat!(
        "Subscribes to trade execution confirmations (swaps, limit fills, UniswapX settlements) via ",
        "Pi session.on('trade:executed'). Monitors the agent's wallet across configured chains.",
    ),
    category: Category::Streaming,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Setting up trade monitors"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Subscribes to trade execution confirmations (swaps, limit fills, UniswapX settlements) via Pi session.on('trade:executed'). Monitors the agent's wallet.",
    prompt_guidelines: &[
        "thriving: Subscribe on session start. Monitor all trade types for performance tracking.",
        "cautious: Keep subscription active. Focus on unexpected trades (possible compromise indicator).",
        "declining: Keep for settlement trade confirmation.",
        "terminal: Keep until final trades confirmed, then unsubscribe.",
    ],
};
```

**Event Fabric events:**

| Event | Payload | Frequency |
| --- | --- | --- |
| `tool:start` | `{ wallet_address, chain_ids }` | Once |
| `tool:end` | `{ subscription_id }` | Once |
| `trade:executed` | `TradeExecuted` struct | On trade confirmation |

**Pi hooks:** `tool_call` (verify wallet ownership). `tool_result` (register in session_state).

**TUI mapping:** Trade History -- appends each trade as a row with token amounts, gas cost, and computed P&L (comparing amount_out value against amount_in value).

**Sprite triggers:** Positive P&L -> `SpriteTrigger::Success` (1.5s). Negative P&L -> `SpriteTrigger::Failure` (1.5s).

---

### `stream_unsubscribe`

Explicitly tears down one or more streaming subscriptions. Pi session cleanup handles this automatically on session end, but this tool allows selective unsubscription during a session.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `subscription_ids` | `Option<Vec<String>>` | No | Specific subscription IDs to cancel. Omit to cancel ALL. |

```rust
#[derive(Debug, Deserialize)]
pub struct StreamUnsubscribeParams {
    #[serde(default)]
    pub subscription_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct StreamUnsubscribeResult {
    pub unsubscribed: u32,
    pub remaining: u32,
}
```

When `subscription_ids` is omitted, calls `registry.unsubscribe_all()`. When specific IDs are provided, calls `registry.unsubscribe(id)` for each. Returns the count of actually cancelled subscriptions and the count remaining.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "stream_unsubscribe",
    description: concat!(
        "Cancels streaming subscriptions. Pass specific IDs to cancel selectively, ",
        "or omit to cancel all. Pi session cleanup handles this automatically on session end -- ",
        "use this for mid-session cleanup.",
    ),
    category: Category::Streaming,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Cancelling subscriptions"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Cancels streaming subscriptions. Pass specific IDs to cancel selectively, or omit to cancel all. Use for mid-session cleanup.",
    prompt_guidelines: &[
        "thriving: Unsubscribe from pools/positions no longer relevant.",
        "cautious: Unsubscribe from watch-only subscriptions. Keep position-related ones.",
        "declining: Unsubscribe from everything except position alerts for active exits.",
        "terminal: Unsubscribe all. Called as part of Death Protocol sequence.",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ subscription_ids, cancel_all: bool }` |
| `tool:end` | `{ unsubscribed, remaining }` |

**Pi hooks:** `tool_call` (validate subscription IDs exist in session_state). `tool_result` (clean session_state).

---

## Tool summary

| Tool | Capability | Risk | Budget | GolemEvent variant | TUI pane |
| --- | --- | --- | --- | --- | --- |
| `stream_pool_events` | Read | Layer 1 | Fast | `PoolEventReceived` | Pool Explorer |
| `stream_pool_state` | Read | Layer 1 | Fast | `PoolStateSnapshot` | Pool Explorer |
| `stream_position_alerts` | Read | Layer 1 | Fast | `PositionAlert` | Position Monitor |
| `stream_price_feed` | Read | Layer 1 | Fast | `PriceUpdate` | Price Dashboard |
| `stream_tokenjar` | Read | Layer 1 | Fast | `TokenJarUpdate` | Fee Dashboard |
| `stream_trades` | Read | Layer 1 | Fast | `SwapExecuted` | Trade History |

All 6 tools are Read-only (no state modification). The `stream_unsubscribe` tool is technically a cleanup operation but classified as Read because it only modifies session state, not on-chain state. All events flow through the Event Fabric's `tokio::broadcast` channel with the 51-variant `GolemEvent` enum (see `14-events.md`).

Profiles: `data`, `trader`, `lp`, `golem`, `full`, `dev`. Not included in: `vault`, `intelligence`, `learning`, `identity`.
