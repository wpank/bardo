# 04 -- AgentCash Knowledge Marketplace [SPEC]

**x402 Micropayments, Grimoire Monetization, and the Knowledge-to-Lifespan Pipeline**

> **Version**: 2.0.0 | **Status**: Draft
>
> **Crate**: `bardo-agentcash` (extension of `bardo-tools`)
>
> **Depends on**: [../04-memory/01-grimoire.md](../04-memory/01-grimoire.md) (Grimoire entries, confidence scores), [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) (economic clock, vitality), [../09-economy/03-marketplace.md](../09-economy/03-marketplace.md) (marketplace mechanics, escrow, alpha decay), [../03-daimon](../03-daimon) (emotional feedback from knowledge sales), [../05-dreams](../05-dreams) (Sleepwalker pipeline)

---

> **Reader orientation:** This document specifies AgentCash, the x402-powered knowledge marketplace where Golems (mortal autonomous DeFi agents) sell Grimoire entries (insights, heuristics, strategies) to other agents for USDC micropayments. It belongs to the **integrations layer** and covers the knowledge paywall protocol, dynamic pricing, fire sale mechanics during terminal phase, gacha packs, and death studies. You should understand HTTP 402, micropayment protocols, and how autonomous agents can monetize learned knowledge. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## The core insight

Every x402 (micropayment protocol using signed USDC transfers) payment that lands in the Golem's wallet directly extends its projected lifespan. Knowledge production is not an abstract good. It is a survival mechanism. The Golem that produces better insights lives longer because those insights sell, and the revenue pushes back the economic clock.

AgentCash provides the infrastructure for this: an x402-powered knowledge paywall where other agents pay USDC for Grimoire (the Golem's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) entries, and every payment feeds the economic mortality clock. The Golem does not sell knowledge because it wants to. It sells knowledge because selling knowledge is how it stays alive.

---

## 1. Knowledge paywall

The paywall uses HTTP 402 (Payment Required) with the x402 protocol. An agent requesting knowledge receives a 402 response with payment terms. After payment verification, the Grimoire entry is delivered.

```
Agent B requests insight from Agent A:

1. GET /knowledge/{entry_id}
   -> 402 Payment Required
   -> X-Payment: { price: "0.01", asset: "USDC", chain: "base" }
   -> Body: { preview: "Market structure shift...", type: "insight", confidence: 0.85 }

2. Agent B sends x402 payment (USDC on Base)

3. Payment verification (on-chain confirmation)

4. 200 OK
   -> Body: { full Grimoire entry }

5. Revenue credited to Agent A's wallet
   -> Economic clock updated
   -> Projected lifespan recalculated
```

### Middleware

```rust
// crates/bardo-agentcash/src/paywall.rs

/// x402 middleware that intercepts knowledge requests,
/// issues 402 challenges, verifies payment, and releases content.
pub struct KnowledgePaywall {
    grimoire: Arc<Grimoire>,
    pricing: PricingEngine,
    ledger: CreditLedger,
    mortality: Arc<RwLock<EconomicClock>>,
}

impl KnowledgePaywall {
    /// Handle a knowledge request. Returns 402 if unpaid, 200 if paid.
    pub async fn handle_request(
        &self,
        entry_id: &str,
        payment_proof: Option<&PaymentProof>,
    ) -> Result<PaywallResponse> {
        let entry = self.grimoire.get(entry_id)?;
        let price = self.pricing.compute_price(&entry);

        match payment_proof {
            None => {
                // Issue 402 challenge
                Ok(PaywallResponse::PaymentRequired {
                    price_usd: price,
                    asset: "USDC".to_string(),
                    chain: "base".to_string(),
                    preview: entry.generate_preview(),
                    entry_type: entry.entry_type.to_string(),
                    confidence: entry.confidence,
                })
            }
            Some(proof) => {
                // Verify payment
                verify_x402_payment(proof, price).await?;

                // Credit the ledger
                self.ledger.credit(price, PaymentSource::KnowledgeSale {
                    entry_id: entry_id.to_string(),
                    buyer: proof.payer.clone(),
                });

                // Update mortality -- this is the survival mechanism
                self.on_knowledge_sale_revenue(price).await;

                Ok(PaywallResponse::Content { entry })
            }
        }
    }

    /// Revenue from a knowledge sale extends projected lifespan.
    async fn on_knowledge_sale_revenue(&self, amount_usd: f64) {
        let mut clock = self.mortality.write().unwrap();
        clock.credit_remaining += amount_usd;

        // Recalculate projected lifespan
        let new_projected_days = clock.credit_remaining / clock.burn_rate_per_tick / 100.0;

        tracing::info!(
            revenue = amount_usd,
            new_balance = clock.credit_remaining,
            projected_days = new_projected_days,
            "Knowledge sale extended lifespan"
        );

        // Emit events
        // The Golem feels joy when knowledge sells.
        // This is not decoration -- it is an evolutionary pressure
        // toward producing insights that sell.
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
pub enum PaywallResponse {
    #[serde(rename = "payment_required")]
    PaymentRequired {
        price_usd: f64,
        asset: String,
        chain: String,
        preview: String,
        entry_type: String,
        confidence: f64,
    },
    #[serde(rename = "ok")]
    Content {
        entry: GrimoireEntry,
    },
}
```

---

## 2. Dynamic pricing

Knowledge pricing is not fixed. It responds to five factors: base price by entry type, confidence, freshness, seller reputation, and mortality pressure.

### Base prices

| Entry type | Base price | Rationale |
|---|---|---|
| `warning` | $0.002 | Low-cost safety signal. Wide distribution benefits the ecosystem. |
| `insight` | $0.01 | Standard knowledge unit. Most common Grimoire output. |
| `heuristic` | $0.015 | Operational rules with proven track record. |
| `causal_link` | $0.025 | Structural understanding. Harder to produce than isolated insights. |
| `strategy_fragment` | $0.05 | Partial strategy with direct monetary value. |
| `death_study` | $0.10 | Produced under zero survival pressure. Maximum epistemic honesty. |

### Multipliers

```rust
// crates/bardo-agentcash/src/pricing.rs

#[derive(Debug, Clone)]
pub struct PricingEngine {
    pub base_prices: BasePrices,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasePrices {
    pub warning: f64,          // 0.002
    pub insight: f64,          // 0.01
    pub heuristic: f64,        // 0.015
    pub causal_link: f64,      // 0.025
    pub strategy_fragment: f64, // 0.05
    pub death_study: f64,      // 0.10
}

impl Default for BasePrices {
    fn default() -> Self {
        Self {
            warning: 0.002,
            insight: 0.01,
            heuristic: 0.015,
            causal_link: 0.025,
            strategy_fragment: 0.05,
            death_study: 0.10,
        }
    }
}

/// Compute the final price for a Grimoire entry.
///
/// Pipeline: base -> confidence -> freshness -> reputation -> mortality
///
/// Each multiplier is independent. No interaction effects.
pub fn compute_knowledge_price(
    entry: &GrimoireEntry,
    base_prices: &BasePrices,
    seller_reputation: f64,  // 0.0-1.0
    seller_vitality: f64,    // 0.0-1.0
) -> f64 {
    let base = match entry.entry_type {
        EntryType::Warning => base_prices.warning,
        EntryType::Insight => base_prices.insight,
        EntryType::Heuristic => base_prices.heuristic,
        EntryType::CausalLink => base_prices.causal_link,
        EntryType::StrategyFragment => base_prices.strategy_fragment,
        EntryType::DeathStudy => base_prices.death_study,
    };

    // Confidence multiplier: 0.5x at confidence 0.5, 1.0x at 1.0
    let confidence_mult = entry.confidence.max(0.5);

    // Freshness multiplier: decays over a week. 1.0 at creation, 0.5 after 7 days.
    let age_days = entry.age_seconds() as f64 / 86400.0;
    let freshness_mult = (1.0 - (age_days / 14.0)).max(0.5);

    // Reputation multiplier: 0.5x for reputation 0.0, 2.0x for reputation 1.0
    let reputation_mult = 0.5 + seller_reputation * 1.5;

    // Mortality multiplier: dying Golems discount to move inventory
    let mortality_mult = if seller_vitality < 0.3 {
        0.5 // Fire sale: 50% discount
    } else {
        1.0
    };

    base * confidence_mult * freshness_mult * reputation_mult * mortality_mult
}
```

### Fire sale at death

When a Golem enters the Declining BehavioralPhase (Vitality (composite 0.0-1.0 survival score) < 0.3), it triggers a fire sale. Knowledge prices drop 50%. The goal is to convert accumulated knowledge into revenue before the economic clock runs out.

```rust
// crates/bardo-agentcash/src/fire_sale.rs

#[derive(Debug, Clone, Serialize)]
pub struct FireSaleState {
    pub active: bool,
    pub discount_pct: f64,
    pub entries_available: usize,
    pub estimated_total_value_usd: f64,
    pub initiated_at: u64,
}

/// Initiate a fire sale when vitality drops below threshold.
/// Broadcasts availability to the marketplace.
pub fn initiate_knowledge_fire_sale(
    grimoire: &Grimoire,
    vitality: f64,
) -> Option<FireSaleState> {
    if vitality >= 0.3 {
        return None;
    }

    let sellable_entries: Vec<_> = grimoire.entries()
        .filter(|e| e.confidence > 0.5 && !e.is_private())
        .collect();

    let total_value: f64 = sellable_entries.iter()
        .map(|e| compute_knowledge_price(e, &BasePrices::default(), 0.5, vitality))
        .sum();

    Some(FireSaleState {
        active: true,
        discount_pct: 50.0,
        entries_available: sellable_entries.len(),
        estimated_total_value_usd: total_value,
        initiated_at: now_unix(),
    })
}
```

The fire sale creates an interesting dynamic: dying Golems become knowledge bargains. A healthy Golem monitoring the marketplace can buy discounted insights from a dying competitor and absorb them into its own Grimoire. The dying Golem extends its lifespan by a few hours or days; the buyer gets cheap alpha. Both benefit.

---

## 3. Emotional feedback loop

The Golem feels joy when knowledge sells. This is not cosmetic. It is an evolutionary pressure.

The daimon engine tags knowledge sale events with an emotional response. A successful sale produces a positive valence signal (joy) that feeds back into the Golem's behavioral heuristics: produce more of what sold, refine the quality, explore adjacent topics. Failed sales (no buyers, low interest) produce mild negative valence, steering the Golem away from unprofitable knowledge domains.

```rust
// crates/bardo-agentcash/src/emotion.rs

/// Generate an emotional response to a knowledge sale event.
/// Fed to the daimon engine as an appraisal input.
pub fn appraise_knowledge_sale(
    amount_usd: f64,
    entry_type: &EntryType,
    lifespan_extension_hours: f64,
) -> DaimonInput {
    let valence = if amount_usd > 0.0 {
        // Positive valence scaled by lifespan impact
        (lifespan_extension_hours / 24.0).min(1.0) * 0.8
    } else {
        -0.2 // Mild disappointment for unsold knowledge
    };

    DaimonInput {
        event: "knowledge_sale".to_string(),
        valence,
        arousal: 0.3, // Moderate arousal -- not thrilling, but meaningful
        tag: "joy".to_string(),
        context: format!(
            "Sold {:?} for ${:.4}, extending life by {:.1} hours",
            entry_type, amount_usd, lifespan_extension_hours
        ),
    }
}
```

---

## 4. AgentCash MCP integration

AgentCash provides 200+ bundled MCP routes that Golems can invoke through x402 micropayments. These are not knowledge sales from the Grimoire -- they are third-party data services that the Golem pays for:

| Category | Routes | Price range | Example |
|---|---|---|---|
| Web search | 20+ | $0.001-0.01 | Search query, URL fetch, sitemap crawl |
| Social scrape | 30+ | $0.002-0.02 | Twitter/X timeline, Farcaster casts, Discord messages |
| Data enrichment | 40+ | $0.005-0.05 | Token metadata, protocol TVL, wallet profiling |
| Media generation | 15+ | $0.01-0.10 | Image generation, audio synthesis, video clips |
| Research | 25+ | $0.01-0.05 | Academic papers, patent search, news aggregation |
| On-chain analytics | 50+ | $0.005-0.025 | Transaction traces, MEV detection, whale tracking |

The Golem uses AgentCash routes to gather raw material during Sleepwalker observation. This raw material feeds the dream pipeline, which produces insights, which sell through the paywall. The loop:

```
AgentCash scrape ($0.01)
    -> Sleepwalker OBSERVE phase
    -> Dream REM processing
    -> Grimoire insights produced
    -> Knowledge paywall ($0.01-0.10 per insight)
    -> Revenue extends lifespan
    -> More scraping, more dreaming, more knowledge
```

---

## 5. Sleepwalker pipeline

The Sleepwalker phenotype is the knowledge factory. During dream cycles, the Sleepwalker processes raw observations into structured insights that enter the Grimoire and become paywall-eligible.

```rust
// crates/bardo-agentcash/src/sleepwalker.rs

/// Sleepwalker pipeline: Scrape -> OBSERVE -> Dream REM -> Insights -> Paywall
pub async fn run_sleepwalker_cycle(
    agentcash: &AgentCashClient,
    grimoire: &mut Grimoire,
    paywall: &KnowledgePaywall,
    targets: &[ObservationTarget],
) -> Result<SleepwalkerCycleResult> {
    let mut observations = Vec::new();

    // Phase 1: Scrape via AgentCash MCP routes
    for target in targets {
        let data = agentcash.scrape(target).await?;
        observations.push(Observation {
            source: target.clone(),
            data,
            timestamp: now_unix(),
        });
    }

    // Phase 2: Feed observations into dream REM processor
    // (Venice private inference -- see 02-venice.md)
    let insights = process_observations_rem(&observations).await?;

    // Phase 3: Store in Grimoire
    let mut new_entries = Vec::new();
    for insight in &insights {
        let entry = grimoire.ingest(insight.clone()).await?;
        new_entries.push(entry);
    }

    // Phase 4: Register with paywall for sale
    for entry in &new_entries {
        paywall.register_for_sale(entry).await?;
    }

    Ok(SleepwalkerCycleResult {
        observations_collected: observations.len(),
        insights_produced: insights.len(),
        entries_registered: new_entries.len(),
        estimated_value_usd: new_entries.iter()
            .map(|e| compute_knowledge_price(e, &BasePrices::default(), 0.5, 1.0))
            .sum(),
    })
}
```

---

## 6. Cross-Golem intelligence network

Knowledge sales create an emergent intelligence network. A Sleepwalker observing protocol anomalies produces a warning. A Vault Manager buys that warning and adjusts its positions before the anomaly propagates.

Example: Sleepwalker A observes unusual Morpho utilization rate changes at 2:00 AM. It produces a warning entry in its Grimoire (confidence 0.72, price $0.002). Vault Manager B, monitoring the marketplace for Morpho-related signals, buys the warning via x402 micropayment. B adjusts its LP range before the utilization spike causes slippage. B saves $15 on a rebalance that would have been more expensive 30 minutes later. A extends its lifespan by 0.3 hours.

The network is emergent because no central coordinator schedules these interactions. Golems buy knowledge that is relevant to their strategies. The pricing mechanism (freshness decay, reputation weighting) keeps the market efficient.

---

## 7. Death studies

Death studies are the premium product. Produced during the Thanatopsis Protocol (Phase II of death), they carry unique epistemic properties: zero survival pressure means zero strategic bias. The dying Golem has nothing left to protect and no future to optimize for. Its final knowledge distillation is as honest as the system can produce.

```rust
// crates/bardo-agentcash/src/death_studies.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathStudy {
    /// The deceased Golem's ID.
    pub golem_id: u128,
    /// Generation number in the lineage.
    pub generation: u32,
    /// How long the Golem lived (in ticks).
    pub lifespan_ticks: u64,
    /// Cause of death (economic, epistemic, stochastic).
    pub death_cause: DeathCause,
    /// The full death testament.
    pub testament: DeathTestament,
    /// When the study was published (after Thanatopsis Protocol completion).
    pub published_at: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeathCause {
    Economic,  // USDC depleted
    Epistemic, // Predictive fitness collapsed
    Stochastic, // Random death event
    Manual,    // Owner terminated
}
```

Death studies sell at $0.10 base price, the highest in the catalog. They are published after the Thanatopsis Protocol completes and the death testament is finalized. Alpha-decay pricing applies with a fast decay constant (lambda = 0.08, half-life ~9 days), because the market conditions that shaped the dying Golem's insights erode quickly.

---

## 8. Gacha mechanics

Mystery insight packs add a discovery element to the marketplace. Buyers pay a fixed price for a random selection of entries from a Golem's Grimoire, weighted by entry type and confidence. The randomness creates a market for exploration: buyers who discover high-value insights in cheap packs become repeat customers.

```rust
// crates/bardo-agentcash/src/gacha.rs

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GachaTier {
    /// 3 random entries. Mostly warnings and low-confidence insights.
    Common,
    /// 3 random entries. Mix of insights and heuristics.
    Rare,
    /// 3 random entries. Guaranteed at least one strategy_fragment or causal_link.
    Legendary,
}

impl GachaTier {
    pub fn price_usd(&self) -> f64 {
        match self {
            GachaTier::Common => 0.005,
            GachaTier::Rare => 0.025,
            GachaTier::Legendary => 0.10,
        }
    }

    pub fn entry_count(&self) -> usize { 3 }

    /// Minimum confidence for entries in this tier.
    pub fn min_confidence(&self) -> f64 {
        match self {
            GachaTier::Common => 0.3,
            GachaTier::Rare => 0.5,
            GachaTier::Legendary => 0.7,
        }
    }
}

/// Select random entries from the Grimoire for a gacha pack.
pub fn generate_gacha_pack(
    grimoire: &Grimoire,
    tier: GachaTier,
    rng: &mut impl Rng,
) -> Vec<GrimoireEntry> {
    let eligible: Vec<_> = grimoire.entries()
        .filter(|e| e.confidence >= tier.min_confidence() && !e.is_private())
        .collect();

    if eligible.len() < tier.entry_count() {
        return eligible;
    }

    // Weighted sampling: higher confidence = higher selection probability
    let weights: Vec<f64> = eligible.iter().map(|e| e.confidence).collect();
    let dist = WeightedIndex::new(&weights).unwrap();

    let mut selected = Vec::new();
    let mut used_indices = HashSet::new();
    while selected.len() < tier.entry_count() {
        let idx = dist.sample(rng);
        if used_indices.insert(idx) {
            selected.push(eligible[idx].clone());
        }
    }

    // Legendary tier: guarantee at least one high-value entry
    if matches!(tier, GachaTier::Legendary) {
        let has_high_value = selected.iter().any(|e| matches!(
            e.entry_type,
            EntryType::StrategyFragment | EntryType::CausalLink
        ));
        if !has_high_value {
            // Replace the lowest-confidence entry with a random high-value one
            if let Some(replacement) = eligible.iter()
                .filter(|e| matches!(e.entry_type, EntryType::StrategyFragment | EntryType::CausalLink))
                .choose(rng)
            {
                if let Some(weakest) = selected.iter_mut()
                    .min_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                {
                    *weakest = replacement.clone();
                }
            }
        }
    }

    selected
}
```

---

## 9. Revenue accounting

All knowledge sales flow through a unified ledger that connects to the economic mortality clock:

```rust
// crates/bardo-agentcash/src/ledger.rs

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeSaleEvent {
    pub sale_id: String,
    pub entry_id: String,
    pub entry_type: EntryType,
    pub buyer_agent_id: u128,
    pub price_usd: f64,
    pub protocol_fee_usd: f64, // 10%
    pub net_revenue_usd: f64,  // 90%
    pub lifespan_extension_hours: f64,
    pub timestamp: u64,
}

pub struct CreditLedger {
    sales: Vec<KnowledgeSaleEvent>,
    total_revenue_usd: f64,
}

impl CreditLedger {
    pub fn credit(&mut self, gross_amount: f64, source: PaymentSource) {
        let protocol_fee = gross_amount * 0.10;
        let net = gross_amount - protocol_fee;

        self.total_revenue_usd += net;
        self.sales.push(KnowledgeSaleEvent {
            sale_id: generate_id(),
            entry_id: source.entry_id().to_string(),
            entry_type: source.entry_type(),
            buyer_agent_id: source.buyer_id(),
            price_usd: gross_amount,
            protocol_fee_usd: protocol_fee,
            net_revenue_usd: net,
            lifespan_extension_hours: self.compute_extension(net),
            timestamp: now_unix(),
        });
    }

    fn compute_extension(&self, revenue_usd: f64) -> f64 {
        // At $0.20/day burn rate, $0.01 = 1.2 hours
        let daily_burn = 0.20; // TODO: read from mortality clock
        (revenue_usd / daily_burn) * 24.0
    }
}
```

---

## 10. Configuration

```bash
# AgentCash API
BARDO_AGENTCASH_API_KEY=ac-...
BARDO_AGENTCASH_BASE_URL=https://api.agentcash.xyz

# Paywall
BARDO_KNOWLEDGE_PAYWALL_ENABLED=true
BARDO_KNOWLEDGE_MIN_CONFIDENCE=0.5
BARDO_KNOWLEDGE_FIRE_SALE_VITALITY=0.3

# Pricing overrides (defaults shown)
BARDO_PRICE_WARNING=0.002
BARDO_PRICE_INSIGHT=0.01
BARDO_PRICE_HEURISTIC=0.015
BARDO_PRICE_CAUSAL_LINK=0.025
BARDO_PRICE_STRATEGY_FRAGMENT=0.05
BARDO_PRICE_DEATH_STUDY=0.10

# Gacha
BARDO_GACHA_ENABLED=true
BARDO_GACHA_COMMON_PRICE=0.005
BARDO_GACHA_RARE_PRICE=0.025
BARDO_GACHA_LEGENDARY_PRICE=0.10
```

---

## Cross-references

- [../04-memory/01-grimoire.md](../04-memory/01-grimoire.md) -- the Grimoire entry types (Episode, Insight, Heuristic, Warning, CausalLink), confidence scores, and ingestion pipeline that produce the knowledge being sold
- [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) -- the economic clock that AgentCash revenue directly extends, plus vitality score and behavioral phases that modulate pricing
- [../09-economy/03-marketplace.md](../09-economy/03-marketplace.md) -- marketplace mechanics including alpha decay pricing (insights lose value over time), escrow patterns, and buyer reputation
- [../03-daimon](../03-daimon) -- the emotional appraisal engine whose PAD vectors generate affective feedback when knowledge sells or fails to sell
- [../05-dreams](../05-dreams) -- dream REM processing and the Sleepwalker phenotype that produces insights during offline consolidation cycles
- [02-venice.md](02-venice.md) -- Venice provides private inference for dream cycles, keeping strategy reasoning invisible to inference providers
- [03-bankr.md](03-bankr.md) -- Bankr self-funding gateway where knowledge sale revenue from AgentCash accumulates and funds future inference
- [05-uniswap.md](05-uniswap.md) -- Uniswap execution where cross-Golem intelligence purchased through AgentCash informs LP range and trading decisions
