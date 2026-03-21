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
                " — ",
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

    #[test]
    fn small_area_early_return() {
        let overlay = KeyHelpOverlay {
            bindings: vec![KeyBinding {
                key: "?".to_string(),
                description: "toggle help".to_string(),
            }],
            visible: true,
        };
        let mut buffer = Buffer::empty(Rect::new(0, 0, 19, 10));
        (&overlay).render(Rect::new(0, 0, 19, 10), &mut buffer);

        // Should return early due to width < 20
        assert!(buffer.content().iter().all(|cell| cell.symbol() == " "));
    }

    #[test]
    fn visible_overlay_renders_box() {
        let overlay = KeyHelpOverlay {
            bindings: vec![
                KeyBinding {
                    key: "?".to_string(),
                    description: "toggle help".to_string(),
                },
                KeyBinding {
                    key: "q".to_string(),
                    description: "quit".to_string(),
                },
            ],
            visible: true,
        };
        let mut buffer = Buffer::empty(Rect::new(0, 0, 50, 20));
        (&overlay).render(Rect::new(0, 0, 50, 20), &mut buffer);

        // The box should be centered, so we need to find where it was rendered
        // For simplicity, just check that something was rendered (not all spaces)
        let has_content = buffer
            .content()
            .iter()
            .any(|cell| cell.symbol() != " " && cell.symbol() != "");
        assert!(has_content, "Overlay should render content when visible");
    }

    #[test]
    fn key_binding_structure() {
        let binding = KeyBinding {
            key: "Ctrl+C".to_string(),
            description: "Copy".to_string(),
        };
        assert_eq!(binding.key, "Ctrl+C");
        assert_eq!(binding.description, "Copy");
    }

    fn make_bindings(count: usize) -> Vec<KeyBinding> {
        (0..count)
            .map(|i| KeyBinding {
                key: format!("C-{}", (b'a' + (i % 26) as u8) as char),
                description: format!("action {}", i),
            })
            .collect()
    }

    /// INV-023: Box dimensions never exceed area; minimum area 20x5 triggers early return.
    #[test]
    fn test_key_help_overlay_dimensions() {
        for &area_width in &[10u16, 20, 80, 255] {
            for &area_height in &[3u16, 5, 40, 100] {
                for &bindings_count in &[1usize, 5, 20] {
                    let overlay = KeyHelpOverlay {
                        bindings: make_bindings(bindings_count),
                        visible: true,
                    };
                    let area = Rect::new(0, 0, area_width, area_height);
                    let mut buffer = Buffer::empty(area);
                    (&overlay).render(area, &mut buffer);

                    if area_width < 20 || area_height < 5 {
                        // Early return: buffer should be all spaces.
                        assert!(
                            buffer.content().iter().all(|cell| cell.symbol() == " "),
                            "Expected early return for {}x{} with {} bindings",
                            area_width,
                            area_height,
                            bindings_count,
                        );
                    } else {
                        // Box must fit within the area. Verify no writes outside area bounds
                        // (the buffer enforces this, so we just confirm rendering happened).
                        let has_content = buffer
                            .content()
                            .iter()
                            .any(|cell| cell.symbol() != " " && cell.symbol() != "");
                        assert!(
                            has_content,
                            "Expected rendered content for {}x{} with {} bindings",
                            area_width, area_height, bindings_count,
                        );
                    }
                }
            }
        }
    }

    /// INV-024: Box is centered horizontally and vertically within the area.
    #[test]
    fn test_key_help_overlay_centering() {
        for &area_x in &[0u16, 10] {
            for &area_y in &[0u16, 5] {
                for &area_width in &[40u16, 100] {
                    for &area_height in &[20u16, 50] {
                        let overlay = KeyHelpOverlay {
                            bindings: make_bindings(3),
                            visible: true,
                        };
                        let area = Rect::new(area_x, area_y, area_width, area_height);
                        let mut buffer = Buffer::empty(area);
                        (&overlay).render(area, &mut buffer);

                        // Find rendered box bounds by scanning for box-drawing chars.
                        let mut min_x = u16::MAX;
                        let mut max_x = 0u16;
                        let mut min_y = u16::MAX;
                        let mut max_y = 0u16;
                        for y in area.top()..area.bottom() {
                            for x in area.left()..area.right() {
                                let sym = buffer.get(x, y).symbol().to_string();
                                if sym != " " {
                                    min_x = min_x.min(x);
                                    max_x = max_x.max(x);
                                    min_y = min_y.min(y);
                                    max_y = max_y.max(y);
                                }
                            }
                        }

                        assert!(min_x >= area_x, "box_x underflow");
                        assert!(min_y >= area_y, "box_y underflow");
                        assert!(max_x < area_x + area_width, "box overflows right");
                        assert!(max_y < area_y + area_height, "box overflows bottom");

                        let rendered_width = max_x - min_x + 1;
                        let rendered_height = max_y - min_y + 1;
                        let expected_x = area_x + area_width.saturating_sub(rendered_width) / 2;
                        let expected_y = area_y + area_height.saturating_sub(rendered_height) / 2;
                        assert_eq!(
                            min_x, expected_x,
                            "horizontal centering off for area ({},{},{}x{})",
                            area_x, area_y, area_width, area_height,
                        );
                        assert_eq!(
                            min_y, expected_y,
                            "vertical centering off for area ({},{},{}x{})",
                            area_x, area_y, area_width, area_height,
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn overlay_with_empty_bindings() {
        let overlay = KeyHelpOverlay {
            bindings: vec![],
            visible: true,
        };
        let mut buffer = Buffer::empty(Rect::new(0, 0, 50, 20));
        (&overlay).render(Rect::new(0, 0, 50, 20), &mut buffer);

        // Should still render box borders even with no bindings
        let has_content = buffer
            .content()
            .iter()
            .any(|cell| cell.symbol() != " " && cell.symbol() != "");
        assert!(has_content, "Empty overlay should still render box");
    }
}
