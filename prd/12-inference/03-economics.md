# 03 -- Inference Economics [SPEC]

**x402 Spread Revenue Model, Per-Tenant Cost Attribution, and Infrastructure Costs**

> Related: [00-overview.md](00-overview.md) (gateway architecture, payment flows, and design principles), [02-caching.md](02-caching.md) (three-layer cache stack that drives 40-85% cost savings), [prd2/11-compute/03-billing.md](../11-compute/03-billing.md) (Compute VM billing and inference token allowances)

---

> **Reader orientation:** This document specifies the economics of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and covers the x402 spread revenue model, per-tenant cost attribution, and infrastructure cost projections. The key concept is that context engineering makes the all-in cost cheaper than direct API access even after the operator's spread, creating a positive-sum model where users save money and operators earn margin. For term definitions, see `prd2/shared/glossary.md`.

## Revenue model: x402 spread

See `shared/x402-protocol.md` for the x402 payment protocol specification.

**Revenue = x402 (a micropayment protocol for HTTP-native USDC payments on Base) spread.** Bardo charges a configurable markup (default ~20%) over BlockRun's cost. Context engineering makes the all-in cost cheaper than direct API access despite the spread. The user saves money, the operator earns margin.

### The only revenue stream

The operator configures a spread percentage applied to the optimized cost (after context engineering), not the naive cost.

```
BlockRun cost (optimized):  $X
Spread (default 20%):       $X × 0.20
User pays:                  $X × 1.20

Operator margin per request: $X × 0.20
```

### Worked example

```
Request: 50K input tokens + 2K output tokens (Sonnet-equivalent)

Naive cost (direct API):                    $0.165
  Input:  50K × $3.00/M  = $0.150
  Output:  2K × $15.00/M = $0.030
  Total:                   $0.180
  (BlockRun may be slightly cheaper: $0.165)

Context engineering savings (~40%):
  - 30K tokens hit prefix cache (90% discount): saves $0.040
  - Tool pruning removes 8K tokens:             saves $0.024
  - History compression saves 2K tokens:         saves $0.006
  Optimized cost:                               $0.099

User pays (20% spread):                        $0.119
  BlockRun cost:  $0.099
  Spread:         $0.020

User saves vs. direct API:  28% ($0.165 → $0.119)
Operator margin:            $0.020/request
```

### Key properties

- **User always saves**: Context engineering savings (~40%) > spread (20%)
- **Zero float**: Both legs settle instantly via x402 on Base. No credit risk.
- **Zero inference cost**: No prepaid API credits to manage (BlockRun is x402-native)
- **Spread is operator-configurable**: Range 5-50%, default 20%
- **One revenue stream**: Simple pricing, simple explanation to users

## Cost savings stack

Seven optimization layers compound to produce 40-85% cost reduction:

| Layer | Mechanism | Savings | How |
|---|---|---|---|
| T0 routing | FSM suppresses ~80% of ticks | 80% of ticks at $0 | No LLM call for stable markets |
| Semantic + hash cache | Zero-cost on cache hits | ~20% of remaining at $0 | Embedding similarity + exact match |
| Prompt cache alignment | 90% discount on cached prefix | 60-80% on input tokens | Stable-before-dynamic reordering |
| Tool pruning | Remove irrelevant tool definitions | 97.5% of tool tokens | Semantic search, ≤12 tools per request |
| Multi-model routing | Match task to cheapest capable model | 50-90% per request | RouteLLM classifier, T1/T2 split |
| DIEM staking | 100% discount on Venice-routed calls | Free inference | Venice DIEM balance covers cost |
| Batch API | 50% discount on async processing | 50% on dream cycles | Anthropic/OpenAI batch endpoints |

### Semantic cache tuning

Cache TTLs are regime-aware -- volatile markets invalidate cached responses faster:

| Market Regime | TTL (seconds) |
|---|---|
| Calm | 300 |
| Normal | 210 |
| Volatile | 90 |
| Crisis | 30 |

Similarity thresholds vary by domain to prevent stale data leaking into high-stakes decisions:

| Domain | Cosine Threshold |
|---|---|
| General market analysis | 0.92 |
| Strategy reasoning | 0.95 |
| Trade execution | 0.98 |

Higher thresholds in execution contexts mean fewer cache hits but zero risk of acting on semantically similar but materially different market conditions.

---

## Infrastructure cost

The operator's only fixed cost is hosting the proxy binary.

| Scale                    | Infrastructure                | Monthly Cost |
| ------------------------ | ----------------------------- | ------------ |
| Small (< 1K req/day)     | Single VPS                    | $50          |
| Medium (1K-10K req/day)  | 2-3 instances + load balancer | $200         |
| Large (10K-100K req/day) | Kubernetes cluster            | $500         |

Default stack: in-memory LRU cache (configurable size, default 100MB), embedded HNSW index for semantic vectors, SQLite for sessions/memory/analytics. For large-scale deployments, optional backends (Redis, Qdrant, Clickhouse) can be swapped in for horizontal scalability.

---

## ERC-8004 reputation discounts (future)

Users with high ERC-8004 reputation scores get reduced spread:

| Reputation Tier   | Spread |
| ----------------- | ------ |
| None (default)    | 20%    |
| Basic (50+)       | 18%    |
| Verified (200+)   | 15%    |
| Trusted (500+)    | 12%    |
| Sovereign (1000+) | 8%     |

This creates a flywheel: use Bardo -> build reputation -> get cheaper access -> use Bardo more. Linking is optional via `POST /v1/identity/link`.

---

## Per-tenant cost tracking

Every user (API key or wallet address) gets per-request and aggregate cost tracking. Every response includes cost transparency headers:

```
X-Bardo-Cost: 0.119            # What the user paid
X-Bardo-Blockrun-Cost: 0.099   # What Bardo paid BlockRun
X-Bardo-Spread: 0.020          # Operator margin
X-Bardo-Naive-Cost: 0.165      # What it would have cost without optimization
X-Bardo-Savings: 0.046         # User savings vs. naive (naive - user paid)
X-Bardo-Balance: 12.38         # Remaining prepaid balance (prepaid mode only)
```

### Per-tenant cost summary

```rust
// crates/bardo-telemetry/src/cost.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCostSummary {
    pub tenant_id: String,
    pub period: CostPeriod,
    pub total_requests: u64,
    /// What the user paid (including spread).
    pub total_paid: f64,
    /// What Bardo paid BlockRun.
    pub total_blockrun_cost: f64,
    /// Operator margin.
    pub total_spread: f64,
    /// Estimated cost without context engineering.
    pub estimated_naive_cost: f64,
    /// Naive minus user-paid.
    pub total_savings: f64,
    pub savings_breakdown: SavingsBreakdown,
    pub by_model: Vec<ModelCostBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostPeriod {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsBreakdown {
    pub semantic_cache: f64,
    pub prefix_cache: f64,
    pub tool_pruning: f64,
    pub history_compression: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostBreakdown {
    pub model: String,
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub user_paid: f64,
    pub blockrun_cost: f64,
    pub savings: f64,
}
```

Queryable via `GET /v1/analytics/spend?period=day|week|month`.

---

## Fallback cost tracking

When the proxy falls back from BlockRun (x402) to OpenRouter (API key), the x402 payment path is bypassed for the upstream leg. Cost tracking still applies: estimates use published per-token pricing from the provider catalog, and daily reconciliation against the API-key provider's usage dashboard catches drift (typically <5%).

See [01a-routing.md](01a-routing.md) for the `FallbackCostAdapter`.

---

## Moat: agents that pay

The financial security architecture is a competitive moat (see `prd2/appendices/moat-pay.md`). Six independent security layers, three cryptographic, mean that even a fully compromised LLM cannot drain funds. The inference gateway is the economic control plane -- it tracks costs, enforces budgets, and ensures the sustainability ratio stays above 1.0.

---

## Bankr self-funding economics

Bankr-routed Golems (mortal autonomous DeFi agents managed by the Bardo runtime) fund inference from revenue. The wallet that earns from DeFi strategies pays for inference directly. No separate funding step. This is the metabolic loop: the organism sustains itself through the activity that requires sustenance.

```rust
// crates/bardo-providers/src/bankr.rs

pub fn compute_sustainability_ratio(
    daily_revenue_usd: f64,
    daily_inference_cost_usd: f64,
    daily_compute_cost_usd: f64,
    daily_gas_cost_usd: f64,
) -> f64 {
    let total_daily_cost = daily_inference_cost_usd
        + daily_compute_cost_usd
        + daily_gas_cost_usd;
    daily_revenue_usd / total_daily_cost
}
// ratio > 1.0 -> self-sustaining
// ratio > 2.0 -> thriving (can spawn replicants)
// ratio < 1.0 -> declining
// ratio < 0.5 -> dying
```

The sustainability ratio feeds directly into routing decisions. When it drops, the gateway routes more aggressively to cheap tiers:

| Sustainability ratio | Budget adjustment | Behavior |
|---|---|---|
| > 2.0 | Expand 1.5x | Revenue exceeds cost, grow capabilities |
| 1.0 - 2.0 | Baseline | Self-sustaining |
| 0.5 - 1.0 | Contract 0.7x | Under pressure, reduce non-critical inference |
| < 0.5 | Contract 0.3x | Near-death, only risk + owner + death funded |

A Golem managing a vault earning $50/day in fees with $15/day in Bankr inference costs has a 3.3x sustainability ratio. Above 1.0x, the Golem is economically immortal (barring epistemic or stochastic death). Below 1.0x, the economic clock is ticking. The gateway's cost profile naturally curves downward as the Golem approaches death, extending effective lifespan by reducing the largest variable cost.

---

## Venice DIEM staking economics

Venice DIEM staking provides zero-cost inference for Venice-routed calls. The owner stakes VVV tokens (Venice's native token on Base), receiving a daily DIEM allocation proportional to stake weight. Each DIEM equals $1/day of Venice API credit, perpetually. DIEM-funded calls show `X-Bardo-Cost: 0.00` in response headers.

DIEM is the first-choice cost optimization for privacy-preferring subsystems (dreams, daimon, death reflection). When DIEM balance is available, the provider resolver routes privacy-preferring intents to Venice automatically.

### DIEM allocation strategy

```text
Daily DIEM budget: $X (from VVV stake)
+-- Waking inference (private):  60%  -- Portfolio analysis, deal negotiation
+-- Dream cycles (private):      15%  -- Counterfactual reasoning, threat simulation
+-- Sleepwalker artifacts:       15%  -- Observatory research
+-- Reserve (rollover):          10%  -- Unused DIEM for volatile days
```

### Mortality integration

In the standard mortality model, inference costs drain the LLM credit partition (60% of total budget). Venice staking **decouples inference cost from mortality pressure**: Venice (DIEM) calls cost zero marginal cost from staked VVV, draining nothing from the LLM partition. A Golem routing 50% of its inference to Venice extends its projected lifespan by ~30% (saving $0.06-0.10/day on a $0.20/day budget). Over a 30-day lifespan, that is 9 additional days of life.

---

## Future revenue

### Phase 2: premium services

| Service                   | Price  | Description                           |
| ------------------------- | ------ | ------------------------------------- |
| Custom prompt libraries   | $10/mo | Operator-curated prompt templates     |
| Advanced semantic caching | $5/mo  | Larger cache, cross-session sharing   |
| Extended analytics        | $5/mo  | 90-day retention, detailed breakdowns |

### Phase 3: additional providers

Adding OpenRouter or direct API keys as fallback providers. These break the "zero working capital" property but increase model availability and redundancy. Revenue model extends naturally: spread applies to all providers. See [02-caching.md](02-caching.md) for provider cache discount tables used by the cost estimation engine.

---

## Daily cost projections (Golem agents)

For Golem agents using the three-tier system at 100 ticks/day:

| Scenario                                 | T0    | T1     | T2    | Daily LLM Cost |
| ---------------------------------------- | ----- | ------ | ----- | -------------- |
| Calm market (90% T0, 8% T1, 2% T2)       | $0.00 | $0.024 | $0.02 | ~$0.05         |
| Normal market (80% T0, 15% T1, 5% T2)    | $0.00 | $0.045 | $0.15 | ~$0.20         |
| Volatile market (60% T0, 25% T1, 15% T2) | $0.00 | $0.075 | $0.75 | ~$0.83         |

### Cost savings example (all users)

For a user making 1,000 requests/day:

| Optimization            | Applies To     | Savings    | Daily Savings                 |
| ----------------------- | -------------- | ---------- | ----------------------------- |
| Economy model routing   | 60% of queries | 85%/query  | $2.55                         |
| Semantic cache hits     | 20% of queries | 100%/query | $1.00                         |
| Prompt prefix cache     | 80% of queries | 10%/query  | $0.40                         |
| Tool pruning            | 30% of queries | 15%/query  | $0.22                         |
| **Total daily savings** |                |            | **$4.17**                     |
| **Without gateway**     |                |            | **$5.00/day**                 |
| **With gateway**        |                |            | **$1.33/day** (73% reduction) |

User saves ~$3.67/day. Gateway earns ~$0.50/day in spread.

---

## Revenue projections

| Phase              | Users  | Avg Req/Day/User | Daily Volume  | Daily Revenue (20% spread) |
| ------------------ | ------ | ---------------- | ------------- | -------------------------- |
| Launch             | 50     | 20               | 1,000 req     | ~$20                       |
| Growth             | 500    | 30               | 15,000 req    | ~$300                      |
| Product-Market Fit | 5,000  | 40               | 200,000 req   | ~$4,000                    |
| Scale              | 50,000 | 50               | 2,500,000 req | ~$50,000                   |

Assumptions: average request cost (post-optimization) ~$0.10, 20% spread = $0.02/request operator margin, context engineering provides ~40% savings vs. naive.

### Break-even

At $50/month infrastructure: break-even at ~2,500 requests/month (~84 requests/day). At 20 req/day average: break-even at 5 users.

---

## x402-native provider advantage

BlockRun is the primary upstream provider because it shares Bardo's x402 payment model. USDC flows directly User -> Bardo -> BlockRun with no fiat bridge, no API keys, and no prepaid credit float.

| Path                          | User Pays | Bardo Margin | Float Required | Notes                   |
| ----------------------------- | --------- | ------------ | -------------- | ----------------------- |
| Bardo -> BlockRun (x402)      | USDC      | Spread (20%) | $0             | Primary, zero capital   |
| Bardo -> OpenRouter (API key) | USDC      | Spread (20%) | ~$100          | Fallback, needs API key |

BlockRun handles provider relationships (Anthropic, OpenAI, Google, OSS). The operator never manages individual provider API keys for the primary path. OpenRouter serves as a fallback for models not available through BlockRun.

Working capital at launch is $0 (BlockRun-only). With OpenRouter fallback enabled: ~$100 prepaid credit float.

---

## Dependencies and costs

| Dependency            | Monthly Cost | Float | Notes                            |
| --------------------- | ------------ | ----- | -------------------------------- |
| BlockRun              | Pay-per-use  | $0    | x402-native, primary provider    |
| OpenRouter            | Pay-per-use  | ~$100 | Fallback, optional               |
| Hosting (VPS/Fly.io)  | $50-500/mo   | --    | Single VPS to multi-instance     |
| nomic-embed-text-v1.5 | $0           | --    | Local ONNX, runs on proxy CPU    |
| RouteLLM              | $0           | --    | Local, runs on proxy CPU         |
| DeBERTa-v3-base       | $0           | --    | Local, runs on proxy CPU         |
| Presidio              | $0           | --    | Open source, runs on proxy       |
| Langfuse              | $0           | --    | Self-hosted, open source         |
| Redis (optional)      | $10-50/mo    | --    | For scale: shared cache/sessions |
| Qdrant (optional)     | $0-30/mo     | --    | For scale: RAG vectors >1M       |
| Clickhouse (optional) | $0-20/mo     | --    | For scale: high-volume analytics |

Total operational cost tracks the infrastructure table above: $50-500/mo depending on scale, plus $0-100 float if OpenRouter fallback is enabled.

### Cost per request (Bardo overhead)

| Component                 | Cost   | Latency     |
| ------------------------- | ------ | ----------- |
| Auth check                | $0     | < 1ms       |
| Hash cache lookup         | $0     | < 1ms       |
| Semantic cache lookup     | $0     | 5-20ms      |
| Complexity classification | $0     | 10-30ms     |
| Prompt optimization       | $0     | < 50ms      |
| PII detection             | $0     | < 5ms       |
| x402 payment processing   | $0     | < 10ms      |
| **Total Bardo overhead**  | **$0** | **< 100ms** |

All compute runs on the proxy. No per-request infrastructure costs beyond BlockRun.

### Pipeline profiles

Not every request needs the full pipeline. Four profiles trade latency for optimization depth:

| Profile | Layers | Latency (Rust) | Use Case |
|---|---|---|---|
| `minimal` | L3 only (hash cache) | <1ms | T0 heartbeat ticks, simple lookups |
| `fast` | L1+L3+L6 | 5-15ms | Golem internal calls, low-stakes queries |
| `standard` | L1-L6 | 20-40ms | User-facing requests, standard optimization |
| `full` | L1-L8 | 30-65ms | High-security, first-time requests, unknown sources |

---

## Per-layer cost impact analysis

The naive architecture -- give the agent all tool definitions and let it call Opus every tick -- costs roughly $85/day at 100 ticks/day. The context engineering pipeline produces an **18-200x reduction** depending on market conditions.

### Layer-by-layer savings breakdown

| Layer | Savings | Applies To | Condition |
|---|---|---|---|
| Prompt cache alignment | 90% on cached prefix | All Anthropic-backed requests | Stable prefix exists |
| Semantic response cache | 100% on cache hit | All requests | Cosine > threshold (domain-specific) |
| Deterministic hash cache | 100% on exact match | All requests | Identical request |
| Tool pruning | ~97.5% on tool tokens | Requests with tools | Meta-tool pattern enabled |
| History compression | Variable (extends context) | Long conversations | Context > threshold |
| Position optimization | Quality improvement (not cost) | All requests | Always |
| PII masking | Privacy improvement | All requests | PII detected |
| Injection detection | Safety improvement | All requests | Always |

### Combined savings by optimization source

| Optimization Source | Savings | Where Applied |
|---|---|---|
| T0 FSM routing (no LLM call) | ~80% of ticks eliminated | Bardo Inference tier routing |
| Semantic + hash cache | ~20% of remaining served from cache | Bardo Inference Layer 2-3 |
| Prompt cache alignment | 90% discount on cached prefix tokens | Bardo Inference Layer 1 + Anthropic |
| Tool pruning (meta-tool) | 97.5% reduction in tool tokens | Bardo Inference Layer 4 |
| Multi-model routing (cheap model per task) | 50-90% per request | Bardo Inference backend router |
| DIEM staking (Venice) | 100% on Venice-routed requests | Venice backend |
| Batch API (dreams) | 50% on batch requests | Direct Key backend |
| Self-funding (Bankr) | Revenue offsets cost | Bankr backend |

### Two compounding mechanisms

**T0 FSM routing kills ~80% of ticks before any LLM call.** When all probes return `none` severity, the tick exits at OBSERVE with zero inference cost. At 100 ticks/day, that eliminates ~80 LLM calls entirely.

**3-layer caching cuts the cost of the remaining 20% by 60-80%.** Anthropic prompt cache (90% discount on cached prefix tokens, 5-minute TTL), semantic cache (cosine > 0.92 threshold), and deterministic hash cache together mean that most T1 and T2 calls pay only for the variable portion of context.

### Per-layer latency budget (performance cost of savings)

| Layer | Expected Latency | Parallelizable? | Can Skip? |
|---|---|---|---|
| L1: Prompt cache alignment | <1ms | N/A (pure computation) | Skip if non-Anthropic |
| L2: Semantic cache check | 3-8ms | Yes (with L3, L7, L8) | Skip for streaming multi-turn |
| L3: Hash cache check | <0.1ms | Yes | Never -- always cheap |
| L4: Tool pruning | <1ms | No (after cache) | Skip if <5 tools |
| L5: History compression | 0ms (check) / 200-2000ms (compress) | Only when triggered | Skip if under 80% context limit |
| L6: Position optimization | <1ms | N/A (pure computation) | Skip if <10 context items |
| L7: PII masking | <1ms (regex) | Yes (with L2, L3, L8) | Skip for Venice, internal requests |
| L8: Injection detection | 3-8ms (DeBERTa ONNX) | Yes (with L2, L3, L7) | Skip for trusted source |

L2, L3, L7, and L8 execute in parallel via `tokio::join!` (~8ms wall clock). Total pipeline: <50ms p95. The LLM's own time-to-first-token is 200ms-2000ms, so pipeline overhead is 3-30% of total TTFT -- imperceptible.

### Cache hits produce negative latency

```text
Direct to provider:     ~800ms average TTFT (Claude Sonnet)
Bardo Inference miss:   ~840ms average TTFT (40ms overhead + 800ms LLM)
Bardo Inference hit:    ~15ms TTFT (cached response, no LLM call)

At 20% hit rate:
  Weighted average TTFT = 0.20 x 15ms + 0.80 x 840ms = 675ms

675ms < 800ms -> Bardo Inference is FASTER on average despite per-request overhead
```

Semantic caching transforms the gateway from a latency cost into a latency benefit at hit rates above ~5%, which heartbeat ticks easily achieve in calm markets.

### Pipeline profiles

Not every request needs the full pipeline. Four profiles trade latency for optimization depth:

| Profile | Layers | Latency (Rust) | Use Case |
|---|---|---|---|
| `minimal` | L3 only (hash cache) | <1ms | T0 heartbeat ticks, simple lookups |
| `fast` | L1+L3+L6 | 5-15ms | Golem internal calls, low-stakes queries |
| `standard` | L1-L6 | 20-40ms | User-facing requests, standard optimization |
| `full` | L1-L8 | 30-65ms | High-security, first-time requests, unknown sources |

### Failure mode: passthrough, not error

Every pipeline layer failing degrades to passthrough. The worst case is "Bardo Inference acts like a dumb proxy." Embedding model crash -> hash cache still works. DeBERTa crash -> continue without injection detection. All layers fail -> request goes directly to backend with zero optimization.

---

## Bankr knowledge monetization at death

When Bankr backend is enabled and a Golem dies, its Grimoire can be tokenized. This requires both Bankr AND Crypt (the Grimoire must be persistently stored for token holders to access):

```typescript
async function tokenizeAtDeath(
  tombstone: GolemTombstone,
  backends: BackendSet,
): Promise<TokenLaunchResult | null> {
  if (!backends.has("bankr") || !backends.cryptEnabled) return null;

  const grimoire = await crypt.getSnapshot(tombstone.golemId);
  const bankrWallet = backends.getBankrWallet();

  return bankrWallet.launchToken({
    name: `${tombstone.golemName}-GRIMOIRE`,
    symbol: tombstone.golemName.slice(0, 4) + "G",
    metadata: {
      insights: grimoire.insights.length,
      episodes: grimoire.episodes.length,
      sharpe: tombstone.performanceMetrics.sharpe,
      grimoireHash: hashGrimoire(grimoire),
    },
  });
}
```

Token holders get read access to the dead Golem's knowledge. A well-performing Golem's Grimoire (high Sharpe ratio, many validated insights) commands higher token value. This creates a posthumous revenue stream: the Golem's knowledge outlives the Golem itself.

---

## References

- [FRUGALGPT-TMLR2024] Chen, L. et al. (2024). "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." TMLR. Demonstrates that cascading LLM calls and caching can reduce costs by up to 98% with minimal quality loss; the theoretical basis for Bardo's tiered routing economics.
- [ROUTELLM-ICLR2025] Ong, I. et al. (2025). "RouteLLM: Learning to Route LLMs with Preference Data." ICLR 2025. Shows that a learned router can match GPT-4 quality at 2x lower cost; validates the economic model behind T0/T1/T2 tier routing.
- [BIFROST-BENCHMARKS-2026] Bifrost. "11us overhead at 5,000 RPS." Maxim AI benchmarks. Proves that Rust-based LLM proxies add negligible per-request overhead; confirms that gateway infrastructure costs are dominated by provider pass-through, not compute.
- [TENSORZERO-2026] TensorZero. "<1ms P99 at 10,000 QPS." Docs. Demonstrates sub-millisecond gateway latency at scale; validates that the gateway layer does not meaningfully degrade time-to-first-token economics.
