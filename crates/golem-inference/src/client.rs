//! Gateway client implementing the `InferenceClient` trait.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::error::InferenceError;
use crate::types::{InferenceChunk, InferenceMeta, InferenceRequest, InferenceResponse};

/// Async trait for inference dispatch.
#[async_trait]
pub trait InferenceClient: Send + Sync {
    /// Send a non-streaming inference request.
    async fn complete(
        &self,
        request: &InferenceRequest,
        meta: &InferenceMeta,
    ) -> Result<InferenceResponse, InferenceError>;

    /// Send a streaming inference request, returning an SSE chunk stream.
    async fn stream(
        &self,
        request: &InferenceRequest,
        meta: &InferenceMeta,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<InferenceChunk, InferenceError>> + Send>>,
        InferenceError,
    >;
}

/// HTTP client that talks to a bardo-gateway instance.
pub struct GatewayClient {
    http: reqwest::Client,
    gateway_url: String,
    api_key: String,
}

impl GatewayClient {
    /// Create a new gateway client.
    pub fn new(gateway_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            gateway_url: gateway_url.into(),
            api_key: api_key.into(),
        }
    }

    /// Create from environment variables.
    ///
    /// - `BARDO_GATEWAY_URL` (default: `http://127.0.0.1:4000`)
    /// - `BARDO_GATEWAY_API_KEY`
    pub fn from_env() -> Result<Self, InferenceError> {
        let url =
            std::env::var("BARDO_GATEWAY_URL").unwrap_or_else(|_| "http://127.0.0.1:4000".into());
        let key = std::env::var("BARDO_GATEWAY_API_KEY")
            .map_err(|_| InferenceError::Validation("BARDO_GATEWAY_API_KEY not set".into()))?;
        Ok(Self::new(url, key))
    }
}

#[async_trait]
impl InferenceClient for GatewayClient {
    async fn complete(
        &self,
        request: &InferenceRequest,
        meta: &InferenceMeta,
    ) -> Result<InferenceResponse, InferenceError> {
        let resp = self
            .http
            .post(format!("{}/v1/messages", self.gateway_url))
            .header("X-Api-Key", &self.api_key)
            .header("X-Golem-Id", meta.golem_id.to_string())
            .header("X-Tier", u8::from(meta.tier).to_string())
            .header("X-Vitality", meta.vitality.to_string())
            .header("X-Request-Id", meta.request_id.to_string())
            .json(request)
            .send()
            .await
            .map_err(|e| InferenceError::Provider(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(InferenceError::Unauthorized);
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(InferenceError::Provider(body));
        }

        resp.json::<InferenceResponse>()
            .await
            .map_err(|e| InferenceError::Provider(format!("response parse error: {e}")))
    }

    async fn stream(
        &self,
        request: &InferenceRequest,
        meta: &InferenceMeta,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<InferenceChunk, InferenceError>> + Send>>,
        InferenceError,
    > {
        let mut stream_req = request.clone();
        stream_req.stream = true;

        let resp = self
            .http
            .post(format!("{}/v1/messages", self.gateway_url))
            .header("X-Api-Key", &self.api_key)
            .header("X-Golem-Id", meta.golem_id.to_string())
            .header("X-Tier", u8::from(meta.tier).to_string())
            .header("X-Vitality", meta.vitality.to_string())
            .header("X-Request-Id", meta.request_id.to_string())
            .json(&stream_req)
            .send()
            .await
            .map_err(|e| InferenceError::Provider(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(InferenceError::Unauthorized);
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(InferenceError::Provider(body));
        }

        let byte_stream = resp.bytes_stream();
        let chunk_stream = crate::sse::parse_sse_stream(byte_stream);
        Ok(Box::pin(chunk_stream))
    }
}
