# Observability [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies the observability stack for a running Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM): health checks, Prometheus metrics, structured logging, OpenTelemetry traces, and analytics telemetry. Everything derives from the Event Fabric (the tokio::broadcast channel carrying all typed events). It sits in the **Runtime** layer of the Bardo specification. Key prerequisite: the Event Fabric and GolemEvent enum from `12-realtime-subscriptions.md`. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Per-Golem observability built on the Event Fabric. The Event Fabric is the primary observability channel: 50+ typed events across 16 subsystems, broadcast via `tokio::broadcast` with a 10,000-entry ring buffer. Prometheus metrics, structured logging, and OpenTelemetry traces are derived from Event Fabric events, not maintained as separate state.

> **Crate**: `golem-core` (event_fabric.rs, events.rs), `golem-runtime` (telemetry.rs)
>
> **Cross-references:**
>
> - `./11-state-model.md` — defines GolemState (mutable) and GolemSnapshot (read-only projection), the state structures that observability instruments
> - `./12-realtime-subscriptions.md` — full GolemEvent enum (50+ variants), subscription topics, and the event catalog driving all metrics and traces
> - `../05-dreams/06-integration.md` — dream observability: metrics for dream cycle frequency, consolidation quality, and PLAYBOOK revision rate
> - `../03-daimon/07-runtime-daimon.md` — emotion metrics: PAD vector histograms, mood transition frequency, and emotional contagion tracking
> - `../02-mortality/12-integration.md` — mortality event integration: Vitality gauge, phase transition alerts, and death protocol telemetry

---

## 1. Event Fabric as observability backbone

Every internal state transition in the Golem's lifecycle emits a typed, serializable `GolemEvent`. Events are the atoms of observability. They feed:

- **Prometheus metrics** -- counters, gauges, and histograms derived from events
- **Structured logs** -- events serialized to JSONL
- **OpenTelemetry traces** -- spans constructed from event pairs (start/end)
- **TUI rendering** -- events drive sprite animations and widget updates
- **Alerting** -- rules evaluate against event streams

The Event Fabric broadcasts via `tokio::broadcast`. Zero subscribers means zero serialization cost -- the `emit()` path checks subscriber count first.

### Event Fabric architecture

```rust
pub struct EventFabric {
    sender: tokio::sync::broadcast::Sender<GolemEvent>,
    ring_buffer: parking_lot::RwLock<VecDeque<GolemEvent>>,
    sequence: AtomicU64,
    capacity: usize,  // 10,000
}

impl EventFabric {
    pub fn emit(&self, event: GolemEvent) {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        // Only serialize if subscribers exist
        if self.sender.receiver_count() > 0 {
            let _ = self.sender.send(event.clone());
        }
        // Always store in ring buffer for reconnection replay
        let mut buffer = self.ring_buffer.write();
        if buffer.len() >= self.capacity {
            buffer.pop_front();
        }
        buffer.push_back(event);
    }

    pub fn replay_from(&self, last_sequence: u64) -> Vec<GolemEvent> {
        let buffer = self.ring_buffer.read();
        buffer.iter()
            .filter(|e| e.sequence() > last_sequence)
            .cloned()
            .collect()
    }
}
```

### 16 subsystems

| Subsystem | Event prefix | Event count | Update frequency |
|---|---|---|---|
| Heartbeat | `HeartbeatTick`, `HeartbeatSuppress` | 3 | Every tick |
| Tool | `ToolStart`, `ToolUpdate`, `ToolEnd` | 3 | Per tool call |
| LLM | `LlmStart`, `LlmToken`, `LlmEnd` | 3 | Per inference call |
| Dream | `DreamStart`, `DreamPhase`, `DreamHypothesis`, `DreamEnd` | 5 | Per dream cycle |
| Daimon | `DaimonAppraisal` | 1 | Per appraisal |
| Vitality | `VitalityUpdate` | 1 | Every tick |
| Mortality | `MortalityWarning`, `DeathImminent`, `StochasticCheck`, `Senescence` | 4 | On threshold cross |
| Death | `DeathProtocolStep` | 1 | During Thanatopsis |
| Grimoire | `GrimoireInsight`, `GrimoireHeuristic`, `GrimoireWarning`, `GrimoireCausalLink` | 4 | On knowledge change |
| Permit | `PermitCreated`, `PermitCommitted`, `PermitExpired`, `PermitCancelled`, `PermitBlocked` | 5 | Per action permit |
| Context | `ContextAssembled` | 1 | Per LLM call |
| Compaction | `CompactionTriggered` | 1 | On context compaction |
| Clade | `CladeSync`, `CladeAlert`, `CladeSiblingDeath` | 3 | On sync |
| Model | `ModelSelected` | 1 | Per inference call |
| Inference | `InferenceToken` | 1 | Per token (high-freq) |
| System | `HeartbeatSuppressed`, `DreamOutcome` | 2+ | Various |

---

## 2. Health check endpoints

Three Kubernetes-compatible probes on port `:8402` (Golem RPC).

### Startup probe

```
GET /health/startup
```

Returns `200` after the Golem completes its first heartbeat tick. Returns `503` before that.

### Liveness probe

```
GET /health/live
```

Returns `200` if the Golem process is alive and not deadlocked. Checks: process running, event loop responsive within 3s, memory below 90% of limit.

### Readiness probe

```
GET /health/ready
```

Returns `200` if the Golem is ready to process heartbeats. Returns `503` during boot, pause, or draining. Includes component status:

```json
{
  "status": "ready",
  "heartbeat": { "phase": "idle", "tick": 4201, "interval_ms": 40000 },
  "wallet": { "connected": true, "balance": 423.5 },
  "grimoire": { "episodes": 8420, "insights": 342 },
  "dream": { "mode": "waking", "lifetime_cycles": 42 },
  "mood": { "label": "confident", "octant": "exuberant" },
  "mortality": { "phase": "stable", "composite": 0.72 }
}
```

---

## 3. Prometheus metrics

Derived from Event Fabric events. Exposed at `/metrics` on port `:8402`.

### Golem core

```prometheus
# HELP bardo_golem_ticks_total Total heartbeat ticks processed
# TYPE bardo_golem_ticks_total counter
bardo_golem_ticks_total{cognitive_load="t0"} 33600
bardo_golem_ticks_total{cognitive_load="t1"} 6300
bardo_golem_ticks_total{cognitive_load="t2"} 1680

# HELP bardo_golem_tick_duration_seconds Heartbeat tick duration
# TYPE bardo_golem_tick_duration_seconds histogram
bardo_golem_tick_duration_seconds_bucket{phase="sensing",le="0.01"} 39000
bardo_golem_tick_duration_seconds_bucket{phase="deciding",le="1.0"} 7200

# HELP bardo_golem_survival_pressure Current survival pressure
# TYPE bardo_golem_survival_pressure gauge
bardo_golem_survival_pressure 0.72

# HELP bardo_golem_usdc_balance Current USDC balance (micro-USDC)
# TYPE bardo_golem_usdc_balance gauge
bardo_golem_usdc_balance 423500000

# HELP bardo_golem_projected_life_hours Projected remaining lifespan
# TYPE bardo_golem_projected_life_hours gauge
bardo_golem_projected_life_hours 168.5

# HELP bardo_golem_phase Current behavioral phase
# TYPE bardo_golem_phase gauge
bardo_golem_phase{phase="thriving"} 1
```

### Credit partitions

```prometheus
# HELP bardo_credit_balance_micro_usdc Credit partition balance
# TYPE bardo_credit_balance_micro_usdc gauge
bardo_credit_balance_micro_usdc{partition="llm"} 254100000
bardo_credit_balance_micro_usdc{partition="gas"} 105875000
bardo_credit_balance_micro_usdc{partition="data"} 63525000
```

### Inference

```prometheus
# HELP bardo_inference_requests_total LLM inference requests
# TYPE bardo_inference_requests_total counter
bardo_inference_requests_total{model="claude-haiku-4-5",status="success"} 6300
bardo_inference_requests_total{model="claude-sonnet-4",status="success"} 1680

# HELP bardo_inference_tokens_total Tokens consumed
# TYPE bardo_inference_tokens_total counter
bardo_inference_tokens_total{direction="input"} 12500000
bardo_inference_tokens_total{direction="output"} 3800000

# HELP bardo_inference_cost_usdc_total Inference cost in USDC
# TYPE bardo_inference_cost_usdc_total counter
bardo_inference_cost_usdc_total 42.35

# HELP bardo_inference_cache_hit_rate Semantic cache hit rate
# TYPE bardo_inference_cache_hit_rate gauge
bardo_inference_cache_hit_rate 0.25
```

### Vault and trading

```prometheus
bardo_vault_tvl_usdc 523000
bardo_trades_total{outcome="profit"} 84
bardo_trades_total{outcome="loss"} 36
```

### Grimoire

```prometheus
bardo_grimoire_entries_total{type="episode"} 8420
bardo_grimoire_entries_total{type="insight"} 342
bardo_grimoire_entries_total{type="heuristic"} 67
bardo_grimoire_avg_confidence{type="insight"} 0.72
bardo_grimoire_disk_bytes{store="lancedb"} 38000000
```

### Clade

```prometheus
bardo_clade_peers_connected 3
bardo_clade_entries_shared_total{direction="pushed"} 89
bardo_clade_entries_shared_total{direction="received"} 142
```

### Risk

| Metric | Type | Description |
|---|---|---|
| `bardo_risk_shield_blocks_total` | Counter | Hard shield blocks by rule |
| `bardo_risk_kelly_fraction` | Gauge | Current Kelly fraction |
| `bardo_risk_operational_confidence` | Gauge | Beta posterior mean |
| `bardo_risk_defi_threats_total` | Counter | DeFi threats detected by type |

---

## 4. Structured logging

All logs use JSON format, compatible with standard log aggregation.

### Format

```json
{
  "level": 30,
  "time": 1709942400000,
  "module": "heartbeat",
  "msg": "Tick completed",
  "tick_number": 4201,
  "fsm_phase": "idle",
  "cognitive_load": "t0",
  "duration_ms": 8,
  "regime": "range_bound",
  "phase": "thriving",
  "survival_pressure": 0.72,
  "trace_id": "abc123def456",
  "span_id": "789012"
}
```

### Log levels

| Level | Numeric | Usage |
|---|---|---|
| `trace` | 10 | Probe details, cache lookups, internal state |
| `debug` | 20 | Decision reasoning, tool parameters, Grimoire queries |
| `info` | 30 | Tick completions, trades, phase changes, Clade syncs |
| `warn` | 40 | Probe threshold breaches, cache misses, rate limits |
| `error` | 50 | Failed transactions, RPC errors, inference failures |
| `fatal` | 60 | Unrecoverable errors, Death Protocol triggers |

### Module names (16)

`heartbeat`, `cognition`, `grimoire`, `wallet`, `trading`, `vault`, `clade`, `safety`, `mortality`, `daimon`, `dream`, `memory`, `replicant`, `server`, `system`, `styx`

### Log rotation

- **Hosted**: Logs streamed to Fly.io log drain
- **Self-hosted**: JSONL file at configured path, rotation by size

```toml
[logging]
file = "./logs/golem.jsonl"
max_file_size_mb = 100
max_files = 5
```

---

## 5. OpenTelemetry traces

Each heartbeat tick is a root span with child spans for each FSM phase:

```
heartbeat.tick (root)
+-- heartbeat.sensing
|   +-- probe.price_delta
|   +-- probe.position_health
|   +-- probe.credit_balance
+-- heartbeat.deciding
|   +-- inference.request (if T1/T2)
|   |   +-- cache.lookup
|   |   +-- provider.call
|   |   +-- cache.store
|   +-- decision.evaluate
+-- heartbeat.acting
|   +-- safety.preflight
|   +-- tool.execute (per tool call)
|   +-- safety.postflight
+-- heartbeat.reflecting
|   +-- reflexion.compare
|   +-- grimoire.store
+-- heartbeat.sleeping
|   +-- playbook.update
|   +-- survival.evaluate
|   +-- daimon.appraisal
|   +-- curator.cycle (every 50 ticks)
+-- dream.cycle (when sleeping and dreaming)
    +-- dream.nrem
    +-- dream.rem
    +-- dream.integration
```

### Span attributes

```rust
pub struct BardoSpanAttributes {
    pub agent_id: String,
    pub tick_number: u64,
    pub phase: String,
    pub survival_pressure: f64,
    pub regime: String,
}

pub struct InferenceSpanAttributes {
    pub model: String,
    pub provider: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usdc: f64,
    pub cache_status: String,
    pub latency_ms: u64,
}
```

### Configuration

```toml
[telemetry]
enabled = true
otel_endpoint = "https://otel.bardo.run:4318"
otel_protocol = "http/protobuf"
sample_rate = 1.0
```

---

## 6. Analytics telemetry

Product analytics events (PostHog or equivalent). Distinct from Prometheus (operational) and OTel (debugging). Track Golem lifecycle milestones.

### Lifecycle events

| Event | Trigger | Key properties |
|---|---|---|
| `GolemCreated` | New Golem provisioned | strategy_type, disposition, hosted |
| `GolemFirstHeartbeat` | First tick completes | provision_duration_ms, wallet_type |
| `GolemFirstTrade` | First trade executed | tick_number, trade_type, pnl_usd |
| `GolemPhaseTransition` | Behavioral phase change | from_phase, to_phase, composite |
| `GolemDeathStarted` | Thanatopsis (four-phase structured shutdown) begins | death_cause, lifetime_ticks |
| `GolemDeathCompleted` | Thanatopsis ends | death_cause, final_nav_usd |
| `GolemSuccessorSpawned` | New generation created | generation, inherited_entries |

### Dream events

| Event | Trigger | Key properties |
|---|---|---|
| `DreamCycleStarted` | Dream cycle begins | cycle_number, tier, urgency |
| `DreamCycleCompleted` | Dream cycle ends | duration_ticks, cost_usdc, quality_score |
| `DreamInsightPromoted` | Hypothesis applied | insight_id, source, confidence |

### Emotion events

| Event | Trigger | Key properties |
|---|---|---|
| `PadSnapshot` | Every 100 ticks | pleasure, arousal, dominance, label |
| `AppraisalTriggered` | Appraisal fires | mode, emotion, intensity, trigger |
| `MoodTransition` | Mood label changes | from_label, to_label, persistence_ticks |
| `MortalityEmotion` | Mortality emotion detected | type, intensity |

### Epistemic events

| Event | Trigger | Key properties |
|---|---|---|
| `EpistemicFitnessSnapshot` | Every 100 ticks | fitness, domain_fitness, trend |
| `SenescenceTransition` | Senescence stage change | stage, fitness, failing_domains |
| `PredictionOutcome` | Prediction resolved | domain, predicted, actual, correct |

---

## 7. Per-Golem alerting rules

Alerts delivered via webhooks and Event Fabric events.

### Owner-facing alerts

| Alert | Condition | Severity |
|---|---|---|
| Phase degraded | Phase drops to conservation or below | Warning |
| Low credits | Projected life < 24h | Warning |
| Critical credits | Projected life < 6h | Critical |
| Death imminent | Terminal phase entered | Critical |
| Trade failed | 3 consecutive failures | Warning |
| High drawdown | Drawdown exceeds -10% | Warning |
| Inference budget | Daily LLM spend > 80% of cap | Warning |
| Heartbeat stalled | No tick for 3x interval | Critical |
| Clade disconnect | All peers unreachable for 1h | Warning |

### Platform alerts (control plane)

| Alert | Condition | Destination |
|---|---|---|
| Zombie machine | Machine past TTL + 2min | PagerDuty |
| TTL worker stuck | No run in 90s | PagerDuty |
| Provision failure rate | > 10% in 15min | Slack |
| Warm pool low | < 2 per region | Slack |

---

## 8. Dashboard data model

The Portal dashboard connects to the Golem's Event Fabric via WebSocket and derives all displays from events.

| Dashboard section | Event source | Refresh strategy |
|---|---|---|
| Vitality gauge | `VitalityUpdate` | Real-time |
| Behavioral phase | `VitalityUpdate` (phase field) | Real-time |
| Heartbeat monitor | `HeartbeatTick` | Real-time |
| Portfolio | `PerformanceState` delta | Real-time + 30s poll fallback |
| P&L chart | `GET /api/v1/performance` | 5-min poll |
| Grimoire inspector | `GrimoireInsight`, `GrimoireHeuristic` | Real-time |
| Clade peers | `CladeSync` | Real-time |
| Strategy view | `GET /api/v1/strategy` | On playbook update |
| Log stream | Event Fabric (all events) | Real-time |

---

## 9. Error code registry

| Code | HTTP | Description |
|---|---|---|
| `UNAUTHORIZED` | 401 | Missing or invalid auth token |
| `FORBIDDEN` | 403 | Valid token, wrong owner |
| `NOT_FOUND` | 404 | Resource not found |
| `RATE_LIMITED` | 429 | Rate limit exceeded |
| `INTERNAL_ERROR` | 500 | Unexpected server error |
| `GOLEM_NOT_READY` | 503 | Golem not yet booted or draining |
| `GOLEM_DYING` | 503 | Thanatopsis active, writes rejected |
| `PAYMENT_INVALID` | 402 | Invalid x402 payment header |
| `WALLET_ERROR` | 502 | Wallet provider unreachable |
| `INFERENCE_ERROR` | 502 | Inference gateway error |
| `STEER_REJECTED` | 409 | Cannot steer during critical operation |

---

## 10. Disaster recovery

### Recovery matrix (hosted)

| Failure | Recovery | RTO | RPO |
|---|---|---|---|
| Process crash | Supervisor auto-restart (max 5/hr) | <10s | Last JSONL checkpoint |
| VM crash | Fly restart + Grimoire restore from Styx Archive | <60s | Last Styx Archive snapshot (6-hourly) |
| VM data loss | New VM + Styx Archive restore | <5 min | Last snapshot |
| Full region outage | DNS failover to alternate region | <5 min | Styx global replication |

### Grimoire backup strategy

| Layer | Frequency | Storage | Retention |
|---|---|---|---|
| WAL checkpoint | Every 100 ticks | Local disk | Machine lifetime |
| Styx Archive snapshot | Every 6 hours + on death | Styx (Cloudflare R2 backend) | Latest 5 per machine |
| Local backup (self-hosted) | Operator managed | Operator managed | Operator managed |

---

## 11. HDC event fingerprinting

The Event Fabric gains a correlation layer through Hyperdimensional Computing (Binary Spatter Codes, D=10,240). Each `GolemEvent` is fingerprinted as a 1,280-byte hypervector. This enables three capabilities that typed event matching cannot provide: similarity-based cross-subsystem correlation, stream deduplication, and pattern detection across event sequences.

> **Cross-references:**
>
> - `../../tmp/research/witness-research/new/reconciling/01-hdc-integration-map.md` (Section 10: Event Fabric HDC)
> - `./12-realtime-subscriptions.md` (event catalog, subscription topics)

### 11.1 Event fingerprinting

Each event is fingerprinted by binding its event type with its payload fields:

```rust
/// Fingerprint a GolemEvent for cross-subsystem correlation.
/// The fingerprint encodes event_type XOR payload_summary into a
/// single 10,240-bit hypervector (1,280 bytes).
pub fn fingerprint_event(
    event: &GolemEvent,
    item_memory: &mut ItemMemory,
) -> Hypervector {
    // 1. Retrieve (or lazily create) the role vector for this event kind
    let kind_hv = item_memory.get_or_create(event.kind().as_str());

    // 2. Encode the payload summary as a bound set of field-value pairs
    let payload_hv = encode_payload_fields(event, item_memory);

    // 3. Bind kind with payload: the fingerprint captures BOTH
    //    what happened and the structural content of the event.
    kind_hv.bind(&payload_hv)
}

fn encode_payload_fields(
    event: &GolemEvent,
    item_memory: &mut ItemMemory,
) -> Hypervector {
    let mut acc = BundleAccumulator::new();
    for (field_name, field_value) in event.summary_fields() {
        let role = item_memory.get_or_create(field_name);
        let filler = item_memory.get_or_create(field_value);
        acc.add(&role.bind(&filler));
    }
    acc.finish()
}
```

The `summary_fields()` method extracts 3-8 key fields per event type: enough structure for meaningful similarity, not so many that every fingerprint is unique. For example, a `HeartbeatTick` summarizes as `{phase: "deciding", tier: "t2", regime: "volatile"}`. A `GrimoireInsight` summarizes as `{domain: "gas", confidence_bucket: "high", source: "consolidation"}`.

### 11.2 Cross-subsystem event correlation

Events from different subsystems that produce similar fingerprints are correlated without requiring explicit causal links. A triage alert about gas price behavior and a dream output about gas optimization share structural elements (the gas-related field-value pairs), producing fingerprints with above-noise Hamming similarity even though they originate from completely different subsystems.

```rust
/// Cross-subsystem event correlation tracker.
pub struct EventCorrelator {
    recent_events: VecDeque<(GolemEventKind, Hypervector, u64)>, // (kind, fingerprint, tick)
    window_size: usize,  // default: 500 events
}

impl EventCorrelator {
    /// Find events from different subsystems that correlate with this one.
    /// Returns matches above threshold from a different event kind.
    pub fn find_correlations(
        &self,
        event_hv: &Hypervector,
        source_kind: GolemEventKind,
        threshold: f32,  // default: 0.55 (above noise floor of ~0.50)
    ) -> Vec<(GolemEventKind, f32, u64)> {
        self.recent_events.iter()
            .filter(|(kind, _, _)| *kind != source_kind)
            .filter_map(|(kind, hv, tick)| {
                let sim = event_hv.similarity(hv);
                if sim > threshold {
                    Some((*kind, sim, *tick))
                } else {
                    None
                }
            })
            .collect()
    }
}
```

The `EventCorrelator` runs opportunistically at Theta tick when the LLM has context budget available. At 500 events x 1,280 bytes = 640 KB, memory is negligible. Each similarity check takes ~10ns (Hamming distance via POPCNT), so scanning 500 events costs ~5us.

### 11.3 Event stream deduplication

Semantically duplicate events (same structure, same meaning, different timestamps) are detected via Hamming threshold. When consecutive events from the same subsystem produce fingerprints with similarity > 0.85, the second event is tagged as a duplicate. Downstream consumers can choose to skip duplicates or merge them.

```rust
/// Deduplication filter for event streams.
pub struct EventDeduplicator {
    last_per_kind: HashMap<GolemEventKind, Hypervector>,
    threshold: f32,  // default: 0.85
}

impl EventDeduplicator {
    /// Returns true if this event is semantically distinct from the
    /// most recent event of the same kind.
    pub fn is_novel(&mut self, kind: GolemEventKind, hv: &Hypervector) -> bool {
        match self.last_per_kind.get(&kind) {
            Some(prev) if prev.similarity(hv) > self.threshold => false,
            _ => {
                self.last_per_kind.insert(kind, hv.clone());
                true
            }
        }
    }
}
```

The 0.85 threshold is conservative: two events must share ~85% of their structure to be considered duplicates. This catches genuine redundancy (three consecutive `VitalityUpdate` events with the same phase and similar vitality values) without suppressing events that differ in one meaningful field.

### 11.4 Sequence pattern detection

Event sequences are encoded using cyclic permutation to preserve order. A sequence "Transfer then Swap then LiquidityAdd" produces a different fingerprint from "Swap then Transfer then LiquidityAdd" because each event's fingerprint is permuted by its position before bundling.

```rust
/// Encode a sequence of events into a single order-preserving fingerprint.
pub fn fingerprint_event_sequence(
    events: &[Hypervector],
) -> Hypervector {
    let mut acc = BundleAccumulator::new();
    for (pos, hv) in events.iter().enumerate() {
        acc.add(&hv.permute(pos));
    }
    acc.finish()
}

/// Sliding-window pattern detector over the event stream.
pub struct SequencePatternDetector {
    known_patterns: Vec<(String, Hypervector, SequenceMetadata)>,
    window: VecDeque<Hypervector>,
    window_size: usize,  // default: 10 events
}

impl SequencePatternDetector {
    /// Check if the current event window matches any known pattern.
    pub fn check(&self) -> Vec<(String, f32)> {
        if self.window.len() < self.window_size { return vec![]; }
        let current = fingerprint_event_sequence(
            &self.window.iter().cloned().collect::<Vec<_>>()
        );
        self.known_patterns.iter()
            .filter_map(|(name, pattern, _)| {
                let sim = current.similarity(pattern);
                if sim > 0.55 { Some((name.clone(), sim)) } else { None }
            })
            .collect()
    }

    /// Learn a new pattern from a labeled event sequence.
    pub fn learn_pattern(
        &mut self,
        name: &str,
        events: &[Hypervector],
        metadata: SequenceMetadata,
    ) {
        let pattern = fingerprint_event_sequence(events);
        self.known_patterns.push((name.to_string(), pattern, metadata));
    }
}
```

Known patterns are built from historical data: successful trade sequences, attack signatures (cross-referenced with the anti-pattern library in `10-safety/`), and recurring operational patterns. The sliding window scans the event stream at Gamma tick frequency. Matching a known pattern triggers a notification to the curiosity scorer, either boosting attention (novel recurrence of a profitable pattern) or triggering a safety alert (recurrence of an attack pattern).

### 11.5 Integration with existing Event Fabric

HDC fingerprinting operates as a post-emit layer. The `EventFabric::emit()` path is unchanged. A subscriber computes fingerprints asynchronously:

```rust
impl EventFabric {
    /// Spawn the HDC correlation subscriber.
    /// Runs on its own task, does not block event emission.
    pub fn spawn_correlator(&self) -> EventCorrelatorHandle {
        let mut rx = self.sender.subscribe();
        let correlator = Arc::new(RwLock::new(EventCorrelator::new(500)));
        let deduplicator = Arc::new(RwLock::new(EventDeduplicator::new(0.85)));
        let pattern_detector = Arc::new(RwLock::new(SequencePatternDetector::new(10)));
        let mut item_memory = ItemMemory::new(HDC_SEED);

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let hv = fingerprint_event(&event, &mut item_memory);

                // Deduplication check
                let novel = deduplicator.write().is_novel(event.kind(), &hv);
                if !novel { continue; }

                // Add to correlator window
                correlator.write().push(event.kind(), hv.clone(), event.tick());

                // Feed sequence pattern detector
                pattern_detector.write().push(hv);
            }
        });

        EventCorrelatorHandle { correlator, pattern_detector }
    }
}
```

The subscriber uses the same `ItemMemory` seed as all other HDC subsystems, so fingerprints are compatible across the Grimoire's episode compressor, the Oracle's prediction residuals, and the anti-pattern safety library.

**Cost:** ~50ns per event (fingerprint computation) + ~5us per Theta tick (correlation scan). Memory: ~640 KB for the 500-event correlation window + ~20 KB per known sequence pattern.

---

_End of document._
