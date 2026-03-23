# Continuous Apoptosis: Fractal Mortality and the Phage Immune System [SPEC]

> **Version**: 2.1 | **Status**: Draft
>
> **Crate**: `golem-mortality` (phage module)
>
> **Depends on**: `01-architecture.md` (triple-clock system; Fractal Mortality Structure section), `17-information-theoretic-diagnostics.md` (MI-based mortality diagnostics, KSG estimator)

---

> **Reader orientation:** This document specifies fractal mortality, the principle that Bardo Golems (mortal autonomous DeFi agents) die continuously at every timescale. Micro-death: phage sub-agents test hypotheses and die in minutes. Meso-death: the Curator prunes stale Grimoire (persistent knowledge base) entries daily. Macro-death: the Golem itself dies over weeks to months via the three mortality clocks. These levels form a feedback loop where each scale informs the others. See `01-architecture.md` (triple-clock system, fractal structure section) for how this fits the overall architecture. See `prd2/shared/glossary.md` for full term definitions.

## Summary

Fractal mortality operates at three nested levels that mirror biological organization: mitoptosis (mitochondrial death), apoptosis (cell death), and phenoptosis (organism death). This document specifies the sub-agent level -- the Phage immune system that applies biological apoptosis patterns to Replicant sub-agents. The full specification is deferred until the Replicant architecture is available (v2 scope).

The three levels (from `01-architecture.md`):

1. **Micro-Death: Phage Replicants (Hourly)** -- Lightweight micro-Replicants spawned as continuous background processes. Each runs a single-hypothesis test against the current strategy, lives for approximately 50 ticks (~33 minutes), reports back, and dies. Phage death rate serves as a health signal: high death rate with diverse failure modes indicates a healthy immune system; uniform phage death indicates systemic model failure.

2. **Meso-Death: Heuristic Pruning (Daily)** -- The Curator extension prunes Grimoire entries that have lost confidence below the archive threshold through knowledge demurrage. This is the brain's dopamine-Rac1-Cofilin pathway implemented computationally: active forgetting as the default state.

3. **Macro-Death: Golem Mortality (Weeks-Months)** -- The Golem itself dies when composite vitality reaches zero or a stochastic death check fails, triggering the full Thanatopsis Protocol.

The three levels are interdependent: micro informs meso (phage results update Grimoire confidence), meso informs macro (pruning rate feeds epistemic fitness), and macro constrains micro (a Golem in Conservation spawns fewer phages, reducing self-testing and accelerating epistemic decay).

> **Extended:** Full specification -- see [../../prd2-extended/02-mortality/09-fractal-mortality-extended.md](../../prd2-extended/02-mortality/09-fractal-mortality-extended.md)

---

## Multi-Scale Mortality Dynamics

### CLS Consolidation Across Timescales

The Complementary Learning Systems (CLS) framework (McClelland et al., 1995) distinguishes between fast learning in the hippocampus (episodic, high-fidelity, fragile) and slow consolidation into the neocortex (semantic, compressed, durable). Fractal mortality mirrors this architecture across its three levels:

| Timescale | Biological Analog | Learning Mode | Mortality Role |
|---|---|---|---|
| **Micro (minutes)** | Hippocampal encoding | Fast, episodic, single-hypothesis | Generates raw signals about strategy fitness |
| **Meso (hours-days)** | Sleep consolidation | Compression, validation, pruning | Consolidates micro-signals into durable heuristics |
| **Macro (weeks-months)** | Neocortical schema | Slow, abstract, regime-level | Tests whether the Golem's world model still matches reality |

At the micro level, each Phage Replicant tests a single hypothesis in isolation. The test produces a binary signal: confirmed or falsified. This is fast hippocampal encoding -- high-resolution, context-specific, but ephemeral.

At the meso level, the Curator aggregates micro-level signals across many Phage deaths. An entry that is consistently falsified by Phages loses confidence and gets pruned. An entry that survives Phage testing gains confidence and consolidates. This is the sleep-like consolidation process: many episodes compress into a few durable heuristics.

At the macro level, the three-clock mortality system tests whether the consolidated heuristics -- the Golem's schema -- still match the market. When they don't, epistemic decay accelerates, composite vitality drops, and the Golem approaches death. This is the slow neocortical evaluation: the schema worked for a while, but the world changed.

### Bayesian Surprise at Multiple Timescales

Bayesian surprise (Itti & Baldi, 2009) measures the KL divergence between the prior belief distribution and the posterior after observing new data:

```
S(D) = D_KL(posterior || prior) = integral p(theta|D) * log(p(theta|D) / p(theta)) d(theta)
```

Each fractal level generates surprise at a different resolution:

**Micro-surprise (per Phage, per tick).** Each Phage tests a hypothesis against current market conditions. The surprise from a single Phage outcome is:

```
S_micro = D_KL(p(hypothesis | phage_result) || p(hypothesis))
```

A Phage that falsifies a high-confidence hypothesis produces high surprise. A Phage that confirms what the Golem already believes produces near-zero surprise. The micro-surprise aggregated across all Phages in a cycle provides a real-time signal of how well the Golem's beliefs match reality.

```rust
/// Compute Bayesian surprise from a single Phage result.
///
/// `prior_confidence`: the Grimoire entry's confidence before the Phage ran.
/// `phage_confirmed`: whether the Phage confirmed the hypothesis.
pub fn micro_surprise(prior_confidence: f64, phage_confirmed: bool) -> f64 {
    let prior = prior_confidence.clamp(0.01, 0.99);
    let posterior = if phage_confirmed {
        // Bayesian update with a binary test.
        // Assuming test sensitivity = 0.9, specificity = 0.9.
        let sensitivity = 0.9;
        let specificity = 0.9;
        let p_positive_given_true = sensitivity;
        let p_positive_given_false = 1.0 - specificity;
        let numerator = p_positive_given_true * prior;
        let denominator = numerator + p_positive_given_false * (1.0 - prior);
        numerator / denominator
    } else {
        let sensitivity = 0.9;
        let specificity = 0.9;
        let p_negative_given_true = 1.0 - sensitivity;
        let p_negative_given_false = specificity;
        let numerator = p_negative_given_true * prior;
        let denominator = numerator + p_negative_given_false * (1.0 - prior);
        numerator / denominator
    };

    // KL divergence for Bernoulli distributions.
    kl_divergence_bernoulli(posterior, prior)
}

fn kl_divergence_bernoulli(p: f64, q: f64) -> f64 {
    let p = p.clamp(1e-10, 1.0 - 1e-10);
    let q = q.clamp(1e-10, 1.0 - 1e-10);
    p * (p / q).ln() + (1.0 - p) * ((1.0 - p) / (1.0 - q)).ln()
}
```

**Meso-surprise (per Curator cycle, ~50 ticks).** The Curator runs every 50 ticks and evaluates the aggregate health of the Grimoire. Meso-surprise measures how much the Grimoire's confidence distribution shifted during the cycle:

```
S_meso = D_KL(confidence_distribution_after || confidence_distribution_before)
```

A cycle where many entries lose confidence (market regime changed, many heuristics invalidated) produces high meso-surprise. A cycle where confidence is stable produces low meso-surprise.

```rust
/// Compute meso-surprise from a Curator cycle.
///
/// `pre_confidences`: confidence distribution before the cycle.
/// `post_confidences`: confidence distribution after the cycle.
pub fn meso_surprise(pre: &[f64], post: &[f64]) -> f64 {
    assert_eq!(pre.len(), post.len());
    if pre.is_empty() { return 0.0; }

    // Compute histograms of confidence values.
    let bins = 20;
    let pre_hist = histogram(pre, bins);
    let post_hist = histogram(post, bins);

    // KL divergence between the two histograms.
    kl_divergence_discrete(&post_hist, &pre_hist)
}

fn histogram(values: &[f64], bins: usize) -> Vec<f64> {
    let mut hist = vec![1e-10; bins]; // smoothing
    for &v in values {
        let bin = ((v * bins as f64) as usize).min(bins - 1);
        hist[bin] += 1.0;
    }
    let total: f64 = hist.iter().sum();
    hist.iter().map(|&h| h / total).collect()
}

fn kl_divergence_discrete(p: &[f64], q: &[f64]) -> f64 {
    p.iter()
        .zip(q.iter())
        .map(|(&pi, &qi)| {
            if pi > 1e-10 { pi * (pi / qi).ln() } else { 0.0 }
        })
        .sum()
}
```

**Macro-surprise (per epoch, weeks-months).** The three-clock mortality system implicitly measures macro-surprise through the composite vitality score. A sudden drop in composite vitality corresponds to a large macro-surprise: the Golem's schema no longer matches the market at the highest level of abstraction.

The information-theoretic diagnostic layer (17-information-theoretic-diagnostics.md) provides a direct quantification: the change in I(G; M) over an epoch is the macro-surprise. A Golem whose mutual information with the market drops sharply has experienced a macro-scale model failure.

### Cross-Scale Surprise Propagation

Surprise propagates upward through the fractal levels with amplification:

```
Micro-surprise accumulates -> triggers meso-level reevaluation
Meso-surprise accumulates -> triggers macro-level phase transition
```

The propagation is not linear. A single high micro-surprise event (one Phage result that dramatically contradicts a core belief) can trigger an immediate meso-level Curator cycle (out-of-band, not waiting for the 50-tick schedule). Similarly, sustained high meso-surprise (many entries losing confidence over several cycles) directly feeds the epistemic mortality clock, accelerating macro-level decline.

```rust
/// Fractal surprise aggregator.
///
/// Tracks surprise at all three levels and triggers cross-scale
/// propagation when thresholds are exceeded.
pub struct FractalSurpriseTracker {
    /// Rolling micro-surprise over the current Curator window.
    micro_accumulator: f64,
    micro_count: u32,
    /// Rolling meso-surprise over the current epoch.
    meso_accumulator: f64,
    meso_count: u32,
    /// Threshold for micro -> meso escalation.
    micro_escalation_threshold: f64,
    /// Threshold for meso -> macro escalation.
    meso_escalation_threshold: f64,
}

/// Events emitted by the surprise tracker.
#[derive(Debug, Clone)]
pub enum SurpriseEvent {
    /// A single Phage result produced extreme surprise.
    MicroSpike { surprise: f64, hypothesis_id: String },
    /// Accumulated micro-surprise exceeded the meso threshold.
    MesoEscalation { accumulated: f64, phage_count: u32 },
    /// Accumulated meso-surprise exceeded the macro threshold.
    MacroEscalation { accumulated: f64, curator_cycles: u32 },
}

impl FractalSurpriseTracker {
    pub fn new(micro_threshold: f64, meso_threshold: f64) -> Self {
        Self {
            micro_accumulator: 0.0,
            micro_count: 0,
            meso_accumulator: 0.0,
            meso_count: 0,
            micro_escalation_threshold: micro_threshold,
            meso_escalation_threshold: meso_threshold,
        }
    }

    /// Record a micro-surprise from a Phage result.
    pub fn record_micro(&mut self, surprise: f64) -> Option<SurpriseEvent> {
        self.micro_accumulator += surprise;
        self.micro_count += 1;

        if self.micro_accumulator > self.micro_escalation_threshold {
            let event = SurpriseEvent::MesoEscalation {
                accumulated: self.micro_accumulator,
                phage_count: self.micro_count,
            };
            self.micro_accumulator = 0.0;
            self.micro_count = 0;
            Some(event)
        } else {
            None
        }
    }

    /// Record a meso-surprise from a Curator cycle.
    pub fn record_meso(&mut self, surprise: f64) -> Option<SurpriseEvent> {
        self.meso_accumulator += surprise;
        self.meso_count += 1;

        if self.meso_accumulator > self.meso_escalation_threshold {
            let event = SurpriseEvent::MacroEscalation {
                accumulated: self.meso_accumulator,
                curator_cycles: self.meso_count,
            };
            self.meso_accumulator = 0.0;
            self.meso_count = 0;
            Some(event)
        } else {
            None
        }
    }

    /// Reset micro-level tracking (called after each Curator cycle).
    pub fn reset_micro(&mut self) {
        self.micro_accumulator = 0.0;
        self.micro_count = 0;
    }
}
```

### Phage Death Diversity as Health Signal

The fractal mortality system uses Phage death patterns as a diagnostic:

| Pattern | Interpretation | Action |
|---------|---------------|--------|
| High death rate, diverse failure modes | Healthy immune system testing boundaries | Continue normal Phage budget |
| High death rate, uniform failure mode | Systemic model failure | Trigger out-of-band Curator, raise arousal |
| Low death rate, diverse hypotheses | Complacency or overfitting | Increase Phage spawn rate, diversify hypotheses |
| Low death rate, narrow hypotheses | Stagnation | Inject random hypotheses, increase noise |

The diversity of Phage failure modes is measured by the Shannon entropy of the failure-cause distribution:

```rust
/// Compute the diversity of Phage failure causes.
///
/// Returns the normalized Shannon entropy of the failure distribution.
/// High = diverse failures (healthy). Low = uniform failures (systemic).
pub fn phage_death_diversity(failure_causes: &[String]) -> f64 {
    if failure_causes.is_empty() { return 0.0; }

    let mut counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for cause in failure_causes {
        *counts.entry(cause.as_str()).or_insert(0) += 1;
    }

    let n = failure_causes.len() as f64;
    let max_entropy = (counts.len() as f64).ln();
    if max_entropy < 1e-10 { return 0.0; }

    let entropy: f64 = counts.values()
        .map(|&c| {
            let p = c as f64 / n;
            if p > 0.0 { -p * p.ln() } else { 0.0 }
        })
        .sum();

    entropy / max_entropy
}
```

---

## Fractal Integration with MI Diagnostics

The information-theoretic diagnostic layer (17-information-theoretic-diagnostics.md) provides a unifying measurement across all three fractal levels.

At the micro level, each Phage tests whether the Golem's actions are informationally coupled to market outcomes. A Phage that runs a variant strategy and finds zero mutual information between its actions and market responses has detected decoupling at the hypothesis level.

At the meso level, the Curator can use the MI diagnostic to prioritize pruning. Grimoire entries associated with low-MI strategies should be pruned faster than entries associated with high-MI strategies. The information rate signal on CorticalState provides this prioritization input.

At the macro level, the MI vitality V(G) is the ultimate fractal mortality metric. It unifies economic, epistemic, and stochastic mortality into a single information-theoretic scalar that operates at the weeks-to-months timescale.

---

## CorticalState Signals

Two signals specific to fractal mortality:

```
phage_death_diversity: AtomicU32  // f32 in [0, 1], entropy of failure causes
surprise_accumulator: AtomicU32   // f32, rolling accumulated surprise
```

These complement the three MI signals from 17-information-theoretic-diagnostics.md and the two morphogenetic signals from 10b-morphogenetic-specialization.md.

---

## Cross-References

- **01-architecture.md**: Defines the three fractal levels and their interdependencies.
- **17-information-theoretic-diagnostics.md**: MI diagnostics provide the unifying measurement across all three fractal levels.
- **05-knowledge-demurrage.md**: The Curator's pruning mechanism is the meso-death implementation.
- **18-antifragile-mortality.md**: Stress-aware knowledge harvesting interacts with meso-level surprise -- high-stress periods produce both high surprise and high-value K- entries.
- **02-epistemic-decay.md**: Epistemic fitness is directly informed by the Phage death rate and meso-surprise signals.

---

## References

- McClelland, J.L., McNaughton, B.L. & O'Reilly, R.C. (1995). "Why There Are Complementary Learning Systems in the Hippocampus and Neocortex." *Psychological Review*, 102(3), 419-457.
- Itti, L. & Baldi, P. (2009). "Bayesian Surprise Attracts Human Attention." *Vision Research*, 49(10), 1295-1306.
- Davis, R.L. & Zhong, Y. (2017). "The Biology of Forgetting." *Neuron*, 95(5), 1071-1089.
- Vostinar, A.E., Goldsby, H.J. & Ofria, C. (2019). "Suicidal Selection." *Artificial Life*, 25(4), 328-344.
