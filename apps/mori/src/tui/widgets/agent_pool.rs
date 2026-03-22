use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::RunState;
use crate::tui::atmosphere::Atmosphere;
use crate::tui::bars;
use crate::tui::color;
use crate::tui::theme::Theme;

/// Format a token count as compact string: 0, 1.2k, 45k, 120k, 1.2M
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

/// Render active agents summary with inline context gauges.
/// Agents matching the selected plan are shown first; others are dimmed.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let mut roles: Vec<AgentRole> = state.agents.keys().copied().collect();
    roles.sort_by_key(|r| r.index());

    // Determine the selected plan base for filtering
    let selected_plan_base = state
        .plans
        .get(state.selected_plan_idx)
        .map(|p| p.base.as_str());

    // Partition: agents on the selected plan first, then others
    let is_on_selected_plan = |agent: &crate::state::AgentState| -> bool {
        match (&agent.current_plan, selected_plan_base) {
            (Some(ap), Some(sp)) => ap.contains(sp) || sp.contains(ap.as_str()),
            _ => false,
        }
    };

    let gauge_width = 11usize;
    let mut lines: Vec<Line> = Vec::new();
    let mut other_lines: Vec<Line> = Vec::new();

    // Available content width inside the bordered block (border = 1 on each side)
    let content_width = area.width.saturating_sub(2) as usize;

    for &role in &roles {
        let agent = match state.agents.get(&role) {
            Some(a) if a.active || a.input_tokens > 0 => a,
            _ => continue,
        };

        let on_selected = is_on_selected_plan(agent);
        let accent = if on_selected {
            Theme::role_accent(role)
        } else {
            Theme::FG_DIM
        };
        let fill_pct = if state.context_limit > 0 {
            (agent.input_tokens as f64 / state.context_limit as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let model_slug = state.config.model_for(role).unwrap_or("?");
        let short_model = shorten_model(model_slug);

        // Gauge bar (per-cell gradient)
        let bar_spans = bars::gradient_bar(
            gauge_width,
            fill_pct,
            &color::context_gradient(),
            if agent.active {
                Some(atmosphere.breathing_brightness())
            } else {
                None
            },
        );

        // Token counts: "12k/200k"
        let token_str = format!(
            "{}/{}",
            fmt_tokens(agent.input_tokens),
            fmt_tokens(state.context_limit),
        );

        // Role label
        let name_label = {
            let plan_task = match (&agent.current_plan, &agent.current_task) {
                (Some(p), Some(t)) => format!("{}:{}", p, t),
                (Some(p), None) => p.clone(),
                _ => String::new(),
            };
            if plan_task.is_empty() {
                format!(" {:9}", role.short())
            } else {
                format!(" {}:{}", role.short(), plan_task)
            }
        };

        let max_label = 16;
        let label = if name_label.chars().count() > max_label {
            crate::tui::truncate_chars(&name_label, max_label, "..")
        } else {
            format!("{:width$}", name_label, width = max_label)
        };

        // Fixed-width prefix:  label(16) + space(1) + gauge(11) + space(1) + tokens(var) + space(1) + model(var) + "  "
        let prefix_len =
            label.len() + 1 + gauge_width + 1 + token_str.len() + 1 + short_model.len() + 2;
        let snippet_budget = content_width.saturating_sub(prefix_len);

        // Status snippet from last non-empty line of output
        let snippet: String = agent
            .output
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .map(|l| {
                let trimmed = l.trim();
                if snippet_budget < 4 {
                    String::new()
                } else if trimmed.len() > snippet_budget {
                    crate::tui::truncate_chars(trimmed, snippet_budget, "...")
                } else {
                    trimmed.to_string()
                }
            })
            .unwrap_or_else(|| {
                if agent.active {
                    "working...".to_string()
                } else {
                    "idle".to_string()
                }
            });

        let mut line_spans = vec![
            Span::styled(label, Style::default().fg(accent)),
            Span::styled(" ", Style::default()),
        ];
        line_spans.extend(bar_spans);
        let snippet_color = if on_selected {
            Theme::TEXT_DIM
        } else {
            Theme::TEXT_GHOST
        };
        line_spans.push(Span::styled(
            format!(" {}", token_str),
            Style::default().fg(Theme::FG_DIM),
        ));
        line_spans.push(Span::styled(
            format!(" {}", short_model),
            Style::default().fg(if on_selected {
                Theme::DREAM
            } else {
                Theme::TEXT_GHOST
            }),
        ));
        line_spans.push(Span::styled(
            format!("  {}", snippet),
            Style::default().fg(snippet_color),
        ));
        if on_selected {
            lines.push(Line::from(line_spans));
        } else {
            other_lines.push(Line::from(line_spans));
        }
    }

    // Append non-selected-plan agents after the selected ones
    if !other_lines.is_empty() && !lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " ── other plans ──",
            Style::default().fg(Theme::TEXT_GHOST),
        )));
    }
    lines.extend(other_lines);

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " No active agents",
            Style::default().fg(Theme::TEXT_DIM),
        )));
    }

    let active_count = if !state.parallel_agents.is_empty() {
        state.parallel_agents.iter().filter(|p| p.active).count()
    } else {
        state.agents.values().filter(|a| a.active).count()
    };
    let plan_label = selected_plan_base.unwrap_or("?");
    let title = format!("Agents \u{00b7} {} ({} active)", plan_label, active_count);
    let border_color = if active_count > 0 {
        Theme::ROSE_DIM
    } else {
        Theme::TEXT_PHANTOM
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(Style::default().fg(border_color))
        .title_style(if active_count > 0 {
            Style::default()
                .fg(Theme::ROSE)
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Theme::title_style()
        });

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn shorten_model(slug: &str) -> String {
    slug.replace("gpt-", "")
        .replace("-codex", "c")
        .replace("-mini", "m")
}
