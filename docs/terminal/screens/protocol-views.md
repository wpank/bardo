# Protocol Views Screen

## What It Is

The Protocol Views screen displays a 2×2 grid of DeFi protocol widgets, providing an at-a-glance overview of Uniswap pools, lending markets, ERC-4626 vaults, and bridge routes. Each widget shows key metrics and visual indicators for its protocol type, with navigation between cells using arrow keys.

## Features

- **2×2 grid layout** showing four protocol widgets simultaneously
- **Focused cell navigation** with arrow keys or vim bindings (h/j/k/l)
- **Responsive layout** that collapses to a 1×4 vertical stack in narrow terminals
- **Visual focus indicators** with active border highlighting
- **Mock data display** for development and testing

## Getting Started

Navigate to the Protocol Views screen using `Tab` or `Shift+Tab` to cycle through screens. Once on the screen:

- Use arrow keys or `h`/`j`/`k`/`l` to move focus between the four cells
- The focused cell displays with an active border color
- `Tab` advances to the next screen
- `q` quits the application

## Mock Data Strategy

The Protocol Views screen currently uses placeholder data types for all protocol information. These mock types are temporary and will be replaced with live chain data in future updates.

Each mock type follows a consistent pattern:

- **Placeholder types** with realistic field shapes matching the expected real data structure
- **TODO annotations** marking which fields will connect to live subsystems
- **Default constructors** via `mock_default()` methods that return plausible sample values

The mock types include:

- `MockPoolState` — Uniswap V3/V4 pool data (price, ticks, liquidity, fees)
- `MockLendingMarket` — Aave/Morpho/Compound market data (utilization, APYs, liquidity)
- `MockVaultState` — ERC-4626 vault data (NAV, TVL, share price, 24h change)
- `MockBridgeRoute` — Bridge route data (chains, token, amount, fee, ETA, status)

All mock types use the `mock_default()` naming convention to make it explicit that the data is placeholder. This prevents accidental use of uninitialized data and signals to developers that these values are temporary.

## UniswapPoolWidget

The Uniswap Pool widget displays information about a single Uniswap V3 or V4 liquidity pool.

### Layout

The widget renders in a bordered block with the following sections:

1. **Header** — Pool pair title with fee tier and chain name (e.g., " ETH/USDC · 0.05% · Base ")
2. **Current price** — Large, right-aligned price display in the quote token (e.g., "3,421.50 USDC")
3. **Tick range bar** — Visual representation of the current tick position within the liquidity range
4. **Liquidity depth sparkline** — Two-row braille sparkline showing relative liquidity density across tick positions
5. **Footer** — TVL and 24h volume in compact USD format

### Tick Range Bar

The tick range bar is a one-row visual indicator showing where the current price sits within the pool's active liquidity range:

- **Range segment** — Filled with `█` characters in the active range, colored green when in-range
- **Out-of-range areas** — Shown with `░` characters in a dimmed color
- **Current position cursor** — Marked with `◆` at the exact current tick position
- **Tick labels** — Lower and upper tick values displayed at each end of the bar

The bar color changes based on whether the current tick is within the active range:
- **In-range**: Green border and range segment
- **Out-of-range**: Amber border and dimmed range segment

### Depth Sparkline

The liquidity depth sparkline uses braille characters to show relative liquidity density at evenly-spaced tick positions. The sparkline is rendered in two rows and uses a rose color scheme. The data comes from normalized depth samples in the range [0.0, 1.0], where 1.0 represents the maximum liquidity density in the display range.

### Fee Tier Formatting

Fee tiers are displayed as percentages with two decimal places:
- 1 basis point → "0.01%"
- 5 basis points → "0.05%"
- 30 basis points → "0.30%"
- 100 basis points → "1.00%"

### USD Formatting

TVL and volume values use compact notation:
- Values ≥ $1B → "$X.XB"
- Values ≥ $1M → "$X.XM"
- Values ≥ $1K → "$X.XK"
- Values < $1K → "$X.XX"

## LendingMarketWidget

The Lending Market widget displays information about a single lending market from protocols like Aave V3, Morpho, or Compound.

### Layout

The widget renders in a bordered block with:

1. **Header** — Protocol name, asset symbol, and chain (e.g., " Aave V3 · USDC · Base ")
2. **Utilization label and percentage** — Current utilization rate as a percentage
3. **Utilization bar** — Visual gauge showing utilization with color-coded thresholds
4. **APY display** — Supply and borrow APY shown side-by-side
5. **Liquidity totals** — Total supplied and borrowed amounts in compact USD format

### Utilization Color Thresholds

The utilization bar changes color based on the utilization rate:

- **< 80%**: Green (healthy utilization)
- **80%–95%**: Amber (approaching capacity)
- **≥ 95%**: Red (near full utilization)

These thresholds help quickly identify markets that may be approaching full capacity or experiencing high demand.

### APY Formatting

APY values are displayed as percentages with two decimal places. Supply APY is always shown in green, while borrow APY uses a dimmed rose color unless it exceeds 10%, at which point it switches to amber to indicate high borrowing costs.

The APY display format: `Supply  4.20%  |  Borrow  6.10%`

## VaultWidget

The Vault widget displays information about an ERC-4626 vault from protocols like Beefy, Yearn, or other vault providers.

### Layout

The widget renders as a labeled data table within a bordered block:

1. **Header** — Vault name and chain (e.g., " Beefy USDC/ETH · Base ")
2. **NAV per share** — Net asset value per share with asset symbol (e.g., "1.0842 USDC")
3. **TVL** — Total value locked in compact USD format
4. **APY** — Annual percentage yield
5. **24h change** — 24-hour change in share price with sign and color coding

### NAV Formatting

NAV per share is displayed with precision based on magnitude:
- **≥ 1000**: 2 decimal places
- **< 1000 and ≥ 0.01**: 4 decimal places
- **< 0.01**: 6 decimal places

The asset symbol follows the value (e.g., "1.0842 USDC").

### 24h Change Color Logic

The 24-hour change display uses color and sign to indicate performance:

- **Positive change**: Green color with "+" prefix (e.g., "+0.31%")
- **Negative change**: Red color with "-" prefix (e.g., "-2.50%")
- **Zero change**: Dimmed color with no sign prefix

This provides immediate visual feedback about vault performance over the past 24 hours.

## BridgeStatusWidget

The Bridge Status widget displays information about a bridge route or active bridge transfer.

### Layout

The widget renders in a bordered block with:

1. **Header** — Bridge name and route (e.g., " Across · ETH→Base ")
2. **Amount** — Large display of the bridged amount with token symbol
3. **Fee and ETA** — Bridge fee in USD and estimated completion time
4. **Status badge** — Current transfer status with color-coded indicator
5. **Progress bar** — For in-flight transfers, shows completion progress
6. **Route arrow** — Visual representation of the source and destination chains

### Status Badge Colors

The status badge uses different colors and symbols for each transfer state:

| Status | Color | Symbol | Display |
|--------|-------|--------|---------|
| Quoted | Dimmed | ◌ | "◌ QUOTED" |
| Pending | Amber | ◌ | "◌ PENDING" |
| In Flight | Rose | ◈ | "◈ IN FLIGHT" |
| Complete | Green | ● | "● COMPLETE" |
| Failed | Red | ✗ | "✗ FAILED" |

### Progress Calculation

For in-flight transfers, the progress bar shows completion percentage based on elapsed time:

```
progress = (elapsed_seconds / estimated_time_seconds) × 100
```

The progress is clamped between 0% and 100%. The bar color transitions from rose to green as it approaches completion, providing visual feedback on transfer progress.

The progress bar is only displayed for `InFlight` status. For `Quoted` status, the progress area shows the status badge text instead.

### Route Arrow

The route arrow displays the source and destination chain names connected by an arrow:

```
Ethereum ──────→ Base
```

Chain names are shown in the primary text color, while the arrow characters use a dimmed rose color for visual separation.

## ProtocolViewsScreen

The Protocol Views screen arranges the four protocol widgets in a 2×2 grid layout with focused cell navigation.

### Grid Layout

The screen divides the available area into four equal cells:

- **Top-left**: Uniswap Pool widget
- **Top-right**: Lending Market widget
- **Bottom-left**: Vault widget
- **Bottom-right**: Bridge Status widget

Each cell receives 50% of the screen width and 50% of the screen height in the standard layout.

### Responsive Behavior

In narrow terminals (width < 60 columns) or when the layout breakpoint is `Compact`, the screen automatically switches to a 1×4 vertical stack:

- Each widget receives 25% of the screen height
- Widgets are stacked vertically in reading order
- Navigation behavior adjusts accordingly (Down moves to the next widget, Up moves to the previous)

### Navigation

The screen supports keyboard navigation between cells:

- **Arrow keys** or **vim bindings** (`h`/`j`/`k`/`l`) move focus between cells
- **Tab** advances to the next screen in the catalog
- **Shift+Tab** moves to the previous screen
- **q** quits the application

In the 2×2 grid layout:
- **Right/Left** (`l`/`h`): Moves horizontally, wrapping at edges
- **Down/Up** (`j`/`k`): Moves vertically between rows (cell 0↔2, cell 1↔3)

In the 1×4 stack layout:
- **Down/Right** (`j`/`l`): Moves to the next widget
- **Up/Left** (`k`/`h`): Moves to the previous widget

### Focus Indication

The focused cell is indicated by an active border color that distinguishes it from unfocused cells. The focused cell is rendered last so its border draws on top when cells share edges.

## Configuration

The Protocol Views screen responds to terminal width and layout breakpoints:

- **Terminal width < 60 columns**: Automatically uses compact 1×4 stack layout
- **Layout breakpoint = Compact**: Uses 1×4 stack layout regardless of width
- **Standard/Wide/Ultra breakpoints**: Uses 2×2 grid layout

No additional configuration files or environment variables are required for the Protocol Views screen.

## Architecture

The Protocol Views screen implements the `Screen` trait and integrates with the terminal's screen registry:

```
ProtocolViewsScreen
├── focused_cell tracking
├── mock data storage (pool, market, vault, bridge)
├── layout computation (2×2 grid or 1×4 stack)
├── widget rendering (UniswapPoolWidget, LendingMarketWidget, VaultWidget, BridgeStatusWidget)
└── keyboard event handling
```

The screen is registered in the `ScreenId` enum as `ProtocolViews` and appears in the screen catalog accessible via `Tab` navigation.
