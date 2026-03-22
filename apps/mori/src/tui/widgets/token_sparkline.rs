use std::collections::VecDeque;

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::RunState;
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;
use crate::tui::widgets::braille;

/// Render a multi-row token burn sparkline with braille glyphs.
/// Top row: aggregate cumulative tokens (braille). Per-agent rows below (braille).
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let inner_height = area.height.saturating_sub(2) as usize;
    if inner_width < 10 || inner_height < 2 {
        return;
    }

    // Aggregate cumulative tokens across all agents
    let mut combined: Vec<u64> = Vec::new();
    for (_role, history) in &state.token_burn_history {
        if combined.is_empty() {
            combined = history.iter().copied().collect();
        } else {
            for (i, &val) in history.iter().enumerate() {
                if i < combined.len() {
                    combined[i] = combined[i].saturating_add(val);
                }
            }
        }
    }

    if combined.len() < 2 {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Token Burn")
            .style(Theme::block_style())
            .border_style(Style::default().fg(Theme::TEXT_GHOST))
            .title_style(Theme::title_style());
        let p = Paragraph::new(Line::from(Span::styled(
            format!(" {} waiting for data...", atmosphere.spinner()),
            Style::default().fg(Theme::TEXT_DIM),
        )))
        .block(block);
        f.render_widget(p, area);
        return;
    }

    // Current total and burn rate
    let current_total = *combined.last().unwrap_or(&0);
    let total_str = fmt_tokens(current_total);

    let window = 30.min(combined.len());
    let recent_start = combined.len() - window;
    let delta = combined
        .last()
        .unwrap_or(&0)
        .saturating_sub(combined[recent_start]);
    let rate_per_min = delta as f64 * 60.0 / (window as f64 / 30.0);
    let rate_str = if rate_per_min > 1_000_000.0 {
        format!("{:.1}M/min", rate_per_min / 1_000_000.0)
    } else if rate_per_min > 1_000.0 {
        format!("{:.1}k/min", rate_per_min / 1_000.0)
    } else if rate_per_min > 0.5 {
        format!("{:.0}/min", rate_per_min)
    } else {
        "idle".to_string()
    };

    let rate_color = if rate_per_min > 100_000.0 {
        Theme::EMBER
    } else if rate_per_min > 10_000.0 {
        Theme::WARNING
    } else if rate_per_min > 0.5 {
        Theme::ROSE
    } else {
        Theme::TEXT_DIM
    };

    let mut lines: Vec<Line> = Vec::new();

    // Aggregate braille row
    let label_len = total_str.len() + 2;
    let rate_len = rate_str.len() + 2;
    let spark_w = inner_width.saturating_sub(label_len + rate_len);

    let display: Vec<u64> = if combined.len() > spark_w * 2 {
        combined[combined.len() - spark_w * 2..].to_vec()
    } else {
        combined.clone()
    };

    let mut spans: Vec<Span> = vec![Span::styled(
        format!(" {total_str} "),
        Style::default().fg(Theme::BONE_DIM),
    )];
    spans.extend(braille::braille_spans_u64(&display, spark_w, Theme::ROSE));
    spans.push(Span::styled(
        format!(" {rate_str} "),
        Style::default().fg(rate_color),
    ));
    lines.push(Line::from(spans));

    // Per-agent braille rows
    let mut roles: Vec<AgentRole> = state.token_burn_history.keys().copied().collect();
    roles.sort_by_key(|r| r.index());
    let remaining_rows = inner_height.saturating_sub(lines.len());
    for role in roles.into_iter().take(remaining_rows) {
        if let Some(history) = state.token_burn_history.get(&role) {
            if history.len() < 2 {
                continue;
            }
            let accent = Theme::role_accent(role);
            let label = format!(" {:5} ", role.short());
            let agent_spark_w = inner_width.saturating_sub(label.len());

            let values: Vec<u64> = history.iter().copied().collect();
            let mut agent_spans = vec![Span::styled(label, Style::default().fg(accent))];
            agent_spans.extend(braille::braille_spans_u64(&values, agent_spark_w, accent));
            lines.push(Line::from(agent_spans));
        }
    }

    let border_color = if rate_per_min > 0.5 {
        Theme::ROSE_DIM
    } else {
        Theme::TEXT_GHOST
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Token Burn")
        .style(Theme::block_style())
        .border_style(Style::default().fg(border_color))
        .title_style(Theme::title_style());

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn fmt_tokens(n: u64) -> String {
    if n == 0 {
        "0".to_string()
    } else if n < 1_000 {
        format!("{n}")
    } else if n < 10_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else if n < 1_000_000 {
        format!("{}k", n / 1_000)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

/// Render a compact inline sparkline (no border). Returns spans.
pub fn inline_sparkline(history: &VecDeque<u64>, width: usize) -> Vec<Span<'static>> {
    if history.len() < 2 {
        return vec![Span::styled(
            " ".repeat(width),
            Style::default().fg(Theme::TEXT_GHOST),
        )];
    }

    let values: Vec<u64> = history.iter().copied().collect();
    braille::braille_spans_u64(&values, width, Theme::ROSE_DIM)
}
