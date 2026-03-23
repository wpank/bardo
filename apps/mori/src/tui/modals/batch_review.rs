// Batch review modal (overlay variant of review tab)
// Re-uses review tab rendering in a modal frame

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::Frame;

use crate::state::RunState;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let popup = centered_rect(80, 70, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Batch Review")
        .border_style(ratatui::style::Style::default().fg(Theme::STATUS_WARN))
        .style(Theme::default_style());

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    super::super::views::review::render(f, inner, state);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
