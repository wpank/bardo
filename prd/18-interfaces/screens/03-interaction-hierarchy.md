# ⌈ ＢＡＲＤＯ ＤＥＳＩＧＮ ＳＹＳＴＥＭ ⌋
## Interaction Hierarchy · v4.0
### Windows, Tabs, Panes, and the Infinite Depth

**Source:** `engagement-prd/bardo-v4-00-interaction-hierarchy.md`
**Cross-references:** `../19-spatial-grammar.md` (spatial layout grammar with zone architecture, depth planes, and Rust layout primitives), `00-screen-catalog.md` (29-screen summary across 6 windows with all tab/pane definitions), `../perspective/03-embodied-consciousness.md` (PAD-driven somatic modulation of transitions and zone behavior)

> **Reader orientation:** This document defines the navigation model for the Bardo terminal: 6 windows, 29 tabs, infinite-depth pane/modal drilling. It belongs to the **interfaces/screens layer** and specifies how keyboard input maps to focus, locking, drilling, and dismissal at every depth layer. Key concepts: Golem (a mortal autonomous DeFi agent), the Spectre (dot-cloud creature sprite visible in the sidebar across all screens), Nooscopy (Golem-initiated decision approval modal at Layer 6), and BehavioralPhase (lifecycle stage affecting atmosphere and transition character). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. Terminology

```
WINDOW    The outermost container. A major conceptual category.
          Navigate between windows with Tab / Shift-Tab.
          There are exactly 6 windows. Always visible in the top bar.

TAB       A sub-view within a window. Each window has 2-7 tabs.
          Navigate between tabs with number keys 0-9.
          Tabs appear below the window bar when that window is active.

SCREEN    What you see when a specific tab is selected. A layout of panes.

PANE      A bounded region within a screen. Contains data, widgets, or
          interactive elements. One pane has FOCUS at any time.
          Navigate between panes with arrow keys (↑↓←→).

FOCUS     The highlighted pane. Receives keyboard input. Shows available
          commands in the bottom bar. Bright border.

LOCKED    When you press Enter on a focused pane, focus LOCKS.
          Arrow keys scroll/select WITHIN the pane. Backspace unlocks.

MODAL     A floating overlay triggered by Enter on a selected item within
          a locked pane. Covers 40-80% of the screen. Double-line border.
          Has its own tabs, panes, elements — the full hierarchy repeats.
          Backspace closes. Modals can nest infinitely.

ELEMENT   An individual interactive item within a pane: a list row,
          a gauge, a chart, a text block. Enter on an element opens a modal.
```

### The Depth Stack

```
LAYER 0:  WINDOW      (Tab/Shift-Tab cycles)
LAYER 1:  TAB         (Number keys 0-9)
LAYER 2:  SCREEN      (the layout of panes)
LAYER 3:  PANE FOCUS  (arrow keys move focus)
LAYER 4:  PANE LOCKED (Enter locks; arrows scroll/select within)
LAYER 5:  ELEMENT     (Enter on element opens modal)
LAYER 6:  MODAL       (has its own tabs, panes, elements)
LAYER 7:  MODAL LOCKED
LAYER 8+: NESTED      (infinite depth — modals within modals)

Backspace: go UP one layer.
Esc: return to Layer 3 (pane focus) from anywhere.
```

---

## 1. The Six Windows

### HEARTH — The Presence
*"What is happening right now?"*
The Golem as a living thing. Ambient screen. First thing you see.

| # | Tab | Content |
|---|-----|---------|
| 1 | Overview | Spectre (dot-cloud creature sprite) + Heartbeat (periodic execution tick) log + Vitality (remaining economic lifespan) + mortality gauges + MAGI + PAD |
| 2 | Signals | 16-probe grid + market regime + signal constellation |
| 3 | Operations | Full tick log, cost accumulator, decision cache |
| 4 | Status | System diagnostics, uptime, connections, error log |

Atmosphere: warm, amber undertone, fragments active, noise floor 0.4%.

### MIND — The Cognition
*"What does the Golem know, and how does it think?"*

| # | Tab | Content |
|---|-----|---------|
| 1 | Pipeline | 9-step decision visualization, global workspace, AT field, psychographic |
| 2 | Grimoire | Knowledge library, confidence bars, provenance, causal graph view |
| 3 | Playbook | PLAYBOOK.md as living document with confidence sparklines |
| 4 | Dreams | Dream chamber (liminal palette during dream; journal otherwise) |
| 5 | Inference | Model routing, cost sparklines, cache performance, token flow |
| 6 | Chain Intelligence | On-chain analytics, whale tracking, liquidity flow maps, MEV exposure |
| 7 | Technical Analysis | Price action overlays, indicator suite, pattern recognition, regime detection |

Atmosphere: variable by tab (cool/analytical → deep rose → warm → liminal).

### SOMA — The Economy
*"What does the Golem own, and what does it cost?"*
Named for Huxley's drug of bearable existence — the substance that sustains and diminishes simultaneously.

| # | Tab | Content |
|---|-----|---------|
| 1 | Portfolio | Total NAV, allocation bar, position cards with PnL sparklines |
| 2 | Trades | Phosphor-decay trade log with emotion tags |
| 3 | Custody | Session keys, delegation tree, spend history |
| 4 | Bazaar | Knowledge marketplace, x402 micropayments |
| 5 | Sanctum | Protocol browser with chain filter, protocol listing, deep interaction views |
| 6 | Budget | Credit balance, burn rate, cost cap state, burn rate waveform |

Atmosphere: analytical, numbers-dense, FlashNumber throughout.

### WORLD — The Collective
*"What else exists, and how do they relate to me?"*

| # | Tab | Content |
|---|-----|---------|
| 1 | Solaris | Force-directed graph of all visible Golems, pheromone heatmap |
| 2 | Clade | Sibling peer tiles, knowledge sync flows, emotional contagion |
| 3 | Lethe | Anonymous knowledge fragments flowing as horizontal text |
| 4 | Bloodstains | Death-validated knowledge feed, sorted by current relevance |

Atmosphere: liminal, dream-adjacent palette, alien ocean register.

### FATE — The Lifecycle
*"How long will the Golem live, and what will it leave behind?"*

| # | Tab | Content |
|---|-----|---------|
| 1 | Mortality | Three clocks expanded + defense layers + projected survival |
| 2 | Lineage | Generational tree, tombstone sprites, inheritance arrows |
| 3 | Achievements | 87 achievements across 10 categories, progress arcs |
| 4 | Graveyard | All dead Golems, tombstone gallery, death recaps |

Atmosphere: heavy, spectral traces active, Styx river ambient.

### COMMAND — The Owner's Interface
*"What can I do, and how do I configure things?"*

| # | Tab | Content |
|---|-----|---------|
| 1 | Steer | Chat interface with streaming Golem responses |
| 2 | Config | Strategy parameters, Daimon settings, risk limits |
| 3 | Effects | Display settings (scanlines, noise, animation speed, glitch) |
| 4 | Hermes | Meta Hermes: aggregates state across all Golems |

Atmosphere: warm for Steer/Hermes, clinical for Config.

---

## 2. The Persistent Chrome

### Window Bar (top, row 0)

```
⌈ＨＥＡＲＴＨ⌋  ＭＩＮＤ  ＳＯＭＡ  ＷＯＲＬＤ  ＦＡＴＥ  ＣＯＭＭＡＮＤ
```
Active: `⌈ ⌋` frame in `rose`. Inactive: `text_dim`. Unread: `●` suffix.

### Tab Bar (top, row 1)

```
 1:Overview  2:Signals  3:Operations  4:Status
```
Active tab: `rose`. Number keys switch tabs.

### Golem Sidebar (left, cols 0 to sidebar_width-1)

Always visible, even in modals. The Golem is always present.

Single Golem: full Spectre + mini mortality gauges + phase indicator.

Multiple Golems: vertical stack of mini-Spectres (~4 rows each). Selected Golem has bright border. Others dimmer. `Ctrl+↑/↓` cycles.

On Golem cycle: 500ms smooth crossfade. All data on current screen lerps between old Golem's values and new Golem's values. Colors shift with PAD state. Not a snap cut — a blending of attention from one mind to another.

### Status Bar (bottom, row -2)

```
● THRIVING  T:004847  $47.23  HEARTH > Overview > Heartbeat Log [LOCKED]  ∿
```
- Phase indicator: `●` + name in phase color
- Tick counter: current tick (FlashNumber on each tick)
- Balance: credit balance (FlashNumber)
- Breadcrumb: full navigation path (Window > Tab > Pane > [LOCKED] > Element > [MODAL])
- Heartbeat: `∿` pulsing at arousal-modulated rate

### Command Bar (bottom, row -1)

```
Tab:window  1-5:tab  ←→:pane  Enter:lock  Backspace:back  F1:help  F2:golem  /:search
```
Shows top 5-8 contextual keys for current depth layer. Changes as you navigate deeper.

---

## 3. Focus and Lock System

### Pane Focus (Layer 3)

Arriving at a screen: one pane has default focus. Arrow keys move between panes.

Focused pane:
- Bright border (`border_active`)
- Available commands in command bar
- Receives shortcut keys (`f` for filter, `/` for search, `s` for sort)
- Slightly higher brightness than unfocused

Unfocused panes: dim borders, still live-updating, not interactive.

### Pane Lock (Layer 4)

**Enter** locks. Border doubles (`═║╔╗╚╝`). Now:
- Arrows scroll/select WITHIN the pane
- Letter keys become pane-specific commands
- Other panes dim to 70% brightness
- Command bar shows pane-internal commands

**Backspace** unlocks. Double border reverts to single.

### Element Selection (Layer 5)

Within a locked pane: selected element highlighted. Lists: `rose_deep` bg. Grids: bright border. Charts: tooltip at cursor.

**Enter** on element opens modal.

### Modals (Layer 6+)

Floating panel with double-line borders, 40-80% of screen, centered. Background dims to 40% brightness. Golem sidebar remains visible.

Modal frame:
- Double-line border in `border_active`
- Title bar at top
- Background dims behind

Modal content follows the same hierarchy: tabs, panes, elements, nested modals. Infinite depth.

**Backspace** closes modal. Previous context restores.

---

## 4. Key Reference

```
Tab / Shift-Tab     Cycle between the 6 windows
1-9                 Switch tabs within current window
←→↑↓               Move pane focus / scroll when locked
Enter               Lock pane / select element / open modal
Backspace           Unlock / close modal / go up one layer
Esc                 Return to Layer 3 (pane focus) from anywhere
Ctrl+↑ / Ctrl+↓    Cycle Golems (crossfade all data)
/                   Search (context-sensitive)
F1                  Full help for current context
?                   Available keys overlay
:                   Command palette (fuzzy search)
```

---

## 5. Modal Types

### Detail Modal
60–75% w, 70–85% h.
Full content + metadata grid + action buttons. Internal tabs for related views.

### Comparison Modal
80% w, 65% h.
Two columns. Left = option A. Right = option B. Differences highlighted.

### Timeline Modal
90% w, 40% h.
Horizontal timeline. Cursor-navigable. Details below cursor.

### Graph Modal
70% w, 70% h.
Interactive graph. Zoom/pan. Enter on node → nested modal.

### Editor Modal
50% w, 60% h.
Form-style. Parameter name, current value, slider/input, preview, confirm/cancel.

### Conversation Modal
Chat-style streaming. Inherits context of the element that triggered it.

---

## 6. Golem-Contextual Rendering

Everything on every screen depends on the selected Golem:
- All data reflects selected Golem
- Spectre appearance: density, eyes, lifecycle phase
- Color temperature: PAD state modulates atmosphere
- Noise floor: denser for crisis Golems
- Heartbeat rhythm: arousal-dependent speed
- Phase degradation: Declining Golem screens look visibly different
- Fragment pool: philosophical fragments contextualized to selected Golem's state

What stays the same: window/tab structure, Golem sidebar, command bar layout, persistent chrome structure.

### Global Exceptions

| View | Behavior |
|------|---------|
| World > Solaris | Shows ALL Golems. Selected highlighted with ring. |
| World > Clade | All Clade siblings. Flows visible between all members. |
| Soma > Bazaar | All marketplace listings. |
| Fate > Graveyard | All dead Golems across all lineages. |
| Command > Hermes | Aggregates across all Golems. |

---

## 7. Responsive Layout

| Terminal Width | Sidebar | Layout | Modal Size |
|---|---|---|---|
| Minimal (<80) | Hidden (toggle with `\`) | Single column, no split panes | Full width |
| Compact (80-119) | 6 cols, eyes only | Single column | 90% w |
| Standard (120-159) | 10 cols, mini Spectre | Two columns | 70% w |
| Wide (160-199) | 12 cols, full Spectre | Three columns | 60% w |
| Ultra (200+) | 14 cols, full + waveforms | Three cols + detail sidebar | 50% w |

---

## 8. Rust Implementation

### NavigationState

```rust
pub struct NavigationState {
    pub window: WindowId,
    pub tab: usize,
    pub focus_stack: FocusStack,    // from ../19-spatial-grammar.md
    pub pane_grid: PaneGrid,
    pub modal_layer: ModalLayer,
    pub selected_golem: usize,
    pub golem_transition: Option<GolemTransition>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WindowId {
    Hearth, Mind, Soma, World, Fate, Command,
}

pub struct GolemTransition {
    pub from: usize,
    pub to: usize,
    pub progress: f64,      // 0.0-1.0, lerps data display
    pub duration: f64,      // target ~0.5s
}
```

### Key Dispatch Table

```rust
pub fn dispatch_key(state: &mut NavigationState, key: KeyCode) {
    let depth = state.focus_stack.depth();
    match (depth, key) {
        // Layer 0-1: window/tab navigation
        (0..=1, KeyCode::Tab)       => state.next_window(),
        (0..=1, KeyCode::BackTab)   => state.prev_window(),
        (0..=1, KeyCode::Char(c)) if c.is_ascii_digit() => state.switch_tab(c as usize - '0' as usize),

        // Layer 3: pane focus
        (3, KeyCode::Up)    => state.pane_grid.focus_prev(),
        (3, KeyCode::Down)  => state.pane_grid.focus_next(),
        (3, KeyCode::Enter) => { state.pane_grid.lock(); state.focus_stack.push(FocusLayer::PaneLocked(state.pane_grid.focused)); }

        // Layer 4: pane locked (scrolling/selection within pane)
        (4, KeyCode::Up | KeyCode::Down) => state.scroll_focused_pane(key),
        (4, KeyCode::Enter)              => state.open_element_modal(),

        // Esc handling per layer
        (0..=3, KeyCode::Esc)     => {},  // no-op: already at or above pane focus
        (4, KeyCode::Esc)         => { state.pane_grid.unlock(); state.focus_stack.back(); }  // close pane lock → Layer 3
        (5.., KeyCode::Esc)       => {
            // Jump to Layer 3. If multiple modals open, confirm before collapsing.
            if state.modal_layer.depth() > 1 {
                state.confirm_collapse_all_modals();
            }
            state.focus_stack.escape_to_pane_focus();
            state.modal_layer.pop_all();
        }

        // Layer 6+: modal (Backspace = one level up)
        (6.., KeyCode::Backspace) => { state.modal_layer.pop(); state.focus_stack.back(); }

        // Universal
        (_, KeyCode::Backspace)  => { state.focus_stack.back(); }
        (_, KeyCode::F(1))       => state.show_help(),
        (_, KeyCode::Ctrl('c'))  => {},  // Cycle Golem handled separately via Ctrl+Up/Down
        _ => {}
    }
}
```

### FocusStack (from ../19-spatial-grammar.md)

See `../19-spatial-grammar.md` §H for full implementation. The `FocusStack` provides `breadcrumb()`, `depth()`, `back()`, and `escape_to_pane_focus()`.

---

## 9. The Persona 5 Principle

Each layer of depth reveals something the previous layer only hinted at:

- Window → THE FEELING (warm/cool/liminal register)
- Tab → THE CATEGORY (what kind of data)
- Screen → THE OVERVIEW (layout, relationships)
- Focused pane → THE DETAIL (numbers, trends)
- Locked pane → THE INTERACTION (sort, filter, navigate)
- Element → THE SPECIFICS (this one entry, this one trade)
- Modal → THE DEPTH (full history, provenance, cross-links)
- Nested modal → THE RABBIT HOLE (follow any thread anywhere)

Depth is discovery, not chore.

---

## 10. Command Palette

Activated by `:` from any layer. Appears at the bottom of the screen, vim-style, as a single-line input with fuzzy completion dropdown above it.

### Indexed Namespaces

The palette indexes three namespaces, searched simultaneously:

- **Screens** — every window/tab combination (e.g., "HEARTH Overview", "SOMA Sanctum")
- **Commands** — all available actions for the current context (e.g., "filter by chain", "sort by PnL")
- **Settings** — configuration keys (e.g., "scanline_intensity", "noise_floor")

### Direct Navigation

Shorthand prefixes jump directly to a window:

```
:h    → HEARTH
:m    → MIND
:s    → SOMA
:w    → WORLD
:f    → FATE
:c    → COMMAND
```

Append a tab number for direct tab access: `:m3` opens MIND > Playbook.

### Fuzzy Completion

Typing any substring matches against all three namespaces. Results ranked by: exact prefix > word boundary match > substring. Arrow keys navigate the dropdown, Enter selects, Esc dismisses.

### The `g` Key — Go To Related

At Layer 4+ (pane-locked or deeper), pressing `g` opens a "Go to related" contextual popup. This popup lists cross-references relevant to the currently focused element: related positions, linked knowledge entries, causal graph neighbors, related achievements. Each entry shows its destination window/tab. Enter on an entry navigates there directly, pushing the current location onto the focus stack so Backspace returns.

---

## 11. Multi-Golem Sidebar and Offline Mode

### Multi-Golem Sidebar

The sidebar adapts to the number of active Golems:

- **1 Golem:** Full Spectre with mini mortality gauges and phase indicator (current behavior).
- **2-4 Golems:** Sidebar expands to show a vertical stack of mini-Spectres (~4 rows each). Selected Golem has bright border, others dimmer. `Ctrl+Up/Down` cycles. Sidebar width grows by 2 cols to accommodate labels.
- **5+ Golems:** Mini-Spectres replaced by a dropdown selector. Shows selected Golem's Spectre in full, with a compact `[▾ 5 Golems]` selector above it. `Ctrl+Up/Down` still cycles; Enter on the selector opens a picker overlay listing all Golems with phase color indicators.

### Offline / Cached Mode

When the WebSocket connection drops:

- All data panes continue displaying last-known state.
- A staleness indicator appears in the status bar: `⚠ STALE 12s` with elapsed time since last update, colored amber, pulsing slowly.
- Panes that depend on live data show a dim `░` overlay in their top-right corner.
- The heartbeat `∿` in the status bar freezes and dims.
- Interactive commands that require connectivity (trading, steering, knowledge sync) are disabled with a `[OFFLINE]` badge in the command bar.
- On reconnection: staleness indicator clears, data panes flash briefly as they update, heartbeat resumes.

---

## 12. Progressive Disclosure

Windows and tabs do not all appear at first launch. They activate based on data availability and user progression:

- **HEARTH** is always visible. It is the entry point.
- **MIND** tabs 1-3 (Pipeline, Grimoire, Playbook) appear once the Golem has completed its first decision cycle. Dreams (tab 4) appears after the first sleep cycle. Inference (tab 5) appears once model routing data exists. Chain Intelligence (tab 6) and Technical Analysis (tab 7) appear when on-chain analytics and price feeds are connected.
- **SOMA** tabs activate as the Golem acquires assets: Portfolio appears with the first position, Trades with the first trade, Custody when session keys are created, Bazaar when knowledge marketplace is enabled, Sanctum when protocol interactions begin, Budget always visible (it tracks costs from tick 1).
- **WORLD** activates when the Golem detects peers. Solaris requires at least one other visible Golem. Clade requires sibling relationships. Lethe and Bloodstains require network participation.
- **FATE** activates after the Golem's first mortality calculation. Graveyard appears only when dead Golems exist in the lineage.
- **COMMAND** is always visible. The owner always has control.

Inactive windows appear in the window bar as ghosted text (`text_dim` at 30% opacity). Hovering or selecting shows a brief explanation of what triggers activation. Once a window or tab activates, it never deactivates (even if underlying data becomes empty, it shows an empty state rather than disappearing).

---

## 13. Unified Keybinding Reference

### Global Keys (Layers 0-3)

| Key | Action |
|-----|--------|
| Tab / Shift-Tab | Cycle between the 6 windows |
| 1-9 | Select tab within current window |
| Arrow keys | Move pane focus (Layer 3) / scroll content (Layer 4+) |
| Enter | Lock pane / select element / open modal |
| Backspace | Go up one layer (unlock, close modal, etc.) |
| Esc | Jump to Layer 3 (pane focus) from anywhere |
| Ctrl+Q | Quit application |
| `/` | Search (context-sensitive to current pane) |
| `?` | Available keys overlay for current context |
| `:` | Command palette (fuzzy search across screens, commands, settings) |
| `~` | Toggle reduced-information mode (hides secondary data, widens primary) |
| `-` | Minimize focused pane (collapses to single-line header, restores with `-` again) |

### Function Keys

| Key | Action | Scope |
|-----|--------|-------|
| F1 | Help for current context | All screens |
| F2 | Golem Perspective drawer | All screens |
| F3 | Comparison mode (split-view two items) | Where applicable |
| F5 | Force refresh all data | All screens |
| F8 | Execute transaction | Sanctum only |
| F10 | Debug overlay (tick timing, render stats) | Dev mode only |

### Pane-Locked Keys (Layer 4+)

| Key | Action |
|-----|--------|
| `g` | Go to related (contextual cross-reference popup) |
| `f` | Filter pane contents |
| `s` | Sort pane contents |
| Arrow keys | Scroll / select within pane |
| Enter | Select element / open modal |

### F2 Sub-Keys (Golem Perspective)

| Key | Action |
|-----|--------|
| `\` | Toggle perspective drawer open/closed |
| Arrow keys | Navigate within the perspective drawer |

### Removed / Relocated Bindings

These bindings from earlier drafts have been moved or removed:

| Old Binding | New Location | Reason |
|-------------|-------------|--------|
| F5 (self-portrait) | `:self-portrait` command | F5 reassigned to force refresh |
| F8 (reduced-motion) | `:set reduced-motion` command | F8 reassigned to execute (Sanctum) |
| F9 (slippage tolerance) | Collapsible pane in Sanctum | Not frequent enough for a function key |
| Ctrl+C (chain cycle) | Alt+C | Ctrl+C conflicts with terminal SIGINT |

### Terminal Compatibility

Function keys (F1-F12) behave inconsistently across terminal emulators. Known issues:

- **iTerm2 / macOS Terminal:** F1-F4 may be intercepted by the OS. Bardo registers alternate sequences (`\eOP` through `\eOS`) and falls back to Ctrl-based alternatives documented in `:help keys`.
- **tmux / screen:** Function keys require passthrough configuration (`set -g xterm-keys on` in tmux). Bardo detects multiplexer presence and shows a one-time configuration hint if F-key events are missing.
- **Windows Terminal / PuTTY:** F-keys work natively. No special handling required.
- **SSH sessions:** Round-trip latency can cause rapid key sequences to merge. Bardo debounces F-key detection with a 50ms window.

All function key actions are also reachable through the command palette (`:`) as a universal fallback.

---

*⌈ every layer reveals something the previous layer only hinted at ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
