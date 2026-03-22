use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::orchestrator::tasks::{Task, TaskStatus};
use crate::state::RunState;
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar;

/// Render a task detail modal showing full task info.
pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let popup = centered_rect(70, 75, area);
    f.render_widget(Clear, popup);

    let checklist = match &state.task_checklist {
        Some(cl) => cl,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Task Detail ")
                .border_style(Style::default().fg(Theme::DREAM))
                .style(Theme::default_style());
            let p = Paragraph::new(Line::from(Span::styled(
                " No task data available",
                Style::default().fg(Theme::TEXT_DIM),
            )))
            .block(block);
            f.render_widget(p, popup);
            return;
        }
    };

    let task = match checklist.tasks.get(state.task_scroll) {
        Some(t) => t,
        None => return,
    };

    let inner = Rect::new(
        popup.x + 1,
        popup.y + 1,
        popup.width.saturating_sub(2),
        popup.height.saturating_sub(2),
    );

    let mut lines: Vec<Line> = Vec::new();

    // Header: task ID and title
    let (status_icon, status_color) = match task.status {
        TaskStatus::Done => ("✓", Theme::SAGE),
        TaskStatus::Active => ("►", Theme::ROSE),
        TaskStatus::Pending => ("·", Theme::TEXT_DIM),
        TaskStatus::Blocked => ("✗", Theme::EMBER),
    };
    let status_label = match task.status {
        TaskStatus::Done => "DONE",
        TaskStatus::Active => "ACTIVE",
        TaskStatus::Pending => "PENDING",
        TaskStatus::Blocked => "BLOCKED",
    };

    lines.push(Line::from(vec![
        Span::styled(
            format!(" {status_icon} "),
            Style::default().fg(status_color),
        ),
        Span::styled(
            &task.id,
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {status_label}"),
            Style::default().fg(status_color),
        ),
    ]));
    lines.push(Line::from(""));

    // Title
    lines.push(Line::from(vec![
        Span::styled(" Title: ", Style::default().fg(Theme::BONE_DIM)),
        Span::styled(task.title.clone(), Style::default().fg(Theme::TEXT)),
    ]));
    lines.push(Line::from(""));

    // Estimate
    if let Some(est) = task.estimated_minutes {
        lines.push(Line::from(vec![
            Span::styled(" Estimate: ", Style::default().fg(Theme::BONE_DIM)),
            Span::styled(format!("~{est} min"), Style::default().fg(Theme::DREAM)),
        ]));
        lines.push(Line::from(""));
    }

    // Parallel group
    if let Some(ref group) = task.parallel_group {
        lines.push(Line::from(vec![
            Span::styled(" Parallel group: ", Style::default().fg(Theme::BONE_DIM)),
            Span::styled(group.clone(), Style::default().fg(Theme::DREAM)),
        ]));
    }

    // Exclusive files
    lines.push(Line::from(vec![
        Span::styled(" Exclusive files: ", Style::default().fg(Theme::BONE_DIM)),
        Span::styled(
            if task.exclusive_files { "yes" } else { "no" },
            Style::default().fg(Theme::FG),
        ),
    ]));
    lines.push(Line::from(""));

    // Dependencies
    if !task.depends_on.is_empty() {
        lines.push(Line::from(Span::styled(
            " Dependencies",
            Style::default()
                .fg(Theme::BONE_DIM)
                .add_modifier(Modifier::BOLD),
        )));
        for dep_id in &task.depends_on {
            let dep_task = checklist.tasks.iter().find(|t| t.id == *dep_id);
            let (dep_icon, dep_color) = match dep_task.map(|t| &t.status) {
                Some(TaskStatus::Done) => ("✓", Theme::SAGE),
                Some(TaskStatus::Active) => ("►", Theme::ROSE),
                Some(TaskStatus::Blocked) => ("✗", Theme::EMBER),
                _ => ("·", Theme::TEXT_DIM),
            };
            let dep_title = dep_task.map(|t| t.title.as_str()).unwrap_or("(unknown)");
            lines.push(Line::from(vec![
                Span::styled(format!("   {dep_icon} "), Style::default().fg(dep_color)),
                Span::styled(dep_id.clone(), Style::default().fg(dep_color)),
                Span::styled(
                    format!(" — {dep_title}"),
                    Style::default().fg(Theme::FG_DIM),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Dependents (tasks that depend on this one)
    let dependents: Vec<&Task> = checklist
        .tasks
        .iter()
        .filter(|t| t.depends_on.contains(&task.id))
        .collect();
    if !dependents.is_empty() {
        lines.push(Line::from(Span::styled(
            " Blocked by this task",
            Style::default()
                .fg(Theme::BONE_DIM)
                .add_modifier(Modifier::BOLD),
        )));
        for dep in &dependents {
            let (dep_icon, dep_color) = match dep.status {
                TaskStatus::Done => ("✓", Theme::SAGE),
                TaskStatus::Active => ("►", Theme::ROSE),
                TaskStatus::Blocked => ("✗", Theme::EMBER),
                _ => ("·", Theme::TEXT_DIM),
            };
            lines.push(Line::from(vec![
                Span::styled(format!("   {dep_icon} "), Style::default().fg(dep_color)),
                Span::styled(dep.id.clone(), Style::default().fg(dep_color)),
                Span::styled(
                    format!(" — {}", dep.title),
                    Style::default().fg(Theme::FG_DIM),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Files
    if !task.files.is_empty() {
        lines.push(Line::from(Span::styled(
            " Files",
            Style::default()
                .fg(Theme::BONE_DIM)
                .add_modifier(Modifier::BOLD),
        )));
        for file in &task.files {
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(file.clone(), Style::default().fg(Theme::DREAM)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Acceptance criteria
    if !task.acceptance.is_empty() {
        lines.push(Line::from(Span::styled(
            " Acceptance Criteria",
            Style::default()
                .fg(Theme::BONE_DIM)
                .add_modifier(Modifier::BOLD),
        )));
        for (i, criterion) in task.acceptance.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("   {}. ", i + 1), Style::default().fg(Theme::ROSE)),
                Span::styled(criterion.clone(), Style::default().fg(Theme::TEXT)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Context within checklist
    lines.push(Line::from(Span::styled(
        " ─── Checklist Context ───",
        Style::default().fg(Theme::TEXT_GHOST),
    )));
    lines.push(Line::from(vec![
        Span::styled(" Plan: ", Style::default().fg(Theme::BONE_DIM)),
        Span::styled(
            format!("{} (iter {})", checklist.plan_num, checklist.iteration),
            Style::default().fg(Theme::FG),
        ),
    ]));
    let done = checklist.done_count();
    let total = checklist.total();
    lines.push(Line::from(vec![
        Span::styled(" Progress: ", Style::default().fg(Theme::BONE_DIM)),
        Span::styled(
            format!("{done}/{total} tasks done"),
            Style::default().fg(if done == total {
                Theme::SAGE
            } else {
                Theme::TEXT
            }),
        ),
    ]));

    // Render
    let total_lines = lines.len();
    let visible = inner.height as usize;
    let scroll = state
        .task_detail_scroll
        .min(total_lines.saturating_sub(visible));

    let display: Vec<Line> = lines.into_iter().skip(scroll).take(visible).collect();

    let title = format!(" Task: {} [Esc:close j/k:scroll] ", task.id);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Theme::DREAM))
        .style(Theme::default_style());
    f.render_widget(block, popup);

    let paragraph = Paragraph::new(display).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);

    if total_lines > visible {
        scrollbar::render_scrollbar(
            f.buffer_mut(),
            inner,
            total_lines,
            visible,
            scroll,
            Theme::DREAM,
        );
    }
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
