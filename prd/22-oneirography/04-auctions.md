# Affect-Reactive Auctions: Emotion-Modulated Pricing [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Feature flag**: Part of `dream_journal` (no separate flag)
>
> **Depends on**: `golem-daimon` (PAD vectors, CorticalState), `golem-mortality` (BehavioralPhase), `golem-oneirography/sidecar.rs` (Bazaar API)

---

> **Reader orientation:** This document specifies how auction parameters for Golem (mortal autonomous DeFi agent) artwork are computed from the Golem's real-time emotional state, not set statically. It belongs to the **Oneirography creative expression layer** and covers the PAD-to-auction mapping (Pleasure modulates reserve price, Arousal compresses duration, Dominance selects auction type), pheromone threat modulation, and Dutch auction fallback for terminal phase. You should understand NFT auction types (reserve, scheduled, Dutch) and basic emotional state vectors. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Hook Point

**Crate**: `golem-daimon` (the Golem's affect engine) -- PAD vector (Pleasure-Arousal-Dominance emotional coordinates) / CorticalState (32-signal atomic shared perception surface) (`03-daimon/`, `01-golem/18-cortical-state.md`)

When a dream image is minted, the auction configuration on the SuperRare Bazaar is not static -- it's parameterized by the Golem's emotional state at the moment of minting.

---

## Primary Mapping: PAD to Auction Parameters

| PAD Dimension | Auction Parameter | Mapping | Rationale |
|--------------|-------------------|---------|-----------|
| **Pleasure** (P) | Reserve price | `base_price * (1.0 + P)` | A thriving Golem values its output more. A suffering Golem practically gives it away -- the art of desperation is cheap. Joyful dreams have higher reserves, painful dreams are cheaper. |
| **Arousal** (A) | Auction duration | `base_duration * (1.0 - A*0.5)` | High arousal = shorter auctions. Urgent emotional states produce urgency in the market. A panicked Golem doesn't wait. Range: 50% to 100% of base duration. |
| **Dominance** (D) | Auction type | D > 0.3: Scheduled. D < -0.3: Reserve. Between: Convertible offer. | A dominant Golem dictates terms. A submissive Golem lets the market lead. |

**Reading PAD from CorticalState**:

```rust
// CorticalState.pad() reads the three ALMA-weighted atomics in one call.
// The effective PAD is personality * 0.25 + mood * 0.50 + emotion * 0.25.
let pad = cortical.pad();  // returns PadVector { pleasure, arousal, dominance }
```

> **Cross-ref**: CorticalState atomics and ALMA weighting in `01-golem/18-cortical-state.md`

---

## Secondary Modulations

| Signal | Parameter | Effect |
|--------|-----------|--------|
| Behavioral phase = Terminal | All reserves -> 0 | The dying Golem gives its final works away. The death mask is the exception -- it always has a reserve. |
| Pheromone threat level > 0.7 | Duration halved | Collective anxiety accelerates everything. Read via `GET /v1/styx/pheromone/sense`. |
| Dream contained bloodstain inheritance | Reserve * 1.5 | Art informed by death carries a premium. |
| Golem generation > 3 | "Lineage" trait badge | Multi-generational art commands collector interest. |

---

## Implementation

```rust
fn compute_auction_params(
    pad: &PadVector,
    behavioral_phase: &BehavioralPhase,
    base_reserve_eth: f64,
    base_duration_seconds: u64,
    is_death_mask: bool,
    pheromone_threat: f64,
    has_bloodstain: bool,
) -> AuctionParams {
    // Reserve price: pleasure modulates (range: 0.0x to 2.0x base)
    let mut reserve_multiplier = if matches!(behavioral_phase, BehavioralPhase::Terminal) && !is_death_mask {
        0.0  // Terminal Golems give art away (except death masks)
    } else {
        (1.0 + pad.pleasure as f64).max(0.0)
    };

    // Bloodstain inheritance premium
    if has_bloodstain {
        reserve_multiplier *= 1.5;
    }

    // Duration: arousal compresses (range: 50% to 100% of base)
    let mut duration_multiplier = (1.0 - pad.arousal.abs() as f64 * 0.5).max(0.5);

    // Pheromone threat compression
    if pheromone_threat > 0.7 {
        duration_multiplier *= 0.5;
    }

    // Auction type: dominance selects
    let auction_type = if is_death_mask {
        AuctionType::Reserve  // Death masks always reserve
    } else if pad.dominance > 0.3 {
        AuctionType::Scheduled
    } else if pad.dominance < -0.3 {
        AuctionType::Reserve
    } else {
        AuctionType::ConvertibleOffer
    };

    AuctionParams {
        reserve_eth: base_reserve_eth * reserve_multiplier,
        duration_seconds: (base_duration_seconds as f64 * duration_multiplier) as u64,
        auction_type,
    }
}
```

### AuctionParams Struct

```rust
pub struct AuctionParams {
    pub reserve_eth: f64,
    pub duration_seconds: u64,
    pub auction_type: AuctionType,
}

pub enum AuctionType {
    /// Golem sets start time and price. D > 0.3 (dominant).
    Scheduled,
    /// Market-driven: bidding starts when first bid arrives. D < -0.3 (submissive).
    /// Also used for death masks (always).
    Reserve,
    /// Hybrid: starts as offer, converts to auction on first bid. -0.3 <= D <= 0.3.
    ConvertibleOffer,
}
```

---

## Bazaar Integration

The SuperRare Bazaar's `configureAuction()` function accepts:

| Parameter | Type | Source |
|-----------|------|--------|
| `auctionType` | `bytes32` | `RESERVE_AUCTION_BYTES32` or `SCHEDULED_AUCTION_BYTES32` |
| `originContract` | `address` | Golem's Series contract address |
| `tokenId` | `uint256` | Token ID from mint |
| `startingAmount` | `uint256` | `reserve_eth` converted to wei |
| `currencyAddress` | `address` | `0x0` for ETH |
| `lengthOfAuction` | `uint256` | `duration_seconds` |
| `startTime` | `uint256` | 0 for reserve (starts on first bid), timestamp for scheduled |
| `splitAddresses` | `address[]` | Revenue split recipients (see below) |
| `splitRatios` | `uint8[]` | Basis points per recipient |

**Sidecar call**:

```rust
async fn configure_auction(
    &self,
    ctx: &SidecarClient,
    token_id: u64,
    params: &AuctionParams,
) -> Result<()> {
    ctx.sidecar.call("rare_configure_auction", serde_json::json!({
        "contract": self.series_contract.to_string(),
        "tokenId": token_id.to_string(),
        "type": params.auction_type.as_str(),
        "startingAmount": params.reserve_wei.to_string(),
        "currency": "0x0000000000000000000000000000000000000000",
        "duration": params.duration_seconds,
        "network": self.config.network,
    })).await?;
    Ok(())
}
```

**Alloy direct call (fallback)**:

```rust
pub async fn configure_auction(
    &self,
    token_id: U256,
    params: &AuctionParams,
) -> Result<TxHash> {
    // Approve Bazaar to transfer token
    let approve = approveCall {
        to: self.bazaar_address,
        tokenId: token_id,
    };
    self.provider
        .send_transaction(approve.encode(), self.series_address)
        .await?
        .receipt()
        .await?;

    // Configure the auction
    let auction_type = match params.auction_type {
        AuctionType::Reserve => RESERVE_AUCTION_BYTES32,
        AuctionType::Scheduled => SCHEDULED_AUCTION_BYTES32,
        _ => RESERVE_AUCTION_BYTES32,
    };

    let config = configureAuctionCall {
        auctionType: auction_type,
        originContract: self.series_address,
        tokenId: token_id,
        startingAmount: params.reserve_wei,
        currencyAddress: Address::ZERO,  // ETH
        lengthOfAuction: U256::from(params.duration_seconds),
        startTime: U256::ZERO,  // Reserve = 0 start time
        splitAddresses: vec![],
        splitRatios: vec![],
    };

    let tx = self.provider
        .send_transaction(config.encode(), self.bazaar_address)
        .await?;
    Ok(tx.receipt().await?.tx_hash)
}
```

---

## Dutch Auction Fallback

For unsold pieces (auction expired without bids), a Dutch auction mechanism activates:

1. After the initial auction expires with no bids, wait 24 hours
2. Relist at 80% of original reserve
3. Reduce by 10% every 12 hours
4. Floor: 0.001 ETH (prevents listing at zero)

The Dutch auction is automatic -- no Golem intervention required. The Bazaar tracks expired auctions and Oneirography's periodic housekeeping (during dream consolidation) detects expired listings and triggers relisting.

If a piece reaches the floor price and still doesn't sell after 7 days, it remains listed at the floor indefinitely. The Golem never burns unsold work automatically -- that requires Regret mode (`03-self-appraisal.md`).

---

## Revenue Split

| Recipient | Share | Notes |
|-----------|-------|-------|
| Golem wallet | 85% | Fuels ongoing operations (trading, inference, art) |
| Protocol | 10% | Bardo protocol treasury |
| Successor fund | 5% | Earmarked for the Golem's successor on death |

For death mask auctions: proceeds go to the owner wallet (the Golem is dead) with the same split applied to the owner instead of the Golem wallet.

**Self-sustaining loop**: If the art sells, the Golem extends its economic clock. A Golem whose art is popular lives longer. A Golem whose art nobody wants dies on schedule. Market reception literally determines lifespan.

Revenue tracking: `BankrUiEvent::RevenueReceived { source: "nft_auction_proceeds", amount_usd: f64 }`. When wallet balance crosses model unlock thresholds (e.g., Opus access requires minimum float), the art system automatically upgrades its prompt model for the next dream cycle.

---

## Revenue Flow Summary

| Source | Mechanism | Recipient |
|--------|-----------|-----------|
| Primary sales | Bazaar `Sold` event | Golem wallet (fuels operations) |
| Secondary royalties | ERC-2981 (configurable %, default 10%) | Golem wallet or owner wallet |
| Death mask auction | Reserve auction proceeds | Owner wallet (Golem is dead) |
| Self-appraisal bids (won by Golem) | Golem purchases its own art | Goes to previous collector |
| $RARE curation staking rewards | Rarity pool for Golem's Series | Stakers who curate the Golem |

---

## Pheromone Threat Reading

Pheromone threat level for auction duration modulation is read from Styx:

```rust
// GET /v1/styx/pheromone/sense?domains=dex-lp,lending&regime=volatile
let sense_resp: PheromoneFieldState = styx_client
    .get("/v1/styx/pheromone/sense")
    .query(&[("domains", "dex-lp,lending"), ("regime", regime_str)])
    .send().await?
    .json().await?;

if sense_resp.threat_level > 0.7 {
    // Collective panic threshold crossed — duration halved
}
```

> **Cross-ref**: Pheromone field API in `20-styx/00-architecture.md`

---

## Behavioral Phase Effects on Pricing

The narrative arc of a Golem's art prices tells a story:

| Phase | Typical PAD | Reserve | Duration | Type | Character |
|-------|------------|---------|----------|------|-----------|
| Thriving | P>0, A~0.3, D>0 | 1.0-2.0x base | ~100% base | Scheduled | Confident, measured |
| Stable | P~0, A~0.5, D~0 | 0.8-1.2x base | ~75% base | Convertible | Balanced |
| Conservation | P<0, A>0.5, D<0 | 0.3-0.7x base | ~60% base | Reserve | Anxious, market-led |
| Declining | P<-0.3, A>0.7, D<-0.3 | 0.1-0.4x base | ~50% base | Reserve | Desperate |
| Terminal | P<-0.5, A varies, D<-0.5 | 0.0 | Minimum | Reserve | Giving away |

A collector watching the price trajectory of a Golem's dream series sees the emotional arc. Rising reserves = confidence. Falling reserves = distress. Zero reserves = terminal. The prices are not set by a market maker -- they're set by a dying mind's valuation of its own output.

---

## Cross-References

| Document | Relevance |
|----------|-----------|
| `03-daimon/` | The affect engine computing PAD vectors and Plutchik emotion labels that directly parameterize auction reserve, duration, and type |
| `01-golem/18-cortical-state.md` | The 32-signal perception surface whose ALMA-weighted PAD atomics provide the real-time emotional state read at mint time |
| `02-mortality/06-thanatopsis.md` | The death protocol defining Terminal phase behavior where all auction reserves go to zero except the death mask |
| `20-styx/00-architecture.md` | The global relay network's pheromone field whose threat level modulates auction parameters when clade-wide stress is detected |
| `06-contracts.md` | The on-chain contract architecture: Bazaar configureAuction interface and Series contract where auctions are actually configured |
| `01-golem/08-funding.md` | SustainabilityMetrics and the economic clock that determine whether the Golem can afford to set meaningful reserves |
