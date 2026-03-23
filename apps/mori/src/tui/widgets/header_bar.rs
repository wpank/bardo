use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{RunPlanStatus, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::bars::semantic_color;
use crate::tui::theme::Theme;

const HEARTBEAT_FRAMES: [&str; 4] = ["·", "°", "∙", "●"];

/// Render the header bar with wave progress, plan count, ETA, elapsed, and F-key strip.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let bg = Style::default().bg(Theme::BG_SECONDARY);

    let (task_done, task_total) = state.task_weighted_progress();
    let (completed, total) = if task_total > state.plans.len() {
        (task_done, task_total) // task data available
    } else {
        // fall back to plan counts
        let c = state
            .plans
            .iter()
            .filter(|p| {
                matches!(
                    p.status,
                    RunPlanStatus::Completed | RunPlanStatus::CompletedPrior
                )
            })
            .count();
        (c, state.plans.len())
    };
    let elapsed = state.run_started.elapsed();
    let elapsed_str = crate::state::format_duration(elapsed.as_secs());

    let mut spans = vec![Span::styled(" ", bg)];

    // Heartbeat dot — pulses with atmosphere
    let hb_idx = (atmosphere.frame() / 8) as usize % HEARTBEAT_FRAMES.len();
    let hb_brightness = atmosphere.heartbeat();
    let hb_r = (170.0 * hb_brightness).min(255.0) as u8;
    let hb_g = (112.0 * hb_brightness * 0.8).min(255.0) as u8;
    let hb_b = (136.0 * hb_brightness).min(255.0) as u8;
    spans.push(Span::styled(
        HEARTBEAT_FRAMES[hb_idx],
        Style::default()
            .fg(ratatui::style::Color::Rgb(hb_r, hb_g, hb_b))
            .bg(Theme::BG_SECONDARY),
    ));

    // App name
    spans.push(Span::styled(
        " bardo-ctl",
        Style::default()
            .fg(Theme::ROSE)
            .bg(Theme::BG_SECONDARY)
            .add_modifier(Modifier::BOLD),
    ));

    // Wave indicator with accent
    if !state.execution_waves.is_empty() {
        let total_waves = state.execution_waves.len();
        let wave_idx = state.current_wave + 1;
        spans.push(Span::styled(
            format!("  Wave {wave_idx}/{total_waves}"),
            Style::default().fg(Theme::BONE).bg(Theme::BG_SECONDARY),
        ));
    }

    // Progress bar with gradient
    let bar_width = 15usize;
    if total > 0 {
        let fraction = completed as f64 / total.max(1) as f64;
        let filled = (fraction * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar_color = crate::tui::theme::gradient_fire().sample(fraction);

        spans.push(Span::styled("  ", bg));
        if filled > 0 {
            spans.push(Span::styled(
                "█".repeat(filled),
                Style::default().fg(bar_color).bg(Theme::BG_SECONDARY),
            ));
        }
        if empty > 0 {
            spans.push(Span::styled(
                "░".repeat(empty),
                Style::default()
                    .fg(Theme::TEXT_GHOST)
                    .bg(Theme::BG_SECONDARY),
            ));
        }
    }

    // Plan count with semantic coloring
    let fill_pct = if total > 0 {
        completed as f64 / total as f64
    } else {
        0.0
    };
    let progress_text = if state.complete {
        " COMPLETE".to_string()
    } else if let Some(ref err) = state.error {
        format!(" ERR:{}", err.chars().take(16).collect::<String>())
    } else {
        format!("  {completed}/{total}")
    };
    let progress_style = if state.error.is_some() {
        Style::default()
            .fg(Theme::EMBER)
            .add_modifier(Modifier::BOLD)
    } else if state.complete {
        Style::default()
            .fg(Theme::SAGE)
            .add_modifier(Modifier::BOLD)
    } else {
        // Use semantic color based on completion percentage
        Style::default().fg(semantic_color(fill_pct))
    };
    spans.push(Span::styled(
        progress_text,
        progress_style.bg(Theme::BG_SECONDARY),
    ));

    // Percentage with semantic coloring
    if total > 0 && !state.complete && state.error.is_none() {
        let pct = (fill_pct * 100.0) as u32;
        spans.push(Span::styled(
            format!("  {pct}%"),
            Style::default()
                .fg(semantic_color(fill_pct))
                .bg(Theme::BG_SECONDARY),
        ));
    }

    // ETA
    if !state.complete && state.error.is_none() {
        let eta_secs = state.estimated_remaining_seconds();
        let eta_str = crate::state::format_duration(eta_secs.max(1));
        spans.push(Span::styled(
            format!("  ETA:{eta_str}"),
            Style::default().fg(Theme::DREAM).bg(Theme::BG_SECONDARY),
        ));
    }

    // Elapsed
    spans.push(Span::styled(
        format!("  {elapsed_str}"),
        Style::default().fg(Theme::FG_DIM).bg(Theme::BG_SECONDARY),
    ));

    // Cost
    if state.cumulative_cost_usd > 0.001 {
        spans.push(Span::styled(
            format!("  {}", crate::state::fmt_cost(state.cumulative_cost_usd)),
            Style::default().fg(Theme::BONE_DIM).bg(Theme::BG_SECONDARY),
        ));
    }

    // Active agent spinner
    let active_agent = state.agents.iter().find(|(_, s)| s.active).map(|(r, _)| *r);
    if let Some(role) = active_agent {
        let model = state.config.model_for(role).unwrap_or("?");
        let short = shorten_model(model);
        spans.push(Span::styled(
            format!("  {} {}({})", atmosphere.spinner(), role.short(), short),
            Style::default()
                .fg(Theme::role_accent(role))
                .bg(Theme::BG_SECONDARY),
        ));
    }

    // ─── F-key strip (right-aligned via Layout) — all 6 tabs ───
    let current_tab = crate::tui::tabs::Tab::from_index(state.active_tab)
        .unwrap_or(crate::tui::tabs::Tab::Dashboard);
    let fkey_items = vec![
        (" F1", Theme::ROSE, "dash", crate::tui::tabs::Tab::Dashboard),
        (
            " F2",
            Theme::BONE_DIM,
            "plans",
            crate::tui::tabs::Tab::Plans,
        ),
        (" F3", Theme::SAGE, "agents", crate::tui::tabs::Tab::Agents),
        (" F4", Theme::DREAM, "git", crate::tui::tabs::Tab::Git),
        (" F5", Theme::DREAM, "logs", crate::tui::tabs::Tab::Logs),
        (" F6", Theme::BONE_DIM, "cfg", crate::tui::tabs::Tab::Config),
    ];

    let fkey_width: u16 = fkey_items
        .iter()
        .map(|(k, _, l, _)| k.len() + 1 + l.len())
        .sum::<usize>() as u16
        + 1; // trailing space

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(fkey_width)])
        .split(area);

    // Render left content
    let left_line = Line::from(spans);
    f.render_widget(Paragraph::new(left_line).style(bg), chunks[0]);

    // Render F-key indicators with active tab highlighting
    let mut fkey_spans = Vec::new();
    for (key, color, label, tab) in &fkey_items {
        let is_active = *tab == current_tab;
        if is_active {
            // Inverted colors for active tab
            fkey_spans.push(Span::styled(
                format!("{key}:{label}"),
                Style::default()
                    .fg(Theme::VOID)
                    .bg(*color)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            fkey_spans.push(Span::styled(
                key.to_string(),
                Style::default()
                    .fg(*color)
                    .bg(Theme::BG_SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ));
            fkey_spans.push(Span::styled(
                format!(":{label}"),
                Style::default().fg(Theme::FG_DIM).bg(Theme::BG_SECONDARY),
            ));
        }
    }
    fkey_spans.push(Span::styled(" ", bg));

    let fkey_line = Line::from(fkey_spans);
    f.render_widget(Paragraph::new(fkey_line).style(bg), chunks[1]);
}

fn shorten_model(slug: &str) -> String {
    slug.replace("gpt-", "")
        .replace("-codex", "c")
        .replace("-mini", "m")
}
