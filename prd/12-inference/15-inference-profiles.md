# Inference Parameters: Temperature, Sampling, and Reasoning Policies

> **Version**: 1.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Parent**: `prd2/11-inference/`
>
> **Crate**: `golem-inference`
>
> **Depends on**: `golem-core`, `golem-daimon`, `golem-dreams`, `bardo-gateway`, `bardo-providers`
>
> **Purpose**: Define the complete inference parameter policy for every Golem subsystem across every cognitive state. Today, the inference gateway routes requests to the right model and provider (see `01a-routing.md`), but it does not specify *how* the model should reason -- what temperature to use, what sampling strategy, what reasoning effort, what response format. This document fills that gap. Written for a first-time reader with no assumed familiarity with the Bardo architecture.

---

> **Reader orientation:** This document specifies the inference parameter policies for every Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) subsystem across every cognitive state. It belongs to the Bardo Inference layer and defines temperature, sampling, reasoning effort, and response format per subsystem (heartbeat, risk, dream, Daimon, curator, playbook, operator, death). The key concept is that different subsystems need different inference parameters: risk assessment demands near-deterministic output (temperature 0.1), while dream cycles need creative recombination (temperature 0.8-0.9). For term definitions, see `prd2/shared/glossary.md`.

## Why This Matters

An LLM call is not just a string of prompt text sent to a model. Every provider exposes parameters that fundamentally alter the character of the response: temperature controls randomness, sampling strategies control which tokens are reachable, reasoning effort controls how deeply the model thinks before answering, and response format controls whether the output is free text or structured data. Using the same parameters for every call -- the naive default -- wastes money, produces worse results, and misses provider-specific features that exist precisely to solve the problems each Golem subsystem faces.

The Golem has 12+ subsystems that make inference calls, each with radically different needs. The risk engine needs maximum precision and zero creativity. Hypnagogic onset needs maximum creativity and loosened constraints. The Curator needs structured data extraction. The death testament needs deep, visible reasoning. Treating all of these as "send prompt, get text" is like using the same wrench for every bolt.

This document defines the **InferenceProfile** -- a per-subsystem, per-cognitive-state specification of exactly which parameters to set on every inference call. The profile is attached to the Intent (see `01a-routing.md`) and applied by the gateway after provider resolution, before the call reaches the backend.

---

## The InferenceProfile

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

---

## Provider Parameter Mapping

The gateway translates `InferenceProfile` fields to provider-specific API parameters. Not all providers support all fields. The gateway applies the best available approximation and records what was degraded.

| Profile Field | Venice | Anthropic (Direct/BlockRun) | OpenAI (Direct/BlockRun) | Bankr | OpenRouter |
| --- | --- | --- | --- | --- | --- |
| `temperature` | `temperature` | `temperature` | `temperature` | `temperature` | `temperature` |
| `top_p` | `top_p` | `top_p` | `top_p` | `top_p` | `top_p` |
| `top_k` | `top_k` | `top_k` | ✗ (ignored) | ✗ (passthrough) | `top_k` |
| `min_p` | `min_p` | ✗ (use top_p approx) | ✗ (use top_p approx) | ✗ | model-dependent |
| `frequency_penalty` | `frequency_penalty` | ✗ (ignored) | `frequency_penalty` | passthrough | passthrough |
| `presence_penalty` | `presence_penalty` | ✗ (ignored) | `presence_penalty` | passthrough | passthrough |
| `reasoning_effort` | `reasoning.effort` | `thinking.budget_tokens` | `reasoning_effort` | passthrough | model-dependent |
| `visible_thinking` | `reasoning_content` field | `thinking` blocks | `reasoning` field | passthrough | model-dependent |
| `response_schema` | `response_format.json_schema` | `tool_use` workaround | `response_format.json_schema` | passthrough | model-dependent |
| `prompt_cache_key` | `prompt_cache_key` | ✗ (auto by prefix) | ✗ (auto by prefix) | ✗ | ✗ |
| `cache_control` | `cache_control` on blocks | `cache_control` on blocks | ✗ (auto) | ✗ | ✗ |
| `web_search` | `venice_parameters.web_search` | ✗ | ✗ | ✗ | ✗ |
| `tee_mode` | model suffix `-tee` | ✗ | ✗ | ✗ | ✗ |
| `predicted_output` | ✗ | ✗ | `prediction.content` | ✗ | ✗ |
| `seed` | `seed` | ✗ | `seed` | ✗ | `seed` |

### Graceful degradation rules

When a profile field is not supported by the resolved provider:

1. **temperature, top_p, max_tokens**: Always supported. No degradation.
2. **top_k, min_p**: If unsupported, approximate via `top_p`. Log degradation.
3. **reasoning_effort**: If unsupported, map to temperature/prompt adjustments. `High` -> add "Think step by step" to system prompt. `None` -> add "Answer directly without explanation."
4. **response_schema**: If unsupported, fall back to prompt-guided JSON + post-parse validation. See `01-structured-outputs.md`.
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

    // Temperature: always supported
    if let Some(t) = profile.temperature {
        request.set_temperature(t);
    }

    // Reasoning effort: provider-specific mapping
    if let Some(effort) = profile.reasoning_effort {
        if caps.supports_reasoning_effort {
            request.set_reasoning_effort(effort);
        } else if caps.supports_reasoning {
            // Map effort to budget_tokens for Anthropic
            let budget = match effort {
                ReasoningEffort::None => 0,
                ReasoningEffort::Low => 1024,
                ReasoningEffort::Medium => 4096,
                ReasoningEffort::High => 16384,
                ReasoningEffort::Max => 65536,
            };
            request.set_thinking_budget(budget);
        } else {
            // Fallback: prompt-level reasoning guidance
            match effort {
                ReasoningEffort::None => {
                    request.prepend_system("Answer directly. Do not explain your reasoning.");
                }
                ReasoningEffort::High | ReasoningEffort::Max => {
                    request.prepend_system(
                        "Think through this step by step. Show your reasoning."
                    );
                }
                _ => {} // Medium/Low: no modification
            }
            degraded.push(format!("reasoning_effort:{:?} -> prompt fallback", effort));
        }
    }

    // Structured output: see 01-structured-outputs.md for full logic
    if let Some(ref schema) = profile.response_schema {
        if caps.supports_response_schema {
            request.set_response_format(schema);
        } else {
            // Fallback: inject schema into prompt, parse output
            request.append_system(&format!(
                "\n\nRespond ONLY with valid JSON matching this schema:\n{}",
                serde_json::to_string_pretty(&schema.schema).unwrap()
            ));
            degraded.push("response_schema -> prompt-guided JSON".into());
        }
    }

    // Min-p: Venice and some open models only
    if let Some(mp) = profile.min_p {
        if caps.supports_min_p {
            request.set_min_p(mp);
        } else {
            // Approximate: min_p 0.1 ≈ top_p 0.9
            let approx_top_p = 1.0 - mp;
            request.set_top_p(approx_top_p);
            degraded.push(format!("min_p:{} -> top_p:{}", mp, approx_top_p));
        }
    }

    // Prompt caching: Venice and Anthropic
    if let Some(ref key) = profile.prompt_cache_key {
        if caps.supports_prompt_cache_key {
            request.set_prompt_cache_key(key);
        }
        // No degradation — just cost impact
    }

    if profile.cache_control.unwrap_or(false) {
        if caps.supports_cache_control {
            request.add_cache_control_markers();
        }
    }

    // Web search: Venice only
    if profile.web_search.unwrap_or(false) {
        if caps.supports_web_search {
            request.set_web_search(true);
        } else {
            degraded.push("web_search -> unsupported".into());
        }
    }

    // Predicted output: OpenAI only
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

## Subsystem Parameter Table

The master table. Every subsystem, every parameter, with rationale.

### Waking Subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Rationale |
| --- | --- | --- | --- | --- | --- | --- |
| **heartbeat_t1** | 0.3 | top_p=0.9 | Low | `HeartbeatDecision` schema | cache_control | Fast, cheap, focused. Low creativity needed. Structured output extracts action/severity cleanly. |
| **heartbeat_t2** | 0.5 | top_p=0.95 | High | `HeartbeatDecision` schema | cache_control | Novel situation needs deeper reasoning. Higher temp allows consideration of less obvious options. |
| **risk** | 0.1 | top_p=0.85, top_k=40 | Max | `RiskAssessment` schema | cache_control | Maximum precision. Near-deterministic. Structured output ensures all five risk layers are evaluated. Never degraded by mortality pressure. |
| **daimon** | 0.4 | top_p=0.9 | Low | `DaimonAppraisal` schema | none | Emotional appraisal needs consistency but not rigidity. PAD vector extraction via structured output. Privacy preferred (Venice). |
| **daimon_complex** | 0.6 | top_p=0.95 | High | `DaimonAppraisal` schema | none | Complex emotional situations need deeper processing. Visible thinking captures reasoning chain. Privacy required (Venice). |
| **curator** | 0.3 | top_p=0.9 | Medium | `CuratorEvaluation` schema | cache_control | Systematic evaluation. Structured output extracts quality scores, retention decisions, cross-references. |
| **playbook** | 0.4 | top_p=0.9 | Medium | None (free text) | predicted_output | PLAYBOOK.md edits are free-text diffs. OpenAI's predicted_output saves tokens by diffing against current PLAYBOOK. |
| **operator** | 0.7 | top_p=0.95 | High | None (free text) | cache_control | Owner chat. Natural language, higher creativity for explanations. Never degraded by mortality pressure. |
| **mind_wandering** | 0.8 | min_p=0.1 | None | None (free text) | none | Brief reverie during waking. Loosened constraints. Cheap (T0/T1). No reasoning overhead needed. |

### Dream Subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Special |
| --- | --- | --- | --- | --- | --- | --- |
| **dream_nrem** (replay) | 0.4 | top_p=0.9 | Medium | `ReplayAnalysis` schema | prompt_cache_key per cycle | Systematic replay. Structured output extracts lessons, surprise scores, counterfactual markers. |
| **dream_rem** (imagination) | 0.9 | min_p=0.1 | High | None (free text) | prompt_cache_key | Creative scenario generation. High temperature + min-p for principled diversity. Web search enabled (Venice). |
| **dream_rem_creative** | 1.2 | min_p=0.08 | Medium | None (free text) | none | Boden-mode creative recombination. Highest temperature in the waking/dream cycle. |
| **dream_integration** | 0.3 | top_p=0.85 | High | `DreamIntegration` schema | tee_mode | Consolidation. Analytical. Structured output extracts promoted/staged/discarded decisions with rationale. TEE for attestation. |
| **dream_threat** | 0.5 | top_p=0.9 | High | `ThreatAssessment` schema | prompt_cache_key | Threat rehearsal. Balanced creativity (to imagine novel attacks) with analytical depth. |

### Hypnagogic Subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Special |
| --- | --- | --- | --- | --- | --- | --- |
| **hypnagogic_induction** | 1.0-1.2 | min_p=0.1 | None | None (free text) | none | Initial associative scan. Executive loosening. No reasoning -- raw association. Temperature ramps within session. |
| **hypnagogic_dali** | 1.2-1.5 | min_p=0.08 | None | None (free text) | none | Peak creative range. Dali interrupt: 50-100 token partials at max temperature. Highest temp in the entire system. |
| **hypnagogic_observer** | 0.3 | top_p=0.85 | None | `FragmentEvaluation` schema | none | HomuncularObserver. Analytical evaluation of fragments. Structured output for novelty/relevance/coherence scores. Cheapest tier (T0). |
| **hypnagogic_capture** | 0.5 | top_p=0.9 | Low | `CaptureResult` schema | none | Lucid capture. Moderate analytical. Structured output for promote/stage/discard decisions. |
| **hypnopompic_return** | 0.6 | top_p=0.9 | Low | None (free text) | none | Gradual re-engagement. Slightly creative to allow dream insights to surface before full analytical reassertion. |

### Terminal Subsystems

| Subsystem | Temperature | Sampling | Reasoning Effort | Structured Output | Cache | Special |
| --- | --- | --- | --- | --- | --- | --- |
| **death_reflect** | 0.5 | top_p=0.95 | Max | None (free text) | none | Death Protocol Phase II. Maximum reasoning for honest self-assessment. Free text for narrative quality. Visible thinking required. Venice (privacy + visible thinking). |
| **death_testament** | 0.4 | top_p=0.9 | High | `DeathTestament` schema (partial) | tee_mode | Death Protocol Phase III. Structured for machine-parseable sections (metrics, heuristics, warnings). Free text for reflection narrative. TEE for sealed attestation. |

---

## Temperature Scheduling Within Sessions

Some subsystems use **temperature annealing** -- the temperature changes within a single inference session or across a sequence of related calls. This is distinct from the static per-subsystem temperature above.

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

## Reasoning Effort Policies

### When to use each level

| Level | Cost Multiplier | When | Example Subsystems |
| --- | --- | --- | --- |
| **None** | 1x (no reasoning tokens) | Simple classification, scoring, fragment generation | hypnagogic_induction, hypnagogic_dali, mind_wandering |
| **Low** | ~1.2x | Routine decisions, emotional appraisals | heartbeat_t1, daimon, hypnagogic_capture |
| **Medium** | ~1.5-2x | Balanced analysis, knowledge evaluation | curator, dream_nrem, playbook |
| **High** | ~2-4x | Complex decisions, creative development, threat analysis | heartbeat_t2, daimon_complex, dream_rem, dream_threat |
| **Max** | ~4-8x | Critical decisions, death reflection, risk assessment | risk, death_reflect |

### Provider mapping for reasoning effort

```rust
/// Map normalized ReasoningEffort to provider-specific parameters.
pub fn map_reasoning_effort(
    effort: ReasoningEffort,
    provider: &str,
    model: &str,
) -> ReasoningParams {
    match provider {
        "venice" => {
            // Venice uses string-valued reasoning_effort
            let level = match effort {
                ReasoningEffort::None => "none",
                ReasoningEffort::Low => "low",
                ReasoningEffort::Medium => "medium",
                ReasoningEffort::High => "high",
                ReasoningEffort::Max => {
                    // "max" only supported on Claude Opus 4.6 via Venice
                    if model.contains("opus-4-6") { "max" } else { "high" }
                }
            };
            ReasoningParams::Venice { effort: level.into() }
        }
        "anthropic" | "blockrun" => {
            // Anthropic uses budget_tokens for extended thinking
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
            // OpenAI uses string-valued reasoning_effort
            let level = match effort {
                ReasoningEffort::None => "none",
                ReasoningEffort::Low => "low",
                ReasoningEffort::Medium => "medium",
                ReasoningEffort::High => "high",
                ReasoningEffort::Max => "xhigh", // OpenAI's "extra high"
            };
            ReasoningParams::OpenAI { effort: level.into() }
        }
        "bankr" => {
            // Bankr passes through to underlying provider.
            // Detect underlying provider from model name.
            if model.starts_with("claude") {
                map_reasoning_effort(effort, "anthropic", model)
            } else if model.starts_with("gpt") {
                map_reasoning_effort(effort, "openai", model)
            } else {
                // Gemini, Kimi, Qwen: use low/medium/high if supported
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

---

## Prompt Caching Strategy

### Session-level caching

Every Golem uses a persistent `prompt_cache_key` derived from its Golem ID. This ensures that sequential inference calls within the same Golem's lifecycle hit the same server with warm cache, maximizing cache hit rates for the system prompt, PLAYBOOK.md, and STRATEGY.md that are present in every call.

```rust
/// Generate prompt cache key for a Golem.
pub fn golem_cache_key(golem_id: &str, subsystem: &str) -> String {
    // Group by subsystem to maximize prefix sharing.
    // All heartbeat calls share one cache; all dream calls share another.
    format!("golem-{}-{}", golem_id, subsystem)
}
```

### Cache-aligned prompt structure

The 8-layer context engineering pipeline (see `04-context-engineering.md`) already optimizes for cache hits by placing static content first. The InferenceProfile reinforces this:

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

## Venice-Specific Deep Integration

### Web search during REM dreams

Venice's web search feature is enabled during the REM imagination phase to allow the Golem to incorporate current market information into its creative scenario generation. This is controlled by `DreamVeniceConfig.web_search_enabled` and capped by `web_search_budget_per_cycle_usdc`.

```rust
/// Build REM inference profile with web search.
pub fn rem_profile(config: &DreamVeniceConfig) -> InferenceProfile {
    InferenceProfile {
        temperature: Some(0.9),
        min_p: Some(0.1),
        reasoning_effort: Some(ReasoningEffort::High),
        visible_thinking: Some(true), // Capture reasoning chain
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
        // Partial structured output: metrics + heuristics are structured,
        // narrative reflection is free text.
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
                // fastembed-rs with nomic-embed-text-v1.5
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

## Integration: How Profiles Flow Through the System

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

### Events emitted

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

## Configuration

```toml
# bardo.toml -- inference profile overrides
[inference.profiles]
# Override any subsystem's default profile.
# Unspecified fields use the defaults from this document.

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

### Locked profiles

The following subsystems have locked profiles that cannot be overridden by owner configuration:

- **risk**: Temperature 0.1, reasoning Max. Safety-critical. Always maximum precision.
- **death_reflect**: Temperature 0.5, reasoning Max. The Golem's final honest self-assessment cannot be constrained.
- **operator**: Temperature 0.7, reasoning High. Owner communication quality is never degraded.

Attempting to override a locked profile logs a warning and uses the locked defaults.

---

## Cross-References

| Topic | Document | What it covers |
| --- | --- | --- |
| Model routing and provider resolution | [01a-routing.md](01a-routing.md) | Self-describing providers, declarative intents, and the full routing algorithm including subsystem intent declarations |
| Structured output schemas | [01-structured-outputs.md](01-structured-outputs.md) (companion doc) | StructuredOutput abstraction across providers with JSON Schema enforcement and graceful degradation |
| Context engineering pipeline | [04-context-engineering.md](04-context-engineering.md) | 8-layer pipeline where inference profile parameters interact with prompt cache alignment and tool pruning |
| Venice provider specification | [12-providers.md](12-providers.md) section Venice | Venice private cognition plane: TEE attestation, E2EE inference, sensitivity classification, and DIEM staking |
| Hypnagogic temperature scheduling | [../06-hypnagogia/02-architecture.md](../06-hypnagogia/02-architecture.md) | Temperature ramp during dream-cycle onset/return transitions between waking and sleep states |
| Dream cycle phases | [../05-dreams/01-architecture.md](../05-dreams/01-architecture.md) | NREM memory consolidation, REM creative recombination, and integration insight promotion phases |
| Mortality pressure on inference | [01a-routing.md](01a-routing.md) section Mortality | How dying Golems become more cost-sensitive: quality downgrade, cost_sensitivity increase, exempt subsystems |
| Daimon emotional appraisal | [../03-daimon/01-appraisal.md](../03-daimon/01-appraisal.md) | The Daimon's emotional regulation subsystem: PAD vector extraction, appraisal schemas, and affective bias |

---
