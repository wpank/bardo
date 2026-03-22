use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::RunState;
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;

use super::scrollbar;

pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere, focused: bool) {
    let title = if state.gate_running.is_empty() {
        "Gate Output".to_string()
    } else if state.gate_running.len() == 1 {
        let cmd = state.gate_running.iter().next().unwrap();
        format!("{} {}", atmosphere.spinner(), cmd)
    } else {
        format!(
            "{} {} gates running",
            atmosphere.spinner(),
            state.gate_running.len()
        )
    };

    let visible_height = area.height.saturating_sub(2) as usize;
    let output: &str = if !state.parallel_agents.is_empty() {
        let base = state
            .plans
            .get(state.selected_plan_idx)
            .map(|p| p.base.as_str())
            .unwrap_or("");
        state
            .plan_gate_outputs
            .get(base)
            .map(|s| s.as_str())
            .unwrap_or(&state.command_output)
    } else {
        &state.command_output
    };
    let all_lines: Vec<&str> = output.lines().collect();
    let total = all_lines.len();

    let start = match state.command_output_scroll {
        None => total.saturating_sub(visible_height),
        Some(offset) => offset.min(total.saturating_sub(visible_height.min(total))),
    };
    let end = (start + visible_height).min(total);

    let lines: Vec<Line> = all_lines[start..end]
        .iter()
        .map(|&line| {
            let lower = line.to_lowercase();
            let style =
                if lower.contains("pass") || lower.contains(" ok ") || lower.contains("passed") {
                    Style::default().fg(Theme::SAGE)
                } else if lower.contains("fail") || lower.contains("error") {
                    Style::default().fg(Theme::EMBER)
                } else {
                    Style::default().fg(Theme::FG_DIM)
                };
            Line::from(Span::styled(line, style))
        })
        .collect();

    let is_running = !state.gate_running.is_empty();
    let (border_s, title_s) = if focused {
        (Theme::focused_border_style(), Theme::focused_title_style())
    } else if is_running {
        (
            Style::default().fg(Theme::WARNING),
            Style::default()
                .fg(Theme::WARNING)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
    } else {
        (
            Theme::unfocused_border_style(),
            Theme::unfocused_title_style(),
        )
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(border_s)
        .title_style(title_s);

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
            Theme::WARNING,
        );
    }
}
