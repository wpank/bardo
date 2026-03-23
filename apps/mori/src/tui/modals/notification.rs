use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::LogLevel;
use crate::tui::theme::Theme;

/// Render a toast notification anchored to the bottom-right corner.
/// `stack_index` offsets vertically to stack multiple toasts.
pub fn render_toast(
    f: &mut Frame,
    area: Rect,
    message: &str,
    level: &LogLevel,
    stack_index: usize,
) {
    let width = 44u16.min(area.width.saturating_sub(2));
    let height = 3u16;
    let x = area.x + area.width.saturating_sub(width + 1);
    // Position above status bar, stacked upward
    let y = area.y
        + area
            .height
            .saturating_sub(height + 2 + (stack_index as u16 * height));

    if y < area.y {
        return;
    }

    let toast_area = Rect::new(x, y, width, height);
    f.render_widget(Clear, toast_area);

    let color = match level {
        LogLevel::Info => Theme::SAGE,
        LogLevel::Warn => Theme::WARNING,
        LogLevel::Error => Theme::EMBER,
        LogLevel::Debug => Theme::TEXT_DIM,
    };

    // Truncate message to fit
    let max_len = (width as usize).saturating_sub(4);
    let display = crate::tui::truncate_chars(message, max_len, "...");

    let line = Line::from(vec![Span::styled(
        format!(" {display} "),
        Style::default().fg(color),
    )]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, toast_area);
}
