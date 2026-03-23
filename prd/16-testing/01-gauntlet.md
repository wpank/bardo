# The Gauntlet: Comprehensive Benchmark Suite [SPEC]

> **Version**: 1.1 | **Status**: Draft
>
> **Packages (TypeScript)**: `@bardo/gauntlet`, `@bardo/eval`, `@bardo/dev`, `@bardo/testnet`
>
> **Crates (Rust)**: `golem-runtime` (GolemHarness), `bardo-terminal` (event capture)
>
> **Depends on**: `./04-mirage.md` (live fork infrastructure specification), `./00-thesis-validation.md` (controlled experiments testing the mortality thesis), `./02-knowledge-quality.md` (insight quality scoring and vacuous reasoning detection), `./03-mechanism-testing.md` (property-based tests for every Golem subsystem)

> **Reader orientation:** This document specifies the Gauntlet (the unified benchmark suite that orchestrates all Bardo testing concerns into a single CI/CD-integrated runner). It belongs to Section 16 (Testing) and defines scenario libraries, regime coverage, regression detection, and the three-speed test tiers (Smoke/Nightly/Full). A Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) is the unit under test. See `prd2/shared/glossary.md` for full term definitions.

---

## Purpose

The Gauntlet is a unified benchmark runner that combines all testing concerns -- thesis validation, knowledge quality, mechanism correctness, and lifecycle evaluation -- into a single orchestrated suite. It is the CI/CD integration point and the authoritative answer to "does this build work?"

The Gauntlet operates at three speeds:

- **Smoke** (5 min): core mechanism tests + basic quality checks. Runs on every PR.
- **Nightly** (2-4 hours): full mechanism tests + backtest on 7-day Mirage replay + regime coverage.
- **Full** (24-48 hours): thesis validation factorial + ablation + multi-agent scenarios.

---

## Document Map

| Section | Topic                 |
| ------- | --------------------- |
| S1      | Scenario library      |
| S2      | Regime library        |
| S3      | Golden baselines      |
| S4      | Promptfoo integration |
| S5      | Multi-agent scenarios |
| S6      | GauntletRunner        |
| S7      | Regression detection  |
| S8      | Reporting             |
| S9      | CI/CD integration     |

---

## S1 -- Scenario Library

Each scenario is a self-contained test case with deterministic setup, execution, evaluation, and teardown. Scenarios are composable -- complex tests chain multiple scenarios.

### Scenario Interface

```typescript
interface GauntletScenario {
  /** Unique scenario identifier. */
  id: string;

  /** Human-readable name. */
  name: string;

  /** Category for filtering. */
  category: "market" | "mechanism" | "adversarial" | "multi_agent";

  /** Suite membership. */
  suites: Array<"smoke" | "nightly" | "full">;

  /** Estimated runtime. */
  estimatedDurationMs: number;

  /** Dependencies: other scenarios that must run first. */
  dependencies: string[];

  /** Setup: prepare the environment. */
  setup(ctx: ScenarioContext): Promise<void>;

  /** Run: execute the scenario. */
  run(ctx: ScenarioContext): Promise<ScenarioExecution>;

  /** Evaluate: check results against criteria. */
  evaluate(execution: ScenarioExecution): Promise<ScenarioVerdict>;

  /** Teardown: clean up resources. */
  teardown(ctx: ScenarioContext): Promise<void>;
}

interface ScenarioContext {
  anvil: AnvilManager;
  mirage: MirageManager | null;
  golemFactory: GolemFactory;
  grimoire: GrimoireStore;
  evalPipeline: EvaluationPipeline;
  testData: TestDataProvider;
  logger: Logger;
}

interface ScenarioExecution {
  scenarioId: string;
  startedAt: number;
  completedAt: number;
  durationMs: number;
  metrics: Record<string, number>;
  events: ScenarioEvent[];
  artifacts: Record<string, unknown>;
}

interface ScenarioVerdict {
  scenarioId: string;
  passed: boolean;
  criteria: Array<{
    name: string;
    passed: boolean;
    expected: string;
    actual: string;
  }>;
  notes: string[];
}
```

> **Extended:** Market scenarios (8), mechanism scenarios (8), adversarial scenarios (4), regime library, golden baselines, promptfoo configs, multi-agent scenarios (4), GauntletRunner implementation, regression detection, diagnosis and remediation guides, reporting formats, CI/CD integration, and package structure -- see [../../prd2-extended/16-testing/01-gauntlet-extended.md](../../prd2-extended/16-testing/01-gauntlet-extended.md)

---

## Package Structure

```
packages/gauntlet/
  src/
    runner.ts          # GauntletRunner
    scenarios/
      market/          # Market scenarios
      mechanism/       # Mechanism scenarios
      adversarial/     # Adversarial scenarios
      multi-agent/     # Multi-agent scenarios
    regimes/
      library.ts       # Regime sequence library
      detector.ts      # Regime detection
    baselines/
      golden/          # Golden baseline definitions
      checker.ts       # Baseline comparison
    regression/
      detector.ts      # Regression detection
      history.ts       # Metric history store
    promptfoo/
      configs/         # Promptfoo YAML configs
      runner.ts        # Promptfoo integration
    reporting/
      html.ts          # HTML dashboard generator
      json.ts          # JSONL output
      cli.ts           # CLI summary
      alerts.ts        # Alert channels
    types.ts           # Shared type definitions
  package.json
  tsconfig.json
```

Dependencies: `@bardo/testnet`, `@bardo/dev`, `@bardo/eval`, `promptfoo`, `vitest`, `fast-check`. Golem state and Grimoire data are accessed via the Golem runtime's HTTP API (not directly as npm packages — those live in the `golem-runtime` and `golem-grimoire` Rust crates).
