# bardo-terminal

## What It Is

`bardo-terminal` is the workspace's primary Rust TUI binary. It owns terminal setup and teardown, the 60 fps render loop, the screen registry, responsive layout, the ROSEDUST palette, live system metrics, and the reusable widget layer used by terminal screens.

The current scaffold renders the `HEARTH > Overview` home screen and keeps the rest of the 29-screen catalog registered as navigable placeholders. Keyboard input still routes through the active screen layer today, so the crate behaves like a stable terminal scaffold rather than a fully centralized navigation shell.

## Module Overview

- `main` bootstraps alternate-screen mode, tracing, the app loop, and teardown
- `app` owns the runtime state machine, screen switching, and top/bottom chrome
- `screen` defines the `Screen` contract, the `ScreenId` catalog, and the registry
- `state` holds app state, progress tracking, system metrics, and the current action vocabulary
- `widgets` contains reusable rendering primitives, including the floating keybinding help overlay
- `screens` provides concrete screen implementations, currently led by `HomeScreen`

## Features

- Alternate-screen startup and shutdown with panic-hook cleanup
- 60 fps frame loop with frame-budgeted input polling
- Stable 29-screen catalog with `Tab` and `Shift+Tab` cycling
- Screen focus and blur hooks for stateful views
- Responsive breakpoints for compact, standard, wide, and ultra layouts
- ROSEDUST palette tokens and CRT-style glyph constants
- Home screen placeholder with a creature silhouette, progress bar, connection status, task list, and live system metrics
- Reusable floating keybinding help overlay
- Crate-local widget surface for sparklines, progress bars, feeds, gauges, tabs, status bars, heatmaps, and overlays

## Getting Started

Run the terminal:

```bash
cargo run -p bardo-terminal
```

Run the crate tests:

```bash
cargo test -p bardo-terminal
```

Useful controls while the TUI is running:

- `q` quits cleanly
- `Tab` moves to the next screen
- `Shift+Tab` moves to the previous screen
- resizing the terminal updates the layout breakpoint on the next frame

Tracing follows the standard Rust logging environment:

```bash
RUST_LOG=info cargo run -p bardo-terminal
```

## Configuration

`bardo-terminal` does not currently expose its own config file or CLI flags. Runtime behavior comes from the terminal itself and from the process environment:

| Input | Effect |
| --- | --- |
| `RUST_LOG` | Controls `tracing_subscriber` filtering |
| Terminal width | Selects the responsive breakpoint and sidebar width |
| Terminal color and Unicode support | Affects palette fidelity, box-drawing, and braille rendering |

The responsive layout helper uses these breakpoint thresholds:

| Columns | Layout |
| --- | --- |
| `0..=79` | Compact |
| `80..=119` | Standard |
| `120..=179` | Wide |
| `180+` | Ultra |

## API

The crate is a binary application, so the documented surface is crate-local. The types below are the core runtime model used by the terminal.

### Binary Entry Point

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()>
```

The entry point installs the panic hook, initializes tracing, puts the terminal into raw mode and alternate-screen mode, runs the app loop, and restores the terminal on exit.

### Runtime Scaffold

```rust
pub(crate) struct App {
    state: AppState,
    screens: ScreenRegistry,
    active_screen: ScreenId,
    should_quit: bool,
    sys_stats: SysStats,
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

`App::run` drives the 60 fps loop. Each frame polls input with the remaining frame budget, advances the tick counter, updates animation state, refreshes system metrics when the one-second window elapses, and renders the active screen.

### Screen System

```rust
pub(crate) trait Screen: Send + Sync {
    fn id(&self) -> ScreenId;
    fn title(&self) -> &str;
    fn render(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, state: &AppState);
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<AppAction>;
    fn on_focus(&mut self) {}
    fn on_blur(&mut self) {}
}

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

pub(crate) struct ScreenRegistry { /* ... */ }
impl ScreenRegistry {
    pub(crate) fn new() -> Self;
    pub(crate) fn register(&mut self, screen: Box<dyn Screen>);
    pub(crate) fn get(&self, id: &ScreenId) -> Option<&dyn Screen>;
    pub(crate) fn get_mut(&mut self, id: &ScreenId) -> Option<&mut dyn Screen>;
}

pub(crate) struct HomeScreen { /* ... */ }
pub(crate) struct StubScreen { /* ... */ }
```

`HomeScreen` implements the active `HEARTH` view. The other 28 screens are registered as `StubScreen` placeholders so tab cycling works across the entire catalog without panicking.

### State And Actions

```rust
pub(crate) struct AppState {
    pub(crate) tick_count: u64,
    pub(crate) connection_status: ConnectionStatus,
    pub(crate) vitality: MockVitality,
    pub(crate) layout: LayoutBreakpoint,
    pub(crate) atmosphere: Atmosphere,
    pub(crate) progress: ProgressState,
    pub(crate) sys: SysMetrics,
}

pub(crate) enum AppAction {
    Quit,
    NextScreen,
    PrevScreen,
    Resize(u16, u16),
}

pub(crate) enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
}

pub(crate) struct MockVitality {
    pub(crate) value: f64,
}

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

pub(crate) struct ProgressState { /* task list, start time */ }
pub(crate) struct TaskEntry {
    pub(crate) name: String,
    pub(crate) status: TaskStatus,
    pub(crate) estimated_secs: f64,
    pub(crate) elapsed_secs: f64,
}
pub(crate) enum TaskStatus {
    Pending,
    Active,
    Done,
}

impl ProgressState {
    pub(crate) fn tick(&mut self, dt: f64);
    pub(crate) fn progress_fraction(&self) -> f64;
    pub(crate) fn eta_remaining_secs(&self) -> f64;
    pub(crate) fn wall_elapsed_secs(&self) -> f64;
    pub(crate) fn is_complete(&self) -> bool;
}

pub(crate) fn format_duration(total_secs: f64) -> String;
```

### Navigation Help

```rust
pub(crate) struct KeyBinding {
    pub(crate) key: String,
    pub(crate) description: String,
}

pub(crate) struct KeyHelpOverlay {
    pub(crate) bindings: Vec<KeyBinding>,
    pub(crate) visible: bool,
}

impl ratatui::widgets::Widget for &KeyHelpOverlay
```

The help overlay is a floating widget that renders a centered keybinding panel over whatever is already on screen. Screens or higher-level UI code can keep a static list of `KeyBinding` rows and toggle `visible` when the operator opens the help surface.

## Usage Examples

Render the active home-screen progress bar the same way the scaffold does it:

```rust
use crate::widgets::TotalProgressBar;

frame.render_widget(
    TotalProgressBar {
        progress: state.progress.progress_fraction(),
        eta_secs: state.progress.eta_remaining_secs(),
        elapsed_secs: state.progress.wall_elapsed_secs(),
        heartbeat: state.atmosphere.heartbeat(),
        complete: state.progress.is_complete(),
    },
    progress_inner,
);
```

Build and render a floating keybinding cheat sheet:

```rust
use crate::widgets::{KeyBinding, KeyHelpOverlay};

let overlay = KeyHelpOverlay {
    bindings: vec![
        KeyBinding { key: "q".into(), description: "quit".into() },
        KeyBinding { key: "Tab".into(), description: "next screen".into() },
        KeyBinding { key: "Shift+Tab".into(), description: "previous screen".into() },
    ],
    visible: true,
};

frame.render_widget(&overlay, help_area);
```

Use the screen catalog helpers when composing chrome:

```rust
let window = active_screen.window_name();
let tab = active_screen.tab_name();
let header = format!("{window} / {tab}");
```

## Architecture

The binary follows a straightforward TUI lifecycle:

1. `main` installs a panic hook before touching the terminal so raw mode is restored even if the process panics.
2. Terminal setup enables raw mode, enters the alternate screen, and enables mouse capture.
3. `App::run` executes a frame loop at 60 fps.
4. Each frame polls input, updates animation state, refreshes system metrics when the one-second window elapses, and renders the active screen.
5. Screen navigation uses the full 29-screen catalog; the active screen receives `on_focus` and `on_blur` callbacks as the user cycles.
6. `compute_layout` reserves a top chrome row, a bottom chrome row, and a responsive sprite sidebar when the breakpoint allows it.
7. `HomeScreen` renders the current placeholder HEARTH surface, while the remaining screens stay stubbed but navigable.
8. Teardown always restores the terminal, disables raw mode, leaves the alternate screen, and shows the cursor again.

The live input path is still screen-local: the active screen gets first crack at `KeyEvent` values, which is why the home screen currently handles `q`, `Tab`, and `Shift+Tab` directly. The reusable key-help widget is ready for a centralized navigation layer once that wiring is introduced.

The visual language matches the ROSEDUST design system: rose-dominant color, bone as the single emphasized highlight, never pure black, and box-drawing borders with CRT-style motion. The home screen's pulsing border, waveform, and animated progress bar are the current scaffold's implementation of that motion vocabulary.

## References

- `prd2/18-interfaces/01-cli.md` sections `TUI (Interactive Mode)`, `Entry points`, `Screen navigation`, and `Render loop`
- `prd2/20-styx/05-tui-experience.md` sections `Architecture`, `Persistent Chrome`, `The Screen System (11 Views)`, and `Responsive Layout`
- `prd2/18-interfaces/rendering/00-design-system.md` sections `The ROSEDUST palette`, `The 7 rendering laws`, and `Character vocabulary`
- `prd2/18-interfaces/screens/00-screen-catalog.md` sections `29-Screen Summary` and `Navigation Model`
- `prd2/13-runtime/19-cinematic-system.md` sections `Design philosophy` and `Time is the primary rendering dimension`
