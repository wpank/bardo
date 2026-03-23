# Streaming Anomaly and Changepoint Detection [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-18
>
> **Crates**: `bardo-chain`, `bardo-curiosity`
>
> **Depends on**: [00-architecture.md](00-architecture.md) (chain intelligence pipeline), [02-triage.md](02-triage.md) (triage stages), `../06-curiosity/00-bayesian-surprise.md` (per-feature surprise), `../06-curiosity/02-online-learning.md` (Hedge-weighted composite)

> **Reader orientation:** This document specifies four streaming anomaly and changepoint detection algorithms used in Bardo's chain intelligence layer (section 14): MIDAS-R for graph-stream burst detection, BOCPD for online regime change detection, PELT for retrospective segmentation, and ADWIN for adaptive windowing. These complement the per-feature Bayesian surprise models in the curiosity engine. The key concept is that different anomaly types require different algorithms operating at different timescales, and the Hedge-weighted composite decides how much each signal matters. See `prd2/shared/glossary.md` for full term definitions.

The Bayesian surprise models in the curiosity engine detect when individual observations violate the agent's per-feature beliefs. But they miss two categories of anomaly: structural graph anomalies (sudden bursts of activity between address pairs) and regime changes (the entire distribution shifting, not just a single outlier). This document covers four algorithms that fill those gaps: MIDAS-R for graph-stream anomaly detection, BOCPD for online changepoint detection, PELT for retrospective segmentation, and ADWIN for adaptive windowing. MIDAS-R and BOCPD are the primary real-time algorithms; PELT and ADWIN provide supporting capabilities at different timescales.

Together with Bayesian surprise and the online learning algorithms from the curiosity engine, these four algorithms complete the detection pipeline: Bayesian surprise catches surprising individual observations, MIDAS-R catches surprising interaction patterns, BOCPD catches regime transitions, and the online learner decides how much weight each signal deserves.

---

## MIDAS and MIDAS-R

### What MIDAS detects

[SPEC] MIDAS (Microcluster-Based Detector of Anomalies in Edge Streams) detects bursts of edges in temporal graphs. A temporal graph is a stream of (source, destination, timestamp) tuples. MIDAS identifies microcluster anomalies -- sudden spikes in activity between specific node pairs that deviate from historical baselines.

For Ethereum, model the transaction graph as edges: `(from_address, to_address, block_number)`. MIDAS detects:

- Flash loan cascades: a burst of transactions between the same set of addresses within one block
- Wash trading: repeated transfers between the same pair of addresses at abnormal frequency
- MEV bot coordination: a cluster of transactions from multiple bots all targeting the same contract
- Airdrop farming: sudden activity from a set of previously dormant addresses to a single contract

Bhatia et al. (2020) showed MIDAS achieves 42-52% higher AUC than prior streaming methods (SEDANSPOT, PENminer) while processing each edge in O(1) time and O(1) memory.

### MIDAS-R: adding temporal relations and node-level scores

[SPEC] MIDAS-R extends MIDAS with two additions:

1. **Temporal relations**: Instead of a single "current tick" counter, MIDAS-R maintains a temporal decay factor. Recent edges contribute more than old ones, so the anomaly score reflects the current burst intensity rather than cumulative history.

2. **Node-level scores**: Beyond edge anomalies (unusual activity between a specific pair), MIDAS-R also computes node anomalies (unusual in-degree or out-degree for a single address). This catches scenarios where a contract suddenly receives transactions from many different addresses -- not a pair anomaly, but still suspicious.

### Count-Min Sketch internals

[SPEC] MIDAS-R maintains four Count-Min Sketch (CMS) structures:

```
s_total[w][d]   -- total edge counts, all time
s_current[w][d] -- edge counts in current tick
a_total[w][d]   -- total node counts, all time  (MIDAS-R only)
a_current[w][d] -- node counts in current tick   (MIDAS-R only)
```

Each CMS has width w and depth d. The width controls accuracy (larger = fewer hash collisions), the depth controls failure probability. At w=1024, d=4, each CMS occupies 32KB (1024 * 4 * 8 bytes for 64-bit counters). Total memory: 128KB for all four sketches.

The anomaly score for an edge (u, v) at tick t is the chi-squared statistic comparing observed to expected frequency:

```
score(u, v, t) = (s_current(u,v) - s_total(u,v) / t)^2
                 / (s_total(u,v) / t)
```

If the current-tick count is much higher than the historical average, the score is high. Combined with the node-level score for both u and v, MIDAS-R produces a single anomaly score per edge.

### Rust implementation

[SPEC] CountMinSketch and MidasR structs with full method bodies.

```rust
use std::hash::{Hash, Hasher};
use ahash::AHasher;

/// Count-Min Sketch for MIDAS.
/// Uses multiple hash functions to estimate element frequencies
/// with bounded overcount.
struct CountMinSketch {
    counters: Vec<Vec<f64>>,
    width: usize,
    depth: usize,
    seeds: Vec<u64>,
}

impl CountMinSketch {
    fn new(width: usize, depth: usize) -> Self {
        Self {
            counters: vec![vec![0.0; width]; depth],
            width,
            depth,
            seeds: (0..depth as u64).collect(),
        }
    }

    fn hash_pair(&self, a: u64, b: u64, seed: u64) -> usize {
        let mut hasher = AHasher::default();
        seed.hash(&mut hasher);
        a.hash(&mut hasher);
        b.hash(&mut hasher);
        (hasher.finish() as usize) % self.width
    }

    fn hash_single(&self, a: u64, seed: u64) -> usize {
        let mut hasher = AHasher::default();
        seed.hash(&mut hasher);
        a.hash(&mut hasher);
        (hasher.finish() as usize) % self.width
    }

    fn insert_edge(&mut self, src: u64, dst: u64, weight: f64) {
        for (i, seed) in self.seeds.iter().enumerate() {
            let idx = self.hash_pair(src, dst, *seed);
            self.counters[i][idx] += weight;
        }
    }

    fn insert_node(&mut self, node: u64, weight: f64) {
        for (i, seed) in self.seeds.iter().enumerate() {
            let idx = self.hash_single(node, *seed);
            self.counters[i][idx] += weight;
        }
    }

    fn query_edge(&self, src: u64, dst: u64) -> f64 {
        self.seeds
            .iter()
            .enumerate()
            .map(|(i, seed)| {
                let idx = self.hash_pair(src, dst, *seed);
                self.counters[i][idx]
            })
            .fold(f64::MAX, f64::min)
    }

    fn query_node(&self, node: u64) -> f64 {
        self.seeds
            .iter()
            .enumerate()
            .map(|(i, seed)| {
                let idx = self.hash_single(node, *seed);
                self.counters[i][idx]
            })
            .fold(f64::MAX, f64::min)
    }

    /// Apply temporal decay to all counters.
    fn decay(&mut self, factor: f64) {
        for row in self.counters.iter_mut() {
            for cell in row.iter_mut() {
                *cell *= factor;
            }
        }
    }

    fn clear(&mut self) {
        for row in self.counters.iter_mut() {
            for cell in row.iter_mut() {
                *cell = 0.0;
            }
        }
    }
}

/// MIDAS-R anomaly detector for transaction edge streams.
pub struct MidasR {
    /// Total edge counts (all time, with decay)
    edge_total: CountMinSketch,
    /// Current tick edge counts
    edge_current: CountMinSketch,
    /// Total node counts (in-degree + out-degree, with decay)
    node_total: CountMinSketch,
    /// Current tick node counts
    node_current: CountMinSketch,
    /// Current tick number
    current_tick: u64,
    /// Temporal decay factor applied at each tick boundary
    decay_factor: f64,
    /// Anomaly score threshold for alerting
    threshold: f64,
}

impl MidasR {
    pub fn new(width: usize, depth: usize, decay_factor: f64, threshold: f64) -> Self {
        Self {
            edge_total: CountMinSketch::new(width, depth),
            edge_current: CountMinSketch::new(width, depth),
            node_total: CountMinSketch::new(width, depth),
            node_current: CountMinSketch::new(width, depth),
            current_tick: 0,
            decay_factor,
            threshold,
        }
    }

    /// Process a new edge in the transaction graph.
    /// Returns the anomaly score (higher = more anomalous).
    pub fn process_edge(&mut self, src: u64, dst: u64, tick: u64) -> f64 {
        // Advance tick if needed
        if tick > self.current_tick {
            self.advance_tick(tick);
        }

        // Update current-tick counts
        self.edge_current.insert_edge(src, dst, 1.0);
        self.node_current.insert_node(src, 1.0);
        self.node_current.insert_node(dst, 1.0);

        // Update total counts
        self.edge_total.insert_edge(src, dst, 1.0);
        self.node_total.insert_node(src, 1.0);
        self.node_total.insert_node(dst, 1.0);

        // Compute anomaly scores
        let edge_score = self.chi_squared_score(
            self.edge_current.query_edge(src, dst),
            self.edge_total.query_edge(src, dst),
        );

        let src_score = self.chi_squared_score(
            self.node_current.query_node(src),
            self.node_total.query_node(src),
        );

        let dst_score = self.chi_squared_score(
            self.node_current.query_node(dst),
            self.node_total.query_node(dst),
        );

        // Combined score: max of edge and node anomalies
        edge_score.max(src_score).max(dst_score)
    }

    /// Is this edge anomalous?
    pub fn is_anomalous(&mut self, src: u64, dst: u64, tick: u64) -> bool {
        self.process_edge(src, dst, tick) > self.threshold
    }

    fn advance_tick(&mut self, new_tick: u64) {
        // Apply decay to totals for each elapsed tick
        let ticks_elapsed = new_tick - self.current_tick;
        let cumulative_decay = self.decay_factor.powi(ticks_elapsed as i32);
        self.edge_total.decay(cumulative_decay);
        self.node_total.decay(cumulative_decay);

        // Reset current-tick counters
        self.edge_current.clear();
        self.node_current.clear();

        self.current_tick = new_tick;
    }

    fn chi_squared_score(&self, current: f64, total: f64) -> f64 {
        if total < 1.0 {
            return 0.0; // Not enough history
        }
        let expected = total / (self.current_tick as f64 + 1.0);
        if expected < 0.001 {
            return 0.0; // Expected count too small for meaningful chi-squared
        }
        (current - expected).powi(2) / expected
    }
}
```

### Integration with chain intelligence

[SPEC] MIDAS-R slots into the triage pipeline at Stage 4, running alongside Bayesian surprise:

```rust
// In the triage pipeline, after Stage 2 address triage passes:
fn score_with_midas(
    midas: &mut MidasR,
    tx: &Transaction,
    block_number: u64,
) -> f64 {
    let src = address_to_u64(&tx.from);
    let dst = address_to_u64(&tx.to.unwrap_or_default());
    midas.process_edge(src, dst, block_number)
}

// address_to_u64: take the first 8 bytes of the address as u64.
// Collisions are acceptable -- MIDAS already uses approximate counting.
fn address_to_u64(addr: &Address) -> u64 {
    u64::from_be_bytes(addr.0[..8].try_into().unwrap())
}
```

The anomaly score feeds into the Hedge-weighted curiosity composite. The Hedge algorithm learns how much weight to give MIDAS relative to Bayesian surprise and the other signals.

**Tick alignment**: MIDAS-R uses `block_number` as its tick. Each Ethereum block is one tick. The decay factor of 0.9 per block gives a half-life of about 7 blocks (~84 seconds). This means MIDAS responds quickly to microbursts (activity within a single block or across 2-3 blocks) while maintaining a baseline over roughly the last minute.

**Memory footprint**: At width=1024, depth=4, four CMS structures: 128KB total. This is fixed regardless of how many addresses or transactions the golem observes.

---

## BOCPD: Bayesian online changepoint detection

### What BOCPD detects

[SPEC] BOCPD (Adams & MacKay, 2007) detects regime changes in streaming data. A regime change is a shift in the underlying data-generating distribution -- not a single outlier, but a persistent change. Gas prices shifting from 20 gwei to 100 gwei. Swap volume on a pool dropping by 80% after a governance vote. MEV activity spiking after a new DEX launch.

BOCPD maintains a distribution over "run lengths" -- how long the current regime has been active. When the posterior probability of a short run length spikes, a changepoint has been detected.

### The run-length distribution

At each timestep t, BOCPD maintains a vector of run-length probabilities:

```
P(r_t = r | x_{1:t})  for r = 0, 1, ..., t
```

Where r_t is the current run length. The update has two steps:

**Growth probability**: The current regime continues.
```
P(r_t = r+1 | x_{1:t}) ∝ P(x_t | r_t = r+1) * P(r_{t-1} = r | x_{1:t-1}) * (1 - H(r))
```

**Changepoint probability**: A new regime begins.
```
P(r_t = 0 | x_{1:t}) ∝ P(x_t | r_t = 0) * sum_r [ P(r_{t-1} = r | x_{1:t-1}) * H(r) ]
```

Where H(r) is the hazard function -- the prior probability of a changepoint given that the current run length is r. A constant hazard H = 1/lambda means changepoints occur on average every lambda timesteps.

### Rust implementation

[SPEC] Full BOCPD with Normal-Gamma sufficient statistics and Student-t predictive distribution.

```rust
use statrs::function::gamma::ln_gamma;

/// Bayesian Online Changepoint Detection.
/// Maintains a distribution over run lengths and detects
/// when the data-generating distribution shifts.
pub struct Bocpd {
    /// Run-length distribution: run_lengths[r] = P(run_length = r | data)
    run_lengths: Vec<f64>,
    /// Sufficient statistics for each run length's predictive model.
    /// Using Normal-Gamma for continuous data.
    sufficient_stats: Vec<NormalGammaSufficient>,
    /// Hazard rate: probability of changepoint per timestep.
    /// hazard = 1/expected_run_length
    hazard: f64,
    /// Prior parameters for a fresh regime
    prior_mu: f64,
    prior_kappa: f64,
    prior_alpha: f64,
    prior_beta: f64,
    /// Maximum run length to track (truncation for bounded memory)
    max_run_length: usize,
    /// Timestep counter
    t: usize,
}

/// Sufficient statistics for Normal-Gamma predictive model.
#[derive(Clone, Debug)]
struct NormalGammaSufficient {
    mu: f64,
    kappa: f64,
    alpha: f64,
    beta: f64,
}

impl NormalGammaSufficient {
    fn new(mu: f64, kappa: f64, alpha: f64, beta: f64) -> Self {
        Self { mu, kappa, alpha, beta }
    }

    /// Predictive probability of observing x under this model.
    /// The predictive distribution is a Student-t.
    fn predictive_log_prob(&self, x: f64) -> f64 {
        let nu = 2.0 * self.alpha;
        let sigma_sq = self.beta * (self.kappa + 1.0) / (self.alpha * self.kappa);

        // Student-t log probability
        let z = (x - self.mu) / sigma_sq.sqrt();
        ln_gamma((nu + 1.0) / 2.0) - ln_gamma(nu / 2.0)
            - 0.5 * (nu * std::f64::consts::PI * sigma_sq).ln()
            - ((nu + 1.0) / 2.0) * (1.0 + z * z / nu).ln()
    }

    /// Update with a new observation.
    fn update(&mut self, x: f64) {
        let kappa_new = self.kappa + 1.0;
        let mu_new = (self.kappa * self.mu + x) / kappa_new;
        let alpha_new = self.alpha + 0.5;
        let beta_new = self.beta
            + (self.kappa * (x - self.mu).powi(2)) / (2.0 * kappa_new);

        self.mu = mu_new;
        self.kappa = kappa_new;
        self.alpha = alpha_new;
        self.beta = beta_new;
    }
}

impl Bocpd {
    pub fn new(
        hazard: f64,
        prior_mu: f64,
        prior_kappa: f64,
        prior_alpha: f64,
        prior_beta: f64,
        max_run_length: usize,
    ) -> Self {
        let initial_stats = NormalGammaSufficient::new(
            prior_mu, prior_kappa, prior_alpha, prior_beta,
        );
        Self {
            run_lengths: vec![1.0], // Start with run_length = 0, probability 1
            sufficient_stats: vec![initial_stats],
            hazard,
            prior_mu,
            prior_kappa,
            prior_alpha,
            prior_beta,
            max_run_length,
            t: 0,
        }
    }

    /// Process a new observation. Returns the changepoint probability
    /// (probability that a regime change occurred at this timestep).
    pub fn update(&mut self, x: f64) -> ChangePointResult {
        let n = self.run_lengths.len();

        // Step 1: Compute predictive probabilities for each run length
        let mut pred_probs = Vec::with_capacity(n);
        for stats in &self.sufficient_stats {
            pred_probs.push(stats.predictive_log_prob(x).exp());
        }

        // Step 2: Growth probabilities
        let mut new_run_lengths = vec![0.0; (n + 1).min(self.max_run_length + 1)];

        // Growth: existing run lengths extend by 1
        for r in 0..n.min(self.max_run_length) {
            new_run_lengths[r + 1] = self.run_lengths[r] * pred_probs[r] * (1.0 - self.hazard);
        }

        // Changepoint: new regime starts (run_length = 0)
        let changepoint_mass: f64 = (0..n)
            .map(|r| self.run_lengths[r] * pred_probs[r] * self.hazard)
            .sum();
        new_run_lengths[0] = changepoint_mass;

        // Step 3: Normalize
        let total: f64 = new_run_lengths.iter().sum();
        if total > 0.0 {
            for p in new_run_lengths.iter_mut() {
                *p /= total;
            }
        }

        // Step 4: Update sufficient statistics
        let mut new_stats = Vec::with_capacity(new_run_lengths.len());
        // Run length 0: fresh prior
        new_stats.push(NormalGammaSufficient::new(
            self.prior_mu, self.prior_kappa, self.prior_alpha, self.prior_beta,
        ));
        // Run lengths > 0: extend existing stats
        for r in 0..n.min(self.max_run_length) {
            let mut stats = self.sufficient_stats[r].clone();
            stats.update(x);
            new_stats.push(stats);
        }

        self.run_lengths = new_run_lengths;
        self.sufficient_stats = new_stats;
        self.t += 1;

        // Changepoint probability = P(r_t = 0)
        let cp_prob = self.run_lengths[0];

        // Most probable run length
        let map_run_length = self.run_lengths
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(r, _)| r)
            .unwrap_or(0);

        ChangePointResult {
            changepoint_probability: cp_prob,
            map_run_length,
            is_changepoint: cp_prob > 0.5,
        }
    }

    /// Get the current regime's age (most probable run length).
    pub fn current_run_length(&self) -> usize {
        self.run_lengths
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(r, _)| r)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct ChangePointResult {
    /// Probability that a changepoint occurred at this timestep.
    pub changepoint_probability: f64,
    /// Most probable current run length (age of current regime).
    pub map_run_length: usize,
    /// Shorthand: is changepoint_probability > 0.5?
    pub is_changepoint: bool,
}
```

### Integration with chain intelligence

[SPEC] BOCPD runs at Theta tick cadence (~5 minutes) on aggregate metrics:

```rust
pub struct RegimeDetector {
    /// BOCPD instance per monitored metric
    detectors: HashMap<String, Bocpd>,
}

impl RegimeDetector {
    pub fn new(expected_regime_length: f64) -> Self {
        let hazard = 1.0 / expected_regime_length;
        let mut detectors = HashMap::new();

        // Gas price distribution: expect regime changes every ~50 Theta ticks (~4 hours)
        detectors.insert(
            "gas_price_median".to_string(),
            Bocpd::new(hazard, 30.0, 0.1, 1.0, 100.0, 500),
        );

        // Total swap volume: regime changes every ~100 Theta ticks (~8 hours)
        detectors.insert(
            "swap_volume_log".to_string(),
            Bocpd::new(hazard * 0.5, 10.0, 0.1, 1.0, 10.0, 500),
        );

        // Protocol deployment rate
        detectors.insert(
            "deploy_rate".to_string(),
            Bocpd::new(hazard, 1.0, 0.1, 1.0, 1.0, 500),
        );

        Self { detectors }
    }

    /// Process Theta-tick aggregate metrics. Returns detected changepoints.
    pub fn process_theta_metrics(&mut self, metrics: &HashMap<String, f64>) -> Vec<ChangePointAlert> {
        let mut alerts = Vec::new();
        for (name, bocpd) in self.detectors.iter_mut() {
            if let Some(&value) = metrics.get(name.as_str()) {
                let result = bocpd.update(value);
                if result.is_changepoint {
                    alerts.push(ChangePointAlert {
                        metric: name.clone(),
                        probability: result.changepoint_probability,
                        previous_regime_length: result.map_run_length,
                    });
                }
            }
        }
        alerts
    }
}
```

When BOCPD detects a regime change, the golem should:

1. **Increase exploration weight** in the online learner (boost epsilon, widen LinUCB confidence bounds)
2. **Accelerate Bayesian model decay** to shed stale beliefs faster
3. **Log the changepoint** as a Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) episode for generational memory
4. **Raise arousal** in CorticalState (32-signal atomic shared perception surface; the Golem's real-time self-model) to broaden attention scope

---

## PELT: retrospective changepoint detection

### When to use PELT vs. BOCPD

[SPEC] BOCPD is online -- it processes one observation at a time and maintains a running posterior. PELT (Pruned Exact Linear Time) is offline -- it takes a batch of data and finds the optimal segmentation into regimes. Use PELT at Delta tick cadence to validate and refine BOCPD's online detections.

PELT (Killick et al., 2012) uses dynamic programming with a pruning step that reduces the average time complexity from O(n^2) to O(n). The pruning is exact -- it provably doesn't discard any optimal segmentation.

### When PELT runs

At each Delta tick, the golem has accumulated ~20 minutes of Theta-tick aggregate metrics. PELT runs on this batch to:

- Confirm BOCPD detections (reduce false positives)
- Find changepoints that BOCPD missed (the online posterior can be slow to converge)
- Provide exact changepoint locations for Grimoire logging

```rust
/// PELT changepoint detection for batch analysis.
/// Runs at Delta tick on accumulated Theta metrics.
pub fn pelt_segmentation(
    data: &[f64],
    penalty: f64,
    min_segment_length: usize,
) -> Vec<usize> {
    let n = data.len();
    if n < 2 * min_segment_length {
        return vec![]; // Too short to segment
    }

    // cost[i][j] = cost of segment from i to j (exclusive)
    // Using L2 cost: sum of squared deviations from the segment mean
    let prefix_sum: Vec<f64> = std::iter::once(0.0)
        .chain(data.iter().scan(0.0, |acc, &x| { *acc += x; Some(*acc) }))
        .collect();
    let prefix_sq_sum: Vec<f64> = std::iter::once(0.0)
        .chain(data.iter().scan(0.0, |acc, &x| { *acc += x * x; Some(*acc) }))
        .collect();

    let segment_cost = |start: usize, end: usize| -> f64 {
        let n_seg = (end - start) as f64;
        let sum = prefix_sum[end] - prefix_sum[start];
        let sq_sum = prefix_sq_sum[end] - prefix_sq_sum[start];
        sq_sum - sum * sum / n_seg
    };

    // Dynamic programming with PELT pruning
    let mut opt_cost = vec![f64::MAX; n + 1];
    let mut changepoints = vec![0usize; n + 1];
    let mut candidates: Vec<usize> = vec![0];
    opt_cost[0] = -penalty; // So first segment pays penalty once

    for t in min_segment_length..=n {
        let mut best_cost = f64::MAX;
        let mut best_cp = 0;

        let mut next_candidates = Vec::new();
        for &s in &candidates {
            if t - s >= min_segment_length {
                let cost = opt_cost[s] + segment_cost(s, t) + penalty;
                if cost < best_cost {
                    best_cost = cost;
                    best_cp = s;
                }
                // PELT pruning: keep candidate only if it could still be optimal
                if cost <= opt_cost[t] + penalty {
                    next_candidates.push(s);
                }
            } else {
                next_candidates.push(s); // Not yet eligible, keep for later
            }
        }

        opt_cost[t] = best_cost;
        changepoints[t] = best_cp;
        next_candidates.push(t);
        candidates = next_candidates;
    }

    // Backtrack to find changepoint locations
    let mut result = Vec::new();
    let mut pos = n;
    while pos > 0 && changepoints[pos] > 0 {
        result.push(changepoints[pos]);
        pos = changepoints[pos];
    }
    result.reverse();
    result
}
```

---

## ADWIN: adaptive windowing

### Complementary to BOCPD

[SPEC] ADWIN (Bifet & Gavalda, 2007) maintains a variable-length sliding window over a data stream. When it detects a distribution change (using Hoeffding bounds), it shrinks the window by discarding old data from the changed distribution. The window automatically grows during stationary periods and shrinks at change points.

ADWIN runs at Theta tick, cheaper than BOCPD (O(log n) space vs. O(n) for BOCPD's run-length vector), making it suitable for monitoring many metrics simultaneously. Where BOCPD provides a full posterior over changepoint probabilities, ADWIN provides a binary signal: change detected or not.

```rust
/// ADWIN adaptive windowing for change detection.
/// Maintains a sliding window that shrinks when distribution change is detected.
pub struct Adwin {
    /// Compressed representation of the window using exponential histogram buckets
    buckets: Vec<AdwinBucket>,
    /// Total sum and count across all buckets
    total_sum: f64,
    total_count: usize,
    /// Confidence parameter (smaller = more sensitive to changes)
    delta: f64,
}

#[derive(Clone)]
struct AdwinBucket {
    sum: f64,
    count: usize,
}

impl Adwin {
    pub fn new(delta: f64) -> Self {
        Self {
            buckets: Vec::new(),
            total_sum: 0.0,
            total_count: 0,
            delta,
        }
    }

    /// Add a new value. Returns true if a distribution change was detected.
    pub fn update(&mut self, value: f64) -> bool {
        // Add new bucket
        self.buckets.push(AdwinBucket { sum: value, count: 1 });
        self.total_sum += value;
        self.total_count += 1;

        // Compress adjacent small buckets (exponential histogram maintenance)
        self.compress();

        // Check for change: try splitting the window at each bucket boundary
        self.detect_change()
    }

    /// Current mean of the window.
    pub fn mean(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }
        self.total_sum / self.total_count as f64
    }

    /// Current window length.
    pub fn window_length(&self) -> usize {
        self.total_count
    }

    fn compress(&mut self) {
        // Merge adjacent buckets when they're small relative to the window
        // This keeps O(log n) buckets total
        let mut i = self.buckets.len();
        while i > 1 {
            i -= 1;
            if i > 0
                && self.buckets[i].count == self.buckets[i - 1].count
                && self.buckets[i].count < self.total_count / 4
            {
                let merged_sum = self.buckets[i].sum + self.buckets[i - 1].sum;
                let merged_count = self.buckets[i].count + self.buckets[i - 1].count;
                self.buckets[i - 1] = AdwinBucket {
                    sum: merged_sum,
                    count: merged_count,
                };
                self.buckets.remove(i);
            }
        }
    }

    fn detect_change(&mut self) -> bool {
        if self.total_count < 10 {
            return false; // Need minimum data
        }

        // Try splitting window: old part (front) vs new part (back)
        let mut old_sum = 0.0;
        let mut old_count = 0usize;

        for split in 0..self.buckets.len().saturating_sub(1) {
            old_sum += self.buckets[split].sum;
            old_count += self.buckets[split].count;

            let new_sum = self.total_sum - old_sum;
            let new_count = self.total_count - old_count;

            if old_count < 5 || new_count < 5 {
                continue;
            }

            let old_mean = old_sum / old_count as f64;
            let new_mean = new_sum / new_count as f64;

            // Hoeffding bound for the difference of means
            let n = self.total_count as f64;
            let m = (1.0 / old_count as f64 + 1.0 / new_count as f64).sqrt();
            let epsilon = (2.0 / n * (4.0 / self.delta).ln()).sqrt() * m;

            if (old_mean - new_mean).abs() > epsilon {
                // Change detected: drop old data
                self.buckets = self.buckets[split + 1..].to_vec();
                self.total_sum = new_sum;
                self.total_count = new_count;
                return true;
            }
        }
        false
    }
}
```

### Where ADWIN fits

[SPEC] ADWIN monitors per-protocol metrics at Theta tick: average gas per transaction, event frequency, value distribution mean. It's lightweight enough to run one instance per monitored protocol (100 protocols * ~1KB each = 100KB). When ADWIN detects a change on a specific protocol, it triggers targeted exploration -- increase curiosity for that protocol without raising global arousal.

---

## Algorithm comparison

| Algorithm | Timescale | What it detects | Memory | Cost per update |
|---|---|---|---|---|
| MIDAS-R | Per block (~12s) | Graph microbursts between address pairs | 128KB fixed | O(1) |
| ADWIN | Per Theta tick (~5m) | Per-protocol distribution shifts | ~1KB per metric | O(log n) |
| BOCPD | Per Theta tick (~5m) | Global regime changes | O(max_run_length) per metric | O(max_run_length) |
| PELT | Per Delta tick (~20m) | Retrospective optimal segmentation | O(n) batch | O(n) amortized |

All four feed into the Hedge-weighted curiosity composite. MIDAS-R produces a per-transaction anomaly score. ADWIN and BOCPD produce regime-level signals that modulate overall curiosity sensitivity rather than scoring individual transactions.

---

## References

- Bhatia, S., Hooi, B., Yoon, M., Shin, K. & Faloutsos, C. (2020). "MIDAS: Microcluster-Based Detector of Anomalies in Edge Streams." *Proceedings of the AAAI Conference on Artificial Intelligence*, 34(04), 3242-3249. -- *The primary real-time graph anomaly detector; achieves 42-52% higher AUC than prior streaming methods in O(1) time and memory.*
- Adams, R.P. & MacKay, D.J.C. (2007). "Bayesian Online Changepoint Detection." arXiv:0710.3742. -- *Maintains a posterior over run lengths to detect when the data-generating distribution shifts; used for regime change detection at Theta tick cadence.*
- Killick, R., Fearnhead, P. & Eckley, I.A. (2012). "Optimal Detection of Changepoints with a Linear Computational Cost." *Journal of the American Statistical Association*, 107(500), 1590-1598. -- *PELT algorithm for retrospective changepoint detection with exact pruning; validates BOCPD's online detections at Delta tick.*
- Bifet, A. & Gavalda, R. (2007). "Learning from Time-Changing Data with Adaptive Windowing." *Proceedings of the 2007 SIAM International Conference on Data Mining (SDM)*, 443-448. -- *ADWIN adaptive windowing with Hoeffding bounds; lightweight per-protocol change detection in O(log n) space.*
- Cormode, G. & Muthukrishnan, S. (2005). "An Improved Data Stream Summary: The Count-Min Sketch and its Applications." *Journal of Algorithms*, 55(1), 58-75. -- *The probabilistic frequency estimator underlying MIDAS-R's edge and node counting.*
