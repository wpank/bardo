//! `mirage-rs` binary entrypoint.

#![allow(clippy::redundant_pub_crate)]

use std::{fs, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use clap::Parser;
use mirage_rs::{
    ClassificationConfig, DiffClassifier, MirageError,
    fork::{ForkState, HybridDB, MirageFork},
    provider::UpstreamRpc,
    replay::{FollowerConfig, TargetedFollower},
    resources::{MirageMode, Profile, ResourceModel},
    rpc::start_rpc_server,
};
use tokio::sync::broadcast;

/// Command-line configuration for `mirage-rs`.
#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Bind host.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// Bind port.
    #[arg(long, default_value_t = 8545)]
    port: u16,
    /// Optional upstream HTTP RPC URL.
    #[arg(long)]
    rpc_url: Option<String>,
    /// Optional upstream WebSocket URL.
    #[arg(long)]
    ws_url: Option<String>,
    /// Upstream request budget per second.
    #[arg(long, default_value_t = 100)]
    upstream_rps: u32,
    /// Upstream burst multiplier.
    #[arg(long, default_value_t = 200)]
    upstream_burst: u32,
    /// Effective chain ID.
    #[arg(long, default_value_t = 1)]
    chain_id: u64,
    /// Read-cache capacity.
    #[arg(long, default_value_t = 10_000)]
    cache_size: usize,
    /// Read-cache TTL in seconds.
    #[arg(long, default_value_t = 12)]
    cache_ttl_secs: u64,
    /// Resource profile.
    #[arg(long, value_enum, default_value_t = ProfileArg::Standard)]
    profile: ProfileArg,
    /// Inactivity watchdog timeout in seconds.
    #[arg(long)]
    watchdog_timeout: Option<u64>,
    /// Enforce strict nonce checks.
    #[arg(long, default_value_t = false)]
    strict_nonce: bool,
    /// Enforce strict balance checks.
    #[arg(long, default_value_t = false)]
    strict_balance: bool,
    /// Enable signature verification.
    #[arg(long, default_value_t = false)]
    verify_signatures: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ProfileArg {
    Micro,
    Standard,
    Power,
}

impl From<ProfileArg> for Profile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::Micro => Self::Micro,
            ProfileArg::Standard => Self::Standard,
            ProfileArg::Power => Self::Power,
        }
    }
}

fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cli = Cli::parse();

    let resource_model = ResourceModel::for_profile(
        Profile::from(cli.profile),
        Duration::from_secs(cli.cache_ttl_secs),
    );
    if let Err(error) = resource_model
        .ensure_spawn_budget()
        .context("resource budget check failed")
    {
        let exit_code = startup_exit_code(&error).unwrap_or(1);
        tracing::error!(error = %error, exit_code, "mirage startup failed");
        std::process::exit(exit_code);
    }

    // reqwest::blocking::Client::new() panics when called from inside a Tokio
    // async context, so build the upstream (and its blocking HTTP client) here
    // in sync context before starting the runtime.
    let upstream = Arc::new(UpstreamRpc::new_with_limits(
        cli.rpc_url.clone(),
        cli.ws_url.clone(),
        cli.chain_id,
        cli.upstream_rps,
        cli.upstream_burst,
    ));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    if let Err(error) = rt.block_on(run(cli, upstream)) {
        let exit_code = startup_exit_code(&error).unwrap_or(1);
        tracing::error!(error = %error, exit_code, "mirage startup failed");
        std::process::exit(exit_code);
    }
}

async fn run(cli: Cli, upstream: Arc<UpstreamRpc>) -> anyhow::Result<()> {
    let follower_upstream = Arc::clone(&upstream);
    let resource_model = ResourceModel::for_profile(
        Profile::from(cli.profile),
        Duration::from_secs(cli.cache_ttl_secs),
    );
    let head = {
        let upstream = Arc::clone(&upstream);
        let require_probe = cli.rpc_url.is_some();
        tokio::task::spawn_blocking(move || resolve_initial_head(&upstream, require_probe))
            .await
            .context("health check task panicked")??
    };
    let mut fork = ForkState::new(
        HybridDB::new(
            upstream,
            cli.cache_size,
            Duration::from_secs(cli.cache_ttl_secs),
            resource_model.bytecode_cache_capacity(),
            cli.chain_id,
        ),
        head,
        cli.chain_id,
    );
    fork.strict_nonce = cli.strict_nonce;
    fork.strict_balance = cli.strict_balance;
    fork.verify_signatures = cli.verify_signatures;

    let mirage = MirageFork::new(fork, resource_model, MirageMode::Live);
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(8);
    let bind = format!("{}:{}", cli.host, cli.port);
    let (addr, handle) = start_rpc_server(&bind, mirage.clone(), shutdown_tx.clone())
        .await
        .with_context(|| format!("failed to bind {bind}"))?;

    if cli.rpc_url.is_some() || cli.ws_url.is_some() {
        let follower = TargetedFollower::new(
            follower_upstream,
            &mirage,
            DiffClassifier::new(ClassificationConfig::default()),
            FollowerConfig {
                ws_url: cli.ws_url.clone().unwrap_or_default(),
                http_url: cli.rpc_url.clone().unwrap_or_default(),
                block_budget: Duration::from_secs(10),
                filter_addresses: None,
                filter_selectors: None,
            },
            head,
        );
        let shutdown = shutdown_tx.subscribe();
        tokio::spawn(async move {
            if let Err(error) = follower.run(shutdown).await {
                tracing::warn!("targeted follower exited with error: {error}");
            }
        });
    }

    write_artifacts(addr.port()).context("failed to write startup artifacts")?;
    tracing::info!("mirage ready port={} chain={}", addr.port(), cli.chain_id);

    if let Some(timeout_secs) = cli.watchdog_timeout {
        let mirage = mirage.clone();
        let shutdown = shutdown_tx.clone();
        tokio::spawn(async move {
            let timeout = Duration::from_secs(timeout_secs);
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let idle = mirage.idle_for();
                if idle >= timeout {
                    tracing::warn!(
                        idle_secs = idle.as_secs(),
                        timeout_secs,
                        "watchdog idle timeout reached — initiating shutdown"
                    );
                    let _ = shutdown.send(());
                    break;
                }
            }
        });
    }

    tracing::info!("mirage event loop ready");
    // Hold shutdown_tx alive through the select so the broadcast channel
    // stays open until an explicit signal or Ctrl+C.  Without this, Rust may
    // drop shutdown_tx early (last syntactic use is subscribe() above), which
    // closes the channel and causes recv() to return Err(Closed) immediately.
    let shutdown_guard = shutdown_tx;
    tokio::select! {
        result = shutdown_rx.recv() => {
            match result {
                Ok(()) => tracing::warn!("mirage shutdown signal received — exiting"),
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::error!("mirage shutdown channel closed unexpectedly (all senders dropped) — exiting");
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("mirage shutdown receiver lagged by {n} messages — exiting");
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("mirage ctrl+c received");
        }
    }
    drop(shutdown_guard);

    cleanup_artifacts(addr.port());
    handle.stop().context("failed to stop RPC server")?;
    Ok(())
}

fn startup_exit_code(error: &anyhow::Error) -> Option<i32> {
    error.chain().find_map(|cause| {
        cause
            .downcast_ref::<MirageError>()
            .and_then(|mirage_error| match mirage_error {
                MirageError::Unsupported(message)
                    if message.starts_with("insufficient memory:") =>
                {
                    Some(2)
                }
                _ => None,
            })
    })
}

fn write_artifacts(port: u16) -> anyhow::Result<()> {
    let pid_path = PathBuf::from(format!("/tmp/mirage-{port}.pid"));
    let status_path = PathBuf::from(format!("/tmp/mirage-{port}-status.json"));
    fs::write(pid_path, std::process::id().to_string())?;
    fs::write(
        status_path,
        serde_json::json!({"status": "ready", "port": port}).to_string(),
    )?;
    Ok(())
}

fn cleanup_artifacts(port: u16) {
    let pid_path = PathBuf::from(format!("/tmp/mirage-{port}.pid"));
    let status_path = PathBuf::from(format!("/tmp/mirage-{port}-status.json"));
    let _ = fs::remove_file(pid_path);
    let _ = fs::remove_file(status_path);
}

fn resolve_initial_head(upstream: &UpstreamRpc, require_probe: bool) -> anyhow::Result<u64> {
    let health = upstream.health_check();
    if require_probe {
        return health.context("upstream RPC health check failed");
    }
    Ok(health.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{UpstreamRpc, resolve_initial_head, startup_exit_code};
    use crate::MirageError;

    #[test]
    fn initial_head_requires_probe_when_http_upstream_is_configured() {
        let upstream =
            UpstreamRpc::new_with_limits(Some("http://127.0.0.1:9".to_owned()), None, 1, 1, 1);

        let error = resolve_initial_head(&upstream, true).expect_err("probe should fail");
        let message = error.to_string();
        assert!(message.contains("upstream RPC health check failed"));
    }

    #[test]
    fn low_memory_startup_error_exits_with_code_two() {
        let error = anyhow::Error::new(MirageError::Unsupported(
            "insufficient memory: available=1 required=2".to_owned(),
        ))
        .context("resource budget check failed");

        assert_eq!(startup_exit_code(&error), Some(2));
    }

    #[test]
    fn unrelated_startup_errors_use_default_exit_code() {
        let error = anyhow::Error::new(MirageError::BindFailed(8545)).context("failed to bind");

        assert_eq!(startup_exit_code(&error), None);
    }
}
