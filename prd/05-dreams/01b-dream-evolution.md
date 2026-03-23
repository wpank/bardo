# Dream Extension: EVOLUTION Phase [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `01-architecture.md`, `../04-memory/01b-grimoire-memetic.md`, `../04-memory/01c-grimoire-hdc.md`

---

> **Reader orientation:** This document specifies the EVOLUTION phase, a fourth dream phase that applies evolutionary optimization to the Grimoire's (persistent knowledge base's) knowledge population and the Golem's (mortal autonomous agent's) strategy vector. Part of the Dreams track within Bardo (the Rust runtime for mortal autonomous DeFi agents). It bridges memetic evolution (knowledge entries as replicators) and HDC (hyperdimensional computing) dream seeds. Prerequisites: dream architecture (`01-architecture.md`) and the Grimoire memetic model (`../04-memory/01b-grimoire-memetic.md`). For a full glossary, see `prd2/shared/glossary.md`.

## Why this document exists

The dream engine (`01-architecture.md`) has three core phases: NREM (replay past events for consolidation), REM (imagine counterfactual scenarios), and Integration (consolidate into PLAYBOOK.md and Grimoire). These phases process individual experience. No phase applies evolutionary optimization to the Grimoire's knowledge population or the Golem's strategy vector.

Two research threads converge on the need for a fourth phase:

1. **Memetic evolution** (`04-memory/01b-grimoire-memetic.md`): Grimoire entries are replicators with fitness, selection pressure, and heredity. The Curator's extended cycle runs fitness computation, immune checks, parasite detection, and Price equation diagnostics. During waking ticks, these operations compete for cognitive budget. During dreams, they can run without time pressure.

2. **Strategy evolution**: The strategy concentration vector (what mix of DeFi strategies the Golem runs) adapts during waking operation through performance feedback. But waking adaptation is conservative -- the Golem cannot afford to explore radically different strategy mixes with real capital. Dreams provide a safe environment to evolve strategy parameters using imagined returns from REM counterfactuals rather than real ones.

EVOLUTION is the 4th dream phase. It runs after REM and before Integration. The three existing phases are unchanged in behavior.

---

## Phase ordering

```
ONSET -> NREM -> REM -> EVOLUTION -> INTEGRATION -> RETURN
```

| Phase | Function | Budget (Thriving) | Budget (Terminal) |
|---|---|---|---|
| ONSET (Hypnagogic) | Context dissolve, waking residue | 10% | 10% |
| NREM | Replay, compress, credit assignment | 34% | 16% |
| REM | Counterfactual imagination, creative recombination | 25% | 10% |
| **EVOLUTION** | **Memetic selection, strategy evolution, knowledge recombination** | **5%** | **2%** |
| INTEGRATION | Consolidate into PLAYBOOK.md, Grimoire, dream journal | 21% | 57% |
| RETURN (Hypnopompic) | Recrystallize, Dali interrupt | 5% | 5% |

EVOLUTION gets a small budget because its operations are computationally cheap (T0 deterministic Rust, no LLM). The value comes from running the Curator's extended cycle without competing for waking cognitive budget, and from using REM's counterfactual outputs as inputs to strategy evolution.

---

## EVOLUTION phase activities

Three activities run during EVOLUTION, each drawing on outputs from the preceding REM phase.

### Activity 1: Memetic selection (Variation, Selection, Inheritance)

Run one iteration of the Curator's extended cycle from `01b-grimoire-memetic.md`. During waking operation, this cycle shares the Delta-tick budget with other maintenance tasks. During dreams, it runs with full attention.

```rust
/// EVOLUTION phase: memetic selection on the Grimoire.
/// Runs the same logic as the waking Curator's extended cycle,
/// but during dream time with no cognitive budget competition.
pub struct MemeticEvolutionDream {
    /// The Curator's extended cycle engine.
    curator: CuratorExtendedCycle,
}

impl MemeticEvolutionDream {
    /// Run one evolutionary iteration on the Grimoire.
    ///
    /// Three sub-steps map directly to the biological trio:
    /// 1. Variation: mutate pattern parameters of entries flagged
    ///    for exploration. Generate candidate variants.
    /// 2. Selection: compute fitness, run immune checks, detect parasites.
    ///    Entries below fitness threshold get accelerated demurrage.
    /// 3. Inheritance: high-fitness entries survive into the next
    ///    dream cycle. Their parameters propagate to variants.
    pub fn evolve(&mut self, grimoire: &mut Grimoire) -> EvolutionReport {
        let mut report = EvolutionReport::default();

        // --- Variation ---
        // Take the top-K entries by fitness variance (entries whose
        // fitness has changed most recently -- they are on the move
        // in the fitness landscape and worth exploring).
        let candidates = self.select_variation_candidates(grimoire);
        let variants = self.generate_variants(&candidates);
        report.variants_generated = variants.len();

        // --- Selection ---
        // Compute fitness for all entries including new variants.
        let w_bar = self.curator.compute_fitness(grimoire);
        self.curator.immune_response(grimoire);
        let parasites = self.curator.detect_parasites(grimoire, w_bar);
        report.parasites_detected = parasites.len();

        // Price equation diagnostics.
        let (selection, transmission) = self.curator.price_equation(grimoire);
        report.selection_component = selection;
        report.transmission_component = transmission;

        // --- Inheritance ---
        // High-fitness variants get integrated into the Grimoire.
        // Low-fitness variants are discarded before they consume capacity.
        let survivors = self.filter_survivors(&variants, w_bar);
        report.variants_surviving = survivors.len();
        for survivor in survivors {
            grimoire.insert_entry(survivor);
        }

        report
    }

    /// Select entries with high fitness variance -- the moving frontier.
    fn select_variation_candidates(
        &self,
        grimoire: &Grimoire,
    ) -> Vec<GrimoireEntry> {
        grimoire.entries()
            .filter(|e| e.memetic.fitness > 0.0)
            .sorted_by(|a, b| {
                let a_var = (a.memetic.fitness - a.previous_fitness).abs();
                let b_var = (b.memetic.fitness - b.previous_fitness).abs();
                b_var.partial_cmp(&a_var).unwrap()
            })
            .take(10)
            .cloned()
            .collect()
    }

    /// Generate variants by mutating entry parameters.
    /// Mutation types: confidence perturbation, context expansion,
    /// condition threshold adjustment.
    fn generate_variants(
        &self,
        candidates: &[GrimoireEntry],
    ) -> Vec<GrimoireEntry> {
        candidates.iter()
            .flat_map(|entry| {
                vec![
                    self.mutate_confidence(entry, 0.1),
                    self.mutate_context(entry),
                ]
            })
            .collect()
    }

    /// Keep variants with fitness above population mean.
    fn filter_survivors(
        &self,
        variants: &[GrimoireEntry],
        w_bar: f64,
    ) -> Vec<GrimoireEntry> {
        variants.iter()
            .filter(|v| v.memetic.fitness >= w_bar)
            .cloned()
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct EvolutionReport {
    pub variants_generated: usize,
    pub parasites_detected: usize,
    pub variants_surviving: usize,
    pub selection_component: f64,
    pub transmission_component: f64,
}
```

### Activity 2: Strategy evolution via imagined returns

During REM, the dream engine generates counterfactual scenarios: "what if the market had moved differently?" These counterfactuals produce imagined returns for different strategy configurations. EVOLUTION uses those imagined returns to evolve the strategy concentration vector without risking real capital.

```rust
/// Strategy evolution using REM counterfactual outputs.
/// The activation term in the strategy update uses IMAGINED returns
/// from REM rather than real returns. This lets the strategy vector
/// explore configurations that would be too risky to test live.
pub struct StrategyEvolutionDream {
    /// Current strategy concentrations.
    /// Each element represents the fraction of cognitive/capital
    /// budget allocated to a strategy class.
    pub concentrations: Vec<f64>,
    /// Strategy labels for interpretability.
    pub labels: Vec<String>,
}

impl StrategyEvolutionDream {
    /// Run one step of strategy evolution using imagined returns.
    ///
    /// rem_outputs: counterfactual scenario results from the preceding REM phase.
    /// Each output maps strategy labels to imagined PnL.
    pub fn evolve_step(
        &mut self,
        rem_outputs: &[CounterfactualResult],
    ) -> StrategyEvolutionReport {
        let mut report = StrategyEvolutionReport::default();

        // Compute imagined fitness for each strategy class.
        let mut imagined_fitness: Vec<f64> = vec![0.0; self.concentrations.len()];
        let mut counts: Vec<usize> = vec![0; self.concentrations.len()];

        for output in rem_outputs {
            for (i, label) in self.labels.iter().enumerate() {
                if let Some(&pnl) = output.strategy_returns.get(label) {
                    imagined_fitness[i] += pnl;
                    counts[i] += 1;
                }
            }
        }

        // Average imagined fitness per strategy.
        for i in 0..imagined_fitness.len() {
            if counts[i] > 0 {
                imagined_fitness[i] /= counts[i] as f64;
            }
        }

        // Replicator dynamics on concentrations.
        let mean_fitness: f64 = imagined_fitness.iter()
            .zip(self.concentrations.iter())
            .map(|(f, c)| f * c)
            .sum();

        for i in 0..self.concentrations.len() {
            let delta = self.concentrations[i]
                * (imagined_fitness[i] - mean_fitness)
                * 0.01; // small step size for stability
            self.concentrations[i] = (self.concentrations[i] + delta).max(0.01);
        }

        // Renormalize to sum to 1.0.
        let total: f64 = self.concentrations.iter().sum();
        for c in &mut self.concentrations {
            *c /= total;
        }

        report.updated_concentrations = self.concentrations.clone();
        report.mean_imagined_fitness = mean_fitness;
        report
    }
}

pub struct CounterfactualResult {
    /// Map of strategy label -> imagined PnL from this counterfactual scenario.
    pub strategy_returns: HashMap<String, f64>,
}

#[derive(Debug, Default)]
pub struct StrategyEvolutionReport {
    pub updated_concentrations: Vec<f64>,
    pub mean_imagined_fitness: f64,
}
```

### Activity 3: Knowledge recombination

Wright's recombination mechanism for escaping local optima on the fitness landscape. Merge complementary entries that were individually useful but never tested together.

```rust
/// Knowledge recombination: merge complementary entries.
/// This is Wright's mechanism for escaping local optima.
///
/// Two entries are candidates for recombination if:
/// 1. Both have above-average fitness.
/// 2. They cover different contexts (low HDC similarity between context vectors).
/// 3. They have never been accessed in the same tick (never tested together).
pub struct KnowledgeRecombinator;

impl KnowledgeRecombinator {
    /// Find and recombine complementary entry pairs.
    pub fn recombine(
        &self,
        entries: &[GrimoireEntry],
        item_memory: &mut ItemMemory,
        w_bar: f64,
    ) -> Vec<GrimoireEntry> {
        let high_fitness: Vec<&GrimoireEntry> = entries.iter()
            .filter(|e| e.memetic.fitness > w_bar)
            .collect();

        let mut recombined = Vec::new();

        for i in 0..high_fitness.len() {
            for j in (i + 1)..high_fitness.len() {
                let a = high_fitness[i];
                let b = high_fitness[j];

                // Check context dissimilarity (they cover different ground).
                let context_sim = cosine_similarity(
                    &a.memetic.prediction.context_vector,
                    &b.memetic.prediction.context_vector,
                );
                if context_sim > 0.5 {
                    continue; // too similar, recombination adds nothing
                }

                // Check they have not been co-accessed (never tested together).
                if a.co_access_count(b.id) > 0 {
                    continue; // already tested together, skip
                }

                // Create recombined entry.
                let combined = GrimoireEntry {
                    category: GrimoireEntryType::StrategyFragment,
                    content: format!(
                        "RECOMBINED: [{}] AND [{}]",
                        a.content_summary(), b.content_summary()
                    ),
                    confidence: 0.3, // low initial confidence, must prove itself
                    memetic: MemeticFields {
                        fitness: (a.memetic.fitness + b.memetic.fitness) / 2.0,
                        generation: a.memetic.generation.max(b.memetic.generation) + 1,
                        ..MemeticFields::default()
                    },
                    source: EntrySource::DreamRecombination,
                    ..GrimoireEntry::new()
                };
                recombined.push(combined);
            }
        }

        // Cap recombination output to prevent Grimoire bloat.
        recombined.truncate(5);
        recombined
    }
}
```

---

## HDC dream seeds for cross-dream pattern matching

Each dream scenario produces a transaction fingerprint HV (see `../04-memory/01c-grimoire-hdc.md` for the HDC substrate). These fingerprints enable cross-dream correlation: patterns that recur across dream sessions are candidates for promotion to the waking Grimoire.

```rust
/// Fingerprint a dream scenario output for later correlation.
/// Called after each REM and EVOLUTION phase scenario completes.
pub fn fingerprint_dream_output(
    dream_result: &DreamScenarioResult,
    encoder: &mut TxHvEncoder,
) -> Hypervector {
    let mut acc = BundleAccumulator::new();

    // Encode the scenario's market conditions.
    let regime_hv = encoder.encode_regime(&dream_result.regime);
    acc.add(&regime_hv);

    // Encode the strategy that was tested.
    let strategy_hv = encoder.encode_strategy(&dream_result.strategy);
    acc.add(&strategy_hv);

    // Encode the outcome.
    let outcome_hv = encoder.encode_outcome(dream_result.pnl);
    acc.add(&outcome_hv);

    acc.finish()
}

/// Track dream-to-reality correlation.
/// Stores dream fingerprints and checks incoming real events for matches.
pub struct DreamCorrelationTracker {
    /// Recent dream fingerprints: (dream_tick, fingerprint, metadata).
    dream_fingerprints: VecDeque<(u64, Hypervector, DreamMetadata)>,
    /// Maximum number of dream fingerprints to retain.
    correlation_window: usize,
}

impl DreamCorrelationTracker {
    pub fn new(window: usize) -> Self {
        Self {
            dream_fingerprints: VecDeque::with_capacity(window),
            correlation_window: window,
        }
    }

    /// Store a dream's fingerprint for later correlation.
    pub fn record_dream(
        &mut self,
        tick: u64,
        fingerprint: Hypervector,
        metadata: DreamMetadata,
    ) {
        if self.dream_fingerprints.len() == self.correlation_window {
            self.dream_fingerprints.pop_front();
        }
        self.dream_fingerprints.push_back((tick, fingerprint, metadata));
    }

    /// Check if a real event matches any recent dream output.
    /// High similarity means the dream "predicted" something.
    /// Returns matching dreams sorted by similarity.
    pub fn correlate(
        &self,
        real_event_hv: &Hypervector,
        threshold: f32,
    ) -> Vec<(u64, f32, &DreamMetadata)> {
        let mut matches: Vec<_> = self.dream_fingerprints.iter()
            .map(|(tick, hv, meta)| (*tick, hv.similarity(real_event_hv), meta))
            .filter(|(_, sim, _)| *sim > threshold)
            .collect();
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        matches
    }
}

pub struct DreamMetadata {
    pub phase: DreamPhase,
    pub scenario_type: String,
    pub strategy_tested: String,
    pub imagined_pnl: f64,
}
```

### Dream-prediction feedback loop

The correlation tracker closes a loop that does not currently exist:

1. Dream engine produces scenario outputs during REM and EVOLUTION phases.
2. Each output is fingerprinted and stored with a timestamp.
3. When a real event arrives during waking operation, its fingerprint is compared against recent dream outputs.
4. If similarity exceeds threshold (default 0.6), the dream is tagged as "predictive."
5. Predictive dreams increase the scheduling weight for similar dream topics in subsequent sleep cycles.

This means dreams that turn out to be prophetic -- that generate scenarios similar to events that actually occur -- get replayed more often. The dream engine learns what is worth dreaming about.

---

## Integration with existing dream phases

EVOLUTION does not modify NREM, REM, or Integration behavior. The data flow is:

1. **NREM** produces replay batches with credit assignment. These are inputs to Integration.
2. **REM** produces counterfactual scenarios with imagined returns. These are inputs to both Integration AND EVOLUTION.
3. **EVOLUTION** uses REM's counterfactual outputs for strategy evolution. It runs the Curator's memetic cycle for knowledge selection. It recombines complementary entries.
4. **Integration** consolidates all outputs from NREM, REM, and EVOLUTION into the Grimoire and PLAYBOOK.md.

The key dependency: EVOLUTION must run after REM because it uses REM's counterfactual results. It must run before Integration because its outputs (new variants, recombined entries, updated strategy concentrations) need to be consolidated.

---

## Performance characteristics

| Operation | Cost | Budget impact |
|---|---|---|
| Memetic selection (fitness + immune + parasite) | O(n), ~microseconds | Negligible |
| Strategy evolution step | O(strategies * counterfactuals) | Negligible |
| Knowledge recombination | O(k^2), k = high-fitness entries | ~milliseconds |
| Dream fingerprinting | ~100ns per scenario | Negligible |
| Dream correlation check | ~10ns per stored dream | Negligible |

All operations are T0 deterministic Rust. No LLM calls. EVOLUTION's 5% budget allocation (Thriving phase) is generous for the compute required. The budget is a time allocation within the dream cycle, not a compute budget.

---

## References

1. Dawkins, R. (1976). *The Selfish Gene*. Oxford University Press.
2. Wright, S. (1932). "The Roles of Mutation, Inbreeding, Crossbreeding, and Selection in Evolution."
3. Deperrois, N., et al. (2022). "Learning cortical representations through perturbed and adversarial dreaming." *eLife*.
4. Walker, M.P. & van der Helm, E. (2009). "Overnight therapy? The role of sleep in emotional brain processing." *Psychological Bulletin*, 135(5), 731-748.
5. Mattar, M.G. & Daw, N.D. (2018). "Prioritized Memory Access Explains Planning and Hippocampal Replay." *Nature Neuroscience*, 21(11), 1609-1617.
6. Kanerva, P. (2009). "Hyperdimensional Computing." *Cognitive Computation*, 1(2), 139-159.
7. Gayler, R.W. (2004). "Vector Symbolic Architectures." *Behavioral and Brain Sciences*, 27(3).
