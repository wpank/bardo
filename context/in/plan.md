# Plan 04: Terminal Scaffold & Render Loop

## Context

`bardo-terminal` is the Golem's primary interactive surface: a full-screen TUI written in Rust, rendered at 60fps via ratatui and crossterm. This plan builds the application skeleton — terminal initialization, the render loop, the screen abstraction layer, the ROSEDUST color palette, and the responsive layout engine — without connecting any runtime data. Placeholder types stub out vitality, PAD, and heartbeat stats so the scaffold compiles and runs in isolation. Plans 70a-70c wire the real connections.

The design target is `prd2/18-interfaces/rendering/00-design-system.md`: rose on violet-black, CRT materiality, nothing ever at rest. Even this placeholder scaffold should feel alive: a sine-wave tick counter and a pulsing border are enough to demonstrate the perpetual motion principle.

## Previous Plan

Plan 03 created `apps/mirage-rs/`: in-process revm fork with lazy-latest RPC, CoW state layers, JSON-RPC compat, and a scenario runner. mirage-rs is the EVM testing backbone for all subsequent on-chain simulation.

## Prerequisites

- **Plan 01** — workspace scaffold; `apps/bardo-terminal/` must exist as a workspace member with its `Cargo.toml` included in the root `[workspace.members]`
- **Plan 02** — `golem-core` crate compiled; `EventFabric` is imported optionally (unused until Plan 70a, but the import confirms the dependency compiles)

## Imports

```rust
// In app.rs — optional import to confirm golem-core dependency resolves
#[allow(unused_imports)]
use golem_core::event::EventFabric;
```

The dependency is declared in `Cargo.toml` but not used at runtime until Plans 70a-70c. The `#[allow(unused_imports)]` suppresses the warning without removing the import.

## Exports

| Type | Module | Purpose |
|------|--------|---------|
| `Screen` | `bardo_terminal::screen` | Pluggable screen trait — all 29 screens implement this |
| `ScreenId` | `bardo_terminal::screen` | Enum identifying each of the 29 screens |
| `ScreenRegistry` | `bardo_terminal::screen` | `HashMap<ScreenId, Box<dyn Screen>>` |
| `App` | `bardo_terminal::app` | Application struct — owns terminal, screens, state, event channel |
| `AppState` | `bardo_terminal::state` | Global app state (tick count, connection status, vitality placeholder, layout) |
| `AppAction` | `bardo_terminal::state` | Enum of actions that can mutate app state (Quit, NextScreen, PrevScreen, Resize) |
| `ColorPalette` | `bardo_terminal::palette` | ROSEDUST palette constants as `ratatui::style::Color::Rgb` values |
| `LayoutBreakpoint` | `bardo_terminal::layout` | Responsive breakpoint enum (Compact / Standard / Wide / Ultra) |
| `MockVitality` | `bardo_terminal::state` | Placeholder vitality struct until Plan 70a |
| `ConnectionStatus` | `bardo_terminal::state` | Enum: Connected / Disconnected / Connecting |
| `HomeScreen` | `bardo_terminal::screens::home` | Home screen placeholder (implements `Screen`) |

## Cargo Dependencies

```toml
# apps/bardo-terminal/Cargo.toml
[package]
name = "bardo-terminal"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "bardo-terminal"
path = "src/main.rs"

[dependencies]
ratatui = { workspace = true }
crossterm = { workspace = true }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
color-eyre = "0.6"
golem-core = { path = "../../crates/golem-core" }
```

Notes:
- `ratatui = { workspace = true }` — workspace pins `"0.30"`; an earlier draft referenced `"0.29"` but the workspace is authoritative
- `crossterm = { workspace = true }` — workspace pins `"0.28"`
- No `tokio-tungstenite`, `redb`, or `rodio` yet — those arrive with WebSocket connectivity in later plans
- `golem-core` path dependency confirms Plan 02 output is reachable

## Source Files

```
apps/bardo-terminal/
├── Cargo.toml
└── src/
    ├── main.rs          — tokio main, terminal init, panic hook, run loop, clean shutdown
    ├── app.rs           — App struct, run() method, render loop, event pump
    ├── screen.rs        — Screen trait, ScreenId enum (all 29 as stubs), ScreenRegistry
    ├── state.rs         — AppState, AppAction enum, MockVitality, ConnectionStatus
    ├── layout.rs        — LayoutBreakpoint, responsive layout helpers
    ├── palette.rs       — ROSEDUST color palette constants
    └── screens/
        ├── mod.rs
        └── home.rs      — HomeScreen (placeholder: ASCII box + connection status + tick counter)
```

## Implementation Details

---

### Unit 1: Terminal Init & Render Loop

**Files:** `src/main.rs`, `src/app.rs`

#### Quick Reference

**Crossterm raw mode setup/teardown pattern:**

```rust
// Setup (main.rs before run loop)
crossterm::terminal::enable_raw_mode()?;
let mut stdout = std::io::stdout();
crossterm::execute!(stdout,
    crossterm::terminal::EnterAlternateScreen,
    crossterm::event::EnableMouseCapture,
)?;
let backend = ratatui::backend::CrosstermBackend::new(stdout);
let mut terminal = ratatui::Terminal::new(backend)?;

// Teardown (main.rs after run loop, or in cleanup function)
crossterm::terminal::disable_raw_mode()?;
crossterm::execute!(
    terminal.backend_mut(),
    crossterm::terminal::LeaveAlternateScreen,
    crossterm::event::DisableMouseCapture,
)?;
terminal.show_cursor()?;
```

**Panic hook to restore terminal on crash:**

Install this before anything else in `main()`. If the process panics mid-render, the terminal must be restored or the user's shell is left in raw mode with no cursor.

```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic_info| {
    // Best-effort teardown — ignore errors, we're already panicking
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
    );
    original_hook(panic_info);
}));
```

**60fps tick — 16.67ms deadline, frame skipping on overrun:**

```rust
const TARGET_FPS: u64 = 60;
const FRAME_DURATION: std::time::Duration =
    std::time::Duration::from_micros(1_000_000 / TARGET_FPS); // 16,666µs

// In run loop:
let frame_start = std::time::Instant::now();

// ... poll events, tick state, render ...

let elapsed = frame_start.elapsed();
if elapsed < FRAME_DURATION {
    std::thread::sleep(FRAME_DURATION - elapsed);
}
// If elapsed > FRAME_DURATION, we overran. Skip sleep, go straight to next frame.
// Frame skip is implicit — no explicit tracking needed at this scaffold stage.
```

**Clean shutdown sequence:**

`should_quit` is a bool on `App`. When `q` is pressed, `handle_key` sets `should_quit = true`. The run loop exits, `main.rs` calls the teardown sequence above, flushes any remaining events, and returns.

```rust
// In app.rs run():
pub fn run(
    &mut self,
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> color_eyre::Result<()> {
    let mut last_frame = std::time::Instant::now();

    loop {
        let frame_start = std::time::Instant::now();
        let dt = frame_start.duration_since(last_frame).as_secs_f64();

        // 1. Poll input (non-blocking, up to remaining frame budget)
        let timeout = FRAME_DURATION.saturating_sub(last_frame.elapsed());
        if crossterm::event::poll(timeout)? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    if let Some(action) = self.handle_key(key) {
                        self.apply_action(action);
                    }
                }
                crossterm::event::Event::Resize(w, h) => {
                    self.state.layout = LayoutBreakpoint::from_cols(w);
                    let _ = h; // height unused at scaffold stage
                }
                _ => {}
            }
        }

        // 2. Tick state
        self.state.tick_count = self.state.tick_count.wrapping_add(1);

        // 3. Render
        terminal.draw(|frame| self.render(frame))?;

        last_frame = frame_start;

        if self.should_quit {
            break;
        }

        // 4. Sleep remainder
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
    }
    Ok(())
}
```

**`main.rs` structure:**

```rust
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Install panic hook before touching terminal
    install_panic_hook();

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Build app
    let mut app = App::new();

    // Run
    let result = app.run(&mut terminal);

    // Always teardown, even on error
    teardown_terminal(terminal)?;

    result
}
```

---

### Unit 2: Screen System

**Files:** `src/screen.rs`, `src/screens/mod.rs`, `src/screens/home.rs`

#### Quick Reference

**Full Screen trait definition:**

```rust
use ratatui::{Frame, layout::Rect, widgets::*};
use crossterm::event::KeyEvent;
use crate::state::{AppState, AppAction};

pub trait Screen: Send + Sync {
    fn id(&self) -> ScreenId;
    fn title(&self) -> &str;
    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState);
    fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction>;
    fn on_focus(&mut self) {}   // called when screen becomes active
    fn on_blur(&mut self) {}    // called when navigating away
}
```

**ScreenRegistry:**

```rust
use std::collections::HashMap;

pub struct ScreenRegistry {
    screens: HashMap<ScreenId, Box<dyn Screen>>,
}

impl ScreenRegistry {
    pub fn new() -> Self {
        Self { screens: HashMap::new() }
    }

    pub fn register(&mut self, screen: Box<dyn Screen>) {
        self.screens.insert(screen.id(), screen);
    }

    pub fn get(&self, id: &ScreenId) -> Option<&dyn Screen> {
        self.screens.get(id).map(|s| s.as_ref())
    }

    pub fn get_mut(&mut self, id: &ScreenId) -> Option<&mut dyn Screen> {
        self.screens.get_mut(id).map(|s| s.as_mut())
    }
}
```

**Tab/screen switching:**

`App.active_screen: ScreenId` holds the current screen. `AppAction::NextScreen` / `AppAction::PrevScreen` cycle through `ScreenId::all()`. On switch: call `on_blur()` on old screen, update `active_screen`, call `on_focus()` on new screen.

**ScreenId enum — all 29 screens as stubs, Home implemented:**

Map the 29 screens from `prd2/18-interfaces/screens/00-screen-catalog.md` exactly:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenId {
    // HEARTH window (4 tabs)
    HearthOverview,
    HearthSignals,
    HearthOperations,
    HearthStatus,
    // MIND window (7 tabs)
    MindPipeline,
    MindGrimoire,
    MindPlaybook,
    MindDreams,
    MindInference,
    MindChainIntelligence,
    MindTechnicalAnalysis,
    // SOMA window (5 tabs)
    SomaPortfolio,
    SomaTrades,
    SomaCustody,
    SomaBudget,
    SomaSanctum,
    // WORLD window (5 tabs)
    WorldSolaris,
    WorldClade,
    WorldLethe,
    WorldBloodstains,
    WorldBazaar,
    // FATE window (4 tabs)
    FateMortality,
    FateLineage,
    FateAchievements,
    FateGraveyard,
    // COMMAND window (4 tabs)
    CommandSteer,
    CommandConfig,
    CommandEffects,
    CommandHermes,
}

impl ScreenId {
    /// Ordered list for tab cycling. Matches window order from spec.
    pub fn all() -> &'static [ScreenId] {
        &[
            ScreenId::HearthOverview,
            ScreenId::HearthSignals,
            ScreenId::HearthOperations,
            ScreenId::HearthStatus,
            ScreenId::MindPipeline,
            ScreenId::MindGrimoire,
            ScreenId::MindPlaybook,
            ScreenId::MindDreams,
            ScreenId::MindInference,
            ScreenId::MindChainIntelligence,
            ScreenId::MindTechnicalAnalysis,
            ScreenId::SomaPortfolio,
            ScreenId::SomaTrades,
            ScreenId::SomaCustody,
            ScreenId::SomaBudget,
            ScreenId::SomaSanctum,
            ScreenId::WorldSolaris,
            ScreenId::WorldClade,
            ScreenId::WorldLethe,
            ScreenId::WorldBloodstains,
            ScreenId::WorldBazaar,
            ScreenId::FateMortality,
            ScreenId::FateLineage,
            ScreenId::FateAchievements,
            ScreenId::FateGraveyard,
            ScreenId::CommandSteer,
            ScreenId::CommandConfig,
            ScreenId::CommandEffects,
            ScreenId::CommandHermes,
        ]
    }
}
```

Only `HearthOverview` (mapped to `HomeScreen`) is implemented in this plan. All others get a `StubScreen` that renders their name centered on screen.

---

### Unit 3: Color Palette & Design Tokens

**File:** `src/palette.rs`

#### Quick Reference

All colors extracted verbatim from `prd2/18-interfaces/rendering/00-design-system.md` section 2.

**Base and void:**

```rust
use ratatui::style::Color;

// Base and void
pub const BG_VOID:       Color = Color::Rgb(6,   6,   8);    // #060608 — deepest bg, violet-black
pub const BG_RAISED:     Color = Color::Rgb(12,  10,  14);   // #0C0A0E — panels, containers
pub const BG_MID:        Color = Color::Rgb(8,   8,   16);   // #080810 — headers, status bars
pub const BG_WARM:       Color = Color::Rgb(10,  8,   8);    // #0A0808 — conservation/terminal state
pub const BORDER:        Color = Color::Rgb(24,  20,  32);   // #181420 — panel borders
pub const BORDER_ACTIVE: Color = Color::Rgb(170, 112, 136);  // #AA7088 — active panel (rose)
pub const BORDER_DREAM:  Color = Color::Rgb(88,  88,  120);  // #585878 — dream state (indigo)
```

**Rose spectrum (primary color family):**

```rust
pub const ROSE:       Color = Color::Rgb(170, 112, 136); // #AA7088 — primary text, active data
pub const ROSE_BRIGHT:Color = Color::Rgb(204, 144, 168); // #CC90A8 — alerts, danger, T2 glow
pub const ROSE_DIM:   Color = Color::Rgb(122, 80,  96);  // #7A5060 — secondary labels
pub const ROSE_DEEP:  Color = Color::Rgb(58,  32,  48);  // #3A2030 — ghost text, noise floor
pub const ROSE_EMBER: Color = Color::Rgb(72,  40,  56);  // #482838 — phosphor residue
```

**Bone (the one important number per screen):**

```rust
pub const BONE:     Color = Color::Rgb(200, 184, 144); // #C8B890 — THE most important element
pub const BONE_DIM: Color = Color::Rgb(138, 122, 90);  // #8A7A5A — secondary emphasis in bone context
```

**Text hierarchy:**

```rust
pub const TEXT_PRIMARY: Color = Color::Rgb(152, 128, 144); // #988090 — standard readable text
pub const TEXT_DIM:     Color = Color::Rgb(88,  72,  88);  // #584858 — secondary text, labels
pub const TEXT_GHOST:   Color = Color::Rgb(48,  40,  48);  // #302830 — barely visible
pub const TEXT_PHANTOM: Color = Color::Rgb(32,  24,  32);  // #201820 — subliminal, display artifacts
```

**Semantic colors:**

```rust
pub const DREAM:      Color = Color::Rgb(88,  88,  120); // #585878 — dream state, altered consciousness
pub const DREAM_DIM:  Color = Color::Rgb(56,  56,  88);  // #383858 — dimmed dream
pub const DREAM_DEEP: Color = Color::Rgb(40,  40,  72);  // #282848 — deepest dream bg noise
pub const WARNING:    Color = Color::Rgb(170, 136, 85);  // #AA8855 — amber, mortality warnings
pub const SUCCESS:    Color = Color::Rgb(112, 136, 122); // #70887A — muted sage, healthy/nominal
// DANGER alias:
pub const DANGER:     Color = ROSE_BRIGHT;               // same as rose_bright per spec
```

**CRT materiality:**

```rust
pub const SCANLINE_DARK: Color = Color::Rgb(5,   5,   7);   // #050507 — darkened scanline rows
pub const PHOSPHOR_RES:  Color = Color::Rgb(26,  16,  24);  // #1A1018 — phosphor ghost
pub const NOISE_WARM:    Color = Color::Rgb(42,  24,  32);  // #2A1820 — degraded state noise
pub const NOISE_COOL:    Color = Color::Rgb(32,  24,  40);  // #201828 — dream state noise
```

**Text styles (ratatui Modifier shortcuts):**

```rust
use ratatui::style::Modifier;

pub const STYLE_BOLD:   Modifier = Modifier::BOLD;
pub const STYLE_DIM:    Modifier = Modifier::DIM;
pub const STYLE_ITALIC: Modifier = Modifier::ITALIC;
```

**Box drawing characters used in TUI:**

```rust
// Per spec: "Panel borders are sharp, single-character-width lines using box-drawing characters"
pub const BOX_TOP_LEFT:     char = '┌';
pub const BOX_TOP_RIGHT:    char = '┐';
pub const BOX_BOTTOM_LEFT:  char = '└';
pub const BOX_BOTTOM_RIGHT: char = '┘';
pub const BOX_HORIZONTAL:   char = '─';
pub const BOX_VERTICAL:     char = '│';
pub const BOX_T_DOWN:        char = '┬';
pub const BOX_T_UP:          char = '┴';
pub const BOX_T_RIGHT:       char = '├';
pub const BOX_T_LEFT:        char = '┤';
pub const BOX_CROSS:         char = '┼';
// Active frame bracket (used in tab bar per spec)
pub const FRAME_OPEN:        char = '⌈';
pub const FRAME_CLOSE:       char = '⌋';
// Block fill characters for gauges
pub const BLOCK_FULL:        char = '█';
pub const BLOCK_DARK:        char = '▓';
pub const BLOCK_MED:         char = '▒';
pub const BLOCK_LIGHT:       char = '░';
```

---

### Unit 4: Responsive Layout Engine

**File:** `src/layout.rs`

#### Quick Reference

**LayoutBreakpoint enum with column thresholds (from task specification):**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutBreakpoint {
    /// < 80 columns: single panel, minimal chrome, sprite sidebar suppressed
    Compact,
    /// 80-119 columns: two-panel layout, 6-col mini sprite sidebar
    Standard,
    /// 120-179 columns: three-panel layout, 10-col sprite sidebar
    Wide,
    /// 180+ columns: four-panel layout with sidebar, full 14-col sprite sidebar
    Ultra,
}

impl LayoutBreakpoint {
    pub fn from_cols(cols: u16) -> Self {
        match cols {
            0..=79   => LayoutBreakpoint::Compact,
            80..=119 => LayoutBreakpoint::Standard,
            120..=179 => LayoutBreakpoint::Wide,
            _         => LayoutBreakpoint::Ultra,
        }
    }

    /// Width of the sprite sidebar in columns for this breakpoint.
    /// 0 = sidebar suppressed (Compact).
    pub fn sprite_sidebar_cols(&self) -> u16 {
        match self {
            LayoutBreakpoint::Compact  => 0,
            LayoutBreakpoint::Standard => 6,
            LayoutBreakpoint::Wide     => 10,
            LayoutBreakpoint::Ultra    => 14,
        }
    }

    /// Number of content panels to show.
    pub fn panel_count(&self) -> u8 {
        match self {
            LayoutBreakpoint::Compact  => 1,
            LayoutBreakpoint::Standard => 2,
            LayoutBreakpoint::Wide     => 3,
            LayoutBreakpoint::Ultra    => 4,
        }
    }
}
```

**Layout computation from terminal size:**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Compute the primary content area, excluding sidebar and chrome rows.
/// Returns (sidebar_area, content_area).
/// On Compact, sidebar_area is zero-width and content_area is full width.
pub fn compute_layout(frame_size: Rect, bp: LayoutBreakpoint) -> (Rect, Rect) {
    let sidebar_cols = bp.sprite_sidebar_cols();
    // Reserve 1 row top (tab bar) + 1 row bottom (status bar)
    let chrome_rows = 2u16;
    let inner = Rect {
        x: 0,
        y: 1, // below tab bar
        width: frame_size.width,
        height: frame_size.height.saturating_sub(chrome_rows),
    };

    if sidebar_cols == 0 {
        return (Rect::default(), inner);
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(sidebar_cols),
            Constraint::Min(0),
        ])
        .split(inner);

    (chunks[0], chunks[1])
}
```

**Panel allocation per breakpoint:**

At scaffold stage, content area is not further subdivided — `HomeScreen` takes the full content rect. Later plans (05+) add panel splitting within the content area.

---

### Unit 5: Home Screen Placeholder

**Files:** `src/screens/mod.rs`, `src/screens/home.rs`

#### Quick Reference

**HomeScreen struct:**

```rust
use crate::screen::{Screen, ScreenId};
use crate::state::{AppAction, AppState};
use ratatui::{Frame, layout::Rect};
use crossterm::event::{KeyCode, KeyEvent};

pub struct HomeScreen {
    focused: bool,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self { focused: false }
    }
}
```

**Render: creature placeholder, vitality gauge, connection status, tick counter:**

```rust
impl Screen for HomeScreen {
    fn id(&self) -> ScreenId { ScreenId::HearthOverview }
    fn title(&self) -> &str { "HEARTH" }

    fn render(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        use ratatui::{
            style::{Color, Style, Modifier},
            widgets::{Block, Borders, Paragraph, Gauge},
            text::{Line, Span},
            layout::{Layout, Direction, Constraint},
        };
        use crate::palette::*;

        // Split content into: creature panel (left ~30%) + data panel (right ~70%)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // --- Creature placeholder (left panel) ---
        let creature_art = vec![
            Line::from(Span::styled("  ◉ ◉  ", Style::default().fg(ROSE))),
            Line::from(Span::styled(" ░░░░░ ", Style::default().fg(ROSE_DIM))),
            Line::from(Span::styled(" ░   ░ ", Style::default().fg(ROSE_DIM))),
            Line::from(Span::styled("  ─ ─  ", Style::default().fg(ROSE_DIM))),
        ];
        let creature_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_ACTIVE))
            .title(Span::styled(" SPECTRE ", Style::default().fg(ROSE)));
        let creature_inner = creature_block.inner(chunks[0]);
        frame.render_widget(creature_block, chunks[0]);
        frame.render_widget(
            Paragraph::new(creature_art).alignment(ratatui::layout::Alignment::Center),
            creature_inner,
        );

        // --- Data panel (right) ---
        let data_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // vitality gauge
                Constraint::Length(3),  // connection status
                Constraint::Min(0),     // tick counter + help
            ])
            .split(chunks[1]);

        // Vitality gauge (placeholder — value from MockVitality)
        let vitality_pct = (state.vitality.value * 100.0) as u16;
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(" VITALITY ", Style::default().fg(BONE)))
                    .border_style(Style::default().fg(BORDER))
            )
            .gauge_style(Style::default().fg(SUCCESS).bg(BG_RAISED))
            .percent(vitality_pct.min(100));
        frame.render_widget(gauge, data_chunks[0]);

        // Connection status
        let (status_text, status_color) = match state.connection_status {
            crate::state::ConnectionStatus::Connected    => ("● CONNECTED",    SUCCESS),
            crate::state::ConnectionStatus::Connecting   => ("◌ CONNECTING…",  WARNING),
            crate::state::ConnectionStatus::Disconnected => ("○ DISCONNECTED", DANGER),
        };
        let status_para = Paragraph::new(status_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(BORDER))
            )
            .style(Style::default().fg(status_color));
        frame.render_widget(status_para, data_chunks[1]);

        // Tick counter + help text
        let tick_line = Line::from(vec![
            Span::styled("tick: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                format!("{}", state.tick_count),
                Style::default().fg(ROSE).add_modifier(Modifier::BOLD),
            ),
        ]);
        let help_line = Line::from(Span::styled(
            "  q=quit  Tab=next screen",
            Style::default().fg(TEXT_GHOST),
        ));
        let info_para = Paragraph::new(vec![tick_line, Line::from(""), help_line])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(BORDER))
            );
        frame.render_widget(info_para, data_chunks[2]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => Some(AppAction::Quit),
            KeyCode::Tab => Some(AppAction::NextScreen),
            KeyCode::BackTab => Some(AppAction::PrevScreen),
            _ => None,
        }
    }

    fn on_focus(&mut self)  { self.focused = true; }
    fn on_blur(&mut self)   { self.focused = false; }
}
```

**AppState and placeholder types (src/state.rs):**

```rust
use crate::layout::LayoutBreakpoint;

pub struct AppState {
    pub tick_count: u64,
    pub connection_status: ConnectionStatus,
    pub vitality: MockVitality,      // TODO: connect to golem-mortality in Plan 70a
    pub layout: LayoutBreakpoint,
}

/// Placeholder vitality. Real type comes from golem-mortality (Plans 13a, 13b).
pub struct MockVitality {
    pub value: f64,  // 0.0 = dead, 1.0 = full health
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
}

#[derive(Debug, Clone)]
pub enum AppAction {
    Quit,
    NextScreen,
    PrevScreen,
    Resize(u16, u16),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tick_count: 0,
            connection_status: ConnectionStatus::Disconnected,
            vitality: MockVitality { value: 0.75 },
            layout: LayoutBreakpoint::Standard,
        }
    }
}
```

**StubScreen for unimplemented screens (src/screen.rs):**

Register one `StubScreen` per unimplemented `ScreenId` so that `Tab` cycling works across all 29 screens without panicking.

```rust
/// Generic stub for screens not yet implemented.
pub struct StubScreen {
    id: ScreenId,
    title: String,
}

impl StubScreen {
    pub fn new(id: ScreenId, title: &str) -> Self {
        Self { id, title: title.to_string() }
    }
}

impl Screen for StubScreen {
    fn id(&self) -> ScreenId { self.id }
    fn title(&self) -> &str { &self.title }

    fn render(&self, frame: &mut Frame, area: Rect, _state: &AppState) {
        use ratatui::{style::Style, widgets::{Block, Borders, Paragraph}};
        use crate::palette::{BORDER, ROSE_DIM};
        let msg = format!("[ {} — not yet implemented ]", self.title);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(ROSE_DIM))
                .alignment(ratatui::layout::Alignment::Center),
            inner,
        );
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Char('q') => Some(AppAction::Quit),
            KeyCode::Tab => Some(AppAction::NextScreen),
            KeyCode::BackTab => Some(AppAction::PrevScreen),
            _ => None,
        }
    }
}
```

---

## Failure Recovery

**`cargo check` fails with unresolved `golem-core` import:**
- Verify `crates/golem-core` compiled in Plan 02
- Check `Cargo.toml` path: `golem-core = { path = "../../crates/golem-core" }`
- Run `cargo check -p golem-core` first to isolate the failure

**`ratatui` version:**
- Use `ratatui = { workspace = true }`. Workspace pins `"0.30"`. Do not add a version override — the workspace is the single source of truth.

**Terminal not restoring on crash during development:**
- Confirm the panic hook is installed before any `crossterm` calls
- If shell is stuck in raw mode: `reset` or `stty sane` in the terminal
- If cursor is invisible: `tput cnorm`

**`crossterm::event::poll` returns `Err` on some terminals:**
- Wrap in `color_eyre::Result` and propagate through `?`. The teardown path in `main.rs` will restore the terminal even on propagated errors.

**Compilation error: `Screen` not object-safe:**
- `fn on_focus(&mut self) {}` and `fn on_blur(&mut self) {}` have default implementations — this is fine for object safety as long as they take `&mut self` (not `Self`). If the compiler complains, confirm no method returns `Self` or uses `Self` as a parameter.

**Render loop blocks (poll timeout too long):**
- Confirm `event::poll(timeout)` uses `FRAME_DURATION.saturating_sub(last_frame.elapsed())` as the timeout, not a fixed large value. On the first iteration, `last_frame.elapsed()` is near zero, so timeout is the full frame budget.

---

## Testing Checkpoint

```bash
# Must pass before marking this plan complete
cargo check -p bardo-terminal

# Unit tests
cargo test -p bardo-terminal -- --nocapture

# Manual smoke test: open TUI, verify home screen renders, q quits cleanly
cargo run -p bardo-terminal
```

**What to verify in the manual test:**
1. Terminal enters full-screen mode (alternate screen)
2. Home screen renders: left creature placeholder, right vitality gauge + connection status + tick counter
3. Tick counter increments visibly (~60 ticks/second — may appear as fast flicker; that's correct)
4. `Tab` cycles to the next screen (StubScreen showing screen name)
5. `q` exits and restores the terminal (cursor visible, shell prompt returns, no stray raw mode)
6. Resize the terminal window while running — layout should adapt (breakpoint may change)
7. Crash test: `kill -SIGSEGV <pid>` in another terminal — shell should still be usable after (panic hook fired)

**Expected output of `cargo test`:**

At scaffold stage, tests are minimal — palette constant sanity checks and LayoutBreakpoint boundary tests:

```rust
#[test]
fn test_layout_breakpoints() {
    assert_eq!(LayoutBreakpoint::from_cols(0),   LayoutBreakpoint::Compact);
    assert_eq!(LayoutBreakpoint::from_cols(79),  LayoutBreakpoint::Compact);
    assert_eq!(LayoutBreakpoint::from_cols(80),  LayoutBreakpoint::Standard);
    assert_eq!(LayoutBreakpoint::from_cols(119), LayoutBreakpoint::Standard);
    assert_eq!(LayoutBreakpoint::from_cols(120), LayoutBreakpoint::Wide);
    assert_eq!(LayoutBreakpoint::from_cols(179), LayoutBreakpoint::Wide);
    assert_eq!(LayoutBreakpoint::from_cols(180), LayoutBreakpoint::Ultra);
    assert_eq!(LayoutBreakpoint::from_cols(400), LayoutBreakpoint::Ultra);
}

#[test]
fn test_screen_id_count() {
    assert_eq!(ScreenId::all().len(), 29);
}

#[test]
fn test_palette_void_not_pure_black() {
    // Per spec: "The background is never pure black (#000000)"
    if let ratatui::style::Color::Rgb(r, g, b) = palette::BG_VOID {
        assert!(r > 0 || g > 0 || b > 0, "BG_VOID must not be pure black");
    }
}
```

---

## Completion Report

*(Codex fills this in after implementation. Include: actual ratatui/crossterm versions used, any deviations from plan, compilation errors encountered and resolved, manual test results.)*

## Verification

### Invariants

<!-- INV-001: Frame duration constant -->
- **type**: numeric_range
- **module**: bardo_terminal::app
- **property**: Target frame duration matches 60 FPS deadline
- **formula**: FRAME_DURATION = Duration::from_micros(1_000_000 / 60)
- **constraint**: FRAME_DURATION == 16_666µs (±1µs for floating-point tolerance)
- **test_fn**: `test_frame_duration_60fps`
- **strategy**: unit
- **inputs**: `{ TARGET_FPS: 60 }`
- **oracle**: 1_000_000 / 60 = 16_666.666... → 16_666µs
- **severity**: spec
- **source**: plan Quick Reference, Unit 1

<!-- INV-002: Layout breakpoint from_cols boundary 0-79 -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Column range 0-79 maps to Compact layout
- **formula**: from_cols(cols) → LayoutBreakpoint::Compact iff 0 ≤ cols ≤ 79
- **constraint**: ∀ cols ∈ [0, 79]: from_cols(cols) == Compact
- **test_fn**: `test_layout_breakpoint_compact`
- **strategy**: proptest
- **inputs**: `{ cols: 0u16..=79u16 }`
- **oracle**: "Compact"
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-003: Layout breakpoint from_cols boundary 80-119 -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Column range 80-119 maps to Standard layout
- **formula**: from_cols(cols) → LayoutBreakpoint::Standard iff 80 ≤ cols ≤ 119
- **constraint**: ∀ cols ∈ [80, 119]: from_cols(cols) == Standard
- **test_fn**: `test_layout_breakpoint_standard`
- **strategy**: proptest
- **inputs**: `{ cols: 80u16..=119u16 }`
- **oracle**: "Standard"
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-004: Layout breakpoint from_cols boundary 120-179 -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Column range 120-179 maps to Wide layout
- **formula**: from_cols(cols) → LayoutBreakpoint::Wide iff 120 ≤ cols ≤ 179
- **constraint**: ∀ cols ∈ [120, 179]: from_cols(cols) == Wide
- **test_fn**: `test_layout_breakpoint_wide`
- **strategy**: proptest
- **inputs**: `{ cols: 120u16..=179u16 }`
- **oracle**: "Wide"
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-005: Layout breakpoint from_cols boundary 180+ -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Column range 180+ maps to Ultra layout
- **formula**: from_cols(cols) → LayoutBreakpoint::Ultra iff cols ≥ 180
- **constraint**: ∀ cols ≥ 180: from_cols(cols) == Ultra
- **test_fn**: `test_layout_breakpoint_ultra`
- **strategy**: proptest
- **inputs**: `{ cols: 180u16..65535u16 }`
- **oracle**: "Ultra"
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-006: Sprite sidebar width for Compact -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Compact layout has zero-width sprite sidebar
- **formula**: sprite_sidebar_cols(Compact) = 0
- **constraint**: Compact.sprite_sidebar_cols() == 0
- **test_fn**: `test_sprite_sidebar_compact_zero`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 0
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-007: Sprite sidebar width for Standard -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Standard layout has 6-column sprite sidebar
- **formula**: sprite_sidebar_cols(Standard) = 6
- **constraint**: Standard.sprite_sidebar_cols() == 6
- **test_fn**: `test_sprite_sidebar_standard_6col`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 6
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-008: Sprite sidebar width for Wide -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Wide layout has 10-column sprite sidebar
- **formula**: sprite_sidebar_cols(Wide) = 10
- **constraint**: Wide.sprite_sidebar_cols() == 10
- **test_fn**: `test_sprite_sidebar_wide_10col`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 10
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-009: Sprite sidebar width for Ultra -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Ultra layout has 14-column sprite sidebar
- **formula**: sprite_sidebar_cols(Ultra) = 14
- **constraint**: Ultra.sprite_sidebar_cols() == 14
- **test_fn**: `test_sprite_sidebar_ultra_14col`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 14
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-010: Panel count for Compact -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Compact layout shows 1 content panel
- **formula**: panel_count(Compact) = 1
- **constraint**: Compact.panel_count() == 1
- **test_fn**: `test_panel_count_compact`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 1
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-011: Panel count for Standard -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Standard layout shows 2 content panels
- **formula**: panel_count(Standard) = 2
- **constraint**: Standard.panel_count() == 2
- **test_fn**: `test_panel_count_standard`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 2
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-012: Panel count for Wide -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Wide layout shows 3 content panels
- **formula**: panel_count(Wide) = 3
- **constraint**: Wide.panel_count() == 3
- **test_fn**: `test_panel_count_wide`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 3
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-013: Panel count for Ultra -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Ultra layout shows 4 content panels
- **formula**: panel_count(Ultra) = 4
- **constraint**: Ultra.panel_count() == 4
- **test_fn**: `test_panel_count_ultra`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 4
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-014: Screen count -->
- **type**: numeric_range
- **module**: bardo_terminal::screen
- **property**: ScreenId catalog contains exactly 29 screens
- **formula**: ScreenId::all().len() = 29
- **constraint**: all().len() == 29
- **test_fn**: `test_screen_id_count_29`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: 29
- **severity**: spec
- **source**: plan Unit 2, Quick Reference

<!-- INV-015: Screen cycling wraps -->
- **type**: state_machine
- **module**: bardo_terminal::app
- **property**: Tab navigation cycles through all 29 screens in order; after last screen, returns to first
- **formula**: NextScreen at screen[28] → screen[0]; NextScreen at screen[n] → screen[n+1] for n < 28
- **constraint**: ∀n ∈ [0, 28]: next_screen(screen[n]) == screen[(n+1) % 29]
- **test_fn**: `test_screen_cycling_wraps_at_end`
- **strategy**: integration
- **inputs**: `{ current_screen: all screens in order }`
- **oracle**: see formula
- **severity**: spec
- **source**: plan Unit 2

<!-- INV-016: Screen order matches ScreenId::all -->
- **type**: capacity
- **module**: bardo_terminal::screen
- **property**: ScreenId::all() returns screens in canonical order: HEARTH (4) → MIND (7) → SOMA (5) → WORLD (5) → FATE (4) → COMMAND (4)
- **formula**: all() = [HearthOverview, HearthSignals, HearthOperations, HearthStatus, MindPipeline, ..., CommandHermes]
- **constraint**: First 4 elements are HEARTH window screens; next 7 are MIND; etc.
- **test_fn**: `test_screen_order_matches_windows`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: HEARTH ⇒ MIND ⇒ SOMA ⇒ WORLD ⇒ FATE ⇒ COMMAND
- **severity**: spec
- **source**: plan Unit 2, Quick Reference

<!-- INV-017: BG_VOID not pure black -->
- **type**: numeric_range
- **module**: bardo_terminal::palette
- **property**: BG_VOID background is never pure black per design system spec
- **formula**: BG_VOID = Color::Rgb(6, 6, 8) = #060608
- **constraint**: ∀(r, g, b) in BG_VOID: (r > 0 ∨ g > 0 ∨ b > 0)
- **test_fn**: `test_palette_void_not_pure_black`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: (6 > 0) ∨ (6 > 0) ∨ (8 > 0) = true
- **severity**: spec
- **source**: plan Unit 3, Quick Reference

<!-- INV-018: ROSE color constant -->
- **type**: numeric_range
- **module**: bardo_terminal::palette
- **property**: Primary rose color is #AA7088 (RGB 170, 112, 136)
- **formula**: ROSE = Color::Rgb(170, 112, 136)
- **constraint**: r == 170 ∧ g == 112 ∧ b == 136
- **test_fn**: `test_palette_rose_value`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: #AA7088
- **severity**: spec
- **source**: plan Unit 3, Quick Reference, prd2/18-interfaces/03-tui.md

<!-- INV-019: BONE color constant -->
- **type**: numeric_range
- **module**: bardo_terminal::palette
- **property**: BONE color (most important element) is #C8B890 (RGB 200, 184, 144)
- **formula**: BONE = Color::Rgb(200, 184, 144)
- **constraint**: r == 200 ∧ g == 184 ∧ b == 144
- **test_fn**: `test_palette_bone_value`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: #C8B890
- **severity**: spec
- **source**: plan Unit 3, Quick Reference

<!-- INV-020: BORDER_ACTIVE equals ROSE -->
- **type**: numeric_range
- **module**: bardo_terminal::palette
- **property**: Active panel border color equals primary rose color
- **formula**: BORDER_ACTIVE == ROSE = #AA7088
- **constraint**: BORDER_ACTIVE.rgb() == ROSE.rgb()
- **test_fn**: `test_border_active_equals_rose`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: (170, 112, 136)
- **severity**: spec
- **source**: plan Unit 3, Quick Reference

<!-- INV-021: Text PRIMARY hierarchy -->
- **type**: numeric_range
- **module**: bardo_terminal::palette
- **property**: TEXT_PRIMARY is readable baseline text color #988090
- **formula**: TEXT_PRIMARY = Color::Rgb(152, 128, 144)
- **constraint**: r == 152 ∧ g == 128 ∧ b == 144
- **test_fn**: `test_palette_text_primary`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: #988090
- **severity**: spec
- **source**: plan Unit 3, Quick Reference

<!-- INV-022: Chrome rows overhead -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Layout computation reserves 2 rows for chrome (tab bar + status bar)
- **formula**: chrome_rows = 2
- **constraint**: inner.height = frame_size.height - 2
- **test_fn**: `test_chrome_rows_2`
- **strategy**: unit
- **inputs**: `{ frame_size: Rect { width: 80, height: 24 } }`
- **oracle**: inner.height == 22
- **severity**: spec
- **source**: plan Unit 4, Quick Reference

<!-- INV-023: Tick count wraps without panic -->
- **type**: monotonic
- **module**: bardo_terminal::state
- **property**: Tick count increments via wrapping_add and never overflows
- **formula**: tick_count(t+1) = tick_count(t).wrapping_add(1)
- **constraint**: ∀tick ∈ [0, u64::MAX]: tick.wrapping_add(1) is well-defined
- **test_fn**: `test_tick_count_wrapping`
- **strategy**: proptest
- **inputs**: `{ tick_count: 0u64..u64::MAX }`
- **oracle**: (u64::MAX).wrapping_add(1) == 0
- **severity**: code
- **source**: plan Unit 1, app.rs run loop (line 196)

<!-- INV-024: MockVitality range [0, 1] -->
- **type**: numeric_range
- **module**: bardo_terminal::state
- **property**: Vitality placeholder value is clamped to [0.0, 1.0]
- **formula**: MockVitality.value ∈ [0.0, 1.0]
- **constraint**: 0.0 ≤ value ≤ 1.0
- **test_fn**: `test_vitality_value_range`
- **strategy**: unit
- **inputs**: `{ value: 0.75 (default from AppState::default) }`
- **oracle**: 0.0 ≤ 0.75 ≤ 1.0
- **severity**: code
- **source**: plan Unit 5, state.rs (line 766)

<!-- INV-025: Vitality gauge percent clamping -->
- **type**: numeric_range
- **module**: bardo_terminal::screens::home
- **property**: Vitality gauge percentage clamped to [0, 100] before rendering
- **formula**: gauge_percent = (vitality.value * 100.0).as_u16().min(100)
- **constraint**: 0 ≤ gauge_percent ≤ 100
- **test_fn**: `test_vitality_gauge_clamped`
- **strategy**: proptest
- **inputs**: `{ vitality_value: 0.0f64..=1.0f64 }`
- **oracle**: (value * 100.0).min(100.0) ∈ [0.0, 100.0]
- **severity**: code
- **source**: plan Unit 5, home.rs (line 676)

<!-- INV-026: Terminal setup sequence -->
- **type**: event_sequence
- **module**: bardo_terminal::main
- **property**: Terminal enters raw mode, alternate screen, mouse capture in order
- **formula**: enable_raw_mode() → EnterAlternateScreen → EnableMouseCapture
- **constraint**: Raw mode must be enabled before alternate screen; both before mouse capture
- **test_fn**: `test_terminal_setup_sequence`
- **strategy**: integration
- **inputs**: `{}`
- **oracle**: all three succeed in order without error
- **severity**: spec
- **source**: plan Unit 1, Quick Reference (lines 105-110)

<!-- INV-027: Terminal teardown sequence -->
- **type**: event_sequence
- **module**: bardo_terminal::main
- **property**: Terminal disables raw mode, leaves alternate screen, disables mouse in order
- **formula**: disable_raw_mode() → LeaveAlternateScreen → DisableMouseCapture → show_cursor()
- **constraint**: Raw mode must be disabled first; cursor shown last
- **test_fn**: `test_terminal_teardown_sequence`
- **strategy**: integration
- **inputs**: `{}`
- **oracle**: all four succeed in order without error
- **severity**: spec
- **source**: plan Unit 1, Quick Reference (lines 115-121)

<!-- INV-028: Panic hook restores terminal -->
- **type**: event_sequence
- **module**: bardo_terminal::main
- **property**: Panic handler disables raw mode and leaves alternate screen even on panic
- **formula**: panic → [disable_raw_mode, LeaveAlternateScreen, DisableMouseCapture], then call original_hook
- **constraint**: Terminal restoration must not error even if panic is mid-render
- **test_fn**: `test_panic_hook_teardown`
- **strategy**: integration
- **inputs**: `{}`
- **oracle**: after simulated panic, terminal is usable (not in raw mode)
- **severity**: spec
- **source**: plan Unit 1, Quick Reference (lines 129-139)

<!-- INV-029: Frame sleep calculates correctly -->
- **type**: numeric_range
- **module**: bardo_terminal::app
- **property**: Sleep duration = max(0, FRAME_DURATION - elapsed)
- **formula**: sleep_duration = FRAME_DURATION.saturating_sub(elapsed)
- **constraint**: 0 ≤ sleep_duration ≤ FRAME_DURATION
- **test_fn**: `test_frame_sleep_duration`
- **strategy**: proptest
- **inputs**: `{ elapsed: 0us..=50000us }`
- **oracle**: FRAME_DURATION.saturating_sub(elapsed) is well-defined
- **severity**: code
- **source**: plan Unit 1, app.rs (lines 208-210)

<!-- INV-030: ConnectionStatus enum completeness -->
- **type**: state_machine
- **module**: bardo_terminal::state
- **property**: ConnectionStatus has exactly three variants: Connected, Disconnected, Connecting
- **formula**: ConnectionStatus ∈ {Connected, Disconnected, Connecting}
- **constraint**: No other variants exist
- **test_fn**: `test_connection_status_variants`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: exactly 3
- **severity**: code
- **source**: plan Unit 5, state.rs (lines 747-751)

<!-- INV-031: Default AppState initialization -->
- **type**: numeric_range
- **module**: bardo_terminal::state
- **property**: AppState::default initializes with tick=0, disconnected, vitality=0.75, layout=Standard
- **formula**: AppState::default() = { tick_count: 0, connection_status: Disconnected, vitality: 0.75, layout: Standard }
- **constraint**: All four fields match spec
- **test_fn**: `test_app_state_default`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: see formula
- **severity**: code
- **source**: plan Unit 5, state.rs (lines 761-770)

<!-- INV-032: HomeScreen key bindings -->
- **type**: state_machine
- **module**: bardo_terminal::screens::home
- **property**: HomeScreen handle_key returns: q/Q → Quit, Tab → NextScreen, Shift+Tab → PrevScreen, others → None
- **formula**: handle_key(KeyEvent) → Option<AppAction>
- **constraint**: Only three keys (q, Tab, Shift+Tab) produce actions
- **test_fn**: `test_home_screen_key_bindings`
- **strategy**: unit
- **inputs**: `{ key: KeyCode::Char('q'), KeyCode::Tab, KeyCode::BackTab, KeyCode::Other }`
- **oracle**: Some(Quit), Some(NextScreen), Some(PrevScreen), None
- **severity**: spec
- **source**: plan Unit 5, home.rs (lines 715-722)

<!-- INV-033: StubScreen key bindings -->
- **type**: state_machine
- **module**: bardo_terminal::screen
- **property**: StubScreen handle_key returns: q → Quit, Tab → NextScreen, Shift+Tab → PrevScreen, others → None
- **formula**: handle_key(KeyEvent) → Option<AppAction>
- **constraint**: Same three keys as HomeScreen
- **test_fn**: `test_stub_screen_key_bindings`
- **strategy**: unit
- **inputs**: `{ key: KeyCode::Char('q'), KeyCode::Tab, KeyCode::BackTab, KeyCode::Other }`
- **oracle**: Some(Quit), Some(NextScreen), Some(PrevScreen), None
- **severity**: spec
- **source**: plan Unit 5, screen.rs (lines 811-818)

<!-- INV-034: Layout inner rect excludes top row -->
- **type**: numeric_range
- **module**: bardo_terminal::layout
- **property**: Inner content rect starts at y=1 (below tab bar)
- **formula**: inner.y = 1
- **constraint**: y == 1
- **test_fn**: `test_layout_inner_y_offset`
- **strategy**: unit
- **inputs**: `{ frame_size: Rect { width: 80, height: 24, x: 0, y: 0 } }`
- **oracle**: inner.y == 1
- **severity**: code
- **source**: plan Unit 4, layout.rs (line 564)

<!-- INV-035: Creature panel left split percentage -->
- **type**: numeric_range
- **module**: bardo_terminal::screens::home
- **property**: HomeScreen splits creature left panel at 30% of content width
- **formula**: creature_constraint = Constraint::Percentage(30)
- **constraint**: First split chunk width ≈ 0.30 × area.width
- **test_fn**: `test_home_creature_panel_width`
- **strategy**: unit
- **inputs**: `{ area: Rect { width: 100, height: 20 } }`
- **oracle**: chunks[0].width ≈ 30
- **severity**: code
- **source**: plan Unit 5, home.rs (lines 633-635)

<!-- INV-036: Data panel right split percentage -->
- **type**: numeric_range
- **module**: bardo_terminal::screens::home
- **property**: HomeScreen splits data right panel at 70% of content width
- **formula**: data_constraint = Constraint::Percentage(70)
- **constraint**: Second split chunk width ≈ 0.70 × area.width
- **test_fn**: `test_home_data_panel_width`
- **strategy**: unit
- **inputs**: `{ area: Rect { width: 100, height: 20 } }`
- **oracle**: chunks[1].width ≈ 70
- **severity**: code
- **source**: plan Unit 5, home.rs (line 635)

<!-- INV-037: Data panel vertical splits -->
- **type**: numeric_range
- **module**: bardo_terminal::screens::home
- **property**: Data panel has 3 vertical regions: vitality (3 rows), connection (3 rows), info (remaining)
- **formula**: [Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)]
- **constraint**: data_chunks[0].height == 3, data_chunks[1].height == 3, data_chunks[2].height >= 0
- **test_fn**: `test_home_data_vertical_layout`
- **strategy**: unit
- **inputs**: `{ area: Rect { width: 70, height: 20 } }`
- **oracle**: chunks have heights 3, 3, remainder
- **severity**: code
- **source**: plan Unit 5, home.rs (lines 657-664)

### Regression Anchors

`test_frame_duration_60fps`
`test_layout_breakpoint_compact`
`test_layout_breakpoint_standard`
`test_layout_breakpoint_wide`
`test_layout_breakpoint_ultra`
`test_sprite_sidebar_compact_zero`
`test_sprite_sidebar_standard_6col`
`test_sprite_sidebar_wide_10col`
`test_sprite_sidebar_ultra_14col`
`test_panel_count_compact`
`test_panel_count_standard`
`test_panel_count_wide`
`test_panel_count_ultra`
`test_screen_id_count_29`
`test_screen_cycling_wraps_at_end`
`test_screen_order_matches_windows`
`test_palette_void_not_pure_black`
`test_palette_rose_value`
`test_palette_bone_value`
`test_border_active_equals_rose`
`test_palette_text_primary`
`test_chrome_rows_2`
`test_tick_count_wrapping`
`test_vitality_value_range`
`test_vitality_gauge_clamped`
`test_terminal_setup_sequence`
`test_terminal_teardown_sequence`
`test_panic_hook_teardown`
`test_frame_sleep_duration`
`test_connection_status_variants`
`test_app_state_default`
`test_home_screen_key_bindings`
`test_stub_screen_key_bindings`
`test_layout_inner_y_offset`
`test_home_creature_panel_width`
`test_home_data_panel_width`
`test_home_data_vertical_layout`

### Cross-Crate Contracts

| Upstream | Input Condition | Expected Behavior |
|----------|----------------|-------------------|
| `golem-core::event::EventFabric` | Import present in app.rs | Compiles without error (imported but unused until Plan 70a) |
| `ratatui` workspace | dependency specified as `{ workspace = true }` | Resolves to workspace pinned version (0.30) |
| `crossterm` workspace | dependency specified as `{ workspace = true }` | Resolves to workspace pinned version (0.28) |

### Event Sequence Assertions

**Terminal initialization sequence:**
1. Install panic hook
2. Enable raw mode
3. Enter alternate screen
4. Enable mouse capture
5. Create ratatui Terminal
6. Enter run loop

**Terminal shutdown sequence:**
1. Exit run loop (should_quit == true)
2. Disable raw mode
3. Leave alternate screen
4. Disable mouse capture
5. Show cursor
6. Return control to shell

**Run loop per frame:**
1. Record frame_start time
2. Poll crossterm events (with timeout = remaining budget)
3. If event: call handle_key → apply_action
4. Increment tick_count
5. Call render()
6. Calculate elapsed
7. Sleep remainder (saturating subtraction)
8. Check should_quit

**Screen navigation sequence:**
1. Poll Tab or Shift+Tab key
2. Call on_blur() on current screen
3. Update active_screen
4. Call on_focus() on new screen
5. Next render includes new screen

### Academic References Verified

| Reference | Formula/Constant | PRD2 Match | Notes |
|-----------|-----------------|------------|-------|
| 60 FPS target | FRAME_DURATION = 16.67ms | plan::Quick Reference | Standard modern monitor refresh rate; no academic citation needed |
| Layout breakpoints | 4 breakpoints at 0, 80, 120, 180 cols | plan Unit 4 | UI/UX responsive design best practice; thresholds chosen for readability |
| ROSEDUST palette | RGB(170, 112, 136) primary | plan Unit 3 | Referenced from prd2/18-interfaces/03-tui.md, no academic source needed (design system constant) |
| CRT materiality | Scanline/phosphor colors | plan Unit 3 | Design aesthetic choice, not scientifically grounded |

