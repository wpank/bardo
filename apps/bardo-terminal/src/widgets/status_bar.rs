//! Fixed bottom-row status bar widget.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::palette::{BG_MID, BONE, TEXT_DIM};

/// Single-line bottom status bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StatusBar<'a> {
    /// Phase label. TODO Plan 70a: from AppState
    pub(crate) phase: &'a str,
    /// Current tick.
    pub(crate) tick: u64,
    /// PAD summary text. TODO Plan 70a: from AppState
    pub(crate) pad_summary: &'a str,
    /// Credit balance text. TODO Plan 70a: from AppState
    pub(crate) credit_balance: &'a str,
    /// Optional projected days remaining. TODO Plan 70a: from AppState
    pub(crate) projected_days: Option<f64>,
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        for offset in 0..area.width {
            let cell = buf.get_mut(area.x + offset, area.y);
            cell.set_char(' ');
            cell.set_style(Style::default().bg(BG_MID));
        }

        let left = format!(" {} #{}", self.phase, self.tick);
        buf.set_stringn(
            area.x,
            area.y,
            left,
            area.width as usize,
            Style::default()
                .fg(BONE)
                .bg(BG_MID)
                .add_modifier(Modifier::BOLD),
        );

        let center_x = area.x + area.width / 3;
        if center_x < area.x + area.width {
            buf.set_stringn(
                center_x,
                area.y,
                self.pad_summary,
                area.width.saturating_sub(center_x - area.x) as usize,
                Style::default().fg(TEXT_DIM).bg(BG_MID),
            );
        }

        let right = if let Some(days) = self.projected_days {
            format!("{} | {:.0}d remaining ", self.credit_balance, days)
        } else {
            format!("{} ", self.credit_balance)
        };
        let right_width = right.chars().count() as u16;
        let right_x = area.x + area.width.saturating_sub(right_width);
        buf.set_stringn(
            right_x,
            area.y,
            right,
            area.width.saturating_sub(right_x - area.x) as usize,
            Style::default().fg(TEXT_DIM).bg(BG_MID),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn make_bar(width: u16, credit: &str, days: Option<f64>) -> (StatusBar<'_>, Rect, Buffer) {
        let area = Rect::new(0, 0, width, 1);
        let buf = Buffer::empty(area);
        let bar = StatusBar {
            phase: "STABLE",
            tick: 42,
            pad_summary: "P:0.3 A:0.1 D:0.4",
            credit_balance: credit,
            projected_days: days,
        };
        (bar, area, buf)
    }

    #[test]
    fn test_status_bar_layout() {
        // INV-018: Center section at 1/3 width; right section right-aligned.
        // Test across area_width=[1, 40, 100, 255] and right_len=[0, 20, 50].
        let credits: &[&str] = &["", "$142.50 some padding", &"X".repeat(50)];

        for &width in &[1u16, 40, 100, 255] {
            for &credit in credits {
                for &days in &[None, Some(30.0)] {
                    let (bar, area, mut buf) = make_bar(width, credit, days);

                    let right_text = if let Some(d) = bar.projected_days {
                        format!("{} | {:.0}d remaining ", bar.credit_balance, d)
                    } else {
                        format!("{} ", bar.credit_balance)
                    };
                    let right_len = right_text.chars().count() as u16;

                    let center_x = area.x + area.width / 3;
                    let right_x = area.x + area.width.saturating_sub(right_len);

                    // center_x in [area.x, area.x + area.width]
                    assert!(
                        center_x >= area.x && center_x <= area.x + area.width,
                        "center_x={center_x} out of bounds for width={width}"
                    );

                    // right_x >= area.x
                    assert!(
                        right_x >= area.x,
                        "right_x={right_x} < area.x for width={width}, right_len={right_len}"
                    );

                    // Render should not panic
                    bar.render(area, &mut buf);
                }
            }
        }
    }

    #[test]
    fn status_bar_empty_area_no_panic() {
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        let bar = StatusBar {
            phase: "TERMINAL",
            tick: 0,
            pad_summary: "",
            credit_balance: "",
            projected_days: None,
        };
        bar.render(area, &mut buf);
    }

    #[test]
    fn status_bar_renders_phase_and_tick() {
        let (bar, area, mut buf) = make_bar(80, "$50.00", Some(10.0));
        bar.render(area, &mut buf);

        let content: String = (0..area.width)
            .map(|x| buf.get(x, 0).symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(content.contains("STABLE"), "phase missing from: {content}");
        assert!(content.contains("#42"), "tick missing from: {content}");
    }
}
