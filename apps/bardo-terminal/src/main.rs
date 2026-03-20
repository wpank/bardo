//! `bardo-terminal` is the primary interactive surface for the workspace.
//!
//! This binary owns terminal setup, the render loop, and shutdown cleanup.

mod app;
mod layout;
mod palette;
mod screen;
mod screens;
mod state;
mod sys_stats;
mod widgets;

use std::io::stdout;

use anyhow::Result;
use app::App;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    install_panic_hook();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let run_result = app.run(&mut terminal);
    let teardown_result = teardown_terminal(terminal);

    match (run_result, teardown_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(run_error), Ok(())) => Err(run_error),
        (Ok(()), Err(teardown_error)) => Err(teardown_error),
        (Err(run_error), Err(teardown_error)) => {
            tracing::error!(error = ?teardown_error, "terminal teardown failed after run error");
            Err(run_error)
        }
    }
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture, Show);
        original_hook(panic_info);
    }));
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        let _ = disable_raw_mode();
        return Err(error.into());
    }

    match Terminal::new(CrosstermBackend::new(stdout)) {
        Ok(terminal) => Ok(terminal),
        Err(error) => {
            let _ = execute!(
                std::io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture,
                Show
            );
            let _ = disable_raw_mode();
            Err(error.into())
        }
    }
}

fn teardown_terminal(mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )?;
    terminal.show_cursor()?;
    Ok(())
}
