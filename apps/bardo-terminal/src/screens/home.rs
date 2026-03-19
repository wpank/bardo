//! Home screen placeholder for the terminal scaffold.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::{
    palette::{
        BG_RAISED, BONE, BORDER, BORDER_ACTIVE, DANGER, ROSE, ROSE_DIM, SUCCESS, TEXT_DIM,
        TEXT_GHOST, TEXT_PRIMARY, WARNING,
    },
    screen::{Screen, ScreenId},
    state::{AppAction, AppState, ConnectionStatus},
};

/// Placeholder home screen that renders a creature silhouette and status blocks.
pub(crate) struct HomeScreen {
    focused: bool,
}

impl HomeScreen {
    /// Creates a new home screen placeholder.
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

        let data_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(chunks[1]);

        let vitality_pct = (state.vitality.value.clamp(0.0, 1.0) * 100.0).round() as u16;
        let vitality_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" VITALITY ", Style::default().fg(BONE)))
            .border_style(Style::default().fg(BORDER));
        frame.render_widget(
            Gauge::default()
                .block(vitality_block)
                .gauge_style(Style::default().fg(SUCCESS).bg(BG_RAISED))
                .percent(vitality_pct),
            data_chunks[0],
        );

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

        let info_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if self.focused { BORDER_ACTIVE } else { BORDER }));
        let info_inner = info_block.inner(data_chunks[2]);
        frame.render_widget(info_block, data_chunks[2]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("tick: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(
                        state.tick_count.to_string(),
                        Style::default().fg(ROSE).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("wave: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(
                        tick_waveform(state.tick_count, 18),
                        Style::default().fg(ROSE_DIM),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("layout: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(state.layout.label(), Style::default().fg(TEXT_PRIMARY)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "q=quit  Tab=next  Shift+Tab=prev",
                    Style::default().fg(TEXT_GHOST),
                )),
            ]),
            info_inner,
        );
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
