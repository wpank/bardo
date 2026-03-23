//! NERV institutional aesthetic system.
//!
//! Hazard stripes, crisis modes, pattern/status stamps, unit arrays,
//! psychographic display, AT field, scanline and noise post-processors.

use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use super::palette::*;
use super::tokens::{Phase, PhaseTokens};

// ── Crisis Mode ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrisisMode {
    None,
    Rose,
    Critical,
    Terminal { seconds_remaining: u64 },
}

// ── Pattern Stamp ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternStamp {
    None,
    Nominal,
    Trending,
    Volatile,
    Adversarial,
    Unknown,
}

impl PatternStamp {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Nominal => "PATTERN: NOMINAL",
            Self::Trending => "PATTERN: TRENDING",
            Self::Volatile => "PATTERN: VOLATILE",
            Self::Adversarial => "PATTERN: ADVERSARIAL",
            Self::Unknown => "PATTERN: UNKNOWN",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Nominal => SUCCESS,
            Self::Trending => ROSE,
            Self::Volatile => WARNING,
            Self::Adversarial => ROSE_BRIGHT,
            Self::Unknown => DREAM,
            Self::None => TEXT_GHOST,
        }
    }
}

// ── Status Stamp ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusStamp {
    Refused,
    Vanished,
    ApproachingLimits,
    TestamentInProgress,
}

impl StatusStamp {
    pub fn lines(self) -> &'static [&'static str] {
        match self {
            Self::Refused => &["    ＲＥＦＵＳＥＤ    "],
            Self::Vanished => &["    ＶＡＮＩＳＨＥＤ   "],
            Self::ApproachingLimits => &["    APPROACHING", "    LIMITS      "],
            Self::TestamentInProgress => &["    TESTAMENT", "    IN PROGRESS "],
        }
    }
}

// ── Unit Array ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitState {
    Normal,
    Alert,
    Breach,
    Cascade,
    Dead,
}

pub struct UnitCell {
    pub id: String,
    pub fill_fraction: f64,
    pub state: UnitState,
}

pub struct UnitArray {
    pub units: Vec<UnitCell>,
    pub unit_width: u16,
    pub unit_height: u16,
}

impl UnitArray {
    pub fn new() -> Self {
        Self {
            units: Vec::new(),
            unit_width: 8,
            unit_height: 3,
        }
    }

    pub fn render(&self, _buf: &mut Buffer, _area: Rect) {
        // Stub: full rendering deferred to integration plan
    }
}

// ── AT Field ───────────────────────────────────────────────────────

pub struct AtField {
    pub health: f64,
}

impl AtField {
    pub fn render(&self, _buf: &mut Buffer, _area: Rect) {
        // Stub: diamond wireframe rendering
    }
}

// ── Psychographic Display ──────────────────────────────────────────

pub struct PsychographicDisplay {
    pub pad_pleasure: f64,
    pub pad_arousal: f64,
    pub context_util: f64,
}

impl PsychographicDisplay {
    pub fn density(&self) -> f64 {
        self.pad_arousal.max(0.0) * 0.3
            + (1.0 - self.pad_pleasure).clamp(0.0, 1.0) * 0.3
            + self.context_util * 0.4
    }

    pub fn overflow_cells(&self) -> u32 {
        let d = self.density();
        if d > 0.8 {
            ((d - 0.8) * 5.0).round() as u32
        } else {
            0
        }
    }

    pub fn render(&self, _buf: &mut Buffer, _area: Rect) {
        // Stub: braille density scribble rendering
    }
}

// ── Defense Layer Display ──────────────────────────────────────────

pub struct DefenseLayerDisplay {
    pub layers: [bool; 7],
}

impl DefenseLayerDisplay {
    pub fn new() -> Self {
        Self { layers: [true; 7] }
    }

    pub fn render(&self, _buf: &mut Buffer, _area: Rect) {
        // Stub: 7-layer B7 defense visualization
    }
}

// ── NERV Chrome ────────────────────────────────────────────────────

pub struct NervChrome {
    pub crisis: CrisisMode,
    pub pattern: PatternStamp,
    pub active_status: Option<StatusStamp>,
    pub status_hold_ms: u64,
    pub stripe_rows: u16,
}

impl NervChrome {
    pub fn new() -> Self {
        Self {
            crisis: CrisisMode::None,
            pattern: PatternStamp::None,
            active_status: None,
            status_hold_ms: 0,
            stripe_rows: 1,
        }
    }

    pub fn render(&self, buf: &mut Buffer, area: Rect, _tick_ms: u64) {
        match self.crisis {
            CrisisMode::None => {}
            CrisisMode::Rose => {}
            CrisisMode::Critical => {
                self.render_hazard_stripes(buf, area, 1);
                self.render_status_bar_override(buf, area);
                self.dim_non_essential_panels(buf, area);
            }
            CrisisMode::Terminal { seconds_remaining } => {
                let stripe_rows = self.compute_terminal_stripe_rows(seconds_remaining);
                self.render_hazard_stripes(buf, area, stripe_rows);
                self.render_death_counter(buf, area, seconds_remaining);
                self.dim_non_essential_panels(buf, area);
            }
        }
        if let Some(stamp) = self.active_status {
            self.render_status_stamp(buf, area, stamp);
        }
        self.render_pattern_stamp(buf, area, self.pattern);
    }

    fn render_hazard_stripes(&self, buf: &mut Buffer, area: Rect, rows: u16) {
        let stripe_style = Style::default().fg(ROSE_BRIGHT).bg(BG_VOID);
        for row in 0..rows {
            let y_top = area.y + row;
            let y_bot = area.y + area.height.saturating_sub(1 + row);
            for x in area.x..area.x + area.width {
                let ch = if (x + area.y + row) % 2 == 0 {
                    '╱'
                } else {
                    '╲'
                };
                for &y in &[y_top, y_bot] {
                    if y < area.y + area.height {
                        if buf.area.contains(ratatui::layout::Position::new(x, y)) {
                            let cell = buf.get_mut(x, y);
                            cell.set_char(ch);
                            cell.set_style(stripe_style);
                        }
                    }
                }
            }
        }
    }

    fn compute_terminal_stripe_rows(&self, secs: u64) -> u16 {
        if secs < 60 {
            (60 - secs) as u16 / 5 + 1
        } else {
            1
        }
    }

    fn render_status_bar_override(&self, buf: &mut Buffer, area: Rect) {
        let y = area.y + area.height.saturating_sub(1);
        let text = "\u{2308} \u{FF23}\u{FF2F}\u{FF2E}\u{FF24}\u{FF29}\u{FF34}\u{FF29}\u{FF2F}\u{FF2E}: \u{FF23}\u{FF32}\u{FF29}\u{FF34}\u{FF29}\u{FF23}\u{FF21}\u{FF2C} \u{230B}";
        let style = Style::default().fg(ROSE_BRIGHT).bg(BG_MID);
        buf.set_string(area.x, y, text, style);
    }

    fn render_death_counter(&self, buf: &mut Buffer, area: Rect, secs: u64) {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        let cx = area.x + area.width / 2;
        let cy = area.y + 1;
        let style = Style::default()
            .fg(BONE)
            .bg(BG_VOID)
            .add_modifier(Modifier::BOLD);
        if cx >= 10 && cy < area.y + area.height {
            buf.set_string(
                cx - 10,
                cy,
                "\u{2308} \u{FF34}\u{FF29}\u{FF2D}\u{FF25} \u{FF32}\u{FF25}\u{FF2D}\u{FF21}\u{FF29}\u{FF2E}\u{FF29}\u{FF2E}\u{FF27} \u{230B}",
                style,
            );
        }
        if cx >= 6 && cy + 1 < area.y + area.height {
            buf.set_string(
                cx - 6,
                cy + 1,
                &format!("    {:02}:{:02}:{:02}", h, m, s),
                style,
            );
        }
    }

    fn dim_non_essential_panels(&self, buf: &mut Buffer, area: Rect) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if buf.area.contains(ratatui::layout::Position::new(x, y)) {
                    let cell = buf.get_mut(x, y);
                    if cell.fg != BONE && cell.fg != ROSE_BRIGHT {
                        cell.modifier |= Modifier::DIM;
                    }
                }
            }
        }
    }

    fn render_pattern_stamp(&self, buf: &mut Buffer, area: Rect, stamp: PatternStamp) {
        if stamp == PatternStamp::None {
            return;
        }
        let label = stamp.label();
        let color = stamp.color();
        let x = area
            .x
            .saturating_add(area.width.saturating_sub(label.len() as u16 + 4));
        let y = area.y;
        let style = Style::default().fg(color).bg(BG_MID);
        buf.set_string(x, y, "\u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}", style);
        if y + 1 < area.y + area.height {
            buf.set_string(x, y + 1, &format!("\u{2502} {:20}\u{2502}", label), style);
        }
        if y + 2 < area.y + area.height {
            buf.set_string(x, y + 2, "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}", style);
        }
    }

    fn render_status_stamp(&self, buf: &mut Buffer, area: Rect, stamp: StatusStamp) {
        let lines = stamp.lines();
        let h = lines.len() as u16 + 2;
        let w = 21u16;
        let cx = area.x + area.width / 2 - w / 2;
        let cy = area.y + area.height / 2 - h / 2;
        let style = Style::default().fg(ROSE_BRIGHT).bg(BG_MID);
        buf.set_string(cx, cy, "\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}", style);
        for (i, line) in lines.iter().enumerate() {
            buf.set_string(
                cx,
                cy + 1 + i as u16,
                &format!("\u{2551}{:^19}\u{2551}", line),
                style,
            );
        }
        buf.set_string(cx, cy + h - 1, "\u{255A}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255D}", style);
    }
}

// ── Scanline Post-Processing ───────────────────────────────────────

pub fn render_scanlines(area: Rect, buf: &mut Buffer, phase_offset: bool, strength: f64) {
    for y in area.top()..area.bottom() {
        let is_dark = (y % 2 == 0) ^ phase_offset;
        if is_dark {
            for x in area.left()..area.right() {
                if buf.area.contains(ratatui::layout::Position::new(x, y)) {
                    let cell = buf.get_mut(x, y);
                    if cell.bg == BG_VOID {
                        cell.set_bg(lerp_color_linear(BG_VOID, SCANLINE_DARK, strength as f32));
                    }
                }
            }
        }
    }
}

// ── Background Noise ───────────────────────────────────────────────

pub fn render_background_noise(
    area: Rect,
    buf: &mut Buffer,
    rng: &mut rand::rngs::SmallRng,
    phase: Phase,
) {
    let density = PhaseTokens::noise_density(phase);
    let color = PhaseTokens::noise_color(phase);
    let chars: &[char] = &['\u{2591}', '\u{2592}', '\u{00B7}', '\u{2219}'];
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if rng.r#gen::<f64>() < density {
                let ch = chars[rng.r#gen_range(0..chars.len())];
                if buf.area.contains(ratatui::layout::Position::new(x, y)) {
                    let cell = buf.get_mut(x, y);
                    cell.set_char(ch);
                    cell.set_fg(color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crisis_mode_enum_valid() {
        let modes = [
            CrisisMode::None,
            CrisisMode::Rose,
            CrisisMode::Critical,
            CrisisMode::Terminal {
                seconds_remaining: 120,
            },
        ];
        assert_eq!(modes.len(), 4);
        assert_ne!(modes[0], modes[1]);
        assert_ne!(modes[1], modes[2]);
    }

    #[test]
    fn test_nerv_hazard_stripe_alternates_diagonal_chars() {
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let chrome = NervChrome {
            crisis: CrisisMode::Critical,
            pattern: PatternStamp::None,
            active_status: None,
            status_hold_ms: 0,
            stripe_rows: 1,
        };
        chrome.render_hazard_stripes(&mut buf, area, 1);

        // Check that even and odd x positions have different diagonal chars in top row
        let cell0 = buf.get(0, 0);
        let cell1 = buf.get(1, 0);
        let c0 = cell0.symbol().chars().next().unwrap();
        let c1 = cell1.symbol().chars().next().unwrap();
        assert_ne!(c0, c1, "Adjacent cells should alternate diagonal chars");
        assert!(
            (c0 == '\u{2571}' || c0 == '\u{2572}') && (c1 == '\u{2571}' || c1 == '\u{2572}'),
            "Diagonal chars should be U+2571 or U+2572"
        );
    }

    #[test]
    fn test_terminal_stripe_rows_formula() {
        let chrome = NervChrome::new();
        assert_eq!(chrome.compute_terminal_stripe_rows(0), 13); // (60-0)/5 + 1
        assert_eq!(chrome.compute_terminal_stripe_rows(30), 7); // (60-30)/5 + 1
        assert_eq!(chrome.compute_terminal_stripe_rows(59), 1); // (60-59)/5 + 1
        assert_eq!(chrome.compute_terminal_stripe_rows(60), 1); // >= 60 => 1
        assert_eq!(chrome.compute_terminal_stripe_rows(120), 1);
    }

    #[test]
    fn test_nerv_scanline_darkens_even_rows() {
        let area = Rect::new(0, 0, 10, 4);
        let mut buf = Buffer::empty(area);
        // Set all cells to BG_VOID background
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if buf.area.contains(ratatui::layout::Position::new(x, y)) {
                    let cell = buf.get_mut(x, y);
                    cell.set_bg(BG_VOID);
                }
            }
        }
        render_scanlines(area, &mut buf, false, 0.5);

        // Even rows should be darkened (different from BG_VOID)
        let even_cell = buf.get(0, 0);
        let odd_cell = buf.get(0, 1);
        assert_ne!(
            even_cell.bg, odd_cell.bg,
            "Even rows should be darkened, odd rows unchanged"
        );
        assert_eq!(odd_cell.bg, BG_VOID, "Odd row bg should remain BG_VOID");
    }

    #[test]
    fn test_psychographic_density_formula() {
        let display = PsychographicDisplay {
            pad_pleasure: 0.0,
            pad_arousal: 1.0,
            context_util: 1.0,
        };
        // arousal*0.3 + (1-pleasure)*0.3 + context_util*0.4
        // = 1.0*0.3 + (1.0-0.0)*0.3 + 1.0*0.4 = 0.3 + 0.3 + 0.4 = 1.0
        let d = display.density();
        assert!((d - 1.0).abs() < 1e-9, "Expected 1.0, got {}", d);

        let display2 = PsychographicDisplay {
            pad_pleasure: 1.0,
            pad_arousal: 0.0,
            context_util: 0.0,
        };
        // = 0.0*0.3 + (1.0-1.0)*0.3 + 0.0*0.4 = 0.0
        let d2 = display2.density();
        assert!((d2 - 0.0).abs() < 1e-9, "Expected 0.0, got {}", d2);
    }

    #[test]
    fn test_psychographic_overflow_threshold() {
        let display = PsychographicDisplay {
            pad_pleasure: 0.0,
            pad_arousal: 1.0,
            context_util: 1.0,
        };
        // density = 1.0, overflow = ((1.0 - 0.8) * 5.0) as u32 = 1
        assert_eq!(display.overflow_cells(), 1);

        let display_low = PsychographicDisplay {
            pad_pleasure: 1.0,
            pad_arousal: 0.0,
            context_util: 0.5,
        };
        // density = 0 + 0 + 0.2 = 0.2 < 0.8 => overflow = 0
        assert_eq!(display_low.overflow_cells(), 0);
    }

    #[test]
    fn test_nerv_pattern_stamp_colors_match_spec() {
        assert_eq!(PatternStamp::Nominal.color(), SUCCESS);
        assert_eq!(PatternStamp::Trending.color(), ROSE);
        assert_eq!(PatternStamp::Volatile.color(), WARNING);
        assert_eq!(PatternStamp::Adversarial.color(), ROSE_BRIGHT);
        assert_eq!(PatternStamp::Unknown.color(), DREAM);
    }

    #[test]
    fn test_nerv_crisis_mode_dim_excludes_bone() {
        let area = Rect::new(0, 0, 10, 4);
        let mut buf = Buffer::empty(area);
        // Set some cells to BONE fg and some to TEXT_PRIMARY
        for x in 0..5 {
            if buf.area.contains(ratatui::layout::Position::new(x, 0)) {
                let cell = buf.get_mut(x, 0);
                cell.set_fg(BONE);
            }
        }
        for x in 5..10 {
            if buf.area.contains(ratatui::layout::Position::new(x, 0)) {
                let cell = buf.get_mut(x, 0);
                cell.set_fg(TEXT_PRIMARY);
            }
        }

        let chrome = NervChrome {
            crisis: CrisisMode::Critical,
            pattern: PatternStamp::None,
            active_status: None,
            status_hold_ms: 0,
            stripe_rows: 1,
        };
        chrome.dim_non_essential_panels(&mut buf, area);

        // BONE cells should NOT have DIM modifier
        let bone_cell = buf.get(0, 0);
        assert!(
            !bone_cell.modifier.contains(Modifier::DIM),
            "BONE fg cells should not be dimmed"
        );

        // TEXT_PRIMARY cells should have DIM modifier
        let text_cell = buf.get(5, 0);
        assert!(
            text_cell.modifier.contains(Modifier::DIM),
            "TEXT_PRIMARY fg cells should be dimmed"
        );
    }
}
