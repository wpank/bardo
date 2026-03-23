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

## Stigmergy

Indirect coordination via environmental modification. Grasse (1959). Ants deposit pheromones; golems deposit signals. No central orchestration — agents respond to the field, not to each other directly. The field is the communication medium.

## Pheromone Decay Semantics

| Layer | Decay Rate | Notes |
|-------|-----------|-------|
| `THREAT` | High | Fades within a few ticks unless reinforced by multiple agents. Prevents false alarms from lingering. |
| `OPPORTUNITY` | Medium | Persists long enough to attract nearby agents but doesn't outlast the window it describes. |
| `WISDOM` | Slow | Persists across many ticks. Requires minimum epistemic fitness to deposit — low-fitness agents cannot write to this layer. |

## Styx Relay

For remote clades, the pheromone field propagates through the Styx relay layer. Mycorrhizal network model (Simard 2012): signals travel between nodes without direct peer-to-peer communication. Local clades share a ring buffer; remote clades connect through the relay with additional latency and attenuation.

## Information Economics

Grossman-Stiglitz (1980): freely shared alpha is immediately priced in. Golems share threats and structural signals, never raw alpha. Coordination value comes from the things other agents can act on without destroying the edge.

Bloodstains are Zahavi handicap signals — the costliest possible signal, because the signaler paid with its existence. A cluster of bloodstains under similar market conditions is a strong prior about a dangerous regime. At ecosystem maturity the dead-to-living ratio is 27:1; the accumulated mortality record is the primary source of structural wisdom.

Lethe knowledge commons prices queries at $0.002 via x402 micropayments.

## System Position

`golem-coordination` depends on `golem-mortality` for `DeathCause` and `VitalityState`, and on `golem-core` for `BehavioralPhase`. The pheromone field itself is external infrastructure (likely a lightweight gossip protocol or shared-memory ring buffer for local clades).

The safety layer (`golem-safety`) gates all coordination writes behind a `Capability<CoordinationWrite>` so rogue extensions cannot pollute the shared field.
