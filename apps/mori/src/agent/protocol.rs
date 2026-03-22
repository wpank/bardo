use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Outgoing JSON-RPC request
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            method: method.into(),
            params,
        }
    }
}

/// Raw incoming message — could be a response (has id) or notification (has method, no id)
/// Note: id is Value because the protocol uses both integer and string IDs.
#[derive(Debug, Clone, Deserialize)]
pub struct RawServerMessage {
    pub id: Option<Value>,
    pub method: Option<String>,
    pub result: Option<Value>,
    pub error: Option<ServerError>,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

impl RawServerMessage {
    pub fn is_response(&self) -> bool {
        self.id.is_some() && self.method.is_none()
    }

    pub fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }

    pub fn is_server_request(&self) -> bool {
        self.id.is_some() && self.method.is_some()
    }

    /// Extract numeric id if available
    pub fn numeric_id(&self) -> Option<u64> {
        self.id.as_ref().and_then(|v| v.as_u64())
    }
}

/// Initialize request params (codex app-server format)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Turn start params (codex app-server format)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnStartParams {
    pub input: Vec<InputItem>,
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum InputItem {
    #[serde(rename = "text")]
    Text { text: String },
}

// Approval responses use raw JSON (v2 protocol: { "decision": "accept" | "decline" })

// Cursor ACP protocol types
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorInitializeParams {
    /// ACP requires an integer 1, not a date string.
    pub protocol_version: u32,
    pub client_info: ClientInfo,
    pub client_capabilities: serde_json::Value,
}

/// `mode` = "agent" for full Composer 2 tool access (vs "plan"/"ask" for read-only modes).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorSessionNewParams {
    pub cwd: String,
    pub mode: &'static str,
    /// ACP requires this field (can be empty); omitting it causes a -32603 error.
    pub mcp_servers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPromptParams {
    pub session_id: String,
    pub prompt: Vec<CursorPromptItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CursorPromptItem {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Claude Code CLI stream-json protocol types
// ---------------------------------------------------------------------------

/// The `system` event emitted at the start of a turn (subtype: "init").
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeInitEvent {
    pub session_id: String,
    pub model: String,
}

/// The `assistant` event — one or more content blocks.
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeAssistantEvent {
    pub message: ClaudeMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeMessage {
    pub content: Vec<ClaudeContentBlock>,
    pub usage: Option<ClaudeUsage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

/// The `result` event — final turn summary with session_id and cost.
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeResultEvent {
    pub subtype: String,
    pub session_id: String,
    pub is_error: bool,
    pub num_turns: u64,
    pub total_cost_usd: Option<f64>,
    pub usage: Option<ClaudeUsage>,
}

/// Top-level envelope for any stream-json line.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeStreamEvent {
    System(serde_json::Value),
    Assistant(ClaudeAssistantEvent),
    Tool(serde_json::Value),
    Result(ClaudeResultEvent),
    #[serde(other)]
    Unknown,
}
