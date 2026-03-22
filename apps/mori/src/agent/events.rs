use super::roles::AgentRole;

/// Events emitted by agent connections, consumed by the event bus
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Streaming text delta from agent
    MessageDelta {
        role: AgentRole,
        instance: Option<String>,
        content: String,
    },
    /// Agent turn completed
    TurnCompleted {
        role: AgentRole,
        instance: Option<String>,
        thread_id: Option<String>,
    },
    /// Live diff update
    DiffUpdated {
        role: AgentRole,
        instance: Option<String>,
        diff: String,
    },
    /// Agent requests approval for a command or file change
    ApprovalRequested {
        role: AgentRole,
        instance: Option<String>,
        command: String,
        /// The JSON-RPC request id from the server-initiated approval request.
        /// Must be echoed back verbatim in the response (may be string or number).
        approval_id: serde_json::Value,
    },
    /// Token usage update
    TokenUsage {
        role: AgentRole,
        instance: Option<String>,
        input_tokens: u64,
        output_tokens: u64,
        context_window: Option<u64>,
        /// Cost in USD from the gateway's X-Mori-Cost-Usd header (or claude result event).
        cost_usd: Option<f64>,
    },
    /// Command execution output (routed to command output panel)
    CommandOutput {
        role: AgentRole,
        instance: Option<String>,
        content: String,
    },
    /// Agent process error
    Error {
        role: AgentRole,
        instance: Option<String>,
        error: String,
    },
    /// Agent process exited
    Exited {
        role: AgentRole,
        instance: Option<String>,
        exit_code: Option<i32>,
    },
}

impl AgentEvent {
    /// The role that emitted this event
    pub fn role(&self) -> AgentRole {
        match self {
            Self::MessageDelta { role, .. }
            | Self::TurnCompleted { role, .. }
            | Self::DiffUpdated { role, .. }
            | Self::ApprovalRequested { role, .. }
            | Self::TokenUsage { role, .. }
            | Self::CommandOutput { role, .. }
            | Self::Error { role, .. }
            | Self::Exited { role, .. } => *role,
        }
    }

    /// The instance identifier, if this event came from a multi-agent pool
    pub fn instance(&self) -> Option<&str> {
        match self {
            Self::MessageDelta { instance, .. }
            | Self::TurnCompleted { instance, .. }
            | Self::DiffUpdated { instance, .. }
            | Self::ApprovalRequested { instance, .. }
            | Self::TokenUsage { instance, .. }
            | Self::CommandOutput { instance, .. }
            | Self::Error { instance, .. }
            | Self::Exited { instance, .. } => instance.as_deref(),
        }
    }

    /// Return a clone of this event with the instance field set.
    pub fn with_instance(self, id: Option<String>) -> Self {
        match self {
            Self::MessageDelta { role, content, .. } => Self::MessageDelta {
                role,
                instance: id,
                content,
            },
            Self::TurnCompleted {
                role, thread_id, ..
            } => Self::TurnCompleted {
                role,
                instance: id,
                thread_id,
            },
            Self::DiffUpdated { role, diff, .. } => Self::DiffUpdated {
                role,
                instance: id,
                diff,
            },
            Self::ApprovalRequested {
                role,
                command,
                approval_id,
                ..
            } => Self::ApprovalRequested {
                role,
                instance: id,
                command,
                approval_id,
            },
            Self::TokenUsage {
                role,
                input_tokens,
                output_tokens,
                context_window,
                cost_usd,
                ..
            } => Self::TokenUsage {
                role,
                instance: id,
                input_tokens,
                output_tokens,
                context_window,
                cost_usd,
            },
            Self::CommandOutput { role, content, .. } => Self::CommandOutput {
                role,
                instance: id,
                content,
            },
            Self::Error { role, error, .. } => Self::Error {
                role,
                instance: id,
                error,
            },
            Self::Exited {
                role, exit_code, ..
            } => Self::Exited {
                role,
                instance: id,
                exit_code,
            },
        }
    }
}
