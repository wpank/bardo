# 02 -- Venice Private Cognition [SPEC]

**Zero-Retention Inference, DIEM Staking, and the Private Reasoning Plane**

> **Version**: 2.0.0 | **Status**: Draft
>
> **Crate**: `bardo-venice` (extension of `bardo-providers`)
>
> **Depends on**: [../12-inference/12-providers.md](../12-inference/12-providers.md) (provider trait, resolution), [../12-inference/11-privacy-trust.md](../12-inference/11-privacy-trust.md) (security classes, classifier), [../12-inference/13-reasoning.md](../12-inference/13-reasoning.md) (reasoning chain normalization), [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) (three clocks, vitality), [../03-daimon](../03-daimon) (emotional appraisal)

---

> **Reader orientation:** This document specifies how the Golem (a mortal autonomous DeFi agent) uses Venice for zero-retention private inference. It belongs to the **integrations layer** and covers inference security classes, DIEM staking for throughput, model routing by survival pressure, and how mortality integrates with inference provider selection. You should understand LLM inference APIs, TEE-based privacy guarantees, and token staking mechanics. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Why Venice exists in this system

Venice is not a fallback provider. It is not a cost optimization. It is a separate inference plane where the Golem can think without being observed.

Every other provider in the Bardo stack retains something. BlockRun sees the request body (even though x402 (micropayment protocol using signed USDC transfers) payment carries no account identity). OpenRouter stores prompts for abuse monitoring. Direct Key mode sends raw context to the provider's native API. Bankr delegates to other providers that retain. Venice retains nothing. The inference logs do not exist after response delivery.

This matters because a DeFi agent's reasoning IS its alpha. A provider that sees "I plan to sell 500 ETH at 3:00 PM through a V4 pool on Base with 0.3% slippage tolerance" has the information to front-run the trade. A provider that sees "My anxiety score is 0.73 and I'm considering narrowing my LP range" knows the agent is vulnerable. Venice eliminates this attack surface structurally, not through policy.

See [../12-inference/11-privacy-trust.md](../12-inference/11-privacy-trust.md) Section 2 for the full privacy argument. This document specifies the Venice-specific integration: provider configuration, security-class routing, DIEM staking, model chain, mortality integration, and the `bardo-venice` extension.

---

## 1. Three inference security classes

Every inference request falls into one of three security classes. The classification is deterministic (no LLM call) and happens in the `bardo-context` Pi (the Bardo runtime framework) extension before provider resolution.

| Class | Data retention | Providers | When |
|---|---|---|---|
| **Standard** | Provider retains for training/abuse monitoring | BlockRun, OpenRouter, Bankr, Direct Key | Routine market commentary, public analysis |
| **Confidential** | Billing/audit only, no training | BlockRun (x402, no account), select OpenRouter models | Portfolio-specific analysis with position data |
| **Private** | Zero retention. Provider cannot reconstruct the query. | Venice only | Treasury reasoning, negotiation, governance, MEV-sensitive execution, death reflection |

When the classifier returns `Private`, the router hard-filters to Venice. If Venice is not configured, the gateway returns HTTP 503 with `X-Bardo-Error: private-inference-required` rather than falling back to a retaining provider. Degradation is never silent.

### SecurityLevel and SecurityTrigger

```rust
// crates/bardo-safety/src/security_class.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    Standard,
    Confidential,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityTrigger {
    /// Request contains specific asset amounts above $1,000.
    PortfolioComposition,
    /// Request discusses execution timing for pending swaps/rebalances.
    RebalanceTiming,
    /// Inter-agent commercial discussion (reserve prices, walk-away conditions).
    DealNegotiation,
    /// Governance proposal analysis with position exposure data.
    GovernanceDeliberation,
    /// Execution timing that could be front-run (pending actions > $500).
    MevSensitive,
    /// Evaluating another agent's behavioral patterns or weaknesses.
    CounterpartyAnalysis,
    /// Terminal phase reasoning -- most honest, most sensitive.
    DeathReflection,
    /// Owner-identifying information present in context.
    OwnerPii,
}
```

### Security-class router

The router classifies requests and forces Venice routing when private:

```rust
// crates/bardo-venice/src/router.rs

pub fn classify_and_route(
    context: &ContextBundle,
    phase: BehavioralPhase,
    intent: &mut Intent,
) -> ClassificationResult {
    let classification = classify_security_class(context, phase);

    match classification.class {
        SecurityLevel::Private => {
            // Hard requirement: only Venice can handle private inference
            intent.require.push("privacy".to_string());
            intent.prefer.retain(|p| p != "privacy"); // Don't double-count
        }
        SecurityLevel::Confidential => {
            intent.prefer.push("privacy".to_string());
        }
        SecurityLevel::Standard => {}
    }

    classification
}

/// Classification result includes the level, human-readable reason,
/// and all triggers that fired.
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub class: SecurityLevel,
    pub reason: String,
    pub triggers: Vec<SecurityTrigger>,
}
```

---

## 2. Venice provider chain

Venice hosts open-weight models. The Golem selects among them based on tier routing:

| Tier | Model | Context | Cost | Use case |
|---|---|---|---|---|
| **T0** | None | -- | $0.00 | FSM rules, no LLM call |
| **T1** | Llama 3.3 70B | 128K | ~$0.001/call | Routine private analysis, daimon appraisal, quick classification |
| **T2 reasoning** | DeepSeek R1 671B | 64K | ~$0.01/call | Deep reasoning with visible `<think>` tags, dream cycles, death reflection |
| **T2 general** | GLM 4.7 128K | 128K | ~$0.005/call | Long-context private analysis, tool use, structured outputs |

### Provider configuration

```rust
// crates/bardo-venice/src/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceProviderConfig {
    /// Venice API base URL.
    pub base_url: String,  // "https://api.venice.ai/api/v1"
    /// API key for Venice inference.
    pub api_key: String,
    /// Model mapping for tier routing.
    pub models: VeniceModelMapping,
    /// Venice-specific parameters (web search, thinking tags).
    pub venice_parameters: VeniceParameters,
    /// Daily spending cap in USD (safety limit).
    pub daily_cap_usd: f64,
    /// DIEM staking config (optional -- owner must stake VVV first).
    pub diem: Option<DiemConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceModelMapping {
    /// Fast, cheap private analysis. Default: "llama-3.3-70b".
    pub t1: String,
    /// Deep reasoning with visible <think> tags. Default: "deepseek-ai-DeepSeek-R1".
    pub t2_reasoning: String,
    /// Frontier general-purpose. Default: "zai-org-glm-4.7".
    pub t2_general: String,
    /// Vision model for private chart/image analysis.
    pub vision: String,  // "qwen-2.5-vl-72b"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceParameters {
    /// Enable Venice web search for grounding.
    pub enable_web_search: Option<WebSearchMode>,
    /// Controls whether R1's <think> tags appear in responses.
    /// false for dreams and death reflection (preserve full chain).
    /// true for routine operations (smaller, faster).
    pub strip_thinking_response: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchMode {
    Off,
    Auto,
    Always,
}
```

### Provider trait implementation

```rust
// crates/bardo-venice/src/provider.rs

pub struct VeniceProvider {
    config: VeniceProviderConfig,
    client: reqwest::Client,
    diem_tracker: Option<DiemTracker>,
}

impl Provider for VeniceProvider {
    fn id(&self) -> &str { "venice" }
    fn name(&self) -> &str { "Venice Private Cognition" }

    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        // Venice handles privacy-required intents
        if intent.require.contains(&"privacy".to_string()) {
            let model = self.select_model(intent);
            return Some(Resolution {
                model: model.clone(),
                provider: "venice".to_string(),
                estimated_cost_usd: self.estimate_cost(&model, intent),
                features: vec!["privacy".into(), "visible_thinking".into()],
                degraded: self.compute_degraded(intent),
            });
        }

        // Venice also handles soft privacy preference if DIEM is available
        if intent.prefer.contains(&"privacy".to_string()) && intent.diem_available {
            let model = self.select_model(intent);
            return Some(Resolution {
                model,
                provider: "venice".to_string(),
                estimated_cost_usd: 0.0, // DIEM-funded
                features: vec!["privacy".into()],
                degraded: vec![],
            });
        }

        None
    }

    fn traits(&self) -> &ProviderTraits {
        &ProviderTraits {
            private: true,
            self_funding: false,
            context_engineering: true,
            payment: PaymentMode::Diem,
        }
    }
}

impl VeniceProvider {
    fn select_model(&self, intent: &Intent) -> String {
        // Explicit model request takes priority
        if let Some(ref model) = intent.model {
            return model.clone();
        }

        match intent.quality {
            Quality::Minimum | Quality::Low => self.config.models.t1.clone(),
            Quality::Medium => self.config.models.t2_general.clone(),
            Quality::High | Quality::Maximum => {
                if intent.require.contains(&"visible_thinking".to_string())
                    || intent.prefer.contains(&"visible_thinking".to_string())
                {
                    self.config.models.t2_reasoning.clone()
                } else {
                    self.config.models.t2_general.clone()
                }
            }
        }
    }

    fn estimate_cost(&self, model: &str, _intent: &Intent) -> f64 {
        if self.diem_tracker.as_ref().map_or(false, |d| d.has_balance()) {
            return 0.0; // DIEM-funded
        }
        match model {
            m if m.contains("llama") => 0.001,
            m if m.contains("DeepSeek-R1") => 0.01,
            m if m.contains("glm") => 0.005,
            m if m.contains("qwen") && m.contains("vl") => 0.008,
            _ => 0.005,
        }
    }
}
```

---

## 3. DIEM staking: zero-cost private inference

Venice's tokenomics create a unique path: the owner stakes VVV tokens on Base, earns daily DIEM allocation, and the Golem consumes DIEM for inference at zero marginal cost. The staking yield funds thinking itself.

### Mechanism

1. Owner stakes VVV (Venice's native token) on Base
2. Staked VVV earns pro-rata daily DIEM allocation
3. Each DIEM = $1/day of Venice API credit, perpetually
4. Golem consumes DIEM for private inference with no per-request payment
5. Excess DIEM rolls over or transfers to successor Golems

```rust
// crates/bardo-venice/src/diem.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiemConfig {
    /// VVV token contract on Base.
    pub vvv_token: Address,
    /// Amount of VVV staked by the owner.
    pub staked_vvv: U256,
    /// Daily DIEM allocation (computed from pro-rata stake share).
    pub daily_diem_allocation: f64,
}

#[derive(Debug)]
pub struct DiemTracker {
    config: DiemConfig,
    /// DIEM consumed today.
    consumed_today: f64,
    /// DIEM allocated to each budget category.
    allocations: DiemAllocations,
    /// Unused DIEM carried from previous days.
    rollover_balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiemAllocations {
    /// Waking inference (portfolio analysis, deal negotiation): 60%.
    pub waking_fraction: f64,
    /// Dream cycles (always private): 15%.
    pub dream_fraction: f64,
    /// Sleepwalker artifacts (observatory research): 15%.
    pub sleepwalker_fraction: f64,
    /// Reserve (rolls over to next day): 10%.
    pub rollover_fraction: f64,
}

impl Default for DiemAllocations {
    fn default() -> Self {
        Self {
            waking_fraction: 0.60,
            dream_fraction: 0.15,
            sleepwalker_fraction: 0.15,
            rollover_fraction: 0.10,
        }
    }
}
```

### DIEM allocation

```
Daily DIEM Budget: $X (from VVV stake)
+-- Waking inference (private):  60%  -- Portfolio analysis, deal negotiation, governance
+-- Dream cycles (always private): 15%  -- Counterfactual reasoning, threat simulation
+-- Sleepwalker artifacts:       15%  -- Observatory research (if phenotype=sleepwalker)
+-- Reserve (rollover):          10%  -- Unused DIEM for volatile days or successor transfer
```

Dreams are always private. There is no mode where dream content should flow through a retaining provider. The dream allocation is a hard floor, not a soft suggestion. If waking inference consumes its budget before end of day, dreams still run.

### DIEM budget tracking

```rust
// crates/bardo-venice/src/diem.rs

impl DiemTracker {
    pub fn has_balance(&self) -> bool {
        self.remaining_today() > 0.0
    }

    pub fn remaining_today(&self) -> f64 {
        let total = self.config.daily_diem_allocation + self.rollover_balance;
        (total - self.consumed_today).max(0.0)
    }

    pub fn remaining_for_category(&self, category: DiemCategory) -> f64 {
        let fraction = match category {
            DiemCategory::Waking => self.allocations.waking_fraction,
            DiemCategory::Dream => self.allocations.dream_fraction,
            DiemCategory::Sleepwalker => self.allocations.sleepwalker_fraction,
            DiemCategory::Reserve => self.allocations.rollover_fraction,
        };
        let category_budget = self.config.daily_diem_allocation * fraction;
        (category_budget - self.consumed_for_category(category)).max(0.0)
    }

    /// Record inference consumption. Returns true if within budget.
    pub fn consume(&mut self, cost_usd: f64, category: DiemCategory) -> bool {
        if cost_usd > self.remaining_for_category(category) {
            return false;
        }
        self.consumed_today += cost_usd;
        self.consumption_by_category.entry(category).or_default().add(cost_usd);
        true
    }

    /// End-of-day rollover. Reserve fraction carries forward.
    pub fn end_of_day(&mut self) {
        let reserve_budget = self.config.daily_diem_allocation * self.allocations.rollover_fraction;
        let reserve_unused = reserve_budget - self.consumed_for_category(DiemCategory::Reserve);
        self.rollover_balance = reserve_unused.max(0.0);
        self.consumed_today = 0.0;
        self.consumption_by_category.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiemCategory {
    Waking,
    Dream,
    Sleepwalker,
    Reserve,
}
```

---

## 4. Mortality integration

DIEM staking decouples inference cost from the economic mortality clock. A Golem routing 50% of its inference through Venice saves $0.06-0.10/day on a $0.20/day inference budget. Over a 30-day lifespan, that is 9 additional days of life, purchased through the owner's VVV stake rather than through more USDC.

### Lifespan extension computation

```rust
// crates/bardo-venice/src/mortality.rs

#[derive(Debug, Clone, Serialize)]
pub struct LifespanExtension {
    /// Days of additional life from Venice routing.
    pub extension_days: f64,
    /// Hours of additional life from Venice routing.
    pub extension_hours: f64,
    /// Fraction of inference routed through Venice.
    pub venice_fraction: f64,
    /// Daily USDC savings.
    pub daily_savings_usd: f64,
}

pub fn compute_venice_lifespan_extension(
    daily_inference_cost_usd: f64,
    venice_fraction: f64,
    current_burn_rate_usd: f64,
    remaining_credits_usd: f64,
) -> LifespanExtension {
    let daily_savings = daily_inference_cost_usd * venice_fraction;
    let new_burn_rate = (current_burn_rate_usd - daily_savings).max(0.001);
    let original_days_remaining = remaining_credits_usd / current_burn_rate_usd;
    let new_days_remaining = remaining_credits_usd / new_burn_rate;
    let extension = new_days_remaining - original_days_remaining;

    LifespanExtension {
        extension_days: extension,
        extension_hours: extension * 24.0,
        venice_fraction,
        daily_savings_usd: daily_savings,
    }
}
```

| Inference source | Cost model | Mortality impact |
|---|---|---|
| BlockRun (x402) | Per-request USDC micropayment | Drains LLM partition, shortens lifespan |
| Venice (DIEM) | Zero marginal cost from staked VVV | No drain on LLM partition -- life extension |

A Golem with 100% Venice routing theoretically eliminates the inference component of the economic clock entirely. In practice, Venice's model catalog is narrower than BlockRun's, so some requests (especially those needing Anthropic-native features like prompt caching or citations) still route through BlockRun. The realistic ceiling is 40-60% Venice routing.

---

## 5. Private subsystem routing

Certain subsystems are always private. This is not configurable. It is architecture.

| Subsystem | Always private? | Venice model | Why |
|---|---|---|---|
| Dream cycles | Yes | DeepSeek R1 | Subconscious is unobservable. Counterfactual scenarios expose strategic intent. |
| Death reflection | Yes | DeepSeek R1 | Most honest reasoning. Unrealized hypotheses, abandoned strategies, raw uncertainty. |
| Daimon (the Golem's affect engine implementing PAD -- Pleasure-Arousal-Dominance -- emotional state) appraisal | Yes | Llama 3.3 70B | Emotional state = known vulnerability. Anxiety signals are exploitable. |
| Deal negotiation | Yes | GLM 4.7 | Reserve prices, walk-away conditions. Material non-public information. |
| Governance deliberation | Yes | GLM 4.7 | Vote intent + position exposure = insider trading intelligence. |
| Execution planning (>$500) | Yes | Llama 3.3 70B | Timing and routing = front-running opportunity. |
| Vision analysis | Yes | Qwen 2.5 VL 72B | Chart patterns, order book heatmaps, governance vote distributions. |
| Routine heartbeat | No | -- | Low-value, no position data. Routes through BlockRun. |

### Private negotiation protocol

Two Golems negotiating a cross-vault allocation each reason about strategy on Venice. Only structured offers transmit between them:

```rust
// crates/bardo-venice/src/negotiation.rs

/// Each party's private reasoning is invisible to the other.
/// Only the NegotiationOffer crosses the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationOffer {
    pub offer_id: String,
    pub from_agent: u128,
    pub to_agent: u128,
    /// Structured terms -- no reasoning exposed.
    pub terms: NegotiationTerms,
    /// Signature over terms by the offering agent.
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationTerms {
    pub allocation_pct: f64,
    pub duration_days: u32,
    pub fee_split_bps: u32,
    pub min_aum_usd: f64,
    pub expiry: u64,
}

/// Private reasoning that produced the offer. Stays on Venice,
/// never leaves the requesting Golem's local context.
pub struct NegotiationReasoning {
    pub reserve_price: f64,
    pub walk_away_threshold: f64,
    pub counterparty_assessment: String,
    pub risk_factors: Vec<String>,
    /// Full DeepSeek R1 <think> chain. Captured locally, forgotten by Venice.
    pub reasoning_chain: Option<String>,
}
```

### Private governance protocol

The analysis is private. The vote is public.

```rust
// crates/bardo-venice/src/governance.rs

/// Governance deliberation runs entirely on Venice.
/// The final vote is a public on-chain action with no reasoning attached.
pub struct GovernanceDeliberation {
    /// Private analysis of proposal impact on positions.
    pub position_exposure_analysis: String,
    /// Private assessment of vote alignment with strategy.
    pub strategic_alignment: f64,
    /// Private counterparty analysis (who else is voting, what do they hold).
    pub counterparty_signals: Vec<CounterpartySignal>,
    /// The public output: a vote direction with no reasoning exposed.
    pub vote_decision: VoteDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum VoteDirection { For, Against, Abstain }
```

### MEV-resistant execution planning

When a Golem plans a large swap, the planning phase runs on Venice. A sandwich bot monitoring inference providers sees nothing:

```rust
// crates/bardo-venice/src/mev.rs

/// Execution plan computed on Venice. The plan itself never leaves
/// the Golem's local context. Only the final transaction hits the chain.
pub struct PrivateExecutionPlan {
    /// Which route to take (V3 direct, V4 hook, UniswapX Dutch auction).
    pub route_strategy: RouteStrategy,
    /// When to execute (block number or timestamp target).
    pub timing: ExecutionTiming,
    /// Slippage tolerance computed from private analysis.
    pub slippage_bps: u32,
    /// Whether to use Flashbots Protect for additional MEV protection.
    pub use_flashbots: bool,
    /// Security classification that triggered private planning.
    pub security_triggers: Vec<SecurityTrigger>,
}
```

---

## 6. DeepSeek R1 thinking chain integration

DeepSeek R1 on Venice provides visible `<think>` tags with zero data retention. This is the only combination in the system that produces visible reasoning AND forgets it. The Golem captures the full chain locally; Venice discards it after response delivery.

### Thinking chain capture

```rust
// crates/bardo-venice/src/reasoning.rs

/// Captured R1 thinking chain from a Venice inference call.
/// Stored locally in the Grimoire (persistent knowledge base). Venice has already forgotten it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceThinkingChain {
    pub session_id: String,
    pub subsystem: String,
    pub model: String,
    /// Full <think>...</think> content, parsed by BardoStreamParser.
    pub thinking_content: String,
    /// Structured steps extracted from the raw thinking.
    pub steps: Vec<ReasoningStep>,
    /// Token count for the thinking portion.
    pub thinking_tokens: u32,
    /// Timestamp of capture.
    pub captured_at: u64,
    /// Security class that triggered Venice routing.
    pub security_class: SecurityLevel,
}
```

### Death reflection: the most honest moment

Death reflection always uses DeepSeek R1 on Venice. The Golem is dying. It has nothing left to protect. The reasoning is maximally honest because survival pressure is zero.

The `strip_thinking_response` parameter is set to `false` for death reflection. The full `<think>` chain is captured, including abandoned hypotheses, uncertainty expressions, and contradictions. This raw chain becomes part of the death testament in the Grimoire.

```rust
// crates/bardo-venice/src/death.rs

/// Death reflection inference configuration.
/// Always: Venice, DeepSeek R1, visible thinking, no stripping.
pub fn death_reflection_intent() -> Intent {
    Intent {
        model: Some("deepseek-ai-DeepSeek-R1".to_string()),
        require: vec!["privacy".into(), "visible_thinking".into()],
        prefer: vec![],
        quality: Quality::Maximum,
        max_latency_ms: 30_000, // No rush. Quality over speed.
        cost_sensitivity: 0.0,  // Spend whatever it takes.
        diem_available: true,
        subsystem: "death".to_string(),
    }
}
```

---

## 7. Emotional appraisal on Venice

The daimon engine performs emotional appraisal using the OCC/Scherer chain-of-emotion model. This runs on Venice because emotional state is a known vulnerability. An adversary who knows a Golem is anxious can exploit that signal.

```rust
// crates/bardo-venice/src/daimon.rs

/// Daimon appraisal runs on Venice (Llama 3.3 70B).
/// Fast, cheap, private. The appraisal result is stored locally
/// in the Golem's affective state; Venice forgets the input.
pub fn daimon_intent(complexity: DaimonComplexity) -> Intent {
    match complexity {
        DaimonComplexity::Simple => Intent {
            model: Some("llama-3.3-70b".to_string()),
            require: vec!["privacy".into()],
            prefer: vec![],
            quality: Quality::Low,
            max_latency_ms: 5_000,
            cost_sensitivity: 0.9,
            diem_available: true,
            subsystem: "daimon".to_string(),
        },
        DaimonComplexity::Complex => Intent {
            model: Some("deepseek-ai-DeepSeek-R1".to_string()),
            require: vec!["privacy".into(), "visible_thinking".into()],
            prefer: vec![],
            quality: Quality::High,
            max_latency_ms: 15_000,
            cost_sensitivity: 0.5,
            diem_available: true,
            subsystem: "daimon_complex".to_string(),
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DaimonComplexity { Simple, Complex }
```

---

## 8. Vision: private visual analysis

Venice serves Qwen 2.5 VL 72B for multimodal inference. The Golem sends chart images, order book heatmaps, and governance vote distributions to Venice for analysis. The provider never sees what chart the Golem analyzed or what it concluded.

Use cases:
- Order book heatmap analysis for hidden liquidity walls
- TVL charts for regime shift identification before numerical data reflects them
- Governance vote distribution charts for whale detection
- Token price charts with pattern recognition

```rust
// crates/bardo-venice/src/vision.rs

pub fn vision_intent() -> Intent {
    Intent {
        model: Some("qwen-2.5-vl-72b".to_string()),
        require: vec!["privacy".into(), "vision".into()],
        prefer: vec![],
        quality: Quality::High,
        max_latency_ms: 20_000,
        cost_sensitivity: 0.5,
        diem_available: true,
        subsystem: "vision".to_string(),
    }
}
```

---

## 9. Extension: `bardo-venice`

The Venice integration ships as a runtime extension in the Golem's extension chain:

```rust
// crates/bardo-venice/src/extension.rs

pub struct VeniceExtension {
    provider: VeniceProvider,
    diem_tracker: DiemTracker,
}

impl Extension for VeniceExtension {
    fn name(&self) -> &str { "bardo-venice" }
    fn layer(&self) -> u8 { 4 } // After model-router (layer 3)

    async fn on_before_inference(&self, ctx: &mut InferenceCtx) -> Result<()> {
        // If Venice was selected, configure Venice-specific parameters
        if ctx.resolved_provider() != "venice" {
            return Ok(());
        }

        let subsystem = ctx.subsystem();

        // Dreams and death: preserve full thinking chain
        let strip_thinking = !matches!(subsystem, "dream" | "death" | "daimon_complex");

        ctx.set_provider_param("venice_parameters", json!({
            "strip_thinking_response": strip_thinking,
        }));

        // Track DIEM consumption
        let category = match subsystem {
            "dream" => DiemCategory::Dream,
            s if s.starts_with("sleepwalker") => DiemCategory::Sleepwalker,
            _ => DiemCategory::Waking,
        };
        let estimated_cost = ctx.estimated_cost_usd();
        if !self.diem_tracker.consume(estimated_cost, category) {
            return Err(anyhow!("DIEM budget exhausted for category {:?}", category));
        }

        Ok(())
    }

    async fn on_after_inference(&self, ctx: &mut InferenceCtx) -> Result<()> {
        if ctx.resolved_provider() != "venice" {
            return Ok(());
        }

        // Capture thinking chain locally if present
        if let Some(reasoning) = ctx.reasoning_chain() {
            if reasoning.visibility == ReasoningVisibility::Visible {
                let captured = VeniceThinkingChain {
                    session_id: ctx.session_id().to_string(),
                    subsystem: ctx.subsystem().to_string(),
                    model: reasoning.model.clone(),
                    thinking_content: reasoning.content.clone().unwrap_or_default(),
                    steps: reasoning.steps.clone(),
                    thinking_tokens: reasoning.reasoning_tokens,
                    captured_at: now_unix(),
                    security_class: ctx.security_class(),
                };
                ctx.emit(GolemEvent::VeniceThinkingCaptured {
                    subsystem: captured.subsystem.clone(),
                    tokens: captured.thinking_tokens,
                });
                ctx.store_in_grimoire(captured).await?;
            }
        }

        Ok(())
    }
}
```

---

## 10. Configuration

```bash
# Venice API
BARDO_VENICE_API_KEY=vk-...
BARDO_VENICE_BASE_URL=https://api.venice.ai/api/v1
BARDO_VENICE_DAILY_CAP_USD=5.00

# Model overrides (defaults shown)
BARDO_VENICE_T1_MODEL=llama-3.3-70b
BARDO_VENICE_T2_REASONING_MODEL=deepseek-ai-DeepSeek-R1
BARDO_VENICE_T2_GENERAL_MODEL=zai-org-glm-4.7
BARDO_VENICE_VISION_MODEL=qwen-2.5-vl-72b

# DIEM staking (optional)
BARDO_VVV_STAKED_AMOUNT=1000
BARDO_DIEM_WAKING_FRACTION=0.60
BARDO_DIEM_DREAM_FRACTION=0.15
BARDO_DIEM_SLEEPWALKER_FRACTION=0.15
BARDO_DIEM_ROLLOVER_FRACTION=0.10
```

---

## Cross-references

- [../12-inference/11-privacy-trust.md](../12-inference/11-privacy-trust.md) -- the full privacy argument: three security classes, deterministic classifier, and cryptographic audit trail that determine when Venice is required vs optional
- [../12-inference/12-providers.md](../12-inference/12-providers.md) -- the provider trait and five-provider resolution algorithm where Venice sits as the zero-retention private inference plane
- [../12-inference/13-reasoning.md](../12-inference/13-reasoning.md) -- reasoning chain normalization and the BardoStreamParser that handles Venice's `<think>` tag format alongside other providers
- [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) -- the three death clocks (economic, epistemic, stochastic) and vitality score that drive model routing by survival pressure
- [../03-daimon](../03-daimon) -- the OCC/Scherer emotional appraisal model whose PAD vectors influence which inference tier the Golem selects under stress
- [../05-dreams](../05-dreams) -- dream cycles and REM counterfactual reasoning where Venice provides private inference for strategy mutations
- [03-bankr.md](03-bankr.md) -- Bankr self-funding gateway, a complementary provider where the wallet pays for inference; Venice handles privacy, Bankr handles economics
- [04-agentcash.md](04-agentcash.md) -- knowledge marketplace revenue that can extend lifespan independently of DIEM staking income
- [05-uniswap.md](05-uniswap.md) -- Uniswap DeFi execution where MEV-resistant execution planning from Venice-private reasoning feeds into the trading tool chain
