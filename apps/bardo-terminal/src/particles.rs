//! Character-based particle system for cinematic terminal effects.
//!
//! Particles track sub-character `f32` positions but render to the nearest cell.
//! Visual smoothness comes from character diversity in each preset's `char_set`
//! and velocity accumulation across ticks.

use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use crate::palette::{BONE, DREAM, ROSE, ROSE_BRIGHT, ROSE_DIM, SUCCESS, WARNING};

/// A single particle in the system.
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime_ticks: u32,
    pub max_lifetime: u32,
    pub ch: char,
    pub color: Color,
}

impl Particle {
    pub fn is_alive(&self) -> bool {
        self.lifetime_ticks > 0
    }

    /// Fraction of life remaining: 1.0 = just spawned, 0.0 = about to die.
    pub fn life_fraction(&self) -> f32 {
        if self.max_lifetime == 0 {
            return 0.0;
        }
        self.lifetime_ticks as f32 / self.max_lifetime as f32
    }
}

/// Spawn configuration for an emitter.
pub struct EmitterConfig {
    pub origin_x: u16,
    pub origin_y: u16,
    pub char_set: Vec<char>,
    pub initial_speed: f32,
    pub gravity: f32,
    pub spread_angle_deg: f32,
    pub base_angle_deg: f32,
    pub count: u32,
    pub lifetime_ticks: u32,
}

/// Named presets. Each maps to a specific EmitterConfig.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitterPreset {
    DeathDissolution,
    AchievementSpark,
    TradeSuccess,
    TradeFailure,
    DreamOnset,
    BloodstainMark,
}

impl EmitterPreset {
    pub fn config(&self, origin_x: u16, origin_y: u16) -> EmitterConfig {
        match self {
            EmitterPreset::DeathDissolution => EmitterConfig {
                origin_x,
                origin_y,
                char_set: vec!['█', '▓', '▒', '░', '⣿', '⡀', '·', '∙', '.'],
                initial_speed: 0.35,
                gravity: 0.008,
                spread_angle_deg: 360.0,
                base_angle_deg: 270.0,
                count: 64,
                lifetime_ticks: 90,
            },
            EmitterPreset::AchievementSpark => EmitterConfig {
                origin_x,
                origin_y,
                char_set: vec!['✦', '✧', '★', '☆', '◆', '◇', '·'],
                initial_speed: 0.5,
                gravity: 0.012,
                spread_angle_deg: 360.0,
                base_angle_deg: 0.0,
                count: 32,
                lifetime_ticks: 60,
            },
            EmitterPreset::TradeSuccess => EmitterConfig {
                origin_x,
                origin_y,
                char_set: vec!['↑', '↟', '▲', '△', '◦', '·'],
                initial_speed: 0.25,
                gravity: -0.005,
                spread_angle_deg: 90.0,
                base_angle_deg: 270.0,
                count: 16,
                lifetime_ticks: 45,
            },
            EmitterPreset::TradeFailure => EmitterConfig {
                origin_x,
                origin_y,
                char_set: vec!['↓', '↡', '▼', '▽', '◦', '·'],
                initial_speed: 0.2,
                gravity: 0.015,
                spread_angle_deg: 90.0,
                base_angle_deg: 90.0,
                count: 16,
                lifetime_ticks: 45,
            },
            EmitterPreset::DreamOnset => EmitterConfig {
                origin_x,
                origin_y,
                char_set: vec!['·', '∙', '○', '◦', '⊙', '•'],
                initial_speed: 0.06,
                gravity: -0.002,
                spread_angle_deg: 360.0,
                base_angle_deg: 0.0,
                count: 24,
                lifetime_ticks: 120,
            },
            EmitterPreset::BloodstainMark => EmitterConfig {
                origin_x,
                origin_y,
                char_set: vec!['│', '┊', '╎', '▏', '·'],
                initial_speed: 0.05,
                gravity: 0.02,
                spread_angle_deg: 20.0,
                base_angle_deg: 90.0,
                count: 8,
                lifetime_ticks: 75,
            },
        }
    }

    /// Color palette for this preset, varying by life_fraction.
    pub fn color(&self, life_fraction: f32) -> Color {
        match self {
            EmitterPreset::DeathDissolution => {
                if life_fraction > 0.5 {
                    ROSE
                } else {
                    ROSE_DIM
                }
            }
            EmitterPreset::AchievementSpark => {
                if life_fraction > 0.6 {
                    BONE
                } else {
                    WARNING
                }
            }
            EmitterPreset::TradeSuccess => SUCCESS,
            EmitterPreset::TradeFailure => ROSE_BRIGHT,
            EmitterPreset::DreamOnset => DREAM,
            EmitterPreset::BloodstainMark => ROSE_BRIGHT,
        }
    }

    /// Character selection based on life_fraction (dense near birth, sparse near death).
    pub fn char_for_life(&self, config: &EmitterConfig, life_fraction: f32) -> char {
        let idx = ((1.0 - life_fraction) * (config.char_set.len() - 1) as f32) as usize;
        config.char_set[idx.min(config.char_set.len() - 1)]
    }
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    max_particles: usize,
    presets: Vec<EmitterPreset>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(256),
            max_particles: 512,
            presets: Vec::with_capacity(256),
        }
    }

    pub fn with_max_particles(max: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max.min(512)),
            max_particles: max,
            presets: Vec::with_capacity(max.min(512)),
        }
    }

    /// Spawn particles from a preset at (x, y).
    pub fn emit(&mut self, preset: EmitterPreset, x: u16, y: u16) {
        let config = preset.config(x, y);
        self.spawn_from_config(preset, &config);
    }

    fn spawn_from_config(&mut self, preset: EmitterPreset, config: &EmitterConfig) {
        for i in 0..config.count {
            // Evict oldest if at cap
            if self.particles.len() >= self.max_particles {
                self.particles.remove(0);
                self.presets.remove(0);
            }

            let spread_rad = config.spread_angle_deg.to_radians();
            let base_rad = config.base_angle_deg.to_radians();

            let angle = if config.count > 1 {
                base_rad - spread_rad / 2.0 + spread_rad * (i as f32 / (config.count - 1) as f32)
            } else {
                base_rad
            };

            // Deterministic jitter: +/-10% of initial_speed
            let jitter = 1.0 + (i as f32 * 0.031 % 0.2) - 0.1;
            let speed = config.initial_speed * jitter;

            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;

            let p = Particle {
                x: config.origin_x as f32,
                y: config.origin_y as f32,
                vx,
                vy,
                lifetime_ticks: config.lifetime_ticks,
                max_lifetime: config.lifetime_ticks,
                ch: config.char_set[0],
                color: preset.color(1.0),
            };
            self.particles.push(p);
            self.presets.push(preset);
        }
    }

    /// Advance all particles one tick: apply velocity, gravity, decrement lifetime.
    /// Dead particles are removed.
    pub fn tick(&mut self) {
        for (p, preset) in self.particles.iter_mut().zip(self.presets.iter()) {
            p.x += p.vx;
            p.y += p.vy;
            let config = preset.config(0, 0);
            p.vy += config.gravity;
            p.lifetime_ticks = p.lifetime_ticks.saturating_sub(1);

            let life_frac = p.life_fraction();
            p.color = preset.color(life_frac);
            p.ch = preset.char_for_life(&config, life_frac);
        }

        // Remove dead particles
        let mut i = 0;
        while i < self.particles.len() {
            if !self.particles[i].is_alive() {
                self.particles.remove(i);
                self.presets.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Render all living particles into the ratatui buffer.
    /// Particles outside `area` are clipped.
    pub fn render(&self, buf: &mut Buffer, area: Rect) {
        use ratatui::style::Style;

        for p in &self.particles {
            let col = p.x.round() as i32;
            let row = p.y.round() as i32;

            if col < area.x as i32
                || col >= (area.x + area.width) as i32
                || row < area.y as i32
                || row >= (area.y + area.height) as i32
            {
                continue;
            }

            let cell = buf.get_mut(col as u16, row as u16);
            cell.set_symbol(&p.ch.to_string());
            cell.set_style(Style::default().fg(p.color));
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_system_tick_decrements_lifetime() {
        let mut sys = ParticleSystem::new();
        sys.emit(EmitterPreset::DreamOnset, 10, 10);
        let initial = sys.particle_count();
        assert!(initial > 0);

        // Tick enough times to kill all particles (DreamOnset lifetime = 120)
        for _ in 0..120 {
            sys.tick();
        }
        assert_eq!(sys.particle_count(), 0);
    }

    #[test]
    fn test_particle_system_dead_particles_removed() {
        let mut sys = ParticleSystem::with_max_particles(64);
        // BloodstainMark has count=8, lifetime=75
        sys.emit(EmitterPreset::BloodstainMark, 5, 5);
        assert_eq!(sys.particle_count(), 8);

        for _ in 0..75 {
            sys.tick();
        }
        assert!(sys.is_empty());
    }

    #[test]
    fn test_emitter_preset_death_dissolution_spawns_particles() {
        let mut sys = ParticleSystem::new();
        sys.emit(EmitterPreset::DeathDissolution, 40, 12);
        assert_eq!(sys.particle_count(), 64);
    }

    #[test]
    fn test_emitter_cap_evicts_oldest() {
        let mut sys = ParticleSystem::with_max_particles(4);
        // BloodstainMark emits 8, but cap is 4
        sys.emit(EmitterPreset::BloodstainMark, 5, 5);
        assert_eq!(sys.particle_count(), 4);
    }

    #[test]
    fn test_particle_render_clips_outside_area() {
        let mut sys = ParticleSystem::new();
        // Manually push a particle way outside any reasonable area
        sys.particles.push(Particle {
            x: 999.0,
            y: 999.0,
            vx: 0.0,
            vy: 0.0,
            lifetime_ticks: 10,
            max_lifetime: 10,
            ch: '*',
            color: Color::White,
        });
        sys.presets.push(EmitterPreset::DreamOnset);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        sys.render(&mut buf, area); // must not panic
    }

    #[test]
    fn test_char_for_life_dense_at_birth() {
        let preset = EmitterPreset::DeathDissolution;
        let config = preset.config(0, 0);
        let ch = preset.char_for_life(&config, 1.0);
        assert_eq!(ch, config.char_set[0]);
    }

    #[test]
    fn test_char_for_life_sparse_at_death() {
        let preset = EmitterPreset::DeathDissolution;
        let config = preset.config(0, 0);
        let ch = preset.char_for_life(&config, 0.0);
        assert_eq!(ch, *config.char_set.last().unwrap());
    }
}
