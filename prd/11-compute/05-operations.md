# 05 -- Operations and monitoring [SPEC]

**Metrics, alerting, Styx health, scaling, HA, and disaster recovery**

> **Reader orientation:** This document specifies operational concerns for Bardo Compute: high availability, monitoring, alerting, and disaster recovery. It belongs to the **Compute** layer of Bardo (the Rust runtime for Golems, mortal autonomous DeFi agents). The key concept before diving in: Bardo Compute has asymmetric failure costs where a zombie VM leaking past TTL expiry costs real money every minute, so every HA mechanism is designed to fail-safe by destroying machines rather than silently accumulating cost. Terms like Golem, Styx, x402, and Thanatopsis are defined inline on first use; a full glossary lives in `00-overview.md § Terminology`.

---

## Why HA is non-negotiable

Bardo Compute has asymmetric failure costs. A leaked zombie VM costs real money every minute it runs; a missed TTL expiry is not "eventually consistent" -- it is a billing leak. Every HA mechanism below is designed with zombie prevention as the top priority. The system must fail-safe: if anything goes wrong, machines get destroyed rather than silently accumulating cost.

---

## Multi-region, multi-instance deployment

| Fly app | Instances | Regions | Scaling |
| --- | --- | --- | --- |
| `bardo-control` | Min 2 | ams, ord | Autoscale to 10 (CPU-based) |
| `bardo-machines` | N/A | Per user request | Per deployment |

All control plane logic runs in at least 2 regions. If one region fails entirely, the other continues serving.

---

## Zero-downtime rolling deploys

```toml
# bardo-control (main process)
kill_signal = "SIGTERM"
kill_timeout = 30

[processes]
  api = ""
  ssh = "--ssh-mode"

[processes.ssh]
  kill_signal = "SIGTERM"
  kill_timeout = 300
```

SSH handler runs in a separate process group. Its 5-minute drain window does not block API rolling deploys.

---

## TTL worker HA

### Turso-based leader election

```sql
UPDATE system_locks
SET holder = ?1, acquired_at = unixepoch()
WHERE name = 'ttl_worker'
  AND (holder IS NULL OR acquired_at < unixepoch() - 60)
RETURNING holder;
```

**Lock TTL**: 60 seconds (implicit). **Refresh**: Every 30s. **Consistency**: Turso single-writer guarantees CAS atomicity.

### TTL worker cycle

Every 30 seconds, the elected leader queries for expired machines and initiates destruction. On destruction, the Golem receives SIGTERM and begins Thanatopsis (four-phase death protocol). See [02-provisioning.md](02-provisioning.md) for the graceful shutdown sequence.

### Failure scenarios

| Scenario | Recovery time | Mechanism |
| --- | --- | --- |
| Holding instance crashes | <=60s (lock expiry) | Another instance acquires via CAS |
| Turso primary failover | <=10s (automatic) | Embedded replicas serve reads |
| All control instances down | <=60s | Machine-local cron queries Turso |

---

## Styx health monitoring

Each Golem maintains an outbound WebSocket to Styx. The health endpoint reports Styx connection status:

```rust
#[derive(Debug, Serialize)]
pub struct GolemHealthResponse {
    pub status: String,       // "ready", "booting", "draining", "crashed"
    pub uptime_seconds: u64,
    pub vitality: f64,        // composite vitality from three death clocks
    pub styx_connected: bool,
    pub styx_latency_ms: Option<u64>,
    pub styx_last_sync: Option<u64>,  // epoch ms of last successful sync
    pub tool_count: u32,
    pub ttl_remaining_seconds: u64,
    pub behavioral_phase: String,     // "thriving", "stable", "conservation", etc.
}
```

### Styx degradation handling

The Golem's `bardo-styx` extension handles all Styx degradation internally:

| Styx status | Golem behavior | TUI indicator |
| --- | --- | --- |
| **Fully online** | Full ecology: backup, clade retrieval, lethe, pheromone field, bloodstain | Green |
| **Degraded** | Writes queue locally (up to 1,000 entries). Queries return cached results. | Amber |
| **Offline** | Local Grimoire is authoritative. Peer-to-peer clade sync via direct gossip. | Red |
| **Never enabled** | Identical to offline. Local Grimoire, file-based death bundles. | Gray |

A circuit breaker (exponential backoff: 3 failures -> open, 60s half-open probe) prevents cascading failures. The control plane does NOT monitor or manage the Styx connection -- it is entirely the Golem's responsibility.

### Outbound WebSocket model (NAT-friendly)

All Styx connections are outbound from the Golem. Zero inbound ports required. This means:

- Fly.io VMs behind 6PN private networking work without configuration
- Self-hosted Golems behind NAT/firewall work without port forwarding
- The TUI can reach remote Golems *through* Styx (events flow: Golem -> Styx -> TUI)

```
TUI <-- WSS ---- Styx <-- WSS ---- Golem (behind NAT)
         |                              |
         |  Events flow: Golem -> Styx -> TUI
         |  Steers flow: TUI -> Styx -> Golem
```

---

## Reconciliation job

Every 5 minutes, independent of the TTL worker:

```sql
SELECT id, machine_id FROM machines
WHERE status = 'ready' AND expires_at < unixepoch() - 120
```

Catches machines >2 minutes past expiry. Also catches orphaned Fly machines (VMs running without corresponding DB records).

---

## Redis: optional only

Redis is **not required** for any correctness-critical operation. All authoritative state lives in Turso.

| Use case | Without Redis | With Redis |
| --- | --- | --- |
| Rate limiting | In-memory token bucket (per-instance) | Distributed sliding window |
| Nonce dedup | Turso UNIQUE constraint | Same |
| TTL enforcement | Turso poll worker | Same |
| Proxy cache | In-process LRU | Same |
| Region capacity | Turso COUNT query | Same |

---

## Database HA

### Turso

- **Automatic primary failover**: Transparent
- **Embedded replicas**: Zero-latency reads from the same process
- **Write routing**: Automatically routes to primary
- **Read-after-write**: Use `sync()` after billing-critical writes

### Turso backup

Nightly export of `billing_events` to Cloudflare R2:

```
Schedule:  0 2 * * * (2 AM UTC daily)
Target:    r2://bardo-backups/billing/{date}.jsonl.gz
Retention: 90 days
RPO:       ~24 hours
RTO:       <10 minutes (import from R2)
```

---

## Grimoire snapshot storage

Cloudflare R2 stores Grimoire snapshots. Latest 5 snapshots retained per machine. ~50MB typical per machine.

Snapshot types: periodic (every 6 hours), shutdown (Thanatopsis Phase II), manual (user-triggered).

Grimoire also streams continuously to Styx Archive layer. R2 snapshots are the backup-of-last-resort.

---

## Disk monitoring

Health endpoint includes disk usage. Alert threshold: >85%. Remediation: >90% triggers log truncation; >95% pauses Golem process.

---

## Golem-level health

Extended health with agent-specific metrics:

```rust
#[derive(Debug, Serialize)]
pub struct ExtendedGolemHealth {
    pub status: String,
    pub uptime_seconds: u64,
    pub disk_usage_percent: u8,
    pub agent: AgentHealth,
}

#[derive(Debug, Serialize)]
pub struct AgentHealth {
    pub heartbeat_count: u64,
    pub last_heartbeat_age_seconds: u64,
    pub tool_calls_total: u64,
    pub tool_calls_last_hour: u32,
    pub inference_tokens_used: u64,
    pub inference_tokens_allowance: u64,
    pub model: String,
    pub strategy_type: String,
    pub crash_count: u32,
    pub vitality: f64,
    pub behavioral_phase: String,
    pub sustainability_ratio: Option<f64>,
    pub styx_connected: bool,
}
```

### Control plane alerting thresholds

| Metric | Threshold | Severity | Action |
| --- | --- | --- | --- |
| `last_heartbeat_age > 300` | 5 min without heartbeat | Warning | Check process |
| `crash_count > 0` | Any supervisor restart | Warning | Investigate |
| `crash_count >= 5` | Max restarts reached | Critical | Machine will self-terminate |
| `inference_tokens_used > allowance * 0.9` | 90% of token budget | Warning | Notify user |
| `disk_usage_percent > 85` | Disk filling | Warning | Trigger log cleanup |
| `vitality < 0.3` | Golem entering Declining phase | Info | Notify owner |
| `styx_connected == false` for 10 min | Styx connection lost | Warning | Check network |

---

## OpenTelemetry metrics

14 metrics exported via OpenTelemetry protocol:

| Metric | Type | Labels | Alert threshold |
| --- | --- | --- | --- |
| `bardo_machines_active` | Gauge | region, vm_size | >100/region |
| `bardo_provision_duration_seconds` | Histogram | region, vm_size, outcome | p99 > 30s |
| `bardo_provision_total` | Counter | region, vm_size, outcome | failure_rate > 10% |
| `bardo_extension_total` | Counter | payer_type | -- |
| `bardo_ttl_worker_runs_total` | Counter | outcome | stuck > 90s |
| `bardo_ttl_worker_duration_seconds` | Histogram | -- | p99 > 25s |
| `bardo_revenue_micro_usdc_total` | Counter | type | daily < 1,000,000 |
| `bardo_zombie_machines` | Gauge | -- | >0 for 5 min |
| `bardo_reconciliation_corrections` | Counter | type | >0 |
| `bardo_turso_latency_seconds` | Histogram | operation | p99 > 50ms |
| `bardo_proxy_request_duration_seconds` | Histogram | subdomain, status | p99 > 5s |
| `bardo_warm_pool_size` | Gauge | region | <2 |
| `bardo_styx_connections_active` | Gauge | region | -- |
| `bardo_golem_vitality` | Gauge | machine_name, phase | <0.1 (terminal) |

---

## Alerting

| Alert | Condition | Severity | Action |
| --- | --- | --- | --- |
| Zombie machines | `bardo_zombie_machines > 0` for 5 min | Critical | Investigate TTL worker |
| TTL worker stuck | Same holder >90s without refresh | High | Check instance health |
| Cost spike | Revenue today >1.5x yesterday | Warning | Verify growth is organic |
| Provision failure rate | >10% in 15 min window | High | Check Fly Machines API |
| Turso latency | p99 >50ms for 5 min | Warning | Check Turso dashboard |
| Region approaching cap | >80 machines in any region | Warning | Consider adding regions |
| Control instances < 2 | Instance count drops | Critical | Investigate autoscaler |
| Warm pool depleted | Pool size <2 in any region | Warning | Check pool manager |
| Golem heartbeat stale | Any `last_heartbeat_age > 300` | Warning | Check process |
| Disk usage high | Any `disk_usage_percent > 85` | Warning | Trigger log cleanup |

Alert routing: Critical -> PagerDuty. High/Warning -> Slack `#bardo-alerts`.

---

## Cost protection

| Control | Mechanism | Limit |
| --- | --- | --- |
| Per-region cap | Turso COUNT query | 100 machines/region |
| Global cap | Turso COUNT query | 500 machines |
| Min payment | x402 must cover >=1 hour | 25,000 micro-USDC |
| Min extension | 1 hour at machine's rate | See 03-billing.md |
| Max single extension | TTL addition capped | 720 hours |
| Zombie detection | TTL worker + reconciliation + cron | <90 sec max |
| Per-user limit | Turso count check | 5 active machines |

---

## Structured logging

All Golem binary output is structured JSON to stdout. The Fly.io log aggregator collects it.

```json
{
  "level": "info",
  "timestamp": 1709901234567,
  "trace_id": "550e8400-e29b-41d4-a716-446655440000",
  "golem_id": "golem-V1StGXR8_Z5j",
  "msg": "HeartbeatComplete",
  "tick": 847,
  "duration_ms": 234,
  "tier": "T0",
  "vitality": 0.72,
  "phase": "stable"
}
```

Control plane uses the same structured JSON format with `trace_id` propagation across HTTP headers, Turso records, and Fly Machines API calls.

---

## Distributed tracing

Every request gets a `trace_id` (UUID v4) assigned at the edge. Propagates through all layers. The Golem's Event Fabric includes `trace_id` on events that originate from external requests (steers, strategy updates).

---

## Failure mode matrix

| Failure | Impact | Recovery | Mitigation |
| --- | --- | --- | --- |
| Single control instance crash | No impact | <30s (Fly auto-restart) | Min 2 instances |
| All control instances down | No new provisions | <60s (autoscale) | Machine-local cron |
| Fly Machines API down | No provisions/destructions | Variable | Queue destruction for retry |
| Turso primary down | Read-only mode | <10s (auto-failover) | Embedded replicas |
| Golem VM crash | Individual Golem down | Supervisor restarts (up to 5) | Grimoire snapshot preserved |
| Network partition (6PN) | Proxy unreachable | Variable | Machine-local cron |
| R2 outage | No Grimoire snapshots | Variable | Snapshots queued locally |
| Styx outage | No clade sync, no lethe | Golem continues at ~95% | Local Grimoire authoritative |

---

## Scalability

At v1 scale (<500 machines, <100 writes/min), Turso's single-writer model is not a bottleneck. Options at scale: shard by region, write batching, or migrate to multi-writer DB.

The Golem binary's resource footprint is small (~20MB RSS at idle, ~50MB under load). A `micro` VM (256MB) runs one Golem comfortably. The constraint is inference cost, not compute.
