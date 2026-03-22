//! `bardo-terminal` is the primary interactive surface for the workspace.
//!
//! This binary owns terminal setup, the render loop, and shutdown cleanup.

mod animation;
mod app;
mod layout;
mod mock;
mod navigation;
mod palette;
mod particles;
pub mod rpc_server;
mod screen;
mod screens;
mod sonification;
mod sound;
mod state;
mod sys_stats;
mod widgets;

use std::io::stdout;

use anyhow::{Result, anyhow};
pub use app::{App, EventSource};
use clap::Parser;
pub use screen::{Screen, ScreenId};
pub use state::{AppAction, AppState};

use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "bardo-terminal")]
struct Cli {
    /// Run in headless mode (TestBackend, no TTY)
    #[arg(long)]
    headless: bool,
    /// JSON-RPC port for headless mode
    #[arg(long, default_value = "9100")]
    rpc_port: u16,
    /// Terminal width for headless mode
    #[arg(long, default_value = "120")]
    width: u16,
    /// Terminal height for headless mode
    #[arg(long, default_value = "40")]
    height: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if cli.headless {
        return run_headless(cli.rpc_port, cli.width, cli.height).await;
    }

    // Panic hook must run before raw mode / alternate screen so a mid-render panic
    // cannot strand the user's shell.
    install_panic_hook();

    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let mut events = EventSource::Crossterm;

    // `App::run` blocks on crossterm I/O and timing; keep it off the async scheduler's
    // cooperative path while preserving a Tokio runtime for future async work.
    let run_result = tokio::task::block_in_place(|| app.run(&mut terminal, &mut events));

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

async fn run_headless(rpc_port: u16, width: u16, height: u16) -> Result<()> {
    use ratatui::backend::TestBackend;

    let (action_tx, mut action_rx) = tokio::sync::mpsc::channel::<String>(64);
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Channel for injecting crossterm events into the app loop.
    let (event_tx, event_rx) = std::sync::mpsc::channel::<Event>();

    let rpc = rpc_server::RpcServer::new(rpc_port, action_tx, shutdown_tx);
    tokio::spawn(async move {
        if let Err(e) = rpc.run().await {
            tracing::error!(error = ?e, "RPC server error");
        }
    });

    // Bridge: convert RPC actions and shutdown signals into crossterm events.
    let event_tx_action = event_tx.clone();
    let event_tx_shutdown = event_tx;
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(action) = action_rx.recv() => {
                    if action == "quit" {
                        let _ = event_tx_action.send(Event::Key(KeyEvent::new(
                            KeyCode::Char('q'),
                            KeyModifiers::NONE,
                        )));
                        break;
                    }
                }
                Some(()) = shutdown_rx.recv() => {
                    let _ = event_tx_shutdown.send(Event::Key(KeyEvent::new(
                        KeyCode::Char('q'),
                        KeyModifiers::NONE,
                    )));
                    break;
                }
            }
        }
    });

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    let mut events = EventSource::Channel(event_rx);

    tokio::task::block_in_place(|| app.run(&mut terminal, &mut events))?;

    Ok(())
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
        // Best-effort: mirror `restore_terminal` so the shell is usable after a panic.
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

    #[test]
    fn restore_terminal_collects_all_error_messages() {
        let mut cleanup = FakeCleanup {
            fail_disable_raw_mode: true,
            fail_leave_alternate_screen: true,
            fail_disable_mouse_capture: true,
            fail_show_cursor: true,
            ..FakeCleanup::default()
        };

        let error = restore_terminal(&mut cleanup).expect_err("expected combined errors");
        let text = error.to_string();
        assert!(text.contains("disable raw mode"));
        assert!(text.contains("leave alternate screen"));
        assert!(text.contains("disable mouse capture"));
        assert!(text.contains("show cursor"));
    }
}
