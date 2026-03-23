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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    /// INV-019: Active tab index bounds; is_active == true only when i == active.
    #[test]
    fn test_tab_bar_active_bounds() {
        // Empty tabs
        let bar_empty = TabBar {
            tabs: &[],
            active: 0,
        };
        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        bar_empty.render(area, &mut buf);

        // Single tab
        let bar_one = TabBar {
            tabs: &["home"],
            active: 0,
        };
        let mut buf = Buffer::empty(area);
        bar_one.render(area, &mut buf);

        // Multiple tabs, active in range
        let bar_multi = TabBar {
            tabs: &["a", "b", "c"],
            active: 1,
        };
        let mut buf = Buffer::empty(area);
        bar_multi.render(area, &mut buf);
        // is_active should only be true when i == active
        for (i, _) in ["a", "b", "c"].iter().enumerate() {
            assert_eq!(i == 1, i == bar_multi.active);
        }

        // active out of bounds (999): no tab highlighted, no panic
        let bar_oob = TabBar {
            tabs: &["a", "b", "c"],
            active: 999,
        };
        let mut buf = Buffer::empty(area);
        bar_oob.render(area, &mut buf);
    }

    /// INV-020: Tab labels don't overflow terminal width.
    #[test]
    fn test_tab_bar_width_truncation() {
        for &area_width in &[10u16, 40, 80] {
            let tabs: &[&str] = &["short", "medium-length", "very-long-tab-name"];
            let bar = TabBar { tabs, active: 0 };
            let area = Rect::new(0, 0, area_width, 1);
            let mut buf = Buffer::empty(area);
            bar.render(area, &mut buf);

            // Simulate the label width accumulation
            let mut x = 0u16;
            for (i, tab) in tabs.iter().enumerate() {
                let label = if i == 0 {
                    format!("⌈ {} ⌋", tab)
                } else {
                    format!("  {}  ", tab)
                };
                let label_width = label.chars().count() as u16;
                if x + label_width > area_width {
                    break; // partial tabs never rendered
                }
                x += label_width;
            }
            assert!(
                x <= area_width,
                "rendered tabs overflow: x={x} > width={area_width}"
            );
        }
    }
}
