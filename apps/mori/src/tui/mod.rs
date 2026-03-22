pub mod atmosphere;
pub mod bars;
pub mod color;
pub mod effects_config;
pub mod input;
pub mod layout;
pub mod math;
pub mod modals;
pub mod postfx;
pub mod postfx_pipeline;
pub mod tabs;
pub mod theme;
pub mod vfx;
pub mod views;
pub mod widgets;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

pub use input::TuiAction;

/// Truncate a string to at most `max_chars` characters, appending `suffix` if truncated.
pub fn truncate_chars(s: &str, max_chars: usize, suffix: &str) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let budget = max_chars.saturating_sub(suffix.len());
    let mut r: String = s.chars().take(budget).collect();
    r.push_str(suffix);
    r
}

/// Initialize the terminal for TUI rendering
pub fn init() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal state
pub fn restore() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
