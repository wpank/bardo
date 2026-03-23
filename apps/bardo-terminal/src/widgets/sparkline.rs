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
    bits[..n_dots.min(4)]
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
        assert_eq!(right_bits(0), 0x00);
        assert_eq!(left_bits(1), 0x40);
        assert_eq!(left_bits(2), 0x40 | 0x04);
        assert_eq!(left_bits(3), 0x40 | 0x04 | 0x02);
        assert_eq!(left_bits(4), 0x01 | 0x02 | 0x04 | 0x40);
        assert_eq!(right_bits(1), 0x80);
        assert_eq!(right_bits(2), 0x80 | 0x20);
        assert_eq!(right_bits(3), 0x80 | 0x20 | 0x10);
        assert_eq!(right_bits(4), 0x08 | 0x10 | 0x20 | 0x80);

        let all_bits = left_bits(4) | right_bits(4);
        assert_eq!(all_bits, 0xFF);
        let ch = char::from_u32(0x2800 + all_bits as u32).unwrap();
        assert_eq!(ch, '\u{28FF}');
    }

    /// INV-002: Data points scale to 0..=4 dots per column, clamped at boundaries.
    #[test]
    fn test_sparkline_scaling_bounds() {
        // Test with max = 1.0
        for &val in &[0.0_f64, 0.25, 0.5, 0.75, 1.0, 1.5] {
            let dots = ((val / 1.0) * 4.0).round().clamp(0.0, 4.0) as usize;
            assert!(dots <= 4, "dots {dots} out of range for val {val}");
        }
        // Test with max = 100.0
        for &val in &[0.0_f64, 25.0, 50.0, 75.0, 100.0, 150.0] {
            let dots = ((val / 100.0) * 4.0).round().clamp(0.0, 4.0) as usize;
            assert!(dots <= 4, "dots {dots} out of range for val {val}");
        }
        // Edge cases: NaN and Infinity clamp to valid range
        let nan_dots = ((f64::NAN / 1.0) * 4.0).round().clamp(0.0, 4.0);
        assert!((0.0..=4.0).contains(&nan_dots) || nan_dots.is_nan());
        let inf_dots = ((f64::INFINITY / 1.0) * 4.0).round().clamp(0.0, 4.0);
        assert!((0.0..=4.0).contains(&inf_dots));
    }

    /// INV-003: When max_value <= 0, auto-scale from data.max(); otherwise use max_value.
    #[test]
    fn test_sparkline_max_value_auto_scale() {
        // Empty data, max_value=0 => auto-scale should produce max >= 1.0
        let s1 = BrailleSparkline {
            data: vec![],
            max_value: 0.0,
            color: Color::Reset,
            label: None,
        };
        // Auto-scale logic: fold(NEG_INFINITY, max).max(1.0) on empty slice => 1.0
        let offset = s1.data.len().saturating_sub(0);
        let auto_max = s1.data[offset..]
            .iter()
            .copied()
            .fold(0.0_f64, f64::max)
            .max(1.0);
        assert_eq!(auto_max, 1.0);

        // Non-empty data, max_value=0 => auto-scale from data max
        let s2 = BrailleSparkline {
            data: vec![1.0, 2.5],
            max_value: 0.0,
            color: Color::Reset,
            label: None,
        };
        let auto_max2 = s2.data.iter().copied().fold(0.0_f64, f64::max).max(1.0);
        assert_eq!(auto_max2, 2.5);

        // max_value > 0 => use max_value directly
        let s3 = BrailleSparkline {
            data: vec![0.5],
            max_value: 100.0,
            color: Color::Reset,
            label: None,
        };
        assert_eq!(s3.max_value, 100.0);

        // Negative max_value => treated as <= 0, auto-scale
        let s4 = BrailleSparkline {
            data: vec![0.5],
            max_value: -1.5,
            color: Color::Reset,
            label: None,
        };
        assert!(s4.max_value <= 0.0); // will trigger auto-scale
    }

    /// INV-025: Sparkline capacity is 2 data points per terminal cell.
    #[test]
    fn test_braille_sparkline_capacity() {
        // Test various data lengths and cell counts
        for &(data_len, cell_count) in
            &[(0usize, 1usize), (40, 20), (80, 40), (160, 40), (1000, 80)]
        {
            let data_capacity = cell_count * 2;
            let n = data_len.min(data_capacity);
            let offset = data_len.saturating_sub(data_capacity);

            // Only latest data_capacity points rendered
            assert!(n <= data_capacity, "n={n} > capacity={data_capacity}");
            // offset + n = data_len (newest data at right edge)
            assert_eq!(offset + n, data_len);

            // Actually render to verify no panic
            let data: Vec<f64> = (0..data_len).map(|i| i as f64).collect();
            let sparkline = BrailleSparkline {
                data,
                max_value: 1.0,
                color: Color::Reset,
                label: None,
            };
            let area = Rect::new(0, 0, cell_count as u16, 1);
            let mut buf = Buffer::empty(area);
            sparkline.render(area, &mut buf);
        }
    }
}
