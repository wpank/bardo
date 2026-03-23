# bardo-terminal

A terminal UI for observing live golem state. Built with Ratatui, written in Rust.

## Running

```bash
cargo run -p bardo-terminal

# headless mode (JSON-RPC server for test automation)
cargo run -p bardo-terminal -- --headless
```

## Architecture

The app runs a single-threaded event loop driven by `app.rs`. `App` owns an `AppState` and dispatches `AppAction` variants on every tick. Navigation state — which screen is active, which window has focus, what vim mode is engaged — lives in `state.rs`.

### Screens

`screen.rs` defines `ScreenId`, an enum covering ~30 screens. Each screen is a module under `screens/`. The active screen is rendered by calling its `render()` function with the current `AppState`.

### Navigation

`navigation/` contains four modules:

- `keybindings.rs` — global and per-screen key maps
- `vim.rs` — `VimMode` / `VimModeState`, hjkl movement, `VimDirection`
- `modal.rs` — modal dialog lifecycle
- `palette.rs` — `CommandPalette`, triggered with `:` prefix

Most navigation is vim-style. `h/j/k/l` move between items and panes. `:` opens the command palette for jumping to any screen by name.

### Widgets

Reusable widgets live in `widgets/`:

| Widget | Purpose |
|--------|---------|
| `feed.rs` | Scrolling event feed |
| `gauge.rs` | Progress/capacity bar |
| `heatmap.rs` | Grid heatmap with color scaling |
| `key_help.rs` | Context-sensitive key hint bar |
| `scrolllist.rs` | Keyboard-navigable list |
| `sparkline.rs` | Inline time-series bar chart |
| `status_bar.rs` | Bottom status line |
| `tabs.rs` | Tab bar with active indicator |
| `timeline.rs` | Horizontal timeline strip |
| `cv_map_pane.rs` | CV map visualization pane |
| `listener_mode.rs` | Passive listener mode overlay |
| `visualizer.rs` | General-purpose data visualizer |

### Design System

`design/` implements the ROSEDUST color system. Color tokens are defined as palette constants in `palette.rs` and referenced throughout the widget layer. Two notable rendering techniques:

- **Braille patterns** — high-resolution character-cell rendering using Unicode braille block (U+2800–U+28FF). Used in sparklines and the visualizer.
- **Demoscene effects** — `particles.rs` provides a `ParticleSystem` with configurable `EmitterPreset` variants. `animation.rs` ties these into the render loop for screen transitions and idle effects.

### Audio

`sound.rs` and `sonification.rs` provide optional audio feedback via `rodio`. The feature is compiled in by default but degrades silently if no audio device is available. Sonification maps golem state changes to short tones — latency spikes, task completions, error events.

### RPC Server

`rpc_server.rs` starts a JSON-RPC server when `--headless` is passed. Used by the test suite to drive the UI programmatically without a real terminal. The mock data layer under `mock/` feeds static fixtures into `AppState` for development and testing without a live golem connection.

### System Stats

`sys_stats.rs` polls local CPU, memory, and process metrics on a background interval and writes them into `AppState`. Displayed on the system stats screen.

## Project Structure

```
src/
  main.rs           # entry point, arg parsing, runtime setup
  app.rs            # App struct, event loop, AppAction dispatch
  state.rs          # AppState, WindowId, VimMode, VimDirection
  screen.rs         # ScreenId enum
  layout.rs         # layout helpers
  palette.rs        # top-level palette re-exports
  animation.rs      # animation frame management
  particles.rs      # ParticleSystem, EmitterPreset
  sound.rs          # audio device init
  sonification.rs   # state-to-sound mapping
  rpc_server.rs     # headless JSON-RPC server
  sys_stats.rs      # local system metric polling
  design/           # ROSEDUST design tokens, braille rendering
  navigation/       # keybindings, vim, modal, palette
  screens/          # one module per ScreenId
  widgets/          # reusable Ratatui widgets
  mock/             # static fixtures for dev/test
```
