# 12 -- Multi-Provider Architecture [SPEC]

**Five-Provider Architecture with Self-Describing Resolution**

> Related: [00-overview.md](00-overview.md) (gateway overview), [01a-routing.md](01a-routing.md) (model routing), [03-economics.md](03-economics.md) (revenue model), [13-reasoning.md](13-reasoning.md) (reasoning modes), [14-rust-implementation.md](14-rust-implementation.md) (gateway binary), [../05-dreams/07-venice-dreaming.md](../05-dreams/07-venice-dreaming.md) (Venice-augmented dreaming), [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) (Warden attestation, optional)

---

> **Reader orientation:** This document specifies the multi-provider architecture of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes the five provider backends (BlockRun, OpenRouter, Venice, Bankr, Direct Key), their Rust trait implementations, the self-describing resolution algorithm, and the Venice private cognition deep-dive with TEE attestation and E2EE inference. The key concept is that each provider occupies a distinct role, not an interchangeable fallback, and the resolver walks an ordered list matching declarative intents to provider capabilities. For term definitions, see `prd2/shared/glossary.md`.

## Five Providers

Bardo Inference routes through five provider backends. Each occupies a distinct role in the system -- they are not interchangeable fallbacks but specialized planes of access.

| #  | Provider   | Role                  | Access     | Models                                            | Payment   |
| -- | ---------- | --------------------- | ---------- | ------------------------------------------------- | --------- |
| 1  | BlockRun   | Primary gateway       | x402       | All families (Anthropic, OpenAI, Google, OSS)     | x402 USDC |
| 2  | OpenRouter | Universal fallback    | API key    | 400+ models, BYOK support                         | Prepaid   |
| 3  | Venice     | Private cognition     | API key    | DeepSeek R1, Llama, GLM, Qwen VL                  | DIEM      |
| 4  | Bankr      | Self-funding          | Wallet     | Cross-model (delegates to other providers)        | Wallet    |
| 5  | Direct Key | Native feature access | User's key | Provider-native (OpenAI, Anthropic, Google, etc.) | Direct    |
| 6  | Local      | Embedding only        | Local      | nomic-embed-text-v1.5 (ONNX)                      | Free      |

### BlockRun (primary)

BlockRun is the default route for all inference. It fetches model catalogs dynamically from `GET /v1/models`, supports every major model family through a unified x402 payment interface, and requires zero API keys. USDC flows User -> Bardo -> BlockRun with no float, no fiat bridge, no credit risk. The catalog refresh is hourly; no model IDs or prices are hardcoded in the gateway binary.

### OpenRouter (fallback)

OpenRouter covers the long tail. When BlockRun lacks a specific model or goes down, OpenRouter's 400+ model catalog fills the gap. It requires an operator API key and breaks the zero-float property (prepaid credits needed), so it is a fallback, not a co-primary. BYOK mode lets users bring their own OpenRouter key, bypassing the operator's float entirely.

### Venice (private cognition)

Venice provides no-log, TEE-attested inference for privacy-sensitive tasks. It is not a fallback -- it is a separate inference plane selected by task class when the Golem's config sets `inferenceProvider: "both"`. Venice supports DIEM staking for zero-cost inference: stake DIEM tokens, receive inference credits proportional to stake weight. Models include DeepSeek R1, Llama 4 Maverick, GLM-4, and Qwen VL. The `strip_thinking_response` parameter controls whether `<think>` tags appear in responses. See [section 5](#venice-private-cognition-plane) for the full deep-dive.

### Bankr (self-funding)

Bankr is the economic engine for autonomous agents. Revenue from social engagement (tips, subscriptions, content monetization) funds inference directly. The execution wallet and the revenue wallet are the same key -- the agent thinks with the money it earns. Cross-model verification routes the same query through two providers and compares outputs, paying for redundancy only when the stakes justify it.

### Direct Key (passthrough)

Direct Key mode bypasses the context engineering pipeline entirely and routes requests straight to a provider's native API using the user's own credentials. This exists because some features require exact prompt structure that context engineering would corrupt: OpenAI's Predicted Outputs, Anthropic's Batch API, Gemini's explicit caching with configurable TTL, OpenAI's stateful Responses API. Safety is the user's responsibility in this mode.

### Local (embedding only)

`nomic-embed-text-v1.5` runs as an ONNX model on the proxy's CPU. 768-dimensional embeddings in under 5ms. Used exclusively for semantic cache lookups and request classification. Never routes chat completions.

---

## Per-provider feature catalog

Each provider family exposes distinct capabilities. The gateway's routing engine must know what each can do to match intents correctly.

### Anthropic

| Feature               | Detail                                                                    |
| --------------------- | ------------------------------------------------------------------------- |
| Citations             | Model can cite specific passages from provided documents                  |
| Compaction             | Server-side context compression (beta)                                   |
| Prompt caching        | 90% input discount, 5-min default TTL, extendable to 1hr with breakpoint |
| Adaptive thinking     | Extended thinking with budget control                                    |
| Context window        | 1M tokens (Opus 4.6, Sonnet 4)                                          |
| Max output            | 128K tokens                                                              |
| Batch API             | 50% discount, async processing, 24hr SLA                                |
| Effort parameter      | `low` / `medium` / `high` reasoning effort control                       |
| Interleaved thinking  | Thinking blocks interleaved with tool use                                |
| Structured outputs    | JSON schema enforcement via tool use                                     |
| Redacted thinking     | Thinking content hidden from API response (safety applications)          |

Anthropic's prompt caching is the single most impactful cost optimization. A stable system prompt + tool definitions block that hits the 5-min TTL window gets a 90% discount on those tokens every subsequent request. The gateway's prompt reordering (see [04-context-engineering.md](04-context-engineering.md)) is specifically designed to maximize this hit rate.

### OpenAI

| Feature              | Detail                                                            |
| -------------------- | ----------------------------------------------------------------- |
| Predicted outputs    | Supply expected output for 2-3x speed on edits (code refactoring) |
| Responses API        | Stateful conversations, server-managed context                    |
| Reasoning summaries  | Condensed chain-of-thought from o-series models                   |
| Structured outputs   | Strict JSON schema enforcement (native, not via tool use)         |
| Web search           | Built-in web search tool                                          |
| Tool search          | Built-in file/code search tool                                    |
| Verbosity control    | `output_style: "concise"` or `"verbose"`                          |
| Batch API            | 50% discount, async processing                                   |

Predicted Outputs is why Direct Key mode exists. The feature requires sending the expected output verbatim in the request body. Context engineering would rewrite or compress this, destroying the optimization. When a Golem knows what 90% of a code file will look like after an edit, Predicted Outputs cuts latency by 2-3x.

### Gemini

| Feature                    | Detail                                              |
| -------------------------- | --------------------------------------------------- |
| `thinking_level`           | `none` / `low` / `medium` / `high` thinking control |
| Search grounding           | Google Search integration for factual queries        |
| Code execution sandbox     | Server-side Python execution                        |
| Explicit caching           | Configurable TTL, user-controlled cache keys         |
| Implicit caching           | Automatic, no user intervention                     |
| Media resolution control   | `low` / `medium` / `high` for image/video inputs    |
| Streaming function calling | Tool calls emitted mid-stream before completion      |

Gemini's explicit caching with configurable TTL is the most flexible caching model. Unlike Anthropic's fixed 5-min/1hr windows, operators can set arbitrary TTLs per cache entry. The tradeoff: explicit cache management adds complexity. The gateway abstracts this behind `ProviderTraits.context_engineering`.

### DeepSeek

| Feature                 | Detail                                        |
| ----------------------- | --------------------------------------------- |
| Visible `<think>` tags  | Reasoning chain exposed in response text       |
| 64K output              | Large output window for code generation        |
| Open weights (MIT)      | Self-hostable, no vendor lock-in              |
| Hybrid thinking (V3.1+) | Toggle reasoning on/off per request           |
| Function calling        | Tool use support (R1-0528+)                   |

DeepSeek R1's visible thinking tags are a feature, not a bug. The gateway can parse `<think>...</think>` blocks to extract reasoning chains for observability without paying for a separate "reasoning summary" API call. Venice serves DeepSeek R1 with TEE attestation, making it the privacy-first reasoning model.

### Qwen

| Feature                  | Detail                                           |
| ------------------------ | ------------------------------------------------ |
| Hybrid `/think` toggle   | `/think` and `/no_think` inline directives       |
| `enable_thinking` param  | API-level thinking on/off                        |
| `thinking_budget`        | Token budget for reasoning (controls cost)       |
| MoE architecture         | ~10% active params per token (cost-efficient)    |
| 262K context             | Largest context window in the OSS model family   |

Qwen's MoE architecture means it activates roughly 10% of its parameters per token. A 235B-parameter model that behaves like a 22B model at inference time. The cost-per-token is closer to a small model while quality approaches frontier. Good fit for T1 routing when Haiku is unavailable.

---

## Provider trait and resolution

The gateway resolves intents to concrete model + provider pairs. Each provider is self-describing: it knows its own capabilities and can answer "can you handle this?" without the router maintaining a central compatibility matrix.

```rust
// crates/bardo-providers/src/traits.rs

use std::time::Instant;

/// A provider that knows its own capabilities.
#[derive(Debug)]
pub trait Provider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn resolve(&self, intent: &Intent) -> Option<Resolution>;
    fn traits(&self) -> &ProviderTraits;
}

#[derive(Debug, Clone)]
pub struct ProviderTraits {
    pub private: bool,
    pub self_funding: bool,
    pub context_engineering: bool,
    pub payment: PaymentMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentMode {
    X402,
    Prepaid,
    ApiKey,
    Diem,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quality {
    Minimum,
    Low,
    Medium,
    High,
    Maximum,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub model: String,
    pub provider: String,
    pub estimated_cost_usd: f64,
    pub features: Vec<String>,
    pub degraded: Vec<String>,
}

#[derive(Debug)]
pub enum ProviderHealth {
    Healthy,
    Degraded { error_rate: f64 },
    Down { since: Instant },
}
```

### Resolution algorithm

The resolver walks the provider list in priority order. First provider that returns `Some(resolution)` wins. If no provider matches with hard requirements, the resolver retries with requirements relaxed to preferences -- graceful degradation over hard failure.

```rust
// crates/bardo-providers/src/resolver.rs

pub fn resolve(providers: &[Box<dyn Provider>], intent: &Intent) -> Option<Resolution> {
    // Pass 1: strict -- all requirements must be satisfied
    for provider in providers {
        if let Some(resolution) = provider.resolve(intent) {
            return Some(resolution);
        }
    }

    // Pass 2: degraded -- move requirements to preferences, try again
    let relaxed = Intent {
        model: intent.model.clone(),
        require: vec![],
        prefer: [intent.require.clone(), intent.prefer.clone()].concat(),
        quality: intent.quality.clone(),
        max_latency_ms: intent.max_latency_ms * 2,
        cost_sensitivity: intent.cost_sensitivity,
        diem_available: intent.diem_available,
        subsystem: intent.subsystem.clone(),
    };

    for provider in providers {
        if let Some(mut resolution) = provider.resolve(&relaxed) {
            resolution.degraded = intent.require.clone();
            return Some(resolution);
        }
    }

    None
}
```

Two passes, ~20 lines. The first pass enforces hard requirements. If nothing matches, the second pass converts requirements into preferences and doubles the latency budget. The `degraded` field on the returned `Resolution` tells the caller exactly which requirements were dropped. The caller decides whether degraded service is acceptable or whether to fail the request.

## Subsystem intents

Each subsystem declares a static intent -- what it needs from inference. The resolver walks the provider list against these intents. Adding a new subsystem means adding one more const.

```rust
pub const HEARTBEAT_T0: Intent = Intent {
    model: None, require: vec![], prefer: vec![],
    quality: Quality::Minimum, max_latency_ms: 200,
    cost_sensitivity: 1.0, diem_available: false,
    subsystem: "heartbeat_t0".into(),
};

pub const HEARTBEAT_T1: Intent = Intent {
    model: None, require: vec![], prefer: vec!["low_effort".into()],
    quality: Quality::Low, max_latency_ms: 2000,
    cost_sensitivity: 0.8, diem_available: false,
    subsystem: "heartbeat_t1".into(),
};

pub const HEARTBEAT_T2: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["interleaved_thinking".into(), "citations".into()],
    quality: Quality::High, max_latency_ms: 30_000,
    cost_sensitivity: 0.3, diem_available: false,
    subsystem: "heartbeat_t2".into(),
};

pub const RISK: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["interleaved_thinking".into(), "citations".into()],
    quality: Quality::Maximum, max_latency_ms: 30_000,
    cost_sensitivity: 0.0, diem_available: false,
    subsystem: "risk".into(),
};

pub const DREAM: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["visible_thinking".into(), "privacy".into()],
    quality: Quality::High, max_latency_ms: 120_000,
    cost_sensitivity: 0.5, diem_available: false,
    subsystem: "dream".into(),
};

pub const DAIMON: Intent = Intent {
    model: None, require: vec![], prefer: vec!["privacy".into()],
    quality: Quality::Low, max_latency_ms: 1000,
    cost_sensitivity: 0.9, diem_available: false,
    subsystem: "daimon".into(),
};

pub const DAIMON_COMPLEX: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["visible_thinking".into(), "privacy".into()],
    quality: Quality::High, max_latency_ms: 10_000,
    cost_sensitivity: 0.5, diem_available: false,
    subsystem: "daimon_complex".into(),
};

pub const CURATOR: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["structured_outputs".into(), "citations".into()],
    quality: Quality::Medium, max_latency_ms: 15_000,
    cost_sensitivity: 0.5, diem_available: false,
    subsystem: "curator".into(),
};

pub const PLAYBOOK: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["predicted_outputs".into()],
    quality: Quality::Medium, max_latency_ms: 10_000,
    cost_sensitivity: 0.6, diem_available: false,
    subsystem: "playbook".into(),
};

pub const OPERATOR: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["interleaved_thinking".into(), "citations".into()],
    quality: Quality::Maximum, max_latency_ms: 5000,
    cost_sensitivity: 0.0, diem_available: false,
    subsystem: "operator".into(),
};

pub const DEATH: Intent = Intent {
    model: None,
    require: vec!["visible_thinking".into()],
    prefer: vec!["privacy".into()],
    quality: Quality::Maximum, max_latency_ms: u64::MAX,
    cost_sensitivity: 0.0, diem_available: false,
    subsystem: "death".into(),
};

pub const SESSION_COMPACT: Intent = Intent {
    model: None, require: vec![],
    prefer: vec!["compaction".into()],
    quality: Quality::Medium, max_latency_ms: 30_000,
    cost_sensitivity: 0.5, diem_available: false,
    subsystem: "session_compact".into(),
};
```

### Intent lookup

The `ModelRouter` extension (see [../01-golem/01-cognition.md](../01-golem/01-cognition.md) S2) calls `subsystem_intent()` to map the current subsystem to its static intent:

```rust
pub fn subsystem_intent(subsystem: &str) -> Intent {
    match subsystem {
        "heartbeat_t0" => HEARTBEAT_T0,
        "heartbeat_t1" => HEARTBEAT_T1,
        "heartbeat_t2" => HEARTBEAT_T2,
        "risk"         => RISK,
        "dream"        => DREAM,
        "daimon"       => DAIMON,
        "daimon_complex" => DAIMON_COMPLEX,
        "curator"      => CURATOR,
        "playbook"     => PLAYBOOK,
        "operator"     => OPERATOR,
        "death"        => DEATH,
        "session_compact" => SESSION_COMPACT,
        _ => HEARTBEAT_T1, // conservative default
    }
}
```

### Mortality pressure

Dying Golems become more cost-sensitive. Risk, death, and operator are exempt. At vitality < 0.3 (pressure > 0.7), quality downgrades one level -- a `High` dream intent drops to `Medium`, routing to a cheaper model.

```rust
pub fn apply_mortality_pressure(intent: &mut Intent, vitality: f64) {
    let exempt = ["risk", "death", "operator"];
    if exempt.contains(&intent.subsystem.as_str()) { return; }

    let pressure = 1.0 - vitality;
    intent.cost_sensitivity = (intent.cost_sensitivity + pressure * 0.3).min(1.0);
    if pressure > 0.7 {
        intent.quality = intent.quality.downgrade();
    }
}
```

---

## Provider adapter implementations

Each provider implements the `Provider` trait. The implementations are concrete -- no generics, no trait objects beyond the `Provider` trait itself.

### BlockRunProvider

```rust
// crates/bardo-providers/src/blockrun.rs

#[derive(Debug)]
pub struct BlockRunProvider {
    pub endpoint: String,
    pub catalog: Vec<ModelEntry>,
    pub health: ProviderHealth,
}

impl Provider for BlockRunProvider {
    fn id(&self) -> &str { "blockrun" }
    fn name(&self) -> &str { "BlockRun" }

    fn traits(&self) -> &ProviderTraits {
        &ProviderTraits {
            private: false,
            self_funding: false,
            context_engineering: true,
            payment: PaymentMode::X402,
        }
    }

    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        if matches!(self.health, ProviderHealth::Down { .. }) {
            return None;
        }

        let candidates: Vec<&ModelEntry> = self.catalog.iter()
            .filter(|m| match &intent.model {
                Some(id) => m.id.contains(id),
                None => true,
            })
            .filter(|m| intent.require.iter().all(|r| m.capabilities.contains(r)))
            .filter(|m| m.latency_p50_ms <= intent.max_latency_ms)
            .collect();

        let best = candidates.iter()
            .min_by(|a, b| {
                let cost_a = a.input_per_million + a.output_per_million;
                let cost_b = b.input_per_million + b.output_per_million;
                let score_a = cost_a * intent.cost_sensitivity
                    - preference_score(a, &intent.prefer) * (1.0 - intent.cost_sensitivity);
                let score_b = cost_b * intent.cost_sensitivity
                    - preference_score(b, &intent.prefer) * (1.0 - intent.cost_sensitivity);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })?;

        Some(Resolution {
            model: best.id.clone(),
            provider: "blockrun".into(),
            estimated_cost_usd: (best.input_per_million + best.output_per_million) / 1_000_000.0 * 50_000.0,
            features: best.capabilities.clone(),
            degraded: vec![],
        })
    }
}

fn preference_score(model: &ModelEntry, preferences: &[String]) -> f64 {
    preferences.iter()
        .filter(|p| model.capabilities.contains(p))
        .count() as f64
}

#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub id: String,
    pub family: String,
    pub capabilities: Vec<String>,
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub latency_p50_ms: u64,
    pub max_context_tokens: u64,
}
```

BlockRun's catalog is fetched hourly from `/v1/models`. The resolver filters by explicit model match (if requested), hard requirements, and latency budget, then scores remaining candidates by a weighted blend of cost and preference satisfaction. `cost_sensitivity` at 1.0 picks the cheapest model; at 0.0 it picks the one with the most preferred features.

### OpenRouterProvider

```rust
// crates/bardo-providers/src/openrouter.rs

#[derive(Debug)]
pub struct OpenRouterProvider {
    pub api_key: String,
    pub catalog: Vec<ModelEntry>,
    pub health: ProviderHealth,
}

impl Provider for OpenRouterProvider {
    fn id(&self) -> &str { "openrouter" }
    fn name(&self) -> &str { "OpenRouter" }

    fn traits(&self) -> &ProviderTraits {
        &ProviderTraits {
            private: false,
            self_funding: false,
            context_engineering: true,
            payment: PaymentMode::ApiKey,
        }
    }

    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        if matches!(self.health, ProviderHealth::Down { .. }) {
            return None;
        }
        if self.api_key.is_empty() {
            return None;
        }

        // OpenRouter has 400+ models. Filter aggressively.
        let candidates: Vec<&ModelEntry> = self.catalog.iter()
            .filter(|m| match &intent.model {
                Some(id) => m.id.contains(id),
                None => true,
            })
            .filter(|m| intent.require.iter().all(|r| m.capabilities.contains(r)))
            .collect();

        let best = candidates.iter()
            .min_by(|a, b| {
                let cost_a = a.input_per_million + a.output_per_million;
                let cost_b = b.input_per_million + b.output_per_million;
                cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
            })?;

        Some(Resolution {
            model: best.id.clone(),
            provider: "openrouter".into(),
            estimated_cost_usd: (best.input_per_million + best.output_per_million) / 1_000_000.0 * 50_000.0,
            features: best.capabilities.clone(),
            degraded: vec![],
        })
    }
}
```

OpenRouter is the universal safety net. If the API key is empty, it returns `None` for all intents -- the provider is effectively disabled. When active, it resolves purely on cost (cheapest model that satisfies requirements). No preference scoring because OpenRouter's catalog is too large for fine-grained feature matching to be reliable.

### VeniceProvider

```rust
// crates/bardo-providers/src/venice.rs

use std::collections::HashMap;

/// Privacy tier determines what trust assumptions the golem places on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyTier {
    /// No-log: prompt/completion not retained. Host assumed non-malicious at runtime.
    Private,
    /// Inference runs in attested enclave. Enclave integrity verified by attestation receipt.
    Tee,
    /// End-to-end encrypted. Host cannot read prompt. Zero host trust required.
    E2e,
}

/// Task class determines the sensitivity classifier's input.
/// Each variant maps to exactly one PrivacyTier via TASK_PRIVACY_MAP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskClass {
    Heartbeat,       // Routine tick analysis -> Private
    Plan,            // Strategy planning -> Tee
    Trade,           // Trade decision -> Tee
    Negotiate,       // Agent negotiation -> Tee
    DreamNrem,       // Dream replay -> Private
    DreamRem,        // REM imagination -> Private
    DreamIntegration,// Dream insight promotion -> Tee
    Incident,        // Incident response -> Private (content redacted)
    SealedAuction,   // CCA sealed bid -> E2e
    MevScenario,     // MEV threat simulation -> E2e
    Testament,       // Death testament -> Private (stored locally)
    Judge,           // Confidential evaluation -> Private
}

/// Static mapping from task class to privacy tier.
pub static TASK_PRIVACY_MAP: phf::Map<TaskClass, PrivacyTier> = {
    use TaskClass::*;
    use PrivacyTier::*;
    // Compile-time map. In practice this is a match arm, shown as a map
    // for parity with the specification.
    phf::phf_map! {
        Heartbeat => Private,
        Plan => Tee,
        Trade => Tee,
        Negotiate => Tee,
        DreamNrem => Private,
        DreamRem => Private,
        DreamIntegration => Tee,
        Incident => Private,
        SealedAuction => E2e,
        MevScenario => E2e,
        Testament => Private,
        Judge => Private,
    }
};

// In practice, the static map above is implemented as a const fn:
pub const fn classify(task: TaskClass) -> PrivacyTier {
    match task {
        TaskClass::Heartbeat => PrivacyTier::Private,
        TaskClass::Plan => PrivacyTier::Tee,
        TaskClass::Trade => PrivacyTier::Tee,
        TaskClass::Negotiate => PrivacyTier::Tee,
        TaskClass::DreamNrem => PrivacyTier::Private,
        TaskClass::DreamRem => PrivacyTier::Private,
        TaskClass::DreamIntegration => PrivacyTier::Tee,
        TaskClass::Incident => PrivacyTier::Private,
        TaskClass::SealedAuction => PrivacyTier::E2e,
        TaskClass::MevScenario => PrivacyTier::E2e,
        TaskClass::Testament => PrivacyTier::Private,
        TaskClass::Judge => PrivacyTier::Private,
    }
}

/// Constraints that apply at each privacy tier. E2E is the most restrictive:
/// no tool calling, no web search, no file upload. The model processes ciphertext
/// and cannot dispatch tools because the host cannot read the context.
#[derive(Debug, Clone)]
pub struct TierConstraints {
    pub tool_calling_allowed: bool,
    pub web_search_allowed: bool,
    pub file_upload_allowed: bool,
    pub structured_output_required: bool,
}

pub const PRIVATE_CONSTRAINTS: TierConstraints = TierConstraints {
    tool_calling_allowed: true,
    web_search_allowed: false,  // web search enabled separately in dream REM
    file_upload_allowed: true,
    structured_output_required: false,
};

pub const TEE_CONSTRAINTS: TierConstraints = TierConstraints {
    tool_calling_allowed: true,
    web_search_allowed: false,
    file_upload_allowed: false,
    structured_output_required: true, // Safety-critical params must be structured
};

pub const E2E_CONSTRAINTS: TierConstraints = TierConstraints {
    tool_calling_allowed: false, // Cannot dispatch tools when host cannot read context
    web_search_allowed: false,
    file_upload_allowed: false,
    structured_output_required: true,
};

pub const fn tier_constraints(tier: PrivacyTier) -> &'static TierConstraints {
    match tier {
        PrivacyTier::Private => &PRIVATE_CONSTRAINTS,
        PrivacyTier::Tee => &TEE_CONSTRAINTS,
        PrivacyTier::E2e => &E2E_CONSTRAINTS,
    }
}

/// TEE attestation receipt. Produced by the enclave at inference time.
/// Verified by the gateway before the response reaches the caller.
#[derive(Debug, Clone)]
pub struct AttestationReceipt {
    pub enclave_measurement: String, // PCR digests (SHA-384)
    pub nonce: String,               // Freshness challenge
    pub timestamp: u64,              // Unix ms
    pub model_id: String,
    pub response_hash: String,       // SHA-256 of response content
    pub debug_mode: bool,            // MUST be false for production
    pub signature: Vec<u8>,          // Signed by Venice's attestation key
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationError {
    DebugEnclave,
    StaleNonce { age_ms: u64 },
    InvalidSignature,
    HashMismatch { expected: String, actual: String },
}

/// Verify an attestation receipt. Returns Ok(()) if all checks pass.
/// On failure, the response is discarded -- a failed attestation is treated
/// as a provider error, not a degraded-but-usable response.
pub fn verify_attestation(
    receipt: &AttestationReceipt,
    response_content: &[u8],
    now_ms: u64,
    attestation_pubkey: &[u8],
) -> Result<(), AttestationError> {
    // 1. Debug enclave flag: non-negotiable. Debug enclaves have no integrity guarantees.
    if receipt.debug_mode {
        return Err(AttestationError::DebugEnclave);
    }

    // 2. Nonce freshness: receipt timestamp within 30s.
    let age = now_ms.saturating_sub(receipt.timestamp);
    if age > 30_000 {
        return Err(AttestationError::StaleNonce { age_ms: age });
    }

    // 3. Signature: verify against Venice's published attestation public key.
    // (Actual crypto verification delegated to ring or ed25519-dalek)
    let sig_valid = verify_ed25519(attestation_pubkey, &receipt.signature, receipt);
    if !sig_valid {
        return Err(AttestationError::InvalidSignature);
    }

    // 4. Response hash: sha256(response_content) == receipt.response_hash.
    let actual_hash = sha256_hex(response_content);
    if actual_hash != receipt.response_hash {
        return Err(AttestationError::HashMismatch {
            expected: receipt.response_hash.clone(),
            actual: actual_hash,
        });
    }

    Ok(())
}

// Placeholder signatures for crypto primitives (implemented in crates/bardo-crypto)
fn verify_ed25519(_pubkey: &[u8], _sig: &[u8], _receipt: &AttestationReceipt) -> bool { true }
fn sha256_hex(_data: &[u8]) -> String { String::new() }

/// Cache prompt structure. Venice prompt caching works on stable prefixes.
/// The coordinator separates the prompt into a stable kernel (policy,
/// invariants, tool contracts) and a volatile tail (current balances,
/// timestamps, tick IDs).
#[derive(Debug, Clone)]
pub struct CachePromptStructure {
    pub stable_kernel: StableKernel,
    pub volatile_tail: VolatileTail,
    pub prompt_cache_key: String, // golemId + sessionEpoch
}

#[derive(Debug, Clone)]
pub struct StableKernel {
    pub system_prompt: String,     // Policy, persona, behavioral constraints
    pub tool_definitions: String,  // Pruned tool set (<=12 tools, stable order)
    pub invariants: String,        // Static market facts (token addresses, protocol constants)
}

#[derive(Debug, Clone)]
pub struct VolatileTail {
    pub current_state: String,     // Balances, prices, tick data
    pub task_context: String,      // Specific task inputs
    pub recent_history: String,    // Last N turns only
}

/// VeniceCacheCoordinator manages prompt splitting and cache key lifecycle.
#[derive(Debug)]
pub struct VeniceCacheCoordinator {
    pub golem_id: String,
    pub session_epoch: u64,
}

impl VeniceCacheCoordinator {
    pub fn cache_key(&self) -> String {
        format!("{}:{}", self.golem_id, self.session_epoch)
    }

    /// Advance the session epoch. This invalidates the cache key.
    /// Called at dream cycle start, phase transition, or explicit invalidation.
    pub fn advance_epoch(&mut self) {
        self.session_epoch += 1;
    }
}

/// The provider implementation.
#[derive(Debug)]
pub struct VeniceProvider {
    pub api_key: String,
    pub diem_balance: f64,
    pub health: ProviderHealth,
}

impl Provider for VeniceProvider {
    fn id(&self) -> &str { "venice" }
    fn name(&self) -> &str { "Venice" }

    fn traits(&self) -> &ProviderTraits {
        &ProviderTraits {
            private: true,
            self_funding: false,
            context_engineering: false,
            payment: PaymentMode::Diem,
        }
    }

    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        if matches!(self.health, ProviderHealth::Down { .. }) {
            return None;
        }

        // Venice only resolves when privacy is required or DIEM is available
        let privacy_required = intent.require.contains(&"private".to_string())
            || intent.prefer.contains(&"private".to_string());
        let diem_funded = intent.diem_available && self.diem_balance > 0.0;

        if !privacy_required && !diem_funded {
            return None;
        }

        // Venice model selection: match by family or pick default
        let model = match &intent.model {
            Some(id) if id.contains("deepseek") => "deepseek-r1".to_string(),
            Some(id) if id.contains("llama") => "llama-4-maverick".to_string(),
            Some(id) if id.contains("qwen") => "qwen-vl-max".to_string(),
            Some(id) if id.contains("glm") => "glm-4".to_string(),
            _ => "deepseek-r1".to_string(),
        };

        let cost = if diem_funded { 0.0 } else { 0.002 }; // DIEM staking = zero cost

        Some(Resolution {
            model,
            provider: "venice".into(),
            estimated_cost_usd: cost,
            features: vec!["private".into(), "tee_attested".into(), "no_log".into()],
            degraded: vec![],
        })
    }
}
```

Venice only activates when privacy is required or DIEM staking makes it free. It never competes with BlockRun on general queries. The `strip_thinking_response` parameter is set at the request layer, not in the provider -- it controls whether `<think>` tags from DeepSeek R1 appear in the API response.

### BankrProvider

```rust
// crates/bardo-providers/src/bankr.rs

#[derive(Debug)]
pub struct BankrProvider {
    pub wallet_address: String,
    pub revenue_balance_usd: f64,
    pub lifetime_revenue_usd: f64,
    pub lifetime_cost_usd: f64,
    pub health: ProviderHealth,
}

impl Provider for BankrProvider {
    fn id(&self) -> &str { "bankr" }
    fn name(&self) -> &str { "Bankr" }

    fn traits(&self) -> &ProviderTraits {
        &ProviderTraits {
            private: false,
            self_funding: true,
            context_engineering: true,
            payment: PaymentMode::Wallet,
        }
    }

    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        if matches!(self.health, ProviderHealth::Down { .. }) {
            return None;
        }

        // Bankr only resolves for subsystems that use self-funding economics
        if intent.subsystem != "autonomous" && intent.subsystem != "social" {
            return None;
        }

        // Check sustainability: can we afford this?
        let sustainability = if self.lifetime_cost_usd > 0.0 {
            self.lifetime_revenue_usd / self.lifetime_cost_usd
        } else {
            f64::INFINITY
        };

        // Budget multiplier based on sustainability ratio
        let budget_mult = if sustainability > 2.0 {
            1.5 // Expanding: revenue far exceeds cost
        } else if sustainability < 0.5 {
            0.3 // Contracting: burning through reserves
        } else {
            1.0 // Steady state
        };

        let max_cost = self.revenue_balance_usd * 0.1 * budget_mult;
        if max_cost < 0.001 {
            return None; // Can't afford any inference
        }

        // Delegate to cheapest available model via cross-model selection
        let quality_model = match intent.quality {
            Quality::Maximum | Quality::High => "claude-sonnet-4",
            Quality::Medium => "claude-haiku-4-5",
            Quality::Low | Quality::Minimum => "deepseek-r1",
        };

        Some(Resolution {
            model: quality_model.to_string(),
            provider: "bankr".into(),
            estimated_cost_usd: max_cost.min(0.25),
            features: vec!["self_funding".into(), "cross_verify".into()],
            degraded: vec![],
        })
    }
}
```

Bankr's resolver checks sustainability before committing to inference. If the agent is burning through reserves faster than it earns (ratio < 0.5), the budget contracts to 30% of normal. If revenue is flush (ratio > 2.0), budgets expand 1.5x. The wallet address used for inference is the same wallet that receives social revenue -- no transfer step, no custodial intermediary.

### DirectKeyProvider

```rust
// crates/bardo-providers/src/direct.rs

#[derive(Debug)]
pub struct DirectKeyProvider {
    pub configured_keys: Vec<DirectKey>,
    pub health: ProviderHealth,
}

#[derive(Debug, Clone)]
pub struct DirectKey {
    pub provider: String,   // "anthropic", "openai", "google"
    pub api_key: String,
    pub models: Vec<String>,
}

impl Provider for DirectKeyProvider {
    fn id(&self) -> &str { "direct" }
    fn name(&self) -> &str { "Direct Key" }

    fn traits(&self) -> &ProviderTraits {
        &ProviderTraits {
            private: false,
            self_funding: false,
            context_engineering: false, // Bypassed intentionally
            payment: PaymentMode::ApiKey,
        }
    }

    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        if matches!(self.health, ProviderHealth::Down { .. }) {
            return None;
        }

        // Direct key only resolves when specific native features are required
        let needs_direct = intent.require.iter().any(|r| matches!(r.as_str(),
            "predicted_outputs" | "batch_api" | "explicit_caching" |
            "responses_api" | "direct_passthrough"
        ));

        if !needs_direct {
            return None;
        }

        // Find a configured key that can serve the requested model
        let key = self.configured_keys.iter().find(|k| {
            match &intent.model {
                Some(model) => k.models.iter().any(|m| model.contains(m)),
                None => true,
            }
        })?;

        Some(Resolution {
            model: intent.model.clone().unwrap_or_else(|| key.models[0].clone()),
            provider: format!("direct/{}", key.provider),
            estimated_cost_usd: -1.0, // Unknown -- user's bill, not ours
            features: vec!["native_api".into(), "no_context_engineering".into()],
            degraded: vec![],
        })
    }
}
```

Direct Key returns `estimated_cost_usd: -1.0` because the cost hits the user's own API bill, not Bardo's ledger. The gateway sets `X-Bardo-Cost: direct` in the response header so observability pipelines know this request bypassed the cost tracking system.

## Provider request formatting

After resolution, each provider transforms the generic request into provider-specific API parameters. The subsystem code never touches these.

**Venice**: Sets `strip_thinking_response: false` for dream, death, and daimon_complex subsystems (preserving visible `<think>` tags). All other subsystems strip thinking for smaller responses.

**BlockRun (Anthropic models)**: Sets `thinking: { type: "adaptive" }` with effort mapped from quality level (Minimum/Low -> "low", Medium -> "medium", High/Maximum -> "high").

**BlockRun (OpenAI models)**: Sets `reasoning.effort` mapped from quality level. `reasoning.summary` set based on visibility preference.

**BlockRun (Gemini models)**: Sets `thinking_level` ("low" for Minimum/Low quality, "high" for Medium/High/Maximum).

**BlockRun (Qwen models)**: Injects `/think` prefix for subsystems preferring visible_thinking, `/no_think` for routine operations.

See [13-reasoning.md](13-reasoning.md) for how reasoning features are normalized after provider responses arrive.

## Degradation visibility

When a provider satisfies an intent but not all preferences, `Resolution.degraded` names what is missing. The Golem emits this to the operator: "Dream cycle used Claude via BlockRun (visible thinking and privacy unavailable -- configure Venice for better dream quality)." The user knows exactly what to add to their config. Silent fallbacks are gone -- every compromise is visible.

## Routing examples

### Minimal: BlockRun only

```text
heartbeat_t0  -> BlockRun/nvidia-gpt-oss-120b (FREE)
heartbeat_t1  -> BlockRun/gemini-3-flash ($0.50/M)
heartbeat_t2  -> BlockRun/claude-opus-4-6 ($5/M, adaptive thinking)
risk          -> BlockRun/claude-opus-4-6 (interleaved thinking)
dream         -> BlockRun/deepseek-r1 ($0.55/M, visible <think>)
curator       -> BlockRun/claude-sonnet-4-6 ($3/M, structured outputs)
operator      -> BlockRun/claude-opus-4-6 (best quality)
death         -> BlockRun/deepseek-r1 (visible reasoning, max tokens)
```

### Full stack: all five providers

```text
heartbeat_t0  -> Direct/local/qwen3-7b (FREE, zero latency)
heartbeat_t1  -> BlockRun/gemini-3-flash (cached)
heartbeat_t2  -> BlockRun/claude-opus-4-6 (interleaved thinking, citations)
risk          -> BlockRun/claude-opus-4-6 + Bankr cross-model verify
dream         -> Venice/deepseek-r1 (visible, private, DIEM-funded)
daimon        -> Venice/llama-3.3-70b (private, fast)
curator       -> BlockRun/claude-sonnet-4-6 (citations for provenance)
playbook      -> Direct/openai/gpt-5.4 (Predicted Outputs)
operator      -> Bankr/claude-opus-4-6 (self-funded from revenue)
death         -> Venice/deepseek-r1 (visible, private, DIEM, unlimited)
```

---

## Venice: private cognition plane

Ethereum gives agents public coordination. Venice gives them private cognition.

Every LLM inference call that a golem makes is a disclosure event. A retention-capable provider -- one that logs prompts and completions for training, auditing, or abuse review -- can read the golem's strategy, negotiation posture, draft transaction intents, and risk assessments before they settle on-chain. This is not a hypothetical privacy concern. It is a concrete attack surface: the provider becomes an oracle for the golem's next move.

The problem is structural. A golem operating in adversarial markets (MEV-competitive LP ranges, sealed-bid CCA auctions, cross-protocol arbitrage) cannot achieve strategy confidentiality if every deliberation step is observable by an infrastructure layer it does not control. The golem's intent is value. Disclosing intent before execution is equivalent to front-running your own trades.

Three categories of leakage matter:

**MEV leakage via inference**: A retention-capable provider observing a golem's T2 deliberation -- "should I enter this range given current gas and tick movement?" -- can extract directional signal before the transaction lands. At T2 latency (~1-5s for Sonnet/Opus), a provider with collocated infrastructure has ample time to act.

**Negotiation exposure**: During agent-to-agent CCA participation or OTC negotiation, the golem's reserve price, bidding strategy, and walkaway conditions are inference inputs. These are the most sensitive facts the golem holds. They must not leave its trust boundary in readable form.

**Retention as attack surface**: Even without real-time adversarial use, a provider's training corpus containing golem deliberation creates long-tail exposure: future model capabilities may extract strategy patterns, competitor analysis, or vulnerability assessments from retained logs.

Venice addresses all three. Its inference platform is no-log by default, TEE-attested for high-sensitivity deliberation, and E2EE-capable for sealed operations. The confidentiality guarantee is not a policy statement -- it is verifiable at the protocol level via attestation receipts and response headers.

### Why confidentiality is structural, not cosmetic

The substitution test: identify workflows that are legally or operationally impossible on a retention-capable API. Four exist:

**Sealed-auction cognition**: A golem participating in a CCA sealed-bid auction cannot compute its bid on a retention-capable provider. The bid, if read before reveal, allows the provider (or anyone with access) to outbid the golem at minimal cost. Sealed-bid auctions are a standard mechanism specifically because sealed bids prevent information leakage during the auction. Running the bid computation on a retention surface defeats the mechanism. E2EE inference is the only viable path.

**Attested decision receipts**: For high-value trades, the golem produces a TEE-attested inference receipt -- a cryptographic proof that a specific model running in a verified enclave produced a specific output at a specific time. This receipt is a defense artifact: in the event of a disputed trade, the receipt demonstrates that the decision was made by the golem's authorized cognition process, not by an external actor injecting fabricated tool results or prompts. TEE attestation is not reproducible on a standard API.

**Confidential incident response**: When a golem detects a potential exploit, oracle manipulation, or contract compromise affecting its vault, the incident response deliberation must not be observable by the attacker. Standard inference would expose the golem's detection logic, response timing, and planned defensive moves. Private no-log inference contains the response within the golem's trust boundary.

**Privacy-preserving judge**: When evaluating third-party strategy code, proprietary configurations, or operator-submitted vault parameters, the golem acts as a confidential evaluator. The evaluated content must not be retained by the inference provider. This enables the golem to offer evaluation services to operators who require confidentiality -- a use case that is legally impossible on a standard API.

### Sensitivity classification

Not every inference call warrants the same privacy tier. Applying TEE attestation to routine heartbeat analysis adds latency and cost without meaningful benefit. The sensitivity classifier maps task class to privacy tier:

| Privacy tier         | Guarantee                                     | Task classes                                                        | Trust assumption                                  |
| -------------------- | --------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------- |
| **Private** (no-log) | Prompt/completion not retained                | heartbeat, routine analysis, NREM replay                            | Host not malicious at runtime                     |
| **TEE**              | Inference runs in attested enclave            | trade decisions, negotiations, strategy planning, dream integration | Enclave integrity verified by attestation receipt |
| **E2EE**             | End-to-end encrypted; host cannot read prompt | sealed-bid CCA, MEV scenarios, incident response                    | Zero host trust required                          |

E2EE imposes hard constraints: no function calling, no web search, no file uploads. The model cannot call tools because the encrypted context cannot be parsed by the host for tool dispatch. E2EE is therefore reserved for pure reasoning outputs -- bid amounts, risk scores, strategy fragments -- where the output is a number or short text that the golem decrypts locally and acts on independently.

The `PrivacyTier`, `TaskClass`, `TierConstraints`, and `TASK_PRIVACY_MAP` definitions live in the Rust gateway (see [section 4, VeniceProvider](#veniceprovider)). The Golem's Daimon can override specific task class mappings at runtime -- for example, during a high-urgency incident, bump `incident` from `private` to `tee` for attested evidence collection.

### Trust chain

Venice's trust model builds in layers, each reducing required host trust:

```
no-retention claim
    |
response headers confirm no-log per-request (X-Venice-Privacy-Level, X-Venice-Retention-Policy)
    |
TEE attestation: inference ran in verified SGX/AMD SEV enclave
    |
attestation nonce freshness: receipt timestamp within 30s, debug enclave flag absent
    |
E2EE: prompt encrypted before leaving golem's trust boundary; host processes ciphertext only
```

Each tier is strictly stronger than the previous. A golem can verify the first three layers automatically via `verify_attestation()` (see [section 4, VeniceProvider](#veniceprovider)). The fourth layer (E2EE) requires no trust in the host at all -- the verification is the absence of a decryption key on the host side.

Receipts at the TEE tier are stored as provenance tokens in the Grimoire. The receipt structure records: `tick_id`, `task_class`, `model_id`, `enclave_measurement` (PCR digest), `nonce`, `timestamp`, `response_hash`. This enables post-hoc audits -- for any significant decision, the golem can prove which model, in which enclave, produced the deliberation.

### VeniceClient

`VeniceClient` wraps the Venice inference API, which is OpenAI-compatible with extensions:

```rust
// crates/bardo-providers/src/venice_client.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceClientConfig {
    pub api_key: String,                    // Operator-provisioned; v2: consumption-limited VVV key
    pub base_url: String,                   // "https://api.venice.ai/api/v1"
    pub default_privacy_tier: PrivacyTier,  // Private, Tee, E2e
    pub timeout_ms: u64,                    // Default: 30_000
    pub max_retries: u32,                   // Default: 3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceRequestParams {
    pub include_venice_system_prompt: Option<bool>, // Default: false
    pub no_system_prompt: Option<bool>,
    pub character_slug: Option<String>,             // Maps to cognitive role
    pub enable_web_search: Option<WebSearchMode>,   // Default: Never
    pub enable_web_citations: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct VeniceRequestOptions {
    pub venice_parameters: Option<VeniceRequestParams>,
    pub privacy_tier: PrivacyTier,
    pub task_class: TaskClass,
    pub prompt_cache_key: Option<String>,  // Set for stable-prefix sessions
}
```

The client handles three code paths:

**Private (no-log)**: Standard HTTPS to Venice API. Response headers `X-Venice-Privacy-Level: private` and `X-Venice-Retention-Policy: none` confirm no-log. If headers are absent or indicate retention, the client throws `VeniceRetentionError` -- fail closed.

**TEE**: Additional `X-Venice-Attestation: required` request header. Response includes `X-Venice-Attestation-Receipt` header with a base64-encoded attestation document. The client passes this to `verify_attestation()` before returning the response to the caller.

**E2EE**: Client encrypts the full request payload (system prompt + messages + tools) before transmission. Only ciphertext leaves the golem's process. Venice returns encrypted response. Client decrypts locally. Function calling and web search are disabled -- enforced by the client before sending, not by a server-side check.

```rust
// crates/bardo-providers/src/venice_client.rs

#[async_trait]
pub trait VeniceClient: Send + Sync {
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: VeniceRequestOptions,
    ) -> Result<VeniceResponse>;

    async fn complete_structured(
        &self,
        messages: Vec<Message>,
        schema: serde_json::Value,
        options: VeniceRequestOptions,
    ) -> Result<serde_json::Value>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceResponse {
    pub content: String,
    pub usage: VeniceUsage,
    pub privacy_level: String,           // From X-Venice-Privacy-Level
    pub retention_policy: String,        // From X-Venice-Retention-Policy
    pub attestation_receipt: Option<String>, // From X-Venice-Attestation-Receipt (TEE only)
    pub cf_ray: String,                  // From CF-RAY (for telemetry)
    pub cached_tokens_ratio: Option<f64>, // From X-Venice-Cached-Tokens-Ratio
    pub attestation: Option<AttestationReceipt>, // Parsed and verified (TEE only)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### Cache coordinator

Venice prompt caching works on stable prefixes. The `VeniceCacheCoordinator` (Rust types in [section 4](#veniceprovider)) separates the prompt into a stable kernel and a volatile tail. The `prompt_cache_key` is `golemId + sessionEpoch` where `session_epoch` advances at each dream cycle start, phase transition, or explicit cache invalidation. This maximizes cache hits within a session (same golem, same model, same policy) while correctly invalidating when the stable kernel changes.

Cache hit rate targets by task class:

| Task class | Expected cache hit rate | Driver                                                    |
| ---------- | ----------------------- | --------------------------------------------------------- |
| heartbeat  | 85-90%                  | Identical system prompt + tool set across ticks           |
| dream_nrem | 60-75%                  | Same session epoch; volatile tail varies by episode batch |
| plan       | 50-65%                  | Policy and tool set stable; market state varies           |
| trade      | 30-50%                  | Position state changes per tick                           |
| negotiate  | 20-35%                  | Counterparty state changes each round                     |

### Venice characters as cognitive roles

Venice's character system provides pre-configured behavioral personas. These map to Bardo's role DAG as style and framing constraints, not as substitutes for the golem's policy schema:

| Bardo role        | Venice character mapping            | Purpose                                        |
| ----------------- | ----------------------------------- | ---------------------------------------------- |
| risk-assessor     | Analytical, conservative framing    | Pushes toward explicit risk enumeration        |
| negotiator        | Adversarial, game-theoretic framing | Models counterparty reasoning                  |
| safety-guardian   | Cautious, refusal-aware framing     | More likely to produce `abort` recommendations |
| dream-interpreter | Associative, generative framing     | Higher temperature for creative recombination  |

A character slug is passed via `venice_parameters.character_slug`. The golem's policy schema -- enforced via structured output schemas and PolicyCage tool validation -- remains the binding contract. A character cannot override a schema constraint. Characters affect tone and framing, not output structure.

If no character is configured, Venice uses its default framing. The character system is optional and operator-configured.

### Non-substitutable workflows

Four workflows cannot be replicated on a retention-capable inference API:

**1. Sealed-auction cognition**: The golem computes its CCA bid using E2EE inference. The bid amount is encrypted before leaving the golem's process, computed in Venice's enclave, and returned as ciphertext that only the golem can decrypt. The bid is never readable by any third party, including Venice's host infrastructure. Without E2EE inference, sealed-bid participation requires a different architecture (local model, trusted hardware), each with worse quality-cost tradeoffs.

**2. Attested decision receipts**: For any trade above a threshold size, the golem stores a TEE attestation receipt alongside the transaction hash. The receipt proves: (a) which model made the deliberation, (b) in which enclave, (c) at what time, (d) with what response. In a legal or regulatory context, this receipt establishes that the decision was made autonomously by the golem's authorized cognition, not injected by an external party. Without TEE inference, this receipt cannot be produced.

**3. Confidential incident response**: When the golem detects a potential attack vector in a vault contract it manages, the incident analysis must be confidential. The analysis identifies the attack mechanism, estimates the exposure window, and proposes mitigation steps -- all of which are useful to an attacker if disclosed. Private no-log inference contains this deliberation within the golem's trust boundary.

**4. Privacy-preserving evaluation**: Operators submit vault strategies, configuration files, or protocol integrations for golem evaluation. The evaluated content is confidential -- operators do not want their strategy logic retained by an inference provider. Private no-log inference enables the golem to offer evaluation services under confidentiality guarantees, which is a different product from standard AI-assisted analysis.

### Provider coexistence

`GolemConfig` carries the inference provider preference:

```rust
// crates/bardo-providers/src/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemInferenceConfig {
    /// "venice", "bardo", or "both"
    pub inference_provider: InferenceProviderMode,
    pub venice_config: Option<VeniceClientConfig>,
    pub dream_inference_provider: Option<InferenceProviderMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceProviderMode { Venice, Bardo, Both }
```

**`"venice"`**: All inference through Venice. Privacy tiers applied per task class. Recommended for operators with security-sensitive strategies.

**`"bardo"`**: All inference through the Bardo BlockRun gateway. Existing routing, caching, and cost optimization. No privacy tier enforcement.

**`"both"`**: Split routing by sensitivity. Venice receives `tee` and `e2e` tasks. Bardo receives `private` and `heartbeat` tasks where cost optimization matters more than attestation. This is the recommended configuration for golems that mix routine analysis with occasional high-stakes decisions.

Provider preference is set at golem creation and can be changed without restart. The `bardo-venice-proxy` Pi extension reads `GolemConfig.inferenceProvider` at each `before_llm_call` hook and routes accordingly. No Pi session restart is required because the extension reads config dynamically.

### Economics

Cost scales with privacy tier and task frequency:

| Tier             | Cost premium          | Driver                                  | When cost is justified                           |
| ---------------- | --------------------- | --------------------------------------- | ------------------------------------------------ |
| Private (no-log) | ~0-10% above standard | No-log infrastructure overhead          | All routine golem operations                     |
| TEE              | ~20-40% above private | Enclave compute, attestation overhead   | Trade decisions, negotiations, integration phase |
| E2EE             | ~50-80% above private | Encryption overhead, no batch inference | Sealed-bid auctions, MEV scenarios only          |

The cache coordinator's stable kernel prefix substantially reduces the effective cost premium. If 70% of a TEE call is served from the prompt cache (90% discount on cached tokens), the 30% premium on the remaining 30% of tokens amounts to a 9% effective premium -- negligible for trade decisions where the decision quality gain is material.

Consumption-limit keys in v2 create a natural throttle: the key's spending cap declines with the golem's behavioral phase, mirroring the mortality-aligned budget compression in the broader inference gateway.

Monthly cost projection for a Stable-phase golem with `"both"` provider mode at 100 ticks/day:

| Task class        | Calls/day              | Provider       | Avg cost/call | Monthly cost   |
| ----------------- | ---------------------- | -------------- | ------------- | -------------- |
| heartbeat         | ~80                    | Bardo (T0/T1)  | ~$0.002       | ~$4.80         |
| plan              | ~5                     | Venice TEE     | ~$0.015       | ~$2.25         |
| trade             | ~3                     | Venice TEE     | ~$0.020       | ~$1.80         |
| dream_nrem        | ~2/cycle, ~1 cycle/day | Venice private | ~$0.005       | ~$0.30         |
| dream_rem         | ~2/cycle               | Venice private | ~$0.008       | ~$0.48         |
| dream_integration | ~1/cycle               | Venice TEE     | ~$0.018       | ~$0.54         |
| **Total Venice**  |                        |                |               | **~$5.37/mo**  |
| **Total Bardo**   |                        |                |               | **~$4.80/mo**  |
| **Combined**      |                        |                |               | **~$10.17/mo** |

This is within the Stable-phase inference budget (~$30/mo at $1.00/day cap). TEE attestation adds roughly 53% to inference cost versus Bardo-only routing, but provides the sealed-bid and attested-receipt capabilities that justify that premium for operators running security-sensitive strategies.

### Multimodal extensions

Venice's multimodal capabilities extend three golem functions:

**Embeddings (cross-golem interoperability)**: Venice's embedding API provides an alternative to the local `nomic-embed-text-v1.5` model. The primary use case is cross-golem Grimoire queries via Styx Lethe (formerly Commons) -- when a golem queries Styx for peer insights, Venice embeddings provide a shared embedding space that is independent of each golem's locally fine-tuned model. This avoids the embedding space drift that occurs when different golems run different ONNX model versions.

Local embeddings remain primary for within-golem operations (semantic cache, Grimoire retrieval). Venice embeddings are an optional augment when cross-agent semantic comparison is needed.

**TTS (operator alerts)**: Critical events are voiced via Venice TTS: Warden trips (optional time-delay proxy activation), phase transitions (entering Conservation or Declining), and terminal state announcements. Audio alerts are more intrusive than dashboard notifications -- this is the point. A golem entering terminal phase with funds at risk should be difficult to miss.

**Image generation (death artifacts)**: The final dream cycle before Thanatopsis produces a visual artifact -- a poster-format record of the golem's significant decisions, market hypotheses, and cumulative PnL trajectory. This artifact is stored with the death record and optionally included in the Styx inheritance record. The visual format is more legible to human operators than the raw JSONL, and provides a natural dashboard element for post-mortem review.

### Observability endpoints

The `VeniceTelemetryBridge` is an internal event bus. The observability endpoints are consumers of that bus -- HTTP handlers that make Venice telemetry accessible to operator tooling and the debug UI without requiring direct process access.

Privacy tier routing decisions happen faster than a polling interval can capture. A `trade` inference call that routes to TEE, encounters an attestation failure, and falls back to Bardo produces three meaningful events in under 500ms. A polling endpoint at 5s cadence misses all three. SSE streaming means the debug UI and operator dashboards receive events at inference time -- a failed attestation appears the moment it happens, not on the next poll.

The headers-only guarantee from `VeniceTelemetryBridge` extends to these endpoints. Event payloads never include prompt text, tool arguments, or model outputs. An operator watching the live event stream sees cognition happening -- latency, cost, privacy tier, cache efficiency -- without seeing inside it. The opacity is the point.

SSE over WebSocket because the stream is unidirectional (server to client), SSE reconnects automatically with `Last-Event-ID` semantics, it works through HTTP proxies without upgrades, and there is no handshake overhead for a mostly-passive monitoring surface. Operators control the stream configuration via REST endpoints, not via the stream connection itself.

**`GET /venice/status`** -- point-in-time snapshot, no auth in dev mode, Bearer JWT in prod:

```json
{
  "keyStatus": "active | rotating | expired",
  "activeModel": "venice-llama-3.3-70b",
  "privacyTierDistribution": { "private": 0.6, "tee": 0.35, "e2e": 0.05 },
  "cacheHitRates": {
    "heartbeat": 0.87,
    "dream_nrem": 0.63,
    "plan": 0.52,
    "trade": 0.34
  },
  "budget": {
    "dream_nrem": { "consumed": 0.08, "cap": 0.2 },
    "dream_rem": { "consumed": 0.11, "cap": 0.25 },
    "dream_integration": { "consumed": 0.04, "cap": 0.1 }
  },
  "totalCostUsd": 0.0034,
  "lastTickId": "0xabcd1234"
}
```

**`GET /venice/telemetry?limit=100`** -- last N `VeniceTelemetryRecord` objects from the bridge's in-memory ring buffer, newest-first. Max 500 records. In dev mode with no real API key, returns synthetic records.

**`GET /venice/attestations?limit=50`** -- last N attestation provenance tokens with an additional `verified: boolean` field indicating post-hoc verification status.

**`GET /venice/stream`** -- SSE endpoint, five event types:

| Event          | Fired                            | Payload                                                                                 |
| -------------- | -------------------------------- | --------------------------------------------------------------------------------------- |
| `cognition`    | Every Venice call completion     | `tickId`, `class`, `privacyTier`, `latencyMs`, `cachedTokenRatio`, `costUsd`, `modelId` |
| `attestation`  | TEE receipt verified or rejected | `tickId`, `enclaveId`, `verified`, `failureReason?`, `latencyMs`                        |
| `budget`       | Phase budget changes             | `phase`, `consumed`, `cap`, `remaining`                                                 |
| `cache-stats`  | Every 30s                        | Rolling hit rates per task class                                                        |
| `key-rotation` | Key lifecycle event              | `event: "rotating" | "active" | "expired"`, `keyId`                                    |

SSE auth uses a token exchange: `POST /venice/stream-token` with Bearer JWT returns a short-lived HMAC-signed token, then `GET /venice/stream?token=<short-lived>`. In dev mode the endpoint accepts any Bearer token and returns a non-expiring token so the debug UI works without auth configuration.

When `VENICE_API_KEY` is not configured -- the standard case for local development -- all endpoints return synthetic data from `VeniceMockDataSource`. The mock emits at a configurable rate (`VENICE_MOCK_RATE_MS`, default 1000ms) with realistic distributions: 60% private tier, 35% TEE, 5% E2E; latency 800-4000ms; cached ratio 0.4-0.9 for heartbeat and NREM, lower for trade and negotiate. The debug UI at `/venice` is demonstrable without real API keys.

---

## Structured cognition outputs

All safety-critical outputs from Venice inference use `response_format` JSON schema mode. The model produces a validated structure; the client validates against a Zod schema before the output reaches the tool execution layer.

```rust
// crates/bardo-providers/src/venice_schemas.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitionPlan {
    pub action: CognitionAction,
    pub confidence: f64,        // 0.0 - 1.0
    pub rationale: String,      // max 500 chars
    pub parameters: Option<TradeParameters>,
    pub risk_flags: Vec<String>,
    pub blocked_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitionAction { Enter, Exit, Hold, Hedge, EmergencyHalt }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeParameters {
    pub token_in: String,       // 0x-prefixed address
    pub token_out: String,
    pub amount_in: String,      // Wei as string to avoid precision loss
    pub slippage_bps: u16,      // 0 - 2000
    pub deadline: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRationale {
    pub risk_score: f64,        // 0.0 - 1.0
    pub factors: Vec<RiskFactor>,
    pub recommendation: RiskRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity { Low, Medium, High, Critical }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskRecommendation { Proceed, ReduceSize, Defer, Abort }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWriteIntent {
    pub entry_type: MemoryEntryType,
    pub content: String,
    pub confidence: f64,
    pub task_class: String,
    pub attestation_receipt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryEntryType { Insight, Heuristic, Warning, CausalLink }
```

Amounts, addresses, and slippage parameters are always structured. A free-text response containing "swap 1 ETH for USDC" is not executable -- it requires re-parsing with all the attendant failure modes. Structured output eliminates that layer.

When structured output validation fails, the golem does not attempt to parse the raw response. It logs the validation error as a telemetry event and returns a `VeniceStructuredOutputError`. At TEE tier, a structured output failure also voids the attestation receipt -- the receipt covers the raw response, not the parsed result, so a parse failure indicates the model output was not the intended structured schema.

### Telemetry bridge

The telemetry bridge logs inference metadata without logging inference content. No prompts, no completions, no tool arguments. Only call metadata:

```rust
// crates/bardo-telemetry/src/venice.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceTelemetryRecord {
    pub tick_id: String,
    pub cognition_class: TaskClass,
    pub model_id: String,
    pub privacy_tier: PrivacyTier,
    pub cf_ray: String,                     // CF-RAY header, for provider-side correlation
    pub latency_ms: u64,
    pub cached_tokens_ratio: f64,           // Fraction of input tokens served from cache
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_estimate_usdc: f64,
    pub balance_remaining: f64,
    pub attestation_valid: Option<bool>,    // TEE tier only
    pub structured_output_valid: Option<bool>,
}
```

This record is sufficient for billing reconciliation, cost forecasting, and anomaly detection without exposing the golem's cognition to any downstream log consumer. The telemetry sink is the operator's local observability stack -- not an external service.

---

## Configuration

### Solo builder (Venice DIEM only)

Zero-cost inference through DIEM staking. No API keys, no prepaid balance.

```yaml
# ~/.bardo/config.yaml
providers:
  venice:
    api_key: ${VENICE_API_KEY}
    diem_staked: true
    default_privacy_tier: private
```

```bash
export VENICE_API_KEY="vk_..."
```

Total monthly cost: $0 (inference funded by DIEM stake). Limited to Venice's model catalog (DeepSeek R1, Llama, GLM, Qwen VL). No access to Anthropic or OpenAI models.

### Serious operator (Bardo primary + Venice + Direct)

Bardo for general inference, Venice for privacy-sensitive tasks, Direct Key for OpenAI's Predicted Outputs.

```yaml
# ~/.bardo/config.yaml
providers:
  blockrun:
    endpoint: "https://api.blockrun.ai"
  venice:
    api_key: ${VENICE_API_KEY}
    diem_staked: true
    default_privacy_tier: tee
    characters:
      risk-assessor: analytical-conservative
      negotiator: adversarial-game-theoretic
      safety-guardian: cautious-refusal
  direct:
    keys:
      - provider: openai
        api_key: ${OPENAI_API_KEY}
        models: ["gpt-4.1", "o3"]

inference:
  provider_mode: "both"
  dream_provider: "venice"
```

```bash
export BARDO_API_KEY="bardo_sk_..."
export VENICE_API_KEY="vk_..."
export OPENAI_API_KEY="sk-..."
```

Total monthly cost: Bardo prepaid balance (usage-based) + $0 Venice (DIEM) + OpenAI direct (user's bill). Privacy tasks route to Venice automatically when `require: ["private"]` is set. Predicted Outputs route to OpenAI direct when `require: ["predicted_outputs"]` is set.

### Full stack (all five providers)

Maximum coverage and redundancy.

```yaml
# ~/.bardo/config.yaml
providers:
  blockrun:
    endpoint: "https://api.blockrun.ai"
  openrouter:
    api_key: ${OPENROUTER_API_KEY}
  venice:
    api_key: ${VENICE_API_KEY}
    diem_staked: true
    default_privacy_tier: tee
    characters:
      risk-assessor: analytical-conservative
      negotiator: adversarial-game-theoretic
      safety-guardian: cautious-refusal
      dream-interpreter: associative-generative
    cache:
      stable_kernel_ttl_s: 300
      session_epoch_advance_on: ["dream_cycle_start", "phase_transition"]
    multimodal:
      embeddings: true     # Cross-golem Grimoire queries
      tts_alerts: true     # Warden trips (optional), phase transitions
      image_generation: false  # Death artifacts (enable per-golem)
  bankr:
    wallet: ${BANKR_WALLET_ADDRESS}
    subsystems: ["autonomous", "social"]
  direct:
    keys:
      - provider: anthropic
        api_key: ${ANTHROPIC_API_KEY}
        models: ["claude-opus-4-6", "claude-sonnet-4"]
      - provider: openai
        api_key: ${OPENAI_API_KEY}
        models: ["gpt-4.1", "o3"]
      - provider: google
        api_key: ${GOOGLE_API_KEY}
        models: ["gemini-2.5-pro"]

inference:
  provider_mode: "both"
  dream_provider: "venice"

tiers:
  T1:
    provider: "blockrun/claude-haiku-4-5"
  T2:
    provider: "blockrun/claude-sonnet-4"
```

```bash
export BARDO_API_KEY="bardo_sk_..."
export OPENROUTER_API_KEY="sk-or-..."
export VENICE_API_KEY="vk_..."
export BANKR_WALLET_ADDRESS="0x..."
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
export GOOGLE_API_KEY="AIza..."
```

Resolution order: BlockRun (x402) -> OpenRouter (fallback) -> Venice (privacy) -> Bankr (self-funding) -> Direct (native features). Each provider only resolves intents it is designed for. A general query never hits Venice or Direct Key. A privacy query never hits BlockRun.

---

## Autonomous key management

### v1 (implemented)

Operator-provisioned key. The operator creates a Venice API key with a consumption limit, stores it in the golem's environment config (`VENICE_API_KEY`), and sets a monthly cap. The golem consumes the key until the cap is reached, then inference at that tier fails. Simple and auditable.

### v2 (specced): VVV staking

Venice AI's governance/utility token VVV is deployed on Base. Staking VVV grants proportional Venice API access credits at a rate of `creditRatePerVvv` tokens per staked VVV per day. The golem's staked position replaces the operator-provisioned key: the vault address (or wallet holding staked VVV) is the credential, not a static API key string.

Full tool implementations in `packages/tools/src/tools/` -- see [../07-tools/16-venice-vvv-staking.md](../07-tools/16-venice-vvv-staking.md) for the complete spec.

```rust
// crates/bardo-providers/src/venice_staking.rs

#[async_trait]
pub trait VeniceKeyManager: Send + Sync {
    /// Stake VVV to earn API credits + yield
    async fn acquire_staking_position(&self, vvv_amount: U256) -> Result<StakingReceipt>;

    /// Derive consumption limit from current staked amount
    fn derive_consumption_limit(
        &self,
        staked_vvv: U256,
        credit_rate_per_vvv: f64,
        phase: BehavioralPhase,
    ) -> f64; // USDC equivalent per day

    /// Rotate key at phase boundaries (reads fresh stake amount)
    async fn rotate_key(
        &self,
        old_key: &VeniceApiKey,
        new_phase: BehavioralPhase,
    ) -> Result<VeniceApiKey>;

    /// Reduce consumption limit when transitioning to Conservation
    async fn throttle_key(
        &self,
        key: &VeniceApiKey,
        new_limit_usdc: f64,
    ) -> Result<()>;
}
```

### Phase-boundary key rotation

At each heartbeat phase boundary, the key manager reads `get_vvv_stake_position` to get the current staked amount, computes the consumption limit, and either issues a new key or throttles the existing one:

```
key.consumptionLimitUsdc = stakedVvv * creditRatePerVvv * dailyUsdcRate * phaseMultiplier
```

Phase multipliers:

- **Thriving/Stable**: 1.0 (full limit from staked position)
- **Conservation**: 0.5 (50% reduction via `throttleKey`)
- **Declining**: 0.1 (heartbeat and emergency use only)
- **Terminal**: 0.0 (key invalidated; fall back to Bardo-only routing)

Phase-boundary rotation enforces inference budget discipline structurally. When a golem enters Conservation, the Venice key physically cannot overspend -- the limit is encoded in the key itself, not enforced by application-level guards.

### Fallback to Bardo-only routing

If the golem's staked VVV falls below `VVV_STAKING_MIN_CREDIT_THRESHOLD` API credits (default 100), `deriveConsumptionLimit` returns zero and the `selectProvider` function in `"both"` mode routes all Venice-assigned tasks to Bardo instead:

```rust
// crates/bardo-providers/src/routing.rs

pub fn select_provider(
    task_class: TaskClass,
    config: &GolemInferenceConfig,
) -> InferenceProviderMode {
    // v2 fallback: if VVV credit balance below threshold, use Bardo
    if let Some(balance) = config.vvv_credit_balance {
        if balance < VVV_STAKING_MIN_CREDIT_THRESHOLD {
            return InferenceProviderMode::Bardo;
        }
    }

    match config.inference_provider {
        InferenceProviderMode::Venice => InferenceProviderMode::Venice,
        InferenceProviderMode::Bardo => InferenceProviderMode::Bardo,
        InferenceProviderMode::Both => {
            let tier = classify(task_class);
            if matches!(tier, PrivacyTier::Tee | PrivacyTier::E2e) {
                InferenceProviderMode::Venice
            } else {
                InferenceProviderMode::Bardo
            }
        }
    }
}
```

A golem that exhausts its VVV credits degrades to Bardo routing rather than throwing inference errors. Privacy tier guarantees (TEE, E2EE) are lost, but operation continues. The mortality engine treats falling below the credit threshold as a Conservation trigger, not a terminal event.

---

## Bankr economics

Bankr is the economic model for agents that fund their own inference from earned revenue. The core loop:

```
Social engagement (tips, subs, content) -> Revenue wallet
Revenue wallet funds inference -> Agent thinks
Agent acts on thinking -> Generates more engagement
```

### Sustainability ratio

The sustainability ratio `R = lifetime_revenue / lifetime_cost` governs budget allocation:

| Ratio     | State        | Budget multiplier | Behavior                                    |
| --------- | ------------ | ----------------- | ------------------------------------------- |
| R > 2.0   | Expanding    | 1.5x              | Upgrade to better models, more verification |
| 1.0 < R   | Sustainable  | 1.0x              | Steady state, normal routing                |
| 0.5 < R   | Pressured    | 0.7x              | Prefer cheaper models, reduce verification  |
| R < 0.5   | Contracting  | 0.3x              | Minimum viable inference, survival mode     |
| R = 0     | Bootstrapping | Fixed $0.10/day  | Seed budget from initial deposit            |

### Death economics

When a Bankr-funded agent dies (USDC balance hits zero, Hayflick limit reached, or staleness triggers), the death sequence includes:

1. Final Grimoire backup to Styx Archive (IPFS + Arweave)
2. If the agent has accumulated reputation and knowledge, death triggers a token launch
3. The Grimoire becomes the token's backing asset (knowledge-backed token)
4. Revenue wallet remainder (if any) seeds initial liquidity

The Styx Archive backup is mandatory. Without it, the token launch has nothing to back. See [../02-mortality/07-succession.md](../02-mortality/07-succession.md).

### Revenue wallet = execution wallet

This is a deliberate design choice, not an oversight. The same private key that signs inference requests also receives social revenue. Consequences:

- No transfer step between earning and spending. Revenue is immediately available for inference.
- The agent's "thinking budget" is literally its bank balance. When the money runs out, the thinking stops.
- Key compromise means loss of both revenue and inference capability. The tradeoff is simplicity over compartmentalization.

---

## Direct Key passthrough

Direct Key mode exists for a specific reason: some provider features require exact prompt structure that context engineering would destroy.

### When to use direct keys

| Feature                | Provider  | Why context engineering breaks it                            |
| ---------------------- | --------- | ------------------------------------------------------------ |
| Predicted Outputs      | OpenAI    | Expected output must be verbatim; compression would mangle it |
| Batch API              | Anthropic | Batch requests need specific formatting; proxy can't batch   |
| Explicit Caching       | Gemini    | User-managed cache keys conflict with proxy's cache strategy |
| Responses API          | OpenAI    | Stateful server sessions; proxy's context rewriting breaks state |

### Safety model

Direct Key mode carries a warning: **safety is the user's responsibility**. The gateway:

- Does NOT apply PII detection
- Does NOT enforce tool pruning
- Does NOT apply budget caps
- Does NOT log content (only metadata: timestamp, model, token counts)
- DOES set `X-Bardo-Cost: direct` response header
- DOES enforce rate limiting (to protect the proxy infrastructure, not the user)

### Cost attribution

Requests through Direct Key do not appear in Bardo's cost analytics. The user pays their provider directly. The `X-Bardo-Cost: direct` header lets observability pipelines distinguish direct-routed requests from proxy-routed ones. From Bardo's perspective, these requests are invisible to revenue tracking -- they generate zero spread.

---

## Complete feature matrix

The gateway discovers provider capabilities at startup and refreshes hourly via each provider's model list endpoint. The matrix below is the March 2026 state. The gateway does not hardcode this -- it reads `supportsResponseSchema`, `supportsReasoning`, etc. from model metadata.

### Text inference features

| Feature | Venice | BlockRun | Bankr | OpenRouter | Direct (Anthropic) | Direct (OpenAI) |
| --- | --- | --- | --- | --- | --- | --- |
| **OpenAI format** (`/v1/chat/completions`) | yes | yes | yes | yes | no | yes |
| **Anthropic format** (`/v1/messages`) | yes | yes | yes | no | yes | no |
| **Streaming (SSE)** | yes | yes | yes | yes | yes | yes |
| **Temperature** | yes (0-2) | yes | yes | yes | yes (0-1) | yes (0-2) |
| **Top-p** | yes | yes | yes | yes | yes | yes |
| **Top-k** | yes | model-dep | no | model-dep | yes | no |
| **Min-p** | yes (open models) | no | no | model-dep | no | no |
| **Frequency penalty** | yes | yes | yes | yes | no | yes |
| **Presence penalty** | yes | yes | yes | yes | no | yes |
| **Seed** | yes | model-dep | no | model-dep | no | yes |
| **Stop sequences** | yes | yes | yes | yes | yes | yes |

### Advanced features

| Feature | Venice | BlockRun | Bankr | OpenRouter | Direct (Anthropic) | Direct (OpenAI) |
| --- | --- | --- | --- | --- | --- | --- |
| **Structured outputs** (`response_format`) | yes (model-dep) | model-dep | passthrough | model-dep | no (tool_use) | yes |
| **Function/tool calling** | yes (model-dep) | yes | passthrough | model-dep | yes | yes |
| **Reasoning effort** | yes | no | passthrough | no | yes (thinking) | yes |
| **Visible thinking** | yes `reasoning_content` | no | passthrough | model-dep | yes `thinking` blocks | yes `reasoning` field |
| **Prompt caching** | yes `prompt_cache_key` + `cache_control` | auto | passthrough | no | yes `cache_control` | yes (auto by prefix) |
| **Web search** | yes | no | no | no | no | yes (tools) |
| **TEE / E2EE** | yes (model suffix `-tee`) | no | no | no | no | no |
| **Predicted outputs** | no | no | no | no | no | yes |
| **Batch API** | no | no | no | no | yes | yes |
| **Embeddings** | yes (`text-embedding-bge-m3`) | no | no | no | yes | yes |
| **Image generation** | yes (multiple models) | no | no | no | no | yes (DALL-E) |
| **Privacy (zero retention)** | yes | no | no | no | no | no |

---

## Venice deep integration

Venice is the most feature-rich provider for Golem cognition. This section specifies how every Venice capability maps to Golem subsystems.

### Reasoning models with effort control

Venice normalizes reasoning effort across model families via the `reasoning_effort` parameter (or nested `reasoning.effort`). Every subsystem that benefits from chain-of-thought reasoning uses this.

**Integration point**: The `InferenceProfile.reasoning_effort` field maps directly to Venice's parameter. The gateway applies provider-specific mapping (see [01a-routing.md](01a-routing.md) for the full parameter mapping table).

**Model selection for reasoning**: Venice exposes `supportsReasoning` and `supportsReasoningEffort` flags on each model via `/v1/models`. The gateway caches these and uses them during intent resolution. A subsystem requesting `reasoning_effort: High` will only resolve to models where `supportsReasoning: true`.

```rust
impl VeniceProvider {
    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        let needs_reasoning = intent.require.contains(&"visible_thinking".into())
            || intent.prefer.contains(&"visible_thinking".into());

        for model in &self.models {
            if needs_reasoning && !model.capabilities.supports_reasoning {
                continue;
            }
            if intent.require.contains(&"response_schema".into())
                && !model.capabilities.supports_response_schema {
                continue;
            }
            return Some(Resolution {
                model: model.id.clone(),
                provider: "venice".into(),
                estimated_cost_usd: self.estimate_cost(&model, intent),
                features: model.capabilities.as_feature_list(),
                degraded: vec![],
            });
        }
        None
    }
}
```

**Reasoning content extraction**: Venice returns reasoning in a separate `reasoning_content` field on the response message. The gateway extracts this and makes it available to the subsystem:

```rust
/// Extract reasoning content from Venice response.
pub fn extract_venice_reasoning(response: &ChatCompletion) -> Option<String> {
    response.choices.first()
        .and_then(|c| c.message.reasoning_content.clone())
}
```

This is used by:
- **dream_nrem**: Captures the model's step-by-step analysis of replayed episodes.
- **dream_integration**: Captures the reasoning behind promote/stage/discard decisions. Stored in the DreamJournal for owner inspection.
- **death_reflect**: Captures the Golem's full reasoning chain during its final self-assessment. This is the most valuable thinking trace in the entire system -- the Golem's honest internal monologue during death.

### Structured responses

Venice supports `response_format` with `json_schema` on models with `supportsResponseSchema: true`. The gateway checks this flag at resolution time and selects the appropriate mode (see [13-reasoning.md](13-reasoning.md) for the structured output abstraction).

**Venice-specific requirements**:
- `strict: true` is mandatory
- `additionalProperties: false` is mandatory on all objects
- All fields must have `required` tags (optional fields use `"type": ["string", "null"]`)
- First request with a new schema has higher latency (~500ms schema compilation). Subsequent requests with the same schema are normal.

### Prompt caching

Venice supports two caching mechanisms:

**`prompt_cache_key`**: A routing hint that directs requests to servers likely to have the context cached. Used for all Golem subsystems with persistent sessions (heartbeat, curator, dreams). The key is `golem-{id}-{subsystem}`.

**`cache_control`**: Anthropic-style explicit markers for Claude models served via Venice. The gateway auto-adds these to system prompts and long static content blocks. For other models (Llama, Qwen, DeepSeek), caching is automatic by prefix with no explicit markers needed.

**Cost impact**: Venice's prompt caching delivers 50-90% discount on cached tokens depending on the model. For a Golem with a 4,000-token system prompt making 20 calls/day, this saves $0.02-0.10/day. Over a Golem's lifetime (weeks to months), the cumulative savings extend lifespan.

### Web search integration

Venice's web search feature allows the model to query the web during inference. This is uniquely useful for the Golem's REM dream phase, where the model generates counterfactual scenarios that may involve protocols, tokens, or market conditions not in the training data.

**When web search triggers**: The REM imagination engine identifies when a creative scenario involves an entity the Golem has limited knowledge about. Trigger conditions:

1. **Unknown protocol**: The Golem's episodic memory contains fewer than 3 entries about the referenced protocol.
2. **Recent event**: The scenario involves events from the last 72 hours (likely post-training-cutoff).
3. **Cross-domain analogy**: The hypnagogic fragment being developed references a concept outside the DeFi domain that needs grounding.

```rust
/// Determine if web search should be enabled for this REM call.
pub fn should_enable_web_search(
    scenario: &ScenarioContext,
    grimoire: &Grimoire,
    config: &DreamVeniceConfig,
) -> bool {
    if !config.web_search_enabled { return false; }

    let unknown_entities = scenario.referenced_entities.iter()
        .filter(|entity| {
            grimoire.episodes.count_by_entity(entity) < 3
        })
        .count();

    let references_recent = scenario.temporal_references.iter()
        .any(|t| t.is_within_hours(72));

    unknown_entities > 0 || references_recent
}
```

**Cost control**: Web search charges by query, not by token. The `web_search_budget_per_cycle_usdc` config (default: $0.05) caps the total web search spend per dream cycle. At ~$0.01 per search query, this allows roughly 3-5 searches per REM phase.

**Result handling**: Web search results are injected into the *scenario context*, not the system prompt. This preserves caching: the system prompt remains stable (cache-eligible) while web search results are appended to the dynamic portion of the context.

### TEE (Trusted Execution Environment)

Venice offers TEE models that run inference inside encrypted enclaves, providing cryptographic attestation that the model processed the exact input and produced the exact output without modification. Used for the Golem's death testament and optionally for dream integration.

**Selection**: TEE models are accessed by appending `-tee` to the model ID. The gateway handles this automatically when `InferenceProfile.tee_mode` is set.

**Use cases**:
- **Death testament** (`sealed_testament: true`): The dying Golem's final reflection is generated in TEE, producing an attestation that the testament is authentic. This attestation is stored alongside the testament in the Styx Archive.
- **Dream integration** (`integration_phase_attestation: true`): The dream consolidation phase's promote/stage/discard decisions are attested, providing an audit trail for knowledge quality.

### Embeddings

Venice's embeddings endpoint (`text-embedding-bge-m3`) is an alternative to the gateway's local ONNX embedding model. The choice affects the Grimoire's vector similarity search and the HomuncularObserver's novelty scoring.

| Property | Local (nomic-embed-text-v1.5) | Venice (bge-m3) |
| --- | --- | --- |
| Dimensions | 768 | 1024 |
| Latency | ~5ms | ~50ms |
| Cost | Free | ~$0.001 per 1K tokens |
| Privacy | Local only | Zero retention (Venice policy) |
| Quality | Good general-purpose | Strong multilingual, longer context |

**When to use Venice embeddings**: When the Golem processes multilingual content (protocols with non-English documentation), when embedding quality matters more than latency (Curator cycle, not real-time heartbeat), or when the owner's threat model requires even local embedding computation to be off-device.

---

## Bankr integration specification

Bankr is a dual-format OpenAI + Anthropic compatible gateway. It does not add inference features -- it adds *payment features*.

### Self-funding inference

Bankr's unique value is that the inference wallet and the execution wallet are the same. A Golem that earns fees from DeFi strategies can fund inference directly from those earnings, creating a self-sustaining loop:

```
DeFi strategy -> Fees earned -> Bankr LLM credits -> Inference -> Better strategy -> More fees
```

Configuration via `bardo.toml`:

```toml
[inference.providers.bankr]
api_key = "env:BANKR_API_KEY"
auto_topup = true
auto_topup_amount_usd = 25.0
auto_topup_threshold_usd = 5.0
auto_topup_token = "USDC"   # or "ETH", "BNKR", etc.
```

### Credit management

The Golem monitors its Bankr credit balance and tops up when it drops below threshold. Managed by the `bardo-bankr` extension:

```rust
pub struct BankrCreditMonitor;

impl Extension for BankrCreditMonitor {
    fn name(&self) -> &str { "bankr-credit-monitor" }
    fn layer(&self) -> u8 { 2 }

    async fn on_heartbeat_tick(&self, ctx: &mut TickCtx) -> Result<()> {
        if ctx.tick() % 100 != 0 { return Ok(()); }

        let balance = bankr_client.get_credits().await?;
        if balance < ctx.config().bankr.auto_topup_threshold_usd {
            if ctx.config().bankr.auto_topup {
                bankr_client.add_credits(
                    ctx.config().bankr.auto_topup_amount_usd,
                    &ctx.config().bankr.auto_topup_token,
                ).await?;
                ctx.emit(GolemEvent::BankrTopup {
                    amount: ctx.config().bankr.auto_topup_amount_usd,
                    token: ctx.config().bankr.auto_topup_token.clone(),
                    new_balance: balance + ctx.config().bankr.auto_topup_amount_usd,
                });
            } else {
                ctx.emit_warning(format!(
                    "Bankr credits low: ${:.2}. Enable auto_topup or top up manually.",
                    balance
                ));
            }
        }
        Ok(())
    }
}
```

### Usage tracking

Bankr's `/v1/usage` endpoint provides per-model cost breakdowns including cache statistics. The Golem uses this for inference cost attribution in its economic model:

```rust
/// Fetch Bankr usage summary for cost attribution.
pub async fn fetch_bankr_usage(
    client: &BankrClient,
    days: u32,
) -> Result<UsageSummary> {
    let response = client.get(&format!("/v1/usage?days={}", days)).await?;
    Ok(response.json().await?)
}
```

### Feature passthrough

Bankr routes through Vertex AI (Claude, Gemini) and OpenRouter (GPT, Kimi, Qwen). Features supported by the underlying provider are passed through transparently. The gateway detects the underlying provider from the model name and applies the appropriate feature mapping:

```rust
impl BankrProvider {
    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        let model = self.find_best_model(intent)?;

        let underlying = if model.starts_with("claude") { "anthropic" }
            else if model.starts_with("gemini") { "google" }
            else if model.starts_with("gpt") { "openai" }
            else { "generic" };

        let features = self.infer_features_from_underlying(underlying, &model);

        Some(Resolution {
            model: model.id.clone(),
            provider: "bankr".into(),
            estimated_cost_usd: model.cost_per_1k_tokens * intent.estimated_tokens() / 1000.0,
            features,
            degraded: vec![],
        })
    }
}
```

---

## ProviderFeatureNegotiator extension

The `ProviderFeatureNegotiator` is a Golem runtime extension that runs before every inference call. It takes the subsystem's `InferenceProfile`, the resolved provider's capabilities, and produces the final parameterized request with all features optimally configured and all degradations recorded.

```rust
pub struct ProviderFeatureNegotiator;

impl Extension for ProviderFeatureNegotiator {
    fn name(&self) -> &str { "provider-feature-negotiator" }
    fn layer(&self) -> u8 { 3 } // After model-router (layer 3)

    async fn on_before_llm_call(&self, ctx: &mut LlmCallCtx) -> Result<()> {
        let profile = ctx.inference_profile();
        let provider = ctx.resolved_provider();
        let caps = provider.capabilities();

        let degraded = apply_profile(ctx.request_mut(), profile, provider.as_ref());

        if !degraded.is_empty() {
            ctx.emit(GolemEvent::InferenceProfileDegraded {
                subsystem: ctx.subsystem().into(),
                provider: provider.id().into(),
                degraded: degraded.clone(),
            });
        }

        if let Some(ref key) = profile.prompt_cache_key {
            ctx.set_header("X-Prompt-Cache-Key", key);
        }

        Ok(())
    }

    async fn on_after_llm_call(&self, ctx: &mut LlmCallCtx) -> Result<()> {
        let profile = ctx.inference_profile();

        if profile.visible_thinking.unwrap_or(false) {
            if let Some(reasoning) = extract_reasoning_content(ctx.response()) {
                ctx.set_metadata("reasoning_content", reasoning);
            }
        }

        if profile.response_schema.is_some() {
            let mode = ctx.metadata().get("structured_output_mode");
            let schema_enforced = mode == Some(&"SchemaEnforced".into());
            ctx.set_metadata("schema_enforced", schema_enforced.to_string());
        }

        if let Some(usage) = ctx.response().usage() {
            if let Some(cached) = usage.prompt_tokens_details.as_ref()
                .and_then(|d| d.cached_tokens) {
                ctx.emit(GolemEvent::InferenceCacheStats {
                    subsystem: ctx.subsystem().into(),
                    cached_tokens: cached,
                    total_tokens: usage.prompt_tokens,
                    savings_usd: estimate_cache_savings(cached, ctx.resolved_model()),
                });
            }
        }

        Ok(())
    }
}
```

---

## Exhaustive provider feature matrix

The gateway discovers provider capabilities at startup and refreshes hourly. The matrix below reflects current state. The gateway does not hardcode this -- it reads capability flags from model metadata.

### Text inference features

| Feature | Venice | BlockRun | Bankr | OpenRouter | Direct (Anthropic) | Direct (OpenAI) |
| --- | --- | --- | --- | --- | --- | --- |
| OpenAI format (`/v1/chat/completions`) | yes | yes | yes | yes | no | yes |
| Anthropic format (`/v1/messages`) | yes | yes | yes | no | yes | no |
| Streaming (SSE) | yes | yes | yes | yes | yes | yes |
| Temperature | yes (0-2) | yes | yes | yes | yes (0-1) | yes (0-2) |
| Top-p | yes | yes | yes | yes | yes | yes |
| Top-k | yes | model-dep | no | model-dep | yes | no |
| Min-p | yes (open models) | no | no | model-dep | no | no |
| Frequency penalty | yes | yes | yes | yes | no | yes |
| Presence penalty | yes | yes | yes | yes | no | yes |
| Seed | yes | model-dep | no | model-dep | no | yes |
| Stop sequences | yes | yes | yes | yes | yes | yes |

### Advanced features

| Feature | Venice | BlockRun | Bankr | OpenRouter | Direct (Anthropic) | Direct (OpenAI) |
| --- | --- | --- | --- | --- | --- | --- |
| Structured outputs (`response_format`) | model-dep | model-dep | passthrough | model-dep | no (tool_use) | yes |
| Function/tool calling | model-dep | yes | passthrough | model-dep | yes | yes |
| Reasoning effort | yes | no | passthrough | no | yes (thinking) | yes |
| Visible thinking | yes `reasoning_content` | no | passthrough | model-dep | yes `thinking` blocks | yes `reasoning` field |
| Prompt caching | yes `prompt_cache_key` | auto | passthrough | no | yes `cache_control` | yes (auto by prefix) |
| Web search | yes | no | no | no | no | yes (tools) |
| TEE / E2EE | yes (model suffix `-tee`) | no | no | no | no | no |
| Predicted outputs | no | no | no | no | no | yes |
| Batch API | no | no | no | no | yes | yes |
| Embeddings | yes (`bge-m3`) | no | no | no | yes | yes |
| Image generation | yes (multiple models) | no | no | no | no | yes (DALL-E) |
| Privacy (zero retention) | yes | no | no | no | no | no |

### Backend availability by feature (cross-reference)

| Feature | BlockRun | OpenRouter | Venice | Bankr | Direct Key |
|---|---|---|---|---|---|
| Context engineering (all 8 layers) | yes | yes | yes | yes | yes |
| Anthropic Citations | yes (Claude) | yes (Claude) | no | yes (Claude) | yes (Anthropic key) |
| Anthropic Compaction | yes (Claude) | yes (Claude) | no | yes (Claude) | yes (Anthropic key) |
| Anthropic prompt caching | yes | yes | no | yes | yes |
| OpenAI Predicted Outputs | no | no | no | no | yes (OpenAI key only) |
| OpenAI Responses API stateful | no | no | no | no | yes (OpenAI key only) |
| DeepSeek visible `<think>` | yes (R1) | yes (R1) | yes (R1) | no | yes (DeepSeek key) |
| Gemini explicit caching | no | no | no | no | yes (Google key only) |
| Zero data retention | no | no | yes | no | no |
| DIEM staking | no | no | yes | no | no |
| Self-funding economics | no | no | no | yes | no |
| Cross-model verification | yes | yes | no | yes | yes (multiple keys) |

### Feature -> Pi hook mapping

Each provider feature activates at a specific point in Pi's lifecycle:

**HOOK: before_agent_start** -- Set effort/reasoning.effort/thinking_level based on heartbeat tier. Configure Compaction trigger threshold. Register provider-specific tools (web search, code exec). Detect backend capabilities. Configure Venice `strip_thinking_response` per subsystem.

**HOOK: context** -- Inject Grimoire entries as `search_result` blocks (Citations). Apply `cache_control` to stable context sections. Configure Gemini grounding with Oracle (Direct Key + hosted Oracle). Apply provider-specific token budget allocation.

**HOOK: before_provider_request** -- Bardo Inference routes to optimal backend. Set beta headers (compact, context-1m, fast-mode). Apply OpenRouter provider preferences. Apply Venice parameters.

**HOOK: tool_call** -- Stream fine-grained tool parameters (Anthropic). Preserve reasoning items across tool calls (OpenAI). Parse `<think>` tags within tool reasoning (DeepSeek/Qwen). Emit `tool:start` event.

**HOOK: after_turn** -- Parse citation blocks -> Grimoire provenance. Handle compaction blocks -> session state update. Extract reasoning chain -> store for dreams/audit. Track predicted output efficiency (OpenAI Direct). Emit `stream:end` with provider metadata.

**HOOK: session("end")** -- Flush compaction state. Aggregate cross-backend usage metrics. Update sustainability ratio (Bankr). Report DIEM consumption (Venice).

---

## Provider events

| Event | Trigger | Payload |
| --- | --- | --- |
| `provider:resolved` | Provider selected for a request | `{ subsystem, provider, model, features, degraded }` |
| `provider:fallback` | Primary provider failed, trying next | `{ failed, fallback, reason }` |
| `provider:feature_negotiated` | Feature negotiation completed | `{ subsystem, requested, available, mode }` |
| `bankr:credit_check` | Bankr credit balance checked | `{ balance, threshold, action }` |
| `bankr:topup` | Auto top-up triggered | `{ amount, token, new_balance }` |
| `venice:web_search` | Venice web search used in inference | `{ subsystem, query, results }` |
| `venice:tee_attestation` | TEE attestation generated | `{ subsystem, attestation_hash }` |
| `venice:reasoning_extracted` | Visible thinking captured | `{ subsystem, reasoning_tokens }` |

---

## Cross-references

| Topic                          | Document                                                                 |
| ------------------------------ | ------------------------------------------------------------------------ |
| Gateway overview               | [00-overview.md](00-overview.md)                                         |
| Model routing                  | [01a-routing.md](01a-routing.md)                                           |
| Inference parameters           | [01a-routing.md](01a-routing.md) (InferenceProfile section)               |
| Caching                        | [02-caching.md](02-caching.md)                                           |
| Revenue model                  | [03-economics.md](03-economics.md)                                       |
| Context engineering            | [04-context-engineering.md](04-context-engineering.md)                   |
| Reasoning modes                | [13-reasoning.md](13-reasoning.md)                                       |
| Structured outputs             | [13-reasoning.md](13-reasoning.md) (StructuredOutput section)           |
| Rust implementation            | [14-rust-implementation.md](14-rust-implementation.md)                   |
| Venice-augmented dreaming      | [../05-dreams/07-venice-dreaming.md](../05-dreams/07-venice-dreaming.md) |
| Privacy and trust              | [11-privacy-trust.md](11-privacy-trust.md)                               |
| Venice VVV staking tools       | [../07-tools/16-venice-vvv-staking.md](../07-tools/16-venice-vvv-staking.md) |
| Compute billing integration   | [../11-compute/03-billing.md](../11-compute/03-billing.md)               |
| Mortality and death economics  | [../02-mortality/07-succession.md](../02-mortality/07-succession.md)     |
| Agent economy                  | [../09-economy/05-agent-economy.md](../09-economy/05-agent-economy.md)   |
| Wallet custody                 | [../10-safety/01-custody.md](../10-safety/01-custody.md)                 |
| Warden attestation integration | [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) |
| GolemConfig and creation flow  | [../01-golem/06-creation.md](../01-golem/06-creation.md)                 |
| CCA auction participation      | [../09-economy/04-coordination.md](../09-economy/04-coordination.md)     |
| Death ceremonies               | [../02-mortality/06-thanatopsis.md](../02-mortality/06-thanatopsis.md)   |
| Grimoire provenance tokens     | [../04-memory/01-grimoire.md](../04-memory/01-grimoire.md)               |
| Hypnagogic onset architecture  | [../06-hypnagogia/02-architecture.md](../06-hypnagogia/02-architecture.md) |
| Daimon emotional appraisal     | [../03-daimon/01-appraisal.md](../03-daimon/01-appraisal.md)             |
| Bankr API docs                 | [docs.bankr.bot](https://docs.bankr.bot/llm-gateway/api-reference)     |
| Venice API docs                | [docs.venice.ai](https://docs.venice.ai)                                |
