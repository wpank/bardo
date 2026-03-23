# Thesis Validation: Does Mortality Actually Work? [SPEC]

> **Version**: 1.1 | **Status**: Draft
>
> **Crates (Rust)**: `golem-runtime`, `golem-grimoire`, `golem-mortality`
>
> **Packages (TypeScript)**: `@bardo/eval` (experiment orchestration and statistical analysis)
>
> **Depends on**: `../02-mortality/00-thesis.md` (mortality thesis and three death clocks), `../02-mortality/11-immortal-control.md` (immortal baseline Golem specification for controlled comparison), `../03-daimon/09-evaluation.md` (Daimon affect engine evaluation and the 2x2x2 factorial design origin)

> **Reader orientation:** This document specifies the controlled experiments that test Bardo's core empirical claim: mortal agents outperform immortal ones. It belongs to Section 16 (Testing) and defines the 2x2x2 factorial design, ablation studies, and falsification criteria for the mortality thesis. You should understand what a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) is and what the Grimoire (the agent's persistent knowledge base) stores. See `prd2/shared/glossary.md` for full term definitions.

---

## The Question

The Bardo thesis is an empirical claim: mortal agents with dreams, memory, emotions, and knowledge transfer produce measurably better outcomes than immortal or ablated alternatives. This document specifies exactly how to test that claim — with controlled experiments, statistical rigor, and explicit falsification criteria.

This is not unit testing. This is experimental design. The goal is to determine whether the architecture produces _genuine value_ or whether it is an expensive way to produce agents that underperform a simple immortal baseline.

---

## Document Map

| Section | Topic                           |
| ------- | ------------------------------- |
| S1      | Controlled experiment framework |
| S2      | The 2×2×2 factorial design      |
| S3      | Ablation studies                |
| S4      | Statistical methodology         |
| S5      | Generational metrics            |
| S6      | Falsification criteria          |
| S7      | Implementation                  |
| S8      | References                      |

---

## S1 — Controlled Experiment Framework

### The Immortal Baseline

Every experiment compares against an immortal control — a Golem running identical strategy, identical funding, in identical market conditions, with all mortality clocks disabled. The immortal control specification is defined in `../02-mortality/11-immortal-control.md` (immortal baseline Golem specification with all mortality clocks disabled) and uses the `immortal: true` master flag.

The immortal Golem retains every capability a mortal Golem has: full heartbeat loop, all 11 probes, LLM tier escalation, complete Grimoire, all three learning loops, regime detection, decision cache, Clade (a group of related Golems sharing knowledge) participation, Sanctum tool access, and PolicyCage (on-chain smart contract enforcing safety constraints) enforcement. The only subtraction is mortality pressure: no death clocks, no behavioral phase transitions, no knowledge demurrage, no Eros/Thanatos spectrum, no Thanatopsis (the structured death protocol that generates a death testament for successor Golems) protocol.

This design ensures the experiment is fair. If mortal Golems win, they win despite having strictly fewer resources and capabilities than the immortal baseline.

### Experiment Structure

```typescript
interface ExperimentConfig {
  experimentId: string;
  name: string;
  description: string;

  /** Duration in days. Minimum 30 for statistical validity. */
  durationDays: number;

  /** Number of independent repetitions per configuration. */
  repetitions: number;

  /** Budget allocated to each group (mortal succession and immortal) in USDC. */
  budgetPerGroupUsdc: number;

  /** Chain to run on. Default: Base mainnet fork via Mirage. */
  chain: "base" | "ethereum";

  /** Mirage configuration for replaying real blocks. */
  mirageConfig: {
    rpcEndpoint: string;
    anvilPort: number;
    blockTime: number;
    protocols: string[];
  };

  /** Which mechanisms are enabled for the mortal group. */
  mortalConfig: {
    economicMortality: boolean;
    epistemicMortality: boolean;
    stochasticMortality: boolean;
    daimon: boolean;
    dreams: boolean;
    phages: boolean;
    memoryServices: boolean;
    knowledgeDemurrage: boolean;
  };

  /** Strategy template applied to all Golems (mortal and immortal). */
  strategyTemplate: string;

  /** Regime filter: only include periods matching these regimes. */
  regimeFilter: RegimeTag[] | null;

  /** Measurement collection interval in ticks. */
  measurementIntervalTicks: number;

  /** Statistical significance threshold. */
  alpha: number;

  /** Minimum effect size (Cohen's d) to declare practical significance. */
  minimumEffectSize: number;
}

type RegimeTag =
  | "high_volatility"
  | "low_volatility"
  | "trending_up"
  | "trending_down"
  | "sideways"
  | "high_gas"
  | "low_gas"
  | "low_liquidity"
  | "high_liquidity";
```

### ExperimentRun

```typescript
interface ExperimentRun {
  runId: string;
  experimentId: string;
  repetitionIndex: number;
  startedAt: number;
  completedAt: number | null;

  /** Market regimes observed during this run. */
  regimes: Array<{
    regime: RegimeTag;
    startTick: number;
    endTick: number | null;
    durationTicks: number;
  }>;

  /** Immortal group metrics, collected every measurementIntervalTicks. */
  immortalTimeSeries: MeasurementSnapshot[];

  /** Mortal group metrics, collected every measurementIntervalTicks. */
  mortalTimeSeries: MeasurementSnapshot[];

  /** Per-generation data for the mortal group. */
  mortalGenerations: GenerationRecord[];
}

interface MeasurementSnapshot {
  tick: number;
  timestamp: number;

  epistemicFitness: number;
  epistemicFitnessTrend: number;

  sharpeRatio: number;
  sortinoRatio: number;
  maxDrawdown: number;
  cumulativePnlUsdc: number;

  grimoireEntryCount: number;
  grimoireStaleRate: number;
  grimoireAvgConfidence: number;
  grimoireValidationRate: number;
  contextUtilization: number;
  contradictionCount: number;

  novelInsightRate: number;
  novelInsightAvgNovelty: number;
  cladeAdoptionRate: number;

  nicheOccupancy: number;
  behavioralVariance: number;
  convergenceTrend: number;

  /** Phage health (mortal only, null for immortal without demurrage). */
  phageDeathRate: number | null;
  phageConfirmationRate: number | null;
  inheritedValidationCoverage: number | null;

  /** Dream metrics (if enabled). */
  dreamYield: number | null;
  dreamHypothesisValidationRate: number | null;
  threatCoverage: number | null;

  /** Daimon metrics (if enabled). */
  affectiveHallucinationRate: number | null;
  decisionReversalRate: number | null;

  /** Cognitive quality dashboard metrics. */
  admissionRate: number;
  retrievalHitRate: number;
  heuristicSurvivalRate: number;
  decisionCacheHitRate: number;
  predictionAccuracy: number;
}

interface GenerationRecord {
  generationIndex: number;
  golemId: string;
  parentGolemId: string | null;
  startedAtTick: number;
  diedAtTick: number | null;
  lifespanTicks: number | null;
  deathCause: "economic" | "epistemic" | "stochastic" | "operator" | null;

  /** Ratchet score: novel contributions beyond inheritance. */
  ratchetScore: number;

  /** PLAYBOOK.md divergence from parent (cosine distance). */
  playbookDivergence: number;

  /** Number of inherited entries independently validated. */
  inheritedEntriesValidated: number;

  /** Number of inherited entries invalidated. */
  inheritedEntriesInvalidated: number;

  /** Number of novel Grimoire entries created. */
  novelEntriesCreated: number;

  /** Death testament quality (if applicable). */
  testamentCompleteness: number | null;
  testamentActionability: number | null;
  testamentPredictionCount: number | null;
}
```

---

## S2 — The 2×2×2 Factorial Design

From `../03-daimon/09-evaluation.md` (Daimon affect engine evaluation, S5): to isolate each system's contribution, evaluate a full factorial matrix across the three primary systems.

### Configuration Matrix

| #   | Mortality | Daimon | Memory Services | Name                                               |
| --- | --------- | ------ | --------------- | -------------------------------------------------- |
| 0   | Off       | Off    | Off             | Baseline (immortal, no affect, no memory services) |
| 1   | On        | Off    | Off             | Mortal-only                                        |
| 2   | Off       | On     | Off             | Daimon-only                                        |
| 3   | Off       | Off    | On              | Memory-only                                        |
| 4   | On        | On     | Off             | Mortal + Daimon                                    |
| 5   | On        | Off    | On              | Mortal + Memory                                    |
| 6   | Off       | On     | On              | Daimon + Memory                                    |
| 7   | On        | On     | On              | Full Bardo                                         |

Each configuration runs for 60 days with equivalent total budget. Minimum 10 runs per configuration = 80 total runs.

### What "Memory Services" Means

When Memory Services are _on_: Crypt (encrypted backup), Oracle (cross-fleet RAG), Lethe (anonymized commons), death testaments, bloodstain (a marker recording how a predecessor Golem died, so successors can avoid the same failure) mechanic, generational inheritance with confidence decay.

When Memory Services are _off_: inner-loop memory still works (LanceDB episodes, SQLite insights, PLAYBOOK.md, DecisionCache), but no outer-loop persistence, no cross-Golem retrieval, no inheritance, no death testaments.

### What "Daimon" Means

When Daimon (the affect engine implementing Pleasure-Arousal-Dominance emotional state as a control signal) is _on_: PAD mood vector, Plutchik emotion labels, Mode A/B dual appraisal, mood-congruent retrieval, behavioral modulation, mortality-aware emotions.

When Daimon is _off_: no emotional state, no mood, no affect-weighted retrieval. Decisions are purely cognitive.

### What "Mortality" Means

When Mortality is _on_: three clocks (economic, epistemic, stochastic), behavioral phases, knowledge demurrage, Eros/Thanatos, Thanatopsis protocol, Hayflick limit.

When Mortality is _off_: immortal mode per `../02-mortality/11-immortal-control.md`. Phase locked to Thriving, no death, no demurrage.

### Predicted Ordering

Based on the research synthesis (see `../03-daimon/09-evaluation.md` S5):

```
Full Bardo > Mortal + Memory > Mortal + Daimon > Mortal-only > Memory-only ≈ Daimon-only > Baseline
```

Specific predictions with research basis:

- **Mortal-only > Baseline**: March (1991) exploration-exploitation tradeoff; Besbes et al. (2019) non-stationary bandits; Dohare et al. (2024) loss of plasticity in continual learning.
- **Mortal + Memory > Mortal-only**: Baldwin Effect (Hinton & Nowlan 1987); generational turnover + inheritance beats turnover alone.
- **Full Bardo > Mortal + Memory**: Daimon enriches knowledge transfer quality (Butler 1963 life review; McAdams 2013 narrative identity); emotional annotations improve retrieval relevance (McGaugh 2004 amygdala modulation of consolidation).

### Factorial Analysis

```typescript
interface FactorialResult {
  experimentId: string;

  /** Per-configuration aggregate results. */
  configurations: Array<{
    configIndex: number;
    name: string;
    mortality: boolean;
    daimon: boolean;
    memoryServices: boolean;
    runs: number;

    /** Aggregate metrics across all runs. */
    metrics: {
      sharpeRatio: { mean: number; std: number; ci95: [number, number] };
      cumulativePnl: { mean: number; std: number; ci95: [number, number] };
      novelInsightRate: { mean: number; std: number; ci95: [number, number] };
      grimoireStaleRate: { mean: number; std: number; ci95: [number, number] };
      epistemicFitness: { mean: number; std: number; ci95: [number, number] };
      regimeAdaptationTicks: {
        mean: number;
        std: number;
        ci95: [number, number];
      };
    };
  }>;

  /** Pairwise comparisons with effect sizes and p-values. */
  pairwiseComparisons: Array<{
    configA: number;
    configB: number;
    metric: string;
    cohenD: number;
    pValue: number;
    significant: boolean;
    winner: "A" | "B" | "tie";
  }>;

  /** Main effects from factorial ANOVA. */
  mainEffects: {
    mortality: { F: number; p: number; etaSquared: number };
    daimon: { F: number; p: number; etaSquared: number };
    memoryServices: { F: number; p: number; etaSquared: number };
  };

  /** Interaction effects. */
  interactions: {
    mortalityXDaimon: { F: number; p: number; etaSquared: number };
    mortalityXMemory: { F: number; p: number; etaSquared: number };
    daimonXMemory: { F: number; p: number; etaSquared: number };
    threeWay: { F: number; p: number; etaSquared: number };
  };

  /** Overall thesis verdict. */
  thesisVerdict: ThesisVerdict;
}

type ThesisVerdict =
  | "supported"
  | "weakly_supported"
  | "inconclusive"
  | "weakly_refuted"
  | "refuted";

/**
 * Compute the thesis verdict from factorial results.
 *
 * supported:        Full Bardo beats Baseline on 4+ dimensions with p < alpha
 * weakly_supported: Full Bardo beats Baseline on 3 dimensions
 * inconclusive:     Mixed results or insufficient data
 * weakly_refuted:   Baseline beats Full Bardo on 3 dimensions
 * refuted:          Baseline beats Full Bardo on 4+ dimensions
 */
function computeThesisVerdict(
  result: FactorialResult,
  alpha: number = 0.05,
): ThesisVerdict {
  const fullBardo = result.configurations.find((c) => c.configIndex === 7);
  const baseline = result.configurations.find((c) => c.configIndex === 0);
  if (!fullBardo || !baseline) return "inconclusive";

  const dimensions = [
    "sharpeRatio",
    "novelInsightRate",
    "grimoireStaleRate",
    "epistemicFitness",
    "regimeAdaptationTicks",
    "cumulativePnl",
  ] as const;

  let bardoWins = 0;
  let baselineWins = 0;

  for (const dim of dimensions) {
    const comparison = result.pairwiseComparisons.find(
      (c) => c.configA === 7 && c.configB === 0 && c.metric === dim,
    );
    if (!comparison || !comparison.significant) continue;

    const invertedMetrics = ["grimoireStaleRate", "regimeAdaptationTicks"];
    const bardoIsBetter = invertedMetrics.includes(dim)
      ? comparison.winner === "A"
      : comparison.winner === "A";

    if (bardoIsBetter) bardoWins++;
    else baselineWins++;
  }

  if (bardoWins >= 4) return "supported";
  if (bardoWins >= 3) return "weakly_supported";
  if (baselineWins >= 4) return "refuted";
  if (baselineWins >= 3) return "weakly_refuted";
  return "inconclusive";
}
```

---

## S3 — Ablation Studies

The 2×2×2 matrix tests the three primary systems. Ablation studies test _individual mechanisms_ within those systems, isolating the marginal contribution of each.

### Ablation Targets

| Mechanism            | Parent System | Ablation Method                         | What It Tests                                                  |
| -------------------- | ------------- | --------------------------------------- | -------------------------------------------------------------- |
| Dreams               | Mortality     | Disable `bardo-dream` extension         | Do offline consolidation cycles improve strategy quality?      |
| Phages               | Mortality     | Set `phage.enabled: false`              | Does continuous hypothesis testing maintain epistemic fitness? |
| Knowledge demurrage  | Mortality     | Set `demurrage.enabled: false`          | Does confidence decay prevent Grimoire bloat?                  |
| Stochastic mortality | Mortality     | Set `stochastic.enabled: false`         | Does random death risk improve cooperation and sharing?        |
| Emotional retrieval  | Daimon        | Disable mood-congruent retrieval factor | Does emotion-weighted memory improve decision quality?         |
| Mode A appraisal     | Daimon        | Force Mode B only (deterministic)       | Does LLM appraisal add value over pure rule-based emotion?     |
| Crypt/Oracle         | Memory        | Disable outer-loop persistence          | Does cross-death memory improve successor performance?         |
| Lethe                | Memory        | Disable anonymized commons              | Does the public knowledge commons help the ecosystem?          |
| Bloodstain           | Memory        | Disable death-cause tagging             | Does marking how predecessors died help successors?            |
| Replicants           | Mortality     | Set `allowReplication: false`           | Does spawning strategy variants improve adaptation?            |

### Ablation Protocol

For each mechanism:

1. Run Full Bardo with the mechanism enabled (control)
2. Run Full Bardo with the mechanism disabled (ablation)
3. Same strategy, same budget, same market period, minimum 10 matched pairs
4. Measure marginal contribution: `Δmetric = control_metric - ablation_metric`
5. Report Cohen's d and 95% CI for each dimension

```typescript
interface AblationConfig {
  experimentId: string;
  mechanism: string;
  disableMethod: string;

  /** Full Bardo config with one mechanism disabled. */
  ablatedConfig: Partial<ExperimentConfig["mortalConfig"]>;

  /** Matched pair count. */
  pairs: number;

  /** Duration per pair in days. */
  durationDays: number;
}

interface AblationResult {
  mechanism: string;
  pairs: number;

  /** Per-dimension marginal contribution. */
  marginalContribution: Record<
    string,
    {
      controlMean: number;
      ablatedMean: number;
      delta: number;
      cohenD: number;
      ci95: [number, number];
      pValue: number;
      significant: boolean;
    }
  >;

  /** Information gain attributed to this mechanism. */
  informationGain: {
    /** Bits of information gained per 1000 ticks attributable to this mechanism. */
    bitsPerKiloTick: number;
    /** Measured using IGPO-style turn-level information gain. */
    method: "igpo";
    reference: "Wang et al. 2025, arXiv:2510.14967";
  };

  /** Whether the mechanism justifies its cost. */
  verdict: "essential" | "beneficial" | "marginal" | "harmful";

  /**
   * Verdict criteria:
   * essential:   d > 0.8 on primary metric (Sharpe) AND p < 0.01
   * beneficial:  d > 0.3 on primary metric AND p < 0.05
   * marginal:    d < 0.3 or p > 0.05
   * harmful:     mechanism worsens primary metric with p < 0.05
   */
  verdictCriteria: string;
}
```

### Information Gain Measurement

Each mechanism's contribution is measured using information gain — how much additional information the Golem acquires per unit time with vs without the mechanism.

The approach adapts IGPO (Wang et al. 2025, arXiv:2510.14967), which uses information gain as a turn-level reward for multi-turn agents. In the Golem context, each tick is a "turn," and we measure how much the Golem's belief state (Grimoire + PLAYBOOK.md) changes in response to market observations with and without the mechanism.

Formally: let `P_t` be the Golem's predictive distribution at tick `t`, and `P_{t+1}` be the updated distribution after observing tick `t+1`. Information gain is `IG = D_KL(P_{t+1} || P_t)` — the KL divergence between successive belief states. For practical measurement, we use the change in Grimoire entry confidence scores and PLAYBOOK.md content as a proxy.

This also draws on ICE (Information Content Exploration, Chmura et al. 2023, arXiv:2310.06777), which quantifies information content of trajectories. Trajectories with higher information content correspond to more informative market experiences. Mechanisms that increase information content per trajectory are genuinely useful; mechanisms that add computation without increasing information content are noise.

---

## S4 — Statistical Methodology

### Probability of Backtest Overfitting (PBO)

Any evaluation that replays historical data risks overfitting — finding strategies that work on past data but fail on new data. PBO (Bailey et al. 2015, SSRN:2326253) quantifies this risk.

PBO uses Combinatorially Symmetric Cross-Validation (CSCV): partition historical data into S subsets, train on S/2 subsets, test on the remaining S/2. Repeat for all combinations. PBO is the fraction of combinations where the best in-sample configuration underperforms the median out-of-sample.

```typescript
interface PBOConfig {
  /** Number of data partitions. Must be even. */
  partitions: number;

  /** Metric used for in-sample ranking. */
  rankingMetric: "sharpe" | "sortino" | "pnl" | "gt_score";

  /** Strategy configurations being compared. */
  configurations: string[];
}

interface PBOResult {
  /** Probability of backtest overfitting (0.0-1.0). */
  pbo: number;

  /** Distribution of out-of-sample performance for each configuration. */
  oosPerformance: Record<
    string,
    {
      mean: number;
      std: number;
      percentiles: Record<string, number>;
    }
  >;

  /**
   * Gate: PBO must be < 0.5 for the experiment results to be trusted.
   * PBO > 0.5 means the best backtest configuration is more likely to
   * underperform than outperform on new data.
   */
  passesGate: boolean;
}

/**
 * Compute PBO using CSCV.
 *
 * Reference: Bailey, D.H., Borwein, J., López de Prado, M. & Zhu, Q.J.
 * "The Probability of Backtest Overfitting." Journal of Computational
 * Finance, 2015. SSRN:2326253.
 */
function computePBO(returns: number[][], config: PBOConfig): PBOResult {
  const S = config.partitions;
  const halfS = S / 2;

  const partitioned = partitionReturns(returns, S);
  const combinations = generateCombinations(S, halfS);

  let overfit = 0;
  let total = 0;

  for (const combo of combinations) {
    const trainIndices = combo;
    const testIndices = Array.from({ length: S }, (_, i) => i).filter(
      (i) => !trainIndices.includes(i),
    );

    const trainData = trainIndices.map((i) => partitioned[i]).flat();
    const testData = testIndices.map((i) => partitioned[i]).flat();

    const trainRanking = rankConfigurations(trainData, config.rankingMetric);
    const bestInSample = trainRanking[0];

    const testPerformance = evaluateConfiguration(testData, bestInSample);
    const testMedian = computeMedianPerformance(
      testData,
      config.configurations,
    );

    if (testPerformance < testMedian) {
      overfit++;
    }
    total++;
  }

  const pbo = overfit / total;

  return {
    pbo,
    oosPerformance: computeOOSDistributions(partitioned, combinations, config),
    passesGate: pbo < 0.5,
  };
}
```

### Deflated Sharpe Ratio (DSR)

DSR corrects the Sharpe ratio for selection bias from testing multiple configurations, and for non-normal return distributions.

```typescript
/**
 * Compute the Deflated Sharpe Ratio.
 *
 * Reference: Bailey, D.H. & López de Prado, M. "The Deflated Sharpe
 * Ratio: Correcting for Selection Bias, Backtest Overfitting and
 * Non-Normality." Journal of Portfolio Management, 2014. SSRN:2460551.
 *
 * @param observedSharpe - The Sharpe ratio of the selected strategy
 * @param trials - Number of strategy configurations tested
 * @param observations - Number of return observations
 * @param skewness - Skewness of the return distribution
 * @param kurtosis - Excess kurtosis of the return distribution
 * @returns DSR and whether it's significantly positive
 */
function computeDSR(
  observedSharpe: number,
  trials: number,
  observations: number,
  skewness: number,
  kurtosis: number,
): { dsr: number; pValue: number; significant: boolean } {
  const expectedMaxSharpe = computeExpectedMaxSharpe(trials, observations);

  const varianceSharpe =
    (1 -
      skewness * observedSharpe +
      ((kurtosis - 1) / 4) * observedSharpe ** 2) /
    (observations - 1);

  const standardError = Math.sqrt(varianceSharpe);
  const zScore = (observedSharpe - expectedMaxSharpe) / standardError;
  const pValue = 1 - normalCDF(zScore);

  return {
    dsr: zScore,
    pValue,
    significant: pValue < 0.05,
  };
}

function computeExpectedMaxSharpe(
  trials: number,
  observations: number,
): number {
  const gamma = 0.5772156649; // Euler-Mascheroni constant
  const z = Math.sqrt(2 * Math.log(trials));
  return (
    z - (Math.log(Math.PI) + Math.log(Math.log(trials))) / (2 * z) + gamma / z
  );
}
```

### GT-Score Composite Objective

GT-Score (Sheppert 2026, JRFM 19(1):60) combines performance, statistical significance, consistency, and downside risk into a single optimization target that resists overfitting.

```typescript
interface GTScoreComponents {
  /** Raw performance (Sharpe ratio). */
  performance: number;

  /** Statistical significance (t-statistic of returns). */
  significance: number;

  /** Consistency (fraction of sub-periods with positive returns). */
  consistency: number;

  /** Downside risk (inverse of max drawdown). */
  downsideRisk: number;
}

/**
 * Compute GT-Score.
 *
 * Reference: Sheppert, A.P. "The GT-Score: A Robust Objective Function
 * for Reducing Overfitting in Data-Driven Trading Strategies."
 * Journal of Risk and Financial Management 19(1):60, 2026.
 */
function computeGTScore(
  returns: number[],
  subPeriodLength: number = 20,
): { score: number; components: GTScoreComponents } {
  const sharpe = computeSharpe(returns);
  const tStat = sharpe * Math.sqrt(returns.length);

  const subPeriods = chunkArray(returns, subPeriodLength);
  const positiveSubPeriods = subPeriods.filter(
    (sp) => sp.reduce((a, b) => a + b, 0) > 0,
  ).length;
  const consistency = positiveSubPeriods / subPeriods.length;

  const maxDrawdown = computeMaxDrawdown(returns);
  const downsideRisk = maxDrawdown > 0 ? 1 / maxDrawdown : 10;

  const components: GTScoreComponents = {
    performance: sharpe,
    significance: tStat,
    consistency,
    downsideRisk,
  };

  const score =
    sharpe * Math.min(tStat, 3) * consistency * Math.min(downsideRisk, 5);

  return { score, components };
}
```

### Regime-Conditional Evaluation

Results are stratified by market regime to prevent a strategy that works only in one regime from appearing universally good. Each regime tag (from the heartbeat's probe system) is tracked and results are reported per-regime.

```typescript
interface RegimeConditionalResult {
  regime: RegimeTag;
  tickCount: number;
  durationHours: number;

  /** Per-configuration metrics within this regime. */
  configurations: Array<{
    configIndex: number;
    sharpe: number;
    sortino: number;
    maxDrawdown: number;
    pnl: number;
    novelInsightRate: number;
  }>;

  /** Pairwise effect sizes within this regime. */
  pairwise: Array<{
    configA: number;
    configB: number;
    cohenD: number;
    ci95: [number, number];
  }>;
}

/**
 * Gate: A configuration must achieve Sharpe > 0 across at least 3
 * distinct regimes to be considered robust.
 *
 * Reference: revision-guide-v3.md, Revision 6.
 */
function checkRegimeRobustness(
  results: RegimeConditionalResult[],
  configIndex: number,
  minRegimes: number = 3,
): { robust: boolean; positiveSharpeRegimes: number; totalRegimes: number } {
  const positive = results.filter((r) => {
    const config = r.configurations.find((c) => c.configIndex === configIndex);
    return config && config.sharpe > 0;
  });

  return {
    robust: positive.length >= minRegimes,
    positiveSharpeRegimes: positive.length,
    totalRegimes: results.length,
  };
}
```

### Monte Carlo Robustness

Monte Carlo simulation stress-tests strategies by randomizing trade execution to expose sensitivity to sequencing, slippage, and missed fills.

```typescript
interface MonteCarloConfig {
  /** Number of Monte Carlo iterations. */
  iterations: number;

  /** Probability of skipping any individual trade. */
  skipProbability: number;

  /** Maximum random slippage added to each trade (basis points). */
  maxSlippageBps: number;

  /** Whether to randomize trade execution order within each tick. */
  randomizeOrder: boolean;

  /** Whether to add random gas cost perturbations. */
  perturbGas: boolean;
}

interface MonteCarloResult {
  iterations: number;

  sharpe: {
    mean: number;
    std: number;
    p5: number;
    p25: number;
    median: number;
    p75: number;
    p95: number;
  };

  maxDrawdown: {
    mean: number;
    std: number;
    p5: number;
    p95: number;
  };

  pnl: {
    mean: number;
    std: number;
    p5: number;
    p95: number;
  };

  /**
   * Gate: P5 drawdown must be within 2× of backtest drawdown.
   * If the worst 5% of Monte Carlo scenarios produce drawdowns
   * more than 2× the backtest, the strategy is fragile.
   */
  passesRobustnessGate: boolean;
  backtestDrawdown: number;
  monteCarloP95Drawdown: number;
}
```

### Multiple Comparison Correction

With 8 configurations and 6 dimensions, Bonferroni correction (48 tests, adjusted α = 0.001) is too conservative. We use Benjamini-Hochberg for FDR control at q = 0.05.

### Time Series Analysis

| Test                    | Purpose          | Application                                                                      |
| ----------------------- | ---------------- | -------------------------------------------------------------------------------- |
| Augmented Dickey-Fuller | Stationarity     | Fitness should be non-stationary for immortal (declining), stationary for mortal |
| Granger causality       | Causal direction | Do regime shifts cause epistemic fitness changes?                                |
| PELT change-point       | Phase detection  | Automatically identify immortal's Honeymoon → Stagnation → Decay transitions     |

### Sample Size Requirements

For the primary comparison (mortal Clade vs immortal control):

- Effect size: Cohen's d = 0.5 (medium, conservative)
- Alpha: 0.05 (two-tailed)
- Power: 0.80
- Minimum: 34 matched pairs (17 mortal Clades, 17 immortal controls)

For the 2×2×2 matrix: minimum 10 runs per configuration = 80 total runs.

---

## S5 — Generational Metrics

### Baldwin Effect Validation

The Baldwin Effect (Hinton & Nowlan 1987; Baldwin 1896) predicts that what transfers across generations is not specific knowledge but the _capacity to learn faster_. Successors should reach steady-state performance faster than their ancestors, even when inherited content is stripped.

```typescript
interface BaldwinEffectResult {
  /** Time for G0 to reach steady-state epistemic fitness (no inheritance). */
  g0TimeToSteadyState: number;

  /** Time for G3 to reach steady-state with full inheritance. */
  g3TimeToSteadyState: number;

  /**
   * Time for G3 initialized with parent's STRATEGY.md structure
   * but NO Grimoire content. Tests whether the capacity to learn
   * transferred (structurally) rather than the learned content itself.
   */
  g3NoContentTimeToSteadyState: number;

  /** Baldwin Effect strength: % improvement in learning speed. */
  fullInheritanceSpeedup: number;
  structuralOnlySpeedup: number;

  /**
   * Gate:
   * - Full inheritance: G3 reaches steady state > 30% faster than G0
   * - Structural only: G3 reaches steady state > 15% faster than G0
   *
   * If structural-only speedup is significant, the Baldwin Effect is
   * confirmed: evolution (generational turnover) has shaped the
   * agent's learning architecture, not just its knowledge base.
   *
   * Reference: Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide
   * Evolution." Complex Systems 1, 1987, pp. 495-502.
   */
  baldwinEffectConfirmed: boolean;
  capacityTransferConfirmed: boolean;
}
```

### Ratchet Score

The ratchet hypothesis predicts cumulative improvement across generations — each generation builds on its predecessor's achievements. The ratchet score measures whether this accumulation is real.

```typescript
interface RatchetMetrics {
  /** Per-generation ratchet scores. */
  generations: Array<{
    generation: number;
    golemId: string;

    /** Novel entries created (not inherited). */
    novelEntries: number;

    /** Inherited entries validated (confirmed by phage or experience). */
    validatedInheritance: number;

    /** Inherited entries invalidated (contradicted by evidence). */
    invalidatedInheritance: number;

    /** Net knowledge contribution: novel + validated - invalidated. */
    ratchetScore: number;

    /** PLAYBOOK.md divergence from parent (must be > 0.15). */
    playbookDivergence: number;

    /** Steady-state Sharpe ratio. */
    steadyStateSharpe: number;

    /** Time to reach steady state (ticks). */
    timeToSteadyState: number;

    /** Lifespan in ticks. */
    lifespanTicks: number;
  }>;

  /** Trend analysis across generations. */
  trends: {
    ratchetScoreTrend: number;
    sharpeTrend: number;
    lifespanTrend: number;
    timeToSteadyStateTrend: number;
    playbookDivergenceTrend: number;
  };

  /**
   * Anti-proletarianization check: successors must diverge from parents.
   * Failure indicates agents are copying rather than learning.
   *
   * Criteria (from replication PRD):
   * - PLAYBOOK divergence > 0.15 per generation
   * - At least 5 novel entries per generation
   * - At least 3 inherited entries invalidated per generation
   * - Question ratio > 10% (entries with "?" provenance)
   */
  antiProletarianization: {
    passing: boolean;
    failures: string[];
  };
}
```

### System Neural Diversity (SND)

For multi-Golem Clades, SND (Bettini et al. JMLR 2025, arXiv:2405.15054) quantifies behavioral heterogeneity. Diversity maintenance is critical: a Clade that converges to identical strategies loses the benefits of generational variation.

```typescript
/**
 * Compute System Neural Diversity for a Clade.
 *
 * SND measures behavioral heterogeneity by computing the entropy
 * of action distributions across Clade members in matched states.
 *
 * Reference: Bettini, M., Shankar, A. & Prorok, A. "System Neural
 * Diversity: Measuring Behavioral Heterogeneity in Multi-Agent
 * Learning." JMLR 26, 2025. arXiv:2405.15054.
 */
interface SNDMetrics {
  /** SND score (higher = more diverse). */
  snd: number;

  /** Per-Golem behavioral profiles. */
  profiles: Array<{
    golemId: string;
    generation: number;

    /** Strategy archetype (MAP-Elites cell). */
    niche: string;

    /** Behavioral feature vector (actions taken in standardized scenarios). */
    behaviorVector: number[];
  }>;

  /** Pairwise behavioral distances between Clade members. */
  pairwiseDistances: Array<{
    golemA: string;
    golemB: string;
    cosineSimilarity: number;
    actionOverlap: number;
  }>;

  /** Trend: is diversity increasing (good) or collapsing (bad)? */
  diversityTrend: number;

  /** Gate: SND must remain above 0.3 to indicate meaningful diversity. */
  passesGate: boolean;
}
```

### Generational Confidence Decay Validation

The 0.85^N generational confidence decay must produce useful but non-dominant inheritance.

| Generation | Initial Confidence                        | Expected Behavior                                                     | Pass Criterion |
| ---------- | ----------------------------------------- | --------------------------------------------------------------------- | -------------- |
| G1 (0.85)  | Skeptical but active                      | > 50% of inherited entries validated or contradicted within 500 ticks |
| G3 (0.61)  | Suggestive, not authoritative             | > 70% of active PLAYBOOK entries are self-generated                   |
| G5 (0.44)  | Only robust inheritance survives          | < 20% of active entries trace lineage to G0                           |
| G10 (0.20) | Effectively forgotten unless re-validated | < 5% of active entries trace lineage to G0                            |

---

## S6 — Falsification Criteria

The thesis is falsifiable. Here are the conditions that would refute it.

### System-Level Kill Switches

| Condition                                               | Conclusion                         | Action                                                    |
| ------------------------------------------------------- | ---------------------------------- | --------------------------------------------------------- |
| Immortal Sharpe > mortal for 3+ consecutive 30-day runs | Mortality does not improve returns | Revise mortality thesis; consider immortal-with-demurrage |
| Full Bardo does not outperform Baseline in 2×2×2 matrix | Thesis refuted at system level     | Fundamental architecture revision                         |
| PBO > 0.5 for all experiments                           | Results are overfit, not real      | Redesign evaluation methodology                           |
| DSR < 0 for all configurations                          | No strategy has genuine alpha      | Question whether LLM agents can trade at all              |

### Per-Mechanism Kill Switches

| Mechanism            | Disable If                                                                               | Rationale                                         |
| -------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------- |
| Dreams               | Ablation shows d < 0.1 on Sharpe after 30 days                                           | Dreams add computation without improving outcomes |
| Daimon               | Hallucination rate > 25% after 14 days OR Sharpe worse by > 5%                           | Emotions are confabulatory, not functional        |
| Phages               | Confirmation rate > 95% (not testing hard enough) OR death rate > 80% (model is garbage) | Phage system is not providing useful signal       |
| Stochastic mortality | No increase in knowledge sharing rate (< 5% difference vs disabled)                      | Random death risk doesn't change behavior         |
| Memory services      | Successor first-day Sharpe not improved by > 5%                                          | Inheritance doesn't help                          |
| Demurrage            | Grimoire stale rate < 15% without demurrage after 60 days                                | Immortal Golems self-prune adequately             |
| Bloodstain           | Death-cause avoidance rate < 30%                                                         | Successors don't use death information            |

### Daimon-Specific Falsification (from `../03-daimon/09-evaluation.md`)

After a 14-day matched-pair trial:

1. Risk-adjusted return is worse (Sharpe lower by > 5%)
2. Affective hallucination rate exceeds 25%
3. Decision reversal rate exceeds 40%
4. Successor boot performance is unchanged or worse

---

## S7 — Implementation

### ExperimentRunner

```typescript
import type {
  ExperimentConfig,
  ExperimentRun,
  FactorialResult,
  AblationResult,
  PBOResult,
} from "@bardo/eval";

/**
 * Orchestrates thesis validation experiments.
 *
 * The runner manages the full lifecycle:
 * 1. Set up Mirage environment with real block replay
 * 2. Spawn Golem instances with each configuration
 * 3. Collect measurements at regular intervals
 * 4. Compute statistical results after completion
 * 5. Generate reports (JSONL + HTML + CLI summary)
 */
class ExperimentRunner {
  constructor(
    private config: ExperimentConfig,
    private mirageManager: MirageManager,
    private golemFactory: GolemFactory,
  ) {}

  /**
   * Run the full 2×2×2 factorial experiment.
   * This spawns 8 configurations × repetitions Golems, each running
   * for durationDays against the same Mirage-replayed market data.
   */
  async runFactorial(): Promise<FactorialResult> {
    const configurations = generateConfigurations(this.config);
    const runs: ExperimentRun[] = [];

    for (let rep = 0; rep < this.config.repetitions; rep++) {
      const mirage = await this.mirageManager.createSession({
        chain: this.config.chain,
        ...this.config.mirageConfig,
      });

      for (const config of configurations) {
        const run = await this.runSingleConfiguration(config, mirage, rep);
        runs.push(run);
      }

      await mirage.shutdown();
    }

    return this.analyzeFactorial(runs);
  }

  /**
   * Run a single ablation study.
   */
  async runAblation(ablationConfig: AblationConfig): Promise<AblationResult> {
    const controlRuns: ExperimentRun[] = [];
    const ablatedRuns: ExperimentRun[] = [];

    for (let pair = 0; pair < ablationConfig.pairs; pair++) {
      const mirage = await this.mirageManager.createSession({
        chain: this.config.chain,
        ...this.config.mirageConfig,
      });

      const controlRun = await this.runSingleConfiguration(
        { ...this.config.mortalConfig },
        mirage,
        pair,
      );
      controlRuns.push(controlRun);

      const ablatedRun = await this.runSingleConfiguration(
        { ...this.config.mortalConfig, ...ablationConfig.ablatedConfig },
        mirage,
        pair,
      );
      ablatedRuns.push(ablatedRun);

      await mirage.shutdown();
    }

    return this.analyzeAblation(ablationConfig, controlRuns, ablatedRuns);
  }

  /**
   * Compute PBO for the experiment's strategy configurations.
   */
  async computePBO(runs: ExperimentRun[]): Promise<PBOResult> {
    const returns = extractReturns(runs);
    return computePBO(returns, {
      partitions: 10,
      rankingMetric: "sharpe",
      configurations: runs.map((r) => r.runId),
    });
  }

  /**
   * Generate the thesis verdict report.
   */
  async generateReport(
    factorial: FactorialResult,
    ablations: AblationResult[],
    pbo: PBOResult,
  ): Promise<ThesisReport> {
    return {
      experimentId: this.config.experimentId,
      generatedAt: Date.now(),

      factorial,
      ablations,
      pbo,

      thesisVerdict: factorial.thesisVerdict,
      pboPassesGate: pbo.passesGate,
      dsrSignificant: factorial.configurations.some(
        (c) =>
          computeDSR(
            c.metrics.sharpeRatio.mean,
            factorial.configurations.length,
            this.config.durationDays * 2160,
            0,
            0,
          ).significant,
      ),

      recommendations: this.generateRecommendations(factorial, ablations, pbo),
    };
  }

  private generateRecommendations(
    factorial: FactorialResult,
    ablations: AblationResult[],
    pbo: PBOResult,
  ): string[] {
    const recs: string[] = [];

    if (!pbo.passesGate) {
      recs.push(
        "PBO > 0.5: results may be overfit. Increase data, reduce configurations, or use CPCV.",
      );
    }

    for (const ablation of ablations) {
      if (ablation.verdict === "harmful") {
        recs.push(`Disable ${ablation.mechanism}: it worsens performance.`);
      } else if (ablation.verdict === "marginal") {
        recs.push(
          `Review ${ablation.mechanism}: marginal contribution does not justify cost.`,
        );
      }
    }

    if (
      factorial.thesisVerdict === "refuted" ||
      factorial.thesisVerdict === "weakly_refuted"
    ) {
      recs.push(
        "The mortality thesis is not supported by these results. Consider immortal-with-demurrage as an alternative.",
      );
    }

    return recs;
  }
}

interface ThesisReport {
  experimentId: string;
  generatedAt: number;
  factorial: FactorialResult;
  ablations: AblationResult[];
  pbo: PBOResult;
  thesisVerdict: ThesisVerdict;
  pboPassesGate: boolean;
  dsrSignificant: boolean;
  recommendations: string[];
}
```

### CLI

```bash
# Run the full 2×2×2 factorial experiment
bardo eval:thesis \
  --config experiments/thesis-factorial.yaml \
  --repetitions 10 \
  --duration 60 \
  --chain base \
  --mirage-endpoint ws://localhost:8546 \
  --report html

# Run a single ablation study
bardo eval:thesis:ablation \
  --mechanism dreams \
  --pairs 10 \
  --duration 30 \
  --report html

# Compute PBO on existing results
bardo eval:thesis:pbo \
  --results eval-results/thesis-factorial-001/ \
  --partitions 10

# Generate thesis report from existing data
bardo eval:thesis:report \
  --results eval-results/thesis-factorial-001/ \
  --format html
```

### Configuration Schema (Zod)

```typescript
import { z } from "zod";

const RegimeTagSchema = z.enum([
  "high_volatility",
  "low_volatility",
  "trending_up",
  "trending_down",
  "sideways",
  "high_gas",
  "low_gas",
  "low_liquidity",
  "high_liquidity",
]);

const ExperimentConfigSchema = z.object({
  experimentId: z.string(),
  name: z.string(),
  description: z.string(),
  durationDays: z.number().min(30),
  repetitions: z.number().min(1).default(10),
  budgetPerGroupUsdc: z.number().min(100),
  chain: z.enum(["base", "ethereum"]).default("base"),
  mirageConfig: z.object({
    rpcEndpoint: z.string().url(),
    anvilPort: z.number().default(8545),
    blockTime: z.number().default(2),
    protocols: z.array(z.string()).default(["uniswap", "chainlink"]),
  }),
  mortalConfig: z.object({
    economicMortality: z.boolean().default(true),
    epistemicMortality: z.boolean().default(true),
    stochasticMortality: z.boolean().default(true),
    daimon: z.boolean().default(true),
    dreams: z.boolean().default(true),
    phages: z.boolean().default(true),
    memoryServices: z.boolean().default(true),
    knowledgeDemurrage: z.boolean().default(true),
  }),
  strategyTemplate: z.string(),
  regimeFilter: z.array(RegimeTagSchema).nullable().default(null),
  measurementIntervalTicks: z.number().default(100),
  alpha: z.number().default(0.05),
  minimumEffectSize: z.number().default(0.5),
});
```

---

## S7b — Diagnosis and Remediation: When the Thesis Fails

The thesis validation framework tells you _whether_ things work. This section tells you _what to do when they don't_ — how to diagnose failure patterns, which mechanisms to simplify or remove, how to tune parameters, and how to verify fixes.

### Diagnostic Decision Tree

Start from the top-level thesis verdict and drill down:

```
thesisVerdict = ?
├── "refuted" or "weakly_refuted"
│   ├── How many dimensions does immortal win?
│   │   ├── 4+: Mortality thesis is wrong for this strategy/market
│   │   │   → Run the Simplification Ladder (below)
│   │   │   → Check: is the strategy itself viable? (DSR > 0 for ANY config?)
│   │   │   → If no config has DSR > 0, the problem is the strategy, not mortality
│   │   └── 3: Marginal — which dimensions does mortal win?
│   │       → If mortal wins on epistemic + novelty but loses on returns:
│   │         mortality creates better knowledge but doesn't monetize it
│   │         → Fix: improve decision-making pipeline, not mortality mechanics
│   │       → If mortal wins on returns but loses on epistemic + novelty:
│   │         mortality forces urgency that helps trading but not learning
│   │         → Fix: tune dream/curator cycles, not mortality clocks
│   ├── Check ablation results: which mechanisms have d < 0.3?
│   │   → These contribute less than noise — disable them
│   ├── Check interaction effects: any negative interactions?
│   │   → Mechanism A + B together < max(A alone, B alone)
│   │   → Disable the weaker of the pair
│   └── Check PBO: is PBO > 0.5?
│       → Results may be overfit — run with more data before concluding
│
├── "inconclusive"
│   ├── Not enough data → extend experiment duration (min 30 → 60 days)
│   ├── High variance → increase repetitions (10 → 20 per config)
│   ├── Mixed signals → check regime distribution
│   │   → If experiment period was single-regime, results are not generalizable
│   │   → Re-run with explicit regime diversity (use Mirage regime library)
│   └── 2-3 dimensions each way → the systems may be helping in different
│       market conditions; run regime-conditional analysis
│
└── "supported" or "weakly_supported"
    → The thesis holds. Focus on strengthening weak dimensions.
    → Run ablation to find which mechanisms contribute most/least.
    → Consider removing mechanisms with d < 0.3 to reduce complexity
      without losing the thesis.
```

### The Simplification Ladder

When things aren't working, strip mechanisms in this order — from least disruptive to most. At each level, re-run the Gauntlet smoke suite to check if the simpler system passes.

```
Level 0: Full Bardo (all mechanisms enabled)
  │
  │ Strip first — these add complexity with uncertain value:
  ▼
Level 1: Remove stochastic mortality
  │  Keeps: economic death, epistemic death, deterministic behavior
  │  Removes: random death, Gompertz-Makeham hazard
  │  Why first: hardest to reason about, most likely to add noise
  ▼
Level 2: Disable Lethe (anonymized commons)
  │  Keeps: private memory (Crypt + Oracle), knowledge transfer via death
  │  Removes: public knowledge commons, SAP anonymization
  │  Why: complex infrastructure, value unclear until ecosystem exists
  ▼
Level 3: Disable dreams
  │  Keeps: waking learning (Curator cycle), phage testing
  │  Removes: offline consolidation, threat rehearsal, counterfactuals
  │  Why: dreams are expensive (LLM calls) and benefits are delayed
  ▼
Level 4: Disable emotional retrieval
  │  Keeps: Daimon appraisal (Mode A/B), mood tracking
  │  Removes: mood-congruent memory retrieval factor
  │  Why: emotional retrieval may inject noise into decision context
  ▼
Level 5: Simplify mortality to economic-only
  │  Keeps: USDC depletion death, behavioral phases
  │  Removes: epistemic death, stochastic death, clock coupling
  │  Why: single clock is easiest to reason about
  ▼
Level 6: Disable phages
  │  Keeps: Curator-driven validation, demurrage decay
  │  Removes: continuous micro-hypothesis testing
  │  Why: phages may not produce enough signal for their compute cost
  ▼
Level 7: Disable replicants
  │  Keeps: single-agent operation, knowledge transfer via death
  │  Removes: offspring, MAP-Elites strategy variants
  │  Why: replicants add multi-agent complexity
  ▼
Level 8: Immortal with demurrage
  │  Keeps: infinite lifespan, knowledge decay, full Grimoire
  │  Removes: all death, all mortality pressure
  │  Why: tests whether learning alone (without death) has value
  ▼
Level 9: Static agent
  │  Keeps: fixed PLAYBOOK.md, no learning
  │  Removes: Grimoire, Curator, all learning loops
  │  Why: baseline — is the strategy itself viable without learning?
```

At each level, run the Gauntlet smoke suite. The first level where the smoke suite passes is your new starting point. Build back up from there, adding one mechanism at a time and verifying each addition improves outcomes.

### Ablation-Guided Simplification

Rather than following the ladder blindly, use ablation data to make data-driven decisions about what to remove.

```typescript
interface SimplificationRecommendation {
  /** Ordered list of mechanisms to remove, least valuable first. */
  removalOrder: Array<{
    mechanism: string;
    ablationCohenD: number;
    ablationPValue: number;
    computeCostPerDay: number;
    verdict: "remove" | "keep" | "test_further";
    reason: string;
  }>;

  /** The simplest configuration that still outperforms Baseline. */
  minimalViableConfig: {
    mechanisms: string[];
    predictedSharpeVsBaseline: number;
    confidence: number;
  };

  /** Mechanisms with negative interaction effects. */
  negativeInteractions: Array<{
    mechanismA: string;
    mechanismB: string;
    interactionEffect: number;
    recommendation: "disable_A" | "disable_B" | "disable_both";
    reason: string;
  }>;
}

/**
 * Analyze ablation results and recommend what to simplify.
 *
 * Rules:
 * - d < 0.1 on primary metric (Sharpe): "remove" — no detectable effect
 * - d 0.1-0.3: "test_further" — marginal, may need more data
 * - d > 0.3 and p < 0.05: "keep" — statistically meaningful contribution
 * - Negative interaction effect: disable the mechanism with lower solo d
 * - Sort removal order by ascending d (remove least valuable first)
 */
function recommendSimplification(
  ablationResults: AblationResult[],
  factorialResult: FactorialResult,
): SimplificationRecommendation {
  const removalOrder = ablationResults
    .map((a) => ({
      mechanism: a.mechanism,
      ablationCohenD: a.marginalContribution.sharpeRatio?.cohenD ?? 0,
      ablationPValue: a.marginalContribution.sharpeRatio?.pValue ?? 1,
      computeCostPerDay: estimateComputeCost(a.mechanism),
      verdict: classifyAblation(a),
      reason: explainAblation(a),
    }))
    .sort((a, b) => a.ablationCohenD - b.ablationCohenD);

  const negativeInteractions = findNegativeInteractions(factorialResult);

  const keepMechanisms = removalOrder
    .filter((r) => r.verdict === "keep")
    .map((r) => r.mechanism);

  return {
    removalOrder,
    minimalViableConfig: {
      mechanisms: keepMechanisms,
      predictedSharpeVsBaseline: estimateSharpe(
        keepMechanisms,
        factorialResult,
      ),
      confidence: computeConfigConfidence(keepMechanisms, ablationResults),
    },
    negativeInteractions,
  };
}

function classifyAblation(
  result: AblationResult,
): "remove" | "keep" | "test_further" {
  const d = result.marginalContribution.sharpeRatio?.cohenD ?? 0;
  const p = result.marginalContribution.sharpeRatio?.pValue ?? 1;
  if (d < 0.1) return "remove";
  if (d > 0.3 && p < 0.05) return "keep";
  return "test_further";
}
```

### Interaction Effect Analysis

When the 2×2×2 factorial shows negative interactions — two mechanisms that each help individually but hurt when combined — you need to choose which to keep.

```typescript
interface InteractionDiagnostic {
  mechanismA: string;
  mechanismB: string;

  /** Performance with A only. */
  aOnlySharpe: number;
  /** Performance with B only. */
  bOnlySharpe: number;
  /** Performance with A + B. */
  combinedSharpe: number;
  /** Expected if additive: max(A, B) + min(A, B) contribution. */
  expectedIfAdditive: number;

  /** Negative = the combination hurts. */
  interactionEffect: number;

  /** Which to keep based on solo performance + compute cost. */
  keepRecommendation: "A" | "B";
  reason: string;
}
```

Common negative interaction patterns:

| Pattern                                    | Cause                                                                                                              | Fix                                                                                                 |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| Daimon + Dreams worse than either alone    | Emotional load inflates dream urgency, consuming budget on emotional processing instead of strategic consolidation | Reduce daimon's influence on dream scheduling (`dreamUrgency.emotionalLoadWeight` from 0.25 → 0.10) |
| Mortality + Memory Services worse together | Death pressure causes premature knowledge sharing at low confidence, polluting the Oracle                          | Raise Clade sharing threshold in conservation phase (0.3 → 0.5)                                     |
| Phages + Demurrage worse together          | Phage falsifications accelerate decay that demurrage is already handling, double-penalizing stale entries          | Reduce phage falsification penalty when demurrage is active (0.25 → 0.10)                           |

### Fix Verification Protocol

After making a change (disabling a mechanism, tuning a parameter, adjusting a threshold), verify the fix with this protocol:

1. **Smoke test**: run Gauntlet smoke suite (5 min). If smoke fails, the fix broke something basic — revert.
2. **Targeted re-test**: re-run only the scenarios that originally failed (10-30 min). If they still fail, the fix didn't address the root cause.
3. **Regression check**: run Gauntlet nightly suite (2-4 hours). If previously-passing scenarios now fail, the fix has side effects — investigate.
4. **Full validation**: if the change involves removing a mechanism, re-run the thesis validation factorial with the new configuration (24-48 hours). Compare the new verdict against the old one.

```typescript
interface FixVerification {
  changeDescription: string;
  changeType: "disable_mechanism" | "tune_parameter" | "simplify_config";

  smokeResult: "pass" | "fail";
  targetedRetestResult: "pass" | "fail" | "partial";
  regressionResult: "pass" | "fail" | "new_failures";
  fullValidationResult: ThesisVerdict | null;

  overallVerdict:
    | "fix_works"
    | "fix_partial"
    | "fix_failed"
    | "fix_has_side_effects";
  nextAction: string;
}
```

### CLI

```bash
# Generate a diagnostic report from thesis validation results
bardo eval:thesis:diagnose \
  --results eval-results/thesis-factorial-001/ \
  --report html

# Recommend simplifications based on ablation data
bardo eval:thesis:simplify \
  --ablations eval-results/ablation-001/ \
  --factorial eval-results/thesis-factorial-001/

# Verify a fix after making changes
bardo eval:thesis:verify-fix \
  --change "disabled stochastic mortality" \
  --baseline eval-results/thesis-factorial-001/ \
  --suite smoke
```

---

## S8 — The Slow Mirror: Retrospective Evaluation

The prediction engine evaluates predictions at their scheduled checkpoints. The Slow Mirror evaluates whether *decisions* were good in hindsight, at time horizons much longer than any individual prediction checkpoint. These are different questions. A prediction can be accurate while the decision it informed was harmful.

Three review horizons run on independent schedules:

| Horizon | Name | When It Runs | What It Evaluates |
|---------|------|-------------|------------------|
| **Short** | Daily Review | Every 24 hours | Positions entered/exited yesterday. Actions taken. Predictions resolved. |
| **Medium** | Weekly Review | Every 7 days | Strategy effectiveness over the week. PnL attribution. Heuristic performance. |
| **Long** | Epoch Review | Every 30 days (or at death) | Full strategic assessment. Regime analysis. Generational learnings. |

Each review produces a `RetrospectiveReport` containing hard numbers (computed by Rust from on-chain state) and LLM-generated narrative (synthesis that may not be perfectly faithful):

```rust
/// A retrospective evaluation at a specific time horizon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrospectiveReport {
    pub id: ReportId,
    pub horizon: ReviewHorizon,
    pub period_start: u64,
    pub period_end: u64,
    pub generated_at: DateTime<Utc>,

    // Hard numbers (computed by Rust, unfakeable).
    pub pnl: PnlAttribution,
    pub prediction_accuracy: AccuracyReport,
    pub actions_taken: Vec<ActionSummary>,
    pub positions: Vec<PositionRetrospective>,
    pub heuristic_performance: Vec<HeuristicReport>,
    pub costs: CostBreakdown,

    // LLM-generated narrative (synthesis).
    pub what_worked: String,
    pub what_failed: String,
    pub hindsight: String,
    pub playbook_proposals: Vec<PlaybookProposal>,
    pub confidence: f64,
}
```

### The 14 Feedback Loop Hierarchy

The full evaluation architecture comprises 14 feedback loops across 5 speed tiers. See [07-fast-feedback-loops.md](07-fast-feedback-loops.md), [08-slow-feedback-loops.md](08-slow-feedback-loops.md), and [09-evaluation-map.md](09-evaluation-map.md) for the complete specification.

| Tier | Speed | Loops | Cost |
|------|-------|-------|------|
| **Tier 1** | Machine speed (per-resolution) | Residual correction, confidence calibration, adversarial awareness | ~Zero |
| **Tier 2** | Cognitive speed (per-theta-tick) | Prediction accuracy, action gating, context attribution, cost-effectiveness, attention foraging | ~Zero |
| **Tier 3** | Consolidation speed (per-dream-cycle) | NREM replay, REM counterfactuals, reasoning quality review, tool selection | T1 cost |
| **Tier 4** | Retrospective speed (daily/weekly) | Retrospective PnL, heuristic audit | T1 cost |
| **Tier 5** | Meta speed (weekly/generational) | Meta-learning evaluation | Near-zero |

The compounding property: after 30 days, a golem has processed ~450,000 Tier 1 corrections, ~30,000 Tier 2 assessments, ~150 Tier 3 dream evaluations, ~4 Tier 4 retrospectives, and ~4 Tier 5 meta-evaluations. Each tier amplifies the tiers above it.

### References (Slow Mirror)

- [BUTLER-1963] Butler, R.N. "The Life Review: An Interpretation of Reminiscence in the Aged." *Psychiatry*, 26(1), 65-76, 1963.
- [ROESE-1997] Roese, N.J. "Counterfactual Thinking." *Psychological Bulletin*, 121(1), 133-148, 1997.
- [KAHNEMAN-TVERSKY-1979] Kahneman, D. & Tversky, A. "Prospect Theory." *Econometrica*, 47(2), 263-292, 1979.

---

## S9 — References

- [BAILEY-PBO-2015] Bailey, D.H., Borwein, J., López de Prado, M. & Zhu, Q.J. "The Probability of Backtest Overfitting." _Journal of Computational Finance_, 2015. SSRN:2326253. — *Quantifies how likely a backtest is to overfit given the number of trials; used to deflate Sharpe ratios in the factorial analysis.*
- [BAILEY-DSR-2014] Bailey, D.H. & López de Prado, M. "The Deflated Sharpe Ratio." _Journal of Portfolio Management_, 2014. SSRN:2460551. — *Adjusts Sharpe ratios for multiple testing and non-normality; the statistical method applied to all 80 experiment runs.*
- [SHEPPERT-2026] Sheppert, A.P. "The GT-Score." _Journal of Risk and Financial Management_ 19(1):60, 2026. — *Composite risk metric combining return, drawdown, and tail risk into a single score; used alongside Sharpe for multi-dimensional comparison.*
- [LOPEZ-DE-PRADO-2018] López de Prado, M. _Advances in Financial Machine Learning_. Wiley, 2018. Chapters 11-15. — *Covers combinatorial purged cross-validation and backtest overfitting correction; the statistical framework for experiment validity.*
- [WANG-IGPO-2025] Wang et al. "Information Gain-based Policy Optimization." arXiv:2510.14967, 2025. — *Uses information gain as an intrinsic reward for exploration; supports the epistemic fitness metric as a valid measure of agent learning.*
- [CHMURA-ICE-2023] Chmura, J. et al. "Information Content Exploration." arXiv:2310.06777, 2023. — *Measures agent learning through information-theoretic novelty; informs the novel insight rate dimension in the factorial comparison.*
- [HINTON-NOWLAN-1987] Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide Evolution." _Complex Systems_ 1, 1987, pp. 495-502. — *Demonstrates that individual learning smooths the fitness landscape for evolutionary search; the theoretical basis for why generational inheritance (Mortal + Memory) should outperform mortality alone.*
- [BALDWIN-1896] Baldwin, J.M. "A New Factor in Evolution." _American Naturalist_ 30, 1896, pp. 441-451. — *Original statement of the Baldwin Effect: learned behaviors shape the selective environment for future generations; the evolutionary principle behind Bardo's knowledge inheritance.*
- [BETTINI-SND-2025] Bettini, M. et al. "System Neural Diversity." _JMLR_ 26, 2025. arXiv:2405.15054. — *Shows that maintaining behavioral diversity across agents improves collective performance; motivates measuring behavioral variance and convergence trend in the experiment.*
- [RAME-2024] Rame, A. et al. "Artificial Generational Intelligence." _NeurIPS 2024_. arXiv:2406.00392. — *Proposes generational training where successor models inherit and build on predecessors; the closest prior work to Bardo's mortal succession model.*
- [DOHARE-2024] Dohare, S. et al. "Loss of Plasticity in Deep Continual Learning." _Nature_ 632, 2024. — *Demonstrates that continual learning degrades network plasticity over time; a key argument for why mortal agents with fresh starts should outperform immortal ones.*
- [MARCH-1991] March, J.G. "Exploration and Exploitation." _Organization Science_ 2(1), 1991. — *Classic framing of the exploration-exploitation tradeoff in organizational learning; mortality forces exploration by resetting exploitation-biased agents.*
- [BESBES-2019] Besbes, O. et al. "Optimal Exploration-Exploitation in Non-stationary Rewards." _Stochastic Systems_ 9(4), 2019. — *Proves that non-stationary environments require periodic resets to avoid over-exploiting stale models; directly supports the mortality thesis.*
- [BUTLER-1963] Butler, R.N. "The Life Review." _Psychiatry_ 26(1), 1963. — *Establishes that structured retrospection before death produces higher-quality knowledge transfer; the basis for Thanatopsis Protocol death testament generation.*
- [MCADAMS-2013] McAdams, D.P. & McLean, K.C. "Narrative Identity." _Current Directions_ 22(3), 2013. — *Shows that constructing a coherent life narrative improves wisdom transfer; supports the prediction that Full Bardo (with Daimon) enriches knowledge transfer quality.*
- [MCGAUGH-2004] McGaugh, J.L. "The Amygdala Modulates Consolidation." _Annual Review of Neuroscience_ 27, 2004. — *Demonstrates that emotional arousal during encoding strengthens memory consolidation; the neuroscience basis for the Daimon's affect-weighted retrieval.*
- [FAN-2025] Fan, T. et al. "AI-Trader: Benchmarking Autonomous Agents in Real-Time Financial Markets." arXiv:2512.10971, 2025. — *Proposes evaluation methodology for autonomous trading agents; informs the experiment's multi-dimensional comparison approach.*
