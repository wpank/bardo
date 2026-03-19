# bardo-terminal

## What It Is

`bardo-terminal` is the workspace’s full-screen terminal UI scaffold for the Golem control surface. It boots a raw-mode alternate-screen ratatui loop at frame pace, registers the 29-screen catalog, and renders a placeholder HEARTH home screen while live data paths are wired in future plans.

The scaffold is aligned to the terminal spec goals:

- `bardo-terminal` is the main full-screen TUI application (`bardo-terminal` in spec) [`03-tui.md`](../../../prd2/18-interfaces/03-tui.md).
- Rendered surface follows the ROSEDUST palette and terminal aesthetic baseline [`00-design-system.md`](../../../prd2/18-interfaces/rendering/00-design-system.md).
- Screen catalog follows the 29-screen ordered model (`HEARTH`, `MIND`, `SOMA`, `WORLD`, `FATE`, `COMMAND`) [`00-screen-catalog.md`](../../../prd2/18-interfaces/screens/00-screen-catalog.md).

## Features

- Deterministic terminal lifecycle (setup → run → teardown) with panic-safe restoration.
- 60 fps event/render frame pacing and frame budget handling.
- Catalog-first screen system with 29 IDs and pluggable `Screen` implementations.
- Placeholder home screen showing:
  - animated creature panel
  - vitality gauge (placeholder value)
  - connection status label
  - tick counter and waveform
- Responsive frame layout with four breakpoints:
  - `Compact` (`< 80` cols): single panel
  - `Standard` (`80..=119`): two panels plus sprite sidebar
  - `Wide` (`120..=179`): three panels plus wider sidebar
  - `Ultra` (`>= 180`): four panels plus full sidebar width
- Screen navigation:
  - `q` quits
  - `Tab` moves forward
  - `Shift+Tab` moves backward

## Module Overview

### `app`

Owns the runtime loop and current navigation context:

- initializes all screens from `ScreenId::all()`
- pumps events (`crossterm::event::poll/read`)
- applies actions (`Quit`, `NextScreen`, `PrevScreen`, `Resize`)
- calls `render` each frame
- advances a tick counter for placeholder animation

### `screen`

Defines a pluggable screen architecture:

- `Screen` trait contract used by all screens
- `ScreenId` enum with all 29 IDs in catalog order
- `ScreenRegistry` for keyed screen storage and lookup
- `StubScreen` for unimplemented tabs to keep navigation safe during scaffold

### `state`

Shared app state and transitions:

- tick counter
- connection status
- placeholder vitality
- layout breakpoint
- app-level action enum

### `layout`

Responsive helpers:

- `LayoutBreakpoint::from_cols`
- `sprite_sidebar_cols`
- `panel_count`
- `compute_layout` for header/footer-safe content split

### `palette`

ROSEDUST token constants for colors and drawing characters, including the exact values from the design spec.

### `screens::home`

Home placeholder implementation of `Screen`, providing the initial user-facing content:

- creature silhouette
- vitality/probe panels
- status and help footer text

## Public API

`bardo-terminal` is a binary crate, so the types are crate-local (`pub(crate)`), but the scaffold API is documented here for implementers:

```rust
pub(crate) struct App {
    state: AppState,
    screens: ScreenRegistry,
    active_screen: ScreenId,
    should_quit: bool,
}

impl App {
    pub(crate) fn new() -> Self;
    pub(crate) fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> anyhow::Result<()>;
}
```

```rust
pub(crate) trait Screen: Send + Sync {
    fn id(&self) -> ScreenId;
    fn title(&self) -> &str;
    fn render(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, state: &AppState);
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<AppAction>;
    fn on_focus(&mut self) {}
    fn on_blur(&mut self) {}
}
```

```rust
pub(crate) enum ScreenId {
    HearthOverview,
    HearthSignals,
    HearthOperations,
    HearthStatus,
    MindPipeline,
    MindGrimoire,
    MindPlaybook,
    MindDreams,
    MindInference,
    MindChainIntelligence,
    MindTechnicalAnalysis,
    SomaPortfolio,
    SomaTrades,
    SomaCustody,
    SomaBudget,
    SomaSanctum,
    WorldSolaris,
    WorldClade,
    WorldLethe,
    WorldBloodstains,
    WorldBazaar,
    FateMortality,
    FateLineage,
    FateAchievements,
    FateGraveyard,
    CommandSteer,
    CommandConfig,
    CommandEffects,
    CommandHermes,
}

impl ScreenId {
    pub(crate) fn all() -> &'static [Self];
    pub(crate) const fn window_name(self) -> &'static str;
    pub(crate) const fn tab_name(self) -> &'static str;
}
```

```rust
pub(crate) struct ScreenRegistry {
    screens: std::collections::HashMap<ScreenId, Box<dyn Screen>>,
}

impl ScreenRegistry {
    pub(crate) fn new() -> Self;
    pub(crate) fn register(&mut self, screen: Box<dyn Screen>);
    pub(crate) fn get(&self, id: &ScreenId) -> Option<&dyn Screen>;
    pub(crate) fn get_mut(&mut self, id: &ScreenId) -> Option<&mut dyn Screen>;
}
```

```rust
pub(crate) enum AppAction {
    Quit,
    NextScreen,
    PrevScreen,
    Resize(u16, u16),
}

pub(crate) struct AppState {
    pub(crate) tick_count: u64,
    pub(crate) connection_status: ConnectionStatus,
    pub(crate) vitality: MockVitality,
    pub(crate) layout: LayoutBreakpoint,
}

pub(crate) enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
}

pub(crate) struct MockVitality { pub(crate) value: f64; }

pub(crate) enum LayoutBreakpoint {
    Compact,
    Standard,
    Wide,
    Ultra,
}
```

```rust
pub(crate) struct ColorPalette;

pub(crate) const BG_VOID: ratatui::style::Color = ratatui::style::Color::Rgb(6, 6, 8);
pub(crate) const BG_RAISED: ratatui::style::Color = ratatui::style::Color::Rgb(12, 10, 14);
pub(crate) const BG_MID: ratatui::style::Color = ratatui::style::Color::Rgb(8, 8, 16);
pub(crate) const BG_WARM: ratatui::style::Color = ratatui::style::Color::Rgb(10, 8, 8);
pub(crate) const BORDER: ratatui::style::Color = ratatui::style::Color::Rgb(24, 20, 32);
pub(crate) const BORDER_ACTIVE: ratatui::style::Color = ratatui::style::Color::Rgb(170, 112, 136);
pub(crate) const BORDER_DREAM: ratatui::style::Color = ratatui::style::Color::Rgb(88, 88, 120);
pub(crate) const ROSE: ratatui::style::Color = ratatui::style::Color::Rgb(170, 112, 136);
pub(crate) const ROSE_BRIGHT: ratatui::style::Color = ratatui::style::Color::Rgb(204, 144, 168);
pub(crate) const ROSE_DIM: ratatui::style::Color = ratatui::style::Color::Rgb(122, 80, 96);
pub(crate) const ROSE_DEEP: ratatui::style::Color = ratatui::style::Color::Rgb(58, 32, 48);
pub(crate) const ROSE_EMBER: ratatui::style::Color = ratatui::style::Color::Rgb(72, 40, 56);
pub(crate) const BONE: ratatui::style::Color = ratatui::style::Color::Rgb(200, 184, 144);
pub(crate) const BONE_DIM: ratatui::style::Color = ratatui::style::Color::Rgb(138, 122, 90);
pub(crate) const TEXT_PRIMARY: ratatui::style::Color = ratatui::style::Color::Rgb(152, 128, 144);
pub(crate) const TEXT_DIM: ratatui::style::Color = ratatui::style::Color::Rgb(88, 72, 88);
pub(crate) const TEXT_GHOST: ratatui::style::Color = ratatui::style::Color::Rgb(48, 40, 48);
pub(crate) const TEXT_PHANTOM: ratatui::style::Color = ratatui::style::Color::Rgb(32, 24, 32);
pub(crate) const DREAM: ratatui::style::Color = ratatui::style::Color::Rgb(88, 88, 120);
pub(crate) const DREAM_DIM: ratatui::style::Color = ratatui::style::Color::Rgb(56, 56, 88);
pub(crate) const DREAM_DEEP: ratatui::style::Color = ratatui::style::Color::Rgb(40, 40, 72);
pub(crate) const WARNING: ratatui::style::Color = ratatui::style::Color::Rgb(170, 136, 85);
pub(crate) const SUCCESS: ratatui::style::Color = ratatui::style::Color::Rgb(112, 136, 122);
pub(crate) const DANGER: ratatui::style::Color = ratatui::style::Color::Rgb(204, 144, 168);
```

## Usage Examples

Run the scaffold:

```bash
cargo run -p bardo-terminal
```

In an interactive frame loop, press:

- `q` to exit cleanly
- `Tab` to move forward through the ordered catalog
- `Shift+Tab` to move backward

Resize the terminal to see `LayoutBreakpoint` changes reflected in the frame chrome.

If you are integrating tests or future plans, the following instantiation pattern is what the scaffold uses:

```rust
let mut terminal = setup_terminal()?;
let mut app = App::new();
let result = app.run(&mut terminal);
teardown_terminal(terminal)?;
```

## Configuration

Runtime is controlled mostly by environment only through standard `tracing` initialization:

- `RUST_LOG` for startup/shutdown diagnostics

Terminal behavior is intentionally fixed to scaffold mode for now:

- raw-mode enable/disable around the run loop
- fixed target fps (`60` via ~16.67ms frame duration)
- no external config file layer in this plan

## Architecture

The app lifecycle is:

1. Install panic hook early (`std::panic::take_hook`)
2. Set up terminal backend and alternate screen (`enable_raw_mode`, `EnterAlternateScreen`, `EnableMouseCapture`)
3. Build `App` and register all `ScreenId` variants (`HomeScreen` + `StubScreen`s)
4. Loop at frame cadence:
   - event polling using remaining frame budget
   - action dispatch (`Quit`/`Next`/`Prev`/`Resize`)
   - tick increment
   - draw via `terminal.draw`
   - sleep until next frame when budget remains
5. Teardown regardless of run result (`disable_raw_mode`, `LeaveAlternateScreen`, cursor show)

## Spec Notes

- `bardo-terminal` appears in the main interface spec as the 60 FPS TUI and terminal visual layer entry point [`03-tui.md`](../../../prd2/18-interfaces/03-tui.md).
- ROSEDUST values and character constraints come from the design-system token tables and terminal-materiality guidance [`00-design-system.md`](../../../prd2/18-interfaces/rendering/00-design-system.md).
- Screen names and ordering map to the 29-screen catalog [`00-screen-catalog.md`](../../../prd2/18-interfaces/screens/00-screen-catalog.md).
