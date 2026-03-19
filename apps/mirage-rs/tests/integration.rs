//! Process-spawning integration tests for `mirage-rs`.

#![allow(clippy::default_trait_access, clippy::expect_used)]

use std::time::Duration;

use alloy_primitives::{Address, U256, address};
use mirage_rs::{
    JobStatus, MirageClient, PositionRequest, RunMode, Scenario, ScenarioAssertions,
    TransactionRequest, spawn_mirage_test_instance,
};
use serde::de::DeserializeOwned;

#[tokio::test]
async fn integration_eth_transfer_state_diff() {
    let mut instance = spawn_mirage_test_instance(None, Some(18_545))
        .await
        .expect("spawn test instance");
    let client = MirageClient::new(instance.config()).expect("construct client");
    client
        .wait_ready(Duration::from_secs(5))
        .await
        .expect("instance ready");

    let sender = address!("0x1000000000000000000000000000000000000001");
    let receiver = address!("0x1000000000000000000000000000000000000002");
    let before_sender = rpc_call::<String>(
        &instance.config().url,
        "eth_getBalance",
        serde_json::json!([sender, "latest"]),
    )
    .await
    .expect("read sender balance before");
    let before_receiver = rpc_call::<String>(
        &instance.config().url,
        "eth_getBalance",
        serde_json::json!([receiver, "latest"]),
    )
    .await
    .expect("read receiver balance before");

    let tx_hash = client
        .eth_send_transaction(TransactionRequest {
            from: Some(sender),
            to: Some(receiver),
            gas: Some(21_000),
            value: Some(U256::from(25_u64)),
            data: Some(Default::default()),
            gas_price: None,
            nonce: None,
            chain_id: Some(1),
        })
        .await
        .expect("submit transfer");

    let receipt = rpc_call::<serde_json::Value>(
        &instance.config().url,
        "eth_getTransactionReceipt",
        serde_json::json!([tx_hash]),
    )
    .await
    .expect("get receipt");
    assert_eq!(receipt["status"], "0x1");

    let after_sender = rpc_call::<String>(
        &instance.config().url,
        "eth_getBalance",
        serde_json::json!([sender, "latest"]),
    )
    .await
    .expect("read sender balance after");
    let after_receiver = rpc_call::<String>(
        &instance.config().url,
        "eth_getBalance",
        serde_json::json!([receiver, "latest"]),
    )
    .await
    .expect("read receiver balance after");

    assert!(parse_u256(&after_sender) < parse_u256(&before_sender));
    assert!(parse_u256(&after_receiver) > parse_u256(&before_receiver));

    instance.shutdown().await.expect("shutdown instance");
}

#[tokio::test]
async fn integration_snapshot_revert() {
    let mut instance = spawn_mirage_test_instance(None, Some(18_546))
        .await
        .expect("spawn test instance");
    let client = MirageClient::new(instance.config()).expect("construct client");
    client
        .wait_ready(Duration::from_secs(5))
        .await
        .expect("instance ready");

    let sender = address!("0x2000000000000000000000000000000000000001");
    let receiver = address!("0x2000000000000000000000000000000000000002");
    let snapshot = client.evm_snapshot().await.expect("take snapshot");
    client
        .eth_send_transaction(TransactionRequest {
            from: Some(sender),
            to: Some(receiver),
            gas: Some(21_000),
            value: Some(U256::from(10_u64)),
            data: Some(Default::default()),
            gas_price: None,
            nonce: None,
            chain_id: Some(1),
        })
        .await
        .expect("submit transfer");
    let changed = rpc_call::<String>(
        &instance.config().url,
        "eth_getBalance",
        serde_json::json!([receiver, "latest"]),
    )
    .await
    .expect("read changed balance");
    assert!(parse_u256(&changed) > U256::from(1_000_000_000_000_000_000_u64));

    assert!(client.evm_revert(snapshot).await.expect("revert snapshot"));
    let reverted = rpc_call::<String>(
        &instance.config().url,
        "eth_getBalance",
        serde_json::json!([receiver, "latest"]),
    )
    .await
    .expect("read reverted balance");
    assert_eq!(
        parse_u256(&reverted),
        U256::from(1_000_000_000_000_000_000_u64)
    );

    instance.shutdown().await.expect("shutdown instance");
}

#[tokio::test]
async fn integration_scenario_runner_cow_isolation() {
    let mut instance = spawn_mirage_test_instance(None, Some(18_547))
        .await
        .expect("spawn test instance");
    let client = MirageClient::new(instance.config()).expect("construct client");
    client
        .wait_ready(Duration::from_secs(5))
        .await
        .expect("instance ready");

    let sender = address!("0x3000000000000000000000000000000000000001");
    let left = address!("0x3000000000000000000000000000000000000002");
    let right = address!("0x3000000000000000000000000000000000000003");
    let set_id = client
        .mirage_begin_scenario_set("latest")
        .await
        .expect("begin set");

    let left_scenario = Scenario {
        id: "left-branch".to_owned(),
        name: "left transfer".to_owned(),
        transactions: vec![tx(sender, left, 4)],
        track_addresses: vec![sender, left],
        max_gas: Some(30_000),
        timeout: Duration::from_secs(1),
        assertions: ScenarioAssertions::default(),
    };
    let right_scenario = Scenario {
        id: "right-branch".to_owned(),
        name: "right transfer".to_owned(),
        transactions: vec![tx(sender, right, 9)],
        track_addresses: vec![sender, right],
        max_gas: Some(30_000),
        timeout: Duration::from_secs(1),
        assertions: ScenarioAssertions::default(),
    };
    client
        .mirage_define_scenario(&set_id, &left_scenario)
        .await
        .expect("define left");
    client
        .mirage_define_scenario(&set_id, &right_scenario)
        .await
        .expect("define right");

    let job_id = client
        .mirage_run_scenario_set(&set_id, RunMode::Parallel)
        .await
        .expect("run scenario set");
    let job = wait_for_job(&client, &job_id).await;
    assert!(matches!(job.status, JobStatus::Complete));
    assert_eq!(job.results.as_ref().expect("results present").len(), 2);

    let owner_view = client
        .mirage_get_position(PositionRequest {
            owner: sender,
            protocol_type: "raw-balances".to_owned(),
            contract: None,
            token_addresses: vec![sender, left, right],
        })
        .await
        .expect("read position snapshot");
    assert_eq!(owner_view.protocol_type, "raw-balances");

    instance.shutdown().await.expect("shutdown instance");
}

fn tx(from: Address, to: Address, value: u64) -> TransactionRequest {
    TransactionRequest {
        from: Some(from),
        to: Some(to),
        gas: Some(21_000),
        value: Some(U256::from(value)),
        data: Some(Default::default()),
        gas_price: None,
        nonce: None,
        chain_id: Some(1),
    }
}

async fn wait_for_job(client: &MirageClient, job_id: &str) -> mirage_rs::ScenarioJob {
    for _ in 0..20 {
        let job = client
            .mirage_get_scenario_results(job_id)
            .await
            .expect("poll scenario job");
        if matches!(job.status, JobStatus::Complete | JobStatus::Failed) {
            return job;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("scenario job did not complete in time");
}

async fn rpc_call<T: DeserializeOwned>(
    url: &str,
    method: &str,
    params: serde_json::Value,
) -> anyhow::Result<T> {
    let response = reqwest::Client::new()
        .post(url)
        .json(&serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}))
        .send()
        .await?;
    let value = response.json::<serde_json::Value>().await?;
    Ok(serde_json::from_value(value["result"].clone())?)
}

fn parse_u256(text: &str) -> U256 {
    U256::from_str_radix(text.trim_start_matches("0x"), 16).expect("valid U256 quantity")
}
