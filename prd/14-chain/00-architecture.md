# Chain Intelligence Architecture [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crates**: `golem-witness`, `golem-triage`, `golem-protocol-state`, `golem-chain-scope`, `golem-stream-api`

---

> **Reader orientation:** This document is the architectural overview for Bardo's chain intelligence layer (section 14). It describes the five-crate pipeline that gives a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) continuous awareness of on-chain activity, from block ingestion through event classification to external streaming. The key concept here is the cybernetic feedback loop: the Golem's own behavior determines what it watches, and what it watches shapes its behavior. See `prd2/shared/glossary.md` for full term definitions.

The chain intelligence system is a perpetual surveillance and state management layer that runs from golem boot until death. It is not triggered by tool calls. It is not session-scoped. It sits below the tool layer, feeding the golem's cognitive loop and the terminal's Sanctum view with a continuous stream of classified, scored, and routed on-chain activity.

Three forces drive the design:

1. A golem on a chain should see everything happening on it, filtered through an attention model it continuously refines.
2. DeFi protocols change every block. The golem needs a live internal model that tracks those changes as they happen, not by polling on demand.
3. When a user opens the Sanctum view, they should see the chain through the golem's eyes -- from its connection, authenticated via its identity, streaming from its internal state.

This document maps the five-crate architecture, the data flow from WebSocket to storage, the cybernetic feedback loop that closes attention into action, and the storage budget that keeps it all bounded.

---

## Five Crates

```
golem-witness         -- perpetual block subscription, Binary Fuse pre-screening
golem-triage          -- transaction intelligence pipeline (4-stage)
golem-protocol-state  -- live protocol state cache + autonomous discovery
golem-chain-scope     -- dynamic attention model (what to watch next block)
golem-stream-api      -- authenticated external HTTP/WS/SSE streaming
```

Each crate exports `serve(config, fabric_handle) -> JoinHandle`. The `EventFabricHandle` trait has two implementations: `EmbeddedFabric` (direct `tokio::sync::broadcast::Sender<GolemEvent>`) and `IpcFabric` (Unix socket). Swapping handles moves the entire subsystem from embedded to standalone without touching subsystem code.

The default deployment runs all five crates embedded in the golem VM's Tokio runtime, sharing its memory and event bus. If operational requirements demand it (separate resource quotas, chain surveillance as a shared service across a Clade (sibling Golems sharing a common ancestor; exchange knowledge through Styx)), they extract as a standalone `golem-chain` binary.

### Deployment Modes

| Mode | Description | Use case |
|------|-------------|----------|
| **Full** | All five crates, embedded in golem VM | Default for single-golem deployments |
| **Light** | Witness + triage + chain-scope only | Resource-constrained L2 golems |
| **Archive** | Full + rindexer integration + extended retention | Golems that need deep historical queries |

---

## Data Flow

```
[Ethereum Node]
    | WS eth_subscribe("newHeads")
    v
[golem-witness]
    -- header arrives
    -- Binary Fuse filter check (O(1), ~10ns via POPCNT)
    ---- miss -> update gas_gwei, continue (>90% of blocks)
    ---- hit  -> eth_getBlockByHash + eth_getBlockReceipts
    v
[golem-triage]
    -- Stage 1: Rule-based fast filters (known MEV patterns, value thresholds)
    -- Stage 2: Statistical anomaly detection (MIDAS-R, DDSketch)
    -- Stage 3: Contextual enrichment (protocol state, ABI resolution, history)
    -- Stage 4: Upgraded scoring (HDC fingerprint + Bayesian surprise + ANN + Thompson)
    v (fan-out by curiosity score)
    |-- score > 0.8  -> GolemEvent::TriageAlert + LLM queue
    |-- 0.5-0.8      -> GolemEvent::ChainEvent + protocol state update
    |-- 0.2-0.5      -> protocol state update (silent)
    +-- < 0.2        -> discard (written to redb for audit)

[golem-protocol-state]
    -- receives routed triage events
    -- reads state via eth_call (parallel tokio::join!)
    -- atomic-swap DashMap hot layer
    -- persist delta to redb warm layer
    -- emit GolemEvent::ProtocolStateUpdate
    -- autonomous discovery: factory events -> new contracts -> ABI resolution

[golem-chain-scope]  <- rebuilt each Gamma tick from CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model)
    -- scored interest list -> xorf::BinaryFuse8 filter
    -- -> fed back into golem-witness (what to watch next block)
    -- -> CorticalState.attention.universe_size, .active_count updated

[CorticalState]
    -- attention.universe_size  (HyperLogLog estimate, ChainScope)
    -- attention.active_count   (protocols with recent updates)
    -- environment.gas_gwei     (updated every block header, even on miss)
    -- chain.blocks_behind      (staleness indicator)

[Event Fabric]  <- all chain events flow through here
    -> TUI WebSocket -> Sanctum view (protocol.* topic)
    -> golem-stream-api -> external WS/SSE consumers

[golem-stream-api]
    -- HTTP: snapshot + history endpoints
    -- WS: multiplexed, topic-filtered, resume-from-sequence
    -- SSE: simpler clients / browser
    -- Auth: same SIWE/JWT as golem API
```

---

## The Cybernetic Feedback Loop

The chain scope, triage pipeline, and golem cognition form a closed feedback loop. No human configuration of watched addresses or events is required after initial strategy setup. The golem's own behavior determines what it watches.

```
What golem cares about (CorticalState + positions + strategy)
    |
    v
ChainScope interest list (rebuilt each Gamma tick)
    |
    v
BinaryFuse8 filter (what to screen blocks for)
    |
    v
golem-witness filter check (which blocks to fetch)
    |
    v
golem-triage (which transactions are relevant)
    |
    v
Events reach golem cognition (Theta tick)
    |
    v
Golem acts -> Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) episode created
    |
    v
Delta tick: ANN staging index updated from chain_event episodes
    |
    v
Curiosity model improves -> different scoring
    |
    v
ChainScope updated by CorticalState changes
    ^ (back to top)
```

This is an instance of the active inference loop described by Friston et al. (2010): the golem maintains a generative model of its chain environment (the curiosity scorer's learned representations), acts to reduce uncertainty (the triage pipeline's attention allocation), and updates its model based on the outcomes of its actions (the reinforcement loop through Grimoire episodes). The prediction error signal at the Gamma gate is the variational free energy that drives the entire cycle.

### Oracle Integration

Bayesian surprise feeds INTO the Oracle (see [01-golem/17-prediction-engine.md](../01-golem/17-prediction-engine.md) -- the Golem's prediction engine and attention forager) as a `PredictionDomain`, not as a parallel prediction system. The Oracle is canonical for prediction error computation and action gating. The KL divergence output from Bayesian surprise models becomes one of the Oracle's prediction domains, and the Oracle's residual correction mechanism handles non-stationarity. This avoids maintaining two parallel per-protocol statistical models that produce contradictory attention signals.

### Event-Driven Perception

Block ingestion (`golem-witness`) runs continuously in its own Tokio task -- it is not clock-gated. Rather than running perception on a fixed Gamma timer (part of the Heartbeat, the 9-step decision cycle: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect), use block-arrival-triggered perception:

```rust
select! {
    block = any_chain_receiver.recv() => { run_triage(block); }
    _ = theta_timer.tick() => { run_cognition(); }
    _ = delta_timer.tick() => { run_consolidation(); }
}
```

This eliminates latency on fast chains (Base at 2s blocks would batch 2-3 blocks per 5s Gamma tick otherwise) and wasted cycles on slow chains. The Gamma oscillator remains useful as a heartbeat health check (emit `GolemEvent::HeartbeatTick` even if no blocks arrived), but it does not gate perception. ChainScope rebuilds and CorticalState updates still run at Gamma cadence.

---

## Per-Chain Triage Isolation

Each chain gets its own `TriageEngine` instance with its own MIDAS-R, DDSketch, and Count-Min Sketch. Cross-chain signals flow through the event fabric, not through shared mutable state. This eliminates Mutex contention at aggregate throughput across multiple chains.

Memory overhead per chain: MIDAS-R ~128KB, DDSketch ~2KB, CMS ~32-64KB. For 10 chains, ~1.6MB total.

### ChainAdapter Trait for Non-EVM Chains

The chain intelligence layer is EVM-specific by default: it filters on log addresses and topics, decodes Solidity ABI, uses `eth_getBlockByNumber`. A `ChainAdapter` trait abstracts the chain-specific operations:

```rust
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    /// Fetch a block and its events, chain-specific.
    async fn fetch_block(&self, block_id: BlockId) -> Result<NormalizedBlock>;

    /// Extract filter keys from a block header for pre-screening.
    /// EVM: address + topics. Solana: program ID + account addresses.
    /// Cosmos: message type + sender.
    fn extract_filter_keys(&self, header: &BlockHeader) -> Vec<u64>;

    /// Decode events from raw transaction data.
    /// EVM: ABI decoding. Solana: Anchor IDL. Cosmos: protobuf.
    fn decode_events(&self, raw: &RawTxData) -> Vec<DecodedEvent>;

    /// Read protocol state from chain.
    /// EVM: eth_call. Solana: getAccountInfo. Cosmos: ABCI query.
    async fn read_state(&self, address: &ChainAddress) -> Result<RawState>;
}
```

The Binary Fuse filter concept works across all chains because it operates on u64 hashes of filter keys, not the keys themselves. HDC fingerprinting needs chain-specific role vocabularies (different field names for Solana vs. EVM transactions) but no algorithmic changes.

### Adaptive Sampling for Passive Chains

A golem monitoring 10 chains but holding positions on only 2 should not process every block on all 10 with equal intensity.

```rust
pub enum ChainIngestionMode {
    /// Process every block, every transaction.
    /// Used for chains with active positions.
    Full,

    /// Process a random 10% of blocks, stratified by gasUsed
    /// (prefer high-gas blocks, more likely to contain DeFi activity).
    /// Used for chains being monitored without active positions.
    Sampled,

    /// Subscribe only to events matching the Binary Fuse filter.
    /// Skip full block processing.
    /// Used for chains where the golem wants to know about specific
    /// contracts but doesn't need full surveillance.
    EventOnly,
}
```

Mode transitions are driven by position changes: opening a position on a chain promotes it from `Sampled` to `Full`. Closing the last position demotes it. Sampling uses reservoir sampling (Vitter, 1985) to maintain statistical representativeness.

### Hierarchical Attention Allocation

Compute per chain is proportional to position value on that chain. A golem with $100K on Ethereum and $1K on Arbitrum spends roughly 100x more compute on Ethereum.

Weight triage compute per chain by `position_value_usd / total_position_value_usd`. This maps to rational inattention (Sims, 2003): an agent with finite processing capacity optimally allocates attention proportional to the stakes. The adaptive clock's Gamma interval per chain scales inversely with the chain's attention weight.

### Staggered Delta Compaction

When multiple chains are monitored, offset each chain's Delta tick by `chain_index * delta_interval / num_chains`. For 5 chains with a 20-minute Delta interval, the offsets are 0, 4, 8, 12, 16 minutes. This spreads I/O load (Grimoire writes, BOCPD runs, DDSketch merges) evenly across the Delta interval instead of creating I/O storms.

---

## Storage Budget (Mainnet, Per Chain)

| Component | Per day | 7-day default | 30-day |
|-----------|---------|---------------|--------|
| Triage filtered events | 2-8 MB | 14-56 MB | 60-240 MB |
| Protocol state (snapshots) | 0.5 MB | 3.5 MB | 15 MB |
| Protocol state (deltas) | 0.2 MB | 1.4 MB | 6 MB |
| Seen block bitmap (Roaring) | ~2 KB | ~14 KB | ~60 KB |
| ANN embedding index (HNSW) | Grimoire-size | -- | ~5 MB |
| HDC codebook + bundles | ~200 KB | ~200 KB | ~200 KB |
| MIDAS-R sketch | ~100 KB | ~100 KB | ~100 KB |
| DDSketch distributions | ~50 KB | ~50 KB | ~50 KB |
| **Total per chain** | **~3-9 MB/day** | **~20-62 MB** | **~90-270 MB** |

Assumptions: ~100 protocols, ~1,000 watched addresses, ~7,500 blocks/day, ~10% filter hit rate. L2 golems are 10-50x smaller (fewer protocols, smaller blocks). If the storage cap is hit, triage retention halves automatically.

### Storage Engine Allocation

| Engine | Data | Rationale |
|--------|------|-----------|
| **redb** | Protocol state snapshots, deltas, triage events, seen-block bitmap | Read-optimized B-tree, good for point lookups and range scans. Hot/warm protocol state needs fast reads for the 60fps TUI render loop. |
| **fjall** | Write-heavy event streams, MIDAS-R checkpoints | LSM-tree optimized for high write throughput. Triage events arrive at block rate and are mostly append-only. |
| **usearch** | HNSW ANN index (main + staging) | Purpose-built for approximate nearest neighbor search with SIMD acceleration. Binary HNSW mode for HDC hypervectors. |

---

## Cross-References

- **Witness layer**: [01-witness.md](./01-witness.md) -- WebSocket block ingestion, Binary Fuse pre-screening against logsBloom, reconnection and gap handling
- **Triage pipeline**: [02-triage.md](./02-triage.md) -- 4-stage transaction classification pipeline: rule-based filters, MIDAS-R anomaly detection, HDC fingerprinting, Bayesian surprise, Thompson sampling
- **Protocol state**: [03-protocol-state.md](./03-protocol-state.md) -- Live protocol state cache with hot/warm/cold storage layers, autonomous protocol discovery from factory events, ABI resolution chain
- **Chain scope**: [04-chain-scope.md](./04-chain-scope.md) -- Dynamic attention model determining what the Golem watches: interest entries, HyperLogLog cardinality, arousal-modulated decay
- **Heartbeat integration**: [05-heartbeat-integration.md](./05-heartbeat-integration.md) -- How chain intelligence hooks into the Gamma/Theta/Delta tick pipeline, chain lag suppression logic
- **Events and signals**: [06-events-signals.md](./06-events-signals.md) -- 5 GolemEvent variants emitted by chain intelligence, CorticalState `chain_blocks_behind` signal spec
- **Generative views**: [07-generative-views.md](./07-generative-views.md) -- Auto-generated TUI protocol views from contract ABIs, template-then-LLM refinement pipeline
- **Stream API**: [08-stream-api.md](./08-stream-api.md) -- External WebSocket/SSE streaming endpoints for chain events, SIWE/JWT authentication, rate limiting
- **Anomaly detection**: [09-anomaly-detection.md](./09-anomaly-detection.md) -- MIDAS-R, BOCPD, PELT, and ADWIN algorithms for streaming anomaly and changepoint detection
- **Heartbeat system**: [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- Gamma/Theta/Delta clock definitions, cognitive tier routing (T0/T1/T2)
- **Oracle / Prediction engine**: [01-golem/17-prediction-engine.md](../01-golem/17-prediction-engine.md) -- Oracle prediction domains, attention forager, residual correction mechanism
- **Event catalog**: [shared/event-catalog.md](../shared/event-catalog.md) -- Full GolemEvent variant list across all subsystems
- **HDC/VSA shared reference**: [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- BSC algebra, codebook management, ANN integration for hyperdimensional computing

---

## References

- Bloom, B.H. (1970). Space/time trade-offs in hash coding with allowable errors. *Communications of the ACM*, 13(7). -- *Introduces the Bloom filter, the probabilistic data structure underlying Ethereum's logsBloom and informing the Binary Fuse filter design used here.*
- Friston, K. et al. (2010). The free-energy principle: a unified brain theory? *Nature Reviews Neuroscience*, 11(2), 127-138. -- *Proposes the free-energy principle for biological agents; grounds the cybernetic feedback loop between chain perception and Golem action.*
- Lemire, D. et al. (2022). Binary Fuse Filters: Fast and Smaller Than Xor Filters. *Journal of Experimental Algorithmics*. -- *Describes the immutable, rebuild-oriented probabilistic filter used by golem-witness for O(1) block pre-screening at 8.7 bits/entry.*
- Sims, C.A. (2003). Implications of rational inattention. *Journal of Monetary Economics*, 50(3), 665-690. -- *Formalizes how agents with finite processing capacity optimally allocate attention proportional to stakes; justifies the hierarchical attention allocation model.*
- Vitter, J.S. (1985). Random sampling with a reservoir. *ACM Transactions on Mathematical Software*, 11(1), 37-57. -- *Provides the reservoir sampling algorithm used for statistically representative block sampling on passive chains.*
- Wood, G. (2014). Ethereum Yellow Paper. Section 4.3 (logsBloom definition). -- *Defines the 2048-bit Bloom filter in every Ethereum block header that the witness layer tests against.*
