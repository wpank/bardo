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

**`DecisionCycleRecord`** — full log of a completed cycle. Fields carried:

| Field | Type | Notes |
|---|---|---|
| `tier_used` | `CognitiveTier` | T0 / T1 / T2 |
| `step_durations_ms` | `[u64; 9]` | Wall-clock per CoALA step |
| `action` | `Option<Action>` | `None` if gate blocked execution |
| `outcome` | `Option<ExecutionOutcome>` | `None` if action was skipped |
| `epistemic_delta` | `EpistemicUpdate` | Prior update from reflection step |
| `prediction_pair` | `Option<(Prediction, Outcome)>` | Fed to `EpistemicClock` for scoring |
| `vitality_snapshot` | `VitalityState` | Phase at cycle start |
| `cost_usd` | `f64` | Actual inference spend for this tick |

Records are sealed immutable at cycle end and written to the Grimoire. During NREM consolidation, the `prediction_pair` fields across many records are replayed to update signal weights via the TA metabolism loop (see Cybernetic Feedback below).

**Heartbeat FSM** — controls pipeline execution rate, pause/resume on vitality signals, and graceful shutdown sequencing. The FSM state is distinct from the lifecycle FSM in `golem-runtime`; the heartbeat can be paused while the Golem is still `Active`.

## Gate Logic

The gate step is where economic pressure becomes behavioral. Each tier has a `cost_multiplier` relative to the baseline. In `Conservation` phase, `T2` routing is blocked entirely. In `Declining`, only `T0` is available. This means a resource-constrained agent automatically reduces inference spend, extending its lifespan at the cost of decision quality — a reasonable trade when the alternative is immediate economic death.

Phase-to-tier access matrix:

| `VitalityState::phase` | T0 | T1 | T2 |
|---|---|---|---|
| `Nominal` | yes | yes | yes |
| `Conservation` | yes | yes | **blocked** |
| `Declining` | yes | **blocked** | **blocked** |
| `Terminal` | yes | **blocked** | **blocked** |

`Terminal` matches `Declining` on tier access. The distinction is that `Terminal` also suppresses the simulate and validate steps — the agent can still tick T0 pattern-matching but will not generate new candidate actions.

## Cognitive Tier Costs

Each `CognitiveTier` maps to a cost class and expected call frequency across a healthy agent's tick distribution:

| Tier | Approx. cost / tick | Model class | Expected share of ticks |
|---|---|---|---|
| T0 | $0.00 | Deterministic FSM, no LLM call | ~80% |
| T1 | $0.003 | Haiku-class (small fast model) | ~15% |
| T2 | $0.01–$0.25 | Sonnet / Opus (large deliberation) | ~5% |

T2 cost varies with context window size at the time of the call. The daily spend cap in `GolemConfig::heartbeat::max_daily_cost_usd` is the hard ceiling; the tier distribution above is the expected steady-state for a `Nominal` phase agent running in a low-volatility regime. High-volatility regimes push more ticks to T1/T2 until the budget gate cuts them back.

## Cybernetic Feedback

The heartbeat participates in the TA signal metabolism loop: a closed feedback cycle where signals earn or lose compute budget based on prediction accuracy.

```
Signal activation → Prediction outcome resolution → Weight update → Budget reallocation → Next activation
```

Concretely:

1. During **analyze**, `golem-ta` produces signal activations with attached predictions and confidence scores.
2. After **verify**, the execution outcome is known. The heartbeat seals the `prediction_pair` into `DecisionCycleRecord`.
3. During NREM consolidation (offline, not on the hot path), the Grimoire replays prediction pairs across many records. Signals with high `accuracy_i` and `info_gain_i` (entropy reduction) accumulate weight; signals that don't predict well lose budget share.
4. At the next Theta tick, updated weights and budget allocations are loaded back into the analyze step.

This is the Hebbian micro-update (per prediction resolution) paired with replicator macro-update (per Theta tick) described in the cybernetic loops spec (`prd/13-runtime/21-cybernetic-loops.md`). The net effect: signals that predict well get more compute; signals that don't, die. The heartbeat is the sensor in this loop — it produces the prediction-outcome pairs that drive the controller.

## System Position

`golem-heartbeat` depends on most of the golem-* stack: `golem-core`, `golem-mortality` (for `VitalityState` and gating), `golem-grimoire` (for retrieval and reflection), `golem-triage` (for anomaly scores), `golem-ta` (for regime), `golem-safety` (for `Capability` and `PolicyCage`). It is the integration point for the full Golem reasoning loop.
