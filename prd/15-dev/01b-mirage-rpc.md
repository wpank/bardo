# mirage-rs RPC Reference

**Version:** 2.0.0
**Last Updated:** 2026-03-18
**Crate:** `mirage-rs/`

---

> **Reader orientation:** This document is the complete JSON-RPC method reference for mirage-rs, Bardo's in-process EVM fork (section 15). It covers standard `eth_*` methods with mirage-specific behavior, Hardhat/Anvil compatibility methods, `evm_*` test utilities, and `mirage_*` extensions for state manipulation, watch list management, resource control, and the scenario runner. The key concept is that mirage-rs speaks the same JSON-RPC protocol as Anvil, so existing tools (viem, wagmi, Foundry cast) connect without changes, while `mirage_*` extensions expose the targeted-replay and dirty-tracking features unique to mirage-rs. See `prd2/shared/glossary.md` for full term definitions.

mirage-rs v2 exposes a JSON-RPC 2.0 server on a configurable local port (default: 8545). It supports standard `eth_*` methods with mirage-specific behavior, Hardhat/Anvil compatibility methods, `evm_*` test utilities, and `mirage_*` extensions for state manipulation, watch list management, resource control, and the scenario runner.

All methods use standard JSON-RPC 2.0 request/response format.

```json
{ "jsonrpc": "2.0", "method": "eth_blockNumber", "params": [], "id": 1 }
{ "jsonrpc": "2.0", "result": "0x12a05f0", "id": 1 }
```

---

## Standard eth_* Methods

### eth_blockNumber

Returns the local block counter. In live mode, starts at the mainnet block number at startup and increments with each `eth_sendTransaction`. In historical mode, tracks the current replay block.

**Parameters:** none

**Returns:** `QUANTITY` -- current local block number (hex)

### eth_chainId

Returns the chain ID mirage was started with.

**Parameters:** none

**Returns:** `QUANTITY` -- chain ID

### eth_getBalance

Returns the ETH balance of an address. Reads from DirtyStore if overridden; otherwise reads from upstream at latest (or pinned block in historical mode).

**Parameters:**
1. `ADDRESS` -- account address
2. `BLOCK_TAG` -- accepted for compatibility but all resolve to current local state

**Returns:** `QUANTITY` -- balance in wei

### eth_getStorageAt

Returns the value of a storage slot. Reads from DirtyStore first (dirty slots), then upstream.

**Parameters:**
1. `ADDRESS` -- contract address
2. `QUANTITY` -- storage slot (hex 32-byte value)
3. `BLOCK_TAG`

**Returns:** `DATA` -- 32-byte storage value

### eth_getCode

Returns bytecode at an address. Returns dirty code if set via `mirage_setCode`; otherwise reads from upstream. Results are cached in the bytecode cache keyed by code hash and never invalidated (bytecode is immutable post-deployment).

**Parameters:**
1. `ADDRESS`
2. `BLOCK_TAG`

**Returns:** `DATA` -- bytecode

### eth_getTransactionCount

Returns the nonce for an address. Returns dirty nonce if locally modified; otherwise upstream.

**Parameters:**
1. `ADDRESS`
2. `BLOCK_TAG`

**Returns:** `QUANTITY` -- nonce

### eth_call

Executes a read-only call against local state. Pins the block at call time for consistency. Does not mutate state.

**Parameters:**
1. `TransactionRequest` -- `{ from, to, data, value, gas, gasPrice }`
2. `BLOCK_TAG`

**Returns:** `DATA` -- return data from the call

**Error:** `-32015 EXECUTION_REVERTED` -- EVM reverted; includes revert reason in `data`

`eth_call` always executes against current local state (DirtyStore + lazy-latest upstream reads). Block tag is accepted for compatibility but does not change what state is used.

### eth_sendTransaction

Executes a transaction against local state. Mutates state. Triggers DiffClassifier to update the watch list. Increments the local block counter.

**Parameters:**
1. `TransactionRequest` -- `{ from, to, data, value, gas, gasPrice, nonce }`

**Returns:** `DATA` -- transaction hash (32-byte hex)

**Errors:**
- `-32015 EXECUTION_REVERTED` -- EVM reverted; state not committed
- `-32010 INVALID_FROM` -- `from` address not recognized

**Behavior:**
- Nonce validation disabled by default (any nonce accepted). Enable via `--strict-nonce`.
- Balance check disabled by default. Enable via `--strict-balance`.
- Gas price ignored for local execution; gas limit enforced.
- Transaction committed immediately (no mempool).
- Receipt available via `eth_getTransactionReceipt` after execution.

### eth_sendRawTransaction

Same as `eth_sendTransaction` but accepts a signed RLP-encoded transaction.

**Parameters:**
1. `DATA` -- signed raw transaction bytes

**Returns:** `DATA` -- transaction hash

### eth_getTransactionReceipt

Returns the receipt for a locally-executed transaction. Returns `null` if the hash is not found.

**Parameters:**
1. `DATA` -- transaction hash

**Returns:** `TransactionReceipt | null`

### eth_getTransactionByHash

Returns the transaction object for a locally-executed transaction.

**Parameters:**
1. `DATA` -- transaction hash

**Returns:** `Transaction | null`

### eth_getLogs

Returns logs from locally-executed transactions matching a filter. Only returns logs from transactions executed locally within this mirage session. Does not proxy to mainnet logs.

**Parameters:**
1. `LogFilter` -- `{ fromBlock, toBlock, address, topics }`

**Returns:** `Log[]`

### eth_getBlockByNumber

Returns a synthetic block object for the local block number. Block hashes are deterministically generated. Transactions array contains locally-executed transactions.

**Parameters:**
1. `BLOCK_TAG`
2. `BOOL` -- include full transaction objects (true) or hashes only (false)

**Returns:** `Block | null`

### eth_getBlockByHash

Same as `eth_getBlockByNumber` but looked up by block hash.

### eth_estimateGas

Estimates gas by executing against a temporary state snapshot (does not commit). Returns gas used plus a 20% buffer.

**Parameters:**
1. `TransactionRequest`
2. `BLOCK_TAG`

**Returns:** `QUANTITY` -- estimated gas

### eth_gasPrice

Without `--sim-gas`: returns `0x1` (1 wei). Local execution ignores gas price.

With `--sim-gas`: returns a sine-wave-modulated gas price approximating mainnet fluctuations.

### eth_feeHistory

Returns synthetic fee history. Constant values without `--sim-gas`; sine-wave history with it.

**Parameters:**
1. `QUANTITY` -- number of blocks
2. `BLOCK_TAG`
3. `Array<QUANTITY>` -- reward percentiles

**Returns:** `FeeHistory`

### eth_maxPriorityFeePerGas

Returns `0x0` without `--sim-gas`, or a simulated priority fee with it.

### net_version

Returns chain ID as a decimal string.

### web3_clientVersion

Returns `"mirage-rs/2.0.0"`.

---

## Hardhat/Anvil Compatibility Methods

These methods exist for compatibility with tooling targeting Hardhat or Anvil. Each has both a `hardhat_` and `anvil_` prefix that work identically.

### hardhat_impersonateAccount / anvil_impersonateAccount

Allow `eth_sendTransaction` with `from` set to any address, bypassing signature verification.

**Parameters:** `ADDRESS`

**Returns:** `true`

### hardhat_stopImpersonatingAccount / anvil_stopImpersonatingAccount

Remove an account from the impersonation list.

**Parameters:** `ADDRESS`

**Returns:** `true`

### hardhat_setBalance / anvil_setBalance

Set the ETH balance of an address directly.

**Parameters:**
1. `ADDRESS`
2. `QUANTITY` -- new balance in wei

**Returns:** `true`

### hardhat_setCode / anvil_setCode

Set the bytecode at an address.

**Parameters:**
1. `ADDRESS`
2. `DATA` -- new bytecode

**Returns:** `true`

### hardhat_setStorageAt / anvil_setStorageAt

Set a storage slot value.

**Parameters:**
1. `ADDRESS`
2. `QUANTITY` -- slot index
3. `DATA` -- 32-byte value

**Returns:** `true`

### hardhat_mine / anvil_mine

Mine N empty blocks. Increments the local block counter without executing transactions.

**Parameters:**
1. `QUANTITY` -- number of blocks (default: 1)
2. `QUANTITY` -- interval in seconds between blocks (default: 12)

**Returns:** `true`

### hardhat_reset / anvil_reset

Reset mirage state to a fresh fork. Clears DirtyStore, ReadCache, watch list, and snapshots. Re-forks from the specified block (or latest if omitted). This is destructive -- all local state is lost.

**Parameters:**
1. `{ forking: { jsonRpcUrl: string, blockNumber?: number } }` -- optional fork config

**Returns:** `true`

### hardhat_setNextBlockBaseFeePerGas / anvil_setNextBlockBaseFeePerGas

Set the base fee for the next mined block.

**Parameters:** `QUANTITY` -- base fee in wei

**Returns:** `true`

### anvil_setNonce

Set the nonce for an account directly.

**Parameters:**
1. `ADDRESS`
2. `QUANTITY` -- new nonce

**Returns:** `true`

### hardhat_setCoinbase / anvil_setCoinbase

Set the coinbase address for subsequent blocks.

**Parameters:** `ADDRESS` -- new coinbase address

**Returns:** `true`

### anvil_setPrevRandao

Set the PREVRANDAO value for the next block.

**Parameters:** `DATA` -- 32-byte value

**Returns:** `true`

---

## evm_* Methods

### evm_snapshot

Save a snapshot of current local state. Returns a snapshot ID for `evm_revert`.

Snapshots capture DirtyStore (dirty accounts, slots, watch list, impersonation list). ReadCache is not snapshotted -- it is a cache, not state.

**Parameters:** none

**Returns:** `QUANTITY` -- snapshot ID

### evm_revert

Restore state to a previous snapshot. The snapshot is consumed -- calling `evm_revert` with the same ID twice fails.

**Parameters:** `QUANTITY` -- snapshot ID

**Returns:** `BOOL`

**Error:** `-32001 SNAPSHOT_NOT_FOUND`

### evm_mine

Mine one or more empty blocks.

**Parameters:** `QUANTITY` -- number of blocks (default: 1)

**Returns:** `true`

### evm_increaseTime

Add seconds to the current block timestamp.

**Parameters:** `QUANTITY` -- seconds to add

**Returns:** `QUANTITY` -- new timestamp

### evm_setNextBlockTimestamp

Set the timestamp for the next mined block.

**Parameters:** `QUANTITY` -- Unix timestamp in seconds

**Returns:** `true`

**Error:** `-32602 INVALID_PARAMS` -- timestamp in the past

---

## mirage_* Core State Manipulation

### mirage_setBalance

Set the ETH balance of an address in DirtyStore.

**Parameters:**
1. `ADDRESS`
2. `QUANTITY` -- new balance in wei

**Returns:** `true`

### mirage_setCode

Override the bytecode at an address. Useful for deploying mock contracts at existing addresses.

**Parameters:**
1. `ADDRESS`
2. `DATA` -- new bytecode (hex)

**Returns:** `true`

### mirage_setStorageAt

Set a storage slot value in DirtyStore.

**Parameters:**
1. `ADDRESS`
2. `QUANTITY` -- slot index (32-byte hex)
3. `DATA` -- 32-byte value

**Returns:** `true`

### mirage_mintERC20

Increase an ERC-20 balance without knowing the token's storage layout. mirage traces the `balanceOf` slot automatically and overrides it.

**Parameters:**
1. `ADDRESS` -- token contract
2. `ADDRESS` -- recipient
3. `QUANTITY` -- amount to mint (token units)

**Returns:** `true`

**Error:** `-32020 SLOT_DETECTION_FAILED` -- could not determine the balance slot

Calls `balanceOf(recipient)` to find the return value, uses `debug_traceCall` to identify which storage slot was read, then writes `current_balance + amount` to that slot.

### mirage_cleanup

Prunes ephemeral artifacts from disk. Only removes files belonging to dead processes.

**Parameters:**
```json
{ "stale_pids": true, "checkpoints": false }
```

**Returns:**
```json
{
  "files_removed": 2,
  "bytes_freed": 104857600,
  "details": [
    { "path": "/tmp/mirage-8545.pid", "reason": "stale (process not running)" }
  ]
}
```

### mirage_computeDomainSeparator

Calls a contract and returns the EIP-712 domain separator it currently computes. Useful for verifying what the contract will validate against when checking typed structured data signatures in fork state.

**Parameters:**
1. `ADDRESS` -- contract address

**Returns:** `DATA` -- 32-byte domain separator hash

### mirage_prefetchSlots

Pre-fetch specific storage slots from upstream into the ReadCache. Prevents mid-execution fetch failures when the upstream RPC is slow or unreachable.

**Parameters:**
1. `ADDRESS` -- contract address
2. `Array<QUANTITY>` -- array of storage slot indices (32-byte hex)

**Returns:** `true`

### mirage_prefetchAccount

Pre-fetch all account info (balance, nonce, code) from upstream into the ReadCache.

**Parameters:**
1. `ADDRESS` -- account address

**Returns:** `true`

---

## mirage_* Watch List Management

### mirage_watchContract

Manually add a contract to the watch list. TargetedFollower will replay mainnet transactions touching this address.

**Parameters:** `ADDRESS`

**Returns:** `true`

**Error:** `-32030 WATCH_LIST_FULL`

### mirage_unwatchContract

Remove a contract from the watch list and add it to the unwatch list (prevents auto-classification from re-adding it).

**Parameters:** `ADDRESS`

**Returns:** `true`

### mirage_getWatchList

Return the current watch list with metadata.

**Returns:**
```json
[
  {
    "address": "0x...pool",
    "source": "AutoClassified",
    "added_at_block": 19500000,
    "replay_count": 143,
    "dirty_slot_count": 7
  }
]
```

### mirage_getDirtySlots

Return the dirty storage slots for a specific address.

**Parameters:** `ADDRESS`

**Returns:**
```json
[
  { "slot": "0x0", "value": "0x...sqrtPriceX96" },
  { "slot": "0x1", "value": "0x...tick" }
]
```

### mirage_status

Return a full status snapshot of the mirage instance.

**Returns:**
```json
{
  "version": "2.0.0",
  "mode": "live",
  "status": "ready",
  "block_number": 19500042,
  "upstream_block": 19500042,
  "chain_id": 1,
  "watch_list_size": 7,
  "dirty_slot_count": 143,
  "snapshot_count": 2,
  "memory_bytes": 134217728,
  "resource_pressure": 0.25,
  "cache_hit_rate": 0.82,
  "divergence_detected": false,
  "uptime_seconds": 3642,
  "disk_usage_bytes": 52428800,
  "pid_file": "/tmp/mirage-8545.pid",
  "output_dir": null
}
```

`mode`: `"live"`, `"historical"`, or `"proxy"` (degraded).

`divergence_detected`: `true` if TargetedFollower detected a state divergence for any watched contract since the last call (resets on read).

---

## mirage_* Resource Management

### mirage_getResourceUsage

Current resource utilization. Intended for periodic polling by golems (mortal autonomous agents compiled as single Rust binaries running on micro VMs).

**Returns:**
```json
{
  "memory_bytes": 134217728,
  "memory_limit_bytes": 536870912,
  "resource_pressure": 0.25,
  "cache_hit_rate": 0.82,
  "cache_entries": 4231,
  "cache_capacity": 10000,
  "watch_list_size": 7,
  "dirty_slot_count": 143,
  "upstream_rpc_calls": 891,
  "upstream_rpc_errors": 2,
  "mode": "live",
  "disk_usage_bytes": 52428800,
  "checkpoint_count": 3,
  "checkpoint_bytes": 157286400,
  "output_dir": "/tmp/replay-output"
}
```

`resource_pressure` ranges from 0.0 (idle) to 1.0 (at memory limit, degraded to proxy mode).

### mirage_setResourceLimits

Adjust resource limits at runtime. All fields optional; omitted fields retain current values.

**Parameters:**
```json
{
  "max_cache_entries": 20000,
  "max_watched_contracts": 32,
  "cache_ttl_seconds": 6,
  "max_memory_bytes": 1073741824
}
```

**Returns:** `true`

**Error:** `-32602 INVALID_PARAMS` -- value below minimum

---

## mirage_* Position Helpers

### mirage_getPosition

Convenience read: returns token balances and protocol-specific position state for a set of addresses in one call.

**Parameters:**
```json
{
  "owner": "0x...golem",
  "protocols": [
    { "address": "0x...pool", "type": "uniswap-v3-position" },
    { "address": "0x...aave", "type": "aave-v3-account" }
  ],
  "tokens": ["0x...weth", "0x...usdc"]
}
```

Supported types: `"uniswap-v3-position"`, `"aave-v3-account"`, `"raw-balances"`.

**Returns:**
```json
{
  "owner": "0x...golem",
  "token_balances": {
    "0x...weth": "1500000000000000000",
    "0x...usdc": "3200000000"
  },
  "positions": {
    "0x...pool": {
      "type": "uniswap-v3-position",
      "tick_lower": -75000,
      "tick_upper": -73000,
      "current_tick": -74321,
      "in_range": true,
      "liquidity": "12345678901234567890",
      "fees_token0": "1234567890123456",
      "fees_token1": "987654"
    },
    "0x...aave": {
      "type": "aave-v3-account",
      "health_factor": "1850000000000000000",
      "total_collateral_usd": "5000000000000000000000",
      "total_debt_usd": "2700000000000000000000",
      "ltv": "8000",
      "liquidation_threshold": "8250"
    }
  }
}
```

**Error:** `-32040 UNKNOWN_PROTOCOL_TYPE`

### mirage_subscribeEvents

Subscribe to EVM events emitted by locally-executed transactions (including TargetedFollower replays). Returns a stream endpoint.

**Parameters:**
```json
{
  "filter": {
    "addresses": ["0x...pool"],
    "topics": ["0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67"]
  },
  "transport": "sse"
}
```

`transport`: `"sse"` (server-sent events) or `"ws"` (WebSocket). If `filter` is omitted, delivers all events.

**Returns:**
```json
{
  "stream_url": "http://localhost:8545/events/stream_abc123",
  "subscription_id": "stream_abc123"
}
```

Each event on the stream:
```json
{
  "block_number": 19500001,
  "tx_hash": "0x...",
  "log_index": 0,
  "contract": "0x...pool",
  "topics": ["0xc42079..."],
  "data": "0x...",
  "source": "local_tx",
  "decoded": {
    "event": "Swap",
    "sender": "0x...",
    "amount0": "-1000000000000000000",
    "amount1": "2450000000",
    "sqrtPriceX96": "1997231467053478179446912",
    "tick": -74321
  }
}
```

`source`: `"local_tx"` (submitted by the golem) or `"follower_replay"` (replayed from mainnet).

Unsubscribe: `DELETE http://localhost:8545/events/{subscription_id}`.

---

## mirage_* Scenario Runner

### mirage_beginScenarioSet

Create a new scenario set with a baseline state.

**Parameters:**
```json
{ "baseline_state": "latest" }
```

`baseline_state`: `"latest"` or a snapshot ID (hex).

**Returns:** `{ "set_id": "abc123" }`

### mirage_defineScenario

Add a scenario to a set.

**Parameters:**
```json
{
  "set_id": "abc123",
  "name": "entry_5eth",
  "transactions": [
    { "from": "0x...", "to": "0x...", "data": "0x...", "value": "0x4563918244f40000", "gas": "0x7a120" }
  ],
  "track_addresses": ["0x...weth", "0x...usdc", "0x...pool"],
  "max_gas": 3000000,
  "timeout_seconds": 30
}
```

**Returns:** `{ "scenario_id": "abc123-0" }`

**Errors:** `-32050 SET_NOT_FOUND`, `-32051 SET_ALREADY_RUNNING`

### mirage_runScenarioSet

Execute all scenarios in a set. Execution is asynchronous.

**Parameters:**
```json
{ "set_id": "abc123", "mode": "sequential" }
```

`mode`: `"sequential"` or `"parallel"`.

**Returns:** `{ "job_id": "def456" }`

**Errors:** `-32050 SET_NOT_FOUND`, `-32052 SET_HAS_NO_SCENARIOS`, `-32053 PARALLEL_UNAVAILABLE`

### mirage_getScenarioResults

Poll for results from a running or completed scenario job.

**Parameters:** `{ "job_id": "def456" }`

**Returns:**
```json
{
  "job_id": "def456",
  "status": "complete",
  "set_id": "abc123",
  "scenarios": [
    {
      "scenario_id": "abc123-0",
      "name": "entry_1eth",
      "status": "success",
      "gas_used": 312450,
      "wall_time_ms": 145,
      "peak_memory_bytes": 52428800,
      "final_balances": { "0x...weth": "0", "0x...usdc": "2450000000" },
      "position_state": {},
      "logs": [{ "address": "0x...pool", "topics": ["0xc42079..."], "data": "0x..." }],
      "revert_reason": null
    }
  ],
  "total_wall_time_ms": 412,
  "peak_memory_bytes": 52428800
}
```

Job status: `"running"` | `"complete"` | `"failed"`. Scenario status: `"success"` | `"reverted"` | `"timeout"` | `"gas_exceeded"` | `"error"`.

### mirage_compareScenarios

Produce a ranked comparison table from completed results.

**Parameters:**
```json
{ "job_id": "def456", "metric": "pnl" }
```

`metric`: `"pnl"` | `"gas"` | `"state_diff"`

**Returns (pnl):**
```json
{
  "metric": "pnl",
  "baseline_balances": { "0x...weth": "10000000000000000000", "0x...usdc": "0" },
  "ranking": [
    { "rank": 1, "scenario": "entry_5eth", "pnl_usd": 12.45,
      "pnl_token_deltas": { "0x...weth": "-5000000000000000000", "0x...usdc": "12450000000" } }
  ]
}
```

**Errors:** `-32054 JOB_NOT_FOUND`, `-32055 JOB_NOT_COMPLETE`

---

## Error Code Reference

| Code | Name | Description |
|------|------|-------------|
| -32700 | PARSE_ERROR | Invalid JSON |
| -32600 | INVALID_REQUEST | Not a valid JSON-RPC request |
| -32601 | METHOD_NOT_FOUND | Method does not exist |
| -32602 | INVALID_PARAMS | Invalid parameter types or values |
| -32603 | INTERNAL_ERROR | Internal mirage error |
| -32001 | SNAPSHOT_NOT_FOUND | evm_revert called with unknown snapshot ID |
| -32003 | NONCE_TOO_LOW | Expected nonce N, got M (with `--strict-nonce`) |
| -32004 | NONCE_TOO_HIGH | Gap in nonce sequence (with `--strict-nonce`) |
| -32010 | INVALID_FROM | from address not in impersonation list and no private key |
| -32015 | EXECUTION_REVERTED | EVM execution reverted; data field contains revert reason |
| -32020 | SLOT_DETECTION_FAILED | mirage_mintERC20 could not detect balance slot |
| -32030 | WATCH_LIST_FULL | Watch list at max capacity |
| -32040 | UNKNOWN_PROTOCOL_TYPE | mirage_getPosition received unknown type |
| -32050 | SET_NOT_FOUND | Scenario set ID not found |
| -32051 | SET_ALREADY_RUNNING | Attempt to define scenario on a running set |
| -32052 | SET_HAS_NO_SCENARIOS | mirage_runScenarioSet called on empty set |
| -32053 | PARALLEL_UNAVAILABLE | Parallel mode not supported on this instance |
| -32054 | JOB_NOT_FOUND | Scenario job ID not found |
| -32055 | JOB_NOT_COMPLETE | Results requested before job finished |
| -32099 | UPSTREAM_ERROR | Upstream RPC fetch or rate limit failure |

---

## Unsupported Methods

The following return `-32601 METHOD_NOT_FOUND` or static values:

- `eth_sign`, `eth_signTransaction`, `eth_signTypedData_v4` -- no private key management
- `eth_accounts` -- returns empty array (compatibility)
- `eth_syncing` -- returns `false`
- `eth_mining` -- returns `false`
- `eth_hashrate` -- returns `"0x0"`
- `eth_submitWork`, `eth_submitHashrate` -- mining not supported
- `net_listening` -- returns `true`
- `net_peerCount` -- returns `"0x0"`

---

## Cross-References

- Architecture overview: [01-mirage-rs.md](./01-mirage-rs.md) -- Core architecture: HybridDB three-tier database, DirtyStore, targeted follower, CoW state layers, Block-STM parallel execution
- Scenario runner and historical mode: [01c-mirage-scenarios.md](./01c-mirage-scenarios.md) -- Classification rules, targeted follower pipeline, historical replay modes, scenario runner with CoW branching, Latin Hypercube parameter sweeps
- Bardo integration and golem workflows: [01d-mirage-integration.md](./01d-mirage-integration.md) -- F6 fork workflow, Fork Inspector overlay, golem sidecar lifecycle, CorticalState pressure gating, resource profiles

## References

- JSON-RPC 2.0 Specification. https://www.jsonrpc.org/specification -- *Defines the request/response envelope format, error codes, and batch semantics that mirage-rs implements.*
- Ethereum JSON-RPC Specification. https://ethereum.github.io/execution-apis/api-documentation/ -- *The canonical reference for `eth_*`, `net_*`, and `web3_*` method signatures and return types.*
- Hardhat Network Reference. https://hardhat.org/hardhat-network/docs/reference -- *Documents the `hardhat_*` method namespace that mirage-rs implements for compatibility with Hardhat-targeted tooling.*
- Foundry Anvil Reference. https://book.getfoundry.sh/reference/anvil/ -- *Documents the `anvil_*` method namespace that mirage-rs implements for compatibility with Foundry-targeted tooling.*
