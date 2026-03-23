# Stream API [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crate**: `bardo-stream-api`

---

> **Reader orientation:** This document specifies the stream API, the external-facing surface of Bardo's chain intelligence layer (section 14). It exposes the Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) live protocol state and chain events via HTTP snapshots, WebSocket multiplexed streams, and SSE. The terminal connects here for the Sanctum view. The key concept is that this is not a separate service; it runs as a path prefix on the existing Golem HTTP server, sharing authentication. See `prd2/shared/glossary.md` for full term definitions.

The stream API serves the golem's chain intelligence externally. The terminal connects here for the Sanctum view. External integrations connect here for programmatic access to protocol state and live chain events. Authentication uses the same SIWE/JWT tokens as the rest of the golem API -- no separate auth system.

---

## Server Architecture

Built on `axum` with HTTP/2. Added to the existing golem VM HTTP server as a new path prefix `/chain/`. Not a separate port. Shares the existing auth middleware.

```rust
pub fn chain_router(
    protocol_state: Arc<ProtocolStateEngine>,
    chain_scope: Arc<ChainScope>,
    event_fabric: EventFabricHandle,
    triage_store: Arc<TriageStore>,
    auth: Arc<AuthMiddleware>,
) -> Router {
    Router::new()
        // Snapshot endpoints
        .route("/chain/v1/:chain_id/protocols", get(list_protocols))
        .route(
            "/chain/v1/:chain_id/protocol/:address",
            get(get_protocol_state),
        )
        .route(
            "/chain/v1/:chain_id/protocol/:address/history",
            get(get_protocol_history),
        )
        .route(
            "/chain/v1/:chain_id/protocol/:address/schema",
            get(get_protocol_view_schema),
        )
        .route("/chain/v1/:chain_id/scope", get(get_scope_summary))
        .route(
            "/chain/v1/:chain_id/blocks/recent",
            get(get_recent_blocks),
        )
        .route("/chain/v1/:chain_id/events", get(query_triage_events))
        // Streaming
        .route("/chain/v1/stream", get(ws_stream_handler))
        .route("/chain/v1/stream/sse", get(sse_stream_handler))
        // Auth (challenge-response for multi-golem terminal)
        .route("/chain/v1/auth/challenge", get(issue_challenge))
        .route("/chain/v1/auth/verify", post(verify_challenge))
        .layer(auth.layer())
}
```

---

## HTTP Endpoints

### `GET /chain/v1/{chain_id}/protocols`

Returns all tracked protocols, sorted by activity (most recently updated first).

```json
[
  {
    "address": "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640",
    "chain_id": 1,
    "family": "UniswapV3Pool",
    "protocol_id": "uniswap_v3_usdc_eth_500",
    "last_updated_block": 21847001,
    "last_updated_ms": 1741968000000,
    "parent": "0x1F98431c8aD98523631AE4a59f267346ea31F984"
  }
]
```

### `GET /chain/v1/{chain_id}/protocol/{address}`

Returns current hot-layer state for a specific protocol. Reads from `ProtocolStateEngine.hot` (see [03-protocol-state.md](./03-protocol-state.md)) -- zero-latency DashMap read.

```json
{
  "family": "UniswapV3Pool",
  "tick": -197531,
  "sqrt_price_x96": "4295128739...",
  "liquidity": "8234719274...",
  "fee_growth_global_0_x128": "...",
  "fee_growth_global_1_x128": "...",
  "block_number": 21847001,
  "timestamp_ms": 1741968000000
}
```

### `GET /chain/v1/{chain_id}/protocol/{address}/history`

Query parameters: `?from_block=X&to_block=Y&limit=100`

Returns delta history from the warm layer (redb). Useful for chart rendering and historical analysis.

### `GET /chain/v1/{chain_id}/protocol/{address}/schema`

Returns the ProtocolViewSchema (PVS) JSON for generative views. See [07-generative-views.md](./07-generative-views.md) for schema structure.

### `GET /chain/v1/{chain_id}/scope`

Returns current ChainScope summary (see [04-chain-scope.md](./04-chain-scope.md)):

```json
{
  "chain_id": 1,
  "interest_count": 847,
  "active_above_threshold": 234,
  "universe_size_estimate": 42891,
  "bloom_capacity": 65536,
  "top_entries": [
    { "address": "0x...", "reason": "ActivePosition", "score": 1.0 }
  ]
}
```

### `GET /chain/v1/{chain_id}/blocks/recent`

Query parameter: `?n=10`

```json
[
  {
    "block_number": 21847001,
    "filter_hit": true,
    "relevant_tx_count": 3,
    "gas_gwei": 12.4,
    "timestamp_ms": 1741968000000
  }
]
```

### `GET /chain/v1/{chain_id}/events`

Query parameters: `?from_block=X&limit=100&min_score=0.5`

Returns filtered triage events from the store:

```json
[
  {
    "tx_hash": "0xabc...",
    "block_number": 21847001,
    "category": "ProtocolInteraction",
    "protocol_id": "uniswap_v3_usdc_eth_500",
    "score": 0.87,
    "summary": null,
    "timestamp_ms": 1741968000000
  }
]
```

---

## WebSocket -- Multiplexed Live Stream

```
WS /chain/v1/stream
Authorization: Bearer <jwt>
```

### Protocol

Client sends JSON control messages; server pushes JSON event messages.

**Subscribe:**
```json
{ "action": "subscribe", "topics": ["protocol.1.*", "chain.1.triage"] }
```

**Resume after disconnect** (replays from Event Fabric ring buffer):
```json
{ "action": "resume", "from_sequence": 12345 }
```

**Unsubscribe:**
```json
{ "action": "unsubscribe", "topics": ["chain.1.triage"] }
```

**Server push (event):**
```json
{
  "type": "ProtocolStateUpdate",
  "sequence": 12346,
  "timestamp": 1741968000000,
  "payload": {
    "protocol_id": "...",
    "state_delta": {},
    "block_number": 21847001
  }
}
```

**Server push (backpressure notification):**
```json
{
  "type": "MessagesDrop",
  "count": 7,
  "oldest_sequence": 12340,
  "newest_sequence": 12346
}
```

### Topic Namespace

```
protocol.{chain_id}.*              -- all protocol state updates on a chain
protocol.{chain_id}.{address}      -- specific protocol address
chain.{chain_id}.blocks            -- block ingestion events
chain.{chain_id}.triage            -- triage alerts and chain events
chain.{chain_id}.discovery         -- new protocol discovery events
chain.*.scope                      -- ChainScope rebuilds (all chains)
```

Aligned with the dot-notation topic filtering from the existing Event Fabric. See [06-events-signals.md](./06-events-signals.md) for the event variants that flow through these topics.

### Backpressure

Each client has a bounded send buffer (1,024 messages). When full:

- `protocol.*` topics: last-write-wins -- drop intermediate updates, always send latest state
- `chain.*.triage` topics: drop middle messages in burst, preserve first and last
- Client receives `MessagesDrop` on buffer drain

On reconnect, `from_sequence` replays from the Event Fabric's 10,000-entry ring buffer (~30-60 minutes of mainnet activity).

Slow clients are never allowed to block the server. A client that can't keep up gets `MessagesDrop` events and can request historical backfill via the HTTP endpoint.

### WebSocket Handler Implementation

```rust
async fn ws_stream_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<StreamApiState>>,
    auth: AuthContext,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_client(socket, state, auth))
}

async fn handle_ws_client(
    socket: WebSocket,
    state: Arc<StreamApiState>,
    auth: AuthContext,
) {
    let (mut sender, mut receiver) = socket.split();

    // Per-client subscription state.
    let mut subscriptions: HashSet<TopicPattern> = HashSet::new();

    // Per-client bounded send buffer.
    let (tx, mut rx) = mpsc::channel::<WsMessage>(1024);

    // Subscribe to Event Fabric broadcast.
    let mut fabric_rx = state.event_fabric.subscribe();

    // Spawn sender task.
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Main event loop: fan-in from client messages and fabric events.
    loop {
        tokio::select! {
            // Client control message.
            Some(Ok(msg)) = receiver.next() => {
                if let Ok(ctrl) = serde_json::from_str::<ControlMessage>(
                    &msg.to_text().unwrap_or_default()
                ) {
                    match ctrl.action.as_str() {
                        "subscribe" => {
                            for topic in &ctrl.topics {
                                subscriptions.insert(TopicPattern::parse(topic));
                            }
                        }
                        "unsubscribe" => {
                            for topic in &ctrl.topics {
                                subscriptions.remove(&TopicPattern::parse(topic));
                            }
                        }
                        "resume" => {
                            // Replay from ring buffer.
                            let events = state.event_fabric
                                .replay_from(ctrl.from_sequence);
                            for event in events {
                                if matches_any(&event, &subscriptions) {
                                    let _ = tx.try_send(event.to_ws_message());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Fabric event.
            Ok(event) = fabric_rx.recv() => {
                if matches_any(&event, &subscriptions) {
                    if tx.try_send(event.to_ws_message()).is_err() {
                        // Buffer full -- apply backpressure policy.
                        let _ = tx.try_send(WsMessage::Text(
                            serde_json::to_string(&MessagesDrop {
                                count: 1,
                                oldest_sequence: event.sequence,
                                newest_sequence: event.sequence,
                            }).unwrap()
                        ));
                    }
                }
            }

            else => break,
        }
    }

    send_task.abort();
}
```

---

## SSE -- Server-Sent Events

```
GET /chain/v1/stream/sse?topics=protocol.1.*,chain.1.triage
Authorization: Bearer <jwt>
```

For simpler clients or browsers using `EventSource`. The JWT can also be passed as a query parameter (`?token=<jwt>`) since `EventSource` doesn't support custom headers.

```
Content-Type: text/event-stream

id: 12346
event: ProtocolStateUpdate
data: {"protocol_id":"...","state_delta":{...},"block_number":21847001}

id: 12347
event: TriageAlert
data: {"tx_hash":"0xabc...","score":0.91,...}
```

SSE doesn't support bidirectional communication, so it doesn't support `resume`. Reconnection with `Last-Event-ID` header is handled automatically by browsers and SSE clients.

---

## Authentication

### Standard Bearer JWT (TUI and Existing Clients)

The same SIWE/JWT session token from `~/.bardo/auth.json`. Validated by the existing auth middleware -- no new auth code for the common case.

### Multi-Golem Terminal (EIP-712 Challenge-Response)

When the terminal has sessions with multiple golems, each golem serves its own `bardo-stream-api`. The terminal authenticates to each separately:

```
1. GET /chain/v1/auth/challenge
   -> { nonce: "abc123", expires_at: 1741968300000 }

2. Client: Sign EIP-712 CladeChallenge { nonce, target_golem_address }

3. POST /chain/v1/auth/verify
   Body: { signature: "0x...", wallet_address: "0x...", challenge }

4. Server: ecrecover(nonce, sig) == wallet_address
          operatorOf(wallet_address) validates ownership
          Nonce not previously used (anti-replay)

5. Response: { access_token: "jwt...", expires_in: 300 }
```

Challenge TTL: 60 seconds. Access token TTL: 5 minutes (short because it's for a golem-specific chain view, not general golem access).

---

## TUI Integration

### Sanctum Data Flow

```
User navigates to Sanctum -> Protocol Browser
    -> GET /chain/v1/1/protocols
    -> Render protocol list from snapshot

User selects Uniswap V3 USDC/ETH pool
    -> GET /chain/v1/1/protocol/{pool_address}      (initial state)
    -> WS subscribe "protocol.1.{pool_address}"     (live updates)

Each ProtocolStateUpdate arrives:
    -> FlashNumbers widget animates changed values
    -> Liquidity depth chart redraws if liquidity changed
    -> Price ticker updates if tick/sqrtPrice changed
```

### Golem List Sidebar

```
+- GOLEMS -----------------------------------+
| # Aether-7          ========..... 74%      |
|   mainnet #21847234 .  12 gwei             |
|   847 addrs . 23 protocols                 |
+--------------------------------------------+
```

Block/gas: polled at Gamma rate (~250ms). Scope summary: polled at Delta rate (~30s).

### No-Golem Fallback

If the terminal has no active golem session, a local instance of `bardo-witness` + `bardo-protocol-state` + `bardo-stream-api` runs within the terminal process, using the terminal's own RPC endpoint from `~/.bardo/config.toml`. The UX is identical -- the Sanctum view works without a golem.

---

## Rate Limiting

External consumers (non-TUI) are subject to the existing rate limits:

| Tier | Header | Rate limit |
|------|--------|------------|
| Public (no auth) | None | 30 req/min/IP |
| Read key | `X-API-Key: bardo_read_xxx` | 300 req/min/key |
| Authenticated | `Authorization: Bearer <token>` | 1000 req/min/user |

WebSocket connections: 5 concurrent connections per authenticated user. TUI gets a dedicated connection (not subject to this limit).

---

## Dependencies

```toml
[dependencies]
axum = { version = "0.8", features = ["ws", "macros"] }
tokio-tungstenite = "0.24"
tower = { version = "0.5", features = ["timeout", "limit"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
jsonwebtoken = "9"
alloy-signer = "1"
alloy-primitives = "1"
serde_json = "1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
tracing = "0.1"
```

---

## Cross-References

- **Architecture**: [00-architecture.md](./00-architecture.md) -- Where stream API fits in the 5-crate chain intelligence architecture and overall data flow
- **Protocol state**: [03-protocol-state.md](./03-protocol-state.md) -- Hot layer DashMap that backs the snapshot endpoints; warm layer redb that backs history queries
- **Events**: [06-events-signals.md](./06-events-signals.md) -- GolemEvent variants and their wire format that stream through this API
- **Generative views**: [07-generative-views.md](./07-generative-views.md) -- ProtocolViewSchema (PVS) JSON exposed by the `/protocol/{address}/schema` endpoint
- **TUI**: [18-interfaces/03-tui.md](../18-interfaces/03-tui.md) -- Sanctum view and widget catalog that consume stream API data

---

## References

- EIP-712: Typed structured data hashing and signing. Ethereum Improvement Proposals. -- *Defines the typed signing scheme used in the multi-Golem terminal challenge-response authentication.*
- EIP-4361 (SIWE): Sign-In with Ethereum. Ethereum Improvement Proposals. -- *The primary authentication method; SIWE/JWT tokens from the existing Golem API are reused here.*
