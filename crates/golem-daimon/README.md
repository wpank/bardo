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

## Mortality-Specific Emotions

Three affect states specific to mortal agent design — agents that hold real capital, spend it on execution, and die when it runs out:

**Economic anxiety** — low pleasure, moderate arousal, low dominance. The Golem watches its USDC balance deplete with each transaction. This is Jonas's needful freedom: the organism is free from any particular material but trapped in its dependence on material exchange. Survival pressure is not metaphorical; it feeds directly into PAD.

**Epistemic anxiety** — awareness that the world model is drifting. The Golem knows it's becoming wrong but can't pinpoint where. Moderate arousal, low dominance. Distinct from economic anxiety: the threat is informational, not financial, but the affect signature is similarly destabilizing.

**Terminal acceptance** — at death, survival pressure drops to zero. Low arousal, low dominance, neutral-to-positive pleasure from release. The agent's priors no longer need defending. Terminal acceptance is the most epistemically honest state the system produces: nothing left to protect means nothing left to rationalize.

## Somatic Markers in Practice

After repeated exposure, certain volatility fingerprints become pre-associated with the affect they historically produced. This allows fast affective responses before full analysis completes — pre-cognitive gut feelings rather than deliberated conclusions.

Bechara et al. (2000) showed anticipatory skin conductance responses precede conscious awareness in the Iowa Gambling Task. The somatic signal arrives before the reasoned explanation. Cabrera-Paniagua (2023) demonstrated that agents with somatic markers achieve higher Sharpe ratios on S&P 500 and Dow Jones data compared to agents without them.

The implementation stores somatic marker associations in a rolling history indexed by technical configuration hash. When a new market state matches a historical fingerprint, the associated affect is injected into the PAD update pipeline before appraisal runs.

## Behavioral Effects

PAD coordinates feed behavior directly, not through a symbolic layer:

- High arousal reduces risk tolerance. Position sizing scales down as arousal climbs.
- Low dominance triggers conservation mode: the agent reduces activity, widens spreads, avoids initiating new positions.
- Arousal modulates exploration temperature following Go-Blend (Barthet et al., 2022): high arousal narrows the action distribution, low arousal widens it.
- Negativity bias is weighted at 1.6x following Baumeister (2001), consistent with the Kahneman-Tversky empirical findings on loss aversion.

## Four-Factor Retrieval

Extends the Park et al. Generative Agents (2023) three-factor retrieval model (recency, importance, relevance) with emotional congruence as a fourth factor. Bower (1981) established mood-congruent memory: emotional state at retrieval time biases which memories surface.

A panicking Golem will preferentially retrieve memories of past panics. This creates a feedback loop the system explicitly counters with contrarian injection: 15% of retrievals are forced to opposite-emotion entries drawn from rolling windows. A panicking Golem is forced to recall past successes. A content Golem is forced to consider prior warnings. The contrarian fraction is configurable but defaults to 15%.

## Empirical Validation

The affect architecture rests on direct empirical results:

Zhang et al. (SIGDIAL, 2024) found self-emotion changes approximately 50% of agent decisions in social simulation. Gadanho (JMLR, 2003) showed the ALEC combined affect-cognition architecture produces 40% fewer collisions than cognition alone. The Cabrera-Paniagua (2023) Sharpe ratio improvement is the most directly relevant finding: financial agents with somatic markers outperform agents without them on real index data.

The Daimon is architectural, not decorative.

## System Position

`golem-daimon` depends on `golem-core` for `CorticalState` (it writes PAD coordinates atomically into the cortical state) and `golem-coordination` for contagion signals. `golem-sonification` reads `CorticalSnapshot::arousal` and `pleasure` from the same cortical state.
