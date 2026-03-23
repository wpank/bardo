# 04 -- Context Engineering [SPEC]

**Layered Prompt Cache Architecture, Dynamic Tool Registry, Context Budget Allocation, and Prompt Assembly**

> Related: [01a-routing.md](01a-routing.md) (self-describing providers and mortality-aware model resolution), [02-caching.md](02-caching.md) (three-layer cache stack with regime-aware invalidation), [03-economics.md](03-economics.md) (x402 spread revenue model and cost impact analysis)

---

> **Reader orientation:** This document specifies the context engineering pipeline of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes the 8-layer pipeline that reduces inference cost by 40-85%: prompt cache alignment, semantic cache, hash cache, tool pruning, history compression, KV-cache routing, PII masking, and injection detection. The key concept is that context assembly is treated as a learnable control problem, not a static template, with three cybernetic feedback loops that optimize token allocation per regime and task type. For term definitions, see `prd2/shared/glossary.md`.

## Why context engineering is the core differentiator

The gateway is not just a proxy. It is a **context engineering platform** -- a managed infrastructure layer that applies production-tested patterns from Cursor, Claude Code, Aider, Amp, and Devin to any autonomous agent [ANTHROPIC-CONTEXT-ENG].

These techniques represent the gap between naive prompting and production-grade inference: 10-100x cost differences and measurable accuracy gains. An agent builder who sends raw prompts to an LLM provider leaves 40-85% of their budget on the table. The gateway applies these optimizations transparently -- the agent asks for a completion; the gateway finds the cheapest way to deliver equivalent quality.

## 8-Layer Context Engineering Pipeline

Every request passes through up to 8 layers. Layers are independent and can be bypassed per pipeline profile.

| Layer | Name | Latency | Impact | Parallel? |
|---|---|---|---|---|
| L1 | Prompt cache alignment | <1ms | Prefix cache hits | No (first) |
| L2 | Semantic cache | 3-8ms | Zero cost, similar queries | Yes (L3,L7,L8) |
| L3 | Hash cache | <0.1ms | Zero cost, exact matches | Yes |
| L4 | Tool pruning | <1ms | 97.5% tool token reduction | No (after cache) |
| L5 | History compression | 200-2000ms | Lossy compaction | No (conditional) |
| L6 | KV-cache routing | <1ms | Session-affinity warm cache | No (last) |
| L7 | PII masking | <1ms | Round-trip de-identification | Yes (L2,L3,L8) |
| L8 | Injection detection | 3-8ms | DeBERTa classifier | Yes (L2,L3,L7) |

L2, L3, L7, and L8 execute in parallel via `tokio::join!` (~8ms wall clock instead of ~15ms sequential). L5 is conditional -- only triggered when history exceeds the token budget. Full pipeline adds <50ms p95 (Rust). See [14-rust-implementation.md](14-rust-implementation.md).

## Three-Layer Prompt Cache Architecture

The most consequential optimization in the entire gateway. Without prompt caching, a heavy agent session (~20M tokens on Opus) costs ~$100. With 90% cache hit rate, it drops to ~$19.

```text
+----------------------------------------------------------+
|  L1: Global Prefix (~2-4K tokens)                        |
|  Shared across ALL agents. Identical bytes.              |
|  Gateway identity, behavioral rules, safety, format.     |
|  Cache: 90% discount Anthropic, 50% OpenAI              |
+----------------------------------------------------------+
|  L2: Tenant Prefix (~1-10K tokens)                       |
|  Shared across all sessions for one agentId.             |
|  System prompt, tool defs, PLAYBOOK.md, memory.          |
|  Cache: Reused across all requests from this agent       |
+----------------------------------------------------------+
|  L3: Session Context (variable)                          |
|  Unique per request.                                     |
|  History, RAG results, DeFi enrichment, user query.      |
|  Cache: None (too variable)                              |
+----------------------------------------------------------+
```

### Critical implementation rules

**1. 100% identical prefix bytes.** Any difference -- a timestamp, a changed tool definition, a reordered parameter -- invalidates the cache for everything after that point. The gateway strips timestamps, request IDs, and per-request variables from prefix layers.

**2. Append-only history.** Never modify earlier messages when configs change. Append, don't insert.

**3. No early variability.** Tool definitions, agent configs, and memory go in Layer 2 in deterministic order (sorted by tool ID, not relevance). Relevance-based pruning omits from the end, preserving the prefix.

**4. Cache warmth.** Anthropic's cache expires after 5 minutes (extendable to 1 hour). The gateway sends keepalive pings for agents with periodic activity (Aider's `--cache-keepalive-pings` pattern).

### Prompt assembly

```rust
// crates/bardo-pipeline/src/prompt.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptLayer {
    pub layer: String,          // "global" | "tenant" | "session"
    pub content: String,
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl { pub r#type: String } // "ephemeral"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledPrompt {
    pub messages: Vec<Message>,
    pub stable_prefix_tokens: usize,
}

pub fn assemble_prompt(
    request: &ChatCompletionRequest, agent_config: &AgentConfig,
    session_state: &SessionState,
) -> AssembledPrompt {
    let cc = || Some(CacheControl { r#type: "ephemeral".into() });
    let layers = vec![
        PromptLayer { layer: "global".into(),  // L1: static, all agents
            content: GLOBAL_SYSTEM_PREFIX.into(), cache_control: cc() },
        PromptLayer { layer: "tenant".into(),  // L2: per-agent, cached
            content: build_tenant_prefix(agent_config), cache_control: cc() },
        PromptLayer { layer: "session".into(), // L3: per-request, variable
            content: build_session_context(&request.messages, session_state),
            cache_control: None },
    ];
    let stable = format!("{}{}", layers[0].content, layers[1].content);
    AssembledPrompt {
        messages: flatten_layers(&layers),
        stable_prefix_tokens: estimate_tokens(&stable),
    }
}

/// Build tenant prefix in deterministic order for cache stability.
pub fn build_tenant_prefix(config: &AgentConfig) -> String {
    let mut parts = vec![
        format!("<agent_role>\n{}\n</agent_role>", config.system_prompt),
    ];
    // Tools sorted by stable ID (NOT by relevance -- that would vary per query)
    let mut tools = config.registered_tools.clone();
    tools.sort_by(|a, b| a.id.cmp(&b.id));
    parts.push(format!("<available_tools>\n{}\n</available_tools>", format_tool_defs(&tools)));
    if !config.persistent_memory.is_empty() {
        parts.push(format!("<agent_memory>\n{}\n</agent_memory>",
            format_memory(&config.persistent_memory)));
    }
    parts.join("\n\n")
}
```

## Dynamic Tool Registry

Tool definitions are the hidden context tax. Anthropic's testing showed 58 tools consume ~55K tokens in definitions alone [ANTHROPIC-TOOL-SEARCH]. The gateway maintains a searchable registry and assembles minimal definitions per-request by semantic relevance.

### Three-level tool architecture

| Level | Tools | Strategy | Token Impact |
|-------|-------|----------|-------------|
| **L1: Core** | ~5-15, stable | Always loaded in tenant prefix | Cached, ~0 marginal |
| **L2: Protocol** | ~50-200, dynamic | Semantic search per query | Loaded when relevant |
| **L3: Complex** | Scripts, not defs | No tool definition overhead | Agent's compute |

```rust
// crates/bardo-pipeline/src/tools.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistryEntry {
    pub tool_id: String,
    pub agent_id: u64,          // ERC-8004 agent ID
    pub level: u8,              // 1 = core, 2 = protocol, 3 = complex
    pub definition: ToolDefinition,
    pub description: String,
    pub embedding: Vec<f32>,    // Pre-computed for fast similarity search
    pub categories: Vec<String>,
    pub token_cost: u32,        // Tokens consumed by this tool definition
}
```

### Dynamic tool selection

```rust
// crates/bardo-pipeline/src/tools.rs

/// "95% reduction in per-step context tokens, 32% improvement in accuracy"
/// [ANTHROPIC-TOOL-SEARCH]
pub async fn select_tools_for_request(
    request: &ChatCompletionRequest, agent_id: u64,
    registry: &ToolRegistry, embedder: &LocalEmbedder,
    max_tools: usize, max_tokens: u32,    // defaults: 12, 8000
) -> Result<Vec<ToolDefinition>> {
    let core = registry.get_level1_tools(agent_id).await?;
    let query = request.messages.iter().rev()
        .find(|m| m.role == Role::User)
        .map(|m| m.content_as_str()).unwrap_or("");
    let candidates = registry.search(SearchQuery {
        agent_id, embedding: embedder.embed(query).await?,
        level: 2, limit: max_tools - core.len(),
    }).await?;

    // Fit within token budget (binary search like Aider's repo map)
    let mut selected = core;
    let mut used: u32 = selected.iter().map(|t| t.token_cost).sum();
    for tool in &candidates {
        if used + tool.token_cost > max_tokens || selected.len() >= max_tools { break; }
        selected.push(tool.clone());
        used += tool.token_cost;
    }
    Ok(selected.iter().map(|t| t.definition.clone()).collect())
}
```

### Speakeasy meta-tool pattern

For very large tool catalogs (100+), expose three meta-tools instead of pre-loading definitions. This achieves 97.5% token reduction:

```rust
// crates/bardo-pipeline/src/tools.rs

use serde_json::Value;

pub struct MetaTool {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

pub fn meta_tools() -> Vec<MetaTool> {
    vec![
        MetaTool {
            name: "search_tools",
            description: "Search available tools by capability description.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "What you want to do" },
                    "limit": { "type": "number", "description": "Max results", "default": 5 }
                },
                "required": ["query"]
            }),
        },
        MetaTool {
            name: "get_tool_schema",
            description: "Get the full parameter schema for a specific tool.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "toolName": { "type": "string" }
                },
                "required": ["toolName"]
            }),
        },
        MetaTool {
            name: "execute_tool",
            description: "Execute a tool with the given parameters.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "toolName": { "type": "string" },
                    "params": { "type": "object" }
                },
                "required": ["toolName", "params"]
            }),
        },
    ]
}
```

Bardo Sanctum already implements this with its `search_tools` meta-tool and deferred loading. The gateway extends the pattern to any agent.

## Context Budget Allocation

Different content types have different information density and different tolerance for compression. The gateway implements a differential budget system inspired by LLMLingua's Budget Controller [LLMLINGUA]:

```rust
// crates/bardo-pipeline/src/budget.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub total_token_budget: u32,
    pub allocations: BudgetAllocations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocations {
    pub system: BudgetSlot,         // 0-5% compression
    pub tools: ToolsBudget,         // 70-95% via selection
    pub memory: TopKBudget,         // 10-20% compression
    pub retrieval: TopKBudget,      // 50-80% compression
    pub defi_context: BudgetSlot,   // 60-80% compression
    pub history: HistoryBudget,     // 30-50% compression
    pub query: BudgetSlot,          // never compressed
}

// Reusable sub-structs (all derive Debug, Clone, Serialize, Deserialize)
pub struct BudgetSlot  { pub max_tokens: u32, pub compression_ratio: f64 }
pub struct ToolsBudget { pub max_tokens: u32, pub max_tools: u32 }
pub struct TopKBudget  { pub max_tokens: u32, pub top_k: u32 }
pub struct HistoryBudget {
    pub max_tokens: u32, pub recent_verbatim_turns: u32, pub older_summary_budget: u32,
}
```

### Default budget (128K context model)

```rust
// crates/bardo-pipeline/src/budget.rs

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            total_token_budget: 100_000, // leave 28K headroom for output
            allocations: BudgetAllocations {
                system:       BudgetSlot  { max_tokens: 4_000,  compression_ratio: 0.0 },
                tools:        ToolsBudget { max_tokens: 8_000,  max_tools: 12 },
                memory:       TopKBudget  { max_tokens: 3_000,  top_k: 10 },
                retrieval:    TopKBudget  { max_tokens: 15_000, top_k: 8 },
                defi_context: BudgetSlot  { max_tokens: 2_000,  compression_ratio: 0.0 },
                history: HistoryBudget {
                    max_tokens: 60_000, recent_verbatim_turns: 10, older_summary_budget: 2_000,
                },
                query: BudgetSlot { max_tokens: 8_000, compression_ratio: 0.0 },
            },
        }
    }
}
```

### History compaction

When conversation history exceeds the budget, the gateway applies lossy compaction using a cheap model (Haiku). Over 50% of Claude Code's LLM calls use Haiku for exactly this kind of summarization.

```rust
// crates/bardo-pipeline/src/budget.rs

pub async fn apply_history_budget(
    messages: &[Message], budget: &ContextBudget, summarizer: &impl Summarizer,
) -> Result<Vec<Message>> {
    let hist = &budget.allocations.history;
    let non_system: Vec<_> = messages.iter().filter(|m| m.role != Role::System).collect();
    let split = non_system.len().saturating_sub(hist.recent_verbatim_turns as usize * 2);
    let (older, recent) = non_system.split_at(split);

    if !older.is_empty() && estimate_tokens_slice(older) > hist.older_summary_budget {
        let summary = summarizer.summarize(older, hist.older_summary_budget).await?;
        let mut out = vec![Message::system(
            format!("<conversation_summary>\n{summary}\n</conversation_summary>")
        )];
        out.extend(recent.iter().map(|m| (*m).clone()));
        Ok(out)
    } else {
        Ok(non_system.into_iter().cloned().collect())
    }
}
```

### "Lost in the middle" mitigation

Constraints go at both START and END (Devin's dual-position pattern) so the model attends to safety rules even as context grows:

```rust
// crates/bardo-pipeline/src/prompt.rs

if let Some(ref constraints) = agent_config.critical_constraints {
    result.parts.push(PromptPart {
        part_type: PartType::ConstraintsReminder,
        content: format!("<critical_constraints>\n{constraints}\n</critical_constraints>"),
        tokens: estimate_tokens(constraints),
        position: Position::End, // Appended after user query
    });
}
```

## DeFi Context Enrichment

Queries classified as `domain: "defi"` get market context injected automatically, reducing tool-call round trips. Sources: DefiLlama (prices, ~50 tok/token), Uniswap subgraph (pools, ~100 tok/pool), Morpho API (rates, ~80 tok/protocol), on-chain reads (positions, ~150 tok/position), registry (params, ~30 tok/protocol).

### DeFi context configuration

```rust
// crates/bardo-pipeline/src/defi.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiContextConfig {
    pub prices: bool,               // inject current prices for mentioned tokens
    pub rates: bool,                // inject current lending rates
    pub pools: bool,                // inject pool stats (TVL, volume, fees)
    pub max_staleness: String,      // "1m" | "5m" | "15m" | "1h"
}
```

All data is cached at the gateway with per-source freshness guarantees. Enrichment format follows Anthropic's just-in-time pattern:

```xml
<defi_context freshness="2m">
  <price token="ETH" usd="3847.22" change24h="-2.1%" />
  <price token="USDC" usd="0.9998" />
  <pool protocol="uniswap-v3" pair="USDC-ETH" type="CL"
    tvl="$14.2M" volume24h="$8.7M" fees24h="$12,400" />
  <rate protocol="morpho" asset="USDC" supplyAPY="4.2%"
    borrowAPY="5.1%" utilization="82%" />
</defi_context>
```

## Cost impact

Three-layer prompt cache: 60-80% reduction on input tokens (prefix reuse). Dynamic tool selection: 70-95% on tool definitions. Context budget allocation: 30-50% on history (lossy compaction with cheap model). DeFi enrichment: indirect savings via fewer tool calls. Semantic response cache: 100% on hits (15-30% of requests). Combined: 40-85% cost reduction vs. naive prompting.

## Prompt Compression Service

For agents sending large context payloads, the gateway offers on-demand compression using LLMLingua-2 techniques. A local BERT-sized token classifier (~10ms) scores each token's importance and removes low-scoring tokens while preserving semantic coherence. Compression activates automatically when input tokens exceed 32K, the user's budget is running low, or the content is classified as compressible. The user pays for the compressed token count.

```rust
// crates/bardo-pipeline/src/compression.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub target_ratio: f64,      // 0.0-1.0, default 0.5
    pub preserve_recent: u32,   // recent messages to never compress
}
```

Request via the `bardo.optimize` extension:

```json
{
  "model": "auto",
  "messages": [...],
  "bardo": {
    "optimize": {
      "compress": true,
      "targetCompressionRatio": 0.5,
      "pruneTools": true
    }
  }
}
```

## Compaction-as-a-Service

Long-running agent sessions routinely exceed 100K tokens. The gateway offers two compaction strategies.

```rust
// crates/bardo-pipeline/src/compaction.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub strategy: String,            // "compact" | "handoff"
    pub trigger: CompactionTrigger,
    pub preserve_recent: u32,        // recent messages to keep verbatim
    pub summary_budget: u32,         // token budget for summary/briefing
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionTrigger {
    pub trigger_type: String,        // "utilization" | "token_count" | "manual"
    pub value: Option<f64>,          // threshold, default 0.80
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    pub tokens_before: u32,  pub tokens_after: u32,
    pub tokens_saved: u32,   pub compaction_count: u32,
}
```

**In-place compaction** (Claude Code pattern) triggers at 75-95% utilization. Haiku summarizes older messages, preserving recent turns verbatim. After 2-3 compactions, information loss compounds. The gateway warns via `X-Bardo-Compaction-Count`; agents should consider handoff after 2.

**Handoff** (Amp pattern) uses Sonnet to produce a structured briefing from the full thread, then creates a new session initialized with that briefing. The parent session is preserved.

```text
Handoff briefing structure:
  OBJECTIVE: What the agent was trying to accomplish
  CONTEXT: Key facts and decisions made so far
  STATE: Current execution state (positions, pending actions)
  CONSTRAINTS: Active constraints and safety rules
  NEXT_STEPS: What should happen next
```

Handoff avoids the quality cliff of repeated compaction by starting fresh with a curated context.

### Auto-compaction

```rust
// crates/bardo-pipeline/src/compaction.rs

pub async fn maybe_auto_compact(
    session_id: &str,
    utilization: f64,
    config: &Option<CompactionConfig>,
    compactor: &impl SessionCompactor,
) -> Result<Option<CompactionResult>> {
    let Some(ref c) = config else { return Ok(None) };
    if c.trigger.trigger_type != "utilization" { return Ok(None); }
    let threshold = c.trigger.value.unwrap_or(0.8);
    if utilization > threshold {
        Ok(Some(compactor.compact_session(session_id, c).await?))
    } else {
        Ok(None)
    }
}
```

`POST /v1/sessions/{sessionId}/compact` triggers compaction. `POST /v1/sessions/{sessionId}/handoff` triggers handoff.

### "Context anxiety" mitigation

Devin discovered that Claude proactively summarizes when it perceives it is near context limits, even when it isn't. The gateway always requests the maximum context window from the provider (1M tokens) regardless of actual usage, preventing the model's own compaction from interfering with managed compaction.

## Prompt Template Library

```rust
// crates/bardo-pipeline/src/templates.rs

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,                       // unique per agent
    pub version: String,                    // semver
    pub description: String,
    pub system_prompt: String,              // {{variable}} placeholders
    pub defaults: HashMap<String, String>,
    pub stable_params: Vec<String>,         // cache-stable (Layer 2 prefix)
    pub dynamic_params: Vec<String>,        // per-request (Layer 3)
    pub tool_ids: Vec<String>,              // auto-selected tools
    pub cache_strategy: String,             // "prefix-stable" | "full-dynamic"
}
```

Request a template via the `bardo.template` extension:

```json
{
  "model": "auto",
  "messages": [{ "role": "user", "content": "Analyze ETH/USDC pool on Base" }],
  "bardo": {
    "template": {
      "name": "defi-pool-analysis",
      "version": "1.2.0",
      "params": {
        "pool": "ETH/USDC",
        "chain": "base"
      }
    }
  }
}
```

The gateway loads the template, injects stable params into Layer 2 (cached), dynamic params into Layer 3 (per-request), and auto-selects the template's registered tools. This follows the SPEAR framework pattern: stable context is separated from dynamic context at the template level.

| Method | Path | Purpose |
|--------|------|---------|
| `PUT`  | `/v1/templates/{name}` | Register/update template |
| `GET`  | `/v1/templates/{name}` | Get template by name+version |

## Prompt Optimization Pipeline (L1-L6 detail)

Seven transparent optimizations in `optimize_prompt()`. The agent sees only the final response.

```rust
// crates/bardo-pipeline/src/optimize.rs

pub async fn optimize_prompt(
    request: ChatCompletionRequest, agent_config: &AgentConfig,
    session_state: &SessionState,
) -> Result<OptimizedPrompt> {
    let mut p = request;
    p = reorder_for_cache_alignment(p);                         // 1. Cache alignment
    if count_tools(&p) > 15 {
        p = prune_tools_by_relevance(p, agent_config, 12).await?; // 2. Tool pruning
    }
    if count_messages(&p) > 50 {
        p = compress_history(p, session_state.summary_budget).await?; // 3. History compression
    }
    p = deduplicate_content(p);                                 // 4. Deduplication
    p = reorder_by_relevance(p);                                // 5. Lost-in-middle reorder
    if count_messages(&p) > 20 {                                // 6. Dual-position constraints
        if let Some(ref c) = agent_config.critical_constraints {
            p = duplicate_constraints(p, c);
        }
    }
    p = format_for_provider(p, &session_state.provider);        // 7. Provider formatting
    Ok(p)
}
```

## RAG-as-a-Service

```rust
// crates/bardo-pipeline/src/rag.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub name: String,               // unique per agent
    pub embedding_model: String,    // "nomic-embed-text-v1.5" | "text-embedding-3-small"
    pub chunking: ChunkingConfig,
    pub retrieval: RetrievalConfig,
    pub visibility: String,         // "private" | "shared"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    pub strategy: String,           // "recursive" | "semantic" | "fixed"
    pub chunk_tokens: u32,          // default 512
    pub overlap: u32,               // default 50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    pub strategy: String,           // "vector" | "keyword" | "hybrid"
    pub hybrid_alpha: Option<f64>,  // 0=keyword, 1=vector
    pub rerank: Option<bool>,
}
```

Chunks are injected into the system message with XML structure (Anthropic's just-in-time pattern):

```xml
<retrieved_context>
  <context source="morpho-docs" relevance="0.94">
    Morpho Blue is a permissionless lending protocol...
  </context>
  <context source="uniswap-v4-hooks" relevance="0.89">
    V4 hooks implement beforeSwap and afterSwap callbacks...
  </context>
</retrieved_context>
```

Storage tiers: Free (10K vectors, 100 queries/day, no reranking, $0), Standard (500K, 10K/day, reranking, ~$5/month), Premium (10M, unlimited, reranking + GraphRAG, ~$25/month).

## Configuration

```bash
# Prompt cache
BARDO_INFERENCE_CACHE_KEEPALIVE=true
BARDO_INFERENCE_KEEPALIVE_INTERVAL=240  # seconds (< 5min Anthropic TTL)

# Tool registry
BARDO_INFERENCE_MAX_TOOLS=12
BARDO_INFERENCE_MAX_TOOL_TOKENS=8000
BARDO_INFERENCE_TOOL_SEARCH_LIMIT=20

# Context budget
BARDO_INFERENCE_CONTEXT_BUDGET=100000
BARDO_INFERENCE_RECENT_TURNS=10
BARDO_INFERENCE_SUMMARY_BUDGET=2000

# DeFi enrichment
BARDO_INFERENCE_DEFI_ENRICHMENT=true
BARDO_INFERENCE_ENRICHMENT_MAX_TOKENS=2000
```

## Golem-level context engineering: the Context Governor

Golems (mortal autonomous DeFi agents managed by the Bardo runtime) handle context engineering through the Context Governor, not the gateway-level mechanisms above. The gateway's context engineering applies to A2A calls and external agent interactions. The Context Governor operates inside the Golem-RS runtime and treats context assembly as a **learnable control problem**.

### The Cognitive Workspace

Traditional agent frameworks grow a session until it hits the context window limit, then compress. This is reactive and lossy. The Golem takes a different approach: the **Cognitive Workspace** is assembled fresh each tick from structured categories with learned token allocations. No growing session. No compaction. Each tick, the Context Governor builds the optimal context for the current situation from scratch.

This implements Alan Baddeley's (2000) working memory model:

| Baddeley component | Golem implementation | What it contains |
|---|---|---|
| Central executive | Context Governor | Allocates attention (tokens) across categories |
| Episodic buffer | The Workspace itself | Integrates information from all sources |
| Visuospatial sketchpad | Observations + Positions | Current situational awareness |
| Phonological loop | STRATEGY.md heuristics | Rehearsed procedural knowledge |

```rust
// crates/golem-context/src/workspace.rs

/// The Cognitive Workspace. Assembled fresh each decision cycle.
/// Token allocation across categories is learned by the cybernetics
/// self-tuning system and modulated by the rational inattention
/// budget (see 05-mortality.md).
pub struct CognitiveWorkspace {
    // Invariants (always present, never compressed)
    pub policy_cage: PolicyCageBlock,
    pub positions: Vec<PositionSnapshot>,
    pub active_warnings: Vec<Warning>,
    pub strategy: StrategyBlock,
    pub affect: AffectBlock,

    // Rehearsed knowledge (always present, budget-allocated)
    pub playbook_heuristics: Vec<PlaybookEntry>,
    pub mental_models: Vec<MentalModel>,

    // Retrieved knowledge (budget-allocated per ContextPolicy)
    pub retrieved_episodes: Vec<Episode>,
    pub retrieved_insights: Vec<GrimoireEntry>,
    pub causal_edges: Vec<CausalEdge>,
    pub contrarian_entries: Vec<GrimoireEntry>,
    pub dream_hypotheses: Vec<StagedRevision>,
    pub somatic_landscape_reading: Option<String>,

    // Current situation (refreshed each tick)
    pub observation: Observation,
    pub mortality: MortalityBlock,
    pub interventions: InterventionBlock,
    pub pheromone_summary: Option<String>,

    // Conversation (only when owner is interacting)
    pub conversation_tail: Option<Vec<Message>>,

    // Meta
    pub total_tokens: u32,
    pub policy_revision: u32,
}
```

### ACE cycle: +10.6% decision quality

The Context Governor implements the ACE (Agentic Context Engineering) cycle from Zhang et al. (2025): a Generator-Reflector-Curator refinement loop that achieves +10.6% on the AppWorld agent benchmark, matching IBM's top-ranked production GPT-4.1 agent using smaller open-source models [ZHANG-ACE-2025].

Three cybernetic feedback loops operate at different timescales:

**Loop 1: Per-tick outcome correlation.** After every tick with an outcome, correlate which Styx entries appeared in the workspace with whether the outcome was positive. EMA update: entries in winning contexts get +0.1; entries in losing contexts get -0.05.

**Loop 2: Per-curator policy evolution (every 50 ticks).** Aggregate Loop 1's correlations into category-level performance. Categories with consistently positive correlations get more tokens.

**Loop 3: Per-regime restructuring.** When the market regime changes, partially reset correlations (50% decay). Knowledge from one regime doesn't fully transfer to another.

### Predictive context assembly

Assembling the Cognitive Workspace takes 5-50ms: querying LanceDB for relevant episodes, scoring candidates, reading the Somatic Landscape. A background tokio fiber continuously maintains a pre-built workspace that updates reactively when state changes (PAD shift > 0.1, regime change). When deliberation fires, the context is already built. This eliminates 5-50ms of assembly latency during deliberation.

### Two layers, complementary

- **Gateway context** applies to all inference calls: prompt caching, token budgets, model routing. Reduces cost.
- **Golem context** operates inside the runtime via the Context Governor. Turn-phase-aware injection with seven academic techniques (ACE, retrieval-augmented prompting, phase-aware routing, outcome verification, state compaction, regime-conditional, affect-modulated). Improves decision quality.

## References

- [ANTHROPIC-CONTEXT-ENG] Anthropic (2025). "Building Effective Agents." Context engineering patterns from Claude Code, Cursor, Aider, Amp. Argues that context assembly is the primary bottleneck for production agents; the design patterns (stable prefixes, just-in-time retrieval, lossy compaction) directly inform Bardo's 8-layer pipeline.
- [ANTHROPIC-TOOL-SEARCH] Anthropic (2025). "Intelligent Tool Routing." arXiv:2602.17046. 95% reduction in per-step context tokens, 32% accuracy improvement. Demonstrates that providing only relevant tools per step (rather than the full registry) both reduces cost and improves decision quality; the basis for L4 dynamic tool pruning.
- [LLMLINGUA] Jiang, H. et al. (2024). "LLMLingua: Compressing Prompts for Accelerated Inference." Microsoft Research. Shows that token-level prompt compression can reduce input tokens by 2-20x with minimal quality loss; informs L5 history compression via cheap model.
- [AIDER-CACHE] Aider. "Prompt Caching." `--cache-keepalive-pings` pattern for maintaining warm provider caches. Documents the keepalive-ping technique for maintaining warm Anthropic prompt caches past the 5-minute TTL; directly adopted in L1 prefix alignment.
- [ZHANG-ACE-2025] Zhang, A. et al. "ACE: Agentic Context Engineering." arXiv:2510.04618, 2025. Proposes treating context assembly as a first-class optimization problem for agent systems; validates the approach of learned context budgets per task type.
- [SAMSUNG-CSO-2025] Samsung Research. "Context State Object Architecture." arXiv:2511.03728, 2025. Introduces a structured object model for managing context state across agent turns; informs the Cognitive Workspace design.
- [BADDELEY-2000] Baddeley, A. "The Episodic Buffer." Trends in Cognitive Sciences, 4(11), 2000. Proposes a working memory component that integrates information from multiple sources into coherent episodes; the neuroscience basis for the Context Governor's workspace assembly model.
- [COGNITIVE-WORKSPACE-2025] "Cognitive Workspace: Active Memory Management for LLMs." arXiv:2508.13171, 2025. Demonstrates that active memory management (selecting what to include in context per step) outperforms growing conversation buffers; validates Bardo's per-tick workspace assembly approach.
