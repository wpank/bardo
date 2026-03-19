//! `mirage-rs` — in-process fork state, JSON-RPC surface, and integration client.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::cast_precision_loss,
    clippy::match_same_arms,
    clippy::missing_const_for_fn
)]

use alloy_primitives::{Address, B256, Bytes, U256, keccak256};

pub mod cow;
pub mod fork;
pub mod integration;
pub mod provider;
pub mod replay;
pub mod resources;
pub mod rpc;
pub mod scenario;

pub use cow::{BytecodeCache, CowState, MultiVersionStore, VersionEntry};
pub use fork::{
    Classification, ClassificationConfig, DiffClassifier, DirtyAccount, DirtyStore, ForkState,
    HybridDB, MirageFork, MirageStatus, ReadCache, WatchEntry, WatchSource,
};
pub use integration::{
    EventFilter, EventSource, MirageClient, MirageConfig, MirageEvent, MirageTestInstance,
    PositionRequest, PositionSnapshot, spawn_mirage_test_instance,
};
pub use provider::{BlockTag, UpstreamRpc};
pub use replay::{
    AccountDiff, FollowerConfig, LogEntry, SpeculativeExecutor, SpeculativeResult, StateDiff,
};
pub use resources::{MirageMode, Profile, ResourceModel, ResourceUsage};
pub use scenario::{
    JobStatus, RunMode, Scenario, ScenarioAssertions, ScenarioJob, ScenarioResult, ScenarioRunner,
    ScenarioSet, ScenarioSetStatus, ScenarioStatus,
};

/// Result alias for mirage operations.
pub type Result<T> = std::result::Result<T, MirageError>;

/// Transaction request accepted by the RPC server and client.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// Sender address.
    pub from: Option<Address>,
    /// Destination address.
    pub to: Option<Address>,
    /// Gas limit.
    pub gas: Option<u64>,
    /// Transferred value.
    pub value: Option<U256>,
    /// Input data.
    #[serde(default, alias = "input")]
    pub data: Option<Bytes>,
    /// Legacy gas price.
    pub gas_price: Option<u128>,
    /// Nonce.
    pub nonce: Option<u64>,
    /// Chain ID.
    pub chain_id: Option<u64>,
}

/// Simplified bytecode wrapper.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Bytecode(Bytes);

impl Bytecode {
    /// Creates bytecode from raw bytes.
    #[must_use]
    pub fn new_raw(bytes: Bytes) -> Self {
        Self(bytes)
    }

    /// Returns the bytecode hash.
    #[must_use]
    pub fn hash_slow(&self) -> B256 {
        keccak256(&self.0)
    }

    /// Returns the underlying bytes.
    #[must_use]
    pub fn bytecode(&self) -> &Bytes {
        &self.0
    }
}

/// Simplified account information used by the lazy fork.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// Code hash.
    pub code_hash: B256,
    /// Contract bytecode.
    pub code: Option<Bytecode>,
}

/// Simplified execution result used by the fallback executor.
#[derive(Debug, Clone, Default)]
pub struct ExecutionResult {
    /// Whether the execution succeeded.
    pub success: bool,
    /// Gas used.
    pub gas_used: u64,
    /// Output bytes.
    pub output: Bytes,
}

/// Shared error surface for `mirage-rs` library code.
#[derive(Debug, thiserror::Error)]
pub enum MirageError {
    /// Invalid JSON-RPC parameters or malformed local API input.
    #[error("invalid params: {0}")]
    InvalidParams(String),
    /// Unsupported operation for the current simplified fork state.
    #[error("unsupported operation: {0}")]
    Unsupported(String),
    /// Referenced account cannot be used as the transaction sender.
    #[error("invalid from address: {0}")]
    InvalidFrom(Address),
    /// Requested snapshot ID does not exist or was already consumed.
    #[error("snapshot not found: {0}")]
    SnapshotNotFound(u64),
    /// Requested scenario set does not exist.
    #[error("scenario set not found: {0}")]
    SetNotFound(String),
    /// Requested scenario job does not exist.
    #[error("scenario job not found: {0}")]
    JobNotFound(String),
    /// Requested scenario job has not completed yet.
    #[error("scenario job not complete: {0}")]
    JobNotComplete(String),
    /// The requested protocol type is not supported by the position helper.
    #[error("unknown protocol type: {0}")]
    UnknownProtocolType(String),
    /// ERC-20 slot detection failed for the requested token/account pair.
    #[error("ERC-20 balance slot detection failed for token {0}")]
    SlotDetectionFailed(Address),
    /// Target address is already tracked and the watch list is at capacity.
    #[error("watch list full")]
    WatchListFull,
    /// Upstream RPC request failed.
    #[error("upstream RPC error: {0}")]
    Upstream(String),
    /// Local bind failed.
    #[error("failed to bind mirage on port {0}")]
    BindFailed(u16),
    /// A time-bound operation exceeded its timeout.
    #[error("operation timed out: {0}")]
    Timeout(String),
    /// I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// HTTP client failure.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    /// Background blocking task failure.
    #[error("background task failed: {0}")]
    BackgroundTask(String),
    /// JSON serialization or parsing failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// TOML parsing failure.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

impl MirageError {
    /// Returns the JSON-RPC error code for this failure.
    #[must_use]
    pub const fn rpc_code(&self) -> i32 {
        match self {
            Self::InvalidParams(_) => -32602,
            Self::Unsupported(_) => -32603,
            Self::InvalidFrom(_) => -32010,
            Self::SnapshotNotFound(_) => -32001,
            Self::SetNotFound(_) => -32050,
            Self::JobNotFound(_) => -32054,
            Self::JobNotComplete(_) => -32055,
            Self::UnknownProtocolType(_) => -32040,
            Self::SlotDetectionFailed(_) => -32020,
            Self::WatchListFull => -32030,
            Self::Upstream(_) => -32099,
            Self::BindFailed(_)
            | Self::Timeout(_)
            | Self::BackgroundTask(_)
            | Self::Io(_)
            | Self::Http(_)
            | Self::Json(_)
            | Self::Toml(_) => -32603,
        }
    }
}
