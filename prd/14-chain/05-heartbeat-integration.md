# Heartbeat Integration [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crates**: `bardo-witness`, `bardo-triage`, `bardo-protocol-state`, `bardo-chain-scope`

---

> **Reader orientation:** This document specifies how the chain intelligence layer (section 14) integrates with the Golem's Heartbeat (the 9-step decision cycle: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect). It maps chain intelligence operations to Gamma (~250ms), Theta (~2s), and Delta (~30s) tick levels, defining what computation runs at each cadence. The key concept is that block ingestion is continuous and ungated, but the Golem's cognitive response is clock-gated. See `prd2/shared/glossary.md` for full term definitions.

Block ingestion (`bardo-witness`) runs continuously in its own task -- it is not clock-gated. But the golem's cognitive response to chain events follows the heartbeat clocks (see [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- Gamma/Theta/Delta clock definitions and cognitive tier routing). This document specifies what chain intelligence adds to each tick level.

The principle: chain data is always flowing. The heartbeat clocks determine when the golem thinks about what it saw.

---

## Gamma Tick Additions (~250ms)

The Gamma tick handles fast perception -- cheap reads and arithmetic. Chain intelligence adds five steps here.

### `gamma_chain_perception()` -- New Function

Called in the Gamma pipeline after existing market observation steps:

```rust
async fn gamma_chain_perception(ctx: &GolemContext) {
    // 1. Update CorticalState.chain_blocks_behind.
    //    Already done continuously by bardo-witness on each block,
    //    but this ensures CorticalState is current at cognition time.
    let chain_head = ctx.witness
        .latest_chain_block_number(ctx.chain_id);
    let processed = ctx.triage
        .latest_processed_block_number();
    let behind = chain_head.saturating_sub(processed);
    ctx.cortical_state.chain_blocks_behind
        .store(behind as u32, Ordering::Relaxed);

    // 2. Rebuild ChainScope from updated CorticalState + recent Grimoire.
    //    This is the cybernetic feedback step: what the golem cares about
    //    determines what it watches next.
    //    Uses dirty flag to skip full rebuild when nothing changed.
    ctx.chain_scope
        .rebuild(&ctx.cortical_state, &ctx.grimoire)
        .await;

    // 3. Check protocol state freshness.
    //    Force-refresh any protocols that haven't been updated within their
    //    expected interval (catches events missed due to gaps).
    ctx.protocol_state
        .check_and_schedule_refreshes(ctx.heartbeat.tick_count)
        .await;

    // 4. Drain medium-score triage events (score 0.5-0.8)
    //    into the perception buffer.
    //    High-score events (>0.8) were already queued directly by triage.
    let events = ctx.triage.drain_medium_score(GAMMA_BATCH_MAX);
    ctx.perception_buffer.extend(events);

    // 5. Update CorticalState.attention.active_count.
    let active = ctx.protocol_state
        .active_protocol_count(GAMMA_WINDOW_MS);
    ctx.cortical_state.active_count
        .store(active as u16, Ordering::Relaxed);
}
```

**Cost**: Near-zero. All operations are atomic reads/writes, in-memory lookups, or bounded-size iterations. No RPC calls unless `check_and_schedule_refreshes` finds staleness, in which case it queues eth_calls that execute in the background.

### Chain Lag Suppression

Before the Gamma tick completes, the existing prediction error check gains one additional suppression condition:

```rust
// Existing: suppress if pe < threshold (T0, nothing interesting happened)
// New: also suppress write actions if chain is lagging
let chain_lag = ctx.cortical_state.chain_blocks_behind
    .load(Ordering::Relaxed);

if chain_lag > STALENESS_THRESHOLD {
    // Still run T0 (perception + protocol state maintenance).
    // Suppress T1/T2 (deliberation + trading) until chain catches up.
    observation.chain_lag_suppression = true;
}
```

`STALENESS_THRESHOLD`: 3 blocks on mainnet (45s), 10 blocks on L2s with faster block times. The suppression is a hint, not a hard block -- the LLM may still choose to act if it determines the chain lag is acceptable for the specific action.

---

## Theta Tick Additions (~2s)

The Theta tick handles deliberation -- LLM calls are gated here. Chain intelligence adds one step: processing the LLM escalation queue from triage.

### `theta_chain_cognition()` -- New Function

Called during the Theta pipeline if `CognitiveTier >= T1`:

```rust
async fn theta_chain_cognition(ctx: &GolemContext) {
    // Take queued high-score events from triage (score > 0.8 events that
    // bardo-triage emitted TriageAlert for but haven't been LLM-analyzed yet).
    let queued = ctx.triage.take_llm_queue(MAX_THETA_CHAIN_EVENTS);
    if queued.is_empty() { return; }

    for event in queued {
        // Analyze with full cognitive context.
        let analysis = ctx.inference.analyze_chain_event(
            &event,
            ctx.cortical_state.snapshot(),
            &ctx.active_strategy,
            &ctx.grimoire.recent_chain_events(5),
        ).await?;

        // Store as Grimoire episode -- this is the learning step.
        let embedding = ctx.encoder.encode(&analysis.summary).await?;
        // Store in the Grimoire (persistent knowledge base: episodes, insights, heuristics)
        ctx.grimoire.store_episode(GrimoireEpisode {
            category: EpisodeCategory::ChainEvent,
            content: serde_json::to_value(&analysis)?,
            embedding,
            emotional_tag: EmotionalTag::from_pad(
                ctx.cortical_state.read_pad()
            ),
            importance: analysis.importance_score,
            source: EpisodeSource::ChainTriage,
            ..Default::default()
        }).await?;

        // Insert into ANN staging index immediately.
        // Queryable at next Gamma tick, not next Delta.
        if let Some(fp) = &event.fingerprint {
            ctx.triage.curiosity_scorer
                .ann_index
                .insert_episode(event.episode_id, &embedding);
        }

        // Apply reinforcement to curiosity model.
        if analysis.golem_should_act {
            // Positive: this kind of tx is actionable.
            if let Some(hv) = &event.hypervector {
                ctx.triage.curiosity_scorer
                    .hdc_codebook
                    .positive_reinforce(hv);
            }
            ctx.triage.curiosity_scorer
                .thompson
                .update(0, true);  // escalation was correct
        } else {
            // Negative: this kind of tx is noise for this golem's strategy.
            if let Some(hv) = &event.hypervector {
                ctx.triage.curiosity_scorer
                    .hdc_codebook
                    .negative_reinforce(hv);
            }
            ctx.triage.curiosity_scorer
                .thompson
                .update(0, false);  // escalation was wrong
        }

        // Update Hedge weights based on signal accuracy.
        ctx.triage.curiosity_scorer.update_weights(
            &event.signal_scores,
            analysis.importance_score,
        );

        // If LLM recommends action, inject into normal Theta action pipeline.
        if let Some(action) = analysis.recommended_action {
            ctx.action_queue.push(action);
        }
    }
}
```

**Cost**: One LLM call per queued event. High-score events are rare (score > 0.8) -- typical rate is 0-5 per Theta tick on mainnet. Cost per call: T1 (~$0.001) or T2 (~$0.01) depending on whether the event warrants full deliberation.

**MAX_THETA_CHAIN_EVENTS**: 10 per Theta tick. Prevents a burst of high-score triage events from monopolizing the Theta budget.

### Epsilon-Greedy Exploration

With probability epsilon, the Theta tick also processes a random low-score event that would normally be discarded:

```rust
// Epsilon-greedy: occasionally evaluate events we'd normally ignore.
// This provides counterfactual labels -- the golem can learn that
// it's discarding interesting events.
let epsilon = 0.05 * (1.0 / (1.0 + ctx.triage.total_evaluations() as f64 / 10000.0));
if rng.gen::<f64>() < epsilon {
    if let Some(random_event) = ctx.triage.sample_low_score_event() {
        // ... same analysis as above, but with lower priority
    }
}
```

---

## Delta Tick Additions (~30s)

The Delta tick handles consolidation -- slow maintenance work. Chain intelligence adds six steps.

### `delta_chain_consolidation()` -- New Function

```rust
async fn delta_chain_consolidation(ctx: &GolemContext) {
    // 1. Compact triage event store.
    //    Delete entries beyond retention window.
    // BehavioralPhase: one of five survival phases (Thriving/Stable/Conservation/Desperate/Terminal)
    let retention = match ctx.cortical_state.phase() {
        BehavioralPhase::Thriving | BehavioralPhase::Stable => {
            TRIAGE_RETENTION_DEFAULT  // 7 days
        }
        BehavioralPhase::Conservation => {
            TRIAGE_RETENTION_DEFAULT / 2
        }
        BehavioralPhase::Declining | BehavioralPhase::Terminal => {
            TRIAGE_RETENTION_DEFAULT / 4
        }
    };
    ctx.triage_store.compact(retention).await;

    // 2. Merge ANN staging index if threshold reached.
    //    The staging index accepts inserts at Theta tick;
    //    merge into main when ~500 new episodes accumulate.
    let staging_count = ctx.triage.curiosity_scorer
        .ann_index.staging_count
        .load(Ordering::Relaxed);
    if staging_count >= ANN_STAGING_MERGE_THRESHOLD {
        ctx.triage.curiosity_scorer
            .ann_index.merge_staging();
    }

    // 3. Persist ChainScope interest entries and seen-blocks bitmap.
    ctx.chain_scope.persist(&ctx.redb).await;
    ctx.witness.persist_seen_blocks(&ctx.redb).await;

    // 4. Apply score decay and prune stale entries from interest list.
    //    (Decay is also applied per-entry at each Gamma rebuild, but
    //    Delta does the full pruning pass including garbage collection.)
    ctx.chain_scope.apply_decay_and_prune().await;

    // 5. Update rindexer config if new protocols discovered since last Delta.
    if ctx.protocol_registry.has_new_since(ctx.last_delta_tick) {
        ctx.rindexer.update_config(&ctx.protocol_registry).await;
    }

    // 6. Reset HyperLogLog cardinality sketch for next window.
    ctx.chain_scope.reset_cardinality_sketch();
}
```

**Cost**: Disk I/O (redb writes) + ANN merge if triggered. The ANN merge dominates at ~100ms for 10,000 Grimoire episodes. Acceptable within Delta cadence.

### Staggered Delta Compaction

When multiple chains are monitored, offset each chain's Delta tick by `chain_index * delta_interval / num_chains`. For 5 chains with a 20-minute Delta interval, the offsets are 0, 4, 8, 12, 16 minutes. This spreads I/O load evenly across the Delta interval instead of creating I/O storms where all chains write to Grimoire, run BOCPD, and merge DDSketches simultaneously.

---

## Updated Tick Cost Table

| Scale | Interval | Chain intelligence cost |
|-------|----------|------------------------|
| **Gamma** | ~250ms | ~0 (atomic reads, in-memory DashMap, filter rebuild) |
| **Theta** | ~2s | T1 $0.001-$0.01 per high-score event (0-10 events/tick) |
| **Delta** | ~30s | ~100ms ANN merge + redb I/O. $0 inference cost. |

---

## Chain Lag and Heartbeat Suppression

When `chain_blocks_behind > STALENESS_THRESHOLD`:

1. **Gamma** still runs normally -- perception, scope rebuild, and protocol state maintenance all work fine with stale data.
2. **Theta** T1/T2 deliberation is suppressed for actions that require fresh chain state:
   - Suppressed: swaps, LP rebalancing, vault deposits/withdrawals.
   - Not suppressed: Grimoire curation, dream scheduling, knowledge operations.
3. **Delta** runs normally -- consolidation doesn't depend on fresh chain state.

The suppression is a hint, not a hard block -- the LLM may still choose to act if it determines the chain lag is acceptable for the specific action (e.g., a slow rebalancing operation where a few missed blocks don't matter).

---

## Cross-References

- **Architecture**: [00-architecture.md](./00-architecture.md) -- Overall data flow and cybernetic feedback loop for the chain intelligence layer
- **Triage**: [02-triage.md](./02-triage.md) -- Curiosity scorer whose signals are updated by Theta tick feedback (positive/negative reinforcement)
- **Chain scope**: [04-chain-scope.md](./04-chain-scope.md) -- Gamma-tick rebuild details, dirty flag optimization, arousal-modulated decay
- **Events**: [06-events-signals.md](./06-events-signals.md) -- `chain_blocks_behind` CorticalState signal spec, all GolemEvent variants
- **Heartbeat**: [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- Gamma/Theta/Delta clock definitions, cognitive tier routing (T0/T1/T2)

---

## References

- Friston, K. et al. (2010). The free-energy principle: a unified brain theory? *Nature Reviews Neuroscience*, 11(2), 127-138. -- *Grounds the active inference loop between chain perception and Golem action that these tick hooks implement.*
- Parr, T., Pezzulo, G. & Friston, K. (2022). *Active Inference: The Free Energy Principle in Mind, Brain, and Behavior*. MIT Press. -- *Textbook treatment of active inference; the Theta-tick reinforcement loop follows this framework.*
- Shalev-Shwartz, S. (2011). Online Learning and Online Convex Optimization. *Foundations and Trends in Machine Learning*, 4(2), 107-194. -- *Theoretical foundation for the Hedge algorithm that weights triage scoring signals at Theta tick.*
