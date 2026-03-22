use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::{ParallelAgentState, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::bars::gradient_bar;
use crate::tui::color::{context_gradient, Gradient};
use crate::tui::theme::Theme;

/// Render a scrollable table of all parallel agent instances.
///
/// Each row shows:  `role:plan:task  [token bar]  Ntk/200k  pct%  last-line`
/// Filters to the currently selected plan when one is selected.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let mut filtered: Vec<(usize, &ParallelAgentState)> =
        state.parallel_agents.iter().enumerate().collect();
    // Active agents first, then by label alphabetically
    filtered.sort_by_key(|(_, p)| (!p.active, p.role.short().to_string(), p.task.clone()));

    let active_count = filtered.iter().filter(|(_, p)| p.active).count();
    let title = format!(" Agents ({} active) ", active_count);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Theme::unfocused_border_style())
        .title_style(Theme::unfocused_title_style())
        .style(Theme::block_style());

    let inner = block.inner(area);
    let inner_w = inner.width as usize;

    let gradient = context_gradient();
    let breathing: Option<f64> = Some(atmosphere.breathing_brightness());

    let mut lines: Vec<Line> = Vec::new();

    for (idx, agent) in &filtered {
        let selected = *idx == state.agent_list_cursor;
        lines.push(build_row(
            agent, state, inner_w, &gradient, breathing, selected, atmosphere,
        ));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  no parallel agents spawned",
            Style::default().fg(Theme::TEXT_DIM),
        )));
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn build_row(
    agent: &ParallelAgentState,
    state: &RunState,
    width: usize,
    gradient: &Gradient,
    breathing: Option<f64>,
    selected: bool,
    atmosphere: &Atmosphere,
) -> Line<'static> {
    let accent = Theme::role_accent(agent.role);

    // Label: "impl[5.4]:01-ws:T1" style
    let model_tag = shorten_model(&agent.model);
    let label = if model_tag.is_empty() {
        format!(
            "{}:{:.8}:{:.6}",
            agent.role.short(),
            shorten(&agent.plan),
            agent.task
        )
    } else {
        format!(
            "{}[{}]:{:.8}:{:.6}",
            agent.role.short(),
            model_tag,
            shorten(&agent.plan),
            agent.task
        )
    };
    let label_padded = format!("{:<24}", label);

    // Token bar — Cursor ACP doesn't expose token counts, show placeholder
    let is_cursor_model = agent.model.starts_with("composer-")
        || agent.model.starts_with("cursor-")
        || agent.model == "auto"
        || agent.model.starts_with("sonnet-")
        || agent.model.starts_with("opus-")
        || agent.model.starts_with("haiku-");

    let total_tokens = agent.input_tokens + agent.output_tokens;
    let context_limit = state.context_limit.max(1);
    let pct = (total_tokens as f64 / context_limit as f64).min(1.0);
    let pct_pct = (pct * 100.0) as u64;

    let bar_w = 10usize;
    let bar_spans = if is_cursor_model {
        gradient_bar(bar_w, 0.0, gradient, None)
    } else {
        gradient_bar(bar_w, pct, gradient, breathing)
    };

    let token_str = if is_cursor_model {
        format!("  {:>14}  ", "—")
    } else {
        format!(
            "  {:>5}k/{:>3}k  {:>3}%  ",
            total_tokens / 1000,
            context_limit / 1000,
            pct_pct
        )
    };

    // Last line of output
    let last_line = agent
        .output
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    let max_tail = width
        .saturating_sub(label.len().max(24))
        .saturating_sub(bar_w)
        .saturating_sub(token_str.len())
        .saturating_sub(4);
    let tail = truncate(last_line, max_tail);

    let status_span = if agent.active {
        Span::styled(
            format!("{} ", atmosphere.spinner()),
            Style::default().fg(Theme::SAGE),
        )
    } else {
        Span::styled("✓ ", Style::default().fg(Theme::TEXT_DIM))
    };

    let row_modifier = if selected {
        Modifier::REVERSED
    } else {
        Modifier::empty()
    };

    let mut spans = vec![
        Span::styled(
            label_padded,
            Style::default().fg(accent).add_modifier(if agent.active {
                Modifier::BOLD | row_modifier
            } else {
                row_modifier
            }),
        ),
        Span::styled(
            token_str,
            Style::default()
                .fg(Theme::TEXT_DIM)
                .add_modifier(row_modifier),
        ),
    ];
    spans.extend(bar_spans);
    spans.push(Span::raw("  "));
    spans.push(status_span);
    spans.push(Span::styled(
        tail,
        Style::default()
            .fg(Theme::TEXT_DIM)
            .add_modifier(row_modifier),
    ));

    Line::from(spans)
}

fn shorten(plan: &str) -> String {
    // "01-workspace-scaffold" → "01-ws"
    let mut parts = plan.splitn(2, '-');
    let num = parts.next().unwrap_or(plan);
    let rest = parts.next().unwrap_or("");
    if rest.is_empty() {
        plan.to_string()
    } else {
        let abbrev: String = rest
            .split('-')
            .map(|w| w.chars().next().unwrap_or('?'))
            .collect();
        format!("{num}-{abbrev}")
    }
}

fn shorten_model(model: &str) -> String {
    if model.is_empty() {
        return String::new();
    }
    // "gpt-5.4" → "5.4", "claude-opus-4-6" → "o4.6", "claude-sonnet-4-6" → "s4.6",
    // "claude-haiku-4-5" → "h4.5", "cursor-small" → "sm"
    let m = model.to_lowercase();
    if let Some(rest) = m.strip_prefix("gpt-") {
        return rest.to_string();
    }
    if m.contains("opus") {
        let ver: String = m
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        return format!("o{ver}");
    }
    if m.contains("sonnet") {
        let ver: String = m
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        return format!("s{ver}");
    }
    if m.contains("haiku") {
        let ver: String = m
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        return format!("h{ver}");
    }
    if m.contains("cursor-small") || m == "small" {
        return "sm".to_string();
    }
    // Fallback: first 6 chars
    model.chars().take(6).collect()
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.len() <= max {
        s.to_string()
    } else {
        let end = (0..=max.saturating_sub(1).min(s.len()))
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0);
        format!("{}…", &s[..end])
    }
}
