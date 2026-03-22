use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::PendingApproval;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, approval: &PendingApproval) {
    let popup = centered_rect(60, 30, area);
    f.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            format!("Agent: {}", approval.role),
            Style::default()
                .fg(Theme::BONE_DIM)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("Command:", Style::default().fg(Theme::FG_DIM))),
        Line::from(Span::styled(
            approval.command.clone(),
            Style::default().fg(Theme::FG),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y] approve  ", Style::default().fg(Theme::STATUS_OK)),
            Span::styled(" [n] reject ", Style::default().fg(Theme::STATUS_ERROR)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Approval Required")
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
