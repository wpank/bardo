# Triage Pipeline Evaluation [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-18
**Companion to:** `14-chain/02-triage.md`

> Evaluates the four-stage triage pipeline: precision/recall on labeled datasets, Bayesian surprise calibration, Hedge weight convergence, Thompson sampling regret, and end-to-end routing accuracy. The triage pipeline runs at block rate with no LLM calls, so evaluation must verify that its autonomous scoring matches human and LLM assessments of transaction relevance.

> **Reader orientation:** This document evaluates the four-stage triage pipeline that processes every on-chain transaction a Golem (mortal autonomous agent) sees: rule-based fast filters, statistical anomaly detection, contextual enrichment, and curiosity scoring with Bayesian surprise. It belongs to Section 16 (Testing) and measures precision/recall on labeled datasets, Hedge weight convergence, Thompson sampling regret, and end-to-end routing accuracy. The triage pipeline runs at block rate with no LLM calls. See `prd2/shared/glossary.md` for full term definitions.

---

## Document Map

| Section | Topic |
|---------|-------|
| Overview | What we're evaluating and why |
| Labeled Dataset | Construction, labeling methodology, dataset structure |
| Stage 1 Evaluation | Rule-based filter precision and recall |
| Stage 2 Evaluation | MIDAS-R, DDSketch, Count-Min Sketch accuracy |
| Stage 3 Evaluation | Enrichment coverage and ABI resolution rates |
| Stage 4 Evaluation | Curiosity scoring, Bayesian surprise, Hedge weights |
| End-to-End Routing | Full pipeline routing accuracy |
| Self-Learning Loop | Does the curiosity model improve over time? |
| Property Test Sketches | Rust property tests for individual scoring components |

---

## Overview

The triage pipeline processes every transaction that passes the Binary Fuse pre-screen. It classifies transactions through four stages: rule-based fast filters, statistical anomaly detection, contextual enrichment, and upgraded scoring with Bayesian surprise and Thompson sampling. The pipeline emits `GolemEvent::ChainEvent` or `GolemEvent::TriageAlert` for high-scoring transactions, updates protocol state silently for medium-scoring ones, and discards the rest.

Evaluation answers two questions:
1. **Precision:** When triage says a transaction is interesting (score > threshold), how often is it actually interesting?
2. **Recall:** Of all actually interesting transactions, what fraction does triage catch?

"Interesting" is defined by a labeled dataset where human annotators and LLM reviewers tag transactions as relevant or irrelevant to a golem with specific active positions. The definition is context-dependent: a Uniswap swap is interesting to a golem with an LP position in that pool but irrelevant to a golem with no Uniswap exposure.

### Dependencies

- `14-chain/02-triage.md` -- triage pipeline specification
- `14-chain/04-chain-scope.md` -- ChainScope attention model
- Research: `06-curiosity-learning/00-bayesian-surprise.md` -- Bayesian surprise mathematics
- Research: `01-chain-intelligence/02-triage.md` -- four-stage pipeline design

---

## 1. Labeled Dataset [SPEC]

### 1.1 Dataset structure [SPEC]

Each dataset entry contains:

```rust
pub struct LabeledTransaction {
    /// On-chain transaction data.
    pub tx: NormalizedTx,
    pub receipt: TransactionReceipt,
    pub block: BlockMetadata,

    /// The golem context this label was produced under.
    pub golem_context: GolemContext,

    /// Human-assigned label.
    pub label: RelevanceLabel,

    /// LLM-assigned label (from Theta-tick analysis).
    pub llm_label: Option<RelevanceLabel>,

    /// Agreement: human and LLM labels match.
    pub consensus: bool,

    /// Annotator notes (why this label).
    pub rationale: String,
}

pub struct GolemContext {
    /// Active positions at the time of this transaction.
    pub positions: Vec<PositionSummary>,
    /// Strategy targets (addresses the golem is considering).
    pub strategy_targets: Vec<Address>,
    /// Watched protocols.
    pub watched_protocols: Vec<ProtocolId>,
}

pub enum RelevanceLabel {
    /// Directly affects a golem position or strategy.
    HighRelevance,
    /// Indirectly relevant (same protocol, different pool; same token, different pair).
    MediumRelevance,
    /// Not relevant to this golem's current state.
    Irrelevant,
    /// Novel or surprising, worth the golem's attention regardless of position.
    Novel,
}
```

### 1.2 Dataset construction [SPEC]

**Source blocks:** 1000 consecutive mainnet blocks, selected from three different market regimes:
- **Quiet period** (300 blocks): low volatility, normal gas, steady state
- **Volatile period** (400 blocks): high price movement, elevated gas, oracle updates every few blocks
- **Crisis period** (300 blocks): liquidation cascades, MEV spikes, protocol governance events

**Labeling process:**
1. For each block, filter transactions through a reference golem's ChainScope (known positions and strategy).
2. Surviving transactions (typically 5-20% of the block) are presented to annotators.
3. Each transaction is labeled by 2 human annotators and 1 LLM reviewer.
4. Consensus label is majority vote. Disagreements are adjudicated by a third human.

**Minimum dataset size:** 5,000 labeled transactions per market regime. 15,000 total.

### 1.3 Dataset split [SPEC]

| Split | Purpose | Size |
|-------|---------|------|
| Train | Calibrate Hedge weights and Thompson priors | 60% (9,000) |
| Validation | Tune thresholds, detect overfitting | 20% (3,000) |
| Test | Final evaluation metrics (never used for tuning) | 20% (3,000) |

---

## 2. Stage 1 Evaluation: Rule-Based Fast Filters [SPEC]

Stage 1 uses known ABI selectors, watched addresses, and MEV bytecode hashes to make O(1) pass/filter decisions.

### 2.1 Property: Zero false negatives for watched addresses [SPEC]

**Statement:** If a transaction involves an address in `ChainScope.watched_addresses`, Stage 1 must never filter it out. Any transaction touching a watched address must survive to Stage 2.

```rust
#[test]
fn stage1_never_drops_watched_addresses() {
    let dataset = load_labeled_dataset("test");
    let stage = RuleFilterStage::from_scope(&test_scope());

    for entry in &dataset {
        let is_watched = entry.golem_context.positions.iter()
            .any(|p| p.contract_address == entry.tx.to.unwrap_or_default())
            || test_scope().watched_addresses.contains(
                &entry.tx.to.unwrap_or_default()
            );

        if is_watched {
            let mut ctx = TriageContext::from_entry(entry);
            let result = stage.process(&mut ctx).await;
            assert!(
                result.is_some(),
                "Stage 1 dropped a watched-address tx: block {} tx {}",
                entry.block.number, entry.tx.hash
            );
        }
    }
}
```

### 2.2 Metric: Selector coverage [SPEC]

**Definition:** Fraction of labeled `HighRelevance` transactions whose events were decoded by Stage 1's selector map.

**Target:** >= 90%. The remaining 10% are transactions to unknown contracts that Stage 3 enrichment will later decode.

### 2.3 Metric: Filter rate [SPEC]

**Definition:** Fraction of all transactions dropped by Stage 1.

**Target:** >= 85% in quiet periods, >= 70% in crisis periods. Stage 1 is the volume reducer. If it passes too much, downstream stages are overwhelmed. If it filters too aggressively, it drops interesting transactions.

### 2.4 Property: MEV detection does not produce false positives on normal transactions [SPEC]

**Test:** Run Stage 1 on 1000 known non-MEV transactions. Zero should be categorized as `TxCategory::MevActivity`.

---

## 3. Stage 2 Evaluation: Statistical Anomaly Detection [SPEC]

Stage 2 adds MIDAS-R graph anomaly scores, DDSketch distribution outlier detection, and Count-Min Sketch frequency spike detection.

### 3.1 MIDAS-R calibration [SPEC]

**Test:** Feed 500 blocks through the MIDAS-R detector. For each transaction, compare the MIDAS anomaly score against the human label.

**Metrics:**
- AUC-ROC for binary classification: `anomaly score > threshold` vs `label in {HighRelevance, Novel}`
- Target AUC: >= 0.65 (MIDAS alone is one signal among five; it doesn't need to be a strong classifier by itself)

**Property test: MIDAS scores are bounded [SPEC]**

```rust
proptest! {
    #[test]
    fn midas_scores_are_nonnegative(
        from in any::<u32>(),
        to in any::<u32>(),
        block in any::<u64>(),
    ) {
        let mut midas = MidasR::new(1024, 4);
        let score = midas.insert_edge(from, to, block);
        prop_assert!(score >= 0.0, "MIDAS score must be non-negative");
    }
}
```

### 3.2 DDSketch percentile accuracy [SPEC]

**Property:** For any sequence of observations, DDSketch quantile estimates satisfy the relative error guarantee: `|x_hat - x_q| <= alpha * x_q` where alpha = 0.01.

```rust
proptest! {
    #[test]
    fn ddsketch_relative_error(
        values in prop::collection::vec(1.0f64..1e12, 100..1000),
        quantile in 0.01f64..0.99,
    ) {
        let alpha = 0.01;
        let mut sketch = DDSketch::new(alpha);
        for &v in &values {
            sketch.add(v);
        }

        let estimated = sketch.quantile(quantile).unwrap();

        // Compute exact quantile.
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((quantile * sorted.len() as f64) as usize)
            .min(sorted.len() - 1);
        let exact = sorted[idx];

        let error = (estimated - exact).abs() / exact;
        prop_assert!(
            error <= alpha + 1e-9,
            "relative error {:.4} > alpha {:.4}",
            error, alpha,
        );
    }
}
```

### 3.3 Count-Min Sketch frequency estimation [SPEC]

**Property:** Estimated frequency is never less than the true frequency (one-sided error). Overestimate is bounded by `epsilon * N` where N is total observations.

### 3.4 First-seen address detection [SPEC]

**Test:** Feed 100 blocks. For addresses that appear for the first time, verify the `address_novelty` score is > 0. For addresses seen 10+ times, verify `address_novelty` is 0.

---

## 4. Stage 3 Evaluation: Contextual Enrichment [SPEC]

### 4.1 ABI resolution rate [SPEC]

**Definition:** Fraction of unknown contracts encountered by triage that receive ABI data within 1 Theta tick.

**Target:** >= 60% (Sourcify and Etherscan coverage combined). Unresolved contracts fall back to selector-level matching via 4byte.directory.

### 4.2 Protocol state lookup latency [SPEC]

**Property:** `ProtocolStateEngine::get_hot()` completes in < 10us. It's a DashMap read, not a network call.

**Test:** Benchmark 10,000 lookups. 99th percentile < 10us.

### 4.3 Retroactive re-categorization [SPEC]

**Test:** Submit 100 transactions for unknown contracts. After ABI resolution populates the registry, verify that previously-stored `TxCategory::Unknown` entries in redb are updated to their correct category at the next Delta tick.

---

## 5. Stage 4 Evaluation: Curiosity Scoring [SPEC]

Stage 4 combines five signals through a Hedge-weighted compositor: HDC fingerprint similarity, Bayesian surprise, ANN lookup, MIDAS anomaly, and heuristic score.

### 5.1 Bayesian surprise calibration [SPEC]

**Central question:** Does KL divergence correlate with human-assessed novelty?

**Test methodology:**
1. Split the labeled dataset into a warmup set (first 500 transactions, used to initialize conjugate models) and an evaluation set (remaining).
2. For each transaction in the evaluation set, compute Bayesian surprise.
3. Compare surprise values against human labels.

**Metrics:**

| Metric | Target | Measurement |
|--------|--------|-------------|
| Spearman rank correlation | >= 0.3 | Surprise rank vs human novelty rank |
| Precision at top-100 | >= 0.5 | Of the 100 highest-surprise transactions, >= 50 are labeled HighRelevance or Novel |
| Calibration | Monotonic | Mean human relevance score increases monotonically across surprise quintiles |

```rust
#[test]
fn bayesian_surprise_correlates_with_novelty() {
    let dataset = load_labeled_dataset("test");
    let mut scorer = BayesianSurpriseScorer::new(0.995, 0.99);

    // Warmup phase: observe without evaluating.
    for entry in dataset.iter().take(500) {
        let features = extract_features(entry);
        if let Some(protocol) = entry.protocol_id() {
            scorer.score(&protocol, &features);
        }
    }

    // Evaluation phase: collect (surprise, label) pairs.
    let mut pairs: Vec<(f64, f64)> = Vec::new();
    for entry in dataset.iter().skip(500) {
        let features = extract_features(entry);
        if let Some(protocol) = entry.protocol_id() {
            let surprise = scorer.score(&protocol, &features);
            let relevance = match entry.label {
                RelevanceLabel::HighRelevance => 1.0,
                RelevanceLabel::Novel => 0.9,
                RelevanceLabel::MediumRelevance => 0.5,
                RelevanceLabel::Irrelevant => 0.0,
            };
            pairs.push((surprise, relevance));
        }
    }

    let rho = spearman_correlation(&pairs);
    assert!(
        rho >= 0.3,
        "Spearman correlation {:.3} < 0.3",
        rho,
    );
}
```

### 5.2 Conjugate model properties [SPEC]

**Property: KL divergence is non-negative [SPEC]**

```rust
proptest! {
    #[test]
    fn kl_divergence_nonnegative_beta(
        alpha in 0.1f64..100.0,
        beta_param in 0.1f64..100.0,
        obs in any::<bool>(),
    ) {
        let mut model = BetaBinomial::new(alpha, beta_param);
        let kl = model.observe(&obs);
        prop_assert!(kl >= 0.0, "KL divergence must be non-negative, got {}", kl);
    }

    #[test]
    fn kl_divergence_nonnegative_normal_gamma(
        mu in -1000.0f64..1000.0,
        kappa in 0.01f64..100.0,
        alpha in 0.5f64..100.0,
        beta_param in 0.01f64..100.0,
        obs in -1e6f64..1e6,
    ) {
        let mut model = NormalGamma::new(mu, kappa, alpha, beta_param);
        let kl = model.observe(&obs);
        prop_assert!(kl >= 0.0, "KL divergence must be non-negative, got {}", kl);
    }

    #[test]
    fn kl_divergence_nonnegative_dirichlet(
        k in 2usize..20,
        obs_idx in 0usize..20,
    ) {
        let obs_idx = obs_idx % k;
        let mut model = DirichletMultinomial::new(vec![1.0; k]);
        let kl = model.observe(&obs_idx);
        prop_assert!(kl >= 0.0, "KL divergence must be non-negative, got {}", kl);
    }
}
```

**Property: Surprise decreases for repeated identical observations [SPEC]**

After many observations of the same value, the model becomes confident. Observing the same value again should produce low surprise.

```rust
#[test]
fn surprise_decreases_with_repetition() {
    let mut model = NormalGamma::new(0.0, 0.1, 1.0, 1.0);

    let mut surprises: Vec<f64> = Vec::new();
    for _ in 0..100 {
        let s = model.observe(&5.0);
        surprises.push(s);
    }

    // Surprise should trend downward. Compare first 10 average
    // to last 10 average.
    let first_10: f64 = surprises[..10].iter().sum::<f64>() / 10.0;
    let last_10: f64 = surprises[90..].iter().sum::<f64>() / 10.0;
    assert!(
        last_10 < first_10,
        "surprise should decrease: first_10={:.4}, last_10={:.4}",
        first_10, last_10,
    );
}
```

**Property: Outlier observation produces high surprise [SPEC]**

After 100 observations of value ~5.0, observing 500.0 should produce significantly higher surprise than observing 5.1.

```rust
#[test]
fn outlier_produces_high_surprise() {
    let mut model = NormalGamma::new(0.0, 0.1, 1.0, 1.0);
    for _ in 0..100 {
        model.observe(&5.0);
    }

    let mut model_a = model.clone();
    let mut model_b = model.clone();

    let surprise_normal = model_a.observe(&5.1);
    let surprise_outlier = model_b.observe(&500.0);

    assert!(
        surprise_outlier > surprise_normal * 10.0,
        "outlier surprise ({:.4}) should be >> normal ({:.4})",
        surprise_outlier, surprise_normal,
    );
}
```

### 5.3 Decay preserves model responsiveness [SPEC]

**Property:** After exponential decay, the model should be more surprised by an outlier than it was before decay (because decay reduces confidence).

```rust
#[test]
fn decay_increases_responsiveness() {
    let mut model = NormalGamma::new(0.0, 0.1, 1.0, 1.0);
    for _ in 0..1000 {
        model.observe(&5.0);
    }

    // High confidence, low surprise for outlier.
    let mut pre_decay = model.clone();
    let surprise_before = pre_decay.observe(&100.0);

    // Apply aggressive decay.
    model.decay(0.5);

    let surprise_after = model.observe(&100.0);
    assert!(
        surprise_after > surprise_before,
        "post-decay surprise ({:.4}) should exceed pre-decay ({:.4})",
        surprise_after, surprise_before,
    );
}
```

### 5.4 Hedge weight convergence [SPEC]

**Central question:** Do the Hedge/Exponential Weights converge to stable values that reflect the predictive quality of each signal?

**Test methodology:**
1. Feed the full training set through the curiosity scorer.
2. At each Theta tick, update Hedge weights using LLM feedback (from labeled data: treat the human label as the LLM feedback).
3. After processing all training data, record the final weights.
4. Repeat on the validation set. Verify weights remain in a similar range (no catastrophic shift).

**Metrics:**

| Metric | Target |
|--------|--------|
| Weight stability | After 1000 updates, max weight change per update < 0.01 |
| Dominant signal identification | The signal with highest correlation to labels gets the highest weight |
| Cold-start vs warm | Heuristic weight is highest in the first 100 updates, then decreases |

```rust
#[test]
fn hedge_weights_converge() {
    let mut scorer = CuriosityScorer::new_with_defaults();
    let dataset = load_labeled_dataset("train");

    let mut weight_history: Vec<[f64; 5]> = Vec::new();

    for entry in &dataset {
        let mut ctx = TriageContext::from_entry(entry);
        let score = scorer.score(&mut ctx);

        let actual_importance = match entry.label {
            RelevanceLabel::HighRelevance => 1.0,
            RelevanceLabel::Novel => 0.8,
            RelevanceLabel::MediumRelevance => 0.4,
            RelevanceLabel::Irrelevant => 0.0,
        };

        let signals = extract_signals(&ctx);
        scorer.update_weights(&signals, actual_importance);
        weight_history.push(scorer.weights);
    }

    // Check convergence: last 100 updates have small max change.
    let last_100 = &weight_history[weight_history.len()-100..];
    for window in last_100.windows(2) {
        let max_delta = window[0].iter().zip(window[1].iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        assert!(
            max_delta < 0.01,
            "weight change {:.4} > 0.01 in converged region",
            max_delta
        );
    }
}
```

### 5.5 Thompson sampling regret [SPEC]

**Test:** Simulate 5000 routing decisions using Thompson sampling. Compare cumulative regret against a fixed-threshold policy.

**Expected:** Thompson sampling's cumulative regret grows sublinearly (O(sqrt(T * K * ln(K))) for K=4 actions). The fixed-threshold policy's regret grows linearly.

```rust
#[test]
fn thompson_sublinear_regret() {
    let dataset = load_labeled_dataset("test");
    let mut router = ThompsonRouter::new();
    let mut rng = StdRng::seed_from_u64(42);

    let mut thompson_regret = 0.0;
    let mut fixed_regret = 0.0;

    for entry in &dataset {
        let score = entry_to_score(entry);
        let optimal_action = optimal_routing(entry);

        // Thompson routing.
        let thompson_action = router.route(score, &mut rng);
        thompson_regret += routing_loss(thompson_action, optimal_action);

        // Fixed threshold routing.
        let fixed_action = fixed_threshold_route(score);
        fixed_regret += routing_loss(fixed_action, optimal_action);

        // Update Thompson with ground truth.
        let was_relevant = matches!(
            entry.label,
            RelevanceLabel::HighRelevance | RelevanceLabel::Novel
        );
        router.update(thompson_action as usize, was_relevant);
    }

    assert!(
        thompson_regret < fixed_regret,
        "Thompson regret ({:.1}) should be less than fixed ({:.1})",
        thompson_regret, fixed_regret,
    );
}
```

### 5.6 HDC fingerprint properties [SPEC]

**Property: Deterministic encoding [SPEC]**

The same transaction always produces the same hypervector.

```rust
proptest! {
    #[test]
    fn hdc_encoding_deterministic(
        from in any::<[u8; 20]>().prop_map(Address::from),
        to in any::<[u8; 20]>().prop_map(Address::from),
        selector in any::<[u8; 4]>(),
        value in any::<U256>(),
        gas in any::<u64>(),
    ) {
        let tx = make_tx(from, to, &selector, value, gas);
        let encoder = HdcTxEncoder::new_seeded(42);

        let hv1 = encoder.encode(&tx, &[]);
        let hv2 = encoder.encode(&tx, &[]);

        prop_assert_eq!(hv1, hv2);
    }
}
```

**Property: Different roles produce different fingerprints [SPEC]**

A transaction from A to B should produce a different fingerprint than from B to A.

```rust
#[test]
fn hdc_role_sensitivity() {
    let encoder = HdcTxEncoder::new_seeded(42);
    let addr_a = address!("0xaaaa...");
    let addr_b = address!("0xbbbb...");

    let tx_ab = make_tx(addr_a, addr_b, &[0; 4], U256::ZERO, 21000);
    let tx_ba = make_tx(addr_b, addr_a, &[0; 4], U256::ZERO, 21000);

    let hv_ab = encoder.encode(&tx_ab, &[]);
    let hv_ba = encoder.encode(&tx_ba, &[]);

    let similarity = hv_ab.hamming_similarity(&hv_ba);
    // Should be related (same addresses) but not identical (different roles).
    assert!(similarity < 0.8, "from/to swap should produce different vectors");
    assert!(similarity > 0.3, "same addresses should produce related vectors");
}
```

---

## 6. End-to-End Routing Accuracy [SPEC]

### 6.1 Routing confusion matrix [SPEC]

Run the full pipeline on the test set. For each transaction, compare the pipeline's routing decision against the human label.

| | Actual: Relevant | Actual: Irrelevant |
|---|---|---|
| Routed: Escalate/Emit | True Positive | False Positive |
| Routed: Silent/Discard | False Negative | True Negative |

**Targets:**

| Metric | Target |
|--------|--------|
| Precision (Escalate + Emit) | >= 0.60 |
| Recall (HighRelevance) | >= 0.90 |
| Recall (Novel) | >= 0.70 |
| F1 | >= 0.70 |
| False positive rate | <= 0.20 |

Recall on `HighRelevance` (directly affects positions) must be high -- missing a relevant event is worse than seeing an irrelevant one. Recall on `Novel` can be lower because novel events are by definition unexpected and hard to score.

### 6.2 Latency budget [SPEC]

The entire four-stage pipeline must complete within the Gamma tick interval (~250ms) for a full block of transactions.

| Stage | Budget per transaction | Budget per block (200 txs surviving Stage 1) |
|-------|----------------------|----------------------------------------------|
| Stage 1 | < 1us | < 30ms (all ~150 txs in block) |
| Stage 2 | < 5us | < 1ms |
| Stage 3 | < 100us (async, non-blocking) | < 20ms |
| Stage 4 | < 10us | < 2ms |
| Total | < 116us | < 53ms |

**Test:** Benchmark the pipeline on 100 blocks. 99th percentile per-block latency must be < 100ms.

---

## 7. Self-Learning Loop Evaluation [SPEC]

### 7.1 Does scoring improve over time? [SPEC]

**Test:** Feed the training set through the pipeline in chronological order. Measure precision@100 (of the top 100 scored transactions, how many are labeled relevant?) at three checkpoints: after 1000, 5000, and 10000 transactions.

**Expected:** Monotonically increasing precision@100 as Hedge weights stabilize and the ANN index grows.

### 7.2 Epsilon-greedy exploration effectiveness [SPEC]

**Test:** Run the pipeline with epsilon-greedy exploration enabled. Count how many low-scoring transactions were escalated by epsilon exploration. Of those, what fraction turned out to be labeled HighRelevance or Novel?

**Expected:** >= 5% of epsilon-explored transactions are actually relevant. This confirms that the pipeline is discovering interesting events it would otherwise miss.

### 7.3 Negative reinforcement reduces false positives [SPEC]

**Test:** Feed 1000 transactions. For the first 500, the golem "dismisses" all false positive escalations (simulating negative feedback). Measure false positive rate on the second 500 compared to the first 500.

**Expected:** False positive rate decreases by >= 20% in the second half.

---

## 8. Regression Suite [SPEC]

### 8.1 Nightly triage evaluation [SPEC]

Runs against the previous day's transactions for active golems:

1. Collect all transactions processed by triage.
2. For transactions the golem acted on (traded, rebalanced, alerted): label as HighRelevance.
3. For transactions the golem explicitly dismissed: label as Irrelevant.
4. Compute precision and recall against these implicit labels.

This is a noisy signal (the golem's actions aren't a complete labeling) but it tracks trends.

### 8.2 Model drift detection [SPEC]

At each Delta tick, snapshot the Hedge weights and Bayesian model parameters. Compare against the previous Delta snapshot. If any weight changes by more than 0.1 or any conjugate model's effective count exceeds 10,000 (over-confidence), log a warning.

---

## Cross-references

- `14-chain/02-triage.md` -- four-stage triage pipeline specification (rule filters, anomaly detection, enrichment, curiosity scoring)
- `14-chain/04-chain-scope.md` -- ChainScope attention model that determines which addresses feed into Stage 1
- `16-testing/14-chain-scope-testing.md` -- companion testing doc for ChainScope (interest list convergence, Binary Fuse filters, Hebbian reinforcement)
- Research: `06-curiosity-learning/00-bayesian-surprise.md` -- Bayesian surprise mathematics (KL divergence, conjugate models, Hedge weighting)
- Research: `01-chain-intelligence/02-triage.md` -- original four-stage pipeline design and scoring architecture

## References

- Bhatia, S. et al. (2020). MIDAS: Microcluster-Based Detector of Anomalies in Edge Streams. *AAAI 2020*. — *The streaming graph anomaly detector used in Stage 2; scores edges in constant memory and time, making it feasible at block rate.*
- Itti, L. & Baldi, P. (2009). Bayesian Surprise Attracts Human Attention. *Vision Research*, 49(10). — *Formalizes surprise as KL divergence between prior and posterior; the mathematical foundation for Stage 4's curiosity scoring.*
- Masson, C. et al. (2019). DDSketch: A Fast and Fully-Mergeable Quantile Sketch. *PVLDB*, 12(12). — *Provides the relative-error quantile sketch used in Stage 2 for detecting distributional outliers in gas, value, and event count.*
- Russo, D. et al. (2017). A Tutorial on Thompson Sampling. *arXiv:1707.02038*. — *Describes the exploration strategy used in Stage 4 to balance exploiting known-interesting addresses against exploring novel ones.*
- Shalev-Shwartz, S. (2011). Online Learning and Online Convex Optimization. *Foundations and Trends in Machine Learning*, 4(2). — *Theoretical framework for the Hedge algorithm that weights the five curiosity signals in Stage 4's compositor.*
