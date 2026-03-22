//! Listener Mode widget — five horizontal mood sliders (0-10 scale).
//!
//! Replaces the module graph when active. Arrow Up/Down selects,
//! Arrow Left/Right adjusts (±0.5, Shift ±0.1). All sliders at 5.0 = neutral.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::{
    palette::{BG_RAISED, BG_VOID, BONE, ROSE, ROSE_BRIGHT, ROSE_DIM, TEXT_DIM, TEXT_PRIMARY},
    sonification::{LISTENER_SLIDERS, ListenerState},
};

/// Horizontal bar width in terminal cells (maps 0-10 range).
const BAR_WIDTH: u16 = 30;
/// Label column width.
const LABEL_WIDTH: u16 = 12;

/// Renders the five Listener Mode sliders.
pub(crate) struct ListenerModeWidget<'a> {
    pub(crate) state: &'a ListenerState,
}

impl<'a> Widget for ListenerModeWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < LABEL_WIDTH + BAR_WIDTH + 6 || area.height < 5 {
            return;
        }

        // Title
        let title = if self.state.active {
            "LISTENER MODE [L/Esc to exit]"
        } else {
            "LISTENER MODE (inactive)"
        };
        let title_style = Style::default()
            .fg(if self.state.active {
                ROSE_BRIGHT
            } else {
                TEXT_DIM
            })
            .add_modifier(Modifier::BOLD);

        if area.y < buf.area().bottom() {
            buf.set_stringn(area.x, area.y, title, area.width as usize, title_style);
        }

        let slider_start_y = area.y + 1;

        for (i, &name) in LISTENER_SLIDERS.iter().enumerate() {
            let y = slider_start_y + i as u16;
            if y >= area.bottom() {
                break;
            }

            let is_selected = i == self.state.cursor;
            let value = self.state.values[i];

            // Prefix arrow
            let prefix = if is_selected { "▶ " } else { "  " };
            let label_style = Style::default()
                .fg(if is_selected { BONE } else { TEXT_PRIMARY })
                .bg(if is_selected { BG_RAISED } else { BG_VOID });

            let mut x = area.x;

            // Prefix + Label
            let label_text = format!("{prefix}{name:<width$}", width = (LABEL_WIDTH - 2) as usize);
            buf.set_stringn(x, y, &label_text, LABEL_WIDTH as usize, label_style);
            x += LABEL_WIDTH;

            // Bar background
            let filled = ((value / 10.0) * BAR_WIDTH as f64).round() as u16;
            let bar_color = slider_color(i, is_selected);

            for bx in 0..BAR_WIDTH {
                let cx = x + bx;
                if cx >= buf.area().right() {
                    break;
                }
                let cell = buf.get_mut(cx, y);
                if bx < filled {
                    cell.set_char('█');
                    cell.set_style(Style::default().fg(bar_color));
                } else {
                    cell.set_char('░');
                    cell.set_style(Style::default().fg(TEXT_DIM));
                }
            }
            x += BAR_WIDTH;

            // Value label
            let val_text = format!(" {value:.1}");
            let val_style = Style::default().fg(if is_selected { BONE } else { TEXT_PRIMARY });
            if x + val_text.len() as u16 <= buf.area().right() && y < buf.area().bottom() {
                buf.set_stringn(x, y, &val_text, 5, val_style);
            }
        }
    }
}

fn slider_color(index: usize, selected: bool) -> ratatui::style::Color {
    if selected {
        ROSE_BRIGHT
    } else {
        match index {
            0 => ROSE,     // Brightness
            1 => ROSE_DIM, // Density
            2 => BONE,     // Motion
            3 => ROSE,     // Depth
            4 => ROSE_DIM, // Warmth
            _ => TEXT_PRIMARY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sonification::ListenerState;

    #[test]
    fn listener_mode_renders_five_sliders() {
        let state = ListenerState::default();
        let widget = ListenerModeWidget { state: &state };
        let area = Rect::new(0, 0, 60, 7);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Title row should contain "LISTENER MODE"
        let title_row: String = (0..60)
            .map(|x| buf.get(x, 0).symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(title_row.contains("LISTENER MODE"));
    }

    #[test]
    fn listener_mode_shows_slider_names() {
        let state = ListenerState::default();
        let widget = ListenerModeWidget { state: &state };
        let area = Rect::new(0, 0, 60, 7);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        for name in &["Brightness", "Density", "Motion", "Depth", "Warmth"] {
            let found = (1..7).any(|y| {
                let row: String = (0..60)
                    .map(|x| buf.get(x, y).symbol().to_string())
                    .collect::<Vec<_>>()
                    .join("");
                row.contains(name)
            });
            assert!(found, "Missing slider: {name}");
        }
    }

    #[test]
    fn listener_mode_selected_slider_has_arrow() {
        let mut state = ListenerState::default();
        state.cursor = 2; // Motion
        let widget = ListenerModeWidget { state: &state };
        let area = Rect::new(0, 0, 60, 7);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Row 3 (y=3) should have the arrow indicator
        let row3: String = (0..12)
            .map(|x| buf.get(x, 3).symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(row3.contains("▶"));
    }

    #[test]
    fn listener_mode_bar_fill_reflects_value() {
        let mut state = ListenerState::default();
        state.values[0] = 10.0; // full
        state.values[1] = 0.0; // empty
        let widget = ListenerModeWidget { state: &state };
        let area = Rect::new(0, 0, 60, 7);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Count filled cells in Brightness row (y=1, starting at LABEL_WIDTH)
        let filled_count = (LABEL_WIDTH..LABEL_WIDTH + BAR_WIDTH)
            .filter(|&x| buf.get(x, 1).symbol() == "█")
            .count();
        assert_eq!(filled_count, BAR_WIDTH as usize); // fully filled

        // Density row (y=2) should be all empty
        let empty_count = (LABEL_WIDTH..LABEL_WIDTH + BAR_WIDTH)
            .filter(|&x| buf.get(x, 2).symbol() == "░")
            .count();
        assert_eq!(empty_count, BAR_WIDTH as usize);
    }

    #[test]
    fn listener_mode_no_panic_on_tiny_area() {
        let state = ListenerState::default();
        let widget = ListenerModeWidget { state: &state };
        let area = Rect::new(0, 0, 10, 2);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // should not panic
    }
}
