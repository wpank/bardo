//! Sonification data types for TUI rendering.
//!
//! Provides CV mapping state, visualizer buffers, and Listener Mode slider
//! state. These types are owned by screen state and rendered by widgets in
//! `widgets::cv_map_pane`, `widgets::visualizer`, and `widgets::listener_mode`.

use std::collections::VecDeque;

// ── CV Mapping ──────────────────────────────────────────────────────

/// Interpolation curve applied to a CV output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CvCurve {
    Linear,
    Exp(f64),
    Log,
    SCurve,
}

impl CvCurve {
    pub(crate) fn label(self) -> String {
        match self {
            Self::Linear => "linear".into(),
            Self::Exp(p) => format!("exp({p})"),
            Self::Log => "log".into(),
            Self::SCurve => "s-curve".into(),
        }
    }
}

/// One row of the CV map table: a named output driven by a source signal.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CvOutput {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) output_min: f64,
    pub(crate) output_max: f64,
    pub(crate) curve: CvCurve,
    pub(crate) alpha: f64,
}

impl CvOutput {
    /// True when output_min > output_max (inverted mapping).
    pub(crate) fn is_inverted(&self) -> bool {
        self.output_min > self.output_max
    }

    /// Format the range column, e.g. "0.1 - 0.9".
    pub(crate) fn range_label(&self) -> String {
        format!("{:.1} - {:.1}", self.output_min, self.output_max)
    }
}

/// Returns the 13 default CV outputs from the spec.
pub(crate) fn default_cv_map() -> Vec<CvOutput> {
    vec![
        CvOutput {
            name: "clock_bpm".into(),
            source: "regime".into(),
            output_min: 0.0,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.05,
        },
        CvOutput {
            name: "filter_brightness".into(),
            source: "pleasure".into(),
            output_min: 0.1,
            output_max: 0.9,
            curve: CvCurve::Exp(2.0),
            alpha: 0.08,
        },
        CvOutput {
            name: "plaits_engine".into(),
            source: "arousal".into(),
            output_min: 0.0,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.10,
        },
        CvOutput {
            name: "dattorro_mix".into(),
            source: "dominance".into(),
            output_min: 0.0,
            output_max: 0.95,
            curve: CvCurve::SCurve,
            alpha: 0.06,
        },
        CvOutput {
            name: "dattorro_decay".into(),
            source: "dominance".into(),
            output_min: 0.2,
            output_max: 0.98,
            curve: CvCurve::Exp(1.5),
            alpha: 0.04,
        },
        CvOutput {
            name: "beads_gate_density".into(),
            source: "arousal".into(),
            output_min: 0.05,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.12,
        },
        CvOutput {
            name: "probability_gate".into(),
            source: "arousal".into(),
            output_min: 0.05,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.10,
        },
        CvOutput {
            name: "stages_attack".into(),
            source: "pleasure".into(),
            output_min: 0.01,
            output_max: 0.5,
            curve: CvCurve::Log,
            alpha: 0.08,
        },
        CvOutput {
            name: "stages_decay".into(),
            source: "pleasure".into(),
            output_min: 0.05,
            output_max: 2.0,
            curve: CvCurve::Log,
            alpha: 0.08,
        },
        CvOutput {
            name: "lfo_rate_mult".into(),
            source: "arousal".into(),
            output_min: 0.5,
            output_max: 2.0,
            curve: CvCurve::Linear,
            alpha: 0.15,
        },
        CvOutput {
            name: "master_volume".into(),
            source: "regime".into(),
            output_min: 0.3,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.02,
        },
        CvOutput {
            name: "pan_spread".into(),
            source: "arousal".into(),
            output_min: 0.0,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.10,
        },
        CvOutput {
            name: "harmonic_shift".into(),
            source: "pleasure".into(),
            output_min: -0.5,
            output_max: 0.5,
            curve: CvCurve::Linear,
            alpha: 0.06,
        },
    ]
}

/// Mutable state for the CV Map pane.
#[derive(Debug, Clone)]
pub(crate) struct CvMapState {
    pub(crate) outputs: Vec<CvOutput>,
    pub(crate) cursor: usize,
    pub(crate) editing: bool,
    /// Which column is focused during edit (0=source, 1=min, 2=max, 3=curve, 4=alpha).
    pub(crate) edit_col: usize,
}

impl Default for CvMapState {
    fn default() -> Self {
        Self {
            outputs: default_cv_map(),
            cursor: 0,
            editing: false,
            edit_col: 0,
        }
    }
}

impl CvMapState {
    pub(crate) fn move_cursor_down(&mut self) {
        if !self.outputs.is_empty() {
            self.cursor = (self.cursor + 1).min(self.outputs.len() - 1);
        }
    }

    pub(crate) fn move_cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub(crate) fn delete_selected(&mut self) {
        if !self.outputs.is_empty() {
            self.outputs.remove(self.cursor);
            if self.cursor >= self.outputs.len() && !self.outputs.is_empty() {
                self.cursor = self.outputs.len() - 1;
            }
        }
    }

    pub(crate) fn reset_defaults(&mut self) {
        self.outputs = default_cv_map();
        self.cursor = 0;
        self.editing = false;
    }

    pub(crate) fn add_output(&mut self) {
        self.outputs.push(CvOutput {
            name: format!("cv_output_{}", self.outputs.len()),
            source: "pleasure".into(),
            output_min: 0.0,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.10,
        });
        self.cursor = self.outputs.len() - 1;
    }
}

// ── Visualizer ──────────────────────────────────────────────────────

/// Rolling buffer sizes for the four visualizer strips.
const WAVEFORM_SAMPLES: usize = 256;
const ENVELOPE_SAMPLES: usize = 40;

/// State for the four stacked visualizer displays.
#[derive(Debug, Clone)]
pub(crate) struct VisualizerState {
    /// True when the visualizer pane is visible (toggled with 'v').
    pub(crate) visible: bool,

    // -- Mini waveform strip (256 samples, stereo) --
    pub(crate) waveform_left: VecDeque<f32>,
    pub(crate) waveform_right: VecDeque<f32>,

    // -- Envelope follower (40 samples, scrolling) --
    pub(crate) envelope: VecDeque<f32>,

    // -- Piano roll (36 keys: C2..B4, decay tracking) --
    /// For each of the 36 keys, seconds since last note-on. None = never played.
    pub(crate) note_age: [Option<f64>; 36],
    /// Which keys are currently held.
    pub(crate) note_active: [bool; 36],

    // -- Clock phase (gamma/theta/delta) --
    /// Phase 0.0..1.0 for each clock.
    pub(crate) clock_phase: [f64; 3],
    /// Cycle duration in seconds for each clock.
    pub(crate) clock_period: [f64; 3],
    /// Whether the clock just fired (for flash).
    pub(crate) clock_fired: [bool; 3],
}

impl Default for VisualizerState {
    fn default() -> Self {
        Self {
            visible: false,
            waveform_left: VecDeque::with_capacity(WAVEFORM_SAMPLES),
            waveform_right: VecDeque::with_capacity(WAVEFORM_SAMPLES),
            envelope: VecDeque::with_capacity(ENVELOPE_SAMPLES),
            note_age: [None; 36],
            note_active: [false; 36],
            clock_phase: [0.0; 3],
            clock_period: [0.125, 0.5, 2.0], // gamma ~8Hz, theta ~2Hz, delta ~0.5Hz
            clock_fired: [false; 3],
        }
    }
}

impl VisualizerState {
    /// Push a stereo sample pair into the waveform ring.
    pub(crate) fn push_waveform(&mut self, left: f32, right: f32) {
        if self.waveform_left.len() >= WAVEFORM_SAMPLES {
            self.waveform_left.pop_front();
        }
        if self.waveform_right.len() >= WAVEFORM_SAMPLES {
            self.waveform_right.pop_front();
        }
        self.waveform_left.push_back(left);
        self.waveform_right.push_back(right);
    }

    /// Push an envelope sample.
    pub(crate) fn push_envelope(&mut self, value: f32) {
        if self.envelope.len() >= ENVELOPE_SAMPLES {
            self.envelope.pop_front();
        }
        self.envelope.push_back(value);
    }

    /// Advance clock phases by dt seconds.
    pub(crate) fn tick_clocks(&mut self, dt: f64) {
        for i in 0..3 {
            if self.clock_period[i] <= 0.0 {
                continue;
            }
            let prev = self.clock_phase[i];
            self.clock_phase[i] = (self.clock_phase[i] + dt / self.clock_period[i]) % 1.0;
            self.clock_fired[i] = self.clock_phase[i] < prev; // wrapped
        }
    }

    /// Advance note ages by dt. Active notes stay at age 0.
    pub(crate) fn tick_notes(&mut self, dt: f64) {
        for i in 0..36 {
            if self.note_active[i] {
                self.note_age[i] = Some(0.0);
            } else if let Some(age) = self.note_age[i].as_mut() {
                *age += dt;
            }
        }
    }
}

// ── Listener Mode ───────────────────────────────────────────────────

/// Names and indices for the five Listener Mode sliders.
pub(crate) const LISTENER_SLIDERS: [&str; 5] =
    ["Brightness", "Density", "Motion", "Depth", "Warmth"];

/// Five-slider Listener Mode state. All values 0.0..10.0; 5.0 = neutral.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ListenerState {
    pub(crate) active: bool,
    pub(crate) values: [f64; 5],
    pub(crate) cursor: usize,
}

impl Default for ListenerState {
    fn default() -> Self {
        Self {
            active: false,
            values: [5.0; 5],
            cursor: 0,
        }
    }
}

impl ListenerState {
    pub(crate) fn move_cursor_down(&mut self) {
        self.cursor = (self.cursor + 1).min(LISTENER_SLIDERS.len() - 1);
    }

    pub(crate) fn move_cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Adjust the selected slider by delta, clamping to 0.0..10.0.
    pub(crate) fn adjust(&mut self, delta: f64) {
        self.values[self.cursor] = (self.values[self.cursor] + delta).clamp(0.0, 10.0);
    }

    /// Reset all sliders to neutral (5.0).
    pub(crate) fn reset(&mut self) {
        self.values = [5.0; 5];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- CvOutput tests --

    #[test]
    fn default_cv_map_has_13_outputs() {
        assert_eq!(default_cv_map().len(), 13);
    }

    #[test]
    fn cv_output_inverted_detection() {
        let normal = CvOutput {
            name: "test".into(),
            source: "x".into(),
            output_min: 0.0,
            output_max: 1.0,
            curve: CvCurve::Linear,
            alpha: 0.1,
        };
        assert!(!normal.is_inverted());

        let inverted = CvOutput {
            name: "test".into(),
            source: "x".into(),
            output_min: 1.0,
            output_max: 0.0,
            curve: CvCurve::Linear,
            alpha: 0.1,
        };
        assert!(inverted.is_inverted());
    }

    #[test]
    fn cv_curve_labels() {
        assert_eq!(CvCurve::Linear.label(), "linear");
        assert_eq!(CvCurve::Exp(2.0).label(), "exp(2)");
        assert_eq!(CvCurve::Log.label(), "log");
        assert_eq!(CvCurve::SCurve.label(), "s-curve");
    }

    // -- CvMapState tests --

    #[test]
    fn cv_map_state_cursor_bounds() {
        let mut state = CvMapState::default();
        assert_eq!(state.cursor, 0);
        for _ in 0..20 {
            state.move_cursor_down();
        }
        assert_eq!(state.cursor, 12); // 13 items, max index 12
        for _ in 0..20 {
            state.move_cursor_up();
        }
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn cv_map_state_delete_and_add() {
        let mut state = CvMapState::default();
        let initial = state.outputs.len();
        state.cursor = 5;
        state.delete_selected();
        assert_eq!(state.outputs.len(), initial - 1);
        assert_eq!(state.cursor, 5);

        state.add_output();
        assert_eq!(state.outputs.len(), initial);
        assert_eq!(state.cursor, state.outputs.len() - 1);
    }

    #[test]
    fn cv_map_state_reset_defaults() {
        let mut state = CvMapState::default();
        state.outputs.clear();
        state.cursor = 99;
        state.editing = true;
        state.reset_defaults();
        assert_eq!(state.outputs.len(), 13);
        assert_eq!(state.cursor, 0);
        assert!(!state.editing);
    }

    // -- VisualizerState tests --

    #[test]
    fn visualizer_waveform_ring_caps_at_256() {
        let mut viz = VisualizerState::default();
        for i in 0..300 {
            viz.push_waveform(i as f32, -(i as f32));
        }
        assert_eq!(viz.waveform_left.len(), WAVEFORM_SAMPLES);
        assert_eq!(viz.waveform_right.len(), WAVEFORM_SAMPLES);
        // newest sample is the last pushed
        assert_eq!(*viz.waveform_left.back().unwrap(), 299.0);
    }

    #[test]
    fn visualizer_envelope_ring_caps_at_40() {
        let mut viz = VisualizerState::default();
        for i in 0..60 {
            viz.push_envelope(i as f32);
        }
        assert_eq!(viz.envelope.len(), ENVELOPE_SAMPLES);
    }

    #[test]
    fn visualizer_clock_phase_wraps_and_fires() {
        let mut viz = VisualizerState::default();
        viz.clock_phase = [0.95, 0.5, 0.1];
        viz.clock_period = [1.0, 1.0, 1.0];
        viz.tick_clocks(0.1);
        assert!(viz.clock_fired[0]); // 0.95 + 0.1 > 1.0 => wrapped
        assert!(!viz.clock_fired[1]);
        assert!(!viz.clock_fired[2]);
    }

    #[test]
    fn visualizer_note_age_tracking() {
        let mut viz = VisualizerState::default();
        // Activate note 12 (C3)
        viz.note_active[12] = true;
        viz.note_age[12] = Some(0.0);
        viz.tick_notes(0.5);
        assert_eq!(viz.note_age[12], Some(0.0)); // active stays at 0

        // Release note 12
        viz.note_active[12] = false;
        viz.tick_notes(0.3);
        assert_eq!(viz.note_age[12], Some(0.3));
        viz.tick_notes(0.2);
        assert_eq!(viz.note_age[12], Some(0.5));
    }

    // -- ListenerState tests --

    #[test]
    fn listener_state_defaults_to_neutral() {
        let state = ListenerState::default();
        assert!(!state.active);
        assert_eq!(state.values, [5.0; 5]);
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn listener_state_adjust_clamps() {
        let mut state = ListenerState::default();
        state.adjust(6.0); // 5.0 + 6.0 = 11.0 -> clamped to 10.0
        assert_eq!(state.values[0], 10.0);
        state.adjust(-15.0); // 10.0 - 15.0 = -5.0 -> clamped to 0.0
        assert_eq!(state.values[0], 0.0);
    }

    #[test]
    fn listener_state_cursor_bounds() {
        let mut state = ListenerState::default();
        for _ in 0..10 {
            state.move_cursor_down();
        }
        assert_eq!(state.cursor, 4);
        for _ in 0..10 {
            state.move_cursor_up();
        }
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn listener_state_reset() {
        let mut state = ListenerState::default();
        state.values = [0.0, 2.0, 8.0, 10.0, 3.0];
        state.reset();
        assert_eq!(state.values, [5.0; 5]);
    }
}
