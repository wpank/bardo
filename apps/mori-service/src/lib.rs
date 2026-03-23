//! Mori-as-a-Service: paid build orchestration over HTTP.
//!
//! Wraps the Mori orchestrator with a service layer that accepts paid work
//! via REST API, Twitter, and GitHub. Payments in USDC on Base via x402/MPP.

pub mod api;
pub mod db;
pub mod integrations;
pub mod proposal;
pub mod state;
pub mod types;
