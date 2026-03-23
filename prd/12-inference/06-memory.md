# 06 -- Agent memory service [SPEC]

**Styx retrieval augmentation, persistent cross-session memory, importance scoring, background consolidation**

> Related: [04-context-engineering.md](04-context-engineering.md) (8-layer pipeline; memory context budget: 3K tokens, top-K 10), [05-sessions.md](05-sessions.md) (scratchpad is per-session working state; memory is persistent), [09-api.md](09-api.md) (API reference with 33 endpoints including memory management)

---

> **Reader orientation:** This document specifies the agent memory service for Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes how persistent cross-session memory is backed by Styx (the knowledge retrieval and injection plane), with importance scoring and background consolidation. The key concept is that memory persists across sessions and accumulates learned knowledge, separate from the per-session scratchpad in 05-sessions. For term definitions, see `prd2/shared/glossary.md`.

## What is agent memory

Managed persistent memory backed by the Styx knowledge system (Vault for per-Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) storage, Clade (a peer-to-peer network of Golems sharing knowledge) for sibling sharing, Lethe (formerly Commons) for global knowledge). Unlike the scratchpad (per-session working state), agent memory persists across sessions and accumulates learned knowledge over time.

An agent that discovers "USDC/ETH 0.05% pool has consistently higher volume during Asian trading hours" stores this as a memory. Every future session that touches ETH/USDC pools gets this insight injected automatically.

### Styx retrieval integration

The `bardo-oracle` extension (now `bardo-styx`) hooks `before_llm_call` to pre-fetch knowledge from Styx namespaces:

- **Vault (L0)**: Per-Golem knowledge. Always local, always available. Zero network.
- **Clade (L1)**: Sibling knowledge. Shared among Golems in the same lineage.
- **Lethe (L2)**: Global knowledge. Shared across the ecosystem.

Styx entries are injected as Anthropic `search_result` content blocks when a Claude backend is available, enabling Citations for provenance tracking. When Citations are unavailable, the Context Governor falls back to embedding similarity (cosine > 0.8 between injected content and response sentences) for provenance estimation -- ~30% less accurate but functional.

---

## Memory types

```rust
// crates/bardo-gateway/src/memory.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Fact,
    Preference,
    Episode,
    StrategyOutcome,
    Constraint,
}
```

| Type               | Description                    | DeFi example                                             |
| ------------------ | ------------------------------ | -------------------------------------------------------- |
| `fact`             | Learned factual knowledge      | "Morpho's USDC vault has 2.3% base APY as of 2026-03-01" |
| `preference`       | Agent behavioral preferences   | "Prefer 0.30% fee tier for stablecoin pairs"             |
| `episode`          | Specific event memories        | "Lost 2.3% on ETH short during March 2026 flash crash"   |
| `strategy_outcome` | Strategy performance records   | "DCA into ETH over 30 days yielded +12% vs. lump sum"    |
| `constraint`       | Learned operational boundaries | "Never LP in pools with <$100K TVL -- slippage too high" |

---

## Memory storage and retrieval

### Store

```rust
// crates/bardo-gateway/src/memory.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStoreRequest {
    /// Memory content (freeform text).
    pub content: String,
    /// Memory type classification.
    pub memory_type: MemoryType,
    /// Importance score (0.0-1.0). Higher = more likely to be retrieved.
    pub importance: f64,
    /// Optional metadata for filtering.
    pub metadata: Option<MemoryMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub tokens: Option<Vec<String>>,
    pub chains: Option<Vec<u64>>,
    pub protocols: Option<Vec<String>>,
    pub strategy_id: Option<String>,
}
```

### Search

```rust
// crates/bardo-gateway/src/memory.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    /// Natural language query.
    pub query: String,
    /// Maximum memories to return (default: 10).
    pub top_k: Option<u32>,
    /// Filter by memory type.
    pub type_filter: Option<Vec<MemoryType>>,
    /// Minimum importance threshold.
    pub min_importance: Option<f64>,
}
```

### Endpoints

| Method | Path                | Purpose                                |
| ------ | ------------------- | -------------------------------------- |
| `POST` | `/v1/memory`        | Store a new memory                     |
| `POST` | `/v1/memory/search` | Search memories by semantic similarity |

---

## Importance scoring

Memories are ranked by a composite importance score that blends three signals: recency, access frequency, and the explicit importance the author assigned at write time.

```rust
// crates/bardo-gateway/src/memory.rs

/// Composite importance score combining recency decay, access frequency,
/// and author-assigned importance.
pub fn calculate_importance(memory: &StoredMemory) -> f64 {
    const RECENCY_WEIGHT: f64 = 0.3;
    const ACCESS_WEIGHT: f64 = 0.3;
    const EXPLICIT_WEIGHT: f64 = 0.4;

    // Exponential decay over 30 days
    let recency_score = (-memory.age_in_days / 30.0).exp();
    // Saturates at 10 accesses
    let access_score = (memory.access_count as f64 / 10.0).min(1.0);
    let explicit_score = memory.importance;

    RECENCY_WEIGHT * recency_score
        + ACCESS_WEIGHT * access_score
        + EXPLICIT_WEIGHT * explicit_score
}
```

Memories that are frequently retrieved gain importance. Memories that are never accessed decay naturally.

---

## Context injection

When memory is enabled for a session, the gateway retrieves the top-K most relevant memories and assembles them into structured XML for the system message:

```rust
// crates/bardo-gateway/src/memory.rs

/// Build the `<agent_memory>` XML block from retrieved memories,
/// truncating to fit within `budget_tokens`.
pub fn inject_memory_context(
    memories: &[StoredMemory],
    budget_tokens: usize,
) -> String {
    let mut xml = String::from("<agent_memory>\n");
    let mut tokens_used = 4; // opening + closing tag overhead

    for m in memories {
        let entry = format!(
            "  <memory type=\"{}\" importance=\"{:.2}\" age=\"{}\">\n    {}\n  </memory>\n",
            m.memory_type.as_str(),
            m.importance,
            m.age_display(),
            m.content,
        );

        let entry_tokens = entry.len() / 4; // rough char-to-token estimate
        if tokens_used + entry_tokens > budget_tokens {
            break;
        }
        tokens_used += entry_tokens;
        xml.push_str(&entry);
    }

    xml.push_str("</agent_memory>");
    xml
}
```

The output looks like this:

```xml
<agent_memory>
  <memory type="constraint" importance="0.95" age="2d">
    Never LP in pools with less than $100K TVL. Slippage on exit
    caused 3.2% loss on 2026-02-28.
  </memory>
  <memory type="strategy_outcome" importance="0.82" age="7d">
    DCA strategy on ETH over 30 days outperformed lump sum by 12%
    during the Feb 2026 correction.
  </memory>
  <memory type="fact" importance="0.70" age="1d">
    Morpho USDC vault base APY: 2.3%. Updated 2026-03-08.
  </memory>
</agent_memory>
```

Memory is allocated 3,000 tokens in the default context budget (see [04-context-engineering.md](04-context-engineering.md)). Top-K defaults to 10 memories.

---

## Background consolidation

Every 24 hours, a background job runs Haiku over each agent's memory store to:

1. **Deduplicate** -- merge memories that say the same thing differently
2. **Update** -- revise facts with newer information (e.g., APY changes)
3. **Re-rank** -- adjust importance scores based on access patterns
4. **Prune** -- archive memories below a minimum importance threshold

This is the MemGPT/Letta "working context block" pattern. The memory store is a living document that gets continuously refined, not an append-only log.

Consolidation runs on Haiku (~$0.001 per agent per day). The cost is negligible relative to the value of clean, deduplicated memory.

---

## Storage tiers and pricing

| Tier     | Max memories | Queries/day | Consolidation     | Cost                    |
| -------- | ------------ | ----------- | ----------------- | ----------------------- |
| Free     | 100          | 50          | Weekly            | $0 (included in margin) |
| Standard | 5,000        | 500         | Daily             | $2/month                |
| Premium  | 50,000       | Unlimited   | Daily + on-demand | $10/month               |

Storage backend: Qdrant (self-hosted) for vector similarity, PostgreSQL for metadata and full-text search.

---

## Memory citation mechanics

### Anthropic Citations for Grimoire provenance

The `bardo-styx` extension retrieves entries from the local Grimoire (the agent's persistent knowledge base; always available) and optionally from the hosted Oracle (if enabled). These entries are injected as Anthropic `search_result` content blocks in the request body. Bardo Inference passes them through to any Claude-capable backend (BlockRun, OpenRouter, Bankr, Direct Anthropic key). Claude responds with `citation` objects pointing back to specific entries.

```rust
// Oracle extension prepares search_result blocks
pub async fn inject_grimoire_entries(
    ctx: &mut LlmCallCtx,
    grimoire: &Grimoire,
    oracle_mode: OracleMode,
) -> Result<()> {
    // Always retrieve from local Grimoire
    let entries = grimoire.retrieve(&ctx.current_message(), 10).await?;

    // If hosted Oracle enabled, also search L1/L2
    if oracle_mode == OracleMode::Hosted {
        let hosted = hosted_oracle.retrieve(
            &ctx.current_message(),
            &["L1_clade", "L2_global"],
            5,
        ).await?;
        entries.extend(hosted);
    }

    // Inject as search_result blocks -- Bardo Inference passes these
    // through to whatever Claude backend it routes to
    for entry in &entries {
        ctx.add_search_result(SearchResult {
            source: format!("grimoire://{}/{}/{}", entry.namespace, entry.entry_type, entry.id),
            title: format!("[{}] {}", entry.entry_type.to_uppercase(), entry.title_or_id()),
            content: entry.content.clone(),
            citations_enabled: true,
        });
    }
    Ok(())
}
```

**Context engineering interaction**: Bardo Inference's prompt cache alignment (Layer 1) orders the `search_result` blocks by stability -- Grimoire entries that haven't changed recently get placed in the cached prefix. New entries go after the cache boundary. Stable knowledge gets cached (90% discount) while fresh knowledge pays full price.

### Provenance feedback loop

Citations create a closed feedback loop for the Context Governor:

- Entry cited -> category USEFUL -> increase weight for future ticks
- Entry injected but not cited -> WASTEFUL -> decrease weight
- Response references knowledge not in any entry -> MISSING context signal -> trigger Oracle re-ranking

**Without Claude access**: If no Claude backend is configured, citations are unavailable. The Context Governor falls back to embedding similarity (cosine > 0.8 between injected content and response sentences) for provenance estimation. ~30% less accurate but functional.

---

## Context assembly details

### Compaction through Bardo Inference

When the Golem's session approaches the context limit, Bardo Inference can delegate to Anthropic's server-side Compaction API (if a Claude backend is available) with DeFi-aware custom instructions:

```text
Preserve with EXACT values:
1. VAULT STATE: Addresses, token balances, LP positions, USDC values
2. PENDING ACTIONS: ActionPermit IDs, expiry times, parameters
3. STRATEGY PARAMETERS: PLAYBOOK.md heuristics with exact thresholds
4. RISK STATE: Current tier, active guardrails, drawdown
5. EMOTIONAL STATE: PAD vector values, Plutchik label
6. MORTALITY STATE: Vitality score, phase, projected TTL
7. DECISIONS: Key decisions and rationale
8. OPEN QUESTIONS: Unresolved operator queries
```

When no Claude backend is available, Bardo Inference's history compression layer (Layer 5) handles it using the cheapest available model (Haiku, Gemini Flash, or Qwen). The compression model can differ from the request model -- Bardo Inference uses Haiku for compression even if the main request goes to Venice/R1.

### Stacked caching for knowledge context

Three layers of caching compound for Grimoire/memory content:

```text
Request with Grimoire entries arrives at Bardo Inference
    |
    +- Layer 2: Semantic cache check (application level)
    |   Hit? -> Return cached response. Cost: $0.00. Latency: <10ms.
    |
    +- Layer 3: Hash cache check (application level)
    |   Hit? -> Return cached response. Cost: $0.00. Latency: <5ms.
    |
    +- Layer 1: Prompt cache alignment (optimize for provider cache)
    |   Reorder for stable prefix -> maximize provider cache hits
    |
    +-> Send to backend
         |
         +- Anthropic prompt caching: 90% discount on cached prefix
         +- Gemini implicit caching: automatic discount on repeated content
```

### Feature degradation by configuration

| Feature | Full Capability | Without Claude | Without hosted Oracle | Minimal (single backend) |
|---|---|---|---|---|
| Grimoire retrieval | L0+L1+L2 with citations | L0+L1+L2, heuristic provenance | L0 only, with citations | L0 only, heuristic provenance |
| Session continuity | Server-side Compaction | Client-side compression | Same as full | Client-side compression |
| Caching | 3-layer stacked (70% savings) | 2-layer (semantic + hash) | Same as full | 2-layer only |
| PLAYBOOK evolution | Predicted Outputs (3x speed) | Full regeneration | Same as full | Full regeneration |
| Knowledge tokenization | Bankr token launch (requires Crypt) | Same as full | Same as full | Not available |

The minimal configuration (single BlockRun backend, no hosted services) still gets Bardo Inference's universal context engineering (8 layers), local Grimoire, and client-side session management. Every additional backend or service is strictly additive.

---

## Cross-references

| Topic                             | Document                                               | What it covers |
| --------------------------------- | ------------------------------------------------------ | -------------- |
| Memory context budget (3K tokens) | [04-context-engineering.md](04-context-engineering.md) | 8-layer pipeline including memory injection budget allocation and Context Governor workspace assembly |
| Scratchpad (per-session memory)   | [05-sessions.md](05-sessions.md)                       | Checkpoint/resume, scratchpad working memory (per-session, not persistent), and sub-agent spawning |
| Memory API endpoints              | [09-api.md](09-api.md)                                 | API reference with 33 endpoints including memory store, retrieve, and search operations |
| Revenue from Memory-as-a-Service  | [03-economics.md](03-economics.md)                     | x402 spread revenue model including memory service economics and per-tenant cost attribution |
| Grimoire provenance (Citations)   | [12-providers.md](12-providers.md) (Anthropic section)  | Five provider backends; the Anthropic section covers Citations for Grimoire provenance tracking |
| Context Governor feedback loops   | [04-context-engineering.md](04-context-engineering.md)  | Three cybernetic feedback loops that tune memory injection weights based on decision outcomes |
