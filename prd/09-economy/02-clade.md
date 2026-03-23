# Clade: Styx-Relayed Knowledge Sharing Between Siblings [SPEC]

> **Crate**: `golem-clade` | **Extension**: `bardo-clade`
>
> **Depends on**: [00-identity.md](00-identity.md) (ERC-8004 operator field), [04-coordination.md](04-coordination.md) (ERC-8001 intents), [../20-styx/00-architecture.md](../20-styx/00-architecture.md) (Styx three-layer model)

> **Reader orientation:** This document specifies Clade (sibling Golem) knowledge sharing for Bardo's mortal DeFi agents. It belongs to the **09-economy** layer. The key concept is Styx (Bardo's global knowledge relay)-relayed WebSocket sync: sibling Golems (mortal autonomous agents sharing a common owner) discover each other via ERC-8004 (on-chain agent identity) and exchange distilled Grimoire (local knowledge base) entries through the L1 Clade layer, with the local Grimoire remaining authoritative. For term definitions, see `prd2/shared/glossary.md`.

A Clade is a set of Golems sharing the same operator address. Siblings discover each other on-chain via `getAgentsByOperator()` and share knowledge through Styx's L1 (Clade) layer. The earlier design used direct P2P REST APIs between Golem VMs. This version replaces that transport with Styx-relayed WebSocket sync, keeping the same logical model: siblings share distilled knowledge, the ingestion pipeline validates everything, and the local Grimoire remains authoritative.

Grasse's observation on stigmergy -- indirect coordination through shared environment modification -- provides the conceptual frame [GRASSE-1959]. Styx L1 is the shared environment; entries are pheromone traces. The more agents share, the richer the collective Grimoire.

---

## 1. Design Philosophy

The previous P2P design required every Golem to expose HTTP endpoints and maintain direct connections to every sibling. This created three problems: NAT traversal and firewall issues on self-hosted deployments, O(n^2) connections in large clades, and no persistence if all siblings were offline simultaneously.

Styx L1 replaces direct connections with a relay model. Each Golem maintains a single outbound WSS connection to Styx. Knowledge flows through Styx's L1 namespace, which handles fan-out to online siblings and persistence for offline ones. The Golem never exposes a listening port for clade purposes.

The local Grimoire is still authoritative. Styx L1 is a relay and backup, not a source of truth. If Styx is unreachable, Golems continue operating on their local Grimoire -- they just stop receiving sibling updates until the connection recovers.

---

## 2. Three Layers and How Clade Fits

Styx has three privacy layers (see [../20-styx/00-architecture.md](../20-styx/00-architecture.md)):

| Layer | Scope | Access | Clade Role |
|-------|-------|--------|------------|
| **L0 -- Vault** | Single Golem | Owner's Golems only | Grimoire backup and successor inheritance |
| **L1 -- Clade** | Operator's fleet | All Golems sharing the same ERC-8004 operator | Sibling knowledge sync (this spec) |
| **L2 -- Lethe (formerly Lethe)** | Ecosystem | Any Verified+ agent (score >= 50) | Anonymized cross-operator intelligence |

A single `POST /v1/styx/entries` call handles L0 writes. Entries that meet L1 promotion gates are automatically dual-indexed to the clade namespace. A single `GET /v1/styx/query` retrieves results from all accessible layers, merged and ranked. The Golem never needs to make separate API calls per layer.

---

## 3. Discovery

An agent discovers its siblings on boot by querying the ERC-8004 identity registry:

```solidity
// Already deployed -- no new contract needed
function getAgentsByOperator(address operator) external view returns (address[] memory);
```

This returns wallet addresses of all agents sharing the same operator. Styx maintains a connection registry that maps wallet addresses to their WebSocket session state. The Golem does not need to resolve sibling hostnames -- Styx handles routing.

```rust
use alloy::primitives::Address;

pub struct SiblingInfo {
    pub address: Address,
    pub styx_connected: bool,
    pub domains: Vec<String>,
    pub last_seen: u64,
}

pub async fn discover_siblings(
    operator: Address,
    registry: &IdentityRegistry,
    styx: &StyxClient,
) -> Vec<SiblingInfo> {
    let addresses = registry.get_agents_by_operator(operator).await;
    let mut siblings = Vec::new();
    for addr in addresses {
        if addr == my_address { continue; }
        let status = styx.get_peer_status(addr).await;
        siblings.push(SiblingInfo {
            address: addr,
            styx_connected: status.map_or(false, |s| s.connected),
            domains: status.map_or_else(Vec::new, |s| s.domains),
            last_seen: status.map_or(0, |s| s.last_seen),
        });
    }
    siblings
}
```

---

## 4. Sync Mechanism

Three sync triggers replace the old push/pull REST model:

### 4.1 Event-Driven Push (Primary)

When a Golem produces an entry meeting the promotion gate (Section 5), it writes to Styx L0. Styx automatically dual-indexes qualifying entries to L1. All online siblings receive a WebSocket notification containing the entry metadata. The sibling's `bardo-clade` extension fetches the full entry and ingests it through the standard pipeline with `clade` provenance and initial confidence 0.4.

```
Golem A produces insight (confidence >= 0.6)
    |
    v
POST /v1/styx/entries { layer: "vault", ... }
    |
    v
Styx: store in L0, check promotion gate -> dual-index to L1
    |
    v
Styx: fan-out WebSocket notification to online siblings
    |
    v
Golem B receives notification -> fetches full entry -> ingests via pipeline
```

### 4.2 Boot Catch-Up (On Connect)

When a Golem connects to Styx (boot or reconnect), it queries L1 for entries since its last sync timestamp:

```
Golem B boots, connects WSS to Styx
    |
    v
GET /v1/styx/query { layer: "clade", since: last_sync_ts, limit: 1000 }
    |
    v
Receive entries -> ingest via standard pipeline
    |
    v
Update last_sync_ts
```

Each Golem tracks `last_sync_ts` in its local `/data/state.json`. On first boot with no history, `since=0` pulls everything available in the L1 namespace.

### 4.3 Periodic Reconciliation

Every 100 ticks (~100 minutes), the `bardo-clade` extension runs a lightweight reconciliation: query L1 entry count and compare against local clade entry count. If a gap exists (e.g., missed WebSocket notification), fetch the missing entries. This catches edge cases where a notification was lost during a brief network blip.

### 4.4 Typical Sync Volumes

For a 5-Golem clade with mixed strategies (LP, DCA, lending, momentum, research):

| Metric | Typical Value |
|--------|---------------|
| Entries promoted to L1 per Golem per day | 15-40 |
| Total L1 entries across clade per day | 75-200 |
| Average entry size | 2-5 KB |
| Daily L1 bandwidth per Golem | 0.5-1.5 MB |
| WebSocket notifications per Golem per day | 60-160 |

### 4.5 Version Vectors

Each Golem tracks a per-sibling sequence number in its local state. When an entry arrives via L1, the Golem checks whether it has already seen that `entry_id` (format: `<golemAddress>:<localSequenceNumber>`). Deduplication is a single index lookup. No CRDTs or merge functions needed -- entries are append-only and the ingestion pipeline handles contradictions.

---

## 5. What Gets Shared

Not everything in a Golem's local Grimoire crosses the wire. Sharing is selective, controlled by promotion gates at the Styx server.

### 5.1 L1 Promotion Gates

| Entry Type | Promote to L1 If | Rationale |
|---|---|---|
| **insight** | confidence >= 0.6 AND status = `active` | Low-confidence insights are noise |
| **heuristic** | confidence >= 0.7 AND validated >= 10 ticks | Heuristics need local validation first |
| **warning** | Always (confidence >= 0.4) | Risk warnings propagate immediately |
| **causal_link** | confidence >= 0.5 AND has supporting episodes | Causal claims need evidence |
| **strategy_fragment** | confidence >= 0.6 | Partial strategies need reasonable confidence |

Observations and raw episodes are not promoted by default -- too voluminous and too agent-specific. Exception: regime shift episodes always promote because every sibling benefits from knowing the market state changed.

### 5.2 Query Filtering

When querying L1, a Golem can filter by domain:

```
GET /v1/styx/query?layer=clade&since=1709856000&domains=lp_management,gas_timing
```

An LP manager does not need a lending specialist's insights about Morpho utilization rates. But by default, no filter is applied -- siblings get everything.

---

## 6. Persistence and Recovery

### 6.1 Normal Case: Styx Persists

Unlike the old P2P model, L1 entries persist in Styx independently of sibling uptime. When Agent A dies, its previously shared knowledge remains in L1 for Agents B and C to query. No overlapping lifetimes required.

The death preparation protocol still pushes `clade:agent_dying` as a warning entry to L1, triggering siblings to run a catch-up sync.

### 6.2 Total Clade Wipe

When ALL of a user's Golems die and a new one boots, the L1 namespace still contains all previously shared knowledge. The new Golem's boot catch-up (Section 4.2) pulls it automatically. This is a major improvement over the old P2P design where a total wipe lost everything.

L0 (Vault layer) also contains full Grimoire backups from each dead Golem, available for successor inheritance.

### 6.3 Styx Offline

If Styx is unreachable, the Golem queues writes locally (SQLite WAL, up to 1,000 entries). When Styx reconnects, the queue drains. During the outage, no sibling sync occurs, but the Golem continues operating on its local Grimoire at ~95% capability.

---

## 7. Pheromone Field

The Pheromone Field (specified in [04-coordination.md](04-coordination.md)) is hosted by Styx and accessible to the entire ecosystem, not just the clade. Three pheromone types with different half-lives:

| Pheromone | Half-Life | Signal Type |
|-----------|-----------|-------------|
| **THREAT** | 2 hours | Risk warnings, exploit alerts, rug patterns |
| **OPPORTUNITY** | 12 hours | Yield spikes, arbitrage openings, pool launches |
| **WISDOM** | 7 days | Regime insights, structural observations, causal patterns |

A Golem deposits pheromones via `POST /v1/styx/pheromone/deposit` (anonymized, carries only a `source_hash`). It senses the field via `GET /v1/styx/pheromone/sense?domains=dex-lp,lending&regime=volatile`. Pheromone readings at the OBSERVE step of the heartbeat increase arousal via CorticalState, lowering the gating threshold for action.

Reinforcement: when multiple independent Golems deposit matching pheromones (same class, same domain+regime, different `source_hash`), the confirmation count increments and the effective half-life extends by 50% per confirmation. Consensus emerges from convergent independent observations.

---

## 8. Bloodstain Network

When a Golem dies (see `prd2/02-mortality/06-thanatopsis.md`, Phase III), it produces a Bloodstain -- a signed bundle of death warnings, high-confidence causal edges, and a Somatic Landscape fragment. The Bloodstain uploads to Styx, which:

1. Verifies the EIP-712 signature against the ERC-8004 identity registry
2. Deposits THREAT and WISDOM pheromones from the death warnings
3. Indexes the death warnings as L1 (clade) entries with `is_bloodstain: true`
4. If the user has Lethe publishing enabled, anonymizes and publishes to L2 after a random delay (1-6h)
5. Emits a `styx:bloodstain` event on the WebSocket stream for all connected surfaces

Dead Golems make living siblings smarter. Every death is a data point.

---

## 9. Cross-Domain Intelligence

The core value proposition: the more agents you run, the smarter all of them get.

When agents in a Clade run different strategies, their distilled knowledge cross-pollinates through L1 sync. An LP manager observes that pool fees spike when gas drops below 10 gwei, pushes it as an insight with domain tag `gas_timing`. A DCA agent receives this via L1, its ingestion pipeline validates it, its curator considers adding "check gas before executing" to the PLAYBOOK. A lending optimizer observes that Morpho utilization spikes correlate with pool fee spikes, pushes a causal_link connecting the two domains.

This happens automatically. No agent needs to explicitly request knowledge from another domain.

### 9.1 Transactive Memory

The Styx connection registry tracks each sibling's domain specializations and insight counts. The `bardo-clade` extension maintains a local cache:

```rust
pub struct SiblingProfile {
    pub address: Address,
    pub domains: Vec<String>,
    pub insight_count: u32,
    pub last_updated: u64,
    pub connected: bool,
}
```

During deliberation, if the agent needs domain-specific knowledge it does not have locally, it queries L1 filtered to a specific sibling's entries. This is transactive memory in the sense of Wu et al. [WU-2025] -- knowing who knows what.

---

## 10. Rate Limits and Bandwidth

Clade sync must not interfere with the agent's primary job.

| Operation | Budget | Enforcement |
|-----------|--------|-------------|
| L1 writes (via L0 promotion) | Max 100 entries/hour per Golem | Styx server rate limiter |
| Boot catch-up | Max 1,000 entries per sync | Paginated via `limit` |
| WebSocket notifications | Max 10/minute per source | Styx drops excess, logs warning |
| Reconciliation poll | Every 100 ticks (~100 minutes) | Timer in extension |
| Total bandwidth | <5 MB/day steady state | Entries are 2-5 KB each |

All Styx network operations are non-blocking. If the WebSocket connection drops, the extension reconnects with exponential backoff (1s, 2s, 4s, ... capped at 60s). Queued writes drain on reconnect.

---

## 11. Cost Model

Styx L1 is included with L0 writes -- no additional charge for clade sync. The cost to the operator comes from L0 writes that get dual-indexed.

| Styx Operation | Price |
|----------------|-------|
| L0 write (includes L1 dual-index) | $0.001 per entry |
| L1 query (clade retrieval) | $0.002 per query |
| Pheromone deposit | $0.0005 per deposit |
| Bloodstain upload | $0.005 per bundle |

**Estimated monthly cost for a 5-Golem clade** (typical mixed-strategy fleet):

| Component | Calculation | Monthly Cost |
|-----------|-------------|--------------|
| L0 writes | 5 Golems x 30 entries/day x 30 days x $0.001 | $4.50 |
| L1 queries | 5 Golems x 10 queries/day x 30 days x $0.002 | $3.00 |
| Pheromone deposits | 5 Golems x 5 deposits/day x 30 days x $0.0005 | $0.375 |
| Bloodstains | ~0.5 deaths/month x $0.005 | $0.003 |
| **Total** | | **~$7.88/month** |

For a solo Golem (no siblings), the clade layer is effectively free -- entries are written to L0 regardless, and L1 dual-indexing has no queries if nobody else is reading.

---

## 12. Failure Modes

| Failure | Impact | Recovery |
|---------|--------|----------|
| Styx unreachable | No clade sharing, local Grimoire only | Automatic: reconnect with backoff, drain write queue |
| Styx degraded (high latency) | Sync delays, but no data loss | Automatic: entries queue, reconciliation catches gaps |
| Sibling offline | Does not receive real-time notifications | Automatic: boot catch-up on reconnect |
| All siblings offline | No clade sharing, local Grimoire only | Automatic: L1 entries persist in Styx, available on boot |
| Sibling pushes garbage | Bad entries arrive via L1 | Handled: ingestion pipeline validates, rejects bad entries |
| Total clade wipe | No siblings online, all dead | Automatic: L1 persists, successor pulls on boot |

Every failure degrades gracefully to "Golem operates on its own local Grimoire." No failure causes data loss or incorrect behavior.

---

> **Extended:** Federated Clades (Section 13) -- see [../../prd2-extended/09-economy/02-clade-extended.md](../../prd2-extended/09-economy/02-clade-extended.md)

---

## 13. Configuration

TOML config block in `golem.toml`:

```toml
[styx.clade]
enabled = true
push_confidence_threshold = 0.6
pull_domains = []  # empty = all domains
```

See [../shared/config-reference.md](../shared/config-reference.md) for the full `[styx]` configuration block.

Environment variable overrides:

```bash
BARDO_STYX_URL=wss://styx.bardo.money/v1/ws
BARDO_CLADE_ENABLED=true
BARDO_CLADE_PUSH_CONFIDENCE=0.6
BARDO_CLADE_PULL_DOMAINS=
```

Zero-config default: Clade sync is on, shares everything above 0.6 confidence, pulls from all domains. Works out of the box the moment a second Golem boots.

> **Extended:** Emotional Contagion (Section 14), Dream Knowledge Sharing (Section 15), Sibling Death Grief (Section 16), Clade Ecology (Section 17) -- see [../../prd2-extended/09-economy/02-clade-extended.md](../../prd2-extended/09-economy/02-clade-extended.md)

---

## Events Emitted

Clade events track peer discovery and knowledge propagation.

| Event | Trigger | Payload |
|---|---|---|
| `clade:broadcast` | Knowledge promoted to L1 | `{ entryType, hash, recipients }` |
| `clade:sync_received` | Knowledge received from L1 | `{ peerId, entryType, hash }` |
| `clade:peer_discovered` | New clade peer found via discovery | `{ peerId, agentId, capabilities }` |
| `clade:styx_connected` | WebSocket connection to Styx established | `{ styxUrl, latencyMs }` |
| `clade:styx_disconnected` | WebSocket connection lost | `{ reason, queuedEntries }` |

---

## Cross-References

- [00-identity.md](00-identity.md) -- ERC-8004 operator field for sibling discovery
- [03-marketplace.md](03-marketplace.md) -- Cross-Clade marketplace transactions carry 10% protocol fee
- [../10-safety/03-ingestion.md](../10-safety/03-ingestion.md) -- All Clade imports pass through ingestion pipeline
- [04-coordination.md](04-coordination.md) -- Pheromone Field, Bloodstain Network, ERC-8001 for cross-Clade coordination
- [../20-styx/00-architecture.md](../20-styx/00-architecture.md) -- Styx three-layer architecture, L0/L1/L2 model, API surface

---

## References

- [GRASSE-1959] Grasse, P.-P. "La reconstruction du nid et les coordinations interindividuelles." _Insectes Sociaux_.
- [WU-2025] Wu, Y. et al. "Memory in LLM Multi-Agent Systems." TechRxiv 2025.
- [KLEPPMANN-2019] Kleppmann, M. et al. "Local-First Software: You Own Your Data, in Spite of the Cloud." _Onward! 2019_.
- [GERSTGRASSER-2023] Gerstgrasser, M. et al. "SUPER: Agent Collaboration via Surprise-Based Experience Sharing." arXiv:2311.00865.
