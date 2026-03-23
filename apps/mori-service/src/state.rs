//! Shared application state for the Mori service.

use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;

use crate::db::ServiceDb;
use crate::proposal::ProposalEngine;
use crate::types::{BuildEvent, RunId, ServiceConfig};

/// Shared state threaded through all Axum handlers.
#[derive(Clone)]
pub struct ServiceState {
    /// Service configuration.
    pub config: Arc<ServiceConfig>,
    /// SQLite persistence layer.
    pub db: Arc<ServiceDb>,
    /// Proposal generation engine.
    pub engine: Arc<ProposalEngine>,
    /// Per-run SSE broadcast channels.
    pub event_channels: Arc<DashMap<RunId, broadcast::Sender<BuildEvent>>>,
    /// HTTP client for outbound calls (GitHub API, Twitter API, webhooks).
    pub http: reqwest::Client,
}

impl ServiceState {
    /// Create a new service state from config and database path.
    pub fn new(config: ServiceConfig, db_path: &str) -> anyhow::Result<Self> {
        let db = Arc::new(ServiceDb::open(db_path)?);
        let engine = Arc::new(ProposalEngine::new(config.billing.retry_buffer_pct));

        Ok(Self {
            config: Arc::new(config),
            db,
            engine,
            event_channels: Arc::new(DashMap::new()),
            http: reqwest::Client::new(),
        })
    }

    /// Get or create a broadcast channel for a run's SSE events.
    pub fn event_sender(&self, run_id: &str) -> broadcast::Sender<BuildEvent> {
        self.event_channels
            .entry(run_id.to_string())
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    /// Subscribe to a run's SSE events.
    pub fn event_receiver(&self, run_id: &str) -> broadcast::Receiver<BuildEvent> {
        self.event_sender(run_id).subscribe()
    }
}
