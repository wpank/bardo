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
