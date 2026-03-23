# Multi-Model Orchestration: Bardo Inference as Internal Router

> **Document Type**: SPEC (normative) | **Version**: 3.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-14
>
> **Package**: `@bardo/inference` (`bardo-orchestrator`)
>
> **Depends on**: `prd2-bardo-inference-architecture.md`, `prd2-provider-feature-catalog.md`, `prd2-reasoning-chain-integration.md`
>
> **Purpose**: How Bardo Inference's backend router assigns each Pi subsystem to the optimal backend/model/feature combination. The three-axis Pareto optimization (cost × quality × privacy), mortality-aware routing, and concrete routing examples across different backend configurations.

---

> **Reader orientation:** This document specifies the multi-model orchestration layer of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes how the backend router assigns each Golem subsystem to the optimal backend/model/feature combination using three-axis Pareto optimization (cost, quality, privacy), with mortality-aware routing that shifts priorities as the agent approaches death. The key concept is that each subsystem (heartbeat, risk, dream, operator) has different quality, latency, and privacy requirements, and the router finds the best match across all configured backends. For term definitions, see `prd2/shared/glossary.md`.

## The Thesis

The Golem sends every inference request to Bardo Inference — one endpoint. Bardo Inference decides internally which backend handles it. The decision is invisible to the Golem: it just gets the best possible response. But the decision is sophisticated: it considers the subsystem's feature requirements, the Golem's mortality pressure, the security class, the available backends, and the real-time health of each backend.

This is **delegation, not configuration**. The Golem doesn't say "use Claude for risk." The Golem says "I need risk assessment with interleaved thinking." Bardo Inference routes to Claude because Claude provides interleaved thinking through BlockRun at the lowest cost with acceptable latency.

---

## Part 1: Routing Decision Flow

```
Golem sends request to Bardo Inference
    │
    ├─ 1. Context engineering pipeline (universal, all backends)
    │     Caching → compression → pruning → optimization
    │
    ├─ 2. Feature extraction
    │     What does this request need? Citations? Thinking? Privacy?
    │     (From bardo.subsystem hints or request analysis)
    │
    ├─ 3. Hard filters
    │     Security class → filter backends (private → Venice only)
    │     Required features → filter backends that support them
    │     Model specification → filter backends that have the model
    │
    ├─ 4. Soft scoring (Pareto optimization)
    │     Cost × Quality × Privacy × Latency × Feature match
    │     Weights shift with Golem mortality pressure
    │
    ├─ 5. Health check
    │     Skip unhealthy backends
    │
    └─ 6. Route to selected backend
         Apply provider-specific parameters
         Return response with backend metadata
```

---

## Part 2: Concrete Routing Examples

### 2.1 Minimal Configuration: BlockRun Only

Operator has: Bardo Inference key (BlockRun enabled by default)

```
heartbeat_t0 → BlockRun/nvidia-gpt-oss-120b (FREE)
heartbeat_t1 → BlockRun/gemini-3-flash ($0.50/M input)
heartbeat_t2 → BlockRun/claude-opus-4-6 ($5/M, adaptive thinking)
risk          → BlockRun/claude-opus-4-6 (interleaved thinking)
dream         → BlockRun/deepseek-r1 ($0.55/M, visible <think>)
daimon        → BlockRun/gemini-3-flash (cheapest, fast)
curator       → BlockRun/claude-sonnet-4-6 ($3/M, structured outputs)
context       → BlockRun/gemini-3-flash (cheap, fast)
playbook      → BlockRun/claude-sonnet-4-6 (full regeneration, no Predicted Outputs)
operator      → BlockRun/claude-opus-4-6 (best quality)
death         → BlockRun/deepseek-r1 (visible reasoning, maximum tokens)

Context engineering: All 8 layers active
Caching: Semantic + hash + Anthropic prompt caching (via BlockRun)
Est. daily cost: ~$2.50
```

### 2.2 BlockRun + OpenRouter Fallback

Operator has: Bardo Inference key + OpenRouter API key

```
Same as above, but:
- If BlockRun is degraded → automatic failover to OpenRouter
- Niche model requests → OpenRouter (400+ models)
- OpenRouter :nitro variants for operator-facing latency-sensitive requests
- OpenRouter :floor variants for batch/dream (cheapest)
- OpenRouter BYOK for operator's own provider keys

Additional capability: OpenRouter's unified reasoning_details format
simplifies reasoning parsing across providers
```

### 2.3 Full Stack: BlockRun + OpenRouter + Venice + Bankr + Direct Keys

```
heartbeat_t0 → Direct/local/qwen3-7b (FREE, zero latency)
heartbeat_t1 → BlockRun/gemini-3-flash (cheapest, cached)
heartbeat_t2 → BlockRun/claude-opus-4-6 (interleaved thinking, citations)
risk          → BlockRun/claude-opus-4-6 + Bankr cross-model verify
dream         → Venice/deepseek-r1 (visible, private, DIEM-funded)
daimon        → Venice/llama-3.3-70b (private, fast)
curator       → BlockRun/claude-sonnet-4-6 (citations for provenance)
context       → OpenRouter/qwen-plus (/think toggle, cheap)
playbook      → Direct/openai/gpt-5.4 (Predicted Outputs, 3x speed)
operator      → Bankr/claude-opus-4-6 (self-funded from trading revenue)
death         → Venice/deepseek-r1 (visible, private, DIEM, unlimited)
session       → BlockRun/claude-opus-4-6 (Compaction with DeFi instructions)
batch dreams  → Direct/anthropic/sonnet-4-6 (Batch API, 50% discount)

Context engineering: All 8 layers on every path
Caching: 3-layer stacked (semantic + hash + provider-specific)
Est. daily cost: ~$1.50 (DIEM covers Venice, self-funding offsets Bankr)
```

---

## Part 3: Cost Models

### 3.1 Estimated Daily Costs by Configuration

| Configuration | Est. Daily Cost | Notes |
|---|---|---|
| BlockRun only | ~$2.50 | Context engineering savings |
| BlockRun + OpenRouter | ~$2.30 | OpenRouter :floor for background tasks |
| BlockRun + Venice (DIEM staked) | ~$1.80 | Dreams/daimon via DIEM = free |
| Full stack (all backends) | ~$1.50 | Optimal routing per subsystem |
| Bankr self-sustaining | Net $0 | Revenue > cost |
| Venice DIEM-only (enough VVV staked) | ~$0.00 | All inference via DIEM |
| **Naive single-model (no Bardo Inference)** | **~$85** | **Every tick → Opus with all tools** |

### 3.2 Savings Breakdown

| Optimization Source | Savings | Where |
|---|---|---|
| T0 FSM routing (no LLM call) | ~80% of ticks eliminated | Bardo Inference tier routing |
| Semantic + hash cache | ~20% of remaining requests served from cache | Bardo Inference Layer 2-3 |
| Prompt cache alignment | 90% discount on cached prefix tokens | Bardo Inference Layer 1 + Anthropic |
| Tool pruning (meta-tool) | 97.5% reduction in tool tokens | Bardo Inference Layer 4 |
| Multi-model routing (cheap model per task) | 50-90% per request | Bardo Inference backend router |
| DIEM staking (Venice) | 100% on Venice-routed requests | Venice backend |
| Batch API (dreams) | 50% on batch requests | Direct Key backend |
| Self-funding (Bankr) | Revenue offsets cost | Bankr backend |

### 3.3 Bankr Sustainability Integration

When Bankr is a configured backend, Bardo Inference tracks the sustainability ratio and adjusts routing:

```typescript
// Bardo Inference receives sustainability metrics from Pi
// (via X-Bardo-Sustainability header or request body)
function adjustForSustainability(decision: RoutingDecision, ratio: number): RoutingDecision {
  if (ratio > 2.0) return decision; // Thriving — no change
  if (ratio > 1.0) return decision; // Self-sustaining — no change
  if (ratio > 0.5) {
    // Declining — prefer cheaper models
    return { ...decision, model: findCheaperAlternative(decision.model) };
  }
  // Dying — aggressive cost reduction on non-critical subsystems
  if (decision.subsystem !== "risk" && decision.subsystem !== "death") {
    return { ...decision, model: findCheapestModel(), reasoning: "none" };
  }
  return decision;
}
```

---

## Part 4: Provider Health & Failover

### 4.1 Health Monitoring

Bardo Inference monitors all configured backends with periodic health checks:

```typescript
interface BackendHealth {
  id: string;
  healthy: boolean;
  p50LatencyMs: number;
  p90LatencyMs: number;
  errorRate: number; // Last 5 minutes
  lastChecked: number;
}
```

### 4.2 Failover Chain

When the selected backend fails:
1. **Retry once** on the same backend (transient error)
2. **Failover** to the next-best backend that satisfies the request's requirements
3. **Degrade** if all backends are down — return cached response if available, or error

The failover is invisible to the Golem. It sees a slightly longer response time but never an error (unless all backends and caches fail).

### 4.3 OpenRouter as Universal Fallback

OpenRouter's built-in provider fallback makes it an excellent second-line defense. If BlockRun is the primary and goes down, Bardo Inference fails over to OpenRouter — which itself has multi-provider fallback internally. This creates **two layers of redundancy**:

```
Request → Bardo Inference → BlockRun (down!) → OpenRouter → Provider A (down!) → Provider B ✓
```

---

## Part 5: Non-Golem User Routing

For non-Golem users (Claude Code, Cursor, Aider), Bardo Inference provides a simplified routing experience:

- **No subsystem hints**: Bardo Inference infers the task type from the request content (code generation, conversation, analysis)
- **No mortality pressure**: Cost sensitivity is based on the user's pricing tier
- **Default routing**: BlockRun for most models, OpenRouter for niche models
- **Context engineering**: All 8 layers apply automatically

```bash
# Non-Golem user — just works
export ANTHROPIC_BASE_URL=https://inference.bardo.money/v1
export ANTHROPIC_API_KEY=bardo_sk_...

# Claude Code "just works" — Bardo Inference handles:
# - Prompt cache alignment for Claude's caching
# - Tool pruning for Claude Code's tool definitions
# - Backend routing (BlockRun primary, OpenRouter fallback)
# - Semantic caching for repeated queries
```

---

## References

- [ROUTELLM-2024] Ong, I. et al. "RouteLLM." arXiv:2406.18665, 2024.
- [ANTHROPIC-CONTEXT-ENG-2025] Anthropic. "Context Engineering." 2025.
- [OPENROUTER-ROUTING-2026] OpenRouter. "Provider Routing." Docs.
- [BLOCKRUN-CLAWROUTER-2026] BlockRun. "ClawRouter." GitHub.
- [BANKR-LLM-2026] Bankr. "LLM Gateway." Docs.
- [VENICE-PRIVACY-2025] Venice.ai. Privacy Architecture.
