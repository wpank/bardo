# 13 -- Unified Reasoning Chain Integration [SPEC]

**Provider-Agnostic Reasoning Normalization and Streaming**

Cross-references: [12-providers.md](./12-providers.md) (five provider backends with per-provider reasoning format details) | [14-rust-implementation.md](./14-rust-implementation.md) (10-crate Rust workspace where the reasoning parser lives) | [01a-routing.md](./01a-routing.md) (model routing and reasoning effort mapping per subsystem) | [Cognition Model](../01-golem/01-cognition.md) (the Golem's cognitive architecture and subsystem decomposition)

---

> **Reader orientation:** This document specifies the unified reasoning chain integration for Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes how the gateway normalizes reasoning outputs from five different provider formats (Anthropic thinking blocks, DeepSeek `<think>` tags, OpenAI reasoning items, Gemini server-side thinking, Qwen thinking toggles) into a single `ReasoningChain` type. The key concept is that subsystems should not need to know which provider produced a reasoning trace. For term definitions, see `prd2/shared/glossary.md`.

## 1. The Problem
Every provider exposes reasoning differently. Anthropic returns `thinking` content blocks (visible or redacted), interleaved with tool-use blocks. DeepSeek wraps reasoning in inline `<think>...</think>` tags in the response stream -- raw SSE bytes containing markup that must be stripped. OpenAI's Responses API returns `reasoning` items with an optional `summary` field. Gemini performs server-side thinking that is never returned -- you get token counts but no content. Qwen uses the same `<think>` tag syntax as DeepSeek with `enable_thinking` and `thinking_budget` parameters. OpenRouter wraps everything into `reasoning_details`, but the underlying structure depends on which provider fulfilled the request.

A Pi subsystem that wants to inspect, log, or render reasoning should not need to know which provider produced it. Bardo normalizes all formats into a single `ReasoningChain` type with a streaming parser that handles byte-level tag detection across SSE chunk boundaries.

---

## 2. ReasoningChain

```rust
#[derive(Debug)]
pub struct ReasoningChain {
    pub provider: String,
    pub model: String,
    pub visibility: ReasoningVisibility,
    pub content: Option<String>,
    pub reasoning_tokens: u32,
    pub interleaved: bool,
    pub steps: Vec<ReasoningStep>,
    pub cost: ReasoningCost,
}

#[derive(Debug)]
pub enum ReasoningVisibility {
    Visible,     // Full text available (DeepSeek R1, Qwen /think)
    Summarized,  // Summary only (OpenAI, Claude summarized)
    Opaque,      // Reasoning happened, content not returned (Gemini, Claude redacted)
    None,        // No reasoning performed
}

#[derive(Debug)]
pub struct ReasoningStep {
    pub step_type: ReasoningStepType,
    pub content: String,
    pub confidence: Option<f64>,
}

#[derive(Debug)]
pub enum ReasoningStepType {
    Analysis, Planning, Evaluation, Reflection, Uncertainty, Decision,
}

#[derive(Debug)]
pub struct ReasoningCost {
    pub reasoning_tokens: u32,
    pub billed_as_output: bool,
    pub discount_factor: f64,  // 1.0 = full price; exists for future cached reasoning discounts
}
```

---

## 3. ReasoningParser Trait
```rust
#[derive(Debug)]
pub struct RawResponse {
    pub provider: String,
    pub model: String,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
}
#[derive(Debug)]
pub enum ReasoningEvent {
    ThinkStart,
    ThinkChunk(Vec<u8>),
    ThinkEnd,
    ContentChunk(Vec<u8>),
    UsageUpdate { reasoning_tokens: u32 },
}
pub struct ParserState {
    pub reasoning_buffer: Vec<u8>,
    pub content_buffer: Vec<u8>,
    pub in_thinking: bool,
    pub reasoning_tokens_seen: u32,
    pub provider_state: Box<dyn std::any::Any + Send>,
}
pub trait ReasoningParser: Send + Sync {
    fn can_parse(&self, provider: &str, model: &str) -> bool;
    fn parse_response(&self, raw: &RawResponse) -> ReasoningChain;
    fn parse_stream_chunk(&self, chunk: &[u8], state: &mut ParserState) -> Vec<ReasoningEvent>;
}
// Shared builder used by all parser implementations.
fn build_chain(raw: &RawResponse, visibility: ReasoningVisibility, content: Option<String>,
               reasoning_tokens: u32, interleaved: bool,
               billed_as_output: bool, discount_factor: f64) -> ReasoningChain {
    ReasoningChain {
        provider: raw.provider.clone(), model: raw.model.clone(),
        visibility, content, reasoning_tokens, interleaved,
        steps: Vec::new(),
        cost: ReasoningCost { reasoning_tokens, billed_as_output, discount_factor },
    }
}
```

### 3.1 AnthropicParser

Extracts from `content_block` SSE events. Handles `thinking` (Visible), `redacted_thinking` (Opaque), and detects interleaving when thinking appears between tool_use blocks.
```rust
#[derive(Debug)]
pub struct AnthropicParser;

impl ReasoningParser for AnthropicParser {
    fn can_parse(&self, provider: &str, _model: &str) -> bool { provider == "anthropic" }

    fn parse_response(&self, raw: &RawResponse) -> ReasoningChain {
        let body: serde_json::Value = serde_json::from_slice(&raw.body).unwrap();
        let blocks = body["content"].as_array().cloned().unwrap_or_default();
        let (mut text, mut vis, mut saw_tool, mut interleaved) =
            (String::new(), ReasoningVisibility::None, false, false);
        for b in &blocks {
            match b["type"].as_str() {
                Some("thinking") => {
                    if saw_tool { interleaved = true; }
                    if let Some(t) = b["thinking"].as_str() {
                        text.push_str(t); vis = ReasoningVisibility::Visible;
                    }
                }
                Some("redacted_thinking") => { vis = ReasoningVisibility::Opaque; }
                Some("tool_use") => { saw_tool = true; }
                _ => {}
            }
        }
        let tokens = body["usage"]["cache_creation_input_tokens"].as_u64().unwrap_or(0) as u32;
        build_chain(raw, vis, nonempty(text), tokens, interleaved, true, 1.0)
    }

    fn parse_stream_chunk(&self, chunk: &[u8], state: &mut ParserState) -> Vec<ReasoningEvent> {
        let mut events = Vec::new();
        for line in String::from_utf8_lossy(chunk).lines() {
            let Some(data) = line.strip_prefix("data: ") else { continue };
            if data == "[DONE]" { continue; }
            let Ok(ev) = serde_json::from_str::<serde_json::Value>(data) else { continue };
            match ev["type"].as_str() {
                Some("content_block_start")
                    if ev["content_block"]["type"].as_str() == Some("thinking") => {
                    state.in_thinking = true; events.push(ReasoningEvent::ThinkStart);
                }
                Some("content_block_delta") if state.in_thinking => {
                    if let Some(t) = ev["delta"]["thinking"].as_str() {
                        let b = t.as_bytes().to_vec();
                        state.reasoning_buffer.extend_from_slice(&b);
                        events.push(ReasoningEvent::ThinkChunk(b));
                    }
                }
                Some("content_block_stop") if state.in_thinking => {
                    state.in_thinking = false; events.push(ReasoningEvent::ThinkEnd);
                }
                _ => {}
            }
        }
        events
    }
}

fn nonempty(s: String) -> Option<String> { if s.is_empty() { None } else { Some(s) } }
```

### 3.2 DeepSeekParser + QwenParser

Both use inline `<think>...</think>` tags. Streaming delegates to `BardoStreamParser` (section 4). Qwen adds `enable_thinking` / `thinking_budget` parameters but the response format is identical.
```rust
#[derive(Debug)]
pub struct DeepSeekParser;
#[derive(Debug)]
pub struct QwenParser;
impl ReasoningParser for DeepSeekParser {
    fn can_parse(&self, p: &str, m: &str) -> bool { p == "deepseek" || m.starts_with("deepseek-r1") }
    fn parse_response(&self, raw: &RawResponse) -> ReasoningChain { parse_think_tag_response(raw) }
    fn parse_stream_chunk(&self, chunk: &[u8], state: &mut ParserState) -> Vec<ReasoningEvent> {
        state.provider_state.downcast_mut::<BardoStreamParser>().unwrap().feed(chunk)
    }
}
impl ReasoningParser for QwenParser {
    fn can_parse(&self, p: &str, m: &str) -> bool { p == "qwen" || m.starts_with("qwen") }
    fn parse_response(&self, raw: &RawResponse) -> ReasoningChain { parse_think_tag_response(raw) }
    fn parse_stream_chunk(&self, chunk: &[u8], state: &mut ParserState) -> Vec<ReasoningEvent> {
        state.provider_state.downcast_mut::<BardoStreamParser>().unwrap().feed(chunk)
    }
}

fn parse_think_tag_response(raw: &RawResponse) -> ReasoningChain {
    let body: serde_json::Value = serde_json::from_slice(&raw.body).unwrap();
    let full = body["choices"][0]["message"]["content"].as_str().unwrap_or("");
    let (reasoning, _clean) = extract_think_tags(full);
    let tokens = body["usage"]["completion_tokens_details"]["reasoning_tokens"]
        .as_u64().unwrap_or(0) as u32;
    let vis = if reasoning.is_some() { ReasoningVisibility::Visible }
              else { ReasoningVisibility::None };
    build_chain(raw, vis, reasoning, tokens, false, true, 1.0)
}

fn extract_think_tags(text: &str) -> (Option<String>, String) {
    let (mut reasoning, mut clean, mut in_think, mut buf) =
        (String::new(), String::new(), false, String::new());
    for ch in text.chars() {
        buf.push(ch);
        if buf.ends_with("<think>") {
            clean.push_str(&buf[..buf.len() - 7]); buf.clear(); in_think = true;
        } else if buf.ends_with("</think>") {
            reasoning.push_str(&buf[..buf.len() - 8]); buf.clear(); in_think = false;
        } else if buf.len() > 8 {
            let flush: String = buf.drain(..buf.len() - 8).collect();
            if in_think { reasoning.push_str(&flush); } else { clean.push_str(&flush); }
        }
    }
    if in_think { reasoning.push_str(&buf); } else { clean.push_str(&buf); }
    (nonempty(reasoning), clean)
}
```

### 3.3 OpenAIParser

Extracts from Responses API `reasoning` items with `summary` field. Effort levels control whether summaries populate.
```rust
#[derive(Debug)]
pub struct OpenAIParser;

impl ReasoningParser for OpenAIParser {
    fn can_parse(&self, p: &str, _m: &str) -> bool { p == "openai" }

    fn parse_response(&self, raw: &RawResponse) -> ReasoningChain {
        let body: serde_json::Value = serde_json::from_slice(&raw.body).unwrap();
        let (mut text, mut vis) = (String::new(), ReasoningVisibility::None);
        if let Some(output) = body["output"].as_array() {
            for item in output.iter().filter(|i| i["type"].as_str() == Some("reasoning")) {
                match item["summary"].as_array() {
                    Some(parts) => {
                        for p in parts { if let Some(t) = p["text"].as_str() { text.push_str(t); } }
                        vis = ReasoningVisibility::Summarized;
                    }
                    None => { vis = ReasoningVisibility::Opaque; }
                }
            }
        }
        let tokens = body["usage"]["output_tokens_details"]["reasoning_tokens"]
            .as_u64().unwrap_or(0) as u32;
        build_chain(raw, vis, nonempty(text), tokens, false, false, 1.0)
    }

    fn parse_stream_chunk(&self, chunk: &[u8], state: &mut ParserState) -> Vec<ReasoningEvent> {
        let mut events = Vec::new();
        for line in String::from_utf8_lossy(chunk).lines() {
            let Some(data) = line.strip_prefix("data: ") else { continue };
            if data == "[DONE]" { continue; }
            let Ok(ev) = serde_json::from_str::<serde_json::Value>(data) else { continue };
            match ev["type"].as_str() {
                Some("response.reasoning.delta") => {
                    if !state.in_thinking { state.in_thinking = true; events.push(ReasoningEvent::ThinkStart); }
                    if let Some(t) = ev["delta"].as_str() {
                        let b = t.as_bytes().to_vec();
                        state.reasoning_buffer.extend_from_slice(&b);
                        events.push(ReasoningEvent::ThinkChunk(b));
                    }
                }
                Some("response.reasoning.done") if state.in_thinking => {
                    state.in_thinking = false; events.push(ReasoningEvent::ThinkEnd);
                }
                _ => {}
            }
        }
        events
    }
}
```

### 3.4 GeminiParser

Server-side only reasoning. Token counts from `usageMetadata.thoughtsTokenCount`.
```rust
#[derive(Debug)]
pub struct GeminiParser;

impl ReasoningParser for GeminiParser {
    fn can_parse(&self, p: &str, _m: &str) -> bool { p == "gemini" || p == "google" }
    fn parse_response(&self, raw: &RawResponse) -> ReasoningChain {
        let body: serde_json::Value = serde_json::from_slice(&raw.body).unwrap();
        let tokens = body["usageMetadata"]["thoughtsTokenCount"].as_u64().unwrap_or(0) as u32;
        let vis = if tokens > 0 { ReasoningVisibility::Opaque } else { ReasoningVisibility::None };
        build_chain(raw, vis, None, tokens, false, false, 0.0)
    }

    fn parse_stream_chunk(&self, _chunk: &[u8], state: &mut ParserState) -> Vec<ReasoningEvent> {
        // Gemini streams contain no reasoning text -- only usage metadata updates
        vec![]
    }
}
```

### 3.5 OpenRouterParser

Wraps the other five parsers. Uses `reasoning_details` when present; otherwise detects upstream provider from model string and delegates.
```rust
pub struct OpenRouterParser { inner: Vec<Box<dyn ReasoningParser>> }

impl OpenRouterParser {
    pub fn new() -> Self {
        Self { inner: vec![
            Box::new(AnthropicParser), Box::new(DeepSeekParser),
            Box::new(QwenParser), Box::new(OpenAIParser), Box::new(GeminiParser),
        ]}
    }
}

impl ReasoningParser for OpenRouterParser {
    fn can_parse(&self, p: &str, _m: &str) -> bool { p == "openrouter" }

    fn parse_response(&self, raw: &RawResponse) -> ReasoningChain {
        let body: serde_json::Value = serde_json::from_slice(&raw.body).unwrap();
        if let Some(details) = body["choices"][0]["message"]["reasoning_details"].as_str() {
            let tokens = body["usage"]["completion_tokens_details"]["reasoning_tokens"]
                .as_u64().unwrap_or(0) as u32;
            return build_chain(raw, ReasoningVisibility::Visible,
                               Some(details.to_string()), tokens, false, true, 1.0);
        }
        let upstream = body["model"].as_str()
            .and_then(|m| m.split('/').next()).unwrap_or("unknown");
        self.inner.iter().find(|p| p.can_parse(upstream, &raw.model))
            .map(|p| p.parse_response(raw))
            .unwrap_or_else(|| build_chain(raw, ReasoningVisibility::None, None, 0, false, false, 1.0))
    }

    fn parse_stream_chunk(&self, chunk: &[u8], state: &mut ParserState) -> Vec<ReasoningEvent> {
        state.provider_state.downcast_mut::<BardoStreamParser>().unwrap().feed(chunk)
    }
}
```

---

## 4. Streaming -- BardoStreamParser

Byte-level state machine for `<think>` / `</think>` tag detection across SSE chunk boundaries. Handles tags split across TCP frames.
```rust
#[derive(Debug)]
pub struct BardoStreamParser {
    state: StreamParserState,
    think_buf: Vec<u8>,
    content_buf: Vec<u8>,
}

#[derive(Debug)]
enum StreamParserState {
    Content,
    MaybeTag { matched: usize },      // matching against b"<think>"
    InThinking,
    MaybeCloseTag { matched: usize },  // matching against b"</think>"
}

const OPEN: &[u8] = b"<think>";
const CLOSE: &[u8] = b"</think>";

impl BardoStreamParser {
    pub fn new() -> Self {
        Self { state: StreamParserState::Content, think_buf: vec![], content_buf: vec![] }
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<ReasoningEvent> {
        let mut events = Vec::new();
        for &byte in chunk {
            match self.state {
                StreamParserState::Content => {
                    if byte == b'<' { self.state = StreamParserState::MaybeTag { matched: 1 }; }
                    else { self.content_buf.push(byte); }
                }
                StreamParserState::MaybeTag { matched } => {
                    if byte == OPEN[matched] {
                        if matched + 1 == OPEN.len() {
                            Self::flush(&mut self.content_buf, &mut events, false);
                            events.push(ReasoningEvent::ThinkStart);
                            self.state = StreamParserState::InThinking;
                        } else { self.state = StreamParserState::MaybeTag { matched: matched + 1 }; }
                    } else {
                        self.content_buf.extend_from_slice(&OPEN[..matched]);
                        self.content_buf.push(byte);
                        self.state = StreamParserState::Content;
                    }
                }
                StreamParserState::InThinking => {
                    if byte == b'<' { self.state = StreamParserState::MaybeCloseTag { matched: 1 }; }
                    else { self.think_buf.push(byte); }
                }
                StreamParserState::MaybeCloseTag { matched } => {
                    if byte == CLOSE[matched] {
                        if matched + 1 == CLOSE.len() {
                            Self::flush(&mut self.think_buf, &mut events, true);
                            events.push(ReasoningEvent::ThinkEnd);
                            self.state = StreamParserState::Content;
                        } else { self.state = StreamParserState::MaybeCloseTag { matched: matched + 1 }; }
                    } else {
                        self.think_buf.extend_from_slice(&CLOSE[..matched]);
                        self.think_buf.push(byte);
                        self.state = StreamParserState::InThinking;
                    }
                }
            }
        }
        if !self.content_buf.is_empty() && matches!(self.state, StreamParserState::Content) {
            Self::flush(&mut self.content_buf, &mut events, false);
        }
        events
    }

    pub fn finish(&mut self) -> Vec<ReasoningEvent> {
        let mut events = Vec::new();
        Self::flush(&mut self.think_buf, &mut events, true);
        Self::flush(&mut self.content_buf, &mut events, false);
        events
    }

    fn flush(buf: &mut Vec<u8>, events: &mut Vec<ReasoningEvent>, is_thinking: bool) {
        if buf.is_empty() { return; }
        let data = buf.drain(..).collect();
        events.push(if is_thinking { ReasoningEvent::ThinkChunk(data) }
                     else { ReasoningEvent::ContentChunk(data) });
    }
}
```

State transitions: `Content -> MaybeTag{1..6} -> InThinking -> MaybeCloseTag{1..7} -> Content`. On mismatch at any point, partially matched bytes flush to the active buffer and state returns to its parent.

---

## 5. Pipeline profiles and reasoning depth

The inference gateway selects a pipeline profile per request, which determines both context engineering depth and reasoning configuration. The four profiles map directly to the subsystem's reasoning requirements:

```rust
/// Pipeline profile determines which context engineering layers run
/// and how reasoning parameters are configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineProfile {
    /// L3 only. <1ms. T0 heartbeat ticks.
    /// No reasoning -- deterministic or fast classifier.
    Minimal,
    /// L1+L3+L6. 5-15ms. Golem internal calls.
    /// Light reasoning (low effort / no thinking).
    Fast,
    /// L1-L6. 20-40ms. User-facing requests.
    /// Standard reasoning with interleaved thinking when available.
    Standard,
    /// L1-L8. 30-65ms. High-security, first-time requests.
    /// Maximum reasoning depth. PII masking + injection detection.
    Full,
}

pub fn select_profile_and_reasoning(
    subsystem: &str,
    security_class: SecurityLevel,
) -> (PipelineProfile, ReasoningConfig) {
    match subsystem {
        "heartbeat_t0" => (PipelineProfile::Minimal, ReasoningConfig::none()),
        "heartbeat_t1" => (PipelineProfile::Fast, ReasoningConfig::low()),
        "heartbeat_t2" => (PipelineProfile::Standard, ReasoningConfig::high()),
        "dream" => (PipelineProfile::Fast, ReasoningConfig::visible_max()),
        "daimon" => (PipelineProfile::Fast, ReasoningConfig::none()),
        "daimon_complex" => (PipelineProfile::Fast, ReasoningConfig::visible_max()),
        "risk" => (PipelineProfile::Standard, ReasoningConfig::interleaved_max()),
        "death" => (PipelineProfile::Full, ReasoningConfig::visible_max()),
        "operator" => (PipelineProfile::Standard, ReasoningConfig::interleaved_high()),
        _ if security_class == SecurityLevel::Private => {
            (PipelineProfile::Full, ReasoningConfig::high())
        }
        _ => (PipelineProfile::Standard, ReasoningConfig::medium()),
    }
}
```

The subsystem's reasoning requirements determine both the pipeline profile (how much optimization happens) and the `ReasoningConfig` (how the model is asked to think). Dream subsystems use `Fast` profile (skip PII masking and injection detection) but `visible_max` reasoning (maximum `<think>` chain depth). Risk uses `Standard` profile but `interleaved_max` reasoning (thinking interleaved with tool calls). The profile and reasoning config are independent axes.

### Reasoning budget and mortality

A declining Golem's reasoning budgets contract. Risk, death, and operator subsystems are exempt.

```rust
pub fn apply_mortality_to_reasoning(
    config: &mut ReasoningConfig,
    subsystem: &str,
    vitality: f64,
) {
    let exempt = ["risk", "death", "operator"];
    if exempt.contains(&subsystem) { return; }

    let pressure = 1.0 - vitality;
    if pressure > 0.7 {
        // Conservation: downgrade reasoning depth
        config.depth = config.depth.downgrade();
    }
    if pressure > 0.9 {
        // Terminal: no reasoning except exempt subsystems
        config.depth = ReasoningDepth::None;
    }
}
```

---

## 6. Surface-specific rendering

| Surface  | Rendering                                                                     |
|----------|-------------------------------------------------------------------------------|
| Web      | Collapsible panel below response, syntax-highlighted thinking text            |
| TUI      | Spinner with `[thinking...]` during reasoning, final answer only in output    |
| Telegram | Suppressed entirely -- too noisy for chat messages                            |
| Discord  | Suppressed by default, optional "Show reasoning" reaction to reveal in thread |
| API      | Full `ReasoningChain` in response body, `X-Bardo-Reasoning-Tokens` header    |

The web panel renders Markdown within thinking content. When `visibility` is `Opaque`, it shows "Reasoning performed (not visible)" with the token count.

---

## 7. Cost implications

| Provider    | Reasoning Token Billing               | Discount | Notes                                                     |
|-------------|----------------------------------------|----------|-----------------------------------------------------------|
| Anthropic   | Billed as output tokens                | None     | Thinking tokens count toward output token total            |
| OpenAI      | Billed separately as reasoning tokens  | None     | Visible in `usage.output_tokens_details.reasoning_tokens`  |
| DeepSeek R1 | Billed as output                       | None     | `<think>` content counts as output tokens                  |
| Gemini      | Not billed separately                  | N/A      | Server-side only, included in base cost                    |
| Qwen        | Billed as output when thinking enabled | None     | `thinking_budget` parameter caps the cost                  |

`billed_as_output` controls whether reasoning tokens count alongside output tokens or sit in a separate bucket. `discount_factor` is 1.0 everywhere today but exists for future cached-reasoning discounts. Reasoning token counts populate the `reasoning_tokens` field in `InferenceLog` (see [07-safety.md](07-safety.md)), connecting per-request cost attribution to the audit trail.

---

## 8. Structured outputs: type-safe inference across providers

A Golem is a Rust binary. Its subsystems communicate through typed structs, enums, and events. When the LLM returns free text, someone has to parse that text into typed data -- and parsing free text is fragile, expensive, and error-prone. A response like "I think the risk is moderate, maybe around 0.6, and the position should probably be reduced" requires regex extraction, heuristic parsing, and fallback logic for every possible phrasing variant.

Structured outputs eliminate this. When the provider supports `response_format` with a JSON Schema, the model is *constrained* to produce output that conforms to the schema. The response is valid JSON with guaranteed fields, types, and structure. The Golem deserializes it directly into a Rust struct with `serde_json::from_str`. No parsing. No regex. No heuristics.

Venice, OpenAI, and several open models (via Venice/OpenRouter) support structured outputs natively. Anthropic Claude supports structured outputs through the tool_use mechanism. Bankr passes through to the underlying provider's capabilities. The design principle: every subsystem that uses structured outputs must also work without them. The structured schema is an *optimization* that produces cleaner data with less post-processing, but the subsystem falls back to prompt-guided JSON + parsing when the provider doesn't support schema enforcement.

### 8.1 StructuredOutput trait

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
    /// Implementations should handle formatting variations.
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
    if let Some(start) = text.find("```json") {
        let content_start = start + 7;
        if let Some(end) = text[content_start..].find("```") {
            return Some(text[content_start..content_start + end].trim());
        }
    }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Some(&text[start..=end]);
        }
    }
    None
}
```

### 8.2 Gateway decision logic

The gateway decides whether to use schema enforcement, the Anthropic tool-use workaround, or prompt-guided fallback:

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
                // whose input_schema matches the response schema.
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

### 8.3 Provider support matrix for structured outputs

| Provider | Schema Enforcement | Tool Use Workaround | Notes |
| --- | --- | --- | --- |
| **Venice** (uncensored, Llama, Qwen) | yes `response_format.json_schema` | yes | `supportsResponseSchema` flag on model. `strict: true` required. |
| **Venice** (Claude via Venice) | no | yes tool_use | Claude doesn't support response_format natively; use tool_use. |
| **Venice** (DeepSeek R1) | no | no | Reasoning models often incompatible. Use prompt-guided. |
| **Anthropic** (Direct/BlockRun) | no | yes tool_use | Anthropic's structured output path is tool_use with a synthetic tool. |
| **OpenAI** (Direct/BlockRun) | yes `response_format.json_schema` | yes | Full support. `strict: true`. |
| **Bankr** | Passthrough | Passthrough | Depends on underlying provider (Claude via Vertex, GPT via OpenRouter). |
| **OpenRouter** | Model-dependent | Model-dependent | Check model capabilities at resolution time. |

### 8.4 Subsystem schemas

Seven subsystems define structured output schemas. Each schema includes both the JSON Schema definition and a `from_text` fallback parser for providers that lack schema enforcement.

#### HeartbeatDecision

Used by: `heartbeat_t1`, `heartbeat_t2`

```rust
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
```

#### DaimonAppraisal

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
```

#### RiskAssessment

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

#### CuratorEvaluation

Used by: `curator`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratorEvaluation {
    pub entries_evaluated: u32,
    pub entries_retained: u32,
    pub entries_pruned: u32,
    pub cross_references: Vec<CrossRef>,
    pub promotions: Vec<Promotion>,
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

#### ReplayAnalysis

Used by: `dream_nrem`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayAnalysis {
    pub episode_id: String,
    pub lessons: Vec<Lesson>,
    pub surprise_score: f64,
    pub counterfactuals: Vec<Counterfactual>,
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

#### FragmentEvaluation

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

#### DreamIntegration

Used by: `dream_integration`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamIntegration {
    pub promoted: Vec<IntegrationItem>,
    pub staged: Vec<IntegrationItem>,
    pub discarded: Vec<IntegrationItem>,
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

#### ThreatAssessment

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

### 8.5 Anthropic tool-use workaround

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

### 8.6 Validation and error handling

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

**Retry logic**: If validation fails on a schema-enforced response (content error, not schema error), the subsystem may retry once with additional prompt guidance specifying the validation failure. If the retry also fails, the subsystem falls back to default values and logs a warning.

### 8.7 Structured output cost impact

| Mode | First-call Latency | Subsequent Latency | Token Overhead |
| --- | --- | --- | --- |
| Schema enforced (Venice/OpenAI) | +200-500ms (schema compilation) | Normal | ~5-10% fewer output tokens (no formatting) |
| Tool-use workaround (Anthropic) | Normal | Normal | ~10% more tokens (tool definition) |
| Prompt-guided fallback | Normal | Normal | ~15-20% more tokens (schema in prompt + formatting) |

The schema-enforced path is cheapest after the first call (schemas are cached per-session). The prompt-guided fallback is most expensive but universally available.

### 8.8 Structured output configuration

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

## 9. Reasoning chain -> Grimoire + dream replay integration

### Subsystem-specific reasoning strategies

**Dreams: visible reasoning as narration.** Ideal path: Bardo Inference -> Venice -> DeepSeek R1 (visible + private + DIEM-funded). Fallback: BlockRun/OpenRouter -> DeepSeek R1 (visible, not private). Last resort: any Claude backend with adaptive thinking (summarized, not visible). The visible reasoning chain is stored in the Grimoire as a dream episode, emitted to the BardoEventBus for UI rendering, and used by the Curator cycle for knowledge consolidation.

**Risk: interleaved thinking with tools.** Ideal path: any Claude backend -> Opus 4.6 with interleaved thinking. When Bankr is enabled, risk assessment can add cross-model verification -- send the same risk prompt to two providers and compare outputs. Fallback: GPT-5.x with high reasoning effort.

**Context reflection: inline think toggle.** Ideal path: any Qwen backend -> `/think` for anomalous, `/no_think` for routine. Fallback: Claude Haiku with `effort: "low"`.

**Death reflection: maximum everything.** Ideal path: Venice -> DeepSeek R1 (64K output, visible, private, DIEM). Budget unlimited. Death is always fully funded regardless of mortality pressure.

### Reasoning budget and mortality integration

```rust
pub fn compute_reasoning_budgets(
    vitality: f64,
    daily_budget_usd: f64,
) -> SubsystemBudgets {
    let pressure = 1.0 - vitality;
    SubsystemBudgets {
        heartbeat: daily_budget_usd * 0.20 * (1.0 - pressure * 0.5),
        risk: daily_budget_usd * 0.15,     // Never reduced
        dream: daily_budget_usd * 0.15 * (1.0 - pressure * 0.3),
        daimon: daily_budget_usd * 0.05 * (1.0 - pressure * 0.7),
        context: daily_budget_usd * 0.10 * (1.0 - pressure * 0.5),
        curator: daily_budget_usd * 0.10 * (1.0 - pressure * 0.3),
        playbook: daily_budget_usd * 0.05 * (1.0 - pressure * 0.5),
        operator: daily_budget_usd * 0.15, // Never reduced
        death: f64::INFINITY,              // Always fully funded
    }
}
```

DIEM and self-funding adjustments: Venice-routed calls consume DIEM allocation (zero USD cost), giving effectively unlimited budget within the daily DIEM cap. When Bankr sustainability ratio > 2.0, all budgets expand 1.5x. Below 0.5, budgets contract to 0.3x (except risk and death).

### Degradation when budget exhausted

| Subsystem | Degradation |
|---|---|
| Dreams | Skip cycle (dreams are periodic, missing one is acceptable) |
| Daimon | Fall back to deterministic OCC rules (no LLM) |
| Risk | **Never degraded** -- safety critical |
| Context | Fall back to static heuristics |
| Heartbeat | Reduce T2 calls to T1 |
| Death | **Never degraded** -- always fully funded |

---

## 10. Cross-references

- **[01a-routing.md](./01a-routing.md)** -- Model routing and InferenceProfile. The per-subsystem parameter tables define which subsystems use structured outputs.
- **[04-context-engineering.md](./04-context-engineering.md)** -- Context budget allocation. Reasoning tokens affect how much context budget remains for tool results and conversation history.
- **[03-economics.md](./03-economics.md)** -- Cost attribution. `ReasoningCost` feeds into per-tenant cost tracking and x402 spread calculations.
- **[07-safety.md](./07-safety.md)** -- Audit logging. `ReasoningChain` data populates `InferenceLog` fields for per-request cost and token attribution.
- **[12-providers.md](./12-providers.md)** -- Provider registry. `supports_reasoning` flag gates `ReasoningParser` attachment. Venice deep integration and Bankr feature passthrough.
- **[14-rust-implementation.md](./14-rust-implementation.md)** -- Crate structure. Types live in `bardo-inference/src/reasoning/`.
- **[Cognition Model](../01-golem/01-cognition.md)** -- PAD-modulated routing uses reasoning token counts as a signal for model downgrades.
- **[Extension Runtime](../01-golem/13-runtime-extensions.md)** -- Extension hook system receives `ReasoningEvent` streams for interception and logging.
- **[Risk Engine](../01-golem/00-overview.md)** -- RiskAssessment schema maps to the five-layer risk evaluation pipeline.
- **[Daimon Appraisal](../03-daimon/01-appraisal.md)** -- DaimonAppraisal schema extracts PAD vectors for emotional state tracking.
- **[Dream Consolidation](../05-dreams/04-consolidation.md)** -- DreamIntegration and ReplayAnalysis schemas drive promote/stage/discard decisions.
- **[Hypnagogic Observer](../06-hypnagogia/04-homunculus.md)** -- FragmentEvaluation schema scores novelty/relevance/coherence of creative fragments.
- **[Venice Structured Responses](https://docs.venice.ai/overview/guides/structured-responses)** -- Venice API documentation for JSON Schema enforcement.
- **[Anthropic Tool Use](https://docs.anthropic.com/en/docs/build-with-claude/tool-use)** -- Anthropic's tool_use mechanism used for structured output workaround.
- **[OpenAI Structured Outputs](https://platform.openai.com/docs/guides/structured-outputs)** -- OpenAI's native JSON Schema enforcement.
