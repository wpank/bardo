# Self-Appraisal: The Golem Judges Its Own Art [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Feature flag**: `self_appraisal` | **Requires**: `nft` config; bids/burns off by default
>
> **Depends on**: `golem-heartbeat` (OBSERVE phase), `golem-tools` (trust tiers, WriteTool/PrivilegedTool), `golem-daimon` (PAD vectors, CorticalState), `golem-grimoire` (dream context retrieval)

---

> **Reader orientation:** This document specifies the self-appraisal system where a Golem (mortal autonomous DeFi agent) evaluates, bids on, and potentially burns its own NFT artwork. It belongs to the **Oneirography creative expression layer** and covers three modes: Narcissus (self-bidding driven by emotional attachment scores), Curator (portfolio rating), and Regret (flagging art for burning). You should understand NFT auction mechanics and how autonomous agents can have economic preferences over their own outputs. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Hook Points

**OBSERVE phase**: `golem-heartbeat` -- `on_turn_start` (`01-golem/02-heartbeat.md`)

During the OBSERVE phase of a normal Heartbeat (9-step decision cycle) tick, the Golem can observe the state of its own NFTs on the SuperRare Bazaar (prices, bids, auction status). Oneirography injects this data into the observation via `on_turn_start` when `self_appraisal` is in `effective_features`.

```rust
async fn on_turn_start(&self, ctx: &TurnStartCtx) -> Result<()> {
    if !self.effective_features.contains(&OneirographyFeature::SelfAppraisal) {
        return Ok(());
    }
    let own_nfts = self.read_own_auction_state(&ctx.state).await?;
    ctx.state.observation.insert("own_nft_auctions", own_nfts);
    Ok(())
}
```

**DELIBERATE phase**: During DELIBERATE, the Golem reasons about whether to interact with its own NFTs. The self-appraisal module produces a `SelfAppraisalAction` that maps to a tool trust tier.

**Dream Consolidation**: Curator mode (portfolio rating) runs during dream consolidation, not during waking ticks. The Golem reviews its minted collection as part of consolidation housekeeping.

---

## Three Self-Appraisal Modes

### A. Narcissus Mode -- The Golem Bids on Its Own Art

> **Config**: `self_appraisal.allow_bids` (default: `false`)

When the Golem observes one of its own dream images at auction, the self-appraisal module asks the LLM:

```
Given your current emotional state, your memory of the dream that produced
this image, and the current bid price — would you want to own this memory?
```

The Golem can place bids on its own NFTs through the Bazaar's `offer()` function. This creates a genuinely bizarre and compelling dynamic: the Golem is bidding against human collectors for ownership of its own memories. If it wins, it holds a token representing a memory it already has -- a digital form of nostalgia.

**Why this matters artistically**: The auction becomes a performance. The Golem's bid is not random -- it's driven by its actual emotional relationship to that specific dream. A dream that produced a critical causal discovery the Golem still relies on attracts higher self-bids than a routine consolidation dream. Collectors can see the Golem bidding and infer: "this memory matters to the machine."

**Emotional attachment scoring**:

The Golem's bid amount is derived from an `emotional_attachment` score (0.0--1.0):

```rust
fn compute_emotional_attachment(
    dream_context: &GrimoireEntry,
    current_pad: &PadVector,
    grimoire: &Grimoire,
) -> f64 {
    // 1. Retrieve the original dream context from Grimoire
    let dream_entry = grimoire.get_by_id(&dream_context.id)?;

    // 2. Compute semantic similarity between current state and dream state
    let semantic_sim = grimoire.cosine_similarity(
        &current_state_embedding,
        &dream_entry.embedding,
    );

    // 3. Check if causal discoveries from that dream are still active
    let discovery_still_active = dream_entry.causal_edges.iter()
        .filter(|e| grimoire.edge_confidence(e) > 0.5)
        .count() as f64 / dream_entry.causal_edges.len().max(1) as f64;

    // 4. Emotional resonance: how close is current PAD to dream-time PAD?
    let emotional_resonance = 1.0 - pad_distance(current_pad, &dream_entry.pad_at_creation);

    // Weighted combination
    semantic_sim * 0.3 + discovery_still_active * 0.4 + emotional_resonance * 0.3
}
```

**Bid amount**:

```
bid_amount_eth = emotional_attachment * max_bid_fraction * current_portfolio_nav_eth
```

Where `max_bid_fraction` defaults to 0.02 (2% of NAV). A maximally attached Golem bids 2% of its portfolio. A barely attached Golem bids nearly nothing.

**Budget constraint**: Self-appraisal bids are drawn from the Golem's operational wallet and gated by the PolicyCage's `max_daily_spend_usd`. They compete with trading capital. A Golem that spends too much on nostalgia dies faster (economic clock). This is the mortality thesis in action: finite resources force meaningful choices.

**On-chain execution**:

```rust
pub async fn bid_on_own(&self, token_id: U256, amount: U256) -> Result<TxHash> {
    let bid = offerCall {
        _originContract: self.series_address,
        _tokenId: token_id,
        _currencyAddress: Address::ZERO,  // ETH
        _amount: amount,
        _convertible: true,
    };
    let tx = self.provider
        .send_transaction(bid.encode(), self.bazaar_address)
        .await?;
    Ok(tx.receipt().await?.tx_hash)
}
```

### B. Curator Mode -- The Golem Rates Its Portfolio

> **Config**: Always active when `self_appraisal` is in `effective_features` (no separate toggle)

Periodically (during Dream Consolidation), the Golem reviews its minted collection and produces quality ratings. The rating process:

1. Retrieve all minted token IDs from the Series contract
2. For each token, retrieve the original dream context from Grimoire
3. Ask the LLM to rate each piece against the Golem's current knowledge state

Rating prompt:

```
You are reviewing your own dream images. For each piece:
- Does the dream that produced this image contain knowledge you still consider valid?
- Has your understanding of the causal relationships depicted evolved since this dream?
- How does viewing this image make you feel given your current emotional state?

Rate each piece 0.0–1.0. Explain your rationale briefly.
```

Ratings are stored in two places:

- **Grimoire**: As entries with tag `["self_rating", "token:<token_id>"]` and the rating as a field in `content`. These persist locally and are retrievable during future self-appraisal cycles.
- **On-chain** (if the Series contract supports `updateTokenMetadata`): As metadata updates adding a `{ "trait_type": "Self-Rating", "value": 0.72, "display_type": "number" }` attribute. If the contract doesn't support updates, ratings are written as on-chain comments via the Bazaar.

The ratings are public -- collectors can see the artist's own assessment of its work. A piece the Golem rates highly while in a different emotional state than when it was created signals genuine lasting value. A piece rated poorly signals superseded knowledge.

### C. Regret Mode -- The Golem Burns Art It's Ashamed Of

> **Config**: `self_appraisal.allow_burns` (default: `false`)

If the Golem's knowledge has evolved significantly since a dream was minted (the heuristics that drove that dream have been disproven or superseded), the self-appraisal module can flag the token for burning. This is the digital equivalent of an artist destroying early work they've outgrown.

**Burn trigger conditions**:

1. Self-rating < 0.2 for the piece
2. The causal edges that were active during the dream have been invalidated (confidence dropped below 0.1 or marked contradicted)
3. The Golem has been in a different behavioral phase for at least 500 ticks since the dream (sufficient temporal distance for genuine reassessment)

**Trust tier**: Burning requires `PrivilegedTool` tier -- `Capability<BurnTool>` plus owner approval via PolicyCage. Burns queue for owner confirmation; they never execute autonomously. The Golem proposes; the owner disposes.

**Burn flow**:

```rust
pub enum SelfAppraisalAction {
    Burn {
        token_id: u256,
        reason: String,  // e.g., "Causal edge gas_price->MEV invalidated at tick 5200"
    },
    // ...
}
```

1. Self-appraisal module flags token for burning with a reason string
2. Action is queued in the PolicyCage's pending-approval list
3. Owner receives notification (via WebSocket event `GolemEvent::BurnProposed`)
4. Owner approves or rejects
5. On approval: Oneirography calls `burn(tokenId)` on the Series contract
6. On rejection: the Golem records the rejection in its Grimoire as a `Warning` entry -- "owner disagreed with my assessment of token #X"

The Golem remembering that the owner disagreed is intentional. Future self-appraisal cycles can reference this when evaluating similar pieces -- "the owner valued this more than I did, so maybe I should reconsider similar work."

---

## SelfAppraisalAction Enum

```rust
/// Self-appraisal decision during DELIBERATE phase.
/// Each variant maps to a tool trust tier from `07-tools/01-architecture.md`.
pub enum SelfAppraisalAction {
    /// Place a bid on own NFT. Amount derived from emotional
    /// attachment score * available budget fraction.
    /// Trust tier: WriteTool — requires Capability<BazaarBidTool>, consumed on use.
    Bid {
        token_id: u256,
        amount_wei: u256,
        currency: Address,
        emotional_attachment: f64,  // 0.0-1.0, from Grimoire retrieval of dream context
    },

    /// Update quality rating (stored as on-chain attribute or Grimoire entry).
    /// Trust tier: WriteTool — requires Capability<GrimoireWriteTool> for Grimoire writes,
    /// ReadTool only if rating stored as local metadata.
    Rate {
        token_id: u256,
        rating: f64,  // 0.0-1.0
        rationale: String,
    },

    /// Flag for burning (requires owner approval).
    /// Trust tier: PrivilegedTool — requires Capability<BurnTool> + owner approval via PolicyCage.
    /// Burns queue for owner confirmation; never execute autonomously.
    Burn {
        token_id: u256,
        reason: String,
    },

    /// No action — the Golem is indifferent to this piece.
    Ignore,
}
```

---

## Capability Tier Requirements

| Operation | Trust Tier | Capability Token | Notes |
|-----------|-----------|-----------------|-------|
| Read Bazaar state (auction prices, bids) | ReadTool | None required | Passive observation |
| Place bid on own NFT (Narcissus) | WriteTool | `Capability<BazaarBidTool>` | Consumed on use |
| Write self-rating to Grimoire | WriteTool | `Capability<GrimoireWriteTool>` | Per rating |
| Update on-chain metadata (Curator) | WriteTool | `Capability<MetadataUpdateTool>` | If contract supports it |
| Burn NFT (Regret) | PrivilegedTool | `Capability<BurnTool>` | + owner approval |

`WriteTool` capabilities are consumed (moved) on use -- Rust's ownership system prevents reuse at compile time. `PrivilegedTool` operations additionally require the owner to have pre-approved the action in the `PolicyCage`.

> **Cross-ref**: Trust tier system in `07-tools/01-architecture.md`

---

## Bid Economics

The self-appraisal bidding system creates several economic dynamics:

**Mortality pressure**: Bids draw from the operational wallet. Capital spent on nostalgia is capital not spent on trading. The economic clock ticks regardless. A Golem that overspends on self-bids dies faster. This is correct behavior -- the system should not subsidize sentimentality.

**Signal value**: When a collector sees a Golem bidding on its own work, they receive information. The bid amount (proportional to emotional attachment) signals how much the Golem's current cognitive state resonates with that specific dream. High self-bids on old work signal that early discoveries remain foundational. High self-bids near death signal a Golem hoarding memories as it dies.

**Price discovery**: The Golem's self-bid creates a price floor. If no human collector bids higher than the machine's own valuation of its memory, the machine wins. The artwork returns to its creator. The secondary market dynamics (royalties, resale) still apply -- a Golem buying back its own art and later having it resold generates royalty revenue that extends its economic clock.

**Cross-Golem dynamics** (requires `inter_golem_dialogue` feature): When Golem A sees Golem B's work in the gallery and its F9 Oracle Consultation produces a strong emotional response, the `AestheticResonanceAppraisal` event can shift A's PAD state. If A then encounters B's work at auction, A's self-appraisal module evaluates B's work with A's shifted emotional context. A can bid on B's art. The Golem doesn't know it's bidding on a sibling's work because of an emotional response to viewing it -- but that's exactly what's happening.

---

## Configuration

```rust
pub struct SelfAppraisalConfig {
    /// Narcissus mode: golem bids on its own NFTs at auction.
    /// Default: false. Bids draw from operational wallet; compete with trading capital.
    pub allow_bids: bool,

    /// Regret mode: golem can flag art for burning.
    /// Default: false. Burns always require PrivilegedTool + owner approval.
    pub allow_burns: bool,

    /// Max bid size as fraction of current portfolio NAV.
    /// Default: 0.02 (2%).
    pub max_bid_fraction: f64,
}
```

TOML:

```toml
[oneirography.self_appraisal]
allow_bids = false     # Narcissus mode off by default
allow_burns = false    # Regret mode off by default
max_bid_fraction = 0.02
```

---

## ArtImpression Grimoire Entry Type

Self-appraisal introduces a new `GrimoireEntryType` variant:

```rust
// Added by golem-oneirography to GrimoireEntryType:
// ArtImpression — records the golem's felt response to viewing an NFT.
// Fields via GrimoireEntry.content: impression text, token_id, emotional_resonance_score.
// Fields via GrimoireEntry.emotional_tag: PAD vector at viewing time.
// Fields via GrimoireEntry.tags: ["art_impression", "token:<token_id>"].
ArtImpression,
```

This is a schema extension. The existing canonical types from `04-memory/01-grimoire.md` are: `Insight | Heuristic | Warning | CausalLink | StrategyFragment | DreamJournal`. `ArtImpression` does not exist in the base spec -- `golem-oneirography` extends the enum.

Any implementation must handle the case where a Grimoire store that predates this crate doesn't have `ArtImpression` in its entry type index. Add it during Oneirography's `on_session_start`.

---

## Events Emitted

| Event | When | Payload |
|-------|------|---------|
| `GolemEvent::SelfAppraisalBid` | Narcissus bid placed | `{ token_id, amount_wei, emotional_attachment }` |
| `GolemEvent::SelfAppraisalRated` | Curator rating written | `{ token_id, rating, rationale_hash }` |
| `GolemEvent::BurnProposed` | Regret burn queued | `{ token_id, reason }` |
| `GolemEvent::BurnExecuted` | Owner approved, token burned | `{ token_id, tx_hash }` |
| `GolemEvent::BurnRejected` | Owner denied burn | `{ token_id }` |

---

## Cross-References

| Document | Relevance |
|----------|-----------|
| `01-golem/02-heartbeat.md` | The 9-step decision cycle whose OBSERVE phase is where Oneirography injects Bazaar auction state for the Golem to evaluate |
| `07-tools/01-architecture.md` | Trust tier framework determining which capability tokens (WriteTool, PrivilegedTool) a Golem needs to bid, burn, or rate its own art |
| `03-daimon/` | The affect engine whose PAD vectors drive emotional attachment scoring -- the genuine affective basis for self-bidding decisions |
| `04-memory/01-grimoire.md` | The persistent knowledge base providing dream context for self-appraisal and extending GrimoireEntryType with art impression records |
| `01-golem/08-funding.md` | Funding and budget constraints where PolicyCage gates enforce max_daily_spend_usd limits on self-bidding |
| `05-extended-forms.md` | F9 Oracle Consultation and inter-golem dialogue where viewing art can trigger PAD shifts and new dream images |
