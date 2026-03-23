use ratatui::style::{Color, Style};
use ratatui::text::Span;

use super::color::{amber_gradient, ember_gradient, sage_gradient, Gradient};
use super::theme::Theme;

/// Render a gradient-colored bar with trailing edge and optional breathing modulation.
///
/// Characters: `█` (filled, per-cell gradient color) + `▓` (trailing edge, dim) + `░` (empty, TEXT_GHOST)
pub fn gradient_bar(
    width: usize,
    fill_pct: f64,
    gradient: &Gradient,
    breathing: Option<f64>,
) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }

    let pct = fill_pct.clamp(0.0, 1.0);
    let brightness = breathing.unwrap_or(1.0);

    let filled = (pct * width as f64) as usize;
    let has_trailing = filled > 0 && filled < width;
    let empty = width
        .saturating_sub(filled)
        .saturating_sub(if has_trailing { 1 } else { 0 });

    let mut spans = Vec::new();

    // Filled cells with per-cell gradient color
    for i in 0..filled {
        let t = if filled > 1 {
            i as f64 / (filled - 1) as f64
        } else {
            1.0
        };
        let (r, g, b) = gradient.sample(t * pct);
        let r = (r as f64 * brightness).min(255.0) as u8;
        let g = (g as f64 * brightness).min(255.0) as u8;
        let b = (b as f64 * brightness).min(255.0) as u8;
        spans.push(Span::styled(
            "█".to_string(),
            Style::default().fg(ratatui::style::Color::Rgb(r, g, b)),
        ));
    }

    // Trailing edge
    if has_trailing {
        let (r, g, b) = gradient.sample(pct);
        let dim = 0.5 * brightness;
        let r = (r as f64 * dim).min(255.0) as u8;
        let g = (g as f64 * dim).min(255.0) as u8;
        let b = (b as f64 * dim).min(255.0) as u8;
        spans.push(Span::styled(
            "▓".to_string(),
            Style::default().fg(ratatui::style::Color::Rgb(r, g, b)),
        ));
    }

    // Empty cells
    if empty > 0 {
        spans.push(Span::styled(
            "░".repeat(empty),
            Style::default().fg(Theme::TEXT_GHOST),
        ));
    }

    spans
}

/// Semantic progress bar: color shifts based on completion %
/// Red (0-30%) -> Amber (30-70%) -> Green (70-100%)
pub fn semantic_bar(width: usize, fill_pct: f64, breathing: Option<f64>) -> Vec<Span<'static>> {
    let pct = fill_pct.clamp(0.0, 1.0);
    let gradient = if pct < 0.3 {
        ember_gradient()
    } else if pct < 0.7 {
        amber_gradient()
    } else {
        sage_gradient()
    };
    gradient_bar(width, pct, &gradient, breathing)
}

/// Compact semantic bar without breathing (for dense layouts)
pub fn mini_semantic_bar(width: usize, fill_pct: f64) -> Vec<Span<'static>> {
    semantic_bar(width, fill_pct, None)
}

/// Get semantic color based on completion percentage
/// Red (0-30%) -> Amber (30-70%) -> Green (70-100%)
pub fn semantic_color(fill_pct: f64) -> Color {
    let pct = fill_pct.clamp(0.0, 1.0);
    if pct < 0.3 {
        Theme::EMBER
    } else if pct < 0.7 {
        Theme::BONE
    } else {
        Theme::SAGE
    }
}
