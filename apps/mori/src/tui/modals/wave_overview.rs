use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::{RunPlanStatus, RunState};
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let width = (area.width * 80 / 100).min(100);
    let height = (area.height * 70 / 100).min(40);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(x, y, width, height);

    f.render_widget(Clear, modal_area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " Wave Execution Overview",
        Style::default()
            .fg(Theme::BONE)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if state.execution_waves.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Sequential execution — no wave structure",
            Style::default().fg(Theme::TEXT_DIM),
        )));
    } else {
        for (i, (_wave_num, plans)) in state.execution_waves.iter().enumerate() {
            let is_current = i == state.current_wave;
            let wave_done = plans
                .iter()
                .filter(|base| {
                    state.plans.iter().any(|p| {
                        &p.base == *base
                            && matches!(
                                p.status,
                                RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
                            )
                    })
                })
                .count();
            let wave_total = plans.len();
            let all_done = wave_done == wave_total;

            let icon = if all_done {
                "✓"
            } else if is_current {
                "►"
            } else {
                "○"
            };
            let color = if all_done {
                Theme::SAGE
            } else if is_current {
                Theme::ROSE
            } else {
                Theme::TEXT_DIM
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {icon} Wave {} ", i + 1),
                    Style::default().fg(color).add_modifier(if is_current {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
                Span::styled(
                    format!("({wave_done}/{wave_total})"),
                    Style::default().fg(Theme::FG_DIM),
                ),
            ]));

            // Plan details within wave
            for base in plans {
                let plan = state.plans.iter().find(|p| &p.base == base);
                let (plan_icon, plan_color) = match plan.map(|p| &p.status) {
                    Some(RunPlanStatus::Completed) | Some(RunPlanStatus::CompletedPrior) => {
                        ("✓", Theme::SAGE)
                    }
                    Some(RunPlanStatus::Active) => ("►", Theme::ROSE),
                    Some(RunPlanStatus::Failed) => ("✗", Theme::EMBER),
                    Some(RunPlanStatus::Skipped) => ("⊘", Theme::TEXT_DIM),
                    _ => ("○", Theme::TEXT_GHOST),
                };

                let time_info = plan
                    .and_then(|p| {
                        if matches!(
                            p.status,
                            RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
                        ) {
                            p.actual_minutes
                                .map(|m| crate::state::format_duration(m as u64 * 60))
                        } else if matches!(p.status, RunPlanStatus::Active) {
                            p.started_at
                                .map(|s| crate::state::format_duration(s.elapsed().as_secs()))
                        } else {
                            p.estimated_minutes.map(|m| {
                                format!("~{}", crate::state::format_duration(m as u64 * 60))
                            })
                        }
                    })
                    .unwrap_or_default();

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("      {plan_icon} "),
                        Style::default().fg(plan_color),
                    ),
                    Span::styled(base.clone(), Style::default().fg(plan_color)),
                    Span::styled(format!(" {time_info}"), Style::default().fg(Theme::FG_DIM)),
                ]));
            }
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Esc] close",
        Style::default().fg(Theme::FG_DIM),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Waves ")
        .style(Style::default().bg(Theme::BG).fg(Theme::FG))
        .border_style(Style::default().fg(Theme::DREAM))
        .title_style(Style::default().fg(Theme::DREAM));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, modal_area);
}
