# DeFi-Native Technical Analysis: Indicators Impossible in Traditional Finance [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Source**: `tmp/research/witness-research/new/ta/07-defi-native-technical-analysis.md`
>
> **Depends on**: Doc 0 (Overview), Doc 1 (HDC Encoding)

**Audience:** Researchers and engineers familiar with Rust, DeFi protocol mechanics, and the Bardo system architecture (Doc 0).

---

> **Reader orientation:** This document develops ten families of DeFi-native indicators that cannot exist in traditional finance, because the data they require is structurally unavailable in opaque markets. It belongs to the **TA research layer** (Doc 7 of 10) within the Golem (mortal autonomous DeFi agent) runtime and covers concentrated liquidity shape analysis, lending market indicators, perpetual funding rate signals, on-chain options metrics, yield term structures, vault mechanics, restaking flows, streaming payments, gas market signals, and cross-primitive composites. You should understand multiple DeFi protocol types (AMMs, lending, perps, options). For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Abstract

Traditional financial markets hide their state. Order books are partially visible. Position data is delayed or aggregated. Liquidity is inferred from trade impact, not observed directly. Technical analysis in that world operates on the shadows cast by hidden market structure -- price and volume are the only reliable signals because everything else is opaque.

DeFi inverts this. Every pool's reserves, every position's health factor, every funding rate payment, every vault's share price, every gas auction's clearing price -- all of it sits in public smart contract storage, queryable by anyone with an RPC endpoint. The entire market microstructure is transparent. This creates a category of technical indicators that cannot exist in traditional finance, not because nobody thought of them, but because the data they require is structurally unavailable.

This document develops ten families of DeFi-native indicators for the Bardo runtime, plus cross-primitive composites that combine signals across protocol types. Each family targets a specific DeFi primitive -- concentrated liquidity, lending markets, perpetual funding rates, on-chain options, yield term structures, vault mechanics, restaking flows, streaming payments, gas markets, and emerging primitives. Every indicator includes a full Rust implementation integrated with the `TaAnalyzer` trait from Doc 0. Cross-primitive composites (the DeFi Stress Index, Capital Rotation Indicator, Cross-Protocol Momentum, Implied Correlation Index) require simultaneous observation of multiple protocol types -- something no single traditional data feed provides. The complete indicator suite runs within one Gamma tick's compute budget through careful use of streaming algorithms and incremental updates.

## The problem [SPEC]

A human trader watching a Uniswap V3 pool sees a price chart. Maybe a volume chart below it. That is what the frontend shows. But the contract stores the full tick bitmap -- a 256x256 grid of initialized ticks, each carrying net liquidity deltas. It stores every position's tick range, liquidity amount, and accumulated fees. It stores the observation ring buffer for TWAP calculations. The price chart is a projection from a high-dimensional state space down to one dimension. Almost all the information is lost.

The same pattern repeats across every DeFi primitive. Aave publishes utilization rates and interest rates on a dashboard, but the contract stores individual position health factors, liquidation thresholds per collateral type, and the exact parameterization of the interest rate curve. Pendle shows a yield percentage, but the contract stores PT and YT prices across multiple maturities, implying a full term structure. GMX shows a funding rate, but the contract stores open interest breakdowns by direction, position size distributions, and fee accrual histories.

Traditional TA ignores this data because it does not exist in traditional markets. A stock's order book is partially visible (Level 2 data) and fully visible to nobody except the exchange. A bond's yield curve is observable, but the positions creating that curve are not. Options Greeks are computed from market maker quotes that may or may not reflect actual inventory.

DeFi-native TA starts from the opposite premise: the full state is available. The question is not "what can we observe?" but "what is worth computing?" The Golem's compute budget is finite (Doc 3's signal metabolism enforces this). Every indicator in this document must justify its cost in predictive value per Gamma tick.

Ten indicator families follow. Each one exploits data that traditional finance cannot provide.

## 1. Concentrated liquidity shape analysis [SPEC]

Uniswap V3 and V4 replaced the uniform x*y=k curve with concentrated liquidity: LPs choose specific price ranges (tick ranges) for their capital. The result is a piecewise-linear liquidity distribution across tick space. This distribution is fully observable on-chain. TradFi order books, by contrast, show at best the top N levels of resting orders -- and market makers can pull those orders before execution.

The shape of the tick-space liquidity distribution encodes collective LP beliefs about future price movement. A distribution skewed above the current price means LPs expect upward movement (they are positioning to earn fees in higher ranges). A distribution with gaps means certain price ranges have no liquidity -- price can jump discontinuously through those ranges.

Five indicators extract signal from this shape.

### Tick asymmetry index

The ratio of total liquidity above the current tick to total liquidity below it. An asymmetry index of 2.0 means twice as much liquidity sits above the current price as below. This is a directional signal: LPs are positioning for upward moves.

The calculation is straightforward but the interpretation is not. High asymmetry can mean informed LPs positioning for a move, or it can mean stale positions from LPs who forgot to rebalance. The migration velocity indicator (below) helps distinguish these cases.

### Liquidity migration velocity

The rate at which liquidity is being repositioned across tick space, measured as the sum of absolute liquidity changes per tick per block. Fast migration (high velocity) means LPs are actively adjusting, which suggests informed positioning. Slow migration with high asymmetry suggests stale positions.

Migration velocity spikes before large price moves in backtesting on Uniswap V3 historical data. This makes sense: informed LPs (including MEV-aware LPs and protocol-managed positions) reposition ahead of anticipated volatility.

### Density gap detection

Gaps in the tick distribution -- ranges where initialized ticks exist on both sides but no liquidity sits in between. Gaps create potential price discontinuities: a swap large enough to exhaust liquidity at the current tick will skip through the gap to the next initialized tick, producing a price jump proportional to the gap width.

Gap detection scans the tick bitmap for sequences of uninitialized ticks flanked by initialized ones. The gap's significance depends on its width (in tick space) and its proximity to the current price. A gap 100 ticks below the current price is less immediately relevant than one 5 ticks away.

### Position concentration (Herfindahl index)

The Herfindahl-Hirschman Index across tick ranges, measuring how concentrated or dispersed liquidity is. High HHI (few positions controlling most liquidity) means fragile liquidity: if one large LP withdraws, the distribution changes dramatically. Low HHI (many small positions spread across ranges) means resilient liquidity.

HHI is computed from position counts and sizes per tick range. A sudden increase in HHI -- one large position dominating -- is a risk signal. The liquidity might be JIT (see below) or it might be a whale repositioning.

### JIT liquidity signature

Just-in-time liquidity is a MEV strategy: a searcher observes a pending swap in the mempool, adds concentrated liquidity in the exact tick range the swap will traverse (to earn fees), and removes it in the same block after the swap executes. JIT liquidity appears and disappears within a single block.

Detection compares consecutive tick snapshots. Liquidity that exists in block N but not in block N-1 or N+1 is JIT. The JIT score measures the fraction of total liquidity that exhibits this transient pattern. High JIT scores indicate active MEV extraction and suggest that the pool's fee economics are being captured by searchers rather than passive LPs.

```rust
use std::collections::{HashMap, VecDeque};

/// Liquidity data for a single tick range.
#[derive(Clone, Debug)]
pub struct TickLiquidity {
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: u128,
    pub position_count: u32,
}

/// A snapshot of the full tick distribution at a specific block.
#[derive(Clone, Debug)]
pub struct TickDistribution {
    pub ticks: Vec<TickLiquidity>,
    pub current_tick: i32,
    pub block_number: u64,
    pub total_liquidity: u128,
}

/// A detected gap in the tick distribution.
#[derive(Clone, Debug)]
pub struct TickGap {
    pub lower_bound: i32,
    pub upper_bound: i32,
    pub width_ticks: u32,
    /// Distance from current tick to nearest edge of the gap.
    pub proximity: u32,
}

/// Tracks liquidity migration across tick space.
struct MigrationTracker {
    previous_distribution: Option<HashMap<(i32, i32), u128>>,
    velocity_history: VecDeque<f64>,
    max_history: usize,
}

impl MigrationTracker {
    fn new(max_history: usize) -> Self {
        Self {
            previous_distribution: None,
            velocity_history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Compare current distribution to previous, return total absolute
    /// liquidity delta across all tick ranges.
    fn update(&mut self, distribution: &TickDistribution) -> f64 {
        let current: HashMap<(i32, i32), u128> = distribution
            .ticks
            .iter()
            .map(|t| ((t.tick_lower, t.tick_upper), t.liquidity))
            .collect();

        let velocity = match &self.previous_distribution {
            None => 0.0,
            Some(prev) => {
                let mut total_delta: u128 = 0;

                // Liquidity changes in existing ranges
                for (range, &curr_liq) in &current {
                    let prev_liq = prev.get(range).copied().unwrap_or(0);
                    total_delta += curr_liq.abs_diff(prev_liq);
                }

                // Ranges that disappeared
                for (range, &prev_liq) in prev {
                    if !current.contains_key(range) {
                        total_delta += prev_liq;
                    }
                }

                total_delta as f64
            }
        };

        self.previous_distribution = Some(current);

        if self.velocity_history.len() >= self.max_history {
            self.velocity_history.pop_front();
        }
        self.velocity_history.push_back(velocity);

        velocity
    }

    fn mean_velocity(&self) -> f64 {
        if self.velocity_history.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.velocity_history.iter().sum();
        sum / self.velocity_history.len() as f64
    }
}

/// Configuration for the concentrated liquidity analyzer.
#[derive(Clone, Debug)]
pub struct ClAnalyzerConfig {
    /// Number of historical snapshots to retain.
    pub snapshot_depth: usize,
    /// Migration velocity history length.
    pub velocity_history: usize,
    /// Minimum gap width (in ticks) to report.
    pub min_gap_width: u32,
    /// Maximum gap proximity (ticks from current) to report.
    pub max_gap_proximity: u32,
}

impl Default for ClAnalyzerConfig {
    fn default() -> Self {
        Self {
            snapshot_depth: 64,
            velocity_history: 128,
            min_gap_width: 10,
            max_gap_proximity: 500,
        }
    }
}

/// Analyzer for concentrated liquidity tick distributions.
///
/// Computes five indicators from on-chain Uniswap V3/V4 state:
/// asymmetry index, migration velocity, density gaps, position
/// concentration (HHI), and JIT liquidity score.
pub struct ConcentratedLiquidityAnalyzer {
    tick_snapshots: VecDeque<TickDistribution>,
    migration_tracker: MigrationTracker,
    config: ClAnalyzerConfig,
}

impl ConcentratedLiquidityAnalyzer {
    pub fn new(config: ClAnalyzerConfig) -> Self {
        let velocity_history = config.velocity_history;
        Self {
            tick_snapshots: VecDeque::with_capacity(config.snapshot_depth),
            migration_tracker: MigrationTracker::new(velocity_history),
            config,
        }
    }

    /// Ingest a new tick distribution snapshot.
    pub fn update(&mut self, distribution: TickDistribution) {
        self.migration_tracker.update(&distribution);

        if self.tick_snapshots.len() >= self.config.snapshot_depth {
            self.tick_snapshots.pop_front();
        }
        self.tick_snapshots.push_back(distribution);
    }

    /// Ratio of liquidity above current tick to liquidity below.
    /// Returns > 1.0 when more liquidity sits above (bullish LP positioning).
    /// Returns < 1.0 when more liquidity sits below (bearish LP positioning).
    /// Returns 1.0 when balanced or when no data is available.
    pub fn asymmetry_index(&self) -> f64 {
        let snapshot = match self.tick_snapshots.back() {
            Some(s) => s,
            None => return 1.0,
        };

        let mut above: u128 = 0;
        let mut below: u128 = 0;

        for tick in &snapshot.ticks {
            let midpoint = (tick.tick_lower + tick.tick_upper) / 2;
            if midpoint > snapshot.current_tick {
                above += tick.liquidity;
            } else if midpoint < snapshot.current_tick {
                below += tick.liquidity;
            } else {
                // Straddles current tick -- split proportionally
                let range = (tick.tick_upper - tick.tick_lower) as u128;
                if range > 0 {
                    let above_fraction =
                        (tick.tick_upper - snapshot.current_tick) as u128;
                    let below_fraction =
                        (snapshot.current_tick - tick.tick_lower) as u128;
                    above += tick.liquidity * above_fraction / range;
                    below += tick.liquidity * below_fraction / range;
                }
            }
        }

        if below == 0 {
            return if above == 0 { 1.0 } else { f64::INFINITY };
        }

        above as f64 / below as f64
    }

    /// Rate of liquidity repositioning across tick space.
    /// Higher values mean LPs are actively moving positions.
    /// Normalized by total liquidity to produce a dimensionless ratio.
    pub fn migration_velocity(&self) -> f64 {
        let snapshot = match self.tick_snapshots.back() {
            Some(s) => s,
            None => return 0.0,
        };

        if snapshot.total_liquidity == 0 {
            return 0.0;
        }

        self.migration_tracker.mean_velocity() / snapshot.total_liquidity as f64
    }

    /// Find gaps in the tick distribution near the current price.
    /// Gaps are contiguous regions of uninitialized ticks flanked by
    /// initialized ticks on both sides.
    pub fn density_gaps(&self) -> Vec<TickGap> {
        let snapshot = match self.tick_snapshots.back() {
            Some(s) => s,
            None => return Vec::new(),
        };

        if snapshot.ticks.is_empty() {
            return Vec::new();
        }

        // Sort ticks by lower bound
        let mut sorted_ticks: Vec<&TickLiquidity> = snapshot
            .ticks
            .iter()
            .filter(|t| t.liquidity > 0)
            .collect();
        sorted_ticks.sort_by_key(|t| t.tick_lower);

        let mut gaps = Vec::new();

        for window in sorted_ticks.windows(2) {
            let prev_upper = window[0].tick_upper;
            let next_lower = window[1].tick_lower;

            if next_lower > prev_upper {
                let width = (next_lower - prev_upper) as u32;
                if width < self.config.min_gap_width {
                    continue;
                }

                let proximity = if snapshot.current_tick < prev_upper {
                    (prev_upper - snapshot.current_tick) as u32
                } else if snapshot.current_tick > next_lower {
                    (snapshot.current_tick - next_lower) as u32
                } else {
                    0 // Current price is inside the gap
                };

                if proximity <= self.config.max_gap_proximity {
                    gaps.push(TickGap {
                        lower_bound: prev_upper,
                        upper_bound: next_lower,
                        width_ticks: width,
                        proximity,
                    });
                }
            }
        }

        gaps
    }

    /// Herfindahl-Hirschman Index of liquidity concentration across
    /// tick ranges. Range [0.0, 1.0] where 1.0 means all liquidity
    /// is in a single range (maximally concentrated / fragile).
    pub fn concentration_hhi(&self) -> f64 {
        let snapshot = match self.tick_snapshots.back() {
            Some(s) => s,
            None => return 0.0,
        };

        if snapshot.total_liquidity == 0 {
            return 0.0;
        }

        let total = snapshot.total_liquidity as f64;
        let mut hhi: f64 = 0.0;

        for tick in &snapshot.ticks {
            let share = tick.liquidity as f64 / total;
            hhi += share * share;
        }

        hhi
    }

    /// Fraction of current liquidity that appears to be JIT
    /// (present in this block but absent in the previous block).
    /// Range [0.0, 1.0]. High values indicate active MEV extraction.
    pub fn jit_liquidity_score(&self) -> f64 {
        if self.tick_snapshots.len() < 2 {
            return 0.0;
        }

        let current = &self.tick_snapshots[self.tick_snapshots.len() - 1];
        let previous = &self.tick_snapshots[self.tick_snapshots.len() - 2];

        let prev_map: HashMap<(i32, i32), u128> = previous
            .ticks
            .iter()
            .map(|t| ((t.tick_lower, t.tick_upper), t.liquidity))
            .collect();

        let mut jit_liquidity: u128 = 0;

        for tick in &current.ticks {
            let key = (tick.tick_lower, tick.tick_upper);
            let prev_liq = prev_map.get(&key).copied().unwrap_or(0);
            if prev_liq == 0 && tick.liquidity > 0 {
                // This liquidity did not exist in the previous block
                jit_liquidity += tick.liquidity;
            }
        }

        if current.total_liquidity == 0 {
            return 0.0;
        }

        jit_liquidity as f64 / current.total_liquidity as f64
    }

    /// Combined signal: is the asymmetry index changing in the same
    /// direction as migration velocity? Concordance suggests informed
    /// LP repositioning. Discordance suggests noise.
    pub fn informed_repositioning_score(&self) -> f64 {
        if self.tick_snapshots.len() < 3 {
            return 0.0;
        }

        let len = self.tick_snapshots.len();
        let current_asym = self.asymmetry_from_snapshot(&self.tick_snapshots[len - 1]);
        let prev_asym = self.asymmetry_from_snapshot(&self.tick_snapshots[len - 2]);
        let older_asym = self.asymmetry_from_snapshot(&self.tick_snapshots[len - 3]);

        let asym_direction = (current_asym - prev_asym).signum();
        let asym_acceleration = (current_asym - prev_asym) - (prev_asym - older_asym);

        let velocity = self.migration_velocity();

        // High velocity + consistent asymmetry direction + positive
        // acceleration = informed repositioning
        if velocity > 0.0 && asym_direction != 0.0 {
            (asym_acceleration.abs() * velocity).tanh()
        } else {
            0.0
        }
    }

    fn asymmetry_from_snapshot(&self, snapshot: &TickDistribution) -> f64 {
        let mut above: u128 = 0;
        let mut below: u128 = 0;

        for tick in &snapshot.ticks {
            let midpoint = (tick.tick_lower + tick.tick_upper) / 2;
            if midpoint > snapshot.current_tick {
                above += tick.liquidity;
            } else {
                below += tick.liquidity;
            }
        }

        if below == 0 {
            return 0.0;
        }
        above as f64 / below as f64
    }
}
```

### Subsystem integration

The concentrated liquidity analyzer feeds the Bardo runtime at multiple points:

**CorticalState.** The asymmetry index and JIT score map to new atomic signals on `TaCorticalExtension`. The Daimon reads these: high JIT scores correlate with adversarial MEV activity, which affects the Golem's emotional response to the pool. The Oracle reads asymmetry as a directional bias input.

**HDC encoding (Doc 1).** Each tick distribution snapshot is encoded as a 10,240-bit hypervector. The encoding uses spatial binding: tick range position is bound with liquidity magnitude, producing a distributed representation that supports Hamming-distance similarity queries. Two distributions with similar shapes but different absolute tick locations will still match if the relative structure is preserved.

**Signal metabolism (Doc 3).** The five indicators form a signal family. They compete for compute budget against indicators from other families. If concentrated liquidity signals stop predicting outcomes (because the market shifted to a regime where lending dynamics matter more), the metabolism decays their budget allocation.

**Causal graph (Doc 4).** Migration velocity is a candidate causal parent for price movement. The causal discovery engine tests this: does a spike in migration velocity cause the subsequent price move, or does a hidden confound (e.g., a large incoming swap visible in the mempool) cause both? Interventional testing via mirage-rs can distinguish these.

### Timescale mapping

| Indicator | Gamma tick | Theta tick | Delta tick |
|-----------|-----------|------------|------------|
| Asymmetry index | Incremental update from new tick data | Full recompute, trend analysis | Historical pattern extraction |
| Migration velocity | Streaming delta from previous block | Velocity regime classification | Velocity-outcome correlation analysis |
| Density gaps | Gap proximity check against current tick | Gap evolution tracking | Gap-based price discontinuity backtest |
| Concentration HHI | Incremental HHI from position updates | HHI trend, fragility assessment | Position turnover analysis |
| JIT score | Block-over-block comparison | JIT pattern classification | MEV profitability estimation |

At Gamma frequency, each indicator runs as a streaming update -- O(n) in the number of tick ranges, which is bounded by the pool's initialized tick count (typically hundreds, not thousands). Total Gamma cost for a single pool: under 100 microseconds.

## 2. Lending utilization dynamics [SPEC]

Lending protocols (Aave, Compound, Morpho) operate on a simple mechanism: depositors supply assets, borrowers borrow them, and an algorithmically determined interest rate balances supply and demand. The utilization ratio -- borrowed / supplied -- drives everything.

Traditional TA has no analog for this. There is no "utilization ratio" in stock markets. The closest concept is short interest (shares borrowed / shares outstanding), but short interest is reported with a two-week delay and does not drive a real-time interest rate curve.

In DeFi, utilization ratios update every block. The interest rate curve is a known function of utilization (Aave uses a two-slope model with a kink). Liquidation thresholds, health factors, and position sizes are all on-chain. This creates four indicators.

### Utilization oscillation frequency

The utilization ratio oscillates as borrowers enter and exit. The frequency of this oscillation carries information. Fast oscillation (many small borrows and repayments) indicates retail activity -- many small actors optimizing short-term rates. Slow oscillation (few large borrows held for extended periods) indicates institutional or strategic borrowing.

A Fast Fourier Transform on the utilization time series decomposes it into frequency components. The dominant frequency identifies the borrower population profile. Sudden shifts in the frequency spectrum -- the dominant frequency changing -- signal a change in the borrower population.

### Rate-change velocity

The first derivative of the interest rate with respect to time (blocks, in practice). Near the kink point of Aave's two-slope model, small utilization changes produce large rate changes. This creates cascading effects: a rate spike causes some borrowers to repay (reducing utilization), which causes the rate to drop (attracting new borrowers), which raises utilization again.

Rate-change velocity measures the amplitude of this feedback cycle. High velocity means the protocol is oscillating near its kink point -- an unstable equilibrium where small perturbations produce large effects.

### Liquidation proximity distribution

A histogram of all tracked positions by their distance from liquidation (health factor minus 1.0, or equivalently, percentage price decline needed to trigger liquidation). Clustering near the threshold is a systemic risk indicator: a moderate price decline would trigger many simultaneous liquidations, creating a cascade.

This indicator is impossible in traditional finance because position-level data is not public. In DeFi, every position's collateral value, debt value, and liquidation threshold are on-chain.

### Supply-borrow asymmetry

The differential between the rate of change of supply and the rate of change of borrowing. Supply-led utilization changes (new deposits outpacing new borrows) have different implications than borrow-led changes (new borrows outpacing new deposits). Supply-led increases suggest depositors seeking yield in this market (bullish on the underlying). Borrow-led increases suggest leveraged demand for the underlying.

```rust
use std::collections::VecDeque;

/// A single utilization reading at a specific block.
#[derive(Clone, Debug)]
pub struct UtilizationSnapshot {
    pub block_number: u64,
    pub utilization_ratio: f64,
    pub total_supply: f64,
    pub total_borrows: f64,
    pub supply_rate: f64,
    pub borrow_rate: f64,
}

/// A position's proximity to liquidation.
#[derive(Clone, Debug)]
pub struct PositionHealth {
    pub health_factor: f64,
    pub collateral_value: f64,
    pub debt_value: f64,
    /// Percentage price decline that would trigger liquidation.
    pub liquidation_distance: f64,
}

/// Tracks liquidation proximity across all monitored positions.
struct LiquidationProximityTracker {
    positions: Vec<PositionHealth>,
    /// Histogram bucket width (e.g., 0.05 = 5% increments).
    bucket_width: f64,
}

impl LiquidationProximityTracker {
    fn new(bucket_width: f64) -> Self {
        Self {
            positions: Vec::new(),
            bucket_width,
        }
    }

    fn update(&mut self, positions: Vec<PositionHealth>) {
        self.positions = positions;
    }

    /// Returns histogram: (bucket_midpoint, count) pairs.
    /// Bucket 0.025 means "positions that liquidate with a 0-5% price decline."
    fn distribution(&self) -> Vec<(f64, u32)> {
        let mut buckets: HashMap<u64, u32> = HashMap::new();

        for pos in &self.positions {
            let bucket_idx =
                (pos.liquidation_distance / self.bucket_width).floor() as u64;
            *buckets.entry(bucket_idx).or_insert(0) += 1;
        }

        let mut result: Vec<(f64, u32)> = buckets
            .into_iter()
            .map(|(idx, count)| {
                let midpoint =
                    (idx as f64 + 0.5) * self.bucket_width;
                (midpoint, count)
            })
            .collect();

        result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        result
    }

    /// Fraction of total collateral value within `threshold` percent
    /// of liquidation. Higher = more systemic risk.
    fn at_risk_fraction(&self, threshold: f64) -> f64 {
        let total_collateral: f64 =
            self.positions.iter().map(|p| p.collateral_value).sum();

        if total_collateral == 0.0 {
            return 0.0;
        }

        let at_risk: f64 = self
            .positions
            .iter()
            .filter(|p| p.liquidation_distance <= threshold)
            .map(|p| p.collateral_value)
            .sum();

        at_risk / total_collateral
    }
}

use std::collections::HashMap;
use std::f64::consts::PI;

/// Analyzer for lending protocol utilization dynamics.
///
/// Four indicators: utilization oscillation frequency (via FFT),
/// rate-change velocity, liquidation proximity distribution, and
/// supply-borrow asymmetry.
pub struct LendingDynamicsAnalyzer {
    utilization_history: VecDeque<UtilizationSnapshot>,
    liquidation_tracker: LiquidationProximityTracker,
    max_history: usize,
}

impl LendingDynamicsAnalyzer {
    pub fn new(max_history: usize, bucket_width: f64) -> Self {
        Self {
            utilization_history: VecDeque::with_capacity(max_history),
            liquidation_tracker: LiquidationProximityTracker::new(bucket_width),
            max_history,
        }
    }

    pub fn update_utilization(&mut self, snapshot: UtilizationSnapshot) {
        if self.utilization_history.len() >= self.max_history {
            self.utilization_history.pop_front();
        }
        self.utilization_history.push_back(snapshot);
    }

    pub fn update_positions(&mut self, positions: Vec<PositionHealth>) {
        self.liquidation_tracker.update(positions);
    }

    /// Compute the frequency spectrum of the utilization time series
    /// using a discrete Fourier transform.
    ///
    /// Returns (frequency, magnitude) pairs sorted by magnitude
    /// descending. Frequency is in units of cycles-per-block.
    pub fn utilization_frequency_spectrum(&self) -> Vec<(f64, f64)> {
        let n = self.utilization_history.len();
        if n < 8 {
            return Vec::new();
        }

        // Extract utilization values, subtract mean for zero-centering
        let values: Vec<f64> = self
            .utilization_history
            .iter()
            .map(|s| s.utilization_ratio)
            .collect();
        let mean: f64 = values.iter().sum::<f64>() / n as f64;
        let centered: Vec<f64> = values.iter().map(|v| v - mean).collect();

        // DFT (not FFT -- for the sizes we deal with, O(n^2) is fine
        // and avoids a dependency on an FFT crate for prototype code)
        let mut spectrum = Vec::with_capacity(n / 2);

        for k in 1..=(n / 2) {
            let mut real = 0.0;
            let mut imag = 0.0;

            for (t, &val) in centered.iter().enumerate() {
                let angle = 2.0 * PI * k as f64 * t as f64 / n as f64;
                real += val * angle.cos();
                imag -= val * angle.sin();
            }

            let magnitude = (real * real + imag * imag).sqrt() / n as f64;
            let frequency = k as f64 / n as f64; // cycles per block
            spectrum.push((frequency, magnitude));
        }

        spectrum.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        spectrum
    }

    /// Dominant oscillation frequency of utilization.
    /// Returns None if insufficient data.
    pub fn dominant_frequency(&self) -> Option<f64> {
        let spectrum = self.utilization_frequency_spectrum();
        spectrum.first().map(|(freq, _)| *freq)
    }

    /// First derivative of the interest rate (borrow rate) with
    /// respect to blocks. Units: rate change per block.
    pub fn rate_change_velocity(&self) -> f64 {
        if self.utilization_history.len() < 2 {
            return 0.0;
        }

        let len = self.utilization_history.len();
        let current = &self.utilization_history[len - 1];
        let previous = &self.utilization_history[len - 2];

        let block_delta =
            (current.block_number - previous.block_number) as f64;
        if block_delta == 0.0 {
            return 0.0;
        }

        (current.borrow_rate - previous.borrow_rate) / block_delta
    }

    /// Smoothed rate-change velocity over the last `window` snapshots.
    /// Reduces noise from individual block-to-block fluctuations.
    pub fn smoothed_rate_velocity(&self, window: usize) -> f64 {
        if self.utilization_history.len() < window + 1 {
            return 0.0;
        }

        let len = self.utilization_history.len();
        let start = len - window - 1;
        let mut total_velocity = 0.0;

        for i in (start + 1)..len {
            let current = &self.utilization_history[i];
            let previous = &self.utilization_history[i - 1];
            let block_delta =
                (current.block_number - previous.block_number) as f64;
            if block_delta > 0.0 {
                total_velocity +=
                    (current.borrow_rate - previous.borrow_rate) / block_delta;
            }
        }

        total_velocity / window as f64
    }

    /// Histogram of position distances from liquidation.
    pub fn liquidation_proximity_distribution(&self) -> Vec<(f64, u32)> {
        self.liquidation_tracker.distribution()
    }

    /// Fraction of collateral value within `threshold` of liquidation.
    pub fn liquidation_at_risk(&self, threshold: f64) -> f64 {
        self.liquidation_tracker.at_risk_fraction(threshold)
    }

    /// Differential between supply growth rate and borrow growth rate.
    /// Positive = supply growing faster (depositors entering).
    /// Negative = borrows growing faster (leveraged demand).
    pub fn supply_borrow_asymmetry(&self) -> f64 {
        if self.utilization_history.len() < 2 {
            return 0.0;
        }

        let len = self.utilization_history.len();
        let current = &self.utilization_history[len - 1];
        let previous = &self.utilization_history[len - 2];

        if previous.total_supply == 0.0 || previous.total_borrows == 0.0 {
            return 0.0;
        }

        let supply_growth =
            (current.total_supply - previous.total_supply) / previous.total_supply;
        let borrow_growth =
            (current.total_borrows - previous.total_borrows) / previous.total_borrows;

        supply_growth - borrow_growth
    }

    /// Exponentially weighted supply-borrow asymmetry over the
    /// full history. Recent observations weighted more heavily.
    pub fn ewma_supply_borrow_asymmetry(&self, alpha: f64) -> f64 {
        if self.utilization_history.len() < 2 {
            return 0.0;
        }

        let mut ewma = 0.0;

        for i in 1..self.utilization_history.len() {
            let current = &self.utilization_history[i];
            let previous = &self.utilization_history[i - 1];

            if previous.total_supply == 0.0 || previous.total_borrows == 0.0 {
                continue;
            }

            let supply_growth =
                (current.total_supply - previous.total_supply)
                    / previous.total_supply;
            let borrow_growth =
                (current.total_borrows - previous.total_borrows)
                    / previous.total_borrows;
            let asymmetry = supply_growth - borrow_growth;

            ewma = alpha * asymmetry + (1.0 - alpha) * ewma;
        }

        ewma
    }
}
```

### Why this is impossible in TradFi

Stock lending markets do exist, but they are bilateral OTC markets with no public utilization data. LIBOR (now SOFR) is a benchmark rate, not a live utilization-driven rate. A stock analyst cannot observe "what percentage of shares available for shorting are currently borrowed" in real time, cannot see individual short positions' margin distances, and cannot compute the frequency spectrum of a utilization time series that does not exist publicly.

DeFi lending protocols publish this state every block. The Golem reads it directly from contract storage.

## 3. Perpetual funding rate mean-reversion [SPEC]

Perpetual futures on decentralized exchanges (GMX, dYdX, Hyperliquid) charge periodic funding rates to keep the perpetual price anchored to the spot price. When longs outnumber shorts, longs pay shorts (positive funding). When shorts outnumber longs, shorts pay longs (negative funding). The rate adjusts until the position imbalance resolves.

This creates a bounded oscillator. Funding rates cannot diverge indefinitely because arbitrage forces them back: extreme positive funding attracts short sellers (who collect the funding payment), and extreme negative funding attracts long buyers. The mean-reversion property is structural, not statistical -- it comes from the protocol mechanism, not from historical patterns that might break.

Four indicators exploit this structure.

### Funding rate z-score

How far the current funding rate is from its historical mean, measured in standard deviations. Z-scores beyond +/-2 represent extreme positioning imbalances. The mean-reversion property implies these extremes are short-lived -- the question is how fast they revert.

### Mean-reversion half-life (Ornstein-Uhlenbeck estimation)

The funding rate follows an Ornstein-Uhlenbeck process: a mean-reverting stochastic process with three parameters. Theta (mean-reversion speed) determines how fast the rate returns to its long-term mean. Mu is the long-term mean itself. Sigma is the volatility of the process.

The half-life is ln(2) / theta. A half-life of 10 blocks means that a funding rate deviation from the mean is expected to halve every 10 blocks. Short half-lives mean fast reversion (the arbitrage is efficient). Long half-lives mean slow reversion (the imbalance is sticky, possibly because the directional conviction behind it is strong).

Parameter estimation uses maximum likelihood on the discretized OU process.

### Basis-funding decorrelation

The basis is the spread between the perpetual price and the spot price. Funding rate is the mechanism that keeps the basis near zero. Normally, basis and funding rate move together: positive basis leads to positive funding (longs pay shorts to reduce the basis).

When basis and funding rate diverge -- positive basis but zero or negative funding, or zero basis but extreme funding -- the arbitrage mechanism has broken. This is either a bug, a temporary dislocation, or a structural change in the protocol. In any case, it represents an opportunity or a risk that the decorrelation signal flags.

### OI-weighted funding flow

Total funding flow is open interest multiplied by the funding rate. A 0.01% funding rate on $10M of open interest moves $1,000 per funding period. The same rate on $1B of open interest moves $100,000. The absolute funding flow, not just the rate, determines how much capital the arbitrage opportunity represents and how much incentive exists for reversion.

```rust
use std::collections::VecDeque;

/// A single funding rate observation.
#[derive(Clone, Debug)]
pub struct FundingSnapshot {
    pub block_number: u64,
    pub funding_rate: f64,
    pub spot_price: f64,
    pub perp_price: f64,
    pub open_interest: f64,
}

/// Ornstein-Uhlenbeck process parameters.
/// The OU process is: dx = theta * (mu - x) * dt + sigma * dW
#[derive(Clone, Debug)]
pub struct OrnsteinUhlenbeckParams {
    /// Mean-reversion speed. Higher = faster reversion.
    pub theta: f64,
    /// Long-term mean.
    pub mu: f64,
    /// Volatility of the process.
    pub sigma: f64,
}

impl OrnsteinUhlenbeckParams {
    /// Half-life: time for a deviation from mu to halve.
    /// Units match the discretization of theta (blocks if theta
    /// is estimated per-block).
    pub fn half_life(&self) -> f64 {
        if self.theta <= 0.0 {
            return f64::INFINITY;
        }
        (2.0_f64).ln() / self.theta
    }

    /// Stationary variance of the process.
    pub fn stationary_variance(&self) -> f64 {
        if self.theta <= 0.0 {
            return f64::INFINITY;
        }
        self.sigma * self.sigma / (2.0 * self.theta)
    }
}

/// Analyzer for perpetual funding rate dynamics.
///
/// Four indicators: z-score, mean-reversion half-life (via OU
/// parameter estimation), basis-funding decorrelation, and
/// OI-weighted funding flow.
pub struct FundingRateAnalyzer {
    funding_history: VecDeque<FundingSnapshot>,
    ou_params: Option<OrnsteinUhlenbeckParams>,
    max_history: usize,
    /// Minimum observations before fitting OU parameters.
    min_observations_for_fit: usize,
}

impl FundingRateAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            funding_history: VecDeque::with_capacity(max_history),
            ou_params: None,
            max_history,
            min_observations_for_fit: 30,
        }
    }

    pub fn update(&mut self, snapshot: FundingSnapshot) {
        if self.funding_history.len() >= self.max_history {
            self.funding_history.pop_front();
        }
        self.funding_history.push_back(snapshot);
    }

    /// Z-score of the current funding rate relative to history.
    pub fn z_score(&self) -> f64 {
        if self.funding_history.len() < 2 {
            return 0.0;
        }

        let rates: Vec<f64> = self
            .funding_history
            .iter()
            .map(|s| s.funding_rate)
            .collect();

        let n = rates.len() as f64;
        let mean: f64 = rates.iter().sum::<f64>() / n;
        let variance: f64 =
            rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        let current = rates.last().unwrap();
        (current - mean) / std_dev
    }

    /// Estimate Ornstein-Uhlenbeck parameters via maximum likelihood
    /// on the discretized process.
    ///
    /// Uses the exact discretization:
    ///   x_{t+1} = mu + (x_t - mu) * exp(-theta * dt) + noise
    ///
    /// where noise ~ N(0, sigma^2 * (1 - exp(-2*theta*dt)) / (2*theta))
    pub fn fit_ou_parameters(&mut self) {
        let n = self.funding_history.len();
        if n < self.min_observations_for_fit {
            return;
        }

        let rates: Vec<f64> = self
            .funding_history
            .iter()
            .map(|s| s.funding_rate)
            .collect();

        // For uniform dt = 1 block, the discretized OU reduces to
        // an AR(1) process: x_{t+1} = a + b * x_t + noise
        // where b = exp(-theta), a = mu * (1 - b)
        let n_f = (n - 1) as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_xy = 0.0;

        for i in 0..(n - 1) {
            let x = rates[i];
            let y = rates[i + 1];
            sum_x += x;
            sum_y += y;
            sum_xx += x * x;
            sum_xy += x * y;
        }

        let denom = n_f * sum_xx - sum_x * sum_x;
        if denom.abs() < 1e-15 {
            return;
        }

        let b = (n_f * sum_xy - sum_x * sum_y) / denom;
        let a = (sum_y - b * sum_x) / n_f;

        // Recover OU parameters from AR(1) coefficients
        if b <= 0.0 || b >= 1.0 {
            // Not mean-reverting or explosive -- OU model does not apply
            return;
        }

        let theta = -(b.ln()); // dt = 1
        let mu = a / (1.0 - b);

        // Estimate sigma from residuals
        let mut residual_sum_sq = 0.0;
        for i in 0..(n - 1) {
            let predicted = a + b * rates[i];
            let residual = rates[i + 1] - predicted;
            residual_sum_sq += residual * residual;
        }
        let residual_variance = residual_sum_sq / n_f;

        // sigma^2 = 2 * theta * residual_variance / (1 - exp(-2*theta))
        let exp_neg_2theta = (-2.0 * theta).exp();
        let sigma = (2.0 * theta * residual_variance / (1.0 - exp_neg_2theta)).sqrt();

        self.ou_params = Some(OrnsteinUhlenbeckParams { theta, mu, sigma });
    }

    /// Mean-reversion half-life in blocks.
    /// Returns None if OU parameters have not been estimated.
    pub fn mean_reversion_halflife(&self) -> Option<f64> {
        self.ou_params.as_ref().map(|p| p.half_life())
    }

    /// Correlation between basis (perp - spot) and funding rate
    /// over the observation window. Low correlation indicates
    /// a broken arbitrage mechanism.
    pub fn basis_decorrelation(&self) -> f64 {
        if self.funding_history.len() < 10 {
            return 0.0;
        }

        let basis: Vec<f64> = self
            .funding_history
            .iter()
            .map(|s| s.perp_price - s.spot_price)
            .collect();
        let funding: Vec<f64> = self
            .funding_history
            .iter()
            .map(|s| s.funding_rate)
            .collect();

        pearson_correlation(&basis, &funding)
    }

    /// Total funding flow: open interest * funding rate.
    /// Represents the absolute dollar amount flowing between
    /// longs and shorts per funding period.
    pub fn oi_weighted_funding_flow(&self) -> f64 {
        match self.funding_history.back() {
            Some(s) => s.open_interest * s.funding_rate,
            None => 0.0,
        }
    }

    /// Is the funding rate currently in an extreme state with
    /// a short half-life? If yes, mean-reversion is imminent.
    /// Returns a confidence score in [0.0, 1.0].
    pub fn reversion_imminence(&self) -> f64 {
        let z = self.z_score().abs();
        let half_life = match self.mean_reversion_halflife() {
            Some(hl) => hl,
            None => return 0.0,
        };

        // High z-score + short half-life = high reversion confidence
        // Use sigmoid: 1 / (1 + exp(-k * (z - z0))) * decay(half_life)
        let z_signal = 1.0 / (1.0 + (-2.0 * (z - 1.5)).exp());
        let hl_signal = (-half_life / 50.0).exp(); // decays with longer half-lives

        z_signal * hl_signal
    }
}

/// Pearson correlation coefficient between two equal-length slices.
fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom == 0.0 {
        return 0.0;
    }

    cov / denom
}
```

### Cybernetic feedback

The funding rate analyzer participates in the Golem's feedback loop at a specific point: the Oracle's prediction engine. When the funding rate z-score exceeds +/-2, the Oracle can predict mean-reversion with a confidence derived from the OU parameters. If the prediction hits (the rate does revert), the Oracle's accuracy improves, the epistemic clock stabilizes, and the Golem lives longer. If the prediction misses (the rate stays extreme, indicating the OU model's assumptions have broken), the prediction residual feeds back into Doc 4's causal graph to discover what changed.

The key insight: the structural mean-reversion property gives the Golem a prediction with a known mechanism behind it. This is different from correlative TA patterns that might stop working. The funding rate must revert eventually because the protocol enforces it. The question is when, and the OU half-life estimates that.

## 4. On-chain options Greeks surfaces [SPEC]

Panoptic (built on Uniswap V4) and other on-chain options protocols make the full options surface observable. In traditional options markets, the Greeks surface is reconstructed from dealer quotes -- and those quotes may not reflect actual inventory or risk exposure. On-chain, the surface is computed from actual positions with known parameters.

Four indicators derive from this surface.

### IV surface curvature

The implied volatility surface maps strike prices and expiries to implied volatility values. The second partial derivatives of this surface (curvature in the strike direction, curvature in the expiry direction, and the cross partial) carry information about regime expectations.

High curvature in the strike direction means the volatility smile is steep -- the market expects large price moves. High curvature in the expiry direction means the term structure is steep -- near-term vol differs sharply from long-term vol. Changes in curvature over time predict regime transitions.

### Greeks skew dynamics

The put-call skew (difference in IV between out-of-the-money puts and out-of-the-money calls at the same delta) measures directional fear. Rising put skew means increasing demand for downside protection. The rate of change of skew (its first derivative) indicates how fast sentiment is shifting.

### Gamma exposure aggregation

Net gamma across all on-chain options positions determines how dealer hedging (or LP hedging, in DeFi) affects price dynamics. Positive net gamma means hedging dampens volatility: when price rises, delta-hedging requires selling (pushing price back down), and vice versa. Negative net gamma amplifies volatility: hedging reinforces the price move.

The transition from positive to negative net gamma is a regime change signal. It often coincides with increased realized volatility as the dampening effect vanishes.

### Vanna-volga decomposition

Decompose option pricing residuals into components driven by vanna (sensitivity of delta to vol) and volga (sensitivity of vega to vol). The relative magnitude of vanna and volga contributions indicates which risk factor dominates current pricing: spot-vol correlation (vanna) or vol-of-vol (volga).

```rust
use std::collections::VecDeque;

/// A single entry in the Greeks surface: one strike-expiry pair.
#[derive(Clone, Debug)]
pub struct GreeksEntry {
    pub strike: f64,
    pub expiry_seconds: u64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub iv: f64,
    /// Position size at this strike-expiry (signed: positive = long, negative = short).
    pub net_position: f64,
}

/// A snapshot of the full Greeks surface at a given block.
#[derive(Clone, Debug)]
pub struct GreeksSurface {
    pub entries: Vec<GreeksEntry>,
    pub underlying_price: f64,
    pub block_number: u64,
}

/// A snapshot of the implied volatility surface.
#[derive(Clone, Debug)]
pub struct IvSurface {
    /// (strike, expiry_seconds, iv) triples.
    pub points: Vec<(f64, u64, f64)>,
    pub underlying_price: f64,
    pub block_number: u64,
}

/// Analyzer for on-chain options Greeks surfaces.
pub struct OnChainGreeksAnalyzer {
    greeks_snapshots: VecDeque<GreeksSurface>,
    iv_surface_history: VecDeque<IvSurface>,
    max_history: usize,
}

impl OnChainGreeksAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            greeks_snapshots: VecDeque::with_capacity(max_history),
            iv_surface_history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn update(&mut self, greeks: GreeksSurface, iv: IvSurface) {
        if self.greeks_snapshots.len() >= self.max_history {
            self.greeks_snapshots.pop_front();
            self.iv_surface_history.pop_front();
        }
        self.greeks_snapshots.push_back(greeks);
        self.iv_surface_history.push_back(iv);
    }

    /// Second derivative of IV with respect to strike (smile curvature)
    /// at the current underlying price. Computed via finite differences
    /// on the nearest three strikes.
    pub fn iv_surface_curvature(&self) -> f64 {
        let surface = match self.iv_surface_history.back() {
            Some(s) => s,
            None => return 0.0,
        };

        if surface.points.len() < 3 {
            return 0.0;
        }

        // Group by the shortest expiry for a clean strike-smile slice
        let min_expiry = surface
            .points
            .iter()
            .map(|(_, e, _)| *e)
            .min()
            .unwrap_or(0);

        let mut smile: Vec<(f64, f64)> = surface
            .points
            .iter()
            .filter(|(_, e, _)| *e == min_expiry)
            .map(|(s, _, iv)| (*s, *iv))
            .collect();

        smile.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        if smile.len() < 3 {
            return 0.0;
        }

        // Find the three strikes closest to ATM
        let atm = surface.underlying_price;
        smile.sort_by(|a, b| {
            (a.0 - atm)
                .abs()
                .partial_cmp(&(b.0 - atm).abs())
                .unwrap()
        });

        let nearest_three = &smile[..3.min(smile.len())];
        let mut sorted: Vec<(f64, f64)> = nearest_three.to_vec();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        if sorted.len() < 3 {
            return 0.0;
        }

        // Second derivative via central finite difference
        let (x0, y0) = sorted[0];
        let (x1, y1) = sorted[1];
        let (x2, y2) = sorted[2];

        let h1 = x1 - x0;
        let h2 = x2 - x1;

        if h1 == 0.0 || h2 == 0.0 {
            return 0.0;
        }

        2.0 * (y2 / (h2 * (h1 + h2)) - y1 / (h1 * h2) + y0 / (h1 * (h1 + h2)))
    }

    /// Rate of change of put-call skew.
    /// Skew = IV(25-delta put) - IV(25-delta call).
    /// Returns the first difference over the last two snapshots.
    pub fn skew_rate_of_change(&self) -> f64 {
        if self.greeks_snapshots.len() < 2 {
            return 0.0;
        }

        let current_skew = self.compute_skew(
            &self.greeks_snapshots[self.greeks_snapshots.len() - 1],
        );
        let previous_skew = self.compute_skew(
            &self.greeks_snapshots[self.greeks_snapshots.len() - 2],
        );

        current_skew - previous_skew
    }

    /// Net gamma exposure across all positions on the surface.
    /// Positive = dealer hedging dampens vol. Negative = amplifies vol.
    pub fn net_gamma_exposure(&self) -> f64 {
        let surface = match self.greeks_snapshots.back() {
            Some(s) => s,
            None => return 0.0,
        };

        surface
            .entries
            .iter()
            .map(|e| e.gamma * e.net_position)
            .sum()
    }

    /// Is net gamma flipping sign? Returns the number of blocks
    /// since the last sign change. Short duration = unstable regime.
    pub fn gamma_flip_recency(&self) -> Option<u64> {
        if self.greeks_snapshots.len() < 2 {
            return None;
        }

        let current_gamma = self.net_gamma_exposure();
        let current_sign = current_gamma.signum();

        for i in (0..self.greeks_snapshots.len() - 1).rev() {
            let past_gamma: f64 = self.greeks_snapshots[i]
                .entries
                .iter()
                .map(|e| e.gamma * e.net_position)
                .sum();
            let past_sign = past_gamma.signum();

            if past_sign != current_sign && past_sign != 0.0 {
                let current_block = self.greeks_snapshots.back().unwrap().block_number;
                return Some(current_block - self.greeks_snapshots[i].block_number);
            }
        }

        None
    }

    fn compute_skew(&self, surface: &GreeksSurface) -> f64 {
        // Find entries closest to 25-delta put and 25-delta call
        let target_put_delta = -0.25;
        let target_call_delta = 0.25;

        let mut best_put: Option<&GreeksEntry> = None;
        let mut best_put_dist = f64::INFINITY;
        let mut best_call: Option<&GreeksEntry> = None;
        let mut best_call_dist = f64::INFINITY;

        for entry in &surface.entries {
            // Puts have negative delta
            if entry.delta < 0.0 {
                let dist = (entry.delta - target_put_delta).abs();
                if dist < best_put_dist {
                    best_put_dist = dist;
                    best_put = Some(entry);
                }
            }
            // Calls have positive delta
            if entry.delta > 0.0 {
                let dist = (entry.delta - target_call_delta).abs();
                if dist < best_call_dist {
                    best_call_dist = dist;
                    best_call = Some(entry);
                }
            }
        }

        match (best_put, best_call) {
            (Some(p), Some(c)) => p.iv - c.iv,
            _ => 0.0,
        }
    }
}
```

### What TradFi cannot see

A traditional options trader can compute Greeks from quoted prices. But those quotes come from market makers who may shade them based on their own inventory and risk limits. The quoted surface might not reflect actual aggregate gamma exposure. On-chain, every position's exact parameters are visible. The net gamma computation is exact, not estimated from quotes that might be stale or strategic.

Panoptic positions are Uniswap V3 LP positions used as options. Their Greeks are derived from the AMM mechanics, not from a separate options pricing model. This means the options surface is directly connected to the liquidity surface (Section 1) -- a connection that does not exist in traditional finance where the options market and the cash equity market are separate venues.

## 5. Yield term structure [SPEC]

Pendle splits yield-bearing assets into Principal Tokens (PT) and Yield Tokens (YT). PT represents the principal, redeemable at maturity for the underlying. YT represents the yield stream until maturity. The prices of PT and YT across multiple maturities imply a yield curve -- the market's collective expectation of future yields.

Traditional fixed income has yield curves too, but they are constructed from government bond prices by central banks and data providers. The individual positions creating the curve are invisible. In DeFi, every Pendle position -- every PT and YT holder, every LP in the Pendle AMM -- is on-chain.

Four indicators extract signal from the DeFi-native term structure.

### Yield curve slope

The difference between implied rates at the longest and shortest available maturities. A positive slope (long rates above short rates) is the normal state: lenders demand compensation for locking capital longer. A flat or inverted slope signals that the market expects yields to decline -- possibly because confidence in the underlying yield source is falling.

### Curve steepening/flattening velocity

The rate of change of the slope. Fast steepening means the market is rapidly pricing in higher long-term yields relative to short-term. Fast flattening means the opposite. Velocity, not just level, predicts regime transitions.

### Term premium dynamics

The excess yield of holding a long-maturity PT versus rolling short-maturity PTs. In a world of constant expected yields, the two strategies should produce the same return. The term premium measures the market's demand for compensation against yield uncertainty at longer horizons. Rising term premium = rising uncertainty.

### Cross-asset term structure divergence

When Pendle yield curves for different underlyings (e.g., stETH yield vs. GLP yield vs. DAI savings rate) diverge, it signals sector-specific stress. Yield curves for staking derivatives steepening while lending yield curves flatten indicates capital rotating from lending to staking.

```rust
use std::collections::{HashMap, VecDeque};

/// A single point on the yield curve.
#[derive(Clone, Debug)]
pub struct YieldPoint {
    /// Time to maturity in seconds.
    pub maturity_seconds: u64,
    /// Annualized implied yield from PT price.
    pub implied_rate: f64,
    /// PT price as fraction of underlying (e.g., 0.95 = 5% discount).
    pub pt_price: f64,
    /// YT price as fraction of underlying.
    pub yt_price: f64,
}

/// A complete yield curve snapshot.
#[derive(Clone, Debug)]
pub struct YieldCurveSnapshot {
    pub points: Vec<YieldPoint>,
    pub block_number: u64,
    pub underlying_asset: String,
}

/// Analyzer for DeFi yield term structures (Pendle PT/YT implied curves).
pub struct YieldTermStructureAnalyzer {
    /// Per-asset yield curve histories.
    term_structures: HashMap<String, VecDeque<YieldCurveSnapshot>>,
    max_history: usize,
}

impl YieldTermStructureAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            term_structures: HashMap::new(),
            max_history,
        }
    }

    pub fn update(&mut self, snapshot: YieldCurveSnapshot) {
        let history = self
            .term_structures
            .entry(snapshot.underlying_asset.clone())
            .or_insert_with(|| VecDeque::with_capacity(self.max_history));

        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back(snapshot);
    }

    /// Yield curve slope for a given asset: long rate minus short rate.
    /// Positive = normal curve. Negative = inverted.
    pub fn slope(&self, asset: &str) -> f64 {
        let history = match self.term_structures.get(asset) {
            Some(h) if !h.is_empty() => h,
            _ => return 0.0,
        };

        let curve = &history[history.len() - 1];
        if curve.points.len() < 2 {
            return 0.0;
        }

        let mut sorted = curve.points.clone();
        sorted.sort_by_key(|p| p.maturity_seconds);

        let short_rate = sorted.first().unwrap().implied_rate;
        let long_rate = sorted.last().unwrap().implied_rate;

        long_rate - short_rate
    }

    /// Rate of change of slope over the last two snapshots.
    /// Positive = steepening. Negative = flattening.
    pub fn slope_velocity(&self, asset: &str) -> f64 {
        let history = match self.term_structures.get(asset) {
            Some(h) if h.len() >= 2 => h,
            _ => return 0.0,
        };

        let current_slope = self.slope_from_snapshot(&history[history.len() - 1]);
        let previous_slope = self.slope_from_snapshot(&history[history.len() - 2]);

        current_slope - previous_slope
    }

    /// Term premium: excess return from holding the longest-maturity
    /// PT versus the implied return of rolling the shortest-maturity PT.
    /// Positive = market demands compensation for maturity risk.
    pub fn term_premium(&self, asset: &str) -> f64 {
        let history = match self.term_structures.get(asset) {
            Some(h) if !h.is_empty() => h,
            _ => return 0.0,
        };

        let curve = &history[history.len() - 1];
        if curve.points.len() < 2 {
            return 0.0;
        }

        let mut sorted = curve.points.clone();
        sorted.sort_by_key(|p| p.maturity_seconds);

        let short = sorted.first().unwrap();
        let long = sorted.last().unwrap();

        // Simple term premium: long rate minus what you'd get by
        // rolling the short rate for the same duration.
        // This assumes constant short rates (the expectations hypothesis).
        // The premium is what the market charges above that.
        long.implied_rate - short.implied_rate
    }

    /// Cross-asset divergence: how different are the yield curve slopes
    /// across all tracked assets? High divergence means sector-specific
    /// stress. Low divergence means broad yield regime.
    pub fn cross_asset_divergence(&self) -> f64 {
        let slopes: Vec<f64> = self
            .term_structures
            .keys()
            .map(|asset| self.slope(asset))
            .collect();

        if slopes.len() < 2 {
            return 0.0;
        }

        let mean = slopes.iter().sum::<f64>() / slopes.len() as f64;
        let variance =
            slopes.iter().map(|s| (s - mean).powi(2)).sum::<f64>()
                / (slopes.len() - 1) as f64;

        variance.sqrt()
    }

    /// Which assets' yield curves diverge most from the mean slope?
    /// Returns (asset, deviation) pairs sorted by absolute deviation.
    pub fn divergence_leaders(&self) -> Vec<(String, f64)> {
        let slopes: Vec<(String, f64)> = self
            .term_structures
            .keys()
            .map(|asset| (asset.clone(), self.slope(asset)))
            .collect();

        if slopes.len() < 2 {
            return Vec::new();
        }

        let mean = slopes.iter().map(|(_, s)| s).sum::<f64>() / slopes.len() as f64;

        let mut deviations: Vec<(String, f64)> = slopes
            .into_iter()
            .map(|(asset, slope)| (asset, slope - mean))
            .collect();

        deviations.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        deviations
    }

    fn slope_from_snapshot(&self, curve: &YieldCurveSnapshot) -> f64 {
        if curve.points.len() < 2 {
            return 0.0;
        }

        let mut sorted = curve.points.clone();
        sorted.sort_by_key(|p| p.maturity_seconds);

        sorted.last().unwrap().implied_rate - sorted.first().unwrap().implied_rate
    }
}
```

### Why term structures matter for the Golem

The yield term structure is a forward-looking signal. It encodes market participants' expectations about future yields, which in turn depend on expected DeFi activity, token emission rates, and risk assessments. A Golem that tracks yield curve changes can anticipate capital flows before they happen: if the stETH yield curve inverts (short rates exceed long rates), capital is likely to flow out of long-duration stETH positions, creating selling pressure.

The causal graph (Doc 4) tests whether yield curve changes cause capital flows or merely correlate with them. Interventional simulations via mirage-rs can distinguish: fork the state, artificially shift the yield curve, and observe whether capital flows follow.

## 6. Vault share price mechanics [SPEC]

ERC-4626 standardized vault interfaces, making every compatible vault's share price, total assets, and deposit/withdrawal flows observable through a common API. The share price (total assets / total shares) is the vault's summary statistic, but the dynamics around that price carry more information than the price level itself.

Four indicators.

### Share price acceleration

The second derivative of share price with respect to blocks. Positive acceleration means yield is increasing (the vault is earning more per unit time). Negative acceleration means yield is decaying. Zero acceleration is steady state. Acceleration changes precede yield changes in the first derivative, giving the Golem an early signal.

### Redemption queue pressure

The ratio of pending withdrawals to total vault assets. Most vaults process withdrawals with some delay (especially if the underlying assets are illiquid or locked in strategies). Rising redemption pressure signals declining confidence. If pressure exceeds the vault's liquid reserves, the vault must unwind strategy positions to meet redemptions -- a forced selling event.

### Strategy rotation detection

Many vaults periodically rotate their underlying strategy allocations. A vault might shift from providing Uniswap liquidity to lending on Aave to farming on Convex, depending on which offers the best risk-adjusted yield. Detecting these rotations (via changes in the vault's underlying asset composition, observable through events or state queries) provides a meta-signal: the vault operator -- who presumably has sophisticated yield analysis -- believes one strategy now dominates another.

### Yield harvesting periodicity

The time interval between harvest operations (when the vault collects earned yield and compounds it into the strategy). Regular harvests at fixed intervals suggest automated, healthy operation. Irregular harvests suggest manual intervention. Missed harvests suggest operator issues. A sudden decrease in harvest frequency might mean the operator is extracting value less often (lower yields to harvest) or that the automation has broken.

```rust
use std::collections::VecDeque;

/// A vault state snapshot at a specific block.
#[derive(Clone, Debug)]
pub struct VaultSnapshot {
    pub block_number: u64,
    pub share_price: f64,
    pub total_assets: f64,
    pub total_shares: f64,
    pub pending_withdrawals: f64,
    /// True if a harvest event was observed in this block.
    pub harvest_observed: bool,
}

/// Analyzer for ERC-4626 vault share price mechanics.
pub struct VaultMechanicsAnalyzer {
    share_price_history: VecDeque<VaultSnapshot>,
    harvest_blocks: VecDeque<u64>,
    max_history: usize,
}

impl VaultMechanicsAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            share_price_history: VecDeque::with_capacity(max_history),
            harvest_blocks: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn update(&mut self, snapshot: VaultSnapshot) {
        if snapshot.harvest_observed {
            if self.harvest_blocks.len() >= self.max_history {
                self.harvest_blocks.pop_front();
            }
            self.harvest_blocks.push_back(snapshot.block_number);
        }

        if self.share_price_history.len() >= self.max_history {
            self.share_price_history.pop_front();
        }
        self.share_price_history.push_back(snapshot);
    }

    /// Second derivative of share price (acceleration of yield).
    /// Positive = yield accelerating. Negative = yield decelerating.
    pub fn share_price_acceleration(&self) -> f64 {
        if self.share_price_history.len() < 3 {
            return 0.0;
        }

        let len = self.share_price_history.len();
        let p2 = self.share_price_history[len - 1].share_price;
        let p1 = self.share_price_history[len - 2].share_price;
        let p0 = self.share_price_history[len - 3].share_price;

        let b2 = self.share_price_history[len - 1].block_number as f64;
        let b1 = self.share_price_history[len - 2].block_number as f64;
        let b0 = self.share_price_history[len - 3].block_number as f64;

        let dt1 = b2 - b1;
        let dt0 = b1 - b0;

        if dt1 == 0.0 || dt0 == 0.0 {
            return 0.0;
        }

        let v1 = (p2 - p1) / dt1; // first derivative (recent)
        let v0 = (p1 - p0) / dt0; // first derivative (previous)

        let dt_mid = (dt1 + dt0) / 2.0;
        (v1 - v0) / dt_mid // second derivative
    }

    /// Ratio of pending withdrawals to total assets.
    /// Range [0.0, 1.0+]. Values approaching 1.0 signal bank run risk.
    pub fn redemption_queue_pressure(&self) -> f64 {
        match self.share_price_history.back() {
            Some(s) if s.total_assets > 0.0 => {
                s.pending_withdrawals / s.total_assets
            }
            _ => 0.0,
        }
    }

    /// Is redemption pressure increasing over the last N snapshots?
    /// Returns the linear trend coefficient (positive = increasing pressure).
    pub fn redemption_pressure_trend(&self, window: usize) -> f64 {
        let len = self.share_price_history.len();
        if len < window {
            return 0.0;
        }

        let start = len - window;
        let pressures: Vec<f64> = self.share_price_history
            .range(start..)
            .map(|s| {
                if s.total_assets > 0.0 {
                    s.pending_withdrawals / s.total_assets
                } else {
                    0.0
                }
            })
            .collect();

        linear_trend(&pressures)
    }

    /// Mean interval between harvests (in blocks).
    /// Returns None if fewer than 2 harvests observed.
    pub fn mean_harvest_interval(&self) -> Option<f64> {
        if self.harvest_blocks.len() < 2 {
            return None;
        }

        let mut intervals = Vec::new();
        for i in 1..self.harvest_blocks.len() {
            intervals.push(
                (self.harvest_blocks[i] - self.harvest_blocks[i - 1]) as f64,
            );
        }

        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        Some(mean)
    }

    /// Coefficient of variation of harvest intervals.
    /// Low CV = regular harvests (automated). High CV = irregular (manual).
    pub fn harvest_regularity(&self) -> f64 {
        if self.harvest_blocks.len() < 3 {
            return 0.0;
        }

        let mut intervals = Vec::new();
        for i in 1..self.harvest_blocks.len() {
            intervals.push(
                (self.harvest_blocks[i] - self.harvest_blocks[i - 1]) as f64,
            );
        }

        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        if mean == 0.0 {
            return 0.0;
        }

        let variance = intervals
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (intervals.len() - 1) as f64;

        variance.sqrt() / mean // coefficient of variation
    }

    /// Blocks since last harvest. Combined with mean_harvest_interval,
    /// indicates whether a harvest is overdue.
    pub fn blocks_since_last_harvest(&self) -> Option<u64> {
        let last_harvest = self.harvest_blocks.back()?;
        let current_block = self.share_price_history.back()?.block_number;
        Some(current_block - last_harvest)
    }
}

/// Simple linear trend via least-squares slope.
fn linear_trend(values: &[f64]) -> f64 {
    let n = values.len() as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xx = 0.0;
    let mut sum_xy = 0.0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        sum_x += x;
        sum_y += y;
        sum_xx += x * x;
        sum_xy += x * y;
    }

    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-15 {
        return 0.0;
    }

    (n * sum_xy - sum_x * sum_y) / denom
}
```

### Integration with the mortality engine

Vault redemption pressure has a direct path to the Golem's mortality assessment. If the Golem holds positions in or through a vault, rising redemption pressure threatens the Golem's capital. The mortality engine reads the redemption pressure signal from `TaCorticalExtension` and factors it into economic vitality. A vault showing pressure above 0.5 (half the assets are being redeemed) might trigger Conservation behavioral phase, causing the Golem to reduce exposure.

Share price acceleration feeds into the Oracle's yield prediction models. The Oracle predicts future share prices; the acceleration term provides curvature information that improves the prediction beyond linear extrapolation.

## 7. Restaking delegation flows [SPEC]

EigenLayer introduced restaking: validators stake ETH to secure Ethereum, then restake that same ETH to secure additional protocols (Actively Validated Services, or AVSs). The delegation patterns -- who stakes with which operator, which operator secures which AVS, and how these relationships change -- create a new observation space.

Three indicators.

### Delegation concentration shift

The Herfindahl index of operator stakes measures concentration. A single operator controlling 40% of all restaked ETH creates a very different security profile than 100 operators each controlling 1%. The rate of change of HHI over time signals whether the system is centralizing or decentralizing.

Centralization is a risk signal at the protocol level (fewer operators = more correlated failure risk) and an opportunity signal at the individual level (concentrated operators attract attention, and attention creates predictable behavior patterns).

### AVS security budget utilization

Each AVS has an economic security requirement (the minimum amount of staked capital needed to make attacks unprofitable) and an actual economic security amount (the total value staked to secure it). The ratio is the security budget utilization.

Over-secured AVSs (utilization < 0.5) might attract restakers looking for easy rewards. Under-secured AVSs (utilization > 0.9) are approaching their security margin, and further growth might be constrained until more capital is allocated. The rate of change indicates whether the AVS is gaining or losing security.

### Operator performance divergence

When operators' realized returns start diverging from each other (high variance across the operator population), something has changed. Possible causes: MEV extraction differences (some operators capture more MEV), slashing events, AVS reward changes, or operator infrastructure issues.

The standard deviation of per-operator returns, tracked over time, acts as an early warning. Rising divergence precedes problems that eventually become visible in aggregate metrics.

```rust
use std::collections::{HashMap, VecDeque};

/// A snapshot of operator delegation state.
#[derive(Clone, Debug)]
pub struct OperatorSnapshot {
    pub operator_id: String,
    pub total_delegated: f64,
    pub avs_assignments: Vec<String>,
    pub realized_return: f64,
}

/// AVS security state.
#[derive(Clone, Debug)]
pub struct AvsSecuritySnapshot {
    pub avs_id: String,
    pub required_security: f64,
    pub actual_security: f64,
}

/// A full restaking state snapshot.
#[derive(Clone, Debug)]
pub struct RestakingSnapshot {
    pub block_number: u64,
    pub operators: Vec<OperatorSnapshot>,
    pub avs_states: Vec<AvsSecuritySnapshot>,
}

/// Analyzer for restaking delegation flow patterns.
pub struct RestakingFlowAnalyzer {
    snapshots: VecDeque<RestakingSnapshot>,
    max_history: usize,
}

impl RestakingFlowAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn update(&mut self, snapshot: RestakingSnapshot) {
        if self.snapshots.len() >= self.max_history {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    /// Herfindahl index of operator delegation stakes.
    /// Range [0.0, 1.0] where 1.0 = maximally concentrated.
    pub fn delegation_hhi(&self) -> f64 {
        let snapshot = match self.snapshots.back() {
            Some(s) => s,
            None => return 0.0,
        };

        let total: f64 = snapshot.operators.iter().map(|o| o.total_delegated).sum();
        if total == 0.0 {
            return 0.0;
        }

        snapshot
            .operators
            .iter()
            .map(|o| {
                let share = o.total_delegated / total;
                share * share
            })
            .sum()
    }

    /// Rate of change of delegation HHI.
    /// Positive = centralizing. Negative = decentralizing.
    pub fn delegation_hhi_velocity(&self) -> f64 {
        if self.snapshots.len() < 2 {
            return 0.0;
        }

        let current_hhi = self.hhi_from_snapshot(
            &self.snapshots[self.snapshots.len() - 1],
        );
        let previous_hhi = self.hhi_from_snapshot(
            &self.snapshots[self.snapshots.len() - 2],
        );

        current_hhi - previous_hhi
    }

    /// Security budget utilization for each AVS.
    /// Returns (avs_id, utilization_ratio) pairs.
    pub fn avs_utilization(&self) -> Vec<(String, f64)> {
        let snapshot = match self.snapshots.back() {
            Some(s) => s,
            None => return Vec::new(),
        };

        snapshot
            .avs_states
            .iter()
            .map(|avs| {
                let util = if avs.required_security > 0.0 {
                    avs.actual_security / avs.required_security
                } else {
                    0.0
                };
                (avs.avs_id.clone(), util)
            })
            .collect()
    }

    /// Standard deviation of per-operator returns.
    /// Rising divergence = early warning signal.
    pub fn operator_return_divergence(&self) -> f64 {
        let snapshot = match self.snapshots.back() {
            Some(s) => s,
            None => return 0.0,
        };

        let returns: Vec<f64> = snapshot
            .operators
            .iter()
            .map(|o| o.realized_return)
            .collect();

        if returns.len() < 2 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>()
            / (returns.len() - 1) as f64;

        variance.sqrt()
    }

    fn hhi_from_snapshot(&self, snapshot: &RestakingSnapshot) -> f64 {
        let total: f64 = snapshot.operators.iter().map(|o| o.total_delegated).sum();
        if total == 0.0 {
            return 0.0;
        }
        snapshot
            .operators
            .iter()
            .map(|o| (o.total_delegated / total).powi(2))
            .sum()
    }
}
```

## 8. Streaming payment aggregation [SPEC]

Sablier (and similar protocols like Llamapay) enable token streaming: continuous payments that flow from sender to recipient over time. Stream creation and cancellation rates aggregate into a sentiment signal for the broader ecosystem.

Three indicators.

### Stream creation rate

The number of new streams created per block, normalized by active stream count. A rising creation rate signals expanding economic activity -- new payroll arrangements, vesting schedules, subscription payments, or grant disbursements. The creation rate is a leading indicator of protocol ecosystem health because streams represent future economic commitments.

### Cancellation spike detection

A sudden increase in stream cancellations relative to the baseline cancellation rate. Cancellations break future payment commitments. A spike suggests distress: employers cutting payroll streams, projects canceling grant payments, or subscribers churning. The spike detection uses a z-score against the rolling cancellation rate.

### Payment velocity

The aggregate streaming rate (tokens per second flowing through all active streams) normalized by total locked tokens. High velocity means capital is moving. Low velocity means capital is sitting idle. Declining velocity with stable stream count suggests the streams are being created with lower values -- possibly a sign of decreasing confidence in the token or the economy.

```rust
use std::collections::VecDeque;

/// A snapshot of streaming payment protocol state.
#[derive(Clone, Debug)]
pub struct StreamingSnapshot {
    pub block_number: u64,
    pub active_streams: u64,
    pub streams_created_this_block: u64,
    pub streams_cancelled_this_block: u64,
    pub total_locked_tokens: f64,
    pub aggregate_rate_per_second: f64,
}

/// Analyzer for streaming payment aggregation signals.
pub struct StreamingPaymentAnalyzer {
    snapshots: VecDeque<StreamingSnapshot>,
    max_history: usize,
}

impl StreamingPaymentAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn update(&mut self, snapshot: StreamingSnapshot) {
        if self.snapshots.len() >= self.max_history {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    /// Stream creation rate normalized by active stream count.
    /// Higher = expanding activity.
    pub fn creation_rate(&self) -> f64 {
        match self.snapshots.back() {
            Some(s) if s.active_streams > 0 => {
                s.streams_created_this_block as f64 / s.active_streams as f64
            }
            _ => 0.0,
        }
    }

    /// Rolling mean creation rate over the last `window` snapshots.
    pub fn mean_creation_rate(&self, window: usize) -> f64 {
        let len = self.snapshots.len();
        if len == 0 {
            return 0.0;
        }

        let start = len.saturating_sub(window);
        let slice = &self.snapshots.as_slices().0[start..];

        let mut total_created = 0u64;
        let mut total_active = 0u64;

        for s in slice {
            total_created += s.streams_created_this_block;
            total_active += s.active_streams;
        }

        if total_active == 0 {
            return 0.0;
        }

        total_created as f64 / total_active as f64
    }

    /// Z-score of current cancellation rate against rolling history.
    /// High positive z-score = cancellation spike.
    pub fn cancellation_spike_score(&self) -> f64 {
        if self.snapshots.len() < 10 {
            return 0.0;
        }

        let rates: Vec<f64> = self
            .snapshots
            .iter()
            .map(|s| {
                if s.active_streams > 0 {
                    s.streams_cancelled_this_block as f64 / s.active_streams as f64
                } else {
                    0.0
                }
            })
            .collect();

        let n = rates.len() as f64;
        let mean = rates.iter().sum::<f64>() / n;
        let variance =
            rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        (rates.last().unwrap() - mean) / std_dev
    }

    /// Aggregate payment velocity: streaming rate / total locked.
    /// Dimensionless throughput metric.
    pub fn payment_velocity(&self) -> f64 {
        match self.snapshots.back() {
            Some(s) if s.total_locked_tokens > 0.0 => {
                s.aggregate_rate_per_second / s.total_locked_tokens
            }
            _ => 0.0,
        }
    }

    /// Is payment velocity declining while stream count is stable?
    /// This combination suggests declining average stream value.
    pub fn declining_conviction_signal(&self, window: usize) -> f64 {
        let len = self.snapshots.len();
        if len < window {
            return 0.0;
        }

        let velocities: Vec<f64> = self.snapshots
            .range((len - window)..)
            .map(|s| {
                if s.total_locked_tokens > 0.0 {
                    s.aggregate_rate_per_second / s.total_locked_tokens
                } else {
                    0.0
                }
            })
            .collect();

        let stream_counts: Vec<f64> = self.snapshots
            .range((len - window)..)
            .map(|s| s.active_streams as f64)
            .collect();

        let velocity_trend = linear_trend(&velocities);
        let count_trend = linear_trend(&stream_counts);

        // Declining velocity + stable/rising count = declining conviction
        if velocity_trend < 0.0 && count_trend >= -0.01 {
            velocity_trend.abs()
        } else {
            0.0
        }
    }
}
```

### Why streaming payments are invisible in TradFi

Payroll, subscriptions, and grant payments in traditional finance are private. You cannot observe how many new employment contracts were signed this block, or how many subscriptions were cancelled this second. DeFi streaming protocols make this data public by construction. The aggregate creation/cancellation dynamics are a real-time measure of economic activity that has no traditional analog.

## 9. Gas market microstructure [SPEC]

EIP-1559 transformed Ethereum's gas market from a first-price auction into a hybrid mechanism with a protocol-set base fee and a user-set priority fee (tip). The base fee adjusts each block to target 50% utilization. This creates a control system with observable dynamics -- the gas market is itself an indicator.

Four indicators.

### Base fee oscillation mode

The base fee follows a deterministic adjustment rule: increase by up to 12.5% if the previous block was more than 50% full, decrease by up to 12.5% if less than 50% full. This creates oscillatory behavior around the equilibrium. An FFT on the base fee time series reveals dominant frequencies.

Low-frequency oscillation (the base fee drifts slowly) indicates stable demand. High-frequency oscillation (rapid base fee swings) indicates demand volatility -- many blocks are either very full or very empty, and the control system is hunting for equilibrium.

### Priority fee distribution shape

The distribution of priority fees within each block encodes the urgency premium. The kurtosis of this distribution matters. High kurtosis (heavy tails) means a few transactions are paying orders of magnitude more than the rest -- a signature of MEV competition, where searchers bid aggressively to have their transactions included in a specific position within the block.

The skewness also carries information. Right-skewed priority fees (long right tail) indicate a small number of very urgent transactions. Symmetric distributions indicate uniform urgency.

### Block utilization cyclicality

Block fullness follows time-of-day and day-of-week patterns. US market hours produce different gas demand than Asian market hours. Weekend demand differs from weekday demand. The Golem detects these cycles via autocorrelation analysis on block utilization time series.

Cyclicality matters because it creates predictable windows of low and high gas cost. The Golem can time its own transactions -- executing non-urgent operations during predicted low-gas windows.

### Gas elasticity

The price sensitivity of transaction demand: how much does transaction count change when the base fee changes? Inelastic demand (transactions persist even at high gas prices) indicates that the activity is either highly profitable (worth paying for) or automated (bots that do not care about cost). Elastic demand (transactions drop off with rising gas) indicates discretionary activity.

Gas elasticity shifts over time and differs by transaction type. The Golem measures it as the rolling regression coefficient of transaction count on base fee.

```rust
use std::collections::VecDeque;
use std::f64::consts::PI;

/// A single block's gas market data.
#[derive(Clone, Debug)]
pub struct GasBlockSnapshot {
    pub block_number: u64,
    pub base_fee_gwei: f64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub priority_fees: Vec<f64>,
    pub transaction_count: u32,
}

/// Analyzer for gas market microstructure.
pub struct GasMarketAnalyzer {
    block_history: VecDeque<GasBlockSnapshot>,
    max_history: usize,
}

impl GasMarketAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            block_history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn update(&mut self, block: GasBlockSnapshot) {
        if self.block_history.len() >= self.max_history {
            self.block_history.pop_front();
        }
        self.block_history.push_back(block);
    }

    /// Frequency spectrum of the base fee time series.
    /// Returns (frequency, magnitude) pairs sorted by magnitude.
    pub fn base_fee_oscillation_spectrum(&self) -> Vec<(f64, f64)> {
        let n = self.block_history.len();
        if n < 16 {
            return Vec::new();
        }

        let values: Vec<f64> = self
            .block_history
            .iter()
            .map(|b| b.base_fee_gwei)
            .collect();
        let mean = values.iter().sum::<f64>() / n as f64;
        let centered: Vec<f64> = values.iter().map(|v| v - mean).collect();

        let mut spectrum = Vec::with_capacity(n / 2);

        for k in 1..=(n / 2) {
            let mut real = 0.0;
            let mut imag = 0.0;

            for (t, &val) in centered.iter().enumerate() {
                let angle = 2.0 * PI * k as f64 * t as f64 / n as f64;
                real += val * angle.cos();
                imag -= val * angle.sin();
            }

            let magnitude = (real * real + imag * imag).sqrt() / n as f64;
            let frequency = k as f64 / n as f64;
            spectrum.push((frequency, magnitude));
        }

        spectrum.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        spectrum
    }

    /// Kurtosis of the priority fee distribution for the most recent block.
    /// High kurtosis = heavy tails = MEV competition.
    pub fn priority_fee_kurtosis(&self) -> f64 {
        let block = match self.block_history.back() {
            Some(b) => b,
            None => return 0.0,
        };

        let fees = &block.priority_fees;
        if fees.len() < 4 {
            return 0.0;
        }

        let n = fees.len() as f64;
        let mean = fees.iter().sum::<f64>() / n;
        let m2: f64 = fees.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / n;
        let m4: f64 = fees.iter().map(|f| (f - mean).powi(4)).sum::<f64>() / n;

        if m2 == 0.0 {
            return 0.0;
        }

        // Excess kurtosis (normal distribution = 0)
        (m4 / (m2 * m2)) - 3.0
    }

    /// Rolling kurtosis over recent blocks (aggregated priority fees).
    pub fn rolling_priority_fee_kurtosis(&self, window: usize) -> f64 {
        let len = self.block_history.len();
        if len < window {
            return 0.0;
        }

        let mut all_fees = Vec::new();
        for block in self.block_history.range((len - window)..) {
            all_fees.extend_from_slice(&block.priority_fees);
        }

        if all_fees.len() < 4 {
            return 0.0;
        }

        let n = all_fees.len() as f64;
        let mean = all_fees.iter().sum::<f64>() / n;
        let m2: f64 = all_fees.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / n;
        let m4: f64 = all_fees.iter().map(|f| (f - mean).powi(4)).sum::<f64>() / n;

        if m2 == 0.0 {
            return 0.0;
        }

        (m4 / (m2 * m2)) - 3.0
    }

    /// Block utilization ratio for the most recent block.
    pub fn current_utilization(&self) -> f64 {
        match self.block_history.back() {
            Some(b) if b.gas_limit > 0 => b.gas_used as f64 / b.gas_limit as f64,
            _ => 0.0,
        }
    }

    /// Autocorrelation of block utilization at a given lag.
    /// Used to detect cyclicality (e.g., lag = ~7200 blocks for daily cycles).
    pub fn utilization_autocorrelation(&self, lag: usize) -> f64 {
        let n = self.block_history.len();
        if n < lag + 10 {
            return 0.0;
        }

        let utilizations: Vec<f64> = self
            .block_history
            .iter()
            .map(|b| {
                if b.gas_limit > 0 {
                    b.gas_used as f64 / b.gas_limit as f64
                } else {
                    0.0
                }
            })
            .collect();

        let mean = utilizations.iter().sum::<f64>() / n as f64;

        let mut cov = 0.0;
        let mut var = 0.0;

        for i in 0..(n - lag) {
            let x = utilizations[i] - mean;
            let y = utilizations[i + lag] - mean;
            cov += x * y;
            var += x * x;
        }

        if var == 0.0 {
            return 0.0;
        }

        cov / var
    }

    /// Gas price elasticity of demand: regression coefficient of
    /// transaction count on base fee over the last `window` blocks.
    /// Negative = elastic (higher gas reduces demand).
    /// Near zero = inelastic (demand persists regardless of gas).
    pub fn gas_elasticity(&self, window: usize) -> f64 {
        let len = self.block_history.len();
        if len < window {
            return 0.0;
        }

        let slice: Vec<&GasBlockSnapshot> =
            self.block_history.range((len - window)..).collect();

        let prices: Vec<f64> = slice.iter().map(|b| b.base_fee_gwei.ln()).collect();
        let counts: Vec<f64> = slice
            .iter()
            .map(|b| (b.transaction_count as f64).max(1.0).ln())
            .collect();

        // Log-log regression for elasticity
        let n = prices.len() as f64;
        let mean_p = prices.iter().sum::<f64>() / n;
        let mean_c = counts.iter().sum::<f64>() / n;

        let mut cov = 0.0;
        let mut var_p = 0.0;

        for i in 0..prices.len() {
            let dp = prices[i] - mean_p;
            let dc = counts[i] - mean_c;
            cov += dp * dc;
            var_p += dp * dp;
        }

        if var_p == 0.0 {
            return 0.0;
        }

        cov / var_p
    }
}
```

### Gas as a meta-indicator

Gas market indicators differ from the other families in this document: they are not protocol-specific but network-wide. Every DeFi transaction pays gas. The gas market integrates information from all on-chain activity into a single price signal.

This makes gas indicators a useful normalization baseline. A spike in Uniswap swap volume might be significant, or it might just reflect a network-wide activity surge (visible as elevated base fee). Normalizing swap volume by gas market activity isolates the pool-specific signal from the network-wide trend.

The Golem also uses gas elasticity for its own transaction timing. When gas elasticity is high (demand is sensitive to price), the Golem can wait for a low-gas window and execute then. When elasticity is low (demand is inelastic -- probably MEV or liquidation bots that cannot wait), the Golem knows it must compete on priority fee.

## 10. Emerging primitive indicators [SPEC]

The DeFi design space keeps expanding. Five emerging categories deserve indicator coverage even though their protocols are younger and their data shorter.

### Intent-based trading

UniswapX, CoW Protocol, and 1inch Fusion route trades through solver networks rather than directly through AMMs. The solver competition dynamics create new observables.

**Solver competition intensity**: the number of solvers submitting fills for a given order. High competition means the order size is attractive and the market is efficient. Low competition means either the order is unprofitable to fill or solver participation is declining.

**Batch auction clearing price deviation**: in CoW Protocol's batch auctions, the clearing price may differ from the AMM spot price. The deviation measures how much the auction mechanism improves (or worsens) execution relative to direct AMM routing. Persistent positive deviation means the auction is finding better prices; persistent negative deviation means something is wrong.

**MEV protection effectiveness**: intent-based protocols claim to protect users from MEV extraction. The indicator measures the actual MEV extracted from intent-based trades versus AMM trades of comparable size. Declining protection effectiveness is a risk signal for the protocol's value proposition.

```rust
/// Intent-based trading indicators.
pub struct IntentTradingAnalyzer {
    solver_counts: VecDeque<(u64, u32)>,     // (block, solver count)
    price_deviations: VecDeque<(u64, f64)>,  // (block, deviation from AMM)
    mev_extraction: VecDeque<(u64, f64)>,    // (block, extracted MEV in USD)
    max_history: usize,
}

impl IntentTradingAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            solver_counts: VecDeque::with_capacity(max_history),
            price_deviations: VecDeque::with_capacity(max_history),
            mev_extraction: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn record_auction(
        &mut self,
        block: u64,
        solver_count: u32,
        price_deviation: f64,
        mev_extracted: f64,
    ) {
        if self.solver_counts.len() >= self.max_history {
            self.solver_counts.pop_front();
            self.price_deviations.pop_front();
            self.mev_extraction.pop_front();
        }
        self.solver_counts.push_back((block, solver_count));
        self.price_deviations.push_back((block, price_deviation));
        self.mev_extraction.push_back((block, mev_extracted));
    }

    /// Mean solver count over recent auctions.
    pub fn mean_solver_competition(&self, window: usize) -> f64 {
        let len = self.solver_counts.len();
        if len == 0 {
            return 0.0;
        }
        let start = len.saturating_sub(window);
        let sum: u32 = self.solver_counts.range(start..).map(|(_, c)| c).sum();
        let count = len - start;
        sum as f64 / count as f64
    }

    /// Mean clearing price deviation from AMM spot.
    /// Positive = auction finds better prices. Negative = worse.
    pub fn mean_price_improvement(&self, window: usize) -> f64 {
        let len = self.price_deviations.len();
        if len == 0 {
            return 0.0;
        }
        let start = len.saturating_sub(window);
        let sum: f64 = self.price_deviations.range(start..).map(|(_, d)| d).sum();
        let count = len - start;
        sum / count as f64
    }

    /// Trend of MEV extraction from intent-based trades.
    /// Rising = protection degrading. Falling = protection improving.
    pub fn mev_protection_trend(&self, window: usize) -> f64 {
        let len = self.mev_extraction.len();
        if len < window {
            return 0.0;
        }
        let values: Vec<f64> = self
            .mev_extraction
            .range((len - window)..)
            .map(|(_, m)| *m)
            .collect();
        linear_trend(&values)
    }
}
```

### RWA protocol indicators

Tokenized real-world assets (treasuries, real estate, credit) create an on-chain/off-chain interface. Two indicators exploit the boundary.

**Tokenized treasury yield tracking**: the on-chain yield of tokenized T-bills (e.g., Ondo OUSG) should track the off-chain T-bill rate. Deviations indicate either a pricing error, a redemption issue, or a premium/discount reflecting on-chain demand dynamics not present in the off-chain market.

**On-chain credit spread dynamics**: tokenized private credit (e.g., Maple, Goldfinch) has credit spreads that differ from TradFi equivalents. The on-chain spread reflects DeFi-specific risk factors: smart contract risk, oracle risk, and governance risk. Tracking the on-chain credit spread over time provides a DeFi-specific fear gauge.

### Cross-chain messaging indicators

Bridge protocols and cross-chain messaging layers produce flow data that encodes inter-chain capital movement.

**Bridge flow analysis**: net token flows across bridges, per asset and per chain pair. Sustained outflows from one chain signal declining confidence in that chain's ecosystem. Inflows signal the opposite.

**Message latency as congestion signal**: cross-chain message delivery times depend on both source and destination chain congestion, plus relayer availability. Rising latency indicates congestion or relayer issues. Latency spikes precede cross-chain arbitrage breakdowns.

### Account abstraction indicators

ERC-4337 (account abstraction) creates a new transaction lifecycle: UserOperations go through bundlers before reaching the mempool.

**Bundler competition dynamics**: the number of active bundlers and their market share. Bundler concentration (few bundlers dominating) creates censorship risk. Bundler competition metrics track the health of the AA ecosystem.

**Paymaster subsidy patterns**: paymasters sponsor gas costs for specific UserOperations. The rate and selectivity of paymaster subsidies indicate which protocols are subsidizing user activity -- a proxy for growth investment.

**UserOp mempool depth**: the number of pending UserOperations waiting for bundling. Rising depth means bundlers cannot keep up with demand. Falling depth with rising submission rate means bundlers are becoming more efficient.

### Prediction market indicators

Polymarket and other on-chain prediction markets create probability estimates that correlate with DeFi positions.

**Outcome probability as correlated signal**: prediction market prices for events that affect DeFi (regulatory actions, protocol upgrades, economic data releases) provide forward-looking probability estimates. A rising probability of a negative regulatory action correlates with expected DeFi outflows. The Golem can use prediction market prices as exogenous signals for DeFi position management.

```rust
/// Emerging primitive indicators -- unified interface.
#[derive(Clone, Debug)]
pub enum EmergingPrimitiveSignal {
    /// Intent-based: solver competition, price improvement, MEV protection.
    IntentTrading {
        solver_competition: f64,
        price_improvement: f64,
        mev_trend: f64,
    },
    /// RWA: yield tracking deviation, credit spread.
    RealWorldAsset {
        yield_deviation: f64,
        credit_spread: f64,
    },
    /// Cross-chain: net flows, message latency.
    CrossChain {
        net_flow: f64,
        message_latency_ms: f64,
    },
    /// Account abstraction: bundler concentration, paymaster subsidies.
    AccountAbstraction {
        bundler_hhi: f64,
        paymaster_subsidy_rate: f64,
        mempool_depth: u64,
    },
    /// Prediction markets: event probability for DeFi-correlated outcomes.
    PredictionMarket {
        event_id: String,
        probability: f64,
        volume_24h: f64,
    },
}
```

## Cross-primitive composite indicators [SPEC]

Individual indicator families analyze one protocol type. The composites combine signals across multiple types. This is where DeFi-native TA produces its most distinctive signals, because cross-primitive observation requires a system that monitors multiple protocol categories simultaneously -- something no single protocol dashboard provides.

### DeFi Stress Index

A composite stress metric analogous to the VIX, but built from DeFi-native components instead of options-implied volatility. Five components, each normalized to [0.0, 1.0]:

1. **Lending liquidation proximity** (from Section 2): fraction of collateral value within 10% of liquidation.
2. **LP migration velocity** (from Section 1): normalized by rolling mean. High velocity during stress = flight from risk.
3. **Funding rate extremes** (from Section 3): absolute z-score of funding rates. Extreme positioning during stress.
4. **Gas priority fee kurtosis** (from Section 9): heavy tails indicate MEV competition spikes during volatile periods.
5. **Vault redemption queue pressure** (from Section 6): rising pressure across multiple vaults signals systemic stress.

The weights are not fixed. They are derived from the causal graph (Doc 4): components with stronger causal links to adverse outcomes receive higher weights. The weights update every Delta tick as the causal graph evolves.

```rust
use std::collections::HashMap;

/// Weights for the DeFi Stress Index components.
/// Updated by the causal graph (Doc 4) every Delta tick.
#[derive(Clone, Debug)]
pub struct StressWeights {
    pub lending_liquidation: f64,
    pub lp_migration: f64,
    pub funding_extremes: f64,
    pub gas_kurtosis: f64,
    pub vault_redemption: f64,
}

impl Default for StressWeights {
    fn default() -> Self {
        // Equal weights as initial prior. Causal graph updates these.
        Self {
            lending_liquidation: 0.2,
            lp_migration: 0.2,
            funding_extremes: 0.2,
            gas_kurtosis: 0.2,
            vault_redemption: 0.2,
        }
    }
}

/// Composite DeFi Stress Index.
///
/// Combines signals from five indicator families into a single
/// stress metric. Component weights are learned from the causal
/// graph rather than fixed, so the index adapts to changing
/// market structure.
pub struct DeFiStressIndex {
    lending: LendingDynamicsAnalyzer,
    cl: ConcentratedLiquidityAnalyzer,
    funding: FundingRateAnalyzer,
    gas: GasMarketAnalyzer,
    vault: VaultMechanicsAnalyzer,
    weights: StressWeights,
    /// Historical stress values for trend analysis.
    stress_history: VecDeque<(u64, f64)>,
    max_history: usize,
}

impl DeFiStressIndex {
    pub fn new(
        lending: LendingDynamicsAnalyzer,
        cl: ConcentratedLiquidityAnalyzer,
        funding: FundingRateAnalyzer,
        gas: GasMarketAnalyzer,
        vault: VaultMechanicsAnalyzer,
        max_history: usize,
    ) -> Self {
        Self {
            lending,
            cl,
            funding,
            gas,
            vault,
            weights: StressWeights::default(),
            stress_history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Update weights from the causal graph.
    /// Called every Delta tick.
    pub fn update_weights(&mut self, weights: StressWeights) {
        self.weights = weights;
    }

    /// Compute the composite stress index.
    /// Range [0.0, 1.0] where 1.0 = maximum stress.
    pub fn compute(&self) -> f64 {
        let components = self.raw_components();

        let weighted_sum = components.lending_liquidation * self.weights.lending_liquidation
            + components.lp_migration * self.weights.lp_migration
            + components.funding_extremes * self.weights.funding_extremes
            + components.gas_kurtosis * self.weights.gas_kurtosis
            + components.vault_redemption * self.weights.vault_redemption;

        let weight_sum = self.weights.lending_liquidation
            + self.weights.lp_migration
            + self.weights.funding_extremes
            + self.weights.gas_kurtosis
            + self.weights.vault_redemption;

        if weight_sum == 0.0 {
            return 0.0;
        }

        (weighted_sum / weight_sum).clamp(0.0, 1.0)
    }

    /// Per-component contributions to the stress index.
    /// Useful for diagnosing what is driving stress.
    pub fn component_contributions(&self) -> Vec<(&str, f64)> {
        let components = self.raw_components();
        vec![
            ("lending_liquidation", components.lending_liquidation * self.weights.lending_liquidation),
            ("lp_migration", components.lp_migration * self.weights.lp_migration),
            ("funding_extremes", components.funding_extremes * self.weights.funding_extremes),
            ("gas_kurtosis", components.gas_kurtosis * self.weights.gas_kurtosis),
            ("vault_redemption", components.vault_redemption * self.weights.vault_redemption),
        ]
    }

    /// Record current stress level for trend analysis.
    pub fn record(&mut self, block_number: u64) {
        let stress = self.compute();
        if self.stress_history.len() >= self.max_history {
            self.stress_history.pop_front();
        }
        self.stress_history.push_back((block_number, stress));
    }

    /// Is stress rising or falling?
    pub fn stress_trend(&self, window: usize) -> f64 {
        let len = self.stress_history.len();
        if len < window {
            return 0.0;
        }
        let values: Vec<f64> = self
            .stress_history
            .range((len - window)..)
            .map(|(_, s)| *s)
            .collect();
        linear_trend(&values)
    }

    fn raw_components(&self) -> StressComponents {
        // Normalize each component to [0.0, 1.0]

        // Lending: fraction of collateral within 10% of liquidation
        let lending_liquidation = self.lending.liquidation_at_risk(0.10).clamp(0.0, 1.0);

        // LP migration: current velocity / rolling mean, capped
        let velocity = self.cl.migration_velocity();
        let lp_migration = (velocity * 5.0).tanh(); // sigmoid-like normalization

        // Funding: absolute z-score, normalized
        let z = self.funding.z_score().abs();
        let funding_extremes = (z / 3.0).clamp(0.0, 1.0);

        // Gas: kurtosis, normalized (excess kurtosis > 6 = high)
        let kurtosis = self.gas.priority_fee_kurtosis().max(0.0);
        let gas_kurtosis = (kurtosis / 6.0).clamp(0.0, 1.0);

        // Vault: redemption pressure directly in [0.0, 1.0+]
        let vault_redemption = self.vault.redemption_queue_pressure().clamp(0.0, 1.0);

        StressComponents {
            lending_liquidation,
            lp_migration,
            funding_extremes,
            gas_kurtosis,
            vault_redemption,
        }
    }
}

struct StressComponents {
    lending_liquidation: f64,
    lp_migration: f64,
    funding_extremes: f64,
    gas_kurtosis: f64,
    vault_redemption: f64,
}
```

### Capital Rotation Indicator

Tracks capital flow between DeFi primitive types. When TVL in LP positions declines while lending supply increases, capital is rotating from yield-seeking to risk-off. The rotation vector identifies which primitives are gaining and losing capital, and the speed of rotation.

```rust
/// DeFi primitive categories for capital flow tracking.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeFiPrimitive {
    LiquidityProvision,
    Lending,
    Staking,
    Restaking,
    Vaults,
    Perpetuals,
    Options,
    YieldTokenization,
    StreamingPayments,
}

/// Tracks capital allocation across DeFi primitive types.
pub struct CapitalRotationIndicator {
    /// Per-primitive TVL history.
    tvl_tracker: HashMap<DeFiPrimitive, VecDeque<(u64, f64)>>,
    max_history: usize,
}

impl CapitalRotationIndicator {
    pub fn new(max_history: usize) -> Self {
        Self {
            tvl_tracker: HashMap::new(),
            max_history,
        }
    }

    /// Record a TVL observation for a primitive type.
    pub fn record_tvl(
        &mut self,
        primitive: DeFiPrimitive,
        block: u64,
        tvl: f64,
    ) {
        let history = self
            .tvl_tracker
            .entry(primitive)
            .or_insert_with(|| VecDeque::with_capacity(self.max_history));

        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back((block, tvl));
    }

    /// Compute the rotation vector: for each pair of primitives,
    /// the net capital flow direction and magnitude.
    ///
    /// Returns (source, destination, flow_magnitude) triples where
    /// positive magnitude means capital is flowing from source to
    /// destination (source TVL declining, destination TVL rising).
    pub fn rotation_vector(&self) -> Vec<(DeFiPrimitive, DeFiPrimitive, f64)> {
        let growth_rates: HashMap<&DeFiPrimitive, f64> = self
            .tvl_tracker
            .iter()
            .filter_map(|(prim, history)| {
                if history.len() < 2 {
                    return None;
                }
                let recent = history[history.len() - 1].1;
                let previous = history[history.len() - 2].1;
                if previous == 0.0 {
                    return None;
                }
                Some((prim, (recent - previous) / previous))
            })
            .collect();

        let mut flows = Vec::new();

        let primitives: Vec<&DeFiPrimitive> = growth_rates.keys().copied().collect();
        for i in 0..primitives.len() {
            for j in (i + 1)..primitives.len() {
                let rate_a = growth_rates[primitives[i]];
                let rate_b = growth_rates[primitives[j]];
                let diff = rate_b - rate_a;

                if diff.abs() > 0.001 {
                    // Minimum threshold to report
                    if diff > 0.0 {
                        // Capital flowing from A to B
                        flows.push((
                            primitives[i].clone(),
                            primitives[j].clone(),
                            diff,
                        ));
                    } else {
                        // Capital flowing from B to A
                        flows.push((
                            primitives[j].clone(),
                            primitives[i].clone(),
                            diff.abs(),
                        ));
                    }
                }
            }
        }

        flows.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        flows
    }

    /// Overall rotation intensity: sum of absolute growth rate
    /// differences across all pairs. High = active rotation.
    /// Low = stable allocation.
    pub fn rotation_intensity(&self) -> f64 {
        self.rotation_vector().iter().map(|(_, _, m)| m).sum()
    }

    /// Identify the "risk appetite" direction: are flows moving toward
    /// higher-risk or lower-risk primitives?
    /// Positive = risk-on. Negative = risk-off.
    pub fn risk_appetite(&self) -> f64 {
        // Simple risk ranking (higher = riskier)
        fn risk_rank(p: &DeFiPrimitive) -> f64 {
            match p {
                DeFiPrimitive::Lending => 0.2,
                DeFiPrimitive::Staking => 0.3,
                DeFiPrimitive::Vaults => 0.4,
                DeFiPrimitive::StreamingPayments => 0.3,
                DeFiPrimitive::LiquidityProvision => 0.6,
                DeFiPrimitive::YieldTokenization => 0.5,
                DeFiPrimitive::Restaking => 0.5,
                DeFiPrimitive::Perpetuals => 0.8,
                DeFiPrimitive::Options => 0.9,
            }
        }

        let flows = self.rotation_vector();
        if flows.is_empty() {
            return 0.0;
        }

        let mut weighted_direction = 0.0;
        let mut total_magnitude = 0.0;

        for (source, dest, magnitude) in &flows {
            let risk_change = risk_rank(dest) - risk_rank(source);
            weighted_direction += risk_change * magnitude;
            total_magnitude += magnitude;
        }

        if total_magnitude == 0.0 {
            return 0.0;
        }

        weighted_direction / total_magnitude
    }
}
```

### Cross-Protocol Momentum

When multiple independent protocols show directional alignment -- LP asymmetry bullish, funding rate positive, lending utilization rising, vault inflows increasing -- the momentum signal is stronger than any individual indicator. The cross-protocol momentum indicator measures this alignment.

The measurement uses the HDC entanglement mechanism from Doc 1. Each protocol's directional signal is encoded as a hypervector. The bundle (majority vote) of all protocol hypervectors produces a consensus direction vector. The similarity between this consensus vector and each individual protocol's vector measures how aligned or divergent each protocol is from the consensus.

```rust
/// Cross-protocol momentum measurement.
///
/// Uses HDC encoding (Doc 1) to measure directional alignment
/// across multiple protocol types simultaneously.
pub struct CrossProtocolMomentum {
    /// Per-protocol directional signal: positive = bullish, negative = bearish.
    protocol_signals: HashMap<String, VecDeque<(u64, f64)>>,
    max_history: usize,
}

impl CrossProtocolMomentum {
    pub fn new(max_history: usize) -> Self {
        Self {
            protocol_signals: HashMap::new(),
            max_history,
        }
    }

    /// Record a directional signal for a protocol.
    /// Signal range: [-1.0, 1.0] where +1.0 = strongly bullish.
    pub fn record_signal(&mut self, protocol: &str, block: u64, signal: f64) {
        let history = self
            .protocol_signals
            .entry(protocol.to_string())
            .or_insert_with(|| VecDeque::with_capacity(self.max_history));

        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back((block, signal));
    }

    /// Mean directional signal across all protocols.
    /// Range [-1.0, 1.0]. Near +/-1.0 = strong cross-protocol alignment.
    /// Near 0.0 = mixed signals.
    pub fn consensus_direction(&self) -> f64 {
        let signals: Vec<f64> = self
            .protocol_signals
            .values()
            .filter_map(|h| h.back().map(|(_, s)| *s))
            .collect();

        if signals.is_empty() {
            return 0.0;
        }

        signals.iter().sum::<f64>() / signals.len() as f64
    }

    /// Standard deviation of per-protocol signals.
    /// Low = aligned. High = divergent.
    pub fn signal_dispersion(&self) -> f64 {
        let signals: Vec<f64> = self
            .protocol_signals
            .values()
            .filter_map(|h| h.back().map(|(_, s)| *s))
            .collect();

        if signals.len() < 2 {
            return 0.0;
        }

        let mean = signals.iter().sum::<f64>() / signals.len() as f64;
        let variance = signals
            .iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f64>()
            / (signals.len() - 1) as f64;

        variance.sqrt()
    }

    /// Cross-protocol momentum strength: consensus direction * (1 - dispersion).
    /// Strong when signals are aligned and directional.
    /// Weak when signals are divergent or neutral.
    pub fn momentum_strength(&self) -> f64 {
        let consensus = self.consensus_direction();
        let dispersion = self.signal_dispersion().min(1.0);
        consensus * (1.0 - dispersion)
    }

    /// Which protocols diverge most from consensus?
    /// Returns (protocol, deviation) sorted by absolute deviation.
    pub fn divergent_protocols(&self) -> Vec<(String, f64)> {
        let consensus = self.consensus_direction();

        let mut deviations: Vec<(String, f64)> = self
            .protocol_signals
            .iter()
            .filter_map(|(name, history)| {
                history.back().map(|(_, signal)| {
                    (name.clone(), signal - consensus)
                })
            })
            .collect();

        deviations.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        deviations
    }
}
```

### Implied Correlation Index

Options-implied correlation versus realized correlation across protocol pairs. When the on-chain options surface (Section 4) implies a correlation between two assets that differs from their realized correlation (computed from actual price movements), the gap represents either a mispricing or a market expectation of correlation regime change.

A rising implied correlation with stable realized correlation suggests the market expects a shock that will increase co-movement -- a systemic event that hits multiple protocols simultaneously. A falling implied correlation with stable realized suggests the market expects divergence -- different protocols heading in different directions.

```rust
/// Implied vs. realized correlation analysis across protocol pairs.
pub struct ImpliedCorrelationIndex {
    /// Per-pair implied correlations from options surfaces.
    implied: HashMap<(String, String), VecDeque<(u64, f64)>>,
    /// Per-pair realized correlations from price returns.
    realized: HashMap<(String, String), VecDeque<(u64, f64)>>,
    max_history: usize,
}

impl ImpliedCorrelationIndex {
    pub fn new(max_history: usize) -> Self {
        Self {
            implied: HashMap::new(),
            realized: HashMap::new(),
            max_history,
        }
    }

    pub fn record_implied(
        &mut self,
        asset_a: &str,
        asset_b: &str,
        block: u64,
        correlation: f64,
    ) {
        let key = Self::pair_key(asset_a, asset_b);
        let history = self
            .implied
            .entry(key)
            .or_insert_with(|| VecDeque::with_capacity(self.max_history));
        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back((block, correlation));
    }

    pub fn record_realized(
        &mut self,
        asset_a: &str,
        asset_b: &str,
        block: u64,
        correlation: f64,
    ) {
        let key = Self::pair_key(asset_a, asset_b);
        let history = self
            .realized
            .entry(key)
            .or_insert_with(|| VecDeque::with_capacity(self.max_history));
        if history.len() >= self.max_history {
            history.pop_front();
        }
        history.push_back((block, correlation));
    }

    /// Correlation risk premium: implied - realized.
    /// Positive = market pricing in more correlation than realized.
    /// Negative = market pricing in less.
    pub fn correlation_premium(
        &self,
        asset_a: &str,
        asset_b: &str,
    ) -> Option<f64> {
        let key = Self::pair_key(asset_a, asset_b);

        let implied = self.implied.get(&key)?.back()?.1;
        let realized = self.realized.get(&key)?.back()?.1;

        Some(implied - realized)
    }

    /// Mean correlation premium across all pairs.
    /// Positive = systemic stress priced into options.
    pub fn mean_correlation_premium(&self) -> f64 {
        let mut total = 0.0;
        let mut count = 0;

        for (key, impl_history) in &self.implied {
            if let Some(real_history) = self.realized.get(key) {
                if let (Some(impl_val), Some(real_val)) =
                    (impl_history.back(), real_history.back())
                {
                    total += impl_val.1 - real_val.1;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return 0.0;
        }
        total / count as f64
    }

    /// Is the correlation premium rising or falling?
    pub fn correlation_premium_trend(&self, window: usize) -> f64 {
        // Compute mean premium at each block in the window
        // This is approximate -- we use the latest `window` entries
        // from the first available pair as a proxy for timing
        let premiums: Vec<f64> = self
            .implied
            .iter()
            .next()
            .map(|(key, impl_history)| {
                let real_history = match self.realized.get(key) {
                    Some(h) => h,
                    None => return Vec::new(),
                };

                let len = impl_history.len().min(real_history.len());
                if len < window {
                    return Vec::new();
                }

                (0..window)
                    .map(|i| {
                        let idx = len - window + i;
                        impl_history[idx].1 - real_history[idx].1
                    })
                    .collect()
            })
            .unwrap_or_default();

        if premiums.len() < 2 {
            return 0.0;
        }

        linear_trend(&premiums)
    }

    fn pair_key(a: &str, b: &str) -> (String, String) {
        // Canonical ordering for consistent key lookup
        if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }
}
```

## Subsystem interactions [SPEC]

The DeFi-native indicators interact with every other subsystem in the Bardo runtime. These are the main interaction paths.

### Chain intelligence pipeline

All ten indicator families consume data from the chain intelligence pipeline: `bardo-witness` for raw block data, `bardo-triage` for scored events, and `bardo-protocol-state` for live contract state. The pipeline is the sole data source. No indicator family makes its own RPC calls.

This constraint matters for two reasons. First, it means the indicator suite respects the Golem's attention filter. If `bardo-chain-scope` does not include a protocol in the interest set, no indicator can analyze it. Indicators that need data from a new protocol must request it through the chain scope feedback loop. Second, it means all indicators operate on the same consistent view of chain state -- there are no race conditions between indicators seeing different blocks.

### CorticalState signals

The DeFi-native indicator suite writes four new signals to `TaCorticalExtension`:

```rust
/// DeFi-native indicator signals for CorticalState.
/// Added to TaCorticalExtension alongside the signals from Docs 1-6.
pub struct DefiNativeCorticalSignals {
    /// DeFi Stress Index [0.0, 1.0]. High = systemic stress.
    pub defi_stress: AtomicU32,       // f32 as bits

    /// Capital rotation intensity. High = active rotation.
    pub rotation_intensity: AtomicU32, // f32 as bits

    /// Cross-protocol momentum [-1.0, 1.0]. Directional alignment.
    pub cross_momentum: AtomicU32,     // f32 as bits

    /// Correlation premium. Implied - realized. Systemic risk gauge.
    pub correlation_premium: AtomicU32, // f32 as bits
}
```

Other subsystems read these:

- The **mortality engine** reads `defi_stress` and factors it into environmental risk assessment. Sustained stress above 0.7 might trigger Conservation phase.
- The **Oracle** reads `cross_momentum` as a directional input for multi-asset predictions.
- The **Daimon** reads `correlation_premium` and modulates emotional response: rising premium triggers caution-associated affect.
- The **adversarial detector** (Doc 8) reads `rotation_intensity` -- rapid rotation away from a specific primitive might indicate informed insiders exiting ahead of bad news.

### HDC encoding (Doc 1)

Every indicator output is encoded as a 10,240-bit BSC hypervector for storage in the Grimoire and similarity queries via the ANN index. The encoding scheme:

- **Scalar indicators** (asymmetry index, z-score, HHI, etc.) are quantized into 16 bins and encoded using level hypervectors from Doc 1's codebook.
- **Distribution indicators** (liquidation proximity histogram, priority fee distribution) are encoded using spatial binding: each bin position is bound with the bin magnitude, then bundled into a single vector.
- **Composite indicators** (DeFi Stress Index, rotation vector) are encoded by binding their component encodings with role hypervectors (one per component), then bundling.

The encoding preserves similarity: two market states with similar indicator profiles produce hypervectors with low Hamming distance. This enables the Golem to retrieve similar past states from the Grimoire during Theta-tick analysis -- "when have I seen a stress profile like this before, and what happened next?"

### Signal metabolism (Doc 3)

All ten indicator families form signal families that compete for compute budget in Doc 3's Hebbian selection mechanism. The families:

| Family | Indicators | Budget claim |
|--------|-----------|--------------|
| Concentrated liquidity | Asymmetry, migration, gaps, HHI, JIT | Per-pool (scales with pool count) |
| Lending dynamics | Utilization FFT, rate velocity, liquidation, supply-borrow | Per-market (scales with tracked markets) |
| Funding rate | z-score, OU half-life, decorrelation, OI flow | Per-perp market |
| Options Greeks | IV curvature, skew RoC, net gamma | Per-options protocol |
| Yield term structure | Slope, velocity, premium, divergence | Per-Pendle market |
| Vault mechanics | Acceleration, redemption, rotation, harvest | Per-vault |
| Restaking flows | Delegation HHI, AVS util, operator divergence | Network-wide (single budget) |
| Streaming payments | Creation, cancellation, velocity | Per-streaming protocol |
| Gas microstructure | Base fee FFT, kurtosis, cyclicality, elasticity | Network-wide (single budget) |
| Composites | Stress, rotation, momentum, correlation | Network-wide (derived from others) |

Budget allocation follows performance. A family that consistently produces `TaInsight` values leading to accurate Oracle predictions gets more budget. A family that produces noise gets pruned. Over the Golem's lifetime, the active indicator set narrows to the families that work for the specific markets and conditions the Golem encounters.

### Causal graph (Doc 4)

The cross-primitive composite indicators are the primary input to causal edge discovery. When the DeFi Stress Index rises and is followed by a price decline, is the relationship causal or spurious? The causal discovery engine tests this by running interventional simulations: fork the state via mirage-rs, artificially elevate the stress index components (increase liquidation proximity, spike migration velocity), and observe whether the price decline follows.

This creates a feedback loop specific to the composite indicators. Each component's causal weight in the stress index is updated based on interventional evidence, not just correlation. A component that causally drives adverse outcomes receives higher weight. A component that merely correlates receives lower weight. The stress index becomes more accurate over time because its weights are calibrated by causal evidence.

## Cybernetic feedback loop [SPEC]

The full DeFi-native indicator suite participates in Bardo's cybernetic loop at every stage:

```
Chain data arrives (bardo-witness)
    |
    v
Triage scores events (bardo-triage)
    |
    v
Protocol state updates (bardo-protocol-state)
    |
    v
[Gamma tick] DeFi-native indicators update streaming state
    - Tick distributions, utilization ratios, funding rates,
      gas market data, all incremental
    - CorticalState signals updated (stress, momentum, etc.)
    |
    v
[Theta tick] Full indicator analysis
    - Composites computed (stress index, rotation, momentum)
    - TaInsight values produced for Oracle
    - HDC encoding for Grimoire storage
    |
    v
Oracle decides; tools execute
    |
    v
[Reflect] Outcomes observed
    - Did the stress index predict the drawdown?
    - Did the rotation indicator anticipate the capital flow?
    - Prediction residuals computed
    |
    v
[Delta tick] Model updates
    - Causal graph updates stress weights
    - Signal metabolism adjusts family budgets
    - OU parameters re-estimated
    |
    v
[Dream NREM] Replay episodes
    - High-stress episodes replayed to refine stress response
    - Composite indicator accuracy backtested
    |
    v
[Dream REM] Counterfactual exploration
    - "What if liquidation proximity had been 5% higher?"
    - Interventional causal tests on composite components
    |
    v
Chain scope updates (bardo-chain-scope)
    - Interest filter adjusted based on which indicator families
      are performing well
    - Protocols tracked by high-performing families get
      higher attention scores
    |
    v
(back to top: chain data arrives with refined attention)
```

The loop has a specific property for DeFi-native indicators: the indicators themselves can change what the Golem observes. If the concentrated liquidity analyzer discovers that a specific pool's tick distribution predicts price moves, chain scope will elevate that pool's attention score, causing the Witness to fetch more data about it, causing the analyzer to receive more frequent updates, causing its predictions to improve. Attention follows signal quality.

The reverse also works. If a lending market's indicators stop predicting anything useful (perhaps the market dried up and utilization is flat), the signal metabolism decays that family's budget, chain scope reduces the market's attention score, and the Witness eventually stops fetching full state for that market. The Golem organically stops watching things that are not informative.

## Evaluation protocol [SPEC]

### Per-indicator predictive accuracy

Each indicator is evaluated by its ability to predict future outcomes within its domain:

- **Concentrated liquidity indicators**: predict next-N-block price direction and volatility for the pool. Measured by AUROC for direction and RMSE for volatility.
- **Lending indicators**: predict liquidation events (binary) and utilization direction. Measured by precision-recall for liquidations, AUROC for direction.
- **Funding rate indicators**: predict funding rate sign change (binary) and magnitude. Measured by AUROC and MAE.
- **Options indicators**: predict realized volatility relative to implied (vol risk premium). Measured by RMSE.
- **Yield indicators**: predict yield curve changes. Measured by directional accuracy.
- **Vault indicators**: predict share price acceleration direction. Measured by AUROC.
- **Restaking indicators**: predict operator slashing events. Measured by precision-recall (rare events).
- **Streaming indicators**: predict cancellation spikes. Measured by precision-recall.
- **Gas indicators**: predict base fee direction and transaction throughput. Measured by MAE.
- **Composites**: predict portfolio drawdowns exceeding threshold. Measured by precision-recall and early warning lead time.

### Composite vs. individual performance

The primary research question for this document is whether composite indicators outperform their individual components. The null hypothesis is that the composite indicators provide no improvement over simply using all individual indicators independently.

Test protocol:
1. Train the Oracle with individual indicators only. Measure prediction accuracy and portfolio performance.
2. Train the Oracle with composite indicators added. Measure the same metrics.
3. Compare using paired Welch's t-test on per-Theta-tick prediction accuracy.

The secondary question is whether the causal-graph-weighted stress index outperforms an equal-weighted version. This tests whether the learned weights (from Doc 4's interventional testing) add value beyond simple averaging.

### Novelty validation

For each indicator, demonstrate that the required data is structurally unavailable in traditional finance:

| Indicator | Required DeFi data | TradFi equivalent | Why unavailable |
|-----------|-------------------|-------------------|-----------------|
| Tick asymmetry | Full tick bitmap | Full order book | Hidden; only top-of-book visible |
| Migration velocity | Position-level liquidity changes | Market maker inventory | Proprietary |
| JIT score | Per-block liquidity transients | Flash orders | Banned by regulation |
| Liquidation proximity | Per-position health factors | Margin levels | Broker-confidential |
| Utilization FFT | Block-by-block utilization | Lending utilization | Not public, not real-time |
| Funding z-score | Real-time funding rate | Repo rate | Lagged, aggregated |
| OU half-life | High-frequency funding data | Overnight rate dynamics | Available but different mechanism |
| Net gamma exposure | Exact position-level gamma | Aggregate dealer gamma | Estimated, not observed |
| Yield curve slope | PT/YT prices at multiple maturities | Treasury curve | Observable, but positions hidden |
| Redemption pressure | Pending withdrawals / total assets | Fund redemption queue | Reported quarterly, not real-time |
| Delegation HHI | Operator-level stake amounts | Fund manager AUM | Aggregated, delayed |
| Stream creation rate | Per-block stream creation events | Employment data | Monthly, surveyed |
| Priority fee kurtosis | Per-transaction priority fees | Transaction costs | Available but less granular |
| Cross-protocol momentum | Simultaneous multi-protocol observation | Multi-asset momentum | Observable, but not from protocol state |

The table confirms that most indicators require data visible only through on-chain state. The few that have TradFi analogs (funding rate, yield curve, gas/transaction costs) differ in granularity: DeFi provides per-block, per-transaction resolution versus TradFi's daily or lower frequency.

### Computational cost

The full indicator suite must run within one Gamma tick's compute budget (target: under 5ms total). Per-family cost estimates:

| Family | Operations per update | Estimated Gamma cost |
|--------|----------------------|---------------------|
| Concentrated liquidity | O(ticks) ~500 | 50 us |
| Lending dynamics | O(positions) ~1000 | 100 us |
| Funding rate | O(1) | 5 us |
| Options Greeks | O(entries) ~200 | 30 us |
| Yield term structure | O(points * assets) ~50 | 10 us |
| Vault mechanics | O(1) | 5 us |
| Restaking flows | O(operators) ~100 | 20 us |
| Streaming payments | O(1) | 5 us |
| Gas microstructure | O(txs) ~200 | 30 us |
| Composites | O(families) ~10 | 10 us |
| **Total** | | **~265 us** |

The total is well within the 5ms Gamma budget, leaving room for the other TA subsystems (Docs 1-6, 8-10) and the core runtime operations.

Theta-tick operations (full composite computation, HDC encoding, TaInsight generation) are more expensive but have a longer budget (up to 500ms). The FFT computations (utilization and gas oscillation modes) run at Theta frequency, not Gamma, keeping the per-Gamma cost low.

## References

1. Adams, H., Zinsmeister, N., Salem, M., Keefer, R., and Robinson, D. (2021). "Uniswap v3 Core." Uniswap Labs whitepaper. Concentrated liquidity mechanism.

2. Aave. (2020). "Aave Protocol Whitepaper v2.0." Interest rate model and liquidation mechanics.

3. Uhlenbeck, G.E. and Ornstein, L.S. (1930). "On the Theory of the Brownian Motion." *Physical Review,* 36(5), 823-841. Ornstein-Uhlenbeck process used for funding rate mean-reversion modeling.

4. Lambert, G. (2023). "Panoptic: Perpetual Options on Uniswap v3." Panoptic whitepaper. On-chain options Greeks derivation from AMM mechanics.

5. Pendle Finance. (2023). "Pendle Finance: Yield Tokenization." Whitepaper. PT/YT yield curve mechanics.

6. EIP-4626. (2022). "Tokenized Vault Standard." Ethereum Improvement Proposal. Standard vault interface.

7. EigenLayer. (2023). "EigenLayer: The Restaking Collective." Whitepaper. Restaking delegation and AVS security mechanics.

8. Sablier. (2023). "Sablier V2: Token Streaming Protocol." Documentation. Streaming payment mechanics.

9. Buterin, V. (2019). "EIP-1559: Fee market change for ETH 1.0 chain." Ethereum Improvement Proposal. Base fee adjustment mechanism.

10. Herfindahl, O.C. (1950). "Concentration in the U.S. Steel Industry." Columbia University PhD dissertation. Herfindahl-Hirschman Index for concentration measurement.

11. Cooley, J.W. and Tukey, J.W. (1965). "An Algorithm for the Machine Calculation of Complex Fourier Series." *Mathematics of Computation,* 19(90), 297-301. FFT algorithm basis for oscillation mode analysis.

12. Graf, T.M. and Lemire, D. (2022). "Binary Fuse Filters: Fast and Smaller Than Xor Filters." *ACM Journal of Experimental Algorithmics,* 27, 1-15. Filter design used in bardo-witness.

13. Kanatnikov, P., Kaplan, E.H., and Li, L. (2023). "Order Flow, Liquidity, and Market Making in Decentralized Exchanges." Working paper. MEV and JIT liquidity dynamics in Uniswap V3.

14. Biais, B., Bisiere, C., Bouvard, M., Casamatta, C., and Menkveld, A.J. (2023). "Equilibrium Bitcoin Pricing." *Journal of Finance,* 78(4), 1727-1786. Funding rate and perpetual pricing mechanism analysis.

15. Milionis, J., Moallemi, C.C., Roughgarden, T., and Zhang, A.L. (2023). "Automated Market Making and Loss-Versus-Rebalancing." Working paper. LP loss analysis and tick-space dynamics.

16. UniswapX. (2023). "UniswapX: Auction-Based Routing." Whitepaper. Intent-based trading and solver competition.

17. CoW Protocol. (2023). "CoW Protocol: Coincidence of Wants." Documentation. Batch auction mechanics.

18. EIP-4337. (2023). "Account Abstraction Using an Alt Mempool." Ethereum Improvement Proposal. UserOperation and bundler mechanics.
