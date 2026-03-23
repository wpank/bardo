# golem-ta

**Status: shell — no public API yet.**

Topological data analysis for market regime detection. Standard technical analysis indicators are path-dependent and noisy. TDA captures the shape of price-action data in a way that is robust to noise and sensitive to the structural features that matter: how many loops are present, how persistent they are, how the topology changes across time scales.

## Planned Public API

**`PersistenceDiagram`** — the central TDA object. Built from a filtration over price-volume data. Each point `(birth, death)` in the diagram represents a topological feature (connected component, loop) that appears at scale `birth` and disappears at scale `death`. Features with high persistence `(death - birth)` are structurally significant; low-persistence features are noise.

**`BettiCurves`** — time series of Betti numbers extracted from persistence diagrams at a sequence of scales. `β₀` counts connected components; `β₁` counts loops. A trending market has low `β₁` (few loops). A ranging market has high persistent `β₁`. A transition regime has unstable Betti curves — the topology is changing.

**Regime classification** — maps `BettiCurves` to one of four regimes:
- `Trending` — directional, low loop persistence
- `Volatile` — high short-lived features, rapid topology change
- `RangeBound` — stable high `β₁`, price oscillating inside a structure
- `Transition` — unstable curves, regime is changing but destination unknown

Transitions carry a confidence score derived from the stability of the current Betti curve. Hysteresis prevents rapid regime flipping — the classifier requires the new regime's signature to persist for a minimum number of ticks before committing.

**`TaCorticalExtension`** — implements `golem_core::extension::Extension`. Runs the TDA pipeline on each heartbeat tick, writes the current regime and confidence to `CorticalState`, and feeds regime-change events to `golem-triage` for surprise scoring.

## System Position

`golem-ta` is an Extension, not a dependency of the heartbeat pipeline. It registers at a specific layer in the extension registry and runs before the heartbeat's analyze step. The regime classification in `CorticalState` is consumed by `golem-heartbeat` when making gating decisions and by `golem-daimon` for appraisal.

The TDA computation is the most compute-intensive part of the analysis pipeline. The plan is to run it at a lower frequency than the full heartbeat — once every N ticks — and cache the result between updates.
