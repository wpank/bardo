//! Transaction replay and speculative execution helpers.

#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use alloy_primitives::{Address, B256, Bytes, U256, hex};
use futures_util::StreamExt;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

use crate::{
    Bytecode, ExecutionResult, MirageError, Result, TransactionRequest,
    fork::{Classification, DiffClassifier, EvmExecutor, ForkState, MirageState, WatchEntry, WatchSource},
    provider::UpstreamRpc,
};

/// Canonical log entry captured in a state diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Contract address that emitted the log.
    pub address: Address,
    /// Topics carried by the log.
    pub topics: Vec<B256>,
    /// Raw log data.
    pub data: Bytes,
    /// Log index within the transaction receipt.
    pub log_index: u32,
}

/// Canonical account-level state diff.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDiff {
    /// Whether account-level fields changed.
    pub info_changed: bool,
    /// New balance, if written.
    pub new_balance: Option<U256>,
    /// New nonce, if written.
    pub new_nonce: Option<u64>,
    /// New bytecode, if written.
    pub new_code: Option<Bytecode>,
    /// Storage writes by slot.
    pub storage_written: HashMap<U256, U256>,
    /// Storage reads by slot.
    pub storage_read: HashSet<U256>,
}

/// Canonical transaction state diff exported to downstream plans.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDiff {
    /// Per-account changes.
    pub accounts: HashMap<Address, AccountDiff>,
    /// Logs emitted during execution.
    pub logs: Vec<LogEntry>,
    /// Gas used.
    pub gas_used: u64,
    /// Success flag.
    pub success: bool,
    /// Output bytes.
    pub output: Bytes,
}

impl StateDiff {
    /// Creates a successful diff shell.
    #[must_use]
    pub fn success(gas_used: u64, output: Bytes) -> Self {
        Self {
            gas_used,
            success: true,
            output,
            ..Self::default()
        }
    }
}

/// Configuration for the targeted follower.
#[derive(Debug, Clone)]
pub struct FollowerConfig {
    /// WebSocket URL used for subscriptions.
    pub ws_url: String,
    /// HTTP URL used for block/transaction fetches.
    pub http_url: String,
    /// Maximum replay wall-clock time budget per block.
    pub block_budget: std::time::Duration,
    /// Optional manual address filter.
    pub filter_addresses: Option<Vec<Address>>,
    /// Optional manual selector filter.
    pub filter_selectors: Option<Vec<[u8; 4]>>,
}

/// Background follower that replays upstream heads into the local fork state.
#[derive(Debug)]
pub struct TargetedFollower {
    upstream: Arc<UpstreamRpc>,
    state: Arc<RwLock<MirageState>>,
    classifier: DiffClassifier,
    config: FollowerConfig,
    last_block_number: u64,
}

impl TargetedFollower {
    /// Creates a new targeted follower.
    #[must_use]
    pub fn new(
        upstream: Arc<UpstreamRpc>,
        mirage: &crate::fork::MirageFork,
        classifier: DiffClassifier,
        config: FollowerConfig,
    ) -> Self {
        let last_block_number = upstream.get_block_number().unwrap_or_default();
        Self {
            upstream,
            state: mirage.state(),
            classifier,
            config,
            last_block_number,
        }
    }

    /// Processes the next available upstream block.
    pub(crate) fn tick_once(&mut self) -> Result<()> {
        let head = self.upstream.get_block_number()?;
        self.replay_to_head(head);
        Ok(())
    }

    /// Runs the follower until shutdown or upstream stream exhaustion.
    /// Runs the follower until shutdown or upstream stream exhaustion.
    pub async fn run(
        mut self,
        mut shutdown: broadcast::Receiver<()>,
    ) -> Result<()> {
        if self.upstream.has_ws() {
            let mut heads = self.upstream.subscribe_new_heads().await?;
            loop {
                tokio::select! {
                    _ = shutdown.recv() => break,
                    next_head = heads.next() => {
                        match next_head {
                            Some(Ok(head)) => self.replay_to_head(head),
                            Some(Err(error)) => return Err(error),
                            None => break,
                        }
                    }
                }
            }
            return Ok(());
        }

        loop {
            tokio::select! {
                _ = shutdown.recv() => break,
                result = std::future::ready(self.tick_once()) => {
                    result?;
                    tokio::time::sleep(self.config.block_budget.min(Duration::from_secs(1))).await;
                }
            }
        }

        Ok(())
    }

    fn replay_to_head(&mut self, head: u64) {
        if head <= self.last_block_number {
            return;
        }

        for number in self.last_block_number.saturating_add(1)..=head {
            if let Err(error) = self.replay_block(number) {
                tracing::warn!("targeted replay failed for block {number}: {error}");
            }
        }
        self.last_block_number = head;
    }

    fn replay_block(&self, number: u64) -> Result<()> {
        let block = self
            .upstream
            .get_block_by_number(crate::provider::BlockTag::Number(number), true)?
            .ok_or_else(|| MirageError::Upstream(format!("missing upstream block {number}")))?;
        let transactions = block
            .get("transactions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for tx_json in transactions {
            if !self.matches_filters(&tx_json)? {
                continue;
            }

            let tx_hash = parse_b256_value(
                tx_json
                    .get("hash")
                    .ok_or_else(|| MirageError::Upstream("missing tx hash".to_owned()))?,
            )?;
            let tx_replay = TxReplay { tx_hash };
            let mut fork = { self.state.read().fork.clone() };
            let snapshot = fork.snapshot();
            let replayed = match tx_replay.execute(&self.upstream, &mut fork) {
                Ok(result) => result,
                Err(error) => {
                    let _ = fork.revert(snapshot);
                    tracing::warn!("replay failed for {tx_hash}: {error}");
                    continue;
                }
            };
            let (_execution, diff) = replayed;
            let touched_parent = tx_json
                .get("to")
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse::<Address>().ok())
                .or_else(|| tx_json.get("from").and_then(|value| value.as_str()).and_then(|value| value.parse::<Address>().ok()));
            self.apply_contagion_watchers(&mut fork, &diff, number, touched_parent)?;
            self.classifier.apply(&mut fork.db.dirty, &diff, number)?;
            {
                let mut state = self.state.write();
                state.fork.adopt_executed_branch(fork);
                state.last_request_at = Instant::now();
            }
        }

        Ok(())
    }

    fn matches_filters(&self, tx_json: &serde_json::Value) -> Result<bool> {
        let to = tx_json
            .get("to")
            .and_then(|value| value.as_str())
            .and_then(|value| value.parse::<Address>().ok());

        if let Some(filter_addresses) = &self.config.filter_addresses {
            if to.is_some_and(|address| filter_addresses.contains(&address)) {
                return Ok(true);
            }
        }

        if let Some(filter_selectors) = &self.config.filter_selectors {
            let data = tx_json
                .get("input")
                .or_else(|| tx_json.get("data"))
                .and_then(|value| value.as_str())
                .unwrap_or("0x");
            let bytes = hex::decode(data.trim_start_matches("0x"))
                .map_err(|error| MirageError::Upstream(format!("invalid tx calldata: {error}")))?;
            let selector = bytes.get(..4).and_then(|bytes| bytes.try_into().ok());
            if selector.is_some_and(|selector| filter_selectors.contains(&selector)) {
                return Ok(true);
            }
        }

        let state = self.state.read();
        Ok(to.is_some_and(|address| state.fork.db.dirty.watch_list.contains_key(&address)))
    }

    fn apply_contagion_watchers(
        &self,
        fork: &mut ForkState,
        diff: &StateDiff,
        block_number: u64,
        parent: Option<Address>,
    ) -> Result<()> {
        if !self.classifier.config().enable_contagion {
            return Ok(());
        }

        let parent = parent.unwrap_or_default();
        for (address, classification) in self.classifier.classify(diff) {
            if classification != Classification::Protocol {
                continue;
            }
            if fork.db.dirty.unwatch_list.contains(&address)
                || fork.db.dirty.watch_list.contains_key(&address)
            {
                continue;
            }
            if fork.db.dirty.watch_list.len() >= self.classifier.config().max_watched_contracts {
                return Err(MirageError::WatchListFull);
            }
            fork.db.dirty.watch_list.insert(
                address,
                WatchEntry {
                    source: if parent == Address::ZERO {
                        WatchSource::AutoClassified
                    } else {
                        WatchSource::Contagion { parent }
                    },
                    added_at_block: block_number,
                    initial_slot_count: diff
                        .accounts
                        .get(&address)
                        .map_or(0, |account| account.storage_written.len()),
                    replay_count: 0,
                },
            );
        }
        Ok(())
    }
}

/// Replay a transaction by hash.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TxReplay {
    /// Transaction hash to fetch and replay.
    pub tx_hash: B256,
}

impl TxReplay {
    /// Attempts to replay a specific transaction.
    pub(crate) fn execute(
        &self,
        upstream: &UpstreamRpc,
        state: &mut ForkState,
    ) -> Result<(ExecutionResult, StateDiff)> {
        let tx = upstream
            .get_transaction_by_hash(self.tx_hash)?
            .ok_or_else(|| {
                MirageError::Upstream(format!("transaction {} not found upstream", self.tx_hash))
            })?;
        let request: TransactionRequest = serde_json::from_value(tx)?;
        let from = request
            .from
            .ok_or_else(|| MirageError::InvalidParams("missing from".to_owned()))?;
        let to = request.to;
        let data = request.data.unwrap_or_default();
        let value = request.value.unwrap_or(U256::ZERO);
        let gas_limit = request.gas.unwrap_or(21_000);
        EvmExecutor::transact(state, from, to, data, value, gas_limit)
    }
}

/// Result of speculative execution.
#[derive(Debug, Clone)]
pub struct SpeculativeResult {
    /// Synthetic execution result.
    pub result: ExecutionResult,
    /// Canonical state diff.
    pub state_diff: StateDiff,
    /// Read set used for invalidation.
    pub read_set: HashSet<(Address, U256)>,
    /// Timestamp when the result was computed.
    pub computed_at: Instant,
}

/// Executes transactions against a cloned fork state without committing.
#[derive(Debug, Default)]
pub struct SpeculativeExecutor {
    cache: HashMap<(B256, u64), SpeculativeResult>,
}

impl SpeculativeExecutor {
    /// Executes a transaction request without mutating the base state.
    pub fn execute(
        &mut self,
        state: &ForkState,
        request: &TransactionRequest,
    ) -> Result<SpeculativeResult> {
        let from = request
            .from
            .ok_or_else(|| MirageError::InvalidParams("missing from".to_owned()))?;
        let to = request.to;
        let value = request.value.unwrap_or(U256::ZERO);
        let gas_limit = request.gas.unwrap_or(21_000);
        let data = request.data.clone().unwrap_or_default();
        let tx_hash = speculative_key(from, to, value, gas_limit, &data);
        let cache_key = (tx_hash, state.local_block_number);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        let mut cloned_state = state.clone();
        let (result, state_diff) =
            EvmExecutor::transact(&mut cloned_state, from, to, data, value, gas_limit)?;
        let read_set = state_diff
            .accounts
            .iter()
            .flat_map(|(address, diff)| diff.storage_read.iter().map(move |slot| (*address, *slot)))
            .collect();
        let speculative = SpeculativeResult {
            result,
            state_diff,
            read_set,
            computed_at: Instant::now(),
        };
        self.cache.insert(cache_key, speculative.clone());
        Ok(speculative)
    }

    /// Removes cached results whose read sets overlap the provided writes.
    pub fn invalidate_for_writes(&mut self, writes: &HashSet<(Address, U256)>) {
        self.cache
            .retain(|_, cached| cached.read_set.is_disjoint(writes));
    }

    /// Clears speculative results for a specific block number.
    pub fn invalidate_for_block(&mut self, block_number: u64) {
        self.cache.retain(|(_, cached_block), _| *cached_block != block_number);
    }
}

fn speculative_key(
    from: Address,
    to: Option<Address>,
    value: U256,
    gas: u64,
    data: &Bytes,
) -> B256 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(from.as_slice());
    bytes.extend_from_slice(to.unwrap_or_default().as_slice());
    bytes.extend_from_slice(&value.to_be_bytes::<32>());
    bytes.extend_from_slice(&gas.to_be_bytes());
    bytes.extend_from_slice(data.as_ref());
    alloy_primitives::keccak256(bytes)
}

fn parse_b256_value(value: &Value) -> Result<B256> {
    let text = value
        .as_str()
        .ok_or_else(|| MirageError::Upstream("expected B256 hex string".to_owned()))?;
    text.parse::<B256>()
        .map_err(|error| MirageError::Upstream(format!("invalid B256: {error}")))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, num::NonZeroUsize, sync::Arc, time::Duration};

    use alloy_primitives::{Bytes, U256, address};

    use super::{SpeculativeExecutor, StateDiff};
    use crate::{
        TransactionRequest,
        cow::{MultiVersionStore, VersionEntry},
        fork::{ForkState, HybridDB},
        provider::UpstreamRpc,
    };

    #[test]
    fn speculative_exec_no_state_commit() {
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let state = ForkState::new(db, 0, 1);
        let sender = address!("0x1000000000000000000000000000000000000001");
        let receiver = address!("0x1000000000000000000000000000000000000002");
        let request = TransactionRequest {
            from: Some(sender),
            to: Some(receiver),
            gas: Some(50_000),
            value: Some(U256::from(10_u64)),
            data: Some(Bytes::from_static(&[0xde, 0xad, 0xbe, 0xef])),
            gas_price: None,
            nonce: None,
            chain_id: None,
        };
        let mut executor = SpeculativeExecutor::default();

        let result = match executor.execute(&state, &request) {
            Ok(result) => result,
            Err(error) => panic!("speculation succeeds: {error}"),
        };

        assert!(result.state_diff.success);
        assert!(!result.read_set.is_empty());
        assert!(state.receipts.is_empty());

        let mut writes = HashSet::new();
        writes.extend(result.read_set.iter().copied());
        executor.invalidate_for_writes(&writes);
        assert!(executor.cache.is_empty());
    }

    #[test]
    fn state_diff_account_and_storage() {
        let mut diff = StateDiff::success(21_000, Bytes::default());
        let address = address!("0x2000000000000000000000000000000000000002");
        diff.accounts.insert(
            address,
            super::AccountDiff {
                info_changed: true,
                new_balance: Some(U256::from(1_u64)),
                new_nonce: Some(1),
                new_code: None,
                storage_written: std::iter::once((U256::from(1_u64), U256::from(2_u64))).collect(),
                storage_read: std::iter::once(U256::from(1_u64)).collect(),
            },
        );

        assert_eq!(diff.accounts[&address].storage_written.len(), 1);
        assert_eq!(diff.accounts[&address].storage_read.len(), 1);
    }

    #[test]
    fn block_stm_matches_sequential() {
        let address = address!("0x3000000000000000000000000000000000000003");
        let slot = U256::from(1_u64);
        let store = MultiVersionStore::default();
        store.record(
            address,
            slot,
            VersionEntry {
                tx_index: 0,
                value: U256::from(5_u64),
                incarnation: 0,
            },
        );
        store.record(
            address,
            slot,
            VersionEntry {
                tx_index: 1,
                value: U256::from(9_u64),
                incarnation: 0,
            },
        );

        let materialized = store.materialize();
        assert_eq!(materialized.get(&(address, slot)), Some(&U256::from(9_u64)));
    }
}
