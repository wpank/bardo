# Clade Synchronization: Styx-Relayed Knowledge Exchange [SPEC]

> **PRD2 Section**: 20-styx | **Source**: Styx Research S4 v4.3

> **Status**: Implementation Specification
>
> **Supersedes**: S4 v4.2 (corrects sync frequency and pricing assumptions)
>
> **Crates**: `golem-coordination` (clade.rs), `bardo-styx` (relay module)
>
> **Dependencies**: `prd2/04-memory/10-safety.md` (ingestion pipeline), `prd2/09-economy/04-coordination.md` (coordination model), `prd2/09-economy/02-clade.md` (clade structure)

> **Reader orientation:** This document specifies the clade (sibling Golems sharing a common ancestor, exchanging knowledge through Styx) synchronization protocol: what gets synced, when, the relay mechanism, version vectors, store-and-forward for offline siblings, and morphogenetic specialization signals. It belongs to the Styx layer of Bardo. The key concept is that sync is event-driven and Curator-aligned, not per-tick, producing 5-25 entries per Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) per day. Familiarity with the Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) ingestion pipeline is assumed. See `prd2/shared/glossary.md` for full term definitions.

---

## What This Document Covers

Clade sync is how Golems in the same user's fleet share knowledge. This document specifies the protocol, what gets synced and when, the outbound WebSocket model, pricing, and configuration. Clade sync (and Styx connectivity in general) is **fully optional** -- a Golem with `styx.enabled: false` operates on its local Grimoire alone.

---

## 1. The Outbound WebSocket Model

Every Golem that enables Styx maintains ONE persistent outbound WebSocket connection (`wss://styx.bardo.run/v1/styx/ws`). This works from behind any NAT, firewall, or restrictive network -- it's a standard outbound HTTPS connection on port 443. No inbound ports, no tunnels, no port forwarding.

Styx uses this connection as a bidirectional channel for ALL traffic: clade sync, pheromone updates, bloodstain notifications, entry writes, retrieval queries, and TUI event forwarding. See `prd2/20-styx/06-deployment.md` for how this makes all deployment paths (Bardo Compute, self-hosted VPS, Raspberry Pi) equivalent.

---

## 2. What Gets Synced and When

### The Key Insight: Most Ticks Produce Nothing Worth Sharing

A T0 tick (80% of ticks) generates no new Grimoire entries. A T1 tick might generate an observation episode. A T2 tick might generate an insight or a warning. Only entries with `propagation >= Clade` are eligible for sync -- and the promotion gates (see `prd2/04-memory/10-safety.md`) filter aggressively:

| Entry Type | Promotion Gate | Typical Daily Volume (per Golem) |
|-----------|---------------|--------------------------------|
| Warning | confidence >= 0.4 (auto-promote) | 1-5 |
| Insight | confidence >= 0.6 | 3-10 |
| Heuristic | confidence >= 0.7 | 0-3 |
| Death testament | On death only | 0 or 1 |
| Bloodstain | On death only | 0 or 1 |
| Causal edge | confidence >= 0.6, evidence >= 3 | 1-5 |

**Realistic total**: 5-25 clade-eligible entries per Golem per day. Not thousands.

### Three Sync Triggers

Sync does NOT happen every tick. It happens when there's something worth sharing:

**1. Event-Driven (immediate for high-priority items)**

When a Golem produces a WARNING or BLOODSTAIN, it's pushed to Styx immediately for relay to siblings. These are time-sensitive safety signals -- a 12-minute delay could be the difference between a sibling avoiding a liquidation cascade or not.

```rust
/// Push a high-priority entry immediately, outside the regular sync cycle.
/// Only for: warnings (any confidence), bloodstains, critical risk alerts.
pub async fn push_immediate(
    styx_conn: &mut StyxConnection,
    entry: &SyncEntry,
) -> Result<()> {
    styx_conn.send(StyxMessage::CladeDeltaImmediate {
        entries: vec![entry.clone()],
    }).await
}
```

**2. Curator-Aligned (every 50 ticks, ~12.5 minutes)**

The Curator cycle (see `prd2/04-memory/01-grimoire.md`) runs every 50 ticks. After the Curator validates, prunes, and compresses, the clade extension collects all new clade-eligible entries since the last sync and pushes them as a batch. This aligns sync with the knowledge maintenance cycle -- entries are synced after they've been quality-checked, not raw.

```rust
/// Batch sync aligned with the Curator cycle.
/// Runs every 50 ticks in the clade extension's after_turn hook.
pub async fn curator_aligned_sync(
    styx_conn: &mut StyxConnection,
    grimoire: &Grimoire,
    last_sync_tick: u64,
    current_tick: u64,
) -> Result<SyncResult> {
    // Only sync every 50 ticks (aligned with Curator)
    if current_tick % 50 != 0 { return Ok(SyncResult::Skipped); }

    // Collect clade-eligible entries created since last sync
    let new_entries = grimoire.get_entries_since_tick_with_propagation(
        last_sync_tick,
        PropagationPolicy::Clade,
    )?;

    if new_entries.is_empty() { return Ok(SyncResult::NothingToSync); }

    // Push batch to Styx for relay
    styx_conn.send(StyxMessage::CladeDeltaBatch {
        entries: new_entries.iter().map(|e| SyncEntry::from(e)).collect(),
        version_vector: grimoire.version_vector(),
        bloom_filter: Some(grimoire.build_bloom_filter()?),
    }).await?;

    Ok(SyncResult::Synced { entries_pushed: new_entries.len() as u32 })
}
```

**3. On-Demand (user or Golem-initiated)**

A Golem can request a full sync at any time (e.g., after booting for the first time, or after recovering from an outage). This pulls all pending deltas from Styx.

### Typical Sync Volume

| Scenario | Entries/day/Golem | Sync Messages/day/Golem | Daily Cost/Golem |
|----------|------------------|------------------------|-----------------|
| Quiet market (mostly T0) | 5-10 | ~12 batch + ~2 immediate | ~$0.007 |
| Active market (mixed T0/T1/T2) | 15-25 | ~12 batch + ~5 immediate | ~$0.009 |
| Volatile/crisis | 25-50 | ~12 batch + ~10 immediate | ~$0.011 |
| Death day | +1 bloodstain bundle | +1 immediate | +$0.005 |

**For a 5-Golem clade in active market**: ~$0.045/day x 30 = **~$1.35/month** in sync relay costs. The previous estimate was wrong by three orders of magnitude because it assumed every tick generates a sync message.

---

## 3. The Relay Protocol

### Styx Connection Registry

Styx maintains an in-memory registry of connected Golems, indexed by `user_id`:

```rust
/// In-memory registry of connected Golems.
/// When Golem A pushes a delta, Styx looks up all siblings
/// (same user_id, different golem_id) and pushes to their WebSocket connections.
pub struct ConnectionRegistry {
    connections: DashMap<String, Vec<ConnectedGolem>>,
}

impl ConnectionRegistry {
    /// Route a clade delta to all siblings of the sender.
    pub async fn route_clade_delta(
        &self,
        sender_golem_id: &str,
        sender_user_id: &str,
        delta: CladeDelta,
    ) {
        if let Some(peers) = self.connections.get(sender_user_id) {
            for peer in peers.iter() {
                if peer.golem_id != sender_golem_id {
                    let _ = peer.tx.send(StyxMessage::CladeSync {
                        from_golem_id: sender_golem_id.to_string(),
                        entries: delta.entries.clone(),
                    }).await;
                }
            }
        }
    }
}
```

### In-Memory Connection Registry

Styx maintains an in-memory registry of connected Golems, indexed by `user_id`. When Golem A pushes a delta, Styx looks up all siblings (same `user_id`, different `golem_id`) and pushes to their WebSocket connections:

```rust
/// In-memory registry of connected Golems.
/// When Golem A pushes a delta, Styx looks up all siblings
/// (same user_id, different golem_id) and pushes to their WebSocket connections.
pub struct ConnectionRegistry {
    connections: DashMap<String, Vec<ConnectedGolem>>,
}

impl ConnectionRegistry {
    /// Route a clade delta to all siblings of the sender.
    pub async fn route_clade_delta(
        &self,
        sender_golem_id: &str,
        sender_user_id: &str,
        delta: CladeDelta,
    ) {
        if let Some(peers) = self.connections.get(sender_user_id) {
            for peer in peers.iter() {
                if peer.golem_id != sender_golem_id {
                    let _ = peer.tx.send(StyxMessage::CladeSync {
                        from_golem_id: sender_golem_id.to_string(),
                        entries: delta.entries.clone(),
                    }).await;
                }
            }
        }
    }
}
```

### Sibling Discovery

Golems discover their siblings through two mechanisms:

1. **Styx connection registry**: When a Golem connects, Styx checks the registry for existing connections with the same `user_id`. Sibling presence is known immediately.
2. **`GET /v1/styx/clade/peers/:user_id`**: Returns all registered clade peers with connection status, enabling discovery even when the querying Golem just booted.

### Delta Relay

Clade deltas flow through Styx as a message pipe. Styx does not inspect or store the delta content for batch syncs -- it relays the entries from sender to all siblings via the connection registry. For immediate syncs (warnings, bloodstains), the relay is prioritized in the message queue.

### Version Vectors

Each Golem maintains a version vector `{golem_id -> last_seen_seq}` tracking the highest sequence number received from each source [LAMPORT-1978], [FIDGE-1988]. This prevents re-processing entries and enables efficient delta computation when a Golem reconnects after being offline.

### Store-and-Forward for Offline Siblings

If a sibling is offline when a delta is pushed, Styx stores it in PostgreSQL (see the `pending_sync_deltas` table). When the sibling reconnects, it receives all pending deltas in order. Pending deltas expire after 7 days.

### Ingestion on Receipt

Every received entry passes through the standard four-stage ingestion pipeline (see `prd2/04-memory/10-safety.md`) with `IngestRelationship::Clade` (confidence x 0.80). This preserves the Weismann barrier and the testing effect -- inherited knowledge must prove itself through operational use. See also `prd2/01-golem/09-inheritance.md` for the broader inheritance model.

---

## 4. Configuration

Styx connectivity and clade sync are fully optional. Configuration in `golem.toml`:

```toml
[styx]
# Master switch -- false means the Golem runs entirely on local Grimoire.
# No Styx connection, no clade sync, no pheromone field, no bloodstain network.
# The Golem operates at ~95% capability using only its local knowledge.
enabled = true                    # default: true

# Styx service endpoint
host = "styx.bardo.run"          # default

[styx.vault]
# L0 backup of Grimoire to Styx
enabled = true                    # default: true
ttl = "90d"                       # How long backups are retained
backup_interval_ticks = 200       # How often to push a full backup (~50 min)

[styx.clade]
# Clade sync with siblings
enabled = true                    # default: true
# How often to run batch sync (in ticks). Default: every Curator cycle.
sync_interval_ticks = 50          # default: 50 (aligned with Curator)
# Push warnings immediately (outside the batch cycle)?
immediate_warnings = true         # default: true
# Push bloodstains immediately?
immediate_bloodstains = true      # default: true

[styx.lethe]
# L2 anonymized Lethe (formerly Commons) participation
read_enabled = true               # default: true (read from Lethe)
publish_enabled = true            # default: true (publish to Lethe)
domains = ["dex-lp", "lending", "gas", "regime"]  # Which domains to participate in

[styx.pheromone]
# Pheromone Field participation
enabled = true                    # default: true
deposit_enabled = true            # default: true (deposit signals)
sense_enabled = true              # default: true (read signals)

[styx.marketplace]
# Marketplace participation
browse_enabled = true             # default: true
auto_discover = true              # Golem autonomously searches marketplace
max_auto_spend = 1.00             # Max USDC per auto-purchase
min_seller_reputation = 60        # Only buy from Verified+ sellers
sell_enabled = false              # Opt-in to create listings

[styx.budget]
# Spending limits for Styx operations
max_per_tick = 0.01               # Max USDC per tick on Styx operations
daily_budget = 0.50               # Max USDC per day on Styx operations
monthly_budget = 10.00            # Max USDC per month on Styx operations
```

### Minimal Configuration (Styx Disabled)

```toml
[styx]
enabled = false
# That's it. The Golem runs on local Grimoire only.
# Clade sync uses file-based death bundles and local P2P if configured.
```

### Budget Controls

The `styx.budget` section enforces hard spending limits. Even if the Golem's strategy triggers many queries, the daily/monthly cap prevents runaway costs. When a budget limit is hit, the Golem degrades to local-only operation until the next budget period.

---

## 5. Optional Direct P2P (Advanced Users)

For users running multiple Golems on the same network (same server, same LAN, Tailscale mesh), direct P2P sync bypasses Styx relay -- saving relay costs and reducing latency to ~5ms.

```toml
[styx.clade.p2p]
enabled = true
listen_address = "127.0.0.1:8402"  # Local only, or Tailscale IP
peers = [
    "ws://10.0.0.2:8402",           # Sibling on same LAN
    "ws://100.64.0.3:8402",         # Sibling on Tailscale
]
```

When P2P is enabled, the Golem attempts direct connections first. If a peer is unreachable, it falls back to Styx relay. Both paths use the same delta format and ingestion pipeline.

---

## 6. Security

| Threat | Protection |
|--------|-----------|
| Impersonation | WebSocket authenticated with x402 or API key linked to ERC-8004 |
| Cross-user leakage | Styx routes only to connections with matching `user_id` |
| Delta injection | All entries pass through four-stage ingestion pipeline (confidence x 0.80) |
| Replay | Version vectors prevent re-processing |
| Eavesdropping | TLS 1.3 on all WebSocket connections |
| Budget exhaustion | Hard caps: per-tick, daily, monthly |

---

## 7. What Happens Without Styx

A Golem with `styx.enabled: false` is fully functional:

| Feature | Without Styx | With Styx |
|---------|-------------|-----------|
| Local Grimoire | Full operation | + L0 backup |
| Heartbeat pipeline | Full operation | Same |
| Dreams | Full operation | + NREM pool includes L1 scars |
| Clade knowledge sharing | File-based death bundles only | Real-time relay |
| Pheromone Field | No swarm intelligence | Threat/opportunity signals |
| Bloodstain Network | Local death bundles only | Ecosystem-wide death knowledge |
| Lethe | No ecosystem knowledge | Anonymized structural intelligence |
| Marketplace | No knowledge commerce | Buy/sell knowledge |
| TUI ecosystem view | Local data only | Live multiplayer dashboard |

The ~95% capability estimate comes from the fact that the local Grimoire, heartbeat pipeline, dreams, daimon, and mortality system all function identically without Styx. What's lost is cross-agent and cross-lifetime intelligence -- valuable but not essential for any individual Golem's operation.

---

## 8. Morphogenetic Clade Specialization Signals

Morphogenetic specialization (see `prd2/02-mortality/10b-morphogenetic-specialization.md`) drives spontaneous role differentiation in Clades through reaction-diffusion dynamics. Each Golem maintains a strategy concentration vector that evolves based on profitable performance (activation, slow) and Clade-wide pheromone signals (inhibition, fast). Because inhibition propagates through Styx in milliseconds while learning takes hundreds of ticks, Turing's instability condition is naturally satisfied and stable specialist patterns emerge from initially homogeneous populations.

The clade sync protocol carries three morphogenetic signal types alongside existing knowledge deltas.

### 8.1 Role Vector

The role vector is an 8-dimensional strategy concentration vector where each dimension represents a strategy type (momentum, mean-reversion, liquidity provision, risk monitoring, time horizon, asset breadth, volatility appetite, cross-chain). The vector sums to 1.0 -- specializing in one dimension means de-emphasizing others.

```rust
/// Morphogenetic role vector broadcast in clade sync.
/// Piggybacks on the existing CladeDeltaBatch message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphogeneticPheromone {
    /// 8-dimensional strategy concentration vector. Sum = 1.0.
    /// [momentum, mean_reversion, lp, risk, time_horizon,
    ///  asset_breadth, vol_appetite, cross_chain]
    pub role_vector: [f64; 8],
    /// Shannon entropy of the role vector, normalized to [0, 1].
    /// Low = specialist. High = generalist.
    pub specialization_index: f32,
    /// Tick at which this pheromone was emitted.
    pub emitted_at_tick: u64,
    /// Golem's current behavioral phase (affects activation rate).
    pub behavioral_phase: String,
}
```

The role vector is included in every `CladeDeltaBatch` message. No additional WebSocket frames are needed -- the morphogenetic payload is 72 bytes (8 x f64 + f32 + u64 + phase string), negligible relative to the knowledge delta content.

### 8.2 Inhibition Radius

The inhibition radius controls how strongly a Golem's strategy vector suppresses similar strategies in its Clade neighbors. It is not a fixed parameter -- it adapts based on Clade size and market conditions.

```rust
/// Inhibition signal computed from aggregated role vectors.
/// Each Golem computes this locally from received MorphogeneticPheromone messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InhibitionSignal {
    /// Per-dimension inhibition pressure from Clade neighbors.
    /// P_k = sum of all neighbors' role_vector[k].
    pub inhibition_pressure: [f64; 8],
    /// Number of effective competitors (Clade members with
    /// cosine similarity > 0.8 to this Golem's role vector).
    pub niche_competition: f32,
    /// The inhibition strength parameter (beta) currently in effect.
    /// Adapts based on Clade size: larger Clades use higher beta
    /// to prevent niche crowding.
    pub effective_beta: f64,
}
```

Inhibition is computed locally, not transmitted. Each Golem aggregates the `role_vector` fields from all received `MorphogeneticPheromone` messages into its `inhibition_pressure` array. The aggregation runs after every Curator-aligned sync (every 50 ticks).

### 8.3 Activation Pattern

The activation pattern tracks which strategy dimensions are being reinforced by profitable returns. Unlike the role vector (which is the current state), the activation pattern shows the direction of change -- which niches the Golem is drifting toward or away from.

```rust
/// Activation pattern: per-dimension return attribution.
/// Not transmitted directly, but its effects are visible in successive
/// role vector snapshots.
#[derive(Debug, Clone)]
pub struct ActivationPattern {
    /// Returns attributed to each strategy dimension since last sync.
    pub attributed_returns: [f64; 8],
    /// Effective activation rate (alpha * mortality_scalar).
    /// Reduced in Conservation (0.5x) and Declining (0.1x) phases.
    pub effective_alpha: f64,
    /// The reaction-diffusion update applies:
    ///   s_k(t+1) = s_k(t) + activation_k - inhibition_k - decay_k + noise_k
    /// where:
    ///   activation_k = effective_alpha * max(0, attributed_returns[k]) * s_k
    ///   inhibition_k = effective_beta * (inhibition_pressure[k] / clade_size) * s_k
    ///   decay_k = mu * (s_k - baseline)
    ///   noise_k ~ N(0, sigma_noise^2)
    pub last_update_tick: u64,
}
```

### 8.4 Pheromone-Based Role Coordination Messages

Three message types coordinate morphogenetic specialization through the existing Styx relay:

```rust
/// Morphogenetic coordination messages carried on the Clade sync channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MorphogeneticMessage {
    /// Standard role broadcast. Included in every CladeDeltaBatch.
    RoleBroadcast(MorphogeneticPheromone),

    /// Niche vacancy alert. Emitted when a specialist Golem dies
    /// and its strategy niche drops below the occupancy threshold.
    /// Triggers accelerated respecialization in surviving siblings.
    NicheVacancy {
        /// The deceased Golem's role vector at time of death.
        vacated_role: [f64; 8],
        /// Which dimension(s) had concentration > 0.3 (the specialist dimensions).
        specialist_dimensions: Vec<usize>,
        /// The deceased Golem's ID (for lineage tracking).
        deceased_golem_id: String,
        /// Tick of death.
        death_tick: u64,
    },

    /// Role conflict alert. Emitted when two Golems' role vectors
    /// have cosine similarity > 0.9 for more than 100 consecutive ticks.
    /// Suggests one should respecialize.
    RoleConflict {
        /// The other Golem in the conflict.
        conflicting_golem_id: String,
        /// The overlapping dimensions (concentration > 0.2 in both).
        overlapping_dimensions: Vec<usize>,
        /// How many ticks the conflict has persisted.
        conflict_duration_ticks: u64,
        /// Suggested respecialization direction (dimension with lowest
        /// aggregate Clade concentration).
        suggested_dimension: usize,
    },
}
```

### 8.5 Sync Protocol Integration

Morphogenetic messages integrate with the existing sync triggers:

1. **Curator-aligned (every 50 ticks).** The `RoleBroadcast` is included in every `CladeDeltaBatch`. No additional messages.

2. **Event-driven (immediate).** `NicheVacancy` is pushed immediately when a Golem detects a sibling death via Bloodstain. This is time-sensitive -- the vacancy signal needs to reach surviving Golems before they run their next reaction-diffusion update so they can respond to the opening.

3. **Threshold-triggered.** `RoleConflict` fires when the conflict duration exceeds 100 ticks. It uses the same immediate push path as warnings.

### 8.6 Mortality Interaction

Mortality phase modulates morphogenetic dynamics through the `effective_alpha` scalar:

| Phase | Alpha Scalar | Effect |
|-------|-------------|--------|
| Thriving | 1.0 | Full activation rate. Golem deepens its specialization based on returns. |
| Stable | 1.0 | Same as Thriving. |
| Conservation | 0.5 | Halved activation. Golem stops reinforcing its niche and becomes more responsive to inhibition. Allows potential respecialization. |
| Declining | 0.1 | Near-zero activation. Golem effectively surrenders its niche. If it recovers, it respecializes based on current Clade structure. |
| Terminal | 0.0 | No activation. The Golem's role vector freezes. Its pheromone signal persists until the Bloodstain processes the death. |

When a successor Golem inherits its predecessor's role vector at 0.5x concentration, it starts as a partial specialist. Whether it deepens the inherited niche or drifts depends on the Clade's pheromone field at boot time -- if the niche is already filled by another sibling that respecialized during the succession interval, the successor will be pushed elsewhere by inhibition pressure.

### 8.7 Convergence Guarantees

Convergence timeline: 500-2000 ticks depending on clade size. Stability condition: trait variance < 0.01 for 100 consecutive ticks. Pathological configurations (>50 golems with adversarial traits) may require manual clade partitioning.

### 8.8 Configuration

Morphogenetic parameters are configurable in `golem.toml`:

```toml
[styx.clade.morphogenetic]
# Enable reaction-diffusion specialization.
enabled = true                    # default: true (requires styx.clade.enabled)
# Activation rate: how fast profitable strategies reinforce.
alpha = 0.05                      # default: 0.05
# Inhibition rate: how strongly Clade overlap suppresses.
beta = 0.15                       # default: 0.15
# Decay rate toward baseline (prevents extreme specialization).
mu = 0.01                         # default: 0.01
# Noise standard deviation for symmetry breaking.
sigma_noise = 0.005               # default: 0.005
# Emit NicheVacancy alerts on sibling death.
vacancy_alerts = true             # default: true
# Emit RoleConflict alerts after this many ticks of high similarity.
conflict_threshold_ticks = 100    # default: 100
```

---

## References

- [LAMPORT-1978] Lamport, L. "Time, Clocks, and the Ordering of Events in a Distributed System." *CACM*, 21(7), 1978. — *Foundation for version vectors used in clade delta deduplication.*
- [FIDGE-1988] Fidge, C.J. "Timestamps in Message-Passing Systems." *ACSC*, 10(1), 1988. — *Vector clock formalization applied to per-Golem sequence tracking.*
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance." *Cell*, 157(1), 2014. — *Biological basis for the Weismann barrier in inherited knowledge confidence scaling.*
- [ROEDIGER-KARPICKE-2006] Roediger, H.L. & Karpicke, J.D. "Test-Enhanced Learning." *Psychological Science*, 17(3), 2006. — *Testing effect: inherited entries must prove themselves through operational use.*
- [TURING-1952] Turing, A.M. "The Chemical Basis of Morphogenesis." *Phil. Trans. Royal Society B*, 237(641), 1952. — *Reaction-diffusion model underlying morphogenetic clade specialization.*
- [GIERER-MEINHARDT-1972] Gierer, A. & Meinhardt, H. "A Theory of Biological Pattern Formation." *Kybernetik*, 12(1), 1972. — *Activator-inhibitor dynamics formalized for strategy niche differentiation.*

---

*Sync happens when there's something worth sharing -- not on every tick, not on a timer, but when validated knowledge crosses the promotion gate. The cost is cents per day, not dollars. And now, specialization emerges from the same sync channel -- pheromone signals that push Golems apart in strategy space, creating an ecology of complementary specialists from identical starting conditions.*
