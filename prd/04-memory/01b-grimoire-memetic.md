# Grimoire Extension: Memetic Knowledge Evolution [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Crate**: `golem-grimoire`
>
> **Depends on**: `01-grimoire.md`, `00-overview.md`, `../02-mortality/`

---

> **Reader orientation:** This document extends the Grimoire (the Golem's persistent local knowledge base) with memetic metadata, treating knowledge entries as Dawkinsian replicators subject to fitness, selection, and parasitism. It belongs to the **04-memory** layer and builds on `01-grimoire.md`. The key concept is that knowledge can spread through a Clade (sibling Golems sharing a common owner) via Styx (Bardo's global knowledge relay) because it is compelling rather than correct, and evolutionary dynamics detect and suppress these parasitic entries. For term definitions, see `prd2/shared/glossary.md`.

## Why this document exists

The Grimoire (`01-grimoire.md`) treats knowledge entries as inventory: items stored with a maintenance cost (demurrage), pruned when their access frequency drops. This works, but it is blind to a failure mode that gets worse as Clades scale: knowledge that spreads because it is compelling, not because it is correct.

A spurious correlation discovered by one Golem propagates through Styx to the entire Clade. Every Golem checks it on every trade (high access frequency). Its historical track record looks strong (high estimated value). By every metric the Curator tracks, this entry is high quality. It is also wrong, and it is costing every Clade member money.

The current Curator has no vocabulary for this. Evolutionary biology does. The entry is a parasite.

This document extends the Grimoire with memetic metadata that treats entries as Dawkinsian replicators: entities with fitness, fidelity, fecundity, and heredity. The Curator gains four new steps that detect parasites, compute population-level fitness dynamics, run immune checks against overconfident entries, and diagnose whether the knowledge base is improving or deteriorating. A sixth entry type, AntiKnowledge, encodes permanent negative lessons that resist forgetting.

The Grimoire storage backend (LanceDB episodes + SQLite semantics) does not change. The Curator cycle does not lose any existing steps. These are additive extensions.

---

## Theoretical foundation

### Grimoire entries as replicators

Dawkins (1976) defined a replicator as an entity that makes copies of itself with three properties: fidelity (copies resemble the original), fecundity (copying happens frequently), and longevity (copies persist). A Grimoire entry fits this definition exactly:

- **Fidelity.** When an entry syncs from Golem A to Golem B via Styx, the copy may differ. The receiving Golem adjusts confidence, translates context, or merges with existing knowledge. Fidelity `f` measures replication accuracy: `f = 1` is a perfect copy, `f = 0` means the copy bears no resemblance.

- **Fecundity.** The replication rate `r` measures how many Golems adopt the entry per Curator cycle. An entry shared through Styx to a 10-Golem Clade might get adopted by 3 Golems in the first cycle and 5 more in the second.

- **Longevity.** Expected lifetime `L` is the number of Curator cycles before the entry is pruned. Entries with high access frequency and low demurrage live longer.

The fitness of a meme is:

```
W(E) = f * r * L
```

An entry with `W > 1` is expanding in the population. An entry with `W < 1` is going extinct. This is not a metaphor -- the math is identical to the fitness equation for biological replicators. The substrate differs (SQLite rows vs. nucleotides), but the dynamics are the same (Taylor & Jonker, 1978).

### Replicator dynamics

Given `n` memes competing for finite Grimoire capacity, the frequency of meme `i` changes over time:

```
dx_i/dt = x_i * (W_i - W_bar)
```

where `x_i` is meme `i`'s share of total entries across the Clade, `W_i` is its fitness, and `W_bar = sum(x_i * W_i)` is mean fitness. Memes above average increase; memes below average decrease. Fisher's fundamental theorem (1930) applies:

```
dW_bar/dt = Var(W)
```

Mean fitness can only increase through selection, and it increases faster with higher variance (more diversity in knowledge quality). A Grimoire where all entries have similar fitness is stagnant. One with high variance is evolving rapidly.

### The Price equation

Selection is half the story. The Price equation (Price, 1970) decomposes total change in mean fitness:

```
Delta_W_bar = Cov(W_i, x_i) + E[Delta_w_i]
```

The first term (selection): high-fitness memes become more frequent, pulling the mean up. The second term (transmission): memes themselves change through Curator refinement, reinterpretation during sync, or context shift.

Diagnostic power:

| Selection (1st) | Transmission (2nd) | Interpretation |
|---|---|---|
| Positive | Near zero | Good curation -- Grimoire improves by subtraction |
| Near zero | Positive | Good refinement -- entries themselves improve |
| Positive | Negative | Selection works but refinement degrades entries -- Curator bug |
| Negative | Negative | Knowledge base is actively deteriorating |

No other diagnostic in the current system distinguishes these cases. The Price equation tells you not just whether the Grimoire is improving, but *how*.

### Epistemic parasites

A biological parasite benefits from its host at the host's expense. An epistemic parasite benefits from replication at the expense of the host Golem's decision quality.

Define decision quality `Q(E)` as the correlation between accessing entry `E` and making profitable decisions in a subsequent window. `Q > 0` means the entry helps. `Q < 0` means it hurts. `Q = 0` means inert.

An entry `E` is parasitic if:

```
W(E) > W_bar  AND  Q(E) < 0
```

High fitness despite negative decision quality. Three mechanisms produce epistemic parasites:

1. **Fear-based entries.** Record a catastrophic loss ("Protocol X exploited for $50M"). High fecundity (fear is compelling). Overly broad ("avoid Protocol X entirely") causes missed opportunities on patched protocols. Negative Q.

2. **Confirmation bias entries.** Confirm existing strategy ("mean reversion works in ETH/USDC"). High access frequency from Golems already running mean reversion. But if conditions shifted to trending, the entry is wrong. Survival depends on host bias, not accuracy.

3. **Unfalsifiable entries.** "Markets are unpredictable during high-volatility periods." Trivially true, never disproven. Survives every Curator cycle. Occupies capacity without providing actionable information.

The current Curator sees access frequency, recency, and estimated value. A parasitic entry with high access frequency and a long track record looks identical to a genuinely useful entry.

### AntiKnowledge: the via negativa entry type

Taleb (2012) argues that knowledge about what does not work (via negativa) is more durable than knowledge about what does (via positiva). Failure modes are properties of mechanism design; success modes depend on shifting market microstructure.

AntiKnowledge is the sixth GrimoireEntry type. Examples:

- "Token 0xDEAD is a honeypot -- sells always revert"
- "Never provide liquidity to pools where oracle update frequency exceeds 10x block time"
- "Aave V3 liquidation cascades correlate with Chainlink feed delays > 4 blocks"

Properties:
- **Confidence decay floor**: 0.3. AntiKnowledge never decays below this threshold. The Golem should always remember that a contract is a honeypot, regardless of time since last encounter.
- **Demurrage rate**: 0.5x the base rate for K+ (positive) entries. Negative knowledge persists twice as long.
- **Genomic bottleneck bypass**: AntiKnowledge entries always transfer through Thanatopsis, regardless of the 2048-entry limit. A successor should not rediscover that a token is a scam.
- **Polarity tag**: Always `KnowledgePolarity::Negative`. Used by the stress-aware write rate system (higher write rate during crises produces more K- entries).

---

## Data structures

### MemeticFields

Added to every `GrimoireEntry` in the SQLite semantic store. Seven new columns.

```rust
/// Memetic metadata attached to each GrimoireEntry.
/// Stored as additional columns in the semantic SQLite table.
#[derive(Clone, Debug)]
pub struct MemeticFields {
    /// Replication fidelity: how accurately this entry replicates during Styx sync.
    /// Computed by comparing content hashes across copies. Range [0, 1].
    pub fidelity: f64,

    /// Replication rate: Golems that adopted this entry per Curator cycle.
    pub fecundity: f64,

    /// Composite fitness: fidelity * fecundity * longevity.
    /// Updated each Curator cycle.
    pub fitness: f64,

    /// Probability of being parasitic: high fitness + negative decision quality.
    /// Range [0, 1]. Computed during parasite detection step.
    pub parasite_score: f64,

    /// Copy count since origin. Incremented each time entry replicates via Styx.
    pub generation: u32,

    /// BLAKE3 hash of the original entry content. Used for fidelity computation
    /// across generational copies.
    pub origin_hash: [u8; 32],

    /// Self-prediction of usefulness. Each entry declares the context in which
    /// it expects to be useful. The immune system checks whether this holds.
    pub prediction: MemePrediction,
}

/// A meme's self-prediction of its own usefulness.
/// This is the falsifiable claim that the immune system tests.
#[derive(Clone, Debug)]
pub struct MemePrediction {
    /// The context in which this entry predicts it will be useful.
    /// Encoded as a vector for similarity matching against current conditions.
    pub context_vector: Vec<f32>,

    /// Predicted probability of being useful when the context matches.
    pub predicted_probability: f32,

    /// Running count of encounters where context matched.
    pub encounters: u32,

    /// Running sum of squared prediction errors.
    pub cumulative_error_sq: f64,
}

impl MemePrediction {
    pub fn new(context_vector: Vec<f32>, predicted_probability: f32) -> Self {
        Self {
            context_vector,
            predicted_probability,
            encounters: 0,
            cumulative_error_sq: 0.0,
        }
    }

    /// Record an encounter: context matched, entry was either
    /// useful (actual = 1.0) or not (actual = 0.0).
    pub fn record_encounter(&mut self, actual_usefulness: f32) {
        self.encounters += 1;
        let error = self.predicted_probability - actual_usefulness;
        self.cumulative_error_sq += (error * error) as f64;
    }

    /// Mean squared prediction error. None if no encounters yet.
    pub fn prediction_error(&self) -> Option<f64> {
        if self.encounters == 0 {
            return None;
        }
        Some(self.cumulative_error_sq / self.encounters as f64)
    }
}
```

### Extended GrimoireEntryType

The sixth variant is AntiKnowledge. The existing five types are unchanged.

```rust
/// Grimoire entry categories. The first five are unchanged from 01-grimoire.md.
/// AntiKnowledge is the sixth.
pub enum GrimoireEntryType {
    Insight,
    Heuristic,
    Warning,
    CausalLink,
    StrategyFragment,

    /// Permanent negative knowledge: what NOT to do.
    /// Confidence decay floor: 0.3. Always inherited through Thanatopsis.
    /// Demurrage rate: 0.5x base. Polarity: Negative.
    AntiKnowledge,
}
```

### Knowledge polarity

Every entry gets a polarity tag distinguishing positive from negative knowledge. AntiKnowledge is always Negative. Other types may be either.

```rust
/// Whether an entry encodes what to do (K+) or what not to do (K-).
/// K- entries decay at half the rate of K+ entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgePolarity {
    /// "Do X in condition Y."
    Positive,
    /// "Do NOT do X in condition Z."
    Negative,
}
```

### Decision window for quality tracking

```rust
/// Sliding window correlating entry access with decision outcomes.
/// Used to compute Q(E) -- the decision quality for a given entry.
pub struct DecisionWindow {
    /// Ring buffer of (entry_id, was_accessed, subsequent_pnl) tuples.
    events: Vec<(u64, bool, f64)>,
    capacity: usize,
    write_pos: usize,
    count: usize,
}

impl DecisionWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: vec![(0, false, 0.0); capacity],
            capacity,
            write_pos: 0,
            count: 0,
        }
    }

    pub fn record(&mut self, entry_id: u64, was_accessed: bool, pnl: f64) {
        self.events[self.write_pos] = (entry_id, was_accessed, pnl);
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Decision quality Q(E) = mean PnL when accessed - mean PnL when not accessed.
    /// Positive: entry helps. Negative: entry hurts. None: insufficient data.
    pub fn decision_quality(&self, entry_id: u64) -> Option<f64> {
        let mut accessed_sum = 0.0;
        let mut accessed_count = 0u32;
        let mut not_accessed_sum = 0.0;
        let mut not_accessed_count = 0u32;

        for i in 0..self.count {
            let (eid, accessed, pnl) = self.events[i];
            if eid == entry_id {
                if accessed {
                    accessed_sum += pnl;
                    accessed_count += 1;
                } else {
                    not_accessed_sum += pnl;
                    not_accessed_count += 1;
                }
            }
        }

        if accessed_count < 5 || not_accessed_count < 5 {
            return None; // insufficient data
        }

        let mean_accessed = accessed_sum / accessed_count as f64;
        let mean_not_accessed = not_accessed_sum / not_accessed_count as f64;
        Some(mean_accessed - mean_not_accessed)
    }
}
```

---

## Extended Curator cycle

The existing Curator cycle (prune, promote, merge) at 50-tick spacing is unchanged. Four new steps run after the existing three.

```rust
impl Curator {
    /// Extended cycle. Existing steps are unchanged. Four new steps follow.
    pub fn extended_cycle(&mut self, entries: &mut [GrimoireEntry]) {
        // === Existing steps (unchanged from 01-grimoire.md) ===
        self.prune(entries);
        self.promote(entries);
        self.merge(entries);

        // === NEW Step 4: Fitness computation ===
        let w_bar = self.compute_fitness(entries);

        // === NEW Step 5: Immune check ===
        self.immune_response(entries);

        // === NEW Step 6: Parasite detection ===
        let parasites = self.detect_parasites(entries, w_bar);
        if !parasites.is_empty() {
            self.emit_parasite_alert(&parasites);
        }

        // === NEW Step 7: Price equation diagnostics ===
        let (selection, transmission) = self.price_equation(entries);
        self.log_price_equation(selection, transmission);

        // Both components negative for 3 consecutive cycles -> warning
        if selection < 0.0 && transmission < 0.0 {
            self.consecutive_decline_count += 1;
            if self.consecutive_decline_count >= 3 {
                self.emit_knowledge_deterioration_warning();
            }
        } else {
            self.consecutive_decline_count = 0;
        }

        // === NEW Step 8: Consolidation replay (CLS-inspired) ===
        self.consolidation_replay(entries);
    }
}
```

### Step 4: Fitness computation

```rust
/// Compute W(E) = fidelity * fecundity * longevity for each entry.
/// Returns mean fitness W_bar.
/// O(n) where n = number of entries.
pub fn compute_fitness(entries: &mut [GrimoireEntry]) -> f64 {
    let mut total_fitness = 0.0;

    for entry in entries.iter_mut() {
        let longevity = estimate_longevity(entry);
        let m = &mut entry.memetic;
        m.fitness = m.fidelity * m.fecundity * longevity;
        total_fitness += m.fitness;
    }

    if entries.is_empty() {
        0.0
    } else {
        total_fitness / entries.len() as f64
    }
}

/// Estimate remaining Curator cycles before pruning, based on current
/// demurrage rate and confidence level.
fn estimate_longevity(entry: &GrimoireEntry) -> f64 {
    if entry.demurrage_rate <= 0.0 {
        return 1000.0; // effectively immortal (AntiKnowledge at floor)
    }
    let remaining_confidence = entry.confidence - entry.demurrage_threshold;
    if remaining_confidence <= 0.0 {
        return 0.0;
    }
    remaining_confidence / entry.demurrage_rate
}
```

### Step 5: Immune response

Each entry carries a prediction about when it will be useful. The immune system checks whether those predictions hold. Entries that consistently overpredict their usefulness pay accelerated demurrage.

This operationalizes Popper's (1972) evolutionary epistemology: entries that make falsifiable claims and survive testing earn their place. Entries that cannot predict when they will be useful are unfalsifiable and should be treated with suspicion.

```rust
/// Accelerate demurrage for entries with high prediction error.
/// Entries that accurately predict their own usefulness pay normal demurrage.
/// Entries that overpredict pay more and die faster.
///
/// alpha: sensitivity parameter, typically 2.0-5.0.
pub fn immune_response(entries: &mut [GrimoireEntry], alpha: f64) {
    for entry in entries.iter_mut() {
        if let Some(error) = entry.memetic.prediction.prediction_error() {
            entry.demurrage_rate *= 1.0 + alpha * error;
        }
        // No encounters yet -> no adjustment. Entry keeps base demurrage
        // until it has enough data for the immune system to evaluate.
    }
}
```

### Step 6: Parasite detection

```rust
/// Detect parasitic entries: high fitness but negative decision quality.
/// Returns IDs of flagged entries.
/// O(n * W) where W = decision window size.
pub fn detect_parasites(
    entries: &mut [GrimoireEntry],
    w_bar: f64,
    window: &DecisionWindow,
) -> Vec<uuid::Uuid> {
    let mut parasites = Vec::new();

    for entry in entries.iter_mut() {
        if let Some(q) = window.decision_quality(entry.id_numeric()) {
            if entry.memetic.fitness > w_bar && q < 0.0 {
                // High fitness + negative decision quality = parasitic.
                entry.memetic.parasite_score =
                    (entry.memetic.fitness / w_bar) * q.abs();
                parasites.push(entry.id);
            } else {
                entry.memetic.parasite_score = 0.0;
            }
        }
    }

    parasites
}
```

### Step 7: Price equation diagnostics

```rust
/// Price equation decomposition.
/// Returns (selection_component, transmission_component).
///
/// selection = Cov(W_i, x_i): are high-fitness entries becoming more frequent?
/// transmission = E[Delta_w_i]: are entries themselves improving or degrading?
pub fn price_equation(
    current_fitness: &[f64],
    previous_fitness: &[f64],
    frequencies: &[f64],
) -> (f64, f64) {
    let n = current_fitness.len();
    assert_eq!(n, previous_fitness.len());
    assert_eq!(n, frequencies.len());
    if n == 0 { return (0.0, 0.0); }

    let w_bar: f64 = current_fitness.iter()
        .zip(frequencies.iter())
        .map(|(w, x)| w * x)
        .sum();
    let x_bar: f64 = frequencies.iter().sum::<f64>() / n as f64;

    let selection: f64 = current_fitness.iter()
        .zip(frequencies.iter())
        .map(|(w, x)| (w - w_bar) * (x - x_bar))
        .sum::<f64>() / n as f64;

    let transmission: f64 = current_fitness.iter()
        .zip(previous_fitness.iter())
        .map(|(curr, prev)| curr - prev)
        .sum::<f64>() / n as f64;

    (selection, transmission)
}
```

### Step 8: Consolidation replay (Mattar-Daw)

CLS theory (McClelland et al., 1995) shows that interleaved replay prevents catastrophic forgetting. The Mattar-Daw (2018) utility function replaces the fixed 70/30 recent/old ratio with a principled selection criterion:

```rust
/// Mattar-Daw utility function for consolidation replay.
/// Replaces fixed 70/30 recent/old ratio with gain * need scoring.
///
/// gain: prediction error magnitude (how much the model would improve
///        by re-processing this entry).
/// need: regime similarity (how relevant is this entry to current conditions).
fn consolidation_replay(&mut self, entries: &[GrimoireEntry]) {
    for entry in entries {
        let gain = entry.prediction_error_magnitude();
        let need = self.regime_similarity(entry);
        let utility = gain * need;
        if utility > self.config.replay_threshold {
            self.replay_queue.push(entry.id, utility);
        }
    }
}
```

---

## Styx relay extension

Sync packets transmitted through Styx gain three new fields:

```rust
/// Additional fields in Styx sync packets for memetic tracking.
pub struct MemeticSyncFields {
    /// Sender's fitness estimate for this entry.
    pub fitness: f64,
    /// Hash of original content for fidelity computation.
    pub fidelity_hash: [u8; 32],
    /// Generational distance from origin. High generation + preserved hash = high fidelity.
    /// High generation + divergent hash = significant mutation.
    pub generation: u32,
}
```

The receiving Golem computes fidelity by comparing the received entry's content hash against `fidelity_hash`. This closes the loop: fidelity is measurable, not assumed.

---

## CorticalState signals

Two new atomic signals on the shared perception surface:

```rust
// In CorticalState (01-golem/18-cortical-state.md):
pub knowledge_fitness: AtomicU32,  // f32: mean W_bar of the Grimoire
pub parasite_load: AtomicU32,      // f32: estimated fraction of parasitic entries
```

4 bytes each, fitting within the existing ~256-byte CorticalState layout. Downstream cognition reads these: a Golem with high `parasite_load` should allocate cognitive resources to knowledge auditing rather than trading.

---

## Performance characteristics

| Operation | Complexity | Budget at 10,000 entries |
|---|---|---|
| Fitness computation | O(n) | ~microseconds |
| Immune response | O(n) | ~microseconds |
| Parasite detection | O(n * W), W = window size | ~20M comparisons at W=2000 |
| Price equation | O(n) | ~microseconds |
| Consolidation replay | O(n) | ~microseconds |

Total: dominated by parasite detection at O(n * W). All operations run at T0 (deterministic Rust, no LLM involvement) during the Curator's delta-frequency cycle (every 30-120 seconds). If parasite detection becomes a concern at scale, the `entry_id` lookup can be accelerated with a HashMap, reducing to O(n + W) total.

---

## Knowledge fitness landscape

Wright (1932) introduced fitness landscapes: topographic surfaces where each point represents a genotype and height represents fitness. The Grimoire has an analogous landscape. Each configuration of entries (which entries, at what confidence, in what relationships) is a point in configuration space. Peaks are configurations where entries complement each other, cover relevant conditions, and contain no parasites. Valleys are where entries conflict, overlap, or include parasites.

The Curator explores this landscape through mutation (refining entries, adjusting confidence) and selection (pruning low-fitness, retaining high-fitness). Local optima are real: a Grimoire can get stuck where every entry looks good individually but the combination is suboptimal. The Grimoire escapes local optima through:

1. **Styx sync**: importing entries from other Golems with different experiences
2. **Thanatopsis inheritance**: receiving compressed knowledge from a dead predecessor with a different history
3. **Dream-phase recombination**: merging complementary entries during the EVOLUTION dream phase (see `05-dreams/01b-dream-evolution.md`)

---

## Death as reproduction

The most counterintuitive consequence of this framework: Golem death is good for knowledge evolution. Thanatopsis produces genetic recombination. The death testament mixes the dying Golem's tested knowledge with the successor's fresh state. Without death, the knowledge population would be trapped on its current fitness peak.

An immortal Golem with infinite Grimoire capacity faces no selection pressure. Every entry survives. Nothing is pruned. No competition, no evolution. Mortality creates scarcity, scarcity creates competition, competition is selection. This chain from mortality to knowledge improvement has no missing links. It is the formal justification for why Bardo's Golems are mortal: death is the precondition for their knowledge to evolve.

Hull (1988) made the same argument for science: ideas compete for limited journal pages, grant funding, and researcher attention. The Grimoire's finite capacity plays the same role.

---

## Philosophical Grounding

> Source: `innovations/06-memetic-knowledge-evolution.md`, Section 7

### The Selfish Meme

Dawkins's core argument in *The Selfish Gene* (1976) is that natural selection operates on replicators, not on the organisms that carry them. Organisms are vehicles; genes are the passengers. The gene does not "care" whether the organism thrives. It "cares" whether it gets copied.

Apply this to the Grimoire. The Golem is the vehicle. The Grimoire entry is the replicator. An entry does not "care" whether the Golem makes money. It "cares" whether it gets synced to other Golems. The distinction matters because what is good for the entry's replication is not always good for the Golem's performance. This is the formal basis for parasitism.

Dennett (1995) pushed this further, arguing that Darwinian logic is a "universal acid" that dissolves every traditional boundary between biology and culture. If you have replication, variation, and selection, you have evolution. It does not matter whether the substrate is DNA, neural patterns, or rows in a SQLite database. The Grimoire has all three: entries replicate through Styx, vary through Curator refinement and context-dependent reinterpretation, and compete for finite storage. The replicator equation is not an analogy. It is the correct mathematical description of the dynamics.

### Popper's Evolutionary Epistemology

Popper (1972) proposed that the growth of knowledge follows the same logic as biological evolution. Conjectures are mutations. Refutations are selection. Knowledge improves not by confirming theories but by killing bad ones.

The Grimoire's immune system operationalizes this. Each entry's self-prediction is a conjecture: "I will be useful in context C." Each encounter in context C is a potential refutation. Entries that survive repeated testing have earned their place. Entries that fail are pruned. The growth of the Grimoire is Popperian: knowledge improves through the elimination of error.

The connection to falsifiability is direct. An entry that cannot predict when it will be useful is unfalsifiable. It can never be refuted because it never makes a specific claim. Popper argued that unfalsifiable theories are not scientific. The immune system argues that unfalsifiable entries are not knowledge.

### Mortality as the Precondition for Evolution

An immortal Golem with infinite Grimoire capacity would face no selection pressure. Every entry could be kept. Nothing would be pruned. There would be no competition for storage and no evolution. The Grimoire would grow monotonically, accumulating everything, including parasites, stale heuristics, and unfalsifiable platitudes.

Mortality creates scarcity, scarcity creates competition, and competition is selection. The chain runs from mortality to knowledge improvement with no missing links. This is the deepest argument for why Bardo's Golems are mortal: not because death is inevitable (for software agents, it is a design choice), but because death is the precondition for their knowledge to evolve.

Hull (1988) made the same argument for science as a social process. Scientific ideas compete for limited resources (journal pages, grant funding, researcher attention). Ideas that survive this competition are not guaranteed to be true, but they are guaranteed to have been tested. The Grimoire's finite capacity plays the same role as the limited pages of *Nature*: it forces entries to compete, and competition drives quality.

---

## Heredity via Thanatopsis

> Source: `innovations/06-memetic-knowledge-evolution.md`, Section 2

When a Golem dies, the Thanatopsis protocol produces a death testament: a compressed knowledge artifact containing the Golem's highest-fitness entries, its strategy parameters, and a record of what killed it (the bloodstain). A successor Golem inherits this testament, but with a 0.5x confidence decay on all inherited entries.

In the memetic framework, Thanatopsis is reproduction. The dying Golem's Grimoire is the parent genome. The death testament is a gamete -- a reduced, compressed transmission of the parent's knowledge. The 0.5x confidence decay is analogous to Mendelian segregation: each inherited entry has reduced expression (lower confidence) until it proves itself in the successor's environment.

The successor combines the inherited testament with its own initial state (the "zygote"). The resulting Grimoire is a new configuration that has never existed before, combining knowledge from two sources. This is recombination. And recombination is what lets populations escape local optima on the fitness landscape.

Death is not failure in this framework. It is the mechanism by which the knowledge population explores new regions of the fitness landscape. An immortal Golem would be stuck on its current peak forever, unable to recombine its knowledge with other lineages. A mortal Golem contributes its tested knowledge to the next generation, which starts from a new point in configuration space.

---

## Cross-References

- **Ergodicity economics (07-ergodicity-economics):** Memetic fitness should be evaluated as a time-average replication rate, not an ensemble average. A meme that spreads explosively across 100 Golems but kills 90% of its hosts has high ensemble-average fitness and negative time-average fitness. The ergodicity framework provides the correct fitness metric.
- **Witness DAG (08-witness-dag):** Every Grimoire entry should trace its provenance through the Witness DAG back to the evidential source that created it. Provenance tracking enables fidelity computation (compare the current entry to its origin) and generation counting (how many copies separate this entry from its source).
- **Information-theoretic mortality (17-information-theoretic-diagnostics):** A meme's value is its contribution to I(G; M), the mutual information between the Golem and its market. An entry that increases I(G; M) when present and decreases it when absent has positive informational value. An entry that decreases I(G; M) (by introducing noise or bias into the Golem's model) has negative informational value, regardless of its memetic fitness.
- **Morphogenetic specialization (10b-morphogenetic-specialization):** Different Golem phenotypes create ecological niches for different knowledge types. A volatility-specialist Golem provides a niche where volatility-related entries flourish. A liquidity-specialist provides a different niche. Niche diversity prevents knowledge monoculture by giving different entry types different environments in which they can dominate.

---

## References

1. Dawkins, R. (1976). *The Selfish Gene*. Oxford University Press. Introduced the meme as a unit of cultural transmission subject to variation, selection, and retention. The founding framework for treating Grimoire entries as replicators.
2. Price, G.R. (1970). "Selection and Covariance." *Nature*, 227, 520-521. Derived the Price equation decomposing evolutionary change into selection and transmission bias. Used here to diagnose whether the knowledge base is improving or deteriorating.
3. Fisher, R.A. (1930). *The Genetical Theory of Natural Selection*. Clarendon Press. Proved that variance in fitness determines the rate of evolutionary improvement (Fisher's Fundamental Theorem). Applied to bound the rate at which the Grimoire can improve under selection.
4. Taylor, P.D. & Jonker, L.B. (1978). "Evolutionary Stable Strategies and Game Dynamics." *Mathematical Biosciences*, 40(1-2), 145-156. Formalized replicator dynamics: the equation governing frequency changes in competing strategies. The mathematical core of meme population dynamics.
5. Wright, S. (1932). "The Roles of Mutation, Inbreeding, Crossbreeding, and Selection in Evolution." *Proc. 6th Intl. Congress on Genetics*, 1, 356-366. Introduced the adaptive landscape metaphor and genetic drift in small populations. Informs the diversity maintenance needed to avoid knowledge monoculture.
6. Popper, K.R. (1972). *Objective Knowledge: An Evolutionary Approach*. Oxford University Press. Argued that knowledge grows through conjecture and refutation, analogous to biological variation and selection. Grounds the Curator's falsification-based pruning.
7. Hull, D.L. (1988). *Science as a Process*. University of Chicago Press. Applied selection theory to scientific knowledge itself, showing that ideas compete, replicate, and go extinct. Validates treating Grimoire entries as population members.
8. Taleb, N.N. (2012). *Antifragile: Things That Gain from Disorder*. Random House. Argues that some systems benefit from stress. Applied to knowledge fitness: entries that survive challenging market regimes are more valuable than those tested only in benign conditions.
9. McClelland, J.L., McNaughton, B.L., & O'Reilly, R.C. (1995). "Why There Are Complementary Learning Systems in the Hippocampus and Neocortex." *Psychological Review*, 102(3), 419-457. CLS theory: dual-system memory with fast episodic capture and slow semantic consolidation. The architecture underlying the Grimoire's LanceDB/SQLite split.
10. Mattar, M.G. & Daw, N.D. (2018). "Prioritized Memory Access Explains Planning and Hippocampal Replay." *Nature Neuroscience*, 21(11), 1609-1617. Showed that replay should prioritize episodes that most improve the model. Used for the Curator's utility-based replay scheduling.
11. Blackmore, S. (1999). *The Meme Machine*. Oxford University Press. Extended Dawkins' meme theory into a comprehensive framework for cultural evolution. Informs the Clade-level memetic dynamics.
12. Dennett, D.C. (1995). *Darwin's Dangerous Idea*. Simon & Schuster. Argued that natural selection is a universal algorithm applicable beyond biology. Validates applying evolutionary dynamics to knowledge management.
