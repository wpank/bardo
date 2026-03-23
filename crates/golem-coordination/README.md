# golem-coordination

**Status: shell — no public API yet.**

Multi-agent coordination layer. Golems in a clade communicate not through direct messaging but through a shared pheromone field — a sparse signal space partitioned into three layers. Each layer has different propagation semantics and decay rates.

## Planned Public API

**Pheromone layers:**

- `THREAT` — danger signals. High decay rate; a threat pheromone deposited by one agent fades within a few ticks unless reinforced by others. Triggers defensive posture in neighbors. Used for: chain anomalies, liquidation risk, protocol exploits in progress.
- `OPPORTUNITY` — positive signals. Medium decay. Attracts agents toward high-alpha zones. Used for: arbitrage windows, yield spikes, liquidity events.
- `WISDOM` — slow-decay, high-weight signals deposited by agents with high epistemic fitness. Persists across many ticks. Used for: regime changes that proved durable, protocol parameter shifts, structural market state.

**`PropagationPolicy`** — controls how pheromones spread across the clade graph. Configures decay rate per layer, maximum propagation radius, and minimum depositor fitness threshold (so low-fitness agents can't pollute the wisdom layer).

**Clade membership sync** — maintains the set of active agents in the local clade, their current `BehavioralPhase`, and their relative positions in the pheromone field. Updates asynchronously via a background fiber.

**Bloodstain ingestion** — when an agent dies, it deposits a `Bloodstain`: a compressed record of the conditions that led to its death, the `DeathCause`, and its final `VitalityState`. Bloodstains are treated as high-weight wisdom signals for the clade members that observe them. A cluster of bloodstains in similar market conditions is a strong signal about a dangerous regime.

## System Position

`golem-coordination` depends on `golem-mortality` for `DeathCause` and `VitalityState`, and on `golem-core` for `BehavioralPhase`. The pheromone field itself is external infrastructure (likely a lightweight gossip protocol or shared-memory ring buffer for local clades).

The safety layer (`golem-safety`) gates all coordination writes behind a `Capability<CoordinationWrite>` so rogue extensions cannot pollute the shared field.
