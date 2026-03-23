#![allow(dead_code, unreachable_pub)]

//! Keybinding map and configuration loading surface for terminal navigation.

use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;
use tracing::warn;

use crate::{
    screen::ScreenId,
    state::{AppAction, VimDirection, WindowId},
};

/// Global and per-screen keybinding tables.
#[derive(Debug, Clone)]
pub struct KeybindingMap {
    /// Actions that fire regardless of the active screen.
    pub global: HashMap<KeyEvent, AppAction>,
    /// Per-screen overrides checked before the global bindings.
    pub per_screen: HashMap<ScreenId, HashMap<KeyEvent, AppAction>>,
    /// When true, vim-specific bindings are considered enabled.
    pub vim_mode: bool,
}

impl Default for KeybindingMap {
    fn default() -> Self {
        Self::default_bindings()
    }
}

impl KeybindingMap {
    /// Builds the default keybinding table described by the navigation plan.
    pub fn default_bindings() -> Self {
        let mut global = HashMap::new();

        bind(
            &mut global,
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            AppAction::Quit,
        );
        bind(
            &mut global,
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
            AppAction::Quit,
        );
        bind(
            &mut global,
            KeyCode::Tab,
            KeyModifiers::NONE,
            AppAction::NextScreen,
        );
        bind(
            &mut global,
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
            AppAction::PrevScreen,
        );
        bind(
            &mut global,
            KeyCode::Char('1'),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Hearth),
        );
        bind(
            &mut global,
            KeyCode::Char('2'),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Mind),
        );
        bind(
            &mut global,
            KeyCode::Char('3'),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Soma),
        );
        bind(
            &mut global,
            KeyCode::Char('4'),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::World),
        );
        bind(
            &mut global,
            KeyCode::Char('5'),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Fate),
        );
        bind(
            &mut global,
            KeyCode::Char('6'),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Command),
        );
        bind(
            &mut global,
            KeyCode::Char('7'),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Protocol),
        );
        bind(
            &mut global,
            KeyCode::Char('h'),
            KeyModifiers::NONE,
            AppAction::GotoScreen(ScreenId::HearthOverview),
        );
        bind(
            &mut global,
            KeyCode::Char('b'),
            KeyModifiers::NONE,
            AppAction::GotoScreen(ScreenId::MindPipeline),
        );
        bind(
            &mut global,
            KeyCode::Char('m'),
            KeyModifiers::NONE,
            AppAction::GotoScreen(ScreenId::MindGrimoire),
        );
        bind(
            &mut global,
            KeyCode::Char('f'),
            KeyModifiers::NONE,
            AppAction::GotoScreen(ScreenId::SomaPortfolio),
        );
        bind(
            &mut global,
            KeyCode::Char('v'),
            KeyModifiers::NONE,
            AppAction::GotoScreen(ScreenId::SomaSanctum),
        );
        bind(
            &mut global,
            KeyCode::Char('w'),
            KeyModifiers::NONE,
            AppAction::GotoScreen(ScreenId::WorldSolaris),
        );
        bind(
            &mut global,
            KeyCode::Char('x'),
            KeyModifiers::NONE,
            AppAction::GotoScreen(ScreenId::WorldBazaar),
        );
        bind(
            &mut global,
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            AppAction::ShowHelp,
        );
        bind(
            &mut global,
            KeyCode::Char('/'),
            KeyModifiers::NONE,
            AppAction::OpenCommandPalette,
        );
        bind(
            &mut global,
            KeyCode::Esc,
            KeyModifiers::NONE,
            AppAction::CloseModal,
        );
        bind(
            &mut global,
            KeyCode::F(1),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Hearth),
        );
        bind(
            &mut global,
            KeyCode::F(2),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Mind),
        );
        bind(
            &mut global,
            KeyCode::F(3),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Soma),
        );
        bind(
            &mut global,
            KeyCode::F(4),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::World),
        );
        bind(
            &mut global,
            KeyCode::F(5),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Fate),
        );
        bind(
            &mut global,
            KeyCode::F(6),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Command),
        );
        bind(
            &mut global,
            KeyCode::F(7),
            KeyModifiers::NONE,
            AppAction::GotoWindow(WindowId::Protocol),
        );
        bind(
            &mut global,
            KeyCode::Up,
            KeyModifiers::NONE,
            AppAction::ScrollUp,
        );
        bind(
            &mut global,
            KeyCode::Down,
            KeyModifiers::NONE,
            AppAction::ScrollDown,
        );

        Self {
            global,
            per_screen: HashMap::new(),
            // `hjkl`, `gg`, and `G` are handled by `navigation::vim`; this flag
            // enables that layer without leaking vim-only motions into global bindings.
            vim_mode: true,
        }
    }

    /// Loads keybinding overrides from TOML and merges them over the defaults.
    ///
    /// Missing files return an error so the caller can fall back to defaults.
    pub fn load_from_toml(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read keybindings from {}", path.display()))?;
        let config: KeybindingConfig = toml::from_str(&contents)
            .with_context(|| format!("failed to parse keybindings TOML from {}", path.display()))?;

        let mut bindings = Self::default_bindings();
        merge_table(&mut bindings.global, config.global, "global");

        for (screen_name, overrides) in config.screen {
            let Some(screen_id) = parse_screen_id(&screen_name) else {
                warn!(screen = %screen_name, "skipping unknown keybinding screen section");
                continue;
            };

            let table = bindings.per_screen.entry(screen_id).or_default();
            merge_table(table, overrides, &format!("screen.{screen_name}"));
        }

        Ok(bindings)
    }

    /// Resolves a key event by checking the per-screen map before the global table.
    pub fn resolve(&self, key: KeyEvent, screen: ScreenId) -> Option<AppAction> {
        self.per_screen
            .get(&screen)
            .and_then(|bindings| bindings.get(&key))
            .or_else(|| {
                if global_binding_suppressed_for_screen(screen, &key) {
                    return None;
                }
                self.global.get(&key)
            })
            .cloned()
    }
}

/// Returns true when a global shortcut must not fire so the active screen can handle the key.
///
/// [`ScreenId::ProtocolViews`] uses vim-style `h` for moving focus; the default global map also binds
/// plain `h` to jump HEARTH, which would otherwise win in [`KeybindingMap::resolve`] and never reach
/// [`crate::screens::protocol_views::ProtocolViewsScreen`].
fn global_binding_suppressed_for_screen(screen: ScreenId, key: &KeyEvent) -> bool {
    screen == ScreenId::ProtocolViews
        && key.modifiers == KeyModifiers::NONE
        && matches!(key.code, KeyCode::Char('h'))
}

#[derive(Debug, Default, Deserialize)]
struct KeybindingConfig {
    #[serde(default)]
    global: HashMap<String, String>,
    #[serde(default)]
    screen: HashMap<String, HashMap<String, String>>,
}

fn bind(
    map: &mut HashMap<KeyEvent, AppAction>,
    code: KeyCode,
    modifiers: KeyModifiers,
    action: AppAction,
) {
    map.insert(KeyEvent::new(code, modifiers), action);
}

fn merge_table(
    map: &mut HashMap<KeyEvent, AppAction>,
    entries: HashMap<String, String>,
    scope: &str,
) {
    for (key_str, action_str) in entries {
        let Some(key_event) = parse_key_str(&key_str) else {
            warn!(scope = scope, key = %key_str, "skipping invalid keybinding key");
            continue;
        };
        let Some(action) = parse_action_str(&action_str) else {
            warn!(scope = scope, action = %action_str, "skipping unknown keybinding action");
            continue;
        };

        map.insert(key_event, action);
    }
}

/// Parses strings like `ctrl+c`, `tab`, and `f1` into [`KeyEvent`] values.
pub(crate) fn parse_key_str(input: &str) -> Option<KeyEvent> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut modifiers = KeyModifiers::NONE;
    let mut key_token = None;

    for token in trimmed.split('+') {
        let normalized = token.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "" => return None,
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "alt" => modifiers |= KeyModifiers::ALT,
            other => {
                if key_token.replace(other.to_owned()).is_some() {
                    return None;
                }
            }
        }
    }

    let key_token = key_token?;
    let mut code = match key_token.as_str() {
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "esc" | "escape" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "backspace" => KeyCode::Backspace,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        other if other.starts_with('f') => {
            let number = other[1..].parse::<u8>().ok()?;
            if (1..=12).contains(&number) {
                KeyCode::F(number)
            } else {
                return None;
            }
        }
        other => {
            let mut chars = other.chars();
            let ch = chars.next()?.to_ascii_lowercase();
            if chars.next().is_some() {
                return None;
            }
            KeyCode::Char(ch)
        }
    };

    if code == KeyCode::Tab && modifiers.contains(KeyModifiers::SHIFT) {
        code = KeyCode::BackTab;
    }

    Some(KeyEvent::new(code, modifiers))
}

fn parse_action_str(input: &str) -> Option<AppAction> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((kind, value)) = trimmed.split_once(':') {
        let value = value.trim();
        return match kind.trim().to_ascii_lowercase().as_str() {
            "gotoscreen" => parse_screen_id(value).map(AppAction::GotoScreen),
            "gotowindow" => parse_window_id(value).map(AppAction::GotoWindow),
            "executecommand" => value.parse::<usize>().ok().map(AppAction::ExecuteCommand),
            "paletteinput" => parse_single_char(value).map(AppAction::PaletteInput),
            "modalinput" => parse_single_char(value).map(AppAction::ModalInput),
            "vimnavigate" => parse_vim_direction(value).map(AppAction::VimNavigate),
            "vimcommand" if !value.is_empty() => Some(AppAction::VimCommand(value.to_owned())),
            _ => None,
        };
    }

    match trimmed.to_ascii_lowercase().as_str() {
        "quit" => Some(AppAction::Quit),
        "nextscreen" => Some(AppAction::NextScreen),
        "prevscreen" => Some(AppAction::PrevScreen),
        "opencommandpalette" => Some(AppAction::OpenCommandPalette),
        "closecommandpalette" => Some(AppAction::CloseCommandPalette),
        "palettebackspace" => Some(AppAction::PaletteBackspace),
        "paletteselectnext" => Some(AppAction::PaletteSelectNext),
        "paletteselectprev" => Some(AppAction::PaletteSelectPrev),
        "showhelp" => Some(AppAction::ShowHelp),
        "hidehelp" => Some(AppAction::HideHelp),
        "closemodal" => Some(AppAction::CloseModal),
        "confirmmodal" => Some(AppAction::ConfirmModal),
        "modalbackspace" => Some(AppAction::ModalBackspace),
        "entervimmode" => Some(AppAction::EnterVimMode),
        "exitvimmode" => Some(AppAction::ExitVimMode),
        "scrollup" => Some(AppAction::ScrollUp),
        "scrolldown" => Some(AppAction::ScrollDown),
        "scrolltop" => Some(AppAction::ScrollTop),
        "scrollbottom" => Some(AppAction::ScrollBottom),
        _ => None,
    }
}

fn parse_screen_id(input: &str) -> Option<ScreenId> {
    let trimmed = input.trim();

    ScreenId::all()
        .iter()
        .copied()
        .find(|screen_id| format!("{screen_id:?}").eq_ignore_ascii_case(trimmed))
}

fn parse_window_id(input: &str) -> Option<WindowId> {
    match input.trim().to_ascii_lowercase().as_str() {
        "hearth" => Some(WindowId::Hearth),
        "mind" => Some(WindowId::Mind),
        "soma" => Some(WindowId::Soma),
        "world" => Some(WindowId::World),
        "fate" => Some(WindowId::Fate),
        "command" => Some(WindowId::Command),
        "protocol" => Some(WindowId::Protocol),
        _ => None,
    }
}

fn parse_vim_direction(input: &str) -> Option<VimDirection> {
    match input.trim().to_ascii_lowercase().as_str() {
        "up" => Some(VimDirection::Up),
        "down" => Some(VimDirection::Down),
        "left" => Some(VimDirection::Left),
        "right" => Some(VimDirection::Right),
        _ => None,
    }
}

fn parse_single_char(input: &str) -> Option<char> {
    let normalized = input.trim().to_ascii_lowercase();
    let mut chars = normalized.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }

    Some(ch)
}

#[cfg(test)]
mod tests {
    use super::{KeybindingMap, parse_key_str};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::{
        screen::ScreenId,
        state::{AppAction, WindowId},
    };

    #[test]
    fn keybinding_default_quit() {
        let bindings = KeybindingMap::default_bindings();

        assert!(bindings.vim_mode);
        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                ScreenId::HearthOverview
            ),
            Some(AppAction::Quit)
        );
    }

    #[test]
    fn keybinding_goto_window() {
        let bindings = KeybindingMap::default_bindings();

        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
                ScreenId::HearthOverview
            ),
            Some(AppAction::GotoWindow(WindowId::Hearth))
        );
        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::F(7), KeyModifiers::NONE),
                ScreenId::HearthOverview
            ),
            Some(AppAction::GotoWindow(WindowId::Protocol))
        );
        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE),
                ScreenId::MindPipeline
            ),
            Some(AppAction::GotoWindow(WindowId::Protocol))
        );
    }

    #[test]
    fn keybinding_protocol_views_skips_global_h_jump_to_hearth() {
        let bindings = KeybindingMap::default_bindings();
        let h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        assert_eq!(bindings.resolve(h, ScreenId::ProtocolViews), None);
        assert_eq!(
            bindings.resolve(h, ScreenId::MindPipeline),
            Some(AppAction::GotoScreen(ScreenId::HearthOverview))
        );
    }

    #[test]
    fn keybinding_load_from_toml_merges_per_screen_override() {
        let path = write_temp_keybindings(
            "override",
            r#"
[global]
"f2" = "GotoWindow:Mind"
"tab" = "NoSuchAction"

[screen.HearthOverview]
"q" = "ShowHelp"
"r" = "ScrollTop"
"#,
        );

        let bindings =
            KeybindingMap::load_from_toml(&path).expect("temporary TOML file should load");

        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                ScreenId::HearthOverview
            ),
            Some(AppAction::ShowHelp)
        );
        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                ScreenId::MindPipeline
            ),
            Some(AppAction::Quit)
        );
        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
                ScreenId::HearthOverview
            ),
            Some(AppAction::ScrollTop)
        );
        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
                ScreenId::MindPipeline
            ),
            Some(AppAction::GotoWindow(WindowId::Mind))
        );
        assert_eq!(
            bindings.resolve(
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                ScreenId::MindPipeline
            ),
            Some(AppAction::NextScreen)
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_from_toml_missing_file_returns_error() {
        let path = unique_temp_path("missing");
        let error = KeybindingMap::load_from_toml(&path)
            .expect_err("missing config file should propagate an error");

        assert!(
            error.to_string().contains("failed to read keybindings"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parse_key_str_supports_modifiers_named_keys_and_chars() {
        assert_eq!(
            parse_key_str("ctrl+c"),
            Some(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
        );
        assert_eq!(
            parse_key_str("F1"),
            Some(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("tab"),
            Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("Shift+Tab"),
            Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT))
        );
        assert_eq!(
            parse_key_str("alt+/"),
            Some(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::ALT))
        );
        assert_eq!(
            parse_key_str("?"),
            Some(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE))
        );
        assert_eq!(parse_key_str(""), None);
        assert_eq!(parse_key_str("ctrl+shift+"), None);
        assert_eq!(parse_key_str("f13"), None);
        assert_eq!(parse_key_str("tab+f1"), None);
    }

    fn write_temp_keybindings(name: &str, contents: &str) -> PathBuf {
        let path = unique_temp_path(name);
        fs::write(&path, contents).expect("temporary keybindings file should be writable");
        path
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "bardo-terminal-keybindings-{name}-{}-{nanos}.toml",
            std::process::id()
        ))
    }
}
