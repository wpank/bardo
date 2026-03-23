# golem-daimon

**Status: shell — no public API yet.**

Continuous affect engine based on the PAD (Pleasure-Arousal-Dominance) model. Every Golem has a `DaimonState` — a point in a three-dimensional affect space — that evolves as market conditions, prediction outcomes, and social signals arrive. The daimon is not decorative. PAD vectors feed directly into `golem-sonification` (arousal → event density, pleasure → timbre) and into `golem-dreams` (emotional valence shapes which counterfactuals get imagined during REM).

## Planned Public API

**`PadVector { pleasure: f64, arousal: f64, dominance: f64 }`** — the core state. All three axes are in [-1.0, 1.0]. A Golem at `(0.8, 0.2, 0.6)` is content, calm, and confident. One at `(-0.7, 0.9, -0.5)` is distressed, highly aroused, and feeling low control — the affect signature of a rapid drawdown with no good options.

**OCC appraisal** — Ortony-Clore-Collins appraisal theory. Each tick, market events are appraised along axes: desirability (did the outcome match the goal?), praiseworthiness (was the agent's action good?), appealingness (is the stimulus attractive?). Appraisal outputs feed the PAD update.

**Scherer component process model** — extends OCC with novelty, coping potential, and norm compatibility checks. High novelty + low coping potential → high arousal, low dominance. This is the affect signature a new market regime should produce.

**Somatic markers** — learned associations between market patterns and affect states. After repeated exposure, certain technical configurations (e.g. a specific volatility fingerprint before a liquidation cascade) become pre-associated with the affect they historically produced. Somatic markers allow fast affective responses before full analysis.

**Mood EMA** — PAD updates are applied via exponential moving average, not instant replacement. This prevents single-tick affect spikes from dominating behavior. The time constant is configurable; default is slow enough that mood persists across several hundred ticks.

**`PlutchikLabel`** — discrete emotion label derived from the PAD coordinates, following Plutchik's wheel. Used for logging and debugging; the actual affect computation stays in continuous PAD space. `joy`, `trust`, `fear`, `surprise`, `sadness`, `disgust`, `anger`, `anticipation`.

**Clade emotional contagion** — neighbors' affect states in the pheromone field weakly attract the local PAD vector. High fear in the clade raises local arousal and reduces dominance even before direct observation of the cause.

## System Position

`golem-daimon` depends on `golem-core` for `CorticalState` (it writes PAD coordinates atomically into the cortical state) and `golem-coordination` for contagion signals. `golem-sonification` reads `CorticalSnapshot::arousal` and `pleasure` from the same cortical state.
