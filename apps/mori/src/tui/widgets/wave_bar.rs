use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::{RunPlanStatus, RunState};
use crate::tui::bars::semantic_bar;
use crate::tui::theme::{self, Theme};

/// Render the wave progress bar showing current execution wave status.
pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    if state.execution_waves.is_empty() {
        return;
    }

    let wave_idx = state.current_wave;
    let _total_waves = state.execution_waves.len();

    // Count completed plans in current wave
    let (_wave_num, wave_plans) = state
        .execution_waves
        .get(wave_idx)
        .map(|(n, p)| (*n, p.as_slice()))
        .unwrap_or((0, &[]));

    let (task_done, task_total) = state.wave_task_progress(wave_idx);
    let (wave_done, wave_total) = if task_total > wave_plans.len() {
        (task_done, task_total)
    } else {
        let done = wave_plans
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
        (done, wave_plans.len())
    };

    // Wave progress bar
    let bar_width = (area.width as usize).saturating_sub(36);
    let fill_pct = if wave_total > 0 {
        wave_done as f64 / wave_total as f64
    } else {
        0.0
    };

    // ETA for current wave
    let wave_plan_bases: Vec<String> = wave_plans.to_vec();
    let eta_mins = state
        .time_estimator
        .estimated_wave_minutes(&wave_plan_bases);
    let eta_str = crate::state::format_duration(eta_mins as u64 * 60);

    // Build title line with semantic bar
    let mut title_spans = vec![Span::styled(
        format!(" Wave {} ", wave_idx + 1),
        Style::default().fg(Theme::BONE),
    )];
    title_spans.extend(semantic_bar(bar_width, fill_pct, None));

    // Add count and percentage with semantic color
    let count_color = crate::tui::bars::semantic_color(fill_pct);
    let pct = (fill_pct * 100.0) as u32;
    let count_text = if task_total > wave_plans.len() {
        format!("  {}/{} tasks  {}%", wave_done, wave_total, pct)
    } else {
        format!("  {}/{} plans  {}%", wave_done, wave_total, pct)
    };
    title_spans.push(Span::styled(count_text, Style::default().fg(count_color)));

    title_spans.push(Span::styled(
        format!("  ETA:{}", eta_str),
        Style::default().fg(Theme::DREAM),
    ));

    let title_line = Line::from(title_spans);

    // Compact plan status line
    let mut plan_spans: Vec<Span> = vec![Span::styled(" ", Style::default())];
    for base in wave_plans {
        let plan = state.plans.iter().find(|p| &p.base == base);
        let (icon, color) = match plan.map(|p| &p.status) {
            Some(RunPlanStatus::MergedToMain) => ("\u{2b06}", Theme::BONE), // ⬆
            Some(RunPlanStatus::Completed) | Some(RunPlanStatus::CompletedPrior) => {
                ("✓", Theme::SAGE)
            }
            Some(RunPlanStatus::Active) => ("►", Theme::ROSE),
            Some(RunPlanStatus::Failed) => ("✗", Theme::EMBER),
            Some(RunPlanStatus::Skipped) => ("⊘", Theme::TEXT_DIM),
            _ => ("○", Theme::TEXT_GHOST),
        };

        // Short name: take last component after '-'
        let short = base.rsplit('-').next().unwrap_or(base);
        let elapsed_str = plan
            .and_then(|p| {
                p.started_at.map(|s| {
                    let secs = s.elapsed().as_secs();
                    crate::state::format_duration(secs)
                })
            })
            .unwrap_or_default();

        plan_spans.push(Span::styled(format!("{icon} "), Style::default().fg(color)));
        plan_spans.push(Span::styled(format!("{short}"), Style::default().fg(color)));
        if !elapsed_str.is_empty() {
            plan_spans.push(Span::styled(
                format!(" {elapsed_str}"),
                Style::default().fg(Theme::FG_DIM),
            ));
        }
        plan_spans.push(Span::styled("  ", Style::default()));
    }

    let lines = vec![title_line, Line::from(plan_spans)];

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Theme::block_style())
        .border_style(Style::default().fg(Theme::FG_DIM));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
