# golem-triage

**Status: shell — no public API yet.**

Anomaly detection and event routing. Every observation that enters the Golem's perception layer passes through triage before reaching the heartbeat pipeline. Triage scores observations for surprise, detects changepoints, and routes them to the appropriate cognitive tier — fast pattern matching for low-surprise events, full LLM inference only for genuinely novel ones.

## 4-Stage Pipeline

Every transaction that survives ingestion passes through four stages in sequence. Stages can short-circuit: a rejection at Stage 1 means Stages 2–4 never run.

| Stage | Name | Mechanism | Role |
|-------|------|-----------|------|
| 1 | Rule-based fast filters | Hard limits on gas, value, address allowlists/denylists | Rejects >90% of transactions in O(1) before any statistical work |
| 2 | Statistical anomaly | MIDAS-R (graph burst detection), DDSketch (quantile tracking), CountMinSketch (frequency estimation) | Runs on survivors from Stage 1; flags statistically unusual patterns |
| 3 | Contextual enrichment | Protocol state lookup, ABI resolution, historical ANN query against the HNSW index | Adds protocol-level context and nearest-neighbor distance before scoring |
| 4 | Upgraded scoring | HDC fingerprint binding, Bayesian surprise (KL divergence), Thompson routing, CuriosityScore composition | Produces the final `TriagedEvent` with tier suggestion |

## Planned Public API

**Bayesian surprise scoring:**

- `GammaPoissonSurprise` — rate anomaly detection. Models event arrival rates as Gamma-Poisson conjugate. Computes the KL divergence between the posterior predictive after the new observation and the prior. High KL → high surprise. Used for: transaction frequency spikes, gas price anomalies, volume bursts.
- `NormalInverseGammaSurprise` — value shift detection. Models continuous signals as Normal with unknown mean and variance, using Normal-Inverse-Gamma conjugate. Detects sustained mean shifts rather than spikes. Used for: price level changes, yield drift, liquidity depth shifts.

**`BocpdDetector`** — Bayesian Online Changepoint Detection. Maintains a run-length distribution over recent data. At each step, updates the probability that a changepoint just occurred. When the run-length posterior mass concentrates on "0" (a new run just started), fires a changepoint event. More reliable than threshold-based detection because it explicitly models the uncertainty about when a regime change happened.

**`MidasRDetector`** — streaming graph anomaly detection based on the MIDAS-R algorithm. Processes a stream of (source, destination, timestamp) edges and scores each edge for anomalous frequency. Detects sudden bursts of repeated interaction patterns — relevant for protocol-level activity where certain address pairs become abnormally active.

**`CountMinSketch`** — approximate frequency estimation with configurable width and depth. Used to count symbol, address, or pattern occurrences over a sliding window without storing the full history. Feeds into both MIDAS-R and the gamma-Poisson scorer.

**`ThompsonRouter`** — Beta-distribution Thompson sampling for routing observations to cognitive tiers T0/T1/T2. Each tier has a Beta prior over its accuracy on past observations of similar surprise levels. At each routing decision, the router samples from each tier's Beta posterior and routes to the highest sample. This naturally explores less-used tiers when evidence is thin and exploits the best-performing tier as evidence accumulates.

**HDC codebook search via HNSW** — each observation is encoded as an `HdcVector` and queried against the HNSW approximate nearest-neighbor index of previously-seen patterns. Near-duplicate patterns (high similarity) get low surprise scores before any statistical model is consulted.

## CuriosityScore

`CuriosityScore` is a hedge-weighted composite of five signals, producing an `f32` score alongside a `NoveltyReason` enum and a `tier_suggestion`. The scorer is `CuriosityScorer`; the output struct is `CuriosityScore`.

**`NoveltyReason` variants:**

| Variant | When it fires |
|---------|---------------|
| `Routine` | All signals within normal bounds; no anomaly detected |
| `MagnitudeDeviation` | Value or gas far outside the per-protocol NIG posterior |
| `PatternDeviation` | HDC nearest-neighbor distance above similarity threshold |
| `TimingAnomaly` | Arrival rate spike detected by the Gamma-Poisson model |
| `HighImpactEvent` | Large value move or contract deploy with elevated Bayesian surprise |
| `UnseenProtocolCombination` | Protocol + selector combination absent from the codebook |

Cold-start behavior: a `cold_start_factor` scalar is multiplied into the final score. It starts high (inflating scores early to encourage exploration) and anneals monotonically toward 1.0 as `episode_count` increases. This satisfies the invariant: scores decrease as episode_count grows, all else equal.

## HDC Transaction Encoding

`HdcTxEncoder` maps a `NormalizedTx` into a 10,240-bit binary sparse code using role-filler binding. Each field occupies a fixed role slot and is XOR-bound with a randomly-initialized but fixed codebook vector for that concept:

| Field | Encoding |
|-------|----------|
| `protocol` | Direct codebook lookup by `ProtocolFamily` |
| `selector` | 4-byte function selector mapped to random hypervector |
| `gas_tier` | Thermometer encoding across five tiers |
| `value_bucket` | Thermometer encoding across five ETH buckets |
| `topic clusters` | Bundled (majority vote) over log topic hypervectors |
| `tokens` | Bundled over ERC-20/ERC-721 token address hypervectors |

**Thermometer encoding for gas tiers:** instead of a single hypervector per tier, each tier accumulates the bundles of all lower tiers. Minimal gets one bundle; extreme gets all five. Adjacent tiers share bundles, so their Hamming distance is lower than non-adjacent tiers. This gives the ANN index a meaningful geometry over gas costs without conflating distinct tiers.

Target latency: mean under 10 microseconds per transaction (p99 under 15µs). Same `NormalizedTx` always produces the same `HdcVector`; determinism is required for fingerprint-based deduplication.

**ADWIN** (`Adwin` struct) — adaptive windowing with Hoeffding bounds for online drift detection. Maintains a sliding window that automatically shrinks when a significant change in the mean is detected. Complements BOCPD: BOCPD models the full run-length distribution and is better suited for abrupt regime changes, while ADWIN handles gradual drift in non-stationary environments. Both feed drift severity into the curiosity multiplier via `StageScores`.

## System Position

`golem-triage` sits between raw chain data ingestion and the `golem-heartbeat` pipeline. It depends on `bardo-primitives` for `HdcVector` and on `golem-core` for the `CognitiveTier` type used in routing decisions.
