# bardo-terminal protocol views

## What It Is

The protocol views screen family is the terminal's DeFi dashboard surface. It combines mock protocol state with four focused widgets so the app can present a Uniswap pool, a lending market, an ERC-4626 vault, and a bridge route side by side in the terminal.

The goal is progressive disclosure: each panel shows the few fields that matter first, while leaving the richer on-chain model for later data connections.

## Features

- 2x2 protocol grid in the default wide layout
- Compact fallback that stacks the four panels vertically in narrow terminals
- Focusable cells with clear border emphasis on the active panel
- Mock data records for pool, market, vault, and bridge state
- Uniswap view with pair label, fee tier, tick range, depth sparkline, TVL, and 24h volume
- Lending view with utilization, supply APY, borrow APY, and liquidity totals
- Vault view with NAV per share, TVL, APY, and 24h share-price change
- Bridge view with route, fee, ETA, status badge, and progress handling

## Getting Started

Run the terminal and cycle to the `PROTOCOLS` screen with `Tab` or `Shift+Tab`:

```bash
cargo run -p bardo-terminal
```

Once the screen is visible:

- Use the arrow keys to move the focus between the four panels
- Resize the terminal to see the layout collapse into a vertical stack
- Press `q` to exit through the standard terminal shortcut

## Configuration

The protocol views screen does not have a separate configuration file. It inherits the terminal's keyboard bindings and responsive layout settings.

The mock values shown in the widgets come from constructor defaults, so the screen renders consistently even before live chain data is connected.

## Module Overview

- `mock::protocol_data` defines the placeholder DeFi records that feed every protocol widget
- `widgets::protocol` contains the pool, lending, vault, and bridge widget implementations
- `screens::protocol_views` composes the widgets into the screen shown in the terminal

## API

### Mock Protocol Data

```rust
pub(crate) struct MockPoolState {
    pub(crate) token0_symbol: String,
    pub(crate) token1_symbol: String,
    pub(crate) current_price: f64,
    pub(crate) current_tick: i32,
    pub(crate) tick_range: MockTickRange,
    pub(crate) active_liquidity: u128,
    pub(crate) fee_tier_bps: u16,
    pub(crate) tvl_usd: f64,
    pub(crate) volume_24h_usd: f64,
    pub(crate) depth_samples: Vec<f64>,
    pub(crate) chain: String,
}
impl MockPoolState {
    pub(crate) fn mock_default() -> Self;
}

pub(crate) struct MockTickRange {
    pub(crate) lower_tick: i32,
    pub(crate) upper_tick: i32,
    pub(crate) current_tick: i32,
}
impl MockTickRange {
    pub(crate) fn position_fraction(&self) -> Option<f64>;
    pub(crate) fn is_in_range(&self) -> bool;
}

pub(crate) struct MockLendingMarket {
    pub(crate) protocol_name: String,
    pub(crate) asset_symbol: String,
    pub(crate) utilization: f64,
    pub(crate) supply_apy: f64,
    pub(crate) borrow_apy: f64,
    pub(crate) total_liquidity_usd: f64,
    pub(crate) total_borrow_usd: f64,
    pub(crate) chain: String,
}
impl MockLendingMarket {
    pub(crate) fn mock_default() -> Self;
}

pub(crate) struct MockVaultState {
    pub(crate) vault_name: String,
    pub(crate) asset_symbol: String,
    pub(crate) nav_per_share: f64,
    pub(crate) tvl_usd: f64,
    pub(crate) share_price_24h_change: f64,
    pub(crate) apy: f64,
    pub(crate) protocol_name: String,
    pub(crate) chain: String,
}
impl MockVaultState {
    pub(crate) fn mock_default() -> Self;
}

pub(crate) enum MockBridgeStatus {
    Quoted,
    Pending,
    InFlight,
    Complete,
    Failed,
}

pub(crate) struct MockBridgeRoute {
    pub(crate) from_chain: String,
    pub(crate) to_chain: String,
    pub(crate) token_symbol: String,
    pub(crate) amount: f64,
    pub(crate) fee_usd: f64,
    pub(crate) estimated_time_secs: u64,
    pub(crate) bridge_name: String,
    pub(crate) status: MockBridgeStatus,
}
impl MockBridgeRoute {
    pub(crate) fn mock_default() -> Self;
}
```

### Protocol Widgets

```rust
pub(crate) struct UniswapPoolWidgetConfig {
    pub(crate) show_depth: bool,
    pub(crate) price_range_pct: f32,
}
impl Default for UniswapPoolWidgetConfig;

pub(crate) struct UniswapPoolWidget<'a> {
    pub(crate) pool: &'a MockPoolState,
    pub(crate) config: UniswapPoolWidgetConfig,
}
impl<'a> UniswapPoolWidget<'a> {
    pub(crate) fn new(pool: &'a MockPoolState) -> Self;
    pub(crate) fn with_config(self, config: UniswapPoolWidgetConfig) -> Self;
}

pub(crate) struct LendingMarketWidget<'a> {
    pub(crate) market: &'a MockLendingMarket,
}
impl<'a> LendingMarketWidget<'a> {
    pub(crate) fn new(market: &'a MockLendingMarket) -> Self;
}

pub(crate) struct VaultWidget<'a> {
    pub(crate) vault: &'a MockVaultState,
}
impl<'a> VaultWidget<'a> {
    pub(crate) fn new(vault: &'a MockVaultState) -> Self;
}

pub(crate) struct BridgeStatusWidgetConfig {
    pub(crate) show_progress: bool,
}
impl Default for BridgeStatusWidgetConfig;

pub(crate) struct BridgeStatusWidget<'a> {
    pub(crate) route: &'a MockBridgeRoute,
    pub(crate) config: BridgeStatusWidgetConfig,
    pub(crate) now_secs: u64,
    pub(crate) submitted_at_secs: u64,
}
impl<'a> BridgeStatusWidget<'a> {
    pub(crate) fn new(route: &'a MockBridgeRoute) -> Self;
    pub(crate) fn with_timing(self, now_secs: u64, submitted_at_secs: u64) -> Self;
}

pub(crate) fn format_fee_tier(bps: u16) -> String;
pub(crate) fn format_usd_compact(value: f64) -> String;
pub(crate) fn utilization_color(util: f64) -> ratatui::style::Color;
pub(crate) fn format_pct_change(change: f64) -> (String, ratatui::style::Color);
pub(crate) fn flight_progress_pct(now_secs: u64, submitted_at_secs: u64, estimated_time_secs: u64) -> u16;
```

### Screen Surface

```rust
pub(crate) enum ScreenId {
    /* existing screens omitted */
    ProtocolViews,
}

pub(crate) struct ProtocolViewsScreen {
    /* private fields omitted */
}

impl ProtocolViewsScreen {
    pub(crate) fn new() -> Self;
}

impl crate::screen::Screen for ProtocolViewsScreen {
    fn id(&self) -> ScreenId;
    fn title(&self) -> &str;
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::layout::Rect,
        state: &crate::state::AppState,
    );
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<crate::state::AppAction>;
}
```

`ScreenId::all()` includes `ProtocolViews` at the end of the tab-cycling order, and the screen title shown in chrome is `PROTOCOLS`.

## Usage Examples

Render a single protocol widget inside a terminal frame:

```rust
use ratatui::{backend::TestBackend, Terminal};

use crate::mock::protocol_data::MockPoolState;
use crate::widgets::protocol::UniswapPoolWidget;

fn smoke_test() {
    let backend = TestBackend::new(40, 8);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let pool = MockPoolState::mock_default();

    terminal
        .draw(|frame| {
            frame.render_widget(UniswapPoolWidget::new(&pool), frame.size());
        })
        .expect("draw");
}
```

Render the full screen and move focus between panels:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};

use crate::screen::Screen;
use crate::screens::protocol_views::ProtocolViewsScreen;
use crate::state::AppState;

fn render_protocol_views() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let mut screen = ProtocolViewsScreen::new();
    let state = AppState::default();

    screen.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));

    terminal
        .draw(|frame| {
            screen.render(frame, frame.size(), &state);
        })
        .expect("draw");
}
```

## Architecture

The screen is a thin composition layer over four stateless widget views:

- Uniswap pool widget in the top-left pane
- Lending market widget in the top-right pane
- Vault widget in the bottom-left pane
- Bridge status widget in the bottom-right pane

Each widget receives mock protocol state through borrowed data or a config struct, then renders a narrow slice of the underlying DeFi model. That keeps the screen dense enough to be useful while still leaving room for later live data integration.

The top-level screen keeps the layout responsive. On wide terminals it renders a 2x2 grid; on narrow terminals it collapses into a stacked layout so the four protocol panels remain readable.

## References

- `prd2/18-interfaces/protocol/01-protocol-view-catalog.md` sections `How to read`, `Uniswap pools and positions`, `Aave and Morpho lending`, `ERC-4626 vaults`, and `Across bridge routes`
- `prd2/14-chain/07-generative-views.md` sections `Protocol family classification`, `DEX template family`, `Lending template family`, `Vault template family`, `Bridge template family`, and `DisplayField shapes`
- `prd2/18-interfaces/screens/04-oracle-surfaces.md` sections `Progressive disclosure model` and `Data density guidance`
- `prd2/07-tools/17-tools-uniswap-api.md` sections `QuoteResult` and `pool info data shapes`
