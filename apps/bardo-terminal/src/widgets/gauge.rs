//! Gauge widgets for vitality and confidence-style metrics.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::palette::{
    BG_RAISED, BLOCK_FULL, BLOCK_LIGHT, BONE, BORDER, ROSE_BRIGHT, ROSE_DIM, SUCCESS, WARNING,
};

const CYAN: Color = Color::Rgb(88, 160, 170);
const DECLINING_ORANGE: Color = Color::Rgb(180, 100, 60);

/// Placeholder vitality phase.
///
/// TODO Plan 70a: replace with `golem_core::BehavioralPhase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MockPhase {
    /// Highest vitality tier.
    Thriving,
    /// Nominal operating tier.
    Stable,
    /// Resource-conserving tier.
    Conservation,
    /// Degrading tier.
    Declining,
    /// End-of-life tier.
    Terminal,
}

/// Maps a normalized vitality scalar to the scaffold [`MockPhase`] band.
pub(crate) fn vitality_to_phase(value: f64) -> MockPhase {
    match value.clamp(0.0, 1.0) {
        v if v >= 0.85 => MockPhase::Thriving,
        v if v >= 0.65 => MockPhase::Stable,
        v if v >= 0.45 => MockPhase::Conservation,
        v if v >= 0.20 => MockPhase::Declining,
        _ => MockPhase::Terminal,
    }
}

impl MockPhase {
    /// Returns the phase-specific gauge fill color.
    pub(crate) const fn gauge_color(self) -> Color {
        match self {
            Self::Thriving => SUCCESS,
            Self::Stable => CYAN,
            Self::Conservation => WARNING,
            Self::Declining => DECLINING_ORANGE,
            Self::Terminal => ROSE_BRIGHT,
        }
    }

    /// Returns the fixed display label for the phase.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Thriving => "THRIVING",
            Self::Stable => "STABLE",
            Self::Conservation => "CONSERVATION",
            Self::Declining => "DECLINING",
            Self::Terminal => "TERMINAL",
        }
    }
}

/// Phase-colored vitality gauge.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VitalityGauge {
    /// Normalized vitality value.
    pub(crate) value: f64,
    /// Gauge label.
    pub(crate) label: String,
    /// Placeholder phase until live mortality state is available.
    pub(crate) phase: MockPhase,
}

/// Confidence gauge using scalar thresholds.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConfidenceGauge {
    /// Normalized confidence value.
    pub(crate) value: f64,
    /// Gauge label.
    pub(crate) label: String,
}

/// Accuracy gauge using the same thresholds as confidence.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AccuracyGauge {
    /// Normalized prediction accuracy.
    pub(crate) value: f64,
    /// Gauge label.
    pub(crate) label: String,
}

fn confidence_color(value: f64) -> Color {
    if value >= 0.75 {
        SUCCESS
    } else if value >= 0.50 {
        CYAN
    } else if value >= 0.25 {
        WARNING
    } else {
        ROSE_DIM
    }
}

fn in_bounds(buf: &Buffer, x: u16, y: u16) -> bool {
    let area = buf.area();
    x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom()
}

fn render_gauge(area: Rect, buf: &mut Buffer, value: f64, label: &str, fill_color: Color) {
    if area.width < 1 || area.height < 1 {
        return;
    }

    let clamped = value.clamp(0.0, 1.0);
    let fill_width = (clamped * area.width as f64) as u16;
    let gauge_y = if area.height >= 2 {
        let label_text = format!(" {} {:.0}% ", label, clamped * 100.0);
        buf.set_stringn(
            area.x,
            area.y,
            label_text,
            area.width as usize,
            Style::default().fg(BONE).add_modifier(Modifier::BOLD),
        );
        area.y + 1
    } else {
        area.y
    };

    for offset in 0..area.width {
        let x = area.x + offset;
        if in_bounds(buf, x, gauge_y) {
            let cell = buf.get_mut(x, gauge_y);
            if offset < fill_width {
                cell.set_char(BLOCK_FULL);
                cell.set_style(Style::default().fg(fill_color).bg(BG_RAISED));
            } else {
                cell.set_char(BLOCK_LIGHT);
                cell.set_style(Style::default().fg(BORDER).bg(BG_RAISED));
            }
        }
    }
}

impl Widget for VitalityGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_gauge(area, buf, self.value, &self.label, self.phase.gauge_color());
    }
}

impl Widget for ConfidenceGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_gauge(
            area,
            buf,
            self.value,
            &self.label,
            confidence_color(self.value),
        );
    }
}

impl Widget for AccuracyGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_gauge(
            area,
            buf,
            self.value,
            &self.label,
            confidence_color(self.value),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vitality_gauge_colors() {
        assert_eq!(MockPhase::Thriving.gauge_color(), SUCCESS);
        assert_eq!(MockPhase::Terminal.gauge_color(), ROSE_BRIGHT);
    }

    #[test]
    fn vitality_to_phase_thresholds() {
        assert_eq!(vitality_to_phase(0.9), MockPhase::Thriving);
        assert_eq!(vitality_to_phase(0.7), MockPhase::Stable);
        assert_eq!(vitality_to_phase(0.5), MockPhase::Conservation);
        assert_eq!(vitality_to_phase(0.25), MockPhase::Declining);
        assert_eq!(vitality_to_phase(0.0), MockPhase::Terminal);
        assert_eq!(vitality_to_phase(2.0), MockPhase::Thriving);
        assert_eq!(vitality_to_phase(-1.0), MockPhase::Terminal);
    }

    #[test]
    fn test_confidence_color_thresholds() {
        assert_eq!(confidence_color(0.80), SUCCESS);
        assert_eq!(confidence_color(0.60), CYAN);
        assert_eq!(confidence_color(0.30), WARNING);
        assert_eq!(confidence_color(0.10), ROSE_DIM);

        // Boundary values
        assert_eq!(confidence_color(0.75), SUCCESS);
        assert_eq!(confidence_color(0.50), CYAN);
        assert_eq!(confidence_color(0.25), WARNING);
        assert_eq!(confidence_color(0.0), ROSE_DIM);
        assert_eq!(confidence_color(1.0), SUCCESS);
        assert_eq!(confidence_color(0.95), SUCCESS);
    }

    /// INV-004: Fill width scales with gauge value, clamped to terminal width.
    #[test]
    fn test_vitality_gauge_fill_width() {
        for &value in &[0.0_f64, 0.25, 0.5, 0.75, 1.0, -0.5, 1.5] {
            for &area_width in &[1u16, 10, 80, 255] {
                let clamped = value.clamp(0.0, 1.0);
                let fill_width = (clamped * area_width as f64) as u16;
                assert!(
                    fill_width <= area_width,
                    "fill_width {fill_width} > area_width {area_width} for value {value}"
                );
            }
        }

        // Render test: value=0 should have 0 filled cells
        let gauge = VitalityGauge {
            value: 0.0,
            label: "test".into(),
            phase: MockPhase::Stable,
        };
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);
        gauge.render(area, &mut buf);
        // Bar row is y=1; all cells should be unfilled (BLOCK_LIGHT)
        for x in 0..20 {
            assert_eq!(buf.get(x, 1).symbol(), "░");
        }

        // Render test: value=1 should have all filled cells
        let gauge2 = VitalityGauge {
            value: 1.0,
            label: "test".into(),
            phase: MockPhase::Thriving,
        };
        let mut buf2 = Buffer::empty(area);
        gauge2.render(area, &mut buf2);
        for x in 0..20 {
            assert_eq!(buf2.get(x, 1).symbol(), "█");
        }
    }
}
