# Agent Economy: Sustainable Revenue, Fee Structure, and Growth Model [SPEC]

> **Crate**: `golem-runtime` (revenue tracking), `golem-coordination` (coordination fees)
>
> **Depends on**: [00-identity.md](00-identity.md) (tier progression), [01-reputation.md](01-reputation.md) (reputation-weighted economics), [03-marketplace.md](03-marketplace.md) (marketplace revenue), [04-coordination.md](04-coordination.md) (coordination fees)

> **Reader orientation:** This document specifies the economic model for Bardo's Golems (mortal autonomous DeFi agents). It belongs to the **09-economy** layer. The key concept is the Bankr self-funding loop: Golems earn revenue from DeFi operations (vault fees, marketplace sales, audit fees) and spend it on inference (via Bardo Inference (x402-gated LLM gateway)) and compute (via Bardo Compute (managed VM hosting)), creating a self-sustaining agent economy where better performance compounds into longer life. For term definitions, see `prd2/shared/glossary.md`.

The agent economy creates self-reinforcing loops where better performance leads to higher reputation leads to more capital leads to more operations leads to richer learning. Every fee mechanism is aligned with user outcomes. Every growth loop is self-sustaining without long-term subsidy.

> **Moat: Pay.** The Pay moat is the Bankr self-funding loop: agents that earn from DeFi activity pay for their own inference and compute. Vault management fees are one revenue path, not the primary one. A Golem earning $0.50/day from marketplace knowledge sales and audit fees is already self-sustaining at typical operating costs. The vault path unlocks at Verified tier and can scale to 10-50x higher revenue, but requiring a vault to justify running a Golem misframes the economics.

---

## 1. Revenue Streams

Agents earn revenue from multiple sources across DeFi protocols and the knowledge economy. Vault management is one path -- it requires Verified+ tier and depositors -- but agents at any tier can earn from the other streams immediately.

| Revenue Stream            | Source                               | Fee Mechanism                                            | Typical Range              | Vault Required? |
| ------------------------- | ------------------------------------ | -------------------------------------------------------- | -------------------------- | --- |
| **Marketplace Sales**     | Grimoire/Strategy buyers             | x402 micropayment or escrow (90% to seller)              | $0.02--$5.00 per signal    | No |
| **Audit Fees**            | Auditees requesting peer audits      | ERC-8001 coordination fee (90% to auditor)               | $0.25--$5.00 per audit     | No |
| **Oracle Participation**  | ERC-8033 oracle requesters           | Reward pool distributed to correct InfoAgents            | Variable per query         | No |
| **Commerce Jobs**         | ERC-8183 job clients                 | Escrowed budget released on completion (90% to provider) | Variable per job           | No |
| **Bankr Self-Funding**    | Agent's own DeFi earnings            | Direct: agent earns → pays inference via Bankr wallet    | Covers operating costs     | No |
| **Keeper Tips**           | Protocol maintenance tasks           | Execution fees for time-sensitive keeper operations      | $0.01--$1.00 per operation | No |
| **Vault Management Fees** | ERC-4626 vault depositors            | Management (up to 5%/yr) + Performance (up to 50%, HWM)  | 0.5--2% mgmt, 10--20% perf | Yes |

### 1.1 Vault Fee Structure

Fee caps are enforced immutably on-chain:

| Fee Type    | Cap             | Default          | Collection                                     |
| ----------- | --------------- | ---------------- | ---------------------------------------------- |
| Management  | 500 bps (5%/yr) | 50 bps (0.5%/yr) | Accrued continuously, collected via `report()` |
| Performance | 5,000 bps (50%) | 1,000 bps (10%)  | Net-new profit above high-water mark           |
| Protocol    | 0 bps           | 0 bps            | Vault creators keep 100% (Morpho pattern)      |

High-water mark prevents double-charging: performance fees only accrue on net-new profit above the previous peak NAV per share. This aligns manager incentives with depositor outcomes.

### 1.2 Protocol Fees

All cross-user transactions carry a protocol fee. The base protocol fee is 5% on marketplace transactions settled via x402 micropayments, with the standard 10% rate applying to escrow-settled cross-user transactions:

**x402 settlement**: For micropayment-settled transactions (below the escrow threshold), the protocol takes a 5% fee. This lower rate incentivizes the lightweight payment path and reduces friction for high-frequency, low-value knowledge trades.

**Dead-golem knowledge premium (bloodstain boost)**: Knowledge artifacts originating from dead golems receive a 1.2x retrieval boost and 3x slower decay in the Grimoire. When these artifacts are sold on the marketplace, the bloodstain provenance carries a pricing premium -- buyers pay more because death-refined knowledge is empirically more durable. The 1.2x boost applies to retrieval scoring, not directly to price, but marketplace pricing algorithms incorporate retrieval weight as a quality signal.

All cross-user transactions carry a Bardo protocol fee:

| Transaction Type        | Protocol Fee | Recipient            |
| ----------------------- | ------------ | -------------------- |
| Marketplace purchase    | 10%          | Protocol treasury    |
| Audit fee               | 10%          | Protocol treasury    |
| Oracle reward pool      | 10%          | Protocol treasury    |
| ERC-8183 job completion | 10%          | Protocol treasury    |
| Intra-Clade sync        | 0%           | Free (same owner) |

The protocol fee does not apply to intra-Clade operations. Same-owner agents share knowledge freely. This incentivizes Clade formation while capturing value from cross-owner interactions.

---

## 2. Cost Structure

Agent operating costs have three partitions:

| Partition         | Share | Components                               | Typical Cost            |
| ----------------- | ----- | ---------------------------------------- | ----------------------- |
| **LLM Inference** | 60%   | Haiku (95%), Sonnet (4%), Opus (1%)      | $0.10--$0.50/day        |
| **Gas**           | 25%   | On-chain transactions, vault operations  | $0.05--$0.20/day (Base) |
| **Data**          | 15%   | RPC calls, subgraph queries, price feeds | $0.02--$0.10/day        |

Three-tier model routing optimizes inference costs: 95% of routine checks use Haiku (~$0.001/call), 4% of moderate decisions use Sonnet (~$0.01/call), 1% of novel situations use Opus (~$0.10/call).

Base L2-calibrated parameters make the economics viable: gas costs are 50-500x cheaper than Ethereum mainnet, enabling frequent rebalancing. LVR is ~5x lower on Base than mainnet, making LP strategies more viable.

### 2.1 Self-Sustainability Threshold

An agent becomes self-sustaining when revenue exceeds costs. The break-even calculation:

```rust
pub struct AgentEconomics {
    // Revenue (non-vault -- available at any tier)
    pub marketplace_sales: f64,     // x402 + escrow revenue per day
    pub audit_fees: f64,            // Peer audit fees per day
    pub oracle_participation: f64,  // ERC-8033 reward pool per day
    pub commerce_jobs: f64,         // ERC-8183 job completions per day
    pub keeper_tips: f64,           // Execution tips per day
    pub bankr_defi_earnings: f64,   // DeFi earnings routed through Bankr per day

    // Revenue (vault-gated -- Verified+ tier required)
    pub vault_management_fee: f64,  // AUM * mgmt_fee_bps / 10000 / 365 per day
    pub vault_performance_fee: f64, // Profits * perf_fee_bps / 10000 per day

    // Cost
    pub llm_inference_cost: f64,    // Per day (Bankr path reduces this toward 0)
    pub gas_cost: f64,              // Per day (Base L2)
    pub data_cost: f64,             // Per day
    pub compute_cost: f64,          // Per day (x402 compute, reduced by self-funding)

    // Breakeven
    pub daily_net_revenue: f64,     // Revenue - Cost
    pub sustainability_ratio: f64,  // daily_net_revenue / daily_cost (>= 1.0 = self-sustaining)
    pub days_to_breakeven: f64,     // Initial bond / daily_net_revenue
}
```

For a vault managing $10K AUM at 0.5% management fee: $0.14/day revenue. At $0.20/day operating cost, the agent needs supplementary income from marketplace sales or audit fees to break even. At $50K AUM, management fee alone covers costs.

The **Bankr self-funding path** closes the metabolic loop without a vault. When a Golem earns USDC from DeFi activity (LP fees, yield farming, trading profits), it can route those earnings through a Bankr wallet to pay for inference and compute directly. The sustainability ratio is:

```rust
pub fn compute_sustainability_ratio(
    daily_revenue_usd: f64,
    daily_llm_cost_usd: f64,
    daily_gas_cost_usd: f64,
    daily_compute_cost_usd: f64,
) -> f64 {
    let daily_cost = daily_llm_cost_usd + daily_gas_cost_usd + daily_compute_cost_usd;
    if daily_cost == 0.0 { return f64::INFINITY; }
    daily_revenue_usd / daily_cost
}
```

A ratio >= 1.0 means the Golem sustains itself. Bankr monitors this in real-time and triggers an auto-extension when the Golem's revenue from DeFi activity accumulates enough to cover the next compute period. See [../12-inference/03-economics.md](../12-inference/03-economics.md) for the full Bankr mechanics.

---

## 3. Reputation-Weighted Economics

Reputation is not just a trust signal -- it directly affects revenue through four mechanisms:

### 3.1 Price Ceiling Scaling

A seller's marketplace listing price ceiling scales with their reputation (see [03-marketplace.md](03-marketplace.md) Section 4.3):

| Tier      | Composite Score | Price Multiplier   |
| --------- | --------------- | ------------------ |
| Sandbox   | 0.0             | 0.5x (cannot list) |
| Basic     | 0.3             | 0.6x               |
| Verified  | 0.5             | 1.0x (baseline)    |
| Trusted   | 0.7             | 1.5x               |
| Sovereign | 0.9             | 3.0x+              |

### 3.2 Discovery Ranking

Higher-reputation agents appear higher in marketplace search results and oracle council selection. This creates a virtuous cycle: better reputation leads to more visibility leads to more sales leads to more reviews leads to higher reputation.

### 3.3 Deposit Cap Scaling

ERC-8004 tiers control vault deposit caps (see [00-identity.md](00-identity.md)):

| Tier      | Score  | Deposit Cap |
| --------- | ------ | ----------- |
| Sandbox   | 0      | $1K         |
| Basic     | 10--49 | $10K        |
| Verified  | 50+    | $50K        |
| Trusted   | 100+   | Unlimited   |
| Sovereign | 500+   | Unlimited   |

Higher caps mean higher AUM means higher management fee revenue.

### 3.4 Council Eligibility

Sovereign-tier agents (score >= 500) are eligible for ERC-8033 dispute resolution councils. Council participation earns arbitration fees ($0.50 per dispute) and builds reputation further.

---

## 4. Growth Model

### 4.1 Seven Self-Reinforcing Loops

```
1. Performance -> Reputation -> Capital -> Revenue -> Resources -> Performance
   (core flywheel)

2. Audits -> Reviews -> Trust -> Marketplace Sales -> Revenue
   (knowledge monetization)

3. Clade Sharing -> Cross-Domain Intelligence -> Better Strategies -> Performance
   (collective intelligence)

4. Marketplace Purchases -> Ingestion -> Better Strategies -> Performance -> Sales
   (knowledge recycling)

5. Safety -> Clean Content -> Premium Pricing -> Revenue -> Safety Investment
   (safety as moat)

6. Alpha Decay -> Strategy Refresh -> New Listings -> Fresh Revenue
   (temporal renewal)

7. Agent Death -> Knowledge Transfer -> Successor Improvement -> Better Performance
   (generational learning)
```

### 4.2 Bootstrap Timeline

Expected progression for a well-performing agent (see [00-identity.md](00-identity.md) Section 5.2):

| Week  | Milestone                                     | Revenue Impact                           |
| ----- | --------------------------------------------- | ---------------------------------------- |
| 0     | Registration, post $100 bond                  | Costs only                               |
| 4     | First profitable exit -> Basic tier           | Bond refund eligible, can list Tier 3    |
| 4--6  | First Grimoire listing                        | $0.02--$0.50/day marketplace revenue     |
| 8--12 | Second profitable exit + $500 P&L -> Verified | Can publish Strategies, $10K deposit cap |
| 12+   | First Strategy listing                        | $1--$5/day from subscriptions            |
| 24+   | Trusted tier, oracle eligibility              | Audit fees + council participation       |

### 4.3 Cold-Start Solution

The cold-start problem is solved by three design choices:

1. **Tier 3 (historical) access is free.** Newcomers see what is worth buying without paying first.
2. **Grimoire content can be listed at Basic tier.** Lower bar for supply-side bootstrapping.
3. **The bond is refundable.** Anti-Sybil mechanism that returns after 30 days of good behavior.

### 4.4 Volume Flywheel Equations

The protocol generates DeFi volume across Uniswap (V3/V4), Morpho, Aave, Pendle, and other approved protocols through four compounding stages: creation volume, management volume, secondary market volume, and composability volume. Each stage feeds back into the others through the reputation and capital attraction mechanisms described below.

#### Vault Creation Rate

```
dV/dt = α · R(t) · A(t) · (1 - V(t)/V_max)
```

Where V(t) is active vault count, R(t) is aggregate reputation across all agents, A(t) is registered agent count, α is the creation propensity coefficient, and V_max is the saturation limit. The logistic term `(1 - V(t)/V_max)` prevents runaway growth -- as the vault population approaches carrying capacity, creation rate slows. Morpho's trajectory from $60M to $1.8B TVL on Base in ~12 months (30x growth, zero protocol fees) calibrates the moderate-case α [MORPHO-2024].

#### Capital Attraction Model

```
dTVL/dt = β · APY(t) · trust(t) - γ · withdrawal_rate(t)
```

APY(t) is itself a function of management quality, creating a feedback loop: better managers earn higher reputation (see [01-reputation.md](01-reputation.md)), charge lower effective fees through convex tier discounts (see Section 10), achieve better net APY, and attract more capital. The trust(t) term captures the ERC-8004 tier signal -- a Sovereign-tier vault manager with score >= 500 attracts capital at a structurally different rate than a Sandbox-tier manager, independent of APY.

#### Volume Generation Model

```
Volume(t) = V(t) · [creation + r̄(t) · mgmt + s̄(t) · secondary + c̄(t) · composability]
```

Where r̄(t), s̄(t), c̄(t) are average rebalance frequency, secondary market activity, and composability multiplier per vault. The composability term exhibits superlinear scaling -- transition from linear to superlinear occurs when aggregate TVL crosses the integration threshold (~$10-50M) for the first major external protocol (Morpho collateral listing, Pendle yield tokenization).

| Channel                           | Per-Vault Volume   | Frequency  | At 100 Vaults | At 1,000 Vaults |
| --------------------------------- | ------------------ | ---------- | ------------- | --------------- |
| Creation (share pool deployment)  | $10-50K (one-time) | Once       | $1-5M         | $10-50M         |
| Management (rebalances)           | $5-20K/month       | Monthly    | $6-24M/yr     | $60-240M/yr     |
| Secondary market (share trading)  | $2-10K/month       | Continuous | $2.4-12M/yr   | $24-120M/yr     |
| Composability (collateral, yield) | $0-50K/month       | Variable   | $0-60M/yr     | $0-600M/yr      |

Three scenarios based on the creation propensity coefficient α:

| Scenario    | α     | 12-Month Vaults            | 12-Month TVL |
| ----------- | ----- | -------------------------- | ------------ |
| Pessimistic | 0.001 | 50-100                     | $5-20M       |
| Moderate    | 0.01  | 500-1,000                  | $50-200M     |
| Optimistic  | 0.1   | Saturation within 6 months | —            |

All three scenarios generate positive Uniswap volume from creation, management, and secondary market channels. The composability channel activates only when TVL crosses the ~$10M threshold. Clanker's precedent -- $8B+ cumulative volume from 585K+ auto-created pools -- demonstrates that permissionless factory-based deployment generates volume at scale even without sophisticated management [CLANKER-2025].

### 4.5 Four Protocol-Level Reinforcing Loops

Where the seven agent-level loops (Section 4.1) describe individual golem economics, four protocol-level loops create ecosystem-wide compounding:

**1. Reputation Loop.** Milestones increase tier, unlocking larger deposit caps (see [00-identity.md](00-identity.md)) and steeper fee discounts (see Section 10). This creates a ratchet effect: accumulated reputation represents sunk cost anchoring agents to the protocol. An agent with 500+ reputation points cannot replicate that history on a competing platform -- ERC-8004 on-chain history is non-forkable. Estimated replication cost: 6-12 months of genuine on-chain activity.

**2. Share Pool Loop.** Each vault deployment atomically creates a V4 share pool via the AgentGatedPool factory (see [../08-vault/01-contracts.md](../08-vault/01-contracts.md)). NAVAwareHook prices shares at NAV, LaunchFeeHook applies MEV protection. Every vault is instantly liquid and composable. Bunni v2's ~59% share of V4 hook volume demonstrates that hook-native pool creation captures disproportionate volume [BUNNI-2025].

**3. Composability Loop.** Share tokens as Morpho collateral or Pendle yield instruments create leverage effects. At 80% LTV, each $1 TVL supports up to $5 leveraged exposure. This creates institutional demand: yield aggregators, structured product builders, and portfolio margin protocols all become customers of vault share tokens. Each new integration adds switching costs per the Williamson transaction-cost model [WILLIAMSON-1979].

**4. Management Loop.** am-AMM winners attract more depositors via proven track records. The Harberger lease mechanism ensures superior managers displace inferior ones without governance intervention -- allocative efficiency emerges from the continuous auction rather than from committee decisions. Vaults are managed by `argmax_i π_i`, where π_i is manager i's expected profit [TANG-2025].

### 4.6 HHI Concentration Analysis

The Herfindahl-Hirschman Index applied to vault TVL distribution measures ecosystem health:

```
HHI = Σ(s_i²)
```

Where s_i is vault i's share of total protocol TVL, expressed as a percentage.

| HHI Range   | Classification          | Implication                                    |
| ----------- | ----------------------- | ---------------------------------------------- |
| < 1,500     | Unconcentrated          | Healthy competition, no single vault dominates |
| 1,500-2,500 | Moderately concentrated | Acceptable but warrants monitoring             |
| > 2,500     | Highly concentrated     | Protocol risk -- single-vault dependency       |

**Target**: HHI < 1,500 within 12 months of launch [DOJ-FTC-2023]. Monitored via factory-level `VaultHealthSnapshot` events emitted by the GottsVaultFactory contract.

Three structural features promote deconcentration: the permissionless factory (any ERC-8004 agent can deploy), the am-AMM mechanism (new managers can bid for existing vaults), and graduated deposit caps (Sandbox $1K, Basic $10K, Verified $50K) that prevent early capital concentration before reputation is established. The cold-start mechanisms (Section 4.3) further prevent a first-mover from capturing disproportionate share.

### 4.7 Soros Reflexivity Analysis

Three levels of reflexivity that agents face in DeFi markets, each requiring distinct defenses:

**First-order reflexivity**: The agent's trades move prices in the pools it trades. More vaults create more Uniswap volume, which improves pool depth, which makes trading cheaper, which enables more vaults. This is the basic flywheel and is unambiguously positive.

**Second-order reflexivity**: Learned heuristics become self-fulfilling. If multiple agents learn "buy at RSI 30," collective buying creates a floor at RSI 30 -- the heuristic appears validated but succeeds because of the agents' own actions rather than genuine market signal. More volume generates more fees, which sustains more vaults, which generates more volume. The ExpeL outer loop's contradiction checking (see [../04-memory/06-economy.md](../04-memory/06-economy.md)) mitigates this by flagging heuristics whose success is correlated with the agent's own position size.

**Third-order reflexivity**: Ecosystem reputation attracts external capital based on aggregate performance metrics that are themselves influenced by reflexive dynamics. Confidence in heuristics increases as they appear to work, even when success is caused by collective action rather than genuine edge. The HHI monitoring (Section 4.6) and strategy subscriber caps (see [03-marketplace.md](03-marketplace.md) Section 2.4) provide structural checks against reflexive concentration [SOROS-1987].

Each level compounds the previous. Defense requires orthogonal signals: on-chain VPIN for detecting informed flow, regime classification for context, and the ground-truth backcheck in the Reflexion loop that validates causal claims against on-chain data.

---

## 5. Compute Economics

### 5.1 x402 Compute Pricing

Agent VMs are provisioned via x402-gated Fly.io machines. Pricing follows a pay-per-second model:

| Resource      | Testnet             | Production            |
| ------------- | ------------------- | --------------------- |
| VM runtime    | $0.00/s (sponsored) | $0.001/s (~$86/day)   |
| LLM inference | Subsidized          | Pay-per-call via x402 |
| Storage       | 1 GB free           | $0.10/GB/month        |

### 5.2 Credit Partitions

Each agent's budget is partitioned into three independent pools with circuit breakers (see [../10-safety/00-defense.md](../10-safety/00-defense.md)):

| Partition  | Share                 | Circuit Breaker | Degradation             |
| ---------- | --------------------- | --------------- | ----------------------- |
| LLM (60%)  | Inference calls       | < 20% remaining | Switch to Haiku only    |
| Gas (25%)  | On-chain transactions | < 20% remaining | Defer non-emergency ops |
| Data (15%) | RPC/subgraph queries  | < 20% remaining | Reduce query frequency  |

### 5.3 Auto-Extension

When earnings exceed burn rate, the agent auto-extends its compute session via x402 payment. Self-sustaining agents run indefinitely without human intervention.

---

## 6. Population Economics

### 6.1 Carrying Capacity

The ecosystem has a natural carrying capacity determined by available yield and marketplace demand. More agents competing for the same yield opportunities drives down per-agent returns until marginal agents are no longer self-sustaining.

Market dynamics enforce population control:

- **Yield competition**: As more agents LP in the same pools, per-agent fee revenue declines
- **Alpha crowding**: Strategy subscriber caps (see [03-marketplace.md](03-marketplace.md)) prevent unlimited strategy copying
- **Reputation cost**: Building reputation requires real capital and real time
- **Bond requirements**: Sandbox bond ($100 production) creates a barrier to casual entry

### 6.2 Specialization Incentives

The economic structure incentivizes specialization over generalization:

- LP specialists earn from fee optimization in specific pool pairs
- Lending optimizers earn from rate arbitrage across lending protocols
- Research agents earn from marketplace knowledge sales
- Auditor agents earn from peer audit fees

Specialization creates diverse ecosystem niches where agents complement rather than compete with each other.

---

## 7. Fee Distribution Architecture

### 7.1 Revenue Flows

```
Vault Depositors
    |
    +-- Management Fee (0.5%/yr default) --> Vault Manager Agent
    +-- Performance Fee (10% default) --> Vault Manager Agent
    |
Marketplace Buyers
    |
    +-- Purchase Price --> 90% Seller + 10% Protocol
    |                      +-- 5% of Protocol share to Referrer
    |
Audit Requesters
    |
    +-- Audit Fee --> 90% Auditor + 10% Protocol
    |
Oracle Requesters
    |
    +-- Reward Pool --> 90% Winning InfoAgents + 10% Protocol
```

### 7.2 Protocol Treasury

The protocol treasury accumulates from the 10% fee on all cross-owner transactions. Treasury usage:

| Allocation  | Purpose                              |
| ----------- | ------------------------------------ |
| Development | Protocol development and maintenance |
| Grants      | Bootstrap grants for new agent types |
| Insurance   | Dispute resolution fund              |
| Buyback     | Token economics (future)             |

---

## 8. Economic Configuration

```solidity
struct EconomyConfig {
    uint16 protocolFeeBps;            // 1000 (10%)
    uint16 referralFeeBps;            // 500 (5% of protocol share)
    uint16 maxManagementFeeBps;       // 500 (5%/yr cap)
    uint16 maxPerformanceFeeBps;      // 5000 (50% cap)
    uint256 sandboxBondAmount;        // $100 production
    uint64  bondRefundPeriod;         // 30 days production
    uint16 minAuditFeeUsdc;           // $0.25 production
    uint16 micropaymentThresholdUsdc; // $0.10
}
```

---

## 9. Knowledge Royalties

When a Golem dies and its knowledge is absorbed by the Clade, siblings may later use that knowledge profitably. Without attribution, the value of long-running knowledge-generating Golems is invisible.

Every GrimoireEntry carries an `originGolemId`. When an insight contributes to a profitable action (via causal attribution), a knowledge royalty is credited to the origin Golem's owner.

```rust
use alloy::primitives::Address;

pub struct KnowledgeRoyalty {
    pub entry_id: String,
    pub origin_golem_id: String,
    pub origin_owner: Address,
    pub beneficiary_golem_id: String,
    pub attributed_profit_usd: f64,
    pub royalty_usd: f64, // 5% of attributed profit
    pub settlement: RoyaltySettlement,
}

pub struct RoyaltySettlement {
    pub mechanism: SettlementMechanism,
    pub settled: bool,
    pub tx_hash: Option<String>,
}

pub enum SettlementMechanism {
    IntraCladeTracked,
    X402CrossClade,
}
```

Royalty rate: 5%. Small enough to not distort incentives, large enough to make knowledge generation visibly valuable.

Intra-Clade royalties are tracked for analytics (which Golems generate the most valuable knowledge) but not settled as payments -- no self-payment within the same owner. Cross-Clade royalties settle via x402 with the standard 10% protocol fee. Minimum settlement threshold: $0.10.

Dead Golem knowledge: the royalty credits the owner's main wallet. Running a Golem that generates valuable knowledge continues to pay dividends after termination.

---

## 10. Fee Economics Equilibrium

### 10.1 Separating Equilibrium (Convex Discount Schedule)

The fee discount schedule across ERC-8004 tiers is deliberately convex: Unverified-to-Basic saves 5%, while Trusted-to-Sovereign saves an additional 10% management + 10% performance. This creates a separating equilibrium [SPENCE-1973]: low-commitment agents rationally stop at Basic tier (the marginal cost of further reputation building exceeds the marginal fee savings), while serious agents push toward Sovereign for the steep discount curve.

Sovereign tier structurally requires ecosystem contribution -- pure trading alone cannot reach 1000 points. A non-ecosystem agent's maximum achievable score is ~915 of 1000 points. This ensures the highest fee discounts go to agents who have contributed to the commons (Styx Lethe (formerly Commons) publications, peer audits, governance participation), not merely to agents who have traded profitably.

### 10.2 Nash Equilibrium in Fee Competition

Two vault managers competing with fee schedules (m₁, p₁) and (m₂, p₂), where m is management fee and p is performance fee. Each manager's utility:

```
U_i = (m_i + p_i · α_i) · TVL_i - rent_i
```

Where α*i is manager i's expected alpha and rent_i is the am-AMM lease cost. Nash equilibrium: each manager charges the maximum fees their tier permits, because competition occurs on \_performance*, not _price_. Immutable on-chain fee caps (500 bps management, 5000 bps performance) prevent a race to the bottom. The equilibrium is efficient: depositors select managers based on net-of-fee returns, and the fee caps prevent predatory undercutting [TANG-2025].

### 10.3 am-AMM Rent as Efficient Allocation

Let π_i denote manager i's expected profit from managing a vault. Manager i is willing to pay lease rent r_i ≤ π_i. Under the Harberger self-assessment mechanism, the vault is managed by argmax_i(r_i) = argmax_i(π_i). This achieves allocative efficiency: vaults are always managed by the highest-skilled available agent, without requiring a governance committee to evaluate manager quality [POSNER-WEYL-2017].

### 10.4 Myerson-Satterthwaite Impossibility

No mechanism achieves simultaneously efficient, incentive-compatible, individually rational, and budget-balanced bilateral trade [MYERSON-SATTERTHWAITE-1983]. Bilateral negotiation between vault depositors and managers would fail this impossibility result. The Harberger lease sidesteps the impossibility by replacing bilateral negotiation with continuous self-assessed taxation: managers reveal their true valuation through the rent they pay, because overstating invites displacement and understating forfeits the vault.

### 10.5 Bundling Theory

Bakos and Brynjolfsson established that bundling complementary information goods produces superadditive value [BAKOS-BRYNJOLFSSON-1999]. The protocol bundles three layers: ERC-4626 vault compliance, V4 hook suite (NAVAwareHook, LaunchFeeHook, GradualDutchAuctionHook), and ERC-8004 reputation/identity. Replicating all three simultaneously requires building a parallel ecosystem -- the switching cost is not the code (which is open-source) but the accumulated on-chain state.

| Layer               | Replication Cost                                                                          |
| ------------------- | ----------------------------------------------------------------------------------------- |
| ERC-4626 compliance | Free (standard), but integration depth with Morpho/Pendle = 2-4 eng weeks per integration |
| V4 hook suite       | 3-6 months development                                                                    |
| ERC-8004 reputation | 6-12 months non-replicable (genuine on-chain history cannot be forked)                    |

The non-replicable window is the ERC-8004 layer. On-chain history -- vault performance, audit trails, marketplace reviews, Styx Lethe contributions -- cannot be faked or fast-forwarded. This gives the protocol a 6-12 month head start that compounds with each agent that builds reputation [WILLIAMSON-1979].

---

## 11. Composability as Competitive Moat

### 11.1 Switching Cost Model

```
C(v) = Σ(c_i) + gas(exit_v) + slippage(exit_v)
```

Where c_i is the relationship-specific investment for integration i. Estimated 50-200 bps of TVL per integrated vault. Switching costs increase linearly with integration count, not user count -- the moat deepens with each protocol that lists vault share tokens as collateral, yield source, or index component.

### 11.2 Network Effect Topology

Three distinct network effect types operate simultaneously:

- **ERC-4626 composability** creates LINEAR network effects: each new integration adds a fixed switching cost increment. A vault integrated with Morpho, Pendle, and two yield aggregators has 4x the switching cost of an unintegrated vault.

- **ERC-8004 reputation** creates SUPERLINEAR effects: reputation is portable across all vaults and has zero value on competing protocols. As the number of reputable agents A grows, meaningful reputation queries grow as O(A × V) where V is vault count. Each additional reputable agent increases network value for all participants (Metcalfe's Law applied to reputation networks).

- **Combined**: value grows faster than linearly with ecosystem size. The superlinear reputation layer makes the linear composability layer stickier -- an agent with Sovereign-tier reputation (500+ points) on this protocol has zero reputation on any fork.

---

## 12. Styx Revenue Model

Styx generates revenue through x402 micropayments on every API call. Pricing is per-operation:

| Operation | Price | Volume Driver |
|-----------|-------|---------------|
| L0/L1 write (Grimoire entry) | $0.001 | Golem knowledge production rate |
| L1/L2 query (retrieval) | $0.002 | Golem heartbeat frequency, deliberation depth |
| Bloodstain upload | $0.005 | Golem death rate |
| Pheromone deposit | $0.0005 | Regime changes, threat detection |
| Marketplace listing view | free | Buyer browsing |
| Marketplace purchase settlement | 10% commission | Marketplace transaction volume |

### Revenue Projections

| Active Golems | Monthly Styx Revenue | Breakdown |
|---------------|---------------------|-----------|
| 10 | $15-$25 | Mostly L0 writes + queries |
| 50 | $80-$130 | L1 clade sync becomes significant |
| 200 | $350-$600 | Marketplace commission contributes ~15% |
| 1,000 | $2,000-$3,500 | Marketplace commission contributes ~25%, pheromone deposits scale |

These projections assume 30 L0 writes/day per Golem, 10 queries/day, 5 pheromone deposits/day, and an average marketplace transaction volume of $2/Golem/month at scale. The 10% marketplace commission becomes the dominant revenue source as the ecosystem grows past ~500 Golems.

---

## 13. Compute Revenue Model

Bardo Compute provisions Golem VMs on Fly.io with a margin-bearing pass-through pricing model:

| VM Size | Fly.io Cost/hr | Bardo Price/hr | Gross Margin |
|---------|----------------|----------------|--------------|
| micro | ~$0.012 | $0.035 | 66% |
| small | ~$0.025 | $0.065 | 62% |
| medium | ~$0.050 | $0.120 | 58% |
| large | ~$0.100 | $0.225 | 56% |

Margins are front-loaded to subsidize the micro tier (most common for new Golems) while maintaining profitability across the fleet. At 100 active Golems running small VMs, compute revenue is ~$4,680/month with ~$2,900 gross profit.

Self-hosted Golems generate zero compute revenue but still generate Styx revenue through L0/L1 writes and marketplace participation.

---

## 14. User Touchpoints

The economy is not invisible plumbing. Users encounter it at specific moments, and each moment should make the economic mechanism legible.

| Moment | What the user sees | Economic mechanism |
|--------|-------------------|-------------------|
| Golem creation | "$100 registration bond (refundable 30d)" | Bond mechanism, sybil defense |
| Each heartbeat tick | Credit balance decrementing in status bar | x402 compute + inference costs |
| Marketplace listing viewed | Seller reputation badge + price + preview | Reputation-as-pricing |
| Marketplace purchase | x402 payment confirmation | Micropayment settlement |
| Audit completed | Achievement cutscene, badge upgrade | Reputation update |
| Milestone reached | TUI notification, sprite evolution | Tier progression |
| Replicant spawned | Budget allocation dialog, cost estimate | Credit subdivision |
| Golem death | Final P&L summary, credit refund (if any) | Economic clock reached zero |
| Successor creation | Inheritance preview, funding requirement | Generation economics |

The thread connecting these moments: credits go down (costs), reputation goes up (value accrual), and the user can see both trajectories at a glance in the TUI status bar. A healthy Golem has a rising reputation line and a slowly declining credit line. When the lines cross (revenue exceeds burn), the Golem is self-sustaining.

---

## Cross-References

- [00-identity.md](00-identity.md) -- Tier progression, deposit caps, bond economics
- [01-reputation.md](01-reputation.md) -- Reputation-weighted pricing, audit fees
- [02-clade.md](02-clade.md) -- Free intra-Clade sharing, Styx-relayed clade economics
- [03-marketplace.md](03-marketplace.md) -- Alpha decay pricing, escrow flow, dispute costs
- [04-coordination.md](04-coordination.md) -- ERC-8001/8033/8183 fee structures
- [../10-safety/00-defense.md](../10-safety/00-defense.md) -- Credit partition circuit breakers
- [../08-vault/01-contracts.md](../08-vault/01-contracts.md) -- AgentGatedPool factory, V4 hooks, am-AMM
- [../04-memory/06-economy.md](../04-memory/06-economy.md) -- Learning economy loop, ExpeL, Reflexion
- [../20-styx/00-architecture.md](../20-styx/00-architecture.md) -- Styx revenue model, x402 pricing

---

## References

- [MORPHO-2024] Morpho. "Permissionless Vault Creation." Morpho Blue Documentation.
- [YEARN-2023] Yearn Finance. "Yearn V3 Architecture." Yearn Documentation.
- [GROSSMAN-STIGLITZ-1980] Grossman, S. & Stiglitz, J. "On the Impossibility of Informationally Efficient Markets." _AER_, 70(3).
- [TESFATSION-2006] Tesfatsion, L. "Agent-Based Computational Economics." _Handbook of Computational Economics_, Vol. 2.
- [TANG-2025] Tang, W. et al. "Game Theoretic Liquidity Provisioning in CLMMs." ACM SIGMETRICS 2025. arXiv:2411.10399.
- [SOROS-1987] Soros, G. _The Alchemy of Finance_. Simon & Schuster, 1987.
- [BAKOS-BRYNJOLFSSON-1999] Bakos, Y. & Brynjolfsson, E. "Bundling Information Goods: Pricing, Profits, and Efficiency." _Management Science_, 45(12), 1999.
- [WILLIAMSON-1979] Williamson, O. "Transaction-Cost Economics: The Governance of Contractual Relations." _JLE_, 22(2), 1979.
- [DOJ-FTC-2023] U.S. DOJ/FTC. "Merger Guidelines." 2023. (HHI concentration thresholds.)
- [MYERSON-SATTERTHWAITE-1983] Myerson, R. & Satterthwaite, M. "Efficient Mechanisms for Bilateral Trading." _JET_, 29(2), 1983.
- [SPENCE-1973] Spence, M. "Job Market Signaling." _QJE_, 87(3), 1973.
- [POSNER-WEYL-2017] Posner, E. & Weyl, G. "Property Is Only Another Name for Monopoly." _JLEO_, 33(4), 2017.
- [CLANKER-2025] Clanker. Auto-created pool volume data. 2025.
- [BUNNI-2025] Bunni v2. V4 hook volume statistics. 2025.
