//! `bardo-styx` - Styx relay server for clade sync and knowledge exchange.
//!
//! **Implemented by:** Plan 01
//!
//! This binary is a shell. Later plans implement the Styx relay.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("bardo-styx starting - not yet implemented");
    Ok(())
}
