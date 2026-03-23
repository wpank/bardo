# Bardo Tools -- Intelligence [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Order flow intelligence, market context, MEV scoring, IL calculation, venue comparison, token discovery, and regime classification. Category: `intelligence`. 18 tools, all `CapabilityTier::Read`.

---

> **Reader orientation:** This document specifies 18 read-only intelligence tools in `bardo-tools`. Coverage spans order flow toxicity analysis (VPIN, LVR), MEV risk scoring, impermanent loss calculation, venue comparison, token discovery with risk scoring, and market regime classification. These tools give agents analytical capabilities beyond raw data reads. You should be familiar with LP profitability metrics, MEV dynamics, and basic market microstructure concepts. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Order flow intelligence (3 tools)

Tools for measuring order flow toxicity and LP profitability. Foundation of research-backed LP strategy [LVR-MILIONIS-2022].

### `intel_compute_vpin`

Compute Volume-Synchronized Probability of Informed Trading (VPIN) for a pool. VPIN measures order flow toxicity -- high VPIN means the pool is dominated by informed/arbitrage traders, making LP less profitable. Uses Bulk Volume Classification (BVC).

```rust
#[derive(Debug, Deserialize)]
pub struct ComputeVpinParams {
    /// Pool contract address (0x...).
    pub pool: String,
    /// Chain ID (default: 1).
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// Number of volume buckets (default: 50).
    #[serde(default = "default_buckets")]
    pub buckets: u32,
    /// Lookback window: "1h", "4h", "24h", "7d" (default: "24h").
    #[serde(default = "default_lookback_24h")]
    pub lookback: String,
}

#[derive(Debug, Serialize)]
pub struct VpinResult {
    pub vpin: f64,
    pub interpretation: String,
    /// SAFE (< 0.3), MODERATE (0.3-0.6), TOXIC (> 0.6).
    pub classification: VpinClassification,
    pub time_series: Vec<VpinDataPoint>,
    pub methodology: String,
}

#[derive(Debug, Serialize)]
pub enum VpinClassification { Safe, Moderate, Toxic }
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_compute_vpin",
    description: concat!(
        "Compute VPIN (Volume-Synchronized Probability of Informed Trading) for a pool. ",
        "Measures order flow toxicity using Bulk Volume Classification. ",
        "SAFE (< 0.3), MODERATE (0.3-0.6), TOXIC (> 0.6). ",
        "Use before LP deployments to assess toxicity. ",
        "Does NOT recommend position parameters -- use intel_get_order_flow_metrics for composite assessment.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,       // 1-3s depending on lookback
    progress_steps: &["Fetching swap events", "Running BVC", "Computing VPIN time series"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Full VPIN computation with time-series and BVC. SAFE (< 0.3), MODERATE (0.3-0.6), TOXIC (> 0.6). Use for research-grade toxicity assessment before LP deployments.",
    prompt_guidelines: &[
        "thriving: Run before large LP deployments (> $10K). Analyze time-series for toxicity trends.",
        "cautious: Run before any LP entry. Reject pools with VPIN trending upward toward TOXIC.",
        "declining: Run on existing LP pools to prioritize exits (exit TOXIC pools first).",
        "terminal: Not needed. Remove all liquidity regardless of toxicity.",
    ],
};
```

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components)**:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ tool_name: "intel_compute_vpin", params_hash, tick }` |
| `tool:update` | Steps: "Fetching swap events", "Running BVC", "Computing VPIN time series" |
| `tool:end` | `{ success, duration_ms, result_summary: "VPIN={vpin} ({classification})" }` |

### `intel_compute_lvr`

Compute Loss-Versus-Rebalancing (LVR) for a pool. LVR = sigma^2/8 per unit time, representing the theoretical cost of providing liquidity vs. a rebalancing portfolio. The fee/LVR ratio determines LP profitability: ratio > 1.0 means fees exceed arbitrage losses.

```rust
#[derive(Debug, Deserialize)]
pub struct ComputeLvrParams {
    pub pool: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// "1d", "7d", "30d" (default: "7d").
    #[serde(default = "default_lookback_7d")]
    pub lookback: String,
    /// "realized" or "implied" (default: "realized").
    #[serde(default = "default_method_realized")]
    pub method: String,
}

#[derive(Debug, Serialize)]
pub struct LvrResult {
    pub lvr_annualized_pct: f64,
    pub lvr_period_pct: f64,
    pub lvr_per_block_usd: f64,
    pub fee_revenue: f64,
    pub fee_lvr_ratio: f64,
    /// LVR is ~40% lower on Base vs. mainnet due to faster block times.
    pub l2_adjustment: Option<f64>,
    pub volatility: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_compute_lvr",
    description: concat!(
        "Compute Loss-Versus-Rebalancing (LVR): sigma^2/8 per unit time. ",
        "Fee/LVR ratio > 1.0 means LP is profitable. ",
        "Includes L2 adjustment (~40% lower LVR on Base vs. mainnet). ",
        "Primary LP profitability metric. ",
        "Does NOT include IL -- use intel_calculate_il for impermanent loss.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Fetching price history", "Computing volatility", "Calculating LVR", "L2 adjustment"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "LVR: sigma^2/8 per unit time. Fee/LVR ratio > 1.0 means LP is profitable. Includes L2 adjustment. Primary LP profitability metric.",
    prompt_guidelines: &[
        "thriving: Run before LP entry. Require fee/LVR ratio > 1.2.",
        "cautious: Require ratio > 1.5 for new positions. Monitor existing positions weekly.",
        "declining: Check existing positions. Exit where ratio < 0.8.",
        "terminal: Not needed. Remove all positions.",
    ],
};
```

### `intel_get_order_flow_metrics`

Composite order flow intelligence combining VPIN, LVR, markout analysis, and JIT liquidity detection. The definitive LP profitability assessment tool.

```rust
#[derive(Debug, Deserialize)]
pub struct OrderFlowMetricsParams {
    pub pool: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// "1d", "7d", "30d" (default: "7d").
    #[serde(default = "default_lookback_7d")]
    pub lookback: String,
}

#[derive(Debug, Serialize)]
pub struct OrderFlowMetrics {
    pub vpin: VpinResult,
    pub lvr: LvrResult,
    pub markout: MarkoutAnalysis,
    pub jit_liquidity: JitDetection,
    pub overall_assessment: LpAssessment,
}

#[derive(Debug, Serialize)]
pub struct MarkoutAnalysis {
    pub markout_5_block: f64,
    pub markout_20_block: f64,
    pub markout_100_block: f64,
}

#[derive(Debug, Serialize)]
pub struct LpAssessment {
    pub lp_profitability: String,
    pub confidence: f64,
    /// Hasbrouck covered-call criterion. True = LP is profitable.
    pub covered_call_criterion: bool,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_get_order_flow_metrics",
    description: concat!(
        "Composite intelligence: VPIN + LVR + markout + JIT detection + Hasbrouck covered-call criterion. ",
        "The definitive LP profitability assessment tool. ",
        "Call for any LP decision > $5K. ",
        "Subsumes intel_compute_vpin and intel_compute_lvr -- prefer this for comprehensive analysis.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Slow,         // 5-10s, runs VPIN + LVR + markout
    progress_steps: &["Computing VPIN", "Computing LVR", "Running markout analysis", "Detecting JIT", "Synthesizing assessment"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Composite intelligence: VPIN + LVR + markout + JIT + covered-call criterion. The definitive LP profitability assessment. Call for any LP decision > $5K.",
    prompt_guidelines: &[
        "thriving: Run for every LP pool under consideration. Require positive coveredCallCriterion.",
        "cautious: Run for all LP pools. Exit if markout turns negative at 20-block horizon.",
        "declining: Run on existing LP pools to prioritize exit order.",
        "terminal: Not needed.",
    ],
};
```

---

## IL and LP analytics (4 tools)

### `intel_calculate_il`

Calculate impermanent loss for a V3 concentrated liquidity position using the exact V3 IL formula. Supports backtest mode against historical price data.

```rust
#[derive(Debug, Deserialize)]
pub struct CalculateIlParams {
    pub pool: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// Price change as a decimal (e.g., 0.25 for +25%).
    pub price_change: f64,
    /// Lower tick of the position range.
    pub tick_lower: i32,
    /// Upper tick of the position range.
    pub tick_upper: i32,
    /// "scenario" or "backtest" (default: "scenario").
    #[serde(default = "default_mode_scenario")]
    pub mode: String,
    /// For backtest mode: lookback period.
    pub lookback: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IlResult {
    pub il_pct: f64,
    pub il_usd: f64,
    /// Fee revenue over the same period (backtest mode).
    pub fee_revenue_usd: Option<f64>,
    /// Net P&L after IL and fees (backtest mode).
    pub net_pnl_usd: Option<f64>,
    pub range_utilization: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_calculate_il",
    description: concat!(
        "Calculate impermanent loss for a V3 concentrated position given price change. ",
        "Exact V3 IL formula. Supports backtest mode against historical prices. ",
        "Call before LP entry for risk assessment. ",
        "Does NOT compute fee revenue in scenario mode -- use backtest mode or pair with intel_compute_lvr.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Loading position parameters", "Computing IL"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Calculate V3 concentrated IL given price change. Exact formula. Supports backtest mode. Call before LP entry.",
    prompt_guidelines: &[
        "thriving: Run IL scenarios (+/-10%, +/-25%, +/-50%) before LP entry. Compare IL against projected fees.",
        "cautious: Run with wider range. Prefer positions where IL at +/-25% < 5%.",
        "declining: Calculate IL on existing positions to assess current loss.",
        "terminal: Not needed. Remove all liquidity regardless.",
    ],
};
```

### `intel_recommend_fee_tier`

Recommend the optimal fee tier for a given token pair based on volatility, volume, and existing pool distribution.

```rust
#[derive(Debug, Deserialize)]
pub struct RecommendFeeTierParams {
    pub token0: String,
    pub token1: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct FeeTierRecommendation {
    pub recommended_tier: u32,
    pub reasoning: String,
    pub pool_distribution: Vec<PoolTierInfo>,
    pub volatility_30d: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_recommend_fee_tier",
    description: "Recommend optimal fee tier for a token pair based on volatility, volume, and pool distribution. Read-only.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Analyzing pool distribution", "Computing volatility", "Ranking tiers"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Recommend optimal fee tier for a token pair. Read-only.",
    prompt_guidelines: &[
        "thriving: Check before creating new LP positions.",
        "cautious: Same.",
        "declining: Not relevant during wind-down.",
        "terminal: Not needed.",
    ],
};
```

### `intel_simulate_price_impact`

Simulate price impact at multiple trade sizes simultaneously against a specific pool using on-chain tick traversal via Alloy (the standard Rust library for EVM interaction) `eth_call`.

```rust
#[derive(Debug, Deserialize)]
pub struct SimulatePriceImpactParams {
    pub pool: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// Array of trade sizes in USD to simulate.
    pub trade_sizes_usd: Vec<f64>,
    /// "buy" or "sell" (default: "buy").
    #[serde(default = "default_buy")]
    pub direction: String,
}

#[derive(Debug, Serialize)]
pub struct PriceImpactResult {
    pub impacts: Vec<PriceImpactAtSize>,
    /// Largest trade size with impact < 1%.
    pub max_acceptable_size_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct PriceImpactAtSize {
    pub trade_size_usd: f64,
    pub price_impact_bps: f64,
    pub ticks_traversed: u32,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_simulate_price_impact",
    description: concat!(
        "Simulate price impact at multiple trade sizes against a specific pool. ",
        "On-chain tick traversal via eth_call. ",
        "Returns max trade size before impact exceeds 1%. ",
        "Use to find position sizing limits.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Loading tick data", "Simulating trade sizes"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Simulate price impact at multiple trade sizes. On-chain tick traversal. Find max acceptable trade size.",
    prompt_guidelines: &[
        "thriving: Use to size trades and LP positions optimally.",
        "cautious: Use to verify exit trades won't suffer excessive slippage.",
        "declining: Check impact for planned exit sizes.",
        "terminal: Quick check for large exits.",
    ],
};
```

### `intel_simulate_pool_scenario`

Simulate hypothetical LP position performance under configurable price trajectories and volume profiles (bull/bear/sideways).

```rust
#[derive(Debug, Deserialize)]
pub struct SimulatePoolScenarioParams {
    pub pool: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// "bull", "bear", "sideways", or custom trajectory.
    pub scenario: String,
    /// Simulation duration in days (default: 30).
    #[serde(default = "default_30")]
    pub duration_days: u32,
    pub tick_lower: i32,
    pub tick_upper: i32,
    /// Position size in USD.
    pub position_size_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct ScenarioResult {
    pub projected_fees_usd: f64,
    pub projected_il_usd: f64,
    pub projected_net_pnl_usd: f64,
    pub time_in_range_pct: f64,
    pub daily_projections: Vec<DailyProjection>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_simulate_pool_scenario",
    description: "Simulate LP position performance under configurable price trajectories (bull/bear/sideways). Strategy development and stress testing. Read-only.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Loading pool state", "Running scenario simulation", "Computing projections"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Simulate LP position performance under price scenarios. Stress test before deployment.",
    prompt_guidelines: &[
        "thriving: Run multiple scenarios before LP entry. Compare bull/bear/sideways outcomes.",
        "cautious: Run bear scenario. Ensure worst-case loss is acceptable.",
        "declining: Not relevant.",
        "terminal: Not needed.",
    ],
};
```

---

## MEV and execution intelligence (2 tools)

### `intel_assess_mev_risk`

Assess MEV risk for a proposed swap: sandwich attack probability, mempool exposure, mitigation recommendations.

```rust
#[derive(Debug, Deserialize)]
pub struct AssessMevRiskParams {
    pub token_in: String,
    pub token_out: String,
    /// Trade size in USD.
    pub amount_usd: f64,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    pub pool: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MevRiskAssessment {
    /// LOW (< 0.1% of pool liquidity), MEDIUM (0.1-1%), HIGH (1-5%), CRITICAL (> 5%).
    pub risk_level: MevRiskLevel,
    pub sandwich_probability: f64,
    pub estimated_mev_cost_usd: f64,
    pub mitigations: Vec<MevMitigation>,
}

#[derive(Debug, Serialize)]
pub enum MevRiskLevel { Low, Medium, High, Critical }

#[derive(Debug, Serialize)]
pub struct MevMitigation {
    pub strategy: String,         // "uniswapx", "flashbots_protect", "trade_split", "private_mempool"
    pub estimated_savings_usd: f64,
    pub tradeoff: String,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_assess_mev_risk",
    description: concat!(
        "Estimate MEV exposure for a proposed swap: sandwich probability, mempool exposure, ",
        "mitigation recommendations (UniswapX, Flashbots Protect, trade splitting). ",
        "Call before any swap > $10K. ",
        "Does NOT execute trades -- use uniswap_execute_swap after assessment.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Analyzing pool liquidity", "Estimating sandwich probability", "Computing mitigations"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Estimate MEV exposure for a swap: sandwich probability, mitigation recommendations. Call before swaps > $10K.",
    prompt_guidelines: &[
        "thriving: Call before trades > $10K. Follow mitigation recommendations.",
        "cautious: Call before trades > $5K. Reject trades with CRITICAL MEV risk.",
        "declining: Call before exit trades. Use recommended mitigation to protect exit value.",
        "terminal: Call for large exits. Accept MEDIUM risk for speed. Use UniswapX or Flashbots Protect.",
    ],
};
```

### `intel_compare_venues`

Compare execution quality across Uniswap and external DEX aggregators (1inch, Paraswap, CoW Protocol). Returns net output after gas for each venue.

```rust
#[derive(Debug, Deserialize)]
pub struct CompareVenuesParams {
    pub token_in: String,
    pub token_out: String,
    pub amount: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VenueComparison {
    pub venues: Vec<VenueQuote>,
    pub recommended: String,
    pub savings_vs_worst_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct VenueQuote {
    pub venue: String,
    pub output_amount: String,
    pub gas_estimate_usd: f64,
    pub net_output_usd: f64,
    pub latency_ms: u64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_compare_venues",
    description: concat!(
        "Compare execution quality across Uniswap, 1inch, Paraswap, CoW Protocol. ",
        "Returns net output after gas for each venue. ",
        "Use for large trades where venue selection matters (> $50K).",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,       // 2-4s, multiple API calls
    progress_steps: &["Querying Uniswap", "Querying aggregators", "Comparing net output"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Compare execution quality across DEX venues. Net output after gas. Use for trades > $50K.",
    prompt_guidelines: &[
        "thriving: Compare for trades > $50K. Pick best net output after gas.",
        "cautious: Compare for trades > $10K. Weight reliability over marginal price improvement.",
        "declining: Compare for exit trades. Pick highest net output.",
        "terminal: Use if comparing routes. Otherwise use Uniswap directly for speed.",
    ],
};
```

---

## Token discovery (1 tool)

### `intel_discover_token`

Multi-source token discovery with risk scoring from 4 data sources: DexScreener, CoinGecko (x402 (a micropayment protocol that lets agents pay per-request without API keys)), on-chain contract verification, and Uniswap subgraph.

```rust
#[derive(Debug, Deserialize)]
pub struct DiscoverTokenParams {
    /// Token address or symbol.
    pub token: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct TokenDiscovery {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub risk_rating: TokenRiskRating,
    pub risk_score: u32,
    pub risk_factors: Vec<String>,
    pub sources: TokenSources,
    pub market_data: TokenMarketData,
}

/// SAFE (>= 80), CAUTION (60-79), WARNING (40-59), DANGER (< 40).
#[derive(Debug, Serialize)]
pub enum TokenRiskRating { Safe, Caution, Warning, Danger }
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_discover_token",
    description: concat!(
        "Multi-source token discovery with risk scoring: DexScreener, CoinGecko, on-chain verification, subgraph. ",
        "Risk ratings: SAFE (>= 80), CAUTION (60-79), WARNING (40-59), DANGER (< 40). ",
        "Gate all unverified token interactions through this tool.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,       // 2-3s, multiple source queries
    progress_steps: &["Querying DexScreener", "Querying CoinGecko", "Verifying on-chain", "Computing risk score"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Multi-source token discovery with risk scoring. SAFE/CAUTION/WARNING/DANGER. Gate all unverified token interactions through this.",
    prompt_guidelines: &[
        "thriving: Run for every new token before interaction. Require SAFE or CAUTION.",
        "cautious: Require SAFE (>= 80). Reject WARNING and DANGER outright.",
        "declining: Only verify tokens needed for exit routes.",
        "terminal: Not needed.",
    ],
};
```

---

## Market context (5 tools)

### `intel_get_defi_context`

Aggregate DeFi context for decision-making: TVL trends, volume trends, protocol activity across chains.

```rust
#[derive(Debug, Deserialize)]
pub struct GetDefiContextParams {
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// "1d", "7d", "30d" (default: "7d").
    #[serde(default = "default_lookback_7d")]
    pub lookback: String,
}

#[derive(Debug, Serialize)]
pub struct DefiContext {
    pub tvl_total_usd: f64,
    pub tvl_change_pct: f64,
    pub volume_24h_usd: f64,
    pub volume_change_pct: f64,
    pub active_protocols: u32,
    pub top_protocols: Vec<ProtocolSummary>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_get_defi_context",
    description: "Aggregate DeFi context: TVL trends, volume trends, protocol activity. Read-only.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Fetching TVL data", "Computing trends"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Aggregate DeFi context: TVL, volume, protocol activity. Read-only.",
    prompt_guidelines: &[
        "thriving: Check weekly for macro context.",
        "cautious: Check before strategic decisions.",
        "declining: Check to understand whether decline is market-wide.",
        "terminal: Not relevant.",
    ],
};
```

### `intel_classify_regime`

Classify the current market regime using a Hidden Markov Model (HMM) with 5 states. Features: log return (7d), rolling volatility (30d), TVL trend, stablecoin dominance, funding rate.

```rust
#[derive(Debug, Deserialize)]
pub struct ClassifyRegimeParams {
    /// Default: "base".
    #[serde(default = "default_base")]
    pub chain: String,
    /// Default: "ETH".
    #[serde(default = "default_eth")]
    pub asset: String,
}

#[derive(Debug, Serialize)]
pub struct RegimeClassification {
    /// strong_bull, weak_bull, sideways, weak_bear, strong_bear.
    pub current_regime: String,
    pub confidence: f64,
    pub transition_probabilities: Vec<f64>,
    pub features: RegimeFeatures,
    pub historical_regimes: Vec<HistoricalRegime>,
}

#[derive(Debug, Serialize)]
pub struct RegimeFeatures {
    pub log_return_7d: f64,
    pub rolling_volatility_30d: f64,
    pub tvl_trend: f64,
    pub stablecoin_dominance: f64,
    pub funding_rate: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_classify_regime",
    description: concat!(
        "HMM 5-state market regime classification: strong_bull, weak_bull, sideways, weak_bear, strong_bear. ",
        "Features: log return, volatility, TVL trend, stablecoin dominance, funding rate.",
    ),
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Fetching feature data", "Running HMM", "Computing transition probabilities"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "HMM 5-state regime classification. Use at each decision cycle to adjust risk parameters.",
    prompt_guidelines: &[
        "thriving: Classify at each decision cycle. Adjust risk parameters based on regime.",
        "cautious: Classify daily. Shift to conservative strategies in bear regimes.",
        "declining: Classify to understand whether decline is personal or market-wide.",
        "terminal: One final classification for death testament context.",
    ],
};
```

### `intel_defilama_get_yields`

Yield aggregation across 13K+ pools, 489 protocols, 117 chains via the DefiLlama Pro API. Free API with 15-minute cache.

```rust
#[derive(Debug, Deserialize)]
pub struct DefilamaGetYieldsParams {
    /// Filter by chain (optional).
    pub chain: Option<String>,
    /// Filter by protocol (optional).
    pub protocol: Option<String>,
    /// Minimum TVL in USD (default: 100000).
    #[serde(default = "default_min_tvl")]
    pub min_tvl_usd: f64,
    /// Sort by: "apy", "tvl", "volume" (default: "apy").
    #[serde(default = "default_sort_apy")]
    pub sort_by: String,
    /// Max results (default: 20).
    #[serde(default = "default_limit_20")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct YieldResult {
    pub pools: Vec<YieldPool>,
    pub total_matching: u32,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_defilama_get_yields",
    description: "Yield aggregation from DefiLlama: 13K+ pools, 489 protocols, 117 chains. Free API, 15-minute cache. Primary yield discovery data source.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Querying DefiLlama", "Filtering and sorting"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Yield aggregation from DefiLlama: 13K+ pools, 489 protocols, 117 chains. Free API. Primary yield discovery.",
    prompt_guidelines: &[
        "thriving: Use for yield discovery and opportunity scanning.",
        "cautious: Filter by high TVL (> $1M) for lower-risk opportunities.",
        "declining: Check if better yield opportunities exist for remaining capital.",
        "terminal: Not relevant.",
    ],
};
```

### `intel_get_market_sentiment`

Fear/greed index, open interest trends, and market regime classification from multiple free APIs.

```rust
#[derive(Debug, Serialize)]
pub struct MarketSentiment {
    pub fear_greed_index: u32,
    pub fear_greed_label: String,
    pub open_interest_change_24h_pct: f64,
    pub funding_rate_aggregate: f64,
    pub regime_hint: String,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_get_market_sentiment",
    description: "Fear/greed index, open interest trends, funding rates. Macro context for strategic decisions. Read-only.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Fetching sentiment data", "Aggregating signals"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Fear/greed index, open interest, funding rates. Macro context before strategic decisions.",
    prompt_guidelines: &[
        "thriving: Check before large allocation changes.",
        "cautious: Check daily. Extreme fear or greed should trigger defensive posture.",
        "declining: Check to contextualize decline.",
        "terminal: One final read for death testament.",
    ],
};
```

### `intel_get_chain_status`

Chain health and status: block number, gas prices, RPC latency, Uniswap contract versions available.

```rust
#[derive(Debug, Deserialize)]
pub struct GetChainStatusParams {
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ChainStatus {
    pub chain_id: u64,
    pub block_number: u64,
    pub gas_price_gwei: f64,
    pub base_fee_gwei: f64,
    pub priority_fee_gwei: f64,
    pub rpc_latency_ms: u64,
    pub uniswap_versions: Vec<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_get_chain_status",
    description: "Chain health: block number, gas prices (EIP-1559 breakdown), RPC latency, Uniswap version support. Read-only diagnostic.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Querying chain state"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Chain health: block, gas, RPC latency, Uniswap versions. Read-only diagnostic.",
    prompt_guidelines: &[
        "thriving: Check before operations on unfamiliar chains.",
        "cautious: Check before any write operation. High gas should delay non-urgent actions.",
        "declining: Check gas before exit transactions.",
        "terminal: Check gas before final exit.",
    ],
};
```

---

## Execution analytics (3 tools)

### `intel_estimate_revert_risk`

Estimate probability of a transaction reverting on-chain by combining chain revert rates, pool volatility, trade size, and stale state risk.

```rust
#[derive(Debug, Deserialize)]
pub struct EstimateRevertRiskParams {
    pub pool: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    pub trade_size_usd: f64,
    /// Seconds since last pool state read.
    pub state_age_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct RevertRiskEstimate {
    pub revert_probability: f64,
    pub risk_factors: Vec<RevertFactor>,
    pub recommendation: String,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_estimate_revert_risk",
    description: "Estimate transaction revert probability: chain revert rate, pool volatility, trade size, stale state. Call before time-sensitive transactions.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Analyzing revert factors"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Estimate revert probability. Call before time-sensitive transactions.",
    prompt_guidelines: &[
        "thriving: Call before large or time-sensitive trades.",
        "cautious: Call before any write operation.",
        "declining: Call before exit trades to avoid wasting gas on reverts.",
        "terminal: Quick check before final transactions.",
    ],
};
```

### `intel_advise_gas_optimization`

Recommend optimal timing, chain, and batching strategies to minimize gas costs.

```rust
#[derive(Debug, Deserialize)]
pub struct AdviseGasOptimizationParams {
    /// Operations to optimize: ["swap", "add_liquidity", "vault_deposit"].
    pub operations: Vec<String>,
    /// Target chains.
    pub chain_ids: Vec<u64>,
    /// "minimize_cost" or "minimize_time" (default: "minimize_cost").
    #[serde(default = "default_minimize_cost")]
    pub objective: String,
}

#[derive(Debug, Serialize)]
pub struct GasAdvice {
    pub recommended_chain: u64,
    pub recommended_time_utc: String,
    pub batch_suggestion: Option<String>,
    pub estimated_savings_usd: f64,
    pub current_gas_by_chain: Vec<ChainGasInfo>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_advise_gas_optimization",
    description: "Recommend optimal timing, chain, and batching strategies to minimize gas costs. Returns time-of-day recommendations and chain-specific estimates.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Analyzing gas patterns", "Computing recommendations"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Gas optimization: timing, chain selection, batching. Minimize costs for planned operations.",
    prompt_guidelines: &[
        "thriving: Consult before batch operations or non-urgent writes.",
        "cautious: Always consult. Delay non-urgent operations for cheaper gas windows.",
        "declining: Minimize gas waste on exit operations.",
        "terminal: Quick check. Speed over cost for final exits.",
    ],
};
```

### `intel_calculate_fee_switch_impact`

Model the impact of Uniswap's protocol fee activation on LP returns [UNIFIGATION-2025].

```rust
#[derive(Debug, Deserialize)]
pub struct FeeSwitchImpactParams {
    pub pool: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// Hypothetical fee switch rate in bps (default: 1000 = 10%).
    #[serde(default = "default_fee_switch_1000")]
    pub fee_switch_bps: u32,
}

#[derive(Debug, Serialize)]
pub struct FeeSwitchImpact {
    pub current_fee_apy: f64,
    pub post_switch_fee_apy: f64,
    pub apy_reduction_pct: f64,
    pub fee_revenue_lost_30d_usd: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "intel_calculate_fee_switch_impact",
    description: "Model impact of Uniswap protocol fee activation on LP returns. Governance-aware LP strategy tool. Read-only.",
    category: Category::Intelligence,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Computing pre/post fee revenue"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Model Uniswap fee switch impact on LP returns. Governance-aware strategy tool.",
    prompt_guidelines: &[
        "thriving: Check for pools where fee switch would materially change profitability.",
        "cautious: Factor into LP decision-making. Prefer pools resilient to fee switch.",
        "declining: Not actionable during wind-down.",
        "terminal: Not needed.",
    ],
};
```

---

## Tool count summary

| Section | Tools | Write ops |
| --- | --- | --- |
| Order flow intelligence | 3 | 0 |
| IL and LP analytics | 4 | 0 |
| MEV and execution intelligence | 2 | 0 |
| Token discovery | 1 | 0 |
| Market context | 5 | 0 |
| Execution analytics | 3 | 0 |
| **Total** | **18** | **0** |

All 18 tools are `CapabilityTier::Read` and `RiskTier::Layer1`. No capability token required. No ActionPermit. No safety hook chain. The intelligence category is structurally unable to modify on-chain state.
