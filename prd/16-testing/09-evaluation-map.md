# The Complete Evaluation Map [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: All previous testing documents. This is the capstone showing how evaluation systems compose.
>
> **Source**: `active-inference-research/new/17-evaluation-map.md`

> **Reader orientation:** This is the capstone document for Section 16 (Testing), showing how all 14 feedback loops compose across five speed tiers. It provides a single view of what evaluates what, at what speed, and how the loops feed each other -- from machine-speed residual correction (~15,000/day) through weekly meta-learning evaluation. If you are reading one testing document, this is the map. See `prd2/shared/glossary.md` for full term definitions.

---

## Purpose

The Bardo architecture has 14 distinct feedback loops spread across multiple documents. A reader who has been through them all needs a single view: what evaluates what, at what speed, and how the loops feed each other.

---

## The 14 Feedback Loops, Ordered by Speed

### Tier 1: Machine Speed (per-resolution, 5-15 seconds, ~zero cost)

| # | Loop | What It Evaluates | Metric | Gate Action |
|---|------|------------------|--------|-------------|
| 1 | **Residual correction** | Systematic bias in predictions | Mean residual per (category, regime) | Shift prediction centers, adjust widths |
| 2 | **Confidence calibration** | LLM stated confidence vs actual accuracy | ECE per (category, regime) | Calibrate stated confidence before use |
| 3 | **Adversarial awareness** | Slippage excess, sandwich patterns | Mean excess bps, sandwich frequency | Escalate to private mempool, adjust gas |

These three loops run on every prediction resolution or every action. Pure arithmetic -- no LLM, no human, no waiting. They compound at ~15,000 iterations/day.

### Tier 2: Cognitive Speed (per-theta-tick, 30-120 seconds, T0 cost, where T0/T1/T2 are the three inference tiers: T0 is fast/cheap local inference, T1 is mid-tier cloud inference, T2 is the most capable and expensive model)

| # | Loop | What It Evaluates | Metric | Gate Action |
|---|------|------------------|--------|-------------|
| 4 | **Prediction accuracy** | Were specific claims correct? | Hit rate per category | Action gate permits/blocks |
| 5 | **Action gating** | Should the golem trade this tick? | Action accuracy vs inaction accuracy | Permit or suppress execution |
| 6 | **Context attribution** | Were the right memories retrieved? | Credit/debit per Grimoire entry | Boost/demote retrieval ranking |
| 7 | **Cost-effectiveness** | Is expensive inference worth it? | delta-accuracy per dollar per tier | Shift inference tier routing |
| 8 | **Attention foraging** | What should the golem monitor? | Prediction error per item | Promote/demote attention tier |

These five loops run on every theta tick (or a subset). Mostly arithmetic with occasional cheap reads. ~1,000 iterations/day.

### Tier 3: Consolidation Speed (per-dream-cycle, 4-12 hours, T1-T2 cost)

| # | Loop | What It Evaluates | Metric | Gate Action |
|---|------|------------------|--------|-------------|
| 9 | **NREM residual replay** | What patterns exist in prediction errors? | Pattern detection in residuals | Propose bias corrections, environmental models |
| 10 | **REM counterfactual generation** | What would have happened if...? | Counterfactual prediction accuracy | Generate novel testable hypotheses |
| 11 | **Reasoning quality review** | Is reasoning consistent and aligned? | Alignment rate, contradiction count | Flag fragile reasoning for replay |
| 12 | **Tool selection evaluation** | Did we use the best tool? | Execution quality vs alternatives | Update tool routing preferences |

These four loops run during dream cycles or per-action. LLM reasoning (T1/T2). ~30-50 iterations/day.

### Tier 4: Retrospective Speed (daily/weekly, T1 cost)

| # | Loop | What It Evaluates | Metric | Gate Action |
|---|------|------------------|--------|-------------|
| 13 | **Retrospective PnL** | Was this decision good in hindsight? | Position lifecycle PnL, regret score | Re-tag episodes, update somatic markers |
| 14 | **Heuristic audit** | Which PLAYBOOK rules make money? | PnL per heuristic citation | Promote/demote/investigate heuristics |

These two loops run on daily/weekly schedules. They require full position lifecycle data before producing meaningful results.

### Tier 5: Meta Speed (weekly/generational)

Not numbered above because meta-learning evaluates the other loops, not domain performance:

| Metric | What It Measures | Healthy Signal |
|--------|-----------------|---------------|
| Corrector convergence rate | How fast residuals converge per category | Decreasing over time |
| Dream yield | Fraction of creative predictions confirmed | Stable or increasing |
| Attention precision | Fraction of promoted items that become useful | Above 50% |
| Heuristic half-life | How long PLAYBOOK rules survive | Increasing over time |
| Time-to-competence | Ticks for gen N+1 to reach 70% accuracy | Decreasing over generations |
| Shadow strategy comparison | Would different params have been better? | Real config outperforms shadows |

---

## How the Loops Feed Each Other

```
TIER 1 (machine speed)
+-------------------+  +--------------------+  +---------------------+
| 1. Residual       |  | 2. Confidence      |  | 3. Adversarial      |
|    correction     |  |    calibration      |  |    awareness        |
+--------+----------+  +---------+----------+  +---------+-----------+
         |                       |                        |
         | corrected             | calibrated             | slippage
         | predictions           | confidence             | patterns
         |                       |                        |
         v                       v                        v
TIER 2 (cognitive speed)
+-------------------+  +--------------------+  +---------------------+
| 4. Prediction     |  | 5. Action gate     |  | 6. Context          |
|    accuracy       |<-| (uses 4's data)    |  |    attribution      |
+--------+----------+  +--------------------+  +---------+-----------+
         |                                                |
+--------+----------+  +--------------------+             |
| 7. Cost-          |  | 8. Attention       |             |
|    effectiveness  |  |    foraging        |             |
+--------+----------+  +---------+----------+             |
         |                       |                        |
         | tier routing          | promoted items         | entry scores
         | adjustments           | for richer             | for retrieval
         |                       | predictions            | ranking
         v                       v                        v
TIER 3 (consolidation speed)
+-------------------+  +--------------------+  +---------------------+
| 9. NREM replay    |  | 10. REM            |  | 11. Reasoning       |
| (largest          |-->|  counterfactuals   |  |     quality         |
|  residuals)       |  |  (seeded by 9)     |  |     review          |
+--------+----------+  +---------+----------+  +---------+-----------+
         |                       |                        |
         | bias corrections      | novel hypotheses       | alignment flags
         | environmental         | testable               | contradiction
         | models                | predictions            | counts
         |                       |                        |
         +----------------------------------------------- +
                                 |
+----------------------------------------------------------------+
| 12. Tool selection evaluation                                    |
|     (feeds tool routing, prediction calibration)                 |
+-----------------------------+----------------------------------+
                              |
                              v
TIER 4 (retrospective speed)
+-------------------------------+  +----------------------------+
| 13. Retrospective PnL         |  | 14. Heuristic audit        |
|     position lifecycle        |  |     which rules work?      |
|     vs-inaction comparison    |  |     promote/demote         |
|     regret scoring            |  |                            |
+-------------------------------+  +----------------------------+
         |                                    |
         | re-tagged episodes                 | heuristic
         | somatic markers                    | adjustments
         | long-horizon bias corrections      |
         v                                    v
TIER 5 (meta speed)
+----------------------------------------------------------------+
| META-LEARNING EVALUATION                                         |
|                                                                  |
| Evaluates all loops above:                                       |
| - Is the corrector converging faster?                            |
| - Are dreams producing better hypotheses?                        |
| - Is attention finding useful items?                             |
| - Are heuristics lasting longer?                                 |
| - Is each generation reaching competence faster?                 |
|                                                                  |
| Adjusts: dream frequency, attention thresholds, corrector        |
| parameters, inheritance compression ratio                        |
+----------------------------------------------------------------+
         |
         | If meta-learning score negative for 2 consecutive weeks:
         v
+----------------------------------------------------------------+
| MORTALITY ENGINE                                                 |
| Accelerate epistemic death clock -- the golem can't improve      |
+----------------------------------------------------------------+
```

### The Compounding Property

Each tier's output feeds the tier below it:
- **Tier 1** produces corrected, calibrated predictions. **Tier 2** makes better gating and attention decisions with better data.
- **Tier 2** identifies high-error items and useful context. **Tier 3** dreams about the right things and reviews the right reasoning.
- **Tier 3** discovers patterns and novel hypotheses. **Tier 4** evaluates whether those discoveries translate to PnL.
- **Tier 4** audits strategic effectiveness. **Tier 5** evaluates whether the evaluation loops are improving.
- **Tier 5** adjusts the loops, feeding back to **Tier 1** with better parameters.

After 30 days: ~450,000 Tier 1 corrections, ~30,000 Tier 2 assessments, ~150 Tier 3 dream evaluations, ~4 Tier 4 retrospectives, ~4 Tier 5 meta-evaluations. Each tier amplifies the tiers above it.

---

## The Karpathy Property Across All Loops

Every loop has the same structure: **one metric, one arena, one gate, running as fast as the arena allows**.

| Loop | Arena | Metric | Gate | Speed |
|------|-------|--------|------|-------|
| Residual correction | (category, regime) buffer | Mean residual | Shift center, adjust width | Per-resolution |
| Confidence calibration | (category, regime) calibration curve | ECE | Calibrate stated confidence | Per-resolution |
| Adversarial awareness | Per-transaction slippage | Excess bps | Escalate gas/mempool strategy | Per-action |
| Prediction accuracy | Per-category ledger | Hit rate | Permit/block actions | Per-tick |
| Action gating | Action vs inaction accuracy | Relative accuracy | Permit/block | Per-tick |
| Context attribution | Per-entry credit/debit | Context value score | Boost/demote retrieval | Per-tick |
| Cost-effectiveness | Per-tier accuracy gain | delta-accuracy / $cost | Shift tier routing | Per-tick |
| Attention foraging | Per-item prediction error | Violation count | Promote/demote tier | Per-tick |
| NREM replay | Residual patterns | Pattern significance | Propose corrections | Per-dream |
| REM counterfactuals | Hypothetical scenarios | Counterfactual accuracy | Register predictions | Per-dream |
| Reasoning quality | Trace consistency | Alignment rate | Flag for replay | Per-dream |
| Tool selection | Execution vs alternatives | Execution quality gap | Update preferences | Per-action |
| Retrospective PnL | Position lifecycle | PnL trajectory, regret | Re-tag episodes, somatic markers | Daily/weekly |
| Heuristic audit | PLAYBOOK citations vs PnL | Avg PnL per citation | Promote/demote/investigate | Weekly |

The power is not in any individual loop. It is in all 14 running simultaneously and compounding across each other.

---

## The Owner's View: What They See and When

| When | What the Owner Sees | Which Loops Are Visible |
|------|--------------------|-----------------------|
| **Every glance** | Spectre clarity (accuracy), eye state (affect), status bar (accuracy %, vitality) | 1, 2, 4 |
| **Every few minutes** | Decision ring blazes on escalated ticks, toasts for promotions/blocks | 4, 5, 8 |
| **After dream cycle** | Dream results toast, creative predictions registered | 9, 10, 11 |
| **Daily** | Daily review in FATE > Reviews. PnL attribution, position retrospectives | 13 |
| **Weekly** | Weekly review: heuristic audit, shadow strategy comparison, meta-learning dashboard | 6, 7, 12, 13, 14, Meta |
| **At death** | Death testament: comprehensive epoch review, inheritance report, successor recommendations | All loops contribute |

The owner does not need to understand the 14 loops to benefit from them. They see:
1. **Is the golem getting smarter?** (Spectre clarity + accuracy trend)
2. **Is it making money?** (PnL in status bar + weekly review)
3. **What did it learn?** (Dream results + PLAYBOOK evolution)
4. **Should I change anything?** (Shadow comparisons + heuristic audit)
5. **Is the system itself improving?** (Meta-learning dashboard)

---

## What Could Go Wrong

### Feedback loop oscillation
If meta-learning adjusts dream frequency, and dream frequency changes creative yield, which changes meta-learning scores, the system could oscillate. Mitigation: meta-learning adjustments are rate-limited (one per weekly review) with asymmetric bias (reducing > increasing).

### Over-evaluation paralysis
With 14 feedback loops, the system might spend more time evaluating than acting. Mitigation: Tiers 1-2 are near-zero cost. Tiers 3-4 are bounded by dream cycle and retrospective schedules. Total evaluation cost is <5% of inference budget.

### Metric gaming across loops
If the action gate uses calibrated confidence, and calibration is computed from the same predictions the gate evaluates, there is circularity. Mitigation: calibration is computed on a rolling window excluding the most recent 50 predictions.

### False signal from small samples
Many loops need significant data before metrics are reliable. Every loop reports sample size, and metrics are marked "insufficient data" below configurable thresholds.

---

## References

- [KARPATHY-2026] Karpathy, A. "autoresearch." GitHub, March 2026. — *Demonstrates a single-metric, single-arena evaluation loop running at maximum speed; the structural pattern adopted by every tier in the evaluation map.*
- [XIONG-2023] Xiong, M. et al. "Can LLMs Express Their Uncertainty?" arXiv:2306.13063, 2023. — *Shows LLMs are systematically overconfident (80-100% stated vs 50-60% actual accuracy), motivating the Tier 1 confidence calibration loop.*
- [LIU-METACOGNITION-2025] Liu, T. & Hernández-Lobato, J.M. "Truly Self-Improving Agents Require Intrinsic Metacognitive Learning." ICML 2025. arXiv:2506.05109. — *Argues that agents need loops that evaluate their own evaluation process; the theoretical basis for Tier 5 meta-learning.*
- [CHEN-2025] Chen, A. et al. "Reasoning Models Don't Always Say What They Think." arXiv:2505.05410, 2025. — *Shows reasoning traces may not reflect actual computation, motivating the Tier 3 reasoning quality review loop.*
- [VOVK-2005] Vovk, V. et al. *Algorithmic Learning in a Random World*. Springer, 2005. — *Foundational conformal prediction framework that underpins the residual correction loop's interval width computation.*
- [BARRETT-2017] Barrett, L.F. *How Emotions Are Made: The Secret Life of the Brain*. Houghton Mifflin, 2017. — *Constructionist emotion theory informing how the Daimon's affect signals feed into retrospective episode re-tagging.*
- [HUANG-2024] Huang, J. et al. "Large Language Models Cannot Self-Correct Reasoning Yet." ICLR 2024. — *Demonstrates that intrinsic self-correction degrades LLM performance, constraining what the reasoning quality review (Loop 11) can trust from LLM-generated narratives.*
- [PAN-2024] Pan, A. et al. "Feedback Loops Drive In-Context Reward Hacking in LLMs." ICML 2024. — *Warns that iterated LLM feedback loops can produce reward hacking; the reason meta-learning adjustments are rate-limited to one per weekly review.*
