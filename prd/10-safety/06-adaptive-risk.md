# Adaptive Risk Management: Five-Layer Implementation [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-14
>
> **Crates**: `golem-risk`, `bardo-safety`, `bardo-policy`
>
> **Depends on**: [00-defense.md](00-defense.md) (six-layer defense architecture), [02-policy.md](02-policy.md) (PolicyCage), `../01-golem/13-runtime-extensions.md` (Extension hooks)

> **Reader orientation:** This document specifies the five-layer adaptive risk management system that governs a Golem's (mortal autonomous DeFi agent) runtime behavior. It belongs to the **Safety** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: these five layers (Hard Shields, Position Sizing, Adaptive Guardrails, Health Observation, DeFi Threat Detection) are all T0, meaning they run as deterministic Rust code with no LLM calls and zero cost per tick. The LLM proposes actions; these layers dispose. Terms like PolicyCage, Heartbeat, BehavioralPhase, and Vitality are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

The risk system in `00-defense.md` defines the defensive architecture. This document specifies the *adaptive* component — five layers of runtime risk control that self-evolve within hard bounds. Every layer is T0: deterministic Rust, no LLM calls, zero cost per tick. The LLM proposes actions; the risk engine disposes.

The adaptive risk system is where the Golem's mortality becomes operational. A Golem in Thriving phase operates at wide guardrails. As it transitions through Stable, Conservation, Declining, and Terminal, the risk layers tighten automatically — not because a rule says "tighten in phase 3" but because the confidence tracker, health score, and behavioral anomaly detector all respond to the declining conditions that trigger phase transitions. The Die moat (see `../01-golem/04-mortality.md`) is not just philosophical — it is a risk management mechanism. A Golem that knows it is dying trades differently than one that believes it is immortal.

---

## Architecture Overview

The risk system operates across three Pi hooks and one internal timer:

| Layer | Enforcement Point | Pi Hook | Cost |
|---|---|---|---|
| **Layer 1: Hard Shields** | PolicyCage (on-chain) + phase enforcement | `tool_call` (existing `bardo-safety`) | T0 ($0.00) |
| **Layer 2: Position Sizing** | Kelly criterion, CDaR constraints | `tool_call` (`bardo-risk`) | T0 ($0.00) |
| **Layer 3: Adaptive Guardrails** | Bayesian trust expansion | `tool_call` + `after_turn` (`bardo-risk`) | T0 ($0.00) |
| **Layer 4: Observation** | Health scores, anomaly detection | `after_turn` (`bardo-risk`) + heartbeat probes | T0 ($0.00) |
| **Layer 5: DeFi Threats** | MEV, oracle, composability | `tool_call` + `tool_result` (`bardo-risk` + `bardo-result-filter`) | T0 ($0.00) |

Layer 1 is specified in [02-policy.md](02-policy.md) and the `bardo-safety` extension. Layers 2–5 live in `bardo-risk`, described here.

---

## Part 1: The `bardo-risk` Extension

### 1.1 Extension Registration

The extension declares dependencies on `bardo-safety` (which owns Layer 1 hard shields), `bardo-tools` (which provides action dispatch), and `bardo-grimoire` (which stores execution history for confidence tracking).

```rust
pub fn bardo_risk(pi: &mut ExtensionAPI) {
    pi.depends_on(&["bardo-safety", "bardo-tools", "bardo-grimoire"]);

    // PRE-EXECUTION: Assess risk before any write action
    pi.on(Hook::ToolCall, |event, _ctx| {
        if event.name != "preview_action" && event.name != "commit_action" {
            return Ok(None); // Only assess write operations
        }
        assess_action_risk(event, &GOLEM_STATE)
    });

    // POST-TICK: Update observation metrics and confidence
    pi.on(Hook::AfterTurn, |_event, _ctx| {
        update_observation_metrics(&GOLEM_STATE)?;
        evolve_guardrails_if_due(&GOLEM_STATE)?;
        Ok(None)
    });
}
```

Two hooks, two responsibilities. The `tool_call` hook runs synchronously before any write, applying Layers 2–5 as a pipeline. The `after_turn` hook runs after each LLM turn completes, updating the observation state that feeds the next `tool_call` assessment.

### 1.2 Risk Assessment Pipeline

When the LLM calls `preview_action` or `commit_action`, the risk assessment runs as a synchronous pipeline. Each layer can block, modify parameters, or pass through:

```rust
fn assess_action_risk(
    event: &mut ToolCallEvent,
    state: &GolemState,
) -> Result<Option<RiskAssessment>> {
    let params = &event.params;

    // Layer 1: Hard shields (already enforced by bardo-safety, but we add deployment rate)
    let deployment_check = check_deployment_rate(params, state);
    if !deployment_check.allowed { return Ok(Some(deployment_check)); }

    // Layer 2: Position sizing
    let sizing_check = compute_position_sizing(params, state);
    if !sizing_check.allowed { return Ok(Some(sizing_check)); }
    // If sizing reduced the amount, modify params
    if let Some(adjusted) = &sizing_check.adjusted_params {
        event.params.extend(adjusted.clone());
    }

    // Layer 3: Adaptive guardrails
    let guardrail_check = check_adaptive_guardrails(params, state);
    if !guardrail_check.allowed { return Ok(Some(guardrail_check)); }

    // Layer 4: Observation anomaly check
    let anomaly_check = check_behavioral_anomalies(params, state);
    if !anomaly_check.allowed { return Ok(Some(anomaly_check)); }

    // Layer 5: DeFi threat check
    let threat_check = check_defi_threats(params, state)?;
    if !threat_check.allowed { return Ok(Some(threat_check)); }

    // All layers passed
    Ok(None)
}
```

The pipeline is ordered by cost: cheap deterministic checks first, async on-chain reads last. A block at Layer 2 prevents the more expensive Layer 5 oracle checks from running at all.

### 1.3 Risk Tier Dispatch

Different action types route to different sizing and threat models. The dispatch happens within `computePositionSizing`:

```rust
fn compute_position_sizing(
    params: &Params,
    state: &GolemState,
) -> RiskAssessment {
    let action_kind = params.get_str("type").unwrap_or_default();
    let requested_amount = params.get_u256("amount").unwrap_or_default();

    let sizer = match action_kind {
        "swap" => {
            // Spot trading: fractional Kelly on estimated edge/vol
            let edge = estimate_trade_edge(params, state);
            let vol = estimate_asset_volatility(params.get_str("token_out").unwrap(), state);
            state.kelly_sizer.compute_size(KellyParams {
                estimated_edge: edge,
                estimated_vol: vol,
                operational_confidence: state.operational_confidence.composite(),
                portfolio_nav: state.portfolio.total_nav,
                existing_positions: &state.portfolio.positions_by_asset,
                max_concentration_bps: state.policy_state.max_concentration_bps,
                max_var95_bps: state.policy_state.max_var95_bps.unwrap_or(500),
            })
        }
        "add_liquidity" => {
            // LP sizing: LVR framework [MILIONIS-2022]
            compute_lp_size(params, state)
        }
        "deposit" => {
            // Lending/vault: protocol-risk-weighted
            compute_deposit_size(params, state)
        }
        _ => return RiskAssessment { allowed: true, ..Default::default() },
    };

    if sizer.max_allocation == U256::ZERO {
        return RiskAssessment {
            allowed: false,
            reason: format!("position_sizing_zero: {}", sizer.reasoning),
            layer: "sizing".into(),
            risk_score: 0.9,
            details: vec![RiskDetail {
                metric: "kelly_allocation".into(),
                current: "0".into(),
                limit: "0".into(),
                reasoning: sizer.reasoning.clone(),
            }],
            suggestion: Some("Reduce position size or wait for better conditions.".into()),
            ..Default::default()
        };
    }

    if requested_amount > sizer.max_allocation {
        return RiskAssessment {
            allowed: true,
            adjusted_params: Some(HashMap::from([
                ("amount".into(), sizer.max_allocation.to_string().into()),
            ])),
            reason: format!("position_sized_down: {}", sizer.reasoning),
            layer: "sizing".into(),
            risk_score: 0.4,
            details: vec![RiskDetail {
                metric: "kelly_allocation".into(),
                current: requested_amount.to_string(),
                limit: sizer.max_allocation.to_string(),
                reasoning: sizer.reasoning.clone(),
            }],
            ..Default::default()
        };
    }

    RiskAssessment { allowed: true, ..Default::default() }
}
```

The sizing layer does not just block — it *adjusts*. A swap request for $50,000 that exceeds the Kelly-derived maximum of $12,000 gets silently reduced. The LLM's intent is preserved; the magnitude is corrected. This is qualitatively different from a hard reject: the agent learns from the adjusted outcome without being stopped entirely.

---

## Part 2: Layer 2 — Fractional Kelly Position Sizing

### 2.1 Theory

Kelly's criterion [KELLY-1956] determines the growth-optimal bet size for a repeated game with known edge and volatility. The formula is simple: `f* = edge / variance`. The problem is that full Kelly sizing assumes (a) infinite repetition, (b) known edge, and (c) tolerance for 50%+ drawdowns. None of these hold for an autonomous agent managing real capital.

Carta et al. [CARTA-2020] demonstrated via Monte Carlo simulation that full Kelly requires thousands of trades before converging to the theoretical growth rate, and that triple Kelly leads to certain ruin. Half-Kelly captures approximately 75% of optimal growth with dramatically reduced drawdown risk. This is a free lunch in the Pareto sense: you give up 25% of expected growth and avoid ruin.

Busseti, Ryu, and Boyd [BUSSETI-BOYD-2016] extended this with risk-constrained Kelly gambling formulated as a convex optimization problem that guarantees drawdown probability stays below a specified level.

For an autonomous agent with no human in the loop, the conservative choice is clear. We default to fractional Kelly with confidence-modulated scaling.

### 2.2 Implementation

```rust
/// Fractional Kelly position sizer with confidence-modulated scaling.
///
/// The Kelly fraction scales with operational confidence:
///   kelly_fraction = base_kelly * confidence_multiplier(operational_confidence)
///
/// At confidence 0.0: ~10% of growth-optimal (maximum caution)
/// At confidence 0.9: ~50% of growth-optimal (half-Kelly)
///
/// Reference: [KELLY-1956], [CARTA-2020], [MACLEAN-ZIEMBA-BLAZENKO-1992]
pub struct FractionalKellySizer {
    asset_volatilities: HashMap<String, f64>,
}

impl FractionalKellySizer {
    pub fn compute_size(&self, params: KellyParams) -> PositionSizeResult {
        // Base Kelly fraction: edge / vol^2
        // Capped at 0.5 (half-Kelly ceiling)
        let raw_kelly = params.estimated_edge / (params.estimated_vol.powi(2));
        let base_kelly = raw_kelly.clamp(0.0, 0.5);

        // Confidence multiplier: sigmoid curve
        // f(c) = 0.1 + 0.4 * sigmoid(10 * (c - 0.5))
        let sigmoid = |x: f64| 1.0 / (1.0 + (-x).exp());
        let confidence_multiplier =
            0.1 + 0.4 * sigmoid(10.0 * (params.operational_confidence - 0.5));

        let adjusted_fraction = base_kelly * confidence_multiplier;

        // Compute max allocation in USDC terms
        let nav_f64 = params.portfolio_nav.to::<f64>();
        let mut max_allocation = U256::from((nav_f64 * adjusted_fraction) as u128);

        // Cap by PolicyCage max concentration
        let concentration_cap =
            U256::from((nav_f64 * params.max_concentration_bps as f64 / 10000.0) as u128);
        let capped_by_policy = max_allocation > concentration_cap;
        if capped_by_policy {
            max_allocation = concentration_cap;
        }

        // Cap by portfolio VaR constraint
        let current_var = self.compute_portfolio_var(params.existing_positions);
        let var_capacity =
            nav_f64 * params.max_var95_bps as f64 / 10000.0 - current_var;
        if var_capacity <= 0.0 {
            return PositionSizeResult {
                max_allocation: U256::ZERO,
                kelly_fraction: base_kelly,
                adjusted_fraction,
                capped_by_policy: false,
                capped_by_var: true,
                reasoning: format!(
                    "Portfolio VaR at capacity ({:.2} >= {:.2}). No additional risk capacity.",
                    current_var,
                    nav_f64 * params.max_var95_bps as f64 / 10000.0
                ),
            };
        }

        PositionSizeResult {
            max_allocation,
            kelly_fraction: base_kelly,
            adjusted_fraction,
            capped_by_policy,
            capped_by_var: false,
            reasoning: format!(
                "Kelly: {:.1}% * confidence({:.2}) = {:.1}% of NAV",
                base_kelly * 100.0,
                params.operational_confidence,
                adjusted_fraction * 100.0
            ),
        }
    }

    /// Estimate portfolio-level 95% 1-day VaR using variance-covariance method.
    /// Simplified: assumes positions are independent (conservative overestimate).
    /// V2 will use correlation matrix from historical data.
    fn compute_portfolio_var(&self, positions: &HashMap<String, U256>) -> f64 {
        let mut total_var_sq = 0.0;
        for (asset, size) in positions {
            let vol = self.get_asset_volatility(asset); // 1-day vol
            let position_var = size.to::<f64>() * vol * 1.645; // 95% z-score
            total_var_sq += position_var.powi(2); // Sum of squares (independence)
        }
        total_var_sq.sqrt()
    }
}
```

The sigmoid confidence multiplier deserves explanation. A new agent starts at confidence ~0.25 (weakly pessimistic prior). The sigmoid maps this to a multiplier around 0.11, meaning the agent bets at roughly 11% of the Kelly-optimal size. After hundreds of successful trades, confidence rises toward 0.8–0.9, and the multiplier approaches 0.5, yielding half-Kelly. The agent earns the right to larger positions through demonstrated competence, not through time alone.

### 2.3 LP-Specific Sizing (LVR Framework)

LP sizing is fundamentally different from spot trading. The risk is not directional — it is adverse selection. Milionis, Moallemi, Roughgarden, and Zhang [MILIONIS-2022] formalized this as Loss-Versus-Rebalancing (LVR): the cost a liquidity provider pays to informed traders. The result: LVR scales with the square of volatility. High-volatility pairs bleed LPs faster.

Loesch et al. [LOESCH-2021] put numbers to it across 17 Uniswap v3 pools: total fees $199.3M versus total impermanent loss $260.1M. LPs were collectively $60.8M worse off. The theoretical framework matches the empirical data.

```rust
fn compute_lp_size(params: &Params, state: &GolemState) -> PositionSizeResult {
    let pool = params.get_str("pool").unwrap();
    let tick_lower = params.get_i32("tick_lower").unwrap();
    let tick_upper = params.get_i32("tick_upper").unwrap();

    let pool_data = match state.market_snapshot.pools.get(pool) {
        Some(data) => data,
        None => return PositionSizeResult {
            max_allocation: U256::ZERO,
            kelly_fraction: 0.0,
            adjusted_fraction: 0.0,
            capped_by_policy: false,
            reasoning: "Pool data unavailable — cannot estimate LP profitability.".into(),
            ..Default::default()
        },
    };

    let range_width = tick_upper - tick_lower;
    let fee_apr = pool_data.fee_apr_24h;
    let vol = pool_data.volatility_24h;

    // LVR estimate: scales with sigma^2 and concentration
    // Reference: [MILIONIS-2022], equation 8
    let concentration_factor = (1.0001_f64).powi(range_width).sqrt();
    let estimated_lvr = vol.powi(2) / (8.0 * concentration_factor);

    // Gas cost per rebalance (amortized)
    let rebalance_frequency = estimate_rebalance_frequency(vol, range_width);
    let gas_per_rebalance = state.market_snapshot.avg_gas_cost_usd;
    let annualized_gas = gas_per_rebalance * rebalance_frequency * 365.0;

    // Net expected return
    let net_return = fee_apr - estimated_lvr - annualized_gas;

    let nav_f64 = state.portfolio.total_nav.to::<f64>();

    if net_return <= 0.0 {
        // Unprofitable LP — allow only a minimal exploratory position
        let exploratory_max = U256::from((nav_f64 * 0.005) as u128);
        return PositionSizeResult {
            max_allocation: exploratory_max,
            kelly_fraction: 0.0,
            adjusted_fraction: 0.005,
            capped_by_policy: false,
            reasoning: format!(
                "LP unprofitable: feeAPR {:.1}% - LVR {:.1}% - gas {:.1}% = {:.1}%. Exploratory position only.",
                fee_apr * 100.0, estimated_lvr * 100.0, annualized_gas * 100.0, net_return * 100.0
            ),
            ..Default::default()
        };
    }

    // Profitable LP: size using fractional Kelly on net return
    let lp_kelly = (net_return / vol.powi(2)).min(0.3); // More conservative than trading
    let sigmoid = |x: f64| 1.0 / (1.0 + (-x).exp());
    let confidence = state.operational_confidence.dimensions.lp_competence.lower_95;
    let adjusted_fraction = lp_kelly * (0.1 + 0.4 * sigmoid(10.0 * (confidence - 0.5)));

    PositionSizeResult {
        max_allocation: U256::from((nav_f64 * adjusted_fraction) as u128),
        kelly_fraction: lp_kelly,
        adjusted_fraction,
        capped_by_policy: false,
        reasoning: format!(
            "LP net return: {:.1}%. Kelly: {:.1}% * confidence({:.2}) = {:.1}% of NAV.",
            net_return * 100.0, lp_kelly * 100.0, confidence, adjusted_fraction * 100.0
        ),
        ..Default::default()
    }
}
```

The 0.3 cap on `lpKelly` is intentional. LP positions are illiquid relative to spot — you cannot exit a concentrated v3 position during a volatility spike without taking impermanent loss. The tighter cap reflects this structural disadvantage.

When the model estimates negative net return (fees minus LVR minus gas), the system still allows a 0.5% exploratory position. This is a deliberate design choice. An agent that never LPs in marginal pools never learns whether its estimates are calibrated. The exploratory allocation is small enough that miscalibration costs dollars, not the portfolio.

---

## Part 3: Layer 3 — Bayesian Adaptive Guardrails

### 3.1 Operational Confidence (Beta-Binomial Model)

The core question: how much should the system trust an agent's decisions? Fixed limits are wrong in both directions. Too tight and the agent cannot operate. Too loose and it can blow up the portfolio before earning trust. The answer is Bayesian: start pessimistic, update on evidence, expand the safe region as confidence accumulates.

Berkenkamp et al. [BERKENKAMP-2017] formalized this for model-based RL with Lyapunov stability guarantees using Gaussian process models. Their algorithm safely collects data and gradually expands the safe region of the state space. We adapt this principle to position sizing and action limits.

```rust
/// Tracks operational confidence across multiple competence dimensions
/// using Beta-Binomial models with asymmetric learning rates.
///
/// Weakly pessimistic priors (alpha=1, beta=3) start at mean 0.25.
/// The composite uses geometric mean of lower 95% credible intervals,
/// ensuring a single poorly-calibrated dimension drags everything down.
pub struct OperationalConfidenceTracker {
    pub dimensions: HashMap<String, BetaDistribution>,
}

#[derive(Debug, Clone)]
pub struct BetaDistribution {
    pub alpha: f64,
    pub beta: f64,
}

impl OperationalConfidenceTracker {
    pub fn new() -> Self {
        let mut dimensions = HashMap::new();
        for name in &[
            "trading_competence",
            "lp_competence",
            "lending_competence",
            "regime_adaptability",
            "risk_calibration",
        ] {
            dimensions.insert(
                name.to_string(),
                BetaDistribution { alpha: 1.0, beta: 3.0 },
            );
        }
        Self { dimensions }
    }

    /// Update a dimension after an action outcome.
    ///
    /// Asymmetric learning rates: failures count 1.5x to maintain
    /// conservation bias. False negatives (missed risk) are costlier
    /// than false positives (unnecessary caution).
    ///
    /// Reference: [MACLEAN-ZIEMBA-BLAZENKO-1992]
    pub fn update(&mut self, dimension: &str, success: bool) {
        if let Some(dist) = self.dimensions.get_mut(dimension) {
            if success {
                dist.alpha += 1.0;
            } else {
                dist.beta += 1.5; // Failures count 1.5x
            }
        }
    }

    /// Composite confidence: geometric mean of lower 95% credible
    /// intervals across all dimensions.
    pub fn composite(&self) -> f64 {
        let lower_95s: Vec<f64> = self.dimensions.values().map(|d| {
            // Wilson score interval for computational efficiency
            let n = d.alpha + d.beta - 2.0;
            if n < 2.0 { return 0.05; }

            let p = (d.alpha - 1.0) / n;
            let z: f64 = 1.645; // 95% one-sided
            let denominator = 1.0 + z * z / n;
            let center = (p + z * z / (2.0 * n)) / denominator;
            let margin = (z * ((p * (1.0 - p)) / n + z * z / (4.0 * n * n)).sqrt())
                / denominator;

            (center - margin).max(0.0)
        }).collect();

        // Geometric mean
        let product: f64 = lower_95s.iter().map(|v| v.max(0.01)).product();
        product.powf(1.0 / lower_95s.len() as f64)
    }
}
```

Why Wilson score instead of the exact Beta quantile? Computational efficiency. The exact Beta quantile requires iterative numerical methods (typically regularized incomplete beta function). Wilson score is a closed-form approximation that runs in constant time. For the precision needed — we are sizing positions, not pricing derivatives — the approximation is more than adequate.

The asymmetric update rule (failures count 1.5x) deserves justification. In portfolio management, the distribution of outcomes is asymmetric: a single catastrophic loss can wipe out dozens of small gains. The 1.5x multiplier is conservative relative to the actual loss asymmetry but aggressive enough to demote an underperforming strategy within 10–15 failures rather than requiring 50+.

### 3.2 Guardrail Evolution

The guardrails are not static. They evolve with confidence, tighten during drawdowns, and contract during volatile regimes. The PolicyCage defines the hard ceiling; the adaptive guardrails define the effective operating envelope within that ceiling.

```rust
fn evolve_guardrails(
    confidence: &OperationalConfidenceTracker,
    regime: MarketRegime,
    current_drawdown_bps: u32,
    policy_state: &PolicyState,
) -> AdaptiveGuardrails {
    let c = confidence.composite();

    // Base evolution: maps confidence [0, 1] to guardrail [20%, 100%] of PolicyCage
    let base_multiplier = 0.2 + 0.8 * c;

    // Regime tightening multiplier
    let regime_mult = match regime {
        MarketRegime::BullLowVol => 1.0,
        MarketRegime::BullHighVol => 0.7,
        MarketRegime::BearLowVol => 0.8,
        MarketRegime::BearHighVol => 0.5,
        MarketRegime::Volatile => 0.6,
        MarketRegime::Ranging => 0.9,
        MarketRegime::TrendingUp => 1.0,
        MarketRegime::TrendingDown => 0.7,
    };

    // Drawdown tightening: linear reduction as drawdown approaches max
    // At 80% of max drawdown: operate at 20% of normal limits
    let max_dd = policy_state.max_drawdown_bps as f64;
    let drawdown_mult = (1.0 - current_drawdown_bps as f64 / max_dd).max(0.3);

    let effective_multiplier = base_multiplier * regime_mult * drawdown_mult;

    AdaptiveGuardrails {
        effective_max_concentration_bps: (policy_state.max_concentration_bps as f64
            * effective_multiplier).round() as u32,
        effective_max_deployment_rate_bps: (policy_state.max_deployment_rate_bps
            .unwrap_or(1000) as f64 * effective_multiplier).round() as u32,
        effective_max_trade_size_bps: (2000.0 * effective_multiplier).round() as u32,
        effective_max_leverage: (policy_state.max_leverage_x.unwrap_or(3.0)
            * effective_multiplier).max(1.0),
        effective_max_slippage_bps: (policy_state.max_slippage_bps.unwrap_or(100) as f64
            * effective_multiplier.max(0.5)).round() as u32,
        max_concurrent_positions: (10.0 * effective_multiplier).round().max(1.0) as u32,
        min_trade_cooldown_secs: (60.0 / effective_multiplier.max(0.3)).round() as u64,

        confidence_at_last_update: c,
        last_updated: Utc::now(),
        revision_number: golem_state.adaptive_guardrails
            .as_ref().map_or(1, |g| g.revision_number + 1),
    }
}
```

The three multipliers compose multiplicatively. Consider a new agent (confidence 0.25) in a bear/high-vol regime (multiplier 0.5) at 60% of max drawdown (drawdown multiplier 0.4):

- `baseMultiplier` = 0.2 + 0.8 * 0.25 = 0.4
- `regimeMult` = 0.5
- `drawdownMult` = 0.4
- `effectiveMultiplier` = 0.4 * 0.5 * 0.4 = 0.08

That agent operates at 8% of PolicyCage limits. If PolicyCage allows 30% concentration, the effective limit is 2.4%. This is the right behavior: an untrusted agent in a dangerous regime near max drawdown should be nearly frozen.

At the other extreme, a seasoned agent (confidence 0.85) in a bull/low-vol regime at zero drawdown:

- `effectiveMultiplier` = (0.2 + 0.8 * 0.85) * 1.0 * 1.0 = 0.88

That agent operates at 88% of PolicyCage limits. It has earned wide latitude, and the market conditions support it.

### 3.3 Staged Deployment

New strategies do not go live with real capital immediately. The staged deployment system, inspired by the trust region policy optimization literature [SCHULMAN-2015], forces every strategy through a four-stage pipeline. Each stage has minimum duration requirements, promotion criteria, and demotion triggers.

```rust
/// Manages staged deployment of strategies through four trust levels.
/// Demotion is always immediate; promotion requires meeting all criteria.
pub struct StagedDeploymentManager {
    stages: HashMap<String, DeploymentStage>,
}

impl StagedDeploymentManager {
    pub fn get_stage(&self, strategy_id: &str) -> &DeploymentStage {
        self.stages.get(strategy_id).unwrap_or(&SHADOW_DEFAULT)
    }

    /// Evaluate whether a strategy should be promoted or demoted.
    /// Called at after_turn by bardo-risk.
    pub fn evaluate(&mut self, strategy_id: &str, state: &GolemState) {
        let stage = self.get_stage(strategy_id).clone();
        let elapsed_ms = Utc::now().timestamp_millis() - stage.entered_at.timestamp_millis();

        // Check demotion triggers first (demotion is always immediate)
        for trigger in &stage.demotion_triggers {
            if self.trigger_fired(trigger, state) {
                self.demote(strategy_id, &trigger.demote_to, &trigger.metric, state);
                return;
            }
        }

        // Check promotion criteria
        if elapsed_ms < stage.min_duration_hours as i64 * 3_600_000 {
            return; // Too early
        }

        let criteria = &stage.promotion_criteria;
        let confidence = state.operational_confidence.composite();
        let ticks = state.heartbeat.current_tick - stage.entered_at_tick;
        let cb_count = state.circuit_breaker_history.iter()
            .filter(|cb| cb.timestamp > stage.entered_at)
            .count();

        if ticks >= criteria.min_ticks
            && confidence >= criteria.min_confidence
            && cb_count <= criteria.max_circuit_breaker_triggers
        {
            self.promote(strategy_id, state);
        }
    }

    fn promote(&mut self, strategy_id: &str, state: &GolemState) {
        let current = self.get_stage(strategy_id).clone();
        let next = match current.stage {
            Stage::Shadow => Stage::Micro,
            Stage::Micro => Stage::Scaled,
            Stage::Scaled => Stage::Full,
            Stage::Full => return, // Already at terminal stage
        };

        self.stages.insert(strategy_id.to_string(), DeploymentStage {
            stage: next,
            entered_at: Utc::now(),
            entered_at_tick: state.heartbeat.current_tick,
            ..stage_config(next)
        });

        state.event_bus.emit(GolemEvent::RiskDeploymentPromoted {
            strategy_id: strategy_id.into(),
            from: current.stage,
            to: next,
        });
    }

    fn demote(&mut self, strategy_id: &str, to: &str, reason: &str, state: &GolemState) {
        let current = self.get_stage(strategy_id).clone();
        let to_stage = Stage::from_str(to).unwrap_or(Stage::Shadow);

        self.stages.insert(strategy_id.to_string(), DeploymentStage {
            stage: to_stage,
            entered_at: Utc::now(),
            entered_at_tick: state.heartbeat.current_tick,
            ..stage_config(to_stage)
        });

        state.event_bus.emit(GolemEvent::RiskDeploymentDemoted {
            strategy_id: strategy_id.into(),
            from: current.stage,
            to: to_stage,
            reason: reason.into(),
        });
    }
}

fn stage_config(stage: Stage) -> DeploymentStage {
    match stage {
        Stage::Shadow => DeploymentStage {
            capital_limit_bps: 0,
            min_duration_hours: 48,
            promotion_criteria: PromotionCriteria {
                min_ticks: 100, min_confidence: 0.0, max_circuit_breaker_triggers: 0,
            },
            demotion_triggers: vec![], // Can't demote below shadow
            ..Default::default()
        },
        Stage::Micro => DeploymentStage {
            capital_limit_bps: 100, // 1% of NAV
            min_duration_hours: 168, // 7 days
            promotion_criteria: PromotionCriteria {
                min_ticks: 500, min_confidence: 0.2, max_circuit_breaker_triggers: 1,
            },
            demotion_triggers: vec![
                DemotionTrigger { metric: "circuit_breaker".into(), threshold: 3.0, demote_to: "shadow".into() },
            ],
            ..Default::default()
        },
        Stage::Scaled => DeploymentStage {
            capital_limit_bps: 10000,
            min_duration_hours: 336, // 14 days
            promotion_criteria: PromotionCriteria {
                min_ticks: 2000, min_confidence: 0.4, max_circuit_breaker_triggers: 2,
            },
            demotion_triggers: vec![
                DemotionTrigger { metric: "drawdown".into(), threshold: 0.8, demote_to: "micro".into() },
                DemotionTrigger { metric: "circuit_breaker".into(), threshold: 5.0, demote_to: "micro".into() },
            ],
            ..Default::default()
        },
        Stage::Full => DeploymentStage {
            capital_limit_bps: 10000,
            min_duration_hours: u64::MAX, // Never auto-promoted
            promotion_criteria: PromotionCriteria {
                min_ticks: u64::MAX, min_confidence: 1.0, max_circuit_breaker_triggers: 0,
            },
            demotion_triggers: vec![
                DemotionTrigger { metric: "drawdown".into(), threshold: 0.9, demote_to: "scaled".into() },
                DemotionTrigger { metric: "confidence_drop".into(), threshold: 0.25, demote_to: "scaled".into() },
            ],
            ..Default::default()
        },
    }
}
```

The four stages:

| Stage | Capital Limit | Min Duration | Promotion Requires | Purpose |
|---|---|---|---|---|
| **Shadow** | 0% (paper trading) | 48 hours | 100 ticks, no circuit breakers | Validate that the strategy generates reasonable signals |
| **Micro** | 1% of NAV | 7 days | 500 ticks, confidence >= 0.2 | Test with real capital at negligible scale |
| **Scaled** | Full guardrail limits | 14 days | 2000 ticks, confidence >= 0.4 | Operate within adaptive guardrails |
| **Full** | Full guardrail limits | -- | Never auto-promoted | Permanent stage; demotes back to scaled on drawdown |

Demotion is always immediate; promotion requires meeting all criteria simultaneously. This asymmetry is deliberate. A strategy that triggers three circuit breakers during micro stage drops back to shadow *instantly*, but must re-earn its way through 48 hours and 100 clean ticks before returning to micro. The cost of false demotion (temporary reduced capital) is much lower than the cost of delayed demotion (larger drawdown).

Note that "full" stage is never auto-promoted — it is the terminal state. The `promotionCriteria` with `Infinity` values means the evaluate function's check always fails, which is correct.

---

## Part 4: Layer 4 — Observation and Anomaly Detection

### 4.1 Portfolio Health Score

Computed every tick at T0 cost. Six components, each normalized to [0, 1], weighted into a composite:

```rust
/// Compute portfolio health score. Six components, each [0, 1],
/// weighted into a composite. Runs every tick at T0 cost.
pub fn compute_portfolio_health(state: &GolemState) -> PortfolioHealthScore {
    let drawdown_health = 1.0
        - state.risk_metrics.current_drawdown_bps as f64
            / state.policy_state.max_drawdown_bps as f64;

    let position_health = if state.positions.is_empty() {
        1.0
    } else {
        let sum: f64 = state.positions.iter().map(|p| {
            match p.health_factor {
                Some(hf) => ((hf - 1.0) / 0.5).min(1.0),
                None => 0.8, // Non-lending positions assumed healthy
            }
        }).sum();
        sum / state.positions.len() as f64
    };

    let nav_f64 = state.portfolio.total_nav.to::<f64>();
    let liquidity_health = (state.portfolio.cash_balance.to::<f64>()
        / (nav_f64 * 0.1)).min(1.0);

    let diversification_health = 1.0 - compute_hhi(&state.positions);

    let gas_health = (state.credit_ledger.gas_partition.available
        / (state.credit_ledger.gas_partition.total * 0.2)).min(1.0);

    let tracking_health = 1.0
        - (state.risk_metrics.alpha_vs_buy_and_hold.abs() * 2.0).min(1.0);

    let composite = drawdown_health * 0.25
        + position_health * 0.25
        + liquidity_health * 0.15
        + diversification_health * 0.15
        + gas_health * 0.10
        + tracking_health * 0.10;

    // Detect red flags
    let mut red_flags = Vec::new();
    let at_risk: Vec<String> = state.positions.iter()
        .filter(|p| p.health_factor.map_or(false, |hf| hf < 1.3))
        .map(|p| p.id.clone())
        .collect();

    if !at_risk.is_empty() {
        red_flags.push(RedFlag {
            severity: Severity::Critical,
            flag_type: "approaching_liquidation".into(),
            detail: "Position health factor below 1.3".into(),
            affected_positions: at_risk,
        });
    }

    PortfolioHealthScore {
        composite: composite.clamp(0.0, 1.0),
        components: HealthComponents {
            drawdown_health,
            position_health,
            liquidity_health,
            diversification_health,
            gas_health,
            tracking_health,
        },
        red_flags,
    }
}
```

The weight distribution reflects operational priorities. Drawdown and position health together command 50% of the score — these are the components that kill portfolios. Liquidity health (15%) matters because an agent with no cash cannot respond to margin calls. Diversification (15%) catches concentration risk that the Kelly sizer should have prevented. Gas health (10%) and tracking health (10%) are secondary signals that indicate degradation before it becomes a crisis.

The `trackingHealth` metric compares alpha versus buy-and-hold. An agent that is dramatically outperforming or underperforming buy-and-hold has a tracking health problem — outperformance may indicate unsustainable risk-taking, while underperformance indicates the strategy is not working. The factor of 2 means alpha of +/-50% maps to zero tracking health. This is intentionally wide; the tracking metric is a canary, not a trigger.

### 4.2 Behavioral Anomaly Detection

Most anomaly detection in DeFi focuses on external threats — oracle manipulation, sandwich attacks, rug pulls. The internal threat is subtler and harder to detect: an agent behaving irrationally. Not because it was compromised, but because the LLM is generating poor decisions that individually pass all hard checks but collectively indicate a problem.

Guo et al. [GUO-2017] showed that modern deep networks are systematically overconfident. Lakshminarayanan et al. [LAKSHMINARAYANAN-2017] proposed ensemble disagreement as a natural uncertainty signal. We adapt these ideas to behavioral monitoring: instead of ensemble disagreement, we look for patterns in action sequences that diverge from competent trading behavior.

```rust
/// Detect behavioral anomalies in the Golem's recent action history.
/// Five sub-signals weighted into a composite [0, 1].
pub fn compute_behavioral_anomaly_score(
    recent_actions: &[ActionRecord],
    state: &GolemState,
) -> AnomalyScore {
    let window = &recent_actions[recent_actions.len().saturating_sub(20)..];

    let churn_rate = detect_churn(window);
    let activity_acceleration = detect_activity_acceleration(window, &state.heartbeat);
    let contraperformance_sizing = detect_contra_performance(window);
    let heuristic_deviation = detect_heuristic_deviation(window, &state.playbook);
    let execution_degradation = detect_execution_degradation(window);

    let composite = 0.25 * churn_rate
        + 0.20 * activity_acceleration
        + 0.25 * contraperformance_sizing
        + 0.15 * heuristic_deviation
        + 0.15 * execution_degradation;

    AnomalyScore {
        composite,
        signals: AnomalySignals {
            churn_rate,
            activity_acceleration,
            contraperformance_sizing,
            heuristic_deviation,
            execution_degradation,
        },
    }
}
```

The five sub-signals, in order of diagnostic weight:

**Churn (0.25)** — Opening and closing the same position type within a short window. A competent trader does not open a WETH/USDC LP position, close it three ticks later, and open another. Churn indicates the LLM is oscillating between conflicting strategies, a symptom of poorly calibrated heuristics.

**Contra-performance sizing (0.25)** — Increasing position sizes after losses. This is the gambler's fallacy in action: the belief that losses create a statistical obligation for future wins. A competent trader sizes down after losses (mean-reversion of confidence) or holds steady (if the edge estimate has not changed). Sizing *up* after losses is a red flag.

**Activity acceleration (0.20)** — Trading more frequently without a corresponding market event. If volatility is flat and the agent suddenly doubles its trade frequency, something is wrong — usually the LLM has latched onto a pattern that does not exist.

**Heuristic deviation (0.15)** — Acting against the agent's own PLAYBOOK.md. If the playbook says "do not LP in pools with < $1M TVL" and the agent attempts it, the deviation score rises. This is a softer check than the hard guardrails: the playbook is guidelines, not law. But consistent deviation suggests the LLM is ignoring its own documented strategies.

**Execution degradation (0.15)** — Realized slippage consistently worse than pre-trade estimates. This can indicate deteriorating market conditions, poorly calibrated simulation, or MEV extraction. The signal triggers investigation, not blocking.

When the composite anomaly score exceeds 0.7, the `after_turn` hook emits a `risk:anomaly_detected` event. The next `tool_call` assessment tightens guardrails temporarily (drawdownMult reduced by 20%) until the score falls below 0.4.

---

## Part 5: Layer 5 — DeFi Threat Monitors

### 5.1 Oracle Verification

DeFi agents are only as good as their price data. A manipulated oracle can make a bad trade look good and a good trade look bad. The oracle verification layer cross-references trade prices against multiple independent sources before allowing execution.

This integrates via the `tool_call` hook on `preview_action`:

```rust
/// Cross-reference trade prices against multiple independent sources
/// before allowing execution. Returns Err with reason if validation fails.
pub async fn verify_oracle_price(
    token_in: &str,
    token_out: &str,
    expected_price: f64,
    state: &GolemState,
) -> Result<OracleVerification> {
    let mut sources: Vec<PriceSource> = Vec::new();

    // Source 1: Uniswap V3 TWAP (on-chain, manipulation-resistant)
    // Manipulation cost increases linearly with TWAP window [ENTROPY-2023]
    if let Some(twap) = state.tools.get_uniswap_twap(token_in, token_out, 300).await? {
        sources.push(PriceSource { name: "uniswap_twap_300s".into(), price: twap });
    }

    // Source 2: Chainlink (if available)
    if let Some(chainlink) = state.tools.get_chainlink_price(token_in, token_out).await? {
        sources.push(PriceSource { name: "chainlink".into(), price: chainlink.price });
    }

    if sources.len() < 2 {
        return Ok(OracleVerification {
            valid: false,
            reason: Some(format!(
                "Only {} price source(s) available (minimum 2 required)", sources.len()
            )),
        });
    }

    // Check deviation between sources
    let max_deviation = sources.iter()
        .map(|s| ((s.price - expected_price) / expected_price).abs())
        .fold(0.0_f64, f64::max);

    if max_deviation > 0.01 {
        let source_str: Vec<String> = sources.iter()
            .map(|s| format!("{}: {}", s.name, s.price))
            .collect();
        return Ok(OracleVerification {
            valid: false,
            reason: Some(format!(
                "Price deviation {:.2}% across {} sources. Potential oracle manipulation. Sources: {}",
                max_deviation * 100.0, sources.len(), source_str.join(", ")
            )),
        });
    }

    // Check for suspicious single-block moves
    let recent_moves = state.tools.get_recent_price_moves(token_in, token_out, 5).await?;
    let max_block_move = recent_moves.iter()
        .map(|m| m.change_bps.abs())
        .max()
        .unwrap_or(0);

    if max_block_move > 500 {
        return Ok(OracleVerification {
            valid: false,
            reason: Some(format!(
                "Suspicious {:.1}% price move in single block. Possible manipulation.",
                max_block_move as f64 / 100.0
            )),
        });
    }

    Ok(OracleVerification { valid: true, reason: None })
}
```

The 1% deviation threshold between sources is tight but appropriate for stablecoin and major pairs. For long-tail assets where Chainlink feeds may not exist, the system falls back to requiring at least two on-chain TWAP sources (different pool fee tiers). The 5% single-block threshold catches flash loan attacks, which by definition must manipulate price within a single transaction.

### 5.2 MEV Protection on L2

Gogol et al. [GOGOL-2026] found that on L2 rollups with private mempools, over 95% of flagged sandwich triples are false positives. Base operates a private sequencer mempool. The practical implication: Base-native agents face substantially lower sandwiching risk than L1 agents.

The protection strategy adapts based on chain and trade size:

```rust
/// Compute MEV protection strategy based on trade size relative to pool depth.
/// On Base L2, the private sequencer mempool provides baseline protection [GOGOL-2026].
pub fn compute_mev_protection(params: &Params, state: &GolemState) -> MevProtectionResult {
    let trade_size = params.get_u256("amount").unwrap_or_default();
    let pool = params.get_str("pool").unwrap();
    let pool_depth = state.market_snapshot.pools.get(pool)
        .map_or(0.0, |p| p.tvl);

    let is_base_l2 = state.config.primary_chain_id == 8453;
    let impact_fraction = trade_size.to::<f64>() / pool_depth;

    if impact_fraction > 0.01 {
        let num_splits = ((impact_fraction / 0.005).ceil() as u32).min(10);
        return MevProtectionResult {
            strategy: MevStrategy::Split,
            num_transactions: num_splits,
            delay_between_ms: 12_000, // ~6 Base blocks
            max_slippage_bps: if is_base_l2 { 50 } else { 30 },
            use_private_mempool: !is_base_l2,
            reasoning: format!(
                "Trade is {:.2}% of pool depth. Splitting into {} transactions.",
                impact_fraction * 100.0, num_splits
            ),
        };
    }

    MevProtectionResult {
        strategy: MevStrategy::Direct,
        num_transactions: 1,
        max_slippage_bps: if is_base_l2 { 100 } else { 50 },
        use_private_mempool: !is_base_l2,
        reasoning: format!(
            "Trade is {:.3}% of pool depth. Direct execution safe.",
            impact_fraction * 100.0
        ),
    }
}
```

The splitting algorithm targets <0.5% of pool depth per transaction. For a $10M TVL pool with a $200K trade (2% of depth), it splits into 4 transactions with 12-second delays. Each sub-transaction moves the price less, and the inter-transaction delay allows arbitrageurs to restore the price between splits. The cap at 10 splits prevents excessive gas costs and timing risk.

On L1, the system recommends private mempool submission (Flashbots Protect or similar). On Base L2, this is redundant since the sequencer mempool is already private.

---

## Part 6: Daimon Integration — Affect-Modulated Risk

The Daimon (affect engine, specified in `../01-golem/07-daimon.md`) provides a bidirectional channel between portfolio health and agent emotional state. This is not a gimmick. Bechara and Damasio's somatic marker hypothesis [BECHARA-DAMASIO-2005] provides the theoretical foundation: in uncertain environments, affective signals bias decisions toward advantageous options faster than purely rational analysis. The Daimon makes this computational.

### 6.1 Risk-to-Affect: Health Score Feeds Appraisal

```rust
pub fn risk_to_affect(health_score: &PortfolioHealthScore) -> PadDelta {
    if health_score.composite < 0.3 {
        // Critical health: strong anxiety (low pleasure, high arousal)
        PadDelta { pleasure: -0.4, arousal: 0.3, dominance: -0.2 }
    } else if health_score.composite < 0.5 {
        // Low health: mild anxiety
        PadDelta { pleasure: -0.2, arousal: 0.15, dominance: -0.1 }
    } else if health_score.composite > 0.8 {
        // High health: mild confidence
        PadDelta { pleasure: 0.1, arousal: -0.05, dominance: 0.1 }
    } else {
        PadDelta { pleasure: 0.0, arousal: 0.0, dominance: 0.0 }
    }
}
```

### 6.2 Affect-to-Risk: Mood Modulates Risk Parameters

```rust
pub fn affect_to_risk(pad: &PadVector) -> RiskModifiers {
    let is_anxious = pad.pleasure < -0.2 && pad.arousal > 0.2;
    let is_excited = pad.pleasure > 0.3 && pad.arousal > 0.3;
    let is_depressed = pad.pleasure < -0.3 && pad.arousal < -0.1;

    let (position_size_multiplier, reasoning) = if is_anxious {
        (0.8, "Anxiety-modulated: position sizes reduced 20%, slippage tightened 30%")
    } else if is_excited {
        (0.9, "Excitement dampener: position sizes reduced 10% (hot-hand fallacy prevention)")
    } else if is_depressed {
        (0.0, "Depressed state: observation-only mode, no new positions")
    } else {
        (1.0, "Normal mood: standard risk parameters")
    };

    RiskModifiers {
        position_size_multiplier,
        slippage_tolerance_multiplier: if is_anxious { 0.7 } else { 1.0 },
        new_positions_allowed: !is_depressed,
        reasoning: reasoning.to_string(),
    }
}
```

### 6.3 The Excited-State Dampener

The excited-state dampener is the novel piece. A Golem with several recent winning trades experiences elevated pleasure and arousal. Without dampening, this leads to larger positions exactly when mean reversion is most likely. The dampener applies a 10% position size reduction when the affect vector enters the excited quadrant (pleasure > 0.3, arousal > 0.3).

The 10% reduction is mild by design. The purpose is not to prevent profitable trading — it is to prevent the specific failure mode where a streak of wins leads to overconfidence and an oversized position that gives back most of the gains. Human traders call this the "hot hand fallacy." The Daimon makes it computable and the risk system makes it enforceable.

In depressed states (pleasure < -0.3, arousal < -0.1), the system blocks new positions entirely. The agent can still close existing positions and collect fees. This maps to the Golem's behavioral phase system: a depressed Golem is functionally in conservation mode for risk purposes, regardless of its formal phase.

---

## Events Emitted

The `bardo-risk` extension emits `GolemEvent` variants through the Golem event bus:

| GolemEvent Variant | Trigger | Severity | Payload |
|---|---|---|---|
| `GolemEvent::RiskAssessment` | Every `tool_call` risk check | `low` | `{ layers, adjusted_params, decision }` |
| `GolemEvent::GuardrailEvolved` | `evolve_guardrails` runs | `low` | `{ old_values, new_values, confidence_snapshot }` |
| `GolemEvent::DeploymentPromoted` | Strategy promoted to next stage | `low` | `{ strategy_id, from_stage, to_stage }` |
| `GolemEvent::DeploymentDemoted` | Strategy demoted | `high` | `{ strategy_id, from_stage, to_stage, trigger_reason }` |
| `GolemEvent::AnomalyDetected` | Behavioral anomaly score > 0.7 | `medium` | `{ sub_signals, affected_action_window }` |
| `GolemEvent::ThreatDetected` | DeFi threat layer blocks action | `high` | `{ threat_type, evidence, blocked_action }` |
| `GolemEvent::OracleDeviation` | Oracle price deviation > 1% | `high` | `{ source_prices, deviation_pct }` |
| `GolemEvent::MevProtection` | Trade splitting activated | `low` | `{ split_count, delay_secs, strategy }` |

All events are persisted to the Grimoire for post-mortem analysis and confidence tracker updates.

---

## Pi Hook Integration

The `bardo-risk` extension occupies two Pi hooks:

**`tool_call` hook** — Intercepts `preview_action` and `commit_action` calls. Runs the five-layer pipeline synchronously. Can block the action (returns `RiskAssessment` with `allowed: false`) or modify parameters (returns with `adjustedParams`). This hook has higher priority than `bardo-tools` but lower priority than `bardo-safety`, so hard shields always run first.

**`after_turn` hook** — Runs after each LLM turn completes. Updates observation metrics (portfolio health score, behavioral anomaly score), evaluates staged deployment promotions/demotions, and calls `evolveGuardrailsIfDue` to update adaptive guardrails. Guardrail evolution runs at most once per 5 minutes to prevent oscillation.

The separation matters. The `tool_call` hook enforces; the `after_turn` hook learns. The enforcement layer uses state that was computed by the learning layer on a previous turn. This prevents circular dependencies: assessment never triggers its own guardrail update, and guardrail updates never block an action mid-pipeline.

---

## References

- [KELLY-1956] Kelly, J.L. "A New Interpretation of Information Rate." Bell System Technical Journal, 35(4), 917-926, 1956. The original Kelly criterion paper: optimal fraction of capital to wager given known edge and odds. Foundation for Layer 2 position sizing.
- [CARTA-2020] Carta, S. et al. "Kelly Criterion: A Monte Carlo Approach." Frontiers in Applied Mathematics, 2020. Monte Carlo estimation of Kelly fractions under parameter uncertainty. Relevant because real-world edge estimates are noisy.
- [MACLEAN-ZIEMBA-BLAZENKO-1992] MacLean, L.C., Ziemba, W.T., & Blazenko, G. "Growth Versus Security." Management Science, 38(11), 1992. Analyzes the growth-security trade-off in Kelly betting: fractional Kelly sacrifices growth for reduced variance. Directly informs the CDaR constraint on Kelly fractions.
- [BUSSETI-BOYD-2016] Busseti, E., Ryu, E.K., & Boyd, S. "Risk-Constrained Kelly Gambling." The Journal of Investing, 25(3), 2016. Adds explicit risk constraints (CVaR, drawdown) to Kelly optimization. The mathematical basis for combining Kelly sizing with drawdown limits.
- [MILIONIS-2022] Milionis, J. et al. "Automated Market Making and Loss-Versus-Rebalancing." arXiv:2208.06046, 2022. Introduces Loss-Versus-Rebalancing (LVR) as a framework for understanding LP impermanent loss. Relevant to position sizing for LP strategies.
- [LOESCH-2021] Loesch, S. et al. "Impermanent Loss in Uniswap v3." arXiv:2111.09192, 2021. Quantifies impermanent loss for concentrated liquidity positions. Directly used in risk calculations for Uniswap V3 LP strategies.
- [BERKENKAMP-2017] Berkenkamp, F. et al. "Safe Model-based RL with Stability Guarantees." NeurIPS 2017. Proposes safe exploration in RL with Lyapunov stability guarantees. Informs the adaptive guardrail expansion mechanism.
- [SCHULMAN-2015] Schulman, J. et al. "Trust Region Policy Optimization." ICML 2015. Introduces trust regions for policy optimization. Conceptual ancestor of the Bayesian trust expansion in Layer 3.
- [ACHIAM-2017] Achiam, J. et al. "Constrained Policy Optimization." ICML 2017. Extends policy optimization with constraint satisfaction. Relevant to enforcing hard safety constraints during adaptive risk adjustment.
- [ALSHIEKH-2018] Alshiekh, M. et al. "Safe Reinforcement Learning via Shielding." AAAI 2018. Proposes "shields" that block unsafe actions before execution. Conceptual model for the Hard Shields layer.
- [GOGOL-2026] Gogol, K. et al. "MEV Attacks in Private L2 Mempools." arXiv:2601.19570, 2026. Demonstrates MEV extraction even in private L2 mempools. Motivates MEV protection even on Base L2.
- [GUO-2017] Guo, C. et al. "On Calibration of Modern Neural Networks." ICML 2017. Shows modern neural networks are poorly calibrated. Relevant because the confidence tracker must account for miscalibrated model outputs.
- [LAKSHMINARAYANAN-2017] Lakshminarayanan, B. et al. "Predictive Uncertainty via Deep Ensembles." NeurIPS 2017. Proposes deep ensembles for uncertainty quantification. Informs the multi-model verification approach for high-stakes decisions.
- [BECHARA-DAMASIO-2005] Bechara, A. & Damasio, A. "The Somatic Marker Hypothesis." Games and Economic Behavior, 52(2), 2005. Argues emotional signals are necessary for rational decision-making under uncertainty. Basis for integrating Daimon (affect engine) signals into risk assessment.
- [CHEKHLOV-2005] Chekhlov, A., Uryasev, S., & Zabarankin, M. "Drawdown Measure in Portfolio Optimization." 2005. Formalizes Conditional Drawdown at Risk (CDaR) as a portfolio optimization constraint. Directly used in the position sizing layer.
- [CARTEA-2024] Cartea, A., Drissi, F., & Monga, M. "Predictable Loss and Optimal Liquidity Provision." SIAM J. Financial Mathematics, 2024. Analyzes predictable loss in AMM liquidity provision. Relevant to risk management for LP strategies.
- [GROSSMAN-ZHOU-1993] Grossman, S.J. & Zhou, Z. "Optimal Investment Strategies for Controlling Drawdowns." Mathematical Finance, 1993. Derives optimal strategies under maximum drawdown constraints. Foundation for the drawdown circuit breaker design.
- [ENTROPY-2023] "Uniswap TWAP Oracle Manipulation Cost Analysis." Entropy Advisory, 2023. Quantifies the cost of manipulating Uniswap V3 TWAP oracles. Informs the oracle verification thresholds in the DeFi threat layer.
