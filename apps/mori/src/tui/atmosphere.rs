use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::color::hsv_to_rgb;
use super::effects_config::EffectsConfig;
use super::math::Vec2;
use super::postfx;
use super::postfx_pipeline;
use super::tabs::Tab;

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f64,
    max_life: f64,
    hue: f64,
    size: f64,
    ch: char,
    color: Color,
}

#[derive(Clone, Copy)]
enum ForceField {
    Gravity(f64),
    Drag(f64),
    Radial { cx: f64, cy: f64, strength: f64 },
    Wind(Vec2),
}

pub struct Atmosphere {
    frame_count: u64,
    elapsed: f64,
    dt: f64,
    last_frame: Instant,
    heartbeat_phase: f64,
    breathing_phase: f64,
    particles: Vec<Particle>,
    emitter_accum: f64,
    effects_config: EffectsConfig,
    rng: fastrand::Rng,
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self::new()
    }
}

const BRAILLE: &[char] = &[
    '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}',
    '\u{2807}', '\u{280F}',
];
const ETHEREAL: &[char] = &[
    '\u{2727}', '\u{00B7}', '\u{00B0}', '\u{2726}', '\u{2218}', '\u{22C6}', '\u{2736}', '\u{274B}',
];
const PARTICLE_CAP: usize = 500;

impl Atmosphere {
    const HEARTBEAT_PERIOD_FRAMES: f64 = 60.0;

    pub fn new() -> Self {
        Self {
            frame_count: 0,
            elapsed: 0.0,
            dt: 1.0 / 30.0,
            last_frame: Instant::now(),
            heartbeat_phase: 0.0,
            breathing_phase: 0.0,
            particles: Vec::with_capacity(PARTICLE_CAP),
            emitter_accum: 0.0,
            effects_config: EffectsConfig::default(),
            rng: fastrand::Rng::new(),
        }
    }

    pub fn spinner(&self) -> char {
        BRAILLE[(self.frame_count / 3) as usize % BRAILLE.len()]
    }

    pub fn spinner_ethereal(&self) -> char {
        ETHEREAL[(self.frame_count / 4) as usize % ETHEREAL.len()]
    }

    pub fn heartbeat(&self) -> f64 {
        1.0 + 0.05 * self.heartbeat_phase.sin()
    }

    pub fn shimmer(&self) -> f64 {
        let phase = (self.frame_count as f64 / 8.0).sin();
        1.0 + phase * 0.10
    }

    pub fn frame(&self) -> u64 {
        self.frame_count
    }

    pub fn breathing_brightness(&self) -> f64 {
        // 0.88-1.0 range, ~5.2s period
        0.94 + 0.06 * (self.breathing_phase).sin()
    }

    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    pub fn effects_config(&self) -> &EffectsConfig {
        &self.effects_config
    }

    pub fn effects_config_mut(&mut self) -> &mut EffectsConfig {
        &mut self.effects_config
    }

    /// Exponential decay lerp: smoothly moves `current` toward `target`.
    /// Rate controls speed (higher = faster). Uses fixed dt approximation.
    pub fn lerp_toward(current: f64, target: f64, rate: f64) -> f64 {
        current + (target - current) * (1.0 - (-rate * 0.033).exp())
    }

    /// Phosphor decay: returns brightness multiplier (1.0 = just changed, 0.0 = fully decayed).
    /// frames_since_change is how many frames ago the value last changed.
    pub fn phosphor_decay(frames_since_change: u64) -> f64 {
        let t = frames_since_change as f64 / 30.0;
        (-t * 4.0).exp()
    }

    pub fn tick(&mut self) {
        self.tick_with_degraded(false);
    }

    /// Tick with optional degraded mode that skips particle physics.
    pub fn tick_with_degraded(&mut self, degraded: bool) {
        let now = Instant::now();
        self.dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;

        // Clamp dt to avoid huge jumps on lag
        let dt = self.dt.min(0.1);

        self.frame_count = self.frame_count.wrapping_add(1);
        self.elapsed += dt;
        self.heartbeat_phase =
            (self.frame_count as f64 / Self::HEARTBEAT_PERIOD_FRAMES) * std::f64::consts::TAU;
        // ~5.2s period breathing
        self.breathing_phase = self.elapsed * (std::f64::consts::TAU / 5.2);

        if !degraded {
            self.update_particles(dt);
        }
    }

    pub fn apply(&self, buf: &mut Buffer, area: Rect, active_tab: &Tab) {
        // Ambient fill on empty cells
        if self.effects_config.ambient_fill {
            postfx::ambient_fill(area, buf, self.elapsed);
        }

        // Per-tab postfx pipeline
        postfx_pipeline::apply_pipeline(
            active_tab,
            area,
            buf,
            self.elapsed,
            self.frame_count,
            &self.effects_config,
        );

        // Render particles on top
        if self.effects_config.particles {
            self.render_particles(buf, area);
        }
    }

    pub fn spawn_burst(&mut self, x: f32, y: f32, count: usize) {
        let remaining = PARTICLE_CAP.saturating_sub(self.particles.len());
        let count = count.min(remaining);
        for i in 0..count {
            let angle = (i as f64 / count as f64) * std::f64::consts::TAU;
            let speed = 0.3 + (i as f64 * 0.1) % 0.5;
            let ch = ETHEREAL[i % ETHEREAL.len()];
            let hue = (self.rng.f64() * 60.0 + 320.0) % 360.0; // rose-ish
            let (r, g, b) = hsv_to_rgb(hue, 0.6, 0.8);
            self.particles.push(Particle {
                pos: Vec2::new(x as f64, y as f64),
                vel: Vec2::from_polar(angle, speed),
                life: 1.0,
                max_life: 1.0,
                hue,
                size: 1.0,
                ch,
                color: Color::Rgb(r, g, b),
            });
        }
    }

    fn update_particles(&mut self, dt: f64) {
        let forces = [ForceField::Gravity(0.6), ForceField::Drag(0.98)];

        for p in &mut self.particles {
            // Apply forces
            for force in &forces {
                match *force {
                    ForceField::Gravity(g) => {
                        p.vel.y += g * dt;
                    }
                    ForceField::Drag(d) => {
                        p.vel = p.vel * d;
                    }
                    ForceField::Radial { cx, cy, strength } => {
                        let dir = Vec2::new(cx - p.pos.x, cy - p.pos.y);
                        let dist = dir.length().max(0.1);
                        let force = dir.normalized() * (strength / (dist * dist)) * dt;
                        p.vel += force;
                    }
                    ForceField::Wind(w) => {
                        p.vel += w * dt;
                    }
                }
            }

            p.pos += p.vel * (dt * 30.0); // Scale to roughly match frame-based movement
            p.life -= dt * 1.2; // ~0.83s lifetime

            // Fade color with life
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let (r, g, b) = hsv_to_rgb(p.hue, 0.6, 0.8 * alpha);
            p.color = Color::Rgb(r, g, b);
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn render_particles(&self, buf: &mut Buffer, area: Rect) {
        for p in &self.particles {
            let px = p.pos.x.round() as u16;
            let py = p.pos.y.round() as u16;
            if px >= area.x && px < area.x + area.width && py >= area.y && py < area.y + area.height
            {
                let cell = &mut buf[(px, py)];
                cell.set_char(p.ch);
                cell.set_style(cell.style().fg(p.color));
            }
        }
    }
}
