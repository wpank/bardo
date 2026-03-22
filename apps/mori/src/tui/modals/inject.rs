use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, input: &str, target: Option<&str>) {
    let popup = centered_rect(60, 20, area);
    f.render_widget(Clear, popup);

    let lines = vec![Line::from(Span::styled(
        format!("> {input}\u{2588}"),
        Style::default().fg(Theme::FG),
    ))];

    let title = if let Some(t) = target {
        format!("Steer {t} via conductor (Enter to send, Esc to cancel)")
    } else {
        "Inject Message (Enter to send, Esc to cancel)".to_string()
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Theme::STATUS_WARN))
                .style(Theme::default_style()),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, popup);
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
