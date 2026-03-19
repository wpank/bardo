//! Pheromone heatmap widget with a Viridis-inspired gradient.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::palette::BLOCK_FULL;

const VIRIDIS: [(u8, u8, u8); 7] = [
    (68, 1, 84),
    (72, 40, 120),
    (62, 83, 160),
    (49, 126, 157),
    (53, 183, 121),
    (149, 216, 64),
    (253, 231, 37),
];

fn in_bounds(buf: &Buffer, x: u16, y: u16) -> bool {
    let area = buf.area();
    x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom()
}

fn lerp_u8(start: u8, end: u8, t: f64) -> u8 {
    (start as f64 + (end as f64 - start as f64) * t).round() as u8
}

fn viridis_color(value: f64) -> Color {
    let clamped = value.clamp(0.0, 1.0);
    let scaled = clamped * 6.0;
    let index = scaled.floor() as usize;
    let t = scaled - index as f64;
    let (r0, g0, b0) = VIRIDIS[index.min(6)];
    let (r1, g1, b1) = VIRIDIS[(index + 1).min(6)];

    Color::Rgb(lerp_u8(r0, r1, t), lerp_u8(g0, g1, t), lerp_u8(b0, b1, t))
}

fn layer_tint(base: Color, layer: PheromoneLayer, value: f64) -> Color {
    match (base, layer) {
        (Color::Rgb(r, g, b), PheromoneLayer::Threat) => Color::Rgb(
            r.saturating_add(40),
            g.saturating_sub(20),
            b.saturating_sub(20),
        ),
        (Color::Rgb(r, g, b), PheromoneLayer::Wisdom) => {
            let t = (1.0 - value).clamp(0.0, 0.5) as f32;
            let blend = |component: u8, target: u8| {
                (component as f32 * (1.0 - t) + target as f32 * t).round() as u8
            };
            Color::Rgb(blend(r, 88), blend(g, 88), blend(b, 120))
        }
        _ => base,
    }
}

/// Pheromone field layer tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PheromoneLayer {
    /// Threat field.
    Threat,
    /// Opportunity field.
    Opportunity,
    /// Wisdom field.
    Wisdom,
}

/// Two-dimensional pheromone heatmap.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PheromoneHeatmap {
    /// Grid[row][col] values.
    pub(crate) grid: Vec<Vec<f64>>,
    /// Requested grid width for layout hints.
    pub(crate) width: u16,
    /// Requested grid height for layout hints.
    pub(crate) height: u16,
    /// Layer tint.
    pub(crate) layer: PheromoneLayer,
    /// TODO Plan 70a: replace with live pheromone pulse data from golem-coordination.
    pub(crate) pulse_cells: Vec<(usize, usize)>,
}

impl Widget for PheromoneHeatmap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let rows = self.grid.len().min(area.height as usize);
        let cols = self
            .grid
            .first()
            .map(|row| row.len().min(area.width as usize))
            .unwrap_or(0);

        for row in 0..rows {
            for col in 0..cols {
                let x = area.x + col as u16;
                let y = area.y + row as u16;
                if !in_bounds(buf, x, y) {
                    continue;
                }

                let value = self.grid[row].get(col).copied().unwrap_or(0.0);
                let color = if self.pulse_cells.iter().any(|&(r, c)| r == row && c == col) {
                    Color::Rgb(255, 255, 255)
                } else {
                    layer_tint(viridis_color(value), self.layer, value)
                };

                let cell = buf.get_mut(x, y);
                cell.set_char(BLOCK_FULL);
                cell.set_style(Style::default().fg(color).bg(color));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viridis_interpolates_endpoints() {
        assert_eq!(viridis_color(0.0), Color::Rgb(68, 1, 84));
        assert_eq!(viridis_color(1.0), Color::Rgb(253, 231, 37));
    }

    #[test]
    fn threat_layer_adds_rose_bias() {
        assert_eq!(
            layer_tint(Color::Rgb(10, 100, 120), PheromoneLayer::Threat, 0.5),
            Color::Rgb(50, 80, 100)
        );
    }
}
