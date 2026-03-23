//! SSE (Server-Sent Events) stream parser.

use bytes::Bytes;
use futures::{Stream, StreamExt};

use crate::error::InferenceError;
use crate::types::InferenceChunk;

/// Parse a byte stream of SSE events into typed inference chunks.
pub fn parse_sse_stream(
    byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<InferenceChunk, InferenceError>> + Send {
    let mut buffer = String::new();

    byte_stream
        .map(move |result| {
            let bytes = result.map_err(|e| InferenceError::Provider(e.to_string()))?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            let mut chunks = Vec::new();

            // SSE events are separated by double newlines
            while let Some(pos) = buffer.find("\n\n") {
                let event_text = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                // Skip empty events and keep-alives
                if event_text.trim().is_empty() || event_text.starts_with(":") {
                    continue;
                }

                // Extract data lines
                let mut data = String::new();
                for line in event_text.lines() {
                    if let Some(d) = line.strip_prefix("data: ") {
                        data.push_str(d);
                    }
                }

                if data.is_empty() || data == "[DONE]" {
                    continue;
                }

                match serde_json::from_str::<InferenceChunk>(&data) {
                    Ok(chunk) => chunks.push(Ok(chunk)),
                    Err(e) => chunks.push(Err(InferenceError::Provider(format!(
                        "SSE parse error: {e}, data: {data}"
                    )))),
                }
            }

            Ok(chunks)
        })
        .filter_map(
            |result: Result<Vec<Result<InferenceChunk, InferenceError>>, InferenceError>| {
                futures::future::ready(match result {
                    Ok(chunks) if chunks.is_empty() => None,
                    Ok(chunks) => Some(futures::stream::iter(chunks)),
                    Err(e) => Some(futures::stream::iter(vec![Err(e)])),
                })
            },
        )
        .flatten()
}
