//! `bardo-terminal` - standalone ratatui TUI client.
//!
//! **Implemented by:** Plan 01
//!
//! This binary is a shell. Later plans implement the terminal UI.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("bardo-terminal starting - not yet implemented");
    Ok(())
}
