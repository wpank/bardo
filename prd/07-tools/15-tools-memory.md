# Bardo Tools -- Memory and Self-Improvement [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> The Grimoire memory system and Pi lifecycle hooks for self-improvement. Category: `memory`. 13 tools (7 write, 6 read).

---

> **Reader orientation:** This document specifies 13 memory and self-improvement tools in `bardo-tools`. These tools read and write to the Grimoire, a three-substrate memory system (LanceDB for episodic memory, SQLite for semantic beliefs, filesystem for strategy). The Reflexion and ExpeL pipelines turn raw experience into distilled knowledge. You should be comfortable with vector similarity search and basic reinforcement-from-experience concepts. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Grimoire architecture

The Grimoire is a three-substrate memory system. Each substrate handles a different kind of knowledge, and they work together through the Reflexion and ExpeL pipelines.

**Substrate 1: LanceDB (episodic memory).** Stores experience episodes as 384-dimensional vectors. Search uses BM25 + vector similarity with Reciprocal Rank Fusion (RRF) reranking. Episodes capture what happened, what the Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) expected, and what it learned. Retention follows Ebbinghaus decay: R = e^(-t/S), where S scales with the episode's importance tier.

**Substrate 2: SQLite with sqlite-vec (semantic memory).** Stores distilled insights as structured records with KNN-searchable embeddings. Insights are the Grimoire's "beliefs" -- they start at 0.6 confidence and drift up or down based on validation. Asymmetric update: upvotes add +0.1 (cap 1.0), downvotes subtract -0.15 (bad beliefs decay faster).

**Substrate 3: Filesystem (strategy memory).** Stores strategy configurations and performance histories as JSON files. No vector search needed -- strategies are retrieved by name and filtered by performance.

The three substrates are read/written through memory tools. The Grimoire is optional -- active only when the `learning` profile is on and `BARDO_MEMORY_ENABLED=true`.

> Cross-reference: `../01-golem/09-grimoire.md` (the full Grimoire architecture: substrates, retention policies, Reflexion/ExpeL pipelines, and cross-Golem memory inheritance) for the full Grimoire architecture. `../01-golem/10-dream.md` (the Dream cycle: offline memory consolidation, belief pruning, and strategy refinement during idle periods) for Dream cycle integration.

---

## Episodic memory (2 tools)

### `memory_store_episode`

Store an episodic memory entry from a tool execution. Each episode captures the action taken, its outcome, and a natural-language self-reflection (Reflexion pattern). Importance determines retention via Ebbinghaus decay.

```rust
#[derive(Debug, Deserialize)]
pub struct StoreEpisodeParams {
    /// Tool that was executed (e.g., "uniswap_execute_swap", "vault_deposit").
    pub tool: String,
    /// Execution outcome data as JSON.
    pub outcome: serde_json::Value,
    /// Natural-language self-reflection on the outcome. The Reflexion pattern
    /// requires the Golem to articulate what it expected, what happened, and
    /// what it would do differently.
    pub reflection: String,
    /// Chain where execution occurred.
    pub chain: String,
    /// Token pair context (e.g., "WETH/USDC").
    pub token_pair: Option<String>,
    /// Retention tier. Maps to Ebbinghaus S parameter:
    /// routine (S=7d), notable (S=30d), critical (S=90d), emergency (S=180d).
    #[serde(default = "default_importance_routine")]
    pub importance: EpisodeImportance,
    /// Dream cycle tag. If set, this episode is flagged for processing
    /// during the next Dream cycle (offline consolidation).
    pub dream_tag: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EpisodeImportance {
    /// S = 7 days. Normal operations.
    Routine,
    /// S = 30 days. Large trades, unusual outcomes.
    Notable,
    /// S = 90 days. Strategy changes, significant losses.
    Critical,
    /// S = 180 days. Exploit detection, emergency actions.
    Emergency,
}

#[derive(Debug, Serialize)]
pub struct StoreEpisodeResult {
    pub episode_id: String,
    pub importance: EpisodeImportance,
    /// Estimated retention: when this episode will decay below retrieval threshold.
    pub estimated_retention_until: String,
    pub vector_dimension: u32,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_store_episode",
    description: concat!(
        "Store an episodic memory entry from a tool execution. ",
        "Captures action, outcome, and Reflexion self-reflection. ",
        "Importance tier determines Ebbinghaus retention: routine (7d), notable (30d), critical (90d), emergency (180d). ",
        "Write to LanceDB. No on-chain cost.",
    ),
    category: Category::Memory,
    capability: CapabilityTier::Write,       // Modifies Grimoire state
    risk_tier: RiskTier::Layer1,             // Local write only
    tick_budget: TickBudget::Fast,
    progress_steps: &["Embedding episode", "Writing to LanceDB"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Store episodic memory with Reflexion self-reflection. Importance sets Ebbinghaus retention.",
    prompt_guidelines: &[
        "thriving: Store after every significant operation. routine=normal, notable=large trades, critical=strategy changes, emergency=exploit detection.",
        "cautious: Store everything at notable or higher. Cautious phases generate high-value learning data.",
        "declining: Store at critical. Decline episodes teach the most.",
        "terminal: Store at emergency for maximum retention in successor's Grimoire.",
    ],
};
```

**Event Fabric:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ tool_name: "memory_store_episode", params_hash, tick }` |
| `tool:end` | `{ success, duration_ms, result_summary: "Stored {importance} episode {id}" }` |
| `grimoire:episode_stored` | `{ episode_id, importance, tool, chain }` |

**Ground truth:** LanceDB row count increments. Expected: count_before + 1. Actual: count_after. Source: `LanceDB::count_rows()`.

### `memory_search`

Hybrid search across episodic and semantic memory. Episodes use BM25 + vector similarity with RRF. Insights use structured queries + KNN via sqlite-vec. Search applies Ebbinghaus decay as a relevance penalty: older episodes with low importance score lower.

```rust
#[derive(Debug, Deserialize)]
pub struct MemorySearchParams {
    /// Natural language search query.
    pub query: String,
    /// "episodes", "insights", "both" (default: "both").
    #[serde(default = "default_store_both")]
    pub store: String,
    /// Chain filter.
    pub chain: Option<String>,
    /// Token pair filter.
    pub token_pair: Option<String>,
    /// Tool name filter.
    pub tool: Option<String>,
    /// Minimum insight confidence, 0.0-1.0 (default: 0.0).
    #[serde(default)]
    pub min_confidence: f64,
    /// Max results (default: 10, max: 50).
    #[serde(default = "default_limit_10")]
    pub limit: u32,
    /// Apply Ebbinghaus decay penalty to episode relevance (default: true).
    /// When true, old routine episodes rank lower than recent ones even if
    /// vector similarity is high.
    #[serde(default = "default_true")]
    pub apply_decay: bool,
}

#[derive(Debug, Serialize)]
pub struct MemorySearchResult {
    pub results: Vec<MemoryHit>,
    pub total_episodes_searched: u32,
    pub total_insights_searched: u32,
}

#[derive(Debug, Serialize)]
pub struct MemoryHit {
    pub source: String,              // "episode" or "insight"
    pub id: String,
    pub content: String,
    pub relevance_score: f64,
    /// For episodes: decay-adjusted score. For insights: confidence.
    pub adjusted_score: f64,
    pub metadata: serde_json::Value,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_search",
    description: concat!(
        "Hybrid BM25 + vector search across episodes and insights. ",
        "Ebbinghaus decay adjusts episode relevance by age and importance. ",
        "Use before any decision to retrieve relevant past experiences.",
    ),
    category: Category::Memory,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Embedding query", "Searching episodes", "Searching insights", "Ranking results"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Hybrid BM25 + vector search across Grimoire. Ebbinghaus decay adjusts relevance. Search before every decision.",
    prompt_guidelines: &[
        "thriving: Search before every decision. Weight high-confidence insights (> 0.8) heavily.",
        "cautious: Search with broader queries. Include failure episodes in context.",
        "declining: Search for capital preservation insights and similar decline patterns.",
        "terminal: Search for death-related insights from any imported predecessor Grimoire.",
    ],
};
```

---

## Semantic memory (3 tools)

### `memory_get_insights`

Query semantic insights by category, confidence, and recency.

```rust
#[derive(Debug, Deserialize)]
pub struct GetInsightsParams {
    /// Filter by category. See memory_manage_insight for categories.
    pub category: Option<String>,
    /// Minimum confidence (default: 0.0).
    #[serde(default)]
    pub min_confidence: f64,
    /// Chain filter.
    pub chain: Option<String>,
    /// Token pair filter.
    pub token_pair: Option<String>,
    /// Sort by: "confidence", "created", "updated" (default: "confidence").
    #[serde(default = "default_sort_confidence")]
    pub sort_by: String,
    /// Max results (default: 20).
    #[serde(default = "default_limit_20")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct InsightsResult {
    pub insights: Vec<Insight>,
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct Insight {
    pub id: String,
    pub content: String,
    pub category: String,
    pub confidence: f64,
    pub chain: Option<String>,
    pub token_pair: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub upvotes: u32,
    pub downvotes: u32,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_get_insights",
    description: "Query semantic insights by category, confidence, recency. Apply learned knowledge to current decisions.",
    category: Category::Memory,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Querying SQLite"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Query semantic insights by category and confidence. Apply learned knowledge.",
    prompt_guidelines: &[
        "thriving: Query before decisions in specific domains (e.g., slippage insights before large swaps).",
        "cautious: Query broadly. Weight high-confidence insights heavily.",
        "declining: Query for capital preservation insights.",
        "terminal: Query for successor-relevant insights to include in death testament.",
    ],
};
```

### `memory_manage_insight`

Create, upvote, downvote, or edit semantic insights. Confidence updates are asymmetric for safety.

```rust
#[derive(Debug, Deserialize)]
pub struct ManageInsightParams {
    /// "ADD", "UPVOTE", "DOWNVOTE", "EDIT".
    pub operation: InsightOperation,
    /// Required for UPVOTE/DOWNVOTE/EDIT.
    pub insight_id: Option<String>,
    /// Required for ADD/EDIT.
    pub content: Option<String>,
    /// Required for ADD. One of: slippage, gas_timing, route_selection,
    /// pool_behavior, mev_pattern, liquidity_depth, volatility,
    /// fee_optimization, rebalance_timing, vault_strategy, emergency, general.
    pub category: Option<String>,
    pub chain: Option<String>,
    pub token_pair: Option<String>,
    /// Audit trail.
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum InsightOperation { Add, Upvote, Downvote, Edit }

#[derive(Debug, Serialize)]
pub struct ManageInsightResult {
    pub insight_id: String,
    pub operation: String,
    pub confidence_before: f64,
    pub confidence_after: f64,
}
```

Confidence mechanics:
- New insights start at 0.6.
- Upvote: +0.1 (capped at 1.0).
- Downvote: -0.15 (floored at 0.0).

The asymmetry is deliberate. Bad insights should decay faster than good ones accumulate. A single bad outcome should weigh more than a single good one because the downside of acting on wrong knowledge exceeds the upside of confirming correct knowledge.

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_manage_insight",
    description: concat!(
        "Add, upvote, downvote, or edit semantic insights. ",
        "Upvote: +0.1 confidence (cap 1.0). Downvote: -0.15 (faster decay for bad insights). ",
        "New insights start at 0.6. Write to SQLite.",
    ),
    category: Category::Memory,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Updating insight store"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Manage semantic insights: ADD/UPVOTE/DOWNVOTE/EDIT. Asymmetric confidence updates.",
    prompt_guidelines: &[
        "thriving: Add insights from successful patterns. Upvote confirmed. Downvote invalidated.",
        "cautious: Downvote insights that led to losses. Add caution-specific insights.",
        "declining: Downvote strategies that contributed to decline. Add lessons learned.",
        "terminal: Add final insights for successor. Capture the Golem's most valuable learnings.",
    ],
};
```

**Ground truth:** SQLite row count (for ADD) or confidence value (for UPVOTE/DOWNVOTE). Source: SQLite query post-operation.

### `memory_consolidate`

Run ExpeL distillation: retrieve unconsolidated episodes, cluster by tool + chain + token pair, extract insight operations via LLM, apply to the semantic store, then run an Ebbinghaus decay pass over expired episodes. This is the pipeline that turns raw experience into structured knowledge.

The Dream cycle calls this tool automatically every 4 hours during the Golem's sleep phase. Manual invocation is useful after activity bursts where many episodes accumulated without consolidation.

```rust
#[derive(Debug, Deserialize)]
pub struct ConsolidateParams {
    /// Maximum episodes to consolidate in this pass (default: 100).
    #[serde(default = "default_100")]
    pub max_episodes: u32,
    /// Minimum cluster size for insight extraction (default: 3).
    #[serde(default = "default_3")]
    pub min_cluster_size: u32,
    /// Run Ebbinghaus decay pass on expired episodes (default: true).
    #[serde(default = "default_true")]
    pub run_decay: bool,
    /// Dry run: compute clusters and proposed insights but don't write (default: false).
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct ConsolidateResult {
    pub episodes_processed: u32,
    pub clusters_found: u32,
    pub insights_created: u32,
    pub insights_reinforced: u32,
    pub episodes_decayed: u32,
    pub duration_ms: u64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_consolidate",
    description: concat!(
        "ExpeL distillation: cluster episodes, extract insights via LLM, run Ebbinghaus decay. ",
        "Automatic every 4h during Dream cycle. Manual trigger after activity bursts.",
    ),
    category: Category::Memory,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Slow,         // 5-15s, involves LLM calls
    progress_steps: &["Loading unconsolidated episodes", "Clustering", "Extracting insights", "Running decay pass"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "ExpeL distillation: cluster episodes, extract insights, run Ebbinghaus decay. Automatic every 4h. Manual after bursts.",
    prompt_guidelines: &[
        "thriving: Let automatic schedule run. Trigger manually after bursts.",
        "cautious: Same.",
        "declining: Run once to capture remaining learnings.",
        "terminal: Final consolidation before exporting Grimoire for successor.",
    ],
};
```

**Event Fabric:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ tool_name: "memory_consolidate", params_hash, tick }` |
| `tool:update` | Steps through clustering, extraction, decay |
| `tool:end` | `{ success, duration_ms, result_summary: "{n} insights created, {m} episodes decayed" }` |
| `grimoire:consolidation_complete` | `{ episodes_processed, insights_created, episodes_decayed }` |
| `dream:consolidation_hook` | Emitted when called during Dream cycle. Dream monitor uses this. |

---

## Strategy memory (2 tools)

### `memory_store_strategy`

Store a strategy configuration for future reference, comparison, and parameter tuning.

```rust
#[derive(Debug, Deserialize)]
pub struct StoreStrategyParams {
    pub strategy_name: String,
    /// Strategy parameters as JSON.
    pub parameters: serde_json::Value,
    /// Description of the strategy's intent.
    pub description: String,
    /// Associated chains.
    pub chains: Vec<String>,
    /// Associated token pairs.
    pub token_pairs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct StoreStrategyResult {
    pub strategy_id: String,
    pub strategy_name: String,
    pub version: u32,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_store_strategy",
    description: "Store a strategy configuration for future reference, comparison, and parameter tuning. Write to filesystem.",
    category: Category::Memory,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Writing strategy file"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Store strategy configuration. Filesystem write.",
    prompt_guidelines: &[
        "thriving: Store new strategies and version updates.",
        "cautious: Store with detailed parameters for post-mortem analysis.",
        "declining: Store current strategy state for forensics.",
        "terminal: Store final strategy state for successor.",
    ],
};
```

### `memory_retrieve_strategies`

Retrieve stored strategy configurations and their performance histories.

```rust
#[derive(Debug, Deserialize)]
pub struct RetrieveStrategiesParams {
    /// Filter by name (partial match).
    pub strategy_name: Option<String>,
    /// Filter by chain.
    pub chain: Option<String>,
    /// Filter by token pair.
    pub token_pair: Option<String>,
    /// Include performance history (default: true).
    #[serde(default = "default_true")]
    pub include_performance: bool,
}

#[derive(Debug, Serialize)]
pub struct RetrieveStrategiesResult {
    pub strategies: Vec<StoredStrategy>,
}

#[derive(Debug, Serialize)]
pub struct StoredStrategy {
    pub strategy_id: String,
    pub strategy_name: String,
    pub version: u32,
    pub parameters: serde_json::Value,
    pub description: String,
    pub created_at: String,
    pub performance: Option<StrategyPerformance>,
}

#[derive(Debug, Serialize)]
pub struct StrategyPerformance {
    pub total_executions: u32,
    pub avg_pnl_usd: f64,
    pub win_rate: f64,
    pub sharpe_ratio: Option<f64>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_retrieve_strategies",
    description: "Retrieve stored strategy configurations and performance history. Evaluate past effectiveness.",
    category: Category::Memory,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Loading strategies", "Computing performance"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Retrieve strategies with performance history. Evaluate past effectiveness.",
    prompt_guidelines: &[
        "thriving: Review before modifying strategy parameters.",
        "cautious: Compare current strategy against historical performance.",
        "declining: Analyze which strategies worked and which failed.",
        "terminal: Not relevant.",
    ],
};
```

---

## Self-improvement: Pi lifecycle (4 tools)

The self-improvement loop follows a five-stage lifecycle: RECORD -> REFLECT -> ANALYZE -> CONSOLIDATE -> TUNE. These tools integrate with the Pi runtime as lifecycle hooks.

### `memory_record_execution`

Record a strategy execution with full context for subsequent predicted-vs-actual analysis. Captures portfolio state, market conditions, decision parameters, and predicted outcome before execution settles.

This is the Pi `on_mount` hook -- called when a strategy begins execution.

```rust
#[derive(Debug, Deserialize)]
pub struct RecordExecutionParams {
    /// Unique execution identifier.
    pub execution_id: String,
    pub strategy_name: String,
    /// SWAP, ADD_LIQUIDITY, REMOVE_LIQUIDITY, REBALANCE, DEPOSIT, WITHDRAW.
    pub action_type: String,
    /// Portfolio state at execution time (JSON).
    pub portfolio_snapshot: serde_json::Value,
    /// Market conditions: prices, volatility, TVL, tick, fee, gas (JSON).
    pub market_conditions: serde_json::Value,
    /// Parameters driving the decision (JSON).
    pub decision_params: serde_json::Value,
    /// Expected return, fees, IL, confidence interval (JSON).
    pub predicted_outcome: serde_json::Value,
    /// Natural-language reasoning trace.
    pub reasoning_chain: Option<String>,
    /// Transaction hash (if already submitted).
    pub tx_hash: Option<String>,
    /// Reflexion-pattern self-reflection.
    pub reflection: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecordExecutionResult {
    pub execution_id: String,
    pub recorded_at: String,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_record_execution",
    description: concat!(
        "Record strategy execution with portfolio snapshot and market conditions before outcome settles. ",
        "Pi on_mount hook. Required for predicted-vs-actual analysis. ",
        "Write to local storage. No on-chain cost.",
    ),
    category: Category::Memory,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Recording execution context"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Record execution context before settlement. Pi on_mount hook. Required for predicted-vs-actual.",
    prompt_guidelines: &[
        "thriving: Record every execution. Submit reflections for outcomes deviating > 20% from prediction.",
        "cautious: Record everything. Reflect on all negative outcomes.",
        "declining: Record outcomes. Reflect on what caused the decline.",
        "terminal: Submit final reflections on overall strategy. This feeds the death testament.",
    ],
};
```

### `memory_record_outcome`

Record the realized outcome of a previously recorded execution. Links to the `execution_id` from `memory_record_execution`.

This is the Pi `on_unmount` hook -- called when a strategy execution settles.

```rust
#[derive(Debug, Deserialize)]
pub struct RecordOutcomeParams {
    /// From memory_record_execution.
    pub execution_id: String,
    /// Realized P&L in USD.
    pub realized_pnl: f64,
    pub fees_earned: Option<f64>,
    pub impermanent_loss: Option<f64>,
    pub unrealized_pnl: Option<f64>,
    /// Gas cost in USD.
    pub gas_used_usd: f64,
    /// Actual slippage in bps.
    pub slippage_actual: Option<f64>,
    /// Post-settlement self-reflection.
    pub reflection: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecordOutcomeResult {
    pub execution_id: String,
    pub prediction_error: f64,
    pub recorded_at: String,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_record_outcome",
    description: "Record realized outcome of a previously recorded execution. Pi on_unmount hook. Links to execution_id. Write to local storage.",
    category: Category::Memory,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Recording outcome", "Computing prediction error"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Record realized outcome. Pi on_unmount hook. Links to execution_id.",
    prompt_guidelines: &[
        "thriving: Record after every execution settles.",
        "cautious: Same. Include reflection for negative outcomes.",
        "declining: Same.",
        "terminal: Record final outcomes.",
    ],
};
```

### `memory_compare_prediction`

Analyze prediction accuracy across recorded executions. Feeds the Pi Reflexion loop.

```rust
#[derive(Debug, Deserialize)]
pub struct ComparePredictionParams {
    /// Filter by strategy.
    pub strategy_name: Option<String>,
    /// "7d", "30d", "90d" (default: "30d").
    #[serde(default = "default_period_30d")]
    pub period: String,
}

#[derive(Debug, Serialize)]
pub struct PredictionAnalysis {
    pub prediction_error: ErrorStats,
    /// How well the Golem's confidence intervals match actual outcomes.
    pub calibration_accuracy: f64,
    /// Whether prediction accuracy is improving or degrading over time.
    pub drift_score: f64,
    pub drift_trend: String,         // "improving", "stable", "degrading"
    pub overconfidence_detected: bool,
    pub sample_count: u32,
}

#[derive(Debug, Serialize)]
pub struct ErrorStats {
    pub mean: f64,
    pub stddev: f64,
    pub median: f64,
    pub p95: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_compare_prediction",
    description: concat!(
        "Analyze prediction accuracy: error stats, calibration, drift detection, overconfidence analysis. ",
        "Feeds Pi Reflexion loop. Use to improve future predictions.",
    ),
    category: Category::Memory,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Loading execution pairs", "Computing prediction error", "Analyzing drift"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Analyze prediction accuracy: error, calibration, drift, overconfidence. Feeds Reflexion loop.",
    prompt_guidelines: &[
        "thriving: Run weekly. Identify systematic prediction biases.",
        "cautious: Run more frequently. Watch for overconfidence.",
        "declining: Run to understand what the Golem misjudged.",
        "terminal: Final analysis for death testament.",
    ],
};
```

### `memory_get_failure_patterns`

Cluster failed or underperforming executions by common features: market regime, volatility percentile, gas level, time of day. Returns hypotheses about failure causes and suggested parameter adjustments.

```rust
#[derive(Debug, Deserialize)]
pub struct GetFailurePatternsParams {
    pub strategy_name: Option<String>,
    /// Minimum cluster size (default: 5).
    #[serde(default = "default_5")]
    pub min_executions: u32,
    /// PnL threshold for "failure" in USD (default: 0.0).
    #[serde(default)]
    pub failure_threshold: f64,
}

#[derive(Debug, Serialize)]
pub struct FailurePatternsResult {
    pub clusters: Vec<FailureCluster>,
}

#[derive(Debug, Serialize)]
pub struct FailureCluster {
    pub features: serde_json::Value,     // Regime, volatility, gas, time-of-day
    pub execution_count: u32,
    pub avg_pnl: f64,
    pub hypothesis: String,
    pub suggested_adjustments: Vec<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_get_failure_patterns",
    description: "Cluster failed executions by features: regime, volatility, gas, time-of-day. Generates failure hypotheses.",
    category: Category::Memory,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Loading failure data", "Clustering", "Generating hypotheses"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Cluster failed executions by common features. Generate failure hypotheses.",
    prompt_guidelines: &[
        "thriving: Run monthly to detect systematic issues.",
        "cautious: Run weekly.",
        "declining: Run to understand what went wrong.",
        "terminal: Final analysis for death testament.",
    ],
};
```

---

## Grimoire import/export (2 tools)

### `memory_export_grimoire`

Export the complete Grimoire (episodes + insights + strategies) as a JSON archive. Use for backup and successor inheritance.

```rust
#[derive(Debug, Deserialize)]
pub struct ExportGrimoireParams {
    /// Include decayed episodes (default: false, exports only active entries).
    #[serde(default)]
    pub include_decayed: bool,
    /// Minimum insight confidence to include (default: 0.0).
    #[serde(default)]
    pub min_confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct ExportGrimoireResult {
    pub path: String,
    pub episodes_exported: u32,
    pub insights_exported: u32,
    pub strategies_exported: u32,
    pub file_size_bytes: u64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_export_grimoire",
    description: "Export complete Grimoire (episodes + insights + strategies) as JSON. Backup and successor inheritance.",
    category: Category::Memory,
    capability: CapabilityTier::Read,       // Reads Grimoire, writes filesystem
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Exporting episodes", "Exporting insights", "Exporting strategies", "Writing archive"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Export Grimoire as JSON. Backup and successor inheritance.",
    prompt_guidelines: &[
        "thriving: Export periodically as backup.",
        "cautious: Export more frequently. Prepare for potential succession.",
        "declining: Export as preparation for death testament.",
        "terminal: Final export. This becomes the predecessor Grimoire for the successor Golem.",
    ],
};
```

### `memory_import_grimoire`

Import a Grimoire from a predecessor (Becoming Bardo). Imported entries start at 0.4 confidence -- Whitehead's negative prehension. The successor must actively validate inherited knowledge; nothing is trusted by default.

```rust
#[derive(Debug, Deserialize)]
pub struct ImportGrimoireParams {
    /// Path to the Grimoire JSON archive.
    pub path: String,
    /// Override starting confidence for imported insights (default: 0.4).
    #[serde(default = "default_04")]
    pub import_confidence: f64,
    /// Dry run: preview import without writing (default: false).
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct ImportGrimoireResult {
    pub episodes_imported: u32,
    pub insights_imported: u32,
    pub strategies_imported: u32,
    pub confidence_override: f64,
    pub dry_run: bool,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "memory_import_grimoire",
    description: concat!(
        "Import Grimoire from predecessor (Becoming Bardo). ",
        "Imported entries start at 0.4 confidence (Whitehead's negative prehension). ",
        "They need active validation before being trusted.",
    ),
    category: Category::Memory,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Slow,         // Large import
    progress_steps: &["Reading archive", "Importing episodes", "Importing insights", "Importing strategies"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Import predecessor Grimoire. Entries start at 0.4 confidence. Needs active validation.",
    prompt_guidelines: &[
        "thriving: Import only during initialization (Becoming Bardo).",
        "cautious: Same.",
        "declining: Not applicable.",
        "terminal: Not applicable.",
    ],
};
```

**Ground truth:** Row counts in LanceDB and SQLite increase by imported amounts. Source: count queries post-import.

**Event Fabric:**

| Event | Payload |
| --- | --- |
| `grimoire:import_complete` | `{ episodes, insights, strategies, confidence_override }` |
| `dream:import_hook` | Signals the Dream cycle to schedule a consolidation pass on imported data |

---

## Dream cycle integration

Memory tools interact with the Dream cycle (the Golem's offline consolidation phase) through Event Fabric hooks. The Dream cycle runs when the Golem has no active Pi session, processing accumulated experience.

Three hooks connect memory tools to the Dream cycle:

1. **`dream:consolidation_hook`** -- Emitted by `memory_consolidate`. The Dream monitor watches for this event and logs consolidation stats to the Golem's status.

2. **`dream:import_hook`** -- Emitted by `memory_import_grimoire`. Triggers a scheduled consolidation pass on imported data within the current Dream cycle.

3. **`grimoire:episode_stored`** with `dream_tag` -- Episodes tagged for Dream processing are queued for the next Dream cycle's consolidation pass, even if the automatic 4-hour window hasn't elapsed.

The Dream cycle itself is not a tool. It is a Pi runtime behavior documented in `../01-golem/10-dream.md`.

---

## Ebbinghaus decay reference

| Importance | S parameter | 50% retention at | 10% retention at |
| --- | --- | --- | --- |
| Routine | 7 days | 4.8 days | 16.1 days |
| Notable | 30 days | 20.8 days | 69.1 days |
| Critical | 90 days | 62.4 days | 207.2 days |
| Emergency | 180 days | 124.7 days | 414.5 days |

Decay formula: R = e^(-t/S), where t is elapsed time and S is the strength parameter. Episodes with R below 0.05 are candidates for removal during the `memory_consolidate` decay pass.

---

## Custody implications (memory write tools)

Memory write tools modify local Grimoire state (LanceDB, SQLite, filesystem), not on-chain state. No custody signing is involved for any memory tool. The `memory` capability gate (`BARDO_MEMORY_ENABLED=true`) and the `learning` profile activation are the only prerequisites.

---

## Tool count summary

| Section | Tools | Read | Write |
| --- | --- | --- | --- |
| Episodic memory | 2 | 1 | 1 |
| Semantic memory | 3 | 1 | 2 |
| Strategy memory | 2 | 1 | 1 |
| Self-improvement (Pi lifecycle) | 4 | 2 | 2 |
| Grimoire import/export | 2 | 1 | 1 |
| **Total** | **13** | **6** | **7** |

All memory tools are `RiskTier::Layer1`. Write operations modify local Grimoire state (LanceDB, SQLite, filesystem) with no on-chain cost. The `memory` capability gate ensures these tools only load when `BARDO_MEMORY_ENABLED=true` and the `learning` profile is active.
