//! `bardo-gateway` — inference gateway and provider router.
//!
//! Proxies LLM requests to Anthropic and OpenAI APIs with:
//! - Auto-detection of request format (Anthropic Messages vs OpenAI Chat Completions)
//! - Moka async cache with LRU + TTL for exact request deduplication
//! - Tier-based model routing (T0 suppression, T1 Haiku, T2 Opus/Sonnet)
//! - Per-session cost tracking
//! - Tool definition pruning for unused tools
//! - JSON key normalization for better cache hit rates
//! - Cost tracking headers on every response
//! - X-Api-Key authentication

mod auth;
mod cache;
mod error;
mod format;
mod handler;
mod prefix;
mod pricing;
mod provider;
mod session;
mod sse;
mod state;
mod tier;
mod tools;

use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{get, post},
};
use clap::Parser;
use dashmap::DashMap;
use pricing::default_pricing;

use state::AppState;

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

    let api_key = cli
        .api_key
        .or_else(|| std::env::var("BARDO_GATEWAY_API_KEY").ok())
        .unwrap_or_else(|| {
            let key = uuid::Uuid::new_v4().to_string();
            tracing::warn!(key = %key, "no API key set, generated random key");
            key
        });

    let anthropic_api_key =
        std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    let openai_api_key = std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|k| !k.is_empty());
    let has_openai = openai_api_key.is_some();

    let bind_addr = format!("{}:{}", cli.bind, cli.port);

    let state = AppState {
        api_key: api_key.clone(),
        anthropic_api_key,
        openai_api_key,
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?,
        cache: Arc::new(cache::ResponseCache::new(cli.max_cache, cli.ttl)),
        pricing: default_pricing(),
        bind_addr: bind_addr.clone(),
        stats: Arc::new(state::GatewayStats::new()),
        sessions: Arc::new(DashMap::new()),
        tool_tracker: Arc::new(tools::ToolTracker::new()),
    };

    // Routes that require authentication
    let authed = Router::new()
        .route("/v1/messages", post(handler::messages))
        .route("/v1/chat/completions", post(handler::chat_completions))
        .route("/v1/costs", get(handler::costs))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    // Public routes
    let app = Router::new()
        .route("/v1/health", get(handler::health))
        .route("/v1/stats", get(handler::stats).with_state(state.clone()))
        .merge(authed)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!(addr = %bind_addr, api_key = %api_key, "bardo-gateway listening");

    if has_openai {
        tracing::info!("OpenAI provider enabled");
    }

    axum::serve(listener, app).await?;
    Ok(())
}
