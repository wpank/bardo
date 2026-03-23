# ⌈ ＰＲＯＴＯＣＯＬ ＶＩＥＷ ＣＡＴＡＬＯＧ ⌋
## Terminal Interface · v4.06
### Every Protocol, Every View, Every Pane

**Source:** `engagement-prd/bardo-v4-06-protocol-view-catalog.md`

**Cross-references:** `00-sanctum-protocol-layer.md` §3 (common protocol view chrome shared by all Sanctum views), `../perspective/01-golem-perspective.md` (F2 Perspective overlay adding the Golem's inner thoughts to protocol data), `../screens/02-widget-catalog.md` (33-widget TUI component library; FlashNumber §1, sparklines §8)

> **Reader orientation:** This document is the per-protocol view catalog for the Sanctum layer, specifying the exact tab/pane layouts for each supported DeFi protocol (Uniswap, Aave, Morpho, Aerodrome, etc.) inside the Bardo terminal. It belongs to the **interfaces/protocol layer**. Each entry lists the protocol's chains, Golem tools (the `@bardo/sanctum` adapters backing the view), internal tabs, and what the Golem's Perspective (F2) would surface. Key concepts: Golem (a mortal autonomous DeFi agent), Grimoire (persistent knowledge store with historical context per protocol), and the Spectre (dot-cloud creature visible in the sidebar during all protocol interactions). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. How to Read This Document

Each protocol entry specifies:
- **What it is** — one-line description
- **Chains** — where it's deployed (relevant to Bardo)
- **Golem Tools** — which `@bardo/tools` adapters back this view
- **Tabs** — the internal tab structure of the Protocol View modal
- **Golem-Specific Context** — what the Golem's Perspective (F2) would surface here

Protocols are organized by category. Each tab listing shows its panes. For brevity, standard widgets (FlashNumber, sparklines, confidence bars) are referenced by Widget Catalog section number rather than re-described.

All Protocol Views share the common chrome described in `00-sanctum-protocol-layer.md` §3.

---

## 1. DEX — Decentralized Exchanges

---

### 1.1 Uniswap

**What it is:** The dominant AMM. Concentrated liquidity (V3), hook-extensible pools (V4), intent-based routing (UniswapX).

**Chains:** Ethereum, Base, Arbitrum, Celo

**Golem Tools:** `execute_swap`, `get_quote`, `simulate_price_impact`, `add_liquidity`, `remove_liquidity`, `collect_fees`, `get_position`, `get_tick_data`, `calculate_il`, `get_fee_earnings_history`, `get_pool_info`, `get_token_price`, `get_token_info`, `get_order_flow_metrics`

**Golem Skills:** `spot-trading`, `concentrated-lp`

#### Tab 1: Swap

The core swap interface. Mirrors the uniswap.org swap UI.

**Panes:**

- **Swap Form** (center, 60%)
  - Token In selector + amount input + balance display
  - Token Out selector + estimated receive amount
  - Version selector: V2 / V3 / V4 / UniswapX (auto-route recommended)
  - Route visualization: which pools the swap routes through, fee tiers, hop count
  - Price impact estimate (FlashNumber, rose if >1%)
  - Slippage setting (inline, configurable)
  - Gas estimate
  - `F8` to execute → Confirmation Modal with full breakdown

- **Price Chart** (right, 40%)
  - Token pair price over time (1H / 1D / 1W / 1M selectable)
  - Rendered as waveform trace (Widget Catalog §3) or sparkline
  - Current price as bone number at top
  - 24h high/low markers

- **Recent Trades** (bottom strip)
  - Last 20 swaps on this pair, scrolling. Each: direction, amount, price, timestamp
  - The Golem's own trades highlighted with a `◆` marker

**Golem Context (F2):** Best execution route analysis, historical slippage on this pair, gas timing recommendation, comparison to the Golem's past swaps on this pair.

#### Tab 2: Pools

Pool explorer. Browse and analyze liquidity pools.

**Panes:**

- **Pool List** (left, 45%)
  - All pools for the selected chain, sortable by: TVL, volume (24h), fee tier, APR
  - Each row: token pair, fee tier, TVL (FlashNumber), 24h volume, 7d APR estimate
  - Filterable: token, fee tier, TVL range, "my positions" toggle
  - Pools where the Golem has positions are marked with `◆`

- **Pool Detail** (right, 55%)
  - Selected pool's full stats: TVL, volume, fees collected, token reserves
  - Tick liquidity distribution chart (horizontal bar, showing where liquidity is concentrated)
  - Current price indicator within the tick range
  - Fee APR estimate based on recent volume
  - Top LPs (anonymized, showing position sizes and ranges)

*When locked on Pool Detail:* Enter opens **Pool Deep Dive Modal**:
  - Full historical TVL/volume/fee charts
  - Impermanent loss calculator for custom ranges
  - All positions in this pool (the Golem's highlighted)

**Golem Context (F2):** The Golem's assessment of pool health, comparison to similar pools on other DEXs, historical fee earnings analysis, optimal range suggestion based on the Golem's causal graph.

#### Tab 3: Positions

The Golem's (or terminal wallet's) LP positions.

**Panes:**

- **Position List** (left, 40%)
  - All active LP positions: pool, range, liquidity, uncollected fees, in/out of range status
  - In-range positions: `success` border. Out-of-range: `rose_bright` pulsing border
  - Each position shows a mini range bar: current price as a dot within the selected tick range

- **Position Detail** (right, 60%)
  - Selected position's full breakdown: entry price, current value, fees earned, IL estimate
  - Range visualization: thick bar showing lower tick, upper tick, current price
  - Fee earnings history as sparkline
  - Token composition (how much of each token is held)
  - Collect Fees button, Remove Liquidity button, Adjust Range button

- **Add Liquidity Form** (accessible via `n` key — new position)
  - Token pair selector, fee tier selector
  - Range selector: price range input OR tick range input
  - Amount inputs for each token
  - Estimated APR based on historical data
  - `F8` to execute

**Golem Context (F2):** Each position annotated with: entry rationale (if from a Grimoire episode), performance vs. expectation, rebalancing recommendation, IL vs. fees P&L, comparison to simply holding.

#### Tab 4: Tokens

Token browser and analytics.

**Panes:**

- **Token List** (left, 50%)
  - All tokens known to the Golem (from observation pipeline) + top tokens by volume
  - Each: name, symbol, price (FlashNumber), 24h change (green/rose), balance if held
  - Filterable: "held tokens", "recently traded", price range, volume
  - Sortable: price, 24h change, volume, balance value

- **Token Detail** (right, 50%)
  - Selected token: price chart (waveform), market cap, circulating supply
  - Available pools for this token (with links to Pool tab)
  - The Golem's recent transactions involving this token
  - Token contract address (copyable)
  - Safety indicator: verified (green), unverified (amber), flagged (rose)

**Golem Context (F2):** Token due diligence analysis (from `token-analyst` archetype tools), honeypot detection results, liquidity depth assessment, correlation with the Golem's other holdings.

---

### 1.2 Aerodrome

**What it is:** The dominant DEX on Base. MetaDEX combining Uniswap V2-style volatile pools, Curve-style stable pools, and Slipstream (concentrated liquidity). Ve-tokenomics for emission voting (view-only for Golems — trading/LP side only, no governance participation).

**Chains:** Base

**Golem Tools:** Same swap/LP tools as Uniswap, adapted for Aerodrome's router contracts. Additional: `get_aerodrome_pool_info`, `get_aerodrome_gauge_info`, `get_aerodrome_emissions`

#### Tab 1: Swap

Functionally identical to Uniswap Swap tab, but routing through Aerodrome's router. The route visualization shows whether the swap goes through a volatile pool (vAMM), stable pool (sAMM), or Slipstream (clAMM).

**Unique pane:**
- **Pool Type Indicator**: Shows which AMM invariant the selected route uses (constant product / StableSwap hybrid / concentrated), with a brief explanation.

#### Tab 2: Pools

Similar to Uniswap Pools, but with Aerodrome-specific data:

- **Pool Type Column**: `volatile` / `stable` / `concentrated (Slipstream)`
- **Gauge APR**: Emission rewards from AERO gauge, separate from fee APR
- **Voting Weight**: How much veAERO is voting for this pool's gauge (view-only)
- **Bribe Info**: Any active bribes on this pool's gauge (view-only)

Pools are sortable by: TVL, volume, fee APR, gauge APR, total APR (fee + gauge).

#### Tab 3: Positions

Same structure as Uniswap Positions. For Slipstream positions, the range visualization is identical to Uniswap V3 concentrated positions. For legacy V2-style positions, the range is full (no concentration).

**Unique pane:**
- **Emissions Earned**: AERO rewards earned from gauges on LP positions. Claimable with a dedicated action.

#### Tab 4: Tokens

Same as Uniswap Tokens, scoped to Aerodrome-listed tokens on Base.

**Golem Context (F2):** Comparison between Aerodrome and Uniswap for the same pair (if both are available on Base), fee analysis including emission rewards, optimal pool type recommendation (volatile vs. stable vs. Slipstream for the given pair).

---

### 1.3 Curve

**What it is:** The premier stablecoin/pegged-asset DEX. StableSwap invariant for low-slippage swaps between correlated assets.

**Chains:** Ethereum, Base, Arbitrum

**Golem Tools:** `get_curve_pool_info`, `get_curve_swap_quote`, `execute_curve_swap`, `add_curve_liquidity`, `remove_curve_liquidity`

#### Tab 1: Swap
- Optimized for stable pairs (USDC/USDT/DAI, stETH/ETH, etc.)
- Shows the StableSwap advantage: comparison with constant-product slippage
- Bonus/penalty indicator for balanced vs. imbalanced swaps

#### Tab 2: Pools
- All Curve pools on the selected chain
- Each pool shows: tokens (2-4 per pool), TVL, volume, base APY, reward APY
- Pool balance indicator: shows how balanced the pool's reserves are (important for Curve)

#### Tab 3: Positions
- The Golem's LP positions in Curve pools
- Shows proportional share, fees earned, reward tokens claimable

**Golem Context (F2):** Peg stability analysis, comparison of Curve vs. Uniswap for the same pairs, pool balance risk assessment.

---

## 2. LENDING — Borrow and Supply Protocols

---

### 2.1 Aave

**What it is:** The largest decentralized lending protocol. Variable and stable interest rates, flash loans, isolation mode markets.

**Chains:** Ethereum, Base, Arbitrum

**Golem Tools:** `get_lending_position`, `supply_collateral`, `borrow_asset`, `repay_debt`, `calculate_health_factor`, `simulate_leverage_change`, `get_aave_market_info`, `get_aave_rates`

**Golem Skills:** `lending-loop`

#### Tab 1: Markets

Overview of all Aave markets on the selected chain.

**Panes:**

- **Market List** (full width)
  - Each market: asset, total supply, total borrow, supply APY, borrow APY (variable), borrow APY (stable if available), utilization rate
  - APYs as FlashNumbers — they change with every block
  - Markets where the Golem has positions are marked
  - Sortable: supply APY, borrow APY, utilization, TVL

*When locked:* Enter on a market opens **Market Detail Modal**:
  - Historical rate charts (supply and borrow APY over time)
  - Utilization curve (showing the interest rate model's response to utilization)
  - Reserve configuration (LTV, liquidation threshold, liquidation bonus)
  - Oracle source and price feed health

#### Tab 2: Supply

Supply (lend) assets to Aave.

**Panes:**

- **Supply Form** (center)
  - Asset selector (filtered to Golem's held tokens)
  - Amount input + balance + Max button
  - Current supply APY + projected APY after deposit
  - Collateral toggle (enable/disable as collateral)
  - `F8` to execute

- **Supply Positions** (below)
  - Currently supplied assets: asset, amount, value, current APY, earnings
  - Each position: withdraw button, toggle collateral

#### Tab 3: Borrow

Borrow assets against supplied collateral.

**Panes:**

- **Borrow Form** (center)
  - Asset selector (filtered to borrowable assets)
  - Amount input
  - Rate mode: variable / stable
  - Current borrow APY
  - Health factor preview (current → projected after borrow)
  - Health factor gauge (Widget Catalog §2): thick bar, rose_bright below 1.5, hazard stripes below 1.2
  - `F8` to execute

- **Borrow Positions** (below)
  - Currently borrowed assets: asset, amount, value, rate mode, current APY, accrued interest
  - Each position: repay button, switch rate mode

- **Health Factor** (right, persistent)
  - Large health factor number (bone if >2, warn if 1.2-2, rose_bright if <1.2)
  - Breakdown: collateral value, borrow value, liquidation threshold
  - Projected health factor under stress scenarios (10% drop, 25% drop, 50% drop)

**Golem Context (F2):** Health factor trajectory analysis, optimal leverage recommendation, comparison of Aave rates to Morpho/Compound, liquidation risk assessment with Monte Carlo simulation, historical rate pattern analysis.

#### Tab 4: Positions Overview

Unified view of all supply and borrow positions.

**Panes:**

- **Summary** (top strip): Net worth on Aave, total supply value, total borrow value, net APY, health factor
- **Positions Table** (center): All positions in a single sortable table
- **Liquidation Scenarios** (right): Visual stress test showing at what price levels liquidation occurs

---

### 2.2 Morpho

**What it is:** Optimized lending with permissionless market creation. Markets are created by curators with custom parameters (oracle, LLTV, IRM). More capital-efficient than pool-based lending.

**Chains:** Ethereum, Base

**Golem Tools:** `get_morpho_market_info`, `get_morpho_position`, `supply_morpho`, `borrow_morpho`, `repay_morpho`, `withdraw_morpho`, `create_morpho_market`, `get_morpho_utilization`

**Golem Skills:** `lending-loop`

#### Tab 1: Markets

**Panes:**

- **Market List** (full width)
  - Each market: collateral asset / loan asset, LLTV, oracle, supply APY, borrow APY, utilization, total supply, total borrow, curator
  - Markets grouped by loan asset (USDC markets, WETH markets, etc.)
  - Morpho-specific: curator reputation (if ERC-8004 registered), market age, number of suppliers/borrowers

*When locked:* Enter on a market opens **Market Detail Modal**:
  - Interest rate model visualization (utilization → rate curve)
  - Oracle configuration and price feed history
  - Curator info (who created this market, their track record)
  - Bad debt history (if any)

#### Tab 2: Supply / Borrow

Combined form (Morpho markets are per-pair, so supply and borrow happen in the same market context).

**Panes:**

- **Market Selector** (top): Choose which market to interact with
- **Supply Form** (left): Supply the loan asset to earn interest
- **Borrow Form** (right): Supply collateral and borrow the loan asset
- **Position Summary** (bottom): Current supply/borrow, health factor, earned interest

Health factor visualization follows the same pattern as Aave.

#### Tab 3: Create Market

For Golems with sufficient reputation (Verified tier+), create a new Morpho market.

**Panes:**

- **Market Configuration Form**:
  - Collateral asset selector
  - Loan asset selector
  - Oracle selector (Chainlink, custom)
  - LLTV (liquidation LTV) slider
  - Interest Rate Model selector
  - Preview: estimated initial rates, comparison to existing similar markets

- **Existing Market Comparison** (right): Similar markets already deployed, their TVL and rates

`F8` → deploys the market contract on-chain.

**Golem Context (F2):** Market creation analysis — is there demand for this pair? Comparison to existing markets. Oracle reliability assessment. Projected utilization based on the Golem's market observations.

#### Tab 4: Positions Overview

Same pattern as Aave Tab 4, scoped to Morpho positions.

---

### 2.3 Compound

**What it is:** Pioneer of algorithmic interest rates. Pool-based lending with governance-managed parameters.

**Chains:** Ethereum, Base

**Golem Tools:** `get_compound_market_info`, `supply_compound`, `borrow_compound`, `repay_compound`, `withdraw_compound`

Tabs follow the same Aave pattern: Markets, Supply, Borrow, Positions Overview. Compound-specific: COMP reward tracking, governance proposal indicators (view-only).

---

### 2.4 Spark

**What it is:** MakerDAO's lending front-end. Borrow DAI against ETH and other collateral. Closely integrated with the Maker/Sky ecosystem.

**Chains:** Ethereum

**Golem Tools:** `get_spark_market_info`, `supply_spark`, `borrow_spark_dai`

Tabs: Markets, Supply, Borrow DAI, Positions. Spark-specific: DSR (DAI Savings Rate) integration — a pane showing the current DSR and an option to deposit DAI for yield.

---

## 3. YIELD — Aggregators and Structured Products

---

### 3.1 Pendle

**What it is:** Yield tokenization. Split yield-bearing assets into principal tokens (PT) and yield tokens (YT). Trade future yield.

**Chains:** Ethereum, Arbitrum

**Golem Tools:** `get_pendle_markets`, `get_pendle_position`, `buy_pt`, `buy_yt`, `add_pendle_liquidity`, `redeem_pendle`

#### Tab 1: Markets
- List of yield markets: underlying asset, maturity date, implied yield (PT), YT price
- Each market: time to maturity countdown, TVL, volume

#### Tab 2: Trade
- Buy/sell PT or YT
- Fixed yield calculator: "Lock in X% yield until maturity date"
- Yield speculation: "Buy YT to bet on yield going up"

#### Tab 3: Positions
- PT holdings, YT holdings, LP positions in Pendle pools
- Maturity countdown for each position
- Accrued yield

**Golem Context (F2):** Yield curve analysis, comparison of fixed yield (PT) vs. variable yield, historical implied yield trends, maturity risk assessment.

---

### 3.2 Beefy / Yearn

**What it is:** Yield optimizers that auto-compound rewards across protocols.

**Chains:** Base, Arbitrum (Beefy) / Ethereum (Yearn)

**Golem Tools:** `get_vault_list`, `deposit_vault`, `withdraw_vault`, `get_vault_apy`

#### Tab 1: Vaults
- List of available vaults: underlying strategy, APY (live), TVL, platform
- Strategy description for each vault
- Risk rating (if available)

#### Tab 2: My Vaults
- Deposited vaults: deposit amount, current value, earned yield, APY
- Withdraw form per vault

**Golem Context (F2):** Risk analysis of the underlying strategy, comparison to manual LP/lending, smart contract risk assessment.

---

## 4. STAKING — Liquid Staking and Restaking

---

### 4.1 Lido

**What it is:** Liquid staking for Ethereum. Stake ETH, receive stETH.

**Chains:** Ethereum

**Golem Tools:** `stake_lido`, `get_lido_stats`, `get_steth_rate`

#### Tab 1: Stake
- Stake ETH → receive stETH
- Current staking APR
- stETH/ETH exchange rate
- Withdrawal queue status

#### Tab 2: Position
- stETH balance, ETH equivalent, earned rewards
- stETH integration opportunities (use in Aave, Curve, etc.)

---

### 4.2 EigenLayer

**What it is:** Restaking protocol. Use staked ETH to secure additional networks.

**Chains:** Ethereum

**Golem Tools:** `get_eigenlayer_strategies`, `deposit_eigenlayer`, `withdraw_eigenlayer`, `get_eigenlayer_position`

#### Tab 1: Strategies
- Available restaking strategies: underlying asset, TVL, additional yield
- AVS (Actively Validated Services) secured by each strategy

#### Tab 2: Positions
- Restaked assets, earned rewards, withdrawal status
- Points/rewards tracking

---

### 4.3 Rocket Pool

**What it is:** Decentralized ETH staking with node operator network.

**Chains:** Ethereum

**Golem Tools:** `stake_rocketpool`, `get_reth_rate`, `get_rocketpool_stats`

Similar to Lido: Stake tab, Position tab. Rocket Pool–specific: rETH exchange rate, minipool status (view-only).

---

## 5. BRIDGE — Cross-Chain Asset Transfer

---

### 5.1 Across

**What it is:** Optimistic bridge with fast fills via relayer network.

**Chains:** Multi-chain (Ethereum ↔ Base, Arbitrum, Optimism, etc.)

**Golem Tools:** `get_bridge_quote`, `submit_bridge_transfer`, `check_bridge_status`, `get_bridgeable_tokens`

**Golem Skills:** `cross-chain-arbitrage`

#### Tab 1: Bridge
- Source chain / destination chain selectors
- Token selector + amount
- Quote: estimated receive amount, fee, estimated time
- Route details: which relayer, fill time history
- `F8` to execute

#### Tab 2: History
- Past bridge transfers: status (pending/completed/failed), source/dest, amount, time elapsed
- Pending transfers with live status tracking

**Golem Context (F2):** Bridge safety analysis, historical fill times for this route, fee comparison across bridges, cross-chain arbitrage opportunity detection.

---

### 5.2 Stargate / Hop

Similar structure to Across. Stargate-specific: LayerZero messaging visualization. Hop-specific: rollup-to-rollup optimized routing.

---

## 6. STABLECOIN — Minting and Management

---

### 6.1 MakerDAO / Sky

**What it is:** The original DeFi lending protocol. Mint DAI stablecoin against collateral. Recently rebranded to Sky.

**Chains:** Ethereum

**Golem Tools:** `get_maker_vault_info`, `open_maker_vault`, `deposit_maker_collateral`, `mint_dai`, `repay_maker_vault`, `get_dsr_rate`

#### Tab 1: Vaults (Maker CDPs)
- Open vault types: ETH-A, ETH-B, WBTC-A, etc.
- Each: collateral ratio, stability fee, liquidation ratio, debt ceiling
- The Golem's open vaults with current health

#### Tab 2: DSR (DAI Savings Rate)
- Deposit DAI to earn the DSR
- Current rate, historical rate chart
- Deposit/withdraw form

**Golem Context (F2):** Collateral ratio safety analysis, comparison of Maker stability fees to Aave/Morpho borrow rates, DSR vs. alternative stablecoin yield.

---

## 7. IDENTITY — On-Chain Identity Management

---

### 7.1 ENS (Ethereum Name Service)

**What it is:** Decentralized naming system. Map human-readable names to Ethereum addresses.

**Chains:** Ethereum

**Golem Tools:** `get_ens_name`, `resolve_ens`, `register_ens`, `set_ens_records`, `get_ens_expiry`

#### Tab 1: Lookup
- Search any `.eth` name: resolved address, owner, expiry, records (avatar, URL, social)
- Reverse lookup: paste an address, see if it has an ENS name

#### Tab 2: My Names
- ENS names owned by the Golem/terminal wallet
- Each: name, expiry date (countdown), linked records
- Renew, set records, transfer actions

#### Tab 3: Register
- Register a new `.eth` name
- Availability check, price estimate (based on name length and registration duration)
- Registration flow: commit → wait → reveal (2-step to prevent front-running)

**Golem Context (F2):** ENS name valuation (if the Golem has observed ENS market data), expiry reminders, record completeness check.

---

## 8. DERIVATIVES — Perpetuals and Options

---

### 8.1 GMX

**What it is:** Decentralized perpetual exchange. Trade with leverage (up to 50x) on-chain.

**Chains:** Arbitrum

**Golem Tools:** `get_gmx_markets`, `open_gmx_position`, `close_gmx_position`, `get_gmx_position`, `get_gmx_funding_rate`

#### Tab 1: Trade
- Market selector (ETH-USD, BTC-USD, etc.)
- Long/Short toggle
- Leverage slider (1x-50x)
- Size input, collateral input
- Liquidation price preview
- Funding rate display
- `F8` to execute

#### Tab 2: Positions
- Open perpetual positions: market, direction, size, entry price, mark price, PnL, liquidation price
- Close/modify controls per position

#### Tab 3: Markets
- All available markets with: price, 24h volume, open interest, funding rate
- Price charts per market

**Golem Context (F2):** Leverage risk analysis, funding rate arbitrage assessment, correlation with Golem's spot positions, hedging strategy evaluation.

---

## 9. Protocol-Specific Golem Integrations

### Cross-Protocol Analysis

When the Golem has positions across multiple protocols, the Perspective overlay (F2) can provide **cross-protocol analysis**:

- "Your Aave supply of 10 ETH and your Uniswap LP of 5 ETH have 85% correlated risk"
- "Your Morpho borrow rate (3.2%) is lower than your Aave borrow rate (3.8%) for the same asset"
- "Bridging 2 ETH from Ethereum to Base would let you earn 2.1% more on Aerodrome than your current Uniswap position"

This analysis is generated when F2 is pressed on the Protocol Browser (Sanctum Tab 6) rather than on a specific protocol view — it gives the portfolio-level perspective.

### Tool Profile Matching

Not all Golems have all protocol tools loaded. The Protocol Browser shows which protocols the current Golem can interact with based on its loaded tool profile:

| Profile | Protocols Available |
|---------|-------------------|
| `data` | All (read-only) |
| `trader` | DEXs (Uniswap, Aerodrome, Curve) + data |
| `lp` | DEXs + LP management + data |
| `vault` | All + vault-specific tools |
| `full` | Everything |

Protocols outside the Golem's tool profile are visible but marked as "read-only" — the Golem can view data but cannot execute transactions. The owner can still execute directly via the terminal wallet.

---

## 10. Future Protocols (Deferred)

The following protocols are recognized but deferred for initial implementation:

| Protocol | Category | Reason for Deferral |
|----------|----------|-------------------|
| Bardo Vaults (ERC-4626) | Yield | Pending launch decision |
| Synthetix | Derivatives | Complexity of synth minting |
| Balancer | DEX | Lower priority than Uniswap/Aerodrome/Curve |
| PancakeSwap | DEX | BNB Chain focus, lower relevance |
| Convex | Yield | Primarily Curve boost management |
| Frax | Stablecoin | Smaller market, complex mechanics |
| Hyperliquid | Derivatives | Separate L1, not EVM-compatible in standard sense |

These can be added by extending the `@bardo/tools` adapter layer and adding new Protocol View definitions. The Sanctum architecture is designed to be protocol-additive — each new protocol is a new set of tabs/panes following the same navigation model.

---

## 11. Implementation Priority

### Phase 1: Core (Launch)
1. **Uniswap** — full 4-tab view (Swap, Pools, Positions, Tokens)
2. **Aave** — full 4-tab view (Markets, Supply, Borrow, Positions)
3. **Morpho** — full 4-tab view (Markets, Supply/Borrow, Create Market, Positions)
4. **Aerodrome** — full 4-tab view (Swap, Pools, Positions, Tokens)
5. **Protocol Browser** — the navigation hub

### Phase 2: Expansion
6. **Curve** — 3-tab view
7. **Across** — 2-tab view (Bridge, History)
8. **Compound** — 4-tab view (mirrors Aave)
9. **ENS** — 3-tab view (Lookup, My Names, Register)

### Phase 3: Advanced
10. **Pendle** — 3-tab view
11. **Lido** — 2-tab view
12. **EigenLayer** — 2-tab view
13. **GMX** — 3-tab view
14. **Beefy / Yearn** — 2-tab view
15. **Spark / MakerDAO** — 2-tab view
16. **Stargate / Hop** — 2-tab view
17. **Rocket Pool** — 2-tab view

---

*⌈ every protocol is a world the golem can touch ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
