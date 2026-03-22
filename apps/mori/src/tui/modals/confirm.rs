use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::ConfirmAction;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, action: &ConfirmAction) {
    if let ConfirmAction::MergeBatchToMain {
        batch_branch,
        plan_count,
        failed_count,
        last_commit,
    } = action
    {
        render_merge_to_main(
            f,
            area,
            batch_branch,
            *plan_count,
            *failed_count,
            last_commit,
        );
        return;
    }

    let popup = centered_rect(50, 25, area);
    f.render_widget(Clear, popup);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", action.title()),
            Style::default()
                .fg(Theme::STATUS_WARN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", action.description()),
            Style::default().fg(Theme::FG),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y] confirm  ", Style::default().fg(Theme::STATUS_ERROR)),
            Span::styled("  [n] cancel ", Style::default().fg(Theme::STATUS_OK)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirm Action")
                .border_style(Style::default().fg(Theme::EMBER))
                .style(Theme::default_style()),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, popup);
}

fn render_merge_to_main(
    f: &mut Frame,
    area: Rect,
    batch_branch: &str,
    plan_count: usize,
    failed_count: usize,
    last_commit: &str,
) {
    let popup = centered_rect(60, 40, area);
    f.render_widget(Clear, popup);

    // Build the "branch flow" graphic:  batch ──⬆──▶ main
    let flow_line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            batch_branch,
            Style::default()
                .fg(Theme::DREAM)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ──⬆──▶  ", Style::default().fg(Theme::ROSE_DIM)),
        Span::styled(
            "main",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    // Plan count badge
    let plan_badge = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("✓ ", Style::default().fg(Theme::SAGE)),
        Span::styled(
            format!(
                "{plan_count} plan{} ready",
                if plan_count == 1 { "" } else { "s" }
            ),
            Style::default().fg(Theme::FG_BRIGHT),
        ),
    ]);

    // Failed warning (only shown when relevant)
    let failed_line = if failed_count > 0 {
        Some(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("⚠ ", Style::default().fg(Theme::STATUS_WARN)),
            Span::styled(
                format!(
                    "{failed_count} plan{} failed — excluded",
                    if failed_count == 1 { "" } else { "s" }
                ),
                Style::default().fg(Theme::STATUS_WARN),
            ),
        ]))
    } else {
        None
    };

    // Commit hash
    let commit_line = Line::from(vec![
        Span::styled("  HEAD  ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(
            last_commit,
            Style::default()
                .fg(Theme::TEXT_GHOST)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    // Separator rule
    let rule = Line::from(Span::styled(
        "  ─────────────────────────────────────",
        Style::default().fg(Theme::ROSE_EMBER),
    ));

    // Warning
    let warning = Line::from(vec![
        Span::styled("  ⚠  ", Style::default().fg(Theme::EMBER)),
        Span::styled(
            "Irreversible without force-push to main",
            Style::default().fg(Theme::EMBER),
        ),
    ]);

    let confirm_line = Line::from(vec![
        Span::styled(
            "  [y] ",
            Style::default()
                .fg(Theme::STATUS_ERROR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("merge now  ", Style::default().fg(Theme::FG_DIM)),
        Span::styled(
            "[n] ",
            Style::default()
                .fg(Theme::STATUS_OK)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("cancel", Style::default().fg(Theme::FG_DIM)),
    ]);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ⬆  Merge Batch → main",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        flow_line,
        Line::from(""),
        plan_badge,
    ];
    if let Some(fl) = failed_line {
        lines.push(fl);
    }
    lines.push(Line::from(""));
    lines.push(commit_line);
    lines.push(Line::from(""));
    lines.push(rule);
    lines.push(Line::from(""));
    lines.push(warning);
    lines.push(Line::from(""));
    lines.push(confirm_line);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    " Merge to Main ",
                    Style::default()
                        .fg(Theme::BONE)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(Theme::ROSE_BRIGHT))
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
