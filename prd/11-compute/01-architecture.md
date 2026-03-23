# 01 -- System architecture [SPEC]

**2-App Fly.io topology, orchestration, and network isolation**

> **Reader orientation:** This document specifies the system architecture for Bardo Compute, the managed VM hosting service for Golems (mortal autonomous DeFi agents). It belongs to the **Compute** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: the system is two Fly.io apps, one control plane monolith (`bardo-control`) handling API, proxy, and operations, and one app (`bardo-machines`) hosting user Golem VMs on a private network. All inter-app communication uses Fly's 6PN private networking. Terms like Golem, Grimoire, Styx, and x402 are defined inline on first use; a full glossary lives in `00-overview.md § Terminology`.

---

## 2-app Fly topology

Two Fly apps compose the system. One monolith control plane handles all API, proxy, and operational concerns. One app hosts user Golem VMs on a private network. Each VM runs a single Golem-RS binary -- a statically compiled Rust executable with the full cognitive architecture.

```
+-----------------------------------------------------------------+
|                        Public Internet                          |
|   api.bardo.money  *.bardo.money  ssh.bardo.money  bardo.money  |
+------+--------------+--------------+--------------+-------------+
       |              |              |              |
       v              v              v              v
+------------------------------------------+   +----------+
|              bardo-control               |   | bardo-web|
|  +---------+  +----------+  +--------+  |   | (Vercel) |
|  | API     |  | Proxy    |  | SSH WS |  |   | Landing  |
|  | routes  |  | middleware|  | handler|  |   | page     |
|  | billing |  |          |  |        |  |   +----------+
|  | admin   |  |          |  |        |  |
|  | TTL     |  |          |  |        |  |
|  | worker  |  |          |  |        |  |
|  +---------+  +----------+  +--------+  |
+------+--------------+-------------------+
       |              | Fly 6PN (private)
       |         +----+------------------+
       |         |    |                  |
       v         v    v                  v
+----------------------------------------------------------------+
|                      bardo-machines                            |
|  +----------+  +----------+  +----------+  +----------+       |
|  | golem-a  |  | golem-b  |  | golem-c  |  | golem-d  |  ...  |
|  | Rust bin |  | Rust bin |  | Rust bin |  | Rust bin |       |
|  | :3000 pub|  | :3000 pub|  | :3000 pub|  | :3000 pub|       |
|  | :3001 own|  | :3001 own|  | :3001 own|  | :3001 own|       |
|  | :3002 int|  | :3002 int|  | :3002 int|  | :3002 int|       |
|  | WSS->Styx|  | WSS->Styx|  | WSS->Styx|  | WSS->Styx|       |
|  +----------+  +----------+  +----------+  +----------+       |
+----------------------------------------------------------------+
       |
       v
+--------------+
|   Turso DB   |
|  (primary +  |
|   replicas)  |
+--------------+
```

Each Golem VM maintains an **outbound** WebSocket to Styx (the global knowledge relay at `wss://styx.bardo.run`). This connection is initiated by the Golem, not the control plane. The control plane knows nothing about Styx -- it only manages VM lifecycle.

---

## Host header routing

The `bardo-control` monolith uses Host header inspection to route requests:

- `api.bardo.money` -- API routes (`/v1/*`, `/admin/*`, `/internal/*`)
- `*.bardo.money` -- Proxy middleware forwards to Golem VM via Fly 6PN
- `ssh.bardo.money` -- WebSocket SSH handler (upgrade to SSH tunnel)

The SSH WebSocket handler runs as a separate Fly `processes` group within the same app, with its own `kill_timeout = 300` (5-minute drain for active SSH sessions). This prevents SSH drain from blocking rolling deploys of the API/proxy process group.

---

## App specifications

| Fly app | Purpose | Min instances | VM size | Regions | Scaling |
| --- | --- | --- | --- | --- | --- |
| `bardo-control` | API, proxy, TTL worker, admin, billing | 2 | shared-cpu-2x, 1GB | ams, ord | Autoscale to 10 |
| `bardo-machines` | Golem VMs (6PN, no public IPs) | N/A | Per request | Per user | Per deployment |

---

## What runs inside each Golem VM

The Golem VM image contains a single Rust binary (`golem-binary`) compiled from the `bardo-golem-rs` Cargo workspace. This binary includes:

- **`golem-runtime`**: Extension registry, hook dispatch, type-state lifecycle
- **`golem-heartbeat`**: 9-step decision pipeline, gating, FSM
- **`golem-grimoire`**: LanceDB + SQLite + PLAYBOOK.md knowledge store
- **`golem-daimon`**: PAD affect engine, somatic markers
- **`golem-mortality`**: Three death clocks, vitality phases (BehavioralPhases: Thriving through Terminal), Thanatopsis (four-phase death protocol)
- **`golem-dreams`**: NREM replay, REM imagination, consolidation
- **`golem-custody`**: Delegation/Embedded/LocalKey wallet management
- **`bardo-tools`**: 423+ DeFi tools, Alloy on-chain, capability-gated
- **`bardo-styx`**: Outbound WebSocket to Styx, clade sync, pheromone reads

Configuration is injected via `golem.toml` at machine creation time.

```rust
/// Top-level Golem configuration, injected via Fly file injection
/// or generated by `bardo init` for self-hosted deployments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemConfig {
    pub golem_id: String,
    pub owner_address: Address,
    pub custody: CustodyMode,
    pub strategy_path: PathBuf,
    pub grimoire_path: PathBuf,
    pub styx_url: String,           // "wss://styx.bardo.run/v1/styx/ws"
    pub heartbeat_interval_secs: u64,
    pub inference: InferenceConfig,
    pub tool_profile: ToolProfile,
    pub vm_tier: VmTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmTier { Micro, Small, Medium, Large }
```

---

## Data stores

| Store | Role | Why |
| --- | --- | --- |
| **Turso (libSQL)** | Sole source of truth for machine state, billing, audit trail, TTL enforcement, concurrency control | ACID transactions, embedded replicas for zero-latency reads, automatic primary failover, CAS for optimistic concurrency |
| **In-process LRU** | Proxy cache (1000 entries, 60s TTL) | Sub-ms machine lookup for subdomain-to-6PN routing, no external dependency |
| **Redis (optional)** | Rate limiting acceleration only | If present, accelerates rate limit checks. If absent, falls back to in-memory. Never used for correctness. |

**Turso is authoritative.** There is no Redis-backed state that gates correctness. TTL enforcement, payment verification, and machine lifecycle are all driven by Turso queries.

---

## Subdomain routing summary

| Domain | Destination | Platform |
| --- | --- | --- |
| `bardo.money`, `www.bardo.money` | Landing page + compute dashboard UI | Vercel (Next.js) |
| `api.bardo.money` | REST API, billing, admin, TTL worker | Fly (`bardo-control`) |
| `*.bardo.money` (golem subdomains) | Reverse proxy to Golem VM | Fly (`bardo-control`) |
| `ssh.bardo.money` | WebSocket SSH handler | Fly (`bardo-control`, SSH process group) |

---

## Proxy implementation

Proxy logic is a Hono middleware within `bardo-control`. Host header check determines routing: golem subdomain proxies to VM, API subdomain falls through to API routes, SSH subdomain hands off to WebSocket handler.

```rust
// Pseudocode -- the control plane is Rust (Axum), not TypeScript
async fn proxy_middleware(req: Request) -> Response {
    let host = req.headers().get("host").unwrap_or_default();
    let subdomain = host.split('.').next().unwrap_or("");

    match subdomain {
        "api" => next(req).await,
        "ssh" => handle_ssh_websocket(req).await,
        name => {
            let machine = lookup_machine(name).await;
            match machine {
                Some(m) if m.status == "ready" => {
                    let path = req.uri().path();
                    if path.starts_with("/owner/") || path.starts_with("/rpc/") {
                        let jwt = extract_bearer(&req);
                        if !verify_privy_jwt(jwt, &m.user_id).await {
                            return Response::unauthorized();
                        }
                        proxy_to_machine(&req, &m.private_ip, 3001).await
                    } else {
                        proxy_to_machine(&req, &m.private_ip, 3000).await
                    }
                }
                _ => Response::not_found("GOLEM_NOT_FOUND"),
            }
        }
    }
}
```

### In-process proxy cache

LRU cache (1000 entries, 60s TTL) with singleflight pattern to prevent thundering herd on cache miss. No Redis dependency for proxy caching.

---

## Two-tier endpoint model

Each Golem VM exposes three ports:

| Port | Server | Audience | Auth | Purpose |
| --- | --- | --- | --- | --- |
| `:3000` | Public endpoint | Anyone (via proxy) | None | Agent discovery, tool exposure, strategy metadata, performance, TTL |
| `:3001` | Auth-gated endpoint | Owner only (via proxy) | Privy JWT | Strategy CRUD, steers, wallet ops, logs, Grimoire |
| `:3002` | Internal endpoint | Control plane (6PN only) | Fly OIDC token | Health checks, TTL push updates, crash reports |

---

## Public endpoint routes (`:3000`)

| Method | Path | Rate limit | Description |
| --- | --- | --- | --- |
| `GET` | `/health` | 60/min/IP | Golem health, uptime, TTL remaining |
| `GET` | `/info` | 30/min/IP | Strategy type, VM size, capabilities, region |
| `GET` | `/strategy/meta` | 30/min/IP | Strategy type, risk level, target protocols, asset |
| `GET` | `/performance` | 30/min/IP | PnL, NAV, share price, drawdown, uptime |
| `GET` | `/ttl` | 30/min/IP | TTL remaining, pricing tiers, extension history |
| `GET` | `/tools` | 30/min/IP | List exposed tools (with x402 prices) |
| `POST` | `/tools/call` | 10/min/caller | Execute an exposed tool (rate-limited, x402-gated) |
| `GET` | `/grimoire/insights` | 5/min/IP | Public Grimoire insights |

Golems expose a subset of `bardo-tools` for agent-to-agent calls. Tool list is discoverable via `/tools` and executable via `/tools/call`. Pricing is expressed via x402 headers.

---

## Auth-gated routes (`:3001`)

Owner-only routes for strategy management, runtime control, wallet operations, monitoring, and Grimoire access. All require valid Privy JWT verified against the machine's `userId`.

`PUT /owner/strategy` writes `STRATEGY.md` and sends a steer to the heartbeat pipeline. The Golem picks up the new strategy on its next planning cycle. No restart needed.

---

## Health endpoints

| Endpoint | Location | Auth | Purpose | Response |
| --- | --- | --- | --- | --- |
| `GET /health` | API (`api.bardo.money`) | None | Control plane health | `{ status, version, checks: { db, fly_api }, uptime }` |
| `GET /health` | VM `:3000` (public) | None | Golem runtime health | `{ status, uptime, vitality, tool_count, ttl_remaining }` |
| `GET /health` | VM `:3002` (internal) | OIDC | Provisioning health + disk | `{ status, uptime, disk_usage_percent, styx_connected }` |

The public health endpoint includes `vitality` (the composite vitality score from the three death clocks) and `styx_connected` (whether the Styx WebSocket is active). These replace the TypeScript-era `golemPid` and `toolCount` fields.

---

## Consistent identifiers

| Audience | Identifier | Example |
| --- | --- | --- |
| Public | `:name` | `golem-V1StGXR8_Z5j` |
| User | `:name` | `golem-V1StGXR8_Z5j` |
| Internal | Fly OIDC `machine_id` claim | (auto from token) |
| Admin | `:id` (UUID) | `550e8400-e29b-41d4-...` |

---

## SSE and WebSocket support

### SSE dashboard events

Real-time dashboard updates via SSE from the control plane:

```
GET /v1/machines/mine/events
Authorization: Bearer <privy-jwt>
Accept: text/event-stream

event: status_change
data: {"machine_name":"golem-V1St...","status":"ready","expires_at":1709740800}

event: extended
data: {"machine_name":"golem-V1St...","new_expires_at":1709776800,"payer_type":"external_user"}

event: destroyed
data: {"machine_name":"golem-V1St...","reason":"ttl_expired"}
```

Keepalive: `: keepalive\n\n` every 55 seconds.

### GolemEvent streaming

The Golem's Event Fabric broadcasts 50+ typed `GolemEvent` variants over its outbound Styx WebSocket. The TUI subscribes to these events for real-time display. See [06-api.md](06-api.md) for the event catalog.

### WebSocket SSH

Ticket-based authentication (no JWT in query strings):

1. Client: `POST /v1/ssh/ticket` with Privy JWT
2. API: Generate ticket (UUID, 30s TTL, in-memory Map)
3. Client: `wss://ssh.bardo.money/ws?machine=golem-V1StGXR8_Z5j&ticket=550e8400-...`
4. Proxy: Consume ticket (single-use), open SSH via 6PN, relay bidirectionally

Sticky sessions via Fly `fly-replay` header for WebSocket duration.

---

## CORS

```rust
const ALLOWED_ORIGINS: &[&str] = &[
    "https://bardo.money",
    "https://www.bardo.money",
    "https://app.bardo.money",
];
```

`Vary: Origin` header set for multi-origin support. WebSocket upgrades skip CORS.

---

## Agent discovery

`GET /v1/golems` with query params for filtering by strategy type, asset, region, and status. Returns public metadata only. This endpoint enables both human browsing and agent-to-agent discovery.

---

## Agent-to-agent communication

Tool calls via public endpoint (`/tools/call`), rate limited, x402-gated. Full A2A protocol support (ERC-8001) deferred to phase 2.

---

## Operational details

### Turso failover

Read replicas are deployed in the same region as the primary. Write failures (primary unavailable) surface as user-facing errors with code `COMPUTE_STORE_WRITE_FAILED`. No silent retry -- explicit retry with idempotency keys is the v1 pattern.

### SSH ticket lifecycle

Tickets are stored in Turso with a `used_at` timestamp. Replay attacks are prevented by the one-time-use constraint. 30-second TTL. See [04-security.md](04-security.md) for the full SSH security model.

---

## Cross-references

| Topic | Document |
| --- | --- |
| VM provisioning | [02-provisioning.md](02-provisioning.md) — Warm pool mechanics, Docker image, entrypoint script, golem.toml injection, machine lifecycle states, and graceful shutdown via Thanatopsis. |
| Payment and TTL | [03-billing.md](03-billing.md) — EIP-3009 x402 payment flow, pricing tiers, metabolic self-funding loop, inference billing, TTL calculation, and two-layer TTL enforcement. |
| Security and auth | [04-security.md](04-security.md) — VM isolation, three-mode custody security, attacker taxonomy, top 5 threats with mitigations, four authentication contexts, and rate limiting matrix. |
| Operations | [05-operations.md](05-operations.md) — Multi-region deployment, TTL worker HA, Styx health monitoring, reconciliation job, OpenTelemetry metrics, and failure mode matrix. |
| API reference | [06-api.md](06-api.md) — Full REST API specification, Turso database schema (6+ tables), GolemEvent types enum, and HTTP error codes. |
| GolemEvent catalog | [prd2/01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) — The 28-extension architecture: `Extension` trait with 20 async lifecycle hooks, Event Fabric broadcasting 50+ typed `GolemEvent` variants, and topological hook dispatch order. |
