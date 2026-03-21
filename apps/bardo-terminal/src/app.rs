//! Application state machine and render loop for the terminal scaffold.

#[allow(unused_imports)]
use golem_core::event::EventFabric;

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    layout::{LayoutBreakpoint, compute_layout},
    navigation::{CommandPalette, KeybindingMap, Modal, ModalManager, VimMode, VimModeState},
    palette::{
        BG_MID, BG_RAISED, BONE, BORDER_ACTIVE, BORDER_DREAM, ROSE, ROSE_BRIGHT, ROSE_DIM,
        TEXT_DIM, TEXT_PRIMARY, WARNING,
    },
    screen::{ScreenId, ScreenRegistry, StubScreen},
    screens::{HomeScreen, ProtocolViewsScreen},
    state::{AppAction, AppState, VimDirection, WindowId, format_duration},
    sys_stats::SysStats,
    widgets::{KeyBinding, KeyHelpOverlay},
};

const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

/// Application scaffold that owns terminal state and screen navigation.
pub(crate) struct App {
    state: AppState,
    screens: ScreenRegistry,
    active_screen: ScreenId,
    should_quit: bool,
    keybindings: KeybindingMap,
    command_palette: CommandPalette,
    modal_manager: ModalManager,
    vim_mode_state: VimModeState,
    show_help: bool,
    sys_stats: SysStats,
}

impl App {
    /// Creates the scaffold with all screens registered.
    pub(crate) fn new() -> Self {
        let mut screens = ScreenRegistry::new();

        for &screen_id in ScreenId::all() {
            match screen_id {
                ScreenId::HearthOverview => screens.register(Box::new(HomeScreen::new())),
                ScreenId::ProtocolViews => screens.register(Box::new(ProtocolViewsScreen::new())),
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
            keybindings: load_keybindings(),
            command_palette: CommandPalette::default(),
            modal_manager: ModalManager::new(),
            vim_mode_state: VimModeState::new(false),
            show_help: false,
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
        if key.kind != KeyEventKind::Press {
            return None;
        }

        let key = normalize_key_event(key);

        if self.vim_intercepts(&key) {
            let mode_before = self.vim_mode_state.mode;
            let sequence_len_before = self.vim_mode_state.sequence_buffer.len();
            if let Some(action) = self.vim_mode_state.process_key(key) {
                return Some(action);
            }

            if vim_consumed_key(&self.vim_mode_state, mode_before, sequence_len_before, key) {
                return None;
            }
        }

        if self.modal_manager.has_modal() {
            return normalize_modal_action(self.modal_manager.handle_key(key));
        }

        if self.command_palette.visible {
            return palette_action_for_key(&self.command_palette, key);
        }

        if let Some(action) = self
            .keybindings
            .resolve(key, self.active_screen)
            .filter(|action| self.keybinding_is_active(action))
        {
            return Some(action);
        }

        self.forward_key_to_active_screen(key)
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
            AppAction::GotoScreen(screen_id) => self.navigate_to(screen_id),
            AppAction::GotoWindow(window_id) => self.navigate_to(screen_for_window(window_id)),
            AppAction::OpenCommandPalette => {
                self.command_palette.visible = true;
                self.command_palette.query.clear();
                self.command_palette.selected = 0;
                self.command_palette.update_filter();
            }
            AppAction::CloseCommandPalette => self.close_command_palette(),
            AppAction::ExecuteCommand(filtered_index) => {
                let next_action = self
                    .command_palette
                    .filtered
                    .get(filtered_index)
                    .and_then(|command_index| self.command_palette.commands.get(*command_index))
                    .map(|command| command.action.clone());
                self.close_command_palette();
                if let Some(next_action) = next_action {
                    self.apply_action(next_action);
                }
            }
            AppAction::PaletteInput(ch) => {
                self.command_palette.query.push(ch);
                self.command_palette.update_filter();
            }
            AppAction::PaletteBackspace => {
                self.command_palette.query.pop();
                self.command_palette.update_filter();
            }
            AppAction::PaletteSelectNext => {
                if !self.command_palette.filtered.is_empty() {
                    self.command_palette.selected =
                        (self.command_palette.selected + 1) % self.command_palette.filtered.len();
                }
            }
            AppAction::PaletteSelectPrev => {
                if !self.command_palette.filtered.is_empty() {
                    self.command_palette.selected = if self.command_palette.selected == 0 {
                        self.command_palette.filtered.len() - 1
                    } else {
                        self.command_palette.selected - 1
                    };
                }
            }
            AppAction::ShowHelp => {
                self.show_help = !self.show_help;
            }
            AppAction::HideHelp => {
                self.show_help = false;
            }
            AppAction::CloseModal => self.close_transient_surface(),
            AppAction::ConfirmModal => self.apply_modal_key(KeyCode::Enter),
            AppAction::ModalInput(ch) => self.apply_modal_text_key(KeyCode::Char(ch)),
            AppAction::ModalBackspace => self.apply_modal_key(KeyCode::Backspace),
            AppAction::EnterVimMode => self.set_vim_mode(true),
            AppAction::ExitVimMode => self.set_vim_mode(false),
            AppAction::VimNavigate(direction) => self.apply_vim_navigation(direction),
            AppAction::VimCommand(command) => self.apply_vim_command(&command),
            AppAction::ScrollUp => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
            }
            AppAction::ScrollDown => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            }
            AppAction::ScrollTop => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE))
            }
            AppAction::ScrollBottom => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE))
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

    fn navigate_to(&mut self, next_screen: ScreenId) {
        if next_screen == self.active_screen {
            return;
        }

        if let Some(screen) = self.screens.get_mut(&self.active_screen) {
            screen.on_blur();
        }

        self.active_screen = next_screen;
        self.focus_active_screen();
    }

    fn close_command_palette(&mut self) {
        self.command_palette.visible = false;
        self.command_palette.query.clear();
        self.command_palette.selected = 0;
        self.command_palette.update_filter();
    }

    fn close_transient_surface(&mut self) {
        if self.modal_manager.has_modal() {
            self.apply_modal_key(KeyCode::Esc);
            return;
        }

        if self.command_palette.visible {
            self.close_command_palette();
            return;
        }

        if self.show_help {
            self.show_help = false;
            return;
        }

        if self.vim_mode_state.mode == VimMode::Command {
            self.vim_mode_state.command_buffer.clear();
            self.vim_mode_state.sequence_buffer.clear();
            self.vim_mode_state.mode = VimMode::Normal;
        }
    }

    fn apply_modal_key(&mut self, key_code: KeyCode) {
        let action = self
            .modal_manager
            .handle_key(KeyEvent::new(key_code, KeyModifiers::NONE));
        if let Some(action) = action.filter(|action| *action != AppAction::CloseModal) {
            self.apply_action(action);
        }
    }

    fn apply_modal_text_key(&mut self, key_code: KeyCode) {
        let action = self
            .modal_manager
            .handle_key(KeyEvent::new(key_code, KeyModifiers::NONE));
        if let Some(action) = action.filter(|action| *action != AppAction::CloseModal) {
            self.apply_action(action);
        }
    }

    fn set_vim_mode(&mut self, enabled: bool) {
        self.vim_mode_state.enabled = enabled;
        self.vim_mode_state.mode = VimMode::Normal;
        self.vim_mode_state.command_buffer.clear();
        self.vim_mode_state.sequence_buffer.clear();
        self.keybindings.vim_mode = enabled;
    }

    fn apply_vim_navigation(&mut self, direction: VimDirection) {
        match direction {
            VimDirection::Left => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE))
            }
            VimDirection::Right => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
            }
            VimDirection::Up => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
            }
            VimDirection::Down => {
                self.dispatch_screen_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
            }
        }
    }

    fn apply_vim_command(&mut self, command: &str) {
        match command.trim().to_ascii_lowercase().as_str() {
            "q" | "quit" => self.apply_action(AppAction::Quit),
            "help" => {
                self.show_help = true;
            }
            "palette" => self.apply_action(AppAction::OpenCommandPalette),
            "" => {}
            other => self.modal_manager.push(Modal::Alert {
                title: "Unknown Command".into(),
                message: format!(":{other} is not recognized"),
            }),
        }
    }

    fn keybinding_is_active(&self, action: &AppAction) -> bool {
        match action {
            AppAction::VimNavigate(_) | AppAction::ScrollBottom => {
                self.vim_mode_state.active() && self.vim_mode_state.mode == VimMode::Normal
            }
            _ => true,
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

            screen.render(frame, content_area, &self.state);
            self.render_status_bar(frame, footer_area);
            self.modal_manager.render(frame, size);
            self.command_palette.render(frame, size);
            if self.show_help {
                frame.render_widget(
                    &KeyHelpOverlay {
                        bindings: help_bindings(),
                        visible: true,
                    },
                    size,
                );
            }
        }
    }

    fn screen_index(&self, screen_id: ScreenId) -> Option<usize> {
        ScreenId::all()
            .iter()
            .position(|candidate| *candidate == screen_id)
    }

    fn vim_intercepts(&self, key: &KeyEvent) -> bool {
        self.vim_mode_state.enabled
            && match self.vim_mode_state.mode {
                VimMode::Normal | VimMode::Command => true,
                VimMode::Insert => key.code == KeyCode::Esc,
            }
    }

    fn dispatch_screen_key(&mut self, key: KeyEvent) {
        if let Some(action) = self.forward_key_to_active_screen(key) {
            self.apply_action(action);
        }
    }

    fn forward_key_to_active_screen(&mut self, key: KeyEvent) -> Option<AppAction> {
        self.screens
            .get_mut(&self.active_screen)
            .and_then(|screen| screen.handle_key(key))
    }

    fn render_status_bar(&self, frame: &mut Frame<'_>, area: Rect) {
        let screen_index = self.screen_index(self.active_screen).unwrap_or(0) + 1;
        let mut spans = vec![
            Span::styled(
                format!(" {} ", self.active_screen.window_name()),
                Style::default()
                    .fg(BONE)
                    .bg(BG_MID)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} / {}  ", screen_index, ScreenId::all().len()),
                Style::default().fg(TEXT_DIM).bg(BG_MID),
            ),
            Span::styled(
                format!(
                    "{}  {} panels  ",
                    self.state.connection_status.label(),
                    self.state.layout.panel_count(),
                ),
                Style::default().fg(ROSE_DIM).bg(BG_MID),
            ),
        ];

        if self.vim_mode_state.enabled {
            spans.push(Span::styled(
                "VIM ",
                Style::default().fg(TEXT_DIM).bg(BG_MID),
            ));
            spans.push(Span::styled(
                self.vim_mode_state.mode_indicator(),
                Style::default()
                    .fg(vim_indicator_color(self.vim_mode_state.mode))
                    .bg(BG_MID)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("  ", Style::default().bg(BG_MID)));
        }

        spans.push(Span::styled(
            "/ palette  ? help  Esc back",
            Style::default().fg(TEXT_PRIMARY).bg(BG_MID),
        ));

        frame.render_widget(
            Paragraph::new(Line::from(spans))
                .block(Block::default().style(Style::default().bg(BG_MID)))
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}

fn load_keybindings() -> KeybindingMap {
    bardo_config_path("keybindings.toml")
        .and_then(|path| KeybindingMap::load_from_toml(&path).ok())
        .unwrap_or_else(KeybindingMap::default_bindings)
}

fn bardo_config_path(file_name: &str) -> Option<PathBuf> {
    let mut path = resolve_home_dir(
        env::var_os("HOME"),
        env::var_os("USERPROFILE"),
        env::var_os("HOMEDRIVE"),
        env::var_os("HOMEPATH"),
    )?;
    path.push(".bardo");
    path.push(file_name);
    Some(path)
}

fn resolve_home_dir(
    home: Option<OsString>,
    userprofile: Option<OsString>,
    homedrive: Option<OsString>,
    homepath: Option<OsString>,
) -> Option<PathBuf> {
    if let Some(home) = home.filter(|value| !value.is_empty()) {
        return Some(Path::new(&home).to_path_buf());
    }

    if let Some(userprofile) = userprofile.filter(|value| !value.is_empty()) {
        return Some(Path::new(&userprofile).to_path_buf());
    }

    match (homedrive, homepath) {
        (Some(drive), Some(path)) if !drive.is_empty() && !path.is_empty() => Some(PathBuf::from(
            format!("{}{}", drive.to_string_lossy(), path.to_string_lossy()),
        )),
        _ => None,
    }
}

fn normalize_key_event(key: KeyEvent) -> KeyEvent {
    KeyEvent::new(key.code, key.modifiers)
}

fn normalize_modal_action(action: Option<AppAction>) -> Option<AppAction> {
    match action {
        Some(AppAction::CloseModal) => None,
        other => other,
    }
}

fn vim_consumed_key(
    vim_mode_state: &VimModeState,
    mode_before: VimMode,
    sequence_len_before: usize,
    key: KeyEvent,
) -> bool {
    match mode_before {
        VimMode::Insert => key.code == KeyCode::Esc,
        VimMode::Command => true,
        VimMode::Normal => {
            sequence_len_before > 0
                || !vim_mode_state.sequence_buffer.is_empty()
                || matches!(
                    key.code,
                    KeyCode::Char('i')
                        | KeyCode::Char(':')
                        | KeyCode::Char('h')
                        | KeyCode::Char('j')
                        | KeyCode::Char('k')
                        | KeyCode::Char('l')
                        | KeyCode::Char('g')
                        | KeyCode::Char('G')
                        | KeyCode::Esc
                )
        }
    }
}

fn palette_action_for_key(palette: &CommandPalette, key: KeyEvent) -> Option<AppAction> {
    match key.code {
        KeyCode::Esc => Some(AppAction::CloseCommandPalette),
        KeyCode::Enter => {
            if palette.filtered.get(palette.selected).is_some() {
                Some(AppAction::ExecuteCommand(palette.selected))
            } else {
                Some(AppAction::CloseCommandPalette)
            }
        }
        KeyCode::Down | KeyCode::Tab => Some(AppAction::PaletteSelectNext),
        KeyCode::Up | KeyCode::BackTab => Some(AppAction::PaletteSelectPrev),
        KeyCode::Backspace => Some(AppAction::PaletteBackspace),
        KeyCode::Char(ch)
            if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
        {
            Some(AppAction::PaletteInput(ch))
        }
        _ => None,
    }
}

fn vim_indicator_color(mode: VimMode) -> ratatui::style::Color {
    match mode {
        VimMode::Normal => BONE,
        VimMode::Insert => ROSE_BRIGHT,
        VimMode::Command => WARNING,
    }
}

fn help_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding {
            key: "q".into(),
            description: "quit".into(),
        },
        KeyBinding {
            key: "Tab / Shift+Tab".into(),
            description: "cycle screens".into(),
        },
        KeyBinding {
            key: "1-6 / F1-F6".into(),
            description: "jump windows".into(),
        },
        KeyBinding {
            key: "/".into(),
            description: "open command palette".into(),
        },
        KeyBinding {
            key: "?".into(),
            description: "toggle help".into(),
        },
        KeyBinding {
            key: "Esc".into(),
            description: "close modal/palette/help".into(),
        },
        KeyBinding {
            key: "i".into(),
            description: "enter vim insert".into(),
        },
        KeyBinding {
            key: ":".into(),
            description: "vim command mode".into(),
        },
        KeyBinding {
            key: "gg / G".into(),
            description: "top / bottom".into(),
        },
    ]
}

const fn screen_for_window(window_id: WindowId) -> ScreenId {
    match window_id {
        WindowId::Hearth => ScreenId::HearthOverview,
        WindowId::Mind => ScreenId::MindPipeline,
        WindowId::Soma => ScreenId::SomaPortfolio,
        WindowId::World => ScreenId::WorldSolaris,
        WindowId::Fate => ScreenId::FateMortality,
        WindowId::Command => ScreenId::CommandSteer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventState;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_text(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");

        terminal
            .draw(|frame| app.render(frame))
            .expect("render succeeds");

        let buffer = terminal.backend().buffer();
        let area = buffer.area();
        let mut text = String::new();

        for y in 0..area.height {
            for x in 0..area.width {
                text.push_str(buffer.get(x, y).symbol());
            }
            text.push('\n');
        }

        text
    }

    #[test]
    fn app_starts_on_the_home_screen() {
        let app = App::new();

        assert_eq!(app.active_screen, ScreenId::HearthOverview);
        assert_eq!(app.state.layout, LayoutBreakpoint::Standard);
        assert!(!app.should_quit);
        assert!(!app.command_palette.visible);
        assert!(!app.modal_manager.has_modal());
        assert!(!app.vim_mode_state.active());
        assert!(!app.show_help);
        assert!(
            app.keybindings
                .resolve(
                    KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                    app.active_screen
                )
                .is_some()
        );
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

    #[test]
    fn app_handle_key_owns_global_navigation_keys() {
        let mut app = App::new();

        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(AppAction::Quit)
        );
        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            Some(AppAction::NextScreen)
        );
        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(AppAction::PrevScreen)
        );
    }

    #[test]
    fn app_handle_key_ignores_non_press_events() {
        let mut app = App::new();

        let repeat =
            KeyEvent::new_with_kind(KeyCode::Char('q'), KeyModifiers::NONE, KeyEventKind::Repeat);
        let release = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        };

        assert_eq!(app.handle_key(repeat), None);
        assert_eq!(app.handle_key(release), None);
    }

    #[test]
    fn app_goto_window_maps_to_window_root_screen() {
        let mut app = App::new();

        app.apply_action(AppAction::GotoWindow(WindowId::Command));

        assert_eq!(app.active_screen, ScreenId::CommandSteer);
    }

    #[test]
    fn app_routes_palette_before_keybindings() {
        let mut app = App::new();
        app.command_palette.visible = true;

        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(AppAction::PaletteInput('q'))
        );
    }

    #[test]
    fn app_routes_modal_before_palette() {
        let mut app = App::new();
        app.command_palette.visible = true;
        app.modal_manager.push(Modal::Confirm {
            title: "Quit".into(),
            message: "Leave?".into(),
            on_confirm: AppAction::Quit,
            on_cancel: None,
        });

        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(AppAction::Quit)
        );
    }

    #[test]
    fn app_routes_vim_before_modal() {
        let mut app = App::new();
        app.set_vim_mode(true);
        app.modal_manager.push(Modal::Alert {
            title: "Info".into(),
            message: "Ignored".into(),
        });

        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
            Some(AppAction::VimNavigate(VimDirection::Down))
        );
    }

    #[test]
    fn close_modal_action_falls_back_to_palette_help_then_vim_command() {
        let mut app = App::new();
        app.command_palette.visible = true;
        app.show_help = true;
        app.set_vim_mode(true);
        app.vim_mode_state.mode = VimMode::Command;
        app.vim_mode_state.command_buffer = "help".into();

        app.apply_action(AppAction::CloseModal);
        assert!(!app.command_palette.visible);
        assert!(app.show_help);
        assert_eq!(app.vim_mode_state.mode, VimMode::Command);

        app.apply_action(AppAction::CloseModal);
        assert!(!app.show_help);
        assert_eq!(app.vim_mode_state.mode, VimMode::Command);

        app.apply_action(AppAction::CloseModal);
        assert_eq!(app.vim_mode_state.mode, VimMode::Normal);
        assert!(app.vim_mode_state.command_buffer.is_empty());
    }

    #[test]
    fn show_help_toggles_overlay_state() {
        let mut app = App::new();

        app.apply_action(AppAction::ShowHelp);
        assert!(app.show_help);
        app.apply_action(AppAction::ShowHelp);
        assert!(!app.show_help);
        app.apply_action(AppAction::HideHelp);
        assert!(!app.show_help);
    }

    #[test]
    fn execute_command_runs_selected_palette_action() {
        let mut app = App::new();
        app.command_palette.visible = true;

        app.apply_action(AppAction::ExecuteCommand(0));

        assert!(app.should_quit);
        assert!(!app.command_palette.visible);
        assert!(app.command_palette.query.is_empty());
    }

    #[test]
    fn vim_commands_dispatch_built_ins_and_alert_unknown_commands() {
        let mut app = App::new();

        app.apply_action(AppAction::VimCommand("palette".into()));
        assert!(app.command_palette.visible);

        app.apply_action(AppAction::VimCommand("help".into()));
        assert!(app.show_help);

        app.apply_action(AppAction::VimCommand("mystery".into()));
        assert!(app.modal_manager.has_modal());
        match app.modal_manager.stack.last() {
            Some(Modal::Alert { title, message }) => {
                assert_eq!(title, "Unknown Command");
                assert!(message.contains(":mystery"));
            }
            _ => panic!("expected alert modal"),
        }
    }

    #[test]
    fn render_shows_navigation_surfaces_and_vim_status() {
        let mut app = App::new();
        app.set_vim_mode(true);
        app.vim_mode_state.mode = VimMode::Command;
        app.command_palette.visible = true;
        app.command_palette.query = "qui".into();
        app.command_palette.update_filter();
        app.show_help = true;

        let text = render_text(&app, 120, 36);

        assert!(text.contains("Command Palette"));
        assert!(text.contains("KEY BINDINGS"));
        assert!(text.contains("VIM COMMAND"));
        assert!(text.contains("/ palette  ? help  Esc back"));
    }

    #[test]
    fn render_shows_modal_overlay() {
        let mut app = App::new();
        app.modal_manager.push(Modal::Alert {
            title: "Info".into(),
            message: "Overlay active".into(),
        });

        let text = render_text(&app, 100, 30);

        assert!(text.contains("Info"));
        assert!(text.contains("Overlay active"));
    }

    #[test]
    fn resolve_home_dir_supports_platform_fallbacks() {
        assert_eq!(
            resolve_home_dir(
                Some(OsString::from("/tmp/home")),
                Some(OsString::from("C:\\Users\\will")),
                None,
                None,
            ),
            Some(PathBuf::from("/tmp/home"))
        );
        assert_eq!(
            resolve_home_dir(None, Some(OsString::from("C:\\Users\\will")), None, None,),
            Some(PathBuf::from("C:\\Users\\will"))
        );
        assert_eq!(
            resolve_home_dir(
                None,
                None,
                Some(OsString::from("C:")),
                Some(OsString::from("\\Users\\will")),
            ),
            Some(PathBuf::from("C:\\Users\\will"))
        );
    }

    #[test]
    fn vim_indicator_uses_expected_mode_colors() {
        assert_eq!(vim_indicator_color(VimMode::Normal), BONE);
        assert_eq!(vim_indicator_color(VimMode::Insert), ROSE_BRIGHT);
        assert_eq!(vim_indicator_color(VimMode::Command), WARNING);
    }
}
