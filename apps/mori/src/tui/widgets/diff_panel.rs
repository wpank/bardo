use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::RunState;
use crate::tui::theme::Theme;

use super::scrollbar;

const TAB_ROLES: &[AgentRole] = &[
    AgentRole::Strategist,
    AgentRole::Implementer,
    AgentRole::Architect,
    AgentRole::Auditor,
    AgentRole::Scribe,
    AgentRole::Critic,
    AgentRole::Conductor,
];

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let active_role = TAB_ROLES
        .get(state.selected_agent_tab)
        .copied()
        .unwrap_or(AgentRole::Implementer);

    // Use agent diff if available, otherwise fall back to branch diff
    let agent_diff = state
        .agent_state(active_role)
        .map(|a| a.diff.as_str())
        .unwrap_or("");

    let diff_text = if agent_diff.is_empty() {
        state.branch_diff.as_str()
    } else {
        agent_diff
    };

    let visible_height = area.height.saturating_sub(2) as usize;
    let diff_lines: Vec<&str> = diff_text.lines().collect();
    let total = diff_lines.len();

    // Scroll logic: auto-scroll or pinned
    let start = match state.diff_scroll {
        None => total.saturating_sub(visible_height),
        Some(offset) => offset.min(total.saturating_sub(visible_height.min(total))),
    };
    let end = (start + visible_height).min(total);

    let mut lines: Vec<Line> = diff_lines[start..end]
        .iter()
        .map(|&line| {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                Style::default().fg(Theme::SAGE)
            } else if line.starts_with('-') && !line.starts_with("---") {
                Style::default().fg(Theme::EMBER)
            } else if line.starts_with("@@") {
                Style::default().fg(Theme::DREAM)
            } else {
                Style::default().fg(Theme::FG_DIM)
            };
            Line::from(Span::styled(line, style))
        })
        .collect();

    // Scroll indicator
    if state.diff_scroll.is_some() && start > 0 {
        if let Some(first) = lines.first_mut() {
            *first = Line::from(Span::styled(
                format!("▲ {} lines above", start),
                Style::default().fg(Theme::FG_DIM),
            ));
        }
    }

    let source = if agent_diff.is_empty() {
        "branch"
    } else {
        active_role.short()
    };
    let title = format!("Diff ({}) [{} lines]", source, total);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(Theme::unfocused_border_style())
        .title_style(Theme::unfocused_title_style());

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    // Scrollbar
    if total > visible_height {
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        scrollbar::render_scrollbar(
            f.buffer_mut(),
            inner,
            total,
            visible_height,
            start,
            Theme::SAGE,
        );
    }
}
