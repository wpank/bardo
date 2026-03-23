//! Auto-detection and translation between Anthropic and OpenAI request formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use bardo_inference::{
    ContentBlock, InferenceRequest, InferenceResponse, Message, Role, StopReason,
};

/// Detected request format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFormat {
    /// Anthropic Messages API format.
    Anthropic,
    /// OpenAI Chat Completions format.
    OpenAi,
}

/// Detect format from the raw JSON request body.
///
/// Anthropic indicators: model contains "claude", or `max_tokens` is present at the
/// top level (required by the Anthropic API, optional in OpenAI).
/// Claude Code sends content as a plain string in Anthropic format, so we can't
/// use "content is a string" as an OpenAI signal.
pub fn detect_format(body: &Value) -> ApiFormat {
    // If the model name contains "claude", it's destined for Anthropic.
    if let Some(model) = body.get("model").and_then(|m| m.as_str()) {
        if model.contains("claude") {
            return ApiFormat::Anthropic;
        }
        // Known OpenAI model prefixes
        if model.starts_with("gpt-")
            || model.starts_with("o1")
            || model.starts_with("o3")
            || model.starts_with("o4")
        {
            return ApiFormat::OpenAi;
        }
    }

    // Anthropic requires max_tokens at top level; OpenAI makes it optional.
    if body.get("max_tokens").is_some() {
        return ApiFormat::Anthropic;
    }

    // Anthropic uses "system" as a top-level string; OpenAI puts system in messages.
    if body.get("system").is_some() {
        return ApiFormat::Anthropic;
    }

    // Fallback: check for OpenAI-style "messages" with a "system" role entry.
    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                return ApiFormat::OpenAi;
            }
        }
    }

    // Default to Anthropic (the primary use case).
    ApiFormat::Anthropic
}

// ── OpenAI types ───────────────────────────────────────────────────

/// OpenAI Chat Completion request.
#[derive(Debug, Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
}

fn default_max_tokens() -> Option<u32> {
    Some(4096)
}

/// OpenAI message.
#[derive(Debug, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI Chat Completion response.
#[derive(Debug, Serialize)]
pub struct OpenAiResponse {
    pub id: String,
    pub object: String,
    pub model: String,
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
}

/// OpenAI response choice.
#[derive(Debug, Serialize)]
pub struct OpenAiChoice {
    pub index: u32,
    pub message: OpenAiResponseMessage,
    pub finish_reason: Option<String>,
}

/// OpenAI response message.
#[derive(Debug, Serialize)]
pub struct OpenAiResponseMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI usage.
#[derive(Debug, Serialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

// ── Raw Value translation (provider-agnostic) ──────────────────────

/// Extract text content from an Anthropic message's `content` field.
///
/// Anthropic `content` is either a plain string or an array of content blocks
/// like `[{"type":"text","text":"..."}]`. Both cases return the concatenated text.
fn extract_anthropic_content(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| {
                if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                    b.get("text").and_then(|t| t.as_str()).map(String::from)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

/// Convert an Anthropic Messages API request body to OpenAI Chat Completions format.
///
/// Used when a client speaks Anthropic format (e.g. Claude Code) but the gateway
/// should forward to an OpenAI-compatible provider.
pub fn anthropic_body_to_openai_request(body: &Value) -> Value {
    let mut messages: Vec<Value> = Vec::new();

    // Anthropic puts the system prompt at the top level; OpenAI injects it as
    // the first message with role "system".
    if let Some(system) = body.get("system").and_then(|s| s.as_str()) {
        messages.push(serde_json::json!({"role": "system", "content": system}));
    }

    if let Some(anthropic_messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in anthropic_messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = msg
                .get("content")
                .map(extract_anthropic_content)
                .unwrap_or_default();
            messages.push(serde_json::json!({"role": role, "content": content}));
        }
    }

    let mut req = serde_json::json!({
        "model": body.get("model").cloned().unwrap_or(Value::String("gpt-4o".into())),
        "messages": messages,
    });

    // Pass through shared fields.
    for field in &["max_tokens", "temperature", "stream"] {
        if let Some(v) = body.get(*field) {
            req[*field] = v.clone();
        }
    }

    req
}

/// Convert an Anthropic Messages API response body to OpenAI Chat Completions format.
///
/// Used when the gateway forwarded to Anthropic on behalf of a client that
/// speaks OpenAI format (e.g. Cursor, Codex).
pub fn anthropic_body_to_openai_response(body: &Value) -> Value {
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg_unknown");
    let model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Extract text from first content block.
    let content_text = body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|block| {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                block.get("text").and_then(|t| t.as_str())
            } else {
                None
            }
        })
        .unwrap_or("");

    let finish_reason = match body
        .get("stop_reason")
        .and_then(|r| r.as_str())
        .unwrap_or("end_turn")
    {
        "end_turn" => "stop",
        "max_tokens" => "length",
        "tool_use" => "tool_calls",
        _ => "stop",
    };

    let input_tokens = body
        .get("usage")
        .and_then(|u| u.get("input_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);
    let output_tokens = body
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);

    serde_json::json!({
        "id": id,
        "object": "chat.completion",
        "model": model,
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": content_text},
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": input_tokens,
            "completion_tokens": output_tokens,
            "total_tokens": input_tokens + output_tokens,
        },
    })
}

/// Convert an OpenAI Chat Completions response body to Anthropic Messages format.
///
/// Used when the gateway forwarded to OpenAI on behalf of a client that
/// speaks Anthropic format (e.g. Claude Code).
pub fn openai_body_to_anthropic_response(body: &Value) -> Value {
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg_unknown");
    let model = body
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let content_text = body
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    let stop_reason = match body
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(|r| r.as_str())
        .unwrap_or("stop")
    {
        "stop" => "end_turn",
        "length" => "max_tokens",
        "tool_calls" => "tool_use",
        _ => "end_turn",
    };

    let input_tokens = body
        .get("usage")
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);
    let output_tokens = body
        .get("usage")
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0);

    serde_json::json!({
        "id": id,
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": [{"type": "text", "text": content_text}],
        "stop_reason": stop_reason,
        "stop_sequence": null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
        },
    })
}

// ── Conversion ─────────────────────────────────────────────────────

/// Convert an OpenAI request to Anthropic format.
pub fn openai_to_anthropic(req: &OpenAiRequest) -> InferenceRequest {
    let mut system = None;
    let mut messages = Vec::new();

    for msg in &req.messages {
        if msg.role == "system" {
            system = Some(msg.content.clone());
        } else {
            let role = if msg.role == "assistant" {
                Role::Assistant
            } else {
                Role::User
            };
            messages.push(Message::text(role, &msg.content));
        }
    }

    InferenceRequest {
        model: req.model.clone(),
        messages,
        system,
        max_tokens: req.max_tokens.unwrap_or(4096),
        temperature: req.temperature.unwrap_or(1.0),
        stream: req.stream,
        tools: None,
        metadata: None,
    }
}

/// Convert an Anthropic response to OpenAI format.
#[allow(dead_code)]
pub fn anthropic_to_openai(resp: &InferenceResponse) -> OpenAiResponse {
    let content = resp
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    let finish_reason = resp.stop_reason.as_ref().map(|r| match r {
        StopReason::EndTurn => "stop".into(),
        StopReason::MaxTokens => "length".into(),
        StopReason::ToolUse => "tool_calls".into(),
        StopReason::StopSequence => "stop".into(),
    });

    OpenAiResponse {
        id: resp.id.clone(),
        object: "chat.completion".into(),
        model: resp.model.clone(),
        choices: vec![OpenAiChoice {
            index: 0,
            message: OpenAiResponseMessage {
                role: "assistant".into(),
                content,
            },
            finish_reason,
        }],
        usage: OpenAiUsage {
            prompt_tokens: resp.usage.input_tokens,
            completion_tokens: resp.usage.output_tokens,
            total_tokens: resp.usage.input_tokens + resp.usage.output_tokens,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anthropic_request_system_extracted() {
        let body = serde_json::json!({
            "model": "gpt-4o",
            "system": "You are a helpful assistant.",
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 100,
        });
        let req = anthropic_body_to_openai_request(&body);
        let messages = req["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are a helpful assistant.");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[1]["content"], "Hello");
    }

    #[test]
    fn anthropic_request_content_blocks_flattened() {
        let body = serde_json::json!({
            "model": "gpt-4o",
            "messages": [{
                "role": "user",
                "content": [{"type": "text", "text": "Hello"}, {"type": "text", "text": " world"}]
            }],
            "max_tokens": 50,
        });
        let req = anthropic_body_to_openai_request(&body);
        assert_eq!(req["messages"][0]["content"], "Hello world");
    }

    #[test]
    fn anthropic_response_to_openai_fields() {
        let body = serde_json::json!({
            "id": "msg_123",
            "model": "claude-sonnet-4",
            "content": [{"type": "text", "text": "Hi there"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5},
        });
        let resp = anthropic_body_to_openai_response(&body);
        assert_eq!(resp["id"], "msg_123");
        assert_eq!(resp["object"], "chat.completion");
        assert_eq!(resp["choices"][0]["message"]["content"], "Hi there");
        assert_eq!(resp["choices"][0]["finish_reason"], "stop");
        assert_eq!(resp["usage"]["prompt_tokens"], 10);
        assert_eq!(resp["usage"]["completion_tokens"], 5);
        assert_eq!(resp["usage"]["total_tokens"], 15);
    }

    #[test]
    fn openai_response_to_anthropic_fields() {
        let body = serde_json::json!({
            "id": "chatcmpl-abc",
            "model": "gpt-4o",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "Hi"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 8, "completion_tokens": 3, "total_tokens": 11},
        });
        let resp = openai_body_to_anthropic_response(&body);
        assert_eq!(resp["id"], "chatcmpl-abc");
        assert_eq!(resp["type"], "message");
        assert_eq!(resp["content"][0]["text"], "Hi");
        assert_eq!(resp["stop_reason"], "end_turn");
        assert_eq!(resp["usage"]["input_tokens"], 8);
        assert_eq!(resp["usage"]["output_tokens"], 3);
    }

    #[test]
    fn stop_reason_mappings_anthropic_to_openai() {
        for (input, expected) in &[
            ("end_turn", "stop"),
            ("max_tokens", "length"),
            ("tool_use", "tool_calls"),
        ] {
            let body = serde_json::json!({
                "id": "x", "model": "m",
                "content": [{"type":"text","text":""}],
                "stop_reason": input,
                "usage": {"input_tokens":0,"output_tokens":0},
            });
            let resp = anthropic_body_to_openai_response(&body);
            assert_eq!(
                resp["choices"][0]["finish_reason"], *expected,
                "input: {input}"
            );
        }
    }

    #[test]
    fn stop_reason_mappings_openai_to_anthropic() {
        for (input, expected) in &[
            ("stop", "end_turn"),
            ("length", "max_tokens"),
            ("tool_calls", "tool_use"),
        ] {
            let body = serde_json::json!({
                "id": "x", "model": "m",
                "choices": [{"index":0,"message":{"role":"assistant","content":""},"finish_reason":input}],
                "usage": {"prompt_tokens":0,"completion_tokens":0,"total_tokens":0},
            });
            let resp = openai_body_to_anthropic_response(&body);
            assert_eq!(resp["stop_reason"], *expected, "input: {input}");
        }
    }
}
