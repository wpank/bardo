//! Home screen with live progress bar, ETA countdown, and per-task timing.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    palette::{
        BONE, BORDER, BORDER_ACTIVE, DANGER, DREAM, ROSE, ROSE_BRIGHT, ROSE_DIM, SUCCESS, TEXT_DIM,
        TEXT_GHOST, TEXT_PRIMARY, WARNING,
    },
    screen::{Screen, ScreenId},
    state::{AppAction, AppState, ConnectionStatus, TaskStatus, format_duration},
    widgets::TotalProgressBar,
};

/// Home screen with creature silhouette, pipeline progress, and per-task ETA.
pub(crate) struct HomeScreen {
    focused: bool,
}

impl HomeScreen {
    pub(crate) fn new() -> Self {
        Self { focused: false }
    }
}

impl Screen for HomeScreen {
    fn id(&self) -> ScreenId {
        ScreenId::HearthOverview
    }

    fn title(&self) -> &str {
        "HEARTH"
    }

    fn render(&self, frame: &mut Frame<'_>, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // ── Left: creature silhouette ────────────────────────────
        let creature_border = if self.focused && state.tick_count % 8 < 4 {
            BORDER_ACTIVE
        } else {
            BORDER
        };

        let creature_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(creature_border))
            .title(Span::styled(" SPECTRE ", Style::default().fg(ROSE)));
        let creature_inner = creature_block.inner(chunks[0]);
        frame.render_widget(creature_block, chunks[0]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("  ◉ ◉  ", Style::default().fg(ROSE))),
                Line::from(Span::styled(" ░░░░░ ", Style::default().fg(ROSE_DIM))),
                Line::from(Span::styled(" ░   ░ ", Style::default().fg(ROSE_DIM))),
                Line::from(Span::styled("  ─ ─  ", Style::default().fg(ROSE_DIM))),
            ])
            .alignment(Alignment::Center),
            creature_inner,
        );

        // ── Right: progress + tasks ──────────────────────────────
        let data_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // progress bar (inner: 3 rows)
                Constraint::Length(3), // connection status
                Constraint::Min(0),    // task list
            ])
            .split(chunks[1]);

        // ── Progress bar ─────────────────────────────────────────
        let progress_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" PIPELINE ", Style::default().fg(BONE)))
            .border_style(Style::default().fg(BORDER));
        let progress_inner = progress_block.inner(data_chunks[0]);
        frame.render_widget(progress_block, data_chunks[0]);
        frame.render_widget(
            TotalProgressBar {
                progress: state.progress.progress_fraction(),
                eta_secs: state.progress.eta_remaining_secs(),
                elapsed_secs: state.progress.wall_elapsed_secs(),
                heartbeat: state.atmosphere.heartbeat(),
                complete: state.progress.is_complete(),
            },
            progress_inner,
        );

        // ── Connection status ────────────────────────────────────
        let (status_text, status_color) = match state.connection_status {
            ConnectionStatus::Connected => ("● CONNECTED", SUCCESS),
            ConnectionStatus::Connecting => ("◌ CONNECTING…", WARNING),
            ConnectionStatus::Disconnected => ("○ DISCONNECTED", DANGER),
        };
        frame.render_widget(
            Paragraph::new(status_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(BORDER)),
                )
                .style(Style::default().fg(status_color))
                .alignment(Alignment::Center),
            data_chunks[1],
        );

        // ── Task list with per-task ETA and elapsed ──────────────
        let task_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" PHASES ", Style::default().fg(BONE)))
            .border_style(Style::default().fg(if self.focused { BORDER_ACTIVE } else { BORDER }));
        let task_inner = task_block.inner(data_chunks[2]);
        frame.render_widget(task_block, data_chunks[2]);

        let mut lines: Vec<Line> = Vec::new();

        for task in &state.progress.tasks {
            let (icon_display, icon_style, name_style, time_str) = match task.status {
                TaskStatus::Done => (
                    "✓".to_string(),
                    Style::default().fg(SUCCESS),
                    Style::default().fg(TEXT_DIM),
                    format_duration(task.elapsed_secs),
                ),
                TaskStatus::Active => {
                    let spinner = state.atmosphere.spinner();
                    let pulse = pulse_color(ROSE, state.atmosphere.heartbeat());
                    let elapsed = format_duration(task.elapsed_secs);
                    let remaining = (task.estimated_secs - task.elapsed_secs).max(0.0);
                    let eta = format_duration(remaining);
                    (
                        format!("{spinner}"),
                        Style::default().fg(pulse).add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(ROSE_BRIGHT)
                            .add_modifier(Modifier::BOLD),
                        format!(
                            "{elapsed} / ~{}  ETA {eta}",
                            format_duration(task.estimated_secs)
                        ),
                    )
                }
                TaskStatus::Pending => (
                    "○".to_string(),
                    Style::default().fg(TEXT_GHOST),
                    Style::default().fg(TEXT_GHOST),
                    format!("~{}", format_duration(task.estimated_secs)),
                ),
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" {icon_display} "), icon_style),
                Span::styled(format!("{:<14}", task.name), name_style),
                Span::styled(
                    format!(" {time_str}"),
                    match task.status {
                        TaskStatus::Active => Style::default().fg(DREAM),
                        TaskStatus::Done => Style::default().fg(TEXT_DIM),
                        TaskStatus::Pending => Style::default().fg(TEXT_GHOST),
                    },
                ),
            ]));
        }

        // Summary line
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" elapsed: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                format_duration(state.progress.wall_elapsed_secs()),
                Style::default().fg(TEXT_PRIMARY),
            ),
            Span::styled("   ETA: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                if state.progress.is_complete() {
                    "done".to_string()
                } else {
                    format_duration(state.progress.eta_remaining_secs())
                },
                Style::default().fg(if state.progress.is_complete() {
                    SUCCESS
                } else {
                    ROSE
                }),
            ),
        ]));

        // Waveform and help
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" wave: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                tick_waveform(state.tick_count, 18),
                Style::default().fg(ROSE_DIM),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            " q=quit  Tab=next  Shift+Tab=prev",
            Style::default().fg(TEXT_GHOST),
        )));

        frame.render_widget(Paragraph::new(lines), task_inner);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => Some(AppAction::Quit),
            KeyCode::Tab => Some(AppAction::NextScreen),
            KeyCode::BackTab => Some(AppAction::PrevScreen),
            _ => None,
        }
    }

    fn on_focus(&mut self) {
        self.focused = true;
    }

    fn on_blur(&mut self) {
        self.focused = false;
    }
}

/// Modulate a color's brightness with the heartbeat oscillator.
fn pulse_color(base: Color, heartbeat: f64) -> Color {
    match base {
        Color::Rgb(r, g, b) => {
            let scale = heartbeat.clamp(0.9, 1.1);
            Color::Rgb(
                (r as f64 * scale).min(255.0) as u8,
                (g as f64 * scale).min(255.0) as u8,
                (b as f64 * scale).min(255.0) as u8,
            )
        }
        other => other,
    }
}

fn tick_waveform(tick_count: u64, width: usize) -> String {
    const SYMBOLS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let phase = tick_count as f64 * 0.17;

    (0..width)
        .map(|index| {
            let sample = (phase + index as f64 * 0.42).sin();
            let normalized = ((sample + 1.0) * 0.5) * (SYMBOLS.len() - 1) as f64;
            SYMBOLS[normalized.round() as usize]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppAction, ConnectionStatus, MockVitality};
    use crossterm::event::KeyModifiers;

    #[test]
    fn home_screen_defaults_to_unfocused() {
        assert!(!HomeScreen::new().focused);
    }

    #[test]
    fn home_screen_handles_global_keys() {
        let mut screen = HomeScreen::new();

        assert_eq!(
            screen.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(AppAction::Quit)
        );
        assert_eq!(
            screen.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            Some(AppAction::NextScreen)
        );
        assert_eq!(
            screen.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE)),
            Some(AppAction::PrevScreen)
        );
    }

    #[test]
    fn tick_waveform_has_requested_width() {
        assert_eq!(tick_waveform(123, 18).chars().count(), 18);
    }

    #[test]
    fn connection_status_labels_cover_all_variants() {
        assert_eq!(ConnectionStatus::Connected.label(), "CONNECTED");
        assert_eq!(ConnectionStatus::Connecting.label(), "CONNECTING…");
        assert_eq!(ConnectionStatus::Disconnected.label(), "DISCONNECTED");
        assert_eq!(MockVitality::default(), MockVitality { value: 0.75 });
    }
}
