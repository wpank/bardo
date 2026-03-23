use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::{FocusZone, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::bars::gradient_bar;
use crate::tui::color::context_gradient;
use crate::tui::theme::Theme;
use crate::tui::widgets;
use crate::tui::widgets::status_badge::{badge, Status};

/// Full agent-centric view: scrollable agent list on the left, detail pane on the right.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    // Left ~30% | 1-cell spacer | Right ~70%
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(1),
            Constraint::Percentage(70),
        ])
        .split(area);

    render_left_panel(f, columns[0], state, atmosphere);
    // columns[1] is the spacer — leave it empty
    render_right_panel(f, columns[2], state, atmosphere);
}

// ---------------------------------------------------------------------------
// Left panel: agent list + summary + token sparkline
// ---------------------------------------------------------------------------

fn render_left_panel(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    // In parallel mode use the per-instance pool widget instead of the role list.
    if !state.parallel_agents.is_empty() {
        crate::tui::widgets::parallel_pool::render(f, area, state, atmosphere);
        return;
    }

    let has_burn_data = !state.token_burn_history.is_empty();

    // Vertical split: agent list (flex) | summary (2 lines) | sparkline (if data)
    let sparkline_height = if has_burn_data { 6 } else { 0 };
    let constraints = if has_burn_data {
        vec![
            Constraint::Min(4),
            Constraint::Length(2),
            Constraint::Length(sparkline_height),
        ]
    } else {
        vec![Constraint::Min(4), Constraint::Length(2)]
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // --- Agent list ---
    let mut agents: Vec<(AgentRole, &crate::state::AgentState)> = state
        .agents
        .iter()
        .filter(|(_, a)| a.active || a.input_tokens > 0)
        .map(|(r, a)| (*r, a))
        .collect();
    agents.sort_by_key(|(r, a)| (!a.active, r.index()));

    let focused = state.focus == FocusZone::AgentOutput;

    let border_style = if focused {
        Theme::focused_border_style()
    } else {
        Theme::unfocused_border_style()
    };
    let title_style = if focused {
        Theme::focused_title_style()
    } else {
        Theme::unfocused_title_style()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Agents ")
        .border_style(border_style)
        .title_style(title_style)
        .style(Theme::block_style());

    let inner_height = sections[0].height.saturating_sub(2) as usize;

    // Build list lines
    let mut list_lines: Vec<Line> = Vec::with_capacity(agents.len());
    for (i, (role, agent)) in agents.iter().enumerate() {
        let selected = i == state.agent_list_cursor;

        // Role short name, padded to 6 chars
        let short = format!("{:6}", role.short());
        let backend_tag = format!("{} ", role.backend().short());

        // Plan:task assignment
        let assignment = match (&agent.current_plan, &agent.current_task) {
            (Some(plan), Some(task)) => format!("{}:{}", abbreviate_plan(plan), task),
            (Some(plan), None) => abbreviate_plan(plan),
            _ => "idle".to_string(),
        };

        // Status badge
        let status = if agent.active {
            badge("active", Status::Active)
        } else if agent.input_tokens > 0 {
            badge("done", Status::Done)
        } else {
            badge("idle", Status::Idle)
        };

        let cursor_char = if selected { "► " } else { "  " };
        let accent = Theme::role_accent(*role);

        let line_style = if selected {
            Theme::selected_style()
        } else {
            Style::default()
        };

        let mut spans = vec![
            Span::styled(cursor_char, Style::default().fg(Theme::ROSE)),
            Span::styled(
                short,
                Style::default().fg(accent).add_modifier(if agent.active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ),
            Span::styled(backend_tag, Style::default().fg(Theme::TEXT_GHOST)),
            Span::styled(
                format!(" {:<16} ", truncate(&assignment, 16)),
                Style::default().fg(if agent.active {
                    Theme::TEXT
                } else {
                    Theme::TEXT_DIM
                }),
            ),
            status,
        ];

        // Active spinner
        if agent.active {
            spans.push(Span::styled(
                format!(" {}", atmosphere.spinner()),
                Style::default().fg(accent),
            ));
        }

        let mut line = Line::from(spans);
        line.style = line_style;
        list_lines.push(line);
    }

    // Handle scrolling if list is taller than visible area
    let total = list_lines.len();
    let scroll_offset = state.agent_scroll.unwrap_or(0);
    let start = if state.agent_list_cursor >= scroll_offset + inner_height {
        state
            .agent_list_cursor
            .saturating_sub(inner_height.saturating_sub(1))
    } else if state.agent_list_cursor < scroll_offset {
        state.agent_list_cursor
    } else {
        scroll_offset
    };
    let end = (start + inner_height).min(total);
    let start = start.min(end);
    let visible: Vec<Line> = if total > 0 {
        list_lines[start..end].to_vec()
    } else {
        vec![Line::from(Span::styled(
            "  no agents spawned",
            Style::default().fg(Theme::TEXT_DIM),
        ))]
    };

    let paragraph = Paragraph::new(visible).block(block);
    f.render_widget(paragraph, sections[0]);

    // --- Summary line ---
    let active_count = agents.iter().filter(|(_, a)| a.active).count();
    let total_count = agents.len();
    let total_input: u64 = state.agents.values().map(|a| a.input_tokens).sum();
    let total_output: u64 = state.agents.values().map(|a| a.output_tokens).sum();

    let mut summary_spans = vec![
        Span::styled(
            format!("  Active: {} of {}  ", active_count, total_count),
            Style::default().fg(Theme::TEXT_DIM),
        ),
        Span::styled(
            format!(
                "Tokens: {}/{}",
                fmt_tokens(total_input),
                fmt_tokens(total_output)
            ),
            Style::default().fg(Theme::BONE_DIM),
        ),
    ];
    if state.cumulative_cost_usd > 0.001 {
        summary_spans.push(Span::styled(
            format!("  {}", crate::state::fmt_cost(state.cumulative_cost_usd)),
            Style::default().fg(Theme::DREAM),
        ));
    }
    let summary = Line::from(summary_spans);

    let summary_p = Paragraph::new(vec![Line::from(""), summary]).style(Theme::block_style());
    f.render_widget(summary_p, sections[1]);

    // --- Token sparkline ---
    if has_burn_data && sections.len() > 2 {
        widgets::token_sparkline::render(f, sections[2], state, atmosphere);
    }
}

// ---------------------------------------------------------------------------
// Right panel: agent detail header + output widget
// ---------------------------------------------------------------------------

fn render_right_panel(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let focused = state.focus == FocusZone::AgentOutput;
    let border_style = if focused {
        Theme::focused_border_style()
    } else {
        Theme::unfocused_border_style()
    };
    let title_style = if focused {
        Theme::focused_title_style()
    } else {
        Theme::unfocused_title_style()
    };

    // Parallel mode: show selected parallel agent's detail + output
    if !state.parallel_agents.is_empty() {
        if let Some(pa) = state.parallel_agents.get(state.agent_list_cursor) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Min(4)])
                .split(area);
            render_parallel_detail_header(f, chunks[0], pa, state, atmosphere, border_style);
            render_parallel_agent_output(
                f,
                chunks[1],
                pa,
                state,
                focused,
                border_style,
                title_style,
            );
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Agent Detail ")
                .border_style(border_style)
                .title_style(title_style)
                .style(Theme::block_style());
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "  No agent selected",
                    Style::default().fg(Theme::TEXT_DIM),
                )))
                .block(block),
                area,
            );
        }
        return;
    }

    // Sequential mode: find the selected agent from the role-based list
    let mut agents: Vec<(AgentRole, &crate::state::AgentState)> = state
        .agents
        .iter()
        .filter(|(_, a)| a.active || a.input_tokens > 0)
        .map(|(r, a)| (*r, a))
        .collect();
    agents.sort_by_key(|(r, a)| (!a.active, r.index()));

    let selected = agents.get(state.agent_list_cursor);

    if let Some(&(role, agent)) = selected {
        let accent = Theme::role_accent(role);

        // Vertical: header info (5 lines) | agent output (rest)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(4)])
            .split(area);

        // --- Header block ---
        render_detail_header(
            f,
            chunks[0],
            role,
            agent,
            state,
            atmosphere,
            accent,
            border_style,
            title_style,
        );

        // --- Agent output (delegated to widget) ---
        widgets::agent_output::render(f, chunks[1], state, atmosphere, focused);
    } else {
        // No agent selected — empty detail pane
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Agent Detail ")
            .border_style(border_style)
            .title_style(title_style)
            .style(Theme::block_style());

        let empty = Paragraph::new(Line::from(Span::styled(
            "  No agent selected",
            Style::default().fg(Theme::TEXT_DIM),
        )))
        .block(block);

        f.render_widget(empty, area);
    }
}

fn render_parallel_detail_header(
    f: &mut Frame,
    area: Rect,
    pa: &crate::state::ParallelAgentState,
    state: &RunState,
    atmosphere: &Atmosphere,
    border_style: Style,
) {
    let accent = Theme::role_accent(pa.role);
    let title = if pa.active {
        format!(" {} {} ", atmosphere.spinner(), pa.role.label())
    } else {
        format!(" {} ", pa.role.label())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
        .title_style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
        .style(Theme::block_style());

    let inner = block.inner(area);

    let line1 = Line::from(vec![
        Span::styled("Plan: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(pa.plan.clone(), Style::default().fg(Theme::BONE)),
        Span::styled("    Task: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(pa.task.clone(), Style::default().fg(Theme::BONE)),
    ]);

    let instance_str = if pa.instance_id.chars().count() > 20 {
        format!("{}...", pa.instance_id.chars().take(20).collect::<String>())
    } else {
        pa.instance_id.clone()
    };
    let line2 = Line::from(vec![
        Span::styled("Instance: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(instance_str, Style::default().fg(Theme::DREAM)),
    ]);

    let total_tokens = pa.input_tokens + pa.output_tokens;
    let context_limit = state.context_limit;
    let pct = if context_limit > 0 {
        total_tokens as f64 / context_limit as f64
    } else {
        0.0
    };
    let pct_display = (pct * 100.0).min(100.0) as u64;

    let label = format!(
        "Tokens: {}/{}  ",
        fmt_tokens(total_tokens),
        fmt_tokens(context_limit)
    );
    let pct_suffix = format!(" {}%", pct_display);
    let gauge_width = (inner.width as usize)
        .saturating_sub(label.len())
        .saturating_sub(pct_suffix.len());
    let gradient = crate::tui::color::context_gradient();
    let breathing = Some(atmosphere.breathing_brightness());
    let bar_spans = crate::tui::bars::gradient_bar(gauge_width, pct, &gradient, breathing);
    let mut line3_spans = vec![Span::styled(label, Style::default().fg(Theme::TEXT_DIM))];
    line3_spans.extend(bar_spans);
    line3_spans.push(Span::styled(
        pct_suffix,
        Style::default().fg(if pct > 0.8 {
            Theme::EMBER
        } else if pct > 0.5 {
            Theme::WARNING
        } else {
            Theme::SAGE
        }),
    ));
    let line3 = Line::from(line3_spans);

    let mut line4_spans = vec![
        Span::styled("  In: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(
            fmt_tokens(pa.input_tokens),
            Style::default().fg(Theme::ROSE_DIM),
        ),
        Span::styled("  Out: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(
            fmt_tokens(pa.output_tokens),
            Style::default().fg(Theme::DREAM),
        ),
    ];
    if pa.cost_usd > 0.001 {
        line4_spans.push(Span::styled(
            format!("  Cost: {}", crate::state::fmt_cost(pa.cost_usd)),
            Style::default().fg(Theme::BONE_DIM),
        ));
    }
    let line4 = Line::from(line4_spans);

    let paragraph = Paragraph::new(vec![line1, line2, line3, line4])
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_parallel_agent_output(
    f: &mut Frame,
    area: Rect,
    pa: &crate::state::ParallelAgentState,
    state: &RunState,
    _focused: bool,
    border_style: Style,
    title_style: Style,
) {
    let accent = Theme::role_accent(pa.role);
    let output = pa.output.as_str();
    let title = format!(" {} output ", pa.role.label());

    let styled_lines: Vec<Line> = if output.is_empty() {
        vec![Line::from(Span::styled(
            "  waiting...",
            Style::default().fg(Theme::TEXT_DIM),
        ))]
    } else {
        output
            .lines()
            .map(|line| {
                let lower = line.to_lowercase();
                let style = if lower.contains("error") || lower.contains("fail") {
                    Style::default().fg(Theme::EMBER)
                } else if lower.contains("✓") || lower.contains("pass") || lower.contains("ok ") {
                    Style::default().fg(Theme::SAGE)
                } else if line.trim_start().starts_with("# ")
                    || line.trim_start().starts_with("## ")
                {
                    Style::default()
                        .fg(Theme::BONE)
                        .add_modifier(Modifier::BOLD)
                } else if line.trim_start().starts_with("▸ ") || line.trim_start().starts_with("> ")
                {
                    Style::default().fg(Theme::DREAM)
                } else {
                    Style::default().fg(Theme::FG_DIM)
                };
                Line::from(Span::styled(line.to_string(), style))
            })
            .collect()
    };

    let visible_height = area.height.saturating_sub(2) as usize;
    let total = styled_lines.len();
    let start = match state.agent_scroll {
        None => total.saturating_sub(visible_height),
        Some(offset) => offset.min(total.saturating_sub(visible_height.min(total))),
    };
    let end = (start + visible_height).min(total);

    let mut visible: Vec<Line> = styled_lines[start..end].to_vec();
    if state.agent_scroll.is_some() && start > 0 {
        if let Some(first) = visible.first_mut() {
            *first = Line::from(Span::styled(
                format!("▲ {} lines above", start),
                Style::default().fg(Theme::FG_DIM),
            ));
        }
    }
    if state.agent_scroll.is_some() && end < total {
        visible.push(Line::from(Span::styled(
            "[End] to resume auto-scroll",
            Style::default().fg(Theme::FG_DIM),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(border_style)
        .title_style(title_style);

    let paragraph = Paragraph::new(visible)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    if total > visible_height {
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        widgets::scrollbar::render_scrollbar(
            f.buffer_mut(),
            inner,
            total,
            visible_height,
            start,
            accent,
        );
    }
}

fn render_detail_header(
    f: &mut Frame,
    area: Rect,
    role: AgentRole,
    agent: &crate::state::AgentState,
    state: &RunState,
    atmosphere: &Atmosphere,
    accent: ratatui::style::Color,
    border_style: Style,
    _title_style: Style,
) {
    let title = if agent.active {
        format!(" {} {} ", atmosphere.spinner(), role.label())
    } else {
        format!(" {} ", role.label())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
        .title_style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
        .style(Theme::block_style());

    let inner = block.inner(area);

    // Line 1: Plan and Task
    let plan_str = agent.current_plan.as_deref().unwrap_or("none");
    let task_str = agent.current_task.as_deref().unwrap_or("none");

    let line1 = Line::from(vec![
        Span::styled("Plan: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(plan_str.to_string(), Style::default().fg(Theme::BONE)),
        Span::styled("    Task: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(task_str.to_string(), Style::default().fg(Theme::BONE)),
    ]);

    // Line 2: Turns, thread
    let thread_str = agent
        .thread_id
        .as_deref()
        .map(|t| {
            if t.chars().count() > 12 {
                format!("{}...", t.chars().take(12).collect::<String>())
            } else {
                t.to_string()
            }
        })
        .unwrap_or_else(|| "—".to_string());

    let line2 = Line::from(vec![
        Span::styled("Turns: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(
            format!("{}", agent.turn_count),
            Style::default().fg(Theme::TEXT),
        ),
        Span::styled("    Thread: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(thread_str, Style::default().fg(Theme::DREAM)),
    ]);

    // Line 3: Token usage with gauge bar
    let total_tokens = agent.input_tokens + agent.output_tokens;
    let context_limit = state.context_limit;
    let pct = if context_limit > 0 {
        total_tokens as f64 / context_limit as f64
    } else {
        0.0
    };
    let pct_display = (pct * 100.0).min(100.0) as u64;

    // Calculate gauge bar width: fill the remaining space
    let label = format!(
        "Tokens: {}/{}  ",
        fmt_tokens(total_tokens),
        fmt_tokens(context_limit),
    );
    let pct_suffix = format!(" {}%", pct_display);
    let gauge_width = (inner.width as usize)
        .saturating_sub(label.len())
        .saturating_sub(pct_suffix.len());

    let gradient = context_gradient();
    let breathing = Some(atmosphere.breathing_brightness());
    let bar_spans = gradient_bar(gauge_width, pct, &gradient, breathing);

    let mut line3_spans = vec![Span::styled(label, Style::default().fg(Theme::TEXT_DIM))];
    line3_spans.extend(bar_spans);
    line3_spans.push(Span::styled(
        pct_suffix,
        Style::default().fg(if pct > 0.8 {
            Theme::EMBER
        } else if pct > 0.5 {
            Theme::WARNING
        } else {
            Theme::SAGE
        }),
    ));
    let line3 = Line::from(line3_spans);

    // Line 4: Input/output token breakdown + cost
    let mut line4_spans = vec![
        Span::styled("  In: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(
            fmt_tokens(agent.input_tokens),
            Style::default().fg(Theme::ROSE_DIM),
        ),
        Span::styled("  Out: ", Style::default().fg(Theme::TEXT_DIM)),
        Span::styled(
            fmt_tokens(agent.output_tokens),
            Style::default().fg(Theme::DREAM),
        ),
        Span::styled(
            format!("  Iter: {}", state.current_iteration),
            Style::default().fg(Theme::TEXT_DIM),
        ),
    ];
    if state.cumulative_cost_usd > 0.001 {
        line4_spans.push(Span::styled(
            format!(
                "  Cost: {}",
                crate::state::fmt_cost(state.cumulative_cost_usd)
            ),
            Style::default().fg(Theme::BONE_DIM),
        ));
    }
    let line4 = Line::from(line4_spans);

    let lines = vec![line1, line2, line3, line4];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format token count compactly: 0, 123, 4.5k, 45k, 1.2M
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

/// Abbreviate plan name: "02-core-types" -> "02-types"
fn abbreviate_plan(plan: &str) -> String {
    // Keep the number prefix and last segment if hyphenated
    if let Some((num, rest)) = plan.split_once('-') {
        let parts: Vec<&str> = rest.split('-').collect();
        if parts.len() > 1 {
            format!("{}-{}", num, parts.last().unwrap_or(&rest))
        } else {
            plan.to_string()
        }
    } else {
        plan.to_string()
    }
}

/// Truncate a string to fit within `max` bytes, adding ellipsis if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 1 {
        let end = (0..=max.saturating_sub(1).min(s.len()))
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0);
        format!("{}…", &s[..end])
    } else {
        let end = (0..=max.min(s.len()))
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0);
        s[..end].to_string()
    }
}
