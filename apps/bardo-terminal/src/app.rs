//! Application state machine and render loop for the terminal scaffold.

#[allow(unused_imports)]
use golem_core::event::EventFabric;

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    layout::{LayoutBreakpoint, compute_layout},
    palette::{BG_MID, BG_RAISED, BORDER_ACTIVE, BORDER_DREAM, ROSE, ROSE_DIM, TEXT_DIM},
    screen::{ScreenId, ScreenRegistry, StubScreen},
    screens::HomeScreen,
    state::{AppAction, AppState, format_duration},
    sys_stats::SysStats,
};

const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

/// Application scaffold that owns terminal state and screen navigation.
pub(crate) struct App {
    state: AppState,
    screens: ScreenRegistry,
    active_screen: ScreenId,
    should_quit: bool,
    sys_stats: SysStats,
}

impl App {
    /// Creates the scaffold with all screens registered.
    pub(crate) fn new() -> Self {
        let mut screens = ScreenRegistry::new();

        for &screen_id in ScreenId::all() {
            match screen_id {
                ScreenId::HearthOverview => screens.register(Box::new(HomeScreen::new())),
                other => screens.register(Box::new(StubScreen::new(
                    other,
                    format!("{} / {}", other.window_name(), other.tab_name()),
                ))),
            }
        }

        let mut app = Self {
            state: AppState::default(),
            screens,
            active_screen: ScreenId::HearthOverview,
            should_quit: false,
            sys_stats: SysStats::new(),
        };
        app.focus_active_screen();
        app
    }

    /// Runs the 60 FPS render loop until the user quits.
    pub(crate) fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let mut last_frame = Instant::now();

        loop {
            let frame_start = Instant::now();
            let dt = frame_start.duration_since(last_frame).as_secs_f64();

            let timeout = FRAME_DURATION.saturating_sub(last_frame.elapsed());
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Some(action) = self.handle_key(key) {
                            self.apply_action(action);
                        }
                    }
                    Event::Resize(width, height) => {
                        self.apply_action(AppAction::Resize(width, height));
                    }
                    _ => {}
                }
            }

            self.state.tick_count = self.state.tick_count.wrapping_add(1);
            self.state.atmosphere.tick(dt);
            self.state.progress.tick(dt);
            if let Some(sys) = self.sys_stats.tick(dt) {
                self.state.sys = sys;
            }
            terminal.draw(|frame| self.render(frame))?;

            last_frame = frame_start;

            if self.should_quit {
                break;
            }

            let elapsed = frame_start.elapsed();
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        self.screens
            .get_mut(&self.active_screen)
            .and_then(|screen| screen.handle_key(key))
    }

    fn apply_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => {
                self.should_quit = true;
            }
            AppAction::NextScreen => self.switch_screen(1),
            AppAction::PrevScreen => self.switch_screen(-1),
            AppAction::Resize(width, _height) => {
                self.state.layout = LayoutBreakpoint::from_cols(width);
            }
        }
    }

    fn switch_screen(&mut self, step: isize) {
        let screens = ScreenId::all();
        let Some(current_index) = screens
            .iter()
            .position(|candidate| *candidate == self.active_screen)
        else {
            return;
        };

        let next_index =
            (current_index as isize + step).rem_euclid(screens.len() as isize) as usize;
        let next_screen = screens[next_index];

        if next_screen == self.active_screen {
            return;
        }

        if let Some(screen) = self.screens.get_mut(&self.active_screen) {
            screen.on_blur();
        }

        self.active_screen = next_screen;
        self.focus_active_screen();
    }

    fn focus_active_screen(&mut self) {
        if let Some(screen) = self.screens.get_mut(&self.active_screen) {
            screen.on_focus();
        }
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let size = frame.size();
        let (sidebar_area, content_area) = compute_layout(size, self.state.layout);

        if sidebar_area.width > 0 {
            let sidebar_border = if self.state.tick_count % 8 < 4 {
                BORDER_ACTIVE
            } else {
                BORDER_DREAM
            };
            let sidebar_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(sidebar_border))
                .style(Style::default().bg(BG_RAISED));
            let sidebar_inner = sidebar_block.inner(sidebar_area);
            frame.render_widget(sidebar_block, sidebar_area);
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled("SPECTRE", Style::default().fg(ROSE))),
                    Line::from(Span::styled(
                        self.state.layout.label(),
                        Style::default().fg(TEXT_DIM),
                    )),
                ])
                .alignment(Alignment::Center),
                sidebar_inner,
            );
        }

        let header_area = Rect::new(size.x, size.y, size.width, 1);
        let footer_area = Rect::new(
            size.x,
            size.y.saturating_add(size.height.saturating_sub(1)),
            size.width,
            1,
        );

        if let Some(screen) = self.screens.get(&self.active_screen) {
            let eta_str = if self.state.progress.is_complete() {
                "done".to_string()
            } else {
                format_duration(self.state.progress.eta_remaining_secs())
            };
            let header_text = format!(
                " {}  •  {} / {}   ETA {}   elapsed {}   {} ",
                screen.title(),
                self.active_screen.window_name(),
                self.active_screen.tab_name(),
                eta_str,
                format_duration(self.state.progress.wall_elapsed_secs()),
                self.state.connection_status.label(),
            );
            frame.render_widget(
                Paragraph::new(header_text)
                    .block(Block::default().style(Style::default().bg(BG_MID)))
                    .style(Style::default().fg(ROSE).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Left),
                header_area,
            );

            let footer_text = format!(
                " q=quit   Tab=next screen   Shift+Tab=previous screen   {}/{} screens   {} panels ",
                self.screen_index(self.active_screen).unwrap_or(0) + 1,
                ScreenId::all().len(),
                self.state.layout.panel_count(),
            );
            frame.render_widget(
                Paragraph::new(footer_text)
                    .block(Block::default().style(Style::default().bg(BG_MID)))
                    .style(Style::default().fg(ROSE_DIM)),
                footer_area,
            );

            screen.render(frame, content_area, &self.state);
        }
    }

    fn screen_index(&self, screen_id: ScreenId) -> Option<usize> {
        ScreenId::all()
            .iter()
            .position(|candidate| *candidate == screen_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_starts_on_the_home_screen() {
        let app = App::new();

        assert_eq!(app.active_screen, ScreenId::HearthOverview);
        assert_eq!(app.state.layout, LayoutBreakpoint::Standard);
        assert!(!app.should_quit);
    }

    #[test]
    fn app_cycles_screens_in_catalog_order() {
        let mut app = App::new();
        app.apply_action(AppAction::NextScreen);
        assert_eq!(app.active_screen, ScreenId::HearthSignals);
        app.apply_action(AppAction::PrevScreen);
        assert_eq!(app.active_screen, ScreenId::HearthOverview);
    }

    #[test]
    fn app_resize_updates_breakpoint() {
        let mut app = App::new();
        app.apply_action(AppAction::Resize(50, 20));
        assert_eq!(app.state.layout, LayoutBreakpoint::Compact);
        app.apply_action(AppAction::Resize(130, 20));
        assert_eq!(app.state.layout, LayoutBreakpoint::Wide);
    }
}
