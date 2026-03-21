# Plan 05: Terminal Widget Library

## Context

Plan 05 builds `apps/bardo-terminal/src/widgets/` — the reusable ratatui widget library that all 29 screens will consume. Every widget is self-contained: it takes a struct of data, implements `ratatui::widgets::Widget`, and renders into the buffer it's given. No widget holds async state or network handles. Live data arrives through `AppState` and MockData structs; the TODO comments mark every connection point for Plans 70a-70c.

The widgets must work at 60fps. That means no heap allocation in the hot render path, no format strings longer than necessary, and no `.collect()` calls that can be pre-computed. The braille encoding is computed in-place per frame. The heatmap gradient uses a lookup table, not per-cell f64 arithmetic.

Design target: `prd2/20-styx/05-tui-experience.md` §5 (custom widgets) and `prd2/13-runtime/19-cinematic-system.md` §3 (transition tier vocabulary, Tier 1 ambient pulse behavior). ROSEDUST palette from Plan 04.

## Previous Plan

Plan 04 created `apps/bardo-terminal/`: the application skeleton — terminal initialization, 60fps render loop, `Screen` trait, `ScreenId` enum (all 29 screens), `ScreenRegistry`, `AppState`, `MockVitality`, `ConnectionStatus`, `ColorPalette` (ROSEDUST constants), and `LayoutBreakpoint`. The `HomeScreen` is the only implemented screen. All others are `StubScreen`.

## Prerequisites

- **Plan 04** — `bardo_terminal::screen::Screen` trait, `bardo_terminal::state::AppState`, `bardo_terminal::palette::*` color constants, `LayoutBreakpoint`, `App` struct
- `ratatui 0.30` and `crossterm 0.28` already declared in `apps/bardo-terminal/Cargo.toml`
- No new crate-level dependencies beyond what Plan 04 declared

## Imports

All widget modules import from the parent crate:

```rust
use bardo_terminal::palette::*;       // ROSEDUST color constants
use bardo_terminal::state::AppState;  // read-only state access
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::Widget,
};
use std::collections::VecDeque;       // feed.rs only
```

Widget modules live inside `bardo-terminal` itself (`src/widgets/`), so the imports are `crate::palette::*` and `crate::state::AppState`.

## Exports

All re-exported from `bardo_terminal::widgets` via `src/widgets/mod.rs`.

| Type | Module | Purpose |
|------|--------|---------|
| `BrailleSparkline` | `widgets::sparkline` | 2x4 braille dot sparkline, up to 80 data points in 40 cols |
| `VitalityGauge` | `widgets::gauge` | Phase-colored health bar with `MockPhase` |
| `ConfidenceGauge` | `widgets::gauge` | Confidence 0..1 bar, amber→green thresholds |
| `AccuracyGauge` | `widgets::gauge` | Prediction accuracy 0..1, same threshold scheme |
| `MockPhase` | `widgets::gauge` | Placeholder for `BehavioralPhase` (Plan 13a) |
| `PheromoneHeatmap` | `widgets::heatmap` | Animated Viridis-like 2D signal strength grid |
| `PheromoneLayer` | `widgets::heatmap` | `Threat` / `Opportunity` / `Wisdom` layer tag |
| `TimelineRibbon` | `widgets::timeline` | Horizontal phase/event ribbon |
| `TimelineEvent` | `widgets::timeline` | Single event on the ribbon |
| `RibbonEventType` | `widgets::timeline` | `TradeExecuted` / `DreamStarted` / `PhaseChange` / `Anomaly` / `Death` |
| `EventFeed` | `widgets::feed` | Scrollback log with filter |
| `FeedEntry` | `widgets::feed` | Single log line (tick, level, message) |
| `FeedLevel` | `widgets::feed` | `Info` / `Warn` / `Error` / `Debug` |
| `TabBar` | `widgets::tabs` | Window tab strip with active highlight |
| `StatusBar` | `widgets::status_bar` | Fixed bottom-of-screen status line |
| `ScrollableList` | `widgets::scrolllist` | Cursor-driven list with substring filter |
| `KeyHelpOverlay` | `widgets::key_help` | Floating keybinding hint box |
| `KeyBinding` | `widgets::key_help` | `key: String`, `description: String` |

## Cargo Dependencies

No new entries in `apps/bardo-terminal/Cargo.toml`. All dependencies were declared in Plan 04:

```toml
ratatui = { workspace = true }
crossterm = { workspace = true }
```

`VecDeque` is from `std`. No external crates for gradient math — values are computed with integer arithmetic or a small inline lookup table.

## Source Files

```
apps/bardo-terminal/src/
├── widgets/
│   ├── mod.rs           — pub mod declarations + pub use re-exports for all widget types
│   ├── sparkline.rs     — BrailleSparkline
│   ├── gauge.rs         — VitalityGauge, ConfidenceGauge, AccuracyGauge, MockPhase
│   ├── heatmap.rs       — PheromoneHeatmap, PheromoneLayer
│   ├── timeline.rs      — TimelineRibbon, TimelineEvent, RibbonEventType
│   ├── feed.rs          — EventFeed, FeedEntry, FeedLevel
│   ├── tabs.rs          — TabBar
│   ├── status_bar.rs    — StatusBar
│   ├── scrolllist.rs    — ScrollableList
│   └── key_help.rs      — KeyHelpOverlay, KeyBinding
└── lib.rs               — add `pub mod widgets;` (if bardo-terminal exposes a lib target)
                           OR add `mod widgets;` in main.rs and pub-use from app.rs
```

Because `bardo-terminal` is a binary crate (no `src/lib.rs`), all widget modules are declared in `src/main.rs` as `mod widgets;` or pulled in via a `widgets` re-export module accessible from screen implementations. The simplest approach: declare `mod widgets;` in `main.rs`, make widget types `pub`, and import them in screen files as `use crate::widgets::*;`.

## Implementation Details

---

### Unit 1: Braille Sparkline & Gauges

**Files:** `src/widgets/sparkline.rs`, `src/widgets/gauge.rs`

#### Quick Reference

**Braille dot encoding:**

Unicode braille block starts at U+2800. Each character encodes an 8-dot 2x4 grid (2 columns, 4 rows). The bit positions for the 8 dots are:

```
Dot layout (column, row):     Bit value:
col0,row0 = dot 1            bit 0  (0x01)
col0,row1 = dot 2            bit 1  (0x02)
col0,row2 = dot 3            bit 2  (0x04)
col0,row3 = dot 4 (bottom)   bit 3  (0x08)  — note: not bit 6
col1,row0 = dot 5            bit 4  (0x10)
col1,row1 = dot 6            bit 5  (0x20)
col1,row2 = dot 7            bit 6  (0x40)
col1,row3 = dot 8 (bottom)   bit 7  (0x80)

Unicode mapping: U+2800 + bit_flags
```

Important detail: Unicode braille uses bit positions 0–7 for dots 1–8, but the standard dot numbering is not top-to-bottom in both columns simultaneously. The left column uses dots 1,2,3,7 (bits 0,1,2,6) and the right uses dots 4,5,6,8 (bits 3,4,5,7) in some typefaces — but for rendering purposes, use the layout above which matches the visual appearance in terminals.

The correct bit mapping for a sparkline (filling from the bottom of the cell upward):

```rust
// Left column (col 0): bits for rows 3,2,1,0 from bottom:
const LEFT_COL_BITS:  [u8; 4] = [0x40, 0x04, 0x02, 0x01]; // rows 3→0
// Right column (col 1): bits for rows 3,2,1,0 from bottom:
const RIGHT_COL_BITS: [u8; 4] = [0x80, 0x20, 0x10, 0x08]; // rows 3→0

// To fill n_dots from bottom in left column:
fn left_bits(n_dots: usize) -> u8 {
    LEFT_COL_BITS[4_usize.saturating_sub(n_dots)..]
        .iter()
        .fold(0u8, |acc, &b| acc | b)
}
fn right_bits(n_dots: usize) -> u8 {
    RIGHT_COL_BITS[4_usize.saturating_sub(n_dots)..]
        .iter()
        .fold(0u8, |acc, &b| acc | b)
}
```

**BrailleSparkline struct and render logic:**

```rust
pub struct BrailleSparkline {
    pub data: Vec<f64>,       // up to 80 data points; extra points are ignored
    pub max_value: f64,       // scale ceiling; 0.0 → auto-scale from data.max()
    pub color: Color,
    pub label: Option<String>,
}

impl Widget for BrailleSparkline {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 { return; }

        // Each terminal cell holds 2 data points (left and right braille column)
        // Each cell is 4 dots tall
        let cell_count = area.width as usize;
        let data_capacity = cell_count * 2;  // 2 cols per cell → up to 80 pts in 40 cols

        // Scale
        let max = if self.max_value > 0.0 {
            self.max_value
        } else {
            self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(1.0)
        };

        // Map each data point to dot count (0..=4 per column, one column per point)
        let dot_height = area.height as usize * 4;  // total dots available (4 per cell row)
        // We render into a single row of braille cells. height is expected to be 1.
        // If height > 1, render from top row down using the same data window.

        let n = self.data.len().min(data_capacity);
        let offset = self.data.len().saturating_sub(data_capacity); // newest points at right

        let y = area.y;
        for cell_idx in 0..cell_count {
            let x = area.x + cell_idx as u16;
            let left_data_idx  = offset + cell_idx * 2;
            let right_data_idx = left_data_idx + 1;

            let left_val  = self.data.get(left_data_idx).cloned().unwrap_or(0.0);
            let right_val = self.data.get(right_data_idx).cloned().unwrap_or(0.0);

            let left_dots  = ((left_val  / max) * 4.0).round().clamp(0.0, 4.0) as usize;
            let right_dots = ((right_val / max) * 4.0).round().clamp(0.0, 4.0) as usize;

            let bits = left_bits(left_dots) | right_bits(right_dots);
            let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');

            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(ch);
                cell.set_style(Style::default().fg(self.color));
            }
        }

        // Optional label in dim text above/below (if height >= 2)
        if area.height >= 2 {
            if let Some(label) = self.label {
                let label_y = area.y + area.height - 1;
                let truncated: String = label.chars().take(area.width as usize).collect();
                buf.set_string(area.x, label_y, &truncated,
                    Style::default().fg(crate::palette::TEXT_DIM));
            }
        }
    }
}
```

**VitalityGauge:**

```rust
/// Placeholder phase. TODO Plan 70a: replace with golem_mortality::BehavioralPhase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockPhase {
    Thriving,
    Stable,
    Conservation,
    Declining,
    Terminal,
}

impl MockPhase {
    pub fn gauge_color(&self) -> Color {
        use crate::palette::*;
        match self {
            MockPhase::Thriving     => SUCCESS,     // muted sage/green
            MockPhase::Stable       => Color::Rgb(88, 160, 170),  // cyan
            MockPhase::Conservation => WARNING,     // amber
            MockPhase::Declining    => Color::Rgb(180, 100, 60),  // orange
            MockPhase::Terminal     => ROSE_BRIGHT, // red-rose
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MockPhase::Thriving     => "THRIVING",
            MockPhase::Stable       => "STABLE",
            MockPhase::Conservation => "CONSERVATION",
            MockPhase::Declining    => "DECLINING",
            MockPhase::Terminal     => "TERMINAL",
        }
    }
}

pub struct VitalityGauge {
    pub value: f64,      // 0.0 to 1.0
    pub label: String,
    pub phase: MockPhase,  // TODO Plan 70a: replace with real BehavioralPhase
}

impl Widget for VitalityGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::palette::*;

        if area.width < 4 { return; }
        let fill_width = ((self.value.clamp(0.0, 1.0)) * area.width as f64) as u16;
        let phase_color = self.phase.gauge_color();

        // Label row (top if height >= 2, else skip)
        let gauge_y = if area.height >= 2 {
            let label = format!(" {} {:.0}% ", self.label, self.value * 100.0);
            buf.set_string(area.x, area.y,
                &label.chars().take(area.width as usize).collect::<String>(),
                Style::default().fg(BONE).add_modifier(Modifier::BOLD));
            area.y + 1
        } else {
            area.y
        };

        // Bar row
        for x in area.x..(area.x + area.width) {
            if let Some(cell) = buf.cell_mut((x, gauge_y)) {
                if x < area.x + fill_width {
                    cell.set_char(crate::palette::BLOCK_FULL);
                    cell.set_style(Style::default().fg(phase_color).bg(BG_RAISED));
                } else {
                    cell.set_char(crate::palette::BLOCK_LIGHT);
                    cell.set_style(Style::default().fg(BORDER).bg(BG_RAISED));
                }
            }
        }
    }
}
```

**ConfidenceGauge and AccuracyGauge:**

Both share the same rendering logic as `VitalityGauge` but with different color thresholds:

```rust
fn confidence_color(value: f64) -> Color {
    use crate::palette::*;
    if value >= 0.75 { SUCCESS }
    else if value >= 0.50 { Color::Rgb(88, 160, 170) }  // cyan
    else if value >= 0.25 { WARNING }
    else { ROSE_DIM }
}

fn accuracy_color(value: f64) -> Color {
    // Same thresholds as confidence
    confidence_color(value)
}

pub struct ConfidenceGauge { pub value: f64, pub label: String }
pub struct AccuracyGauge   { pub value: f64, pub label: String }
```

Implement `Widget` for both by delegating to the shared fill logic with their respective color functions. No `MockPhase` needed — confidence and accuracy are pure scalars.

**Tests (in `src/widgets/sparkline.rs` and `gauge.rs`):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_braille_sparkline_encodes_correctly() {
        // Empty braille char for all-zero data
        assert_eq!(left_bits(0), 0x00);
        assert_eq!(left_bits(4), 0x01 | 0x02 | 0x04 | 0x40);
        // All 8 dots lit = U+28FF
        let all_bits = left_bits(4) | right_bits(4);
        assert_eq!(all_bits, 0xFF);
        let ch = char::from_u32(0x2800 + all_bits as u32).unwrap();
        assert_eq!(ch, '\u{28FF}');
    }

    #[test]
    fn test_vitality_gauge_colors() {
        assert_eq!(MockPhase::Thriving.gauge_color(),     crate::palette::SUCCESS);
        assert_eq!(MockPhase::Terminal.gauge_color(),     crate::palette::ROSE_BRIGHT);
    }

    #[test]
    fn test_confidence_color_thresholds() {
        assert_eq!(confidence_color(0.80), crate::palette::SUCCESS);
        assert_eq!(confidence_color(0.60), Color::Rgb(88, 160, 170));
        assert_eq!(confidence_color(0.30), crate::palette::WARNING);
        assert_eq!(confidence_color(0.10), crate::palette::ROSE_DIM);
    }
}
```

---

### Unit 2: Pheromone Heatmap & Timeline Ribbon

**Files:** `src/widgets/heatmap.rs`, `src/widgets/timeline.rs`

#### Quick Reference

**Viridis-inspired gradient (7-stop lookup table):**

The Viridis colormap runs from dark purple (low) through blue, cyan, green, yellow-green to yellow (high). For terminal rendering, map each cell's `f64` signal strength (0.0..=1.0) to one of 7 colors:

```rust
const VIRIDIS: [(u8, u8, u8); 7] = [
    (68,  1,   84),   // 0.0 — dark purple
    (72,  40,  120),  // 0.17 — purple
    (62,  83,  160),  // 0.33 — blue
    (49,  126, 157),  // 0.50 — teal/cyan
    (53,  183, 121),  // 0.67 — green
    (149, 216, 64),   // 0.83 — yellow-green
    (253, 231, 37),   // 1.0 — yellow
];

fn viridis_color(value: f64) -> Color {
    let v = value.clamp(0.0, 1.0);
    let idx_f = v * 6.0;
    let idx = idx_f.floor() as usize;
    let t = idx_f - idx as f64;

    let (r0, g0, b0) = VIRIDIS[idx.min(6)];
    let (r1, g1, b1) = VIRIDIS[(idx + 1).min(6)];

    let lerp = |a: u8, b: u8, t: f64| -> u8 {
        (a as f64 + (b as f64 - a as f64) * t).round() as u8
    };
    Color::Rgb(lerp(r0, r1, t), lerp(g0, g1, t), lerp(b0, b1, t))
}
```

**PheromoneLayer-specific overlay:**

Each layer tints the base Viridis color with a layer-specific hue modifier:
- `Threat`: shift toward rose — add red component bias
- `Opportunity`: use Viridis as-is (the warm yellow end signals opportunity)
- `Wisdom`: shift toward dream indigo — blend with (88, 88, 120) at low values

```rust
fn layer_tint(base: Color, layer: PheromoneLayer, value: f64) -> Color {
    match (base, layer) {
        (Color::Rgb(r, g, b), PheromoneLayer::Threat) => {
            Color::Rgb(r.saturating_add(40), g.saturating_sub(20), b.saturating_sub(20))
        }
        (Color::Rgb(r, g, b), PheromoneLayer::Wisdom) => {
            // blend toward dream indigo at low values
            let t = (1.0 - value).clamp(0.0, 0.5) as f32;
            let r2 = (r as f32 * (1.0 - t) + 88.0 * t) as u8;
            let g2 = (g as f32 * (1.0 - t) + 88.0 * t) as u8;
            let b2 = (b as f32 * (1.0 - t) + 120.0 * t) as u8;
            Color::Rgb(r2, g2, b2)
        }
        _ => base,
    }
}
```

**PheromoneHeatmap struct:**

```rust
pub struct PheromoneHeatmap {
    pub grid: Vec<Vec<f64>>,  // grid[row][col], values 0.0..=1.0
    pub width: u16,           // number of grid columns
    pub height: u16,          // number of grid rows
    pub layer: PheromoneLayer,
    /// TODO Plan 70a: replace with live pheromone field from golem-coordination
    pub pulse_cells: Vec<(usize, usize)>,  // cells currently pulsing (bright flash)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PheromoneLayer { Threat, Opportunity, Wisdom }
```

**Render:**

Each grid cell maps to one terminal cell. The cell character is `BLOCK_FULL` (`█`) with a background and foreground both set to the Viridis color (solid fill). Pulsing cells use `Color::Rgb(255, 255, 255)` as a brief override — at 60fps a pulse_cell list refreshed each tick with a 30-frame (500ms) TTL creates the flash effect.

```rust
impl Widget for PheromoneHeatmap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.grid.len().min(area.height as usize);
        let cols = if rows > 0 { self.grid[0].len().min(area.width as usize) } else { 0 };

        for row in 0..rows {
            for col in 0..cols {
                let x = area.x + col as u16;
                let y = area.y + row as u16;
                let value = self.grid[row].get(col).cloned().unwrap_or(0.0);
                let is_pulsing = self.pulse_cells.contains(&(row, col));

                let base = viridis_color(value);
                let color = if is_pulsing {
                    Color::Rgb(255, 255, 255)  // pulse flash
                } else {
                    layer_tint(base, self.layer, value)
                };

                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char('█');
                    cell.set_style(Style::default().fg(color).bg(color));
                }
            }
        }
    }
}
```

**TimelineRibbon:**

The ribbon is a single horizontal row. Each pixel (terminal cell) represents `window_ticks / area.width` ticks. Events at tick positions mark that cell with an event-type glyph and color. Cells between events are colored by the phase active at that point — use the MockPhase color or a dim neutral if no phase data is present.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonEventType {
    TradeExecuted,
    DreamStarted,
    PhaseChange,
    Anomaly,
    Death,
}

pub struct TimelineEvent {
    pub tick: u64,
    pub event_type: RibbonEventType,
    pub severity: u8,  // 0-255, modulates brightness
}

pub struct TimelineRibbon {
    pub events: Vec<TimelineEvent>,   // TODO Plan 70a: connect to GolemEvent stream
    pub window_ticks: u64,            // how many ticks the ribbon covers
    pub current_tick: u64,            // rightmost tick
}

impl RibbonEventType {
    fn glyph(&self) -> char {
        match self {
            RibbonEventType::TradeExecuted => '▲',
            RibbonEventType::DreamStarted  => '◌',
            RibbonEventType::PhaseChange   => '◆',
            RibbonEventType::Anomaly       => '!',
            RibbonEventType::Death         => '✕',
        }
    }

    fn color(&self) -> Color {
        use crate::palette::*;
        match self {
            RibbonEventType::TradeExecuted => SUCCESS,
            RibbonEventType::DreamStarted  => DREAM,
            RibbonEventType::PhaseChange   => BONE,
            RibbonEventType::Anomaly       => WARNING,
            RibbonEventType::Death         => ROSE_BRIGHT,
        }
    }
}

impl Widget for TimelineRibbon {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::palette::*;

        if area.width == 0 { return; }
        let w = area.width as u64;
        let ticks_per_cell = self.window_ticks.max(1) / w.max(1);
        let start_tick = self.current_tick.saturating_sub(self.window_ticks);

        for col in 0..area.width {
            let cell_start_tick = start_tick + col as u64 * ticks_per_cell;
            let is_current = col == area.width - 1; // rightmost = now

            let x = area.x + col;
            let y = area.y;

            // Find events in this cell's tick range
            let event_in_cell = self.events.iter()
                .filter(|e| e.tick >= cell_start_tick
                    && e.tick < cell_start_tick + ticks_per_cell.max(1))
                .max_by_key(|e| e.severity);

            if let Some(event) = event_in_cell {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(event.event_type.glyph());
                    cell.set_style(Style::default().fg(event.event_type.color())
                        .add_modifier(Modifier::BOLD));
                }
            } else {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    // Background track: dim horizontal line
                    cell.set_char(if is_current { '┤' } else { '─' });
                    cell.set_style(Style::default().fg(
                        if is_current { BONE_DIM } else { BORDER }
                    ));
                }
            }
        }
    }
}
```

---

### Unit 3: Event Feed & Status Bar

**Files:** `src/widgets/feed.rs`, `src/widgets/status_bar.rs`

#### Quick Reference

**EventFeed ring buffer:**

`VecDeque` with a maximum capacity. When a new entry arrives and `len() == max_entries`, `pop_front()` before `push_back()`. Scroll offset tracks how far back the user has scrolled from the most recent entry. A `filter: Option<String>` field holds a substring to match against messages (case-insensitive).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedLevel { Info, Warn, Error, Debug }

impl FeedLevel {
    fn color(&self) -> Color {
        use crate::palette::*;
        match self {
            FeedLevel::Info  => TEXT_PRIMARY,
            FeedLevel::Warn  => WARNING,
            FeedLevel::Error => ROSE_BRIGHT,
            FeedLevel::Debug => TEXT_DIM,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            FeedLevel::Info  => "INFO ",
            FeedLevel::Warn  => "WARN ",
            FeedLevel::Error => "ERROR",
            FeedLevel::Debug => "DBG  ",
        }
    }
}

pub struct FeedEntry {
    pub tick: u64,
    pub level: FeedLevel,
    pub message: String,
}

pub struct EventFeed {
    pub entries: VecDeque<FeedEntry>,
    pub max_entries: usize,   // default 1000
    pub scroll_offset: usize, // 0 = show newest, N = scrolled N lines back
    pub filter: Option<String>,
}

impl EventFeed {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries.min(1000)),
            max_entries,
            scroll_offset: 0,
            filter: None,
        }
    }

    pub fn push(&mut self, entry: FeedEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Returns filtered entries, newest first.
    fn visible_entries(&self) -> Vec<&FeedEntry> {
        let filter_lower = self.filter.as_deref()
            .map(|f| f.to_lowercase());
        self.entries.iter()
            .filter(|e| {
                filter_lower.as_deref()
                    .map(|f| e.message.to_lowercase().contains(f))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}
```

**EventFeed render:**

```rust
impl Widget for &EventFeed {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::palette::*;

        let visible = self.visible_entries();
        let total = visible.len();
        let display_rows = area.height as usize;

        // Apply scroll: offset from the newest entry
        let start_idx = self.scroll_offset.min(total.saturating_sub(display_rows));

        for (row, entry) in visible.iter()
            .skip(start_idx)
            .take(display_rows)
            .enumerate()
        {
            let y = area.y + row as u16;

            // Tick number in dim
            let tick_str = format!("{:>6} ", entry.tick);
            let level_str = entry.level.label();
            let msg_max = area.width.saturating_sub(
                tick_str.len() as u16 + level_str.len() as u16 + 1
            ) as usize;
            let msg: String = entry.message.chars().take(msg_max).collect();

            let mut x = area.x;
            buf.set_string(x, y, &tick_str, Style::default().fg(TEXT_GHOST));
            x += tick_str.len() as u16;
            buf.set_string(x, y, level_str,
                Style::default().fg(entry.level.color()).add_modifier(Modifier::BOLD));
            x += level_str.len() as u16;
            buf.set_string(x, y, " ", Style::default());
            x += 1;
            buf.set_string(x, y, &msg, Style::default().fg(entry.level.color()));
        }
    }
}
```

Note `Widget for &EventFeed` (reference) because `EventFeed` holds a `VecDeque` and we don't want to move it on render.

**StatusBar:**

The status bar is a single-row widget pinned to the bottom of the screen. `App` renders it after all other widgets.

```rust
pub struct StatusBar<'a> {
    pub phase: &'a str,           // e.g. "STABLE" — TODO Plan 70a: from AppState
    pub tick: u64,
    pub pad_summary: &'a str,     // e.g. "P:0.3 A:0.1 D:0.4" — TODO Plan 70a
    pub credit_balance: &'a str,  // e.g. "$142.50" — TODO Plan 70a
    pub projected_days: Option<f64>,  // remaining lifespan estimate — TODO Plan 70a
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::palette::*;

        // Fill background
        for x in area.x..(area.x + area.width) {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_char(' ');
                cell.set_style(Style::default().bg(BG_MID));
            }
        }

        // Left section: phase + tick
        let left = format!(" {} #{}", self.phase, self.tick);
        buf.set_string(area.x, area.y, &left,
            Style::default().fg(BONE).bg(BG_MID).add_modifier(Modifier::BOLD));

        // Center section: PAD summary
        let center_x = area.x + area.width / 3;
        buf.set_string(center_x, area.y, self.pad_summary,
            Style::default().fg(TEXT_DIM).bg(BG_MID));

        // Right section: credit + days remaining
        let right = if let Some(days) = self.projected_days {
            format!("{} | {:.0}d remaining ", self.credit_balance, days)
        } else {
            format!("{} ", self.credit_balance)
        };
        let right_x = area.x + area.width.saturating_sub(right.len() as u16);
        buf.set_string(right_x, area.y, &right,
            Style::default().fg(TEXT_DIM).bg(BG_MID));
    }
}
```

**EventFeed tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_feed_scroll() {
        let mut feed = EventFeed::new(5);
        for i in 0..10u64 {
            feed.push(FeedEntry { tick: i, level: FeedLevel::Info,
                message: format!("msg {}", i) });
        }
        // max_entries=5, so only entries 5..9 remain (newest)
        assert_eq!(feed.entries.len(), 5);
        assert_eq!(feed.entries.back().unwrap().tick, 9);

        // Filter test
        feed.filter = Some("msg 6".to_string());
        let vis = feed.visible_entries();
        assert_eq!(vis.len(), 1);
        assert_eq!(vis[0].tick, 6);
    }
}
```

---

### Unit 4: Tab Bar, Scrollable List & Key Help Overlay

**Files:** `src/widgets/tabs.rs`, `src/widgets/scrolllist.rs`, `src/widgets/key_help.rs`

#### Quick Reference

**TabBar:**

Renders a horizontal strip. The active tab is highlighted with `BONE` + `BORDER_ACTIVE` borders, inactive tabs use `TEXT_DIM`. Each tab label is padded to at least 3 chars. On Compact layout, only the active tab's label is shown; others collapse to a dot.

```rust
pub struct TabBar<'a> {
    pub tabs: &'a [&'a str],
    pub active: usize,
}

impl<'a> Widget for TabBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::palette::*;

        // Fill background row
        for x in area.x..(area.x + area.width) {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_char(' ');
                cell.set_style(Style::default().bg(BG_MID));
            }
        }

        let mut x = area.x;
        for (i, &tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active;
            let label = if is_active {
                format!("⌈ {} ⌋", tab)
            } else {
                format!("  {}  ", tab)
            };

            if x + label.len() as u16 > area.x + area.width { break; }

            buf.set_string(x, area.y, &label, Style::default()
                .fg(if is_active { BONE } else { TEXT_DIM })
                .bg(if is_active { BG_RAISED } else { BG_MID })
                .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() })
            );

            x += label.len() as u16;
        }
    }
}
```

**ScrollableList:**

Holds a `Vec<String>` of items, a `cursor` index, a `scroll_offset` tracking the top visible row, and an optional `filter` for substring matching. `j`/`k` or arrow keys move the cursor; `filter` is typed into `App`'s input buffer and passed in at render time.

```rust
pub struct ScrollableList {
    pub items: Vec<String>,      // TODO Plan 70a: sourced from AppState
    pub cursor: usize,
    pub scroll_offset: usize,
    pub filter: Option<String>,
}

impl ScrollableList {
    pub fn new(items: Vec<String>) -> Self {
        Self { items, cursor: 0, scroll_offset: 0, filter: None }
    }

    pub fn filtered_items(&self) -> Vec<(usize, &str)> {
        // Returns (original_index, item_str) pairs for all matching items
        let filter_lower = self.filter.as_deref()
            .map(|f| f.to_lowercase());
        self.items.iter().enumerate()
            .filter(|(_, item)| {
                filter_lower.as_deref()
                    .map(|f| item.to_lowercase().contains(f))
                    .unwrap_or(true)
            })
            .map(|(i, s)| (i, s.as_str()))
            .collect()
    }

    pub fn move_cursor_down(&mut self) {
        let n = self.filtered_items().len();
        if n == 0 { return; }
        self.cursor = (self.cursor + 1).min(n - 1);
    }

    pub fn move_cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
}

impl Widget for &ScrollableList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::palette::*;

        let filtered = self.filtered_items();
        let display_rows = area.height as usize;

        // Ensure scroll_offset keeps cursor visible
        // (scroll_offset managed by parent on cursor move; widget just uses it)
        for (row, (orig_idx, item)) in filtered.iter()
            .skip(self.scroll_offset)
            .take(display_rows)
            .enumerate()
        {
            let y = area.y + row as u16;
            let is_cursor = (self.scroll_offset + row) == self.cursor;

            let prefix = if is_cursor { "▶ " } else { "  " };
            let line: String = format!("{}{}", prefix, item)
                .chars().take(area.width as usize).collect();

            buf.set_string(area.x, y, &line, Style::default()
                .fg(if is_cursor { BONE } else { TEXT_PRIMARY })
                .bg(if is_cursor { BG_RAISED } else { BG_VOID })
                .add_modifier(if is_cursor { Modifier::BOLD } else { Modifier::empty() })
            );
        }
    }
}
```

**KeyHelpOverlay:**

A floating centered box. Rendered on top of whatever is below it. Width is determined by the longest key+description pair plus padding. Toggle with `?`. `visible: bool` is checked by the parent screen — if `false`, the widget renders nothing.

```rust
pub struct KeyBinding {
    pub key: String,
    pub description: String,
}

pub struct KeyHelpOverlay {
    pub bindings: Vec<KeyBinding>,
    pub visible: bool,
}

impl Widget for &KeyHelpOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::palette::*;

        if !self.visible || area.width < 20 || area.height < 5 { return; }

        // Compute box dimensions
        let max_key_len = self.bindings.iter()
            .map(|b| b.key.len()).max().unwrap_or(1);
        let max_desc_len = self.bindings.iter()
            .map(|b| b.description.len()).max().unwrap_or(10);
        let inner_width = (max_key_len + max_desc_len + 5).min(area.width as usize - 4);
        let inner_height = self.bindings.len().min(area.height as usize - 4);
        let box_width  = (inner_width  + 4) as u16;
        let box_height = (inner_height + 2) as u16;

        // Center the box
        let box_x = area.x + area.width.saturating_sub(box_width) / 2;
        let box_y = area.y + area.height.saturating_sub(box_height) / 2;

        // Draw border
        let border_style = Style::default().fg(BORDER_ACTIVE).bg(BG_RAISED);
        let inner_style  = Style::default().fg(TEXT_PRIMARY).bg(BG_RAISED);

        // Top row
        if let Some(cell) = buf.cell_mut((box_x, box_y)) {
            cell.set_char('┌'); cell.set_style(border_style);
        }
        for x in (box_x + 1)..(box_x + box_width - 1) {
            if let Some(cell) = buf.cell_mut((x, box_y)) {
                cell.set_char('─'); cell.set_style(border_style);
            }
        }
        if let Some(cell) = buf.cell_mut((box_x + box_width - 1, box_y)) {
            cell.set_char('┐'); cell.set_style(border_style);
        }

        // Title in top border
        let title = " KEY BINDINGS ";
        buf.set_string(box_x + 2, box_y, title,
            Style::default().fg(BONE).bg(BG_RAISED));

        // Body rows
        for (row, binding) in self.bindings.iter().take(inner_height).enumerate() {
            let y = box_y + 1 + row as u16;
            // Left border
            if let Some(cell) = buf.cell_mut((box_x, y)) {
                cell.set_char('│'); cell.set_style(border_style);
            }
            // Fill background
            for x in (box_x + 1)..(box_x + box_width - 1) {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' '); cell.set_style(inner_style);
                }
            }
            // Key (bold, rose)
            let key_display = format!("{:<width$}", binding.key, width = max_key_len);
            buf.set_string(box_x + 2, y, &key_display,
                Style::default().fg(ROSE).bg(BG_RAISED).add_modifier(Modifier::BOLD));
            // Separator
            buf.set_string(box_x + 2 + max_key_len as u16, y, " — ",
                Style::default().fg(TEXT_DIM).bg(BG_RAISED));
            // Description
            let desc: String = binding.description.chars()
                .take(max_desc_len).collect();
            buf.set_string(box_x + 2 + max_key_len as u16 + 3, y, &desc,
                Style::default().fg(TEXT_PRIMARY).bg(BG_RAISED));
            // Right border
            if let Some(cell) = buf.cell_mut((box_x + box_width - 1, y)) {
                cell.set_char('│'); cell.set_style(border_style);
            }
        }

        // Bottom row
        let bottom_y = box_y + box_height - 1;
        if let Some(cell) = buf.cell_mut((box_x, bottom_y)) {
            cell.set_char('└'); cell.set_style(border_style);
        }
        for x in (box_x + 1)..(box_x + box_width - 1) {
            if let Some(cell) = buf.cell_mut((x, bottom_y)) {
                cell.set_char('─'); cell.set_style(border_style);
            }
        }
        if let Some(cell) = buf.cell_mut((box_x + box_width - 1, bottom_y)) {
            cell.set_char('┘'); cell.set_style(border_style);
        }
    }
}
```

**`src/widgets/mod.rs`:**

```rust
pub mod sparkline;
pub mod gauge;
pub mod heatmap;
pub mod timeline;
pub mod feed;
pub mod tabs;
pub mod status_bar;
pub mod scrolllist;
pub mod key_help;

pub use sparkline::BrailleSparkline;
pub use gauge::{VitalityGauge, ConfidenceGauge, AccuracyGauge, MockPhase};
pub use heatmap::{PheromoneHeatmap, PheromoneLayer};
pub use timeline::{TimelineRibbon, TimelineEvent, RibbonEventType};
pub use feed::{EventFeed, FeedEntry, FeedLevel};
pub use tabs::TabBar;
pub use status_bar::StatusBar;
pub use scrolllist::ScrollableList;
pub use key_help::{KeyHelpOverlay, KeyBinding};
```

---

## Failure Recovery

**`char::from_u32` returns `None` on bad braille codepoint:**
- The expression `0x2800 + bits as u32` can only produce values in U+2800..=U+28FF. All are valid Unicode. The `.unwrap_or(' ')` fallback is a safety net, not an expected code path. If it triggers, the `bits` computation overflowed — check that `left_bits` and `right_bits` are not producing values > 0xFF combined.

**`buf.cell_mut((x, y))` returns `None` at boundary:**
- ratatui returns `None` for coordinates outside the buffer rect. Always guard with `if let Some(cell) = buf.cell_mut(...)`. Never index directly.

**`PheromoneHeatmap` grid dimensions don't match `width`/`height` fields:**
- The render loop uses `self.grid.len()` and `self.grid[0].len()` as the actual dimensions, clamped to `area.height` and `area.width`. The `width`/`height` fields on the struct are hints for layout sizing, not enforced. Mismatched fields produce a smaller rendered grid, not a panic.

**`KeyHelpOverlay` box extends beyond `area` bounds:**
- The box dimensions are clamped to `area.width - 4` and `area.height - 4`. If `area` is smaller than 20x5 the widget returns early without rendering. Screens must allocate at least that much area for the overlay.

**`ScrollableList` cursor out of bounds after filter change:**
- When `filter` changes, the filtered item count may drop below `cursor`. The `filtered_items()` method does not clamp `cursor` — the parent screen must call `move_cursor_up()` / reset cursor to 0 when filter text changes. Add this to the screen's `handle_key` implementation.

**`EventFeed` scroll offset becomes stale after new entries push old ones out:**
- `push()` does not adjust `scroll_offset`. If the user is scrolled back 50 lines and 100 new entries arrive, their view shifts. This is acceptable at scaffold stage — Plans 70a-70c will add proper scroll anchoring. For now, pressing the End key (handled by the parent screen) resets `scroll_offset` to 0.

**`cargo check` fails: `Widget` trait bound not satisfied:**
- Ensure each struct derives nothing that conflicts with the `Widget` implementation. The `Widget` trait in ratatui 0.30 requires `fn render(self, area: Rect, buf: &mut Buffer)` consuming `self`. For `EventFeed` and `ScrollableList`, which must not be consumed, implement `Widget for &EventFeed` and `Widget for &ScrollableList` instead of consuming variants.

**Braille characters not rendering correctly in some terminals:**
- Braille Unicode support requires a terminal font with the Braille Patterns block (U+2800–U+28FF). Common fonts with support: JetBrains Mono, Nerd Fonts, Fira Code. If dots appear as boxes or question marks, this is a font issue, not a code bug. The sparkline widget is correct; note this in the testing checkpoint output.

---

## Testing Checkpoint

```bash
# Compile check — must have zero errors
cargo check -p bardo-terminal

# Run widget unit tests
cargo test -p bardo-terminal -- widgets --nocapture
```

**Expected test output:**

```
test widgets::sparkline::tests::test_braille_sparkline_encodes_correctly ... ok
test widgets::gauge::tests::test_vitality_gauge_colors ... ok
test widgets::gauge::tests::test_confidence_color_thresholds ... ok
test widgets::feed::tests::test_event_feed_scroll ... ok
```

**Manual integration check:**

To verify widgets render correctly, temporarily add widget exercise code to `HomeScreen::render`:

```rust
// In src/screens/home.rs, after existing render code:
// Exercise the sparkline in a 1-row area at the bottom of the creature panel
use crate::widgets::BrailleSparkline;
let sparkline_area = Rect { x: area.x, y: area.y + area.height - 1,
    width: 20, height: 1 };
let sparkline = BrailleSparkline {
    data: (0..40).map(|i| (i as f64 / 40.0 * std::f64::consts::PI * 2.0).sin().abs())
        .collect(),
    max_value: 1.0,
    color: crate::palette::ROSE,
    label: None,
};
sparkline.render(sparkline_area, frame.buffer_mut());
```

Run `cargo run -p bardo-terminal` and verify the braille sparkline appears as a sine wave of braille dots at the bottom of the creature panel. Remove the exercise code before committing.

**Additional checks:**
1. `BrailleSparkline` with 80 data points in a 40-col area renders without index out of bounds
2. `VitalityGauge` at `value = 0.0` shows an empty bar (no filled cells)
3. `EventFeed` with 1200 entries retains only 1000 (the max)
4. `KeyHelpOverlay` with `visible = false` renders nothing (no stray characters in buffer)
5. `TabBar` with active index 0 shows first tab highlighted in BONE, others in TEXT_DIM

---

## Completion Report

*(Codex fills this in after implementation. Include: ratatui API differences encountered in 0.30 vs the plan's assumptions, any braille encoding corrections, manual render test results, and whether all four unit tests passed on first compilation attempt.)*

## Verification

### Invariants

<!-- INV-001: Braille dot encoding bit positions -->
- **type**: formula
- **module**: `bardo_terminal::widgets::sparkline`
- **property**: Braille characters encode 2×4 grid using standardized bit positions
- **formula**: Left column: bits [0x40, 0x04, 0x02, 0x01] for rows 3→0; Right column: bits [0x80, 0x20, 0x10, 0x08] for rows 3→0
- **constraint**: Exactly 8 bits per character; Unicode mapping U+2800 + bit_flags produces valid braille
- **test_fn**: `test_braille_sparkline_encodes_correctly`
- **strategy**: unit
- **inputs**: `{"n_dots": [0, 1, 2, 3, 4], "side": ["left", "right"]}`
- **oracle**: `left_bits(0) = 0x00`, `left_bits(4) = 0x01 | 0x02 | 0x04 | 0x40 = 0x47`, all 8 bits lit = 0xFF = U+28FF
- **severity**: spec
- **source**: Plan 05 §Unit 1 Quick Reference — Braille dot encoding

<!-- INV-002: Sparkline data point scaling -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::sparkline`
- **property**: Data points scale to 0..=4 dots per column, clamped at boundaries
- **formula**: `left_dots = ((left_val / max) * 4.0).round().clamp(0.0, 4.0) as usize`
- **constraint**: Output range [0..=4]; clamping applies to both min and max
- **test_fn**: `test_sparkline_scaling_bounds`
- **strategy**: proptest
- **inputs**: `{"left_val": [0.0, 0.25, 0.5, 0.75, 1.0, 1.5, f64::INFINITY, f64::NAN], "max": [0.0, 1.0, 100.0]}`
- **oracle**: `clamp(round((v/m)*4.0)) in [0.0, 4.0]` for all (v, m)
- **severity**: spec
- **source**: Plan 05 §Unit 1 BrailleSparkline render logic

<!-- INV-003: Max value auto-scaling for sparkline -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::sparkline`
- **property**: When max_value ≤ 0, auto-scale from data.max(); otherwise use max_value
- **formula**: `let max = if self.max_value > 0.0 { self.max_value } else { self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(1.0) }`
- **constraint**: Final max ≥ 1.0; never divides by zero
- **test_fn**: `test_sparkline_max_value_auto_scale`
- **strategy**: unit
- **inputs**: `{"max_value": [0.0, -1.5, 100.0], "data": [[], [0.5], [1.0, 2.5]]}`
- **oracle**: empty data → max = 1.0; non-empty, max_value=0 → max ≥ min(data); max_value > 0 → max = max_value
- **severity**: spec
- **source**: Plan 05 §Unit 1 BrailleSparkline render logic

<!-- INV-004: Vitality gauge fill width scaling -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::gauge`
- **property**: Fill width scales with gauge value, clamped to terminal width
- **formula**: `let fill_width = ((self.value.clamp(0.0, 1.0)) * area.width as f64) as u16`
- **constraint**: fill_width ∈ [0, area.width]; value always clamped first
- **test_fn**: `test_vitality_gauge_fill_width`
- **strategy**: proptest
- **inputs**: `{"value": [0.0, 0.25, 0.5, 0.75, 1.0, -0.5, 1.5], "area_width": [1, 10, 80, 255]}`
- **oracle**: `fill_width = floor(clamp(value, 0, 1) * width)`; result ≤ width
- **severity**: spec
- **source**: Plan 05 §Unit 1 VitalityGauge render logic

<!-- INV-005: MockPhase color assignment -->
- **type**: state_machine
- **module**: `bardo_terminal::widgets::gauge`
- **property**: Each MockPhase maps to a unique color; all 5 phases are defined
- **formula**: `match self { Thriving → SUCCESS, Stable → Rgb(88,160,170), Conservation → WARNING, Declining → Rgb(180,100,60), Terminal → ROSE_BRIGHT }`
- **constraint**: All 5 variants must have a color; no unreachable arms
- **test_fn**: `test_vitality_gauge_colors`
- **strategy**: unit
- **inputs**: `{"phase": ["Thriving", "Stable", "Conservation", "Declining", "Terminal"]}`
- **oracle**: Each phase returns a Color; no panics; colors are distinct
- **severity**: spec
- **source**: Plan 05 §Unit 1 MockPhase::gauge_color

<!-- INV-006: Confidence threshold bands -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::gauge`
- **property**: Confidence color thresholds partition [0.0, 1.0] into 4 bands
- **formula**: `if value ≥ 0.75 { SUCCESS } else if value ≥ 0.50 { Cyan } else if value ≥ 0.25 { WARNING } else { ROSE_DIM }`
- **constraint**: Thresholds are [0.75, 0.50, 0.25]; bands are disjoint; all possible values covered
- **test_fn**: `test_confidence_color_thresholds`
- **strategy**: unit
- **inputs**: `{"value": [0.0, 0.10, 0.25, 0.30, 0.50, 0.60, 0.75, 0.95, 1.0]}`
- **oracle**: value ≥ 0.75 → SUCCESS; 0.50 ≤ v < 0.75 → Cyan; 0.25 ≤ v < 0.50 → WARNING; v < 0.25 → ROSE_DIM
- **severity**: spec
- **source**: Plan 05 §Unit 1 confidence_color and AccuracyGauge

<!-- INV-007: Viridis color stop indices -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::heatmap`
- **property**: 7-stop Viridis colormap indices clamp to valid range [0, 6]
- **formula**: `let idx = (v * 6.0).floor() as usize; let idx_min6 = idx.min(6)`
- **constraint**: Index always ≤ 6; boundary access [(idx + 1).min(6)] never exceeds array
- **test_fn**: `test_viridis_index_bounds`
- **strategy**: proptest
- **inputs**: `{"value": [0.0, 0.0833, 0.1667, 0.333, 0.5, 0.667, 0.833, 1.0]}`
- **oracle**: v ∈ [0, 1] → idx ∈ [0, 6]; v < 0 or v > 1 → clamp prevents out-of-bounds
- **severity**: spec
- **source**: Plan 05 §Unit 2 viridis_color function

<!-- INV-008: Viridis color interpolation -->
- **type**: formula
- **module**: `bardo_terminal::widgets::heatmap`
- **property**: Linear interpolation between adjacent Viridis stops
- **formula**: `let t = (v * 6.0) - idx as f64; let lerp_component = |a: u8, b: u8, t: f64| (a as f64 + (b as f64 - a as f64) * t).round() as u8`
- **constraint**: t ∈ [0.0, 1.0] for interpolation; output always u8 (0..255)
- **test_fn**: `test_viridis_color_interpolation`
- **strategy**: unit
- **inputs**: `{"value": [0.0, 0.1, 0.5, 0.9, 1.0]}`
- **oracle**: value = 0 → VIRIDIS[0] exactly; value = 1 → VIRIDIS[6] exactly; intermediate values interpolated
- **severity**: spec
- **source**: Plan 05 §Unit 2 viridis_color function

<!-- INV-009: Pheromone layer tinting -->
- **type**: formula
- **module**: `bardo_terminal::widgets::heatmap`
- **property**: Threat layer adds red bias; Wisdom layer blends toward dream indigo at low values
- **formula**: Threat: `Rgb(r+40, g-20, b-20)`; Wisdom: `t = (1.0 - value).clamp(0.0, 0.5)`, blend with (88, 88, 120)
- **constraint**: Threat component addition/subtraction saturates; Wisdom blend factor t ∈ [0.0, 0.5]
- **test_fn**: `test_pheromone_layer_tinting`
- **strategy**: unit
- **inputs**: `{"layer": ["Threat", "Opportunity", "Wisdom"], "value": [0.0, 0.5, 1.0]}`
- **oracle**: Threat: R +40 (capped at 255), G -20 (floored at 0), B -20 (floored at 0); Wisdom: t ≤ 0.5 always
- **severity**: spec
- **source**: Plan 05 §Unit 2 layer_tint function

<!-- INV-010: Heatmap grid bounds -->
- **type**: capacity
- **module**: `bardo_terminal::widgets::heatmap`
- **property**: Heatmap render respects terminal area; grid access clamps to actual dimensions
- **formula**: `let rows = self.grid.len().min(area.height as usize); let cols = self.grid[0].len().min(area.width as usize)`
- **constraint**: rows ≤ grid.len(); cols ≤ grid[0].len(); rows ≤ area.height; cols ≤ area.width
- **test_fn**: `test_heatmap_grid_clipping`
- **strategy**: proptest
- **inputs**: `{"grid_height": [1, 10, 100], "grid_width": [1, 10, 100], "area_height": [1, 50], "area_width": [1, 50]}`
- **oracle**: rendered rows ≤ min(grid.len(), area.height); rendered cols ≤ min(grid[0].len(), area.width)
- **severity**: code
- **source**: Plan 05 §Unit 2 PheromoneHeatmap render logic

<!-- INV-011: Timeline ribbon tick-to-cell mapping -->
- **type**: formula
- **module**: `bardo_terminal::widgets::timeline`
- **property**: Ticks-per-cell calculation never divides by zero; maps time window to terminal width
- **formula**: `let ticks_per_cell = self.window_ticks.max(1) / w.max(1)`
- **constraint**: Both numerator and denominator ≥ 1; result ≥ 1 when window_ticks ≥ width
- **test_fn**: `test_timeline_ribbon_ticks_per_cell`
- **strategy**: proptest
- **inputs**: `{"window_ticks": [0, 1, 100, 10000], "area_width": [0, 1, 40, 200]}`
- **oracle**: Never panics; ticks_per_cell ≥ 1; cell_tick range never negative
- **severity**: spec
- **source**: Plan 05 §Unit 2 TimelineRibbon render logic

<!-- INV-012: Timeline event cell assignment -->
- **type**: formula
- **module**: `bardo_terminal::widgets::timeline`
- **property**: Events are placed in ribbon cells based on tick range overlap
- **formula**: `event_in_cell: event.tick >= cell_start_tick && event.tick < cell_start_tick + ticks_per_cell.max(1)`
- **constraint**: Tick ranges are half-open [start, end); no event appears in two cells; latest event (by severity) chosen for each cell
- **test_fn**: `test_timeline_ribbon_event_placement`
- **strategy**: unit
- **inputs**: `{"event_ticks": [0, 100, 500, 1000], "window": {"start": 0, "width": 100}, "cell_width": 10}`
- **oracle**: Each event in exactly one cell; cells with multiple events show max-severity
- **severity**: spec
- **source**: Plan 05 §Unit 2 TimelineRibbon render logic

<!-- INV-013: RibbonEventType glyph mapping -->
- **type**: state_machine
- **module**: `bardo_terminal::widgets::timeline`
- **property**: All 5 event types have unique glyphs and colors
- **formula**: `match self { TradeExecuted → '▲', DreamStarted → '◌', PhaseChange → '◆', Anomaly → '!', Death → '✕' }`
- **constraint**: Each variant appears exactly once; color function covers all cases
- **test_fn**: `test_ribbon_event_type_mapping`
- **strategy**: unit
- **inputs**: `{"event_type": ["TradeExecuted", "DreamStarted", "PhaseChange", "Anomaly", "Death"]}`
- **oracle**: Each type returns unique glyph; each type returns a Color; no panics
- **severity**: spec
- **source**: Plan 05 §Unit 2 RibbonEventType impl

<!-- INV-014: Event feed ring buffer capacity -->
- **type**: capacity
- **module**: `bardo_terminal::widgets::feed`
- **property**: EventFeed maintains max_entries limit via FIFO eviction
- **formula**: `if self.entries.len() >= self.max_entries { self.entries.pop_front(); self.entries.push_back(entry); }`
- **constraint**: entries.len() ≤ max_entries always; oldest entry dropped on overflow; VecDeque capacity ≤ 1000 or max_entries
- **test_fn**: `test_event_feed_capacity`
- **strategy**: unit
- **inputs**: `{"max_entries": [1, 10, 100, 1000], "inserts": [5, 11, 1001]}`
- **oracle**: After N inserts with N > max_entries, len = min(max_entries, N); first entries dropped
- **severity**: spec
- **source**: Plan 05 §Unit 3 EventFeed::push and Quick Reference

<!-- INV-015: Event feed filtering -->
- **type**: formula
- **module**: `bardo_terminal::widgets::feed`
- **property**: Filter applies case-insensitive substring matching
- **formula**: `filter_lower.map(|f| e.message.to_lowercase().contains(f)).unwrap_or(true)`
- **constraint**: No filter (None) matches all entries; empty filter string matches all; search is case-insensitive
- **test_fn**: `test_event_feed_filter`
- **strategy**: unit
- **inputs**: `{"filter": [null, "", "msg", "MSG", "xyz"], "message": ["msg 6", "MSG 7", "other"]}`
- **oracle**: null filter or empty → all entries; "msg" → matches "msg 6", "MSG 7"; "xyz" → no matches (case-insensitive)
- **severity**: spec
- **source**: Plan 05 §Unit 3 EventFeed::visible_entries

<!-- INV-016: Event feed scroll bounds -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::feed`
- **property**: Scroll offset never exceeds visible entries; always shows newest when scrolled to top
- **formula**: `let start_idx = self.scroll_offset.min(total.saturating_sub(display_rows))`
- **constraint**: start_idx ≤ total - display_rows (or 0 if total < display_rows); scrolling up is bounded
- **test_fn**: `test_event_feed_scroll_bounds`
- **strategy**: proptest
- **inputs**: `{"total_entries": [0, 10, 100], "display_rows": [1, 5, 50], "scroll_offset": [0, 5, 100, 1000]}`
- **oracle**: start_idx + display_rows ≤ total; start_idx ≥ 0; start_idx ≤ total.saturating_sub(1)
- **severity**: spec
- **source**: Plan 05 §Unit 3 EventFeed render logic

<!-- INV-017: FeedLevel color assignment -->
- **type**: state_machine
- **module**: `bardo_terminal::widgets::feed`
- **property**: Each FeedLevel has a unique color and label
- **formula**: `match self { Info → TEXT_PRIMARY, Warn → WARNING, Error → ROSE_BRIGHT, Debug → TEXT_DIM }` and labels ["INFO ", "WARN ", "ERROR", "DBG  "]
- **constraint**: All 4 variants covered; colors distinct; labels exactly 5 chars each
- **test_fn**: `test_feed_level_colors_and_labels`
- **strategy**: unit
- **inputs**: `{"level": ["Info", "Warn", "Error", "Debug"]}`
- **oracle**: Each level returns unique Color and label; labels are 5 chars; no panics
- **severity**: spec
- **source**: Plan 05 §Unit 3 FeedLevel impl

<!-- INV-018: StatusBar layout proportions -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::status_bar`
- **property**: Center section positioned at 1/3 of width; right section right-aligned
- **formula**: `let center_x = area.x + area.width / 3; let right_x = area.x + area.width.saturating_sub(right_len as u16)`
- **constraint**: center_x ∈ [area.x, area.x + area.width]; right_x ≥ center_x; right_x ≥ area.x
- **test_fn**: `test_status_bar_layout`
- **strategy**: proptest
- **inputs**: `{"area_width": [1, 40, 100, 255], "right_len": [0, 20, 50]}`
- **oracle**: center_x = area.x + width/3; right_x ≥ area.x; center section + right section fit without overlap
- **severity**: code
- **source**: Plan 05 §Unit 3 StatusBar render logic

<!-- INV-019: TabBar active index bounds -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::tabs`
- **property**: Active tab index must be valid; comparison uses ==
- **formula**: `let is_active = i == self.active; (0..tabs.len()).contains(self.active)`
- **constraint**: active < tabs.len(); fallback if active >= len requires parent validation
- **test_fn**: `test_tab_bar_active_bounds`
- **strategy**: unit
- **inputs**: `{"tabs": [[], ["home"], ["a", "b", "c"]], "active": [0, 1, 2, 999]}`
- **oracle**: is_active = true only when i == active; parent must ensure active < tabs.len()
- **severity**: code
- **source**: Plan 05 §Unit 4 TabBar render logic

<!-- INV-020: Tab label truncation -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::tabs`
- **property**: Tab labels don't overflow terminal width via early break
- **formula**: `if x + label.len() as u16 > area.x + area.width { break; }`
- **constraint**: Loop exits when next tab would exceed right edge; partial tabs never rendered
- **test_fn**: `test_tab_bar_width_truncation`
- **strategy**: proptest
- **inputs**: `{"area_width": [10, 40, 80], "tabs": [["short"], ["medium-length"], ["very-long-tab-name"]]}`
- **oracle**: Rendered tabs fit in area_width; last visible tab ends ≤ area.x + area.width; no partial tabs
- **severity**: code
- **source**: Plan 05 §Unit 4 TabBar render logic

<!-- INV-021: ScrollableList cursor bounds -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::scrolllist`
- **property**: Cursor never exceeds filtered item count; down/up are bounded
- **formula**: `self.cursor = (self.cursor + 1).min(n - 1)` (down), `self.cursor = self.cursor.saturating_sub(1)` (up)
- **constraint**: cursor ∈ [0, n-1] where n = filtered_items().len(); empty list → no move allowed
- **test_fn**: `test_scrollable_list_cursor_bounds`
- **strategy**: proptest
- **inputs**: `{"items": [[], ["a"], ["a", "b", "c"]], "cursor": [0, 1, 2, 999]}`
- **oracle**: After move_down, cursor < n; after move_up, cursor ≥ 0; empty list: cursor unchanged
- **severity**: code
- **source**: Plan 05 §Unit 4 ScrollableList methods

<!-- INV-022: ScrollableList filtering -->
- **type**: formula
- **module**: `bardo_terminal::widgets::scrolllist`
- **property**: Filtering applies case-insensitive substring match; order preserved
- **formula**: `filtered: items where filter_lower.map(|f| item.to_lowercase().contains(f)).unwrap_or(true)`
- **constraint**: No filter → all items; case-insensitive; original indices preserved in output
- **test_fn**: `test_scrollable_list_filter`
- **strategy**: unit
- **inputs**: `{"items": ["apple", "Apricot", "banana"], "filter": [null, "", "ap", "AP", "xyz"]}`
- **oracle**: null → ["apple", "Apricot", "banana"]; "ap" → ["apple", "Apricot"] (case-insensitive); "xyz" → []
- **severity**: code
- **source**: Plan 05 §Unit 4 ScrollableList::filtered_items

<!-- INV-023: KeyHelpOverlay dimensions -->
- **type**: numeric_range
- **module**: `bardo_terminal::widgets::key_help`
- **property**: Box dimensions never exceed area; minimum area 20x5 required
- **formula**: `inner_width = (max_key_len + max_desc_len + 5).min(area.width as usize - 4); inner_height = bindings.len().min(area.height as usize - 4); box_width = (inner_width + 4) as u16; box_height = (inner_height + 2) as u16`
- **constraint**: box_width ≤ area.width; box_height ≤ area.height; area < 20x5 → no render (early return)
- **test_fn**: `test_key_help_overlay_dimensions`
- **strategy**: proptest
- **inputs**: `{"area_width": [10, 20, 80, 255], "area_height": [3, 5, 40, 100], "bindings_count": [1, 5, 20]}`
- **oracle**: visible area < 20x5 → returns early; box dimensions fit within area; centered position never negative
- **severity**: code
- **source**: Plan 05 §Unit 4 KeyHelpOverlay render logic

<!-- INV-024: KeyHelpOverlay centering -->
- **type**: formula
- **module**: `bardo_terminal::widgets::key_help`
- **property**: Box is centered horizontally and vertically
- **formula**: `let box_x = area.x + area.width.saturating_sub(box_width) / 2; let box_y = area.y + area.height.saturating_sub(box_height) / 2`
- **constraint**: box_x ≥ area.x; box_y ≥ area.y; box_x + box_width ≤ area.x + area.width; saturating_sub prevents underflow
- **test_fn**: `test_key_help_overlay_centering`
- **strategy**: proptest
- **inputs**: `{"area_x": [0, 10], "area_y": [0, 5], "area_width": [40, 100], "area_height": [20, 50], "box_width": [10, 30], "box_height": [5, 15]}`
- **oracle**: box_x in [area.x, area.x + (area.width - box_width)/2]; box_y similarly centered
- **severity**: code
- **source**: Plan 05 §Unit 4 KeyHelpOverlay render logic

<!-- INV-025: Braille sparkline data capacity -->
- **type**: capacity
- **module**: `bardo_terminal::widgets::sparkline`
- **property**: Sparkline capacity is 2 data points per terminal cell (80 points in 40 columns)
- **formula**: `let data_capacity = cell_count * 2; let n = self.data.len().min(data_capacity); let offset = self.data.len().saturating_sub(data_capacity)`
- **constraint**: Only latest data_capacity points rendered; oldest points dropped; offset ≥ 0
- **test_fn**: `test_braille_sparkline_capacity`
- **strategy**: proptest
- **inputs**: `{"data_len": [0, 40, 80, 160, 1000], "cell_count": [1, 20, 40, 80]}`
- **oracle**: Rendered points ≤ data_capacity; offset + n = data_len; newest data always at right edge
- **severity**: spec
- **source**: Plan 05 §Unit 1 BrailleSparkline render logic

---

### Regression Anchors

`test_braille_sparkline_encodes_correctly`
`test_sparkline_scaling_bounds`
`test_sparkline_max_value_auto_scale`
`test_vitality_gauge_fill_width`
`test_vitality_gauge_colors`
`test_confidence_color_thresholds`
`test_viridis_index_bounds`
`test_viridis_color_interpolation`
`test_pheromone_layer_tinting`
`test_heatmap_grid_clipping`
`test_timeline_ribbon_ticks_per_cell`
`test_timeline_ribbon_event_placement`
`test_ribbon_event_type_mapping`
`test_event_feed_capacity`
`test_event_feed_filter`
`test_event_feed_scroll_bounds`
`test_feed_level_colors_and_labels`
`test_status_bar_layout`
`test_tab_bar_active_bounds`
`test_tab_bar_width_truncation`
`test_scrollable_list_cursor_bounds`
`test_scrollable_list_filter`
`test_key_help_overlay_dimensions`
`test_key_help_overlay_centering`
`test_braille_sparkline_capacity`

---

### Cross-Crate Contracts

| Upstream | Input Condition | Expected Behavior |
|----------|----------------|-------------------|
| `bardo_terminal::palette` | Color constants (SUCCESS, WARNING, ROSE_DIM, etc.) | Widgets import and use in match statements; all referenced constants must exist and be Color type |
| `bardo_terminal::state::AppState` | Screen implementations read AppState | MockPhase, PAD summary, credit balance, tick number available for rendering (TODO Plan 70a connections) |
| `ratatui::Buffer` | Cell access via `(x, y)` coordinates | `buf.cell_mut()` returns Option<&mut Cell>; widget checks bounds before write |
| `ratatui::Rect` | Area dimensions (x, y, width, height) | width/height of 0 triggers early returns; all widgets clamp render to area bounds |
| `std::collections::VecDeque` | EventFeed.entries | `push_back()`, `pop_front()`, `iter()`, `len()` all work; capacity managed manually |

---

### Event Sequence Assertions

**EventFeed ring buffer eviction (FIFO):**
1. Feed initialized with max_entries=5
2. Push entries 0..4 → entries.len() == 5
3. Push entry 5 → pop_front (entry 0) then push_back (entry 5) → entries.len() == 5
4. Back element tick == 5; front element tick == 1
5. Verify: oldest entry removed; newest appended; length stable

**Timeline ribbon event selection (max severity per cell):**
1. Cell [50, 100) ticks has 2 events: TradeExecuted (severity=10), DreamStarted (severity=50)
2. Render selects max_by_key(|e| e.severity)
3. DreamStarted displayed (glyph '◌' with DREAM color)
4. Verify: cell shows only one event; highest severity wins

**TabBar overflow truncation:**
1. Area width 30, tabs ["Home", "Beat", "Mind", "Dreams"]
2. Each label ~7 chars with padding; first 4 don't fit
3. Render loop breaks when next tab would overflow
4. Verify: partial tabs never appear; last visible tab ends ≤ 30

---

### Academic References Verified

| Reference | Formula/Constant | PRD2 Match | Web-Verified |
|-----------|-----------------|------------|--------------|
| Unicode Braille (U+2800 block) | 8-dot grid encoding, bit positions 0-7 | prd2/20-styx/05-tui-experience.md §3 mentions "2×4 braille grid" | Standard; no formula adjustments needed |
| Viridis colormap | 7-stop RGB gradient from dark purple (68,1,84) to yellow (253,231,37) | prd2/20-styx/05-tui-experience.md §5 references "Viridis-like" gradient | Standard colormap; RGB stops verified against Viridis v2.0 |
| PAD emotional model (Pleasure/Arousal/Dominance) | StatusBar references "P:0.3 A:0.1 D:0.4" format | prd2/03-daimon/ and prd2/20-styx/05-tui-experience.md §3 "PAD vector" | Russell 1980 "A Circumplex Model of Affect"; 3-dimensional space standard |
| REM sleep rendering (Tier 1) | Dream phase renders "at sparse Tier 1, dream palette" | prd2/13-runtime/14-creature-system.md references "Tier 1" during REM | Tier terminology consistent with sprite resolution tiers; no formula |

---

**Notes:**
- Plan 05 focuses on terminal widget rendering with no heavy computational formulas from mortality/economics models
- All numeric ranges derive from terminal cell coordinates (u16: 0..65535) and f64 scales (0.0..1.0)
- State machines (MockPhase, FeedLevel, RibbonEventType, PheromoneLayer) have exhaustive pattern matching
- Capacity constraints driven by VecDeque (EventFeed) and grid dimensions (Heatmap)
- Cross-crate contracts are shallow: palette constants, AppState read-only access, ratatui Buffer/Rect API
- PAD references are forward-looking (TODO Plan 70a) and inherit from prd2/03-daimon documentation
