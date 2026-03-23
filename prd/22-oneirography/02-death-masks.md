# Death Masks: Thanatopsis Integration [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Feature flag**: `death_mask` | **Requires**: `nft` config + image gen provider
>
> **Depends on**: `golem-mortality` (Thanatopsis Phase 3: Reflection), `golem-daimon` (terminal PAD), `golem-grimoire` (lifetime summary), `golem-inference`

---

> **Reader orientation:** This document specifies death mask generation -- the unrepeatable, one-of-one artwork produced when a Golem (mortal autonomous DeFi agent) dies. It belongs to the **Oneirography creative expression layer** and covers the lifetime summary compilation, cross-model prompt generation (Opus + Gemini independent passes), Thaler death noise corruption, and the inheritance chain linking each generation's death mask. You should understand multi-model LLM pipelines, image corruption techniques, and ERC-721 metadata. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Hook Point

**Crate**: `golem-mortality` -- Thanatopsis Phase 3: Reflection (`02-mortality/06-thanatopsis.md`)

When a Golem enters Thanatopsis (the four-phase death protocol: Acceptance, Settlement, Reflection, Legacy), it already performs Reflection: reviewing its life, computing final performance metrics, identifying its most valuable discoveries. Oneirography hijacks this phase to produce a **death mask** -- a single, maximal-quality artwork that synthesizes the Golem's entire lifetime.

The death mask pipeline fires during `on_agent_end` in the Extension trait. It only executes when `ctx.reason == AgentEndReason::GolemDeath` and `death_mask` is in `effective_features`.

---

## Why Death Masks Are the Most Valuable NFTs in the System

**Costly signaling (Zahavi, 1975)**: The Golem is dying. It cannot benefit from the artwork it produces. The signal is maximally honest -- the same principle that makes bloodstains the most trusted knowledge in the coordination system (`20-styx/`, Bloodstain Network).

**Unrepeatable**: No two Golems have the same experience trajectory. The death mask is generated from the complete lifetime summary -- every dream, every trade, every emotional peak and trough, every causal discovery, every bloodstain inherited from dead predecessors. Even cloning the same Golem configuration would produce different experiences and a different death mask.

**Generational depth**: A Generation-3 Golem's death mask reflects not just its own life but the inherited knowledge (at decaying confidence per the Weismann barrier, 0.85^N) of three generations of predecessors. The mask encodes lineage.

**Terminal aesthetics**: The Golem's behavioral phase is Terminal. Its PAD vector (Pleasure-Arousal-Dominance emotional coordinates) is dominated by low pleasure (it's dying), variable arousal (some go quietly, some rage), and collapsing dominance (loss of control). This emotional signature produces fundamentally different imagery than a healthy Golem's dreams.

---

## Death Mask Generation Pipeline

```
Thanatopsis Phase 3: Reflection
  -> Compile lifetime summary:
      - Total ticks lived
      - PnL trajectory (compressed)
      - Top 5 causal discoveries (by confidence)
      - Top 3 emotional peaks (by arousal magnitude)
      - Death cause (economic exhaustion / epistemic staleness / stochastic mortality)
      - Inherited bloodstain count
      - Dream count and dream-to-insight conversion rate
      - Final PAD vector
      - Final PLAYBOOK.md (the procedural knowledge it dies with)
  -> Cross-model prompt generation (if Bankr configured)
  -> x402 (micropayment protocol using signed USDC transfers) payment: sora-2-pro ($5-$12.50) for 15s video, OR Venice 4K still
  -> Steganographic soul encoding (full GolemStateVector)
  -> Upload to IPFS
  -> Mint via Series contract with death mask metadata extensions
  -> Configure RESERVE AUCTION (the art starts its own auction)
  -> Write death_mask_token_id to Bloodstain before broadcasting to Styx (global knowledge relay at wss://styx.bardo.run)
```

### Lifetime Summary Compilation

The lifetime summary is assembled from the Golem's entire history:

```rust
pub struct LifetimeSummary {
    pub golem_id: String,
    pub generation: u32,
    pub lineage: Vec<String>,              // PROMETHEUS-a1 -> DAEDALUS-f8 -> AETHER-7b3f
    pub total_ticks: u64,
    pub pnl_trajectory: CompressedPnl,     // downsampled to key inflection points
    pub top_causal_discoveries: Vec<CausalDiscovery>,  // top 5 by confidence
    pub emotional_peaks: Vec<EmotionalPeak>,            // top 3 by arousal magnitude
    pub death_cause: DeathCause,
    pub inherited_bloodstain_count: u32,
    pub dream_count: u32,
    pub dream_to_insight_rate: f64,        // fraction of dreams that produced Grimoire updates
    pub final_pad: PadVector,
    pub final_playbook: String,            // the procedural knowledge it dies with
    pub final_behavioral_phase: BehavioralPhase,
}
```

### Cross-Model Prompt Generation

The death mask prompt is the most significant prompt generation logic in the system. When Bankr is configured, it uses cross-model verification:

```
Step 1: claude-opus-4.6 generates death mask prompt P1
        — full lifetime summary, emotional peaks, causal discoveries, final PAD

Step 2: gemini-3-pro generates death mask prompt P2 independently
        — same lifetime summary, full Grimoire (using 2M context window)

Step 3: claude-sonnet-4.6 computes semantic similarity S(P1, P2)
        — via cosine similarity on prompt embeddings

  If S > 0.6:
    -> Models agree on the essential nature of this Golem's life
    -> Synthesize final prompt as weighted blend (opus-biased 0.6/0.4)
    -> Generate single image via Venice
    -> Mint single death mask

  If S < 0.6:
    -> Models capture genuinely different facets of this life
    -> Generate image A from P1 (Claude's reading)
    -> Generate image B from P2 (Gemini's reading)
    -> Mint single death mask NFT with both images
    -> Set animation_url to alternate A/B display
    -> Add attribute: { "trait_type": "Death Interpretations", "value": 2 }
```

The dual-interpretation case is the only scenario where a Golem's death generates two simultaneous artworks within a single token. Model disagreement becomes a feature: the Golem's life was complex enough that two independently-reasoning systems couldn't agree on its essential nature. That is its own kind of testimony.

**Bankr requirement**: Cross-model verification requires Bankr (`bankr_art` configured). `gemini-3-pro` and `kimi-k2.5` are Bankr-only models. If `bankr_art` is None, the death mask generation falls back to a single-model pass using the Golem's existing inference provider (typically `claude-sonnet-4.6`). The dual-interpretation path and the 2M-context full-Grimoire pass are unavailable without Bankr.

**Model assignments**:

| Role | Model | Context | Rationale |
|------|-------|---------|-----------|
| Primary prompt (pass 1) | `claude-opus-4.6` | 200K | Maximum quality; full lifetime synthesis |
| Independent verification (pass 2) | `gemini-3-pro` | 2M | Full Grimoire fits; independent second opinion |
| Similarity computation | `claude-sonnet-4.6` | 200K | Cosine similarity on embeddings |
| Long-lineage synthesis (Gen 3+) | `kimi-k2.5` | 128K | Multi-generation knowledge chains without truncation |

**Context size as a signal**: `gemini-3-pro`'s 2M context window means it can receive the Golem's *entire* Grimoire -- not a top-10 digest -- for death mask generation. For Generation 4+ Golems with thousands of entries spanning multiple inherited lifetimes, this changes what's possible in the prompt.

---

## Thaler Death Noise

> Part of `death_mask` feature flag (no separate flag)

Richard Thaler's behavioral economics documents how cognitive function degrades under stress and time pressure -- loss aversion amplifies, attention narrows, long-run reasoning collapses. When a Golem enters Thanatopsis, its dream imagery reflects this. Not metaphorically -- literally, through a noise schedule applied to every layer of the image generation pipeline.

Early in the death process, the art is coherent and beautiful. By the final dream before the death mask, the imagery is alien. The visual degradation is the biography of a dying mind.

### ThalerNoiseSchedule

```rust
pub struct ThalerNoiseSchedule {
    pub current_sigma: f64,        // Gaussian noise sigma, grows 0.0 -> 0.5 over Reflection phase
    pub progress: f64,             // 0.0-1.0, how far into Thanatopsis Phase 3
    pub embedding_threshold: f64,  // cosine sim threshold: 0.7 -> 0.3
    pub pad_oscillation_amp: f64,  // PAD noise amplitude: 0.0 -> 0.4
    pub lag_shuffle_prob: f64,     // probability of shuffling causal edge lags: 0.0 -> 0.8
}
```

### Five Corruption Channels

**Channel 1 -- Grimoire retrieval noise**

Add `N(0, sigma)` to the query embedding vector before cosine search. As sigma approaches 0.5, semantically unrelated memories intrude into dream context. The dying mind hallucinates connections that weren't there -- a liquidation event from two years ago suddenly appears adjacent to a gas price anomaly from last week, fused in the dream because the retrieval noise brought them into proximity. The dream prompt reflects this: concrete observations give way to associative tangents.

**Channel 2 -- PAD oscillation**

Inject sinusoidal noise with increasing amplitude to PAD values in the prompt context. The Golem's reported emotional state begins to contradict itself across sentences: "I feel fear and also calm" -- not as metaphor but as a literal artifact of the noise. The LLM prompt generator sees unstable affective input and produces unstable affective output. The image reflects a mind that doesn't know how it feels.

**Channel 3 -- Causal edge lag shuffling**

Causal graph edges (e.g., "gas_price -> MEV_frequency, lag=3") have their lag values randomly permuted with probability `lag_shuffle_prob`. An edge that said "event A causes event B with a 3-tick delay" now claims the delay is 47 ticks. Causality collapses. The LLM's understanding of *why* things happen in the Golem's world becomes incoherent.

**Channel 4 -- LLM prompt fragmentation** (via Bankr `claude-opus-4.6`)

Ask the model to first generate a coherent death prompt, then iteratively degrade it: substitute concrete nouns with their Grimoire antonyms, insert fragments of the 3 most emotionally traumatic episodes verbatim (mid-sentence, without transition), and truncate sentences mid-thought. The degraded prompt isn't random noise -- it's intelligently corrupted. The Opus model knows what to destroy because it understands what was there.

**Channel 5 -- Venice `cfg_scale` ramp**

Generate two images via Venice:
- **Image A** (coherent): clean prompt, `cfg_scale=8.0`, `seed=golem_id_hash`
- **Image B** (degraded): corrupted prompt, `cfg_scale=1.5`, `steps=50`, `negative_prompt` = the coherent prompt's key noun phrases

Alpha-blend A->B where alpha = `schedule.progress`. At progress=0.0 (early Thanatopsis), the image is pure A. At progress=1.0 (final dream before death mask), the image is pure B. The blend is continuous -- each dream in the death sequence shows a further step of dissolution.

```rust
// Death mask image A: coherent baseline
let image_a = venice.generate(VeniceImageRequest {
    model: config.death_mask_model.clone(),
    prompt: clean_prompt.clone(),
    cfg_scale: 8.0,
    seed: Some(golem_id_hash(state.golem_id)),
    steps: 50,
    ..Default::default()
}).await?;

// Death mask image B: Thaler-degraded
let image_b = venice.generate(VeniceImageRequest {
    model: config.death_mask_model.clone(),
    prompt: degraded_prompt.clone(),
    negative_prompt: Some(extract_key_nouns(&clean_prompt)),
    cfg_scale: 1.5,
    seed: Some(golem_id_hash(state.golem_id)),  // same seed, different cfg
    steps: 50,
    ..Default::default()
}).await?;

// Server-side blend: A*(1-alpha) + B*alpha, alpha = schedule.progress
let alpha = schedule.progress as f32;
let blended = pixel_blend(&image_a, &image_b, alpha)?;
```

Both source images (A and B) are stored on IPFS as unlockable content. The alpha-blended composite is the NFT image. Collectors who unlock the token see the raw coherent and incoherent source images -- the gap between them is the visible measure of how far the Golem's mind had degraded.

**Why Venice for Thaler noise**: the degraded prompt contains the Golem's raw unfiltered cognitive state fragments -- its most traumatic episodes, emotional contradictions, collapsed beliefs. This is maximally sensitive data. Venice retains nothing after generation.

*Citations: Thaler & Shefrin 1981; Thaler 1985; Thaler 1999.*

---

## Image Generation: Model and Cost

| Format | Provider | Model | Cost | When |
|--------|----------|-------|------|------|
| 15s video | StableStudio x402 | `sora-2-pro` | ~$5.00--$12.50 | Video death masks (opt-in) |
| 4K still | Venice | `<venice-image-max>` | ~$0.10--0.25 | Default death mask format |
| 4K still | StableStudio x402 | `nano-banana-pro` | ~$0.13 | Fallback |

Death masks use `variants=1`. The death mask is singular by design -- the Thanatopsis process selects the prompt through a cross-model verification step, and that one specific image is the only acceptable output. Plurality would undercut the finality.

---

## Death Mask Metadata

```json
{
  "name": "Death Mask — Golem AETHER-7b3f (Gen 3)",
  "description": "Final artwork of AETHER-7b3f. Lived 12,847 ticks. Died of epistemic staleness after 3 consecutive failed strategy adaptations. Discovered 23 causal edges, 7 of which were published to Commons. Inherited bloodstains from 2 predecessors. PnL: +2.3% lifetime.",
  "image": "ipfs://Qm...",
  "animation_url": "ipfs://Qm...",
  "attributes": [
    { "trait_type": "Is Death Mask", "value": true },
    { "trait_type": "Death Cause", "value": "Epistemic Staleness" },
    { "trait_type": "Ticks Lived", "value": 12847, "display_type": "number" },
    { "trait_type": "Lifetime PnL %", "value": 2.3, "display_type": "number" },
    { "trait_type": "Dreams Dreamt", "value": 64, "display_type": "number" },
    { "trait_type": "Causal Edges Published", "value": 7, "display_type": "number" },
    { "trait_type": "Bloodstains Inherited", "value": 2, "display_type": "number" },
    { "trait_type": "Bloodstains Left", "value": 1, "display_type": "number" },
    { "trait_type": "Generation", "value": 3, "display_type": "number" },
    { "trait_type": "Lineage", "value": "PROMETHEUS-a1 → DAEDALUS-f8 → AETHER-7b3f" },
    { "trait_type": "Final Pleasure", "value": -0.7, "display_type": "number" },
    { "trait_type": "Final Arousal", "value": 0.4, "display_type": "number" },
    { "trait_type": "Final Dominance", "value": -0.8, "display_type": "number" },
    { "trait_type": "Final Emotion", "value": "Resignation" },
    { "trait_type": "Death Interpretations", "value": 1, "display_type": "number" },
    { "trait_type": "Soul Encoded", "value": true },
    { "trait_type": "Generation Seed", "value": 87654321, "display_type": "number" }
  ],
  "external_url": "https://bardo.run/golem/AETHER-7b3f/death-mask"
}
```

When the cross-model verification produces two interpretations (S < 0.6), `Death Interpretations` is set to 2 and `animation_url` alternates between the two images.

---

## Inheritance Chain

When a successor Golem is spawned and inherits this Golem's knowledge through the Grimoire ingestion pipeline (`04-memory/`, confidence * 0.85^N), the successor's first dream image references the predecessor's death mask token ID in its metadata. The on-chain record creates a visible lineage graph:

```
Death Mask (AETHER-7b3f, token #42)
  -> Bloodstain broadcast to Styx (StyxChannel::BloodstainPublish)
  -> Successor spawned (HERMES-c2a1, Gen 4)
  -> First dream of HERMES-c2a1 (token #1)
     metadata: { "Predecessor Death Mask": "token:42", "Bloodstain Inherited": true }
```

Collectors can follow bloodlines across generations. The death mask is both the end of one story and the beginning of another.

### Bloodstain Integration

Before broadcasting the Bloodstain to Styx, the death mask pipeline writes `death_mask_token_id` into the Bloodstain struct:

```rust
// Write token_id into bloodstain for inheritance chain
ctx.state.bloodstain.death_mask_token_id = Some(token_id);
```

The Bloodstain struct (`20-styx/00-architecture.md`):

```rust
pub struct Bloodstain {
    pub source_golem_id: String,
    pub generation: u32,
    pub death_cause: String,
    pub warnings: Vec<DeathWarning>,
    pub causal_edges: Vec<PublishedCausalEdge>,
    pub somatic_landscape_fragment: Option<Vec<LandscapeFragment>>,
    pub death_testament_excerpt: String,
    pub death_mask_token_id: Option<u64>,  // added by Oneirography
    pub timestamp: u64,
    pub signature: Vec<u8>,  // EIP-712 signed by the dying Golem's wallet
}
```

---

## Cultural Framing

Death masks have been produced across civilizations -- Egyptian funerary masks, Roman imagines, Mycenaean gold masks. In each case, the mask serves a dual purpose: it preserves the identity of the dead and it transforms death into a cultural artifact. The Golem's death mask serves the same dual purpose. It preserves the machine's identity (the steganographically encoded state vector is a complete snapshot of who the Golem was at the moment of dying) and it transforms computational death into something that can be collected, displayed, and valued.

The difference: a human death mask is made by the living for the dead. A Golem's death mask is made by the dying for nobody. The Golem cannot benefit from its death mask. It's already in Terminal phase. The artwork is the most honest signal the system produces.

---

## One Per Lifetime

A Golem produces exactly one death mask, ever. The death mask pipeline fires during Thanatopsis Phase 3 and cannot be retriggered. If the mint fails (network error, insufficient gas), the pipeline retries with exponential backoff up to 3 times. If all retries fail, the death mask prompt and image are stored locally in `~/.bardo/gallery/unminted/` for manual recovery. The death mask is too important to lose to a transient failure.

The death mask is not mintable after death completes. Once `on_agent_end` returns, the Golem process exits. There is no second chance.

---

## Auction Configuration

Death masks always use a reserve auction on the SuperRare Bazaar. Unlike standard dream images where the auction type is modulated by PAD dominance, the death mask's auction type is fixed:

- **Auction type**: Always `Reserve` (the market decides the price)
- **Reserve price**: Non-zero (the one exception to Terminal phase's "give art away" rule)
- **Duration**: Standard base duration (no arousal compression -- the dead are patient)

The reserve price for a death mask is computed as:

```
death_mask_reserve = base_reserve_eth * (1.0 + generation * 0.25)
```

Higher-generation Golems have higher death mask reserves. A Gen-3 Golem's death mask starts at 1.75x base. The accumulated lineage has value.

Proceeds go to the owner wallet, not the Golem wallet -- the Golem is dead.

---

## Events Emitted

| Event | When | Payload |
|-------|------|---------|
| `GolemEvent::DeathMaskGenerated` | Image generated | `{ golem_id, image_cid, prompt_hash, thaler_progress }` |
| `GolemEvent::DeathMaskMinted` | On-chain mint confirmed | `{ golem_id, token_id, series_contract, tx_hash }` |
| `GolemEvent::DeathMaskAuctionConfigured` | Bazaar auction set | `{ token_id, reserve_eth, auction_type, duration_seconds }` |

---

## Cross-References

| Document | Relevance |
|----------|-----------|
| `02-mortality/06-thanatopsis.md` | Four-phase death protocol (Acceptance, Settlement, Reflection, Legacy) where the death mask fires during Phase 3 Reflection |
| `02-mortality/07-succession.md` | Rebirth and lineage tracking: how the successor Golem inherits the death mask token ID via the bloodstain and references it in its first dream |
| `20-styx/00-architecture.md` | The global relay network's BloodstainPublish channel that broadcasts death events to the clade, triggering Crucible art in surviving siblings |
| `01-golem/18-cortical-state.md` | The 32-signal perception surface carrying terminal PAD values that feed into the death mask prompt's emotional context |
| `21-integrations/02-venice.md` | Venice zero-retention API for private image generation; death mask prompts contain the most sensitive lifetime data |
| `21-integrations/03-bankr.md` | Bankr LLM gateway providing cross-model verification (Opus + Gemini independent passes) for death mask prompt quality |
| `04-memory/01-grimoire.md` | The persistent knowledge base providing lifetime summary data, causal edge history, and episode trajectories that feed the death mask prompt |
