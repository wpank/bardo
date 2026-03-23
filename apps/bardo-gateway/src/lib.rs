//! `bardo-gateway` — inference gateway and provider router.
//!
//! This crate can be used as a library (embedded in mori) or as a standalone binary.
//! Call `start_server(config).await` to run the gateway as a background tokio task.

pub mod auth;
pub mod bankr;
pub mod batch;
pub mod cache;
pub mod compress;
pub mod cost_db;
pub mod dashboard;
pub mod error;
pub mod format;
pub mod handler;
pub mod mpp;
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
pub mod venice;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};
use bytes::Bytes;
use dashmap::DashMap;
use tokio::sync::Semaphore;

use providers::{
    AnthropicProvider, BankrProvider, OpenAiProvider, OpenRouterProvider, Provider, VeniceProvider,
};
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
    pub venice_api_key: Option<String>,
    pub bankr_api_key: Option<String>,
    pub bankr_base_url: Option<String>,
    pub max_cache: u64,
    pub ttl: u64,
    pub max_body_size: usize,
    pub max_concurrent: usize,
    pub pool_max_idle: usize,
    pub pool_idle_timeout: u64,
    /// MPP configuration (None = MPP disabled).
    pub mpp: Option<mpp::MppConfig>,
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
            venice_api_key: None,
            bankr_api_key: None,
            bankr_base_url: None,
            max_cache: 10_000,
            ttl: 3600,
            max_body_size: 10_485_760,
            max_concurrent: 256,
            pool_max_idle: 64,
            pool_idle_timeout: 90,
            mpp: None,
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
    assert!(
        !anthropic_api_keys.is_empty(),
        "at least one Anthropic API key required"
    );

    if anthropic_api_keys.len() > 1 {
        tracing::info!(
            keys = anthropic_api_keys.len(),
            "Anthropic key rotation enabled"
        );
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
    let mut provider_list: Vec<Arc<dyn Provider>> = vec![Arc::new(AnthropicProvider::new(
        http.clone(),
        anthropic_api_keys,
    ))];
    if let Some(ref key) = config.venice_api_key {
        provider_list.push(Arc::new(VeniceProvider::new(http.clone(), key.clone())));
        tracing::info!("Venice provider enabled (zero-retention inference)");
    }
    if let Some(ref key) = config.openai_api_key {
        provider_list.push(Arc::new(OpenAiProvider::new(http.clone(), key.clone())));
        tracing::info!("OpenAI provider enabled");
    }
    if let Some(ref key) = config.bankr_api_key {
        let bankr = BankrProvider::new(http.clone(), key.clone());
        let bankr = if let Some(ref base) = config.bankr_base_url {
            bankr.with_base_url(base.clone())
        } else {
            bankr
        };
        provider_list.push(Arc::new(bankr));
        tracing::info!("Bankr provider enabled (self-funding inference)");
    }
    if let Some(ref key) = config.openrouter_api_key {
        provider_list.push(Arc::new(OpenRouterProvider::new(http.clone(), key.clone())));
        tracing::info!("OpenRouter provider enabled");
    }

    let (stats_tx, _) = tokio::sync::broadcast::channel::<state::StatsEvent>(1024);
    let bind_addr = format!("{}:{}", config.bind, config.port);
    let state_http = http.clone();

    // Open cost database and restore persisted state before building AppState.
    let db_path = std::env::current_dir()
        .unwrap_or_default()
        .join(".mori/costs.db");
    let cost_db = cost_db::CostDb::open(&db_path).ok().map(Arc::new);

    // Restore stats from previous runs.
    let gateway_stats = Arc::new(state::GatewayStats::new());
    if let Some(ref db) = cost_db {
        match db.restore_stats() {
            Ok(restored) => {
                cost_db::CostDb::apply_restored(&gateway_stats, &restored);
                tracing::info!(
                    requests = restored.total_requests,
                    cost_usd = restored.total_cost_micro_usd as f64 / 1e6,
                    naive_usd = restored.total_naive_cost_micro_usd as f64 / 1e6,
                    models = restored.model_counts.len(),
                    "restored stats from previous runs"
                );
            }
            Err(e) => tracing::warn!(error = %e, "failed to restore stats"),
        }
    }

    // Restore semantic cache from previous runs.
    // Prefer embedding backend when compiled with the `embedding` feature.
    #[cfg(feature = "embedding")]
    let sem_cache = Arc::new(semantic_cache::SemanticCache::new_with_embeddings(
        0.92, 5000,
    ));
    #[cfg(not(feature = "embedding"))]
    let sem_cache = Arc::new(semantic_cache::SemanticCache::new());
    if let Some(ref db) = cost_db {
        match db.load_semantic_cache() {
            Ok(entries) => {
                let mut count = 0u64;
                let mut skipped = 0u64;
                for entry in entries {
                    // Skip tool_use responses — replaying them produces
                    // invalid tool IDs on subsequent turns.
                    if let Ok(body) = serde_json::from_slice::<serde_json::Value>(&entry.response) {
                        if body.get("stop_reason").and_then(|v| v.as_str()) == Some("tool_use") {
                            skipped += 1;
                            continue;
                        }
                        if let Some(content) = body.get("content").and_then(|c| c.as_array()) {
                            if content
                                .iter()
                                .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
                            {
                                skipped += 1;
                                continue;
                            }
                        }
                    }
                    sem_cache.put(
                        entry.fingerprint,
                        entry.response,
                        entry.cost_usd,
                        entry.model,
                    );
                    count += 1;
                }
                if skipped > 0 {
                    tracing::info!(skipped, "filtered tool_use entries from semantic cache");
                }
                if count > 0 {
                    tracing::info!(entries = count, "restored semantic cache");
                }
            }
            Err(e) => tracing::warn!(error = %e, "failed to restore semantic cache"),
        }
    }

    // Restore tool usage patterns from previous runs.
    let tool_tracker = Arc::new(tools::ToolTracker::new());
    if let Some(ref db) = cost_db {
        match db.load_tool_usage() {
            Ok(entries) => {
                let count = entries.len();
                for entry in entries {
                    for tool in &entry.tools_used {
                        tool_tracker.record_usage(&entry.session_id, tool);
                    }
                    for _ in 0..entry.request_count {
                        tool_tracker.record_request(&entry.session_id);
                    }
                }
                if count > 0 {
                    tracing::info!(sessions = count, "restored tool usage patterns");
                }
            }
            Err(e) => tracing::warn!(error = %e, "failed to restore tool usage"),
        }
    }

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
        stats: gateway_stats,
        sessions: Arc::new(DashMap::new()),
        tool_tracker,
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
        semantic_cache: sem_cache.clone(),
        inflight: Arc::new(DashMap::new()),
        mpp: config.mpp.as_ref().filter(|c| c.enabled).map(|c| {
            tracing::info!(
                recipient = %c.recipient_address,
                spread = c.default_spread,
                "MPP payment protocol enabled"
            );
            Arc::new(mpp::MppState::new(c.clone()))
        }),
        max_body_size: config.max_body_size,
    };

    // Start the cost writer (subscribes to stats broadcast).
    if let Some(ref db) = cost_db {
        db.start_writer(state.stats_tx.subscribe());
        tracing::info!(path = %db_path.display(), "cost tracking enabled");
    }

    // Periodic semantic cache persistence (every 60s).
    if let Some(db) = cost_db.clone() {
        let cache_ref = sem_cache;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                // Persist current semantic cache entries.
                // This is a simple approach — iterate all entries and upsert.
                // For large caches, delta tracking would be better.
                let entries: Vec<(u64, Bytes, f64, String)> = cache_ref
                    .iter()
                    .map(|e| {
                        (
                            e.fingerprint,
                            e.response.clone(),
                            e.cost_usd,
                            e.model.clone(),
                        )
                    })
                    .collect();
                for (fp, resp, cost, model) in entries {
                    let _ = db.save_semantic_entry(fp, &resp, cost, &model).await;
                }
            }
        });
    }

    // Periodic tool usage persistence (every 5 minutes).
    if let Some(db) = cost_db {
        let tracker = state.tool_tracker.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                for entry in tracker.all_sessions() {
                    let _ = db.save_tool_usage(&entry.0, &entry.1, entry.2).await;
                }
            }
        });
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

    // MPP session expiry background task.
    if state.mpp.is_some() {
        let mpp_ref = state.mpp.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Some(ref mpp) = mpp_ref {
                    let expired = mpp.sessions.expire_stale();
                    if !expired.is_empty() {
                        tracing::info!(count = expired.len(), "expired stale MPP sessions");
                    }
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            mpp::middleware::mpp_payment,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    let dashboard_dir = std::env::current_dir()
        .unwrap_or_default()
        .join("tmp/bardo-dashboard");

    // MPP session management routes (auth handled by MPP protocol, not API key).
    let mpp_routes = Router::new()
        .route("/v1/mpp/sessions", post(mpp::middleware::session_open))
        .route(
            "/v1/mpp/sessions/{id}",
            get(mpp::middleware::session_status).delete(mpp::middleware::session_close),
        );

    let app = Router::new()
        .route("/v1/health", get(handler::health))
        .route("/v1/models", get(handler::models))
        .route("/v1/stats", get(handler::stats).with_state(state.clone()))
        .route("/v1/ws/stats", get(dashboard::ws_stats))
        .nest_service(
            "/dashboard",
            tower_http::services::ServeDir::new(&dashboard_dir),
        )
        .merge(mpp_routes)
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
