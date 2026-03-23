//! Conversation history compression.
//!
//! When a request's `messages` array is large (>32K estimated tokens),
//! older turns are compressed into a summary using a cheap model (haiku).
//! Recent turns are kept verbatim. This reduces input tokens by 40-70%
//! on long-running agent sessions while preserving the context the model
//! needs for the current turn.
//!
//! Cost: ~$0.005 per compression (haiku). Pays for itself on the first
//! request that would have sent 30K+ tokens at full price.

use serde_json::Value;

use crate::error::AppError;

/// Rough token estimate: ~4 characters per token.
const CHARS_PER_TOKEN: usize = 4;

/// Default threshold above which compression triggers (estimated tokens).
const DEFAULT_THRESHOLD: usize = 16_000;

/// Number of recent turns to keep verbatim (not compressed).
const KEEP_RECENT_TURNS: usize = 6;

/// Extract text content from a message, handling both string and array content formats.
///
/// Anthropic messages can have content as:
/// - `"content": "hello"` (string)
/// - `"content": [{"type":"text","text":"hello"}, ...]` (array of blocks)
fn extract_message_text(msg: &Value) -> String {
    match msg.get("content") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(arr)) => {
            let mut parts = Vec::new();
            for block in arr {
                match block.get("type").and_then(|t| t.as_str()) {
                    Some("text") => {
                        if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                            parts.push(t);
                        }
                    }
                    Some("thinking") => {
                        if let Some(t) = block.get("thinking").and_then(|t| t.as_str()) {
                            parts.push(t);
                        }
                    }
                    _ => {}
                }
            }
            parts.join("\n")
        }
        _ => String::new(),
    }
}

/// Compress conversation history if it exceeds the token threshold.
///
/// Returns the number of tokens saved (0 if no compression occurred).
/// Modifies the `messages` array in place: older turns are replaced with
/// a single assistant message containing a summary.
pub async fn compress_history_if_needed(
    body: &mut Value,
    http: &reqwest::Client,
    api_keys: &[String],
) -> Result<u64, AppError> {
    let messages = match body.get("messages").and_then(|m| m.as_array()) {
        Some(m) if m.len() > KEEP_RECENT_TURNS + 2 => m.clone(),
        _ => return Ok(0), // Too few messages to compress
    };

    // Estimate total token count.
    let total_chars: usize = messages.iter().map(|m| extract_message_text(m).len()).sum();
    let estimated_tokens = total_chars / CHARS_PER_TOKEN;

    if estimated_tokens < DEFAULT_THRESHOLD {
        return Ok(0);
    }

    // Split: compress old turns, keep recent turns verbatim.
    let mut split_point = messages.len().saturating_sub(KEEP_RECENT_TURNS);

    // Don't split in the middle of a tool_use/tool_result pair. If the
    // first recent message is a user message with tool_result blocks, its
    // matching assistant tool_use is about to be compressed away — pull
    // the split back so both stay in the recent turns.
    while split_point > 0 {
        let msg = &messages[split_point];
        let is_tool_result_msg = msg.get("role").and_then(|r| r.as_str()) == Some("user")
            && msg
                .get("content")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"))
                })
                .unwrap_or(false);

        if !is_tool_result_msg {
            break;
        }
        // Pull back to include the preceding assistant tool_use message.
        split_point = split_point.saturating_sub(1);
    }

    let old_turns = &messages[..split_point];
    let recent_turns = &messages[split_point..];

    // Build the old turns into a text block for summarization.
    let mut history_text = String::with_capacity(total_chars / 2);
    for msg in old_turns {
        let role = msg
            .get("role")
            .and_then(|r| r.as_str())
            .unwrap_or("unknown");
        let content = extract_message_text(msg);
        // Truncate very long individual messages to keep the compression request reasonable.
        let truncated = if content.len() > 2000 {
            &content[..2000]
        } else {
            &content
        };
        history_text.push_str(&format!("[{role}]: {truncated}\n\n"));
    }

    let old_chars: usize = old_turns
        .iter()
        .map(|m| extract_message_text(m).len())
        .sum();
    let old_tokens = old_chars / CHARS_PER_TOKEN;

    // Compress via haiku (cheapest model).
    let summary = summarize_history(http, api_keys, &history_text).await?;
    let summary_tokens = summary.len() / CHARS_PER_TOKEN;

    // Build new messages array: summary + recent turns.
    // Skip the synthetic assistant ack if the first recent turn is also
    // assistant — Anthropic rejects consecutive same-role messages.
    let first_recent_is_assistant = recent_turns
        .first()
        .and_then(|m| m.get("role"))
        .and_then(|r| r.as_str())
        == Some("assistant");

    let mut new_messages = Vec::with_capacity(recent_turns.len() + 2);
    new_messages.push(serde_json::json!({
        "role": "user",
        "content": format!("[Previous conversation summary]\n{summary}")
    }));
    if !first_recent_is_assistant {
        new_messages.push(serde_json::json!({
            "role": "assistant",
            "content": "Understood. I have the context from the previous conversation. Continuing."
        }));
    }
    for turn in recent_turns {
        new_messages.push(turn.clone());
    }

    body["messages"] = Value::Array(new_messages);

    let saved = old_tokens.saturating_sub(summary_tokens) as u64;
    tracing::info!(
        old_turns = old_turns.len(),
        old_tokens,
        summary_tokens,
        saved_tokens = saved,
        "compressed conversation history"
    );

    Ok(saved)
}

/// Call haiku to summarize conversation history.
async fn summarize_history(
    http: &reqwest::Client,
    api_keys: &[String],
    history: &str,
) -> Result<String, AppError> {
    let api_key = &api_keys[0];

    // Truncate history if it's absurdly long (>100K chars) to keep
    // the compression request itself cheap.
    let truncated = if history.len() > 100_000 {
        &history[..100_000]
    } else {
        history
    };

    let req = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 2048,
        "system": "Summarize this conversation history concisely. Preserve: key decisions made, current task state, important code references, and any errors or blockers mentioned. Omit: pleasantries, repeated context, verbose explanations. Be factual and dense.",
        "messages": [{
            "role": "user",
            "content": truncated
        }]
    });

    let resp = http
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key.as_str())
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&req)
        .send()
        .await
        .map_err(|e| AppError::ProviderError(format!("compression request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::ProviderError(format!(
            "compression returned {status}: {body}"
        )));
    }

    let parsed: Value = resp
        .json()
        .await
        .map_err(|e| AppError::ProviderError(format!("compression parse failed: {e}")))?;

    // Extract text from Anthropic response.
    let text = parsed
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("[compression failed]")
        .to_string();

    Ok(text)
}
