use crate::tui::theme::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Done,
    Error,
    Warning,
    Revision,
    Paused,
    Idle,
    Pending,
}

pub fn status_style(status: Status) -> Style {
    match status {
        Status::Active => Style::default()
            .fg(Theme::VOID)
            .bg(Theme::ROSE)
            .add_modifier(Modifier::BOLD),
        Status::Done => Style::default()
            .fg(Theme::VOID)
            .bg(Theme::SAGE)
            .add_modifier(Modifier::BOLD),
        Status::Error => Style::default()
            .fg(Theme::VOID)
            .bg(Theme::EMBER)
            .add_modifier(Modifier::BOLD),
        Status::Warning => Style::default().fg(Theme::VOID).bg(Theme::WARNING),
        Status::Revision => Style::default().fg(Theme::VOID).bg(Theme::WARNING),
        Status::Paused => Style::default().fg(Theme::VOID).bg(Theme::DREAM),
        Status::Idle => Style::default().fg(Theme::TEXT_GHOST).bg(Theme::BG_RAISED),
        Status::Pending => Style::default().fg(Theme::TEXT_DIM).bg(Theme::BG_RAISED),
    }
}

/// Render a styled status badge: " label " with status-appropriate colors.
pub fn badge(label: &str, status: Status) -> Span<'static> {
    Span::styled(format!(" {label} "), status_style(status))
}

/// Render a pulsed status badge for active items.
/// `heartbeat` is a 0.0-1.0+ breathing value from the atmosphere.
pub fn badge_pulsed(label: &str, status: Status, heartbeat: f64) -> Span<'static> {
    let base = status_style(status);
    if status == Status::Active {
        // Pulse the background brightness
        let scale = (0.8 + 0.4 * heartbeat).min(1.3);
        let bg = match base.bg {
            Some(ratatui::style::Color::Rgb(r, g, b)) => ratatui::style::Color::Rgb(
                (r as f64 * scale).min(255.0) as u8,
                (g as f64 * scale).min(255.0) as u8,
                (b as f64 * scale).min(255.0) as u8,
            ),
            other => other.unwrap_or(Theme::ROSE),
        };
        Span::styled(
            format!(" {label} "),
            Style::default()
                .fg(Theme::VOID)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        badge(label, status)
    }
}
