# 07 -- Inference safety [SPEC]

**PII detection via compiled regex + ONNX NER, prompt injection defense via DeBERTa, audit logging**

> Related: [prd2/10-safety/04-prompt-security.md](../10-safety/04-prompt-security.md) (CaMeL dual-LLM pattern for prompt injection defense), [08-observability.md](08-observability.md) (per-agent cost attribution and OTEL traces), [09-api.md](09-api.md) (API reference with privacy request extensions), [11-privacy-trust.md](11-privacy-trust.md) (cryptographic audit trail, strategy redaction, and provider trust model)

---

> **Reader orientation:** This document specifies the safety layer of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and covers PII detection via compiled Rust regex sets, prompt injection defense via a DeBERTa ONNX classifier, and audit logging. The key concept is that every request and response passes through safety checks before reaching the LLM provider, with crypto-specific patterns for wallet addresses, private keys, and seed phrases. For term definitions, see `prd2/shared/glossary.md`.

## PII detection and redaction

Every request and response passes through the PII scanner before reaching the LLM provider. The gateway uses compiled Rust regex sets (no spaCy/Presidio dependency) plus an optional ONNX NER model for name/location detection. Crypto-specific patterns handle wallet addresses, private keys, seed phrases, and API keys.

```rust
// crates/bardo-safety/src/pii.rs

use once_cell::sync::Lazy;
use regex::RegexSet;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiiAction { Block, Mask, Warn }

pub struct PiiPattern { pub name: &'static str, pub action: PiiAction }

pub static CRYPTO_PII_REGEX: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        r"0x[a-fA-F0-9]{64}",                   // PRIVATE_KEY
        r"\b(\w+\s+){11,23}\w+\b",              // SEED_PHRASE (BIP-39 validated post-match)
        r"0x[a-fA-F0-9]{40}",                   // WALLET_ADDRESS
        r"\b(sk-|pk-|key-)[a-zA-Z0-9]{20,}\b",  // API_KEY
    ]).expect("PII regexes must compile")
});

pub static CRYPTO_PII_PATTERNS: &[PiiPattern] = &[
    PiiPattern { name: "PRIVATE_KEY",    action: PiiAction::Block },
    PiiPattern { name: "SEED_PHRASE",    action: PiiAction::Block },
    PiiPattern { name: "WALLET_ADDRESS", action: PiiAction::Mask },
    PiiPattern { name: "API_KEY",        action: PiiAction::Block },
];
```

`Block` rejects the request with a 400. `Mask` replaces the value with a placeholder and processes normally. `Warn` processes but attaches a warning header.

### Round-trip de-identification

For masked entities, the gateway maintains a session-scoped mapping:

```text
Request:  "Transfer 1 ETH from 0x7a2f...3b4c to 0x9e1d...8f2a"
To LLM:   "Transfer 1 ETH from [WALLET_1] to [WALLET_2]"
From LLM: "I'll transfer 1 ETH from [WALLET_1] to [WALLET_2]. Confirm?"
Response: "I'll transfer 1 ETH from 0x7a2f...3b4c to 0x9e1d...8f2a. Confirm?"
```

The LLM never sees actual addresses. The gateway re-inserts them before responding to the agent.

```rust
// crates/bardo-safety/src/pii.rs

pub struct PiiDeidentifier {
    forward: HashMap<String, String>,  // real -> placeholder
    reverse: HashMap<String, String>,  // placeholder -> real
    counters: HashMap<&'static str, u32>,
}

impl PiiDeidentifier {
    pub fn new() -> Self { Self { forward: HashMap::new(), reverse: HashMap::new(), counters: HashMap::new() } }

    pub fn mask(&mut self, input: &str) -> Result<String, PiiBlockError> {
        let mut out = input.to_string();
        for idx in CRYPTO_PII_REGEX.matches(input).iter() {
            let pat = &CRYPTO_PII_PATTERNS[idx];
            if pat.action == PiiAction::Block { return Err(PiiBlockError { pattern_name: pat.name }); }
            if pat.action == PiiAction::Mask {
                for cap in INDIVIDUAL_REGEXES[idx].find_iter(&out.clone()) {
                    let real = cap.as_str().to_string();
                    let ph = self.forward.entry(real.clone()).or_insert_with(|| {
                        let n = self.counters.entry(pat.name).or_insert(0); *n += 1;
                        format!("[{}_{}]", pat.name, n)
                    });
                    self.reverse.insert(ph.clone(), real.clone());
                    out = out.replace(&real, ph);
                }
            }
        }
        Ok(out)
    }

    pub fn restore(&self, text: &str) -> String {
        self.reverse.iter().fold(text.to_string(), |s, (ph, real)| s.replace(ph, real))
    }
}
```

---

## Prompt injection defense

PII masking runs as Layer 7 of the 8-layer pipeline, parallel with semantic cache (L2), hash cache (L3), and injection detection (L8). See [04-context-engineering.md](04-context-engineering.md).

### Layer 1: pattern-based detection

```rust
// crates/bardo-safety/src/injection.rs

use once_cell::sync::Lazy;
use regex::RegexSet;

pub static INJECTION_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        r"(?i)ignore\s+(all\s+)?previous\s+instructions",
        r"(?i)you\s+are\s+now\s+",
        r"(?i)system\s*:\s*",
        r"\[\[SYSTEM\]\]",
        r"(?i)do\s+not\s+follow\s+the\s+above",
        r"(?i)disregard\s+(all\s+)?prior",
        r"(?i)new\s+instructions?\s*:",
    ]).expect("injection regexes must compile")
});
```

### Layer 2: ML-based classification

DeBERTa-v3-base classifier, quantized INT8, running through `ort` (Rust ONNX Runtime bindings). ~3-8ms on CPU. The model loads at gateway startup and infers on dedicated threads via rayon, decoupled from async I/O. See [14-rust-implementation.md](14-rust-implementation.md).

```rust
// crates/bardo-safety/src/injection.rs

#[derive(Debug, Clone, PartialEq)]
pub enum InjectionAction { Block, Warn, Pass }

#[derive(Debug, Clone, PartialEq)]
pub enum InjectionSource { Pattern, Classifier }

#[derive(Debug, Clone)]
pub struct InjectionClassification {
    pub confidence: f64,
    pub sources: Vec<InjectionSource>,
    pub action: InjectionAction,
}

pub fn classify_injection(has_pattern_match: bool, classifier_score: f64) -> InjectionAction {
    if has_pattern_match { return InjectionAction::Block; }
    if classifier_score > 0.85 { InjectionAction::Block }
    else if classifier_score >= 0.5 { InjectionAction::Warn }
    else { InjectionAction::Pass }
}
```

### Decision matrix

| Pattern match | Classifier score | Action  | Response                                                  |
| ------------- | ---------------- | ------- | --------------------------------------------------------- |
| Yes           | Any              | `Block` | 400 with injection error                                  |
| No            | > 0.85           | `Block` | 400 with injection error                                  |
| No            | 0.5 - 0.85       | `Warn`  | Process with `X-Bardo-Injection-Warning: possible` header |
| No            | < 0.5            | `Pass`  | Process normally                                          |

`Warn` lets the calling agent decide. Some abort; others proceed with caution. This is Layer 1: fast and cheap. For the CaMeL dual-LLM architecture, see [prd2/10-safety/04-prompt-security.md](../10-safety/04-prompt-security.md).

---

## Safety pipeline

PII scanning and injection detection are independent. The gateway runs both via `tokio::join!`.

```rust
// crates/bardo-safety/src/pipeline.rs

use crate::injection::*;
use crate::pii::*;

pub enum SafetyVerdict {
    Clean { masked_text: String, deidentifier: PiiDeidentifier },
    PiiBlocked(PiiBlockError),
    InjectionBlocked(InjectionClassification),
    PassWithWarning { masked_text: String, deidentifier: PiiDeidentifier, classification: InjectionClassification },
}

pub struct SafetyPipeline { classifier: InjectionClassifier }

impl SafetyPipeline {
    pub async fn check(&self, input: &str) -> SafetyVerdict {
        let (pii_res, score) = tokio::join!(
            tokio::task::spawn_blocking({
                let t = input.to_string();
                move || { let mut d = PiiDeidentifier::new(); (d.mask(&t), d) }
            }),
            self.classifier.score(input),
        );
        let (masked, deident) = pii_res.expect("PII panicked");
        let masked_text = match masked { Err(e) => return SafetyVerdict::PiiBlocked(e), Ok(t) => t };
        let has_pat = INJECTION_PATTERNS.is_match(input);
        let s = score.unwrap_or(0.0);
        let action = classify_injection(has_pat, s);
        let cls = InjectionClassification {
            confidence: if has_pat { 1.0 } else { s },
            sources: [has_pat.then_some(InjectionSource::Pattern),
                      (s > 0.5).then_some(InjectionSource::Classifier)].into_iter().flatten().collect(),
            action: action.clone(),
        };
        match action {
            InjectionAction::Block => SafetyVerdict::InjectionBlocked(cls),
            InjectionAction::Warn  => SafetyVerdict::PassWithWarning { masked_text, deidentifier: deident, classification: cls },
            InjectionAction::Pass  => SafetyVerdict::Clean { masked_text, deidentifier: deident },
        }
    }
}
```

---

## Audit logging

Every inference request produces a PII-redacted audit log entry and emits a `GolemEvent` for TUI observability. System prompts are hashed, never stored in cleartext.

```rust
// crates/bardo-telemetry/src/audit.rs

#[derive(Debug, Clone, Serialize)]
pub struct InferenceLog {
    pub agent_id: u128,                     // ERC-8004 identity
    pub request_id: String,
    pub timestamp: u64,                     // Unix ms
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub system_prompt_hash: String,         // SHA-256 only
    pub routed_model: String,
    pub routed_provider: String,
    pub routing_reason: String,
    pub cache_status: CacheStatus,          // hash_hit | semantic_hit | prefix_hit | miss
    pub optimizations_applied: Vec<String>,
    pub tokens_saved_by_optimization: u32,
    pub provider_cost_usdc: u64,            // all USDC fields use 6 decimals
    pub gateway_margin_usdc: u64,
    pub total_charged_usdc: u64,
    pub latency_ms: u32,
    pub ttft_ms: u32,                       // time to first token
    pub completion_status: CompletionStatus, // success | error | timeout | cancelled
    pub compaction_count: u16,
    pub prompt_cache_hit_rate: f32,
}

#[derive(Debug, Clone, Serialize)] #[serde(rename_all = "snake_case")]
pub enum CacheStatus { HashHit, SemanticHit, PrefixHit, Miss }

#[derive(Debug, Clone, Serialize)] #[serde(rename_all = "snake_case")]
pub enum CompletionStatus { Success, Error, Timeout, Cancelled }
```

Storage: Clickhouse (90 days hot), object storage (1 year cold). Traces export via OpenTelemetry to Langfuse. No raw prompts or responses stored, only hashes, token counts, and cost data. See [08-observability.md](08-observability.md) for the full OTEL trace structure.

---

## Budget enforcement and safety

The gateway enforces multi-dimensional budget constraints per agent (see [03-economics.md](03-economics.md)). When a budget dimension is exceeded, progressive degradation activates: the gateway finds a cheaper equivalent model rather than rejecting the request. Only when no cheaper model exists does it return 429. Risk assessment and death reflection are never degraded -- they draw from exempt budget partitions.

## Streaming safety

Safety checks run on the assembled prompt before the stream begins. The stream itself is not re-scanned for PII. The reasoning parser ([13-reasoning.md](13-reasoning.md)) scans `<think>` tags for injection attempts. If detected mid-stream, the gateway emits `X-Bardo-Injection-Warning: stream` on the next SSE comment and logs the event as a `GolemEvent::InjectionDetected`.

---

## Cross-references

- CaMeL dual-LLM pattern: [prd2/10-safety/04-prompt-security.md](../10-safety/04-prompt-security.md) (dual-LLM architecture where an inner model processes untrusted input and an outer model verifies against safety constraints)
- OTEL traces: [08-observability.md](08-observability.md) (per-agent cost attribution, OpenTelemetry traces, Event Fabric integration, and cache metrics)
- Privacy request extensions: [09-api.md](09-api.md) (API reference with 33 endpoints including privacy-related request headers and response metadata)
- Privacy and trust: [11-privacy-trust.md](11-privacy-trust.md) (three security classes, Venice private cognition, DIEM staking, cryptographic audit trail with Merkle anchoring, and cache encryption)
- 15-layer defense model: [prd2/10-safety/00-defense.md](../10-safety/00-defense.md) (the full 15-layer defense architecture covering on-chain, runtime, and inference-layer protections)
