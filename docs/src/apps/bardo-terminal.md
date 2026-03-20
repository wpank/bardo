# bardo-terminal

## What It Is

`bardo-terminal` is the workspace's primary Rust TUI binary. It owns terminal setup and teardown, the 60 fps render loop, the screen registry, responsive layout, the ROSEDUST palette, live system metrics, the reusable widget layer, and the centralized navigation shell that turns raw key events into typed app actions.

The interaction layer covers global keybindings, per-screen overrides, the command palette, modal routing, and vim-style command modes. That keeps the global chrome consistent while still letting individual screens handle local input once the navigation layer has had first pass.

## Module Overview

- `main` bootstraps alternate-screen mode, tracing, the app loop, and teardown
- `app` owns the runtime state machine, input routing, screen switching, and top/bottom chrome
- `navigation` centralizes keybinding resolution, command palette filtering and rendering, modal stack handling, and vim mode state
- `navigation/keybindings` defines the key-to-action map and TOML override loader
- `navigation/palette` defines the command registry, fuzzy filtering, and floating launcher overlay
- `navigation/modal` defines the stack-based modal manager and centered dialogs
- `navigation/vim` defines the Normal / Insert / Command state machine and colon-command buffer
- `screen` defines the `Screen` contract, the `ScreenId` catalog, and the registry
- `state` holds app state, progress tracking, system metrics, and the expanded action vocabulary
- `widgets` contains reusable rendering primitives, including the floating keybinding help overlay
- `screens` provides concrete screen implementations, currently led by `HomeScreen`

## Features

- Alternate-screen startup and shutdown with panic-hook cleanup
- 60 fps frame loop with frame-budgeted input polling
- Stable 29-screen catalog with `Tab` and `Shift+Tab` cycling
- Global keybinding map with screen-specific overrides and config-backed defaults
- Direct window jumps for the six terminal windows and sidebar shortcuts
- Command palette overlay with fuzzy search and keyboard selection
- Stack-based modal handling for confirm, input, and alert dialogs
- Vim mode with Normal, Insert, and Command states
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
- `Ctrl+C` also quits through the default global binding
- `Tab` moves to the next screen
- `Shift+Tab` moves to the previous screen
- `1` through `6` jump directly to the six logical windows
- `/` opens the command palette
- `?` toggles the help overlay
- `Esc` closes the top modal, the palette, or the help overlay, depending on what is visible
- `i`, `:`, `h`, `j`, `k`, `l`, `gg`, and `G` work when vim mode is enabled
- resizing the terminal updates the layout breakpoint on the next frame

Tracing follows the standard Rust logging environment:

```bash
RUST_LOG=info cargo run -p bardo-terminal
```

## Configuration

`bardo-terminal` does not currently expose its own CLI flags. Runtime behavior comes from the terminal itself, the process environment, and the optional keybinding override file:

| Input | Effect |
| --- | --- |
| `RUST_LOG` | Controls `tracing_subscriber` filtering |
| Terminal width | Selects the responsive breakpoint and sidebar width |
| Terminal color and Unicode support | Affects palette fidelity, box-drawing, and braille rendering |
| `~/.bardo/keybindings.toml` | Overrides the default keybinding map when present |

The keybinding loader uses the workspace's TOML dependency and treats missing files, unknown key strings, and unknown action names as non-fatal. Built-in defaults remain in place whenever a user override cannot be parsed.

```toml
# ~/.bardo/keybindings.toml
[global]
"ctrl+c" = "Quit"
"tab" = "NextScreen"
"F1" = "GotoWindow:Hearth"

[screen.HearthOverview]
"r" = "ScrollTop"
```

Key strings are case-insensitive. Modifier prefixes use `ctrl+`, `shift+`, and `alt+`. Supported special keys include `tab`, `backtab`, `esc`, `enter`, `backspace`, `up`, `down`, `left`, and `right`.

The responsive layout helper uses these breakpoint thresholds:

| Columns | Layout |
| --- | --- |
| `0..=79` | Compact |
| `80..=119` | Standard |
| `120..=179` | Wide |
| `180+` | Ultra |

## API

The crate is a binary application, so the documented surface is crate-local. The types below are the core runtime and navigation model used by the terminal.

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
    keybindings: KeybindingMap,
    command_palette: CommandPalette,
    modal_manager: ModalManager,
    vim_mode_state: VimModeState,
    show_help: bool,
}

impl App {
    pub(crate) fn new() -> Self;
    pub(crate) fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<
            ratatui::backend::CrosstermBackend<std::io::Stdout>,
        >,
    ) -> anyhow::Result<()>;
    pub(crate) fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<AppAction>;
    pub(crate) fn apply_action(&mut self, action: AppAction);
}
```

`App::run` drives the 60 fps loop. Each frame polls input with the remaining frame budget, advances the tick counter, updates animation state, refreshes system metrics when the one-second window elapses, and renders the active screen.

`App::handle_key` applies the routing order used by the navigation layer:

1. Vim mode intercepts keys while vim mode is enabled.
2. The top-most modal handles confirm, input, and alert events.
3. The command palette handles query input and selection.
4. The keybinding map resolves per-screen and global shortcuts.
5. Any remaining key is forwarded to the active screen.

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
    GotoScreen(ScreenId),
    GotoWindow(WindowId),
    OpenCommandPalette,
    CloseCommandPalette,
    ExecuteCommand(usize),
    PaletteInput(char),
    PaletteBackspace,
    PaletteSelectNext,
    PaletteSelectPrev,
    ShowHelp,
    HideHelp,
    CloseModal,
    ConfirmModal,
    ModalInput(char),
    ModalBackspace,
    EnterVimMode,
    ExitVimMode,
    VimNavigate(VimDirection),
    VimCommand(String),
    ScrollUp,
    ScrollDown,
    ScrollTop,
    ScrollBottom,
}

pub(crate) enum WindowId {
    Hearth,
    Mind,
    Soma,
    World,
    Fate,
    Command,
}

pub(crate) enum VimDirection {
    Up,
    Down,
    Left,
    Right,
}
```

### Navigation Surface

```rust
pub struct KeybindingMap {
    pub global: std::collections::HashMap<crossterm::event::KeyEvent, AppAction>,
    pub per_screen: std::collections::HashMap<
        ScreenId,
        std::collections::HashMap<crossterm::event::KeyEvent, AppAction>,
    >,
    pub vim_mode: bool,
}

impl KeybindingMap {
    pub fn default_bindings() -> Self;
    pub fn load_from_toml(path: &std::path::Path) -> anyhow::Result<Self>;
    pub fn resolve(
        &self,
        key: crossterm::event::KeyEvent,
        screen: ScreenId,
    ) -> Option<AppAction>;
}

pub struct Command {
    pub name: String,
    pub description: String,
    pub action: AppAction,
    pub keybinding: Option<String>,
}

pub struct CommandPalette {
    pub visible: bool,
    pub query: String,
    pub commands: Vec<Command>,
    pub filtered: Vec<usize>,
    pub selected: usize,
}

impl CommandPalette {
    pub fn update_filter(&mut self);
    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Option<AppAction>;
}

pub enum Modal {
    Confirm {
        title: String,
        message: String,
        on_confirm: AppAction,
        on_cancel: Option<AppAction>,
    },
    Input {
        title: String,
        placeholder: String,
        buffer: String,
        on_submit: Box<dyn Fn(String) -> AppAction + Send>,
    },
    Alert {
        title: String,
        message: String,
    },
}

pub struct ModalManager {
    pub stack: Vec<Modal>,
}

impl ModalManager {
    pub fn new() -> Self;
    pub fn push(&mut self, modal: Modal);
    pub fn pop(&mut self) -> Option<Modal>;
    pub fn has_modal(&self) -> bool;
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<AppAction>;
    pub fn render(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect);
}

pub enum VimMode {
    Normal,
    Insert,
    Command,
}

pub struct VimModeState {
    pub mode: VimMode,
    pub command_buffer: String,
    pub sequence_buffer: Vec<char>,
    pub enabled: bool,
}

impl VimModeState {
    pub fn new(enabled: bool) -> Self;
    pub fn active(&self) -> bool;
    pub fn mode_indicator(&self) -> &str;
    pub fn process_key(&mut self, key: crossterm::event::KeyEvent) -> Option<AppAction>;
}
```

The command palette and modal manager are both overlay systems: they render above the active screen and consume the relevant keys before the screen sees them. The modal stack handles nested alerts and confirmations, while the palette keeps its filtered command indices sorted by score.

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
```

`KeyHelpOverlay` is the reusable floating help widget that the navigation layer uses when the user toggles `?`.

### Usage Example

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use bardo_terminal::{
    navigation::{CommandPalette, KeybindingMap},
    screen::ScreenId,
    state::AppAction,
};

fn example() -> anyhow::Result<()> {
    let bindings = KeybindingMap::default_bindings();
    let action = bindings.resolve(
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        ScreenId::HearthOverview,
    );

    assert!(matches!(action, Some(AppAction::Quit)));

    let mut palette = CommandPalette {
        visible: true,
        query: "qui".to_owned(),
        commands: Vec::new(),
        filtered: Vec::new(),
        selected: 0,
    };
    palette.update_filter();

    Ok(())
}
```

## Architecture

The terminal is structured around a single render loop that layers interaction, chrome, and content in a fixed order:

1. input reaches the navigation layer first
2. the app updates shared state and layout from the resulting action
3. the active screen renders its content
4. the reusable widgets and overlays fill in the supporting pieces of the view

The screen catalog stays stable so the same keybindings and tab order remain available while the placeholder screens are replaced with real views.

## References

- `prd2/18-interfaces/01-cli.md` sections `TUI entry points` and `Screen navigation`
- `prd2/18-interfaces/rendering/00-design-system.md` sections `The ROSEDUST palette`, `The 7 rendering laws`, and `Character vocabulary`
- `prd2/18-interfaces/screens/00-screen-catalog.md` sections `29-Screen Summary` and `Navigation Model`
- `prd2/20-styx/05-tui-experience.md` sections `Architecture`, `Persistent chrome`, `Sidebar navigation keys`, and `Responsive Layout`
- `prd2/13-runtime/19-cinematic-system.md` sections `Design philosophy` and `Time is the primary rendering dimension`
