//! `bardo-inference` — zero-Golem inference protocol types.
//!
//! Shared between `bardo-gateway`, `golem-inference`, and any other crate
//! that needs to speak the inference protocol without pulling in the full
//! Golem platform stack.
//!
//! # What's here
//!
//! - [`Role`], [`ContentBlock`], [`Message`] — conversation primitives
//! - [`InferenceRequest`], [`InferenceResponse`], [`TokenUsage`] — wire types
//! - [`StopReason`], [`InferenceChunk`], [`ChunkDelta`] — streaming support
//! - [`InferenceError`], [`ErrorPayload`], [`ErrorDetail`] — error vocabulary

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod types;

pub use error::{ErrorDetail, ErrorPayload, InferenceError};
pub use types::{
    ChunkDelta, ContentBlock, InferenceChunk, InferenceRequest, InferenceResponse, Message, Role,
    StopReason, TokenUsage,
};
