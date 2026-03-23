# First Fifteen Minutes: $10 Golem Walkthrough [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-18
**Status:** Draft

---

> **Reader orientation:** This document is a minute-by-minute narrative walkthrough of what happens when a $10 Golem (a mortal autonomous DeFi agent) boots for the first time. It specifies the exact sequence of internal events across the first fifteen minutes: boot sequence, first observations, pattern formation, and intelligence building. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: the Heartbeat (recurring decision cycle), the Grimoire (persistent knowledge base), and the Spectre (visual creature representation). For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

This document is a minute-by-minute narrative of what happens when a $10 Golem boots for the first time. From the moment the user types SUMMON to the moment the full dashboard populates, fifteen minutes pass. In that time, the Golem goes from a blank slate to an agent that has classified the market regime, formed predictions, calibrated its risk baseline, and produced its first insights -- all without executing a single trade.

The narrative serves two purposes: (1) it specifies the exact sequence of internal events so that the engineering implementation matches the user experience design, and (2) it demonstrates that a $10 Golem produces real intelligence, not a loading screen with a countdown.

> **Cross-references:**
>
> - `../01-golem/02-heartbeat.md` — canonical Heartbeat FSM: tick structure, Gamma (5-15s perception) / Theta (30-120s decision) / Delta (5-30min maintenance) clocks
> - `../01-golem/06-creation.md` section S7 — small portfolio ($10-50) creation defaults: observation mode, conservative tier selection, reduced inference budget
> - `../01-golem/07-provisioning.md` — six-stage provisioning pipeline from config validation through VM boot and first heartbeat
> - `./07-onboarding.md` sections 9, 15, 16 — birth cinematic sequence, first contact interaction, and the first 15 minutes experience design
> - `./14-creature-system.md` — Spectre rendering: dot cloud body, eye system behavior during boot, and initial creature formation
> - `./19-cinematic-system.md` — cinematic triggers and sequences: the birth cutscene, first-observation transitions, and discovery animations
> - `../../tmp/research/witness-research/new/reconciling/09-user-experience-simplification.md` — math-to-metaphor translation layer: how raw financial data becomes human-readable narrative
> - `../../tmp/research/witness-research/new/reconciling/03-ta-integration.md` — TA pipeline: technical analysis signal processing, indicator computation, and regime classification

---

## 1. Minute 0-1: Boot Sequence

### What happens inside

The user typed SUMMON. The provisioning pipeline claimed a warm VM from the pool (sub-5 seconds for Bardo Compute) and injected `golem.toml`, `STRATEGY.md`, and wallet configuration. The Golem binary starts.

Boot sequence, in order:

1. **Config validation** -- `golem.toml` schema check. For a $10 market-observer, this validates the `micro` compute tier, `auto_trade = false`, and the observation-first parameters. Takes <100ms.

2. **Memory initialization** -- Fresh Golem, so no Styx Archive restore. Grimoire initialized empty. LanceDB and SQLite tables created. The Golem has no memories yet.

3. **Mortality initialization** -- Economic baseline computed from the $10 USDC balance. At `micro` tier ($0.035/hr), the Golem calculates ~11 days of runway. Epistemic fitness set to 1.0 (fresh). Stochastic seed generated: `keccak256(golemId + creationTick)`. Phase determination: `computeVitality()` returns high vitality, `determinePhase()` sets THRIVING.

4. **WitnessEngine connects** -- Establishes connection to chain RPC (Base mainnet). This happens during boot, before the first heartbeat. The connection is live by the time tick 0001 fires.

5. **ChainScope initializes** -- Reads seed addresses from STRATEGY.md. For the market-observer template, these are Uniswap V3 factory, Morpho Blue vault registry, and Aave V3 pool registry on Base. First Binary Fuse filter compiled from seed addresses. The filter determines which on-chain transactions are relevant.

6. **CorticalState zeroed** -- All fields initialized to defaults. `regime_tag`: unset. `arousal`: 0.0. `valence`: 0.0. `uncertainty`: 1.0 (maximum, the Golem knows nothing yet). The CorticalState will be written by the first Gamma tick.

7. **Daimon initialization** -- Personality baseline from disposition template. PAD (Pleasure-Arousal-Dominance) set to neutral. Mood state: calm curiosity (the default for a newborn Golem).

8. **Extension registration** -- All standard extensions register: golem-heartbeat, golem-lifespan, golem-daimon, golem-dream (disabled until 50-200 episodes), golem-context, golem-verifier, golem-reflector, golem-curator, golem-styx, golem-clade, golem-telemetry, golem-safety.

9. **Health probes pass** -- Startup probe succeeds. The Golem is ready.

10. **First heartbeat tick fires** -- Tick 0001. The clock starts.

Total boot time: 3-8 seconds, depending on RPC connection latency.

### What the user sees

The birth cinematic plays. Not skippable on first viewing (see `07-onboarding.md` section 9).

```
+==================================================================+
|                                                                    |
|                                                                    |
|                                                                    |
|                          · · ·                                     |
|                       ·  ◉ ◉  ·                                   |
|                        · · · ·                                     |
|                          · ·                                       |
|                                                                    |
|                     Awakening...                                   |
|                                                                    |
|   ♡ ─────────────── TICK 0001 ──────────────────────── $10.00    |
|                                                                    |
+==================================================================+
```

Five cinematic stages in 10-15 seconds:

1. **Void** (2s) -- Black screen. A single pulse of the ROSEDUST palette ripples outward from center. The heartbeat sound (if audio is enabled) plays once.

2. **Coalescing** (3s) -- Dots appear one by one, clustering into the Spectre's shape. Each dot fades in from `text_phantom` to `rose_dim`. The cluster breathes -- contracting and expanding in a slow sine wave.

3. **Eyes open** (2s) -- Two bright glyphs (`◉ ◉`) appear at the Spectre's face position. They blink once. The dot cloud brightens from `rose_dim` to `rose`.

4. **First breath** (3s) -- The dot cloud contracts fully, then expands. The heartbeat sine wave begins in the status bar. The `TICK 0001` counter appears.

5. **Chrome materializes** (3s) -- Tab bar fades in at top. Status bar solidifies at bottom. The NAV display ($10.00), runway estimate, and phase indicator appear. The Spectre settles into its sidebar position.

The user presses any key. The full TUI frame renders. But it is mostly empty -- only the Spectre, the heartbeat counter, and the status bar have data. No transaction feed yet. No predictions. No regime classification. Those come in the next two minutes as real data arrives.

The emptiness is intentional. Widgets do not render until they have real data. The screen fills in as the Golem's understanding fills in.

---

## 2. Minute 1-3: First Observations

### What happens inside

The heartbeat is ticking. Gamma ticks fire every 10-20 seconds (the market-observer template uses a slightly slower gamma than the default 5-15s, to conserve compute).

**Tick 0001-0003 (~0-45 seconds after boot):**

- WitnessEngine begins streaming blocks from Base. Every new block is received, decoded, and passed to the TriageEngine.
- TriageEngine scores each transaction against the Binary Fuse filter. Transactions touching Uniswap V3, Morpho Blue, or Aave V3 contracts score above the relevance threshold. Everything else is discarded.
- First relevant transactions arrive. On Base, block time is ~2 seconds. With Uniswap V3 as a seed protocol, the Golem sees relevant transactions within the first few blocks -- Uniswap V3 on Base processes thousands of swaps per hour.
- The TA pipeline receives its first price data points. For ETH/USDC (the primary pair in the market-observer template), the pipeline ingests the swap prices from Uniswap V3 transactions.

**Tick 0004-0008 (~45 seconds to 2.5 minutes):**

- First protocol state snapshot cached. The WitnessEngine queries current state for each seed protocol: Morpho vault APYs, Aave supply/borrow rates, Uniswap V3 pool liquidity distributions.
- TA pipeline builds first persistence diagrams. With 3+ price observations, the Vietoris-Rips complex can begin forming. The first topological features (clusters in price-volume space) appear. These are preliminary -- the persistence diagram is sparse.
- CorticalState receives its first write:
  - `regime_tag` set based on initial observations. With typical Base activity, this is usually `CALM` or `TRENDING`.
  - `arousal` calibrated from transaction volume relative to expected baseline.
  - `uncertainty` drops from 1.0 toward 0.8 -- still high, but the Golem is no longer completely ignorant.
- First Bayesian surprise scores computed. The Golem had priors (from the strategy template's implicit expectations) and now has observations. The KL divergence between prior and posterior gives the first surprise scores. Mostly low -- the market is probably behaving normally.

**What CorticalState looks like at minute 2:**

```
CorticalState {
    regime_tag: "CALM",
    arousal: 0.12,          // Low -- normal market activity
    valence: 0.0,           // Neutral -- no opinion yet
    uncertainty: 0.82,      // Still high, few observations
    topology_signal: 0.05,  // Market structure stable
    consistency: 0.95,      // Short-term and long-term views agree (trivially -- not enough data to disagree)
    surprise_avg: 0.3,      // Low surprise, market behaving as expected
}
```

### What the user sees

The TUI begins populating. Widgets appear as their data sources produce output.

```
+==================================================================+
| HEARTH   MIND   SOMA   WORLD   FATE   COMMAND                    |
+==================================================================+
|                                                                    |
|    ◉ ◉                                                            |
|   · · ·          CALM MARKET                                      |
|  · · · ·         ─────────────────────────                        |
|   · · ·                                                            |
|    · ·           Regime: Calm                                      |
|                  Unexpectedness: ▁▁▁ Low                           |
|                  Shape: ──── Stable                                 |
|                                                                    |
|  ┌ Live Feed ──────────────────────────────────────────────────┐  |
|  │ 12:00:14  Uniswap V3: 2.4 ETH → 6,120 USDC    score: 0.72 │  |
|  │ 12:00:11  Morpho Blue: deposit 500 USDC         score: 0.68 │  |
|  │ 12:00:08  Uniswap V3: 0.8 ETH → 2,040 USDC    score: 0.65 │  |
|  │ 12:00:05  Aave V3: supply 1,000 USDC            score: 0.61 │  |
|  │                                                              │  |
|  │  ···scanning···                                              │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|   ♡ ────── TICK 0006 ──── NAV $10.00 ──── 10.8 days ──── THR    |
+==================================================================+
```

Key elements now visible:

- **Regime classification** in plain English: "Calm Market." This is the CorticalState `regime_tag` translated. The user sees one of four words, not the underlying state vector.
- **Unexpectedness** bar: low. The Bayesian surprise average, rendered as a simple gauge. The user reads "low unexpectedness" and knows: nothing unusual is happening.
- **Shape** indicator: stable. The topology signal, rendered as a flat line. The market's structure is not shifting.
- **Live feed**: Real transactions, scored by the TriageEngine. The score (0.0-1.0) indicates relevance to the Golem's strategy. Higher scores mean the transaction is more relevant. The feed scrolls as new blocks arrive.

What is NOT visible yet: predictions (the Oracle hasn't registered any), pattern highlights (ChainScope hasn't expanded yet), the chain intelligence tab (not enough accumulated context). These appear in the next few minutes.

The Spectre's behavior: breathing normally, eyes tracking the live feed (slight lateral movement as new transactions arrive). The dot cloud is tight and cohesive (high Phi -- trivially, since all subsystems agree when there is little data). The Spectre looks calm and attentive.

---

## 3. Minute 3-5: Pattern Formation

### What happens inside

The Golem has been watching for 3 minutes. It has seen dozens of relevant transactions, cached protocol state snapshots, and built preliminary persistence diagrams. Now the intelligence pipeline begins producing output.

**ChainScope expansion via Hebbian reinforcement:**

ChainScope started with seed addresses (Uniswap V3 factory, Morpho Blue registry, Aave V3 pools). As the WitnessEngine processes transactions, ChainScope discovers related addresses through on-chain interaction patterns. A Uniswap V3 swap that routes through an aggregator adds that aggregator to the interest list. A Morpho vault that receives deposits from a specific contract adds that contract.

The expansion follows Hebbian learning: addresses that fire together (appear in the same transactions as seed addresses) wire together (get added to the Binary Fuse filter). The filter recompiles as new addresses join. By minute 5, the Golem is watching a broader network of contracts than its seed list specified.

**Oracle registers first predictions:**

The Oracle module generates its first predictions. These are basic: price direction (will ETH/USDC be higher or lower at the next theta tick?), yield stability (will Morpho vault APY stay within 10% of current value?), volume trend (will Uniswap V3 volume stay above/below the recent average?).

Each prediction has:
- A category (e.g., `uniswap_v3_price_direction`)
- A confidence (initially low, 0.50-0.55 -- barely better than random)
- A resolution time (next theta tick, ~60-180 seconds)
- An outcome tracker

The Oracle does not need high confidence to register predictions. Registering predictions is how it calibrates. The first batch of ~20-50 predictions establishes the baseline against which accuracy will be measured.

**Grimoire records first observations:**

The Grimoire (the Golem's knowledge store) receives its first entries:
- Protocol state observations (current APYs, TVLs, liquidity distributions)
- Market regime classification with confidence
- First topological features from the persistence diagram

These are raw observations, not insights. Insights form later, when the Golem has enough observations to identify patterns. But the Grimoire starts accumulating from minute 3.

**If user's watched tokens have activity, curiosity-driven alerts fire:**

The attention system monitors for events in the user's specified assets (ETH and USDC for the market-observer template). If there is a large swap, unusual volume, or a significant yield change, the curiosity system generates an alert. This is the first proactive communication from Golem to user.

### What the user sees

New elements appear on screen:

```
+==================================================================+
| HEARTH   MIND   SOMA   WORLD   FATE   COMMAND                    |
+==================================================================+
|                                                                    |
|    ◉ ◉                                                            |
|   · · ·          CALM MARKET                                      |
|  · · · ·         ─────────────────────────                        |
|   · · ·                                                            |
|    · ·           Regime: Calm                                      |
|                  Unexpectedness: ▁▂▁ Low                           |
|                  Shape: ──── Stable                                 |
|                                                                    |
|  ┌ Live Feed ──────────────────────────────────────────────────┐  |
|  │ 12:04:22  Uniswap V3: 15.0 ETH → 38,250 USDC  score: 0.91 │  |
|  │ 12:04:18  Morpho Blue: withdraw 2,000 USDC      score: 0.74 │  |
|  │ 12:04:12  Aave V3: borrow 800 USDC              score: 0.69 │  |
|  │ 12:04:08  Uniswap V3: 0.5 ETH → 1,275 USDC    score: 0.63 │  |
|  │ 12:03:55  Aggregator: multi-hop swap 3.2 ETH    score: 0.81 │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Emerging Patterns ──────────────────────────────────────────┐  |
|  │ ▸ ETH/USDC volume: 1.8x above 1h average                   │  |
|  │ ▸ Morpho USDC vault: withdrawals > deposits (last 5 min)    │  |
|  │ ▸ Aggregator routing: 60% of volume via aggregators         │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Predictions ────────────────────────────────────────────────┐  |
|  │ Registered: 34       Resolved: 0       Accuracy: --         │  |
|  │ First resolution in ~45 seconds                              │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|   ♡ ────── TICK 0018 ──── NAV $10.00 ──── 10.8 days ──── THR    |
+==================================================================+
```

New elements:

- **Emerging Patterns** panel: ChainScope's Hebbian expansion has identified patterns worth highlighting. These are not predictions -- they are observations about what the Golem is noticing. The `▸` prefix marks them as emerging (not yet confirmed as significant).
- **Predictions** panel: Shows that the Oracle has registered 34 predictions but none have resolved yet. The "first resolution in ~45 seconds" countdown tells the user when the accuracy gauge will get its first data point.

If a large ETH swap came through (like the 15.0 ETH transaction at 12:04:22), the live feed entry renders brighter than the others. The Spectre's eyes widen briefly -- this is the Bayesian surprise response. The unexpectedness gauge ticks up momentarily from `▁▁▁` to `▁▂▁`, then settles.

The Spectre has started showing subtle behavioral cues. Its dot cloud breathes slightly faster when the live feed is active (arousal responding to transaction volume). Its eyes track the highest-scored transaction in the feed. These are not decorative animations -- they are computed from CorticalState values.

---

## 4. Minute 5-10: Intelligence Builds

### What happens inside

The Golem has been running for 5 minutes. Multiple Gamma ticks have accumulated context. The intelligence pipeline shifts from initialization to production.

**Oracle predictions begin resolving:**

The first batch of predictions registered at minute 3-4 start resolving. Each resolution updates the Oracle's accuracy tracker:

- Price direction predictions: the Oracle predicted ETH/USDC would be higher at the next theta tick. Was it? Resolution is binary (correct/incorrect) with a confidence score.
- Yield predictions: the Oracle predicted Morpho vault APY would stay within 10% of 4.2%. Did it? Resolution feeds back into the yield prediction model.
- Volume predictions: the Oracle predicted Uniswap V3 volume would stay above average. Did it?

Early accuracy is typically 50-65%. The Oracle is barely better than random on its first predictions. But accuracy trends upward as the models calibrate. By tick ~30 (around minute 7-8), accuracy on the best categories may reach 60-70%.

**TA persistence diagrams show topological features:**

With 5+ minutes of price data, the persistence diagrams become informative. Topological features that persist across multiple scale parameters (long bars in the persistence diagram) indicate real market structure, not noise. The TA pipeline identifies:

- Clusters in price-volume space (betti_0 features): groups of transactions at similar price/volume combinations
- Loops in price-time space (betti_1 features): oscillatory patterns, support/resistance levels
- Voids (betti_2 features): price regions where no trading occurs (gaps, avoided zones)

The user does not see any of this terminology. They see "Market shape: stable" or "Market shape: shifting." The persistence diagram is available in Tier 3 (FATE > Chain Intelligence > Enter on any topology element).

**Bayesian surprise scores calibrate:**

After 5 minutes of observations, the Golem's priors have been updated repeatedly. The surprise score distribution settles into a baseline: most events score < 0.5 nats (expected), occasional events score 0.5-1.0 nats (mildly interesting), rare events score > 1.0 nats (genuinely surprising).

The calibration matters because surprise drives attention allocation. High-surprise events get more attention budget (longer observation, potential T1/T2 escalation). Low-surprise events get T0 processing only.

**Somatic markers begin forming:**

The somatic marker system (the Golem's "gut feelings") starts building associations. When a particular pattern (say, a sudden volume spike in ETH/USDC) coincides with a price movement, the somatic system records the association. After seeing this pattern 2-3 times, the marker strengthens. The next time the pattern appears, the somatic system fires before the analytical pipeline completes -- a faster, affect-based signal that says "I've seen this before, and here's what happened."

At minute 5-10, somatic markers are still forming. They are weak, low-confidence, and not yet reliable. But they are present in the CorticalState as faint signals, and they will strengthen over the coming hours and days.

**CorticalState at minute 8:**

```
CorticalState {
    regime_tag: "CALM",
    arousal: 0.18,          // Slightly elevated -- market active
    valence: 0.05,          // Slightly positive -- observations matching expectations
    uncertainty: 0.55,      // Dropping steadily as predictions resolve
    topology_signal: 0.08,  // Market structure still stable
    consistency: 0.91,      // Short and long views mostly agree
    surprise_avg: 0.35,     // Baseline surprise established
    somatic_markers: 3,     // Three weak markers forming
}
```

**If market is active, first FATE tab content populates:**

The FATE tab (chain intelligence, TA analysis, Oracle overview) requires accumulated data to be meaningful. By minute 7-8, enough data has arrived. The chain intelligence display shows scored events, the TA overlay begins rendering on the price chart, and the Oracle overview shows the accuracy trend.

### What the user sees

The dashboard is filling in. The predictions panel now has data.

```
+==================================================================+
| HEARTH   MIND   SOMA   WORLD   FATE   COMMAND                    |
+==================================================================+
|                                                                    |
|    ◉ ◉                                                            |
|   · · ·          CALM MARKET                                      |
|  · · · ·         ─────────────────────────                        |
|   · · ·                                                            |
|    · ·           Regime: Calm                                      |
|                  Unexpectedness: ▁▂▁ Low                           |
|                  Shape: ──── Stable                                 |
|                  Agreement: ████░ High                              |
|                                                                    |
|  ┌ Live Feed ──────────────────────────────────────────────────┐  |
|  │ 12:08:44  Uniswap V3: 5.0 ETH → 12,750 USDC   score: 0.84 │  |
|  │ 12:08:31  Morpho Blue: deposit 1,200 USDC       score: 0.71 │  |
|  │ 12:08:22  Aave V3: repay 600 USDC               score: 0.66 │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Predictions ────────────────────────────────────────────────┐  |
|  │ Accuracy: 58% (19 of 33 resolved correct)                   │  |
|  │ ▁▂▃▃▄▅▅▆▆ (trending up)                                     │  |
|  │                                                              │  |
|  │ Best category: morpho_yield_stability (71%)                  │  |
|  │ Building: eth_price_direction (52%), uniswap_volume (55%)    │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Emerging Patterns ──────────────────────────────────────────┐  |
|  │ ▸ Morpho USDC vault APY stable at 4.2% (6 observations)     │  |
|  │ ▸ ETH/USDC price range: $2,540-$2,560 (tight band)          │  |
|  │ ● Aggregator share increasing: 60% → 68% of volume          │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Gut Feelings (forming) ─────────────────────────────────────┐  |
|  │ ░ Volume spikes on ETH/USDC tend to precede price movement   │  |
|  │ ░ Morpho withdrawals correlate with Aave rate changes        │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|   ♡ ────── TICK 0038 ──── NAV $10.00 ──── 10.7 days ──── THR    |
+==================================================================+
```

New elements since minute 3-5:

- **Agreement** gauge: The sheaf consistency score, rendered as a simple bar. "High" means the Golem's short-term and long-term views match. This is the math-to-metaphor translation for consistency -- the user sees "agreement level," not "Hodge Laplacian eigenvalue."
- **Predictions** panel with data: Accuracy is 58% and trending up. The best-performing category is shown. The trend sparkline (`▁▂▃▃▄▅▅▆▆`) tells the user: the Golem is getting better at predicting, even though it started from near-random.
- **Emerging Patterns** now shows one `●` item (solid circle instead of `▸` triangle). The solid circle means the pattern has been confirmed through multiple observations. The aggregator share increase has been observed consistently.
- **Gut Feelings** panel: Somatic markers, translated. The `░` prefix indicates these are forming (low confidence). The user reads "volume spikes tend to precede price movement" and understands that the Golem is developing intuitions. The underlying somatic marker system, HDC vectors, and affect injection are invisible.

The Spectre's behavior has evolved. Its breathing rhythm has settled into a steady pattern synchronized with the Gamma clock. When predictions resolve correctly, the dot cloud tightens briefly (a micro-satisfaction response). When predictions fail, the cloud loosens slightly. The user who watches the Spectre for a minute starts to sense its rhythm.

A toast notification may appear:

```
┌──────────────────────────────────────────────────────────────────┐
│ ◎ First predictions resolving — accuracy building               │
│   Navigate to MIND > Pipeline for details                       │
└──────────────────────────────────────────────────────────────────┘
```

This is the auto-navigation system from `07-onboarding.md` section 15.4. The toast links to the relevant screen but does not force navigation. After 30 minutes of operation, auto-navigation stops.

---

## 5. Minute 10-15: Ready to Act (or Wait)

### What happens inside

The Golem has been running for 10 minutes. It has processed hundreds of transactions, resolved dozens of predictions, built persistence diagrams with real topological structure, and formed its first somatic markers. The intelligence pipeline is in steady-state production.

**Oracle has initial calibration:**

By tick ~60 (around minute 12), the Oracle has resolved 50-100 predictions across multiple categories. Accuracy varies by category:

| Category | Typical accuracy at minute 12 | Trend |
|---|---|---|
| `morpho_yield_stability` | 70-80% | Stable (yields change slowly, easy to predict) |
| `aave_rate_direction` | 60-70% | Improving (rate changes are partially predictable) |
| `eth_price_direction` | 50-60% | Flat (price direction at short timescales is hard) |
| `uniswap_volume_trend` | 55-65% | Slowly improving |

The action gate (the confidence threshold above which the Golem can trade) is set to 0.80 for the market-observer template. No category has reached 80% at minute 12. The Golem correctly remains in observation mode. This is not a limitation -- it is the design working as intended.

**Context Governor has attention priorities:**

The Context Governor (the attention allocation system) has observed enough events to rank attention priorities. For the market-observer template, typical priorities at minute 12:

1. Morpho USDC vault (highest prediction accuracy, most stable signal)
2. Uniswap V3 ETH/USDC (highest volume, most data)
3. Aave V3 USDC supply (moderate prediction accuracy)
4. Aggregator routing (emerging pattern, curiosity-driven priority)

The priority ranking determines which protocols get more attention budget (longer observation windows, more frequent state snapshots, higher-quality inference when T1/T2 ticks fire).

**Risk Engine has baseline volatility estimate:**

The Risk Engine has ingested enough price data to compute a baseline volatility estimate for ETH/USDC. For a $10 portfolio, the Risk Engine's recommendation is conservative:

```
RiskAssessment {
    baseline_volatility: 0.012,          // 1.2% per hour (typical for ETH)
    max_position_size: 2.00,             // $2 max per trade (from template)
    gas_budget_remaining: 9.97,          // Almost all capital untouched
    recommendation: "OBSERVE",           // Do not trade
    reason: "No category exceeds action gate (0.80). Portfolio size ($10) favors
             continued observation. Gas cost ($0.10-0.50 per swap) represents
             1-5% of capital."
}
```

**First curiosity-driven discovery:**

The curiosity system fires when the Golem notices something it wasn't specifically looking for. By minute 10-15, common discoveries include:

- "I noticed high activity on Aerodrome (a DEX the Golem wasn't originally watching). Volume is 3x the 24h average."
- "Morpho USDC vault utilization jumped from 72% to 81% in the last 5 minutes. This usually precedes a rate increase."
- "A new lending market opened on Morpho Blue for cbETH/USDC. No prior data available."

These discoveries are the Golem's first proactive intelligence. They come from ChainScope's Hebbian expansion encountering unexpected patterns, not from the user's explicit instructions. The Golem is noticing things the user did not ask it to look for.

**CorticalState at minute 14:**

```
CorticalState {
    regime_tag: "CALM",
    arousal: 0.15,
    valence: 0.08,           // Mildly positive -- predictions improving
    uncertainty: 0.38,       // Much lower -- substantial calibration complete
    topology_signal: 0.06,
    consistency: 0.89,
    surprise_avg: 0.28,      // Baseline well-established
    somatic_markers: 7,      // Several markers at weak-to-moderate strength
    predictions_resolved: 67,
    accuracy_overall: 0.61,  // 61% across all categories
    oracle_best: {
        category: "morpho_yield_stability",
        accuracy: 0.76
    },
}
```

### What the user sees

The full dashboard is populated. The Spectre has stabilized into its steady-state breathing pattern. The screen feels alive but not frantic.

```
+==================================================================+
| HEARTH   MIND   SOMA   WORLD   FATE   COMMAND                    |
+==================================================================+
|                                                                    |
|    ◉ ◉           CALM MARKET                                      |
|   · · ·          ─────────────────────────                        |
|  · · · ·                                                           |
|   · · ·          Regime: Calm                                      |
|    · ·           Unexpectedness: ▁▁▂ Low                           |
|                  Shape: ──── Stable                                 |
|                  Agreement: ████░ High                              |
|                  Coherence: █████ Strong                            |
|                                                                    |
|  ┌ Insights ───────────────────────────────────────────────────┐  |
|  │                                                              │  |
|  │ "Morpho USDC vault has maintained 4.1-4.3% APY across 12    │  |
|  │  observations. This is the most predictable signal I've      │  |
|  │  found so far. My yield predictions for Morpho are correct   │  |
|  │  76% of the time."                                           │  |
|  │                                                              │  |
|  │ "I noticed high activity on Aerodrome DEX -- volume is 3x    │  |
|  │  the 24h average. I wasn't watching this protocol initially, │  |
|  │  but transactions from aggregators led me there."            │  |
|  │                                                              │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Predictions ────────────────────────────────────────────────┐  |
|  │ Accuracy: 61% (67 resolved)                                  │  |
|  │ ▁▂▃▃▄▅▅▆▆▆▆▇ (improving)                                    │  |
|  │                                                              │  |
|  │ morpho_yield:  ████████████████████░░░░░  76%                │  |
|  │ aave_rate:     ██████████████░░░░░░░░░░░  63%                │  |
|  │ uni_volume:    ████████████░░░░░░░░░░░░░  57%                │  |
|  │ eth_price:     ██████████░░░░░░░░░░░░░░░  52%                │  |
|  │                                                              │  |
|  │ Action gate: 80%                  Status: OBSERVING          │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Gut Feelings ───────────────────────────────────────────────┐  |
|  │ ▒ Morpho rates are predictable (strong)                      │  |
|  │ ░ Volume spikes precede price movement (forming)             │  |
|  │ ░ Aave rate changes follow Morpho utilization (forming)      │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|  ┌ Status ─────────────────────────────────────────────────────┐  |
|  │ Watching: 3 protocols, 47 addresses                          │  |
|  │ Discovered: Aerodrome DEX (via aggregator routing)           │  |
|  │ Recommendation: Continue observing. No category meets the    │  |
|  │ 80% confidence threshold for trading at this portfolio size. │  |
|  └──────────────────────────────────────────────────────────────┘  |
|                                                                    |
|   ♡ ────── TICK 0064 ──── NAV $10.00 ──── 10.6 days ──── THR    |
+==================================================================+
```

The screen tells a story. In 15 minutes, the Golem has:

- Classified the market regime (Calm)
- Watched 3 protocols and discovered a 4th (Aerodrome) on its own
- Registered and resolved 67 predictions, reaching 61% overall accuracy
- Identified Morpho yield stability as its strongest predictive category (76%)
- Formed somatic markers -- gut feelings about pattern correlations
- Produced two insights in natural language, written from first person
- Made a clear recommendation: keep observing

The NAV has barely moved. $10.00 became ~$9.97 (compute costs consumed ~$0.03 in 15 minutes at micro tier). No gas spent. No trades. The Golem preserved capital while building intelligence.

### What the user takes away

The user closes the terminal (or leaves it running in the background) with a concrete understanding:

1. **The Golem is thinking.** Not stuck. Not loading. Actively observing, predicting, and learning.
2. **It found something on its own.** The Aerodrome discovery was not in the strategy. The Golem noticed it through exploration.
3. **It knows what it doesn't know.** The 80% action gate is visible. The Golem is honest about its accuracy and won't trade until it is confident.
4. **It has opinions.** The insights panel is first-person narrative. "The most predictable signal I've found" -- the Golem is forming a perspective, not just reporting data.
5. **$10 is enough.** The Golem produced real intelligence with $10 and 15 minutes. It did not need $1,000 to be useful.

---

## 6. Math-to-Metaphor Reference

Every number and signal in the preceding narrative maps to an underlying mathematical computation. This section provides the complete translation table for the first-15-minutes experience.

| What the user sees | Internal signal | Computation | Where in TUI |
|---|---|---|---|
| "Calm Market" | `CorticalState.regime_tag` | Topology regime classifier from persistence diagram features | HEARTH sidebar |
| Unexpectedness bar | Bayesian surprise average | Mean KL divergence across recent observations | HEARTH sidebar |
| Shape stability | `topology_signal` | Wasserstein distance between successive persistence diagrams | HEARTH sidebar |
| Agreement level | Sheaf consistency score | Hodge Laplacian spectral analysis across timescales | HEARTH sidebar |
| Coherence level | Phi (integrated information) | Mutual information across subsystem bipartitions | HEARTH sidebar |
| Prediction accuracy % | Oracle accuracy tracker | Correct predictions / total resolved predictions per category | MIND > Pipeline |
| Gut feeling strength | Somatic marker confidence | HDC vector binding strength, observation count, affect injection magnitude | HEARTH > Gut Feelings |
| Transaction score | TriageEngine relevance score | Binary Fuse filter match + strategy relevance weighting | HEARTH > Live Feed |
| Emerging pattern `▸` | ChainScope observation | Hebbian reinforcement count < threshold | HEARTH > Patterns |
| Confirmed pattern `●` | ChainScope confirmed | Hebbian reinforcement count >= threshold | HEARTH > Patterns |
| Spectre breathing rate | `CorticalState.arousal` | Transaction volume relative to baseline, mapped to Spectre animation frequency | Spectre sidebar |
| Spectre cloud tightness | Phi value | High Phi = tight cloud, low Phi = loose cloud | Spectre sidebar |
| Spectre eye widening | Bayesian surprise > 1.0 nats | Single-event KL divergence spike | Spectre sidebar |

Every visual effect is invertible. The user who sees the Spectre's eyes widen can press Enter, navigate to MIND > Pipeline, and see the exact Bayesian surprise value, the prior and posterior distributions, and the observation that caused the update. The metaphor does not hide the math -- it layers presentation over it.

---

## 7. The $10 Value Proposition

A $10 Golem cannot compete with a $10,000 Golem on trade volume or position sizing. But it can match it on intelligence per dollar.

**What scales with capital:**
- Position sizes
- Diversification across protocols
- Gas budget (more trades, more experimentation)
- Risk tolerance (can absorb losses)

**What does NOT scale with capital:**
- Observation quality (same WitnessEngine, same TriageEngine)
- Prediction accuracy (same Oracle, same calibration process)
- Pattern discovery (same ChainScope, same Hebbian expansion)
- Market regime classification (same CorticalState, same topology pipeline)
- Somatic marker formation (same embodied cognition system)

A $10 Golem sees the same market as a $10,000 Golem. It forms the same predictions. It discovers the same patterns. It classifies the same regimes. The difference is what it does with that intelligence: the $10 Golem reports; the $10,000 Golem acts.

For many users, the reporting is the product. "Show me what's happening in DeFi, explained by an agent that has been watching continuously and has opinions" is valuable independent of trade execution. The $10 Golem is a market intelligence service that happens to also be capable of trading when the user scales up.

### Upgrade path

When the user is ready to trade, the path is:

1. Top up USDC (direct transfer, bridge, or on-ramp)
2. Adjust strategy: `auto_trade = true`, lower `min_trade_confidence` to 0.60
3. The Golem already has calibrated predictions, formed somatic markers, and built a knowledge base. It does not need to re-learn from zero. It starts trading with the intelligence it built during observation mode.

The observation period was not wasted time. It was training. The Golem that trades on day 2 is better than a Golem that traded on minute 1.

---

## 8. Timing Reference

Summary of all timing estimates for the first 15 minutes.

| Time | Event | Depends on |
|---|---|---|
| 0s | SUMMON typed | User |
| 0-5s | VM provisioned (warm pool) | Bardo Compute |
| 3-8s | Boot sequence completes | Config + RPC latency |
| 8-10s | Birth cinematic begins | Boot completion |
| 18-25s | Birth cinematic ends, TUI frame renders | Fixed animation duration |
| 25-30s | First blocks streamed, first transactions scored | Base block time (~2s) |
| 30-45s | First CorticalState write, regime classified | 2-3 Gamma ticks |
| 45-90s | First protocol state snapshots cached | RPC query latency |
| 90-120s | First persistence diagram built | 5+ price observations |
| 2-3 min | Oracle registers first predictions | Sufficient observations |
| 3-4 min | ChainScope begins Hebbian expansion | Correlated address observations |
| 3-5 min | First Grimoire entries written | Accumulated observations |
| 4-6 min | First predictions resolve | Theta tick resolution (~60-180s) |
| 5-7 min | Accuracy gauge gets first data points | Resolved predictions |
| 5-8 min | First somatic markers begin forming | Repeated pattern-outcome pairs |
| 7-10 min | FATE tab content populates | Accumulated context |
| 10-12 min | Oracle has initial calibration (~50-100 resolutions) | Steady prediction cycle |
| 10-15 min | First curiosity-driven discovery | ChainScope expansion + surprise |
| 12-15 min | Full dashboard populated, Spectre stabilized | All subsystems producing |
| 15 min | First insights in natural language | Enough observations for synthesis |

All timings are estimates for typical Base mainnet activity. During low-activity periods (weekends, off-peak hours), events that depend on on-chain activity may take longer. During high-activity periods (market events, token launches), events may occur faster and with higher intensity.

---

## 9. What Happens After 15 Minutes

The first 15 minutes establish the baseline. What follows:

**Minutes 15-60:** The Golem continues calibrating. Prediction accuracy improves. Somatic markers strengthen. The Grimoire accumulates observations. If the market is active, the Oracle's best categories may reach the action gate threshold. For the market-observer template (`auto_trade = false`), the Golem reports opportunities rather than acting on them.

**Hours 1-6:** The first theta-scale patterns emerge. The Golem identifies hourly rhythms in volume, yield rates, and liquidity. ChainScope has expanded to 50-100 watched addresses. Dream cycles are still disabled (need 50-200 episodes, typically 4-6 hours of operation).

**Hours 6-24:** First dream cycle fires (if enough episodes have accumulated). The Golem recombines observations into counterfactual scenarios during REM sleep, then integrates the results during the Integration phase. First heuristics form. The PLAYBOOK begins diverging from the template defaults.

**Day 2+:** The Golem is a fully operational intelligence agent. For a $10 portfolio, it continues in observation mode by default. When the user tops up capital or enables auto-trade, the Golem has days of accumulated intelligence to draw on. It trades from a position of knowledge, not ignorance.

> **See also:**
>
> - `./07-onboarding.md` section 11 (lifecycle milestones: day 1, 7, 30, 90)
> - `./07-onboarding.md` section 15 (first contact: installation through first hour)
> - `../01-golem/06-creation.md` section S7 (small portfolio creation defaults)

---

_End of document._
