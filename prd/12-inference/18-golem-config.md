# Golem Inference Provider Configuration

> **Document Type**: SPEC (normative) | **Version**: 1.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-14
>
> **Package**: `@bardo/golem` (`bardo-provider-adapter`)
>
> **Depends on**: `prd2-bardo-inference.md`, `prd2-model-routing.md`
>
> **Purpose**: How a user creates a Golem and specifies inference providers. What each provider combination unlocks. How autonomous inference is paid for. How the Pi runtime resolves available capabilities without requiring Bardo Inference.

---

> **Reader orientation:** This document specifies how a user creates a Golem (a mortal autonomous DeFi agent managed by the Bardo runtime) and configures its inference providers. It belongs to the Bardo Inference layer and covers the provider configuration matrix, payment method selection, capability unlocks per provider combination, and how the runtime resolves available capabilities without requiring the full gateway. The key concept is that the provider configuration at Golem creation time determines which inference features are available (private cognition, self-funding, native API features) and how they are paid for. For term definitions, see `prd2/shared/glossary.md`.

## 1. The Mental Model

When a user creates a Golem, they configure **where inference comes from**. The Golem's Pi runtime auto-detects which models and features are available through the configured providers, and every Pi-native subsystem (heartbeat, dreams, risk, daimon, curator, etc.) adapts to use the best available capability -- no manual wiring required.

There are **five provider sources** (BlockRun, OpenRouter, Venice, Bankr, Direct Keys) a Golem can use. They are not mutually exclusive -- a Golem can combine any or all of them:

```
+------------------------------------------------------------------+
|                    GOLEM INFERENCE CONFIG                          |
|                                                                   |
|  +--- Provider Sources (configure 1 or more) -----------------+  |
|  |                                                             |  |
|  |  [1] Bardo Inference  <- Optional. Context engineering      |  |
|  |      (x402 or prepaid)   proxy with BlockRun backbone.      |  |
|  |                          Adds 8-layer optimization.         |  |
|  |                                                             |  |
|  |  [2] Venice API Key   <- Zero data retention. DIEM.         |  |
|  |                          Private cognition for dreams,      |  |
|  |                          death, MEV-sensitive reasoning.    |  |
|  |                                                             |  |
|  |  [3] Bankr API Key    <- Self-funding economics.            |  |
|  |      + Wallet ID         Inference wallet = execution       |  |
|  |                          wallet. Token launching.           |  |
|  |                                                             |  |
|  |  [4] Direct API Keys  <- Raw provider access. Full          |  |
|  |      (Anthropic,         native API surface per provider.   |  |
|  |       OpenAI, Google,    Unlocks features that can't        |  |
|  |       DeepSeek, local)   pass through any proxy.            |  |
|  |                                                             |  |
|  +-------------------------------------------------------------+  |
|                                                                   |
|  +--- Payment Config -----------------------------------------+  |
|  |  Who pays for autonomous inference? (configurable)          |  |
|  |  - Golem wallet (funded at creation)                        |  |
|  |  - User's prepaid Bardo Inference balance                   |  |
|  |  - Bankr self-funding (revenue from strategies)             |  |
|  |  - Venice DIEM (staked VVV covers inference)                |  |
|  +-------------------------------------------------------------+  |
+------------------------------------------------------------------+
```

**Bardo Inference is not required.** A Golem with only a Venice API key and a direct Anthropic key is fully functional -- it just doesn't get context engineering optimizations. The Pi runtime handles routing internally using whatever providers are configured.

**Bardo Inference, when configured, is the preferred path.** All requests route through it first. It applies context engineering (caching, compression, tool pruning) and routes to its own backend set (BlockRun always, plus any additional backends the operator has configured on the Bardo Inference instance). See `prd2-bardo-inference.md` for details.

---

## 2. Provider Source Details

### 2.1 Bardo Inference

**What it is**: A hosted context engineering proxy run by the Bardo operator. All requests flow through an 8-layer optimization pipeline before reaching any model provider. BlockRun is always enabled as the backbone -- no BlockRun API key is needed; Bardo Inference pays BlockRun via x402 and marks up the cost.

**What the user pays**: Either per-request x402 (wallet-based micropayment signing) or a prepaid USDC balance. The cost includes the BlockRun x402 fee plus a flat spread markup set by the operator. Even with the spread, context engineering savings typically make Bardo Inference cheaper than calling providers directly.

**What it unlocks**:
- 8-layer context engineering pipeline (prompt cache alignment, semantic cache, hash cache, tool pruning, history compression, lost-in-the-middle mitigation, PII masking, injection detection)
- Access to all models available on BlockRun (30+ models: Claude, GPT, Gemini, DeepSeek, Grok, Qwen, etc.)
- Automatic failover and health monitoring across backends
- If the operator has configured additional backends on the Bardo Inference instance (Venice, Bankr, OpenRouter, direct keys), the Golem gets access to those too -- transparently

**What it does NOT provide on its own**:
- Zero data retention (needs Venice)
- Self-funding economics (needs Bankr)
- OpenAI Predicted Outputs, Responses API stateful sessions (needs Direct OpenAI Key)
- Gemini explicit caching, custom search grounding (needs Direct Google Key)

**Configuration**:
```typescript
{
  bardoInference: {
    enabled: true,
    // Payment: x402 wallet signing or prepaid API key
    payment:
      | { type: "x402"; walletKey: string }      // Per-request signing
      | { type: "prepaid"; apiKey: string },      // bardo_sk_... key with deposited balance
  }
}
```

### 2.2 Venice

**What it is**: A privacy-first inference provider with structural zero data retention. Models hosted on Venice never log prompts or completions. Venice offers DIEM staking -- users stake VVV tokens and receive a daily inference allowance at zero marginal cost.

**What it unlocks**:
- `securityClass: "private"` routing -- MEV-sensitive reasoning, portfolio composition analysis, death reflection, and any cognition the Golem needs to keep confidential
- DeepSeek R1 with visible `<think>` tags + zero retention = **private visible reasoning** (unique to Venice)
- DIEM staking: zero-cost inference within daily allocation
- Llama 3.3 70B, GLM-4.7, Qwen 2.5 VL 72B (private vision for chart analysis)
- Venice-specific `strip_thinking_response` parameter for controlling reasoning visibility

**What it does NOT provide**:
- Anthropic models (no Claude on Venice)
- Anthropic Citations, Compaction, prompt caching
- OpenAI models
- Cross-model verification

**Configuration**:
```typescript
{
  venice: {
    enabled: true,
    apiKey: "vn_...",
    staking: {
      enabled: true,      // Using DIEM allocation
      // If staking is enabled, Venice-routed calls consume DIEM, not USD
    },
  }
}
```

### 2.3 Bankr

**What it is**: An LLM gateway where the inference wallet is the same wallet used for on-chain execution. Revenue from DeFi strategies flows into the same account that pays for inference -- enabling self-sustaining Golems.

**What it unlocks**:
- Self-funding economics: sustainability ratio (revenue / cost) drives routing decisions
- On-chain execution in the same wallet context
- Cross-model verification: parallel inference calls to 2+ models for risk-critical decisions
- Token launching: at death, a Golem's Grimoire can be tokenized through Bankr (requires Crypt)

**Models available**: Claude (Haiku/Sonnet/Opus), GPT (4o/5.2), Gemini (Flash/Pro), DeepSeek R1

**What it does NOT provide**:
- Zero data retention (not a privacy backend)
- DeepSeek visible `<think>` tags (R1 available but without Venice's `strip_thinking_response` control)
- Anthropic prompt caching passthrough (depends on Bankr's implementation)

**Configuration**:
```typescript
{
  bankr: {
    enabled: true,
    apiKey: "bankr_...",
    walletId: "0x...",     // Bankr wallet for inference + execution
  }
}
```

### 2.4 Direct API Keys

**What it is**: The user's own API keys for specific providers. Gives raw native API access with no proxy overhead. Required for features that cannot pass through any intermediary.

**Features exclusive to Direct Keys**:

| Feature | Provider | Why Direct Required |
|---|---|---|
| Predicted Outputs | OpenAI | `prediction` parameter not in OpenAI-compatible proxy spec |
| Responses API (stateful) | OpenAI | `previous_response_id` requires OpenAI's stateful server |
| Explicit context caching | Google | `client.caches.create()` requires Gemini SDK |
| Custom search grounding | Google | `externalApi` tool format is Gemini-specific |
| Batch API (50% discount) | Anthropic / OpenAI | Async batch processing, provider-native |
| Fast mode | Anthropic | Research preview beta header |
| Local inference | Ollama / vLLM | Localhost, no network |

**Configuration**:
```typescript
{
  directKeys: {
    anthropic: { apiKey: "sk-ant-..." },
    openai: { apiKey: "sk-..." },
    google: { apiKey: "AIza..." },
    deepseek: { apiKey: "sk-..." },
    local: { baseUrl: "http://localhost:11434/v1" },  // Ollama
    // Any OpenAI-compatible endpoint
    [key: string]: { apiKey?: string; baseUrl: string },
  }
}
```

**When Bardo Inference is also configured**: Direct key requests still pass through Bardo Inference's context engineering pipeline -- caching, compression, and tool pruning apply before the request reaches the direct provider endpoint. The optimization is free; only the direct provider charges apply (no Bardo Inference spread on direct key requests).

---

## 3. Full Configuration Schema

```typescript
interface GolemInferenceConfig {
  /**
   * Bardo Inference -- optional but recommended.
   * Provides context engineering, BlockRun backbone, and multi-backend routing.
   */
  bardoInference?: {
    enabled: boolean;
    payment:
      | { type: "x402"; walletKey: string }
      | { type: "prepaid"; apiKey: string };
  };

  /**
   * Venice -- optional. Private cognition with zero data retention.
   */
  venice?: {
    enabled: boolean;
    apiKey: string;
    staking?: { enabled: boolean };
  };

  /**
   * Bankr -- optional. Self-funding economics with on-chain execution.
   */
  bankr?: {
    enabled: boolean;
    apiKey: string;
    walletId: string;
  };

  /**
   * Direct API keys -- optional. Raw provider access for native features.
   */
  directKeys?: {
    anthropic?: { apiKey: string };
    openai?: { apiKey: string };
    google?: { apiKey: string };
    deepseek?: { apiKey: string };
    local?: { baseUrl: string };
    [key: string]: { apiKey?: string; baseUrl: string } | undefined;
  };

  /**
   * Provider priority order.
   * When multiple providers can serve a request, try in this order.
   * Default: ["bardoInference", "venice", "bankr", "directKeys"]
   */
  providerPriority?: ("bardoInference" | "venice" | "bankr" | "directKeys")[];

  /**
   * Who pays for autonomous inference (heartbeats, dreams, curator cycles).
   * Configurable per Golem.
   */
  autonomousPayment:
    | { type: "golem_wallet"; walletKey: string; budgetUsd: number }
    | { type: "prepaid_balance" }       // Draws from bardoInference.payment
    | { type: "bankr_self_funding" }    // Revenue from strategies covers cost
    | { type: "venice_diem" }           // Staked VVV covers Venice-routed calls
    | { type: "composite"; primary: string; fallback: string };

  /**
   * Global cost sensitivity (0-1). Higher = more cost-sensitive routing.
   * Overridden by mortality pressure at runtime.
   */
  costSensitivity?: number;

  /**
   * Optional hosted services.
   */
  services?: {
    oracle?: { mode: "local" | "hosted"; endpoint?: string; apiKey?: string };
    crypt?: { enabled: boolean };
  };
}
```

---

## 4. What Each Configuration Unlocks

### 4.1 Capability Matrix

| Capability | Bardo Inference Only | Venice Only | Bankr Only | Direct Anthropic Only | Full Stack |
|---|---|---|---|---|---|
| Claude (Opus/Sonnet/Haiku) | yes via BlockRun | no | yes | yes | yes |
| GPT (5.x) | yes via BlockRun | no | yes | no | yes |
| Gemini (3.x) | yes via BlockRun | no | yes | no (needs Google key) | yes |
| DeepSeek R1 | yes via BlockRun | yes | no | yes (needs DS key) | yes |
| Qwen 3.x | yes via BlockRun | no | no | yes (needs Alibaba key) | yes |
| Context engineering (8 layers) | yes | no | no | no | yes |
| Prompt cache alignment | yes | no | no | no | yes |
| Semantic/hash caching | yes | no | no | no | yes |
| Tool pruning (meta-tool) | yes | no | no | no | yes |
| Zero data retention | no (needs Venice on BI) | yes | no | no | yes |
| Private visible reasoning | no | yes (R1) | no | no | yes |
| DIEM (zero-cost inference) | no | yes | no | no | yes |
| Self-funding economics | no | no | yes | no | yes |
| Cross-model verification | no | no | yes | no | yes |
| On-chain execution | no | no | yes | no | yes |
| Anthropic Citations | yes (Claude via BR) | no | yes (Claude) | yes | yes |
| Anthropic Compaction | yes (Claude via BR) | no | yes (Claude) | yes | yes |
| Anthropic adaptive thinking | yes | no | yes | yes | yes |
| OpenAI Predicted Outputs | no | no | no | yes (OpenAI key) | yes |
| Gemini explicit caching | no | no | no | yes (Google key) | yes |
| Gemini custom grounding | no | no | no | yes (Google key + hosted Oracle) | yes |
| Local inference (Ollama) | no | no | no | yes | yes |
| Token launching at death | no | no | yes (+ Crypt) | no | yes |

### 4.2 Minimum Viable Configurations

**Cheapest possible** -- Venice with DIEM staking:
```typescript
{ venice: { enabled: true, apiKey: "...", staking: { enabled: true } },
  autonomousPayment: { type: "venice_diem" } }
```
- Cost: $0/day (within DIEM allocation)
- Models: DeepSeek R1, Llama 3.3, GLM-4.7, Qwen 2.5 VL
- Limitations: No Claude, no GPT, no citations, no compaction

**Best value** -- Bardo Inference (BlockRun):
```typescript
{ bardoInference: { enabled: true, payment: { type: "prepaid", apiKey: "bardo_sk_..." } },
  autonomousPayment: { type: "prepaid_balance" } }
```
- Cost: ~$1.50-$2.50/day (with context engineering savings)
- Models: All 30+ on BlockRun (Claude, GPT, Gemini, DeepSeek, Qwen, Grok)
- Limitations: No privacy, no self-funding, no Direct-Key-only features

**Self-sustaining** -- Bankr:
```typescript
{ bankr: { enabled: true, apiKey: "...", walletId: "0x..." },
  autonomousPayment: { type: "bankr_self_funding" } }
```
- Cost: Net $0 if revenue > cost
- Models: Claude, GPT, Gemini, DeepSeek via Bankr
- Limitations: No privacy, no context engineering

**Full stack** -- everything:
```typescript
{ bardoInference: { enabled: true, payment: { type: "prepaid", apiKey: "..." } },
  venice: { enabled: true, apiKey: "...", staking: { enabled: true } },
  bankr: { enabled: true, apiKey: "...", walletId: "0x..." },
  directKeys: { anthropic: { apiKey: "..." }, openai: { apiKey: "..." }, google: { apiKey: "..." } },
  providerPriority: ["bardoInference", "venice", "bankr", "directKeys"],
  autonomousPayment: { type: "composite", primary: "bankr_self_funding", fallback: "venice_diem" } }
```
- Cost: ~$0.50-$1.50/day (DIEM covers dreams, self-funding covers operator-facing, context engineering reduces everything else)
- Models: Everything
- Capabilities: Everything

---

## 5. How the Pi Runtime Resolves Capabilities

### 5.1 Capability Detection at Golem Boot

When a Golem starts, the `bardo-provider-adapter` extension scans the configured providers and builds a **capability map**:

```typescript
interface CapabilityMap {
  models: Map<string, ProviderSource[]>;        // model -> which providers have it
  features: Map<ProviderFeature, ProviderSource[]>; // feature -> which providers have it
  securityClasses: Set<SecurityClass>;          // what security levels are available
  hasContextEngineering: boolean;               // Bardo Inference configured?
  hasSelfFunding: boolean;                      // Bankr configured?
  hasPrivacy: boolean;                          // Venice configured?
  paymentMethods: PaymentMethod[];              // How autonomous inference is funded
}

// Built at boot, refreshed on config change
function buildCapabilityMap(config: GolemInferenceConfig): CapabilityMap {
  const map: CapabilityMap = { /* ... */ };

  if (config.bardoInference?.enabled) {
    // Bardo Inference exposes all BlockRun models + any additional backends
    // The Golem doesn't know which backends are configured on the BI instance --
    // it discovers available models by querying the /v1/models endpoint
    const biModels = await fetch(`${BI_ENDPOINT}/v1/models`, { headers: biAuth });
    for (const model of biModels) {
      map.models.set(model.id, [...(map.models.get(model.id) ?? []), "bardoInference"]);
    }
    map.hasContextEngineering = true;

    // Features available through Bardo Inference depend on which backends
    // the operator has configured. The Golem queries capabilities:
    const biCaps = await fetch(`${BI_ENDPOINT}/v1/capabilities`, { headers: biAuth });
    for (const feature of biCaps.features) {
      map.features.set(feature, [...(map.features.get(feature) ?? []), "bardoInference"]);
    }
  }

  if (config.venice?.enabled) {
    map.models.set("deepseek-r1", [...(map.models.get("deepseek-r1") ?? []), "venice"]);
    map.models.set("llama-3.3-70b", [...(map.models.get("llama-3.3-70b") ?? []), "venice"]);
    map.models.set("glm-4.7", [...(map.models.get("glm-4.7") ?? []), "venice"]);
    map.models.set("qwen-2.5-vl-72b", [...(map.models.get("qwen-2.5-vl-72b") ?? []), "venice"]);
    map.features.set("zero_data_retention", ["venice"]);
    map.features.set("visible_thinking", [...(map.features.get("visible_thinking") ?? []), "venice"]);
    map.features.set("diem_staking", ["venice"]);
    map.hasPrivacy = true;
  }

  if (config.bankr?.enabled) {
    // Query Bankr for available models
    const bankrModels = await fetch("https://llm.bankr.bot/v1/models", { headers: bankrAuth });
    for (const model of bankrModels) {
      map.models.set(model.id, [...(map.models.get(model.id) ?? []), "bankr"]);
    }
    map.features.set("self_funding", ["bankr"]);
    map.features.set("cross_model_verification", ["bankr"]);
    map.features.set("onchain_execution", ["bankr"]);
    map.hasSelfFunding = true;
  }

  if (config.directKeys) {
    for (const [provider, keyConfig] of Object.entries(config.directKeys)) {
      // Direct keys unlock provider-native features
      if (provider === "anthropic") {
        map.features.set("predicted_outputs", []); // No -- that's OpenAI
        map.features.set("batch_api", [...(map.features.get("batch_api") ?? []), "direct_anthropic"]);
        map.features.set("anthropic_citations", [...(map.features.get("anthropic_citations") ?? []), "direct_anthropic"]);
        // ... etc
      }
      if (provider === "openai") {
        map.features.set("predicted_outputs", ["direct_openai"]);
        map.features.set("responses_api_stateful", ["direct_openai"]);
      }
      if (provider === "google") {
        map.features.set("gemini_explicit_caching", ["direct_google"]);
        map.features.set("gemini_custom_grounding", ["direct_google"]);
      }
      if (provider === "local") {
        map.features.set("local_inference", ["direct_local"]);
      }
    }
  }

  return map;
}
```

### 5.2 Subsystem Adaptation

Every Pi subsystem queries the capability map at runtime and adapts its behavior:

```typescript
// Example: Dream subsystem checks what's available
function configureDreamInference(caps: CapabilityMap): DreamInferenceConfig {
  // Best case: Venice R1 (visible thinking + privacy + DIEM)
  if (caps.hasPrivacy && caps.models.has("deepseek-r1") && caps.features.has("diem_staking")) {
    return { provider: "venice", model: "deepseek-r1", reasoning: "visible", private: true };
  }
  // Good case: Any R1 (visible thinking, not private)
  if (caps.models.has("deepseek-r1")) {
    return { provider: caps.models.get("deepseek-r1")![0], model: "deepseek-r1", reasoning: "visible", private: false };
  }
  // Fallback: Claude with adaptive thinking (summarized, not visible)
  if (caps.models.has("claude-opus-4-6")) {
    return { provider: caps.models.get("claude-opus-4-6")![0], model: "claude-opus-4-6", reasoning: "summarized", private: false };
  }
  // Last resort: whatever is available with maximum reasoning
  const bestModel = selectBestAvailableModel(caps, { reasoning: "any", quality: "maximum" });
  return { provider: bestModel.provider, model: bestModel.id, reasoning: "opaque", private: false };
}
```

This pattern repeats for every subsystem. See `prd2-model-routing.md` section 2 for the full subsystem -> requirement -> resolution mapping.

### 5.3 Request Routing Without Bardo Inference

When Bardo Inference is **not** configured, the Pi runtime's `bardo-provider-adapter` extension handles routing directly:

```typescript
// Without Bardo Inference: Pi routes directly to configured providers
async function routeWithoutBI(
  request: InferenceRequest,
  config: GolemInferenceConfig,
  caps: CapabilityMap,
): Promise<InferenceResponse> {
  const priority = config.providerPriority ?? ["venice", "bankr", "directKeys"];

  // 1. Determine requirements from request metadata
  const requirements = extractRequirements(request);

  // 2. Filter providers that satisfy requirements
  let candidates = priority.filter(p => {
    if (requirements.private && p !== "venice") return false;
    if (requirements.selfFunding && p !== "bankr") return false;
    for (const feature of requirements.requiredFeatures) {
      if (!caps.features.get(feature)?.some(src => src.startsWith(p))) return false;
    }
    return caps.models.get(request.model)?.some(src => src.startsWith(p)) ?? false;
  });

  // 3. Try in priority order with fallback
  for (const provider of candidates) {
    try {
      return await callProvider(provider, request, config);
    } catch (err) {
      if (isRetryable(err)) continue;
      throw err;
    }
  }

  throw new Error(`No provider can serve: model=${request.model}, features=${requirements.requiredFeatures}`);
}
```

**What the Golem loses without Bardo Inference**:
- No context engineering (caching, compression, tool pruning, PII masking, injection detection)
- No BlockRun access (x402 backbone)
- No automatic multi-backend failover within a single endpoint
- No semantic caching across requests
- No prompt cache alignment optimization

**What the Golem keeps without Bardo Inference**:
- Smart subsystem -> model routing (handled by Pi runtime)
- Feature detection and capability adaptation
- Provider fallback (within configured providers)
- All Pi lifecycle hooks and subsystem behavior

---

## 6. Payment for Autonomous Inference

### 6.1 The Problem

A Golem runs autonomously -- heartbeats fire every 60 seconds, dreams run nightly, curator cycles consolidate knowledge. Each of these makes inference calls. Someone has to pay.

### 6.2 Payment Methods

| Method | How It Works | Best For |
|---|---|---|
| **Golem wallet** | User funds a wallet at creation. The Golem draws from it per-request. When depleted, inference stops (graceful degradation, not crash). | Users who want fixed budgets |
| **Prepaid balance** | Draws from the user's Bardo Inference prepaid USDC balance. Shared across all the user's Golems. | Users running multiple Golems on Bardo Inference |
| **Bankr self-funding** | Revenue from DeFi strategies covers inference cost. When sustainability ratio > 1.0, the Golem is self-sustaining. Below 1.0, budgets contract. | Golems with active trading strategies |
| **Venice DIEM** | Staked VVV provides daily inference allocation. Venice-routed calls are free within DIEM cap. | Users who hold VVV and want zero-cost private inference |
| **Composite** | Primary payment method with a fallback. E.g., Bankr self-funding as primary, Venice DIEM as fallback when revenue is low. | Full-stack configurations |

### 6.3 Budget Enforcement

```typescript
interface GolemBudget {
  dailyLimitUsd: number;        // Max spend per day
  subsystemAllocations: {
    heartbeat: number;          // % of daily budget
    risk: number;               // % -- never reduced below minimum
    dream: number;              // %
    daimon: number;             // %
    context: number;            // %
    curator: number;            // %
    playbook: number;           // %
    operator: number;           // %
    death: typeof Infinity;     // Always fully funded
  };
  mortalityPressure: number;    // 0-1, derived from vitality score
  // Subsystem allocations contract under mortality pressure
  // Risk and death are exempt from contraction
}
```

When the daily budget is exhausted:
1. Non-critical subsystems degrade (dreams skip, daimon falls back to deterministic rules)
2. Risk assessment is **never** degraded
3. Death reflection is **never** degraded
4. The Golem emits a `budget:exhausted` event to the operator

---

## 7. Configuration Examples

### 7.1 Solo Builder (Minimum Cost)

```typescript
const config: GolemInferenceConfig = {
  venice: { enabled: true, apiKey: "vn_...", staking: { enabled: true } },
  directKeys: { local: { baseUrl: "http://localhost:11434/v1" } },
  providerPriority: ["venice", "directKeys"],
  autonomousPayment: { type: "venice_diem" },
  costSensitivity: 0.9,
};
// T0 heartbeat -> local Ollama (free)
// Dreams -> Venice R1 (DIEM, free)
// Risk -> Venice Llama 3.3 (DIEM, free, no interleaved thinking -- degraded but functional)
// Daily cost: $0
```

### 7.2 Serious Operator (Best Quality)

```typescript
const config: GolemInferenceConfig = {
  bardoInference: { enabled: true, payment: { type: "prepaid", apiKey: "bardo_sk_..." } },
  venice: { enabled: true, apiKey: "vn_...", staking: { enabled: true } },
  directKeys: { anthropic: { apiKey: "sk-ant-..." }, openai: { apiKey: "sk-..." } },
  providerPriority: ["bardoInference", "venice", "directKeys"],
  autonomousPayment: { type: "prepaid_balance" },
  costSensitivity: 0.3,
};
// Context engineering ON. Best models per subsystem.
// Dreams -> Venice R1 (private, visible, DIEM).
// Risk -> Claude Opus via BI/BlockRun (interleaved thinking, citations).
// PLAYBOOK -> Direct OpenAI (Predicted Outputs, 3x speed).
// Daily cost: ~$1.50
```

### 7.3 Self-Sustaining Golem

```typescript
const config: GolemInferenceConfig = {
  bardoInference: { enabled: true, payment: { type: "prepaid", apiKey: "bardo_sk_..." } },
  bankr: { enabled: true, apiKey: "bankr_...", walletId: "0x..." },
  venice: { enabled: true, apiKey: "vn_...", staking: { enabled: true } },
  providerPriority: ["bardoInference", "bankr", "venice"],
  autonomousPayment: { type: "composite", primary: "bankr_self_funding", fallback: "venice_diem" },
  costSensitivity: 0.5,
};
// Revenue funds inference. DIEM covers overflow. Context engineering reduces cost.
// When sustainability ratio > 2.0, can afford premium models.
// When ratio < 0.5, aggressive cost reduction (except risk + death).
// Daily cost: Net $0 when self-sustaining
```

---

## References

- [BLOCKRUN-SDK-2026] BlockRun. "TypeScript SDK." GitHub. https://github.com/BlockRunAI/blockrun-llm-ts
- [VENICE-API-2025] Venice.ai. API Documentation. https://docs.venice.ai/
- [BANKR-LLM-2026] Bankr. "LLM Gateway." Bankr Docs. https://docs.bankr.bot/llm-gateway/overview
- [X402-STATE-2025] bc1beat. "The State of x402." December 2025.
- [ANTHROPIC-CONTEXT-ENG-2025] Anthropic. "Context Engineering." Blog post, 2025.
