use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::orchestrator::tasks::TaskStatus;
use crate::state::RunState;
use crate::tui::atmosphere::Atmosphere;
use crate::tui::bars::semantic_bar;
use crate::tui::theme::Theme;

use super::scrollbar;

pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere, focused: bool) {
    let selected_base = state
        .plans
        .get(state.selected_plan_idx)
        .map(|p| p.base.as_str());
    let cl_ref = selected_base.and_then(|b| state.checklist_for_plan(b));
    let (title, lines) = match cl_ref {
        Some(cl) => {
            // Count done tasks, applying executor_completed_tasks overlay
            let plan_label = selected_base.unwrap_or("?");
            let done: usize = cl
                .tasks
                .iter()
                .filter(|t| {
                    let key = format!("{}:{}", plan_label, t.id);
                    matches!(t.status, TaskStatus::Done)
                        || state.executor_completed_tasks.contains(&key)
                })
                .count();
            let total = cl.total();
            let title = format!("Tasks \u{00b7} {} ({}/{})", plan_label, done, total);

            let mut lines: Vec<Line> = Vec::new();

            // Progress bar
            let inner_width = area.width.saturating_sub(4) as usize;
            if inner_width > 8 && total > 0 {
                let est_remaining = cl.estimated_remaining_minutes();
                let fill_pct = done as f64 / total.max(1) as f64;
                let bar_spans = semantic_bar(inner_width, fill_pct, Some(atmosphere.heartbeat()));
                let suffix = if est_remaining > 0 {
                    format!(
                        "  {}/{}  ETA:{}",
                        done,
                        total,
                        crate::state::format_duration(est_remaining as u64 * 60)
                    )
                } else {
                    format!("  {}/{}", done, total)
                };
                let mut bar_line = vec![Span::styled(" ", Style::default())];
                bar_line.extend(bar_spans);
                bar_line.push(Span::styled(suffix, Style::default().fg(Theme::FG_DIM)));
                lines.push(Line::from(bar_line));
            }

            // Visible height minus border (2) minus progress bar (1)
            let visible = area.height.saturating_sub(3) as usize;
            let scroll = state.task_scroll.min(cl.tasks.len().saturating_sub(1));

            let start = scroll;
            let end = (scroll + visible).min(cl.tasks.len());

            // Scroll indicators
            if start > 0 {
                lines.push(Line::from(Span::styled(
                    " ▲ more",
                    Style::default().fg(Theme::TEXT_DIM),
                )));
            }

            for (i, task) in cl.tasks[start..end].iter().enumerate() {
                let global_idx = start + i;
                let is_selected = global_idx == scroll && focused;

                // Check executor_completed_tasks overlay: marks TOML-Pending tasks as Done
                // when the executor has completed them (e.g., resumed from prior run).
                let overlay_key = format!("{}:{}", plan_label, task.id);
                let effective_status = if task.status == TaskStatus::Pending
                    && state.executor_completed_tasks.contains(&overlay_key)
                {
                    TaskStatus::Done
                } else {
                    task.status.clone()
                };

                let (icon, style) = match effective_status {
                    TaskStatus::Done => ("✓", Style::default().fg(Theme::STATUS_OK)),
                    TaskStatus::Active => {
                        let pulse_color = pulse_rose(atmosphere.heartbeat());
                        (
                            "►",
                            Style::default()
                                .fg(pulse_color)
                                .add_modifier(Modifier::BOLD),
                        )
                    }
                    TaskStatus::Blocked => ("✗", Style::default().fg(Theme::STATUS_ERROR)),
                    TaskStatus::Pending => ("·", Style::default().fg(Theme::TEXT_DIM)),
                };

                let (text_style, bg) = if is_selected {
                    (
                        Style::default()
                            .fg(Theme::BONE)
                            .add_modifier(Modifier::BOLD)
                            .bg(Theme::BG_HIGHLIGHT),
                        Some(Theme::BG_HIGHLIGHT),
                    )
                } else if effective_status == TaskStatus::Active {
                    (Style::default().fg(Theme::FG), None)
                } else {
                    (Style::default().fg(Theme::FG_DIM), None)
                };

                let icon_style = if let Some(bg_color) = bg {
                    style.bg(bg_color)
                } else {
                    style
                };

                // Truncate title to fit available width
                let prefix_len = 4 + task.id.len() + 1; // " X " + id + " "
                let max_title = (area.width as usize).saturating_sub(prefix_len + 3); // 3 for border + scrollbar
                let title_display = if task.title.chars().count() > max_title && max_title > 3 {
                    format!(
                        " {}",
                        crate::tui::truncate_chars(&task.title, max_title, "...")
                    )
                } else {
                    format!(" {}", task.title)
                };

                // Time estimate / actual
                let time_span = match effective_status {
                    TaskStatus::Done => String::new(),
                    _ => task
                        .estimated_minutes
                        .map(|m| format!(" ~{}", crate::state::format_duration(m as u64 * 60)))
                        .unwrap_or_default(),
                };

                let mut task_spans = vec![
                    Span::styled(format!(" {icon} "), icon_style),
                    Span::styled(&task.id, icon_style),
                    Span::styled(title_display, text_style),
                ];
                if !time_span.is_empty() {
                    task_spans.push(Span::styled(time_span, Style::default().fg(Theme::DREAM)));
                }

                if effective_status == TaskStatus::Active {
                    // Find agent working on this task
                    let agent_label = state
                        .parallel_agents
                        .iter()
                        .find(|pa| {
                            pa.active
                                && pa.task == task.id
                                && (pa.plan.contains(plan_label) || plan_label.contains(&pa.plan))
                        })
                        .map(|pa| {
                            format!(" [{}]", pa.instance_id.chars().take(8).collect::<String>())
                        })
                        .unwrap_or_default();

                    // Countdown: estimated_minutes - elapsed since task started
                    let started_key = format!("{}:{}", plan_label, task.id);
                    let countdown = task.estimated_minutes.and_then(|est_m| {
                        state.task_started_at.get(&started_key).map(|t| {
                            let elapsed_secs = t.elapsed().as_secs();
                            let est_secs = est_m as u64 * 60;
                            est_secs.saturating_sub(elapsed_secs)
                        })
                    });

                    if !agent_label.is_empty() {
                        task_spans
                            .push(Span::styled(agent_label, Style::default().fg(Theme::DREAM)));
                    }
                    if let Some(rem) = countdown {
                        task_spans.push(Span::styled(
                            format!(" -{}", crate::state::format_duration(rem.max(1))),
                            Style::default().fg(Theme::EMBER),
                        ));
                    }
                }

                lines.push(Line::from(task_spans));

                // Expanded detail
                if state.task_expanded.as_deref() == Some(&task.id) {
                    for file in &task.files {
                        lines.push(Line::from(Span::styled(
                            format!("       - {file}"),
                            Style::default().fg(Theme::DREAM),
                        )));
                    }
                    for ac in &task.acceptance {
                        lines.push(Line::from(Span::styled(
                            format!("       > {ac}"),
                            Style::default().fg(Theme::TEXT_DIM),
                        )));
                    }
                    if !task.depends_on.is_empty() {
                        for dep in &task.depends_on {
                            let dep_status = cl
                                .tasks
                                .iter()
                                .find(|t| t.id == *dep)
                                .map(|t| match t.status {
                                    TaskStatus::Done => ("done", Theme::SAGE),
                                    _ => ("pending", Theme::TEXT_DIM),
                                })
                                .unwrap_or(("unknown", Theme::TEXT_DIM));
                            lines.push(Line::from(vec![
                                Span::styled(
                                    format!("       ~ {dep} "),
                                    Style::default().fg(Theme::TEXT_DIM),
                                ),
                                Span::styled(
                                    format!("({})", dep_status.0),
                                    Style::default().fg(dep_status.1),
                                ),
                            ]));
                        }
                    }
                }
            }

            if end < cl.tasks.len() {
                lines.push(Line::from(Span::styled(
                    " ▼ more",
                    Style::default().fg(Theme::TEXT_DIM),
                )));
            }

            (title, lines)
        }
        None => {
            let title = "Tasks".to_string();
            // Check if current plan is completed prior
            let is_completed_prior = state
                .current_plan()
                .map(|p| {
                    p.status == crate::state::RunPlanStatus::CompletedPrior
                        || p.status == crate::state::RunPlanStatus::Completed
                })
                .unwrap_or(false);
            let lines = if is_completed_prior {
                vec![Line::from(Span::styled(
                    "  Previously completed",
                    Style::default().fg(Theme::SAGE),
                ))]
            } else {
                vec![Line::from(Span::styled(
                    format!(" {} waiting for strategist...", atmosphere.spinner()),
                    Style::default().fg(Theme::TEXT_DIM),
                ))]
            };
            (title, lines)
        }
    };

    let (border_style, ttl_style) = if focused {
        (Theme::focused_border_style(), Theme::focused_title_style())
    } else {
        (
            Theme::unfocused_border_style(),
            Theme::unfocused_title_style(),
        )
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(border_style)
        .title_style(ttl_style);

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);

    // Scrollbar for task list (use the same resolved checklist, not just the live one)
    if let Some(cl) = cl_ref {
        let total_tasks = cl.tasks.len();
        let visible_tasks = area.height.saturating_sub(3) as usize; // border + progress bar
        if total_tasks > visible_tasks {
            let inner = Rect::new(
                area.x + 1,
                area.y + 1,
                area.width.saturating_sub(2),
                area.height.saturating_sub(2),
            );
            scrollbar::render_scrollbar(
                f.buffer_mut(),
                inner,
                total_tasks,
                visible_tasks,
                state.task_scroll,
                Theme::ROSE,
            );
        }
    }
}

/// Modulate ROSE color brightness with heartbeat oscillator
fn pulse_rose(heartbeat: f64) -> ratatui::style::Color {
    let base_r = 170.0;
    let base_g = 112.0;
    let base_b = 136.0;
    let scale = heartbeat.clamp(0.9, 1.1);
    ratatui::style::Color::Rgb(
        (base_r * scale).min(255.0) as u8,
        (base_g * scale).min(255.0) as u8,
        (base_b * scale).min(255.0) as u8,
    )
}
