//! `golem-binary` - single binary entry point for the golem runtime.
//!
//! **Implemented by:** Plan 01
//!
//! This binary is a shell. Later plans implement startup.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("bardo-golem starting - not yet implemented");
    Ok(())
}
