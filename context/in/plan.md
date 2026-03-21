# Plan 04b: Testing Infrastructure Bootstrap

## Context

Plans 01–04 have established the Cargo workspace, golem-core types, the mirage-rs EVM fork,
and the bardo-terminal scaffold. A comprehensive testing architecture spec exists at
`gringotts/bardo/tmp/testing2.md`. This plan wires that spec into the codebase and
bardo-ctl before any chain or TA crates are implemented — so every subsequent plan inherits
working test primitives from day one.

Four problems this plan solves:

1. `GolemConfig` has no test-friendly constructors. Every integration test that needs a
   headless golem must manually construct the full config. Two convenience constructors fix
   this.

2. `bardo-terminal`'s `App::run()` is hardcoded to `CrosstermBackend`. Plan 12 (headless
   terminal) cannot extend it cleanly without a `Backend` generic.

3. `mirage-rs` has no Phase 0 tests from `testing2.md §12` and no `MIRAGE_TEST_PORT` env
   convention. Both are buildable today with no upstream deps.

4. `bardo-ctl` has no `FullLoopTest` pipeline phase, no `FullLoopValidator` agent role,
   and no `full_loop_gate()`. Plans touching `golem-ta` or `golem-chain` have nowhere to
   route their integration tests.

## Previous Plan

Plan 04 created `bardo-terminal`: 60fps ratatui render loop, 29-screen ScreenRegistry,
ROSEDUST palette, responsive layout engine, panic hook, crossterm init/teardown.

## Prerequisites

- **Plan 01** — workspace scaffold
- **Plan 02** — `golem-core` implemented: `GolemConfig`, `EventFabric`, `CorticalState`,
  `DeploymentMode`, `GolemEvent`, `EventPayload`
- **Plan 03** — `mirage-rs` implemented: `spawn_mirage_test_instance`, `MirageClient`,
  existing integration test suite
- **Plan 04** — `bardo-terminal` implemented: `App`, `AppState`, `ScreenRegistry`

## Imports

```rust
// crates/golem-runtime/src/lib.rs — needs types from golem-core
use golem_core::{
    config::GolemConfig,
    event::{GolemEvent, EventPayload},
    cortical::CorticalSnapshot,
};

// apps/bardo-terminal/src/app.rs — uses ratatui Backend trait
use ratatui::backend::Backend;
```

## Exports

| Type / fn | Module | Purpose |
|-----------|--------|---------|
| `GolemConfig::test_with_rpc(url: String) -> Self` | `golem_core::config` | Pre-built config for integration tests — chain module points at given URL |
| `GolemConfig::test_headless() -> Self` | `golem_core::config` | Pre-built config for pure FSM/unit tests — no RPC, no wallet |
| `DeploymentMode::Headless` | `golem_core::config` | New variant; headless runtime skips infra |
| `EventSource` | `bardo_terminal::app` | `Crossterm \| Channel(mpsc::Receiver<Event>)` — injected event source for tests |
| `GolemRuntime` | `golem_runtime` | Struct with headless API (stubs — full impl: Plan 16) |
| `GolemRuntime::headless(config) -> Self` | `golem_runtime` | Creates headless instance (todo! until Plan 16) |
| `GolemRuntime::tick(&mut self) -> Vec<GolemEvent>` | `golem_runtime` | Advance one tick (todo! until Plan 16) |
| `GolemRuntime::snapshot(&self) -> GolemSnapshot` | `golem_runtime` | Full state snapshot (todo! until Plan 16) |
| `GolemSnapshot` | `golem_runtime` | Serializable runtime state snapshot |
| `AgentRole::FullLoopValidator` | `bardo_ctl::agent::roles` | Orchestrates full-loop integration test |
| `PipelinePhase::FullLoopTest` | `bardo_ctl::orchestrator::pipeline` | Gate phase for golem-ta/golem-chain plans |

## Cargo Dependencies

No new workspace-level crates. `golem-runtime` gains a dep on `golem-core`. `nextest.toml`
gains an integration profile.

```toml
# crates/golem-runtime/Cargo.toml — add:
[dependencies]
golem-core = { path = "../golem-core" }
serde = { workspace = true }
```

## Source Files Modified / Created

```
crates/golem-core/src/config.rs               — test constructors + DeploymentMode::Headless
apps/bardo-terminal/src/app.rs                — Backend generic + EventSource
apps/mirage-rs/tests/integration.rs           — test_pool_slot0 + MIRAGE_TEST_PORT
apps/mirage-rs/tests/scenarios/new_pool.toml  — new pool scenario
crates/golem-runtime/src/lib.rs               — GolemRuntime headless API stubs + GolemSnapshot
tmp/bardo-ctl/src/orchestrator/pipeline.rs    — FullLoopTest phase
tmp/bardo-ctl/src/orchestrator/gates.rs       — full_loop_gate(), wait_for_mirage_ready()
tmp/bardo-ctl/src/orchestrator/plan.rs        — extra_gates_for_plan()
tmp/bardo-ctl/src/agent/roles.rs              — FullLoopValidator
nextest.toml                                  — integration profile
plans/context/testing-reference.md            — canonical pointer
plans/context/prompts/retrofit-testing-docs.sh — metaprompt script
```

## Implementation Details

This plan has 11 implementation units across code, test infrastructure, and bardo-ctl.
See the full plan document for detailed code snippets, test patterns, and Cargo changes.

**Abbreviated overview of units:**

- **Unit 1:** golem-core `test_with_rpc()`, `test_headless()`, `DeploymentMode::Headless`
- **Unit 2:** bardo-terminal Backend generic + `EventSource` enum
- **Unit 3:** mirage-rs Phase 0 integration tests + `MIRAGE_TEST_PORT` support
- **Unit 4:** mirage-rs `new_pool.toml` scenario
- **Unit 5:** golem-runtime `GolemRuntime` headless API stubs
- **Unit 6:** bardo-ctl `FullLoopTest` pipeline phase + trigger logic
- **Unit 7:** bardo-ctl gate functions: `full_loop_gate()`, `wait_for_mirage_ready()`
- **Unit 8:** bardo-ctl `FullLoopValidator` agent role
- **Unit 9:** nextest integration profile
- **Unit 10:** `plans/context/testing-reference.md` canonical pointer
- **Unit 11:** `retrofit-testing-docs.sh` metaprompt script

## Testing Checkpoint

```bash
# All of these must pass after implementation:
cargo check --workspace
cargo test -p golem-core -- test_with_rpc_constructor test_headless_constructor --exact
cargo check -p golem-runtime
cargo test -p bardo-terminal -- test_app_backend_generic_compiles --exact
cargo test -p mirage-rs --test integration -- test_pool_slot0_matches_expected_price --nocapture
cargo nextest run --profile integration --list
```

## Completion Report

*(Codex fills this in after implementation.)*

## Verification

### Invariants

<!-- INV-001: test_with_rpc_constructor -->
- **type**: state_machine
- **module**: golem_core::config
- **property**: GolemConfig::test_with_rpc sets DeploymentMode::Headless and mirage.url
- **formula**: config.golem.mode == Headless && config.mirage.url == Some(url)
- **constraint**: both fields set correctly by constructor
- **test_fn**: `test_with_rpc_constructor`
- **strategy**: unit
- **inputs**: `{"url": "http://127.0.0.1:18545"}`
- **oracle**: mode == Headless, mirage.url == Some("http://127.0.0.1:18545")
- **severity**: spec
- **source**: plan 04b Unit 1

<!-- INV-002: test_headless_constructor -->
- **type**: state_machine
- **module**: golem_core::config
- **property**: GolemConfig::test_headless sets DeploymentMode::Headless and no mirage URL
- **formula**: config.golem.mode == Headless && config.mirage.url == None
- **constraint**: mode is Headless, url is None
- **test_fn**: `test_headless_constructor`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: mode == Headless, mirage.url == None
- **severity**: spec
- **source**: plan 04b Unit 1

<!-- INV-003: test_app_backend_generic_compiles -->
- **type**: state_machine
- **module**: bardo_terminal::app
- **property**: App::run is generic over B: Backend — compiles with TestBackend
- **formula**: App::run::<TestBackend> is callable (compile-time check)
- **constraint**: no type errors when B = ratatui::backend::TestBackend
- **test_fn**: `test_app_backend_generic_compiles`
- **strategy**: unit
- **inputs**: `{"backend": "TestBackend"}`
- **oracle**: compiles without error
- **severity**: spec
- **source**: plan 04b Unit 2

<!-- INV-004: test_pool_slot0_matches_expected_price -->
- **type**: numeric_range
- **module**: mirage_rs::integration
- **property**: sqrtPriceX96 round-trip error < 1e-9; synthetic balance set correctly
- **formula**: |decoded_price - price| / price < 1e-9
- **constraint**: relative error < 1e-9
- **test_fn**: `test_pool_slot0_matches_expected_price`
- **strategy**: integration
- **inputs**: `{"price": 1800.0}`
- **oracle**: rel_error < 1e-9
- **severity**: spec
- **source**: plan 04b Unit 3

<!-- INV-005: test_headless_constructor_compiles (golem-runtime) -->
- **type**: state_machine
- **module**: golem_runtime
- **property**: GolemRuntime::headless(config) compiles; config.golem.mode is accessible
- **formula**: GolemRuntime::headless(GolemConfig::test_headless()) does not panic at construction
- **constraint**: construction succeeds; runtime.config().golem.mode == Headless
- **test_fn**: `test_headless_constructor_compiles`
- **strategy**: unit
- **inputs**: `{}`
- **oracle**: construction succeeds
- **severity**: code
- **source**: plan 04b Unit 5

<!-- INV-006: full_loop_gate_returns_diagnostic_without_golem_eval -->
- **type**: state_machine
- **module**: bardo_ctl::orchestrator::gates
- **property**: full_loop_gate returns passed=false with diagnostic when golem-eval absent
- **formula**: GateResult.passed == false && output contains "golem-eval crate not yet implemented"
- **constraint**: does not panic; returns actionable diagnostic
- **test_fn**: `test_full_loop_gate_diagnostic`
- **strategy**: unit
- **inputs**: `{"golem_eval_exists": false}`
- **oracle**: passed=false, output contains expected message
- **severity**: code
- **source**: plan 04b Unit 7
