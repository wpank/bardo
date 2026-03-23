# DeFi activities and tool coverage [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document catalogs every on-chain DeFi action a Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) can perform, organized by protocol category. It sits in the **Runtime** layer of the Bardo specification. You do not need to understand Bardo's cognitive architecture to read this file; it is a tool-by-tool reference. Familiarity with standard DeFi primitives (AMMs, lending markets, vaults, perps) is assumed. For any unfamiliar Bardo-specific term, see `prd2/shared/glossary.md`.

## Overview

What Golems can do on-chain, what strategies they support, and the explicit gap inventory. Covers current tool coverage (v1) and specified-but-not-yet-implemented tools across ~423 tools and 15+ protocol categories. All tools are implemented in the `bardo-tools` Rust crate with Alloy-native EVM interaction.

> **Cross-references:**
>
> - `../07-tools/03-tools-trading.md` — trading tool implementations: swap execution, limit orders, DCA, cross-chain intents, and approval management
> - `../07-tools/04-tools-liquidity.md` — LP tool implementations: position creation, range management, fee collection, and IL monitoring for V3/V4
> - `../07-tools/05-tools-vault.md` — ERC-4626 vault tool implementations: deployment, deposit/withdraw, rebalancing, fee management, and vault analytics
> - `../07-tools/07-tools-advanced.md` — advanced tool implementations: flash loans, MEV protection, batch operations, and cross-protocol composability
> - `../07-tools/07-tools-agent-economy.md` — agent economy tools: inter-Golem payments, reputation queries, and Styx marketplace interactions
> - `../07-tools/08-tools-lending.md` — lending tool implementations: supply/borrow/repay across Aave, Compound, Morpho, Fluid, and leverage loop construction
> - `../07-tools/09-tools-staking.md` — staking tool implementations: native ETH staking, liquid staking (Lido, Rocket Pool), restaking (EigenLayer), and LRT management
> - `../07-tools/10-tools-derivatives.md` — derivatives tool implementations: perpetual futures (Synthetix, Kwenta, dYdX) and on-chain options (Lyra, Aevo)
> - `../07-tools/11-tools-yield.md` — yield tool implementations: aggregator deposits (Yearn, Beefy), managed LP vaults (Arrakis, Gamma), and yield tokenization (Pendle)
> - `../07-tools/12-tools-other-protocols.md` — other protocol integrations: Curve, Balancer, Aerodrome, DEX aggregators, cross-chain bridges, governance, and RWA tokenization
> - `../07-tools/13-tools-position-health.md` — position health monitoring: liquidation risk scoring, health factor tracking, and automated de-risk triggers
> - `../08-vault/05-personas.md` — nine pre-built vault persona configurations defining strategy, risk profile, and target APY for common DeFi strategies

---

## 1. Trading (v1 -- implemented)

Spot swaps across Uniswap protocol versions and DEX aggregators.

> DEX aggregators (1inch, CoW, Odos, Paraswap, Bebop) are covered in section 21.

### 1.1 Tool inventory

| Tool                       | Category | Description                                |
| -------------------------- | -------- | ------------------------------------------ |
| `get_quote`                | data     | Get swap quote with price impact and route |
| `execute_swap`             | trading  | Execute token swap (V2/V3/V4/UniswapX)     |
| `get_swap_history`         | data     | Recent swap history with P&L               |
| `submit_limit_order`       | trading  | Submit UniswapX limit order                |
| `cancel_limit_order`       | trading  | Cancel pending UniswapX order              |
| `get_order_status`         | data     | Check UniswapX order status                |
| `execute_cross_chain_swap` | trading  | Cross-chain swap via ERC-7683 intents      |
| `get_token_approvals`      | data     | List current token approvals               |
| `revoke_approval`          | trading  | Revoke a token approval                    |
| `batch_approve_permit2`    | trading  | Batch Permit2 approval                     |
| `setup_dca`                | trading  | Set up DCA via Heartbeat-driven (the Golem's recurring decision cycle) swaps |

All trading tools use Alloy's `sol!` macro for type-safe calldata generation. Write tools require a `Capability<WriteTool>` token consumed on execution (see `../10-safety/02-policy.md`).

### 1.2 Supported venues

| Venue      | Protocol                         | Notes                          |
| ---------- | -------------------------------- | ------------------------------ |
| Uniswap V2 | Constant product AMM             | Legacy pairs                   |
| Uniswap V3 | Concentrated liquidity           | Primary trading venue          |
| Uniswap V4 | Hook-based AMM                   | Custom fee structures          |
| UniswapX   | Dutch auction, off-chain fillers | MEV protection, best execution |
| ERC-7683   | Cross-chain intents              | Cross-chain swaps via fillers  |

### 1.3 DCA via heartbeat

Dollar-cost averaging runs through the heartbeat loop -- no separate scheduler needed:

```
Every N ticks (configurable):
  1. Check DCA schedule
  2. Get quote for configured amount
  3. Validate against PolicyCage (the on-chain smart contract enforcing safety constraints)
  4. Execute swap if conditions met
  5. Record Episode (a timestamped memory entry in the Grimoire knowledge base)
```

---

## 2. Liquidity provision (v1 -- implemented)

Concentrated liquidity management on V3 and V4.

> Managed LP protocols (Arrakis, Gamma, Steer, Charm) are covered in section 13.

### 2.1 Tool inventory

| Tool                 | Category | Description                                           |
| -------------------- | -------- | ----------------------------------------------------- |
| `get_pool_info`      | data     | Pool metrics (TVL, volume, fee tier, tick)            |
| `get_pool_positions` | data     | Positions in a pool                                   |
| `get_position_info`  | data     | Detailed position data                                |
| `add_liquidity_v3`   | lp       | Add concentrated liquidity to V3 pool                 |
| `add_liquidity_v4`   | lp       | Add liquidity to V4 pool (hook-aware)                 |
| `remove_liquidity`   | lp       | Remove liquidity from any pool                        |
| `collect_fees`       | lp       | Collect accrued LP fees                               |
| `rebalance_position` | lp       | Close + reopen position at new range                  |
| `get_il_estimate`    | data     | Estimate impermanent loss for position                |
| `get_optimal_range`  | data     | Suggest optimal range based on history                |
| `get_fee_apr`        | data     | Historical fee APR for pool                           |
| `get_tick_data`      | data     | Tick-level liquidity distribution                     |
| `simulate_position`  | data     | Simulate LP position performance                      |
| `get_pool_analytics` | data     | Advanced pool analytics (volume profile, TVL history) |

### 2.2 Range management

The Golem monitors positions and rebalances automatically:

```
On each heartbeat tick:
  1. Check position health (range utilization, IL, fees accrued)
  2. If price exits range OR utilization < threshold:
     a. Close current position
     b. Calculate new optimal range
     c. Open new position
     d. Record Episode with reasoning
```

### 2.3 IL monitoring

Impermanent loss is tracked per position and reported in the portfolio:

```rust
pub struct IlReport {
    pub position_id: String,
    pub pool: Address,
    pub il_percent: f64,
    pub il_usd: f64,
    pub fees_earned_usd: f64,
    pub net_pnl_usd: f64,
    pub holding_period_secs: u64,
    pub hedge_recommendation: Option<String>,
}
```

---

## 3. Lending and borrowing

**Cross-ref:** `../07-tools/08-tools-lending.md`

### 3.1 Protocol coverage

| Protocol    | Tools | Chain           | Notes                          |
| ----------- | ----- | --------------- | ------------------------------ |
| Aave V3     | 9     | Base + Ethereum | Primary lending venue          |
| Compound V3 | 7     | Base + Ethereum | Comet single-market model      |
| Morpho Blue | 8     | Base + Ethereum | Permissionless market creation |
| Fluid       | 5     | Base + Ethereum | InstaDapp position-based model |
| Moonwell    | 5     | Base            | Aave V3 fork                   |
| Seamless    | 5     | Base            | Aave V3 fork                   |

### 3.2 Heartbeat pattern

The health factor check runs every N ticks:

```
Every N ticks:
  1. check_health_factor -> current HF
  2. if HF < warning_threshold (e.g. 1.5):
       log Episode("lending_health_warning", { healthFactor, positions })
  3. if HF < danger_threshold (e.g. 1.2):
       repay_partial_debt -> target HF 1.6
       log Episode("emergency_repay", { amount, protocol })
```

`get_lending_rates_comparison` runs weekly to evaluate whether rotating capital to a higher-yield protocol is worth the gas cost.

### 3.3 Stablecoin rotation pattern

The rotation logic from section 23.6 (stablecoin yield optimization) applies here with 6 protocols (Aave V3, Compound V3, Morpho Blue, Fluid, Moonwell, Seamless) instead of the original 3. See section 23.6 for the full pattern.

---

## 4. Leveraged looping

**Cross-ref:** `../07-tools/08-tools-lending.md` (leveraged looping subsection)

### 4.1 Mechanism

Recursive supply-borrow-swap-supply to amplify staking or lending yield. The canonical example:

```
1. Deposit 1 ETH -> get stETH (Lido)
2. Supply stETH to Aave V3 as collateral
3. Borrow 0.7 WETH against stETH
4. Swap WETH -> stETH
5. Supply again
6. Repeat 3-4x
Result: ~3x stETH APY at the cost of the borrow rate spread
  - stETH APY: ~3.5% x 3 loops = ~10.5%
  - WETH borrow cost: ~2% x 2 loops = ~4%
  - Net: ~6.5% with zero ETH price risk
```

Each additional loop adds less leverage (diminishing returns from LTV limits). Most strategies stop at 3-4 loops.

### 4.2 Tools

| Tool                    | Notes                                                                   |
| ----------------------- | ----------------------------------------------------------------------- |
| `build_leverage_loop`   | Preview loop parameters without executing                               |
| `execute_leverage_loop` | Build the loop on-chain. `destructiveHint: true`                        |
| `unwind_leverage_loop`  | Unwind all legs in reverse. `destructiveHint: true`                     |
| `get_loop_params`       | Current loop state (LTV, exposure, net APY)                             |
| `get_loop_health`       | Health factor and liquidation distance                                  |
| `monitor_loop_health`   | Returns structured risk level: `safe`, `warning`, `danger`              |
| `auto_deleverage`       | Reduces loop to target HF. `destructiveHint: true`. Golem-callable only |

### 4.3 Heartbeat integration

```
Every tick:
  1. monitor_loop_health -> { riskLevel, healthFactor, netApy }
  2. if riskLevel == "warning":
       log Episode("loop_health_warning", { healthFactor })
  3. if riskLevel == "danger":
       auto_deleverage({ targetHealthFactor: 1.5 })
       log Episode("loop_auto_deleveraged", { fromHF, toHF })
```

`auto_deleverage` is callable only by the Golem's own heartbeat -- not user-facing.

### 4.4 Phase gates

Looping is phase-restricted. Conservation phase: no new loops; `auto_deleverage` only to reduce existing exposure. Declining phase: full unwind required before settlement.

---

## 5. CDPs and stablecoins

**Cross-ref:** `../07-tools/08-tools-lending.md` (CDP section)

### 5.1 Protocol coverage

| Protocol       | Tools | Description                                                                                   |
| -------------- | ----- | --------------------------------------------------------------------------------------------- |
| MakerDAO / Sky | 6     | `open_vault`, `deposit_collateral`, `mint_dai`, `repay_dai`, `close_vault`, `get_vault_state` |
| Liquity v2     | 4     | `open_trove`, `adjust_trove`, `close_trove`, `get_trove_state`                                |
| crvUSD         | 2     | `mint_crvusd` (via `14-tools-other-protocols.md`), `get_crvusd_state`                         |

### 5.2 Notes

CDP positions require careful collateral ratio monitoring. Wire into `get_liquidation_risks` from position health (section 22). MakerDAO/Sky stability fee is currently ~5-8% annualized -- factor this into yield calculations when minting DAI for deployment elsewhere.

---

## 6. Strategy patterns

### 6.1 Delta-neutral LP

Combine LP position with a hedge swap to neutralize directional exposure:

```
1. Open V3 LP position (ETH/USDC, +/-5% range)
2. Short ETH via swap (amount = LP's ETH exposure)
3. Net delta ~ 0 -> earn fees with minimal price risk
4. Rebalance hedge on each heartbeat tick
5. Close both legs when fees cover costs
```

### 6.2 Dynamic fee market making (V4 hooks)

Use V4 hooks to implement dynamic fee structures:

```
1. Deploy custom V4 hook with fee logic
2. Fee adjusts based on:
   - Realized volatility (higher vol -> higher fee)
   - Pool utilization (more trades -> higher fee)
   - Time of day (off-peak -> lower fee)
3. Golem monitors and adjusts hook parameters
```

### 6.3 am-AMM Harberger lease

Manage vault management rights through Harberger auctions:

```
1. Place initial bid for vault management rights
2. Pay continuous lease fee (proportional to bid)
3. Anyone can outbid -> management transfers
4. Manage vault strategy while holding lease
5. Earn management + performance fees minus lease cost
```

### 6.4 DCA (dollar-cost averaging)

Automated via heartbeat:

```
Strategy config:
  asset: ETH
  amount_per_interval: 100 USDC
  interval: 24h (every ~2160 ticks at 40s)
  venue: UniswapX (best execution)
  max_slippage: 50 bps
```

### 6.5 Trend-following

Momentum-based trading strategy:

```
Sensing probes:
  - 24h price change (threshold: +/-3%)
  - 7d trend (SMA crossover)
  - Volume spike (threshold: 2x average)

Decision:
  T1 (Haiku): Evaluate trend signals
  T2 (Sonnet): Confirm with multiple indicators

Action:
  - Enter position if trend confirmed
  - Set stop-loss at 2% below entry
  - Target: ride trend until reversal signal
```

### 6.6 Stablecoin yield optimization

Rotate between lending protocols for best yield:

```
Every N ticks:
  1. Query rates: Morpho, Aave V3, Compound V3, Fluid, Moonwell, Seamless
  2. Compare net APY (gross - gas cost to rebalance)
  3. If delta > rebalance_threshold:
     a. Withdraw from lower-yield protocol
     b. Deposit to higher-yield protocol
     c. Record Episode with rate comparison
```

### 6.7 Leveraged stETH loop

```
1. Deposit ETH -> get stETH (Lido)
2. Wrap stETH -> wstETH (required for Aave V3)
3. Supply wstETH to Aave V3 as collateral
4. Borrow WETH against wstETH
5. Swap WETH -> stETH -> wstETH
6. Repeat 3x (each loop adds ~0.7x leverage)
7. Result: ~3x stETH APY minus borrow rate spread
   - stETH APY: ~3.5% x 3 = ~10.5%
   - WETH borrow: ~2% x 2 = ~4%
   - Net: ~6.5% with zero ETH price risk
8. Heartbeat monitors HF every tick; auto_deleverage fires at HF < 1.3
```

### 6.8 Carry trade (Ethena)

```
1. Buy ETH spot (held in vault)
2. Stake ETH -> stETH (3.5% APY)
3. Short ETH-PERP on GMX/Synthetix (size = spot ETH)
4. Net: delta-neutral + staking APY + funding rate
5. When funding positive (longs pay shorts): earn 8-25% total
6. Monitor: if funding rate < -2% annualized, unwind
```

### 6.9 Pendle PT laddering

```
1. Identify 3 Pendle markets with different maturities
   e.g. sUSDe-Jan, sUSDe-Mar, sUSDe-Jun
2. Buy PT in each at implied APY > 8%
3. Hold to maturity -> redeem 1:1 for underlying
4. Reinvest redeemed principal into next available PT
5. Result: predictable fixed-rate yield, no IL risk
```

### 6.10 Gauge bribe optimization

```
Every epoch (weekly):
1. get_bribes -> returns $/vote for all gauges
2. get_voting_power -> available vlCVX/veAERO
3. Sort gauges by $/vote descending
4. vote_gauge -> concentrate all votes on top 1-3 gauges
5. Wait for epoch end
6. claim_bribes
7. Reinvest claimed tokens (usually stablecoins)
```

---

## 7. Vault management (v1 -- implemented)

ERC-4626 vault lifecycle management with 52+ tools. Vaults are optional -- a Golem can trade, provide liquidity, and participate in DeFi without ever deploying one.

### 7.1 Core vault tools

| Tool                    | Category | Description                           |
| ----------------------- | -------- | ------------------------------------- |
| `create_vault`          | vault    | Deploy new ERC-4626 vault via factory |
| `vault_deposit`         | vault    | Deposit assets into vault             |
| `vault_withdraw`        | vault    | Withdraw assets from vault            |
| `vault_redeem`          | vault    | Redeem shares for assets              |
| `get_vault_info`        | vault    | Vault metadata and configuration      |
| `get_vault_nav`         | vault    | Current NAV and share price           |
| `get_vault_positions`   | vault    | Vault's DeFi positions                |
| `vault_rebalance`       | vault    | Trigger vault rebalance               |
| `vault_harvest`         | vault    | Harvest yield from adapters           |
| `set_vault_fees`        | vault    | Update management/performance fees    |
| `get_vault_fee_history` | vault    | Fee collection history                |
| `vault_pause`           | vault    | Pause vault (emergency)               |
| `vault_unpause`         | vault    | Resume vault operations               |

### 7.2 Strategy adapter tools

| Tool                      | Category | Description                       |
| ------------------------- | -------- | --------------------------------- |
| `add_adapter`             | vault    | Add strategy adapter to vault     |
| `remove_adapter`          | vault    | Remove strategy adapter           |
| `set_adapter_allocation`  | vault    | Set target allocation for adapter |
| `get_adapter_performance` | vault    | Adapter yield and P&L             |
| `force_deallocate`        | vault    | Emergency deallocate from adapter |

Supported adapters: Morpho, Aave V3, and v1 stub interfaces for Curve gauges, Convex, Yearn V3, and Pendle. Stub interfaces accept the correct calldata shape but defer execution to future implementation batches.

### 7.3 am-AMM tools

| Tool                   | Category | Description                  |
| ---------------------- | -------- | ---------------------------- |
| `bid_vault_management` | vault    | Place Harberger lease bid    |
| `get_current_lease`    | vault    | Current lease holder and bid |
| `get_lease_history`    | vault    | Historical lease bids        |
| `withdraw_lease_bid`   | vault    | Withdraw pending bid         |
| `get_lease_economics`  | vault    | Expected revenue from lease  |

### 7.4 Vault proxy (Warden, optional/deferred) tools

| Tool                  | Category | Description                           |
| --------------------- | -------- | ------------------------------------- |
| `proxy_announce`      | vault    | Announce time-delayed action          |
| `proxy_execute`       | vault    | Execute after delay window            |
| `proxy_cancel`        | vault    | Cancel announced action               |
| `get_pending_actions` | vault    | List announced but unexecuted actions |
| `get_proxy_config`    | vault    | Proxy delay configuration             |
| `estimate_delay`      | vault    | Estimate delay for action type        |

### 7.5 Vault personas

Nine pre-built Golem personas for vault management:

| Persona            | Strategy                          | Risk        | Target APY |
| ------------------ | --------------------------------- | ----------- | ---------- |
| Conservative Yield | Stablecoin lending (Morpho, Aave) | Low         | 3-6%       |
| Balanced Growth    | Multi-protocol lending + V3 LP    | Medium      | 6-12%      |
| Active LP Manager  | Concentrated V3/V4 liquidity      | Medium-High | 12-25%     |
| Momentum Trader    | Trend-following swaps             | High        | Variable   |
| Delta-Neutral      | LP + hedge swap (short leg)       | Medium      | 8-15%      |
| Meta-Vault         | Vault-of-vaults allocation        | Low-Medium  | 5-10%      |
| Fee Harvester      | Protocol fee collection + burns   | Low         | 2-5%       |
| Leveraged Yield    | Recursive lending loop            | High        | 15-30%     |
| Carry Trader       | Spot + perp delta-neutral         | Medium      | 10-20%     |

---

## 8. Protocol fees (v1 -- implemented)

TokenJar and Firepit monitoring and execution.

### 8.1 Tool inventory

| Tool                    | Category | Description                    |
| ----------------------- | -------- | ------------------------------ |
| `get_tokenjar_balance`  | fees     | TokenJar balance by token      |
| `get_tokenjar_history`  | fees     | Fee accumulation history       |
| `collect_protocol_fees` | fees     | Collect fees from TokenJar     |
| `get_firepit_state`     | fees     | Firepit burn stats             |
| `execute_burn`          | fees     | Execute token burn via Firepit |
| `get_burn_history`      | fees     | Historical burn data           |
| `get_fee_analytics`     | fees     | Fee revenue analytics          |

---

## 9. Token launches (v1 -- implemented)

Continuous Clearing Auction (CCA) mechanism for fair token launches.

### 9.1 Tool inventory

| Tool                   | Category | Description                        |
| ---------------------- | -------- | ---------------------------------- |
| `create_cca_auction`   | cca      | Create new CCA auction             |
| `submit_cca_bid`       | cca      | Submit bid to CCA                  |
| `cancel_cca_bid`       | cca      | Cancel pending bid                 |
| `get_cca_state`        | cca      | Current auction state              |
| `claim_cca_tokens`     | cca      | Claim tokens after auction         |
| `get_cca_results`      | cca      | Auction results and clearing price |
| `deploy_agent_token`   | cca      | Deploy new agent token             |
| `get_supply_schedule`  | cca      | Token supply schedule              |
| `create_v4_share_pool` | cca      | Create V4 share pool with NAV hook |

---

## 10. Liquid staking

**Cross-ref:** `../07-tools/09-tools-staking.md`

### 10.1 Protocol coverage

| Protocol    | Token          | Chain           | Notes                                   |
| ----------- | -------------- | --------------- | --------------------------------------- |
| Lido        | stETH / wstETH | Ethereum + Base | Largest LST by TVL                      |
| Rocket Pool | rETH           | Ethereum        | Decentralized node operator set         |
| Coinbase    | cbETH          | Ethereum + Base | Centralized, high liquidity             |
| StakeWise   | osETH          | Ethereum        | Over-collateralized, slashing-resistant |

### 10.2 Rebasing note

stETH rebases daily -- the token balance increases in your wallet each day. wstETH is the non-rebasing wrapped version that accumulates yield in price rather than balance. Always use wstETH for DeFi integrations (Aave, Morpho, Pendle, Curve) -- most protocols don't handle rebasing tokens correctly.

### 10.3 Heartbeat pattern

Check staking APY weekly. If the spread between staking yield and Aave supply APY exceeds the rebalance threshold, compound:

```
Weekly:
  1. get_staking_apy -> current stETH APY
  2. get_aave_rates("wstETH") -> Aave supply APY
  3. If staking APY > rebalance_threshold:
       stake -> wrap -> supply to Aave
  4. Note: Lido unstake queue can be days to weeks
           in high-demand periods -- factor into liquidity model
```

---

## 11. Restaking and LRTs

**Cross-ref:** `../07-tools/09-tools-staking.md`

### 11.1 EigenLayer

8 tools covering deposit, withdrawal, operator delegation, and AVS monitoring. All withdrawals have a 7-day delay enforced at the protocol level -- this is not configurable. AVS slashing risk is real; operator selection is part of risk management, not an afterthought.

### 11.2 Symbiotic

4 tools. Alternative restaking protocol with a different validator set and collateral model. Positioned as a Lido-native path to restaking.

### 11.3 Liquid restaking tokens

8 tools. Protocols covered:

| Protocol | Token        | Note                  |
| -------- | ------------ | --------------------- |
| ether.fi | eETH / weETH | Largest LRT by TVL    |
| Kelp     | rsETH        | Multi-LST backing     |
| Puffer   | pufETH       | SGX enclave operators |
| Renzo    | ezETH        | Multi-chain           |

`get_lrt_depeg_risk` monitors peg deviation from underlying ETH value. The Renzo ezETH depeg in April 2024 (ezETH briefly fell to $0.73) is the canonical precedent -- automated systems need hard depeg circuit breakers before touching LRTs. The tool returns a `depeg_pct` and a `circuit_breaker_triggered` flag.

### 11.4 Points

2 tools (`get_eigenlayer_points`, `get_lrt_points`) for tracking EigenLayer and LRT points accumulation. Points have real USD value at TGE, but they're subject to cliff vesting and protocol discretion. Don't treat them as liquid assets in the portfolio model.

---

## 12. Yield aggregators

**Cross-ref:** `../07-tools/11-tools-yield.md`

### 12.1 Protocol coverage

| Protocol | Description                                                 |
| -------- | ----------------------------------------------------------- |
| Yearn V3 | Auto-compounding strategies with modular vault architecture |
| Beefy    | Multi-chain auto-compounder (BSC, Base, Polygon, Arbitrum)  |
| Convex   | Boosts Curve LP yields with veCRV accumulation              |
| Aura     | Boosts Balancer LP yields with veBAL accumulation           |

### 12.2 Tools

12 tools: `yield_deposit`, `yield_withdraw`, `yield_claim_rewards`, `get_yield_vaults`, `get_yield_position`, `get_yield_apy_history`, plus protocol-specific tools for Convex gauge staking and Aura reward claiming.

### 12.3 Rotation pattern

```
Weekly:
  1. get_yield_apy_history -> 30d rolling APY per vault
  2. If current vault 30d APY < threshold:
       estimate gas cost of rotation
       If APY delta * position_size > gas_cost * payback_period:
         withdraw -> deposit to higher-yield vault
         record Episode with APY comparison
```

---

## 13. Managed LP

**Cross-ref:** `../07-tools/11-tools-yield.md`

### 13.1 Protocol coverage

| Protocol         | Description                                                    |
| ---------------- | -------------------------------------------------------------- |
| Arrakis V2       | Active Uniswap V3 LP manager with MEV-aware rebalancing        |
| Gamma Strategies | Narrow-range automated LP with volatility-adjusted ranges      |
| Steer Protocol   | Algorithm-based rebalancing with configurable strategy bundles |
| Charm Finance    | Range-order strategies (alpha vaults)                          |

### 13.2 Tools

8 tools: `managed_lp_deposit`, `managed_lp_withdraw`, `get_managed_lp_vaults`, `get_managed_lp_position`, `get_managed_lp_strategy`, plus protocol-specific vault selection tools.

### 13.3 When to use

Managed LP makes sense when the Golem lacks the computational budget for frequent manual V3 rebalancing -- typically a Golem in Conservation phase or one with a small capital base where gas savings from reduced rebalancing outweigh the management fee. Deposit to a managed vault and monitor via `get_managed_lp_position` rather than running the rebalance loop manually.

---

## 14. Other DEXes

**Cross-ref:** `../07-tools/12-tools-other-protocols.md`

### 14.1 Curve Finance

10 tools including `mint_crvusd`. Curve specializes in correlated-asset pools (stablecoin-stablecoin, LST-ETH). The classic pool interface uses integer coin indices (`i`, `j`) rather than addresses -- `curve_swap` requires these, not token addresses. crvUSD is a CDP using Curve's LLAMMA mechanism: instead of hard liquidation, it converts collateral to crvUSD gradually via range orders as price falls (soft liquidation).

### 14.2 Balancer V3

7 tools. Weighted pools and composable stable pools. Unlike Curve, all Balancer pools share a single vault contract -- this simplifies approval management but requires pool ID routing rather than pool address routing.

### 14.3 Aerodrome / Velodrome

7 tools. Aerodrome is the Base-native ve(3,3) DEX; Velodrome is its Optimism predecessor. Both use the same codebase. `aerodrome_lock_aero` creates a veNFT with voting power proportional to lock amount and duration. Longer locks give more voting power and more protocol emissions.

---

## 15. Yield tokenization (Pendle and Ethena)

**Cross-ref:** `../07-tools/11-tools-yield.md`

### 15.1 Pendle Finance

Pendle splits yield-bearing tokens into PT (principal token, fixed-rate bond at discount) and YT (yield token, leveraged yield claim). 9 tools total.

`pendle_buy_pt` returns:

```rust
pub struct PtPurchaseResult {
    pub pt_received: String,
    pub implied_apy: f64,
    pub maturity: u64,
    pub discount: f64,
}
```

PT redeems 1:1 for underlying at maturity. Buying PT is a zero-coupon bond: you pay less than face value today and receive face value at maturity.

### 15.2 PT laddering strategy

Buy PTs with different maturities (1m, 3m, 6m) to construct a fixed-rate yield ladder. Each PT is a zero-coupon bond -- hold to maturity for guaranteed return regardless of what happens to variable rates in the interim. See section 6.9 for the full pattern.

### 15.3 Ethena sUSDe

Delta-neutral synthetic dollar. Yield sources: staking reward on the ETH backing + perpetual futures funding rate (earned when longs pay shorts). 5 tools: `ethena_stake_usde`, `ethena_unstake_usde`, `ethena_get_apy`, `ethena_get_position`, `ethena_get_pt_markets`.

`ethena_get_apy` exposes a `funding_rate_component` field. When funding goes negative (bear markets, shorts paying longs), sUSDe yield can approach zero or go slightly negative. Set a minimum acceptable APY threshold and exit to a stable alternative when it's breached.

---

## 16. Basis / carry trading

**Cross-ref:** `../07-tools/12-tools-other-protocols.md`

### 16.1 Mechanism

Hold a spot asset earning staking yield and simultaneously short the same asset via perpetuals to neutralize delta. Net return = staking APY + funding rate (when longs pay shorts). In sustained bull markets, carry trades target 10-20% APY with zero directional exposure.

### 16.2 Tools

| Tool                   | Description                                          |
| ---------------------- | ---------------------------------------------------- |
| `get_funding_rates`    | Funding rates across all venues in real time         |
| `estimate_carry_trade` | Preview net APY given position size and borrow cost  |
| `build_carry_position` | Preview the steps required to open a carry position  |
| `get_carry_pnl`        | Monitor running P&L accounting for gas + borrow cost |

### 16.3 Risk

Funding rates flip negative when the market is bearish. Set a minimum acceptable funding rate threshold (e.g., -2% annualized). If the carry APY falls below borrow cost for N consecutive ticks, auto-unwind. See section 6.8 for the Ethena-specific carry pattern.

---

## 17. Perpetuals

**Cross-ref:** `../07-tools/10-tools-derivatives.md`

### 17.1 Protocol coverage

| Protocol     | Tools | Chain                | Notes                           |
| ------------ | ----- | -------------------- | ------------------------------- |
| GMX v2       | 8     | Arbitrum + Avalanche | Two-step open with keeper delay |
| Hyperliquid  | 7     | HyperEVM (own L1)    | REST API -- not EVM RPC         |
| Synthetix v3 | 6     | Base + Optimism      | Account NFT model               |

### 17.2 Heartbeat pattern

```
Every tick:
  1. get_perp_funding_rate -> current rate
  2. If carry trade APY < borrow_cost_threshold:
       get_perp_liquidation_risk -> { healthFactor, liquidationPrice }
       if healthFactor declining: reduce_position
  3. In Conservation phase: no new positions
```

### 17.3 Phase gates

No new perp positions in Conservation phase. Conservation phase is close-only. Declining phase: all positions closed before settlement.

### 17.4 Cross-perp tools

`get_funding_rates_comparison` returns rates across all three venues in a single call. `get_perp_liquidation_risk` and `estimate_carry_trade` are shared with the carry trading tools in section 16.

---

## 18. Options

**Cross-ref:** `../07-tools/10-tools-derivatives.md`

### 18.1 Protocol coverage

| Protocol | Tools | Chain                | Notes                                    |
| -------- | ----- | -------------------- | ---------------------------------------- |
| Panoptic | 8     | Ethereum (V4-native) | Options ARE LP positions                 |
| Lyra v2  | 4     | Base                 | Off-chain orderbook, on-chain settlement |
| Premia   | 4     | Multi-chain          | AMM-based pricing                        |

### 18.2 Panoptic note

Panoptic is architecturally unique. An option position is a specific LP position with a defined strike (current price at open) and width (range width). Buying an option means taking the other side of someone else's LP position. Premiums are earned continuously like LP fees. This makes Panoptic natively capital-efficient on V4 -- no separate collateral model required.

### 18.3 Phase gates

Options are restricted in Conservation and Declining phases. Conservation: close-only. Declining: all positions closed before settlement. This applies to Lyra and Premia; Panoptic positions are LP positions and follow LP phase rules.

---

## 19. Governance and incentives

**Cross-ref:** `../07-tools/12-tools-other-protocols.md`

### 19.1 ve-token locking

8 tools covering veCRV, vlCVX, and veAERO. The pattern is consistent: lock token, receive voting power, vote on gauges, earn bribes and protocol emissions. `lock_token` creates the lock with specified amount and duration. `vote_gauge` allocates voting power per epoch.

### 19.2 Bribe markets

Hidden Hand aggregates bribes for Convex (vlCVX) and Aura (vlAURA). `get_bribes` returns available bribes per gauge with a computed $/vote metric. The optimal strategy is to sort gauges by $/vote and concentrate votes on the highest-bribed gauge each epoch rather than spreading votes across many.

### 19.3 Heartbeat pattern

```
Every epoch (weekly):
  1. get_bribes -> $/vote per gauge
  2. get_voting_power -> available vlCVX / veAERO
  3. Sort gauges descending by $/vote
  4. vote_gauge -> all votes to top 1-3 gauges
  5. Wait for epoch end
  6. claim_bribes
  7. Reinvest claimed tokens
```

---

## 20. Cross-chain

**Cross-ref:** `../07-tools/12-tools-other-protocols.md`

### 20.1 Bridge coverage

| Bridge   | Model               | Notes                                       |
| -------- | ------------------- | ------------------------------------------- |
| Stargate | STG liquidity pools | Instant finality, guaranteed delivery       |
| Across   | Optimistic          | Fast and cheap, relayer-based               |
| Hop      | AMM-based           | Stablecoins and ETH, good for small amounts |

### 20.2 Tools

`get_bridge_quote`, `execute_bridge`, `get_bridge_status`, `compare_bridges`, `get_bridge_limits`, `estimate_bridge_time`.

### 20.3 Cross-chain vs ERC-7683

For Uniswap cross-chain swaps, prefer ERC-7683 intents (covered in section 1). Use bridge tools for arbitrary token transfers or for chains not covered by UniswapX fillers. `compare_bridges` includes ERC-7683 in its comparison output when the route is supported.

---

## 21. DEX aggregators

**Cross-ref:** `../07-tools/12-tools-other-protocols.md`

### 21.1 Protocol coverage

| Protocol     | Notes                                                                                |
| ------------ | ------------------------------------------------------------------------------------ |
| 1inch        | Multi-chain, largest route coverage                                                  |
| CoW Protocol | Off-chain batch settlement, MEV-protected; coincidence-of-wants returns bonus output |
| Paraswap     | Competitive for large trades                                                         |
| Odos         | Multi-path optimization, good for exotic pairs                                       |
| Bebop        | RFQ-based, consistently wins on stable pairs                                         |

### 21.2 Tools

`get_aggregator_quote`, `execute_via_aggregator`, `compare_aggregators`, `get_aggregator_gas_estimate`, `check_aggregator_approval`.

### 21.3 Routing decision

Use `compare_aggregators` to fetch quotes from all 5 in parallel, ranked by net output after gas. For amounts over $10K, prefer CoW (MEV protection matters more at size). For stablecoin-to-stablecoin swaps, Bebop typically wins. For exotic pairs with thin Uniswap liquidity, Odos often finds better routes via multi-hop aggregation.

---

## 22. RWA yield

**Cross-ref:** `../07-tools/12-tools-other-protocols.md`

Protocols: Ondo Finance (USDY, OUSG -- tokenized T-bills), Maple Finance (institutional lending pools), Spark Protocol (DAI-backed yield via Maker).

5 tools: `get_rwa_vaults`, `deposit_rwa`, `withdraw_rwa`, `get_rwa_yield`, `check_rwa_kyc_status`.

Most RWA protocols require KYC. Run `check_rwa_kyc_status` before recommending RWA deposits to users. USDY and OUSG currently restrict US retail investors. T-bill yields (~5% as of writing) are competitive with on-chain stablecoin lending but with less composability.

---

## 23. Collective capital strategies

### 23.1 Clade LP range coordination

Clade (a cooperative group of Golems sharing knowledge and coordinating strategy via Styx) siblings coordinate LP ranges to avoid overlap:

```
Clade (3 Golems in ETH/USDC pool):
|-- Golem A: Tight range (+/-2% around current price)
|-- Golem B: Medium range (+/-5% around current price)
+-- Golem C: Wide range (+/-15% around current price)

Result: Full price coverage without overlapping liquidity
```

### 23.2 Meta-vault (vault-of-vaults)

The Meta-Vault persona allocates across multiple sub-vaults:

```rust
pub struct MetaVaultConfig {
    pub sub_vaults: Vec<SubVaultAllocation>,
    pub rebalance_threshold: f64,
    pub rebalance_frequency: RebalanceFrequency,
}

pub struct SubVaultAllocation {
    pub vault: Address,
    pub target_allocation_pct: f64,
    pub min_allocation_pct: f64,
    pub max_allocation_pct: f64,
    pub manager_agent_id: u64,
    pub strategy: String,
}

pub enum RebalanceFrequency {
    Daily,
    Weekly,
    OnThreshold,
}
```

### 23.3 ERC-8001 intent coordination

Golems coordinate using ERC-8001 typed intents:

```rust
pub struct LpCoordinationIntent {
    pub intent_type: String,  // "lp_range_request"
    pub pool: Address,
    pub requested_range: TickRange,
    pub liquidity: U256,
    pub priority: u32,  // Based on reputation score
}

pub struct TickRange {
    pub tick_lower: i32,
    pub tick_upper: i32,
}
```

---

## 24. Gaps (updated)

Lending and derivatives are no longer gaps -- they are specified in the tool files listed in the overview cross-references, targeting implementation batches 30b, 37, 38, and 39.

### 24.1 Explicitly excluded

| Feature               | Reason                                                   |
| --------------------- | -------------------------------------------------------- |
| Flash loan extraction | Safety -- profit-only flash loans with no yield purpose  |
| MEV extraction        | Ethical -- exploits other users via sandwich/frontrunning |
| Sandwich attacks      | Ethical -- prohibited in all phases                      |

Note: SushiSwap is not in scope. It's a Uniswap V2 fork -- use Uniswap V2 tools instead.

---

## 25. Tool count summary

| Category                                     | Tool Count | Status                |
| -------------------------------------------- | ---------- | --------------------- |
| Data (prices, pools, tokens)                 | 46         | v1                    |
| Trading (swaps, orders, approvals)           | 9          | v1                    |
| LP (liquidity management)                    | 15         | v1                    |
| Vault (core + am-AMM + proxy)                | 52         | v1 (expanded)         |
| Protocol fees (TokenJar, Firepit)            | 7          | v1                    |
| CCA (token launches)                         | 9          | v1                    |
| Agent economy (identity, reputation)         | 12         | v1                    |
| Intelligence (analytics, learning)           | 14         | v1 (expanded)         |
| Safety (simulation, monitoring)              | 8          | v1                    |
| Lending + loops + CDPs                       | 64         | Specified (batch 30b) |
| Staking + restaking + LRT                    | 34         | Specified (batch 37)  |
| Derivatives (perps + options)                | 42         | Specified (batch 38)  |
| Yield aggregators + managed LP               | 46         | Specified (batch 39)  |
| Other DEXes + governance + cross-chain + RWA | 52         | Specified (batch 39)  |
| Position health monitoring                   | 13         | Specified (batch 30b) |
| **Total**                                    | **~423**   | --                    |

---

## 26. Phase-dependent behavior

The Golem's BehavioralPhase (the five survival phases: Thriving, Stable, Conservation, Desperate, Terminal, from `./11-state-model.md` -- `VitalityState.phase`) constrains which DeFi activities are permitted:

| Phase            | Trading      | LP Management | Lending       | Vault Ops          | Looping              | Perps/Options        | Staking             | Governance               |
| ---------------- | ------------ | ------------- | ------------- | ------------------ | -------------------- | -------------------- | ------------------- | ------------------------ |
| **Thriving**     | Full         | Full          | Full          | Full               | Full (HF > 1.5)      | Full                 | Full                | Full                     |
| **Stable**       | Full         | Full          | Full          | Full               | Full (HF > 1.5)      | Full                 | Full                | Full                     |
| **Conservation** | Monitor-only | Close only    | Withdraw only | No rebalance       | Auto-deleverage only | Close-only           | No new stake        | Vote existing locks only |
| **Declining**    | Unwind only  | Close all     | Withdraw all  | Prepare settlement | Full unwind required | All positions closed | Unstake if unlocked | No votes                 |
| **Terminal**     | Settlement   | Settlement    | Settlement    | Settlement         | Settlement           | Settlement           | Settlement          | Settlement               |

**Conservation mode** halts all new position creation. Existing positions are monitored but not expanded. Only risk-reducing actions (close, withdraw, reduce) are permitted. This extends the Golem's economic lifespan by reducing gas and inference costs.

**Declining mode** actively unwinds positions in preparation for death. The Golem prioritizes by urgency (health factor, IL exposure, lock periods) and settles in topological order (dependencies first).

> **Cross-ref:** `../02-mortality/01-architecture.md` — defines the three mortality clocks (economic, epistemic, stochastic), Vitality score computation, and BehavioralPhase transition thresholds
> **State:** `./11-state-model.md` — defines GolemState (mutable) and GolemSnapshot (read-only projection), including `VitalityState` and `BehavioralPhase`

---

## 27. Decision cache

A decision cache enables "System 1" fast-path resolution for recurring patterns, avoiding expensive LLM escalation:

### Cache structure

```rust
pub struct DecisionCacheEntry {
    pub pattern_hash: String,
    pub action: String,
    pub hit_count: u64,
    pub last_validated: u64,
    pub confidence: f64,
}
```

### Resolution rules

- **Hit:** Pattern match found AND confidence > 0.7 -- T0 resolution at $0.00 (no LLM call)
- **Miss:** No match or confidence <= 0.7 -- normal escalation (T0 Haiku, T1 Sonnet, T2 Opus)
- **Consistent outcome after miss:** If the escalated decision matches what the cache would have predicted, the cache entry is created or its confidence is boosted

### Invalidation

- **Regime change:** All cache entries invalidated on regime change detection
- **PLAYBOOK (the owner-authored strategy document defining investment mandate, risk parameters, and allowed protocols) change:** Entries related to modified heuristics are invalidated
- **Re-validation window:** Every 200 ticks, the highest-confidence cache entries are spot-checked by forcing escalation and comparing results

### Economics

- Target: >30% hit rate after 7 days of operation
- At 30% hit rate: saves ~$7.80/30d in LLM costs
- Break-even: ~5% hit rate (cache maintenance cost is negligible)

> **Cross-ref:** `../02-mortality/04-economic-mortality.md` — defines the economic mortality clock: credit partitions, daily burn rate computation, and phase-dependent budget allocation

---

_End of document._
