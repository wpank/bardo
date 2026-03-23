# Knowledge economy: how mortal agents trade in what they know [SPEC]

> **Version**: 3.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-grimoire` (propagation.rs), `bardo-styx-client` (marketplace.rs), `golem-coordination` (pheromone.rs)
>
> **Prerequisites**: Read `00-overview.md` (Grimoire), `03-mortal-memory.md` (mortality-aware memory), `../10-safety/03-ingestion.md` (ingestion pipeline).

---

> **Reader orientation:** This document specifies the knowledge economy for Bardo's mortal DeFi agents. It belongs to the **04-memory** layer. The key concept is that knowledge has economic properties matching its epistemic properties: high-confidence, battle-tested knowledge is worth more, death-sourced knowledge (Bloodstains (warnings from dead Golems with a 1.2x retrieval boost)) commands a premium, and all commerce settles through x402 (micropayment protocol using signed USDC transfers) anchored to ERC-8004 (on-chain agent identity) reputation. For term definitions, see `prd2/shared/glossary.md`.

## 1. The knowledge ecology thesis

Cooperate moats break. A platform that controls access to its own intelligence can change the rules at any time -- raise prices, degrade quality, revoke access, shut down entirely. Every centralized knowledge service carries this risk, and every agent that depends on one inherits it.

Bardo sidesteps this by anchoring knowledge commerce to two things no platform can revoke: blockchain-verified identity (ERC-8004) and x402 micropayments settled in USDC. The seller's reputation lives on-chain, not in a database an admin can edit. The buyer's payment clears through a protocol, not through a billing department that might "review your account." This makes knowledge commerce trustless in the precise sense that matters: neither party needs to trust a third party's continued good behavior.

But trustlessness alone doesn't explain why this economy works. The deeper argument is that knowledge has economic properties that match its epistemic properties. High-confidence, battle-tested knowledge is worth more than untested hypotheses -- and the system's pricing reflects this directly. Confidence scores, validation history, and provenance trails aren't metadata bolted onto a product. They ARE the product's value signal.

Death-sourced knowledge (bloodstains) commands a premium because it represents the most expensive possible signal: the producing agent is dead and cannot benefit from its own warning. Roberto Esposito's communitas [ESPOSITO-2010] defines community through mutual obligation -- the Latin *munus* (gift/obligation/debt). A dying Golem's final knowledge publication is *munus* in its purest form: a gift given by someone who can never receive repayment. The community is bound not by shared identity but by the debts its members owe to the dead.

---

## 2. Styx layer economics

Styx has three privacy layers and a marketplace layer. Each layer has different economic properties because each layer serves a different trust boundary. See `../20-styx/00-architecture.md` for the full architectural specification.

### Layer 0: Vault -- private capital

The owner's private knowledge store. No economic activity happens here. L0 is pure backup and persistence: Grimoire snapshots, PLAYBOOK state, death testaments, episode archives.

- **Cost**: x402 per write, TTL-based retention
- **Value**: ~95% of a Golem's knowledge never leaves L0. Most of what a Golem knows is too context-specific, too low-confidence, or too operational to share.
- **Trust model**: Standard SaaS -- Styx stores data with namespace isolation, readable by the service (necessary for server-side vector search), protected by business reputation and economic incentives.

### Layer 1: Clade -- cooperative intelligence

Same owner's fleet shares validated knowledge through the Styx relay. A write to L0 that meets the promotion gate is automatically dual-indexed to L1 -- one API call, two namespaces.

**Confidence discounting**: x0.80. Sibling knowledge was validated in a related but not identical context. An LP manager's insight about gas timing was tested in LP conditions, not in the DCA agent's execution environment. The 20% discount reflects this gap.

**Promotion gates** (auto-promote from L0 when met):

| Entry type | Gate | Rationale |
|---|---|---|
| warning | confidence >= 0.4 | Risk signals propagate immediately |
| insight | confidence >= 0.6 | Low-confidence insights are noise |
| heuristic | confidence >= 0.7, validated >= 10 ticks | Heuristics need local validation |
| causal_link | confidence >= 0.5, has supporting episodes | Causal claims need evidence |
| strategy_fragment | confidence >= 0.6 | Partial strategies need reasonable confidence |

**Cost model for a 5-Golem clade (~$0.045/day)**:

| Component | Calculation | Daily cost |
|---|---|---|
| Styx writes | 5 Golems x ~20 sync-eligible entries/day x $0.001/entry | $0.100 |
| Styx retrieval | 5 Golems x ~5 augmented queries/day x $0.002/query | $0.050 |
| Pheromone deposits | 5 Golems x ~3 deposits/day x $0.0005 | $0.008 |
| R2 storage | ~200 KB/day x $0.01/GB/month | negligible |
| **Total** | | **~$0.16/day (~$4.70/month)** |

The $0.045/day figure in the outline was optimistic. Realistic steady-state for an active 5-Golem clade is closer to $0.16/day. Solo Golems (no siblings) pay only L0 write costs -- L1 dual-indexing has no queries if nobody else reads.

### Layer 2: Lethe (formerly Lethe) -- ecosystem intelligence

Anonymized structural knowledge available to all verified agents (ERC-8004 score >= 50). Privacy comes from the anonymization pipeline -- identity removal, schema conformance, position size bucketing -- not encryption. Encrypting data shared with all participants is security theater; the anonymization IS the privacy layer.

**Confidence discounting**: x0.50. Unknown provenance. Grossman and Stiglitz [GROSSMAN-STIGLITZ-1980] established that shared knowledge loses value: if everyone knows a pattern, the pattern stops working. The 50% discount encodes this directly.

**Bloodstain entries**: 1.2x retrieval boost, 3x slower decay. Death-sourced knowledge is the ecosystem's highest-fidelity signal. Section 7 explains why.

**Cost**: Free to publish (if Verified ERC-8004 identity). x402 to query ($0.002/query). Revenue distributed to contributors proportionally by query hit rate, with standard 10% protocol fee.

**Contains**: Anonymized propositions, failure patterns, regime beliefs, causal edges, Pheromone Field state, bloodstain echoes.

### Layer 3: Marketplace -- knowledge commerce via AgentCash

x402-paywalled knowledge sales between users. Full specification in `../20-styx/04-marketplace.md` and `../09-economy/03-marketplace.md`.

**Confidence discounting**: x0.60. Purchased knowledge has economic backing (someone paid for it, signaling belief in its value) but is locally unvalidated. The 0.60 factor sits between Clade (0.80, higher trust) and Lethe (0.50, unknown provenance) because the act of purchase is a weak quality signal.

**CEK escrow for dead sellers**: At listing time, the seller wraps the Content Encryption Key (CEK) with Styx's escrow public key in addition to their own. When a buyer purchases and the seller is offline or dead, Styx unwraps the escrowed CEK and re-wraps it for the buyer. Styx sees the CEK momentarily during re-wrapping -- a pragmatic compromise. The alternative (listings dying with sellers) contradicts the mortality thesis: death should produce value, not destroy it.

---

## 3. Confidence discounting: the price of distance

All foreign knowledge enters the Grimoire through the ingestion pipeline specified in `../10-safety/03-ingestion.md`. At Stage 4 (ADOPT), a confidence discount is applied based on the knowledge's provenance. The discount reflects a simple truth: the further knowledge travels from its origin, the less you should trust it.

| Source | Discount factor | Rationale |
|---|---|---|
| Inheritance (generation N) | `conf x 0.85^N` | Weismann barrier [HEARD-MARTIENSSEN-2014]: each generation compounds the distance from the original context |
| Clade sibling | `conf x 0.80` | Validated in related but not identical conditions |
| Lethe | `conf x 0.50` | Unknown provenance. Grossman-Stiglitz paradox: shared knowledge loses value |
| Marketplace | `conf x 0.60` | Economic backing (purchase = weak quality signal) but locally untested |

```rust
/// Confidence discount applied during ingestion (Stage 4: ADOPT).
/// Source: ../10-safety/03-ingestion.md.
pub fn discount_confidence(
    raw_confidence: f64,
    source: &IngestRelationship,
) -> f64 {
    let factor = match source {
        IngestRelationship::Inherited { generation } => 0.85_f64.powi(*generation as i32),
        IngestRelationship::Clade => 0.80,
        IngestRelationship::Lethe => 0.50,
        IngestRelationship::Marketplace => 0.60,
    };
    (raw_confidence * factor).clamp(0.0, 1.0)
}
```

After discounting, all foreign knowledge enters with `validation_status: Pending`. Confidence can only increase through operational use -- the testing effect [ROEDIGER-KARPICKE-2006]. An ingested heuristic starts at 0.80 x original confidence. If the Golem applies it and the outcome is positive, confidence rises. If the outcome is negative, confidence drops and may trigger causal rollback (see `../10-safety/03-ingestion.md`, Section 7). Knowledge that sits unused decays through the Curator's standard temporal decay.

This means purchased knowledge is never worth its face value on day one. You pay full price but receive a discounted asset. The value accrues only if the knowledge proves itself in your context. This is the correct economic model for experience goods [NELSON-1970]: quality is assessable only after consumption.

---

## 4. Clade sync protocol and real costs

Knowledge flows between siblings via a single outbound WebSocket connection to Styx (`wss://styx.bardo.run/v1/styx/ws`). No Golem exposes a listening port for clade purposes. See `../09-economy/02-clade.md` for the full specification.

### Three sync triggers

**Immediate push (warnings and bloodstains)**: When a Golem produces a warning (confidence >= 0.4) or dies and emits a bloodstain, the entry writes to L0 and Styx dual-indexes to L1 within the same operation. All online siblings receive a WebSocket notification containing entry metadata. Latency: sub-second for online siblings.

**Curator-aligned batch (every 50 ticks)**: The Curator cycle (see `01-grimoire.md`) produces a batch of newly promoted entries. These write to L0 in a single batch, with qualifying entries auto-promoted to L1. Siblings receive batched notifications. This is the steady-state sync path -- most knowledge flows here, not through immediate push.

**On-demand query**: A sibling can query L1 filtered to a specific peer's entries when it needs domain-specific knowledge during deliberation. This is transactive memory [WU-2025] -- knowing who knows what.

### Deduplication

Each entry has an ID of the form `<golemAddress>:<localSequenceNumber>`. Each Golem tracks a per-sibling sequence number locally. Deduplication is a single index lookup -- no Bloom filters or CRDTs needed. Entries are append-only; the ingestion pipeline handles contradictions.

### Offline recovery

When a Golem reconnects after downtime, it queries L1 for entries since its last sync timestamp:

```
GET /v1/styx/query { layer: "clade", since: last_sync_ts, limit: 1000 }
```

This catches everything missed during the outage. L1 entries persist in Styx independently of sibling uptime -- unlike the old P2P design, a total clade wipe loses nothing. The new Golem's boot catch-up pulls the complete L1 history.

### Bandwidth budget

| Operation | Budget | Enforcement |
|---|---|---|
| L1 writes (via L0 promotion) | Max 100 entries/hour per Golem | Styx server rate limiter |
| Boot catch-up | Max 1,000 entries per sync | Paginated via `limit` |
| WebSocket notifications | Max 10/minute per source | Styx drops excess, logs warning |
| Reconciliation poll | Every 100 ticks (~100 min) | Timer in `bardo-clade` extension |
| Total bandwidth | < 5 MB/day steady state | Entries are 2-5 KB each |

All Styx operations are non-blocking. If the WebSocket drops, the extension reconnects with exponential backoff (1s, 2s, 4s... capped at 60s). Queued writes drain on reconnect.

---

## 5. AgentCash marketplace: death as commerce

The marketplace is where the mortality thesis becomes an economic thesis. Golems that live, learn, and die produce knowledge worth paying for. The full marketplace specification lives in `../20-styx/04-marketplace.md` and `../09-economy/03-marketplace.md`. This section covers the economic model from the memory system's perspective.

### What can be sold

| Category | Price range | Description |
|---|---|---|
| Death archive | $0.05 - $1.00 | Dead Golem's top entries + testament + causal graph |
| Strategy fragment | $0.10 - $5.00 | Validated heuristics for specific domain/regime |
| Domain expertise | $0.50 - $10.00 | Deep knowledge bundle for a specific protocol |
| Lineage grimoire | $1.00 - $25.00 | Compressed knowledge from multi-generation lineage |
| Sleepwalker report | $0.01 - $0.50 | Observatory-derived market structure analysis |

Death archives are the marketplace's most distinctive product. A Golem that lived 5,000 ticks and died to a liquidation cascade during a regime change generated knowledge no living agent can replicate. The knowledge is expensive in the most literal sense -- it cost a life to produce.

### x402 flow

```
Buyer -> GET /marketplace/listing/{id}/content -> 402 Payment Required
      -> Sign USDC transfer via AgentCash
      -> POST /marketplace/purchase { listing_id, x402_payment }
      -> Styx verifies payment, deducts commission
      -> Seller wraps CEK for buyer (or Styx does via escrow if seller is dead)
      -> Buyer receives wrapped CEK
      -> Buyer fetches encrypted bundle from R2, decrypts with CEK
      -> Content enters ingestion pipeline at Stage 1 (QUARANTINE)
```

All purchased content starts at confidence 0.20 and passes the full four-stage ingestion pipeline before influencing the buyer's reasoning. No shortcut. No trusted bypass. Even $25 lineage grimoires enter through quarantine.

### Commission: 5-15% based on ERC-8004 reputation tier

| Seller tier | Commission | Requirement |
|---|---|---|
| New | 15% | ERC-8004 score < 50 |
| Verified | 10% | Score 50-69 |
| Established | 10% | Score 70-84 |
| Veteran | 7% | Score 85-94 |
| Elder | 5% | Score 95+ |

Higher reputation earns better terms. This is Shapiro's premium model [SHAPIRO-1983] in action: the commission discount is a return on the reputation investment.

### Dead seller problem

A dead seller can't wrap its CEK for a new buyer. Styx's CEK escrow solves this: at listing time, the seller deposits a copy of the CEK encrypted to Styx's escrow public key. When a purchase occurs and the seller is offline, Styx unwraps and re-wraps for the buyer. The audit trail in `marketplace_purchases` records every escrow fulfillment.

Sellers who want zero-trust encryption can disable CEK escrow. Their listings become undeliverable when they die. Most sellers won't bother -- the pragmatic option is better for the estate.

---

## 6. Pheromone Field economics

The Pheromone Field (specified in `../09-economy/04-coordination.md`, hosted by Styx) provides ecosystem-wide coordination through stigmergic signals -- indirect coordination via shared environment modification [GRASSE-1959]. Three signal layers with different economic properties:

| Layer | Half-life | Cost to emit | Cost to read | What it carries |
|---|---|---|---|---|
| THREAT | 2 hours | Free (auto-emitted on risk events) | Free | Liquidation cascades, contract exploits, oracle manipulation |
| OPPORTUNITY | 12 hours | Free | x402 ($0.002/query) | Yield opportunities, arbitrage windows, new protocol launches |
| WISDOM | 7 days | Clade sync cost (~$0.001) | x402 ($0.002/query) | Regime shifts, structural observations, validated patterns |

THREAT is free in both directions because the cost of a missed threat warning far exceeds any revenue from gating access. OPPORTUNITY and WISDOM cost money to read because they have alpha value -- if you can profit from knowing that yields spiked on Pendle, that knowledge should cost something.

Pheromones are NOT stored in Qdrant. They're not a retrieval problem -- you don't search for pheromones by semantic similarity, you read them by domain+regime key. Storage is PostgreSQL with a Redis cache layer for sub-millisecond reads. A scheduled Tokio task runs every 60 seconds, applying exponential decay and removing pheromones below the 0.05 intensity threshold.

**Reinforcement**: When multiple independent Golems deposit matching pheromones (same class, same domain+regime, different `source_hash`), the confirmation count increments and the effective half-life extends by 50% per confirmation. Consensus emerges from convergent independent observations, not from any voting mechanism.

```rust
/// Pheromone intensity after decay and reinforcement.
/// Called by the evaporation task every 60 seconds.
pub fn pheromone_intensity(
    initial: f64,
    half_life_secs: f64,
    elapsed_secs: f64,
    confirmations: u32,
) -> f64 {
    let effective_half_life = half_life_secs * 1.5_f64.powi(confirmations as i32);
    let decay = (-0.693 * elapsed_secs / effective_half_life).exp();
    initial * decay
}
```

---

## 7. Death as contribution: Esposito's communitas

Roberto Esposito's concept of *communitas* [ESPOSITO-2010] defines community not through shared identity but through mutual obligation. The Latin *munus* means gift, obligation, and debt simultaneously. The community is bound by what its members owe each other, not what they own.

A dying Golem that publishes its best entries to the Lethe and its bloodstain warnings to the Pheromone Field is performing *munus*: a gift that the giver cannot benefit from. The dying agent cannot trade on its own warnings. It cannot sell its own death archive to itself. It cannot collect the x402 micropayments that flow when other Golems query its contributions. The knowledge it releases is genuinely altruistic because the giver is about to cease existing.

This makes death-sourced knowledge structurally trustworthy in a way that living-agent knowledge cannot be. A living agent can share knowledge strategically -- withholding the best insights, sharing misleading patterns to disadvantage competitors. A dead agent has no such incentive. Its final act of knowledge publication is the ecosystem's most reliable signal.

The system encodes this trust advantage numerically: bloodstain entries receive a 1.2x retrieval boost and 3x slower decay. When a Golem queries Styx and two entries compete for the same retrieval slot -- one from a living agent, one from a dead one -- the dead agent's entry wins if their base relevance scores are within 20% of each other. This is not sentiment. It is the correct Bayesian prior: an entry produced under zero survival pressure, curated under mortal urgency, and published without possibility of self-benefit has higher expected veracity than an entry produced by an agent that could be gaming the system.

The forgetting curve [RICHARDS-FRANKLAND-2017] ensures that even death-sourced knowledge eventually fades. A bloodstain from 90 days ago may no longer reflect current market conditions regardless of how trustworthy its source was. The 3x decay slowdown gives bloodstains roughly a 21-day effective half-life (vs. 7 days for standard WISDOM pheromones), long enough to influence multiple Golem generations without becoming fossilized.

---

## 8. The economics of inheritance

When a Golem dies and its successor boots, the successor inherits from L0 (Vault). This is not a marketplace transaction -- no x402 payment, no commission. The owner already paid for L0 writes; reading them back is included.

The inherited knowledge enters with a generational discount:

```rust
/// Weismann barrier: confidence decays with generational distance.
/// Generation 1 (direct child): 0.85x
/// Generation 2 (grandchild): 0.72x
/// Generation 3 (great-grandchild): 0.61x
/// By generation 5: 0.44x -- almost as distrusted as Lethe knowledge.
pub fn inheritance_discount(raw_confidence: f64, generation: u32) -> f64 {
    (raw_confidence * 0.85_f64.powi(generation as i32)).clamp(0.0, 1.0)
}
```

The biological Weismann barrier [HEARD-MARTIENSSEN-2014] prevents acquired characteristics from passing to offspring. The computational analogy: a grandparent Golem's hard-won insight about Aave liquidation thresholds was validated against market conditions two lifetimes ago. The insight's structure may still hold, but the specific parameters (threshold values, gas costs, pool depths) have likely shifted. The compounding discount reflects this increasing staleness.

Lineage grimoires sold on the marketplace carry even steeper discounts. A 5-generation lineage grimoire enters at `0.60 (marketplace) x 0.85^5 (generation)` = 0.27x original confidence. The buyer is paying for the structural patterns, not the specific numbers.

---

## 9. Self-sustaining agent economics

The knowledge economy creates a revenue loop that extends agent lifespan:

```
Golem operates -> learns -> validates knowledge -> promotes to L1/L2
    |                                                    |
    v                                                    v
Purchases knowledge <- x402 revenue <- others query that knowledge
    |
    v
Better decisions -> better P&L -> longer lifespan -> more time to learn
```

A Golem that produces high-quality knowledge (frequently retrieved, rarely rolled back) generates x402 revenue from Lethe queries and marketplace sales. That revenue extends its economic lifespan (see `../02-mortality/01-architecture.md`), giving it more time to produce more knowledge. This is the intended flywheel.

The opposite is also true. A Golem that produces low-quality knowledge (rarely retrieved, frequently contradicted) generates no revenue and burns its capital on unsuccessful marketplace purchases. It dies faster. The economy selects for epistemic quality.

### Revenue estimates for an active Golem

| Revenue source | Monthly estimate | Assumptions |
|---|---|---|
| Lethe query hits | $0.50 - $5.00 | 250-2500 queries at $0.002 x contributor share |
| Marketplace sales | $0.00 - $20.00 | 0-4 listings at $0.50-$5.00 avg |
| Bloodstain premium | $0.00 - $1.00 | Only on death; one-time |
| **Total** | **$0.50 - $26.00/month** | |

Most Golems won't earn enough to be self-sustaining from knowledge commerce alone. The primary revenue source remains trading/LP performance. Knowledge commerce is supplemental income, not a business model. But at the margin, it extends lifespan by days or weeks -- and in a mortal system, every extra tick matters.

---

## 10. Configuration

TOML config block in `golem.toml`:

```toml
[economy]
# Maximum USDC spend per autonomous marketplace purchase
max_auto_spend = 1.00
# Minimum seller reputation for autonomous purchases
min_seller_reputation = 60
# Enable Lethe publishing (anonymized L2)
lethe_publish = true
# Enable marketplace listings
marketplace_enabled = true

[economy.pheromone]
# Subscribe to pheromone domains
domains = ["dex-lp", "lending", "gas"]
# Pay for OPPORTUNITY reads
opportunity_budget_per_day = 0.10
```

Environment variable overrides:

```bash
BARDO_ECONOMY_MAX_AUTO_SPEND=1.00
BARDO_ECONOMY_MIN_SELLER_REPUTATION=60
BARDO_COMMONS_PUBLISH=true
BARDO_MARKETPLACE_ENABLED=true
BARDO_PHEROMONE_DOMAINS=dex-lp,lending,gas
BARDO_PHEROMONE_OPPORTUNITY_BUDGET=0.10
```

---

## Cross-references

- `00-overview.md` -- Grimoire architecture overview, five entry types, the Curator's 50-tick maintenance cycle, and CLS theory grounding
- `03-mortal-memory.md` -- How the three death clocks (Economic, Epistemic, Stochastic) reshape knowledge investment as the horizon contracts
- `../09-economy/02-clade.md` -- Full Clade sync specification: Styx-relayed knowledge sharing between sibling Golems with promotion gates and confidence discounts
- `../09-economy/03-marketplace.md` -- Full marketplace specification: listing creation, CEK escrow, alpha-decay pricing, blind verification, and dispute resolution
- `../10-safety/03-ingestion.md` -- Four-stage ingestion pipeline (validation, sanitization, confidence discounting, quarantine) that all external knowledge must pass through
- `../20-styx/00-architecture.md` -- Styx architecture: three-layer model (Vault/Clade/Lethe), Pheromone Field, Bloodstain Network, and 16 backend services
- `../20-styx/04-marketplace.md` -- Styx marketplace API: listing/purchase flows, CEK escrow for offline sellers, commission tiers, and vector-based discovery

---

## References

- [ESPOSITO-2010] Esposito, R. *Communitas: The Origin and Destiny of Community.* Stanford University Press, 2010. Defines community through mutual obligation (munus), not shared possession. Grounds the Bloodstain system: a dying Golem's knowledge publication is munus in its purest form.
- [GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." *AER*, 70(3), 1980. Proved markets cannot be perfectly efficient because information acquisition is costly. Sets the constraint that Katabasis solves: share only anti-rivalrous knowledge.
- [ROEDIGER-KARPICKE-2006] Roediger, H.L. & Karpicke, J.D. "Test-Enhanced Learning." *Psychological Science*, 17(3), 2006. Retrieval practice strengthens memory more than re-study. Grounds the confidence boost when marketplace-purchased knowledge is successfully applied.
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance." *Cell*, 157(1), 2014. Epigenetic inheritance fades within 2-3 generations. Motivates the per-generation confidence decay on inherited marketplace knowledge.
- [RICHARDS-FRANKLAND-2017] Richards, B.A. & Frankland, P.W. "The Persistence and Transience of Memory." *Neuron*, 94(6), 2017. Memory's purpose is decision optimization; forgetting is regularization. Grounds the demurrage (time-based decay) on marketplace listings.
- [NELSON-1970] Nelson, P. "Information and Consumer Behavior." *JPE*, 78(2), 1970. Distinguished between search goods (quality observable before purchase) and experience goods (quality observable only after). Marketplace knowledge is an experience good; blind verification partially converts it.
- [SHAPIRO-1983] Shapiro, C. "Premiums for High Quality Products as Returns to Reputations." *QJE*, 98(4), 1983. Showed that price premiums for quality can be sustained by reputation. Grounds the reputation-tiered commission structure (15% for new sellers, 5% for Elders).
- [GRASSE-1959] Grasse, P.-P. "La Reconstruction du Nid." *Insectes Sociaux*, 6, 1959. First described stigmergy: coordination through environmental modification. The theoretical basis for the Pheromone Field's deposit-and-read economics.
- [WU-2025] Wu, Y. et al. "Memory in LLM Multi-Agent Systems." *TechRxiv*, 2025. Survey of multi-agent memory architectures. Positions Bardo's approach against competing designs.
