//! Sub-pixel braille canvas engine.
//!
//! Unicode U+2800-U+28FF provides a 2x4 dot grid per cell (8 sub-pixels).
//! An 80x24 terminal becomes 160x96 effective resolution.

use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}};

/// Sub-pixel braille canvas. One BrailleCanvas maps to a rectangle of terminal cells.
/// The pixel grid is `width_chars*2` wide by `height_chars*4` tall.
pub struct BrailleCanvas {
    pub width_chars: u32,
    pub height_chars: u32,
    /// Flat pixel grid: index = y * (width_chars*2) + x, value = on/off.
    pixels: Vec<bool>,
    pub fg: Color,
}

impl BrailleCanvas {
    pub fn new(width_chars: u32, height_chars: u32, fg: Color) -> Self {
        let pixel_w = width_chars * 2;
        let pixel_h = height_chars * 4;
        Self {
            width_chars,
            height_chars,
            pixels: vec![false; (pixel_w * pixel_h) as usize],
            fg,
        }
    }

    /// Clear all pixels.
    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|p| *p = false);
    }

    /// Pixel width of the canvas.
    pub fn pixel_width(&self) -> u32 {
        self.width_chars * 2
    }

    /// Pixel height of the canvas.
    pub fn pixel_height(&self) -> u32 {
        self.height_chars * 4
    }

    /// Set a single pixel at pixel coordinates (px, py).
    /// Silently ignores out-of-bounds coordinates.
    pub fn set_pixel(&mut self, px: u32, py: u32, on: bool) {
        let pixel_w = self.width_chars * 2;
        if px < pixel_w && py < self.height_chars * 4 {
            self.pixels[(py * pixel_w + px) as usize] = on;
        }
    }

    /// Set a pixel by floating-point normalized coordinates (x in [0,1), y in [0,1)).
    pub fn set_norm(&mut self, nx: f32, ny: f32, on: bool) {
        let px = (nx * (self.width_chars * 2) as f32).floor() as u32;
        let py = (ny * (self.height_chars * 4) as f32).floor() as u32;
        self.set_pixel(px, py, on);
    }

    /// Encode the 2x4 block at cell (cx, cy) into a braille character.
    fn encode_cell(&self, cx: u32, cy: u32) -> char {
        let pixel_w = self.width_chars * 2;
        let mut bits: u8 = 0;
        let px = cx * 2;
        let py = cy * 4;
        let dot = |dx: u32, dy: u32| -> bool {
            let idx = ((py + dy) * pixel_w + (px + dx)) as usize;
            self.pixels.get(idx).copied().unwrap_or(false)
        };
        // Unicode Braille dot-to-bit mapping:
        //   dot1=bit0 (0,0), dot2=bit1 (0,1), dot3=bit2 (0,2)
        //   dot4=bit3 (1,0), dot5=bit4 (1,1), dot6=bit5 (1,2)
        //   dot7=bit6 (0,3), dot8=bit7 (1,3)
        if dot(0, 0) { bits |= 1 << 0; }
        if dot(0, 1) { bits |= 1 << 1; }
        if dot(0, 2) { bits |= 1 << 2; }
        if dot(1, 0) { bits |= 1 << 3; }
        if dot(1, 1) { bits |= 1 << 4; }
        if dot(1, 2) { bits |= 1 << 5; }
        if dot(0, 3) { bits |= 1 << 6; }
        if dot(1, 3) { bits |= 1 << 7; }
        char::from_u32(0x2800 | bits as u32).unwrap_or('⠀')
    }

    /// Write all cells to a ratatui Buffer starting at `origin`.
    pub fn render_to_buffer(&self, buf: &mut Buffer, origin: Rect) {
        let style = Style::default().fg(self.fg);
        for cy in 0..self.height_chars {
            for cx in 0..self.width_chars {
                let x = origin.x + cx as u16;
                let y = origin.y + cy as u16;
                if x < origin.x + origin.width && y < origin.y + origin.height {
                    let ch = self.encode_cell(cx, cy);
                    if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, y)) {
                        cell.set_char(ch);
                        cell.set_style(style);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_braille_empty_cell_is_u2800() {
        let canvas = BrailleCanvas::new(1, 1, Color::White);
        let ch = canvas.encode_cell(0, 0);
        assert_eq!(ch, '⠀');
        assert_eq!(ch as u32, 0x2800);
    }

    #[test]
    fn test_braille_full_cell_is_u28ff() {
        let mut canvas = BrailleCanvas::new(1, 1, Color::White);
        for py in 0..4 {
            for px in 0..2 {
                canvas.set_pixel(px, py, true);
            }
        }
        let ch = canvas.encode_cell(0, 0);
        assert_eq!(ch, '⣿');
        assert_eq!(ch as u32, 0x28FF);
    }

    #[test]
    fn test_braille_single_top_left_dot_is_u2801() {
        let mut canvas = BrailleCanvas::new(1, 1, Color::White);
        canvas.set_pixel(0, 0, true);
        let ch = canvas.encode_cell(0, 0);
        assert_eq!(ch as u32, 0x2801);
    }

    #[test]
    fn test_braille_canvas_pixel_grid_size() {
        let canvas = BrailleCanvas::new(10, 5, Color::White);
        assert_eq!(canvas.pixels.len(), (10 * 2 * 5 * 4) as usize);
        assert_eq!(canvas.pixel_width(), 20);
        assert_eq!(canvas.pixel_height(), 20);
    }

    #[test]
    fn test_braille_set_pixel_out_of_bounds_safe() {
        let mut canvas = BrailleCanvas::new(1, 1, Color::White);
        // These should not panic
        canvas.set_pixel(100, 100, true);
        canvas.set_pixel(u32::MAX, u32::MAX, true);
        canvas.set_pixel(2, 0, true); // just past width
        canvas.set_pixel(0, 4, true); // just past height
    }

    #[test]
    fn test_braille_set_norm_maps_correctly() {
        let mut canvas = BrailleCanvas::new(2, 2, Color::White);
        // (0.0, 0.0) -> pixel (0, 0)
        canvas.set_norm(0.0, 0.0, true);
        assert!(canvas.pixels[0]);

        // (0.5, 0.5) -> pixel (2, 4) on a 4x8 pixel grid
        canvas.clear();
        canvas.set_norm(0.5, 0.5, true);
        let idx = (4 * 4 + 2) as usize; // py=4, pixel_w=4, px=2
        assert!(canvas.pixels[idx]);
    }

    #[test]
    fn test_braille_encode_range() {
        // All possible bit patterns should produce valid braille chars
        for bits in 0u32..=255 {
            let ch = char::from_u32(0x2800 | bits).unwrap();
            assert!(ch as u32 >= 0x2800);
            assert!(ch as u32 <= 0x28FF);
        }
    }

    #[test]
    fn test_braille_canvas_render_matches_expected() {
        // 2x1 canvas: first cell has top-left dot, second cell empty
        let mut canvas = BrailleCanvas::new(2, 1, Color::White);
        canvas.set_pixel(0, 0, true); // cell 0: top-left dot

        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);
        canvas.render_to_buffer(&mut buf, area);

        let cell0 = buf.cell(ratatui::layout::Position::new(0, 0)).unwrap();
        assert_eq!(cell0.symbol(), "\u{2801}"); // dot at (0,0) = bit 0

        let cell1 = buf.cell(ratatui::layout::Position::new(1, 0)).unwrap();
        assert_eq!(cell1.symbol(), "\u{2800}"); // empty
    }
}
