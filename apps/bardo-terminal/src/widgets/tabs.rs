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
        for (i, &tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn render_tab_bar(tabs: &[&str], active: usize, width: u16) -> Buffer {
        let area = Rect::new(0, 0, width, 1);
        let mut buf = Buffer::empty(area);
        let tab_bar = TabBar { tabs, active };
        tab_bar.render(area, &mut buf);
        buf
    }

    #[test]
    fn test_tab_bar_active_bounds() {
        // Empty tabs: renders background only, no panic
        let buf = render_tab_bar(&[], 0, 40);
        assert!(buf.content().iter().all(|c| c.symbol() == " "));

        // Single tab, active=0: the one tab is highlighted
        let buf = render_tab_bar(&["home"], 0, 40);
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("home"));
        // Active tab cell uses BONE fg
        let first_label_cell = &buf.content()[0];
        assert_eq!(first_label_cell.fg, BONE);

        // Three tabs, active=1: only index 1 is active
        let buf = render_tab_bar(&["a", "b", "c"], 1, 40);
        // Tab "a" (index 0) should use TEXT_DIM
        let cell_a = &buf.content()[0];
        assert_eq!(cell_a.fg, TEXT_DIM);
        // Find where "b" tab starts (after "  a  " = 5 chars)
        // Active tab "b" rendered as "⌈ b ⌋" = 5 chars
        let cell_b = &buf.content()[5];
        assert_eq!(cell_b.fg, BONE);

        // Out-of-bounds active index: no tab is highlighted, no panic
        let buf = render_tab_bar(&["a", "b", "c"], 999, 40);
        // All rendered tab cells should use TEXT_DIM (none active)
        for cell in buf.content().iter() {
            if cell.symbol() != " " {
                assert_eq!(cell.fg, TEXT_DIM);
            }
        }
    }

    #[test]
    fn test_tab_bar_width_truncation() {
        // Narrow area: only tabs that fully fit are rendered
        // "  Home  " = 8 chars, "  Beat  " = 8 chars, "  Mind  " = 8 chars
        // Width 10: only first tab fits (8 <= 10), second would need 16 > 10
        let tabs = &["Home", "Beat", "Mind", "Dreams"];
        let buf = render_tab_bar(tabs, 999, 10);
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("Home"));
        assert!(!content.contains("Beat"));

        // Width 40: verify no content appears past the right edge
        let width: u16 = 40;
        let buf = render_tab_bar(tabs, 0, width);
        // All rendered content must be within area bounds
        for (i, cell) in buf.content().iter().enumerate() {
            assert!(
                (i as u16) < width,
                "cell at index {} exceeds width {}",
                i,
                width
            );
        }

        // Very narrow: width 1, nothing fits, just background
        let buf = render_tab_bar(&["tab"], 0, 1);
        assert!(buf.content().iter().all(|c| c.symbol() == " "));

        // Exact fit: label "  X  " = 5 chars in width 5
        let buf = render_tab_bar(&["X", "Y"], 999, 5);
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("X"));
        assert!(!content.contains("Y"));
    }
}
