//! Tweening and animation framework.
//!
//! Provides easing functions, scalar tweens, and a named animation manager.
//! Used by the particle system and future cinematic sequences.

/// Easing function variants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl Easing {
    /// Apply the easing function to a normalized time t in [0.0, 1.0].
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::Bounce => {
                let t = 1.0 - t;
                if t < 1.0 / 2.75 {
                    1.0 - 7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    1.0 - (7.5625 * t * t + 0.75)
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    1.0 - (7.5625 * t * t + 0.9375)
                } else {
                    let t = t - 2.625 / 2.75;
                    1.0 - (7.5625 * t * t + 0.984375)
                }
            }
            Easing::Elastic => {
                if t == 0.0 {
                    return 0.0;
                }
                if t == 1.0 {
                    return 1.0;
                }
                let p = 0.3_f32;
                let s = p / 4.0;
                let t = t - 1.0;
                2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI / p)).sin() + 1.0
            }
        }
    }
}

/// A single tween: interpolates a scalar from `start` to `end` over `duration_ticks`.
pub struct Tween {
    pub start: f32,
    pub end: f32,
    pub duration_ticks: u32,
    pub elapsed_ticks: u32,
    pub easing: Easing,
}

impl Tween {
    pub fn new(start: f32, end: f32, duration_ticks: u32, easing: Easing) -> Self {
        Self {
            start,
            end,
            duration_ticks,
            elapsed_ticks: 0,
            easing,
        }
    }

    pub fn tick(&mut self) {
        self.elapsed_ticks = self.elapsed_ticks.saturating_add(1);
    }

    /// Current interpolated value. Clamped after completion.
    pub fn value(&self) -> f32 {
        if self.duration_ticks == 0 {
            return self.end;
        }
        let t = (self.elapsed_ticks as f32 / self.duration_ticks as f32).min(1.0);
        let eased = self.easing.apply(t);
        self.start + (self.end - self.start) * eased
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed_ticks >= self.duration_ticks
    }
}

/// Named animations wrapping a Tween.
pub enum Animation {
    FadeIn { tween: Tween },
    FadeOut { tween: Tween },
    SlideLeft { tween: Tween },
    SlideRight { tween: Tween },
    Pulse { tween: Tween, cycles: u32 },
}

impl Animation {
    pub fn tick(&mut self) {
        match self {
            Animation::FadeIn { tween }
            | Animation::FadeOut { tween }
            | Animation::SlideLeft { tween }
            | Animation::SlideRight { tween }
            | Animation::Pulse { tween, .. } => tween.tick(),
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Animation::FadeIn { tween }
            | Animation::FadeOut { tween }
            | Animation::SlideLeft { tween }
            | Animation::SlideRight { tween }
            | Animation::Pulse { tween, .. } => tween.is_complete(),
        }
    }

    /// The scalar value produced by this animation at the current tick.
    pub fn value(&self) -> f32 {
        match self {
            Animation::FadeIn { tween }
            | Animation::FadeOut { tween }
            | Animation::SlideLeft { tween }
            | Animation::SlideRight { tween } => tween.value(),
            Animation::Pulse { tween, cycles } => {
                let t = tween.elapsed_ticks as f32 / tween.duration_ticks.max(1) as f32;
                let osc = (2.0 * std::f32::consts::PI * *cycles as f32 * t).sin();
                1.0 + 0.5 * osc
            }
        }
    }
}

/// Manages a collection of named animations.
pub struct Animator {
    active: Vec<(String, Animation)>,
}

impl Animator {
    pub fn new() -> Self {
        Self { active: Vec::new() }
    }

    /// Start a named animation. Replaces any existing animation with the same ID.
    pub fn start(&mut self, id: impl Into<String>, animation: Animation) {
        let id = id.into();
        self.active.retain(|(existing_id, _)| existing_id != &id);
        self.active.push((id, animation));
    }

    /// Advance all active animations one tick. Remove completed ones.
    pub fn tick(&mut self) {
        for (_, anim) in &mut self.active {
            anim.tick();
        }
        self.active.retain(|(_, anim)| !anim.is_complete());
    }

    /// Get the current scalar value of a named animation.
    pub fn get_value(&self, id: &str) -> Option<f32> {
        self.active
            .iter()
            .find(|(existing_id, _)| existing_id == id)
            .map(|(_, anim)| anim.value())
    }

    pub fn is_running(&self, id: &str) -> bool {
        self.active.iter().any(|(existing_id, _)| existing_id == id)
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tween_linear_midpoint_is_midvalue() {
        let mut tw = Tween::new(0.0, 10.0, 10, Easing::Linear);
        for _ in 0..5 {
            tw.tick();
        }
        assert!((tw.value() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_easing_ease_in_out_is_symmetric() {
        let a = Easing::EaseInOut.apply(0.25);
        let b = Easing::EaseInOut.apply(0.75);
        assert!((a + b - 1.0).abs() < 0.001);
        assert!((Easing::EaseInOut.apply(0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_tween_completes_after_duration() {
        let mut tw = Tween::new(0.0, 100.0, 5, Easing::Linear);
        for _ in 0..5 {
            tw.tick();
        }
        assert!(tw.is_complete());
        assert!((tw.value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_tween_value_clamps_after_completion() {
        let mut tw = Tween::new(0.0, 42.0, 5, Easing::Linear);
        for _ in 0..10 {
            tw.tick();
        }
        assert!((tw.value() - 42.0).abs() < 0.01);
    }

    #[test]
    fn test_animator_start_replaces_existing() {
        let mut animator = Animator::new();
        animator.start(
            "x",
            Animation::FadeIn {
                tween: Tween::new(0.0, 1.0, 10, Easing::Linear),
            },
        );
        animator.start(
            "x",
            Animation::FadeOut {
                tween: Tween::new(1.0, 0.0, 10, Easing::Linear),
            },
        );
        assert_eq!(animator.active_count(), 1);
    }

    #[test]
    fn test_animator_tick_removes_completed() {
        let mut animator = Animator::new();
        animator.start(
            "short",
            Animation::FadeIn {
                tween: Tween::new(0.0, 1.0, 1, Easing::Linear),
            },
        );
        animator.tick();
        assert_eq!(animator.active_count(), 0);
    }

    #[test]
    fn test_animator_get_value_returns_none_for_unknown_id() {
        let animator = Animator::new();
        assert!(animator.get_value("nonexistent").is_none());
    }

    #[test]
    fn test_easing_bounce_output_range() {
        for &t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
            let v = Easing::Bounce.apply(t);
            assert!(v >= -0.1 && v <= 1.1, "Bounce({t}) = {v} out of range");
        }
    }

    #[test]
    fn test_easing_elastic_endpoints() {
        assert!((Easing::Elastic.apply(0.0)).abs() < 0.001);
        assert!((Easing::Elastic.apply(1.0) - 1.0).abs() < 0.001);
    }
}
