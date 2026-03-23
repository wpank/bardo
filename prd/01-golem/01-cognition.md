# Cognition: The Inference Engine [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-inference` (routing.rs, x402.rs, budget.rs), `golem-context` (workspace.rs)
>
> **Prerequisites**: Read `02-heartbeat.md` (decision cycle, T0/T1/T2 tiers), `../04-memory/01-grimoire.md` (retrieval), `05-mortality.md` (attention budget).

> **Reader orientation:** This document specifies the Golem's inference engine -- how it decides when and how to think. It belongs to the `01-golem` cognition layer and covers the three-tier cognitive gating system (T0/T1/T2), x402 (micropayment protocol where agents pay for inference/compute/data via signed USDC transfers, no API keys) as the economic primitive, and Bardo Inference (x402-gated LLM gateway routing between Anthropic/OpenAI/Google/Venice/Grok). The key prerequisite: understand the Heartbeat (the 9-step decision cycle) from `02-heartbeat.md` (the 9-step decision cycle each Golem executes on every tick). Full term definitions are in `prd2/shared/glossary.md` (canonical Bardo term definitions).

---

## The Think Moat

This document specifies the second of six structural moats (see `00-overview.md`): the ability to **Think** efficiently. The three-tier cognitive gating system (T0/T1/T2) reduces inference cost by 18x while routing full deliberation to the ticks that actually need it. Bardo Inference's 8-layer context engineering pipeline adds another 6x. Combined: ~230x cost reduction under normal market conditions. No competing framework has an equivalent gating architecture tied to mortality pressure.

---

## S1 -- x402 as Economic Primitive

Every Golem already has a wallet. x402 turns that wallet into payment credentials for LLM inference. No API keys, no accounts, no billing setup. The wallet signs an EIP-3009 `transferWithAuthorization` message on USDC (Base), the gateway verifies payment before forwarding to the upstream LLM provider, and the response streams back over SSE. One HTTP request, one micropayment, one response.

The protocol:

1. Client sends request with pre-computed `X-402-Payment` header (signed EIP-3009 payload).
2. Gateway validates signature and checks the Facilitator contract's idempotency registry.
3. Facilitator calls `USDC.transferWithAuthorization()` on Base, splitting payment: 90% to upstream provider, 10% to Bardo treasury (cross-user only; intra-Clade is free).
4. Gateway forwards request to upstream LLM.
5. Response streams back via SSE.
6. If payment was insufficient (response longer than estimated), gateway returns HTTP 402 with updated requirements and the client retries.

```rust
/// EIP-3009 TransferWithAuthorization parameters for x402 inference payments.
///
/// The Golem's wallet signs this payload to authorize a USDC transfer
/// to the Facilitator contract. The gateway verifies the signature
/// and submits it on-chain before forwarding the inference request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransferWithAuthorizationParams {
    /// Agent wallet (payer).
    pub from: alloy::primitives::Address,
    /// Facilitator contract (payee).
    pub to: alloy::primitives::Address,
    /// USDC amount (6 decimals).
    pub value: alloy::primitives::U256,
    /// Unix timestamp — earliest valid time.
    pub valid_after: alloy::primitives::U256,
    /// Unix timestamp — latest valid time (5-min TTL).
    pub valid_before: alloy::primitives::U256,
    /// Random nonce (prevents replay).
    pub nonce: alloy::primitives::FixedBytes<32>,
}
```

Every payment is keyed by `(golem_id, tick_number, step_name)`. Duplicate nonces are deduplicated by the Facilitator's `mapping(bytes32 => bool)` registry. Transient network errors retry safely.

**Why x402 over API keys**: The wallet is the identity. Agents already have one. x402 eliminates API key management, enables per-request micropayment in USDC, enables semantic caching at the gateway, enables per-strategy cost attribution, and enables automatic provider fallback. Direct API keys remain a fallback for owners who want them.

| Feature | Direct API Key | Bardo Inference |
|---|---|---|
| Setup | Create account, add billing, generate key | Just a wallet with USDC on Base |
| Payment | Monthly invoice, credit card | Per-request micropayment, USDC |
| Semantic cache | No | 15--30% cost savings via embedding similarity |
| Prompt cache routing | Manual | Provider-sticky routing maximizes cache hits |
| Automatic fallbacks | No | Cascade across providers on failure |
| Cost tracking | Per-provider dashboards | Unified per-strategy attribution |
| Agent-native | Requires API key management | Wallet is the identity -- agents already have one |

### Payment Failure Handling

| Failure | Response |
|---|---|
| Insufficient balance | Downgrade inference tier (T2 -> T1 -> T0) |
| Facilitator paused | Fall back to API-key billing |
| LLM provider down | Cascade to next provider in priority |
| Expired signature | Re-sign once, then fail |
| Rate limit | Exponential backoff (1s, 2s, 4s, max 30s) |

### 1.1 Payment Flow

```
Client (Golem)                    Gateway                    Facilitator           Upstream LLM
     |                               |                           |                      |
     |-- POST /v1/chat/completions ->|                           |                      |
     |   X-402-Payment: {auth, sig}  |                           |                      |
     |   X-402-Metadata: {strategy}  |                           |                      |
     |                               |-- verify payment -------->|                      |
     |                               |   transferWithAuth(sig)   |                      |
     |                               |<-- payment confirmed -----|                      |
     |                               |                           |                      |
     |                               |-- forward request --------------------------->|
     |                               |                           |                      |
     |<-- SSE stream --------------- |<-- SSE stream --------------------------------|
     |                               |                           |                      |
     |                               |-- record cost attribution |                      |
     |                               |   (strategy, tick, tier)  |                      |
```

### 1.2 Facilitator Contract

The Facilitator is a smart contract on Base that mediates all x402 payments. It receives the `transferWithAuthorization` signature from the gateway, calls `USDC.transferWithAuthorization()`, splits the payment (90% upstream provider, 10% Bardo treasury for cross-user requests; 0% for intra-Clade), and emits a `PaymentProcessed(from, to, value, idempotencyKey)` event for reconciliation.

```rust
/// Facilitator configuration for x402 payment routing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FacilitatorConfig {
    pub facilitator: alloy::primitives::Address,
    pub usdc: alloy::primitives::Address,
    pub treasury: alloy::primitives::Address,
    /// 1000 = 10%.
    pub cross_user_fee_bps: u32,
    /// 0 = free for intra-Clade.
    pub intra_clade_fee_bps: u32,
}
```

---

## S2 -- Three-Tier Model Routing

Model routing is a survival decision. Choosing Opus over Haiku costs $0.25 versus $0.003 -- an 83x difference. The LLM partition receives 60% of the credit budget (see [02-mortality/](../02-mortality/)), so inference spending directly determines lifespan. An unnecessary Opus call at $0.25 burns the same budget as 83 Haiku calls or 1.25 days of life at $0.20/day.

Based on FrugalGPT [CHEN-2023]: don't use an expensive model when a cheap one suffices. See [../12-inference/01-routing.md](../12-inference/01-routing.md) for the full routing specification and [../12-inference/12-providers.md](../12-inference/12-providers.md) for the provider resolution algorithm.

Tiers gate WHEN the LLM fires. Intents determine WHICH model and provider handle the call. Each subsystem declares an intent — what features and quality it needs. The resolver matches intents against the ordered provider list.

| Tier | Model Class | Approximate Cost | When Used | What the LLM Sees |
|------|-------------|-----------------|-----------|-------------------|
| **T0** | None | $0.00 | PE < threshold (~80% of ticks) | Nothing -- no LLM call at all |
| **T1** | Haiku-class | $0.001--0.003/call | PE in [theta, 2*theta) (~15% of ticks) | Reduced workspace: observation, top-5 entries, positions, warnings |
| **T2** | Sonnet/Opus-class | $0.01--0.25/call | PE >= 2*theta or forced (~5% of ticks) | Full Cognitive Workspace |

```rust
/// The `model-router` extension. Reads cognitive tier from the heartbeat's
/// gating step, constructs an Intent for the current subsystem, applies
/// mortality pressure, and resolves against the ordered provider list.
///
/// Extension #18 in the runtime extension chain (Layer 3: Safety).
pub struct ModelRouter;

impl Extension for ModelRouter {
    fn name(&self) -> &str { "model-router" }
    fn layer(&self) -> u8 { 3 }

    async fn on_before_agent_start(&self, ctx: &mut AgentStartCtx) -> Result<()> {
        let state = ctx.golem_state();
        let tier = state.heartbeat.cognitive_tier;

        if tier == CognitiveTier::T0 {
            return Ok(()); // No LLM call — FSM handles it
        }

        // Look up the subsystem intent
        let subsystem = ctx.current_subsystem();
        let mut intent = subsystem_intent(subsystem);

        // Apply mortality pressure (exempt: risk, death, operator)
        apply_mortality_pressure(&mut intent, state.mortality.vitality);

        // Resolve against ordered provider list — first match wins
        let resolution = resolve(&state.providers, &intent)
            .ok_or_else(|| anyhow!("No provider for intent: {}", intent.subsystem))?;

        // Set model on the session
        ctx.set_model(&resolution.model, &resolution.provider);

        // Log degradation for operator visibility
        if !resolution.degraded.is_empty() {
            ctx.emit_warning(format!(
                "{} routed to {}/{} (unavailable: {})",
                intent.subsystem, resolution.provider, resolution.model,
                resolution.degraded.join(", ")
            ));
        }

        Ok(())
    }
}
```

### 2.1 Subsystem Intents

Each subsystem declares a static intent. Heartbeat tiers map to intents directly: T0 uses `heartbeat_t0` (no LLM), T1 uses `heartbeat_t1`, T2 uses `heartbeat_t2`. Non-heartbeat subsystems (risk, dream, daimon, context, curator, playbook, operator, death) bypass tier gating and use their own intents. Full 13-intent const declarations in [../12-inference/12-providers.md](../12-inference/12-providers.md).

| Subsystem | Quality | Key Preferences | Cost Sensitivity |
|---|---|---|---|
| heartbeat_t1 | low | low_effort | 0.8 |
| heartbeat_t2 | high | interleaved_thinking, citations | 0.3 |
| risk | maximum | interleaved_thinking, citations | 0.0 (never reduced) |
| dream | high | visible_thinking, privacy | 0.5 |
| daimon | low | privacy | 0.9 |
| daimon_complex | high | visible_thinking, privacy | 0.5 |
| context | medium | inline_think_toggle | 0.5 |
| curator | medium | structured_outputs, citations | 0.5 |
| playbook | medium | predicted_outputs | 0.6 |
| operator | maximum | interleaved_thinking, citations | 0.0 (never reduced) |
| death | maximum | visible_thinking (required), privacy | 0.0 |
| session_compact | medium | compaction | 0.5 |

Death is the only subsystem with a hard requirement (`require: ["visible_thinking"]`). All others use soft preferences. If no provider matches strictly, the resolver drops requirements to preferences on its second pass — `Resolution.degraded` lists what was lost.

### Feature-Based Model Selection

Within each tier, the intent's `require` and `prefer` fields drive which specific model and provider are selected:

| Subsystem | Key Preferences | Typical Resolution |
|---|---|---|
| Dreams | visible_thinking, privacy | Venice -> DeepSeek R1 (private visible reasoning) |
| Risk | interleaved_thinking, citations | BlockRun -> Claude Opus (tool-interleaved thinking) |
| Context | inline_think_toggle | Qwen -> `/think` for anomalous, `/no_think` for routine |
| PLAYBOOK | predicted_outputs | Direct OpenAI -> GPT-5.x (3x faster surgical edits) |
| Death | visible_thinking (required) | Venice -> DeepSeek R1 (only providers with visible thinking qualify) |
| Curator | structured_outputs, citations | BlockRun -> Claude Sonnet (structured analysis with provenance) |
| Operator | interleaved_thinking, citations | BlockRun -> Claude Opus (best conversational quality) |

Provider order matters. If Venice is first in the config, privacy-preferring intents resolve there. If Bardo Inference is first, cost-optimized intents resolve there. The user's config encodes their priorities without understanding the feature matching internals.

### 2.2 PAD-Modulated Routing

The Daimon's PAD mood vector (see [03-daimon/](../03-daimon/) for the affect engine specification) modulates three routing dimensions:

**Exploration.** Higher arousal increases the temperature parameter by +0.1, biasing generation toward more creative and divergent outputs. REM dream phases also use elevated temperature for exploratory hypothesis generation, distinct from the arousal-driven boost during waking ticks.

**Risk assessment.** Lower pleasure biases the router toward higher-tier models for safety-critical decisions. An anxious mood (low pleasure, high arousal) prefers T2 for trade execution and position changes. A confident mood (high pleasure, low arousal) relaxes the constraint, allowing T1 for routine analysis that would otherwise escalate.

**Tier preference.** The PAD vector feeds into the routing classifier as auxiliary features, shifting the escalation boundary. In the terminal phase, PAD modulation is overridden -- model routing is locked to budget-gated Haiku only, regardless of mood state.

```rust
/// PAD-based routing modifiers applied to the inference router.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PADRoutingModifiers {
    /// Range: [0.0, 0.1]. Added to temperature when arousal is high.
    pub temperature_delta: f64,
    /// Negative pleasure lowers T1->T2 threshold.
    pub escalation_bias: f64,
    /// False in terminal phase — mood modulation is disabled.
    pub active: bool,
}
```

| Mood State | Pleasure | Arousal | Routing Effect |
|---|---|---|---|
| Anxious | Low | High | Prefer T2 for decisions, +0.1 temperature |
| Confident | High | Low | Allow T1 for routine, baseline temperature |
| Excited | High | High | Allow T1 with +0.1 temperature (creative) |
| Depressed | Low | Low | Prefer T2 for decisions, baseline temperature |

---

## S3 -- The Cognitive Workspace

### Why Not Session Compaction?

Traditional agent frameworks grow a session (conversation history) until it hits the context window limit, then compress older messages via "compaction." This is reactive and lossy -- information is summarized under time pressure, and the summarization itself costs inference tokens.

The Golem takes a fundamentally different approach: the **Cognitive Workspace** is assembled fresh each tick from structured categories with learned token allocations. There is no growing session. There is no compaction. Each tick, the Context Governor builds the optimal context for the current situation from scratch.

### Baddeley's Working Memory Model as Architecture

This implements Alan Baddeley's (2000) working memory model -- one of the most influential theories in cognitive psychology [BADDELEY-2000]:

| Baddeley Component | Golem Implementation | What It Contains |
|--------------------|-----------------------|-----------------|
| **Central executive** | Context Governor | Allocates attention (tokens) across categories; learned by cybernetics self-tuning |
| **Episodic buffer** | The Workspace itself | Integrates information from all sources into a unified context |
| **Visuospatial sketchpad** | Observations + Positions | Current situational awareness: market state, portfolio, anomalies |
| **Phonological loop** | PLAYBOOK heuristics | Rehearsed procedural knowledge -- "inner speech" of trained responses |

The Cognitive Workspace paper (2025, arXiv:2508.13171) validated this approach computationally: "active memory management with deliberate information curation achieves 58.6% memory reuse rate compared to 0% for traditional RAG, with 17-18% net efficiency gain" [COGNITIVE-WORKSPACE-2025].

### The Workspace Struct

```rust
/// The Cognitive Workspace. Assembled fresh each decision cycle.
///
/// This is what the LLM "sees" — the complete context for deliberation.
/// Token allocation across categories is learned by the cybernetics
/// self-tuning system (§5) and modulated by the rational inattention
/// budget (05-mortality.md §6).
///
/// The workspace is NOT a growing conversation. It is a structured
/// snapshot built from multiple sources, sized to fit the current
/// cognitive tier's token budget, and optimized for the current
/// behavioral phase and market regime.
pub struct CognitiveWorkspace {
    // ═══ Invariants (always present, never compressed) ═══
    //
    // These are included at every tier (T1 and T2) because they represent
    // hard constraints that the LLM must respect regardless of context.

    /// On-chain PolicyCage constraints: approved assets, max positions,
    /// max drawdown, rebalance frequency limits. Read from the smart
    /// contract via Alloy at each tick.
    pub policy_cage: PolicyCageBlock,

    /// Current DeFi positions: LP ranges, lending positions, vault deposits.
    /// The LLM must know what the Golem currently holds.
    pub positions: Vec<PositionSnapshot>,

    /// Active risk warnings. These are never compressed or omitted.
    pub active_warnings: Vec<Warning>,

    /// Compiled strategy parameters from STRATEGY.md.
    pub strategy: StrategyBlock,

    /// Current affect state: PAD vector, discrete emotion, mood, phase.
    /// The LLM reads this to understand its own "emotional context."
    pub affect: AffectBlock,

    // ═══ Rehearsed Knowledge (always present, budget-allocated) ═══

    /// Top-N PLAYBOOK.md heuristics ranked by relevance to current observation.
    /// These are the Golem's "trained responses" — procedural knowledge
    /// distilled through dream consolidation.
    pub playbook_heuristics: Vec<PlaybookEntry>,

    /// 13 always-loaded mental models (Munger-inspired reasoning scaffolds).
    /// These don't change — they're static reasoning frameworks like
    /// "second-order thinking," "inversion," "margin of safety."
    pub mental_models: Vec<MentalModel>,

    // ═══ Retrieved Knowledge (budget-allocated per ContextPolicy) ═══
    //
    // These categories compete for token budget. The cybernetics system
    // learns which categories contribute most to good decisions and
    // allocates more tokens to them.

    /// Episodes retrieved by four-factor scoring (../04-memory/01-grimoire.md §4).
    pub retrieved_episodes: Vec<Episode>,

    /// Insights and heuristics from the Grimoire.
    pub retrieved_insights: Vec<GrimoireEntry>,

    /// Relevant causal graph edges (from BFS traversal, 03a §7).
    pub causal_edges: Vec<CausalEdge>,

    /// Mood-opposite entries forced every 100 ticks (anti-rumination).
    pub contrarian_entries: Vec<GrimoireEntry>,

    /// Hypotheses from dreams awaiting live validation.
    pub dream_hypotheses: Vec<StagedRevision>,

    /// Natural language "gut feeling" from the Somatic Landscape (04-daimon.md §4).
    pub somatic_landscape_reading: Option<String>,

    // ═══ Current Situation (refreshed each tick) ═══

    /// The market observation that triggered this tick's deliberation.
    pub observation: Observation,

    /// Mortality context: vitality components, phase, estimated TTL.
    pub mortality: MortalityBlock,

    /// Active steers and pending followUps.
    pub interventions: InterventionBlock,

    /// Pheromone Field summary: threat/opportunity signals from the swarm.
    pub pheromone_summary: Option<String>,

    // ═══ Conversation (only when user is interacting) ═══

    /// Last N messages from the conversation sidecar (if user is chatting).
    /// Most ticks this is None — the heartbeat runs autonomously.
    pub conversation_tail: Option<Vec<Message>>,

    // ═══ Meta ═══

    /// Total tokens used by this workspace.
    pub total_tokens: u32,

    /// Which revision of the learned ContextPolicy produced this workspace.
    pub policy_revision: u32,
}
```

### Assembly

```rust
/// Assemble the Cognitive Workspace for a given tick.
///
/// Uses the rational inattention budget (05-mortality.md) to allocate
/// tokens across categories. The budget is modified by:
/// - Behavioral phase (dying → less exploration, more invariants)
/// - Daily operational budget (running low → more conservative)
/// - Cybernetics policy revision (learned optimal allocation)
pub async fn assemble_workspace(
    grimoire: &Grimoire,
    state: &GolemState,
    observation: &Observation,
    attention_budget: &AttentionBudget,
) -> Result<CognitiveWorkspace> {
    let total = (attention_budget.total_tokens as f64
        * attention_budget.mortality_modifier) as u32;

    // ── Invariants (always included) ────────────────────────
    let invariant_budget = (total as f64
        * attention_budget.allocations[&ContextCategory::Invariants]) as u32;
    let policy_cage = read_policy_cage_block(state).await?;
    let positions = state.positions.clone();
    let active_warnings = state.active_warnings.clone();
    let affect = AffectBlock::from_state(state);

    // ── Retrieved knowledge (budget-allocated) ──────────────
    let retrieval_budget = (total as f64
        * attention_budget.allocations[&ContextCategory::RetrievedKnowledge]) as u32;
    let pad = state.cortical_state.read_pad();
    let retrieved = grimoire.retrieve(
        &observation.to_retrieval_query(),
        &pad,
        state.current_tick,
        retrieval_budget,
    ).await?;

    // ── Causal edges (budget-allocated) ─────────────────────
    let causal_budget = (total as f64
        * attention_budget.allocations
            .get(&ContextCategory::CausalGraph)
            .copied().unwrap_or(0.1)) as u32;
    let causal_edges = grimoire.get_relevant_causal_edges(
        observation, causal_budget,
    )?;

    // ── Dream hypotheses (budget-allocated) ─────────────────
    let dream_budget = (total as f64
        * attention_budget.allocations
            .get(&ContextCategory::DreamHypotheses)
            .copied().unwrap_or(0.1)) as u32;
    let hypotheses = grimoire.get_pending_staged_revisions(dream_budget)?;

    // ── Somatic landscape gut feeling ───────────────────────
    let landscape = state.daimon.somatic_landscape
        .gut_feeling(&state.current_strategy_params());

    // ── Pheromone summary ───────────────────────────────────
    let pheromone_summary = if !state.pheromone_readings.is_empty() {
        Some(state.pheromone_readings.to_natural_language())
    } else { None };

    Ok(CognitiveWorkspace {
        policy_cage,
        positions,
        active_warnings,
        strategy: state.strategy_block(),
        affect,
        playbook_heuristics: grimoire.get_top_playbook_heuristics(10)?,
        mental_models: state.config.mental_models.clone(),
        retrieved_episodes: retrieved.episodes,
        retrieved_insights: retrieved.insights,
        causal_edges,
        contrarian_entries: retrieved.contrarian,
        dream_hypotheses: hypotheses,
        somatic_landscape_reading: Some(landscape),
        observation: observation.clone(),
        mortality: state.mortality_block(),
        interventions: state.intervention_block(),
        pheromone_summary,
        conversation_tail: state.conversation_sidecar.recent_messages(5),
        total_tokens: total,
        policy_revision: state.context_policy_revision,
    })
}
```

---

## S4 -- Predictive Context Assembly

### The Problem

Assembling the Cognitive Workspace takes 5-50ms: querying LanceDB for relevant episodes, scoring candidates, querying SQLite for causal edges, reading the Somatic Landscape. This happens synchronously in the heartbeat pipeline -- when the Golem needs to deliberate (T1/T2), it blocks waiting for context assembly.

### The Solution: Background Pre-Assembly

A background tokio fiber continuously maintains a pre-built workspace that updates reactively when the Golem's state changes (PAD shift, regime change, new knowledge). When deliberation fires, the context is already built -- the heartbeat pipeline reads the pre-assembled workspace instead of building one from scratch.

```rust
/// Background fiber that maintains a pre-assembled Cognitive Workspace.
///
/// Monitors the CorticalState for state changes (PAD shift > 0.1,
/// regime change). When a significant change is detected, re-assembles
/// the workspace in the background. The heartbeat pipeline can read
/// the pre-assembled workspace from the shared Arc<RwLock<>> without
/// waiting for assembly.
///
/// This eliminates 5-50ms of assembly latency during deliberation.
/// The fiber runs every 5 seconds — frequent enough to catch state
/// changes, infrequent enough to not waste CPU.
pub async fn predictive_context_fiber(
    grimoire: Arc<Grimoire>,
    cortical_state: Arc<CorticalState>,
    current_workspace: Arc<parking_lot::RwLock<Option<CognitiveWorkspace>>>,
) {
    let mut last_pad = cortical_state.read_pad();
    let mut last_regime = cortical_state.read_regime();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let new_pad = cortical_state.read_pad();
        let new_regime = cortical_state.read_regime();
        let pad_delta = pad_distance(&last_pad, &new_pad);
        let regime_changed = new_regime != last_regime;

        if pad_delta > 0.1 || regime_changed {
            if let Ok(ws) = assemble_workspace_from_cortical_state(
                &grimoire, &cortical_state,
            ).await {
                *current_workspace.write() = Some(ws);
            }
            last_pad = new_pad;
            last_regime = new_regime;
        }
    }
}
```

---

## S5 -- Cybernetic Self-Tuning

The Context Governor doesn't have fixed token allocations -- it LEARNS them. Three feedback loops operate at different timescales to adapt the context policy based on outcomes. See `14-context-governor.md` for the full specification; here is the summary.

### Loop 1: Per-Tick Outcome Correlation

After every tick that has an outcome (a trade was executed and verified), Loop 1 correlates which Grimoire entries appeared in the workspace with whether the outcome was positive. Entries that consistently appear in contexts that produce good decisions get higher future retrieval weights.

```rust
impl CyberneticsEngine {
    /// Loop 1: per-tick outcome correlation.
    /// EMA update: entries in winning contexts get +0.1;
    /// entries in losing contexts get -0.05 (asymmetric because
    /// a single bad entry doesn't invalidate all knowledge).
    pub fn loop1_update(
        &mut self,
        workspace: &CognitiveWorkspace,
        outcome: &OutcomeRecord,
    ) {
        let success = outcome.pnl_impact.unwrap_or(0.0) > 0.0;
        let signal = if success { 0.1 } else { -0.05 };

        for entry in workspace.retrieved_episodes.iter()
            .chain(workspace.retrieved_insights.iter())
        {
            let corr = self.loop1_correlations
                .entry(entry.id.clone())
                .or_insert(0.0);
            *corr = (*corr * 0.95) + signal; // EMA decay + new signal
        }
    }
}
```

### Loop 2: Per-Curator Policy Evolution (Every 50 Ticks)

Every 50 ticks, Loop 2 aggregates Loop 1's per-entry correlations into category-level performance metrics. Categories with consistently positive correlations get more tokens; categories with negative correlations get fewer.

```rust
    /// Loop 2: evolve the ContextPolicy every 50 ticks.
    /// Adjusts token allocations across categories based on which
    /// categories produced entries with positive outcome correlations.
    pub fn loop2_evolve_policy(&mut self, policy: &mut ContextPolicy) {
        // Compute average correlation per category
        let mut category_scores: HashMap<ContextCategory, (f64, u32)> = HashMap::new();
        for (entry_id, corr) in &self.loop1_correlations {
            if let Some(cat) = self.entry_to_category.get(entry_id) {
                let entry = category_scores.entry(*cat).or_insert((0.0, 0));
                entry.0 += corr;
                entry.1 += 1;
            }
        }

        // Adjust allocations proportionally
        for (cat, (total_corr, count)) in &category_scores {
            if *count == 0 { continue; }
            let avg_corr = total_corr / *count as f64;
            if let Some(alloc) = policy.allocations.get_mut(cat) {
                // Positive correlation → increase allocation (up to 2× base)
                // Negative correlation → decrease allocation (down to 0.5× base)
                *alloc = (*alloc * (1.0 + avg_corr * 0.1)).clamp(0.05, 0.50);
            }
        }

        // Renormalize so allocations sum to 1.0
        let total: f64 = policy.allocations.values().sum();
        for alloc in policy.allocations.values_mut() {
            *alloc /= total;
        }

        self.policy_revision += 1;
    }
```

### Loop 3: Per-Regime Restructuring

When the market regime changes (e.g., `range_bound` to `volatile`), Loop 3 partially resets Loop 1's correlations. This prevents the system from applying learnings from one regime to a fundamentally different one.

```rust
    /// Loop 3: partial reset on regime change.
    /// Decay all correlations by 50% — don't throw away everything
    /// (some knowledge transfers across regimes) but reduce confidence
    /// in regime-specific patterns.
    pub fn loop3_regime_change(&mut self, new_regime: MarketRegime) {
        for corr in self.loop1_correlations.values_mut() {
            *corr *= 0.5;
        }
        self.current_regime = new_regime;
    }
}
```

---

## S6 -- The Intervention System

Owners communicate with their Golem through two primitives:

**Steer** (high priority): "Change what you're doing RIGHT NOW." A steer preempts the current tick -- it's injected into context immediately and can cancel in-flight tool calls. Semantics: reflex arc.

**FollowUp** (low priority): "Consider this next time you deliberate." A followUp is queued for the appropriate decision window and incorporated into the Cognitive Workspace at the next relevant point. Semantics: learning signal.

Both become Grimoire episodes after outcome resolution -- the Golem learns from owner feedback the same way it learns from market outcomes. Repeated owner steers can graduate to self-protective heuristics through dream consolidation: what was once external guidance becomes internal wisdom (the Baldwin Effect applied to runtime control).

```rust
/// An intervention from any source.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Intervention {
    pub id: String,
    pub source: InterventionSource,
    pub severity: Severity,
    pub intent: String,
    pub scope: InterventionScope,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InterventionSource {
    /// Human owner via web/TUI/Telegram
    Owner { user_id: String, surface: String },
    /// The Golem's own risk daemon
    RiskDaemon { trigger: String },
    /// PolicyCage near-violation detected
    PolicyDaemon { constraint: String },
}

/// Decision windows: when different interventions are processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DecisionWindow {
    /// Steer: processed before next action.
    Immediate,
    /// FollowUp: processed at the DECIDING step.
    NextDecide,
    /// Review task: processed at REFLECTING step.
    NextReflect,
    /// Memory task: processed at next Curator cycle.
    NextCurator,
    /// Deep revision: processed at next dream cycle.
    NextDream,
}
```

---

## S7 -- Semantic Cache

The semantic cache reduces redundant LLM calls by matching semantically similar prompts to cached responses. Two layers operate in sequence.

**Layer 1 -- Exact-match hash cache.** Normalize the prompt (strip timestamps, sort tool results, remove ephemeral metadata), compute SHA-256, look up in an LRU cache. Hit = zero LLM cost.

**Layer 2 -- Embedding similarity cache.** For prompts that are not exact matches but semantically equivalent ("What is ETH's price trend?" and "How is Ethereum performing?"), compute an embedding via `nomic-embed-text-v1.5` (local ONNX, ~3ms, 768 dimensions) and search an in-memory HNSW index for cached prompts above the similarity threshold.

```rust
/// Configuration for the semantic cache layer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticCacheConfig {
    /// 0.92 for market analysis, 0.95 for strategy, 0.98 for trade execution.
    pub similarity_threshold: f64,
    /// Default: 10,000.
    pub max_entries: usize,
    /// Default: 300 (calm), reduced in volatile regimes.
    pub base_ttl_seconds: u64,
    /// Multiplier for volatile regime (e.g., 0.3 = 90s TTL).
    pub volatile_regime_multiplier: f64,
    /// How cache entries are scoped.
    pub isolation_mode: CacheIsolationMode,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CacheIsolationMode {
    /// Each Golem has its own cache.
    PerAgent,
    /// Golems in the same Clade share cache.
    Shared,
    /// Tiered: shared for reads, per-agent for writes.
    Tiered,
}
```

Cache invalidation triggers: regime change (invalidates all market-analysis entries immediately), on-chain state change (pool swaps, position changes invalidate position-specific entries), TTL expiry (regime-aware), and reflector invalidation (when PLAYBOOK.md heuristics update, entries that depended on old heuristics are invalidated).

Expected savings: 15--30% reduction in LLM calls across a typical Clade. Golems in the same Clade running similar strategies benefit most -- the first Golem's analysis is cached and reused by siblings querying through the same gateway.

---

## S8 -- Daily Cost Projections

At ~720-2,880 ticks/day (Adaptive Clock theta frequency, 30-120s regime-dependent) with the expected distribution (~80% T0, ~15% T1, ~5% T2). Per-call costs: T1 avg $0.002, T2 avg $0.05 (see [02-heartbeat.md](02-heartbeat.md) for the full range). The table below uses a midpoint of ~1,440 ticks/day (60s average theta interval).

| Scenario | T1 Calls | T1 Cost | T2 Calls | T2 Cost | Raw Daily | With Context Engineering |
|---|---|---|---|---|---|---|
| Calm market (90% T0, 8% T1, 2% T2) | 461 | $0.92 | 115 | $5.76 | ~$6.68 | ~$1.00 |
| Normal market (80% T0, 15% T1, 5% T2) | 864 | $1.73 | 288 | $14.40 | ~$16.13 | ~$2.50 |
| Volatile market (60% T0, 25% T1, 15% T2) | 1,440 | $2.88 | 864 | $43.20 | ~$46.08 | ~$8.00 |

"Raw Daily" is the API cost without Bardo Inference optimizations. "With Context Engineering" applies Bardo Inference's 8-layer pipeline: semantic/hash caching (~20% of requests served from cache), prompt cache alignment (90% discount on cached prefix tokens), tool pruning (97.5% token reduction), and multi-model routing (50-90% per-request savings). See [../12-inference/01-routing.md](../12-inference/01-routing.md) for the full savings breakdown.

Target daily cost per strategy: $1.50--$3.00 total (LLM + compute + gas + data) when routed through Bardo Inference. The LLM partition receives 60% of the credit budget, so a $3.00/day budget allocates $1.80/day to inference.

Without the gating system (every tick at T2): 5,760 × $0.10 = **$576/day**. Tier gating alone provides a **~35x cost reduction**. Context engineering provides an additional **~6x reduction**, bringing the combined savings to **~230x** under normal market conditions.

**Dream budget.** During the Thriving phase, 5--10% of the LLM partition is allocated to dream cycles (see [05-dreams/](../05-dreams/)). For a $1.80/day LLM budget, this is $0.09--$0.18/day of dream compute. NREM replay uses T1 exclusively. REM imagination uses T1 for initial association and T2 for promising threads. Dream compute is the first budget line cut under economic pressure.

---

## S9 -- Provider Architecture

### Provider Resolution

Each provider is self-describing -- it implements a `resolve(intent)` method that returns a `Resolution` or `None`. The Golem's config is an ordered list of providers. The resolver walks the list in order; first match wins.

```rust
/// A provider that knows its own capabilities.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Unique identifier (e.g., "blockrun", "openrouter", "venice").
    fn id(&self) -> &str;
    /// Human-readable name.
    fn name(&self) -> &str;
    /// Resolve an intent to a concrete model + provider pair.
    /// Returns None if this provider cannot handle the request.
    fn resolve(&self, intent: &Intent) -> Option<Resolution>;
    /// Provider-specific traits (privacy, payment mode, etc.).
    fn traits(&self) -> &ProviderTraits;
}

#[derive(Debug, Clone)]
pub struct ProviderTraits {
    /// Inference logs are not stored. TEE-attested.
    pub private: bool,
    /// Revenue from social engagement funds inference.
    pub self_funding: bool,
    /// Context engineering applies to this provider's requests.
    pub context_engineering: bool,
    /// How this provider is paid.
    pub payment: PaymentMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentMode {
    /// USDC on Base via x402 protocol.
    X402,
    /// Prepaid API credits (requires operator float).
    Prepaid,
    /// User's own API key (passthrough).
    ApiKey,
    /// Venice DIEM staking (zero-cost inference).
    Diem,
    /// Agent wallet pays directly from earned revenue.
    Wallet,
}

#[derive(Debug, Clone)]
pub struct Intent {
    pub model: Option<String>,
    pub require: Vec<String>,
    pub prefer: Vec<String>,
    pub quality: Quality,
    pub max_latency_ms: u64,
    pub cost_sensitivity: f64,
    pub diem_available: bool,
    pub subsystem: String,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub model: String,
    pub provider: String,
    pub estimated_cost_usd: f64,
    pub features: Vec<String>,
    pub degraded: Vec<String>,
}
```

### Four Provider Types

From the Golem's perspective, there are four provider types. Each implements the `Provider` trait (above) and self-describes its capabilities. The resolver walks the user's ordered list — first match wins. No central registry.

| Provider | Role | Payment | Key Features |
|---|---|---|---|
| **Bardo Inference** | Context engineering proxy | x402 USDC / Prepaid | 8-layer optimization, 30+ models via BlockRun backbone, auto-failover via OpenRouter |
| **Venice** | Private cognition | DIEM staking / API key | Zero-log, TEE-attested, DeepSeek R1 visible `<think>` |
| **Bankr** | Self-funding | Wallet | Revenue-funded inference, cross-model verification |
| **Direct Key** | Native features | User's key | Predicted Outputs, Batch API, explicit caching |

Bardo Inference is opaque to the Golem — internally it routes to BlockRun (primary, 30+ models, x402 payment) and OpenRouter (fallback, 400+ models, BYOK). The Golem sees one provider; Bardo Inference picks the backend. See [../12-inference/12-providers.md](../12-inference/12-providers.md) for the internal backend routing.

Venice returns `None` for intents that don't prefer privacy. Direct Key returns `None` unless the intent requires a native-only feature like `predicted_outputs`. Bardo Inference handles everything else.

See [../12-inference/12-providers.md](../12-inference/12-providers.md) for full provider implementations including `resolve()` and request formatting methods.

### Fallback Cascade with Health Monitoring

Provider health is tracked via 30-second pings. Providers exceeding 5% error rate over 5 minutes are temporarily removed from the pool.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelHealth {
    pub status: HealthStatus,
    pub avg_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub error_rate: Option<f64>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
    Unknown,
}
```

### Degradation Visibility

When a provider satisfies an intent but not all preferences, `Resolution.degraded` names what is missing. The Golem emits this to the operator: "Dream cycle used Claude via BlockRun (visible thinking and privacy unavailable — configure Venice for better dream quality)." The user knows exactly what to add to their config. Silent fallbacks are gone — every compromise is visible.

### Mortality-Aware Model Selection

A declining Golem routes more aggressively to cheap tiers. Conservation phase favors T1 over T2 unless prediction error is extreme. Terminal Golems use T0 only, except death reflection (always Opus, charged to the death reserve). The gateway's cost profile naturally curves downward as the Golem approaches death, extending effective lifespan by reducing the largest variable cost.

### Budget Allocation Under Mortality Pressure

When vitality drops, inference budgets contract. Risk and death are never reduced.

```rust
/// Apply mortality pressure to an inference intent.
///
/// Exempt subsystems: "risk", "death", "operator" — these always
/// get full attention regardless of vitality.
pub fn apply_mortality_pressure(
    intent: &mut Intent,
    vitality: f64,
) {
    let exempt = ["risk", "death", "operator"];
    if exempt.contains(&intent.subsystem.as_str()) { return; }

    let pressure = 1.0 - vitality;
    intent.cost_sensitivity = (intent.cost_sensitivity + pressure * 0.3).min(1.0);
    if pressure > 0.7 {
        intent.quality = intent.quality.downgrade();
    }
}
```

Budget partition by subsystem under pressure:

| Subsystem | Healthy (vitality > 0.7) | Stressed (0.3-0.7) | Terminal (<0.3) |
|---|---|---|---|
| Heartbeat | 20% | 15% | 10% |
| Risk | 15% | 15% | 15% |
| Dream | 15% | 10% | 0% (skipped) |
| Daimon | 5% | 2% | 0% (deterministic OCC) |
| Context | 10% | 7% | 3% |
| Curator | 10% | 7% | 0% (skipped) |
| PLAYBOOK | 5% | 4% | 2% |
| Operator | 15% | 15% | 15% |
| Death | unlimited | unlimited | unlimited |

---

## S10 -- Payment Atomicity

Three mechanisms prevent payment failure mid-inference from losing partial results and desyncing credit accounting.

**Pre-authorization.** Before each tick, estimate the maximum tick cost. If the estimate exceeds the available LLM partition, skip the tick entirely rather than risk mid-tick failure.

**Idempotency keys.** Every payment is keyed by `(golem_id, tick_number, step_name)`. If a payment fails and retries, the Facilitator contract deduplicates.

**Local pessimistic credit ledger.** Two balances: `committed` (reserved for in-flight operations) and `available` (free to allocate). When a tick starts, estimated cost moves from `available` to `committed`. On success, `committed` decreases by actual cost. On failure, `committed` is released back to `available`. The on-chain balance is the source of truth, reconciled every 10 ticks.

```rust
/// Local pessimistic credit ledger for payment atomicity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreditLedger {
    /// On-chain USDC balance (source of truth, reconciled every 10 ticks).
    pub on_chain_balance: alloy::primitives::U256,
    /// Reserved for in-flight operations.
    pub committed: alloy::primitives::U256,
    /// Free to allocate.
    pub available: alloy::primitives::U256,
    /// Tick of last reconciliation.
    pub last_reconciled: u64,
    /// Pending operations keyed by idempotency key.
    pub pending_operations: HashMap<String, alloy::primitives::U256>,
}
```

Reconciliation detects external top-ups (the user "feeding" the Golem) as well as earned revenue.

---

## S11 -- Per-Strategy Cost Attribution

Every inference call is tagged with the strategy that initiated it. This enables cost-per-strategy tracking for the credit partition system (see [02-mortality/](../02-mortality/)).

```rust
/// Metadata attached to every inference request for cost attribution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InferenceRequestMetadata {
    pub golem_id: String,
    pub strategy_id: String,
    pub tick_number: u64,
    pub tier: CognitiveTier,
    pub step: String,
    pub partition: CostPartition,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum CostPartition {
    Llm,
    Gas,
    Data,
}
```

The metadata is attached to every request via the `X-402-Metadata` header. The gateway logs it alongside payment events.

---

## S12 -- Revenue Model

> **Moved to [../revenue-model.md](../revenue-model.md) Section 1 (Protocol Fee Architecture).** See that document for the full revenue model including fee tables, unit economics, and projections across all subsystems.

---

## S13 -- Configuration

The config is an ordered list of providers and a payment method. The resolver walks providers in order — first match wins. No CapabilityMap, no central registry.

```rust
/// Golem inference configuration.
///
/// The `providers` list determines routing priority. The resolver walks
/// it in order: first provider whose resolve() returns Some(resolution)
/// handles the request. This means putting Venice first routes privacy-
/// preferring intents there; putting BlockRun first routes cost-optimized
/// intents there.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GolemInferenceConfig {
    /// Ordered list of inference providers. First is tried first.
    pub providers: Vec<ProviderConfig>,
    /// Who pays for autonomous inference.
    pub payment: PaymentConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProviderConfig {
    Bardo { api_key: String },
    Venice { api_key: String, diem: bool },
    Bankr { api_key: String, wallet_id: String },
    Anthropic { api_key: String },
    OpenAi { api_key: String },
    Google { api_key: String },
    DeepSeek { api_key: String },
    Local { base_url: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PaymentConfig {
    GolemWallet { wallet_key: String, daily_budget_usd: f64 },
    Prepaid { api_key: String },
    Bankr { wallet_id: String },
    Diem,
    Composite { primary: String, fallback: String },
}
```

Example configurations:

```yaml
# Solo builder: zero cost via DIEM staking
providers:
  - type: venice
    api_key: "vn_..."
    diem: true
  - type: local
    base_url: "http://localhost:11434/v1"
payment:
  type: diem

# Serious operator: BlockRun primary + Venice for privacy + OpenAI for Predicted Outputs
providers:
  - type: bardo
    api_key: "bardo_sk_..."
  - type: venice
    api_key: "vn_..."
    diem: true
  - type: openai
    api_key: "sk-..."
payment:
  type: prepaid
  api_key: "bardo_sk_..."
```

---

## Four Modes of Intelligence

The Golem generates intelligence through four modes. All four produce the same `Prediction` struct. All four resolve against external reality. All four feed the same residual corrector. The modes differ in cost, speed, and the kind of intelligence they produce.

| Mode | When | Cost | Speed | What It Produces | Volume |
|------|------|------|-------|-----------------|--------|
| **Analytical** | Waking theta cycle | $0.00-$0.03 | Per-tick | Calibrated expectations about tracked items | ~14,000/day |
| **Corrective** | Every resolution | $0.00 | Per-resolution | Bias-adjusted prediction parameters | ~15,000/day |
| **Creative** | Dreams + hypnagogia | $0.01-$0.05 | Per-dream-cycle | Novel hypotheses as testable predictions | ~20-50/day |
| **Collective** | Continuous (Styx) | ~$0.001 | Per-sync | Community-calibrated adjustments | ~1,000/day |

### Mode 1: Analytical (Waking Cognition)

The bulk of the system. During each theta tick, PredictionDomain implementations generate predictions for ACTIVE items. The ResidualCorrector adjusts them. The Ledger stores them. Gamma ticks resolve them against reality.

~80% of theta ticks are suppressed at the gate (T0, zero cost). The remaining ~20% escalate through the full cognitive pipeline: Grimoire retrieval, LLM deliberation, action prediction, execution, verification.

**Inaction predictions**: Every suppressed tick generates an explicit prediction: "I predict holding is optimal." This resolves by comparing portfolio value at prediction time vs. resolution time. If inaction accuracy exceeds action accuracy, the action gate blocks trades. Patience emerges from the math, not from a tuned parameter.

### Mode 2: Corrective (Automatic Calibration)

Zero-cost arithmetic running at gamma frequency. Two operations:

1. **Bias correction**: Shift prediction centers to correct systematic errors.
2. **Interval calibration**: Adjust prediction widths to hit the target coverage rate.

The corrector converges fast on systematic biases (the easy wins), then plateaus. The creative mode's slower but deeper insights handle the structural patterns the corrector can't reach.

The parallel to Karpathy's autoresearch loop [KARPATHY-2026] is direct:

| Property | Karpathy's loop | Residual corrector |
|----------|----------------|-------------------|
| Metric | val_bpb | Prediction residual |
| Loop time | 5 minutes | 5-15 seconds |
| Volume | ~276/night | ~15,000/day |
| Operator | Code modification | Arithmetic |
| Cost | GPU time | ~Zero |

**Known limitation**: The corrector assumes exchangeability within (category, regime) windows. Regime transitions break this, causing a cold-start penalty as the corrector rebuilds statistics for the new regime. This typically costs 1-2 hours of degraded accuracy after a regime change.

### Mode 3: Creative (Offline Hypothesis Generation)

During dream cycles, the Golem generates novel hypotheses by replaying prediction errors, imagining counterfactual scenarios, and forming associations between semantically distant concepts. The critical change from previous specifications: creative outputs become testable predictions, not vague journal entries.

#### The lifecycle

```
HYPNAGOGIC ONSET (seconds to minutes)
  Top N prediction residuals presented as scrambled fragments.
  Temperature elevated, executive constraints loosened.
  Dalí interrupts capture half-formed associations.
  → Each fragment registered as a Creative prediction (confidence 0.10-0.20)

NREM REPLAY (minutes)
  50 predictions with largest residuals replayed.
  LLM scans for systematic patterns.
  → Pattern observations registered as Creative predictions (confidence 0.25-0.40)

REM IMAGINATION (minutes)
  Hypnagogic fragments developed into full counterfactual scenarios.
  "IF condition X, THEN outcome Y within Z hours."
  → Counterfactuals registered as Creative predictions (confidence 0.20-0.35)

INTEGRATION (minutes)
  Surviving hypotheses consolidated into:
  - PLAYBOOK.md heuristic proposals
  - Environmental model candidates (cross-item patterns)
  - ResidualCorrector bias adjustments (applied immediately)

HYPNOPOMPIC RETURN
  Top 3 creative predictions surfaced into waking context.
```

The analytical mode processes items independently — it can't discover that "gas price drops predict fee spikes across all ETH pools 2 hours later" because it never forms associations between items. The creative mode, working with loosened constraints on replayed residual patterns, can form these cross-item connections. When a creative prediction is confirmed, the environmental model applies to all relevant items simultaneously. This is the compounding flywheel's acceleration point: learning becomes multiplicative (improving N items at once) rather than additive (improving one item at a time).

**Known limitation -- multiple comparisons**: If the dream engine generates 30 creative predictions and 34% hit, some hits are expected by chance. Creative predictions must be confirmed by at least 3 independent resolution events across different items before promotion to environmental model status. This is a Bonferroni-like correction that trades sensitivity for specificity.

**Creative accuracy of 34% sounds bad**: It's the expected rate for genuinely novel hypotheses. A creative prediction that says "staking withdrawal queue length predicts Aerodrome fee spikes with a 4-hour lag" is a bold claim. Most bold claims are wrong. The 34% that are right are disproportionately valuable because they improve many predictions at once.

**Hypnagogia is aspirational at full depth**: The 4-layer hypnagogia stack (prompt-level, temperature, memory replay, steering vectors) maps to different capability tiers. Layer 1 (prompt-level) works with any LLM API. Layer 2 (temperature control) works with configurable temperature. Layers 3-4 (steering vectors, representation engineering) require model weight access that most providers don't offer. v1 implements Layers 1-2. The architecture accommodates all four so Layers 3-4 can be added without rearchitecting.

### Mode 4: Collective (Distributed Calibration)

Three layers via Styx:

| Layer | Privacy | What's Shared | Benefit |
|-------|---------|--------------|---------|
| **Vault (L0)** | Private | Nothing | Baseline solo performance |
| **Clade (L1)** | Fleet (same owner) | Residual stats, attention signals, environmental models | 7x convergence speedup at 50 members |
| **Commons (L2)** | Public (anonymized) | Anonymized aggregates, pheromone deposits | Community calibration |

Siblings merge incoming statistics with weighted averaging: the sibling's weight is proportional to its sample size relative to the local sample size. High-confidence siblings contribute more.

**Known limitation -- Clade poisoning**: A compromised Golem could share systematically biased residuals, causing siblings to miscalibrate. Mitigation: incoming Clade updates are incorporated at reduced weight (0.7x) and must converge with local observations over time. Persistent divergence between local and Clade statistics triggers an alert visible in the TUI.

**Known limitation -- Commons information leakage**: Anonymized residual statistics may reveal strategy information (which categories, which items, which regimes). Mitigation: Commons pheromones are aggregated to the hour granularity with k-anonymity guarantees (min 5 contributors per aggregate). Strategy-sensitive categories can be excluded from Commons sharing via `golem.toml`.

**Known limitation -- front-running from attention signals**: If a Clade sibling broadcasts "I found something interesting at pool X," an adversary monitoring Styx could front-run. Mitigation: attention signals within Clades share item hashes, not addresses. The sibling must independently discover and verify the item.

The collective mode is not required. A solo Golem with no Styx connection runs Modes 1-3 at full capability. A user who distrusts the collective can disable Styx sharing entirely in `golem.toml`.

### How the Modes Feed Each Other

```
ANALYTICAL → produces residuals → feeds CORRECTIVE
CORRECTIVE → adjusts predictions → improves ANALYTICAL
ANALYTICAL → largest residuals → seed CREATIVE (dream replay)
CREATIVE → confirmed models → improve ANALYTICAL across many items
ANALYTICAL → residual stats → shared via COLLECTIVE
COLLECTIVE → community calibration → improves CORRECTIVE
```

---

## Three Inference Deployment Modes

Where LLM calls happen is the single biggest decision an owner makes about cost, privacy, and operational complexity. Bardo supports three deployment modes. Each gives the Golem access to the same 8-layer context engineering pipeline, the same provider routing, the same caching stack. The difference is where the code runs.

### Mode A: Embedded in the TUI Process

The lightest option. No separate inference process. No local model. The TUI calls remote API providers directly, with the gateway's context engineering pipeline running in-process. When the TUI starts with `inference.mode = "embedded"`, it spawns an in-process Axum server on a random localhost port that runs the full gateway pipeline (prompt cache alignment, tool pruning, history compression, semantic caching, provider routing) but never binds to an external interface. Only the local Golem and Meta Hermes can reach it.

### Mode B: Local Gateway

A standalone `bardo-inference` process runs on the same machine as the Golem. Multiple Golems can share one local gateway. An optional Ollama integration provides local model inference for T0/T1 ticks, with cloud fallback for T2.

### Mode C: Remote (Bardo Compute)

Inference routes to `gateway.bardo.run`, the managed Bardo Inference service. Payment is x402 or prepaid credits. The full 8-layer context engineering pipeline runs server-side. The Golem sends a single HTTP request per inference call; the gateway handles provider routing, caching, and cost optimization.

All three modes are functionally equivalent. The Golem's cognitive pipeline does not know or care which mode is active. The inference abstraction layer normalizes the interface.

---

## References

- [BADDELEY-2000] Baddeley, A. "The Episodic Buffer: A New Component of Working Memory?" *Trends in Cognitive Sciences*, 4(11), 2000. — *Adds the episodic buffer to working memory; the theoretical basis for assembling a Cognitive Workspace fresh each tick.*
- [COGNITIVE-WORKSPACE-2025] "Cognitive Workspace: Active Memory Management for LLMs." arXiv:2508.13171, 2025. — *Proposes a compressed structured log replacing raw conversation history, achieving 6x initial context reduction; directly implemented in the CognitiveWorkspace struct.*
- [CHEN-2023] Chen, L. et al. "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." arXiv:2305.05176, 2023. — *Demonstrates matching GPT-4 performance with up to 98% cost reduction through intelligent cascading; validates the T0/T1/T2 gating approach.*
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." *Journal of Monetary Economics*, 50(3), 2003. — *Formalizes that information processing is costly and agents optimally allocate attention; grounds the inference budget as an attention allocation mechanism.*
- [KARPATHY-2026] Karpathy, A. "autoresearch." GitHub, March 2026. — *Reference implementation for agentic research patterns; comparison point for the Golem's autonomous knowledge acquisition pipeline.*
- [VOVK-2005] Vovk, V. et al. *Algorithmic Learning in a Random World*. Springer, 2005. — *Conformal prediction framework providing distribution-free calibration guarantees; informs the prediction engine's accuracy calibration.*
- [SUROWIECKI-2004] Surowiecki, J. *The Wisdom of Crowds*. Doubleday, 2004. — *Argues that diverse independent judgments aggregate to superior predictions; supports multi-model verification for high-stakes decisions.*
- Ong, I. et al. (2025). "RouteLLM: Learning to Route LLMs with Preference Data." *ICLR 2025*. — *Learns to route between models based on query difficulty; informs the adaptive tier selection mechanism.*
- Weston, J. & Sukhbaatar, S. (2024). "Distilling System 2 into System 1." arXiv:2407.06023. — *Shows slow deliberation can be distilled into fast heuristics; the theoretical basis for Golem heuristic extraction from LLM reasoning.*
- Google Research (2024). "Agents Thinking Fast and Slow: A Talker-Reasoner Architecture." — *Formalizes the dual-process separation in agent design; a parallel framework to the T0/T1/T2 gating system.*
- Zhang, H. et al. (2025). "DPT-Agent: Dual Process Theory for Language Agents." arXiv:2502.11882. — *Applies Kahneman's dual-process theory to agent architecture with explicit fast/slow subsystems; a direct validation of the LLM-last approach.*

---

*The context is the golem's attention. What it attends to determines what it sees. What it sees determines what it does.*
