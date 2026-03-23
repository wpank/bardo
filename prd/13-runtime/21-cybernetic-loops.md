# Cybernetic feedback loops: runtime subsystem [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-18
**Status:** Draft

---

> **Reader orientation:** This document specifies two self-correcting cybernetic feedback loops inside the Golem (a mortal autonomous DeFi agent) runtime: the TA signal metabolism loop (regulating how technical analysis signals are consumed) and the sheaf consistency loop (maintaining coherence across temporal scales). It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: the Heartbeat (recurring decision cycle) and the Event Fabric (internal event bus) from the observability spec. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Two self-correcting feedback loops run inside the runtime subsystem. Each loop takes an output, measures how well it worked, and feeds the measurement back to change the next output. These are the loops that don't have a natural home in other subsystem documents -- the metabolism loop operates across the TA/triage boundary, and the sheaf consistency loop operates across the temporal-scale boundary.

Both loops follow the same cybernetic structure: sensor -> comparator -> controller -> actuator -> environment -> sensor. The sensor observes the environment. The comparator measures the gap between observed and desired state. The controller computes a correction. The actuator applies it. The environment changes. The sensor observes the new state. Repeat.

> **Cross-references:**
>
> - `../../tmp/research/witness-research/new/reconciling/08-cybernetic-loops.md` — full catalog of all cybernetic loops across Bardo subsystems
> - `./09-observability.md` — Event Fabric architecture and Prometheus metrics that instrument loop behavior
> - `./13-engagement-loops.md` — engagement cadence architecture: the user-facing loops that sit above these runtime loops
> - `../01-golem/14-context-governor.md` — context governor loops: feedback mechanisms managing LLM context window assembly
> - `../03-daimon/01-appraisal.md` — PAD (Pleasure-Arousal-Dominance) vector from the Daimon affect engine, which both loops use for emotion-modulated parameter adjustment

Other cybernetic loops are documented in their owning subsystems:

| Loop | Subsystem document | Owner |
| --- | --- | --- |
| Dynamic Attention | `01-golem/14-context-governor.md` | ChainScope / AttentionForager |
| Curiosity Scorer | `01-golem/14-context-governor.md` | Hedge combiner |
| Mortality | `02-mortality/` | Vitality clocks |
| Memetic Evolution | `04-memory/` | Grimoire Curator / Dreams |
| Context Governor | `01-golem/14-context-governor.md` | Context Governor |
| Morphogenetic Specialization | `02-mortality/` | Clade reaction-diffusion |
| VCG Attention Auction | `01-golem/14-context-governor.md` | Attention allocator |

---

## 1. TA signal metabolism loop

**Signal activation -> Prediction outcome -> Weight update -> Allocation shift -> Next activation**

The metabolism loop governs how technical analysis signals earn and lose compute budget. Signals that predict well get more resources. Signals that predict poorly die. The loop operates at two timescales: Hebbian micro-updates per prediction resolution, and replicator macro-updates per Theta tick.

### 1.1 Loop diagram

```
                    ┌─────────────────────────────────────────────┐
                    │              ENVIRONMENT                     │
                    │   Market state, DeFi protocol events,        │
                    │   transaction flow, price movements           │
                    └──────────────┬──────────────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │            SENSOR                     │
                    │  TA signal activations: each signal   │
                    │  produces a prediction + confidence    │
                    │  for a specific DeFi context type      │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │          COMPARATOR                    │
                    │  Prediction outcome resolution:        │
                    │  did the predicted event happen?        │
                    │  accuracy_i = correct / total           │
                    │  info_gain_i = entropy reduction        │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │          CONTROLLER                    │
                    │  Two coupled mechanisms:                │
                    │  1. Hebbian micro: delta_w_ij per      │
                    │     prediction resolution               │
                    │  2. Replicator macro: fitness-weighted  │
                    │     budget reallocation per Theta tick   │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │          ACTUATOR                      │
                    │  Updated signal-context weights         │
                    │  Reallocated compute budget shares      │
                    │  Speciation events (new variants)       │
                    │  Death events (removed signals)         │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │        ENVIRONMENT (changed)           │
                    │  Next tick: signals run with new        │
                    │  weights and budgets, producing         │
                    │  different activations                   │
                    └──────────────────────────────────────┘
```

### 1.2 Sensor: signal activation

Each TA signal in the population produces a prediction for a specific DeFi context type (Swap, LP, Lending, Vault). The signal's activation is its prediction confidence in [0, 1]. The sensor records:

- Which signal activated
- What it predicted (direction, magnitude bucket)
- For which DeFi context
- At what confidence level
- The Daimon's current PAD state (modulates learning rate)

### 1.3 Comparator: outcome resolution

After each prediction resolves (the predicted event either happened or didn't within the resolution window), the comparator computes two metrics:

- **Accuracy**: binary correct/incorrect, accumulated as a rolling average
- **Information gain**: the entropy reduction the signal's prediction provided over the base rate

These feed into the per-signal fitness function:

```rust
/// Per-signal fitness, computed at each Theta tick.
pub fn signal_fitness(
    accuracy: f32,     // rolling accuracy over last 100 resolutions
    info_gain: f32,    // average entropy reduction per activation
    cost: f32,         // normalized compute cost of running this signal
) -> f32 {
    0.5 * accuracy + 0.3 * info_gain - 0.2 * cost
}
```

### 1.4 Controller: Hebbian + replicator

Two mechanisms operate at different timescales:

**Hebbian micro-level (per prediction resolution):**

```rust
/// Update signal-context weight after a prediction resolves.
pub fn hebbian_update(
    weight: &mut f32,
    activation: f32,       // signal's prediction confidence
    outcome: bool,         // did the prediction come true?
    eta_base: f32,         // base learning rate (default: 0.05)
    daimon_fear: f32,      // 0.0-1.0, from PAD vector
) {
    // Fear triples learning rate (learn faster from scary outcomes)
    // Calm halves it (don't overreact to boring data)
    let eta = eta_base * (1.0 + 2.0 * daimon_fear);
    let outcome_signal = if outcome { 1.0 } else { -1.0 };
    *weight += eta * activation * outcome_signal;
    *weight = weight.clamp(-1.0, 1.0);
}
```

Successful predictions strengthen signal-context associations. Failed predictions weaken them. The Daimon modulates learning rate: fear amplifies learning (the Golem pays more attention to outcomes during stressful conditions).

**Replicator macro-level (per Theta tick):**

```rust
/// Replicator dynamics: update budget shares based on fitness.
pub fn replicator_update(
    population: &mut Vec<TaSignal>,
    selection_pressure: f32,  // default: 1.0
) {
    let w_bar: f32 = population.iter()
        .map(|s| s.fitness * s.budget_share)
        .sum();

    for signal in population.iter_mut() {
        // dx/dt = x * (W - W_bar) * pressure
        let delta = signal.budget_share
            * (signal.fitness - w_bar)
            * selection_pressure;
        signal.budget_share += delta;
        signal.budget_share = signal.budget_share.max(0.001);
    }

    // Normalize to sum = 1.0
    let total: f32 = population.iter().map(|s| s.budget_share).sum();
    for signal in population.iter_mut() {
        signal.budget_share /= total;
    }
}
```

### 1.5 Actuator: budget reallocation, speciation, and death

Three output effects:

- **Budget reallocation**: signals with fitness above the population mean gain budget share. Signals below the mean lose it.
- **Speciation**: a signal with >25% accuracy divergence between two DeFi contexts forks into two context-specific variants. Each inherits the parent's weights for its respective context.
- **Death**: a signal whose budget share falls below 1% of total is removed. A tombstone persists for one Delta cycle to prevent immediate re-creation.

```rust
/// Speciation check: run after each replicator update.
pub fn check_speciation(signal: &TaSignal) -> Option<(TaSignal, TaSignal)> {
    let contexts = signal.per_context_accuracy();
    for (ctx_a, acc_a) in &contexts {
        for (ctx_b, acc_b) in &contexts {
            if (acc_a - acc_b).abs() > 0.25 {
                // Fork: create two specialists
                let mut specialist_a = signal.clone();
                specialist_a.context_filter = Some(ctx_a.clone());
                specialist_a.budget_share = signal.budget_share * 0.6;

                let mut specialist_b = signal.clone();
                specialist_b.context_filter = Some(ctx_b.clone());
                specialist_b.budget_share = signal.budget_share * 0.4;

                return Some((specialist_a, specialist_b));
            }
        }
    }
    None
}
```

### 1.6 Convergence

The replicator equation converges to an equilibrium where all surviving signals have equal fitness. Typical steady-state: 15-40 signals from a starting population of ~10 with speciation active.

Hebbian weights converge to the signal's true context-specific accuracy. With eta_base = 0.05, 95% convergence requires ~60 prediction resolutions per context. For a signal evaluated in Swap context every Theta tick (~60s), convergence takes ~60 minutes.

Fisher's fundamental theorem applies: the rate of increase in mean population fitness equals the variance in fitness. High diversity drives fast improvement. As selection narrows the population, variance drops and improvement slows. This is correct behavior -- the system converges when it has found a good signal population.

### 1.7 CorticalState integration

The metabolism loop writes three signals to the CorticalState each Theta tick:

- `ta_population_size`: current number of active TA signals
- `ta_mean_fitness`: population-average fitness score
- `ta_speciation_count`: number of speciation events in the last Delta window

These feed the Spectre's `triage_load` and `curiosity_drive` extended channels (see `./14-creature-system.md` section 4.4).

---

## 2. Sheaf consistency loop

**Multi-timeframe observation -> Cohomology computation -> Consistency score -> Context/action modulation -> Observation refinement**

The sheaf consistency loop verifies that the Golem's observations across temporal scales (gamma, theta, delta) tell a coherent story. When they contradict each other, the loop tightens the action gate and amplifies attention on the contradicting timescale. When they agree, the loop permits aggressive action and compresses redundant context.

### 2.1 Loop diagram

```
                    ┌─────────────────────────────────────────────┐
                    │              ENVIRONMENT                     │
                    │   Market at three temporal grains:            │
                    │   gamma (5-15s), theta (30-120s),            │
                    │   delta (~50 theta ticks)                     │
                    └──────────────┬──────────────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │            SENSOR                     │
                    │  Observations at each timescale:      │
                    │  F(gamma): R^8 (price, volume, gas,   │
                    │    spread, order flow, volatility,     │
                    │    velocity, acceleration)              │
                    │  F(theta): R^6 (trend, strength,      │
                    │    regime, correlation, confidence,     │
                    │    pattern match)                       │
                    │  F(delta): R^4 (strategy P&L,          │
                    │    prediction accuracy, knowledge       │
                    │    quality, long-term volatility)       │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │          COMPARATOR                    │
                    │  Sheaf cohomology computation:          │
                    │  1. Build cochain complex from          │
                    │     temporal poset                       │
                    │  2. Coboundary d_0: measure per-edge    │
                    │     disagreement                         │
                    │  3. Hodge Laplacian: spectral gap        │
                    │  C = 1 - (harmonic eigenvalues /         │
                    │          trace(L_1))                      │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │          CONTROLLER                    │
                    │  Consistency-dependent responses:       │
                    │  Low C (< 0.5): tighten action gate,   │
                    │    boost contradicting timescale in     │
                    │    context, raise arousal                │
                    │  High C (> 0.8): permit aggressive      │
                    │    action, compress redundant context,   │
                    │    baseline arousal                      │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │          ACTUATOR                      │
                    │  Context Governor weight adjustments    │
                    │  Action gate threshold modulation       │
                    │  Daimon arousal injection                │
                    │  Restriction map recalibration           │
                    │    (at Delta frequency)                  │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │        ENVIRONMENT (changed)           │
                    │  Refined observations at next tick:      │
                    │  recalibrated restriction maps produce   │
                    │  different consistency measurements      │
                    └──────────────────────────────────────┘
```

### 2.2 Sensor: multi-timeframe observation

The sheaf is a mathematical object that assigns data to each node in a temporal poset and encodes the expected relationships between adjacent nodes via restriction maps.

```rust
/// Observation vectors at each temporal grain.
pub struct TemporalObservations {
    pub gamma: Vec<GammaObservation>,   // R^8 per gamma tick
    pub theta: Vec<ThetaObservation>,   // R^6 per theta tick
    pub delta: Vec<DeltaObservation>,   // R^4 per delta tick
}

/// A gamma-scale observation: raw market microstructure.
pub struct GammaObservation {
    pub price: f64,
    pub volume: f64,
    pub gas_price: f64,
    pub spread: f64,
    pub order_flow: f64,
    pub volatility: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub tick: u64,
}

/// Restriction map: what a coarse-grained observation implies
/// about fine-grained data.
pub struct RestrictionMap {
    /// Linear map from theta-space to gamma-space.
    /// If theta says "uptrend," what range of gamma prices is consistent?
    pub theta_to_gamma: nalgebra::DMatrix<f64>,
    /// Linear map from delta-space to theta-space.
    pub delta_to_theta: nalgebra::DMatrix<f64>,
}
```

### 2.3 Comparator: cohomology computation

The coboundary operator measures disagreement across each containment edge:

```rust
/// Compute the sheaf consistency score across temporal scales.
pub fn compute_consistency(
    observations: &TemporalObservations,
    restriction: &RestrictionMap,
) -> ConsistencyResult {
    // 1. Coboundary: measure per-edge disagreement
    let gamma_theta_disagreement = observations.gamma.iter()
        .zip(observations.theta.iter())
        .map(|(g, t)| {
            let predicted_gamma = &restriction.theta_to_gamma * &t.as_vector();
            (predicted_gamma - g.as_vector()).norm()
        })
        .collect::<Vec<_>>();

    let theta_delta_disagreement = observations.theta.iter()
        .zip(observations.delta.iter())
        .map(|(t, d)| {
            let predicted_theta = &restriction.delta_to_theta * &d.as_vector();
            (predicted_theta - t.as_vector()).norm()
        })
        .collect::<Vec<_>>();

    // 2. Hodge Laplacian: spectral analysis
    let l1 = build_hodge_laplacian(
        &gamma_theta_disagreement,
        &theta_delta_disagreement,
    );
    let eigenvalues = l1.symmetric_eigenvalues();

    // 3. Consistency score from spectral gap
    let harmonic_sum: f64 = eigenvalues.iter()
        .filter(|&&e| e < 1e-6)
        .sum();
    let trace = eigenvalues.iter().sum::<f64>();
    let score = if trace > 0.0 {
        1.0 - (harmonic_sum / trace)
    } else {
        1.0
    };

    // 4. Locate the worst edge
    let worst_edge = if gamma_theta_disagreement.iter().sum::<f64>()
        > theta_delta_disagreement.iter().sum::<f64>()
    {
        WorstEdge::GammaTheta
    } else {
        WorstEdge::ThetaDelta
    };

    ConsistencyResult {
        score,
        worst_edge,
        gamma_theta_norm: gamma_theta_disagreement.iter().sum::<f64>(),
        theta_delta_norm: theta_delta_disagreement.iter().sum::<f64>(),
        harmonic_cochains: extract_harmonics(&l1, &eigenvalues),
    }
}
```

A consistency score near 1.0 means gamma, theta, and delta views tell the same story. Near 0.0 means irreconcilable contradictions across timescales.

### 2.4 Controller: consistency-dependent responses

The consistency score drives three downstream effects:

**Low consistency (C < 0.5) -- timescales disagree:**

- The Context Governor increases weight on the contradicting timescale in context assembly. If gamma and theta disagree, both get more context budget so the LLM can reason about the contradiction.
- The Daimon's arousal increases. Contradiction is surprising; it warrants more attention.
- The action gate tightens. The Golem should not act on contradictory data. Position sizing scales down proportionally.

**High consistency (C > 0.8) -- timescales agree:**

- The Context Governor compresses multi-timeframe context. All scales agree, so representing all three is redundant. The freed token budget goes to other context sources.
- Arousal stays at baseline.
- The action gate operates normally. Position sizing is unrestricted (within risk limits).

```rust
/// Modulate system behavior based on consistency score.
pub fn apply_consistency_modulation(
    consistency: &ConsistencyResult,
    context_governor: &mut ContextGovernor,
    daimon: &mut DaimonState,
    action_gate: &mut ActionGate,
) {
    if consistency.score < 0.5 {
        // Contradiction: amplify the disagreeing timescale
        match consistency.worst_edge {
            WorstEdge::GammaTheta => {
                context_governor.boost_source("gamma", 1.5);
                context_governor.boost_source("theta", 1.5);
            }
            WorstEdge::ThetaDelta => {
                context_governor.boost_source("theta", 1.5);
                context_governor.boost_source("delta", 1.5);
            }
        }
        // Raise arousal: contradiction is attention-worthy
        daimon.inject_arousal(0.2 * (1.0 - consistency.score));
        // Tighten action gate: don't act on contradictory data
        action_gate.set_threshold_multiplier(1.0 + (1.0 - consistency.score));
    } else if consistency.score > 0.8 {
        // Agreement: compress redundant context
        context_governor.compress_timescale_context();
        action_gate.set_threshold_multiplier(1.0);
    }
}
```

### 2.5 Actuator: restriction map recalibration

The restriction maps are not static. At Delta frequency, the system recalibrates them based on observed prediction residuals:

```rust
/// Recalibrate restriction maps based on observed disagreement patterns.
/// Runs at Delta frequency (~50 Theta ticks).
pub fn recalibrate_restrictions(
    restriction: &mut RestrictionMap,
    recent_disagreements: &[ConsistencyResult],
    learning_rate: f64,  // default: 0.01
) {
    // If gamma-theta disagreement is persistently high for a specific
    // feature dimension, the restriction map for that dimension is
    // poorly calibrated.
    let mean_gt_norm: f64 = recent_disagreements.iter()
        .map(|r| r.gamma_theta_norm)
        .sum::<f64>() / recent_disagreements.len() as f64;

    if mean_gt_norm > 0.5 {
        // Widen the restriction map's expected range for gamma given theta.
        // This is equivalent to saying "theta predictions about gamma
        // should be less precise" -- the map was overconfident.
        restriction.theta_to_gamma *= 1.0 - learning_rate;
    }

    // Same for theta-delta
    let mean_td_norm: f64 = recent_disagreements.iter()
        .map(|r| r.theta_delta_norm)
        .sum::<f64>() / recent_disagreements.len() as f64;

    if mean_td_norm > 0.5 {
        restriction.delta_to_theta *= 1.0 - learning_rate;
    }
}
```

### 2.6 Convergence

The Hodge decomposition is exact: the cochain complex decomposes into exact, coexact, and harmonic components. The consistency score converges as the restriction maps calibrate.

Well-calibrated maps produce consistency scores above 0.9 during normal markets and drop below 0.5 during genuine regime transitions. The score is not trying to reach 1.0 at all times. Low consistency during a real regime change is correct -- the timescales genuinely disagree, and the Golem should know.

Over ~10 Delta cycles, the restriction maps converge to reflect the true statistical relationship between timescales. After convergence, consistency drops are informative: they signal genuine structural change rather than map miscalibration.

### 2.7 CorticalState integration

The sheaf consistency loop writes two signals to the CorticalState:

- `consistency_score`: the scalar in [0, 1]
- `worst_edge`: which timescale boundary has the largest disagreement

These feed the Spectre's `topology_stability` extended channel (consistency and topology interact -- low consistency often coincides with topological regime shifts) and the Spectre's eye focus behavior (unfocused alternation when consistency drops below 0.6; see `./14-creature-system.md` section 4.4).

---

## 3. Loop interactions

The metabolism and sheaf consistency loops interact through the CorticalState:

| Producer | Consumer | Interface |
| --- | --- | --- |
| Sheaf consistency (low C) | Metabolism (raised arousal) | When consistency drops, the Daimon raises arousal, which triples the metabolism's Hebbian learning rate. TA signals learn faster during contradictory market conditions. |
| Metabolism (speciation) | Sheaf consistency (new observation dimensions) | When a TA signal speciates into context-specific variants, the theta observation vector gains new dimensions. The restriction maps must recalibrate to account for the new signal's predictions. |
| Sheaf consistency (high C) | Metabolism (compressed context) | When all timescales agree, the Context Governor compresses multi-timeframe context. The freed budget goes partly to TA context, giving the metabolism loop more information to evaluate signal quality. |

Both loops also interact with loops documented elsewhere:

- The **Context Governor** (documented in `01-golem/14-context-governor.md`) consumes both consistency scores and metabolism fitness data for context assembly decisions.
- The **Curiosity Scorer** (documented in `01-golem/14-context-governor.md`) uses metabolism population fitness to modulate the HDC structural similarity signal weight.
- The **Dynamic Attention** loop (documented in `01-golem/14-context-governor.md`) receives Hebbian reinforcement from addresses that appear in high-fitness TA signal activations.

---

## 4. Episodic Replay Loop (from source 10-episodic-replay)

**Past episode retrieval -> Narrative synthesis -> Context injection -> Deliberation -> Outcome recording -> Retrieval quality update**

Episodic replay is a case-based reasoning loop that converts Grimoire retrieval results into structured narrative context before deliberation. It bridges the gap between semantic retrieval (facts about similar past situations) and episodic memory (the "I was there" experience that frames current decisions).

### 4.1 Theoretical grounding

Tulving (1972) drew the foundational distinction between semantic memory (facts) and episodic memory (personally experienced events in spatio-temporal context). Grimoire retrieval is semantic; `EpisodicReplay` constructs the episodic frame.

Schank (1982) introduced Scripts and Memory Organization Packets (MOPs): past experiences are indexed by abstract situation structures. `EpisodeQuery.regime x affect` is the situation index; `NarrativeTemplate` renders the matched script.

Aamodt and Plaza (1994) formalized the CBR cycle: Retrieve, Reuse, Revise, Retain. `EpisodicReplay` implements Retrieve and Reuse. The Golem performs Revise during deliberation; Retain is handled by Grimoire storage.

Wilson and McNaughton (1994) demonstrated neural replay during sleep. `EpisodicReplay` is the waking counterpart -- replaying past episodes into working context before deliberation.

Lewis et al. (2020) established that grounding LLM generation in retrieved context (RAG) significantly improves factual accuracy. `EpisodicReplay` is a domain-specific RAG implementation.

### 4.2 Loop diagram

```
Query assembly (regime + affect + task embedding)
    |
    v
Grimoire retrieval (top-K similar episodes)
    |
    v
EpisodeSummary compression (full traces -> 5-8 word summaries)
    |
    v
NarrativeTemplate rendering (structured prefix)
    |
    v
Token budget enforcement (trim if exceeds budget)
    |
    v
Context injection (prepend to deliberation workspace at Step 5)
    |
    v
Deliberation (LLM reasons against the episodic prior)
    |
    v
Outcome recording (new episode stored in Grimoire)
```

### 4.3 Relevance scoring

Grimoire retrieval uses a composite similarity measure across three dimensions:

- **Regime match** (weight 0.4): Episodes from the same regime score highest.
- **Affect distance** (weight 0.3): Euclidean distance in PAD space. Episodes where the Golem was in a similar emotional state rank higher.
- **Task embedding similarity** (weight 0.3): Cosine similarity between current task embedding and episode's task embedding.

Recency weighting: `1.0 / (1.0 + (current_tick - episode_tick) / 10000.0)`. A 5000-tick-old episode scores at 67% of an identical current-tick episode.

### 4.4 Implementation

```rust
// crates/golem-core/src/episodic_replay.rs

pub struct EpisodicReplay {
    top_k: usize,
    token_budget: u32,
    template: NarrativeTemplate,
}

impl EpisodicReplay {
    /// Step 4 (Retrieve) integration point.
    pub async fn build_prefix(
        &self,
        query: EpisodeQuery,
        grimoire: &dyn Grimoire,
    ) -> NarrativePrefix {
        let episodes = grimoire
            .retrieve_similar_episodes(&query, self.top_k)
            .await;

        if episodes.is_empty() {
            return NarrativePrefix {
                text: String::new(),
                tokens: 0,
                episode_count: 0,
            };
        }

        let summaries: Vec<EpisodeSummary> = episodes.iter()
            .map(|e| EpisodeSummary::from_episode(e, &query))
            .collect();

        let text = self.template.render(&summaries, &query);
        let tokens = count_tokens(&text);

        let (text, tokens) = if tokens > self.token_budget {
            let trimmed = trim_to_token_budget(&text, self.token_budget);
            let t = count_tokens(&trimmed);
            (trimmed, t)
        } else {
            (text, tokens)
        };

        NarrativePrefix { text, tokens, episode_count: summaries.len() }
    }
}
```

### 4.5 Example rendered prefix

```
## Past experience
In 3 similar sessions (volatile):
  - tick 4812: volatile, reduce_exposure -> gain
  - tick 3901: volatile, hold -> loss
  - tick 2744: volatile, reduce_exposure -> gain
```

This occupies ~30-40 tokens. The Golem sees the pattern immediately: reducing exposure in volatile conditions succeeded twice; holding once led to a loss.

### 4.6 CorticalState integration

Episodic replay does not write CorticalState signals directly. Its output is one of the context segments tracked by the `DeltaCompressor` (see `01-golem/03c-state-management.md` Section 2). The episodic prefix may change each tick as the query changes, contributing to `tokens_delta`.

### 4.7 Interaction with other loops

- **TA signal metabolism** (Section 1): TA signal fitness scores inform the task embedding used in episode queries. High-fitness signals' activation patterns shape what "similar" means.
- **Sheaf consistency** (Section 2): When consistency drops, the Context Governor assigns more budget to episodic context (the Golem should recall what happened last time timescales contradicted).
- **Dream consolidation** (documented in `01-golem/03b-cognitive-mechanisms.md` Section 2): Dream cycles compress raw episodes into the semantic summaries that episodic replay later retrieves. The quality of replay depends on the quality of consolidation.

---

_End of document._
