# bardo-terminal navigation

## What It Is

The navigation layer is the terminal's typed keyboard router. It receives raw `crossterm::event::KeyEvent` values, resolves them through the global and per-screen keybinding maps, and then routes the result into the command palette, modal stack, vim mode state machine, or active screen.

This is the part of the terminal that makes the 30-screen interface feel like a single application instead of a set of unrelated views.

## Features

- Global and per-screen keybindings backed by a TOML config file
- Direct screen jumps and window jumps from a single typed action surface
- `/` command palette with fuzzy search and keyboard navigation
- Stack-based modals for confirm, input, and alert flows
- Optional vim mode with `Normal`, `Insert`, and `Command` states
- Consistent `Esc` behavior that closes the topmost overlay first
- Help overlay integration so the current binding set is always discoverable

## Getting Started

The default bindings work out of the box:

```bash
cargo run -p bardo-terminal
```

Useful starting keys:

- `?` toggles the help overlay
- `/` opens the command palette
- `Esc` closes the topmost modal or overlay
- `Tab` and `Shift+Tab` cycle screens
- `1` through `6` jump to the six logical windows
- `:` enters vim command mode when vim mode is enabled

## Configuration

The navigation layer loads overrides from `~/.bardo/keybindings.toml` and falls back to the built-in defaults when the file is missing.

```toml
# ~/.bardo/keybindings.toml
[global]
"ctrl+c" = "Quit"
"tab" = "NextScreen"
"backtab" = "PrevScreen"
"?" = "ShowHelp"
"/" = "OpenCommandPalette"
"F1" = "GotoWindow:Hearth"

[screen.HearthOverview]
"r" = "ScrollTop"
"g" = "ScrollBottom"
```

Supported key strings are case-insensitive. The parser accepts:

- Modifier prefixes: `ctrl+`, `shift+`, and `alt+`
- Special keys: `tab`, `backtab`, `esc`, `enter`, `backspace`, `up`, `down`, `left`, and `right`
- Function keys: `F1` through `F12`
- Single-character keys

Action strings deserialize into `AppAction` variants. Unknown actions are ignored instead of failing config loading, which keeps a partially valid override file usable.

## API

### Typed Actions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowId {
    Hearth,
    Mind,
    Soma,
    World,
    Fate,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum AppAction {
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
```

### Keybinding Map

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
    pub fn load_from_toml(path: &std::path::Path) -> color_eyre::Result<Self>;
    pub fn resolve(
        &self,
        key: crossterm::event::KeyEvent,
        screen: ScreenId,
    ) -> Option<AppAction>;
}

pub fn parse_key_str(s: &str) -> Option<crossterm::event::KeyEvent>;
```

`resolve` checks per-screen bindings before global bindings. `parse_key_str` is the shared helper used by the default table and the TOML loader.

### Command Palette

```rust
#[derive(Debug, Clone)]
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
    pub fn render(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect);
}

pub fn default_commands() -> Vec<Command>;
```

The palette filters commands by the length of the longest common subsequence between the query and the command name. Commands with a zero score are hidden, and the selected row is clamped to the filtered list length after every update.

### Modal Manager

```rust
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
    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Option<AppAction>;
    pub fn render(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect);
}
```

`Confirm` maps `Enter` or `y` to the confirm action and `Esc` or `n` to the cancel action. `Input` collects text until `Enter` submits or `Esc` cancels. `Alert` closes on `Enter`, `Esc`, or space.

### Vim Mode

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn process_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Option<AppAction>;
}
```

`Normal` mode handles `hjkl`, `gg`, `G`, `:`, and `i`. `Insert` mode only reacts to `Esc`. `Command` mode accumulates text until `Enter` converts the buffer into `AppAction::VimCommand`.

## Usage Examples

Load custom bindings with the default fallback:

```rust
use std::path::Path;

use crate::navigation::KeybindingMap;

fn load_bindings() -> KeybindingMap {
    KeybindingMap::load_from_toml(Path::new("/home/will/.bardo/keybindings.toml"))
        .unwrap_or_else(|_| KeybindingMap::default_bindings())
}
```

Dispatch a palette command:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::navigation::CommandPalette;

let mut palette = CommandPalette {
    visible: true,
    query: String::new(),
    commands: crate::navigation::default_commands(),
    filtered: Vec::new(),
    selected: 0,
};

palette.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
palette.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
```

React to vim commands:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::navigation::VimModeState;

let mut vim = VimModeState::new(true);
vim.process_key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
vim.process_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
let action = vim.process_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
assert!(matches!(action, Some(crate::state::AppAction::VimCommand(_))));
```

## Architecture

The navigation pipeline runs in this order:

1. Vim mode gets first crack at input when it is enabled.
2. If a modal is open, the modal stack handles `Enter`, `Esc`, text entry, and backspace.
3. If the command palette is visible, it consumes query and selection keys.
4. The keybinding map resolves per-screen and global bindings.
5. The active screen receives any key that is still unhandled.

That flow matches the terminal interaction model described in [prd2/18-interfaces/01-cli.md](../../../prd2/18-interfaces/01-cli.md) sections `TUI (Interactive Mode)` and `Screen navigation`, and [prd2/20-styx/05-tui-experience.md](../../../prd2/20-styx/05-tui-experience.md) sections `Persistent Chrome` and `The Screen System (11 Views)`.
