//! Atmospheric stack: 8-layer post-processor applied after widget rendering.
//!
//! Each layer processes the ratatui Buffer in order. Layers can be individually
//! enabled/disabled and configured.

use rand::SeedableRng;
use ratatui::{buffer::Buffer, layout::Rect};

use super::nerv;
use super::tokens::Phase;

// ── Layer Structs ──────────────────────────────────────────────────

pub struct NoiseFloorLayer {
    pub enabled: bool,
}

pub struct ScanlineLayer {
    pub enabled: bool,
    pub phase_offset: bool,
}

pub struct PlasmaBgLayer {
    pub enabled: bool,
}

pub struct ParticleOverlayLayer {
    pub enabled: bool,
}

pub struct NervChromeLayer {
    pub enabled: bool,
}

pub struct TransitionBlendLayer {
    pub enabled: bool,
}

pub struct PhosphorDecayLayer {
    pub enabled: bool,
}

pub struct PhaseDegradationLayer {
    pub enabled: bool,
}

// ── Atmospheric Stack ──────────────────────────────────────────────

pub struct AtmosphericStack {
    pub noise_floor: NoiseFloorLayer,
    pub scanline: ScanlineLayer,
    pub plasma_bg: PlasmaBgLayer,
    pub particle_overlay: ParticleOverlayLayer,
    pub nerv_chrome: NervChromeLayer,
    pub transition_blend: TransitionBlendLayer,
    pub phosphor_decay: PhosphorDecayLayer,
    pub phase_degradation: PhaseDegradationLayer,
}

impl AtmosphericStack {
    pub fn new() -> Self {
        Self {
            noise_floor: NoiseFloorLayer { enabled: true },
            scanline: ScanlineLayer {
                enabled: true,
                phase_offset: false,
            },
            plasma_bg: PlasmaBgLayer { enabled: false },
            particle_overlay: ParticleOverlayLayer { enabled: false },
            nerv_chrome: NervChromeLayer { enabled: false },
            transition_blend: TransitionBlendLayer { enabled: false },
            phosphor_decay: PhosphorDecayLayer { enabled: false },
            phase_degradation: PhaseDegradationLayer { enabled: false },
        }
    }

    pub fn process(
        &self,
        buf: &mut Buffer,
        area: Rect,
        phase: Phase,
        rng: &mut rand::rngs::SmallRng,
    ) {
        // Layer 1: Noise floor
        if self.noise_floor.enabled {
            nerv::render_background_noise(area, buf, rng, phase);
        }

        // Layer 2: Scanlines
        if self.scanline.enabled {
            let strength = super::tokens::PhaseTokens::scanline_strength(phase) as f64;
            nerv::render_scanlines(area, buf, self.scanline.phase_offset, strength);
        }

        // Layer 3: Plasma background (stub)
        if self.plasma_bg.enabled {
            // Wired when PlasmaEffect integration is needed
        }

        // Layer 4: Particle overlay (stub)
        if self.particle_overlay.enabled {
            // Wired when particle system integration is needed
        }

        // Layer 5: NERV chrome (stub)
        if self.nerv_chrome.enabled {
            // Wired when NervChrome integration is needed
        }

        // Layer 6: Transition blend (stub)
        if self.transition_blend.enabled {
            // Wired when ScreenTransition integration is needed
        }

        // Layer 7: Phosphor decay (stub)
        if self.phosphor_decay.enabled {
            // Wired when phosphor persistence integration is needed
        }

        // Layer 8: Phase degradation (stub)
        if self.phase_degradation.enabled {
            // Wired when full phase-driven degradation is needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atmospheric_stack_new_defaults() {
        let stack = AtmosphericStack::new();
        assert!(stack.noise_floor.enabled);
        assert!(stack.scanline.enabled);
        assert!(!stack.plasma_bg.enabled);
        assert!(!stack.particle_overlay.enabled);
        assert!(!stack.nerv_chrome.enabled);
        assert!(!stack.transition_blend.enabled);
        assert!(!stack.phosphor_decay.enabled);
        assert!(!stack.phase_degradation.enabled);
    }

    #[test]
    fn test_atmospheric_stack_apply_no_panic() {
        let stack = AtmosphericStack::new();
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        stack.process(&mut buf, area, Phase::Thriving, &mut rng);
        // Should not panic
    }

    #[test]
    fn test_atmospheric_stack_all_phases() {
        let stack = AtmosphericStack::new();
        let area = Rect::new(0, 0, 20, 6);
        let mut buf = Buffer::empty(area);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        for phase in [
            Phase::Thriving,
            Phase::Stable,
            Phase::Conservation,
            Phase::Declining,
            Phase::Terminal,
        ] {
            stack.process(&mut buf, area, phase, &mut rng);
        }
    }
}
