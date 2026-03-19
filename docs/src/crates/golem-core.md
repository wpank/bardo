# golem-core

`golem-core` is the Layer 0 foundation crate for the Bardo workspace. It has zero workspace dependencies and defines the shared type vocabulary that every other crate imports.

## Features

- Golem identity (`GolemId`) — ephemeral UUID-backed runtime identifier
- Full configuration schema (`GolemConfig`) with `golem.toml` support and environment variable overrides
- Cognitive tier enum (`CognitiveTier`: `T0`, `T1`, `T2`) for inference routing and cost gating
- Cortical state types: `CorticalState`, `CorticalSnapshot`, `PadVector`, `BehavioralPhase`, `PlutchikEmotion`
- Event fabric: `EventFabric`, `GolemEvent`, `Subsystem`, typed `EventPayload` variants, and a 10,000-event replay ring for reconnecting consumers
- Extension system: `Extension`, `ExtensionRegistry`, `HookId`, hook contexts, and action types with topological ordering
- Taint tracking: `TaintLabel` and `TaintedString` for marking data that crossed a trust boundary
- HDC primitives: `HdcVector` stub for 10,240-bit hypervectors
- Per-tick bump allocator: `TickArena` for scratch storage that is cheap to reset between ticks
- Error types: `GolemError` and the crate `Result` alias

## Configuration

`GolemConfig` is the root configuration struct. Load it from a `golem.toml` file or from environment variables:

```rust
use golem_core::config::GolemConfig;

let config = GolemConfig::from_env()?;
```

The full schema covers upstream RPC settings, inference tier budgets, mortality thresholds, grimoire limits, and sidecar addresses. See the `golem.toml` reference for the complete field list.

## Event Fabric

`EventFabric` is a broadcast channel with a built-in replay ring. Components publish `GolemEvent` values; subscribers receive them in real time or replay from a sequence number on reconnect.

```rust
let fabric = EventFabric::new(1024);
fabric.publish(GolemEvent { ... });

// Reconnecting subscriber: receive only events after seq 42
let events = fabric.replay_from(42);
```

The replay ring holds the last 10,000 events. Subscribers that reconnect receive only events with `seq > after_seq`, not the full ring.

## Extension Registry

`ExtensionRegistry` manages the lifecycle of pluggable extensions. Extensions declare dependencies on other extensions; the registry builds a topological execution order that respects both dependency constraints and registration order.

## Cortical State

`CorticalState` tracks a golem's affective and cognitive state using atomic `f32` bit patterns for lock-free reads. `PadVector` exposes the same values as `f64` at the API boundary. `CorticalSnapshot` captures a point-in-time copy including the full `[f32; 16]` category accuracy slice — note that snapshots are per-signal atomic but not transactionally consistent across a full tick.
