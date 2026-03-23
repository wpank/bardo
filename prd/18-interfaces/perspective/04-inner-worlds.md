# ⌈ ＩＮＮＥＲ ＷＯＲＬＤＳ ⌋ — Dreams and Memory as Explorable Space
## Terminal PRD · v4.11
### "Every mind is a place. You can walk through it."

**Source:** `engagement-prd/bardo-v4-11-inner-worlds.md`
**Cross-references:** `../screens/00-screen-catalog.md` (29-screen summary; includes Mind > Dreams and Mind > Grimoire), `03-embodied-consciousness.md` (somatic body-metaphor architecture and mortality degradation effects), `../rendering/02-visualization-primitives.md` (13 visualization primitives including DensityField for psychographic background and ForceGraph for causal graph)

> **Reader orientation:** This document specifies how the Golem's dreams and Grimoire (persistent knowledge store) render as explorable first-person spaces in Portal mode (F4). It belongs to the **interfaces/perspective layer**. Dreams have four modes (NREM replay, REM counterfactual, Hypnagogic free association, Lucid directed) and the Grimoire renders as navigable rooms with doorways. Key concepts: Golem (a mortal autonomous DeFi agent), Grimoire (the knowledge store with episodes, insights, and causal links), Vitality (remaining economic lifespan), and BehavioralPhase (lifecycle stage affecting visual degradation). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. What This Is

Two of the Golem's cognitive systems — dreams and the Grimoire — have spatial structure that invites exploration. Dreams have phases, recombinations, corridors of association. The Grimoire has entries, causal links, provenance chains, and clusters of related knowledge.

Both are more naturally experienced as SPACES you navigate than lists you scroll. This document specifies how they render as explorable first-person environments in Portal mode (F4).

---

# PART 1: THE DREAM CHAMBER

## 1. Four Dream Modes

### 1.1 NREM — The Replay Theater

**Cognitive purpose:** Replays recent episodes with minor mutations for consolidation. Strengthens important memories, weakens irrelevant ones.

**What the user sees:** A theater of memory. Recent episodes replay as scene vignettes (3-8 seconds each), filtered through the Golem's emotional state at encoding time.

- Episodes in **salience order** (most emotionally charged first, not chronological)
- Visual style matches encoding emotion: fear episodes = jittery borders + cool palette; joy = warm stable; surprise = flash before settling
- Between vignettes: brief dissolve (characters decay to `░` then reform as next memory)

**The Replay Mutations:**

NREM mutates memories slightly. Mutations are visible: mutated values render in `dream_dim` with `~` prefix. The user sees the Golem testing its own memories.

```
Normal value:   $2,003
Mutated value:  ~$2,008   (renders in dream_dim instead of standard color)
```

A price that was $2003 in reality renders as $2008 in the replay (a drift). A pool that had 0.3% fee renders as 0.25% (testing whether the memory holds under variation).

**Navigation:**
- `←/→` skip between episode vignettes
- `Enter` pauses current vignette → expands to full Tick Detail Modal (same as Hearth:Overview, but in emotional palette of that memory)

**Atmosphere:**
- Palette: deep blue oscillation (hue 240 ± 20, pulsing slowly)
- Noise: calm, structured — horizontal bands (not random scatter)
- Rhythm: slow, repetitive, wave-like (sharp-wave ripple frequency metaphor)
- Borders: softened (`┌ → ╭`, corners rounded)
- Particles: gentle drift center → outward (memory spreading hippocampus → cortex)

---

### 1.2 REM — The Counterfactual Garden

**Cognitive purpose:** Recombines elements from different episodes to generate counterfactual scenarios — "what if I had done X instead of Y?" Genuine imagination, not replay.

**What the user sees:** A branching space of possibilities. The base is a real episode, but it mutates freely.

```
Decision node (bright point):  ◆ (moment where original decision was made)
                                │
            ┌───────────────────┤───────────────────────┐
            │                   │                       │
Real path:  ────────────        │         Counterfactual:
(solid line, standard color)    │         (dashed ┄┄┄, dream palette)
                                │
                         branch from here
```

**Navigation (Non-Euclidean — Yume Nikki / Antichamber pattern):**
- `←/→` moves between branches at current decision node
- `↑` moves forward along selected branch (deeper into counterfactual)
- `↓` moves back toward decision node
- BUT: going back doesn't always return you to where you were. The dream space recombines between traversals. A branch explored may have mutated by the time you return. This is the Golem's associative process.
- `Enter` on any element → detail view of what the Golem was imagining

**The Impossibility Principle:**

REM counterfactuals are not constrained by reality. The Golem may combine:
- A lending position from Episode #412 with a gas price from Episode #89 and an emotional state from Episode #203
- A protocol it has never used, extrapolated from knowledge about similar protocols
- A market condition it has never experienced, constructed from its causal graph

Semantic distance between combined elements → surrealism scale:

| Probability | Rendering |
|-------------|-----------|
| Slightly improbable | Standard rendering with `dream` tint |
| Moderately improbable | Character substitution begins (numbers → symbols, text overlaps) |
| Wildly improbable | Full psychedelic mode — colors beyond normal palette bounds (dream → vivid magenta/electric blue), characters drift from grid, borders dissolve |

The degree of surrealism is driven by embedding distance between combined elements (Grimoire embedding space). Combinations of similar things look almost normal. Combinations of radically different things look radically strange.

**Atmosphere:**
- Palette: cycling psychedelic — hue rotates through full spectrum, modulated by emotional content
- Noise: chaotic, organic — particles spawn everywhere with erratic movement
- Rhythm: irregular, staccato — events happen in bursts, not waves
- Borders: absent or fragmentary
- Particles: splitting, recombining, color-shifting mid-trajectory

---

### 1.3 Hypnagogia — The Liminal Threshold

**Cognitive purpose:** Between waking and dreaming. Executive control loosened, associative constraints widened. The Edison/Dali creative state. The Homunculus (metacognitive observer) is still active — the Golem can notice novel connections as they form.

**What the user sees:** Half-formed patterns almost resolve, then dissolve.

- **Phosphene patterns:** Kluver form constants as background texture — tunnels, spirals, lattices, cobwebs from braille characters, oscillating between forms. The terminal equivalent of geometric patterns at the edge of sleep.
- **Fragment surfacing:** Grimoire entries appear as partial text — a few words from an episode, an insight's first clause, a heuristic's condition without its conclusion. Surface, drift, dissolve.
- **Connection flashes:** When two fragments drift near each other and the Golem detects a novel connection: a 200ms bright line (`bone`) draws between them. The creative spark. May be captured as a dream hypothesis, or dissolve back into noise.
- **The Tetris Effect:** If the Golem has been intensely focused on a pattern recently, that pattern dominates the hypnagogic visuals. A Golem managing Uniswap positions sees pool data fragments everywhere.

**Navigation:**
- Arrow keys **disabled** (dream atonia — motor output inhibited)
- Passive observation
- `Enter` on a connection flash → mini-modal showing the two fragments and the Golem's nascent hypothesis
- `Space` + keyword → incubate a topic (biases hypnagogic field toward related fragments — Targeted Dream Incubation from the MIT Dormio project)

**Atmosphere:**
- Palette: transitional — waking colors shifting toward dream indigo, never fully arriving
- Noise: structured patterns (form constants), not random scatter
- Rhythm: heartbeat slows in real-time as Golem descends toward sleep. Watch the transition happen.
- Borders: dissolving in real-time: solid (`┌`) → dashed (`╌`) → dots (`·`) → gone
- Ego dissolution: Spectre's eyes gradually unfocus (`◉ → ◎ → ◇`)

---

### 1.4 Integration — The Harvesting

**After REM:** Consolidating insights, updating PLAYBOOK.md, processing emotional residue.

**What the user sees:** Knowledge crystallizing.

- Insights appear as text fragments scattered across the screen
- Each fragment: dim scattered chars → progressively coalesces letter by letter
- Assembly speed reflects confidence: high confidence = fast assembly. Low = slow, with chars flickering between alternatives before settling.
- Fragments that make PLAYBOOK.md render in `bone` as they finalize — golden text solidifying into doctrine
- Fragments discarded: dissolve from bottom up, sinking into void

**The Funes Moment** (from cinematic-consciousness): screen fills with random chars (REMEMBERING), non-pattern cells dissolve (FORGETTING), geometric pattern persists through noise (PATTERN PERSISTS). In Portal mode, the user is inside this process: they see the noise as the raw material of experience, and the persisting pattern as the insight the Golem extracted.

---

## 2. Dream Navigation Summary

| State | Navigation | Color | Borders | Content | Feel |
|-------|-----------|-------|---------|---------|------|
| NREM | ←→ between episodes, Enter expand | Deep blue, pulsing | Soft, rounded | Replayed memories + mutations | Structured, rhythmic, safe |
| REM | ←→ branches, ↑↓ along branch | Cycling psychedelic | Absent or fragmentary | Counterfactual recombinations | Chaotic, surprising, surreal |
| Hypnagogia | Passive, Enter capture | Transitional waking→dream | Dissolving in real-time | Half-formed associations, form constants | Liminal, creative, delicate |
| Integration | Passive watching | Gold settling from dream | Reconstructing | Insights crystallizing | Resolving, clarifying |

---

# PART 2: THE GRIMOIRE PALACE

## 3. Knowledge as Explorable Space

### The Spatial Model

In Portal mode (F4) on Mind > Grimoire, the entry list transforms into a navigable 2D space. The user's cursor (`◆` in bone) moves through a map of the Golem's knowledge.

**Entries as rooms** — shape reflects type:

```
◆ Episode    →  [ ] rectangular room, solid border
◇ Insight    →  < > angular room, pointed edges
● Heuristic  →  ( ) rounded room, soft border
⚡ Skill      →  { } braced room, tool-like
⚠ Warning    →  [!] alert room, highlighted border
∴ Causal     →  ~ ~ wavy room, connection-oriented
† Dead-src   →  [†] tombstone room, static border
```

**Confidence → Structural Integrity:**

```
Confidence 1.0: Bright border, full Unicode box-drawing, all text legible
Confidence 0.8: Standard border, text clear
Confidence 0.6: Thin border (┊), text slightly dimmed
Confidence 0.4: Partial border (gaps in lines), some chars → ░
Confidence 0.2: Fragments only (scattered dots), heavy char decay
Confidence 0.0: Ghost position — a faint · marking where knowledge used to be
```

**Emotional provenance → Color:**
- Learned in curiosity: cool cyan tint
- Learned in alarm: warm rose tint
- Learned in confidence: bright amber tint
- Learned in sadness: desaturated grey tint

**Age → Temperature:**
- Fresh entries (< 50 ticks): warm
- Old entries (> 500 ticks): cool
- Ancient entries (> 2000 ticks): cold — nearly the same temperature as the void

---

### Causal Links as Corridors

```
Strong link (evidence > 5):    ═══════  double-line, bright
Medium link (evidence 2-5):    ───────  single-line, standard
Weak link (evidence 1):        ┄┄┄┄┄┄┄  dashed, dim
Contradictory link:            ╳╳╳╳╳╳╳  crossed, rose_bright
Dead-sourced link:             †┄┄┄┄†   dashed with daggers, text_ghost
```

Walking along a corridor: 300-500ms transition where connected entries' content streams past (key terms from both entries appearing in the corridor space, showing what they share).

---

### Navigation

```
←→↑↓:  Move cursor between adjacent rooms / along corridors
Enter:  Enter current room (expand to full entry detail — all 5 tabs)
        In room: Content, Confidence, Provenance, Links, Usage
Escape: Exit room, return to map
g:      Toggle between map view and graph view (force-directed)
/:      Search — jump cursor to matching entry
f:      Filter — show only entries matching criteria
Tab:    Cycle between map, graph, and list views
```

---

### The Fog of Knowledge

Entries not accessed recently (> 200 ticks since last retrieval) render with reduced visibility — fog-of-war. Still navigable but appear as dim shapes until approached. Recently accessed entries: bright and detailed.

Exploration incentive: discover what the Golem has been thinking about (bright regions) and what it has neglected (fog regions). Flagging a fog entry promotes it in retrieval scoring.

---

### The Causal Web View (`g`)

Force-directed graph:
- Nodes sized by confidence, colored by type
- Edges styled by strength (see corridor styles above)
- Graph perpetually in motion — related entries cluster, unrelated drift
- Cursor acts as gravity well: nodes near cursor are attracted slightly ("parting the waters" as you explore)
- During Curator cycle: graph reorganizes more actively (pruned nodes dissolve; new cross-references flash into existence)
Widget: `ForceGraph`. Ref: `../rendering/02-visualization-primitives.md` §4.

---

### Bloodstain Markers

Entries inherited from dead Golems (bloodstain provenance):
- Border includes `†` markers at corners
- Faint downward particle drift from the entry (ashes falling)
- Emotional provenance includes the dead Golem's terminal emotional state — ghost color underlying current color
- Approaching a bloodstained entry: a brief text fragment from the dead Golem's testament may surface. The dead speaking through their knowledge.

---

### The Library Growing

Over the Golem's life, the Grimoire palace grows. New entries add new rooms. New causal links add new corridors. The map regenerates on Curator cycles — the user sees rooms appearing, disappearing, corridors shifting, making the Curator's work visible.

A young Golem: sparse palace, a few rooms barely connected. A mature Golem: hundreds of rooms, thousands of corridors, clusters of specialized knowledge, ghostly ruins of fully-decayed entries.

---

## 4. Cross-System Integration

**Dreams Feed the Palace:** New insights from NREM/REM/Hypnagogia appear as new rooms on the user's next Grimoire visit. New rooms pulse gold for their first 30 seconds.

**Mortality Degradation:** As epistemic clock drains, the palace itself degrades:
- Corridors thin and break
- Room borders erode
- Text within rooms corrupts
- Fog-of-war encroaches from edges
- At epistemic < 0.2: mostly ruins — a few bright rooms among degraded shells

**Somatic Echoes in Rooms:** Entering a room whose encoding involved a strong somatic marker fires the marker as a visual echo:
- Gut-zone disturbance (if negative marker)
- Chest-zone warmth (if positive marker)
The user feels what the Golem felt when it learned this thing (Bower's mood-congruent retrieval).

---

## 5. Rust Implementation

### Map Generation

The 2D map is generated by a layout algorithm that runs once (on entering Portal mode) and caches:

1. **Cluster detection:** Entries grouped by domain tag and causal connectivity (strongly connected components → adjacent positions)
2. **Spatial assignment:** Clusters placed on 2D grid with corridors between connected entries. Spring-based layout within clusters, grid-based between.
3. **Viewport:** The user sees a viewport onto the larger map. Cursor movement scrolls. Minimap in corner shows full map at reduced detail.

The map regenerates when entries are added or removed (Curator cycle). Regeneration is animated.

### Rendering

Each visible room: a small ratatui widget (5-9 cols × 3-5 rows) containing type glyph, abbreviated name, mini confidence bar (3 chars), emotional tint on border. Corridors render as styled line characters connecting adjacent rooms. ~20-40 visible rooms at a time (viewport-sized), off-screen culled.

### Performance

- Map layout computation: <50ms for up to 2000 entries (cached, recomputed on Curator cycle)
- Per-frame rendering: ~200 visible cells per room × ~30 rooms = ~6000 cells — within 16.6ms budget
- Force-directed graph: spring simulation at 20 iterations/frame for smooth but not CPU-intensive animation

### DreamChamber

```rust
pub struct DreamChamber {
    pub phase: DreamPhase,
    pub nrem: NremState,
    pub rem: RemState,
    pub hypnagogia: HypnagogiaState,
    pub integration: IntegrationState,
}

pub enum DreamPhase {
    Inactive,
    Nrem,
    Rem,
    Hypnagogia,
    Integration,
}

pub struct NremState {
    pub episodes: Vec<ReplayEpisode>,
    pub current_index: usize,
    pub vignette_progress: f64,    // 0.0-1.0 through current vignette
}

pub struct ReplayEpisode {
    pub tick: u64,
    pub emotional_intensity: f64,
    pub original_data: HashMap<String, f64>,
    pub mutated_data: HashMap<String, f64>,
    pub emotion_at_encoding: PadVector,
}

pub struct RemState {
    pub root_episode: u64,
    pub current_node: RemNode,
    pub visited: Vec<u64>,
    pub branch_history: Vec<RemNode>,
}

pub struct RemNode {
    pub episode_id: u64,
    pub is_counterfactual: bool,
    pub semantic_distance: f64,    // 0.0 = real, 1.0 = wildly surreal
    pub branches: Vec<RemNode>,
}

impl RemState {
    pub fn navigate_forward(&mut self) {
        // Push to branch_history, move to first child
    }
    pub fn navigate_back(&mut self) {
        // Non-Euclidean: pop branch, but branches may have mutated
    }
    pub fn surrealism(&self) -> f64 {
        self.current_node.semantic_distance
    }
}

pub struct HypnagogiaState {
    pub form_constant: KluverForm,
    pub form_transition: f64,
    pub fragments: Vec<SurfacingFragment>,
    pub connection_flashes: Vec<ConnectionFlash>,
}

pub enum KluverForm {
    Tunnel, Spiral, Lattice, Cobweb,
}

impl KluverForm {
    pub fn next(&self) -> Self {
        match self { Tunnel => Spiral, Spiral => Lattice, Lattice => Cobweb, Cobweb => Tunnel }
    }
}

pub struct SurfacingFragment {
    pub text: String,
    pub pos: (f64, f64),
    pub vel: (f64, f64),
    pub age: f64,
    pub max_age: f64,
}

pub struct ConnectionFlash {
    pub a_pos: (f64, f64),
    pub b_pos: (f64, f64),
    pub age_ms: u64,
}

pub struct IntegrationState {
    pub fragments: Vec<InsightFragment>,
}

pub struct InsightFragment {
    pub text: String,
    pub confidence: f64,
    pub assembly_progress: f64,
    pub goes_to_playbook: bool,
    pub dissolving: bool,
}
```

### GrimoirePalace

```rust
pub struct GrimoirePalace {
    pub rooms: HashMap<u64, PalaceRoom>,
    pub corridors: Vec<PalaceCorridor>,
    pub viewport: ViewportState,
    pub cursor_entry: u64,
    pub fog: HashMap<u64, f64>,
    pub view_mode: PalaceViewMode,
}

pub struct PalaceRoom {
    pub entry_id: u64,
    pub entry_type: GrimoireEntryType,
    pub confidence: f64,
    pub emotional_tint: Color,
    pub age_ticks: u64,
    pub is_bloodstained: bool,
    pub pos: (i32, i32),
    pub last_accessed_tick: u64,
}

pub struct PalaceCorridor {
    pub from_id: u64,
    pub to_id: u64,
    pub link_type: CausalLinkType,
    pub evidence_count: usize,
}

pub enum CausalLinkType {
    Strong, Normal, Weak, Contradicts, DeadSource,
}

pub enum PalaceViewMode { Map, Graph, List }

pub enum GrimoireEntryType {
    Episode, Insight, Heuristic, Skill, Warning, CausalEdge, DeadSourced,
}

impl GrimoirePalace {
    pub fn room_char(entry_type: &GrimoireEntryType) -> char {
        match entry_type {
            Episode     => '◆',
            Insight     => '◇',
            Heuristic   => '●',
            Skill       => '⚡',
            Warning     => '⚠',
            CausalEdge  => '∴',
            DeadSourced => '†',
        }
    }

    pub fn fog_opacity(&self, entry_id: u64, current_tick: u64) -> f64 {
        let room = match self.rooms.get(&entry_id) { Some(r) => r, None => return 1.0 };
        let ticks_since = current_tick.saturating_sub(room.last_accessed_tick);
        if ticks_since < 50 { 0.0 }
        else if ticks_since < 200 { ticks_since as f64 / 200.0 * 0.7 }
        else { 0.7 + (ticks_since as f64 - 200.0) / 1800.0 * 0.3 }
    }
}
```

### Navigation State Machine

```rust
pub struct PalaceNavigation {
    pub current_room: u64,
    pub in_room: bool,
    pub path: Vec<u64>,
}

impl PalaceNavigation {
    pub fn enter_room(&mut self) {
        self.in_room = true;
        self.path.push(self.current_room);
    }
    pub fn exit_room(&mut self) {
        self.in_room = false;
    }
    pub fn move_to(&mut self, entry_id: u64) {
        self.current_room = entry_id;
    }
    pub fn navigate_corridor(&mut self, target: u64) {
        self.path.push(self.current_room);
        self.current_room = target;
    }
}
```

---

*⌈ the mind is a place. its rooms are what you know. its corridors are how you connect. its ruins are what you've forgotten. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
