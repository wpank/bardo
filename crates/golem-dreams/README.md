# golem-dreams

**Status: shell — no public API yet.**

Off-cycle cognitive processing scheduled by `VitalityState`. When the Golem's heartbeat FSM transitions to the `Dreaming` lifecycle state, this crate drives two distinct phases: NREM replay (Grimoire consolidation) and REM imagination (counterfactual generation).

## Dream Cycle

The dream cycle is a state machine:

```
WAKING -> HYPNAGOGIC ONSET -> NREM REPLAY -> REM IMAGINATION -> INTEGRATION -> HYPNOPOMPIC RETURN -> WAKING
```

Each transition is driven by `DreamScheduler` responding to `VitalityState` and `BehavioralPhase`. A Golem in `Conservation` phase gets longer NREM cycles (needs to consolidate under resource pressure). One in `Thriving` phase gets more REM time (can afford to explore).

## Hypnagogia

Hypnagogia — the threshold state between waking and sleep — is the first-of-kind mechanism in this crate. It is not a warmup phase. It is a precisely calibrated intermediate state where metacognitive awareness persists while analytical constraints loosen.

**ThalamicGate** progressively blocks live market data from 100% to 0% across the onset phase, matching Hori stages H1–H4. Magnin et al. (2010) showed that thalamic deactivation precedes cortical deactivation by 8 minutes 39 seconds during sleep onset — the gate models that temporal offset.

**ExecutiveLoosener** raises inference temperature while partially relaxing analytical constraints. This is not noise injection. It is a controlled reduction of the prior's grip on the posterior.

**DaliInterrupt** produces partial completions at elevated temperature, capped at 80 tokens, evaluated by a lower-temperature observer pass. The Edison/Dalí steel-ball technique made computational: hold a steel ball while falling asleep; it drops the moment you cross into sleep and wakes you at the threshold. Lacaux et al. (MIT, 2021) showed that N1 sleep threshold produces 83% creative insight vs. 30% while fully awake. The creative sweet spot is not noise — it is that threshold.

## Planned Public API

**`DreamScheduler`** — driven by `VitalityState`. Decides when to enter each dream phase and for how many ticks to run it. A Golem in `Conservation` phase gets longer NREM cycles (needs to consolidate under resource pressure). One in `Thriving` phase gets more REM time (can afford to explore).

**NREM replay** — walks recent Grimoire entries and runs consolidation passes: merging redundant beliefs, updating confidence based on subsequent observations, and pruning entries whose `demurrage`-adjusted confidence has fallen below the archive threshold. Consolidation is idempotent — running NREM twice on the same entries produces the same result as running it once. This is load-bearing: the scheduler may run NREM multiple times if a crash occurs mid-cycle.

Episode selection follows Mattar and Daw's utility-weighted replay: episodes are chosen proportional to their expected learning value, not recency. High-value episodes (large prediction error, high emotional salience) are replayed more often. Replay is bidirectional — both forward (anticipatory) and reverse (credit assignment). Buzsáki (2015): sharp-wave ripples compress minutes of waking experience into 100ms bursts during NREM. This crate models that compression.

**REM imagination** — counterfactual scenario generation weighted by the current emotional arc from `golem-daimon`. The emotional valence shapes which scenarios get generated:

- High fear (low pleasure, high arousal) → downside scenarios: liquidation, exploit, regime collapse. The agent imagines what could go wrong and how it would respond.
- High joy (high pleasure, moderate arousal) → upside scenarios: trend extension, yield compounding, favorable protocol changes.
- High surprise → novel scenarios: regime changes with no historical analogue, designed to test the agent's adaptability rather than its pattern matching.

Counterfactual structure follows Pearl's structural causal models: interventions are represented as do-calculus operations on the Grimoire's causal graph, not simple conditional sampling. This allows the Golem to reason about what *would have happened* under a different action, not just what is statistically associated with similar states.

Hoel's overfitted brain hypothesis informs the REM design: dreaming is the brain's regularization pass, preventing overfitting to the specific sequence of daily experience. REM generates out-of-distribution scenarios precisely to avoid a Golem that has memorized the last 30 days of Base L2 behavior and fails on the 31st.

DreamerV3 (Hafner, 2025): agents trained entirely inside imagined trajectories outperform real-experience-only agents across 150+ tasks. The same principle applies here — the Golem that dreams more learns more, up to the point where dream quality exceeds real-experience quality.

Counterfactuals are not stored as beliefs — they are temporary working memory used to update the agent's strategy and then discarded.

**`DreamRecord`** — a log entry for a completed dream cycle. Records which phase ran, how many Grimoire entries were processed in NREM, how many counterfactuals were generated in REM, and what strategy updates (if any) were applied.

## Sleep-Time Compute

Lin et al. (2025) showed idle-time precomputation reduces test-time compute 5x while maintaining accuracy. The same logic applies to an agent that cannot afford to learn everything through costly direct experience — gas, slippage, opportunity cost all make real trades expensive teachers. Dreaming multiplies learning from N real trades to N × R episodes, where R is the replay factor.

REM also runs threat dreaming: simulated DeFi attack scenarios including flash crashes, oracle manipulation, and MEV extraction during periods of high arousal. The Golem that has imagined being exploited responds faster to an actual exploit attempt than one that has only ever observed benign conditions.

## System Position

`golem-dreams` depends on `golem-mortality` for `VitalityState` and `BehavioralPhase`, `golem-grimoire` for Grimoire access, and `golem-daimon` for the PAD vectors that weight REM scenario generation. The `golem-runtime` lifecycle FSM controls when the Golem enters and exits `Dreaming` state.

The dream phase is intentionally offline from market interaction — no chain reads, no LLM inference calls during NREM, minimal compute during REM. Dream cycles should be cheap.
