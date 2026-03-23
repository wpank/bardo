# ⌈ ＢＡＲＤＯ ＤＥＳＩＧＮ ＳＹＳＴＥＭ ⌋
## Spatial Grammar · v4.0
### Layout, Composition, and Depth Rules

**Cross-references:** `04-design-system.md` (ROSEDUST palette, the visual identity spec for color and atmospheric rendering), `20-interaction-hierarchy.md` (window/tab/pane navigation model for the 29-screen system), `23-embodied-consciousness.md` (zone architecture mapped to somatic body metaphor, PAD-driven interface transformation)

> **Reader orientation:** This document defines the spatial layout grammar for `bardo-terminal`, the 60fps TUI that is the Golem's primary interface. It belongs to the **interfaces layer** and specifies how screen real estate is divided into five named zones (HEAD, CHEST, GUT, LIMB, GROUND), panel hierarchy rules, depth planes, and compositional patterns. Key concepts: the Spectre (dot-cloud creature sprite representing each Golem, always visible in the sidebar), PAD vector (Pleasure-Arousal-Dominance emotional coordinates from the Daimon subsystem), and the Golem (a mortal autonomous DeFi agent). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## A. Zone Architecture

The terminal divides into five named zones, each corresponding to an aspect of embodied cognition. Zone names come from the somatic architecture in `23-embodied-consciousness.md`.

```
┌──────────────────────────────────────────────────┐
│                   HEAD ZONE                       │
│           (top 3 rows: labels, phase)             │
├──────────────────────────────────────────────────┤
│                                                    │
│                 CHEST ZONE                         │
│          (upper center: emotional core)            │
│                                                    │
├─────────────────────────────────────────────────  │
│   LIMB ZONE     │                │   LIMB ZONE   │
│  (left margin)  │   GUT ZONE     │  (right mrgn) │
│                 │  (lower ctr)   │               │
│                 │                │               │
├──────────────────────────────────────────────────┤
│                  GROUND ZONE                      │
│            (bottom 2 rows: stability)             │
└──────────────────────────────────────────────────┘
```

### Zone Definitions

**HEAD (top 3 rows)**
Default: window bar (row 1), tab bar (row 2), optional context strip (row 3).
PAD response: brightens with Pleasure, sharpens with Dominance.
Characters: ALL CAPS labels, framing glyphs (`⌈ ⌋`), phase indicators.

**CHEST (upper center, rows 4 through ~35%)**
Default: primary status widget (Spectre or vitality number), main data focus.
PAD response: color temperature shifts with Pleasure/Arousal. Heartbeat pulse manifests here.
Characters: bone numbers, creature glyphs, primary data.

**GUT (lower content, rows ~35% through ~80%)**
Default: secondary data panels, logs, lists.
PAD response: somatic marker pre-signals appear here (wave substitution, character shifts) before conscious decision text renders in HEAD.
Characters: data tables, phosphor-decay logs, gauge arrays.

**LIMB (borders and margins, cols 0–2 and final 2 cols)**
Default: Golem sidebar occupies left LIMB; right LIMB carries status indicators.
PAD response: border weight shifts with Dominance (`┊` → `│` → `┃` → `║`).
Characters: box-drawing, navigation indicators, tension markers.

**GROUND (bottom 2 rows)**
Default: status bar (row -2), command bar (row -1).
PAD response: dims with Arousal drop (the ground stabilizes). Hazard stripes manifest here in crisis.
Characters: breadcrumb, heartbeat indicator, contextual key reference.

### Zone Character Budget

| Zone | Default Height | Default Density |
|------|---------------|----------------|
| HEAD | 2-3 rows | HIGH — every cell informative |
| CHEST | 25-35% of height | MEDIUM — data + breathing space |
| GUT | 35-50% of height | VARIABLE — dense or sparse by content |
| LIMB | 2-3 cols each side | MEDIUM — secondary indicators |
| GROUND | 2 rows | HIGH — always full |

---

## B. Persistent Chrome Spec

### Character Budget

**Window Bar (HEAD, row 0):**
```
⌈ＨＥＡＲＴＨ⌋  ＭＩＮＤ  ＳＯＭＡ  ＷＯＲＬＤ  ＦＡＴＥ  ＣＯＭＭＡＮＤ
```
Active window: `⌈ ⌋` frame in `rose`. Inactive: `text_dim`. Unread: `●` suffix.
Height: 1 row. Always rendered.

**Tab Bar (HEAD, row 1):**
```
 1:Overview  2:Signals  3:Operations  4:Status
```
Active tab: `rose`. Inactive: `text_dim`. Number keys selectable.
Height: 1 row. Visible only when a window is active.

**Golem Sidebar (LIMB, cols 0 to N-1):**
Single Golem (a mortal autonomous DeFi agent): full Spectre (the Golem's dot-cloud creature sprite) + mini mortality gauges + phase indicator.
Multi-Golem: stacked mini-Spectres (eyes + dots, ~4 rows each). Selected has bright border.
Width variable by responsive breakpoint (see below).

**Status Bar (GROUND, row -2):**
```
● THRIVING  T:004847  $47.23  HEARTH > Overview [LOCKED]  ∿
```
Contents: phase `●`, tick counter, balance, breadcrumb, heartbeat `∿`.
Height: 1 row. Always rendered.

**Command Bar (GROUND, row -1):**
```
Tab:window  1-5:tab  ←→:pane  Enter:lock  Backspace:back  F1:help
```
Top 5-8 contextual keys for current depth layer.
Height: 1 row. Always rendered.

### Responsive Breakpoints

| Width | Sidebar Width | Layout | Modal Size |
|-------|--------------|--------|-----------|
| < 80 cols | 0 (hidden) | Single col, stack panes | 95% width |
| 80–119 cols | 6 cols, eyes only | Single col | 90% width |
| 120–159 cols | 10 cols, mini Spectre | Two cols | 70% width |
| 160–199 cols | 12 cols, full Spectre | Three cols | 60% width |
| 200+ cols | 14 cols, Spectre + waveforms | Three cols + detail sidebar | 50% width |

---

## C. Panel Hierarchy System

### Border Weight by State

```
NORMAL (unfocused):    ─ │ ┌ ┐ └ ┘   (single, dim)
FOCUSED:               ─ │ ┌ ┐ └ ┘   (single, bright)
LOCKED:                ═ ║ ╔ ╗ ╚ ╝   (double, bright)
MODAL:                 ═ ║ ╔ ╗ ╚ ╝   (double, rose_bright)
DEGRADED (dashed):     ╌ ╎ ┌ ┐ └ ┘   (single, dim, broken)
```

Color:
- Unfocused: `border` (#181420)
- Focused: `border_active` — typically `rose_dim`
- Locked: `border_active` + double weight
- Degraded: `text_ghost`

### Nesting Rules

Maximum visual nesting depth: 4 levels before abstraction (collapse inner panes into tabs).

```
LEVEL 1: Screen pane (single-line border, standard)
  LEVEL 2: Sub-pane within pane (thin border, dim)
    LEVEL 3: Card/element (minimal border or background tint only)
      LEVEL 4: Inline indicator (no border, color distinction only)
```

Nesting beyond level 4 uses tabs or scrollable list, not further nested boxes.

### Border Dimming on Focus Change

When a pane loses focus, its border dims from `border_active` to `border` over 150ms. Adjacent panes that also dim create a "spotlight" effect on the focused pane.

---

## D. Information Density Zones

### The 50% Void Rule

Interior content areas should have at least 50% of cells void or near-void. Density concentrates at:
1. Margins (LIMB zone): dense symbolic data
2. Headers and footers (HEAD, GROUND): dense labels
3. Active data elements: dense and bright

The interior is sparse.

### Density Gradient Formula

```
density(x, y) = 0.4 * edge_proximity(x, y) + 0.2 * significance(x, y)
```

Where:
- `edge_proximity(x, y)` = 1 at the border, 0 at the center (Manhattan distance normalized)
- `significance(x, y)` = 1 for cells containing primary data (vitality, key metrics), 0 for ambient filler

This formula produces the canonical Eva screen effect: dense institutional margins containing an emotionally significant void center.

### Corner Label Zones

Following the Corner Element System (B14 in `17-nerv-aesthetic.md`), each pane's corners hold 10–14 char data labels. These are the densest cells in any pane — tactical display data at maximum information density with minimum area.

---

## E. Modal Geometry

### Five Modal Types with Size Rules

**Detail Modal**
For: Grimoire (the Golem's persistent knowledge store) entries, trade details, achievement details.
Size: 60–75% width, 70–85% height.
Layout: full content top, metadata grid bottom, action buttons at baseline.
Internal tabs: allowed (number keys switch).

**Comparison Modal**
For: Eros/Thanatos splits, counterfactual branches, generational comparisons.
Size: 80% width, 65% height.
Layout: two-column side by side. Differences highlighted.

**Timeline Modal**
For: Confidence history, PnL history, phase timeline, dream replay.
Size: 90% width, 40% height.
Layout: horizontal timeline as primary axis, details below cursor position.

**Graph Modal**
For: Causal graph, Clade topology, lineage tree, force-directed layouts.
Size: 70% width, 70% height.
Layout: interactive graph filling the modal. Zoom/pan with arrow keys.

**Editor Modal**
For: Config parameters, strategy tuning, budget allocation.
Size: 50% width, 60% height.
Layout: form-style. Parameter name, current value, slider/input, preview.

**Nooscopy Modal** (special case — Nooscopy is the Golem-initiated "mind-seeing" mode for decision approval)
For: Golem-initiated decision approval.
Size: 70–85% width, 85–90% height.
Layout: five structured sections. Breathing double-line border. See `22-nooscopy.md` (full specification of the decision approval modal, including the five-section layout and approval flow).

### Modal Backdrop

Background dims to 40% brightness behind any modal. Golem sidebar remains visible at full brightness (the Golem is always present).

---

## F. Compositional Patterns

Eight reusable spatial arrangements:

### F1. Anchor + Surround
Center element (Spectre, primary metric, globe wireframe) with orbital data panels at N/E/S/W. Thin connection lines from center to panels. Dense at center and panel edges, sparse in between.

Use: HEARTH Overview, FATE Mortality, World > Solaris detail.

### F2. Triptych
Three equal columns of identical weight. Bright borders between them. No center emphasis. Collaborative — each column is a peer. MAGI Theater standard layout.

Use: MAGI panel, Mind > Pipeline specialist modules, generation comparison.

### F3. Asymmetric Focus
65% / 35% split. Dense content at 65% side, sidebar at 35%. Density gradient at the join: 65% side is information-rich near the border, thinning toward its outer edge. 35% side is sparse.

Use: MIND > Grimoire (list / preview), SOMA > Portfolio (cards / NAV strip).

### F4. Corner-Heavy
Four strong corner labels with void center. The center emphasizes emptiness — the data lives at the edges. The more important the thing in the center (one creature, one number), the more the corners frame it.

Use: HEARTH > Overview (Spectre at center, metrics at corners), Nooscopy modal.

### F5. Horizon Split
Strong horizontal line at mid-screen. Different visual register above vs. below: above = conscious/intentional (labeled, structured), below = somatic/background (ambient, texture). The horizon is the body's midline.

Use: FATE > Mortality (three clocks above / defense layers below), MIND > Dreams.

### F6. Lattice
Uniform grid, each cell same visual weight. No center emphasis. Pure data wall — architectural mass. See B1 (Unit Array) in `17-nerv-aesthetic.md`.

Use: Probe Grid, Grimoire entry grid, Achievement grid, Clade peer tiles.

### F7. Cascade
Stacked priority — most important fills top, remainder compressed below. Top section uses 50% of height for one element. Subsequent sections get diminishing height allocation.

Use: SOMA > Budget (credit balance dominates, allocation below), any single-metric focus view.

### F8. Void Pool
Single element in wide void. Meaning through isolation. The element is important because of what surrounds it, not because of decoration on it.

Use: Phase transition moments, death countdown, stasis label, TESTAMENT stamp.

---

## G. Depth Planes

The terminal implies 3–4 depth layers through brightness and character density:

| Depth | Layer | Brightness | Characters | Colors |
|-------|-------|-----------|-----------|--------|
| 0 (deepest) | Deep void | v: 0.0-0.05 | `⠀ ·` | `text_phantom` |
| 1 | Background fill | v: 0.05-0.15 | `░ ╌ +` | `text_ghost` |
| 2 | Midground | v: 0.15-0.45 | `▒ │ ─ ┄` | `rose_dim` or `text_dim` |
| 3 | Foreground | v: 0.45-0.85 | `▓ █ ─ │` | `rose` |
| 4 (surface) | Accents | v: 0.85-1.0 | `█ ✦ ●` | `bone` or `rose_bright` |

Rule: never more than 5% of cells at surface brightness. The eye reads brightness as proximity — a dense surface layer destroys the depth illusion.

### Layered Background Construction

```
Void background (#060608)
  └─ Deep void layer: scattered `·` at 0.5% density in text_phantom
      └─ Background fill: geometric pattern (B13) at 10% density in text_ghost
          └─ Midground: dim data labels, border shadows in rose_deep
              └─ Foreground: primary data, borders in rose/text_primary
                  └─ Accent: sparse highlights, bone numbers, rose_bright glyphs
```

---

## H. Rust Layout Primitives

### Zone

```rust
pub struct Zone {
    pub kind: ZoneKind,
    pub area: Rect,
}

pub enum ZoneKind {
    Head, Chest, Gut, Limb, Ground,
}

impl Zone {
    pub fn from_terminal(full: Rect) -> [Zone; 5] {
        let head_h = 2u16;
        let ground_h = 2u16;
        let limb_w = 14u16;  // sidebar + margin
        [
            Zone { kind: ZoneKind::Head,   area: Rect::new(full.x, full.y, full.width, head_h) },
            Zone { kind: ZoneKind::Ground, area: Rect::new(full.x, full.bottom() - ground_h, full.width, ground_h) },
            Zone { kind: ZoneKind::Limb,   area: Rect::new(full.x, full.y + head_h, limb_w, full.height - head_h - ground_h) },
            Zone { kind: ZoneKind::Chest,
                   area: Rect::new(full.x + limb_w, full.y + head_h,
                                   full.width - limb_w,
                                   (full.height - head_h - ground_h) / 3) },
            Zone { kind: ZoneKind::Gut,
                   area: Rect::new(full.x + limb_w,
                                   full.y + head_h + (full.height - head_h - ground_h) / 3,
                                   full.width - limb_w,
                                   (full.height - head_h - ground_h) * 2 / 3) },
        ]
    }
}
```

### PaneGrid

```rust
pub struct PaneGrid {
    pub panes: Vec<PaneDescriptor>,
    pub focused: usize,
    pub locked: Option<usize>,
}

pub struct PaneDescriptor {
    pub id: usize,
    pub area: Rect,
    pub title: String,
    pub border_state: BorderState,
}

pub enum BorderState {
    Normal,
    Focused,
    Locked,
    Degraded(f64),   // 0.0-1.0 corruption level
}

impl PaneGrid {
    pub fn focus_next(&mut self) {
        self.focused = (self.focused + 1) % self.panes.len();
    }
    pub fn focus_prev(&mut self) {
        self.focused = self.focused.checked_sub(1)
            .unwrap_or(self.panes.len() - 1);
    }
    pub fn lock(&mut self) {
        self.locked = Some(self.focused);
    }
    pub fn unlock(&mut self) {
        self.locked = None;
    }
    pub fn render_borders(&self, buf: &mut Buffer) {
        for (i, pane) in self.panes.iter().enumerate() {
            let is_focused = i == self.focused;
            let is_locked = self.locked == Some(i);
            let border = match &pane.border_state {
                BorderState::Locked => render_double_border(pane.area, buf),
                BorderState::Focused => render_single_border_bright(pane.area, buf),
                BorderState::Degraded(d) => render_degraded_border(pane.area, buf, *d),
                BorderState::Normal => render_single_border_dim(pane.area, buf),
            };
        }
    }
}
```

### ModalLayer

```rust
pub struct ModalLayer {
    pub stack: Vec<Modal>,
}

pub struct Modal {
    pub area: Rect,
    pub title: String,
    pub content_type: ModalType,
    pub border_style: BorderStyle,
}

pub enum ModalType {
    Detail, Comparison, Timeline, Graph, Editor, Nooscopy,
}

pub enum BorderStyle {
    Double,             // ═ ║ ╔ ╗ ╚ ╝
    Breathing(f64),     // double + opacity pulse from AnimationContext
    Dashed,             // ╌ ╎ ┌ ┐ └ ┘
}

impl ModalLayer {
    pub fn push(&mut self, modal: Modal) {
        self.stack.push(modal);
    }
    pub fn pop(&mut self) -> Option<Modal> {
        self.stack.pop()
    }
    pub fn pop_all(&mut self) {
        self.stack.clear();
    }
    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }
}
```

### FocusStack + Breadcrumb

```rust
pub struct FocusStack {
    layers: Vec<FocusLayer>,
}

pub enum FocusLayer {
    Window(usize),
    Tab(usize),
    Pane(usize),
    PaneLocked(usize),
    Element(usize),
    Modal(usize),
}

impl FocusStack {
    pub fn breadcrumb(&self) -> String {
        // Generate "HEARTH > Overview > Heartbeat Log [LOCKED]" string
        self.layers.iter().map(|l| l.label()).collect::<Vec<_>>().join(" > ")
    }
    pub fn depth(&self) -> usize {
        self.layers.len()
    }
    pub fn back(&mut self) -> Option<FocusLayer> {
        // Layer 0 (Window) is the root — back() at depth ≤ 1 is a no-op.
        if self.layers.len() <= 1 {
            return None;
        }
        self.layers.pop()
    }
    pub fn escape_to_pane_focus(&mut self) {
        // Pop until at Layer 3 (Pane focus)
        while self.layers.len() > 3 {
            self.layers.pop();
        }
    }
}
```

**Layer 0 back() semantics:** `focus_stack.back()` at Layer 0 (the root window level) is a no-op. The window layer is the navigation floor. There is nowhere further back to go, so the call returns `None` and the stack stays unchanged. This prevents accidental depopulation of the focus stack and keeps the window bar always anchored.

---

*⌈ the space between elements is as meaningful as the elements themselves ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
