#![allow(dead_code, unreachable_pub)]

//! Command palette surface for terminal navigation.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::{
    palette::*,
    state::{AppAction, WindowId},
};

/// A command exposed through the terminal command palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// Human-readable command title.
    pub name: String,
    /// Short description shown in the palette list.
    pub description: String,
    /// Action emitted when the command is selected.
    pub action: AppAction,
    /// Optional keybinding hint rendered beside the command.
    pub keybinding: Option<String>,
}

/// Floating command palette state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPalette {
    /// Whether the palette is currently visible.
    pub visible: bool,
    /// Current search query.
    pub query: String,
    /// Registered commands.
    pub commands: Vec<Command>,
    /// Filtered indices into `commands`.
    pub filtered: Vec<usize>,
    /// Selected index inside `filtered`.
    pub selected: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        let commands = default_commands();
        let filtered = (0..commands.len()).collect();
        Self {
            visible: false,
            query: String::new(),
            commands,
            filtered,
            selected: 0,
        }
    }
}

impl CommandPalette {
    /// Rebuilds the filtered command list for the current query.
    pub fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.commands.len()).collect();
        } else {
            let mut scored: Vec<(usize, usize)> = self
                .commands
                .iter()
                .enumerate()
                .filter_map(|(index, command)| {
                    let score = lcs_score(&self.query, &command.name);
                    (score > 0).then_some((index, score))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            self.filtered = scored.into_iter().map(|(index, _score)| index).collect();
        }

        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    /// Processes a keypress while the palette is open.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Esc => {
                self.close();
                Some(AppAction::CloseCommandPalette)
            }
            KeyCode::Enter => {
                if let Some(&command_index) = self.filtered.get(self.selected) {
                    let action = self.commands[command_index].action.clone();
                    self.close();
                    Some(action)
                } else {
                    self.close();
                    Some(AppAction::CloseCommandPalette)
                }
            }
            KeyCode::Down | KeyCode::Tab => {
                if !self.filtered.is_empty() {
                    self.selected = (self.selected + 1) % self.filtered.len();
                }
                None
            }
            KeyCode::Up | KeyCode::BackTab => {
                if !self.filtered.is_empty() && self.selected > 0 {
                    self.selected -= 1;
                } else if !self.filtered.is_empty() {
                    self.selected = self.filtered.len() - 1;
                }
                None
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.update_filter();
                None
            }
            KeyCode::Char(c) if accepts_palette_input(key.modifiers) => {
                self.query.push(c);
                self.update_filter();
                None
            }
            _ => None,
        }
    }

    /// Renders the palette as a centered floating overlay.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        if !self.visible {
            return;
        }

        let width = 60u16.min(area.width.saturating_sub(4));
        let height = 20u16.min(area.height.saturating_sub(4));
        if width == 0 || height == 0 {
            return;
        }

        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        let palette_area = Rect {
            x,
            y,
            width,
            height,
        };

        frame.render_widget(Clear, palette_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_ACTIVE))
            .style(Style::default().bg(BG_RAISED))
            .title(Line::from(Span::styled(
                " Command Palette ",
                Style::default().fg(ROSE),
            )));
        let inner = block.inner(palette_area);
        frame.render_widget(block, palette_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(inner);

        let query_display = format!(" > {}_", self.query);
        frame.render_widget(
            Paragraph::new(query_display)
                .style(Style::default().fg(BONE).bg(BG_RAISED))
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(BORDER)),
                ),
            chunks[0],
        );

        let list_height = chunks[1].height as usize;
        let start = self.selected.saturating_sub(list_height.saturating_sub(1));
        let items: Vec<ListItem<'static>> = self
            .filtered
            .iter()
            .skip(start)
            .take(list_height)
            .enumerate()
            .map(|(offset, &command_index)| {
                let command = &self.commands[command_index];
                let is_selected = start + offset == self.selected;
                let name_style = if is_selected {
                    Style::default().fg(BONE).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(TEXT_PRIMARY)
                };
                let keybinding = command.keybinding.as_deref().unwrap_or("");
                let line = Line::from(vec![
                    Span::styled(format!(" {:<30}", command.name), name_style),
                    Span::styled(format!("{:>8} ", keybinding), Style::default().fg(TEXT_DIM)),
                ]);
                let item = ListItem::new(line);
                if is_selected {
                    item.style(Style::default().bg(BG_RAISED))
                } else {
                    item
                }
            })
            .collect();

        frame.render_widget(
            List::new(items).style(Style::default().bg(BG_RAISED)),
            chunks[1],
        );
    }

    fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.selected = 0;
        self.update_filter();
    }
}

/// Returns the default command registry exposed by the palette.
pub fn default_commands() -> Vec<Command> {
    vec![
        Command {
            name: "Quit".into(),
            description: "Exit bardo-terminal".into(),
            action: AppAction::Quit,
            keybinding: Some("q".into()),
        },
        Command {
            name: "Next Screen".into(),
            description: "Go to next tab".into(),
            action: AppAction::NextScreen,
            keybinding: Some("Tab".into()),
        },
        Command {
            name: "Previous Screen".into(),
            description: "Go to previous tab".into(),
            action: AppAction::PrevScreen,
            keybinding: Some("Shift+Tab".into()),
        },
        Command {
            name: "HEARTH".into(),
            description: "Go to HEARTH window".into(),
            action: AppAction::GotoWindow(WindowId::Hearth),
            keybinding: Some("F1".into()),
        },
        Command {
            name: "MIND".into(),
            description: "Go to MIND window".into(),
            action: AppAction::GotoWindow(WindowId::Mind),
            keybinding: Some("F2".into()),
        },
        Command {
            name: "SOMA".into(),
            description: "Go to SOMA window".into(),
            action: AppAction::GotoWindow(WindowId::Soma),
            keybinding: Some("F3".into()),
        },
        Command {
            name: "WORLD".into(),
            description: "Go to WORLD window".into(),
            action: AppAction::GotoWindow(WindowId::World),
            keybinding: Some("F4".into()),
        },
        Command {
            name: "FATE".into(),
            description: "Go to FATE window".into(),
            action: AppAction::GotoWindow(WindowId::Fate),
            keybinding: Some("F5".into()),
        },
        Command {
            name: "COMMAND".into(),
            description: "Go to COMMAND window".into(),
            action: AppAction::GotoWindow(WindowId::Command),
            keybinding: Some("F6".into()),
        },
        Command {
            name: "Toggle Help".into(),
            description: "Show/hide keybinding hints".into(),
            action: AppAction::ShowHelp,
            keybinding: Some("?".into()),
        },
        Command {
            name: "Scroll Top".into(),
            description: "Jump to top of current pane".into(),
            action: AppAction::ScrollTop,
            keybinding: Some("gg (vim)".into()),
        },
        Command {
            name: "Scroll Bottom".into(),
            description: "Jump to bottom of current pane".into(),
            action: AppAction::ScrollBottom,
            keybinding: Some("G (vim)".into()),
        },
    ]
}

fn accepts_palette_input(modifiers: KeyModifiers) -> bool {
    modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT
}

fn lcs_score(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_lowercase().chars().collect();
    let b: Vec<char> = b.to_lowercase().chars().collect();
    let m = a.len();
    let n = b.len();
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            curr[j] = if a[i - 1] == b[j - 1] {
                prev[j - 1] + 1
            } else {
                prev[j].max(curr[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }

    prev[n]
}

#[cfg(test)]
mod tests {
    use super::{CommandPalette, default_commands};
    use crate::state::{AppAction, WindowId};

    #[test]
    fn default_commands_include_plan_entries() {
        let commands = default_commands();

        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::Quit)
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::NextScreen)
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::PrevScreen)
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::GotoWindow(WindowId::Hearth))
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::GotoWindow(WindowId::Mind))
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::GotoWindow(WindowId::Soma))
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::GotoWindow(WindowId::World))
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::GotoWindow(WindowId::Fate))
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::GotoWindow(WindowId::Command))
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::ShowHelp)
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::ScrollTop)
        );
        assert!(
            commands
                .iter()
                .any(|command| command.action == AppAction::ScrollBottom)
        );
    }

    #[test]
    fn palette_default_state_starts_hidden_with_all_commands_filtered() {
        let palette = CommandPalette::default();

        assert!(!palette.visible);
        assert!(palette.query.is_empty());
        assert_eq!(
            palette.filtered,
            (0..palette.commands.len()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn command_palette_empty_query_shows_all() {
        let mut palette = CommandPalette {
            visible: true,
            query: String::new(),
            commands: default_commands(),
            filtered: vec![],
            selected: 0,
        };

        palette.update_filter();

        assert_eq!(palette.filtered.len(), palette.commands.len());
        assert_eq!(
            palette.filtered,
            (0..palette.commands.len()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn command_palette_fuzzy_filter() {
        let mut palette = CommandPalette {
            visible: true,
            query: "qui".into(),
            commands: default_commands(),
            filtered: vec![],
            selected: 0,
        };

        palette.update_filter();

        assert!(!palette.filtered.is_empty());
        assert_eq!(palette.commands[palette.filtered[0]].name, "Quit");
    }
}
