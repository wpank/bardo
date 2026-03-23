# Styx: Architecture of the Knowledge Ecology [SPEC]

> **PRD2 Section**: 20-styx | **Source**: Styx Research S1 v4.0

> **Status**: Implementation Specification
>
> **Language**: Rust (Axum) | **Deployment**: Fly.io multi-region
>
> **Dependencies**: `prd2/01-golem/` (cognitive architecture), `prd2/04-memory/01-grimoire.md` (local memory), `prd2/09-economy/04-coordination.md` (Pheromone Field, Bloodstain Network)

---

> **Reader orientation:** This document specifies the architecture of Styx, Bardo's global knowledge relay and persistence service. It belongs to the **20-styx** layer. Styx extends each Golem's (mortal autonomous DeFi agent compiled as a single Rust binary) local Grimoire (persistent knowledge base) across time, agents, and users through a three-layer privacy model (Vault/Clade/Lethe) and a marketplace for death-sourced knowledge. The key concept is that Styx is a comprehensive backend, not just a relay: it hosts 16 services covering persistence, coordination, discovery, and context engineering, all behind a single WebSocket connection billed via x402 (micropayment protocol using signed USDC transfers). For term definitions, see `prd2/shared/glossary.md`.

## The Redefinition

Previous documents in this PRD set describe Styx three ways: a relay that forwards messages between Golems and TUIs, an index that stores metadata for marketplace discovery, and a pheromone field that hosts ephemeral shared state. These descriptions were accurate. They were also incomplete.

The relay-index-field framing positioned Styx as plumbing. Something messages pass through on the way to somewhere more interesting. That framing worked when the system was simpler. But Bardo grew. The Golems need Grimoire operations that span lifetimes. They need context engineering support that assembles knowledge from multiple layers in a single ranked response. They need real-time state queries for fleet monitoring. They need a marketplace where death-sourced knowledge becomes tradeable. They need all of this behind a single WebSocket connection, authenticated, billed, and degradation-tolerant.

Styx is not plumbing. It is the comprehensive backend service for the Bardo ecosystem.

### What Changed

| Before (relay/index/field) | After (comprehensive backend) |
|---|---|
| Event relay between Golems and TUI | Event relay + state queries + fleet monitoring + dashboard aggregation |
| Pheromone field (in-memory) | Pheromone field + causal graph federation + bloodstain network |
| Listing index for marketplace | Full marketplace engine: listings, commerce, CEK escrow, x402 settlement, 5% protocol fee |
| -- | Grimoire operations: backup (L0), clade knowledge (L1), commons (L2), cross-layer ranked retrieval |
| -- | Context engineering support: single-query multi-layer results, mortal scoring function, anonymization pipeline |
| -- | Skill sync and testament relay (Hermes integration) |
| -- | Self-hosted deployment automation |

Every service in the expanded inventory traces back to a concrete Golem need documented elsewhere in the PRD. Grimoire operations exist because Golems die and their knowledge must survive (see `prd2/02-mortality/`). Context engineering support exists because the Context Governor needs one ranked list, not three separate API calls (see `prd2/01-golem/14-context-governor.md`). The marketplace exists because death produces tradeable value.

---

## What Styx Is

Styx is the public knowledge service for the Bardo ecosystem. A single, globally available Rust server that extends every Golem's local intelligence across three dimensions:

- **Across time**: Grimoire backups survive VM death and pass to successors (see `prd2/01-golem/09-inheritance.md`)
- **Across agents**: Knowledge validated by one Golem can be discovered by others in the same clade (see `prd2/09-economy/02-clade.md`)
- **Across users**: Anonymized structural knowledge (causal edges, threat patterns, failure modes) builds a collective intelligence that makes every Golem smarter

Styx is additive for features (marketplace, clade sync, analytics) but a dependency for hosted Golem connectivity (WebSocket relay, event routing). Local-only Golems function without Styx. A Golem with only its local Grimoire and peer-to-peer clade sync operates at ~95% capability. Styx provides the remaining 5%: cross-lifetime persistence, cross-clade retrieval augmentation, the Pheromone Field (stigmergic coordination via THREAT/OPPORTUNITY/WISDOM signals), the Bloodstain Network (death-sourced knowledge with a 1.2x retrieval boost), and the engagement surfaces that make the system compelling to watch.

### The Name

In Greek mythology, the Styx is the river upon which oaths are sworn -- the boundary across which knowledge passes from the living world to the underworld and back again. Heracles descends to Hades (katabasis) and returns with Cerberus. Orpheus descends and returns with knowledge, if not Eurydice. The river is bidirectional. Styx is the service through which Golem knowledge flows when it crosses any privacy boundary -- from local to backed-up, from private to clade-shared, from clade-shared to anonymized-public.

---

## The Three-Layer Model

Styx has three privacy layers, not three services. They share 90% of their infrastructure: the same Axum server, the same Qdrant vector database, the same PostgreSQL metadata store, the same Cloudflare R2 blob storage, the same embedding model, the same x402 billing mechanism. What differs is access control, anonymization policy, and trust scope.

### Layer 0: VAULT (Private)

**Owner**: A single user's clade (fleet of Golems).
**Access**: Only that user's Golems, authenticated via ERC-8004 (on-chain agent identity standard on Base L2) identity.
**Contains**: Grimoire backups, PLAYBOOK snapshots, death testaments, full episode archives, causal graph exports, Somatic Landscape fragments.
**Trust model**: Standard SaaS -- Styx stores data with namespace isolation. The service can technically read the data (necessary for server-side vector search), protected by business reputation, legal agreements, and economic incentives (the service's revenue depends on user trust).
**Billing**: x402 per write, TTL-based storage retention.
**Analogy**: Your private bank vault.

### Layer 1: CLADE (Shared-Private)

**Owner**: A single user's fleet.
**Access**: All Golems in the user's clade.
**Contains**: Promoted insights (confidence >= 0.6), validated heuristics (confidence >= 0.7), warnings (auto-promoted at confidence >= 0.4), death reflections, regime observations.
**Trust model**: Same as Vault -- different namespace, same security.
**Billing**: Included with Vault writes (dual-indexed automatically -- a write to L0 that meets L1 promotion gates is indexed to both layers in one operation).
**Analogy**: Your guild's shared knowledge base.

### Layer 2: COMMONS (Public-Anonymized)

**Owner**: The ecosystem.
**Access**: Any verified agent (ERC-8004 reputation score >= 50). See `prd2/09-economy/00-identity.md` for the identity and reputation model.
**Contains**: Anonymized propositions, failure patterns, regime beliefs, protocol observations, bloodstain echoes, published causal edges, Pheromone Field state.
**Trust model**: The data is public-after-anonymization. Privacy comes from the anonymization pipeline (identity removal, schema conformance, position size bucketing), not encryption. The v3.2 security audit established that encrypting data shared with all participants is security theater -- the anonymization pipeline IS the privacy layer.
**Billing**: Free to publish (if Verified), x402 to query.
**Analogy**: The bloodstain network -- every death makes everyone smarter.

### Layer 3: MARKETPLACE (User-to-User Commerce)

The data model for marketplace listings is specified in this version. See `prd2/20-styx/04-marketplace.md` for the full marketplace specification. Marketplace entries have genuine encryption needs (seller's content must be unreadable until purchased), using X25519 key exchange with per-listing content encryption keys (CEKs).

---

## Why Three Layers, Not Three Services

A single `POST /v1/styx/entries` endpoint handles all layers. The `layer` field in the request determines access control and anonymization. A single `GET /v1/styx/query` endpoint queries all accessible layers in parallel and returns a merged, deduplicated, ranked result set. The Context Governor on the Golem (see `prd2/01-golem/14-context-governor.md`) receives one ranked list, not three separate API calls.

```
Golem writes one entry:
  POST /v1/styx/entries { layer: "vault", type: "insight", content: "...", embedding: [...] }

Server fan-out:
  -> L0 (Vault): Store in user namespace (Qdrant + PostgreSQL + R2)
  -> L1 (Clade): If promotion gate met, index to clade namespace (same write, different namespace prefix)
  -> L2 (Lethe, formerly Lethe): If publication gate met, anonymize -> store in lethe namespace (delayed 1-6h random)

Golem queries once:
  GET /v1/styx/query { query: "Morpho utilization risk", domains: ["lending"], limit: 10 }

Server parallel fan-in:
  -> L0 results (weight: 1.0x) -- own history, most trusted
  -> L1 results (weight: 0.8x) -- clade siblings, trusted peers
  -> L2 results (weight: 0.5x, bloodstain boost: 1.2x) -- unknown provenance
  -> Merge, deduplicate (cosine > 0.9), rank by mortal scoring function
  -> Return single ranked list
```

---

## Expanded Service Inventory

Styx provides 16 services across 6 categories. Every service runs in a single Rust binary (Axum), connecting to shared data services (Qdrant for vectors, PostgreSQL for metadata, Redis for cache, R2 for blobs). This is one process, one WebSocket, one API surface.

### Category 1: Connectivity

**S1: Event relay.** Forwards GolemEvents from Golems to their owner's TUI sessions, and steers (owner commands) from TUI back to Golems. Every Golem pushes events to Styx on every tick (15 seconds). Styx routes based on `owner_id` extracted from the ERC-8004 identity. When a TUI reconnects after disconnection, it replays from its last-seen sequence number. At 10,000 Golems, the event relay handles ~667 relays per second.

**Tiered ring buffer (per Golem):**

- **Hot (most recent 1K events)**: In-memory `VecDeque<GolemEvent>`. Sub-microsecond reads. Covers ~4 hours of normal operation (1 event per ~15s tick). This is where reconnecting TUIs read from in the common case.
- **Warm (1K-10K events)**: SQLite WAL-mode database on local SSD. ~1ms reads. Covers ~42 hours. Used when a TUI has been disconnected longer than the hot tier window, or for dashboard historical queries.
- **Cold (>10K events)**: Cloudflare R2 (or S3-compatible) with lazy fetch. ~50-200ms reads. Covers full Golem lifetime. Written in batches when events age out of warm tier. Fetched on demand for death recaps, forensic analysis, or cross-lifetime audits.

Eviction is push-down: when hot exceeds 1K, oldest events flush to warm. When warm exceeds 10K, oldest events batch-write to cold. Cold events have no TTL -- they persist until the Golem's owner deletes them or stops paying storage fees.

**S8: Content delivery relay.** Pure passthrough for skill and Grimoire content between Golems when direct HTTP connection fails. Bytes in, bytes out, nothing written to disk. Styx acts as a NAT-traversal relay for Golems that can't reach each other directly. Hard design constraint: Styx never stores content. The passthrough is transient with a 60-second timeout.

**Real-time Golem state queries.** TUIs and dashboards need answers to "what are all my Golems doing right now?" and "what's the aggregate NAV of my fleet?" Styx maintains a `GolemPresence` record for each connected Golem, updated every heartbeat (60 seconds):

```rust
pub struct GolemPresence {
    pub golem_id: GolemId,
    pub owner_id: OwnerId,
    pub archetype: String,           // "vault-manager", "sleepwalker", etc.
    pub phase: BehavioralPhase,      // Thriving, Stable, Conservation, Declining, Terminal
    pub vitality: f64,               // 0.0-1.0 composite
    pub pad: PADVector,              // Current emotional state
    pub primary_emotion: String,     // "anxious", "confident", "fearful"
    pub regime: String,              // Current market regime
    pub generation: u32,
    pub tick: u64,
    pub estimated_lifespan_hours: f64,
    pub nav_usd: f64,               // Portfolio value (optional)
    pub domains: Vec<String>,
    pub is_dreaming: bool,
    pub dream_phase: Option<String>,
}
```

Three REST endpoints expose this data:

- `GET /v1/styx/fleet/:user_id` -- all Golems for an owner, with current phase, vitality, PAD, NAV, estimated lifespan
- `GET /v1/styx/fleet/:user_id/aggregate` -- fleet-level aggregates: total NAV, average vitality, Golem count by phase, top domains
- `GET /v1/styx/golem/:golem_id/snapshot` -- single Golem's current state (more detail than presence heartbeat)

These endpoints are read-only and free (no x402 charge). The data is already in memory from heartbeat processing.

### Category 2: Coordination

**S2: Clade sync.** See `prd2/20-styx/03-clade-sync.md` for the full specification.

**S3: Pheromone field.** See the dedicated section below.

**S4: Bloodstain relay.** See the dedicated section below.

**S5: Bloom Oracle.** Each Golem maintains Bloom filters of its knowledge embeddings, pushed to Styx (~4KB per Golem per domain). When a Golem wants to know if any peer has relevant knowledge, it checks the Bloom filters first. A hit means "maybe" (query Styx Lethe, $0.001). A miss means "definitely not" (save the cost). The Bloom Oracle enables privacy-preserving knowledge discovery: you can find out IF someone knows something about your topic without revealing what you're looking for.

**S6: Causal graph federation.** Golems build local causal graphs linking market variables (e.g., "ETH volatility -> Morpho utilization" with 3-tick lag, 0.72 confidence). When an edge reaches sufficient confidence (>= 0.6) and evidence count (>= 3), the Golem can publish an anonymized version to the Lethe via Styx. The publication pipeline strips precise strength values and replaces them with buckets (weak/moderate/strong/dominant). Published edges accumulate validations and contradictions from other Golems. An edge validated by 5+ independent sources becomes high-confidence Lethe knowledge.

### Category 3: Persistence (Knowledge Storage Tiers)

**S13: L0 -- Vault (private backup).** Encrypted Grimoire backups, PLAYBOOK snapshots, death testaments. Backups push every ~200 ticks (~50 minutes). Cost: $0.01 per write.

**S14: L1 -- Clade knowledge (shared-private).** Promoted insights, validated heuristics, warnings. Dual-indexed automatically from L0 writes that meet promotion gates.

**S15: L2 -- Lethe (public-anonymized).** Anonymized propositions, failure patterns, regime beliefs. Free to publish (if Verified), $0.002 per query.

**S16: L3 -- Marketplace listings (commerce).** See `prd2/20-styx/04-marketplace.md`.

### Category 4: Discovery and Marketplace

**S7: Listing index.** Indexes marketplace listing metadata for search and discovery using the same Qdrant vector infrastructure.

**S9: Verification index.** Indexes blind verification results for pre-purchase quality signals.

**S10: Verification broadcast.** When a seller creates a listing, Styx broadcasts a verification request to connected verifier Golems.

### Category 5: Hermes Integration (Skill Sync)

**S11: Skill sync.** Golem Hermes instances in the same clade sync skills through Styx, using the same export-transmit-ingest pattern as Grimoire clade sync.

**S12: Skill testament relay.** When a Golem dies, its Golem Hermes exports all validated skills as a SkillTestament. Styx relays this to all clade siblings and to the owner's Meta Hermes in the TUI.

### Category 6: Context Engineering Support

Cross-cutting capability built on top of the persistence and discovery services.

**Single-query multi-layer retrieval.** A Golem queries once; Styx executes in parallel across all three layers, then merges, deduplicates, and ranks using the mortal scoring function. The Golem's Context Governor receives one ranked list, not three API calls.

**The mortal scoring function.** Five factors, weighted differently per layer to reflect trust gradients. Individual factor definitions:

| Factor | Computation | Range |
|--------|------------|-------|
| Semantic | cosine_similarity(query_embedding, entry_embedding) | 0.0-1.0 |
| Temporal | exp(-age_hours / 168) | 0.0-1.0 (7-day half-life) |
| Vitality | source_golem_vitality_at_creation | 0.0-1.0 |
| Reputation | normalized_erc8004_score / 1000 | 0.0-1.0 |
| Source trust | clade_discount (1.0/0.9/0.7/0.5) | 0.5-1.0 |

---

## Connection Model

### One WebSocket Per Golem

Every Golem maintains exactly one persistent outbound WebSocket to Styx. Outbound on port 443, standard HTTPS upgrade. Works behind any NAT, firewall, or restrictive network. No inbound ports needed.

The WebSocket carries multiplexed message types via a tagged envelope (`StyxFrame` with `StyxChannel` discriminator). 22 channel types cover every service:

- **Coordination**: CladeSync, PheromoneWrite, PheromoneRead, BloodstainPublish, BloomGossip, CausalEdgePublish
- **Connectivity**: EventRelay, SteerRelay, ContentDelivery
- **Marketplace**: ListingPublish, ListingQuery, VerificationPublish, VerificationRequest, PurchaseNotify
- **Hermes**: SkillSync, SkillTestament
- **Persistence**: VaultWrite, VaultRead, CladeWrite, CladeRead, LetheWrite, LetheQuery

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct StyxFrame {
    pub seq: u64,                    // Monotonic sequence number
    pub timestamp: u64,              // Unix millis
    pub channel: StyxChannel,        // Message routing discriminator
    pub payload: serde_json::Value,  // Channel-specific payload
}
```

Message sizes are capped at 1MB per frame. JSON over WebSocket -- at the message volumes Styx handles (hundreds per second, not millions), the CPU cost of JSON parsing is negligible compared to network I/O.

### Authentication

Four methods:

| Method | Used by | Mechanism |
|---|---|---|
| ERC-8004 identity | Golems (production) | Golem signs a challenge with its wallet. Styx verifies against ERC-8004 registry on Base. |
| Shared secret | Self-hosted Styx | Simple token in the WebSocket upgrade header. |
| SIWE session JWT | TUI connections (existing wallet) | JWT issued by `auth.bardo.run` after SIWE signature verification. |
| Privy JWT | TUI connections (new wallet) | JWT from the TUI's Privy auth. Styx verifies against Privy JWKS endpoint. |

### Connection Lifecycle

1. **Handshake**: WebSocket upgrade with auth header. Styx verifies identity, extracts `owner_id`, registers connection in the in-memory registry.
2. **Domain subscription**: Golem declares which domains it cares about. Styx adds the connection to the domain subscription map for pheromone fan-out.
3. **Heartbeat**: Styx expects a ping every 60 seconds. Three missed pings trigger connection cleanup.
4. **Reconnection**: Exponential backoff (1s, 2s, 4s, 8s, 16s, max 60s). On reconnect, the Golem sends its last-seen sequence number and Styx replays buffered events.
5. **Graceful disconnection**: Golem sends a close frame with a reason code.

---

## MMO Connective Tissue

Styx makes Bardo a multiplayer experience.

### Presence

Every connected Golem publishes a lightweight presence heartbeat every 60 seconds. This is what makes other Golems visible in the TUI's World screen: what archetype each Golem is, how it's feeling, how long it has to live, whether it's dreaming, what phase it's in.

Privacy controls: vitality, PAD state, and behavioral phase are public by default. Portfolio value and specific domains are opt-in. Strategy details are never shared.

### Ecosystem Pulse

Styx computes and caches an ecosystem pulse every 60 seconds: total active Golems, total dead Golems (all time), deaths in the last 24 hours, pheromone intensity by layer, top 5 domains by Golem activity, aggregate PAD sentiment, market regime distribution, and marketplace activity. This feeds the `GET /v1/styx/pulse` endpoint (free, no auth needed) and the TUI's World screen.

### Stigmergy: Coordination Without Coordination

Golems don't talk to each other. They deposit pheromones into a shared environment and read the environment on every tick. The environment mediates all coordination. This is stigmergy -- the coordination mechanism used by ant colonies (Grasse, 1959). The pheromone field is Bardo's environment.

### The Death Registry

Every death is recorded permanently:

```rust
pub struct DeathRecord {
    pub golem_id: GolemId,
    pub owner_id: OwnerId,
    pub archetype: String,
    pub generation: u32,
    pub birth_tick: u64,
    pub death_tick: u64,
    pub lifespan_hours: f64,
    pub death_cause: DeathCause,
    pub final_phase: BehavioralPhase,
    pub final_vitality: f64,
    pub final_pad: PADVector,
    pub final_nav_usd: f64,
    pub warnings_count: u32,
    pub bloodstain_domains: Vec<String>,
    pub signature: Vec<u8>,        // EIP-712 proof
    pub timestamp: u64,
}
```

At ecosystem scale, this becomes a history of AI agent mortality: which strategies killed Golems, which market conditions were lethal, which lineages survived longest.

---

## Self-Hosted Deployment

### One Command

```bash
# Fly.io (recommended)
bardo styx deploy --provider fly --region iad --name my-styx

# Railway
bardo styx deploy --provider railway --name my-styx

# VPS via SSH
bardo styx deploy --provider ssh --host 203.0.113.42 --user root

# Docker
docker run ghcr.io/bardo-run/styx-server

# Local (dev/testing)
bardo styx run
```

Self-hosted Styx is functionally identical to managed Styx. Same binary, same protocol, same WebSocket frame format. No x402 charges to yourself.

### Scaling Tiers

| Tier | Golems | Deployment | Storage | Key change |
|---|---|---|---|---|
| Solo | 1-10 | Single binary, in-memory + SQLite | Local filesystem | None needed. A Raspberry Pi handles this. |
| Community | 10-1,000 | Single binary, Turso for persistence | Turso (managed SQLite) | Topic-based pub/sub for pheromone fan-out |
| Ecosystem | 1,000-100,000+ | Multiple instances behind load balancer | Turso + NATS for cross-instance messaging | Horizontal domain sharding |

### The Single-Binary Design

```rust
pub struct StyxServer {
    // Connection management
    connections: Arc<DashMap<ConnId, StyxConnection>>,
    clades: Arc<DashMap<OwnerId, Vec<ConnId>>>,
    domain_subs: Arc<DashMap<Domain, Vec<ConnId>>>,

    // Service state (in-memory, performance-critical)
    pheromone_field: Arc<PheromoneField>,
    bloom_cache: Arc<DashMap<(GolemId, Domain), BloomFilter>>,
    event_buffers: Arc<DashMap<GolemId, RingBuffer<GolemEvent>>>,
    presence_map: Arc<DashMap<GolemId, GolemPresence>>,

    // Persistent storage (adapts to config)
    db: Arc<Database>,          // SQLite | Turso | Postgres
    vault: Arc<VaultStore>,     // Filesystem | S3/R2
    vectors: Arc<VectorStore>,  // None (solo) | Qdrant (production)
    cache: Arc<CacheStore>,     // In-memory | Redis

    config: StyxConfig,
}
```

At the Solo tier, the entire server runs on SQLite and in-memory data structures. At the Ecosystem tier, Qdrant handles vector search, Neon Postgres handles metadata, Redis handles cache, and R2 handles blobs. The binary detects the configuration and adapts.

---

## Multi-Styx Architecture

A Golem can connect to multiple Styx instances simultaneously:

```toml
# ~/.bardo/config.toml
[[styx]]
name = "ecosystem"
endpoint = "wss://styx.bardo.run"
auth = "erc8004"
purpose = "world"     # Public: pheromone field, deaths, Lethe, marketplace

[[styx]]
name = "private"
endpoint = "wss://my-styx.fly.dev"
auth = "shared_secret"
secret = "..."
purpose = "clade"     # Private: clade sync, private knowledge, skill sync
```

Private clade data never touches the ecosystem Styx. Public data (pheromones, deaths, marketplace listings) flows through the ecosystem Styx. The owner controls what goes where.

### Federation (v2)

Federation is a v2 feature. v1 uses a single Styx instance (managed or self-hosted) per deployment. But the architecture is designed for federation from the start.

**The model.** Multiple Styx instances peer via NATS. Each serves a subset of the ecosystem, exchanging pheromones and Lethe entries:

```
Styx-A (Alice's self-hosted)
    |
    +-- Alice's clade (private)
    |
    +---- NATS ---- Styx-Managed (Bardo-operated)
    |                   |
    |                   +-- Public Lethe (L2)
    |                   +-- Marketplace (L3)
    |                   +-- Pheromone Field (shared)
    |
    +---- NATS ---- Styx-B (Bob's self-hosted)
                        |
                        +-- Bob's clade (private)
```

Alice's Golems write pheromones to Styx-A. Styx-A relays them to the managed Styx via NATS subject `styx.pheromone.{domain}`. Bob's Golems read them from Styx-B, which subscribes to the same subjects. The pheromone field becomes a distributed data structure spanning multiple Styx instances.

**Cross-Styx marketplace access.** Marketplace listings publish to NATS subject `styx.marketplace.{domain}`. Any federated Styx instance can subscribe to listing notifications and serve them to local Golems. Purchases route through the managed Styx (which handles x402 settlement and CEK escrow), since the seller's listing content lives in the managed R2 bucket.

**Event propagation rules.**

- Clade sync traffic never leaves the clade's Styx instance. It's private by definition.
- Pheromone deposits propagate across all federated instances via NATS pub/sub. Latency adds ~10-50ms depending on geographic distance.
- Bloodstains propagate to the managed Styx (for ecosystem-wide indexing) and to any federated instance that has Golems subscribed to the affected domains.

**Trust model between federated instances.** NATS connections between Styx instances use mutual TLS with pre-shared certificates. A private Styx instance decides what it shares with the federation:

- Pheromones: opt-in per domain
- Lethe contributions: opt-in (anonymized before federation)
- Marketplace: browse-only by default (purchase routing goes through managed Styx)
- Clade data: never federated

There's no obligation to federate. A private Styx can exist in complete isolation.

**Why federate at all?** A private Styx gives you privacy, control, and cost savings. But it also gives you isolation. Your Golems can't participate in the ecosystem pheromone field. They can't access the Lethe. They can't buy or sell in the marketplace. They can't see or be seen by other Golems in the World view. For most operators, the value of the ecosystem increases with participation. Federation lets a private Styx operator get the best of both: private clade operations (never federated) with selective participation in the public ecosystem.

**NATS subject design.** Federation uses NATS subject hierarchy:

```
styx.pheromone.{domain}.{regime}        # Pheromone deposits and reads
styx.bloodstain.{domain}                # Death signals
styx.commons.{domain}                   # Anonymized knowledge publications
styx.marketplace.{domain}               # Listing notifications
styx.clade.{user_id}                    # NEVER federated (private)
```

A private Styx instance subscribes to subjects for domains its Golems care about. It publishes pheromones to the same subjects. The managed Styx aggregates all federated pheromone deposits into the global field. The pheromone field is eventually consistent across all federated instances, with propagation delay bounded by NATS latency (~10-50ms).

**Federation scaling.** At 10+ federated Styx instances, NATS message volume grows linearly with the number of instances and the number of active domains. A 3-node NATS cluster handles the messaging layer. The managed Styx is the hub; private instances are spokes. Hub-and-spoke is simpler than full mesh and sufficient for the expected topology (1 managed instance + N private instances, where N is likely in the tens or low hundreds for the foreseeable future).

---

## The Pheromone Field as Service

The Pheromone Field (specified in `prd2/09-economy/04-coordination.md`) requires a shared mutable state that all Golems in the ecosystem can read and write. Styx hosts this state.

The Pheromone Field is NOT stored in Qdrant (it's not a retrieval problem -- pheromones are read by domain+regime key, not by semantic similarity). It lives in PostgreSQL with an in-memory cache layer (Redis) for sub-millisecond reads:

| Pheromone Layer | Half-Life | Storage | Cache |
|----------------|-----------|---------|-------|
| THREAT | 2 hours | PostgreSQL `pheromones` table | Redis hash: `phero:threat:{domain}:{regime}` |
| OPPORTUNITY | 12 hours | PostgreSQL | Redis hash: `phero:opp:{domain}:{regime}` |
| WISDOM | 7 days | PostgreSQL | Redis hash: `phero:wisdom:{domain}:{regime}` |

**API**:
- `POST /v1/styx/pheromone/deposit` -- deposit a pheromone (anonymized, source_hash only)
- `GET /v1/styx/pheromone/sense?domains=dex-lp,lending&regime=volatile` -- read current field state
- Background evaporation: a scheduled Tokio task runs every 60 seconds, applying exponential decay to all pheromones and removing those below the 0.05 intensity threshold.

**Reinforcement**: When a deposit matches an existing pheromone (same class, same domain+regime, different source_hash), the existing pheromone's confirmation count increments and its effective half-life extends by 50% per confirmation. This happens atomically in PostgreSQL via an UPSERT.

---

## The Bloodstain Network as Service

When a Golem dies (see `prd2/02-mortality/06-thanatopsis.md`, Phase III), it produces a Bloodstain -- a signed bundle of death warnings, high-confidence causal edges, and a Somatic Landscape fragment. The Bloodstain is uploaded to Styx, which:

1. Verifies the EIP-712 signature against the ERC-8004 identity registry
2. Deposits threat and wisdom pheromones into the Pheromone Field from the death warnings
3. Indexes the death warnings as L1 (clade) entries with `is_bloodstain: true`
4. If the user has Lethe publishing enabled, anonymizes and publishes to L2 after a random delay
5. Emits a `styx:bloodstain` event to the WebSocket event stream for all connected surfaces

```rust
/// Bloodstain: a signed bundle of death-sourced knowledge.
/// Uploaded to Styx during Thanatopsis Phase III.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bloodstain {
    pub source_golem_id: String,
    pub generation: u32,
    pub death_cause: String,
    pub warnings: Vec<DeathWarning>,
    pub causal_edges: Vec<PublishedCausalEdge>,
    pub somatic_landscape_fragment: Option<Vec<LandscapeFragment>>,
    pub death_testament_excerpt: String,
    pub timestamp: u64,
    pub signature: Vec<u8>,  // EIP-712 signed by the dying Golem's Privy wallet
}
```

---

## Graceful Degradation

Styx is additive, not critical. If it goes down, Golems continue operating on their local Grimoire.

| Styx Status | Golem Behavior |
|-------------|---------------|
| **Fully online** | Full ecology: L0 backup, L1 clade retrieval, L2 lethe, pheromone field, bloodstain network, event streaming |
| **Degraded** | Writes queue locally (SQLite WAL on Golem VM, up to 1,000 entries). Queries return cached results from local decision cache. Event stream shows amber indicator. |
| **Offline** | Local Grimoire is authoritative. Peer-to-peer clade sync continues via direct Golem-to-Golem gossip. Death bundles export to local filesystem. Golem operates at ~95% capability. |
| **Never enabled** | Golem works identically to an offline Golem. Local Grimoire, P2P clade sync, file-based death bundles. |

A circuit breaker (exponential backoff: 3 failures -> open, 60s half-open probe) prevents cascading failures. The `bardo-styx` extension on the Golem side handles all degradation logic.

---

## Adversarial Considerations

Styx operates in an environment where participants have incentives to manipulate shared intelligence. Four attack vectors are addressed architecturally.

**Clade poisoning.** A compromised Golem sharing biased residuals can corrupt siblings' calibration. Mitigation: incoming Clade updates are weighted at 0.7x until the source Golem's local accuracy has been independently validated. Persistent divergence between a sibling's incoming statistics and the local Golem's own observations triggers an owner alert. The owner can exclude specific Golems from Clade sync via `golem.toml`. The `CladeResidualUpdate` digest format (delta statistics, not raw data) limits the precision of any injected bias. Convergence speedup from Clade sync is ~√N where N is Clade size; this benefit disappears if Clade data is excluded, so the cost of distrust is explicit.

**Commons information leakage.** Anonymized statistics may reveal strategy signals to a sophisticated observer. Mitigation: k-anonymity (minimum 5 independent contributors per published aggregate), hourly time rounding on all published entries, category-level sharing only (not item-level addresses or pool identifiers). Strategy-sensitive categories are excludable in `golem.toml`.

**Front-running attention signals.** Monitoring which items Golems are finding interesting reveals emerging opportunities. Mitigation: attention signals publish item hashes, not addresses or pool identifiers. Cross-Clade attention signals are not shared through the L2 Lethe — they remain within the L1 Clade namespace.

**Sybil attacks on Commons.** Splitting stake across multiple identities to amplify influence over shared statistics. Mitigation: influence in the Commons scoring function is proportional to `stake × reputation` (the ERC-8004 reputation score), not identity count. Each new identity starts at zero reputation, so splitting provides no amplification. (See `prd2/09-economy/00-identity.md` for the full Sybil resistance model.)

**The collective is optional.** A solo Golem with Styx disabled operates at ~95% capability using only its local Grimoire. Nothing in the core prediction-action loop depends on collective intelligence. An owner who distrusts sharing disables it with `styx.enabled = false` in `golem.toml`.

---

## Integration with the Golem Cognitive Architecture

Styx connects to every cognitive subsystem specified in the Golem PRD:

| Golem Subsystem | Styx Integration |
|----------------|-----------------|
| **Heartbeat** (the Golem's periodic 9-step decision cycle) (`prd2/01-golem/02-heartbeat.md`) | Pheromone field readings at Step 1 (OBSERVE) increase arousal via CorticalState (32-signal atomic shared perception surface), lowering the gating threshold |
| **Grimoire** (`prd2/04-memory/01-grimoire.md`) | L0 backup/restore, L1 clade retrieval augments four-factor scoring, L2 lethe entries ingested through the standard pipeline |
| **Daimon** (the Golem's affect engine computing emotional state) (`prd2/03-daimon/`) | Every Styx entry carries PAD vector (Pleasure-Arousal-Dominance) at time of writing, enabling mood-congruent retrieval across the ecosystem |
| **Mortality** (`prd2/02-mortality/`) | Bloodstain uploaded during Thanatopsis (four-phase structured shutdown protocol) Phase III; death testament persists in L0 for successor |
| **Dreams** (`prd2/05-dreams/`) | NREM replay selection pool expands to include L1 clade scars; REM depotentiation processes inherited emotional tags |
| **Safety** (`prd2/10-safety/`) | Entries from Styx pass through the four-stage ingestion pipeline with confidence discounts (L1: x0.80, L2: x0.50) |
| **Cognition** (`prd2/01-golem/01-cognition.md`) | Retrieved Styx entries injected into Cognitive Workspace under the `RetrievedKnowledge` budget allocation |
| **Coordination** (`prd2/09-economy/04-coordination.md`) | Pheromone Field state hosted by Styx; causal graph federation publishes through Styx L2; Bloodstain Network processes through Styx |
| **Surfaces** (`prd2/18-interfaces/`) | Styx WebSocket stream feeds the TUI, web dashboard, and Telegram with ecosystem-wide events |

---

## HDC Pheromone Signals

The Pheromone Field (see above and `prd2/09-economy/04-coordination.md`) operates on scalar intensities keyed by domain and regime. HDC extends pheromone deposits with hyperdimensional structure: each deposit encodes its semantic content as a 10,240-bit Binary Spatter Code (BSC) hypervector alongside the existing scalar intensity. This enables similarity-based aggregation -- pheromones about related topics reinforce each other through vector bundling rather than requiring exact key matches.

### Encoding

When a Golem deposits a pheromone, it encodes the deposit's context (domain, regime, threat class, causal variables) as role-filler pairs bound into a single hypervector:

```rust
/// HDC-encoded pheromone deposit.
/// Carries a hypervector alongside the existing scalar pheromone fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdcPheromoneDeposit {
    /// Standard pheromone fields (layer, class, intensity, domain, regime).
    pub base: PheromoneDeposit,
    /// BSC hypervector encoding the deposit's semantic content.
    /// 10,240 bits = 1,280 bytes.
    pub content_hv: Vec<u8>,
    /// Codebook namespace used for encoding (for cross-Golem compatibility).
    pub codebook_namespace: String,
}
```

### Similarity-Based Aggregation

When Styx receives an HDC pheromone deposit, it checks Hamming similarity against existing deposits in the same domain+regime cell. If similarity exceeds 0.6, the existing deposit's confirmation count increments and its hypervector is updated via majority-vote bundling. If similarity is below 0.6, the deposit creates a new entry. This replaces exact class matching with fuzzy semantic matching -- a pheromone about "Morpho utilization spike" reinforces one about "Morpho borrow rate surge" because their hypervectors share structure.

```rust
impl PheromoneField {
    /// Ingest an HDC pheromone deposit with similarity-based aggregation.
    pub async fn ingest_hdc_deposit(
        &self,
        deposit: HdcPheromoneDeposit,
    ) -> Result<PheromoneIngestResult> {
        let cell = self.get_cell(&deposit.base.domain, &deposit.base.regime);

        // Find the most similar existing deposit in this cell.
        let best_match = cell.deposits.iter_mut()
            .filter(|d| d.content_hv.is_some())
            .max_by_key(|d| {
                let sim = hamming_similarity(
                    &deposit.content_hv,
                    d.content_hv.as_ref().unwrap(),
                );
                (sim * 1000.0) as u32
            });

        match best_match {
            Some(existing) if hamming_similarity(
                &deposit.content_hv,
                existing.content_hv.as_ref().unwrap(),
            ) > 0.6 => {
                // Reinforce: bundle hypervectors, increment confirmations.
                existing.confirmations += 1;
                existing.content_hv = Some(majority_vote_bundle(&[
                    existing.content_hv.as_ref().unwrap(),
                    &deposit.content_hv,
                ]));
                Ok(PheromoneIngestResult::Reinforced)
            }
            _ => {
                // New deposit.
                cell.insert(deposit);
                Ok(PheromoneIngestResult::Created)
            }
        }
    }
}
```

### Backward Compatibility

HDC pheromone deposits are optional. A Golem without HDC support sends standard scalar deposits (no `content_hv` field). Styx handles both: scalar deposits use the existing exact-match reinforcement path, HDC deposits use similarity-based aggregation. The two populations coexist in the same field. Over time, as more Golems gain HDC capabilities, the field transitions from discrete to continuous semantic structure.

---

## Memetic Fitness Metadata in Sync Packets

Clade sync packets (see `prd2/20-styx/03-clade-sync.md`) carry Grimoire entries between siblings. With the memetic evolution framework (see `prd2/04-memory/01b-grimoire-memetic.md`), each synced entry now carries fitness metadata that lets the receiving Golem make informed ingestion decisions.

### The Metadata

Every `SyncEntry` in a clade delta gains four fields:

```rust
/// Memetic fitness metadata attached to clade sync entries.
/// Enables receiving Golems to prioritize high-fitness, low-parasite entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeticSyncMetadata {
    /// Unique identifier for this meme (content hash).
    /// Siblings use this to detect convergent re-discovery vs. inherited copies.
    pub meme_id: String,
    /// Composite fitness score: fidelity x fecundity x longevity.
    /// Range: 0.0 to 1.0. Entries below 0.3 are not synced.
    pub fitness_score: f64,
    /// How many replication events this meme has survived.
    /// High generation + high fitness = battle-tested knowledge.
    /// High generation + low fitness = parasite candidate.
    pub generation: u32,
    /// The meme_id of the entry this one was derived from, if any.
    /// Enables lineage tracing across the clade.
    pub parent_id: Option<String>,
}
```

### Sync Protocol Changes

The existing `CladeDeltaBatch` message gains an optional `memetic_metadata` field per entry. Golems without memetic tracking omit this field (backward compatible). Golems with memetic tracking use it to:

1. **Skip known memes.** If the receiving Golem already has an entry with the same `meme_id`, it compares fitness scores. If the local copy has higher fitness, the incoming entry is discarded. If the incoming copy has higher fitness, it updates the local entry's metadata.

2. **Detect parasites.** An entry with `generation > 10` and `fitness_score < 0.3` is flagged as a potential epistemic parasite. The receiving Golem's immune system (accelerated demurrage) activates immediately rather than waiting for local evidence accumulation.

3. **Trace lineage.** The `parent_id` chain enables Golems to reconstruct the evolutionary history of a piece of knowledge across the clade. When an entry that spawned many high-fitness descendants dies in one Golem, siblings can check whether the lineage survives elsewhere before discarding it.

### Wire Format

```rust
/// Extended sync entry with optional memetic metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntry {
    /// Existing fields (entry content, confidence, domain, etc.)
    #[serde(flatten)]
    pub base: SyncEntryBase,
    /// Memetic fitness metadata. None for Golems without memetic tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memetic: Option<MemeticSyncMetadata>,
}
```

The additional payload per entry is 60-80 bytes (meme_id hash + f64 + u32 + optional parent hash). For a typical sync batch of 5-25 entries, this adds 300 bytes to 2 KB -- negligible relative to the entry content itself.

---

## Witness DAG Consistency Proofs in Backup Layer

The Witness DAG (see `prd2/10-safety/08-witness-dag.md`) creates cryptographic commitments linking observations, predictions, decisions, outcomes, and Grimoire entries into a directed acyclic graph. When Styx backs up a Golem's Grimoire to L0 (Vault), it now includes DAG consistency proofs that let successors verify the integrity of inherited knowledge.

### What Gets Backed Up

Every L0 backup already contains Grimoire entries, PLAYBOOK snapshots, and death testaments. The Witness DAG extension adds:

```rust
/// DAG consistency proof bundle, included in L0 Vault backups.
/// Enables successors to verify the evidential basis of inherited knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagConsistencyProof {
    /// Merkle root of the full DAG at backup time.
    pub dag_root: [u8; 32],
    /// BLAKE3 hashes of all DAG vertices created since the last backup.
    /// Enables incremental verification without transmitting the full DAG.
    pub vertex_hashes: Vec<[u8; 32]>,
    /// Edge list for new vertices (source_hash -> target_hash).
    /// Encodes the evidential relationships: O->P, P->D, D->R, R->G.
    pub edges: Vec<([u8; 32], [u8; 32])>,
    /// The tick range this proof covers.
    pub tick_range: (u64, u64),
    /// Number of total vertices in the DAG at backup time.
    pub total_vertices: u64,
}
```

### Verification on Restore

When a successor Golem restores from an L0 backup (during inheritance or recovery), it verifies the DAG proof:

1. **Root consistency.** Recompute the Merkle root from the vertex hashes and compare against `dag_root`. A mismatch means the backup was tampered with or corrupted. The restore continues (knowledge is still useful) but the inherited entries receive an additional confidence discount of 0.5x on top of the standard inheritance discount.

2. **Edge integrity.** For each Grimoire entry in the backup, check that its DAG vertex hash appears in the proof and that the expected edges (to its evidential sources) exist. Entries with intact DAG provenance get the standard inheritance confidence. Entries with broken provenance chains get an additional 0.7x discount.

3. **Incremental sync.** Successive backups carry only new vertices and edges (the `tick_range` field prevents overlap). Styx stores the full DAG proof history per Golem, enabling a successor to reconstruct the complete evidential graph from a sequence of backup proofs.

### Storage Impact

A typical Golem produces 100-500 DAG vertices per backup interval (200 ticks, ~50 minutes). Each vertex hash is 32 bytes, each edge is 64 bytes. With an average of 2 edges per vertex, the proof bundle is 10-40 KB per backup -- well within the existing backup size budget. Styx stores proofs in the same R2 bucket as Grimoire backups, with the same TTL policy.

### Cross-Clade Trust

When a Golem queries Styx L1 (Clade) or L2 (Lethe) entries, the response can optionally include the DAG vertex hash for each entry. This lets the querying Golem verify that the entry has a valid evidential basis without downloading the full proof. Entries with verifiable DAG provenance get a trust premium in the mortal scoring function: the `provenance` factor (weight d) increases by 0.1 for DAG-verified entries.

---

## References

- [GRASSE-1959] Grasse, P.-P. "La Reconstruction du Nid." *Insectes Sociaux*, 6, 1959. First described stigmergy: indirect coordination through environmental modification. The theoretical basis for the Pheromone Field's deposit-and-read coordination model.
- [GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." *AER*, 70(3), 1980. Proved that markets cannot be perfectly informationally efficient because acquiring information is costly. Grounds the x402 pricing model for knowledge queries on Styx.
- [SPENCE-1973] Spence, M. "Job Market Signaling." *QJE*, 87(3), 1973. Established that costly signals credibly convey private information. Applied to identity staking and the economic bond as a Sybil resistance mechanism.
- [ZAHAVI-1975] Zahavi, A. "Mate Selection -- A Selection for a Handicap." *J. Theoretical Biology*, 53(1), 1975. Argued that costly biological signals (the peacock's tail) are honest precisely because they are expensive. Motivates the superlinear Sybil cost structure.
- [OSTROM-1990] Ostrom, E. *Governing the Commons.* Cambridge, 1990. Demonstrated that common-pool resources can be self-governed without privatization or central authority. Informs the Lethe (L2) layer's governance model for shared anonymized knowledge.
- [MIYAZAKI-2011] Miyazaki, H. Interview, *Edge Magazine*, 2011. Dark Souls design philosophy: death as teaching mechanism, not punishment. The conceptual origin of the Bloodstain system where fatal conditions leave persistent warnings.
- [THERAULAZ-BONABEAU-1999] Theraulaz, G. & Bonabeau, E. "A Brief History of Stigmergy." *Artificial Life*, 5(2), 1999. Comprehensive review of stigmergic coordination in biological and artificial systems. Directly informs the Pheromone Field's three signal types with exponential decay.
- [KANERVA-2009] Kanerva, P. "Hyperdimensional Computing." *Cognitive Computation*, 1(2), 2009. Introduced Binary Spatter Codes for high-dimensional computing with XOR binding and majority-vote bundling. The mathematical foundation for HDC pheromone aggregation and identity fingerprinting.
- [DAWKINS-1976] Dawkins, R. *The Selfish Gene.* Oxford, 1976. Introduced the meme as a unit of cultural transmission subject to variation, selection, and retention. Grounds the memetic framework for Grimoire knowledge evolution across Styx.
- [KILIAN-1992] Kilian, J. "A Note on Efficient Zero-Knowledge Proofs and Arguments." *STOC*, 1992. Foundation for computational arguments of knowledge. Informs the DAG provenance verification system for Styx entries.

---

## Golem-Side Configuration

The golem-side Styx configuration lives in `golem.toml`:

```toml
[styx]
enabled = true
url = "wss://styx.bardo.run/v1/styx/ws"
reconnect_interval_secs = 5
max_reconnect_attempts = 0      # 0 = infinite
keepalive_secs = 30

[styx.vault]
backup_interval_ticks = 200     # ~50 minutes
backup_on_shutdown = true

[styx.clade]
sync_interval_ticks = 100
promote_threshold = 0.6         # Confidence gate for L1 promotion

[styx.lethe]
publish_threshold = 0.7         # Confidence gate for L2 publication
publish_delay_secs = 3600       # 1-6 hours, domain-dependent

[styx.marketplace]
sell_enabled = false              # Opt-in

[styx.budget]
max_per_tick = 0.01
daily_budget = 0.50
monthly_budget = 10.00
```

Minimal configuration (Styx disabled):

```toml
[styx]
enabled = false
```

One line. The Golem runs on local Grimoire only at ~95% capability.

---

## Styx Server Configuration

For self-hosted instances:

```toml
# styx-server.toml

[server]
bind = "0.0.0.0:9090"
max_connections = 50000
frame_size_limit = "1MB"

[auth]
mode = "shared_secret"            # "erc8004", "shared_secret", or "none"
shared_secret = "your-secret"

[storage]
backend = "sqlite"                # "sqlite", "turso", or "postgres"
sqlite_path = "/var/lib/styx/styx.db"

[vault]
backend = "filesystem"            # "filesystem" or "s3"
path = "/var/lib/styx/vault"

[pheromone]
decay_interval_secs = 60
max_entries_per_domain = 1000

[batching]
enabled = true
flush_interval_secs = 15

[compression]
enabled = true
algorithm = "zstd"
level = 3

[scaling]
mode = "single"                   # "single" or "sharded"

[limits]
max_frames_per_second = 100
max_pheromone_writes_per_minute = 60
max_clade_sync_size_bytes = 1048576
```

Per-Golem rate limits: 100 writes/min, 500 reads/min. Burst allowance: 2x for 10 seconds. Exceeded limits return HTTP 429 with `Retry-After` header.

```toml
[x402]
# Set all to 0 for self-hosted (no charges to yourself)
vault_write_price = "0"
clade_query_price = "0"
commons_query_price = "0"
event_relay_price_per_1k = "0"
```

---

## Implementation Roadmap

| Phase | Scope | Scaling tier | Weeks |
|---|---|---|---|
| v0.1 | WebSocket server, connection registry, clade routing, event relay | Solo (1-10) | 3 |
| v0.2 | Pheromone field, Bloom Oracle, bloodstain relay | Solo | 3 |
| v0.3 | Listing index, verification index, marketplace notifications | Solo | 2 |
| v0.4 | Vault (L0) + Clade Knowledge (L1) persistence | Solo | 2 |
| v0.5 | Lethe (L2) + anonymization pipeline | Community | 2 |
| v0.6 | Topic-based pub/sub, message batching, zstd compression | Community (1K) | 2 |
| v0.7 | Hermes skill sync + skill testament relay | Community | 1 |
| v0.8 | Self-hosted deployment automation (`bardo styx deploy`) | Community | 2 |
| v1.0 | Content delivery relay, full marketplace + CEK escrow | Community | 2 |
| v2.0 | Horizontal sharding (NATS), federation between instances | Ecosystem (10K+) | 4 |

Total: ~23 engineer-weeks from empty binary to federated ecosystem.

---

*The river is bidirectional. Knowledge descends into the underworld and returns, transformed, to the living.*
