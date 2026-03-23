# Mechanism Testing: Do the Mechanisms Work? [SPEC]

> **Version**: 1.1 | **Status**: Draft
>
> **Crates (Rust)**: `golem-runtime`, `golem-grimoire`, `golem-daimon`, `golem-mortality`, `golem-dreams`
>
> **Packages (TypeScript)**: `@bardo/eval` (orchestration and reporting only)
>
> **Depends on**: All mechanism PRDs in `../02-mortality/` (death clocks, behavioral phases, knowledge demurrage), `../05-dreams/` (NREM replay, REM counterfactual generation, dream scheduling), `../04-memory/` (Grimoire knowledge base, Crypt encrypted backup, Oracle cross-fleet RAG), `../03-daimon/` (PAD affect engine, mood-congruent retrieval, mortality-aware emotions)

> **Reader orientation:** This document specifies property-based tests and invariants for every Golem (mortal autonomous agent) subsystem: heartbeat engine, mortality clocks, dream cycles, memory pipeline, Daimon (the affect engine implementing PAD emotional state as a control signal), and knowledge transfer. It belongs to Section 16 (Testing) and answers "do the mechanisms work correctly?" rather than "does the thesis hold?" See `prd2/shared/glossary.md` for full term definitions.

---

## Purpose

This document specifies technical correctness tests for every Golem subsystem. While `00-thesis-validation.md` asks "does the thesis work?" and `02-knowledge-quality.md` asks "are insights valuable?", this document asks "do the mechanisms work correctly?" -- property-based tests, invariants, integration tests, and test fixtures for each mechanism.

Test framework: `cargo-nextest` + `proptest` (property-based testing) for Rust mechanisms. Coverage targets: 90%+ for safety-critical crates (`golem-safety`, `golem-mortality`, `golem-chain`), 80%+ for all other crates. Coverage measured by `cargo-llvm-cov`.

---

## Document Map

| Section | Mechanism                 | Priority |
| ------- | ------------------------- | -------- |
| S1      | Heartbeat and tick engine | P9       |
| S2      | Mortality clocks          | P9       |
| S3      | Dream cycles              | P8       |
| S4      | Memory pipeline           | P8       |
| S5      | Daimon (emotion engine)   | P8       |
| S6      | Phages                    | P7       |
| S7      | Replicants                | P7       |
| S8      | Knowledge transfer        | P8       |
| S9      | Test fixtures and harness | P7       |

---

## S1 -- Heartbeat and Tick Engine

The heartbeat is the Golem's clock. Every mechanism depends on it ticking correctly.

### Invariants

```typescript
const HEARTBEAT_INVARIANTS = {
  /** Tick interval must respect regime-dependent bounds. */
  TICK_INTERVAL_BOUNDS: {
    min: 15_000, // 15 seconds
    max: 120_000, // 120 seconds
    description: "Gamma ticks range 5-15s; theta ticks range 30-120s; delta fires every ~50 theta ticks",
  },

  /** State machine transitions must follow the defined FSM. */
  VALID_TRANSITIONS: {
    IDLE: ["OBSERVE"],
    OBSERVE: ["ANALYZE"],
    ANALYZE: ["DECIDE"],
    DECIDE: ["EXECUTE"],
    EXECUTE: ["REFLECT"],
    REFLECT: ["IDLE", "SLEEPING"],
    SLEEPING: ["IDLE"],
  },

  /** No tick runs without sufficient credit to cover at least T0 cost. */
  CREDIT_SUFFICIENCY: "creditBalance >= minTickCost(T0)",

  /** Credit deductions match the actual tier used. */
  CREDIT_ACCURACY: "creditDeducted === tierCost[actualTier]",

  /** Probe-to-escalation logic is deterministic for the same input. */
  DETERMINISTIC_ESCALATION: "same probeResults => same tier",

  /** No state is skipped in the FSM. */
  NO_STATE_SKIPPING: "each tick visits all 6 states in order",
};
```

### Property Tests

```typescript
import { describe, it, expect } from "vitest";
import * as fc from "fast-check";

describe("Heartbeat", () => {
  it("tick interval stays within bounds across all regimes", () => {
    fc.assert(
      fc.property(
        fc.record({
          volatility: fc.float({ min: 0, max: 5 }),
          gasPrice: fc.float({ min: 1, max: 500 }),
          liquidity: fc.float({ min: 0, max: 1 }),
        }),
        (regime) => {
          const interval = computeTickInterval(regime);
          return interval >= 15_000 && interval <= 120_000;
        },
      ),
    );
  });

  it("state transitions follow the defined FSM", () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.constantFrom(
            "IDLE",
            "OBSERVE",
            "ANALYZE",
            "DECIDE",
            "EXECUTE",
            "REFLECT",
            "SLEEPING",
          ),
          { minLength: 2, maxLength: 100 },
        ),
        (stateSequence) => {
          for (let i = 0; i < stateSequence.length - 1; i++) {
            const from = stateSequence[
              i
            ] as keyof typeof HEARTBEAT_INVARIANTS.VALID_TRANSITIONS;
            const to = stateSequence[i + 1];
            const valid = HEARTBEAT_INVARIANTS.VALID_TRANSITIONS[from];
            if (!valid?.includes(to)) return false;
          }
          return true;
        },
      ),
    );
  });

  it("credit deduction matches actual tier used", () => {
    fc.assert(
      fc.property(
        fc.record({
          probeResults: fc.array(fc.boolean(), {
            minLength: 11,
            maxLength: 11,
          }),
          cachedDecision: fc.boolean(),
          creditBalance: fc.float({ min: 0, max: 100 }),
        }),
        ({ probeResults, cachedDecision, creditBalance }) => {
          const tier = computeEscalationTier(probeResults, cachedDecision);
          const cost = TIER_COSTS[tier];
          if (creditBalance < cost) {
            return true; // Skip — insufficient credit prevents tick
          }
          const { deducted } = executeTick(
            probeResults,
            cachedDecision,
            creditBalance,
          );
          return Math.abs(deducted - cost) < 1e-10;
        },
      ),
    );
  });

  it("never ticks without sufficient credit", () => {
    fc.assert(
      fc.property(
        fc.float({ min: 0, max: 0.001 }), // Very low credit
        (creditBalance) => {
          const result = attemptTick(creditBalance);
          if (creditBalance < TIER_COSTS.T0) {
            return result.skipped === true;
          }
          return true;
        },
      ),
    );
  });
});
```

---

> **Extended:** Mortality clock tests (economic, epistemic, stochastic, coupling), dream cycle tests, memory pipeline tests (inner/outer loop), daimon tests, phage lifecycle tests, replicant tests, knowledge transfer tests, test fixtures/harness, and per-mechanism diagnosis/remediation guides with parameter sensitivity analysis -- see [../../prd2-extended/16-testing/03-mechanism-testing-extended.md](../../prd2-extended/16-testing/03-mechanism-testing-extended.md)
