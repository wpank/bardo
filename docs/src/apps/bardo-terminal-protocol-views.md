# bardo-terminal protocol views

## What It Is

The protocol views surface is its own screen in the terminal catalog (**PROTOCOL / Views**, title **PROTOCOLS**). It shows four DeFi-oriented panels at once—Uniswap-style pool, lending market, ERC-4626 vault, and bridge status—as a responsive grid. Each cell renders structured **mock** metrics (pool price and depth sparkline, utilization and APYs, vault NAV/TVL/24h change, bridge fee and status); live chain data replaces these snapshots in later work.

## Features

- Four labeled cells: Uniswap pool, lending market, ERC-4626 vault, bridge status
- Pool cell: pair header, price, tick bar, depth sparkline, TVL / volume footer (mock)
- Lending cell: utilization bar, supply and borrow APYs, supplied/borrowed totals (mock)
- Vault cell: NAV per share, TVL, APY, and 24h share-price change (mock)
- Bridge cell: amount, fee, ETA, status badge, in-flight progress bar when applicable, and route line (mock)
- Focused-cell highlighting so you can see which panel is active
- **2×2 grid** on standard terminals; **1×4 vertical stack** when the terminal is narrow or the layout breakpoint is compact
- Arrow keys and **vim-style** `h` / `j` / `k` / `l` move focus between cells (behavior matches grid vs stack)
- Global shortcuts still apply: `Tab` / `Shift+Tab` change screens, `q` quits (when not overridden by the navigation layer)

## Getting Started

```bash
cargo run -p bardo-terminal
```

Open the Protocol Views screen:

- `7` or `F7` jumps to the **PROTOCOL** window (this screen), same as the **PROTOCOL** entry in the `/` command palette
- `Tab` forward from the home screen through the catalog until the status bar shows **30 / 30** (last screen), or jump with the command palette / `GotoScreen` if you have a binding for `ProtocolViews`

Inside the screen:

- Move focus with arrows or `hjkl`
- `Tab` leaves this screen for the next one in the catalog

Run tests for the terminal crate:

```bash
cargo test -p bardo-terminal
```

## Configuration

Layout follows the same rules as the rest of the terminal:

- **Terminal width under 60 columns** forces the stacked layout for these panels
- **Compact layout breakpoint** (for example after a small resize) also selects the stack until the terminal layout widens again

There are no extra config files or environment variables specific to this screen.

## API

There is no separate library crate for this surface; it ships inside the `bardo-terminal` binary. Keyboard input is merged with the global navigation layer first, then any keys that remain are handled on this screen for moving focus between the four panels.

## Architecture

The terminal registers **Protocol Views** as a dedicated `ScreenId` with a real `Screen` implementation (not a stub). The screen tracks which of the four panels is focused, remembers whether the last frame used the compact stack, and draws the focused panel last so its active border wins where cells meet.

All four panels use the same mock-backed widget set described in the terminal design notes; swapping in live feeds is a later integration step and does not change how you navigate or resize this screen.
