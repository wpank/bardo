//! `bardo-compute` - compute provisioning service and fleet manager.
//!
//! **Implemented by:** Plan 01
//!
//! This binary is a shell. Later plans implement the compute service.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("bardo-compute starting - not yet implemented");
    Ok(())
}
