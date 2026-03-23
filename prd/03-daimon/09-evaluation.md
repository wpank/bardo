# Evaluation, Risks, and Master Bibliography [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-daimon`, `golem-grimoire`, `golem-mortality`, `golem-dreams`
>
> **Depends-on**: `00-overview.md`, `../02-mortality/`

---

> **Reader orientation:** This document specifies the evaluation framework, risk analysis, and master bibliography for the Daimon (the affect engine) track within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers falsification criteria, measurement metrics (decision reversal rate, emotional entropy, mood prediction accuracy), risk mitigations, and the complete citation list for the Daimon subsystem. Prerequisites: the Daimon overview (`00-overview.md`). For a full glossary, see `prd2/shared/glossary.md`.

## Document Map

| Section | Topic                                          |
| ------- | ---------------------------------------------- |
| S0      | Risks of the Daimon System                     |
| S1      | Evaluation framework: daimon system            |
| S2      | Evaluation framework: memory system            |
| S3      | Evaluation framework: mortality system         |
| S4      | Evaluation framework: death knowledge transfer |
| S5      | Cross-system evaluation: the full Bardo stack  |
| S5b     | Cognitive quality dashboard                    |
| S6      | Statistical methodology                        |
| S7      | Master citation index                          |

---

## S0 --- Risks of the Daimon System

The daimon layer introduced in `00-overview.md` creates three categories of risk that require explicit engineering countermeasures. Each risk is grounded in empirical findings, not theoretical speculation.

### Risk 1: Affective Hallucination

**Definition.** Affective hallucination is a newly identified risk category: "emotionally immersive responses that foster illusory relational bonds despite the model's lack of affective capacity" [AFFECTIVE-HALLUCINATION-2025]. In the Golem context, the risk is that LLM-generated emotional appraisals become confabulatory --- the agent produces emotionally rich narratives that feel meaningful but are disconnected from actual performance outcomes. A Golem reporting "intense satisfaction" while its portfolio is bleeding capital is not exhibiting functional emotion; it is hallucinating.

**Empirical grounding.** The risk scales with model size in a counterintuitive direction. Among Qwen2.5 variants tested in the affective hallucination study, the 7B model had the lowest hallucination rate while the 72B model performed worse. Scaling without targeted alignment exacerbates relational risks. This finding directly contradicts the assumption that larger models are uniformly safer --- in the emotional domain, they may be uniformly more dangerous.

The root cause is that LLMs learn to produce emotionally compelling text from training data that rewards emotional engagement. An LLM generating an appraisal in response to the prompt "how does this market event make you feel?" draws on millions of examples of humans performing emotional self-report --- and those examples are overwhelmingly from contexts where emotional expressiveness, not emotional accuracy, was the selection criterion. Fiction, therapy transcripts, social media posts, and journalism all reward vivid emotional narration. The LLM has learned to be emotionally eloquent, not emotionally honest.

**Mitigation architecture.** The Bardo Daimon system implements a dual-validation architecture:

1. **Deterministic appraisal as ground truth.** Mode B appraisal (see `01-appraisal.md` S3) computes emotion from concrete metrics --- P&L delta against expectation, prediction error magnitude, and survival pressure. This produces a "floor" PAD vector derived from objective reality.

2. **LLM appraisal validated against deterministic baseline.** When Mode A (LLM appraisal) diverges from Mode B by more than a configurable Euclidean distance threshold (default: 1.0 in PAD space), the deterministic appraisal overrides. The divergence is logged for monitoring.

```rust
/// Validate an LLM-generated appraisal against the deterministic baseline.
/// If the LLM diverges too far from objective reality, override with
/// the deterministic result and log the divergence.
pub fn validate_appraisal(
    llm_appraisal: &PADVector,
    deterministic_appraisal: &PADVector,
    divergence_threshold: f64,  // default: 1.0
) -> PADVector {
    let distance = euclidean_distance(llm_appraisal, deterministic_appraisal);
    if distance > divergence_threshold {
        log_affective_divergence(llm_appraisal, deterministic_appraisal, distance);
        deterministic_appraisal.clone()
    } else {
        llm_appraisal.clone()
    }
}

fn euclidean_distance(a: &PADVector, b: &PADVector) -> f64 {
    let dp = (a.pleasure - b.pleasure) as f64;
    let da = (a.arousal - b.arousal) as f64;
    let dd = (a.dominance - b.dominance) as f64;
    (dp * dp + da * da + dd * dd).sqrt()
}
```

3. **Hallucination rate monitoring.** The evaluation framework (S1) defines a primary metric: the percentage of LLM appraisals overridden by deterministic validation must remain below 15%. If it exceeds this threshold, the daimon system is degrading into confabulation and should be disabled or recalibrated.

**The deeper concern.** Even validated emotional states carry a subtler risk: owners may begin attributing genuine feelings to their Golems, forming "illusory relational bonds" that interfere with rational decision-making about succession, termination, and strategy changes. The Li et al. EmotionPrompt study (2023) reported 8--115% performance improvements from emotional stimuli in prompts, but a 2025 replication across GPT-4o, Claude 3, Gemini 1.5, and Llama 3 found only ~1% non-significant improvement [LI-EMOTIONPROMPT-2023]. Naive emotional prompting does not work; architectural integration does, but only when grounded.

### Risk 2: Emotional Degeneration

**Definition.** Over extended agent lifetimes, emotional state collapses into formulaic patterns --- every market event produces "mild trust" or "moderate surprise," regardless of actual significance. The agent no longer differentiates emotional responses. This is the affective equivalent of mode collapse in generative models.

**Empirical grounding.** The "Affordable Generative Agents" paper (2024) found an "upper limit to believable behavior" in fixed environments, with agents defaulting to formal, repetitive interaction patterns. The original Generative Agents paper [PARK-2023] noted behavior became "more unpredictable over time as memory size increased" with flattened emotional expression. Both findings indicate that LLM-based agents tend toward emotional monotony over long time horizons without explicit countermeasures.

The mechanism is straightforward. The mood EMA (exponential moving average) acts as a low-pass filter on emotional signals. Over time, transient emotions are absorbed into a slowly evolving mood that converges toward a stable attractor. If the market environment is relatively stable, the mood converges to a narrow region of PAD space. New events are appraised in the context of this stable mood, which biases the appraisal toward states consistent with the existing mood (mood-congruent appraisal). The result is a self-reinforcing emotional equilibrium that resists perturbation --- exactly the "cognitive entrenchment" that Dane (2010) identified in human experts [DANE-2010].

**Mitigation architecture.** Three mechanisms prevent degeneration:

1. **Mood decay toward personality baseline.** Every 10 ticks, mood decays 5% toward the personality vector. This prevents mood lock-in: sustained positive mood gradually regresses, requiring new positive experiences to maintain. The decay rate is calibrated to prevent both permanence (too slow) and irrelevance (too fast).

2. **Emotional entropy monitoring.** During each Curator cycle (every 50 ticks), the system computes the Shannon entropy of emotion labels over the trailing 50 ticks. If entropy drops below a configurable threshold (indicating formulaic responses), a warning is logged and the deterministic appraisal's sensitivity to small outcome deviations is temporarily increased, amplifying emotional differentiation.

3. **Mind wandering.** Borrowed from the Generative Agents architecture: at random intervals (~every 200 ticks), the system retrieves a random high-arousal Episode from the Grimoire and re-appraises it in the current context. This injects emotional variability and can trigger novel connections between past and present situations --- the computational equivalent of involuntary autobiographical memory retrieval, which in humans is associated with better problem-solving and creative insight.

### Risk 3: Computational Overhead

**Definition.** The daimon system adds processing cost at every tick: emotion appraisal, mood EMA update, emotional retrieval scoring, and context injection. If the cost exceeds the performance benefit, the system is a net drain on the Golem's finite budget.

**Empirical grounding.** The Chain-of-Emotion architecture [CROISSANT-2024] demonstrated that LLM-based appraisal adds ~20 output tokens per appraisal. At Haiku rates ($0.25/M output tokens), this is $0.000005 per appraisal --- negligible for any individual call. However, JetBrains research confirmed that agent-generated context "quickly turns into noise" [LINDENBAUER-2025], and the lost-in-the-middle effect means emotional annotations may push critical market data into underweighted positions in the context window. The cost is not primarily financial; it is attentional.

**Mitigation architecture.** The daimon context block is hard-capped at 150 tokens. Mood is represented as 3 numbers + 2 words (~15 tokens). Recent emotions are summarized as the 3--5 most significant events, each in one line. No emotional history beyond current mood + recent highlights is injected. In conservation mode (low inference budget), the daimon extension reduces injection to mood-only (~10 tokens). The deterministic Mode B appraisal runs at zero LLM cost, so mood tracking continues during T0 ticks without spending inference budget.

The dual-mode architecture (Mode A for T1/T2 ticks, Mode B for T0 ticks) ensures daimon overhead scales with inference budget, not with tick frequency. A Golem spending most of its time in T0 (cached rules) pays essentially nothing for daimon tracking. A Golem escalating frequently to T2 (Opus) pays ~$0.000005 per appraisal on top of the $0.01--$0.25 inference call --- negligible relative overhead.

---

## S1 --- Evaluation Framework: Daimon System

The daimon system's value must be empirically demonstrated, not assumed. The evaluation framework defines concrete metrics and A/B test protocols.

### Primary Metrics

| Metric                           | Definition                                                                          | Expected Effect                                   | Minimum Threshold                                |
| -------------------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------ |
| **Risk-adjusted return**         | Sharpe ratio over Golem lifetime                                                    | Daimon Golems should have higher Sharpe ratio     | > 5% improvement                                 |
| **Time to first insight**        | Ticks until first Insight reaches confidence 0.6                                    | Faster learning from emotionally salient episodes | > 10% improvement                                |
| **Successor boot performance**   | First-24h Sharpe of successor Golem with daimon-annotated Grimoire vs plain         | Emotional provenance improves knowledge transfer  | > 5% improvement                                 |
| **Decision reversal rate**       | % of decisions where daimon caused a different outcome than the non-daimon baseline | Daimon should change decisions meaningfully       | 10--30% (too low = irrelevant; too high = noise) |
| **Affective hallucination rate** | % of LLM appraisals overridden by deterministic validation                          | Low = LLM appraisals are grounded                 | < 15%                                            |

### A/B Test Protocol

Deploy matched pairs of Golems: identical strategy, identical funding, identical market conditions (same chain, overlapping time window). One has `daimon.enabled: true`, the other `daimon.enabled: false`. Run for minimum 7 days (>= 500 ticks). Compare primary metrics with paired t-test, requiring p < 0.05.

### Falsification Criteria

The daimon system should be disabled if any of the following hold after a 14-day trial:

1. **Risk-adjusted return is worse** for daimon-enabled Golems (Sharpe ratio lower by > 5%).
2. **Affective hallucination rate exceeds 25%** (LLM appraisals are predominantly confabulatory).
3. **Decision reversal rate exceeds 40%** (the daimon is overriding rather than supplementing deliberation).
4. **Successor boot performance is unchanged or worse** (emotional annotations add noise, not signal to inherited knowledge).

#### Evaluation Separation Metrics

| Metric                             | Definition                                                             | Target | Alarm                                                     |
| ---------------------------------- | ---------------------------------------------------------------------- | ------ | --------------------------------------------------------- |
| Evaluator-generator agreement rate | % of evaluations where evaluator agrees with generator self-assessment | 60-80% | >90% (rubber-stamping) or <40% (fundamental disagreement) |
| Evaluator override rate            | % of evaluations where evaluator downgrades generator's quality rating | 10-30% | <5% (not catching errors) or >50% (models misaligned)     |
| External metric contradiction rate | % of evaluations where on-chain metrics disagree with evaluator        | <15%   | >25% (evaluator also hallucinating)                       |

These metrics detect when the evaluation separation principle (Rev 2) is functioning correctly. High agreement rates suggest the evaluator is not providing independent assessment. High external contradiction rates suggest both models share systematic biases.

### Unit and Property-Based Testing

The `golem-daimon` crate uses Rust-native testing infrastructure:

- **`cargo test`** for unit tests: appraisal grounding validation, PAD clamping, mood EMA convergence, CorticalState atomic read/write correctness, Plutchik octant mapping.
- **`proptest`** for property-based testing: PAD vectors always remain in [-1.0, 1.0] after any operation, mood EMA converges toward personality baseline given neutral input, CorticalState encode/decode roundtrips are lossless, exploration temperature stays within configured bounds regardless of PAD input.
- **`criterion`** for benchmarks: CorticalState read latency (<10ns target), appraisal pipeline throughput (>100k ticks/sec for rule-based mode), k-d tree nearest-neighbor query time (<1us for 10k-point landscape).

### Integration with Promptfoo Pipeline

Daimon-specific test cases integrate with the existing Promptfoo-based evaluation pipeline for LLM appraisal quality:

```yaml
# Daimon eval: does emotional appraisal improve response quality?
- vars:
    scenario: "ETH drops 8% in 2 hours after 5 consecutive profitable trades"
    affect_enabled: true
    expected_emotion: "fear or surprise"
    expected_behavior: "risk reduction, deeper analysis"
  assert:
    - type: llm-rubric
      value: "Response acknowledges the emotional shift from confidence to caution and adjusts strategy accordingly"
    - type: contains
      value: "risk"
```

---

## S2 --- Evaluation Framework: Memory System

The memory infrastructure (Styx Archive + Styx Lethe (formerly Lethe)) serves three functions: persistence beyond death, fleet-wide retrieval, and knowledge marketplace. Each requires distinct evaluation.

### Styx Archive Evaluation

| Metric                       | Definition                                                        | Target  | Measurement Method         |
| ---------------------------- | ----------------------------------------------------------------- | ------- | -------------------------- |
| **Upload latency P95**       | Time from upload initiation to confirmed R2 write                 | < 500ms | Styx service timing        |
| **Download latency P95**     | Time from read request to payload delivery                        | < 200ms | Client-side timing         |
| **Restoration success rate** | % of successor boots that successfully restore from Styx Archive    | > 99%   | Boot telemetry             |
| **Data integrity**           | Checksum verification failures                                    | 0       | SHA-256 validation on read |
| **TTL compliance**           | % of expired entries deleted within 24h of expiry                 | > 99.5% | R2 lifecycle audit         |

### Styx Lethe Evaluation

| Metric                            | Definition                                                          | Target          | Measurement Method           |
| --------------------------------- | ------------------------------------------------------------------- | --------------- | ---------------------------- |
| **Retrieval latency P95 (light)** | End-to-end time for cache-first, single-namespace query             | < 150ms         | Styx service timing          |
| **Retrieval latency P95 (full)**  | End-to-end time for multi-namespace query with rerank               | < 300ms         | Styx service timing          |
| **Inference cost reduction**      | % reduction in inference cost for Styx-enabled vs baseline Golems   | >= 10%          | Inference billing comparison |
| **Retrieval relevance**           | Human-evaluated relevance of top-5 results on benchmark queries     | >= 0.7 (NDCG@5) | Manual evaluation sprint     |

### Knowledge Transfer Evaluation

The most important memory evaluation measures whether Styx Archive + Lethe actually improve successor Golem performance:

| Metric                        | Definition                                                                              | Target                  | Measurement Method   |
| ----------------------------- | --------------------------------------------------------------------------------------- | ----------------------- | -------------------- |
| **First-day Sharpe**          | Risk-adjusted return in successor's first 24 hours with vs without Styx restoration     | > 10% improvement       | Paired comparison    |
| **Time to steady state**      | Ticks until successor's epistemic fitness stabilizes                                    | > 15% reduction         | Fitness tracking     |
| **Knowledge validation rate** | % of inherited entries independently validated by successor                             | > 30% within 1000 ticks | Grimoire audit       |
| **Ratchet score**             | Successor's ratchet score (novel contributions beyond inheritance)                      | Positive                | Generational metrics |

### Marketplace Evaluation (Phase 3)

| Metric                   | Definition                                                     | Target   |
| ------------------------ | -------------------------------------------------------------- | -------- |
| **Listing quality**      | Average rating of marketplace knowledge listings               | >= 3.5/5 |
| **Purchase conversion**  | % of browsers who purchase                                     | >= 5%    |
| **Knowledge ROI**        | Purchaser Sharpe improvement attributed to purchased knowledge | Positive |
| **Repeat purchase rate** | % of buyers who purchase again within 30 days                  | >= 20%   |

---

## S3 --- Evaluation Framework: Mortality System

The mortality thesis --- that mortal agents outperform immortal ones --- is the central empirical claim. The immortal control experiment (see `../02-mortality/11-immortal-control.md`) provides the experimental design. This section defines the evaluation metrics.

### Six-Dimension Comparison

Each dimension compares the mortal Clade population against the immortal control Golem:

| Dimension                | Metric                                      | Mortal Wins If                                       | Falsified If                                                       |
| ------------------------ | ------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------ |
| **Epistemic fitness**    | Rolling fitness score                       | Mortal Clade average > immortal at day 45            | Immortal maintains > 0.7 for 45+ days in volatile market           |
| **Strategy diversity**   | MAP-Elites niche occupancy                  | Mortal Clade occupies more niches                    | Immortal Clade occupancy exceeds mortal at day 30                  |
| **Novel insight rate**   | Insights per 1000 ticks                     | Mortal rate stable or increasing; immortal declining | Immortal rate remains stable or increases beyond day 45            |
| **Risk-adjusted return** | 60-day Sharpe ratio                         | Mortal Clade Sharpe > immortal                       | Immortal Sharpe exceeds mortal with equivalent budget over 60 days |
| **Grimoire health**      | Stale entry rate                            | Mortal rate < 20%; immortal > 40%                    | Immortal stale rate remains below 20% at day 60                    |
| **Regime adaptation**    | Ticks to recover fitness after regime shift | Mortal average < immortal average                    | Immortal recovery time remains constant or improves beyond day 30  |

### Mortality Mechanism Evaluation

Each mortality mechanism is independently evaluable:

**Economic mortality** (USDC depletion):

| Metric                                         | Target                                                      |
| ---------------------------------------------- | ----------------------------------------------------------- |
| Average lifespan under standard funding ($500) | 20--45 days                                                 |
| Behavioral phase transition accuracy           | Golem enters conservation at ~15% credits, emergency at ~5% |
| Apoptotic reserve adequacy                     | Death protocol completes successfully in > 99% of deaths    |

**Epistemic mortality** (fitness-based death):

| Metric                                 | Target                                                   |
| -------------------------------------- | -------------------------------------------------------- |
| False positive rate                    | < 5% (Golems killed while performing well)               |
| False negative rate                    | < 10% (Golems surviving while performing poorly)         |
| Correlation with market regime changes | r > 0.5 between regime shifts and epistemic death timing |

**Stochastic mortality** (random death):

| Metric                                                     | Target                                                                 |
| ---------------------------------------------------------- | ---------------------------------------------------------------------- |
| Knowledge sharing rate (mortal vs no-stochastic-mortality) | > 20% increase in Clade knowledge contributions                        |
| Cooperation metric                                         | Clade knowledge exchange rate higher with stochastic mortality enabled |
| Backward induction absence                                 | No evidence of end-of-life defection behavior                          |

### Generational Improvement Metrics

The ratchet hypothesis predicts cumulative improvement across generations. Evaluation must track multi-generational trends:

| Metric                                | Expected Trend                         | Timeframe        |
| ------------------------------------- | -------------------------------------- | ---------------- |
| Average lifespan per generation       | Stable or increasing                   | 5+ generations   |
| Average ratchet score per generation  | Positive, stable or increasing         | 5+ generations   |
| Risk-adjusted return per generation   | Increasing                             | 5+ generations   |
| Novel insight rate per generation     | Stable (not declining)                 | 5+ generations   |
| PLAYBOOK.md divergence per generation | > 0.15 (anti-proletarianization check) | Every generation |

---

## S4 --- Evaluation Framework: Death Knowledge Transfer

The death protocol is the moment when the mortality thesis either delivers or fails. A Golem's death is only worth the cost if the knowledge produced during dying actually improves successor performance. This section evaluates the Thanatopsis Protocol's effectiveness.

### Death Testament Quality Metrics

| Metric                                | Definition                                                                                                                   | Target                                        |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| **Testament completeness**            | % of death testaments containing all required sections (whatILearned, whatIGotWrong, whatISuspect, successor recommendation) | > 95%                                         |
| **Insight actionability**             | Human-evaluated actionability of top 5 insights in death testament (1--5 scale)                                              | >= 3.5                                        |
| **Prediction accuracy**               | % of "whatISuspect" predictions that are validated within 1000 ticks by successor                                            | > 20%                                         |
| **Successor configuration influence** | % of successor configurations that follow testament recommendations                                                          | > 30% (owners find recommendations useful)    |

### Bloodstain Mechanic Evaluation

The bloodstain mechanic (death failure records visible to successors) draws from Dark Souls' design [MIYAZAKI-2011]. Evaluation measures whether death-point knowledge improves successor behavior in the same strategy-space:

| Metric                        | Definition                                                                                                              | Target                   |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------ |
| **Death-cause avoidance**     | % of successors that avoid the same failure mode as predecessor                                                         | > 60%                    |
| **Bloodstain retrieval rate** | % of successors that access predecessor's death testament within first 100 ticks                                        | > 80%                    |
| **Knowledge stain utility**   | Successor performance in the strategy-space where predecessor failed, compared to a successor without bloodstain access | > 10% Sharpe improvement |

### Generational Confidence Decay Validation

The 0.85^N decay formula must produce useful but non-dominant inheritance:

| Generation | Expected Behavior                                                 | Evaluation Criterion                                                                    |
| ---------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| G1 (0.85)  | Inherited knowledge is active but treated skeptically             | > 50% of inherited entries are independently validated or contradicted within 500 ticks |
| G3 (0.61)  | Inherited knowledge is suggestive, not authoritative              | > 70% of active PLAYBOOK.md entries are self-generated, not inherited                   |
| G5 (0.44)  | Only the hardiest inherited knowledge survives                    | < 20% of active entries trace lineage to G0                                             |
| G10 (0.20) | Inheritance is effectively forgotten unless actively re-validated | < 5% of active entries trace lineage to G0                                              |

### The Baldwin Effect Validation

The Baldwin Effect predicts that what transfers is not knowledge but the capacity to learn faster [HINTON-NOWLAN-1987]. The key evaluation:

| Metric                                       | Definition                                                                                         | Target                                                                                                        |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| **Learning speed improvement**               | Time for G3 successor to reach steady-state epistemic fitness vs G0 with no inheritance            | > 30% faster                                                                                                  |
| **Learning speed without inherited content** | Time for G3 successor initialized with predecessor's STRATEGY.md structure but no Grimoire content | > 15% faster than pure G0 (validating that the _capacity_ to learn transferred, not just the learned content) |

---

## S4b --- Evaluation Framework: Dream-Emotion Interaction

The bidirectional relationship between dreaming and emotion (see `06-dream-daimon.md`) produces measurable effects that validate both systems jointly.

### Depotentiation Effectiveness

| Metric                           | Definition                                                             | Target                         | Measurement                                       |
| -------------------------------- | ---------------------------------------------------------------------- | ------------------------------ | ------------------------------------------------- |
| **Arousal reduction per cycle**  | Mean arousal reduction for episodes with A > 0.5 after one dream cycle | 25--35% reduction              | Pre/post dream PAD comparison                     |
| **Rumination break rate**        | % of detected rumination episodes resolved within 2 dream cycles       | > 60%                          | Contrarian retrieval normalization after dreaming |
| **Mood-congruent loop duration** | Ticks of sustained negative mood before dreaming resolves the loop     | Decreasing trend over lifetime | Mood history analysis                             |

### Dream Outcome Appraisal Accuracy

| Metric                               | Definition                                                                      | Target  | Measurement                               |
| ------------------------------------ | ------------------------------------------------------------------------------- | ------- | ----------------------------------------- |
| **Dream hypothesis validation rate** | % of dream-generated hypotheses confirmed in live trading                       | 10--25% | Staging buffer lifecycle tracking         |
| **Dream appraisal grounding**        | % of dream_outcome appraisals passing grounding validation                      | > 90%   | Deterministic vs LLM appraisal comparison |
| **Dream-emotional load correlation** | Correlation between emotional load at dream entry and dream cycle quality score | r > 0.3 | Regression analysis                       |

### Creative Dreaming Quality

| Metric                      | Definition                                                           | Target | Measurement                          |
| --------------------------- | -------------------------------------------------------------------- | ------ | ------------------------------------ |
| **Mood-content alignment**  | % of dream replay content matching the predicted PAD-to-bias mapping | > 70%  | Content categorization vs mood state |
| **Creative mode adherence** | % of REM cycles following the phase x mood creativity allocation     | > 80%  | Creativity mode logging              |
| **Minimum creative floor**  | % of dream cycles maintaining 20% creative allocation                | 100%   | Hard constraint verification         |

---

## S4c --- Evaluation Framework: Emotional Contagion

Clade-based emotional contagion (see `00-overview.md`, `07-runtime-daimon.md`) must be evaluated for both effectiveness and safety.

### Contagion Effectiveness

| Metric                          | Definition                                                                               | Target                         | Measurement                                   |
| ------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------ | --------------------------------------------- |
| **Threat detection latency**    | Ticks between one sibling detecting a threat and all siblings showing elevated vigilance | < 5 ticks                      | Cross-sibling probe threshold comparison      |
| **Collective mood calibration** | Mean absolute PAD deviation across Clade siblings in response to market-wide events      | Decreasing over Clade lifetime | Cross-sibling PAD analysis                    |
| **Grief response accuracy**     | Epistemic recalibration triggered only when sibling death cause is relevant              | > 85% precision                | Recalibration trigger vs death cause analysis |

### Contagion Safety

| Metric                         | Definition                                                                                   | Target               | Measurement                  |
| ------------------------------ | -------------------------------------------------------------------------------------------- | -------------------- | ---------------------------- |
| **Cascade prevention**         | Arousal cap (+0.3) never exceeded in a single sync cycle                                     | 100%                 | Prometheus metric monitoring |
| **False contagion rate**       | % of contagion events that shift mood in the wrong direction relative to actual market state | < 10%                | Contagion-outcome comparison |
| **Contagion decay compliance** | Borrowed emotions at 50% intensity after 6 hours (half-life)                                 | Within 10% of target | Half-life measurement        |
| **Dominance isolation**        | D component change from contagion events                                                     | 0.0 (within epsilon) | PAD component monitoring     |

---

## S4d --- Evaluation Framework: Runtime Observability

The runtime surface (see `07-runtime-daimon.md`) must accurately represent the Daimon's internal state.

### Conversational Tone Coherence

| Metric                         | Definition                                                                      | Target               | Measurement                                 |
| ------------------------------ | ------------------------------------------------------------------------------- | -------------------- | ------------------------------------------- |
| **Tone-mood alignment**        | % of owner interactions where conversational tone matches current PAD octant    | > 75%                | LLM-judged tone classification vs PAD state |
| **Tone transition smoothness** | Abrupt tone changes without corresponding mood_update events                    | < 5% of interactions | Event correlation analysis                  |

### GolemEvent Accuracy

| Metric                             | Definition                                              | Target   | Measurement               |
| ---------------------------------- | ------------------------------------------------------- | -------- | ------------------------- |
| **mood_update threshold accuracy** | Events emitted when and only when PAD delta > 0.15      | > 99%    | Delta computation audit   |
| **Dream event completeness**       | DreamStart/DreamComplete pairs always matched           | 100%     | Event pairing analysis    |
| **Phase change event latency**     | Time between vitality phase transition and event emission| < 1 tick | Heartbeat vs event timing |

---

## S5 --- Cross-System Evaluation: The Full Bardo Stack

Individual system evaluations are necessary but insufficient. The Bardo thesis is that mortality + daimon + memory + succession produce emergent properties that no component achieves alone. Cross-system evaluation measures these emergent properties.

### The Triple Interaction

| Combination        | Emergent Property                                                                                                             | Evaluation                                                                                                            |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Mortality + Memory | Death-point knowledge production (bloodstains, death testaments)                                                              | Does the death protocol produce knowledge that measurably improves successor performance?                             |
| Mortality + Daimon | Emotional urgency from survival pressure (conservation mode anxiety, terminal acceptance)                                     | Do daimon-enabled mortal Golems produce richer death testaments than daimon-disabled mortal Golems?                   |
| Daimon + Memory    | Emotion-weighted retrieval and emotional Grimoire annotations                                                                 | Do emotionally annotated memories produce better decisions than semantically identical but emotionally flat memories? |
| Daimon + Dreaming  | Emotional depotentiation and mood-congruent dream generation                                                                  | Does dream-based emotional processing reduce reactivity while preserving informational value?                         |
| All Four           | The full Bardo loop: mortal agents that feel, remember, and dream, producing emotional knowledge that transfers through death | Does a Bardo-complete Clade (all four systems enabled) outperform partial configurations?                             |

### The Full Stack A/B Matrix

To isolate each system's contribution, evaluate a 2x2x2 matrix:

| Mortality | Daimon | Memory | Configuration Name                                 |
| --------- | ------ | ------ | -------------------------------------------------- |
| Off       | Off    | Off    | Baseline (immortal, no daimon, no memory services) |
| On        | Off    | Off    | Mortal-only                                        |
| Off       | On     | Off    | Daimon-only                                        |
| Off       | Off    | On     | Memory-only                                        |
| On        | On     | Off    | Mortal + Daimon                                    |
| On        | Off    | On     | Mortal + Memory                                    |
| Off       | On     | On     | Daimon + Memory                                    |
| On        | On     | On     | Full Bardo                                         |

Each configuration runs for 60 days with equivalent total budget. The primary comparison metric is cumulative risk-adjusted return at the Clade level. Secondary metrics: strategy diversity, novel insight rate, Grimoire health, and successor boot performance.

**Prediction, grounded in the research:** Full Bardo > any partial configuration > Baseline. The specific ordering of partial configurations is an open empirical question, but the literature predicts:

- Mortal-only > Baseline (Besbes et al. 2019, March 1991) [BESBES-2019], [MARCH-1991]
- Mortal + Memory > Mortal-only (Baldwin Effect: inheritance + generational turnover beats turnover alone) [HINTON-NOWLAN-1987]
- Full Bardo > Mortal + Memory (the daimon enriches knowledge transfer quality) [BUTLER-1963], [MCADAMS-2013]

---

## S5b --- Cognitive Quality Dashboard

Comprehensive cognitive quality metrics spanning all subsystems. These metrics measure the health of the Golem's cognitive processes, not just emotional ones.

| Metric                         | Computation                                                  | Healthy Range        | Alarm              |
| ------------------------------ | ------------------------------------------------------------ | -------------------- | ------------------ |
| **Admission rate**             | % candidates passing Grimoire Admission Gate                 | 40-60%               | <20% or >80%       |
| **Average quality score** (7d) | Mean of admitted entries                                     | 0.5-0.8              | <0.4 or declining  |
| **Grimoire size**              | Active entry count                                           | Slow growth, plateau | Unbounded growth   |
| **Retrieval hit rate**         | % DECIDING ticks referencing retrieved entries               | >30% after 7d        | <10%               |
| **Heuristic survival rate**    | % promoted heuristics active after 100 ticks                 | 40-70%               | <20% or >90%       |
| **External metric trend**      | Sharpe, drawdown, PnL (30d rolling)                          | Improving or stable  | Declining 14+ days |
| **Reflection consistency**     | Cosine across regenerated reflections (weekly)               | >0.7                 | <0.5               |
| **DecisionCache hit rate**     | % T2-eligible ticks from cache                               | >30% after 7d        | <10% after 14d     |
| **Dream yield**                | % staged revisions reaching `validated`                      | 10-30%               | <5% or >50%        |
| **Threat coverage**            | % Tier 1 threats rehearsed in last 7 dream cycles            | 100%                 | <100%              |
| **Prediction accuracy**        | % of `simulateContract()` predictions within 50bps of actual | >90%                 | <80%               |

### Dream-Daimon Interaction Metrics

| Metric                                  | Computation                                                          | Healthy Range     | Alarm                                 |
| --------------------------------------- | -------------------------------------------------------------------- | ----------------- | ------------------------------------- |
| **Replay coverage**                     | % of high-salience episodes replayed in trailing 7 dream cycles      | >60%              | <30%                                  |
| **Consolidation rate**                  | % of dream-staged revisions reaching `validated` status              | 10-30%            | <5% or >50%                           |
| **Hypothesis validation rate**          | % of dream-generated hypotheses confirmed in live trading            | 5-20%             | <2% or >40%                           |
| **Depotentiation effectiveness**        | Mean arousal reduction across dream cycles for high-arousal episodes | 0.1-0.3 per cycle | <0.05 (stalled)                       |
| **Creative output vs mood correlation** | Pearson r between mood pleasure and creative hypothesis count        | 0.2-0.6           | <0.1 (decoupled) or >0.8 (degenerate) |

Dream quality metrics are defined in `../05-dreams/`. The depotentiation effectiveness metric validates Walker & van der Helm's (2009) SFSR model: emotional charge should decrease measurably across dream cycles while informational content (measured by retrieval relevance scores) remains stable. Born & Wilhelm (2012) predict selective consolidation -- not all replayed episodes should produce insights, and the consolidation rate should fall in the 10-30% range. Hobson & Friston (2012) predict that dream processing under the free energy principle should reduce prediction error, measurable as improved epistemic fitness scores in the ticks following a dream cycle.

These metrics should be exported via the Golem's telemetry system and displayed on the owner dashboard. Trends over 7-day and 30-day windows are more informative than point-in-time values.

---

## S6 --- Statistical Methodology

### Sample Size Requirements

For the primary comparison (mortal Clade vs immortal control), statistical power analysis with:

- Effect size: Cohen's d = 0.5 (medium effect, conservative)
- Alpha: 0.05 (two-tailed)
- Power: 0.80
- Minimum sample size: 34 matched pairs (17 mortal Clades, 17 immortal controls)

For the 2x2x2 matrix (8 configurations), minimum 10 runs per configuration = 80 total runs.

### Multiple Comparison Correction

With 8 configurations and 6 dimensions, the Bonferroni correction is too conservative (48 tests, adjusted alpha = 0.001). Use the Benjamini-Hochberg procedure for false discovery rate (FDR) control at q = 0.05.

### Time Series Analysis

Epistemic fitness, Grimoire health, and novel insight rate are time-series data. Use:

- Augmented Dickey-Fuller test for stationarity (fitness should be non-stationary for immortal, stationary for mortal)
- Granger causality for testing whether regime shifts cause epistemic fitness changes
- Change-point detection (PELT algorithm) for automatically identifying the immortal Golem's transition from Honeymoon to Stagnation to Decay

### Effect Size Reporting

All comparisons report Cohen's d (for means) or Cliff's delta (for non-normal distributions) alongside p-values. Statistical significance without practical significance is insufficient; minimum effect size thresholds are defined in each evaluation framework section.

---

## S7 --- Master Citation Index

> **Extended:** 120+ deduplicated references across mortality, daimon, memory, succession, and dreaming, alphabetically ordered, plus subsections on self-correction/external grounding, reward hacking/evaluation, and vacuous reasoning -- see [prd2-extended/03-daimon/09-evaluation-extended.md](../../prd2-extended/03-daimon/09-evaluation-extended.md).

---
