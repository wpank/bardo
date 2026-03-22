use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::{RunPlanStatus, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;

/// The phases shown in the phase bar for the current plan
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

pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere, focused: bool) {
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

    let plan_completed = state
        .plans
        .get(state.selected_plan_idx)
        .map(|p| {
            matches!(
                p.status,
                RunPlanStatus::Completed
                    | RunPlanStatus::CompletedPrior
                    | RunPlanStatus::MergedToMain
            )
        })
        .unwrap_or(false);

    let title = if !state.parallel_agents.is_empty() {
        format!("Phase · {selected_base}")
    } else {
        format!("Phase (iter {})", state.current_iteration)
    };

    let mut lines: Vec<Line> = PHASES
        .iter()
        .enumerate()
        .flat_map(|(idx, &phase)| {
            let mut result = Vec::new();

            // Separator between test-gate (4) and reviewing (5)
            if idx == 5 {
                result.push(Line::from(Span::styled(
                    "  ─────",
                    Style::default().fg(Theme::TEXT_GHOST),
                )));
            }

            // Hide doc-revision phase unless it's active or was used
            if phase == "doc-revision"
                && state.doc_revision_count == 0
                && current_phase != "doc-revision"
            {
                return result;
            }

            let is_current = phase == current_phase;
            let is_done = plan_completed || phase_index(phase) < phase_index(current_phase);

            let (icon, style) = if is_current {
                let hb = if phase == "compile-gate" || phase == "test-gate" {
                    // Shimmer: faster, brighter oscillation for gates
                    1.0 + 0.10 * (atmosphere.frame() as f64 / 8.0).sin()
                } else {
                    atmosphere.heartbeat()
                };
                let pulse_color = pulse_active(hb);
                (
                    format!("{}", atmosphere.spinner_ethereal()),
                    Style::default()
                        .fg(pulse_color)
                        .add_modifier(Modifier::BOLD),
                )
            } else if is_done {
                ("✓".to_string(), Style::default().fg(Theme::SAGE))
            } else {
                ("○".to_string(), Style::default().fg(Theme::TEXT_GHOST))
            };

            // Elapsed time: for done phases, look up in phase_elapsed; for current, show live
            let phase_started = if !state.parallel_agents.is_empty() {
                state
                    .plan_phase_started
                    .get(selected_base.as_str())
                    .copied()
            } else {
                state.phase_started
            };
            let time_str = if is_current {
                phase_started
                    .map(|t| {
                        let d = t.elapsed();
                        format!("{}m{:02}s", d.as_secs() / 60, d.as_secs() % 60)
                    })
                    .unwrap_or_default()
            } else if is_done {
                state
                    .phase_elapsed
                    .iter()
                    .find(|(name, _)| name == phase)
                    .map(|(_, d)| format!("{}m{:02}s", d.as_secs() / 60, d.as_secs() % 60))
                    .unwrap_or_default()
            } else {
                // Future phase: show estimate if available
                state
                    .time_estimator
                    .phase_averages
                    .get(phase)
                    .map(|avg| format!("~{:.0}m", avg))
                    .unwrap_or_default()
            };

            // Phase percentage for active agent phases
            let pct_str = if is_current {
                phase_percentage(state, phase)
            } else {
                String::new()
            };

            // Pulsing timer color for active phase
            let time_style = if is_current {
                let pulse = atmosphere.heartbeat();
                let base_r = 170.0_f64;
                let r = (base_r * pulse).clamp(0.0, 255.0) as u8;
                Style::default().fg(Color::Rgb(r, 112, 136))
            } else {
                Style::default().fg(Theme::FG_DIM)
            };

            result.push(Line::from(vec![
                Span::styled(format!(" {icon} "), style),
                Span::styled(phase, style),
                Span::styled(format!(" {pct_str}"), Style::default().fg(Theme::DREAM)),
                Span::styled(format!(" {time_str}"), time_style),
            ]));

            // Verdict breakdown sub-line after verdict phase
            if phase == "verdict"
                && is_done
                && (!state.code_verdict.is_empty() || !state.doc_verdict.is_empty())
            {
                let code_color = if state.code_verdict == "APPROVE" {
                    Theme::SAGE
                } else {
                    Theme::EMBER
                };
                let mut verdict_spans = vec![
                    Span::styled("     code: ", Style::default().fg(Theme::FG_DIM)),
                    Span::styled(&state.code_verdict, Style::default().fg(code_color)),
                ];
                if !state.doc_verdict.is_empty() {
                    let doc_color = if state.doc_verdict == "APPROVE" {
                        Theme::SAGE
                    } else {
                        Theme::EMBER
                    };
                    verdict_spans
                        .push(Span::styled("  docs: ", Style::default().fg(Theme::FG_DIM)));
                    verdict_spans.push(Span::styled(
                        &state.doc_verdict,
                        Style::default().fg(doc_color),
                    ));
                }
                result.push(Line::from(verdict_spans));
            }

            result
        })
        .collect();

    // Plan ETA: estimated time remaining for the current plan
    if let Some(remaining) = state.estimated_current_plan_remaining() {
        if remaining > 0 {
            lines.push(Line::from(Span::styled(
                format!(
                    "  ETA: ~{}",
                    crate::state::format_duration(remaining as u64 * 60)
                ),
                Style::default().fg(Theme::DREAM),
            )));
        }
    }

    // Show error/halt state or iteration reason
    if let Some(ref err) = state.error {
        lines.push(Line::from(""));
        let truncated = crate::tui::truncate_chars(err, 43, "...");
        lines.push(Line::from(Span::styled(
            format!("  HALTED: {truncated}"),
            Style::default().fg(Theme::EMBER),
        )));
        lines.push(Line::from(Span::styled(
            "  [R:restart X:force D:reset]",
            Style::default().fg(Theme::WARNING),
        )));
    } else if state.current_iteration > 1 && !state.iteration_reason.is_empty() {
        lines.push(Line::from(""));
        let truncated = crate::tui::truncate_chars(&state.iteration_reason, 48, "...");
        lines.push(Line::from(Span::styled(
            format!("  iter {}: {truncated}", state.current_iteration),
            Style::default().fg(Theme::WARNING),
        )));
    }

    let (border_s, ttl_s) = if focused {
        (Theme::focused_border_style(), Theme::focused_title_style())
    } else {
        (Style::default().fg(Theme::FG_DIM), Theme::title_style())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(border_s)
        .title_style(ttl_s);

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn phase_index(phase: &str) -> usize {
    PHASES
        .iter()
        .position(|&p| p == phase)
        .unwrap_or(usize::MAX)
}

/// Modulate STATUS_ACTIVE brightness with heartbeat
fn pulse_active(heartbeat: f64) -> Color {
    let base_r = 170.0;
    let base_g = 112.0;
    let base_b = 136.0;
    let scale = heartbeat.clamp(0.9, 1.1);
    Color::Rgb(
        (base_r * scale).min(255.0) as u8,
        (base_g * scale).min(255.0) as u8,
        (base_b * scale).min(255.0) as u8,
    )
}

/// Estimate phase completion percentage
fn phase_percentage(state: &RunState, phase: &str) -> String {
    use crate::agent::AgentRole;

    match phase {
        "strategist" => token_pct(state, AgentRole::Strategist),
        "implementer" => token_pct(state, AgentRole::Implementer),
        "reviewing" => {
            // Average of active reviewers
            let roles = [AgentRole::Architect, AgentRole::Auditor, AgentRole::Scribe];
            let active: Vec<_> = roles
                .iter()
                .filter(|&&r| {
                    state
                        .agent_state(r)
                        .map(|a| a.input_tokens > 0)
                        .unwrap_or(false)
                })
                .collect();
            if active.is_empty() {
                return String::new();
            }
            let avg_pct: f64 = active
                .iter()
                .map(|&&r| {
                    let tokens = state.agent_state(r).map(|a| a.input_tokens).unwrap_or(0);
                    (tokens as f64 / state.context_limit as f64 * 100.0).min(99.0)
                })
                .sum::<f64>()
                / active.len() as f64;
            format!("{:.0}%", avg_pct)
        }
        "critic-review" | "doc-revision" => token_pct(state, AgentRole::Critic),
        "compile-gate" => elapsed_pct(state, 30.0),
        "test-gate" => elapsed_pct(state, 120.0),
        _ => String::new(),
    }
}

fn token_pct(state: &RunState, role: crate::agent::AgentRole) -> String {
    if let Some(agent) = state.agent_state(role) {
        if agent.input_tokens > 0 {
            let pct = (agent.input_tokens as f64 / state.context_limit as f64 * 100.0).min(99.0);
            return format!("{:.0}%", pct);
        }
    }
    String::new()
}

fn elapsed_pct(state: &RunState, typical_secs: f64) -> String {
    if let Some(started) = state.phase_started {
        let elapsed = started.elapsed().as_secs_f64();
        let pct = (elapsed / typical_secs * 100.0).min(99.0);
        format!("{:.0}%", pct)
    } else {
        String::new()
    }
}
