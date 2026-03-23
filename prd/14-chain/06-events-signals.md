# Events and Signals [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crate**: `golem-chain` (umbrella for `bardo-witness`, `bardo-triage`, `bardo-protocol-state`, `bardo-chain-scope`)

---

> **Reader orientation:** This document defines the five GolemEvent variants and one CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model) signal emitted by the chain intelligence layer (section 14). These events flow through the Event Fabric broadcast channel to the TUI, the stream API, and any other subscriber. The key concept is event-driven perception: chain intelligence communicates with the rest of the Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) exclusively through typed, serializable events. See `prd2/shared/glossary.md` for full term definitions.

The chain intelligence system emits five `GolemEvent` variants through the Event Fabric and maintains one `CorticalState` signal. All events flow through the same broadcast channel as other golem events, consumable by the TUI, `bardo-stream-api` (see [08-stream-api.md](./08-stream-api.md)), and any other subscriber.

---

## GolemEvent Variants

Five variants belong to the `chain` category.

```rust
// -- Chain Intelligence --

/// Protocol state mutation detected and recorded.
/// High-frequency during active DeFi activity (~1-10 per block on watched protocols).
#[serde(rename = "chain.protocol_state_update")]
ChainTxObserved {
    /// Base fields: timestamp, golem_id, sequence, tick.

    /// Protocol identifier (e.g., "uniswap_v3_usdc_eth_500").
    protocol_id: String,

    /// Chain where the state change occurred.
    chain_id: u64,

    /// Only the fields that changed (delta, not full state).
    /// Keyed by field name, values are the new values.
    state_delta: serde_json::Value,

    /// Block number where the state was read.
    block_number: u64,
},

/// Block processing complete with stats.
/// Emitted after triage finishes processing a relevant block.
#[serde(rename = "chain.block_processed")]
ChainBlockProcessed {
    chain_id: u64,

    /// Transaction hash.
    tx_hash: String,

    /// Block containing the transaction.
    block_number: u64,

    /// Transaction category (TxCategory serialized).
    category: String,

    /// Protocol involved, if identified.
    protocol_id: Option<String>,

    /// Curiosity score assigned by the triage pipeline.
    curiosity_score: f32,

    /// Human-readable summary. Populated async by LLM at Theta tick.
    /// Initially None; updated via event amendment when LLM analysis completes.
    summary: Option<String>,
},

/// MIDAS/curiosity threshold exceeded.
/// Emitted when triage scores a transaction above the escalation threshold.
/// Triggers LLM analysis at the next Theta tick.
#[serde(rename = "chain.anomaly_detected")]
ChainAnomalyDetected {
    chain_id: u64,
    tx_hash: String,
    block_number: u64,

    /// Transaction category.
    category: String,

    /// Curiosity score (always > 0.8 for this event type).
    score: f32,

    /// Human-readable reason for escalation.
    /// Examples: "active position counterparty + large value",
    ///           "MIDAS anomaly score 3.2x baseline",
    ///           "first interaction with newly discovered protocol".
    reason: String,
},

/// Tracked protocol state mutation.
/// Emitted when autonomous discovery identifies a new protocol
/// from factory events, bytecode matching, or ABI resolution.
#[serde(rename = "chain.protocol_state_changed")]
ChainProtocolStateChanged {
    chain_id: u64,

    /// Contract address of the discovered protocol.
    address: String,

    /// Classified protocol family (e.g., "UniswapV3Pool").
    protocol_family: String,

    /// How it was discovered.
    /// One of: "factory_event", "bytecode_match", "abi_resolution".
    discovered_via: String,

    /// Factory that deployed this contract (if known).
    parent_address: Option<String>,
},

/// Attention model updated.
/// Emitted when the witness detects a gap in block coverage
/// after a WebSocket reconnection.
#[serde(rename = "chain.scope_adjusted")]
ChainScopeAdjusted {
    chain_id: u64,

    /// First missed block number.
    from_block: u64,

    /// Last missed block number.
    to_block: u64,

    /// Total number of missed blocks.
    gap_size: u64,
},
```

---

## Subscription Categories

The five variants belong to the `chain` subscription category. Subscribers can filter by topic:

| Topic pattern | Matches |
|---------------|---------|
| `chain.*` | All chain intelligence events |
| `chain.{chain_id}.*` | All events for a specific chain |
| `chain.{chain_id}.protocol_state_update` | Protocol state changes only |
| `chain.{chain_id}.triage` | Triage alerts and chain events |
| `chain.{chain_id}.discovery` | New protocol discovery events |
| `chain.*.scope` | ChainScope adjustments (all chains) |

The `dashboard` composite subscription includes `chain`: `dashboard` = heartbeat + vitality + daimon + tool + permit + cortical + chain.

---

## Wire Format Examples

### ChainTxObserved

```json
{
  "type": "chain.protocol_state_update",
  "timestamp": 1741968015000,
  "golem_id": "g-7f3a",
  "sequence": 4220,
  "tick": 3848,
  "protocol_id": "uniswap_v3_usdc_eth_500",
  "chain_id": 1,
  "state_delta": {
    "tick": -197531,
    "sqrt_price_x96": "4295128739572847539847209847239847",
    "liquidity": "8234719274916234"
  },
  "block_number": 21847001
}
```

### ChainAnomalyDetected

```json
{
  "type": "chain.anomaly_detected",
  "timestamp": 1741968030000,
  "golem_id": "g-7f3a",
  "sequence": 4221,
  "tick": 3848,
  "chain_id": 1,
  "tx_hash": "0xdef456...",
  "block_number": 21847002,
  "category": "ProtocolInteraction",
  "score": 0.92,
  "reason": "active position counterparty + large value"
}
```

### ChainProtocolStateChanged

```json
{
  "type": "chain.protocol_state_changed",
  "timestamp": 1741968045000,
  "golem_id": "g-7f3a",
  "sequence": 4222,
  "tick": 3849,
  "chain_id": 1,
  "address": "0x1234...",
  "protocol_family": "UniswapV3Pool",
  "discovered_via": "factory_event",
  "parent_address": "0x1F98431c8aD98523631AE4a59f267346ea31F984"
}
```

### ChainScopeAdjusted

```json
{
  "type": "chain.scope_adjusted",
  "timestamp": 1741968060000,
  "golem_id": "g-7f3a",
  "sequence": 4223,
  "tick": 3849,
  "chain_id": 1,
  "from_block": 21846500,
  "to_block": 21847000,
  "gap_size": 500
}
```

---

## CorticalState Signal: `chain_blocks_behind`

One new signal in the `ENVIRONMENT` group:

```rust
// In CorticalState:
// == ENVIRONMENT -- written by domain probes ==
pub(crate) regime: AtomicU8,                // existing
pub(crate) gas_gwei: AtomicU32,             // existing
pub(crate) chain_blocks_behind: AtomicU32,  // NEW: staleness indicator
```

### Properties

| Property | Value |
|----------|-------|
| Type | `AtomicU32` (stored as raw `u32`) |
| Semantics | Number of blocks the triage pipeline is behind the chain head |
| Range | `[0, infinity)` -- 0 means current, >0 means lagging |
| Writer | `bardo-witness`, updated each Gamma tick |
| Readers | Theta-tick gating (trading suppression), TUI staleness indicator |

### How It's Computed

```rust
// In gamma_chain_perception():
let chain_head = witness.latest_chain_block_number();
let processed = triage.latest_processed_block_number();
let behind = chain_head.saturating_sub(processed);
cortical_state.chain_blocks_behind.store(behind as u32, Ordering::Relaxed);
```

### How It's Used

```rust
// In Theta-tick gating:
let behind = cortical_state.chain_blocks_behind.load(Ordering::Relaxed);
if behind > STALENESS_THRESHOLD {
    // Suppress trading actions that depend on fresh chain state.
    // Still allow reads, position monitoring, etc.
    tracing::warn!(chain_id, behind, "chain lag detected -- suppressing write actions");
    return SuppressReason::ChainLag;
}
```

Suggested `STALENESS_THRESHOLD`: 3 blocks on mainnet (45s), 10 blocks on L2s with faster block times.

### TUI Representation

The TUI shows a staleness indicator next to the chain status in the golem list sidebar:

- `blocks_behind == 0`: green dot
- `0 < blocks_behind <= threshold`: amber dot
- `blocks_behind > threshold`: red dot + "syncing" label

### Size Impact

The CorticalState struct gains one `AtomicU32` (4 bytes). Previous size: 192 bytes (padded from 178 bytes). With the new field:
- Raw payload: 178 + 4 = 182 bytes
- With `#[repr(C, align(64))]` padding: still 192 bytes

No size change to the padded struct. The new field fits within the existing padding.

---

## Emitter Matrix Update

| Category | Emitter crate | Variant count | Consumers |
|----------|---------------|---------------|-----------|
| Chain Intelligence | `golem-chain` | 5 | TUI Sanctum, bardo-stream-api, Prometheus |

See [shared/event-catalog.md](../shared/event-catalog.md) for the full cross-system event catalog.

---

## Cross-References

- **Architecture**: [00-architecture.md](./00-architecture.md) -- Overall event flow through Event Fabric, five-crate overview, data flow diagram
- **Heartbeat integration**: [05-heartbeat-integration.md](./05-heartbeat-integration.md) -- When each event type is emitted relative to Gamma/Theta/Delta ticks
- **Stream API**: [08-stream-api.md](./08-stream-api.md) -- How external clients subscribe to these events via WebSocket/SSE, topic filtering
- **Heartbeat**: [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- CorticalState layout and all 32 signals
- **Event catalog**: [shared/event-catalog.md](../shared/event-catalog.md) -- Full GolemEvent variant list across all subsystems (not just chain intelligence)

---

## References

- Wood, G. (2014). Ethereum Yellow Paper. Section 4.3 (logsBloom definition). -- *Defines the 2048-bit block-level Bloom filter; referenced for context on the upstream witness layer's pre-screening.*
