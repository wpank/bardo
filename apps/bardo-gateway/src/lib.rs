//! `bardo-gateway` — inference gateway and provider router.
//!
//! This crate can be used as a library (embedded in mori) or as a standalone binary.
//! Call `start_server(config).await` to run the gateway as a background tokio task.

pub mod auth;
pub mod batch;
pub mod cache;
pub mod compress;
pub mod cost_db;
pub mod dashboard;
pub mod error;
pub mod format;
pub mod handler;
pub mod prefix;
pub mod pricing;
pub mod provider;
pub mod providers;
pub mod semantic_cache;
pub mod session;
pub mod sse;
pub mod state;
pub mod tier;
pub mod tools;

use std::sync::Arc;

use axum::{
    Router, middleware,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use dashmap::DashMap;
use tokio::sync::Semaphore;

use providers::{AnthropicProvider, OpenAiProvider, OpenRouterProvider, Provider};
use state::AppState;

/// Configuration for the gateway server.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub port: u16,
    pub bind: String,
    pub api_key: String,
    pub anthropic_api_keys: Vec<String>,
    pub openai_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub max_cache: u64,
    pub ttl: u64,
    pub max_body_size: usize,
    pub max_concurrent: usize,
    pub pool_max_idle: usize,
    pub pool_idle_timeout: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: 4000,
            bind: "127.0.0.1".into(),
            api_key: String::new(),
            anthropic_api_keys: vec![],
            openai_api_key: None,
            openrouter_api_key: None,
            max_cache: 10_000,
            ttl: 3600,
            max_body_size: 10_485_760,
            max_concurrent: 256,
            pool_max_idle: 64,
            pool_idle_timeout: 90,
        }
    }
}

/// Start the gateway server. Runs until the task is cancelled or the process exits.
///
/// Returns the bound address string (e.g. `"127.0.0.1:4000"`).
pub async fn start_server(config: GatewayConfig) -> anyhow::Result<()> {
    let api_key = if config.api_key.is_empty() {
        let key = uuid::Uuid::new_v4().to_string();
        tracing::warn!(key = %key, "no API key set, generated random key");
        key
    } else {
        config.api_key.clone()
    };

    let anthropic_api_keys = config.anthropic_api_keys.clone();
    assert!(!anthropic_api_keys.is_empty(), "at least one Anthropic API key required");

    if anthropic_api_keys.len() > 1 {
        tracing::info!(keys = anthropic_api_keys.len(), "Anthropic key rotation enabled");
    }

    let primary_key = anthropic_api_keys[0].clone();

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .pool_max_idle_per_host(config.pool_max_idle)
        .pool_idle_timeout(std::time::Duration::from_secs(config.pool_idle_timeout))
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .tcp_nodelay(true)
        .build()?;

    let batch_api_keys = anthropic_api_keys.clone();
    let mut provider_list: Vec<Arc<dyn Provider>> = vec![
        Arc::new(AnthropicProvider::new(http.clone(), anthropic_api_keys)),
    ];
    if let Some(ref key) = config.openai_api_key {
        provider_list.push(Arc::new(OpenAiProvider::new(http.clone(), key.clone())));
        tracing::info!("OpenAI provider enabled");
    }
    if let Some(ref key) = config.openrouter_api_key {
        provider_list.push(Arc::new(OpenRouterProvider::new(http.clone(), key.clone())));
        tracing::info!("OpenRouter provider enabled");
    }

    let (stats_tx, _) = tokio::sync::broadcast::channel::<state::StatsEvent>(1024);
    let bind_addr = format!("{}:{}", config.bind, config.port);
    let state_http = http.clone();

    let state = AppState {
        api_key: api_key.clone(),
        anthropic_api_key: primary_key,
        openai_api_key: config.openai_api_key,
        openrouter_api_key: config.openrouter_api_key,
        providers: provider_list,
        http,
        cache: Arc::new(cache::ResponseCache::new(config.max_cache, config.ttl)),
        pricing: pricing::PricingTable::new(pricing::default_pricing()),
        bind_addr: bind_addr.clone(),
        stats: Arc::new(state::GatewayStats::new()),
        sessions: Arc::new(DashMap::new()),
        tool_tracker: Arc::new(tools::ToolTracker::new()),
        concurrency: if config.max_concurrent > 0 {
            Some(Arc::new(Semaphore::new(config.max_concurrent)))
        } else {
            None
        },
        stats_tx,
        batch_manager: Some({
            let mgr = Arc::new(batch::BatchManager::new(state_http, batch_api_keys));
            mgr.start_poll_loop();
            mgr.start_flush_timer();
            tracing::info!("Batch API enabled");
            mgr
        }),
        semantic_cache: Arc::new(semantic_cache::SemanticCache::new()),
        inflight: Arc::new(DashMap::new()),
    };

    // Persistent cost tracking.
    {
        let db_path = std::env::current_dir()
            .unwrap_or_default()
            .join(".mori/costs.db");
        match cost_db::CostDb::open(&db_path) {
            Ok(db) => {
                let db = Arc::new(db);
                db.start_writer(state.stats_tx.subscribe());
                tracing::info!(path = %db_path.display(), "cost tracking enabled");
            }
            Err(e) => {
                tracing::warn!(error = %e, "cost tracking disabled (SQLite open failed)");
            }
        }
    }

    // Session eviction.
    {
        let sessions = state.sessions.clone();
        let tool_tracker = state.tool_tracker.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
                let mut evicted = 0u64;
                sessions.retain(|_, session| {
                    if session.last_active < cutoff {
                        evicted += 1;
                        false
                    } else {
                        true
                    }
                });
                if evicted > 0 {
                    tool_tracker.retain_sessions(|sid| sessions.contains_key(sid));
                    tracing::info!(evicted, remaining = sessions.len(), "pruned stale sessions");
                }
            }
        });
    }

    // Routes.
    let authed = Router::new()
        .route("/v1/messages", post(handler::messages))
        .route("/v1/chat/completions", post(handler::chat_completions))
        .route("/v1/costs", get(handler::costs))
        .route("/v1/batch/submit", post(batch::batch_submit))
        .route("/v1/batch/flush", post(batch::batch_flush))
        .route("/v1/batch/status", get(batch::batch_status))
        .route("/v1/batch/result/{id}", get(batch::batch_result))
        .layer(middleware::from_fn_with_state(state.clone(), auth::require_auth));

    let dashboard_dir = std::env::current_dir()
        .unwrap_or_default()
        .join("tmp/bardo-dashboard");

    let app = Router::new()
        .route("/v1/health", get(handler::health))
        .route("/v1/models", get(handler::models))
        .route("/v1/stats", get(handler::stats).with_state(state.clone()))
        .route("/v1/ws/stats", get(dashboard::ws_stats))
        .nest_service("/dashboard", tower_http::services::ServeDir::new(&dashboard_dir))
        .merge(authed)
        .layer(DefaultBodyLimit::max(config.max_body_size))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!(
        addr = %bind_addr,
        api_key = %api_key,
        max_body = config.max_body_size,
        max_concurrent = config.max_concurrent,
        "bardo-gateway listening"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
