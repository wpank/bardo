# golem-heartbeat

**Status: shell — no public API yet.**

The 9-step CoALA (Cognitive Architecture for Language Agents) decision pipeline. Every tick, the heartbeat runs the full cycle: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect. Each step is logged in a `DecisionCycleRecord` so the full reasoning trace is available for post-hoc analysis and succession.

## Planned Public API

**`HeartbeatEngine`** — the main type. Owns the pipeline state and drives the FSM.

**9-step pipeline:**

1. **observe** — collect fresh chain data, social signals, and coordination pheromones into the working context
2. **retrieve** — pull relevant Grimoire entries for the current market conditions (HDC similarity search)
3. **analyze** — run `golem-ta` regime detection, `golem-triage` anomaly scoring, update epistemic priors
4. **gate** — cost-aware routing decision via `CognitiveTier`. `T0` is pure pattern matching (no LLM call). `T1` is a fast small model. `T2` is full deliberation with a large model. The gate reads `VitalityState::phase` to restrict tier access — a `Terminal` agent cannot route to `T2`.
5. **simulate** — generate candidate actions and simulate their outcomes using the current world model
6. **validate** — check candidate actions against safety constraints, position limits, and `PolicyCage` rules
7. **execute** — submit the validated action via `golem-tools`; requires `Capability<Execute>`
8. **verify** — confirm the action landed on-chain as expected; update position state
9. **reflect** — update epistemic priors based on the action outcome; record prediction-outcome pair for `EpistemicClock`

**`DecisionCycleRecord`** — full log of a completed cycle: which tier was used, wall-clock time per step, the action taken (or skipped), the outcome, and the epistemic update applied. Written to the Grimoire for consolidation during NREM.

**Heartbeat FSM** — controls pipeline execution rate, pause/resume on vitality signals, and graceful shutdown sequencing. The FSM state is distinct from the lifecycle FSM in `golem-runtime`; the heartbeat can be paused while the Golem is still `Active`.

## Gate Logic

The gate step is where economic pressure becomes behavioral. Each tier has a `cost_multiplier` relative to the baseline. In `Conservation` phase, `T2` routing is blocked entirely. In `Declining`, only `T0` is available. This means a resource-constrained agent automatically reduces inference spend, extending its lifespan at the cost of decision quality — a reasonable trade when the alternative is immediate economic death.

## System Position

`golem-heartbeat` depends on most of the golem-* stack: `golem-core`, `golem-mortality` (for `VitalityState` and gating), `golem-grimoire` (for retrieval and reflection), `golem-triage` (for anomaly scores), `golem-ta` (for regime), `golem-safety` (for `Capability` and `PolicyCage`). It is the integration point for the full Golem reasoning loop.
