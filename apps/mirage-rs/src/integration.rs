//! Client and test-harness integration for local mirage instances.

#![allow(clippy::single_match_else)]

use std::{path::PathBuf, process::Stdio, time::Duration};

use alloy_primitives::{Address, B256, Bytes, hex};
use futures_util::stream::{self, BoxStream};
use futures_util::StreamExt;
use golem_core::config::GolemConfig;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{process::Child, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    MirageError, Result, TransactionRequest,
    fork::MirageStatus,
    resources::ResourceUsage,
    scenario::{RunMode, Scenario, ScenarioJob},
};

/// Connection config for a mirage sidecar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirageConfig {
    /// Base URL for the local JSON-RPC server.
    pub url: String,
    /// Per-request timeout.
    pub timeout: Duration,
    /// Retry attempts on transport errors.
    pub retry_attempts: u32,
    /// Initial retry backoff.
    pub retry_backoff: Duration,
}

impl MirageConfig {
    /// Derives client config from `golem.toml`.
    #[must_use]
    pub fn from_golem_config(config: &GolemConfig) -> Self {
        Self {
            url: format!(
                "{}:{}",
                config.mirage.url.trim_end_matches('/'),
                config.mirage.port
            ),
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_backoff: Duration::from_millis(500),
        }
    }

    /// Returns the default local development config.
    #[must_use]
    pub fn default_local() -> Self {
        Self {
            url: "http://127.0.0.1:8545".to_owned(),
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_backoff: Duration::from_millis(500),
        }
    }
}

/// Position helper request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionRequest {
    /// Position owner address.
    pub owner: Address,
    /// Protocol type string.
    pub protocol_type: String,
    /// Optional contract address.
    pub contract: Option<Address>,
    /// Addresses to include in the raw balance snapshot.
    pub token_addresses: Vec<Address>,
}

/// Position helper response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSnapshot {
    /// Requested owner address.
    pub owner: Address,
    /// Echoed protocol type.
    pub protocol_type: String,
    /// Raw payload for protocol-specific readers.
    pub data: serde_json::Value,
}

/// Event-source provenance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventSource {
    /// Emitted by a locally submitted transaction.
    LocalTx,
    /// Emitted while replaying upstream state.
    FollowerReplay,
}

/// Event-stream filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventFilter {
    /// Optional address filter.
    pub addresses: Option<Vec<Address>>,
    /// Optional topic filter.
    pub topics: Option<Vec<B256>>,
}

/// Event payload delivered to downstream consumers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirageEvent {
    /// Block number containing the event.
    pub block_number: u64,
    /// Transaction hash.
    pub tx_hash: B256,
    /// Log index within the receipt.
    pub log_index: u32,
    /// Contract that emitted the log.
    pub contract: Address,
    /// Event topics.
    pub topics: Vec<B256>,
    /// Raw log data.
    pub data: Bytes,
    /// Event provenance.
    pub source: EventSource,
    /// Optional decoded payload.
    pub decoded: Option<serde_json::Value>,
}

/// Async JSON-RPC client for mirage.
#[derive(Debug, Clone)]
pub struct MirageClient {
    config: MirageConfig,
    inner: reqwest::Client,
}

impl MirageClient {
    /// Builds a new client.
    pub fn new(config: MirageConfig) -> Result<Self> {
        let inner = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self { config, inner })
    }

    /// Executes `eth_call`.
    pub async fn eth_call(&self, req: TransactionRequest) -> Result<Bytes> {
        let response: String = self
            .rpc_call("eth_call", serde_json::json!([req, "latest"]))
            .await?;
        parse_bytes_response(&response)
    }

    /// Executes `eth_sendTransaction`.
    pub async fn eth_send_transaction(&self, req: TransactionRequest) -> Result<B256> {
        self.rpc_call("eth_sendTransaction", serde_json::json!([req]))
            .await
    }

    /// Captures an `evm_snapshot`.
    pub async fn evm_snapshot(&self) -> Result<u64> {
        let raw: String = self.rpc_call("evm_snapshot", serde_json::json!([])).await?;
        parse_hex_u64(&raw)
    }

    /// Restores an `evm_revert` snapshot.
    pub async fn evm_revert(&self, id: u64) -> Result<bool> {
        self.rpc_call("evm_revert", serde_json::json!([format!("0x{id:x}")]))
            .await
    }

    /// Adds a contract to the watch list.
    pub async fn mirage_watch_contract(&self, addr: Address) -> Result<()> {
        let _: bool = self
            .rpc_call("mirage_watchContract", serde_json::json!([addr]))
            .await?;
        Ok(())
    }

    /// Reads a position helper snapshot.
    pub async fn mirage_get_position(&self, req: PositionRequest) -> Result<PositionSnapshot> {
        self.rpc_call("mirage_getPosition", serde_json::json!([req]))
            .await
    }

    /// Reads the current status snapshot.
    pub async fn mirage_status(&self) -> Result<MirageStatus> {
        self.rpc_call("mirage_status", serde_json::json!([])).await
    }

    /// Reads current resource usage.
    pub async fn mirage_get_resource_usage(&self) -> Result<ResourceUsage> {
        self.rpc_call("mirage_getResourceUsage", serde_json::json!([]))
            .await
    }

    /// Creates a new scenario set.
    pub async fn mirage_begin_scenario_set(&self, baseline: &str) -> Result<String> {
        self.rpc_call("mirage_beginScenarioSet", serde_json::json!([baseline]))
            .await
    }

    /// Adds a scenario to an existing set.
    pub async fn mirage_define_scenario(
        &self,
        set_id: &str,
        scenario: &Scenario,
    ) -> Result<String> {
        self.rpc_call(
            "mirage_defineScenario",
            serde_json::json!([set_id, scenario]),
        )
        .await
    }

    /// Starts scenario execution.
    pub async fn mirage_run_scenario_set(&self, set_id: &str, mode: RunMode) -> Result<String> {
        self.rpc_call("mirage_runScenarioSet", serde_json::json!([set_id, mode]))
            .await
    }

    /// Polls scenario results.
    pub async fn mirage_get_scenario_results(&self, job_id: &str) -> Result<ScenarioJob> {
        self.rpc_call("mirage_getScenarioResults", serde_json::json!([job_id]))
            .await
    }

    /// Waits until the sidecar reports `ready`.
    pub async fn wait_ready(&self, timeout: Duration) -> Result<()> {
        let started = Instant::now();
        loop {
            match self.mirage_status().await {
                Ok(status) if status.status == "ready" => return Ok(()),
                Ok(_) | Err(_) if started.elapsed() < timeout => {
                    sleep(Duration::from_millis(500)).await;
                }
                Ok(status) => {
                    return Err(MirageError::Timeout(format!(
                        "status remained {}",
                        status.status
                    )));
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// Returns a stream of currently known local events matching the filter.
    pub async fn subscribe_events(
        &self,
        filter: EventFilter,
    ) -> Result<BoxStream<'static, MirageEvent>> {
        let stream_id: String = self
            .rpc_call("mirage_subscribeEvents", serde_json::json!([filter]))
            .await?;
        let mut url = reqwest::Url::parse(&self.config.url)
            .map_err(|error| MirageError::Unsupported(format!("invalid mirage url: {error}")))?;
        let scheme = match url.scheme() {
            "https" => "wss",
            _ => "ws",
        };
        url.set_scheme(scheme)
            .map_err(|()| MirageError::Unsupported("failed to convert mirage url to ws".to_owned()))?;
        url.set_path(&format!("/events/{stream_id}"));

        let (socket, _) = connect_async(url.as_str())
            .await
            .map_err(|error| MirageError::Unsupported(format!("websocket connect failed: {error}")))?;
        let (_, read) = socket.split();
        let events = stream::unfold(read, |mut read| async move {
            while let Some(message) = read.next().await {
                let message = match message {
                    Ok(message) => message,
                    Err(error) => {
                        tracing::warn!("event stream closed: {error}");
                        return None;
                    }
                };
                let payload = match message {
                    Message::Text(text) => text.to_string(),
                    Message::Binary(bytes) => match String::from_utf8(bytes.to_vec()) {
                        Ok(text) => text,
                        Err(error) => {
                            tracing::warn!("event stream delivered invalid utf8: {error}");
                            continue;
                        }
                    },
                    Message::Close(_) => return None,
                    _ => continue,
                };
                match serde_json::from_str::<MirageEvent>(&payload) {
                    Ok(event) => return Some((event, read)),
                    Err(error) => {
                        tracing::warn!("failed to decode mirage event: {error}");
                        continue;
                    }
                }
            }
            None
        });
        Ok(Box::pin(events))
    }

    /// Sends a shutdown request to the sidecar.
    pub async fn shutdown(&self) -> Result<bool> {
        self.rpc_call("mirage_shutdown", serde_json::json!([]))
            .await
    }

    async fn rpc_call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let mut backoff = self.config.retry_backoff;
        let mut last_error = None;
        for _attempt in 0..=self.config.retry_attempts {
            match self.rpc_call_once(method, params.clone()).await {
                Ok(value) => return Ok(value),
                Err(error) => {
                    last_error = Some(error);
                    sleep(backoff).await;
                    backoff = backoff.saturating_mul(2);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| MirageError::Timeout(method.to_owned())))
    }

    async fn rpc_call_once<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let response = self
            .inner
            .post(&self.config.url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params,
            }))
            .send()
            .await?;
        let value = response.json::<serde_json::Value>().await?;
        if let Some(error) = value.get("error") {
            return Err(MirageError::Unsupported(error.to_string()));
        }
        let result = value
            .get("result")
            .cloned()
            .ok_or_else(|| MirageError::Unsupported(format!("missing result for {method}")))?;
        serde_json::from_value(result).map_err(Into::into)
    }
}

/// Spawned mirage process managed by tests.
#[derive(Debug)]
pub struct MirageTestInstance {
    process: Child,
    port: u16,
    pid_file: PathBuf,
}

impl MirageTestInstance {
    /// Returns the connection config for this process.
    #[must_use]
    pub fn config(&self) -> MirageConfig {
        MirageConfig {
            url: format!("http://127.0.0.1:{}", self.port),
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            retry_backoff: Duration::from_millis(250),
        }
    }

    /// Shuts down the child process.
    pub async fn shutdown(&mut self) -> Result<()> {
        let client = MirageClient::new(self.config())?;
        let _ = client.shutdown().await;
        let _ = tokio::time::timeout(Duration::from_secs(5), self.process.wait()).await;
        if self.process.try_wait()?.is_none() {
            self.process.kill().await?;
        }
        let _ = tokio::fs::remove_file(&self.pid_file).await;
        Ok(())
    }
}

/// Spawns a new test instance and waits for readiness.
pub async fn spawn_mirage_test_instance(
    rpc_url: Option<&str>,
    port: Option<u16>,
) -> Result<MirageTestInstance> {
    let port = port.unwrap_or(18_545);
    let pid_file = PathBuf::from(format!("/tmp/mirage-{port}.pid"));
    let executable = match std::env::var("CARGO_BIN_EXE_mirage-rs") {
        Ok(path) => path,
        Err(_) => {
            let current = std::env::current_exe()?;
            let target_dir = current
                .parent()
                .and_then(|path| path.parent())
                .ok_or_else(|| {
                    MirageError::Unsupported(
                        "workspace target/debug directory not found".to_owned(),
                    )
                })?;
            target_dir.join("mirage-rs").to_string_lossy().into_owned()
        }
    };

    let mut command = tokio::process::Command::new(executable);
    command
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(url) = rpc_url {
        command.arg("--rpc-url").arg(url);
    }
    let process = command.spawn()?;
    let instance = MirageTestInstance {
        process,
        port,
        pid_file,
    };
    let client = MirageClient::new(instance.config())?;
    client.wait_ready(Duration::from_secs(10)).await?;
    Ok(instance)
}

fn parse_hex_u64(value: &str) -> Result<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16)
        .map_err(|error| MirageError::InvalidParams(format!("invalid hex quantity: {error}")))
}

fn parse_bytes_response(value: &str) -> Result<Bytes> {
    let bytes = hex::decode(value.trim_start_matches("0x"))
        .map_err(|error| MirageError::InvalidParams(format!("invalid bytes response: {error}")))?;
    Ok(Bytes::from(bytes))
}

use std::time::Instant;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use alloy_primitives::{Bytes, U256, address, hex};
    use futures_util::StreamExt;

    use crate::{
        TransactionRequest,
        integration::{EventFilter, MirageClient},
        rpc::spawn_rpc_server_for_tests,
    };

    #[tokio::test]
    async fn mirage_client_wait_ready() {
        let (url, handle) = match spawn_rpc_server_for_tests().await {
            Ok(value) => value,
            Err(error) => panic!("server starts: {error}"),
        };
        let client = MirageClient::new(crate::integration::MirageConfig {
            url,
            timeout: Duration::from_secs(5),
            retry_attempts: 1,
            retry_backoff: Duration::from_millis(50),
        })
        .unwrap_or_else(|error| panic!("client initializes: {error}"));

        client
            .wait_ready(Duration::from_secs(2))
            .await
            .unwrap_or_else(|error| panic!("server becomes ready: {error}"));
        handle
            .stop()
            .unwrap_or_else(|error| panic!("server stops cleanly: {error}"));
    }

    #[tokio::test]
    async fn mirage_client_subscribe_events_streams_live_logs() {
        let (url, handle) = match spawn_rpc_server_for_tests().await {
            Ok(value) => value,
            Err(error) => panic!("server starts: {error}"),
        };
        let client = MirageClient::new(crate::integration::MirageConfig {
            url,
            timeout: Duration::from_secs(5),
            retry_attempts: 1,
            retry_backoff: Duration::from_millis(50),
        })
        .unwrap_or_else(|error| panic!("client initializes: {error}"));

        let token = address!("0x3300000000000000000000000000000000000001");
        let owner = address!("0x3300000000000000000000000000000000000002");

        client
            .rpc_call::<bool>(
                "mirage_setCode",
                serde_json::json!([token, "0x6001600055"]),
            )
            .await
            .unwrap_or_else(|error| panic!("token code set: {error}"));
        client
            .rpc_call::<bool>(
                "mirage_mintERC20",
                serde_json::json!([token, owner, "0x10"]),
            )
            .await
            .unwrap_or_else(|error| panic!("token minted: {error}"));

        let mut events = client
            .subscribe_events(EventFilter {
                addresses: Some(vec![token]),
                topics: None,
            })
            .await
            .unwrap_or_else(|error| panic!("subscribe events: {error}"));

        let calldata = Bytes::from(hex::decode(
            "a9059cbb00000000000000000000000033000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000005",
        )
        .unwrap_or_else(|error| panic!("calldata bytes: {error}")));
        let request = TransactionRequest {
            from: Some(owner),
            to: Some(token),
            gas: Some(100_000),
            value: Some(U256::ZERO),
            data: Some(calldata),
            gas_price: None,
            nonce: None,
            chain_id: None,
        };
        client
            .eth_send_transaction(request)
            .await
            .unwrap_or_else(|error| panic!("send tx: {error}"));

        let event = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .unwrap_or_else(|error| panic!("event timeout: {error}"))
            .unwrap_or_else(|| panic!("event stream closed"));
        assert_eq!(event.contract, token);
        assert_eq!(event.tx_hash != alloy_primitives::B256::ZERO, true);
        assert_eq!(event.topics.len(), 3);

        handle
            .stop()
            .unwrap_or_else(|error| panic!("server stops cleanly: {error}"));
    }
}
