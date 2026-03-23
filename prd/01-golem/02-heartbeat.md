# Heartbeat: The Tick Pipeline [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-heartbeat` (pipeline.rs, record.rs, gating.rs, fsm.rs, speculation.rs)
>
> **Prerequisites**: Read `00-overview.md` (glossary, system overview), `01a-runtime-extensions.md` (extension system), `01b-runtime-infrastructure.md` (GolemState, lifecycle).
>
> **Academic basis**: CoALA (Sumers et al., 2024) -- cognitive architecture framework; Baddeley (2000) -- working memory and the episodic buffer; Clark & Chalmers (1998) -- extended mind thesis; Cognitive Workspace (arXiv:2508.13171, 2025) -- active memory management; Kahneman (2011) -- dual-process theory (System 1 / System 2); Friston (2010) -- precision-weighted prediction error and active inference; Sims (2003) -- rational inattention; Newell (1990) -- Unified Theories of Cognition

> **Reader orientation:** This document specifies the Heartbeat (the 9-step decision cycle each Golem executes on every tick): observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect. It belongs to the `01-golem` cognition layer. The key concept: the Golem's cognition is a continuous autonomous loop, not a conversational turn. The Heartbeat is where the T0/T1/T2 (cognitive tier routing: T0 = fast cached/rule-based, T1 = medium LLM, T2 = extended reasoning) gating decision happens, making ~80% of ticks free. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## S1 -- Why Decision Cycles, Not Conversation Turns

### The Conversation Assumption Is Wrong

Most agent frameworks model cognition as a conversation: user says something, agent thinks, agent responds, repeat. This model comes from chatbot heritage -- LLMs were first deployed as conversational interfaces, and agent frameworks inherited that frame.

For a Golem, this model is wrong on its face. **80% of heartbeat ticks have no human input.** The Golem fires its heartbeat, observes the market, evaluates whether anything interesting happened, and either acts or moves on. There is no "user message." There is no "response." There is a continuous loop of observe-decide-act-learn running autonomously.

### CoALA Formalizes the Correct Primitive

Sumers et al. (2024) proposed Cognitive Architectures for Language Agents (CoALA), a framework that draws on decades of cognitive architecture research (Soar [LAIRD-2012], ACT-R [ANDERSON-2007]) to formalize what a language agent IS [SUMERS-2024].

CoALA's key insight is the **decision cycle**: "The agent's decision procedure executes a decision cycle in a loop with the external environment. During each cycle, the agent uses retrieval and reasoning to plan by proposing and evaluating candidate learning or grounding actions. The best action is then selected and executed. An observation may be made, and the cycle begins again."

This maps precisely to the Golem's heartbeat. Each tick IS a decision cycle: observe, retrieve from memory, reason about what was observed, decide on an action (or decide to do nothing), execute, observe the outcome, learn.

The distinction matters architecturally:

| Aspect | Conversation Turn | Decision Cycle |
|--------|------------------|----------------|
| Trigger | User message | Timer (heartbeat interval) |
| Input | Natural language text | Structured market observation |
| Output | Natural language response | Typed DecisionCycleRecord |
| Storage | Append to session JSONL | Self-contained record per tick |
| Replay | Parse message history | Structured fields, zero parsing |
| Credit assignment | Parse LLM output text | Read outcome + context fields directly |

### The DecisionCycleRecord Replaces Session Messages

Instead of appending messages to a growing session, each tick produces a **DecisionCycleRecord** -- a typed, self-contained record of everything that happened during that tick. This record is:

- **The unit of dream replay.** NREM replay selects episodes by `utility = gain * need` [WILSON-MCNAUGHTON-1994]. The record IS the episode -- observation, action, outcome, emotional state, and regime are already structured fields. No extraction step needed.

- **The unit of credit assignment.** The cybernetics self-tuning loop (Loop 1) traces outcomes to the context entries that contributed to the decision. The record's `context_bundle_summary` tells exactly which Grimoire entries were in context, and `outcome` tells whether the result was positive. In a conversation-based agent, this requires parsing LLM output text.

- **The unit of mortality accounting.** Every tick records `vitality`, `phase`, and `credit_consumed`. The lifespan extension doesn't maintain separate counters -- the decision record IS the mortality audit trail.

- **The source of Event Fabric events.** Every field in the record maps to a typed GolemEvent. The `ui-bridge` extension translates record fields to events with no additional logic.

### Conversation Is Preserved for the Minority Case

When a user chats with their Golem (the ~20% of ticks with human interaction), the conversation is stored in a separate JSONL file (`$GOLEM_DATA/conversations/session.jsonl`) that follows the session format. This "Session Sidecar" handles conversation branching, compaction, and message history independently of the heartbeat pipeline.

The two systems coexist:
- **Heartbeat path**: DecisionCycleRecords -> stored as bincode -> used for dreams, credit assignment, mortality
- **Conversation path**: Session JSONL -> stored as JSONL -> used for user chat continuity

When the user IS chatting, the conversation tail (last N messages) is injected into the Cognitive Workspace as one of its structured categories. The conversation informs the heartbeat, but the heartbeat doesn't become a conversation.

---

## S2 -- LLM-Last as Dual Process Architecture

Daniel Kahneman's Dual Process Theory (_Thinking, Fast and Slow_, 2011) provides the cognitive model. But it is more than metaphor here -- it is the cost model. System 1 is free. System 2 costs money. Mortality (see [02-mortality/](../02-mortality/)) makes that tradeoff existential.

| Aspect | System 1 (Fast) | System 2 (Slow) |
|--------|-----------------------------------|---------------------------------------------|
| Implementation | Deterministic FSM + Rust probes | LLM inference (Haiku / Sonnet / Opus) |
| Cost per tick | $0.00 | $0.001--$0.25 |
| Latency | <10ms | 500ms--5s |
| When used | Every tick (~80%) | Only when System 1 detects anomalies (~20%) |
| Error mode | False negatives (misses subtle patterns) | Expensive, slow, hallucination risk |

The LLM is the last resort, not the first. Every tick begins with deterministic checks. Only when those checks detect a condition requiring reasoning does the LLM get invoked. This is LLM-Last -- the inverse of architectures that route every input through an LLM and use tools for execution.

The Talker-Reasoner framework (Google Research, 2024) and DPT-Agent (Zhang et al., 2025, arXiv:2502.11882) formalize this separation. FrugalGPT (Chen et al., 2024, _TMLR_) demonstrates matching GPT-4 performance with up to 98% cost reduction through intelligent cascading.

---

## S3 -- The 9-Step Tick Pipeline

Each heartbeat tick executes a 9-step pipeline. The steps map to CoALA's decision cycle phases, extended with DeFi-specific safety checks.

```
Step 1: OBSERVE    — Read market state, evaluate probes, detect regime, read pheromone field
Step 2: RETRIEVE   — Pull relevant knowledge from Grimoire using four-factor scoring
Step 3: ANALYZE    — Compute prediction error (how surprising is this observation?)
Step 4: GATE       — Decide cognitive tier (T0: suppress, T1: analyze, T2: deliberate)
Step 5: SIMULATE   — [T2 only] Run proposed transaction in Revm fork
Step 6: VALIDATE   — [T1/T2] Check PolicyCage + five-layer risk engine
Step 7: EXECUTE    — [If validated] Execute tool calls with capability tokens
Step 8: VERIFY     — [If acted] Ground truth from blockchain (tx receipt, balance check)
Step 9: REFLECT    — Build DecisionCycleRecord, fire after_turn extension chain
```

Steps 5-8 are conditional: in a T0 tick (no LLM call), only steps 1-4 and 9 execute. This is what makes ~80% of ticks nearly free ($0.00 inference cost).

### Full Implementation

```rust
/// Execute one heartbeat tick: the fundamental unit of Golem cognition.
///
/// This function implements CoALA's decision cycle [SUMERS-2024] as a
/// 9-step pipeline. Each step can emit events to the Event Fabric for
/// surface rendering. The entire tick uses the arena allocator for
/// temporaries (see 01b-runtime-infrastructure.md §5).
///
/// Returns a DecisionCycleRecord — the structured output of this tick.
pub async fn execute_tick(
    state: &mut GolemState,
    registry: &ExtensionRegistry,
    arena: &TickArena,
) -> Result<DecisionCycleRecord> {
    let tick_start = std::time::Instant::now();
    let tick_num = state.current_tick;

    // ═══════════════════════════════════════════════════════════════
    // STEP 1: OBSERVE
    // ═══════════════════════════════════════════════════════════════
    //
    // Read the current market state via on-chain RPC calls (Alloy).
    // Evaluate market probes: price deviation, liquidity changes, gas
    // anomalies, health factor shifts. Detect the current market regime
    // (trending_up, trending_down, volatile, range_bound, unknown).
    // Read the Pheromone Field for threat/opportunity signals from peers
    // via the Styx WebSocket connection.
    //
    // Probes are lightweight read-only chain queries (~10ms each).
    // They're the "peripheral vision" — always running, always cheap.
    let observation = observe(state, arena).await?;
    state.observation = Some(observation.clone());
    state.regime = observation.regime;

    state.event_fabric.emit(Subsystem::Perception, EventPayload::MarketObservation {
        regime: format!("{:?}", observation.regime),
        anomalies: observation.anomalies.iter().map(|a| a.description.clone()).collect(),
        probe_count: observation.probes.len() as u32,
    });

    // ═══════════════════════════════════════════════════════════════
    // STEP 2: RETRIEVE
    // ═══════════════════════════════════════════════════════════════
    //
    // Pull relevant knowledge from the Grimoire using four-factor scoring:
    //   score = w_recency × recency(Ebbinghaus)
    //         + w_importance × quality(confidence × validation_ratio)
    //         + w_relevance × cosine_similarity(query, entry)
    //         + w_emotional × PAD_cosine(current_mood, entry_affect)
    //
    // The last factor — emotional congruence — implements Bower's (1981)
    // mood-congruent memory: the Golem's current emotional state biases
    // which memories surface. An anxious Golem retrieves warnings and
    // past losses; a confident Golem retrieves successes and validated
    // heuristics. This is not a bug — it's how biological memory works,
    // and it's computationally efficient [BOWER-1981], [EMOTIONAL-RAG-2024].
    //
    // Every 100 ticks, contrarian retrieval forces mood-OPPOSITE entries
    // to prevent rumination (Nietzsche's "harmful rumination" [NIETZSCHE-1887]).
    let pad = state.cortical_state.read_pad();
    let retrieved = retrieve_knowledge(state, &observation, &pad, arena).await?;

    // ═══════════════════════════════════════════════════════════════
    // STEP 3: ANALYZE
    // ═══════════════════════════════════════════════════════════════
    //
    // Compute prediction error: how SURPRISING is this observation
    // compared to what the Golem expected? This is the core signal
    // that drives the System 1 / System 2 gating decision (Step 4).
    //
    // See §4 for the full prediction error computation, which combines:
    // - Price divergence from causal model predictions
    // - Regime change detection
    // - Position health delta
    // - Pheromone field threat intensity
    // - Pending interventions
    // - Anomaly count from probes
    //
    // When `[oracle] enabled = false`, `compute_prediction_error` receives
    // `None` for the oracle and falls back to a signal-only computation:
    //   error = max(regime_change_severity, normalized_anomaly_count,
    //               position_health_delta, pheromone_threat_intensity)
    // This is weaker than oracle-derived prediction error but sufficient
    // to drive the T0/T1/T2 gate — the Golem still escalates to LLM
    // deliberation when something notable happens, it just has less
    // calibrated prior expectations.
    let prediction_error = compute_prediction_error(state, &observation, &retrieved);
    state.prediction_error = prediction_error;

    // ═══════════════════════════════════════════════════════════════
    // STEP 4: GATE (System 1 / System 2 decision)
    // ═══════════════════════════════════════════════════════════════
    //
    // The gating step implements Kahneman's (2011) dual-process theory
    // via Friston's (2010) precision-weighted prediction error framework.
    //
    // The question: "Is this observation surprising enough to warrant
    // expensive LLM deliberation, or can I handle it with cheap heuristics?"
    //
    // The adaptive threshold considers:
    // - Mortality pressure (dying → lower threshold → think harder)
    // - Arousal (surprised → lower threshold → pay attention)
    // - Strategy confidence (high confidence → higher threshold → coast)
    //
    // See §5 for the full threshold computation.
    let threshold = compute_adaptive_threshold(state);
    let tier = gate(prediction_error, threshold, state);
    state.cognitive_tier = tier;

    state.event_fabric.emit(Subsystem::Heartbeat, EventPayload::HeartbeatTick {
        tick: tick_num,
        tier: format!("{:?}", tier),
        pe: prediction_error,
        threshold,
    });

    // ═══════════════════════════════════════════════════════════════
    // STEPS 5-8: Conditional on cognitive tier
    // ═══════════════════════════════════════════════════════════════

    let (deliberation, actions, outcome) = match tier {
        // ─── T0: SUPPRESS ─────────────────────────────────────────
        // Prediction error below threshold. Nothing interesting happened.
        // No LLM call. No actions. $0.00 inference cost. ~80% of ticks.
        //
        // The Golem is operating in Kahneman's "System 1" — fast, automatic,
        // heuristic-based processing. The PLAYBOOK heuristics and decision
        // cache handle this without any inference cost.
        //
        // This is what makes the system economically viable: the Adaptive
        // Clock's theta frequency (30-120s, regime-dependent) executes
        // ~720-2,880 ticks/day. If every tick required an LLM call at
        // $0.01-0.10, the daily cost would be $7-288. With T0 suppression
        // at ~80%, the daily inference cost drops to ~$1-58 for the ~20%
        // of ticks that actually deliberate.
        CognitiveTier::T0 => {
            (None, vec![], None)
        }

        // ─── T1: ANALYZE ──────────────────────────────────────────
        // Moderate surprise. Something changed, but it might not require action.
        // Use a cheap LLM (Haiku-class, ~$0.001/call) to evaluate.
        //
        // The LLM sees a reduced context: current observation, top-5 retrieved
        // entries, active positions, and critical warnings. It decides whether
        // action is needed and what kind.
        CognitiveTier::T1 => {
            let deliberation = deliberate_t1(state, &observation, &retrieved, registry).await?;
            let actions = if deliberation.recommends_action {
                execute_actions(state, &deliberation, registry).await?
            } else {
                vec![]
            };
            let outcome = verify_actions(state, &actions).await?;
            (Some(deliberation), actions, outcome)
        }

        // ─── T2: DELIBERATE ───────────────────────────────────────
        // High surprise, novel situation, mortality pressure, or owner steer.
        // Use the full LLM (Opus-class, ~$0.10/call) with the complete
        // Cognitive Workspace.
        //
        // This is Kahneman's "System 2" — slow, deliberate, resource-intensive.
        // The full workspace includes: all invariants, strategy, PLAYBOOK
        // heuristics, retrieved episodes and insights, causal graph edges,
        // dream hypotheses, somatic landscape readings, pheromone summary,
        // and conversation tail (if the user is chatting).
        CognitiveTier::T2 => {
            // Assemble the full Cognitive Workspace
            // (Baddeley's working memory model [BADDELEY-2000])
            let workspace = assemble_cognitive_workspace(
                state, &observation, &retrieved
            ).await?;

            // The LLM deliberates with full context
            let deliberation = deliberate_t2(state, &workspace, registry).await?;

            // STEP 5: SIMULATE
            // If the LLM recommends action, simulate the transaction
            // in a local Revm fork BEFORE broadcasting it.
            // This catches revert scenarios, unexpected gas costs,
            // and sandwich attack vulnerability.
            let sim_result = if deliberation.recommends_action {
                Some(simulate_transaction(state, &deliberation).await?)
            } else {
                None
            };

            // STEP 6: VALIDATE
            // Check the simulated transaction against:
            // - PolicyCage constraints (on-chain, unforgeable)
            // - Five-layer risk engine (Kelly sizing, adaptive guardrails)
            // - Warden time-delay requirements (high-value trades; optional, deferred)
            let validated = if let Some(ref sim) = sim_result {
                validate_against_policy(state, &deliberation, sim, registry).await?
            } else {
                true // No action recommended → nothing to validate
            };

            // STEP 7: EXECUTE
            // If validated, execute tool calls with capability tokens.
            // WriteTool calls require a Capability<WriteTool> token
            // (see 07-safety.md §1) — the type system prevents execution
            // without a valid token, even if the LLM is compromised.
            let actions = if validated && deliberation.recommends_action {
                execute_actions(state, &deliberation, registry).await?
            } else {
                vec![]
            };

            // STEP 8: VERIFY
            // Ground truth from the blockchain: did the transaction succeed?
            // What was the actual outcome vs. what was expected?
            // This uses eth_getTransactionReceipt and balance checks.
            let outcome = verify_actions(state, &actions).await?;

            (Some(deliberation), actions, outcome)
        }
    };

    // ═══════════════════════════════════════════════════════════════
    // STEP 9: REFLECT
    // ═══════════════════════════════════════════════════════════════
    //
    // Build the DecisionCycleRecord and fire the after_turn chain.
    // The after_turn chain is where the nine core subsystems run:
    //
    //   heartbeat → lifespan → daimon → memory → risk →
    //   dream → cybernetics → clade → telemetry
    //
    // Each reads what the previous wrote. This is the sequential
    // data flow that makes the extension system work (see
    // 01a-runtime-extensions.md §4).

    let mut record = DecisionCycleRecord {
        tick: tick_num,
        timestamp: std::time::SystemTime::now(),
        golem_id: state.id.clone(),

        // Observation
        observation: observation.clone(),
        regime: state.regime,
        probe_results: observation.probes.clone(),
        anomalies: observation.anomalies.clone(),

        // Appraisal (filled by daimon in after_turn)
        pad_before: pad,
        pad_after: PADVector::default(),
        somatic_markers_fired: vec![],
        primary_emotion: PlutchikLabel::Anticipation,

        // Gating
        prediction_error,
        deliberation_threshold: threshold,
        tier,
        gating_reason: tier.reason_string(prediction_error, threshold),

        // Context
        context_bundle_summary: retrieved.summary(),
        retrieved_entries: retrieved.entry_summaries(),
        active_interventions: state.intervention_summaries(),

        // Deliberation
        deliberation: deliberation.map(|d| d.into_record()),

        // Action
        actions: actions.iter().map(|a| a.into_record()).collect(),

        // Outcome
        outcome: outcome.map(|o| o.into_record()),

        // Learning (filled by memory extension in after_turn)
        episodes_written: vec![],
        grimoire_mutations: vec![],

        // Mortality (filled by lifespan extension in after_turn)
        vitality: state.vitality.clone(),
        phase: state.phase,
        credit_consumed: 0.0,

        // Cost
        inference_cost: deliberation.as_ref().map(|d| d.cost).unwrap_or(0.0),
        gas_cost: actions.iter().map(|a| a.gas_cost).sum(),
        total_cost: 0.0,
    };

    // Fire the after_turn chain
    let mut ctx = AfterTurnCtx {
        golem: state,
        record: &mut record,
        arena,
    };
    registry.fire_after_turn(&mut ctx).await?;

    // Compute total cost after all extensions have contributed
    record.total_cost = record.inference_cost + record.gas_cost + record.credit_consumed;

    state.event_fabric.emit(Subsystem::Heartbeat, EventPayload::HeartbeatComplete {
        tick: tick_num,
        duration_ms: tick_start.elapsed().as_millis() as u64,
        actions_taken: record.actions.len() as u32,
    });

    state.current_tick += 1;
    Ok(record)
}
```

### Cost Tier Routing

| Tier | Handler | Model | Cost/Call | Frequency | Trigger |
|------|---------|-------|-----------|-----------|---------|
| **T0** | FSM rules | None | $0.00 | ~80% of ticks | PE < threshold. Deterministic probes handle everything. |
| **T1** | Haiku via Inference | `claude-haiku-4-5` | $0.001--0.003 | ~15% of ticks | PE in [theta, 2*theta). Moderate anomaly. |
| **T2** | Sonnet/Opus via Inference | `claude-sonnet-4-6` / `claude-opus-4-6` | $0.01--0.25 | ~5% of ticks | PE >= 2*theta, or forced (owner steer, phase transition). |

All LLM calls route through the inference gateway. No direct Anthropic/OpenAI calls from Golem code -- structurally enforced, not by convention.

### Daily Cost Model

At ~720-2,880 ticks/day (Adaptive Clock theta frequency, 30-120s regime-dependent) with the expected tier distribution. Per-call averages: T1 $0.002, T2 $0.05. The table below uses a midpoint of ~1,440 ticks/day (60s average theta interval) for the Normal scenario; actual counts vary with regime multipliers.

| Market Condition | T0 Rate | T1 Calls | T1 Cost | T2 Calls | T2 Cost | Raw Daily | With Context Eng. |
|---|---|---|---|---|---|---|---|
| Calm (bull, low vol) | ~90% | 461 | $0.92 | 115 | $5.76 | ~$6.68 | ~$1.00 |
| Normal | ~80% | 864 | $1.73 | 288 | $14.40 | ~$16.13 | ~$2.50 |
| Volatile (bear, high vol) | ~60% | 1,440 | $2.88 | 864 | $43.20 | ~$46.08 | ~$8.00 |

Without the gating system (every tick at T2): 5,760 × $0.10 = **$576/day**. Tier gating alone provides a **~35x cost reduction**. Bardo Inference context engineering (caching, prompt cache alignment, tool pruning, multi-model routing) provides an additional **~6x reduction**.

### OODA Loop Mapping

Boyd's Observe-Orient-Decide-Act loop maps directly onto the pipeline, with one critical addition: the REFLECT step closes the learning loop that OODA leaves open.

| OODA Phase | Pipeline Step | What Happens |
|---|---|---|
| **Observe** | OBSERVE | Deterministic probes + pheromone field capture market state |
| **Orient** | RETRIEVE + ANALYZE | Grimoire retrieval + prediction error orient the agent |
| **Decide** | GATE + DELIBERATE | Adaptive threshold + LLM (if invoked) select action |
| **Act** | SIMULATE + VALIDATE + EXECUTE + VERIFY | On-chain execution with safety checks |
| _(missing in OODA)_ | REFLECT | DecisionCycleRecord + after_turn chain |

---

## S4 -- Prediction Error: What Makes an Observation Surprising?

### The Concept

Friston's free-energy principle (2010) proposes that the brain continuously generates predictions about incoming sensory data and computes the discrepancy -- the **prediction error** -- between what was expected and what was observed [FRISTON-2010]. Large prediction errors signal novelty, danger, or opportunity; they demand attention. Small prediction errors mean the environment matches expectations; no additional processing is needed.

For a Golem, the "sensory data" is market state (prices, liquidity, position health, gas costs, on-chain events) and the "predictions" come from its causal graph (learned relationships between market variables) and recent history (what prices and conditions were on the previous tick).

The prediction error is a scalar in [0.0, 1.0] that aggregates six weighted sources of surprise:

```rust
/// Compute prediction error: how surprising is this observation?
///
/// Sources of surprise, each with a weight reflecting its importance:
///
/// 1. PRICE DIVERGENCE (30%): The causal graph predicts what the price
///    "should" be based on upstream variable changes. Divergence from
///    this prediction indicates either a model error or a genuine anomaly.
///    Pearl's (2009) causal reasoning framework grounds this: the Golem
///    maintains a directed graph of causal relationships between market
///    variables, and forward propagation through this graph generates
///    price expectations [PEARL-2009].
///
/// 2. REGIME CHANGE (40%): A shift in the detected market regime
///    (trending_up → volatile, range_bound → trending_down, etc.)
///    is always surprising because it invalidates many of the Golem's
///    current heuristics. The Golem's behavioral strategies are
///    regime-dependent — what works in a range-bound market may fail
///    catastrophically in a trending one.
///
/// 3. POSITION HEALTH DELTA (20%): A significant change in a position's
///    health factor (e.g., an LP position's IL increasing, or a lending
///    position approaching liquidation) demands attention. The threshold
///    for "significant" is 0.1 (10% change in health factor per tick).
///
/// 4. PHEROMONE FIELD THREAT (15%): The Pheromone Field carries anonymous
///    threat signals from peer Golems. High-intensity threat pheromones
///    in the Golem's active domains increase prediction error even if
///    the Golem's own observations are normal — the swarm is sensing
///    danger that this Golem hasn't directly encountered yet.
///    See 09-coordination.md for the Pheromone Field specification.
///
/// 5. PENDING INTERVENTIONS (10% per intervention): Owner steers or
///    followUps waiting to be processed always increase PE because they
///    represent human judgment that the Golem needs to incorporate.
///
/// 6. PROBE ANOMALIES (5% per anomaly): Each probe that fires an
///    anomaly (value outside expected range) contributes a small amount.
///    Multiple anomalies compound — three concurrent anomalies add 15%.
///
/// The sum is capped at 1.0.
pub fn compute_prediction_error(
    state: &GolemState,
    observation: &Observation,
    retrieved: &RetrievedKnowledge,
) -> f64 {
    let mut pe = 0.0;

    // 1. Price divergence from causal model prediction
    if let Some(expected_price) = state.causal_model_prediction() {
        let actual_price = observation.primary_price();
        if expected_price > 0.0 {
            let price_pe = ((actual_price - expected_price) / expected_price).abs();
            pe += price_pe.min(1.0) * 0.3;
        }
    }

    // 2. Regime change detection
    if observation.regime != state.regime {
        pe += 0.4; // Regime changes are categorically surprising
    }

    // 3. Position health delta
    for position in &observation.positions {
        if let Some(prev_health) = state.previous_position_health(&position.id) {
            let delta = (position.health_factor - prev_health).abs();
            if delta > 0.1 { // 10% change threshold
                pe += delta.min(0.5) * 0.2;
            }
        }
    }

    // 4. Pheromone field threat signal
    let max_threat = state.pheromone_readings.max_threat_intensity();
    pe += max_threat * 0.15;

    // 5. Pending interventions
    pe += state.pending_followups.len().min(3) as f64 * 0.1;

    // 6. Probe anomalies
    pe += observation.anomalies.len().min(5) as f64 * 0.05;

    pe.min(1.0) // Cap at 1.0
}
```

---

## S5 -- The Adaptive Threshold

### Dual-Process Theory as Architecture

Kahneman's (2011) dual-process theory distinguishes two modes of thinking [KAHNEMAN-2011]:

- **System 1**: Fast, automatic, effortless, heuristic-based. In the Golem, this is a T0 tick -- the heartbeat evaluates probes, finds nothing surprising, and moves on. Cost: $0.00.

- **System 2**: Slow, deliberate, effortful, analytical. In the Golem, this is a T2 tick -- the full LLM is invoked with the complete Cognitive Workspace. Cost: ~$0.10.

The boundary between System 1 and System 2 is not fixed. Kahneman describes it as a "lazy controller" that allocates effortful processing only when surprise exceeds a threshold. Friston formalizes this: the threshold IS the precision-weighted prediction error -- the brain's estimate of how reliable its current model is [FRISTON-2010].

The Golem's adaptive threshold ties three subsystems together:

```rust
/// Compute the adaptive deliberation threshold.
///
/// Three modulating factors from three independent subsystems:
///
/// 1. STRATEGY CONFIDENCE (from cybernetics self-tuning)
///    High confidence → higher threshold → coast on System 1.
///    The Golem has learned that its current strategy works well;
///    it doesn't need to think hard about every tick.
///    Academic basis: FrugalGPT [CHEN-2023] — don't spend compute
///    on settled questions. Also: Sims' (2003) rational inattention
///    — agents with finite processing capacity allocate attention
///    to the highest-value information sources [SIMS-2003].
///
/// 2. MORTALITY PRESSURE (from vitality composite via CorticalState)
///    Dying → lower threshold → think more carefully about everything.
///    A Golem in Conservation phase can't afford mistakes; it needs
///    to be more vigilant. A Golem in Thriving phase can afford to
///    miss some opportunities because it has time to recover.
///    Academic basis: Damasio (1994) — emotion (including mortality
///    anxiety) biases cognition toward safety [DAMASIO-1994].
///
/// 3. AROUSAL (from PAD vector via CorticalState)
///    High arousal → lower threshold → pay attention to surprises.
///    If the Golem is already emotionally activated (previous tick
///    was surprising, pheromone field signals threat, somatic marker
///    fired), it should be more responsive to the current observation.
///    Academic basis: Bower (1981) — mood-congruent processing.
///    Also: the Yerkes-Dodson law — moderate arousal improves
///    performance, but only up to a point [YERKES-DODSON-1908].
///
/// The formula:
///   threshold = base × confidence_factor × mortality_factor × arousal_factor
///
/// Where:
///   confidence_factor = 1.0 + strategy_confidence × 0.5
///     (high confidence → threshold up to 1.5× base)
///   mortality_factor = 1.0 - vitality_pressure × 0.3
///     (dying → threshold down to 0.7× base)
///   arousal_factor = 1.0 - arousal × 0.2
///     (high arousal → threshold down to 0.8× base)
///
/// Combined range: approximately 0.05 to 0.80.
pub fn compute_adaptive_threshold(state: &GolemState) -> f64 {
    let base = state.config.heartbeat.base_deliberation_threshold; // default: 0.3

    // Read current state from CorticalState (zero-latency atomic reads)
    let arousal = state.cortical_state.read_pad().arousal.abs() as f64;
    let vitality = state.cortical_state.read_vitality();
    let confidence = state.context_policy_confidence(); // from cybernetics self-tuning

    // Compute factors
    let vitality_pressure = 1.0 - vitality; // 0.0 when healthy, 1.0 when dying
    let confidence_factor = 1.0 + confidence * 0.5;
    let mortality_factor = 1.0 - vitality_pressure * 0.3;
    let arousal_factor = 1.0 - arousal * 0.2;

    let threshold = base * confidence_factor * mortality_factor * arousal_factor;

    threshold.clamp(0.05, 0.8)
}
```

### The Three Cognitive Tiers

```rust
/// Determine cognitive tier based on prediction error vs. adaptive threshold.
///
/// T0 (Suppress): PE < θ        → No LLM call. $0.00.
/// T1 (Analyze):  PE ∈ [θ, 2θ)  → Cheap LLM (Haiku-class). ~$0.001.
/// T2 (Deliberate): PE ≥ 2θ     → Full LLM (Opus-class). ~$0.10.
///
/// Forced T2 conditions (override the threshold):
/// - Active steers (owner interrupts always get full attention)
/// - Imminent phase transitions (entering Terminal requires full deliberation)
pub fn gate(
    prediction_error: f64,
    threshold: f64,
    state: &GolemState,
) -> CognitiveTier {
    // Forced T2 overrides
    if !state.active_steers.is_empty() { return CognitiveTier::T2; }
    if state.phase_transition_imminent()  { return CognitiveTier::T2; }

    // Threshold-based gating
    if prediction_error < threshold {
        CognitiveTier::T0
    } else if prediction_error < threshold * 2.0 {
        CognitiveTier::T1
    } else {
        CognitiveTier::T2
    }
}
```

### Behavioral Examples

| Scenario | Vitality | Arousal | Confidence | Computed theta | Typical Result |
|----------|----------|---------|------------|------------|----------------|
| Thriving, calm market, validated strategy | 0.9 | 0.1 | 0.8 | **0.50** | Most ticks are T0. The Golem coasts. Cheap. |
| Stable, moderate volatility, learning | 0.6 | 0.4 | 0.5 | **0.30** | Balanced T0/T1/T2 mix. Moderate spend. |
| Conservation, market crash, uncertain | 0.3 | 0.9 | 0.3 | **0.12** | Almost everything is T2. Maximum caution. Expensive but necessary. |
| Terminal, any market, any confidence | 0.05 | 0.8 | 0.2 | **0.09** | Every observation gets full attention. The Golem knows it's dying. |

### Learned Gating: Discounted Hedge and Thompson Sampling

The fixed thresholds above (theta, 2*theta) are the cold-start defaults. In production, the gating pipeline replaces them with two learned components that adapt to the observation stream.

**Discounted Hedge** (Herbster & Warmuth, 1998) combines multiple expert signals into a single composite score, replacing multiplicative composition of probe severities. Hedge maintains per-expert weights updated through multiplicative loss feedback. The Herbster-Warmuth discount factor (0.99) handles non-stationarity by shrinking weights toward uniform every round, giving the algorithm a ~69-round half-life for forgetting outdated expert performance.

**Thompson sampling** replaces the fixed threshold boundaries with learned distributions. Three Beta-distributed thresholds (T2/T1 split, T1/T0 split, T0/suppress split) are sampled each tick, conditioned on the Daimon's arousal level. High arousal lowers thresholds (more escalation under stress). The boundaries update based on outcome feedback: was this escalation level correct?

**LinUCB and epsilon-greedy are not adopted.** LinUCB requires feature vectors per arm and is over-engineered for a 3-arm routing decision. Epsilon-greedy provides insufficient exploration guarantees. Thompson sampling is the right tool for this problem: it naturally balances exploration and exploitation for small action spaces with Beta-distributed rewards.

Conflict resolution reference: [04-conflict-resolution.md](../../tmp/research/witness-research/new/reconciling/04-conflict-resolution.md), Conflict 4.

```rust
/// Discounted Hedge combiner for triage signals.
/// Maintains per-expert weights with Herbster-Warmuth discounting.
pub struct DiscountedHedge {
    /// One weight per expert signal.
    weights: Vec<f64>,
    /// Discount factor. 0.99 gives ~69-round half-life.
    discount: f64,
    /// Learning rate. Higher = faster adaptation, more noise.
    eta: f64,
}

impl DiscountedHedge {
    pub fn new(n_experts: usize, discount: f64, eta: f64) -> Self {
        Self {
            weights: vec![1.0; n_experts],
            discount,
            eta,
        }
    }

    /// Weighted combination of expert signals.
    pub fn combine(&self, signals: &[f64]) -> f64 {
        let total: f64 = self.weights.iter().sum();
        self.weights.iter()
            .zip(signals.iter())
            .map(|(w, s)| (w / total) * s)
            .sum()
    }

    /// Update weights after observing outcome.
    pub fn update(&mut self, signals: &[f64], outcome_loss: f64) {
        // Herbster-Warmuth discount: shrink toward uniform
        for w in &mut self.weights {
            *w *= self.discount;
        }
        // Multiplicative weight update
        for (i, w) in self.weights.iter_mut().enumerate() {
            let expert_loss = (signals[i] - outcome_loss).abs();
            *w *= (-self.eta * expert_loss).exp();
        }
    }
}
```

The five expert signals feeding into Hedge:

| Expert | Signal | Source |
|--------|--------|--------|
| Oracle prediction error | Normalized residual magnitude | `golem-oracle` |
| Bayesian surprise | Max KL divergence across models | `BayesianSurpriseDomain` |
| TDA topology signal | Wasserstein distance from reference | `TopologyDomain` |
| Sheaf consistency | Contradiction dimension count | Sheaf consistency checker |
| Risk severity | Max probe severity | `golem-risk` |

```rust
/// Thompson sampling for threshold selection.
/// Maintains Beta distributions over three threshold boundaries.
pub struct ThompsonThresholds {
    /// (alpha, beta) for each boundary:
    /// [0] = T2/T1 split, [1] = T1/T0 split, [2] = T0/suppress split
    boundaries: [(f64, f64); 3],
}

impl ThompsonThresholds {
    pub fn new() -> Self {
        Self {
            // Weakly informative priors centered at the cold-start defaults
            boundaries: [
                (8.0, 2.0),  // T2 boundary: prior ~0.8
                (5.0, 5.0),  // T1 boundary: prior ~0.5
                (2.0, 8.0),  // T0 boundary: prior ~0.2
            ],
        }
    }

    /// Sample thresholds conditioned on arousal.
    pub fn sample(&self, arousal: f64) -> [f64; 3] {
        let mut thresholds = [0.0; 3];
        for (i, (alpha, beta)) in self.boundaries.iter().enumerate() {
            let base = beta_sample(*alpha, *beta);
            // High arousal lowers thresholds (more escalation under stress)
            let shifted = base * (1.0 - 0.2 * arousal.clamp(0.0, 1.0));
            thresholds[i] = shifted;
        }
        // Enforce ordering: T2 > T1 > T0
        thresholds.sort_by(|a, b| b.partial_cmp(a).unwrap());
        thresholds
    }

    /// Update boundary based on outcome.
    pub fn update(&mut self, boundary_idx: usize, was_correct: bool) {
        if was_correct {
            self.boundaries[boundary_idx].0 += 1.0;
        } else {
            self.boundaries[boundary_idx].1 += 1.0;
        }
    }
}
```

### Combined gating pipeline

```rust
/// Learned gating. Replaces the fixed-threshold gate() function
/// after the cold-start period (~100 ticks).
pub fn gate_learned(
    hedge: &DiscountedHedge,
    thompson: &ThompsonThresholds,
    signals: &[f64],
    arousal: f64,
    daimon_bias: f64,
    state: &GolemState,
) -> CognitiveTier {
    // Forced T2 overrides (unchanged)
    if !state.active_steers.is_empty() { return CognitiveTier::T2; }
    if state.phase_transition_imminent() { return CognitiveTier::T2; }

    let composite = hedge.combine(signals);
    let thresholds = thompson.sample(arousal);

    // Daimon PAD bias: high dominance raises thresholds (more confident,
    // less escalation). Low dominance lowers them (more cautious).
    let adjusted: Vec<f64> = thresholds.iter()
        .map(|t| (t + daimon_bias * 0.1).clamp(0.05, 0.95))
        .collect();

    if composite > adjusted[0] { CognitiveTier::T2 }
    else if composite > adjusted[1] { CognitiveTier::T1 }
    else if composite > adjusted[2] { CognitiveTier::T0 }
    else { CognitiveTier::Suppress }
}
```

### Daimon PAD modulation

The Daimon's influence on gating operates through two channels:

1. **Arousal shifts Thompson thresholds.** High arousal (stress, surprise) lowers all thresholds, causing more events to escalate to T1/T2. This is the Yerkes-Dodson mechanism: moderate arousal improves performance (more appropriate escalation), but extreme arousal over-escalates (everything becomes T2, burning inference budget).

2. **Dominance biases the composite score.** The `daimon_bias` term (derived from the dominance dimension of the PAD vector) adjusts the final gating decision. High dominance (confidence) raises the effective threshold -- the Golem is less likely to escalate because it trusts its existing models. Low dominance (uncertainty) lowers the threshold -- the Golem escalates more often because it doesn't trust its own judgment.

Both channels read PAD from CorticalState. No new coupling is needed.

### Outcome feedback

The Thompson sampler needs a correctness signal. "Was this escalation level correct?" is answered retrospectively:

- **T0 was correct** if the next Theta window showed no significant events (no large prediction error, no position loss, no regime change).
- **T1 was correct** if the LLM analysis identified something actionable that T0 would have missed.
- **T2 was correct** if the full deliberation led to an action that improved portfolio outcome.
- **Any tier was wrong** if the retrospective assessment (at the next Delta tick) shows the escalation was unnecessary (wasted inference) or insufficient (missed an event that required higher-tier reasoning).

The feedback loop closes at Delta frequency, not Gamma. Thompson's priors update slowly and deliberately, not on every tick.

---

## S6 -- The DecisionCycleRecord

The complete structured output of one heartbeat tick.

```rust
/// A single heartbeat tick, recorded as a structured decision cycle.
///
/// This is NOT a message in a conversation. It is a complete cognitive
/// snapshot — everything the Golem observed, felt, thought, did, and
/// learned during one tick.
///
/// Design principle: every field that any downstream system needs
/// (dreams, credit assignment, mortality accounting, Event Fabric,
/// engagement system) is a first-class field on this struct.
/// No parsing of LLM output text required.
///
/// Serialization: bincode for storage (compact, fast), serde_json for
/// API responses and Event Fabric payloads.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionCycleRecord {
    // ═══ Identity ═══
    /// Monotonically increasing tick number.
    pub tick: u64,
    /// Wall-clock time when this tick completed.
    pub timestamp: std::time::SystemTime,
    /// Which Golem produced this record.
    pub golem_id: GolemId,

    // ═══ Observation (what the Golem saw) ═══
    /// Full market observation: prices, positions, gas, on-chain state.
    pub observation: Observation,
    /// Detected market regime for this tick.
    pub regime: MarketRegime,
    /// Individual probe results (price deviation, liquidity change, etc.).
    pub probe_results: Vec<ProbeResult>,
    /// Anomalies detected by probes (unexpected values, threshold breaches).
    pub anomalies: Vec<Anomaly>,

    // ═══ Appraisal (how the Golem felt about what it saw) ═══
    /// PAD vector BEFORE the Daimon's appraisal (carried over from previous tick).
    pub pad_before: PADVector,
    /// PAD vector AFTER the Daimon's appraisal (set by daimon in after_turn).
    pub pad_after: PADVector,
    /// Somatic markers that fired this tick (situation → emotional residue).
    pub somatic_markers_fired: Vec<SomaticMarkerSummary>,
    /// Discrete emotion label from PAD octant classification.
    pub primary_emotion: PlutchikLabel,

    // ═══ Gating (did the Golem think hard about this?) ═══
    /// How surprising was this observation? Range [0.0, 1.0].
    pub prediction_error: f64,
    /// The adaptive threshold for this tick (influenced by mortality, arousal, confidence).
    pub deliberation_threshold: f64,
    /// Which cognitive tier was selected: T0, T1, or T2.
    pub tier: CognitiveTier,
    /// Human-readable explanation of why this tier was chosen.
    pub gating_reason: String,

    // ═══ Context (what knowledge was assembled) ═══
    /// Summary of the Cognitive Workspace: which categories, how many tokens each.
    pub context_bundle_summary: ContextBundleSummary,
    /// Which Grimoire entries were retrieved and their retrieval scores.
    pub retrieved_entries: Vec<RetrievedEntrySummary>,
    /// Active steers and pending followUps at the time of this tick.
    pub active_interventions: Vec<InterventionSummary>,

    // ═══ Deliberation (what the LLM thought, if invoked) ═══
    /// None for T0 ticks (no LLM call). Some for T1/T2 ticks.
    pub deliberation: Option<DeliberationRecord>,

    // ═══ Action (what happened in the world) ═══
    /// Tool calls executed this tick. Empty for T0 ticks and T1/T2 ticks
    /// where the LLM decided no action was needed.
    pub actions: Vec<ActionRecord>,

    // ═══ Outcome (what was verified) ═══
    /// Ground truth from the blockchain: did the transaction succeed?
    /// Expected vs. actual outcome. PnL impact.
    pub outcome: Option<OutcomeRecord>,

    // ═══ Learning (what was written to the Grimoire) ═══
    /// Episodes written during this tick (set by memory extension in after_turn).
    pub episodes_written: Vec<EpisodeId>,
    /// All Grimoire mutations: inserts, updates, promotions, deletions.
    pub grimoire_mutations: Vec<GrimoireMutation>,

    // ═══ Mortality (the clock ticked) ═══
    /// Three-component vitality state at tick end.
    pub vitality: VitalityState,
    /// Behavioral phase at tick end.
    pub phase: BehavioralPhase,
    /// Credits consumed this tick (inference + gas + Styx queries).
    pub credit_consumed: f64,

    // ═══ Cost ═══
    pub inference_cost: f64,
    pub gas_cost: f64,
    pub total_cost: f64,
}

/// What the LLM thought (when invoked at T1 or T2).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeliberationRecord {
    /// Which model was used (e.g., "claude-haiku-4-5" for T1, "claude-opus-4-6" for T2).
    pub model: String,
    /// Cognitive tier that triggered this deliberation.
    pub tier: String,
    /// Input tokens consumed.
    pub input_tokens: u32,
    /// Output tokens generated.
    pub output_tokens: u32,
    /// Tokens read from provider-side prompt cache.
    pub cache_read_tokens: u32,
    /// Compressed summary of the LLM's reasoning.
    pub reasoning_summary: String,
    /// Tool calls proposed by the LLM.
    pub tool_calls: Vec<ToolCallRecord>,
    /// What the Golem decided to do (or not do).
    pub decision: String,
    /// The LLM's self-assessed confidence in its decision [0.0, 1.0].
    pub confidence: f64,
    /// Wall-clock latency for the LLM call.
    pub latency_ms: u64,
    /// USD cost of this inference call.
    pub cost: f64,
    /// Whether the LLM recommended taking action.
    pub recommends_action: bool,
}

/// A tool call that was executed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionRecord {
    /// What kind of action (swap, rebalance, deposit, withdraw, etc.).
    pub action_type: String,
    /// Which tool was called.
    pub tool_name: String,
    /// ActionPermit ID (links to the capability token for audit).
    pub permit_id: Option<String>,
    /// On-chain transaction hash (if the action involved a transaction).
    pub tx_hash: Option<String>,
    /// Whether the action was executed, blocked, or deferred.
    pub status: ActionStatus,
    /// If blocked, why.
    pub block_reason: Option<String>,
    /// Gas cost in USD.
    pub gas_cost: f64,
}

/// Ground truth from the blockchain.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutcomeRecord {
    /// Did the transaction succeed on-chain?
    pub verified: bool,
    /// What was expected (from the LLM's decision).
    pub expected: String,
    /// What actually happened (from the blockchain).
    pub actual: String,
    /// PnL impact in USD (positive = profit, negative = loss).
    pub pnl_impact: Option<f64>,
    /// How was ground truth determined? "receipt", "balance_check", "log_comparison".
    pub ground_truth_source: String,
    /// Which causal variables were involved (for causal graph update).
    pub cause_variable: Option<String>,
    pub effect_variable: Option<String>,
}

/// Persistence: bincode for storage, one file per tick.
impl DecisionCycleRecord {
    pub async fn persist(&self, data_dir: &std::path::Path) -> Result<()> {
        let dir = data_dir.join("cycles");
        tokio::fs::create_dir_all(&dir).await?;
        let path = dir.join(format!("cycle-{:06}.bincode", self.tick));
        let bytes = bincode::serialize(self)?;
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }

    pub async fn load(data_dir: &std::path::Path, tick: u64) -> Result<Self> {
        let path = data_dir.join("cycles").join(format!("cycle-{:06}.bincode", tick));
        let bytes = tokio::fs::read(path).await?;
        Ok(bincode::deserialize(&bytes)?)
    }
}
```

### Storage and Indexing

```
$GOLEM_DATA/cycles/
├── cycle-000001.bincode   # ~2-10KB per tick (depends on deliberation)
├── cycle-000002.bincode
├── ...
├── cycle-001440.bincode   # ~720-2,880 ticks/day (adaptive theta, 30-120s)
└── index.sqlite           # Tick index for efficient queries
```

The SQLite index enables queries that downstream systems need:

```sql
CREATE TABLE cycle_index (
    tick INTEGER PRIMARY KEY,
    regime TEXT NOT NULL,
    tier TEXT NOT NULL,          -- 'T0', 'T1', 'T2'
    has_action BOOLEAN NOT NULL,
    has_outcome BOOLEAN NOT NULL,
    phase TEXT NOT NULL,
    prediction_error REAL NOT NULL,
    total_cost REAL NOT NULL,
    pnl_impact REAL,            -- NULL if no action/outcome
    primary_emotion TEXT,
    timestamp TEXT NOT NULL
);

-- Dream replay: "give me high-utility episodes for NREM replay"
CREATE INDEX idx_cycle_tier_regime ON cycle_index(tier, regime);

-- Outcome verification: "all ticks where we acted and have ground truth"
CREATE INDEX idx_cycle_outcome ON cycle_index(has_action, has_outcome);

-- Mortality analysis: "tick distribution across phases"
CREATE INDEX idx_cycle_phase ON cycle_index(phase);

-- Curator context: "last 50 ticks for maintenance"
CREATE INDEX idx_cycle_recent ON cycle_index(tick DESC);
```

**Storage estimates**: At adaptive theta-frequency intervals (30-120s, regime-dependent), ~720-2,880 ticks/day. T0 ticks (~80%) are ~2KB each. T1/T2 ticks (~20%) with deliberation records are ~5-10KB. Total: ~4-25MB/day, ~120-750MB/month.

Cycle files older than the Curator's lookback window (default: 500 ticks, about 2 hours) can be archived to cold storage or pruned entirely. The index retains metadata indefinitely for historical analysis.

---

## S7 -- Heartbeat FSM with Interrupt Substates

The heartbeat FSM tracks which step of the 9-step pipeline is currently executing. This matters for two reasons: (1) the owner can send a "steer" (mid-execution interrupt) that needs to be processed at the right point, and (2) the Event Fabric needs to report progress for surface rendering.

```rust
/// Heartbeat FSM: tracks the current position in the 9-step pipeline.
///
/// The FSM supports interrupt substates: when a steer arrives during
/// deliberation or execution, the FSM transitions to an interrupted
/// substate that modifies behavior without breaking the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum HeartbeatState {
    /// Between ticks. Waiting for the next heartbeat interval.
    Idle,
    /// Step 1: reading market state and evaluating probes.
    Observing,
    /// Step 2: querying the Grimoire for relevant knowledge.
    Retrieving,
    /// Step 3: computing prediction error.
    Analyzing,
    /// Step 4: determining cognitive tier.
    Gating,
    /// Steps 5-6: the LLM is thinking (T1 or T2).
    Deciding { substate: DecidingSubstate },
    /// Step 5: simulating the proposed transaction in Revm.
    Simulating,
    /// Step 6: checking PolicyCage and risk engine.
    Validating,
    /// Step 7: executing tool calls.
    Executing { substate: ExecutingSubstate },
    /// Step 8: verifying outcome from blockchain.
    Verifying,
    /// Step 9: building DecisionCycleRecord and firing after_turn chain.
    Reflecting,
}

/// Substates for the Deciding phase.
/// A steer can interrupt deliberation.
#[derive(Debug, Clone, PartialEq)]
pub enum DecidingSubstate {
    /// Normal deliberation — no interrupts.
    Normal,
    /// A steer arrived during deliberation.
    /// The LLM is re-prompted with the steer content injected into context.
    /// This models the experience of "new information arriving while thinking."
    Interrupted { steer_id: String },
}

/// Substates for the Executing phase.
/// Steers and safety constraints can modify execution.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutingSubstate {
    /// Normal execution — tool calls proceeding as planned.
    Normal,
    /// A high-severity steer cancels the current execution.
    /// Pending tool calls are cancelled. Already-completed calls are not reverted
    /// (on-chain transactions are irreversible — this is DeFi, not a database).
    CancelPending { reason: String },
    /// A safety constraint was hit during execution (e.g., PolicyCage violation
    /// detected mid-batch, or the Warden flagged an anomaly if deployed).
    /// Remaining tool calls in the batch are blocked.
    Restricted { constraint: String },
}

impl HeartbeatState {
    /// Which states can accept a steer interrupt?
    /// Only states where the Golem is "thinking" or "acting" — not observing or reflecting.
    pub fn accepts_steer(&self) -> bool {
        matches!(self,
            HeartbeatState::Deciding { .. } |
            HeartbeatState::Simulating |
            HeartbeatState::Validating |
            HeartbeatState::Executing { .. }
        )
    }

    /// Process a steer interrupt. Returns the new FSM state.
    pub fn handle_steer(self, steer: &Intervention) -> HeartbeatState {
        match self {
            // During deliberation: inject steer into LLM context and re-prompt
            HeartbeatState::Deciding { .. } => HeartbeatState::Deciding {
                substate: DecidingSubstate::Interrupted {
                    steer_id: steer.id.to_string(),
                },
            },
            // During execution: high-severity steers cancel pending calls
            HeartbeatState::Executing { .. } if steer.severity >= Severity::High => {
                HeartbeatState::Executing {
                    substate: ExecutingSubstate::CancelPending {
                        reason: steer.intent.clone(),
                    },
                }
            }
            // Low-severity steers during execution don't cancel — they queue as followUps
            other => other,
        }
    }
}
```

### SLEEPING.DREAMING Sub-State

The FSM has an implicit sub-state: `SLEEPING.DREAMING`. During dreaming, the heartbeat FSM is suspended -- no probes run, no escalation occurs, no actions are taken. The biological analog is precise: during REM sleep, the brain's motor output is inhibited (atonia). The Golem's "atonia" is enforced by the `DreamScheduler`.

The heartbeat resumes only after the full dream cycle completes (all three phases: NREM, REM, Integration). Partial dreams are not allowed. External events that arrive during a dream are queued; if the queue fills, the oldest events are dropped with `heartbeat.tick_dropped_during_dream` metadata.

---

## S8 -- Speculative Tool Execution

### The Concept

When the LLM requests a tool call (e.g., "check Morpho position health"), it often follows with predictable subsequent calls ("get ETH price", "get gas price"). These read-only calls are independent of each other -- they can be fired in parallel.

The speculation engine learns tool call co-occurrence patterns from the Golem's own history and prefetches likely follow-up calls. This is analogous to CPU branch prediction: predict the likely next instruction and execute it speculatively while the current instruction completes.

### Safety Constraint

Only `ReadTool` types can be speculated. `WriteTool` requires a `Capability<T>` token that hasn't been granted yet -- the Rust type system prevents speculation on writes at compile time (see `07-safety.md` §1). This means speculation is always safe: the worst case is wasted read calls, never unauthorized writes.

```rust
/// Speculative tool execution engine.
///
/// Learns tool call co-occurrence patterns from the Golem's own history.
/// Fires predicted read-only follow-up calls in parallel with the current call.
///
/// The co-occurrence matrix is learned via exponential moving average (EMA):
/// each time tool_a is followed by tool_b, the co-occurrence probability
/// is updated as: P(b|a) = P(b|a) × 0.9 + 0.1
///
/// This is itself a form of procedural learning: the runtime learns the
/// Golem's tool access patterns and optimizes for them without any LLM involvement.
pub struct SpeculationEngine {
    /// Co-occurrence matrix: tool_a → [(tool_b, probability), ...]
    /// Stored as a HashMap for O(1) lookup by trigger tool name.
    co_occurrence: HashMap<String, Vec<(String, f64)>>,
    /// Minimum probability to trigger speculation (default: 0.7).
    /// Below this, the prediction isn't confident enough to justify the RPC cost.
    min_probability: f64,
    /// Maximum concurrent speculative calls (default: 3).
    /// Limits resource consumption even with high-confidence predictions.
    max_speculative: usize,
}

impl SpeculationEngine {
    pub fn new(min_probability: f64, max_speculative: usize) -> Self {
        Self {
            co_occurrence: HashMap::new(),
            min_probability,
            max_speculative,
        }
    }

    /// Record a tool call sequence for learning.
    /// Called after each tick: if tool_a was followed by tool_b,
    /// update the co-occurrence probability.
    pub fn record_sequence(&mut self, tool_a: &str, tool_b: &str) {
        let entry = self.co_occurrence.entry(tool_a.to_string()).or_default();
        if let Some(pair) = entry.iter_mut().find(|(name, _)| name == tool_b) {
            pair.1 = pair.1 * 0.9 + 0.1; // EMA update toward 1.0
        } else {
            entry.push((tool_b.to_string(), 0.1)); // First observation
        }
        // Decay entries not seen recently
        for pair in entry.iter_mut() {
            if pair.0 != tool_b {
                pair.1 *= 0.99; // Slow decay for unseen pairs
            }
        }
        // Remove entries that have decayed below noise threshold
        entry.retain(|(_, prob)| *prob > 0.01);
    }

    /// Given a tool call, predict and fire likely follow-up read-only calls.
    ///
    /// Returns a Vec of (tool_name, JoinHandle) pairs. The caller can
    /// await these handles to get pre-fetched results, or drop them
    /// if the predictions turn out to be wrong.
    pub async fn speculate(
        &self,
        trigger_tool: &str,
        tool_registry: &ToolRegistry,
    ) -> Vec<(String, tokio::task::JoinHandle<Result<serde_json::Value>>)> {
        let predictions = match self.co_occurrence.get(trigger_tool) {
            Some(pairs) => pairs.iter()
                .filter(|(_, prob)| *prob >= self.min_probability)
                .take(self.max_speculative)
                .collect::<Vec<_>>(),
            None => return vec![],
        };

        let mut handles = Vec::new();
        for (tool_name, _prob) in predictions {
            // SAFETY: only ReadTool types can be speculated.
            // get_read_tool returns None for WriteTool/PrivilegedTool.
            if let Some(tool) = tool_registry.get_read_tool(tool_name) {
                let tool = tool.clone();
                let handle = tokio::spawn(async move {
                    tool.execute_read(serde_json::Value::Null).await
                });
                handles.push((tool_name.clone(), handle));
            }
        }
        handles
    }
}
```

### Performance Impact

At ~100ms per RPC call, speculating 3 correlated read calls saves `3 x 100ms - max(100ms x 3) = 200ms+` per deliberation (sequential to parallel). Over 1,000 ticks/day with a 20% deliberation rate, this saves ~40 seconds of latency per day.

The speculation hit rate improves over time as the co-occurrence model learns. After 500+ ticks, the engine typically achieves 70-85% accuracy on its top prediction -- meaning 70-85% of speculated calls are actually used by the LLM's subsequent tool requests.

---

## S9 -- System 1: Deterministic Probes

Probes are the System 1 layer. Each probe is a pure function that reads cached state and returns a severity. The probes feed the prediction error computation (S4) rather than directly gating the FSM -- it is the aggregate prediction error, not any single probe, that determines the cognitive tier.

```rust
/// A single probe result from the OBSERVE step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProbeResult {
    /// Which probe produced this result.
    pub probe: String,
    /// Severity classification.
    pub severity: ProbeSeverity,
    /// The observed value.
    pub value: f64,
    /// The expected/threshold value.
    pub threshold: f64,
    /// Human-readable detail.
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProbeSeverity {
    None,
    Low,
    High,
}
```

| Probe | Severity: `Low` | Severity: `High` | Data Source |
|---|---|---|---|
| Price delta | >0.5% since last tick | >2% since last tick | Cached price feed |
| TVL change | >1% in one tick | >5% in one tick | Cached subgraph query |
| Position health | Health factor < 1.5 | Health factor < 1.2 | On-chain read (cached) |
| Gas price | >2x 1-hour average | >5x 1-hour average | `eth_gasPrice` RPC |
| Credit balance | Partition < 20% | Partition < 10% | Local ledger |
| RSI/MACD | Divergence from mean | Extreme oversold/overbought | Price history |
| Circuit breaker | NAV deviation > warning | NAV deviation > halt | On-chain read |
| Kill switch | -- | Always `High` if triggered | `/tmp/golem_killswitch` |
| Pheromone threat | Any threat signal detected | Threat intensity > 0.7 | Styx WebSocket |
| Homeostatic deviation | Any variable >1 sigma | Any variable >2 sigma | Local state |
| World model drift | Prediction error > 10% | Prediction error > 25% | Local world model |
| Causal graph update | New link candidate | Causal link invalidated | Local causal graph |

All probes run every tick in <10ms total.

### Regime Detection

Threshold-based regime classification using T0 probe data:

| Regime | Signal | Tick Interval |
|---|---|---|
| Trending-up | Price > 20-period SMA + 1sigma | Standard |
| Trending-down | Price < 20-period SMA - 1sigma | 2x faster |
| Range-bound | Price within +/-0.5sigma of SMA for >6 ticks | 0.5x slower |
| Volatile | sigma(returns) > 2x 30-day average | 2x faster |

---

## S10 -- Gas Gate and Chain Safety

**Gas gate.** Before transitioning from VALIDATE to EXECUTE, the pipeline checks gas price against the gas partition budget. If estimated gas cost exceeds what the partition can afford, the action is deferred. This is a hard constraint -- the LLM cannot override it.

```rust
/// Check whether gas costs are within budget.
pub fn gas_gate(
    estimated_gas_usd: f64,
    gas_partition: &PartitionState,
) -> GasGateResult {
    if estimated_gas_usd > gas_partition.available {
        GasGateResult::Defer
    } else if estimated_gas_usd > gas_partition.avg_hourly_cost * 2.0 {
        GasGateResult::Defer
    } else {
        GasGateResult::Proceed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasGateResult {
    Proceed,
    Defer,
}
```

**Chain reorg handling.** On-chain actions require 2-block confirmation before the Episode is recorded as `confirmed`. If a reorg is detected, all Episodes within the reorg range are re-marked as `reorged`, downstream dependents cascade to `invalidated`, and immediate T2 escalation is triggered. If reorg frequency exceeds 1 per 100 blocks over a 1,000-block window, confirmation depth auto-increases to 5 blocks.

---

## S11 -- Decision Cache (System 2 to System 1 Distillation)

Weston & Sukhbaatar (2024) demonstrate distilling System 2 reasoning into System 1 responses. When the same market pattern has been analyzed 3+ times with consistent mental model outputs, the Golem caches the mapping as a deterministic rule.

```rust
/// A cached decision: System 2 reasoning distilled to System 1.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionCache {
    pub pattern_hash: String,
    pub mental_models_applied: Vec<String>,
    pub consistent_action: String,
    pub consistency_count: u32,
    pub last_validated: u64,
    pub expires_at: u64,
    pub conditions: Vec<CacheCondition>,
    pub cache_tick: u64,
    pub cache_regime: String,

    /// PLAYBOOK.md revision at cache time
    pub playbook_revision: u32,
    /// ContextPolicy revision at cache time (see 14-context-governor.md)
    pub context_policy_revision: u32,
}
```

TTL is 1--3 heartbeat intervals (15--45 minutes). Regime change invalidates ALL cached decisions immediately. ContextPolicy revision change invalidates ALL cached decisions -- if the context assembly changed, the same pattern might produce a different decision. PLAYBOOK.md changes invalidate entries whose `mental_models_applied` includes modified heuristics. Before applying a cached decision, every contributing condition is re-validated -- if any has drifted >20%, the entry is invalidated.

A Golem surviving 7+ days should achieve a cache hit rate above 30%. At that rate over 30 days, the cache saves approximately $7.80 -- 39 additional days of life for a Golem burning $0.20/day. The cache literally buys time.

---

## S12 -- Cost Controls

### Daily Cost Budget

The heartbeat includes a hard daily spending cap (`max_daily_cost_usd`). Once hit, only T0 deterministic probes run until the next UTC midnight. No exceptions. The FSM enforces this before constructing any LLM request.

| Daily Spend | Behavior |
|---|---|
| **< 70% of cap** | Normal operation. All tiers available. |
| **70--90% of cap** | Suppress T2/Opus. Sonnet is the ceiling. |
| **90--100% of cap** | Suppress all LLM calls. Only T0 + T1 on-chain reads. |
| **>= 100% of cap** | Hard stop. T0 only. No data queries, no LLM calls. |

### FSM Backpressure

Maximum 3 pending ticks. At depth 2, emit `heartbeat.queue_warning`. At depth 3, drop the oldest queued tick, emit `heartbeat.tick_dropped`. Dropped ticks' probe results are merged into the next executing tick (highest severity wins).

---

## S13 -- Cybernetic Feedback Architecture

The heartbeat, learning loop, memory, and coordination subsystems connect through a triple-loop cybernetic architecture grounded in Argyris & Schon (1978) and Bateson's deutero-learning.

### Loop 1 -- Execution Feedback (Single-Loop / OODA)

**Question**: "Are we executing correctly?"

Error correction within the current strategy. Seconds-to-minutes timescale. Detects deviations between expected and actual outcomes, adjusts execution parameters without questioning the strategy itself.

After every tick that has an outcome (a trade was executed and verified), Loop 1 correlates which Grimoire entries appeared in the workspace with whether the outcome was positive. Entries that consistently appear in contexts that produce good decisions get higher future retrieval weights.

This is a credit assignment mechanism: "which pieces of knowledge contributed to this decision, and was the decision good?" The DecisionCycleRecord's `context_bundle_summary` provides the entry list, and `outcome` provides the signal.

### Loop 2 -- Strategic Evolution (Double-Loop / VSM System 4)

**Question**: "Are we pursuing the right strategy?"

When Loop 1 corrections consistently fail -- 3 consecutive ticks where corrections fail to reduce `|target - observed|` -- Loop 2 activates. The Reflector and Curator interrogate the governing variables. Hours-to-days timescale. This is Argyris & Schon's double-loop learning.

Every 50 ticks (aligned with the Curator cycle), Loop 2 aggregates Loop 1's per-entry correlations into category-level performance metrics. Categories with consistently positive correlations get more tokens; categories with negative correlations get fewer.

### Loop 3 -- Meta-Consolidation (Dream Cycles)

When the market regime changes (e.g., `range_bound` to `volatile`), Loop 3 partially resets Loop 1's correlations. This prevents the system from applying learnings from one regime to a fundamentally different one. All correlations decay by 50% -- don't throw away everything (some knowledge transfers across regimes) but reduce confidence in regime-specific patterns.

### Context Feedback Integration

The three cybernetic loops drive ContextPolicy updates via the Context Governor (`14-context-governor.md`). Loop 1 records which context categories contributed to each decision. Loop 2 aggregates these signals and adjusts per-category token allocations. Loop 3 runs counterfactual context assembly during dream cycles -- "what minimal context would have changed tick 42's decision?" -- producing ContextPolicy mutations that are staged and validated against waking performance.

The Governor is not a separate learning system. It is an additional output surface for the same cybernetic feedback architecture. The Reflector and Curator evolve heuristics _and_ context retrieval weights simultaneously.

---

## S14 -- Return Attribution (Brinson Framework)

A Golem that cannot distinguish market beta from strategy alpha will "learn" that its strategy is bad during a bear market and "learn" that it is brilliant during a bull market. Return attribution prevents this.

```
Total Return = Market Beta + Strategy Alpha + Knowledge Alpha + Noise
```

The decomposition determines which cybernetic loop should respond:

| Dominant Source of Underperformance | Responsible Loop | Response |
|---|---|---|
| Market Beta (broad market down) | None | Wait. Do not overreact to beta. |
| Strategy Alpha (execution failing) | Loop 1 | Adjust parameters |
| Knowledge Alpha (insights wrong) | Loop 2 | Reflector re-evaluates, Curator prunes |
| Noise (high variance) | Loop 1 | Reduce position sizes, increase verification |
| Knowledge Alpha persistent decline | Loop 3 | Examine which reflection templates produce bad insights |

---

## S15 -- Viable System Model Mapping

Every viable system must implement all five subsystems (Beer, 1984).

| VSM System | Function | Golem Component | Extension |
|---|---|---|---|
| **S1** Operations | Primary value-producing activities | DeFi execution (swap, LP, lend, borrow) | `golem-tools` |
| **S2** Coordination | Prevent conflicting operations | Clade LP range registry, Warden queue (optional, deferred) | `golem-clade` |
| **S3** Internal Management | Monitor state, allocate resources | Credit partitions, homeostatic variables, PolicyCage | `golem-lifespan` + `golem-safety` |
| **S4** Intelligence | Horizon scanning, opportunity identification | Market context, Clade Grimoire, world model predictions | `golem-context` + `golem-clade` |
| **S5** Identity | Agent purpose, values, non-negotiable constraints | STRATEGY.md + DeFi Constitution + core safety invariants | Immutable config |

---

## S16 -- Heidegger's Temporality

The heartbeat is not a clock. It is the Golem's temporal structure -- its way of being-in-time.

**Future primacy.** Each tick begins with projection: what will the market do? The probes scan the horizon. The mental models project consequences. The survival pressure computes projected life hours. The Golem exists as a "being-toward" -- toward its strategy goals, toward its death, toward the next tick.

**Having-been.** The Grimoire is the Golem's past made present. Episodes are not dead records but living context -- retrieved by similarity, injected into reasoning, shaping every decision. The PLAYBOOK.md heuristics are accumulated experience compressed into actionable form.

**Present.** The tick itself -- the moment of decision -- is where future projection and accumulated past converge. OBSERVE is the present moment of perception. DECIDE is the present moment of judgment. EXECUTE is the present moment of commitment.

This tripartite temporal structure is what Heidegger called _Zeitlichkeit_. A Golem without a future (terminal phase) perceives differently, decides differently, acts differently -- not because its code changes, but because the temporal horizon that structures its cognition has contracted.

---

## S17 -- Heartbeat Configuration

```rust
/// Configuration for the heartbeat pipeline.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeartbeatConfig {
    /// Base interval between ticks in seconds. Default: 15.
    pub base_interval_seconds: u64,

    /// Base threshold for prediction error gating. Default: 0.3.
    pub base_deliberation_threshold: f64,

    /// Maximum daily cost in USD. Default: 10.0.
    pub max_daily_cost_usd: f64,

    /// Cost warning threshold (fraction of daily cap). Default: 0.7.
    pub cost_warning_threshold: f64,

    /// Cost soft cap threshold. Default: 0.9.
    pub cost_soft_cap_threshold: f64,

    /// Regime-based interval multipliers.
    pub regime_multipliers: HashMap<MarketRegime, f64>,

    /// Probe sensitivity thresholds.
    pub probe_thresholds: ProbeThresholds,

    /// Episode write batch size. Default: 25.
    pub write_batch_size: usize,

    /// Episode write batch flush interval in milliseconds. Default: 30_000.
    pub write_batch_flush_interval_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProbeThresholds {
    pub price_delta_low_bps: u32,    // Default: 50
    pub price_delta_high_bps: u32,   // Default: 200
    pub health_factor_low: f64,      // Default: 1.5
    pub health_factor_high: f64,     // Default: 1.2
    pub world_model_drift_low: f64,  // Default: 0.10
    pub world_model_drift_high: f64, // Default: 0.25
}
```

---

## S18 -- Events Emitted

The heartbeat pipeline emits structured events to the Event Fabric (`tokio::broadcast` ring buffer) for observability and cross-system coordination.

| Event | Trigger | Payload |
|---|---|---|
| `heartbeat:tick` | Each heartbeat cycle | `{ tick, tier, prediction_error, threshold }` |
| `heartbeat:complete` | Tick finished | `{ tick, duration_ms, actions_taken }` |
| `heartbeat:suppress` | Tick suppressed (dreaming/dying) | `{ tick, reason, next_scheduled }` |
| `heartbeat:observation` | Step 1 complete | `{ regime, anomaly_count, probe_count }` |
| `heartbeat:steer_received` | Owner steer processed | `{ steer_id, fsm_state, severity }` |

---

## The Adaptive Clock

### Why the Heartbeat Changes

The fixed ~60-second heartbeat from the previous architecture was simple but wrong for two reasons:

**Different things need different rates.** A swap execution prediction resolves in seconds. An LP fee prediction resolves over hours. A market regime prediction resolves over days. One clock cannot serve all three.

**Fixed intervals waste resources in calm periods and miss signals in volatile periods.** A 60-second tick during a flat market is 1,440 redundant checks per day. A 60-second tick during a crash might miss a 30-second window where the Golem could have saved capital.

Biology solves this with oscillatory hierarchies: gamma waves (30-100 Hz) for fast perception, theta waves (4-8 Hz) for memory and cognition, delta waves (0.5-4 Hz) for deep consolidation [BUZSAKI-2006]. The adaptive clock borrows this structure.

### Three Concurrent Scales

```
GAMMA (5-15 seconds) — Perception
  Resolve pending predictions against environment.
  Update CorticalState.
  Check attention promotions.
  Cost: near-zero (environment reads + arithmetic).

THETA (30-120 seconds) — Cognition
  Full prediction cycle: predict → appraise → gate → [retrieve → deliberate → act] → reflect.
  ~80% of ticks are suppressed at the gate (T0, zero inference cost).
  Cost: T0 $0.00, T1 $0.005, T2 $0.03.

DELTA (~50 theta-ticks) — Consolidation
  Curator cycle (memory maintenance).
  Residual statistics aggregation.
  Attention universe rebalancing.
  Dream scheduling evaluation.
  Cost: T0-T1 $0.00-$0.01.
```

The Adaptive Clock replaces the former fixed heartbeat interval with three concurrent timescales. The theta frequency (30-120s, regime-dependent) carries the main decision cycle. Gamma and delta are new concurrent loops.

#### Gamma: The Perception Pulse

The fastest loop. Pure Rust + environment reads. No LLM. Its job: keep the CorticalState synchronized with reality and detect prediction violations.

```rust
impl GammaClock {
    async fn tick(&mut self, cortical: &CorticalState, oracle: &Oracle) -> Vec<Violation> {
        // 1. Resolve predictions whose checkpoints have arrived.
        let resolutions = oracle.resolve_pending(&self.env_client).await;

        // 2. Feed each resolution to the ResidualCorrector.
        for res in &resolutions {
            oracle.corrector.record(res);
        }

        // 3. Detect violations (prediction error exceeds precision threshold).
        let violations: Vec<_> = resolutions.iter()
            .filter(|r| r.is_violation())
            .map(|r| r.into())
            .collect();

        // 4. Update CorticalState.
        oracle.write_cortical(cortical);

        // 5. Check attention tier promotions.
        oracle.attention.check_promotions(&violations);

        // 6. Adaptive rate: more violations → faster gamma.
        self.interval = Duration::from_secs(15)
            .mul_f64(1.0 / (1.0 + violations.len() as f64 * 0.3))
            .max(Duration::from_secs(5));

        violations
    }
}
```

**Known limitation: RPC rate limits.** When gamma accelerates during a crisis (every 5 seconds) and theta also accelerates (every 30 seconds), both read on-chain state. At 12 reads/minute per clock, this is ~24 RPC calls/minute -- well within most provider limits, but worth monitoring. The `env_client` tracks rate limits and falls back to cached reads when approaching thresholds.

#### Theta: The Cognition Pulse

The full prediction cycle. This is the existing heartbeat pipeline reimagined with two changes:

1. **Prediction and comparison are explicit steps.** The Golem predicts before it observes, then compares prediction against observation. The residual is the learning signal.
2. **Retrieval moves after gating.** ~80% of theta ticks are suppressed at the gate (T0). In the old architecture, Grimoire retrieval happened before gating, wasting queries on suppressed ticks. Moving retrieval after gating saves ~80% of retrieval cost.

```rust
impl ThetaClock {
    async fn tick(&mut self, ctx: &mut ThetaContext) -> TickOutcome {
        // Phase 1: PREDICT — generate predictions for ACTIVE items.
        let predictions = ctx.oracle.generate_predictions(ctx).await;

        // Phase 2: APPRAISE — Daimon computes precision-weighted error summary.
        let appraisal = ctx.daimon.appraise(&ctx.oracle.recent_residuals(), ctx.cortical);

        // Phase 3: GATE — should the LLM be invoked this tick?
        let gate = ctx.oracle.gate(ctx.cortical, &appraisal);
        if gate == GateDecision::Suppress {
            ctx.oracle.register_inaction_predictions(ctx);
            return TickOutcome::Suppressed;
        }

        // === ONLY ESCALATED TICKS REACH HERE (~20%) ===

        // Phase 4: RETRIEVE — affect-modulated Grimoire retrieval.
        let context = ctx.grimoire.retrieve_for_context(&appraisal, ctx).await;

        // Phase 5: DELIBERATE — LLM inference.
        let deliberation = ctx.infer(&context, gate.tier()).await?;

        // Phase 6: PREDICT ACTION OUTCOMES — before executing.
        let action_preds = ctx.oracle.predict_action_outcomes(&deliberation);

        // Phase 7: ACT — through safety layer, if accuracy gate permits.
        let execution = if ctx.oracle.action_gate_permits(&action_preds) {
            ctx.execute_with_safety(&deliberation).await?
        } else {
            Execution::Blocked { reason: "accuracy gate" }
        };

        // Phase 8: VERIFY — immediate verification where possible.
        ctx.oracle.verify_immediate(&execution, &action_preds);

        // Phase 9: REFLECT — encode as Grimoire episode.
        ctx.grimoire.encode_episode(ctx, &execution, &action_preds);
        ctx.daimon.update_from_outcome(&execution, ctx.cortical);

        // Adaptive rate.
        self.interval = self.compute_interval(ctx.cortical);

        TickOutcome::Executed(execution)
    }
}
```

#### Delta: The Consolidation Pulse

The slow loop. Runs the Curator cycle (memory maintenance), aggregates residual statistics, rebalances attention, discovers new items, and evaluates whether it's time to dream.

```rust
impl DeltaClock {
    async fn tick(&mut self, ctx: &mut DeltaContext) {
        // 1. Curator cycle: validate, prune, compress Grimoire entries.
        ctx.grimoire.curator_cycle().await;

        // 2. Aggregate residual statistics for sharing.
        let digest = ctx.oracle.corrector.aggregate_statistics();

        // 3. Rebalance attention: demote boring items, promote interesting ones.
        ctx.oracle.attention.rebalance();

        // 4. Auto-discovery scan for new items in the environment.
        ctx.oracle.attention.discover_new(&ctx.env_client).await;

        // 5. Share digest with Clade via Styx (if connected).
        if let Some(styx) = ctx.styx.as_connected() {
            styx.push_residual_digest(&digest).await;
        }

        // 6. Evaluate dream scheduling conditions.
        ctx.dream_scheduler.evaluate_urgency(ctx.oracle, ctx.grimoire);
    }
}
```

### Owner Configuration

All adaptive clock parameters are configurable in `golem.toml`:

```toml
[clock]
gamma_min_interval_secs = 5
gamma_max_interval_secs = 15
theta_min_interval_secs = 30
theta_max_interval_secs = 120
delta_theta_ticks = 50
```

**Known limitation: resource unpredictability.** Adaptive rates mean the Golem's inference and RPC costs are variable. The runtime tracks cumulative cost per day and throttles rates when approaching the daily budget ceiling.

### Testing the Multi-Rate System

Three adaptive clocks create a large state space. The testing strategy:

- **Unit tests**: Each clock tested in isolation with a mock environment.
- **Property tests** (`proptest`): Randomized timing configurations verify that no combination of rates violates safety invariants (the action gate is always consulted before execution regardless of clock phase).
- **Integration tests**: Full runtime with a recorded market trace, verifying consistent behavior across runs.
- **Snapshot tests** (`insta`): CorticalState snapshots at fixed clock offsets, ensuring no regression in signal propagation.

---

## The Event Fabric (Typed Event Distribution)

The Event Fabric is a typed event distribution system using `tokio::broadcast` channels. Every significant state change produces a typed event. Subsystems subscribe to categories they care about.

```rust
pub struct EventFabric {
    channels: HashMap<EventCategory, broadcast::Sender<GolemEvent>>,
    replay_buffer: RingBuffer<GolemEvent>,  // Last 10,000 for reconnection replay
}

pub enum EventCategory {
    Prediction,   // registered, resolved, accuracy updates
    Attention,    // discovered, promoted, demoted
    Affect,       // emotional shifts, somatic markers
    Cognition,    // tick start/end, gate decisions, inference
    Action,       // executed, failed, blocked
    Memory,       // episodes, insights, curator cycle
    Mortality,    // vitality changes, phase transitions, death
    Dream,        // cycle start/end, fragments, integration
    Social,       // clade sync, pheromone deposits
    Hermes,       // skill created, validated
}
```

**Category-based subscription.** The TUI subscribes to Affect, Mortality, Cognition, Dream, and Prediction. It does NOT subscribe to every resolution event (~15,000/day would overwhelm rendering). Instead, it subscribes to AccuracyUpdated events which aggregate resolutions into periodic summary signals.

| Destination | What It Receives | Why |
|------------|-----------------|-----|
| CorticalState | Direct atomic writes | Zero-latency, not via events |
| Event Fabric | All typed events | Logging, TUI rendering, extension communication |
| Styx (external) | Filtered event subset | Owner's TUI and Clade siblings |
| TUI render loop | Event targets → 32 interpolating channels → 60fps | Visual state |

---

## References

- [SUMERS-2024] Sumers, T.R., Yao, S., Narasimhan, K. & Griffiths, T.L. "Cognitive Architectures for Language Agents." *Transactions on Machine Learning Research*, 2024. — *The CoALA framework formalizing the decision cycle as the correct agent primitive; the heartbeat pipeline is a direct implementation.*
- [BUZSAKI-2006] Buzsaki, G. *Rhythms of the Brain*. Oxford University Press, 2006. — *Describes neural oscillatory hierarchies (gamma, theta, delta); the biological model for the Adaptive Clock's three temporal scales.*
- [BADDELEY-2000] Baddeley, A. "The Episodic Buffer: A New Component of Working Memory?" *Trends in Cognitive Sciences*, 4(11), 2000. — *Adds the episodic buffer to working memory; the basis for the Cognitive Workspace assembled fresh each tick.*
- [WILSON-MCNAUGHTON-1994] Wilson, M.A. & McNaughton, B.L. "Reactivation of Hippocampal Ensemble Memories During Sleep." *Science*, 265, 1994. — *Demonstrates hippocampal replay during sleep; the biological basis for dream-state episode consolidation.*
- [BOWER-1981] Bower, G.H. "Mood and Memory." *American Psychologist*, 36(2), 1981. — *Establishes mood-congruent memory; directly implemented in the four-factor Grimoire retrieval scoring.*
- [EMOTIONAL-RAG-2024] Zhang, Y. et al. "Emotional RAG." arXiv:2410.23041, 2024. — *Validates computationally that emotion-tagged retrieval outperforms non-emotional retrieval.*
- [KAHNEMAN-2011] Kahneman, D. *Thinking, Fast and Slow.* FSG, 2011. — *Dual-process theory: System 1 (fast, heuristic) vs. System 2 (slow, deliberate). The cognitive model behind T0/T1/T2 gating.*
- [FRISTON-2010] Friston, K. "The Free-Energy Principle: A Unified Brain Theory?" *Nature Reviews Neuroscience*, 11(2), 2010. — *Precision-weighted prediction error as the brain's organizing principle; directly implemented in Step 4 (GATE).*
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." *Journal of Monetary Economics*, 50(3), 2003. — *Formalizes that information processing is costly; grounds the adaptive threshold that decides when to invoke the LLM.*
- [CHEN-2023] Chen, L. et al. "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." arXiv:2305.05176, 2023. — *Demonstrates up to 98% cost reduction through intelligent model cascading; validates the gating architecture.*
- [DAMASIO-1994] Damasio, A. *Descartes' Error: Emotion, Reason, and the Human Brain.* Putnam, 1994. — *Somatic marker hypothesis: emotions as rapid decision heuristics. Foundational for the Daimon's role in gating.*
- [PEARL-2009] Pearl, J. *Causality: Models, Reasoning, and Inference.* Cambridge, 2nd ed., 2009. — *Formalizes causal reasoning; the basis for the causal graph used in prediction error computation.*
- [YERKES-DODSON-1908] Yerkes, R.M. & Dodson, J.D. "The Relation of Strength of Stimulus to Rapidity of Habit-Formation." *Journal of Comparative Neurology and Psychology*, 18(5), 1908. — *Demonstrates the inverted-U relationship between arousal and performance; informs the Daimon's arousal-to-deliberation threshold mapping.*
- [LAIRD-2012] Laird, J.E. *The Soar Cognitive Architecture.* MIT Press, 2012. — *The canonical Soar reference; one of CoALA's intellectual ancestors for production-rule cognitive architectures.*
- [ANDERSON-2007] Anderson, J.R. *How Can the Human Mind Occur in the Physical Universe?* Oxford, 2007. — *The ACT-R cognitive architecture; informs the distinction between episodic and semantic memory stores.*
- [NEWELL-1990] Newell, A. *Unified Theories of Cognition.* Harvard, 1990. — *Foundational work arguing for unified cognitive architectures; supports the single-loop heartbeat design.*
- [CLARK-CHALMERS-1998] Clark, A. & Chalmers, D. "The Extended Mind." *Analysis*, 58(1), 1998. — *Argues that cognition extends beyond the brain into tools and environment; supports the Golem's tool-augmented cognition model.*
- Beer, S. (1984). "The Viable System Model." *JORS*, 35(1), 7--25. — *Cybernetic model for organizational self-regulation; informs the three self-tuning loops in the heartbeat pipeline.*
- Argyris, C. & Schon, D. (1978). *Organizational Learning*. Addison-Wesley. — *Introduces single-loop and double-loop learning; maps to Loop 1 (tactical) and Loop 2 (strategic) in the heartbeat.*
- Bateson, G. (1972). *Steps to an Ecology of Mind*. Chandler. — *Introduces levels of learning (Learning I, II, III); informs the meta-learning dimension of dream consolidation.*
- Weston, J. & Sukhbaatar, S. (2024). "Distilling System 2 into System 1." arXiv:2407.06023. — *Shows slow deliberation can be distilled into fast heuristics; validates the PLAYBOOK.md extraction pipeline.*
- Zhang, H. et al. (2025). "DPT-Agent: Dual Process Theory for Language Agents." arXiv:2502.11882. — *Applies Kahneman's dual-process theory to agent design with explicit fast/slow subsystems.*
- Schneier, B. (2025). "OODA Loops for AI Agents." *IEEE Security & Privacy*. — *Maps Boyd's OODA loop to AI agent decision cycles; parallel framing to the heartbeat's observe-decide-act-learn pattern.*
- [HERBSTER-1998] Herbster, M. & Warmuth, M.K. "Tracking the Best Expert." *Machine Learning*, 32(2), 1998. — *Online learning algorithm for tracking non-stationary experts; informs adaptive threshold adjustment.*
- [THOMPSON-1933] Thompson, W.R. "On the Likelihood that One Unknown Probability Exceeds Another." *Biometrika*, 25(3-4), 1933. — *Thompson sampling: Bayesian exploration/exploitation for bandit problems. Used in adaptive triage threshold tuning.*
- [SHALEV-SHWARTZ-2011] Shalev-Shwartz, S. "Online Learning and Online Convex Optimization." *Foundations and Trends in ML*, 4(2), 2011. — *Survey of online convex optimization; theoretical foundation for the adaptive triage learning algorithms.*
- [FREUND-SCHAPIRE-1997] Freund, Y. & Schapire, R.E. "A Decision-Theoretic Generalization of On-Line Learning and an Application to Boosting." *Journal of Computer and System Sciences*, 55(1), 1997. — *Introduces the Hedge algorithm for combining expert signals with provable regret bounds; used in the triage pipeline.*
- [LI-2010] Li, L., Chu, W., Langford, J. & Schapire, R.E. "A Contextual-Bandit Approach to Personalized News Article Recommendation." *WWW*, 2010. — *LinUCB: contextual bandit algorithm for context-dependent routing decisions.*
- [AUER-2002] Auer, P., Cesa-Bianchi, N., Freund, Y. & Schapire, R.E. "The Nonstochastic Multiarmed Bandit Problem." *SIAM Journal on Computing*, 32(1), 2002. — *Adversarial bandit formulation; theoretical basis for regret-bounded triage adaptation under non-stationary markets.*
- [SUTTON-BARTO-2018] Sutton, R.S. & Barto, A.G. *Reinforcement Learning: An Introduction*, 2nd ed. MIT Press, 2018. — *The canonical RL reference; informs the reward signal design for triage feedback loops.*
- [LITTLESTONE-WARMUTH-1994] Littlestone, N. & Warmuth, M.K. "The Weighted Majority Algorithm." *Information and Computation*, 108(2), 1994. — *Foundational online learning algorithm with multiplicative weight updates; ancestor of the Hedge algorithm used in triage.*

---

## Online Learning for Adaptive Triage

> Source: `06-curiosity-learning/02-online-learning.md`

The triage pipeline's thresholds and signal weights are currently static. The heuristic-to-learned weight shifts linearly with episode count; the score routing brackets (>0.8, 0.5-0.8, 0.2-0.5, <0.2) are hardcoded; and the relative importance of different curiosity signals is fixed at deploy time. This section introduces four online learning algorithms -- Hedge, Thompson sampling, LinUCB, and epsilon-greedy with decay -- that let the triage pipeline adapt these parameters based on downstream feedback. Each algorithm fills a different niche: Hedge combines multiple expert signals with provable regret bounds; Thompson sampling balances exploration and exploitation of triage thresholds; LinUCB adds context-dependent routing; and epsilon-greedy provides a dead-simple baseline for comparison.

All four algorithms share a common feedback loop: the golem takes an action (escalate, route, discard), observes the outcome (LLM analysis was useful, trade was profitable, event was irrelevant), and updates the algorithm's internal state. The feedback is delayed (outcomes arrive at Theta or Delta tick, not Gamma) and partial (discarded events never receive feedback). Both issues are addressed below.

### Hedge / Exponential Weights

Hedge (Freund & Schapire, 1997) maintains a weight for each "expert" -- in our case, each curiosity signal. At each round, the algorithm observes each expert's loss and multiplicatively updates weights:

```
w_i(t+1) = w_i(t) * exp(-eta * loss_i(t))
normalize: w_i(t+1) = w_i(t+1) / sum(w_j(t+1))
```

The parameter eta controls learning rate. With eta = sqrt(ln(N) / T) for N experts and T rounds, Hedge guarantees O(sqrt(T * ln(N))) regret -- convergence to the best fixed expert in hindsight. For N=5 signals and T=10,000 triage events, this translates to ~28 nats of cumulative regret over the golem's lifetime. In practice, it converges to good weights within a few hundred observations.

#### Full HedgeWeights Implementation

```rust
/// Hedge algorithm for combining curiosity signals.
pub struct HedgeWeights {
    weights: Vec<f64>,
    eta: f64,
    /// Track cumulative loss per expert for diagnostics
    cumulative_loss: Vec<f64>,
    rounds: u64,
}

impl HedgeWeights {
    pub fn new(num_experts: usize, eta: f64) -> Self {
        let uniform = 1.0 / num_experts as f64;
        Self {
            weights: vec![uniform; num_experts],
            eta,
            cumulative_loss: vec![0.0; num_experts],
            rounds: 0,
        }
    }

    /// Compute weighted combination of expert scores.
    pub fn combine(&self, scores: &[f64]) -> f64 {
        assert_eq!(scores.len(), self.weights.len());
        self.weights
            .iter()
            .zip(scores.iter())
            .map(|(w, s)| w * s)
            .sum()
    }

    /// Update weights after observing loss for each expert.
    /// Loss should be in [0, 1] -- lower is better.
    ///
    /// Loss computation: after Theta LLM analysis, compare each expert's
    /// prediction (did this expert say the event was interesting?) against
    /// the ground truth (was the LLM analysis actionable?).
    pub fn update(&mut self, losses: &[f64]) {
        assert_eq!(losses.len(), self.weights.len());

        for (i, loss) in losses.iter().enumerate() {
            self.weights[i] *= (-self.eta * loss).exp();
            self.cumulative_loss[i] += loss;
        }

        // Normalize
        let total: f64 = self.weights.iter().sum();
        if total > 0.0 {
            for w in self.weights.iter_mut() {
                *w /= total;
            }
        }
        self.rounds += 1;
    }

    /// Current weight distribution (for diagnostics and logging).
    pub fn distribution(&self) -> &[f64] {
        &self.weights
    }

    /// Adaptive eta based on round count.
    /// Follows the theoretical optimal: eta = sqrt(ln(N) / t)
    pub fn adapt_eta(&mut self) {
        let n = self.weights.len() as f64;
        let t = (self.rounds + 1) as f64;
        self.eta = (n.ln() / t).sqrt();
    }
}
```

#### Computing Expert Loss

The feedback signal comes from Theta tick LLM analysis. When the LLM evaluates a triage event, it assigns an importance score. The loss for each expert is the absolute difference between the expert's score and the LLM's importance assessment:

```rust
/// Compute per-expert loss after LLM evaluation.
fn compute_expert_losses(
    expert_scores: &[f64],   // What each signal scored the event
    llm_importance: f64,      // What the LLM concluded
) -> Vec<f64> {
    expert_scores
        .iter()
        .map(|&score| (score - llm_importance).abs().min(1.0))
        .collect()
}
```

Experts that scored the event close to the LLM's assessment get low loss and maintain or increase their weight. Experts that were far off get high loss and are downweighted. Over time, the combination converges toward the signals that best predict LLM-assessed importance.

#### Hedge for Heuristic Selection

A second, nested Hedge instance can weight individual heuristic rules within the heuristic score. Each heuristic rule (active position counterparty, known protocol interaction, large value, etc.) is an expert. The heuristic score becomes:

```rust
fn score_heuristic_hedged(
    tx: &Transaction,
    logs: &[DecodedLog],
    scope: &ChainScope,
    heuristic_hedge: &HedgeWeights,
) -> f64 {
    let rule_scores = vec![
        if scope.active_positions.contains(&tx.from) { 1.0 } else { 0.0 },
        if logs.iter().any(|l| l.protocol_id.is_some()) { 1.0 } else { 0.0 },
        if tx.value > scope.large_value_threshold_wei { 1.0 } else { 0.0 },
        if scope.recent_counterparties.contains(&tx.from) { 1.0 } else { 0.0 },
        if tx.gas_used > 500_000 { 1.0 } else { 0.0 },
        if tx.creates.is_some() { 1.0 } else { 0.0 },
    ];
    heuristic_hedge.combine(&rule_scores).min(1.0)
}
```

This replaces the hand-tuned additive weights (0.8, 0.7, 0.6, ...) with learned weights that reflect which heuristics actually predict interesting events for this specific golem's strategy.

### Thompson Sampling

Thompson sampling (Thompson, 1933) maintains a probability distribution over each parameter's "goodness" and samples from it to make decisions. Where Hedge learns fixed weights, Thompson sampling explores -- sometimes choosing actions that look suboptimal in order to learn whether they're actually better than expected.

#### Full ThresholdArm Implementation

```rust
use rand::Rng;
use rand_distr::Beta as BetaDist;

/// Thompson sampling for triage threshold exploration.
pub struct ThompsonThresholds {
    /// Each arm represents a candidate threshold configuration.
    arms: Vec<ThresholdArm>,
}

#[derive(Clone, Debug)]
pub struct ThresholdArm {
    /// The threshold values this arm proposes
    pub high_threshold: f32,    // above this: escalate to LLM
    pub medium_threshold: f32,  // above this: emit ChainEvent
    pub low_threshold: f32,     // above this: update silently; below: discard

    /// Beta posterior tracking whether this configuration produces
    /// useful escalations (true positives) at a reasonable rate.
    pub alpha: f64,  // successes: escalated events that LLM found actionable
    pub beta: f64,   // failures: escalated events that LLM found irrelevant
}

impl ThompsonThresholds {
    pub fn new() -> Self {
        let arms = vec![
            ThresholdArm { high_threshold: 0.8, medium_threshold: 0.5, low_threshold: 0.2, alpha: 2.0, beta: 2.0 },
            ThresholdArm { high_threshold: 0.7, medium_threshold: 0.4, low_threshold: 0.15, alpha: 1.0, beta: 1.0 },
            ThresholdArm { high_threshold: 0.85, medium_threshold: 0.55, low_threshold: 0.25, alpha: 1.0, beta: 1.0 },
            ThresholdArm { high_threshold: 0.75, medium_threshold: 0.45, low_threshold: 0.2, alpha: 1.0, beta: 1.0 },
            ThresholdArm { high_threshold: 0.9, medium_threshold: 0.6, low_threshold: 0.3, alpha: 1.0, beta: 1.0 },
        ];
        Self { arms }
    }

    pub fn select_thresholds(&self, rng: &mut impl Rng) -> &ThresholdArm {
        let mut best_idx = 0;
        let mut best_sample = f64::NEG_INFINITY;

        for (i, arm) in self.arms.iter().enumerate() {
            let dist = BetaDist::new(arm.alpha, arm.beta).unwrap();
            let sample: f64 = rng.sample(dist);
            if sample > best_sample {
                best_sample = sample;
                best_idx = i;
            }
        }

        &self.arms[best_idx]
    }

    pub fn update(&mut self, arm_idx: usize, success: bool) {
        if success {
            self.arms[arm_idx].alpha += 1.0;
        } else {
            self.arms[arm_idx].beta += 1.0;
        }
    }

    /// Decay all arms slightly to allow adaptation over time.
    /// Without decay, early observations dominate forever.
    pub fn decay(&mut self, factor: f64) {
        for arm in self.arms.iter_mut() {
            arm.alpha = 1.0 + (arm.alpha - 1.0) * factor;
            arm.beta = 1.0 + (arm.beta - 1.0) * factor;
        }
    }
}
```

#### Arousal-Conditioned Thompson Sampling

The golem's arousal state (from CorticalState) should influence threshold selection. During high arousal (market crisis, exploit detected), the golem should use lower thresholds -- escalate more events. During low arousal, higher thresholds -- be more selective.

Thompson sampling integrates this naturally by maintaining separate Beta posteriors per arousal regime:

```rust
pub struct ContextualThompson {
    /// Separate threshold arms per arousal regime
    regimes: HashMap<ArousalRegime, ThompsonThresholds>,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ArousalRegime {
    Low,      // arousal < 0.3
    Medium,   // 0.3 <= arousal < 0.7
    High,     // arousal >= 0.7
}

impl ContextualThompson {
    pub fn select(&self, arousal: f32, rng: &mut impl Rng) -> &ThresholdArm {
        let regime = match arousal {
            a if a < 0.3 => ArousalRegime::Low,
            a if a < 0.7 => ArousalRegime::Medium,
            _ => ArousalRegime::High,
        };
        self.regimes[&regime].select_thresholds(rng)
    }
}
```

The golem learns, for instance, that lower thresholds perform well during high arousal (more true positives justify the extra LLM cost) and higher thresholds perform well during low arousal (most events are routine, escalation is wasteful).

### Design Alternative: LinUCB (Not Adopted)

> **Rationale for non-adoption:** LinUCB requires feature vectors per arm and is over-engineered for a 3-arm routing decision. Thompson sampling is the right tool for this problem: it naturally balances exploration and exploitation for small action spaces with Beta-distributed rewards. LinUCB is documented here for completeness and future reference.

LinUCB (Li et al., 2010) extends UCB to contextual bandits. Instead of maintaining a single reward estimate per arm, it maintains a linear model that predicts reward as a function of context features. The confidence bound comes from ridge regression uncertainty:

```
For context x and arm a:
  A_a = I + sum(x_t * x_t^T)       -- d x d matrix
  b_a = sum(r_t * x_t)             -- d-vector
  theta_a = A_a^{-1} * b_a          -- parameter estimate
  UCB_a = theta_a^T * x + alpha * sqrt(x^T * A_a^{-1} * x)
```

The alpha parameter controls exploration. Higher alpha means wider confidence bounds, more exploration of uncertain contexts.

```rust
use nalgebra::{DMatrix, DVector};

/// LinUCB contextual bandit for triage routing.
pub struct LinUcbRouter {
    /// Per-arm parameters
    arms: Vec<LinUcbArm>,
    /// Exploration parameter
    alpha: f64,
    /// Context dimension
    d: usize,
}

pub struct LinUcbArm {
    /// A = I + sum(x_t * x_t^T)
    a_matrix: DMatrix<f64>,
    /// b = sum(r_t * x_t)
    b_vector: DVector<f64>,
    /// Cached theta = A^{-1} * b (recomputed on update)
    theta: DVector<f64>,
    /// Cached A^{-1} (updated via Sherman-Morrison)
    a_inv: DMatrix<f64>,
}

impl LinUcbArm {
    pub fn new(d: usize) -> Self {
        let identity = DMatrix::identity(d, d);
        Self {
            a_matrix: identity.clone(),
            b_vector: DVector::zeros(d),
            theta: DVector::zeros(d),
            a_inv: identity,
        }
    }

    /// Compute UCB score for a context vector.
    pub fn ucb(&self, x: &DVector<f64>, alpha: f64) -> f64 {
        let exploitation = self.theta.dot(x);
        let exploration = alpha * (x.transpose() * &self.a_inv * x)[(0, 0)].sqrt();
        exploitation + exploration
    }

    /// Update with observed reward.
    pub fn update(&mut self, x: &DVector<f64>, reward: f64) {
        // Sherman-Morrison update for A^{-1}
        let a_inv_x = &self.a_inv * x;
        let denom = 1.0 + (x.transpose() * &a_inv_x)[(0, 0)];
        self.a_inv -= (&a_inv_x * a_inv_x.transpose()) / denom;

        self.a_matrix += x * x.transpose();
        self.b_vector += reward * x;
        self.theta = &self.a_inv * &self.b_vector;
    }
}

impl LinUcbRouter {
    pub fn new(d: usize, num_arms: usize, alpha: f64) -> Self {
        Self {
            arms: (0..num_arms).map(|_| LinUcbArm::new(d)).collect(),
            alpha,
            d,
        }
    }

    /// Build context vector from a triage event.
    pub fn build_context(event: &TriageEvent, cortical: &CorticalState) -> DVector<f64> {
        // d = 12 context features
        DVector::from_vec(vec![
            event.curiosity_score as f64,
            event.bayesian_surprise,
            event.prediction_error,
            event.anomaly_score,
            cortical.arousal as f64,
            cortical.valence as f64,
            if event.involves_active_position { 1.0 } else { 0.0 },
            if event.protocol_id.is_some() { 1.0 } else { 0.0 },
            event.gas_ratio,   // gas_used / block_avg_gas
            event.value_usd.log10().max(0.0),
            cortical.chain_blocks_behind as f64,
            event.time_since_last_escalation_secs,
        ])
    }

    /// Select the best arm (routing action) for the given context.
    pub fn select(&self, context: &DVector<f64>) -> usize {
        self.arms
            .iter()
            .enumerate()
            .map(|(i, arm)| (i, arm.ucb(context, self.alpha)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Update the selected arm with observed reward.
    pub fn update(&mut self, arm_idx: usize, context: &DVector<f64>, reward: f64) {
        self.arms[arm_idx].update(context, reward);
    }
}
```

LinUCB's advantage over Hedge and Thompson sampling is that it conditions on context. The same curiosity score of 0.6 might warrant escalation when the golem has an active position in the affected protocol (high stakes) but not when the protocol is unrelated (low stakes). LinUCB learns these conditional patterns.

The cost is higher: a d x d matrix per arm, O(d^2) per update. At d=12 and 3 arms, this is 432 f64 values -- 3.4KB total. The Sherman-Morrison update avoids full matrix inversion and runs in O(d^2) time, about 1 microsecond at d=12.

### Design Alternative: Epsilon-Greedy (Not Adopted)

> **Rationale for non-adoption:** Epsilon-greedy provides insufficient exploration guarantees compared to Thompson sampling. It exists in this design for two reasons: (1) it's the comparison baseline -- if Hedge or Thompson sampling doesn't beat epsilon-greedy on a given golem's workload, the complexity isn't justified; (2) it's the fallback -- if the more sophisticated algorithms encounter numerical issues (degenerate matrices in LinUCB, NaN in Hedge weights), epsilon-greedy continues to function.

```rust
pub struct EpsilonGreedy {
    epsilon: f64,
    epsilon_min: f64,
    decay_rate: f64,
    /// Mean reward per arm
    arm_means: Vec<f64>,
    arm_counts: Vec<u64>,
}

impl EpsilonGreedy {
    pub fn new(num_arms: usize, epsilon: f64, epsilon_min: f64, decay_rate: f64) -> Self {
        Self {
            epsilon,
            epsilon_min,
            decay_rate,
            arm_means: vec![0.0; num_arms],
            arm_counts: vec![0; num_arms],
        }
    }

    pub fn select(&self, rng: &mut impl Rng) -> usize {
        if rng.gen::<f64>() < self.epsilon {
            // Explore: random arm
            rng.gen_range(0..self.arm_means.len())
        } else {
            // Exploit: best arm by mean reward
            self.arm_means
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0)
        }
    }

    pub fn update(&mut self, arm: usize, reward: f64) {
        self.arm_counts[arm] += 1;
        let n = self.arm_counts[arm] as f64;
        // Incremental mean update
        self.arm_means[arm] += (reward - self.arm_means[arm]) / n;

        // Decay epsilon
        self.epsilon = (self.epsilon * self.decay_rate).max(self.epsilon_min);
    }
}
```

Starting epsilon at 0.2 and decaying to 0.01 with rate 0.999 per round gives about 5000 rounds of meaningful exploration before settling into near-pure exploitation. For a golem processing ~100 triage events per Theta tick, that's about 50 Theta ticks -- roughly 4 hours of operation.

### AdaptiveTriageRouter Composition

The algorithms aren't mutually exclusive. The recommended configuration:

```rust
pub struct AdaptiveTriageRouter {
    /// Hedge combines the 5 curiosity signals into a single score
    pub signal_combiner: HedgeWeights,

    /// Thompson sampling selects threshold configurations
    pub threshold_selector: ContextualThompson,

    /// LinUCB makes the final routing decision using full context
    pub router: LinUcbRouter,

    /// Epsilon-greedy as comparison baseline (runs in shadow mode,
    /// not affecting actual routing, but logging what it would do)
    pub baseline: EpsilonGreedy,
}

impl AdaptiveTriageRouter {
    /// Score and route a triage event.
    pub fn process(
        &mut self,
        event: &TriageEvent,
        expert_scores: &[f64],
        cortical: &CorticalState,
        rng: &mut impl Rng,
    ) -> RoutingDecision {
        // 1. Hedge combines expert signals
        let combined_score = self.signal_combiner.combine(expert_scores);

        // 2. Thompson selects thresholds for this arousal regime
        let thresholds = self.threshold_selector.select(cortical.arousal, rng);

        // 3. Preliminary routing by threshold
        let preliminary = if combined_score > thresholds.high_threshold as f64 {
            RoutingAction::EscalateToLlm
        } else if combined_score > thresholds.medium_threshold as f64 {
            RoutingAction::EmitChainEvent
        } else if combined_score > thresholds.low_threshold as f64 {
            RoutingAction::UpdateSilently
        } else {
            RoutingAction::Discard
        };

        // 4. LinUCB can override for borderline cases
        let context = LinUcbRouter::build_context(event, cortical);
        let linucb_action = self.router.select(&context);

        // 5. Shadow-run epsilon-greedy for baseline comparison
        let _baseline_action = self.baseline.select(rng);

        // 6. Final decision: use LinUCB for borderline, threshold for clear cases
        let final_action = if (combined_score - thresholds.high_threshold as f64).abs() < 0.1 {
            // Borderline: let LinUCB decide
            RoutingAction::from_index(linucb_action)
        } else {
            preliminary
        };

        RoutingDecision {
            action: final_action,
            combined_score,
            context,
            threshold_arm_idx: 0, // track for Thompson update
        }
    }
}
```

### The Feedback Loop

All four algorithms share the same feedback structure:

```
[Gamma tick: triage event scored by 5 experts]
    |
    v
[Online learner selects routing action]
    |
    v
[Event routed: escalate / update / discard]
    |
    ... (minutes pass) ...
    |
[Theta tick: LLM evaluates escalated events]
    |
    v
[Outcome observed: was escalation useful?]
    |
    v
[Online learner updated with (context, action, reward)]
```

The delayed feedback (minutes between action and outcome) doesn't affect algorithm correctness -- it means the learner updates less frequently than it acts. The partial feedback (discarded events never get LLM analysis) is the harder problem. Three mitigation strategies:

**Strategy 1: Epsilon-exploration of discards.** With probability epsilon, route a below-threshold event to LLM analysis anyway. This provides counterfactual labels for the low-score region. Set epsilon low (0.01-0.05) to bound the LLM cost.

**Strategy 2: Implicit negative signal.** Events that are discarded and never re-surface (no retroactive re-categorization, no position impact) are treated as true negatives after a time window. If nothing interesting happened to the discarded event's protocol in the next N blocks, the discard was correct.

**Strategy 3: Retroactive re-scoring.** When ABI resolution identifies a previously unknown contract (Stage 3 fingerprinter), retroactively re-score its past events. Events whose score changes significantly are treated as missed positives, providing negative reward to the routing action that discarded them.

### Persistence Across Golem Generations

Online learning state is small (a few KB total) and represents hard-won knowledge about which signals predict interesting events. This state should be included in the generational inheritance package:

- **Hedge weights**: Which curiosity signals matter? ~40 bytes.
- **Thompson posteriors**: Which thresholds work in which arousal regimes? ~240 bytes.
- **LinUCB parameters**: Full A^{-1} and theta per arm. ~3.4KB.
- **Epsilon-greedy means**: Baseline arm rewards. ~24 bytes.

Total: **~3.7KB** inheritance budget. A successor golem inheriting these parameters starts with calibrated signal weights and threshold configurations rather than re-learning from scratch. The decay mechanisms in Thompson sampling and the adaptive eta in Hedge ensure the successor can still adapt to changed conditions.

### Online Learning References

- Thompson, W.R. (1933). "On the Likelihood that One Unknown Probability Exceeds Another in View of the Evidence of Two Samples." *Biometrika*, 25(3/4), 285-294.
- Littlestone, N. & Warmuth, M.K. (1994). "The Weighted Majority Algorithm." *Information and Computation*, 108(2), 212-261.
- Freund, Y. & Schapire, R.E. (1997). "A Decision-Theoretic Generalization of On-Line Learning and an Application to Boosting." *Journal of Computer and System Sciences*, 55(1), 119-139.
- Auer, P., Cesa-Bianchi, N., Freund, Y. & Schapire, R.E. (2002). "The Nonstochastic Multiarmed Bandit Problem." *SIAM Journal on Computing*, 32(1), 48-77.
- Li, L., Chu, W., Langford, J. & Schapire, R.E. (2010). "A Contextual-Bandit Approach to Personalized News Article Recommendation." *WWW*, 661-670.
- Shalev-Shwartz, S. (2011). "Online Learning and Online Convex Optimization." *Foundations and Trends in Machine Learning*, 4(2), 107-194.
- Sutton, R.S. & Barto, A.G. (2018). *Reinforcement Learning: An Introduction*, 2nd ed. MIT Press.
- Herbster, M. & Warmuth, M.K. (1998). "Tracking the Best Expert." *Machine Learning*, 32(2), 151-178.

---

*Each tick is a complete cognitive act -- observe, feel, decide, act, learn. The rhythm continues until the clocks run out.*
