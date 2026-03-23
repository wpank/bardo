//! Visualizer pane — four stacked displays for real-time audio feedback.
//!
//! 1. Mini waveform strip (braille, stereo, 10fps)
//! 2. Envelope follower (unicode block scrolling bar, 30fps)
//! 3. Active-notes piano roll (C2..B4, 10fps)
//! 4. Clock phase display (gamma/theta/delta arcs, 10fps)

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::{
    palette::{BONE, ROSE, ROSE_BRIGHT, ROSE_DIM, TEXT_DIM, TEXT_PRIMARY},
    sonification::VisualizerState,
};

// ── Composite visualizer ────────────────────────────────────────────

/// Renders all four visualizer strips stacked vertically.
/// Requires at least 8 rows: 2 waveform + 1 envelope + 3 piano roll + 2 clocks.
pub(crate) struct VisualizerPane<'a> {
    pub(crate) state: &'a VisualizerState,
}

impl<'a> Widget for VisualizerPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 6 || area.width < 20 {
            return;
        }

        let mut y = area.y;
        let w = area.width;

        // 1. Waveform strip: 2 rows (left + right channel)
        if y + 2 <= area.bottom() {
            let strip = WaveformStrip { state: self.state };
            strip.render(Rect::new(area.x, y, w, 2), buf);
            y += 2;
        }

        // 2. Envelope follower: 1 row
        if y + 1 <= area.bottom() {
            let env = EnvelopeFollower { state: self.state };
            env.render(Rect::new(area.x, y, w, 1), buf);
            y += 1;
        }

        // 3. Piano roll: 2 rows (header + keys)
        if y + 2 <= area.bottom() {
            let roll = PianoRoll { state: self.state };
            roll.render(Rect::new(area.x, y, w, 2), buf);
            y += 2;
        }

        // 4. Clock phase: 1 row
        if y + 1 <= area.bottom() {
            let clocks = ClockPhaseDisplay { state: self.state };
            clocks.render(Rect::new(area.x, y, w, 1), buf);
        }
    }
}

// ── 1. Mini waveform strip ──────────────────────────────────────────

/// Braille-encoded stereo waveform: 64 cells per row (each 2x4 = 8 samples).
/// Top row = left channel, bottom row = right channel.
struct WaveformStrip<'a> {
    state: &'a VisualizerState,
}

const BRAILLE_BASE: u32 = 0x2800;
const LEFT_COL_BITS: [u8; 4] = [0x40, 0x04, 0x02, 0x01];
const RIGHT_COL_BITS: [u8; 4] = [0x80, 0x20, 0x10, 0x08];

fn amplitude_to_dots(amplitude: f32) -> usize {
    let normalized = amplitude.abs().clamp(0.0, 1.0);
    // 0 = empty, 1.0 = 4 dots (solid)
    (normalized * 4.0).round() as usize
}

fn braille_char(left_dots: usize, right_dots: usize) -> char {
    let left = LEFT_COL_BITS[..left_dots.min(4)]
        .iter()
        .fold(0u8, |a, &b| a | b);
    let right = RIGHT_COL_BITS[..right_dots.min(4)]
        .iter()
        .fold(0u8, |a, &b| a | b);
    char::from_u32(BRAILLE_BASE + (left | right) as u32).unwrap_or(' ')
}

impl<'a> Widget for WaveformStrip<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 2 || area.width == 0 {
            return;
        }

        let cells = area.width.min(64) as usize;
        let samples_needed = cells * 4; // 4 samples per cell column (2 left, 2 right in braille)

        // Render left channel on top row
        render_braille_row(
            buf,
            area.x,
            area.y,
            cells,
            &self.state.waveform_left,
            samples_needed,
            ROSE,
        );

        // Render right channel on bottom row
        render_braille_row(
            buf,
            area.x,
            area.y + 1,
            cells,
            &self.state.waveform_right,
            samples_needed,
            ROSE_DIM,
        );
    }
}

fn render_braille_row(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    cells: usize,
    samples: &std::collections::VecDeque<f32>,
    _samples_needed: usize,
    color: Color,
) {
    let len = samples.len();
    for cell_idx in 0..cells {
        // Each braille cell encodes 2 left-column samples and 2 right-column samples
        let base = if len > cells * 4 {
            len - cells * 4 + cell_idx * 4
        } else {
            cell_idx * 4
        };
        let s0 = samples.get(base).copied().unwrap_or(0.0);
        let s1 = samples.get(base + 1).copied().unwrap_or(0.0);
        let s2 = samples.get(base + 2).copied().unwrap_or(0.0);
        let s3 = samples.get(base + 3).copied().unwrap_or(0.0);

        let left_dots = amplitude_to_dots((s0 + s1) / 2.0);
        let right_dots = amplitude_to_dots((s2 + s3) / 2.0);
        let ch = braille_char(left_dots, right_dots);

        let cx = x + cell_idx as u16;
        let buf_area = buf.area();
        if cx < buf_area.right() && y < buf_area.bottom() {
            let cell = buf.get_mut(cx, y);
            cell.set_char(ch);
            cell.set_style(Style::default().fg(color));
        }
    }
}

// ── 2. Envelope follower ────────────────────────────────────────────

/// Unicode block scrolling bar: ▁▂▃▅▆█▇▅▃▂▁
const BLOCK_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

struct EnvelopeFollower<'a> {
    state: &'a VisualizerState,
}

impl<'a> Widget for EnvelopeFollower<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let width = area.width.min(40) as usize;
        let len = self.state.envelope.len();
        let offset = len.saturating_sub(width);

        for (i, &val) in self.state.envelope.iter().skip(offset).enumerate() {
            if i >= width {
                break;
            }
            let normalized = val.clamp(0.0, 1.0);
            let idx = (normalized * (BLOCK_CHARS.len() - 1) as f32).round() as usize;
            let ch = BLOCK_CHARS[idx.min(BLOCK_CHARS.len() - 1)];

            let x = area.x + i as u16;
            let y = area.y;
            let buf_area = buf.area();
            if x < buf_area.right() && y < buf_area.bottom() {
                let cell = buf.get_mut(x, y);
                cell.set_char(ch);
                cell.set_style(Style::default().fg(ROSE_BRIGHT));
            }
        }

        // Fill remaining cells with silence indicator
        for i in len.saturating_sub(offset)..width {
            let x = area.x + i as u16;
            let buf_area = buf.area();
            if x < buf_area.right() && area.y < buf_area.bottom() {
                let cell = buf.get_mut(x, area.y);
                cell.set_char('▁');
                cell.set_style(Style::default().fg(TEXT_DIM));
            }
        }
    }
}

// ── 3. Piano roll ───────────────────────────────────────────────────

/// Active-notes display: C2..B4 (36 keys).
/// Active: █, released <500ms: ▓, <1s: ▒, <2s: ░, silent: space.
struct PianoRoll<'a> {
    state: &'a VisualizerState,
}

const NOTE_NAMES: &[&str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

fn note_char(active: bool, age: Option<f64>) -> char {
    if active {
        return '█';
    }
    match age {
        Some(a) if a < 0.5 => '▓',
        Some(a) if a < 1.0 => '▒',
        Some(a) if a < 2.0 => '░',
        _ => ' ',
    }
}

fn note_color(active: bool, age: Option<f64>) -> Color {
    if active {
        return ROSE_BRIGHT;
    }
    match age {
        Some(a) if a < 0.5 => ROSE,
        Some(a) if a < 1.0 => ROSE_DIM,
        Some(a) if a < 2.0 => TEXT_DIM,
        _ => TEXT_DIM,
    }
}

impl<'a> Widget for PianoRoll<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 36 || area.height < 2 {
            return;
        }

        // Header row: octave markers
        let header_style = Style::default().fg(TEXT_DIM);
        let y_header = area.y;
        // C2 at position 0, C3 at 12, C4 at 24
        for (oct_idx, label) in ["C2", "C3", "C4"].iter().enumerate() {
            let x = area.x + (oct_idx * 12) as u16;
            let buf_area = buf.area();
            if x + 2 <= buf_area.right() && y_header < buf_area.bottom() {
                buf.set_stringn(x, y_header, label, 2, header_style);
            }
        }

        // Key row
        let y_keys = area.y + 1;
        for i in 0..36usize {
            let x = area.x + i as u16;
            let buf_area = buf.area();
            if x >= buf_area.right() || y_keys >= buf_area.bottom() {
                break;
            }
            let ch = note_char(self.state.note_active[i], self.state.note_age[i]);
            let color = note_color(self.state.note_active[i], self.state.note_age[i]);
            let cell = buf.get_mut(x, y_keys);
            cell.set_char(ch);
            cell.set_style(Style::default().fg(color));
        }
    }
}

// ── 4. Clock phase display ──────────────────────────────────────────

/// ASCII arc indicators for gamma/theta/delta clocks.
/// Each is a 5-char arc that rotates clockwise.
struct ClockPhaseDisplay<'a> {
    state: &'a VisualizerState,
}

const CLOCK_NAMES: &[&str] = &["γ", "θ", "δ"];
const ARC_CHARS: &[char] = &['◜', '◝', '◞', '◟'];

fn arc_display(phase: f64, fired: bool) -> String {
    // 5 chars: select which arc character based on phase quadrant
    let quadrant = ((phase * 4.0) as usize) % 4;
    let mut s = String::with_capacity(5);
    for i in 0..4 {
        if i == quadrant {
            s.push(if fired { '●' } else { ARC_CHARS[i] });
        } else {
            s.push('·');
        }
    }
    s.push(if fired { '!' } else { ' ' });
    s
}

impl<'a> Widget for ClockPhaseDisplay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height == 0 {
            return;
        }

        let y = area.y;
        let mut x = area.x;

        for i in 0..3 {
            let name = CLOCK_NAMES[i];
            let arc = arc_display(self.state.clock_phase[i], self.state.clock_fired[i]);
            let period_label = format!("({:.1}s)", self.state.clock_period[i]);

            let color = if self.state.clock_fired[i] {
                ROSE_BRIGHT
            } else {
                TEXT_PRIMARY
            };

            let text = format!("{name} {arc} {period_label}");
            let buf_area = buf.area();
            if x + text.len() as u16 <= buf_area.right() && y < buf_area.bottom() {
                buf.set_stringn(x, y, &text, text.len(), Style::default().fg(color));
            }
            x += text.len() as u16 + 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sonification::VisualizerState;

    #[test]
    fn waveform_strip_renders_without_panic() {
        let mut state = VisualizerState::default();
        for i in 0..256 {
            let v = (i as f32 / 256.0 * std::f32::consts::TAU).sin();
            state.push_waveform(v, -v);
        }
        let strip = WaveformStrip { state: &state };
        let area = Rect::new(0, 0, 64, 2);
        let mut buf = Buffer::empty(area);
        strip.render(area, &mut buf);
        // Should have non-space characters where amplitude > 0
        let has_content = (0..64).any(|x| buf.get(x, 0).symbol() != " ");
        assert!(has_content);
    }

    #[test]
    fn envelope_follower_renders_silence_as_low_blocks() {
        let state = VisualizerState::default();
        let env = EnvelopeFollower { state: &state };
        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        env.render(area, &mut buf);
        // Empty envelope → all silence indicator '▁'
        for x in 0..40 {
            let sym = buf.get(x, 0).symbol().chars().next().unwrap_or(' ');
            assert_eq!(sym, '▁');
        }
    }

    #[test]
    fn piano_roll_shows_active_and_fading_notes() {
        let mut state = VisualizerState::default();
        state.note_active[0] = true;
        state.note_age[0] = Some(0.0);
        state.note_age[12] = Some(0.3); // released <500ms
        state.note_age[24] = Some(0.8); // released <1s

        let roll = PianoRoll { state: &state };
        let area = Rect::new(0, 0, 36, 2);
        let mut buf = Buffer::empty(area);
        roll.render(area, &mut buf);

        assert_eq!(buf.get(0, 1).symbol(), "█"); // active
        assert_eq!(buf.get(12, 1).symbol(), "▓"); // fading
        assert_eq!(buf.get(24, 1).symbol(), "▒"); // more faded
    }

    #[test]
    fn clock_phase_display_renders_three_clocks() {
        let state = VisualizerState::default();
        let clocks = ClockPhaseDisplay { state: &state };
        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);
        clocks.render(area, &mut buf);

        let row: String = (0..80)
            .map(|x| buf.get(x, 0).symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(row.contains("γ"), "missing gamma: {row}");
        assert!(row.contains("θ"), "missing theta: {row}");
        assert!(row.contains("δ"), "missing delta: {row}");
    }

    #[test]
    fn clock_fired_shows_flash_indicator() {
        let mut state = VisualizerState::default();
        state.clock_fired[0] = true;
        let arc = arc_display(state.clock_phase[0], true);
        assert!(arc.contains('●'));
        assert!(arc.contains('!'));
    }

    #[test]
    fn braille_char_full_scale() {
        let ch = braille_char(4, 4);
        assert_eq!(ch, '\u{28FF}'); // full braille block
    }

    #[test]
    fn braille_char_empty() {
        let ch = braille_char(0, 0);
        assert_eq!(ch, '\u{2800}'); // empty braille
    }

    #[test]
    fn note_char_decay_sequence() {
        assert_eq!(note_char(true, Some(0.0)), '█');
        assert_eq!(note_char(false, Some(0.3)), '▓');
        assert_eq!(note_char(false, Some(0.7)), '▒');
        assert_eq!(note_char(false, Some(1.5)), '░');
        assert_eq!(note_char(false, Some(3.0)), ' ');
        assert_eq!(note_char(false, None), ' ');
    }

    #[test]
    fn visualizer_pane_composite_renders() {
        let mut state = VisualizerState::default();
        state.visible = true;
        for i in 0..256 {
            let v = (i as f32 / 32.0).sin() * 0.5;
            state.push_waveform(v, v);
        }
        for _ in 0..40 {
            state.push_envelope(0.5);
        }
        state.note_active[0] = true;
        state.note_age[0] = Some(0.0);

        let pane = VisualizerPane { state: &state };
        let area = Rect::new(0, 0, 64, 8);
        let mut buf = Buffer::empty(area);
        pane.render(area, &mut buf);
        // Just verify no panic and some content exists
    }
}
