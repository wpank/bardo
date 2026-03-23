//! `golem-inference` — tier routing, inference types, and gateway client.
//!
//! This crate defines the type contracts and traits for foundation model
//! inference. The HTTP gateway (`apps/bardo-gateway`) implements the full
//! service; this crate provides the shared vocabulary.
//!
//! # Tier Routing
//!
//! [`TierRouter::select_model`] maps a [`CognitiveTier`] and vitality score
//! to a model identifier:
//! - T0 → `None` (inference suppressed)
//! - T1 → `"claude-haiku-4-5"`
//! - T2 → `"claude-opus-4-6"` (vitality ≥ 0.3) or `"claude-sonnet-4"` (below)
//!
//! # Client
//!
//! [`GatewayClient`] implements [`InferenceClient`] by talking to a
//! `bardo-gateway` instance over HTTP.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod error;
pub mod router;
pub mod sse;
pub mod types;

pub use client::{GatewayClient, InferenceClient};
pub use error::{ErrorPayload, InferenceError};
pub use router::TierRouter;
pub use types::{
    ChunkDelta, ContentBlock, InferenceChunk, InferenceMeta, InferenceRequest, InferenceResponse,
    Message, Role, StopReason, TokenUsage,
};
