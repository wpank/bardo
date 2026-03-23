# golem-runtime

**Status: shell — no public API yet.**

Extension registry, hook dispatch, and lifecycle FSM. The runtime is what makes a Golem a Golem rather than a collection of independent crates. It boots extensions in topological dependency order, dispatches hooks on each lifecycle event, and owns the `GolemState` that every other subsystem reads.

## Planned Public API

**`GolemRuntime`** — top-level runtime handle. Constructed with a `GolemConfig` and a set of registered extensions, then started.

**Extension registry** — extensions implement `golem_core::extension::Extension` (with `name()`, `layer()`, and async hook methods). The registry takes the registered set, performs a topological sort on declared dependencies, and boots extensions in that order. An extension at layer 6 (e.g. `SonificationExtension`) cannot be started until all lower-layer extensions have initialized.

**`GolemState`** — shared state accessible to all extensions. Wraps `CorticalState` with additional runtime metadata: current lifecycle phase, boot timestamp, extension health map. Written by the runtime, read by extensions.

**Lifecycle FSM** — five states with valid transitions enforced at compile time via the type-state pattern:

```
Provisioning → Active → Dreaming → Active  (cycling dream phases)
             → Terminal → Dead
Active       → Terminal → Dead
```

`Provisioning` → `Active` fires `on_session(SessionReason::Start)` on all extensions.
`Active` → `Terminal` fires `on_end` on all extensions in reverse registration order.
`Terminal` → `Dead` is the final state; the runtime releases all resources.

Type states mean you cannot call `Active`-only methods on a `Provisioning` runtime at compile time.

**Hook dispatch** — per-tick hooks, per-block hooks (fired once per N ticks for slower subsystems), and lifecycle hooks (`on_session`, `on_end`). Hooks are dispatched concurrently where extensions declare no shared dependencies, sequentially where they do.

**Shutdown handler** — catches SIGTERM and SIGINT, transitions to `Terminal`, and gives extensions a configurable drain window before hard-killing. The default drain window is 5 seconds.

## Extension Scale

28 concrete extensions organized into 7 dependency layers. Layer 1 has no dependencies; layer 7 can depend on anything below. Topological sort at boot enforces this — an extension at layer 6 cannot start until all lower-layer extensions have initialized.

## Hook Detail

20 lifecycle hooks. Key action return types:

| Hook | Return Type | Variants |
|------|------------|---------|
| `on_input` | `InputAction` | `Pass`, `Transform`, `Suppress` |
| `on_tool_call` | `ToolAction` | `Allow`, `Block`, `Modify` |

Hooks dispatch concurrently where extensions declare no shared dependencies. Where they do share dependencies, dispatch is sequential.

## Event Fabric

`tokio::broadcast` pub/sub bus in `golem-core::event::EventFabric`. Extensions subscribe to cross-crate signals without taking a direct dependency on the emitting crate. Events are fire-and-forget from the emitter's perspective; slow subscribers get dropped frames, not backpressure.

## Somatic Bus

Lock-free `AtomicU32` reads in `golem-core::somatic_bus` for real-time PAD (pleasure-arousal-dominance) affect access without mutexes. O(1) snapshots. Extensions that need the current affective state read directly from the atomic; no lock contention on the hot path.

## Lifecycle FSM

Five states. The `Active` ↔ `Dreaming` cycle is explicit:

```
Provisioning → Active ↔ Dreaming
             ↓           ↓
           Terminal → Terminal
                ↓
              Dead
```

`Active` → `Dreaming` is triggered by `VitalityState` scheduling (consolidation phase). `Dreaming` → `Active` returns when the dream cycle completes. Either state can transition to `Terminal` on shutdown or fatal error. `Terminal` → `Dead` is final and releases all resources.

Type states mean you cannot call `Active`-only methods on a `Dreaming` or `Provisioning` runtime at compile time.

## System Position

`golem-runtime` is the entry point for a Golem binary. It depends on `golem-core` and on every extension crate it boots. The `golem-heartbeat`, `golem-mortality`, `golem-sonification`, and `golem-ta` extensions are all registered here. `golem-safety`'s `PolicyCage` is initialized during `Provisioning` and cannot be modified after the transition to `Active`.
