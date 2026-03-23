# golem-triage

**Status: shell — no public API yet.**

Anomaly detection and event routing. Every observation that enters the Golem's perception layer passes through triage before reaching the heartbeat pipeline. Triage scores observations for surprise, detects changepoints, and routes them to the appropriate cognitive tier — fast pattern matching for low-surprise events, full LLM inference only for genuinely novel ones.

## Planned Public API

**Bayesian surprise scoring:**

- `GammaPoissonSurprise` — rate anomaly detection. Models event arrival rates as Gamma-Poisson conjugate. Computes the KL divergence between the posterior predictive after the new observation and the prior. High KL → high surprise. Used for: transaction frequency spikes, gas price anomalies, volume bursts.
- `NormalInverseGammaSurprise` — value shift detection. Models continuous signals as Normal with unknown mean and variance, using Normal-Inverse-Gamma conjugate. Detects sustained mean shifts rather than spikes. Used for: price level changes, yield drift, liquidity depth shifts.

**`BocpdDetector`** — Bayesian Online Changepoint Detection. Maintains a run-length distribution over recent data. At each step, updates the probability that a changepoint just occurred. When the run-length posterior mass concentrates on "0" (a new run just started), fires a changepoint event. More reliable than threshold-based detection because it explicitly models the uncertainty about when a regime change happened.

**`MidasRDetector`** — streaming graph anomaly detection based on the MIDAS-R algorithm. Processes a stream of (source, destination, timestamp) edges and scores each edge for anomalous frequency. Detects sudden bursts of repeated interaction patterns — relevant for protocol-level activity where certain address pairs become abnormally active.

**`CountMinSketch`** — approximate frequency estimation with configurable width and depth. Used to count symbol, address, or pattern occurrences over a sliding window without storing the full history. Feeds into both MIDAS-R and the gamma-Poisson scorer.

**`ThompsonRouter`** — Beta-distribution Thompson sampling for routing observations to cognitive tiers T0/T1/T2. Each tier has a Beta prior over its accuracy on past observations of similar surprise levels. At each routing decision, the router samples from each tier's Beta posterior and routes to the highest sample. This naturally explores less-used tiers when evidence is thin and exploits the best-performing tier as evidence accumulates.

**HDC codebook search via HNSW** — each observation is encoded as an `HdcVector` and queried against the HNSW approximate nearest-neighbor index of previously-seen patterns. Near-duplicate patterns (high similarity) get low surprise scores before any statistical model is consulted.

## System Position

`golem-triage` sits between raw chain data ingestion and the `golem-heartbeat` pipeline. It depends on `bardo-primitives` for `HdcVector` and on `golem-core` for the `CognitiveTier` type used in routing decisions.
