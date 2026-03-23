//! Core inference types — re-exported from `bardo-inference`, with Golem-specific extensions.

// Re-export the shared protocol types from bardo-inference.
pub use bardo_inference::{
    ChunkDelta, ContentBlock, InferenceChunk, InferenceRequest, InferenceResponse, Message, Role,
    StopReason, TokenUsage,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use golem_core::{CognitiveTier, GolemId};

// ── Golem-specific metadata ────────────────────────────────────────

/// Per-request metadata for routing and attribution.
///
/// Golem-specific extension not shared with `bardo-inference` because it
/// references `GolemId` and `CognitiveTier` from `golem-core`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceMeta {
    /// The golem (or agent) making this request.
    pub golem_id: GolemId,
    /// Cognitive tier for routing.
    pub tier: CognitiveTier,
    /// Vitality score (0.0-1.0) for T2 model selection.
    pub vitality: f32,
    /// Unique request ID.
    pub request_id: Uuid,
}

impl InferenceMeta {
    /// Create new metadata with a fresh request ID.
    pub fn new(golem_id: GolemId, tier: CognitiveTier, vitality: f32) -> Self {
        Self {
            golem_id,
            tier,
            vitality,
            request_id: Uuid::new_v4(),
        }
    }
}
