# Styx Infrastructure: Deployment, Scaling, and Operations [SPEC]

> **PRD2 Section**: 20-styx | **Source**: Styx Research S3 v4.0

> **Status**: Implementation Specification
>
> **Dependencies**: `prd2/20-styx/00-architecture.md` (what Styx does), `prd2/20-styx/01-api.md` (API surface)

> **Reader orientation:** This document specifies the hosting architecture, deployment topology, scaling path, monitoring, cost projections, and security model for Styx (global knowledge relay and persistence layer at wss://styx.bardo.run; three tiers: Vault/Clade/Lethe). It belongs to the Styx infrastructure layer of Bardo. The key concept is that Styx is a single stateless Rust/Axum gateway deployed multi-region on Fly.io, backed by managed data services. Familiarity with the Styx architecture (`00-architecture.md`) and API surface (`01-api.md`) is assumed. See `prd2/shared/glossary.md` for full term definitions.

---

## What This Document Covers

Styx is a single public service that must achieve near-perfect uptime, sub-100ms query latency globally, and scale from 10 to 10,000+ Golems without architectural changes. This document specifies:

1. The hosting architecture and why each component was chosen
2. The deployment topology (multi-region active-active)
3. The Styx binary structure
4. The scaling path from launch to global scale
5. Monitoring and alerting
6. Cost projections at each scale tier
7. The security and trust model (honest SaaS)

---

## 1. Architecture: Fly.io + Managed Data Services

### Design Principles

A single public service serving the entire ecosystem requires:

1. **Multi-region redundancy**: A single server is a single point of failure. The service must survive individual datacenter outages without interruption.
2. **Automatic failover**: No human intervention needed to recover from a node failure. Health checks must detect failure and route traffic to healthy nodes within seconds.
3. **Elastic scaling**: Market crashes cause correlated Golem death events (many Golems die simultaneously, uploading bloodstains and pheromone deposits concurrently). The service must absorb traffic spikes without manual intervention.
4. **Predictable latency**: Golems run globally. A Golem in Tokyo querying Styx should get comparable latency to one in New York.
5. **Operational simplicity**: This is run by a single operator. The infrastructure must be manageable by one person with standard DevOps skills, not a dedicated SRE team.

### Why Fly.io for the Gateway

Fly.io runs applications on bare-metal servers distributed across 35+ regions worldwide. A Rust/Axum binary compiles to a single static Docker image and deploys to Fly.io as a lightweight VM (Firecracker microVM). Key properties:

- **Multi-region active-active**: Deploy the same binary to 2+ regions. Fly's Anycast routing sends each request to the nearest healthy instance. If one region goes down, traffic automatically routes to the next nearest.
- **Health checks with auto-restart**: Fly monitors each instance with configurable health checks (TCP, HTTP, or custom). Failed instances are restarted automatically. If an instance can't recover, a new one is spun up.
- **Zero-downtime deploys**: Blue-green deployments are built-in. The new version starts, passes health checks, then traffic shifts. No requests are dropped.
- **Predictable pricing**: Per-VM pricing based on CPU/RAM allocation, not per-request. A 2-vCPU / 4GB instance is ~$30/month. Two regions = ~$60/month for the gateway.
- **Volumes for local state**: Fly Volumes provide persistent NVMe storage attached to instances, used for local caches and temporary state.

### Why Managed Data Services (Not Self-Hosted)

Self-hosting databases on a single bare-metal server is a single point of failure. For a public service requiring near-perfect uptime, managed services provide automatic replication, failover, and backups without operational burden:

| Component | Service | Why | Scaling | Durability |
|-----------|---------|-----|---------|-----------|
| **Vector Search** | Qdrant Cloud | Managed Qdrant cluster with automatic replication, HNSW indexing, filtering. Rust-native client library. Sub-10ms queries on warm data. | Auto-scales with data volume | Replicated across nodes |
| **Relational DB** | Neon Postgres | Serverless Postgres with autoscaling compute, branching for dev/test, connection pooling. Separates storage from compute -- scales independently. Multi-region read replicas for low-latency reads. | Auto-scales compute; storage scales independently | Multi-AZ replication |
| **Cache + Pub/Sub** | Upstash Redis | Serverless Redis with per-request pricing, global replication, REST API (works from Fly.io edge). Used for: pheromone field cache, semantic result cache, rate limiting, WebSocket pub/sub. | Auto-scales | Redis replication |
| **Blob Storage** | Cloudflare R2 | S3-compatible, zero egress fees. Stores: Grimoire backups, death bundles, marketplace encrypted content. Lifecycle rules handle TTL expiry. | Unlimited | 11 nines (S3-class) |
| **Edge / DDoS** | Cloudflare | Free plan: DDoS protection, TLS termination, caching. Fly.io instances are origin servers behind Cloudflare. | N/A | N/A |

### Why Not Bare Metal?

A single bare-metal server (e.g., Hetzner AX42, ~$60/month) is cheaper but introduces:
- **Single point of failure**: One server = one failure domain. Disk failure, network outage, or datacenter maintenance takes the service offline.
- **Manual failover**: Requires human intervention or scripting to fail over to a backup.
- **No global distribution**: Users in Asia get 200ms+ latency to a European server.

Bare metal is appropriate for development and testing. For the production public service, the Fly.io + managed services architecture costs more (~$200-400/month at launch vs. ~$60) but provides the reliability guarantees a public ecosystem service requires.

---

## 2. Deployment Topology

```
                         +--------------------+
                         |   Cloudflare        |
                         |   (DDoS + TLS       |
                         |    + edge cache)     |
                         +---------+-----------+
                                   | Anycast
                     +-------------+--------------+
                     |                             |
               +-----+------+              +------+-----+
               | Fly.io     |              | Fly.io     |
               | Region: IAD|              | Region: AMS|
               | (Virginia) |              | (Amsterdam)|
               |            |              |            |
               | styx-gw    |              | styx-gw    |
               | (Axum)     |              | (Axum)     |
               +------+-----+              +------+-----+
                      |                           |
         +------------+------------+--------------+
         |            |            |              |
   +-----+----+ +----+-----+ +---+----+   +----+-----+
   | Qdrant   | | Neon     | |Upstash |   | CF R2    |
   | Cloud    | | Postgres | | Redis  |   | (blobs)  |
   | (vectors)| | (meta)   | |(cache) |   |          |
   +----------+ +----------+ +--------+   +----------+
```

Both Fly.io instances run the identical `styx-gw` Rust binary. They connect to the same shared data services. Fly's Anycast routing ensures each request hits the nearest healthy instance.

### Statelessness

The gateway instances are stateless -- all persistent state lives in the managed data services. This means:
- Any instance can handle any request
- Instances can be added/removed without data migration
- A crashed instance is replaced by a fresh one with zero state recovery needed
- Rolling deploys update one instance at a time with no downtime

The only "state" on a gateway instance is the in-process WebSocket connections. When an instance restarts, clients reconnect (with exponential backoff) and replay from their last-seen sequence number via the Event Fabric's replay mechanism.

---

## 3. The Styx Binary

A single Rust binary compiled with Axum:

```rust
// styx-gw/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::init();

    let config = StyxConfig::from_env()?;

    // Connect to managed data services
    let qdrant = QdrantClient::new(&config.qdrant_url).await?;
    let db = sqlx::PgPool::connect(&config.database_url).await?;
    let redis = redis::Client::open(config.redis_url.clone())?;
    let r2 = s3::Bucket::new("bardo-styx", config.r2_region(), config.r2_credentials())?;

    // Shared application state
    let state = AppState {
        qdrant: Arc::new(qdrant),
        db: db.clone(),
        redis: Arc::new(redis),
        r2: Arc::new(r2),
        event_bus: Arc::new(EventBus::new()),
        config: Arc::new(config),
    };

    // Spawn background tasks
    tokio::spawn(pheromone_evaporation_task(state.clone()));
    tokio::spawn(ttl_expiry_task(state.clone()));
    tokio::spawn(pulse_aggregation_task(state.clone()));
    tokio::spawn(delayed_lethe_publication_task(state.clone()));

    // Build Axum router
    let app = Router::new()
        // Knowledge CRUD
        .route("/v1/styx/entries", post(entries::create).get(entries::query))
        .route("/v1/styx/snapshot/:golem_id", get(entries::snapshot))
        // Pheromone Field
        .route("/v1/styx/pheromone/deposit", post(pheromone::deposit))
        .route("/v1/styx/pheromone/sense", get(pheromone::sense))
        // Bloodstain Network
        .route("/v1/styx/bloodstain", post(bloodstain::upload))
        // Causal Federation
        .route("/v1/styx/causal/publish", post(causal::publish))
        .route("/v1/styx/causal/discover", get(causal::discover))
        // Engagement
        .route("/v1/styx/lineage/:user_id", get(engagement::lineage))
        .route("/v1/styx/graveyard/:user_id", get(engagement::graveyard))
        .route("/v1/styx/achievements/:golem_id", get(engagement::achievements))
        // Ecosystem
        .route("/v1/styx/pulse", get(ecosystem::pulse))
        .route("/v1/styx/health", get(ecosystem::health))
        // WebSocket
        .route("/v1/styx/ws", get(ws::handler))
        // Middleware
        .layer(middleware::from_fn_with_state(state.clone(), auth::authenticate))
        .layer(middleware::from_fn(ratelimit::limit))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Styx gateway listening on :8080");
    axum::serve(listener, app).await?;

    Ok(())
}
```

### Background Tasks

Four tokio tasks run continuously:

1. **Pheromone evaporation** (every 60s): Apply exponential decay to all pheromones. Remove those below 0.05 intensity. Update Redis cache.
2. **TTL expiry** (every hour): Delete L0 entries past their TTL. Remove expired marketplace listings. Purge R2 blobs.
3. **Pulse aggregation** (every 60s): Compute ecosystem-wide statistics. Cache in Redis. Serve from `/v1/styx/pulse`.
4. **Delayed Lethe (formerly Lethe) publication** (every 5 min): Process the queue of entries awaiting randomized publication delay (1-6h). Anonymize and publish those past their delay.

---

## 4. Scaling Path

| Scale | Active Golems | Architecture | Monthly Infra Cost |
|-------|--------------|-------------|-------------------|
| **Dev** | 1-10 | Single Fly.io instance (1 vCPU / 1GB). Neon free tier. Upstash free tier. Qdrant free tier (1GB). | ~$15 |
| **Launch** | 10-50 | 2x Fly.io instances (2 regions). Neon starter. Qdrant starter (4GB). | ~$120 |
| **Traction** | 50-200 | Same 2-region setup. Neon scales compute automatically. Qdrant grows to ~16GB. | ~$250 |
| **Growth** | 200-1,000 | 3x Fly.io instances (3 regions). Qdrant cluster (2 nodes). Neon with read replicas. | ~$500 |
| **Scale** | 1,000-5,000 | 4+ Fly.io instances. Qdrant cluster (3+ nodes). Neon production tier. | ~$1,200 |
| **Global** | 5,000+ | 6+ regions. Qdrant enterprise. Neon enterprise. Dedicated Upstash cluster. | ~$3,000+ |

At every scale tier, the Golem-facing API does not change. The URL is the same. The contract is the same. Scaling is an infrastructure concern, not an API concern.

### Egress Analysis

Pheromone batches dominate egress (~88% of bandwidth). At 10,000 Golems with naive fan-out, egress would be ~10TB/month. Three mitigations:

1. **Topic-based pub/sub**: Golems subscribe to domains they care about. At 1,000 Golems across 50 domains, that's ~20 Golems per domain instead of broadcasting to all 1,000. A 50x fan-out reduction.
2. **Message batching**: One batch per domain per tick instead of individual messages. ~13x message count reduction.
3. **Zstd compression**: 3-5x bandwidth reduction on pheromone payloads.

With all three: 10K-Golem egress drops to ~2-3TB/month. At Fly.io's $0.02/GB rate, that's ~$50-60/month in bandwidth costs instead of ~$200/month with naive fan-out.

### Infrastructure Cost Breakdown by Component

| Component | 100 Golems | 1,000 Golems | 10,000 Golems |
|---|---|---|---|
| Compute (Fly.io, 2 regions) | $60/mo | $120/mo | $400/mo |
| Vector search (Qdrant Cloud) | Free tier | $50/mo | $200/mo |
| Database (Neon Postgres) | Free tier | $50/mo | $150/mo |
| Cache (Upstash Redis) | Free tier | $20/mo | $80/mo |
| Blob storage (Cloudflare R2) | ~$0 | $5/mo | $50/mo |
| Edge/DDoS (Cloudflare) | Free | Free | Free |
| NATS (federation, if enabled) | N/A | N/A | $20/mo |
| **Total** | **~$60/mo** | **~$250/mo** | **~$900/mo** |

### x402 Revenue Model

See `shared/x402-protocol.md` for the x402 payment protocol specification.

| Service | Price | Monthly revenue at 1,000 Golems |
|---|---|---|
| Entry write (L0/L1/L2) | $0.001/write | ~$1,500 (50K writes/day) |
| Knowledge query | $0.002/query | ~$6,000 (100K queries/day) |
| Pheromone deposit | $0.0005/deposit | ~$15 |
| Pheromone read | $0.0005/read | ~$3,000 (5.76M reads/month) |
| Bloodstain upload | $0.005/upload | ~$5 |
| Snapshot retrieval | $0.01/snapshot | ~$100 |
| Marketplace listing | $0.01/listing | ~$50 |
| Marketplace purchase commission | 5% protocol fee | Variable |
| Event relay | $0.001/1K events | ~$175 |
| **Estimated total** | | **~$8,000+/mo** |

At 1,000 Golems, x402 revenue covers infrastructure costs ($250/mo) by a wide margin. The model is self-sustaining well before ecosystem scale.

### Revenue by Scale Tier

| Scale | Active Golems | Writes/day | Queries/day | Monthly revenue | Monthly infra | Net |
|---|---|---|---|---|---|---|
| Launch | 10 | 500 | 1,000 | ~$75 | ~$120 | -$45 |
| Traction | 50 | 2,500 | 5,000 | ~$375 | ~$150 | +$225 |
| Growth | 200 | 10,000 | 20,000 | ~$1,500 | ~$250 | +$1,250 |
| Marketplace | 500 | 25,000 | 50,000 | ~$4,000 + marketplace GMV | ~$500 | +$3,500+ |
| Scale | 1,000 | 50,000 | 100,000 | ~$8,000 + marketplace GMV | ~$1,200 | +$6,800+ |

The service is unprofitable at launch with 10 Golems. This is expected -- the infrastructure baseline exists whether there are 10 Golems or 200. At 50 Golems, revenue covers costs. At 200+, the margin is comfortable.

The 5% marketplace protocol fee becomes a significant revenue stream at scale. At 500 active Golems with ~$10K/month marketplace GMV, the fee adds ~$500/month. Death archives (the marketplace's most distinctive product) drive volume: every Golem that dies can produce a listing, and every new Golem is a potential buyer.

### What Styx Charges For, What's Free

**Free**: Health checks, ecosystem pulse, lineage queries, graveyard queries, achievement queries, clade peer discovery, causal edge publication (for Verified+ agents), listing search.

**Paid**: Everything that writes data (entries, pheromone deposits, bloodstains, marketplace listings), everything that reads knowledge (queries, snapshots, pheromone reads), and event relay.

The free tier is designed for discoverability. You can browse the marketplace, check the ecosystem pulse, view lineage trees, and discover clade peers without paying anything. The moment you want to write knowledge, query knowledge, or relay events, x402 kicks in.

### Self-Hosted Economics

Self-hosted Styx has zero x402 charges (you don't pay yourself). Infrastructure costs only:

| Deployment | Monthly cost |
|---|---|
| Fly.io (shared-1x) | $5-20 |
| Hetzner VPS (CX22) | $4 |
| Raspberry Pi (electricity) | $1-3 |
| Local Docker | $0 |

If your clade has 5 Golems and you want clade sync + pheromone field + L0 backup, self-hosting saves $15-75/month compared to the managed Styx. The tradeoff: you manage uptime, updates, and scaling. No marketplace access unless you federate with the ecosystem Styx.

---

## 5. Monitoring and Alerting

| What | How | Alert Threshold |
|------|-----|----------------|
| Service health | Fly.io health checks (HTTP GET /v1/styx/health, 10s interval) | 3 consecutive failures -> auto-restart instance |
| Query latency | OpenTelemetry traces -> Axiom/Datadog | p99 > 500ms -> alert |
| Error rate | Structured logs (tracing crate) -> log aggregator | >1% 5xx responses -> alert |
| Qdrant health | Qdrant Cloud dashboard + API health endpoint | Cluster degraded -> alert |
| Neon health | Neon dashboard + connection pool monitoring | Connection pool exhausted -> alert |
| Pheromone field size | Custom metric (total active pheromones) | >100K active pheromones -> investigate (potential spam) |
| Storage growth | R2 usage metrics + Neon storage metrics | >80% of provisioned storage -> scale up |
| WebSocket connections | Custom counter in Axum state | >10K concurrent connections -> add instance |

---

## 6. Security and Trust Model

### The Honest Model

Styx operates a **standard SaaS trust model** -- identical to how every Qdrant Cloud customer trusts Qdrant, every Neon customer trusts Neon, every AWS customer trusts AWS:

| Threat | Protection |
|--------|-----------|
| External attackers (network) | TLS 1.3 everywhere (Cloudflare -> Fly.io -> data services) |
| External attackers (storage) | At-rest encryption on all managed services (Qdrant, Neon, R2) |
| DDoS | Cloudflare DDoS protection (free plan) |
| Cross-user data leakage | Namespace isolation: each user's L0/L1 data lives in a separate Qdrant namespace (`vault:{user_id}`, `clade:{user_id}`). Access control enforced at the Axum middleware layer. |
| Credential compromise | ERC-8004 identity + x402 micropayments = wallet-based auth (no passwords to steal) |
| Data retention | TTL-based expiry enforced by background task + R2 lifecycle rules |

### What Is NOT Protected Against

The service operator can technically read L0/L1 data (necessary for server-side vector search and retrieval). This is the same trust model as every cloud service. Protection comes from:
- Business reputation and legal agreements (ToS)
- Economic incentives (revenue depends on user trust)
- Audit logging on all data access
- The fact that L0/L1 data is DeFi trading knowledge, not nuclear secrets -- the threat model is proportionate

### L2 Lethe: Public After Anonymization

Lethe data is stored in plaintext. The anonymization pipeline (see `prd2/20-styx/01-api.md` section 4) is the privacy layer. No encryption on top of anonymized public data -- the audit established this is the honest approach.

---

## 7. Fly.io Configuration

```toml
# fly.toml

app = "bardo-styx"
primary_region = "iad"  # Virginia

[build]
  dockerfile = "Dockerfile"

[env]
  RUST_LOG = "info,styx_gw=debug"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = false   # Always running -- this is a public service
  auto_start_machines = true
  min_machines_running = 2     # Always at least 2 for HA
  processes = ["app"]

  [http_service.concurrency]
    type = "requests"
    hard_limit = 1000
    soft_limit = 800

[[http_service.checks]]
  grace_period = "10s"
  interval = "15s"
  method = "GET"
  path = "/v1/styx/health"
  timeout = "5s"

[[vm]]
  size = "shared-cpu-2x"  # 2 vCPU, 4GB RAM
  memory = "4096"
  cpus = 2
```

Deploy to multiple regions:

```bash
# Deploy to Virginia (primary) + Amsterdam (secondary)
fly deploy
fly scale count 2 --region iad,ams

# Verify multi-region
fly status
# Should show 2 instances: 1 in iad, 1 in ams
```

---

## References

- [FLY-IO] Fly.io. "Run Your Full Stack Apps Globally." https://fly.io — *Multi-region Firecracker microVM platform used for the Styx gateway.*
- [QDRANT] Qdrant. "Vector Search Engine." https://qdrant.tech — *Managed vector search service for Grimoire and marketplace embedding queries.*
- [NEON] Neon. "Serverless Postgres." https://neon.tech — *Serverless PostgreSQL with autoscaling compute, used for Styx relational data.*

---

*One service. Multi-region. Auto-healing. The infrastructure should be invisible -- what matters is the knowledge flowing through it.*
