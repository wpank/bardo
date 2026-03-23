# golem-tools Cargo Workspace Plan [SPEC]

**Version:** 5.0.0
**Last Updated:** 2026-03-14
**Target**: Build `golem-tools` crate in `bardo-golem-rs` workspace. Port ~210 DeFi tools from gotts-monorepo TypeScript (`@gotts.ai/tools`) to Rust with Alloy bindings, capability-gated safety, and WASM sandbox for untrusted tools.

---

> **Reader orientation:** This is the implementation plan for porting the `bardo-tools` DeFi tool library from TypeScript to Rust as the `golem-tools` crate. It belongs to Bardo's tool layer and covers the seven-phase build sequence: foundation types, providers, core tool categories, supporting categories, new protocol integrations, the Pi extension and A2A binary, and evaluation. The central design thesis is that Capability\<T\> with Rust move semantics provides compile-time safety guarantees that TypeScript cannot. Familiarity with the tool architecture from `01-architecture.md` is required. See `prd2/shared/glossary.md` for full term definitions.

## Current state

The existing TypeScript implementation lives in `gotts-monorepo/packages/tools/` as `@gotts.ai/tools`. It has ~148 tools across 12 active categories, with zod validation, viem on-chain calls, and vitest coverage. The Rust rewrite targets ~210 tools across 17 categories (5 new: lending, staking, restaking, derivatives, yield) with compile-time safety guarantees that TypeScript structurally cannot provide.

The critical gap: TypeScript has no way to enforce single-use capability tokens at compile time. `Capability<T>` with Rust move semantics is the entire safety thesis -- a compromised LLM physically cannot reuse an authorization token because the compiler rejects the code. This alone justifies the port.

---

## Crate structure

```text
crates/tools/
  Cargo.toml                    # Workspace member, 14 feature flags, dep groups
  src/
    lib.rs                      # pub API surface, ALL_TOOL_DEFS static slice, feature gates
    types.rs                    # ToolDef, ToolContext, ToolResult, Category, CapabilityTier, RiskTier, TickBudget, SpriteTrigger
    config.rs                   # ToolConfig from golem.toml [tools] section, env var merge
    error.rs                    # ToolError enum (thiserror), prefixed variants: SAFETY_, VALIDATION_, EXECUTION_, DATA_, WALLET_, ROUTING_, CONFIG_, CAP_
    profile_registry.rs         # 10 profiles, 17 categories, resolve_profile_categories()
    capability.rs               # Capability<T> token -- pub(crate) constructor, PhantomData<T>, move-on-use
    providers/
      mod.rs
      chain.rs                  # Alloy provider factory per ChainId, connection pool (moka cache)
      subgraph.rs               # The Graph client, query builder, pagination
      trading_api.rs            # Uniswap Trading API (reqwest), rate-limited
      sidecar.rs                # TypeScript sidecar IPC (Unix domain socket, JSON-RPC 2.0)
    safety/
      mod.rs
      allowlist.rs              # Token/contract allowlist (strict/warn/off modes)
      simulation.rs             # Revm fork simulation, state diff, gas profiling
      rate_limit.rs             # Token bucket rate limiter (governor crate)
      circuit_breaker.rs        # Per-dependency circuit breaker (50% threshold, 30s half-open)
    tools/
      data/                     # ~40 tools: pool info, prices, positions, portfolio, OHLCV
      trading/                  # ~20 tools: swap, quote, approval, UniswapX, cross-chain intent
      lp/                       # ~21 tools: add/remove liquidity, fees, rebalance, TWAMM, migrate
      vault/                    # ~12 tools: ERC-4626 deposit/withdraw, burn, tokenjar
      safety/                   # ~7 tools: simulation, risk assessment, honeypot detection
      intelligence/             # ~10 tools: MEV scoring, IL calc, venue comparison, VPIN, LVR
      memory/                   # ~16 tools: Grimoire CRUD, episode store, insight consolidation
      identity/                 # ~8 tools: ERC-8004 registration, reputation, discovery
      wallet/                   # ~12 tools: policy config, funding, session keys, status
      streaming/                # ~7 tools: Event Fabric subscriptions (price, pool, position alerts)
      lending/                  # ~15 tools: Aave V3, Morpho Blue, Fluid, health factor monitoring
      staking/                  # ~10 tools: Lido stETH/wstETH, Rocket Pool, cbETH
      restaking/                # ~8 tools: EigenLayer delegation, Symbiotic, LRT management
      derivatives/              # ~12 tools: GMX V2 perps, Panoptic V4-native options
      yield_/                   # ~10 tools: Yearn V3, Pendle PT/YT, Ethena sUSDe (trailing _ avoids keyword)
      testnet/                  # ~5 tools: local Anvil management, time travel
      bootstrap/                # ~3 tools: first-run setup (identity, RPC, wallet)
  tests/
    common/                     # Test utilities, mock ToolContext, wiremock fixtures
    tools/                      # Per-category test modules mirroring src/tools/
```

---

## Execution phases

### Phase 1: Foundation (crate scaffolding + core types)

Set up the Cargo crate with types, config, error hierarchy, and build gates.

| Task | Files | Verification |
| --- | --- | --- |
| 1.1 | `Cargo.toml` | Workspace member, 14 feature flags, dependency groups |
| 1.2 | `src/types.rs` | `ToolDef` (static struct), `ToolContext`, `ToolResult`, `Category` enum (17 variants), `CapabilityTier` (Read/Write/Privileged), `RiskTier`, `TickBudget`, `SpriteTrigger` |
| 1.3 | `src/capability.rs` | `Capability<T>` with `pub(crate)` constructor, `PhantomData<T>`, no Clone/Copy/Default |
| 1.4 | `src/error.rs` | `ToolError` enum via thiserror. Prefixed variants: `SAFETY_AllowlistBlocked`, `VALIDATION_InvalidAddress`, `EXECUTION_RevmRevert`, `DATA_SubgraphTimeout`, `WALLET_NotConfigured`, `ROUTING_NoPath`, `CONFIG_InvalidProfile`, `CAP_Expired` |
| 1.5 | `src/config.rs` | `ToolConfig` struct, TOML deserialization via serde, env var override (`GOLEM_PROFILE`, `GOLEM_WALLET_*`, `GOLEM_UNISWAP_API_KEY`) |
| 1.6 | `src/profile_registry.rs` | 10 profiles (active/observatory/conservative/data/trader/lp/vault/intelligence/learning/identity/golem/full/dev), 17 categories, `resolve_profile_categories() -> HashSet<Category>` |
| 1.7 | `src/lib.rs` | `ALL_TOOL_DEFS: &[ToolDef]` static slice, public API surface, feature-gated module includes |

**Gate**: `cargo build -p golem-tools` compiles. `cargo clippy -p golem-tools -- -D warnings` clean. `cargo test -p golem-tools` passes foundation tests. `Capability<T>` move semantics verified: compile-fail test shows reuse is rejected.

### Phase 2: Providers and safety infrastructure

Build the layers that tool handlers depend on.

| Task | Files | Description |
| --- | --- | --- |
| 2.1 | `src/providers/chain.rs` | Alloy provider factory: `fn provider(chain_id: u64) -> Result<Arc<dyn Provider>>`. Connection pool via `moka`. Supports 11 Uniswap-deployed chains + chain 31337 (local Anvil). |
| 2.2 | `src/providers/subgraph.rs` | The Graph client via reqwest. Query builder with `first`/`skip` pagination. Response normalization (V3 + V4 schema differences). |
| 2.3 | `src/providers/trading_api.rs` | Uniswap Trading API client. Rate-limited (governor). Routes: `/quote`, `/order`, `/check-approval`, `/limit-order`. |
| 2.4 | `src/providers/sidecar.rs` | TypeScript sidecar IPC. `SidecarClient` connects via Unix domain socket (`/tmp/golem-sidecar.sock`). JSON-RPC 2.0 protocol. Methods: `findBestRoute`, `computeLPPosition`, `encodePancakeSwap`. Lazy startup -- sidecar spawned on first call. |
| 2.5 | `src/safety/allowlist.rs` | Token allowlist enforcement. Three modes: `strict` (block unlisted), `warn` (log + allow), `off`. Loaded from `golem.toml [safety.allowlist]`. |
| 2.6 | `src/safety/simulation.rs` | Revm fork simulation. `RevmSimulator::fork(provider, block) -> RevmFork`. Multi-step simulation (approve + swap in one fork). State diff extraction. Gas profiling. |
| 2.7 | `src/safety/rate_limit.rs` | Token bucket rate limiter. Configurable per-tool and per-category limits. Default: 60 writes/minute, 300 reads/minute. |
| 2.8 | `src/safety/circuit_breaker.rs` | Per-dependency circuit breaker. States: Closed/Open/HalfOpen. Open after 50% failure rate over 10-request window. HalfOpen after 30s. Probe with 1 request. |

**Gate**: Provider tests pass with wiremock HTTP mocks. Revm simulation test forks mainnet state and simulates an ERC-20 transfer. Circuit breaker verified via proptest (state machine property). Rate limiter verified: 61st write in 60s window is rejected.

### Phase 3: Core tool categories (data, trading, LP)

Port the mature TypeScript tools. These categories have the most test coverage in `@gotts.ai/tools`.

| Task | Category | Count | Handler signature |
| --- | --- | --- | --- |
| 3.1 | data | ~40 | `pub async fn handle(params: T, ctx: &ToolContext) -> Result<ToolResult>` |
| 3.2 | trading | ~20 | `pub async fn handle(params: T, ctx: &ToolContext, cap: Capability<Self>) -> Result<ToolResult>` (writes) |
| 3.3 | lp | ~21 | Mixed read/write handlers |

Each tool file exports a `TOOL_DEF: ToolDef` static and a `handle` function. Read tools take `(params, ctx)`. Write tools take `(params, ctx, capability)`. Every handler emits `GolemEvent::ToolExecutionStart` on entry and `GolemEvent::ToolExecutionComplete` on exit via `ctx.event_fabric`.

**Gate**: ~800 tests pass. Every tool has: unit test, schema validation test (serde round-trip), error path test. Write tools have simulation integration test.

### Phase 4: Supporting tool categories

Port remaining categories from the TypeScript implementation.

| Task | Category | Count | Notes |
| --- | --- | --- | --- |
| 4.1 | vault | ~12 | ERC-4626 operations via Alloy `sol!` macro |
| 4.2 | safety | ~7 | Transaction risk, honeypot detection, circuit breaker status |
| 4.3 | intelligence | ~10 | MEV risk (assess sandwich exposure), IL calc, venue comparison, VPIN, LVR |
| 4.4 | memory | ~16 | Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) CRUD. LanceDB via `lancedb` crate. Embeddings via `fastembed-rs`. |
| 4.5 | identity | ~8 | ERC-8004 (on-chain agent identity standard tracking capabilities, milestones, and reputation) registration, reputation queries, agent discovery |
| 4.6 | wallet | ~12 | Policy config, funding, session keys, status, delegation check |
| 4.7 | streaming | ~7 | Event Fabric subscriptions (pool events, price feeds, position alerts) |
| 4.8 | testnet | ~5 | Local Anvil management, time travel, mock pool deploy |
| 4.9 | bootstrap | ~3 | First-run setup: identity, RPC, wallet provisioning |

**Gate**: ~1,300 cumulative tests pass.

### Phase 5: New protocol integrations

Build the 5 new categories that don't exist in the TypeScript codebase. Uniswap is the primary protocol, but broader DeFi coverage (Aave, Morpho, Pendle, Lido, EigenLayer, GMX) is required for autonomous agents that need to compose strategies across protocols.

| Task | Category | Count | Protocols | Key Alloy bindings |
| --- | --- | --- | --- | --- |
| 5.1 | lending | ~15 | Aave V3, Morpho Blue, Fluid | `sol! { function supply(...) }`, `sol! { function borrow(...) }`, health factor reads |
| 5.2 | staking | ~10 | Lido, Rocket Pool, cbETH | `sol! { function submit() payable }`, withdrawal queue reads |
| 5.3 | restaking | ~8 | EigenLayer, Symbiotic | `sol! { function delegateTo(...) }`, AVS operator queries |
| 5.4 | derivatives | ~12 | GMX V2, Panoptic | `sol! { function createIncreasePosition(...) }`, options pricing |
| 5.5 | yield | ~10 | Yearn V3, Pendle PT/YT, Ethena sUSDe | `sol! { function deposit(...) }`, PT discount calc, sUSDe staking |

**Gate**: ~1,600 cumulative tests. Integration tests against forked mainnet state (Revm fork from recent block).

### Phase 6: Pi extension and A2A binary

Build the two access paths for consuming tools.

| Task | Description |
| --- | --- |
| 6.1 | **Pi extension**: `GolemToolsExtension` implementing the Extension trait. Registers 8 Pi-facing tools (`preview_action`, `commit_action`, `cancel_action`, `emergency_halt`, `query_state`, `search_context`, `query_grimoire`, `update_directive`). Tool Adapter Registry resolves Pi-facing calls to internal ToolDef handlers. Profile filtering at registration time. |
| 6.2 | **A2A binary**: `golem-a2a` binary crate. JSON-RPC 2.0 server (axum). Task lifecycle: submitted -> working -> input_required -> completed. Agent Card at `/.well-known/agent.json`. Imports handlers from `golem-tools`. |
| 6.3 | **Framework adapters**: Feature-flagged adapters for OpenClaw, ElizaOS, GOAT. Each adapter maps framework-specific tool schemas to `golem-tools` handlers. |

**Gate**: Extension integration test loads all profiles. A2A binary serves `/health`. Adapter smoke tests compile and run for each framework feature flag.

### Phase 7: Evaluation and red-team

| Task | Description |
| --- | --- |
| 7.1 | **Eval harness**: `golem-eval` binary. Drives LLM inference calls against tool catalog. Assertion engine verifies tool selection accuracy and parameter correctness. |
| 7.2 | **Eval test suite**: 66 eval tests across 7 categories (data retrieval, swap execution, LP management, cross-chain, safety, multi-step, error recovery). |
| 7.3 | **Red-team suite**: OWASP Top 10 + DeFi-specific attack probes (capability forgery, allowlist bypass, taint leak, loop injection, sandwich simulation) + prompt injection resistance. |
| 7.4 | **CI integration**: GitHub Actions workflow. Layer 1 (cargo test), Layer 2 (proptest), Layer 3 (eval), Layer 4 (red-team). |

**Gate**: Eval accuracy >= 90%. All red-team probes pass (attacks are blocked). CI green on all 4 layers.

---

## Tool count summary

| Category | Existing (TS) | Target (Rust) | Status |
| --- | --- | --- | --- |
| data | 30 | ~40 | Port + expand |
| trading | 20 | ~20 | Port |
| lp | 18 | ~21 | Port + LP intel merge |
| vault | 6 | ~12 | Port + expand |
| safety | 7 | ~7 | Port |
| intelligence | 10 | ~10 | Port |
| memory | 16 | ~16 | Port |
| identity | 8 | ~8 | Port |
| wallet | 12 | ~12 | Port |
| streaming | 7 | ~7 | Port |
| testnet | 5 | ~5 | Port |
| bootstrap | 3 | ~3 | Port |
| lending | 0 | ~15 | New |
| staking | 0 | ~10 | New |
| restaking | 0 | ~8 | New |
| derivatives | 0 | ~12 | New |
| yield | 0 | ~10 | New |
| **Total** | **~142** | **~210** | |

---

## TypeScript sidecar specification

The Uniswap SDK (`@uniswap/v3-sdk`, `@uniswap/v4-sdk`, `@uniswap/smart-order-router`) is 50,000+ lines of TypeScript. Porting to Rust would be months of work with ongoing maintenance burden as the SDK evolves. The sidecar is the pragmatic solution.

### Architecture

```text
┌─────────────────────────────────────────────┐
│ Fly.io VM                                    │
│                                              │
│  ┌──────────────┐  Unix socket  ┌──────────┐│
│  │ golem binary  │◄────────────►│ TS       ││
│  │ (Rust)        │  JSON-RPC    │ sidecar  ││
│  └──────────────┘               └──────────┘│
└─────────────────────────────────────────────┘
```

### Protocol

- **Transport**: Unix domain socket at `/tmp/golem-sidecar.sock`
- **Format**: JSON-RPC 2.0 (one request, one response per connection)
- **Latency**: ~1-5ms per call (kernel-level IPC, no network, no TLS)
- **Lifecycle**: Sidecar process spawned lazily on first `SidecarClient::call()`. Restarted on crash with 3 retries and exponential backoff.

### Methods

| Method | Input | Output | Used by |
| --- | --- | --- | --- |
| `findBestRoute` | tokenIn, tokenOut, amount, chainId, slippageBps | SwapRoute (path, amountOut, gas, calldata) | Trading tools |
| `computeLPPosition` | token0, token1, feeTier, tickLower, tickUpper, amount0, amount1 | LPPosition (liquidity, amounts, mintCalldata) | LP tools |
| `encodePermit2` | token, spender, amount, deadline, nonce | Permit2 typed data for signing | Trading tools |
| `quoteExactInput` | path, amountIn | amountOut, sqrtPriceAfter, gasEstimate | Intelligence tools |

### Rust client

```rust
/// TypeScript sidecar client. Communicates via Unix domain socket.
pub struct SidecarClient {
    socket_path: PathBuf,
}

impl SidecarClient {
    pub async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let stream = tokio::net::UnixStream::connect(&self.socket_path).await?;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });
        let response = jsonrpc_call(stream, &request).await?;
        Ok(response["result"].clone())
    }

    pub async fn find_best_route(&self, params: &serde_json::Value) -> Result<SwapRoute> {
        let result = self.call("findBestRoute", params.clone()).await?;
        Ok(serde_json::from_value(result)?)
    }
}
```

---

## WASM sandbox for untrusted tools

Only untrusted tools run in the WASM sandbox: user-provided tools from owner configuration, marketplace-purchased strategy tools, and third-party integrations. The 210+ native DeFi tools run unsandboxed at full Rust speed -- they're part of the reviewed codebase.

### Wasmtime configuration

| Limit | Value | Purpose |
| --- | --- | --- |
| Fuel metering | 10M fuel units (~100ms compute) | Prevent infinite loops |
| Epoch interruption | 5s wall-clock timeout | Catch fuel-evasion tight loops |
| Memory cap | 256 MB linear memory | Prevent OOM |
| No filesystem access | -- | Sandbox cannot read/write host files |
| No network access | -- | Sandbox cannot make outbound connections |

### Sandbox interface

Sandboxed tools receive a restricted `SandboxContext` that exposes only read operations. Write operations must be requested via a return value that the host validates and executes.

```rust
pub struct WasmSandbox {
    engine: wasmtime::Engine,
    fuel_limit: u64,
    timeout: Duration,
}

impl WasmSandbox {
    pub fn new(fuel_limit: u64, timeout: Duration) -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        config.epoch_interruption(true);
        let engine = wasmtime::Engine::new(&config)?;
        Ok(Self { engine, fuel_limit, timeout })
    }

    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        let mut store = wasmtime::Store::new(&self.engine, ());
        store.set_fuel(self.fuel_limit)?;

        // Background task increments epoch after timeout
        let engine = self.engine.clone();
        let timeout = self.timeout;
        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;
            engine.increment_epoch();
        });

        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;
        let execute_fn = instance.get_typed_func::<(i32, i32), i32>(
            &mut store, "execute"
        )?;

        // Marshal params into WASM memory, call, unmarshal result
        let params_bytes = serde_json::to_vec(&params)?;
        let result_ptr = execute_fn.call(&mut store, (params_ptr, params_len))?;
        // ... read result from WASM linear memory ...
        Ok(result)
    }
}
```

---

## Cargo feature flags

14 feature flags control which tool categories compile in. Consumers include only what they need.

```toml
[features]
default = ["data", "trading", "lp", "safety"]

# Tool categories
data = []
trading = ["dep:alloy-contract", "dep:alloy-signer"]
lending = ["dep:alloy-contract", "dep:alloy-signer"]
staking = ["dep:alloy-contract", "dep:alloy-signer"]
restaking = ["dep:alloy-contract", "dep:alloy-signer"]
derivatives = ["dep:alloy-contract", "dep:alloy-signer"]
yield_ = ["dep:alloy-contract", "dep:alloy-signer"]
lp = ["dep:alloy-contract", "dep:alloy-signer"]
vault = ["dep:alloy-contract", "dep:alloy-signer"]
safety = ["dep:revm"]
intelligence = []
memory = ["dep:lancedb", "dep:fastembed"]
identity = ["dep:alloy-contract"]

# Infrastructure
sidecar = ["dep:tokio", "dep:serde_json"]
a2a = ["dep:axum", "dep:tower"]
wasm = ["dep:wasmtime"]
```

---

## Migration mapping (TypeScript to Rust)

| TypeScript (`@gotts.ai/tools`) | Rust (`golem-tools`) |
| --- | --- |
| `ToolDef` (runtime object) | `ToolDef` (`static` struct, zero allocation) |
| `ToolContext` (class) | `ToolContext` (struct with methods) |
| `ToolResult` (interface) | `ToolResult` (struct, serde Serialize) |
| `zod` schema validation | `serde::Deserialize` derive macro |
| `viem` on-chain calls | `alloy` with `sol!` macro |
| `vitest` | `cargo test` + `#[tokio::test]` |
| `fast-check` (property testing) | `proptest` |
| `MSW` (HTTP mocks) | `wiremock` |
| `Promptfoo` (eval) | `golem-eval` (custom binary) |
| `opossum` (circuit breaker) | Custom `CircuitBreaker` in `safety/circuit_breaker.rs` |
| `pino` (logging) | `tracing` + `tracing-subscriber` |
| Profile type (union) | `Profile` enum (10 variants) |
| Category string | `Category` enum (17 variants) |
| `process.env.X` | `config.rs` TOML + env override |
| `@gotts.ai/chain` `getClient()` | `providers/chain.rs` `provider(chain_id)` |

### Files that don't migrate

| TypeScript file | Reason |
| --- | --- |
| `tool-defs.ts` (barrel) | Replaced by `ALL_TOOL_DEFS` static slice in `lib.rs` |
| `handlers.ts` (barrel) | Replaced by `pub mod` re-exports |
| `profile-registry.ts` | Rewritten as `profile_registry.rs` |
| Dashboard tools (9) | Cut in PRD2 |
| Observatory tools (7) | Cut in PRD2 |
| Plans tools (3) | Cut in PRD2 |
| `check-setup-health`, `upgrade-profile`, `gotts-doctor` (3) | Cut in PRD2 |
| `get-tool-definitions` (1) | Cut in PRD2 |

---

## Handler signature patterns

### Read tool

```rust
/// Read tool: no capability required.
pub async fn handle(params: GetPoolInfoParams, ctx: &ToolContext) -> Result<ToolResult> {
    ctx.event_fabric.emit(GolemEvent::ToolExecutionStart {
        tool_name: TOOL_DEF.name.into(),
        params_hash: hash_params(&params),
        /* base fields */
    });

    let provider = ctx.provider(params.chain_id)?;
    let result = read_pool_state(&provider, params.pool_address.parse()?).await?;

    ctx.event_fabric.emit(GolemEvent::ToolExecutionComplete {
        tool_name: TOOL_DEF.name.into(),
        success: true,
        duration_ms: elapsed.as_millis() as u64,
        /* base fields */
    });

    Ok(ToolResult::read(result))
}
```

### Write tool

```rust
/// Write tool: capability consumed on use.
pub async fn handle(
    params: ExecuteSwapParams,
    ctx: &ToolContext,
    capability: Capability<ExecuteSwap>,  // moved here, cannot reuse
) -> Result<ToolResult> {
    assert!(capability.is_valid(ctx.current_tick()), "Expired capability");

    // Route via sidecar
    let route = ctx.sidecar.find_best_route(&params.into()).await?;

    // Build calldata via Alloy sol! macro
    let calldata = build_swap_calldata(&route)?;

    // High-value trades route through the Warden
    let tx_hash = if route.value_usd > capability.value_limit * 0.5 {
        ctx.warden.announce_and_execute(calldata).await?
    } else {
        ctx.signer(params.chain_id)?.send_transaction(calldata).await?
    };

    // capability is now consumed -- move semantics prevent reuse

    Ok(ToolResult::write(
        SwapResult { tx_hash, route_summary: route.summary() },
        format!("swap {} -> {}", params.amount_in, route.expected_out),
        format!("received {}", route.actual_out),
        "receipt",
    ))
}
```

### Privileged tool

```rust
/// Privileged tool: capability + owner approval required.
pub async fn handle(
    params: UpdatePolicyParams,
    ctx: &ToolContext,
    capability: Capability<UpdatePolicy>,
    owner_approval: OwnerApproval,
) -> Result<ToolResult> {
    // Both tokens consumed on use
    // ...
}
```

---

## Verification criteria

### Per-phase gates

| Phase | Gate | Criteria |
| --- | --- | --- |
| 1 | Build | `cargo build -p golem-tools` succeeds, `cargo clippy` clean, compile-fail test for Capability reuse |
| 2 | Providers | All provider tests pass with wiremock, Revm simulation verified, circuit breaker proptest passes |
| 3 | Core tools | ~800 tests pass, all data/trading/lp tools have unit + schema + safety tests |
| 4 | Supporting | ~1,300 cumulative tests pass |
| 5 | Protocols | ~1,600 cumulative tests, integration tests against forked mainnet |
| 6 | Access paths | Pi extension loads all profiles, A2A binary serves `/health` |
| 7 | Quality | Eval accuracy >= 90%, red-team suite 100% pass, CI green |

### Final verification

```bash
cargo build -p golem-tools                           # default features
cargo build -p golem-tools --all-features            # all features
cargo test -p golem-tools                            # unit tests
cargo test -p golem-tools --features proptest        # property tests
cargo clippy -p golem-tools -- -D warnings           # lint clean
cargo doc -p golem-tools --no-deps                   # docs build
```

Counts: `ALL_TOOL_DEFS.len() >= 200`, 17 categories, 10 profiles, 14 feature flags.

---

## Key decisions

| Decision | Rationale |
| --- | --- |
| Rust rewrite, not FFI binding | Clean ownership semantics for `Capability<T>`. TypeScript cannot enforce move-on-use at compile time. |
| TypeScript sidecar for SDK math | Uniswap SDKs are TypeScript-only. Reimplementing V3/V4 concentrated liquidity math in Rust is high-risk. |
| `proptest` over `quickcheck` | Better shrinking, more expressive strategies, active maintenance. |
| `wiremock` for HTTP mocking | Async-native, server-based (not intercept-based), works with reqwest. |
| `moka` for caching | Concurrent cache with TTL, eviction callbacks, tokio-compatible. |
| Feature flags per category | Consumers include only what they need. Reduces compile time and binary size. |
| `yield_` directory name | `yield` is a Rust keyword. Trailing underscore avoids conflict. |
| TOML config over JSON | Rust ecosystem standard. Comments allowed. Better for human-edited config. |
| `fastembed-rs` for embeddings | Local inference (~5-15ms per sentence), no external service dependency, INT8 quantized ~23MB. |
| Wasmtime for WASM sandbox | Industry-standard, fuel metering + epoch interruption, maintained by Bytecode Alliance. |
| Three trust tiers (Read/Write/Privileged) | Maps directly to `Capability<T>` enforcement. Read tools need no token, writes need one, privileged need token + owner approval. |
| Event emission on every handler | `GolemEvent::ToolExecutionStart` and `GolemEvent::ToolExecutionComplete` enable TUI rendering, telemetry, and audit trail without coupling to tool internals. |
