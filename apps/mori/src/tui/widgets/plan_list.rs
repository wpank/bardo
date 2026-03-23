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

pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere, focused: bool) {
    let total = state.plans.len();
    let completed = state
        .plans
        .iter()
        .filter(|p| {
            matches!(
                p.status,
                RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
            )
        })
        .count();
    let failed = state
        .plans
        .iter()
        .filter(|p| p.status == RunPlanStatus::Failed)
        .count();

    let eta_secs = state.estimated_remaining_seconds();
    let title = if eta_secs > 0 && !state.complete {
        if focused {
            format!(
                "Plans ({completed}/{total}) ETA:{} [Enter:view D:reset]",
                crate::state::format_duration(eta_secs)
            )
        } else {
            format!(
                "Plans ({completed}/{total}) ETA:{}",
                crate::state::format_duration(eta_secs)
            )
        }
    } else {
        if focused {
            format!("Plans ({completed}/{total}) [Enter:view D:reset]")
        } else {
            format!("Plans ({completed}/{total})")
        }
    };

    let inner_width = area.width.saturating_sub(4) as usize;
    // Available height for plan items (subtract borders=2, progress bar=1)
    let visible_plan_slots = area.height.saturating_sub(3) as usize;

    let mut lines: Vec<Line> = Vec::new();

    // Animated progress bar with fractional blocks and percentage
    if inner_width > 8 && total > 0 {
        let fraction = completed as f64 / total.max(1) as f64;
        let bar_width = inner_width.saturating_sub(6);
        let failed_w = if failed > 0 {
            (failed * bar_width) / total.max(1)
        } else {
            0
        };
        let effective_bar_width = bar_width.saturating_sub(failed_w);

        let mut bar_spans = vec![Span::styled(" ", Style::default())];
        bar_spans.extend(crate::tui::bars::gradient_bar(
            effective_bar_width,
            fraction,
            &crate::tui::color::fire_gradient(),
            Some(atmosphere.heartbeat()),
        ));
        if failed_w > 0 {
            bar_spans.push(Span::styled(
                "█".repeat(failed_w),
                Style::default().fg(Theme::EMBER),
            ));
        }
        let pct = (fraction * 100.0) as u32;
        bar_spans.push(Span::styled(
            format!(" {pct}%"),
            Style::default().fg(Theme::FG_DIM),
        ));
        lines.push(Line::from(bar_spans));
    }

    // Use scroll offset from state (maintained by SelectPlanUp/Down handlers)
    let scroll_offset = if total > visible_plan_slots {
        state
            .plan_scroll_offset
            .min(total.saturating_sub(visible_plan_slots))
    } else {
        0
    };

    // Show scroll-up indicator
    if scroll_offset > 0 {
        lines.push(Line::from(Span::styled(
            format!(" ▲ {} more", scroll_offset),
            Style::default().fg(Theme::FG_DIM),
        )));
    }

    // Plan items (windowed)
    let visible_end = (scroll_offset + visible_plan_slots).min(total);
    // Adjust visible slots if we added a scroll indicator
    let plan_range_start = scroll_offset;
    let plan_range_end = if scroll_offset > 0 {
        // One line used by the ▲ indicator
        (scroll_offset + visible_plan_slots.saturating_sub(1)).min(total)
    } else {
        visible_end
    };

    for i in plan_range_start..plan_range_end {
        let plan = &state.plans[i];
        let is_selected = i == state.selected_plan_idx && focused;

        let (icon, style) = match plan.status {
            RunPlanStatus::Completed => ("✓", Style::default().fg(Theme::STATUS_OK)),
            RunPlanStatus::CompletedPrior => ("✓", Style::default().fg(Theme::SAGE)),
            RunPlanStatus::MergedToMain => ("✓", Style::default().fg(Theme::STATUS_OK)),
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
            RunPlanStatus::Completed
            | RunPlanStatus::CompletedPrior
            | RunPlanStatus::MergedToMain => Style::default().fg(Theme::SAGE),
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

        let phase_info = if plan.status == RunPlanStatus::Active {
            format!(" [{}]", plan.phase)
        } else {
            String::new()
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

        // Time estimate/actual
        let time_span = match plan.status {
            RunPlanStatus::Completed | RunPlanStatus::CompletedPrior => {
                plan.actual_minutes.map(|m| {
                    Span::styled(
                        format!(" {}", crate::state::format_duration(m as u64 * 60)),
                        Style::default().fg(Theme::SAGE).bg(bg),
                    )
                })
            }
            RunPlanStatus::Active => {
                if let Some(started) = plan.started_at {
                    let elapsed_secs = started.elapsed().as_secs();
                    let est = plan.estimated_minutes.unwrap_or(0);
                    if est > 0 {
                        Some(Span::styled(
                            format!(
                                " {}/~{}",
                                crate::state::format_duration(elapsed_secs),
                                crate::state::format_duration(est as u64 * 60)
                            ),
                            Style::default().fg(Theme::DREAM).bg(bg),
                        ))
                    } else {
                        Some(Span::styled(
                            format!(" {}", crate::state::format_duration(elapsed_secs)),
                            Style::default().fg(Theme::DREAM).bg(bg),
                        ))
                    }
                } else {
                    plan.estimated_minutes.map(|m| {
                        Span::styled(
                            format!(" ~{}", crate::state::format_duration(m as u64 * 60)),
                            Style::default().fg(Theme::DREAM).bg(bg),
                        )
                    })
                }
            }
            _ => plan.estimated_minutes.map(|m| {
                Span::styled(
                    format!(" ~{}", crate::state::format_duration(m as u64 * 60)),
                    Style::default().fg(Theme::TEXT_DIM).bg(bg),
                )
            }),
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

        let mut plan_spans = vec![
            Span::styled(format!(" {icon} "), icon_s),
            Span::styled(plan.base.clone(), text_s),
            Span::styled(phase_info, Style::default().fg(Theme::FG_DIM).bg(bg)),
        ];

        // Add per-plan task count and mini bar
        if let Some((done, total, fill_pct)) = task_progress {
            let count_color = crate::tui::bars::semantic_color(fill_pct);
            plan_spans.push(Span::styled(
                format!("  {}/{}", done, total),
                Style::default().fg(count_color).bg(bg),
            ));
            if total > 0 && area.width > 80 {
                // Add mini bar if width permits
                let bar_width = (area.width.saturating_sub(70) / 4).max(0).min(10) as usize;
                if bar_width > 0 {
                    let bar_spans = mini_semantic_bar(bar_width, fill_pct);
                    plan_spans.extend(bar_spans.into_iter().map(|s| {
                        let mut new_s = s.clone();
                        new_s.style = new_s.style.bg(bg);
                        new_s
                    }));
                }
            }
        }

        if let Some(ts) = time_span {
            plan_spans.push(ts);
        }
        if let Some(vs) = verify_span {
            plan_spans.push(vs);
        }

        // Git info: branch path, last commit time, dirty indicator
        if let Some(ref branch) = plan.git_branch_short {
            plan_spans.push(Span::styled(
                " │",
                Style::default().fg(Theme::ROSE_DIM).bg(bg),
            ));
            let has_live_data = plan.git_last_commit_secs.is_some() || plan.git_dirty.is_some();
            if has_live_data {
                if let Some(secs) = plan.git_last_commit_secs {
                    if secs > 0 {
                        plan_spans.push(Span::styled(
                            format!(" {}", crate::state::format_ago(secs)),
                            Style::default().fg(Theme::TEXT_GHOST).bg(bg),
                        ));
                    }
                }
                if let Some((added, removed)) = plan.git_dirty {
                    plan_spans.push(Span::styled(
                        format!(" +{added}"),
                        Style::default().fg(Theme::SAGE).bg(bg),
                    ));
                    plan_spans.push(Span::styled(
                        format!("-{removed}"),
                        Style::default().fg(Theme::EMBER).bg(bg),
                    ));
                }
            } else {
                plan_spans.push(Span::styled(
                    format!(" ⎇ {branch}"),
                    Style::default().fg(Theme::TEXT_GHOST).bg(bg),
                ));
            }
        }

        lines.push(Line::from(plan_spans));
    }

    // Show scroll-down indicator
    if plan_range_end < total {
        lines.push(Line::from(Span::styled(
            format!(" ▼ {} more", total - plan_range_end),
            Style::default().fg(Theme::FG_DIM),
        )));
    }

    let border_color = if focused { Theme::ROSE } else { Theme::FG_DIM };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(Style::default().fg(border_color))
        .title_style(Theme::title_style());

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
