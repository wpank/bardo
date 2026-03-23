# golem-context

**Status: shell — no public API yet.**

`CognitiveWorkspace` assembly and context policy enforcement. When the heartbeat pipeline's gate step decides to make an LLM inference call, `golem-context` is responsible for assembling what goes into the prompt — which memories to include, how much of the prediction history to surface, what intervention data is relevant. It also enforces token budgets so inference costs stay predictable.

## Planned Public API

**`CognitiveWorkspace`** — the assembled context handed to an LLM call. Not a raw string — a structured object with typed slots. Each slot has its own token budget and fill strategy.

**Per-category token budgets:**

- `memory` — Grimoire entries retrieved for the current conditions. Budget scales with `CognitiveTier`: `T2` gets full budget, `T1` gets half, `T0` gets none.
- `prediction` — recent prediction-outcome pairs from `EpistemicClock`. Surfaces what the agent has been right and wrong about recently. Biased toward recent entries but includes notable failures regardless of recency.
- `intervention` — `Intervention<T>` values that have been gated into the workspace. Typed, so an intervention carrying a `LiquidationEvent` cannot be accidentally handled as a `RegimeChange`.

**`ContextPolicy`** — configures token budget allocations and fill strategies per cognitive tier. Policies are immutable once the runtime transitions to `Active` — the `PolicyCage` in `golem-safety` enforces this.

**`Intervention<T>`** — typed gate for workspace field access. An `Intervention<LiquidationEvent>` must be explicitly acknowledged and cleared; it cannot linger across ticks. The type parameter ensures the handler code is specific to the intervention type rather than pattern-matching on an untyped enum.

**Background fibers for context health monitoring** — long-running tasks that watch for workspace staleness (e.g. Grimoire entries whose confidence has degraded since retrieval) and flag them for eviction. Runs at low priority and does not block the inference path.

## Token Budget Scaling

T2 gets full budget, T1 gets half, T0 gets none (no LLM call). For T2 and T1, the assembler runs a binary search over the ranked symbol set — symbols ranked by PageRank score — to find the maximum set that fits within budget without overflowing.

## Intervention Lifecycle

An `Intervention<T>` must be explicitly acknowledged and cleared per tick. It cannot linger across ticks. The type parameter ensures the handler is specific to the intervention type — there is no untyped enum to pattern-match on, so a `LiquidationEvent` intervention cannot be accidentally routed to a `RegimeChange` handler.

## Allocation Efficiency

The workspace assembly runs on the hot path between the gate decision and the LLM call. Pre-allocated slot buffers are reused across ticks where `ContextPolicy` permits. No heap allocation per tick in the common case.

## System Position

`golem-context` sits between `golem-heartbeat`'s gate step and the actual LLM call. It depends on `golem-grimoire` for memory retrieval, `golem-mortality` for `EpistemicClock` data, and `golem-safety` for `ContextPolicy` enforcement. The workspace assembly happens on the hot path, so the design prioritizes allocation efficiency — pre-allocated slot buffers, reused across ticks where policy permits.
