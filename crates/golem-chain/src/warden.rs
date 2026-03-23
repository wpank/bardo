//! Time-delay safety mechanism for chain actions.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Classification of a warden-protected action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    /// Token transfer.
    Transfer,
    /// Token approval / permit.
    Approval,
    /// Liquidity position change.
    LiquidityChange,
    /// Governance vote or delegation.
    Governance,
    /// Contract deployment.
    Deployment,
}

/// Lifecycle state of a warden action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WardenStatus {
    /// Action has been announced but delay has not started.
    Announced,
    /// Delay timer is running.
    Waiting,
    /// Delay elapsed; action may be executed.
    Ready,
    /// Action was successfully executed.
    Executed,
    /// Action was cancelled before execution.
    Cancelled,
}

/// A single warden-tracked action with its lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardenAction {
    /// Unique action identifier.
    pub id: Uuid,
    /// What kind of action this is.
    pub action_type: ActionType,
    /// Current lifecycle state.
    pub status: WardenStatus,
}

/// The warden: tracks pending chain actions and enforces time delays.
pub struct Warden {
    _private: (),
}
