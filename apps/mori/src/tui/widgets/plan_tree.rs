use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::orchestrator::tasks::TaskStatus;
use crate::state::{RunPlanStatus, RunState, VerifyStatus};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::bars::mini_semantic_bar;
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar;

/// Render the collapsible plan tree: Wave → Plan hierarchy.
/// Falls back to flat list when no waves are configured.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere, focused: bool) {
    let total = state.plans.len();
    let completed = state
        .plans
        .iter()
        .filter(|p| {
            matches!(
                p.status,
                RunPlanStatus::Completed
                    | RunPlanStatus::CompletedPrior
                    | RunPlanStatus::MergedToMain
            )
        })
        .count();

    let title = if focused {
        format!("Plans ({completed}/{total}) [Enter:view ←→:wave]")
    } else {
        format!("Plans ({completed}/{total})")
    };

    let mut lines: Vec<Line> = Vec::new();

    // Filter indicator
    if !state.filter_text.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(" /", Style::default().fg(Theme::DREAM)),
            Span::styled(state.filter_text.clone(), Style::default().fg(Theme::BONE)),
            Span::styled("/ ", Style::default().fg(Theme::DREAM)),
        ]));
    }

    if state.execution_waves.is_empty() {
        // Flat plan list (no waves)
        render_flat_plans(&mut lines, state, atmosphere, focused, area);
    } else {
        // Hierarchical: Wave → Plan tree
        render_wave_tree(&mut lines, state, atmosphere, focused, area);
    }

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
        .title(title)
        .style(Theme::block_style())
        .border_style(border_style)
        .title_style(title_style);

    let visible_height = area.height.saturating_sub(2) as usize;
    let total_lines = lines.len();

    // Scroll to keep selected visible
    let scroll_offset = state
        .plan_scroll_offset
        .min(total_lines.saturating_sub(visible_height));
    let visible: Vec<Line> = lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();

    let paragraph = Paragraph::new(visible).block(block);
    f.render_widget(paragraph, area);

    // Scrollbar
    if total_lines > visible_height {
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        scrollbar::render_scrollbar(
            f.buffer_mut(),
            inner,
            total_lines,
            visible_height,
            scroll_offset,
            Theme::ROSE,
        );
    }
}

fn render_wave_tree(
    lines: &mut Vec<Line<'static>>,
    state: &RunState,
    atmosphere: &Atmosphere,
    focused: bool,
    area: Rect,
) {
    // For each pending (not done, not active) wave, find the wave_num of the most
    // recent earlier wave that still has incomplete plans. Used for "after W{N}" labels.
    let wave_blockers: Vec<Option<usize>> = state
        .execution_waves
        .iter()
        .enumerate()
        .map(|(idx, (_, plans))| {
            let all_done = !plans.is_empty()
                && plans.iter().all(|base| {
                    state.plans.iter().any(|p| {
                        &p.base == base
                            && matches!(
                                p.status,
                                RunPlanStatus::Completed
                                    | RunPlanStatus::CompletedPrior
                                    | RunPlanStatus::MergedToMain
                            )
                    })
                });
            let any_active = plans.iter().any(|base| {
                state
                    .plans
                    .iter()
                    .any(|p| &p.base == base && p.status == RunPlanStatus::Active)
            });
            if !all_done && !any_active && idx > 0 {
                (0..idx)
                    .rev()
                    .find(|&ei| {
                        state
                            .execution_waves
                            .get(ei)
                            .map(|(_, ep)| {
                                ep.is_empty()
                                    || !ep.iter().all(|b| {
                                        state.plans.iter().any(|p| {
                                            &p.base == b
                                                && matches!(
                                                    p.status,
                                                    RunPlanStatus::Completed
                                                        | RunPlanStatus::CompletedPrior
                                                        | RunPlanStatus::MergedToMain
                                                )
                                        })
                                    })
                            })
                            .unwrap_or(false)
                    })
                    .map(|ei| state.execution_waves[ei].0)
            } else {
                None
            }
        })
        .collect();

    for (idx, (wave_num, wave_plans)) in state.execution_waves.iter().enumerate() {
        let collapsed = !state.wave_expanded.contains(&idx);

        // Wave header
        let wave_done = wave_plans
            .iter()
            .filter(|base| {
                state.plans.iter().any(|p| {
                    &p.base == *base
                        && matches!(
                            p.status,
                            RunPlanStatus::Completed
                                | RunPlanStatus::CompletedPrior
                                | RunPlanStatus::MergedToMain
                        )
                })
            })
            .count();
        let wave_total = wave_plans.len();

        let all_done = wave_done == wave_total && wave_total > 0;
        let any_active = wave_plans.iter().any(|base| {
            state
                .plans
                .iter()
                .any(|p| &p.base == base && p.status == RunPlanStatus::Active)
        });

        let (wave_icon, wave_style) = if all_done {
            ("✓", Style::default().fg(Theme::SAGE))
        } else if any_active {
            (
                "►",
                Style::default()
                    .fg(Theme::ROSE)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            ("·", Style::default().fg(Theme::TEXT_GHOST))
        };

        let collapse_icon = if collapsed { "▸" } else { "▾" };

        let mut wave_spans = vec![
            Span::styled(
                format!(" {collapse_icon} "),
                Style::default().fg(Theme::FG_DIM),
            ),
            Span::styled(format!("{wave_icon} "), wave_style),
            Span::styled(
                format!("Wave {} ", wave_num),
                Style::default()
                    .fg(Theme::BONE_DIM)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({wave_done}/{wave_total}) "),
                Style::default().fg(Theme::FG_DIM),
            ),
        ];
        let wave_fill = wave_done as f64 / wave_total.max(1) as f64;
        wave_spans.extend(crate::tui::bars::gradient_bar(
            8,
            wave_fill,
            &crate::tui::color::ocean_gradient(),
            if any_active {
                Some(atmosphere.heartbeat())
            } else {
                None
            },
        ));
        if let Some(blocker_num) = wave_blockers.get(idx).copied().flatten() {
            wave_spans.push(Span::styled(
                format!(" after W{}", blocker_num),
                Style::default().fg(Theme::TEXT_GHOST),
            ));
        }
        lines.push(Line::from(wave_spans));

        if collapsed {
            continue;
        }

        // Plans within wave
        for base in wave_plans {
            if let Some(plan) = state.plans.iter().find(|p| &p.base == base) {
                if matches_filter(plan, &state.filter_text) {
                    render_plan_line(lines, plan, state, atmosphere, focused, area, true);
                }
            }
        }
    }
}

fn render_flat_plans(
    lines: &mut Vec<Line<'static>>,
    state: &RunState,
    atmosphere: &Atmosphere,
    focused: bool,
    area: Rect,
) {
    for plan in &state.plans {
        if matches_filter(plan, &state.filter_text) {
            render_plan_line(lines, plan, state, atmosphere, focused, area, false);
        }
    }
}

fn render_plan_line(
    lines: &mut Vec<Line<'static>>,
    plan: &crate::state::RunPlanEntry,
    state: &RunState,
    _atmosphere: &Atmosphere,
    focused: bool,
    area: Rect,
    indented: bool,
) {
    let i = state
        .plans
        .iter()
        .position(|p| p.base == plan.base)
        .unwrap_or(0);
    let is_selected = i == state.selected_plan_idx && focused;

    let (icon, style) = match plan.status {
        RunPlanStatus::MergedToMain => (
            "\u{2b06}",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        RunPlanStatus::Completed => ("✓", Style::default().fg(Theme::STATUS_OK)),
        RunPlanStatus::CompletedPrior => ("✓", Style::default().fg(Theme::SAGE)),
        RunPlanStatus::Active => (
            "►",
            Style::default()
                .fg(Theme::STATUS_ACTIVE)
                .add_modifier(Modifier::BOLD),
        ),
        RunPlanStatus::Skipped => ("⊘", Style::default().fg(Theme::STATUS_DIM)),
        RunPlanStatus::Failed => ("✗", Style::default().fg(Theme::STATUS_ERROR)),
        RunPlanStatus::Pending => ("○", Style::default().fg(Theme::FG_DIM)),
    };

    // Semantic styling based on plan status
    let text_style = match plan.status {
        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior | RunPlanStatus::MergedToMain => {
            Style::default().fg(Theme::SAGE)
        }
        RunPlanStatus::Active => Style::default()
            .fg(Theme::ROSE_BRIGHT)
            .add_modifier(Modifier::BOLD),
        RunPlanStatus::Failed => Style::default().fg(Theme::EMBER),
        RunPlanStatus::Pending | RunPlanStatus::Skipped => Style::default().fg(Theme::TEXT_DIM),
    };

    let bg = if is_selected {
        Theme::BG_HIGHLIGHT
    } else {
        Theme::BG
    };
    let icon_s = if is_selected { style.bg(bg) } else { style };
    let text_s = if is_selected {
        text_style.bg(bg)
    } else {
        text_style
    };

    let indent = if indented { "   " } else { " " };

    let phase_info = if plan.status == RunPlanStatus::Active {
        format!(" [{}]", plan.phase)
    } else {
        String::new()
    };

    // Per-plan task progress
    let task_progress = if let Some(cl) = state.checklist_for_plan(&plan.base) {
        let done: usize = cl
            .tasks
            .iter()
            .filter(|t| {
                matches!(t.status, TaskStatus::Done)
                    || state
                        .executor_completed_tasks
                        .contains(&format!("{}:{}", plan.base, t.id))
                    || state
                        .executor_completed_tasks
                        .contains(&format!("{}:{}", cl.plan_num, t.id))
            })
            .count();
        let total = cl.total();
        let fill_pct = if total > 0 {
            done as f64 / total as f64
        } else {
            0.0
        };
        Some((done, total, fill_pct))
    } else if matches!(
        plan.status,
        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
    ) {
        Some((1, 1, 1.0))
    } else {
        None
    };

    // Verify indicator
    let verify_span = state
        .verify_entries
        .iter()
        .find(|v| v.plan_base == plan.base && v.plan_num == plan.num)
        .map(|v| {
            let (sym, color) = match &v.status {
                VerifyStatus::Running => ("⟲", Theme::DREAM),
                VerifyStatus::Passed => ("✓v", Theme::SAGE),
                VerifyStatus::Failed(_) => ("✗v", Theme::EMBER),
                VerifyStatus::Pending => ("○v", Theme::TEXT_DIM),
            };
            Span::styled(format!(" {sym}"), Style::default().fg(color).bg(bg))
        });

    // Time info
    let time_span = match plan.status {
        RunPlanStatus::Completed | RunPlanStatus::CompletedPrior => plan.actual_minutes.map(|m| {
            Span::styled(
                format!(" {}", crate::state::format_duration(m as u64 * 60)),
                Style::default().fg(Theme::SAGE).bg(bg),
            )
        }),
        RunPlanStatus::Active => {
            if let Some(started) = plan.started_at {
                let elapsed_secs = started.elapsed().as_secs();
                Some(Span::styled(
                    format!(" {}", crate::state::format_duration(elapsed_secs)),
                    Style::default().fg(Theme::DREAM).bg(bg),
                ))
            } else {
                None
            }
        }
        _ => plan.estimated_minutes.map(|m| {
            Span::styled(
                format!(" ~{}", crate::state::format_duration(m as u64 * 60)),
                Style::default().fg(Theme::TEXT_DIM).bg(bg),
            )
        }),
    };

    let mut spans = vec![
        Span::styled(format!("{indent}{icon} "), icon_s),
        Span::styled(plan.base.clone(), text_s),
        Span::styled(phase_info, Style::default().fg(Theme::FG_DIM).bg(bg)),
    ];

    // Add per-plan task count and mini bar
    if let Some((done, total, fill_pct)) = task_progress {
        let count_color = crate::tui::bars::semantic_color(fill_pct);
        spans.push(Span::styled(
            format!("  {}/{}", done, total),
            Style::default().fg(count_color).bg(bg),
        ));
        if total > 0 && area.width > 80 {
            // Add mini bar if width permits
            let bar_width = (area.width.saturating_sub(70) / 4).max(0).min(10) as usize;
            if bar_width > 0 {
                let bar_spans = mini_semantic_bar(bar_width, fill_pct);
                spans.extend(bar_spans.into_iter().map(|s| {
                    let mut new_s = s.clone();
                    new_s.style = new_s.style.bg(bg);
                    new_s
                }));
            }
        }
    }

    if let Some(ts) = time_span {
        spans.push(ts);
    }
    if let Some(vs) = verify_span {
        spans.push(vs);
    }

    // Git info: branch path, last commit time, dirty indicator
    if let Some(ref branch) = plan.git_branch_short {
        spans.push(Span::styled(
            " │",
            Style::default().fg(Theme::ROSE_DIM).bg(bg),
        ));
        let has_live_data = plan.git_last_commit_secs.is_some() || plan.git_dirty.is_some();
        if has_live_data {
            // Active worktree: show time + dirty, skip redundant branch name
            if let Some(secs) = plan.git_last_commit_secs {
                if secs > 0 {
                    spans.push(Span::styled(
                        format!(" {}", crate::state::format_ago(secs)),
                        Style::default().fg(Theme::TEXT_GHOST).bg(bg),
                    ));
                }
            }
            if let Some((added, removed)) = plan.git_dirty {
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
            // No live worktree: show the branch path so user knows where to look
            spans.push(Span::styled(
                format!(" ⎇ {branch}"),
                Style::default().fg(Theme::TEXT_GHOST).bg(bg),
            ));
        }
    }

    lines.push(Line::from(spans));
}

fn matches_filter(plan: &crate::state::RunPlanEntry, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    let lower = plan.base.to_lowercase();
    let filter_lower = filter.to_lowercase();
    // Simple substring match (fuzzy enough for plan names)
    lower.contains(&filter_lower)
}
