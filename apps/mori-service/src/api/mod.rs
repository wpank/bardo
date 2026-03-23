//! HTTP API route handlers.

pub mod auth;
pub mod deliverables;
pub mod drafts;
pub mod proposals;
pub mod runs;

use axum::Router;

use crate::state::ServiceState;

/// Build the complete API router.
pub fn router(state: ServiceState) -> Router<ServiceState> {
    Router::new()
        .merge(drafts::router())
        .merge(proposals::router())
        .merge(runs::router())
        .merge(deliverables::router())
        .with_state(state)
}
