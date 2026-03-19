# golem-core

## What It Is

`golem-core` is the Layer 0 foundation crate for Bardo. It defines the shared runtime vocabulary that every later crate imports: identity, `golem.toml` configuration, the lock-free cortical surface, the typed event fabric, the extension hook skeleton, taint labels, hyperdimensional primitives, and the per-tick arena allocator.

## Features

- `GolemId` for ephemeral in-process runtime identity
- `GolemConfig` for the canonical `golem.toml` schema with `GOLEM_*` and `BARDO_*` overrides
- `MirageSection` for local `mirage-rs` sidecar connectivity settings
- `CorticalState`, `CorticalSnapshot`, `PadVector`, `BehavioralPhase`, and `PlutchikEmotion` for zero-latency shared perception
- `EventFabric`, `GolemEvent`, `EventPayload`, and `Subsystem` for non-blocking event broadcast plus bounded replay
- `Extension`, `ExtensionRegistry`, hook contexts, and hook action types for runtime lifecycle orchestration
- `TaintLabel` and `TaintedString` for explicit information-flow tracking
- `CognitiveTier` for routing inference spend across `T0`, `T1`, and `T2`
- `HdcVector` for 10,240-bit hypervector operations
- `TickArena` for tick-scoped bump allocation
- `GolemError` and `Result` as the crate-wide error surface

## Getting Started

Import the crate root and use the re-exports directly:

```rust
use std::path::Path;

use golem_core::{
    CognitiveTier, CorticalState, EventFabric, EventPayload, GolemConfig, GolemId, Subsystem,
    TaintLabel, TaintedString, TickArena,
};

fn example() -> golem_core::Result<()> {
    let _golem_id = GolemId::new();
    let _tier = CognitiveTier::try_from(1)?;

    let _config = GolemConfig::from_file(Path::new("golem.toml"))?;

    let cortical = CorticalState::new();
    cortical.write_affect(0.5, -0.3, 0.1, 7);
    let _snapshot = cortical.snapshot();

    let events = EventFabric::new(1_024);
    events.emit(
        Subsystem::Heartbeat,
        42,
        EventPayload::HeartbeatComplete {
            tick: 42,
            duration_ms: 12,
            actions_taken: 3,
        },
    );

    let arena = TickArena::new();
    let secret = TaintedString::new("0xabc".to_owned(), TaintLabel::WalletSecret);
    let _copied = arena.alloc(secret.value.clone());

    Ok(())
}
```

Downstream crates should prefer the root re-exports instead of reaching into implementation modules unless they need a specific module path for documentation or organization.

## Configuration

`GolemConfig` is the canonical runtime schema loaded from `golem.toml`. The top-level sections are:

- `golem`
- `heartbeat`
- `inference`
- `safety`
- `custody`
- `styx`
- `succession`
- `daimon`
- `dreams`
- `oracle`
- `mortality`
- `compute`
- `mirage`

Environment overrides use the `GOLEM_*` and `BARDO_*` prefixes.

```rust
use std::path::Path;

use golem_core::GolemConfig;

fn load_configs() -> golem_core::Result<()> {
    let _from_file = GolemConfig::from_file(Path::new("golem.toml"))?;
    let _from_str = GolemConfig::from_str(
        r#"
        [golem]
        name = "oracle-3"
        "#,
    )?;
    Ok(())
}
```

Useful overrides include `GOLEM_NAME`, `GOLEM_TICK_INTERVAL`, `GOLEM_MODE`, `GOLEM_CUSTODY_MODE`, `GOLEM_INFERENCE_PAYMENT`, `GOLEM_INFERENCE_DAILY_BUDGET`, `GOLEM_SUCCESSION_AUTO`, `GOLEM_SUCCESSION_BUDGET`, `GOLEM_DAIMON_ENABLED`, `GOLEM_DREAMS_ENABLED`, `GOLEM_ORACLE_ENABLED`, `GOLEM_COMPUTE_TIER`, `BARDO_STYX_ENABLED`, `BARDO_STYX_HOST`, `BARDO_CLADE_ENABLED`, `BARDO_STYX_DAILY_BUDGET`, `BARDO_STYX_MONTHLY_BUDGET`, `BARDO_IMMORTAL`, `BARDO_MORTALITY_ENABLED`, `BARDO_STOCHASTIC_SEED`, `BARDO_MIRAGE_URL`, `BARDO_MIRAGE_HOST`, `BARDO_MIRAGE_PORT`, `BARDO_MIRAGE_TIMEOUT_MS`, `BARDO_MIRAGE_RETRY_ATTEMPTS`, and `BARDO_MIRAGE_RETRY_BACKOFF_MS`.

## API

### Identity and Errors

`GolemId` is a transparent `uuid::Uuid` newtype with `new`, `from_uuid`, `as_uuid`, `Display`, and conversion impls. `GolemError` is the shared error enum for configuration loading, event fabric plumbing, cortical state helpers, and extension failures.

### Configuration

`GolemConfig` exposes typed sections for runtime operation:

- `GolemSection`
- `HeartbeatConfig`
- `InferenceConfig`
- `SafetyConfig`
- `CustodyConfig`
- `StyxConfig`
- `SuccessionConfig`
- `DaimonConfig`
- `DreamsConfig`
- `OracleConfig`
- `MortalityConfig`
- `ComputeConfig`
- `MirageSection`

`GolemConfig::from_file`, `GolemConfig::from_str`, and `GolemConfig::with_env_overrides` all preserve the same schema and override behavior.

### Cortical State

`CorticalState` is a cache-aligned atomic signal surface shared across runtime consumers. Writers update signal groups with release ordering, readers use acquire ordering, and `snapshot()` provides a best-effort point-in-time read for rendering and context assembly.

The main public helpers are:

- `pad()`
- `prediction_accuracy()`
- `phase()`
- `snapshot()`
- `write_affect(...)`
- `write_prediction(...)`
- `write_attention(...)`
- `write_environment(...)`
- `write_mortality(...)`
- `write_inference(...)`
- `write_creative(...)`
- `write_derived(...)`

### Event Fabric

`EventFabric` combines a live `tokio::sync::broadcast` channel with a bounded replay buffer. `emit` never blocks, `subscribe` attaches a live receiver, and `replay_from` returns cloned buffered events from a sequence number onward.

### Extension Hooks

`Extension` exposes the runtime hook surface for session lifecycle, input processing, agent start, turn execution, tool safety, post-turn learning, prompt steering, outbound messaging, debugging, error handling, and shutdown. `ExtensionRegistry` validates dependency ordering and provides dispatch helpers including `fire_after_turn`, `fire_tool_call`, `fire_session`, and `fire_end`.

### Taint, HDC, and Allocation

`TaintedString` keeps provenance explicit at the type level. `HdcVector` provides correctness-first `zeros`, `random`, `bind`, `bundle`, `permute`, and `similarity` helpers for later hyperdimensional work. `TickArena` wraps `bumpalo::Bump` for tick-scoped temporary allocation with O(1) reset.

## Architecture

At a high level, `golem-core` is the one crate every other Bardo crate can depend on without creating layering problems. It centralizes foundational types so later layers can exchange configuration, events, shared perception, and extension hooks without redefining those concepts. The result is a stable base API: runtime crates build behavior on top of it, application crates consume the same vocabulary, and the workspace avoids fragmented copies of the same core types.
