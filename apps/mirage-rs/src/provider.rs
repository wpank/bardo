//! Upstream RPC access for lazy latest reads.

#![allow(
    clippy::needless_pass_by_value,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening
)]

use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use alloy_primitives::{Address, B256, Bytes, U256, hex, keccak256};
use futures_util::{SinkExt, Stream, StreamExt, stream};
use parking_lot::RwLock;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{AccountInfo, Bytecode, MirageError, Result};

const DEFAULT_MOCK_BALANCE: u64 = 1_000_000_000_000_000_000;

/// Upstream block selector for lazy reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockTag {
    /// Resolve reads against the latest upstream head.
    Latest,
    /// Resolve reads against a fixed block number.
    Number(u64),
}

impl BlockTag {
    fn as_json(self) -> Value {
        match self {
            Self::Latest => Value::String("latest".to_owned()),
            Self::Number(number) => Value::String(format!("0x{number:x}")),
        }
    }
}

/// Thread-safe upstream call counters.
#[derive(Debug, Default)]
pub struct UpstreamStats {
    /// Total upstream RPC calls.
    pub calls: u64,
    /// Total upstream RPC failures.
    pub errors: u64,
}

#[derive(Debug)]
struct RateLimitState {
    tokens: f64,
    last_refill_at: Instant,
}

/// Mock upstream state used by tests and offline mode.
#[derive(Debug, Clone, Default)]
pub struct MockUpstream {
    /// Current upstream block number.
    pub block_number: u64,
    /// Chain ID exposed by the mock.
    pub chain_id: u64,
    /// Artificial latency injected into mock reads for concurrency tests.
    pub delay: Duration,
    /// Account information keyed by address.
    pub accounts: HashMap<Address, AccountInfo>,
    /// Storage overrides keyed by `(address, slot)`.
    pub storage: HashMap<(Address, U256), U256>,
    /// Known block hashes keyed by block number.
    pub block_hashes: HashMap<u64, B256>,
}

/// Minimal upstream RPC adapter.
#[derive(Debug)]
pub struct UpstreamRpc {
    http_url: Option<String>,
    ws_url: Option<String>,
    chain_id: u64,
    client: Option<Client>,
    stats: Arc<RwLock<UpstreamStats>>,
    mock: Arc<RwLock<MockUpstream>>,
    code_by_hash: Arc<RwLock<HashMap<B256, Bytecode>>>,
    addresses_by_code_hash: Arc<RwLock<HashMap<B256, Address>>>,
    retry_attempts: u32,
    retry_backoff: Duration,
    requests_per_second: u32,
    burst: u32,
    rate_limit: Arc<RwLock<RateLimitState>>,
}

impl UpstreamRpc {
    /// Creates a new upstream adapter.
    #[must_use]
    pub fn new(http_url: Option<String>, ws_url: Option<String>, chain_id: u64) -> Self {
        Self::new_with_limits(http_url, ws_url, chain_id, 100, 200)
    }

    /// Creates a new upstream adapter with explicit throttling limits.
    #[must_use]
    pub fn new_with_limits(
        http_url: Option<String>,
        ws_url: Option<String>,
        chain_id: u64,
        requests_per_second: u32,
        burst: u32,
    ) -> Self {
        let client = http_url.as_ref().map(|_| Client::new());
        let burst = burst.max(1);
        Self {
            http_url,
            ws_url,
            chain_id,
            client,
            stats: Arc::new(RwLock::new(UpstreamStats::default())),
            mock: Arc::new(RwLock::new(MockUpstream {
                block_number: 0,
                chain_id,
                delay: Duration::ZERO,
                accounts: HashMap::new(),
                storage: HashMap::new(),
                block_hashes: HashMap::new(),
            })),
            code_by_hash: Arc::new(RwLock::new(HashMap::new())),
            addresses_by_code_hash: Arc::new(RwLock::new(HashMap::new())),
            retry_attempts: 2,
            retry_backoff: Duration::from_millis(100),
            requests_per_second,
            burst,
            rate_limit: Arc::new(RwLock::new(RateLimitState {
                tokens: f64::from(burst),
                last_refill_at: Instant::now(),
            })),
        }
    }

    /// Creates a mock-only upstream adapter.
    #[must_use]
    pub fn mock(chain_id: u64) -> Self {
        Self::new(None, None, chain_id)
    }

    /// Returns the configured chain ID.
    #[must_use]
    pub const fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Returns whether a WebSocket upstream URL was configured.
    #[must_use]
    pub const fn has_ws(&self) -> bool {
        self.ws_url.is_some()
    }

    /// Returns call counters.
    #[must_use]
    pub fn stats(&self) -> (u64, u64) {
        let stats = self.stats.read();
        (stats.calls, stats.errors)
    }

    /// Inserts mock account info for tests.
    pub fn set_mock_account(&self, address: Address, info: AccountInfo) {
        if let Some(code) = info.code.clone() {
            self.code_by_hash.write().insert(info.code_hash, code);
        }
        self.addresses_by_code_hash
            .write()
            .insert(info.code_hash, address);
        self.mock.write().accounts.insert(address, info);
    }

    /// Inserts mock storage for tests.
    pub fn set_mock_storage(&self, address: Address, slot: U256, value: U256) {
        self.mock.write().storage.insert((address, slot), value);
    }

    /// Injects artificial latency into mock reads for concurrency tests.
    pub fn set_mock_delay(&self, delay: Duration) {
        self.mock.write().delay = delay;
    }

    /// Returns the upstream block number.
    pub fn get_block_number(&self) -> Result<u64> {
        if self.http_url.is_none() {
            self.maybe_delay_mock();
            return Ok(self.mock.read().block_number);
        }
        let response = self.call("eth_blockNumber", json!([]))?;
        parse_hex_u64(&response)
    }

    /// Returns upstream account info.
    pub fn get_account_info(
        &self,
        address: Address,
        block: BlockTag,
    ) -> Result<Option<AccountInfo>> {
        if self.http_url.is_none() {
            self.maybe_delay_mock();
            let info = self
                .mock
                .read()
                .accounts
                .get(&address)
                .cloned()
                .unwrap_or_else(|| AccountInfo {
                    balance: U256::from(DEFAULT_MOCK_BALANCE),
                    nonce: 0,
                    code_hash: Bytecode::default().hash_slow(),
                    code: Some(Bytecode::default()),
                });
            return Ok(Some(info));
        }

        let balance =
            parse_hex_u256(&self.call("eth_getBalance", json!([address, block.as_json()]))?)?;
        let nonce = parse_hex_u64(
            &self.call("eth_getTransactionCount", json!([address, block.as_json()]))?,
        )?;
        let code_bytes =
            parse_bytes(&self.call("eth_getCode", json!([address, block.as_json()]))?)?;
        let code = Bytecode::new_raw(code_bytes);
        self.code_by_hash
            .write()
            .insert(code.hash_slow(), code.clone());
        self.addresses_by_code_hash
            .write()
            .insert(code.hash_slow(), address);
        Ok(Some(AccountInfo {
            balance,
            nonce,
            code_hash: code.hash_slow(),
            code: Some(code),
        }))
    }

    /// Returns upstream storage.
    pub fn get_storage_at(&self, address: Address, slot: U256, block: BlockTag) -> Result<U256> {
        if self.http_url.is_none() {
            self.maybe_delay_mock();
            return Ok(self
                .mock
                .read()
                .storage
                .get(&(address, slot))
                .copied()
                .unwrap_or(U256::ZERO));
        }

        parse_hex_u256(&self.call(
            "eth_getStorageAt",
            json!([address, format!("0x{slot:x}"), block.as_json()]),
        )?)
    }

    /// Returns a block payload with optional full transaction bodies.
    pub fn get_block_by_number(
        &self,
        block: BlockTag,
        full_transactions: bool,
    ) -> Result<Option<Value>> {
        if self.http_url.is_none() {
            self.maybe_delay_mock();
            let mock = self.mock.read();
            let number = match block {
                BlockTag::Latest => mock.block_number,
                BlockTag::Number(number) => number,
            };
            let hash = mock
                .block_hashes
                .get(&number)
                .copied()
                .unwrap_or_else(|| keccak256(number.to_be_bytes()));
            return Ok(Some(json!({
                "hash": hash,
                "number": format!("0x{number:x}"),
                "timestamp": "0x0",
                "transactions": Vec::<Value>::new(),
            })));
        }

        let response = self.call(
            "eth_getBlockByNumber",
            json!([block.as_json(), full_transactions]),
        )?;
        Ok(if response.is_null() {
            None
        } else {
            Some(response)
        })
    }

    /// Returns a transaction payload by hash.
    pub fn get_transaction_by_hash(&self, tx_hash: B256) -> Result<Option<Value>> {
        if self.http_url.is_none() {
            self.maybe_delay_mock();
            return Ok(None);
        }

        let response = self.call("eth_getTransactionByHash", json!([tx_hash]))?;
        Ok(if response.is_null() {
            None
        } else {
            Some(response)
        })
    }

    /// Returns upstream bytecode by code hash.
    pub fn get_code_by_hash(&self, code_hash: B256, block: BlockTag) -> Result<Bytecode> {
        if let Some(code) = self.code_by_hash.read().get(&code_hash).cloned() {
            return Ok(code);
        }

        if code_hash == Bytecode::default().hash_slow() {
            return Ok(Bytecode::default());
        }

        self.maybe_delay_mock();

        if let Some(code) = self
            .mock
            .read()
            .accounts
            .values()
            .find_map(|info| (info.code_hash == code_hash).then(|| info.code.clone()))
            .flatten()
        {
            self.code_by_hash.write().insert(code_hash, code.clone());
            return Ok(code);
        }

        if let Some(address) = self.addresses_by_code_hash.read().get(&code_hash).copied() {
            let code_bytes =
                parse_bytes(&self.call("eth_getCode", json!([address, block.as_json()]))?)?;
            let code = Bytecode::new_raw(code_bytes);
            if code.hash_slow() == code_hash {
                self.code_by_hash.write().insert(code_hash, code.clone());
                return Ok(code);
            }
        }

        Err(MirageError::Upstream(format!(
            "unknown bytecode hash {code_hash}"
        )))
    }

    /// Returns a deterministic or upstream-provided block hash.
    pub fn get_block_hash(&self, number: u64) -> Result<B256> {
        if self.http_url.is_none() {
            self.maybe_delay_mock();
            let hash = *self
                .mock
                .write()
                .block_hashes
                .entry(number)
                .or_insert_with(|| keccak256(number.to_be_bytes()));
            return Ok(hash);
        }

        let response = self.call(
            "eth_getBlockByNumber",
            json!([format!("0x{number:x}"), false]),
        )?;
        response
            .get("hash")
            .ok_or_else(|| MirageError::Upstream("missing block hash".to_owned()))
            .and_then(parse_b256)
    }

    /// Performs a basic connectivity check.
    pub fn health_check(&self) -> Result<u64> {
        self.get_block_number()
    }

    /// Subscribes to `newHeads` notifications from the WebSocket upstream.
    pub async fn subscribe_new_heads(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<u64>> + Send>>> {
        let ws_url = self.ws_url.as_ref().ok_or_else(|| {
            MirageError::Upstream("no WebSocket upstream URL configured".to_owned())
        })?;
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|error| MirageError::Upstream(format!("websocket connect failed: {error}")))?;
        let (mut write, mut read) = ws_stream.split();
        write
            .send(Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "eth_subscribe",
                    "params": ["newHeads"],
                })
                .to_string()
                .into(),
            ))
            .await
            .map_err(|error| {
                MirageError::Upstream(format!("websocket subscribe failed: {error}"))
            })?;

        let mut subscription_id = None;
        while let Some(message) = read.next().await {
            let message = message.map_err(|error| {
                MirageError::Upstream(format!("websocket read failed: {error}"))
            })?;
            let text = message.to_text().map_err(|error| {
                MirageError::Upstream(format!("websocket text frame failed: {error}"))
            })?;
            let value: Value = serde_json::from_str(text)?;
            if let Some(result) = value.get("result") {
                subscription_id = Some(result.clone());
                break;
            }
        }

        let subscription_id = subscription_id
            .ok_or_else(|| MirageError::Upstream("missing websocket subscription id".to_owned()))?;

        let heads = stream::unfold(
            (read, subscription_id),
            |(mut read, subscription_id)| async move {
                while let Some(message) = read.next().await {
                    let message = match message {
                        Ok(message) => message,
                        Err(error) => {
                            return Some((
                                Err(MirageError::Upstream(format!(
                                    "websocket read failed: {error}"
                                ))),
                                (read, subscription_id),
                            ));
                        }
                    };
                    let Ok(text) = message.to_text() else {
                        continue;
                    };
                    let Ok(value) = serde_json::from_str::<Value>(text) else {
                        continue;
                    };
                    let Some(params) = value.get("params") else {
                        continue;
                    };
                    if params.get("subscription") != Some(&subscription_id) {
                        continue;
                    }
                    let Some(result) = params.get("result") else {
                        continue;
                    };
                    let Some(number) = result.get("number") else {
                        continue;
                    };
                    match parse_hex_u64(number) {
                        Ok(number) => return Some((Ok(number), (read, subscription_id))),
                        Err(error) => return Some((Err(error), (read, subscription_id))),
                    }
                }
                None
            },
        );

        Ok(Box::pin(heads))
    }

    fn call(&self, method: &str, params: Value) -> Result<Value> {
        {
            let mut stats = self.stats.write();
            stats.calls = stats.calls.saturating_add(1);
        }

        let url = self
            .http_url
            .as_deref()
            .ok_or_else(|| MirageError::Upstream("no upstream URL configured".to_owned()))?;
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| MirageError::Upstream("no HTTP client configured".to_owned()))?;

        let mut attempt = 0;
        let mut backoff = self.retry_backoff;
        loop {
            self.enforce_rate_limit();
            let response = client
                .post(url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": method,
                    "params": params,
                }))
                .send();

            let value = match response {
                Ok(response) => response.json::<Value>()?,
                Err(_error) if attempt < self.retry_attempts => {
                    attempt = attempt.saturating_add(1);
                    thread::sleep(backoff);
                    backoff = backoff.saturating_mul(2);
                    continue;
                }
                Err(error) => {
                    let mut stats = self.stats.write();
                    stats.errors = stats.errors.saturating_add(1);
                    return Err(MirageError::Http(error));
                }
            };

            if let Some(error) = value.get("error") {
                let mut stats = self.stats.write();
                stats.errors = stats.errors.saturating_add(1);
                return Err(MirageError::Upstream(error.to_string()));
            }

            return value.get("result").cloned().ok_or_else(|| {
                MirageError::Upstream(format!("missing result for method {method}"))
            });
        }
    }

    fn enforce_rate_limit(&self) {
        if self.requests_per_second == 0 {
            return;
        }
        let mut rate_limit = self.rate_limit.write();
        let now = Instant::now();
        let elapsed = now.duration_since(rate_limit.last_refill_at).as_secs_f64();
        if elapsed > 0.0 {
            rate_limit.tokens = rate_limit
                .tokens
                .mul_add(1.0, elapsed * f64::from(self.requests_per_second))
                .min(f64::from(self.burst));
            rate_limit.last_refill_at = now;
        }
        if rate_limit.tokens >= 1.0 {
            rate_limit.tokens -= 1.0;
            return;
        }

        let deficit = 1.0 - rate_limit.tokens;
        let wait_seconds = deficit / f64::from(self.requests_per_second);
        drop(rate_limit);
        thread::sleep(Duration::from_secs_f64(wait_seconds));

        let mut rate_limit = self.rate_limit.write();
        rate_limit.tokens = 0.0;
        rate_limit.last_refill_at = Instant::now();
    }

    fn maybe_delay_mock(&self) {
        let delay = self.mock.read().delay;
        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }
}

fn parse_hex_u64(value: &Value) -> Result<u64> {
    let text = value
        .as_str()
        .ok_or_else(|| MirageError::Upstream("expected hex quantity string".to_owned()))?;
    u64::from_str_radix(text.trim_start_matches("0x"), 16)
        .map_err(|error| MirageError::Upstream(format!("invalid hex quantity: {error}")))
}

fn parse_hex_u256(value: &Value) -> Result<U256> {
    let text = value
        .as_str()
        .ok_or_else(|| MirageError::Upstream("expected hex quantity string".to_owned()))?;
    U256::from_str_radix(text.trim_start_matches("0x"), 16)
        .map_err(|error| MirageError::Upstream(format!("invalid U256 quantity: {error}")))
}

fn parse_b256(value: &Value) -> Result<B256> {
    let text = value
        .as_str()
        .ok_or_else(|| MirageError::Upstream("expected B256 hex string".to_owned()))?;
    text.parse::<B256>()
        .map_err(|error| MirageError::Upstream(format!("invalid B256: {error}")))
}

fn parse_bytes(value: &Value) -> Result<Bytes> {
    let text = value
        .as_str()
        .ok_or_else(|| MirageError::Upstream("expected bytes hex string".to_owned()))?;
    let bytes = hex::decode(text.trim_start_matches("0x"))
        .map_err(|error| MirageError::Upstream(format!("invalid bytecode: {error}")))?;
    Ok(Bytes::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_rpc_rate_limit_tokens_start_at_burst() {
        let upstream = UpstreamRpc::new_with_limits(None, None, 1, 100, 200);
        // Burst of 200 calls should complete nearly instantly
        let start = Instant::now();
        for _ in 0..200 {
            upstream.enforce_rate_limit();
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(200),
            "200 burst calls should complete quickly, took {elapsed:?}"
        );

        // After burst exhaustion, next call must wait (~10ms for 100 rps)
        let start = Instant::now();
        upstream.enforce_rate_limit();
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(5),
            "call after burst exhaustion should wait >=5ms, took {elapsed:?}"
        );
    }

    #[test]
    fn test_upstream_rpc_retry_backoff() {
        let upstream = UpstreamRpc::new(None, None, 1);
        assert_eq!(upstream.retry_attempts, 2);
        assert_eq!(upstream.retry_backoff, Duration::from_millis(100));
        assert_eq!(upstream.requests_per_second, 100);
        assert_eq!(upstream.burst, 200);
    }

    #[test]
    fn test_upstream_rpc_mock_get_block_number() {
        let upstream = UpstreamRpc::mock(1);
        upstream.mock.write().block_number = 42;
        assert_eq!(upstream.get_block_number().unwrap(), 42);
    }

    #[test]
    fn test_rate_limit_disabled_when_zero_rps() {
        let upstream = UpstreamRpc::new_with_limits(None, None, 1, 0, 0);
        let start = Instant::now();
        for _ in 0..500 {
            upstream.enforce_rate_limit();
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(50),
            "zero rps should skip rate limiting, took {elapsed:?}"
        );
    }

    #[test]
    fn test_mock_account_info() {
        let upstream = UpstreamRpc::mock(1);
        let address = Address::ZERO;
        let info = upstream
            .get_account_info(address, BlockTag::Latest)
            .unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.balance, U256::from(DEFAULT_MOCK_BALANCE));
    }

    #[test]
    fn test_mock_storage() {
        let upstream = UpstreamRpc::mock(1);
        let address = Address::ZERO;
        let slot = U256::from(1);
        upstream.set_mock_storage(address, slot, U256::from(99));
        let value = upstream
            .get_storage_at(address, slot, BlockTag::Latest)
            .unwrap();
        assert_eq!(value, U256::from(99));
    }

    #[test]
    fn test_block_tag_json() {
        assert_eq!(BlockTag::Latest.as_json(), Value::String("latest".into()));
        assert_eq!(
            BlockTag::Number(255).as_json(),
            Value::String("0xff".into())
        );
    }
}
