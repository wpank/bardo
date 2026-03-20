#![allow(dead_code, unreachable_pub)]

//! Vim-mode navigation surface for terminal navigation.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::{AppAction, VimDirection};

/// Active vim-mode input mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    /// Normal navigation mode.
    Normal,
    /// Insert/text entry mode.
    Insert,
    /// Colon-command mode.
    Command,
}

/// Vim-mode state tracked by the terminal app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VimModeState {
    /// Current vim input mode.
    pub mode: VimMode,
    /// Buffer for `:` commands.
    pub command_buffer: String,
    /// Buffer for multi-key sequences like `gg`.
    pub sequence_buffer: Vec<char>,
    /// Whether vim-mode handling is enabled.
    pub enabled: bool,
}

impl VimModeState {
    /// Creates a new vim-mode state.
    pub fn new(enabled: bool) -> Self {
        Self {
            mode: VimMode::Normal,
            command_buffer: String::new(),
            sequence_buffer: Vec::new(),
            enabled,
        }
    }

    /// Returns true when vim handling is enabled.
    ///
    /// The plan prose mentions "pass-through" in Insert mode, but the active
    /// contract used by the app is the `enabled` flag. Insert-mode pass-through
    /// is handled by `process_key()` returning `None` for non-Esc keys.
    pub fn active(&self) -> bool {
        self.enabled
    }

    /// Returns the status-line label for the current mode.
    pub fn mode_indicator(&self) -> &str {
        match self.mode {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Command => "COMMAND",
        }
    }

    /// Processes a keypress through the vim state machine.
    pub fn process_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        if !self.enabled {
            return None;
        }

        match self.mode {
            VimMode::Normal => self.process_normal(key),
            VimMode::Insert => self.process_insert(key),
            VimMode::Command => self.process_command(key),
        }
    }

    fn process_normal(&mut self, key: KeyEvent) -> Option<AppAction> {
        if !self.sequence_buffer.is_empty() {
            match (self.sequence_buffer.as_slice(), key.code) {
                (['g'], KeyCode::Char('g')) => {
                    self.sequence_buffer.clear();
                    return Some(AppAction::ScrollTop);
                }
                _ => {
                    self.sequence_buffer.clear();
                }
            }
        }

        match key.code {
            KeyCode::Char('i') => {
                self.mode = VimMode::Insert;
                None
            }
            KeyCode::Char(':') => {
                self.mode = VimMode::Command;
                self.command_buffer.clear();
                None
            }
            KeyCode::Char('h') => Some(AppAction::VimNavigate(VimDirection::Left)),
            KeyCode::Char('j') => Some(AppAction::VimNavigate(VimDirection::Down)),
            KeyCode::Char('k') => Some(AppAction::VimNavigate(VimDirection::Up)),
            KeyCode::Char('l') => Some(AppAction::VimNavigate(VimDirection::Right)),
            KeyCode::Char('g') => {
                self.sequence_buffer.push('g');
                None
            }
            KeyCode::Char('G') => Some(AppAction::ScrollBottom),
            KeyCode::Esc => {
                self.sequence_buffer.clear();
                None
            }
            _ => {
                self.sequence_buffer.clear();
                None
            }
        }
    }

    fn process_insert(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Esc => {
                self.mode = VimMode::Normal;
                None
            }
            _ => None,
        }
    }

    fn process_command(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Enter => {
                let command = std::mem::take(&mut self.command_buffer);
                self.mode = VimMode::Normal;
                Some(AppAction::VimCommand(command))
            }
            KeyCode::Esc => {
                self.command_buffer.clear();
                self.mode = VimMode::Normal;
                None
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                None
            }
            KeyCode::Char(c)
                if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.command_buffer.push(c);
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};

    use super::{VimMode, VimModeState};
    use crate::state::{AppAction, VimDirection};

    fn key(code: KeyCode) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn new_vim_state_starts_in_normal_mode() {
        let state = VimModeState::new(true);

        assert!(state.active());
        assert_eq!(state.mode, VimMode::Normal);
        assert!(state.command_buffer.is_empty());
        assert!(state.sequence_buffer.is_empty());
    }

    #[test]
    fn mode_indicator_tracks_current_mode() {
        let mut state = VimModeState::new(false);

        assert_eq!(state.mode_indicator(), "NORMAL");
        state.mode = VimMode::Insert;
        assert_eq!(state.mode_indicator(), "INSERT");
        state.mode = VimMode::Command;
        assert_eq!(state.mode_indicator(), "COMMAND");
    }

    #[test]
    fn active_tracks_enabled_flag() {
        let mut disabled = VimModeState::new(false);
        let mut enabled = VimModeState::new(true);

        disabled.mode = VimMode::Insert;
        enabled.mode = VimMode::Command;

        assert!(!disabled.active());
        assert!(enabled.active());
    }

    #[test]
    fn vim_mode_hjkl_navigate() {
        let mut vim = VimModeState::new(true);

        assert_eq!(
            vim.process_key(key(KeyCode::Char('h'))),
            Some(AppAction::VimNavigate(VimDirection::Left))
        );
        assert_eq!(
            vim.process_key(key(KeyCode::Char('j'))),
            Some(AppAction::VimNavigate(VimDirection::Down))
        );
        assert_eq!(
            vim.process_key(key(KeyCode::Char('k'))),
            Some(AppAction::VimNavigate(VimDirection::Up))
        );
        assert_eq!(
            vim.process_key(key(KeyCode::Char('l'))),
            Some(AppAction::VimNavigate(VimDirection::Right))
        );
    }

    #[test]
    fn vim_mode_gg_sequence() {
        let mut vim = VimModeState::new(true);

        assert_eq!(vim.process_key(key(KeyCode::Char('g'))), None);
        assert_eq!(vim.sequence_buffer, vec!['g']);
        assert_eq!(
            vim.process_key(key(KeyCode::Char('g'))),
            Some(AppAction::ScrollTop)
        );
        assert!(vim.sequence_buffer.is_empty());
    }

    #[test]
    fn vim_mode_colon_command() {
        let mut vim = VimModeState::new(true);

        assert_eq!(vim.process_key(key(KeyCode::Char(':'))), None);
        assert_eq!(vim.mode, VimMode::Command);
        assert_eq!(vim.process_key(key(KeyCode::Char('q'))), None);
        assert_eq!(vim.command_buffer, "q");
        assert_eq!(
            vim.process_key(key(KeyCode::Enter)),
            Some(AppAction::VimCommand("q".to_string()))
        );
        assert_eq!(vim.mode, VimMode::Normal);
        assert!(vim.command_buffer.is_empty());
    }

    #[test]
    fn vim_insert_esc_returns_normal() {
        let mut vim = VimModeState::new(true);

        assert_eq!(vim.process_key(key(KeyCode::Char('i'))), None);
        assert_eq!(vim.mode, VimMode::Insert);
        assert_eq!(vim.process_key(key(KeyCode::Esc)), None);
        assert_eq!(vim.mode, VimMode::Normal);
    }

    #[test]
    fn vim_mode_sequence_clears_on_mismatch() {
        let mut vim = VimModeState::new(true);

        assert_eq!(vim.process_key(key(KeyCode::Char('g'))), None);
        assert_eq!(vim.process_key(key(KeyCode::Char('x'))), None);
        assert!(vim.sequence_buffer.is_empty());
        assert_eq!(vim.mode, VimMode::Normal);
    }

    #[test]
    fn disabled_vim_mode_does_not_consume_keys() {
        let mut vim = VimModeState::new(false);

        assert_eq!(vim.process_key(key(KeyCode::Char('j'))), None);
        assert_eq!(vim.mode, VimMode::Normal);
        assert!(vim.command_buffer.is_empty());
        assert!(vim.sequence_buffer.is_empty());
    }
}
