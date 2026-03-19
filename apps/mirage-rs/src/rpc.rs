//! JSON-RPC server surface for `mirage-rs`.

#![allow(
    clippy::default_trait_access,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use std::{
    convert::Infallible,
    net::{SocketAddr, ToSocketAddrs},
    num::NonZeroUsize,
    sync::Arc,
    time::Duration,
};

use alloy_primitives::{Address, B256, Bytes, U256, hex, keccak256};
use axum::{
    Router,
    body::Body,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{Request, Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get},
};
use jsonrpsee::{
    RpcModule,
    core::RegisterMethodError,
    server::{ServerBuilder, ServerHandle},
    types::ErrorObjectOwned,
};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use parking_lot::RwLock;
use tower_http::cors::CorsLayer;
use tokio::sync::broadcast;
use tower::Service;

use crate::{
    Bytecode, MirageError, Result, TransactionRequest,
    fork::{
        ClassificationConfig, DiffClassifier, EvmExecutor, ForkState, HybridDB, LocalBlock,
        LocalReceipt, LocalTransaction, MirageFork, MirageState, WatchSource, lock_state_writes,
        with_state_write,
    },
    integration::{EventFilter, EventSource, MirageEvent, PositionRequest, PositionSnapshot},
    provider::UpstreamRpc,
    resources::{MirageMode, PressureAction, Profile, ResourceModel},
    scenario::{
        JobStatus, RunMode, Scenario, ScenarioJob, ScenarioRunner, ScenarioSet, ScenarioSetStatus,
        rank_scenario_results,
    },
};

#[derive(Clone)]
struct ServerContext {
    state: Arc<RwLock<MirageState>>,
    shutdown: broadcast::Sender<()>,
}

/// Starts a JSON-RPC server on the provided address.
pub async fn start_rpc_server(
    address: impl ToSocketAddrs,
    mirage: MirageFork,
    shutdown: broadcast::Sender<()>,
) -> Result<(SocketAddr, ServerHandle)> {
    let address = address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| MirageError::Unsupported("no socket address resolved".to_owned()))?;
    let local_state = mirage.state();
    let module = build_rpc_module(ServerContext {
        state: Arc::clone(&local_state),
        shutdown,
    })
    .map_err(|error| MirageError::Unsupported(error.to_string()))?;
    let (stop_handle, server_handle) = jsonrpsee::server::stop_channel();
    let rpc_service = ServerBuilder::default()
        .to_service_builder()
        .build(module, stop_handle);
    let rpc_fallback = tower::service_fn(move |request: Request<Body>| {
        let mut rpc_service = rpc_service.clone();
        async move {
            match Service::call(&mut rpc_service, request).await {
                Ok(response) => Ok::<Response<Body>, Infallible>(response.map(Body::new)),
                Err(error) => {
                    tracing::warn!("jsonrpsee rpc service failed: {error}");
                    let mut response = Response::new(Body::from("internal server error".to_owned()));
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    Ok(response)
                }
            }
        }
    });
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|_| MirageError::BindFailed(address.port()))?;
    let local_addr = listener
        .local_addr()
        .map_err(|_| MirageError::BindFailed(address.port()))?;
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/events/{stream_id}", get(event_ws_handler))
        .route("/events/{stream_id}", delete(unsubscribe_event_handler))
        .fallback_service(rpc_fallback)
        .layer(CorsLayer::permissive())
        .with_state(local_state);
    let shutdown_handle = server_handle.clone();
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_handle.stopped())
            .await
        {
            tracing::warn!("mirage http server exited with error: {error}");
        }
    });
    Ok((local_addr, server_handle))
}

/// Starts an ephemeral server for tests.
pub async fn spawn_rpc_server_for_tests() -> Result<(String, ServerHandle)> {
    let upstream = Arc::new(UpstreamRpc::mock(1));
    let db = HybridDB::new(upstream, 64, Duration::from_secs(12), NonZeroUsize::MIN, 1);
    let fork = ForkState::new(db, 0, 1);
    let mirage = MirageFork::new(
        fork,
        ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
        MirageMode::Live,
    );
    let (shutdown, _) = broadcast::channel(4);
    let (addr, handle) = start_rpc_server("127.0.0.1:0", mirage, shutdown).await?;
    Ok((format!("http://{addr}"), handle))
}

fn build_rpc_module(
    context: ServerContext,
) -> std::result::Result<RpcModule<ServerContext>, RegisterMethodError> {
    let mut module = RpcModule::new(context);

    module.register_async_method("web3_clientVersion", |_params, _ctx, _| async {
        Ok::<_, ErrorObjectOwned>("mirage-rs/2.0.0".to_owned())
    })?;

    module.register_async_method("net_version", |_params, ctx, _| async move {
        let state = ctx.state.read();
        Ok::<_, ErrorObjectOwned>(state.fork.chain_id.to_string())
    })?;

    module.register_async_method("eth_chainId", |_params, ctx, _| async move {
        let state = ctx.state.read();
        Ok::<_, ErrorObjectOwned>(hex_u64(state.fork.chain_id))
    })?;

    module.register_async_method("eth_blockNumber", |_params, ctx, _| async move {
        let state = ctx.state.read();
        Ok::<_, ErrorObjectOwned>(hex_u64(state.fork.local_block_number))
    })?;

    module.register_async_method("eth_gasPrice", |_params, _ctx, _| async {
        Ok::<_, ErrorObjectOwned>("0x1".to_owned())
    })?;

    module.register_async_method("eth_maxPriorityFeePerGas", |_params, _ctx, _| async {
        Ok::<_, ErrorObjectOwned>("0x0".to_owned())
    })?;

    module.register_async_method("eth_feeHistory", |_params, _ctx, _| async {
        Ok::<_, ErrorObjectOwned>(serde_json::json!({
            "oldestBlock": "0x0",
            "baseFeePerGas": ["0x1"],
            "gasUsedRatio": [0.5],
            "reward": [["0x0"]],
        }))
    })?;

    module.register_async_method("eth_getBalance", |params, ctx, _| async move {
        let (address, _block): (Address, serde_json::Value) =
            params.parse().map_err(invalid_params)?;
        let balance = run_fork_snapshot(&ctx.state, true, move |mut fork| {
            Ok(fork.db.basic(address)?.unwrap_or_default().balance)
        })
        .await
        .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(hex_u256(balance))
    })?;

    module.register_async_method("eth_getTransactionCount", |params, ctx, _| async move {
        let (address, _block): (Address, serde_json::Value) =
            params.parse().map_err(invalid_params)?;
        let nonce = run_fork_snapshot(&ctx.state, true, move |mut fork| {
            Ok(fork.db.basic(address)?.unwrap_or_default().nonce)
        })
        .await
        .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(hex_u64(nonce))
    })?;

    module.register_async_method("eth_getStorageAt", |params, ctx, _| async move {
        let (address, slot, _block): (Address, U256, serde_json::Value) =
            params.parse().map_err(invalid_params)?;
        let value = run_fork_snapshot(&ctx.state, true, move |mut fork| {
            fork.db.storage(address, slot)
        })
        .await
        .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(format!("0x{:064x}", value))
    })?;

    module.register_async_method("eth_getCode", |params, ctx, _| async move {
        let (address, _block): (Address, serde_json::Value) =
            params.parse().map_err(invalid_params)?;
        let code = run_fork_snapshot(&ctx.state, true, move |mut fork| {
            Ok(fork
                .db
                .basic(address)?
                .unwrap_or_default()
                .code
                .unwrap_or_default())
        })
        .await
        .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(format!("0x{}", hex::encode(code.bytecode())))
    })?;

    module.register_async_method("eth_call", |params, ctx, _| async move {
        let (request, _block): (TransactionRequest, serde_json::Value) =
            params.parse().map_err(invalid_params)?;
        let from = request
            .from
            .ok_or_else(|| invalid_params_message("missing from"))?;
        let to = extract_to(request.to).ok_or_else(|| invalid_params_message("missing to"))?;
        let data = request.data.unwrap_or_default();
        let value = request.value.unwrap_or(U256::ZERO);
        let gas = request.gas.unwrap_or(21_000);
        let result = run_fork_snapshot(&ctx.state, false, move |fork| {
            EvmExecutor::call(&fork, from, to, data, value, gas)
        })
        .await
        .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(format!("0x{}", hex::encode(&result.output)))
    })?;

    module.register_async_method("eth_estimateGas", |params, ctx, _| async move {
        let (request,): (TransactionRequest,) = params.parse().map_err(invalid_params)?;
        let result = run_fork_snapshot(&ctx.state, false, move |fork| {
            let mut executor = crate::replay::SpeculativeExecutor::default();
            executor.execute(&fork, &request)
        })
        .await
        .map_err(rpc_error)?;
        let estimate = result.state_diff.gas_used.saturating_mul(12) / 10;
        Ok::<_, ErrorObjectOwned>(hex_u64(estimate.max(21_000)))
    })?;

    module.register_async_method("eth_sendTransaction", |params, ctx, _| async move {
        let (request,): (TransactionRequest,) = params.parse().map_err(invalid_params)?;
        let tx_hash = commit_transaction_request(&ctx.state, request, None)
            .await
            .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(tx_hash)
    })?;

    module.register_async_method("eth_sendRawTransaction", |params, ctx, _| async move {
        let (raw,): (Bytes,) = params.parse().map_err(invalid_params)?;
        let decoded = decode_signed_raw_transaction(&raw).map_err(rpc_error)?;
        let tx_hash =
            commit_transaction_request(&ctx.state, decoded.request, Some(decoded.tx_hash))
                .await
                .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(tx_hash)
    })?;

    module.register_async_method("eth_getTransactionReceipt", |params, ctx, _| async move {
        let (tx_hash,): (B256,) = params.parse().map_err(invalid_params)?;
        let state = ctx.state.read();
        let receipt = state.fork.receipts.get(&tx_hash).map(receipt_json);
        Ok::<_, ErrorObjectOwned>(receipt)
    })?;

    module.register_async_method("eth_getTransactionByHash", |params, ctx, _| async move {
        let (tx_hash,): (B256,) = params.parse().map_err(invalid_params)?;
        let state = ctx.state.read();
        let tx = state.fork.transactions.get(&tx_hash).map(transaction_json);
        Ok::<_, ErrorObjectOwned>(tx)
    })?;

    module.register_async_method("eth_getLogs", |_params, _ctx, _| async {
        Ok::<_, ErrorObjectOwned>(Vec::<serde_json::Value>::new())
    })?;

    module.register_async_method("eth_getBlockByNumber", |params, ctx, _| async move {
        let (number, _full): (String, bool) = params.parse().map_err(invalid_params)?;
        let state = ctx.state.read();
        let block = if number == "latest" {
            state
                .fork
                .blocks_by_number
                .get(&state.fork.local_block_number)
        } else {
            parse_hex_quantity(&number)
                .ok()
                .and_then(|number| state.fork.blocks_by_number.get(&number))
        }
        .map(block_json);
        Ok::<_, ErrorObjectOwned>(block)
    })?;

    module.register_async_method("eth_getBlockByHash", |params, ctx, _| async move {
        let (hash, _full): (B256, bool) = params.parse().map_err(invalid_params)?;
        let state = ctx.state.read();
        Ok::<_, ErrorObjectOwned>(state.fork.blocks_by_hash.get(&hash).map(block_json))
    })?;

    register_impersonation_methods(&mut module)?;
    register_state_mutation_methods(&mut module)?;
    register_snapshot_methods(&mut module)?;
    register_mirage_methods(&mut module)?;

    Ok(module)
}

fn register_impersonation_methods(
    module: &mut RpcModule<ServerContext>,
) -> std::result::Result<(), RegisterMethodError> {
    for method in ["hardhat_impersonateAccount", "anvil_impersonateAccount"] {
        module.register_async_method(method, |params, ctx, _| async move {
            let (address,): (Address,) = params.parse().map_err(invalid_params)?;
            with_state_write(&ctx.state, |state| {
                state.fork.impersonated_accounts.insert(address);
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    for method in [
        "hardhat_stopImpersonatingAccount",
        "anvil_stopImpersonatingAccount",
    ] {
        module.register_async_method(method, |params, ctx, _| async move {
            let (address,): (Address,) = params.parse().map_err(invalid_params)?;
            with_state_write(&ctx.state, |state| {
                state.fork.impersonated_accounts.remove(&address);
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    Ok(())
}

fn register_state_mutation_methods(
    module: &mut RpcModule<ServerContext>,
) -> std::result::Result<(), RegisterMethodError> {
    for method in [
        "hardhat_setBalance",
        "anvil_setBalance",
        "mirage_setBalance",
    ] {
        module.register_async_method(method, |params, ctx, _| async move {
            let (address, balance): (Address, U256) = params.parse().map_err(invalid_params)?;
            with_state_write(&ctx.state, |state| {
                state.fork.db.set_balance(address, balance);
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    for method in [
        "hardhat_setStorageAt",
        "anvil_setStorageAt",
        "mirage_setStorageAt",
    ] {
        module.register_async_method(method, |params, ctx, _| async move {
            let (address, slot, value): (Address, U256, U256) =
                params.parse().map_err(invalid_params)?;
            with_state_write(&ctx.state, |state| {
                state.fork.db.set_storage(address, slot, value);
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    for method in ["hardhat_setCode", "anvil_setCode", "mirage_setCode"] {
        module.register_async_method(method, |params, ctx, _| async move {
            let (address, code): (Address, Bytes) = params.parse().map_err(invalid_params)?;
            let bytecode = Bytecode::new_raw(code);
            with_state_write(&ctx.state, |state| {
                state.fork.db.set_code(address, bytecode);
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    module.register_async_method("anvil_setNonce", |params, ctx, _| async move {
        let (address, nonce): (Address, u64) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            state.fork.db.set_nonce(address, nonce);
        })
        .await;
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    for method in ["hardhat_mine", "anvil_mine", "evm_mine"] {
        module.register_async_method(method, |params, ctx, _| async move {
            let params: std::result::Result<Vec<serde_json::Value>, _> = params.parse();
            let count = params
                .ok()
                .and_then(|values| {
                    values
                        .first()
                        .and_then(|value| value.as_str())
                        .and_then(|text| parse_hex_quantity(text).ok())
                })
                .unwrap_or(1);
            with_state_write(&ctx.state, |state| {
                state.fork.local_block_number = state.fork.local_block_number.saturating_add(count);
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    for method in ["hardhat_reset", "anvil_reset"] {
        module.register_async_method(method, |_params, ctx, _| async move {
            with_state_write(&ctx.state, |state| {
                state.fork.db.reset();
                state.fork.receipts.clear();
                state.fork.transactions.clear();
                state.fork.blocks_by_hash.clear();
                state.fork.blocks_by_number.clear();
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    for method in [
        "hardhat_setNextBlockBaseFeePerGas",
        "anvil_setNextBlockBaseFeePerGas",
    ] {
        module.register_async_method(method, |params, ctx, _| async move {
            let (value,): (u128,) = params.parse().map_err(invalid_params)?;
            with_state_write(&ctx.state, |state| {
                state.fork.next_base_fee_per_gas = value;
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    for method in ["hardhat_setCoinbase", "anvil_setCoinbase"] {
        module.register_async_method(method, |params, ctx, _| async move {
            let (value,): (Address,) = params.parse().map_err(invalid_params)?;
            with_state_write(&ctx.state, |state| {
                state.fork.coinbase = value;
            })
            .await;
            Ok::<_, ErrorObjectOwned>(true)
        })?;
    }
    module.register_async_method("anvil_setPrevRandao", |params, ctx, _| async move {
        let (value,): (B256,) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            state.fork.prev_randao = value;
        })
        .await;
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    Ok(())
}

fn register_snapshot_methods(
    module: &mut RpcModule<ServerContext>,
) -> std::result::Result<(), RegisterMethodError> {
    module.register_async_method("evm_snapshot", |_params, ctx, _| async move {
        let snapshot = with_state_write(&ctx.state, |state| hex_u64(state.fork.snapshot())).await;
        Ok::<_, ErrorObjectOwned>(snapshot)
    })?;
    module.register_async_method("evm_revert", |params, ctx, _| async move {
        let (snapshot_id,): (String,) = params.parse().map_err(invalid_params)?;
        let id = parse_hex_quantity(&snapshot_id).map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| state.fork.revert(id).map_err(rpc_error)).await
    })?;
    module.register_async_method("evm_increaseTime", |params, ctx, _| async move {
        let (seconds,): (u64,) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            state.fork.timestamp = state.fork.timestamp.saturating_add(seconds);
        })
        .await;
        Ok::<_, ErrorObjectOwned>(hex_u64(seconds))
    })?;
    module.register_async_method("evm_setNextBlockTimestamp", |params, ctx, _| async move {
        let (timestamp,): (u64,) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            state.fork.timestamp = timestamp;
        })
        .await;
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    Ok(())
}

fn register_mirage_methods(
    module: &mut RpcModule<ServerContext>,
) -> std::result::Result<(), RegisterMethodError> {
    module.register_async_method("mirage_mintERC20", |params, ctx, _| async move {
        let (token, owner, amount): (Address, Address, U256) =
            params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            touch_request(state);
            let current_balance = state
                .fork
                .db
                .erc20_balance_of(token, owner)
                .unwrap_or(U256::ZERO);
            let next_balance = current_balance.saturating_add(amount);
            state
                .fork
                .db
                .set_erc20_balance(token, owner, next_balance)
                .map_err(rpc_error)?;
            let added_at_block = state.fork.local_block_number;
            state
                .fork
                .db
                .dirty
                .watch_list
                .entry(token)
                .or_insert_with(|| crate::fork::WatchEntry {
                    source: crate::fork::WatchSource::Manual,
                    added_at_block,
                    initial_slot_count: 1,
                    replay_count: 0,
                });
            Ok::<_, ErrorObjectOwned>(())
        })
        .await?;
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    module.register_async_method("mirage_prefetchAccount", |params, ctx, _| async move {
        let (address,): (Address,) = params.parse().map_err(invalid_params)?;
        let account = run_fork_snapshot(&ctx.state, true, move |mut fork| fork.db.basic(address))
            .await
            .map_err(rpc_error)?;
        if let Some(account) = account {
            let mut state = ctx.state.write();
            state.fork.db.read_cache.insert_account(address, account);
        }
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    module.register_async_method("mirage_prefetchSlots", |params, ctx, _| async move {
        let (address, slots): (Address, Vec<U256>) = params.parse().map_err(invalid_params)?;
        let prefetched = run_fork_snapshot(&ctx.state, true, move |mut fork| {
            slots
                .into_iter()
                .map(|slot| fork.db.storage(address, slot).map(|value| (slot, value)))
                .collect::<Result<Vec<_>>>()
        })
        .await
        .map_err(rpc_error)?;
        let mut state = ctx.state.write();
        for (slot, value) in prefetched {
            state
                .fork
                .db
                .read_cache
                .insert_storage(address, slot, value);
        }
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    module.register_async_method("mirage_watchContract", |params, ctx, _| async move {
        let (address,): (Address,) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            touch_request(state);
            let added_at_block = state.fork.local_block_number;
            state.fork.db.dirty.watch_list.insert(
                address,
                crate::fork::WatchEntry {
                    source: crate::fork::WatchSource::Manual,
                    added_at_block,
                    initial_slot_count: 0,
                    replay_count: 0,
                },
            );
        })
        .await;
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    module.register_async_method("mirage_unwatchContract", |params, ctx, _| async move {
        let (address,): (Address,) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            touch_request(state);
            state.fork.db.dirty.watch_list.remove(&address);
            state.fork.db.dirty.unwatch_list.insert(address);
        })
        .await;
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    module.register_async_method("mirage_getWatchList", |_params, ctx, _| async move {
        let state = ctx.state.read();
        let watch_list = state
            .fork
            .db
            .dirty
            .watch_list
            .iter()
            .map(|(address, entry)| serde_json::json!({"address": address, "entry": entry}))
            .collect::<Vec<_>>();
        Ok::<_, ErrorObjectOwned>(watch_list)
    })?;
    module.register_async_method("mirage_getDirtySlots", |params, ctx, _| async move {
        let (address,): (Address,) = params.parse().map_err(invalid_params)?;
        let state = ctx.state.read();
        let slots = state
            .fork
            .db
            .dirty
            .accounts
            .get(&address)
            .map(|account| account.storage.clone())
            .unwrap_or_default();
        Ok::<_, ErrorObjectOwned>(slots)
    })?;
    module.register_async_method("mirage_status", |_params, ctx, _| async move {
        let state = ctx.state.read();
        Ok::<_, ErrorObjectOwned>(state.fork.status(state.mode))
    })?;
    module.register_async_method("mirage_getResourceUsage", |_params, ctx, _| async move {
        let usage = with_state_write(&ctx.state, |state| {
            touch_request(state);
            apply_resource_pressure(state);
            state.fork.resource_usage(&state.resource_model, state.mode)
        })
        .await;
        Ok::<_, ErrorObjectOwned>(usage)
    })?;
    module.register_async_method("mirage_setResourceLimits", |params, ctx, _| async move {
        let (profile,): (Option<Profile>,) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            touch_request(state);
            if let Some(profile) = profile {
                state.resource_model =
                    ResourceModel::for_profile(profile, state.resource_model.cache_ttl);
            }
        })
        .await;
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    module.register_async_method("mirage_getPosition", |params, ctx, _| async move {
        let (request,): (PositionRequest,) = params.parse().map_err(invalid_params)?;
        let snapshot = run_fork_snapshot(&ctx.state, true, move |mut fork| {
            let balances = request
                .token_addresses
                .iter()
                .map(|address| {
                    let balance = fork
                        .db
                        .erc20_balance_of(*address, request.owner)
                        .or_else(|_| fork.db.basic(*address).map(|info| info.unwrap_or_default().balance))
                        .unwrap_or(U256::ZERO);
                    (*address, hex_u256(balance))
                })
                .collect::<Vec<_>>();
            let data = match request.protocol_type.as_str() {
                "raw-balances" => serde_json::json!({"balances": balances}),
                "uniswap-v3-position" => serde_json::json!({
                    "balances": balances,
                    "positionNftBalance": request
                        .contract
                        .and_then(|contract| fork.db.erc20_balance_of(contract, request.owner).ok())
                        .map_or_else(|| hex_u256(U256::ZERO), hex_u256),
                    "pool": request.contract,
                }),
                "aave-v3-account" => serde_json::json!({
                    "balances": balances,
                    "market": request.contract,
                    "healthFactor": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                    "debtBalance": hex_u256(U256::ZERO),
                }),
                unknown => return Err(MirageError::UnknownProtocolType(unknown.to_owned())),
            };
            Ok(PositionSnapshot {
                owner: request.owner,
                protocol_type: request.protocol_type,
                data,
            })
        })
        .await
        .map_err(rpc_error)?;
        Ok::<_, ErrorObjectOwned>(snapshot)
    })?;
    module.register_async_method("mirage_subscribeEvents", |params, ctx, _| async move {
        let (filter,): (EventFilter,) = params.parse().map_err(invalid_params)?;
        let id = register_event_subscription(&ctx.state, filter);
        Ok::<_, ErrorObjectOwned>(id)
    })?;
    module.register_async_method("mirage_beginScenarioSet", |params, ctx, _| async move {
        let (baseline,): (String,) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            touch_request(state);
            if state.reject_new_forks {
                return Err::<String, _>(rpc_error(MirageError::Unsupported(
                    "resource pressure is refusing new scenario forks".to_owned(),
                )));
            }
            let (baseline_snapshot_id, baseline_fork) = if baseline == "latest" {
                let snapshot_id = state.fork.snapshot();
                (snapshot_id, Some(state.fork.clone()))
            } else {
                let snapshot_id = parse_hex_quantity(&baseline).map_err(invalid_params)?;
                let mut baseline_fork = state.fork.clone();
                baseline_fork.revert(snapshot_id).map_err(rpc_error)?;
                let refreshed_snapshot = baseline_fork.snapshot();
                (refreshed_snapshot, Some(baseline_fork))
            };
            let set_id = format!("set-{}", state.scenarios.len() + 1);
            state.scenarios.insert(
                set_id.clone(),
                ScenarioSet {
                    id: set_id.clone(),
                    baseline_snapshot_id,
                    baseline_fork,
                    scenarios: Vec::new(),
                    status: ScenarioSetStatus::Draft,
                },
            );
            Ok::<_, ErrorObjectOwned>(set_id)
        })
        .await
    })?;
    module.register_async_method("mirage_defineScenario", |params, ctx, _| async move {
        let (set_id, scenario): (String, Scenario) = params.parse().map_err(invalid_params)?;
        with_state_write(&ctx.state, |state| {
            touch_request(state);
            let set = state
                .scenarios
                .get_mut(&set_id)
                .ok_or_else(|| rpc_error(MirageError::SetNotFound(set_id.clone())))?;
            set.scenarios.push(scenario.clone());
            Ok::<_, ErrorObjectOwned>(scenario.id)
        })
        .await
    })?;
    module.register_async_method("mirage_runScenarioSet", |params, ctx, _| async move {
        let (set_id, mode): (String, RunMode) = params.parse().map_err(invalid_params)?;
        let job_id = {
            let mut state = ctx.state.write();
            let set = state
                .scenarios
                .get_mut(&set_id)
                .ok_or_else(|| rpc_error(MirageError::SetNotFound(set_id.clone())))?;
            set.status = ScenarioSetStatus::Running;
            let job_id = format!("job-{}", state.jobs.len() + 1);
            state.jobs.insert(
                job_id.clone(),
                ScenarioJob {
                    job_id: job_id.clone(),
                    set_id: set_id.clone(),
                    status: JobStatus::Running,
                    results: None,
                    total_wall_time_ms: None,
                },
            );
            job_id
        };
        let state = Arc::clone(&ctx.state);
        let job_id_for_task = job_id.clone();
        tokio::spawn(async move {
            let started = tokio::time::Instant::now();
            let set = { state.read().scenarios.get(&set_id).cloned() };
            if let Some(set) = set {
                let runner = ScenarioRunner::new(Arc::clone(&state));
                let results = match mode {
                    RunMode::Sequential => runner.run_sequential(&set).await,
                    RunMode::Parallel => runner.run_parallel(&set).await,
                };
                let mut state = state.write();
                if let Some(job) = state.jobs.get_mut(&job_id_for_task) {
                    job.status = JobStatus::Complete;
                    job.total_wall_time_ms =
                        Some(started.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
                    job.results = Some(results);
                }
                if let Some(set) = state.scenarios.get_mut(&set_id) {
                    set.status = ScenarioSetStatus::Complete;
                }
            }
        });
        Ok::<_, ErrorObjectOwned>(job_id)
    })?;
    module.register_async_method("mirage_getScenarioResults", |params, ctx, _| async move {
        let (job_id,): (String,) = params.parse().map_err(invalid_params)?;
        let state = ctx.state.read();
        let job = state
            .jobs
            .get(&job_id)
            .cloned()
            .ok_or_else(|| rpc_error(MirageError::JobNotFound(job_id.clone())))?;
        Ok::<_, ErrorObjectOwned>(job)
    })?;
    module.register_async_method("mirage_compareScenarios", |params, ctx, _| async move {
        let (job_id,): (String,) = params.parse().map_err(invalid_params)?;
        let state = ctx.state.read();
        let job = state
            .jobs
            .get(&job_id)
            .ok_or_else(|| rpc_error(MirageError::JobNotFound(job_id.clone())))?;
        Ok::<_, ErrorObjectOwned>(rank_scenario_results(
            job.results.clone().unwrap_or_default(),
        ))
    })?;
    module.register_async_method(
        "mirage_computeDomainSeparator",
        |params, ctx, _| async move {
            let (contract,): (Address,) = params.parse().map_err(invalid_params)?;
            let result = run_fork_snapshot(&ctx.state, false, move |fork| {
                EvmExecutor::call(
                    &fork,
                    Address::ZERO,
                    contract,
                    Bytes::from_static(&[0x36, 0x44, 0xe5, 0x15]),
                    U256::ZERO,
                    100_000,
                )
            })
            .await
            .map_err(rpc_error)?;
            Ok::<_, ErrorObjectOwned>(format!("0x{}", hex::encode(&result.output)))
        },
    )?;
    module.register_async_method("mirage_cleanup", |_params, _ctx, _| async {
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    module.register_async_method("mirage_shutdown", |_params, ctx, _| async move {
        let _ = ctx.shutdown.send(());
        Ok::<_, ErrorObjectOwned>(true)
    })?;
    Ok(())
}

async fn run_fork_snapshot<T, F>(
    state: &Arc<RwLock<MirageState>>,
    touch: bool,
    task: F,
) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(ForkState) -> Result<T> + Send + 'static,
{
    let fork = if touch {
        let mut state = state.write();
        touch_request(&mut state);
        state.fork.clone()
    } else {
        state.read().fork.clone()
    };
    tokio::task::spawn_blocking(move || task(fork))
        .await
        .map_err(|error| MirageError::BackgroundTask(error.to_string()))?
}

fn touch_request(state: &mut MirageState) {
    state.last_request_at = std::time::Instant::now();
    apply_resource_pressure(state);
}

fn apply_resource_pressure(state: &mut MirageState) {
    let usage = state.fork.resource_usage(&state.resource_model, state.mode);
    apply_pressure_action(state, usage.pressure_action());
}

fn apply_pressure_action(state: &mut MirageState, action: PressureAction) {
    match action {
        PressureAction::None => {
            state.reject_new_forks = false;
        }
        PressureAction::EvictCache => {
            state.reject_new_forks = false;
            let target_entries = state.resource_model.cache_capacity / 2;
            state.fork.db.evict_read_cache_to(target_entries);
        }
        PressureAction::Throttle => {
            state.reject_new_forks = true;
            let target_entries = state.resource_model.cache_capacity / 4;
            state.fork.db.evict_read_cache_to(target_entries);
            state
                .fork
                .db
                .dirty
                .watch_list
                .retain(|_, entry| matches!(entry.source, WatchSource::Manual));
        }
        PressureAction::DemoteToProxy => {
            state.reject_new_forks = true;
            state.fork.db.evict_read_cache_to(0);
            state
                .fork
                .db
                .dirty
                .watch_list
                .retain(|_, entry| matches!(entry.source, WatchSource::Manual));
            state.jobs.clear();
            state.scenarios.clear();
            state.mode = MirageMode::Proxy;
        }
    }
}

fn register_event_subscription(state: &Arc<RwLock<MirageState>>, filter: EventFilter) -> String {
    let mut state = state.write();
    state.last_request_at = std::time::Instant::now();
    state.next_event_subscription_id = state.next_event_subscription_id.saturating_add(1);
    let stream_id = format!("stream-{}", state.next_event_subscription_id);
    state.event_subscriptions.insert(stream_id.clone(), filter);
    stream_id
}

fn publish_receipt_events(state: &MirageState, receipt: &LocalReceipt) {
    for log in &receipt.logs {
        let event = MirageEvent {
            block_number: receipt.block_number,
            tx_hash: receipt.transaction_hash,
            log_index: log.log_index,
            contract: log.address,
            topics: log.topics.clone(),
            data: log.data.clone(),
            source: EventSource::LocalTx,
            decoded: None,
        };
        let _ = state.event_bus.send(event);
    }
}

async fn health_handler(State(state): State<Arc<RwLock<MirageState>>>) -> impl IntoResponse {
    let state = state.read();
    axum::Json(state.fork.status(state.mode))
}

async fn event_ws_handler(
    Path(stream_id): Path<String>,
    State(state): State<Arc<RwLock<MirageState>>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let filter = {
        let state = state.read();
        state.event_subscriptions.get(&stream_id).cloned()
    };
    filter.map_or_else(
        || StatusCode::NOT_FOUND.into_response(),
        |filter| ws.on_upgrade(move |socket| handle_event_socket(socket, state, filter)),
    )
}

async fn unsubscribe_event_handler(
    Path(stream_id): Path<String>,
    State(state): State<Arc<RwLock<MirageState>>>,
) -> impl IntoResponse {
    let removed = state.write().event_subscriptions.remove(&stream_id).is_some();
    axum::Json(removed)
}

async fn handle_event_socket(
    mut socket: WebSocket,
    state: Arc<RwLock<MirageState>>,
    filter: EventFilter,
) {
    let mut receiver = state.read().event_bus.subscribe();
    while let Ok(event) = receiver.recv().await {
        if !event_matches_filter(&event, &filter) {
            continue;
        }
        let payload = match serde_json::to_string(&event) {
            Ok(payload) => payload,
            Err(error) => {
                tracing::warn!("failed to serialize mirage event: {error}");
                continue;
            }
        };
        if socket.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}

fn event_matches_filter(event: &MirageEvent, filter: &EventFilter) -> bool {
    let address_match = filter
        .addresses
        .as_ref()
        .is_none_or(|addresses| addresses.contains(&event.contract));
    let topic_match = filter
        .topics
        .as_ref()
        .is_none_or(|topics| event.topics.iter().any(|topic| topics.contains(topic)));
    address_match && topic_match
}

fn extract_to(kind: Option<Address>) -> Option<Address> {
    kind
}

fn receipt_json(receipt: &LocalReceipt) -> serde_json::Value {
    serde_json::json!({
        "transactionHash": receipt.transaction_hash,
        "blockHash": receipt.block_hash,
        "blockNumber": hex_u64(receipt.block_number),
        "from": receipt.from,
        "to": receipt.to,
        "gasUsed": hex_u64(receipt.gas_used),
        "status": if receipt.success { "0x1" } else { "0x0" },
        "logs": receipt.logs,
    })
}

fn transaction_json(tx: &LocalTransaction) -> serde_json::Value {
    serde_json::json!({
        "hash": tx.hash,
        "from": tx.from,
        "to": tx.to,
        "value": hex_u256(tx.value),
        "input": format!("0x{}", hex::encode(&tx.input)),

        "gas": hex_u64(tx.gas),
        "nonce": hex_u64(tx.nonce),
        "blockNumber": hex_u64(tx.block_number),
    })
}

fn block_json(block: &LocalBlock) -> serde_json::Value {
    serde_json::json!({
        "hash": block.hash,
        "number": hex_u64(block.number),
        "timestamp": hex_u64(block.timestamp),
        "transactions": block.transactions,
    })
}

fn hex_u64(value: u64) -> String {
    format!("0x{value:x}")
}

fn hex_u256(value: U256) -> String {
    format!("0x{value:x}")
}

fn parse_hex_quantity(value: &str) -> Result<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16)
        .map_err(|error| MirageError::InvalidParams(format!("invalid hex quantity: {error}")))
}

#[derive(Debug, Clone)]
struct CommittedTransaction {
    tx_hash: B256,
    block_number: u64,
    diff: crate::replay::StateDiff,
    transaction: LocalTransaction,
    receipt: LocalReceipt,
    block: LocalBlock,
}

async fn commit_transaction_request(
    state: &Arc<RwLock<MirageState>>,
    request: TransactionRequest,
    override_hash: Option<B256>,
) -> Result<B256> {
    let fork = {
        let _writer_guard = lock_state_writes(state).await;
        let mut state = state.write();
        touch_request(&mut state);
        state.fork.clone()
    };
    let committed = tokio::task::spawn_blocking(move || {
        commit_transaction_on_snapshot(fork, request, override_hash)
    })
    .await
    .map_err(|error| MirageError::BackgroundTask(error.to_string()))??;
    let receipt = committed.receipt.clone();
    let classifier = DiffClassifier::new(ClassificationConfig::default());
    let _writer_guard = lock_state_writes(state).await;
    let mut state = state.write();
    state.fork.commit_local_transaction(
        &committed.diff,
        committed.transaction,
        committed.receipt,
        committed.block,
    );
    classifier.apply(
        &mut state.fork.db.dirty,
        &committed.diff,
        committed.block_number,
    )?;
    publish_receipt_events(&state, &receipt);
    Ok(committed.tx_hash)
}

fn commit_transaction_on_snapshot(
    mut fork: ForkState,
    request: TransactionRequest,
    override_hash: Option<B256>,
) -> Result<CommittedTransaction> {
    let from = request
        .from
        .ok_or_else(|| MirageError::InvalidParams("missing from".to_owned()))?;
    let to = extract_to(request.to);
    let data = request.data.unwrap_or_default();
    let value = request.value.unwrap_or(U256::ZERO);
    let gas = request.gas.unwrap_or(21_000);
    let (_result, diff) = EvmExecutor::transact(&mut fork, from, to, data, value, gas)?;
    let current_hash = latest_local_tx_hash(&fork)?;
    let tx_hash = if let Some(expected_hash) = override_hash {
        adopt_latest_transaction_hash(&mut fork, current_hash, expected_hash)?;
        expected_hash
    } else {
        current_hash
    };
    let transaction = fork
        .transactions
        .get(&tx_hash)
        .cloned()
        .ok_or_else(|| MirageError::Unsupported("missing tx after commit".to_owned()))?;
    let receipt = fork
        .receipts
        .get(&tx_hash)
        .cloned()
        .ok_or_else(|| MirageError::Unsupported("missing receipt after commit".to_owned()))?;
    let block = fork
        .blocks_by_number
        .get(&transaction.block_number)
        .cloned()
        .ok_or_else(|| MirageError::Unsupported("missing block after commit".to_owned()))?;

    Ok(CommittedTransaction {
        tx_hash,
        block_number: transaction.block_number,
        diff,
        transaction,
        receipt,
        block,
    })
}

fn latest_local_tx_hash(state: &ForkState) -> Result<B256> {
    state
        .transactions
        .iter()
        .max_by_key(|(_, tx)| tx.block_number)
        .map(|(hash, _)| *hash)
        .ok_or_else(|| MirageError::Unsupported("missing tx after commit".to_owned()))
}

fn adopt_latest_transaction_hash(
    state: &mut ForkState,
    current_hash: B256,
    new_hash: B256,
) -> Result<()> {
    if current_hash == new_hash {
        return Ok(());
    }

    let mut tx = state.transactions.remove(&current_hash).ok_or_else(|| {
        MirageError::Unsupported("latest transaction missing from store".to_owned())
    })?;
    tx.hash = new_hash;
    let block_number = tx.block_number;
    state.transactions.insert(new_hash, tx);

    let mut receipt = state
        .receipts
        .remove(&current_hash)
        .ok_or_else(|| MirageError::Unsupported("latest receipt missing from store".to_owned()))?;
    receipt.transaction_hash = new_hash;
    state.receipts.insert(new_hash, receipt);

    if let Some(block) = state.blocks_by_number.get_mut(&block_number) {
        for hash in &mut block.transactions {
            if *hash == current_hash {
                *hash = new_hash;
            }
        }
    }
    if let Some(block) = state
        .blocks_by_hash
        .values_mut()
        .find(|block| block.number == block_number)
    {
        for hash in &mut block.transactions {
            if *hash == current_hash {
                *hash = new_hash;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct DecodedRawTransaction {
    tx_hash: B256,
    request: TransactionRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RlpValue {
    Bytes(Vec<u8>),
    List(Vec<Self>),
}

fn decode_signed_raw_transaction(raw: &Bytes) -> Result<DecodedRawTransaction> {
    if raw.is_empty() {
        return Err(MirageError::InvalidParams(
            "raw transaction is empty".to_owned(),
        ));
    }

    let tx_hash = keccak256(raw);
    match raw[0] {
        0x01 => decode_typed_raw_transaction(0x01, &raw[1..], tx_hash),
        0x02 => decode_typed_raw_transaction(0x02, &raw[1..], tx_hash),
        0x03 => decode_typed_raw_transaction(0x03, &raw[1..], tx_hash),
        _ => decode_legacy_raw_transaction(raw, tx_hash),
    }
}

fn decode_legacy_raw_transaction(raw: &[u8], tx_hash: B256) -> Result<DecodedRawTransaction> {
    let fields = rlp_decode_top_list(raw)?;
    if fields.len() != 9 {
        return Err(MirageError::InvalidParams(format!(
            "legacy transaction must have 9 fields, found {}",
            fields.len()
        )));
    }

    let nonce = rlp_u64(&fields[0])?;
    let gas_price = rlp_u128(&fields[1])?;
    let gas = rlp_u64(&fields[2])?;
    let to = rlp_address_opt(&fields[3])?;
    let value = rlp_u256(&fields[4])?;
    let data = Bytes::from(rlp_bytes(&fields[5])?);
    let v = rlp_u64(&fields[6])?;
    let r = rlp_u256(&fields[7])?;
    let s = rlp_u256(&fields[8])?;
    let (chain_id, recovery_id) = decode_legacy_v(v)?;

    let mut signing_fields = fields[..6].to_vec();
    if let Some(chain_id) = chain_id {
        signing_fields.push(rlp_from_u64(chain_id));
        signing_fields.push(RlpValue::Bytes(Vec::new()));
        signing_fields.push(RlpValue::Bytes(Vec::new()));
    }
    let signing_hash = keccak256(rlp_encode(&RlpValue::List(signing_fields)));
    let from = recover_address(signing_hash, r, s, recovery_id)?;

    Ok(DecodedRawTransaction {
        tx_hash,
        request: TransactionRequest {
            from: Some(from),
            to,
            gas: Some(gas),
            value: Some(value),
            data: Some(data),
            gas_price: Some(gas_price),
            nonce: Some(nonce),
            chain_id,
        },
    })
}

fn decode_typed_raw_transaction(
    tx_type: u8,
    raw: &[u8],
    tx_hash: B256,
) -> Result<DecodedRawTransaction> {
    let fields = rlp_decode_top_list(raw)?;
    let (chain_id, nonce, gas_price, gas, to, value, data, signing_fields, recovery_id, r, s) =
        match tx_type {
            0x01 => {
                if fields.len() != 11 {
                    return Err(MirageError::InvalidParams(format!(
                        "type 1 transaction must have 11 fields, found {}",
                        fields.len()
                    )));
                }
                (
                    Some(rlp_u64(&fields[0])?),
                    Some(rlp_u64(&fields[1])?),
                    Some(rlp_u128(&fields[2])?),
                    Some(rlp_u64(&fields[3])?),
                    rlp_address_opt(&fields[4])?,
                    Some(rlp_u256(&fields[5])?),
                    Bytes::from(rlp_bytes(&fields[6])?),
                    fields[..8].to_vec(),
                    rlp_u8(&fields[8])?,
                    rlp_u256(&fields[9])?,
                    rlp_u256(&fields[10])?,
                )
            }
            0x02 => {
                if fields.len() != 12 {
                    return Err(MirageError::InvalidParams(format!(
                        "type 2 transaction must have 12 fields, found {}",
                        fields.len()
                    )));
                }
                (
                    Some(rlp_u64(&fields[0])?),
                    Some(rlp_u64(&fields[1])?),
                    Some(rlp_u128(&fields[3])?),
                    Some(rlp_u64(&fields[4])?),
                    rlp_address_opt(&fields[5])?,
                    Some(rlp_u256(&fields[6])?),
                    Bytes::from(rlp_bytes(&fields[7])?),
                    fields[..9].to_vec(),
                    rlp_u8(&fields[9])?,
                    rlp_u256(&fields[10])?,
                    rlp_u256(&fields[11])?,
                )
            }
            0x03 => {
                if fields.len() != 14 {
                    return Err(MirageError::InvalidParams(format!(
                        "type 3 transaction must have 14 fields, found {}",
                        fields.len()
                    )));
                }
                (
                    Some(rlp_u64(&fields[0])?),
                    Some(rlp_u64(&fields[1])?),
                    Some(rlp_u128(&fields[3])?),
                    Some(rlp_u64(&fields[4])?),
                    rlp_address_opt(&fields[5])?,
                    Some(rlp_u256(&fields[6])?),
                    Bytes::from(rlp_bytes(&fields[7])?),
                    fields[..11].to_vec(),
                    rlp_u8(&fields[11])?,
                    rlp_u256(&fields[12])?,
                    rlp_u256(&fields[13])?,
                )
            }
            _ => {
                return Err(MirageError::InvalidParams(format!(
                    "unsupported typed transaction {tx_type:#x}"
                )));
            }
        };

    let mut payload = vec![tx_type];
    payload.extend_from_slice(&rlp_encode(&RlpValue::List(signing_fields)));
    let signing_hash = keccak256(payload);
    let from = recover_address(signing_hash, r, s, recovery_id)?;

    Ok(DecodedRawTransaction {
        tx_hash,
        request: TransactionRequest {
            from: Some(from),
            to,
            gas,
            value,
            data: Some(data),
            gas_price,
            nonce,
            chain_id,
        },
    })
}

fn decode_legacy_v(v: u64) -> Result<(Option<u64>, u8)> {
    match v {
        27 => Ok((None, 0)),
        28 => Ok((None, 1)),
        value if value >= 35 => {
            let adjusted = value - 35;
            let parity = u8::from(adjusted % 2 != 0);
            Ok((Some(adjusted / 2), parity))
        }
        _ => Err(MirageError::InvalidParams(format!(
            "invalid legacy v value {v}"
        ))),
    }
}

fn recover_address(signing_hash: B256, r: U256, s: U256, recovery_id: u8) -> Result<Address> {
    let recovery_id = RecoveryId::from_byte(recovery_id)
        .ok_or_else(|| MirageError::InvalidParams(format!("invalid recovery id {recovery_id}")))?;
    let signature =
        Signature::from_scalars(r.to_be_bytes::<32>(), s.to_be_bytes::<32>()).map_err(|error| {
            MirageError::InvalidParams(format!("invalid signature scalars: {error}"))
        })?;
    let verifying_key =
        VerifyingKey::recover_from_prehash(signing_hash.as_slice(), &signature, recovery_id)
            .map_err(|error| {
                MirageError::InvalidParams(format!("failed to recover sender: {error}"))
            })?;
    let encoded = verifying_key.to_encoded_point(false);
    let hash = keccak256(&encoded.as_bytes()[1..]);
    Ok(Address::from_slice(&hash.as_slice()[12..]))
}

fn rlp_decode_top_list(input: &[u8]) -> Result<Vec<RlpValue>> {
    let (value, consumed) = rlp_decode(input)?;
    if consumed != input.len() {
        return Err(MirageError::InvalidParams(
            "unexpected trailing RLP bytes".to_owned(),
        ));
    }
    match value {
        RlpValue::List(fields) => Ok(fields),
        RlpValue::Bytes(_) => Err(MirageError::InvalidParams(
            "expected top-level RLP list".to_owned(),
        )),
    }
}

fn rlp_decode(input: &[u8]) -> Result<(RlpValue, usize)> {
    let first = *input
        .first()
        .ok_or_else(|| MirageError::InvalidParams("unexpected end of RLP input".to_owned()))?;
    match first {
        0x00..=0x7f => Ok((RlpValue::Bytes(vec![first]), 1)),
        0x80..=0xb7 => {
            let len = usize::from(first - 0x80);
            let end = 1 + len;
            let bytes = input
                .get(1..end)
                .ok_or_else(|| MirageError::InvalidParams("short RLP string".to_owned()))?;
            Ok((RlpValue::Bytes(bytes.to_vec()), end))
        }
        0xb8..=0xbf => {
            let len_of_len = usize::from(first - 0xb7);
            let len = rlp_len(input, 1, len_of_len)?;
            let start = 1 + len_of_len;
            let end = start + len;
            let bytes = input
                .get(start..end)
                .ok_or_else(|| MirageError::InvalidParams("short long RLP string".to_owned()))?;
            Ok((RlpValue::Bytes(bytes.to_vec()), end))
        }
        0xc0..=0xf7 => {
            let len = usize::from(first - 0xc0);
            let start = 1;
            let end = start + len;
            let payload = input
                .get(start..end)
                .ok_or_else(|| MirageError::InvalidParams("short RLP list".to_owned()))?;
            Ok((RlpValue::List(rlp_decode_list_payload(payload)?), end))
        }
        0xf8..=0xff => {
            let len_of_len = usize::from(first - 0xf7);
            let len = rlp_len(input, 1, len_of_len)?;
            let start = 1 + len_of_len;
            let end = start + len;
            let payload = input
                .get(start..end)
                .ok_or_else(|| MirageError::InvalidParams("short long RLP list".to_owned()))?;
            Ok((RlpValue::List(rlp_decode_list_payload(payload)?), end))
        }
    }
}

fn rlp_decode_list_payload(mut payload: &[u8]) -> Result<Vec<RlpValue>> {
    let mut values = Vec::new();
    while !payload.is_empty() {
        let (value, consumed) = rlp_decode(payload)?;
        values.push(value);
        payload = &payload[consumed..];
    }
    Ok(values)
}

fn rlp_len(input: &[u8], start: usize, len_of_len: usize) -> Result<usize> {
    let end = start + len_of_len;
    let bytes = input
        .get(start..end)
        .ok_or_else(|| MirageError::InvalidParams("short RLP length".to_owned()))?;
    bytes.iter().try_fold(0_usize, |acc, byte| {
        acc.checked_mul(256)
            .and_then(|value| value.checked_add(usize::from(*byte)))
            .ok_or_else(|| MirageError::InvalidParams("RLP length overflow".to_owned()))
    })
}

fn rlp_bytes(value: &RlpValue) -> Result<Vec<u8>> {
    match value {
        RlpValue::Bytes(bytes) => Ok(bytes.clone()),
        RlpValue::List(_) => Err(MirageError::InvalidParams("expected RLP bytes".to_owned())),
    }
}

fn rlp_u64(value: &RlpValue) -> Result<u64> {
    let bytes = rlp_bytes(value)?;
    if bytes.is_empty() {
        return Ok(0);
    }
    if bytes.len() > 8 {
        return Err(MirageError::InvalidParams(
            "integer does not fit in u64".to_owned(),
        ));
    }
    Ok(bytes
        .into_iter()
        .fold(0_u64, |acc, byte| (acc << 8) | u64::from(byte)))
}

fn rlp_u128(value: &RlpValue) -> Result<u128> {
    let bytes = rlp_bytes(value)?;
    if bytes.is_empty() {
        return Ok(0);
    }
    if bytes.len() > 16 {
        return Err(MirageError::InvalidParams(
            "integer does not fit in u128".to_owned(),
        ));
    }
    Ok(bytes
        .into_iter()
        .fold(0_u128, |acc, byte| (acc << 8) | u128::from(byte)))
}

fn rlp_u8(value: &RlpValue) -> Result<u8> {
    let value = rlp_u64(value)?;
    u8::try_from(value)
        .map_err(|_| MirageError::InvalidParams(format!("integer does not fit in u8: {value}")))
}

fn rlp_u256(value: &RlpValue) -> Result<U256> {
    Ok(U256::from_be_slice(&rlp_bytes(value)?))
}

fn rlp_address_opt(value: &RlpValue) -> Result<Option<Address>> {
    let bytes = rlp_bytes(value)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() != 20 {
        return Err(MirageError::InvalidParams(format!(
            "address must be 20 bytes, found {}",
            bytes.len()
        )));
    }
    Ok(Some(Address::from_slice(&bytes)))
}

fn rlp_from_u64(value: u64) -> RlpValue {
    if value == 0 {
        return RlpValue::Bytes(Vec::new());
    }
    RlpValue::Bytes(trim_leading_zeros(value.to_be_bytes().to_vec()))
}

fn rlp_encode(value: &RlpValue) -> Vec<u8> {
    match value {
        RlpValue::Bytes(bytes) => rlp_encode_bytes(bytes),
        RlpValue::List(items) => {
            let payload = items.iter().flat_map(rlp_encode).collect::<Vec<_>>();
            rlp_encode_with_offset(&payload, 0xc0, 0xf7)
        }
    }
}

fn rlp_encode_bytes(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() == 1 && bytes[0] < 0x80 {
        vec![bytes[0]]
    } else {
        rlp_encode_with_offset(bytes, 0x80, 0xb7)
    }
}

fn rlp_encode_with_offset(payload: &[u8], short_offset: u8, long_offset: u8) -> Vec<u8> {
    if payload.len() <= 55 {
        let mut encoded = Vec::with_capacity(1 + payload.len());
        let short_len = u8::try_from(payload.len())
            .unwrap_or_else(|_| unreachable!("short RLP payload length always fits in u8"));
        encoded.push(short_offset + short_len);
        encoded.extend_from_slice(payload);
        encoded
    } else {
        let length_bytes = trim_leading_zeros((payload.len() as u64).to_be_bytes().to_vec());
        let mut encoded = Vec::with_capacity(1 + length_bytes.len() + payload.len());
        let length_of_length = u8::try_from(length_bytes.len())
            .unwrap_or_else(|_| unreachable!("RLP length-of-length always fits in u8"));
        encoded.push(long_offset + length_of_length);
        encoded.extend_from_slice(&length_bytes);
        encoded.extend_from_slice(payload);
        encoded
    }
}

fn trim_leading_zeros(mut bytes: Vec<u8>) -> Vec<u8> {
    let first_non_zero = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len());
    bytes.drain(..first_non_zero);
    bytes
}

fn invalid_params(error: impl std::fmt::Display) -> ErrorObjectOwned {
    invalid_params_message(error.to_string())
}

fn invalid_params_message(message: impl Into<String>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32602, message.into(), None::<()>)
}

fn rpc_error(error: MirageError) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(error.rpc_code(), error.to_string(), None::<()>)
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, sync::Arc, time::Duration};

    use alloy_primitives::{Address, B256, Bytes, U256, address, keccak256};
    use k256::{
        FieldBytes,
        ecdsa::{SigningKey, hazmat::SignPrimitive},
        sha2,
    };
    use tokio::{sync::broadcast, time::sleep};

    use super::{
        RlpValue, ServerContext, apply_pressure_action, commit_transaction_request,
        decode_signed_raw_transaction, rlp_encode, rlp_from_u64, run_fork_snapshot,
    };
    use crate::{
        TransactionRequest,
        fork::{ForkState, HybridDB, MirageFork, WatchEntry, WatchSource, with_state_write},
        provider::UpstreamRpc,
        resources::{MirageMode, PressureAction, Profile, ResourceModel},
    };

    #[test]
    fn decode_raw_legacy_transaction() {
        let signing_key = SigningKey::from_bytes((&[7_u8; 32]).into())
            .unwrap_or_else(|error| panic!("signing key: {error}"));
        let from = signing_key_address(&signing_key);
        let to = address!("0x1000000000000000000000000000000000000001");
        let raw = sign_legacy(&signing_key, 1, 5, 21_000, Some(to), 9, &[0xde, 0xad]);
        let decoded = decode_signed_raw_transaction(&raw)
            .unwrap_or_else(|error| panic!("decode legacy: {error}"));
        assert_transaction(
            decoded.request,
            from,
            Some(to),
            21_000,
            9,
            &[0xde, 0xad],
            Some(1),
            Some(5),
        );
    }

    #[test]
    fn decode_raw_typed_transactions() {
        let signing_key = SigningKey::from_bytes((&[9_u8; 32]).into())
            .unwrap_or_else(|error| panic!("signing key: {error}"));
        let from = signing_key_address(&signing_key);
        let to = address!("0x2000000000000000000000000000000000000002");

        let type1 = sign_typed(&signing_key, 0x01, Some(to));
        let decoded1 = decode_signed_raw_transaction(&type1)
            .unwrap_or_else(|error| panic!("decode type1: {error}"));
        assert_transaction(
            decoded1.request,
            from,
            Some(to),
            80_000,
            11,
            &[0x01, 0x02],
            Some(1),
            Some(3),
        );

        let type2 = sign_typed(&signing_key, 0x02, Some(to));
        let decoded2 = decode_signed_raw_transaction(&type2)
            .unwrap_or_else(|error| panic!("decode type2: {error}"));
        assert_transaction(
            decoded2.request,
            from,
            Some(to),
            80_000,
            11,
            &[0x01, 0x02],
            Some(1),
            Some(4),
        );

        let type3 = sign_typed(&signing_key, 0x03, None);
        let decoded3 = decode_signed_raw_transaction(&type3)
            .unwrap_or_else(|error| panic!("decode type3: {error}"));
        assert_transaction(
            decoded3.request,
            from,
            None,
            80_000,
            11,
            &[0x01, 0x02],
            Some(1),
            Some(4),
        );
    }

    #[tokio::test]
    async fn run_fork_snapshot_reads_dirty_state_without_holding_server_lock() {
        let address = address!("0x3000000000000000000000000000000000000003");
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let (shutdown, _) = broadcast::channel(1);
        let context = ServerContext {
            state: mirage.state(),
            shutdown,
        };
        context
            .state
            .write()
            .fork
            .db
            .set_balance(address, U256::from(42_u64));

        let observed = run_fork_snapshot(&context.state, true, move |mut fork| {
            Ok(fork.db.basic(address)?.unwrap_or_default().balance)
        })
        .await
        .unwrap_or_else(|error| panic!("snapshot read succeeds: {error}"));

        assert_eq!(observed, U256::from(42_u64));
    }

    #[tokio::test]
    async fn commit_transaction_request_runs_without_holding_state_write_lock() {
        let from = address!("0x3100000000000000000000000000000000000001");
        let to = address!("0x3100000000000000000000000000000000000002");
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

        let state = mirage.state();
        let request = TransactionRequest {
            from: Some(from),
            to: Some(to),
            gas: Some(21_000),
            value: Some(U256::from(1_u64)),
            data: None,
            gas_price: None,
            nonce: None,
            chain_id: None,
        };

        let state_for_task = Arc::clone(&state);
        let task = tokio::spawn(async move {
            commit_transaction_request(&state_for_task, request, None).await
        });
        sleep(Duration::from_millis(20)).await;

        let started = std::time::Instant::now();
        let read_guard = state.read();
        assert!(started.elapsed() < Duration::from_millis(75));
        drop(read_guard);

        let tx_hash = task
            .await
            .unwrap_or_else(|error| panic!("join tx task: {error}"))
            .unwrap_or_else(|error| panic!("commit transaction: {error}"));
        assert_ne!(tx_hash, B256::ZERO);
    }

    #[tokio::test]
    async fn commit_transaction_request_releases_writer_gate_during_blocking_execution() {
        let from = address!("0x3200000000000000000000000000000000000001");
        let to = address!("0x3200000000000000000000000000000000000002");
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

        let state = mirage.state();
        let request = TransactionRequest {
            from: Some(from),
            to: Some(to),
            gas: Some(21_000),
            value: Some(U256::from(1_u64)),
            data: None,
            gas_price: None,
            nonce: None,
            chain_id: None,
        };

        let state_for_task = Arc::clone(&state);
        let task = tokio::spawn(async move {
            commit_transaction_request(&state_for_task, request, None).await
        });
        sleep(Duration::from_millis(20)).await;

        let started = std::time::Instant::now();
        with_state_write(&state, |state| {
            state.reject_new_forks = !state.reject_new_forks;
        })
        .await;
        assert!(started.elapsed() < Duration::from_millis(75));

        let tx_hash = task
            .await
            .unwrap_or_else(|error| panic!("join tx task: {error}"))
            .unwrap_or_else(|error| panic!("commit transaction: {error}"));
        assert_ne!(tx_hash, B256::ZERO);
    }

    #[test]
    fn throttle_pressure_rejects_new_forks_and_keeps_manual_watches() {
        let upstream = Arc::new(UpstreamRpc::mock(1));
        let db = HybridDB::new(upstream, 32, Duration::from_secs(12), NonZeroUsize::MIN, 1);
        let fork = ForkState::new(db, 0, 1);
        let mirage = MirageFork::new(
            fork,
            ResourceModel::for_profile(Profile::Standard, Duration::from_secs(12)),
            MirageMode::Live,
        );
        let state_handle = mirage.state();
        let mut state = state_handle.write();
        let manual = address!("0x4000000000000000000000000000000000000004");
        let auto = address!("0x5000000000000000000000000000000000000005");
        state.fork.db.dirty.watch_list.insert(
            manual,
            WatchEntry {
                source: WatchSource::Manual,
                added_at_block: 0,
                initial_slot_count: 0,
                replay_count: 0,
            },
        );
        state.fork.db.dirty.watch_list.insert(
            auto,
            WatchEntry {
                source: WatchSource::AutoClassified,
                added_at_block: 0,
                initial_slot_count: 0,
                replay_count: 0,
            },
        );
        state.reject_new_forks = false;

        apply_pressure_action(&mut state, PressureAction::Throttle);

        assert!(state.reject_new_forks);
        assert!(state.fork.db.dirty.watch_list.contains_key(&manual));
        assert!(!state.fork.db.dirty.watch_list.contains_key(&auto));
        assert_eq!(state.mode, MirageMode::Live);
    }

    fn assert_transaction(
        request: TransactionRequest,
        from: Address,
        to: Option<Address>,
        gas: u64,
        value: u64,
        data: &[u8],
        chain_id: Option<u64>,
        gas_price: Option<u128>,
    ) {
        assert_eq!(request.from, Some(from));
        assert_eq!(request.to, to);
        assert_eq!(request.gas, Some(gas));
        assert_eq!(request.value, Some(U256::from(value)));
        assert_eq!(request.data.as_ref().map(Bytes::as_ref), Some(data));
        assert_eq!(request.chain_id, chain_id);
        assert_eq!(request.gas_price, gas_price);
    }

    fn signing_key_address(signing_key: &SigningKey) -> Address {
        let encoded = signing_key.verifying_key().to_encoded_point(false);
        let hash = keccak256(&encoded.as_bytes()[1..]);
        Address::from_slice(&hash.as_slice()[12..])
    }

    fn sign_legacy(
        signing_key: &SigningKey,
        chain_id: u64,
        gas_price: u64,
        gas_limit: u64,
        to: Option<Address>,
        value: u64,
        data: &[u8],
    ) -> Bytes {
        let unsigned = RlpValue::List(vec![
            rlp_from_u64(0),
            rlp_from_u64(gas_price),
            rlp_from_u64(gas_limit),
            to.map_or_else(
                || RlpValue::Bytes(Vec::new()),
                |value| RlpValue::Bytes(value.as_slice().to_vec()),
            ),
            rlp_from_u64(value),
            RlpValue::Bytes(data.to_vec()),
            rlp_from_u64(chain_id),
            RlpValue::Bytes(Vec::new()),
            RlpValue::Bytes(Vec::new()),
        ]);
        let unsigned_rlp = rlp_encode(&unsigned);
        let hash = keccak256(&unsigned_rlp);
        let mut field_bytes = FieldBytes::default();
        field_bytes.copy_from_slice(hash.as_slice());
        let (signature, recovery_id) = signing_key
            .as_nonzero_scalar()
            .try_sign_prehashed_rfc6979::<sha2::Sha256>(&field_bytes, &[])
            .unwrap_or_else(|error| panic!("sign legacy prehash: {error}"));
        let recovery_id = recovery_id.unwrap_or_else(|| panic!("legacy recovery id present"));
        let v = chain_id * 2 + 35 + u64::from(recovery_id.to_byte());
        let signed = RlpValue::List(vec![
            rlp_from_u64(0),
            rlp_from_u64(gas_price),
            rlp_from_u64(gas_limit),
            to.map_or_else(
                || RlpValue::Bytes(Vec::new()),
                |value| RlpValue::Bytes(value.as_slice().to_vec()),
            ),
            rlp_from_u64(value),
            RlpValue::Bytes(data.to_vec()),
            rlp_from_u64(v),
            RlpValue::Bytes(signature.r().to_bytes().to_vec()),
            RlpValue::Bytes(signature.s().to_bytes().to_vec()),
        ]);
        Bytes::from(rlp_encode(&signed))
    }

    fn sign_typed(signing_key: &SigningKey, tx_type: u8, to: Option<Address>) -> Bytes {
        let unsigned = match tx_type {
            0x01 => RlpValue::List(vec![
                rlp_from_u64(1),
                rlp_from_u64(0),
                rlp_from_u64(3),
                rlp_from_u64(80_000),
                to.map_or_else(
                    || RlpValue::Bytes(Vec::new()),
                    |value| RlpValue::Bytes(value.as_slice().to_vec()),
                ),
                rlp_from_u64(11),
                RlpValue::Bytes(vec![0x01, 0x02]),
                RlpValue::List(Vec::new()),
            ]),
            0x02 => RlpValue::List(vec![
                rlp_from_u64(1),
                rlp_from_u64(0),
                rlp_from_u64(1),
                rlp_from_u64(4),
                rlp_from_u64(80_000),
                to.map_or_else(
                    || RlpValue::Bytes(Vec::new()),
                    |value| RlpValue::Bytes(value.as_slice().to_vec()),
                ),
                rlp_from_u64(11),
                RlpValue::Bytes(vec![0x01, 0x02]),
                RlpValue::List(Vec::new()),
            ]),
            0x03 => RlpValue::List(vec![
                rlp_from_u64(1),
                rlp_from_u64(0),
                rlp_from_u64(1),
                rlp_from_u64(4),
                rlp_from_u64(80_000),
                to.map_or_else(
                    || RlpValue::Bytes(Vec::new()),
                    |value| RlpValue::Bytes(value.as_slice().to_vec()),
                ),
                rlp_from_u64(11),
                RlpValue::Bytes(vec![0x01, 0x02]),
                RlpValue::List(Vec::new()),
                rlp_from_u64(5),
                RlpValue::List(Vec::new()),
            ]),
            _ => panic!("unsupported tx type"),
        };
        let unsigned_rlp = rlp_encode(&unsigned);
        let mut payload = vec![tx_type];
        payload.extend_from_slice(&unsigned_rlp);
        let hash = keccak256(payload);
        let mut field_bytes = FieldBytes::default();
        field_bytes.copy_from_slice(hash.as_slice());
        let (signature, recovery_id) = signing_key
            .as_nonzero_scalar()
            .try_sign_prehashed_rfc6979::<sha2::Sha256>(&field_bytes, &[])
            .unwrap_or_else(|error| panic!("sign typed prehash: {error}"));
        let recovery_id = recovery_id.unwrap_or_else(|| panic!("typed recovery id present"));

        let mut fields = match unsigned {
            RlpValue::List(fields) => fields,
            _ => unreachable!(),
        };
        fields.push(rlp_from_u64(u64::from(recovery_id.to_byte())));
        fields.push(RlpValue::Bytes(signature.r().to_bytes().to_vec()));
        fields.push(RlpValue::Bytes(signature.s().to_bytes().to_vec()));
        let mut encoded = vec![tx_type];
        encoded.extend_from_slice(&rlp_encode(&RlpValue::List(fields)));
        Bytes::from(encoded)
    }
}
