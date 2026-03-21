//! Chain Intelligence protocol overview: Uniswap pool, lending, vault, and bridge panels (mock data).

use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Block, Borders},
};

use crate::{
    layout::LayoutBreakpoint,
    mock::protocol_data::{MockBridgeRoute, MockLendingMarket, MockPoolState, MockVaultState},
    palette::{BORDER, BORDER_ACTIVE, ROSE_DIM},
    screen::{Screen, ScreenId},
    state::{AppAction, AppState},
    widgets::{BridgeStatusWidget, LendingMarketWidget, UniswapPoolWidget, VaultWidget},
};

const CELL_LABELS: [&str; 4] = [
    "Uniswap pool",
    "Lending market",
    "ERC-4626 vault",
    "Bridge status",
];

/// Screen displaying protocol widgets in a 2×2 grid or a 1×4 stack when space is tight.
pub(crate) struct ProtocolViewsScreen {
    /// Focused cell index 0..4 (pool, lending, vault, bridge).
    focused_cell: usize,
    /// Updated each [`Screen::render`] so [`Screen::handle_key`] can match navigation to layout.
    compact_layout: AtomicBool,
    // TODO(plan-70a): pool sourced from golem_chain_intelligence::uniswap
    pool: MockPoolState,
    // TODO(plan-70a): market sourced from golem_chain_intelligence::aave
    market: MockLendingMarket,
    // TODO(plan-70a): vault sourced from golem_chain_intelligence::erc4626
    vault: MockVaultState,
    // TODO(plan-70a): bridge route sourced from bardo-styx bridge monitor
    bridge: MockBridgeRoute,
}

impl ProtocolViewsScreen {
    pub(crate) fn new() -> Self {
        Self {
            focused_cell: 0,
            compact_layout: AtomicBool::new(false),
            pool: MockPoolState::mock_default(),
            market: MockLendingMarket::mock_default(),
            vault: MockVaultState::mock_default(),
            bridge: MockBridgeRoute::mock_default(),
        }
    }

    fn render_cell(&self, frame: &mut Frame<'_>, area: Rect, cell_index: usize) {
        let focused = cell_index == self.focused_cell;
        let border_color = if focused { BORDER_ACTIVE } else { BORDER };
        let title = CELL_LABELS[cell_index];
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" {title} "),
                Style::default().fg(ROSE_DIM),
            ));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        match cell_index {
            0 => frame.render_widget(UniswapPoolWidget { state: &self.pool }, inner),
            1 => frame.render_widget(
                LendingMarketWidget {
                    state: &self.market,
                },
                inner,
            ),
            2 => frame.render_widget(VaultWidget::new(&self.vault), inner),
            3 => frame.render_widget(
                BridgeStatusWidget {
                    route: &self.bridge,
                },
                inner,
            ),
            _ => {}
        }
    }
}

impl Screen for ProtocolViewsScreen {
    fn id(&self) -> ScreenId {
        ScreenId::ProtocolViews
    }

    fn title(&self) -> &str {
        "PROTOCOLS"
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, state: &AppState) {
        let compact = area.width < 60 || state.layout == LayoutBreakpoint::Compact;
        self.compact_layout.store(compact, Ordering::Relaxed);

        if compact {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(area);
            let mut order: [(usize, Rect); 4] =
                [(0, rows[0]), (1, rows[1]), (2, rows[2]), (3, rows[3])];
            order.sort_by_key(|(idx, _)| (*idx == self.focused_cell) as u8);
            for (idx, r) in order {
                self.render_cell(frame, r, idx);
            }
            return;
        }

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        // Draw focused cell last so its active border wins when edges meet.
        let order_grid = [
            (0usize, top[0]),
            (1, top[1]),
            (2, bottom[0]),
            (3, bottom[1]),
        ];
        let mut draw_order = order_grid;
        draw_order.sort_by_key(|(idx, _)| (*idx == self.focused_cell) as u8);
        for (idx, r) in draw_order {
            self.render_cell(frame, r, idx);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        let compact = self.compact_layout.load(Ordering::Relaxed);

        match key.code {
            KeyCode::Right | KeyCode::Char('l') => {
                self.focused_cell = (self.focused_cell + 1) % 4;
                None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.focused_cell = (self.focused_cell + 3) % 4;
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.focused_cell = if compact {
                    (self.focused_cell + 1) % 4
                } else {
                    (self.focused_cell + 2) % 4
                };
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.focused_cell = if compact {
                    (self.focused_cell + 3) % 4
                } else {
                    (self.focused_cell + 2) % 4
                };
                None
            }
            KeyCode::Tab => Some(AppAction::NextScreen),
            KeyCode::BackTab => Some(AppAction::PrevScreen),
            KeyCode::Char('q') => Some(AppAction::Quit),
            _ => None,
        }
    }

    fn on_focus(&mut self) {
        self.focused_cell = 0;
    }

    fn on_blur(&mut self) {}
}

#[cfg(test)]
impl ProtocolViewsScreen {
    fn focused_cell(&self) -> usize {
        self.focused_cell
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use ratatui::{Terminal, backend::TestBackend};

    use crate::state::AppAction;

    #[test]
    fn test_protocol_views_screen_id() {
        let screen = ProtocolViewsScreen::new();
        assert_eq!(screen.id(), ScreenId::ProtocolViews);
        assert_eq!(screen.title(), "PROTOCOLS");
    }

    #[test]
    fn test_protocol_views_screen_renders_without_panic() {
        let screen = ProtocolViewsScreen::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let state = AppState::default();

        terminal
            .draw(|frame| screen.render(frame, frame.size(), &state))
            .expect("protocol views screen should render");
    }

    #[test]
    fn test_protocol_views_all_cells_emit_expected_mock_labels() {
        let screen = ProtocolViewsScreen::new();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let state = AppState::default();

        terminal
            .draw(|frame| screen.render(frame, frame.size(), &state))
            .expect("protocol views screen should render");

        let output = terminal.backend().buffer().clone();
        let text: String = (0..output.area.height)
            .flat_map(|y| (0..output.area.width).map(move |x| (x, y)))
            .map(|(x, y)| output.get(x, y).symbol().to_string())
            .collect();

        assert!(text.contains("ETH"), "pool cell should show base symbol");
        assert!(text.contains("USDC"), "pool / lending should mention USDC");
        assert!(
            text.contains("Aave"),
            "lending cell should show protocol name"
        );
        assert!(
            text.contains("Beefy"),
            "vault cell should show protocol name"
        );
        assert!(
            text.contains("Across"),
            "bridge cell should show bridge name"
        );
    }

    #[test]
    fn test_protocol_views_tab_key_returns_next_screen() {
        let mut screen = ProtocolViewsScreen::new();
        let action = screen.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(action, Some(AppAction::NextScreen));
    }

    #[test]
    fn test_protocol_views_arrow_keys_cycle_focus() {
        let mut screen = ProtocolViewsScreen::new();
        assert_eq!(screen.focused_cell(), 0);

        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.focused_cell(), 1);
        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.focused_cell(), 2);
        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.focused_cell(), 3);
        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.focused_cell(), 0);
    }

    #[test]
    fn test_screen_id_all_includes_protocol_views() {
        let all = ScreenId::all();
        assert!(all.contains(&ScreenId::ProtocolViews));
    }

    #[test]
    fn test_protocol_views_compact_stack_j_k_move_sequential() {
        let mut screen = ProtocolViewsScreen::new();
        let backend = TestBackend::new(50, 40);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let mut state = AppState::default();
        state.layout = LayoutBreakpoint::Compact;

        terminal
            .draw(|frame| screen.render(frame, frame.size(), &state))
            .expect("render should succeed");

        assert_eq!(screen.focused_cell(), 0);
        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.focused_cell(), 1);
        screen.handle_key(KeyEvent::from(KeyCode::Char('j')));
        assert_eq!(screen.focused_cell(), 2);
        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.focused_cell(), 1);
        screen.handle_key(KeyEvent::from(KeyCode::Char('k')));
        assert_eq!(screen.focused_cell(), 0);
    }
}
