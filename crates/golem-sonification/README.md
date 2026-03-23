# golem-sonification

Real-time audio synthesis driven by a Golem's `CorticalState`. The crate runs a modular synthesis rack and maps cortical signals — vitality, arousal, behavioral phase — directly into audio parameters. A dying agent sounds different from a thriving one.

## Architecture: Four Threads

Thread ownership is strict. The cpal audio callback (Thread 4) must never allocate, so parameters cross thread boundaries through an `AtomicParameterBridge` — 16 slots of atomically-stored `f32` values, written by Thread 2 and read by Thread 3 without locking.

```
Thread 1: Golem runtime (tokio)
          └─ calls SonificationExtension hooks via Extension trait

Thread 2: "sono-param-updater" at 120Hz (8,333µs interval)
          └─ reads CorticalState::snapshot()
          └─ calls CvMapper::update_from_snapshot() → writes AtomicParameterBridge

Thread 3: "sono-rack-proc" at audio block rate (~667µs at 48kHz / BLOCK_SIZE=32)
          └─ reads AtomicParameterBridge
          └─ calls Rack::process_block() → (left: SignalBlock, right: SignalBlock)
          └─ writes AudioOutput ring buffer

Thread 4: cpal callback (OS-owned)
          └─ reads ring buffer → writes to hardware
          └─ no allocation, no locks
```

`SonificationExtension::boot()` is idempotent — calling it twice does nothing.

## `Rack` and the Module Graph

`Rack` holds an `IndexMap<String, Box<dyn Module>>` and a `Vec<PatchCable>`. Modules are processed in topological order via Kahn's algorithm, recomputed after every `add_module`, `connect`, or `disconnect` call.

```rust
use golem_sonification::{Rack, Module, PatchCable, PortId, PortDirection, SignalBlock};
use golem_sonification::modules::{mixer::Mixer, noise::NoiseSource, vca::Vca};

let mut rack = Rack::new();
rack.add_module(Box::new(NoiseSource::new("noise_1")));
rack.add_module(Box::new(Vca::new("vca_1")));
rack.add_module(Box::new(Mixer::new("mixer_1")));

rack.connect(
    PortId { module_id: "noise_1".into(), port_name: "out".into() },
    PortId { module_id: "vca_1".into(), port_name: "in".into() },
    1.0,  // attenuation
);
rack.connect(
    PortId { module_id: "vca_1".into(), port_name: "out".into() },
    PortId { module_id: "mixer_1".into(), port_name: "in_1".into() },
    1.0,
);

rack.master_level = 0.8;
let (left, right) = rack.process_block();
```

`process_block()` returns a stereo `(SignalBlock, SignalBlock)`. Master output is taken from the first module whose id starts with `"mixer"`, or falls back to the last module in processing order. `BLOCK_SIZE=32` samples at `SAMPLE_RATE=48000`.

The `Module` trait requires `id()`, `display_name()`, `ports() -> Vec<PortDeclaration>`, `process(inputs, outputs)`, plus `serialize_state`/`deserialize_state` for patch persistence.

## `CvMapper`

Maps `CorticalSnapshot` → `AtomicParameterBridge`:

- `composite_vitality → cv_index::MASTER_LEVEL` — weighted average: economic×0.4 + epistemic×0.3 + stochastic×0.3
- `arousal → cv_index::EVENT_DENSITY`

The bridge has 16 slots (`cv_index::COUNT`). Indices 0 and 1 are currently wired; the rest are available for additional mappings.

## `SonificationExtension`

Implements `golem_core::extension::Extension` at layer 6. Wire it into the extension registry before starting the runtime.

```rust
use golem_sonification::SonificationExtension;
use golem_core::cortical::CorticalState;
use std::sync::Arc;

let cortical = Arc::new(CorticalState::new());
let ext = SonificationExtension::new(Arc::clone(&cortical));
// register ext with the extension registry
```

On `SessionReason::Start`, it calls `boot()`, which wires the default patch (NoiseSource → Vca → Mixer) and spawns Threads 2 and 3. On `on_end`, it sets the shutdown flag and waits 50ms for threads to drain.

## NFT Audio

`nft` module handles NFT audio generation — generating a deterministic audio fingerprint from a golem's identity. Not wired into the runtime by default.

## Preset System

`preset` holds named patch configurations that can be loaded into a `Rack` via `deserialize`. The rack's `serialize()` / `deserialize()` methods round-trip cables and `master_level` as JSON. Module state restoration requires modules to be added to the rack first (no module factory yet).

## Dependencies

```toml
[dependencies]
golem-sonification = { path = "../../crates/golem-sonification" }
```

Depends on `golem-core` for `CorticalState` and the `Extension` trait. Uses `cpal` for audio I/O, `ringbuf` for the lock-free ring buffer between Thread 3 and Thread 4, and `indexmap` for deterministic module ordering.
