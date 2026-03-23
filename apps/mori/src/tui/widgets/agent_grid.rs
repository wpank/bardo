use crate::agent::AgentRole;
use crate::state::RunState;
use crate::tui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render a grid of agent cards showing status at a glance
pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let mut active_agents: Vec<(AgentRole, &crate::state::AgentState)> = state
        .agents
        .iter()
        .filter(|(_, a)| a.active)
        .map(|(r, a)| (*r, a))
        .collect();
    active_agents.sort_by_key(|(r, _)| r.index());

    if active_agents.is_empty() {
        let msg = Paragraph::new("No active agents")
            .style(Theme::dim_style())
            .block(Block::default().borders(Borders::ALL).title("Agents"));
        f.render_widget(msg, area);
        return;
    }

    let count = active_agents.len();
    let (cols, rows) = match count {
        1..=2 => (1, count),
        3..=4 => (2, (count + 1) / 2),
        _ => (3, (count + 2) / 3),
    };

    let row_constraints: Vec<Constraint> = (0..rows)
        .map(|_| Constraint::Ratio(1, rows as u32))
        .collect();
    let col_constraints: Vec<Constraint> = (0..cols)
        .map(|_| Constraint::Ratio(1, cols as u32))
        .collect();

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    let mut agent_idx = 0;
    for row in 0..rows {
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints.clone())
            .split(row_chunks[row]);

        for col in 0..cols {
            if agent_idx >= count {
                break;
            }
            let (role, agent_state) = &active_agents[agent_idx];
            render_agent_card(f, col_chunks[col], *role, agent_state);
            agent_idx += 1;
        }
    }
}

fn render_agent_card(f: &mut Frame, area: Rect, role: AgentRole, state: &crate::state::AgentState) {
    let accent = Theme::role_accent(role);
    let title = format!(" {} ", role);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(title, Style::default().fg(accent)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let context_pct = if state.input_tokens > 0 {
        // Rough estimate: 200k context window
        ((state.input_tokens as f64 / 200_000.0) * 100.0).min(100.0)
    } else {
        0.0
    };

    let w = inner.width as usize;
    let lines = vec![
        Line::from(vec![
            Span::styled("ctx ", Theme::dim_style()),
            Span::styled(format!("{:.0}%", context_pct), Style::default().fg(accent)),
            Span::raw("  "),
            Span::styled(
                format!("{}tok", state.input_tokens / 1000),
                Theme::dim_style(),
            ),
        ]),
        Line::from(vec![Span::styled(
            if let Some(ref task) = state.current_task {
                task.chars().take(w.saturating_sub(2)).collect::<String>()
            } else {
                "idle".to_string()
            },
            Theme::dim_style(),
        )]),
        Line::from(vec![Span::styled(
            state
                .output
                .lines()
                .last()
                .unwrap_or("")
                .chars()
                .take(w.saturating_sub(2))
                .collect::<String>(),
            Style::default().fg(Theme::TEXT_GHOST),
        )]),
    ];

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}
