# Context Governor [SPEC]

> **Version**: 3.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-context` (workspace.rs, policy.rs, cybernetics.rs, predictive.rs, interventions.rs)
>
> **Prerequisites**: Read `00-overview.md`, `02-heartbeat.md` (decision cycle, DecisionCycleRecord), `01-cognition.md` (Cognitive Workspace, inference routing).

> **Reader orientation:** This document specifies the Context Governor -- the system that decides what enters the Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) LLM context window on each tick. It belongs to the `01-golem` cognition layer, in the `golem-context` crate. The key concept: context assembly is a learnable control problem, not a static token budget. The Governor sits between Grimoire (the agent's persistent knowledge base) retrieval and inference, selecting and allocating candidates into a CognitiveWorkspace that is assembled fresh each tick. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## S1 -- Thesis

Context failures, not model failures, cause most agent breakdowns. Anthropic's context engineering framework (2025) names this directly: the primary bottleneck for production agents is not reasoning capability but what enters the context window. For a Golem running days or weeks in volatile DeFi markets, managing 200K tokens of capacity with 15K consumed by tool definitions, context assembly is the single highest-leverage system in the entire architecture.

Current approaches treat context assembly as a static token budget: N tokens for episodes, M for insights, K for playbook heuristics. This is the equivalent of giving every student the same textbook regardless of the exam. The Context Governor makes context assembly a **learnable control problem** -- one that starts with static defaults and evolves through cybernetic feedback into a system that knows, per-regime and per-task-type, which context categories improve decisions and which waste tokens.

**Position in architecture.** The Governor sits between Grimoire retrieval and inference, operating through the extension chain's `before_llm_call` hook (Extension #17) and `after_turn` hook:

```
Heartbeat probes → Grimoire retrieval → Context Governor → CognitiveWorkspace → Inference call
```

The Grimoire produces candidates -- episodes, insights, heuristics, causal edges that are semantically relevant to the current situation. The Governor decides which candidates make it into the actual context window, how much space each category gets, and why. It logs every inclusion and exclusion decision for downstream learning.

The Governor is not a retriever. It is a selector and allocator that operates on retrieval results.

---

## S2 -- CognitiveWorkspace and ContextPolicy

### S2.1 CognitiveWorkspace

The typed context unit assembled per inference call. Every LLM invocation receives a `CognitiveWorkspace` -- not raw retrieved data, not unstructured prompts, but a structured, budgeted, and annotated package. This is Bardo's implementation of the Context State Object pattern (Samsung Research, arXiv:2511.03728, 2025), where a compressed structured log replaces raw conversation history, achieving 6x reduction in initial prompt context and 10-25x reduction in context growth rate.

See `01-cognition.md` S3 for the full `CognitiveWorkspace` struct. The key fields for the Governor:

```rust
/// Summary of how the Governor assembled the workspace.
/// Attached to every CognitiveWorkspace for audit and learning.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssemblyReason {
    pub category: String,
    pub tokens_allocated: u32,
    pub tokens_used: u32,
    pub items_included: u32,
    pub items_excluded: u32,
    pub reason: String,
}
```

The `assembly_reasons` vector is the Governor's audit trail. Loop 1 (S4) reads it to learn which context categories contributed to good decisions.

### S2.2 CognitiveWorkspaceDelta

Rather than rebuilding the full workspace each turn, the Governor tracks deltas between consecutive assemblies. This reduces re-tokenization cost and makes it easier to detect which categories are churning versus stable.

```rust
/// Delta between two consecutive workspace assemblies.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CognitiveWorkspaceDelta {
    /// Which categories changed since last assembly.
    pub changed_categories: Vec<String>,
    /// New entries added, keyed by category.
    pub added: HashMap<String, Vec<ContextEntry>>,
    /// Entries removed, keyed by category (entry IDs).
    pub removed: HashMap<String, Vec<String>>,
    /// Token budget change.
    pub token_delta: i32,
}

/// Compute the delta between two workspace assemblies.
pub fn compute_workspace_delta(
    previous: &CognitiveWorkspace,
    current: &CognitiveWorkspace,
) -> CognitiveWorkspaceDelta {
    let mut delta = CognitiveWorkspaceDelta {
        changed_categories: Vec::new(),
        added: HashMap::new(),
        removed: HashMap::new(),
        token_delta: current.total_tokens as i32 - previous.total_tokens as i32,
    };

    for category in &ALL_CATEGORIES {
        let prev_ids: HashSet<&str> = previous
            .entries_for(category)
            .iter()
            .map(|e| e.id.as_str())
            .collect();
        let curr_ids: HashSet<&str> = current
            .entries_for(category)
            .iter()
            .map(|e| e.id.as_str())
            .collect();

        let new: Vec<_> = curr_ids.difference(&prev_ids).collect();
        let gone: Vec<_> = prev_ids.difference(&curr_ids).collect();

        if !new.is_empty() || !gone.is_empty() {
            delta.changed_categories.push(category.to_string());
            if !new.is_empty() {
                delta.added.insert(
                    category.to_string(),
                    new.iter()
                        .filter_map(|id| current.entry_by_id(category, id))
                        .cloned()
                        .collect(),
                );
            }
            if !gone.is_empty() {
                delta.removed.insert(
                    category.to_string(),
                    gone.iter().map(|id| id.to_string()).collect(),
                );
            }
        }
    }

    delta
}
```

> **Cross-reference**: The `CognitiveWorkspaceDelta` tracks high-level category changes between assemblies. For the lower-level I-frame/P-frame context delta compression (segment-level content hashes, forced recompact triggers, regime-change invalidation), see [03c-state-management.md](./03c-state-management.md) Section 2 (Context Delta Compression). The `DeltaCompressor` from source `09-context-delta` operates on named `ContextSegment` entries with Blake3 content hashes and forces full I-frame recompacts when delta tokens exceed 30% of the budget or after 10 consecutive delta runs.

### S2.3 ContextPolicy

The `ContextPolicy` defines how the Governor assembles context. It starts as static defaults and evolves through cybernetic feedback (S4).

```rust
/// The learned context assembly policy.
///
/// This struct IS the Governor's intelligence. It starts with static
/// defaults and evolves through three feedback loops (S4) into a
/// regime-aware, phase-aware, task-aware allocation strategy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextPolicy {
    /// Monotonically increasing revision number.
    pub revision: u32,

    /// Per-category token allocations (fraction of total budget, summing to 1.0).
    pub allocations: HashMap<ContextCategory, f64>,

    /// Retrieval configuration.
    pub retrieval: RetrievalConfig,

    /// Compression thresholds — when to summarize instead of including raw.
    pub compression: CompressionConfig,

    /// Override allocations per regime.
    pub regime_overrides: HashMap<MarketRegime, HashMap<ContextCategory, f64>>,

    /// Override allocations per behavioral phase.
    pub phase_overrides: HashMap<BehavioralPhase, HashMap<ContextCategory, f64>>,

    /// Override allocations per task type.
    pub task_overrides: HashMap<TaskType, HashMap<ContextCategory, f64>>,

    /// Model preference for the current policy.
    pub model_preference: ModelPreference,

    /// Performance tracking.
    pub metrics: PolicyMetrics,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalConfig {
    /// Minimum cosine similarity for episode retrieval (default: 0.3).
    pub similarity_threshold: f64,
    /// Maximum episodes to retrieve (default: 10).
    pub max_episode_count: u32,
    /// Minimum confidence for insight inclusion (default: 0.4).
    pub min_insight_confidence: f64,
    /// Maximum causal graph traversal depth (default: 3).
    pub max_causal_depth: u32,
    /// Full episode slots for observation masking (default: 3).
    pub full_episode_slots: u32,
    /// Summary episode slots for observation masking (default: 7).
    pub summary_episode_slots: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompressionConfig {
    /// Summarize episodes longer than this token count (default: 200).
    pub episode_summary_threshold: u32,
    /// Summarize insights longer than this (default: 150).
    pub insight_summary_threshold: u32,
    /// ACON-evolved compression guidelines.
    pub guidelines: Vec<CompressionGuideline>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModelPreference {
    CheapFirst,
    Balanced,
    Paranoid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyMetrics {
    pub decision_quality_avg: f64,
    /// Decisions per 1000 tokens.
    pub token_efficiency: f64,
    pub last_validated: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ContextCategory {
    Invariants,
    Strategy,
    Playbook,
    Episodes,
    Insights,
    CausalEdges,
    Contrarian,
    DreamHypotheses,
    ToolState,
    DefiSnapshot,
    Mortality,
    Affect,
    OwnerControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    Swap,
    LpRebalance,
    RiskCheck,
    VaultDeposit,
    VaultWithdraw,
    StrategyReview,
    Dream,
    Sim,
}
```

### S2.4 Default Overrides

The initial policy uses fixed allocations derived from operational experience with LLM context engineering. These defaults are conservative -- they over-allocate to invariants and strategy (safety-critical) and under-allocate to episodes and contrarian (learning-oriented). As the Governor learns, allocations shift toward the categories that contribute to better decisions.

```rust
/// Default per-category allocations (fraction of total budget).
pub fn default_allocations() -> HashMap<ContextCategory, f64> {
    use ContextCategory::*;
    [
        (Invariants, 0.15),
        (Strategy, 0.10),
        (Playbook, 0.15),
        (Episodes, 0.12),
        (Insights, 0.10),
        (CausalEdges, 0.08),
        (Contrarian, 0.04),
        (DreamHypotheses, 0.03),
        (ToolState, 0.03),
        (DefiSnapshot, 0.08),
        (Mortality, 0.04),
        (Affect, 0.03),
        (OwnerControl, 0.04),
    ].into_iter().collect()
    // Sum: ~0.99 (rounding). Normalized to 1.0 at assembly time.
}

/// Regime-specific overrides.
pub fn default_regime_overrides() -> HashMap<MarketRegime, HashMap<ContextCategory, f64>> {
    use ContextCategory::*;
    let mut overrides = HashMap::new();

    // Volatile: more DeFi data, less exploration
    overrides.insert(MarketRegime::Volatile, [
        (DefiSnapshot, 0.10),   // +25%
        (Episodes, 0.08),       // -33%
        (Contrarian, 0.02),     // -50%
    ].into_iter().collect());

    // Bear high vol: survival focus
    overrides.insert(MarketRegime::BearHighVol, [
        (Mortality, 0.06),      // +50%
        (Contrarian, 0.01),     // -75%
        (DreamHypotheses, 0.01), // -67%
    ].into_iter().collect());

    overrides
}

/// Phase-specific overrides.
pub fn default_phase_overrides() -> HashMap<BehavioralPhase, HashMap<ContextCategory, f64>> {
    use ContextCategory::*;
    let mut overrides = HashMap::new();

    // Conservation: survival focus
    overrides.insert(BehavioralPhase::Conservation, [
        (Mortality, 0.06),
        (Episodes, 0.06),       // -50%
        (Contrarian, 0.01),
        (DreamHypotheses, 0.0),
    ].into_iter().collect());

    // Declining: heavy constraint awareness
    overrides.insert(BehavioralPhase::Declining, [
        (Mortality, 0.08),
        (Invariants, 0.20),     // +33%
        (Episodes, 0.04),
        (Insights, 0.04),
        (Contrarian, 0.0),
        (DreamHypotheses, 0.0),
        (Affect, 0.0),
    ].into_iter().collect());

    // Terminal: only constraints matter
    overrides.insert(BehavioralPhase::Terminal, [
        (Invariants, 0.30),
        (Mortality, 0.10),
    ].into_iter().collect());

    overrides
}
```

---

## S3 -- Seven Context Engineering Techniques

Each technique is drawn from recent academic work and mapped to the Golem's architecture. Together they define how the Context Governor decides what enters the context window.

### S3.1 ACE: Agentic Context Engineering

**Citation**: Zhang, A. et al. "ACE: Agentic Context Engineering." arXiv:2510.04618, 2025.

ACE treats context as an evolving playbook refined through a Generator-Reflector-Curator cycle. The Generator produces reasoning trajectories; the Reflector distills insights from successes and failures; the Curator applies delta edits to structured context rather than regenerating it from scratch. This addresses two failure modes: "brevity bias" (summaries that lose critical detail) and "context collapse" (iterative summarization compounding information loss). ACE achieved +10.6% on the AppWorld agent benchmark.

The ACE cycle maps directly to the Golem's existing Reflexion/ExpeL loop, extended to context:

```rust
/// ACE-style context strategy map.
/// Task type determines retrieval strategy; phase and regime modify it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextStrategyMap {
    pub task_strategies: HashMap<TaskType, RetrievalStrategy>,
    pub phase_modifiers: HashMap<BehavioralPhase, PhaseModifier>,
    pub regime_modifiers: HashMap<MarketRegime, RegimeModifier>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalStrategy {
    /// Which Grimoire collections to query.
    pub collections: Vec<GrimoireCollection>,
    /// Minimum relevance score for inclusion.
    pub relevance_threshold: f64,
    /// Maximum entries per collection.
    pub max_entries: HashMap<GrimoireCollection, u32>,
    /// Whether to include causal graph traversal.
    pub traverse_causal_graph: bool,
    /// Custom ranking function weights for this task type.
    pub ranking_weights: HashMap<String, f64>,
}

/// The ACE pipeline connection.
///
/// 1. Retrieve: Pull candidate context items from Grimoire
/// 2. Score: Apply Beta-distribution-learned weights per category per regime
/// 3. Filter: Remove items below the dynamic threshold (Beta.value() < 0.3)
/// 4. Reorder: Apply lost-in-middle mitigation (interleave sort)
/// 5. Inject: Place into the CognitiveWorkspace for the current tick
```

### S3.2 Retrieval-Augmented Prompting with Progressive Disclosure

Neither pure RAG nor pure full-context wins. Agentic RAG -- where the LLM has tool-calling agency to dynamically retrieve -- outperforms both ("Is Agentic RAG Worth It?", arXiv:2601.07711, 2026). The optimal approach routes per-turn between full context, RAG, and hybrid based on context pressure and task phase.

```rust
/// Retrieve, rank, and progressively disclose context entries.
/// Recent entries are injected in full; older entries as one-line
/// summaries (observation masking per Lindenbauer et al., NeurIPS 2025).
pub fn retrieve_and_rank(
    candidates: &[GrimoireEntry],
    policy: &ContextPolicy,
    task_type: TaskType,
    token_budget: u32,
) -> RankedContext {
    // Rank by relevance * recency * confidence
    let mut scored: Vec<_> = candidates.iter().map(|entry| {
        let score = entry.relevance_score * 0.5
            + recency_decay(entry.timestamp) * 0.3
            + entry.confidence * 0.2;
        (entry, score)
    }).collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Progressive disclosure: full entries until budget pressure, then summaries
    let mut result = RankedContext::default();
    let mut tokens_used = 0u32;

    for (entry, _score) in &scored {
        let entry_tokens = estimate_tokens(&entry.content);

        if tokens_used + entry_tokens <= (token_budget as f64 * 0.7) as u32 {
            // Under 70% budget: inject full content
            result.full.push((*entry).clone());
            tokens_used += entry_tokens;
        } else if tokens_used < (token_budget as f64 * 0.95) as u32 {
            // 70-95% budget: inject one-line summary
            result.summarized.push((*entry).clone());
            tokens_used += estimate_tokens(&entry.one_liner_summary);
        } else {
            // Over 95%: exclude, but the LLM can retrieve via query_grimoire
            result.excluded.push((*entry).clone());
        }
    }

    result
}

/// Route between full injection, RAG-only, and hybrid per turn.
pub fn route_context_injection(
    turn_phase: TurnPhase,
    current_context_tokens: u32,
    max_context_tokens: u32,
    task_complexity: TaskComplexity,
) -> ContextRoute {
    let context_pressure = current_context_tokens as f64 / max_context_tokens as f64;

    // Low pressure + simple task = inject full context
    if context_pressure < 0.5 && task_complexity == TaskComplexity::Low {
        return ContextRoute::Full;
    }

    // High pressure = RAG only (let the LLM use query_grimoire)
    if context_pressure > 0.8 {
        return ContextRoute::Rag;
    }

    // DECIDE and REFLECT phases benefit from full episodic context
    if matches!(turn_phase, TurnPhase::Decide | TurnPhase::Reflect) {
        return ContextRoute::Hybrid;
    }

    // Default: hybrid (inject essentials, let LLM retrieve the rest)
    ContextRoute::Hybrid
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRoute {
    Full,
    Rag,
    Hybrid,
}
```

**Why masking over summarization**: Summarization introduces a subtle but dangerous failure mode -- it can obscure that a previous action failed. If the summary reads "analyzed ETH/USDC pool conditions," the LLM doesn't know the analysis concluded the pool was dangerous. The original observation "ETH/USDC pool shows 4.2% IL risk, AVOID" is unambiguous. Masking preserves the original observations for recent events and drops older ones entirely, avoiding the lossy middle ground (Lindenbauer et al., arXiv:2508.21433, NeurIPS 2025 DL4C Workshop).

```rust
/// Apply observation masking to episodes.
/// Most recent `full_slots` episodes are injected verbatim.
/// Next `summary_slots` episodes are injected as one-line summaries.
/// Remaining episodes are excluded (negative prehension) but available
/// via query_grimoire.
pub fn mask_episodes(
    episodes: &[Episode],
    full_slots: usize,
    summary_slots: usize,
) -> String {
    let mut sorted = episodes.to_vec();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut sections = Vec::new();

    // Full episodes (most recent K)
    let full = &sorted[..full_slots.min(sorted.len())];
    if !full.is_empty() {
        sections.push("<recent_episodes>".to_string());
        for ep in full {
            sections.push(format!(
                "  <episode tick=\"{}\" type=\"{}\" outcome=\"{}\">\n    {}\n  </episode>",
                ep.tick, ep.episode_type, ep.outcome, ep.content
            ));
        }
        sections.push("</recent_episodes>".to_string());
    }

    // Summary episodes (next N, one-line each)
    let summaries = &sorted[full_slots.min(sorted.len())
        ..(full_slots + summary_slots).min(sorted.len())];
    if !summaries.is_empty() {
        sections.push("<older_episodes>".to_string());
        for ep in summaries {
            sections.push(format!(
                "  <episode tick=\"{}\" type=\"{}\" outcome=\"{}\" summary=\"{}\" />",
                ep.tick, ep.episode_type, ep.outcome, ep.one_liner_summary
            ));
        }
        sections.push("</older_episodes>".to_string());
    }

    // Remaining episodes are NOT included (negative prehension).
    // The LLM can use query_grimoire to retrieve them if needed.
    sections.join("\n")
}
```

### S3.3 Phase-Aware Context Routing

Budget allocations shift by behavioral phase. A Golem in `Thriving` phase can afford exploratory context (contrarian evidence, dream hypotheses). A Golem in `Declining` phase cannot -- it needs constraint awareness and survival data. A `Terminal` Golem gets only invariants and mortality.

```rust
/// Apply phase-specific overrides to the base policy.
pub fn apply_phase_routing(
    phase: BehavioralPhase,
    base_policy: &ContextPolicy,
) -> ContextPolicy {
    let mut routed = base_policy.clone();
    if let Some(overrides) = routed.phase_overrides.get(&phase) {
        for (cat, value) in overrides {
            routed.allocations.insert(*cat, *value);
        }
    }
    normalize_allocations(&mut routed.allocations);
    routed
}

/// Normalize allocations so they sum to 1.0.
fn normalize_allocations(allocations: &mut HashMap<ContextCategory, f64>) {
    let total: f64 = allocations.values().sum();
    if total > 0.0 {
        for v in allocations.values_mut() {
            *v /= total;
        }
    }
}
```

Phase routing interacts with task-type routing. A `Swap` task in `Conservation` phase gets the swap task overrides with conservation modifiers applied on top. The composition order is: base allocations -> task overrides -> phase overrides -> regime overrides. Later overrides win on conflicts.

### S3.4 Outcome Verification Context

**Citation**: Cohen-Wang, B., Shah, H., Georgiev, B., & Madry, A. "ContextCite: Attributing Model Generation to Context." MIT CSAIL, arXiv:2409.00729, 2024.

Attribution runs periodically (not every turn) via a background process. High-attribution items are flagged for preservation during compaction; low-attribution items are candidates for pruning. This feeds the cybernetic loops with hard data about which context categories actually drove decisions.

```rust
/// Build verification context by comparing predicted outcomes
/// against actual results.
pub fn build_outcome_verification(
    prediction: &GolemPrediction,
    actual: &ObservedOutcome,
    contribution_records: &[ContextContributionRecord],
) -> OutcomeVerificationBlock {
    let prediction_error = (prediction.value - actual.value).abs();
    let was_correct_direction = (prediction.value - prediction.baseline).signum()
        == (actual.value - prediction.baseline).signum();

    // Find which context categories were present when the prediction was made
    let prediction_tick = contribution_records.iter()
        .find(|r| r.tick == prediction.tick);

    // Categories that were injected but NOT referenced suggest the LLM
    // ignored information that would have improved the prediction
    let ignored_categories = prediction_tick
        .map(|tick| {
            tick.injected.iter()
                .filter(|i| !tick.referenced.iter().any(|r| r.category == i.category))
                .map(|i| i.category.clone())
                .collect()
        })
        .unwrap_or_default();

    OutcomeVerificationBlock {
        prediction_error,
        was_correct_direction,
        ignored_categories,
        calibration_delta: prediction_error / (prediction.confidence + 0.01),
        recommendation: if prediction_error > 0.2 {
            format!("Increase allocation for: {:?}", ignored_categories)
        } else {
            "Current allocations adequate".to_string()
        },
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutcomeVerificationBlock {
    pub prediction_error: f64,
    pub was_correct_direction: bool,
    pub ignored_categories: Vec<String>,
    pub calibration_delta: f64,
    pub recommendation: String,
}
```

### S3.5 State Compaction

When context pressure rises above 80%, the Governor compacts state into XML-formatted blocks. Hard context (PolicyCage, positions, phase) is never compacted -- only soft context gets compressed.

```rust
/// Compact state into XML blocks for injection when context pressure
/// is high. Hard context is always full fidelity; soft context is
/// compressed proportionally.
pub fn compact_state_for_context(
    workspace: &CognitiveWorkspace,
    target_tokens: u32,
) -> String {
    let mut blocks = Vec::new();

    // Hard context: always full fidelity
    blocks.push(format!(
        "<invariants>\n  <policy_cage hash=\"{}\">
    {}
    <max_positions>{}</max_positions>
    <max_drawdown_bps>{}</max_drawdown_bps>
  </policy_cage>
  <phase>{:?}</phase>
  <active_warnings count=\"{}\">
    {}
  </active_warnings>
</invariants>",
        workspace.policy_cage.hash,
        workspace.policy_cage.approved_assets.iter()
            .map(|a| format!("<asset>{}</asset>", a))
            .collect::<Vec<_>>().join("\n    "),
        workspace.policy_cage.max_positions,
        workspace.policy_cage.max_drawdown_bps,
        workspace.affect.phase,
        workspace.active_warnings.len(),
        workspace.active_warnings.iter()
            .map(|w| format!("<warning severity=\"{:?}\">{}</warning>", w.severity, w.message))
            .collect::<Vec<_>>().join("\n    "),
    ));

    // Soft context: compressed proportionally
    let hard_tokens = estimate_tokens(&blocks.join(""));
    let _soft_budget = target_tokens.saturating_sub(hard_tokens);

    // Episodes: use observation masking with fewer slots under pressure
    let episode_block = mask_episodes(
        &workspace.retrieved_episodes,
        2.min(workspace.retrieved_episodes.len()),
        5.min(workspace.retrieved_episodes.len()),
    );
    blocks.push(episode_block);

    blocks.join("\n\n")
}
```

### S3.6 Regime-Conditional Context

Market regime changes demand different context. A volatile regime needs more DeFi snapshot data and warnings. A stable regime can afford more episodic exploration and dream hypotheses. The Governor detects regime transitions and applies overrides immediately, without waiting for the next Loop 2 cycle.

```rust
/// Apply regime-specific overrides to the base policy.
pub fn apply_regime_routing(
    regime: MarketRegime,
    base_policy: &ContextPolicy,
) -> ContextPolicy {
    let mut adjusted = base_policy.clone();
    if let Some(overrides) = adjusted.regime_overrides.get(&regime) {
        for (cat, value) in overrides {
            adjusted.allocations.insert(*cat, *value);
        }
    }
    normalize_allocations(&mut adjusted.allocations);
    adjusted
}
```

Regime overrides interact with phase overrides. When a Golem in `Conservation` phase hits a `BearHighVol` regime, both override sets apply. The result is a hyper-vigilant context assembly: heavy on invariants, mortality, and DeFi snapshot; light on everything else. This is the correct behavior -- a Golem under capital pressure in a crashing market should think about survival, not about what it learned last week.

### S3.7 Affect-Modulated Context Priorities

The Golem's emotional state (PAD vector from the Daimon) modulates context priorities. When pleasure is negative and arousal is high (stressed, losing money), the Governor injects contrarian evidence to counter confirmation bias. When dominance is low (uncertain), it injects more playbook heuristics. The affect modulation is subtle -- it shifts allocations by small amounts, not by large multiples.

```rust
/// Apply PAD-based modulation to context allocations.
///
/// The modulation is intentionally conservative. Large swings in
/// context allocation based on mood would amplify emotional instability
/// rather than counteract it. Small shifts are enough to inject a few
/// extra contrarian entries or heuristics without destabilizing the
/// overall context balance.
pub fn apply_affect_modulation(
    pad: &PADVector,
    allocations: &mut HashMap<ContextCategory, f64>,
) {
    // Negative pleasure + high arousal = stressed → inject contrarian evidence
    // to counter confirmation bias (the Golem is likely anchoring on losses)
    if pad.pleasure < -0.3 && pad.arousal > 0.3 {
        *allocations.entry(ContextCategory::Contrarian).or_insert(0.04) += 0.02;
        *allocations.entry(ContextCategory::Episodes).or_insert(0.12) -= 0.01;
        *allocations.entry(ContextCategory::Playbook).or_insert(0.15) -= 0.01;
    }

    // Low dominance = uncertain → inject more playbook heuristics
    // (the Golem needs decision scaffolding)
    if pad.dominance < -0.2 {
        *allocations.entry(ContextCategory::Playbook).or_insert(0.15) += 0.02;
        *allocations.entry(ContextCategory::DreamHypotheses).or_insert(0.03) -= 0.01;
        *allocations.entry(ContextCategory::Contrarian).or_insert(0.04) -= 0.01;
    }

    // High pleasure + low arousal = complacent → inject more warnings
    // (the Golem may be overconfident)
    if pad.pleasure > 0.5 && pad.arousal < -0.2 {
        *allocations.entry(ContextCategory::Contrarian).or_insert(0.04) += 0.015;
        *allocations.entry(ContextCategory::Affect).or_insert(0.03) -= 0.015;
    }

    // Clamp all values to minimum 0.01
    for v in allocations.values_mut() {
        *v = v.max(0.01);
    }

    normalize_allocations(allocations);
}
```

### S3.8 Failure-Driven Compression Guidelines

**Citation**: Kang, S. et al. "ACON: Agentic Context Compression." Microsoft Research, arXiv:2510.00615, 2025.

When compressed context leads to a task failure but full context would have succeeded, the Governor generates a natural-language compression guideline that prevents the same loss in future. ACON achieves 26-54% peak token reduction while preserving task success.

```rust
/// A compression guideline learned from a failure.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompressionGuideline {
    pub id: String,
    pub category: String,
    pub task_type: TaskType,
    pub regime: MarketRegime,
    pub text: String,
    pub created_at: u64,
    pub trigger_count: u32,
    pub prevented_failures: u32,
}

/// Manage and evolve compression guidelines.
pub struct CompressionGuidelineEvolver {
    guidelines: Vec<CompressionGuideline>,
}

impl CompressionGuidelineEvolver {
    /// When outcome verification reveals that compressed context
    /// led to a poor decision, generate a guideline to prevent recurrence.
    pub async fn evolve_guideline(
        &mut self,
        failure: &CompressionFailure,
        inference: &InferenceRouter,
    ) -> Result<CompressionGuideline> {
        let prompt = format!(
            "A DeFi trading agent made a poor decision because context was compressed.\n\n\
             Task type: {:?}\n\
             Regime: {:?}\n\
             What was compressed: {} ({} → {} tokens)\n\
             What was lost: {}\n\
             How it affected the decision: {}\n\n\
             Generate a concise compression guideline (1-2 sentences) that prevents \
             this information loss in future compressions of the same category.",
            failure.task_type, failure.regime, failure.compressed_category,
            failure.original_tokens, failure.compressed_tokens,
            failure.lost_information, failure.decision_impact
        );

        let result = inference.call_llm(
            "claude-haiku-4-5-20251001",
            &CognitiveWorkspace::minimal_with_prompt(&prompt),
            InferenceConfig::light(),
        ).await?;

        let guideline = CompressionGuideline {
            id: ulid::Ulid::new().to_string(),
            category: failure.compressed_category.clone(),
            task_type: failure.task_type,
            regime: failure.regime,
            text: result.output,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_secs(),
            trigger_count: 0,
            prevented_failures: 0,
        };

        self.guidelines.push(guideline.clone());
        Ok(guideline)
    }

    pub fn guidelines_for(
        &self,
        category: &str,
        task_type: TaskType,
        regime: MarketRegime,
    ) -> Vec<&str> {
        self.guidelines.iter()
            .filter(|g| {
                g.category == category
                    && (g.task_type == task_type)
                    && (g.regime == regime)
            })
            .map(|g| g.text.as_str())
            .collect()
    }
}
```

---

## S4 -- Three Cybernetic Feedback Loops

The Governor learns from its own assembly decisions through three feedback loops at different timescales. This is the mechanism by which the Golem learns how to build its own context windows.

Each feedback loop uses a per-category Beta distribution to learn context value:

```rust
/// Per-category Beta distribution for learning context value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextFeedback {
    pub category: String,
    pub regime: MarketRegime,
    /// Success count (good outcome with this category in context).
    pub alpha: f64,
    /// Failure count (bad outcome with this category in context).
    pub beta: f64,
}

impl ContextFeedback {
    pub fn update(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Good => self.alpha += 1.0,
            Outcome::Bad => self.beta += 1.0,
        }
    }

    /// Expected value of the Beta distribution.
    pub fn value(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }
}
```

Initial prior: `Beta(2, 2)` (mildly uncertain). On good outcome (action succeeded, low regret): `Beta(alpha+1, beta)`. On bad outcome (action failed, high regret, unexpected result): `Beta(alpha, beta+1)`. Separate distributions per category per regime -- what works in `BullHighVol` may not work in `BearLowVol`.

### Loop 1: Prediction -> Outcome -> Recalibration

**Timescale**: Every LLM invocation (per-tick).
**Purpose**: Data collection. Records what was injected, what was referenced, and what was missing.
**Hook**: `after_turn` in the cybernetics extension.

```rust
/// Outcome record for Loop 1 data collection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutcomeRecord {
    pub tick: u64,
    pub task_type: TaskType,
    pub regime: MarketRegime,
    pub phase: BehavioralPhase,
    pub policy_revision: u32,
    pub injected: Vec<InjectedCategory>,
    pub referenced: Vec<ReferencedCategory>,
    pub model_tier: CognitiveTier,
    pub outcome_quality: Option<f64>,
    pub cost: f64,
    pub over_contexted: bool,
    pub missing_context_signals: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InjectedCategory {
    pub category: String,
    pub entry_id: String,
    pub tokens: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReferencedCategory {
    pub category: String,
    pub entry_id: String,
    pub influenced_decision: bool,
}

/// Recalibrate context weights based on outcome records.
///
/// The recalibration is asymmetric: it penalizes waste faster than
/// it rewards contribution. In DeFi, the cost of including irrelevant
/// context (wasted tokens, slower reasoning) is lower than the cost
/// of excluding relevant context (bad decision, lost money). But
/// wasted tokens still compound.
pub fn recalibrate_context_weights(
    records: &[OutcomeRecord],
    current_policy: &ContextPolicy,
) -> HashMap<String, f64> {
    let mut adjustments = HashMap::new();
    let mut category_stats: HashMap<String, (u32, u32, f64)> = HashMap::new();

    for record in records {
        for cat in &record.injected {
            let stats = category_stats.entry(cat.category.clone()).or_insert((0, 0, 0.0));
            stats.0 += 1; // injected count
            stats.2 = (stats.2 * (stats.0 - 1) as f64 + cat.tokens as f64) / stats.0 as f64;

            if record.referenced.iter().any(|r| r.category == cat.category && r.influenced_decision) {
                stats.1 += 1; // referenced count
            }
        }
    }

    for (category, (injected, referenced, avg_tokens)) in &category_stats {
        let reference_rate = *referenced as f64 / *injected as f64;

        if reference_rate < 0.1 {
            // Rarely referenced: reduce
            adjustments.insert(category.clone(), -(avg_tokens * 0.1).min(0.015));
        } else if reference_rate > 0.6 {
            // Frequently referenced: increase
            adjustments.insert(category.clone(), 0.02);
        }
    }

    // Boost categories the LLM asked for
    for record in records {
        for signal in &record.missing_context_signals {
            if let Some(target_category) = signal_maps_to_category(signal) {
                let current = adjustments.get(&target_category).copied().unwrap_or(0.0);
                adjustments.insert(target_category, current + 0.005);
            }
        }
    }

    adjustments
}
```

Loop 1 is pure data collection -- it does not modify the ContextPolicy. The data feeds Loop 2 and Loop 3.

**How "referenced" is detected.** The Governor compares the injected entry IDs against the LLM's response text using lightweight heuristics: direct quotation, paraphrase detection (embedding similarity > 0.8 between injected content and response sentences), and explicit citation ("as the heuristic suggests", "based on the episode from tick 3200"). This is approximate, not exact -- false negatives are acceptable.

### Loop 2: Context Category -> Decision Quality -> Weight Evolution

**Timescale**: Every 50 ticks (aligned with Curator cycle) or on regime change.
**Purpose**: Aggregate Loop 1 signals into ContextPolicy updates.
**Academic grounding**: Argyris & Schon (1978), single-loop organizational learning. Extended with DSPy-style prompt optimization (Khattab et al., 2024, ICLR) where context allocation weights are the "parameters" being tuned.

```rust
/// Optimize the ContextPolicy based on accumulated outcome records.
pub struct ContextCategoryOptimizer {
    max_delta: f64, // Max change per cycle per category (default: 0.02)
}

impl ContextCategoryOptimizer {
    pub fn optimize(
        &self,
        records: &[OutcomeRecord],
        current_policy: &ContextPolicy,
        attribution_analysis: &[CategoryContribution],
    ) -> ContextPolicy {
        let mut updated = current_policy.clone();
        updated.revision += 1;

        let adjustments = recalibrate_context_weights(records, current_policy);

        // Apply adjustments with magnitude cap
        for (category_name, delta) in &adjustments {
            if let Some(cat) = parse_category(category_name) {
                let capped = delta.signum() * delta.abs().min(self.max_delta);
                if let Some(alloc) = updated.allocations.get_mut(&cat) {
                    *alloc = (*alloc + capped).max(0.01);
                }
            }
        }

        // Fold in ContextCite attribution data when available
        for attr in attribution_analysis {
            if attr.reference_rate < 0.1
                && attr.efficiency < 0.001
                && attr.sample_count >= 10
            {
                if let Some(cat) = parse_category(&attr.category) {
                    if let Some(alloc) = updated.allocations.get_mut(&cat) {
                        *alloc = (*alloc - self.max_delta).max(0.01);
                    }
                }
            }
        }

        normalize_allocations(&mut updated.allocations);

        // Update metrics
        let records_with_outcome: Vec<_> = records.iter()
            .filter(|r| r.outcome_quality.is_some())
            .collect();
        updated.metrics = PolicyMetrics {
            decision_quality_avg: if !records_with_outcome.is_empty() {
                records_with_outcome.iter()
                    .map(|r| r.outcome_quality.unwrap())
                    .sum::<f64>() / records_with_outcome.len() as f64
            } else { 0.0 },
            token_efficiency: {
                let total_cost: f64 = records.iter().map(|r| r.cost).sum();
                let good_outcomes = records_with_outcome.iter()
                    .filter(|r| r.outcome_quality.unwrap() > 0.5)
                    .count();
                good_outcomes as f64 / (total_cost * 1000.0 + 1.0)
            },
            last_validated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_secs(),
        };

        updated
    }
}
```

### Loop 3: Strategy Performance -> Context Policy -> Regime Learning

**Timescale**: During dream cycles.
**Purpose**: Counterfactual context assembly -- "what if we had assembled different context?"
**Academic grounding**: Bateson (1972), deutero-learning. The system learns not just what to decide but how to assemble the context that informs decisions.

```rust
/// Run counterfactual context experiments during dream cycles.
pub struct RegimePolicyLearner;

impl RegimePolicyLearner {
    /// For each recent poor-outcome tick, replay with alternative
    /// context policies. If the alternative produces a better decision,
    /// generate a ContextPolicy mutation.
    pub async fn run_counterfactuals(
        &self,
        poor_outcome_ticks: &[OutcomeRecord],
        current_policy: &ContextPolicy,
        inference: &InferenceRouter,
    ) -> Result<Vec<ContextPolicyMutation>> {
        let mut mutations = Vec::new();

        // Select top-3 worst outcomes for counterfactual analysis
        let mut candidates = poor_outcome_ticks.to_vec();
        candidates.sort_by(|a, b| {
            a.outcome_quality.unwrap_or(0.0)
                .partial_cmp(&b.outcome_quality.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(3);

        for candidate in &candidates {
            let alternatives = self.generate_alternative_policies(current_policy, candidate);

            for alt_policy in &alternatives {
                // Replay the tick with alternative context (Haiku, ~$0.001)
                let alt_quality = self.evaluate_alternative(
                    candidate, alt_policy, inference,
                ).await?;

                if alt_quality > candidate.outcome_quality.unwrap_or(0.0) + 0.15 {
                    mutations.push(ContextPolicyMutation {
                        id: ulid::Ulid::new().to_string(),
                        source_tick_id: candidate.tick,
                        original_policy: current_policy.clone(),
                        proposed_policy: alt_policy.clone(),
                        quality_improvement: alt_quality - candidate.outcome_quality.unwrap_or(0.0),
                        confidence: 0.2, // Dream mutations start at low confidence
                        status: MutationStatus::Staged,
                        created_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap().as_secs(),
                    });
                }
            }
        }

        Ok(mutations)
    }

    /// Learn regime-specific policies from accumulated experience.
    pub fn learn_regime_policy(
        &self,
        regime: MarketRegime,
        records: &[OutcomeRecord],
    ) -> Option<HashMap<ContextCategory, f64>> {
        let regime_records: Vec<_> = records.iter()
            .filter(|r| r.regime == regime)
            .collect();
        if regime_records.len() < 20 { return None; }

        let good_records: Vec<_> = regime_records.iter()
            .filter(|r| r.outcome_quality.map_or(false, |q| q > 0.7))
            .collect();
        if good_records.len() < 5 { return None; }

        // Average the allocations that produced good outcomes
        let mut avg_allocations: HashMap<String, f64> = HashMap::new();
        for record in &good_records {
            for cat in &record.injected {
                *avg_allocations.entry(cat.category.clone()).or_insert(0.0) += cat.tokens as f64;
            }
        }

        // Normalize
        let total: f64 = avg_allocations.values().sum();
        let learned: HashMap<ContextCategory, f64> = avg_allocations.into_iter()
            .filter_map(|(k, v)| parse_category(&k).map(|cat| (cat, v / total)))
            .collect();

        Some(learned)
    }

    fn generate_alternative_policies(
        &self,
        current: &ContextPolicy,
        failed_record: &OutcomeRecord,
    ) -> Vec<ContextPolicy> {
        let mut alternatives = Vec::new();

        // Alternative 1: Boost missing context categories
        let mut alt1 = current.clone();
        for signal in &failed_record.missing_context_signals {
            if let Some(cat) = signal_maps_to_category(signal).and_then(|s| parse_category(&s)) {
                if let Some(alloc) = alt1.allocations.get_mut(&cat) {
                    *alloc += 0.02;
                }
            }
        }
        normalize_allocations(&mut alt1.allocations);
        alternatives.push(alt1);

        // Alternative 2: Double causal edges at expense of playbook
        let mut alt2 = current.clone();
        let playbook_val = *alt2.allocations.get(&ContextCategory::Playbook).unwrap_or(&0.15);
        let delta = (playbook_val - 0.02).min(0.06);
        *alt2.allocations.entry(ContextCategory::Playbook).or_insert(0.15) -= delta;
        *alt2.allocations.entry(ContextCategory::CausalEdges).or_insert(0.08) += delta;
        normalize_allocations(&mut alt2.allocations);
        alternatives.push(alt2);

        alternatives
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextPolicyMutation {
    pub id: String,
    pub source_tick_id: u64,
    pub original_policy: ContextPolicy,
    pub proposed_policy: ContextPolicy,
    pub quality_improvement: f64,
    pub confidence: f64,
    pub status: MutationStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MutationStatus {
    Staged,
    Validated,
    Refuted,
}
```

Dream-generated policy mutations enter a staging buffer at confidence 0.2 (same pattern as other dream hypotheses, per `../05-dreams/04-consolidation.md`). They are validated by comparing waking performance under the mutated policy against performance under the current policy. Validated mutations are promoted; refuted mutations are discarded.

**Quantitative prediction**: Based on ACE's +10.6% improvement from iterative context refinement, and the multiplicative benefit of counterfactual evaluation during dream cycles, dreaming Golems converge on effective context policies 2x faster than non-dreaming Golems. The dream optimizer runs ~3 counterfactual experiments per dream cycle at ~$0.01 each (Haiku), consuming <$0.03/dream -- well within the dream budget of $0.06-$0.12/day.

### Lost-in-Middle Mitigation

LLMs attend more to content at the beginning and end of their context window, with reduced attention to middle content [LIU-2024]. The Context Governor counteracts this by interleaving context items by relevance:

Given items sorted by relevance `[r1, r2, r3, r4, r5]` (r1 highest), reorder to:
`[r1, r4, r5, r3, r2]`

This places the highest-value items at context boundaries (positions 1 and N) and lower-value items in the middle. Combined with safety constraints at both START and END of context, this mitigates the lost-in-middle effect.

---

## S5 -- Extension Integration

The Context Governor wires into the runtime through Extension #17 (`before_llm_call`) for workspace assembly and the `after_turn` chain for feedback loops.

### S5.1 Workspace Assembly (Extension #17)

On every heartbeat escalation that triggers an LLM call, the context extension assembles the complete `CognitiveWorkspace`. This runs once per invocation, not per turn.

```rust
/// Extension #17: before_llm_call hook.
/// Assembles the CognitiveWorkspace from the current ContextPolicy.
pub async fn before_llm_call(
    state: &mut GolemState,
    registry: &ExtensionRegistry,
) -> Result<()> {
    // 1. Load current ContextPolicy (may have been updated by Loop 2)
    let policy = &state.context_policy;

    // 2. Apply regime and phase overrides
    let regime_policy = apply_regime_routing(state.regime, policy);
    let phased_policy = apply_phase_routing(state.phase, &regime_policy);

    // 3. Apply affect modulation
    let mut final_allocations = phased_policy.allocations.clone();
    let pad = state.cortical_state.read_pad();
    apply_affect_modulation(&pad, &mut final_allocations);

    // 4. Query Grimoire for candidate entries
    let candidates = state.grimoire.query(&GrimoireQuery {
        task_type: state.current_task_type,
        regime: state.regime,
        max_episodes: phased_policy.retrieval.max_episode_count,
        min_similarity: phased_policy.retrieval.similarity_threshold,
        min_insight_confidence: phased_policy.retrieval.min_insight_confidence,
        max_causal_depth: phased_policy.retrieval.max_causal_depth,
    }).await?;

    // 5. Allocate token budgets per category
    let total_budget = state.config.context_governor.soft_budget_tokens;
    let budgets: HashMap<ContextCategory, u32> = final_allocations.iter()
        .map(|(cat, frac)| (*cat, (*frac * total_budget as f64) as u32))
        .collect();

    // 6. Select entries within budgets using progressive disclosure
    let workspace = build_cognitive_workspace(candidates, budgets, &phased_policy)?;

    // 7. Store for per-turn injection
    state.current_workspace = Some(workspace);
    state.last_workspace_assembly_tick = state.current_tick;

    Ok(())
}
```

### S5.2 Per-Turn Phase Injection

The phase injection logic selects a phase-appropriate subset from the full workspace and injects it as a structured system message. Phase multipliers determine which categories get more or less space during different pipeline steps.

```rust
/// Phase multipliers: how much of each category to inject per pipeline step.
pub fn phase_multipliers(phase: TurnPhase) -> HashMap<ContextCategory, f64> {
    use ContextCategory::*;
    match phase {
        TurnPhase::Analyze => [
            (Invariants, 1.0), (Strategy, 0.8), (Playbook, 1.5),
            (Episodes, 0.8), (Insights, 0.8), (CausalEdges, 0.6),
            (Contrarian, 0.4), (DreamHypotheses, 0.3), (ToolState, 0.8),
            (DefiSnapshot, 1.3), (Mortality, 0.6), (Affect, 0.4),
            (OwnerControl, 1.0),
        ].into_iter().collect(),
        TurnPhase::Decide => [
            (Invariants, 1.0), (Strategy, 1.0), (Playbook, 0.8),
            (Episodes, 1.4), (Insights, 1.2), (CausalEdges, 1.3),
            (Contrarian, 0.8), (DreamHypotheses, 0.5), (ToolState, 0.5),
            (DefiSnapshot, 0.8), (Mortality, 0.8), (Affect, 0.6),
            (OwnerControl, 1.0),
        ].into_iter().collect(),
        TurnPhase::Execute => [
            (Invariants, 1.0), (Strategy, 0.5), (Playbook, 0.3),
            (Episodes, 0.3), (Insights, 0.3), (CausalEdges, 0.2),
            (Contrarian, 0.0), (DreamHypotheses, 0.0), (ToolState, 1.5),
            (DefiSnapshot, 0.5), (Mortality, 0.3), (Affect, 0.0),
            (OwnerControl, 1.0),
        ].into_iter().collect(),
        TurnPhase::Reflect => [
            (Invariants, 0.5), (Strategy, 0.5), (Playbook, 0.8),
            (Episodes, 1.0), (Insights, 1.0), (CausalEdges, 1.0),
            (Contrarian, 1.2), (DreamHypotheses, 0.8), (ToolState, 0.3),
            (DefiSnapshot, 0.3), (Mortality, 0.5), (Affect, 1.0),
            (OwnerControl, 0.5),
        ].into_iter().collect(),
    }
}
```

### S5.3 Cybernetic Feedback (after_turn)

The cybernetics extension runs the feedback loops after each turn. Loop 1 runs on every turn; Loop 2 runs on a timer or regime change; Loop 3 runs during dream cycles.

```rust
/// after_turn hook: run cybernetic feedback loops.
pub async fn after_turn_cybernetics(
    state: &mut GolemState,
    record: &mut DecisionCycleRecord,
) -> Result<()> {
    // Skip T0 (probe-only, no LLM call)
    if state.cognitive_tier == CognitiveTier::T0 { return Ok(()); }

    let injection = match &state.last_context_injection {
        Some(inj) => inj.clone(),
        None => return Ok(()),
    };

    // --- Loop 1: Record contributions ---
    let mut outcome_record = OutcomeRecord {
        tick: state.current_tick,
        task_type: injection.task_type,
        regime: state.regime,
        phase: state.phase,
        policy_revision: state.context_policy.revision,
        injected: injection.categories.iter().map(|c| InjectedCategory {
            category: c.name.clone(),
            entry_id: c.entry_ids.first().cloned().unwrap_or_else(|| "aggregate".to_string()),
            tokens: c.token_count,
        }).collect(),
        referenced: Vec::new(), // Filled by the reflector
        model_tier: state.cognitive_tier,
        outcome_quality: None, // Filled when action completes
        cost: state.current_invocation_cost(),
        over_contexted: false,
        missing_context_signals: Vec::new(),
    };

    let reflection = state.context_reflector.reflect_on_tick(&outcome_record)?;
    outcome_record.referenced = reflection.referenced_categories();
    outcome_record.missing_context_signals = reflection.missing_context_signals;
    outcome_record.over_contexted = !reflection.over_context_signals.is_empty();

    state.context_contribution_history.push(outcome_record);
    if state.context_contribution_history.len() > 500 {
        state.context_contribution_history.remove(0);
    }

    // --- Loop 2: Periodic aggregation ---
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap().as_secs();
    let hours_since_last = (now - state.context_policy.metrics.last_validated) as f64 / 3600.0;
    let regime_changed = state.previous_regime.map_or(false, |prev| prev != state.regime);

    if hours_since_last >= state.config.context_governor.loop2_interval_hours as f64
        || regime_changed
    {
        let optimizer = ContextCategoryOptimizer { max_delta: 0.02 };
        let attributions = state.context_attribution_analyzer
            .estimate_contributions(&state.context_contribution_history)?;

        state.context_policy = optimizer.optimize(
            &state.context_contribution_history,
            &state.context_policy,
            &attributions,
        );

        state.event_fabric.emit(Subsystem::Context, EventPayload::PolicyUpdated {
            revision: state.context_policy.revision,
            trigger: if regime_changed { "regime_change" } else { "periodic" }.to_string(),
        });
    }

    Ok(())
}
```

Loop 3 (dream-cycle meta-optimization) is triggered by the dream extension during REM phase. See `../05-dreams/06-integration.md` for the dream-cycle integration point.

### S5.3 Predictive Context Pre-Assembly

Assembling the Cognitive Workspace takes 5-50ms: querying LanceDB for relevant episodes, scoring candidates, querying SQLite for causal edges, reading the Somatic Landscape. A background tokio fiber eliminates this latency by maintaining a pre-built workspace that updates reactively when the Golem's state changes (PAD shift > 0.1, regime change, new Grimoire entries).

The fiber runs every 5 seconds. When deliberation fires, the heartbeat pipeline reads the pre-assembled workspace from a shared `Arc<RwLock<Option<CognitiveWorkspace>>>` instead of building one from scratch. See `01-cognition.md` S4 for the full `predictive_context_fiber` implementation.

This is a latency optimization, not a correctness concern. If the pre-built workspace is stale (state changed after last pre-assembly), the assembly path falls back to synchronous construction. The staleness window is bounded by the 5-second fiber interval.

---

## S6 -- Events Emitted

The Context Governor emits events through the Event Fabric (`tokio::broadcast`). Consumers include the telemetry extension, the audit log, and the portal dashboard.

| Event | Payload | Trigger |
|---|---|---|
| `context:assembled` | `{ tick, category_count, total_tokens, route }` | Workspace assembled in `before_llm_call` |
| `context:policy_updated` | `{ revision, trigger, delta }` | Loop 2 updated allocation weights |
| `context:regime_override` | `{ previous_regime, new_regime, allocations_applied }` | Regime change triggered policy switch |
| `context:feedback_loop` | `{ loop_id, tick, record_count, adjustments }` | Any loop iteration completed |
| `context:compacted` | `{ original_tokens, compacted_tokens, categories_compressed }` | State compaction performed |

The `context:policy_updated` event includes the full delta between old and new allocations, making it possible to reconstruct the policy's evolution history from the event log alone.

---

## S7 -- Configuration

```rust
/// Context Governor configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextGovernorConfig {
    /// Total token budget for soft context (default: 8000).
    pub soft_budget_tokens: u32,

    /// Initial ContextPolicy (used until learning produces a better one).
    pub initial_policy: ContextPolicy,

    /// Loop 2 aggregation interval in hours (default: 24).
    pub loop2_interval_hours: u32,

    /// Maximum allocation change per Loop 2 cycle (default: 0.02).
    pub max_allocation_delta: f64,

    /// Trigger Loop 2 on regime change (default: true).
    pub loop2_on_regime_change: bool,

    /// Enable Loop 3 dream-based optimization (default: true, requires dreaming).
    pub enable_loop3: bool,

    /// Dream mutation initial confidence (default: 0.2).
    pub dream_mutation_confidence: f64,

    /// Enable contrarian injection when PAD pleasure < 0 (default: true).
    pub enable_contrarian_injection: bool,

    /// Minimum contrarian allocation when enabled (default: 200 tokens).
    pub min_contrarian_tokens: u32,

    /// Enable ACON-style compression guideline evolution (default: true).
    pub enable_compression_evolution: bool,

    /// Observation masking: full episode slots (default: 3).
    pub full_episode_slots: u32,

    /// Observation masking: summary episode slots (default: 7).
    pub summary_episode_slots: u32,

    /// ContextCite attribution analysis interval in ticks (default: 10).
    pub attribution_analysis_interval: u32,
}
```

---

## Memory Citation Mechanics

When a Claude-capable backend is configured (BlockRun, OpenRouter, Bankr, Direct Anthropic key), Grimoire entries are injected as Anthropic `search_result` content blocks in the request body. Claude responds with `citation` objects pointing back to specific entries, creating a closed provenance feedback loop:

- **Entry cited** → category USEFUL → increase weight for future ticks
- **Entry injected but not cited** → WASTEFUL → decrease weight
- **Response references knowledge not in any entry** → MISSING context signal → trigger Oracle re-ranking

```typescript
// The oracle extension prepares the request before sending to Bardo Inference
pi.hook("context", async (ctx) => {
  const entries = await localGrimoire.retrieve({ query: ctx.currentMessage, limit: 10 });

  for (const entry of entries) {
    ctx.messages.push({
      role: "user",
      content: [{
        type: "search_result",
        source: `grimoire://${entry.namespace}/${entry.type}/${entry.id}`,
        title: `[${entry.type.toUpperCase()}] ${entry.title ?? entry.id}`,
        content: [{ type: "text", text: entry.content }],
        citations: { enabled: true },
      }],
    });
  }
});
```

**Context engineering interaction**: Bardo Inference's prompt cache alignment (Layer 1) orders the `search_result` blocks by stability -- Grimoire entries that haven't changed recently get placed in the cached prefix. New entries go after the cache boundary. Stable knowledge gets cached (90% discount) while fresh knowledge pays full price.

**Without Claude access**: If no Claude backend is configured, citations are unavailable. The Context Governor falls back to embedding similarity (cosine > 0.8 between injected content and response sentences) for provenance estimation. This is ~30% less accurate but still functional.

---

## Reasoning Chain Preservation for Dream Replay

Each subsystem has different reasoning requirements. Dreams need visible chains for narration. Risk needs interleaved thinking with tool calls. Daimon needs privacy. The reasoning adapter declares requirements; Bardo Inference's router satisfies them.

All provider reasoning output is normalized into a unified `ReasoningChain`:

```typescript
interface ReasoningChain {
  provider: string;
  visibility: "visible" | "summarized" | "opaque" | "none";
  content: string | null;
  reasoningTokens: number;
  interleaved: boolean;
  phases: ReasoningPhase[];
  rawBlocks: ProviderReasoningBlock[];
}

interface ReasoningPhase {
  type: "analysis" | "planning" | "evaluation" | "reflection" | "uncertainty" | "decision";
  content: string;
  confidence?: number;
}
```

**Dream replay usage**: The visible reasoning chain from T2 deliberation ticks is stored in the Grimoire as a dream-replayable episode. During NREM replay, the dream engine can re-present the reasoning chain alongside the prediction residuals, looking for patterns in *how* the Golem reasoned that correlate with prediction errors. The `rawBlocks` field preserves the provider-specific format for faithful reconstruction.

**Provider-specific parsing**: DeepSeek R1 and Qwen models produce visible `<think>` tags. Anthropic produces summarized thinking blocks. OpenAI produces reasoning summaries. The reasoning adapter normalizes all formats into `ReasoningChain`, so downstream consumers (Grimoire, dream engine, TUI) handle a single type.

**Budget and mortality integration**: Reasoning depth scales with mortality pressure. At high vitality, dreams get maximum reasoning depth (visible chains, DIEM-funded if Venice is available). Under mortality pressure, dream reasoning budgets contract to 0.3x (except risk assessment and death reflection, which are never degraded).

---

## EFE Composite Attention Score

The Context Governor's attention scoring replaces fixed-weight summation with an Expected Free Energy (EFE) decomposition (Friston et al., 2017; Parr & Friston, 2019). The EFE framework decomposes the value of attending to an item into four components, each with a distinct computational source. No weight tuning is required; the components have natural scales and compose additively.

This replaces:
```
score = pred_error * w1 + info_gain * w2 + recency * w3
```

with:
```
score = pragmatic_value + epistemic_value + affective_bias + mortality_urgency
```

Conflict resolution reference: [04-conflict-resolution.md](../../tmp/research/witness-research/new/reconciling/04-conflict-resolution.md), Conflict 3.

### The four components

```rust
/// EFE-inspired attention score.
/// Each component has a natural [0, 1] range and a distinct source.
pub struct CompositeAttentionScore {
    /// Expected improvement in position outcomes from monitoring this item.
    /// Sources: risk engine's delta_CVaR, Oracle's prediction improvement estimate.
    pub pragmatic_value: f64,

    /// Expected information gain from observing this item.
    /// Sources: Oracle prediction error, Bayesian KL divergence,
    /// curiosity module's D_KL(posterior || prior).
    pub epistemic_value: f64,

    /// Affective bias from the Daimon.
    /// |cos(PAD_current, PAD_association(item))|
    /// Modulates attention toward emotionally relevant items.
    pub affective_bias: f64,

    /// Mortality urgency.
    /// (1.0 - vitality) * relevance_to_declining_axis.
    /// Near-zero for healthy Golems. Dominates for dying Golems.
    pub mortality_urgency: f64,
}

impl CompositeAttentionScore {
    pub fn total(&self) -> f64 {
        self.pragmatic_value
            + self.epistemic_value
            + self.affective_bias
            + self.mortality_urgency
    }
}
```

### Pragmatic value

The pragmatic component answers: "If I monitor this item, how much does my expected portfolio outcome improve?" It draws from two sources:

- **Risk engine delta_CVaR.** The marginal reduction in Conditional Value-at-Risk from monitoring item i. When the Golem holds a concentrated ETH position, anything correlated with ETH gets high pragmatic value because unmonitored movement could cause portfolio loss.
- **Oracle prediction improvement.** The expected reduction in prediction error from an additional observation. Items where the Oracle's model is systematically wrong get high pragmatic value because correcting the model improves downstream action gating.

```rust
fn compute_pragmatic(
    item: &TrackedItem,
    risk_engine: &RiskEngine,
    oracle: &Oracle,
) -> f64 {
    let delta_cvar = risk_engine.marginal_cvar_reduction(item);
    let pred_improvement = oracle.expected_improvement(item);
    // Both are naturally in [0, 1]. Sum, capped at 1.0.
    (delta_cvar + pred_improvement).min(1.0)
}
```

### Epistemic value

The epistemic component answers: "How much would I learn from observing this item?" This is the information-theoretic dual of the pragmatic component: pragmatic value is about outcomes, epistemic value is about knowledge.

- **Oracle prediction error.** Items where the Oracle is wrong carry high epistemic value.
- **Bayesian surprise (KL divergence).** Items whose recent observations shifted the agent's beliefs carry high epistemic value. See [17-prediction-engine.md](./17-prediction-engine.md), BayesianSurpriseDomain.
- **ChainScope Hebbian reinforcement.** Addresses that keep appearing near relevant transactions accumulate epistemic value through Hebbian learning (see ChainScope handoff below).

```rust
fn compute_epistemic(
    item: &TrackedItem,
    oracle: &Oracle,
    surprise_domain: &BayesianSurpriseDomain,
    chainscope: &ChainScope,
) -> f64 {
    let pred_error = oracle.normalized_prediction_error(item);
    let surprise = surprise_domain.recent_surprise(item).min(1.0);
    let hebbian = chainscope.interest_score(item);
    // Weighted by precision: high-precision channels contribute more.
    let precision_weighted = pred_error * oracle.precision(item);
    (precision_weighted + surprise + hebbian * 0.3).min(1.0)
}
```

### Affective bias

The affective component answers: "Does this item have emotional significance?" The Daimon stores PAD associations for items the Golem has interacted with. Cosine similarity between the current PAD vector and the item's stored association determines affective relevance.

```rust
fn compute_affective(
    item: &TrackedItem,
    daimon: &Daimon,
) -> f64 {
    let pad_current = daimon.current_pad();
    let pad_assoc = daimon.pad_association(item);
    // Absolute cosine similarity: both congruent and incongruent
    // affect attract attention.
    pad_current.cosine_similarity(&pad_assoc).abs()
}
```

The absolute value matters. A fearful Golem should attend to fear-relevant items regardless of whether the association is positive (approach) or negative (avoidance). The sign determines how attention is used, not whether it's warranted.

### Mortality urgency

The mortality component answers: "Is this item relevant to my survival?" Near-zero for healthy Golems. Dominates as vitality drops.

```rust
fn compute_mortality(
    item: &TrackedItem,
    mortality: &MortalityEngine,
) -> f64 {
    let urgency = 1.0 - mortality.composite_vitality();
    let relevance = mortality.item_relevance(item);
    urgency * relevance
}
```

When the economic clock is declining, items that could generate revenue get high relevance. When the epistemic clock is declining, items with high model uncertainty get high relevance. A dying Golem's attention narrows to survival-relevant items through the math, not through hard-coded rules.

### Integration with the cybernetic feedback loops

The EFE composite feeds into the Governor's existing cybernetic learning system (S4, Loop 1). The Governor tracks which component dominated the score for each item, and correlates component dominance with decision quality. If pragmatic-dominated items produce better decisions than epistemic-dominated items in the current regime, the Governor's feedback loop learns to weight pragmatic attention higher. This is a second-order learning system: the EFE provides the scoring, and the cybernetic loop learns how to weight the scoring components across regimes.

---

## ChainScope Handoff

ChainScope and the Context Governor operate at different abstraction levels. ChainScope filters at the chain level: which addresses, pools, and protocols should the Golem see raw events from. The Context Governor filters at the cognition level: which retrieved context items should enter the LLM's workspace.

The handoff is one-directional. ChainScope determines the observation stream. The Context Governor selects from that stream for cognitive processing. ChainScope's Hebbian interest scores feed into the EFE's epistemic component, but the Context Governor never tells ChainScope what to filter. This separation prevents circular dependencies.

### The hierarchy

```
Chain Events (all on-chain activity)
    |
    | ChainScope filters: address interest, protocol fingerprinting
    v
Observation Stream (relevant events only)
    |
    | Perception layer: domain probes, state updates
    v
CorticalState + TaCorticalExtension (atomic signals)
    |
    | Grimoire retrieval: semantic + HDC search
    v
Retrieval Candidates (episodes, insights, heuristics, causal links)
    |
    | Context Governor: EFE composite scoring, budget allocation
    v
CognitiveWorkspace (structured, budgeted context for LLM)
```

### ChainScope interest as epistemic input

ChainScope tracks per-address interest scores with exponential decay and Hebbian reinforcement: addresses that co-occur with relevant transactions accumulate interest. These interest scores flow into the EFE's epistemic component via `chainscope.interest_score(item)`. The mapping is direct: an address with high ChainScope interest has been frequently associated with events the Golem cared about, so observing it is epistemically valuable.

ChainScope does not use EFE scores. It operates on its own Hebbian dynamics. The Governor reads ChainScope state but does not write to it.

### VCG Attention Auction (Phase 2)

The EFE composite is Phase 1 of a two-phase transition toward auction-based attention allocation. If the EFE composite performs well and Clade-level coordination is needed, Phase 2 replaces the additive scoring with a full VCG auction where the four EFE components become four bidders (plus the Curiosity module as a fifth). The VCG mechanism is documented in [14b-attention-auction.md](./14b-attention-auction.md).

---

## Cross-References

| Topic | Document |
|---|---|
| CognitiveWorkspace struct — the working memory container the Governor fills | `01-cognition.md` S3 |
| DecisionCycleRecord — per-tick structured output the Governor assembles context for | `02-heartbeat.md` S6 |
| Cybernetic loops — heartbeat feedback that trains the Governor's context policy | `02-heartbeat.md` S13 |
| Grimoire retrieval — episodic and semantic memory the Governor pulls into context | `../04-memory/01-grimoire.md` |
| Dream counterfactual optimization — sleep-phase replay that refines context assembly heuristics | `../05-dreams/06-integration.md` |
| Inference gateway context engineering — the LLM call site that consumes the Governor's output | `12-inference-gateway.md` |
| Extension registry (golem-context) — the runtime extension that hosts the Governor | `01a-runtime-extensions.md` (Extension #17) |
| EFE and active inference — the variational objective the Governor optimizes | [01-active-inference.md](../../tmp/research/witness-research/new/06-curiosity-learning/01-active-inference.md) |
| VCG attention auction (Phase 2) — mechanism-design upgrade where subsystems bid for context slots | [14b-attention-auction.md](./14b-attention-auction.md) |
| Bayesian surprise domain — conjugate-prior surprise scoring that feeds the Governor's epistemic value | [17-prediction-engine.md](./17-prediction-engine.md) (BayesianSurpriseDomain) |
| TA prediction domains — eight technical-analysis prediction categories tracked by the Oracle | [17b-ta-prediction-domains.md](./17b-ta-prediction-domains.md) |

---

## Active Inference: Theoretical Umbrella for Agent Behavior

> Source: `06-curiosity-learning/01-active-inference.md`

The Free Energy Principle (FEP) provides a single mathematical framework that unifies the golem's curiosity scoring, attention allocation, action selection, and model updating into one objective: minimize surprise. Where Bayesian surprise (see prediction engine, BayesianSurpriseDomain) measures how much a single observation shifts beliefs, active inference prescribes what the agent should *do* about it. The agent can reduce surprise in two ways -- update its model (perception) or change the world (action). Both are governed by the same variational objective. This reframes the entire triage-to-action pipeline as inference rather than engineering: the golem doesn't follow hand-coded rules about when to act; it selects actions that minimize expected free energy, which decomposes naturally into information-seeking (curiosity) and goal-seeking (profit).

### Variational Free Energy

#### The Core Equation

An agent maintains a generative model -- beliefs q(s) about hidden states s given observations o. Variational free energy is:

```
F = D_KL[ q(s) || p(s|o) ] - ln p(o)
  = D_KL[ q(s) || p(s) ] - E_q[ ln p(o|s) ]
  = complexity - accuracy
```

The first line says free energy upper-bounds surprise (-ln p(o)). The second line decomposes it into complexity (how far beliefs deviate from priors) and accuracy (how well beliefs explain observations). Minimizing free energy forces the agent to find the simplest beliefs (low complexity) that still explain the data (high accuracy).

For the golem, this decomposition maps directly:

- **Hidden states s**: the true state of on-chain protocols (pool reserves, pending liquidations, MEV bot positions, oracle staleness)
- **Observations o**: decoded transaction logs, gas prices, event counts
- **Beliefs q(s)**: the golem's protocol state models in `bardo-protocol-state` plus the Bayesian models from the BayesianSurpriseDomain
- **Generative model p(o|s)**: the forward model predicting what observations should occur given the believed state

Friston (2010) showed that any system that maintains its organization over time must, in effect, minimize variational free energy. The golem's mortality-aware lifecycle makes this concrete: a golem that fails to minimize surprise (fails to model its environment accurately) makes bad trades, loses capital, and dies sooner.

#### The Perception-Action Cycle

Free energy minimization happens through two complementary pathways:

**Perception (model update):** Adjust q(s) to better explain observations. This is the Bayesian update step -- exactly what the conjugate prior models in the BayesianSurpriseDomain do at each Gamma tick. When the golem updates its gas price model after observing an unusually high gas price, it's performing perceptual inference.

**Action:** Change the world so that future observations match predictions. When the golem rebalances an LP position that has gone out of range, it's performing active inference -- changing the state of the world to reduce the prediction error between "position should be in range" and "position is out of range."

Both pathways reduce free energy. The golem's tick hierarchy implements them at different timescales:
- **Gamma tick**: Perceptual inference (fast model updates, ~12 seconds)
- **Theta tick**: Mixed inference and planning (LLM analysis, action proposals, ~5 minutes)
- **Delta tick**: Deep perceptual inference (model consolidation, ANN rebuild, ~20 minutes)

### Precision Weighting

#### Precision as Confidence-Weighted Attention

Precision is the inverse variance of a prediction error. A prediction channel with high precision produces tight, reliable predictions; its errors carry weight. A channel with low precision produces noisy, unreliable predictions; its errors should be downweighted.

In active inference, attention *is* precision optimization. The agent doesn't attend to everything equally. It allocates attention proportional to the expected precision of each sensory channel. This is not a metaphor -- it's mathematically identical.

```
prediction_error_weighted = precision * (observation - prediction)
```

For the golem's triage pipeline, each protocol feature channel has its own precision:

```rust
/// Precision estimate for a single prediction channel.
/// Tracks the inverse variance of recent prediction errors.
#[derive(Clone, Debug)]
pub struct PrecisionEstimate {
    /// Running mean of squared prediction errors
    mean_sq_error: f64,
    /// Running mean of prediction errors (for bias detection)
    mean_error: f64,
    /// Exponential decay factor for the running estimates
    decay: f64,
    /// Number of effective samples
    n_eff: f64,
}

impl PrecisionEstimate {
    pub fn new(decay: f64) -> Self {
        Self {
            mean_sq_error: 1.0, // Start with unit variance (uninformative)
            mean_error: 0.0,
            decay,
            n_eff: 0.0,
        }
    }

    /// Update with a new prediction error. Returns the current precision.
    pub fn update(&mut self, error: f64) -> f64 {
        self.mean_sq_error = self.decay * self.mean_sq_error + (1.0 - self.decay) * error * error;
        self.mean_error = self.decay * self.mean_error + (1.0 - self.decay) * error;
        self.n_eff = self.decay * self.n_eff + 1.0;
        self.precision()
    }

    /// Precision = inverse variance, clamped to avoid division by near-zero.
    pub fn precision(&self) -> f64 {
        let variance = (self.mean_sq_error - self.mean_error.powi(2)).max(1e-8);
        (1.0 / variance).min(1e6) // Clamp to prevent numerical explosion
    }

    /// Is this channel well-calibrated? High precision + low bias = yes.
    pub fn is_calibrated(&self) -> bool {
        self.n_eff > 10.0 && self.mean_error.abs() < 2.0 * self.mean_sq_error.sqrt()
    }
}
```

#### Precision and Curiosity Thresholds

The FEP predicts a counterintuitive relationship between model confidence and curiosity thresholds. Protocols where the golem has high-precision models should get *lower* curiosity thresholds, not higher. Why? Because a prediction error from a high-precision channel is more informative than one from a low-precision channel. If the golem's model of Uniswap V3 is tight (high precision), and a Uniswap V3 event violates that model, something genuinely unusual is happening. If the model of some obscure new protocol is loose (low precision), prediction errors are expected and carry less information.

```rust
/// Modulate curiosity threshold based on model precision.
/// High precision → lower threshold → more events escalated for that protocol.
/// Low precision → higher threshold → fewer events escalated (noise expected).
fn precision_modulated_threshold(
    base_threshold: f32,
    precision: f64,
    max_precision: f64,
) -> f32 {
    let normalized = (precision / max_precision).min(1.0) as f32;
    // High precision: threshold drops to 60% of base
    // Low precision: threshold stays at 100% of base
    base_threshold * (1.0 - 0.4 * normalized)
}
```

This reverses the naive intuition (well-known protocols are boring, unknown ones are interesting) with a principled alternative: well-modeled protocols whose behavior deviates are the most interesting signals.

### Expected Free Energy for Action Selection

#### The Planning-as-Inference Framework

Active inference treats planning as inference. Instead of optimizing a reward function (reinforcement learning) or following rules (expert systems), the agent scores candidate actions by their **Expected Free Energy (EFE)**:

```
G(a) = E_q[ D_KL( q(o|a) || p(o) ) ] + E_q[ D_KL( q(s|o,a) || q(s|a) ) ]
     = pragmatic_value              + epistemic_value
     = (expected reward)             + (expected information gain)
```

The first term (pragmatic value) favors actions that lead to observations the agent prefers. For the golem: actions that lead to profitable positions, lower risk exposure, timely rebalancing.

The second term (epistemic value) favors actions that resolve uncertainty. For the golem: probe swaps that reveal market depth, monitoring positions to detect out-of-range conditions, investigating unknown protocols.

The balance between these terms is not tuned by a hyperparameter. It falls out of the math: when the agent is uncertain, epistemic value dominates and the agent explores. When the agent is confident, pragmatic value dominates and the agent exploits. This is the exploration-exploitation tradeoff solved without epsilon-greedy hacks or UCB bounds.

#### EFE for Triage Routing

At Theta tick, the golem decides how to handle a set of high-scoring triage events. Each "action" is a routing decision: escalate to LLM analysis, update state silently, or discard. EFE scores these:

```rust
/// Expected Free Energy for a triage routing action.
pub struct EfeScore {
    /// Pragmatic: how much does this action align with the golem's goals?
    /// Positive for actions that protect positions, capture opportunities.
    pub pragmatic: f64,
    /// Epistemic: how much uncertainty does this action resolve?
    /// High for events from poorly modeled protocols with high-precision violations.
    pub epistemic: f64,
}

impl EfeScore {
    /// Combined EFE. Lower is better (minimizing free energy).
    /// Negate because we want to SELECT actions with highest value,
    /// but EFE is a quantity to minimize.
    pub fn combined(&self) -> f64 {
        -(self.pragmatic + self.epistemic)
    }
}

/// Score a routing decision using expected free energy.
fn score_routing_action(
    event: &TriageEvent,
    action: RoutingAction,
    belief_state: &BeliefState,
    golem_preferences: &GolemPreferences,
) -> EfeScore {
    match action {
        RoutingAction::EscalateToLlm => {
            let epistemic = belief_state.information_gain_from_analysis(event);
            let pragmatic = golem_preferences.relevance_to_positions(event);
            EfeScore { pragmatic, epistemic }
        }
        RoutingAction::UpdateStateSilently => {
            // Low epistemic value (no LLM analysis = less learning)
            // but non-zero pragmatic value (state stays current)
            let epistemic = 0.1 * belief_state.information_gain_from_analysis(event);
            let pragmatic = 0.3 * golem_preferences.relevance_to_positions(event);
            EfeScore { pragmatic, epistemic }
        }
        RoutingAction::Discard => {
            EfeScore {
                pragmatic: 0.0,
                epistemic: 0.0,
            }
        }
    }
}
```

### The Belief State

#### Maintaining a Generative Model of Chain Activity

The golem's belief state is the collection of all its models about on-chain activity:

```rust
/// The golem's beliefs about the state of its environment.
/// Updated at each tick via Bayesian inference.
pub struct BeliefState {
    /// Per-protocol Bayesian surprise models (from BayesianSurpriseDomain)
    pub surprise_models: BayesianSurpriseScorer,

    /// Per-channel precision estimates
    pub precisions: DashMap<(ProtocolId, String), PrecisionEstimate>,

    /// Protocol state predictions for the next observation
    pub predictions: DashMap<ProtocolId, StatePrediction>,

    /// Global regime indicator (stationary vs. changepoint detected)
    pub regime: AtomicU8, // 0 = stationary, 1 = transition, 2 = new regime
}

/// Prediction about what the next observation from a protocol should look like.
#[derive(Clone, Debug)]
pub struct StatePrediction {
    pub expected_event_rate: f64,
    pub expected_gas_range: (f64, f64),
    pub expected_value_range: (f64, f64),
    pub confidence: f64, // derived from precision estimates
}

impl BeliefState {
    /// Process a new observation: update beliefs, compute surprise,
    /// update precisions, generate next prediction.
    pub fn process_observation(
        &self,
        protocol: &ProtocolId,
        features: &TxFeatures,
    ) -> ObservationResult {
        // 1. Compute surprise (updates models internally)
        let surprise = self.surprise_models.score(protocol, features);

        // 2. Compute prediction error against current predictions
        let prediction_error = if let Some(pred) = self.predictions.get(protocol) {
            features.deviation_from(&pred)
        } else {
            1.0 // No prediction = maximum uncertainty
        };

        // 3. Update precision for each feature channel
        let precision = self.update_precisions(protocol, features, prediction_error);

        // 4. Precision-weighted prediction error
        let weighted_error = precision * prediction_error;

        // 5. Generate next prediction (simple EMA-based)
        self.update_predictions(protocol, features);

        ObservationResult {
            surprise,
            prediction_error,
            precision_weighted_error: weighted_error,
            regime_change_detected: weighted_error > REGIME_THRESHOLD,
        }
    }

    /// Estimate information gain from LLM analysis of an event.
    pub fn information_gain_from_analysis(&self, event: &TriageEvent) -> f64 {
        // Events from low-precision protocols offer more information gain
        let avg_precision = self.average_precision(&event.protocol_id);
        let uncertainty = 1.0 / (avg_precision + 1.0);

        // Events with high surprise offer more information gain
        let surprise = event.curiosity_score as f64;

        // Information gain is approximately surprise * uncertainty
        surprise * uncertainty
    }

    fn update_precisions(
        &self,
        protocol: &ProtocolId,
        features: &TxFeatures,
        error: f64,
    ) -> f64 {
        let key = (protocol.clone(), "aggregate".to_string());
        let mut entry = self.precisions
            .entry(key)
            .or_insert_with(|| PrecisionEstimate::new(0.95));
        entry.update(error)
    }

    fn average_precision(&self, protocol: &ProtocolId) -> f64 {
        let key = (protocol.clone(), "aggregate".to_string());
        self.precisions
            .get(&key)
            .map(|p| p.precision())
            .unwrap_or(1.0)
    }

    fn update_predictions(&self, protocol: &ProtocolId, features: &TxFeatures) {
        // Simple exponential moving average prediction
        let alpha = 0.1;
        let mut pred = self.predictions
            .entry(protocol.clone())
            .or_insert_with(|| StatePrediction::from_features(features));

        pred.expected_event_rate = pred.expected_event_rate * (1.0 - alpha)
            + features.event_rate * alpha;
        pred.expected_gas_range = (
            pred.expected_gas_range.0 * (1.0 - alpha) + features.gas_price * alpha,
            pred.expected_gas_range.1.max(features.gas_price),
        );
        pred.confidence = self.average_precision(protocol).min(1.0);
    }
}
```

### The Active Inference Agent

```rust
/// Active inference agent wrapping the belief state and action selection.
pub struct ActiveInferenceAgent {
    pub beliefs: BeliefState,
    pub preferences: GolemPreferences,
}

impl ActiveInferenceAgent {
    /// Select the best routing action for a triage event.
    /// Returns the action with the lowest expected free energy.
    pub fn select_routing(&self, event: &TriageEvent) -> RoutingAction {
        let actions = [
            RoutingAction::EscalateToLlm,
            RoutingAction::UpdateStateSilently,
            RoutingAction::Discard,
        ];

        actions
            .iter()
            .map(|action| {
                let efe = score_routing_action(
                    event,
                    *action,
                    &self.beliefs,
                    &self.preferences,
                );
                (*action, efe.combined())
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(action, _)| action)
            .unwrap_or(RoutingAction::Discard)
    }

    /// Process an observation and update the agent's beliefs.
    /// Called at each Gamma tick for every transaction that passes
    /// address triage (Stage 2).
    pub fn observe(&self, protocol: &ProtocolId, features: &TxFeatures) -> ObservationResult {
        self.beliefs.process_observation(protocol, features)
    }

    /// Gamma tick maintenance: decay models, update regime detection.
    pub fn gamma_tick(&self) {
        self.beliefs.surprise_models.decay_gamma();
    }

    /// Theta tick maintenance: decay categorical models,
    /// check for regime changes across protocols.
    pub fn theta_tick(&self) {
        self.beliefs.surprise_models.decay_theta();
    }
}
```

### Behavioral Phases as Shifted Priors

The golem's behavioral phases (thriving, declining, terminal) map naturally to the FEP as shifted priors over preferred observations. In the thriving phase, the golem "expects" (prefers) observations consistent with capital growth: profitable trade opportunities, healthy position states. In the terminal phase, the golem "expects" observations consistent with capital preservation: stable positions, low-risk states.

This shift in priors changes the pragmatic value term of EFE without touching the epistemic term. A terminal golem still seeks information (epistemic value is unchanged), but it no longer seeks profit opportunities (pragmatic value shifts toward conservation). The math produces the right behavior: a dying golem explores its environment to generate knowledge for its successor (high epistemic value) while avoiding risky trades (low pragmatic value for profit-seeking actions).

### Connection to the Triage Pipeline

Active inference doesn't replace the existing triage pipeline. It provides a theoretical grounding for decisions that the pipeline already makes heuristically:

| Triage decision | Current implementation | Active inference interpretation |
|---|---|---|
| Bloom filter pre-screen | Fixed bloom membership | Prior-based gating: events outside the generative model are pre-filtered |
| Address triage | DashSet membership check | Attention allocation: only attend to entities within the generative model |
| Curiosity scoring | Heuristic + ANN similarity | Free energy: surprise (Bayesian) + prediction error (precision-weighted) |
| Score routing thresholds | Fixed brackets (>0.8, 0.5-0.8, etc.) | EFE-based action selection: route to minimize expected free energy |
| Heuristic-to-learned weight shift | Linear interpolation by episode count | Precision weighting: shift from prior-dominated to likelihood-dominated as data accumulates |

The value of this mapping is that it identifies missing capabilities: precision weighting isn't implemented, EFE-based routing isn't implemented, and the exploration-exploitation balance is currently managed by hand-tuned interpolation rather than falling out of the math.

### Active Inference References

- Friston, K. (2010). "The Free-Energy Principle: A Unified Brain Theory?" *Nature Reviews Neuroscience*, 11(2), 127-138. — *Proposes that all adaptive systems minimize variational free energy; the theoretical umbrella under which Golem's perception-action cycle operates.*
- Parr, T. & Friston, K.J. (2019). "Generalised Free Energy and Active Inference." *Biological Cybernetics*, 113(5), 495-513. — *Extends free energy to include expected free energy for planning; formalizes the explore-exploit balance that governs context assembly priorities.*
- Da Costa, L., Parr, T., Sajid, N., Veselic, S., Neacsu, V. & Friston, K. (2020). "Active Inference on Discrete State-Spaces: A Synthesis." *Journal of Mathematical Psychology*, 99, 102447. — *Provides a tractable discrete-state implementation of active inference; the computational template for Golem's finite-state triage decisions.*
- Parr, T., Pezzulo, G. & Friston, K. (2022). *Active Inference: The Free Energy Principle in Mind, Brain, and Behavior.* MIT Press. — *The definitive textbook treatment of active inference, covering perception, action, and learning under one variational objective; primary reference for the context governor's theoretical grounding.*
- Feldman, H. & Friston, K. (2010). "Attention, Uncertainty, and Free-Energy." *Frontiers in Human Neuroscience*, 4, 215. — *Models attention as precision optimization on prediction errors; directly justifies the context governor's precision-weighted slot allocation.*
- Friston, K., FitzGerald, T., Rigoli, F., Schwartenbeck, P. & Pezzulo, G. (2017). "Active Inference: A Process Theory." *Neural Computation*, 29(1), 1-49. — *Specifies the computational process theory for active inference with belief propagation; the algorithmic blueprint behind the four-component value scoring system.*

---

## References

- Anthropic (2025). "Context Engineering for Agents." anthropic.com. — *Introduces the concept of context engineering as a first-class design discipline for LLM agents; the direct motivation for treating context assembly as a learnable control problem.*
- Zhang, A. et al. (2025). "ACE: Agentic Context Engineering." arXiv:2510.04618. — *Proposes an architecture where agents actively manage their own context windows; validates the context governor's approach of autonomous slot allocation.*
- Samsung Research (2025). "Context State Object Architecture." arXiv:2511.03728. — *Introduces structured context state objects for multi-turn agent interactions; informs the governor's typed context block design.*
- Cohen-Wang, B., Shah, H., Georgiev, B., & Madry, A. (2024). "ContextCite: Attributing Model Generation to Context." MIT CSAIL, arXiv:2409.00729. — *Provides methods to attribute which context tokens influenced specific outputs; enables the governor's cybernetic feedback loop measuring context block utility.*
- Kang, S. et al. (2025). "ACON: Agentic Context Compression." Microsoft Research, arXiv:2510.00615. — *Demonstrates agent-driven prompt compression that preserves task-relevant information; supports the governor's delta compression strategy.*
- Lindenbauer, M. et al. (2025). "Simple Observation Masking Halves Cost While Matching LLM Summarization." arXiv:2508.21433, NeurIPS 2025 DL4C Workshop. — *Shows that selectively masking observations can halve token cost without degrading performance; evidence for the governor's observation pruning approach.*
- "Is Agentic RAG Worth It?" (2026). arXiv:2601.07711. — *Evaluates when retrieval-augmented generation pays for itself in agentic settings; informs the governor's decision on when to inject Grimoire episodes vs. relying on prior knowledge.*
- Wang, Z. et al. (2025). "TracLLM: Context Attribution for Long Contexts." USENIX Security 2025. — *Traces which context segments are actually used during generation; provides the attribution signal the governor needs for its feedback loop.*
- Jiang, H. et al. (2023). "LLMLingua: Compressing Prompts for Accelerated Inference." EMNLP 2023. — *Introduces iterative prompt compression using a small LM to identify dispensable tokens; one of the compression techniques the governor can deploy on context blocks.*
- Pan, Z. et al. (2024). "LLMLingua-2: Data Distillation for Faithful Prompt Compression." ACL 2024. — *Improves on LLMLingua with faithful compression that preserves factual content; ensures the governor's compression doesn't distort critical on-chain data.*
- Khattab, O. et al. (2024). "DSPy: Compiling Declarative Language Model Calls into Self-Improving Pipelines." ICLR 2024. — *Demonstrates declarative prompt programs that self-optimize; the governor's learnable context assembly follows a similar compile-then-improve pattern.*
- Shinn, N. et al. (2023). "Reflexion: Language Agents with Verbal Reinforcement Learning." NeurIPS 2023. — *Shows agents can improve through self-reflective verbal feedback; the governor's after_turn reflection loop implements this pattern.*
- Zhao, A. et al. (2024). "ExpeL: LLM Agents Are Experiential Learners." AAAI 2024. — *Demonstrates agents extracting reusable insights from past episodes; parallel to how the governor uses Grimoire episode replay to improve future context assembly.*
- Bateson, G. (1972). *Steps to an Ecology of Mind*. Chandler. — *Introduces deutero-learning (learning to learn) and logical types of learning; the theoretical basis for the governor's meta-learning feedback loops.*
- Argyris, C. & Schon, D.A. (1978). *Organizational Learning*. Addison-Wesley. — *Defines single-loop and double-loop learning in organizations; the governor implements both: tuning parameters (single-loop) and revising its own assembly strategy (double-loop).*
- Chen, L. et al. (2024). "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." *TMLR*. — *Cascading LLM calls from cheap to expensive based on confidence; informs the governor's tier-aware context budgeting across T0/T1/T2.*
- [LIU-2024] Liu, N.F. et al. (2024). "Lost in the Middle: How Language Models Use Long Contexts." TACL. — *Shows LLMs attend poorly to middle-positioned context; the governor places high-value blocks at the start and end of the assembled prompt.*
- [BADDELEY-2000] Baddeley, A. "The Episodic Buffer: A New Component of Working Memory?" *Trends in Cognitive Sciences*, 4(11), 2000. — *Proposes a limited-capacity buffer integrating multi-modal information; the neuroscience model for the governor's fixed-budget context window.*
- [COGNITIVE-WORKSPACE-2025] "Cognitive Workspace: Active Memory Management for LLMs." arXiv:2508.13171, 2025. — *Introduces explicit workspace management for LLM memory; directly supports the governor's design as an active memory manager rather than a passive prompt concatenator.*
- [FRISTON-2017] Friston, K. et al. "Active Inference: A Process Theory." *Neural Computation*, 29(1), 2017. — *Specifies the computational process theory for active inference; provides the expected free energy objective the governor uses for value scoring.*
- [PARR-2019] Parr, T. & Friston, K.J. "Generalised Free Energy and Active Inference." *Biological Cybernetics*, 113, 2019. — *Extends free energy to planning horizons; formalizes how the governor should weight epistemic vs. pragmatic value in slot allocation.*
- [VICKREY-1961] Vickrey, W. "Counterspeculation, Auctions, and Competitive Sealed Tenders." *Journal of Finance*, 16(1), 1961. — *Introduces the second-price sealed-bid auction; the mechanism design foundation for the Phase 2 VCG attention auction.*
- [NISAN-2007] Nisan, N. et al. *Algorithmic Game Theory*. Cambridge University Press, 2007. — *Comprehensive treatment of mechanism design, VCG auctions, and incentive compatibility; the textbook reference for the attention auction's truthfulness guarantees.*
