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

use anyhow::{Result, anyhow};
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
    let teardown_result = teardown_terminal(&mut terminal);

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

trait TerminalCleanup {
    fn disable_raw_mode(&mut self) -> Result<()>;
    fn leave_alternate_screen(&mut self) -> Result<()>;
    fn disable_mouse_capture(&mut self) -> Result<()>;
    fn show_cursor(&mut self) -> Result<()>;
}

struct CrosstermCleanup<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl TerminalCleanup for CrosstermCleanup<'_> {
    fn disable_raw_mode(&mut self) -> Result<()> {
        disable_raw_mode().map_err(Into::into)
    }

    fn leave_alternate_screen(&mut self) -> Result<()> {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).map_err(Into::into)
    }

    fn disable_mouse_capture(&mut self) -> Result<()> {
        execute!(self.terminal.backend_mut(), DisableMouseCapture).map_err(Into::into)
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.terminal.show_cursor().map_err(Into::into)
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

fn teardown_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    restore_terminal(&mut CrosstermCleanup { terminal })
}

fn restore_terminal(cleanup: &mut impl TerminalCleanup) -> Result<()> {
    let mut errors = Vec::new();

    if let Err(error) = cleanup.disable_raw_mode() {
        errors.push(format!("disable raw mode: {error:#}"));
    }
    if let Err(error) = cleanup.leave_alternate_screen() {
        errors.push(format!("leave alternate screen: {error:#}"));
    }
    if let Err(error) = cleanup.disable_mouse_capture() {
        errors.push(format!("disable mouse capture: {error:#}"));
    }
    if let Err(error) = cleanup.show_cursor() {
        errors.push(format!("show cursor: {error:#}"));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(errors.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::{Result, TerminalCleanup, restore_terminal};
    use anyhow::anyhow;

    #[derive(Default)]
    struct FakeCleanup {
        actions: Vec<&'static str>,
        fail_disable_raw_mode: bool,
        fail_leave_alternate_screen: bool,
        fail_disable_mouse_capture: bool,
        fail_show_cursor: bool,
    }

    impl TerminalCleanup for FakeCleanup {
        fn disable_raw_mode(&mut self) -> Result<()> {
            self.actions.push("disable_raw_mode");
            if self.fail_disable_raw_mode {
                Err(anyhow!("disable raw mode failed"))
            } else {
                Ok(())
            }
        }

        fn leave_alternate_screen(&mut self) -> Result<()> {
            self.actions.push("leave_alternate_screen");
            if self.fail_leave_alternate_screen {
                Err(anyhow!("leave alternate screen failed"))
            } else {
                Ok(())
            }
        }

        fn disable_mouse_capture(&mut self) -> Result<()> {
            self.actions.push("disable_mouse_capture");
            if self.fail_disable_mouse_capture {
                Err(anyhow!("disable mouse capture failed"))
            } else {
                Ok(())
            }
        }

        fn show_cursor(&mut self) -> Result<()> {
            self.actions.push("show_cursor");
            if self.fail_show_cursor {
                Err(anyhow!("show cursor failed"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn restore_terminal_runs_every_cleanup_step() {
        let mut cleanup = FakeCleanup {
            fail_disable_raw_mode: true,
            fail_disable_mouse_capture: true,
            ..FakeCleanup::default()
        };

        let error = restore_terminal(&mut cleanup).expect_err("cleanup should report failures");
        let error_text = error.to_string();

        assert_eq!(
            cleanup.actions,
            vec![
                "disable_raw_mode",
                "leave_alternate_screen",
                "disable_mouse_capture",
                "show_cursor",
            ]
        );
        assert!(error_text.contains("disable raw mode"));
        assert!(error_text.contains("disable mouse capture"));
        assert!(!error_text.contains("show cursor"));
    }

    #[test]
    fn restore_terminal_succeeds_when_every_step_succeeds() {
        let mut cleanup = FakeCleanup::default();

        restore_terminal(&mut cleanup).expect("cleanup should succeed");

        assert_eq!(
            cleanup.actions,
            vec![
                "disable_raw_mode",
                "leave_alternate_screen",
                "disable_mouse_capture",
                "show_cursor",
            ]
        );
    }
}
