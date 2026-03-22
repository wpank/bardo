use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::{LogLevel, RunState};
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar;

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = state.log_messages.len();
    let start = total.saturating_sub(visible_height + state.log_scroll);
    let end = total.saturating_sub(state.log_scroll);

    let lines: Vec<Line> = state.log_messages[start..end]
        .iter()
        .map(|entry| {
            let time = entry.timestamp.format("%H:%M:%S").to_string();

            // Level icon with color
            let (icon, icon_color) = match entry.level {
                LogLevel::Error => ("✗", Theme::EMBER),
                LogLevel::Warn => ("⚠", Theme::WARNING),
                LogLevel::Info => ("·", Theme::TEXT),
                LogLevel::Debug => ("‥", Theme::TEXT_DIM),
            };

            // Event detection: append contextual icon
            let event_icon =
                if entry.message.contains("completed") || entry.message.contains("APPROVE") {
                    " ✓"
                } else if entry.message.contains("phase:") || entry.message.contains("entering") {
                    " →"
                } else if entry.message.contains("retry") || entry.message.contains("iteration") {
                    " ⟲"
                } else if entry.message.contains("gate") || entry.message.contains("cargo") {
                    " ⚙"
                } else {
                    ""
                };

            // Source label: color by role accent if matches an agent role, otherwise DREAM
            let source_color = source_accent(&entry.source);

            let msg_style = match entry.level {
                LogLevel::Info => Style::default().fg(Theme::FG_DIM),
                LogLevel::Warn => Style::default().fg(Theme::STATUS_WARN),
                LogLevel::Error => Style::default().fg(Theme::STATUS_ERROR),
                LogLevel::Debug => Style::default().fg(Theme::TEXT_DIM),
            };

            Line::from(vec![
                Span::styled(format!("{time} "), Style::default().fg(Theme::TEXT_PHANTOM)),
                Span::styled(
                    format!("{icon}{event_icon}"),
                    Style::default().fg(icon_color),
                ),
                Span::styled(
                    format!(" [{}] ", entry.source),
                    Style::default().fg(source_color),
                ),
                Span::styled(&entry.message, msg_style),
            ])
        })
        .collect();

    let title = format!("Logs ({total})");
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Theme::block_style())
                .border_style(Theme::unfocused_border_style())
                .title_style(Theme::unfocused_title_style()),
        )
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
            Theme::DREAM,
        );
    }
}

/// Map a log source label to a role accent color if it matches an agent role.
fn source_accent(source: &str) -> ratatui::style::Color {
    let lower = source.to_lowercase();
    let role_map: &[(&str, AgentRole)] = &[
        ("conductor", AgentRole::Conductor),
        ("cond", AgentRole::Conductor),
        ("strategist", AgentRole::Strategist),
        ("strat", AgentRole::Strategist),
        ("implementer", AgentRole::Implementer),
        ("impl", AgentRole::Implementer),
        ("architect", AgentRole::Architect),
        ("arch", AgentRole::Architect),
        ("auditor", AgentRole::Auditor),
        ("audit", AgentRole::Auditor),
        ("scribe", AgentRole::Scribe),
        ("critic", AgentRole::Critic),
        ("crit", AgentRole::Critic),
    ];
    for (label, role) in role_map {
        if lower.contains(label) {
            return Theme::role_accent(*role);
        }
    }
    Theme::DREAM
}
