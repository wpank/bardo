# golem-core

Layer 0 of the Bardo workspace. Every other crate depends on this one; it has zero internal workspace dependencies.

Defines identity, configuration, event fabric, cortical state, taint labels, the extension hook system, HDC primitives, and the tick arena allocator. The design goal is that these types are cheap to construct, cheap to pass around, and have no async or I/O anywhere in the crate.

## Modules

### `alloc` — TickArena

Bump allocator for per-tick memory. Allocate fast during a tick, call `reset()` at the tick boundary to reclaim everything in O(1) without individual drops.

```rust
let mut arena = TickArena::new();
let val: &u32 = arena.alloc(42u32);
let slice: &[u8] = arena.alloc_slice_copy(&[1, 2, 3]);
arena.reset(); // all prior references are now invalid
```

`TickArena` is not thread-safe. Create one per tick context. Do not hold references across `reset()` — the type system will not catch this.

### `cortical` — CorticalState, PadVector, BehavioralPhase, PlutchikEmotion

`CorticalState` is a lock-free, cache-line-aligned signal surface shared between subsystems. It fits in 256 bytes (compile-time assertion) and is always heap-allocated inside an `Arc`:

```rust
let state = CorticalState::new();
state.write_affect(0.8, 0.3, 0.5, PlutchikEmotion::Joy as u8);
let snap: CorticalSnapshot = state.snapshot();
```

Grouped write methods keep atomic operations batched: `write_affect`, `write_prediction`, `write_attention`, `write_environment`, `write_mortality`, `write_inference`, `write_creative`, `write_derived`.

`PadVector` is the three-dimensional affective state (Pleasure, Arousal, Dominance), each `f64` in `[-1.0, 1.0]`. `PadVector::ZERO` is the origin. `clamp(min, max)` is `const`.

`BehavioralPhase` maps vitality to one of five lifecycle stages:

| Phase        | Vitality     |
|--------------|--------------|
| Thriving     | ≥ 0.7        |
| Stable       | 0.5 – 0.7    |
| Conservation | 0.3 – 0.5    |
| Declining    | 0.1 – 0.3    |
| Terminal     | < 0.1        |

`PlutchikEmotion` classifies a `PadVector` into one of eight labels via PAD octant:

```rust
PlutchikEmotion::from_pad(&PadVector { pleasure: 1.0, arousal: 1.0, dominance: 1.0 })
// => PlutchikEmotion::Joy

PlutchikEmotion::from_pad(&PadVector::ZERO)
// => PlutchikEmotion::Anticipation (zero vector maps here by convention)
```

Full octant table: `(+,+,+)` Joy, `(+,-,+)` Trust, `(-,+,-)` Fear, `(-,+,+)` Anger, `(-,-,-)` Sadness, `(+,+,-)` Surprise, `(-,-,+)` Disgust, `(_,_,_)` Anticipation.

### `cognitive` — CognitiveTier

Re-export of `bardo_primitives::InferenceTier` as `CognitiveTier`. Same type, different name for use in Golem-platform contexts.

### `event` — EventFabric, GolemEvent, EventPayload, Subsystem

`EventFabric` is a publish/subscribe bus for `GolemEvent`. `Subsystem` tags events by source (Cortical, Chain, Grimoire, Inference, etc.).

### `extension` — Extension trait and ExtensionRegistry

The hook system. Implement `Extension` to intercept lifecycle events:

```rust
#[async_trait]
impl Extension for MyExt {
    fn name(&self) -> &str { "my-ext" }
    fn layer(&self) -> u8 { 1 }
    fn depends_on(&self) -> &[&str] { &["heartbeat"] }

    async fn on_tool_call(&self, call: &ToolCall, ctx: &mut ToolCallCtx) -> anyhow::Result<ToolAction> {
        if call.name == "large_swap" {
            return Ok(ToolAction::Block("warden delay required".into()));
        }
        Ok(ToolAction::Allow)
    }
}
```

`ExtensionRegistry` does topological ordering across all registered extensions before firing hooks. Call `registry.build()` once after registration; it panics on cycles or missing dependencies. Hook methods return action types where appropriate:

- `on_input` → `InputAction` (Pass, Transform, Suppress)
- `on_tool_call` → `ToolAction` (Allow, Block, Modify)

Full hook list: `on_session`, `on_input`, `on_before_agent_start`, `on_agent_start`, `on_turn_start`, `on_context`, `on_before_provider_request`, `on_tool_call`, `on_tool_execution_start/update/end`, `on_tool_result`, `on_turn_end`, `on_agent_end`, `on_after_turn`, `on_system_prompt`, `on_steer`, `on_send_message`, `on_debug`, `on_error`, `on_end`.

### `id` — GolemId

UUID-backed runtime identifier. Ephemeral: not on-chain, not persisted across restarts. Used for intra-process cost attribution and log correlation.

```rust
let id = GolemId::new();
```

### `taint` — TaintedString, TaintLabel

`TaintedString` wraps a `String` with a `TaintLabel` provenance marker. Prevents unsanitized external data from flowing into privileged operations. The type system carries the taint, not runtime checks.

```rust
pub enum TaintLabel {
    External,
    UserInput,
    ChainData,
    Sanitized,
}
```

## Usage

```toml
[dependencies]
golem-core = { path = "../../crates/golem-core" }
```
