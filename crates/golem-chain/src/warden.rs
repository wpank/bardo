//! Warden time-delay mechanism for safety-gated on-chain actions.
//!
//! The Warden enforces mandatory time delays between announcing an action and
//! executing it. No state-mutating action can execute without a configurable
//! waiting period. Actual on-chain execution requires Plan 10's `ActionPermit`.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::ChainId;
use crate::error::ChainError;

/// A single time-delayed action with lifecycle tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardenAction {
    /// Unique identifier for this action.
    pub id: Uuid,
    /// What kind of action this is.
    pub action_type: ActionType,
    /// Minimum wait before the action may execute.
    pub delay: Duration,
    /// Wall-clock time of announcement.
    pub announced_at: SystemTime,
    /// Current lifecycle status.
    pub status: WardenStatus,
    /// Optional human-readable description for logging and TUI.
    pub description: Option<String>,
    /// Chain this action targets.
    pub chain_id: ChainId,
}

/// Lifecycle status of a warden action.
///
/// Transitions: `Announced` -> `Waiting` -> `Ready` -> `Executed` | `Cancelled`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WardenStatus {
    /// Registered but delay has not started counting.
    Announced,
    /// Delay is counting down.
    Waiting,
    /// Delay elapsed; action may execute.
    Ready,
    /// Executed (terminal).
    Executed,
    /// Cancelled before execution (terminal).
    Cancelled,
}

/// Categories of time-delayed actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// V4 pool hook parameter change.
    PoolParameterUpdate,
    /// Vault strategy adjustment.
    VaultRebalance,
    /// UniswapX order cancellation.
    OrderCancel,
    /// Large swap above configurable threshold.
    LargeSwap {
        /// USD threshold that triggered the delay.
        threshold_usd: u64,
    },
    /// Cross-chain bridge operation.
    CrossChainBridge,
    /// Generic extensibility.
    Custom(String),
}

impl ActionType {
    /// Default delay for this action type.
    pub fn default_delay(&self) -> Duration {
        match self {
            ActionType::PoolParameterUpdate => Duration::from_secs(3600),
            ActionType::VaultRebalance => Duration::from_secs(1800),
            ActionType::OrderCancel => Duration::from_secs(300),
            ActionType::LargeSwap { .. } => Duration::from_secs(600),
            ActionType::CrossChainBridge => Duration::from_secs(7200),
            ActionType::Custom(_) => Duration::from_secs(300),
        }
    }
}

/// Registry of pending warden actions.
pub struct Warden {
    actions: HashMap<Uuid, WardenAction>,
}

impl Warden {
    /// Create a new empty warden registry.
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Register a new action with the default delay for its type.
    /// Returns the UUID callers use to track it.
    pub fn announce(
        &mut self,
        action_type: ActionType,
        chain_id: ChainId,
        description: Option<String>,
    ) -> Uuid {
        let delay = action_type.default_delay();
        self.announce_with_delay(action_type, delay, chain_id, description)
    }

    /// Register a new action with a custom delay.
    pub fn announce_with_delay(
        &mut self,
        action_type: ActionType,
        delay: Duration,
        chain_id: ChainId,
        description: Option<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let action = WardenAction {
            id,
            action_type,
            delay,
            announced_at: SystemTime::now(),
            status: WardenStatus::Announced,
            description,
            chain_id,
        };
        self.actions.insert(id, action);
        id
    }

    /// Cancel a pending action. Returns `Err` if already executed.
    pub fn cancel(&mut self, id: Uuid) -> Result<(), ChainError> {
        let action = self
            .actions
            .get_mut(&id)
            .ok_or(ChainError::WardenActionNotFound(id))?;

        match action.status {
            WardenStatus::Executed => Err(ChainError::WardenActionNotFound(id)),
            WardenStatus::Cancelled => Ok(()),
            _ => {
                action.status = WardenStatus::Cancelled;
                Ok(())
            }
        }
    }

    /// Mark an action as executed. Called by the executor (Plan 10+) after broadcast.
    pub fn mark_executed(&mut self, id: Uuid) -> Result<(), ChainError> {
        let action = self
            .actions
            .get_mut(&id)
            .ok_or(ChainError::WardenActionNotFound(id))?;

        match action.status {
            WardenStatus::Ready => {
                action.status = WardenStatus::Executed;
                Ok(())
            }
            _ => Err(ChainError::WardenActionNotFound(id)),
        }
    }

    /// Poll all actions and advance status based on elapsed time.
    /// Returns IDs that transitioned to `Ready` this poll.
    pub fn poll(&mut self) -> Vec<Uuid> {
        let now = SystemTime::now();
        let mut newly_ready = Vec::new();

        for action in self.actions.values_mut() {
            match action.status {
                WardenStatus::Announced => {
                    action.status = WardenStatus::Waiting;
                }
                WardenStatus::Waiting => {
                    let elapsed = now
                        .duration_since(action.announced_at)
                        .unwrap_or(Duration::ZERO);
                    if elapsed >= action.delay {
                        action.status = WardenStatus::Ready;
                        newly_ready.push(action.id);
                    }
                }
                _ => {}
            }
        }

        newly_ready
    }

    /// Fetch a single action by ID.
    pub fn get(&self, id: Uuid) -> Option<&WardenAction> {
        self.actions.get(&id)
    }

    /// All actions in `Ready` state.
    pub fn ready_actions(&self) -> Vec<&WardenAction> {
        self.actions
            .values()
            .filter(|a| a.status == WardenStatus::Ready)
            .collect()
    }

    /// All non-terminal actions (Announced + Waiting + Ready).
    pub fn pending_actions(&self) -> Vec<&WardenAction> {
        self.actions
            .values()
            .filter(|a| {
                matches!(
                    a.status,
                    WardenStatus::Announced | WardenStatus::Waiting | WardenStatus::Ready
                )
            })
            .collect()
    }

    /// Prune terminal actions older than `max_age`.
    pub fn prune(&mut self, max_age: Duration) {
        let now = SystemTime::now();
        self.actions.retain(|_, action| {
            if matches!(
                action.status,
                WardenStatus::Executed | WardenStatus::Cancelled
            ) {
                let age = now
                    .duration_since(action.announced_at)
                    .unwrap_or(Duration::ZERO);
                age < max_age
            } else {
                true
            }
        });
    }
}

impl Default for Warden {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_warden_lifecycle_announced_to_ready() {
        let mut warden = Warden::new();
        let id =
            warden.announce_with_delay(ActionType::OrderCancel, Duration::from_millis(0), 1, None);

        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Announced);

        warden.poll();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Waiting);

        let ready = warden.poll();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Ready);
        assert!(ready.contains(&id));
    }

    #[test]
    fn test_warden_cancel_prevents_execution() {
        let mut warden = Warden::new();
        let id =
            warden.announce_with_delay(ActionType::OrderCancel, Duration::from_millis(0), 1, None);

        warden.poll();
        warden.poll();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Ready);

        warden.cancel(id).unwrap();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Cancelled);

        assert!(warden.mark_executed(id).is_err());
    }

    #[test]
    fn test_warden_default_delays_per_type() {
        assert_eq!(
            ActionType::PoolParameterUpdate.default_delay(),
            Duration::from_secs(3600),
        );
        assert_eq!(
            ActionType::VaultRebalance.default_delay(),
            Duration::from_secs(1800),
        );
        assert_eq!(
            ActionType::OrderCancel.default_delay(),
            Duration::from_secs(300),
        );
        assert_eq!(
            ActionType::LargeSwap {
                threshold_usd: 100_000
            }
            .default_delay(),
            Duration::from_secs(600),
        );
        assert_eq!(
            ActionType::CrossChainBridge.default_delay(),
            Duration::from_secs(7200),
        );
        assert_eq!(
            ActionType::Custom("test".into()).default_delay(),
            Duration::from_secs(300),
        );
    }

    #[test]
    fn test_warden_prune_removes_terminal() {
        let mut warden = Warden::new();
        let id =
            warden.announce_with_delay(ActionType::OrderCancel, Duration::from_millis(0), 1, None);

        warden.poll();
        warden.poll();
        warden.mark_executed(id).unwrap();

        warden.prune(Duration::ZERO);
        assert!(warden.get(id).is_none());
    }

    #[test]
    fn test_warden_prune_keeps_non_terminal() {
        let mut warden = Warden::new();
        let id = warden.announce(ActionType::VaultRebalance, 1, None);

        warden.prune(Duration::ZERO);
        assert!(warden.get(id).is_some());
    }

    #[test]
    fn test_warden_action_uuid_uniqueness_1m_insertions() {
        let mut warden = Warden::new();
        let mut ids = HashSet::new();

        for _ in 0..10_000 {
            let id = warden.announce(ActionType::OrderCancel, 1, None);
            assert!(ids.insert(id), "duplicate UUID detected");
        }
    }

    #[test]
    fn test_warden_status_state_machine_valid_transitions() {
        let mut warden = Warden::new();

        let id =
            warden.announce_with_delay(ActionType::OrderCancel, Duration::from_millis(0), 1, None);
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Announced);

        warden.poll();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Waiting);

        warden.poll();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Ready);

        warden.mark_executed(id).unwrap();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Executed);

        assert!(warden.mark_executed(id).is_err());
        assert!(warden.cancel(id).is_err());
    }

    #[test]
    fn test_warden_poll_monotonic_advancement_timing() {
        let mut warden = Warden::new();
        let id = warden.announce_with_delay(
            ActionType::CrossChainBridge,
            Duration::from_secs(3600),
            1,
            None,
        );

        warden.poll();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Waiting);

        warden.poll();
        assert_eq!(warden.get(id).unwrap().status, WardenStatus::Waiting);
    }
}
