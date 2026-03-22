use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::orchestrator::tasks::{Task, TaskStatus};
use crate::state::RunState;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let popup = centered_rect(60, 70, area);
    f.render_widget(Clear, popup);

    // Build flat task list: (plan_num, &Task)
    let mut items: Vec<(String, &Task)> = Vec::new();
    if let Some(ref cl) = state.task_checklist {
        for t in &cl.tasks {
            items.push((cl.plan_num.clone(), t));
        }
    }
    for (pnum, cl) in &state.plan_task_cache {
        for t in &cl.tasks {
            items.push((pnum.clone(), t));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Tasks — Enter to ingest, Esc to close ")
        .border_style(Style::default().fg(Theme::DREAM))
        .style(Theme::default_style());
    f.render_widget(block, popup);

    let inner = Rect::new(
        popup.x + 1,
        popup.y + 1,
        popup.width.saturating_sub(2),
        popup.height.saturating_sub(2),
    );

    if items.is_empty() {
        let p = Paragraph::new(Line::from(Span::styled(
            " No tasks loaded",
            Style::default().fg(Theme::TEXT_DIM),
        )));
        f.render_widget(p, inner);
        return;
    }

    // Reserve last row for footer hint
    let list_height = inner.height.saturating_sub(1) as usize;
    let cursor = state.task_picker_cursor.min(items.len().saturating_sub(1));

    // Scroll offset: keep cursor visible
    let scroll = if cursor >= list_height {
        cursor - list_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, (plan_num, task)) in items.iter().enumerate().skip(scroll).take(list_height) {
        let (icon, color) = match task.status {
            TaskStatus::Done => ("✓", Theme::SAGE),
            TaskStatus::Active => ("►", Theme::ROSE),
            TaskStatus::Pending => ("·", Theme::TEXT_DIM),
            TaskStatus::Blocked => ("✗", Theme::EMBER),
        };
        let status_label = match task.status {
            TaskStatus::Done => "DONE   ",
            TaskStatus::Active => "ACTIVE ",
            TaskStatus::Pending => "PENDING",
            TaskStatus::Blocked => "BLOCKED",
        };

        let id_col = format!("{:<12}", task.id);
        let title_col = {
            let max = 38usize;
            if task.title.len() > max {
                format!("{:.width$}…", task.title, width = max - 1)
            } else {
                format!("{:<width$}", task.title, width = max)
            }
        };

        if i == cursor {
            lines.push(Line::from(vec![Span::styled(
                format!(
                    " {icon} {id_col} {title_col} {status_label}  [{}]",
                    plan_num
                ),
                Style::default()
                    .fg(Theme::BONE)
                    .bg(Theme::BG_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            )]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!(" {icon} "), Style::default().fg(color)),
                Span::styled(format!("{id_col} "), Style::default().fg(Theme::FG)),
                Span::styled(format!("{title_col} "), Style::default().fg(Theme::TEXT)),
                Span::styled(status_label, Style::default().fg(color)),
                Span::styled(
                    format!("  [{}]", plan_num),
                    Style::default().fg(Theme::TEXT_GHOST),
                ),
            ]));
        }
    }

    // Footer hint
    lines.push(Line::from(Span::styled(
        " j/k:move  Enter:ingest  Esc:close",
        Style::default().fg(Theme::TEXT_GHOST),
    )));

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
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
