use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::RunState;
use crate::tui::theme::{self, Theme};

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let width = (area.width * 85 / 100).min(120);
    let height = (area.height * 75 / 100).min(40);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(x, y, width, height);

    f.render_widget(Clear, modal_area);

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            " Role        ",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Bck  ",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Plan       ",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Task   ",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Context         ",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Model    ",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Thread",
            Style::default()
                .fg(Theme::BONE)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        " ─".to_string() + &"─".repeat(width as usize - 4),
        Style::default().fg(Theme::TEXT_GHOST),
    )));

    let mut roles: Vec<AgentRole> = state.agents.keys().copied().collect();
    roles.sort_by_key(|r| r.index());

    let gauge_width = 12;

    for &role in &roles {
        let agent = match state.agents.get(&role) {
            Some(a) => a,
            None => continue,
        };

        let accent = Theme::role_accent(role);
        let active_marker = if agent.active { "●" } else { "○" };
        let active_color = if agent.active {
            accent
        } else {
            Theme::TEXT_DIM
        };

        let fill_pct = if state.context_limit > 0 {
            (agent.input_tokens as f64 / state.context_limit as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let pct = (fill_pct * 100.0) as u32;
        let filled = (fill_pct * gauge_width as f64) as usize;
        let empty = gauge_width - filled;
        let bar_color = theme::gradient_context().sample(fill_pct);

        let model = state.config.model_for(role).unwrap_or("?");
        let short_model = shorten_model(model);

        let plan_str = agent.current_plan.as_deref().unwrap_or("-");
        let task_str = agent.current_task.as_deref().unwrap_or("-");
        let thread_str = agent
            .thread_id
            .as_deref()
            .map(|t| t.get(..8).unwrap_or(t))
            .unwrap_or("-");

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {active_marker} "),
                Style::default().fg(active_color),
            ),
            Span::styled(format!("{:10} ", role.label()), Style::default().fg(accent)),
            Span::styled(
                format!("{:4} ", role.backend().short()),
                Style::default().fg(Theme::TEXT_GHOST),
            ),
            Span::styled(
                format!("{:10} ", plan_str),
                Style::default().fg(Theme::FG_DIM),
            ),
            Span::styled(
                format!("{:6} ", task_str),
                Style::default().fg(Theme::FG_DIM),
            ),
            Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
            Span::styled("░".repeat(empty), Style::default().fg(Theme::TEXT_GHOST)),
            Span::styled(format!(" {:>3}% ", pct), Style::default().fg(Theme::FG_DIM)),
            Span::styled(
                format!("{:8} ", short_model),
                Style::default().fg(Theme::DREAM),
            ),
            Span::styled(thread_str.to_string(), Style::default().fg(Theme::TEXT_DIM)),
        ]));
    }

    if state.agents.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No agents spawned",
            Style::default().fg(Theme::TEXT_DIM),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Esc] close",
        Style::default().fg(Theme::FG_DIM),
    )));

    let active_count = state.agents.values().filter(|a| a.active).count();
    let total_count = state.agents.len();
    let title = format!(" Agents ({active_count}/{total_count} active) ");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Theme::BG).fg(Theme::FG))
        .border_style(Style::default().fg(Theme::ROSE))
        .title_style(Style::default().fg(Theme::ROSE));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, modal_area);
}

fn shorten_model(slug: &str) -> String {
    slug.replace("gpt-", "")
        .replace("-codex", "c")
        .replace("-mini", "m")
}
