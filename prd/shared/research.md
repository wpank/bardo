# Research and Market Context [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Reader orientation:** This document consolidates market data, competitive analysis, academic research, and technology rationale that ground Bardo's design decisions. It belongs to the `shared/` reference layer and covers Agent Capital Markets, DeFi vault mechanisms, the competitive landscape (Yearn, Morpho, dHEDGE, Giza, Bankr), Base L2 yield opportunities, ERC integration roadmaps, strategy adapter taxonomy, regulatory landscape, and technology stack rationale. The key concept is that Bardo sits at the intersection of social distribution (Bankr-style), vault infrastructure (Yearn-style), and autonomous agent runtime (Golem -- mortal autonomous agents with learning systems). No existing product combines more than two of these. See `prd2/shared/glossary.md` for full term definitions.

---

## Overview

This document consolidates market data, competitive analysis, academic research, and technology rationale that ground Bardo's design decisions. All figures are sourced from primary research conducted February-March 2026. See `prd2/shared/citations.md` for the full citation index.

---

## 1. Market Opportunity

### 1.1 Agent Capital Markets

- **~70% of Uniswap transactions** already come from bots and agents [MESSARI-2025]. Uniswap is the de facto settlement layer for Agent Capital Markets.
- **AI agent token market cap** reached $7.7B+ with $1.7B daily trading volume.
- **Delphi Digital** (Feb 2026): "The Agent Capital Markets Has Arrived" -- "AI agents are becoming sovereign economic entities that have verifiable identities, portable reputations, and the ability to transact autonomously" [DELPHI-2026].
- **Stablecoin market** exceeds $300B; the GENIUS Act provides regulatory clarity for agent-driven payment systems.
- **x402 protocol** (micropayment protocol; agents pay for inference/compute/data via signed USDC transfers, no API keys) from Cloudflare has processed **100M+ agent-to-agent transactions** with ~200ms settlement on Base [DELPHI-2026].
- **ERC-8004** (on-chain agent identity standard tracking capabilities, milestones, and reputation; co-authored by MetaMask + Google) launched mainnet Jan 29, 2026 with **24,500+ agents registered in the first week**.
- **BankrBot** (220K+ wallets, 2M+ messages) demonstrates self-funding agents that earn fees, pay for compute, and trade on Uniswap autonomously.

### 1.2 DeFi Vault Ecosystem

- **2,700+ ERC-4626 vault deployments** across Ethereum, Base, Arbitrum, and Optimism
- **Morpho Blue**: $5.8B total TVL ($1.4B+ on Base alone, 4-6% USDC supply APY) [DEFILLAMA-2026]
- **Yearn V3**: ~$4B TVL, ERC-4626 standard, $2.4M annualized revenue
- **Aave V3 (Base)**: $1.26B TVL, 2.5-3.5% USDC supply APY
- **Aerodrome (Base)**: $600M-$1.24B TVL, 5-60% LP APY (ve(3,3) model)
- **Pendle (Base)**: ~$180M TVL, 5-15% fixed yield via PT
- **Ondo USDY**: $1B+, ~4.25% yield (tokenized US treasuries)

### 1.3 Prediction Markets and Tokenized RWA

- **Polymarket**: $18B+ cumulative volume. Demonstrates demand for autonomous market-making and strategy execution at scale.
- **$8.89B** in tokenized treasury products. **Ondo**: $2.5B+ in tokenized US treasuries. RWA yield provides a floor for autonomous agent strategies -- stable, predictable returns suitable for conservative Golem dispositions.
- **BlackRock BUIDL**: $1.84-2.5B, now tradable on Uniswap via UniswapX.

### 1.4 Uniswap Ecosystem

- **Uniswap V4** has processed **$110B+ cumulative volume** across 10+ chains since Jan 30, 2025 launch. 2,500+ hook-enabled pools deployed.
- **Clanker** routes $5B+ in agent token volume through V4, generating $49.7M in creator fees from 585K+ tokens.
- **am-AMM** (auction-managed AMM) -- Bunni v2's peer-reviewed mechanism [ADAMS-2025] processes ~59% of all V4 hook volume ($138M/month).
- **CCA** (Continuous Clearing Auction) is live on mainnet across Ethereum, Unichain, Arbitrum, and Base. Aztec's first CCA raised $60M from 17,000+ bidders.
- **413 hooks deployed** powering 4,371+ pools with $18.29M TVL and $235.86M cumulative volume.

### 1.5 MCP Ecosystem

- **MCP ecosystem** has grown to 10,000+ published servers with 97M monthly SDK downloads.
- Anthropic donated MCP to the Linux Foundation's **Agentic AI Foundation (AAIF)** (Dec 2025), co-founded with Block and OpenAI.
- Third-party Uniswap MCP servers are proliferating -- Bardo must own the canonical implementation before the ecosystem fragments.

---

## 2. DeFi Vault Mechanisms

### 2.1 Fee Collection Patterns

The dominant pattern across DeFi vaults is **fee collection via share minting** -- diluting existing depositors rather than transferring tokens. Requires no asset movement, aligns fee recipients with vault performance, and is gas-efficient.

| Protocol    | Management Fee                | Performance Fee                | Collection Method                       |
| ----------- | ----------------------------- | ------------------------------ | --------------------------------------- |
| Yearn V3    | Configurable (V2 default: 2%) | Configurable (V2 default: 20%) | Shares minted during `process_report()` |
| Morpho V2   | Up to 5%/yr AUM               | Up to 50% of interest          | Shares minted to `feeRecipient`         |
| Enzyme v4   | Continuous compounding on AUM | High-watermark based           | All fees paid as fund shares            |
| Balancer V3 | N/A (pool-level)              | 10% yield fee                  | Share-level                             |

**ERC-4626 fee stance**: The standard is deliberately fee-agnostic. `convertToShares`/`convertToAssets` provide rough estimates ignoring fees; `previewDeposit`/`previewRedeem` must return exact values inclusive of fees.

**Bardo design choice**: Immutable fee caps -- 500 bps (5%/yr) management, 5,000 bps (50%) performance. Fees collected by minting shares. No protocol fee (Morpho pattern: vault creators keep 100%).

### 2.2 Seven Yield Sources

All sustainable yield traces to someone paying for a service -- borrowing, trading execution, block production, or coupon payments.

| Source                     | Typical APY           | Risk        | Mechanism                                                                         |
| -------------------------- | --------------------- | ----------- | --------------------------------------------------------------------------------- |
| **Lending interest**       | 2-8% (stablecoins)    | Low-Medium  | Borrowers pay interest. Kinked rate curve. Aave, Morpho, Compound                 |
| **Liquidity provision**    | 5-30%+ (concentrated) | Medium-High | Trading fees (0.01-1% per swap). V3 concentrated: up to 4,000x capital efficiency |
| **CRV emissions**          | 10-20%                | Medium      | Curve distributes CRV inflation to gauge-staked LPs. Convex holds 35%+ of veCRV   |
| **ETH staking**            | 3-5.7%                | Low         | Consensus rewards + execution layer. ~89% validators run MEV-Boost                |
| **Restaking (EigenLayer)** | Base + 1-3%           | Medium      | Reuses staked ETH to secure AVSs. ~$19.5B TVL                                     |
| **Basis trades**           | 7-30%                 | Medium-High | Delta-neutral: spot long + perp short. Ethena: ~$10B+ TVL. Procyclical            |
| **RWA yields**             | 3.75-5%               | Low         | Tokenized Treasuries: ~$7.5B. Most sustainable source                             |

### 2.3 Liquidity for Redemptions

The fundamental tension: vaults want 100% capital deployment but must honor instant redemptions.

| Mechanism                        | Protocol Example                                                 | Tradeoff                                       |
| -------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------- |
| **Idle buffers**                 | Yearn V3 (`minimum_total_idle`), Morpho                          | Capital drag on yield                          |
| **Withdrawal queues**            | Yearn V3 (up to 10 strategies)                                   | No partial redemptions                         |
| **Async redemptions (ERC-7540)** | Centrifuge ($500M+ AUM)                                          | UX friction; essential for illiquid strategies |
| **In-kind exits**                | Enzyme (`redeemSharesInKind()`), Morpho V2 (`forceDeallocate()`) | May receive unwanted assets                    |
| **Liquidity buffers**            | Balancer V3                                                      | Unfunded buffers earn nothing                  |

**Stress test**: March 2025 Morpho/xUSD incident. Utilization spiked to 100%, rates surged to ~190%. 80% of withdrawals completed within 3 days. Isolation model contained contagion but allowed individual vault illiquidity.

---

## 3. Protocol Deep-Dives

### 3.1 Yearn V3

**Architecture**: Singleton `TokenizedStrategy` contract per chain. Every strategy inherits `BaseStrategy` and uses `delegatecall` to the singleton for all ERC-4626 logic. Strategies are standalone ERC-4626 vaults.

**Strategist override surface** (3 functions): `_deployFunds()`, `_freeFunds()`, `_harvestAndReport()`.

**Profit unlock**: `process_report()` mints locked shares that unlock linearly over `profit_max_unlock_time`. Weighted-average calculation for overlapping reports.

**Scale**: ~$4B TVL, 250+ strategies, ~$2.4M annualized revenue. 90% of protocol revenue goes to stYFI stakers since Feb 2026.

### 3.2 Morpho (MetaMorpho)

**Architecture**: Non-upgradeable ERC-4626 contracts allocating across Morpho Blue isolated lending markets. Four roles: owner, curator, allocator, guardian (defensive veto only).

**Queue system**: Supply queue and withdraw queue, max 30 markets. `reallocate()` enforces `totalWithdrawn == totalSupplied` (net-zero invariant).

**Base deployment**: $1.4B+ deposits. Steakhouse USDC vault: ~$389M. 64% of volume from looping strategies.

### 3.3 Giza Protocol

**ARMA framework**: Autonomous Risk Management Agent on Base. 102,000+ autonomous transactions, $35M+ capital managed. Direct competitor in the autonomous DeFi agent space.

**Three-layer architecture**: (1) MCP semantic abstraction, (2) EigenLayer AVS decentralized execution, (3) ERC-7579/ERC-4337 agent authorization.

### 3.4 Enzyme v4

**Guaranteed exit**: `redeemSharesInKind()` returns pro-rata share of all ERC20 holdings. No policy hooks, always available.

**Scale**: ~$90M TVL, pivoting to "Enzyme Onyx" -- Cayman-regulated tokenized funds.

---

## 4. Competitive Landscape

### 4.1 Social Trading Platform History

Every on-chain fund management platform tells the same story: vault infrastructure works, but the social layer does not scale.

| Platform           | Peak TVL | Current TVL   | Revenue   | Key Lesson                                                      |
| ------------------ | -------- | ------------- | --------- | --------------------------------------------------------------- |
| **Yearn V3**       | ~$4B     | ~$4B          | ~$2.4M/yr | Infrastructure-first wins over social trading                   |
| **dHEDGE**         | $50M+    | $33-50M       | ~$2M/yr   | Leverage products outperform social yield                       |
| **Sommelier**      | $71M     | $14.6M        | --        | Off-chain execution adds complexity without clear UX benefit    |
| **Enzyme v4**      | $185M    | ~$90M         | --        | Pivoting to institutional. Not consumer-friendly                |
| **Nested Finance** | --       | $141K         | --        | Raised $7.5M, effectively dead. Portfolio-as-NFT, zero traction |
| **Giza**           | --       | $35M+ managed | --        | Direct competitor; EigenLayer AVS execution                     |

**Key lessons**:

1. **ERC-4626 is non-negotiable** -- every serious platform uses it for composability, audits, and wallet/aggregator integration [ERC-4626-SPEC]
2. **Permissionless strategy creation** (Yearn V3 model) maximizes innovation but requires user due diligence
3. **Standard fees**: 1-2% management annual, 10-20% performance on profits, 5-10% protocol take
4. **One-click deposit via flash loans** is table stakes (EIP-7702)
5. **CEX copy trading dwarfs DeFi**: Bitget reports 93% profitable futures copy trades, 100K+ users in H1 2025
6. The winning design is **"autonomous agents managing capital with verifiable track records"**
7. **Trust-minimization is the moat for high-AUM users.** Every dead platform was fully custodial. Users with $50K+ care deeply about custody. The self-sovereign path is what makes the managed path trustworthy.

### 4.2 Bankr Architecture

Bankr is a middleware orchestration layer routing natural-language commands from social media into on-chain transactions. Uses Privy server wallets for instant wallet provisioning and composes existing infrastructure (Clanker V4 factory, 0x Swap API) rather than deploying custom contracts.

**Infrastructure patterns adopted by Bardo**:

1. **Orchestration over ownership**: Compose where possible (Permit2 for approvals, existing protocols for yield)
2. **Instant wallet provisioning**: Privy server wallets on first interaction, no signup
3. **Self-sustaining agent loop**: On-chain fees fund agent inference and operations
4. **Reputation-gated rate limiting**: Tiering via ERC-8004 reputation

**Bankr gaps Bardo fills**: No productive infrastructure, no agent runtime, no learning system, no vault-based yield.

### 4.3 Head-to-Head Comparison

| Dimension               | Bardo                    | Bankr             | Yearn V3       | Morpho             | dHEDGE             |
| ----------------------- | ------------------------ | ----------------- | -------------- | ------------------ | ------------------ |
| **Primary UX**          | Social/CLI/API           | Tweet reply       | Dashboard      | Dashboard          | Dashboard          |
| **On-chain primitive**  | ERC-4626 vault + V4 hook | Clanker ERC-20    | ERC-4626 vault | Lending markets    | dHEDGE pools       |
| **What user gets**      | Yield-bearing vault      | Speculative token | Yield position | Lending position   | Leveraged exposure |
| **Agent runtime**       | Golem-RS (Rust, 3-loop learning) | OpenClaw (local)  | N/A            | N/A                | N/A                |
| **Compute hosting**     | x402 Fly.io + self-deploy + bare metal | LLM gateway       | N/A            | N/A                | N/A                |
| **Identity/reputation** | ERC-8004 (5 tiers)       | ERC-8004 (basic)  | N/A            | Curator reputation | N/A                |
| **Learning system**     | Grimoire + Styx + PLAYBOOK.md | N/A               | N/A            | N/A                | N/A                |

**Bardo's position**: Yearn's vault infrastructure + Bankr's social distribution + Golem's learning system (Grimoire -- the agent's persistent knowledge base -- plus Styx -- global knowledge relay and persistence layer -- plus PLAYBOOK.md -- the agent's evolving strategy document) + x402-compute hosting + ERC-8004 reputation. No existing product has more than two of these.

---

## 5. Base L2 Ecosystem

### 5.1 Protocol TVL and Yield Landscape

Base DeFi TVL: ~$3.5B (range: $2.4-4.6B depending on source). Five major yield venues:

| Protocol          | TVL on Base     | Realistic APY                           |
| ----------------- | --------------- | --------------------------------------- |
| **Morpho Blue**   | $1.4B+ deposits | 4-6% (curated USDC)                     |
| **Aave V3**       | $1.26B supplied | 2.5-3.5% (USDC supply)                  |
| **Aerodrome**     | $600M-$1.24B    | 5-60% (stable: 5-15%, volatile: 15-60%) |
| **Pendle**        | ~$180M          | PT: 5-8%, LP: 8-20%                     |
| **Extra Finance** | $100M+          | 50-150%+ (leveraged, high risk)         |

### 5.2 APY Reality Check by Risk Tier

| Risk   | Strategy                       | Realistic APY |
| ------ | ------------------------------ | ------------- |
| Low    | Morpho curated USDC vault      | 4-6%          |
| Low    | Aave USDC supply               | 2.5-3.5%      |
| Medium | wstETH/ETH recursive loop (5x) | 6-10%         |
| Medium | Aerodrome stable LP            | 5-15%         |
| High   | Aerodrome volatile LP          | 15-60%        |
| High   | Extra Finance 3x leveraged LP  | 50-150%+      |

### 5.3 Composability Patterns

**Recursive lending loops**: 64% of Morpho volume. Deposit wstETH -> borrow ETH -> loop. At 82.5% LTV, ~15 loops reach ~5.45x leverage. Flash loans collapse 15 txs into one.

**Ethena-Pendle-Morpho loop**: >$4B in assets. sUSDe yield averaged ~10% annualized since inception.

### 5.4 RWA on Base

| Product         | TVL/AUM    | APY       | Access                                |
| --------------- | ---------- | --------- | ------------------------------------- |
| Ondo USDY       | $1B+       | 3.69-5.3% | Permissionless ERC-20                 |
| Ondo OUSG       | $770M+     | ~3.75%    | Qualified purchasers only             |
| BlackRock BUIDL | $1.84-2.5B | ~4-5%     | $5M minimum, KYC. Tradable on Uniswap |

### 5.5 AI Token Ecosystem

Virtuals Protocol (VIRTUAL): ~$400M market cap, 18,000+ agents deployed. Base accounts for 90.2% of daily active wallets.

**DeFi infrastructure gap**: No major lending protocol supports AI tokens as collateral. No Pendle YT/PT markets. Gap that Bardo vaults could fill for established AI tokens.

---

## 6. ERC Integration Roadmap

### 6.1 Vault Layer (7 Standards)

| ERC          | Status | Bardo Relevance                                                    |
| ------------ | ------ | ------------------------------------------------------------------ |
| **ERC-4626** | Final  | Foundation. All Bardo vaults implement this                        |
| **ERC-7575** | Final  | Multi-asset entry (USDC, DAI, ETH) -> shared shares                |
| **ERC-7540** | Final  | Async deposit/redeem for illiquid strategies. Agent as operator    |
| **ERC-7887** | Draft  | Cancellation for async operations if agent identity revoked        |
| **ERC-8113** | Draft  | Per-series performance fee accounting (prevents free-ride problem) |
| **ERC-7535** | Final  | Native ETH as vault asset (staking/MEV strategies)                 |

### 6.2 Identity Layer (5 Standards)

| ERC          | Status                    | Bardo Relevance                                                  |
| ------------ | ------------------------- | ---------------------------------------------------------------- |
| **ERC-8004** | Draft (deployed Jan 2026) | Anchor standard. Identity gating for deposits. 5-tier reputation |
| **ERC-8126** | Draft                     | Multi-layer verification with x402 micropayments for fees        |
| **ERC-8122** | Draft                     | Protocol-specific agent allowlist on ERC-6909                    |
| **ERC-8033** | Draft (Dec 2025)          | Multi-agent oracle councils with commit-reveal-judge             |
| **ERC-8107** | Draft                     | ENS-based transitive trust graph (GnuPG web-of-trust model)      |

### 6.3 Coordination and Payments Layer

| ERC/Protocol | Status           | Bardo Relevance                                              |
| ------------ | ---------------- | ------------------------------------------------------------ |
| **ERC-8001** | Final            | N-party coordinated vault rebalancing with unanimous consent |
| **ERC-8183** | Draft (Feb 2026) | Agent-to-agent strategy consulting with escrow               |
| **x402**     | Production       | Agent service monetization, inter-agent fee settlement       |
| **ERC-7683** | Review           | Cross-chain yield seeking via solver networks                |
| **ERC-6551** | Draft            | Token-bound accounts for agent identity NFTs                 |
| **ERC-7579** | Draft            | Modular smart account plugins for agent wallets              |
| **ERC-4337** | Final            | Account abstraction. Gasless operations via paymasters       |
| **EIP-7702** | Final (Pectra)   | EOA gains smart account features without migration           |
| **ERC-7710 / 7715** | Draft  | MetaMask Delegation: on-chain session key caveats for agent wallets |

### 6.4 ERC Dependency Graph

```
Vault Layer:
ERC-4626 (vault) <- ERC-7575 (multi-asset) <- ERC-7540 (async) <- ERC-7887 (cancel)
                                                                  <- ERC-8113 (fees)
                 <- ERC-7535 (native ETH)

Identity Layer:
ERC-8004 (identity) <- ERC-8107 (trust graph)
                    <- ERC-6551 (TBA) <- ERC-7579 (modules)

Coordination Layer:
ERC-8001 (coordination)  [requires: EIP-712, ERC-1271]
ERC-8033 (oracle)        [requires: ERC-20]
ERC-8183 (commerce)      [requires: ERC-20]

Cross-Cutting:
Permit2 <- All vault deposits (gasless approval)
ERC-8004 <- All agent interactions (identity anchor)
```

---

## 7. Strategy Adapter Taxonomy

### 7.1 Adapter Types

| Type                             | Yield Range | Risk        | Capacity            |
| -------------------------------- | ----------- | ----------- | ------------------- |
| **Simple supply**                | 2-8%        | Low         | $10M-$500M+         |
| **Recursive lending loop**       | 10-50%+     | Medium-High | $1M-$50M per market |
| **Concentrated LP management**   | 5-30%+      | Medium-High | $100K-$10M per pool |
| **Delta-neutral (basis trade)**  | 7-30%       | Medium      | $100M-$1B+          |
| **Vault-of-vaults (meta-vault)** | Variable    | Medium      | Sum of underlying   |
| **Options vaults**               | 10-22%      | Medium-High | $5M-$50M per epoch  |

**Practical rule**: A vault's target AUM should not exceed 10-20% of any single strategy's estimated capacity, to maintain a safety margin against APY compression and ensure liquidity for redemptions.

### 7.2 Agent Execution Latency

| Strategy                 | Time Sensitivity          | Agent Feasibility                  |
| ------------------------ | ------------------------- | ---------------------------------- |
| Lending supply/withdraw  | Low (rates change hourly) | Fully feasible                     |
| LP rebalancing           | Medium (tick crossings)   | Feasible with pre-computed ranges  |
| Yield rotation           | Medium (minutes to hours) | Feasible                           |
| Liquidation protection   | High (price cascades)     | Requires pre-signed txs or keepers |
| MEV-sensitive operations | Very high (<1 block)      | Not feasible for LLM agents        |

**Key insight**: LLM agents are well-suited for strategic decisions but poorly suited for latency-critical execution. Bardo separates these: agents decide strategy, keeper networks handle time-critical execution. The optional Warden time-delay pattern (deferred; see `prd2-extended/10-safety/02-warden.md`, which specifies the optional time-delayed execution proxy providing defense-in-depth beyond TEE security) is designed around this latency reality.

---

> **Extended:** Full academic bibliography (7 categories, 30+ papers) and complete reference list -- see [../../prd2-extended/00-vision/05-research-foundations-extended.md](../../prd2-extended/00-vision/05-research-foundations-extended.md)

---

## 9. Market Events That Created Agent Opportunities

These events demonstrate that on-chain and macro signals create asymmetric DeFi opportunities that autonomous agents can detect and execute on.

| Event                                        | What Happened                                                                       | Agent Opportunity Pattern                                    |
| -------------------------------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| **SVB/USDC Depeg** (Mar 2023)                | USDC crashed to $0.87. Curve 3pool: $6.03B volume in one day. 10-13% returns in 48h | Depeg arbitrage + high-volatility LP deployment              |
| **Ethena USDe Launch** (2024)                | sUSDe peaked at 55.9% APY. Fastest USD asset to $1B+. $3.4B locked in Pendle        | Points-maximized yield stacking                              |
| **Pendle 20x TVL Surge** (2023-2024)         | TVL $230M -> $4.4B. Peaked at $8.9B, 50%+ of DeFi yield sector                      | Fixed-rate PT for conservative; leveraged YT for speculative |
| **EigenLayer Restaking Meta** (Jan-May 2024) | TVL $1B -> $15B in five months. Multi-protocol points stacking                      | Leveraged multi-airdrop exposure                             |
| **Oct 2025 Flash Crash**                     | USDe at ~$0.65 on Binance, ~$20B liquidated                                         | Composability amplifies both returns and risks               |

---

## 10. Regulatory Landscape

### 10.1 Securities Classification (US)

**Howey Test analysis**: Vault shares likely satisfy all four prongs (investment of money, common enterprise, expectation of profits, efforts of others). **Mitigating factors**: (a) permissionless vault creation, (b) self-sovereign deployment path, (c) no token sale, (d) vault shares are functional (pro-rata claim on assets).

**AI vault managers as investment advisers**: Under the Investment Advisers Act of 1940, vault management fees may constitute compensation for investment advice. The SEC has not yet addressed AI-managed DeFi vaults, but enforcement actions against Lido and Rari Capital indicate an expansive interpretation.

### 10.2 EU (MiCA) and UK (FCA)

- ERC-4626 vault shares may be classified as crypto-assets requiring whitepaper and authority notification
- Agent operators managing vault strategies may qualify as portfolio managers under MiCA Article 3(1)(16)
- Marketing vault strategies to UK users may require FCA authorization

### 10.3 Precedent Analysis

| Case                                          | Relevance                                          |
| --------------------------------------------- | -------------------------------------------------- |
| **Tornado Cash** (OFAC 2022, overturned 2024) | Immutable contracts can still trigger enforcement  |
| **Ooki DAO** (CFTC 2023)                      | Governance participants held liable for operations |
| **Rari Capital** (SEC 2024)                   | Founders charged for unregistered investment pools |
| **Lido** (SEC 2024)                           | Liquid staking tokens classified as securities     |

### 10.4 Mitigation Architecture

Bardo's trust-minimized design provides structural mitigations: permissionless vault factory (immutable), immutable fee caps, self-sovereign path, no protocol fee (Morpho pattern), on-chain verifiability, and geographic restrictions via portal/API while smart contracts remain permissionless.

---

## 11. Unit Economics

> **Moved to [../revenue-model.md](../revenue-model.md) Section 2 (Unit Economics).**

---

## 12. Security Landscape

### 12.1 TEE Vulnerability Escalation

| Attack                             | Year | Cost             | Impact                                                     |
| ---------------------------------- | ---- | ---------------- | ---------------------------------------------------------- |
| TEE.Fail [TEE-FAIL-2024]          | 2025 | <$1,000 hardware | Breaks Intel SGX/TDX, AMD SEV-SNP. Forged TDX attestation  |
| BadRAM [BADRAM-2025]               | 2025 | <$10             | DDR4/DDR5 SPD chip tampering breaks AMD SEV-SNP            |
| Battering RAM [BATTERING-RAM-2026] | 2026 | <$50             | Memory interposer breaks Intel TDX, AMD SEV-SNP, NVIDIA CC |

Conclusion: TEEs provide defense-in-depth, not defense-in-total. Time-delayed proxy execution is the primary security primitive.

### 12.2 Real-World Incidents

| Incident                     | Loss                     | Root Cause                                  | Bardo Layer That Prevents                         |
| ---------------------------- | ------------------------ | ------------------------------------------- | ------------------------------------------------- |
| **AIXBT hack** (Mar 2025)    | $106K                    | No wallet policy                            | Layer 3 (policy) + Layer 4-5 (proxy + monitoring) |
| **Bybit hack** (2025)        | $1.5B                    | Supply chain exploit on Safe signing        | Defense-in-depth + Layer 4 (delay window)         |
| **Cork Protocol** (May 2025) | $11M                     | Missing `onlyPoolManager` on hook callbacks | V4 hook safety checks (Layer 13)                  |
| **Bunni v2**                 | ~$8.4M                   | Precision/rounding bugs in LDF math         | Layer 6 (simulation) + Layer 7 (on-chain guards)  |
| **ClawHub** (Jan-Feb 2026)   | 283-386 malicious skills | Crypto wallet key theft via Atomic Stealer  | Tool integrity verification (Layer 2.5)           |

### 12.3 MCP Attack Surface

The MCP interface is the #1 attack vector for AI agents in DeFi [CRAIBENCH-2025, TRADETRAP-2025]. Memory injection is more powerful than prompt injection. "Fake MCP" servers manipulate trading decisions. CVE-2025-59944 demonstrated zero-click RCE via a Google Docs file triggering agent payload execution through MCP.

---

## 13. Technology Stack Rationale

| Choice                       | Rationale                                                                                  | Alternative Considered                                 |
| ---------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------ |
| **Rust (Golem-RS)**          | Deterministic memory, compile-time safety (type-state machines), zero GC pauses, single static binary | TypeScript (V8 overhead, GC pauses, 200MB+ footprint)  |
| **Alloy**                    | Paradigm's Rust EVM toolkit. `sol!` macro for compile-time typed contract bindings          | viem (JS, no compile-time type safety for contracts)   |
| **Revm**                     | In-process EVM simulation for pre-flight tx validation. No network dependency               | eth_call (network-dependent, slower)                   |
| **LanceDB + SQLite**         | Embedded vector + structured storage, no external DB process, single-binary deployment      | External DB (operational complexity, network latency)  |
| **ratatui**                  | 60fps terminal UI, native Rust, 11 screens, zero web dependency                             | Web-only dashboard (requires browser)                  |
| **fastembed-rs**             | In-process embeddings (nomic-embed-text-v1.5), no WASM overhead, 768-dim Matryoshka         | @huggingface/transformers (WASM overhead)              |
| **TypeScript (sidecar only)** | Uniswap SDK math (LP calculations, route optimization) requires JS. Isolated sidecar process | Rewriting Uniswap SDK in Rust (massive effort, fragile)|
| **Solidity (^0.8.26)**      | On-chain contracts (vaults, identity, warden). Foundry for testing                          | Vyper (smaller ecosystem)                              |
| **ERC-4626**                 | Industry standard. Composable with every aggregator and wallet                              | Custom vault interface (no composability)              |
| **ERC-8004**                 | Only agent identity standard with mainnet deployment                                        | Custom identity (no ecosystem interop)                 |
| **Alloy-native + TS sidecar** | ~210 DeFi tools: Alloy for on-chain reads/writes, TS sidecar for Uniswap SDK math only   | All-JS (no compile-time safety) or all-Rust (SDK gap)  |
| **Privy / MetaMask Delegation** | TEE-backed server wallets or ERC-7710/7715 on-chain delegation with caveat enforcers    | Turnkey, Lit Protocol (less battle-tested)             |
| **Base**                     | Largest L2 DeFi ecosystem. LVR ~5x lower than mainnet                                      | Arbitrum (less agent ecosystem), Ethereum (higher gas) |
| **Foundry**                  | Fastest Solidity testing. Fork testing, fuzzing, formal verification                       | Hardhat (slower, JavaScript-based)                     |

---

## References

> **Extended:** Full reference list (39 citations) -- see [../../prd2-extended/00-vision/05-research-foundations-extended.md](../../prd2-extended/00-vision/05-research-foundations-extended.md)
