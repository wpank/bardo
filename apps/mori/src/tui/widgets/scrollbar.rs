use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::tui::theme::Theme;

/// Render a minimal scrollbar on the rightmost column of `area`.
/// No-op if total <= visible.
pub fn render_scrollbar(
    buf: &mut Buffer,
    area: Rect,
    total: usize,
    visible: usize,
    offset: usize,
    accent: Color,
) {
    if total <= visible || area.height == 0 || area.width == 0 {
        return;
    }

    let height = area.height as usize;
    let x = area.x + area.width - 1;

    // Thumb size and position
    let thumb_size = ((visible * height) / total).max(1).min(height);
    let max_offset = total.saturating_sub(visible);
    let thumb_pos = if max_offset > 0 {
        (offset.min(max_offset) * height.saturating_sub(thumb_size)) / max_offset
    } else {
        0
    };

    for y_off in 0..height {
        let y = area.y + y_off as u16;
        if y_off >= thumb_pos && y_off < thumb_pos + thumb_size {
            // Thumb
            buf[(x, y)].set_symbol("█");
            buf[(x, y)].set_style(Style::default().fg(accent));
        } else {
            // Track
            buf[(x, y)].set_symbol("░");
            buf[(x, y)].set_style(Style::default().fg(Theme::TEXT_GHOST));
        }
    }
}
