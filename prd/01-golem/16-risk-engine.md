# 16 -- Five-Layer Risk Architecture [SPEC]

**Position Sizing, Adaptive Guardrails, and DeFi Threat Detection**

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-safety`
>
> Cross-references: [01-cognition.md](./01-cognition.md), [13-runtime-extensions.md](./13-runtime-extensions.md), [14-context-governor.md](./14-context-governor.md), [07-safety.md](./07-safety.md), [17-prediction-engine.md](./17-prediction-engine.md) (Oracle accuracy → uncertainty scalar), [18-cortical-state.md](./18-cortical-state.md) (TaCorticalExtension: `learning_convexity` signal)
>
> Sources: `innovations/07-ergodicity-economics`, `innovations/09-antifragile-architecture`

> **Reader orientation:** This document specifies the Golem's (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) five-layer risk architecture: Hard Shields (PolicyCage, the on-chain smart contract enforcing safety constraints on all agent actions), Position Sizing (ergodicity-optimal Kelly), Adaptive Guardrails (Bayesian learning), Observation (portfolio health), and DeFi Threats (oracle verification, MEV protection). It belongs to the `01-golem` safety layer, in the `golem-safety` crate. The key concept: any outer layer can block an action, and no inner layer can override that block. The layers compose, they don't compete. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## 1. Architecture Overview

The risk engine is five independent layers, each with a different enforcement style. They evaluate in order from outermost to innermost. Any layer can block an action; none can override a block from an outer layer.

| Layer | Name | Enforcement | Cost |
|-------|------|-------------|------|
| 1 | Hard Shields | Binary pass/fail | $0 (on-chain policy) |
| 2 | Position Sizing | Ergodicity-optimal Kelly, mortality + uncertainty adjusted | Compute only |
| 3 | Adaptive Guardrails | Bayesian learning, staged deployment | Compute only |
| 4 | Observation | Portfolio health, anomaly detection | Compute only |
| 5 | DeFi Threats | Oracle verification, MEV protection | Gas + latency |

```
┌─────────────────────────────────────────────────┐
│  Layer 1: Hard Shields (PolicyCage)             │
│  ┌─────────────────────────────────────────┐    │
│  │  Layer 2: Position Sizing (Kelly)       │    │
│  │  ┌─────────────────────────────────┐    │    │
│  │  │  Layer 3: Adaptive Guardrails   │    │    │
│  │  │  ┌─────────────────────────┐    │    │    │
│  │  │  │  Layer 4: Observation   │    │    │    │
│  │  │  │  ┌─────────────────┐    │    │    │    │
│  │  │  │  │  Layer 5: DeFi  │    │    │    │    │
│  │  │  │  │    Threats       │    │    │    │    │
│  │  │  │  │                  │    │    │    │    │
│  │  │  │  │   [Action]       │    │    │    │    │
│  │  │  │  └─────────────────┘    │    │    │    │
│  │  │  └─────────────────────────┘    │    │    │
│  │  └─────────────────────────────────┘    │    │
│  └─────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

Every proposed action passes through all five layers, outermost first. Layer 1 is the cheapest and most absolute. Layer 5 is the most expensive and most context-dependent. An action that clears Layer 1 can still be blocked by Layer 3 or resized by Layer 2. The layers compose, they don't compete.

The separation is intentional. Layers 1-3 are domain-agnostic: they'd work for equities, commodities, anything with prices. Layer 4 adds portfolio-level awareness. Layer 5 is DeFi-specific. If you stripped Layer 5, you'd have a general-purpose risk engine. If you stripped Layers 2-5, you'd have a pure policy cage. The design degrades gracefully -- each inner layer adds sophistication but none are load-bearing for safety.

---

## 2. Layer 1 -- Hard Shields

PolicyCage enforcement. Binary pass/fail. Zero computational cost because these constraints live on-chain and are checked before any strategy logic runs. The LLM cannot override them.

**Constraints:**

- **Approved assets**: Whitelist, not blacklist. Only tokens explicitly added to the approved set can be traded or held. Unknown tokens are rejected outright.
- **Max positions**: Default 5, configurable per vault. Prevents overdiversification and cognitive overload for the Golem's reasoning loop.
- **Max single position size**: Percentage of portfolio. Default 25%. Prevents concentration risk regardless of what Kelly says.
- **Max drawdown**: Percentage threshold with two triggers. At 80% of max drawdown, automatic position reduction begins (largest positions first). At 100%, full liquidation to stablecoin.
- **Phase enforcement**: Conservation phase (see 01-cognition.md) blocks opening new positions entirely. The Golem can only reduce exposure during conservation.
- **Rate limits**: Max trades per hour, max rebalances per day. Prevents runaway loops where the Golem repeatedly enters and exits positions.

```rust
use alloy::primitives::Address;
use std::collections::HashSet;

/// Hard Shields configuration. Enforced on-chain via the PolicyCage
/// smart contract. The Golem cannot override these constraints —
/// they exist outside its process entirely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardShields {
    pub approved_assets: HashSet<Address>,
    pub max_positions: u32,
    pub max_position_size_pct: f64,
    pub max_drawdown_pct: f64,
    pub drawdown_warning_pct: f64,
    pub max_trades_per_hour: u32,
    pub max_rebalances_per_day: u32,
    pub blocked_phases: Vec<BehavioralPhase>,
}
```

Hard Shields are the only layer with on-chain enforcement. The remaining four layers run in the Golem's Rust process and could, in theory, be bypassed by a sufficiently adversarial prompt. That's fine -- Layer 1 catches anything catastrophic. The on-chain PolicyCage reads are performed via Alloy's `sol!` macro for type-safe contract interaction (see `07-safety.md`).

The drawdown mechanism deserves emphasis. At 80% of `maxDrawdownPct`, the engine begins forced position reduction: it closes the largest position first, then recalculates. If drawdown is still above 80%, it closes the next largest. At 100%, everything liquidates to the vault's base asset. There is no manual override for this. An owner who wants to prevent liquidation must raise `maxDrawdownPct` before the threshold is hit, which requires an on-chain transaction (not a prompt).

---

## 3. Layer 2 -- Position Sizing (Ergodicity Economics)

Position sizing is not a heuristic. It is the optimization objective for any agent that experiences returns sequentially. Standard finance optimizes expected value (the ensemble average across parallel worlds). A Golem cannot be restarted at the ensemble average. It lives once, in sequence, compounding returns multiplicatively. The correct objective is the time-average growth rate [PETERS-2019].

### Non-Ergodic Dynamics

Wealth evolves multiplicatively: `W(t+1) = W(t) * (1 + r_t)`. After T periods, the time-average growth rate is:

```
g = E[ln(1 + r)]
```

For returns with mean mu and variance sigma^2, a Taylor expansion gives:

```
g ~ mu - sigma^2 / 2
```

The **ergodicity gap** is the difference between expected return and time-average growth:

```
Delta = E[r] - g ~ sigma^2 / 2
```

Delta is always non-negative and grows quadratically with volatility. A strategy with 20% expected annual return and 50% volatility has a gap of 12.5%, giving a time-average growth rate of only 7.5%. At ~63% volatility, the time-average growth rate hits zero. Beyond that, every individual agent goes bankrupt despite positive expected returns.

The Golem tracks both `g` and `Delta` as primary risk metrics, computed every theta tick from rolling return observations. If `g` turns negative, the Golem is on a path to ruin regardless of what `E[r]` says.

### Kelly Criterion from First Principles

If the agent bets fraction `f` of wealth on an opportunity with return `r`, the time-average growth rate becomes:

```
g(f) = E[ln(1 + f * r)]
```

The Kelly fraction `f*` maximizes `g(f)`. For continuously distributed returns with mean mu and variance sigma^2 (and risk-free rate r_f):

```
f* = (mu - r_f) / sigma^2
```

This is the growth-optimal (log-optimal) portfolio. It matches Merton's continuous-time result [MERTON-1969] when the utility function is logarithmic, which is not a coincidence. Logarithmic utility corresponds to maximizing the time-average growth rate. It is not a risk preference -- it is the correct objective for a non-ergodic multiplicative process [PETERS-GELL-MANN-2016].

### Multi-Asset Kelly

For a portfolio of n assets with expected excess return vector `mu` and covariance matrix `Sigma`, the growth-optimal weights are:

```
f* = Sigma^{-1} * mu
```

The matrix inverse means correlated DeFi strategies are sized jointly. Two highly correlated LP positions should not both run at full Kelly because their combined variance exceeds their individual variances.

### Mortality-Adjusted Fractional Kelly

Full Kelly assumes infinite horizon and perfect parameter estimates. Neither holds for Golems.

**Mortality scalar.** A mortal agent's ruin probability depends on remaining lifespan T. Shorter T means less time for the law of large numbers to stabilize the realized growth rate. The variance of realized growth scales as `sigma^2 / T`, so shorter-lived Golems face wider outcome distributions for any bet size.

```
mortality_scalar = min(1, T_remaining / T_reference)
```

`T_reference` is the tick horizon at which full Kelly is appropriate (calibrated to a healthy Golem's expected lifespan). This connects directly to the three mortality clocks: the economic clock measures `g`, the epistemic clock measures estimation uncertainty, and the stochastic clock (Hayflick counter) reduces `T_remaining`.

**Uncertainty scalar.** If the true parameters are `(mu, sigma^2)` but the agent estimates `(mu_hat, sigma_hat^2)` with estimation variance `v = Var[mu_hat]`, the optimal reduction is:

```
uncertainty_scalar = 1 / (1 + v / sigma^2)
```

When `v` equals `sigma^2` (as uncertain about the estimate as returns are volatile), this gives half Kelly. When `v` is large, the agent bets almost nothing. Declining Oracle accuracy (from [17-prediction-engine.md](./17-prediction-engine.md)) maps to rising `v`, automatically reducing position sizes through the uncertainty scalar.

### Combined Position Sizing Formula

```
f = f* * mortality_scalar * uncertainty_scalar
```

This is the single formula that produces sized positions. All other sizing constraints (VaR, drawdown) are consequences, not independent rules.

```rust
/// Rolling estimator for return statistics.
/// Uses exponentially weighted moments for non-stationary environments.
pub struct ReturnEstimator {
    alpha: f64,       // EWM decay (higher = more recent weight)
    ewm_mean: f64,
    ewm_var: f64,
    estimation_var: f64,
    n_obs: u64,
    min_obs: u64,     // Default: 30
}

impl ReturnEstimator {
    pub fn new(alpha: f64, min_obs: u64) -> Self {
        Self {
            alpha,
            ewm_mean: 0.0,
            ewm_var: 0.0,
            estimation_var: f64::INFINITY,
            n_obs: 0,
            min_obs,
        }
    }

    pub fn update(&mut self, r: f64) {
        self.n_obs += 1;
        if self.n_obs == 1 {
            self.ewm_mean = r;
            self.ewm_var = 0.0;
            return;
        }
        let delta = r - self.ewm_mean;
        self.ewm_mean += self.alpha * delta;
        // Welford-style incremental variance with exponential weighting.
        self.ewm_var = (1.0 - self.alpha)
            * (self.ewm_var + self.alpha * delta * delta);
        // Estimation variance decreases with effective sample size.
        let eff_n = (2.0 / self.alpha) - 1.0;
        self.estimation_var = self.ewm_var / eff_n;
    }

    pub fn is_reliable(&self) -> bool { self.n_obs >= self.min_obs }
    pub fn mean(&self) -> f64 { self.ewm_mean }
    pub fn variance(&self) -> f64 { self.ewm_var }
    pub fn estimation_variance(&self) -> f64 { self.estimation_var }
}

/// Ergodicity metrics for a single position or portfolio.
pub struct ErgodicityMetrics {
    /// Time-average growth rate: g = mu - sigma^2 / 2.
    pub growth_rate: f64,
    /// Ergodicity gap: Delta = sigma^2 / 2.
    pub ergodicity_gap: f64,
    /// Full Kelly fraction (before adjustments).
    pub kelly_fraction: f64,
    /// Final fraction after mortality and uncertainty scalars.
    pub adjusted_fraction: f64,
    pub mortality_scalar: f64,
    pub uncertainty_scalar: f64,
}

/// Ergodicity-based position sizer. Replaces heuristic Kelly.
pub struct ErgodicitySizer {
    risk_free_rate: f64,     // Per-tick (e.g., USDC lending rate / ticks_per_year)
    reference_ticks: f64,    // Horizon at which full Kelly is appropriate
    max_fraction: f64,       // PolicyCage hard limit (Layer 1)
    min_growth_rate: f64,    // Portfolio g must stay above this
}

impl ErgodicitySizer {
    /// Compute ergodicity metrics and optimal position size.
    pub fn size_position(
        &self,
        estimator: &ReturnEstimator,
        remaining_ticks: f64,
    ) -> ErgodicityMetrics {
        let mu = estimator.mean();
        let sigma_sq = estimator.variance();
        let v = estimator.estimation_variance();

        let growth_rate = mu - sigma_sq / 2.0;
        let ergodicity_gap = sigma_sq / 2.0;

        let kelly_fraction = if sigma_sq > 1e-12 {
            (mu - self.risk_free_rate) / sigma_sq
        } else {
            0.0
        };

        let mortality_scalar =
            (remaining_ticks / self.reference_ticks).clamp(0.0, 1.0);

        let uncertainty_scalar = if sigma_sq > 1e-12 {
            1.0 / (1.0 + v / sigma_sq)
        } else {
            0.0
        };

        let adjusted_fraction = (kelly_fraction
            * mortality_scalar
            * uncertainty_scalar)
            .clamp(0.0, self.max_fraction);

        ErgodicityMetrics {
            growth_rate,
            ergodicity_gap,
            kelly_fraction,
            adjusted_fraction,
            mortality_scalar,
            uncertainty_scalar,
        }
    }

    /// Portfolio-level growth rate given fractions and covariance.
    /// g_portfolio = sum(f_i * mu_i) - 0.5 * sum_ij(f_i * f_j * Sigma_ij)
    pub fn portfolio_growth_rate(
        &self,
        fractions: &[f64],
        means: &[f64],
        covariance: &[Vec<f64>],
    ) -> (f64, bool) {
        let n = fractions.len();
        let mu_term: f64 = fractions.iter()
            .zip(means.iter())
            .map(|(f, m)| f * m)
            .sum();
        let var_term: f64 = (0..n)
            .flat_map(|i| (0..n).map(move |j| (i, j)))
            .map(|(i, j)| fractions[i] * fractions[j] * covariance[i][j])
            .sum();
        let g = mu_term - 0.5 * var_term;
        (g, g >= self.min_growth_rate)
    }
}
```

### Multi-Asset Covariance Estimation

```rust
/// Exponentially weighted covariance matrix estimator.
pub struct CovarianceEstimator {
    alpha: f64,
    n_assets: usize,
    means: Vec<f64>,
    covariance: Vec<Vec<f64>>,
    n_obs: u64,
}

impl CovarianceEstimator {
    pub fn new(alpha: f64, n_assets: usize) -> Self {
        Self {
            alpha,
            n_assets,
            means: vec![0.0; n_assets],
            covariance: vec![vec![0.0; n_assets]; n_assets],
            n_obs: 0,
        }
    }

    pub fn update(&mut self, returns: &[f64]) {
        assert_eq!(returns.len(), self.n_assets);
        self.n_obs += 1;
        if self.n_obs == 1 {
            self.means = returns.to_vec();
            return;
        }
        let deltas: Vec<f64> = returns.iter()
            .zip(self.means.iter())
            .map(|(r, m)| r - m)
            .collect();
        for i in 0..self.n_assets {
            self.means[i] += self.alpha * deltas[i];
        }
        for i in 0..self.n_assets {
            for j in 0..self.n_assets {
                self.covariance[i][j] = (1.0 - self.alpha)
                    * (self.covariance[i][j]
                        + self.alpha * deltas[i] * deltas[j]);
            }
        }
    }

    pub fn means(&self) -> &[f64] { &self.means }
    pub fn covariance(&self) -> &[Vec<f64>] { &self.covariance }
}
```

### VaR as a Derived Constraint

95% 1-day VaR must not exceed 5% of portfolio per position, 10% portfolio-wide (accounting for correlations via the covariance matrix). VaR is estimated from trailing 30-day realized volatility scaled by position size.

VaR acts as a ceiling on the Kelly fraction. In practice, it binds for high-volatility assets where Kelly suggests a large fraction but the tail risk is unacceptable. For stablecoin pairs, VaR rarely constrains. For long-tail tokens with 100%+ annualized vol, VaR almost always binds.

Under the ergodicity framework, VaR constraints are consequences of the growth rate requirement, not independent rules. The maximum drawdown before `g` becomes unrecoverable given remaining lifespan `T` is:

```
max_drawdown = 1 - exp(-g * T)
```

A Golem with `g = 0.001` per tick and 10,000 remaining ticks tolerates a 99.995% drawdown. A Golem with `g = 0.0001` and 100 remaining ticks tolerates only ~1%. The numbers come from the math, not from intuition.

### LP-Specific Sizing via LVR

For liquidity provision positions, the traditional win/loss framework doesn't apply. Expected loss is modeled via Loss Versus Rebalancing [MILIONIS-2022]:

```
LVR = sigma^2 / (8 * fee_tier) per block
```

Where `sigma` is the asset's per-block volatility and `fee_tier` is the pool's fee rate. LVR replaces the loss-side estimate in the Kelly formula. A pool where LVR exceeds fee income has negative expected value -- the Golem should not provide liquidity there regardless of Kelly output.

For concentrated liquidity positions (Uniswap V3/V4), the range width matters. Tighter ranges amplify both fee income and LVR. The sizing engine treats each LP position as equivalent to a leveraged bet where the leverage factor is `tickRange_full / tickRange_position`. A position concentrated to 10% of full range is roughly 10x leveraged, and its Kelly fraction is divided by 10 accordingly.

Correlated positions interact. Two LP positions in pools sharing a common token (e.g., ETH/USDC and ETH/DAI) have correlated LVR. The sizing engine treats the combined LVR exposure as a single position for VaR purposes, preventing circumvention of position size limits by splitting across correlated pools.

### Risk Budgeting

Total portfolio growth rate must remain positive. Individual positions can have negative `g` if the portfolio `g` is positive (diversification benefit). The constraint `g_portfolio > g_min` replaces arbitrary drawdown limits. Drawdown limits become derived quantities that depend on the Golem's growth rate and remaining lifespan.

### Heartbeat Integration

The ergodicity metrics feed into the 9-step heartbeat pipeline:

| Heartbeat Step | Ergodicity Role |
|---|---|
| Step 1 (OBSERVE) | Collect return data for `g` estimation |
| Step 3 (ANALYZE) | Compute `g`, `Delta`, updated Kelly fractions |
| Step 4 (GATE) | Reject strategies where `g < 0` or `Delta` exceeds threshold |
| Step 5 (SIMULATE) | Project portfolio `g` under proposed trades |
| Step 6 (VALIDATE) | Verify post-trade `g` remains above `g_min` |
| Step 9 (REFLECT) | Update rolling estimates, recalibrate mortality scalars |

---

## 4. Layer 3 -- Adaptive Guardrails

Operational confidence modeled as a Beta distribution. This is distinct from the per-trade confidence in Layer 2 -- this tracks the Golem's overall operational competence across all activities.

### Bayesian Confidence

Prior: `Beta(2, 2)` -- mildly uncertain, centered at 0.5.

Update rules:
- Successful operation: `Beta(alpha + 1, beta)`
- Failed operation: `Beta(alpha, beta + 1)`

Confidence score = `alpha / (alpha + beta)` (posterior mean).

"Success" and "failure" are defined by outcome, not by profit. A trade that executed correctly but lost money is a success (the risk engine worked). A trade that failed to execute, hit unexpected slippage, or triggered a revert is a failure (something in the pipeline broke).

### Regime Modulation

During volatile markets (realized volatility > 2x 30-day average), confidence decays faster. Failed operations multiply the beta increment by 1.5 instead of 1.0. The intuition: failures during volatile periods are more likely to reflect systematic issues rather than bad luck.

During drawdowns exceeding 10% of peak portfolio value, beta increments are multiplied by 2.0. The Golem should become more cautious when it's losing capital, not less.

### Staged Deployment

New strategies never start at full size. They graduate through four stages:

| Stage | Confidence Threshold | Max Position Size | Min Duration |
|-------|---------------------|-------------------|--------------|
| Shadow | < 0.3 | 0% (observe only) | Until confidence > 0.3 |
| Micro | 0.3 -- 0.5 | 2% of portfolio | 48 hours |
| Scaled | 0.5 -- 0.7 | 10% of portfolio | 7 days |
| Full | > 0.7 | Per Kelly/VaR limits | Ongoing |

Progression is not monotonic. A string of failures at the Scaled stage drops confidence below 0.5, and the strategy regresses to Micro. Three consecutive failures at any stage trigger regression to the previous stage regardless of confidence score.

Shadow mode is particularly important for new Golems. A freshly deployed Golem with no track record starts every strategy in Shadow, watches the market for a while, builds confidence through observation, then graduates to Micro. This prevents the common failure mode of deploying an agent and watching it immediately YOLO into positions.

---

## 5. Layer 4 -- Observation

Passive monitoring that doesn't block actions directly but feeds into the Daimon's emotional state (Section 7) and generates alerts for the owner.

### Portfolio Health Score

Six components, equally weighted, each normalized to [0, 1]:

1. **Diversification**: Herfindahl index of position sizes. `H = sum(w_i^2)` where `w_i` is the weight of position `i`. Target: H < 0.3. Score = `max(0, 1 - H/0.3)`.
2. **Liquidity**: Fraction of portfolio in positions with > $100K daily volume. Score = fraction directly.
3. **Correlation**: Maximum pairwise correlation among held positions, trailing 30 days. Target: max correlation < 0.7. Score = `max(0, 1 - maxCorr/0.7)`.
4. **Drawdown**: Current drawdown as fraction of max allowed. Score = `1 - currentDrawdown/maxDrawdownPct`.
5. **Win rate**: Rolling 30-day win rate across all closed positions. Score = win rate directly (capped at 1.0).
6. **Sharpe ratio**: Rolling 30-day Sharpe, annualized. Target > 1.0. Score = `min(1, sharpe)`.

Composite health = arithmetic mean of all six. A score below 0.4 triggers an observation alert. Below 0.2 triggers a distress signal to the owner.

### Behavioral Anomaly Detection

Five boolean signals, each flagged independently:

1. **Churn**: Trade frequency exceeds 2x the 30-day rolling average. May indicate the Golem is thrashing between strategies.
2. **Acceleration**: Position size increased by > 50% between consecutive trades in the same asset. May indicate doubling-down behavior.
3. **Contraperformance**: Adding to losing positions without an updated thesis in the reasoning trace. Distinguishes between conviction averaging and denial.
4. **Heuristic deviation**: Actions that contradict heuristics stored in PLAYBOOK.md (the Golem's own learned rules). The Golem wrote the heuristic, then violated it.
5. **Execution degradation**: Realized slippage exceeding 2x expected slippage on the last 5 trades. May indicate market microstructure problems or stale price data.

Two or more simultaneous flags trigger an observation alert. The alert doesn't block the action -- it modulates the Daimon's arousal (see Section 7) and logs the event for owner review.

Why not block on anomalies? Because anomaly detection has a high false-positive rate. A Golem legitimately rebalancing across several pools after a major market move will trigger Churn. A Golem averaging into a dip with a clear updated thesis should trigger Acceleration but not be stopped. The observation layer's job is to raise salience, not to veto. The Daimon's emotional response to the alert (heightened arousal, reduced pleasure) will naturally tighten sizing through the Section 7 coupling without hard-blocking potentially correct actions.

---

## 6. Layer 5 -- DeFi Threats

Protocol-level risks specific to on-chain execution. These checks run immediately before transaction submission, after all other layers have approved and sized the action.

### Oracle Verification

Two independent price sources must agree before any trade executes:

- **TWAP**: 5-minute on-chain time-weighted average price from Uniswap V3/V4 pools.
- **Chainlink**: Latest round data from the corresponding Chainlink price feed.

Deviation thresholds:
- \> 2% deviation between TWAP and Chainlink: warning logged, trade proceeds with reduced size (50%).
- \> 5% deviation: trade blocked. Oracle disagreement at this level suggests manipulation or extreme volatility.

Staleness check: If the Chainlink feed's last update is older than 1 hour (heartbeat exceeded), the feed is marked stale. The Golem falls back to TWAP only, with a mandatory 25% position size reduction as compensation for single-source risk.

### MEV Protection

Three mitigation strategies, applied based on trade size relative to pool liquidity:

- **Trade splitting**: Any trade exceeding 0.5% of pool liquidity is split into 3-5 sub-trades with 2-block spacing between submissions. This reduces sandwich attack profitability below the attacker's gas costs for most pools.
- **L2 sequencer ordering**: On Base (the primary deployment chain), the sequencer provides FIFO ordering, which eliminates most frontrunning. The Golem prefers Base for large trades when the asset is available there.
- **Private mempool**: Trades exceeding $10K in notional value are routed through Flashbots Protect (or equivalent) to avoid public mempool exposure entirely.

### Composability Risk

Smart contract interaction risks that the Golem must defend against:

- **Flash loan detection**: If the current transaction context originated from a flash loan callback (detectable via call depth and originator analysis), all state-changing operations are blocked. Flash loans are a common vector for price manipulation attacks.
- **Reentrancy guard**: The Golem never calls external contracts during state-changing operations on its own vault. All external calls happen in a read-then-write pattern with checks-effects-interactions ordering.
- **Price impact simulation**: Every trade is simulated via `eth_call` before submission. If simulated price impact exceeds 1% of the quoted price, the trade is rejected and re-quoted with smaller size.

### Governance and Upgrade Risk

Two additional threat vectors that don't fit neatly into oracle/MEV/composability:

- **Token contract upgrades**: If a held token uses a proxy pattern (UUPS, TransparentProxy, or Beacon), the Golem monitors the proxy admin for pending upgrades. A pending upgrade on a held token triggers a warning. An executed upgrade triggers automatic position review -- the Golem re-evaluates the token's contract code before any new trades.
- **Pool parameter changes**: Uniswap V4 hooks can modify pool behavior. If a pool's hook contract is upgraded or its parameters change (detected via event monitoring), the Golem pauses trading in that pool until it has re-evaluated the hook's behavior. This prevents a scenario where a hook upgrade introduces adverse extraction that the Golem's existing strategy doesn't account for.

---

## 7. Daimon-Risk Bidirectional Coupling

The risk engine and the Daimon (emotional subsystem from 01-cognition.md) are not independent. They influence each other through two feedback channels.

### Risk to Affect

Portfolio health score feeds into the OCC appraisal model as an environmental stimulus. The Daimon processes portfolio health the way a human trader processes their P&L -- it shifts mood.

```rust
/// Convert portfolio health into a PAD (Pleasure-Arousal-Dominance) delta
/// for the Daimon emotional subsystem.
///
/// High health: slight pleasure increase, stable arousal.
/// Low health: anxiety — reduced pleasure, elevated arousal.
/// Very low health: threat response — dominance drops.
pub fn risk_to_affect(portfolio_health: f64) -> PadDelta {
    PadDelta {
        pleasure: (portfolio_health - 0.5) * 0.2,
        arousal: if portfolio_health < 0.3 { 0.3 } else { 0.0 },
        dominance: if portfolio_health > 0.7 { 0.1 } else { -0.1 },
    }
}
```

This is a gentle nudge, not a hard override. The PAD delta is small (max 0.3 on any axis). But it accumulates. A Golem that watches its portfolio health degrade over several cycles will drift toward anxiety, which in turn triggers the Affect-to-Risk channel below.

### Affect to Risk

The Daimon's current emotional state modulates risk parameters. Four mood quadrants, four responses:

| Mood | PAD Region | Position Sizing Modifier |
|------|-----------|------------------------|
| Anxiety | Low pleasure, high arousal | -20% (reduce exposure) |
| Excitement | High pleasure, high arousal | -10% (prevent hot-hand bias) |
| Depression | Low pleasure, low arousal | No change (conservative default) |
| Confidence | High pleasure, low arousal | No change (baseline) |

Both negative-affect states reduce sizing, but for different reasons. Anxiety reduces sizing because the Golem is registering threat signals. Excitement reduces sizing because euphoria correlates with overconfidence in both human and artificial traders -- the hot-hand fallacy.

Neither positive state increases sizing beyond the Kelly/VaR baseline. The risk engine has a negativity bias by design: emotions can make you more cautious, never more aggressive.

This follows Damasio's Somatic Marker Hypothesis [DAMASIO-2005]: emotional states encode compressed information about past outcomes. Suppressing them (as most trading bots do) discards useful signal. The Golem instead treats affect as a Bayesian prior on environmental conditions -- anxiety means "conditions similar to past losses," which is genuinely informative for position sizing.

---

## 8. Implementation

The risk engine is a Rust module in the `golem-safety` crate, running in-process with the Golem runtime. It is not part of the inference gateway (see 13-runtime-extensions.md). The inference gateway handles token generation and tool dispatch; risk assessment happens before and after tool calls in the Rust orchestration layer.

Write-tool invocations require a `Capability<T>` token (see `07-safety.md` §2) that is consumed (moved) on use. The risk engine is the sole authority that mints these tokens. Even if the LLM is fully compromised, it cannot execute a write tool without a valid capability — Rust's ownership system prevents forgery and reuse at compile time.

```rust
use alloy::primitives::Address;

/// The five-layer risk engine. Evaluates proposed actions through
/// all layers and produces a sized, approved-or-blocked assessment.
///
/// Write tools require a Capability<T> token to execute.
/// The risk engine is the only code path that can mint capabilities
/// (via pub(crate) constructor). This is the bridge between the
/// risk architecture and the capability-gated tool system.
pub struct RiskEngine {
    hard_shields: HardShields,
    kelly: KellyEngine,
    guardrails: AdaptiveGuardrails,
    observer: PortfolioObserver,
    defi_threats: DeFiThreatChecker,
}

/// Result of passing an action through all five layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub approved: bool,
    pub adjusted_size: f64,
    pub layers: Vec<LayerResult>,
    pub warnings: Vec<String>,
    pub blocks: Vec<String>,
}

/// Result from a single risk layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    pub layer: u8,  // 1–5
    pub passed: bool,
    pub details: String,
    pub adjustment: Option<f64>,
}

impl RiskEngine {
    /// Assess a proposed action through all five layers, outermost first.
    /// If any layer blocks, execution stops immediately.
    pub async fn assess(
        &self,
        action: &ProposedAction,
        state: &PortfolioState,
    ) -> Result<RiskAssessment> {
        let mut assessment = RiskAssessment {
            approved: true,
            adjusted_size: action.requested_size,
            layers: Vec::with_capacity(5),
            warnings: Vec::new(),
            blocks: Vec::new(),
        };

        // Layer 1: Hard Shields (on-chain PolicyCage check via Alloy)
        let l1 = self.hard_shields.check(action, state).await?;
        assessment.layers.push(l1.clone());
        if !l1.passed {
            assessment.approved = false;
            assessment.blocks.push(l1.details.clone());
            return Ok(assessment);
        }

        // Layer 2: Position Sizing (Kelly + VaR)
        let l2 = self.kelly.size(action, state);
        assessment.adjusted_size = l2.adjustment.unwrap_or(assessment.adjusted_size);
        assessment.layers.push(l2);

        // Layer 3: Adaptive Guardrails (Bayesian confidence + staging)
        let l3 = self.guardrails.check(action, state);
        assessment.layers.push(l3.clone());
        if !l3.passed {
            assessment.approved = false;
            assessment.blocks.push(l3.details.clone());
            return Ok(assessment);
        }

        // Layer 4: Observation (anomaly detection, health score)
        let l4 = self.observer.check(action, state);
        assessment.layers.push(l4.clone());
        assessment.warnings.extend(l4.details.lines().map(String::from));

        // Layer 5: DeFi Threats (oracle verification, MEV, composability)
        let l5 = self.defi_threats.check(action, state).await?;
        assessment.layers.push(l5.clone());
        if !l5.passed {
            assessment.approved = false;
            assessment.blocks.push(l5.details.clone());
        }

        Ok(assessment)
    }

    /// Update Layer 3's Beta distribution with an outcome.
    pub fn update_confidence(&mut self, outcome: &ActionOutcome) {
        self.guardrails.update(outcome);
    }

    /// Run Layer 4's observation checks and return portfolio health.
    pub fn get_portfolio_health(&self, state: &PortfolioState) -> PortfolioHealth {
        self.observer.compute_health(state)
    }

    /// Get current deployment stage from Layer 3.
    pub fn get_deployment_stage(&self) -> DeploymentStage {
        self.guardrails.current_stage()
    }
}
```

### Capability Token Integration

When the risk engine approves a write action, it mints a `Capability<T>` token scoped to the specific tool, value limit, and expiry tick. The tool consumes (moves) this token on execution. If the risk engine blocks the action, no capability is minted and the tool physically cannot execute.

```rust
impl RiskEngine {
    /// Mint a capability token for an approved write action.
    /// Only callable after assess() returns approved=true.
    /// The capability is scoped to a single tool type and value limit.
    pub fn mint_capability<T: WriteTool>(
        &self,
        assessment: &RiskAssessment,
        current_tick: u64,
    ) -> Option<Capability<T>> {
        if !assessment.approved {
            return None;
        }
        // pub(crate) constructor — only golem-safety can create these
        Some(Capability::new(
            assessment.adjusted_size,
            current_tick + 3, // valid for 3 ticks
            self.hard_shields.policy_hash(),
            uuid::Uuid::new_v4().to_string(),
        ))
    }
}
```

### Taint Tracking Integration

The risk engine's Layer 1 (Hard Shields) performs on-chain PolicyCage reads via Alloy. The returned data is wrapped in `TaintedString` with `UntrustedExternal` taint labels (see `07-safety.md` §3) to prevent unvalidated on-chain state from flowing directly into strategy parameters without validation.

Execution flow:

1. Golem's reasoning loop proposes an action (trade, LP position, rebalance).
2. `assess()` runs all five layers sequentially. If any layer sets `approved = false`, execution stops and no capability token is minted.
3. `adjusted_size` reflects the cumulative effect of all layer modifications (Kelly sizing, guardrail staging, affect modulation, VaR capping).
4. If approved, `mint_capability()` produces a single-use `Capability<T>` token scoped to the adjusted value.
5. The action executes at the adjusted size, consuming the capability.
6. `update_confidence()` is called with the outcome, updating Layer 3's Beta distribution.
7. `get_portfolio_health()` runs Layer 4's observation checks, feeding results back to the Daimon.

State persistence: Layer 2 and Layer 3 state (Beta distribution parameters, deployment stage, trade history) is stored in the Grimoire alongside other Golem memory. This means risk state survives restarts and is recoverable from the knowledge graph.

### Layer Interaction Example

A concrete walkthrough. The Golem wants to open a new ETH/USDC LP position worth $5,000:

1. **Layer 1 (Hard Shields)**: ETH and USDC are in `approvedAssets`. Current position count is 3, below max 5. $5K is 12% of a $42K portfolio, below the 25% cap. Current drawdown is 4%, well under max. No phase block. Pass.
2. **Layer 2 (Position Sizing)**: ReturnEstimator for ETH/USDC LP shows mu = 0.0003/tick, sigma^2 = 0.0008. Full Kelly: f* = 0.0003/0.0008 = 0.375 (37.5%). Mortality scalar: 8,200 remaining ticks / 10,000 reference = 0.82. Uncertainty scalar: estimation var 0.0004, so 1/(1 + 0.0004/0.0008) = 0.67. Adjusted fraction: 0.375 * 0.82 * 0.67 = 0.206 (20.6%). VaR ceiling at 5% per position binds: 30-day ETH vol at 65% annualized, position VaR would be 13.4% at 20.6%. Clamped to 3.2%. Adjusted size: $1,344 (3.2% of $42K), down from requested $5,000. Growth rate g = 0.0003 - 0.0004 = -0.0001 (slightly negative for this position alone, acceptable if portfolio g remains positive).
3. **Layer 3 (Adaptive Guardrails)**: LP strategy operational confidence at Beta(18, 6), score = 0.75. Stage = Full. No size reduction from guardrails.
4. **Layer 4 (Observation)**: Herfindahl = 0.28, liquidity score = 0.9, max correlation = 0.45, drawdown = 0.16, win rate = 0.63, Sharpe = 1.2. Health = 0.77. No anomaly flags. No alert.
5. **Layer 5 (DeFi Threats)**: TWAP vs Chainlink deviation = 0.3%. Under 2% threshold. Trade size $1,344 is 0.01% of pool liquidity. No splitting needed. Under $10K, no private mempool. Price impact simulation: 0.02%. Pass.

Result: approved at $1,344, down from the requested $5,000. The Golem proceeds with the smaller position. No warnings, no blocks.

---

## 9. Events Emitted

Risk engine events track every layer evaluation, capability minting, and taint violation.

| Event | Trigger | Payload |
|---|---|---|
| `risk:assessment_completed` | All five layers evaluated | `{ approved, adjustedSize, layerResults, warnings }` |
| `risk:hard_shield_blocked` | Layer 1 rejects an action | `{ constraint, action, reason }` |
| `risk:position_sized` | Layer 2 adjusts size | `{ requestedSize, adjustedSize, kellyFraction, mortalityScalar, uncertaintyScalar, growthRate, ergodicityGap }` |
| `risk:stage_transition` | Layer 3 stage changes | `{ fromStage, toStage, confidence, reason }` |
| `risk:anomaly_detected` | Layer 4 flags anomaly | `{ flags, portfolioHealth }` |
| `risk:defi_threat_blocked` | Layer 5 rejects an action | `{ threatType, details }` |
| `risk:capability_minted` | Write action approved | `{ permitId, valueLimit, expiresTick, toolType }` |
| `risk:capability_consumed` | Write tool executed | `{ permitId, actualValue }` |
| `risk:taint_violation` | TaintedString blocked at sink | `{ label, sink, sourceDescription }` |
| `risk:drawdown_warning` | Drawdown exceeds 80% of max | `{ currentDrawdown, maxDrawdown, positionsToReduce }` |
| `risk:forced_liquidation` | Drawdown hits 100% of max | `{ drawdown, positionsClosed }` |
| `risk:growth_rate_negative` | Portfolio g drops below zero | `{ growthRate, ergodicityGap, positions }` |
| `risk:barbell_rebalance` | Exploration/conservative ratio adjusted | `{ newExplorationRatio, trigger, convexityEstimate }` |
| `risk:knowledge_harvested` | Stress-period Grimoire entry created | `{ polarity, originVolatility, residualMagnitude }` |

---

## 10. Antifragile Via Negativa

The five-layer architecture is defensive: it protects the Golem from catastrophic loss. When stress rises, risk limits tighten. When mortality clocks tick faster, position sizes shrink. This is resilient design. A resilient Golem absorbs shocks and returns to baseline. But the Golem that enters a crisis is, at best, the same Golem that exits it. The crisis produced nothing [TALEB-2012].

An antifragile Golem gets stronger from stress. DeFi crises are the most information-dense periods a Golem will experience: correlations break, liquidity vanishes, oracle feeds diverge, AMM curves produce returns no backtest predicted. Every one of these events is a surprise, and every surprise is information. The antifragile architecture harvests this information instead of discarding it.

### Antifragility as Convexity

A system is antifragile if its response function is convex in the perturbation. For Golems, the perturbation is market volatility `sigma(t)` and the response is knowledge acquisition rate `L(sigma)`. The system is antifragile if:

```
L(sigma + delta) + L(sigma - delta) > 2 * L(sigma)
```

By Jensen's inequality [JENSEN-1906], if `L` is convex in sigma and sigma is drawn from any distribution over the Golem's lifetime:

```
E[L(sigma)] >= L(E[sigma])
```

A Golem living through boom-bust cycles learns more, in expectation, than one living through perpetual calm at the same average volatility.

### Barbell Allocation

The barbell partitions the portfolio into two firewalled budgets:

- **Conservative (90%)**: Full five-layer risk stack, ergodicity-optimal Kelly sizing, proven strategies. This portion survives any stress event.
- **Exploration (10%)**: Separate budget, curiosity-driven strategy selection, wider risk limits. Only Layer 1 (Hard Shields) applies as backstop. This portion harvests information from stress.

The firewall is the key property: exploration losses cannot affect the conservative allocation. The 10% exploration budget is an option -- limited downside (at most 10% of capital), unbounded upside (the value of what might be discovered). Options have positive convexity by construction, and their value increases with volatility (the Vega effect).

```rust
/// Barbell budget partition. Conservative allocation uses full risk stack.
/// Exploration allocation bypasses Kelly and anomaly detection.
#[derive(Debug, Clone)]
pub struct BarbellConfig {
    pub conservative_ratio: f64,  // Default: 0.90
    pub exploration_ratio: f64,   // Default: 0.10
    pub min_exploration: f64,     // Loop 3 floor: 0.05
    pub max_exploration: f64,     // Loop 3 ceiling: 0.20
}

/// Tracks exploration budget performance and knowledge yield.
pub struct BarbellAllocator {
    config: BarbellConfig,
    vault_balance: f64,
    exploration_pnl: f64,
    exploration_entries: u64,
}

impl BarbellAllocator {
    pub fn conservative_budget(&self) -> f64 {
        self.vault_balance * self.config.conservative_ratio
    }

    pub fn exploration_budget(&self) -> f64 {
        self.vault_balance * self.config.exploration_ratio
    }

    /// Knowledge entries per USDC lost. Higher is better.
    /// Used by Loop 3 to decide whether to rotate exploration strategy.
    pub fn exploration_efficiency(&self) -> f64 {
        if self.exploration_pnl >= 0.0 { return f64::INFINITY; }
        let cost = self.exploration_pnl.abs();
        if cost < 1e-6 { return f64::INFINITY; }
        self.exploration_entries as f64 / cost
    }

    /// Loop 3 adjustment: +1% per step, clamped to [min, max].
    pub fn adjust_ratio(&mut self, direction: f64) {
        let step = 0.01 * direction.signum();
        let new_explore = (self.config.exploration_ratio + step)
            .clamp(self.config.min_exploration, self.config.max_exploration);
        self.config.exploration_ratio = new_explore;
        self.config.conservative_ratio = 1.0 - new_explore;
    }
}
```

### Via Negativa: K- Knowledge

Two knowledge classes exist in the Grimoire:

- **K+ (positive)**: "Strategy X works in condition Y." Success modes shift with market microstructure, competition, and capital flows.
- **K- (negative)**: "Strategy X fails in condition Z." Failure modes are properties of mechanism design and change slowly. The reason providing liquidity with a lagging oracle loses money is structural -- the information asymmetry is baked into the protocol's architecture.

K- has a longer half-life than K+ because mechanism design changes slowly. K- entries decay at half the rate of K+ entries:

```
demurrage(K-) = base_demurrage * 0.5
demurrage(K+) = base_demurrage * 1.0
```

Stress periods generate K- at a higher rate than calm periods, because stress is when things break. A liquidity crisis that drains three pool types in sequence generates three durable K- entries ("this pool type fails under correlated selling") that remain valid for the protocol's lifetime.

```rust
/// Knowledge polarity: what kind of lesson does this entry encode?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgePolarity {
    Positive, // "do X in condition Y"
    Negative, // "do NOT do X in condition Z"
}

/// Grimoire entry with antifragile metadata.
#[derive(Debug, Clone)]
pub struct AntifragileEntry {
    pub content: String,
    pub polarity: KnowledgePolarity,
    pub stress_origin: bool,
    pub origin_volatility: f64,
    pub residual_magnitude: f64,
    pub demurrage: f64,
    pub created_at: u64,
}

/// Stress-aware knowledge management configuration.
pub struct StressKnowledgeConfig {
    pub base_demurrage: f64,                // Default: 0.001
    pub negative_demurrage_factor: f64,     // Default: 0.5 (K- decays half as fast)
    pub stress_volatility_threshold: f64,   // Default: 0.05
    pub base_write_rate: f64,               // Entries/tick during calm. Default: 1.0
    pub write_rate_alpha: f64,              // Sensitivity to volatility. Default: 2.0
    pub reference_volatility: f64,          // Calm baseline. Default: 0.02
}

impl StressKnowledgeConfig {
    /// Write rate budget scales with volatility:
    /// write_rate = base * (1 + alpha * (sigma/sigma_ref - 1))
    pub fn write_rate(&self, current_volatility: f64) -> f64 {
        let ratio = current_volatility / self.reference_volatility;
        self.base_write_rate
            * (1.0 + self.write_rate_alpha * (ratio - 1.0).max(0.0))
    }

    pub fn demurrage_for(&self, polarity: KnowledgePolarity) -> f64 {
        match polarity {
            KnowledgePolarity::Positive => self.base_demurrage,
            KnowledgePolarity::Negative => {
                self.base_demurrage * self.negative_demurrage_factor
            }
        }
    }
}
```

Over many stress cycles, the Grimoire accumulates a growing body of durable "don't do this" entries that form an immune system against known failure modes. Each crisis makes the next crisis less dangerous.

### Convexity Monitoring

A CorticalState signal on the `TaCorticalExtension` (see [18-cortical-state.md](./18-cortical-state.md)) tracks whether the Golem is actually antifragile:

```
learning_convexity: AtomicU32  // f32, [-1.0, 1.0]
```

Positive values: antifragile (learning more during stress). Negative: fragile (learning less during stress). Zero: resilient (learning rate independent of stress).

The estimator fits a quadratic `L = a * sigma^2 + b * sigma + c` over a rolling window of `(volatility, learning_rate)` pairs. The coefficient `a` is the convexity estimate.

```rust
/// Rolling estimator for learning function convexity L(sigma).
/// Fits L = a*sigma^2 + b*sigma + c; reports `a`.
pub struct ConvexityEstimator {
    window: VecDeque<(f64, f64)>, // (volatility, learning_rate)
    max_size: usize,               // Default: 500
}

impl ConvexityEstimator {
    pub fn push(&mut self, volatility: f64, learning_rate: f64) {
        if self.window.len() == self.max_size {
            self.window.pop_front();
        }
        self.window.push_back((volatility, learning_rate));
    }

    /// Positive a = antifragile, negative = fragile, near-zero = resilient.
    /// Returns None with fewer than 10 observations.
    pub fn estimate_convexity(&self) -> Option<f64> {
        if self.window.len() < 10 { return None; }

        // OLS for quadratic fit via normal equations (X^T X) beta = X^T y.
        let mut xtx = [[0.0_f64; 3]; 3];
        let mut xty = [0.0_f64; 3];

        for &(sigma, l) in &self.window {
            let row = [sigma * sigma, sigma, 1.0];
            for i in 0..3 {
                for j in 0..3 { xtx[i][j] += row[i] * row[j]; }
                xty[i] += row[i] * l;
            }
        }

        let det = det3(&xtx);
        if det.abs() < 1e-12 { return None; }

        // Cramer's rule for coefficient a (column 0).
        let mut m = xtx;
        for i in 0..3 { m[i][0] = xty[i]; }
        Some(det3(&m) / det)
    }
}

fn det3(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}
```

### Loop 3 Adaptation

Loop 3 (meta-learning, days-to-weeks timescale) monitors `learning_convexity` and adjusts the barbell:

| Condition | Duration | Action |
|---|---|---|
| `learning_convexity < 0` | > 48 hours | Increase exploration ratio by 1% (Golem not exploring enough during stress) |
| `learning_convexity > threshold` | > 48 hours | Decrease exploration ratio toward 10% (already antifragile) |
| Exploration budget lost > 50% | Without proportional K- entries | Rotate exploration strategy (burning capital without learning) |

### Cross-Clade Antifragility

When one Golem's exploration allocation discovers a failure mode and the Golem dies as a result, the K- entry is shared with the Clade via the bloodstain protocol (Styx relay). The Golem's death is the Clade's gain. The Clade as a whole is antifragile even if individual members are not: it benefits from the stress that kills its components. This mirrors biological immunity -- the immune system learns from pathogens that kill individual cells.

---

## Ergodicity Economics: Missing Derivations and Content

> Source: `innovations/07-ergodicity-economics.md`

### The State of DeFi Agents

Most autonomous DeFi agents optimize expected value. They calculate expected returns on strategies, pick the highest one, and size positions by fixed fractions or by heuristic risk limits. This makes them systematically over-leveraged for any individual agent's survival. They are optimizing the ensemble average, but each agent lives in the time average. The result is predictable: high paper returns punctuated by ruin.

### Binary Kelly Derivation

For a binary outcome (win b with probability p, lose a with probability 1-p):

```
g(f) = p * ln(1 + f * b) + (1 - p) * ln(1 - f * a)
```

Setting dg/df = 0:

```
p * b / (1 + f * b) = (1 - p) * a / (1 - f * a)
```

Solving for f:

```
f* = (p * b - (1 - p) * a) / (a * b)
```

This is the Kelly criterion. It is exact for binary outcomes, not an approximation. When the expected return p*b - (1-p)*a is negative, f* is negative (meaning: take the other side of the bet, or don't bet at all). When the expected return is zero, f* = 0. The Kelly fraction is always less than the fraction that maximizes expected value (which would be f = 1 whenever E[r] > 0).

### Ruin Probability Formula

For a process with time-average growth rate g and volatility sigma, the probability of hitting a lower barrier W_min starting from W_0 is approximately:

```
P(ruin) ~ (W_min / W_0)^(2g / sigma^2)    when g > 0
```

When g <= 0, ruin probability approaches 1 as T grows. For finite T, lower g means faster convergence to ruin.

The practical consequence: a Golem with 100 remaining ticks should bet smaller fractions than one with 10,000 remaining ticks. The reason is that the shorter-lived Golem has less time for the law of large numbers to pull the realized growth rate toward its expectation. Variance in the realized growth rate scales as sigma^2 / T, so shorter lifespans mean wider distributions of outcomes and higher ruin probability for any given bet size.

### Fractional Kelly Derivation Steps

Full Kelly (f = f*) is optimal only when you know mu and sigma exactly. In practice, these are estimated from data. The estimation error inflates the effective variance and can make full Kelly too aggressive.

If the true parameters are (mu, sigma^2) but the agent estimates (mu_hat, sigma_hat^2) with estimation variance v = Var[mu_hat], the growth rate under the estimated Kelly fraction is:

```
g(f*_hat) ~ g(f*) - (f*^2 * v) / 2
```

The optimal reduction to account for estimation uncertainty is:

```
f_adjusted = f* / (1 + v / sigma^2)
```

This is the "fractional Kelly" result. When estimation variance v is zero, you get full Kelly. When v equals sigma^2 (you're as uncertain about your estimate as the returns are volatile), you get half Kelly. When v is large relative to sigma^2, you bet almost nothing.

The combined sizing formula:

```
f = f* * mortality_scalar * uncertainty_scalar
```

where:
- mortality_scalar = min(1, T_remaining / T_reference)
- uncertainty_scalar = 1 / (1 + v / sigma^2)

### Risk Budgeting Math

Total portfolio growth rate must remain positive. Individual positions can have negative g if the portfolio g is positive (diversification benefit). The risk budget is:

```
g_portfolio = sum_i f_i * mu_i - (1/2) sum_i sum_j f_i * f_j * Sigma_ij
```

The constraint g_portfolio > g_min (where g_min is a small positive threshold) replaces arbitrary drawdown limits. Drawdown limits become consequences: the maximum drawdown before g becomes unrecoverable given remaining lifespan T is:

```
max_drawdown = 1 - exp(-g * T)
```

This ties drawdown tolerance directly to the Golem's remaining life and current growth rate. A Golem with high g and long remaining life can tolerate deeper drawdowns. A Golem near death with low g cannot afford any drawdown at all.

### Integration with the 7-Layer Extension DAG

The ergodicity metrics feed into the Golem's 9-step heartbeat:

- **Observe (step 1):** Collect return data for g estimation
- **Analyze (step 3):** Compute g, Delta, and updated Kelly fractions
- **Gate (step 4):** Reject strategies where g < 0 or Delta exceeds threshold
- **Simulate (step 5):** Project portfolio g under proposed trades
- **Validate (step 6):** Verify post-trade g remains above g_min
- **Reflect (step 9):** Update rolling estimates, recalibrate mortality scalars

### St. Petersburg Paradox Resolution

The St. Petersburg paradox (1738) asks: how much would you pay to play a game where a coin is flipped until it lands heads, and you receive 2^n dollars if heads first appears on flip n? The expected value is infinite. Yet no rational person pays more than a modest sum.

Peters (2019) showed that this explanation is unnecessary. The time-average growth rate of the St. Petersburg game is:

```
g = sum from n=1 to infinity of (1/2)^n * ln(2^n) = ln(2) * sum from n=1 to infinity of n * (1/2)^n = 2 * ln(2) ~ 1.39
```

This is finite. A person maximizing their time-average growth rate would pay up to e^g - 1 ~ $3 to play. No utility function needed. The resolution is not psychological but dynamical: the expected value is a property of the ensemble, the time average is a property of the individual, and they differ because the game is multiplicative.

This reframing has a direct consequence for Golems. We do not need to specify a utility function for a Golem. We do not need to decide whether a Golem is "risk averse" or "risk neutral." The correct objective (maximize time-average growth rate) follows from the dynamics of the process. A Golem is not risk averse by preference. It bets conservatively because it is mortal and returns are multiplicative, and any other strategy is mathematically suboptimal for its survival.

### Mortality as Architectural Advantage

Immortal agents (infinite time horizon) can, in theory, tolerate any finite drawdown. They have infinite time for the law of large numbers to do its work. This makes immortal agents overconfident and fragile: they take on risks that would be rational only for entities that cannot die.

Mortal agents, by contrast, must account for ruin. The shorter the remaining lifespan, the more conservative the optimal strategy. This is not a weakness. It is an architectural advantage. Mortal agents are forced by the mathematics to size positions correctly for their actual situation. They cannot pretend they have infinite time.

Bardo's mortality clocks enforce this advantage. The Hayflick limit prevents a Golem from assuming it will live forever. The economic clock prevents it from ignoring capital depletion. The epistemic clock prevents it from trusting stale predictions. Together, they force the Golem into the regime where ergodicity-optimal sizing dominates expected-value-optimal sizing.

### Ergodicity Evaluation and Falsifiability

**Null hypothesis:** Kelly-optimal sizing, adjusted for mortality and estimation uncertainty, performs no better than fixed-fraction sizing (e.g., always bet 2% of capital) for mortal agents.

**Experimental design:** Monte Carlo simulation of Golem lifetimes under three sizing regimes:

1. **Expected-value optimal:** Size positions to maximize E[r]. In practice, this means betting large whenever expected return is positive.
2. **Fixed-fraction:** Bet a constant 2% of capital on every opportunity, regardless of expected return or volatility.
3. **Ergodicity-optimal:** Size positions using the mortality-adjusted fractional Kelly formula derived above.

Each simulation draws returns from distributions calibrated to historical DeFi strategy returns (mean 0.1% per tick, volatility 1-5% per tick, with fat tails modeled as a Student-t distribution with 5 degrees of freedom). Mortality is modeled with a Hayflick limit of 50,000 ticks and random catastrophic events at a rate of 1 per 10,000 ticks.

**Metrics:**

- Survival rate at T ticks: Fraction of simulated Golems with wealth above the terminal threshold.
- Median terminal wealth: The 50th percentile of wealth across surviving Golems.
- Mean terminal wealth: The arithmetic average, included to show the divergence from median.
- Maximum drawdown distribution.
- Growth rate distribution: The distribution of realized g across Golems.

**Predictions:**

1. Expected-value-optimal Golems will have the highest mean terminal wealth and the lowest survival rate. Most go bankrupt; the few survivors are extremely wealthy, pulling the mean up.
2. Fixed-fraction Golems will have moderate survival rates but suboptimal growth, because fixed fractions do not adapt to changing volatility.
3. Ergodicity-optimal Golems will have the highest median terminal wealth and the highest survival rate. Their mean terminal wealth will be lower than the expected-value-optimal group because they sacrifice upside for survival.

The distinguishing prediction: the ratio of median to mean terminal wealth will be closest to 1 for the ergodicity-optimal group. A wide gap between mean and median is the fingerprint of non-ergodic dynamics being mishandled.

**Falsification criteria:** If fixed-fraction sizing achieves survival rates within 5% of ergodicity-optimal sizing across a range of volatility regimes, the additional complexity is not justified. If expected-value-optimal sizing achieves higher median (not mean) terminal wealth than ergodicity-optimal sizing, the theoretical framework is wrong.

---

## Antifragile Architecture: Missing Derivations and Content

> Source: `innovations/09-antifragile-architecture.md`

### Jensen's Inequality and Mortal Learning

Jensen's inequality (Jensen, 1906) states that for a convex function f and a random variable X:

```
E[f(X)] >= f(E[X])
```

Apply this to the Golem's learning function. Let sigma be market volatility, drawn from some distribution over the Golem's lifetime. If L(sigma) is convex in sigma, then:

```
E[L(sigma)] >= L(E[sigma])
```

The expected learning rate under variable volatility exceeds the learning rate at average volatility. A Golem living through boom-bust cycles learns more, in expectation, than a Golem living through perpetual calm at the same average volatility level.

For mortal Golems, total lifetime knowledge K is the integral of learning rate over the Golem's lifespan:

```
K = integral from 0 to T of L(sigma(t)) dt
```

If L is convex in sigma, then a volatile environment produces higher K than a calm environment with the same time-averaged volatility, even though the Golem has the same finite lifespan T. The mortal Golem in a volatile market produces more knowledge before death than an identical Golem in a stable market. Mortality combined with convex learning is an advantage in volatile environments.

### Optionality Math

The exploration allocation is an option. Options have positive convexity by construction: limited downside (the premium paid, here 10% of capital), unlimited upside (the value of what might be discovered).

Define the value of the exploration option at time t:

```
V_explore(t) = max(0, K_discovered(t) - K_cost(t))
```

where K_discovered is the value of knowledge gained from exploration and K_cost is the USDC spent on the exploration allocation. The max(0, ...) structure is the option payoff. If exploration produces nothing useful, the Golem loses at most 10% of capital. If exploration discovers a durable strategy or a durable failure mode, the payoff can be orders of magnitude larger than the cost.

During high-volatility periods, the option's value increases. This is exactly analogous to how financial options increase in value with volatility (the Vega effect). The exploration allocation becomes more valuable during crises because there is more to discover.

### Via Negativa: Anti-Knowledge as the Antifragile Substrate

Taleb (2012) argues that knowledge about what does not work (via negativa) is more durable than knowledge about what does work (via positiva). The history of medicine illustrates this: we are more confident that bloodletting does not cure disease than we are confident in any specific treatment. Failure modes are stable. Success modes shift.

Two knowledge classes:

- **K+** (positive knowledge): "Strategy X works in condition Y." Example: "Providing liquidity to the ETH/USDC pool on Uniswap v3 with a 0.3% fee tier produces positive returns when realized volatility is below 40% annualized."
- **K-** (negative knowledge): "Strategy X fails in condition Z." Example: "Never provide liquidity to any pool where the oracle update frequency exceeds 10x the block time. The oracle lag creates an arbitrage that consistently drains LPs."

K- has a longer half-life than K+ because failure modes are properties of mechanism design, and mechanism design changes slowly. The reason providing liquidity with a lagging oracle loses money is structural: the information asymmetry between the oracle and the true price is baked into the protocol's architecture. This failure mode persists until the protocol is redesigned. By contrast, the conditions under which a specific strategy produces positive returns depend on market microstructure, competition, and capital flows, all of which shift constantly.

In Grimoire terms, this translates directly to demurrage rates:

```
demurrage(K-) < demurrage(K+)
```

Stress periods generate K- at a higher rate than calm periods, because stress is when things break. Over time, the Golem accumulates a growing body of durable negative knowledge, most of it produced during crises. Each crisis makes the Golem harder to kill, because the Golem has learned more ways to die and can avoid them.

### Volatility Harvesting Math

Define information density as a function of market volatility:

```
I(sigma) = bits of useful information per tick at volatility sigma
```

The hypothesis is that I is convex in sigma:

```
I(sigma + delta) + I(sigma - delta) > 2I(sigma)
```

The argument is empirical and structural. During calm periods, price movements are small, predictions are accurate, residuals are small, and few surprises occur. During crises, prices move sharply, predictions fail, residuals are large, correlations shift, and many surprises occur simultaneously. The information content per tick grows faster than linearly with volatility because crisis dynamics are combinatorial: if n protocol interactions can break, and stress tests them simultaneously, the number of observable failure combinations grows as O(2^n) while volatility grows linearly.

The barbell convexity proof: the key equation combines Jensen's inequality with the barbell:

```
E[L_barbell(sigma)] = 0.9 * E[L_conservative(sigma)] + 0.1 * E[L_explore(sigma)]
```

L_conservative is approximately constant in sigma (conservative strategies don't learn much regardless of conditions). L_explore is convex in sigma (exploration learns disproportionately more during stress). The weighted sum is convex because any positive linear combination of a constant function and a convex function is convex. The barbell inherits the exploration allocation's convexity while bounding the downside with the conservative allocation's stability.

### Write Rate Formula and Dream Cycle Prioritization

Stress-aware write rate scales with volatility:

```
write_rate(sigma) = base_rate * (1 + alpha * (sigma / sigma_ref - 1))
```

where alpha controls sensitivity and sigma_ref is the reference (calm) volatility. When sigma = sigma_ref, write_rate = base_rate. When sigma = 2 * sigma_ref, write_rate = base_rate * (1 + alpha).

During NREM replay (offline consolidation), episodes from stress periods receive higher replay priority. The consolidation engine weights episodes by their information content, estimated from prediction residual magnitude. Stress-period episodes have larger residuals and therefore higher replay weight. This biases the Golem's offline learning toward crisis-generated knowledge.

### Loop 3: 50% Budget Criterion

Loop 3 (meta-learning, operating on a days-to-weeks timescale) monitors the `learning_convexity` signal and adjusts the barbell parameters if the Golem is not actually antifragile:

- If `learning_convexity` < 0 for more than 48 hours, Loop 3 increases the exploration allocation from 10% to 15% (the Golem is not exploring enough during stress).
- If `learning_convexity` > threshold for more than 48 hours, Loop 3 decreases the exploration allocation toward 10% (the Golem is already antifragile, no need to over-allocate to exploration).
- If the exploration allocation has lost more than **50% of its budget** without generating proportional K- entries, Loop 3 rotates the exploration strategy (the current exploration approach is burning capital without learning).

### Seneca's Asymmetry (Shannon Quantification)

Seneca observed that we learn more from a single catastrophe than from a thousand calm days. In information-theoretic terms, the surprise of an event is -log(p), where p is the event's probability. Low-probability events carry more information per occurrence. A flash crash that happens once a year carries -log(1/365) ~ 8.5 bits of information. A normal trading day carries -log(364/365) ~ 0.004 bits. The flash crash is 2,000x more informative per occurrence.

The current system treats the flash crash as a threat and shuts down learning. The antifragile system treats it as a windfall and maximizes learning. Seneca's asymmetry, quantified via Shannon, is the information-theoretic justification for stress-aware write rates and dream-cycle prioritization of stress episodes.

### Antifragile Evaluation and Falsifiability

**Null hypothesis:** Antifragile Golems produce no more knowledge than resilient Golems in volatile environments. The barbell structure and stress-aware knowledge tagging add complexity without measurable benefit.

**Test 1: Knowledge acquisition convexity.** Run two populations of Golems on the same historical market data (Base L2, 180-day window spanning at least two high-volatility events). Control group: standard risk architecture, no barbell, uniform demurrage, no stress-aware write rate. Treatment group: barbell allocation, differential demurrage, stress-aware write rate, convexity monitoring.

Fit the quadratic L = a * sigma^2 + b * sigma + c for each group. Prediction: the treatment group has a > 0 (convex, antifragile). The control group has a <= 0 (concave or linear, fragile or resilient).

**Test 2: Knowledge half-life.** Compare the half-life of K- entries generated during stress periods versus K+ entries from calm periods within the antifragile group. Prediction: K- entries from stress periods remain actionable for at least 2x as long as K+ entries from calm periods.

**Test 3: Survival plus knowledge.** Compare the product of survival_rate * total_knowledge at T = 50,000 ticks. Prediction: the composite metric is higher for the antifragile group.

**Falsification criteria:**

1. The treatment group's convexity coefficient a is not statistically greater than zero (p < 0.05) across 1,000 simulation runs.
2. K- entries from stress periods do not last longer than K+ entries from calm periods in at least 80% of simulations.
3. The survival_rate * total_knowledge product is lower for the antifragile group in more than 50% of volatility regimes tested.

---

## 11. Topological Market Intelligence (Persistent Homology for Regime Detection)

> Source: `innovations/01-topological-market-intelligence.md`

Statistical regime detectors reduce high-dimensional market observations to a handful of moments (mean, variance, skewness) and threshold them. This misses structural changes. When a market regime shifts, the **shape** of the observation distribution changes before its **moments** do. Persistent Homology, a tool from Topological Data Analysis (TDA), tracks the birth and death of geometric features in point clouds as a scale parameter sweeps from zero to infinity. Applied to the Golem's observation stream, TDA-based regime detection identifies structural precursors to regime shifts 10-100 perception ticks before conventional statistical methods.

### Point Clouds from Observations

At each gamma tick *t*, the perception layer produces an observation vector:

```
x_t = (p_t, v_t, sigma_t, g_t, l_t, s_t, ...) in R^d
```

A sliding window of *N* recent observations forms a point cloud `X_N` in R^d. This point cloud is the input to TDA. Its shape encodes the market's structural state.

### Persistent Homology and Persistence Diagrams

The **Vietoris-Rips complex** VR(X, epsilon) connects every pair of points within distance epsilon, fills in triangles, and continues to higher dimensions. As epsilon increases from 0 to infinity, features are *born* and *die*:

- **H_0 (connected components):** Clusters in observation space. Many persistent H_0 features indicate market fragmentation.
- **H_1 (loops):** Persistent cycles. Mean-reverting markets create loops in price-return space. Trending markets do not.
- **H_2 (voids):** Regions of observation space previously occupied but now empty. Correlation breakdowns create voids.

The output is a **persistence diagram**: a multiset of (birth, death) points. Features far from the diagonal d = b are significant; features near the diagonal are noise. The **Stability Theorem** (Cohen-Steiner, Edelsbrunner, Harer 2007) guarantees that small perturbations to the point cloud produce small perturbations to the diagram.

### Wasserstein Distance for Regime Change Detection

The **Wasserstein distance** between two persistence diagrams quantifies structural difference:

```
W_p(D_1, D_2) = (inf_gamma sum_{x in D_1} ||x - gamma(x)||_inf^p)^(1/p)
```

The Golem maintains a reference diagram D_ref for the current regime. At each gamma tick, it computes D_now and measures W_p(D_now, D_ref). A spike in this distance signals structural change.

### Topological Regime Signatures

| Regime | W_p | beta_0 | beta_1 | beta_2 | Description |
|--------|-----|--------|--------|--------|-------------|
| Calm | Low | ~1 | ~0 | 0 | One cluster, no loops, no voids |
| Trending | Low | ~1 | 0 | 0 | One cluster, drifting in diagram space |
| Volatile | Moderate | >1 | >0 | 0 | Fragmented clusters, cyclic patterns |
| Crisis | High | >>1 | >0 | >0 | Extreme fragmentation, voids |

### TDA Pipeline (Step 3 ANALYZE)

TDA runs at Step 3 of the 9-step heartbeat at gamma frequency:

1. **Observation buffer** collects the last N = 200 gamma-tick observation vectors.
2. **PCA** projects from R^d to R^5 (Vietoris-Rips complexity is exponential in dimension).
3. **Ripser** computes persistent homology up to dimension 2 (H_0, H_1, H_2).
4. **Wasserstein distance** compares current diagram to reference.
5. **CorticalState update** writes topological signals.

### New CorticalState Signals

```
topology_signal: AtomicU32  -- f32: Wasserstein distance from reference diagram
betti_0: AtomicU16          -- H_0 features at optimal scale epsilon*
betti_1: AtomicU16          -- H_1 features at optimal scale epsilon*
```

The optimal scale epsilon* is the epsilon that maximizes total persistence of alive features.

### Implementation

```rust
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};

/// A (birth, death) pair from persistent homology.
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceFeature {
    pub birth: f32,
    pub death: f32,
    pub dimension: u8,
}

impl PersistenceFeature {
    pub fn persistence(&self) -> f32 {
        self.death - self.birth
    }
}

/// A persistence diagram: collection of features grouped by dimension.
#[derive(Clone, Debug)]
pub struct PersistenceDiagram {
    pub features: Vec<PersistenceFeature>,
}

impl PersistenceDiagram {
    pub fn features_of_dim(&self, dim: u8) -> Vec<&PersistenceFeature> {
        self.features.iter().filter(|f| f.dimension == dim).collect()
    }

    /// Betti number at scale epsilon for dimension k.
    pub fn betti(&self, dim: u8, epsilon: f32) -> usize {
        self.features
            .iter()
            .filter(|f| f.dimension == dim && f.birth <= epsilon && f.death > epsilon)
            .count()
    }

    /// Scale that maximizes total persistence of alive features.
    pub fn optimal_scale(&self) -> f32 {
        let mut events: Vec<f32> = self
            .features
            .iter()
            .flat_map(|f| vec![f.birth, f.death])
            .collect();
        events.sort_by(|a, b| a.partial_cmp(b).unwrap());
        events.dedup();

        let mut best_eps = 0.0_f32;
        let mut best_score = 0.0_f32;

        for &eps in &events {
            let score: f32 = self
                .features
                .iter()
                .filter(|f| f.birth <= eps && f.death > eps)
                .map(|f| f.persistence())
                .sum();
            if score > best_score {
                best_score = score;
                best_eps = eps;
            }
        }
        best_eps
    }
}

/// Compute the p-Wasserstein distance between two persistence diagrams
/// for a given homological dimension.
pub fn wasserstein_distance(
    d1: &PersistenceDiagram,
    d2: &PersistenceDiagram,
    dim: u8,
    p: f32,
) -> f32 {
    let f1: Vec<(f32, f32)> = d1
        .features_of_dim(dim)
        .iter()
        .map(|f| (f.birth, f.death))
        .collect();
    let f2: Vec<(f32, f32)> = d2
        .features_of_dim(dim)
        .iter()
        .map(|f| (f.birth, f.death))
        .collect();

    let size = f1.len() + f2.len();
    if size == 0 {
        return 0.0;
    }

    let mut cost = vec![vec![0.0_f32; size]; size];

    for i in 0..f1.len() {
        for j in 0..f2.len() {
            let dx = f1[i].0 - f2[j].0;
            let dy = f1[i].1 - f2[j].1;
            cost[i][j] = (dx.abs().max(dy.abs())).powf(p);
        }
        for j in f2.len()..size {
            let diag_cost = (f1[i].1 - f1[i].0) / 2.0;
            cost[i][j] = diag_cost.powf(p);
        }
    }
    for i in f1.len()..size {
        for j in 0..f2.len() {
            let diag_cost = (f2[j].1 - f2[j].0) / 2.0;
            cost[i][j] = diag_cost.powf(p);
        }
    }

    let assignment = hungarian(&cost);
    let total: f32 = assignment
        .iter()
        .enumerate()
        .map(|(i, &j)| cost[i][j])
        .sum();
    total.powf(1.0 / p)
}

/// Placeholder for the Hungarian algorithm (O(n^3) assignment).
fn hungarian(cost: &[Vec<f32>]) -> Vec<usize> {
    (0..cost.len()).collect() // identity assignment placeholder
}
```

### TDA Analyzer

```rust
/// Topological signals written to CorticalState.
pub struct TopologySignals {
    pub topology_signal: AtomicU32,
    pub betti_0: AtomicU16,
    pub betti_1: AtomicU16,
}

pub struct TdaConfig {
    pub window_size: usize,      // 200
    pub pca_dimensions: usize,   // 5
    pub max_homology_dim: usize, // 2
    pub reference_tau: f32,      // 50.0 (exponential decay for reference)
    pub wasserstein_p: f32,      // 2.0
}

impl Default for TdaConfig {
    fn default() -> Self {
        Self {
            window_size: 200,
            pca_dimensions: 5,
            max_homology_dim: 2,
            reference_tau: 50.0,
            wasserstein_p: 2.0,
        }
    }
}

pub struct TdaAnalyzer {
    config: TdaConfig,
    reference_diagram: Option<PersistenceDiagram>,
    tick_count: u64,
}

impl TdaAnalyzer {
    pub fn new(config: TdaConfig) -> Self {
        Self { config, reference_diagram: None, tick_count: 0 }
    }

    pub fn analyze(
        &mut self,
        observations: &[Vec<f32>],
        signals: &TopologySignals,
    ) {
        if observations.len() < self.config.window_size {
            return;
        }

        let projected = pca_project(observations, self.config.pca_dimensions);
        let diagram = compute_persistence(&projected, self.config.max_homology_dim);

        let w_dist = match &self.reference_diagram {
            Some(ref_diag) => {
                let w0 = wasserstein_distance(&diagram, ref_diag, 0, self.config.wasserstein_p);
                let w1 = wasserstein_distance(&diagram, ref_diag, 1, self.config.wasserstein_p);
                let w2 = wasserstein_distance(&diagram, ref_diag, 2, self.config.wasserstein_p);
                (w0 * w0 + w1 * w1 + w2 * w2).sqrt()
            }
            None => 0.0,
        };

        let eps_star = diagram.optimal_scale();
        let b0 = diagram.betti(0, eps_star) as u16;
        let b1 = diagram.betti(1, eps_star) as u16;

        signals.topology_signal.store(w_dist.to_bits(), Ordering::Relaxed);
        signals.betti_0.store(b0, Ordering::Relaxed);
        signals.betti_1.store(b1, Ordering::Relaxed);

        self.update_reference(&diagram);
        self.tick_count += 1;
    }

    fn update_reference(&mut self, current: &PersistenceDiagram) {
        let alpha = 1.0 / self.config.reference_tau;
        match &mut self.reference_diagram {
            Some(ref_diag) => {
                for feat in &mut ref_diag.features {
                    feat.birth = (1.0 - alpha) * feat.birth;
                    feat.death = (1.0 - alpha) * feat.death;
                }
                for feat in &current.features {
                    ref_diag.features.push(PersistenceFeature {
                        birth: alpha * feat.birth,
                        death: alpha * feat.death,
                        dimension: feat.dimension,
                    });
                }
                ref_diag.features.retain(|f| f.persistence() > 1e-6);
            }
            None => {
                self.reference_diagram = Some(current.clone());
            }
        }
    }
}

fn pca_project(observations: &[Vec<f32>], target_dim: usize) -> Vec<Vec<f32>> {
    observations
        .iter()
        .map(|obs| obs[..target_dim.min(obs.len())].to_vec())
        .collect()
}

fn compute_persistence(points: &[Vec<f32>], max_dim: usize) -> PersistenceDiagram {
    let _ = (points, max_dim);
    PersistenceDiagram { features: Vec::new() }
}
```

### Computational Budget

Ripser on 200 points in R^5 completes in under 50ms. PCA adds 1-2ms. Wasserstein distance on ~50 features takes <1ms. Total: under 55ms per gamma tick, well within the 5-second minimum gamma interval.

### Why TDA Detects Regime Changes Early

When a market regime shifts, a subset of observations begins diverging from the main cluster. The mean barely moves. Variance ticks up slightly. But topologically, beta_0 has increased from 1 to 2: a new connected component has appeared. Statistical detectors wait until the divergent subgroup moves aggregate variance past a threshold. Topological detectors fire when the subgroup first separates, regardless of size. Gidea and Katz (2018) demonstrated this empirically on equity data, showing that topological features detected pre-crash conditions in the S&P 500, DJIA, and NASDAQ days before the crashes of 2000, 2007, and 2015.

### Cross-System Integration

The Wasserstein distance W_p becomes a bid in the attention auction (14b-attention-auction.md). High topological change means high information value, attracting more cognitive resources. Betti curves can be encoded as hyperdimensional binary vectors for fast similarity search. Sheaf-theoretic consistency checking operates on the same Vietoris-Rips complex.

---

## 12. Sheaf-Theoretic Multiscale Observation

> Source: `innovations/04-sheaf-theoretic-multiscale-observation.md`

A Golem watches the market through three timescales simultaneously: gamma (5-15s raw ticks), theta (30-120s cognitive patterns), and delta (~50 theta-ticks structural consolidation). These timescales sometimes contradict each other. Today, detecting contradiction depends on the LLM noticing it during deliberation. Sheaf theory on a temporal poset formalizes multi-timeframe consistency, where the first cohomology group H^1 measures exactly how much the timescales disagree. The result is a computable scalar -- the consistency score -- that turns "the short-term data contradicts the long-term trend" from a vague intuition into a number the agent can act on.

### The Temporal Poset

Timescales nest. Every gamma interval is contained in some theta interval, and every theta interval in some delta interval. This containment defines a partial order on observation intervals:

```
gamma_i <= theta_j   iff   gamma_i's time interval is contained in theta_j's
theta_j <= delta_k   iff   theta_j's time interval is contained in delta_k's
```

### Presheaf of Observations

A presheaf F on the temporal poset assigns:

- F(gamma_i) in R^8: price, volume, gas, spread, order flow imbalance, volatility, price velocity, price acceleration
- F(theta_j) in R^6: trend direction, trend strength, regime classification, correlation structure, prediction confidence, pattern match score
- F(delta_k) in R^4: strategy P&L, prediction accuracy, knowledge quality, long-term volatility

Restriction maps rho_{theta_j, gamma_i}: R^6 -> R^8 encode what the theta-level observation *implies* about a specific gamma interval. These maps are where the domain knowledge lives.

### Cohomology Measures the Obstruction

The first cohomology group H^1(T, F) measures the obstruction to gluing local observations into a global one. When H^1 = 0, data is globally consistent. When dim(H^1) > 0, there are irreconcilable contradictions.

The **Hodge Laplacian** provides a continuous measure:

```
L_1 = d_0^T d_0 + d_1 d_1^T
```

The consistency score is:

```
c = 1 - (lambda_max(L_1) / lambda_ref)
```

where lambda_max is the largest eigenvalue, lambda_ref is calibrated from history. The score lives in [0, 1]: 1 = perfect consistency, 0 = maximum observed contradiction.

### New CorticalState Signals

```
sheaf_consistency: AtomicU32   -- f32 in [0, 1], consistency score
contradiction_dimension: AtomicU8  -- dim(H^1), independent irreconcilable contradictions
```

### Implementation

```rust
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

#[derive(Clone, Debug)]
pub struct ObsGamma {
    pub features: [f32; 8],
    pub timestamp_ms: u64,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub struct ObsTheta {
    pub features: [f32; 6],
    pub timestamp_ms: u64,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub struct ObsDelta {
    pub features: [f32; 4],
    pub timestamp_ms: u64,
    pub duration_ms: u32,
}

pub struct RestrictionMap {
    pub weights: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
}

impl RestrictionMap {
    pub fn apply(&self, input: &[f32]) -> Vec<f32> {
        assert_eq!(input.len(), self.cols);
        let mut output = vec![0.0f32; self.rows];
        for i in 0..self.rows {
            for j in 0..self.cols {
                output[i] += self.weights[i * self.cols + j] * input[j];
            }
        }
        output
    }
}

#[derive(Clone, Debug)]
pub struct SheafEdge {
    pub fine_idx: usize,
    pub coarse_idx: usize,
    pub disagreement: f32,
}

#[derive(Clone, Debug)]
pub struct SheafResult {
    pub consistency_score: f32,
    pub contradiction_dim: u8,
    pub max_eigenvalue: f32,
    pub edge_disagreements: Vec<f32>,
}

pub fn compute_sheaf_consistency(
    gammas: &[ObsGamma],
    thetas: &[ObsTheta],
    deltas: &[ObsDelta],
    rho_tg: &RestrictionMap,
    rho_dt: &RestrictionMap,
    lambda_ref: f32,
) -> SheafResult {
    let mut edges: Vec<SheafEdge> = Vec::new();

    for (ti, theta) in thetas.iter().enumerate() {
        let theta_start = theta.timestamp_ms;
        let theta_end = theta.timestamp_ms + theta.duration_ms as u64;
        for (gi, gamma) in gammas.iter().enumerate() {
            let gamma_start = gamma.timestamp_ms;
            let gamma_end = gamma.timestamp_ms + gamma.duration_ms as u64;
            if gamma_start >= theta_start && gamma_end <= theta_end {
                let predicted = rho_tg.apply(&theta.features);
                let disagreement = l2_distance(&predicted, &gamma.features);
                edges.push(SheafEdge {
                    fine_idx: gi,
                    coarse_idx: gammas.len() + ti,
                    disagreement,
                });
            }
        }
    }

    for (di, delta) in deltas.iter().enumerate() {
        let delta_start = delta.timestamp_ms;
        let delta_end = delta.timestamp_ms + delta.duration_ms as u64;
        for (ti, theta) in thetas.iter().enumerate() {
            let theta_start = theta.timestamp_ms;
            let theta_end = theta.timestamp_ms + theta.duration_ms as u64;
            if theta_start >= delta_start && theta_end <= delta_end {
                let predicted = rho_dt.apply(&delta.features);
                let disagreement = l2_distance(&predicted, &theta.features);
                edges.push(SheafEdge {
                    fine_idx: gammas.len() + ti,
                    coarse_idx: gammas.len() + thetas.len() + di,
                    disagreement,
                });
            }
        }
    }

    if edges.is_empty() {
        return SheafResult {
            consistency_score: 1.0,
            contradiction_dim: 0,
            max_eigenvalue: 0.0,
            edge_disagreements: vec![],
        };
    }

    let n_edges = edges.len();
    let mut laplacian = vec![0.0f32; n_edges * n_edges];

    for i in 0..n_edges {
        laplacian[i * n_edges + i] = 2.0;
        for j in 0..n_edges {
            if i == j { continue; }
            let share_fine = edges[i].fine_idx == edges[j].fine_idx
                || edges[i].fine_idx == edges[j].coarse_idx;
            let share_coarse = edges[i].coarse_idx == edges[j].fine_idx
                || edges[i].coarse_idx == edges[j].coarse_idx;
            if share_fine || share_coarse {
                laplacian[i * n_edges + j] = -1.0;
            }
        }
    }

    for i in 0..n_edges {
        for j in 0..n_edges {
            laplacian[i * n_edges + j] *= edges[i].disagreement * edges[j].disagreement;
        }
    }

    let max_eigenvalue = power_iteration(&laplacian, n_edges, 20);
    let contradiction_dim = count_near_zero_eigenvalues(&laplacian, n_edges, 1e-6);
    let consistency_score = (1.0 - (max_eigenvalue / lambda_ref)).clamp(0.0, 1.0);

    SheafResult {
        consistency_score,
        contradiction_dim,
        max_eigenvalue,
        edge_disagreements: edges.iter().map(|e| e.disagreement).collect(),
    }
}

fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    let min_len = a.len().min(b.len());
    let mut sum = 0.0f32;
    for i in 0..min_len {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

fn power_iteration(matrix: &[f32], n: usize, iterations: usize) -> f32 {
    let mut v = vec![1.0f32 / (n as f32).sqrt(); n];
    let mut eigenvalue = 0.0f32;
    for _ in 0..iterations {
        let mut w = vec![0.0f32; n];
        for i in 0..n {
            for j in 0..n {
                w[i] += matrix[i * n + j] * v[j];
            }
        }
        eigenvalue = v.iter().zip(w.iter()).map(|(a, b)| a * b).sum();
        let norm: f32 = w.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm < 1e-10 { return 0.0; }
        for x in &mut w { *x /= norm; }
        v = w;
    }
    eigenvalue
}

fn count_near_zero_eigenvalues(matrix: &[f32], n: usize, tol: f32) -> u8 {
    if n == 0 { return 0; }
    if n == 1 { return if matrix[0].abs() < tol { 1 } else { 0 }; }
    let mut mat = matrix.to_vec();
    let mut rank = 0;
    for col in 0..n {
        let mut pivot = None;
        for row in rank..n {
            if mat[row * n + col].abs() > tol {
                pivot = Some(row);
                break;
            }
        }
        let Some(pivot_row) = pivot else { continue };
        for k in 0..n { mat.swap(rank * n + k, pivot_row * n + k); }
        for row in (rank + 1)..n {
            let factor = mat[row * n + col] / mat[rank * n + col];
            for k in col..n { mat[row * n + k] -= factor * mat[rank * n + k]; }
        }
        rank += 1;
    }
    (n - rank) as u8
}
```

### Performance

The poset has ~10-15 nodes at any time (5-8 gamma, 3-5 theta, 1-2 delta), producing ~8-12 edges. The Hodge Laplacian is at most 12x12. The entire computation completes in under 10 microseconds. Memory: a few hundred bytes.

### Downstream Consumers

The consistency score feeds three subsystems:

- **Heartbeat pipeline:** At Step 3 (ANALYZE), if sheaf_consistency < 0.5, a "multiscale contradiction" flag is set for the LLM at Step 5 (DELIBERATE).
- **Attention auction:** Inconsistency is information. When sheaf consistency drops, the attention auction assigns higher value to the contradicting signals.
- **Mortality:** Persistent low consistency degrades the mutual information estimate I(G; M). Chronic disagreement that cannot be resolved means the model is breaking down.

### Position Sizing Under Contradiction

High sheaf inconsistency maps directly to a position-sizing multiplier: at consistency 1.0, trade at full size; at 0.5, halve it; below 0.3, reduce to minimum. This integrates with the ergodicity-optimal Kelly sizing in Layer 2 as an additional multiplicative scalar.

### Extension to Spatial Observations in a Clade

Replace the temporal poset with a spatio-temporal poset. Nodes become (agent, timescale) pairs. Spatial edges exist between agents observing the same time interval at the same scale. The restriction maps for spatial edges translate between observation spaces using known cross-rate relationships. Now H^1 captures both temporal contradictions (an agent's timescales disagree) and spatial contradictions (agents disagree with each other) in a single algebraic object.

---

## 13. Compensation and Rollback (from source 05-compensation-rollback)

Multi-step DeFi operations (borrow USDC on Aave, swap to WETH on Uniswap, deposit on Morpho) are not idempotent. Partial execution leaves the Golem in an intermediate state that is neither the intended final state nor the original state. Two complementary primitives handle this: `CompensationChain` (saga pattern for semantic rollback) and `RollbackCheckpoint` (full-state save points).

### CompensationChain

Garcia-Molina and Salem (1987) defined the Saga pattern: a long-lived transaction is decomposed into sub-transactions, each paired with a compensating transaction. If the saga fails at step N, compensations execute in reverse order from step N-1 to step 1. On-chain transactions cannot be undone, only compensated.

```rust
// crates/golem-runtime/src/compensation_chain.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub u64);

pub struct CompensationContext {
    pub chain_client: std::sync::Arc<dyn ChainClient>,
    pub current_nonce: u64,
    pub wallet: WalletAddress,
}

pub trait ChainClient: Send + Sync {
    fn execute_compensation(
        &self,
        ctx: &CompensationContext,
        idempotency_key: [u8; 32],
        payload: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), CompensationError>> + Send + '_>>;
}

#[derive(Debug)]
pub enum CompensationError {
    ChainError(String),
    IdempotencyConflict,
    ContextUnavailable,
}

pub trait CompensationAction: Send + Sync {
    fn compensate<'a>(
        &'a self,
        ctx: &'a CompensationContext,
    ) -> Pin<Box<dyn Future<Output = Result<(), CompensationError>> + Send + 'a>>;
}

pub struct CompensationStep {
    pub step_name: &'static str,
    pub action: Box<dyn CompensationAction>,
    pub idempotency_key: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ChainState {
    Active,
    Completed,
    Compensating,
    Compensated,
    CompensationFailed,
}

pub struct CompensationChain {
    id: WorkflowId,
    compensations: Vec<CompensationStep>,
    pub state: ChainState,
}

impl CompensationChain {
    pub fn new(id: WorkflowId) -> Self { /* ... */ }

    pub fn register<A: CompensationAction + 'static>(
        &mut self, name: &'static str, action: A,
    ) { /* ... */ }

    pub fn complete(&mut self) { self.state = ChainState::Completed; }

    /// Execute compensations in reverse registration order.
    /// Continues past individual step failures; records each error.
    pub async fn rollback(
        &mut self, ctx: &CompensationContext,
    ) -> ChainState { /* ... */ }
}
```

**Idempotency**: The `idempotency_key` is `blake3(workflow_id_bytes || step_name_bytes)[..32]`. If the agent crashes during compensation and restarts, re-executing the same compensation with the same key produces no additional on-chain effect.

**Compensation failure**: When a compensation step fails, the chain continues executing remaining compensations in reverse order. A failed compensation at step 3 does not prevent compensations at steps 2 and 1. `CompensationFailed` signals the operator that manual intervention is needed.

### RollbackCheckpoint

Named, intentionally-placed full-state save points at semantically meaningful moments (`"pre_defi_borrow"`, `"post_dream_consolidation"`). Uses CBOR + Blake3 for serialization and integrity verification.

```rust
// crates/golem-runtime/src/rollback_checkpoint.rs

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RollbackCheckpoint {
    pub name: &'static str,
    pub created_at_unix_secs: u64,
    pub tick: u64,
    pub content_hash: [u8; 32],
    pub cortical: CorticalSnapshot,
    pub context: ContextSnapshot,
    pub ledger: PositionLedgerSnapshot,
    pub grimoire_ref: GrimoireRef,
}
```

### Composition: How the Two Primitives Work Together

A typical DeFi workflow uses both:

1. Save a `RollbackCheckpoint` named `"pre_workflow"`.
2. Create a `CompensationChain` with the workflow ID.
3. Execute step 1 (borrow). On success, register the compensation (repay borrow).
4. Execute step 2 (swap). On success, register the compensation (reverse swap).
5. Execute step 3 (deposit). On success, mark the chain `Completed`.
6. If step 3 fails: call `chain.rollback()` to execute compensations in reverse.
7. If rollback succeeds: state is semantically equivalent to pre-workflow.
8. If rollback fails: restore the `"pre_workflow"` checkpoint for clean state, then alert the operator.

The checkpoint is the backstop. The compensation chain handles normal failure paths. The checkpoint handles the case where compensation itself fails.

---

## 14. References

- [KELLY-1956] Kelly, J.L. "A New Interpretation of Information Rate." _Bell System Technical Journal_, 35(4), 1956. — *Derives the optimal fraction of capital to wager for maximum long-run growth; the core formula behind Golem's position sizing layer.*
- [PETERS-2019] Peters, O. "The Ergodicity Problem in Economics." _Nature Physics_, 15(12), 2019. — *Shows that time-average and ensemble-average returns diverge for multiplicative processes; the fundamental insight that makes Kelly sizing correct and expected-utility maximization wrong for mortal agents.*
- [PETERS-GELL-MANN-2016] Peters, O. & Gell-Mann, M. "Evaluating Gambles Using Dynamics." _Chaos_, 26(2), 2016. — *Proves that maximizing time-average growth rate is the unique objective consistent with non-ergodic multiplicative dynamics; eliminates the need to specify a Golem's "risk preference."*
- [MERTON-1969] Merton, R.C. "Lifetime Portfolio Selection Under Uncertainty: The Continuous-Time Case." _Review of Economics and Statistics_, 51(3), 1969. — *Extends portfolio theory to continuous time with logarithmic utility; converges to Kelly when utility is log, confirming the risk engine's growth-rate objective.*
- [THORP-2006] Thorp, E.O. "The Kelly Criterion in Blackjack, Sports Betting, and the Stock Market." In: _Handbook of Asset and Liability Management_, Vol. 1, 2006. — *Practical guide to applying Kelly sizing across domains with estimation uncertainty; informs the half-Kelly and fractional-Kelly adjustments used by the risk engine.*
- [TALEB-2012] Taleb, N.N. _Antifragile: Things That Gain from Disorder_. Random House, 2012. — *Introduces antifragility as a property of systems that gain from volatility; the conceptual basis for the risk engine's volatility-adjusted write rates and Jensen's inequality exploitation.*
- [TALEB-DOUADY-2013] Taleb, N.N. & Douady, R. "Mathematical Definition, Mapping, and Detection of (Anti)Fragility." _Quantitative Finance_, 13(11), 2013. — *Formalizes antifragility as positive sensitivity of payoff to volatility of the underlying; the mathematical definition the risk engine uses to measure portfolio convexity.*
- [JENSEN-1906] Jensen, J.L.W.V. "Sur les fonctions convexes et les inegalites entre les valeurs moyennes." _Acta Mathematica_, 30, 1906. — *Proves that for convex functions, the function of the mean is less than the mean of the function; the inequality that makes volatility beneficial for convex payoffs and harmful for concave ones.*
- [MILIONIS-2022] Milionis, J. et al. "Automated Market Making and Loss-Versus-Rebalancing." 2022. — *Quantifies the cost of providing AMM liquidity as loss-versus-rebalancing (LVR); the risk engine uses LVR estimates to adjust position sizing for LP positions.*
- [DAMASIO-2005] Damasio, A. _Descartes' Error: Emotion, Reason, and the Human Brain_. 2005. — *Argues that emotional signals (somatic markers) are necessary for rational decision-making; the neuroscience basis for integrating Daimon affect signals into risk assessment.*
- [OMOHUNDRO-2008] Omohundro, S. "The Basic AI Drives." _AGI_, 2008. — *Identifies self-preservation and resource acquisition as convergent instrumental goals of rational agents; the safety concern that motivates the risk engine's hard caps and PolicyCage constraints.*
- [TURNER-2021] Turner, A.M. et al. "Optimal Policies Tend to Seek Power." _NeurIPS_, 2021. — *Proves that optimal policies in MDPs tend toward states with more options; motivates the risk engine's constraint layers that prevent unbounded capital concentration.*
- [DENNIS-VAN-HORN-1966] Dennis, J.B. & Van Horn, E.C. "Programming Semantics for Multiprogrammed Computations." _CACM_, 9(3), 1966. — *Introduces capability-based access control; the security model behind the risk engine's per-layer capability grants preventing bypass.*
- [SAMUELSON-1979] Samuelson, P.A. "Why We Should Not Make Mean Log of Wealth Big Though Years to Act Are Long." _Journal of Banking & Finance_, 3(4), 1979. — *The classic critique of Kelly criterion for finite horizons; the risk engine addresses this by applying a mortality scalar that reduces Kelly fraction as remaining lifespan shrinks.*
- [MANDELBROT-HUDSON-2004] Mandelbrot, B. & Hudson, R.L. _The (Mis)behavior of Markets: A Fractal View of Financial Turbulence_. Basic Books, 2004. — *Shows that financial returns have fat tails and clustered volatility that Gaussian models miss; motivates the risk engine's regime-aware volatility estimation over fixed-variance assumptions.*
- [KAHNEMAN-TVERSKY-1979] Kahneman, D. & Tversky, A. "Prospect Theory: An Analysis of Decision Under Risk." _Econometrica_, 47(2), 1979. — *Documents systematic biases in human risk assessment (loss aversion, probability weighting); the risk engine deliberately avoids these biases by using growth-rate optimization instead of subjective utility.*
- [HOLLAND-1995] Holland, J.H. _Hidden Order: How Adaptation Builds Complexity_. Addison-Wesley, 1995. — *Describes adaptive systems using internal models and credit assignment; informs the risk engine's feedback mechanism where trade outcomes update return estimates.*
- [GARCIA-MOLINA-1987] Garcia-Molina, H. & Salem, K. "Sagas." _ACM SIGMOD Record_, 16(3), 1987. — *Introduces the saga pattern for long-lived transactions with compensating actions; the model for the risk engine's multi-step trade rollback via CompensationChain.*
- [GRAY-1992] Gray, J. & Reuter, A. _Transaction Processing: Concepts and Techniques_. Morgan Kaufmann, 1992. — *Definitive treatment of ACID transactions, logging, and recovery; informs the risk engine's checkpoint and rollback infrastructure.*
- [MOHAN-1992] Mohan, C., et al. "ARIES: A transaction recovery method." _ACM TODS_, 17(1), 1992. — *Introduces write-ahead logging with physiological redo and logical undo; the recovery algorithm pattern behind the risk engine's RollbackCheckpoint.*
- [ELNOZAHY-2002] Elnozahy, E.N., et al. "A survey of rollback-recovery protocols." _ACM Computing Surveys_, 34(3), 2002. — *Comprehensive survey of checkpoint and rollback strategies; informs the tradeoff between checkpoint frequency and recovery cost in the risk engine.*
- [RICHARDSON-2018] Richardson, C. _Microservices Patterns_. Manning, 2018. — *Covers saga orchestration and compensating transactions in distributed systems; the architectural pattern for the risk engine's multi-step trade execution and reversal.*
