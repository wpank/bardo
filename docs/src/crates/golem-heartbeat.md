# golem-heartbeat

`golem-heartbeat` implements the 9-step cognitive loop that runs once per tick. This is the golem's core decision cycle — the sequence of operations that takes raw market observations and produces (or declines to produce) an on-chain action.

## The 9-Step Pipeline

Each tick executes these steps in order:

1. **Observe** — collect new chain events, price updates, and pheromone signals from the environment
2. **Retrieve** — query the Grimoire for episodic, semantic, and procedural knowledge relevant to the current observations
3. **Analyze** — reason over the assembled `CognitiveWorkspace`: what is happening, and why?
4. **Gate** — apply capital limits, confidence thresholds, and policy constraints; abort the tick if conditions are not met
5. **Simulate** — run candidate actions through mirage-rs to estimate outcomes without committing
6. **Validate** — verify simulation results against expectations; discard actions that don't pass
7. **Execute** — submit the approved transaction through `golem-chain` and the Warden
8. **Verify** — confirm the transaction landed on-chain and read back the result
9. **Reflect** — update the Grimoire and cortical state based on what happened; adjust PLAYBOOK if warranted

## Features

- Drives the full 9-step pipeline via `golem-runtime`'s tick loop
- Assembles `CognitiveWorkspace` via `golem-context` at the start of each tick
- Routes inference calls at the correct `CognitiveTier` (`T0`/`T1`/`T2`) based on cost budget
- Emits `GolemEvent` values at each step for observability
- Respects `golem-safety` capability checks before execution

## Architecture

`golem-heartbeat` is in Layer 2 (Cognition). It orchestrates most of the other crates: context assembly, inference routing, tool execution, chain interaction, and Grimoire writes all happen inside the heartbeat pipeline. The runtime calls `heartbeat.tick()` and the heartbeat handles the rest.

Each step is instrumented. If any step returns an error or the gate step aborts, the tick ends cleanly without side effects. A golem that aborts 100% of its ticks is wasting metabolic USDC on inference calls; the epistemic fitness clock will eventually fire if this continues.
