//! Horizontal tab strip widget.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::palette::{BG_MID, BG_RAISED, BONE, TEXT_DIM};

fn in_bounds(buf: &Buffer, x: u16, y: u16) -> bool {
    let area = buf.area();
    x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom()
}

/// Horizontal tab strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TabBar<'a> {
    /// Tab labels.
    pub(crate) tabs: &'a [&'a str],
    /// Active tab index.
    pub(crate) active: usize,
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        for offset in 0..area.width {
            let x = area.x + offset;
            if in_bounds(buf, x, area.y) {
                let cell = buf.get_mut(x, area.y);
                cell.set_char(' ');
                cell.set_style(Style::default().bg(BG_MID));
            }
        }

        let mut x = area.x;
        for (index, tab) in self.tabs.iter().enumerate() {
            let is_active = index == self.active;
            let label = if is_active {
                format!("⌈ {} ⌋", tab)
            } else {
                format!("  {}  ", tab)
            };
            let width = label.chars().count() as u16;
            if x + width > area.x + area.width {
                break;
            }

            buf.set_stringn(
                x,
                area.y,
                &label,
                width as usize,
                Style::default()
                    .fg(if is_active { BONE } else { TEXT_DIM })
                    .bg(if is_active { BG_RAISED } else { BG_MID })
                    .add_modifier(if is_active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );
            x += width;
        }
    }
}
