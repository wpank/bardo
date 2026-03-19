//! Animated progress bar with ETA countdown and sub-cell fill precision.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::palette::{
    BG_RAISED, BLOCK_DARK, BLOCK_FULL, BLOCK_LIGHT, BLOCK_MED, BONE, BORDER, DREAM, ROSE,
    ROSE_BRIGHT, ROSE_DIM, SUCCESS,
};
use crate::state::format_duration;

fn in_bounds(buf: &Buffer, x: u16, y: u16) -> bool {
    let area = buf.area();
    x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom()
}

/// Interpolate between two RGB colors by t (0.0..1.0).
fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (a, b) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8;
            let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8;
            let bv = (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8;
            Color::Rgb(r, g, bv)
        }
        _ => {
            if t < 0.5 {
                a
            } else {
                b
            }
        }
    }
}

/// Scale a color's brightness by a factor.
fn scale_color(color: Color, factor: f64) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f64 * factor).clamp(0.0, 255.0) as u8,
            (g as f64 * factor).clamp(0.0, 255.0) as u8,
            (b as f64 * factor).clamp(0.0, 255.0) as u8,
        ),
        other => other,
    }
}

/// Smooth animated progress bar with gradient fill, pulsing edge, and ETA countdown.
///
/// Renders in 1-3 rows depending on available height:
/// - 3+ rows: label line, bar, info line
/// - 2 rows: bar, info line
/// - 1 row: bar only
pub(crate) struct TotalProgressBar {
    /// 0.0 to 1.0 completion fraction.
    pub(crate) progress: f64,
    /// Estimated seconds remaining.
    pub(crate) eta_secs: f64,
    /// Wall-clock elapsed seconds.
    pub(crate) elapsed_secs: f64,
    /// Heartbeat oscillator value (0.95..1.05) for edge pulse.
    pub(crate) heartbeat: f64,
    /// Whether the run is complete.
    pub(crate) complete: bool,
}

impl Widget for TotalProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 1 {
            return;
        }

        let bar_y;
        let info_y;

        if area.height >= 3 {
            // Row 0: label with percentage
            let pct_text = if self.complete {
                " COMPLETE".to_string()
            } else {
                format!(" PROGRESS  {:.0}%", self.progress * 100.0)
            };
            buf.set_stringn(
                area.x,
                area.y,
                &pct_text,
                area.width as usize,
                Style::default().fg(BONE).add_modifier(Modifier::BOLD),
            );
            bar_y = area.y + 1;
            info_y = Some(area.y + 2);
        } else if area.height >= 2 {
            bar_y = area.y;
            info_y = Some(area.y + 1);
        } else {
            bar_y = area.y;
            info_y = None;
        }

        // ── Bar rendering with sub-cell precision ───────────────
        let clamped = self.progress.clamp(0.0, 1.0);
        let fill_f = clamped * area.width as f64;
        let fill_full = fill_f as u16;
        let fill_frac = fill_f - fill_full as f64;

        // Sub-cell character for the partial fill at the edge
        let partial_char = if fill_frac < 0.125 {
            ' '
        } else if fill_frac < 0.375 {
            BLOCK_LIGHT
        } else if fill_frac < 0.625 {
            BLOCK_MED
        } else if fill_frac < 0.875 {
            BLOCK_DARK
        } else {
            BLOCK_FULL
        };

        // Gradient: ROSE_DIM -> ROSE -> ROSE_BRIGHT at the fill edge
        for offset in 0..area.width {
            let x = area.x + offset;
            if !in_bounds(buf, x, bar_y) {
                continue;
            }
            let cell = buf.get_mut(x, bar_y);

            if offset < fill_full {
                let t = if fill_full > 0 {
                    offset as f64 / fill_full as f64
                } else {
                    0.0
                };
                let color = if t < 0.5 {
                    lerp_color(ROSE_DIM, ROSE, t * 2.0)
                } else {
                    lerp_color(ROSE, ROSE_BRIGHT, (t - 0.5) * 2.0)
                };
                cell.set_char(BLOCK_FULL);
                cell.set_style(Style::default().fg(color).bg(BG_RAISED));
            } else if offset == fill_full && partial_char != ' ' {
                // Partial cell at the edge pulses with heartbeat
                let pulse = scale_color(ROSE_BRIGHT, self.heartbeat);
                cell.set_char(partial_char);
                cell.set_style(Style::default().fg(pulse).bg(BG_RAISED));
            } else {
                cell.set_char(BLOCK_LIGHT);
                cell.set_style(Style::default().fg(BORDER).bg(BG_RAISED));
            }
        }

        // ── Info line below bar ─────────────────────────────────
        if let Some(y) = info_y {
            let info = if self.complete {
                format!(" done in {}", format_duration(self.elapsed_secs))
            } else {
                format!(
                    " ETA {}  elapsed {}",
                    format_duration(self.eta_secs),
                    format_duration(self.elapsed_secs),
                )
            };
            buf.set_stringn(
                area.x,
                y,
                &info,
                area.width as usize,
                Style::default().fg(if self.complete { SUCCESS } else { DREAM }),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_renders_without_panic() {
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        TotalProgressBar {
            progress: 0.5,
            eta_secs: 60.0,
            elapsed_secs: 60.0,
            heartbeat: 1.0,
            complete: false,
        }
        .render(area, &mut buf);
    }

    #[test]
    fn complete_bar_renders_without_panic() {
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        TotalProgressBar {
            progress: 1.0,
            eta_secs: 0.0,
            elapsed_secs: 120.0,
            heartbeat: 1.0,
            complete: true,
        }
        .render(area, &mut buf);
    }

    #[test]
    fn narrow_bar_does_not_panic() {
        let area = Rect::new(0, 0, 3, 1);
        let mut buf = Buffer::empty(area);
        TotalProgressBar {
            progress: 0.5,
            eta_secs: 30.0,
            elapsed_secs: 30.0,
            heartbeat: 1.0,
            complete: false,
        }
        .render(area, &mut buf);
    }
}
