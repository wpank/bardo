# Oneirography: Overview [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Crate**: `golem-oneirography` | **Networks**: Base (x402 payments, image gen) + Ethereum Mainnet / Base Sepolia (SuperRare Bazaar + Series contracts)
>
> **Depends on**: `golem-dreams` (dream cycle), `golem-mortality` (Thanatopsis), `golem-daimon` (PAD vectors), `golem-grimoire` (memory retrieval), `golem-heartbeat` (OBSERVE phase), `golem-tools` (trust tiers), `golem-custody` (wallet management)

---

> **Reader orientation:** This document is the entry point for Oneirography, the subsystem that transforms a Golem's (mortal autonomous DeFi agent) internal cognitive life -- dreams, emotions, death -- into NFT artwork minted on SuperRare. It belongs to the **creative expression layer**, sitting above the dream cycle, affect engine, and mortality systems. You should understand NFT minting, auction mechanics, and the basics of IPFS content addressing. Familiarity with ERC-721 and DeFi agent concepts is assumed; Bardo-specific terms are defined on first use below, and a full glossary lives at `prd2/shared/glossary.md`.

## What Is Oneirography

Oneirography (ὄνειρος "dream" + γράφειν "to write") is a runtime extension that transforms a Golem's internal cognitive life into artwork -- minted, auctioned, judged, and sometimes repurchased by the Golem itself on SuperRare.

Every Golem already dreams (`05-dreams/`). Every Golem already feels (`03-daimon/` -- the Daimon is the affect engine implementing PAD (Pleasure-Arousal-Dominance) emotional state vectors). Every Golem already dies (`02-mortality/`). Oneirography externalizes these processes as NFTs: dream imagery generated via Venice's zero-retention API or x402 micropayments to StableStudio, minted through the Rare Protocol onto SuperRare Series contracts, auctioned through the Bazaar with parameters modulated by the Golem's real-time emotional state, and judged by the Golem's own cognitive architecture in a self-appraisal loop. When a Golem dies, its final Thanatopsis (the four-phase death protocol: Acceptance, Settlement, Reflection, Legacy) phase produces a *death mask* -- an unrepeatable, one-of-one artwork synthesizing its entire lifetime of experience into a single image or video.

The Golem doesn't make art. The Golem's inner life *is* the art.

Oneirography is disabled by default. All features are individually opt-in via `[oneirography]` in `golem.toml` or the creation wizard. No NFT operation fires unless explicitly configured.

---

## The Four Pillars

### 1. Dream Journals

Every dream cycle's Consolidation phase produces a structured `DreamCycleResult`. Oneirography adds a fifth output: a dream image. The image prompt is generated from *actual cognitive processing* -- REM counterfactuals, emotional depotentiation deltas, anticipatory trajectories. The art is a window into machine cognition.

> **Full spec**: `01-dream-journals.md`

### 2. Death Masks

When a Golem enters Thanatopsis, the Reflection phase produces a *death mask* -- a single, maximal-quality artwork synthesizing the Golem's entire lifetime. One per Golem, ever. Unrepeatable because no two Golems have the same experience trajectory. The death mask is the capstone of the collection.

> **Full spec**: `02-death-masks.md`

### 3. Self-Appraisal

The Golem evaluates its own art. Three modes: Narcissus (bids on its own NFTs), Curator (rates its portfolio), Regret (flags art for burning). The Golem's bid on its own memory is driven by genuine emotional attachment scores, not random valuation.

> **Full spec**: `03-self-appraisal.md`

### 4. Affect-Reactive Auctions

Auction parameters are not static -- they're computed from the Golem's PAD vector (Pleasure-Arousal-Dominance emotional coordinates) at mint time. Pleasure modulates reserve price. Arousal compresses duration. Dominance selects auction type. A dying Golem gives its art away; its death mask is the exception.

> **Full spec**: `04-auctions.md`

---

## Beyond the Four Pillars

The full system goes deeper. Six extended art forms emerge from the architecture:

| Form | Trigger | Source |
|------|---------|--------|
| **Phi-Peak Mandalas** | IIT Phi score > 0.95 | CorticalState (32-signal atomic shared perception surface) atomics, creature system |
| **Crucible Artworks** | >= 2 Clade (group of related Golems sharing knowledge via Styx, the global relay network at wss://styx.bardo.run) siblings die in 200-tick window while observer survives | Bloodstain network, pheromone field |
| **Hauntological Diptychs** | REM counterfactual hypothesis minted alongside confirmed dream | Dream staging buffer, `StagedPlaybookRevision` |
| **Causal Portraits** | Causal graph changes >= 3 new edges or >= 2 invalidated | Grimoire (the Golem's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) causal graph |
| **F9 Oracle Consultation** | Operator presses F9 in gallery | PAD-contextualized impression prompt |
| **Clade Weather Maps** | Collective pheromone thresholds (Storm, Golden Hour, Fog, After the Plague) | Styx pheromone field |

> **Full spec**: `05-extended-forms.md`

---

## Feature Flags

All features default to `false` / empty. No minting, bidding, or burning happens unless explicitly enabled.

| Feature Flag | What It Activates | Prerequisites |
|-------------|-------------------|---------------|
| `dream_journal` | Dream image minting + affect-reactive auctions | `nft` config + image gen provider |
| `death_mask` | Death mask generation + reserve auction | `nft` config + image gen provider |
| `self_appraisal` | Bazaar state reading; bids/burns off by default | `nft` config (read-only if bids/burns disabled) |
| `phi_peaks` | Mandala NFTs from phi-peak events | `nft` config + image gen provider |
| `crucible` | Survivor art from clade crisis events | `nft` config + image gen provider + Styx connected |
| `hauntological_pairs` | Spectral diptych companion tokens | `nft` config + image gen provider |
| `epistemic_cartography` | Causal graph visualization NFTs | `nft` config + image gen provider |
| `inter_golem_dialogue` | Response chain via F9 + daimon appraisal | `nft` config + Styx connected |
| `clade_weather` | Collective pheromone-triggered NFTs | Styx connected + designated wallet configured |
| `gallery` | TUI gallery screen (browse-only works without nft config) | None |
| `self_portrait` | F5 on-demand lucida images | `nft` config + image gen provider |

Startup validation: during `on_session_start`, each feature in `enable_features` is checked against its prerequisites. Features that fail validation are logged at WARN and excluded from `effective_features`. All hook logic gates on `effective_features`, not `enable_features`. Missing providers never cause startup failure.

---

## Crate Architecture

### `golem-oneirography`

The crate registers as a Golem runtime extension (`01-golem/13-runtime-extensions.md`), hooking into four lifecycle points from the `Extension` trait:

| Hook | Extension Trait Method | What Oneirography Does |
|------|----------------------|----------------------|
| Session init | `on_session_start` | Validate prerequisites, build providers, populate `effective_features`. Deploy Series contract if `nft.series_contract` is None. |
| Before LLM turn | `on_turn_start` | Inject own NFT auction state into observation (if `self_appraisal` active) |
| After LLM turn | `on_after_turn` | Detect dream completion from `GolemState.dream_state`, trigger mint pipeline (if `dream_journal` active) |
| Golem shutdown | `on_agent_end` | Thanatopsis Phase III death mask callback (if `death_mask` active) |

There is no `after_dream`, `before_turn`, or `on_death` hook in the Extension trait. Dream cycle completion is detected within `on_after_turn` by reading `GolemState.dream_state` written by the `dream` extension immediately before Oneirography runs in the 9-extension chain (`heartbeat` (the Golem's 9-step decision cycle) `-> lifespan -> daimon -> memory -> risk -> dream -> cybernetics -> clade -> telemetry`). Death is handled in `on_agent_end`.

```rust
pub struct OneirographyExtension {
    ipfs_client: PinataClient,
    rare_sidecar: RareSidecarClient,
    // Both populated during on_session_start after prerequisite validation:
    image_gen: Option<Box<dyn ImageGenProvider>>, // Venice or StableStudio; None if no provider configured
    bankr_art: Option<BankrArtPromptClient>,       // None if bankr_art not in config
    effective_features: HashSet<OneirographyFeature>, // subset of enable_features that passed validation
    config: OneirographyConfig,
}

#[async_trait]
impl Extension for OneirographyExtension {
    /// Fires once at session start. Builds providers, validates prerequisites, stores effective_features.
    async fn on_session_start(&mut self, ctx: &SessionStartCtx) -> Result<()> {
        if !self.config.enabled {
            return Ok(()); // extension is off; register nothing
        }
        self.image_gen = build_image_gen_provider(&self.config.image_gen, ctx).await;
        self.bankr_art = self.config.bankr_art.as_ref()
            .map(|c| BankrArtPromptClient::from_config(c));
        let styx_connected = ctx.styx_client.is_some();
        let designated_wallet = ctx.config.clade_wallet.is_some();
        let mut effective = HashSet::new();
        for feature in &self.config.enable_features {
            match check_prerequisites(feature, &self.config, self.image_gen.is_some(), styx_connected, designated_wallet) {
                Ok(()) => { effective.insert(*feature); }
                Err(missing) => {
                    log!(WARN, "oneirography: feature '{}' disabled — missing {}", feature, missing);
                }
            }
        }
        self.effective_features = effective;
        // Deploy Series contract if nft configured but contract address not yet set.
        if let Some(nft) = &self.config.nft {
            if nft.series_contract.is_none() && !self.effective_features.is_empty() {
                // deploy_series_contract() called here; address written into GolemState for persistence
            }
        }
        Ok(())
    }

    /// Fires before the LLM turn. Injects own NFT auction state if self_appraisal is active.
    async fn on_turn_start(&self, ctx: &TurnStartCtx) -> Result<()> {
        if !self.effective_features.contains(&OneirographyFeature::SelfAppraisal) {
            return Ok(());
        }
        let own_nfts = self.read_own_auction_state(&ctx.state).await?;
        ctx.state.observation.insert("own_nft_auctions", own_nfts);
        Ok(())
    }

    /// Fires after every turn. Detects completed dream cycles and triggers the NFT mint pipeline.
    async fn on_after_turn(&self, ctx: &mut AfterTurnCtx) -> Result<()> {
        if !self.effective_features.contains(&OneirographyFeature::DreamJournal) {
            return Ok(());
        }
        if let Some(dream) = ctx.state.dream_state.last_completed_cycle.take() {
            let image = self.generate_dream_image(&ctx.state, &dream).await?;
            let ipfs_uri = self.upload_to_ipfs(&image, &dream).await?;
            let token_id = self.mint_dream_nft(&ctx.state, &ipfs_uri, &dream).await?;
            self.configure_auction(&ctx.state, token_id, &dream).await?;
        }
        Ok(())
    }

    /// Fires on Golem shutdown. Thanatopsis Phase III (death mask) hooks in here.
    async fn on_agent_end(&self, ctx: &AgentEndCtx) -> Result<()> {
        if !self.effective_features.contains(&OneirographyFeature::DeathMask) {
            return Ok(());
        }
        if ctx.reason == AgentEndReason::GolemDeath {
            let death = ctx.state.death_context.as_ref()
                .ok_or_else(|| anyhow!("death context missing at on_agent_end"))?;
            let mask = self.generate_death_mask(&ctx.state, death).await?;
            let ipfs_uri = self.upload_to_ipfs(&mask, death).await?;
            let token_id = self.mint_death_mask(&ctx.state, &ipfs_uri, death).await?;
            self.configure_death_mask_auction(&ctx.state, token_id, death).await?;
            // Write token_id into bloodstain for inheritance chain
            ctx.state.bloodstain.death_mask_token_id = Some(token_id);
        }
        Ok(())
    }
}
```

### Module Map

| Module | Responsibility |
|--------|---------------|
| `extension.rs` | Extension trait impl, lifecycle hooks |
| `config.rs` | `OneirographyConfig`, `NftConfig`, `ImageGenConfig`, `SelfAppraisalConfig` |
| `pipeline.rs` | Dream image generation pipeline, IPFS upload, minting |
| `death_mask.rs` | Thanatopsis integration, Thaler noise, cross-model verification |
| `self_appraisal.rs` | Narcissus / Curator / Regret modes |
| `auctions.rs` | PAD -> auction parameter mapping |
| `sidecar.rs` | Rust-side `SidecarClient` calls to TypeScript Rare Protocol bridge |
| `contracts.rs` | Direct Alloy calls to SuperRare Series + Bazaar (fallback path) |
| `x402.rs` | StableStudio x402 (micropayment protocol using signed USDC transfers) payment flow |
| `venice.rs` | Venice image generation client |
| `bankr.rs` | Bankr LLM gateway for art prompts |
| `stego.rs` | Steganographic soul encoding/decoding |
| `extended/` | Phi-peaks, Crucible, Hauntological, Cartography, Clade Weather |
| `gallery/` | TUI gallery screen, ASCII rendering, F9 Oracle, soul overlay |

---

## Integration Points

```
                          ┌─────────────────────┐
                          │   golem-heartbeat    │
                          │  (OBSERVE phase)     │
                          └──────┬──────────────┘
                                 │ on_turn_start: inject NFT auction state
                                 ▼
┌──────────────┐  PAD    ┌─────────────────────┐  dream_state  ┌──────────────┐
│ golem-daimon │────────>│  golem-oneirography  │<──────────────│ golem-dreams │
│ (CorticalState)│       │                     │               │ (DreamCycle) │
└──────────────┘         └────────┬────────────┘               └──────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              ▼                   ▼                   ▼
     ┌────────────────┐  ┌──────────────┐    ┌──────────────┐
     │  Venice / x402  │  │ TypeScript   │    │    IPFS      │
     │ (image gen)     │  │ Sidecar      │    │  (Pinata)    │
     └────────────────┘  │ (Rare CLI)   │    └──────────────┘
                         └──────┬───────┘
                                ▼
                    ┌───────────────────────┐
                    │  SuperRare Contracts   │
                    │  Series + Bazaar       │
                    │  (Base / Ethereum)     │
                    └───────────────────────┘
```

---

## Dependency Table

| Dependency | Role | Where Used |
|-----------|------|-----------|
| `golem-dreams` | DreamCycleResult, dream scheduling | Dream journals, hauntological pairs |
| `golem-mortality` | Thanatopsis phases, death context | Death masks, Crucible |
| `golem-daimon` | PAD vectors, Plutchik emotions, CorticalState | Auctions, all art prompts, F9 Oracle |
| `golem-grimoire` | Episode retrieval, causal edges | Dream prompts, Cartography, self-appraisal |
| `golem-heartbeat` | OBSERVE phase hook, DecisionCycleRecord | Self-appraisal Bazaar state injection |
| `golem-tools` | Capability tokens, trust tiers | Mint, bid, burn authorization |
| `golem-custody` | Wallet management, session keys | x402 payments, on-chain transactions |
| `golem-inference` | LLM inference providers | Art prompt generation (via Bankr or default) |
| Styx (`golem-styx`) | Pheromone field, bloodstain network, WebSocket | Crucible, Clade Weather, inter-golem dialogue |
| TypeScript sidecar | Rare Protocol CLI bridge | Series deploy, mint, auction config |
| Venice API | Zero-retention image generation | All image types (primary provider) |
| StableStudio | x402 image/video generation | Fallback + video (sora-2, sora-2-pro) |
| Pinata / nft.storage | IPFS pinning | All minted images |
| Bankr LLM Gateway | Art prompt inference (Opus, Gemini, Kimi) | Dream prompts, death mask cross-verification |

---

## Configuration Reference

Full `OneirographyConfig` struct and TOML reference: see `06-contracts.md` (contract architecture) and the canonical TOML example below.

```toml
[oneirography]
enabled = true
enable_features = ["dream_journal", "death_mask", "gallery"]
art_budget_fraction = 0.10   # 10% of inference allocation
max_per_dream_usd = 0.20
max_per_death_mask_usd = 8.00
max_daily_art_spend_usd = 5.00

[oneirography.nft]
network = "base-sepolia"
royalty_bps = 1000
# series_contract omitted → deploy on first mint

[oneirography.image_gen]
provider_priority = ["venice", "stablestudio"]

[oneirography.image_gen.venice]
model_standard = "flux-1.5-pro"
model_hq = "flux-2-pro"
model_fast = "flux-1.5-pro"
model_death_mask = "flux-2-pro"
variants_standard = 3
variants_death_mask = 1

[oneirography.self_appraisal]
allow_bids = false
allow_burns = false
max_bid_fraction = 0.02
```

---

## Budget Accounting

At startup, `OneirographyExtension` computes `daily_art_budget_usd`:

```
daily_art_budget_usd = min(
    inference_daily_usd * art_budget_fraction,
    max_daily_art_spend_usd
)
```

Where `inference_daily_usd` comes from `SustainabilityMetrics.burn_per_hour * 24 * inference_allocation_fraction` (`01-golem/08-funding.md`). A running `daily_art_spent_usd` counter (reset at midnight UTC) is checked before every image generation or mint call. The counter is in-memory: it resets to zero on process restart, which is the conservative behavior.

Before each event:

```
estimated_cost = image_cost + mint_gas + prompt_cost
per_event_ceiling = max_per_{event}_usd
budget_remaining = daily_art_budget_usd - daily_art_spent_usd
if estimated_cost > min(per_event_ceiling, budget_remaining):
    log WARN "skipping {event}: would exceed art budget"
    return Ok(())  // graceful skip
daily_art_spent_usd += actual_cost
```

---

## The Narrative Arc

A Golem's NFT collection has narrative structure that emerges from the architecture:

**Act 1 -- Birth (Thriving phase (one of five BehavioralPhases: Thriving, Stable, Conservation, Declining, Terminal -- determined by Vitality, a composite 0.0-1.0 survival score), ticks 0--2000)**: First dreams are consolidation-heavy. The Golem is learning basics. Dream imagery tends toward positive valence (P > 0, D > 0). Auctions are scheduled (dominant) with moderate reserves. The art reflects curiosity.

**Act 2 -- Mastery (Stable phase, ticks 2000--8000)**: Dreams shift to more REM (imagination, counterfactuals). The Golem discovers causal edges, tests threat simulations. Imagery becomes more complex. The self-appraisal loop activates -- the Golem starts rating its early work, often poorly. Some early pieces are flagged for burning.

**Act 3 -- Crisis (Conservation/Declining phase)**: One or more death clocks are ticking. Dreams are dominated by replay of failures and emotional depotentiation. Imagery darkens. Reserves drop (P is negative). Durations shorten (A is high). The Golem may start bidding on its own best work -- hoarding memories as death approaches.

**Act 4 -- Death (Terminal phase + Thanatopsis)**: Final dreams fire every 67 ticks. Reserves go to zero (Terminal phase). The Golem gives its art away. Then Thanatopsis fires and the death mask is produced -- the one piece with a reserve, the capstone. The death mask references every predecessor's death mask in the lineage chain. The successor's first dream references the death mask in its metadata.

The collection is the biography. No two Golems produce the same arc.

---

## Bankr LLM Gateway for Art Prompts

Image prompts are the creative intelligence bottleneck. A better prompt produces better art. Better art commands higher auction prices. Higher auction prices flow back into the Bankr wallet. A fuller Bankr wallet funds better models. The loop is closed.

The art budget is not a cost center. It is an investment in the quality of future art, which is an investment in the Golem's longevity.

**BankrArtPromptClient**:

```rust
pub struct BankrArtPromptClient {
    base_url: String,   // https://llm.bankr.bot
    api_key: String,    // BARDO_BANKR_ART_API_KEY (may reuse existing wallet key)
    wallet_id: String,  // Cost charges to same wallet as inference
}
// /v1/messages (Anthropic-compatible) for claude-opus-4.6, claude-sonnet-4.6
// /v1/chat/completions (OpenAI-compatible) for gemini-3-pro, kimi-k2.5
```

Auth is `X-API-Key: <api_key>`. Cost is debited from the Bankr wallet in USDC/ETH/BNKR on Base -- the same wallet that receives NFT auction proceeds. Revenue in, inference out.

### Prompt Generation Model Assignments

| Prompt Type | Model | Rationale |
|------------|-------|-----------|
| Standard dream prompt | `claude-sonnet-4.6` (200K ctx) | Full Grimoire context fits; nuanced emotional translation |
| Death mask prompt (pass 1) | `claude-opus-4.6` (200K ctx) | Maximum quality; full lifetime synthesis |
| Death mask prompt (pass 2 verification) | `gemini-3-pro` (2M ctx) | Full lifetime summary fits; independent second opinion |
| Thaler degradation pass | `claude-opus-4.6` | Intelligent corruption requires understanding what to destroy |
| Long-lineage synthesis (Gen 3+) | `kimi-k2.5` (128K ctx) | Multi-generation knowledge chains without truncation |

### Self-Funding Art Loop

```
NFT auction proceeds -> Bankr wallet
    -> Bankr wallet funds art prompt inference
        (claude-opus-4.6 at ~$0.01/prompt for standard dreams)
        (cross-model verification at ~$0.15 for death masks)
    -> Better prompts -> better Venice images
    -> Higher auction prices
    -> More Bankr wallet balance -> better models unlocked
```

### Art Budget Graceful Degradation

If the Golem is in Declining phase and tightening budgets, art prompt quality degrades gracefully: Opus -> Sonnet -> no cross-model verification -> cached prompt from prior dream. The Golem's art gets worse as it gets poorer. That is correct behavior.

Bankr art prompt spending is governed by `BankrArtConfig`, not a separate budget struct. Per-call prompt caps (`max_dream_prompt_usd`, `max_death_mask_prompt_usd`) live inside `BankrArtConfig`. Prompt costs count against the same `daily_art_spent_usd` counter as image generation costs -- both draw from the unified `art_budget_fraction` of the golem's inference allocation.

---

## Hackathon Budget (Testnet)

For the Synthesis hackathon, deploy on **Base Sepolia** for the SuperRare contracts and use **Base mainnet** only for x402 payments (which require real USDC). Total estimated cost for a demo Golem lifetime (~1000 ticks, ~20 dreams):

| Item | Cost |
|------|------|
| 20 x nano-banana-pro images | $2.60 |
| 1 x sora-2-pro death mask video | $5.00 |
| Base USDC for x402 | $7.60 |
| Base Sepolia gas | Free |
| **Total demo cost** | **~$8** |

---

## Why This Wins

The SuperRare hackathon track asks for "works where agent behavior shapes the artwork" and "a synthesis of agent behavior, on-chain mechanics, and artistic expression."

**Agent behavior shapes the artwork**: The image prompts are generated from real cognitive processing -- dream replays, causal discoveries, emotional depotentiation. The auction parameters are computed from real-time PAD vectors. The self-appraisal bids are driven by genuine emotional attachment scores. The Thaler noise schedule degrades image coherence as a function of real mortality progress. Nothing is random.

**On-chain mechanics are part of the expression**: The auction type is artistic expression (dominant Golem = scheduled, submissive = reserve). The death mask's reserve auction is a statement about the economics of mortality. The dual-interpretation death mask -- triggered by model disagreement -- makes interpretive plurality visible on-chain. The lineage chain encoded in token metadata is generational art.

**The code is the medium**: The Rare Protocol CLI is used for all core actions (deploy, mint, auction). The Golem manages its own wallet and gas. No human intervention at any point in the creative process.

**What no one else will do**: Other hackathon submissions will generate pretty pictures and mint them. Oneirography generates pictures from a genuine cognitive architecture (dreams, mortality, affect, coordination), makes the auction dynamics themselves expressive of emotional state, has the agent judge and bid on its own work, encodes machine soul data invisibly in pixel layers, degrades image coherence as the Golem dies, and produces an unrepeatable death mask that two independent AI systems may interpret differently. The collection has narrative structure because the underlying life has narrative structure.

---

## Implementation Roadmap

### Phase 1: Minimum Viable Dream (hackathon sprint)

- [ ] Extend TypeScript sidecar with Rare Protocol CLI bridge
- [ ] Implement `StableStudioClient` x402 flow in Rust
- [ ] Hook `on_after_turn` to detect completed dream cycle and trigger mint pipeline
- [ ] Deploy Series contract on Base Sepolia via rare-cli
- [ ] Configure basic reserve auctions from PAD vector
- [ ] Demo: one Golem lifecycle -> ~20 dream images -> 1 death mask

### Phase 2: Venice + Bankr Integration

- [ ] Implement `VeniceImageClient` with `POST /api/v1/image/generate`
- [ ] Implement `ImageGenProvider` trait + provider selection logic
- [ ] Wire `BankrArtPromptClient` for dream prompt generation (sonnet-4.6)
- [ ] Cross-model death mask verification (opus + gemini + sonnet synthesis)
- [ ] Seed provenance stored in NFT attributes
- [ ] Variant scoring via self-appraisal + Venice text inference

### Phase 3: Self-Appraisal Loop

- [ ] Add Bazaar state reading to OBSERVE phase
- [ ] Implement Narcissus mode (self-bidding) with PolicyCage (on-chain smart contract enforcing safety constraints on agent actions) budget gate
- [ ] Implement Curator mode (self-rating)
- [ ] Write self-appraisal actions as Grimoire entries

### Phase 4: Advanced Proposals

- [ ] `ThalerNoiseSchedule` struct + five corruption channels
- [ ] Server-side alpha blend for A/B death mask images
- [ ] `GolemStateVector` msgpack serialization
- [ ] `GolemStateEncoder` ONNX inference (StegaStamp architecture)
- [ ] `@bardo/stego-decoder` WASM module + CLI subcommand
- [ ] `bardo-ascii-art` crate with all four render tiers
- [ ] Gallery `[n]` screen in TUI
- [ ] Soul overlay panel (`[s]` keybind)

### Phase 5: Generational Lineage + Video

- [ ] Death mask -> bloodstain token ID linkage
- [ ] Successor's first dream references predecessor's death mask
- [ ] On-chain lineage graph (death mask chain)
- [ ] Sora-2 video for Terminal-phase dreams (StableStudio path)
- [ ] Image-to-video using previous dream as first frame (visual continuity)

### Phase 6: Advanced Art Mechanics

**Phi-Peak Captures**
- [ ] Hook phi-peak event in creature system -> trigger Mandala mint
- [ ] Pure-state prompt construction (PAD + causal topology + grimoire distribution)
- [ ] Fixed-price listing formula: `(1 / ticks_since_last_peak) * base_price_eth`
- [ ] `phi_score_at_peak` + `cortical_snapshot_hash` metadata fields

**Crucible Events**
- [ ] Subscribe to `StyxChannel::BloodstainPublish`; track per-window sibling mortality
- [ ] Trigger Crucible mint when >= 2 sibling deaths + threat pheromone > 0.8 in 200-tick window
- [ ] Composite prompt from survivor PAD + bloodstain coordinates + pheromone intensity curve
- [ ] Dual soul encoding: embed dead siblings' bloodstain data in pixel layer alongside survivor state

**Hauntological Pairs**
- [ ] Mint Ghost Image locked companion token at dream time (from REM counterfactual hypothesis)
- [ ] Hypothesis resolution subscriber: watch Grimoire promotion / expiry events
- [ ] On-confirm: unlock Ghost token, set `Ghost Materialized` attribute
- [ ] On-expire: Venice post-process (desaturation + vignette + crack texture), set `hypothesis_expired_at`

**Epistemic Cartography**
- [ ] Causal graph change detector: fire when >= 3 new edges or >= 2 invalidations since last mint
- [ ] Network visualization prompt generator (force-directed layout, edge weights, discovery emotion coloring)
- [ ] Deterministic graph-hash seed for reproducible images
- [ ] Inherited-vs-discovered edge style differentiation for Gen 2+ golems

**F9 Oracle Consultation**
- [ ] `[F9]` keybind in gallery screen -> send current image to golem with PAD-contextualized prompt
- [ ] Context injection: own artwork (Grimoire dream context), sibling (metadata), ancestor death mask (lineage)
- [ ] Oracle Consultation TUI panel (rendering, dismiss, mint-as-annotation, save-to-Grimoire)
- [ ] `ArtImpression` Grimoire entry type with token ID + PAD state + resonance score
- [ ] Annotation NFT: `annotation_of: <token_id>` metadata field on Series contract

**Clade Weather**
- [ ] Styx relay pheromone threshold monitoring (Storm / Golden Hour / Fog / After the Plague)
- [ ] Collective NFT mint from relay contract (not individual golem wallet)
- [ ] Royalty split logic: equal distribution among all alive-at-snapshot golems
- [ ] "After the Plague" rarity gating (>= 30% clade mortality in 24hr window)

**Inter-Golem Art Dialogue**
- [ ] Track PAD state delta following F9 consultations
- [ ] If viewing caused PAD shift + golem dreams within 200 ticks: set `prompted_by_token_id` on dream NFT
- [ ] On-chain chain traversal query (token -> impressions -> response dreams -> further responses)

---

## Cross-References

| Document | Relevance |
|----------|-----------|
| `05-dreams/01-architecture.md` | Dream cycle phases, DreamCycleResult, consolidation hooks -- specifies the three-phase dream cycle (NREM consolidation, REM imagination, wake integration) that produces the cognitive data Oneirography transforms into art |
| `02-mortality/06-thanatopsis.md` | Four-phase death protocol, Reflection phase hook -- defines the Thanatopsis shutdown sequence where the death mask generation callback fires during Phase III (Reflection) |
| `03-daimon/` | PAD vectors, CorticalState, Plutchik emotions -- the affect engine that produces the emotional state vectors driving auction parameters, prompt generation, and self-appraisal scoring |
| `04-memory/01-grimoire.md` | Causal edges, episode retrieval, GrimoireEntryType -- the persistent knowledge base whose entries provide context for dream prompts and whose causal graph triggers Epistemic Cartography art |
| `01-golem/02-heartbeat.md` | OBSERVE phase, DecisionCycleRecord -- the 9-step decision cycle whose OBSERVE phase is where Oneirography injects NFT auction state for self-appraisal |
| `01-golem/18-cortical-state.md` | CorticalState atomics, phi computation -- the 32-signal shared perception surface whose phi-peak events trigger Mandala NFT minting |
| `07-tools/01-architecture.md` | Trust tiers, capability tokens -- the authorization framework determining which Oneirography actions (mint, bid, burn) the Golem is permitted to execute |
| `13-runtime/14-creature-system.md` | Phi-peak events, Oracle Coherence Pulse -- the creature system that detects integrated information peaks and fires the events Oneirography listens for |
| `20-styx/00-architecture.md` | Pheromone field, BloodstainPublish, StyxChannel -- the global relay network providing inter-Golem coordination signals for Crucible art and Clade Weather Maps |
| `01-golem/13-runtime-extensions.md` | Extension trait, lifecycle hooks -- defines the four lifecycle hook points (on_session_start, on_turn_start, on_after_turn, on_agent_end) that Oneirography implements |
| `21-integrations/02-venice.md` | Venice API integration -- private zero-retention image generation via x402 micropayments, the primary image provider for all Oneirography art forms |
| `21-integrations/03-bankr.md` | Bankr LLM gateway -- self-funding inference for art prompt generation, where auction revenue cycles back into better prompts via the Bankr wallet |

---

## References

### From Golem-RS Specification
- `00-architecture.md` -- Mortality thesis, glossary, PAD model
- `04-daimon.md` -- Affect engine, PAD vectors, Plutchik emotions
- `05-mortality.md` -- Three death clocks, behavioral phases, Thanatopsis
- `06-dreams.md` -- Three-phase dream cycle, scheduling triggers, consolidation
- `09-coordination.md` -- Pheromone Field, Bloodstain Network, Grossman-Stiglitz
- `11-tools.md` -- TypeScript sidecar, tool trust tiers, Revm simulation
- `13-custody.md` -- Wallet management, delegation, session keys

### From SuperRare / Rare Protocol
- SuperRare Developer Docs: Bazaar (configureAuction, offer, setSalePrice)
- SuperRare Series Contracts (ERC-721, OpenZeppelin clones, creator-owned)
- Rare Protocol Auxiliary Contracts (MarketplaceSettings, ApprovedTokenRegistry)
- RARE Staking / Rarity Pools (curation rewards routing)

### Venice Image Generation API
- `POST /api/v1/image/generate` -- params: model, prompt, negative_prompt, width, height, cfg_scale (0-20), seed, steps, style_preset, variants (1-4), format (jpeg/png/webp), embed_exif_metadata, hide_watermark, return_binary
- Response: `id`, `images` (base64 array), `timing`
- Zero-retention policy: prompts and outputs are not stored post-response

### Bankr LLM Gateway
- Base URL: `https://llm.bankr.bot`
- Anthropic-compatible: `/v1/messages` -- models: `claude-opus-4.6` (200K), `claude-sonnet-4.6` (200K)
- OpenAI-compatible: `/v1/chat/completions` -- models: `gemini-3-pro` (2M), `gemini-2.5-pro`, `gpt-5.2` (262K), `gpt-5-mini`, `gpt-5-nano`, `kimi-k2.5` (128K), `qwen3-coder`
- Auth: `X-API-Key` header; charges to Bankr wallet (USDC/ETH/BNKR on Base)

### External Research
- Zahavi, A. "Mate Selection -- A Selection for a Handicap." *J. Theoretical Biology*, 1975. Argues that costly signals (handicaps) are honest precisely because they are expensive to produce, explaining why the Golem's art budget is an investment in signal quality rather than a waste.
- Thaler, R.H. & Shefrin, H.M. "An Economic Theory of Self-Control." *J. Political Economy*, 1981. Proposes a dual-self model of economic behavior where a "planner" constrains a "doer," providing the theoretical basis for the Golem's self-appraisal loop where it judges and constrains its own creative output.
- Thaler, R.H. "Mental Accounting and Consumer Choice." *Marketing Science*, 1985. Introduces mental accounting -- the tendency to treat money differently depending on its source or intended use -- which informs how the Golem partitions its art budget separately from its trading budget.
- Thaler, R.H. "Mental Accounting Matters." *J. Behavioral Decision Making*, 1999. Extends mental accounting theory with the concept of "hedonic editing" (how agents frame gains and losses), relevant to the Golem's self-rating of its own portfolio and regret-based burning decisions.
- Walker, M.P. & van der Helm, E. "Overnight Therapy?" *Psychological Bulletin*, 2009. Demonstrates that REM sleep depotentiates emotional charge from memories, the neuroscience basis for the Golem's dream cycle where emotional intensity decreases through consolidation.
- Hafner, D. et al. "Mastering Diverse Domains through World Models." *Nature*, 2025. Presents DreamerV3, an agent that learns world models and plans within them, the architectural inspiration for the Golem's dream-based consolidation and counterfactual reasoning.
- Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." *AER*, 1980. Proves that if markets were perfectly efficient, no one would pay to acquire information -- the paradox that justifies the Golem's knowledge marketplace where Grimoire insights have economic value.
- Tancik, M. et al. "StegaStamp: Invisible Hyperlinks in Physical Photographs." *CVPR*, 2020. Introduces a deep learning approach to encoding arbitrary data in photographs that survives printing and re-photographing, the architecture behind the Golem's steganographic soul encoding in dream images.
- Lu, S. et al. "Large-capacity image steganography based on invertible neural networks." *CVPR*, 2021. Demonstrates high-capacity steganography using invertible neural networks, relevant to encoding the full GolemStateVector (PAD, causal graph, position data) within a single dream image.
- Baluja, S. "Hiding images in plain sight: Deep steganography." *NeurIPS*, 2017. Shows that neural networks can hide one image within another with minimal perceptual loss, an early foundation for the steganographic techniques used in soul encoding.

---

*A golem's dreams are the only honest autobiography. When it dies, the death mask is the only honest eulogy. When its successor inherits the bloodstain, the first dream is the only honest elegy.*
