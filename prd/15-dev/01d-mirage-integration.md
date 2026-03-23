# mirage-rs Integration: Bardo TUI and Golem Workflows

**Version:** 2.0.0
**Last Updated:** 2026-03-18
**Crate:** `mirage-rs/`

---

> **Reader orientation:** This document specifies how mirage-rs integrates with the Bardo TUI (terminal user interface) and the Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) sidecar lifecycle (section 15). The key concept is F6 fork: pressing F6 in the Sanctum (Bardo's DeFi protocol interaction surface) spawns a mirage fork at the current block, letting operators simulate trades against live market state. The document also covers the Fork Inspector overlay for managing multiple forks, golem sidecar spawn/teardown, resource pressure gating via CorticalState (the Golem's 32-signal atomic shared perception surface), and core golem workflows (position monitoring, hypothesis testing, risk scenarios). See `prd2/shared/glossary.md` for full term definitions.

## Bardo TUI Integration

mirage-rs is a command-line tool and a library, but most golems and operators interact with it through bardo-terminal's Sanctum layer. The Sanctum is bardo's DeFi protocol interaction surface. When a user presses F6 in a protocol view, the terminal spawns a mirage fork, replaces live chain data with simulated state, and exposes fork management through the Fork Inspector overlay.

### F6 -- Fork Here

F6 is the "fork this state" key. Its behavior depends on the current context.

**Not in Paper Mode:** F6 enters Paper/Mirage mode and spawns a mirage fork at the current block. The chain header shows the fork as the active context.

**Already in Paper/Mirage mode with one fork:** F6 creates a second parallel fork branch at the current block, adds it to the chain selector, and switches the protocol view to it.

**Already viewing a fork:** F6 opens the Fork Inspector overlay (see below).

**mirage unavailable** (binary not found, or RPC unreachable): F6 falls back to Estimation mode and shows an `[EST]` indicator in the chain header. No mirage process is started.

**Spawn fails due to insufficient memory:** F6 opens the resource overlay instead of attempting to fork. The overlay explains the failure and suggests actions: reduce the profile, stop other forks, or check `mirage_getResourceUsage`. If `resource_pressure` from `mirage_getResourceUsage` exceeds 0.7, the TUI refuses the new fork without attempting spawn.

### Chain Header in Mirage Mode

When one or more forks are active, the chain header shows all available contexts:

```
[ CHAIN ]  * Mainnet  o Fork #1 (blk 21400000)  o Fork #2 (blk 21400000)
```

The active context is filled. Inactive contexts are open circles. When the active context is a fork, the indicator is amber. When on mainnet, white.

### Spawn Sequence

1. Check `resource_pressure` from any running mirage instance. If > 0.7, refuse and show resource overlay.
2. Check OS free memory: `available >= profile.max_memory + 128 MB`. If not, refuse with an error message naming the shortfall.
3. Spawn `mirage-rs` with the RPC URL from `~/.bardo/config.toml` (or from the golem's connection if one is active).
4. Poll `mirage_status` every 500ms. If not ready within 10 seconds, show a timeout error.
5. On success, switch the protocol view to the fork context and update the chain header.

The fork inherits the current protocol view's watched addresses. If the user is viewing a Uniswap v3 pool, mirage auto-watches that pool contract via `mirage_watchContract` at spawn.

### Fork Cycling

Within the Sanctum, `Shift+Left` / `Shift+Right` cycles through available chain contexts:

- Mainnet (always present)
- Fork #1, Fork #2, ... (if spawned)
- Testnet (if configured in `~/.bardo/config.toml`)

This is distinct from chain switching (`Alt+C`), which moves between actual chains (Ethereum, Base, Arbitrum). Chain-switch navigates to a different network. Fork-cycle navigates to a different fork of the current network.

When the active context is a fork, the protocol view panel border gets an amber tint. If the fork is frozen (block replay paused via `F3+F`), the header shows `Fork #1 (blk 21400000, frozen)` and the border tint shifts to rose.

### Fork Inspector Overlay

When F6 is pressed while already viewing a fork, it opens the Fork Inspector as a floating overlay. Dismiss with Esc.

```
+-- Fork Inspector --------------------------------------------------+
|  Golem: EMBER-3f   |  2 active forks   Total: 847MB / 2048MB      |
|  ----------------------------------------------------------------- |
|  Fork #1 (live)     Mainnet  blk 21400012  312MB  cpu 2.1%        |
|    Watching: 7 contracts  Dirty: 143 slots  Hit rate: 82%          |
|    [Snapshot] [Freeze] [Stop]                                      |
|                                                                    |
|  Fork #2 (frozen)   Mainnet  blk 21400000  198MB  cpu 0.0%        |
|    Watching: 3 contracts  Dirty: 47 slots   Hit rate: 71%          |
|    [Resume] [Cleanup] [Stop]                                       |
|                                                                    |
|  [+ New Fork]  [Stop All]  [Cleanup Artifacts]                     |
|  Disk: 52MB checkpoints in /tmp/replay-abc  [Clean up]             |
+--------------------------------------------------------------------+
```

Navigation: arrow keys move between fork entries, Enter selects and switches to the focused fork, Tab cycles action buttons, `d` on a focused fork prompts for confirmation before stopping it.

### Action Buttons

| Button | Effect |
|--------|--------|
| Snapshot | Calls `evm_snapshot` on the fork. Shows the snapshot ID briefly. |
| Freeze | Pauses TargetedFollower, stops block processing. |
| Resume | Resumes TargetedFollower, catches up to chain head. |
| Stop | Sends `mirage_shutdown`. Waits up to 5s, then SIGTERM. Removes fork from list. |
| Cleanup | Runs `mirage cleanup --stale-pids` for the fork's artifacts. |
| + New Fork | Spawns a new fork at the current block (same as F6 from a non-fork view). |
| Stop All | Stops all forks in order: sends shutdown to each, waits, force-kills stragglers. |
| Cleanup Artifacts | Runs `mirage cleanup --all` to sweep all orphaned PID files, sockets, checkpoints. |

### Resource Display

Each fork entry shows:

- **Memory:** current RSS in MB vs. the profile cap. Green below 70%, amber 70-90%, rose above 90%.
- **CPU:** rolling 5-second average.
- **Upstream RPC calls:** total since spawn.
- **Cache hit rate:** from `mirage_getResourceUsage`.
- **Watch list size:** number of watched contracts.
- **Dirty slot count:** slots modified locally.

The header row shows aggregate memory across all forks vs. the system budget. This number comes from the same `mirage_getResourceUsage` endpoint that governs pressure gating.

The three pressure tiers (0.5 warning, 0.7 throttle, 0.9 emergency) are hard-coded in mirage-rs v2. The TUI reflects them as color transitions -- it does not override them.

### Resource Pressure Flow

```
mirage_getResourceUsage (polled every block)
    |
    v
resource_pressure value
    |-- < 0.5  -> green indicators, new forks allowed
    |-- >= 0.5 -> amber indicators (warning), new forks with warning
    |-- >= 0.7 -> rose indicators (throttle), new forks refused
    |-- >= 0.9 -> flashing rose (emergency), CorticalState signal emitted
    v
Fork Inspector header color matches the highest pressure across all forks
```

### No-Golem Mode

When bardo runs without a golem connected, Sanctum is fully operational. Protocol data flows through the terminal's own RPC connections configured in `~/.bardo/config.toml`. F6 spawns a mirage fork using the same RPC config.

Unavailable without a golem:

- Emotional overlays (F4 Portal)
- Golem Perspective (F2)
- Steer integration
- CorticalState resource gating (replaced by OS-level memory check only)

The Spectre sidebar shows an empty/ghost sprite in this state. Resource gating for fork spawn falls back to the OS-level free-memory check: if `available < profile.max_memory + 128 MB`, the fork is refused with the resource overlay.

No-golem mode is the primary interface for operators who want to use mirage-rs for manual protocol exploration without an autonomous agent.

### Keybinding Map

Three bindings from the engagement-prd spec have been replaced due to terminal conflicts.

**`Ctrl+C` replaced by `Alt+C`.** `Ctrl+C` sends SIGINT in every POSIX terminal emulator. Intercepting it either prevents normal process termination or silently swallows a quit signal. The chain-cycling binding moves to `Alt+C`.

**`Ctrl+H` avoided.** `Ctrl+H` maps to the backspace character (0x08) in most terminals. Emergency halt should not share a binding with backspace. Use `Ctrl+Alt+H` or a dedicated function key.

**`Ctrl+Up/Down` replaced by `Alt+Up/Down`.** tmux intercepts `Ctrl+Up/Down` by default for pane resizing. Any binding relying on it breaks in multiplexer sessions.

| Key | Context | Action |
|-----|---------|--------|
| F2 | Protocol view | Golem Perspective overlay |
| F3 | Protocol view | Toggle Paper Mode / Mirage mode |
| F3+F | Mirage mode | Freeze fork |
| F3+R | Mirage mode | Resume frozen fork |
| F4 | Anywhere | Portal mode / first-person view |
| F5 | Protocol view | Force refresh all data |
| F6 | Protocol view, not on a fork | Fork here (spawn mirage at current block) |
| F6 | Protocol view, on a fork | Open Fork Inspector overlay |
| Alt+C | Sanctum | Cycle chains (Ethereum / Base / Arbitrum / ...) |
| Shift+Left | Sanctum, Mirage mode | Previous fork or mainnet |
| Shift+Right | Sanctum, Mirage mode | Next fork |
| Alt+Up / Alt+Down | Sanctum | Cycle golem (multi-golem setups) |
| Esc | Fork Inspector | Dismiss overlay |

**Grouping rationale:** F2, F3, F4 shift perspective (how you see the data). F5, F6 control environment state (what state you are looking at). `Alt+C`, `Shift+Left/Right` handle chain and fork navigation (where you are in the chain topology).

### Sanctum Spec Gaps

The engagement-prd documents (`bardo-v4-04-sanctum-protocol-layer.md` and `bardo-v4-06-protocol-view-catalog.md`) contain the complete Sanctum spec. Neither has been pulled into prd2 yet. The following are specified in the engagement-prd but missing from prd2:

- Entry points: SOMA tab 6 (Sanctum), and the drill path from position cards
- Chain selector component and its binding (corrected to `Alt+C` here)
- Protocol Browser layout: 40/60 split with category groupings
- Per-protocol tab structure (Uniswap: Swap / Pools / Positions / Tokens; Aave: Markets / Supply / Borrow / Overview)
- Paper Mode (F3) with Mirage and Estimation sub-modes
- No-golem operation mode

---

## Golem Sidecar Lifecycle

mirage runs as a binary sidecar. Each golem spawns one mirage process, communicates over local JSON-RPC, and tears it down on exit.

### Spawn

```
mirage-rs \
  --rpc-url wss://mainnet.infura.io/v3/${API_KEY} \
  --http-url https://mainnet.infura.io/v3/${API_KEY} \
  --port 8545 \
  --chain-id 1 \
  --max-memory 512mb \
  --max-watched-contracts 64 \
  --watchdog-timeout 30s
```

Startup sequence:
1. Bind to `--port` on localhost.
2. Write PID file at `/tmp/mirage-${port}.pid`.
3. Call `eth_blockNumber` to verify upstream connectivity.
4. Write status file at `/tmp/mirage-${port}-status.json`.
5. Emit startup log: `mirage ready port=8545 chain=1 upstream=connected`.

The golem polls `mirage_status` every 500ms until `status == "ready"`. If not ready within 10 seconds, treat as spawn failure and retry once.

### Warm

After spawn, the golem pre-warms optionally:
- Call `mirage_watchContract` for known protocol addresses.
- Submit baseline transactions to establish local position state.

Pre-warming is optional. mirage auto-classifies contracts from the first real transaction.

### Simulate

Normal operation. The golem sends `eth_call`, `eth_sendTransaction`, and `mirage_*` calls as needed. TargetedFollower runs in the background, keeping watched contracts current.

### Extract Results

After a simulation sequence, the golem reads results via:
- `eth_call` for final state.
- `mirage_getPosition` for convenience position snapshots.
- `mirage_getResourceUsage` to check memory pressure.

### Teardown

On golem shutdown:
1. Send `mirage_shutdown` (graceful drain).
2. mirage flushes pending status writes, closes WebSocket subscription.
3. mirage exits with code 0.
4. Golem removes PID file.

If the golem process dies without sending `mirage_shutdown`, the watchdog timer fires after `--watchdog-timeout` (default 30s) and mirage exits cleanly.

### Resource Envelope

| Resource | Default | Config flag |
|----------|---------|-------------|
| Max memory | 512 MB | `--max-memory` |
| Max watched contracts | 64 | `--max-watched-contracts` |
| Watchdog timeout | 30s | `--watchdog-timeout` |
| ReadCache capacity | 10,000 entries | `--cache-capacity` |
| Cache TTL | 12s | `--cache-ttl` |

---

## Resource Model

### Resource Profiles

| Profile | Memory cap | Max watched contracts | Cache entries |
|---------|-----------|----------------------|---------------|
| `micro` | 256 MB | 32 | 5,000 |
| `standard` (default) | 512 MB | 64 | 10,000 |
| `power` | 2 GB | 256 | 50,000 |

`micro` targets constrained VMs and parallel scenario runs where memory is shared across many processes. `standard` is the shipped default for single-instance use. `power` is for dedicated simulation machines or heavy historical replay where wall-clock time matters more than memory footprint.

Set with `--profile micro|standard|power`. Individual flags override profile defaults:

- `--max-memory BYTES` overrides the memory cap.
- `--max-contracts N` overrides the contract watch limit.
- `--cache-size N` overrides the cache entry limit.

Override flags apply after profile defaults load, so `--profile micro --cache-size 8000` works without specifying the other values.

### OS-Level Spawn Gating

Before starting, mirage checks whether the host has enough free memory:

```
required  = profile.max_memory + 128 MB
available = OS query:
              /proc/meminfo        (Linux)
              vm_stat              (macOS)
              GetMemoryStatus      (Windows)

if available < required:
    exit(2) with message
```

If the check fails, mirage exits with code 2 and a structured error:

```
error: insufficient memory to start mirage
  required:  640 MB (512 MB profile + 128 MB headroom)
  available: 380 MB
  suggestion: use --profile micro (384 MB required) or free memory first
```

Golems also run this check before spawning a mirage process. A golem that receives exit code 2 can retry later, reduce the profile, or signal CorticalState. It is not limited to logging a crash.

### Memory Pressure Response

When mirage approaches `--max-memory`:

1. **Warning (50%).** Log warning. Emit `resource_pressure = 0.5`.
2. **Throttle (70%).** Evict ReadCache LRU entries. New contracts demoted to SlotOnly. Emit `resource_pressure = 0.7`.
3. **Emergency (90%).** Stop replaying mainnet transactions entirely. All reads pass through to upstream. Local dirty state preserved but no longer updated by TargetedFollower. Emit `resource_pressure = 0.9` and `mode = "proxy"`.

Proxy mode is a safety valve. The golem receives a signal it can use to decide whether to continue, reduce positions, or pause.

### CorticalState Pressure Gating

| `resource_pressure` | Behavior |
|--------------------|----------|
| < 0.5 | Allow new forks |
| 0.5 (warning) | Allow new forks, log a warning |
| 0.7 (throttle) | Refuse new forks; degrade to sequential scenario mode |
| 0.9 (emergency) | Refuse all new forks; emit a CorticalState signal |

Three tiers: 0.5 (warning), 0.7 (throttle), 0.9 (emergency). Hard-coded in v2.

### Fork Budget

Golems maintain a `fork_budget`: the maximum concurrent mirage instances. Default is 2 (one live, one for scenarios). Configurable per golem.

Pre-spawn decision logic:
1. Check `fork_budget`. At limit: refuse spawn.
2. OS memory check: `available >= profile.max_memory + 128 MB`.
3. Both pass: spawn and record PID in the fork registry.
4. Either fails: degrade to estimation mode or sequential scenarios.

### Fork Registry

```rust
struct ForkEntry {
    pid: u32,
    port: u16,
    mode: ForkMode,      // Live | Historical | Scenario
    profile: Profile,    // Micro | Standard | Power
    spawned_at: Instant,
    output_dir: Option<PathBuf>,
}
```

The registry is ephemeral, per session. Used to enforce `fork_budget`, track resource attribution, and execute ordered teardown.

### Ephemeral Artifact Lifecycle

| Artifact | Created when | Default location | Cleaned when | Max size |
|----------|-------------|-----------------|-------------|----------|
| PID file | mirage starts | `/tmp/mirage-${port}.pid` | Graceful exit; watchdog exit; `mirage cleanup` | bytes |
| Status JSON | mirage starts | `/tmp/mirage-${port}-status.json` | Same as PID file | ~2 KB |
| Unix socket file | mirage starts (Unix socket mode) | `/tmp/mirage-${port}.sock` | Same as PID file | bytes |
| Historical replay output (PnL CSV, `events.jsonl`) | `--output-dir` specified | User-chosen path | User's responsibility; `mirage cleanup --output-dir PATH` | unbounded |
| Checkpoint files | `--checkpoint-every K` set | `${output-dir}/checkpoints/` | `mirage cleanup --checkpoints` | K x ~50 MB per checkpoint |

Stale PID files occur when a mirage process exits without cleanup (SIGKILL, host crash). `mirage cleanup` detects them by checking whether the recorded PID is alive. If the process is gone, the PID file, its corresponding status JSON, and any associated socket file are removed.

On graceful shutdown:

1. Send `mirage_shutdown` to each instance in the fork registry.
2. Wait up to 5s per process.
3. Force SIGTERM after 5s if still running.
4. Run `mirage cleanup --stale-pids` to sweep orphaned artifacts.

On crash: the watchdog timer in each mirage instance triggers after `--watchdog-timeout`. PID and status files are removed by mirage's exit handler. Checkpoint files are left for crash recovery.

### `mirage cleanup` Subcommand

```
mirage cleanup [OPTIONS]
  --all                  Remove all artifacts: stale PIDs, orphaned sockets, old checkpoints
  --stale-pids           Remove PID and status files for dead processes only
  --checkpoints PATH     Remove checkpoint files in the given replay output directory
  --older-than DURATION  Only remove artifacts older than this (e.g., 7d, 24h)
  --dry-run              Print what would be removed without removing anything
  --verbose              List each file removed and the bytes freed
```

The subcommand prints a summary line: files removed and total bytes freed. With `--dry-run`, the summary describes what would be removed.

---

## Standalone CLI UX for Historical Mode

Progress output goes to stderr so that files written to `--output-dir` are not polluted by progress characters.

Default progress bar format:

```
[========--------] 52% block 19499995/19500010  3.2 blk/s  ETA 4.7s
```

`--quiet` suppresses the progress bar entirely. Use this in scripts where stderr is captured or where progress output would interfere with downstream processing.

`--json-progress` replaces the rendered bar with JSON lines emitted to stderr, one per update:

```json
{"block":19499995,"total":19500010,"pct":52.0,"blk_per_sec":3.2,"eta_sec":4.7}
```

`--json-progress` and `--quiet` are mutually exclusive. If both are passed, mirage exits with code 1 and an error message.

Ctrl+C triggers a clean interrupt. If `--checkpoint-every` is set, mirage writes a final checkpoint before exiting. The exit code is 130 (SIGINT convention).

---

## Core Golem Workflows

### 1. Live Position Monitoring

The golem has an open LP position and needs to know when it drifts out of range.

```
POST mirage_watchContract { "address": "0x...pool" }

POST mirage_subscribeEvents {
  "filter": {
    "addresses": ["0x...pool"],
    "topics": ["Swap(address,address,int256,int256,uint160,uint128,int24)"]
  }
}

// Heartbeat loop (every new block).
POST mirage_getPosition {
  "owner": "0x...golem",
  "protocols": [
    { "address": "0x...pool", "type": "uniswap-v3-position" }
  ],
  "tokens": ["0x...token0", "0x...token1"]
}
```

TargetedFollower replays every Swap touching the pool, keeping `sqrtPriceX96`, `tick`, and `liquidity` current.

### 2. Hypothesis Testing

"If I swap 1 ETH for USDC right now, what's my slippage?"

```
POST eth_call {
  "to": "0x...router",
  "data": "<exactInputSingle(...)>",
  "from": "0x...golem",
  "gas": "500000"
}
```

No snapshot needed. `eth_call` never mutates state.

### 3. Trade Simulation

"What if I execute approve -> deposit -> stake?"

```
POST evm_snapshot {}  // Returns: { "id": 1 }

POST eth_sendTransaction { "to": "0x...token", "data": "<approve(...)>" }
POST eth_sendTransaction { "to": "0x...vault", "data": "<deposit(...)>" }
POST eth_sendTransaction { "to": "0x...staker", "data": "<stake(...)>" }

POST mirage_getPosition { ... }

// Accept: keep state. Reject: POST evm_revert { "id": 1 }
```

### 4. Risk Scenario

"What happens to my Aave position if ETH drops 20%?"

```
POST evm_snapshot {}

POST hardhat_impersonateAccount { "address": "0x...oracle_keeper" }
POST mirage_setStorageAt {
  "address": "0x...ethusd_feed",
  "slot": "0x...latestRoundData_slot",
  "value": "0x...new_price_80pct"
}

POST eth_call {
  "to": "0x...aave_pool",
  "data": "<getUserAccountData(golem_address)>"
}
// Returns: healthFactor (< 1e18 = liquidatable)

// If liquidatable, simulate the liquidation.
POST eth_sendTransaction {
  "from": "0x...liquidator",
  "to": "0x...aave_pool",
  "data": "<liquidationCall(...)>"
}

// Observe balances, then revert.
POST evm_revert { "id": 2 }
```

### 5. Retrospective Analysis

"Why did my position get liquidated at block 19500000?" Uses historical mode.

```
mirage-rs \
  --mode historical \
  --from-block 19499990 \
  --to-block 19500010 \
  --replay-mode hybrid \
  --track-addresses 0x...golem,0x...pool \
  --output-dir ./replay-output
```

### 6. Parallel Comparison

"Which entry size gives better PnL?" Uses the scenario runner.

```
POST mirage_beginScenarioSet { "baseline_state": "latest" }
POST mirage_defineScenario { "set_id": "abc", "name": "1eth", "transactions": [...] }
POST mirage_defineScenario { "set_id": "abc", "name": "5eth", "transactions": [...] }
POST mirage_defineScenario { "set_id": "abc", "name": "10eth", "transactions": [...] }
POST mirage_runScenarioSet { "set_id": "abc", "mode": "sequential" }
// Poll mirage_getScenarioResults, then mirage_compareScenarios.
```

---

## Golem-Specific RPC Methods

### `mirage_getResourceUsage`

Returns current resource utilization. Golems should poll periodically and feed `resource_pressure` into CorticalState.

```json
{
  "memory_bytes": 134217728,
  "memory_limit_bytes": 536870912,
  "resource_pressure": 0.25,
  "cache_hit_rate": 0.82,
  "cache_entries": 4231,
  "watch_list_size": 7,
  "dirty_slot_count": 143,
  "mode": "live",
  "upstream_rpc_calls": 891
}
```

### `mirage_setResourceLimits`

Adjust limits at runtime without restarting. Useful when a golem enters a high-load simulation phase and needs to expand cache capacity temporarily.

### `mirage_subscribeEvents`

Subscribe to EVM events from local transactions (including TargetedFollower replays). Returns an SSE or WebSocket stream URL.

Each event on the stream:

```json
{
  "block_number": 19500001,
  "tx_hash": "0x...",
  "contract": "0x...pool",
  "topics": ["0xc42079..."],
  "data": "0x...",
  "source": "follower_replay",
  "decoded": {
    "event": "Swap",
    "amount0": "-1000000000000000000",
    "amount1": "2450000000",
    "sqrtPriceX96": "...",
    "tick": -74321
  }
}
```

`source` is `"local_tx"` (submitted by the golem) or `"follower_replay"` (replayed from mainnet).

### `mirage_getPosition`

Convenience method: reads token balances and protocol-specific position state in a single call.

Supported `type` values:
- `"uniswap-v3-position"` -- reads `slot0`, `positions`, `ticks`
- `"aave-v3-account"` -- reads `getUserAccountData`
- `"raw-balances"` -- token balances only

---

## CorticalState Integration

mirage feeds into the golem's CorticalState in three places:

### 1. Resource Pressure

```
mirage_getResourceUsage -> CorticalState.resource_pressure
```

Polled every block. When `resource_pressure > 0.8`, CorticalState elevates caution. When `mode == "proxy"`, the golem should consider pausing new positions.

### 2. PnL Residual

```
mirage_getPosition (post-simulation) -> prediction_engine.residual_correction
```

After each simulation cycle, the golem compares simulated PnL against its predictive model's estimate. The delta feeds into the prediction engine as a residual correction signal.

### 3. Event Stream to Gamma Ticks

```
mirage_subscribeEvents -> heartbeat_loop.gamma_tick_processor
```

Events from the mirage stream act as "gamma ticks" (the fastest tier of the Heartbeat, the Golem's 9-step decision cycle, running at ~250ms) -- high-frequency signals that trigger position re-evaluation outside the normal block-cadence loop. A large Swap crossing a tick boundary triggers an immediate range check rather than waiting for the next heartbeat.

---

## Error Handling

| Condition | mirage behavior | Golem behavior |
|-----------|-----------------|----------------|
| Upstream RPC timeout | Return cached value if within TTL; else error | Fall back to last known state |
| Watch list at capacity | New protocols demoted to SlotOnly | Accept; monitor dirty_slot_count |
| Memory limit reached | Degrade to proxy mode | Pause new positions; alert |
| Snapshot not found | Return `-32001` error | Do not revert; log and continue |
| WebSocket disconnect | Reconnect with exponential backoff | No action; mirage auto-recovers |
| mirage process crash | N/A | Respawn; warm from scratch or checkpoint |

---

## `mirage status` Subcommand

```
mirage status [--port PORT] [--all]
```

Reads all PID files matching `mirage-*.pid`, verifies processes are alive, prints a table:

```
PORT   MODE        MEMORY     UPTIME   BLOCK      WATCHING
8545   live        312/512MB  1h 23m   21400012   7 contracts
8546   historical  198/256MB  4m  2s   19499997   3 contracts
```

`--all` includes stale (dead) entries, marking them as such.

Fields:

- **PORT**: the port mirage is bound to.
- **MODE**: `live` (following chain head) or `historical` (replay run).
- **MEMORY**: current RSS against the profile cap.
- **UPTIME**: time since start.
- **BLOCK**: most recently processed block number.
- **WATCHING**: count of contracts in the watch list.

---

## Cross-References

- Architecture overview: [01-mirage-rs.md](./01-mirage-rs.md) -- Core architecture: HybridDB three-tier database, DirtyStore, targeted follower, CoW state layers, Block-STM parallel execution
- RPC method reference: [01b-mirage-rpc.md](./01b-mirage-rpc.md) -- Full JSON-RPC method catalog: eth_*, mirage_*, evm_*, hardhat/anvil compatibility, error codes
- Scenario runner and historical mode: [01c-mirage-scenarios.md](./01c-mirage-scenarios.md) -- Classification rules, targeted follower pipeline, historical replay modes, scenario runner with CoW branching
- Heartbeat integration: [../01-chain-intelligence/05-heartbeat-integration.md](../01-chain-intelligence/05-heartbeat-integration.md) -- How chain intelligence hooks into the Heartbeat's (9-step decision cycle) Gamma/Theta/Delta ticks for CorticalState updates and perception processing
- Agent runtime: [../03-agent-runtime/](../03-agent-runtime/) -- CorticalState signals, Daimon (sub-agent personality modules) lifecycle, and behavioral phase management
