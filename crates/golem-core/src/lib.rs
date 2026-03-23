//! `golem-core` - the Layer 0 shared type vocabulary for the Bardo workspace.
//!
//! This crate defines the zero-dependency foundations used by every later
//! layer: identity, configuration, event fabric, cortical state, taint labels,
//! the extension trait skeleton, HDC primitives, and the tick arena wrapper.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod alloc;
pub mod cognitive;
pub mod config;
pub mod cortical;
pub mod error;
pub mod event;
pub mod extension;
pub mod hdc;
pub mod id;
pub mod taint;

pub use alloc::TickArena;
pub use cognitive::CognitiveTier;
pub use config::*;
pub use cortical::{BehavioralPhase, CorticalSnapshot, CorticalState, PadVector, PlutchikEmotion};
pub use error::{GolemError, Result};
pub use event::{EventFabric, EventPayload, GolemEvent, Subsystem};
pub use extension::{
    AfterTurnCtx, AgentEndCtx, AgentMessage, AgentStartCtx, ContextCtx, DebugCtx, EndCtx, ErrorCtx,
    Extension, ExtensionRegistry, HookId, InputAction, InputCtx, InputMessage, MsgCtx,
    OutboundMessage, PromptCtx, ProviderReqCtx, SessionCtx, SessionReason, SteerCtx, SteerMessage,
    ToolAction, ToolCall, ToolCallCtx, ToolExecCtx, ToolResult, ToolResultCtx, TurnEndCtx,
    TurnStartCtx,
};
pub use hdc::HdcVector;
pub use id::GolemId;
pub use taint::{TaintLabel, TaintedString};
