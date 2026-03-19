# golem-context

`golem-context` assembles the `CognitiveWorkspace` — the complete snapshot of a golem's mental state that is constructed at the start of each tick and passed through the 9-step heartbeat pipeline.

## Features

- Assembles `CognitiveWorkspace` from Grimoire, Daimon, cortical state, and recent events
- Provides a single coherent view of golem knowledge and affect for each tick
- Tracks working memory: the subset of Grimoire entries retrieved for the current tick
- Exposes tick-scoped scratch storage via `TickArena`

## Architecture

`golem-context` sits in Layer 2 (Cognition). At the start of each heartbeat tick, `golem-runtime` calls into `golem-context` to build a `CognitiveWorkspace`. That workspace is then passed immutably through the observe, retrieve, analyze, gate, simulate, validate, and execute steps. The verify and reflect steps may update Grimoire and cortical state, which feed into the next tick's workspace.

The workspace bundles together: current `CorticalSnapshot`, retrieved Grimoire entries, recent `GolemEvent` values from the event fabric, affect signals from the Daimon, and the per-tick `TickArena` for zero-cost scratch allocations.
