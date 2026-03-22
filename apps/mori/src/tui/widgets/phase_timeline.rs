use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::RunState;
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;

/// Phase order for timeline display.
const PHASES: &[&str] = &[
    "preflight",
    "strategist",
    "implementer",
    "compile-gate",
    "test-gate",
    "reviewing",
    "critic-review",
    "verdict",
    "doc-revision",
    "committing",
];

/// Short labels for the timeline.
fn short_label(phase: &str) -> &str {
    match phase {
        "preflight" => "pre",
        "strategist" => "strat",
        "implementer" => "impl",
        "compile-gate" => "cc",
        "test-gate" => "test",
        "reviewing" => "rev",
        "critic-review" => "crit",
        "verdict" => "verd",
        "doc-revision" => "doc",
        "committing" => "comm",
        _ => phase,
    }
}

/// Render a horizontal phase timeline with proportional widths.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let width = area.width as usize;
    if width < 20 {
        return;
    }

    let selected_base = state
        .plans
        .get(state.selected_plan_idx)
        .map(|p| p.base.clone())
        .unwrap_or_default();
    let current_phase: &str = if !state.parallel_agents.is_empty() {
        state
            .plans
            .get(state.selected_plan_idx)
            .map(|p| p.phase.as_str())
            .unwrap_or(&state.current_phase)
    } else {
        &state.current_phase
    };
    let current_idx = PHASES
        .iter()
        .position(|&p| p == current_phase)
        .unwrap_or(PHASES.len());

    // Gather durations: completed phases from phase_elapsed, current from phase_started
    let mut durations: Vec<(&str, f64, bool, bool)> = Vec::new(); // (phase, secs, is_done, is_current)
    for (idx, &phase) in PHASES.iter().enumerate() {
        // Skip doc-revision if not used
        if phase == "doc-revision"
            && state.doc_revision_count == 0
            && current_phase != "doc-revision"
        {
            continue;
        }

        let is_current = idx == current_idx;
        let is_done = idx < current_idx;

        let secs = if is_done {
            state
                .phase_elapsed
                .iter()
                .find(|(name, _)| name == phase)
                .map(|(_, d)| d.as_secs_f64())
                .unwrap_or(0.0)
        } else if is_current {
            let phase_started = if !state.parallel_agents.is_empty() {
                state
                    .plan_phase_started
                    .get(selected_base.as_str())
                    .copied()
            } else {
                state.phase_started
            };
            phase_started
                .map(|t| t.elapsed().as_secs_f64())
                .unwrap_or(0.0)
        } else {
            // Future: use estimate
            state
                .time_estimator
                .phase_averages
                .get(phase)
                .copied()
                .unwrap_or(2.0)
                * 60.0
        };

        durations.push((phase, secs.max(1.0), is_done, is_current));
    }

    // Proportional widths
    let total_secs: f64 = durations.iter().map(|(_, s, _, _)| s).sum();
    let label_budget = durations.len() * 2; // minimum 2 chars per phase
    let bar_budget = width.saturating_sub(label_budget);

    let mut spans: Vec<Span> = Vec::new();
    for (phase, secs, is_done, is_current) in &durations {
        let label = short_label(phase);
        let segment_width = ((secs / total_secs) * bar_budget as f64).ceil() as usize;
        let segment_width = segment_width.max(label.len() + 1);

        let (icon, style) = if *is_current {
            (
                "►",
                Style::default()
                    .fg(Theme::phase_accent(phase))
                    .add_modifier(Modifier::BOLD),
            )
        } else if *is_done {
            ("✓", Style::default().fg(Theme::SAGE))
        } else {
            ("·", Style::default().fg(Theme::TEXT_GHOST))
        };

        let content = format!("{icon}{label}");
        let pad = segment_width.saturating_sub(content.len());

        spans.push(Span::styled(content, style));
        if pad > 0 {
            if *is_done {
                // Gradient brightness across done segment: full → half
                let phase_color = Theme::SAGE;
                for j in 0..pad {
                    let t = j as f64 / pad.max(1) as f64;
                    let dimmed = crate::tui::theme::brighten(phase_color, 1.0 - t * 0.5);
                    spans.push(Span::styled("─", Style::default().fg(dimmed)));
                }
            } else if *is_current {
                // Breathing pulse on active segment
                let pulse = atmosphere.breathing_brightness();
                let phase_color = Theme::phase_accent(phase);
                let pulsed = crate::tui::theme::brighten(phase_color, pulse);
                spans.push(Span::styled("━".repeat(pad), Style::default().fg(pulsed)));
            } else {
                spans.push(Span::styled(
                    "┄".repeat(pad),
                    Style::default().fg(Theme::TEXT_GHOST),
                ));
            }
        }
    }

    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}
