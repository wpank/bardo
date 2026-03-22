use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::orchestrator::tasks::{TaskChecklist, TaskStatus};
use crate::state::{format_duration, FocusZone, RunPlanStatus, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::bars::gradient_bar;
use crate::tui::color::ocean_gradient;
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar::render_scrollbar;

/// Hierarchical Plans browser: waves on the left, plan detail on the right.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    // Three-column split: left 35% | spacer 1 cell | right 65%
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(1),
            Constraint::Percentage(75),
        ])
        .split(area);

    let left_focused = state.focus == FocusZone::Plans;
    render_left_panel(f, columns[0], state, atmosphere, left_focused);
    // spacer column is untouched (renders as background)
    if state.pipeline_header_selected {
        render_pipeline_detail(f, columns[2], state, atmosphere);
    } else {
        render_right_panel(f, columns[2], state, atmosphere);
    }
}

// ---------------------------------------------------------------------------
// Left panel: wave list + plans in selected wave
// ---------------------------------------------------------------------------

fn render_left_panel(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    atmosphere: &Atmosphere,
    focused: bool,
) {
    let (border_style, title_style) = if focused {
        (Theme::focused_border_style(), Theme::focused_title_style())
    } else {
        (
            Theme::unfocused_border_style(),
            Theme::unfocused_title_style(),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(" Waves / Plans ", title_style));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.execution_waves.is_empty() {
        render_flat_plan_list(f, inner, state, atmosphere);
        return;
    }

    // Split inner: wave list (dynamic) + separator + plan list (grows)
    // +1 for the pipeline header row prepended before the wave list
    let wave_count = state.execution_waves.len() as u16;
    let wave_height = (wave_count + 1).min(inner.height.saturating_sub(4));

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(wave_height),
            Constraint::Length(1), // "Plans in W{N}:" header
            Constraint::Min(1),    // plan rows
        ])
        .split(inner);

    render_wave_list(f, sections[0], state, atmosphere);
    render_plans_header(f, sections[1], state);
    render_plans_in_wave(f, sections[2], state, atmosphere);
}

fn render_wave_list(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    use crate::agent::AgentRole;
    let breathing = Some(atmosphere.breathing_brightness());
    let grad = ocean_gradient();

    let mut lines: Vec<Line> = Vec::new();

    // Pipeline header row
    {
        let meta_roles = [
            AgentRole::PrePlanner,
            AgentRole::DocVerifier,
            AgentRole::IntegrationTester,
            AgentRole::MergeResolver,
            AgentRole::Refactorer,
        ];
        let active_count = state
            .parallel_agents
            .iter()
            .filter(|pa| pa.active && meta_roles.contains(&pa.role))
            .count();
        let selected = state.pipeline_header_selected;
        let bg = if selected {
            Theme::BG_HIGHLIGHT
        } else {
            Theme::BG
        };
        let cursor = if selected { "► " } else { "  " };
        let count_str = if active_count > 0 {
            format!(" [{active_count} meta active]")
        } else {
            String::new()
        };
        let mut spans = vec![
            Span::styled(cursor, Style::default().fg(Theme::ROSE).bg(bg)),
            Span::styled(
                "\u{25C8}", // ◈
                Style::default().fg(Theme::DREAM).bg(bg),
            ),
            Span::styled(
                " Pipeline",
                Style::default()
                    .fg(if selected {
                        Theme::FG_BRIGHT
                    } else {
                        Theme::FG
                    })
                    .bg(bg)
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            Span::styled(count_str, Style::default().fg(Theme::STATUS_ACTIVE).bg(bg)),
        ];
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (area.width as usize).saturating_sub(used);
        if remaining > 0 {
            spans.push(Span::styled(" ".repeat(remaining), Style::default().bg(bg)));
        }
        lines.push(Line::from(spans));
    }

    for (i, (wave_num, plan_bases)) in state.execution_waves.iter().enumerate() {
        let done = plan_bases
            .iter()
            .filter(|b| {
                state.plans.iter().any(|p| {
                    &p.base == *b
                        && matches!(
                            p.status,
                            RunPlanStatus::Completed
                                | RunPlanStatus::CompletedPrior
                                | RunPlanStatus::MergedToMain
                        )
                })
            })
            .count();
        let total = plan_bases.len();
        let all_done = done == total && total > 0;
        let any_active = plan_bases.iter().any(|b| {
            state
                .plans
                .iter()
                .any(|p| p.base == **b && p.status == RunPlanStatus::Active)
        });

        let icon = if all_done {
            "✓"
        } else if any_active {
            "►"
        } else {
            "·"
        };

        let icon_color = if all_done {
            Theme::STATUS_OK
        } else if any_active {
            Theme::STATUS_ACTIVE
        } else {
            Theme::TEXT_GHOST
        };

        let selected = i == state.selected_wave_idx;
        let bg = if selected {
            Theme::BG_HIGHLIGHT
        } else {
            Theme::BG
        };

        let cursor = if selected { "► " } else { "  " };

        let fill_pct = if total > 0 {
            done as f64 / total as f64
        } else {
            0.0
        };
        let bar_spans = gradient_bar(8, fill_pct, &grad, breathing);
        let count_str = format!(" {done}/{total}");

        let mut spans = vec![
            Span::styled(cursor, Style::default().fg(Theme::ROSE).bg(bg)),
            Span::styled(icon, Style::default().fg(icon_color).bg(bg)),
            Span::styled(
                format!(" W{} ", wave_num),
                Style::default()
                    .fg(if selected {
                        Theme::FG_BRIGHT
                    } else {
                        Theme::FG
                    })
                    .bg(bg)
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ];
        for mut s in bar_spans {
            s.style = s.style.bg(bg);
            spans.push(s);
        }
        spans.push(Span::styled(
            count_str,
            Style::default().fg(Theme::TEXT_DIM).bg(bg),
        ));

        // Fill remaining width with background
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (area.width as usize).saturating_sub(used);
        if remaining > 0 {
            spans.push(Span::styled(" ".repeat(remaining), Style::default().bg(bg)));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

fn render_plans_header(f: &mut Frame, area: Rect, state: &RunState) {
    let wave_num = state
        .execution_waves
        .get(state.selected_wave_idx)
        .map(|(n, _)| *n)
        .unwrap_or(0);

    let line = Line::from(vec![Span::styled(
        format!(" Plans in W{wave_num}:"),
        Style::default()
            .fg(Theme::BONE_DIM)
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(Paragraph::new(line), area);
}

fn render_plans_in_wave(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let plan_bases: Vec<&String> = state
        .execution_waves
        .get(state.selected_wave_idx)
        .map(|(_, bases)| bases.iter().collect())
        .unwrap_or_default();

    let mut lines: Vec<Line> = Vec::new();
    for (i, base) in plan_bases.iter().enumerate() {
        let plan = state.plans.iter().find(|p| &&p.base == base);
        let selected = i == state.selected_plan_idx;
        let bg = if selected {
            Theme::BG_HIGHLIGHT
        } else {
            Theme::BG
        };

        let (icon, icon_color) = match plan.map(|p| &p.status) {
            Some(RunPlanStatus::MergedToMain) => ("\u{2b06}", Theme::BONE), // ⬆
            Some(RunPlanStatus::Completed | RunPlanStatus::CompletedPrior) => {
                ("✓", Theme::STATUS_OK)
            }
            Some(RunPlanStatus::Active) => ("►", Theme::STATUS_ACTIVE),
            Some(RunPlanStatus::Failed) => ("✗", Theme::STATUS_ERROR),
            Some(RunPlanStatus::Skipped) => ("\u{2298}", Theme::TEXT_GHOST), // ⊘
            _ => ("·", Theme::TEXT_GHOST),
        };

        let time_info = plan_time_info(plan, atmosphere);

        let mut spans = vec![
            Span::styled(
                if selected { " ► " } else { "   " },
                Style::default().fg(Theme::ROSE).bg(bg),
            ),
            Span::styled(icon, Style::default().fg(icon_color).bg(bg)),
            Span::styled(
                format!(" {}", base),
                Style::default()
                    .fg(if selected {
                        Theme::FG_BRIGHT
                    } else {
                        Theme::FG
                    })
                    .bg(bg)
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ];

        if let Some(p) = plan {
            if p.status == RunPlanStatus::Active && !p.phase.is_empty() {
                spans.push(Span::styled(
                    format!(" [{}]", p.phase),
                    Style::default().fg(Theme::ROSE_DIM).bg(bg),
                ));
            }
        }

        if !time_info.is_empty() {
            spans.push(Span::styled(
                format!(" {time_info}"),
                Style::default().fg(Theme::TEXT_DIM).bg(bg),
            ));
        }

        // MergedToMain: show commit hash instead of branch
        if let Some(p) = plan {
            if p.status == RunPlanStatus::MergedToMain {
                if let Some(ref hash) = p.merge_commit {
                    spans.push(Span::styled(
                        " │",
                        Style::default().fg(Theme::ROSE_DIM).bg(bg),
                    ));
                    spans.push(Span::styled(
                        format!(" main@{hash}"),
                        Style::default().fg(Theme::BONE_DIM).bg(bg),
                    ));
                }
                let used: usize = spans.iter().map(|s| s.content.len()).sum();
                let remaining = (area.width as usize).saturating_sub(used);
                if remaining > 0 {
                    spans.push(Span::styled(" ".repeat(remaining), Style::default().bg(bg)));
                }
                lines.push(Line::from(spans));
                continue;
            }
        }

        // Git info: branch path (always), live worktree data (when available)
        if let Some(p) = plan {
            if let Some(ref branch) = p.git_branch_short {
                spans.push(Span::styled(
                    " │",
                    Style::default().fg(Theme::ROSE_DIM).bg(bg),
                ));
                let has_live_data = p.git_last_commit_secs.is_some() || p.git_dirty.is_some();
                if has_live_data {
                    if let Some(secs) = p.git_last_commit_secs {
                        if secs > 0 {
                            spans.push(Span::styled(
                                format!(" {}", crate::state::format_ago(secs)),
                                Style::default().fg(Theme::TEXT_GHOST).bg(bg),
                            ));
                        }
                    }
                    if let Some((added, removed)) = p.git_dirty {
                        spans.push(Span::styled(
                            format!(" +{added}"),
                            Style::default().fg(Theme::SAGE).bg(bg),
                        ));
                        spans.push(Span::styled(
                            format!("-{removed}"),
                            Style::default().fg(Theme::EMBER).bg(bg),
                        ));
                    }
                } else {
                    spans.push(Span::styled(
                        format!(" ⎇ {branch}"),
                        Style::default().fg(Theme::TEXT_GHOST).bg(bg),
                    ));
                }
            }
        }

        // Fill remaining width
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (area.width as usize).saturating_sub(used);
        if remaining > 0 {
            spans.push(Span::styled(" ".repeat(remaining), Style::default().bg(bg)));
        }

        lines.push(Line::from(spans));
    }

    let visible = area.height as usize;
    let total = lines.len();
    let offset = if state.selected_plan_idx >= visible {
        state.selected_plan_idx - visible + 1
    } else {
        0
    };
    let visible_lines: Vec<Line> = lines.into_iter().skip(offset).take(visible).collect();

    let para = Paragraph::new(visible_lines);
    f.render_widget(para, area);

    if total > visible {
        let buf = f.buffer_mut();
        render_scrollbar(buf, area, total, visible, offset, Theme::ROSE_DIM);
    }
}

/// Fallback: flat plan list when no waves exist.
fn render_flat_plan_list(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let mut lines: Vec<Line> = Vec::new();
    for (i, plan) in state.plans.iter().enumerate() {
        let selected = i == state.selected_plan_idx;
        let bg = if selected {
            Theme::BG_HIGHLIGHT
        } else {
            Theme::BG
        };

        let (icon, icon_color) = match &plan.status {
            RunPlanStatus::MergedToMain => ("\u{2b06}", Theme::BONE), // ⬆
            RunPlanStatus::Completed | RunPlanStatus::CompletedPrior => ("✓", Theme::STATUS_OK),
            RunPlanStatus::Active => ("►", Theme::STATUS_ACTIVE),
            RunPlanStatus::Failed => ("✗", Theme::STATUS_ERROR),
            RunPlanStatus::Skipped => ("\u{2298}", Theme::TEXT_GHOST),
            RunPlanStatus::Pending => ("·", Theme::TEXT_GHOST),
        };

        let time_info = plan_time_info(Some(plan), atmosphere);

        let mut spans = vec![
            Span::styled(
                if selected { " ► " } else { "   " },
                Style::default().fg(Theme::ROSE).bg(bg),
            ),
            Span::styled(icon, Style::default().fg(icon_color).bg(bg)),
            Span::styled(
                format!(" {}", plan.base),
                Style::default()
                    .fg(if selected {
                        Theme::FG_BRIGHT
                    } else {
                        Theme::FG
                    })
                    .bg(bg)
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ];

        if plan.status == RunPlanStatus::Active && !plan.phase.is_empty() {
            spans.push(Span::styled(
                format!(" [{}]", plan.phase),
                Style::default().fg(Theme::ROSE_DIM).bg(bg),
            ));
        }

        if !time_info.is_empty() {
            spans.push(Span::styled(
                format!(" {time_info}"),
                Style::default().fg(Theme::TEXT_DIM).bg(bg),
            ));
        }

        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (area.width as usize).saturating_sub(used);
        if remaining > 0 {
            spans.push(Span::styled(" ".repeat(remaining), Style::default().bg(bg)));
        }

        lines.push(Line::from(spans));
    }

    let visible = area.height as usize;
    let total = lines.len();
    let offset = if state.selected_plan_idx >= visible {
        state.selected_plan_idx - visible + 1
    } else {
        0
    };
    let visible_lines: Vec<Line> = lines.into_iter().skip(offset).take(visible).collect();
    f.render_widget(Paragraph::new(visible_lines), area);

    if total > visible {
        let buf = f.buffer_mut();
        render_scrollbar(buf, area, total, visible, offset, Theme::ROSE_DIM);
    }
}

/// Derive a short time string for a plan row.
fn plan_time_info(plan: Option<&crate::state::RunPlanEntry>, _atmosphere: &Atmosphere) -> String {
    let Some(plan) = plan else {
        return String::new();
    };
    match &plan.status {
        RunPlanStatus::MergedToMain => {
            if let Some(at) = plan.merged_to_main_at {
                format!("⬆ {}", crate::state::format_ago(at))
            } else {
                "⬆ main".to_string()
            }
        }
        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior => {
            if let Some(mins) = plan.actual_minutes {
                format_duration(mins as u64 * 60)
            } else {
                String::new()
            }
        }
        RunPlanStatus::Active => {
            if let Some(started) = plan.started_at {
                let elapsed_secs = started.elapsed().as_secs();
                if !plan.phase.is_empty() {
                    format!("{} in {}", format_duration(elapsed_secs), plan.phase)
                } else {
                    format_duration(elapsed_secs)
                }
            } else {
                String::new()
            }
        }
        RunPlanStatus::Failed => "err".to_string(),
        RunPlanStatus::Pending => {
            if let Some(est) = plan.estimated_minutes {
                format!("~{est}m")
            } else {
                String::new()
            }
        }
        RunPlanStatus::Skipped => "skip".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Right panel: selected plan detail
// ---------------------------------------------------------------------------

fn render_right_panel(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let detail_focused = state.focus == FocusZone::Tasks;
    let (border_style, title_style) = if detail_focused {
        (Theme::focused_border_style(), Theme::focused_title_style())
    } else {
        (
            Theme::unfocused_border_style(),
            Theme::unfocused_title_style(),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(" Detail ", title_style));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let selected_plan = resolve_selected_plan(state);
    let Some(plan) = selected_plan else {
        let empty = Paragraph::new(Span::styled(
            "No plan selected",
            Style::default().fg(Theme::TEXT_GHOST),
        ));
        f.render_widget(empty, inner);
        return;
    };

    // Vertical layout: header | task section | agents | git info
    let checklist = resolve_checklist(state, &plan.base);
    let task_count = checklist.as_ref().map(|cl| cl.tasks.len()).unwrap_or(0);
    let task_height = if task_count > 0 {
        (task_count as u16 + 3).min(inner.height.saturating_sub(10))
    } else {
        1
    };

    let agent_count = count_agents_on_plan(state, &plan.base);
    let agent_height = if agent_count > 0 {
        (agent_count as u16 + 2).min(6)
    } else {
        1
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),            // plan header
            Constraint::Length(task_height),  // tasks
            Constraint::Length(agent_height), // agents
            Constraint::Length(4),            // git info
            Constraint::Min(0),               // spacer (absorbs remainder)
        ])
        .split(inner);

    render_plan_header(f, sections[0], plan, atmosphere);
    render_task_section(f, sections[1], &plan.base, checklist, atmosphere, state);
    render_agent_section(f, sections[2], state, &plan.base, atmosphere);
    render_git_section(f, sections[3], state, &plan.base);
}

fn render_plan_header(
    f: &mut Frame,
    area: Rect,
    plan: &crate::state::RunPlanEntry,
    atmosphere: &Atmosphere,
) {
    let mut lines: Vec<Line> = Vec::new();

    // Line 1: Plan name
    lines.push(Line::from(vec![
        Span::styled("Plan: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(
            &plan.base,
            Style::default()
                .fg(Theme::FG_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Line 2: Status + Phase
    let status_badge = plan_status_badge(plan);
    let mut line2 = vec![
        Span::styled("Status: ", Style::default().fg(Theme::TEXT_DIM)),
        status_badge,
    ];
    if !plan.phase.is_empty() {
        line2.push(Span::styled(
            format!("  Phase: {}", plan.phase),
            Style::default().fg(Theme::TEXT_DIM),
        ));
    }
    lines.push(Line::from(line2));

    // Line 3: Elapsed + ETA
    let mut line3_spans: Vec<Span> = Vec::new();
    if let Some(started) = plan.started_at {
        let elapsed = started.elapsed().as_secs();
        line3_spans.push(Span::styled(
            format!("Elapsed: {}", format_duration(elapsed)),
            Style::default().fg(Theme::TEXT_DIM),
        ));
        if let Some(est) = plan.estimated_minutes {
            let est_secs = est as u64 * 60;
            let remaining = est_secs.saturating_sub(elapsed);
            line3_spans.push(Span::styled(
                format!("  ETA: ~{}", format_duration(remaining)),
                Style::default().fg(Theme::TEXT_GHOST),
            ));
        }
    } else if let Some(est) = plan.estimated_minutes {
        line3_spans.push(Span::styled(
            format!("ETA: ~{est}m"),
            Style::default().fg(Theme::TEXT_GHOST),
        ));
    }
    if !line3_spans.is_empty() {
        lines.push(Line::from(line3_spans));
    }

    // Line 4: Iteration
    if plan.iteration > 0 {
        let spinner = if plan.status == RunPlanStatus::Active {
            format!(" {}", atmosphere.spinner())
        } else {
            String::new()
        };
        lines.push(Line::from(vec![Span::styled(
            format!("Iteration: {} of 3 max{spinner}", plan.iteration),
            Style::default().fg(Theme::TEXT_DIM),
        )]));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

fn render_task_section(
    f: &mut Frame,
    area: Rect,
    plan_base: &str,
    checklist: Option<&TaskChecklist>,
    atmosphere: &Atmosphere,
    state: &RunState,
) {
    let Some(cl) = checklist else {
        let empty = Paragraph::new(Span::styled(
            "No task data",
            Style::default().fg(Theme::TEXT_GHOST),
        ));
        f.render_widget(empty, area);
        return;
    };

    let done = cl.done_count();
    let total = cl.total();
    let fill_pct = if total > 0 {
        done as f64 / total as f64
    } else {
        0.0
    };

    let breathing = Some(atmosphere.breathing_brightness());
    let grad = ocean_gradient();
    let bar_spans = gradient_bar(12, fill_pct, &grad, breathing);

    let mut header_spans = vec![Span::styled(
        "Tasks: ",
        Style::default()
            .fg(Theme::BONE_DIM)
            .add_modifier(Modifier::BOLD),
    )];
    for s in bar_spans {
        header_spans.push(s);
    }
    header_spans.push(Span::styled(
        format!(" {done}/{total}"),
        Style::default().fg(Theme::TEXT_DIM),
    ));

    let mut lines: Vec<Line> = vec![Line::from(header_spans)];

    let spinner_char = atmosphere.spinner();
    for task in &cl.tasks {
        let task_key = format!("{}:{}", plan_base, task.id);
        let effective_status = if state.executor_completed_tasks.contains(&task_key) {
            &TaskStatus::Done
        } else {
            &task.status
        };
        let (icon, icon_color): (String, _) = match effective_status {
            TaskStatus::Done => ("✓".to_string(), Theme::STATUS_OK),
            TaskStatus::Active => (format!("{}", spinner_char), Theme::STATUS_ACTIVE),
            TaskStatus::Pending => ("·".to_string(), Theme::TEXT_GHOST),
            TaskStatus::Blocked => ("✗".to_string(), Theme::STATUS_ERROR),
        };

        let mut spans = vec![
            Span::styled("  ", Style::default()),
            Span::styled(icon, Style::default().fg(icon_color)),
            Span::styled(
                format!(" {} {}", task.id, task.title),
                Style::default().fg(match effective_status {
                    TaskStatus::Done => Theme::TEXT_DIM,
                    TaskStatus::Active => Theme::FG_BRIGHT,
                    _ => Theme::FG,
                }),
            ),
        ];

        if effective_status != &TaskStatus::Done {
            if let Some(est) = task.estimated_minutes {
                spans.push(Span::styled(
                    format!("  ~{est}m"),
                    Style::default().fg(Theme::TEXT_GHOST),
                ));
            }
        }

        lines.push(Line::from(spans));
    }

    let visible = area.height as usize;
    let total_lines = lines.len();
    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, area);

    if total_lines > visible {
        let buf = f.buffer_mut();
        render_scrollbar(buf, area, total_lines, visible, 0, Theme::ROSE_DIM);
    }
}

fn render_agent_section(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    plan_base: &str,
    atmosphere: &Atmosphere,
) {
    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        "Agents on this plan:",
        Style::default()
            .fg(Theme::BONE_DIM)
            .add_modifier(Modifier::BOLD),
    ))];

    let mut found_any = false;
    for (role, agent) in &state.agents {
        let matches = agent
            .current_plan
            .as_ref()
            .map(|cp| cp == plan_base)
            .unwrap_or(false);
        if !matches && !agent.active {
            continue;
        }
        // If agent is active but on a different plan, skip
        if !matches {
            continue;
        }
        found_any = true;

        let accent = Theme::role_accent(*role);
        let (icon, status_str) = if agent.active {
            (
                format!("{}", atmosphere.spinner()),
                format!(
                    " {}k/{}k",
                    agent.input_tokens / 1000,
                    state.context_limit / 1000
                ),
            )
        } else {
            ("✓".to_string(), " (done)".to_string())
        };

        let mut spans = vec![
            Span::styled("  ", Style::default()),
            Span::styled(icon, Style::default().fg(accent)),
            Span::styled(
                format!(" {}", role.short()),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(status_str, Style::default().fg(Theme::TEXT_DIM)),
        ];

        // Mini token bar for active agents
        if agent.active && state.context_limit > 0 {
            let fill = agent.input_tokens as f64 / state.context_limit as f64;
            let bar_width: usize = 6;
            let filled = (fill * bar_width as f64).round() as usize;
            let empty = bar_width.saturating_sub(filled);
            spans.push(Span::styled(" ", Style::default()));
            if filled > 0 {
                spans.push(Span::styled(
                    "\u{25B8}".repeat(filled), // ▸
                    Style::default().fg(accent),
                ));
            }
            if empty > 0 {
                spans.push(Span::styled(
                    "\u{2591}".repeat(empty), // ░
                    Style::default().fg(Theme::TEXT_GHOST),
                ));
            }
        }

        lines.push(Line::from(spans));
    }

    // In parallel mode, also show instances from parallel_agents filtered by this plan.
    if !state.parallel_agents.is_empty() {
        for pa in state
            .parallel_agents
            .iter()
            .filter(|p| p.plan.contains(plan_base) || plan_base.contains(&p.plan))
        {
            found_any = true;
            let accent = crate::tui::theme::Theme::role_accent(pa.role);
            let (icon, status_str) = if pa.active {
                (
                    format!("{}", atmosphere.spinner()),
                    format!(
                        " {}k/{}k",
                        pa.input_tokens / 1000,
                        state.context_limit / 1000
                    ),
                )
            } else {
                ("✓".to_string(), " (done)".to_string())
            };
            let mut spans = vec![
                Span::styled("  ", Style::default()),
                Span::styled(icon, Style::default().fg(accent)),
                Span::styled(
                    format!(" {}", pa.role.short()),
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(status_str, Style::default().fg(Theme::TEXT_DIM)),
            ];
            if pa.active && state.context_limit > 0 {
                let fill = pa.input_tokens as f64 / state.context_limit as f64;
                let bar_width: usize = 6;
                let filled = (fill * bar_width as f64).round() as usize;
                let empty = bar_width.saturating_sub(filled);
                spans.push(Span::styled(" ", Style::default()));
                if filled > 0 {
                    spans.push(Span::styled(
                        "\u{25B8}".repeat(filled),
                        Style::default().fg(accent),
                    ));
                }
                if empty > 0 {
                    spans.push(Span::styled(
                        "\u{2591}".repeat(empty),
                        Style::default().fg(Theme::TEXT_GHOST),
                    ));
                }
            }
            lines.push(Line::from(spans));
        }
    }

    if !found_any {
        lines.push(Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Theme::TEXT_GHOST),
        )));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

fn render_git_section(f: &mut Frame, area: Rect, state: &RunState, plan_base: &str) {
    let branch = format!("codex/plan/{plan_base}");
    let worktree = format!(".worktrees/plan-{plan_base}");

    // Look up git data from the plan entry
    let plan_entry = state.plans.iter().find(|p| p.base == plan_base);

    let mut lines: Vec<Line> = Vec::new();

    // Line 1: Branch
    lines.push(Line::from(vec![
        Span::styled("Branch:  ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(branch, Style::default().fg(Theme::DREAM)),
    ]));

    // Line 2: Worktree path
    let worktree_exists = state.git_worktree_list.iter().any(|wt| {
        std::path::Path::new(&wt.path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == format!("plan-{plan_base}"))
            .unwrap_or(false)
    });
    let worktree_style = if worktree_exists {
        Style::default().fg(Theme::SAGE)
    } else {
        Style::default().fg(Theme::TEXT_GHOST)
    };
    lines.push(Line::from(vec![
        Span::styled("Worktree: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(worktree, worktree_style),
        if worktree_exists {
            Span::styled(" ●", Style::default().fg(Theme::SAGE))
        } else {
            Span::styled(" (gone)", Style::default().fg(Theme::TEXT_GHOST))
        },
    ]));

    // Line 3: Commit time + dirty state (from polling)
    if let Some(plan) = plan_entry {
        let mut commit_spans: Vec<Span> = vec![Span::styled(
            "Commit:  ",
            Style::default().fg(Theme::TEXT_DIM),
        )];
        match plan.git_last_commit_secs {
            Some(secs) if secs > 0 => {
                commit_spans.push(Span::styled(
                    crate::state::format_ago(secs),
                    Style::default().fg(Theme::TEXT_DIM),
                ));
            }
            _ => {
                commit_spans.push(Span::styled(
                    "unknown",
                    Style::default().fg(Theme::TEXT_GHOST),
                ));
            }
        }
        if let Some((added, removed)) = plan.git_dirty {
            commit_spans.push(Span::styled(
                format!("  +{added}"),
                Style::default().fg(Theme::SAGE),
            ));
            commit_spans.push(Span::styled(
                format!("-{removed} uncommitted",),
                Style::default().fg(Theme::EMBER),
            ));
        }
        lines.push(Line::from(commit_spans));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the RunPlanEntry for the currently selected plan.
fn resolve_selected_plan(state: &RunState) -> Option<&crate::state::RunPlanEntry> {
    if state.execution_waves.is_empty() {
        return state.plans.get(state.selected_plan_idx);
    }
    let plan_bases = state
        .execution_waves
        .get(state.selected_wave_idx)
        .map(|(_, bases)| bases)?;
    let base = plan_bases.get(state.selected_plan_idx)?;
    state.plans.iter().find(|p| &p.base == base)
}

/// Get the task checklist for a plan, preferring the live one if it matches.
fn resolve_checklist<'a>(state: &'a RunState, plan_base: &str) -> Option<&'a TaskChecklist> {
    // If the live checklist matches this plan, use it
    if let Some(ref cl) = state.task_checklist {
        // The plan_num field on TaskChecklist is the numeric prefix; match against plan base
        if plan_base.contains(&cl.plan_num) || cl.plan_num.contains(plan_base) {
            return Some(cl);
        }
    }
    // Otherwise check the cache
    state.plan_task_cache.get(plan_base)
}

/// Count agents currently assigned to a given plan.
fn count_agents_on_plan(state: &RunState, plan_base: &str) -> usize {
    state
        .agents
        .values()
        .filter(|a| {
            a.current_plan
                .as_ref()
                .map(|cp| cp == plan_base)
                .unwrap_or(false)
        })
        .count()
}

/// Pipeline-level detail view: shown when the pipeline header row is selected.
fn render_pipeline_detail(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    use crate::agent::AgentRole;

    let (border_style, title_style) = (Theme::focused_border_style(), Theme::focused_title_style());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(" Pipeline ", title_style));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Batch branch
    lines.push(Line::from(vec![
        Span::styled("Branch: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(state.git_branch.clone(), Style::default().fg(Theme::DREAM)),
    ]));

    lines.push(Line::from(Span::styled(
        "Meta Agents",
        Style::default()
            .fg(Theme::BONE_DIM)
            .add_modifier(Modifier::BOLD),
    )));

    let meta_roles = [
        AgentRole::PrePlanner,
        AgentRole::DocVerifier,
        AgentRole::IntegrationTester,
        AgentRole::MergeResolver,
        AgentRole::Refactorer,
    ];

    let mut found_any = false;
    for pa in &state.parallel_agents {
        if !meta_roles.contains(&pa.role) {
            continue;
        }
        found_any = true;
        let accent = Theme::role_accent(pa.role);
        let (icon, status_str) = if pa.active {
            (
                format!("{}", atmosphere.spinner()),
                format!(
                    " {}k/{}k",
                    pa.input_tokens / 1000,
                    state.context_limit / 1000
                ),
            )
        } else {
            ("✓".to_string(), " (done)".to_string())
        };
        let mut spans = vec![
            Span::styled("  ", Style::default()),
            Span::styled(icon, Style::default().fg(accent)),
            Span::styled(
                format!(" {}", pa.role.short()),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", pa.instance_id),
                Style::default().fg(Theme::TEXT_DIM),
            ),
            Span::styled(status_str, Style::default().fg(Theme::TEXT_DIM)),
        ];
        // Mini token bar for active agents
        if pa.active && state.context_limit > 0 {
            let fill = pa.input_tokens as f64 / state.context_limit as f64;
            let bar_width: usize = 6;
            let filled = (fill * bar_width as f64).round() as usize;
            let empty = bar_width.saturating_sub(filled);
            spans.push(Span::styled(" ", Style::default()));
            if filled > 0 {
                spans.push(Span::styled(
                    "\u{25B8}".repeat(filled),
                    Style::default().fg(accent),
                ));
            }
            if empty > 0 {
                spans.push(Span::styled(
                    "\u{2591}".repeat(empty),
                    Style::default().fg(Theme::TEXT_GHOST),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    if !found_any {
        lines.push(Line::from(Span::styled(
            "  (none active)",
            Style::default().fg(Theme::TEXT_GHOST),
        )));
    }

    let _ = atmosphere; // available for future animated effects
    f.render_widget(Paragraph::new(lines), inner);
}

/// Build a styled status badge span for a plan.
fn plan_status_badge(plan: &crate::state::RunPlanEntry) -> Span<'static> {
    let (label, fg, bg) = match plan.status {
        RunPlanStatus::Active => ("ACTIVE", Theme::BG, Theme::STATUS_ACTIVE),
        RunPlanStatus::Completed => ("DONE", Theme::BG, Theme::STATUS_OK),
        RunPlanStatus::CompletedPrior => ("PRIOR", Theme::BG, Theme::STATUS_OK),
        RunPlanStatus::MergedToMain => ("MERGED", Theme::BG, Theme::STATUS_OK),
        RunPlanStatus::Pending => ("PENDING", Theme::FG_DIM, Theme::BG_SECONDARY),
        RunPlanStatus::Failed => ("FAILED", Theme::BG, Theme::STATUS_ERROR),
        RunPlanStatus::Skipped => ("SKIP", Theme::FG_DIM, Theme::BG_SECONDARY),
    };
    Span::styled(
        format!("[{label}]"),
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
    )
}
