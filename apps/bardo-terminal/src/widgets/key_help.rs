//! Centered keybinding help overlay widget.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::palette::{
    BG_RAISED, BONE, BORDER_ACTIVE, BOX_BOTTOM_LEFT, BOX_BOTTOM_RIGHT, BOX_HORIZONTAL,
    BOX_TOP_LEFT, BOX_TOP_RIGHT, BOX_VERTICAL, ROSE, TEXT_DIM, TEXT_PRIMARY,
};

fn in_bounds(buf: &Buffer, x: u16, y: u16) -> bool {
    let area = buf.area();
    x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom()
}

fn set_cell(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    if in_bounds(buf, x, y) {
        let cell = buf.get_mut(x, y);
        cell.set_char(ch);
        cell.set_style(style);
    }
}

/// One keybinding hint row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyBinding {
    /// Displayed key chord.
    pub(crate) key: String,
    /// Human-readable action description.
    pub(crate) description: String,
}

/// Floating help overlay listing keybindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyHelpOverlay {
    /// Rows to render.
    pub(crate) bindings: Vec<KeyBinding>,
    /// Whether the overlay is visible.
    pub(crate) visible: bool,
}

impl Widget for &KeyHelpOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.visible || area.width < 20 || area.height < 5 {
            return;
        }

        let max_key_len = self
            .bindings
            .iter()
            .map(|binding| binding.key.len())
            .max()
            .unwrap_or(1);
        let max_desc_len = self
            .bindings
            .iter()
            .map(|binding| binding.description.len())
            .max()
            .unwrap_or(10);
        let inner_width =
            (max_key_len + max_desc_len + 5).min(area.width.saturating_sub(4) as usize);
        let inner_height = self
            .bindings
            .len()
            .min(area.height.saturating_sub(4) as usize);
        let box_width = (inner_width + 4) as u16;
        let box_height = (inner_height + 2) as u16;
        let box_x = area.x + area.width.saturating_sub(box_width) / 2;
        let box_y = area.y + area.height.saturating_sub(box_height) / 2;
        let border_style = Style::default().fg(BORDER_ACTIVE).bg(BG_RAISED);
        let inner_style = Style::default().fg(TEXT_PRIMARY).bg(BG_RAISED);

        set_cell(buf, box_x, box_y, BOX_TOP_LEFT, border_style);
        for x in (box_x + 1)..(box_x + box_width - 1) {
            set_cell(buf, x, box_y, BOX_HORIZONTAL, border_style);
        }
        set_cell(
            buf,
            box_x + box_width - 1,
            box_y,
            BOX_TOP_RIGHT,
            border_style,
        );

        buf.set_stringn(
            box_x + 2,
            box_y,
            " KEY BINDINGS ",
            box_width.saturating_sub(4) as usize,
            Style::default().fg(BONE).bg(BG_RAISED),
        );

        for (row, binding) in self.bindings.iter().take(inner_height).enumerate() {
            let y = box_y + 1 + row as u16;
            set_cell(buf, box_x, y, BOX_VERTICAL, border_style);
            for x in (box_x + 1)..(box_x + box_width - 1) {
                set_cell(buf, x, y, ' ', inner_style);
            }

            let key_display = format!("{:<width$}", binding.key, width = max_key_len);
            buf.set_stringn(
                box_x + 2,
                y,
                &key_display,
                max_key_len,
                Style::default()
                    .fg(ROSE)
                    .bg(BG_RAISED)
                    .add_modifier(Modifier::BOLD),
            );
            let separator_x = box_x + 2 + max_key_len as u16;
            buf.set_stringn(
                separator_x,
                y,
                " - ",
                3,
                Style::default().fg(TEXT_DIM).bg(BG_RAISED),
            );
            let desc_x = separator_x + 3;
            buf.set_stringn(
                desc_x,
                y,
                &binding.description,
                max_desc_len.min(area.width as usize),
                Style::default().fg(TEXT_PRIMARY).bg(BG_RAISED),
            );
            set_cell(buf, box_x + box_width - 1, y, BOX_VERTICAL, border_style);
        }

        let bottom_y = box_y + box_height - 1;
        set_cell(buf, box_x, bottom_y, BOX_BOTTOM_LEFT, border_style);
        for x in (box_x + 1)..(box_x + box_width - 1) {
            set_cell(buf, x, bottom_y, BOX_HORIZONTAL, border_style);
        }
        set_cell(
            buf,
            box_x + box_width - 1,
            bottom_y,
            BOX_BOTTOM_RIGHT,
            border_style,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    #[test]
    fn hidden_overlay_renders_nothing() {
        let overlay = KeyHelpOverlay {
            bindings: vec![KeyBinding {
                key: "?".to_string(),
                description: "toggle help".to_string(),
            }],
            visible: false,
        };
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 10));
        (&overlay).render(Rect::new(0, 0, 30, 10), &mut buffer);

        assert!(buffer.content().iter().all(|cell| cell.symbol() == " "));
    }
}
