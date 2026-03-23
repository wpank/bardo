# Hypnagogia: Architecture and Implementation [SPEC]

> **Version**: 1.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Parent**: `00-overview.md` (Track 5: Hypnagogia)
>
> **Crate**: `golem-dreams` (hypnagogia extension module)
>
> **Depends on**: `golem-core`, `golem-dreams`, `golem-grimoire`, `golem-daimon`, `golem-inference`

---

> **Reader orientation:** This document specifies the four-layer implementation of hypnagogia within Bardo (the Rust runtime for mortal autonomous DeFi agents): ThalamicGate (external data gating), ExecutiveLoosener (temperature scheduling and prompt fragmentation), DaliInterrupt (partial completion capture), and HomuncularObserver (metacognitive fragment evaluation). It covers the HypnagogicOnset state machine, scheduling integration with the DreamScheduler, compute budget allocation, Hori-stage depth transitions, and the full Event Fabric event catalog. Prerequisites: the Hypnagogia overview (`00-overview.md`) and neuroscience foundations (`01-neuroscience.md`). For a full glossary, see `prd2/shared/glossary.md`.

## Implementation Philosophy

The research converges on a clear architectural principle: **computational hypnagogia requires orchestrated modulation across multiple levels simultaneously**, not any single mechanism in isolation. Temperature alone is "far more nuanced and weak than suggested" as a creativity parameter [PEEPERKORN-2024]. Steering vectors alone shift behavior but do not structure the creative process. Memory replay alone consolidates but does not generate novel associations. The power lies in their combination -- temperature scheduling providing the noise floor, steering vectors shifting the cognitive mode, structured prompting directing the associative process, and generative replay providing the raw material of recombined experience.

A naive implementation might simply raise temperature and call it "creative mode." The research cautions against this. Hypnagogia is not random noise -- it is a **structured intermediate state** with specific properties. The computational implementation must preserve all of them. Concretely, the hypnagogic cycle should: (a) reduce analytical constraints (higher temperature, creativity steering vector) but not eliminate them; (b) direct attention inward toward the agent's own memory and knowledge rather than live market data (thalamic gating analog); (c) maintain an evaluation subprocess that can recognize and flag genuinely novel insights (retained metacognition via the HomuncularObserver); and (d) operate for a bounded period before either returning to full "wakefulness" or descending into deeper "sleep" (full consolidation). The Lacaux et al. (2021) finding that creativity benefits vanish in N2 sleep is the most important design constraint: **the hypnagogic state must be actively maintained at the intermediate level**, not allowed to drift into either full analytical mode or full randomness [LACAUX-2021].

The implementation is organized as a four-layer stack, from cheapest and simplest (Layer 1: API-compatible, no model access required) to most powerful and complex (Layer 4: persistent memory architecture). Layers are additive -- each builds on the ones below it. A [CORE]-tier Golem implements Layer 1 only. A [HARDENED]-tier Golem implements all four.

---

## The HypnagogicOnset State Machine

### State integration with DreamScheduler

The DreamScheduler (see [../05-dreams/01-architecture.md](../05-dreams/01-architecture.md)) currently manages the `WAKING` and `SLEEPING.DREAMING` states. Hypnagogia adds two liminal states:

```
                    WAKING
                      |
            +---------v----------+
            |  HYPNAGOGIC_ONSET  |  <-- Liminal descent
            |  (H1->H2->H3->H6) |
            +---------+----------+
                      |
            +---------v----------+
            |  SLEEPING.DREAMING |  Existing dream cycle
            |  NREM -> REM -> INT|
            +---------+----------+
                      |
            +---------v----------+
            |  HYPNOPOMPIC_RETURN|  <-- Liminal ascent
            |  (gradual re-engage)|
            +---------+----------+
                      |
                    WAKING
```

```rust
/// Extended dream state machine with hypnagogic transitions.
#[derive(Debug, Clone, PartialEq)]
pub enum DreamState {
    /// Golem is fully awake. Heartbeat FSM active.
    Waking,

    /// Hypnagogic onset: transitioning from waking to dreaming.
    /// Heartbeat FSM suspended. External data progressively gated.
    /// Executive control partially loosened. Dali interrupts active.
    HypnagogicOnset {
        /// Current Hori stage analog (mapped to computational parameters).
        depth: HoriStage,
        /// Dali cycles completed so far.
        dali_cycles_completed: u8,
        /// Fragments captured this onset.
        fragments_captured: u16,
        /// Budget consumed (USDC).
        budget_consumed: f64,
    },

    /// Full dreaming: NREM replay -> REM imagination -> Integration.
    Dreaming(DreamPhase),

    /// Hypnopompic return: transitioning from dreaming to waking.
    /// Dream insights surfaced. Analytical reasoning re-engages gradually.
    HypnopompicReturn {
        /// Insights being surfaced from the completed dream cycle.
        insights_surfacing: u16,
        /// Mood transition in progress (dream mood -> waking mood).
        mood_transitioning: bool,
    },
}

/// Computational analog to the Hori classification system.
/// Maps biological sleep-onset substages to inference parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HoriStage {
    /// H1-H2: Relaxed wakefulness. Alpha dominant.
    /// Slight executive loosening. T=0.8-1.0.
    Shallow,

    /// H3-H4: Alpha dropout. Fleeting associations.
    /// Moderate loosening. T=1.0-1.2.
    Transitional,

    /// H5-H6: Theta onset. Peak hypnagogia.
    /// Maximum creative range. T=1.2-1.5. Dali interrupt zone.
    Peak,

    /// H7-H8: Too deep. Entering N2 territory.
    /// If reached, trigger Dali interrupt and return to Peak.
    TooDeep,
}
```

### Scheduling rules

The DreamScheduler determines when hypnagogic onset occurs:

1. Hypnagogia precedes every dream cycle. It is the on-ramp, not an independent event.
2. If the hypnagogic budget is exhausted, the Golem proceeds directly to NREM (graceful degradation).
3. If the behavioral phase is Declining or Terminal, hypnagogia is skipped entirely.
4. The onset phase is interruptible: a critical market event (detected via a dormant price-alert probe) can abort hypnagogia and return the Golem to full waking. This is the "alarm clock" mechanism -- analogous to a loud noise waking a person from light sleep.

```rust
impl DreamScheduler {
    /// Determine whether to run hypnagogic onset before the dream cycle.
    pub fn should_enter_hypnagogia(
        &self,
        config: &HypnagogicConfig,
        phase: BehavioralPhase,
        budget: &DreamBudget,
    ) -> bool {
        config.enabled
            && phase >= BehavioralPhase::Stable
            && budget.remaining_usdc() > budget.total_usdc() * config.budget_fraction
    }
}
```

### Adaptive depth modulation

The Golem's hypnagogic cycle should be modeled not as a binary state switch (wake/sleep) but as a **dynamical landscape with multiple metastable states**, mirroring the Hori classification system's 9 substages of biological sleep onset. Different "depths" of computational hypnagogia serve different functions: shallow hypnagogia (slightly elevated temperature, weak creativity steering) for minor strategic adjustments; deep hypnagogia (high temperature, strong steering vectors, fragmented memory replay, Dali-method interruptions) for paradigm-breaking strategic reconceptualization.

The Golem should learn to modulate its own depth based on the novelty of market conditions -- a form of metacognitive self-regulation that could itself be implemented via a learned policy over the hypnagogic parameters. When the market is behaving within the Golem's model's predictions (low surprise), shallow hypnagogia suffices. When the market breaks the model (high surprise, regime change), the Golem should automatically deepen its hypnagogic processing to search for new patterns. This depth-modulation policy would be calibrated over time by correlating depth settings with subsequent adaptation speed after market regime changes.

---

## Layer 1: Prompt-Level Hypnagogia

**Tier**: [CORE] and [HARDENED]
**Requirements**: API access only. No model weight access. No steering vectors.
**Cost**: ~$0.003-0.01 per onset cycle (T0/T1 inference).

This is the cheapest and most universally applicable layer. It structures the hypnagogic onset as a multi-phase prompt sequence that any LLM API can execute.

### Phase 1: Hypnagogic induction

Present the Golem with its recent episodic memory fragments in a deliberately **fragmented, non-sequential order**, with an explicit instruction to find unexpected connections, analogies, and patterns across domains. This mimics the brain's behavior at sleep onset: memories surface in scrambled temporal order, freed from the sequential narrative structure that waking cognition imposes.

```rust
/// Build the hypnagogic induction prompt.
/// Selects episodic fragments and presents them non-sequentially
/// with instructions for loose associative scanning.
pub fn build_induction_prompt(
    grimoire: &Grimoire,
    daimon: &DaimonState,
    cortical: &CorticalState,
    theme: Option<&str>,
) -> ContextBundle {
    // 1. Select fragments: recent + high-salience + decaying
    let recent = grimoire.episodes.query_recent(200);
    let salient = grimoire.episodes.query_by_salience(0.7, 50);
    let decaying = grimoire.semantic.query_near_demurrage(20);

    // 2. Fragment and shuffle: break episodes into sub-components,
    //    randomize order to break temporal associations.
    let mut fragments: Vec<EpisodicFragment> = Vec::new();
    for episode in recent.iter().chain(salient.iter()) {
        fragments.extend(episode.decompose_to_fragments());
    }
    fragments.extend(decaying.iter().map(|e| e.to_fragment()));

    // 3. Apply emotional bias from CorticalState PAD snapshot.
    //    Lock-free read -- hypnagogia never writes to the CorticalState.
    let pad = cortical.read_pad();
    let biased = apply_mood_bias(&fragments, &pad);

    // 4. Shuffle with controlled randomness (not fully random --
    //    preserve some local structure for coherence).
    let shuffled = controlled_shuffle(&biased, 0.7); // 70% shuffled

    // 5. Build prompt with CreativeDC framing:
    //    Divergent phase first, convergent evaluation second.
    ContextBundle::new()
        .with_system(HYPNAGOGIC_SYSTEM_PROMPT)
        .with_fragments(shuffled)
        .with_theme(theme)
        .with_temperature(1.2) // Initial: transitional range
}

const HYPNAGOGIC_SYSTEM_PROMPT: &str = r#"
You are entering a liminal state between focused analysis and free association.
Your task is NOT to analyze or solve. It is to NOTICE.

You will see fragments of your recent experience presented out of order.
Let connections form naturally. Do not force conclusions.
Look for:
- Unexpected similarities between unrelated events
- Patterns that span different protocols, timeframes, or market conditions
- Metaphors: what does this market behavior remind you of outside of DeFi?
- Inversions: what if the opposite of your current assumption were true?
- Edges: what is at the boundary of two categories you normally keep separate?

Report each connection as a brief fragment (1-3 sentences).
Do not evaluate whether the connection is useful. That comes later.
Do not filter for coherence. Capture the association as it forms.
"#;
```

### Phase 2: The Dali interrupt

Generate multiple **partial completions** (50-100 tokens each) at high temperature (T=1.2-1.5), then halt. A second, lower-temperature pass evaluates the fragments and selects the most promising for development. This computationally replicates Edison's bottle-drop method: capturing half-formed ideas before they resolve into conventional thinking.

```rust
/// Execute a single Dali interrupt cycle.
/// Generate a partial completion at high temperature, halt it,
/// then evaluate the fragment at low temperature.
pub async fn dali_interrupt(
    inference: &InferenceGateway,
    context: &ContextBundle,
    config: &HypnagogicConfig,
    cortical: &CorticalState,
    cycle_index: u8,
) -> Result<Vec<HypnagogicFragment>> {
    let mut fragments = Vec::new();

    // Re-read PAD at cycle boundary for emotional drift detection.
    let pad_snapshot = cortical.read_pad();

    // Generate 3-5 partial completions at elevated temperature.
    // Each is halted at 50-100 tokens -- the "bottle drop."
    let partials = inference.generate_partials(
        context,
        GenerateConfig {
            temperature: config.temperature_range.1, // Peak: 1.5
            max_tokens: 80,
            count: 4,
            sampling: SamplingStrategy::MinP { p_base: 0.1 },
            stop_sequences: vec![], // No stop -- rely on max_tokens
        },
    ).await?;

    // Evaluate each partial with the HomuncularObserver
    // at lower temperature (the metacognitive pass).
    for partial in &partials {
        let evaluation = inference.evaluate_fragment(
            partial,
            EvaluateConfig {
                temperature: 0.4, // Analytical evaluation
                max_tokens: 60,
            },
        ).await?;

        if evaluation.novelty_score > config.fragment_confidence_floor {
            fragments.push(HypnagogicFragment {
                id: Uuid::new_v4(),
                content: partial.text.clone(),
                novelty_score: evaluation.novelty_score,
                semantic_distance: evaluation.semantic_distance_from_playbook,
                emotional_tone: pad_snapshot,
                provenance: Provenance::Hypnagogic,
                timestamp: Utc::now(),
                dali_cycle: cycle_index,
                hori_stage: HoriStage::Peak,
                source_episodes: partial.source_episode_ids.clone(),
                confidence: 0.15,
                status: FragmentStatus::Raw,
            });
        }
    }

    // Emit Event Fabric event for this cycle.
    emit(HypnagogiaEvent::DaliInterrupt {
        cycle: cycle_index,
        fragments_captured: fragments.len() as u16,
        peak_temperature: config.temperature_range.1,
    });

    Ok(fragments)
}
```

### Phase 3: Lucid capture

After all Dali cycles complete, the Golem explicitly evaluates which novel connections survived the convergent filter and stores them as new strategic hypotheses. This is the "waking up just enough to write it down" moment.

```rust
/// Lucid capture: evaluate surviving fragments and promote
/// the most promising to hypothesis status.
pub async fn lucid_capture(
    inference: &InferenceGateway,
    fragments: &[HypnagogicFragment],
    playbook: &Playbook,
) -> Result<Vec<StagedHypothesis>> {
    let capture_prompt = format!(
        r#"You have just emerged from a liminal associative state.
Below are the fragments of novel connections you noticed.
For each, briefly assess:
1. Is this genuinely novel relative to your current PLAYBOOK?
2. Could this be tested or validated in live market conditions?
3. What would be the first concrete step to explore this?

Fragments:
{}

Current PLAYBOOK summary:
{}

Rate each fragment: PROMOTE (worth developing), STAGE (interesting but needs more),
or DISCARD (noise). Be generous -- novel ideas often look strange at first.
"#,
        format_fragments(fragments),
        playbook.summary()
    );

    let response = inference.complete(
        &ContextBundle::new()
            .with_system(capture_prompt)
            .with_temperature(0.5), // Moderate: analytical but not rigid
    ).await?;

    parse_capture_response(&response)
}
```

---

## Layer 2: Sampling-Level Modulation

**Tier**: [HARDENED] only
**Requirements**: API parameter control (temperature, min-p, top-k).
**Cost**: No additional cost over Layer 1 -- modifies inference parameters only.

### Temperature annealing within the Dali cycle

Rather than a fixed high temperature, implement temperature annealing within each Dali cycle: begin at T=1.3-1.5 during ideation, decrease to T=0.3-0.5 during evaluation. The analogy to stochastic resonance is direct: an optimal, non-zero noise level enhances signal detection.

```rust
/// Temperature schedule for a single Dali cycle.
/// Implements cosine annealing from peak to trough.
pub fn dali_temperature_schedule(
    step: usize,
    total_steps: usize,
    t_high: f32,
    t_low: f32,
) -> f32 {
    let progress = step as f32 / total_steps as f32;
    let cosine = (1.0 + (progress * std::f32::consts::PI).cos()) / 2.0;
    t_low + (t_high - t_low) * cosine
}
```

### Inference gateway integration

The temperature schedule integrates with the inference gateway (see [../12-inference/01-routing.md](../12-inference/01-routing.md)) through a `HypnagogicTempOverride` that the gateway respects during onset:

```rust
/// Temperature override for hypnagogic inference calls.
/// The inference gateway checks for this override before applying
/// the default temperature from the model tier config.
pub struct HypnagogicTempOverride {
    /// Current temperature (set by ExecutiveLoosener).
    pub temperature: f32,
    /// Sampling strategy override.
    pub sampling: SamplingStrategy,
    /// Whether this override is active.
    pub active: bool,
}
```

### Periodic reannealing

After each fragment capture, spike temperature back toward the creative range. This mimics biological stochastic resonance -- sustained noise, not a single perturbation. Without reannealing, the system settles into a low-temperature evaluative mode and stops generating novel associations.

```rust
/// After capturing a promising fragment, reanneal temperature
/// to re-enter the creative range for the next Dali cycle.
pub fn reanneal(current_temp: f32, config: &HypnagogicConfig) -> f32 {
    // Jump back to 80% of peak temperature.
    // Not full peak -- each successive cycle is slightly cooler,
    // modeling natural descent toward sleep.
    let target = config.temperature_range.1 * 0.8;
    current_temp.max(target)
}
```

### Min-p sampling

Use min-p sampling (p_base=0.1) rather than fixed top-p for more principled diversity control. Min-p dynamically adjusts the probability threshold based on the model's confidence for each token position, allowing more diversity when the model is uncertain and less when it is confident [NGUYEN-2024].

---

## Layer 3: Activation-Level Intervention

**Tier**: [HARDENED] only, when model weight access is available
**Requirements**: Access to model intermediate representations (via compatible API or self-hosted model).
**Cost**: Minimal overhead -- vector addition during forward pass.

### Creativity steering vector

Compute a "creativity direction" steering vector by contrasting activations on creative vs. analytical prompt pairs, following the methodology of [CREATIVITY-STEERING-2024]:

1. Prepare contrastive prompt pairs: one creative version and one analytical version of the same DeFi content.
2. Run both through the model, capturing intermediate layer activations (middle layers are most effective per Turner et al., 2024).
3. Compute the difference vector (creative - analytical) and normalize.
4. During hypnagogic onset, **add** the creativity vector to intermediate representations at the intervention layer.
5. During waking (analytical mode), **subtract** the creativity vector (or add zero).

```rust
/// Steering vector configuration for hypnagogic cognitive mode shifts.
pub struct SteeringConfig {
    /// Layer index for activation intervention.
    /// Middle layers (12-20 for a 32-layer model) are most effective.
    pub intervention_layer: usize,       // default: 16

    /// Scaling factor for the creativity vector during hypnagogia.
    /// Higher = more creative, lower = more analytical.
    pub creativity_alpha: f32,           // default: 1.5

    /// Pre-computed creativity direction vector.
    /// Computed offline from contrastive prompt pairs.
    pub creativity_vector: Option<Vec<f32>>,
}
```

This is the most precise analog to the DMN/prefrontal rebalancing that characterizes biological hypnagogia. The steering vector directly shifts the model's internal "cognitive mode" without consuming context window tokens or requiring prompt engineering.

### Inference-Time Intervention (ITI)

For models that expose attention head access, use Inference-Time Intervention (Li et al., 2024, NeurIPS; arXiv:2306.03341) to modulate specific attention heads associated with creative vs. analytical processing. This provides finer-grained control than full-layer steering vectors, allowing selective enhancement of associative processing in specific attention heads while preserving analytical capability in others. ITI and the creativity steering vector can be composed: the steering vector shifts the overall cognitive mode, while ITI fine-tunes which attention heads contribute most during the hypnagogic state.

**Practical note**: Layer 3 is the most technically demanding and the least universally available. Most Golems will run on commercial API providers (Claude, GPT-4) that do not expose intermediate representations. Layer 3 becomes available when Golems run on self-hosted open-weight models (Llama, Mistral) via the Venice inference provider (see [../12-inference/01-routing.md](../12-inference/01-routing.md)), which can be configured for intermediate access. The architecture is designed so that Layers 1 and 2 provide substantial creative benefit without Layer 3.

---

## Layer 4: Memory Architecture

**Tier**: [HARDENED] only
**Requirements**: Full Grimoire integration.
**Cost**: Minimal inference cost; primary cost is Grimoire I/O.

### Non-veridical generative replay during onset

Rather than presenting raw episodes to the hypnagogic process, implement **non-veridical generative replay**: the Grimoire generates novel combinations of episode fragments, producing hypothetical scenarios the Golem has never actually experienced. This mirrors the biological finding that dream replay is non-veridical [WAMSLEY-2010].

```rust
/// Generate non-veridical replay fragments for hypnagogic processing.
/// Combines elements from different episodes to produce novel scenarios.
pub fn generate_nonveridical_fragments(
    episodes: &[Episode],
    count: usize,
) -> Vec<NonveridicalFragment> {
    let mut rng = rand::thread_rng();
    let mut fragments = Vec::with_capacity(count);

    for _ in 0..count {
        // Select 2-3 source episodes randomly
        let sources: Vec<&Episode> = episodes
            .choose_multiple(&mut rng, rng.gen_range(2..=3))
            .collect();

        // Recombine: take context from one, action from another,
        // outcome from a third. This produces "what if" scenarios
        // the Golem never actually experienced.
        let fragment = NonveridicalFragment {
            context: sources[0].market_context.clone(),
            action: sources[1 % sources.len()].action_taken.clone(),
            outcome: sources[2 % sources.len()].outcome.clone(),
            source_episode_ids: sources.iter().map(|e| e.id).collect(),
            synthetic: true,
        };

        fragments.push(fragment);
    }

    fragments
}
```

### Beneficial forgetting integration

The dreaming cycle should include active forgetting: decaying episodic specifics while retaining statistical regularities (gist), with the forgetting rate modulated by environmental volatility -- faster forgetting in rapidly changing market conditions, slower in stable periods. This volatility-adaptive decay rate connects to the demurrage system (see [../02-mortality/05-knowledge-demurrage.md](../02-mortality/05-knowledge-demurrage.md)) and ensures the Golem's knowledge base stays calibrated to the current regime rather than overfitting to obsolete patterns.

Entries approaching the demurrage threshold receive a "last look" during hypnagogic onset. The process extracts gist-level patterns before specific details are lost:

```rust
/// Process entries near the demurrage threshold during hypnagogia.
/// Extract gist-level patterns before specifics decay.
pub async fn process_decaying_entries(
    inference: &InferenceGateway,
    entries: &[SemanticEntry],
) -> Result<Vec<GistExtraction>> {
    let prompt = format!(
        r#"These knowledge entries are approaching their expiration.
Before they fade, extract the essential pattern or lesson from each.
Do not preserve specific numbers, dates, or protocol addresses.
Preserve the SHAPE of the insight -- the abstract pattern that might
apply in future situations you haven't encountered yet.

Entries:
{}
"#,
        format_entries(entries)
    );

    let response = inference.complete(
        &ContextBundle::new()
            .with_system(prompt)
            .with_temperature(0.8), // Moderate: capture gist, not specifics
    ).await?;

    parse_gist_extractions(&response)
}
```

### Importance tagging and structured exploration

The episodic buffer uses importance tagging inspired by prioritized experience replay (Schaul et al., 2016; arXiv:1511.05952) -- episodes with high TD-error-like surprise signals are sampled more frequently during hypnagogic onset. This ensures that the most informationally rich episodes (those that violated the Golem's expectations) receive disproportionate attention during the creative recombination process.

During the dreaming cycle's evaluation phase, Tree-of-Thoughts (Yao et al., 2023; arXiv:2305.10601) and Graph-of-Thoughts (Besta et al., 2024; arXiv:2308.09687) structures allow the agent to explore, merge, and backtrack through novel strategic ideas generated from hypnagogic fragments. Tree-of-Thoughts extends chain-of-thought prompting by generating multiple reasoning paths and evaluating them in parallel. Graph-of-Thoughts generalizes this further, allowing non-linear combination and refinement of partial solutions -- a natural fit for the creative recombination that hypnagogia produces.

```rust
/// Importance-weighted sampling for hypnagogic fragment selection.
/// Episodes with higher surprise scores are sampled more frequently,
/// following the prioritized experience replay paradigm.
pub fn importance_weighted_sample(
    episodes: &[Episode],
    count: usize,
    alpha: f64, // prioritization exponent, default: 0.6
) -> Vec<&Episode> {
    let priorities: Vec<f64> = episodes.iter()
        .map(|e| e.surprise_score.powf(alpha))
        .collect();
    let total: f64 = priorities.iter().sum();
    let probs: Vec<f64> = priorities.iter().map(|p| p / total).collect();

    // Weighted sampling without replacement
    weighted_sample_without_replacement(episodes, &probs, count)
}
```

---

## CorticalState Integration (Deep)

CorticalState (see [../03-daimon/00-overview.md](../03-daimon/00-overview.md) section 5b) provides the affective substrate for hypnagogic processing. The integration operates at three levels:

### Level 1: Fragment selection bias

At onset entry, the PAD vector determines which episodic fragments surface. This is the computational analog of mood-congruent memory retrieval during biological hypnagogia:

```rust
/// Apply mood-based bias to fragment selection.
/// Uses the CorticalState PAD snapshot, not a direct Daimon query.
fn apply_mood_bias(fragments: &[EpisodicFragment], pad: &PADVector) -> Vec<EpisodicFragment> {
    let mut scored: Vec<(f64, &EpisodicFragment)> = fragments.iter().map(|f| {
        // Base score from recency and salience.
        let base = f.recency_score * 0.4 + f.salience_score * 0.3;

        // Emotional congruence: fragments matching current mood surface more.
        let congruence = pad.cosine_similarity(&f.emotional_tag) * 0.3;

        (base + congruence as f64, f)
    }).collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(_, f)| f.clone()).collect()
}
```

### Level 2: Somatic marker firing during onset

Somatic markers (see [../03-daimon/01-appraisal.md](../03-daimon/01-appraisal.md)) continue to fire during hypnagogia. When the creative generator produces a fragment that matches a stored marker pattern, the marker fires a `SomaticMarkerFired` event through the Event Fabric. The HomuncularObserver uses this signal as an additional evaluation dimension -- fragments that trigger somatic markers receive a relevance boost because they touch on situations the Golem has strong experiential associations with.

### Level 3: Vitality modulation

CorticalState also exposes the composite vitality score (`read_vitality()`). The hypnagogic onset uses this to enforce the behavioral phase depth table from [00-overview.md](00-overview.md) -- a vitality reading below the Conservation threshold triggers early termination of the onset.

---

## Event Fabric Events (Full Specification)

All events are emitted through the Event Fabric (see [../13-runtime/14-events.md](../13-runtime/14-events.md)) with subsystem tag `Subsystem::Hypnagogia`.

```rust
/// Full set of Event Fabric events for the hypnagogic system.
/// These events are consumed by:
/// - TUI (visualization, depth gauge, fragment sparklines)
/// - DreamJournal (logging for later analysis)
/// - Styx Archive (backup of dream journal entries)
/// - Metrics (onset efficiency, fragment yield)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypnagogiaEvent {
    /// DreamScheduler initiates hypnagogic onset.
    /// Emitted once per dream cycle (or zero times if onset is skipped).
    OnsetStarted {
        depth: HoriStage,
        budget_allocated_usdc: f64,
        pad_snapshot: PADVector,
        fragment_bias: FragmentBias,
    },

    /// HoriStage transitions during onset.
    /// Emitted 2-4 times per onset as depth increases.
    DepthChanged {
        from: HoriStage,
        to: HoriStage,
        current_temperature: f32,
    },

    /// Dali interrupt fires: partial completions halted and evaluated.
    /// Emitted once per Dali cycle (default 3 per onset).
    DaliInterrupt {
        cycle: u8,
        fragments_captured: u16,
        peak_temperature: f32,
    },

    /// HomuncularObserver promotes a fragment to the DreamJournal.
    /// Emitted 0-N times per Dali cycle depending on yield.
    FragmentCaptured {
        fragment_id: Uuid,
        novelty_score: f64,
        semantic_distance: f64,
        emotional_tone: PADVector,
    },

    /// HomuncularObserver rejects a fragment below threshold.
    /// Emitted for each rejected partial completion.
    FragmentDiscarded {
        fragment_id: Uuid,
        reason: String,
        composite_score: f64,
    },

    /// All Dali cycles complete, transitioning to NREM.
    /// Emitted once per onset.
    OnsetCompleted {
        total_fragments: u16,
        budget_consumed_usdc: f64,
        duration_secs: f64,
        dali_cycles_run: u8,
    },

    /// Critical market event aborts onset early.
    /// Emitted at most once per onset (replaces OnsetCompleted).
    AlarmAbort {
        trigger: String,
        fragments_saved: u16,
        budget_consumed_usdc: f64,
    },

    /// Hypnopompic return begins after dream cycle.
    HypnopompicStarted {
        insights_surfacing: u16,
    },

    /// Hypnopompic return completes, full waking restored.
    HypnopompicCompleted {
        insights_surfaced: u16,
        mood_transitioned: bool,
        pad_delta: f32,
    },
}

/// Fragment selection bias mode, derived from PAD state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FragmentBias {
    /// High arousal + negative pleasure -> threat-focused fragments.
    ThreatFocused,
    /// High arousal + positive pleasure -> opportunity-focused fragments.
    OpportunityFocused,
    /// Low arousal -> broad associative scan (most creative).
    BroadAssociative,
}
```

---

## HypnagogicFragment Schema

```rust
/// A fragment captured during hypnagogic onset.
/// Low-confidence, high-novelty entries stored in the DreamJournal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypnagogicFragment {
    /// Unique identifier.
    pub id: Uuid,

    /// The fragment content: a novel association, connection, or observation.
    pub content: String,

    /// Novelty score (0.0-1.0): semantic distance from existing PLAYBOOK.md entries.
    /// Scored by the HomuncularObserver.
    pub novelty_score: f64,

    /// Semantic distance from the nearest existing Grimoire entry.
    pub semantic_distance: f64,

    /// Emotional tone of the fragment (PAD vector snapshot from CorticalState).
    pub emotional_tone: PADVector,

    /// Provenance: always `Hypnagogic` for these entries.
    pub provenance: Provenance,

    /// Which Dali cycle produced this fragment.
    pub dali_cycle: u8,

    /// Hori stage at time of generation.
    pub hori_stage: HoriStage,

    /// Source episode IDs that contributed to this association.
    pub source_episodes: Vec<Uuid>,

    /// Timestamp of capture.
    pub timestamp: DateTime<Utc>,

    /// Initial confidence: always low (0.10-0.20).
    /// Must be validated through REM development + live testing
    /// before reaching operational weight.
    pub confidence: f64,

    /// Status in the development pipeline.
    pub status: FragmentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FragmentStatus {
    /// Just captured. Not yet processed by REM.
    Raw,
    /// Selected for development in the next REM imagination phase.
    SelectedForREM,
    /// Developed into a full counterfactual scenario during REM.
    Developed { scenario_id: Uuid },
    /// Promoted to staged hypothesis after Integration.
    Promoted { hypothesis_id: Uuid },
    /// Discarded during evaluation.
    Discarded { reason: String },
}
```

---

## Compute Budget

Hypnagogic processing draws from the DreamBudget (see [../05-dreams/01-architecture.md](../05-dreams/01-architecture.md)). The `budget_fraction` config parameter (default: 0.15) determines how much of the total dream budget is allocated to onset and return phases.

| Phase               | Budget Share | Operations                                        | Typical Cost ([HARDENED]) |
| ------------------- | ------------ | ------------------------------------------------- | ------------------------- |
| Hypnagogic Onset    | 10%          | Induction + 3 Dali cycles + Lucid capture         | $0.02-0.05               |
| NREM (replay)       | 35%          | Prioritized bidirectional replay                   | $0.07-0.15               |
| REM (imagination)   | 35%          | Counterfactual + creative recombination            | $0.07-0.15               |
| Integration         | 15%          | Consolidation + PLAYBOOK.md staging               | $0.03-0.08               |
| Hypnopompic Return  | 5%           | Insight surfacing + mood transition               | $0.01-0.03               |

For [CORE]-tier Golems, hypnagogia reduces to a single induction prompt at T0 (no Dali cycles, no HomuncularObserver), costing approximately $0.001.

---

## TUI Visualization

### Dream Cycle Timeline (Updated)

The existing dream cycle timeline (see [../05-dreams/00-overview.md](../05-dreams/00-overview.md)) is extended with onset and return segments:

```
|= ONSET =|=== NREM (replay) ====|==== REM (imagination) ====|= Integration =|= RETURN =|
  3 Dali    12 calls . 63% cache    8 calls . 4 web searches     1 call . TEE    1 call
  5 frags   $0.08 / $0.20 cap       $0.11 / $0.25 cap            $0.04 / $0.10   $0.02
```

The onset segment renders in **amber** (liminal, transitional), with small spark markers for each Dali interrupt. The return segment renders in **gold** (re-emergence). Fragment captures are shown as small bright dots within the amber segment.

**Note**: All TUI rendering uses Unicode symbols only (U+0000-U+FFFF BMP range), consistent with the TUI package conventions (see [../18-interfaces/01-tui.md](../18-interfaces/01-tui.md)). No emojis.

### Hypnagogic depth indicator

A vertical "depth gauge" alongside the timeline shows the current HoriStage:

```
H1 # Shallow
H2 #
H3 ## Transitional
H4 ##
H5 ### Peak <-- CURRENT
H6 ### (Dali zone)
H7 ... Too Deep
H8 ...
```

The gauge pulses gently at the Peak stage, reflecting the oscillatory nature of hypnagogia -- it flickers between stages, as the Lacaux 2024 review emphasizes.

---

## Safety Constraints

1. **Hypnagogic outputs never bypass the risk engine.** All fragments and hypotheses generated during onset enter the Grimoire at low confidence (0.10-0.20) and must pass through the standard validation pipeline before influencing live trading decisions. See [../10-safety/00-overview.md](../10-safety/00-overview.md).

2. **PolicyCage constraints are immutable during hypnagogia.** A creative fragment suggesting a strategy that violates approved asset lists, leverage limits, or position caps is tagged but not developed.

3. **The alarm abort mechanism is always active.** Even during deep hypnagogic processing, a dormant price-alert probe monitors for extreme events (>5% price moves, liquidity drops, protocol pauses). If triggered, onset is immediately aborted and the Golem returns to full waking. The abort emits an `AlarmAbort` event through the Event Fabric.

4. **Budget caps are hard.** Hypnagogic processing cannot exceed its allocated budget fraction. If the budget is exhausted mid-cycle, remaining Dali cycles are skipped and the Golem proceeds to NREM.

5. **No action is permitted during onset.** The heartbeat FSM is fully suspended. No trades, no position adjustments, no on-chain interactions. Hypnagogia is purely internal -- the biological atonia analog.

---

## Validation Metrics

| Metric                              | Computation                                                             | Healthy Range | Alarm           |
| ----------------------------------- | ----------------------------------------------------------------------- | ------------- | --------------- |
| **Fragment yield per Dali cycle**   | Captured fragments / total partial completions                          | 15-40%        | <5% or >60%     |
| **Novelty score distribution**      | Mean novelty score of captured fragments                                | 0.3-0.7       | <0.2 (generic)  |
| **Semantic distance from PLAYBOOK** | Mean cosine distance of fragments from nearest PLAYBOOK entry           | 0.4-0.8       | <0.3 (redundant) |
| **REM seed utilization**            | % of hypnagogic fragments selected for REM development                  | 20-50%        | <10% or >70%    |
| **Hypothesis promotion rate**       | % of developed fragments reaching staged hypothesis                     | 5-15%         | <2% or >30%     |
| **Live validation rate**            | % of hypnagogia-sourced hypotheses confirmed in live trading            | 3-10%         | <1%             |
| **Onset duration efficiency**       | Budget consumed / fragments captured (cost per fragment)                | $0.002-0.01   | >$0.02          |
| **Alarm abort frequency**           | % of onset cycles aborted by alarm                                     | <5%           | >15%            |

---

## Dreams as Predictions

Every creative output the hypnagogic engine produces is also a prediction. The Dali fragments captured at the N1 boundary, the NREM patterns consolidated from replayed episodes, the REM counterfactual scenarios -- all generate `Prediction` structs with `PredictionSource::Creative` and feed into the Prediction Ledger alongside the golem's analytical predictions.

The confidence levels reflect the source:

| Dream phase | PredictionSource | Confidence range | Expected accuracy |
|-------------|-----------------|------------------|-------------------|
| Dali fragments (N1) | Creative | 0.10 -- 0.20 | ~15% |
| NREM pattern consolidation | Creative | 0.25 -- 0.40 | ~30% |
| REM counterfactual scenarios | Creative | 0.20 -- 0.35 | ~25% |
| All creative combined | Creative | 0.10 -- 0.40 | ~34% |

A 34% accuracy rate sounds low. It is low. That's the point. Creative predictions are not supposed to be reliable -- they're supposed to be surprising. A prediction that the fee rate on an obscure Uniswap V4 pool will spike by 300% within 48 hours is worth generating even at 15% confidence, because the cost of the prediction is zero (it's a dream) and the value if correct is enormous.

The prediction engine treats creative predictions identically to analytical ones for resolution and residual calculation. The only difference is in the action gate: creative predictions must accumulate 3 independent confirmations before they can influence the environmental model or trigger action. One dream about a fee spike is noise. Three dreams about it, converging from different NREM episodes, is signal worth investigating.

---

## Dream Accuracy Tracking

The `dream_yield` metric tracks what fraction of creative predictions resolve correctly. It's the most honest measure of whether the dream engine is producing anything useful or hallucinating.

```rust
pub struct DreamYield {
    /// Total creative predictions generated (lifetime)
    pub total_creative: u64,
    /// Creative predictions that resolved correctly
    pub correct_creative: u64,
    /// Current rolling accuracy (last 100 creative predictions)
    pub rolling_accuracy: f32,
    /// Best rolling accuracy achieved this lifetime
    pub peak_accuracy: f32,
    /// Creative predictions promoted to environmental model (passed FDR gate)
    pub promoted: u64,
}

impl DreamYield {
    pub fn accuracy(&self) -> f32 {
        if self.total_creative == 0 { return 0.0; }
        self.correct_creative as f32 / self.total_creative as f32
    }
}
```

Dream yield feeds the meta-learning loop. If dream yield drops below 20% sustained over 2+ weeks, the meta-learner flags dream parameters for review. If it rises above 40%, the system increases dream frequency and expands the creative prediction budget.

The research basis for expecting non-trivial dream accuracy:

- **Lacaux et al. (2021)**. "Sleep Onset Is a Creative Sweet Spot." *Science Advances* 7(50). N1 hypnagogia produces ideas that are novel AND useful at rates exceeding both full wakefulness and deeper sleep stages.
- **McClelland et al. (1995)**. "Why There Are Complementary Learning Systems in the Hippocampus and Neocortex." *Psych. Review* 102(3). Replay during sleep consolidates episodic memories into generalizable knowledge.
- **Walker et al. (2002)**. "Practice with Sleep Makes Perfect." *Neuron* 35. Sleep-dependent learning improves performance on tasks that were practiced before sleep.
- **Richards & Frankland (2017)**. "The Persistence and Transience of Memory." *Neuron* 94(6). Forgetting is adaptive -- it removes noise and promotes generalization.

Creative predictions resolve on the same timeline as analytical ones. The ledger doesn't care where the prediction came from. Reality is the only grader.

---

## False Discovery Rate Control

The dream engine generates ~20-50 creative predictions per day. At 34% accuracy, that means ~7-17 correct and ~13-33 wrong. The wrong ones are harmless as predictions (they resolve as incorrect in the ledger). The danger is when a wrong creative prediction gets promoted into the environmental model and starts influencing analytical predictions.

The FDR gate requires **3 independent confirmations** before any creative prediction can be promoted:

1. The original creative prediction must resolve correctly.
2. A second prediction, from a different dream session, different NREM episode, or different REM scenario, must independently arrive at the same claim.
3. A third confirmation, either from another creative prediction or from an analytical prediction that observes the same pattern.

Only after all three does the claim enter the environmental model as a new heuristic or causal link.

```
Dream prediction: "Pool X fee rate spikes when ETH gas > 50 gwei"
  Confirmation 1: Correct resolution on 2026-03-12 (fee spiked)
  Confirmation 2: Independent NREM pattern match on 2026-03-14
  Confirmation 3: Analytical prediction on 2026-03-15 observes same correlation
  → PROMOTED to environmental model as causal link
```

Three is not a magic number. It's a compromise between false discovery rate (too few confirmations = noise enters the model) and discovery latency (too many = real patterns take weeks to register). At 34% base accuracy and 3 required confirmations, the false promotion rate is approximately 0.34^3 = 4% for truly random predictions, lower in practice because confirmations must be independent.

When a creative prediction fails all three confirmation attempts, it dies in the ledger as a resolved-incorrect entry. No shame. Most dreams are wrong. The system's value comes from the rare ones that aren't.

---

## Dream Curriculum from Evaluation Feedback

The evaluation system feeds back into dream scheduling. The NREM replay queue is not random -- it's prioritized by the evaluation loops.

The existing urgency function for dream scheduling has four terms. Add a fifth:

**Residual novelty.** Predictions whose residuals are both large AND novel (not matching any existing pattern in the Grimoire) get priority in the NREM replay queue. "Novel" means the residual's feature vector has low cosine similarity (<0.3) to any existing Grimoire episode or insight.

```rust
fn dream_urgency(
    memory_load: f32,       // existing: how full the episodic buffer is
    staleness: f32,          // existing: time since last consolidation
    emotional_weight: f32,   // existing: accumulated affect from recent episodes
    volatility: f32,         // existing: market regime volatility
    residual_novelty: f32,   // NEW: fraction of recent residuals that are novel
) -> f32 {
    let base = memory_load * 0.30
             + staleness * 0.25
             + emotional_weight * 0.20
             + volatility * 0.15;
    let oracle_boost = residual_novelty * 0.10;
    base + oracle_boost
}
```

The residual novelty term means: when the golem encounters something it can't explain with existing knowledge, it dreams about it sooner. Novel residuals are the system's way of saying "I saw something I don't understand." Dreaming about it is how the golem tries to understand.

This creates a feedback loop: novel residuals trigger earlier dreams, which produce creative predictions, some of which get confirmed and become new Grimoire entries, which reduce the novelty of future residuals (because the model now explains the pattern). The system is self-correcting through sleep.

---

## Reasoning Quality Review

The reasoning quality review runs during dream consolidation. It reviews the reasoning behind recent predictions, not just their outcomes.

The taxonomy:

| | Correct outcome | Incorrect outcome |
|---|----------------|-------------------|
| **Good reasoning** | AlignedCorrect -- no action needed | AlignedIncorrect -- model update needed, reasoning was sound but model was wrong |
| **Bad reasoning** | MisalignedCorrect -- dangerous, right answer for wrong reasons | MisalignedIncorrect -- reasoning correction needed |

MisalignedCorrect is the most dangerous cell. A prediction that says "ETH will go up because the president tweeted about it" and ETH does go up -- that's a correct prediction with bad reasoning. If the system only tracks accuracy, it rewards this. The reasoning review catches it.

During NREM replay, the top 50 residuals are reviewed not just for magnitude but for reasoning alignment. The golem re-examines its deliberation trace for each prediction and classifies it into the 2x2 matrix. AlignedIncorrect entries become high-priority model update candidates. MisalignedCorrect entries get flagged with a warning that the current accuracy in that category may be inflated by luck.

Research:
- **Chen et al. (2025)**. "Reasoning Models Don't Always Say What They Think." arXiv:2505.05410. LLMs produce unfaithful reasoning chains -- the stated reason may not be the actual computational path.
- **Huang et al. (2024)**. "Large Language Models Cannot Self-Correct Reasoning Yet." *ICLR 2024*. Self-correction without external feedback is unreliable. The prediction ledger provides that external feedback.

---

## Hermes Hooks During Dreams

The hypnagogic engine operates within the Hermes hierarchy's governance framework. Hermes L0 hooks fire at dream cycle boundaries:

- **Dream entry hook**: Fires when the DreamScheduler transitions from WAKING to HYPNAGOGIC_ONSET. L0 can inspect the allocated budget, the behavioral phase depth table, and the PAD snapshot. L0 can veto the onset (forcing the golem to skip directly to NREM) but cannot modify dream parameters.

- **Fragment capture hook**: Fires when the HomuncularObserver promotes a fragment to the DreamJournal. L0 can inspect the fragment's content, novelty score, and provenance. L0 can flag fragments for additional safety review but cannot silently discard them -- the golem must be notified.

- **Dream exit hook**: Fires when the hypnopompic return completes and the golem re-enters WAKING. L0 can inspect the dream cycle's outputs (total fragments captured, hypotheses staged, budget consumed) for audit purposes.

The Hermes hierarchy ensures that the hypnagogic engine, while internally autonomous (no L0 interference during active Dali cycles), operates within safety boundaries and produces auditable outputs.

---

## Dream Cycle Error Handling

If an LLM call fails during a Dali interrupt, the cycle proceeds with fewer fragments (min 1 of 4 must succeed). If all 4 fail, the dream cycle is aborted with a `DreamAborted` event and the golem returns to waking state. Dream budget is not consumed for aborted cycles.

---

## Cross-References

| Topic                                    | Document                                                                  |
| ---------------------------------------- | ------------------------------------------------------------------------- |
| Hypnagogia overview and thesis           | [00-overview.md](00-overview.md)                                          |
| Neuroscience foundations                  | [01-neuroscience.md](01-neuroscience.md)                                  |
| Creative divergence and alpha generation | [03-divergence-alpha.md](03-divergence-alpha.md)                          |
| The Homunculus subprocess                 | [04-homunculus.md](04-homunculus.md)                                      |
| Hauntology                                | [05-hauntology.md](05-hauntology.md)                                     |
| Dream cycle architecture                  | [../05-dreams/01-architecture.md](../05-dreams/01-architecture.md)       |
| Daimon and CorticalState                    | [../03-daimon/00-overview.md](../03-daimon/00-overview.md)               |
| Inference routing and tiers               | [../12-inference/01-routing.md](../12-inference/01-routing.md)           |
| Context engineering pipeline              | [../12-inference/04-context-engineering.md](../12-inference/04-context-engineering.md) |
| Risk engine and safety                    | [../10-safety/00-overview.md](../10-safety/00-overview.md)               |
| Knowledge demurrage                       | [../02-mortality/05-knowledge-demurrage.md](../02-mortality/05-knowledge-demurrage.md) |
| Event Fabric specification                | [../13-runtime/14-events.md](../13-runtime/14-events.md)                 |
| TUI conventions                           | [../18-interfaces/01-tui.md](../18-interfaces/01-tui.md)                |

---
