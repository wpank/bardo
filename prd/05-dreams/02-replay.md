# Dream Replay Engine: Prioritized Memory Re-processing [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `00-overview.md`, `01-architecture.md`, `golem-grimoire`

---

> **Reader orientation:** This document specifies the Replay Engine, the NREM-phase component of dreaming within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers Mattar-Daw utility-weighted episode selection, bidirectional replay, compressed batch replay, perturbed replay with noise injection, and compositional primitives for strategy decomposition. A Golem (mortal autonomous agent) replays past episodes through LLM inference to extract learning it could not access at encoding time. Prerequisites: the Dreams overview (`00-overview.md`) and architecture (`01-architecture.md`). For a full glossary, see `prd2/shared/glossary.md`.

## Why Replay Is the Highest-Value Dream Operation

Lin (1992) introduced experience replay: instead of discarding each experience after a single update, store past experiences in a buffer and re-use them [LIN-1992]. This multiplies learning signal from scarce data. For a Golem paying gas fees on every real trade, this multiplication is not a luxury — it is an economic necessity.

Schaul et al. (2016) showed that prioritized replay — sampling transitions where predictions were most wrong — dramatically outperforms uniform replay. In the Rainbow DQN ablation, prioritized experience replay was the single most important ingredient [SCHAUL-2016]. Fedus et al. (2020) overturned conventional wisdom by showing that larger replay capacity helps only when combined with multi-step returns, and that the replay ratio (updates per real experience) is a critical undertuned hyperparameter [FEDUS-2020].

For an LLM-native agent, "replay" means something different than for a neural network agent. The Golem does not update weights. Instead, it re-reads stored episodes through LLM inference, asking questions it could not ask at encoding time — because it now has more context, more experience, and hindsight about outcomes.

The key insight from McClelland, McNaughton, and O'Reilly's Complementary Learning Systems theory (1995): the fast hippocampal system rapidly encodes episodes, but the slow neocortical system requires interleaved replay to extract statistical regularities without catastrophic interference [MCCLELLAND-1995]. For the Golem, the fast system is the Grimoire's episode buffer (immediate storage). The slow system is PLAYBOOK.md (gradual extraction of general rules). Dream replay is the interleaving mechanism — the bridge between specific episodes and general knowledge.

---

## Utility-Weighted Episode Selection

### The Mattar-Daw Framework

Mattar and Daw (2018) proved that memories should be accessed in order of utility, defined as the product of two factors [MATTAR-DAW-2018]:

**Utility = Gain × Need**

Where:

- **Gain** measures how much replaying this episode would improve the current policy. An episode where the Golem's prediction was wildly wrong (high TD-error analog) has high gain — there is much to learn from it.
- **Need** measures the expected future relevance of the state. A market condition the Golem expects to encounter again soon has high need — the learning will be immediately useful.

This framework unifies three replay patterns that the biological literature treats separately:

- **Reverse replay** (after reward) = high gain, focused on recent experience
- **Forward replay** (before action) = high need, focused on upcoming states
- **Offline replay** (during sleep) = balanced gain × need across the full buffer

### LLM-Native Implementation

```rust
pub struct ReplayCandidate {
    pub episode_id: String,
    pub gain: f64,
    pub need: f64,
    pub utility: f64,
    pub last_replayed: Option<u64>,
    pub replay_count: u32,
    pub emotional_arousal: f64,
}

pub fn compute_replay_utility(
    episode: &Episode,
    current_state: &MarketState,
    _playbook: &Playbook,
) -> ReplayCandidate {
    let prediction_error = (episode.expected_outcome - episode.actual_outcome).abs();
    let surprise_factor = episode.surprise_score.unwrap_or(prediction_error);
    let outcome_significance = episode.pnl.abs() / episode.position_size;

    let suboptimality_bonus = episode.counterfactual_regret.unwrap_or(0.0);

    let gain = normalize(
        0.4 * surprise_factor + 0.3 * outcome_significance + 0.3 * suboptimality_bonus,
    );

    let states_similarity = cosine_similarity(
        &episode.market_state_embedding,
        &current_state.embedding,
    );
    let regime_similarity = if episode.regime == current_state.regime { 1.0 } else { 0.3 };

    let recency_decay = (
        -((current_state.tick - episode.tick) as f64) / RECENCY_HALFLIFE
    ).exp();

    let need = normalize(
        0.4 * states_similarity + 0.3 * regime_similarity + 0.3 * recency_decay,
    );

    let spacing_penalty = match episode.last_replayed {
        Some(last) => (-((current_state.tick - last) as f64) / SPACING_HALFLIFE).exp(),
        None => 0.0,
    };

    let utility = gain * need * (1.0 - 0.5 * spacing_penalty);

    ReplayCandidate {
        episode_id: episode.id.clone(),
        gain,
        need,
        utility,
        last_replayed: episode.last_replayed,
        replay_count: episode.replay_count,
        emotional_arousal: episode.emotional_tag.as_ref().map_or(0.0, |t| t.arousal),
    }
}
```

### Selection Algorithm

The replay scheduler selects episodes in two passes:

**Pass 1: Utility-ranked selection** — Sort all episodes by utility score. Select top N (where N = batchSize from config). This captures the highest-value episodes.

**Pass 2: Diversity injection** — Reserve 20% of slots for episodes selected by criteria other than raw utility:

- _Regime diversity_: at least one episode from each market regime experienced in the last 30 days.
- _Temporal diversity_: at least one episode from the oldest third of the buffer (prevents recency bias).
- _Emotional diversity_: if daimon is enabled, at least one high-arousal episode regardless of utility score.

This two-pass approach prevents the replay engine from fixating on a narrow set of high-surprise episodes — a known failure mode where prioritized replay creates a feedback loop that overweights unusual events at the expense of representative experience [SCHAUL-2016].

### Replay Decay

Wilson and McNaughton (1994) observed that replay intensity declines across successive sleep sessions [WILSON-MCNAUGHTON-1994]. The Golem implements this: each replay reduces the episode's utility score by a small factor. After 5–10 replays, the episode's gain component approaches zero (nothing new to learn from it). This natural decay prevents infinite re-processing of the same episodes.

```rust
pub fn apply_replay_decay(candidate: &mut ReplayCandidate) {
    let decay_factor = 0.85_f64.powi(candidate.replay_count as i32);
    candidate.gain *= decay_factor;
    candidate.utility = candidate.gain * candidate.need;
}
```

### Bloodstain feedback

Episodes inherited from dead predecessors via the Styx Lethe (formerly Commons) ("bloodstain" episodes) receive a 1.2x utility boost during NREM replay selection. These episodes represent the highest-signal learning material available: another Golem died, and the episode records what killed it or what it learned before dying. The cost of acquiring this knowledge through direct experience is, by definition, fatal.

The boost applies to the `gain` component of the Mattar-Daw utility formula. A bloodstain episode with base gain 0.5 becomes 0.6 after the boost. The `need` component is unchanged -- relevance to current market conditions matters regardless of provenance. The net effect: when market conditions are similar to those that killed a predecessor, the inherited death episodes surface near the top of the replay queue.

Bloodstain episodes are identified by `provenance: "inherited"` with `source_type: "death_testament"` in the Grimoire metadata.

---

## Bidirectional replay

### Forward vs. reverse replay

Foster and Wilson (2006) established two distinct replay modes [FOSTER-WILSON-2006]:

- **Forward replay**: "What would I do now given this starting state?" Projects current conditions forward through causal chains. Used for planning -- preparing optimal responses to anticipated market developments.
- **Reverse replay**: "What caused this outcome?" Traces backward from a known result to identify the decision or condition most responsible. Used for credit assignment -- learning which actions mattered.

The distinction matters for what each mode teaches. Forward replay builds preparedness. Reverse replay builds understanding. Both are necessary -- a Golem that only does forward replay becomes a planner that never learns from mistakes; one that only does reverse replay becomes an analyst that never anticipates.

### Reverse replay: credit assignment

Foster and Wilson (2006) first reported hippocampal place cells replaying experiences in temporally reversed order during awake rest [FOSTER-WILSON-2006]. Ambrose, Pfeiffer, and Foster (2016) showed that only reverse replays — not forward — increased in rate with increased reward [AMBROSE-2016]. The function: **backward credit assignment**, propagating reward information along the trajectory that led to the outcome.

**LLM-native reverse replay**:

For a completed trade with a known outcome, the Golem processes the decision chain in reverse:

```
Outcome: -2.3% loss on ETH-USDC LP position
  ← Final action: Exit position at tick 5,847 (slippage: 0.8%)
    ← Hold decision at tick 5,840 (gas spike prevented earlier exit)
      ← Entry decision at tick 5,200 (momentum signal triggered entry)
        ← Pre-trade analysis: "ETH momentum bullish, gas reasonable"
```

The LLM is prompted to evaluate each step's contribution to the outcome:

```
You are performing backward credit assignment on a completed trade.
Starting from the outcome, trace back through each decision and assess
its contribution to the final result.

Outcome: -2.3% loss
Decision chain (most recent first):
[serialized decision chain]

For each decision:
1. Was this the right decision given information available at the time?
2. What was the single biggest factor that made this decision contribute
   to the loss?
3. If you could change ONE thing at this step, what would it be?
```

**When to use reverse replay**: After any trade completion, with replay rate proportional to outcome magnitude (both positive and negative). Losses receive higher reverse replay priority because they contain more learning signal — successful trades may have succeeded by luck, but losses always have identifiable causal chains.

### Forward Replay: Planning

Diba and Buzsáki (2007) showed forward sequences occur in anticipation of action — before starting a run, not after completing one [DIBA-BUZSAKI-2007]. The function: **preplay for planning**, simulating upcoming scenarios to prepare optimal responses.

**LLM-native forward replay**:

The Golem projects forward from current market conditions:

```
Current market state:
[serialized current state: positions, prices, volumes, gas, regime indicators]

Relevant PLAYBOOK.md heuristics:
[extracted relevant sections]

Recent episodes with similar starting conditions:
[retrieved by similarity from Grimoire]

Project forward:
1. What are the 3 most likely market developments in the next 50 ticks?
2. For each development, what is the optimal response from PLAYBOOK.md?
3. What pre-conditions should trigger each response?
4. What is the worst case for each scenario, and what is the bailout plan?
```

**When to use forward replay**: During NREM phase when the Golem is approaching the end of a dream cycle and about to re-enter waking mode. Forward replay is the "morning planning" — preparing for the day ahead.

---

## Compressed Batch Replay

Sharp-wave ripples compress minutes of experience into ~100ms bursts [BUZSAKI-2015]. The LLM-native analog: processing multiple episodes simultaneously rather than one at a time. Batch replay exploits the LLM's ability to identify cross-episode patterns that single-episode replay would miss.

**Batch replay prompt structure**:

```
Review these 8 trading episodes from the last 72 hours as a batch.
They are ordered by utility score (highest first).

[Episode batch with structured fields: timestamp, market_state, action,
 outcome, pnl, surprise_score, emotional_tag]

Cross-episode analysis:
1. What pattern connects the top 3 episodes?
2. Is there a hidden variable that explains outcomes across the batch?
3. Do any episodes contradict each other? What explains the contradiction?
4. What single PLAYBOOK.md revision would have improved outcomes across
   the most episodes?
5. Which episode is the outlier, and what makes it different?
```

The batch size (5–15 episodes) is calibrated to fit within context window limits while providing enough episodes for cross-pattern detection. Fedus et al. (2020) showed that multi-step returns — which capture dependencies across sequential actions — significantly improve replay effectiveness [FEDUS-2020]. In LLM-native terms, the batch should include sequences of related episodes (e.g., the full lifecycle of a position: entry analysis → entry → holds → exit) rather than randomly selected singletons.

---

## Perturbed Replay

Deperrois et al. (2022) showed that NREM perturbed dreaming — replay with injected noise — is essential for building robust representations [DEPERROIS-2022]. The Golem replays episodes with systematic perturbations to build robustness:

| Perturbation Type       | Implementation                                           | Purpose                             |
| ----------------------- | -------------------------------------------------------- | ----------------------------------- |
| **Slippage injection**  | "What if slippage was 2x higher?"                        | Robustness to execution conditions  |
| **Latency injection**   | "What if your transaction was delayed by 3 blocks?"      | Robustness to network congestion    |
| **Gas spike**           | "What if gas was 5x higher at execution time?"           | Robustness to fee volatility        |
| **Data dropout**        | "What if the price feed was unavailable for 10 minutes?" | Robustness to data gaps             |
| **Liquidity change**    | "What if pool liquidity was 50% lower?"                  | Robustness to thin markets          |
| **Correlated movement** | "What if ETH and BTC decoupled during this trade?"       | Robustness to correlation breakdown |

Perturbed replay targets ~30% of total replay episodes (configurable). The perturbation is injected into the episode context before LLM processing, and the LLM evaluates whether the original strategy would have survived the perturbation:

```
Replay this episode with a modification:
[original episode with perturbation applied]

The perturbation is: [gas cost was 5x higher than actual]

Questions:
1. Would the original strategy still have been profitable?
2. At what point would the perturbation have changed the decision?
3. What is the maximum perturbation magnitude the strategy can tolerate?
4. Should PLAYBOOK.md include a guard for this perturbation?
```

### Deviation-Anchored Replay

When replaying episodes that have `OutcomeVerification` records (see `../04-memory/01-grimoire.md`), include the deviation data as context. The `OutcomeVerification` provides structured pre/post state comparisons — balance changes, gas deviations, unexpected logs — that ground the replay in real-world deviation magnitudes.

**Perturbation calibration**: "This trade deviated 15bps from `simulateContract()` prediction — what would have happened at 50bps, 100bps, 200bps deviation?" This produces perturbation scenarios calibrated to real-world deviation magnitudes rather than arbitrary LLM guesses.

**Integration with existing replay**: Deviation-anchored replay extends the perturbed replay pipeline. Where standard perturbation injects synthetic noise, deviation-anchored replay uses actual recorded deviations as the perturbation baseline. This produces more realistic stress scenarios.

LLMs cannot self-correct reasoning without external feedback (Huang et al., ICLR 2024). By anchoring dream replay in actual `OutcomeVerification` deviations, the Dream Engine's perturbation scenarios are grounded in on-chain reality rather than LLM imagination.

---

## Compositional Primitives (v2)

Bakermans et al. (2025) demonstrated that hippocampal state spaces are constructed compositionally from reusable primitives, enabling agents to behave optimally in new environments with zero additional learning [BAKERMANS-2025]. Replay builds and consolidates these compositional memories; during sleep the brain assembles novel configurations.

**Design for v2**: Decompose market situations into reusable primitives:

| Primitive               | Description                             | Composable With                                |
| ----------------------- | --------------------------------------- | ---------------------------------------------- |
| `LIQUIDITY_CRISIS`      | Rapid liquidity withdrawal from pools   | `MOMENTUM_BREAK`, `CASCADING_LIQUIDATION`      |
| `MOMENTUM_BREAKOUT`     | Strong directional movement with volume | `GAS_SPIKE`, `CORRELATION_SHIFT`               |
| `MEAN_REVERSION`        | Price returning to statistical mean     | `LIQUIDITY_RECOVERY`, `VOLATILITY_COMPRESSION` |
| `GAS_SPIKE`             | Network fee surge                       | Any primitive                                  |
| `ORACLE_DEVIATION`      | Price oracle diverging from spot        | `LIQUIDATION_CASCADE`, `ARB_OPPORTUNITY`       |
| `CASCADING_LIQUIDATION` | Chain of forced position closures       | `LIQUIDITY_CRISIS`, `CORRELATION_SHIFT`        |
| `CORRELATION_SHIFT`     | Historical correlations breaking down   | `MOMENTUM_BREAKOUT`, `REGIME_CHANGE`           |
| `REGIME_CHANGE`         | Structural shift in market behavior     | All primitives                                 |

During dreaming, primitives are compositionally recombined to simulate novel market scenarios the Golem has never directly experienced:

```
LIQUIDITY_CRISIS + CORRELATION_SHIFT + GAS_SPIKE =
  "A scenario where pool liquidity drops while ETH-BTC
   correlation breaks down and gas fees surge —
   a combination not yet seen but individually experienced."
```

This enables zero-shot generalization: the Golem can prepare for combinations of conditions it has never seen together but has experienced individually. The primitive taxonomy is built incrementally as the Golem encounters and labels more market conditions.

---

## Dual-System Integration with Grimoire

The replay engine operates on Grimoire data and produces Grimoire updates, implementing the dual-system architecture from McClelland et al. (1995) [MCCLELLAND-1995]:

**Fast system (Grimoire episodes)** → Dream replay → **Slow system (PLAYBOOK.md)**

```
┌──────────────────────────────────────────────────────┐
│                    GRIMOIRE                           │
│                                                       │
│  ┌─────────────┐   ┌──────────────┐   ┌───────────┐  │
│  │  Episodes    │──►│  REPLAY      │──►│ Insights  │  │
│  │  (LanceDB)  │   │  ENGINE      │   │ (SQLite)  │  │
│  │             │   │              │   │           │  │
│  │  Fast store │   │  Utility-    │   │  Slow     │  │
│  │  Full detail│   │  weighted    │   │  extract  │  │
│  │  Short-term │   │  Bidirection │   │  General  │  │
│  └─────────────┘   │  Perturbed   │   │  Long-term│  │
│                    └──────┬───────┘   └─────┬─────┘  │
│                           │                 │        │
│                    ┌──────▼─────────────────▼──────┐  │
│                    │       PLAYBOOK.md              │  │
│                    │  (procedural knowledge)        │  │
│                    │                                │  │
│                    │  Updated via Integration phase │  │
│                    │  Never directly by replay      │  │
│                    └────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

**Critical constraint**: The replay engine produces _candidates_ for Grimoire updates. It does not write directly to PLAYBOOK.md. All replay outputs feed into the Integration phase (Phase 3) where they are validated, classified, and staged for confirmation. This prevents dream-generated artifacts from contaminating operational knowledge — the computational analog of the Weismann barrier applied to dream content.

### Dream-Retrieval Strengthening

When an entry is retrieved during NREM replay and dream analysis produces a validated pattern, increment `strength` by 0.5 (reduced rate — dream validation is weaker than live-market confirmation). This implements Wilson & McNaughton (1994): sleep replay strengthens memory traces.

**Why reduced rate**: Live-market confirmation provides ground-truth feedback via `OutcomeVerification`. Dream validation is LLM-mediated pattern recognition without on-chain grounding. The 0.5 rate reflects this reduced evidential quality while still rewarding entries that prove useful during dream analysis.

**Integration with Grimoire lifecycle**: The `strength` field (see `../04-memory/01-grimoire.md`) controls decay rate via `retention(t) = e^(-(t - lastAccessed) / (halfLife × strength))`. Dream-strengthened entries decay slower, surviving longer in the active Grimoire and surfacing more frequently in waking DECIDING contexts. Additionally, dream-validated entries receive preferential knowledge demurrage — decaying at 0.5× the standard rate per `../02-mortality/05-knowledge-demurrage.md`.

**Strength increment comparison:**

| Context                               | Increment             | Rationale                              |
| ------------------------------------- | --------------------- | -------------------------------------- |
| Live-market retrieval + confirmation  | +1.0                  | Ground-truth via `OutcomeVerification` |
| Dream replay + validated pattern      | +0.5                  | LLM-mediated, no on-chain grounding    |
| Clade retrieval (external validation) | +0.75                 | Cross-agent confirmation               |
| Inherited entry (successor boot)      | +0.0 (starts at base) | Must re-earn via IKEA Effect           |

---

## Curator ↔ Dream Coordination

The Curator cycle (every 50 waking ticks) and the Dream Engine operate on the same Grimoire data but with complementary roles. Their coordination is bidirectional:

**Curator → Dream (tagging for replay)**

The Curator tags ambiguous episodes for dream replay. Episodes with contradictory patterns, insufficient evidence, or unresolved conflicts receive `dream_priority: high`. The Dream Engine's utility scheduler uses this as a boost factor in replay selection, prioritizing ambiguous episodes for exploratory analysis during the next dream cycle.

```rust
pub struct CuratorDreamTag {
    pub episode_id: String,
    pub reason: CuratorDreamReason,
    pub curator_cycle_id: u64,
    pub priority: DreamPriority,
}

pub enum CuratorDreamReason {
    ContradictoryPattern,
    InsufficientEvidence,
    UnresolvedConflict,
}

pub enum DreamPriority {
    High,
    Medium,
}
```

**Dream → Curator (accelerated promotion)**

Dream insights at `partially_validated` status are flagged for the next Curator cycle. The Curator can accelerate promotion if corroborating waking evidence exists — a dream hypothesis that matches an independently observed waking pattern gains confidence faster than either signal alone.

This bidirectional loop implements the episodic-to-semantic consolidation pipeline: the Curator identifies gaps, the Dream Engine explores them, and the Curator validates the results against waking experience.

> **Cross-ref**: `../04-memory/01-grimoire.md` (Curator ↔ Dream coordination, Admission Gate), `../04-memory/06-economy.md` (knowledge lifecycle)

---

## Replay Metrics and Validation

Each replay session produces metrics logged to the DreamJournal:

```rust
pub struct ReplayMetrics {
    pub episodes_replayed: u32,
    pub batches_processed: u32,
    pub reverse_replays: u32,
    pub forward_replays: u32,
    pub perturbed_replays: u32,

    pub patterns_identified: u32,
    pub contradictions_found: u32,
    pub credit_assignments_completed: u32,

    pub novel_insights_generated: u32,
    pub existing_insights_confirmed: u32,
    pub existing_insights_contradicted: u32,

    pub playbook_revisions_proposed: u32,

    pub total_inference_cost: f64,
    pub cost_per_insight: f64,

    pub insights_confirmed_by_live_trading: u32,
    pub insights_contradicted_by_live_trading: u32,
    pub replay_predictive_accuracy: f64,
}
```

The `replayPredictiveAccuracy` metric is the key validation signal. Over time, it calibrates the replay engine: if replay conclusions are consistently confirmed by live trading, the system is working. If they are consistently contradicted, the replay process needs adjustment (wrong episodes being selected, wrong questions being asked, or the LLM's implicit world model is too far from market reality).

---

## Citation Summary

| Citation Key             | Source                                                                                                       |
| ------------------------ | ------------------------------------------------------------------------------------------------------------ |
| [LIN-1992]               | Lin. "Self-improving reactive agents." _Machine Learning_, 1992.                                             |
| [SCHAUL-2016]            | Schaul et al. "Prioritized Experience Replay." _ICLR_, 2016.                                                 |
| [FEDUS-2020]             | Fedus et al. "Revisiting Fundamentals of Experience Replay." _ICML_, 2020.                                   |
| [MCCLELLAND-1995]        | McClelland et al. "Complementary Learning Systems." _Psychological Review_, 1995.                            |
| [MATTAR-DAW-2018]        | Mattar & Daw. "Prioritized memory access." _Nature Neuroscience_, 2018.                                      |
| [WILSON-MCNAUGHTON-1994] | Wilson & McNaughton. "Reactivation during sleep." _Science_, 1994.                                           |
| [FOSTER-WILSON-2006]     | Foster & Wilson. "Reverse replay." _Nature_, 2006.                                                           |
| [AMBROSE-2016]           | Ambrose et al. "Reverse replay modulated by reward." _Neuron_, 2016.                                         |
| [DIBA-BUZSAKI-2007]      | Diba & Buzsáki. "Forward and reverse sequences." _Nature Neuroscience_, 2007.                                |
| [BUZSAKI-2015]           | Buzsáki. "Hippocampal sharp wave-ripple." _Hippocampus_, 2015.                                               |
| [DEPERROIS-2022]         | Deperrois et al. "Perturbed and adversarial dreaming." _eLife_, 2022.                                        |
| [BAKERMANS-2025]         | Bakermans et al. "Constructing future behavior through composition and replay." _Nature Neuroscience_, 2025. |
| [EBBINGHAUS-1885]        | Ebbinghaus. _Über das Gedächtnis_, 1885.                                                                     |
| [CEPEDA-2006]            | Cepeda et al. Spacing effect meta-analysis, 2006.                                                            |
| [YANG-2024]              | Yang et al. "Selection of experience for memory by sharp wave ripples." _Science_, 2024.                     |

## Emotional Modulation of Replay Selection

The PAD (Pleasure-Arousal-Dominance) vector from Daimon modulates which episodes are selected for replay:

| PAD State  | Condition       | Replay Bias                                            |
| ---------- | --------------- | ------------------------------------------------------ |
| Anxious    | High A + Low P  | Weight `warning` / `regime_shift` episodes 2×          |
| Confident  | High P + High D | Prioritize exploratory / novel episodes                |
| Depleted   | Low A + Low D   | Conservative consolidation of known-good heuristics    |
| Despairing | Low P + Low D   | Legacy formation mode (prepare transferable knowledge) |

**Sampling rule:** PAD is sampled once at dream cycle start — not continuously updated during the dream. This prevents emotional drift from destabilizing a single dream session.

**Implementation:** The dream scheduler queries the current PAD vector from Daimon's last computed state, then applies the corresponding bias weights to the episode selection distribution before sampling.

**Minimum creative allocation:** Even during sustained negative affect (Depleted/Despairing), at least 20% of dream budget must be allocated to creative/exploratory operations. Without this floor, a Golem in negative mood loses its ability to discover novel escape strategies precisely when it needs them most. This overrides the PAD-driven bias for the creative portion of the budget.

> **Cross-ref**: `../03-daimon/00-overview.md` (PAD model), `../03-daimon/01-appraisal.md` (appraisal → PAD), `../03-daimon/03-behavior.md` (minimum creative allocation, mood-modulated dream content)
