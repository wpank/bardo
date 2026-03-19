# golem-runtime

`golem-runtime` manages the golem lifecycle and the extension registry. It is the FSM that takes a golem from startup through active ticking, sleep, and death.

## Features

- Lifecycle FSM: states are `Initializing`, `Active`, `Sleeping`, `Dying`, `Dead`
- Extension registry: register, order, and invoke extensions via `HookId` at defined lifecycle points
- Tick loop: call `golem-heartbeat` once per tick interval, with backpressure if a tick runs long
- Transition handling: route `golem-mortality` death signals to the Thanatopsis sequence
- Graceful shutdown: complete in-flight ticks before transitioning to `Dying`

## Architecture

`golem-runtime` is Layer 1 — the second crate above `golem-core`. Everything else in the golem stack is registered as an extension or called from within a tick. The runtime itself has no DeFi logic; it only drives the lifecycle.

```
Initializing → Active ↔ Sleeping → Dying → Dead
```

The transition from `Active` to `Sleeping` is triggered by the sleep scheduler in `golem-dreams`. The transition to `Dying` is triggered by any of the three mortality clocks in `golem-mortality`. The `Dead` state is terminal; the process exits cleanly after the legacy phase completes.

Extensions register hooks at points like `on_tick_start`, `on_tick_end`, `on_sleep_enter`, `on_death`. The registry builds a topological execution order from declared dependencies, so extensions always run in the correct sequence regardless of registration order.
