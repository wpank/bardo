//! Braille sparkline widget for dense single-row traces.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::palette::TEXT_DIM;

const LEFT_COL_BITS: [u8; 4] = [0x40, 0x04, 0x02, 0x01];
const RIGHT_COL_BITS: [u8; 4] = [0x80, 0x20, 0x10, 0x08];

fn filled_bits(bits: &[u8; 4], n_dots: usize) -> u8 {
    bits[4_usize.saturating_sub(n_dots.min(4))..]
        .iter()
        .fold(0u8, |acc, &bit| acc | bit)
}

fn left_bits(n_dots: usize) -> u8 {
    filled_bits(&LEFT_COL_BITS, n_dots)
}

fn right_bits(n_dots: usize) -> u8 {
    filled_bits(&RIGHT_COL_BITS, n_dots)
}

fn in_bounds(buf: &Buffer, x: u16, y: u16) -> bool {
    let area = buf.area();
    x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom()
}

/// Two-column braille sparkline for up to 80 samples in 40 terminal columns.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BrailleSparkline {
    /// Sample values rendered left-to-right.
    pub(crate) data: Vec<f64>,
    /// Explicit scale ceiling. Zero enables auto-scaling from the visible data.
    pub(crate) max_value: f64,
    /// Foreground color for the braille glyphs.
    pub(crate) color: Color,
    /// Optional dim label rendered on the last row when height permits.
    pub(crate) label: Option<String>,
}

impl Widget for BrailleSparkline {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let data_capacity = area.width as usize * 2;
        let offset = self.data.len().saturating_sub(data_capacity);
        let max = if self.max_value > 0.0 {
            self.max_value
        } else {
            self.data[offset..]
                .iter()
                .copied()
                .fold(0.0_f64, f64::max)
                .max(1.0)
        };

        let sparkline_y = area.y;
        for cell_idx in 0..area.width as usize {
            let left_index = offset + cell_idx * 2;
            let right_index = left_index + 1;

            let left_value = self.data.get(left_index).copied().unwrap_or(0.0);
            let right_value = self.data.get(right_index).copied().unwrap_or(0.0);
            let left_dots = ((left_value / max) * 4.0).round().clamp(0.0, 4.0) as usize;
            let right_dots = ((right_value / max) * 4.0).round().clamp(0.0, 4.0) as usize;
            let bits = left_bits(left_dots) | right_bits(right_dots);
            let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');

            let x = area.x + cell_idx as u16;
            if in_bounds(buf, x, sparkline_y) {
                let cell = buf.get_mut(x, sparkline_y);
                cell.set_char(ch);
                cell.set_style(Style::default().fg(self.color));
            }
        }

        if area.height >= 2 {
            if let Some(label) = self.label.as_deref() {
                buf.set_stringn(
                    area.x,
                    area.y + area.height - 1,
                    label,
                    area.width as usize,
                    Style::default().fg(TEXT_DIM),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_braille_sparkline_encodes_correctly() {
        assert_eq!(left_bits(0), 0x00);
        assert_eq!(left_bits(4), 0x01 | 0x02 | 0x04 | 0x40);

        let all_bits = left_bits(4) | right_bits(4);
        assert_eq!(all_bits, 0xFF);
        let ch = char::from_u32(0x2800 + all_bits as u32).unwrap();
        assert_eq!(ch, '\u{28FF}');
    }
}
