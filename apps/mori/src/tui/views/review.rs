use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::{RunPlanStatus, RunState};
use crate::tui::theme::Theme;
use crate::tui::widgets;

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    // Split horizontally: left 30% (summary + actions) | right 70% (diff panel)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(5)])
        .split(columns[0]);

    render_plan_summary(f, sections[0], state);
    render_actions(f, sections[1], state);

    // Right column: diff panel
    widgets::diff_panel::render(f, columns[1], state);
}

fn render_plan_summary(f: &mut Frame, area: Rect, state: &RunState) {
    let items: Vec<ListItem> = state
        .plans
        .iter()
        .filter(|p| p.status != RunPlanStatus::Pending)
        .map(|plan| {
            let (icon, style) = match plan.status {
                RunPlanStatus::Completed => ("✓", Style::default().fg(Theme::STATUS_OK)),
                RunPlanStatus::CompletedPrior => ("✓", Style::default().fg(Theme::SAGE)),
                RunPlanStatus::MergedToMain => ("✓", Style::default().fg(Theme::STATUS_OK)),
                RunPlanStatus::Failed => ("✗", Style::default().fg(Theme::STATUS_ERROR)),
                RunPlanStatus::Skipped => ("⊘", Style::default().fg(Theme::STATUS_DIM)),
                RunPlanStatus::Active => ("►", Style::default().fg(Theme::STATUS_ACTIVE)),
                RunPlanStatus::Pending => (" ", Style::default().fg(Theme::FG_DIM)),
            };

            let iter_info = if plan.iteration > 0 {
                format!(" (iter {})", plan.iteration)
            } else {
                String::new()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {icon} "), style),
                Span::styled(plan.base.clone(), style),
                Span::styled(iter_info, Style::default().fg(Theme::FG_DIM)),
            ]))
        })
        .collect();

    let completed = state
        .plans
        .iter()
        .filter(|p| p.status == RunPlanStatus::Completed)
        .count();
    let total = state.plans.len();
    let title = format!("Plan Summary ({completed}/{total})");

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Theme::block_style())
            .title_style(Theme::title_style()),
    );
    f.render_widget(list, area);
}

fn render_actions(f: &mut Frame, area: Rect, state: &RunState) {
    let is_paused = state.orchestrator_state == "batch-paused";

    let lines = if is_paused {
        vec![
            Line::from(vec![Span::styled(
                " Batch paused. ",
                Style::default()
                    .fg(Theme::STATUS_WARN)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " [c]ontinue  [s]kip  [a]bort ",
                Style::default().fg(Theme::FG),
            )]),
        ]
    } else {
        vec![Line::from(Span::styled(
            " Pipeline running. Review available on batch pause.",
            Style::default().fg(Theme::FG_DIM),
        ))]
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Actions")
                .style(Theme::block_style())
                .title_style(Theme::title_style()),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}
