//! CLI entry point for the standalone bardo-gateway binary.
//!
//! When embedded inside mori, the library's `start_server()` is called directly
//! with a `GatewayConfig` — this binary is not used.

use alloy::primitives::Address;
use bardo_gateway::GatewayConfig;
use clap::Parser;

/// Bardo inference gateway.
#[derive(Parser)]
#[command(
    name = "bardo-gateway",
    about = "Inference gateway and provider router"
)]
struct Cli {
    /// Port to listen on.
    #[arg(short, long, default_value = "4000")]
    port: u16,

    /// Bind address.
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    /// Gateway API key (also reads BARDO_GATEWAY_API_KEY env var).
    #[arg(long, env = "BARDO_GATEWAY_API_KEY")]
    api_key: Option<String>,

    /// Maximum cache entries.
    #[arg(long, default_value = "10000")]
    max_cache: u64,

    /// Cache TTL in seconds.
    #[arg(long, default_value = "3600")]
    ttl: u64,

    /// Maximum request body size in bytes (default: 10MB).
    #[arg(long, default_value = "10485760")]
    max_body_size: usize,

    /// Maximum concurrent in-flight requests (0 = unlimited).
    #[arg(long, default_value = "256")]
    max_concurrent: usize,

    /// Maximum idle connections per upstream host.
    #[arg(long, default_value = "64")]
    pool_max_idle: usize,

    /// Idle connection timeout in seconds.
    #[arg(long, default_value = "90")]
    pool_idle_timeout: u64,

    /// Enable MPP (Machine Payment Protocol) server.
    #[arg(long, env = "BARDO_MPP_ENABLED")]
    mpp_enabled: bool,

    /// MPP recipient address (hex, gateway wallet that receives payments).
    #[arg(long, env = "BARDO_MPP_RECIPIENT")]
    mpp_recipient: Option<String>,

    /// MPP spread percentage (default 0.20 = 20%).
    #[arg(long, env = "BARDO_MPP_SPREAD", default_value = "0.20")]
    mpp_spread: f64,

    /// MPP session TTL in seconds.
    #[arg(long, env = "BARDO_MPP_SESSION_TTL", default_value = "3600")]
    mpp_session_ttl: u64,

    /// MPP quote validity in seconds.
    #[arg(long, env = "BARDO_MPP_QUOTE_VALIDITY", default_value = "300")]
    mpp_quote_validity: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bardo_gateway=info,tower_http=info".into()),
        )
        .init();

    let cli = Cli::parse();

    // Collect Anthropic API keys from environment.
    let primary_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");
    let mut anthropic_api_keys = vec![primary_key];
    for i in 2..=10 {
        if let Ok(key) = std::env::var(format!("ANTHROPIC_API_KEY_{i}")) {
            if !key.is_empty() {
                anthropic_api_keys.push(key);
            }
        }
    }

    let config = GatewayConfig {
        port: cli.port,
        bind: cli.bind,
        api_key: cli.api_key.unwrap_or_default(),
        anthropic_api_keys,
        openai_api_key: std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|k| !k.is_empty()),
        openrouter_api_key: std::env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|k| !k.is_empty()),
        max_cache: cli.max_cache,
        ttl: cli.ttl,
        max_body_size: cli.max_body_size,
        max_concurrent: cli.max_concurrent,
        pool_max_idle: cli.pool_max_idle,
        pool_idle_timeout: cli.pool_idle_timeout,
        mpp: if cli.mpp_enabled {
            let recipient = cli
                .mpp_recipient
                .as_deref()
                .and_then(|s| s.parse::<Address>().ok())
                .unwrap_or(Address::ZERO);
            Some(bardo_gateway::mpp::MppConfig {
                enabled: true,
                recipient_address: recipient,
                default_spread: cli.mpp_spread,
                session_ttl: cli.mpp_session_ttl,
                quote_validity: cli.mpp_quote_validity,
            })
        } else {
            None
        },
    };

    bardo_gateway::start_server(config).await
}
