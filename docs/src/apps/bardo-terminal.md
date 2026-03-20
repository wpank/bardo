# bardo-terminal

## What It Is

`bardo-terminal` is the workspace's Rust TUI binary. It owns terminal setup and teardown, the 60 fps render loop, the 29-screen catalog, responsive layout, the ROSEDUST palette, live system metrics, and the chrome around the active screen.

The current scaffold keeps keyboard handling screen-local: the active screen decides when to emit `AppAction` values such as `Quit`, `NextScreen`, `PrevScreen`, or `Resize`. That matches the PRD's persistent-chrome and 29-screen navigation model while leaving room for a later shared navigation layer.

## Module Overview

- `main` boots raw mode, alternate-screen mode, tracing, and the app loop
- `app` owns the runtime state machine, screen switching, and top/bottom chrome
- `screen` defines the `Screen` trait, the `ScreenId` catalog, and the screen registry
- `state` holds the shared application state, progress tracking, system metrics, and app actions
- `palette` defines the ROSEDUST colors and box-drawing glyphs used by the terminal
- `widgets` contains reusable ratatui widgets, including the floating keybinding help overlay
- `screens` contains concrete screen implementations, currently led by `HomeScreen`

## Features

- Alternate-screen startup and shutdown with panic-hook cleanup
- 60 fps frame loop with frame-budgeted input polling
- Stable 29-screen catalog grouped into 6 logical windows
- Screen cycling with `Tab` and `Shift+Tab`
- Header and footer chrome that show the active screen, ETA, elapsed time, and layout state
- Responsive layout breakpoints for compact, standard, wide, and ultra terminals
- Home screen with a creature silhouette, pipeline progress, task list, and live system metrics
- Reusable widget layer for sparklines, progress bars, feeds, tabs, status bars, and the help overlay

## Getting Started

Run the terminal:

```bash
cargo run -p bardo-terminal
```

Run the crate tests:

```bash
cargo test -p bardo-terminal
```

Useful controls in the current scaffold:

- `q` quits from the active screen when that screen emits `AppAction::Quit`
- `Tab` moves to the next screen
- `Shift+Tab` moves to the previous screen
- resizing the terminal updates the layout breakpoint on the next frame

Tracing follows the standard Rust logging environment:

```bash
RUST_LOG=info cargo run -p bardo-terminal
```

## Configuration

`bardo-terminal` does not currently read its own config file. Its behavior comes from the terminal, the environment, and the built-in app state defaults.

| Input | Effect |
| --- | --- |
| `RUST_LOG` | Controls `tracing_subscriber` filtering |
| Terminal width | Selects the responsive breakpoint and sidebar width |
| Terminal color and Unicode support | Affects palette fidelity, box-drawing, and braille rendering |

The crate does not currently load `~/.bardo/keybindings.toml` or any other app-specific override file.

## API

### Binary Entry Point

```rust
async fn main() -> anyhow::Result<()>
```

The entrypoint installs the panic hook, initializes tracing, enters raw mode and the alternate screen, runs the app loop, and restores the terminal on exit.

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

`App::run` drives the 60 fps loop. Each frame polls input with the remaining frame budget, advances shared state, refreshes system metrics when the one-second window elapses, and renders the active screen.

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

pub(crate) struct ScreenRegistry { /* ... */ }
impl ScreenRegistry {
    pub(crate) fn new() -> Self;
    pub(crate) fn register(&mut self, screen: Box<dyn Screen>);
    pub(crate) fn get(&self, id: &ScreenId) -> Option<&dyn Screen>;
    pub(crate) fn get_mut(&mut self, id: &ScreenId) -> Option<&mut dyn Screen>;
}

pub(crate) struct StubScreen { /* ... */ }
impl StubScreen {
    pub(crate) fn new(id: ScreenId, title: impl Into<String>) -> Self;
}

pub(crate) struct HomeScreen { /* ... */ }
impl HomeScreen {
    pub(crate) fn new() -> Self;
}
```

`ScreenId::all()` returns the stable 29-screen order used for tab cycling. `window_name()` and `tab_name()` supply the chrome labels shown in the header and sidebar.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppAction {
    Quit,
    NextScreen,
    PrevScreen,
    Resize(u16, u16),
}
```

Supporting state types keep the current scaffold self-contained:

```rust
pub(crate) enum ConnectionStatus { Connected, Disconnected, Connecting }
pub(crate) struct MockVitality { pub(crate) value: f64 }
pub(crate) enum TaskStatus { Pending, Active, Done }
pub(crate) struct TaskEntry { pub(crate) name: String, pub(crate) status: TaskStatus, pub(crate) estimated_secs: f64, pub(crate) elapsed_secs: f64 }
pub(crate) struct ProgressState { pub(crate) tasks: Vec<TaskEntry>, pub(crate) start_time: std::time::Instant }
pub(crate) struct Atmosphere { pub(crate) elapsed_secs: f64, pub(crate) dt: f64 }
pub(crate) struct SysMetrics { /* live CPU, memory, network, and disk snapshots */ }

pub(crate) fn format_duration(total_secs: f64) -> String;
```

`ProgressState` tracks per-task and aggregate timing. `format_duration()` is used by the chrome and the home screen to present elapsed time and ETA values.

### Widget Surface

```rust
pub(crate) struct KeyBinding {
    pub(crate) key: String,
    pub(crate) description: String,
}

pub(crate) struct KeyHelpOverlay {
    pub(crate) bindings: Vec<KeyBinding>,
    pub(crate) visible: bool,
}

impl ratatui::widgets::Widget for &KeyHelpOverlay;
```

`KeyHelpOverlay` is the reusable floating help widget for keybinding hints. It is part of the crate's widget layer even though the current app shell still renders its chrome manually.

## Usage Example

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::screen::{ScreenId, ScreenRegistry};
use crate::screens::HomeScreen;
use crate::state::AppAction;

fn build_registry() {
    let mut registry = ScreenRegistry::new();
    registry.register(Box::new(HomeScreen::new()));

    let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    let action = AppAction::NextScreen;

    assert_eq!(ScreenId::all().len(), 29);
    assert!(matches!(action, AppAction::NextScreen));
    assert_eq!(tab.code, KeyCode::Tab);
    assert!(registry.get(&ScreenId::HearthOverview).is_some());
}
```

## Architecture

The terminal is structured around a single render loop:

1. `crossterm` input is polled on each frame
2. the active screen turns key events into `AppAction` values
3. `App` updates shared state and layout from those actions
4. the active screen renders into the content area
5. the shared chrome and reusable widgets fill in the surrounding view

The PRD describes the richer 29-screen / 6-window navigation model and the persistent chrome that surrounds it. This scaffold already provides the screen catalog, chrome layout, and responsive breakpoints that those interactions sit on top of.

## References

- `prd2/18-interfaces/01-cli.md` sections `Architecture` and `Screen navigation`
- `prd2/20-styx/05-tui-experience.md` sections `Architecture`, `Persistent chrome`, `The 11 Screens`, and `Responsive Layout`
- `prd2/18-interfaces/screens/00-screen-catalog.md` sections `29-Screen Summary` and `Navigation Model`
- `prd2/20-styx/05-tui-experience.md` section `The Screen System`
