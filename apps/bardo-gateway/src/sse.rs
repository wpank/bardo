//! SSE stream format translation between Anthropic and OpenAI event formats.
//!
//! Both providers use `text/event-stream` but with incompatible event structures:
//!
//! - **Anthropic**: `event: <type>` + `data: <json>` pairs, multiple event types
//!   per message (`message_start`, `content_block_delta`, `message_stop`, …)
//! - **OpenAI**: only `data: <json>` lines with `choices[0].delta.content`,
//!   terminated by `data: [DONE]`
//!
//! [`anthropic_stream_to_openai`] wraps an Anthropic byte stream and re-emits
//! OpenAI-format events (used by Cursor/Codex when the gateway routed to Anthropic).
//!
//! [`openai_stream_to_anthropic`] wraps an OpenAI byte stream and re-emits
//! Anthropic-format events (used by Claude Code when the gateway routed to OpenAI).

use std::collections::VecDeque;
use std::pin::Pin;

use bytes::Bytes;
use futures::{Stream, StreamExt, stream};
use serde_json::Value;

type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

// ── Anthropic → OpenAI ─────────────────────────────────────────────

/// State for the Anthropic→OpenAI stream transformer.
struct AnthToOai {
    inner: ByteStream,
    /// Partial line buffer for byte chunks that don't end on a newline.
    buf: String,
    /// Current SSE event type from the last `event:` line.
    current_event: String,
    /// Message ID extracted from `message_start` (used in every OpenAI chunk).
    msg_id: String,
    /// Model name passed in by the caller.
    model: String,
    /// Final `finish_reason` to set on the last chunk (extracted from `message_delta`).
    finish_reason: Option<String>,
    /// Set to true once we've emitted `data: [DONE]`.
    done: bool,
    /// Accumulated output bytes to emit this tick before pulling new inner bytes.
    pending: VecDeque<Bytes>,
}

/// Wrap a raw Anthropic SSE byte stream and re-emit OpenAI-format SSE events.
///
/// Cursor and Codex expect OpenAI SSE format. When the gateway routed their
/// `/v1/chat/completions` request to Anthropic, this transformer converts the
/// response stream before returning it to the client.
pub fn anthropic_stream_to_openai(stream: ByteStream, model: String) -> ByteStream {
    let state = AnthToOai {
        inner: stream,
        buf: String::new(),
        current_event: String::new(),
        msg_id: "msg_translated".into(),
        model,
        finish_reason: None,
        done: false,
        pending: VecDeque::new(),
    };

    Box::pin(stream::unfold(state, |mut s| async move {
        loop {
            // Drain any pre-built output first.
            if let Some(chunk) = s.pending.pop_front() {
                return Some((Ok(chunk), s));
            }
            if s.done {
                return None;
            }

            // Pull next bytes from inner stream.
            let bytes = match s.inner.next().await {
                None => {
                    s.done = true;
                    return None;
                }
                Some(Err(e)) => return Some((Err(e), s)),
                Some(Ok(b)) => b,
            };

            s.buf.push_str(&String::from_utf8_lossy(&bytes));

            // Process all complete lines in the buffer.
            while let Some(idx) = s.buf.find('\n') {
                let line = s.buf[..idx].trim_end_matches('\r').to_string();
                s.buf.drain(..=idx);

                if let Some(event_type) = line.strip_prefix("event: ") {
                    s.current_event = event_type.to_string();
                    continue;
                }

                let data = match line.strip_prefix("data: ") {
                    Some(d) => d,
                    None => continue,
                };

                let event: Value = match serde_json::from_str(data) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                match event.get("type").and_then(|t| t.as_str()) {
                    Some("message_start") => {
                        // Capture message id and model.
                        if let Some(id) = event
                            .get("message")
                            .and_then(|m| m.get("id"))
                            .and_then(|i| i.as_str())
                        {
                            s.msg_id = id.to_string();
                        }
                        if let Some(m) = event
                            .get("message")
                            .and_then(|m| m.get("model"))
                            .and_then(|m| m.as_str())
                        {
                            s.model = m.to_string();
                        }
                        // Emit the role-initialization chunk Cursor expects first.
                        let chunk = oai_chunk(&s.msg_id, &s.model, None, Some("assistant"), None);
                        s.pending.push_back(Bytes::from(chunk));
                    }
                    Some("content_block_delta") => {
                        if let Some(text) = event
                            .get("delta")
                            .filter(|d| {
                                d.get("type").and_then(|t| t.as_str()) == Some("text_delta")
                            })
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                        {
                            let chunk = oai_chunk(&s.msg_id, &s.model, Some(text), None, None);
                            s.pending.push_back(Bytes::from(chunk));
                        }
                    }
                    Some("message_delta") => {
                        let stop = event
                            .get("delta")
                            .and_then(|d| d.get("stop_reason"))
                            .and_then(|r| r.as_str())
                            .map(anthropic_stop_to_openai_finish);
                        s.finish_reason = stop;
                    }
                    Some("message_stop") => {
                        let reason = s.finish_reason.clone().unwrap_or_else(|| "stop".into());
                        let chunk = oai_chunk(&s.msg_id, &s.model, None, None, Some(&reason));
                        s.pending.push_back(Bytes::from(chunk));
                        s.pending.push_back(Bytes::from("data: [DONE]\n\n"));
                        s.done = true;
                    }
                    _ => {} // ping, content_block_start/stop — skip
                }
            }
        }
    }))
}

/// Build a single OpenAI SSE `data:` line.
///
/// - `content`: text to stream; `None` for the role-init chunk or the final chunk
/// - `role`: set only on the first chunk (`"assistant"`)
/// - `finish_reason`: set only on the last chunk
fn oai_chunk(
    id: &str,
    model: &str,
    content: Option<&str>,
    role: Option<&str>,
    finish_reason: Option<&str>,
) -> String {
    let mut delta = serde_json::Map::new();
    if let Some(r) = role {
        delta.insert("role".into(), Value::String(r.into()));
        delta.insert("content".into(), Value::String(String::new()));
    } else if let Some(c) = content {
        delta.insert("content".into(), Value::String(c.into()));
    }

    let chunk = serde_json::json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": 0u64,
        "model": model,
        "choices": [{
            "index": 0,
            "delta": Value::Object(delta),
            "finish_reason": finish_reason,
        }],
    });

    format!("data: {}\n\n", chunk)
}

fn anthropic_stop_to_openai_finish(s: &str) -> String {
    match s {
        "end_turn" => "stop",
        "max_tokens" => "length",
        "tool_use" => "tool_calls",
        _ => "stop",
    }
    .into()
}

// ── OpenAI → Anthropic ─────────────────────────────────────────────

/// State for the OpenAI→Anthropic stream transformer.
struct OaiToAnth {
    inner: ByteStream,
    buf: String,
    model: String,
    /// Completion id from the first OpenAI chunk.
    msg_id: String,
    /// Whether we've emitted the Anthropic preamble (message_start, content_block_start, ping).
    started: bool,
    /// Accumulated output_tokens from the final OpenAI usage field.
    output_tokens: u64,
    /// Input tokens from OpenAI's usage (prompt_tokens).
    input_tokens: u64,
    /// Cached input tokens from OpenAI's prompt_tokens_details.cached_tokens.
    cached_tokens: u64,
    /// Reasoning tokens from OpenAI's completion_tokens_details.reasoning_tokens.
    reasoning_tokens: u64,
    /// Final stop_reason extracted from the last OpenAI chunk.
    stop_reason: String,
    done: bool,
    pending: VecDeque<Bytes>,
}

/// Wrap a raw OpenAI SSE byte stream and re-emit Anthropic-format SSE events.
///
/// Claude Code expects Anthropic SSE format. When the gateway routed a
/// `/v1/messages` request to OpenAI, this transformer converts the response
/// stream before returning it to Claude Code.
pub fn openai_stream_to_anthropic(stream: ByteStream, model: String) -> ByteStream {
    let state = OaiToAnth {
        inner: stream,
        buf: String::new(),
        model,
        msg_id: "msg_oai".into(),
        started: false,
        output_tokens: 0,
        input_tokens: 0,
        cached_tokens: 0,
        reasoning_tokens: 0,
        stop_reason: "end_turn".into(),
        done: false,
        pending: VecDeque::new(),
    };

    Box::pin(stream::unfold(state, |mut s| async move {
        loop {
            if let Some(chunk) = s.pending.pop_front() {
                return Some((Ok(chunk), s));
            }
            if s.done {
                return None;
            }

            let bytes = match s.inner.next().await {
                None => {
                    s.done = true;
                    return None;
                }
                Some(Err(e)) => return Some((Err(e), s)),
                Some(Ok(b)) => b,
            };

            s.buf.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(idx) = s.buf.find('\n') {
                let line = s.buf[..idx].trim_end_matches('\r').to_string();
                s.buf.drain(..=idx);

                let data = match line.strip_prefix("data: ") {
                    Some(d) => d,
                    None => continue,
                };

                if data == "[DONE]" {
                    // Emit closing Anthropic sequence with full usage from OpenAI.
                    let stop = s.stop_reason.clone();
                    let out_toks = s.output_tokens;
                    let in_toks = s.input_tokens;
                    let cached = s.cached_tokens;
                    let reasoning = s.reasoning_tokens;
                    s.pending.push_back(anth_event(
                        "content_block_stop",
                        &serde_json::json!({"type":"content_block_stop","index":0}),
                    ));
                    // Include all OpenAI usage in the message_delta so the handler's
                    // streaming tap can extract input_tokens, cached_tokens, and
                    // reasoning_tokens alongside output_tokens.
                    s.pending.push_back(anth_event(
                        "message_delta",
                        &serde_json::json!({
                            "type": "message_delta",
                            "delta": {"stop_reason": stop, "stop_sequence": null},
                            "usage": {
                                "output_tokens": out_toks,
                                "input_tokens": in_toks,
                                "cached_tokens": cached,
                                "reasoning_tokens": reasoning
                            },
                        }),
                    ));
                    s.pending.push_back(anth_event(
                        "message_stop",
                        &serde_json::json!({"type":"message_stop"}),
                    ));
                    s.done = true;
                    continue;
                }

                let event: Value = match serde_json::from_str(data) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                // Extract fields from the OpenAI chunk.
                let choice = event
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first());

                let delta = choice.and_then(|c| c.get("delta"));
                let role = delta.and_then(|d| d.get("role")).and_then(|r| r.as_str());
                let content = delta
                    .and_then(|d| d.get("content"))
                    .and_then(|c| c.as_str());
                let finish_reason = choice
                    .and_then(|c| c.get("finish_reason"))
                    .and_then(|r| r.as_str())
                    .filter(|r| !r.is_empty());

                // Capture completion id + model from any chunk.
                if let Some(id) = event.get("id").and_then(|i| i.as_str()) {
                    s.msg_id = id.to_string();
                }
                if let Some(m) = event.get("model").and_then(|m| m.as_str()) {
                    s.model = m.to_string();
                }

                // Capture usage if present (some providers send it on the last chunk).
                // OpenAI includes prompt_tokens, completion_tokens, and detailed breakdowns.
                if let Some(usage) = event.get("usage") {
                    if let Some(toks) = usage.get("completion_tokens").and_then(|t| t.as_u64()) {
                        s.output_tokens = toks;
                    }
                    if let Some(toks) = usage.get("prompt_tokens").and_then(|t| t.as_u64()) {
                        s.input_tokens = toks;
                    }
                    // OpenAI cached tokens (automatic prefix caching, 50% discount).
                    if let Some(toks) = usage
                        .get("prompt_tokens_details")
                        .and_then(|d| d.get("cached_tokens"))
                        .and_then(|t| t.as_u64())
                    {
                        s.cached_tokens = toks;
                    }
                    // o-series reasoning tokens.
                    if let Some(toks) = usage
                        .get("completion_tokens_details")
                        .and_then(|d| d.get("reasoning_tokens"))
                        .and_then(|t| t.as_u64())
                    {
                        s.reasoning_tokens = toks;
                    }
                }

                // Map finish_reason.
                if let Some(fr) = finish_reason {
                    s.stop_reason = openai_finish_to_anthropic_stop(fr).into();
                }

                // Role chunk → emit Anthropic preamble.
                if role == Some("assistant") && !s.started {
                    s.started = true;
                    let id = s.msg_id.clone();
                    let model = s.model.clone();
                    s.pending.push_back(anth_event(
                        "message_start",
                        &serde_json::json!({
                            "type": "message_start",
                            "message": {
                                "id": id,
                                "type": "message",
                                "role": "assistant",
                                "model": model,
                                "content": [],
                                "stop_reason": null,
                                "stop_sequence": null,
                                "usage": {"input_tokens": 0, "output_tokens": 0},
                            },
                        }),
                    ));
                    s.pending.push_back(anth_event(
                        "content_block_start",
                        &serde_json::json!({
                            "type": "content_block_start",
                            "index": 0,
                            "content_block": {"type": "text", "text": ""},
                        }),
                    ));
                    s.pending
                        .push_back(anth_event("ping", &serde_json::json!({"type": "ping"})));
                }

                // Content chunk → emit text delta.
                if let Some(text) = content {
                    if !text.is_empty() {
                        if !s.started {
                            // Guard: emit preamble if we somehow missed the role chunk.
                            s.started = true;
                            let id = s.msg_id.clone();
                            let model = s.model.clone();
                            s.pending.push_back(anth_event(
                                "message_start",
                                &serde_json::json!({
                                    "type": "message_start",
                                    "message": {
                                        "id": id, "type": "message", "role": "assistant",
                                        "model": model, "content": [], "stop_reason": null,
                                        "stop_sequence": null,
                                        "usage": {"input_tokens": 0, "output_tokens": 0},
                                    },
                                }),
                            ));
                            s.pending.push_back(anth_event(
                                "content_block_start",
                                &serde_json::json!({
                                    "type": "content_block_start", "index": 0,
                                    "content_block": {"type": "text", "text": ""},
                                }),
                            ));
                            s.pending
                                .push_back(anth_event("ping", &serde_json::json!({"type":"ping"})));
                        }
                        s.pending.push_back(anth_event(
                            "content_block_delta",
                            &serde_json::json!({
                                "type": "content_block_delta",
                                "index": 0,
                                "delta": {"type": "text_delta", "text": text},
                            }),
                        ));
                    }
                }
            }
        }
    }))
}

/// Build a single Anthropic SSE event (`event: TYPE\ndata: JSON\n\n`).
pub(crate) fn anth_event(event_type: &str, data: &Value) -> Bytes {
    Bytes::from(format!("event: {event_type}\ndata: {data}\n\n"))
}

fn openai_finish_to_anthropic_stop(s: &str) -> &str {
    match s {
        "stop" => "end_turn",
        "length" => "max_tokens",
        "tool_calls" => "tool_use",
        _ => "end_turn",
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Feed a sequence of byte strings through `anthropic_stream_to_openai` and
    /// collect all output as a single string.
    async fn run_anth_to_oai(lines: &[&str]) -> String {
        let bytes: Vec<Result<Bytes, reqwest::Error>> = lines
            .iter()
            .map(|s| Ok(Bytes::from(s.to_string())))
            .collect();
        let inner: ByteStream = Box::pin(stream::iter(bytes));
        let out_stream = anthropic_stream_to_openai(inner, "gpt-4o".into());
        let chunks: Vec<_> = out_stream.collect().await;
        let mut result = String::new();
        for c in chunks {
            result.push_str(&String::from_utf8_lossy(&c.unwrap()));
        }
        result
    }

    /// Feed a sequence of byte strings through `openai_stream_to_anthropic` and
    /// collect all output as a single string.
    async fn run_oai_to_anth(lines: &[&str]) -> String {
        let bytes: Vec<Result<Bytes, reqwest::Error>> = lines
            .iter()
            .map(|s| Ok(Bytes::from(s.to_string())))
            .collect();
        let inner: ByteStream = Box::pin(stream::iter(bytes));
        let out_stream = openai_stream_to_anthropic(inner, "claude-sonnet-4".into());
        let chunks: Vec<_> = out_stream.collect().await;
        let mut result = String::new();
        for c in chunks {
            result.push_str(&String::from_utf8_lossy(&c.unwrap()));
        }
        result
    }

    #[tokio::test]
    async fn anthropic_delta_line_emits_openai_chunk() {
        let input = [
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n",
        ];
        let output = run_anth_to_oai(&input).await;
        assert!(output.contains("\"Hello\""), "output: {output}");
        assert!(output.contains("chat.completion.chunk"), "output: {output}");
    }

    #[tokio::test]
    async fn anthropic_message_stop_emits_done() {
        let input = [
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n",
        ];
        let output = run_anth_to_oai(&input).await;
        assert!(output.contains("[DONE]"), "output: {output}");
        assert!(output.contains("\"stop\""), "output: {output}");
    }

    #[tokio::test]
    async fn openai_content_line_emits_anthropic_delta() {
        let input = [
            "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n",
        ];
        let output = run_oai_to_anth(&input).await;
        assert!(output.contains("content_block_delta"), "output: {output}");
        assert!(output.contains("\"Hi\""), "output: {output}");
        assert!(output.contains("message_start"), "output: {output}");
    }

    #[tokio::test]
    async fn openai_done_emits_message_stop() {
        let input = [
            "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n",
        ];
        let output = run_oai_to_anth(&input).await;
        assert!(output.contains("message_stop"), "output: {output}");
        assert!(output.contains("end_turn"), "output: {output}");
    }

    #[tokio::test]
    async fn split_bytes_buffered_correctly() {
        // Simulate a line arriving across two separate byte chunks.
        let input = [
            "data: {\"type\":\"content_block_del",
            "ta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"X\"}}\n\n",
        ];
        let output = run_anth_to_oai(&input).await;
        assert!(output.contains("\"X\""), "output: {output}");
    }
}
