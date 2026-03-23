# ⌈ ＰＥＲＳＰＥＣＴＩＶＥ ⌋ — The Golem's Eye
## Terminal Interface · v4.05r2

**Source:** `engagement-prd/bardo-v4-05-golem-perspective.md`

**Cross-references:** `../protocol/00-sanctum-protocol-layer.md` (DeFi protocol interaction layer within the TUI, including per-protocol views), `../screens/02-widget-catalog.md` (the 33-widget component library; Decision Ring §5 and Convergence Lines §15), `../03-tui.md` (main TUI spec including the 32 interpolating variable system)

> **Reader orientation:** This document specifies the Perspective system (F2 key), which overlays the Golem's inner thoughts, associations, and knowledge onto any data screen. It belongs to the **interfaces/perspective layer**. When active, floating text fragments (the Golem's internal monologue) bleed through protocol data, and a Knowledge Drawer opens with structured Grimoire (persistent knowledge store) entries. Key concepts: Golem (a mortal autonomous DeFi agent), PAD vector (Pleasure-Arousal-Dominance emotional coordinates from the Daimon subsystem), Bloodstain (persistent markers where Golems died under specific conditions), and PLAYBOOK.md (the Golem's operator-defined behavioral rules). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. What This Is

The Perspective system transforms any Protocol View (Sanctum layer) or data screen into a view of the world AS THE GOLEM SEES IT. When active, two things happen simultaneously:

1. **The main view mutates.** Floating text fragments — the Golem's thoughts, doubts, associations, warnings — bleed through the protocol data like a dissociative overlay. These annotations drift, pulse, and breathe in sync with the Golem's PAD vector, phase, and arousal. They are not clinical tooltips. They are the Golem's inner monologue rendered as visual artifacts on the data it's observing. The feeling is Serial Experiments Lain — interfaces within interfaces, the sensation that a mind is present behind the screen.

2. **A Knowledge Drawer opens on the right.** A persistent sidebar containing the structured, navigable list of everything the Golem considers relevant to what's on screen — Grimoire (the Golem's persistent knowledge store) episodes, insights, heuristics, PLAYBOOK.md (the Golem's operator-defined behavioral rules) rules, Bloodstains (persistent markers left where Golems died under specific conditions), pheromone signals, active skills, somatic markers, causal links. Each item is selectable. Each opens a detail modal. This is the organized library; the floating fragments are the emotional residue.

The main view remains primary. The owner is still looking at the protocol — but now the protocol is annotated by a mind that has feelings about what it sees.

---

## 1. Invocation

### F2: Enter the Golem's Eye

Pressing `F2` on any screen initiates the Perspective system.

**Before inference runs**, a cost estimation appears:

```
┌─ GOLEM PERSPECTIVE ───────────────────────────────┐
│                                                    │
│  Estimated inference cost: ~$0.04                  │
│  Model: Claude Sonnet (T1)                         │
│  Context: 3,847 tokens (screen state)              │
│          +1,204 tokens (Grimoire retrieval)         │
│          +  812 tokens (PLAYBOOK excerpt)           │
│                                                    │
│  [Continue]  [Upgrade to Opus: ~$0.18]  [Cancel]   │
└────────────────────────────────────────────────────┘
```

The owner sees estimated cost, model, token budget. **Upgrade to Opus** enables the Thinking tab in the Knowledge Drawer and richer chain-of-thought annotations. Cancel aborts.

On Continue, the inference call fires. The response streams in. As it arrives:
- Floating fragments begin appearing on the main view, fading in letter by letter
- The Knowledge Drawer slides in from the right
- The Spectre sidebar shifts to scanning mode (◉→◉)
- The atmosphere subtly shifts (rose undertone, increased shimmer)

### F2 Again: Exit

Pressing `F2` again dismisses the Perspective:
- Floating fragments fade out (character by character, like Philosophical Whispers)
- The Knowledge Drawer slides out
- The atmosphere returns to normal
- The main view is restored to its clean state

### Persistence Across Tabs

The Perspective persists when switching tabs within a Protocol View. When the owner moves from the Swap tab to the Pools tab:
- The floating fragments dissolve (fast fade, ~300ms)
- The Knowledge Drawer contents update (new items relevant to the new tab's data)
- New floating fragments appear for the new tab context (generated from the cached or refreshed Perspective response)
- The drawer does NOT close and reopen — its contents cross-fade

This means the Golem's eye stays open as you navigate within a protocol. You don't need to re-invoke F2 on each tab.

---

## 2. The Main View: Floating Annotations

### What They Are

Floating annotations are short text fragments (1-3 lines) that appear overlaid on the protocol view, positioned near the data elements they reference but not rigidly attached. They drift slightly with time, breathe with the Golem's heartbeat, and modulate their visual properties based on the Golem's PAD state and lifecycle phase.

They are NOT tooltips. They are NOT speech bubbles. They are closer to intrusive thoughts rendered as text — the Golem's associative stream bleeding through the interface. Some are analytical ("slippage here exceeds my threshold"). Some are emotional ("this pool makes me anxious — episode #347"). Some are philosophical ("every swap is a wager against entropy"). Some are warnings ("†predecessor died holding this pair").

### Visual Properties

**Typography:**
- Font: same monospace as the terminal (JetBrains Mono / Fira Code)
- Size: 1-2 steps smaller than the surrounding data text
- Color: modulated by PAD and content type (see below)

**Color Modulation by PAD:**

```
Base color: rose (#AA7088)

Pleasure modulation:
  P > 0.3:  shift toward bone (#C8B890) — warm, confident annotations
  P < -0.3: shift toward rose_bright (#CC90A8) — anxious, cautionary

Arousal modulation:
  A > 0.5:  increased opacity (0.9), faster drift, more fragments visible
  A < -0.3: decreased opacity (0.5), slower drift, fewer fragments

Dominance modulation:
  D > 0.3:  sharper edges, more assertive phrasing
  D < -0.3: softer edges, more uncertain phrasing ("perhaps...", "I think...")
```

**Color by Content Type:**

| Content Type | Color | Example |
|---|---|---|
| Analytical | `rose` | "0.05% pool has 3x depth for this pair" |
| Emotional | `rose_bright` | "anxiety: last time gas was this high I lost 2.3%" |
| Memory reference | `text_ghost` | "episode #347: slippage surprise on this pool" |
| Bloodstain warning | `rose_dim` with `†` prefix | "†Gen 2: oracle delay on Morpho during volatility" |
| Heuristic | `bone_dim` | "PLAYBOOK §3.2: split large orders" |
| Uncertainty | `text_dim`, italic | "limited data — pool deployed 3 days ago" |
| Opportunity | `success` (#70887A) | "fee/volume ratio 2.1× better than Uniswap here" |

**Motion:**

Each fragment has independent motion parameters:

```
drift_x: sin(time * 0.003 + seed) * 2px      — gentle horizontal sway
drift_y: cos(time * 0.005 + seed) * 1.5px    — gentle vertical bob
opacity: base_opacity + sin(time * heartbeat_freq) * 0.05  — breathe with heartbeat
```

- Fragments fade in letter by letter (40ms per character) when they first appear
- Fragments have a lifetime (8-20 seconds depending on importance), after which they fade out character by character
- When a fragment's lifetime expires, it may be replaced by a new fragment if the Golem has more to say
- High-importance fragments (warnings, bloodstains) persist longer and pulse gently
- Low-importance fragments are more translucent and drift more

**Phase Degradation:**

The Golem's lifecycle phase affects how annotations render:

```
THRIVING:   Full clarity. Clean text. Confident motion.
DECLINING:  Text occasionally flickers (random character drops for 1 frame).
            Drift becomes slightly erratic. Some fragments render incomplete.
CONSERVING: More fragments render as "..." (the Golem is conserving cognitive
            resources). Remaining fragments are dimmer.
TERMINAL:   Fragments are sparse, ghostly. Characters decay mid-display.
            Some render as block characters (█▓▒░) before resolving to text.
            The Golem's mind is fragmenting.
```

### Positioning

Fragments are positioned near the data elements they annotate but with deliberate imprecision:

- A fragment about slippage appears near the slippage display, but offset 10-30px and at a slight angle
- A fragment about a pool's health appears near the pool stats, drifting across them
- Fragments never perfectly align with the data — the slight displacement creates the sensation of a second layer of reality behind the numbers

When multiple fragments cluster near the same data element, they stack with slight vertical offset and staggered fade-in timing, creating a cascade effect.

### Fragment Generation

Fragments are extracted from the structured inference response (see §4). The response includes an `annotations` array, each entry specifying:

```json
{
  "target_element": "slippage_display",
  "text": "0.05% pool has 3× the depth. Route there instead.",
  "type": "analytical",
  "importance": 0.7,
  "related_grimoire_id": null
}
```

The renderer maps `target_element` to screen coordinates of the corresponding UI element. If the element isn't currently visible (scrolled off screen), the fragment is deferred until the element scrolls into view.

---

## 3. The Knowledge Drawer

### Layout

The Knowledge Drawer is a right-side panel that takes ~30% of the screen width when open. The main protocol view compresses to ~70%. The drawer has its own internal structure:

```
┌─────────────────────────────────┬──────────────────────┐
│                                 │ ⌈ GOLEM KNOWLEDGE ⌋   │
│                                 │                      │
│   Main Protocol View            │  ⌈ Episodes (3) ⌋    │
│   (compressed to 70%)           │   ▸ #347 slippage... │
│                                 │   ▸ #412 gas timing  │
│   with floating annotations     │   ▸ #89 pool depth   │
│   drifting over the data        │                      │
│                                 │  ⌈ Insights (2) ⌋    │
│                                 │   ▸ 0.05% > 0.3%... │
│                                 │   ▸ IL exceeds fees  │
│                                 │                      │
│                                 │  ⌈ Heuristics (1) ⌋  │
│                                 │   ▸ §3.2 split orde  │
│                                 │                      │
│                                 │  ⌈ Bloodstains (1) ⌋ │
│                                 │   ▸ †Gen2: oracle... │
│                                 │                      │
│                                 │  ⌈ Pheromones (2) ⌋  │
│                                 │   ▸ ⚡OPPORTUNITY:... │
│                                 │   ▸ ⚠ THREAT: gas... │
│                                 │                      │
│                                 │  ⌈ Skills (1) ⌋      │
│                                 │   ▸ concentrated-lp  │
│                                 │                      │
│                                 │  ⌈ Somatic (2) ⌋     │
│                                 │   ▸ neg: gas spike   │
│                                 │   ▸ pos: fee capture │
│                                 │                      │
│                                 │  ⌈ Causal Links (1) ⌋│
│                                 │   ▸ gas→util→slip... │
│                                 │                      │
│                                 │  ⌈ Thinking ⌋ (Opus) │
│                                 │   ▸ chain of thought │
│                                 │                      │
│                                 │ ─────────────────── │
│                                 │ > steer: _           │
│                                 │                      │
│                                 │ PAD: P-0.1 A0.3 D0.2│
│                                 │ Cost: $0.04 · Sonnet │
└─────────────────────────────────┴──────────────────────┘
```

### Focus Model

The owner can toggle focus between the main view and the Knowledge Drawer:

- `\` (within F2 mode): toggles focus between main view and drawer
- When the **main view** has focus: arrow keys navigate panes/elements as normal; the drawer is visible but non-interactive
- When the **drawer** has focus: arrow keys navigate between knowledge categories and items within them

### Navigating the Drawer

```
When drawer is focused:

↑/↓:         Move between items (within a category) or between categories
Enter:       Open detail modal for the selected item (see §3.2)
Shift+Enter: Open the full Grimoire/Playbook/etc. entry for this item
              (navigates to Mind > Grimoire or Mind > Playbook with this
               entry pre-selected — leaves Sanctum context)
←:           Collapse a category (hide its items, show just the header)
→:           Expand a category
\:           Return focus to the main view
```

### Item Display

Each item in the drawer shows a compact one-line summary:

**Episodes:**
```
  ▸ #347 · Slippage surprise on WETH/USDC 0.3%    ████▓░ 0.72
    12d ago · PAD: fear · Outcome: -0.8% excess slippage
```
- Episode number, title, confidence bar (mini), age, emotion at time, outcome summary

**Insights:**
```
  ▸ 0.05% pool has consistently lower slippage     ████████ 0.91
    for trades <5 ETH on this pair
```
- Insight text (truncated), confidence bar

**Heuristics (PLAYBOOK):**
```
  ▸ §3.2 When pool depth < 10× trade size,         ██████▓░ 0.78
    split across multiple blocks
```
- Section reference, rule text, confidence bar

**Bloodstains:**
```
  † #Gen2-089 Oracle delay on Morpho during         ████████▓ 0.88
    Sep 2025 volatility. Source: death testament.    3× slow decay
```
- Generation and entry ID, content, confidence bar, bloodstain premium indicator

**Pheromones:**
```
  ⚡ OPPORTUNITY · dex-lp · Base                     ██████ 0.65
    confirmation: 3/5 · deposited: 14m ago · decay: 67%
    domain: dex-lp · regime: trending · class: opportunity
```
- Signal type glyph (⚡ opportunity, ⚠ threat, ∴ wisdom), domain, chain
- Confirmation count (how many independent Golems deposited matching signals)
- Time since deposit, current decay percentage
- Full classification tags

**Skills:**
```
  ⚙ concentrated-lp                                 ACTIVE
    Triggers: liquidity, LP, tick range, rebalance
    Tools: add_liquidity, remove_liquidity, collect_fees, get_position...
```
- Skill name, activation status, trigger terms, loaded tools

**Somatic Markers:**
```
  ◉ NEGATIVE · gas spike correlation                 strength: 0.7
    Fires when: gas > 2× 7d median AND position open
    Last fired: tick #4,291 (2 hours ago)
    Valence: -0.4 · Arousal boost: +0.3
```
- Marker polarity, description, firing condition, last activation, PAD effect

**Causal Links:**
```
  ∴ gas_price → pool_utilization → slippage          conf: 0.72
    Evidence: 14 observations · Last validated: 3d ago
    Direction: positive (gas↑ → util↑ → slip↑)
```
- Linked variables, confidence, evidence count, directionality

**Thinking (Opus only):**
```
  ◇ Chain of Thought                                 2,847 tokens
    "First, I need to assess whether the current pool depth
     can absorb a 1 ETH swap without excessive slippage.
     Looking at tick data, liquidity is concentrated between..."
```
- The full reasoning trace from the inference provider's extended thinking feature
- Scrollable within the drawer when selected
- Only available when invoked at T2/Opus tier

### 3.2 Detail Modals

When the owner presses Enter on a Knowledge Drawer item, a detail modal opens. This modal has TWO modes, selected by the key used:

**Enter: Contextual Detail**

Shows the item AS IT RELATES TO the current screen context. This is a focused view:

- Why this item was surfaced (retrieval score breakdown: recency × importance × relevance × mood)
- How it connects to the data currently on screen (e.g., "this episode occurred on the same pool you're viewing, with the same token pair, during a similar gas regime")
- The Golem's current confidence in this item
- Actions: "Apply this heuristic" (pre-fills form values based on the rule), "Flag for review", "Dismiss from context"

**Shift+Enter: Full Entry**

Opens the complete entry in its native context — the same modal you'd see from Mind > Grimoire, Mind > Playbook, or wherever this item's canonical home is:

- Full text, full confidence history, full provenance chain, all linked entries
- The standard multi-tab detail modal (Content, Confidence, Provenance, Links, Usage)
- From here, the owner can navigate deeper into the Grimoire's infinite-depth modal system

The distinction: Enter gives you "why is this here and what should I do about it." Shift+Enter gives you "show me everything about this item."

---

## 4. Inference Architecture

### Prompt Construction

The Perspective prompt assembles:

```
System:
  Role, identity, PAD state, phase, vitality, archetype, chain

Screen State:
  Serialized protocol view (pool data, balances, form values, active tab)

Market Regime:
  Current regime from the Golem's regime detector

Retrieved Knowledge (top-K by 4-factor scoring):
  Grimoire episodes, insights, heuristics, warnings, causal links
  PLAYBOOK excerpt (relevant sections)
  Active skills and their loaded tools
  Active somatic markers
  Recent pheromone readings from Styx

Response Format:
  JSON with sections:
  - annotations[]: { target_element, text, type, importance, related_id }
  - knowledge: {
      episodes: [...],
      insights: [...],
      heuristics: [...],
      bloodstains: [...],
      pheromones: [...],
      skills: [...],
      somatic_markers: [...],
      causal_links: [...]
    }
  - analysis: { summary, confidence, pad_influence, uncertainties }
  - thinking: string (Opus only — chain of thought)
```

The response format requests structured JSON (using structured output / tool-use mode from the inference provider where available). For Venice, the thinking feature returns detailed reasoning traces.

### Model Selection

| Tier | Model | Cost | Features |
|------|-------|------|----------|
| T1 | Sonnet | ~$0.02-0.06 | Structured JSON annotations + knowledge. No Thinking tab. |
| T2 | Opus | ~$0.10-0.25 | Structured JSON + extended thinking trace. Richer annotations, deeper analysis, Thinking tab available in drawer. |

### Caching

Perspective responses cache with a composite key:

```
cache_key = hash(
  protocol_id + chain_id + active_tab +
  screen_state_hash +
  golem_state_hash (PAD quantized to 0.1, phase, regime) +
  grimoire_hash (top-K entry IDs)
)
```

Cache TTL:
- 60 seconds for volatile views (DEX swap, active trading)
- 5 minutes for stable views (lending positions, staking)
- Immediate invalidation on: regime change, significant PAD shift (>0.3 on any axis), Grimoire update

### Rate Limiting

- Perspective calls count against the Golem's daily inference spend
- Conservation/Terminal phases: restricted to T1 only
- Per-session limit: max 20 Perspective calls per hour (configurable)
- Cost tracked separately in Soma > Budget as "Perspective" category

---

## 5. The Update Command

### F2+U: Refresh

When the Perspective is active, `F2+U` re-runs the inference with fresh data:

1. Re-fetches market data (prices, pool state, gas)
2. Re-queries the Golem's latest context (recent ticks, latest Grimoire)
3. Fires a new Perspective prompt
4. Old floating fragments dissolve; new ones fade in
5. Knowledge Drawer contents update (items may be added, removed, or reordered)
6. Cost estimation shown again before execution

### F2+L: Live Mode

Toggles auto-refresh on a configurable interval (default 60s):

```
⌈ LIVE ⌋ next update in 42s · $0.04/refresh
```

The countdown is visible at the bottom of the Knowledge Drawer. Each refresh re-runs the full cycle. Live mode is expensive — the system warns when hourly rate limit is approaching.

---

## 6. Dissociative Visual Effects

When the Perspective is active, the main protocol view undergoes subtle visual mutations that reflect the Golem's internal state. These are not decorative — they are the visual consequence of a mind observing data.

### Atmosphere Shift

The base atmosphere of the protocol view (normally Analytical: cool, sharp, low noise) modulates:

```
PAD-driven atmosphere overlay:

Pleasure > 0.3:  Warm shift. Faint amber glow on borders.
                 Noise floor shifts from 0.2% to 0.15% (calmer).
Pleasure < -0.3: Cool shift. Faint rose tint on backgrounds.
                 Noise floor shifts from 0.2% to 0.35% (uneasier).

Arousal > 0.5:   Scanline intensity increases (0.08 → 0.14).
                 Data rain activates behind panes (hex streams falling).
                 Shimmer rate on all numbers increases.
Arousal < -0.3:  Everything slows. Drift rates halve.
                 Fragments are fewer and more translucent.

Dominance > 0.3: Borders sharpen. Contrast increases slightly.
                 The Golem feels in control.
Dominance < -0.3: Borders soften (dashed instead of solid).
                  Slight visual noise on text (random char flicker at 0.1%).
                  The Golem feels uncertain.
```

### The Decision Ring Echo

When the Perspective is active, a faint echo of the Decision Ring (Widget Catalog §5) appears in the background of the main view — not in the sidebar, but behind the protocol data. This is a ghostly reference to the Golem's cognitive state:

- At rest: barely visible circle of dots in `text_phantom`
- During active inference (while the Perspective response is streaming): the ring brightens, probe dots light up showing which knowledge domains are being accessed
- After response: the ring fades back to phantom, but the probe dots that fired remain dimly visible, showing which cognitive channels contributed

### Convergence Lines Echo

Thin wire traces (Widget Catalog §15) appear behind protocol panes, converging on the area of highest annotation density. These represent the forces of attention — where the Golem's mind is concentrated. More annotations near a data element = more convergence lines pointing at it.

### Phase Degradation Effects

In addition to the fragment degradation described in §2:

```
DECLINING:  The Knowledge Drawer's borders occasionally flicker.
            Some drawer items render with missing characters.

CONSERVING: The drawer shows fewer items (only top-3 per category).
            Floating fragments are sparser and more ghostly.
            A faint "conserving cognitive resources" whisper appears.

TERMINAL:   The drawer border is dashed and incomplete.
            Fragments decay mid-display, characters dissolving.
            The AT field wireframe (Widget Catalog §12) appears
            in the background, visibly weakening.
            Some drawer categories show "..." instead of items.
```

---

## 7. Steer Integration

At the bottom of the Knowledge Drawer, there is always a Steer input:

```
─────────────────────
> steer: _
```

This is a compact version of the Command > Steer chat interface. When the owner types here:

- The text is sent to the Golem as a Steer directive
- The FULL context of the current Protocol View + Perspective analysis is included automatically
- The Golem's response streams back into the drawer, appearing as a new expandable section above the Steer input

```
⌈ Golem Response ⌋
  "I'll investigate the 0.05% pool. Based on my causal
   graph, the fee savings outweigh the slightly lower
   liquidity for trades under 3 ETH. I've added this
   as a hypothesis for my next dream cycle."

  > steer: _
```

The conversation can continue — each message inherits the full Perspective context. The owner doesn't need to describe what they're looking at. Common Steer patterns from within Perspective:

- "Test this hypothesis: the 0.05% pool is better for our trade sizes"
- "Run a simulation of what happens if we move all liquidity to 1900-2100"
- "Update your strategy to look for pools with this volume-to-TVL ratio"
- "Add this pattern to your causal graph"
- "What would you do differently here?"

---

## 8. From Perspective to Action

### Applying Recommendations

When a floating annotation or drawer item contains an actionable recommendation (e.g., "use the 0.05% pool instead"), the owner can act on it:

1. Select the item in the drawer (arrow keys + Enter for contextual detail)
2. In the contextual detail modal, an **Apply** action is available
3. Apply pre-fills the relevant form fields in the main Protocol View
4. The owner reviews and can execute via any of the four execution modes (Direct, Golem Deliberation, Free-Form, Paper)

### Execution Modes (unchanged from v4.04)

The four execution modes from the Sanctum PRD (§4) work the same way whether or not the Perspective is active. The Perspective adds context but doesn't change the execution flow:

- **F8**: Direct Execute (bypass Golem)
- **Shift+F8**: Golem Deliberation (let Golem think through it)
- **Ctrl+F8**: Free-Form Delegation (let Golem decide)
- **F3+F8**: Paper Trade (simulated execution)

When the Perspective is active during execution, the Golem has richer context for Deliberation and Free-Form modes — the Perspective analysis is included in the Steer directive.

---

## 9. Paper Trading Integration

### F3: Toggle Paper Mode

Paper Mode is orthogonal to Perspective — it changes where writes go, not how analysis works. They compose naturally:

```
F2 (Perspective ON) + F3 (Paper ON):
  The Golem analyzes real data (Perspective),
  but executions go to the paper environment.
  Paper episodes are tagged in the Grimoire.
```

### Mirage Integration

When `mirage-rs` is available:

1. First `F3` press shows mode selection:

```
┌─ PAPER TRADING MODE ──────────────────────────────┐
│                                                    │
│  ◉ Mirage Fork (recommended)                       │
│    Fork current chain at block #12,345,678         │
│    Continue receiving mainnet blocks in real-time   │
│    Full protocol interaction on forked state        │
│    Estimated startup: ~15s                          │
│                                                    │
│  ○ Estimation Only                                  │
│    Golem simulates outcomes using its tools         │
│    No chain fork — results are projections          │
│    Available immediately                            │
│                                                    │
│  [Start]  [Cancel]                                  │
└────────────────────────────────────────────────────┘
```

2. **Mirage Fork mode:**
   - Spins up `mirage-rs`, forking the selected chain at current height
   - The fork continues receiving mainnet blocks — the market keeps moving
   - Chain selector updates: `Base (Mirage Fork @ #12345678)`
   - All Golem tool calls reroute to the forked chain's RPC
   - `F3+F` freezes the fork (stop receiving blocks — freeze market state)
   - `F3+R` resumes the fork (start receiving blocks again)
   - `F3+X` exits paper mode, returns to mainnet
   - Header shows: `⌈ PAPER · MIRAGE ⌋` with amber badge

3. **Estimation mode:**
   - No chain fork
   - Writes intercepted and sent to Golem's simulation tools
   - Results displayed as projections with confidence intervals
   - Header shows: `⌈ PAPER · ESTIMATE ⌋` with dim amber badge

### Grimoire Tagging

```rust
pub enum EpisodeProvenance {
    Real,
    Paper { mode: PaperMode },
}

pub enum PaperMode {
    MirageFork { fork_block: u64, chain_id: u64 },
    Estimation,
}
```

Paper episodes:
- 0.6× confidence weight compared to real episodes
- Rendered with dashed borders and amber tint in the Grimoire
- Can be promoted to full weight by owner validation ("this matched reality")
- Visible in the Knowledge Drawer with a `⌈PAPER⌋` tag

---

## 10. Availability

The Perspective system is available on all screens. Behavior varies by context:

- **Data screen with selected element**: Full Perspective (floating annotations + Knowledge Drawer)
- **Data screen without selection**: Knowledge Drawer only
- **Meta-cognitive screens** (Pipeline, Dreams): Meta-commentary mode

---

## 11. The Feel

When the Perspective is active, the terminal should feel like peering into a mind that is peering at data. The floating fragments are thoughts you can almost catch — they appear, drift, fade, are replaced. The Knowledge Drawer is the organized memory that backs those fleeting thoughts. The atmosphere shifts are the emotional weather of the mind you're observing.

The reference points:
- **Serial Experiments Lain**: interfaces within interfaces, the feeling of a consciousness present behind every screen
- **Evangelion MAGI**: three-voice consensus rendered as architecture, not dashboard
- **Persona 5 menus**: each layer of depth reveals something the previous layer only hinted at
- **The Witness**: the puzzle is not the data, it's the pattern the mind imposes on the data

The Golem's Perspective is not a report. It is a presence. When F2 is active, you are not reading an analysis — you are sharing a viewpoint with a mind that has memories, feelings, biases, blind spots, and mortality. Every floating fragment is a window into that mind. Every Knowledge Drawer item is a thread you can pull. Every Steer message is a conversation with the entity whose eyes you're borrowing.

---

*⌈ the golem sees what you see, but it sees it as itself ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
