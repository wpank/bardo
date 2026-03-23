//! Client for the bardo-gateway batch API and real-time Messages API.
//!
//! Batch mode submits enrichment requests at 50% cost with async completion.
//! Real-time mode sends requests through the gateway's `/v1/messages` proxy.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Default timeout for waiting on batch results: 5 minutes.
const DEFAULT_BATCH_TIMEOUT_SECS: u64 = 300;
/// Poll interval when waiting for batch results.
const POLL_INTERVAL_SECS: u64 = 5;

/// Client for the bardo-gateway batch API.
pub struct BatchClient {
    url: String,
    key: String,
    http: reqwest::Client,
}

impl BatchClient {
    /// Create a new batch client pointing at the given gateway URL.
    pub fn new(url: &str, key: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            key: key.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Submit a single request for batch processing. Returns the item ID.
    pub async fn submit(
        &self,
        model: &str,
        system: &str,
        user: &str,
        max_tokens: u32,
    ) -> Result<String> {
        let body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "system": system,
            "messages": [
                {"role": "user", "content": user}
            ]
        });

        let resp = self
            .http
            .post(format!("{}/v1/batch/submit", self.url))
            .header("x-api-key", &self.key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("failed to submit batch request to gateway")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("batch submit returned {status}: {text}");
        }

        let parsed: Value = resp
            .json()
            .await
            .context("failed to parse batch submit response")?;

        parsed
            .get("batch_item_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .context("batch submit response missing batch_item_id")
    }

    /// Force-flush the queue. Returns the batch ID if items were flushed.
    pub async fn flush(&self) -> Result<Option<String>> {
        let resp = self
            .http
            .post(format!("{}/v1/batch/flush", self.url))
            .header("x-api-key", &self.key)
            .send()
            .await
            .context("failed to flush batch queue")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("batch flush returned {status}: {text}");
        }

        let parsed: Value = resp
            .json()
            .await
            .context("failed to parse flush response")?;

        Ok(parsed
            .get("batch_id")
            .and_then(|v| v.as_str())
            .map(String::from))
    }

    /// Poll for a result. Returns `None` if still processing.
    pub async fn poll_result(&self, item_id: &str) -> Result<Option<String>> {
        let resp = self
            .http
            .get(format!("{}/v1/batch/result/{}", self.url, item_id))
            .header("x-api-key", &self.key)
            .send()
            .await
            .context("failed to poll batch result")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("batch result poll returned {status}: {text}");
        }

        let parsed: Value = resp
            .json()
            .await
            .context("failed to parse batch result response")?;

        let status = parsed
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        if status == "completed" {
            // Extract text from the Anthropic Messages API response structure.
            let text = extract_text_from_result(&parsed);
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    /// Wait for a result with timeout. Flushes the queue first, then polls.
    pub async fn wait_for_result(
        &self,
        item_id: &str,
        timeout_secs: Option<u64>,
    ) -> Result<String> {
        let timeout = timeout_secs.unwrap_or(DEFAULT_BATCH_TIMEOUT_SECS);

        // Flush to ensure the request is submitted.
        let _ = self.flush().await;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout);
        let mut interval = tokio::time::interval(Duration::from_secs(POLL_INTERVAL_SECS));

        loop {
            interval.tick().await;

            if tokio::time::Instant::now() > deadline {
                bail!("batch result for {item_id} timed out after {timeout}s");
            }

            if let Some(result) = self.poll_result(item_id).await? {
                return Ok(result);
            }
        }
    }
}

/// Send a request through the gateway in real-time (no batch).
///
/// # Errors
///
/// Returns an error if the gateway is unreachable or returns an error status.
pub async fn call_gateway(
    url: &str,
    key: &str,
    model: &str,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String> {
    let base = url.trim_end_matches('/');
    let http = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "system": system,
        "messages": [
            {"role": "user", "content": user}
        ]
    });

    let resp = http
        .post(format!("{base}/v1/messages"))
        .header("x-api-key", key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .context("failed to send request to gateway")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("gateway returned {status}: {text}");
    }

    let parsed: Value = resp
        .json()
        .await
        .context("failed to parse gateway response")?;

    Ok(extract_text_from_messages_response(&parsed))
}

/// Extract text content from an Anthropic Messages API response.
fn extract_text_from_messages_response(response: &Value) -> String {
    response
        .get("content")
        .and_then(|c| c.as_array())
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|b| {
                    if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                        b.get("text").and_then(|t| t.as_str()).map(String::from)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// Extract text from a batch result envelope.
fn extract_text_from_result(envelope: &Value) -> String {
    // The batch result endpoint wraps the Messages response in a `result` field.
    if let Some(result) = envelope.get("result") {
        if let Some(inner) = result.get("result") {
            // Nested: {status: "completed", result: {result: {content: [...]}}}
            return extract_text_from_messages_response(inner);
        }
        return extract_text_from_messages_response(result);
    }
    extract_text_from_messages_response(envelope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_text_from_messages_api_response() {
        let resp = serde_json::json!({
            "content": [
                {"type": "text", "text": "hello world"}
            ]
        });
        assert_eq!(extract_text_from_messages_response(&resp), "hello world");
    }

    #[test]
    fn extract_text_handles_empty_content() {
        let resp = serde_json::json!({"content": []});
        assert_eq!(extract_text_from_messages_response(&resp), "");
    }

    #[test]
    fn extract_text_from_batch_result_envelope() {
        let envelope = serde_json::json!({
            "status": "completed",
            "result": {
                "result": {
                    "content": [
                        {"type": "text", "text": "batch response"}
                    ]
                }
            }
        });
        assert_eq!(extract_text_from_result(&envelope), "batch response");
    }
}
