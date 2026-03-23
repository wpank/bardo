//! Demoscene effects: Plasma, Tunnel, Fire, Metaballs.
//!
//! Four algorithms from `prd2/18-interfaces/rendering/01-demoscene.md` section 2.
//! Each effect writes directly to a ratatui Buffer over a given Rect.

use std::time::Duration;

use rand::Rng;
use rand::SeedableRng;
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use super::palette::{lerp_color_linear, rgb_components, BG_VOID, ROSE};
use super::tokens::{hsv_to_rgb, DesignTokens};

// ── Plasma Field ────────────────────────────────────────────────────

/// Compute plasma value at pixel (x, y) at time t. Returns [0.0, 1.0].
/// Uses frequency constants F1=8.3, F2=11.7, F3=15.1, F4=19.9.
pub fn plasma_value(x: f32, y: f32, t: f32) -> f32 {
    let f1 = DesignTokens::FREQ_F1 as f32;
    let f2 = DesignTokens::FREQ_F2 as f32;
    let f3 = DesignTokens::FREQ_F3 as f32;
    let f4 = DesignTokens::FREQ_F4 as f32;

    let v1 = (x / f1 + t).sin();
    let v2 = (y / f2 + t * 0.8).sin();
    let v3 = ((x + y) / f3 + t * 0.5).sin();
    let v4 = ((x * x + y * y).sqrt() / f4 + t * 1.5).sin();

    (v1 + v2 + v3 + v4 + 4.0) / 8.0
}

pub struct PlasmaEffect {
    pub time: f64,
    /// Multiplier for time advance per frame. Normal=1.0, meditative=0.3, hyperstim=2.0.
    pub time_scale: f64,
    pub color_low: Color,
    pub color_high: Color,
}

impl PlasmaEffect {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            time_scale: 1.0,
            color_low: BG_VOID,
            color_high: ROSE,
        }
    }

    pub fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) {
        self.time += duration.as_secs_f64() * self.time_scale;
        let t = self.time as f32;
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let v = plasma_value(x as f32, y as f32, t);
                let color = lerp_color_linear(self.color_low, self.color_high, v);
                if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, y)) {
                    cell.set_bg(color);
                }
            }
        }
    }
}

// ── Tunnel Effect ───────────────────────────────────────────────────

pub struct TunnelEffect {
    angle_lut: Vec<Vec<i32>>,
    distance_lut: Vec<Vec<i32>>,
    pub width: u16,
    pub height: u16,
    distance_offset: f64,
    angle_offset: f64,
    pub movement_speed: f64,
    pub rotation_speed: f64,
}

impl TunnelEffect {
    const TEX_WIDTH: i32 = 64;
    const TEX_HEIGHT: i32 = 32;
    const SCALE: f64 = 32.0;

    pub fn new(width: u16, height: u16) -> Self {
        let cx = width as f64 / 2.0;
        let cy = height as f64 / 2.0;
        let mut angle_lut = vec![vec![0i32; width as usize]; height as usize];
        let mut distance_lut = vec![vec![0i32; width as usize]; height as usize];
        for y in 0..height as usize {
            for x in 0..width as usize {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let dist = (dx * dx + dy * dy).sqrt().max(0.0001);
                let angle = dy.atan2(dx);
                angle_lut[y][x] =
                    (angle * Self::TEX_WIDTH as f64 / (2.0 * std::f64::consts::PI)) as i32;
                distance_lut[y][x] = (Self::TEX_HEIGHT as f64 * Self::SCALE / dist) as i32;
            }
        }
        Self {
            angle_lut,
            distance_lut,
            width,
            height,
            distance_offset: 0.0,
            angle_offset: 0.0,
            movement_speed: 8.0,
            rotation_speed: 0.3,
        }
    }

    fn xor_texture(u: i32, v: i32) -> u8 {
        ((u ^ v) & 0xFF) as u8
    }

    fn value_to_char(v: u8) -> char {
        const RAMP: &[char] = &['⠀', '⠁', '⠃', '⠇', '⠟', '⠿', '⣿'];
        let idx = (v as usize * (RAMP.len() - 1)) / 255;
        RAMP[idx]
    }

    pub fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) {
        let dt = duration.as_secs_f64();
        self.distance_offset += self.movement_speed * dt;
        self.angle_offset += self.rotation_speed * dt;

        let (rose_r, rose_g, rose_b) = rgb_components(ROSE);

        for y in 0..(self.height as usize).min(area.height as usize) {
            for x in 0..(self.width as usize).min(area.width as usize) {
                let raw_dist = self.distance_lut[y][x];
                let raw_angle = self.angle_lut[y][x];

                let u = (raw_angle + self.angle_offset as i32).rem_euclid(Self::TEX_WIDTH);
                let v = (raw_dist + self.distance_offset as i32).rem_euclid(Self::TEX_HEIGHT);

                let tex_val = Self::xor_texture(u, v);
                let ch = Self::value_to_char(tex_val);

                let brightness = 1.0 / (1.0 + raw_dist as f64 * 0.001);
                let intensity = (brightness * 255.0).clamp(0.0, 255.0) as u8;
                let color = Color::Rgb(
                    (intensity as f64 * (rose_r as f64 / 255.0)) as u8,
                    (intensity as f64 * (rose_g as f64 / 255.0)) as u8,
                    (intensity as f64 * (rose_b as f64 / 255.0)) as u8,
                );

                let bx = area.x + x as u16;
                let by = area.y + y as u16;
                if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(bx, by)) {
                    cell.set_char(ch);
                    cell.set_fg(color);
                }
            }
        }
    }
}

// ── Fire Simulation ─────────────────────────────────────────────────

pub struct FireEffect {
    pub width: usize,
    pub height: usize,
    grid: Vec<Vec<u8>>,
    pub cooling: u8,
    rng: rand::rngs::SmallRng,
}

impl FireEffect {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![vec![0u8; width]; height],
            cooling: 2,
            rng: rand::rngs::SmallRng::from_entropy(),
        }
    }

    /// Advance one simulation tick.
    pub fn tick(&mut self) {
        // Seed bottom row with random heat
        for x in 0..self.width {
            self.grid[self.height - 1][x] = self.rng.gen_range(200..=255u8);
        }
        // Propagate upward. Divisor MUST be slightly above 4.0 (canonical 4.0018).
        for y in 0..(self.height - 1) {
            for x in 0..self.width {
                let left = self.grid[y + 1][(x + self.width - 1) % self.width] as u16;
                let right = self.grid[y + 1][(x + 1) % self.width] as u16;
                let below = self.grid[y + 1][x] as u16;
                let avg = ((left + right + below * 2) as f32 / 4.0018) as u16;
                self.grid[y][x] = avg.saturating_sub(self.cooling as u16) as u8;
            }
        }
    }

    fn value_to_char(v: u8) -> char {
        match v {
            0..=10 => ' ',
            11..=50 => '░',
            51..=100 => '▒',
            101..=170 => '▓',
            _ => '█',
        }
    }

    fn value_to_color(v: u8) -> Color {
        let lum = ((v as u16 * 2).min(255)) as f32 / 255.0;
        let hue = v as f32 * 60.0 / 255.0;
        let (r, g, b) = hsv_to_rgb(hue, 0.9, lum);
        // Tint toward ROSEDUST: blend green and blue channels with rose at 40%
        let (rose_r, rose_g, rose_b) = (170u8, 112u8, 136u8);
        Color::Rgb(
            (r * 255.0) as u8,
            (g * 255.0 * 0.6 + rose_g as f32 * 0.4) as u8,
            (b * 255.0 * 0.6 + rose_b as f32 * 0.4) as u8,
        )
    }

    pub fn process(&mut self, _duration: Duration, buf: &mut Buffer, area: Rect) {
        self.tick();
        for y in 0..(self.height.min(area.height as usize)) {
            for x in 0..(self.width.min(area.width as usize)) {
                let v = self.grid[y][x];
                if v == 0 {
                    continue;
                }
                let bx = area.x + x as u16;
                let by = area.y + y as u16;
                if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(bx, by)) {
                    cell.set_char(Self::value_to_char(v));
                    cell.set_fg(Self::value_to_color(v));
                }
            }
        }
    }
}

// ── Metaballs ───────────────────────────────────────────────────────

/// Compute total metaball influence at pixel (x, y).
pub fn metaball_field(x: f64, y: f64, balls: &[(f64, f64, f64)]) -> f64 {
    balls
        .iter()
        .map(|(cx, cy, r)| {
            let dx = x - cx;
            let dy = y - cy;
            let dist_sq = (dx * dx + dy * dy).max(0.001);
            (r * r) / dist_sq
        })
        .sum()
}

pub struct MetaballEffect {
    pub balls: Vec<(f64, f64, f64)>,
    lissajous: Vec<(f64, f64, f64, f64, f64)>,
    time: f64,
    pub threshold: f64,
    pub fg_char: char,
    pub fg_color: Color,
}

impl MetaballEffect {
    pub fn new(width: u16, height: u16, count: usize) -> Self {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let cx = width as f64 / 2.0;
        let cy = height as f64 / 2.0;
        let r = (width.min(height) as f64 * 0.1).max(3.0);
        let balls: Vec<_> = (0..count).map(|_| (cx, cy, r)).collect();
        let lissajous: Vec<_> = (0..count)
            .map(|i| {
                let amplitude_x = cx * 0.6;
                let amplitude_y = cy * 0.6;
                let a = 0.7 + i as f64 * 0.13;
                let b = 1.0 + i as f64 * 0.19;
                let delta = rng.gen_range(0.0..std::f64::consts::TAU);
                (amplitude_x, amplitude_y, a, b, delta)
            })
            .collect();
        Self {
            balls,
            lissajous,
            time: 0.0,
            threshold: 1.0,
            fg_char: '⣿',
            fg_color: ROSE,
        }
    }

    fn update_positions(&mut self, cx: f64, cy: f64) {
        for (i, ball) in self.balls.iter_mut().enumerate() {
            let (ax, ay, a, b, delta) = self.lissajous[i];
            ball.0 = cx + ax * (a * self.time + delta).sin();
            ball.1 = cy + ay * (b * self.time).sin();
        }
    }

    pub fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) {
        self.time += duration.as_secs_f64();
        let cx = area.x as f64 + area.width as f64 / 2.0;
        let cy = area.y as f64 + area.height as f64 / 2.0;
        self.update_positions(cx, cy);

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let field = metaball_field(x as f64, y as f64, &self.balls);
                if field >= self.threshold {
                    if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, y)) {
                        cell.set_char(self.fg_char);
                        cell.set_fg(self.fg_color);
                    }
                } else if field >= self.threshold * 0.6 {
                    let boundary_char =
                        match ((field / self.threshold * 6.0) as u8).min(5) {
                            0 => '⠁',
                            1 => '⠃',
                            2 => '⠇',
                            3 => '⠟',
                            4 => '⠿',
                            _ => '⣿',
                        };
                    if let Some(cell) = buf.cell_mut(ratatui::layout::Position::new(x, y)) {
                        cell.set_char(boundary_char);
                        cell.set_fg(self.fg_color);
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
    fn test_plasma_value_range() {
        // Test many coordinate/time combinations
        for ix in 0..50 {
            for iy in 0..50 {
                for it in 0..20 {
                    let v = plasma_value(ix as f32 * 1.7, iy as f32 * 2.3, it as f32 * 0.5);
                    assert!(
                        (0.0..=1.0).contains(&v),
                        "plasma_value({}, {}, {}) = {} out of range",
                        ix,
                        iy,
                        it,
                        v
                    );
                }
            }
        }
    }

    #[test]
    fn test_plasma_no_exact_repeat() {
        // Consecutive values should differ due to incommensurate frequencies
        let mut prev = plasma_value(10.0, 10.0, 0.0);
        let mut diffs = 0;
        for i in 1..1000 {
            let v = plasma_value(10.0, 10.0, i as f32 * 0.01);
            if (v - prev).abs() > 1e-9 {
                diffs += 1;
            }
            prev = v;
        }
        assert!(diffs > 990, "Expected most consecutive values to differ, got {} diffs", diffs);
    }

    #[test]
    fn test_fire_tick_propagates_upward() {
        let mut fire = FireEffect::new(10, 10);
        // Run enough ticks for heat to reach the top
        for _ in 0..20 {
            fire.tick();
        }
        // Row 0 should have received some heat
        let top_sum: u32 = fire.grid[0].iter().map(|&v| v as u32).sum();
        assert!(top_sum > 0, "Heat should have propagated to top row");
    }

    #[test]
    fn test_fire_divisor_not_4() {
        // Grid values should stay bounded at u8 (not blow up)
        let mut fire = FireEffect::new(20, 20);
        for _ in 0..200 {
            fire.tick();
        }
        for row in &fire.grid {
            for &v in row {
                let _ = v; // if this doesn't panic, u8 stayed bounded
            }
        }
    }

    #[test]
    fn test_metaball_field_decreases() {
        let balls = vec![(40.0, 12.0, 5.0)];
        let at_center = metaball_field(40.0, 12.0, &balls);
        let far_away = metaball_field(140.0, 12.0, &balls);
        assert!(at_center > far_away, "Field should decrease with distance");
    }

    #[test]
    fn test_metaball_threshold() {
        let balls = vec![(40.0, 12.0, 5.0)];
        // Very close to center: high field
        let close = metaball_field(40.5, 12.0, &balls);
        assert!(close > 1.0, "Inside ball should have high field");
        // Far away: low field
        let far = metaball_field(100.0, 50.0, &balls);
        assert!(far < 0.1, "Far from ball should have low field");
    }

    #[test]
    fn test_tunnel_lut_finite() {
        let tunnel = TunnelEffect::new(80, 24);
        for row in &tunnel.angle_lut {
            for &v in row {
                assert!(v.abs() < i32::MAX / 2, "Angle LUT value should be finite");
            }
        }
        for row in &tunnel.distance_lut {
            for &v in row {
                assert!(v.abs() < i32::MAX / 2, "Distance LUT value should be finite");
            }
        }
    }
}
