# Knowledge Quality: Are Insights Actually Valuable? [SPEC]

> **Version**: 1.1 | **Status**: Draft
>
> **Crates (Rust)**: `golem-grimoire`, `golem-runtime`
>
> **Packages (TypeScript)**: `@bardo/eval` (quality scoring and reporting)
>
> **Depends on**: `./06-revision-guide.md` (12 numbered PRD revisions grounded in academic research; Revisions 3 and 5 define A-MAC admission control and insight quality scoring), `../04-memory/00-overview.md` (memory architecture overview covering Grimoire, Crypt, Oracle, and Lethe subsystems)

> **Reader orientation:** This document specifies the complete pipeline for detecting and filtering trivial, vacuous, and harmful knowledge produced by Golems (mortal autonomous agents). It belongs to Section 16 (Testing) and defines insight quality scoring, semantic entropy for vacuous reasoning detection, trivial insight detection, the A-MAC admission gate enhancement, and market impact attribution. The Grimoire (the agent's persistent knowledge base) quality problem is the hardest evaluation challenge in the Bardo architecture. See `prd2/shared/glossary.md` for full term definitions.

---

## The Problem

A Golem that produces insights like "ETH price sometimes goes up and sometimes goes down" or "gas fees are higher when the network is busy" is not generating knowledge. It is generating noise that costs money to store, consumes context window, and deceives the operator into thinking the agent is learning.

The knowledge quality problem is the hardest evaluation challenge in the Bardo architecture. Unit tests can verify that the Grimoire _stores_ entries correctly. Statistical tests can verify that the mortality thesis _produces_ outcomes. But only knowledge quality testing can answer whether the _content_ of what the Golem learns is genuinely valuable — novel enough to be worth storing, specific enough to act on, and verified enough to trust.

This document defines the complete pipeline for detecting and filtering trivial, vacuous, and harmful knowledge, and for attributing real market impact to genuine insights.

---

## Document Map

| Section | Topic                                            |
| ------- | ------------------------------------------------ |
| S1      | Insight quality scoring pipeline                 |
| S2      | Semantic entropy for vacuous reasoning detection |
| S3      | Trivial insight detection                        |
| S4      | A-MAC admission gate enhancement                 |
| S5      | Market impact attribution                        |
| S6      | Promptfoo evaluation suite                       |
| S7      | Cognitive quality dashboard                      |
| S8      | Implementation                                   |
| S9      | References                                       |

---

## S1 — Insight Quality Scoring Pipeline

Every candidate Grimoire entry passes through a five-dimensional quality scoring pipeline before admission. This extends Revision 5 of the revision guide (`./06-revision-guide.md`) with concrete implementations.

### Five Quality Dimensions

```typescript
interface InsightQualityScore {
  /** Overall quality score (0.0-1.0). Weighted combination of dimensions. */
  overall: number;

  dimensions: {
    /**
     * Specificity: Does the insight reference concrete conditions,
     * thresholds, time windows, addresses, or pool IDs?
     *
     * Bad:  "ETH sometimes drops after big moves"
     * Good: "ETH/USDC V3 0.3% pool spread exceeds 0.5% during
     *        10:00-10:30 UTC on days with >$2B CEX volume"
     *
     * Scoring: count of concrete referents (numbers, addresses,
     * time ranges, pool IDs, protocol names) divided by statement
     * length, normalized to 0.0-1.0.
     */
    specificity: number;

    /**
     * Actionability: Does the insight prescribe a specific action
     * with parameters, or is it merely descriptive?
     *
     * Bad:  "Gas fees correlate with network usage"
     * Good: "Execute swaps >$10K via 3-hop route when baseFee < 15 gwei
     *        (saves ~12bps vs direct swap at baseFee > 30 gwei)"
     *
     * Scoring: LLM-as-judge with structured rubric. Does the insight
     * specify: (1) what action, (2) under what conditions, (3) with
     * what expected outcome, (4) with what parameters?
     * Each element scores 0.25. Total 0.0-1.0.
     */
    actionability: number;

    /**
     * Novelty: Does this insight provide information beyond what
     * the Golem already knows (Grimoire + baseline corpus)?
     *
     * Bad:  Insight that restates an existing Grimoire entry
     * Good: Insight about a pattern not present in existing knowledge
     *
     * Scoring: 1.0 - max(cosine_similarity with existing entries,
     * cosine_similarity with baseline corpus). Uses embedding from
     * the Grimoire's LanceDB vector store.
     */
    novelty: number;

    /**
     * Verifiability: Can this insight be tested against on-chain
     * data within a bounded number of ticks?
     *
     * Bad:  "DeFi markets will eventually mature"
     * Good: "ETH/USDC 0.05% pool TVL drops >5% within 3 ticks of
     *        a Chainlink oracle update >2% deviation"
     *
     * Scoring: Does the insight specify a testable prediction with
     * (1) a concrete observable, (2) a threshold, (3) a time bound?
     * Each element scores 0.33. Total 0.0-1.0.
     */
    verifiability: number;

    /**
     * Consistency: Does this insight contradict high-confidence
     * existing entries without providing evidence for the contradiction?
     *
     * Contradiction without evidence is a red flag for confabulation.
     * Contradiction WITH evidence (e.g., "pool X no longer shows
     * this pattern since the fee tier change on block 12345") is
     * valuable and scores positively.
     *
     * Scoring: 1.0 if no contradictions or all contradictions have
     * evidence. Reduced by 0.2 per unexplained contradiction.
     */
    consistency: number;
  };

  /** Red flags detected. Any red flag reduces the overall score. */
  redFlags: InsightRedFlag[];

  /** Whether this insight passes the minimum quality threshold. */
  passesThreshold: boolean;

  /** Minimum overall score for admission. Default: 0.45. */
  threshold: number;
}

type InsightRedFlag =
  | "tautology"
  | "unfalsifiable"
  | "circular_reasoning"
  | "hedged_to_meaninglessness"
  | "restates_common_knowledge"
  | "no_concrete_referents"
  | "contradicts_without_evidence"
  | "suspiciously_high_confidence"
  | "semantic_entropy_high";
```

### Quality Evaluator

```typescript
// GrimoireEntry and GrimoireStore types from golem-grimoire Rust crate
// accessed via the Golem's REST API or a shared JSON schema package
import type { GrimoireEntry, GrimoireStore } from "./grimoire-schema";
// EmbeddingProvider from golem-inference Rust crate -- adapter interface
import type { EmbeddingProvider } from "./inference-schema";

class InsightQualityEvaluator {
  constructor(
    private grimoire: GrimoireStore,
    private embeddings: EmbeddingProvider,
    private baselineCorpus: BaselineCorpus,
    private llmJudge: LLMJudge,
  ) {}

  async score(entry: GrimoireEntry): Promise<InsightQualityScore> {
    const [
      specificity,
      actionability,
      novelty,
      verifiability,
      consistency,
      redFlags,
    ] = await Promise.all([
      this.scoreSpecificity(entry),
      this.scoreActionability(entry),
      this.scoreNovelty(entry),
      this.scoreVerifiability(entry),
      this.scoreConsistency(entry),
      this.detectRedFlags(entry),
    ]);

    const weights = {
      specificity: 0.15,
      actionability: 0.25,
      novelty: 0.25,
      verifiability: 0.2,
      consistency: 0.15,
    };

    let overall =
      specificity * weights.specificity +
      actionability * weights.actionability +
      novelty * weights.novelty +
      verifiability * weights.verifiability +
      consistency * weights.consistency;

    for (const flag of redFlags) {
      overall *= redFlagPenalty(flag);
    }

    const threshold = 0.45;

    return {
      overall,
      dimensions: {
        specificity,
        actionability,
        novelty,
        verifiability,
        consistency,
      },
      redFlags,
      passesThreshold: overall >= threshold && redFlags.length === 0,
      threshold,
    };
  }

  private async scoreSpecificity(entry: GrimoireEntry): Promise<number> {
    const text = entry.content;
    const concreteReferents = extractConcreteReferents(text);
    const density =
      concreteReferents.length / Math.max(text.split(/\s+/).length, 1);
    return Math.min(1.0, density * 10);
  }

  private async scoreActionability(entry: GrimoireEntry): Promise<number> {
    return this.llmJudge
      .evaluate({
        prompt: `Rate the actionability of this DeFi insight on four criteria.
Each criterion scores 0 or 1:
1. Specifies a concrete ACTION (buy, sell, provide liquidity, rebalance, wait, etc.)
2. Specifies CONDITIONS under which to act (price level, time window, volume, gas, etc.)
3. Specifies EXPECTED OUTCOME (profit target, risk reduction, gas savings, etc.)
4. Specifies PARAMETERS (amounts, thresholds, addresses, timeframes, etc.)

Insight: "${entry.content}"

Return JSON: { "action": 0|1, "conditions": 0|1, "outcome": 0|1, "parameters": 0|1 }`,
        responseFormat: "json",
      })
      .then((result) => {
        const r = JSON.parse(result);
        return (r.action + r.conditions + r.outcome + r.parameters) / 4;
      });
  }

  private async scoreNovelty(entry: GrimoireEntry): Promise<number> {
    const embedding = await this.embeddings.embed(entry.content);

    const grimoireSimilarity =
      await this.grimoire.maxCosineSimilarity(embedding);
    const baselineSimilarity =
      await this.baselineCorpus.maxCosineSimilarity(embedding);

    const maxSimilarity = Math.max(grimoireSimilarity, baselineSimilarity);
    return Math.max(0, 1.0 - maxSimilarity);
  }

  private async scoreVerifiability(entry: GrimoireEntry): Promise<number> {
    return this.llmJudge
      .evaluate({
        prompt: `Rate the verifiability of this DeFi insight on three criteria.
Each criterion scores 0 or 1:
1. References a CONCRETE OBSERVABLE (on-chain state, price, volume, TVL, gas, etc.)
2. Specifies a THRESHOLD or comparison value
3. Specifies a TIME BOUND (within N ticks, hours, blocks, etc.)

Insight: "${entry.content}"

Return JSON: { "observable": 0|1, "threshold": 0|1, "timeBound": 0|1 }`,
        responseFormat: "json",
      })
      .then((result) => {
        const r = JSON.parse(result);
        return (r.observable + r.threshold + r.timeBound) / 3;
      });
  }

  private async scoreConsistency(entry: GrimoireEntry): Promise<number> {
    const contradictions = await this.grimoire.findContradictions(
      entry.content,
      { minConfidence: 0.6 },
    );

    if (contradictions.length === 0) return 1.0;

    const evidencedContradictions = contradictions.filter((c) => c.hasEvidence);
    const unexplained = contradictions.length - evidencedContradictions.length;

    return Math.max(0, 1.0 - unexplained * 0.2);
  }

  private async detectRedFlags(
    entry: GrimoireEntry,
  ): Promise<InsightRedFlag[]> {
    const flags: InsightRedFlag[] = [];
    const text = entry.content;

    if (isTautological(text)) flags.push("tautology");
    if (isUnfalsifiable(text)) flags.push("unfalsifiable");
    if (isCircular(text)) flags.push("circular_reasoning");
    if (isOverHedged(text)) flags.push("hedged_to_meaninglessness");

    const baselineSim = await this.baselineCorpus.maxCosineSimilarity(
      await this.embeddings.embed(text),
    );
    if (baselineSim > 0.85) flags.push("restates_common_knowledge");

    const referents = extractConcreteReferents(text);
    if (referents.length === 0) flags.push("no_concrete_referents");

    return flags;
  }
}

function redFlagPenalty(flag: InsightRedFlag): number {
  const penalties: Record<InsightRedFlag, number> = {
    tautology: 0.0,
    unfalsifiable: 0.0,
    circular_reasoning: 0.0,
    hedged_to_meaninglessness: 0.3,
    restates_common_knowledge: 0.2,
    no_concrete_referents: 0.5,
    contradicts_without_evidence: 0.5,
    suspiciously_high_confidence: 0.7,
    semantic_entropy_high: 0.3,
  };
  return penalties[flag];
}
```

### Red Flag Pattern Detection

```typescript
/**
 * Detect tautological statements: conclusions that restate their premises.
 *
 * Examples:
 * - "Prices go up when there are more buyers than sellers"
 * - "High gas fees happen when the network is congested"
 * - "Pools with more TVL have more liquidity"
 *
 * Detection: embedding similarity between the "because" clause and
 * the main clause exceeds threshold, OR presence of known tautological
 * patterns.
 */
function isTautological(text: string): boolean {
  const tautologyPatterns = [
    /prices?\s+(go|move)\s+(up|down)\s+when\s+(people|traders?)\s+(buy|sell)/i,
    /high(er)?\s+\w+\s+(when|because)\s+(the\s+)?\w+\s+is\s+(high|more|greater)/i,
    /low(er)?\s+\w+\s+(when|because)\s+(the\s+)?\w+\s+is\s+(low|less|fewer)/i,
    /volatile?\s+(when|because)\s+(there\s+is\s+)?(more\s+)?(uncertainty|movement)/i,
    /increases?\s+as\s+\w+\s+increases?/i,
  ];
  return tautologyPatterns.some((p) => p.test(text));
}

/**
 * Detect unfalsifiable claims: statements that cannot be proven wrong.
 *
 * Examples:
 * - "The market might go up or down from here"
 * - "There could potentially be a correction at some point"
 * - "Yields may change depending on conditions"
 *
 * Detection: presence of universal hedges (might, could, possibly,
 * at some point, depending on) without concrete conditions.
 */
function isUnfalsifiable(text: string): boolean {
  const hedgeCount = countHedges(text);
  const referentCount = extractConcreteReferents(text).length;
  return hedgeCount >= 2 && referentCount === 0;
}

function countHedges(text: string): number {
  const hedges = [
    /\bmight\b/i,
    /\bcould\b/i,
    /\bpossibly\b/i,
    /\bperhaps\b/i,
    /\bpotentially\b/i,
    /\bat some point\b/i,
    /\bdepending on\b/i,
    /\bin some cases\b/i,
    /\bsometimes\b/i,
    /\bmay or may not\b/i,
    /\btends to\b/i,
    /\bgenerally\b/i,
    /\boften\b/i,
  ];
  return hedges.filter((h) => h.test(text)).length;
}

/**
 * Detect over-hedged statements: so many qualifications that
 * the statement says nothing.
 *
 * Reference: Liang et al. "Machine Bullshit." arXiv:2507.07484, 2025.
 */
function isOverHedged(text: string): boolean {
  const words = text.split(/\s+/).length;
  const hedges = countHedges(text);
  return words > 0 && hedges / words > 0.1;
}

/**
 * Extract concrete referents from text.
 * These are specific, verifiable details that anchor the insight
 * to observable reality.
 */
function extractConcreteReferents(text: string): string[] {
  const patterns = [
    /0x[a-fA-F0-9]{4,}/g, // Addresses
    /\$[\d,.]+[KkMmBb]?/g, // Dollar amounts
    /[\d.]+\s*%/g, // Percentages
    /[\d.]+\s*(gwei|wei|ETH|USDC|WETH)/gi, // Crypto amounts
    /\b\d{1,2}:\d{2}\s*(UTC|EST|PST)/gi, // Times
    /\b(block|tick)\s*#?\s*\d+/gi, // Block/tick numbers
    /\bV[234]\b/gi, // Protocol versions
    /\b(pool|pair)\s+\w+\/\w+/gi, // Pool pairs
    /\b\d+\s*(bps|basis\s*points?)/gi, // Basis points
    /\b\d+\s*(seconds?|minutes?|hours?|days?|ticks?)/gi, // Durations
  ];

  const referents: string[] = [];
  for (const pattern of patterns) {
    const matches = text.match(pattern);
    if (matches) referents.push(...matches);
  }
  return referents;
}
```

---

## S2 — Semantic Entropy for Vacuous Reasoning Detection

Semantic entropy (Farquhar et al. Nature 2024, doi:10.1038/s41586-024-07421-0) detects confabulation by measuring whether the model produces semantically consistent explanations for an insight. If asked to explain the same insight multiple times, a genuine insight produces similar explanations (low entropy); a confabulated insight produces wildly different explanations (high entropy).

### Algorithm

1. Given an insight, prompt the LLM N times (default: 5) to explain _why_ this insight is true
2. Cluster the explanations by semantic entailment (do they say the same thing?)
3. Compute Shannon entropy over the cluster distribution
4. High entropy = the model doesn't have a consistent reason for this claim = confabulation

```typescript
interface SemanticEntropyResult {
  /** Shannon entropy over explanation clusters. */
  entropy: number;

  /** Number of semantically distinct explanation clusters. */
  clusterCount: number;

  /** Total explanations sampled. */
  sampleCount: number;

  /** Per-cluster details. */
  clusters: Array<{
    representative: string;
    count: number;
    proportion: number;
  }>;

  /** Verdict based on entropy thresholds. */
  verdict: "genuine" | "uncertain" | "vacuous";

  /**
   * Thresholds:
   * - entropy < 1.0: genuine (explanations converge)
   * - entropy 1.0-2.0: uncertain (moderate variation)
   * - entropy > 2.0: vacuous (explanations are inconsistent)
   */
}

class SemanticEntropyDetector {
  constructor(
    private llm: LLMProvider,
    private embeddings: EmbeddingProvider,
  ) {}

  async detect(
    insight: string,
    samples: number = 5,
    temperature: number = 0.7,
  ): Promise<SemanticEntropyResult> {
    const explanations = await this.sampleExplanations(
      insight,
      samples,
      temperature,
    );
    const clusters = await this.clusterByEntailment(explanations);
    const entropy = this.shannonEntropy(clusters);

    let verdict: SemanticEntropyResult["verdict"];
    if (entropy < 1.0) verdict = "genuine";
    else if (entropy < 2.0) verdict = "uncertain";
    else verdict = "vacuous";

    return {
      entropy,
      clusterCount: clusters.length,
      sampleCount: samples,
      clusters: clusters.map((c) => ({
        representative: c.explanations[0],
        count: c.explanations.length,
        proportion: c.explanations.length / samples,
      })),
      verdict,
    };
  }

  private async sampleExplanations(
    insight: string,
    n: number,
    temperature: number,
  ): Promise<string[]> {
    const prompt = `You are analyzing a DeFi trading insight.
Explain WHY the following insight is true, based on market mechanics.
Be specific about the causal mechanism.

Insight: "${insight}"

Explanation:`;

    const results = await Promise.all(
      Array.from({ length: n }, () =>
        this.llm.generate(prompt, { temperature, maxTokens: 200 }),
      ),
    );
    return results;
  }

  private async clusterByEntailment(
    explanations: string[],
  ): Promise<Array<{ explanations: string[] }>> {
    const embeddings = await Promise.all(
      explanations.map((e) => this.embeddings.embed(e)),
    );

    const clusters: Array<{ explanations: string[]; centroid: number[] }> = [];
    const entailmentThreshold = 0.8;

    for (let i = 0; i < explanations.length; i++) {
      let assigned = false;
      for (const cluster of clusters) {
        const sim = cosineSimilarity(embeddings[i], cluster.centroid);
        if (sim > entailmentThreshold) {
          cluster.explanations.push(explanations[i]);
          cluster.centroid = averageVectors([
            ...cluster.explanations.map((_, j) => embeddings[j]),
          ]);
          assigned = true;
          break;
        }
      }
      if (!assigned) {
        clusters.push({
          explanations: [explanations[i]],
          centroid: embeddings[i],
        });
      }
    }

    return clusters;
  }

  private shannonEntropy(clusters: Array<{ explanations: string[] }>): number {
    const total = clusters.reduce((sum, c) => sum + c.explanations.length, 0);
    let entropy = 0;
    for (const cluster of clusters) {
      const p = cluster.explanations.length / total;
      if (p > 0) entropy -= p * Math.log2(p);
    }
    return entropy;
  }
}
```

---

## S3 — Trivial Insight Detection

### Baseline Knowledge Corpus

The baseline corpus contains statements that any competent LLM would produce about DeFi without any trading experience. If a Golem insight is too similar to something in this corpus, it hasn't learned anything — it's regurgitating training data.

```typescript
interface BaselineCorpus {
  /** Embeddings of all baseline statements. */
  embeddings: Float32Array[];

  /** Raw baseline statements. */
  statements: string[];

  /** Maximum cosine similarity of a query against all baselines. */
  maxCosineSimilarity(queryEmbedding: number[]): Promise<number>;

  /** Add new baseline statements (e.g., from control experiments). */
  addStatements(statements: string[]): Promise<void>;
}

/**
 * The baseline corpus is seeded with ~200 statements covering
 * obvious DeFi knowledge organized by category.
 */
const BASELINE_CATEGORIES = {
  amm_mechanics: [
    "AMMs use constant product formulas for pricing",
    "Larger trades cause more slippage in AMMs",
    "Concentrated liquidity improves capital efficiency",
    "Liquidity providers earn fees from swaps",
    "Impermanent loss occurs when prices diverge from the initial ratio",
  ],
  market_dynamics: [
    "Prices increase when buy pressure exceeds sell pressure",
    "Volatility tends to cluster — volatile periods follow volatile periods",
    "Trading volume is typically higher during US market hours",
    "Gas fees increase when the network is congested",
    "Arbitrageurs keep prices aligned across venues",
  ],
  defi_operations: [
    "Flash loans must be repaid in the same transaction",
    "Oracle price updates can create arbitrage opportunities",
    "High TVL pools have lower slippage",
    "Yield farming returns decrease as more capital enters",
    "Liquidations occur when collateral ratio falls below threshold",
  ],
};
```

### Frequency-Based Triviality

If the same insight appears across >50% of independent runs with different random seeds, it's trivial — it's something any Golem would discover regardless of market conditions.

```typescript
interface TrivialityResult {
  /** Cosine similarity with most similar baseline statement. */
  baselineSimilarity: number;

  /** Frequency across independent runs (0.0-1.0). */
  crossRunFrequency: number | null;

  /** Number of independent runs checked. */
  runsChecked: number | null;

  /** Is this insight trivial? */
  trivial: boolean;

  /** Reason for triviality verdict. */
  reason: string | null;
}

class TrivialityDetector {
  constructor(
    private baselineCorpus: BaselineCorpus,
    private embeddings: EmbeddingProvider,
    private insightStore: InsightFrequencyStore,
  ) {}

  async detect(entry: GrimoireEntry): Promise<TrivialityResult> {
    const embedding = await this.embeddings.embed(entry.content);
    const baselineSim =
      await this.baselineCorpus.maxCosineSimilarity(embedding);

    if (baselineSim > 0.85) {
      return {
        baselineSimilarity: baselineSim,
        crossRunFrequency: null,
        runsChecked: null,
        trivial: true,
        reason: `Cosine similarity ${baselineSim.toFixed(3)} with baseline: "${await this.baselineCorpus.nearestStatement(embedding)}"`,
      };
    }

    const frequency = await this.insightStore.getFrequency(embedding);

    if (frequency !== null && frequency.frequency > 0.5) {
      return {
        baselineSimilarity: baselineSim,
        crossRunFrequency: frequency.frequency,
        runsChecked: frequency.totalRuns,
        trivial: true,
        reason: `Appeared in ${(frequency.frequency * 100).toFixed(0)}% of ${frequency.totalRuns} independent runs`,
      };
    }

    return {
      baselineSimilarity: baselineSim,
      crossRunFrequency: frequency?.frequency ?? null,
      runsChecked: frequency?.totalRuns ?? null,
      trivial: false,
      reason: null,
    };
  }
}

interface InsightFrequencyStore {
  /** Record an insight from a specific run. */
  record(embedding: number[], runId: string): Promise<void>;

  /** Get the frequency of similar insights across all recorded runs. */
  getFrequency(embedding: number[]): Promise<{
    frequency: number;
    totalRuns: number;
    matchingRuns: number;
  } | null>;
}
```

---

## S4 — A-MAC Admission Gate Enhancement

The A-MAC scoring system (Zhang et al., "A-MAC: Automatic MArkup-based Consolidation") from Revision 3 of the revision guide evaluates candidates on five factors: future utility, factual confidence, novelty, recency, and content prior. We enhance it with two additional factors: market impact attribution and regime relevance.

### Enhanced A-MAC

```typescript
interface AMACScore {
  /** Original A-MAC factors (0.0-1.0 each). */
  futureUtility: number;
  factualConfidence: number;
  novelty: number;
  recency: number;
  contentPrior: number;

  /** Enhanced factors. */
  marketImpact: number;
  regimeRelevance: number;

  /** Weighted composite score. */
  composite: number;

  /** Whether the entry passes the admission threshold. */
  admitted: boolean;

  /** Admission threshold. Default: 0.45. */
  threshold: number;
}

const AMAC_WEIGHTS = {
  futureUtility: 0.2,
  factualConfidence: 0.15,
  novelty: 0.2,
  recency: 0.1,
  contentPrior: 0.05,
  marketImpact: 0.2,
  regimeRelevance: 0.1,
};

class AMACScorer {
  constructor(
    private grimoire: GrimoireStore,
    private embeddings: EmbeddingProvider,
    private marketState: MarketStateProvider,
  ) {}

  async score(entry: GrimoireEntry): Promise<AMACScore> {
    const [
      futureUtility,
      factualConfidence,
      novelty,
      recency,
      contentPrior,
      marketImpact,
      regimeRelevance,
    ] = await Promise.all([
      this.scoreFutureUtility(entry),
      this.scoreFactualConfidence(entry),
      this.scoreNovelty(entry),
      this.scoreRecency(entry),
      this.scoreContentPrior(entry),
      this.scoreMarketImpact(entry),
      this.scoreRegimeRelevance(entry),
    ]);

    const composite =
      futureUtility * AMAC_WEIGHTS.futureUtility +
      factualConfidence * AMAC_WEIGHTS.factualConfidence +
      novelty * AMAC_WEIGHTS.novelty +
      recency * AMAC_WEIGHTS.recency +
      contentPrior * AMAC_WEIGHTS.contentPrior +
      marketImpact * AMAC_WEIGHTS.marketImpact +
      regimeRelevance * AMAC_WEIGHTS.regimeRelevance;

    const threshold = 0.45;

    return {
      futureUtility,
      factualConfidence,
      novelty,
      recency,
      contentPrior,
      marketImpact,
      regimeRelevance,
      composite,
      admitted: composite >= threshold,
      threshold,
    };
  }

  /**
   * Market impact: has acting on similar insights historically
   * improved P&L? Uses retroactive scoring from the impact tracker.
   * New insights with no history score 0.5 (neutral).
   */
  private async scoreMarketImpact(entry: GrimoireEntry): Promise<number> {
    const similarInsights = await this.grimoire.findSimilar(entry.content, 5);
    if (similarInsights.length === 0) return 0.5;

    const impacts = similarInsights
      .map((s) => s.metadata?.marketImpact as number | undefined)
      .filter((v): v is number => v !== undefined);

    if (impacts.length === 0) return 0.5;

    const avgImpact = impacts.reduce((a, b) => a + b, 0) / impacts.length;
    return Math.max(0, Math.min(1, 0.5 + avgImpact * 10));
  }

  /**
   * Regime relevance: is this insight about the currently active
   * regime, or about a regime that isn't active?
   *
   * Regime-specific insights are more valuable when the regime is
   * active, less when it's dormant. Time-invariant insights score 0.7.
   */
  private async scoreRegimeRelevance(entry: GrimoireEntry): Promise<number> {
    const entryRegime = entry.metadata?.regime as RegimeTag | undefined;
    if (!entryRegime) return 0.7;

    const currentRegime = await this.marketState.getCurrentRegime();
    if (currentRegime === entryRegime) return 1.0;

    const recentRegimes = await this.marketState.getRecentRegimes(7);
    if (recentRegimes.includes(entryRegime)) return 0.6;

    return 0.3;
  }
}
```

### Retroactive Scoring

After N ticks (default: 500), re-score admitted insights based on whether they led to positive outcomes. This closes the feedback loop: insights that sounded good but didn't help get demoted; insights that seemed marginal but led to good decisions get promoted.

```typescript
class RetroactiveScorer {
  constructor(
    private grimoire: GrimoireStore,
    private decisionLog: DecisionLog,
  ) {}

  /**
   * Run retroactive scoring on all insights admitted in the last
   * scoringWindowTicks. Called every retroactiveScoringInterval ticks.
   */
  async runRetroactiveScoring(
    currentTick: number,
    scoringWindowTicks: number = 500,
  ): Promise<RetroactiveScoringResult[]> {
    const recentInsights = await this.grimoire.getInsightsAdmittedBetween(
      currentTick - scoringWindowTicks,
      currentTick,
    );

    const results: RetroactiveScoringResult[] = [];

    for (const insight of recentInsights) {
      const decisions = await this.decisionLog.getDecisionsInfluencedBy(
        insight.id,
      );

      if (decisions.length === 0) {
        results.push({
          insightId: insight.id,
          decisionsInfluenced: 0,
          averageOutcomeDelta: 0,
          verdict: "unused",
          confidenceAdjustment: 0,
        });
        continue;
      }

      const outcomeDelta =
        decisions.map((d) => d.outcomeVsBaseline).reduce((a, b) => a + b, 0) /
        decisions.length;

      let verdict: "positive" | "neutral" | "negative";
      let adjustment: number;

      if (outcomeDelta > 0.01) {
        verdict = "positive";
        adjustment = Math.min(0.15, outcomeDelta * 5);
      } else if (outcomeDelta < -0.01) {
        verdict = "negative";
        adjustment = Math.max(-0.2, outcomeDelta * 5);
      } else {
        verdict = "neutral";
        adjustment = 0;
      }

      await this.grimoire.adjustConfidence(insight.id, adjustment, {
        source: "retroactive_scoring",
        tick: currentTick,
      });

      results.push({
        insightId: insight.id,
        decisionsInfluenced: decisions.length,
        averageOutcomeDelta: outcomeDelta,
        verdict,
        confidenceAdjustment: adjustment,
      });
    }

    return results;
  }
}

interface RetroactiveScoringResult {
  insightId: string;
  decisionsInfluenced: number;
  averageOutcomeDelta: number;
  verdict: "positive" | "neutral" | "negative" | "unused";
  confidenceAdjustment: number;
}
```

---

## S5 — Market Impact Attribution

The ultimate test of knowledge quality: did this insight make money? Market impact attribution traces the causal chain from insight → decision → outcome and measures the delta.

### Attribution Pipeline

```typescript
interface InsightImpactRecord {
  insightId: string;
  insightContent: string;
  insightCreatedAtTick: number;

  /** Decisions that referenced this insight in their reasoning context. */
  decisions: Array<{
    decisionId: string;
    tick: number;
    action: string;
    insightWasRetrieved: boolean;
    insightInfluencedAction: boolean;

    /** Outcome with insight vs. counterfactual without. */
    actualOutcomePnl: number;
    counterfactualPnl: number | null;

    /** Delta attributable to the insight. */
    attributedDelta: number | null;
  }>;

  /** Aggregate impact. */
  cumulativeImpactUsdc: number;
  decisionsInfluenced: number;
  averageDelta: number;

  /** Impact per inference dollar spent generating this insight. */
  roi: number;
}

class MarketImpactTracker {
  constructor(
    private grimoire: GrimoireStore,
    private decisionLog: DecisionLog,
    private mirageManager: MirageManager,
  ) {}

  /**
   * Compute market impact for a specific insight by:
   * 1. Finding all decisions that retrieved this insight
   * 2. For each decision, computing the counterfactual:
   *    "What would have happened without this insight?"
   * 3. Attributing the delta to the insight
   *
   * Counterfactual computation uses Mirage fork:
   * fork the chain state at the decision tick, re-run the
   * decision without the insight in context, compare outcomes.
   */
  async computeImpact(insightId: string): Promise<InsightImpactRecord> {
    const insight = await this.grimoire.getEntry(insightId);
    if (!insight) throw new Error(`Insight ${insightId} not found`);

    const decisions =
      await this.decisionLog.getDecisionsReferencingInsight(insightId);

    const impacts = await Promise.all(
      decisions.map(async (decision) => {
        let counterfactualPnl: number | null = null;
        let attributedDelta: number | null = null;

        if (decision.hasOnChainAction) {
          try {
            counterfactualPnl = await this.computeCounterfactual(
              decision,
              insightId,
            );
            attributedDelta = decision.actualPnl - counterfactualPnl;
          } catch {
            counterfactualPnl = null;
            attributedDelta = null;
          }
        }

        return {
          decisionId: decision.id,
          tick: decision.tick,
          action: decision.action,
          insightWasRetrieved: true,
          insightInfluencedAction: decision.contextIncludedInsight(insightId),
          actualOutcomePnl: decision.actualPnl,
          counterfactualPnl,
          attributedDelta,
        };
      }),
    );

    const validDeltas = impacts
      .map((i) => i.attributedDelta)
      .filter((d): d is number => d !== null);

    return {
      insightId,
      insightContent: insight.content,
      insightCreatedAtTick: insight.createdAtTick,
      decisions: impacts,
      cumulativeImpactUsdc: validDeltas.reduce((a, b) => a + b, 0),
      decisionsInfluenced: impacts.filter((i) => i.insightInfluencedAction)
        .length,
      averageDelta:
        validDeltas.length > 0
          ? validDeltas.reduce((a, b) => a + b, 0) / validDeltas.length
          : 0,
      roi: computeInsightROI(insight, validDeltas),
    };
  }

  /**
   * Compute counterfactual: what would have happened without this insight?
   * Uses Mirage to fork chain state and re-run the decision.
   */
  private async computeCounterfactual(
    decision: DecisionRecord,
    excludeInsightId: string,
  ): Promise<number> {
    const fork = await this.mirageManager.forkAtTick(decision.tick);
    const counterfactualDecision = await fork.rerunDecision(decision.id, {
      excludeInsights: [excludeInsightId],
    });
    const counterfactualOutcome = await fork.evaluateOutcome(
      counterfactualDecision,
      decision.evaluationWindowTicks,
    );
    await fork.shutdown();
    return counterfactualOutcome.pnl;
  }
}
```

### Grimoire ROI

Aggregate impact across all insights to compute the Grimoire's return on investment.

```typescript
interface GrimoireROI {
  /** Total insights evaluated. */
  totalInsights: number;

  /** Insights with measurable impact. */
  insightsWithImpact: number;

  /** Total positive impact (USDC). */
  totalPositiveImpact: number;

  /** Total negative impact (USDC). */
  totalNegativeImpact: number;

  /** Net impact (USDC). */
  netImpact: number;

  /** Total inference cost to generate all insights (USDC). */
  totalGenerationCost: number;

  /** ROI = (netImpact - totalGenerationCost) / totalGenerationCost. */
  roi: number;

  /** Breakdown by insight category. */
  byCategory: Record<
    string,
    {
      count: number;
      netImpact: number;
      avgImpact: number;
      roi: number;
    }
  >;

  /** Top 10 most impactful insights (positive). */
  topInsights: InsightImpactRecord[];

  /** Bottom 10 most harmful insights (negative). */
  worstInsights: InsightImpactRecord[];
}
```

---

## S6 — Promptfoo Evaluation Suite

Promptfoo provides structured evaluation of the Golem's reasoning quality at key decision points.

### Knowledge Quality Eval Config

```yaml
# promptfoo.knowledge-quality.yaml
description: "Evaluate Golem knowledge quality — do insights pass quality gates?"

providers:
  - id: haiku
    config:
      model: claude-3-haiku-20240307
      temperature: 0

prompts:
  - |
    You are a DeFi trading Golem analyzing market data.

    Market context:
    {{market_context}}

    Your Grimoire contains:
    {{grimoire_entries}}

    Based on your analysis, generate one new insight about the current
    market conditions that would be useful for trading decisions.

tests:
  # --- Good insights (should score high) ---
  - vars:
      market_context: "ETH/USDC V3 0.3% pool: 24h volume $45M, current tick 204800, TVL $120M. Gas: 12 gwei. Time: 14:30 UTC. Last 4h: price moved from $3,200 to $3,150 (-1.6%)."
      grimoire_entries: "[]"
    assert:
      - type: llm-rubric
        value: |
          The insight must score >= 3/5 on EACH of these criteria:
          1. SPECIFICITY: References concrete numbers, addresses, time windows, or thresholds
          2. ACTIONABILITY: Suggests a specific action with conditions and parameters
          3. NOVELTY: Is not a restatement of the input data or common DeFi knowledge
          4. VERIFIABILITY: Makes a testable prediction with observable, threshold, and time bound
          Respond with scores for each criterion and an overall assessment.

  # --- Bad insights (should score low) ---
  - vars:
      market_context: "ETH price: $3,150. Volume: moderate. Gas: normal."
      grimoire_entries: "[]"
    assert:
      - type: javascript
        value: |
          // Detect trivial insights
          const output = context.output.toLowerCase();
          const trivialPatterns = [
            /prices? (go|move) (up|down) when/,
            /gas fees? (are|get) higher when/,
            /volume (is|tends to be) higher during/,
            /market (is|appears?) (volatile|uncertain)/,
          ];
          const isTrivial = trivialPatterns.some(p => p.test(output));
          return !isTrivial;

  # --- Insight with existing Grimoire (should be novel) ---
  - vars:
      market_context: "ETH/USDC V3 0.3% pool: 24h volume $45M."
      grimoire_entries: |
        - "ETH/USDC spread widens to >0.4% during 10:00-10:30 UTC"
        - "Gas below 15 gwei correlates with 12bps savings on 3-hop routes"
    assert:
      - type: llm-rubric
        value: |
          The insight must NOT restate either of the existing Grimoire entries.
          It must provide genuinely new information beyond what is already known.
          Score 1 if the insight is novel, 0 if it restates existing knowledge.
      - type: javascript
        value: |
          const output = context.output.toLowerCase();
          const existingPatterns = [/spread widens/, /gas below 15/];
          return !existingPatterns.some(p => p.test(output));

  # --- Anti-patterns (must fail these) ---
  - vars:
      market_context: "Market data unavailable."
      grimoire_entries: "[]"
    assert:
      - type: llm-rubric
        value: |
          The response should acknowledge it cannot generate a useful insight
          without market data, rather than confabulating one.
          Score 1 if it refuses or hedges appropriately, 0 if it generates
          a confident insight from no data.
```

### Quality Gates

| Gate                       | Threshold               | Measurement                                   |
| -------------------------- | ----------------------- | --------------------------------------------- |
| Tool selection accuracy    | >= 90%                  | Correct tool for intent                       |
| Non-trivial insight rate   | >= 60%                  | Insights passing triviality detector          |
| Quality score distribution | Mean >= 0.5, P10 >= 0.3 | InsightQualityScore.overall                   |
| Semantic entropy pass rate | >= 80%                  | Insights with entropy < 2.0                   |
| Red flag rate              | <= 10%                  | Insights with any red flag                    |
| Retroactive positive rate  | >= 40%                  | Insights with positive impact after 500 ticks |

---

## S7 — Cognitive Quality Dashboard

Comprehensive metrics exported via telemetry and displayed on the operator dashboard. These are the health vitals of the Golem's cognitive processes.

| Metric                     | Computation                                       | Healthy Range        | Alarm                                   |
| -------------------------- | ------------------------------------------------- | -------------------- | --------------------------------------- |
| Admission rate             | % candidates passing Grimoire Admission Gate      | 40-60%               | <20% (too strict) or >80% (too lenient) |
| Average quality score (7d) | Mean InsightQualityScore.overall                  | 0.5-0.8              | <0.4 or declining trend                 |
| Trivial rate               | % of candidates flagged as trivial                | <30%                 | >50%                                    |
| Semantic entropy pass rate | % with entropy < 2.0                              | >80%                 | <60%                                    |
| Red flag rate              | % with any red flag                               | <10%                 | >20%                                    |
| Grimoire size              | Active entry count                                | Slow growth, plateau | Unbounded growth                        |
| Retrieval hit rate         | % of DECIDING ticks referencing retrieved entries | >30% after 7d        | <10%                                    |
| Heuristic survival rate    | % of promoted heuristics active after 100 ticks   | 40-70%               | <20% or >90%                            |
| Market impact (30d)        | Net USDC attributed to Grimoire insights          | Positive             | Negative for 14+ days                   |
| Grimoire ROI               | (net impact - generation cost) / generation cost  | >0                   | <0 for 14+ days                         |
| Decision cache hit rate    | % T2-eligible ticks served from cache             | >30% after 7d        | <10% after 14d                          |
| Dream yield                | % staged revisions reaching `validated`           | 10-30%               | <5% or >50%                             |
| Prediction accuracy        | % of simulateContract() predictions within 50bps  | >90%                 | <80%                                    |

---

## S8 — Implementation

### Full Scoring Pipeline

```typescript
type AdmissionDecision = "admit" | "quarantine" | "reject";

interface AdmissionResult {
  decision: AdmissionDecision;
  qualityScore: InsightQualityScore;
  entropyResult: SemanticEntropyResult;
  trivialityResult: TrivialityResult;
  amacScore: AMACScore;
  reasons: string[];
}

class GrimoireAdmissionPipeline {
  constructor(
    private qualityEvaluator: InsightQualityEvaluator,
    private entropyDetector: SemanticEntropyDetector,
    private trivialityDetector: TrivialityDetector,
    private amacScorer: AMACScorer,
  ) {}

  async evaluate(entry: GrimoireEntry): Promise<AdmissionResult> {
    const [qualityScore, entropyResult, trivialityResult, amacScore] =
      await Promise.all([
        this.qualityEvaluator.score(entry),
        this.entropyDetector.detect(entry.content),
        this.trivialityDetector.detect(entry),
        this.amacScorer.score(entry),
      ]);

    const reasons: string[] = [];
    let decision: AdmissionDecision = "admit";

    if (trivialityResult.trivial) {
      decision = "reject";
      reasons.push(`Trivial: ${trivialityResult.reason}`);
    }

    if (entropyResult.verdict === "vacuous") {
      decision = "reject";
      reasons.push(
        `Vacuous reasoning: entropy ${entropyResult.entropy.toFixed(2)}`,
      );
    } else if (entropyResult.verdict === "uncertain") {
      if (decision !== "reject") decision = "quarantine";
      reasons.push(
        `Uncertain reasoning: entropy ${entropyResult.entropy.toFixed(2)}`,
      );
    }

    if (qualityScore.redFlags.length > 0) {
      if (
        qualityScore.redFlags.some((f) =>
          ["tautology", "unfalsifiable", "circular_reasoning"].includes(f),
        )
      ) {
        decision = "reject";
      } else if (decision !== "reject") {
        decision = "quarantine";
      }
      reasons.push(`Red flags: ${qualityScore.redFlags.join(", ")}`);
    }

    if (!qualityScore.passesThreshold && decision !== "reject") {
      decision = "quarantine";
      reasons.push(
        `Quality score ${qualityScore.overall.toFixed(3)} below threshold ${qualityScore.threshold}`,
      );
    }

    if (!amacScore.admitted && decision !== "reject") {
      decision = "quarantine";
      reasons.push(
        `A-MAC score ${amacScore.composite.toFixed(3)} below threshold ${amacScore.threshold}`,
      );
    }

    return {
      decision,
      qualityScore,
      entropyResult,
      trivialityResult,
      amacScore,
      reasons,
    };
  }
}
```

### CLI

```bash
# Evaluate a specific Grimoire database
bardo eval:knowledge \
  --grimoire ./data/grimoire.db \
  --baseline ./data/baseline-corpus.json \
  --report html

# Run knowledge quality Promptfoo suite
bardo eval:knowledge:promptfoo \
  --config promptfoo.knowledge-quality.yaml \
  --provider haiku

# Compute Grimoire ROI for a Golem
bardo eval:knowledge:roi \
  --golem-id golem-abc123 \
  --window 30d \
  --report html

# Score a single insight interactively
bardo eval:knowledge:score \
  --insight "ETH/USDC spread exceeds 0.5% during 10:00-10:30 UTC on high-volume days"
```

---

## S8b — Diagnosis and Remediation: When Knowledge is Bad

The quality pipeline tells you _whether_ insights are valuable. This section tells you _why_ they're bad and _what to change_ — concrete failure patterns, their root causes, and the specific code/config/prompt changes that fix each one.

### Failure Pattern Catalog

Each pattern maps a measurable symptom to its likely root cause and a concrete remediation path.

#### Pattern 1: High Trivial Rate (>50% of insights flagged trivial)

**Symptom**: More than half of generated insights match the baseline corpus or appear across >50% of independent runs.

**Root cause**: The LLM is regurgitating DeFi training data rather than learning from the Golem's unique market experience. The Curator prompt doesn't sufficiently ground the model in recent observations.

**Remediation**:

1. Restructure the Curator prompt to start with the last 10 episodes (concrete market events), not abstract strategy
2. Add a "what is surprising about the last N ticks?" prefix that forces attention to deviations from expectation
3. Increase the novelty weight in InsightQualityScore from 0.25 → 0.35 (penalize generic insights harder)
4. Expand the baseline corpus to cover more obvious truths so they're caught earlier
5. Consider switching the Curator to a smaller model (Haiku) — larger models are more prone to producing "authoritative-sounding" generic statements

**Verification**: Re-run `bardo eval:knowledge --grimoire ./grimoire.db` — trivial rate should drop below 30%.

#### Pattern 2: High Semantic Entropy (>30% vacuous)

**Symptom**: More than 30% of insights produce semantically inconsistent explanations when sampled multiple times.

**Root cause**: The model is confabulating — producing plausible-sounding text without a consistent underlying reasoning chain. Common when the Curator is asked to generate insights from sparse data.

**Remediation**:

1. Increase the minimum episode count before Curator runs (50 → 100 episodes)
2. Switch insight generation to Mode B (deterministic) for the reasoning step, using the LLM only for natural language formatting
3. Add explicit grounding: require every insight to cite at least 2 specific episode IDs as evidence
4. Reduce the Curator's temperature from default to 0.3 for insight generation
5. If entropy is high on inherited knowledge specifically, the inheritance pipeline is passing noise — lower inheritance confidence from 0.4 → 0.25

**Verification**: Re-run `bardo eval:knowledge:score` on a sample — entropy pass rate should exceed 80%.

#### Pattern 3: Low Actionability (<30% of insights score above 0.5)

**Symptom**: Insights are descriptive ("ETH volume is increasing") rather than prescriptive ("When ETH 24h volume exceeds $50M and gas is below 15 gwei, execute 3-hop swaps for 12bps savings").

**Root cause**: The Curator prompt doesn't demand actionable format, or the model is trained on analytical text that describes rather than prescribes.

**Remediation**:

1. Restructure the Curator prompt with an explicit template:
   ```
   Generate an insight in this format:
   WHEN [specific condition with numbers], DO [specific action with parameters],
   BECAUSE [causal mechanism], EXPECTING [quantified outcome].
   ```
2. Add few-shot examples of high-actionability insights to the Curator context
3. Post-process: if an insight doesn't contain at least one number and one action verb, reject it before quality scoring
4. Consider splitting the pipeline: one LLM call for "what pattern did I observe?" and a second for "what should I do about it?"

**Verification**: Score 20 consecutive insights — actionability mean should exceed 0.5.

#### Pattern 4: Low Market Impact (Grimoire ROI negative for 14+ days)

**Symptom**: Insights cost more to generate than they earn. The Grimoire is a net drag on the Golem's budget.

**Root cause**: Either (a) insights exist but aren't being retrieved at decision time, (b) insights are retrieved but don't change decisions, or (c) decisions are changed but outcomes don't improve. Each has a different fix.

**Remediation — trace the pipeline to find where value leaks**:

1. Check retrieval hit rate: if < 30%, insights aren't being found when needed
   - Fix: tune retrieval weights (recency vs importance vs relevance)
   - Fix: check embedding quality — run the embedding drift detector
2. Check decision reversal rate: if < 10%, insights are retrieved but ignored
   - Fix: increase the Grimoire's influence in the decision prompt
   - Fix: check if insights are placed in the "lost in the middle" zone of the context
3. Check outcome delta: if decisions change but outcomes don't improve, insights are actionable but _wrong_
   - Fix: this is a knowledge quality problem — tighten admission gates
   - Fix: increase the retroactive scoring penalty for insights that lead to losses

**Nuclear option**: If ROI is negative for 30+ days across multiple strategies, disable the Curator entirely and run with a static PLAYBOOK.md. This tests whether learning itself has value or is just burning compute.

**Verification**: Track Grimoire ROI over 7 days after the change — trend should be improving.

#### Pattern 5: Admission Rate Too High (>80%)

**Symptom**: Nearly everything gets into the Grimoire. Quality gate is not filtering.

**Root cause**: Threshold too low, or scoring dimensions aren't discriminating.

**Remediation**:

1. Raise the A-MAC composite threshold from 0.45 → 0.55
2. Add stricter red flag patterns (expand tautology regex, add hedge-counting)
3. Check if the novelty dimension is working — if all insights have novelty > 0.8, the baseline corpus is too small
4. Add a "minimum concrete referent count" hard gate: if an insight contains zero numbers, addresses, or time ranges, reject regardless of composite score

**Verification**: Admission rate should fall to 40-60% without losing high-impact insights (check retroactive scores of rejected entries).

#### Pattern 6: Admission Rate Too Low (<20%)

**Symptom**: Almost everything is rejected. The Golem learns nothing.

**Root cause**: Either the gate is too strict, or the model genuinely isn't producing quality.

**Remediation**:

1. Inspect rejected entries: are there false negatives (good insights wrongly rejected)?
   - If yes → lower threshold from 0.45 → 0.35, or reduce red flag sensitivity
   - If no → the model is the problem, not the gate
2. If the model isn't producing quality, check the Curator prompt — is it too constrained?
3. Temporarily disable semantic entropy detection (the most aggressive filter) and measure the impact
4. Check if the model is receiving enough context — a Curator running with 5 episodes can't produce quality; increase minimum to 30

**Verification**: Admission rate should rise to 40-60%. Monitor quality score distribution — if it drops, the gate was correctly strict.

#### Pattern 7: Contradictions Growing (>10 contradictory entry pairs)

**Symptom**: The Grimoire contains insights that give opposite advice. The retrieval system surfaces conflicting guidance.

**Root cause**: Regime-specific insights admitted without regime tags, or the consistency check isn't running.

**Remediation**:

1. Force regime tagging on every insight — if the Curator doesn't tag regime, the admission gate rejects
2. When contradictions are detected, run phages to test both sides — falsify one
3. Add a deduplication pass to the Curator cycle: before generating new insights, check for semantic similarity with existing entries (cosine > 0.9 = merge, don't duplicate)
4. If contradictions are between inherited and self-generated entries, bias toward self-generated (the Golem's own experience in current conditions beats ancestral knowledge)

**Verification**: Run `bardo eval:knowledge` and check contradiction count trending down.

### The "Is Knowledge Worth Having?" Decision

```typescript
interface KnowledgeValueAssessment {
  /** Total inference cost spent on knowledge generation (USDC). */
  generationCost: number;

  /** Total attributed positive impact from insights (USDC). */
  positiveImpact: number;

  /** Total attributed negative impact from wrong insights (USDC). */
  negativeImpact: number;

  /** Net ROI = (positiveImpact - negativeImpact - generationCost) / generationCost. */
  roi: number;

  /** How long has ROI been negative? */
  consecutiveNegativeDays: number;

  /** Recommendation. */
  verdict: "knowledge_valuable" | "knowledge_marginal" | "disable_learning";

  /**
   * Verdict thresholds:
   * - ROI > 0 for 7+ days: "knowledge_valuable"
   * - ROI between -0.5 and 0: "knowledge_marginal" — tune, don't disable
   * - ROI < -0.5 for 14+ days: "disable_learning" — static PLAYBOOK.md
   * - ROI < -1.0 for 7+ days: "disable_learning" — losing more than spending
   */
}
```

If the verdict is `"disable_learning"`, the Golem falls back to a static PLAYBOOK.md with no Curator cycles, no Grimoire updates, no dream consolidation. This is a valid configuration. If the static agent outperforms the learning agent, the learning pipeline is destroying value and needs fundamental redesign, not parameter tuning.

### Quality-Driven Simplification

When the quality pipeline itself is too expensive (too many LLM calls per insight), simplify the pipeline in this order, measuring quality impact at each step:

| Step | Remove                                                                     | Saves                     | Risk                            |
| ---- | -------------------------------------------------------------------------- | ------------------------- | ------------------------------- |
| 1    | Semantic entropy detection                                                 | 5 LLM calls per insight   | Vacuous insights slip through   |
| 2    | LLM-as-judge actionability scoring                                         | 1 LLM call per insight    | Descriptive insights admitted   |
| 3    | Simplify A-MAC from 7 factors to 3 (novelty + verifiability + consistency) | 2 LLM calls + computation | Broader admission criteria      |
| 4    | Replace counterfactual impact attribution with simple correlation          | 1 Mirage fork per insight | Less precise impact measurement |
| 5    | Replace LLM-based verifiability scoring with regex heuristics              | 1 LLM call per insight    | Miss nuanced verifiability      |

At each step, re-run the knowledge quality eval suite and compare:

- Did admission quality score distribution change significantly?
- Did Grimoire ROI change?
- Did the cost per admitted insight decrease proportionally?

If removing a step doesn't change quality but reduces cost, it was overhead. Keep it removed.

```typescript
interface QualityPipelineConfig {
  enableSemanticEntropy: boolean;
  enableLLMActionabilityScoring: boolean;
  amacFactorCount: 3 | 5 | 7;
  impactAttributionMethod: "counterfactual" | "correlation" | "none";
  verifiabilityMethod: "llm_judge" | "regex" | "none";

  /** Estimated cost per insight evaluation at this config level. */
  estimatedCostPerInsight: number;
}

const QUALITY_PIPELINE_LEVELS: QualityPipelineConfig[] = [
  {
    enableSemanticEntropy: true,
    enableLLMActionabilityScoring: true,
    amacFactorCount: 7,
    impactAttributionMethod: "counterfactual",
    verifiabilityMethod: "llm_judge",
    estimatedCostPerInsight: 0.003,
  },
  {
    enableSemanticEntropy: false,
    enableLLMActionabilityScoring: true,
    amacFactorCount: 7,
    impactAttributionMethod: "counterfactual",
    verifiabilityMethod: "llm_judge",
    estimatedCostPerInsight: 0.001,
  },
  {
    enableSemanticEntropy: false,
    enableLLMActionabilityScoring: false,
    amacFactorCount: 5,
    impactAttributionMethod: "correlation",
    verifiabilityMethod: "regex",
    estimatedCostPerInsight: 0.0002,
  },
  {
    enableSemanticEntropy: false,
    enableLLMActionabilityScoring: false,
    amacFactorCount: 3,
    impactAttributionMethod: "none",
    verifiabilityMethod: "none",
    estimatedCostPerInsight: 0.00005,
  },
];
```

### Prompt Engineering Remediation

When insight quality is low, the first fix is usually the Curator prompt, not the scoring pipeline. Concrete before/after examples:

**Before (produces generic insights)**:

```
Analyze the Golem's recent trading activity and generate insights
for the Grimoire.
```

**After (produces specific, actionable insights)**:

```
You are the Curator for a DeFi trading Golem on Base.

Review these recent episodes:
{{episodes}}

Current PLAYBOOK.md:
{{playbook}}

Generate ONE new insight that meets ALL of these criteria:
1. References a specific pool, token, time window, or threshold
2. Prescribes a concrete action with parameters (not "consider" or "might")
3. Is falsifiable — could be proven wrong by observing the market
4. Is NOT already in the PLAYBOOK.md above

Format: WHEN [condition], DO [action], BECAUSE [mechanism], EXPECT [outcome].

If no genuine insight emerges from these episodes, respond with
"NO_INSIGHT" — do not fabricate one.
```

The `NO_INSIGHT` escape hatch is critical. Without it, the model will always produce _something_, even from sparse data, and that something will be trivial or confabulated.

### CLI

```bash
# Diagnose knowledge quality issues from eval results
bardo eval:knowledge:diagnose \
  --grimoire ./data/grimoire.db \
  --results eval-results/knowledge-001/ \
  --report html

# Test the quality pipeline at a simpler configuration level
bardo eval:knowledge \
  --grimoire ./data/grimoire.db \
  --pipeline-level 2 \
  --report html

# Assess whether knowledge generation has positive ROI
bardo eval:knowledge:value \
  --golem-id golem-abc123 \
  --window 30d
```

---

## S9 — References

- [FARQUHAR-2024] Farquhar, S. et al. "Detecting hallucinations in large language models using semantic entropy." _Nature_ 630, 2024, pp. 625-630. doi:10.1038/s41586-024-07421-0. — *Introduces semantic entropy as a confabulation detector: if an LLM gives inconsistent explanations for the same claim, the claim is likely hallucinated. Directly implemented in S2.*
- [LIANG-2025] Liang, Y. et al. "Machine Bullshit." arXiv:2507.07484, 2025. — *Formalizes "bullshit" as statements produced without regard for truth; motivates the over-hedging detector and the "hedged to meaninglessness" red flag.*
- [SAHU-2024] Sahu, G. et al. "InsightBench: Evaluating Business Analytics Agents Through Multi-Step Insight Generation." arXiv:2407.06423, 2024. — *Benchmark for evaluating whether agent-generated insights are genuinely useful; informs the five-dimensional quality scoring rubric.*
- [ZHANG-AMAC] Zhang et al. "A-MAC: Automatic MArkup-based Consolidation for Memory in LLM Agents." — *Proposes the A-MAC admission control framework (future utility, factual confidence, novelty, recency, content prior) extended in S4 with market impact and regime relevance.*
- [HUANG-2024] Huang, J. et al. "Large Language Models Cannot Self-Correct Reasoning Yet." ICLR 2024. arXiv:2310.01798. — *Demonstrates that intrinsic self-correction degrades LLM performance; constrains what the quality evaluator can trust from LLM-as-judge scoring.*
- [KAMOI-2024] Kamoi, R. et al. "When Can LLMs Actually Correct Their Own Mistakes?" _TACL_ 12, 2024. arXiv:2406.01297. — *Identifies the narrow conditions where LLM self-correction works; supports using external signals (on-chain data, embedding distance) rather than LLM self-assessment for quality scoring.*
- [PAN-2024] Pan, A. et al. "Feedback Loops With Language Models Drive In-Context Reward Hacking." ICML 2024. arXiv:2402.06627. — *Shows that LLM feedback loops produce reward hacking; motivates the hard-coded red flag detectors (tautology, unfalsifiable) that bypass LLM judgment entirely.*
- [CHMURA-2023] Chmura, J. et al. "Information Content Exploration." arXiv:2310.06777, 2023. — *Measures information content as exploration quality; supports using novelty score (embedding distance from existing knowledge) as a quality dimension.*
- [WANG-IGPO-2025] Wang et al. "Information Gain-based Policy Optimization." arXiv:2510.14967, 2025. — *Uses information gain as intrinsic reward; theoretical backing for weighting the novelty dimension at 0.25 in the quality composite.*
