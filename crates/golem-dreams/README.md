# golem-dreams

**Status: shell — no public API yet.**

Off-cycle cognitive processing scheduled by `VitalityState`. When the Golem's heartbeat FSM transitions to the `Dreaming` lifecycle state, this crate drives two distinct phases: NREM replay (Grimoire consolidation) and REM imagination (counterfactual generation).

## Planned Public API

**`DreamScheduler`** — driven by `VitalityState`. Decides when to enter each dream phase and for how many ticks to run it. A Golem in `Conservation` phase gets longer NREM cycles (needs to consolidate under resource pressure). One in `Thriving` phase gets more REM time (can afford to explore).

**NREM replay** — walks recent Grimoire entries and runs consolidation passes: merging redundant beliefs, updating confidence based on subsequent observations, and pruning entries whose `demurrage`-adjusted confidence has fallen below the archive threshold. Consolidation is idempotent — running NREM twice on the same entries produces the same result as running it once. This is load-bearing: the scheduler may run NREM multiple times if a crash occurs mid-cycle.

**REM imagination** — counterfactual scenario generation weighted by the current emotional arc from `golem-daimon`. The emotional valence shapes which scenarios get generated:

- High fear (low pleasure, high arousal) → downside scenarios: liquidation, exploit, regime collapse. The agent imagines what could go wrong and how it would respond.
- High joy (high pleasure, moderate arousal) → upside scenarios: trend extension, yield compounding, favorable protocol changes.
- High surprise → novel scenarios: regime changes with no historical analogue, designed to test the agent's adaptability rather than its pattern matching.

Counterfactuals are not stored as beliefs — they are temporary working memory used to update the agent's strategy and then discarded.

**`DreamRecord`** — a log entry for a completed dream cycle. Records which phase ran, how many Grimoire entries were processed in NREM, how many counterfactuals were generated in REM, and what strategy updates (if any) were applied.

## System Position

`golem-dreams` depends on `golem-mortality` for `VitalityState` and `BehavioralPhase`, `golem-grimoire` for Grimoire access, and `golem-daimon` for the PAD vectors that weight REM scenario generation. The `golem-runtime` lifecycle FSM controls when the Golem enters and exits `Dreaming` state.

The dream phase is intentionally offline from market interaction — no chain reads, no LLM inference calls during NREM, minimal compute during REM. Dream cycles should be cheap.
