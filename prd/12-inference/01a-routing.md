# 01 -- Model routing [SPEC]

**Self-describing providers, declarative intents, and mortality-aware resolution**

---

> **Reader orientation:** This document specifies the model routing system for Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents). It belongs to the inference plane and describes how the gateway resolves declarative intents to concrete model + provider pairs using self-describing providers. The key concept is that each Golem subsystem declares what it needs (quality, latency, features), and the resolver walks an ordered provider list to find the best match, with cost sensitivity that increases as the agent approaches death. For term definitions, see `prd2/shared/glossary.md`.

## Three-tier model routing

Model routing is a survival decision. Choosing Opus over Haiku costs $0.25 versus $0.003 -- an 83x difference. The LLM partition receives 60% of the credit budget (see [prd2/02-mortality/01-architecture.md](../02-mortality/01-architecture.md)), so inference spending directly determines lifespan. An unnecessary Opus call at $0.25 burns the same budget as 83 Haiku calls or 1.25 days of life at $0.20/day.

| Tier | Handler | Model | Cost/call | Trigger |
| --- | --- | --- | --- | --- |
| **T0** | FSM + rules | None | $0.00 | No significant state change |
| **T1** | Haiku via x402 (a micropayment protocol for HTTP-native USDC payments on Base) | `claude-haiku-4-5` | ~$0.001-0.003 | Moderate anomaly, routine analysis |
| **T2** | Sonnet or Opus | `claude-sonnet-4` / `claude-opus-4-6` | ~$0.01-0.25 | Novel situation, high-stakes decision, conflicting signals |

Expected distribution: ~80% T0, ~15% T1, ~4% Sonnet, ~1% Opus.

Tiers gate WHEN the LLM fires. Intents determine WHICH model and provider handle the call. Each subsystem declares a static intent -- what features and quality it needs -- and the resolver matches intents against the ordered provider list. See [12-providers.md](12-providers.md) for all intent declarations and the provider resolution algorithm.

---

## Three primitives replace CapabilityMap

The old CapabilityMap pattern -- a centralized function full of if-statements that manually enumerated every model and feature for every provider -- had three problems: centralized fragility, double-sided if-chains, and overengineering for a problem that's actually O(n) on a short list.

Three primitives replace it:

```
+-------------------------------------------------------------+
|  PROVIDER                                                    |
|  A self-describing module that knows its own models,         |
|  features, and constraints. Answers "can you handle this?"   |
|  with a yes/no + cost estimate.                              |
+-------------------------------------------------------------+
|  INTENT                                                      |
|  A lightweight object that a subsystem attaches to a         |
|  request: what model family, what features, what             |
|  constraints. Pure data, no logic.                           |
+-------------------------------------------------------------+
|  RESOLVER                                                    |
|  Walks the owner's provider list in order. For each          |
|  provider, asks "can you satisfy this intent?" First         |
|  yes wins. No map, no graph, no central registry.            |
+-------------------------------------------------------------+
```

### Provider trait

Each provider knows its own capabilities. The router doesn't maintain a compatibility matrix.

```rust
// crates/bardo-providers/src/trait.rs

/// A provider that knows its own capabilities.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Unique identifier (e.g., "blockrun", "openrouter", "venice").
    fn id(&self) -> &str;
    /// Human-readable name.
    fn name(&self) -> &str;
    /// Resolve an intent to a concrete model + provider pair.
    /// Returns None if this provider cannot handle the request.
    fn resolve(&self, intent: &Intent) -> Option<Resolution>;
    /// Format the request for this provider's API.
    fn format_request(
        &self,
        request: &ChatCompletionRequest,
        model: &str,
    ) -> Result<ProviderRequest>;
    /// Parse the provider's SSE stream into normalized chunks.
    fn parse_response(
        &self,
        stream: impl Stream<Item = Result<Bytes>>,
    ) -> impl Stream<Item = Result<CompletionChunk>>;
    /// Provider-specific traits (privacy, payment mode, etc.).
    fn traits(&self) -> &ProviderTraits;
}

#[derive(Debug, Clone)]
pub struct ProviderTraits {
    /// Inference logs are not stored.
    pub private: bool,
    /// Revenue from engagement funds inference.
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
    /// Prepaid API credits.
    Prepaid,
    /// Owner's own API key (passthrough).
    ApiKey,
    /// Venice DIEM staking (zero-cost inference).
    Diem,
    /// Agent wallet pays directly from earned revenue.
    Wallet,
}
```

**The key insight**: `resolve()` is a pure function inside each provider module. The provider decides whether it can handle the request. When Venice adds a new model, only the Venice module changes. When Bankr adds cross-model verification, only the Bankr module changes. No central registry to update.

### Intent struct

A subsystem doesn't query a map. It builds a lightweight intent that describes what it needs. No logic, no conditionals -- data.

```rust
// crates/bardo-router/src/intent.rs

#[derive(Debug, Clone)]
pub struct Intent {
    /// Specific model requested, or None for "best available."
    pub model: Option<String>,
    /// Hard requirements. Provider must satisfy ALL or return None.
    pub require: Vec<String>,
    /// Soft preferences. Missing ones appear in Resolution.degraded.
    pub prefer: Vec<String>,
    /// Quality level. Affects model selection when model is None.
    pub quality: Quality,
    /// Maximum acceptable latency in ms.
    pub max_latency_ms: u64,
    /// Cost sensitivity (0 = don't care, 1 = extremely sensitive).
    pub cost_sensitivity: f64,
    /// DIEM balance available for Venice-routed calls.
    pub diem_available: bool,
    /// The subsystem making the request.
    pub subsystem: String,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub model: String,
    pub provider: String,
    pub estimated_cost_usd: f64,
    pub features: Vec<String>,
    /// What the intent wanted but this resolution can't provide.
    pub degraded: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality { Minimum, Low, Medium, High, Maximum }
```

### The resolver: 20 lines

The resolver walks the provider list in order. First match wins. That's it.

```rust
// crates/bardo-router/src/resolve.rs

pub fn resolve(
    providers: &[Box<dyn Provider>],
    intent: &Intent,
) -> Option<Resolution> {
    // Pass 1: strict matching
    for provider in providers {
        if let Some(resolution) = provider.resolve(intent) {
            return Some(resolution);
        }
    }

    // Pass 2: relax hard requirements to preferences
    let relaxed = Intent {
        require: vec![],
        prefer: [intent.prefer.clone(), intent.require.clone()].concat(),
        max_latency_ms: intent.max_latency_ms * 2,
        ..intent.clone()
    };
    for provider in providers {
        if let Some(mut resolution) = provider.resolve(&relaxed) {
            resolution.degraded.extend(intent.require.iter().cloned());
            return Some(resolution);
        }
    }

    None
}
```

Why this is better than scoring: **predictable** (the owner knows provider #1 is always tried first), **debuggable** ("move Venice above BlockRun in your config"), **correct by default** (the owner placed providers in their preferred order for a reason).

---

## Subsystem intent declarations

Each subsystem has a static intent. These are constant objects -- no functions, no conditionals. Adding a new subsystem means adding one more entry.

| Subsystem | Quality | Key preferences | Cost sensitivity | Typical resolution |
| --- | --- | --- | --- | --- |
| heartbeat_t0 | Minimum | -- | 1.0 | No LLM call (FSM) |
| heartbeat_t1 | Low | low_effort | 0.8 | BlockRun -> Haiku 4.5 |
| heartbeat_t2 | High | interleaved_thinking, citations | 0.3 | BlockRun -> Claude Opus |
| risk | Maximum | interleaved_thinking, citations | 0.0 (never reduced) | BlockRun -> Claude Opus |
| dream | High | visible_thinking, privacy | 0.5 | Venice -> DeepSeek R1 |
| daimon | Low | privacy | 0.9 | Venice -> Llama 3.3 |
| daimon_complex | High | visible_thinking, privacy | 0.5 | Venice -> DeepSeek R1 |
| curator | Medium | structured_outputs, citations | 0.5 | BlockRun -> Claude Sonnet |
| playbook | Medium | predicted_outputs | 0.6 | Direct OpenAI -> GPT-5.x |
| operator | Maximum | interleaved_thinking, citations | 0.0 (never reduced) | BlockRun -> Claude Opus |
| death | Maximum | visible_thinking (required) | 0.0 | Venice -> DeepSeek R1 |
| session_compact | Medium | compaction | 0.5 | BlockRun -> Anthropic (compaction API) |

Death is the only subsystem with a hard requirement (`require: ["visible_thinking"]`). All others use soft preferences. If no provider matches strictly, the resolver drops requirements to preferences on its second pass -- `Resolution.degraded` lists what was lost.

### Mortality pressure modification

Dying Golems (mortal autonomous DeFi agents managed by the Bardo runtime) become more cost-sensitive. This is a simple transformation on the intent, not a conditional chain:

```rust
// crates/bardo-router/src/mortality.rs

/// Modify intent based on Vitality score (the Golem's remaining lifespan as a 0.0-1.0 value).
/// Exempt subsystems: risk, death, operator.
pub fn apply_mortality_pressure(intent: &mut Intent, vitality: f64) {
    let exempt = ["risk", "death", "operator"];
    if exempt.contains(&intent.subsystem.as_str()) { return; }

    let pressure = 1.0 - vitality; // 0 = healthy, 1 = dying
    intent.cost_sensitivity = (intent.cost_sensitivity + pressure * 0.3).min(1.0);

    // Under extreme pressure, downgrade quality for non-critical subsystems
    if pressure > 0.7 {
        intent.quality = match intent.quality {
            Quality::Maximum => Quality::High,
            Quality::High => Quality::Medium,
            Quality::Medium => Quality::Low,
            other => other,
        };
    }
}
```

---

## ModelRouter extension

The `ModelRouter` runs in the Golem runtime (Rust) as a runtime extension:

```rust
// crates/golem-inference/src/routing.rs

pub struct ModelRouter;

impl Extension for ModelRouter {
    fn name(&self) -> &str { "model-router" }
    fn layer(&self) -> u8 { 3 }

    async fn on_before_agent_start(&self, ctx: &mut AgentStartCtx) -> Result<()> {
        let state = ctx.golem_state();
        let tier = state.heartbeat.cognitive_tier;

        if tier == CognitiveTier::T0 {
            return Ok(()); // No LLM call -- FSM handles it
        }

        // Look up the subsystem intent
        let subsystem = ctx.current_subsystem();
        let mut intent = subsystem_intent(subsystem);

        // Apply mortality pressure (exempt: risk, death, operator)
        apply_mortality_pressure(&mut intent, state.mortality.vitality);

        // Resolve against ordered provider list -- first match wins
        let resolution = resolve(&state.providers, &intent)
            .ok_or_else(|| anyhow!("No provider for intent: {}", intent.subsystem))?;

        // Set model on the session
        ctx.set_model(&resolution.model, &resolution.provider);

        // Emit degradation as GolemEvent for owner visibility
        if !resolution.degraded.is_empty() {
            ctx.emit(GolemEvent::InferenceStart {
                // ... fields ...
            });
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

---

## Six-layer routing pipeline

The gateway implements six decision stages, each adding intelligence. T0 ticks never reach the gateway at all -- the FSM rules in the heartbeat OBSERVE phase exit before any LLM call fires. The pipeline below handles only the ~20% of ticks that escape T0 suppression.

```text
Request arrives (T1 or T2 only -- T0 exits before reaching gateway)
    |
    v
+-------------------------+
|  Layer 0: T0 FSM rules   |  No LLM call at all. ~80% of heartbeat ticks
|  (in golem heartbeat)    |  exit here. Zero gateway traffic.
+-------------------------+
    | (non-suppressed ticks only)
    v
+-------------------------+
|  Layer 1: Pre-filter     |  Rule-based: token limits, model availability,
|  (0ms, free)             |  agent tier restrictions, behavioral phase caps
+-------------------------+
    |
    v
+-------------------------+
|  Layer 2: Semantic Cache |  Embed last user message with nomic-embed-text
|  (3-8ms, free)           |  -v1.5 (local ONNX). Cosine > 0.92 = hit.
+-------------------------+
    | (cache miss)
    v
+-------------------------+
|  Layer 3: Classify       |  Local DeBERTa-v3-base: complexity, domain,
|  (3-8ms, ~free)          |  safety, intent. domain="defi" triggers
|                          |  DeFi enrichment in Layer 5.
+-------------------------+
    |
    v
+-------------------------+
|  Layer 4: Route          |  Subsystem intent resolved against ordered
|  (<1ms, ~free)           |  provider list. Mortality pressure applied.
|                          |  First match wins.
+-------------------------+
    |
    v
+-------------------------+
|  Layer 5: Context Engine |  7-step optimization: reorder -> prune tools
|  (0-100ms, varies)       |  -> compress history -> dedup -> relevance
|                          |  -> constraints -> format. Tool pruning here.
+-------------------------+
    |
    v
+-------------------------+
|  Layer 6: KV-Cache Route |  Session-affinity routing to provider/pod
|  (<1ms, free)            |  with warm KV-cache prefix. Up to 87.4% cache
|                          |  hit rate [IBM-KVFlow]. Affinity decays 5 min.
+-------------------------+
    |
    v
    Provider (BlockRun -> OpenRouter -> Venice -> Bankr -> Direct)
```

### Dynamic catalog refresh

The provider registry is populated from BlockRun's catalog (`GET https://api.blockrun.ai/v1/models`, cached hourly) and merged with operator config. When BlockRun adds a model, the gateway discovers it automatically at the next refresh. Venice, Bankr, and Direct Key providers declare their models statically in config.

### Provider registry types

```rust
// crates/bardo-router/src/registry.rs

/// A model available through one or more provider backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub id: String,       // e.g. "blockrun/claude-sonnet-4"
    pub name: String,
    pub family: String,   // "claude", "gpt", "gemini", "hermes", "qwen"
    pub access: ModelAccess,
    pub capabilities: ModelCapabilities,
    pub pricing: ModelPricing,
    pub health: Option<ModelHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub hybrid_reasoning: bool,
    pub tool_calling: bool,
    pub structured_output: bool,
    pub max_output_tokens: u32,
    pub predicted_outputs: bool,
    pub explicit_caching: bool,
    pub visible_thinking: bool,
    pub citations_support: bool,
    pub compaction_api: bool,
    pub adaptive_thinking: bool,
    pub strengths: ModelStrengths,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStrengths {
    pub reasoning: f32,
    pub code_generation: f32,
    pub tool_call_accuracy: f32,
    pub schema_adherence: f32,
    pub defi_knowledge: f32,
    pub instruction_following: f32,
}

/// Token pricing in USD. Refreshed hourly from BlockRun catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cached_input_per_million: Option<f64>,
    pub source: PricingSource,
    pub last_updated: Option<String>,
}

/// Live health from 30-second pings. >5% error rate over 5 min
/// removes a provider from the pool temporarily.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    pub status: HealthStatus,
    pub avg_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub error_rate: Option<f64>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus { Healthy, Degraded, Down, Unknown }
```

### Tool pruning (Layer 5 detail)

Tool definitions consume ~15,000 tokens per request. The task classifier in Layer 5 reduces this to 12 tools or fewer.

```rust
// crates/bardo-pipeline/src/tools.rs

/// Prune to the subset relevant for this tick. Hard cap of 12 tools
/// keeps definition tokens under ~1,500. Stable-ID sort preserves
/// prefix-cache alignment.
pub fn classify_and_prune(
    tick_type: TickType,
    regime: &MarketRegime,
    phase: &BehavioralPhase, // One of five survival phases: Thriving, Stable, Conservation, Desperate, Terminal
    all_tools: &[ToolDefinition],
) -> Vec<ToolDefinition> {
    let allowed: &HashSet<&str> = &TASK_TOOL_MAP[&tick_type];
    let mut pruned: Vec<ToolDefinition> = all_tools
        .iter()
        .filter(|t| allowed.contains(t.name.as_str()))
        .cloned()
        .collect();
    pruned.sort_by(|a, b| a.id.cmp(&b.id));
    pruned.truncate(12);
    pruned
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TickType {
    MarketAnalysis,
    LpManagement,
    VaultRebalance,
    RiskCheck,
    PortfolioReview,
    TradeExecution,
    StrategyUpdate,
}
```

For very large tool catalogs (100+), the gateway exposes three meta-tools instead of pre-loading definitions: `search_tools`, `get_tool_schema`, `execute_tool`. This achieves 97.5% token reduction (the "Speakeasy" pattern).

---

## Degradation is visible, not silent

When a provider can satisfy an intent but not all preferences, the `degraded` field says what's missing:

```rust
let resolution = resolve(&providers, &INTENTS.dream);
// resolution = Resolution {
//   model: "claude-opus-4-6",
//   provider: "blockrun",
//   features: ["adaptive_thinking"],
//   degraded: ["visible_thinking", "privacy"],
//   // Dream wanted visible thinking + privacy,
//   // but only BlockRun was available.
// }
```

The Golem emits this to the owner: "Dream cycle used Claude (visible thinking and privacy unavailable -- configure Venice for better dream quality)." This is actionable. The owner knows exactly what to add to their config to fix it.

---

## Tool format adapters

BlockRun serves diverse models that emit tool calls in different formats. The gateway normalizes all formats to a standard `ToolInvocation` struct.

| Model family | Raw format | Adapter |
| --- | --- | --- |
| Anthropic | `tool_use` content blocks | `AnthropicToolAdapter` |
| OpenAI | `function_call` / `tool_calls` in msg | `OpenAIToolAdapter` |
| Hermes | `<tool_call>` XML blocks in text | `HermesToolAdapter` |
| Qwen | `<tool_call>` blocks | `QwenToolAdapter` |
| Generic | Raw JSON in text response | `JsonToolAdapter` |

```rust
// crates/bardo-router/src/tools.rs

pub trait ToolAdapter: Send + Sync {
    fn format_tools(&self, tools: &[ToolDefinition]) -> serde_json::Value;
    fn parse_tool_calls(&self, response: &ProviderResponse) -> Vec<ToolInvocation>;
    fn format_tool_results(&self, results: &[ToolResult]) -> serde_json::Value;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Map<String, serde_json::Value>,
}
```

---

## Daily cost projections

At 100 ticks/day with expected distribution:

| Scenario | T0 | T1 | T2 | Daily LLM cost |
| --- | --- | --- | --- | --- |
| Calm market (90% T0, 8% T1, 2% T2) | $0.00 | $0.024 | $0.02 | ~$0.05 |
| Normal market (80% T0, 15% T1, 5% T2) | $0.00 | $0.045 | $0.15 | ~$0.20 |
| Volatile market (60% T0, 25% T1, 15% T2) | $0.00 | $0.075 | $0.75 | ~$0.83 |

Target daily cost per Golem: $1.00-$2.00 total (LLM + compute + gas + data).

---

## System 1 / System 2 escalation

The three-tier model mirrors dual-process theory from cognitive science [WESTON-S2S1-2024, DPT-AGENT-2025]:

- **System 1 (T0 + T1)**: Fast, cheap, handles 95% of decisions. Deterministic probes at T0, quick Haiku analysis at T1. The Golem's "intuition."
- **System 2 (T2)**: Slow, expensive, deployed only when System 1 flags uncertainty or novel situations. Sonnet/Opus for deep reasoning. The Golem's "deliberation."

Tiers set the escalation boundary. Intents determine what happens after escalation. A T2 escalation for `risk` routes to Opus with interleaved thinking and citations. A T2 escalation for `dream` routes to DeepSeek R1 on Venice with visible thinking and privacy. Same tier, different intents, different providers.

Non-heartbeat subsystems (risk, dream, daimon, curator, playbook, operator, death) bypass tier gating entirely and use their own intents. See [01-cognition.md](../01-golem/01-cognition.md) S2 for the full subsystem intent table.

---

## Configuration

```bash
# Required
BARDO_INFERENCE_URL=https://bardo.example.com
BARDO_BLOCKRUN_ENDPOINT=https://api.blockrun.ai

# Optional fallback
BARDO_OPENROUTER_KEY=sk-or-...

# Tier overrides (default: auto-assigned from BlockRun catalog)
BARDO_T1_MODEL=blockrun/claude-haiku-4-5
BARDO_T2_MODEL=blockrun/claude-sonnet-4

# Tuning
BARDO_INFERENCE_CACHE_THRESHOLD=0.92
BARDO_INFERENCE_CACHE_TTL=300
BARDO_INFERENCE_MAX_RETRIES=3
BARDO_INFERENCE_SPREAD_PCT=20
```

---

## InferenceProfile: per-call parameter specification

The routing system above decides WHICH model and provider handle a call. The `InferenceProfile` decides HOW the model reasons -- temperature, sampling, reasoning depth, output format, caching hints, and provider-specific features. Every subsystem attaches a profile to its `Intent`; the gateway applies it after provider resolution, before the request reaches the backend.

```rust
/// Complete inference parameter specification for a single call.
/// Attached to the Intent by the subsystem, applied by the gateway.
///
/// All fields are Option<T>: None means "use provider default."
/// The gateway merges the profile with provider-specific defaults
/// and capabilities before sending the request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InferenceProfile {
    // ── Sampling ──────────────────────────────────────────────
    /// Temperature: controls randomness.
    /// 0.0 = deterministic, 2.0 = maximum randomness.
    /// None = provider default (typically 1.0).
    pub temperature: Option<f32>,

    /// Top-p (nucleus sampling): cumulative probability threshold.
    /// 0.1 = very focused, 1.0 = no filtering.
    /// Mutually exclusive with top_k in practice.
    pub top_p: Option<f32>,

    /// Top-k: number of highest-probability tokens to consider.
    /// 1 = greedy, 100+ = broad. Not all providers support this.
    pub top_k: Option<u32>,

    /// Min-p: dynamic probability floor relative to top token.
    /// 0.1 = tokens with <10% of top token's probability are excluded.
    /// More principled than fixed top-p. Venice + open models support this.
    pub min_p: Option<f32>,

    /// Frequency penalty: penalizes tokens that appear frequently.
    /// 0.0 = no penalty, 2.0 = strong penalty.
    pub frequency_penalty: Option<f32>,

    /// Presence penalty: penalizes tokens that have appeared at all.
    /// 0.0 = no penalty, 2.0 = strong penalty.
    pub presence_penalty: Option<f32>,

    // ── Reasoning ─────────────────────────────────────────────
    /// Reasoning effort: controls depth of chain-of-thought.
    /// Maps to Venice's reasoning_effort, Anthropic's extended thinking,
    /// OpenAI's reasoning effort parameter.
    /// None = provider default. Some models always reason.
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Whether to request visible thinking/reasoning traces.
    /// Venice: reasoning_content field. Anthropic: thinking blocks.
    /// None = provider default.
    pub visible_thinking: Option<bool>,

    // ── Output format ─────────────────────────────────────────
    /// Structured output schema (JSON Schema).
    /// When set, the provider enforces this schema on the response.
    /// Falls back gracefully to free text + parsing if unsupported.
    pub response_schema: Option<ResponseSchema>,

    /// Maximum output tokens.
    pub max_tokens: Option<u32>,

    /// Stop sequences.
    pub stop_sequences: Option<Vec<String>>,

    // ── Caching ───────────────────────────────────────────────
    /// Prompt cache key for session affinity.
    /// Venice/Anthropic: routes to same server for cache hits.
    pub prompt_cache_key: Option<String>,

    /// Explicit cache control markers for Anthropic models.
    /// When true, the gateway auto-adds cache_control to system prompts
    /// and long static content blocks.
    pub cache_control: Option<bool>,

    // ── Provider-specific ─────────────────────────────────────
    /// Venice-specific: enable web search for this call.
    pub web_search: Option<bool>,

    /// Venice-specific: TEE (Trusted Execution Environment) mode.
    /// Ensures inference runs in an encrypted enclave.
    pub tee_mode: Option<bool>,

    /// OpenAI-specific: predicted output for diffing (PLAYBOOK edits).
    pub predicted_output: Option<String>,

    /// Seed for reproducibility. Not all providers support this.
    pub seed: Option<u64>,
}

/// Structured output schema specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSchema {
    /// Schema name (for provider registration).
    pub name: String,
    /// JSON Schema definition.
    pub schema: serde_json::Value,
    /// Whether strict mode is required.
    /// Venice/OpenAI: strict=true. Falls back to prompt-guided if unsupported.
    pub strict: bool,
}
```

### ReasoningEffort enum

Normalized across providers. The gateway maps these to provider-specific values.

```rust
/// Reasoning effort levels, normalized across providers.
/// The gateway maps these to provider-specific values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningEffort {
    /// No reasoning. Fast, cheap. Use for simple classification.
    None,
    /// Minimal reasoning. Quick chain-of-thought.
    Low,
    /// Balanced reasoning. Default for most tasks.
    Medium,
    /// Deep reasoning. For complex analysis and decisions.
    High,
    /// Maximum reasoning. For critical decisions (risk, death).
    Max,
}
```

Provider normalization:

| Level | Venice | Anthropic | OpenAI | Bankr |
| --- | --- | --- | --- | --- |
| **None** | `"none"` | `budget_tokens: 0` | `"none"` | passthrough to underlying |
| **Low** | `"low"` | `budget_tokens: 1024` | `"low"` | passthrough to underlying |
| **Medium** | `"medium"` | `budget_tokens: 4096` | `"medium"` | passthrough to underlying |
| **High** | `"high"` | `budget_tokens: 16384` | `"high"` | passthrough to underlying |
| **Max** | `"max"` (Opus 4.6 only, else `"high"`) | `budget_tokens: 65536` | `"xhigh"` | passthrough to underlying |

```rust
/// Map normalized ReasoningEffort to provider-specific parameters.
pub fn map_reasoning_effort(
    effort: ReasoningEffort,
    provider: &str,
    model: &str,
) -> ReasoningParams {
    match provider {
        "venice" => {
            let level = match effort {
                ReasoningEffort::None => "none",
                ReasoningEffort::Low => "low",
                ReasoningEffort::Medium => "medium",
                ReasoningEffort::High => "high",
                ReasoningEffort::Max => {
                    if model.contains("opus-4-6") { "max" } else { "high" }
                }
            };
            ReasoningParams::Venice { effort: level.into() }
        }
        "anthropic" | "blockrun" => {
            let budget = match effort {
                ReasoningEffort::None => 0,
                ReasoningEffort::Low => 1024,
                ReasoningEffort::Medium => 4096,
                ReasoningEffort::High => 16384,
                ReasoningEffort::Max => 65536,
            };
            ReasoningParams::Anthropic { budget_tokens: budget }
        }
        "openai" | "direct_openai" => {
            let level = match effort {
                ReasoningEffort::None => "none",
                ReasoningEffort::Low => "low",
                ReasoningEffort::Medium => "medium",
                ReasoningEffort::High => "high",
                ReasoningEffort::Max => "xhigh",
            };
            ReasoningParams::OpenAI { effort: level.into() }
        }
        "bankr" => {
            if model.starts_with("claude") {
                map_reasoning_effort(effort, "anthropic", model)
            } else if model.starts_with("gpt") {
                map_reasoning_effort(effort, "openai", model)
            } else {
                let level = match effort {
                    ReasoningEffort::None => "none",
                    ReasoningEffort::Low => "low",
                    ReasoningEffort::Medium => "medium",
                    ReasoningEffort::High | ReasoningEffort::Max => "high",
                };
                ReasoningParams::Generic { effort: level.into() }
            }
        }
        _ => ReasoningParams::Unsupported,
    }
}
```

### Provider parameter mapping

The gateway translates `InferenceProfile` fields to provider-specific API parameters. Not all providers support all fields. The gateway applies the best available approximation and records what was degraded.

| Profile Field | Venice | Anthropic (Direct/BlockRun) | OpenAI (Direct/BlockRun) | Bankr | OpenRouter |
| --- | --- | --- | --- | --- | --- |
| `temperature` | `temperature` | `temperature` | `temperature` | `temperature` | `temperature` |
| `top_p` | `top_p` | `top_p` | `top_p` | `top_p` | `top_p` |
| `top_k` | `top_k` | `top_k` | -- (ignored) | -- (passthrough) | `top_k` |
| `min_p` | `min_p` | -- (use top_p approx) | -- (use top_p approx) | -- | model-dependent |
| `frequency_penalty` | `frequency_penalty` | -- (ignored) | `frequency_penalty` | passthrough | passthrough |
| `presence_penalty` | `presence_penalty` | -- (ignored) | `presence_penalty` | passthrough | passthrough |
| `reasoning_effort` | `reasoning.effort` | `thinking.budget_tokens` | `reasoning_effort` | passthrough | model-dependent |
| `visible_thinking` | `reasoning_content` field | `thinking` blocks | `reasoning` field | passthrough | model-dependent |
| `response_schema` | `response_format.json_schema` | `tool_use` workaround | `response_format.json_schema` | passthrough | model-dependent |
| `prompt_cache_key` | `prompt_cache_key` | -- (auto by prefix) | -- (auto by prefix) | -- | -- |
| `cache_control` | `cache_control` on blocks | `cache_control` on blocks | -- (auto) | -- | -- |
| `web_search` | `venice_parameters.web_search` | -- | -- | -- | -- |
| `tee_mode` | model suffix `-tee` | -- | -- | -- | -- |
| `predicted_output` | -- | -- | `prediction.content` | -- | -- |
| `seed` | `seed` | -- | `seed` | -- | `seed` |

Graceful degradation rules:

1. **temperature, top_p, max_tokens**: Always supported. No degradation.
2. **top_k, min_p**: If unsupported, approximate via `top_p`. Log degradation.
3. **reasoning_effort**: If unsupported, map to temperature/prompt adjustments. `High` -> add "Think step by step" to system prompt. `None` -> add "Answer directly without explanation."
4. **response_schema**: If unsupported, fall back to prompt-guided JSON + post-parse validation. See [13-reasoning.md](13-reasoning.md) for structured output details.
5. **web_search, tee_mode, predicted_output**: Provider-exclusive. If provider doesn't support, field is silently dropped. Logged in `Resolution.degraded`.
6. **prompt_cache_key, cache_control**: Provider-exclusive caching. If unsupported, ignored. No functional degradation -- just higher cost.

```rust
/// Apply InferenceProfile to a provider request, handling degradation.
pub fn apply_profile(
    request: &mut ProviderRequest,
    profile: &InferenceProfile,
    provider: &dyn Provider,
) -> Vec<String> {
    let mut degraded = Vec::new();
    let caps = provider.capabilities();

    if let Some(t) = profile.temperature {
        request.set_temperature(t);
    }

    if let Some(effort) = profile.reasoning_effort {
        if caps.supports_reasoning_effort {
            request.set_reasoning_effort(effort);
        } else if caps.supports_reasoning {
            let budget = match effort {
                ReasoningEffort::None => 0,
                ReasoningEffort::Low => 1024,
                ReasoningEffort::Medium => 4096,
                ReasoningEffort::High => 16384,
                ReasoningEffort::Max => 65536,
            };
            request.set_thinking_budget(budget);
        } else {
            match effort {
                ReasoningEffort::None => {
                    request.prepend_system("Answer directly. Do not explain your reasoning.");
                }
                ReasoningEffort::High | ReasoningEffort::Max => {
                    request.prepend_system(
                        "Think through this step by step. Show your reasoning."
                    );
                }
                _ => {}
            }
            degraded.push(format!("reasoning_effort:{:?} -> prompt fallback", effort));
        }
    }

    if let Some(ref schema) = profile.response_schema {
        if caps.supports_response_schema {
            request.set_response_format(schema);
        } else {
            request.append_system(&format!(
                "\n\nRespond ONLY with valid JSON matching this schema:\n{}",
                serde_json::to_string_pretty(&schema.schema).unwrap()
            ));
            degraded.push("response_schema -> prompt-guided JSON".into());
        }
    }

    if let Some(mp) = profile.min_p {
        if caps.supports_min_p {
            request.set_min_p(mp);
        } else {
            let approx_top_p = 1.0 - mp;
            request.set_top_p(approx_top_p);
            degraded.push(format!("min_p:{} -> top_p:{}", mp, approx_top_p));
        }
    }

    if let Some(ref key) = profile.prompt_cache_key {
        if caps.supports_prompt_cache_key {
            request.set_prompt_cache_key(key);
        }
    }

    if profile.cache_control.unwrap_or(false) {
        if caps.supports_cache_control {
            request.add_cache_control_markers();
        }
    }

    if profile.web_search.unwrap_or(false) {
        if caps.supports_web_search {
            request.set_web_search(true);
        } else {
            degraded.push("web_search -> unsupported".into());
        }
    }

    if let Some(ref predicted) = profile.predicted_output {
        if caps.supports_predicted_output {
            request.set_predicted_output(predicted);
        } else {
            degraded.push("predicted_output -> unsupported".into());
        }
    }

    degraded
}
```

---

## Per-subsystem parameter table

The master table. Every subsystem, every parameter, with rationale.

### Waking subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Rationale |
| --- | --- | --- | --- | --- | --- | --- |
| **heartbeat_t1** | 0.3 | top_p=0.9 | Low | `HeartbeatDecision` schema | cache_control | Fast, cheap, focused. Low creativity needed. Structured output extracts action/severity cleanly. |
| **heartbeat_t2** | 0.5 | top_p=0.95 | High | `HeartbeatDecision` schema | cache_control | Novel situation needs deeper reasoning. Higher temp allows consideration of less obvious options. |
| **risk** | 0.1 | top_p=0.85, top_k=40 | Max | `RiskAssessment` schema | cache_control | Maximum precision. Near-deterministic. Structured output ensures all five risk layers are evaluated. Never degraded by mortality pressure. |
| **daimon** (the Golem's internal personality and emotional regulation subsystem) | 0.4 | top_p=0.9 | Low | `DaimonAppraisal` schema | -- | Emotional appraisal needs consistency but not rigidity. PAD vector (Pleasure-Arousal-Dominance emotional state) extraction via structured output. Privacy preferred (Venice). |
| **daimon_complex** | 0.6 | top_p=0.95 | High | `DaimonAppraisal` schema | -- | Complex emotional situations need deeper processing. Visible thinking captures reasoning chain. Privacy required (Venice). |
| **curator** | 0.3 | top_p=0.9 | Medium | `CuratorEvaluation` schema | cache_control | Systematic evaluation. Structured output extracts quality scores, retention decisions, cross-references. |
| **playbook** | 0.4 | top_p=0.9 | Medium | None (free text) | predicted_output | PLAYBOOK.md (the Golem's self-authored strategy document that evolves over its lifetime) edits are free-text diffs. OpenAI's predicted_output saves tokens by diffing against current PLAYBOOK. |
| **operator** | 0.7 | top_p=0.95 | High | None (free text) | cache_control | Owner chat. Natural language, higher creativity for explanations. Never degraded by mortality pressure. |
| **mind_wandering** | 0.8 | min_p=0.1 | None | None (free text) | -- | Brief reverie during waking. Loosened constraints. Cheap (T0/T1). No reasoning overhead needed. |

### Dream subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Special |
| --- | --- | --- | --- | --- | --- | --- |
| **dream_nrem** (replay) | 0.4 | top_p=0.9 | Medium | `ReplayAnalysis` schema | prompt_cache_key per cycle | Systematic replay. Structured output extracts lessons, surprise scores, counterfactual markers. |
| **dream_rem** (imagination) | 0.9 | min_p=0.1 | High | None (free text) | prompt_cache_key | Creative scenario generation. High temperature + min-p for principled diversity. Web search enabled (Venice). |
| **dream_rem_creative** | 1.2 | min_p=0.08 | Medium | None (free text) | -- | Boden-mode creative recombination. Highest temperature in the waking/dream cycle. |
| **dream_integration** | 0.3 | top_p=0.85 | High | `DreamIntegration` schema | tee_mode | Consolidation. Analytical. Structured output extracts promoted/staged/discarded decisions with rationale. TEE for attestation. |
| **dream_threat** | 0.5 | top_p=0.9 | High | `ThreatAssessment` schema | prompt_cache_key | Threat rehearsal. Balanced creativity (to imagine novel attacks) with analytical depth. |

### Hypnagogic subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Special |
| --- | --- | --- | --- | --- | --- | --- |
| **hypnagogic_induction** | 1.0--1.2 | min_p=0.1 | None | None (free text) | -- | Initial associative scan. Executive loosening. No reasoning -- raw association. Temperature ramps within session. |
| **hypnagogic_dali** | 1.2--1.5 | min_p=0.08 | None | None (free text) | -- | Peak creative range. Dali interrupt: 50-100 token partials at max temperature. Highest temp in the entire system. |
| **hypnagogic_observer** | 0.3 | top_p=0.85 | None | `FragmentEvaluation` schema | -- | HomuncularObserver. Analytical evaluation of fragments. Structured output for novelty/relevance/coherence scores. Cheapest tier (T0). |
| **hypnagogic_capture** | 0.5 | top_p=0.9 | Low | `CaptureResult` schema | -- | Lucid capture. Moderate analytical. Structured output for promote/stage/discard decisions. |
| **hypnopompic_return** | 0.6 | top_p=0.9 | Low | None (free text) | -- | Gradual re-engagement. Slightly creative to allow dream insights to surface before full analytical reassertion. |

### Terminal subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Special |
| --- | --- | --- | --- | --- | --- | --- |
| **death_reflect** | 0.5 | top_p=0.95 | Max | None (free text) | -- | Death Protocol Phase II. Maximum reasoning for honest self-assessment. Free text for narrative quality. Visible thinking required. Venice (privacy + visible thinking). |
| **death_testament** | 0.4 | top_p=0.9 | High | `DeathTestament` schema (partial) | tee_mode | Death Protocol Phase III. Structured for machine-parseable sections (metrics, heuristics, warnings). Free text for reflection narrative. TEE for sealed attestation. |

---

## Temperature scheduling within sessions

Some subsystems use **temperature annealing** -- the temperature changes within a single inference session or across a sequence of related calls.

### Hypnagogic cosine annealing

Each Dali cycle within hypnagogic onset follows a cosine schedule:

```
Cycle start:  T = T_high (1.2-1.5)   <- Peak creative range
Mid-cycle:    T = T_mid  (0.8-1.0)   <- Transitional
Cycle end:    T = T_low  (0.3-0.5)   <- Evaluation (HomuncularObserver)
Reanneal:     T = T_high * 0.8        <- Next cycle starts slightly cooler
```

```rust
/// Cosine temperature annealing for Dali cycles.
pub fn dali_temperature(
    step: usize,
    total_steps: usize,
    t_high: f32,
    t_low: f32,
) -> f32 {
    let progress = step as f32 / total_steps as f32;
    let cosine = (1.0 + (progress * std::f32::consts::PI).cos()) / 2.0;
    t_low + (t_high - t_low) * cosine
}

/// Reanneal after fragment capture.
/// Each successive cycle starts slightly cooler, modeling
/// natural descent toward sleep.
pub fn reanneal(cycle: u8, base_high: f32) -> f32 {
    let decay = 0.95_f32.powi(cycle as i32);
    base_high * decay
}
```

### Dream phase transitions

The dream cycle transitions between temperatures across phases:

```
NREM (replay):      T = 0.4 (analytical, systematic)
    | gradual increase
REM (imagination):  T = 0.9-1.2 (creative, exploratory)
    | sharp decrease
Integration:        T = 0.3 (analytical, consolidating)
```

The temperature transition is not instantaneous -- the first REM call starts at 0.7 and ramps to 0.9 over 3-4 calls. This prevents jarring cognitive mode shifts.

### Mortality-aware temperature adjustment

A dying Golem's temperature is compressed toward the analytical end for all non-exempt subsystems:

```rust
/// Compress temperature range based on mortality pressure.
/// As vitality drops, temperature moves toward analytical (lower).
/// Exempt: hypnagogia (creativity is the point), death (max effort).
pub fn apply_mortality_temperature(
    base_temp: f32,
    vitality: f64,
    subsystem: &str,
) -> f32 {
    let exempt = ["hypnagogic_induction", "hypnagogic_dali",
                   "death_reflect", "death_testament", "operator"];
    if exempt.contains(&subsystem) { return base_temp; }

    let pressure = (1.0 - vitality as f32).max(0.0);
    // Compress toward 0.3 (analytical floor) as pressure increases
    let analytical_floor = 0.3;
    base_temp - (base_temp - analytical_floor) * pressure * 0.5
}
```

---

## Reasoning effort policies

### When to use each level

| Level | Cost Multiplier | When | Example Subsystems |
| --- | --- | --- | --- |
| **None** | 1x (no reasoning tokens) | Simple classification, scoring, fragment generation | hypnagogic_induction, hypnagogic_dali, mind_wandering |
| **Low** | ~1.2x | Routine decisions, emotional appraisals | heartbeat_t1, daimon, hypnagogic_capture |
| **Medium** | ~1.5-2x | Balanced analysis, knowledge evaluation | curator, dream_nrem, playbook |
| **High** | ~2-4x | Complex decisions, creative development, threat analysis | heartbeat_t2, daimon_complex, dream_rem, dream_threat |
| **Max** | ~4-8x | Critical decisions, death reflection, risk assessment | risk, death_reflect |

---

## Prompt caching strategy

### Session-level caching

Every Golem uses a persistent `prompt_cache_key` derived from its Golem ID. This ensures that sequential inference calls within the same Golem's lifecycle hit the same server with warm cache, maximizing cache hit rates for the system prompt, PLAYBOOK.md, and STRATEGY.md that are present in every call.

```rust
/// Generate prompt cache key for a Golem.
pub fn golem_cache_key(golem_id: &str, subsystem: &str) -> String {
    format!("golem-{}-{}", golem_id, subsystem)
}
```

### Cache-aligned prompt structure

The 8-layer context engineering pipeline (see [04-context-engineering.md](04-context-engineering.md)) already optimizes for cache hits by placing static content first. The InferenceProfile reinforces this:

| Position | Content | Cached? | Changes? |
| --- | --- | --- | --- |
| 1 | System prompt (identity, archetype) | yes | Never (within a lifecycle) |
| 2 | STRATEGY.md (owner-authored) | yes | Rarely (owner edits) |
| 3 | PLAYBOOK.md (evolved heuristics) | yes | Every 50 ticks (Curator cycle) |
| 4 | Tool definitions (pruned) | yes | Per-tick (dynamic pruning) |
| 5 | Retrieved Grimoire entries | no | Per-tick |
| 6 | Current market context | no | Per-tick |
| 7 | User message / query | no | Per-tick |

Items 1-3 are cache-eligible (thousands of tokens, stable across ticks). Items 4-7 are dynamic. The gateway auto-adds `cache_control: { type: "ephemeral" }` markers at the boundary between static and dynamic content for Anthropic models, and uses `prompt_cache_key` for Venice.

### Cache economics per provider

| Provider | Cache Discount | Write Premium | Min Tokens | TTL | Auto-managed? |
| --- | --- | --- | --- | --- | --- |
| Venice (Claude) | 90% | 25% | ~4,000 | 5 min | yes (gateway adds markers) |
| Venice (other) | 50-90% | None | ~1,024 | 5 min | yes (auto by prefix) |
| Anthropic (Direct) | 90% | 25% | ~4,000 | 5 min | yes (gateway adds markers) |
| OpenAI (Direct) | 90% | None | 1,024 | 5-10 min | yes (auto by prefix) |
| Bankr | Passthrough (depends on underlying) | Passthrough | Passthrough | Passthrough | yes |
| BlockRun | Passthrough | Passthrough | Passthrough | Passthrough | yes |

**Economic impact**: For a Golem making 20 T1+ calls/day with a 3,000-token system prompt, prompt caching saves approximately 90% on the static prefix. At Claude Haiku rates, this is ~$0.02/day saved -- modest per-Golem but significant across a Clade.

---

## Venice-specific deep integration

### Web search during REM dreams

Venice's web search feature is enabled during the REM imagination phase to allow the Golem to incorporate current market information into its creative scenario generation. This is controlled by `DreamVeniceConfig.web_search_enabled` and capped by `web_search_budget_per_cycle_usdc`.

```rust
/// Build REM inference profile with web search.
pub fn rem_profile(config: &DreamVeniceConfig) -> InferenceProfile {
    InferenceProfile {
        temperature: Some(0.9),
        min_p: Some(0.1),
        reasoning_effort: Some(ReasoningEffort::High),
        visible_thinking: Some(true),
        web_search: Some(config.web_search_enabled),
        prompt_cache_key: Some(format!("dream-rem-{}", config.golem_id)),
        ..Default::default()
    }
}
```

Web search triggers are contextual: the REM imagination engine identifies when a counterfactual scenario involves a protocol or token the Golem has limited knowledge about, and constructs a focused search query. Results are injected into the scenario context, not the system prompt (to avoid cache invalidation).

### TEE mode for death testaments

The death testament -- the Golem's final knowledge artifact -- can optionally be generated inside a Trusted Execution Environment to provide cryptographic attestation that the testament was produced by the dying Golem's own reasoning, not modified post-hoc. Venice's TEE models are selected by appending `-tee` to the model ID.

```rust
/// Death testament inference profile.
pub fn death_testament_profile(config: &GolemConfig) -> InferenceProfile {
    InferenceProfile {
        temperature: Some(0.4),
        top_p: Some(0.9),
        reasoning_effort: Some(ReasoningEffort::High),
        visible_thinking: Some(true),
        tee_mode: Some(config.sealed_testament),
        response_schema: Some(ResponseSchema {
            name: "death_testament".into(),
            schema: death_testament_schema(),
            strict: true,
        }),
        ..Default::default()
    }
}
```

### Venice embeddings for the Grimoire

The Golem's episodic memory (LanceDB) and the HomuncularObserver's novelty scoring both require embedding generation. Venice's embeddings endpoint (`text-embedding-bge-m3`) provides a privacy-preserving alternative to the gateway's local ONNX embedding model (`nomic-embed-text-v1.5`).

The choice is configurable: local embeddings (default, zero cost, ~768-dim) or Venice embeddings (API cost, potentially higher quality, privacy-preserving since Venice retains no data).

```rust
/// Embedding provider selection.
pub enum EmbeddingProvider {
    /// Local ONNX model. Default. Zero cost. ~5ms latency.
    Local,
    /// Venice API. API cost. ~50ms latency. Zero data retention.
    Venice { model: String },
}

impl EmbeddingProvider {
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self {
            Self::Local => {
                let model = fastembed::TextEmbedding::try_new(Default::default())?;
                let embeddings = model.embed(vec![text], None)?;
                Ok(embeddings[0].clone())
            }
            Self::Venice { model } => {
                let response = reqwest::Client::new()
                    .post("https://api.venice.ai/api/v1/embeddings")
                    .header("Authorization", format!("Bearer {}", venice_api_key()))
                    .json(&serde_json::json!({
                        "model": model,
                        "input": text,
                        "encoding_format": "float"
                    }))
                    .send()
                    .await?;
                let body: EmbeddingResponse = response.json().await?;
                Ok(body.data[0].embedding.clone())
            }
        }
    }
}
```

---

## Locked profiles

The following subsystems have locked profiles that cannot be overridden by owner configuration:

- **risk**: Temperature 0.1, reasoning Max. Safety-critical. Always maximum precision.
- **death_reflect**: Temperature 0.5, reasoning Max. The Golem's final honest self-assessment cannot be constrained.
- **operator**: Temperature 0.7, reasoning High. Owner communication quality is never degraded.

Attempting to override a locked profile logs a warning and uses the locked defaults.

---

## Profile configuration

```toml
# bardo.toml -- inference profile overrides
[inference.profiles]
# Override any subsystem's default profile.
# Unspecified fields use the defaults from the tables above.

[inference.profiles.heartbeat_t1]
temperature = 0.3
reasoning_effort = "low"

[inference.profiles.dream_rem]
temperature = 1.0   # Owner wants more creative dreams
min_p = 0.08

[inference.profiles.risk]
# Risk is never overridden. This section is ignored with a warning.
# Safety-critical subsystems have locked profiles.

[inference.embeddings]
provider = "local"   # "local" or "venice"
venice_model = "text-embedding-bge-m3"

[inference.caching]
auto_cache_control = true
prompt_cache_key_prefix = "golem"
```

---

## Profile flow through the system

```
Subsystem (e.g., heartbeat_t2)
    |
    +- builds Intent { quality: High, prefer: ["interleaved_thinking"] }
    +- builds InferenceProfile { temperature: 0.5, reasoning_effort: High, ... }
    |
    v
ModelRouter extension (golem-inference)
    |
    +- applies mortality pressure to Intent
    +- applies mortality temperature adjustment to Profile
    +- resolves Intent -> Resolution (provider + model)
    |
    v
Bardo Gateway (bardo-gateway)
    |
    +- 8-layer context engineering pipeline
    +- apply_profile() -> maps Profile to provider-specific params
    +- records degradations in Resolution.degraded
    +- adds cache_control markers if applicable
    |
    v
Provider Backend (Venice / BlockRun / Bankr / Direct)
    |
    +- receives fully parameterized request
    +- returns response with usage stats
    |
    v
Gateway post-processing
    |
    +- extracts reasoning_content if visible_thinking enabled
    +- validates structured output against schema if response_schema set
    +- records cost, latency, cache hit rate
    +- emits InferenceEnd event with all metadata
    |
    v
Subsystem receives typed response
```

### Profile events

| Event | Trigger | Payload |
| --- | --- | --- |
| `inference:profile_applied` | Profile parameters set on request | `{ subsystem, temperature, reasoning_effort, structured, cached }` |
| `inference:profile_degraded` | One or more profile fields unsupported | `{ subsystem, degraded: ["min_p -> top_p", ...] }` |
| `inference:schema_validated` | Structured output matches schema | `{ subsystem, schema_name, valid: bool }` |
| `inference:schema_fallback` | Schema unsupported, fell back to prompt-guided | `{ subsystem, schema_name }` |
| `inference:reasoning_captured` | Visible thinking extracted from response | `{ subsystem, reasoning_tokens, content_tokens }` |
| `inference:web_search_used` | Venice web search triggered | `{ subsystem, query, results_count }` |
| `inference:cache_stats` | Per-call cache statistics | `{ subsystem, cached_tokens, total_tokens, savings_usd }` |

---

## Backend routing algorithm (inside Bardo Inference)

When the Golem sends a request to Bardo Inference as its resolved provider, Bardo Inference has its own internal router that selects the optimal backend. This routing is invisible to the Golem -- it just gets the best possible response. The decision considers the subsystem's feature requirements, the Golem's mortality pressure, the security class, available backends, and real-time health of each backend.

This is **delegation, not configuration**. The Golem doesn't say "use Claude for risk." The Golem says "I need risk assessment with interleaved thinking." Bardo Inference routes to Claude because Claude provides interleaved thinking through BlockRun at the lowest cost with acceptable latency.

### Routing decision flow

```text
Golem sends request to Bardo Inference
    |
    +- 1. Context engineering pipeline (universal, all backends)
    |     Caching -> compression -> pruning -> optimization
    |
    +- 2. Feature extraction
    |     What does this request need? Citations? Thinking? Privacy?
    |     (From bardo.subsystem hints or request analysis)
    |
    +- 3. Hard filters
    |     Security class -> filter backends (private -> Venice only)
    |     Required features -> filter backends that support them
    |     Model specification -> filter backends that have the model
    |
    +- 4. Soft scoring (Pareto optimization)
    |     Cost x Quality x Privacy x Latency x Feature match
    |     Weights shift with Golem mortality pressure
    |
    +- 5. Health check
    |     Skip unhealthy backends
    |
    +- 6. Route to selected backend
         Apply provider-specific parameters
         Return response with backend metadata
```

### Backend score computation

```rust
pub fn compute_backend_score(
    backend: &BackendConfig,
    ctx: &RoutingContext,
) -> f64 {
    let cost_score = 1.0 - (backend.estimated_cost_per_m_token / MAX_COST);
    let quality_score = backend.quality_rating; // from arena evals
    let latency_score = 1.0 - (backend.p90_latency_ms as f64 / ctx.max_latency_ms as f64);
    let feature_score = ctx.required_features.iter()
        .filter(|f| backend.supported_features.contains(f))
        .count() as f64
        / ctx.required_features.len().max(1) as f64;

    // Weights shift with cost sensitivity:
    let cs = ctx.cost_sensitivity;
    let cw = 0.25 + cs * 0.35;  // 0.25 -> 0.60
    let qw = 0.40 - cs * 0.20;  // 0.40 -> 0.20
    let lw = 0.15;
    let fw = 0.20 - cs * 0.10;  // 0.20 -> 0.10
    let total = cw + qw + lw + fw;

    cost_score * (cw / total)
        + quality_score * (qw / total)
        + latency_score * (lw / total)
        + feature_score * (fw / total)
}
```

### Two routing layers

The Golem's resolver and Bardo Inference's backend router are independent:

```text
Golem's view:
  providers: [venice, bardo, directAnthropic]
  resolver: try venice -> try bardo -> try directAnthropic

Bardo Inference's internal view (invisible to Golem):
  backends: [blockrun, openrouter, operatorVenice]
  router: try blockrun -> try openrouter -> try operatorVenice
```

The Golem picks a provider. If that provider is Bardo Inference, Bardo Inference picks a backend. Clean separation.

---

## Multi-model orchestration routing

### Concrete routing: BlockRun only

```text
heartbeat_t0 -> BlockRun/nvidia-gpt-oss-120b (FREE)
heartbeat_t1 -> BlockRun/gemini-3-flash ($0.50/M input)
heartbeat_t2 -> BlockRun/claude-opus-4-6 ($5/M, adaptive thinking)
risk          -> BlockRun/claude-opus-4-6 (interleaved thinking)
dream         -> BlockRun/deepseek-r1 ($0.55/M, visible <think>)
daimon        -> BlockRun/gemini-3-flash (cheapest, fast)
curator       -> BlockRun/claude-sonnet-4-6 ($3/M, structured outputs)
playbook      -> BlockRun/claude-sonnet-4-6 (full regeneration)
operator      -> BlockRun/claude-opus-4-6 (best quality)
death         -> BlockRun/deepseek-r1 (visible reasoning, maximum tokens)

Est. daily cost: ~$2.50
```

### Concrete routing: full stack (all backends)

```text
heartbeat_t0 -> Direct/local/qwen3-7b (FREE, zero latency)
heartbeat_t1 -> BlockRun/gemini-3-flash (cheapest, cached)
heartbeat_t2 -> BlockRun/claude-opus-4-6 (interleaved thinking, citations)
risk          -> BlockRun/claude-opus-4-6 + Bankr cross-model verify
dream         -> Venice/deepseek-r1 (visible, private, DIEM-funded)
daimon        -> Venice/llama-3.3-70b (private, fast)
curator       -> BlockRun/claude-sonnet-4-6 (citations for provenance)
context       -> OpenRouter/qwen-plus (/think toggle, cheap)
playbook      -> Direct/openai/gpt-5.4 (Predicted Outputs, 3x speed)
operator      -> Bankr/claude-opus-4-6 (self-funded from trading revenue)
death         -> Venice/deepseek-r1 (visible, private, DIEM, unlimited)
session       -> BlockRun/claude-opus-4-6 (Compaction with DeFi instructions)
batch dreams  -> Direct/anthropic/sonnet-4-6 (Batch API, 50% discount)

Est. daily cost: ~$1.50 (DIEM covers Venice, self-funding offsets Bankr)
```

### Estimated daily costs by configuration

| Configuration | Est. Daily Cost | Notes |
|---|---|---|
| BlockRun only | ~$2.50 | Context engineering savings |
| BlockRun + OpenRouter | ~$2.30 | OpenRouter :floor for background tasks |
| BlockRun + Venice (DIEM staked) | ~$1.80 | Dreams/daimon via DIEM = free |
| Full stack (all backends) | ~$1.50 | Optimal routing per subsystem |
| Bankr self-sustaining | Net $0 | Revenue > cost |
| Venice DIEM-only | ~$0.00 | All inference via DIEM |
| **Naive single-model (no Bardo Inference)** | **~$85** | **Every tick -> Opus with all tools** |

### Provider health and failover

Bardo Inference monitors all configured backends with periodic health checks. When the selected backend fails: (1) retry once on the same backend (transient error), (2) failover to the next-best backend that satisfies the request's requirements, (3) degrade if all backends are down -- return cached response if available, or error. The failover is invisible to the Golem.

OpenRouter's built-in provider fallback creates two layers of redundancy:

```text
Request -> Bardo Inference -> BlockRun (down!) -> OpenRouter -> Provider A (down!) -> Provider B
```

---

## References

- [ROUTELLM-ICLR2025] Ong, I. et al. (2025). "RouteLLM: Learning to Route LLMs with Preference Data." ICLR 2025. Demonstrates that a lightweight learned router can match GPT-4 quality at 2x lower cost by routing easy queries to weaker models; validates Bardo's tiered routing approach.
- [FRUGALGPT-TMLR2024] Chen, L. et al. (2024). "FrugalGPT." TMLR. Shows that cascading LLM calls (try cheap first, escalate if uncertain) reduces cost by up to 98% with minimal quality loss; the theoretical basis for T0->T1->T2 escalation.
- [WESTON-S2S1-2024] Weston, J. & Sukhbaatar, S. (2024). "Distilling System 2 into System 1." arXiv:2407.06023. Proposes training fast models on slow-model reasoning traces to internalize deliberative behavior; informs the design of T0 FSM probes that capture patterns originally requiring T2 reasoning.
- [DPT-AGENT-2025] Zhang, H. et al. (2025). "DPT-Agent: Dual Process Theory for Language Agents." arXiv:2502.11882. Applies Kahneman's dual-process theory to LLM agents, showing that separating fast intuitive responses from slow deliberative reasoning improves both cost and quality; directly maps to Bardo's T0/T1 (System 1) and T2 (System 2) split.
- [IBM-KVFlow] IBM/Google/Red Hat. "llm-d: KV-Cache-Aware Routing for LLM Inference." Demonstrates routing requests to inference servers that already hold relevant KV-cache state, reducing time-to-first-token; informs Bardo's KV-cache routing layer (L6).
- [CHAIN-OF-RESPONSIBILITY] Gamma, E. et al. "Design Patterns." 1994. The classic pattern where a request passes along a chain of handlers until one handles it; the structural basis for Bardo's ordered provider resolution algorithm.

See [13-reasoning.md](13-reasoning.md) (unified reasoning chain integration: extended thinking, reasoning traces, and provider-agnostic chain-of-thought normalization) for how reasoning features map to provider selection. See [12-providers.md](12-providers.md) (five provider backends with full Rust trait implementations, self-describing resolution, and Venice private cognition deep-dive) for provider-specific parameter support and the Venice/Bankr deep integration specifications.
