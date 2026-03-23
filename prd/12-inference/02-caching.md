# 02 -- Caching and optimization [SPEC]

**Three-layer cache stack, prompt prefix alignment, KV-cache routing, and regime-aware invalidation**

> Related: [01a-routing.md](01a-routing.md) (self-describing providers and mortality-aware model resolution), [03-economics.md](03-economics.md) (x402 spread revenue model and per-tenant cost attribution), [04-context-engineering.md](04-context-engineering.md) (8-layer context engineering pipeline for cost reduction), [11-privacy-trust.md](11-privacy-trust.md) (cache encryption, differential privacy on embeddings, and cryptographic audit trail)

---

> **Reader orientation:** This document specifies the caching and optimization layer of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes the three-layer cache stack that reduces LLM costs by 40-85%. The key concept is that stable prompt prefixes, semantic similarity matching, and deterministic hash lookups can eliminate or heavily discount most inference calls, with cache invalidation driven by market regime changes. For term definitions, see `prd2/shared/glossary.md`.

## Why caching dominates inference economics

Without prompt caching, a heavy agent session (~20M tokens on Opus) costs ~$100. With 90% cache hit rate: ~$19. Anthropic's Claude Code team declares internal SEV incidents when cache hit rates drop [ANTHROPIC-CONTEXT-ENG].

| Layer                            | Type                         | Savings               | Latency      | Storage              |
| -------------------------------- | ---------------------------- | --------------------- | ------------ | -------------------- |
| **L3: Deterministic (Hash)**     | Exact-match SHA-256          | 100% (zero inference) | <1ms         | In-memory LRU        |
| **L2: Semantic**                 | Embedding cosine similarity  | 100% (zero inference) | 5-20ms       | Redis + vector index |
| **L1: Prompt Prefix (Provider)** | Provider-side KV token reuse | ~90% on cached prefix | 0ms overhead | Provider-managed     |

L3 handles ~10% of requests, L2 handles ~30% of L3 misses, L1 applies to all remaining. Expected aggregate savings: 60-80%. See [04-context-engineering.md](04-context-engineering.md) for the full 8-layer pipeline.

### DeFi enrichment injection

For any `domain: "defi"` request, the gateway injects pre-fetched market data before the LLM call. Not a cache layer -- context augmentation that eliminates 2-4 tool call round trips.

```xml
<defi_context freshness="2m">
  <price token="ETH" usd="3847.22" change24h="-2.1%" />
  <price token="USDC" usd="0.9998" />
  <pool protocol="uniswap-v3" pair="USDC-ETH" tvl="$14.2M" volume24h="$8.7M" fees24h="$12,400" />
  <rate protocol="morpho" asset="USDC" supplyAPY="4.2%" borrowAPY="5.1%" utilization="82%" />
</defi_context>
```

The enrichment block is 300-500 tokens, replacing ~4,800 tokens of tool call round trips. Net savings: ~4,300 tokens per task. Freshness: `1m | 5m | 15m | 1h`. When Venice DIEM is configured, semantic cache misses routed to Venice incur zero inference cost.

```rust
// crates/bardo-cache/src/enrichment.rs

use crate::types::InferenceRequest;

/// Assembles `<defi_context>` XML from cached market sources.
/// Returns `None` if domain is not DeFi or all sources fail.
pub async fn assemble_defi_enrichment(
    request: &InferenceRequest,
    cache: &MarketDataCache,
) -> Option<String> {
    if request.domain.as_deref() != Some("defi") { return None; }
    let tokens = request.extract_mentioned_tokens();
    if tokens.is_empty() { return None; }

    let mut xml = format!("<defi_context freshness=\"{}\">", cache.oldest_entry_age());

    for t in &tokens {
        if let Some(p) = cache.get_price(t).await {
            xml.push_str(&format!(
                "\n  <price token=\"{}\" usd=\"{:.2}\" change24h=\"{:.1}%\" />",
                t, p.usd, p.change_24h_pct));
        }
    }
    for pool in &cache.get_pools_for_tokens(&tokens).await {
        xml.push_str(&format!(
            "\n  <pool protocol=\"{}\" pair=\"{}\" tvl=\"${}\" volume24h=\"${}\" fees24h=\"${}\" />",
            pool.protocol, pool.pair, pool.tvl_fmt, pool.vol_24h_fmt, pool.fees_24h_fmt));
    }
    for r in &cache.get_rates_for_tokens(&tokens).await {
        xml.push_str(&format!(
            "\n  <rate protocol=\"{}\" asset=\"{}\" supplyAPY=\"{:.1}%\" borrowAPY=\"{:.1}%\" utilization=\"{:.0}%\" />",
            r.protocol, r.asset, r.supply_apy, r.borrow_apy, r.utilization_pct));
    }
    xml.push_str("\n</defi_context>");
    Some(xml)
}
```

---

## L3: Deterministic cache (hash, exact match)

The fastest and cheapest cache layer. Normalize the prompt, compute SHA-256, look up in an LRU cache. Hit = zero cost, sub-millisecond response.

```rust
// crates/bardo-cache/src/hash.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashCacheEntry {
    pub hash: String,             // SHA-256 of normalized prompt
    pub response: String,         // cached LLM response body
    pub model_id: String,
    pub created_at: u64,          // unix seconds
    pub ttl_seconds: u64,         // 300 for market data, 3600 for static analysis
    pub strategy_id: String,
}
```

Normalization rules:

1. Strip all timestamps and request IDs from messages
2. Sort tool result arrays by deterministic key (tool name, then parameter hash)
3. Remove ephemeral metadata (`traceId`, `requestId`, session counters)
4. Canonicalize whitespace in system prompts

```rust
// crates/bardo-cache/src/normalize.rs

use sha2::{Sha256, Digest};
use crate::types::Message;

/// Applies all four normalization rules, returns SHA-256 digest bytes.
/// Order: strip ephemerals -> sort tool results -> canonicalize whitespace.
pub fn normalize_for_hash(messages: &[Message]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    for msg in messages {
        hasher.update(msg.role.as_bytes());
        hasher.update(b"|");

        let content = canonicalize_whitespace(&strip_ephemerals(&msg.content));
        hasher.update(content.as_bytes());
        hasher.update(b"|");

        if let Some(results) = &msg.tool_results {
            let mut sorted = results.clone();
            sorted.sort_by_key(|r| format!("{}:{}", r.tool_name, hash_params(&r.parameters)));
            for r in &sorted {
                hasher.update(r.tool_name.as_bytes());
                hasher.update(b":");
                hasher.update(r.output.as_bytes());
                hasher.update(b"|");
            }
        }
    }
    hasher.finalize().to_vec()
}

fn strip_ephemerals(content: &str) -> String {
    let re_ts = regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?").unwrap();
    let re_uuid = regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();
    let re_fields = regex::Regex::new(r#""(traceId|requestId|sessionCounter)"\s*:\s*"[^"]*""#).unwrap();
    let out = re_ts.replace_all(content, "");
    let out = re_uuid.replace_all(&out, "");
    re_fields.replace_all(&out, "").into_owned()
}

fn canonicalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn hash_params(params: &serde_json::Value) -> String {
    let digest = Sha256::digest(serde_json::to_string(params).unwrap_or_default().as_bytes());
    hex::encode(&digest[..8])
}
```

The hash cache is per-agent by default. Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) siblings in the same Clade (a peer-to-peer network of Golems sharing knowledge) hit each other's cached responses when querying through the same gateway.

---

## L2: Semantic cache (embedding similarity)

For semantically equivalent prompts that don't match exactly, compute an embedding via `nomic-embed-text-v1.5` (local ONNX via `fastembed-rs`, ~3-5ms) and search an in-memory HNSW index. Cosine > 0.92 = hit. The gateway simulates SSE streaming at ~30ms intervals for consistent UX.

TTLs are regime-aware: 300s calm, 210s normal, 90s volatile, 30s crisis.

```rust
// crates/bardo-cache/src/semantic.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheConfig {
    /// 0.92 market analysis, 0.95 strategy, 0.98 trade execution.
    pub similarity_threshold: f64,
    pub max_entries: usize,                // default: 10,000
    pub base_ttl_seconds: u64,             // 300 (calm)
    pub volatile_regime_multiplier: f64,   // 0.3 -> 90s volatile
    pub isolation_mode: CacheIsolationMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheIsolationMode { PerAgent, Shared, Tiered }

impl Default for SemanticCacheConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.92,
            max_entries: 10_000,
            base_ttl_seconds: 300,
            volatile_regime_multiplier: 0.3,
            isolation_mode: CacheIsolationMode::PerAgent,
        }
    }
}
```

### Semantic cache operations

```rust
// crates/bardo-cache/src/semantic_ops.rs

use fastembed::TextEmbedding;

/// In-memory HNSW + fastembed-rs for single-instance.
/// Optional Redis 7+ backend for multi-instance deployments.
pub struct SemanticCache {
    embedder: TextEmbedding,  // nomic-embed-text-v1.5, local ONNX via ort
    index: HnswIndex,         // instant-distance or qdrant-embedded
}

impl SemanticCache {
    pub async fn lookup(
        &self, request: &ChatCompletionRequest, agent_id: u64, config: &SemanticCacheConfig,
    ) -> CacheLookupResult {
        let key = Self::build_cache_key(&request.messages);
        let emb = self.embedder.embed(vec![&key], None).expect("embed text");
        let idx = match config.isolation_mode {
            CacheIsolationMode::PerAgent => format!("idx:cache:{}", agent_id),
            _ => "idx:cache:shared".into(),
        };

        let results: Vec<CachedEntry> = redis::cmd("FT.SEARCH")
            .arg(&idx)
            .arg(format!("@embedding:[VECTOR_RANGE {} $blob]", 1.0 - config.similarity_threshold))
            .arg("PARAMS").arg(2).arg("blob").arg(&embedding_to_bytes(&emb[0]))
            .arg("RETURN").arg(4).arg("response").arg("model").arg("created_at").arg("token_count")
            .arg("LIMIT").arg(0).arg(1)
            .query_async(&mut self.redis.clone()).await.unwrap_or_default();

        if results.is_empty() { return CacheLookupResult::miss(None); }
        let c = &results[0];
        let age = now_secs() - c.created_at;
        if age > config.base_ttl_seconds { return CacheLookupResult::miss(Some("expired")); }
        CacheLookupResult::hit(1.0 - c.distance, c.response.clone(), c.model.clone(), c.token_count)
    }

    /// Never cache: tool-call responses, completions < 10 tokens, errors.
    pub async fn store(&self, _req: &ChatCompletionRequest, resp: &ChatCompletionResponse, _agent_id: u64, _cfg: &SemanticCacheConfig) {
        if resp.has_tool_calls() || resp.completion_tokens() < 10 { return; }
        // embed, write to Redis with regime-aware TTL
    }

    fn build_cache_key(messages: &[Message]) -> String {
        let sys = messages.iter().find(|m| m.role == "system").map(|m| m.content.as_str()).unwrap_or("");
        let usr: String = messages.iter().filter(|m| m.role == "user").rev().take(2)
            .map(|m| m.content.as_str()).collect::<Vec<_>>().join("\n");
        format!("{}\n---\n{}", sys, usr)
    }
}

fn embedding_to_bytes(e: &[f32]) -> Vec<u8> { e.iter().flat_map(|f| f.to_le_bytes()).collect() }
fn now_secs() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }
```

### DeFi similarity threshold caveat

The default 0.92 threshold requires empirical tuning. "Supply USDC to Morpho" and "supply USDC to Aave" score cosine ~0.94 but require different actions. Per-domain thresholds:

| Domain             | Threshold | Rationale              |
| ------------------ | --------- | ---------------------- |
| Market analysis    | 0.92      | Lower precision needed |
| Strategy reasoning | 0.95      | Action-sensitive       |
| Trade execution    | 0.98      | Protocol-specific      |
| Static reference   | 0.90      | Stable answers         |

### Cache isolation modes

| Mode        | Behavior                                                                                     | Use case                                |
| ----------- | -------------------------------------------------------------------------------------------- | --------------------------------------- |
| `per-agent` | Each agent has its own cache namespace                                                       | Default. Privacy-preserving.            |
| `shared`    | All agents share a single cache pool                                                         | General knowledge queries. Opt-in only. |
| `tiered`    | Shared for non-sensitive; per-agent for queries with wallet addresses, strategy details, PII | Best of both. V2 default.               |

Expected savings: 15-30% reduction in LLM calls across a typical Clade.

---

## L1: Prompt prefix cache (provider-side KV reuse)

Providers cache repeated system prompt prefixes server-side, reducing input token cost by ~90%. The gateway maximizes hits via provider-sticky routing.

```rust
// crates/bardo-cache/src/prefix.rs

use crate::types::{InferenceRequest, ProviderInstance};
use async_trait::async_trait;

/// Routes requests to provider instances by system prompt prefix hash.
#[async_trait]
pub trait PromptCacheRouter: Send + Sync {
    async fn route(&self, request: &InferenceRequest) -> ProviderInstance;
}
```

The routing key hashes the first N tokens of the system prompt (PLAYBOOK.md (the Golem's self-authored strategy document), STRATEGY.md, DeFi Constitution). Golems running the same strategy share a common prefix, so requests cluster naturally. For a Clade of 3 Golems: ~$0.006/call without routing drops to ~$0.002/call with it (3x reduction on system prompt cost).

Anthropic's cache expires after 5 minutes of inactivity (extendable to 1 hour). The gateway sends keepalive pings for agents with periodic activity [ANTHROPIC-CONTEXT-ENG].

---

## KV-cache-aware session routing

Session affinity routes requests to endpoints holding warm KV-cache. Up to 87.4% cache hit rate, 88% faster TTFT [IBM-KVFLOW].

```rust
// crates/bardo-cache/src/affinity.rs

use serde::{Deserialize, Serialize};
use crate::types::{ChatCompletionRequest, RouteDecision};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAffinityState {
    pub agent_id: u64,
    pub session_id: String,
    pub provider: String,
    pub model: String,
    pub last_request_at: u64,
    pub prefix_tokens: u64,
    pub prefix_hash: String,
}

const AFFINITY_TTL_SECS: u64 = 300; // matches Anthropic's cache TTL

pub async fn route_with_cache_affinity(
    route: RouteDecision, request: &ChatCompletionRequest,
    session_id: &str, store: &impl SessionStore,
) -> RouteDecision {
    let key = format!("affinity:{}", session_id);
    if let Some(aff) = store.get(&key).await {
        let now = now_secs();
        if aff.provider == route.provider && aff.model == route.model
            && (now - aff.last_request_at) < AFFINITY_TTL_SECS
        {
            return RouteDecision {
                cache_affinity: true,
                estimated_cache_savings: (aff.prefix_tokens as f64 * 0.9) as u64,
                ..route
            };
        }
    }
    store.set(&key, &SessionAffinityState {
        agent_id: route.agent_id,
        session_id: session_id.into(),
        provider: route.provider.clone(),
        model: route.model.clone(),
        last_request_at: now_secs(),
        prefix_tokens: estimate_stable_prefix_tokens(&request.messages),
        prefix_hash: hash_prefix(&request.messages),
    }, AFFINITY_TTL_SECS).await;
    route
}
```

For agents running multi-step workflows (research -> analysis -> execution), the gateway embeds step metadata in requests. This lets the backend update cache eviction priorities based on which agents run next -- up to 1.83x speedup over standard prefix caching [IBM-KVFLOW].

---

## Cache invalidation

Stale market data is worse than no cache. Invalidation must be aggressive.

| Trigger               | Scope                                   | Mechanism                                       |
| --------------------- | --------------------------------------- | ----------------------------------------------- |
| Regime change         | All market-analysis entries             | Immediate invalidation                          |
| On-chain state change | Position-specific entries               | Pool swap, position change events               |
| TTL expiry            | Per-entry                               | Regime-aware TTL (shorter during volatility)    |
| Heuristic update      | Entries that depended on old heuristics | Reflector invalidation when PLAYBOOK.md updates |
| Manual flush          | Per-agent or global                     | Admin endpoint or API call                      |

### Regime-aware TTL

```rust
// crates/bardo-cache/src/ttl.rs

use serde::{Deserialize, Serialize};
use crate::semantic::SemanticCacheConfig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MarketRegime { Calm, Normal, Volatile, Crisis }

pub fn calculate_ttl(config: &SemanticCacheConfig, regime: MarketRegime) -> u64 {
    match regime {
        MarketRegime::Calm     => config.base_ttl_seconds,                                  // 300s
        MarketRegime::Normal   => (config.base_ttl_seconds as f64 * 0.7) as u64,            // 210s
        MarketRegime::Volatile => (config.base_ttl_seconds as f64 * config.volatile_regime_multiplier) as u64, // 90s
        MarketRegime::Crisis   => 30,                                                       // near-real-time
    }
}
```

---

## Cache metrics

| Metric                        | Type    | Alert threshold     |
| ----------------------------- | ------- | ------------------- |
| `bardo_cache_hit_rate`        | Gauge   | <30% for 15 min     |
| `bardo_cache_entries`         | Gauge   | >maxEntries * 0.95  |
| `bardo_cache_savings_usd`     | Counter | --                  |
| `bardo_cache_stale_served`    | Counter | >0                  |
| `bardo_prefix_cache_hit_rate` | Gauge   | <50% for 15 min     |
| `bardo_cache_invalidations`   | Counter | Spike detection     |

---

## Configuration

```bash
# Semantic cache
BARDO_INFERENCE_CACHE_THRESHOLD=0.92
BARDO_INFERENCE_CACHE_TTL=300
BARDO_INFERENCE_CACHE_MAX_ENTRIES=10000
BARDO_INFERENCE_CACHE_ISOLATION=per-agent

# Prompt prefix cache
BARDO_INFERENCE_PREFIX_CACHE=true
BARDO_INFERENCE_CACHE_KEEPALIVE=true
BARDO_INFERENCE_AFFINITY_TTL=300

# Hash cache
BARDO_INFERENCE_HASH_CACHE_SIZE=5000
```

---

## Provider cache discount reference

| Provider  | Cached input discount | Default TTL            | Notes                              |
| --------- | --------------------- | ---------------------- | ---------------------------------- |
| Anthropic | 90% off               | 5 min (extendable 1hr) | Explicit cache breakpoints         |
| OpenAI    | 50% off               | Automatic              | No explicit cache control          |
| Google    | 75% off               | Implicit               | Vertex AI only, AI Studio does not |

Discounts are fetched dynamically from BlockRun's model catalog. The prompt cache alignment techniques in [04-context-engineering.md](04-context-engineering.md) maximize the hit rate for these discounts.

### Semantic cache threshold tuning

| Threshold | Hit rate       | Accuracy     | Best for                             |
| --------- | -------------- | ------------ | ------------------------------------ |
| 0.98      | Very low ~5%   | Near-perfect | Safety-critical, financial decisions |
| 0.95      | Low ~12%       | Excellent    | Strategy analysis, code generation   |
| 0.92      | Moderate ~20%  | Very good    | General Q&A, explanations (default)  |
| 0.88      | High ~35%      | Good         | Repetitive queries, status checks    |
| 0.85      | Very high ~45% | Acceptable   | FAQ-style, low-stakes queries        |

---

## Compound savings from stacked caching

The three cache layers compound multiplicatively. Worked example with 100K input tokens to Claude Opus:

```
Bardo semantic cache: 20% hit rate -> 20% of requests are free
For the 80% that reach Anthropic: 70% of tokens hit prompt cache -> 90% discount
Net effective cost: 0.20 x $0.00 + 0.80 x (0.30 x $0.50 + 0.70 x $0.05) = $0.148
vs. Direct with no caching: $0.50
-> 70% savings from stacked caching
```

---

## Gemini explicit caching (Direct Key enhancement)

When a Direct Google Key is configured, Bardo Inference (the LLM inference gateway) can create persistent Gemini caches for the Golem's system prompt. Unlike Anthropic's fixed 5-min/1hr TTL windows, Gemini supports arbitrary TTLs per cache entry.

```typescript
// Bardo Inference manages Gemini explicit cache lifecycle
class GeminiCacheManager {
  async ensureCacheExists(systemPrompt: string, playbook: string): Promise<string> {
    const contentHash = hash(systemPrompt + playbook);
    const cached = this.cacheRegistry.get(contentHash);
    if (cached && !cached.expired) return cached.cacheId;

    const cache = await geminiClient.caches.create({
      model: "gemini-3-flash-preview",
      contents: [{ role: "user", parts: [{ text: systemPrompt + "\n\n" + playbook }] }],
      ttl: `${4 * 3600}s`, // 4-hour TTL
    });

    this.cacheRegistry.set(contentHash, { cacheId: cache.name, expiresAt: Date.now() + 4 * 3600 * 1000 });
    return cache.name;
  }
}
```

This is a Direct Key-only feature because the `client.caches.create()` API requires the Gemini SDK with the user's own credentials. BlockRun and OpenRouter route Gemini requests but do not expose the explicit caching API.

---

## References

- [ANTHROPIC-CONTEXT-ENG] Anthropic (2025). "Building Effective Agents." Context engineering patterns from Claude Code, Cursor, Aider, Amp. Establishes that stable prompt prefixes optimized for cache hits are the single highest-leverage cost optimization for production agents; the basis for L1 prefix alignment.
- [IBM-KVFLOW] IBM/Google/Red Hat. "llm-d: KV-Cache-Aware Routing for LLM Inference." 87.4% cache hit rate, 88% faster TTFT. Demonstrates that routing requests to servers already holding relevant KV-cache state dramatically improves time-to-first-token; informs L6 KV-cache routing.
- [ROUTELLM-ICLR2025] Ong, I. et al. (2025). "RouteLLM: Learning to Route LLMs with Preference Data." ICLR 2025. Shows that a learned classifier can route queries to appropriate model tiers, reducing cost with minimal quality loss; validates combining caching with intelligent routing.
