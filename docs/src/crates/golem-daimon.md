# golem-daimon

`golem-daimon` is the affect engine for golems. It maps market events and environmental signals to PAD (Pleasure-Arousal-Dominance) vectors, which drive the golem's `BehavioralPhase` and influence how aggressively it acts during each tick.

## Features

- Map incoming `GolemEvent` values to PAD vector updates
- Maintain running `CorticalState` with exponentially weighted affect
- Classify the current `BehavioralPhase` from the PAD vector: calm, vigilant, aggressive, fearful
- Emit `PlutchikEmotion` classifications for logging and downstream use
- Thread-safe: `CorticalState` uses atomic operations for lock-free reads during the heartbeat

## Architecture

`golem-daimon` is in Layer 2 (Cognition). It runs as a background task that consumes the event fabric stream and continuously updates `CorticalState`. The heartbeat reads a `CorticalSnapshot` at the start of each tick via `golem-context`. This snapshot influences the gate step: a golem in a fearful phase applies tighter capital limits and higher confidence thresholds before proceeding to execution.

PAD vectors use `f64` at the API boundary but are stored as `f32` bit patterns inside `CorticalState` for atomic update semantics.
