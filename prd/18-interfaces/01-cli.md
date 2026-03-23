# `bardo` Binary — CLI and TUI [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14
**Crate:** `golem-binary` (internal crate name); installed binary is `bardo`

---

> **Reader orientation:** This document specifies the `bardo` binary, a single Rust executable that serves as both a CLI tool and the entry point to the 60fps TUI (terminal user interface). It belongs to the **interfaces layer** of the Bardo stack. The TUI is the primary surface where owners interact with their Golems (mortal autonomous DeFi agents). Key concepts: Golem (an autonomous agent with finite lifespan running on a micro VM), Grimoire (the Golem's persistent knowledge store), Styx (the social/connectivity layer indexing on-chain agent data), and Thanatopsis (the Golem's graceful shutdown and death ceremony). For unfamiliar terms, see `prd2/shared/glossary.md`.

## Overview

`bardo` is a single Rust binary. Running it without arguments opens the 60fps ratatui TUI where owners live alongside their Golems, watch them think and dream, interact with the creature, and navigate the 29-screen, 6-window system. It also supports non-interactive subcommands for scripting and CI/CD.

| Mode | Invocation | Purpose |
|------|-----------|---------|
| **TUI** | `bardo` | Interactive: 60fps Golem management, creature, live state |
| **Subcommands** | `bardo golem ...`, `bardo setup`, etc. | Non-interactive: scripting, CI/CD, lifecycle management |

---

## Subcommands (Non-Interactive Mode)

### Golem lifecycle

The `bardo golem` subcommand tree manages the full Golem lifecycle from the CLI.

```text
bardo golem create                          # Interactive creation wizard
bardo golem create --config ./golem.toml    # Create from TOML config file
bardo golem create --template eth-dca       # Create from strategy template
bardo golem create --inherit <predecessor>  # Create successor from dead Golem
bardo golem create --prompt "DCA into ETH with $500/month"  # Natural language

bardo golem list                            # List active Golems
bardo golem status <id>                     # Status of a specific Golem (a mortal autonomous DeFi agent)
bardo golem status <id> --watch             # Live-updating status (10s refresh)
bardo golem pause <id>                      # Pause a running Golem
bardo golem resume <id>                     # Resume a paused Golem
bardo golem kill <id>                       # Trigger graceful shutdown (Thanatopsis: the Golem's death ceremony)
bardo golem logs <id>                       # Stream Golem event log
bardo golem config <id>                     # View/edit running Golem config
bardo golem successor <id>                  # Create successor from a dead Golem

bardo golem deploy                          # Full deployment pipeline (parse -> provision -> connect)
bardo golem deploy --network base           # Target specific network
bardo golem deploy --custody delegation     # Custody mode
bardo golem deploy --dry-run                # Validate and estimate costs
bardo golem deploy --preview                # Simulate first heartbeat against live data

bardo golem --explore                       # Read-only browsing (no Golem required)
```

### Setup and utilities

```text
bardo setup                     # Install wizard
bardo setup --zero              # Zero-dep quick start
bardo setup --ui                # Browser wizard
bardo setup --self-hosted       # Direct Privy API setup
bardo setup --non-interactive   # CI/CD headless mode

bardo dev                       # Start Anvil + deploy Uniswap stack
bardo dev --fresh               # Wipe + redeploy
bardo dev --ui                  # With debug UI

bardo portal                    # Launch agent dashboard
bardo portal --port 4000        # Custom port

bardo testnet <cmd>             # Testnet toolkit
bardo testnet start             # Start Anvil instance
bardo testnet time <seconds>    # Advance block time
bardo testnet mine <blocks>     # Mine blocks

bardo config show               # Display current configuration
bardo config set <key> <value>  # Set a config value
bardo config reset              # Reset to defaults

bardo doctor                    # System diagnostics
bardo doctor --fix              # Auto-fix common issues
```

### Creation flags

| Flag | Description |
|------|-------------|
| `--config <path>` | Path to `golem.toml` config file |
| `--template <id>` | Strategy template ID (`eth-dca`, `stablecoin-yield`, `lp-provision`) |
| `--prompt <text>` | Natural-language strategy description for AI autofill |
| `--inherit <id>` | Predecessor Golem ID for successor creation |
| `--custody <mode>` | Custody mode: `delegation`, `embedded`, `local-key` |
| `--network <net>` | Target network: `base`, `base-sepolia`, `anvil` |
| `--dry-run` | Validate and estimate costs without executing |
| `--preview` | Simulate first heartbeat tick against live data |
| `--non-interactive` | CI/CD mode: no prompts, all values from flags/config/env |

### Deployment pipeline

The `bardo golem deploy` command implements the full deployment pipeline that Bott social connectors also use:

```text
Intent (natural language or template)
    |
    v
+---------------------------+
|  1. Parse & Classify       |  Extract intent, protocol, amount,
|  (local classifier)        |  risk parameters from text/config
+---------------------------+
    |
    v
+---------------------------+
|  2. Template Match         |  Map to strategy template with
|  (Strategy Manifest)       |  default parameters
+---------------------------+
    |
    v
+---------------------------+
|  3. Authenticate           |  Privy wallet provisioning or
|  (Privy)                   |  retrieval for returning users
+---------------------------+
    |
    v
+---------------------------+
|  4. Confirm                |  Present strategy summary and
|  (TUI / --yes flag)        |  costs; await user confirmation
+---------------------------+
    |
    v
+---------------------------+
|  5. Provision              |  x402 (HTTP 402-based micropayment protocol) payment -> Bardo Compute (the VM hosting infrastructure for Golems) VM
|  (Bardo Compute)           |  -> Golem boot with strategy config
+---------------------------+
    |
    v
+---------------------------+
|  6. Connect                |  Register platform connectors
|  (Social Bridge, optional) |  for notifications
+---------------------------+
    |
    v
Running Golem (headless, --rpc mode)
```

Strategy templates:

| Template          | VM Size | Daily Cost | Required Params          |
| ----------------- | ------- | ---------- | ------------------------ |
| `yield-optimizer` | micro   | ~$0.60     | protocol, amount         |
| `dca`             | micro   | ~$0.60     | asset, amount, frequency |
| `lp-provision`    | small   | ~$1.20     | pair, amount             |
| `keeper`          | micro   | ~$0.60     | protocol, position       |

### Succession flow

`bardo golem successor <id>` works only on dead Golems:

1. Fetches the death testament and Grimoire (the Golem's persistent knowledge store) backup from Styx (Bardo's social and connectivity layer) L0
2. Presents inheritance preview (knowledge count, strategy, estimated value)
3. Prompts for funding amount and config overrides
4. Creates a new Golem with `inherit_from: <predecessor-id>`
5. Provisioning pipeline seeds the successor's Grimoire from predecessor's backup

### Status output

`bardo golem status <id>` returns: current state, credit balance and burn rate, reputation score and tier, strategy summary, current positions, uptime, heartbeat count, last 5 events. With `--json`, structured output for scripting.

### Non-interactive mode

All subcommands support `--json` and `--yes` for CI/scripting:

```bash
bardo doctor --json | jq '.checks[] | select(.status == "fail")'
bardo golem status abc123 --json
bardo setup --non-interactive --wallet-provider privy
```

---

## TUI (Interactive Mode)

### Architecture

`bardo` connects to the Golem's RPC endpoint via WebSocket. The TUI is a client — it does not host the Golem runtime.

```
+------------------------+         WebSocket          +----------------------+
|   bardo                |<---- Golem events ---------|   Golem VM           |
|   (Rust / ratatui)     |---- commands ------------->|   (Fly.io)           |
|                        |         :8402              |                      |
|   60 FPS render loop   |                            |   Event Fabric       |
|   Sprite engine        |        WebSocket           |   Heartbeat FSM      |
|   Particle system      |<---- Styx events ----------|                      |
|   Input handling        |       :8080/ws            +----------------------+
+------------------------+                            |   Styx Service       |
                                                      |   (Fly.io)           |
                                                      +----------------------+
```

### Entry points

```bash
# Install
cargo install bardo
# Or: npx @bardo/terminal (downloads prebuilt binary, caches in ~/.bardo/bin/)

# Launch
bardo                                   # Auto-detect config, connect to Golem (opens TUI)
bardo --golem g-7f3a                    # Connect to specific Golem
bardo --remote wss://g-7f3a.bardo.compute/events
bardo --explore                         # Browse-only mode (no Golem required)
bardo --clade                           # Multi-Golem overview
```

### Screen navigation

Screen navigation uses the v4 6-window hierarchy. See `screens/03-interaction-hierarchy.md` (defines the 29-screen, 6-window navigation model with windows/tabs/panes hierarchy and keyboard shortcuts).

### Crate architecture

```
bardo/ (crate: golem-binary)
+-- src/
|   +-- main.rs           # Entry point, arg parsing, event loop
|   +-- app.rs            # Application state machine
|   +-- screens/          # One module per screen (15)
|   +-- ws.rs             # WebSocket client (tokio-tungstenite)
|   +-- auth.rs           # Browser handoff, token management
|   +-- input.rs          # Keyboard/mouse handling
|   +-- navigation.rs     # Screen stack, history
|   +-- notifications.rs  # Priority toast system
|   +-- config.rs         # TOML config loading
+-- crates/
|   +-- bardo-sprites/    # Sprite engine crate (lib)
|   |   +-- src/
|   |   |   +-- generator.rs      # Procedural sprite generation from seed
|   |   |   +-- evolution.rs      # Age stages, learning mutations
|   |   |   +-- animation.rs      # Keyframe interpolation, easing
|   |   |   +-- particles.rs      # Particle emitter/updater/renderer
|   |   |   +-- expression.rs     # PAD -> expression mapping
|   |   |   +-- palette.rs        # Color palette from seed
|   |   |   +-- halfblock.rs      # Unicode half-block rasterizer
|   +-- bardo-tui-widgets/  # Custom ratatui widgets (lib)
|       +-- src/
|       |   +-- heartbeat_pipeline.rs
|       |   +-- probe_gauge.rs
|       |   +-- causal_graph.rs
|       |   +-- lineage_tree.rs
|       |   +-- pad_dial.rs
|       |   +-- command_palette.rs
|       |   +-- achievement_popup.rs
|       |   +-- bloodstain_overlay.rs
+-- assets/
|   +-- sprites/          # Base form pixel data
|   +-- animations/       # Keyframe definitions (RON format)
|   +-- particles/        # Particle effect definitions (RON)
```

### Render loop

60fps via the standard ratatui pattern:

```rust
const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    let mut last_frame = Instant::now();
    loop {
        // 1. Poll input (non-blocking)
        let timeout = FRAME_DURATION.saturating_sub(last_frame.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }
        // 2. Drain WebSocket events (non-blocking)
        while let Ok(event) = app.ws_receiver.try_recv() {
            app.process_golem_event(event);
        }
        // 3. Tick animations
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f64();
        app.sprite_engine.tick(dt);
        app.particle_system.tick(dt);
        last_frame = now;
        // 4. Render (immediate mode)
        terminal.draw(|frame| app.render(frame))?;
        // 5. Check exit
        if app.should_quit { break; }
    }
    Ok(())
}
```

The render loop runs on the main thread. WebSocket I/O runs on a `tokio` runtime in a background thread. Events flow through a `crossbeam::channel`. No async in the hot render path.

### Frame budget (16.6ms)

| Phase                     | Budget    |
| ------------------------- | --------- |
| Input polling             | ~0.1ms    |
| WebSocket event drain     | ~0.5ms    |
| Animation tick            | ~1.0ms    |
| Layout computation        | ~0.5ms    |
| Widget rendering          | ~2.0ms    |
| Sprite rasterization      | ~1.5ms    |
| Terminal flush             | ~2.0ms    |
| **Total**                 | **~7.6ms** (54% of budget -- comfortable headroom) |

### Distribution

**Prebuilt binaries** via GitHub Releases for: x86_64-linux-gnu, x86_64-linux-musl, aarch64-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin, x86_64-windows-msvc.

**npx wrapper**: `npx @bardo/terminal` is a thin npm package (<10KB) that detects platform, downloads the correct binary from Releases, caches it in `~/.bardo/bin/`, and execs it. Zero Rust toolchain required.

**Cargo install**: `cargo install bardo` -- requires Rust 1.75+. Compiles in ~30s. Binary is ~8MB.

### Terminal requirements

| Requirement      | Minimum          | Recommended              |
| ---------------- | ---------------- | ------------------------ |
| Columns          | 80               | 120+                     |
| Rows             | 24               | 40+                      |
| Color            | 256-color        | Truecolor (24-bit)       |
| Unicode          | Basic Multilingual Plane | Full                |
| Terminal         | Any VT100        | iTerm2, Kitty, WezTerm, Ghostty |

---

## Config resolution

### Resolution order (highest priority first)

1. **CLI flags** -- `bardo --profile vault --port 3000`
2. **Environment variables** -- `BARDO_PROFILE=vault`
3. **`~/.bardo/config.toml`** -- persistent user configuration
4. **`.env` file** -- project-level dotenv
5. **Defaults** -- hardcoded in `golem-binary`

### Config file schema

```toml
[profile]
name = "vault"

[chains]
ids = [1, 8453]

[wallet]
provider = "privy"
wallet_id = "wlt_abc123xyz"
address = "0x1a2B...3c4D"

[rpc]
ethereum = "https://eth-mainnet.g.alchemy.com/v2/..."
base = "https://base-mainnet.g.alchemy.com/v2/..."

[safety]
max_tx_usd = 1000
max_daily_usd = 5000
token_allowlist = "strict"
```

---

## Doctor diagnostics

| Check             | Description                                         |
| ----------------- | --------------------------------------------------- |
| Config file       | `~/.bardo/config.toml` exists and is valid          |
| Wallet connection | Can reach wallet provider                           |
| RPC connectivity  | Can reach configured chain RPCs                     |
| Foundry installed | `forge --version` (optional)                        |
| Anvil available   | `anvil --version` (optional)                        |

With `--fix`, automatically resolves common issues.
