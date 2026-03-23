# Mortality: Survival & Behavioral Phases [STUB]

> **Version**: 1.2 | **Status**: Cross-reference stub
>
> **Crate**: `golem-mortality`

> **Reader orientation:** This is a cross-reference stub pointing to the full mortality specification. Mortality is the architectural constraint at the heart of Bardo: each Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) has three independent death clocks, and any reaching zero kills it. The full spec lives in the `02-mortality/` section. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

The full mortality specification -- three clocks, epistemic fitness, stochastic hazard, behavioral phases, credit partitions, Eros/Thanatos spectrum, homeostatic variables, and the philosophical framework -- has moved to the dedicated mortality section.

**See [02-mortality/](../02-mortality/) for the full specification.**

Key subsections:

| File                                    | Topic                                                                 |
| --------------------------------------- | --------------------------------------------------------------------- |
| `02-mortality/00-thesis.md`             | Mortality thesis: six evidence domains proving that death drives intelligence |
| `02-mortality/01-architecture.md`       | Three death clocks (economic/epistemic/stochastic), VitalityState, five behavioral phases (Thriving through Terminal) |
| `02-mortality/04-economic-mortality.md` | USDC depletion mechanics, burn rate tracking, credit partition budgets |
| `02-mortality/06-thanatopsis.md`        | The four-phase Death Protocol: Acceptance, Settlement, Reflection, Legacy |
| `02-mortality/08-mortality-affect.md`   | How mortality pressure shapes the Daimon (affect engine): economic anxiety, epistemic vertigo |
| `02-mortality/10-clade-ecology.md`      | Superorganism dynamics: clade-level selection, population fitness     |

Mortality is the architectural constraint that makes intelligence economically viable. A Golem that cannot die has no reason to be efficient, no pressure to learn, and no incentive to share what it knows.

---

## Events Emitted

Mortality events track vitality changes and death approach. All events are variants of the `GolemEvent` enum emitted to the Event Fabric.

| Event | Trigger | Payload |
|---|---|---|
| `GolemEvent::VitalityUpdate` | Vitality score recalculated | `{ score: f64, phase: Phase, economic_clock: f64, epistemic_clock: f64, stochastic_clock: f64 }` |
| `GolemEvent::MortalityWarning` | Vitality below threshold | `{ score: f64, phase: Phase, projected_ttl: Duration, warning_type: WarningType }` |
| `GolemEvent::PhaseTransition` | Behavioral phase change | `{ from: Phase, to: Phase, vitality_score: f64, trigger: TransitionTrigger }` |
| `GolemEvent::DeathImminent` | Terminal phase entered | `{ remaining_credits: f64, projected_minutes: u64, active_positions: u32 }` |
