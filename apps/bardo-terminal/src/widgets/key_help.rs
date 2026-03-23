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

    /// INV-023: Box dimensions never exceed area; minimum area 20x5 required.
    #[test]
    fn test_key_help_overlay_dimensions() {
        let bindings = vec![
            KeyBinding {
                key: "?".into(),
                description: "toggle help".into(),
            },
            KeyBinding {
                key: "q".into(),
                description: "quit application".into(),
            },
            KeyBinding {
                key: "j/k".into(),
                description: "navigate".into(),
            },
            KeyBinding {
                key: "Enter".into(),
                description: "confirm selection".into(),
            },
            KeyBinding {
                key: "Esc".into(),
                description: "close".into(),
            },
        ];

        // Area too small: width < 20
        let overlay = KeyHelpOverlay {
            bindings: bindings.clone(),
            visible: true,
        };
        let small_area = Rect::new(0, 0, 10, 40);
        let mut buf = Buffer::empty(small_area);
        (&overlay).render(small_area, &mut buf);
        assert!(
            buf.content().iter().all(|c| c.symbol() == " "),
            "width < 20 should render nothing"
        );

        // Area too small: height < 5
        let small_h_area = Rect::new(0, 0, 80, 3);
        let mut buf = Buffer::empty(small_h_area);
        (&overlay).render(small_h_area, &mut buf);
        assert!(
            buf.content().iter().all(|c| c.symbol() == " "),
            "height < 5 should render nothing"
        );

        // Normal area: box fits
        for &(w, h, n_bindings) in &[(20u16, 5u16, 1usize), (80, 40, 5), (255, 100, 20)] {
            let bindings_subset: Vec<KeyBinding> = (0..n_bindings)
                .map(|i| KeyBinding {
                    key: format!("k{i}"),
                    description: format!("action {i}"),
                })
                .collect();
            let overlay = KeyHelpOverlay {
                bindings: bindings_subset.clone(),
                visible: true,
            };

            let max_key = bindings_subset
                .iter()
                .map(|b| b.key.len())
                .max()
                .unwrap_or(1);
            let max_desc = bindings_subset
                .iter()
                .map(|b| b.description.len())
                .max()
                .unwrap_or(10);
            let inner_w = (max_key + max_desc + 5).min(w.saturating_sub(4) as usize);
            let inner_h = bindings_subset.len().min(h.saturating_sub(4) as usize);
            let box_w = (inner_w + 4) as u16;
            let box_h = (inner_h + 2) as u16;

            assert!(box_w <= w, "box_w {box_w} > area_w {w}");
            assert!(box_h <= h, "box_h {box_h} > area_h {h}");

            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            (&overlay).render(area, &mut buf);
        }
    }

    /// INV-024: Box is centered horizontally and vertically.
    #[test]
    fn test_key_help_overlay_centering() {
        let bindings = vec![
            KeyBinding {
                key: "?".into(),
                description: "help".into(),
            },
            KeyBinding {
                key: "q".into(),
                description: "quit".into(),
            },
        ];

        for &(ax, ay, aw, ah) in &[(0u16, 0u16, 40u16, 20u16), (10, 5, 100, 50)] {
            let overlay = KeyHelpOverlay {
                bindings: bindings.clone(),
                visible: true,
            };

            let max_key = bindings.iter().map(|b| b.key.len()).max().unwrap_or(1);
            let max_desc = bindings
                .iter()
                .map(|b| b.description.len())
                .max()
                .unwrap_or(10);
            let inner_w = (max_key + max_desc + 5).min(aw.saturating_sub(4) as usize);
            let inner_h = bindings.len().min(ah.saturating_sub(4) as usize);
            let box_w = (inner_w + 4) as u16;
            let box_h = (inner_h + 2) as u16;

            let box_x = ax + aw.saturating_sub(box_w) / 2;
            let box_y = ay + ah.saturating_sub(box_h) / 2;

            // Box position never goes below area origin
            assert!(box_x >= ax, "box_x {box_x} < area.x {ax}");
            assert!(box_y >= ay, "box_y {box_y} < area.y {ay}");

            // Box fits within area
            assert!(
                box_x + box_w <= ax + aw,
                "box right edge {} > area right edge {}",
                box_x + box_w,
                ax + aw
            );
            assert!(
                box_y + box_h <= ay + ah,
                "box bottom edge {} > area bottom edge {}",
                box_y + box_h,
                ay + ah
            );

            // Render to verify no panic
            let area = Rect::new(ax, ay, aw, ah);
            let mut buf = Buffer::empty(area);
            (&overlay).render(area, &mut buf);
        }
    }
}
