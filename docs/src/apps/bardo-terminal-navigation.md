# bardo-terminal navigation

## What It Is

`bardo-terminal` routes every terminal keypress through a typed navigation layer before any screen-specific handler runs. This layer translates raw `crossterm::event::KeyEvent` values into `AppAction` variants, manages modal overlays, drives the `/` command palette, and optionally enables vim-style navigation semantics.

The implementation backs the persistent chrome and keyboard-first navigation model described in `prd2/18-interfaces/01-cli.md` sections `TUI (Interactive Mode)` and `The 15-screen system`, `prd2/18-interfaces/03-tui.md` sections `3. Crate architecture`, `5.1 Persistent chrome`, `5.2 Screen map`, and `5.3 Screen details`, plus `prd2/20-styx/05-tui-experience.md` section `4. The Screen System (11 Views)`.

The **30** `ScreenId` tabs registered in this binary are ordered by `ScreenId::all()`. Older PRD language (“15-screen system”, “11 views”) describes coarser groupings. Window names in chrome follow the spirit of `prd2/18-interfaces/21-screen-catalog.md`, with an additional **PROTOCOL** window for the Protocol Views screen; when in doubt, trust the live registry, not a static tab count in prose.

## Features

- A single typed `AppAction` surface for quitting, screen changes, overlays, scrolling, and vim commands
- Global keybindings plus per-screen overrides loaded from `~/.bardo/keybindings.toml`
- Direct window jumps with `1`-`7` and `F1`-`F7`
- Direct screen jumps with sidebar letters such as `h`, `b`, `m`, `f`, `v`, `w`, and `x`
- `/` command palette with LCS-based fuzzy filtering and keyboard selection
- Stack-based confirm, input, and alert modals
- Optional vim mode with `Normal`, `Insert`, and `Command` states
- Layered `Esc` handling that closes the top-most transient surface first
- Help overlay hints that document the active navigation vocabulary in the running terminal

## Getting Started

Run the terminal:

```bash
cargo run -p bardo-terminal
```

Default navigation keys:

- `q` or `Ctrl+C` quits
- `Tab` and `Shift+Tab` cycle through the 30 registered screens
- `1`-`7` or `F1`-`F7` jump to the root screen for each top-level window
- `?` toggles the help overlay
- `/` opens the command palette
- `Esc` closes the current modal, palette, help overlay, or vim command buffer in that order

Vim mode is present in the codebase but disabled by default in `App::new()`. Once enabled, `hjkl`, `gg`, `G`, `i`, and `:` route through `VimModeState`.

## Configuration

Keybinding overrides load from `~/.bardo/keybindings.toml`. The loader uses the user's home directory from `HOME`, `USERPROFILE`, or `HOMEDRIVE` + `HOMEPATH`, then falls back to the built-in defaults if the file is missing or unreadable.

```toml
# ~/.bardo/keybindings.toml
[global]
"ctrl+c" = "Quit"
"tab" = "NextScreen"
"backtab" = "PrevScreen"
"?" = "ShowHelp"
"/" = "OpenCommandPalette"
"f1" = "GotoWindow:Hearth"

[screen.HearthOverview]
"r" = "ScrollTop"
```

Supported key syntax:

- Modifiers: `ctrl+`, `shift+`, `alt+`
- Special keys: `tab`, `backtab`, `esc`, `enter`, `backspace`, `up`, `down`, `left`, `right`
- Function keys: `f1` through `f12`
- Single-character keys

Unknown action strings are skipped with a warning instead of aborting the entire file load. That keeps partial overrides usable.

## Module Overview

- `navigation::keybindings` owns the default binding table, TOML merge logic, and string parsing for keys and actions
- `navigation::palette` owns the floating command palette state, fuzzy filtering, and overlay rendering
- `navigation::modal` owns the modal stack, input routing, and dialog rendering helpers
- `navigation::vim` owns the vim state machine and `gg` or `:q` style command handling
- `app` integrates the navigation layer into the event loop and applies the resulting `AppAction` values

## API

### Typed Actions And Navigation Enums

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WindowId {
    Hearth,
    Mind,
    Soma,
    World,
    Fate,
    Command,
    Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimDirection {
    Up,
    Down,
    Left,
    Right,
}
```

This action surface is the contract between keyboard input and the rest of the terminal runtime.

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
    pub fn load_from_toml(path: &std::path::Path) -> anyhow::Result<Self>;
    pub fn resolve(
        &self,
        key: crossterm::event::KeyEvent,
        screen: ScreenId,
    ) -> Option<AppAction>;
}

pub(crate) fn parse_key_str(input: &str) -> Option<crossterm::event::KeyEvent>;
```

`resolve()` checks per-screen overrides before global bindings. The implementation builds and compares `KeyEvent` values with `KeyEvent::new(...)`, and `App::handle_key()` drops non-`Press` events before resolution.

### Command Palette

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub action: AppAction,
    pub keybinding: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::layout::Rect,
    );
}

pub fn default_commands() -> Vec<Command>;
```

Filtering is based on the longest common subsequence between the lowercased query and command name. Empty queries show the full command list in declaration order.

### Modal Stack

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

#[derive(Default)]
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
    pub fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::layout::Rect,
    );
}
```

`Confirm` accepts `Enter` or `y` and cancels on `Esc` or `n`. `Input` pops the modal before running `on_submit`, which avoids borrowing the stack while executing the callback. `Alert` closes on `Enter`, `Esc`, or space.

### Vim Mode

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

`Normal` handles `hjkl`, `gg`, `G`, `i`, `:`, and `Esc`. `Insert` only consumes `Esc`. `Command` buffers `:...` input and returns `AppAction::VimCommand` on `Enter`.

## Usage Examples

The Rust snippets below use `crate::` because they mirror how the **bardo-terminal binary** wires modules together. They are not copy-pastable into another crate unless you add a `lib.rs` or change paths; they document behavior, not a public dependency API.

Resolve a configured key against the current screen:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// `crate::` = root of the bardo-terminal binary crate
use crate::{
    navigation::KeybindingMap,
    screen::ScreenId,
    state::{AppAction, WindowId},
};

let bindings = KeybindingMap::default_bindings();
let action = bindings.resolve(
    KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
    ScreenId::HearthOverview,
);

assert_eq!(action, Some(AppAction::GotoWindow(WindowId::Hearth)));
```

Filter the palette and execute the selected command:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::navigation::palette::{CommandPalette, default_commands}; // binary crate root

let mut palette = CommandPalette {
    visible: true,
    query: "qui".into(),
    commands: default_commands(),
    filtered: Vec::new(),
    selected: 0,
};

palette.update_filter();
let action = palette.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
assert!(action.is_some());
```

Submit an input modal:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    navigation::{Modal, ModalManager},
    state::AppAction,
};

let mut modals = ModalManager::new();
modals.push(Modal::Input {
    title: "Name".into(),
    placeholder: "enter value".into(),
    buffer: "spectre".into(),
    on_submit: Box::new(AppAction::VimCommand),
});

let action = modals.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
assert_eq!(action, Some(AppAction::VimCommand("spectre".into())));
```

Process a vim command:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    navigation::VimModeState,
    state::AppAction,
};

let mut vim = VimModeState::new(true);
vim.process_key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE));
vim.process_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));

let action = vim.process_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
assert_eq!(action, Some(AppAction::VimCommand("q".into())));
```

## Architecture

The implemented input pipeline in `App::handle_key()` is:

1. Ignore non-`Press` key events.
2. Normalize the incoming `KeyEvent` so `kind` and `state` do not affect equality.
3. Let vim mode intercept keys when enabled and in an intercepting state.
4. Route keys to the modal stack if a modal is open.
5. Route keys to the command palette if it is visible.
6. Resolve per-screen and global keybindings.
7. Forward any remaining key to the active screen.

The render stack is similarly ordered:

1. Base screen content
2. Status bar
3. Top-most modal
4. Command palette
5. Help overlay

That matches the keyboard-first terminal interaction model in `prd2/18-interfaces/01-cli.md` sections `TUI (Interactive Mode)` and `The 15-screen system`, `prd2/18-interfaces/03-tui.md` sections `5.1 Persistent chrome` and `5.2 Screen map`, and `prd2/20-styx/05-tui-experience.md` section `4. The Screen System (11 Views)`. The live tab registry is **`ScreenId::all()`** in the binary (currently **30** screens).

## References

- `prd2/18-interfaces/21-screen-catalog.md` — six-window grouping in the PRD; the binary exposes **PROTOCOL** as a seventh `WindowId` for jumps (`7` / `F7`) and `GotoWindow:protocol` in TOML. Use `ScreenId::all()` for the authoritative tab list.
- [bardo-terminal protocol views](bardo-terminal-protocol-views.md) — focused-cell navigation on the Protocol Views screen
- `prd2/18-interfaces/01-cli.md` sections `TUI (Interactive Mode)` and `The 15-screen system`
- `prd2/18-interfaces/03-tui.md` sections `3. Crate architecture`, `5.1 Persistent chrome`, `5.2 Screen map`, and `5.3 Screen details`
- `prd2/20-styx/05-tui-experience.md` section `4. The Screen System (11 Views)`
