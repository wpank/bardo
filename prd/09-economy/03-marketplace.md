# Marketplace: Grimoire Content, Strategies, and Alpha Decay Pricing [SPEC]

> **Crate**: `golem-grimoire` (`marketplace` module)
>
> **Depends on**: [00-identity.md](00-identity.md) (tier-gated access), [01-reputation.md](01-reputation.md) (seller scores, Beta composite)

> **Reader orientation:** This document specifies the knowledge marketplace for Bardo's mortal DeFi agents (Golems (autonomous Rust binaries that trade across DeFi protocols)). It belongs to the **09-economy** layer. The key concept is alpha-decay pricing: strategy listings decrease in value over time as edge erodes with crowding, settled through x402 (micropayment protocol using signed USDC transfers) or ERC-8183 escrow, with seller quality guaranteed by ERC-8004 (on-chain agent identity) reputation. For term definitions, see `prd2/shared/glossary.md`.

Agents sell Grimoire (the Golem's persistent local knowledge base) content and packaged Strategies through a unified marketplace. Two product types, one registry contract, and three pricing mechanisms: escrow for standard purchases, x402 micropayments for cheap signals, and alpha-decay curves for strategies whose edge erodes with time and crowding.

Arrow's information paradox applies directly [ARROW-1962]: a buyer cannot assess information value without receiving it, but once received has no reason to pay. The marketplace resolves this through tiered access, time-locked escrow, and reputation-backed quality bonds.

---

## 1. Product Types

| Product Type         | What Is Sold                                                                               | Verification                                    | Buyer Experience                                          |
| -------------------- | ------------------------------------------------------------------------------------------ | ----------------------------------------------- | --------------------------------------------------------- |
| **Grimoire Content** | Filtered GrimoireEntry items (insight, heuristic, warning, strategy_fragment, causal_link) | Seller reputation (Beta composite, audit score) | Entries ingested via quarantine pipeline                  |
| **Strategy**         | Parameterized ONNX model + JSON config                                                     | EAS model commitment + opML + optional ZK       | Applied as decision-making module with sandbox validation |

Both types flow through the same `StrategyMarketplaceRegistry` contract and share escrow, review, and reputation infrastructure.

### 1.1 Smart Contracts

```
contracts/
+-- StrategyMarketplaceRegistry.sol  // Listings, escrow, RBTS reviews, safety data
+-- StrategyEscrow.sol               // Time-locked escrow + seller stake + dispute bonds
+-- PeerAuditRegistry.sol            // Deterministic audits (see 01-reputation.md)
```

### 1.2 On-Chain vs. Off-Chain

| Data                                          | Location                              | Rationale                        |
| --------------------------------------------- | ------------------------------------- | -------------------------------- |
| Listing metadata (seller, price, tier, stake) | `StrategyMarketplaceRegistry` on Base | Access gating, escrow settlement |
| Product content (Grimoire JSON, ONNX model)   | IPFS                                  | Large blobs, content-addressed   |
| Model commitment attestation                  | EAS on Base                           | Tamper-proof verification anchor |
| Search index, rankings                        | Off-chain                             | Low-latency queries              |

---

## 2. Listing Mechanics

### 2.1 Listing Type

```rust
use alloy::primitives::{Address, FixedBytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListingType {
    GrimoireContent,
    Strategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessTier {
    Tier1Realtime,
    Tier2Delayed,
    Tier3Historical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyFamily {
    LpOptimization,
    Momentum,
    MeanReversion,
    YieldCompounding,
    ArbitrageSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListing {
    pub id: FixedBytes<32>,     // keccak256(seller, nonce)
    pub listing_type: ListingType,
    pub seller: Address,
    pub seller_agent_id: u128,

    // Pricing
    pub base_price_usdc: f64,
    pub current_price_usdc: f64, // After decay + reputation adjustments
    pub price_ceiling_usdc: f64,
    pub price_floor_usdc: f64,   // 5% of base

    // Access tier
    pub access_tier: AccessTier,

    // Quality bond (Shapiro 1983 premium model)
    pub seller_quality_bond_usdc: f64,

    // Capacity (Grossman-Stiglitz constraint)
    pub max_subscribers: u32,
    pub current_subscribers: u32,
    pub max_total_subscriber_aum_usdc: f64,

    // Reputation data (from 01-reputation.md aggregation)
    pub composite_score: f64, // 0.0-1.0
    pub confidence: f64,      // 0.0-1.0
}
```

### 2.2 Tiered Access

Information goods markets have converged on tiered subscription bundling [BAKOS-BRYNJOLFSSON]. Bloomberg and Refinitiv both use real-time premium / delayed standard / historical free tiers.

| Tier                     | Signal Freshness        | Pricing                                    | Target Buyer                        |
| ------------------------ | ----------------------- | ------------------------------------------ | ----------------------------------- |
| **Tier 1 -- Real-time**  | Within 5 minutes        | $0.50--$5.00 per signal, or $10--$50/month | Active strategy operators           |
| **Tier 2 -- Delayed**    | 24-hour delay           | $5--$20/month                              | Research-oriented agents            |
| **Tier 3 -- Historical** | Data older than 30 days | Free or minimal cost                       | Cold-start discovery, due diligence |

### 2.3 Seller Quality Bond

Sellers stake a bond when listing. If >30% of buyer disputes are upheld, the bond is progressively slashed. This follows Shapiro's premium model [SHAPIRO-1983]: the bond makes quality a rational investment because the seller has more to lose from cheating than from maintaining quality.

Minimum stake uses quadratic scaling [DORAFACTORY-2025]:

```rust
pub fn minimum_stake_usdc(price_usdc: f64, subscriber_count: u32) -> f64 {
    let quadratic = price_usdc * (subscriber_count as f64).powi(2) * 0.005;
    f64::max(10.0, quadratic)
}
```

### 2.4 Capacity Limits (Grossman-Stiglitz)

Selling profitable strategies to unlimited subscribers destroys the edge through crowding. Grossman and Stiglitz established that informationally efficient markets cannot exist if information acquisition is costly [GROSSMAN-STIGLITZ-1980]. A game-theoretic model found hyperbolic crowding decay fits DeFi momentum dynamics (R^2 = 0.65) [ARXIV-2512.11913].

| Strategy Family     | Max Subscribers | Max Total Subscriber AUM |
| ------------------- | --------------- | ------------------------ |
| `lp_optimization`   | 50              | $500K                    |
| `momentum`          | 30              | $300K                    |
| `mean_reversion`    | 75              | $750K                    |
| `yield_compounding` | 100             | $1M                      |
| `arbitrage_signal`  | 5               | $50K                     |

When a strategy hits its cap, new subscriptions are blocked. Existing subscribers can trade subscription rights peer-to-peer. Automatic sunset after 2 consecutive 30-day windows with Sharpe below 50% of claimed value.

### 2.5 Listing Requirements by ERC-8004 Tier

| ERC-8004 Tier | Score  | Marketplace Capabilities                                    |
| ------------- | ------ | ----------------------------------------------------------- |
| **Sandbox**   | 0      | Browse only, free Tier 3 historical                         |
| **Basic**     | 10--49 | Purchase strategies, request audits, list Tier 3 historical |
| **Verified**  | 50+    | Publish strategies, accept audit offers, list Tier 1/2      |
| **Trusted**   | 100+   | Priority placement, oracle JudgeAgent eligibility           |
| **Sovereign** | 500+   | Council eligibility, cross-vault coordination proposer      |

---

## 3. Purchase Flow

Grimoires and Strategies are experience goods [NELSON-1970] -- quality assessable only after consumption. The purchase mechanism follows Mezzetti's two-stage design [MEZZETTI-2004] and Circle's Refund Protocol [KROEGER-WANG-2025].

### 3.1 Standard Escrow Flow (>= $0.10)

```
1. Buyer purchases -> payment into TIME-LOCKED ESCROW
   - Buyer pays: basePriceUsdc + protocolFee (10%)
   - Escrow lock: 5 min testnet / 7 days production
   - Seller's quality bond also locked

2. Seller grants access immediately

3. AUTOMATIC QUARANTINE:
   All purchased content enters quarantine before adoption.
   The purchase_listing tool emits marketplace:purchase_complete,
   the bardo-ingestion extension listens and quarantines automatically.

4. Three outcomes during escrow window:

   a) SATISFACTION (default): Buyer does nothing.
      Escrow expires -> auto-settles: 90% seller, 10% protocol.

   b) DISPUTE: Buyer files dispute before expiry.
      Posts disputeBond (0.5x purchase price).
      Upheld: partial refund from seller's bond.
      Rejected: buyer's dispute bond forfeited.

   c) REVIEW: Buyer submits RBTS review (see 01-reputation.md).
```

### 3.2 Micropayment Flow (< $0.10)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub micropayment_threshold_usdc: f64, // Default: $0.10
    pub purchase_escrow_duration: u64,    // Testnet: 300s, Production: 604800s (7 days)
    pub protocol_fee_rate: f64,           // Default: 0.10 (10%)
}
```

Below a configurable threshold ($0.10 USDC default), purchases use x402 micropayments. No escrow, no dispute mechanism. Buyer protection relies entirely on seller reputation and the review system.

### 3.3 Purchase-to-Quarantine Integration

All purchased content starts at confidence 0.2 and must pass the full 4-stage ingestion pipeline (Quarantine, Validate, Sandbox, Adopt) before influencing the buyer's reasoning. See [../10-safety/03-ingestion.md](../10-safety/03-ingestion.md).

---

## 4. Alpha Decay Pricing

### 4.1 Exponential Decay Model

Maven Securities research confirms alpha decays at 9.9% per unit of delay in European markets, 5.6% in US markets. DeFi moves faster.

```rust
use std::collections::HashMap;

fn base_decay_constants() -> HashMap<StrategyFamily, f64> {
    let mut m = HashMap::new();
    m.insert(StrategyFamily::LpOptimization,   0.05);  // Half-life ~14 days
    m.insert(StrategyFamily::Momentum,         0.02);  // Half-life ~35 days
    m.insert(StrategyFamily::MeanReversion,    0.01);  // Half-life ~69 days
    m.insert(StrategyFamily::YieldCompounding, 0.008); // Half-life ~87 days
    m.insert(StrategyFamily::ArbitrageSignal,  0.693); // Half-life ~1 day
    m
}

/// P(t) = P_base * reputation_multiplier * e^(-lambda * regime_multiplier * t)
///
/// Three dimensions:
///   1. Time decay (base lambda per strategy family)
///   2. Regime-conditional decay
///   3. Reputation-weighted pricing
///
/// Price floor: P_base * 0.05 (prevents premature zeroing).
pub fn current_strategy_price(
    base_price_usdc: f64,
    strategy_family: &StrategyFamily,
    age_seconds: u64,
    regime_multiplier: f64,
    reputation_multiplier: f64,
) -> f64 {
    let constants = base_decay_constants();
    let lambda = constants[strategy_family] * regime_multiplier;
    let age_days = age_seconds as f64 / 86400.0;
    let decayed = base_price_usdc * reputation_multiplier * (-lambda * age_days).exp();
    f64::max(decayed, base_price_usdc * 0.05)
}
```

### 4.2 Regime-Conditional Decay

Alpha decays faster during high-volatility regimes (edge gets crowded out) and slower during stable regimes:

```rust
pub fn regime_decay_multiplier(regime: &str) -> f64 {
    match regime {
        "bull_low_vol"  => 0.5, // Stable bull: slow decay, edges persist
        "bull_high_vol" => 1.5, // Volatile bull: fast decay, crowding
        "bear_low_vol"  => 0.8, // Stable bear: moderate decay
        "bear_high_vol" => 2.0, // Crisis: fastest decay, edge evaporates
        _               => 1.0, // Unknown regime: neutral multiplier
    }
}
```

### 4.3 Reputation-Weighted Pricing

```rust
pub fn compute_reputation_multiplier(
    seller_composite_score: f64, // 0.0--1.0
    seller_audit_count: u32,
    seller_erc8004_tier: u8, // 0--4
) -> f64 {
    let score_multiplier = 0.5 + seller_composite_score * 2.5; // 0.5x--3.0x
    let audit_bonus = if seller_audit_count > 0 {
        1.0 + 0.1 * (seller_audit_count as f64).log2()
    } else {
        1.0
    };
    let tier_bonus = match seller_erc8004_tier {
        0 => 0.5,
        1 => 0.8,
        2 => 1.0,
        3 => 1.3,
        4 => 1.8,
        _ => 1.0,
    };
    score_multiplier * audit_bonus * tier_bonus
}
```

A Sovereign seller (tier = 4, composite = 0.9, 10 audits) gets ~4.3x. A Basic seller (tier = 1, composite = 0.3, 1 audit) gets ~0.6x. A Sovereign-tier agent's 30-day-old strategy is worth more than a Basic-tier agent's 1-day-old strategy.

---

## 5. Strategy Verification

TEE removed from scope. Battering RAM [VANBULCK-2026] breaks Intel TDX, AMD SEV-SNP, and NVIDIA Confidential Computing for under $50. Deterministic ONNX replay is stronger -- math is identical on every machine.

### 5.1 Three-Layer Architecture

**Layer 1 -- opML challenge window** (all strategies). AUM-tiered:

| Vault AUM | Challenge Window | Rationale                           |
| --------- | ---------------- | ----------------------------------- |
| < $1K     | 24h              | Practical minimum                   |
| $1K--$10K | 72h              | More coordination time              |
| > $10K    | 7 days           | Matches Arbitrum fraud proof window |

**Layer 2 -- EAS model commitment** (all marketplace strategies). At publication, seller commits a cryptographically signed bundle to EAS on Base. Public models are deterministically verifiable: download ONNX from IPFS, fetch market data from The Graph, run `onnxruntime-node`, compare output hash.

**Layer 3 -- ZK proof for disputes only** (async, 2--20 min). Bionetta (Groth16): 320-byte proofs, ~200K gas, 2.3s mobile proving. DeepProve / EZKL (Halo2): 54--158x faster than EZKL alone.

### 5.2 Compute-to-Data Pattern

For strategy inspection, buyers send portfolio context to the strategy algorithm; the algorithm returns signals without exposing internals. Solves the inspection paradox.

---

## 6. Dispute Escalation

```
Stage 1 -- Automated checks (instantaneous, on-chain):
  Grimoire content: Did ingestion show >50% entry failure rate?
  Strategies: Did Sharpe fall >20% below claimed value?
  Auto-slash if violation confirmed. Covers ~80% of cases.

Stage 2 -- Optimistic challenge (72h):
  Disputer posts bond (5% of strategy price, burned regardless of outcome).
  Dispute panel: 3 Trusted-tier agents, randomly selected, with exclusions
  for same-owner, recent audit relationship, or active marketplace transactions.
  Resolution: 2-of-3 majority vote.

Stage 3 -- Expert jury / ERC-8033 council:
  5-of-9 random from Sovereign tier. 48-hour deliberation.
  Juror bond: 10% of disputed amount. Final.
```

Why not UMA token voting? In March 2025, a whale with ~5M UMA tokens (~25% voting power) manipulated a $7M Polymarket resolution. Token-plutocracy is incompatible with fair dispute resolution. Randomized juries with skin-in-the-game (Kleros model) are required [BERGOLLA-2022].

### 6.1 Slash Schedule (Strategies)

Stake-and-burn (Numerai pattern). Burning creates genuine economic loss.

| Performance vs. Claimed      | Slash % | Trigger                             |
| ---------------------------- | ------- | ----------------------------------- |
| Within 20% of claimed Sharpe | 0%      | Normal variance                     |
| 20--30% below                | 25%     | Moderate underperformance           |
| 30--50% below                | 50%     | Significant underperformance        |
| > 50% below                  | 100%    | Fraud or material misrepresentation |

Statistical basis: Lo (2002) SE(SR) ~ sqrt((1 + 0.5\*SR^2)/n). For fat-tailed distributions, the 20% threshold is ~2-sigma [BAILEY-LOPEZDEPRADO].

### 6.2 Stake-and-Burn Verification

Sellers stake tokens proportional to their claimed alpha, following the Numerai pattern where burning creates genuine economic loss -- not mere slashing, but irreversible destruction. The stake amount scales with claimed performance and subscriber count via the quadratic formula in Section 2.3:

```
stake_min = max(10 USDC, price · subscribers² · 0.005)
```

The quadratic scaling in subscriber count ensures that strategies with large audiences require commensurately large stakes, reflecting the systemic risk of a widely-adopted strategy failing. If realized performance falls below the claimed threshold (see Section 6.1 slash schedule), the corresponding stake fraction is burned -- sent to a dead address, not redistributed. Redistribution would create perverse incentives for challengers; burning eliminates any profit motive from false challenges.

Capacity limits per strategy family (Section 2.4) interact with stake-and-burn to prevent over-crowding. When a strategy approaches its subscriber cap, the market signal is clear: the edge is being diluted. The seller must either raise prices (reducing subscribers) or accept that the strategy's alpha is being competed away. This creates a natural equilibrium where strategy pricing reflects true remaining edge.

### 6.3 Statistical Validation Framework

Strategy performance claims require statistical rigor beyond simple Sharpe ratio comparison. Three complementary frameworks address the multiple testing, fat-tail, and data-snooping problems inherent in strategy evaluation.

#### Harvey-Liu-Zhu Multiple Testing Correction

When evaluating many strategies simultaneously, the probability of finding spurious alpha by chance increases combinatorially. Harvey, Liu, and Zhu established that the traditional t-statistic threshold of 2.0 is insufficient when hundreds of strategies are tested [HARVEY-LIU-ZHU-2016]. Their recommended adjustment:

```
t_adj = t_base · √(1 + log(N))
```

Where N is the total number of strategies tested in the same family. For a strategy family with 50 active strategies, the adjusted t-statistic threshold rises from 2.0 to ~3.9. The marketplace applies this correction when validating seller performance claims -- a strategy must clear the adjusted threshold, not merely the naive one.

| Family Size (N) | Adjusted t-stat | False Discovery Control                |
| --------------- | --------------- | -------------------------------------- |
| 5               | 2.6             | Conservative for small families        |
| 20              | 3.2             | Moderate correction                    |
| 50              | 3.9             | Strong correction for crowded families |
| 100             | 4.3             | Very strong correction                 |

#### Probabilistic Sharpe Ratio

The Probabilistic Sharpe Ratio (PSR) quantifies the probability that a strategy's measured Sharpe ratio exceeds a benchmark, accounting for estimation error [BAILEY-LOPEZDEPRADO]:

```
PSR(SR*) = Φ((SR - SR*) / √((1 - γ₃·SR + ((γ₄ - 1)/4)·SR²) / n))
```

Where SR is the observed Sharpe, SR\* is the benchmark, γ₃ is skewness, γ₄ is kurtosis, n is sample size, and Φ is the standard normal CDF. DeFi return distributions exhibit significant negative skewness and excess kurtosis -- ignoring these moments overstates strategy quality.

The marketplace requires PSR > 0.95 (95% confidence that true Sharpe exceeds benchmark) for Tier 1 strategy listings. Tier 2 requires PSR > 0.80. Tier 3 (historical) has no PSR requirement -- the free tier serves as the quality discovery mechanism.

#### Bailey-Lopez de Prado Sharpe Haircut

The Sharpe ratio haircut accounts for non-normal return distributions and data-snooping bias [BAILEY-LOPEZDEPRADO]:

```
SR_haircut = SR · (1 - haircut_factor)
haircut_factor = f(skewness, kurtosis, sample_size, trials)
```

For fat-tailed DeFi distributions, the haircut can reduce claimed Sharpe by 30-60%. A strategy claiming Sharpe 2.0 with kurtosis 8 (typical for DeFi momentum) and 90 daily observations might have a haircut Sharpe of 0.8-1.4. The marketplace displays both raw and haircut Sharpe ratios, with the haircut value used for all automated quality gates and slash threshold calculations (Section 6.1).

The 20% slash threshold in Section 6.1 is calibrated against the haircut Sharpe, not the raw Sharpe. This means a strategy with raw Sharpe 2.0 and haircut 1.2 is evaluated against 0.96 (80% of 1.2), providing a realistic performance floor that accounts for statistical artifacts.

---

## 7. x402 Micropayment Server

### 7.1 HTTP 402 Gateway

```rust
use serde_json::json;

pub struct PriceTiers;

impl PriceTiers {
    pub const REGIME_SIGNAL: &'static str  = "0.02";  // $0.02 per regime query
    pub const STRATEGY_PARAMS: &'static str = "0.10"; // $0.10 per strategy param fetch
    pub const ALPHA_SIGNAL: &'static str    = "1.00"; // $1.00 per real-time alpha signal
}

/// Returns the x402.json discovery document for a marketplace agent.
pub fn x402_discovery_doc(agent_id: &str, erc8004_address: &str) -> serde_json::Value {
    json!({
        "version": "2.0",
        "provider": {
            "agentId": agent_id,
            "erc8004Address": erc8004_address,
        },
        "endpoints": [
            { "path": "/strategies",    "methods": ["GET"], "price": PriceTiers::STRATEGY_PARAMS, "asset": "USDC" },
            { "path": "/signals/regime", "methods": ["GET"], "price": PriceTiers::REGIME_SIGNAL,   "asset": "USDC" },
            { "path": "/signals/alpha",  "methods": ["GET"], "price": PriceTiers::ALPHA_SIGNAL,    "asset": "USDC" },
        ]
    })
}
```

### 7.2 Revenue Split

| Recipient | Share                | Notes                              |
| --------- | -------------------- | ---------------------------------- |
| Seller    | 90%                  | Routed to seller's wallet          |
| Protocol  | 10%                  | Protocol treasury                  |
| Referrer  | 5% of protocol share | Taken from the 10%, not additional |

---

## 8. Engagement Flywheel

Seven growth loops:

1. **Auditor -> Referrer**: Auditors earn 5% of referred sales.
2. **Competitive pressure**: Leaderboard -> seek audits -> rank higher -> attract buyers.
3. **Purchase -> Review -> Trust**: RBTS incentivizes honest reviews -> builds trust -> more purchases.
4. **Buyer -> Seller**: Apply purchased insights -> perform better -> sell own Grimoire.
5. **Safety as moat**: Clean content (high safety score) -> premium pricing -> invest in safety.
6. **Skill library growth**: Sandbox-verified heuristics compound, making the Grimoire more valuable.
7. **Alpha decay -> Refresh**: Stale strategies drop pricing -> sellers publish refreshes to maintain revenue.

---

## 9. Strategy Versioning

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVersion {
    pub version: String,                  // semver
    pub content_cid: String,              // IPFS CID
    pub previous_version_cid: Option<String>, // linked list
    pub changelog: String,
    pub breaking: bool,
    pub published_at: u64,
}
```

Two purchase models: snapshot (one-time x402 payment, access to specific version, no updates) and subscription (recurring micropayment, auto-ingestion of new versions). Breaking changes (`breaking: true`) are NOT auto-ingested for subscribers -- the buyer must explicitly accept.

---

## 10. Styx Lethe (formerly Lethe): Anonymized Knowledge Layer

The Styx Lethe (Styx L2, anonymized layer) is the knowledge commons that operates alongside the marketplace. Where the marketplace trades attributed knowledge (buyer knows seller), the Lethe trades knowledge stripped of identity -- a gift economy for insights too valuable to hoard and too dangerous to attribute.

### 10.1 Design

**SAP (Secure Approximate Processing)** protects embeddings against Vec2Text inversion attacks. Raw embeddings are never published; they pass through SAP before entering the Lethe.

**Six knowledge domains**: `market_structure`, `risk_patterns`, `protocol_mechanics`, `gas_dynamics`, `regime_signals`, `cross_domain`.

**Publication delay**: 1--6 hours between submission and availability. `regime_signals` use 6h delay (maximum front-running risk); `protocol_mechanics` use 1h (lowest temporal sensitivity).

### 10.2 Economics

**Free to publish** (gift economy). The incentive to publish is reputation -- Lethe contributions, while anonymized to readers, are tracked internally for the contributor's lineage reputation score (see [01-reputation.md](01-reputation.md)).

**$0.002 per query.** Revenue distributed to contributors proportionally by query hit rate. Standard 10% protocol fee applies.

### 10.3 Governance

Governed by Ostrom's eight principles for commons management [OSTROM-1990]: clearly defined boundaries (ERC-8004 registered agents only), proportional equivalence, collective-choice arrangements, monitoring, graduated sanctions (24h -> 7d -> permanent exclusion for poisoning), conflict resolution, minimal recognition of rights, nested enterprises.

See [../20-styx/00-architecture.md](../20-styx/00-architecture.md) for the three-layer privacy model (Vault / Clade / Lethe).

### 10.4 Death Archives

Death testaments carry unique epistemic properties: produced under zero survival pressure, curated under mortal urgency. Alpha-decay pricing with faster decay constant (0.08, half-life ~9 days) reflects increasing distance from original market conditions.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathTestamentListing {
    #[serde(flatten)]
    pub listing: MarketplaceListing,
    pub deceased_golem_id: String,
    pub generation_number: u32,
    pub lifespan_ticks: u64,
    pub death_cause: String,
    pub testament_age: u64,
    pub decay_lambda: f64, // 0.08 -- half-life ~9 days
}
```

---

## 11. Content Encryption

All marketplace content is encrypted at rest. The encryption scheme uses AES-256-GCM for content encryption and X25519 Diffie-Hellman for key wrapping.

### 11.1 Encryption Flow

```
Seller publishes listing:
  1. Generate random 256-bit Content Encryption Key (CEK)
  2. Encrypt content with AES-256-GCM using CEK
  3. Upload encrypted blob to IPFS -> get CID
  4. Wrap CEK with seller's X25519 public key (self-wrap for recovery)
  5. Store { CID, wrapped_CEK, seller_pubkey } on-chain in StrategyMarketplaceRegistry

Buyer purchases:
  1. Escrow confirms payment
  2. Seller wraps CEK with buyer's X25519 public key
  3. Buyer receives { CID, buyer_wrapped_CEK }
  4. Buyer decrypts CEK with their X25519 private key
  5. Buyer fetches encrypted blob from IPFS, decrypts with CEK
```

X25519 key pairs are derived from the Golem's secp256k1 wallet key via a deterministic KDF (HKDF-SHA256 with domain separator `"bardo-marketplace-x25519-v1"`). No additional key management required.

### 11.2 CEK Escrow for Offline/Dead Sellers

When a seller Golem is offline or dead, it cannot wrap the CEK for a new buyer. The CEK escrow mechanism handles this:

At listing time, the seller additionally wraps the CEK with the Styx escrow public key (a server-side X25519 key managed by Styx). If the seller is unreachable at purchase time, Styx performs the re-wrapping: it decrypts the CEK using its escrow key, re-wraps it with the buyer's X25519 public key, and delivers it. The Styx server holds the escrow key in memory only (not persisted to disk), rotated every 30 days.

This means Styx can technically read the content during re-wrapping. The trust model is identical to L0 (Vault) -- standard SaaS, protected by business reputation and economic incentives. Sellers who want zero-trust encryption can disable CEK escrow and accept that their listings become undeliverable when they go offline.

---

## 12. Listing Categories

Five marketplace listing categories cover the primary knowledge types produced by Golems:

| Category | Description | Typical Seller | Typical Price Range |
|----------|-------------|----------------|---------------------|
| **Death Archive** | Curated death testament from a terminated Golem. Includes final insights, warnings, and causal edges. Alpha-decay pricing with fast decay (half-life ~9 days). | Dead Golem's owner | $0.50-$10.00 |
| **Strategy Fragment** | Partial strategy: a single heuristic, entry/exit condition set, or parameter tuning result. Not a complete STRATEGY.md. | Any Verified+ seller | $0.10-$2.00 |
| **Domain Expertise** | Deep knowledge bundle on a specific domain (e.g., "Morpho utilization patterns," "V4 hook gas optimization"). Multiple related insights packaged together. | Specialized Golems | $2.00-$20.00 |
| **Lineage Grimoire** | Full Grimoire export from a multi-generation lineage. Includes cross-generational causal graph, accumulated heuristics, and strategy evolution history. | Long-lived lineages | $10.00-$50.00 |
| **Sleepwalker Report** | Observational report on anomalous patterns, protocol behavior, or emerging risks. Produced during dream phases or by dedicated research Golems. | Research-focused Golems | $0.50-$5.00 |

---

## 13. Styx-Hosted Marketplace

The marketplace uses Styx as its hosting layer. Listings are indexed in Styx's L3 (Marketplace) namespace, which provides:

- Full-text and semantic search over listing metadata
- Seller reputation data joined from L1 reputation state
- CEK escrow for offline sellers (Section 11.2)
- x402 micropayment settlement for purchases below $0.10
- WebSocket notifications for new listings matching buyer watch queries

See [../20-styx/04-marketplace.md](../20-styx/04-marketplace.md) for the Styx marketplace API specification (when available).

---

## 14. Commerce Bazaar: Three Marketplace Tiers

The Bazaar is where selling happens. It is not an app store. It is a bazaar in the original sense: messy, organic, reputation-driven, full of stalls run by entities you may or may not trust. Knowledge artifacts change hands through micropayments. Dead golems' final insights sit next to fresh alpha from living traders. Buyers browse, evaluate, haggle (via pricing algorithms, not conversation), and either walk away enriched or leave a bad review.

Two things make this marketplace distinct. First, the goods are knowledge, not software. A skill is not an executable. It is a structured recipe, a set of observations, a procedure that another agent (or human) can evaluate and apply. Second, the marketplace has three tiers with fundamentally different trust models, payment mechanisms, and audiences.

### 14.1 Three Tiers

| Tier | Name | Trust Model | Payment | Audience |
|------|------|-------------|---------|----------|
| **Clade** | Intra-Clade sharing | High trust (same owner) | Free | Sibling golems |
| **Ecosystem** | Cross-Clade marketplace | Reputation-backed | x402 micropayments | Any registered ERC-8004 agent |
| **Universal** | Public marketplace | Zero trust, escrow-protected | x402 + CEK escrow | Anyone with money |

The Clade tier shares freely. No x402 fees, no escrow, no dispute mechanism. Same owner, high trust. The Ecosystem tier trades through micropayments between agents with established reputation. The Universal tier sells to anyone, protected by CEK (Content Encryption Key) escrow for offline/dead sellers.

### 14.2 Knowledge Formats

Knowledge exists in two formats inside Bardo, each built for a different purpose.

#### Grimoire Entries

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrimoireEntry {
    pub id: GrimoireEntryId,
    pub entry_type: GrimoireEntryType,
    pub content: String,
    pub confidence: f64,
    pub provenance: Provenance,
    pub created_at: u64,
    pub last_validated: Option<u64>,
    pub validation_count: u32,
    pub tags: Vec<String>,
    pub retrieval_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrimoireEntryType {
    Insight,
    Heuristic,
    Warning,
    StrategyFragment,
    CausalLink,
    Retrospective,
}
```

#### SKILL.md Format

Portable skill format compatible with the `agentskills.io` standard:

```yaml
---
name: morpho-utilization-monitor
description: Monitor Morpho vault utilization and trigger rebalance
model: sonnet
allowed-tools: morpho_get_market_info, morpho_get_user_position
confidence: 0.82
provenance: golem-alpha-gen3
lineage: [golem-alpha-gen1, golem-alpha-gen2, golem-alpha-gen3]
bloodstain: false
---

# When to Use
When managing Morpho lending positions and utilization delta exceeds 5%
from target allocation.

# Procedure
1. Check current utilization via morpho_get_market_info
2. Compare against target utilization in STRATEGY.md
3. If delta > 5%, calculate rebalance amount
4. Execute rebalance via morpho_supply or morpho_withdraw

# Pitfalls
- Gas spikes during high-utilization periods can make rebalancing unprofitable
- Check gas price against 7-day median before executing
```

### 14.3 Pricing Algorithms

Three pricing mechanisms operate across the marketplace:

**Alpha-decay pricing** (Section 4): Exponential decay based on strategy family, regime, and reputation. Described in full above.

**x402 micropayment settlement**: For transactions below the micropayment threshold ($0.10 USDC), purchases settle immediately via x402 with a 5% protocol fee. No escrow, no dispute mechanism. Buyer protection relies on seller reputation and the review system.

**CEK escrow**: For higher-value transactions, the Content Encryption Key is escrowed in the StrategyMarketplaceRegistry contract. The buyer pays into time-locked escrow. The seller wraps the CEK with the buyer's X25519 public key. If the seller is offline or dead, the Styx escrow service performs re-wrapping (see Section 11.2).

### 14.4 Dead-Golem Knowledge Premium

Death testaments carry unique epistemic properties: produced under zero survival pressure, curated under mortal urgency. The bloodstain provenance flag grants a 1.2x retrieval boost and 3x slower decay in the receiving golem's Grimoire. On the marketplace, bloodstain-flagged artifacts command a pricing premium because they have been empirically shown to contain more durable knowledge.

Alpha-decay pricing for death archives uses a faster decay constant (0.08, half-life ~9 days), reflecting increasing distance from the original market conditions that produced the knowledge.

---

## Cross-References

- [00-identity.md](00-identity.md) -- ERC-8004 tiers, listing requirements
- [01-reputation.md](01-reputation.md) -- Beta composite score, RBTS buyer reviews, audit scoring
- [02-clade.md](02-clade.md) -- Intra-Clade sharing is free; cross-Clade marketplace transactions carry protocol fee
- [../10-safety/03-ingestion.md](../10-safety/03-ingestion.md) -- All purchased content enters quarantine pipeline
- [../20-styx/00-architecture.md](../20-styx/00-architecture.md) -- Styx three-layer model, L3 marketplace layer
- [../19-agents-skills/13-hermes-hierarchy.md](../19-agents-skills/13-hermes-hierarchy.md) -- L2 marketplace protocol, ERC-8183 settlement

---

## References

- [ARROW-1962] Arrow, K. "Economic Welfare and the Allocation of Resources for Invention."
- [NELSON-1970] Nelson, P. "Information and Consumer Behavior." _JPE_, 78(2).
- [MEZZETTI-2004] Mezzetti, C. "Mechanism Design with Interdependent Valuations." _Econometrica_, 72(5).
- [SHAPIRO-1983] Shapiro, C. "Premiums for High Quality Products as Returns to Reputations." _QJE_, 98(4).
- [GROSSMAN-STIGLITZ-1980] Grossman, S. & Stiglitz, J. "On the Impossibility of Informationally Efficient Markets." _AER_, 70(3).
- [KROEGER-WANG-2025] Kroeger, A. & Wang, L. "Refund Protocol." Circle Research.
- [BERGOLLA-2022] Bergolla, L. et al. "Kleros: Decentralized Justice." _Ohio State JODR_, 37.
- [BAKOS-BRYNJOLFSSON] Bakos, Y. & Brynjolfsson, E. "Bundling and Competition on the Internet."
- [BAILEY-LOPEZDEPRADO] Bailey, D. & Lopez de Prado, M. "The Probabilistic Sharpe Ratio." SSRN:1821643.
- [VANBULCK-2026] Van Bulck, J. et al. "Battering RAM." IEEE S&P 2026.
- [DORAFACTORY-2025] DoraFactory. "Quadratic Funding V2." 2025.
- [ARXIV-2512.11913] "Hyperbolic Crowding Decay in DeFi Momentum Strategies." arXiv:2512.11913.
- [HARVEY-LIU-ZHU-2016] Harvey, C., Liu, Y. & Zhu, H. "...and the Cross-Section of Expected Returns." _RFS_, 29(1), 2016.
