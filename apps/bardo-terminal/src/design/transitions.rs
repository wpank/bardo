//! Screen transitions: tier system, transition types, and active transition state.
//!
//! From `prd2/18-interfaces/rendering/03-transitions.md`.

use crate::animation::{Easing, Tween};

// ── Transition Type ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionType {
    Fade,
    Crossfade,
    HorizontalSlide { direction: i8 },
    VerticalDissolve,
    RadialWipe,
    GlitchCut,
    FadeThrough,
    DreamDissolution,
}

// ── Transition Tier ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransitionTier {
    T0,
    T1,
    T2,
    T3,
    T4,
}

impl TransitionTier {
    pub fn from_novelty(n: f64) -> Self {
        if n < 0.2 {
            Self::T0
        } else if n < 0.5 {
            Self::T1
        } else if n < 0.8 {
            Self::T2
        } else {
            Self::T3
        }
    }

    pub fn duration_ms(self) -> (u64, u64) {
        match self {
            Self::T0 => (50, 200),
            Self::T1 => (200, 1000),
            Self::T2 => (500, 3000),
            Self::T3 => (2000, 8000),
            Self::T4 => (5000, 15_000),
        }
    }
}

// ── Screen Transition ──────────────────────────────────────────────

pub struct ScreenTransition {
    pub transition_type: TransitionType,
    pub tier: TransitionTier,
    pub duration_ticks: u32,
    pub current_tick: u32,
    pub tween: Tween,
    pub easing: Easing,
}

impl ScreenTransition {
    pub fn new(ty: TransitionType, tier: TransitionTier) -> Self {
        let (min_ms, max_ms) = tier.duration_ms();
        let mid_ms = (min_ms + max_ms) / 2;
        let duration_ticks = ((mid_ms as f64 / 1000.0) * 60.0) as u32;
        let easing = Self::easing_for_type(ty);
        Self {
            transition_type: ty,
            tier,
            duration_ticks,
            current_tick: 0,
            tween: Tween::new(0.0, 1.0, duration_ticks, easing),
            easing,
        }
    }

    fn easing_for_type(ty: TransitionType) -> Easing {
        match ty {
            TransitionType::GlitchCut => Easing::Elastic,
            TransitionType::DreamDissolution => Easing::EaseInOut,
            TransitionType::Fade | TransitionType::FadeThrough => Easing::EaseIn,
            _ => Easing::EaseOut,
        }
    }

    pub fn is_done(&self) -> bool {
        self.current_tick >= self.duration_ticks
    }

    pub fn progress(&self) -> f32 {
        self.tween.value()
    }

    pub fn advance(&mut self) {
        self.current_tick += 1;
        self.tween.tick();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_from_novelty_boundaries() {
        assert_eq!(TransitionTier::from_novelty(0.0), TransitionTier::T0);
        assert_eq!(TransitionTier::from_novelty(0.19), TransitionTier::T0);
        assert_eq!(TransitionTier::from_novelty(0.2), TransitionTier::T1);
        assert_eq!(TransitionTier::from_novelty(0.49), TransitionTier::T1);
        assert_eq!(TransitionTier::from_novelty(0.5), TransitionTier::T2);
        assert_eq!(TransitionTier::from_novelty(0.79), TransitionTier::T2);
        assert_eq!(TransitionTier::from_novelty(0.8), TransitionTier::T3);
        assert_eq!(TransitionTier::from_novelty(1.0), TransitionTier::T3);
    }

    #[test]
    fn test_transition_tier_duration_ranges() {
        assert_eq!(TransitionTier::T0.duration_ms(), (50, 200));
        assert_eq!(TransitionTier::T1.duration_ms(), (200, 1000));
        assert_eq!(TransitionTier::T2.duration_ms(), (500, 3000));
        assert_eq!(TransitionTier::T3.duration_ms(), (2000, 8000));
        assert_eq!(TransitionTier::T4.duration_ms(), (5000, 15_000));
    }

    #[test]
    fn test_transition_progress_bounded() {
        let mut t = ScreenTransition::new(TransitionType::Fade, TransitionTier::T1);
        // Progress should start at 0 and stay bounded
        assert!(t.progress() >= 0.0);
        for _ in 0..1000 {
            t.advance();
            let p = t.progress();
            assert!(p >= 0.0 && p <= 1.01, "progress {} out of bounds", p);
        }
    }

    #[test]
    fn test_transition_done_after_duration_ticks() {
        let mut t = ScreenTransition::new(TransitionType::Fade, TransitionTier::T0);
        assert!(!t.is_done());
        for _ in 0..t.duration_ticks {
            t.advance();
        }
        assert!(t.is_done());
    }

    #[test]
    fn test_transition_easing_per_type() {
        let glitch = ScreenTransition::new(TransitionType::GlitchCut, TransitionTier::T1);
        assert_eq!(glitch.easing, Easing::Elastic);

        let dream = ScreenTransition::new(TransitionType::DreamDissolution, TransitionTier::T2);
        assert_eq!(dream.easing, Easing::EaseInOut);

        let fade = ScreenTransition::new(TransitionType::Fade, TransitionTier::T0);
        assert_eq!(fade.easing, Easing::EaseIn);

        let fade_through = ScreenTransition::new(TransitionType::FadeThrough, TransitionTier::T1);
        assert_eq!(fade_through.easing, Easing::EaseIn);

        let slide = ScreenTransition::new(
            TransitionType::HorizontalSlide { direction: 1 },
            TransitionTier::T1,
        );
        assert_eq!(slide.easing, Easing::EaseOut);

        let crossfade = ScreenTransition::new(TransitionType::Crossfade, TransitionTier::T1);
        assert_eq!(crossfade.easing, Easing::EaseOut);
    }

    #[test]
    fn test_fade_transition_starts_at_zero() {
        let t = ScreenTransition::new(TransitionType::Fade, TransitionTier::T1);
        assert_eq!(t.current_tick, 0);
        assert!((t.progress() - 0.0).abs() < 1e-6);
    }
}
