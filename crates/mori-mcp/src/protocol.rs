//! MCP protocol types: JSON-RPC 2.0 + MCP-specific structures.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 core types
// ---------------------------------------------------------------------------

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version, always "2.0".
    pub jsonrpc: String,
    /// Request id. `None` (or null) for notifications.
    #[serde(default)]
    pub id: Option<Value>,
    /// The method name.
    pub method: String,
    /// Optional parameters.
    #[serde(default)]
    pub params: Option<Value>,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version, always "2.0".
    pub jsonrpc: String,
    /// The request id this response corresponds to.
    pub id: Value,
    /// The result, present on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The error, present on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Machine-readable error code.
    pub code: i64,
    /// Human-readable description.
    pub message: String,
    /// Optional structured data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    /// Build a success response.
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Build an error response.
    pub fn error(id: Value, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// MCP-specific types
// ---------------------------------------------------------------------------

/// Server identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    pub version: String,
}

/// Result returned from `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// The protocol version the server speaks.
    pub protocol_version: String,
    /// Capabilities the server advertises.
    pub capabilities: ServerCapabilities,
    /// Server identity.
    pub server_info: ServerInfo,
}

/// Capabilities advertised by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Present when the server supports tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Signals that the server supports tools (empty object).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {}

/// A single tool definition returned in `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// Tool name, used when calling `tools/call`.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema describing the expected input.
    pub input_schema: Value,
}

/// Wrapper for the `tools/list` result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    /// Available tools.
    pub tools: Vec<ToolDefinition>,
}

/// Parameters received in `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    /// Which tool to invoke.
    pub name: String,
    /// Arguments for the tool.
    #[serde(default)]
    pub arguments: Value,
}

/// A single piece of content in a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    /// Content type, e.g. "text".
    #[serde(rename = "type")]
    pub type_: String,
    /// The textual payload.
    pub text: String,
}

/// Result of a `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    /// The content blocks.
    pub content: Vec<ToolContent>,
    /// Whether the tool call resulted in an error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn jsonrpc_request_roundtrip() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };
        let serialized = serde_json::to_string(&req).unwrap_or_default();
        let deserialized: JsonRpcRequest =
            serde_json::from_str(&serialized).unwrap_or_else(|_| req.clone());
        assert_eq!(deserialized.method, "tools/list");
    }

    #[test]
    fn jsonrpc_success_response_roundtrip() {
        let resp = JsonRpcResponse::success(json!(42), json!({"ok": true}));
        let s = serde_json::to_string(&resp).unwrap_or_default();
        let parsed: JsonRpcResponse = serde_json::from_str(&s).unwrap_or_else(|_| resp.clone());
        assert!(parsed.error.is_none());
        assert!(parsed.result.is_some());
        assert_eq!(parsed.id, json!(42));
    }

    #[test]
    fn jsonrpc_error_response_roundtrip() {
        let resp = JsonRpcResponse::error(json!("abc"), -32601, "Method not found".into());
        let s = serde_json::to_string(&resp).unwrap_or_default();
        let parsed: JsonRpcResponse = serde_json::from_str(&s).unwrap_or_else(|_| resp.clone());
        assert!(parsed.result.is_none());
        let err = parsed.error.as_ref();
        assert!(err.is_some());
        if let Some(e) = err {
            assert_eq!(e.code, -32601);
        }
    }

    #[test]
    fn initialize_result_serialization() {
        let init = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {}),
            },
            server_info: ServerInfo {
                name: "mori".to_string(),
                version: "0.1.0".to_string(),
            },
        };
        let s = serde_json::to_string(&init).unwrap_or_default();
        assert!(s.contains("protocolVersion"));
        assert!(s.contains("serverInfo"));
    }

    #[test]
    fn tool_call_result_serialization() {
        let result = ToolCallResult {
            content: vec![ToolContent {
                type_: "text".to_string(),
                text: "hello".to_string(),
            }],
            is_error: None,
        };
        let s = serde_json::to_string(&result).unwrap_or_default();
        // isError should be absent (skip_serializing_if = None)
        assert!(!s.contains("isError"));

        let err_result = ToolCallResult {
            content: vec![ToolContent {
                type_: "text".to_string(),
                text: "boom".to_string(),
            }],
            is_error: Some(true),
        };
        let s2 = serde_json::to_string(&err_result).unwrap_or_default();
        assert!(s2.contains("isError"));
    }

    #[test]
    fn notification_has_no_id() {
        let json_str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let req: JsonRpcRequest =
            serde_json::from_str(json_str).unwrap_or_else(|_| JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: None,
                method: String::new(),
                params: None,
            });
        assert!(req.id.is_none());
        assert_eq!(req.method, "notifications/initialized");
    }
}
