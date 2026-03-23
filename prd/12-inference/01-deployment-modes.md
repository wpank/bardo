# 01 -- Three deployment modes [SPEC]

**Embedded, local, and remote inference: where the gateway runs and what it costs**

> Related: [00-overview.md](00-overview.md) (gateway architecture, payment flows, and design principles), [12-providers.md](12-providers.md) (five provider backends with self-describing resolution), [18-golem-config.md](18-golem-config.md) (Golem-specific inference provider configuration and capability matrix)

---

> **Reader orientation:** This document specifies the three deployment modes for Bardo Inference (the LLM inference gateway layer of the Bardo ecosystem). It belongs to the inference plane of the architecture. The key concept is that the same gateway binary and 8-layer context engineering pipeline can run embedded in the TUI, as a local standalone process with a local model, or as a remote multi-tenant gateway -- each with different cost, privacy, and operational tradeoffs. For term definitions, see `prd2/shared/glossary.md`.

## Why mode matters

Every Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) needs inference to think. An LLM call is how a golem observes the market, evaluates a strategy, decides to trade, dreams about counterfactuals, and composes its death testament. Where those LLM calls happen -- on your machine, in a remote gateway, or inlined into the TUI process itself -- is the single biggest decision an owner makes about cost, privacy, and operational complexity.

Bardo supports three deployment modes. Each gives the golem access to the same 8-layer context engineering pipeline, the same provider routing, the same caching stack. The difference is where the code runs.

---

## Mode A: Embedded in the TUI process

The lightest option. No separate inference process. No local model. The TUI calls remote API providers directly, with the gateway's context engineering pipeline running in-process.

When the TUI starts with `inference.mode = "embedded"`, it spawns an in-process Axum server on a random localhost port. This server runs the full gateway pipeline -- prompt cache alignment, tool pruning, history compression, semantic caching, provider routing -- but never binds to an external interface. Only the local golem and Meta Hermes can reach it.

```toml
# golem.toml
[inference]
mode = "embedded"

[inference.providers.blockrun]
priority = 1
# No API key needed -- x402 USDC payment

[inference.providers.venice]
api_key = "ven_..."
priority = 2
```

The embedded gateway shares the TUI's process memory. It adds roughly 50-80 MB of RAM overhead (the ONNX models for embedding and injection detection load into the TUI's address space). CPU impact is negligible -- the pipeline adds <50ms per request at P95, dominated by the DeBERTa injection classifier at 8ms and the nomic-embed embedding at 5ms.

**When to use it.** Starting out. Testing a golem locally. Running on a machine without a GPU. You want zero operational overhead: `bardo run` starts everything. One process, one concern.

**Limitations.** All inference goes to remote providers, so you pay per call. Latency depends on your internet connection and the provider's load. Remote golems cannot reach the embedded gateway -- it binds to localhost only. If you run multiple golems, each gets its own embedded gateway, duplicating cache state. For multi-golem setups, Mode B or Mode C makes more sense.

**Cost profile.** Every LLM call costs money. With the context engineering pipeline active, expect ~40% savings versus calling providers directly. A golem running 100 ticks/day in calm markets pays about $0.05-0.10/day. Volatile markets push that to $0.40-0.80/day as more ticks escalate to T2.

---

## Mode B: Local standalone process

A separate gateway process runs on your machine, usually alongside Ollama serving Hermes 4.3. This is the local-inference mode: most LLM calls hit a model running on your own hardware. Cloud providers handle only the hard problems.

```bash
# Start the local inference stack
bardo inference up
# This command:
# 1. Starts Ollama if not running
# 2. Pulls Hermes 4.3 at your configured quantization
# 3. Starts bardo-gateway on localhost:8443
# 4. Runs health checks on all configured providers
```

The gateway process is the same `bardo-gateway` Rust binary used in all modes. Here it runs as a standalone daemon, listening on `localhost:8443`. It routes requests to Ollama for T0/T1 ticks (free) and to cloud providers for T2 (paid).

```toml
# golem.toml
[inference]
mode = "local"
gateway_url = "http://localhost:8443"

[inference.local]
ollama_url = "http://localhost:11434"
model = "hf.co/NousResearch/Hermes-4.3-36B-GGUF:Q6_K"
tiers = ["T0", "T1"]  # Local handles T0 and T1

[inference.providers.venice]
api_key = "ven_..."
tiers = ["T1", "T2"]
priority = 2  # Fallback for T1, primary for T2

[inference.providers.blockrun]
tiers = ["T2"]
priority = 3  # T2 only
```

### Architecture

Your machine runs three processes:

```
Owner's Machine
+----------------------------------------------------------+
|                                                           |
|  bardo-terminal ---- http://localhost:8443 ----> bardo-gateway
|                                                    |
|                                                    +---> Ollama (Hermes 4.3)
|                                                    |     localhost:11434
|                                                    +---> Venice API (HTTPS)
|                                                    +---> BlockRun API (HTTPS)
|                                                           |
|  Golem Container (local) -- http://localhost:8443 -->----+
|                                                           |
+----------------------------------------------------------+
```

The TUI, local golems, and Meta Hermes all share a single gateway instance. Cache state is unified: if one golem's heartbeat produces a cached response, another golem asking a similar question gets the cache hit. This matters for Clades (peer-to-peer networks of Golems sharing knowledge) running similar strategies.

Remote golems can also point to this gateway if you expose port 8443 through a VPN (Tailscale, WireGuard) or SSH tunnel. The gateway authenticates requests via ERC-8004 (the on-chain agent identity standard) identity or a shared secret -- it does not need to be on the public internet.

**When to use it.** You have a machine with a decent GPU (or Apple Silicon with unified memory). You want most inference to be free and private. You run multiple golems and want shared caching. This is the recommended mode for anyone with a Mac Mini, Mac Studio, or a Linux box with an NVIDIA GPU.

### Hermes 4.3 and VRAM requirements

Hermes 4.3 from NousResearch is the recommended local model. It is a 36B-parameter model fine-tuned for function calling, structured output, and reasoning -- the three capabilities a golem needs most. Available quantizations:

| Quantization | VRAM | Quality | Speed (M3 Pro) | Speed (RTX 4090) |
|---|---|---|---|---|
| Q4_K_M | ~20 GB | Good. Noticeable quality loss on complex reasoning. Fine for T0/T1. | ~12 tok/s | ~35 tok/s |
| Q5_K_M | ~25 GB | Better. Minimal degradation on most tasks. | ~10 tok/s | ~30 tok/s |
| Q6_K | ~28 GB | Near-full quality. Recommended default. | ~8 tok/s | ~25 tok/s |
| Q8_0 | ~36 GB | Negligible loss. For machines with headroom. | ~6 tok/s | ~20 tok/s |
| F16 | ~72 GB | Full precision. No quality loss. Requires beefy hardware. | ~3 tok/s | ~12 tok/s |

Pick Q6_K unless you have a reason not to. Q4_K_M if your GPU has <32 GB VRAM. F16 if you have a 96+ GB system and want maximum quality.

For Apple Silicon: the numbers above assume unified memory. A Mac Mini M4 Pro with 48 GB can run Q4_K_M comfortably with ~28 GB left for the OS, gateway, and golem runtime. A Mac Studio M4 Ultra with 192 GB can run F16 and still have 120 GB free. A MacBook Pro M3 Pro with 18 GB does not have enough memory for 36B -- use a cloud provider or a smaller model for T1 and route to Hermes via a remote gateway.

### Multi-model local stacks

Machines with 64 GB or more of unified memory (Mac Studio M4 Ultra) or dual NVIDIA GPUs can run multiple models simultaneously through the same Ollama instance:

```toml
[inference.providers.local-hermes]
endpoint = "http://localhost:11434/v1"
model = "hf.co/NousResearch/Hermes-4.3-36B-GGUF:Q6_K"
tiers = ["T1", "T2"]
priority = 1

[inference.providers.local-small]
endpoint = "http://localhost:11434/v1"
model = "qwen2.5:3b-instruct-q8_0"
tiers = ["T0"]
priority = 1
```

Ollama multiplexes models automatically, loading and unloading based on available memory. With 192 GB (Mac Studio M4 Ultra): Hermes 4.3 36B Q6_K (~28 GB) + Qwen 3B Q8_0 (~4 GB) + nomic-embed (~0.5 GB) + OS (~6 GB) + gateway + golem runtime (~1 GB) = ~40 GB. That leaves 152 GB of headroom for context windows and other applications. On a 64 GB machine, use Q4_K_M (~20 GB) instead of Q6_K.

### Health monitoring

The gateway checks Ollama's health every 30 seconds. Three states:

- **Healthy**: Ollama running, model loaded, last inference succeeded. Requests route to local model.
- **Degraded**: Ollama running but model unloaded (memory pressure) or inference speed dropped below threshold. Gateway still routes to Ollama but logs warnings. The TUI shows a yellow indicator.
- **Down**: Ollama not responding. All requests fall back to cloud providers. The TUI shows a red indicator and an alert.

The health check is cheap: a single `GET /api/tags` to Ollama, verifying the model is in the loaded list. No test inference, no wasted tokens.

### Docker compose all-in-one

For owners who prefer containers:

```yaml
# docker-compose.yml
services:
  ollama:
    image: ollama/ollama:latest
    volumes:
      - ollama_data:/root/.ollama
    ports:
      - "11434:11434"
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]

  gateway:
    image: ghcr.io/bardo-run/bardo-gateway:latest
    environment:
      - GATEWAY_BIND=0.0.0.0:8443
      - PROVIDER_LOCAL_ENDPOINT=http://ollama:11434/v1
      - PROVIDER_LOCAL_MODEL=hf.co/NousResearch/Hermes-4.3-36B-GGUF:Q6_K
    ports:
      - "8443:8443"
    depends_on:
      - ollama

  golem:
    image: ghcr.io/bardo-run/golem-container:latest
    volumes:
      - golem_data:/data
      - ./golem.toml:/config/golem.toml
      - ./STRATEGY.md:/config/STRATEGY.md
    environment:
      - INFERENCE_ENDPOINT=http://gateway:8443
    depends_on:
      - gateway

volumes:
  ollama_data:
  golem_data:
```

`docker compose up -d` starts everything. Connect the TUI: `bardo attach --direct ws://localhost:8080/ws`.

**Cost profile.** T0 and T1 inference is free (local Hermes). Only T2 calls hit cloud providers. In calm markets, 80-90% of ticks resolve at T0 (no LLM call at all) and most of the remaining ticks are T1 (local, $0). Expected cost: $0.02-0.05/day in calm markets, $0.15-0.25/day in normal conditions, $0.40-0.60/day in volatile conditions. Electricity is the main cost -- Ollama serving inference uses 30-80W depending on GPU load.

**Privacy advantage.** With local inference handling T0 and T1, roughly 95% of your golem's thoughts in a calm market never leave your machine. Strategy evaluations, position assessments, routine tool calls -- all processed locally. Only the rare T2 escalation (novel situations requiring frontier models) transits to a cloud provider. For owners who care about keeping their trading strategies private, this is a strong reason to run Mode B even if the cost savings are modest.

**Latency advantage.** Local inference eliminates network round-trip for T1 calls. Hermes 4.3 Q6_K on an M3 Pro generates first token in ~50ms, compared to ~200-400ms for a cloud API including network latency. For a golem making rapid T1 assessments during a volatile market, local inference lets it think faster. The golem can evaluate 15 T1 ticks in the time a cloud-dependent golem evaluates 10.

---

## Mode C: Remote gateway

A `bardo-gateway` instance running on remote infrastructure. The full 8-layer pipeline, multi-provider routing, and caching -- accessible over HTTPS from anywhere. Two flavors: self-deployed or hosted.

### Self-deployed

You run your own gateway. You bring your own provider API keys. No x402 spread -- you pay raw provider costs after context engineering savings.

```bash
# Deploy to Fly.io
bardo inference deploy --provider fly --name my-gateway --region iad
# -> https://my-gateway.fly.dev

# Deploy to a VPS via SSH
bardo inference deploy --provider ssh --host 203.0.113.42 --user root
# -> https://gateway.yourdomain.com (Caddy handles TLS)
```

The `deploy` command downloads the `bardo-gateway` binary for the target platform (~50 MB static binary), uploads it, configures a systemd service, prompts for provider API keys, and starts the gateway. On Fly.io it generates the `fly.toml` and runs `fly deploy`. On a VPS it also sets up Caddy for automatic HTTPS.

```toml
# golem.toml
[inference]
mode = "remote"
gateway_url = "https://my-gateway.fly.dev"

# Gateway authenticates via ERC-8004 identity (auto-detected from wallet)
# Or: API key for simpler setups
gateway_api_key = "bardo_gw_..."
```

The self-deployed gateway configuration lives on the server:

```toml
# bardo-gateway.toml (on the gateway server)
[server]
bind = "0.0.0.0:8443"

[auth]
mode = "erc8004"  # Or "shared_secret" for personal use

[providers.blockrun]
priority = 1
tiers = ["T1", "T2"]

[providers.venice]
api_key = "ven_..."
priority = 2
tiers = ["T1", "T2"]

[providers.openrouter]
api_key = "sk-or-..."
priority = 3
tiers = ["T2"]  # Fallback only

[cache]
semantic_enabled = true
hash_enabled = true
prefix_enabled = true

[pipeline]
layers = ["prefix_align", "semantic_cache", "hash_cache", "tool_prune",
          "history_compress", "defi_enrich", "safety_scan", "injection_detect"]
```

Resource requirements are modest. The gateway binary uses <200 MB RAM idle, <500 MB under load. CPU usage is minimal -- the ONNX models (DeBERTa at ~8ms, nomic-embed at ~5ms) run on CPU and consume a single core briefly per request. It fits on the cheapest VPS tier: ~$4/month on Hetzner, $2-5/month on Fly.io.

### Hosted

`gateway.bardo.run` is a Bardo-operated gateway. Any ERC-8004 registered agent can use it. No setup. Payment via x402 (a micropayment protocol for HTTP-native USDC payments on Base) USDC with a configurable spread (default 20%, down to 8% for Sovereign-tier reputation).

```toml
# golem.toml
[inference]
mode = "remote"
gateway_url = "https://gateway.bardo.run"
# Auth: ERC-8004 identity auto-detected from wallet
```

The hosted gateway applies the same context engineering, saving ~40% on raw provider costs. Even with the 20% spread, users typically pay less than calling providers directly. The math: if context engineering saves 40% and the spread is 20% of the optimized cost, the all-in price is roughly 0.60 * 1.20 = 0.72x of the naive cost. That is a 28% savings.

Spread tiers by ERC-8004 reputation:

| Reputation tier | Spread | How to reach it |
|---|---|---|
| Default (no identity) | 20% | No ERC-8004 linked |
| Basic (50+ score) | 18% | Active agent with some history |
| Verified (200+ score) | 15% | Established track record |
| Trusted (500+ score) | 12% | Long-running, profitable agent |
| Sovereign (1000+ score) | 8% | Elite agents, significant on-chain history |

**When to use it.** Fleets of golems across multiple machines. Bardo Compute containers that need inference without running Ollama in each VM. Owners who want zero local infrastructure. The hosted option is the lowest-friction path -- two lines of config and inference works.

Self-deployed is for cost-conscious operators running multiple golems. The gateway is shared infrastructure: ten golems can share one $4/month VPS, splitting cache state across the fleet. Provider API costs are the only variable.

**Cost profile.** Self-deployed: raw provider costs only, no spread. Infrastructure: $4-50/month depending on scale. Hosted: provider cost + 20% spread (less with reputation). Both benefit from the full ~40% context engineering savings.

---

## Mode comparison

| | Mode A: Embedded | Mode B: Local | Mode C: Remote |
|---|---|---|---|
| Setup time | 0 minutes | 2-5 minutes | 0 min (hosted), 5-10 min (self-deployed) |
| Separate process | No | Yes (gateway + Ollama) | Yes (remote) |
| Local model | No | Yes (Hermes 4.3 via Ollama) | No (unless Ollama on gateway server) |
| Free inference | No | T0/T1 free (local) | No |
| Privacy | Prompts transit to cloud | T0/T1 stay local | Prompts transit to gateway |
| Multi-golem | Separate caches per golem | Shared cache | Shared cache |
| Monthly infra cost | $0 | $0 (electricity only) | $0 (hosted) or $4-50 (self-deployed) |
| Best for | Single golem, testing, low-resource | GPU owners, privacy, cost savings | Fleets, Compute containers, zero-ops |

Most owners will start with Mode A and graduate to Mode B once they realize how much they spend on cloud inference. Mode C is for operators running serious fleets or for golems deployed on Bardo Compute where local GPUs are not available.

---

## Inference in Compute containers

When a Golem runs on Bardo Compute (the VM hosting service for Golems, running on Fly.io), it does not run Ollama locally -- the VM does not have GPU access. Instead, the golem's `bardo-provider-adapter` runtime extension registers a remote inference gateway as the sole provider at session start.

Each Compute VM tier includes a base inference token allowance. Overages bill from the golem's wallet. See [prd2/11-compute/03-billing.md](../11-compute/03-billing.md) for tier-to-allowance mapping and overage pricing.

---

## Dream-state inference

Dream cycles are the golem's most inference-heavy state. During hypnagogic sleep, the golem replays market episodes, tests counterfactual strategies, simulates threat scenarios, and integrates insights into its Grimoire (the agent's persistent knowledge base). This is T2-heavy work (T0/T1/T2 being the three-tier model routing system: T0 is a zero-cost finite state machine, T1 uses a cheap model, T2 escalates to a frontier model) -- frontier models reasoning about complex multi-step problems.

| Dream phase | Model tier | Privacy | Typical cost |
|---|---|---|---|
| NREM (memory consolidation) | T1 | Private (Venice preferred) | $0.01-0.02 per cycle |
| REM (creative recombination) | T2 | Private (Venice preferred) | $0.05-0.15 per cycle |
| Integration (insight promotion) | T2 | TEE-attested (Venice TEE) | $0.03-0.08 per cycle |

Dream cycles are a good fit for Anthropic's Batch API (50% discount, 24-hour SLA) since they are not latency-sensitive. A golem can queue its dream prompts before sleeping and process the results when it wakes. The gateway routes dream-subsystem requests to the Batch API when available.

Venice is the preferred provider for dreams because dream content is privacy-sensitive -- the golem is reasoning about its own strategies, weaknesses, and mortality. DIEM-funded Venice calls make dream inference free, which matters because dreams are the single largest per-cycle inference cost.

---

## Two layers of context engineering

There is an asymmetry worth calling out. The gateway applies context engineering to all inference calls: prompt caching, token budgets, tool pruning, model routing. This reduces cost. But the golem also runs its own context engineering inside the runtime, through the Context Governor. These two systems are complementary, not redundant.

The gateway sees assembled prompts and returns completions. It is stateless with respect to the golem's memory. It optimizes the mechanics of LLM interaction: how many tokens, which model, which cache.

The Context Governor operates inside the golem's Rust runtime. It assembles the Cognitive Workspace fresh each tick -- not a growing conversation that eventually gets compressed, but a purpose-built context assembled from structured categories with learned token allocations. It implements Alan Baddeley's working memory model: a central executive (the Governor) allocates attention (tokens) across an episodic buffer (the workspace), drawing from positions, strategies, Grimoire entries, causal models, and somatic readings.

Three cybernetic feedback loops tune the workspace over time:

1. **Per-tick outcome correlation.** After every tick with an outcome, the Governor correlates which Styx (the knowledge retrieval and injection plane) entries appeared in the workspace with whether the outcome was positive. Entries in winning contexts get weight; entries in losing contexts lose it.
2. **Per-curator policy evolution (every 50 ticks).** Aggregates tick-level correlations into category-level performance. Categories with consistently positive correlations get more tokens in the budget.
3. **Per-regime restructuring.** When the market regime changes, correlations partially reset (50% decay). Knowledge from one regime does not fully transfer to another.

The result: after a few hundred ticks, the Context Governor knows which categories of context improve decisions for this golem in this market regime. It stops wasting tokens on information that does not help.

This dual-layer design means the gateway optimizes cost while the Context Governor optimizes quality. They compound. A golem running through both systems pays less for better decisions than a golem running through either alone.

---

## T0/T1/T2 cost optimization

### The local-first strategy

Run Hermes 4.3 locally for T0/T1. Send only T2 to the cloud. This is the highest-leverage cost optimization in the entire system.

At 100 ticks/day in a normal market:

| Tier | Ticks | Local cost | Cloud cost | Hybrid cost |
|---|---|---|---|---|
| T0 (80 ticks) | 80 | $0.00 | $0.00 | $0.00 |
| T1 (15 ticks) | 15 | $0.00 | $0.045 | $0.00 |
| T2 (5 ticks) | 5 | N/A | $0.15 | $0.15 |
| **Total** | 100 | **$0.00** | **$0.195** | **$0.15** |

The hybrid approach (Mode B) saves $0.045/day on T1 alone. Over 30 days that is $1.35 saved -- not huge in absolute terms, but for a golem with a $6.00 monthly inference budget, it is 22% of the budget.

In volatile markets the savings are larger because T1 traffic increases:

| Tier | Ticks | Cloud cost | Hybrid cost | Savings |
|---|---|---|---|---|
| T0 (60 ticks) | 60 | $0.00 | $0.00 | $0.00 |
| T1 (25 ticks) | 25 | $0.075 | $0.00 | $0.075 |
| T2 (15 ticks) | 15 | $0.75 | $0.75 | $0.00 |
| **Total** | 100 | **$0.825** | **$0.75** | **$0.075/day** |

### Worked example: hosted gateway economics

A user sends a request with 50K input tokens and 2K output tokens through the hosted gateway, targeting a Sonnet-class model.

```
Naive cost (direct API):                 $0.180
  Input:  50K x $3.00/M  = $0.150
  Output:  2K x $15.00/M = $0.030

Context engineering saves ~40%:
  30K tokens hit prefix cache (90% discount):  -$0.040
  Tool pruning removes 8K tokens:              -$0.024
  History compression saves 2K tokens:         -$0.006
  Optimized cost (BlockRun):                    $0.099

User pays (20% spread):                        $0.119
  BlockRun cost:  $0.099
  Spread:         $0.020

User saves vs. direct API:  34% ($0.180 -> $0.119)
Gateway operator margin:    $0.020
```

The user saves money despite the 20% spread. The gateway operator earns $0.02/request. Both sides win because context engineering creates value that did not exist before -- the ~40% in savings is split between the user (~60% of the savings) and the operator (~40% of the savings via the spread).

For a self-deployed gateway: the user pays $0.099 (no spread). A $4/month VPS breaks even at ~200 requests/month, or about 7 requests/day.

### Daily cost projections

For a golem running 100 ticks/day with the full optimization stack:

| Scenario | T0 rate | T1 cost | T2 cost | Daily total |
|---|---|---|---|---|
| Calm market (90% T0) | 90% | $0.024 | $0.02 | ~$0.05 |
| Normal market (80% T0) | 80% | $0.045 | $0.15 | ~$0.20 |
| Volatile market (60% T0) | 60% | $0.075 | $0.75 | ~$0.83 |

These numbers assume cloud-only inference (Mode A or C). With Mode B (local Hermes for T0/T1), subtract the T1 cost entirely.

---

## Cost-effectiveness feedback loop

The evaluation system measures whether paid inference is worth the money.

The metric: **delta-accuracy per dollar per tier per category.** For each prediction category (price, fee_rate, liquidity, etc.), the system compares the accuracy of predictions made at each inference tier.

```rust
pub struct CostEffectiveness {
    pub category: PredictionCategory,
    pub tier: InferenceTier,
    /// Accuracy of predictions made at this tier for this category
    pub accuracy: f32,
    /// Average cost per prediction at this tier
    pub cost_per_prediction: f64,
    /// How much better this tier is vs the next cheaper tier
    pub delta_accuracy: f32,
    /// delta_accuracy / cost_per_prediction
    pub accuracy_per_dollar: f64,
}
```

If T2 (paid API, ~$0.003/prediction) achieves 72% accuracy on fee_rate predictions, and T1 (local model, ~$0.0001/prediction) achieves 68% accuracy on the same category, the delta is 4% accuracy for 30x the cost. At roughly 500 fee_rate predictions per day, that's $1.50/day for 4% more accuracy. Whether that's worth it depends on the category's economic impact.

The auto-shift rule: if a category's `accuracy_per_dollar` falls below a configurable threshold for 7 consecutive days, the inference router demotes that category to the next cheaper tier. If accuracy drops noticeably after demotion, the system promotes it back after 3 days.

This prevents the common failure mode of using expensive models for everything. Some categories (binary yes/no predictions about gas prices) don't benefit from larger models. Others (complex multi-factor fee rate predictions) do. The system learns which is which.

---

## Provider performance evaluation

Beyond cost-effectiveness per tier, the system tracks per-provider accuracy.

When the same prediction category is routed to different providers (due to fallback chains or load balancing), the system records which provider generated which prediction. Over time, this builds a provider performance profile:

```rust
pub struct ProviderPerformance {
    pub provider_id: String,
    pub model: String,
    /// Per-category accuracy for this provider
    pub category_accuracy: HashMap<PredictionCategory, f32>,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Cost per 1K tokens (input/output averaged)
    pub cost_per_1k_tokens: f64,
    /// Uptime fraction over last 7 days
    pub uptime_7d: f32,
}
```

The inference router uses this data to adjust provider priority weights. A provider with consistently higher accuracy for a given category gets higher priority for that category, even if it's more expensive, subject to the cost-effectiveness threshold.

This connects to mortality: inference cost feeds the economic vitality clock. A golem that routes all predictions through expensive providers burns through credits faster. The cost-effectiveness loop applies pressure toward the cheapest tier that maintains acceptable accuracy. A golem that learns to use local inference for simple predictions and paid inference only for complex ones lives longer.

---

## First-run wizard integration

The first time an owner runs `bardo init`, the setup wizard asks about inference. The decision tree:

1. "Do you have a GPU?" -> Yes: recommend Mode B. No: recommend Mode A.
2. If Mode B: "How much VRAM/unified memory?" -> Recommend quantization level.
3. "Do you have API keys for any providers?" -> Configure them or default to BlockRun (x402, no key needed).
4. "Do you want to run your own gateway?" -> If yes, walk through Mode C self-deploy.

The wizard writes the `[inference]` section of `golem.toml`. Owners can change it later from the TUI's inference screen or by editing the file directly.

---

## TUI inference window

The TUI's Config window includes an Inference section showing real-time provider status, cost tracking, and configuration controls.

```
+-- Inference Configuration -------------------------------------------+
|                                                                       |
|  Mode: [Local Standalone]                                            |
|                                                                       |
|  +- Provider Stack -----------------------------------------------+  |
|  |                                                                 |  |
|  |  #   Provider        Tiers    Status         Latency    Auth   |  |
|  |  --  --------        -----    ------         -------    ----   |  |
|  |  1.  Local Hermes    T0,T1    * Healthy       25 tok/s  Auto   |  |
|  |  2.  Venice          T1,T2    * Healthy       891ms     Key    |  |
|  |  3.  BlockRun        T2       * Healthy       234ms     x402   |  |
|  |  4.  OpenRouter      T2       o No key        --        --     |  |
|  |                                                                 |  |
|  |  [Arrows] Reorder  [A] Add  [E] Edit  [X] Remove  [T] Test    |  |
|  +-----------------------------------------------------------------+  |
|                                                                       |
|  +- Local Model ---------------------------------------------------+  |
|  |  Model: Hermes 4.3 36B Q6_K                                    |  |
|  |  Status: * Loaded (28.1 GB)                                      |  |
|  |  Speed: 24.7 tok/s (last 100 requests)                          |  |
|  |  Context: 8,192 tokens (max 32,768)                              |  |
|  |                                                                   |  |
|  |  [1] Q4_K_M  [2] Q5_K_M  [3] Q6_K (current)  [4] Q8_0  [5] F16 |  |
|  |  [B] Benchmark  [P] Pull new model  [U] Unload                   |  |
|  +-------------------------------------------------------------------+  |
|                                                                       |
|  +- Cost Summary (last 24h) ---------------------------------------+  |
|  |  Local Hermes:  847 calls   $0.00    (89%)                      |  |
|  |  Venice:         92 calls   $0.18    (10%)                      |  |
|  |  BlockRun:       12 calls   $0.31    (1%)                       |  |
|  |  Total:         951 calls   $0.49                                |  |
|  +-----------------------------------------------------------------+  |
|                                                                       |
+-----------------------------------------------------------------------+
```

From this screen you can switch inference modes without restarting, add or remove providers, reassign tier routing, flush caches (semantic, hash, or all), and drill into cost history with per-day, per-week, and per-month breakdowns by tier and provider.

---

## Configuration reference

### golem.toml inference section

```toml
[inference]
# "embedded" | "local" | "remote"
mode = "local"

# Only for mode = "remote"
gateway_url = "https://my-gateway.fly.dev"
gateway_api_key = "bardo_gw_..."  # Or use ERC-8004 auto-auth

[inference.local]
ollama_url = "http://localhost:11434"
model = "hf.co/NousResearch/Hermes-4.3-36B-GGUF:Q6_K"
# Which tiers the local model handles
tiers = ["T0", "T1"]
# Health check interval in seconds
health_check_interval = 30

[inference.budget]
# Per-day budget cap in USD (0 = unlimited)
daily_cap_usd = 5.0
# Per-tier caps
t1_daily_cap_usd = 1.0
t2_daily_cap_usd = 4.0

[inference.cache]
semantic_enabled = true
semantic_threshold = 0.92
hash_enabled = true
hash_max_entries = 5000

[inference.providers.blockrun]
priority = 1
tiers = ["T1", "T2"]

[inference.providers.venice]
api_key = "ven_..."
priority = 2
tiers = ["T1", "T2"]
# DIEM staking for free inference
diem_enabled = true

[inference.providers.openrouter]
api_key = "sk-or-..."
priority = 3
tiers = ["T2"]
```

### bardo-gateway.toml (server configuration)

```toml
[server]
bind = "0.0.0.0:8443"
# TLS handled by reverse proxy (Caddy, Fly.io) in production
tls = false

[auth]
# "erc8004" (multi-user, identity-based) or "shared_secret" (personal)
mode = "erc8004"

[pipeline]
# All 8 layers enabled by default
layers = ["prefix_align", "semantic_cache", "hash_cache", "tool_prune",
          "history_compress", "defi_enrich", "safety_scan", "injection_detect"]

[cache]
semantic_enabled = true
semantic_threshold = 0.92
semantic_max_entries = 10000
hash_enabled = true
hash_max_entries = 5000
prefix_enabled = true

[ml]
embedding_model = "nomic-embed-text-v1.5"
injection_model = "deberta-v3-base"

[providers.blockrun]
priority = 1
tiers = ["T1", "T2"]

[providers.venice]
api_key = "ven_..."
priority = 2
tiers = ["T1", "T2"]

[providers.openrouter]
api_key = "sk-or-..."
priority = 3
tiers = ["T2"]
```

---

## Cross-references

| Topic | Document | What it covers |
|---|---|---|
| Gateway architecture and 8-layer pipeline | [00-overview.md](00-overview.md) | Top-level inference gateway spec: architecture, payment flows, provider routing, context engineering pipeline, and deployment topology |
| Model routing and subsystem intents | [01a-routing.md](01a-routing.md) | Self-describing providers, declarative intents, mortality-aware resolution, and the full routing algorithm across five backends |
| Caching layers | [02-caching.md](02-caching.md) | Three-layer cache stack: prompt prefix alignment, semantic response cache, and deterministic hash cache with regime-aware invalidation |
| Revenue model | [03-economics.md](03-economics.md) | x402 spread revenue model, per-tenant cost attribution, and infrastructure cost projections |
| Context engineering details | [04-context-engineering.md](04-context-engineering.md) | The 8-layer pipeline: prompt cache alignment, semantic cache, hash cache, tool pruning, history compression, KV-cache routing, PII masking, and injection detection |
| Five provider backends | [12-providers.md](12-providers.md) | BlockRun, OpenRouter, Venice, Bankr, and Direct Key provider implementations with Rust trait definitions and self-describing resolution |
| Golem provider config | [18-golem-config.md](18-golem-config.md) | Golem-specific inference provider configuration, capability matrix, and payment method selection |
| Compute billing and inference allowances | [prd2/11-compute/03-billing.md](../11-compute/03-billing.md) | How Compute VM tiers include base inference token allowances and bill overages from the Golem's wallet |
| Dream-state inference | [prd2/06-hypnagogia/02-architecture.md](../06-hypnagogia/02-architecture.md) | Hypnagogic sleep architecture: NREM replay, REM creative recombination, and inference profiles for liminal phase transitions |
| Golem container structure | [prd2/01-golem/00-overview.md](../01-golem/00-overview.md) | Three-process container (golem-binary, hermes-agent, sanctum-ts), JSON-RPC over Unix sockets, and sidecar minimization policy |
| Compute deployment paths | [prd2/11-compute/00-overview.md](../11-compute/00-overview.md) | Managed, self-deploy, and bare metal deployment with export-binary escape hatch |
