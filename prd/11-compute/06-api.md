# 06 -- API reference [SPEC]

**REST endpoints, database schema, request/response schemas, GolemEvent types, and error codes**

> **Reader orientation:** This document is the API reference for Bardo Compute, the managed VM hosting service for Golems (mortal autonomous DeFi agents). It belongs to the **Compute** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: the API is a REST service backed by Turso (libSQL) for persistence, accepting x402 (EIP-3009 USDC micropayment) headers for provisioning and extension, and emitting typed `GolemEvent` variants for all state transitions. Terms like Golem, Grimoire, Styx, and PolicyCage are defined inline on first use; a full glossary lives in `00-overview.md § Terminology`.

---

## Database schema (Turso / libSQL)

### `machines` table

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE machines (
  id            TEXT PRIMARY KEY,           -- UUID v4
  user_id       TEXT NOT NULL,              -- Privy user ID
  machine_id    TEXT,                       -- Fly machine ID (null during provisioning)
  machine_name  TEXT NOT NULL UNIQUE,       -- golem-{nanoid(12)}
  private_ip    TEXT,                       -- Fly 6PN IPv6 (null during provisioning)
  status        TEXT NOT NULL DEFAULT 'provisioning',
                                            -- provisioning|booting|ready|draining|crashed|destroyed
  vm_size       TEXT NOT NULL DEFAULT 'small',
                                            -- micro|small|medium|large
  strategy_type TEXT,                       -- yield-optimizer, dca, keeper, etc.
  tool_profile  TEXT NOT NULL DEFAULT 'full',
  custody_mode  TEXT NOT NULL DEFAULT 'delegation',
                                            -- delegation|embedded|local_key
  ttl_seconds   INTEGER NOT NULL,           -- Total TTL granted (cumulative)
  expires_at    INTEGER NOT NULL,           -- Epoch seconds (authoritative TTL)
  created_at    INTEGER NOT NULL DEFAULT (unixepoch()),
  destroyed_at  INTEGER,
  destroy_reason TEXT,                      -- ttl_expired|user_destroyed|admin_destroyed|
                                            --   provision_failed|boot_timeout|thanatopsis|orphaned
  region        TEXT NOT NULL,
  wallet_address TEXT,                      -- Custody-mode wallet address (Base)
  erc8004_id    TEXT,                       -- ERC-8004 on-chain identity (Base)
  styx_connected INTEGER NOT NULL DEFAULT 0, -- 0 or 1
  cache_version INTEGER NOT NULL DEFAULT 0  -- Incremented on status change
);

CREATE INDEX idx_machines_user_id ON machines(user_id);
CREATE INDEX idx_machines_status ON machines(status);
CREATE INDEX idx_machines_expires_at ON machines(expires_at);
CREATE UNIQUE INDEX idx_machines_machine_name ON machines(machine_name);
```

### `billing_events` table

```sql
CREATE TABLE billing_events (
  id                 TEXT PRIMARY KEY,
  user_id            TEXT NOT NULL,
  machine_id         TEXT NOT NULL REFERENCES machines(id),
  type               TEXT NOT NULL,             -- provision|extension|refund|failed_provision|self_extension
  amount_micro_usdc  INTEGER NOT NULL,
  tx_hash            TEXT,
  from_address       TEXT NOT NULL,
  to_address         TEXT NOT NULL,
  payer_type         TEXT NOT NULL,             -- owner|self|external_user
  nonce              TEXT NOT NULL UNIQUE,
  ttl_seconds_granted INTEGER NOT NULL,
  vm_size            TEXT NOT NULL,
  created_at         INTEGER NOT NULL DEFAULT (unixepoch()),
  metadata           TEXT
);

CREATE INDEX idx_billing_user_id ON billing_events(user_id);
CREATE INDEX idx_billing_machine_id ON billing_events(machine_id);
CREATE INDEX idx_billing_type ON billing_events(type);
CREATE INDEX idx_billing_created_at ON billing_events(created_at);
```

### `machine_events` table

```sql
CREATE TABLE machine_events (
  id        TEXT PRIMARY KEY,
  machine_id TEXT NOT NULL REFERENCES machines(id),
  user_id   TEXT NOT NULL,
  event     TEXT NOT NULL,
  metadata  TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX idx_events_machine_id ON machine_events(machine_id);
CREATE INDEX idx_events_event ON machine_events(event);
CREATE INDEX idx_events_created_at ON machine_events(created_at);
```

### Supporting tables

```sql
-- System locks (leader election)
CREATE TABLE system_locks (
  name       TEXT PRIMARY KEY,
  holder     TEXT,
  acquired_at INTEGER
);

-- Grimoire snapshots
CREATE TABLE grimoire_snapshots (
  id          TEXT PRIMARY KEY,
  user_id     TEXT NOT NULL,
  machine_id  TEXT NOT NULL REFERENCES machines(id),
  machine_name TEXT NOT NULL,
  size_bytes  INTEGER NOT NULL,
  r2_key      TEXT NOT NULL,
  created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
  type        TEXT NOT NULL DEFAULT 'periodic'
);

-- User SSH keys
CREATE TABLE user_keys (
  id          TEXT PRIMARY KEY,
  user_id     TEXT NOT NULL,
  public_key  TEXT NOT NULL,
  fingerprint TEXT NOT NULL UNIQUE,
  label       TEXT,
  added_at    INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Strategy compilation quotas
CREATE TABLE user_quotas (
  user_id                TEXT PRIMARY KEY,
  strategy_compilations  INTEGER NOT NULL DEFAULT 0,
  free_compilations_used INTEGER NOT NULL DEFAULT 0,
  updated_at             INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Admin users
CREATE TABLE admins (
  user_id TEXT PRIMARY KEY,
  role    TEXT NOT NULL DEFAULT 'operator',
  added_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Daily revenue aggregates
CREATE TABLE daily_revenue (
  date              TEXT PRIMARY KEY,
  total_micro_usdc  INTEGER NOT NULL DEFAULT 0,
  provision_count   INTEGER NOT NULL DEFAULT 0,
  extension_count   INTEGER NOT NULL DEFAULT 0,
  self_extension_count INTEGER NOT NULL DEFAULT 0,
  refund_count      INTEGER NOT NULL DEFAULT 0,
  refund_micro_usdc INTEGER NOT NULL DEFAULT 0,
  avg_ttl_seconds   INTEGER NOT NULL DEFAULT 0,
  active_machines_eod INTEGER NOT NULL DEFAULT 0,
  unique_users      INTEGER NOT NULL DEFAULT 0
);

-- Archived destroyed machines (90+ days)
CREATE TABLE machines_archive (
  -- Same schema as machines + archived_at
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  machine_id TEXT,
  machine_name TEXT NOT NULL,
  private_ip TEXT,
  status TEXT NOT NULL,
  vm_size TEXT NOT NULL,
  strategy_type TEXT,
  tool_profile TEXT NOT NULL,
  custody_mode TEXT NOT NULL,
  ttl_seconds INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  destroyed_at INTEGER,
  destroy_reason TEXT,
  region TEXT NOT NULL,
  wallet_address TEXT,
  erc8004_id TEXT,
  archived_at INTEGER NOT NULL DEFAULT (unixepoch())
);
```

---

## REST API reference

### Public endpoints (no auth)

| Method | Path | Description | Rate limit |
| --- | --- | --- | --- |
| `GET` | `/health` | API health check | 60/min/IP |
| `GET` | `/v1/pricing` | Current pricing tiers | 30/min/IP |
| `GET` | `/v1/golems` | Agent discovery directory | 30/min/IP |
| `GET` | `/v1/machines/:name/status` | Public machine status | 30/min/IP |
| `POST` | `/v1/machines/:name/extend` | Permissionless extension | 10/min/IP, 30/hr/machine |

#### `GET /health`

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "checks": { "db": "ok", "fly_api": "ok" },
  "uptime": 86400
}
```

#### `GET /v1/pricing`

```json
{
  "currency": "USDC",
  "chain": "base",
  "chain_id": 8453,
  "tiers": [
    { "vm_size": "micro", "price_per_hour_micro_usdc": 25000, "min_purchase_hours": 1 },
    { "vm_size": "small", "price_per_hour_micro_usdc": 50000, "min_purchase_hours": 1 },
    { "vm_size": "medium", "price_per_hour_micro_usdc": 100000, "min_purchase_hours": 1 },
    { "vm_size": "large", "price_per_hour_micro_usdc": 200000, "min_purchase_hours": 1 }
  ],
  "max_extension_hours": 720,
  "min_extension_hours": 1,
  "operator_address": "0x..."
}
```

#### `GET /v1/golems`

Query params: `capabilities`, `asset`, `status`, `region`, `limit` (default 50, max 200), `offset`.

#### `GET /v1/machines/:name/status`

All time values include `server_time` for clock skew compensation:

```json
{
  "name": "golem-V1StGXR8_Z5j",
  "status": "ready",
  "vm_size": "small",
  "region": "ams",
  "custody_mode": "delegation",
  "vitality": 0.72,
  "behavioral_phase": "stable",
  "styx_connected": true,
  "ttl": {
    "expires_at": 1710500400,
    "remaining_seconds": 36000,
    "server_time": 1710464400,
    "pricing": { "price_per_hour_micro_usdc": 50000 }
  },
  "performance": { "nav": 1050.25, "uptime_seconds": 604800 }
}
```

### User endpoints (Privy JWT required)

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/v1/machines` | Provision new Golem (returns 202) |
| `GET` | `/v1/machines/mine` | List user's machines (paginated) |
| `GET` | `/v1/machines/mine/events` | SSE dashboard events |
| `GET` | `/v1/machines/:name` | Full machine details |
| `GET` | `/v1/machines/:name/provision-status` | SSE provisioning progress |
| `DELETE` | `/v1/machines/:name` | Destroy machine (graceful Thanatopsis) |
| `POST` | `/v1/keys` | Add SSH public key |
| `GET` | `/v1/keys` | List user's SSH keys |
| `DELETE` | `/v1/keys/:fingerprint` | Remove SSH key |
| `GET` | `/v1/billing/mine` | Billing history |
| `POST` | `/v1/ssh/ticket` | Get SSH ticket |

#### `POST /v1/machines` (202 Accepted)

```json
{
  "id": "uuid",
  "machine_name": "golem-V1StGXR8_Z5j",
  "status": "provisioning",
  "url": "https://golem-V1StGXR8_Z5j.bardo.money",
  "vm_size": "small",
  "region": "ams",
  "custody_mode": "delegation",
  "ttl_seconds": 36000,
  "expires_at": null,
  "tx_hash": null,
  "wallet_address": null,
  "server_time": 1710464400
}
```

### Internal endpoints (Fly OIDC token)

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/internal/machines/self/ttl` | Authoritative TTL (for cron) |
| `POST` | `/internal/machines/self/crash` | Report crash |
| `POST` | `/internal/grimoire/sync` | Upload snapshot |
| `GET` | `/internal/grimoire/:user_id/snapshot` | Download snapshot |

### Admin endpoints (Privy JWT + admin role)

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/admin/stats` | System metrics |
| `GET` | `/admin/machines` | List all machines |
| `GET` | `/admin/machines/:id` | Full details (any machine) |
| `DELETE` | `/admin/machines/:id` | Force destroy |
| `POST` | `/admin/machines/:id/expire` | Force TTL expiry |
| `POST` | `/admin/machines/:id/sweep` | Manual wallet sweep (Embedded mode only) |
| `GET` | `/admin/billing/summary` | Revenue summary |
| `POST` | `/admin/refund` | Manual refund |
| `GET` | `/admin/health/detailed` | Extended health |

Admin endpoints use `:id` (UUID) not `:name`.

### WebSocket SSH endpoint

```
wss://ssh.bardo.money/ws?machine=golem-V1StGXR8_Z5j&ticket=<ticket>
```

---

## Machine event types

Events are stored in the `machine_events` table and pushed via SSE to connected dashboard clients. These are **control plane events** -- distinct from the Golem's `GolemEvent` types emitted by the Event Fabric.

| Event | Trigger | Metadata |
| --- | --- | --- |
| `PaymentVerified` | x402 validated | `{ trace_id, amount_micro_usdc, nonce }` |
| `Provisioning` | Fly machine started | `{ trace_id, region, vm_size, custody_mode }` |
| `Booting` | Health check polling | `{ trace_id }` |
| `Ready` | Health check passed | `{ trace_id, provision_time_ms, styx_connected }` |
| `Extended` | TTL extended | `{ payer_type, amount_micro_usdc, ttl_added, new_expires_at }` |
| `SshConnected` | SSH session via ticket | `{ source_ip, ticket_id }` |
| `StrategyUpdated` | Hot-swap | `{ strategy_type }` |
| `GrimoireSynced` | Periodic sync | `{ snapshot_size_bytes }` |
| `Draining` | Shutdown initiated | `{ reason, behavioral_phase }` |
| `ThanatopsisStarted` | Death protocol begins | `{ vitality, death_cause }` |
| `Destroyed` | Fully destroyed | `{ destroy_reason, final_ttl_seconds, death_testament_uploaded }` |
| `Crashed` | Supervisor max restarts | `{ exit_code, restart_count }` |
| `WalletSwept` | Control plane sweep | `{ usdc_micro_amount, eth_wei, tx_hash }` |
| `DelegationExpired` | Delegation timeout | `{ delegation_hash, expiry_cause }` |
| `ProvisionFailed` | Provisioning failed | `{ trace_id, error, step }` |
| `AdminForceDestroy` | Admin action | `{ admin_id, reason }` |
| `AdminForceExpire` | Admin action | `{ admin_id, reason }` |
| `AdminRefund` | Admin refund | `{ admin_id, amount_micro_usdc, tx_hash }` |
| `AdminSweep` | Admin sweep | `{ admin_id, wallet_address, amount_micro_usdc }` |
| `Warning30m` | 30 min warning | -- |
| `Warning5m` | 5 min warning | -- |

---

## GolemEvent types (from Event Fabric)

The Golem binary emits 50+ typed `GolemEvent` variants via the Event Fabric. These flow over the Styx WebSocket to surfaces (TUI, web dashboard). They use Rust `CamelCase` naming with `snake_case` serde serialization:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GolemEvent {
    // Heartbeat
    MarketObservation { timestamp: u64, tick: u64, regime: String, anomalies: Vec<String>, .. },
    HeartbeatTick { timestamp: u64, tick: u64, tier: String, pe: f64, threshold: f64, .. },
    HeartbeatComplete { timestamp: u64, tick: u64, duration_ms: u64, actions_taken: u32, .. },
    TierSelected { timestamp: u64, tick: u64, tier: String, vitality: f64, arousal: f64, .. },

    // Tools
    ToolExecutionStart { timestamp: u64, tool_name: String, risk_tier: String, .. },
    ToolExecutionUpdate { timestamp: u64, tool_name: String, progress: Option<f64>, .. },
    ToolExecutionEnd { timestamp: u64, tool_name: String, result: String, tx_hash: Option<String>, .. },

    // Permits
    PermitCreated { timestamp: u64, permit_id: String, action_kind: String, .. },
    PermitCommitted { timestamp: u64, permit_id: String, tx_hash: Option<String>, .. },
    PermitBlocked { timestamp: u64, permit_id: String, block_reason: String, .. },

    // Inference
    InferenceStart { timestamp: u64, model: String, tier: String, .. },
    InferenceEnd { timestamp: u64, model: String, cost_usd: f64, latency_ms: u64, .. },

    // Dreams
    DreamStart { timestamp: u64, phase: String, .. },
    DreamReplay { timestamp: u64, episode_id: String, .. },
    DreamInsight { timestamp: u64, insight_text: String, confidence: f64, .. },
    DreamEnd { timestamp: u64, insights_generated: u32, entries_pruned: u32, .. },

    // Mortality
    VitalityUpdate { timestamp: u64, economic: f64, epistemic: f64, stochastic: f64, composite: f64, .. },
    PhaseTransition { timestamp: u64, from_phase: String, to_phase: String, vitality: f64, .. },
    ThanatopsisStarted { timestamp: u64, death_cause: String, .. },
    DeathReflection { timestamp: u64, reflection_text: String, .. },

    // Affect
    EmotionUpdate { timestamp: u64, pleasure: f64, arousal: f64, dominance: f64, label: String, .. },
    SomaticMarker { timestamp: u64, situation: String, valence: f64, .. },

    // Memory
    GrimoireWrite { timestamp: u64, entry_type: String, confidence: f64, .. },
    CuratorCycle { timestamp: u64, entries_pruned: u32, entries_compressed: u32, .. },

    // Custody
    SessionKeyRotated { timestamp: u64, old_address: String, new_address: String, .. },
    DelegationWarning { timestamp: u64, remaining_budget_pct: f64, .. },

    // Styx
    StyxConnected { timestamp: u64, latency_ms: u64, .. },
    StyxDisconnected { timestamp: u64, reason: String, .. },
    PheromoneDeposited { timestamp: u64, pheromone_class: String, domain: String, .. },
    BloodstainReceived { timestamp: u64, source_golem: String, warning_count: u32, .. },

    // Trading
    SwapExecuted { timestamp: u64, pair: String, amount_in: f64, amount_out: f64, slippage_bps: u16, gas_usd: f64, .. },
    PositionOpened { timestamp: u64, protocol: String, pool: String, range: String, .. },
    PositionClosed { timestamp: u64, protocol: String, pnl_usd: f64, .. },

    // Economics
    SustainabilityUpdate { timestamp: u64, ratio: f64, daily_revenue: f64, daily_cost: f64, .. },
    TtlExtended { timestamp: u64, added_seconds: u64, new_expires_at: u64, payer_type: String, .. },
}
```

Each variant carries `timestamp` (Unix ms), `golem_id`, `sequence` (monotonic), and `tick` (heartbeat number). See [prd2/01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) for the full GolemEvent catalog with field definitions.

---

## GolemState (client-side representation)

```rust
/// Client-side state for rendering Golem status in TUI or web dashboard.
/// Assembled from health endpoint + GolemEvent stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemState {
    // Identity
    pub machine_name: String,
    pub user_id: String,
    pub vm_size: VmTier,
    pub region: String,
    pub custody_mode: String,

    // TTL (with clock skew compensation)
    pub expires_at: u64,
    pub clock_offset: i64,       // server_time - local_time (seconds)

    // Strategy
    pub strategy_type: String,

    // Performance
    pub nav: f64,
    pub pnl_percent: f64,
    pub sharpe: f64,
    pub max_drawdown: f64,
    pub uptime_seconds: u64,

    // Vitality
    pub vitality: f64,
    pub behavioral_phase: String,
    pub sustainability_ratio: Option<f64>,

    // Status
    pub status: String,          // "booting", "ready", "draining", "crashed"
    pub styx_connected: bool,
    pub tool_count: u32,
    pub heartbeat_count: u64,
}
```

---

## Error codes

| Code | HTTP status | Meaning |
| --- | --- | --- |
| `PAYMENT_INVALID` | 402 | x402 header missing or malformed |
| `PAYMENT_INSUFFICIENT` | 402 | Below minimum for VM size |
| `PAYMENT_BALANCE_LOW` | 402 | On-chain USDC insufficient |
| `PAYMENT_NONCE_USED` | 409 | EIP-3009 nonce consumed |
| `PAYMENT_EXPIRED` | 402 | `validBefore` has passed |
| `MACHINE_NOT_FOUND` | 404 | Name or ID not found |
| `MACHINE_NOT_READY` | 409 | Not in `ready` state |
| `MACHINE_LIMIT_REACHED` | 403 | At 5 active machines |
| `EXTENSION_TOO_LARGE` | 400 | >720h single extension |
| `EXTENSION_TOO_SMALL` | 400 | Below 1-hour minimum |
| `EXTENSION_CONFLICT` | 409 | CAS conflict (retry) |
| `VALID_BEFORE_TOO_TIGHT` | 400 | <300s window |
| `REGION_FULL` | 503 | Per-region cap reached |
| `GLOBAL_CAP_REACHED` | 503 | Global cap reached |
| `UNAUTHORIZED` | 401 | Missing/invalid JWT |
| `FORBIDDEN` | 403 | Valid JWT but not owner/admin |
| `RATE_LIMITED` | 429 | Rate limit exceeded |
| `PROVISION_FAILED` | 500 | Fly or health check failed |
| `GOLEM_NOT_FOUND` | 404 | Proxy: no matching machine |
| `SSH_TICKET_INVALID` | 401 | Ticket expired or consumed |
| `ERC8004_REGISTRATION_FAILED` | 500 | On-chain identity registration failed |
| `STYX_CONNECTION_FAILED` | 503 | Styx WebSocket handshake failed (non-fatal) |

---

## Environment variables

### bardo-control

| Variable | Description |
| --- | --- |
| `DATABASE_URL` | Turso connection string |
| `DATABASE_AUTH_TOKEN` | Turso auth token |
| `FLY_API_TOKEN` | Fly.io API token |
| `FLY_ORG_SLUG` | Fly org for OIDC issuer URL |
| `BARDO_OPERATOR_ADDRESS` | USDC recipient (Base) |
| `PRIVY_APP_ID` | Privy app ID |
| `PRIVY_APP_SECRET` | Privy API secret |
| `BASE_RPC_URL` | Base chain RPC |
| `STEP_CA_URL` | SSH CA URL |
| `STEP_CA_FINGERPRINT` | CA root certificate fingerprint |
| `BARDO_ADMIN_SECRET` | Break-glass admin auth |
| `REDIS_URL` | Optional -- rate limiting only |

### bardo-machines (Golem VMs)

All config injected via `golem.toml` at machine creation. See [02-provisioning.md](02-provisioning.md).

---

## Retention policy

- **Active machines**: No retention limit
- **Destroyed machines**: Archived after 90 days to `machines_archive`
- **Billing events**: Kept permanently (financial records)
- **Machine events**: Kept with associated machine
- **Grimoire snapshots**: Latest 5 per machine (R2). Full history in Styx Archive layer.
- **Daily revenue**: Kept permanently
