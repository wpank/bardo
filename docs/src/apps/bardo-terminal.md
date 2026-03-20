# bardo-terminal

## What It Is

`bardo-terminal` is the workspace's interactive Rust TUI binary. It owns terminal setup and teardown, a 60 fps render loop, a 29-screen catalog grouped into six logical windows, responsive layout, shared application state, live system metrics, and a reusable widget layer for gauges, sparklines, feeds, tabs, progress bars, and overlays.

The application follows the terminal architecture described in `prd2/18-interfaces/01-cli.md` sections `TUI (Interactive Mode)`, `Architecture`, `Entry points`, `Screen navigation`, and `Render loop`, plus `prd2/20-styx/05-tui-experience.md` sections `1. Architecture`, `4. The Screen System (11 Views)`, and `5. Custom Widgets`.

## Features

- Raw-mode and alternate-screen lifecycle management
- 60 fps frame loop with bounded input polling
- 29-screen catalog grouped into six logical windows
- Shared `AppState` with layout breakpoint, progress state, atmosphere animation, and system metrics
- A home dashboard with pipeline progress, task timing, connection status, and system resource panels
- Reusable widgets for sparklines, gauges, feeds, tabs, progress bars, timelines, and keybinding overlays
- Responsive sidebar and content split driven by terminal width
- Stable `Tab` and `Shift+Tab` screen cycling across the screen catalog

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

Tracing uses the standard Rust logging environment:

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
- `state` defines `AppState`, `AppAction`, progress tracking, atmosphere animation, and live `SysMetrics`
- `layout` computes `LayoutBreakpoint` and the sidebar/content split
- `palette` defines the terminal color constants and glyphs
- `screens::home` provides the concrete home dashboard
- `widgets` contains reusable ratatui widgets such as `BrailleSparkline`, `VitalityGauge`, `ConfidenceGauge`, `TabBar`, `EventFeed`, and `KeyHelpOverlay`

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

### Reusable Widgets

```rust
pub(crate) struct BrailleSparkline {
    pub(crate) data: Vec<f64>,
    pub(crate) max_value: f64,
    pub(crate) color: ratatui::style::Color,
    pub(crate) label: Option<String>,
}

pub(crate) enum MockPhase {
    Thriving,
    Stable,
    Conservation,
    Declining,
    Terminal,
}

pub(crate) struct VitalityGauge {
    pub(crate) value: f64,
    pub(crate) label: String,
    pub(crate) phase: MockPhase,
}

pub(crate) struct ConfidenceGauge {
    pub(crate) value: f64,
    pub(crate) label: String,
}

pub(crate) struct AccuracyGauge {
    pub(crate) value: f64,
    pub(crate) label: String,
}

pub(crate) struct KeyBinding {
    pub(crate) key: String,
    pub(crate) description: String,
}

pub(crate) struct KeyHelpOverlay {
    pub(crate) bindings: Vec<KeyBinding>,
    pub(crate) visible: bool,
}
```

## Usage Examples

Register a custom placeholder screen:

```rust
use crate::screen::{ScreenId, ScreenRegistry, StubScreen};

fn build_registry() -> ScreenRegistry {
    let mut registry = ScreenRegistry::new();
    registry.register(Box::new(StubScreen::new(
        ScreenId::WorldSolaris,
        "WORLD / Solaris",
    )));
    registry
}
```

Select a layout breakpoint from the terminal width:

```rust
use crate::layout::LayoutBreakpoint;

fn choose_layout(cols: u16) -> LayoutBreakpoint {
    LayoutBreakpoint::from_cols(cols)
}
```

Render a compact sparkline:

```rust
use ratatui::{backend::TestBackend, style::Color, Terminal};

use crate::widgets::BrailleSparkline;

fn render_trace() {
    let backend = TestBackend::new(24, 2);
    let mut terminal = Terminal::new(backend).expect("terminal");

    terminal
        .draw(|frame| {
            frame.render_widget(
                BrailleSparkline {
                    data: vec![0.2, 0.3, 0.5, 0.8, 0.6, 0.4],
                    max_value: 1.0,
                    color: Color::Cyan,
                    label: Some("cpu".into()),
                },
                frame.size(),
            );
        })
        .expect("draw");
}
```

## Architecture

The runtime is organized around a small set of stable layers:

1. `main` owns terminal lifecycle, tracing setup, and panic-safe teardown.
2. `App` owns shared state, the render loop, and screen navigation.
3. `ScreenRegistry` stores the active screen set behind the `Screen` trait.
4. `compute_layout` partitions the frame into chrome and content regions.
5. Reusable widgets render compact summaries inside each screen.

This structure follows the TUI architecture and custom-widget guidance in `prd2/18-interfaces/01-cli.md` and `prd2/20-styx/05-tui-experience.md`.

## References

- `prd2/18-interfaces/01-cli.md` sections `TUI (Interactive Mode)`, `Architecture`, `Entry points`, `Screen navigation`, and `Render loop`
- `prd2/20-styx/05-tui-experience.md` sections `1. Architecture`, `4. The Screen System (11 Views)`, `Persistent Chrome`, `Responsive Layout`, and `5. Custom Widgets`
