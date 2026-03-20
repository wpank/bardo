#![allow(dead_code, unreachable_pub)]

//! Modal stack, event routing, and overlay rendering for terminal dialogs.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    palette::{BG_RAISED, BONE, BORDER_ACTIVE, ROSE, TEXT_DIM, TEXT_PRIMARY},
    state::AppAction,
};

/// Modal variants rendered above the active screen.
pub enum Modal {
    /// Confirmation prompt with explicit confirm and optional cancel actions.
    Confirm {
        /// Dialog title.
        title: String,
        /// Dialog body text.
        message: String,
        /// Action emitted when the user confirms.
        on_confirm: AppAction,
        /// Action emitted when the user cancels, if any.
        on_cancel: Option<AppAction>,
    },
    /// Text input prompt that converts the submitted buffer into an action.
    Input {
        /// Dialog title.
        title: String,
        /// Placeholder shown while the buffer is empty.
        placeholder: String,
        /// Mutable input buffer.
        buffer: String,
        /// Callback that turns the submitted buffer into an action.
        on_submit: Box<dyn Fn(String) -> AppAction + Send>,
    },
    /// Informational alert that closes on acknowledgment.
    Alert {
        /// Dialog title.
        title: String,
        /// Dialog body text.
        message: String,
    },
}

impl Modal {
    /// Renders the modal centered inside the provided area.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        match self {
            Self::Alert { title, message } => {
                render_dialog(frame, area, title, message, &["[Enter/Esc/Space] OK"]);
            }
            Self::Confirm { title, message, .. } => {
                render_dialog(
                    frame,
                    area,
                    title,
                    message,
                    &["[y/Enter] Confirm", "[n/Esc] Cancel"],
                );
            }
            Self::Input {
                title,
                placeholder,
                buffer,
                ..
            } => {
                let display = if buffer.is_empty() {
                    format!("{placeholder}_")
                } else {
                    format!("{buffer}_")
                };
                render_dialog(
                    frame,
                    area,
                    title,
                    &display,
                    &["[Enter] Submit", "[Esc] Cancel"],
                );
            }
        }
    }
}

/// Stack-based modal controller. The last modal in the stack is active.
#[derive(Default)]
pub struct ModalManager {
    /// Modal stack where the last element is the active modal.
    pub stack: Vec<Modal>,
}

impl ModalManager {
    /// Creates an empty modal stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a modal onto the active stack.
    pub fn push(&mut self, modal: Modal) {
        self.stack.push(modal);
    }

    /// Pops the active modal from the stack.
    pub fn pop(&mut self) -> Option<Modal> {
        self.stack.pop()
    }

    /// Returns true when a modal is active.
    pub fn has_modal(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Routes a keypress to the top-most modal.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        if matches!(self.stack.last(), Some(Modal::Confirm { .. })) {
            return self.handle_confirm_key(key);
        }

        if matches!(self.stack.last(), Some(Modal::Input { .. })) {
            return self.handle_input_key(key);
        }

        if matches!(self.stack.last(), Some(Modal::Alert { .. })) {
            return self.handle_alert_key(key);
        }

        None
    }

    /// Renders the top-most modal when one is present.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        if let Some(top) = self.stack.last() {
            top.render(frame, area);
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        let Modal::Confirm {
            on_confirm,
            on_cancel,
            ..
        } = self.stack.last_mut()?
        else {
            return None;
        };

        match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                let action = on_confirm.clone();
                self.stack.pop();
                Some(action)
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                let action = on_cancel.as_ref().cloned().unwrap_or(AppAction::CloseModal);
                self.stack.pop();
                Some(action)
            }
            _ => None,
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Enter => {
                let modal = self.stack.pop()?;
                let Modal::Input {
                    buffer, on_submit, ..
                } = modal
                else {
                    return None;
                };

                Some(on_submit(buffer))
            }
            KeyCode::Esc => {
                self.stack.pop();
                Some(AppAction::CloseModal)
            }
            KeyCode::Backspace => {
                let Modal::Input { buffer, .. } = self.stack.last_mut()? else {
                    return None;
                };
                buffer.pop();
                None
            }
            KeyCode::Char(c) if accepts_text_input(key.modifiers) => {
                let Modal::Input { buffer, .. } = self.stack.last_mut()? else {
                    return None;
                };
                buffer.push(c);
                None
            }
            _ => None,
        }
    }

    fn handle_alert_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                self.stack.pop();
                Some(AppAction::CloseModal)
            }
            _ => None,
        }
    }
}

fn accepts_text_input(modifiers: KeyModifiers) -> bool {
    modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT
}

fn render_dialog(frame: &mut Frame<'_>, area: Rect, title: &str, body: &str, hints: &[&str]) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let width = 50u16.min(area.width.saturating_sub(8)).max(1);
    let height = 10u16.min(area.height.saturating_sub(4)).max(1);
    let dialog_area = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_ACTIVE))
        .style(Style::default().bg(BG_RAISED))
        .title(Line::from(vec![
            Span::styled("[", Style::default().fg(ROSE)),
            Span::styled(format!(" {title} "), Style::default().fg(BONE)),
            Span::styled("]", Style::default().fg(ROSE)),
        ]));
    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(body)
            .style(Style::default().fg(TEXT_PRIMARY))
            .wrap(Wrap { trim: true }),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            hints.join("  "),
            Style::default().fg(TEXT_DIM),
        )])),
        chunks[1],
    );
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{Modal, ModalManager};
    use crate::state::AppAction;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn modal_confirm_yes_fires_action() {
        for trigger in [KeyCode::Char('y'), KeyCode::Enter] {
            let mut manager = ModalManager::new();
            manager.push(Modal::Confirm {
                title: "Test".into(),
                message: "Are you sure?".into(),
                on_confirm: AppAction::Quit,
                on_cancel: None,
            });

            let action = manager.handle_key(key(trigger));

            assert_eq!(action, Some(AppAction::Quit));
            assert!(!manager.has_modal());
        }
    }

    #[test]
    fn modal_confirm_esc_fires_cancel() {
        let mut manager = ModalManager::new();
        manager.push(Modal::Confirm {
            title: "Test".into(),
            message: "Sure?".into(),
            on_confirm: AppAction::Quit,
            on_cancel: Some(AppAction::HideHelp),
        });

        let action = manager.handle_key(key(KeyCode::Esc));

        assert_eq!(action, Some(AppAction::HideHelp));
        assert!(!manager.has_modal());
    }

    #[test]
    fn modal_alert_closes_on_enter() {
        for trigger in [KeyCode::Enter, KeyCode::Esc, KeyCode::Char(' ')] {
            let mut manager = ModalManager::new();
            manager.push(Modal::Alert {
                title: "Info".into(),
                message: "Hello".into(),
            });

            let action = manager.handle_key(key(trigger));

            assert_eq!(action, Some(AppAction::CloseModal));
            assert!(!manager.has_modal());
        }
    }

    #[test]
    fn modal_input_submits_on_enter() {
        let mut manager = ModalManager::new();
        manager.push(Modal::Input {
            title: "Name".into(),
            placeholder: "enter value".into(),
            buffer: "spectre".into(),
            on_submit: Box::new(AppAction::VimCommand),
        });

        let action = manager.handle_key(key(KeyCode::Enter));

        assert_eq!(action, Some(AppAction::VimCommand("spectre".into())));
        assert!(!manager.has_modal());
    }

    #[test]
    fn modal_stack_push_pop() {
        let mut manager = ModalManager::new();
        assert!(!manager.has_modal());

        manager.push(Modal::Alert {
            title: "Bottom".into(),
            message: "First".into(),
        });
        manager.push(Modal::Alert {
            title: "Top".into(),
            message: "Second".into(),
        });

        assert!(manager.has_modal());
        assert!(matches!(
            manager.pop(),
            Some(Modal::Alert { title, .. }) if title == "Top"
        ));
        assert!(manager.has_modal());
        assert!(matches!(
            manager.pop(),
            Some(Modal::Alert { title, .. }) if title == "Bottom"
        ));
        assert!(!manager.has_modal());
    }

    #[test]
    fn modal_confirm_esc_close_modal_when_no_cancel_action() {
        let mut manager = ModalManager::new();
        manager.push(Modal::Confirm {
            title: "Test".into(),
            message: "Sure?".into(),
            on_confirm: AppAction::Quit,
            on_cancel: None,
        });

        let action = manager.handle_key(key(KeyCode::Esc));

        assert_eq!(action, Some(AppAction::CloseModal));
        assert!(!manager.has_modal());
    }

    #[test]
    fn modal_confirm_char_n_cancels() {
        let mut manager = ModalManager::new();
        manager.push(Modal::Confirm {
            title: "Test".into(),
            message: "Sure?".into(),
            on_confirm: AppAction::Quit,
            on_cancel: Some(AppAction::HideHelp),
        });

        let action = manager.handle_key(key(KeyCode::Char('n')));

        assert_eq!(action, Some(AppAction::HideHelp));
        assert!(!manager.has_modal());
    }

    #[test]
    fn modal_input_esc_pops_and_close_modal() {
        let mut manager = ModalManager::new();
        manager.push(Modal::Input {
            title: "Name".into(),
            placeholder: "x".into(),
            buffer: "abc".into(),
            on_submit: Box::new(|_| AppAction::Quit),
        });

        let action = manager.handle_key(key(KeyCode::Esc));

        assert_eq!(action, Some(AppAction::CloseModal));
        assert!(!manager.has_modal());
    }

    #[test]
    fn modal_input_backspace_and_chars() {
        let mut manager = ModalManager::new();
        manager.push(Modal::Input {
            title: "Name".into(),
            placeholder: "x".into(),
            buffer: "ab".into(),
            on_submit: Box::new(|_| AppAction::Quit),
        });

        assert!(manager.handle_key(key(KeyCode::Backspace)).is_none());
        let Modal::Input { buffer, .. } = manager.stack.last().expect("modal") else {
            panic!("expected Input");
        };
        assert_eq!(buffer, "a");

        assert!(
            manager
                .handle_key(KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::SHIFT))
                .is_none()
        );
        let Modal::Input { buffer, .. } = manager.stack.last().expect("modal") else {
            panic!("expected Input");
        };
        assert_eq!(buffer, "aZ");

        assert!(
            manager
                .handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL))
                .is_none()
        );
        let Modal::Input { buffer, .. } = manager.stack.last().expect("modal") else {
            panic!("expected Input");
        };
        assert_eq!(buffer, "aZ");
    }

    #[test]
    fn modal_key_routes_to_top_of_stack_only() {
        let mut manager = ModalManager::new();
        manager.push(Modal::Alert {
            title: "Bottom".into(),
            message: "First".into(),
        });
        manager.push(Modal::Confirm {
            title: "Top".into(),
            message: "Second".into(),
            on_confirm: AppAction::Quit,
            on_cancel: None,
        });

        let action = manager.handle_key(key(KeyCode::Enter));
        assert_eq!(action, Some(AppAction::Quit));
        assert!(manager.has_modal());
        assert!(matches!(
            manager.stack.last(),
            Some(Modal::Alert { title, .. }) if title == "Bottom"
        ));
    }
}
