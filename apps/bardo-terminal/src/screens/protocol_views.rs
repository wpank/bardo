//! Protocol overview: Uniswap-style pool, lending, vault, and bridge placeholders.

use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    layout::LayoutBreakpoint,
    palette::{BORDER, BORDER_ACTIVE, ROSE_DIM},
    screen::{Screen, ScreenId},
    state::{AppAction, AppState},
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
}

impl ProtocolViewsScreen {
    pub(crate) fn new() -> Self {
        Self {
            focused_cell: 0,
            compact_layout: AtomicBool::new(false),
        }
    }

    fn render_placeholder(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        cell_index: usize,
        subtitle: &str,
    ) {
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
        frame.render_widget(
            Paragraph::new(subtitle)
                .style(Style::default().fg(ROSE_DIM).add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center),
            inner,
        );
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
                self.render_placeholder(frame, r, idx, "[ loading mock ]");
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
            self.render_placeholder(frame, r, idx, "[ loading mock ]");
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
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let state = AppState::default();

        terminal
            .draw(|frame| screen.render(frame, frame.size(), &state))
            .expect("protocol views screen should render");
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
}
