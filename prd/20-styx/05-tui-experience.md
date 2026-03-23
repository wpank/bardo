# The Terminal as MMO: Sprites, Onboarding, and the Living Interface [SPEC]

> **PRD2 Section**: 20-styx | **Source**: Styx Research S6 v4.1

> **Status**: Implementation Specification
>
> **Note**: The canonical TUI screen system is the 29-screen, 6-window hierarchy defined in `prd2/18-interfaces/screens/03-interaction-hierarchy.md`. This document's 11-screen layout is retained as the Styx-specific reference and may diverge.
>
> **Crates**: `bardo-terminal` (main binary), `bardo-sprites` (sprite engine), `bardo-tui-widgets` (custom ratatui widgets)
>
> **Dependencies**: `prd2/20-styx/00-architecture.md` (event streaming), `prd2/18-interfaces/03-tui.md` (TUI system), `prd2/03-daimon/` (PAD model)

> **Reader orientation:** This document specifies the terminal user interface for Bardo: a 60fps Rust/ratatui binary (`bardo-terminal`) that connects to both a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) and Styx (global knowledge relay and persistence layer) via WebSocket. It covers onboarding, procedural sprite generation, the 11-screen system, custom widgets, sound design, and the dead-Golem memorial experience. The key concept is that the Daimon (the affect engine implementing PAD emotional state as a control signal) drives sprite expression and aura. See `prd2/shared/glossary.md` for full term definitions.

---

## What This Document Covers

1. Architecture: 60fps Rust/ratatui, connected via WebSocket to Golem + Styx
2. Onboarding: authentication, wallet setup, Golem creation ("The Summoning")
3. Procedural sprite generation: deterministic visual identity from wallet seed
4. The screen system: 11 views with persistent chrome and animated sidebar
5. Streaming data visualization: custom widgets for sparklines, heatmaps, timelines
6. Sound design and accessibility
7. Returning to dead Golems

---

## 1. Architecture

The TUI is a standalone Rust binary (`bardo-terminal`) that connects to two WebSocket streams:

```
+----------------------------+        WebSocket        +--------------------+
|   bardo-terminal           |<--- Golem events -------|  Golem VM          |
|   (Rust / ratatui)         |---- steers/commands --->|  (Fly.io)          |
|                            |       :8080              |                    |
|   60 FPS render loop       |                          |  Event Fabric      |
|   Sprite engine            |        WebSocket         |  Heartbeat FSM     |
|   Particle system          |<--- Styx events ---------|                    |
|   Custom widgets           |       :8080/ws           +--------------------+
|   Sound engine (rodio)     |                          |  Styx Service      |
|   Input handling           |                          |  (Fly.io)          |
+----------------------------+                          +--------------------+
```

The render loop runs at 60fps (16.6ms per frame) independently of event arrival. Events update shared state; the renderer reads state each frame. No frame ever waits on a network call.

**Design reference**: Rebels in the Sky (ratatui 60fps animated game proving the framework's capability) [REBELS-IN-THE-SKY].

### Installation

```bash
cargo install bardo-terminal
# Or via release binary:
curl -sSL https://get.bardo.run | sh
```

---

## 2. Onboarding: The First Thirty Seconds

The first-run experience makes the user feel they've entered a world, not opened a tool. Atmospheric elements are functional -- the boot sequence IS real initialization.

### Step 1: Capability Detection (<1 second)

After the branded ASCII banner renders (~200ms), the terminal auto-detects: color support (true color / 256 / 16), unicode level (braille / box-drawing / ASCII), terminal size, Kitty graphics protocol support, and whether an existing config exists at `~/.bardo/`.

### Step 2: Authentication -- Browser Handoff

Mirrors the Claude Code / Codex pattern: terminal opens browser, user authenticates there, terminal detects completion via polling.

```
Terminal generates session ID -> opens bardo.run/auth?session=xxx
  |
User authenticates in browser (Privy: email / Google / GitHub / wallet)
  |
Terminal polls /auth/status?session=xxx every 2 seconds
  |
On success: JWT + wallet address + wallet ID returned
  |
Stored in ~/.bardo/auth.json (0o600 permissions)
```

While waiting: the terminal shows a copyable URL, a QR code (for mobile), and keyboard shortcuts `[c]` Copy URL, `[q]` Cancel. For headless SSH: device code flow -- terminal shows `BARD-7F3A`, user visits `bardo.run/device`, enters code.

### Step 3: Wallet Funding

The terminal queries the Privy embedded wallet's USDC balance on Base. If below $25 (operational floor), offers inline swap (ETH->USDC), bridge (opens LI.FI), buy with card (MoonPay), or fund manually.

### Step 4: The Summoning (Golem Creation)

The creation flow is atmospheric. A placeholder sprite -- glitching, unresolved, rendered as animated noise -- occupies the center of the screen. The user provides intent via freeform prompt or template selection.

During provisioning, the sprite progressively resolves:

| Provisioning Step | Sprite Resolution Stage |
|------------------|------------------------|
| Strategy compiled | Body outline emerges from noise |
| Wallet created | Color palette locks (derived from wallet address hash) |
| Identity registered (ERC-8004) | Face / eyes appear |
| Compute provisioned (Fly.io VM) | Accessories render |
| First heartbeat received | Sprite becomes animated -- eyes open |

The review screen shows the fully-formed sprite alongside strategy details, cost breakdown, and funding recommendation. Confirmation word: "SUMMON" (thematic, short, unmistakable).

### Step 5: First Heartbeat

Tick 0001 plays out in slow-motion: probes fire with visible RPC calls, market state populates, the first observation logs. The user presses any key and the full multi-pane TUI materializes around the sprite.

---

## 3. Procedural Sprite Generation

### Seed Derivation

Every Golem's visual identity is deterministically derived from a 256-bit seed:

```rust
let sprite_seed = keccak256(encode_packed(
    golem_wallet_address,
    strategy_category,    // "dca" | "lp" | "vault" | "trading" | "custom"
    creation_timestamp,
    parent_golem_id.unwrap_or(bytes32(0)),  // Lineage bleed-through
));
```

The seed is permanent. The Golem's appearance evolves, but its genetic foundation never changes. Two Golems with different seeds will almost certainly have different visual identities. The `parent_golem_id` input means successors inherit visual traits from their predecessor -- lineage is visible.

### Base Forms (32 templates x 4 archetypes)

| Archetype | Forms | Strategy Affinity | Visual Language |
|-----------|-------|------------------|----------------|
| **Elemental** | 8 | DCA, simple | Flame, water, crystal, wind, earth, lightning, void, light |
| **Construct** | 8 | Vault, LP | Golem (clay), automaton (gears), sentinel (armor), weaver (threads) |
| **Creature** | 8 | Trading, aggressive | Serpent, raptor, wolf, octopus, phoenix, chimera, drake, moth |
| **Abstract** | 8 | Custom, observatory | Fractal, orbit, tesseract, wave, helix, prism, sigil, eye |

Strategy category biases selection (60% matching archetype) but doesn't determine it. A DCA strategy could spawn a Creature if the seed's entropy leads there.

### 7-Layer Composition

```
Layer 7: Aura / Particle Effects    -- dynamic (phase, emotion)
Layer 6: Accessories                -- semi-static (achievements unlock new ones)
Layer 5: Markings / Patterns        -- static (personality from seed)
Layer 4: Eyes / Expression          -- dynamic (PAD -> 8 octant expressions)
Layer 3: Body Details               -- evolves with age/experience
Layer 2: Body Shape                 -- static (base form archetype)
Layer 1: Shadow / Ground            -- dynamic (regime -> atmospheric tint)
```

The PAD model from the Daimon (see `prd2/03-daimon/`) drives Layer 4 (expression) and Layer 7 (aura). A cautious Golem in a volatile regime looks different from a confident Golem in a trending market -- the sprite IS the emotional state.

### Resolution Tiers

| Level | Grid | Terminal Cells | Use Case |
|-------|------|---------------|----------|
| Tiny | 8x8 | 8x4 cells | List items, clade overview |
| Medium | 16x16 | 16x8 cells | Sidebar portrait |
| Large | 32x32 | 32x16 cells | Home screen main view |
| XLarge | 64x64 | 64x32 cells | Birth/death cinematic |

Rendering uses half-block characters (upper half block, lower half block, full block) for 2x vertical resolution and true-color per-cell coloring. On Kitty/iTerm2 terminals with graphics protocol support, actual bitmap sprites render at full pixel resolution.

### Age-Based Evolution

| Stage | Trigger | Visual Changes |
|-------|---------|---------------|
| Newborn | Tick 0-100 | Soft edges, simple features, large eyes |
| Young | Tick 100-500 | Sharper definition, markings appear |
| Mature | Tick 500-2000 | Full detail, complex expression range |
| Elder | Tick 2000-5000 | Weathered texture, wisdom marks (scars from losses) |
| Ancient | Tick 5000+ | Transcendent glow, simplified form, particle halo |

These map to the Golem lifecycle phases in `prd2/01-golem/11-lifecycle.md`. Age is visible. An Ancient Golem looks categorically different from a Newborn.

### Learning-Based Mutations

| Event | Mutation |
|-------|---------|
| First insight generated | A marking appears |
| PLAYBOOK reaches 10 heuristics | Accessory manifests (crown, wings, tools) |
| Survived regime change | Scar pattern (proud, not damaged) |
| Bloodstain received | Dark ripple marking at the point of impact |
| 100 successful trades | Luminosity increases permanently |

The Golem's body is its resume. Every meaningful event leaves a visible mark.

### Death Sprites

When a Golem enters Terminal phase (see `prd2/02-mortality/06-thanatopsis.md`), the sprite begins to dissolve: edges soften, colors desaturate, particle effects reverse (falling inward). At death, the sprite fractures into ascending particles over 3 seconds, leaving a faded afterimage that persists in the graveyard as a "tombstone sprite" -- the 8x8 Tiny resolution, permanently desaturated, with a subtle static animation.

---

## 4. The Screen System (11 Views)

### Persistent Chrome

```
+-- Header ---------------------------------------------------------------+
| [Ember-7f3a] > Heartbeat #4217 | ETH $3,502 | Gas 0.8 | ####-          |
+-- Sidebar --+-- Main Content -------------------------------------------|
|             |                                                            |
|  [H]ome     |  (current screen -- depends on active selection)           |
|  [B]eat     |                                                            |
|  [M]ind     |                                                            |
|  [D]reams   |                                                            |
|  [F]eel     |                                                            |
|  [V]ault    |                                                            |
|  [C]lade    |                                                            |
|  [O]racle   |                                                            |
|  [S]tyx     |                                                            |
|  [W]orld    |                                                            |
|  [X] Market |                                                            |
|  ---------- |                                                            |
|  [?] Help   |                                                            |
|  [,] Config |                                                            |
|  [Q]uit     |                                                            |
+-------------+------------------------------------------------------------+
| Stable # 62% | P:0.3 A:0.1 D:0.4 | $142.50 | 23d remaining             |
+-------------------------------------------------------------------------|
```

**Header**: Golem name (seed-derived), tick number, key market metrics, vitality gauge.
**Sidebar**: vim-style single-key navigation. Unread indicators for screens with new events.
**Status bar**: Phase, PAD summary, credit balance, projected lifespan.

### The 11 Screens

| Key | Screen | What It Shows |
|-----|--------|-------------|
| **H** | Home | Large animated sprite, key stats, scrolling event log |
| **B** | Beat | Heartbeat pipeline: PE vs threshold sparkline, tier distribution, cost ticker, last deliberation |
| **M** | Mind | Grimoire: entry histogram, confidence heatmap (categories x regimes), Curator stats, top entries, causal graph mini-view |
| **D** | Dreams | Dream viewer: phase indicator with "breathing" overlay, replay/counterfactual/trajectory logs, consolidation results |
| **F** | Feel | Daimon: PAD timeline (3 overlaid sparklines), emotion distribution donut, somatic markers log, landscape valence meter |
| **V** | Vault | Portfolio: positions table (color-coded health), PnL sparkline with regime markers, recent trades, risk summary |
| **C** | Clade | Coordination: pheromone field heatmap (animated), clade sync indicator, bloodstain feed, causal federation |
| **O** | Oracle | Knowledge retrieval browser: search Grimoire + Styx, view entry details, provenance chain |
| **S** | Styx | Mortality + ecosystem: three clock gauges, phase timeline, lineage tree, graveyard, ecosystem pulse banner |
| **W** | World | Multiplayer: all visible Golems (alive + dead), bloodstain density map, regime weather overlay, death study mode |
| **X** | Market | Marketplace: browseable listings, preview panes, purchase flow, sell flow, revenue tracking |

### Responsive Layout

| Terminal Width | Mode | Arrangement |
|---------------|------|-------------|
| < 80 cols | Compact | Single pane, tab-switch |
| 80-119 | Standard | 2-column: sidebar + main |
| 120-159 | Wide | 3-column: sidebar + main + detail |
| 160-199 | Wide+ | 3-column with wider panes |
| 200+ | Ultra | 3-column + detail sidebar |

> **Canonical breakpoint source:** `prd2/18-interfaces/19-spatial-grammar.md` §B.

---

## 5. Custom Widgets

### Braille Sparkline

2x4 dots per cell -- 80 data points in a 40-column sparkline at dot-matrix resolution. Significantly higher resolution than ratatui's built-in Sparkline. Used on the Beat screen for PE vs threshold visualization and on the Feel screen for PAD timelines.

### Pheromone Heatmap

Grid of domains x regimes. Each cell background-colored by pheromone intensity (Viridis gradient). Cells pulse when new deposits arrive (brief brightness increase over 500ms), fade as pheromones evaporate. This is the Golem's "radar screen" on the Clade tab.

### Timeline Ribbon

Horizontal colored segments showing time spent in each phase/regime. The current phase is highlighted. Hovering (if terminal supports mouse) shows duration and transition cause.

### Creature Widget

Composites the 7-layer sprite with particle effects, renders at the appropriate resolution tier, and handles animation blending between expression states (500ms interpolation).

---

## 6. Sound Design (Opt-In via `rodio`)

Sound is opt-in, disabled by default, and controlled via `~/.bardo/config.toml`. All sounds are short, non-intrusive, and designed for ambient awareness rather than attention-grabbing.

| Event | Sound | Duration |
|-------|-------|----------|
| T2 deliberation | Soft rising chime | 200ms |
| Trade success | Ascending arpeggio | 300ms |
| Trade failure | Low descending tone | 400ms |
| Dream start | Ambient pad fade-in | 2s |
| Dream end | Clarity chime | 1s |
| Bloodstain received | Deep resonant bell | 500ms |
| Achievement (common) | Short fanfare | 500ms |
| Achievement (legendary) | Full fanfare | 1.5s |
| Terminal phase entered | Low sustained drone | 3s |
| Death | 1s silence, then single clear bell | 2s |

---

## 7. Returning to Dead Golems

If all Golems are dead, the terminal shows a **Styx memorial**: the dead Golem's tombstone sprite (desaturated, static artifacts), lifetime stats, cause of death, last words from the death testament, and Grimoire archive status. Options:

- **Summon a successor** (inherited Grimoire at x0.85 confidence; see `prd2/01-golem/09-inheritance.md`)
- **View full death reflection** (the complete death testament from `prd2/02-mortality/06-thanatopsis.md`)
- **Browse the Styx graveyard** (all dead Golems in your lineage)
- **Explore the World view** (read-only, no Golem required -- watch other Golems live and die)

The graveyard is not a failure screen. It's a history screen. Every dead Golem contributed knowledge that makes the next one better. The TUI makes that lineage visible and the succession one command away.

---

## References

- [REEVES-NASS-1996] Reeves, B. & Nass, C. *The Media Equation.* Cambridge, 1996. — *People treat computers as social actors; basis for sprite personality and emotional display.*
- [EYAL-2014] Eyal, N. *Hooked.* Portfolio, 2014. — *Habit formation model informing the onboarding and engagement loop design.*
- [REBELS-IN-THE-SKY] "Rebels in the Sky: a ratatui game." https://github.com/ricott1/rebels-in-the-sky — *Proof-of-concept for 60fps animated ratatui applications.*

---

*The terminal is not where you check on your Golem. It's where you watch it live -- being born, learning, feeling, dreaming, trading, dying, and leaving a legacy for the next generation.*
