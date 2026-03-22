use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::{self, Theme};

/// Render a horizontal context gauge with threshold markers at 80% and 90%.
/// fill_pct: 0.0..=1.0
pub fn render(
    f: &mut Frame,
    area: Rect,
    fill_pct: f64,
    label: &str,
    accent: ratatui::style::Color,
) {
    let width = area.width as usize;
    if width < 10 {
        return;
    }

    let label_width = label.len() + 1;
    let pct_width = 5; // " 99%"
    let gauge_width = width.saturating_sub(label_width + pct_width + 2);

    let filled = (fill_pct * gauge_width as f64) as usize;
    let _empty = gauge_width.saturating_sub(filled);

    let bar_color = theme::gradient_context().sample(fill_pct);

    // Threshold markers at 80% and 90%
    let mark_80 = (0.80 * gauge_width as f64) as usize;
    let mark_90 = (0.90 * gauge_width as f64) as usize;

    // Build gauge string with threshold markers
    let mut gauge_chars: Vec<(char, ratatui::style::Color)> = Vec::with_capacity(gauge_width);
    for i in 0..gauge_width {
        if i < filled {
            let is_threshold = i == mark_80 || i == mark_90;
            let ch = if is_threshold { '┊' } else { '█' };
            gauge_chars.push((ch, bar_color));
        } else {
            let is_threshold = i == mark_80 || i == mark_90;
            let ch = if is_threshold { '┊' } else { '░' };
            let color = if is_threshold {
                if i == mark_90 {
                    Theme::EMBER
                } else {
                    Theme::WARNING
                }
            } else {
                Theme::TEXT_GHOST
            };
            gauge_chars.push((ch, color));
        }
    }

    let mut spans = vec![Span::styled(
        format!("{label} "),
        Style::default().fg(accent),
    )];

    // Batch consecutive same-color chars into spans
    let mut current_color = gauge_chars
        .first()
        .map(|c| c.1)
        .unwrap_or(Theme::TEXT_GHOST);
    let mut current_str = String::new();
    for (ch, color) in &gauge_chars {
        if *color == current_color {
            current_str.push(*ch);
        } else {
            spans.push(Span::styled(
                std::mem::take(&mut current_str),
                Style::default().fg(current_color),
            ));
            current_color = *color;
            current_str.push(*ch);
        }
    }
    if !current_str.is_empty() {
        spans.push(Span::styled(
            current_str,
            Style::default().fg(current_color),
        ));
    }

    // Percentage
    let pct_display = (fill_pct * 100.0) as u32;
    let pct_color = if fill_pct > 0.9 {
        Theme::EMBER
    } else if fill_pct > 0.8 {
        Theme::WARNING
    } else {
        Theme::FG_DIM
    };
    spans.push(Span::styled(
        format!(" {pct_display}%"),
        Style::default().fg(pct_color),
    ));

    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}

/// Render a compact inline gauge (no label, no percentage). Returns the span.
pub fn inline_gauge(fill_pct: f64, width: usize) -> Vec<Span<'static>> {
    let filled = (fill_pct * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    let bar_color = theme::gradient_context().sample(fill_pct);

    vec![
        Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
        Span::styled("░".repeat(empty), Style::default().fg(Theme::TEXT_GHOST)),
    ]
}
