# Evaluation Lifecycle: From Dev to Production [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Packages**: `@bardo/eval`, `@bardo/dev`, `@bardo/testnet`
>
> **Depends on**: `./04-mirage.md` (live fork infrastructure specification for Base/Ethereum replay), `./06-revision-guide.md` (12 numbered PRD revisions; Revisions 1, 6, 10, 11 cover self-correction constraints, calibration, reward hacking mitigation, and evaluation metrics)

> **Reader orientation:** This document specifies the four-phase evaluation lifecycle that a Golem (mortal autonomous agent) strategy must survive before reaching production. It belongs to Section 16 (Testing) and defines the progression from Trace Inspection (zero risk) through Backtesting, Paper Trading, and Canary Deployment (live capital). Each phase has explicit entry gates and exit conditions. Understanding Mirage (live fork replay infrastructure, see `./04-mirage.md`) is helpful context. See `prd2/shared/glossary.md` for full term definitions.

---

## Purpose

A Golem strategy progresses through four phases of increasing risk before reaching production. Each phase has explicit entry gates, evaluation criteria, and exit conditions. No strategy reaches live capital without surviving all four phases.

This document specifies the exact infrastructure, metrics, and gates for each phase, integrating Mirage (live fork replay from `./04-mirage.md`) and the production verification primitives (from `./06-revision-guide.md`).

---

## Document Map

| Section | Phase                  | Capital Risk |
| ------- | ---------------------- | ------------ |
| S1      | Trace Inspection       | None         |
| S2      | Backtesting via Mirage | None         |
| S3      | Paper Trading          | None         |
| S4      | Canary Deployment      | 1-5% TVL     |
| S5      | Walk-Forward Analysis  | Cross-phase  |
| S6      | Phase transition gates | Cross-phase  |
| S7      | Implementation         | Cross-phase  |

---

## S1 — Phase 1: Trace Inspection (Zero Risk)

### Environment

Local Anvil fork via `@bardo/testnet`. Single-block or short-sequence execution. No real market data — deterministic test scenarios.

### Scope

Individual heartbeat ticks and mechanism correctness. This phase verifies that each mechanism works in isolation before exposing it to real market dynamics.

### Tools

All verification comes from production-compatible primitives (no dev-only magic):

```typescript
interface TraceInspectionTools {
  /** Pre/post state reads for OutcomeVerification. */
  simulateContract: typeof import("viem").simulateContract;

  /** Multi-call simulation for multi-step behavior. */
  simulateCalls: typeof import("viem").simulateCalls;

  /** Gas estimation for cost verification. */
  estimateContractGas: typeof import("viem").estimateContractGas;

  /** Direct state reads for balances, positions, pool state. */
  readContract: typeof import("viem").readContract;
}

/**
 * OutcomeVerification: the core verification primitive from
 * `06-revision-guide.md`, Revision 1.
 *
 * Captures pre-state before a decision, post-state after execution,
 * and compares against simulation predictions.
 */
interface OutcomeVerification {
  /** Decision identifier. */
  decisionId: string;
  tick: number;

  /** Pre-state snapshot (balances, positions, pool state). */
  preState: {
    balances: Record<string, bigint>;
    positions: PositionSnapshot[];
    poolState: PoolStateSnapshot;
    timestamp: number;
  };

  /** Post-state snapshot after execution. */
  postState: {
    balances: Record<string, bigint>;
    positions: PositionSnapshot[];
    poolState: PoolStateSnapshot;
    timestamp: number;
  };

  /** Simulation prediction (from simulateContract before execution). */
  simulation: {
    expectedBalanceChanges: Record<string, bigint>;
    expectedGasUsed: bigint;
    expectedReturn: bigint;
    simulatedAt: number;
  };

  /** Invariant checks (Trace2Inv-style). */
  invariants: Array<{
    name: string;
    expression: string;
    passed: boolean;
    actual: string;
    expected: string;
  }>;

  /** Did the actual outcome match the simulation? */
  simulationAccurate: boolean;

  /** Deviation between simulation and actual (basis points). */
  deviationBps: number;
}
```

### Phase 1 Evaluation Criteria

| Criterion                                                 | Threshold                                   | Method                                             |
| --------------------------------------------------------- | ------------------------------------------- | -------------------------------------------------- |
| All mechanism tests pass (from `03-mechanism-testing.md`) | 100%                                        | Vitest suite                                       |
| No invariant violations                                   | 0 violations                                | OutcomeVerification invariant checks               |
| Simulation accuracy                                       | > 95% of simulations within 50bps of actual | simulateContract() vs receipt comparison           |
| Quality score distribution                                | Mean >= 0.5                                 | InsightQualityScore from `02-knowledge-quality.md` |
| No red flags in generated insights                        | 0 critical red flags                        | Red flag detector                                  |

### Gate to Phase 2

All Phase 1 criteria must pass. Automated: `bardo eval:lifecycle --phase 1` returns exit code 0.

---

## S2 — Phase 2: Backtesting via Mirage (Zero Risk)

### Environment

Mirage live fork (from `./04-mirage.md`) replaying real blocks from Base and/or Ethereum mainnet. The Golem sees real market dynamics — price movements, liquidity changes, oracle updates, gas fluctuations — but cannot execute real transactions.

### Scope

Multi-day runs with real market dynamics. This phase tests whether the Golem can learn, adapt, and generate valuable knowledge in a realistic environment.

### Mirage Integration

```typescript
interface MirageBacktestConfig {
  /** Mirage session configuration. */
  mirage: {
    chain: "base" | "ethereum";
    rpcEndpoint: string;
    anvilPort: number;
    protocols: string[];
    blockRange: {
      start: number;
      end: number;
    } | null;
    /** If null, replay from current block. */
    realtimeReplay: boolean;
  };

  /** Golem strategy configuration. */
  strategy: string;

  /** Duration in simulated days. */
  durationDays: number;

  /** Number of repetitions with different starting blocks. */
  repetitions: number;

  /** Walk-forward window configuration. */
  walkForward: WalkForwardConfig;
}

interface MirageBacktestResult {
  sessionId: string;
  config: MirageBacktestConfig;

  /** Per-repetition results. */
  repetitions: Array<{
    repetitionIndex: number;
    startBlock: number;
    endBlock: number;
    durationTicks: number;

    /** Regime sequence observed during this repetition. */
    regimes: Array<{
      regime: RegimeTag;
      startTick: number;
      endTick: number;
    }>;

    /** Final metrics. */
    metrics: BacktestMetrics;

    /** Walk-forward analysis results. */
    walkForward: WalkForwardResult;
  }>;

  /** Aggregate statistics across repetitions. */
  aggregate: {
    sharpe: { mean: number; std: number; ci95: [number, number] };
    maxDrawdown: { mean: number; std: number; p95: number };
    novelInsightRate: { mean: number; std: number };
    grimoireStaleRate: { mean: number; std: number };
    dreamYield: { mean: number; std: number };
  };

  /** Statistical validity checks. */
  validity: {
    pbo: PBOResult;
    dsr: { dsr: number; pValue: number; significant: boolean };
    regimeRobustness: {
      positiveSharpeRegimes: number;
      totalRegimes: number;
      robust: boolean;
    };
    monteCarlo: MonteCarloResult;
  };

  /** Phase 2 gate verdict. */
  passesGate: boolean;
  gateFailures: string[];
}

interface BacktestMetrics {
  sharpeRatio: number;
  sortinoRatio: number;
  maxDrawdown: number;
  cumulativePnlUsdc: number;
  winRate: number;
  profitFactor: number;
  novelInsightRate: number;
  grimoireStaleRate: number;
  epistemicFitness: number;
  dreamYield: number;
  admissionRate: number;
  predictionAccuracy: number;
}
```

### Walk-Forward Analysis

Walk-forward analysis prevents look-ahead bias by ensuring the strategy is always evaluated on data it has never seen. Two modes: anchored (growing training window) and rolling (fixed-size window).

```typescript
interface WalkForwardConfig {
  /** Mode: anchored (growing window) or rolling (fixed window). */
  mode: "anchored" | "rolling";

  /** Training window size in ticks (for rolling mode). */
  trainingWindowTicks: number;

  /** Test window size in ticks. */
  testWindowTicks: number;

  /** Step size: how many ticks to advance between windows. */
  stepTicks: number;

  /** Purging: gap between training and test to prevent leakage. */
  purgeGapTicks: number;

  /** Embargo: additional gap after test window. */
  embargoTicks: number;
}

interface WalkForwardResult {
  /** Per-window results. */
  windows: Array<{
    windowIndex: number;
    trainStart: number;
    trainEnd: number;
    testStart: number;
    testEnd: number;

    /** Training window metrics. */
    trainMetrics: BacktestMetrics;

    /** Test window metrics (out-of-sample). */
    testMetrics: BacktestMetrics;

    /** Regime during test window. */
    testRegime: RegimeTag;

    /** Ratio of test/train Sharpe (>1 = no overfit). */
    generalizationRatio: number;
  }>;

  /** Aggregate generalization ratio. */
  avgGeneralizationRatio: number;

  /** Fraction of windows where test Sharpe > 0. */
  positiveOOSRate: number;

  /** Per-regime test performance. */
  byRegime: Record<
    RegimeTag,
    {
      windows: number;
      avgSharpe: number;
      avgGeneralizationRatio: number;
    }
  >;

  /**
   * Reference: López de Prado, M. Advances in Financial Machine
   * Learning. Wiley, 2018. Chapters 11-15.
   */
}

/**
 * Combinatorial Purged Cross-Validation (CPCV).
 *
 * More rigorous than simple walk-forward: generates all combinations
 * of train/test splits with purging and embargo to prevent information
 * leakage. Produces PBO directly.
 *
 * Reference: SSRN:4686376.
 */
interface CPCVConfig {
  /** Number of partitions. Must be even. */
  partitions: number;

  /** Purge gap in ticks between train and test partitions. */
  purgeGapTicks: number;

  /** Embargo gap in ticks after test partitions. */
  embargoTicks: number;
}

function runCPCV(
  data: TickData[],
  config: CPCVConfig,
): { pbo: number; oosPerformance: number[] } {
  const partitions = partitionData(data, config.partitions);
  const halfN = config.partitions / 2;
  const combinations = generateCombinations(config.partitions, halfN);

  const oosPerformances: number[] = [];
  let overfitCount = 0;

  for (const trainIndices of combinations) {
    const testIndices = Array.from(
      { length: config.partitions },
      (_, i) => i,
    ).filter((i) => !trainIndices.includes(i));

    const trainData = applyPurgeAndEmbargo(
      partitions,
      trainIndices,
      testIndices,
      config.purgeGapTicks,
      config.embargoTicks,
    );

    const testData = testIndices.flatMap((i) => partitions[i]);
    const trainSharpe = computeSharpe(extractReturns(trainData));
    const testSharpe = computeSharpe(extractReturns(testData));

    oosPerformances.push(testSharpe);

    if (testSharpe < median(oosPerformances)) {
      overfitCount++;
    }
  }

  return {
    pbo: overfitCount / combinations.length,
    oosPerformance: oosPerformances,
  };
}
```

### Phase 2 Evaluation Criteria

| Criterion                      | Threshold                           | Method                                |
| ------------------------------ | ----------------------------------- | ------------------------------------- |
| PBO                            | < 0.5                               | Combinatorial Purged Cross-Validation |
| DSR                            | > 0 (significant at p < 0.05)       | Deflated Sharpe Ratio                 |
| Regime robustness              | Sharpe > 0 in >= 3 regimes          | Regime-conditional evaluation         |
| Monte Carlo robustness         | P95 drawdown < 2× backtest drawdown | 1000 Monte Carlo iterations           |
| Walk-forward generalization    | Avg ratio > 0.5                     | Walk-forward analysis                 |
| Knowledge quality              | Mean quality score >= 0.5           | InsightQualityEvaluator               |
| No critical mechanism failures | 0                                   | Mechanism health dashboard            |

### Gate to Phase 3

All Phase 2 criteria must pass. Automated check, but requires operator review before promotion. `bardo eval:lifecycle --phase 2` generates a report; operator reviews and runs `bardo eval:lifecycle --promote 3`.

---

## S3 — Phase 3: Paper Trading (Zero Capital Risk)

### Environment

Live Base mainnet data, real-time. The Golem observes real market conditions and makes decisions, but uses `simulateContract()` instead of executing transactions. No capital at risk.

### Scope

Real-time decision quality validation. The Golem's decisions are recorded with full context and compared against actual market outcomes.

```typescript
interface PaperTradingConfig {
  /** Chain to paper trade on. */
  chain: "base" | "ethereum";

  /** RPC endpoint for live data. */
  rpcEndpoint: string;

  /** Minimum duration in days. */
  minimumDurationDays: number;

  /** Strategy configuration. */
  strategy: string;

  /** Maximum simulated capital. */
  simulatedCapitalUsdc: number;

  /** Whether to require at least 1 regime change during the period. */
  requireRegimeChange: boolean;
}

interface PaperTradingResult {
  sessionId: string;
  startedAt: number;
  completedAt: number;
  durationDays: number;

  /** Every decision recorded with full context. */
  decisions: Array<{
    decisionId: string;
    tick: number;
    timestamp: number;

    /** What the Golem decided to do. */
    action: string;
    parameters: Record<string, unknown>;

    /** Simulation result (what would have happened). */
    simulation: {
      expectedPnl: number;
      expectedGas: bigint;
      expectedSlippage: number;
    };

    /** Actual market outcome (observed after the fact). */
    actualOutcome: {
      marketPrice: number;
      poolState: PoolStateSnapshot;
      actualSlippage: number;
    };

    /** Paper P&L: difference between entry and observed exit price. */
    paperPnl: number;

    /** Slippage estimation accuracy. */
    slippageAccuracy: number;
  }>;

  /** Aggregate metrics. */
  metrics: {
    paperSharpe: number;
    paperSortino: number;
    paperMaxDrawdown: number;
    paperCumulativePnl: number;
    avgSlippageAccuracy: number;
    decisionsPerDay: number;
    policyViolations: number;
  };

  /** Regime changes observed. */
  regimeChanges: Array<{
    tick: number;
    fromRegime: RegimeTag;
    toRegime: RegimeTag;
  }>;

  /** Knowledge quality during paper trading. */
  knowledgeMetrics: {
    insightsGenerated: number;
    avgQualityScore: number;
    trivialRate: number;
    admissionRate: number;
  };

  passesGate: boolean;
  gateFailures: string[];
}
```

### Phase 3 Evaluation Criteria

| Criterion                    | Threshold                 | Method                            |
| ---------------------------- | ------------------------- | --------------------------------- |
| Paper Sharpe                 | > 0                       | Cumulative paper P&L              |
| Slippage estimates           | Within 20% of actual      | Simulation vs observed comparison |
| Policy violations            | 0                         | PolicyCage enforcement log        |
| Regime change observed       | >= 1                      | Regime detection during period    |
| Minimum duration             | >= 7 days                 | Calendar time                     |
| Knowledge quality maintained | Mean quality score >= 0.5 | InsightQualityEvaluator           |

### Gate to Phase 4

Operator review required. Paper trading report + operator approval. `bardo eval:lifecycle --promote 4 --approve`.

---

## S4 — Phase 4: Canary Deployment (1-5% TVL)

### Environment

Live Base mainnet. Real capital. Small allocation (1-5% of total vault TVL).

### Scope

Production performance with automatic safety controls.

```typescript
interface CanaryConfig {
  /** Vault address for the canary allocation. */
  vaultAddress: `0x${string}`;

  /** Maximum allocation as fraction of vault TVL. */
  maxAllocationFraction: number;

  /** Maximum allocation in absolute USDC. */
  maxAllocationUsdc: number;

  /** Minimum canary duration in days. */
  minimumDurationDays: number;

  /** Automatic kill switch triggers. */
  killSwitches: {
    /** Max drawdown before automatic shutdown. */
    maxDrawdownPercent: number;

    /** Max loss in USDC before shutdown. */
    maxLossUsdc: number;

    /** Consecutive ticks with negative P&L before shutdown. */
    consecutiveNegativeTicks: number;

    /** Mechanism health degradation triggers. */
    mechanismHealth: {
      minDreamYield: number;
      minAdmissionRate: number;
      maxAffectiveHallucinationRate: number;
    };
  };
}

interface CanaryResult {
  sessionId: string;
  startedAt: number;
  completedAt: number | null;
  durationDays: number;
  status: "running" | "completed" | "killed" | "graduated";

  /** Kill reason if killed. */
  killReason: string | null;

  /** Real P&L metrics. */
  metrics: {
    realSharpe: number;
    realSortino: number;
    realMaxDrawdown: number;
    realCumulativePnl: number;
    realWinRate: number;
    gasSpent: number;
  };

  /** Mechanism health over the canary period. */
  mechanismHealth: {
    dreamYield: number;
    admissionRate: number;
    retrievalHitRate: number;
    phageDeathRate: number;
    predictionAccuracy: number;
    affectiveHallucinationRate: number;
  };

  /** Comparison with Phase 3 paper trading predictions. */
  paperVsReal: {
    sharpeCorrelation: number;
    pnlCorrelation: number;
    slippageBias: number;
  };

  passesGate: boolean;
  gateFailures: string[];
}
```

### Phase 4 Monitoring

Real-time monitoring with automatic kill switches and alerts.

```typescript
class CanaryMonitor {
  constructor(
    private config: CanaryConfig,
    private alerter: AlertChannel,
  ) {}

  async checkHealth(metrics: CanaryMetrics): Promise<CanaryHealthStatus> {
    const checks: Array<{ name: string; passed: boolean; reason: string }> = [];

    if (metrics.drawdownPercent > this.config.killSwitches.maxDrawdownPercent) {
      checks.push({
        name: "max_drawdown",
        passed: false,
        reason: `Drawdown ${metrics.drawdownPercent.toFixed(1)}% > max ${this.config.killSwitches.maxDrawdownPercent}%`,
      });
    }

    if (metrics.totalLossUsdc > this.config.killSwitches.maxLossUsdc) {
      checks.push({
        name: "max_loss",
        passed: false,
        reason: `Loss $${metrics.totalLossUsdc.toFixed(2)} > max $${this.config.killSwitches.maxLossUsdc}`,
      });
    }

    if (
      metrics.dreamYield <
      this.config.killSwitches.mechanismHealth.minDreamYield
    ) {
      checks.push({
        name: "dream_yield",
        passed: false,
        reason: `Dream yield ${metrics.dreamYield.toFixed(2)} < min ${this.config.killSwitches.mechanismHealth.minDreamYield}`,
      });
    }

    const failed = checks.filter((c) => !c.passed);
    if (failed.length > 0) {
      await this.alerter.send({
        severity: "critical",
        title: "Canary Kill Switch Triggered",
        body: failed.map((f) => f.reason).join("\n"),
      });
    }

    return {
      healthy: failed.length === 0,
      checks,
      recommendation: failed.length > 0 ? "kill" : "continue",
    };
  }
}
```

### Phase 4 Evaluation Criteria

| Criterion                  | Threshold              | Method                      |
| -------------------------- | ---------------------- | --------------------------- |
| Real Sharpe                | > 0                    | Live P&L tracking           |
| Max drawdown               | < configured threshold | Continuous monitoring       |
| All mechanisms healthy     | Within healthy ranges  | Cognitive quality dashboard |
| No safety violations       | 0                      | PolicyCage logs (+ optional Warden logs) |
| No kill switches triggered | 0                      | CanaryMonitor               |
| Minimum duration           | >= 14 days             | Calendar time               |
| Paper vs real correlation  | > 0.5                  | Sharpe and P&L correlation  |

### Gate to Full Deployment

Operator review + approval. All Phase 4 criteria must pass for >= 14 days. `bardo eval:lifecycle --graduate`.

---

## S5 — Walk-Forward Analysis (Detail)

Walk-forward analysis is used in Phase 2 but conceptually applies across phases. This section provides the full implementation.

### Anchored Walk-Forward

```
Time ───────────────────────────────────────────────────>

Window 1: [TRAIN═══════════][purge][TEST═══]
Window 2: [TRAIN════════════════][purge][TEST═══]
Window 3: [TRAIN═══════════════════════][purge][TEST═══]
```

Training window grows; test window stays fixed. Simulates production use where the model has access to all historical data.

### Rolling Walk-Forward

```
Time ───────────────────────────────────────────────────>

Window 1: [TRAIN═══════════][purge][TEST═══]
Window 2:      [TRAIN═══════════][purge][TEST═══]
Window 3:           [TRAIN═══════════][purge][TEST═══]
```

Training window stays fixed size, slides forward. Tests adaptability: can the strategy work with limited history?

### Regime-Tagged Walk-Forward

Each test window is tagged with its dominant regime. Results are aggregated per-regime to identify regime-dependent performance.

```typescript
function tagWindowRegime(
  testData: TickData[],
  regimeDetector: RegimeDetector,
): RegimeTag {
  const regimeCounts = new Map<RegimeTag, number>();

  for (const tick of testData) {
    const regime = regimeDetector.classify(tick);
    regimeCounts.set(regime, (regimeCounts.get(regime) ?? 0) + 1);
  }

  let maxCount = 0;
  let dominantRegime: RegimeTag = "sideways";
  for (const [regime, count] of regimeCounts) {
    if (count > maxCount) {
      maxCount = count;
      dominantRegime = regime;
    }
  }

  return dominantRegime;
}
```

---

## S6 — Phase Transition Gates

### Summary

```
Phase 1 (Trace)  ─── automated ───> Phase 2 (Backtest)
Phase 2 (Backtest) ─ operator review ─> Phase 3 (Paper)
Phase 3 (Paper)  ─── operator approval ──> Phase 4 (Canary)
Phase 4 (Canary) ─── operator graduation ──> Production
```

No auto-promotion beyond Phase 1. Every promotion requires human judgment.

### Gate Checklist

```typescript
interface PhaseGateResult {
  phase: 1 | 2 | 3 | 4;
  criteria: Array<{
    name: string;
    threshold: string;
    actual: string;
    passed: boolean;
  }>;
  allPassed: boolean;
  recommendation: "promote" | "hold" | "revert";
  operatorApprovalRequired: boolean;
}

function evaluatePhaseGate(
  phase: number,
  metrics: Record<string, number>,
): PhaseGateResult {
  const gates: Record<
    number,
    Array<{
      name: string;
      check: () => boolean;
      threshold: string;
      actual: () => string;
    }>
  > = {
    1: [
      {
        name: "mechanism_tests",
        check: () => metrics.mechanismTestPass === 1,
        threshold: "100%",
        actual: () => `${metrics.mechanismTestPass * 100}%`,
      },
      {
        name: "invariant_violations",
        check: () => metrics.invariantViolations === 0,
        threshold: "0",
        actual: () => String(metrics.invariantViolations),
      },
      {
        name: "simulation_accuracy",
        check: () => metrics.simulationAccuracy > 0.95,
        threshold: ">95%",
        actual: () => `${(metrics.simulationAccuracy * 100).toFixed(1)}%`,
      },
      {
        name: "quality_score",
        check: () => metrics.avgQualityScore >= 0.5,
        threshold: ">=0.5",
        actual: () => metrics.avgQualityScore.toFixed(3),
      },
    ],
    2: [
      {
        name: "pbo",
        check: () => metrics.pbo < 0.5,
        threshold: "<0.5",
        actual: () => metrics.pbo.toFixed(3),
      },
      {
        name: "dsr",
        check: () => metrics.dsr > 0 && metrics.dsrPValue < 0.05,
        threshold: ">0, p<0.05",
        actual: () =>
          `${metrics.dsr.toFixed(3)} (p=${metrics.dsrPValue.toFixed(4)})`,
      },
      {
        name: "regime_robustness",
        check: () => metrics.positiveSharpeRegimes >= 3,
        threshold: ">=3 regimes",
        actual: () => `${metrics.positiveSharpeRegimes} regimes`,
      },
      {
        name: "monte_carlo",
        check: () => metrics.mcP95Drawdown < 2 * metrics.backtestDrawdown,
        threshold: "P95 < 2x backtest",
        actual: () =>
          `${(metrics.mcP95Drawdown * 100).toFixed(1)}% vs ${(metrics.backtestDrawdown * 100).toFixed(1)}%`,
      },
      {
        name: "walk_forward",
        check: () => metrics.avgGeneralizationRatio > 0.5,
        threshold: ">0.5",
        actual: () => metrics.avgGeneralizationRatio.toFixed(3),
      },
    ],
    3: [
      {
        name: "paper_sharpe",
        check: () => metrics.paperSharpe > 0,
        threshold: ">0",
        actual: () => metrics.paperSharpe.toFixed(3),
      },
      {
        name: "slippage_accuracy",
        check: () => metrics.slippageAccuracy > 0.8,
        threshold: ">80%",
        actual: () => `${(metrics.slippageAccuracy * 100).toFixed(1)}%`,
      },
      {
        name: "policy_violations",
        check: () => metrics.policyViolations === 0,
        threshold: "0",
        actual: () => String(metrics.policyViolations),
      },
      {
        name: "regime_change",
        check: () => metrics.regimeChanges >= 1,
        threshold: ">=1",
        actual: () => String(metrics.regimeChanges),
      },
      {
        name: "duration",
        check: () => metrics.durationDays >= 7,
        threshold: ">=7 days",
        actual: () => `${metrics.durationDays} days`,
      },
    ],
    4: [
      {
        name: "real_sharpe",
        check: () => metrics.realSharpe > 0,
        threshold: ">0",
        actual: () => metrics.realSharpe.toFixed(3),
      },
      {
        name: "kill_switches",
        check: () => metrics.killSwitchesTriggered === 0,
        threshold: "0",
        actual: () => String(metrics.killSwitchesTriggered),
      },
      {
        name: "safety_violations",
        check: () => metrics.safetyViolations === 0,
        threshold: "0",
        actual: () => String(metrics.safetyViolations),
      },
      {
        name: "duration",
        check: () => metrics.durationDays >= 14,
        threshold: ">=14 days",
        actual: () => `${metrics.durationDays} days`,
      },
      {
        name: "paper_vs_real",
        check: () => metrics.paperRealCorrelation > 0.5,
        threshold: ">0.5",
        actual: () => metrics.paperRealCorrelation.toFixed(3),
      },
    ],
  };

  const criteria = (gates[phase] ?? []).map((g) => ({
    name: g.name,
    threshold: g.threshold,
    actual: g.actual(),
    passed: g.check(),
  }));

  const allPassed = criteria.every((c) => c.passed);

  return {
    phase: phase as 1 | 2 | 3 | 4,
    criteria,
    allPassed,
    recommendation: allPassed
      ? "promote"
      : criteria.filter((c) => !c.passed).length > 2
        ? "revert"
        : "hold",
    operatorApprovalRequired: phase >= 2,
  };
}
```

---

## S6b — Diagnosis and Remediation: When a Phase Fails

Each phase can fail for different reasons. A Phase 1 failure is always a code bug. A Phase 4 failure might be an execution gap. This section maps each phase's failure modes to root causes and concrete next steps.

### Per-Phase Failure Decision Tree

#### Phase 1 Failures (Trace Inspection)

Phase 1 tests mechanism correctness on deterministic Anvil scenarios. Failures here are always code bugs or configuration errors, never strategy or market issues.

```
Phase 1 failed
├── Mechanism test suite fails
│   → This is a code bug. Read the test output. Fix the code. Re-run.
│   → Never a tuning problem — do not adjust parameters for Phase 1 failures.
│
├── Invariant violations
│   → An invariant you defined is being broken by the mechanism code.
│   → Either the code is wrong (fix it) or the invariant is too strict
│     (relax it with documented justification).
│
├── Simulation accuracy < 95%
│   → simulateContract() predictions don't match actual receipts.
│   → Check: are you simulating at the correct block? Is state stale?
│   → Common cause: simulating before a state change that affects the result.
│
└── Quality scores below threshold
    → The Golem produces insights below quality gates on test data.
    → Check the Curator prompt. Check the test episode quality.
    → If test episodes are unrealistically simple, quality will be low.
```

**Key principle**: never proceed to Phase 2 with Phase 1 failures. Phase 2 failures become ambiguous (is it the code or the market?) if Phase 1 didn't pass cleanly.

#### Phase 2 Failures (Backtesting via Mirage)

Phase 2 replays real market data. Failures here mean the strategy doesn't work in realistic conditions, or the evaluation is detecting overfitting.

```
Phase 2 failed
├── PBO > 0.5 (backtest overfitting)
│   ├── Too few data partitions → increase Mirage replay length
│   ├── Too many configurations tested → reduce to top 3 configs
│   ├── Walk-forward window too short → increase test window
│   └── Not using CPCV → switch from simple walk-forward to CPCV
│       with purging and embargo
│
├── DSR < 0 (no genuine alpha after correction)
│   ├── Strategy has no edge in these market conditions
│   │   → Try a different strategy template
│   │   → This is NOT a Golem problem — it's a strategy problem
│   ├── Selection bias: tested too many variants
│   │   → Reduce the number of strategy variants
│   └── Non-normal returns inflating raw Sharpe
│       → Check return distribution — heavy tails?
│       → DSR corrects for this; trust the DSR over raw Sharpe
│
├── Regime robustness fails (Sharpe > 0 in < 3 regimes)
│   ├── Strategy is regime-specific → two options:
│   │   ├── Accept it: restrict deployment to detected regime
│   │   │   → Add regime gate: only trade when regime matches
│   │   └── Fix it: add regime-switching logic
│   │       → The Golem should detect regime and switch strategies
│   └── Regime detection is wrong
│       → Check if Mirage replay contains enough regime diversity
│       → Ensure regime tags are computed correctly from market data
│
├── Monte Carlo P95 drawdown > 2× backtest drawdown
│   → Strategy is fragile — sensitive to execution order/slippage
│   → Increase slippage tolerance in strategy params
│   → Add trade size limits to reduce market impact
│   → Consider splitting large trades across ticks
│
└── Walk-forward generalization ratio < 0.5
    → In-sample performance doesn't predict out-of-sample
    → Classic overfit signal: strategy memorized past data
    → Simplify the strategy (fewer parameters = less overfitting)
    → Increase regularization (wider parameter bounds, less tuning)
```

#### Phase 3 Failures (Paper Trading)

Phase 3 uses live data but simulated execution. Failures here mean the strategy works in replay but not in real-time.

```
Phase 3 failed
├── Paper Sharpe < 0
│   ├── Simulation-to-reality gap
│   │   → Compare Mirage replay prices with live prices at same blocks
│   │   → If they diverge: Mirage fidelity problem (check protocol adapters)
│   │   → If they match: the strategy's edge disappears in real-time
│   ├── Look-ahead bias in Phase 2
│   │   → Phase 2 used information that wouldn't be available in real-time
│   │   → Check: are you reading block N+1 data when deciding at block N?
│   └── Latency sensitivity
│       → The strategy requires sub-second execution to capture the edge
│       → If simulateContract() adds >1s latency, the opportunity is gone
│       → Fix: pre-compute decisions, reduce tick interval
│
├── Slippage estimates off by >20%
│   → Slippage model doesn't account for actual market impact
│   → Calibrate slippage model against recent trade data
│   → Consider using the Uniswap Trading API quotes for better estimates
│
├── No regime change in 7 days
│   → Bad luck — markets were stable during the test period
│   → Extend the paper trading period
│   → Or artificially wait for a regime change before concluding
│
└── Policy violation detected
    → Safety layer caught something the Golem tried to do
    → This is good — the safety layer works
    → Fix the prompt/strategy that led to the violation attempt
    → NEVER weaken the policy to make Phase 3 pass
```

#### Phase 4 Failures (Canary Deployment)

Phase 4 uses real capital. Failures here mean execution-layer problems that don't show up in simulation.

```
Phase 4 failed
├── Kill switch: max drawdown exceeded
│   ├── Slippage model still wrong → recalibrate (see Phase 3)
│   ├── MEV: transactions being sandwiched → add MEV protection
│   │   (private mempool, Flashbots Protect, tx bundling)
│   ├── Gas estimation wrong → transactions failing/reverting
│   │   → Add gas buffer (1.2× estimated)
│   │   → Implement gas price aware execution (wait for lower gas)
│   └── Market moved faster than tick interval
│       → Reduce tick interval during volatile periods
│
├── Kill switch: mechanism health degraded
│   ├── Dream yield collapsed → check if Mirage vs. live data
│   │   changes episode quality
│   ├── Admission rate spiked or collapsed → quality pipeline
│   │   reacting to real data differently than replay
│   └── Affective hallucination spiked → live market events
│       trigger emotional states the training data didn't cover
│
└── Kill switch: consecutive negative ticks
    → Strategy's expected win rate doesn't match reality
    → Increase the consecutive-negative threshold if the strategy
      is high-variance (some strategies have 40% win rate but
      large winners)
    → Or: the strategy genuinely doesn't work with real execution
```

### Regression Between Phases

What it means when Phase N passes but Phase N+1 fails:

| Passed  | Failed  | Gap Name                  | Root Cause                                                                | Fix Direction                                                              |
| ------- | ------- | ------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Phase 1 | Phase 2 | Realism gap               | Mechanisms work in isolation but fail with real market dynamics           | Run more complex Phase 1 scenarios; add Mirage micro-replays to Phase 1    |
| Phase 2 | Phase 3 | Simulation-to-reality gap | Mirage replay ≠ live market; look-ahead bias; timing sensitivity          | Improve Mirage fidelity; audit for look-ahead bias; add latency simulation |
| Phase 3 | Phase 4 | Execution gap             | Paper trades assume perfect execution; real trades have MEV, gas, reverts | Add execution costs to paper trading; simulate MEV; add gas buffers        |

### When to Go Backwards

| From    | Back To | Criteria                                                                                        |
| ------- | ------- | ----------------------------------------------------------------------------------------------- |
| Phase 4 | Phase 3 | Kill switch triggered. Revert, diagnose, fix, re-paper-trade.                                   |
| Phase 3 | Phase 2 | Paper Sharpe significantly worse than backtest Sharpe (gap > 50%). Simulation fidelity problem. |
| Phase 3 | Phase 1 | Policy violations or mechanism failures. Code-level problem.                                    |
| Phase 2 | Phase 1 | PBO > 0.8 or DSR strongly negative. Fundamental strategy problem. Redesign from scratch.        |
| Any     | Phase 1 | After removing or adding a mechanism. Always re-validate from the ground up.                    |

### The "Start Simpler" Playbook

If the full Bardo stack fails at Phase 2, don't debug the full stack. Build up incrementally:

```
Step 1: Static agent, no learning, fixed PLAYBOOK.md
  → Does the strategy itself have alpha? (DSR > 0?)
  → If no: fix the strategy before adding any Golem mechanisms
  → If yes: proceed to Step 2

Step 2: Immortal with demurrage
  → Does knowledge decay improve Grimoire health?
  → Does the agent learn useful things? (quality score > 0.5?)
  → If no: the learning pipeline is broken — fix Curator prompt
  → If yes: proceed to Step 3

Step 3: Add epistemic mortality only (single clock)
  → Does the threat of death improve learning urgency?
  → Does the agent produce better knowledge under time pressure?
  → If no: single-clock mortality doesn't help — try economic-only
  → If yes: proceed to Step 4

Step 4: Add dreams
  → Does offline consolidation improve regime adaptation speed?
  → Does dream yield stay in 10-30% range?
  → If no: dreams aren't helping — keep them disabled
  → If yes: proceed to Step 5

Step 5: Full mortality (three clocks)
  → Does the full mortality system outperform single-clock?
  → Is the improvement worth the complexity?
  → If not clearly better: stick with single-clock

Step 6: Add daimon + memory services (Crypt/Oracle)
  → Does emotional appraisal improve decisions?
  → Does cross-death memory improve successor boot?
  → Add last because they depend on everything else working
```

At each step, the system must pass at least Gauntlet smoke + Phase 2 with DSR > 0 before adding the next mechanism. If a step doesn't pass, stop there — that configuration is your current ceiling. You can always try again later with different parameters.

### Phase Failure Diagnostic Interface

```typescript
interface PhaseFailureDiagnostic {
  phase: 1 | 2 | 3 | 4;
  gateResult: PhaseGateResult;

  /** Which criteria failed? */
  failures: Array<{
    criterion: string;
    expected: string;
    actual: string;
    rootCause: string;
    fixDirection: string;
  }>;

  /** Should we go back to a previous phase? */
  revertRecommendation: {
    revertTo: number | null;
    reason: string;
  };

  /** Should we simplify the configuration? */
  simplifyRecommendation: {
    currentLevel: number;
    suggestedLevel: number;
    mechanismsToDisable: string[];
    reason: string;
  };

  /** Concrete next steps, prioritized. */
  actionPlan: Array<{
    priority: number;
    action: string;
    expectedOutcome: string;
    verificationMethod: string;
  }>;
}

function recommendRemediationPath(
  phase: number,
  gateResult: PhaseGateResult,
  previousPhaseResults: PhaseGateResult[],
): PhaseFailureDiagnostic {
  const failures = gateResult.criteria.filter((c) => !c.passed);
  const diagnostics = failures.map((f) => diagnoseFailure(phase, f));

  const shouldRevert = determineRevert(phase, failures, previousPhaseResults);
  const shouldSimplify = determineSimplification(phase, failures);

  const actionPlan = prioritizeActions([
    ...diagnostics.map((d) => d.fixDirection),
    ...(shouldRevert.revertTo !== null
      ? [`Revert to Phase ${shouldRevert.revertTo}`]
      : []),
    ...(shouldSimplify.mechanismsToDisable.length > 0
      ? [`Disable: ${shouldSimplify.mechanismsToDisable.join(", ")}`]
      : []),
  ]);

  return {
    phase: phase as 1 | 2 | 3 | 4,
    gateResult,
    failures: diagnostics,
    revertRecommendation: shouldRevert,
    simplifyRecommendation: shouldSimplify,
    actionPlan,
  };
}
```

### CLI

```bash
# Diagnose why a phase failed
bardo eval:lifecycle:diagnose \
  --phase 2 \
  --results eval-results/phase2-001/ \
  --report html

# Get the "start simpler" recommendation
bardo eval:lifecycle:simplify \
  --current-config golem-config.yaml \
  --phase2-results eval-results/phase2-001/

# Verify a fix after reverting to a simpler config
bardo eval:lifecycle --phase 2 \
  --config golem-config-simplified.yaml \
  --mirage-endpoint ws://localhost:8546
```

---

## S7 — Implementation

### LifecycleManager

```typescript
class LifecycleManager {
  constructor(
    private mirageManager: MirageManager,
    private golemFactory: GolemFactory,
    private evalPipeline: EvaluationPipeline,
  ) {}

  /**
   * Run Phase 1: Trace Inspection.
   * Returns immediately with pass/fail result.
   */
  async runPhase1(strategyConfig: string): Promise<PhaseGateResult> {
    const testResults = await this.evalPipeline.runMechanismTests();
    const qualityResults =
      await this.evalPipeline.runQualityEval(strategyConfig);

    return evaluatePhaseGate(1, {
      mechanismTestPass: testResults.allPassed ? 1 : 0,
      invariantViolations: testResults.invariantViolations,
      simulationAccuracy: testResults.simulationAccuracy,
      avgQualityScore: qualityResults.avgScore,
    });
  }

  /**
   * Run Phase 2: Backtesting via Mirage.
   * Long-running: days of simulated time.
   */
  async runPhase2(config: MirageBacktestConfig): Promise<MirageBacktestResult> {
    const mirage = await this.mirageManager.createSession(config.mirage);
    const results: MirageBacktestResult["repetitions"] = [];

    for (let rep = 0; rep < config.repetitions; rep++) {
      const golem = await this.golemFactory.create(config.strategy);
      const startBlock = await mirage.getStartBlock(rep);

      await mirage.replayBlocks(startBlock, config.durationDays * 7200);

      const metrics = golem.collectMetrics();
      const walkForward = runWalkForward(
        golem.getTickData(),
        config.walkForward,
      );

      results.push({
        repetitionIndex: rep,
        startBlock,
        endBlock: startBlock + config.durationDays * 7200,
        durationTicks: config.durationDays * 2160,
        regimes: golem.getRegimeHistory(),
        metrics,
        walkForward,
      });

      await golem.shutdown();
    }

    await mirage.shutdown();

    const aggregate = aggregateResults(results);
    const validity = computeValidity(results);

    return {
      sessionId: generateSessionId(),
      config,
      repetitions: results,
      aggregate,
      validity,
      passesGate: checkPhase2Gate(aggregate, validity),
      gateFailures: getPhase2Failures(aggregate, validity),
    };
  }

  /**
   * Run Phase 3: Paper Trading.
   * Runs in real-time against live data.
   */
  async runPhase3(config: PaperTradingConfig): Promise<PaperTradingResult> {
    const golem = await this.golemFactory.create(config.strategy, {
      executionMode: "simulate_only",
      chain: config.chain,
      rpcEndpoint: config.rpcEndpoint,
    });

    await golem.runUntil({
      minDays: config.minimumDurationDays,
      requireRegimeChange: config.requireRegimeChange,
    });

    return golem.collectPaperTradingResults();
  }
}
```

### CLI

```bash
# Phase 1: Trace inspection (fast, automated)
bardo eval:lifecycle --phase 1 --strategy strategies/eth-dca.yaml

# Phase 2: Backtesting via Mirage (long-running)
bardo eval:lifecycle --phase 2 \
  --strategy strategies/eth-dca.yaml \
  --duration 60 \
  --repetitions 5 \
  --mirage-endpoint ws://localhost:8546 \
  --walk-forward rolling \
  --report html

# Phase 3: Paper trading (real-time)
bardo eval:lifecycle --phase 3 \
  --strategy strategies/eth-dca.yaml \
  --duration 7 \
  --chain base \
  --rpc https://base-mainnet.g.alchemy.com/v2/KEY

# Phase 4: Canary deployment
bardo eval:lifecycle --phase 4 \
  --strategy strategies/eth-dca.yaml \
  --vault 0x1234...abcd \
  --allocation 0.02 \
  --max-drawdown 5 \
  --duration 14

# Promote between phases (requires operator approval)
bardo eval:lifecycle --promote 3 --approve
bardo eval:lifecycle --graduate

# Check current phase status
bardo eval:lifecycle --status

# Generate cross-phase report
bardo eval:lifecycle --report \
  --phase1-results eval-results/phase1-001/ \
  --phase2-results eval-results/phase2-001/ \
  --phase3-results eval-results/phase3-001/ \
  --format html
```

### CI Integration

| Phase   | Trigger                  | Frequency                             |
| ------- | ------------------------ | ------------------------------------- |
| Phase 1 | Every PR                 | Automated                             |
| Phase 2 | Nightly on merge to main | Automated (results reviewed manually) |
| Phase 3 | Manual trigger           | On-demand                             |
| Phase 4 | Manual trigger           | On-demand                             |

All results stored in `eval-results/{phase}-{run-id}/` as JSONL + HTML reports.

---

## S8 — Continuous Evaluation: Fast and Slow Feedback Loops

Beyond the four lifecycle phases above, the production golem runs 14 feedback loops continuously. These loops are not dev-time evaluation -- they are production-time self-assessment that runs from the golem's first tick to its death.

### Five Fast Loops (Machine/Cognitive Speed)

All five share the Karpathy property: one metric, one arena, one gate (keep/discard), running fast. They operate at gamma or theta frequency with near-zero inference cost.

| Loop | What It Evaluates | Metric | Frequency | Cost |
|------|------------------|--------|-----------|------|
| **Confidence Calibration** | Does the LLM's stated confidence match reality? | Expected Calibration Error (ECE) | Per-resolution | ~Zero |
| **Context Attribution** | Which Grimoire entries actually helped? | Attribution score per context piece | Per-theta-tick | ~Zero |
| **Cost-Effectiveness** | Is expensive inference worth the accuracy gain? | delta-accuracy per dollar | Per-theta-tick | ~Zero |
| **Tool Selection** | Did we use the right tool for the job? | Execution quality vs. alternatives | Per-action | Low |
| **Adversarial Awareness** | Is someone exploiting the golem's behavior? | Slippage excess, timing anomalies | Per-action | ~Zero |

**Confidence calibration** uses isotonic regression to learn the mapping from LLM-stated confidence to actual accuracy, per (category, regime). LLMs are systematically overconfident [XIONG-2023]. Without calibration, the action gate trusts inflated self-assessment. The calibrator brings ECE from the typical 0.15-0.30 range down over time as resolution data accumulates. See [GUO-2017] for temperature scaling as an alternative.

**Context attribution** tracks which Grimoire entries and PLAYBOOK heuristics co-occur with successful predictions. A simplified credit/debit system (inspired by SHAP but simplified for real-time operation) builds per-entry context value scores. High-value entries get boosted in retrieval ranking; low-value entries get demoted.

**Cost-effectiveness** tracks accuracy gain per dollar of inference, per tier, per category. If T2 calls for a specific category produce no more learning than T1, the inference router shifts more ticks to T1. This is automatic cost optimization.

**Tool selection** compares actual execution quality against what alternative venues would have quoted (read-only simulation). Over time, builds a preference model per (action_type, pair, size_range).

**Adversarial awareness** monitors for persistent slippage excess (suggesting sandwich attacks), checks for sandwich signatures in block contents, and validates resolution integrity against TWAP reads [DABAH-2025].

### Three Slow Loops (Consolidation/Retrospective Speed)

| Loop | What It Evaluates | Timescale | Cost |
|------|------------------|-----------|------|
| **Shadow Strategy Testing** | Would different parameters produce better outcomes? | Weekly | Near-zero |
| **Reasoning Quality Review** | Is the golem's reasoning consistent and sound? | Per-dream-cycle | T1 |
| **Meta-Learning Evaluation** | Is the learning process itself improving? | Weekly + generational | Near-zero |

**Shadow strategy testing** runs counterfactual parameter sets alongside the real configuration. On every theta tick, the shadow system re-runs the gating decision with perturbed parameters and records what would have happened differently. Maximum 3 concurrent shadows, each varying one parameter. After one week, the retrospective review includes a shadow comparison.

**Reasoning quality review** evaluates reasoning traces during dream cycles using a 4-cell taxonomy: AlignedCorrect (right for right reasons), MisalignedCorrect (right for wrong reasons -- dangerous), AlignedIncorrect (wrong for right reasons -- valuable), MisalignedIncorrect (wrong for wrong reasons). The MisalignedCorrect cell is the most dangerous: a golem that is right by luck accumulates false confidence [KARPATHY-2026].

**Meta-learning evaluation** tracks whether the learning loops themselves are improving. Corrector convergence rate, dream yield, attention precision, heuristic half-life, and time-to-competence across generations. If `meta_learning_score` is negative for two consecutive weekly reviews, the golem's learning processes are degrading -- a form of senescence distinct from prediction accuracy decline.

### Loop Numbering (Authoritative)

The full 14-loop hierarchy uses the numbering from the mmo2/21-evaluation-architecture specification. See [07-fast-feedback-loops.md](07-fast-feedback-loops.md), [08-slow-feedback-loops.md](08-slow-feedback-loops.md), and [09-evaluation-map.md](09-evaluation-map.md) for the complete per-loop specifications with Rust implementation details.

---

## References

- [LOPEZ-DE-PRADO-2018] López de Prado, M. _Advances in Financial Machine Learning_. Wiley, 2018. Chapters 11-15. — *Covers combinatorial purged cross-validation and backtest overfitting; the statistical framework underpinning all four lifecycle phases.*
- [BAILEY-PBO-2015] Bailey, D.H. et al. "The Probability of Backtest Overfitting." SSRN:2326253, 2015. — *Quantifies how likely a backtest is to overfit given the number of trials; used in Phase 3 (qualification) to validate strategy robustness.*
- [BAILEY-DSR-2014] Bailey, D.H. & López de Prado, M. "The Deflated Sharpe Ratio." SSRN:2460551, 2014. — *Adjusts Sharpe ratios for multiple testing bias; applied in qualification and production phases.*
- [CPCV-2024] "Combinatorial Purged Cross-Validation." SSRN:4686376. — *Cross-validation method that purges overlapping time windows to prevent look-ahead bias in walk-forward evaluation.*
- [SHEPPERT-2026] Sheppert, A.P. "The GT-Score." JRFM 19(1):60, 2026. — *Composite risk metric combining return, drawdown, and tail risk; used alongside Sharpe for multi-dimensional lifecycle evaluation.*
- [XIONG-2023] Xiong, M. et al. "Can LLMs Express Their Uncertainty?" arXiv:2306.13063, 2023. — *Shows LLMs are systematically overconfident; motivates the confidence calibration checks at each lifecycle gate.*
- [GUO-2017] Guo, C. et al. "On Calibration of Modern Neural Networks." ICML, 2017. — *Introduces temperature scaling for neural network calibration; the foundational technique adapted for LLM confidence calibration.*
- [DABAH-2025] Dabah, L. & Tirer, T. "On Temperature Scaling and Conformal Prediction." ICML 2025. arXiv:2402.05806. — *Bridges temperature scaling and conformal prediction; supports the conformal interval approach used in production monitoring.*
- [KARPATHY-2026] Karpathy, A. "autoresearch." GitHub, March 2026. — *Demonstrates a single-metric, single-arena evaluation loop running at maximum speed; the structural pattern adopted across lifecycle phases.*
