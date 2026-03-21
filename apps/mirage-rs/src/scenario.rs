//! Scenario definition and execution support.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use alloy_primitives::{Address, Bytes, U256};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    MirageError, TransactionRequest,
    fork::{
        ClassificationConfig, DiffClassifier, EvmExecutor, ForkState, MirageState,
        lock_state_writes,
    },
    replay::LogEntry,
};

/// Execution mode for a scenario set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunMode {
    /// Reuse one baseline and revert between scenarios.
    Sequential,
    /// Clone the baseline and run branches independently.
    Parallel,
}

/// Lifecycle state for a scenario set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScenarioSetStatus {
    /// Waiting for scenarios to be defined.
    Draft,
    /// Currently executing.
    Running,
    /// Execution finished successfully.
    Complete,
    /// Execution failed.
    Failed,
}

/// One scenario inside a scenario set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    /// Stable scenario ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Transactions to execute.
    pub transactions: Vec<TransactionRequest>,
    /// Addresses whose balances should be captured after execution.
    pub track_addresses: Vec<Address>,
    /// Optional gas ceiling.
    pub max_gas: Option<u64>,
    /// Scenario timeout.
    pub timeout: Duration,
    /// Optional post-run assertions used by fixture-driven scenario tests.
    #[serde(default)]
    pub assertions: ScenarioAssertions,
}

/// Post-run assertions for built-in scenario fixtures.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioAssertions {
    /// Addresses that should appear in the watch list after execution.
    #[serde(default)]
    #[serde(alias = "watch_list_contains")]
    pub watch_list_contains: Vec<Address>,
    /// Optional token balance lower-bound check.
    #[serde(alias = "token_balance_gte")]
    pub token_balance_gte: Option<TokenBalanceAssertion>,
}

/// Lower-bound token balance assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceAssertion {
    /// Token contract to inspect.
    pub token: Address,
    /// Account expected to hold the balance.
    pub address: Address,
    /// Lower bound for the balance.
    pub amount: U256,
}

#[derive(Debug, Deserialize)]
struct ScenarioFixture {
    scenario: ScenarioMeta,
    #[serde(default)]
    transactions: Vec<TransactionToml>,
    #[serde(default)]
    assertions: ScenarioAssertions,
    track: TrackConfig,
}

#[derive(Debug, Deserialize)]
struct ScenarioMeta {
    name: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Default, Deserialize)]
struct TrackConfig {
    #[serde(default)]
    addresses: Vec<Address>,
}

#[derive(Debug, Deserialize)]
struct TransactionToml {
    from: Address,
    to: Option<Address>,
    #[serde(default)]
    data: Bytes,
    #[serde(default)]
    value: U256,
    gas: u64,
}

/// Group of scenarios sharing one baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioSet {
    /// Stable set ID.
    pub id: String,
    /// Snapshot ID used as the baseline.
    pub baseline_snapshot_id: u64,
    /// Full fork baseline used for deterministic parallel/sequential branching.
    #[serde(skip_serializing, skip_deserializing, default)]
    pub baseline_fork: Option<ForkState>,
    /// Scenarios defined in the set.
    pub scenarios: Vec<Scenario>,
    /// Current set status.
    pub status: ScenarioSetStatus,
}

/// Result status for one scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScenarioStatus {
    /// Execution completed without revert.
    Success,
    /// Transaction reverted.
    Reverted,
    /// Execution exceeded the timeout.
    Timeout,
    /// Execution exceeded a gas bound.
    GasExceeded,
    /// Unhandled execution error.
    Error(String),
}

/// Result for one scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioResult {
    /// Scenario ID.
    pub scenario_id: String,
    /// Scenario name.
    pub name: String,
    /// Final scenario status.
    pub status: ScenarioStatus,
    /// Aggregate gas used.
    pub gas_used: u64,
    /// Wall-clock time in milliseconds.
    pub wall_time_ms: u64,
    /// Approximate peak memory during execution.
    pub peak_memory_bytes: u64,
    /// Net profit/loss across tracked balances, measured in wei.
    pub pnl_wei: i128,
    /// Number of accounts touched by the scenario.
    pub state_diff_accounts: usize,
    /// Number of storage slots written by the scenario.
    pub state_diff_storage_slots: usize,
    /// Final balances for tracked addresses.
    pub final_balances: HashMap<Address, U256>,
    /// Opaque protocol-specific state payload.
    pub position_state: serde_json::Value,
    /// Logs emitted by the scenario.
    pub logs: Vec<LogEntry>,
    /// Optional revert reason.
    pub revert_reason: Option<String>,
}

/// Background scenario job status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    /// Still running.
    Running,
    /// Finished successfully.
    Complete,
    /// Failed before completion.
    Failed,
}

/// Async execution tracking payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioJob {
    /// Job ID.
    pub job_id: String,
    /// Scenario set ID.
    pub set_id: String,
    /// Job status.
    pub status: JobStatus,
    /// Results once complete.
    pub results: Option<Vec<ScenarioResult>>,
    /// Aggregate runtime once complete.
    pub total_wall_time_ms: Option<u64>,
}

/// Sorts scenario results by the comparison contract used by the RPC API.
pub(crate) fn rank_scenario_results(mut results: Vec<ScenarioResult>) -> Vec<ScenarioResult> {
    results.sort_by(|left, right| {
        right
            .pnl_wei
            .cmp(&left.pnl_wei)
            .then_with(|| left.gas_used.cmp(&right.gas_used))
            .then_with(|| {
                left.state_diff_storage_slots
                    .cmp(&right.state_diff_storage_slots)
            })
            .then_with(|| left.state_diff_accounts.cmp(&right.state_diff_accounts))
            .then_with(|| left.wall_time_ms.cmp(&right.wall_time_ms))
    });
    results
}

/// Scenario runner using the shared fork state.
#[derive(Debug, Clone)]
pub struct ScenarioRunner {
    state: Arc<RwLock<MirageState>>,
}

impl ScenarioRunner {
    /// Creates a new runner.
    #[must_use]
    pub(crate) const fn new(state: Arc<RwLock<MirageState>>) -> Self {
        Self { state }
    }

    /// Executes each scenario with snapshot/revert between branches.
    pub async fn run_sequential(&self, set: &ScenarioSet) -> Vec<ScenarioResult> {
        let baseline = set
            .baseline_fork
            .clone()
            .unwrap_or_else(|| self.state.read().fork.clone());
        let set = set.clone();
        match tokio::task::spawn_blocking(move || execute_scenario_set(baseline, &set)).await {
            Ok((results, executed_fork)) => {
                let _writer_guard = lock_state_writes(&self.state).await;
                self.state.write().fork.adopt_executed_branch(executed_fork);
                results
            }
            Err(error) => vec![ScenarioResult {
                scenario_id: "join-error".to_owned(),
                name: "join-error".to_owned(),
                status: ScenarioStatus::Error(error.to_string()),
                gas_used: 0,
                wall_time_ms: 0,
                peak_memory_bytes: 0,
                pnl_wei: 0,
                state_diff_accounts: 0,
                state_diff_storage_slots: 0,
                final_balances: HashMap::new(),
                position_state: serde_json::json!({"mode": "raw-balances"}),
                logs: Vec::new(),
                revert_reason: Some(error.to_string()),
            }],
        }
    }

    /// Executes each scenario against a cloned baseline.
    pub async fn run_parallel(&self, set: &ScenarioSet) -> Vec<ScenarioResult> {
        let baseline = set
            .baseline_fork
            .clone()
            .unwrap_or_else(|| self.state.read().fork.clone());
        let tasks = set.scenarios.iter().map(|scenario| {
            let mut branch = baseline.clone();
            let scenario = scenario.clone();
            tokio::spawn(async move { execute_scenario(&mut branch, &scenario) })
        });
        let joined = futures::future::join_all(tasks).await;
        joined
            .into_iter()
            .map(|result| match result {
                Ok(value) => value,
                Err(error) => ScenarioResult {
                    scenario_id: "join-error".to_owned(),
                    name: "join-error".to_owned(),
                    status: ScenarioStatus::Error(error.to_string()),
                    gas_used: 0,
                    wall_time_ms: 0,
                    peak_memory_bytes: 0,
                    pnl_wei: 0,
                    state_diff_accounts: 0,
                    state_diff_storage_slots: 0,
                    final_balances: HashMap::new(),
                    position_state: serde_json::json!({"mode": "raw-balances"}),
                    logs: Vec::new(),
                    revert_reason: Some(error.to_string()),
                },
            })
            .collect()
    }
}

fn execute_scenario_set(
    mut fork: ForkState,
    set: &ScenarioSet,
) -> (Vec<ScenarioResult>, ForkState) {
    let mut baseline = if set.baseline_snapshot_id == 0 {
        fork.snapshot()
    } else {
        set.baseline_snapshot_id
    };
    let mut results = Vec::with_capacity(set.scenarios.len());

    for scenario in &set.scenarios {
        if let Some(baseline_fork) = &set.baseline_fork {
            fork = baseline_fork.clone();
        }
        let result = execute_scenario(&mut fork, scenario);
        results.push(result);
        if fork.revert(baseline).is_ok() {
            baseline = fork.snapshot();
        }
    }

    (results, fork)
}

impl Scenario {
    /// Parses a built-in TOML scenario fixture.
    pub fn from_toml(id: impl Into<String>, input: &str) -> Result<Self, MirageError> {
        let fixture: ScenarioFixture = toml::from_str(input)?;
        Ok(Self {
            id: id.into(),
            name: if fixture.scenario.description.is_empty() {
                fixture.scenario.name
            } else {
                format!(
                    "{} — {}",
                    fixture.scenario.name, fixture.scenario.description
                )
            },
            transactions: fixture
                .transactions
                .into_iter()
                .map(|tx| TransactionRequest {
                    from: Some(tx.from),
                    to: tx.to,
                    gas: Some(tx.gas),
                    value: Some(tx.value),
                    data: Some(tx.data),
                    gas_price: None,
                    nonce: None,
                    chain_id: None,
                })
                .collect(),
            track_addresses: fixture.track.addresses,
            max_gas: None,
            timeout: Duration::from_secs(15),
            assertions: fixture.assertions,
        })
    }

    /// Evaluates the fixture assertions against the current fork state.
    pub fn evaluate_assertions(
        &self,
        state: &mut crate::fork::ForkState,
    ) -> Result<(), MirageError> {
        for address in &self.assertions.watch_list_contains {
            if !state.db.dirty.watch_list.contains_key(address) {
                return Err(MirageError::Unsupported(format!(
                    "watch list missing {address}"
                )));
            }
        }
        if let Some(assertion) = &self.assertions.token_balance_gte {
            let balance = state
                .db
                .erc20_balance_of(assertion.token, assertion.address)?;
            if balance < assertion.amount {
                return Err(MirageError::Unsupported(format!(
                    "token balance {} below required {}",
                    balance, assertion.amount
                )));
            }
        }
        Ok(())
    }
}

fn execute_scenario(fork: &mut crate::fork::ForkState, scenario: &Scenario) -> ScenarioResult {
    let started = Instant::now();
    let mut gas_used = 0_u64;
    let mut logs = Vec::new();
    let initial_balances = scenario
        .track_addresses
        .iter()
        .filter_map(|address| {
            fork.db
                .basic(*address)
                .ok()
                .flatten()
                .map(|info| (*address, info.balance))
        })
        .collect::<HashMap<_, _>>();
    let mut state_diff_accounts = 0_usize;
    let mut state_diff_storage_slots = 0_usize;
    let mut status = ScenarioStatus::Success;
    let mut revert_reason = None;
    for request in &scenario.transactions {
        let Some(from) = request.from else {
            status = ScenarioStatus::Error("missing from".to_owned());
            break;
        };
        let to = request.to;
        let value = request.value.unwrap_or(U256::ZERO);
        let gas_limit = request.gas.unwrap_or(21_000);
        let input = request.data.clone().unwrap_or_default();

        match EvmExecutor::transact(fork, from, to, input, value, gas_limit) {
            Ok((_, diff)) => {
                let classifier = DiffClassifier::new(ClassificationConfig::default());
                let _ = classifier.apply(&mut fork.db.dirty, &diff, fork.local_block_number);
                state_diff_accounts = state_diff_accounts.saturating_add(diff.accounts.len());
                state_diff_storage_slots = state_diff_storage_slots.saturating_add(
                    diff.accounts
                        .values()
                        .map(|account| account.storage_written.len())
                        .sum::<usize>(),
                );
                gas_used = gas_used.saturating_add(diff.gas_used);
                logs.extend(diff.logs);
                if let Some(max_gas) = scenario.max_gas {
                    if gas_used > max_gas {
                        status = ScenarioStatus::GasExceeded;
                        break;
                    }
                }
            }
            Err(error) => {
                status = ScenarioStatus::Error(error.to_string());
                revert_reason = Some(error.to_string());
                break;
            }
        }
        if started.elapsed() > scenario.timeout {
            status = ScenarioStatus::Timeout;
            break;
        }
    }

    if matches!(status, ScenarioStatus::Success) {
        if let Err(error) = scenario.evaluate_assertions(fork) {
            status = ScenarioStatus::Error(error.to_string());
            revert_reason = Some(error.to_string());
        }
    }

    let final_balances = scenario
        .track_addresses
        .iter()
        .filter_map(|address| {
            fork.db
                .basic(*address)
                .ok()
                .flatten()
                .map(|info| (*address, info.balance))
        })
        .collect();
    let pnl_wei = balance_delta(&initial_balances, &final_balances);

    ScenarioResult {
        scenario_id: scenario.id.clone(),
        name: scenario.name.clone(),
        status,
        gas_used,
        wall_time_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
        peak_memory_bytes: crate::resources::ResourceModel::current_process_memory_bytes(),
        pnl_wei,
        state_diff_accounts,
        state_diff_storage_slots,
        final_balances,
        position_state: serde_json::json!({"mode": "raw-balances"}),
        logs,
        revert_reason,
    }
}

fn balance_delta(
    initial_balances: &HashMap<Address, U256>,
    final_balances: &HashMap<Address, U256>,
) -> i128 {
    let initial = initial_balances
        .values()
        .copied()
        .fold(U256::ZERO, U256::saturating_add);
    let final_total = final_balances
        .values()
        .copied()
        .fold(U256::ZERO, U256::saturating_add);
    u256_to_i128_saturating(final_total).saturating_sub(u256_to_i128_saturating(initial))
}

fn u256_to_i128_saturating(value: U256) -> i128 {
    let bytes = value.to_be_bytes::<32>();
    if bytes[..16].iter().any(|byte| *byte != 0) {
        return i128::MAX;
    }
    let mut lower = [0_u8; 16];
    lower.copy_from_slice(&bytes[16..]);
    let value = u128::from_be_bytes(lower);
    i128::try_from(value).unwrap_or(i128::MAX)
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, sync::Arc, time::Duration};

    use alloy_primitives::{U256, address};

    use super::{
        RunMode, Scenario, ScenarioAssertions, ScenarioResult, ScenarioRunner, ScenarioSet,
        ScenarioSetStatus, ScenarioStatus, rank_scenario_results,
    };
    use crate::{
        fork::{ForkState, HybridDB, MirageFork, simple_transaction},
        provider::UpstreamRpc,
        resources::{MirageMode, Profile, ResourceModel},
    };

    #[tokio::test]
    async fn test_scenario_runner_basic() {
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let runner = ScenarioRunner::new(mirage.state());
        let sender = address!("0x1000000000000000000000000000000000000001");
        let receiver = address!("0x1000000000000000000000000000000000000002");
        let set = ScenarioSet {
            id: "set-1".to_owned(),
            baseline_snapshot_id: 0,
            baseline_fork: None,
            scenarios: vec![Scenario {
                id: "scenario-1".to_owned(),
                name: "transfer".to_owned(),
                transactions: vec![simple_transaction(sender, receiver, U256::from(5_u64))],
                track_addresses: vec![sender, receiver],
                max_gas: Some(30_000),
                timeout: Duration::from_secs(1),
                assertions: ScenarioAssertions::default(),
            }],
            status: ScenarioSetStatus::Draft,
        };

        let results = runner.run_sequential(&set).await;
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ScenarioStatus::Success));
    }

    #[tokio::test]
    async fn test_scenario_runner_revert_restores_baseline() {
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let sender = address!("0x2000000000000000000000000000000000000001");
        let receiver = address!("0x2000000000000000000000000000000000000002");
        let set = ScenarioSet {
            id: "set-2".to_owned(),
            baseline_snapshot_id: 0,
            baseline_fork: None,
            scenarios: vec![Scenario {
                id: "scenario-2".to_owned(),
                name: "transfer".to_owned(),
                transactions: vec![simple_transaction(sender, receiver, U256::from(7_u64))],
                track_addresses: vec![sender, receiver],
                max_gas: Some(30_000),
                timeout: Duration::from_secs(1),
                assertions: ScenarioAssertions::default(),
            }],
            status: ScenarioSetStatus::Draft,
        };

        let runner = ScenarioRunner::new(mirage.state());
        let _ = runner.run_sequential(&set).await;
        let balance = {
            let state = mirage.state();
            let maybe_info = state
                .read()
                .fork
                .db
                .upstream
                .get_account_info(sender, crate::provider::BlockTag::Latest);
            let info = match maybe_info {
                Ok(Some(info)) => info,
                Ok(None) => panic!("mock account exists"),
                Err(error) => panic!("mock read succeeds: {error}"),
            };
            info.balance
        };
        assert_eq!(balance, U256::from(1_000_000_000_000_000_000_u64));

        let _ = RunMode::Parallel;
    }

    #[tokio::test]
    async fn scenario_runner_sequential_releases_state_write_lock_during_execution() {
        let sender = address!("0x2100000000000000000000000000000000000001");
        let receiver = address!("0x2100000000000000000000000000000000000002");
        let upstream = Arc::new(UpstreamRpc::mock(1));
        upstream.set_mock_delay(Duration::from_millis(150));
        let db = HybridDB::new(
            Arc::clone(&upstream),
            32,
            Duration::from_secs(12),
            NonZeroUsize::MIN,
            1,
        );
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let set = ScenarioSet {
            id: "set-3".to_owned(),
            baseline_snapshot_id: 0,
            baseline_fork: None,
            scenarios: vec![Scenario {
                id: "scenario-3".to_owned(),
                name: "transfer".to_owned(),
                transactions: vec![simple_transaction(sender, receiver, U256::from(3_u64))],
                track_addresses: vec![sender, receiver],
                max_gas: Some(30_000),
                timeout: Duration::from_secs(1),
                assertions: ScenarioAssertions::default(),
            }],
            status: ScenarioSetStatus::Draft,
        };

        let state = mirage.state();
        let state_for_task = Arc::clone(&state);
        let set_for_task = set.clone();
        let task = tokio::spawn(async move {
            ScenarioRunner::new(state_for_task)
                .run_sequential(&set_for_task)
                .await
        });
        tokio::time::sleep(Duration::from_millis(20)).await;

        let started = std::time::Instant::now();
        let read_guard = state.read();
        assert!(started.elapsed() < Duration::from_millis(75));
        drop(read_guard);

        let results = task
            .await
            .unwrap_or_else(|error| panic!("join scenario task: {error}"));
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ScenarioStatus::Success));
    }

    #[tokio::test]
    async fn scenario_runner_releases_writer_gate_during_execution() {
        let sender = address!("0x2200000000000000000000000000000000000001");
        let receiver = address!("0x2200000000000000000000000000000000000002");
        let upstream = Arc::new(UpstreamRpc::mock(1));
        upstream.set_mock_delay(Duration::from_millis(150));
        let db = HybridDB::new(
            Arc::clone(&upstream),
            32,
            Duration::from_secs(12),
            NonZeroUsize::MIN,
            1,
        );
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let set = ScenarioSet {
            id: "set-4".to_owned(),
            baseline_snapshot_id: 0,
            baseline_fork: None,
            scenarios: vec![Scenario {
                id: "scenario-4".to_owned(),
                name: "transfer".to_owned(),
                transactions: vec![simple_transaction(sender, receiver, U256::from(4_u64))],
                track_addresses: vec![sender, receiver],
                max_gas: Some(30_000),
                timeout: Duration::from_secs(1),
                assertions: ScenarioAssertions::default(),
            }],
            status: ScenarioSetStatus::Draft,
        };

        let state = mirage.state();
        let state_for_task = Arc::clone(&state);
        let set_for_task = set.clone();
        let task = tokio::spawn(async move {
            ScenarioRunner::new(state_for_task)
                .run_sequential(&set_for_task)
                .await
        });
        tokio::time::sleep(Duration::from_millis(20)).await;

        let started = std::time::Instant::now();
        crate::fork::with_state_write(&state, |state| {
            state.reject_new_forks = !state.reject_new_forks;
        })
        .await;
        assert!(started.elapsed() < Duration::from_millis(75));

        let results = task
            .await
            .unwrap_or_else(|error| panic!("join scenario task: {error}"));
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ScenarioStatus::Success));
    }

    #[test]
    fn scenario_results_rank_by_pnl_then_cost() {
        let low_value = ScenarioResult {
            scenario_id: "low".to_owned(),
            name: "low".to_owned(),
            status: ScenarioStatus::Success,
            gas_used: 200,
            wall_time_ms: 10,
            peak_memory_bytes: 0,
            pnl_wei: 20,
            state_diff_accounts: 1,
            state_diff_storage_slots: 1,
            final_balances: Default::default(),
            position_state: serde_json::json!({}),
            logs: Vec::new(),
            revert_reason: None,
        };
        let high_value = ScenarioResult {
            scenario_id: "high".to_owned(),
            name: "high".to_owned(),
            status: ScenarioStatus::Success,
            gas_used: 400,
            wall_time_ms: 10,
            peak_memory_bytes: 0,
            pnl_wei: 30,
            state_diff_accounts: 3,
            state_diff_storage_slots: 5,
            final_balances: Default::default(),
            position_state: serde_json::json!({}),
            logs: Vec::new(),
            revert_reason: None,
        };

        let ranked = rank_scenario_results(vec![low_value.clone(), high_value.clone()]);
        assert_eq!(ranked[0].scenario_id, high_value.scenario_id);
    }

    #[test]
    fn scenario_fixture_supports_assertions() {
        let fixture = r#"
[scenario]
name = "uniswap_v3_lp_entry"
description = "Add concentrated liquidity"

[[transactions]]
from = "0x3000000000000000000000000000000000000001"
to = "0x3000000000000000000000000000000000000010"
value = "0x0"
gas = 500000
data = "0xa9059cbb00000000000000000000000030000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000005"

[assertions]
watch_list_contains = ["0x3000000000000000000000000000000000000010"]

[assertions.token_balance_gte]
token = "0x3000000000000000000000000000000000000010"
address = "0x3000000000000000000000000000000000000020"
amount = "0x5"

[track]
addresses = ["0x3000000000000000000000000000000000000010"]
"#;

        let scenario = Scenario::from_toml("fixture-1", fixture)
            .unwrap_or_else(|error| panic!("fixture parses: {error}"));
        assert_eq!(scenario.transactions.len(), 1);
        assert_eq!(scenario.assertions.watch_list_contains.len(), 1);
        assert!(scenario.assertions.token_balance_gte.is_some());
    }

    #[test]
    fn built_in_fixtures_parse() {
        let fixtures = [
            (
                "eth-crash",
                include_str!("../tests/scenarios/eth_crash.toml"),
            ),
            (
                "volume-spike",
                include_str!("../tests/scenarios/volume_spike.toml"),
            ),
            (
                "uniswap-v3-entry",
                include_str!("../tests/scenarios/uniswap_v3_entry.toml"),
            ),
            (
                "aave-liquidation",
                include_str!("../tests/scenarios/aave_liquidation.toml"),
            ),
            ("new-pool", include_str!("../tests/scenarios/new_pool.toml")),
        ];

        for (id, fixture) in fixtures {
            let scenario = Scenario::from_toml(id, fixture)
                .unwrap_or_else(|error| panic!("fixture {id} parses: {error}"));
            assert!(!scenario.transactions.is_empty());
            assert!(!scenario.track_addresses.is_empty());
        }
    }

    #[test]
    fn built_in_fixtures_match_expected_scale() {
        let eth_crash = Scenario::from_toml(
            "eth-crash",
            include_str!("../tests/scenarios/eth_crash.toml"),
        )
        .unwrap_or_else(|error| panic!("eth crash parses: {error}"));
        assert!((20..=40).contains(&eth_crash.transactions.len()));

        let volume_spike = Scenario::from_toml(
            "volume-spike",
            include_str!("../tests/scenarios/volume_spike.toml"),
        )
        .unwrap_or_else(|error| panic!("volume spike parses: {error}"));
        assert_eq!(volume_spike.transactions.len(), 100);

        let aave = Scenario::from_toml(
            "aave-liquidation",
            include_str!("../tests/scenarios/aave_liquidation.toml"),
        )
        .unwrap_or_else(|error| panic!("aave liquidation parses: {error}"));
        assert!(aave.transactions.len() >= 3);

        let new_pool =
            Scenario::from_toml("new-pool", include_str!("../tests/scenarios/new_pool.toml"))
                .unwrap_or_else(|error| panic!("new pool parses: {error}"));
        assert!(new_pool.transactions.iter().any(|tx| tx.to.is_none()));
    }

    #[test]
    fn test_scenario_status_valid_transitions() {
        // ScenarioSetStatus: Draft → Running → Complete | Failed
        let draft = ScenarioSetStatus::Draft;
        let running = ScenarioSetStatus::Running;
        let complete = ScenarioSetStatus::Complete;
        let failed = ScenarioSetStatus::Failed;

        assert_ne!(draft, running);
        assert_ne!(running, complete);
        assert_ne!(running, failed);
        assert_ne!(complete, failed);

        // ScenarioStatus variants are distinct terminal states
        let success = ScenarioStatus::Success;
        let timeout = ScenarioStatus::Timeout;
        let gas_exceeded = ScenarioStatus::GasExceeded;
        let reverted = ScenarioStatus::Reverted;

        assert!(matches!(success, ScenarioStatus::Success));
        assert!(matches!(timeout, ScenarioStatus::Timeout));
        assert!(matches!(gas_exceeded, ScenarioStatus::GasExceeded));
        assert!(matches!(reverted, ScenarioStatus::Reverted));
        assert!(matches!(
            ScenarioStatus::Error("x".into()),
            ScenarioStatus::Error(_)
        ));
    }

    #[tokio::test]
    async fn test_scenario_timeout_enforced() {
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let sender = address!("0x5000000000000000000000000000000000000001");
        let receiver = address!("0x5000000000000000000000000000000000000002");
        let set = ScenarioSet {
            id: "timeout-set".to_owned(),
            baseline_snapshot_id: 0,
            baseline_fork: None,
            scenarios: vec![Scenario {
                id: "timeout-scenario".to_owned(),
                name: "should timeout".to_owned(),
                transactions: vec![
                    simple_transaction(sender, receiver, U256::from(1_u64)),
                    simple_transaction(sender, receiver, U256::from(1_u64)),
                    simple_transaction(sender, receiver, U256::from(1_u64)),
                ],
                track_addresses: vec![sender],
                max_gas: None,
                timeout: Duration::ZERO,
                assertions: ScenarioAssertions::default(),
            }],
            status: ScenarioSetStatus::Draft,
        };

        let runner = ScenarioRunner::new(mirage.state());
        let results = runner.run_sequential(&set).await;
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ScenarioStatus::Timeout));
    }

    #[tokio::test]
    async fn test_scenario_gas_exceeded() {
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let sender = address!("0x6000000000000000000000000000000000000001");
        let receiver = address!("0x6000000000000000000000000000000000000002");
        let set = ScenarioSet {
            id: "gas-set".to_owned(),
            baseline_snapshot_id: 0,
            baseline_fork: None,
            scenarios: vec![Scenario {
                id: "gas-scenario".to_owned(),
                name: "should exceed gas".to_owned(),
                transactions: vec![simple_transaction(sender, receiver, U256::from(1_u64))],
                track_addresses: vec![sender],
                max_gas: Some(1),
                timeout: Duration::from_secs(10),
                assertions: ScenarioAssertions::default(),
            }],
            status: ScenarioSetStatus::Draft,
        };

        let runner = ScenarioRunner::new(mirage.state());
        let results = runner.run_sequential(&set).await;
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ScenarioStatus::GasExceeded));
    }

    #[tokio::test]
    async fn test_scenario_tracks_all_addresses() {
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let sender = address!("0x7000000000000000000000000000000000000001");
        let receiver = address!("0x7000000000000000000000000000000000000002");
        let bystander = address!("0x7000000000000000000000000000000000000003");
        let set = ScenarioSet {
            id: "track-set".to_owned(),
            baseline_snapshot_id: 0,
            baseline_fork: None,
            scenarios: vec![Scenario {
                id: "track-scenario".to_owned(),
                name: "track all".to_owned(),
                transactions: vec![simple_transaction(sender, receiver, U256::from(1_u64))],
                track_addresses: vec![sender, receiver, bystander],
                max_gas: None,
                timeout: Duration::from_secs(10),
                assertions: ScenarioAssertions::default(),
            }],
            status: ScenarioSetStatus::Draft,
        };

        let runner = ScenarioRunner::new(mirage.state());
        let results = runner.run_sequential(&set).await;
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ScenarioStatus::Success));
        assert!(results[0].final_balances.contains_key(&sender));
        assert!(results[0].final_balances.contains_key(&receiver));
        assert!(results[0].final_balances.contains_key(&bystander));
    }
}
