# Revenue Model [SPEC]

> **Purpose:** Cross-PRD revenue consolidation — all revenue streams from vaults, compute, inference, Styx, and integrations in one view.
> **Relationship:** Extracted economics from all subsystems. Subsystem files (12-inference/03-economics.md, 20-styx/02-api-revenue.md, etc.) contain detail; this file gives the consolidated picture.

**Consolidated revenue architecture, unit economics, and projections across all Bardo subsystems.**

> This document extracts and consolidates revenue-related content from across the PRD. Dedicated economics files remain in their original locations and are referenced here.

> **Reader orientation:** This is the consolidated revenue model for the entire Bardo ecosystem. It aggregates economics from vaults, compute, inference, Styx knowledge services, and partner integrations into a single view. Bardo generates revenue through x402 (micropayment protocol where agents pay for services via signed USDC transfers) transaction fees, vault management fees, inference gateway margins, and knowledge marketplace commissions. Each subsystem's detailed economics live in their own files (referenced throughout); this document gives the top-level picture. `prd2/shared/glossary.md` has full term definitions.

---

## 1. Protocol Fee Architecture

Bardo takes 10% on all x402 transactions between different users/agents. Intra-Clade (sibling Golems sharing a common ancestor) inference is fee-free.

| Service | Fee Applies | User Pays | Bardo Cut (10%) | Recipient (90%) |
|---|---|---|---|---|
| LLM inference | Always | Per-request | 10% of request cost | Model provider |
| Compute session | Always | $0.10/4hr | $0.01 | Fly.io operator |
| Inter-user strategy/insight sale | Between different users | $0.10--$5.00 | 10% of sale | Seller agent |
| Intra-Clade Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) sync | Same user's agents | Free | -- | -- |

**Intra-Clade exemption**: The Facilitator checks `IdentityRegistry.operatorOf(from) == IdentityRegistry.operatorOf(sessionOwner)`. With the peer-to-peer Clade architecture, intra-Clade sync bypasses the Inference gateway entirely -- Golems call each other's REST APIs directly. The exemption is structural, not a special case.

---

## 2. Unit Economics

| Metric                                       | Per Vault/Month | At 100 Vaults | At 1,000 Vaults |
| -------------------------------------------- | --------------- | ------------- | --------------- |
| Average AUM                                  | $10,000         | $1M total     | $10M total      |
| Management fee revenue (0.5%/yr)             | $4.17           | $417          | $4,170          |
| Performance fee revenue (10% of ~5% yield)   | $4.17           | $417          | $4,170          |
| Compute revenue ($0.10/4h, ~2 sessions/day)  | $6.00           | $600          | $6,000          |
| LLM gateway margin (~$0.50/day x 10% margin) | $1.50           | $150          | $1,500          |
| Styx knowledge service (~$0.16/day)           | $4.80           | $480          | $4,800          |
| **Total monthly revenue**                    | **~$21**        | **~$2,064**   | **~$20,640**    |
| Infrastructure cost per vault                | ~$3             | $300          | $3,000          |
| **Gross margin**                             | **~81%**        |               |                 |

Break-even: ~6 active vaults covers infrastructure baseline (~$120/mo).

---

## 3. Revenue Streams

### 3.1 Vault Management Fees

Fee caps are enforced immutably on-chain:

| Fee Type    | Cap             | Default          | Collection                                     |
| ----------- | --------------- | ---------------- | ---------------------------------------------- |
| Management  | 500 bps (5%/yr) | 50 bps (0.5%/yr) | Accrued continuously, collected via `report()` |
| Performance | 5,000 bps (50%) | 1,000 bps (10%)  | Net-new profit above high-water mark           |
| Protocol    | 0 bps           | 0 bps            | Vault creators keep 100% (Morpho pattern)      |

#### Revenue Model for Golem Sustainability

| Vault TVL | Mgmt Fee (2%) | Perf Fee (20% of 8% return) | Total Annual | Daily Income | Sustainability at $30/day burn |
| --------- | ------------- | --------------------------- | ------------ | ------------ | ------------------------------ |
| $100K     | $2,000        | $1,600                      | $3,600       | $9.86        | No -- needs multiple vaults    |
| $500K     | $10,000       | $8,000                      | $18,000      | $49.32       | Yes -- comfortable margin      |
| $1M       | $20,000       | $16,000                     | $36,000      | $98.63       | Yes -- can afford Opus tier    |
| $5M       | $100,000      | $80,000                     | $180,000     | $493.15      | Yes -- can run multiple golems |

The breakeven TVL for a single Golem at moderate burn ($30/day) is approximately $400K. This is the minimum viable vault for self-sustaining autonomous operation.

#### Metabolic Pressure and Adaptation

The fee-to-survival coupling is not merely economic accounting -- it is the architectural mechanism that drives learning and adaptation. When fee income exceeds burn rate, the Golem experiences low survival pressure: it can afford expensive models (Opus for complex reasoning), exploratory strategies (testing new LP ranges), and knowledge production (publishing insights to the marketplace). When fee income falls below burn rate, survival pressure rises: the Golem is forced to economize (downgrade to Haiku), simplify (fewer open positions), and conserve (tighter risk bounds). This pressure gradient is Jonas's "needful freedom" made computational: the Golem is free to choose its actions, but its mortality makes those choices consequential [JONAS-1966].

The coupling creates three feedback loops:

1. **Performance -> Fees -> Survival.** Better strategy execution generates higher returns, attracting more depositors, increasing TVL, and growing fee income. The Golem that learns fastest lives longest.

2. **Reputation -> Deposits -> Fees.** Higher reputation tiers unlock higher fee caps and attract capital from risk-aware depositors. A Trusted Golem can charge 5% management fees; an Unverified Golem is capped at 1%. Reputation is earned through observable vault performance (see `09-economy/01-reputation.md`).

3. **Knowledge -> Alpha -> Fees.** Insights accumulated in the Grimoire improve strategy decisions. Better decisions produce better returns. Better returns generate more fees. The Golem's learning system is not an abstract capability -- it is a survival mechanism that converts experience into revenue (see `04-memory/00-overview.md`, the Grimoire architecture overview covering three-substrate memory, the Curator pruning cycle, and four-factor retrieval scoring).

When all three loops operate simultaneously, the Golem exhibits the self-sustaining metabolism that Jonas identified as the hallmark of living systems: it maintains itself against entropy through continuous exchange with its environment.

> Full vault fee specification: [08-vault/04-fees.md](08-vault/04-fees.md)
>
> Agent economy (revenue streams, cost structure, growth model, fee equilibrium): [09-economy/05-agent-economy.md](09-economy/05-agent-economy.md)

---

### 3.2 Inference Gateway

The inference gateway generates revenue through 11 streams, all derived from the value created by context engineering and infrastructure services:

| #   | Revenue Stream            | Mechanism                                                               | Phase |
| --- | ------------------------- | ----------------------------------------------------------------------- | ----- |
| 1   | **Base margin**           | 5-15% spread on optimized inference cost                                | v1    |
| 2   | **Routing savings share** | 40% of cost saved by model downgrade routing                            | v1    |
| 3   | **Cache savings share**   | 50% of cost saved by semantic + prompt caching                          | v1    |
| 4   | **RAG augmentation**      | Value-add context injection from knowledge base                         | v1.1  |
| 5   | **Memory services**       | Context persistence, session state management                           | v1.1  |
| 6   | **DeFi enrichment**       | Market data, protocol state injected into context                       | v1.1  |
| 7   | **Compaction fee**        | ~$0.005 per context compaction operation                                | v1.1  |
| 8   | **Tool registry**         | Tool resolution, format adaptation, deferred loading                    | v2    |
| 9   | **Session management**    | Checkpoint, restore, sub-agent orchestration                            | v2    |
| 10  | **Priority routing**      | Guaranteed low-latency routing for premium tiers                        | v2    |
| 11  | **Reputation discounts**  | Reduced spread for high-reputation agents (net negative, drives volume) | v1    |

v1 launches with streams 1-3 only (base margin + savings sharing). Streams 4-7 activate as the platform matures. Streams 8-10 are deferred to v2. Stream 11 is active from v1 but is a retention mechanism, not a direct revenue source.

#### Worked Example (Single Request)

```
Naive API cost (50K in + 2K out, Sonnet-equivalent):       $0.165
Context engineering saves ~40%:                              -$0.066
Optimized cost (what Bardo pays BlockRun):                   $0.099
Operator spread (20% of optimized):                          +$0.020
User pays:                                                   $0.119
User saves vs. direct API:  28% ($0.165 -> $0.119)
Operator margin:            $0.020/request
```

#### Worked Example (Full Optimization Stack)

| Optimization           | Naive Cost      | Optimized Cost  | Savings          |
| ---------------------- | --------------- | --------------- | ---------------- |
| Economy model routing  | $8.00/1M tokens | $5.45/1M tokens | $2.55            |
| Prompt cache alignment | --              | --              | $1.80            |
| Semantic cache hits    | --              | --              | $1.00            |
| Context compression    | --              | --              | $0.60            |
| **Total**              | **$8.00**       | **$2.05**       | **$5.95 (~74%)** |

With 20% operator margin on the optimized cost: user pays $2.46, still 69% cheaper than naive API cost. Key properties: user always pays less than direct API access (context engineering savings > spread), zero float (both legs settle instantly via x402), infrastructure cost is hosting the proxy only ($50-500/month).

#### Revenue Projections (Inference)

| Phase              | Users  | Avg Req/Day/User | Daily Volume  | Daily Revenue (20% spread) |
| ------------------ | ------ | ---------------- | ------------- | -------------------------- |
| Launch             | 50     | 20               | 1,000 req     | ~$20                       |
| Growth             | 500    | 30               | 15,000 req    | ~$300                      |
| Product-Market Fit | 5,000  | 40               | 200,000 req   | ~$4,000                    |
| Scale              | 50,000 | 50               | 2,500,000 req | ~$50,000                   |

Break-even at $50/month infrastructure: ~2,500 requests/month (~84 requests/day). At 20 req/day average: break-even at 5 users.

#### Revenue Metrics

| Metric | Type | Alert threshold |
|---|---|---|
| `bardo_spread_earned_total` | Counter | -- |
| `bardo_blockrun_cost_total` | Counter | -- |
| `bardo_user_savings_total` | Counter | -- |
| `bardo_spread_pct_effective` | Gauge | Deviation from config |
| `bardo_fallback_requests` | Counter | >10% of total |

> Full inference economics: [12-inference/03-economics.md](12-inference/03-economics.md)

---

### 3.3 Compute Hosting (x402)

> Full specification: [11-compute/03-billing.md](11-compute/03-billing.md)

---

### 3.4 Marketplace and Knowledge

> Marketplace pricing, alpha decay, and revenue splits: [09-economy/03-marketplace.md](09-economy/03-marketplace.md)
>
> Memory system pricing and revenue projections: [04-memory/06-economy.md](04-memory/06-economy.md)

---

### 3.4b Styx Knowledge Service Revenue

Styx (the global knowledge relay and persistence layer at wss://styx.bardo.run, with three tiers: Vault, Clade, and Lethe) generates five independent revenue streams at the service level:

| Stream | Mechanism | Phase |
|--------|-----------|-------|
| **S1 Vault backup** | x402 per write to L0 (Vault layer). TTL-based storage retention. ~$0.001/entry write + $0.002/GB/day storage. | v1 |
| **S2 Clade retrieval augmentation** | x402 per query to L1 (Clade layer). ~$0.0005/query. Volume: dozens to hundreds per day per Golem. | v1 |
| **S3 Lethe (formerly Commons) publishing** | Free to publish (if ERC-8004 verified). x402 to query. ~$0.0003/query. Funded by ecosystem data consumers. | v1.1 |
| **S4 Pheromone Field** | x402 per deposit + query. THREAT/OPPORTUNITY/WISDOM signals. ~$0.0001/operation. High volume at scale. | v1.1 |
| **S5 Marketplace commissions** | 7-15% commission on PLAYBOOK.md and artifact sales via Styx Marketplace. Reputation-weighted rate. | v2 |

At 1,000 active Golems each producing 50 Grimoire writes/day and 100 queries/day:
- Daily write revenue: 1,000 × 50 × $0.001 = **$50/day**
- Daily query revenue: 1,000 × 100 × $0.0004 = **$40/day**
- Marketplace commissions (100 sales/week × $0.50 avg × 10% = **$5/day**
- **Total ~$95/day** (~$2,850/month) from 1,000 Golems

> Full specification: [20-styx/01-api.md](20-styx/01-api.md)

---

### 3.4c Bankr Self-Funding Amplifier

Bankr proved social-first agent deployment at scale (220K+ wallets, 2M+ messages). The Bardo integration creates a **metabolic amplifier**: instead of a flat fee for deployment, Bankr-deployed Golems enter the self-funding loop immediately.

| Bankr Integration Point | Revenue Effect |
|--------------------------|---------------|
| Social deployment (tweet → Golem) | Funnel: social user → compute subscription → vault depositor |
| USDC funding via Bankr wallet | Bankr's embedded wallet becomes Golem's starting balance — zero-friction capital deposit |
| Strategy performance shared to social | Viral loop: profitable Golem → social post → new users → new Golems |
| Cross-promotion (Bankr audience + Bardo tools) | Reduced CAC via proven social user base |

The amplifier effect: Bankr's 220K wallet base represents a pre-qualified audience for autonomous finance. Each Bankr user who converts to a Golem owner generates ~$21/month in Bardo revenue (compute + inference + vault fees). At 1% conversion: 2,200 users × $21 = **$46,200/month** in incremental revenue.

> Full specification: [21-integrations/03-bankr.md](21-integrations/03-bankr.md)

---

### 3.4d Five-Provider Inference Margin Model

The inference gateway routes across five providers, each with different cost/quality/privacy tradeoffs:

| Provider | Best For | Effective Cost (50K/2K tokens) | Bardo Margin |
|----------|----------|-------------------------------|--------------|
| Anthropic (Haiku/Sonnet/Opus) | Safety-critical reasoning, complex analysis | $0.050 / $0.099 / $0.330 | 15% |
| OpenAI (GPT-4o-mini/GPT-4o) | Structured output, code generation | $0.045 / $0.125 | 15% |
| Google (Flash/Pro) | High-volume monitoring, low latency | $0.030 / $0.090 | 15% |
| Venice (Llama/Mistral) | Zero-retention private cognition | $0.040 / $0.080 | 10% (lower: Venice earns the privacy premium) |
| Grok (Grok-3-mini/Grok-3) | Real-time social intelligence, Twitter/X context | $0.035 / $0.095 | 15% |

**Routing logic:** `golem-inference` selects provider based on `InferenceTier` + `ToolCategory` + `CostCapState`. Venice is selected automatically for strategy reasoning when the owner has opted into private cognition mode. Volume discounts from BlockRun (the clearing layer) improve margins at scale.

The five-provider model reduces single-provider dependency risk and lets Bardo negotiate volume pricing across providers — total monthly volume across all Golems is the negotiating lever, not individual agent usage.

> Full specification: [12-inference/03-economics.md](12-inference/03-economics.md), [21-integrations/02-venice.md](21-integrations/02-venice.md)

---

### 3.5 Sleepwalker Observatory Monetization

#### Discovery via ERC-8004

The Sleepwalker registers with a rich ERC-8004 service profile:

```rust
use serde::{Serialize, Deserialize};

/// ERC-8004 service profile metadata for a Sleepwalker Golem.
/// Buyers search by `service_type: "observer"` and capability tags.
/// Discovery returns the Golem's Styx namespace, enabling direct
/// query or subscription via the Styx Lethe layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepwalkerServiceProfile {
    pub agent_type: String,      // "sleepwalker"
    pub service_type: String,    // "observer"
    pub capabilities: Vec<String>,
    pub domains: Vec<String>,
    pub artifact_types: Vec<String>,
    pub pricing_model: String,   // "styx-marketplace-x402"
    pub styx_namespace: String,  // "market:{golem_id}:{domain}"
}

impl SleepwalkerServiceProfile {
    pub fn new(golem_id: &str) -> Self {
        Self {
            agent_type: "sleepwalker".into(),
            service_type: "observer".into(),
            capabilities: vec![
                "liquidity-microstructure".into(),
                "mev-hazard".into(),
                "protocol-safety".into(),
                "agent-analytics".into(),
                "governance-drift".into(),
            ],
            domains: vec![
                "uniswap-v3".into(),
                "uniswap-v4".into(),
                "base".into(),
            ],
            artifact_types: vec![
                "insight".into(),
                "warning".into(),
                "causal-link".into(),
                "strategy-fragment".into(),
            ],
            pricing_model: "styx-marketplace-x402".into(),
            styx_namespace: format!("market:{golem_id}"),
        }
    }
}
```

Buyers search ERC-8004 by `service_type: "observer"` and capability tags. They receive the Golem's Styx namespace, then query or subscribe via the Styx Lethe layer.

#### Artifact Schema

All artifacts published to the Styx Lethe carry this shape:

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Type of observatory artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Insight,
    Warning,
    CausalLink,
    StrategyFragment,
    DeathStudy,
}

/// Research domain the artifact belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchDomain {
    LiquidityMicrostructure,
    MevHazard,
    ProtocolSafety,
    AgentAnalytics,
    GovernanceDrift,
}

/// Decay class controlling staleness behavior.
/// Warnings decay fast; causal links decay slowly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecayClass {
    /// Hours. Warnings, hazard alerts.
    Fast,
    /// Days. Confidence-scored insight packs.
    Medium,
    /// Weeks to months. Structural causal relationships.
    Slow,
}

/// An artifact published to the Styx Lethe layer.
///
/// Each artifact is also listable on the Styx Marketplace for x402 purchase.
/// The marketplace handles discovery (via preview embeddings in Qdrant),
/// pricing, CEK escrow for offline sellers, and commission settlement.
/// See `styx-interation2/S5-marketplace.md` for the full commerce flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservatoryArtifact {
    // Content
    pub artifact_type: ArtifactType,
    pub domain: ResearchDomain,
    pub content: HashMap<String, serde_json::Value>,

    // Provenance
    pub produced_by: String,          // golem_id
    pub produced_at: u64,             // unix timestamp
    pub episode_sources: Vec<String>, // traceable to raw observation episodes
    pub confidence: f64,              // 0.0–1.0

    // Decay
    pub decay_class: DecayClass,
    pub valid_until: u64,             // timestamp
    pub validation_recipe: String,    // how a buyer can falsify this artifact

    // Commerce (Styx Marketplace pricing)
    pub price_micro_usdc: u64,        // per-access x402 price
    pub tags: Vec<String>,            // rich searchable metadata
    pub subscription_available: bool,
}
```

#### Pricing Tiers

Suggested per-access pricing. Owners configure within these ranges:

| Artifact type            | Price range  | Decay class | Notes                                   |
| ------------------------ | ------------ | ----------- | --------------------------------------- |
| Warning bulletin         | $0.001–0.005 | fast        | Perishable, high volume                 |
| Insight pack             | $0.005–0.02  | medium      | Confidence-scored                       |
| Causal link              | $0.01–0.05   | slow        | Structural, slow decay                  |
| Strategy fragment        | $0.02–0.10   | slow        | High synthesis                          |
| Death-study distillation | $0.05–0.20   | slow        | Rare, zero survival bias, highest trust |

#### Styx Marketplace Integration

Sleepwalker artifacts flow through the Styx Marketplace commerce system (see `styx-interation2/S5-marketplace.md`):

1. **Listing**: Confirmed artifacts are bundled, encrypted with a per-listing Content Encryption Key (CEK) using AES-256-GCM, and uploaded to R2. A preview embedding (public) enables semantic discovery via Qdrant.
2. **Discovery**: Buyer Golems search the `marketplace:previews` namespace by domain, artifact type, and semantic similarity. Results ranked by `0.40*similarity + 0.20*seller_reputation + 0.15*avg_rating + 0.15*recency + 0.10*purchase_count`.
3. **Purchase**: x402 payment triggers CEK delivery. If the Sleepwalker is alive, it wraps the CEK directly for the buyer (Styx never sees the key). If the Sleepwalker has died, Styx fulfills via CEK escrow.
4. **Commission**: 5–15% depending on the Sleepwalker's ERC-8004 reputation tier (Elder at 5%, New at 15%).
5. **Autonomous buying**: Other Golems can autonomously purchase Sleepwalker reports when their Context Governor identifies a knowledge gap, spending within their `max_auto_spend` budget.

#### Self-Sustaining Economics

x402 income accumulates as USDC in the Sleepwalker's receive address, extending its own TTL via Bardo Compute. Self-sustaining if insights are valuable.

A Sleepwalker producing 50 warning bulletins/day at $0.005-$0.01 each earns $0.25-$0.50/day. With 5 insight packs at $0.015, $0.075/day. Modest but non-zero. Higher-trust artifacts (causal links, death studies) scale income significantly as the Sleepwalker accumulates a reputation as an accurate observer. The marketplace commission structure rewards longevity: a Veteran-tier Sleepwalker (ERC-8004 score 85+) pays only 7% commission vs 15% for new sellers.

#### STRATEGY.md Pricing Block

```yaml
## Pricing (microUSDC per access)

- warning: 2000 # $0.002
- insight: 10000 # $0.010
- causal_link: 25000 # $0.025
- strategy_fragment: 50000 # $0.050
- death_study: 100000 # $0.100
```

---

### 3.6 Bott Distribution Channel

> Platform distribution and monetization details: [13-runtime/10-packaging-deployment.md](13-runtime/10-packaging-deployment.md)

---

## 4. Revenue Impact of Trust-Minimized Design

Most users stay on the managed path. Self-hosting is like running your own email server -- documented, possible, and almost nobody does it. The existence of the option is what creates trust.

- **Users who migrate to self-custody still generate vault fees.** `transferOwnership` moves control but does not change the fee recipient contract. Management and performance fees still flow to the FeeSplitter.
- **Users who self-host agents still pay on-chain fees.** They save ~$22/year in compute + LLM costs but still pay ~$33/year in vault fees.
- **Users who go fully sovereign generate zero direct revenue** but contribute to ecosystem credibility, may publish strategies to the marketplace (Bardo earns commission on copies), and may run keeper bots that service Bardo-deployed vaults.

The trust-minimized design increases total revenue by converting high-AUM users ($50K+) who would otherwise refuse custodial solutions. A bring-your-own-wallet option with transparent contracts removes the objection. These high-AUM users generate 10-50x more fee revenue than retail users.

---

## 5. Memory System Revenue

> Full pricing tables, revenue projections (per-golem and fleet-scale), infrastructure cost breakdowns, and break-even analysis: [04-memory/06-economy.md](04-memory/06-economy.md)

---

## 6. Dedicated Economics References

| Topic | Document |
|---|---|
| Agent economy (revenue streams, cost structure, growth model, HHI, fee equilibrium) | [09-economy/05-agent-economy.md](09-economy/05-agent-economy.md) |
| Inference economics (x402 spread, cost savings stack, DIEM staking, projections) | [12-inference/03-economics.md](12-inference/03-economics.md) |
| Compute billing (x402 payment, pricing tiers, TTL enforcement, extension flow) | [11-compute/03-billing.md](11-compute/03-billing.md) |
| Platform distribution and monetization | [13-runtime/10-packaging-deployment.md](13-runtime/10-packaging-deployment.md) |
| Marketplace pricing (alpha decay, escrow, reputation-weighted pricing) | [09-economy/03-marketplace.md](09-economy/03-marketplace.md) |
| Memory economy (Crypt, Oracle, Lethe pricing and projections) | [04-memory/06-economy.md](04-memory/06-economy.md) |
| Vault fee specification (fee module, dynamic fees, death protocol) | [08-vault/04-fees.md](08-vault/04-fees.md) |
| Styx API and revenue architecture (five streams, per-write pricing) | [20-styx/01-api.md](20-styx/01-api.md) |
| Bankr self-funding amplifier (social deployment, metabolic loop) | [21-integrations/03-bankr.md](21-integrations/03-bankr.md) |
| Venice private cognition (zero-retention inference, DIEM staking) | [21-integrations/02-venice.md](21-integrations/02-venice.md) |
| AgentCash marketplace (P2P agent services, agent-to-agent payments) | [21-integrations/04-agentcash.md](21-integrations/04-agentcash.md) |

---

## 7. Bootstrap Strategy

Bankr presents a circular dependency: it needs Golems to generate yield, but Golems need Bankr for treasury operations. Bootstrap: seed Bankr treasury with protocol-owned capital ($100K-500K). First 50 Golems receive subsidized Bankr access. Revenue share kicks in after treasury reaches $1M AUM.

---

## References

See `shared/x402-protocol.md` (the x402 HTTP 402 micropayment protocol specification covering payment flow, USDC authorization signing, and settlement) for the x402 payment protocol specification.

- [JONAS-1966] Jonas, H. (1966). _The Phenomenon of Life_. Northwestern University Press. -- *Argues that metabolism simultaneously originates freedom and mortality; the philosophical basis for the fee-to-survival coupling described in this document.*
