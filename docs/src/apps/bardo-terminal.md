# bardo-terminal

## What It Is

`bardo-terminal` is the workspace's interactive Rust TUI binary. It owns terminal setup and teardown, a 60 fps render loop, the 29-screen catalog, responsive layout, shared application state, live system metrics, and the crate-local widget layer that the later screens compose.

The current implementation follows the terminal architecture described in `prd2/18-interfaces/01-cli.md` sections `TUI (Interactive Mode)`, `Architecture`, `Entry points`, `Screen navigation`, and `Render loop`, plus `prd2/20-styx/05-tui-experience.md` sections `1. Architecture`, `4. The Screen System (11 Views)`, and `5. Custom Widgets`. The color system is anchored in `prd2/18-interfaces/rendering/00-design-system.md`.

## Features

- Raw-mode and alternate-screen lifecycle management with panic-hook recovery
- 60 fps frame loop with bounded input polling and frame skipping on overruns
- 29-screen catalog grouped into six logical windows
- Stable `Tab` and `Shift+Tab` screen cycling across the catalog
- Shared `AppState` with layout breakpoint, atmosphere animation, progress tracking, placeholder vitality, and live system metrics
- Home dashboard with creature silhouette, pipeline progress, per-task timing, connection status, and system resource panels
- Responsive sidebar/content split driven by terminal width
- ROSEDUST palette tokens and CRT-style box-drawing glyphs
- Reusable widgets for sparklines, gauges, feeds, tabs, progress bars, timelines, and help overlays

## Getting Started

Run the terminal:

```bash
cargo run -p bardo-terminal
```

Run the crate tests:

```bash
cargo test -p bardo-terminal
```

Useful live controls:

- `q` quits
- `Tab` moves to the next screen in `ScreenId::all()`
- `Shift+Tab` moves to the previous screen
- Resizing the terminal recomputes `LayoutBreakpoint`

Enable tracing with the standard Rust logging environment:

```bash
RUST_LOG=info cargo run -p bardo-terminal
```

## Configuration

The binary currently relies on runtime environment and terminal capabilities rather than a dedicated application config file.

| Input | Effect |
| --- | --- |
| `RUST_LOG` | Controls `tracing_subscriber` filtering |
| Terminal width | Selects `Compact`, `Standard`, `Wide`, or `Ultra` layout |
| Terminal color and Unicode support | Affects palette fidelity, box drawing, braille sparklines, and block-glyph gauges |

## Module Overview

- `main` boots tracing, installs the panic hook, enters raw mode and alternate-screen mode, and runs the app loop
- `app` owns `AppState`, `ScreenRegistry`, screen switching, chrome rendering, and the frame loop
- `screen` defines `Screen`, `ScreenId`, `ScreenRegistry`, and `StubScreen`
- `state` defines `AppState`, `AppAction`, placeholder vitality, connection status, atmosphere animation, progress tracking, and live `SysMetrics`
- `layout` computes `LayoutBreakpoint` and the sidebar/content split
- `palette` defines the terminal color constants, style modifiers, and box-drawing glyphs
- `screens::home` provides the concrete home dashboard
- `sys_stats` samples CPU, memory, network, and disk metrics into `SysMetrics`
- `widgets` contains reusable ratatui widgets such as `BrailleSparkline`, `TotalProgressBar`, `TabBar`, `EventFeed`, and `KeyHelpOverlay`

## API

### Binary Entry Point

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

The entrypoint installs the panic hook, initializes tracing, sets up the terminal, runs `App::run`, and restores terminal state on exit.

### Runtime Scaffold

```rust
pub(crate) struct App {
    /* private fields omitted */
}

impl App {
    pub(crate) fn new() -> Self;
    pub(crate) fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<
            ratatui::backend::CrosstermBackend<std::io::Stdout>,
        >,
    ) -> anyhow::Result<()>;
}
```

`App::new()` registers `HomeScreen` and fills the remaining screen catalog with `StubScreen` placeholders. `App::run()` polls `crossterm` events, advances shared state, refreshes system metrics, and renders the current frame.

### Screen System

```rust
pub(crate) trait Screen: Send + Sync {
    fn id(&self) -> ScreenId;
    fn title(&self) -> &str;
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::layout::Rect,
        state: &AppState,
    );
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<AppAction>;
    fn on_focus(&mut self) {}
    fn on_blur(&mut self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

pub(crate) struct ScreenRegistry {
    /* private fields omitted */
}

impl ScreenRegistry {
    pub(crate) fn new() -> Self;
    pub(crate) fn register(&mut self, screen: Box<dyn Screen>);
    pub(crate) fn get(&self, id: &ScreenId) -> Option<&dyn Screen>;
    pub(crate) fn get_mut(&mut self, id: &ScreenId) -> Option<&mut dyn Screen>;
}

pub(crate) struct StubScreen {
    /* private fields omitted */
}

impl StubScreen {
    pub(crate) fn new(id: ScreenId, title: impl Into<String>) -> Self;
}

pub(crate) struct HomeScreen {
    /* private fields omitted */
}

impl HomeScreen {
    pub(crate) fn new() -> Self;
}
```

### Shared State And Layout

```rust
#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) tick_count: u64,
    pub(crate) connection_status: ConnectionStatus,
    pub(crate) vitality: MockVitality,
    pub(crate) layout: LayoutBreakpoint,
    pub(crate) atmosphere: Atmosphere,
    pub(crate) progress: ProgressState,
    pub(crate) sys: SysMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppAction {
    Quit,
    NextScreen,
    PrevScreen,
    Resize(u16, u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutBreakpoint {
    Compact,
    Standard,
    Wide,
    Ultra,
}

impl LayoutBreakpoint {
    pub(crate) const fn from_cols(cols: u16) -> Self;
    pub(crate) const fn sprite_sidebar_cols(self) -> u16;
    pub(crate) const fn panel_count(self) -> u8;
    pub(crate) const fn label(self) -> &'static str;
}

pub(crate) fn compute_layout(
    frame_size: ratatui::layout::Rect,
    bp: LayoutBreakpoint,
) -> (ratatui::layout::Rect, ratatui::layout::Rect);

pub(crate) fn format_duration(secs: f64) -> String;
```

### Palette Tokens

```rust
pub(crate) struct ColorPalette;

pub(crate) const BG_VOID: ratatui::style::Color;
pub(crate) const BG_RAISED: ratatui::style::Color;
pub(crate) const BG_MID: ratatui::style::Color;
pub(crate) const BORDER: ratatui::style::Color;
pub(crate) const BORDER_ACTIVE: ratatui::style::Color;
pub(crate) const ROSE: ratatui::style::Color;
pub(crate) const ROSE_BRIGHT: ratatui::style::Color;
pub(crate) const BONE: ratatui::style::Color;
pub(crate) const TEXT_PRIMARY: ratatui::style::Color;
pub(crate) const DREAM: ratatui::style::Color;
pub(crate) const WARNING: ratatui::style::Color;
pub(crate) const SUCCESS: ratatui::style::Color;
pub(crate) const DANGER: ratatui::style::Color;
```

## Usage Examples

Start the binary and navigate between screens:

```bash
cargo run -p bardo-terminal
```

The home screen shows the live scaffold state:

- a creature silhouette in the left panel
- a pipeline progress bar with ETA in the right panel
- connection status and per-task timing
- a four-column system panel for CPU, memory, network, and disk

The responsive layout reacts to terminal width:

```rust
let breakpoint = bardo_terminal::layout::LayoutBreakpoint::from_cols(width);
let panel_count = breakpoint.panel_count();
let sidebar_cols = breakpoint.sprite_sidebar_cols();
```

`Tab` and `Shift+Tab` cycle through the screen catalog in the exact order returned by `ScreenId::all()`.

## Architecture

```
main
├── install panic hook
├── setup terminal
├── App::new()
├── App::run()
│   ├── poll input
│   ├── tick state
│   ├── sample system metrics
│   ├── render chrome + active screen
│   └── sleep to maintain 60 fps
└── teardown terminal

app
├── screen registry
├── screen switching
├── responsive chrome
└── content rendering

screens::home
├── creature placeholder
├── progress bar
├── connection state
├── task timing
└── system resources
```

## References

- `prd2/18-interfaces/01-cli.md`
  - `TUI (Interactive Mode)`
  - `Architecture`
  - `Entry points`
  - `Screen navigation`
  - `Render loop`
- `prd2/18-interfaces/rendering/00-design-system.md`
  - `The ROSEDUST palette`
  - `The 7 rendering laws`
  - `Character vocabulary`
- `prd2/18-interfaces/screens/00-screen-catalog.md`
  - `29-Screen Summary`
  - `Navigation Model`
- `prd2/20-styx/05-tui-experience.md`
  - `1. Architecture`
  - `4. The Screen System (11 Views)`
  - `5. Custom Widgets`
- `prd2/13-runtime/19-cinematic-system.md`
  - `Design philosophy`
  - `Time is the primary rendering dimension`
