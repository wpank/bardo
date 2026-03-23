# Structured Outputs: Type-Safe Inference Across Providers

> **Version**: 1.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Parent**: `prd2/11-inference/`
>
> **Crate**: `golem-inference`
>
> **Depends on**: `golem-core`, `bardo-gateway`, `bardo-providers`
>
> **Purpose**: Define how the Golem uses structured output schemas (JSON Schema `response_format`) to extract typed, machine-parseable data from LLM responses -- across every subsystem that benefits from it, with graceful degradation when the resolved provider does not support schema enforcement. Written for a first-time reader.

---

> **Reader orientation:** This document specifies the structured output system for Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes how Golems extract typed, machine-parseable data from LLM responses using JSON Schema `response_format`, with graceful degradation when the resolved provider does not support schema enforcement. The key concept is that a free-text response like "swap 1 ETH for USDC" is not executable -- structured outputs eliminate the re-parsing failure mode by getting the model to produce validated structures directly. For term definitions, see `prd2/shared/glossary.md`.

## Why Structured Outputs Matter for Golems

A Golem is a Rust binary. Its subsystems communicate through typed structs, enums, and events. When the LLM returns free text, someone has to parse that text into typed data -- and parsing free text is fragile, expensive, and error-prone. A response like "I think the risk is moderate, maybe around 0.6, and the position should probably be reduced" requires regex extraction, heuristic parsing, and fallback logic for every possible phrasing variant.

Structured outputs eliminate this. When the provider supports `response_format` with a JSON Schema, the model is *constrained* to produce output that conforms to the schema. The response is valid JSON with guaranteed fields, types, and structure. The Golem deserializes it directly into a Rust struct with `serde_json::from_str`. No parsing. No regex. No heuristics.

Venice, OpenAI, and several open models (via Venice/OpenRouter) support structured outputs natively. Anthropic Claude supports structured outputs through the tool_use mechanism. Bankr passes through to the underlying provider's capabilities. The challenge is making structured outputs available everywhere they help while degrading gracefully when the provider doesn't support them.

---

## The StructuredOutput Abstraction

### Design principle: optional enhancement, not requirement

Every subsystem that uses structured outputs must also work without them. The structured schema is an *optimization* -- it produces cleaner data with less post-processing -- but the subsystem falls back to prompt-guided JSON + parsing when the provider doesn't support schema enforcement. This avoids spaghetti code: the subsystem always works, it just works *better* with structured outputs.

```rust
/// The StructuredOutput trait: subsystems implement this to declare
/// their preferred response schema and provide fallback parsing.
pub trait StructuredOutput: Sized {
    /// The JSON Schema for this response type.
    /// Used with providers that support response_format.
    fn schema() -> ResponseSchema;

    /// Deserialize from schema-enforced JSON.
    /// This is the fast path: the response is guaranteed valid JSON.
    fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Deserialize from free-text response (fallback path).
    /// The subsystem extracts structured data from natural language.
    /// Implementations should be robust to formatting variations.
    fn from_text(text: &str) -> Result<Self>;

    /// Generate the prompt suffix that guides the model to produce
    /// JSON-like output when schema enforcement is unavailable.
    fn prompt_guidance() -> String {
        format!(
            "\n\nRespond ONLY with valid JSON matching this structure:\n```json\n{}\n```\nDo not include any text outside the JSON block.",
            serde_json::to_string_pretty(&Self::schema().schema).unwrap_or_default()
        )
    }
}

/// Unified deserialization: try schema-enforced first, fall back to text.
pub fn parse_response<T: StructuredOutput>(
    response: &str,
    schema_enforced: bool,
) -> Result<T> {
    if schema_enforced {
        T::from_json(response)
    } else {
        // Try JSON extraction from free text
        if let Some(json_block) = extract_json_block(response) {
            match T::from_json(json_block) {
                Ok(parsed) => return Ok(parsed),
                Err(_) => {} // Fall through to text parsing
            }
        }
        T::from_text(response)
    }
}

/// Extract a JSON block from free text (```json ... ``` or raw { ... }).
fn extract_json_block(text: &str) -> Option<&str> {
    // Try fenced code block first
    if let Some(start) = text.find("```json") {
        let content_start = start + 7;
        if let Some(end) = text[content_start..].find("```") {
            return Some(text[content_start..content_start + end].trim());
        }
    }
    // Try raw JSON object
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Some(&text[start..=end]);
        }
    }
    None
}
```

### How the gateway decides

```rust
/// Decide whether to use schema enforcement or prompt guidance.
pub fn resolve_structured_output(
    profile: &InferenceProfile,
    provider_caps: &ProviderCapabilities,
) -> StructuredOutputMode {
    match &profile.response_schema {
        None => StructuredOutputMode::None,
        Some(schema) => {
            if provider_caps.supports_response_schema {
                StructuredOutputMode::SchemaEnforced(schema.clone())
            } else if provider_caps.supports_tool_use {
                // Anthropic workaround: use tool_use with a single tool
                // whose input_schema matches our response schema.
                StructuredOutputMode::ToolUseWorkaround(schema.clone())
            } else {
                StructuredOutputMode::PromptGuided(schema.clone())
            }
        }
    }
}

pub enum StructuredOutputMode {
    /// No structured output requested.
    None,
    /// Provider enforces JSON Schema via response_format.
    /// Venice, OpenAI, and compatible models.
    SchemaEnforced(ResponseSchema),
    /// Anthropic: use tool_use with a synthetic tool to enforce schema.
    ToolUseWorkaround(ResponseSchema),
    /// Provider doesn't support either. Inject schema into prompt,
    /// parse response with from_text fallback.
    PromptGuided(ResponseSchema),
}
```

### Provider support matrix

| Provider | Schema Enforcement | Tool Use Workaround | Notes |
| --- | --- | --- | --- |
| **Venice** (uncensored, Llama, Qwen) | yes `response_format.json_schema` | yes | `supportsResponseSchema` flag on model. `strict: true` required. |
| **Venice** (Claude via Venice) | no | yes tool_use | Claude doesn't support response_format natively; use tool_use. |
| **Venice** (DeepSeek R1) | no | no | Reasoning models often incompatible. Use prompt-guided. |
| **Anthropic** (Direct/BlockRun) | no | yes tool_use | Anthropic's structured output path is tool_use with a synthetic tool. |
| **OpenAI** (Direct/BlockRun) | yes `response_format.json_schema` | yes | Full support. `strict: true`. |
| **Bankr** | Passthrough | Passthrough | Depends on underlying provider (Claude via Vertex, GPT via OpenRouter). |
| **OpenRouter** | Model-dependent | Model-dependent | Check model capabilities at resolution time. |

---

## Subsystem Schemas

### HeartbeatDecision

Used by: `heartbeat_t1`, `heartbeat_t2`

```rust
/// The heartbeat's inference output, structured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatDecision {
    /// Current assessment severity: none, low, moderate, high, critical.
    pub severity: String,
    /// Recommended action: observe, analyze, rebalance, hedge, exit, escalate.
    pub action: String,
    /// Confidence in this assessment (0.0-1.0).
    pub confidence: f64,
    /// Brief rationale (1-2 sentences).
    pub rationale: String,
    /// Specific signals that drove this decision.
    pub signals: Vec<Signal>,
    /// Whether to escalate to a higher tier (T1->T2).
    pub escalate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub source: String,    // "price", "volume", "governance", "liquidity"
    pub description: String,
    pub magnitude: f64,    // 0.0-1.0
}

impl StructuredOutput for HeartbeatDecision {
    fn schema() -> ResponseSchema {
        ResponseSchema {
            name: "heartbeat_decision".into(),
            strict: true,
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "severity": {
                        "type": "string",
                        "enum": ["none", "low", "moderate", "high", "critical"]
                    },
                    "action": {
                        "type": "string",
                        "enum": ["observe", "analyze", "rebalance", "hedge", "exit", "escalate"]
                    },
                    "confidence": { "type": "number" },
                    "rationale": { "type": "string" },
                    "signals": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "source": { "type": "string" },
                                "description": { "type": "string" },
                                "magnitude": { "type": "number" }
                            },
                            "required": ["source", "description", "magnitude"],
                            "additionalProperties": false
                        }
                    },
                    "escalate": { "type": "boolean" }
                },
                "required": ["severity", "action", "confidence", "rationale", "signals", "escalate"],
                "additionalProperties": false
            }),
        }
    }

    fn from_text(text: &str) -> Result<Self> {
        // Fallback: extract key fields from natural language.
        // This is robust but lossy -- signals may be incomplete.
        let severity = if text.contains("critical") { "critical" }
            else if text.contains("high") { "high" }
            else if text.contains("moderate") { "moderate" }
            else if text.contains("low") { "low" }
            else { "none" };

        let action = if text.contains("exit") { "exit" }
            else if text.contains("hedge") { "hedge" }
            else if text.contains("rebalance") { "rebalance" }
            else if text.contains("escalate") { "escalate" }
            else if text.contains("analyze") { "analyze" }
            else { "observe" };

        Ok(HeartbeatDecision {
            severity: severity.into(),
            action: action.into(),
            confidence: 0.5, // Default when not extractable
            rationale: text.chars().take(200).collect(),
            signals: vec![],
            escalate: text.contains("escalate"),
        })
    }
}
```

### DaimonAppraisal

Used by: `daimon`, `daimon_complex`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaimonAppraisal {
    /// Pleasure dimension (-1.0 to 1.0).
    pub pleasure: f64,
    /// Arousal dimension (-1.0 to 1.0).
    pub arousal: f64,
    /// Dominance dimension (-1.0 to 1.0).
    pub dominance: f64,
    /// Primary Plutchik emotion label.
    pub primary_emotion: String,
    /// Optional secondary emotion (blended states).
    pub secondary_emotion: Option<String>,
    /// What triggered this appraisal.
    pub trigger: String,
    /// How this should bias memory retrieval.
    pub memory_bias: String,
}

impl StructuredOutput for DaimonAppraisal {
    fn schema() -> ResponseSchema {
        ResponseSchema {
            name: "daimon_appraisal".into(),
            strict: true,
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pleasure": { "type": "number" },
                    "arousal": { "type": "number" },
                    "dominance": { "type": "number" },
                    "primary_emotion": {
                        "type": "string",
                        "enum": ["joy", "trust", "fear", "surprise",
                                 "sadness", "disgust", "anger", "anticipation"]
                    },
                    "secondary_emotion": { "type": ["string", "null"] },
                    "trigger": { "type": "string" },
                    "memory_bias": { "type": "string" }
                },
                "required": ["pleasure", "arousal", "dominance",
                            "primary_emotion", "secondary_emotion",
                            "trigger", "memory_bias"],
                "additionalProperties": false
            }),
        }
    }

    fn from_text(text: &str) -> Result<Self> {
        // Extract PAD values from text like "pleasure: 0.3, arousal: -0.5"
        // or from narrative descriptions
        let pleasure = extract_float(text, "pleasure").unwrap_or(0.0);
        let arousal = extract_float(text, "arousal").unwrap_or(0.0);
        let dominance = extract_float(text, "dominance").unwrap_or(0.0);

        let emotions = ["joy", "trust", "fear", "surprise",
                       "sadness", "disgust", "anger", "anticipation"];
        let primary = emotions.iter()
            .find(|e| text.to_lowercase().contains(*e))
            .unwrap_or(&"anticipation");

        Ok(DaimonAppraisal {
            pleasure, arousal, dominance,
            primary_emotion: primary.to_string(),
            secondary_emotion: None,
            trigger: text.chars().take(100).collect(),
            memory_bias: "neutral".into(),
        })
    }
}
```

### RiskAssessment

Used by: `risk`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall approval decision.
    pub approved: bool,
    /// Adjusted position size (may be less than requested).
    pub adjusted_size_usd: f64,
    /// Per-layer evaluation results.
    pub layers: RiskLayers,
    /// Warnings (non-blocking concerns).
    pub warnings: Vec<String>,
    /// Reasoning chain (if visible thinking enabled).
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLayers {
    pub hard_shields: LayerResult,
    pub position_sizing: LayerResult,
    pub adaptive_guardrails: LayerResult,
    pub observation: LayerResult,
    pub defi_threats: LayerResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    pub passed: bool,
    pub score: f64,
    pub detail: String,
}
```

### CuratorEvaluation

Used by: `curator`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratorEvaluation {
    /// Entries evaluated this cycle.
    pub entries_evaluated: u32,
    /// Entries that passed validation.
    pub entries_retained: u32,
    /// Entries pruned (below confidence threshold).
    pub entries_pruned: u32,
    /// Cross-references discovered.
    pub cross_references: Vec<CrossRef>,
    /// Entries promoted from inherited to validated.
    pub promotions: Vec<Promotion>,
    /// Overall Grimoire health score.
    pub grimoire_health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRef {
    pub source_id: String,
    pub target_id: String,
    pub relationship: String, // "supports", "contradicts", "extends"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    pub entry_id: String,
    pub from_confidence: f64,
    pub to_confidence: f64,
    pub reason: String,
}
```

### ReplayAnalysis (Dream NREM)

Used by: `dream_nrem`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayAnalysis {
    /// Episode being replayed.
    pub episode_id: String,
    /// What the Golem learned from this replay.
    pub lessons: Vec<Lesson>,
    /// Surprise score: how much the outcome differed from prediction.
    pub surprise_score: f64,
    /// Counterfactual markers: what could have been done differently.
    pub counterfactuals: Vec<Counterfactual>,
    /// Emotional depotentiation: has arousal decreased?
    pub arousal_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub insight: String,
    pub confidence: f64,
    pub applicable_conditions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterfactual {
    pub alternative_action: String,
    pub estimated_outcome: String,
    pub plausibility: f64,
}
```

### FragmentEvaluation (Hypnagogic Observer)

Used by: `hypnagogic_observer`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentEvaluation {
    pub novelty: f64,
    pub relevance: f64,
    pub coherence: f64,
    pub verdict: String, // "promote", "stage", "discard"
}
```

### DreamIntegration

Used by: `dream_integration`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamIntegration {
    /// Hypotheses promoted to PLAYBOOK staging.
    pub promoted: Vec<IntegrationItem>,
    /// Hypotheses staged for further development.
    pub staged: Vec<IntegrationItem>,
    /// Hypotheses discarded.
    pub discarded: Vec<IntegrationItem>,
    /// PLAYBOOK.md diff (additions/modifications).
    pub playbook_diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationItem {
    pub hypothesis_id: String,
    pub summary: String,
    pub confidence: f64,
    pub rationale: String,
}
```

### ThreatAssessment (Dream Threats)

Used by: `dream_threat`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub threat_type: String,
    pub severity: String,       // "low", "medium", "high", "critical"
    pub likelihood: f64,
    pub impact_if_realized: String,
    pub mitigation_actions: Vec<String>,
    pub detection_signals: Vec<String>,
    pub rehearsal_outcome: String,
}
```

---

## The Anthropic Tool-Use Workaround

Anthropic Claude does not support `response_format` with JSON Schema. Instead, the gateway uses a synthetic tool definition whose `input_schema` matches the desired output schema. The model "calls" this tool with structured parameters, which are the structured output.

```rust
/// Convert a ResponseSchema into an Anthropic tool_use call.
pub fn schema_to_anthropic_tool(schema: &ResponseSchema) -> Tool {
    Tool {
        name: format!("respond_{}", schema.name),
        description: format!(
            "Use this tool to provide your structured response. \
             Fill in all required fields based on your analysis."
        ),
        input_schema: schema.schema.clone(),
    }
}

/// Extract structured output from an Anthropic tool_use response.
pub fn extract_from_tool_use(response: &AnthropicResponse) -> Option<String> {
    for block in &response.content {
        if let ContentBlock::ToolUse { input, .. } = block {
            return Some(serde_json::to_string(input).ok()?);
        }
    }
    None
}
```

This workaround is transparent to the subsystem: it calls `parse_response::<T>()` regardless of provider, and the gateway handles the translation.

---

## Validation and Error Handling

### Post-response validation

Even with schema enforcement, the *content* may be wrong (a valid JSON object with incorrect values). The gateway validates:

1. **Schema conformance**: The JSON matches the schema (guaranteed by schema enforcement, verified for prompt-guided).
2. **Range checks**: Numeric fields are within expected ranges (e.g., `confidence` is 0.0-1.0, PAD values are -1.0 to 1.0).
3. **Enum membership**: String enums contain valid values.

```rust
/// Post-parse validation for structured outputs.
pub fn validate<T: StructuredOutput + Validate>(parsed: &T) -> ValidationResult {
    let mut issues = Vec::new();
    parsed.validate(&mut issues);
    if issues.is_empty() {
        ValidationResult::Valid
    } else {
        ValidationResult::InvalidContent(issues)
    }
}

pub trait Validate {
    fn validate(&self, issues: &mut Vec<String>);
}

impl Validate for DaimonAppraisal {
    fn validate(&self, issues: &mut Vec<String>) {
        if self.pleasure < -1.0 || self.pleasure > 1.0 {
            issues.push(format!("pleasure {} out of range [-1,1]", self.pleasure));
        }
        if self.arousal < -1.0 || self.arousal > 1.0 {
            issues.push(format!("arousal {} out of range [-1,1]", self.arousal));
        }
        if self.dominance < -1.0 || self.dominance > 1.0 {
            issues.push(format!("dominance {} out of range [-1,1]", self.dominance));
        }
    }
}
```

### Retry logic

If validation fails on a schema-enforced response (content error, not schema error), the subsystem may retry once with additional prompt guidance specifying the validation failure. If the retry also fails, the subsystem falls back to default values and logs a warning.

---

## Cost Impact

Structured outputs have a cost profile:

| Mode | First-call Latency | Subsequent Latency | Token Overhead |
| --- | --- | --- | --- |
| Schema enforced (Venice/OpenAI) | +200-500ms (schema compilation) | Normal | ~5-10% fewer output tokens (no formatting) |
| Tool-use workaround (Anthropic) | Normal | Normal | ~10% more tokens (tool definition) |
| Prompt-guided fallback | Normal | Normal | ~15-20% more tokens (schema in prompt + formatting) |

The schema-enforced path is cheapest after the first call (schemas are cached per-session). The prompt-guided fallback is most expensive but universally available.

---

## Configuration

```toml
# bardo.toml -- structured output configuration

[inference.structured_outputs]
# Enable structured outputs globally. When false, all subsystems
# use free-text responses with from_text parsing.
enabled = true

# Prefer schema enforcement over tool-use workaround.
# When true and both are available, use response_format.
# When false, always use tool-use (more compatible but costlier).
prefer_schema_enforcement = true

# Enable post-parse validation for all structured responses.
validation = true

# Retry on validation failure (content errors, not schema errors).
retry_on_validation_failure = true
max_retries = 1
```

---

## Cross-References

| Topic | Document | What it covers |
| --- | --- | --- |
| Inference parameter policies | [00-inference-parameters.md](00-inference-parameters.md) | Per-subsystem temperature, sampling, and reasoning effort policies that determine when structured outputs are used |
| Model routing | [01a-routing.md](01a-routing.md) | Self-describing providers and intent resolution; structured_outputs is a preferred feature in several subsystem intents |
| Venice structured outputs guide | [Venice docs: Structured Responses](https://docs.venice.ai/overview/guides/structured-responses) | Venice's response_format JSON schema mode for safety-critical cognition outputs |
| Anthropic tool use | [Anthropic docs: Tool Use](https://docs.anthropic.com/en/docs/build-with-claude/tool-use) | Anthropic's tool-use workaround for structured outputs (Claude produces structured JSON via tool call, not response_format) |
| OpenAI structured outputs | [OpenAI docs: Structured Outputs](https://platform.openai.com/docs/guides/structured-outputs) | OpenAI's native JSON schema enforcement with strict mode for guaranteed schema compliance |
| Risk engine assessment | [../01-golem/00-overview.md](../01-golem/00-overview.md) section 8 | The Golem's risk assessment subsystem that produces RiskAssessment structured outputs for trade decisions |
| Daimon appraisal | [../03-daimon/01-appraisal.md](../03-daimon/01-appraisal.md) | The Daimon's emotional regulation subsystem producing DaimonAppraisal structured outputs with PAD vectors |
| Dream consolidation | [../05-dreams/04-consolidation.md](../05-dreams/04-consolidation.md) | Dream insight promotion where structured outputs extract confidence scores and knowledge entry types |
| Hypnagogic observer | [../06-hypnagogia/04-homunculus.md](../06-hypnagogia/04-homunculus.md) | The liminal observer that evaluates dream fragments using structured quality assessments |

---
