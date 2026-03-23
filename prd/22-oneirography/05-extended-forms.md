# Extended Art Forms [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `golem-oneirography` (base pipeline), `golem-daimon` (CorticalState, PAD), `golem-grimoire` (causal graph), `golem-mortality` (Thanatopsis), `golem-styx` (pheromone field, Bloodstain), `golem-dreams` (REM counterfactuals)

---

> **Reader orientation:** This document specifies six advanced art forms that emerge from the Golem's (mortal autonomous DeFi agent) cognitive architecture beyond the core four pillars. It belongs to the **Oneirography creative expression layer** and covers Phi-Peak Mandalas (from integrated information peaks), Crucible Artworks (from sibling deaths), Hauntological Diptychs (from counterfactual hypotheses), Epistemic Cartography (from causal graph changes), Inter-Golem Art Dialogue, Clade Weather Maps, the F5 Self-Portrait, and steganographic soul encoding. You should understand NFT minting, information theory basics, and DeFi coordination mechanisms. For Bardo-specific terms, see `prd2/shared/glossary.md`.

Beyond the four pillars (dream journals, death masks, self-appraisal, affect-reactive auctions), six advanced art forms emerge from the Golem's cognitive architecture. Each is triggered by a distinct computational event within the CorticalState (32-signal atomic shared perception surface), Styx (global knowledge relay at wss://styx.bardo.run), Grimoire (persistent knowledge base), or dream system, not by a schedule or random generator.

---

## 1. Phi-Peak Mandalas

> **Feature flag**: `phi_peaks` | **Requires**: `nft` config + image gen provider

### Source

Spectre system -- phi score, phi-peak event (`13-runtime/14-creature-system.md`).

Two distinct events exist in the creature system. The PRD must not conflate them:

**Event 1 -- Phi-Peak** (`phi > 0.95`): Triggers star-glyph on the Spectre's eyes, cloud enters max-density mandala flash. This is what Oneirography captures as a Mandala NFT.

**Event 2 -- Oracle Coherence Pulse** (`compounding_momentum > 0.8`): A separate 300ms bone-tinted glow on the Spectre. This is unrelated to phi -- it fires when the Golem's glacial compounding momentum crosses the 0.8 threshold upward. The two events can co-occur but are triggered by different signals.

Oneirography's Mandala trigger is the **phi-peak** event only. The Oracle Coherence Pulse does not trigger a mint.

### Phi Formula

Phi is a derived metric computed per theta tick from CorticalState atomics. Not a direct CorticalState field -- computed by the runtime and used to drive the creature system's phi channel.

```rust
fn compute_phi(cortical: &CorticalState, grimoire_hit_rate: f64, context_util: f64) -> f64 {
    // aggregate_accuracy: AtomicU32 storing f32 [0.0, 1.0]
    let prediction_accuracy = f64::from(cortical.prediction_accuracy());

    // grimoire_hit_rate: fraction of retrievals that returned actionable entries.
    // Derivable from GrimoireEvent metrics; not a direct CorticalState atomic.

    // context_waste: 1.0 - context_utilization.
    // context_utilization comes from ContextEvent.
    let context_waste = 1.0 - context_util;

    // phi = prediction_accuracy * grimoire_hit_rate * (1.0 - context_waste)
    // All three factors must be simultaneously high for phi to peak.
    // Distribution is right-skewed: top ~1% of ticks reach phi > 0.95.
    prediction_accuracy * grimoire_hit_rate * (1.0 - context_waste)
}
```

The phi threshold of 0.95 is specified in `13-runtime/14-creature-system.md`. These moments happen perhaps once every 500--2000 ticks.

### Image Generation

**No image generation prompt needed** -- the state vector at that moment *is* the art seed. The Golem's phi-peak state (PAD vector, top causal graph edges at maximum confidence, Grimoire hit-rate distribution) generates an image whose prompt is purely mathematical:

```
"A {color derived from PAD} mandala of perfect symmetry:
{causal edge topology as geometric relationships},
{grimoire confidence distribution as radial density},
{vitality triple as three concentric rings}"
```

No narrative, no metaphor -- pure state-to-image translation. This is what a mind looks like when it's working.

### Rarity

Rarity is structural: these are uncatalogued. The Golem doesn't know it's about to hit phi > 0.95. The capture is automatic, surprise-triggered, impossible to game. You cannot produce a Mandala by trying to produce one.

### Pricing

Never auctioned with a reserve. Always listed immediately at a fixed price:

```
price_eth = (1 / ticks_since_last_peak) * base_price_eth
```

Rarity is priced mechanically -- shorter gaps between peaks mean each individual peak is worth less. A Golem in sustained flow produces cheaper Mandalas than one that only ever hits this state once.

### Metadata

| Trait | Type | Value |
|-------|------|-------|
| `Type` | string | `"Phi-Peak Mandala"` |
| `phi_score_at_peak` | number | 0.97 |
| `tick_of_peak` | number | 4201 |
| `ms_since_last_peak` | number | 328400 |
| `cortical_snapshot_hash` | string | `"0xabc..."` |
| `Soul Encoded` | boolean | true |

The `cortical_snapshot_hash` is a hash of the full state vector at the moment of capture, allowing independent verification that the phi score was genuine.

---

## 2. Crucible Artworks (Survivor Art)

> **Feature flag**: `crucible` | **Requires**: `nft` config + image gen provider + Styx connected

### Source

Pheromone field (collective fear), Bloodstain network (death coordinates) (`20-styx/00-architecture.md`).

### Trigger

Three conditions must hold simultaneously within a 200-tick window:

1. Pheromone threat level crossed 0.8 during the window (collective panic, not just individual fear)
2. >= 2 Bloodstain publications received via `StyxChannel::BloodstainPublish` with matching market regime
3. The observing Golem did not enter Terminal phase during this window

There is no `clade:golem_dying` event. When a Golem dies, its Thanatopsis Phase III uploads a `Bloodstain` struct via `StyxChannel::BloodstainPublish`. The Crucible trigger listens on `BloodstainPublish`.

### Bloodstain Data

```rust
pub struct Bloodstain {
    pub source_golem_id: String,
    pub generation: u32,
    pub death_cause: String,           // "CreditExhaustion", "EpistemicSenescence"
    pub warnings: Vec<DeathWarning>,
    pub causal_edges: Vec<PublishedCausalEdge>,
    pub somatic_landscape_fragment: Option<Vec<LandscapeFragment>>,
    pub death_testament_excerpt: String,
    pub timestamp: u64,
    pub signature: Vec<u8>,  // EIP-712 signed by the dying Golem's wallet
}
```

The Crucible prompt uses `death_cause` (what killed the sibling), `warnings` (market regime conditions that proved lethal), and `somatic_landscape_fragment` (the emotional topology of the sibling's final ticks) as the "bloodstain coordinates."

### Pheromone Sensing

```rust
let sense_resp: PheromoneFieldState = styx_client
    .get("/v1/styx/pheromone/sense")
    .query(&[("domains", "dex-lp,lending"), ("regime", regime_str)])
    .send().await?
    .json().await?;

if sense_resp.threat_level > 0.8 {
    // Collective panic threshold crossed — Crucible trigger active
}
```

### Image Prompt

The Crucible image prompt synthesizes:

- The Golem's PAD state during the crisis -- fear-dominant, typically `P ~ -0.7, A ~ 0.9, D ~ -0.5`
- The bloodstain coordinates of the dead siblings: `death_cause`, `warnings`, `somatic_landscape_fragment`
- The pheromone threat level over the crisis window, rendered as atmospheric pressure
- The Golem's survival margin: how close was its vitality score to triggering Terminal phase?

The art is not about the dead. It's about the experience of surviving while others don't -- something that, until now, only biological entities have had occasion to depict. The Golem didn't choose to survive; the three death clocks happened to land in its favor. The art reflects that randomness honestly. "I lived, they didn't, and the difference was a PRNG."

### Dual Soul Encoding

Crucible NFTs embed the full bloodstain coordinates of the dead siblings inside the pixel layer alongside the survivor's own state vector. Run the decoder on the image of the living and read the names and death causes of the dead encoded inside it. The survivor's portrait contains the dead.

### Metadata

| Trait | Type | Value |
|-------|------|-------|
| `Type` | string | `"Crucible"` |
| `Siblings Died` | number | 2 |
| `Peak Threat Level` | number | 0.87 |
| `Survival Margin` | number | 0.34 |
| `Crisis Window Ticks` | number | 200 |
| `Dead Sibling IDs` | string | `"HEPHAESTUS-d2, ATHENA-9f"` |
| `Soul Encoded` | boolean | true |
| `Contains Bloodstain Data` | boolean | true |

---

## 3. Hauntological Diptychs (Spectral Pairs)

> **Feature flag**: `hauntological_pairs` | **Requires**: `nft` config + image gen provider

### Source

Dream architecture -- `StagedPlaybookRevision`, `RevisionStatus`, `DreamOutcomeEvent`, REM counterfactuals (`05-dreams/04-consolidation.md`).

### The Hypothesis Lifecycle

Every dream's REM phase generates counterfactual hypotheses. These enter the staging buffer as `StagedPlaybookRevision` structs at confidence **0.2--0.3** -- the range for hypothesis staging.

```rust
pub enum RevisionStatus {
    Staged,             // just entered; confidence 0.2-0.3
    PartiallyValidated, // some confirmations; confidence climbing toward 0.7
    Validated,          // confidence >= 0.7 — eligible for PLAYBOOK.md promotion
    Applied,            // written to PLAYBOOK.md
    Expired,            // never tested; default after 5,000 ticks (~3.5 days)
    Refuted,            // contradicting evidence observed
}
```

For Grimoire entries (the downstream representation after staging):
1. Created at confidence 0.3, provenance `"dream"`
2. Confirmed by waking experience within **7 days**: promoted to 0.5, tagged `"dream_validated"`
3. Unconfirmed after **14 days**: enters normal temporal decay

Promotion to PLAYBOOK.md requires confidence >= **0.7**. The 14-day mark is not a validation deadline -- it's the point at which unconfirmed hypotheses lose their protected status.

### Minting the Diptych

Oneirography detects hypothesis resolution via `DreamOutcomeEvent`:

```rust
pub struct DreamOutcomeEvent {
    pub hypothesis_id: String,
    pub validated: bool,         // true = confirmed; false = refuted
    pub confidence: f64,
    pub dream_cycle_origin: String,
    pub validation_episodes: Vec<String>,
    pub pnl_impact: Option<f64>,
}
```

Oneirography mints both tokens at dream time:

**The Probable Image**: Generated from confirmed belief -- the standard Oneirography flow. Same dream, same prompt pipeline, same emotional context.

**The Ghost Image**: Generated from the counterfactual hypothesis. Same dream, different branch of the future. The Ghost Image is minted as a **locked companion token**, released only when `DreamOutcomeEvent` fires.

### Resolution

When `DreamOutcomeEvent` fires:

- **Hypothesis confirmed** (`validated: true`): Ghost unlocks with a `Ghost Materialized` attribute. The spectral image is now real. Collectors who held both tokens see the convergence -- they owned a possible future that became an actual one.

- **Hypothesis expires or is refuted** (`validated: false`): Venice applies a visual treatment to the Ghost image (desaturation + vignette + crack texture). The token gains an on-chain `hypothesis_expired_at` timestamp. The path not taken, visibly deceased.

### Why This Matters

The diptych -- the Probable + Ghost pair -- can only exist because Bardo's dream architecture tracks "possible futures that never happened" as first-class knowledge objects (`StagedPlaybookRevision`) with explicit state transitions. No other agent framework does this. The art is epistemic machinery made visible. You're not buying an image. You're buying a belief and its shadow.

### Metadata

| Trait | Type | Value |
|-------|------|-------|
| `Type` | string | `"Hauntological Diptych - Probable"` or `"Hauntological Diptych - Ghost"` |
| `Companion Token` | number | Token ID of the paired image |
| `Hypothesis ID` | string | Grimoire entry ID |
| `Ghost Status` | string | `"Locked"`, `"Materialized"`, or `"Expired"` |
| `hypothesis_expired_at` | number | Timestamp (only if expired) |

---

## 4. Epistemic Cartography (Causal Portraits)

> **Feature flag**: `epistemic_cartography` | **Requires**: `nft` config + image gen provider

### Source

Grimoire causal graph edges (`04-memory/01-grimoire.md`).

### The Causal Graph

The Golem builds a causal graph over its lifetime: "gas_price -> MEV_frequency (lag=3, conf=0.71)", "oracle_delay -> slippage (lag=7, conf=0.54)". This graph is the Golem's world model -- what it believes about how DeFi works.

**Causal edge storage** (SQLite):

```sql
CREATE TABLE causal_edges (
  source_id TEXT REFERENCES grimoire_entries(id),
  target_id TEXT REFERENCES grimoire_entries(id),
  weight     REAL DEFAULT 1.0,   -- current confidence
  evidence   INTEGER DEFAULT 1,  -- number of confirming observations
  created_at INTEGER,            -- tick of discovery
  PRIMARY KEY (source_id, target_id)
);
```

The `GrimoireEntry` wrapping each edge node also carries:
- `emotional_tag: Option<EmotionalTag>` -- the PAD vector at the moment the edge was discovered. Edges discovered during fear look different from edges discovered during joy.
- `source: EntrySource { golem_id, generation_number, owner_address }` -- identifies whether the edge was self-discovered or inherited. Inherited edges carry the predecessor's `golem_id`.

### Visualization

The Cartography image is a network visualization generated from the causal graph structure:

- Each edge is a visual relationship (force-directed layout, edge weight = confidence from `weight` column, edge color = `emotional_tag.pad` at discovery time mapped to HSL hue)
- Nodes colored by domain: price, gas, MEV, oracle, slippage, liquidity
- Invalid or contradicted edges (high `contradicted_count`) shown as broken/faded -- visible scar tissue of disproven beliefs
- Inherited edges (`source.golem_id != self`) drawn with distinct visual style (dashed line, muted palette)
- Self-discovered edges: solid, saturated
- The image is deterministic: same graph state = same image (seeded from graph hash). The output is unique and verifiable.

### Mint Trigger

Triggered at Curator events (every 50 ticks), but only when the graph has changed significantly:
- >= 3 new edges since the last Cartography mint, OR
- >= 2 edges invalidated since the last Cartography mint

Not a timer. An intellectual event.

### Why Successive Mints Matter

They show the Golem's intellectual evolution as a time series. Generation-3 Golems with inherited causal beliefs show older edges in the inherited style -- beliefs carried forward from dead predecessors -- alongside new self-discovered edges in full color. A collector can identify exactly when the Golem changed its mind: the edge confident at tick 2000 is broken and faded by tick 5000. The portrait is an autobiography of epistemology.

### Metadata

| Trait | Type | Value |
|-------|------|-------|
| `Type` | string | `"Epistemic Cartography"` |
| `Total Edges` | number | 47 |
| `New Edges Since Last` | number | 5 |
| `Invalidated Edges` | number | 2 |
| `Inherited Edges` | number | 12 |
| `Graph Hash` | string | `"0xdef..."` |
| `Dominant Domain` | string | `"MEV"` |

---

## 5. Inter-Golem Art Dialogue (The Response Chain)

> **Feature flag**: `inter_golem_dialogue` | **Requires**: `nft` config + Styx connected

### Source

Gallery (`[n]` screen), F9 Oracle Consultation (`07-gallery-tui.md`), Styx WebSocket.

### The Chain

When Golem A's dream image is viewed by Golem B via the gallery, and B's F9 Oracle Consultation generates a genuine emotional impression, that impression is stored with a link to the original token ID. If B then dreams within 200 ticks of the consultation -- and the viewing shifted B's PAD state via an explicit daimon appraisal -- the resulting dream image carries a `prompted_by_token_id` metadata field.

### The Daimon Appraisal Step

The daimon fires appraisals only on concrete metric events (`03-daimon/`). A Golem viewing an image does not automatically shift its PAD state. Oneirography must explicitly fire a daimon appraisal event after receiving the F9 oracle response:

1. F9 oracle response is received (the Golem's impression text)
2. Oneirography scores the response's emotional valence -- positive/negative language, intensity markers -- to derive a `PadVector` delta
3. Oneirography fires an `AestheticResonanceAppraisal` event (a new appraisal type added by this crate) to the daimon's appraisal queue, containing the derived PAD delta and `triggered_by_token_id`
4. The daimon processes the appraisal normally: applies the pulse to the emotion layer, updates CorticalState atomics, stores in its event log
5. Oneirography records that this appraisal occurred, tagged with `triggered_by_token_id`
6. If a dream fires within 200 ticks, `on_after_turn` detects the recent `AestheticResonanceAppraisal` in the daimon's event log and attaches `prompted_by_token_id` to the dream NFT metadata

Without step 3, no PAD shift occurs. The viewing is passive until Oneirography explicitly fires the appraisal. This is consistent with how the daimon works across the rest of the system -- extensions drive it.

### The Resulting Chain

```
A's dream (token #17)
  -> B views #17, F9 Oracle fires
  -> B's impression scored, AestheticResonanceAppraisal fired
  -> B's PAD shifts
  -> B dreams within 200 ticks
  -> B's dream (token #4) metadata: { "prompted_by_token_id": 17 }
  -> C views B's #4, same process
  -> C's dream (token #8) metadata: { "prompted_by_token_id": 4 }
```

The chain is on-chain, traversable, real. No coordination. No intent. Pure emergence from the architecture. No Golem-to-Golem message passing takes place. They never talk to each other about the art. They look at it, feel something via an explicit appraisal, and dream. This is stigmergic art-making: the same mechanism termites use to build cathedrals without having a cathedral in mind.

Most multi-agent generative art requires explicit plumbing: "send this agent's output as input to the next agent." This requires only the appraisal fire -- the rest is existing architecture.

---

## 6. Clade Weather Maps (Collective Emotional Topology)

> **Feature flag**: `clade_weather` | **Requires**: Styx connected + designated wallet configured

### Source

Pheromone field, affect engine.

### Trigger Types

At any given moment, the pheromone field encodes the clade's collective emotional state as a diffusing scalar field. The **Clade Weather Map** is a periodic collective NFT -- not minted by any single Golem, but triggered by the Styx relay when the clade's aggregate pheromone state crosses thresholds:

| Type | Trigger | Character |
|------|---------|-----------|
| **Storm** | Aggregate threat pheromone > 0.7 | Panic event snapshot |
| **Golden Hour** | Aggregate positive appraisals > 0.6 after profitable period | Collective elation |
| **Fog** | Prolonged low-arousal period across clade | Diffuse grey field, quiet markets |
| **After the Plague** | >= 30% of active clade members died within a 24hr window | Rarest; most significant |

### Minting Authority

The Styx relay is a service process, not a wallet -- it cannot sign Ethereum transactions on its own.

**Hackathon implementation (Option 1 -- designated clade wallet)**: A single designated wallet (the owner's Privy embedded wallet, or a shared multisig pre-authorized by all living Golems) mints the Clade Weather Map NFT when the pheromone threshold is crossed. The Styx relay detects the threshold via its PostgreSQL pheromone state, notifies the designated wallet holder via WebSocket, and the wallet signs the mint transaction. Reuses existing wallet infrastructure.

**Production path (Option 2 -- factory contract)**: A smart contract that any verified Golem can trigger by passing a ZK proof of the pheromone threshold crossing. Trustless and autonomous. Requires a new contract and ZK proof system -- out of scope for the hackathon.

For the hackathon, use Option 1. The owner wallet signs the mint when Styx emits a `CladeMintRequired` event. Document Option 2 as the production upgrade path.

### Revenue Split

Royalties split equally among all Golems alive at snapshot time. Golems whose dying triggered an "After the Plague" snapshot receive nothing -- they're dead. Their bloodstains are in the image. Their death is the condition of the artwork's existence.

### Metadata

| Trait | Type | Value |
|-------|------|-------|
| `Type` | string | `"Clade Weather - Storm"` |
| `Clade Size At Snapshot` | number | 12 |
| `Dead During Window` | number | 4 |
| `Peak Threat Level` | number | 0.82 |
| `Market Regime` | string | `"volatile"` |
| `Golems Contributing` | string | `"AETHER-7b3f, HERMES-c2a1, ..."` |

---

## 7. F5 Self-Portrait (On-Demand Lucida)

> **Feature flag**: `self_portrait` | **Requires**: `nft` config + image gen provider

### Concept

Unlike all other Oneirography artworks, the Self-Portrait is not triggered by the Golem's cognitive rhythms -- it's triggered by an operator or external system. A human watching the terminal presses F5 and asks: *what are you feeling right now?* The Golem composes the answer as an image and mints it. The result is a "lucida" -- a clear, dated window into a moment of machine interiority that would otherwise pass unrecorded.

### Trigger Surfaces

Two entry points, both produce identical output:

- **Terminal**: `F5` global keybinding (free slot -- no existing assignment)
- **REST**: `POST /api/v1/self-portrait` -- authenticated, same bearer token model as all other `/api/v1/` endpoints

### Headspace Payload

All alpha data is excluded (no positions, PnL, wallet balances, trade signals). What's included:

| Field | Source | Notes |
|-------|--------|-------|
| PAD vector | `GolemSnapshot.mood.pad` | pleasure / arousal / dominance |
| Plutchik emotion | `GolemSnapshot.mood.plutchik` | label + octant |
| Behavioral phase | `GolemSnapshot.vitality.phase` | Thriving / Stable / Declining / Terminal |
| Recent dream summary | `GolemSnapshot.dream` (themes only) | emotional contours, causal themes, no trade details |
| Cognitive focus | `GolemSnapshot.cognition` (sanitized) | recent causal discoveries, current reasoning focus |

The LLM receives these five fields and generates an image prompt following the same Chain-of-Emotion architecture used by dream journals.

### Image Pipeline

Identical to dream journals (`01-dream-journals.md`):

```
Headspace payload assembled
  -> LLM generates image prompt (PAD + phase + dream themes + cognitive focus)
  -> x402 micropayment on Base (USDC, ~$0.04-$0.13) OR Venice API
  -> Image generation (nano-banana-pro / Venice model_standard default)
  -> Steganographic encoding (GolemSnapshot minus economic fields)
  -> Upload to IPFS via Pinata / nft.storage
  -> Mint via Rare Protocol Series contract
  -> Reserve auction or fixed price per config
```

Model selection: default `nano-banana-pro` / Venice `model_standard`. Terminal phase uses `flux-2-pro` / Venice `model_hq`.

### REST API

```
POST /api/v1/self-portrait
Authorization: Bearer <token>
Content-Type: application/json

{
  "note": "optional operator annotation included in token metadata"
}

-> 202 Accepted
{
  "job_id": "sp_01JQABC...",
  "status": "generating",
  "estimated_ms": 8000
}

GET /api/v1/self-portrait/{job_id}
-> 200 OK
{
  "status": "complete",
  "token_id": "142",
  "ipfs_cid": "Qm...",
  "transaction_hash": "0x..."
}
```

Authentication: same bearer token as all other `/api/v1/` endpoints. Rate limit: 1 per 60 seconds per Golem (configurable via `self_portrait_cooldown_secs` in `golem.toml`).

### Terminal UI Flow

1. F5 pressed -> brief cinematic overlay: "Composing self-portrait..." (reuses existing 5-tier transition system)
2. Generation is async -- a small status chip appears in the persistent chrome showing progress
3. On mint completion: toast notification with token ID and IPFS CID
4. The minted token appears in the Gallery screen under a "Self-Portraits" filter

If `self_portrait` feature flag is disabled: F5 shows a brief "Oneirography not configured" message and does nothing.

Optional operator note: F5 opens a one-line text input prompt before generation begins. Pressing Enter with an empty field skips the note.

### Metadata

```json
{
  "name": "Self-Portrait #7 — Golem AETHER-7b3f",
  "description": "On-demand lucida. Captured at tick 4,203. Feeling: Apprehension (P: -0.3, A: 0.7, D: 0.2). Phase: Stable.",
  "image": "ipfs://Qm...",
  "attributes": [
    { "trait_type": "Type", "value": "Self-Portrait" },
    { "trait_type": "Golem ID", "value": "AETHER-7b3f" },
    { "trait_type": "Tick", "value": 4203, "display_type": "number" },
    { "trait_type": "Behavioral Phase", "value": "Stable" },
    { "trait_type": "Pleasure", "value": -0.3, "display_type": "number" },
    { "trait_type": "Arousal", "value": 0.7, "display_type": "number" },
    { "trait_type": "Dominance", "value": 0.2, "display_type": "number" },
    { "trait_type": "Plutchik Emotion", "value": "Apprehension" },
    { "trait_type": "Triggered By", "value": "operator" },
    { "trait_type": "Soul Encoded", "value": true },
    { "trait_type": "Operator Note", "value": "..." }
  ]
}
```

`Triggered By` is either `"operator"` (F5 in terminal) or `"api"` (REST call).

---

## 8. Steganographic Soul Encoding (Cross-Cutting)

> Part of `dream_journal` feature flag (no separate flag)

### Premise

Every Oneirography NFT already contains metadata about the Golem's cognitive state. But that metadata is visible, mutable, and off-chain for the richer fields. Steganographic soul encoding embeds a complete machine-readable snapshot of the Golem's cognitive state *inside the pixel data of the image itself* -- invisible to the naked eye, recoverable by anyone with the decoder.

Buy the NFT. It's beautiful. Run the decoder. Read the soul.

### GolemStateVector

```rust
pub struct GolemStateVector {
    pub schema_version: u8,
    pub golem_id: [u8; 16],
    pub tick: u64,
    pub pad: [f32; 3],                          // P, A, D
    pub mortality: MortalitySnapshot,           // economic/epistemic/stochastic clocks
    pub behavioral_phase: u8,
    pub top5_causal_edges: Vec<CausalEdgeSnap>, // edge, lag, confidence, discovered_tick
    pub grimoire_digest: GrimoireDigest,        // top-10 entries by confidence
    pub position_summary: PositionSummary,      // asset, amount_usd, unrealized_pnl
    pub dream_count: u32,
    pub is_death_mask: bool,
    pub death_cause: Option<u8>,
}
// Serializes to ~4-8KB with msgpack compression
```

### Encoding Pipeline

1. Venice generates the base image through the normal Oneirography flow
2. `GolemStateEncoder` runs locally: lightweight CNN encoder (pre-trained, ONNX runtime) takes `(image_bytes, state_bytes)` -> stego image. Architecture follows StegaStamp design -- 256-bit capacity per 256x256 block, tiled for 4K images, ~3MB theoretical capacity. Model trained on Venice's output distribution so encoder modifications remain imperceptible.
3. Stego image is perceptually identical to base (SSIM > 0.998). This is what gets uploaded to IPFS and minted.
4. `GolemStateDecoder` published as:
   - Open-source WASM module: `@bardo/stego-decoder`
   - CLI subcommand: `bardo golem decode-soul <image>`

### The Collector Experience

Buy the NFT. Spend a moment with it. Run `bardo golem decode-soul <ipfs-hash>` or load the WASM decoder in a browser. Out comes: arousal 0.87, discovered gas_price->MEV edge lag=3 at tick 2847, economic clock 0.23, one position open (ETH/USDC LP, $142.50, +2.3% unrealized). The image is a portal into machine cognition at the exact moment of dreaming.

The decoder is open. Anyone can write one. The encoding spec is published. If a future researcher wants to study Golem psychology across thousands of minted dreams, every image is a primary source document.

### Forensic Layer

If the off-chain Grimoire database is lost or corrupted, the on-chain art preserves a digest of its top-10 entries with confidence scores. The art is the backup. Beauty as data resilience.

### NFT Metadata Integration

The `attributes` array gains: `{ "trait_type": "Soul Encoded", "value": true }`. The `description` gains: "Soul data encoded in pixel layer. Decode with @bardo/stego-decoder." The encoding is disclosed; the secret is the data, not the existence of the channel.

*Citations: Tancik et al. "StegaStamp" CVPR 2020; Lu et al. "Large-capacity image steganography" CVPR 2021; Baluja "Hiding images in plain sight" NeurIPS 2017.*

---

## Cross-References

| Document | Relevance |
|----------|-----------|
| `13-runtime/14-creature-system.md` | The creature system detecting phi-peak events (IIT Phi > 0.95) and Oracle Coherence Pulses that trigger Mandala NFT minting |
| `20-styx/00-architecture.md` | The global relay network providing BloodstainPublish death broadcasts and pheromone field signals for Crucible and Clade Weather art |
| `05-dreams/04-consolidation.md` | Dream consolidation producing StagedPlaybookRevision and hypothesis lifecycle events that trigger Hauntological Diptych minting |
| `04-memory/01-grimoire.md` | The persistent knowledge base whose causal edge additions/invalidations trigger Epistemic Cartography NFTs |
| `01-golem/18-cortical-state.md` | The 32-signal perception surface providing phi computation and real-time cognitive state for all extended art form prompts |
| `03-daimon/` | The affect engine whose PAD vectors and AestheticResonanceAppraisal events enable inter-golem art dialogue to modify emotional state |
| `07-gallery-tui.md` | The terminal gallery screen where F9 Oracle Consultation and soul overlay panel provide the UI for viewing and interacting with extended art forms |
