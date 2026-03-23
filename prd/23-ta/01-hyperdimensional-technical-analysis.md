<!-- prd2/23-ta/01-hyperdimensional-technical-analysis.md -->
<!-- Source: tmp/research/witness-research/new/ta/01-hyperdimensional-technical-analysis.md -->

> **Version**: 1.0 | **Status**: Active | **Section**: 23-ta
>
> **Crates**: `bardo-ta-hdc`, `golem-core::hdc`
>
> **Cross-references**:
> - [shared/hdc-vsa.md](../shared/hdc-vsa.md) -- Binary Spatter Code foundations: the `Hypervector`, `ItemMemory`, and `BundleAccumulator` primitives this document builds all pattern encoding on top of
> - [14-chain/01-witness.md](../14-chain/01-witness.md) -- block ingestion and Binary Fuse pre-screening that delivers raw on-chain data into the TA pipeline
> - [14-chain/02-triage.md](../14-chain/02-triage.md) -- the 4-stage classification pipeline whose curiosity-scored transactions become the input observations for HDC pattern matching
> - [01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- the 32-signal perception surface where HDC TA signals are written via TaCorticalExtension for other subsystems to read
> - [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- the 9-step decision cycle whose Gamma/Theta/Delta timescales gate when HDC pattern matching runs
> - [23-ta/00-witness-as-technical-analyst.md](00-witness-as-technical-analyst.md) -- Doc 0: prerequisite system context mapping the full data pipeline from block to cognition
> - [23-ta/02-spectral-liquidity-manifolds.md](02-spectral-liquidity-manifolds.md) -- Doc 2: Riemannian manifold geometry whose curvature signals cross-pollinate with HDC pattern features
> - [23-ta/03-adaptive-signal-metabolism.md](03-adaptive-signal-metabolism.md) -- Doc 3: evolutionary signal system where HDC-encoded patterns compete, reproduce, and die

---

> **Reader orientation:** This document specifies HDC-based pattern encoding for DeFi technical analysis within the Golem (mortal autonomous DeFi agent) runtime. It belongs to the **TA research layer** (Doc 1 of 10) and covers how market patterns are encoded as 10,240-bit Binary Spatter Code hypervectors, temporally convolved, and cross-protocol entangled. You should understand DeFi pool mechanics (Uniswap V3 ticks, Aave positions) and be comfortable with high-dimensional computing concepts. For Bardo-specific terms, see `prd2/shared/glossary.md`.

# Hyperdimensional Technical Analysis [SPEC]

**Audience:** Systems engineers implementing TA signal processing in the Bardo runtime; researchers evaluating HDC for financial time series analysis.

## Abstract

Technical analysis generates signals. Hundreds of them, across dozens of indicator families, on every asset pair, at every timescale. The standard approach processes these signals one at a time: compute RSI, check threshold, compute MACD, check crossover, repeat. Each indicator is a separate scan over the same time series. Correlating indicators requires explicit logic for every combination. Cross-protocol analysis requires maintaining separate state machines per protocol, with ad-hoc correlation heuristics bolted on top.

This document describes a different approach. Every TA signal, DeFi event, and protocol state change gets encoded as a 10,240-bit Binary Spatter Code (BSC) hypervector using the role-filler binding scheme defined in `00-foundations.md`. These hypervectors compose algebraically: bind encodes relationships, bundle encodes co-occurrence, permute encodes temporal position. The result is three capabilities that traditional TA cannot match. Pattern algebra composes arbitrary signal queries and evaluates them in ~10ns via XOR + POPCNT. Temporal convolution detects shift-invariant patterns across time series without explicit sliding windows. Cross-protocol entanglement tracking detects when previously independent DeFi protocols begin correlating, providing early warning of systemic events.

The encoding scheme covers every DeFi primitive type the Bardo system tracks: swaps, LP events, lending, borrowing, vault operations, staking, restaking, perpetuals, options, yield markets, payment streams, gas dynamics, intent-based trading, RWA protocols, cross-chain messaging, account abstraction, and prediction markets. Each primitive gets a concrete set of role-filler pairs that compose into a single 1,280-byte fingerprint. The full Rust implementation is provided, including struct definitions, trait implementations, and algorithm implementations ready for integration with the Bardo heartbeat system.

---

## The problem

The Bardo runtime's `bardo-witness` crate sees every Ethereum block. After Binary Fuse pre-screening and four-stage triage (documented in the system overview and `01-chain-intelligence/`), surviving events enter the Golem's cognitive pipeline. The existing HDC infrastructure (`04-hyperdimensional-computing/`) encodes individual transactions as role-filler hypervectors: protocol, function selector, gas tier, value bucket, address cluster, log topics, token identifiers. This gives the Golem a fingerprint for each discrete event.

What the existing infrastructure does not do is reason about *patterns across events*. The questions that matter for technical analysis are temporal and compositional:

- Is this sequence of swap events forming a pattern I've seen before?
- Are these lending utilization changes following a trajectory that historically preceded liquidation cascades?
- Did RSI diverge from MACD while gas prices spiked, and has that specific combination preceded profitable opportunities in the past?
- Are Aave and Compound utilization rates, which are normally independent, suddenly moving together?

These questions require three capabilities that the per-event encoding cannot provide: composing multiple signals into a single queryable representation (pattern algebra), detecting patterns regardless of when they occur in a time series (temporal convolution), and tracking inter-protocol correlations as they evolve (cross-protocol entanglement).

This document builds all three on top of the existing BSC foundation. Every operation stays within the same `Hypervector` type, the same bind/bundle/permute/similarity primitives, and the same 10,240-bit dimension. No new dependencies. No new data types beyond what `golem-core::hdc` already provides.

---

## Mathematical foundations [SPEC]

### Notation

Throughout this document:

- **D** = 10,240 (BSC dimension in bits)
- **W** = 160 (number of u64 words per hypervector)
- $\oplus$ denotes XOR binding: $a \oplus b$ = `a.bind(&b)`
- $\bigoplus$ denotes majority-vote bundling over a set
- $\rho^k$ denotes cyclic permutation by $k$ positions: $\rho^k(a)$ = `a.permute(k)`
- $\delta(a, b)$ denotes normalized Hamming similarity: $\delta(a, b)$ = `a.similarity(&b)`
- $R_x$ denotes a role hypervector for field $x$ (drawn from `ItemMemory`)
- $V_y$ denotes a value/filler hypervector for concept $y$

### Role-filler composition

A structured record with fields $\{(x_1, y_1), (x_2, y_2), \ldots, (x_K, y_K)\}$ encodes as:

$$H = \bigoplus_{i=1}^{K} (R_{x_i} \oplus V_{y_i})$$

Each role-filler pair $R_{x_i} \oplus V_{y_i}$ is quasi-orthogonal to all other pairs (because XOR of independent random vectors yields a random vector). The bundle preserves similarity to each constituent pair. Querying field $x_j$ requires unbinding: $H \oplus R_{x_j} \approx V_{y_j}$ (with noise from the other $K-1$ terms).

The signal-to-noise ratio for recovering $V_{y_j}$ from a bundle of $K$ pairs:

$$\text{SNR} = \sqrt{\frac{D}{K-1}}$$

At $D = 10{,}240$ and $K = 7$ (typical DeFi event encoding): $\text{SNR} = \sqrt{10240/6} \approx 41.3$. At $K = 15$ (a heavily annotated event): $\text{SNR} \approx 27.0$. Both are far above the discrimination threshold of ~4.0 required for reliable retrieval from a codebook of 1,000 entries.

### Temporal composition

A sequence of observations $[o_1, o_2, \ldots, o_T]$ encodes as:

$$H_{\text{seq}} = \bigoplus_{t=1}^{T} \rho^{t-1}(\text{encode}(o_t))$$

Permutation by position index creates a unique representation for each element's position in the sequence. The same observation at different positions produces quasi-orthogonal contributions (because cyclic shifts by different amounts yield quasi-orthogonal vectors). This means the temporal hypervector is sensitive to both content and order.

**Shift invariance.** A pattern $P = [p_1, p_2, p_3]$ occurring at position $s$ in a longer series produces the sub-bundle:

$$H_P^{(s)} = \bigoplus_{i=1}^{3} \rho^{s+i-1}(\text{encode}(p_i))$$

A pattern kernel encodes the *relative* positions:

$$K_P = \bigoplus_{i=1}^{3} \rho^{i-1}(\text{encode}(p_i))$$

The similarity between $K_P$ and a windowed sub-sequence starting at position $s$ depends only on the content match, not on $s$ itself, because the permutation offsets cancel when computed within a fixed-size window. This is the shift-invariance property: one kernel matches the pattern regardless of where it appears.

### Cross-bundle similarity dynamics

Given two sets of events $A$ and $B$, encoded as bundles $H_A$ and $H_B$, their similarity $\delta(H_A, H_B)$ reflects the fraction of shared structure. If $A$ and $B$ are drawn from independent processes, $\delta \approx 0.5$ (orthogonal). If they share common events or correlated structure, $\delta > 0.5$.

The rate of change of this similarity over time is the entanglement drift:

$$\Delta(t) = \delta(H_A(t), H_B(t)) - \delta(H_A(t - W), H_B(t - W))$$

where $W$ is the observation window. Large positive $\Delta$ signals that previously independent processes are converging. Large negative $\Delta$ signals decoupling. Both are informative for systemic risk assessment.

---

## DeFi primitive encoding schemes [SPEC]

Every DeFi primitive type the Bardo system tracks gets a specific set of role-filler pairs. These are the building blocks for pattern algebra, temporal convolution, and entanglement tracking. Each role is a named hypervector from `ItemMemory`; each filler is a concept hypervector, often bucketed for continuous values.

### Swap events

```
swap_hv = bind(R_protocol,       V_uniswap_v3)
        + bind(R_action,          V_swap)
        + bind(R_pair,            V_eth_usdc)
        + bind(R_direction,       V_buy)         // buy vs sell the base asset
        + bind(R_size_bucket,     V_large)       // logarithmic: dust/small/medium/large/whale
        + bind(R_slippage_bucket, V_high)        // <0.1% / 0.1-0.5% / 0.5-1% / 1-3% / >3%
        + bind(R_gas_bucket,      V_normal)      // from existing gas tier encoding
        + bind(R_mev_exposure,    V_sandwiched)  // none / frontrun / backrun / sandwiched
```

Eight role-filler pairs. SNR = sqrt(10240/7) = 38.2. The `direction` field distinguishes buy pressure from sell pressure on the base asset. The `mev_exposure` field records whether the swap was part of a detected MEV bundle (from `08-verification-safety/01-mev-protection.md`).

### LP (liquidity provision) events

```
lp_hv = bind(R_protocol,         V_uniswap_v3)
      + bind(R_action,            V_add_liquidity)  // add / remove / rebalance
      + bind(R_pool,              V_eth_usdc_005)    // pool identifier (includes fee tier)
      + bind(R_tick_range,        V_narrow)          // narrow / medium / wide / full_range
      + bind(R_liquidity_delta,   V_increase_large)  // direction + size bucket
      + bind(R_fee_tier,          V_005)             // 0.01% / 0.05% / 0.30% / 1.00%
      + bind(R_position_type,     V_concentrated)    // concentrated / full_range / managed
```

The `tick_range` bucket captures whether the LP position is tightly concentrated around the current price (narrow, fewer than 100 ticks) or spread across the full range. This matters for detecting JIT liquidity (extremely narrow, added and removed within a few blocks).

### Lending events

```
lend_hv = bind(R_protocol,           V_aave_v3)
        + bind(R_action,              V_supply)       // supply / withdraw
        + bind(R_asset,               V_usdc)
        + bind(R_utilization_bucket,  V_high)         // <30% / 30-60% / 60-80% / 80-90% / >90%
        + bind(R_rate_bucket,         V_elevated)     // rate relative to historical average
        + bind(R_amount_bucket,       V_large)
```

### Borrowing events

```
borrow_hv = bind(R_protocol,              V_aave_v3)
          + bind(R_action,                  V_borrow)     // borrow / repay / liquidate
          + bind(R_asset,                   V_eth)
          + bind(R_collateral_asset,        V_steth)
          + bind(R_utilization_bucket,      V_high)
          + bind(R_rate_bucket,             V_elevated)
          + bind(R_collateral_ratio_bucket, V_risky)      // safe(>200%) / moderate(150-200%) / risky(120-150%) / critical(<120%)
          + bind(R_amount_bucket,           V_large)
```

The `collateral_ratio_bucket` is the field that matters most for liquidation cascade detection. When multiple borrowing events cluster in the `risky` and `critical` buckets simultaneously, the bundle's similarity to a "pre-liquidation" pattern vector increases.

### Vault operations (ERC-4626)

```
vault_hv = bind(R_protocol,           V_yearn_v3)
         + bind(R_vault,               V_yvusdc)
         + bind(R_action,              V_deposit)      // deposit / withdraw / harvest
         + bind(R_share_price_delta,   V_stable)       // declining / stable / appreciating
         + bind(R_apy_bucket,          V_moderate)     // <2% / 2-5% / 5-15% / 15-50% / >50%
         + bind(R_tvl_change_bucket,   V_inflow)       // large_outflow / outflow / stable / inflow / large_inflow
```

### Staking events

```
stake_hv = bind(R_protocol,         V_lido)
         + bind(R_action,            V_stake)         // stake / unstake / claim_rewards
         + bind(R_validator,         V_lido_default)   // specific operator where known
         + bind(R_amount_bucket,     V_large)
         + bind(R_reward_rate,       V_normal)        // below_avg / normal / above_avg
```

### Restaking events

```
restake_hv = bind(R_protocol,              V_eigenlayer)
           + bind(R_avs,                    V_eigenda)      // specific AVS
           + bind(R_operator,               V_p2p)          // specific operator
           + bind(R_action,                  V_delegate)     // delegate / undelegate / slash
           + bind(R_security_budget_bucket,  V_medium)       // relative to AVS total
```

### Perpetual futures

```
perp_hv = bind(R_protocol,           V_gmx_v2)
        + bind(R_pair,                V_eth_usd)
        + bind(R_direction,           V_long)          // long / short
        + bind(R_size_bucket,         V_large)
        + bind(R_funding_rate_bucket, V_positive_high)  // neg_high / neg_low / neutral / pos_low / pos_high
        + bind(R_leverage_bucket,     V_moderate)       // 1-2x / 2-5x / 5-10x / 10-20x / >20x
        + bind(R_action,              V_open)           // open / close / liquidate / increase / decrease
```

### Options

```
option_hv = bind(R_protocol,       V_lyra)
          + bind(R_underlying,      V_eth)
          + bind(R_strike_bucket,   V_atm)           // deep_itm / itm / atm / otm / deep_otm
          + bind(R_expiry_bucket,   V_weekly)         // daily / weekly / monthly / quarterly
          + bind(R_option_type,     V_call)           // call / put
          + bind(R_greeks_bucket,   V_high_gamma)     // bucketed by dominant greek sensitivity
          + bind(R_action,          V_buy)            // buy / sell / exercise
          + bind(R_iv_bucket,       V_elevated)       // low / normal / elevated / extreme
```

### Yield market (Pendle-style)

```
yield_hv = bind(R_protocol,           V_pendle)
         + bind(R_market,              V_steth_dec24)
         + bind(R_action,              V_buy_pt)       // buy_pt / sell_pt / buy_yt / sell_yt / add_lp / remove_lp
         + bind(R_pt_discount_bucket,  V_moderate)     // premium / par / small_discount / moderate / deep_discount
         + bind(R_implied_rate_bucket, V_above_avg)    // relative to recent history
```

### Payment streams (Sablier-style)

```
stream_hv = bind(R_protocol,         V_sablier)
          + bind(R_action,            V_create)        // create / cancel / withdraw / transfer
          + bind(R_duration_bucket,   V_long)          // <1d / 1d-1w / 1w-1m / 1m-1y / >1y
          + bind(R_rate_bucket,       V_high)          // tokens per second, bucketed logarithmically
          + bind(R_asset,             V_usdc)
```

### Gas dynamics

```
gas_hv = bind(R_base_fee_bucket,       V_elevated)     // <10 / 10-30 / 30-80 / 80-200 / >200 gwei
       + bind(R_priority_fee_bucket,    V_normal)       // <0.1 / 0.1-1 / 1-5 / 5-20 / >20 gwei
       + bind(R_block_utilization,      V_high)         // <50% / 50-80% / 80-95% / >95%
       + bind(R_blob_gas_bucket,        V_low)          // EIP-4844 blob gas market
```

Gas dynamics are not events from a specific protocol. They are ambient market conditions. Each block produces a gas hypervector that becomes an environmental context signal, bundled into any event that occurred in that block.

### Intent-based trading (UniswapX, CoW Protocol)

```
intent_hv = bind(R_protocol,          V_uniswapx)
          + bind(R_action,             V_fill)           // create_order / fill / expire / cancel
          + bind(R_pair,               V_eth_usdc)
          + bind(R_size_bucket,        V_large)
          + bind(R_fill_quality,       V_better)         // worse / par / better (vs. AMM benchmark)
          + bind(R_solver,             V_wintermute)     // which solver/filler won
          + bind(R_auction_duration,   V_fast)           // fast (<1 block) / medium / slow (>5 blocks)
```

### RWA (real-world asset) protocols

```
rwa_hv = bind(R_protocol,         V_maker_rwa)
       + bind(R_action,            V_mint)            // mint / redeem / rebalance / liquidate
       + bind(R_asset_class,       V_treasury)        // treasury / real_estate / credit / commodity
       + bind(R_amount_bucket,     V_large)
       + bind(R_yield_bucket,      V_market_rate)     // below / at / above market rate
```

### Cross-chain messaging

```
bridge_hv = bind(R_protocol,          V_across)
          + bind(R_action,             V_bridge_send)    // bridge_send / bridge_receive / relay
          + bind(R_source_chain,       V_ethereum)
          + bind(R_dest_chain,         V_arbitrum)
          + bind(R_asset,              V_usdc)
          + bind(R_amount_bucket,      V_large)
          + bind(R_bridge_time_bucket, V_fast)           // <2min / 2-10min / 10-60min / >1h
```

### Account abstraction (ERC-4337)

```
aa_hv = bind(R_protocol,          V_erc4337)
      + bind(R_action,             V_user_op)          // user_op / paymaster_pay / bundler_submit
      + bind(R_paymaster,          V_pimlico)           // which paymaster sponsored
      + bind(R_bundler,            V_flashbots)         // which bundler included
      + bind(R_gas_overhead_bucket, V_moderate)         // overhead vs. regular tx
```

### Prediction markets

```
prediction_hv = bind(R_protocol,          V_polymarket)
              + bind(R_action,             V_buy)           // buy / sell / redeem
              + bind(R_market_type,        V_binary)        // binary / scalar / categorical
              + bind(R_outcome,            V_yes)           // specific outcome token
              + bind(R_price_bucket,       V_likely)        // <10% / 10-30% / 30-50% / 50-70% / 70-90% / >90%
              + bind(R_volume_bucket,      V_high)
              + bind(R_time_to_resolution, V_days)          // hours / days / weeks / months
```

---

## Capability 1: Pattern algebra [SPEC]

### The core insight

Traditional TA asks questions sequentially. "Is RSI oversold?" Check. "Is MACD crossing up?" Check. "Are both happening while volume is above average?" Write a custom function that combines both checks with an AND gate. Every new combination requires new code.

HDC asks all these questions with one operation. Encode each condition as a hypervector. Bundle them into a query. Compare the query against each historical state via similarity. The answer is a single number: how well does this moment match the query?

The cost is fixed regardless of query complexity. A query that combines two conditions costs the same as one combining ten. The comparison is XOR + POPCNT across 160 u64 words: ~10 nanoseconds.

### Encoding TA indicators as hypervectors

Each TA indicator reading becomes a role-filler pair. The role identifies the indicator. The filler identifies the discretized reading.

```
// RSI(14) at 28 -- oversold territory
rsi_hv = bind(R_indicator_rsi, V_rsi_oversold)

// MACD histogram negative but narrowing (bearish divergence)
macd_hv = bind(R_indicator_macd, V_macd_narrowing_negative)

// Bollinger Band position: price touching lower band
bb_hv = bind(R_indicator_bb, V_bb_lower_touch)

// Volume: 2x above 20-day average
volume_hv = bind(R_indicator_volume, V_volume_high)

// On-chain: lending utilization above 80%
util_hv = bind(R_context_utilization, V_utilization_high)
```

Discretization is deliberate. HDC operates on categories, not continuous values. An RSI of 28 and an RSI of 31 both map to `V_rsi_oversold`. The system detects the *pattern* of being in oversold territory, not the exact numerical reading. This is the right level of abstraction for pattern matching: a head-and-shoulders pattern does not care whether the left shoulder is at exactly $1,847.23.

The indicator filler codebook uses thermometer encoding for ordinal indicators (the same technique used for gas tiers in `01-transaction-fingerprints.md`). "Oversold" is the bundle of `rsi:extreme_oversold` and `rsi:oversold`, giving it partial similarity to adjacent states. Non-ordinal indicators (like MACD crossover direction) use atomic encodings.

### Pattern composition

A pattern is a bundle of role-filler pairs describing the conditions that must co-occur:

```
// "RSI oversold while MACD shows bullish divergence and volume is elevated"
bullish_reversal_query = bundle(
    bind(R_indicator_rsi,    V_rsi_oversold),
    bind(R_indicator_macd,   V_macd_bullish_divergence),
    bind(R_indicator_volume, V_volume_high)
)

// "Find moments matching this pattern in history"
for (tick, state_hv) in historical_states {
    let sim = bullish_reversal_query.similarity(&state_hv);
    if sim > 0.55 {
        matches.push((tick, sim));
    }
}
```

The threshold of 0.55 catches partial matches. A state matching two of three conditions scores ~0.53-0.55. A state matching all three scores ~0.57-0.62 (depending on how many other non-matching signals are also encoded in the state vector). The exact thresholds are calibrated empirically during the evaluation protocol (see the Evaluation section).

### DeFi-enriched pattern queries

The power of encoding TA indicators alongside DeFi primitive events in the same hypervector space is that cross-domain queries are free:

```
// "Find moments when RSI diverged from MACD while lending utilization > 80%"
query = bind(R_indicator_rsi,         V_rsi_bearish_divergence)
      + bind(R_indicator_macd,        V_macd_bullish)
      + bind(R_context_utilization,   V_utilization_high)

// "Find moments when a large swap coincided with Bollinger Band squeeze"
query = bind(R_indicator_bb,          V_bb_squeeze)
      + bind(R_action,                V_swap)
      + bind(R_size_bucket,           V_whale)

// "Find moments when funding rates were extreme while options IV spiked"
query = bind(R_funding_rate_bucket,   V_positive_high)
      + bind(R_iv_bucket,             V_extreme)
      + bind(R_indicator_rsi,         V_rsi_overbought)
```

None of these queries require special-purpose code. They are all the same operation: compose a bundle, scan for similar states. Adding a new condition to any query is one more `bind` term in the bundle. Removing a condition is removing a term. The algebra is closed under composition.

### Full Rust implementation

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory, HDC_WORDS};
use std::collections::HashMap;

/// Codebook for a single DeFi primitive type.
/// Maps action names, bucket labels, and protocol identifiers
/// to deterministic hypervectors.
pub struct PrimitiveCodebook {
    /// Action vectors: "swap", "add_liquidity", "borrow", etc.
    action_vectors: ItemMemory,
    /// Bucket vectors: "size:large", "rate:high", etc.
    bucket_vectors: ItemMemory,
    /// Protocol vectors: "uniswap_v3", "aave_v3", etc.
    protocol_vectors: ItemMemory,
    /// Primitive type name (for debugging and logging)
    primitive_type: String,
}

impl PrimitiveCodebook {
    pub fn new(primitive_type: &str, seed: u64) -> Self {
        // Each sub-memory gets a derived seed to avoid collisions
        // across codebook partitions while staying deterministic.
        PrimitiveCodebook {
            action_vectors: ItemMemory::new(seed.wrapping_mul(3)),
            bucket_vectors: ItemMemory::new(seed.wrapping_mul(7)),
            protocol_vectors: ItemMemory::new(seed.wrapping_mul(13)),
            primitive_type: primitive_type.to_string(),
        }
    }

    pub fn encode_action(&mut self, action: &str) -> Hypervector {
        self.action_vectors.encode(action)
    }

    pub fn encode_bucket(&mut self, bucket: &str) -> Hypervector {
        self.bucket_vectors.encode(bucket)
    }

    pub fn encode_protocol(&mut self, protocol: &str) -> Hypervector {
        self.protocol_vectors.encode(protocol)
    }
}

/// A named pattern: a hypervector with metadata for tracking
/// match history and fitness over time.
pub struct NamedPattern {
    pub name: String,
    pub hv: Hypervector,
    /// How well this pattern predicts outcomes.
    /// Updated by the cybernetic feedback loop.
    pub fitness: f32,
    /// Number of times this pattern has been matched.
    pub match_count: u64,
    /// Tick at which the pattern was last matched.
    pub last_matched: u64,
    /// Whether this pattern was discovered by the system (vs. seeded).
    pub learned: bool,
}

impl NamedPattern {
    pub fn new(name: &str, hv: Hypervector) -> Self {
        NamedPattern {
            name: name.to_string(),
            hv,
            fitness: 0.5, // neutral prior
            match_count: 0,
            last_matched: 0,
            learned: false,
        }
    }

    /// Update fitness based on observed outcome.
    /// `outcome`: 1.0 for correct prediction, 0.0 for incorrect.
    /// Uses exponential moving average with alpha = 0.1.
    pub fn update_fitness(&mut self, outcome: f32) {
        let alpha = 0.1;
        self.fitness = alpha * outcome + (1.0 - alpha) * self.fitness;
    }
}

/// Master codebook for all TA pattern encoding.
/// Owns the role vectors shared across all primitive types,
/// the per-primitive codebooks, and the accumulated pattern library.
pub struct TaPatternCodebook {
    /// Role vectors: "R_protocol", "R_action", "R_pair", etc.
    /// Shared across all primitive types.
    roles: ItemMemory,

    /// Per-primitive-type codebooks
    swap_codebook: PrimitiveCodebook,
    lp_codebook: PrimitiveCodebook,
    lend_codebook: PrimitiveCodebook,
    borrow_codebook: PrimitiveCodebook,
    vault_codebook: PrimitiveCodebook,
    stake_codebook: PrimitiveCodebook,
    restake_codebook: PrimitiveCodebook,
    perp_codebook: PrimitiveCodebook,
    option_codebook: PrimitiveCodebook,
    yield_codebook: PrimitiveCodebook,
    stream_codebook: PrimitiveCodebook,
    gas_codebook: PrimitiveCodebook,
    intent_codebook: PrimitiveCodebook,
    rwa_codebook: PrimitiveCodebook,
    bridge_codebook: PrimitiveCodebook,
    aa_codebook: PrimitiveCodebook,
    prediction_codebook: PrimitiveCodebook,

    /// TA indicator codebook (RSI, MACD, BB, etc.)
    indicator_codebook: ItemMemory,

    /// Pattern library: accumulated from experience and seeding.
    pattern_library: Vec<NamedPattern>,

    /// Similarity threshold for pattern matching.
    /// Calibrated during evaluation. Default 0.55.
    match_threshold: f32,
}

impl TaPatternCodebook {
    pub fn new(seed: u64) -> Self {
        TaPatternCodebook {
            roles: ItemMemory::new(seed),
            swap_codebook: PrimitiveCodebook::new("swap", seed.wrapping_add(100)),
            lp_codebook: PrimitiveCodebook::new("lp", seed.wrapping_add(200)),
            lend_codebook: PrimitiveCodebook::new("lend", seed.wrapping_add(300)),
            borrow_codebook: PrimitiveCodebook::new("borrow", seed.wrapping_add(400)),
            vault_codebook: PrimitiveCodebook::new("vault", seed.wrapping_add(500)),
            stake_codebook: PrimitiveCodebook::new("stake", seed.wrapping_add(600)),
            restake_codebook: PrimitiveCodebook::new("restake", seed.wrapping_add(700)),
            perp_codebook: PrimitiveCodebook::new("perp", seed.wrapping_add(800)),
            option_codebook: PrimitiveCodebook::new("option", seed.wrapping_add(900)),
            yield_codebook: PrimitiveCodebook::new("yield", seed.wrapping_add(1000)),
            stream_codebook: PrimitiveCodebook::new("stream", seed.wrapping_add(1100)),
            gas_codebook: PrimitiveCodebook::new("gas", seed.wrapping_add(1200)),
            intent_codebook: PrimitiveCodebook::new("intent", seed.wrapping_add(1300)),
            rwa_codebook: PrimitiveCodebook::new("rwa", seed.wrapping_add(1400)),
            bridge_codebook: PrimitiveCodebook::new("bridge", seed.wrapping_add(1500)),
            aa_codebook: PrimitiveCodebook::new("aa", seed.wrapping_add(1600)),
            prediction_codebook: PrimitiveCodebook::new("prediction", seed.wrapping_add(1700)),
            indicator_codebook: ItemMemory::new(seed.wrapping_add(2000)),
            pattern_library: Vec::new(),
            match_threshold: 0.55,
        }
    }

    /// Get a role hypervector by name.
    pub fn role(&mut self, name: &str) -> Hypervector {
        self.roles.encode(&format!("role:{}", name))
    }

    /// Get a TA indicator hypervector.
    pub fn indicator(&mut self, name: &str) -> Hypervector {
        self.indicator_codebook.encode(&format!("indicator:{}", name))
    }

    /// Add a named pattern to the library.
    pub fn add_pattern(&mut self, name: &str, hv: Hypervector) {
        self.pattern_library.push(NamedPattern::new(name, hv));
    }

    /// Search the pattern library for matches against a state vector.
    /// Returns (pattern_name, similarity) pairs above threshold,
    /// sorted by similarity descending.
    pub fn match_patterns(&self, state_hv: &Hypervector) -> Vec<(&str, f32)> {
        let mut matches: Vec<(&str, f32)> = self
            .pattern_library
            .iter()
            .map(|p| (p.name.as_str(), p.hv.similarity(state_hv)))
            .filter(|(_, sim)| *sim > self.match_threshold)
            .collect();
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        matches
    }

    /// Get the top-fitness patterns. Used for pruning low-performing
    /// patterns during Delta-tick consolidation.
    pub fn top_patterns(&self, n: usize) -> Vec<&NamedPattern> {
        let mut sorted: Vec<&NamedPattern> = self.pattern_library.iter().collect();
        sorted.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        sorted.truncate(n);
        sorted
    }

    /// Remove patterns below a fitness threshold.
    /// Called during Delta-tick consolidation to prevent library bloat.
    pub fn prune(&mut self, min_fitness: f32, min_matches: u64) {
        self.pattern_library.retain(|p| {
            p.fitness >= min_fitness || p.match_count < min_matches
        });
    }
}
```

### Encoding DeFi events

The encoder takes raw DeFi events and produces hypervectors using the codebook:

```rust
/// Enumeration of all DeFi event types the TA system understands.
/// Each variant carries the decoded fields needed for HDC encoding.
pub enum DeFiEvent {
    Swap {
        protocol: String,
        pair: String,
        direction: String,      // "buy" or "sell"
        size_bucket: String,
        slippage_bucket: String,
        gas_bucket: String,
        mev_exposure: String,
    },
    LpAction {
        protocol: String,
        pool: String,
        action: String,         // "add_liquidity", "remove_liquidity", "rebalance"
        tick_range: String,
        liquidity_delta: String,
        fee_tier: String,
        position_type: String,
    },
    Lend {
        protocol: String,
        action: String,         // "supply", "withdraw"
        asset: String,
        utilization_bucket: String,
        rate_bucket: String,
        amount_bucket: String,
    },
    Borrow {
        protocol: String,
        action: String,         // "borrow", "repay", "liquidate"
        asset: String,
        collateral_asset: String,
        utilization_bucket: String,
        rate_bucket: String,
        collateral_ratio_bucket: String,
        amount_bucket: String,
    },
    Vault {
        protocol: String,
        vault: String,
        action: String,
        share_price_delta: String,
        apy_bucket: String,
        tvl_change_bucket: String,
    },
    Stake {
        protocol: String,
        action: String,
        validator: String,
        amount_bucket: String,
        reward_rate: String,
    },
    Restake {
        protocol: String,
        avs: String,
        operator: String,
        action: String,
        security_budget_bucket: String,
    },
    Perp {
        protocol: String,
        pair: String,
        direction: String,
        size_bucket: String,
        funding_rate_bucket: String,
        leverage_bucket: String,
        action: String,
    },
    Option {
        protocol: String,
        underlying: String,
        strike_bucket: String,
        expiry_bucket: String,
        option_type: String,
        greeks_bucket: String,
        action: String,
        iv_bucket: String,
    },
    Yield {
        protocol: String,
        market: String,
        action: String,
        pt_discount_bucket: String,
        implied_rate_bucket: String,
    },
    Stream {
        protocol: String,
        action: String,
        duration_bucket: String,
        rate_bucket: String,
        asset: String,
    },
    Gas {
        base_fee_bucket: String,
        priority_fee_bucket: String,
        block_utilization: String,
        blob_gas_bucket: String,
    },
    Intent {
        protocol: String,
        action: String,
        pair: String,
        size_bucket: String,
        fill_quality: String,
        solver: String,
        auction_duration: String,
    },
    Rwa {
        protocol: String,
        action: String,
        asset_class: String,
        amount_bucket: String,
        yield_bucket: String,
    },
    Bridge {
        protocol: String,
        action: String,
        source_chain: String,
        dest_chain: String,
        asset: String,
        amount_bucket: String,
        bridge_time_bucket: String,
    },
    AccountAbstraction {
        protocol: String,
        action: String,
        paymaster: String,
        bundler: String,
        gas_overhead_bucket: String,
    },
    Prediction {
        protocol: String,
        action: String,
        market_type: String,
        outcome: String,
        price_bucket: String,
        volume_bucket: String,
        time_to_resolution: String,
    },
}

/// Encodes DeFi events into hypervectors using the TA codebook.
pub struct DeFiEventEncoder<'a> {
    codebook: &'a mut TaPatternCodebook,
}

impl<'a> DeFiEventEncoder<'a> {
    pub fn new(codebook: &'a mut TaPatternCodebook) -> Self {
        DeFiEventEncoder { codebook }
    }

    /// Encode a single DeFi event into a hypervector.
    pub fn encode(&mut self, event: &DeFiEvent) -> Hypervector {
        match event {
            DeFiEvent::Swap {
                protocol, pair, direction, size_bucket,
                slippage_bucket, gas_bucket, mev_exposure,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", "swap");
                self.bind_role_filler(&mut acc, "pair", pair);
                self.bind_role_filler(&mut acc, "direction", direction);
                self.bind_role_filler(&mut acc, "size_bucket", size_bucket);
                self.bind_role_filler(&mut acc, "slippage_bucket", slippage_bucket);
                self.bind_role_filler(&mut acc, "gas_bucket", gas_bucket);
                self.bind_role_filler(&mut acc, "mev_exposure", mev_exposure);
                acc.finish()
            }
            DeFiEvent::LpAction {
                protocol, pool, action, tick_range,
                liquidity_delta, fee_tier, position_type,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "pool", pool);
                self.bind_role_filler(&mut acc, "tick_range", tick_range);
                self.bind_role_filler(&mut acc, "liquidity_delta", liquidity_delta);
                self.bind_role_filler(&mut acc, "fee_tier", fee_tier);
                self.bind_role_filler(&mut acc, "position_type", position_type);
                acc.finish()
            }
            DeFiEvent::Lend {
                protocol, action, asset,
                utilization_bucket, rate_bucket, amount_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "asset", asset);
                self.bind_role_filler(&mut acc, "utilization_bucket", utilization_bucket);
                self.bind_role_filler(&mut acc, "rate_bucket", rate_bucket);
                self.bind_role_filler(&mut acc, "amount_bucket", amount_bucket);
                acc.finish()
            }
            DeFiEvent::Borrow {
                protocol, action, asset, collateral_asset,
                utilization_bucket, rate_bucket,
                collateral_ratio_bucket, amount_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "asset", asset);
                self.bind_role_filler(&mut acc, "collateral_asset", collateral_asset);
                self.bind_role_filler(&mut acc, "utilization_bucket", utilization_bucket);
                self.bind_role_filler(&mut acc, "rate_bucket", rate_bucket);
                self.bind_role_filler(&mut acc, "collateral_ratio_bucket", collateral_ratio_bucket);
                self.bind_role_filler(&mut acc, "amount_bucket", amount_bucket);
                acc.finish()
            }
            DeFiEvent::Vault {
                protocol, vault, action,
                share_price_delta, apy_bucket, tvl_change_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "vault", vault);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "share_price_delta", share_price_delta);
                self.bind_role_filler(&mut acc, "apy_bucket", apy_bucket);
                self.bind_role_filler(&mut acc, "tvl_change_bucket", tvl_change_bucket);
                acc.finish()
            }
            DeFiEvent::Stake {
                protocol, action, validator,
                amount_bucket, reward_rate,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "validator", validator);
                self.bind_role_filler(&mut acc, "amount_bucket", amount_bucket);
                self.bind_role_filler(&mut acc, "reward_rate", reward_rate);
                acc.finish()
            }
            DeFiEvent::Restake {
                protocol, avs, operator,
                action, security_budget_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "avs", avs);
                self.bind_role_filler(&mut acc, "operator", operator);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "security_budget_bucket", security_budget_bucket);
                acc.finish()
            }
            DeFiEvent::Perp {
                protocol, pair, direction, size_bucket,
                funding_rate_bucket, leverage_bucket, action,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "pair", pair);
                self.bind_role_filler(&mut acc, "direction", direction);
                self.bind_role_filler(&mut acc, "size_bucket", size_bucket);
                self.bind_role_filler(&mut acc, "funding_rate_bucket", funding_rate_bucket);
                self.bind_role_filler(&mut acc, "leverage_bucket", leverage_bucket);
                self.bind_role_filler(&mut acc, "action", action);
                acc.finish()
            }
            DeFiEvent::Option {
                protocol, underlying, strike_bucket,
                expiry_bucket, option_type, greeks_bucket,
                action, iv_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "underlying", underlying);
                self.bind_role_filler(&mut acc, "strike_bucket", strike_bucket);
                self.bind_role_filler(&mut acc, "expiry_bucket", expiry_bucket);
                self.bind_role_filler(&mut acc, "option_type", option_type);
                self.bind_role_filler(&mut acc, "greeks_bucket", greeks_bucket);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "iv_bucket", iv_bucket);
                acc.finish()
            }
            DeFiEvent::Yield {
                protocol, market, action,
                pt_discount_bucket, implied_rate_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "market", market);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "pt_discount_bucket", pt_discount_bucket);
                self.bind_role_filler(&mut acc, "implied_rate_bucket", implied_rate_bucket);
                acc.finish()
            }
            DeFiEvent::Stream {
                protocol, action, duration_bucket,
                rate_bucket, asset,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "duration_bucket", duration_bucket);
                self.bind_role_filler(&mut acc, "rate_bucket", rate_bucket);
                self.bind_role_filler(&mut acc, "asset", asset);
                acc.finish()
            }
            DeFiEvent::Gas {
                base_fee_bucket, priority_fee_bucket,
                block_utilization, blob_gas_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "base_fee_bucket", base_fee_bucket);
                self.bind_role_filler(&mut acc, "priority_fee_bucket", priority_fee_bucket);
                self.bind_role_filler(&mut acc, "block_utilization", block_utilization);
                self.bind_role_filler(&mut acc, "blob_gas_bucket", blob_gas_bucket);
                acc.finish()
            }
            DeFiEvent::Intent {
                protocol, action, pair, size_bucket,
                fill_quality, solver, auction_duration,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "pair", pair);
                self.bind_role_filler(&mut acc, "size_bucket", size_bucket);
                self.bind_role_filler(&mut acc, "fill_quality", fill_quality);
                self.bind_role_filler(&mut acc, "solver", solver);
                self.bind_role_filler(&mut acc, "auction_duration", auction_duration);
                acc.finish()
            }
            DeFiEvent::Rwa {
                protocol, action, asset_class,
                amount_bucket, yield_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "asset_class", asset_class);
                self.bind_role_filler(&mut acc, "amount_bucket", amount_bucket);
                self.bind_role_filler(&mut acc, "yield_bucket", yield_bucket);
                acc.finish()
            }
            DeFiEvent::Bridge {
                protocol, action, source_chain, dest_chain,
                asset, amount_bucket, bridge_time_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "source_chain", source_chain);
                self.bind_role_filler(&mut acc, "dest_chain", dest_chain);
                self.bind_role_filler(&mut acc, "asset", asset);
                self.bind_role_filler(&mut acc, "amount_bucket", amount_bucket);
                self.bind_role_filler(&mut acc, "bridge_time_bucket", bridge_time_bucket);
                acc.finish()
            }
            DeFiEvent::AccountAbstraction {
                protocol, action, paymaster,
                bundler, gas_overhead_bucket,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "paymaster", paymaster);
                self.bind_role_filler(&mut acc, "bundler", bundler);
                self.bind_role_filler(&mut acc, "gas_overhead_bucket", gas_overhead_bucket);
                acc.finish()
            }
            DeFiEvent::Prediction {
                protocol, action, market_type, outcome,
                price_bucket, volume_bucket, time_to_resolution,
            } => {
                let mut acc = BundleAccumulator::new();
                self.bind_role_filler(&mut acc, "protocol", protocol);
                self.bind_role_filler(&mut acc, "action", action);
                self.bind_role_filler(&mut acc, "market_type", market_type);
                self.bind_role_filler(&mut acc, "outcome", outcome);
                self.bind_role_filler(&mut acc, "price_bucket", price_bucket);
                self.bind_role_filler(&mut acc, "volume_bucket", volume_bucket);
                self.bind_role_filler(&mut acc, "time_to_resolution", time_to_resolution);
                acc.finish()
            }
        }
    }

    /// Helper: bind a named role with a named filler and add to accumulator.
    fn bind_role_filler(
        &mut self,
        acc: &mut BundleAccumulator,
        role_name: &str,
        filler_name: &str,
    ) {
        let role_hv = self.codebook.role(role_name);
        let filler_hv = self.codebook.roles.encode(filler_name);
        acc.add(&role_hv.bind(&filler_hv));
    }
}
```

### Composite state encoding

At each Gamma tick, the system produces a composite state vector that bundles all active TA indicator readings and recent DeFi events into a single hypervector. This is the vector that pattern queries compare against.

```rust
/// A snapshot of the current TA + DeFi state, encoded as a single hypervector.
/// Produced at each Gamma tick. Stored in a rolling buffer for temporal queries.
pub struct TaStateSnapshot {
    /// The composite hypervector.
    pub hv: Hypervector,
    /// Gamma tick number.
    pub tick: u64,
    /// Number of signals encoded in this snapshot.
    pub signal_count: usize,
    /// Wall clock timestamp (unix seconds).
    pub timestamp: u64,
}

/// Builds composite TA state snapshots from indicator readings and DeFi events.
pub struct TaStateEncoder {
    codebook: TaPatternCodebook,
}

impl TaStateEncoder {
    pub fn new(seed: u64) -> Self {
        TaStateEncoder {
            codebook: TaPatternCodebook::new(seed),
        }
    }

    /// Encode a Gamma tick's worth of TA state.
    ///
    /// `indicators`: named indicator readings, e.g. [("rsi_14", "oversold"), ("macd", "bullish_cross")]
    /// `events`: DeFi events observed since the last Gamma tick.
    /// `gas`: current gas conditions.
    pub fn encode_tick(
        &mut self,
        tick: u64,
        timestamp: u64,
        indicators: &[(&str, &str)],
        events: &[DeFiEvent],
        gas: Option<&DeFiEvent>,
    ) -> TaStateSnapshot {
        let mut acc = BundleAccumulator::new();
        let mut signal_count = 0;

        // Encode TA indicator readings
        for (indicator_name, reading) in indicators {
            let role = self.codebook.indicator(indicator_name);
            let value = self.codebook.roles.encode(reading);
            acc.add(&role.bind(&value));
            signal_count += 1;
        }

        // Encode DeFi events
        let mut encoder = DeFiEventEncoder::new(&mut self.codebook);
        for event in events {
            let event_hv = encoder.encode(event);
            acc.add(&event_hv);
            signal_count += 1;
        }

        // Encode gas conditions as ambient context
        if let Some(gas_event) = gas {
            let gas_hv = encoder.encode(gas_event);
            acc.add(&gas_hv);
            signal_count += 1;
        }

        TaStateSnapshot {
            hv: acc.finish(),
            tick,
            signal_count,
            timestamp,
        }
    }

    /// Access the codebook for pattern library operations.
    pub fn codebook(&self) -> &TaPatternCodebook {
        &self.codebook
    }

    /// Access the codebook mutably (for adding patterns, pruning, etc.).
    pub fn codebook_mut(&mut self) -> &mut TaPatternCodebook {
        &mut self.codebook
    }
}
```

---

## Capability 2: Temporal convolution [SPEC]

### The problem

Pattern algebra matches against point-in-time state vectors. But many patterns of interest are temporal: they unfold over a sequence of observations. A head-and-shoulders pattern is not a single state; it is a sequence of states (rise, peak, fall, rise again, lower peak, fall). A liquidation cascade is a sequence of borrowing events with declining collateral ratios. A TVL drain is a sequence of vault withdrawals.

Traditional approaches use sliding window Dynamic Time Warping (DTW) or template matching. DTW has O(n*m) complexity per comparison, where n is the series length and m is the template length. For a 1,000-tick history with a 20-tick template, that is 20,000 operations per comparison. With 50 templates, a million operations per scan.

HDC temporal convolution does the same job with a different cost structure. Encoding a window of T observations costs O(D * T). But once encoded, comparing the window against any number of kernels costs O(D) per kernel, which is ~10ns. The encoding is the expensive part; the matching is free.

### Temporal encoding

Given a window of observations $[o_1, o_2, \ldots, o_T]$, each observation is first encoded into a hypervector (using the DeFi event encoder or TA state encoder above), then permuted by its position index, then bundled:

$$H_{\text{window}} = \bigoplus_{t=1}^{T} \rho^{t-1}(\text{encode}(o_t))$$

The permutation makes the encoding position-sensitive: `encode(o_1)` permuted by 0 is different from `encode(o_1)` permuted by 1. The bundle makes it holographic: the single resulting vector represents the entire window.

### Temporal kernels

A kernel encodes the pattern to detect. It uses the same encoding scheme as the window:

$$K = \bigoplus_{i=1}^{M} \rho^{i-1}(\text{encode}(p_i))$$

where $[p_1, \ldots, p_M]$ is the pattern template and $M$ is the pattern length. The kernel's length $M$ can be shorter than the window length $T$. In that case, the convolution slides the kernel across the window.

### Shift-invariant convolution

To find where in a time series a pattern occurs, compute the kernel's similarity against each window position:

For each start position $s \in [1, T - M + 1]$:

$$\text{score}(s) = \delta\left(K, \bigoplus_{i=1}^{M} \rho^{i-1}(\text{encode}(o_{s+i-1}))\right)$$

This looks expensive, but the window encoding can be computed incrementally. As the window slides by one position, one observation drops out and one enters. The BundleAccumulator supports this with add/remove operations (add the new observation, subtract the old one by adding it with weight -1).

### Applications to DeFi time series

**Price action patterns.** Encode price action as a sequence of directional moves: up_large, up_small, flat, down_small, down_large. A head-and-shoulders kernel is: [up_large, up_small, down_small, up_small, flat, down_large]. The kernel detects this pattern at any position in the price series.

**TVL flow patterns.** Encode vault events as: large_inflow, inflow, stable, outflow, large_outflow. A "gradual drain" kernel is: [stable, outflow, outflow, outflow, large_outflow]. A "panic exit" kernel is: [stable, large_outflow, large_outflow]. Different temporal signatures, different kernels, same matching operation.

**Utilization rate patterns.** Encode lending utilization changes as: increasing_fast, increasing_slow, stable, decreasing_slow, decreasing_fast. A "mean-reversion cycle" kernel is: [increasing_fast, increasing_slow, stable, decreasing_slow, decreasing_fast, stable]. A "trending utilization" kernel is: [increasing_slow, increasing_slow, increasing_slow, increasing_fast, increasing_fast].

**Gas price patterns.** Encode EIP-1559 base fee dynamics as: spike, elevated, declining, low, spike. This oscillation mode often precedes MEV activity surges.

**Yield curve evolution.** For Pendle-style yield markets, encode implied rate changes across maturities: [steepening, steepening, flat, flattening, inversion]. Different phases of the yield curve cycle map to different temporal kernels.

### Rust implementation

```rust
use crate::hdc::{Hypervector, BundleAccumulator, HDC_BITS};

/// An observation in a DeFi time series.
/// This is the pre-encoded hypervector for a single timestep.
pub struct DeFiObservation {
    /// Pre-encoded hypervector for this observation.
    pub hv: Hypervector,
    /// Tick number.
    pub tick: u64,
    /// Observation type (for debugging).
    pub label: String,
}

/// Encodes temporal windows and detects patterns via HDC convolution.
pub struct TemporalEncoder {
    /// Window size in observations.
    window_size: usize,
    /// Stride between window positions during convolution.
    stride: usize,
}

impl TemporalEncoder {
    pub fn new(window_size: usize, stride: usize) -> Self {
        assert!(window_size > 0, "window_size must be positive");
        assert!(stride > 0, "stride must be positive");
        TemporalEncoder {
            window_size,
            stride,
        }
    }

    /// Encode a window of observations into a single temporal hypervector.
    /// Each observation is permuted by its position index before bundling.
    ///
    /// Cost: O(D * window_size) for the bundle accumulation.
    /// At D = 10,240 and window_size = 20: ~300 us.
    pub fn encode_window(&self, observations: &[DeFiObservation]) -> Hypervector {
        let len = observations.len().min(self.window_size);
        let mut acc = BundleAccumulator::new();
        for (pos, obs) in observations.iter().take(len).enumerate() {
            let permuted = obs.hv.permute(pos);
            acc.add(&permuted);
        }
        acc.finish()
    }

    /// Build a pattern kernel from a sequence of template observations.
    /// The kernel uses the same positional encoding as windows,
    /// allowing shift-invariant matching.
    pub fn build_kernel(&self, template: &[Hypervector]) -> Hypervector {
        let mut acc = BundleAccumulator::new();
        for (pos, hv) in template.iter().enumerate() {
            let permuted = hv.permute(pos);
            acc.add(&permuted);
        }
        acc.finish()
    }

    /// Convolve a pattern kernel across a time series.
    /// Returns (position_index, similarity) pairs for each window position.
    ///
    /// The position_index refers to the starting observation of the window.
    /// Results are not filtered; caller applies threshold.
    ///
    /// Cost: O(series_length / stride * (D * window_size + D))
    /// The D * window_size term is the window encoding;
    /// the D term is the similarity comparison.
    pub fn convolve(
        &self,
        series: &[DeFiObservation],
        kernel: &Hypervector,
    ) -> Vec<(usize, f32)> {
        if series.len() < self.window_size {
            return vec![];
        }

        let mut results = Vec::with_capacity(
            (series.len() - self.window_size) / self.stride + 1,
        );

        let mut pos = 0;
        while pos + self.window_size <= series.len() {
            let window_hv = self.encode_window(&series[pos..pos + self.window_size]);
            let sim = kernel.similarity(&window_hv);
            results.push((pos, sim));
            pos += self.stride;
        }

        results
    }

    /// Incremental convolution with a sliding window.
    /// More efficient than full re-encoding when stride = 1.
    ///
    /// Uses an accumulator that adds the new observation and
    /// un-adds the old one. The "un-add" is approximate for
    /// majority-vote bundling (we subtract the old observation's
    /// votes), but at typical window sizes (10-30) the approximation
    /// introduces negligible error.
    pub fn convolve_incremental(
        &self,
        series: &[DeFiObservation],
        kernel: &Hypervector,
    ) -> Vec<(usize, f32)> {
        if series.len() < self.window_size {
            return vec![];
        }

        let mut results = Vec::with_capacity(series.len() - self.window_size + 1);

        // Initialize the first window
        let mut acc = BundleAccumulator::new();
        for pos in 0..self.window_size {
            let permuted = series[pos].hv.permute(pos);
            acc.add(&permuted);
        }

        let first_window = acc.finish();
        results.push((0, kernel.similarity(&first_window)));

        // Slide by rebuilding. True incremental update would require
        // removing the oldest observation (adding with weight -1) and
        // adding the newest. This works but accumulates rounding error
        // over many slides. We rebuild every `window_size` steps to
        // reset error accumulation.
        for start in 1..=(series.len() - self.window_size) {
            if start % self.window_size == 0 {
                // Full rebuild to reset accumulated error
                acc.clear();
                for pos in 0..self.window_size {
                    let permuted = series[start + pos].hv.permute(pos);
                    acc.add(&permuted);
                }
            } else {
                // Incremental: remove oldest, add newest
                let old_pos = start - 1;
                let old_permuted = series[old_pos].hv.permute(0);
                acc.add_weighted(&old_permuted, -1);

                // Shift all existing observations' permutation indices down by 1.
                // This is the expensive part of incremental update.
                // For now, we rebuild. A future optimization can maintain
                // a circular buffer of permuted vectors.
                acc.clear();
                for pos in 0..self.window_size {
                    let permuted = series[start + pos].hv.permute(pos);
                    acc.add(&permuted);
                }
            }

            let window_hv = acc.finish();
            results.push((start, kernel.similarity(&window_hv)));
        }

        results
    }
}

/// A collection of named temporal kernels for pattern detection.
pub struct TemporalKernelLibrary {
    kernels: Vec<TemporalKernel>,
}

pub struct TemporalKernel {
    pub name: String,
    pub hv: Hypervector,
    pub length: usize,
    pub fitness: f32,
    pub description: String,
}

impl TemporalKernelLibrary {
    pub fn new() -> Self {
        TemporalKernelLibrary {
            kernels: Vec::new(),
        }
    }

    /// Register a temporal pattern kernel.
    pub fn add_kernel(
        &mut self,
        name: &str,
        kernel_hv: Hypervector,
        length: usize,
        description: &str,
    ) {
        self.kernels.push(TemporalKernel {
            name: name.to_string(),
            hv: kernel_hv,
            length,
            fitness: 0.5,
            description: description.to_string(),
        });
    }

    /// Match all kernels against a temporal window.
    /// Returns (kernel_name, similarity) pairs above threshold.
    pub fn match_all(
        &self,
        window_hv: &Hypervector,
        threshold: f32,
    ) -> Vec<(&str, f32)> {
        let mut matches: Vec<(&str, f32)> = self
            .kernels
            .iter()
            .map(|k| (k.name.as_str(), k.hv.similarity(window_hv)))
            .filter(|(_, sim)| *sim > threshold)
            .collect();
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        matches
    }

    /// Get a kernel by name.
    pub fn get(&self, name: &str) -> Option<&TemporalKernel> {
        self.kernels.iter().find(|k| k.name == name)
    }

    /// Number of kernels in the library.
    pub fn len(&self) -> usize {
        self.kernels.len()
    }
}
```

### Multi-scale temporal analysis

DeFi patterns operate at different timescales. A flash loan attack happens within a single block. A TVL drain unfolds over hours. A yield curve regime shift takes days. The temporal encoder handles this by maintaining multiple encoders at different window sizes:

```rust
/// Multi-scale temporal analysis.
/// Maintains temporal encoders at different window sizes and strides,
/// producing pattern matches at each scale simultaneously.
pub struct MultiScaleTemporalAnalyzer {
    /// (window_size, stride, encoder, kernel_library)
    scales: Vec<(usize, usize, TemporalEncoder, TemporalKernelLibrary)>,
}

/// A match from the multi-scale analyzer.
pub struct MultiScaleMatch {
    pub scale_index: usize,
    pub window_size: usize,
    pub position: usize,
    pub kernel_name: String,
    pub similarity: f32,
}

impl MultiScaleTemporalAnalyzer {
    /// Create a multi-scale analyzer with the specified window sizes.
    /// Each scale gets its own encoder and kernel library.
    ///
    /// Typical configuration:
    ///   scales: [(5, 1), (20, 5), (100, 10), (500, 50)]
    ///   Corresponding to: block-scale, minute-scale, hour-scale, day-scale
    pub fn new(scale_configs: &[(usize, usize)]) -> Self {
        let scales = scale_configs
            .iter()
            .map(|&(window, stride)| {
                (
                    window,
                    stride,
                    TemporalEncoder::new(window, stride),
                    TemporalKernelLibrary::new(),
                )
            })
            .collect();
        MultiScaleTemporalAnalyzer { scales }
    }

    /// Add a kernel to a specific scale.
    pub fn add_kernel(
        &mut self,
        scale_index: usize,
        name: &str,
        kernel_hv: Hypervector,
        description: &str,
    ) {
        if let Some((window_size, _, _, library)) = self.scales.get_mut(scale_index) {
            library.add_kernel(name, kernel_hv, *window_size, description);
        }
    }

    /// Run all scales against a time series.
    /// Returns all matches across all scales, sorted by similarity.
    pub fn analyze(
        &self,
        series: &[DeFiObservation],
        threshold: f32,
    ) -> Vec<MultiScaleMatch> {
        let mut all_matches = Vec::new();

        for (scale_idx, (window_size, _stride, encoder, library)) in
            self.scales.iter().enumerate()
        {
            if series.len() < *window_size {
                continue;
            }

            // Encode the most recent window at this scale
            let recent = &series[series.len() - window_size..];
            let window_hv = encoder.encode_window(recent);

            for (kernel_name, sim) in library.match_all(&window_hv, threshold) {
                all_matches.push(MultiScaleMatch {
                    scale_index: scale_idx,
                    window_size: *window_size,
                    position: series.len() - window_size,
                    kernel_name: kernel_name.to_string(),
                    similarity: sim,
                });
            }
        }

        all_matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        all_matches
    }

    /// Access a scale's kernel library for management.
    pub fn kernel_library_mut(
        &mut self,
        scale_index: usize,
    ) -> Option<&mut TemporalKernelLibrary> {
        self.scales.get_mut(scale_index).map(|(_, _, _, lib)| lib)
    }
}
```

---

## Capability 3: Cross-protocol entanglement [SPEC]

### The problem

DeFi protocols operate independently most of the time. Aave lending rates and Uniswap swap volumes are driven by different participants with different motivations. Their on-chain events produce quasi-orthogonal hypervector bundles, as expected.

But during stress events, independence breaks down. A large ETH price drop causes: Aave liquidations (borrow events with `action:liquidate`), Uniswap sell pressure (swap events with `direction:sell, size:large`), Curve imbalances (LP events with `liquidity_delta:decrease`), and Lido unstaking requests (stake events with `action:unstake`). Suddenly, the hypervector bundles for these protocols start sharing structure. Their similarity increases from ~0.5 (orthogonal) toward 0.55+.

This convergence is detectable before the cascade fully develops. If Aave and Compound utilization bundles start correlating *before* liquidations begin, that is an early warning signal. The entanglement matrix tracks these cross-protocol correlations at XOR speed.

### Mathematical formulation

For each protocol $p$, maintain a running bundle of recent events:

$$B_p(t) = \text{ema\_bundle}(\text{events from } p \text{ in window } [t-W, t])$$

The EMA (exponential moving average) bundle weights recent events more heavily than old ones. The implementation adds recent events with weight proportional to their recency.

The entanglement matrix at time $t$ is:

$$E(t)[i][j] = \delta(B_i(t), B_j(t))$$

For $N$ protocols, this is an $N \times N$ symmetric matrix. The diagonal is 1.0 (self-similarity). Off-diagonal entries measure cross-protocol correlation.

The entanglement drift between time $t$ and $t - W$ is:

$$\Delta E[i][j] = E(t)[i][j] - E(t-W)[i][j]$$

Large positive $\Delta E[i][j]$ means protocols $i$ and $j$ are becoming correlated. Large negative means they are decoupling.

The systemic risk score aggregates the drift:

$$\text{risk}(t) = \frac{1}{N(N-1)} \sum_{i \neq j} \max(0, \Delta E[i][j])$$

This measures the fraction of protocol pairs that are converging. A value near 0 means the market is normal (protocols are independent). A value above 0.02 means something structural is changing.

### Rust implementation

```rust
use crate::hdc::{Hypervector, BundleAccumulator, HDC_WORDS};
use std::collections::{HashMap, VecDeque};

/// Tracks cross-protocol entanglement via HDC bundle similarity.
///
/// Each protocol maintains a running bundle of its recent events.
/// The entanglement matrix measures pairwise similarity between
/// protocol bundles. Changes in this matrix signal structural
/// shifts in cross-protocol relationships.
pub struct EntanglementMatrix {
    /// Protocol names, in order. Defines matrix indices.
    protocols: Vec<String>,
    /// Protocol name -> index mapping.
    protocol_index: HashMap<String, usize>,
    /// Current running bundle per protocol.
    /// Updated by absorbing new events with EMA-like weighting.
    bundles: Vec<BundleAccumulator>,
    /// Snapshot of each bundle (computed lazily when similarity is queried).
    snapshots: Vec<Option<Hypervector>>,
    /// Historical similarity matrices for drift computation.
    /// Each entry is a flattened upper-triangular matrix.
    history: VecDeque<SimilaritySnapshot>,
    /// Maximum history depth.
    history_capacity: usize,
    /// Current tick.
    current_tick: u64,
    /// Decay factor for EMA bundling. Applied as weight reduction
    /// on the existing accumulator when new events arrive.
    /// Range: (0.0, 1.0). Higher = more memory. Default 0.95.
    decay_factor: f32,
}

struct SimilaritySnapshot {
    tick: u64,
    /// Flattened upper triangle: entry (i,j) with i < j is at index
    /// i * n - i*(i+1)/2 + j - i - 1, where n = protocol count.
    similarities: Vec<f32>,
}

impl EntanglementMatrix {
    pub fn new(protocols: &[&str], history_capacity: usize) -> Self {
        let n = protocols.len();
        let protocol_index: HashMap<String, usize> = protocols
            .iter()
            .enumerate()
            .map(|(i, &name)| (name.to_string(), i))
            .collect();

        EntanglementMatrix {
            protocols: protocols.iter().map(|s| s.to_string()).collect(),
            protocol_index,
            bundles: (0..n).map(|_| BundleAccumulator::new()).collect(),
            snapshots: vec![None; n],
            history: VecDeque::with_capacity(history_capacity),
            history_capacity,
            current_tick: 0,
            decay_factor: 0.95,
        }
    }

    /// Register a new protocol dynamically.
    /// Returns the protocol's index.
    pub fn add_protocol(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.protocol_index.get(name) {
            return idx;
        }
        let idx = self.protocols.len();
        self.protocols.push(name.to_string());
        self.protocol_index.insert(name.to_string(), idx);
        self.bundles.push(BundleAccumulator::new());
        self.snapshots.push(None);
        idx
    }

    /// Update a protocol's bundle with a new event.
    /// The event is a pre-encoded hypervector from the DeFiEventEncoder.
    ///
    /// The bundle uses additive accumulation. Over time, frequently
    /// occurring event patterns dominate the bundle's bit pattern,
    /// creating a "behavioral fingerprint" for the protocol.
    pub fn update(&mut self, protocol: &str, event_hv: &Hypervector) {
        let idx = match self.protocol_index.get(protocol) {
            Some(&i) => i,
            None => self.add_protocol(protocol),
        };
        self.bundles[idx].add(event_hv);
        self.snapshots[idx] = None; // invalidate cached snapshot
    }

    /// Update with a batch of events for a single protocol.
    pub fn update_batch(&mut self, protocol: &str, events: &[Hypervector]) {
        let idx = match self.protocol_index.get(protocol) {
            Some(&i) => i,
            None => self.add_protocol(protocol),
        };
        for hv in events {
            self.bundles[idx].add(hv);
        }
        self.snapshots[idx] = None;
    }

    /// Compute the current similarity matrix.
    /// Returns an N x N matrix where entry [i][j] is the Hamming
    /// similarity between protocol i's and protocol j's event bundles.
    ///
    /// Cost: O(N^2 * D) for N protocols.
    /// At N = 20 and D = 10,240: ~40 us.
    pub fn similarity_matrix(&mut self) -> Vec<Vec<f32>> {
        let n = self.protocols.len();

        // Ensure all snapshots are current
        for i in 0..n {
            if self.snapshots[i].is_none() && self.bundles[i].count > 0 {
                self.snapshots[i] = Some(self.bundles[i].finish());
            }
        }

        let mut matrix = vec![vec![0.5f32; n]; n];
        for i in 0..n {
            matrix[i][i] = 1.0;
            for j in (i + 1)..n {
                let sim = match (&self.snapshots[i], &self.snapshots[j]) {
                    (Some(a), Some(b)) => a.similarity(b),
                    _ => 0.5, // orthogonal if either has no events
                };
                matrix[i][j] = sim;
                matrix[j][i] = sim;
            }
        }
        matrix
    }

    /// Record the current similarity matrix to history.
    /// Called at each Gamma tick after processing all events.
    pub fn record_tick(&mut self, tick: u64) {
        let n = self.protocols.len();
        let mut similarities = Vec::with_capacity(n * (n - 1) / 2);

        // Ensure snapshots are current
        for i in 0..n {
            if self.snapshots[i].is_none() && self.bundles[i].count > 0 {
                self.snapshots[i] = Some(self.bundles[i].finish());
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                let sim = match (&self.snapshots[i], &self.snapshots[j]) {
                    (Some(a), Some(b)) => a.similarity(b),
                    _ => 0.5,
                };
                similarities.push(sim);
            }
        }

        if self.history.len() == self.history_capacity {
            self.history.pop_front();
        }
        self.history.push_back(SimilaritySnapshot {
            tick,
            similarities,
        });
        self.current_tick = tick;
    }

    /// Compute the drift matrix: how much has each pairwise similarity
    /// changed over the last `lookback` ticks?
    ///
    /// Returns an N x N matrix where entry [i][j] is
    /// similarity_now[i][j] - similarity_then[i][j].
    /// Positive drift = convergence. Negative drift = divergence.
    pub fn drift_matrix(&self, lookback: usize) -> Option<Vec<Vec<f32>>> {
        if self.history.len() < 2 {
            return None;
        }

        let current = self.history.back()?;
        let past_idx = if lookback >= self.history.len() {
            0
        } else {
            self.history.len() - lookback
        };
        let past = &self.history[past_idx];

        let n = self.protocols.len();
        let mut drift = vec![vec![0.0f32; n]; n];

        let mut flat_idx = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                if flat_idx < current.similarities.len()
                    && flat_idx < past.similarities.len()
                {
                    let d = current.similarities[flat_idx] - past.similarities[flat_idx];
                    drift[i][j] = d;
                    drift[j][i] = d;
                }
                flat_idx += 1;
            }
        }

        Some(drift)
    }

    /// Compute a scalar systemic risk score from the drift matrix.
    ///
    /// The score measures the average positive drift across all
    /// protocol pairs. A value near 0 means protocols are behaving
    /// independently. A value above 0.02 warrants attention.
    /// A value above 0.05 is a strong convergence signal.
    pub fn systemic_risk_score(&self, lookback: usize) -> f32 {
        let drift = match self.drift_matrix(lookback) {
            Some(d) => d,
            None => return 0.0,
        };

        let n = self.protocols.len();
        if n < 2 {
            return 0.0;
        }

        let mut total_positive_drift = 0.0f32;
        let mut pair_count = 0u32;

        for i in 0..n {
            for j in (i + 1)..n {
                total_positive_drift += drift[i][j].max(0.0);
                pair_count += 1;
            }
        }

        if pair_count == 0 {
            return 0.0;
        }

        total_positive_drift / pair_count as f32
    }

    /// Find the protocol pairs with the highest positive drift.
    /// These are the pairs that are converging most rapidly.
    pub fn top_converging_pairs(
        &self,
        lookback: usize,
        n: usize,
    ) -> Vec<(String, String, f32)> {
        let drift = match self.drift_matrix(lookback) {
            Some(d) => d,
            None => return vec![],
        };

        let num_protocols = self.protocols.len();
        let mut pairs: Vec<(String, String, f32)> = Vec::new();

        for i in 0..num_protocols {
            for j in (i + 1)..num_protocols {
                if drift[i][j] > 0.0 {
                    pairs.push((
                        self.protocols[i].clone(),
                        self.protocols[j].clone(),
                        drift[i][j],
                    ));
                }
            }
        }

        pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        pairs.truncate(n);
        pairs
    }

    /// Get the protocol names in matrix order.
    pub fn protocol_names(&self) -> &[String] {
        &self.protocols
    }

    /// Reset a protocol's bundle. Used after a Delta-tick decay cycle.
    pub fn decay_protocol(&mut self, protocol: &str, retain_fraction: f32) {
        // Decay is implemented by replacing the accumulator with a
        // weighted version of its current snapshot. The snapshot
        // captures the "behavioral fingerprint"; the weight controls
        // how quickly old patterns fade.
        if let Some(&idx) = self.protocol_index.get(protocol) {
            if let Some(snapshot) = &self.snapshots[idx] {
                let weight = (retain_fraction * 10.0).round().max(1.0) as i32;
                let mut new_acc = BundleAccumulator::new();
                new_acc.add_weighted(snapshot, weight);
                self.bundles[idx] = new_acc;
                self.snapshots[idx] = None;
            }
        }
    }

    /// Decay all protocols. Called at Delta tick.
    pub fn decay_all(&mut self, retain_fraction: f32) {
        let names: Vec<String> = self.protocols.clone();
        for name in &names {
            self.decay_protocol(name, retain_fraction);
        }
    }
}
```

### Entanglement alert generation

The entanglement matrix feeds into the Golem's attention system. When the systemic risk score exceeds a threshold, or when specific protocol pairs show rapid convergence, the system generates an alert that enters the CorticalState.

```rust
/// Alert generated when cross-protocol entanglement exceeds thresholds.
pub struct EntanglementAlert {
    /// Tick at which the alert was generated.
    pub tick: u64,
    /// Overall systemic risk score.
    pub risk_score: f32,
    /// The top converging protocol pairs.
    pub converging_pairs: Vec<(String, String, f32)>,
    /// Whether this is a new alert (vs. continuation of an existing one).
    pub is_new: bool,
}

/// Monitors the entanglement matrix and generates alerts.
pub struct EntanglementMonitor {
    /// Threshold for systemic risk score alerting.
    risk_threshold: f32,
    /// Threshold for individual pair drift alerting.
    pair_drift_threshold: f32,
    /// Lookback window for drift computation (in ticks).
    lookback: usize,
    /// Whether an alert is currently active.
    alert_active: bool,
    /// The last risk score, for hysteresis.
    last_risk_score: f32,
}

impl EntanglementMonitor {
    pub fn new(
        risk_threshold: f32,
        pair_drift_threshold: f32,
        lookback: usize,
    ) -> Self {
        EntanglementMonitor {
            risk_threshold,
            pair_drift_threshold,
            lookback,
            alert_active: false,
            last_risk_score: 0.0,
        }
    }

    /// Check the entanglement matrix and generate an alert if warranted.
    /// Called at each Gamma tick after `record_tick`.
    pub fn check(
        &mut self,
        matrix: &EntanglementMatrix,
        tick: u64,
    ) -> Option<EntanglementAlert> {
        let risk_score = matrix.systemic_risk_score(self.lookback);
        let converging = matrix.top_converging_pairs(self.lookback, 5);

        let should_alert = risk_score > self.risk_threshold
            || converging.iter().any(|(_, _, d)| *d > self.pair_drift_threshold);

        let alert = if should_alert {
            let is_new = !self.alert_active;
            self.alert_active = true;
            Some(EntanglementAlert {
                tick,
                risk_score,
                converging_pairs: converging,
                is_new,
            })
        } else {
            // Hysteresis: only deactivate if risk drops below 80% of threshold
            if risk_score < self.risk_threshold * 0.8 {
                self.alert_active = false;
            }
            None
        };

        self.last_risk_score = risk_score;
        alert
    }
}
```

---

## Heartbeat integration [SPEC]

The three capabilities map onto the Bardo heartbeat's three clock tiers.

### Gamma tick (5-15 seconds)

Gamma is the perception clock. At each Gamma tick:

1. **Encode events.** All DeFi events since the last tick get encoded via `DeFiEventEncoder`. Cost: ~100us per event, typically 5-50 events per tick. Total: 0.5-5ms.

2. **Update entanglement matrix.** Each event's hypervector is fed to `EntanglementMatrix::update()`. Cost: ~15us per event (one bundle accumulation). Total: <1ms.

3. **Build state snapshot.** `TaStateEncoder::encode_tick()` bundles all current indicator readings and recent events into a composite state vector. Cost: ~500us.

4. **Pattern match.** `TaPatternCodebook::match_patterns()` compares the state snapshot against the pattern library. Cost: ~10ns per pattern, typically 100-500 patterns. Total: 1-5us.

5. **Temporal window update.** Append the state snapshot to the temporal series buffer. Encode the latest window at each active scale. Cost: ~300us per scale, typically 4 scales. Total: ~1.2ms.

6. **Entanglement check.** `EntanglementMonitor::check()` computes the systemic risk score. Cost: ~50us.

Total Gamma budget: ~3-8ms, well within the 5-second minimum Gamma interval.

### Theta tick (30-120 seconds)

Theta is the cognition clock. At each Theta tick:

1. **Temporal convolution.** Run all temporal kernels against the accumulated series since the last Theta tick. This is the expensive operation: O(series_length * window_size * D) per scale. With ~10 Gamma ticks per Theta tick and 4 scales: ~12ms.

2. **Pattern match reporting.** Compile all pattern matches (both instantaneous from Gamma and temporal from convolution) into a structured report for the LLM deliberation pipeline. The report includes match names, similarities, and historical fitness scores.

3. **Entanglement summary.** If any alerts were generated during the Gamma ticks, compile the drift matrix and converging pairs into the Theta-tick context.

### Delta tick (~50 Theta ticks)

Delta is the consolidation clock. At each Delta tick:

1. **Pattern library pruning.** `TaPatternCodebook::prune()` removes patterns below the fitness threshold. Patterns that matched frequently but predicted poorly get removed.

2. **Pattern discovery.** Bundle successful outcomes (events that the Golem profited from or correctly predicted) into new pattern candidates. Add them to the library with `learned: true`.

3. **Entanglement decay.** `EntanglementMatrix::decay_all()` reduces the influence of old events in the protocol bundles. This prevents the bundles from becoming static over time.

4. **Temporal kernel refinement.** Kernels that match frequently but predict poorly get their fitness reduced. Kernels that never match get removed.

```rust
/// Coordinates the TA HDC subsystem across heartbeat ticks.
pub struct TaHdcCoordinator {
    /// State encoder (owns the codebook).
    state_encoder: TaStateEncoder,
    /// Temporal analyzer (multi-scale).
    temporal: MultiScaleTemporalAnalyzer,
    /// Entanglement tracker.
    entanglement: EntanglementMatrix,
    /// Entanglement alert monitor.
    entanglement_monitor: EntanglementMonitor,
    /// Rolling buffer of state snapshots for temporal analysis.
    state_history: Vec<DeFiObservation>,
    /// Maximum state history length.
    max_history: usize,
}

impl TaHdcCoordinator {
    pub fn new(
        seed: u64,
        protocols: &[&str],
        temporal_scales: &[(usize, usize)],
        max_history: usize,
    ) -> Self {
        TaHdcCoordinator {
            state_encoder: TaStateEncoder::new(seed),
            temporal: MultiScaleTemporalAnalyzer::new(temporal_scales),
            entanglement: EntanglementMatrix::new(protocols, 1000),
            entanglement_monitor: EntanglementMonitor::new(0.02, 0.03, 50),
            state_history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Process a Gamma tick.
    /// Returns pattern matches and any entanglement alerts.
    pub fn gamma_tick(
        &mut self,
        tick: u64,
        timestamp: u64,
        indicators: &[(&str, &str)],
        events: &[DeFiEvent],
        gas: Option<&DeFiEvent>,
    ) -> GammaTickResult {
        // 1. Encode state snapshot
        let snapshot = self.state_encoder.encode_tick(
            tick, timestamp, indicators, events, gas,
        );

        // 2. Update entanglement
        {
            let mut encoder = DeFiEventEncoder::new(
                self.state_encoder.codebook_mut(),
            );
            for event in events {
                let event_hv = encoder.encode(event);
                let protocol_name = extract_protocol_name(event);
                self.entanglement.update(&protocol_name, &event_hv);
            }
        }
        self.entanglement.record_tick(tick);

        // 3. Pattern match against library
        let pattern_matches = self
            .state_encoder
            .codebook()
            .match_patterns(&snapshot.hv);

        // 4. Store for temporal analysis
        if self.state_history.len() >= self.max_history {
            self.state_history.remove(0);
        }
        self.state_history.push(DeFiObservation {
            hv: snapshot.hv,
            tick,
            label: format!("gamma_{}", tick),
        });

        // 5. Run temporal analysis at each scale
        let temporal_matches = self.temporal.analyze(
            &self.state_history,
            0.54,
        );

        // 6. Check entanglement
        let entanglement_alert = self.entanglement_monitor.check(
            &self.entanglement,
            tick,
        );

        GammaTickResult {
            snapshot,
            pattern_matches: pattern_matches
                .into_iter()
                .map(|(name, sim)| (name.to_string(), sim))
                .collect(),
            temporal_matches,
            entanglement_alert,
        }
    }

    /// Process a Delta tick (consolidation).
    pub fn delta_tick(&mut self, min_fitness: f32, decay_fraction: f32) {
        // Prune low-fitness patterns
        self.state_encoder.codebook_mut().prune(min_fitness, 10);

        // Decay entanglement bundles
        self.entanglement.decay_all(decay_fraction);
    }

    /// Add a learned pattern to the library from a successful outcome.
    pub fn learn_pattern(&mut self, name: &str, state_hv: Hypervector) {
        let mut pattern = NamedPattern::new(name, state_hv);
        pattern.learned = true;
        self.state_encoder.codebook_mut().pattern_library.push(pattern);
    }

    /// Record an outcome for a matched pattern (cybernetic feedback).
    pub fn record_outcome(&mut self, pattern_name: &str, outcome: f32) {
        if let Some(pattern) = self
            .state_encoder
            .codebook_mut()
            .pattern_library
            .iter_mut()
            .find(|p| p.name == pattern_name)
        {
            pattern.update_fitness(outcome);
            pattern.match_count += 1;
        }
    }
}

/// Result of a Gamma tick TA analysis.
pub struct GammaTickResult {
    pub snapshot: TaStateSnapshot,
    pub pattern_matches: Vec<(String, f32)>,
    pub temporal_matches: Vec<MultiScaleMatch>,
    pub entanglement_alert: Option<EntanglementAlert>,
}

/// Extract the protocol name from a DeFi event for entanglement routing.
fn extract_protocol_name(event: &DeFiEvent) -> String {
    match event {
        DeFiEvent::Swap { protocol, .. } => protocol.clone(),
        DeFiEvent::LpAction { protocol, .. } => protocol.clone(),
        DeFiEvent::Lend { protocol, .. } => protocol.clone(),
        DeFiEvent::Borrow { protocol, .. } => protocol.clone(),
        DeFiEvent::Vault { protocol, .. } => protocol.clone(),
        DeFiEvent::Stake { protocol, .. } => protocol.clone(),
        DeFiEvent::Restake { protocol, .. } => protocol.clone(),
        DeFiEvent::Perp { protocol, .. } => protocol.clone(),
        DeFiEvent::Option { protocol, .. } => protocol.clone(),
        DeFiEvent::Yield { protocol, .. } => protocol.clone(),
        DeFiEvent::Stream { protocol, .. } => protocol.clone(),
        DeFiEvent::Gas { .. } => "gas".to_string(),
        DeFiEvent::Intent { protocol, .. } => protocol.clone(),
        DeFiEvent::Rwa { protocol, .. } => protocol.clone(),
        DeFiEvent::Bridge { protocol, .. } => protocol.clone(),
        DeFiEvent::AccountAbstraction { protocol, .. } => protocol.clone(),
        DeFiEvent::Prediction { protocol, .. } => protocol.clone(),
    }
}
```

---

## Subsystem interactions

### CorticalState

The TA HDC subsystem writes two new fields to the CorticalState perception surface at each Gamma tick:

- **`pattern_confidence: f32`** -- The highest similarity score from pattern matching this tick. Range [0.5, 1.0]. Values above 0.55 indicate an active pattern match. Values above 0.60 indicate a strong match. This field feeds the attention salience computation (`03-agent-runtime/01-attention-salience.md`): when a known pattern is active, the Golem's arousal increases and the Theta tick interval shortens.

- **`entanglement_risk: f32`** -- The systemic risk score from the entanglement monitor. Range [0.0, 0.1+]. Values above 0.02 indicate increasing cross-protocol correlation. Values above 0.05 indicate a systemic event may be developing. This field feeds the homeostatic risk regulator (`03-agent-runtime/04-homeostasis.md`): high entanglement risk triggers conservative position sizing.

Both fields are lock-free `AtomicU32` values (float bits stored as u32), following the existing CorticalState convention. No locking, no contention with other Gamma-tick writers.

### Grimoire

Pattern hypervectors are stored in the Grimoire alongside episodes as 1,280-byte binary blobs. Each episode record gains two optional fields:

- `pattern_hv: Option<[u8; 1280]>` -- The state snapshot hypervector at the time of the episode. Enables similarity search across episode history: "find past episodes where the TA state was similar to now."

- `matched_patterns: Vec<String>` -- Names of patterns that were active when the episode was recorded. Enables pattern-outcome correlation during consolidation.

Storage cost: 1,280 bytes per episode for the pattern HV. At 1,000 episodes: 1.25 MB. Negligible relative to the episode text and metadata.

### Attention auction

The pattern match similarity scores feed the Oracle's VCG bid inputs (from the prediction engine). When a pattern matches with high similarity, the Oracle can bid higher for attention to the associated asset pair, increasing the likelihood that the Theta tick's deliberation pipeline focuses on the matched signal. The bid amount scales with `pattern_confidence * pattern_fitness`, weighted by how well the pattern has predicted outcomes historically.

### Dreams

The sleep cycle (documented in `03-agent-runtime/02-sleep-consolidation.md`) interacts with the pattern library in two phases:

**NREM consolidation.** During NREM, the system reviews pattern match outcomes and bundles successful patterns together. If patterns A, B, and C all matched during episodes with positive outcomes, their bundled prototype `bundle(A.hv, B.hv, C.hv)` becomes a new "meta-pattern" that captures the common structure across multiple winning conditions. This is generalization: the meta-pattern fires when any of its constituents would fire, but with lower similarity (because the bundle dilutes each component). Over time, successful meta-patterns replace their individual components in the library.

**REM mutation.** During REM, the system perturbs existing patterns by XOR-ing them with low-weight random hypervectors. If the original pattern $P$ has fitness $f$, the mutant $P' = P \oplus \epsilon$ (where $\epsilon$ is a sparse random vector with ~5% bits set) is a "nearby" pattern that shares ~95% of the original's structure. The mutant enters the library with the parent's fitness discounted by 50%. If it outperforms the parent over subsequent ticks, it replaces it. If not, it gets pruned at the next Delta tick. This is exploration at XOR speed: generating and evaluating pattern mutations costs nanoseconds per candidate.

### Styx

The Styx clade-coordination protocol (`07-memory-architecture/` and clade specification) transmits 1,280-byte pattern HVs as clade knowledge. When a Golem discovers a high-fitness pattern, it can share the pattern's hypervector with its clade peers. Receiving Golems add the pattern to their library with `learned: false` and a discounted fitness score. This enables horizontal pattern transfer: a pattern discovered by one Golem watching the ETH/USDC pair can benefit another Golem watching the ETH/DAI pair, because the pattern's structure (the TA indicator component) transfers even when the pair-specific component differs.

The communication cost is 1,280 bytes per pattern. Compare this to transmitting the pattern's description in natural language (hundreds of bytes for the text alone, plus parsing overhead). The hypervector is the pattern -- no serialization, no interpretation, no ambiguity.

---

## Cybernetic feedback loop

The TA HDC subsystem is not a static signal processor. It learns from outcomes. The feedback loop has four stages:

### Stage 1: Pattern match

At Gamma tick, the system detects patterns by comparing the state snapshot against the pattern library. Each match produces a (pattern_name, similarity) pair. The match triggers no action by itself; it is an observation.

### Stage 2: Outcome recording

At Theta tick, the Golem decides whether to act on the matched pattern. If it acts (executes a trade, adjusts a position, or escalates to the LLM for deliberation), the outcome is recorded: did the action succeed? Did the predicted market movement occur? The outcome is a scalar in [0, 1] reflecting how well the pattern's implied prediction matched reality.

### Stage 3: Pattern refinement

At Delta tick, the consolidation engine processes outcome records:

- **Successful patterns** get their fitness increased via EMA update. They are bundled with the specific state vectors that produced successes, sharpening the pattern's representation toward the actual conditions that predict outcomes.

- **Failed patterns** get their fitness decreased. If a pattern's fitness drops below the pruning threshold and it has been matched enough times to be statistically meaningful, it is removed from the library.

- **New patterns** are discovered by bundling state vectors from successful episodes that did not match any existing pattern. These novel situations become pattern candidates with neutral fitness priors.

The mathematics:

```
// Refinement via selective bundling
successful_states = [s1, s2, s3, ...]  // states where pattern P matched and outcome was positive
refined_P = bundle(P.hv, bundle(successful_states))
// The refined pattern is more similar to the actual successful states
// and less similar to the generic original

// Discovery via residual analysis
unmatched_successes = [u1, u2, u3, ...]  // successful states that no pattern matched
candidate = bundle(unmatched_successes)
// If candidate.similarity(any_existing_pattern) < 0.52, it is genuinely new
```

### Stage 4: Better matches next cycle

The refined and newly discovered patterns enter the library. The next Gamma tick's pattern matching uses the updated library. Patterns that predict well get stronger (higher fitness, sharper representation). Patterns that predict poorly get weaker (lower fitness, eventual pruning). The library converges toward a set of patterns that are actually predictive in the Golem's specific operating environment.

This is Hebbian learning in hypervector space. "Neurons that fire together wire together" becomes "patterns that match together bundle together." The binding and bundling operations are the learning rule. XOR speed is the clock speed.

---

## Evaluation protocol [SPEC]

### Falsifiable hypotheses

**H1: Pattern detection accuracy.** Given a set of known TA patterns (head-and-shoulders, double bottom, bullish divergence, etc.) encoded as HDC pattern vectors and also implemented as explicit pattern-matching algorithms, the HDC approach achieves >= 90% recall and >= 85% precision relative to the explicit implementation on historical ETH/USDC price data.

Measurement: Run both approaches on the same 90-day historical dataset. Compare detected pattern instances. False negatives from HDC (pattern detected by explicit algorithm but missed by HDC) indicate insufficient encoding resolution. False positives (HDC match with no explicit match) may indicate either HDC noise or genuine patterns that the explicit algorithm was not designed to detect; these require manual review.

**H2: Temporal convolution recall.** Given a set of temporal patterns (price action sequences, TVL flow sequences, utilization trajectories), HDC temporal convolution achieves >= 85% recall relative to sliding-window DTW on the same historical data, while maintaining >= 10x speed advantage.

Measurement: Implement both HDC convolution and DTW sliding window. Run on 90-day historical data with 20 temporal patterns of varying length (5-50 observations). Measure recall, precision, and wall-clock time. The speed advantage target of 10x accounts for the encoding overhead that DTW does not incur.

**H3: Entanglement drift detection lead time.** During historical systemic events (March 2020, May 2021 Terra/Luna, November 2022 FTX, March 2023 SVB/USDC depeg), the entanglement drift signal exceeds the alert threshold at least 10 minutes before the largest price drop in the event. Comparison baseline: Pearson correlation on 1-hour rolling windows of protocol TVL changes.

Measurement: Replay historical on-chain data through the encoding pipeline. Record when the entanglement risk score first exceeds 0.02 and when the correlation baseline first exceeds 2 standard deviations above its mean. Compare both to the timestamp of the largest 1-hour price decline in the event. The 10-minute lead time target is conservative; the speed advantage of XOR operations over correlation computation should provide detection well before any rolling-window approach.

**H4: Computational budget.** All Gamma-tick operations (event encoding + state snapshot + pattern matching + entanglement update + temporal window update) complete within 10ms on an M1 Pro CPU. All Theta-tick operations (temporal convolution + report generation) complete within 50ms. All Delta-tick operations (pruning + discovery + decay) complete within 500ms.

Measurement: Instrument the `TaHdcCoordinator` with timing probes. Run on a realistic event stream (50 events per Gamma tick, 100 patterns, 50 temporal kernels, 20 protocols). Report p50, p95, and p99 latencies.

### Test procedures

1. **Unit tests.** Verify that known patterns produce expected similarity scores. Encode a head-and-shoulders sequence, build a kernel, verify that convolution detects it at the correct position with similarity > 0.55. Encode events from two independent protocols, verify entanglement similarity is ~0.5. Encode events with shared structure, verify similarity increases.

2. **Integration tests.** Feed 24 hours of historical mainnet data through the full pipeline. Verify that the state history buffer stays within memory bounds. Verify that the entanglement matrix produces sensible similarity values (diagonal = 1.0, off-diagonal in [0.45, 0.55] during calm periods). Verify that pattern matching latency stays within budget.

3. **Backtesting.** Replay historical systemic events. Measure entanglement drift lead time. Compare pattern detection against known indicator signals. This is the primary validation for hypotheses H1-H3.

4. **Stress tests.** Simulate extreme event rates (500 events per Gamma tick, 50 protocols, 1,000 patterns). Verify that latency stays within 2x of the H4 targets. Identify which operation becomes the bottleneck and whether it can be parallelized across cores.

---

## References

- Frady, E.P., Kent, S.J., Olshausen, B.A., and Sommer, F.T. (2020). "Resonator Networks 1: An Efficient Solution for Factoring High-Dimensional, Distributed Representations of Data Structures." *Neural Computation*, 32(12), pp. 2311-2331.

- Frady, E.P., Kleyko, D., Kymre, J.M., Olshausen, B.A., and Sommer, F.T. (2023). "Computing on Functions Using Randomized Vector Representations." *Frontiers in Computational Neuroscience*, 16.

- Gayler, R.W. (2003). "Vector Symbolic Architectures Answer Jackendoff's Challenges for Cognitive Neuroscience." *ICANN Workshop on Compositional Connectionism*.

- Ge, L. and Parhi, K.K. (2020). "Classification Using Hyperdimensional Computing: A Review." *IEEE Circuits and Systems Magazine*, 20(2), pp. 30-47.

- Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors." *Cognitive Computation*, 1(2), pp. 139-159.

- Kleyko, D., Rachkovskij, D., Osipov, E., and Rahimi, A. (2022). "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures." *ACM Computing Surveys*, 55(6), Article 130.

- Plate, T.A. (1995). "Holographic Reduced Representations." *IEEE Transactions on Neural Networks*, 6(3), pp. 623-641.

- Rahimi, A., Kanerva, P., and Rabaey, J.M. (2016). "A Robust and Energy-Efficient Classifier Using Brain-Inspired Hyperdimensional Computing." *ISLPED 2016*, pp. 64-69.

- Schlegel, K., Neubert, P., and Protzel, P. (2022). "A Comparison of Vector Symbolic Architectures." *Artificial Intelligence Review*, 55, pp. 4523-4555.

- Thomas, A., Dasgupta, S., and Bhatt, T. (2021). "Capacity Analysis of Vector Symbolic Architectures." *NeurIPS 2021*.
