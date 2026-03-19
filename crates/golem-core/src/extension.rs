//! Extension trait skeleton and registry.

use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::GolemError;

/// Session lifecycle reason.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionReason {
    /// Session start.
    Start,
    /// Session resume.
    Resume,
    /// Session is compacting.
    BeforeCompact,
    /// Session is branching.
    BeforeBranch,
}

/// Hook invocation context stubs.
#[derive(Clone, Debug, Default)]
pub struct SessionCtx;
/// Input processing context.
#[derive(Clone, Debug, Default)]
pub struct InputCtx;
/// Agent-start context.
#[derive(Clone, Debug, Default)]
pub struct AgentStartCtx;
/// Turn-start context.
#[derive(Clone, Debug, Default)]
pub struct TurnStartCtx;
/// Context assembly context.
#[derive(Clone, Debug, Default)]
pub struct ContextCtx;
/// Provider-request context.
#[derive(Clone, Debug, Default)]
pub struct ProviderReqCtx;
/// Tool-call context.
#[derive(Clone, Debug, Default)]
pub struct ToolCallCtx;
/// Tool-execution context.
#[derive(Clone, Debug, Default)]
pub struct ToolExecCtx;
/// Tool-result context.
#[derive(Clone, Debug, Default)]
pub struct ToolResultCtx;
/// Turn-end context.
#[derive(Clone, Debug, Default)]
pub struct TurnEndCtx;
/// Agent-end context.
#[derive(Clone, Debug, Default)]
pub struct AgentEndCtx;
/// After-turn context.
#[derive(Clone, Debug, Default)]
pub struct AfterTurnCtx;
/// System-prompt context.
#[derive(Clone, Debug, Default)]
pub struct PromptCtx;
/// Steer context.
#[derive(Clone, Debug, Default)]
pub struct SteerCtx;
/// Outbound-message context.
#[derive(Clone, Debug, Default)]
pub struct MsgCtx;
/// Debug context.
#[derive(Clone, Debug, Default)]
pub struct DebugCtx;
/// Error context.
#[derive(Clone, Debug, Default)]
pub struct ErrorCtx;
/// End-of-process context.
#[derive(Clone, Debug, Default)]
pub struct EndCtx;

/// Input message passed through hook handlers.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMessage {
    /// Message content.
    pub content: String,
    /// Message source.
    pub source: String,
}

/// Input hook actions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputAction {
    /// Pass the message through unchanged.
    Pass,
    /// Replace the message with the supplied text.
    Transform(String),
    /// Suppress the message.
    Suppress,
}

/// Tool invocation request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Tool name.
    pub name: String,
    /// Tool arguments.
    pub arguments: serde_json::Value,
}

/// Tool hook actions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolAction {
    /// Allow the call.
    Allow,
    /// Block the call with a reason.
    Block(String),
    /// Replace the call with a modified payload.
    Modify(ToolCall),
}

/// Tool execution result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    /// Output content.
    pub content: String,
    /// Whether the tool result is an error.
    pub is_error: bool,
}

/// Message passed to the context assembler.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMessage {
    /// Role label.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Mid-execution steering message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SteerMessage {
    /// Steering content.
    pub content: String,
    /// Message priority.
    pub priority: u8,
}

/// Outbound message to a surface or transport.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutboundMessage {
    /// Content payload.
    pub content: String,
    /// Surface identifier.
    pub surface: String,
}

/// Hook identifiers for registry ordering.
#[allow(missing_docs)]
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum HookId {
    Session,
    Input,
    BeforeAgentStart,
    AgentStart,
    TurnStart,
    Context,
    BeforeProviderRequest,
    ToolCall,
    ToolExecutionStart,
    ToolExecutionUpdate,
    ToolExecutionEnd,
    ToolResult,
    TurnEnd,
    AgentEnd,
    AfterTurn,
    SystemPrompt,
    Steer,
    SendMessage,
    Debug,
    Error,
    End,
}

/// Extension trait with default no-op hooks.
#[async_trait]
pub trait Extension: Send + Sync + 'static {
    /// Returns the extension name.
    fn name(&self) -> &str;
    /// Returns the layer number.
    fn layer(&self) -> u8;
    /// Returns named extension dependencies.
    fn depends_on(&self) -> &[&str] {
        &[]
    }

    /// Session hook.
    async fn on_session(
        &self,
        _reason: SessionReason,
        _ctx: &mut SessionCtx,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    /// Input hook.
    async fn on_input(
        &self,
        _msg: &mut InputMessage,
        _ctx: &InputCtx,
    ) -> anyhow::Result<InputAction> {
        Ok(InputAction::Pass)
    }
    /// Pre-agent-start hook.
    async fn on_before_agent_start(&self, _ctx: &mut AgentStartCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Agent-start hook.
    async fn on_agent_start(&self, _ctx: &AgentStartCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Turn-start hook.
    async fn on_turn_start(&self, _ctx: &TurnStartCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Context hook.
    async fn on_context(
        &self,
        _messages: &mut Vec<AgentMessage>,
        _ctx: &ContextCtx,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    /// Provider-request hook.
    async fn on_before_provider_request(&self, _ctx: &mut ProviderReqCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Tool-call hook.
    async fn on_tool_call(
        &self,
        _call: &ToolCall,
        _ctx: &mut ToolCallCtx,
    ) -> anyhow::Result<ToolAction> {
        Ok(ToolAction::Allow)
    }
    /// Tool-execution-start hook.
    async fn on_tool_execution_start(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Tool-execution-update hook.
    async fn on_tool_execution_update(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Tool-execution-end hook.
    async fn on_tool_execution_end(&self, _ctx: &ToolExecCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Tool-result hook.
    async fn on_tool_result(
        &self,
        _result: &mut ToolResult,
        _ctx: &ToolResultCtx,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    /// Turn-end hook.
    async fn on_turn_end(&self, _ctx: &TurnEndCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Agent-end hook.
    async fn on_agent_end(&self, _ctx: &AgentEndCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// After-turn hook.
    async fn on_after_turn(&self, _ctx: &mut AfterTurnCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// System-prompt hook.
    async fn on_system_prompt(&self, _prompt: &mut String, _ctx: &PromptCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Steer hook.
    async fn on_steer(&self, _msg: &SteerMessage, _ctx: &mut SteerCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Send-message hook.
    async fn on_send_message(&self, _msg: &OutboundMessage, _ctx: &MsgCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Debug hook.
    async fn on_debug(&self, _ctx: &DebugCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Error hook.
    async fn on_error(&self, _err: &GolemError, _ctx: &ErrorCtx) -> anyhow::Result<()> {
        Ok(())
    }
    /// Shutdown hook.
    async fn on_end(&self, _ctx: &EndCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Extension registry with topological hook ordering.
pub struct ExtensionRegistry {
    extensions: Vec<Arc<dyn Extension>>,
    firing_orders: HashMap<HookId, Vec<usize>>,
}

impl ExtensionRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            firing_orders: HashMap::new(),
        }
    }

    /// Registers a new extension.
    pub fn register(&mut self, ext: Arc<dyn Extension>) {
        self.extensions.push(ext);
    }

    /// Validates dependencies and computes firing order.
    ///
    /// # Panics
    ///
    /// Panics if the registry contains duplicate extension names, references a
    /// missing dependency, contains a dependency ordered above its consumer, or
    /// contains a dependency cycle.
    pub fn build(&mut self) {
        let mut by_name = HashMap::new();
        for (index, ext) in self.extensions.iter().enumerate() {
            let inserted = by_name.insert(ext.name().to_owned(), index);
            assert!(
                inserted.is_none(),
                "duplicate extension name: {}",
                ext.name()
            );
        }

        let mut indegree = vec![0usize; self.extensions.len()];
        let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); self.extensions.len()];

        for (index, ext) in self.extensions.iter().enumerate() {
            for dep in ext.depends_on() {
                let Some(&dep_index) = by_name.get(*dep) else {
                    panic!(
                        "extension '{}' depends on missing extension '{dep}'",
                        ext.name()
                    );
                };
                let dep_layer = self.extensions[dep_index].layer();
                assert!(
                    dep_layer <= ext.layer(),
                    "extension '{}' (layer {}) depends on '{}' (layer {}) which is higher",
                    ext.name(),
                    ext.layer(),
                    dep,
                    dep_layer
                );
                indegree[index] += 1;
                outgoing[dep_index].push(index);
            }
        }

        let mut ready = BTreeSet::new();
        for (index, ext) in self.extensions.iter().enumerate() {
            if indegree[index] == 0 {
                ready.insert((ext.layer(), index));
            }
        }

        let mut order = Vec::with_capacity(self.extensions.len());
        while let Some((_, index)) = ready.pop_first() {
            order.push(index);
            for &next in &outgoing[index] {
                indegree[next] -= 1;
                if indegree[next] == 0 {
                    let ext = &self.extensions[next];
                    ready.insert((ext.layer(), next));
                }
            }
        }

        assert_eq!(
            order.len(),
            self.extensions.len(),
            "extension dependency cycle detected"
        );

        self.firing_orders.clear();
        for hook in [
            HookId::Session,
            HookId::Input,
            HookId::BeforeAgentStart,
            HookId::AgentStart,
            HookId::TurnStart,
            HookId::Context,
            HookId::BeforeProviderRequest,
            HookId::ToolCall,
            HookId::ToolExecutionStart,
            HookId::ToolExecutionUpdate,
            HookId::ToolExecutionEnd,
            HookId::ToolResult,
            HookId::TurnEnd,
            HookId::AgentEnd,
            HookId::AfterTurn,
            HookId::SystemPrompt,
            HookId::Steer,
            HookId::SendMessage,
            HookId::Debug,
            HookId::Error,
            HookId::End,
        ] {
            self.firing_orders.insert(hook, order.clone());
        }
    }

    fn firing_order(&self, hook: &HookId) -> &[usize] {
        self.firing_orders.get(hook).map_or_else(
            || panic!("extension registry must be built before firing"),
            Vec::as_slice,
        )
    }

    /// Fires the after-turn hook chain.
    pub async fn fire_after_turn(&self, ctx: &mut AfterTurnCtx) -> anyhow::Result<()> {
        for &index in self.firing_order(&HookId::AfterTurn) {
            self.extensions[index].on_after_turn(ctx).await?;
        }
        Ok(())
    }

    /// Fires the tool-call hook chain.
    pub async fn fire_tool_call(
        &self,
        call: &ToolCall,
        ctx: &mut ToolCallCtx,
    ) -> anyhow::Result<ToolAction> {
        let mut current_call = call.clone();
        let mut action = ToolAction::Allow;
        for &index in self.firing_order(&HookId::ToolCall) {
            match self.extensions[index]
                .on_tool_call(&current_call, ctx)
                .await?
            {
                ToolAction::Allow => {}
                ToolAction::Block(reason) => return Ok(ToolAction::Block(reason)),
                ToolAction::Modify(next_call) => {
                    current_call = next_call.clone();
                    action = ToolAction::Modify(next_call);
                }
            }
        }
        Ok(action)
    }

    /// Fires the session hook chain.
    pub async fn fire_session(
        &self,
        reason: SessionReason,
        ctx: &mut SessionCtx,
    ) -> anyhow::Result<()> {
        for &index in self.firing_order(&HookId::Session) {
            self.extensions[index]
                .on_session(reason.clone(), ctx)
                .await?;
        }
        Ok(())
    }

    /// Fires the shutdown hook chain.
    pub async fn fire_end(&self, ctx: &EndCtx) -> anyhow::Result<()> {
        for &index in self.firing_order(&HookId::End) {
            self.extensions[index].on_end(ctx).await?;
        }
        Ok(())
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    struct BadExt;

    #[async_trait]
    impl Extension for BadExt {
        fn name(&self) -> &str {
            "bad"
        }

        fn layer(&self) -> u8 {
            1
        }

        fn depends_on(&self) -> &[&str] {
            &["nonexistent"]
        }
    }

    struct OrderedExt {
        name: &'static str,
        layer: u8,
        deps: &'static [&'static str],
    }

    #[async_trait]
    impl Extension for OrderedExt {
        fn name(&self) -> &str {
            self.name
        }

        fn layer(&self) -> u8 {
            self.layer
        }

        fn depends_on(&self) -> &[&str] {
            self.deps
        }
    }

    #[test]
    fn extension_registry_missing_dep_panics() {
        let result = std::panic::catch_unwind(|| {
            let mut registry = ExtensionRegistry::new();
            registry.register(Arc::new(BadExt));
            registry.build();
        });
        assert!(result.is_err());
    }

    #[test]
    fn extension_registry_topological_order() {
        let mut registry = ExtensionRegistry::new();
        registry.register(Arc::new(OrderedExt {
            name: "alpha",
            layer: 1,
            deps: &[],
        }));
        registry.register(Arc::new(OrderedExt {
            name: "beta",
            layer: 0,
            deps: &[],
        }));
        registry.register(Arc::new(OrderedExt {
            name: "gamma",
            layer: 1,
            deps: &["beta"],
        }));
        registry.build();

        let order = registry
            .firing_orders
            .get(&HookId::AfterTurn)
            .expect("order");
        assert_eq!(order, &vec![1, 0, 2]);
    }
}
