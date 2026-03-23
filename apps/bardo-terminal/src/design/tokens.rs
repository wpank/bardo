//! Design tokens: spacing, typography, animation timing, border styles, phase degradation, PAD micro-shifts.
//!
//! From `prd2/18-interfaces/rendering/00-design-system.md` sections 2, 5, 7, 12, 13.

use ratatui::style::Color;

use super::palette::*;

// ── Vitality Phase ──────────────────────────────────────────────────
// Defined here because golem-mortality is still a shell. Once the mortality
// crate defines its own Phase enum, migrate callers to use that instead.

/// Lifecycle phase of a golem, driving palette degradation and atmospheric intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    Thriving,
    Stable,
    Conservation,
    Declining,
    Terminal,
}

// ── Design Tokens ───────────────────────────────────────────────────

pub struct DesignTokens;

impl DesignTokens {
    // ── Spacing (terminal cells) ────────────────────────────────
    pub const SPACE_XS: u16 = 1;
    pub const SPACE_SM: u16 = 2;
    pub const SPACE_MD: u16 = 4;
    pub const SPACE_LG: u16 = 8;
    pub const SPACE_XL: u16 = 16;

    // ── Border styles ───────────────────────────────────────────
    pub const BORDER_THIN: [char; 11] = ['─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼'];
    pub const BORDER_THICK: [char; 11] = ['═', '║', '╔', '╗', '╚', '╝', '╠', '╣', '╦', '╩', '╬'];
    pub const BORDER_HEAVY: [char; 11] = ['━', '┃', '┏', '┓', '┗', '┛', '┣', '┫', '┳', '┻', '╋'];
    pub const BORDER_ROUNDED: [char; 4] = ['╭', '╮', '╰', '╯'];
    pub const BORDER_DASHED: [char; 8] = ['┄', '┅', '┆', '┇', '┈', '┉', '┊', '┋'];
    pub const DIAGONAL: [char; 2] = ['╱', '╲'];

    // ── Vitality semantic colors ────────────────────────────────
    pub const VITALITY_THRIVING: Color = SUCCESS;
    pub const VITALITY_STABLE: Color = SUCCESS;
    pub const VITALITY_CONSERVATION: Color = WARNING;
    pub const VITALITY_DECLINING: Color = ROSE_DIM;
    pub const VITALITY_TERMINAL: Color = ROSE_BRIGHT;

    // ── Prediction calibration dots ─────────────────────────────
    pub const CALIBRATION_GOOD: Color = SUCCESS;
    pub const CALIBRATION_WARN: Color = WARNING;
    pub const CALIBRATION_BAD: Color = ROSE_BRIGHT;

    // ── Animation timing constants (ms) ─────────────────────────
    pub const BREATH_CYCLE_MIN_MS: u64 = 2000;
    pub const BREATH_CYCLE_MAX_MS: u64 = 4000;
    pub const PHOSPHOR_DECAY_MS: u64 = 5000;
    pub const PHOSPHOR_PERSIST_MS: u64 = 400;
    pub const PHOSPHOR_BLOOM_MS: u64 = 150;
    pub const PANEL_FOCUS_MS: u64 = 300;
    pub const DEATH_FLASH_MS: u64 = 200;
    pub const DEATH_FADE_MS: u64 = 2000;
    pub const DEATH_DISAPPEAR_MS: u64 = 5000;
    pub const DREAM_TRANSITION_MS: u64 = 3000;
    pub const BIRTH_COALESCE_MIN_MS: u64 = 10_000;
    pub const BIRTH_COALESCE_MAX_MS: u64 = 15_000;
    pub const TICK_FLASH_MS: u64 = 100;
    pub const SCANLINE_DRIFT_NORMAL_MS: u64 = 45_000;
    pub const SCANLINE_DRIFT_DEGRADED_MS: u64 = 12_500;
    pub const GLITCH_BAND_MIN_MS: u64 = 50;
    pub const GLITCH_BAND_MAX_MS: u64 = 150;
    pub const FRAGMENT_FADE_IN_MS: u64 = 2000;
    pub const FRAGMENT_HOLD_MIN_MS: u64 = 5000;
    pub const FRAGMENT_HOLD_MAX_MS: u64 = 8000;
    pub const FRAGMENT_FADE_OUT_MS: u64 = 3000;
    pub const FRAGMENT_INTERVAL_MIN_MS: u64 = 30_000;

    // ── Organic frequency constants (incommensurate ratios) ─────
    pub const FREQ_F1: f64 = 8.3;
    pub const FREQ_F2: f64 = 11.7;
    pub const FREQ_F3: f64 = 15.1;
    pub const FREQ_F4: f64 = 19.9;
}

// ── Phase Tokens ────────────────────────────────────────────────────

/// Per-phase palette degradation targets from section 2 of 00-design-system.md.
pub struct PhaseTokens;

impl PhaseTokens {
    /// Background void color target for this phase.
    pub fn bg_void(phase: Phase) -> Color {
        match phase {
            Phase::Thriving => Color::Rgb(6, 6, 8),
            Phase::Stable => Color::Rgb(6, 6, 8),
            Phase::Conservation => Color::Rgb(7, 7, 8),
            Phase::Declining => Color::Rgb(7, 7, 8),
            Phase::Terminal => Color::Rgb(10, 8, 8),
        }
    }

    /// Scanline strength for this phase (0.0-1.0).
    pub fn scanline_strength(phase: Phase) -> f32 {
        match phase {
            Phase::Thriving => 0.12,
            Phase::Stable => 0.14,
            Phase::Conservation => 0.20,
            Phase::Declining => 0.30,
            Phase::Terminal => 0.50,
        }
    }

    /// Noise density (fraction of cells) for this phase.
    pub fn noise_density(phase: Phase) -> f64 {
        match phase {
            Phase::Thriving => 0.003,
            Phase::Stable => 0.004,
            Phase::Conservation => 0.008,
            Phase::Declining => 0.015,
            Phase::Terminal => 0.020,
        }
    }

    /// Noise color for this phase.
    pub fn noise_color(phase: Phase) -> Color {
        match phase {
            Phase::Terminal => NOISE_WARM,
            _ => ROSE_DEEP,
        }
    }

    /// Vitality display color for this phase.
    pub fn vitality_color(phase: Phase) -> Color {
        match phase {
            Phase::Thriving => DesignTokens::VITALITY_THRIVING,
            Phase::Stable => DesignTokens::VITALITY_STABLE,
            Phase::Conservation => DesignTokens::VITALITY_CONSERVATION,
            Phase::Declining => DesignTokens::VITALITY_DECLINING,
            Phase::Terminal => DesignTokens::VITALITY_TERMINAL,
        }
    }
}

// ── PAD Micro-Shift ─────────────────────────────────────────────────

/// Applies PAD (Pleasure-Arousal-Dominance) vector micro-shifts to a base color.
///
/// PAD rules from section 2 of 00-design-system.md:
/// - Pleasure > 0: saturation +P*5%.  Pleasure < 0: saturation -|P|*5%.
/// - Arousal  > 0: brightness +A*3%.  Arousal  < 0: brightness -|A|*3%.
/// - Dominance > 0: hue shift warm +D*2 deg.  Dominance < 0: hue cool -|D|*2 deg.
///
/// Applied every frame; lerp rate 0.02 per frame (converges within ~150 frames / 2.5s).
pub struct PadMicroShift {
    pub pleasure: f32,
    pub arousal: f32,
    pub dominance: f32,
}

impl PadMicroShift {
    pub fn apply(&self, color: Color) -> Color {
        let Color::Rgb(r, g, b) = color else {
            return color;
        };
        let (h, s, l) = rgb_to_hsl(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let s2 = (s + self.pleasure * 0.05).clamp(0.0, 1.0);
        let l2 = (l + self.arousal * 0.03).clamp(0.0, 1.0);
        let h2 = (h + self.dominance * 2.0).rem_euclid(360.0);
        let (r2, g2, b2) = hsl_to_rgb(h2, s2, l2);
        Color::Rgb((r2 * 255.0) as u8, (g2 * 255.0) as u8, (b2 * 255.0) as u8)
    }
}

// ── Color conversion helpers ────────────────────────────────────────

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 1e-6 {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if (max - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 1e-6 {
        return (l, l, l);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h_norm = h / 360.0;

    let hue_to_rgb = |t: f32| -> f32 {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };

    (
        hue_to_rgb(h_norm + 1.0 / 3.0),
        hue_to_rgb(h_norm),
        hue_to_rgb(h_norm - 1.0 / 3.0),
    )
}

/// Convert HSV to RGB. h in [0, 360), s and v in [0, 1].
pub(crate) fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vitality_phase_to_color_all_phases() {
        let phases = [
            Phase::Thriving,
            Phase::Stable,
            Phase::Conservation,
            Phase::Declining,
            Phase::Terminal,
        ];
        for phase in phases {
            let color = PhaseTokens::vitality_color(phase);
            let (r, g, b) = super::super::palette::rgb_components(color);
            assert!(
                r > 0 || g > 0 || b > 0,
                "Vitality color for {:?} must not be black",
                phase
            );
        }
    }

    #[test]
    fn test_scanline_strength_monotone() {
        let phases = [
            Phase::Thriving,
            Phase::Stable,
            Phase::Conservation,
            Phase::Declining,
            Phase::Terminal,
        ];
        for pair in phases.windows(2) {
            let a = PhaseTokens::scanline_strength(pair[0]);
            let b = PhaseTokens::scanline_strength(pair[1]);
            assert!(
                a < b,
                "scanline_strength({:?})={} should be < scanline_strength({:?})={}",
                pair[0],
                a,
                pair[1],
                b
            );
        }
    }

    #[test]
    fn test_noise_density_monotone() {
        let phases = [
            Phase::Thriving,
            Phase::Stable,
            Phase::Conservation,
            Phase::Declining,
            Phase::Terminal,
        ];
        for pair in phases.windows(2) {
            let a = PhaseTokens::noise_density(pair[0]);
            let b = PhaseTokens::noise_density(pair[1]);
            assert!(
                a < b,
                "noise_density({:?})={} should be < noise_density({:?})={}",
                pair[0],
                a,
                pair[1],
                b
            );
        }
    }

    #[test]
    fn test_pad_microshift_clamps() {
        // Extreme PAD values should still produce valid RGB.
        let shift = PadMicroShift {
            pleasure: 1.0,
            arousal: 1.0,
            dominance: 1.0,
        };
        let result = shift.apply(ROSE);
        let Color::Rgb(r, g, b) = result else {
            panic!("Expected Rgb");
        };
        // Just check it doesn't panic and produces valid u8 values
        let _ = (r, g, b);

        let shift_neg = PadMicroShift {
            pleasure: -1.0,
            arousal: -1.0,
            dominance: -1.0,
        };
        let result_neg = shift_neg.apply(ROSE);
        let Color::Rgb(_, _, _) = result_neg else {
            panic!("Expected Rgb");
        };
    }

    #[test]
    fn test_pad_microshift_non_rgb_passthrough() {
        let shift = PadMicroShift {
            pleasure: 0.5,
            arousal: 0.5,
            dominance: 0.5,
        };
        assert_eq!(shift.apply(Color::Yellow), Color::Yellow);
    }

    #[test]
    fn test_hsl_roundtrip() {
        // Rose: Rgb(170, 112, 136)
        let (h, s, l) = rgb_to_hsl(170.0 / 255.0, 112.0 / 255.0, 136.0 / 255.0);
        let (r, g, b) = hsl_to_rgb(h, s, l);
        assert!((r * 255.0 - 170.0).abs() < 2.0);
        assert!((g * 255.0 - 112.0).abs() < 2.0);
        assert!((b * 255.0 - 136.0).abs() < 2.0);
    }

    #[test]
    fn test_hsv_to_rgb_red() {
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!(b.abs() < 0.01);
    }
}
