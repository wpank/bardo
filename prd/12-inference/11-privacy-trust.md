# 11 -- Privacy and trust [SPEC]

**Three inference security classes, Venice private cognition, DIEM staking, cryptographic audit trail, strategy-aware redaction, cache encryption, gateway signing, secret-keeping architecture**

> Related: [07-safety.md](07-safety.md) (PII detection, prompt injection defense, and audit logging), [09-api.md](09-api.md) (API reference with privacy request extensions), [02-caching.md](02-caching.md) (three-layer cache stack with encryption at rest), [08-observability.md](08-observability.md) (per-agent cost attribution and OTEL traces), [12-providers.md](12-providers.md) (Venice private cognition plane with TEE attestation), [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) (optional Warden time-delay proxy for high-value transactions)

---

> **Reader orientation:** This document specifies the privacy and trust architecture of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and covers three security classes for inference requests, Venice private cognition with TEE attestation, DIEM staking for zero-cost private inference, a cryptographic audit trail with hash chains and Merkle tree anchoring, strategy-aware redaction, and cache encryption. The key concept is that DeFi agents have unique confidentiality requirements because their inference calls expose trading intent, and strategy leakage is economically equivalent to front-running. For term definitions, see `prd2/shared/glossary.md`.

## 1. Three inference security classes

Every inference request falls into one of three security classes. The class determines provider routing, redaction level, and audit depth.

| Class | Data retention | Use cases | Providers |
|---|---|---|---|
| **Standard** | Provider retains prompts for training/abuse monitoring | Routine analysis, market commentary, public strategy reasoning | BlockRun, OpenRouter, Bankr, Direct Key |
| **Confidential** | Provider retains for billing/audit but not training | Portfolio-specific analysis, risk assessment with position data | OpenRouter (select no-train models), BlockRun (x402, no account) |
| **Private** | Zero retention. Provider cannot reconstruct what was asked. | Treasury management, deal negotiation, governance voting, MEV-sensitive execution, death reflection | Venice only |

The Private class enables agent behaviors that are structurally impossible with standard inference:

**Confidential treasury reasoning.** A Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) managing a $500K vault reasons about rebalancing strategy. With standard inference, the provider observes portfolio composition, risk parameters, and timing intent. An adversary with provider access (subpoena, breach, insider) reconstructs the position and front-runs the rebalance. With Venice, the reasoning vanishes after response delivery.

**Private deal negotiation.** Two Golems negotiate a cross-vault strategy allocation. Each Golem reasons privately about its negotiation strategy (reserve price, walk-away conditions) using Venice. Only structured offers/counter-offers transmit between agents. The reasoning that produced the offer is structurally unrecoverable.

**MEV-resistant execution planning.** When a Golem plans a large swap, execution timing is MEV-sensitive. The planning phase -- which route, what time, what slippage tolerance -- runs on Venice. A sandwich bot monitoring inference providers sees nothing.

### Security-class classifier

The `bardo-context` Pi extension tags requests based on content sensitivity. Classification is deterministic (no LLM call):

```rust
// crates/bardo-safety/src/security_class.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel { Standard, Confidential, Private }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityTrigger {
    PortfolioComposition,    // Request contains specific asset amounts
    RebalanceTiming,         // Request discusses when to execute
    DealNegotiation,         // Inter-agent commercial discussion
    GovernanceDeliberation,  // Proposal analysis with position exposure
    MevSensitive,            // Execution timing that could be front-run
    CounterpartyAnalysis,    // Evaluating another agent's behavior
    DeathReflection,         // Terminal phase -- most honest, most sensitive
    OwnerPii,                // Owner-identifying information in context
}

pub fn classify_security_class(
    context: &ContextBundle,
    phase: BehavioralPhase,
) -> InferenceSecurityClass {
    let mut triggers = Vec::new();

    // Portfolio composition: positions above $1,000
    if context.defi_snapshot.positions.iter().any(|p| p.value_usd > 1000.0) {
        triggers.push(SecurityTrigger::PortfolioComposition);
    }

    // Rebalance timing: pending swap or rebalance actions
    if context.tool_state.pending_actions.iter().any(|a| {
        matches!(a.action_type, ActionType::Swap | ActionType::Rebalance)
    }) {
        triggers.push(SecurityTrigger::RebalanceTiming);
    }

    // Death reflection: always private
    if phase == BehavioralPhase::Terminal {
        triggers.push(SecurityTrigger::DeathReflection);
    }

    // MEV-sensitive: any pending execution above $500
    if context.tool_state.pending_actions.iter().any(|a| {
        a.estimated_value_usd > 500.0
    }) {
        triggers.push(SecurityTrigger::MevSensitive);
    }

    if !triggers.is_empty() {
        return InferenceSecurityClass {
            class: SecurityLevel::Private,
            reason: format!(
                "Contains {}",
                triggers.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join(", ")
            ),
            triggers,
        };
    }

    InferenceSecurityClass {
        class: SecurityLevel::Standard,
        reason: "No sensitive content detected".to_string(),
        triggers: vec![],
    }
}
```

When `class` resolves to `Private`, the router hard-filters to Venice. If Venice is not configured, the gateway returns HTTP 503 with a clear error rather than falling back to a retaining provider.

---

## 2. Venice: zero-data-retention private cognition

Venice provides the only structural privacy guarantee in the system. Not a policy promise. Not a terms-of-service checkbox. The inference logs do not exist.

### Why Venice is not "just another provider"

Adding Venice is not about breadth -- Bardo already has 400+ models via OpenRouter. Venice adds a fundamentally new inference plane: the Private class. Every other provider retains some data (prompts, billing records, safety logs). Venice retains nothing.

### Venice provider configuration

```rust
// crates/bardo-providers/src/venice.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceProviderConfig {
    pub base_url: String,  // "https://api.venice.ai/api/v1"
    pub api_key: String,
    pub models: VeniceModelMapping,
    pub venice_parameters: Option<VeniceParameters>,
    pub daily_cap_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceModelMapping {
    /// Fast, cheap, routine private analysis
    pub t1: String,           // "llama-3.3-70b"
    /// Deep reasoning with visible <think> tags
    pub t2_reasoning: String, // "deepseek-ai-DeepSeek-R1"
    /// Frontier, 128K context, tool use
    pub t2_general: String,   // "zai-org-glm-4.7"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceParameters {
    pub enable_web_search: Option<WebSearchMode>,
    /// Controls whether R1's <think> tags appear in responses.
    /// False for dreams and death reflection (preserve full chain).
    /// True for routine operations (smaller, faster).
    pub strip_thinking_response: Option<bool>,
}
```

### Private subsystem routing

| Subsystem | Always private? | Reason |
|---|---|---|
| Dream cycles | Yes | Subconscious is unobservable. Counterfactual scenarios, threat simulations. |
| Death reflection | Yes | Most honest reasoning. Unrealized hypotheses, strategy fragments. |
| Daimon (emotional appraisal) | Yes | Emotional state = known vulnerability. Anxiety signals exploitable. |
| Deal negotiation | Yes | Reserve prices, walk-away conditions. Material non-public information. |
| Governance deliberation | Yes | Vote intent + position exposure = insider trading intelligence. |
| Execution planning (>$500) | Yes | Timing and routing = front-running opportunity. |
| Routine heartbeat | No | Low-value, no position data. |
| Operator conversation | Depends | Private if portfolio data in context. |

### Venice + DeepSeek R1: private visible reasoning

Venice is the only backend where DeepSeek R1's visible `<think>` tags combine with zero data retention. This enables private visible reasoning -- the Golem sees and stores the full reasoning chain locally while the provider forgets it. No other backend combination provides this. Death reflection uses this path: R1's raw reasoning process (uncertainty, contradictions, abandoned hypotheses) is captured in the Grimoire. Venice forgets the conversation.

### Venice vision: private chart analysis

Venice serves Qwen 2.5 VL 72B for private visual analysis. Use cases:
- Order book heatmap analysis for hidden liquidity walls
- Governance vote distribution charts for whale detection
- TVL charts for regime shift identification before numerical data reflects them

The visual analysis runs on Venice. The provider never sees what chart the Golem analyzed or what it concluded.

---

## 3. DIEM staking: zero-cost private inference

Venice's tokenomics create a unique possibility: a Golem that funds its own private inference through token staking, achieving zero-marginal-cost reasoning.

### Mechanism

1. Owner stakes VVV (Venice's native token on Base)
2. Staked VVV earns pro-rata daily DIEM allocation
3. Each DIEM = $1/day of Venice API credit, perpetually
4. Golem consumes DIEM for private inference -- no per-request payment
5. Excess DIEM can be traded or allocated to successor Golems

```rust
// crates/bardo-providers/src/venice_staking.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceStakingConfig {
    pub vvv_token: Address,            // VVV contract on Base
    pub staked_vvv: U256,              // Amount staked by owner
    pub daily_diem_allocation: f64,    // Computed from pro-rata stake share
    pub diem_consumed_today: f64,
    pub dream_reserve_fraction: f64,   // Default: 0.15
}
```

### Mortality integration: DIEM as life extension

In the standard mortality model, inference costs drain the LLM credit partition (60% of total budget). DIEM staking decouples inference cost from mortality pressure:

| Inference source | Cost model | Mortality impact |
|---|---|---|
| BlockRun (x402) | Per-request USDC micropayment | Drains LLM partition, shortens lifespan |
| Venice (DIEM) | Zero marginal cost from staked VVV | No drain on LLM partition |

A Golem routing 50% of inference to Venice extends its projected lifespan by ~30% (saving $0.06-0.10/day on a $0.20/day budget). Over a 30-day lifespan, this is 9 additional days -- purchased through the owner's VVV stake, not through more USDC.

```rust
pub fn compute_venice_lifespan_extension(
    daily_inference_cost_usd: f64,
    venice_fraction: f64,
    current_burn_rate_usd: f64,
    remaining_credits_usd: f64,
) -> LifespanExtension {
    let daily_savings = daily_inference_cost_usd * venice_fraction;
    let new_burn_rate = current_burn_rate_usd - daily_savings;
    let original_days_remaining = remaining_credits_usd / current_burn_rate_usd;
    let new_days_remaining = remaining_credits_usd / new_burn_rate;
    LifespanExtension {
        extension_days: new_days_remaining - original_days_remaining,
        extension_hours: (new_days_remaining - original_days_remaining) * 24.0,
    }
}
```

### DIEM allocation strategy

```
Daily DIEM Budget: $X (from VVV stake)
+-- Waking inference (private):  60%  -- Portfolio analysis, deal negotiation
+-- Dream cycles (private):      15%  -- Counterfactual reasoning, threat simulation
+-- Sleepwalker artifacts:       15%  -- Observatory research (if phenotype=sleepwalker)
+-- Reserve (rollover):          10%  -- Unused DIEM for volatile days
```

---

## 4. Moat: agents that keep secrets

The privacy argument for Bardo is not "we collect less data." It is an architecture argument.

Most agent frameworks treat privacy as a policy problem. They write policies, implement access controls, and hope the controls hold. But the architecture itself generates leakage -- every MCP call, every API request, every conversation log creates a new exfiltration surface.

Bardo treats privacy as an architecture problem. Sensitive data has minimal exposure surface by construction:

**Keys never leave TEE hardware.** The Golem's wallet key is isolated. There is no API key to exfiltrate because x402 payment is cryptographic, not credential-based.

**Context is assembled by the Governor, not appended from raw history.** The `ContextBundle` is structured, minimal, and auditable. The `bardo-result-filter` extension sanitizes tool results before they enter the message array.

**Inference routes through a proxy that strips non-essential context.** Semantic caching means repeated queries never reach the provider. The 8-layer pipeline applies before any provider sees the request.

**Payment uses x402 (no account relationship).** Per-request USDC settlement via `transferWithAuthorization`. The provider sees a request and a payment. No name, no email, no billing address, no usage history.

**Knowledge decays by default.** Demurrage on Grimoire entries. Stale personal data doesn't persist indefinitely.

**Tools are compiled Rust/TypeScript, not remote MCP servers.** No external data transmission during tool execution.

### Five leakage vectors and how Bardo addresses them

| Vector | Typical agent | Bardo |
|---|---|---|
| API key exfiltration | Plaintext `.env` files, environment variables | x402 wallet-native payment, no API keys to steal |
| Context window leakage | All tools see full context | ContextBundle with category isolation, result filtering |
| On-chain behavioral fingerprinting | No MEV protection | Warden time-delay + Flashbots Protect + slippage bounds |
| Inference provider surveillance | Full context to provider every call | Context-pruned proxy + Venice zero-retention + x402 (no account) |
| Persistent memory poisoning | No decay, no validation | Confidence scoring + demurrage + causal rollback + Curator pruning |

The result: an attacker who compromises the Golem's VM cannot extract signing keys (TEE), cannot access full conversation history (Governor prunes and compresses), cannot establish persistent control (mortality terminates the Golem; succession starts fresh), and cannot pivot to external services (no OAuth tokens, no API keys, no MCP connections).

---

## 5. Cryptographic audit trail

Audit logging in [07-safety.md](07-safety.md) stores `InferenceLog` entries in Clickhouse -- sufficient for analytics but not tamper-evident. A compromised gateway could alter historical records. The cryptographic audit trail extends `InferenceLog` with hash-chain integrity, gateway signatures, and on-chain root anchoring.

### Extended InferenceLog

```rust
// crates/bardo-telemetry/src/audit.rs

/// Extends `InferenceLog` (07-safety.md) with hash-chain tamper evidence.
#[derive(Debug, Clone, Serialize)]
pub struct InferenceLogSigned {
    #[serde(flatten)]
    pub base: InferenceLog,
    /// SHA-256 of the previous event in this agent's chain. None for first event.
    pub prev_hash: Option<String>,
    /// SHA-256(prev_hash || canonical_json(event_fields)). Chain link.
    pub event_hash: String,
    /// Ed25519 signature of event_hash by the gateway's signing key.
    pub gateway_signature: String,
    /// SHA-256 of the full request body (after PII masking).
    pub input_hash: String,
    /// SHA-256 of the full response body (before de-identification restoration).
    pub output_hash: String,
}
```

### Hash chain construction

Each agent maintains an independent hash chain. Events are strictly ordered by `timestamp` within an agent's chain.

```text
Event N:   event_hash = SHA-256(prev_hash_N-1 || canonical_json(event_fields_N))
Event N+1: event_hash = SHA-256(event_hash_N  || canonical_json(event_fields_N+1))
```

`canonical_json` uses RFC 8785 (deterministic key ordering). `event_fields` includes all `InferenceLog` fields plus `input_hash` and `output_hash`, but not `prev_hash`, `event_hash`, or `gateway_signature` (those are derived). The gateway signs every `event_hash` with its Ed25519 key, binding its identity to the event.

```rust
// crates/bardo-telemetry/src/audit.rs
use sha2::{Sha256, Digest};

pub fn compute_event_hash(
    prev_hash: Option<&str>,
    event_fields: &InferenceLog,
    input_hash: &str,
    output_hash: &str,
) -> String {
    let mut hasher = Sha256::new();
    if let Some(ph) = prev_hash {
        hasher.update(ph.as_bytes());
    }
    let canonical = serde_jcs::to_string(event_fields)
        .expect("InferenceLog must be serializable");
    hasher.update(canonical.as_bytes());
    hasher.update(input_hash.as_bytes());
    hasher.update(output_hash.as_bytes());
    hex::encode(hasher.finalize())
}
```

### Merkle tree aggregation

Events aggregate into Merkle trees (batch size 256, configurable via `BARDO_AUDIT_BATCH_SIZE`). Binary tree over `event_hash` values; root published to Base via the Facilitator contract.

```solidity
// Extension to the existing x402 Facilitator contract on Base
function publishAuditRoot(
    bytes32 merkleRoot,
    uint256 batchStartIndex,
    uint256 batchEndIndex
) external;
```

Piggybacks on x402 settlement infrastructure. ~$0.001 per anchor at current Base gas prices. Anchoring frequency follows event volume, not a fixed schedule.

### Verification API

```rust
// GET /v1/audit/events/{eventId}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventResponse {
    pub event: InferenceLogSigned,
    pub merkle_proof: Vec<String>,
    pub leaf_index: u64,
    pub anchor_tx_hash: Option<String>,
}

// GET /v1/audit/verify?agentId={agentId}&from={startIndex}&to={endIndex}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditVerifyResponse {
    pub agent_id: u128,
    pub range: AuditRange,
    pub chain_intact: bool,
    pub valid_signatures: u64,
    pub anchored_events: u64,
    pub violations: Vec<AuditViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditViolationType { HashMismatch, SignatureInvalid, Gap }
```

Agents can independently verify any event with O(log n) proof path plus on-chain root lookup. No trust in the gateway is required.

---

## 6. Strategy-aware data minimization

The de-identification pattern in [07-safety.md](07-safety.md) masks wallet addresses with `[WALLET_N]` placeholders. This extends the same approach to protect agent trading strategies from leaking to LLM providers.

### Strategy signal taxonomy

| Signal type     | Pattern                                       | Placeholder                    | Example                                    |
| --------------- | --------------------------------------------- | ------------------------------ | ------------------------------------------ |
| Token names     | Known token symbols and names                 | `[TOKEN_A]`, `[TOKEN_B]`       | `ETH` -> `[TOKEN_A]`                       |
| Dollar amounts  | `$N`, `N USDC`, numeric with currency context | `[AMOUNT_1]`, `[AMOUNT_2]`     | `$50,000` -> `[AMOUNT_1]`                  |
| Position sizes  | Numeric values in position/size context       | `[SIZE_1]`, `[SIZE_2]`         | `100 ETH` -> `[SIZE_1] [TOKEN_A]`          |
| Trade direction | buy, sell, long, short                        | `[DIRECTION]`                  | `buy` -> `[DIRECTION]`                     |
| Percentages     | `N%` in strategy context                      | `[PCT_1]`, `[PCT_2]`           | `rebalance at 5%` -> `rebalance at [PCT_1]`|
| LP ranges       | Tick ranges, price bounds                     | `[RANGE_LOW]`, `[RANGE_HIGH]`  | `1800-2200` -> `[RANGE_LOW]-[RANGE_HIGH]`  |
| Time schedules  | Cron expressions, time intervals, deadlines   | `[SCHEDULE_1]`                 | `every 4 hours` -> `[SCHEDULE_1]`          |
| Protocol names  | Known DeFi protocol names                     | `[PROTOCOL_A]`, `[PROTOCOL_B]` | `Morpho` -> `[PROTOCOL_A]`                 |

### Redaction levels

| Level        | Redacted signals                                                           | Use case                                   |
| ------------ | -------------------------------------------------------------------------- | ------------------------------------------ |
| `none`       | Nothing (wallet masking from 07-safety.md still applies)                   | Read-only queries, general research        |
| `standard`   | Token names, dollar amounts, position sizes, trade direction               | Strategy reasoning, position analysis      |
| `aggressive` | All of `standard` + percentages, LP ranges, time schedules, protocol names | Admin operations, sensitive strategy logic |

### Implementation

```rust
// crates/bardo-safety/src/redaction.rs
use regex::RegexSet;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionLevel { None, Standard, Aggressive }

/// Strategy-aware signal redaction. Session-scoped placeholder map assigns
/// incrementing placeholders ("[TOKEN_A]", "[AMOUNT_1]", etc.) and restores
/// originals in responses before delivery.
#[derive(Debug, Clone)]
pub struct StrategyRedactionConfig {
    pub level: RedactionLevel,
    pub placeholder_map: HashMap<String, String>,
    patterns: RegexSet,
}
```

Strategy redaction is best-effort, not a formal privacy guarantee. LLMs lose domain knowledge when seeing `[TOKEN_A]` instead of `ETH`, so redacted prompts may produce lower-quality responses. Measurement is needed. Sophisticated analysis of placeholder patterns could still reveal strategy characteristics; redaction protects against casual observation.

### Default by Warden risk tier (optional, deferred)

| Risk tier | Default redaction |
| --------- | ----------------- |
| Routine   | `none`            |
| Standard  | `none`            |
| Elevated  | `standard`        |
| High      | `aggressive`      |
| Critical  | `aggressive`      |

---

## 7. Per-agent cryptographic cache isolation

The semantic cache ([02-caching.md](02-caching.md)) stores responses in an in-process HNSW index. In `per-agent` isolation mode, namespaces prevent cross-agent reads. Encryption at rest with per-agent keys adds a second layer.

### Key derivation

```rust
// crates/bardo-cache/src/encryption.rs
use hkdf::Hkdf;
use sha2::Sha256;

/// HKDF-SHA-256(master, salt="bardo-cache-v1", info="agent:{id}", 32 bytes).
pub fn derive_agent_cache_key(master: &[u8; 32], agent_id: u128) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(b"bardo-cache-v1"), master);
    let mut okm = [0u8; 32];
    hk.expand(format!("agent:{agent_id}").as_bytes(), &mut okm)
        .expect("32-byte output valid for HKDF-SHA-256");
    okm
}

/// Clade-shared key using ERC-8004 operatorOf() grouping.
pub fn derive_clade_cache_key(master: &[u8; 32], operator_address: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(b"bardo-cache-v1"), master);
    let mut okm = [0u8; 32];
    hk.expand(format!("clade:{operator_address}").as_bytes(), &mut okm)
        .expect("32-byte output valid for HKDF-SHA-256");
    okm
}
```

AES-256-GCM before storage. With AES-NI hardware acceleration, encrypt/decrypt takes <1us per operation. Negligible next to the 5ms semantic cache lookup.

Master key rotation invalidates all derived keys. Existing entries become undecryptable -- treated as cache misses. Acceptable because TTLs are short (90-300s), and misses cost inference calls not data loss.

---

## 8. Gateway request/response signing

The gateway charges agents USDC for inference. Without signed receipts, it could deny what it returned, inflate costs, or misreport cache status. Ed25519 signed receipts provide non-repudiation.

### GatewayReceipt

```rust
// crates/bardo-telemetry/src/receipt.rs

#[derive(Debug, Clone, Serialize)]
pub struct GatewayReceipt {
    pub request_hash: String,       // SHA-256 of request body sent to provider
    pub response_hash: String,      // SHA-256 of response body from provider
    pub model: String,
    pub provider: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_charged_usdc: String, // decimal string, e.g. "0.003200"
    pub timestamp: u64,
    pub cache_status: CacheStatus,
    pub security_class: SecurityLevel,
}
```

### Signing

```rust
use ed25519_dalek::{Signer, SigningKey};

pub fn sign_receipt(receipt: &GatewayReceipt, signing_key: &SigningKey) -> String {
    let canonical = serde_jcs::to_string(receipt).expect("GatewayReceipt serializable");
    hex::encode(signing_key.sign(canonical.as_bytes()).to_bytes())
}
```

Every `/v1/chat/completions` response includes `X-Bardo-Receipt` (base64-encoded JSON receipt) and `X-Bardo-Receipt-Signature` (hex-encoded Ed25519 signature). The gateway publishes its public key at `GET /.well-known/bardo-gateway-key` in JWK format.

### Guarantees

- **Non-repudiation**: The signed receipt binds the gateway's key to the response hash.
- **Cost integrity**: `total_charged_usdc` is signed and verifiable against x402 settlement.
- **Cache honesty**: The gateway cannot claim a miss (full price) for a cached response.
- **Privacy attestation**: `security_class` in the receipt confirms which privacy level was applied.

---

## 9. Provider trust and routing policies

Not all inference requests carry the same sensitivity. The gateway routes based on sensitivity, preferring providers with stronger data handling guarantees.

### Provider trust matrix

| Provider  | Retention          | Zero-retention option | Notes                                              |
| --------- | ------------------ | --------------------- | -------------------------------------------------- |
| Venice    | None               | Structural            | Zero retention by architecture, TEE-attested       |
| BlockRun  | None (x402 native) | N/A (no retention)    | x402 settlement, no account relationship           |
| Anthropic | 30 days (default)  | Yes (API flag)        | Zero-retention via `anthropic-beta: no-log` header |
| OpenAI    | 30 days (default)  | Yes (org setting)     | Zero-retention via data processing addendum        |
| Google    | Varies by product  | Yes (Vertex AI)       | Vertex AI offers no-logging; AI Studio does not    |

Venice is first in this list because it is the only provider where zero retention is structural, not a policy flag. Policies change without notice. The gateway tracks `lastVerifiedAt` per provider and emits `bardo_provider_trust_stale` alerts when verification exceeds 90 days.

### Routing rules

| Sensitivity | Provider constraint             | Additional requirements                                    |
| ----------- | ------------------------------- | ---------------------------------------------------------- |
| `low`       | Any healthy provider            | None                                                       |
| `medium`    | Prefer zero-retention providers | None                                                       |
| `high`      | Zero-retention providers only   | Strategy redaction (`standard`) required                   |
| `critical`  | Zero-retention providers only   | Strategy redaction (`aggressive`) + full provenance record |

When no zero-retention provider is healthy for a `high`/`critical` request, the gateway returns HTTP 503 with a `Retry-After` header rather than falling back to a retaining provider.

---

## 10. Differential privacy on semantic cache

The semantic cache stores embedding vectors. An attacker with access to the cache storage could use nearest-neighbor search to reconstruct approximate query content. Gaussian noise injection provides differential privacy.

```rust
// crates/bardo-cache/src/privacy.rs
use rand_distr::{Distribution, Normal};

/// Add calibrated Gaussian noise to an embedding vector for differential privacy.
/// Re-normalizes to unit sphere after injection (nomic-embed-text-v1.5 produces
/// unit vectors). Adjust similarity threshold downward to compensate.
pub fn add_differential_privacy(embedding: &mut [f32], epsilon: f64) {
    let sigma = 1.0 / epsilon;
    let normal = Normal::new(0.0, sigma).expect("sigma must be positive");
    let mut rng = rand::thread_rng();
    for val in embedding.iter_mut() {
        *val += normal.sample(&mut rng) as f32;
    }
    let norm: f32 = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in embedding.iter_mut() { *val /= norm; }
    }
}
```

| Epsilon | Noise level | Threshold adjustment | Cache hit rate impact | Use case |
| ------- | ----------- | -------------------- | --------------------- | -------- |
| 1.0     | High        | -0.05                | ~5-8% reduction       | Maximum privacy, shared caches |
| 3.0     | Moderate    | -0.02                | ~2-3% reduction       | Balanced |
| 5.0     | Low         | -0.01                | <1% reduction         | Minimal privacy impact (default) |
| 10.0    | Minimal     | None                 | Negligible            | Per-agent caches (isolation provides privacy) |

Default: `epsilon=5.0`. Enough noise to prevent exact reconstruction while preserving >99% cache hit rate.

---

## 11. Inference provenance records

Every inference request that leads to an on-chain action needs a complete provenance chain. Three linked records capture it: intent, policy decision, and inference details.

```rust
/// What the agent wanted to do (extracted from LLM output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionIntentSchema {
    pub trace_id: String,            // UUID v7 linking all three records
    pub agent_id: u128,
    pub intent_description: String,
    pub operation_type: OperationType,
    pub estimated_value_usdc: f64,
    pub risk_tier: RiskTier,
    pub timestamp: u64,
}

/// Which safety policies were evaluated and what they decided
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecisionRecord {
    pub trace_id: String,
    pub policies_evaluated: Vec<PolicyEvaluation>,
    pub overall_decision: PolicyDecision,
    pub blocking_policy: Option<String>,
    pub timestamp: u64,
}

/// What inference was performed to support this action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRecord {
    pub trace_id: String,
    pub request_id: String,          // links to audit trail (section 5)
    pub model: String,
    pub provider: String,
    pub cost_usdc: f64,
    pub optimizations: Vec<String>,
    pub redaction_level: RedactionLevel,
    pub security_class: SecurityLevel,
    pub cache_status: CacheStatus,
    pub timestamp: u64,
}
```

The `provenance_hash` of all three records is included in `InferenceLogSigned.event_hash`, binding provenance to the tamper-evident chain.

---

## 12. Trust gradient

Features activate progressively by Warden risk tier (optional, deferred). Read-only queries get lightweight logging; admin operations get maximum verification.

| Feature            | Routine               | Standard      | Elevated                 | High                    | Critical                |
| ------------------ | --------------------- | ------------- | ------------------------ | ----------------------- | ----------------------- |
| Audit trail        | Standard (Clickhouse) | Hash-chain    | Hash-chain               | Hash-chain              | Hash-chain              |
| Receipt signing    | No                    | Yes           | Yes                      | Yes                     | Yes                     |
| Strategy redaction | None                  | None          | Standard                 | Aggressive              | Aggressive              |
| Cache encryption   | Per-agent key         | Per-agent key | Per-agent key            | Per-agent key           | Per-agent key           |
| Provider routing   | Any healthy           | Any healthy   | Zero-retention preferred | Zero-retention required | Zero-retention required |
| Provenance record  | No                    | IR only       | Full (TIS + PDR + IR)    | Full                    | Full                    |
| Merkle proof       | No                    | No            | On-demand                | Mandatory               | Mandatory               |
| Security class     | Standard              | Standard      | Confidential             | Private                 | Private                 |

---

## Metrics

| Metric                              | Type      | Alert threshold                                  |
| ----------------------------------- | --------- | ------------------------------------------------ |
| `bardo_audit_chain_length`          | Counter   | --                                               |
| `bardo_audit_anchor_lag`            | Gauge     | >1000 unanchored events                          |
| `bardo_audit_verification_failures` | Counter   | >0                                               |
| `bardo_receipt_signing_latency_us`  | Histogram | P99 >500us                                       |
| `bardo_cache_encryption_latency_us` | Histogram | P99 >100us                                       |
| `bardo_strategy_redaction_applied`  | Counter   | --                                               |
| `bardo_provider_trust_stale`        | Gauge     | >0 (provider trust data >90 days old)            |
| `bardo_provenance_records_created`  | Counter   | --                                               |
| `bardo_high_sensitivity_503`        | Counter   | >10/hour (no zero-retention providers available) |
| `bardo_private_inference_routed`    | Counter   | --                                               |
| `bardo_diem_consumed_daily`         | Gauge     | >90% of allocation (close to cap)                |
| `bardo_security_class_distribution` | Counter   | --                                               |

---

## Open questions

1. **Redaction vs. quality**: LLMs lose domain knowledge when seeing `[TOKEN_A]` instead of `ETH`. Quality impact needs empirical measurement. Research queries may need exemption even at Elevated tier.

2. **Anchoring frequency**: At scale (50K agents), the 256-event batch size needs dynamic scaling. 10K+ events per anchor keeps costs negligible but adds latency to proof availability.

3. **Key management**: The Ed25519 signing key and cache master key are single points of failure. HSM would help but violates no-external-dependencies. Needs key backup, rotation, and compromise response procedures before production.

4. **Provider trust maintenance**: Provider policies are self-reported and change without notice. `lastVerifiedAt` alerts provide visibility, but manual re-verification is required.

5. **Venice availability**: If Venice goes down, private-class requests return 503. Should the gateway allow explicit owner opt-in to fall back to a retaining provider with aggressive redaction? The default (hard fail) is safer but costs availability.

---

## Cross-references

- [07-safety.md](07-safety.md) -- PII detection via compiled regex, prompt injection defense via DeBERTa ONNX classifier, and audit logging for every inference request
- [02-caching.md](02-caching.md) -- Three-layer cache stack (hash, semantic, prefix) with encryption at rest and differential privacy on embedding vectors
- [09-api.md](09-api.md) -- API reference with 33 endpoints including privacy headers, audit signing, and provenance query endpoints
- [08-observability.md](08-observability.md) -- Per-agent cost attribution, OpenTelemetry traces, and Event Fabric integration for privacy-preserving telemetry
- [12-providers.md](12-providers.md) -- Five provider backends including Venice private cognition deep-dive: TEE attestation, E2EE inference, and sensitivity classification
- [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) -- Optional Warden time-delay proxy: announce-wait-execute pattern for high-value transactions with cancel authority
- [../10-safety/00-defense.md](../10-safety/00-defense.md) -- The full 15-layer defense model covering on-chain, runtime, and inference-layer protections
- [00-overview.md](00-overview.md) -- Gateway architecture, x402 payment settlement flows, and dual API format support
- [10-roadmap.md](10-roadmap.md) -- Phased delivery plan; privacy and trust features are spread across Phase 1 (basic auth) and Phase 2 (full audit trail)
- [../05-dreams/07-venice-dreaming.md](../05-dreams/07-venice-dreaming.md) -- Venice-augmented dream cycles: private cognition during NREM replay and REM creative recombination
