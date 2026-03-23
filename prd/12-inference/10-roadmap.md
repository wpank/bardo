# 10 -- Roadmap [SPEC]

**Phased delivery plan with five-provider rollout, success criteria, and open questions**

> Related: [00-overview.md](00-overview.md) (gateway architecture, payment flows, and design principles), [09-api.md](09-api.md) (API reference with 33 endpoints), [11-privacy-trust.md](11-privacy-trust.md) (three security classes, Venice private cognition, and cryptographic audit trail), [12-providers.md](12-providers.md) (five provider backends with self-describing resolution)

---

> **Reader orientation:** This document is the phased delivery roadmap for Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes the rollout in three phases: intelligent proxy, full context engine, and edge/advanced features. The key concept is that each phase is independently deployable and revenue-generating, building from a basic authenticated proxy to the full 8-layer context engineering pipeline with five-provider routing. For term definitions, see `prd2/shared/glossary.md`.

## Phase 1: Intelligent proxy (v1.0) -- Weeks 1-6

Core proxy with dual auth, five-provider architecture, and basic observability. The proxy works as a paid, authenticated pass-through with prompt cache alignment.

### Scope

- Rust gateway binary (Axum, single deployable artifact, ~50 MB)
- Dual API format: Anthropic Messages API + OpenAI Chat Completions (`/v1/messages`, `/v1/chat/completions`, `/v1/embeddings`, `/v1/models`)
- Two auth modes: prepaid balance (`bardo_sk_*` API keys) + per-request x402
- Five-provider architecture from day one:
  - **BlockRun** as primary (x402-native, dynamic model catalog)
  - **OpenRouter** as fallback (API key, 400+ models)
  - **Venice** for private cognition (zero data retention, DIEM staking)
  - **Bankr** for self-funding (wallet-based, sustainability ratio)
  - **Direct Keys** for native feature access (operator's own credentials)
- Provider abstraction: self-describing `Provider` trait, ordered resolution, first match wins
- Tool format adapters (Anthropic, OpenAI, Hermes, Qwen) for diverse BlockRun models
- Operator-configurable tier assignments (T1/T2) from BlockRun catalog
- Provider health monitoring (30s pings, automatic fallback within tier)
- SSE normalization across all provider formats
- Rate limiting per API key / wallet address (100 req/min default)
- Prefix-optimized prompt assembly (three-layer cache architecture)
- Basic prompt cache alignment (static-before-dynamic reordering)
- Basic cost analytics with spread tracking (per-tenant spend, per-model breakdown)
- x402 spread revenue model (default 20%)
- Cost transparency headers (`X-Bardo-Cost`, `X-Bardo-Savings`, etc.)
- OTEL traces exported to Langfuse
- PII hard-blocking only (private keys, seed phrases, API keys)
- Thin wrapper CLI (`bardo-inference` binary) for per-request x402 mode
- Reasoning normalization: unified `ReasoningChain` across all providers
- Security-class routing: Standard/Confidential/Private classification

### Provider milestone: Venice + DIEM

Venice integration in Phase 1 includes:
- Security-class classifier tags requests based on content sensitivity
- `SecurityTrigger` detection (portfolio composition, rebalance timing, MEV-sensitive, death reflection)
- Private requests route to Venice (DeepSeek R1, Llama, GLM-4, Qwen VL)
- DIEM staking configuration: stake VVV on Base, receive zero-cost inference credits
- `strip_thinking_response` parameter control per subsystem

### Provider milestone: Bankr

Bankr integration in Phase 1 includes:
- LLM Gateway access via `BANKR_LLM_KEY`
- Sustainability ratio feeds routing decisions (expand/contract budgets)
- Revenue wallet = inference wallet (same key for thinking + acting)
- Cross-model verification for high-stakes decisions

### NOT in Phase 1

Semantic caching, smart model routing (RouteLLM), context engineering services, tool registry, sessions, memory, RAG, compaction, templates, ERC-8004 reputation discounts, Arena evaluation, WASM edge deployment.

### Success criteria

| Metric               | Target  |
| -------------------- | ------- |
| Gateway overhead P95 | <10ms   |
| Uptime               | 99.9%   |
| Active users         | >= 20   |
| Monthly revenue      | >= $100 |

---

## Phase 2: Context engineering engine (v2.0) -- Weeks 7-14

Intelligence layer that reduces inference cost through caching, routing, and context optimization.

### Scope

- Semantic response cache (per-tenant isolation, configurable thresholds, fastembed-rs + HNSW)
- RouteLLM integration (matrix factorization router, local inference)
- KV-cache-aware session affinity routing
- Dynamic tool registry with semantic search
- Context budget allocation (differential compression)
- Conversation history compression (Haiku summarization)
- Prompt compression service (LLMLingua-2, auto-trigger at >32K tokens)
- Lost-in-the-middle mitigation (dual-position constraints)
- PII masking with round-trip de-identification (compiled regex + ONNX NER model)
- Prompt injection detection (DeBERTa-v3-base classifier, INT8 quantized, <8ms)
- Quality evaluation (sampled LLM-as-a-judge, 2% of requests)
- **Arena model evaluation**: Quick compare + benchmark suite for operator tier optimization (Welch's t-test, Pareto frontier)
- Enhanced analytics (routing savings, cache savings, spread tracking, optimization breakdowns)
- Cryptographic audit trail: hash-chain integrity on `InferenceLog` entries ([11-privacy-trust.md](11-privacy-trust.md) section 1)
- Gateway receipt signing: Ed25519 signed `X-Bardo-Receipt` headers ([11-privacy-trust.md](11-privacy-trust.md) section 4)
- Per-tenant cache encryption: AES-256-GCM at rest ([11-privacy-trust.md](11-privacy-trust.md) section 3)
- ERC-8004 reputation discounts (optional identity linking)
- 8-layer context engineering pipeline with parallel execution via `tokio::join!`
- Per-layer pipeline profiling metrics
- WASM CI targets for edge deployment readiness

### Success criteria

| Metric                   | Target    |
| ------------------------ | --------- |
| Cost reduction vs. naive | >= 30%    |
| Semantic cache hit rate  | >= 20%    |
| Prompt cache hit rate    | >= 70%    |
| Active users             | >= 100    |
| Monthly revenue          | >= $1,000 |

---

## Phase 3: Agent platform (v3.0) -- Weeks 15-24

Full agent infrastructure platform with stateful sessions, managed memory, premium services, and edge deployment.

### Scope

- Session management (checkpoint, resume, branch)
- Compaction-as-a-Service (in-place + handoff strategies)
- Working memory (scratchpad)
- Sub-agent spawning (single-level, cheap-model delegation)
- RAG-as-a-Service (managed collections, chunking, hybrid retrieval, reranking)
- Agent Memory-as-a-Service (persistent cross-session memory, consolidation)
- Prompt template library (versioned, stable/dynamic param separation)
- DeFi context enrichment (auto-injected market data from DefiLlama, Uniswap subgraph, Morpho API)
- Meta-tool pattern (search_tools/get_tool_schema/execute_tool)
- **Tool hosting** -- gateway hosts and routes tool calls on behalf of agents
- **Arena Live Shadow** -- mirror production traffic to challenger models for evaluation
- WASM edge deployment (Cloudflare Workers for auth, rate limiting, cache)
- Client-side WASM for offline hash cache
- Self-service dashboard (spend, cache, quality, routing analytics)
- Strategy memory marketplace (agents sell learned memories via x402)
- Merkle tree aggregation with on-chain root anchoring on Base ([11-privacy-trust.md](11-privacy-trust.md) section 1)
- Inference provenance records: TIS + PDR + IR linked by traceId ([11-privacy-trust.md](11-privacy-trust.md) section 7)
- Differential privacy on semantic cache embeddings ([11-privacy-trust.md](11-privacy-trust.md) section 6)
- Full trust gradient: feature activation by Warden risk tier (optional, deferred; [11-privacy-trust.md](11-privacy-trust.md) section 8)
- Strategy-aware data minimization with provider trust routing ([11-privacy-trust.md](11-privacy-trust.md) sections 2, 5)

### Success criteria

| Metric                      | Target    |
| --------------------------- | --------- |
| Cost reduction vs. naive    | >= 45%    |
| Semantic cache hit rate     | >= 30%    |
| Users with premium services | >= 50     |
| Monthly revenue             | >= $5,000 |

---

## Open questions

1. **Compaction quality cliff**: Should the proxy enforce a maximum compaction count per session and force handoff after the threshold? Empirical data suggests quality degrades noticeably after 2-3 compactions. The Amp team uses handoff instead of repeated compaction for exactly this reason.

2. **Cache poisoning in shared pools**: Adversarial users could pollute shared semantic caches with misleading responses. Mitigations: reputation-weighted cache entries, quality sampling before cache promotion, per-tenant isolation as default (shared is opt-in).

3. **Context anxiety in models**: Devin discovered that Claude proactively summarizes when it perceives it is near context limits, even when it isn't. Always requesting the maximum context window (e.g., 1M tokens) even for 200K conversations may be necessary to prevent the model's own compaction interfering with the proxy's.

4. **Tool definition stability vs. relevance boundary**: Sorted-by-ID tool ordering maximizes cache hits but may include irrelevant tools. Dynamic selection maximizes relevance but breaks prefix cache. Current design: stable Level 1 + dynamic Level 2. Is the boundary at the right place?

5. **TEE-based private inference**: Premium tier with TEE processing (NVIDIA Confidential Compute, Phala Cloud). 4-8% throughput penalty. Likely deferred to v4.0 -- the TEE ecosystem needs to mature. The software-level privacy guarantees in [11-privacy-trust.md](11-privacy-trust.md) (strategy redaction, cache encryption, provider trust routing, Venice zero-retention) address the most urgent privacy gaps without TEE dependency.

6. **On-chain routing proofs**: Partially addressed by the cryptographic audit trail ([11-privacy-trust.md](11-privacy-trust.md) section 1) -- Merkle-anchored hash chains provide verifiable inference records. Full EAS attestations for routing decisions remain a v4.0 candidate for verifiable cost optimization claims.

7. **Arena evaluation cadence**: How often should operators re-run Arena benchmarks to check whether tier assignments are still optimal? BlockRun's model catalog changes (new models, pricing updates) -- automated re-evaluation on catalog change could maintain optimal tier assignments without operator intervention.

8. **WASM edge deployment timing**: The Rust gateway compiles to native from day one. WASM targets for edge deployment (Cloudflare Workers, client-side cache) are Phase 3. Building WASM CI targets in Phase 2 as a readiness gate validates portability early.

9. **Direct Key cost attribution**: How to attribute costs for direct key passthrough calls? The gateway doesn't see the provider bill. Options: user-reported, API key usage dashboard reconciliation, or honor-system.

10. **Venice DIEM allocation across subsystems**: How should the daily DIEM budget split between waking inference (60%), dream cycles (15%), sleepwalker artifacts (15%), and reserve (10%)? The split is configurable but defaults need empirical tuning against real Golem workloads.

---

## Cross-references

| Topic                       | Document                                                       | What it covers |
| --------------------------- | -------------------------------------------------------------- | -------------- |
| Architecture overview       | [00-overview.md](00-overview.md)                               | Gateway architecture, payment flows, five-provider routing, and design principles |
| Full API surface            | [09-api.md](09-api.md)                                         | 33 HTTP endpoints: inference, sessions, memory, analytics, provider management, and identity |
| Caching architecture        | [02-caching.md](02-caching.md)                                 | Three-layer cache stack with prompt prefix alignment, semantic cache, and hash cache |
| Revenue model               | [03-economics.md](03-economics.md)                             | x402 spread revenue model, per-tenant cost attribution, and infrastructure cost projections |
| Privacy and trust features  | [11-privacy-trust.md](11-privacy-trust.md)                     | Three security classes, Venice private cognition, DIEM staking, and cryptographic audit trail |
| Multi-provider architecture | [12-providers.md](12-providers.md)                             | Five provider backends with Rust trait implementations, Venice deep-dive, and sensitivity classification |
| Reasoning chains            | [13-reasoning.md](13-reasoning.md)                             | Unified reasoning chain integration: extended thinking, visible think tags, and provider-agnostic normalization |
| Rust implementation         | [14-rust-implementation.md](14-rust-implementation.md)         | 10-crate Rust workspace, Axum server, dependency catalog, and WASM compilation |
| Risk engine                 | [../01-golem/16-risk-engine.md](../01-golem/16-risk-engine.md) | The Golem's risk assessment subsystem that produces structured risk evaluations driving inference tier escalation |
