//! Mori-as-a-Service entry point.
//!
//! Starts an Axum HTTP server exposing the service API, with optional
//! GitHub and Twitter integrations.

use std::net::SocketAddr;

use axum::Router;
use clap::Parser;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use mori_service::api;
use mori_service::state::ServiceState;
use mori_service::types::ServiceConfig;

#[derive(Parser)]
#[command(
    name = "mori-service",
    about = "Mori-as-a-Service: paid build orchestration"
)]
struct Cli {
    /// Port to listen on.
    #[arg(long, default_value = "8080", env = "MORI_SERVICE_PORT")]
    port: u16,

    /// Bind address.
    #[arg(long, default_value = "0.0.0.0", env = "MORI_SERVICE_BIND")]
    bind: String,

    /// Path to config file.
    #[arg(
        long,
        default_value = ".mori/service.toml",
        env = "MORI_SERVICE_CONFIG"
    )]
    config: String,

    /// SQLite database path.
    #[arg(long, default_value = ".mori/service.db", env = "MORI_SERVICE_DB")]
    db: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("mori_service=info".parse()?))
        .init();

    let cli = Cli::parse();

    let config: ServiceConfig = if std::path::Path::new(&cli.config).exists() {
        let raw = std::fs::read_to_string(&cli.config)?;
        toml::from_str(&raw)?
    } else {
        tracing::warn!("Config file not found at {}, using defaults", cli.config);
        default_config()
    };

    let state = ServiceState::new(config, &cli.db)?;
    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", cli.bind, cli.port).parse()?;
    tracing::info!("Mori service listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: ServiceState) -> Router {
    let api_routes = api::router(state.clone());

    Router::new()
        .nest("/api/v1", api_routes)
        .route("/health", axum::routing::get(health))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "mori-service",
    }))
}

fn default_config() -> ServiceConfig {
    ServiceConfig {
        billing: mori_service::types::BillingConfig {
            accept_proposals: true,
            min_proposal_value: 1.0,
            max_proposal_value: 500.0,
            retry_buffer_pct: 0.15,
            escrow: Default::default(),
            session: Default::default(),
        },
        auth: mori_service::types::AuthConfig {
            mode: "both".into(),
            master_key: std::env::var("MORI_MASTER_KEY").ok(),
        },
        gateway: mori_service::types::GatewayConfig { spread_pct: 0.20 },
        integrations: Default::default(),
    }
}
