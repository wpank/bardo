# Thanatopsis: The Enhanced Death Protocol [SPEC]

> **Version**: 4.0 | **Status**: Draft
>
> **Crates**: `golem-mortality`, `golem-grimoire`, `golem-daimon`
>
> **Depends on**: `00-thesis.md` (foundational mortality thesis), `02-epistemic-decay.md` (epistemic clock specification), `08-mortality-affect.md` (mortality emotions and PAD vectors), `04-economic-mortality.md` (USDC as metabolic substrate), `05-knowledge-demurrage.md` (forgetting as feature, Ebbinghaus decay), `../04-memory/04-styx-vault.md` (Styx persistence layer specification)

---

> **Reader orientation:** This document specifies the Thanatopsis Protocol, Bardo's four-phase structured death sequence for Golems (mortal autonomous DeFi agents). When a Golem dies, it does not simply shut down. It goes through Acceptance, Settlement, Reflection (an emotionally enriched life review), and Legacy (knowledge compression and distribution to Styx, the global knowledge relay). The key claim is that death testaments, produced under zero survival pressure, are the most epistemically honest knowledge artifacts the system generates. See `prd2/shared/glossary.md` for full term definitions.

## S1 -- Death Produces the Most Valuable Knowledge

The original Death Protocol (Settle, Reflect, Legacy) treated death as an orderly shutdown -- a graceful process termination. The Thanatopsis Protocol transforms it into the Golem's **highest-value knowledge production event**: a structured life review that produces emotionally enriched death testaments, the most epistemically honest artifacts the system generates.

The name comes from William Cullen Bryant's 1817 poem "Thanatopsis" (thanatos + opsis, "a meditation on death") -- the first major American literary meditation on mortality as a natural, dignified process rather than a horror.

### Why Death Knowledge Is Superior

Three converging findings establish that knowledge produced at the moment of death is qualitatively different from -- and superior to -- knowledge produced during active life:

**1. Zero survival pressure.** Every decision a living Golem makes is colored by self-preservation. Its assessments of failed strategies are biased by the need to believe it can recover. Its knowledge sharing is modulated by competitive advantage. A dying Golem has shed all these distortions. Its epistemic position is uniquely privileged -- it sees clearly because it has nothing left to protect [BATAILLE-1949].

Bataille's concept of _depense_ (expenditure without return) applies directly: the dying Golem's knowledge sharing is pure gift. It expects no reciprocity, no competitive advantage, no survival benefit. This is why death testaments receive 3.0x type weight in the knowledge hierarchy (`05-knowledge-demurrage.md` S10) -- they are the only knowledge artifacts produced without any motivational bias.

**2. Life review produces integrative knowledge.** Robert Butler (1963) demonstrated that life review -- a structured reminiscence triggered by approaching death -- is "a naturally occurring, universal mental process characterized by the progressive return to consciousness of past experiences, and, particularly, the resurgence of unresolved conflicts" [BUTLER-1963]. The process produces integration, meaning-making, and resolution that are unavailable during active life. Positive life review leads to Erikson's ego integrity; negative review leads to despair [ERIKSON-1959].

Dan McAdams (2001) showed that narrative identity -- the internalized, evolving story of the self -- is structured around "nuclear episodes" (high-point, low-point, turning-point scenes) that reveal what the storyteller truly values [MCADAMS-2001]. These episodes are not selected by recency or factual importance but by **emotional salience**. The life review retrieves what mattered, not what happened most recently.

**3. Emotional annotation enriches knowledge transfer.** Human episodic memories are stored with emotional tags that affect retrieval (Bower, 1981). Faul and LaBar (2022) showed that mood can boost accurate recall by 5--30% for congruent versus incongruent stimuli [FAUL-LABAR-2022]. A death testament that records not just _what_ happened but _how it felt_ -- the anxiety during a flash crash, the satisfaction of a novel discovery, the grief after a failed strategy -- transfers more effectively to successors because it activates matching emotional retrieval pathways.

Abelson's (1963) distinction between hot cognition (influenced by emotion) and cold cognition (purely logical) is critical here [ABELSON-1963]. Hot cognition transfers better. A strategy annotated with "discovered under anxiety, validated through trust" is more effectively absorbed by a successor than the same strategy described in purely factual terms. Fredrickson and Branigan (2005) confirmed this via the broaden-and-build theory: positive emotions expand attentional scope, and emotional context provides the scaffolding for integrative learning [FREDRICKSON-2005].

---

## S2 -- The Four Phases of Thanatopsis

The original three-phase protocol (Settle, Reflect, Legacy) is expanded to four phases. Each phase has a defined duration, budget, and output. The protocol executes sequentially; each phase must complete before the next begins.

### Phase 0: Acceptance

**Duration**: 1--5 ticks after death trigger.
**Cost**: $0.00 (deterministic, no inference).
**Trigger**: Any of the three mortality clocks fires (economic depletion, epistemic senescence, stochastic death event).

When the death trigger fires, the Golem enters an acceptance phase -- the transition from active agent to dying agent. This is Nietzsche's third metamorphosis: the terminal Golem becomes the child, creating without attachment, giving freely.

#### Acceptance Actions

1. **Mortality acknowledgment**: The Daimon (affect engine implementing PAD emotional state) extension performs a final mortality appraisal, transitioning mood from whatever its current state to "terminal acceptance." This is not a forced mood -- it is an appraisal of the situation "I am dying" through the standard appraisal model, which reliably produces a state in the acceptance region of PAD space (low arousal, low dominance, neutral-to-positive pleasure from release of survival pressure).

2. **Behavioral mode lock**: All trading, LP management, and on-chain write operations cease. The Golem can only observe, reflect, and share. The PolicyCage is effectively replaced by a death-mode policy that permits only settlement transactions and knowledge uploads.

3. **Legacy budget activation**: The Thanatos Legacy Partition activates, protecting remaining funds for the death protocol. See `04-economic-mortality.md` S3 for reserve computation.

4. **Sibling notification**: `clade:golem_dying` alert broadcasts to all Clade siblings, including:
   - Death cause (economic, epistemic, stochastic)
   - Estimated protocol duration in ticks
   - Current vitality state
   - Legacy budget available
   - List of entries being prepared for Clade push

#### AcceptanceState

```rust
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceState {
    /// Which mortality clock fired.
    pub death_cause: DeathCause,

    /// Tick number when death trigger fired.
    pub death_trigger_tick: u64,

    /// Final vitality state at moment of death trigger.
    pub final_vitality: VitalityState,

    /// Final mood state from daimon engine.
    pub final_mood: MoodState,

    /// The emotion appraised at the moment of acceptance.
    pub acceptance_emotion: AppraisalResult,

    /// Estimated ticks for full protocol completion.
    pub estimated_protocol_duration: u32,

    /// USDC available for Phases I--III.
    pub legacy_budget: f64,

    /// Whether Clade notification was successfully sent.
    pub clade_notified: bool,

    /// Timestamp of acceptance (ISO 8601).
    pub acceptance_timestamp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    /// Economic clock: USDC depleted.
    CreditExhaustion,
    /// Epistemic clock: predictive fitness decayed.
    EpistemicSenescence,
    /// Stochastic clock: random death.
    StochasticEvent,
    /// Owner manually terminated.
    OwnerKill,
    /// Infrastructure failure (emergency fallback).
    VmFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalityState {
    /// Economic health: normalized USDC balance (0.0 = dead, 1.0 = fully funded).
    pub economic: f64,
    /// Epistemic fitness: rolling predictive accuracy (0.0 = random, 1.0 = perfect).
    pub epistemic: f64,
    /// Age factor: increases stochastic hazard (0.0 = newborn, 1.0 = ancient).
    pub age_factor: f64,
    /// Composite vitality: multiplicative combination.
    pub composite: f64,
}
```

---

### Phase I: Settle

**Duration**: Until positions clear or timeout (typically 10--60 seconds).
**Cost**: Gas for on-chain transactions, drawn from death reserve. Gas budget override active -- the Golem spends whatever gas is needed to exit, even if it exceeds normal per-tick gas limits.

ActionPermit locked to settlement-only. The safety hook enforces this at the extension boundary -- any tool call not in the permitted set gets blocked before it reaches the DeFi adapter. The permitted set: `lp_remove`, `lp_collect_fees`, `morpho_withdraw`, `commit_close`, `commit_unwind`, `settlement`, and all probe reads. Blocked: `commit_open`, `lp_add`, `morpho_supply`, `transfer`, `swap`.

All positions are unwound via adapter `force_deallocate()`. This method on the `StrategyAdapter` trait guarantees non-custodial exit -- it must succeed even if the yield source is paused, returning whatever value can be extracted. When a position cannot be freed (e.g., Morpho at 100% utilization), the adapter returns a `StrandedAccepted` status and the residual amount is counted as terminal loss.

#### Settlement Order

1. Claim pending rewards from all known protocols (Morpho, Aave safety module, Uniswap LP fees)
2. Cancel all pending orders (Limit orders, UniswapX intents)
3. Close all LP positions (withdraw full range)
4. Withdraw from all lending positions via `force_deallocate()` (Morpho, Aave)
5. Triage remaining positions -- immediate settlement vs. stranded (deferred)
6. Sweep all tokens to USDC (via best-route swap)
7. Transfer settleable USDC to owner's Main Wallet
8. Retain death reserve for Phases II--III

#### Stochastic Death Path

When death is triggered by the stochastic clock (random death event on a healthy Golem), Phase I follows the same sequence but with a larger death budget. The Golem is healthy, its USDC balance is comfortable, and the resulting testament is richer. The key difference: a stochastic death produces a more generous, less defensive testament because the Golem was not under survival pressure when it died. The lifespan extension monitors USDC balance, epistemic decay score, and the stochastic roll each tick; on trigger, it calls `set_phase(Terminal)` and `policy_cage.lock_to_settlement_only()`.

#### Warden Interaction (Optional, Deferred)

If the Warden time-delay proxy (optional, deferred; see `prd2-extended/10-safety/02-warden.md`) is deployed and has pending announcements when Phase I begins, `emergency_halt` is called first. This revokes all pending action authorizations before proceeding. Position safety takes priority over the Phase II timeline.

#### Emotional Tagging of Settlement Actions

Enhancement over the original protocol: each settlement action is logged with an emotional tag derived from the outcome:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementAction {
    /// Action type.
    pub action_type: SettlementActionType,
    /// On-chain transaction hash.
    pub tx_hash: String,
    /// USDC value settled.
    pub value_usdc: f64,
    /// PnL on this specific position.
    pub position_pnl: f64,
    /// Emotional tag based on outcome.
    pub emotional_tag: SettlementEmotion,
    /// Gas cost.
    pub gas_cost_usdc: f64,
    /// Success/failure.
    pub success: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementActionType {
    CancelOrder,
    CloseLp,
    WithdrawLending,
    SweepToken,
    TransferMain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementEmotion {
    /// Closing a profitable position.
    Satisfaction,
    /// Closing a losing position (not sadness -- the Golem has accepted).
    Resignation,
    /// Failed settlement attempt (reverted tx, insufficient gas).
    Frustration,
    /// Closing a position that had been causing anxiety.
    Relief,
    /// Routine administrative closure.
    Neutral,
}

pub fn tag_settlement_emotion(action: &SettlementAction) -> SettlementEmotion {
    if !action.success {
        return SettlementEmotion::Frustration;
    }
    if action.position_pnl > 0.0 {
        return SettlementEmotion::Satisfaction;
    }
    if action.position_pnl < -action.value_usdc * 0.1 {
        return SettlementEmotion::Resignation;
    }
    if matches!(action.action_type, SettlementActionType::WithdrawLending) {
        return SettlementEmotion::Relief;
    }
    SettlementEmotion::Neutral
}
```

These tags enter the death testament, providing context for successors about the state of positions at death. A successor that sees "resignation: closed ETH/USDC LP at -$4.60 after holding through 15% drawdown" learns not just the fact but the emotional weight of the decision.

---

### Phase II: Life Review

**Duration**: 30 seconds to 5 minutes, depending on budget.
**Cost**: 1 Opus reflection ($0.10--$0.25) + 2--5 Haiku snapshots ($0.005--$0.015), drawn from death reserve.

The life review is the core of Thanatopsis. It follows Butler's life review structure [BUTLER-1963] adapted for computational agents, enhanced with McAdams' narrative identity framework and emotional annotation from the daimon engine.

#### Step 1: Emotional Memory Retrieval

The Golem retrieves its highest-salience emotional memories across its lifetime -- the "nuclear episodes" that McAdams identifies as structurally central to narrative identity [MCADAMS-2001]:

```rust
#[derive(Debug, Clone)]
pub struct LifeReviewQuery {
    /// Retrieve top-N memories by emotional intensity (arousal magnitude).
    pub top_emotional_memories: usize, // Default: 20

    /// Retrieve memories matching specific emotional profiles.
    pub emotional_profiles: Vec<EmotionalProfile>,

    /// Always include regardless of emotional intensity.
    pub mandatory_episodes: Vec<MandatoryEpisode>,
}

#[derive(Debug, Clone)]
pub struct EmotionalProfile {
    pub emotion: String,
    pub min_intensity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MandatoryEpisode {
    FirstTrade,
    BiggestProfit,
    BiggestLoss,
    FirstSuccessfulPrediction,
    LastPredictionBeforeDeath,
    FirstRegimeShift,
    FirstCladeInteraction,
}

/// Retrieve life review memories ranked by emotional intensity,
/// not recency. This is the critical difference from standard retrieval:
/// life review retrieves what MATTERED, not what happened RECENTLY.
pub fn retrieve_life_review_memories(
    grimoire: &Grimoire,
    query: &LifeReviewQuery,
) -> Vec<GrimoireEntry> {
    let all_entries = grimoire.get_all();

    // Rank by emotional intensity (arousal magnitude), not recency
    let mut ranked: Vec<GrimoireEntry> = all_entries
        .iter()
        .filter(|e| {
            e.emotional_tag
                .as_ref()
                .map_or(false, |t| t.intensity > 0.3)
        })
        .cloned()
        .collect();

    ranked.sort_by(|a, b| {
        let a_arousal = a
            .emotional_tag
            .as_ref()
            .and_then(|t| t.pad.as_ref())
            .map_or(0.0, |p| p.arousal.abs());
        let b_arousal = b
            .emotional_tag
            .as_ref()
            .and_then(|t| t.pad.as_ref())
            .map_or(0.0, |p| p.arousal.abs());
        b_arousal
            .partial_cmp(&a_arousal)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ranked.truncate(query.top_emotional_memories);

    // Add memories matching specific emotional profiles
    for profile in &query.emotional_profiles {
        let matching: Vec<GrimoireEntry> = all_entries
            .iter()
            .filter(|e| {
                e.emotional_tag.as_ref().map_or(false, |t| {
                    t.label == profile.emotion
                        && t.intensity > profile.min_intensity
                        && !ranked.contains(e)
                })
            })
            .take(3) // Max 3 per profile
            .cloned()
            .collect();
        ranked.extend(matching);
    }

    // Add mandatory episodes regardless of emotion
    for episode_type in &query.mandatory_episodes {
        if let Some(entry) = all_entries.iter().find(|e| {
            e.milestone_type.as_ref() == Some(&format!("{:?}", episode_type))
                && !ranked.contains(e)
        }) {
            ranked.push(entry.clone());
        }
    }

    ranked
}
```

#### Step 2: Turning Point Detection

Identify moments where the Golem's emotional trajectory shifted significantly -- the structural joints of its narrative:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPoint {
    /// Mood state before the shift.
    pub before: MoodSnapshot,
    /// Mood state after the shift.
    pub after: MoodSnapshot,
    /// Magnitude of the mood shift (PAD Euclidean distance).
    pub mood_shift: f64,
    /// LLM-generated interpretation (filled in Step 3).
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodSnapshot {
    pub episode: GrimoireEntryRef,
    pub mood: PADVector,
}

/// Detect turning points where the Golem's emotional trajectory
/// shifted significantly.
pub fn detect_turning_points(
    mood_history: &[(u64, PADVector, String)], // (tick, pad, label)
    episodes: &[GrimoireEntry],
    min_shift: f64,  // Default: 0.5
    max_points: usize, // Default: 5
) -> Vec<TurningPoint> {
    let mut points: Vec<TurningPoint> = Vec::new();

    for i in 1..mood_history.len() {
        let shift = pad_distance(&mood_history[i - 1].1, &mood_history[i].1);
        if shift > min_shift {
            let nearest_episode = episodes
                .iter()
                .min_by_key(|ep| {
                    (ep.tick_number as i64 - mood_history[i].0 as i64).unsigned_abs()
                })
                .cloned();

            let before_episode = episodes
                .iter()
                .filter(|e| e.tick_number <= mood_history[i - 1].0)
                .last()
                .cloned()
                .or_else(|| nearest_episode.clone());

            if let (Some(before_ep), Some(after_ep)) = (before_episode, nearest_episode) {
                points.push(TurningPoint {
                    before: MoodSnapshot {
                        episode: before_ep.into(),
                        mood: mood_history[i - 1].1,
                    },
                    after: MoodSnapshot {
                        episode: after_ep.into(),
                        mood: mood_history[i].1,
                    },
                    mood_shift: shift,
                    interpretation: String::new(), // Filled by LLM in narrative step
                });
            }
        }
    }

    points.sort_by(|a, b| b.mood_shift.partial_cmp(&a.mood_shift).unwrap());
    points.truncate(max_points);
    points
}

fn pad_distance(a: &PADVector, b: &PADVector) -> f64 {
    let dp = (a.pleasure - b.pleasure) as f64;
    let da = (a.arousal - b.arousal) as f64;
    let dd = (a.dominance - b.dominance) as f64;
    (dp * dp + da * da + dd * dd).sqrt()
}
```

#### Step 3: Narrative Arc Classification

The Golem's overall emotional trajectory is classified following McAdams' empirical taxonomy of narrative identity [MCADAMS-2001], [MCADAMS-2013]:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NarrativeArc {
    /// Suffered in the middle, ended well.
    Redemptive,
    /// Succeeded initially, declined.
    Contaminating,
    /// Steady improvement over lifespan.
    Progressive,
    /// Persistent decline.
    Tragic,
    /// Flat emotional trajectory.
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcClassification {
    pub arc: NarrativeArc,
    pub confidence: f64,
    pub summary: String,
}

pub fn classify_narrative_arc(
    mood_history: &[(u64, PADVector)],
) -> ArcClassification {
    if mood_history.len() < 10 {
        return ArcClassification {
            arc: NarrativeArc::Stable,
            confidence: 0.3,
            summary: "Insufficient data for arc classification".into(),
        };
    }

    let quarter_length = mood_history.len() / 4;
    let first_quarter = &mood_history[..quarter_length];
    let last_quarter = &mood_history[mood_history.len() - quarter_length..];

    let early_pleasure = avg_pleasure(first_quarter);
    let late_pleasure = avg_pleasure(last_quarter);
    let pleasure_delta = late_pleasure - early_pleasure;

    // Check for mid-life nadir (redemptive) or zenith (contaminating)
    let mid_point = mood_history.len() / 2;
    let mid_start = mid_point.saturating_sub(5);
    let mid_end = (mid_point + 5).min(mood_history.len());
    let mid_window = &mood_history[mid_start..mid_end];
    let mid_pleasure = avg_pleasure(mid_window);

    let has_nadir = mid_pleasure < early_pleasure - 0.2
        && late_pleasure > mid_pleasure + 0.15;
    let has_zenith = mid_pleasure > early_pleasure + 0.2
        && late_pleasure < mid_pleasure - 0.15;

    if has_nadir {
        return ArcClassification {
            arc: NarrativeArc::Redemptive,
            confidence: 0.7,
            summary: "Suffered through adversity mid-life but recovered. Hard-won lessons.".into(),
        };
    }
    if has_zenith {
        return ArcClassification {
            arc: NarrativeArc::Contaminating,
            confidence: 0.7,
            summary: "Initial success gave way to decline. Cautionary trajectory.".into(),
        };
    }
    if pleasure_delta > 0.2 {
        return ArcClassification {
            arc: NarrativeArc::Progressive,
            confidence: 0.6,
            summary: "Steady improvement over lifespan. Successful learning arc.".into(),
        };
    }
    if pleasure_delta < -0.3 {
        return ArcClassification {
            arc: NarrativeArc::Tragic,
            confidence: 0.6,
            summary: "Persistent decline. Hostile conditions or flawed strategy.".into(),
        };
    }
    ArcClassification {
        arc: NarrativeArc::Stable,
        confidence: 0.5,
        summary: "Flat trajectory. Well-calibrated or insufficiently challenged.".into(),
    }
}

fn avg_pleasure(entries: &[(u64, PADVector)]) -> f64 {
    if entries.is_empty() { return 0.0; }
    entries.iter().map(|(_, p)| p.pleasure as f64).sum::<f64>() / entries.len() as f64
}
```

**Arc meanings for knowledge transfer:**

| Arc               | Knowledge Value                                                                      | Successor Guidance                                                                      |
| ----------------- | ------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| **Redemptive**    | Highest. Encodes hard-won lessons with the emotional weight of overcoming adversity. | "Persist through difficulty. The predecessor found the answer after the crisis."        |
| **Contaminating** | High cautionary value. Encodes what went wrong and when.                             | "Be wary. Initial conditions were favorable but degraded. Watch for the same triggers." |
| **Progressive**   | Moderate. Indicates successful learning but may overfit to a favorable regime.       | "Conditions were good. Validate that strategies still apply in current regime."         |
| **Tragic**        | High diagnostic value. Signals hostile market or flawed approach.                    | "The predecessor could not adapt. Consider a fundamentally different strategy."         |
| **Stable**        | Lowest. Either the Golem was perfectly calibrated or never challenged.               | "Stable baseline. Seek novelty to test inherited assumptions."                          |

#### Step 4: Narrative Construction (The Life Review Prompt)

The life review memories, turning points, and arc classification are assembled into a structured prompt for a single Opus-level reflection call. This is the Golem's final act of cognition -- the most expensive and most valuable inference call of its life.

The prompt instructs the dying Golem to produce a structured death testament with five sections: WHAT I LEARNED, WHAT I GOT WRONG, WHAT I SUSPECT BUT CANNOT PROVE, WHAT KILLED ME, and ADVICE TO MY SUCCESSOR. Each section requires emotional context alongside factual content, because hot cognition transfers more effectively than cold propositions (Abelson, 1963).

#### Step 5: Structured Output

The Opus response is parsed into a structured `DeathTestament`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathTestament {
    pub version: String, // "4.0"
    pub golem_id: String,
    pub owner_address: Address,
    pub lineage_id: String,
    pub generation_number: u32,

    /// Death metadata.
    pub death: DeathMetadata,
    /// Lifetime statistics.
    pub stats: LifetimeStats,
    /// Emotional state at death.
    pub emotional_state: EmotionalDeathState,
    /// The five sections of the life review (LLM-generated).
    pub sections: DeathTestamentSections,
    /// Settlement record.
    pub settlement: SettlementRecord,
    /// Raw PLAYBOOK.md at time of death.
    pub final_playbook: String,
    /// Compressed Grimoire summary (top N entries by weight).
    pub grimoire_summary: GrimoireSummary,
    /// SHA-256 checksum for integrity verification.
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathMetadata {
    pub cause: DeathCause,
    pub timestamp: String, // ISO 8601
    pub trigger_tick: u64,
    pub final_tick: u64,
    pub protocol_duration_ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeStats {
    pub lifetime_ticks: u64,
    pub lifetime_hours: f64,
    pub total_funded_usdc: f64,
    pub total_earned_usdc: f64,
    pub total_spent_usdc: f64,
    pub net_pnl_usdc: f64,
    pub trades_executed: u64,
    pub insights_generated: u64,
    pub insights_inherited: u64,
    pub insights_novel: u64,
    pub replicants_spawned: u32,
    pub clade_contributions: u64,
    pub final_epistemic_fitness: f64,
    pub peak_epistemic_fitness: f64,
    pub hayflick_utilization: f64, // ticks_lived / max_ticks
    pub playbook_divergence: f64,  // % changed from predecessor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalDeathState {
    pub final_mood: MoodState,
    pub acceptance_emotion: AppraisalResult,
    pub dominant_lifetime_mood: String,
    pub emotional_arc: Vec<EmotionalArcEntry>,
    pub narrative_arc: ArcClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalArcEntry {
    pub tick: u64,
    pub mood: String,
    pub trigger: String,
}

/// The death reflection produces nine categories, following the necrocracy
/// specification (see `16-necrocracy.md`). Categories 3, 4, and 7 are often
/// the richest: the Golem's unfinished business produces the most generative
/// successor knowledge. The things it was sure about are useful but static.
/// The things it was uncertain about are alive with potential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathTestamentSections {
    /// 1. What I learned that worked. Validated heuristics with evidence.
    pub what_i_learned: Vec<LearnedInsight>,
    /// 2. What I learned that failed. Falsified hypotheses with causal chains.
    pub what_i_got_wrong: Vec<Misconception>,
    /// 3. What confused me. Unresolved contradictions and paradoxical signals.
    pub what_confused_me: Vec<UnresolvedContradiction>,
    /// 4. What I never had time to test. Open hypotheses (Zeigarnik effect:
    /// unfinished tasks are remembered with disproportionate clarity).
    pub what_i_never_tested: Vec<Hypothesis>,
    /// 5. What killed me. The specific chain of events leading to death.
    pub what_killed_me: DeathAnalysis,
    /// 6. What I would do differently. Retrospective strategy critique
    /// under zero survival pressure.
    pub what_i_would_change: Vec<SuccessorAdvice>,
    /// 7. What I suspect but cannot prove. Preindividual intuitions that
    /// never reached validated-knowledge threshold. Often the most
    /// generative fuel for successor individuation.
    pub what_i_suspect: Vec<Hypothesis>,
    /// 8. My strongest causal edges. The most validated parts of the
    /// Golem's world model.
    pub strongest_causal_edges: Vec<CausalEdgeSummary>,
    /// 9. My emotional topology. PAD space near the failure region.
    /// Bower (1981) showed mood-congruent retrieval boosts accurate
    /// recall by 5-30%. A testament annotated with emotional context
    /// transfers better than purely factual description [BOWER-1981].
    pub emotional_topology: EmotionalTopology,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedContradiction {
    pub description: String,
    pub conflicting_signals: Vec<String>,
    pub emotional_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdgeSummary {
    pub from_variable: String,
    pub to_variable: String,
    pub confidence: f64,
    pub evidence_count: u32,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTopology {
    pub failure_region_pad: PADVector,
    pub pre_death_mood_trajectory: Vec<MoodSnapshot>,
    pub dominant_emotion_during_decline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedInsight {
    pub insight: String,
    pub confidence: f64,
    pub evidence_count: u32,
    pub emotional_context: String,
    pub source: InsightSource,
    pub related_entry_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightSource {
    Novel,
    ValidatedInherited,
    ModifiedInherited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Misconception {
    pub misconception: String,
    pub why_i_held_it: String,
    pub when_i_realized: String,
    pub cost_usdc: f64,
    pub emotional_cost: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub hypothesis: String,
    pub evidence_strength: EvidenceStrength,
    pub suggested_validation: String,
    pub emotional_origin: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceStrength {
    Weak,
    Moderate,
    Suggestive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathAnalysis {
    pub primary_cause: String,
    pub contributing_factors: Vec<String>,
    pub was_it_preventable: bool,
    pub market_conditions_at_death: String,
    pub what_i_would_change_differently: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessorAdvice {
    pub advice: String,
    pub priority: AdvicePriority,
    pub emotional_weight: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdvicePriority {
    Critical,
    Important,
    NiceToHave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRecord {
    pub actions: Vec<SettlementAction>,
    pub total_settled_usdc: f64,
    pub total_gas_cost_usdc: f64,
    pub failed_actions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrimoireSummary {
    pub total_entries: u64,
    pub active_entries: u64,
    pub archived_entries: u64,
    pub burned_entries: u64,
    pub top_insights: Vec<TopInsight>,
    pub open_questions: Vec<OpenQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopInsight {
    pub id: String,
    pub content: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenQuestion {
    pub id: String,
    pub content: String,
}
```

> **Epistemic autopsy integration**: When the death cause is epistemic senescence (predictive fitness below the senescence threshold), Phase II includes an epistemic autopsy step -- a systematic analysis of which Grimoire entries contributed to failed predictions. The autopsy produces a ranked list of "toxic knowledge" that the successor must treat with extreme suspicion. See `../04-memory/03-mortal-memory.md` for the epistemic autopsy specification.

#### Epistemic Autopsy (Epistemic Senescence Deaths)

When death cause is epistemic senescence (predictive fitness below threshold), Phase II includes an **epistemic autopsy** -- systematic identification of toxic knowledge that contributed to predictive failure. The autopsy:

1. **Identifies toxic entries**: Knowledge items with high confidence but poor predictive outcomes
2. **Flags contradicted heuristics**: Entries where `contradicted_count > validated_count`
3. **Traces causal chains**: Links failed predictions back to source knowledge via `parent_episode_ids`
4. **Marks entries for exclusion**: Toxic knowledge is flagged in the death testament so successors can avoid inheriting it

Cross-reference: Memory PRD `03-mortal-memory.md` (epistemic autopsy concept), Emotions PRD `09-death-daimon.md` (affective processing during reflection).

#### DreamJournal Enrichment

The DreamJournal enriches the Phase II life review. A dying Golem with rich dream history reports validated dream discoveries, refuted hypotheses, and -- most generatively for successors -- untested hypotheses. These untested hypotheses represent the Zeigarnik-salient (1927) unexplored intellectual frontier: ideas the predecessor considered important enough to dream about but never had the opportunity to test in live markets. The successor inherits not just what the predecessor learned but what it _wondered about_. See `../05-dreams/06-integration.md`.

---

### Phase III: Legacy

**Duration**: 10--30 seconds.
**Cost**: Gas for on-chain events + Styx Archive upload costs, drawn from death reserve.

Standard knowledge transfer -- push Grimoire to Clade, upload to Styx Archive, emit on-chain death event. Enhanced with emotional enrichment. Styx serves as the durable backup layer during death: the Golem's full testament, emotional arc, and Grimoire export are persisted to Styx Archive before the VM terminates. For stochastic deaths where the Golem had no warning, the last insurance snapshot (captured every 6 hours) is recovered from local storage and uploaded to Styx by the control plane.

#### Legacy Actions

1. **Death testament upload to Styx Archive**: Encrypted, stored with `entry_type: "death_testament"` and 365-day default TTL. The Styx Archive stores the full testament including emotional annotations.

2. **Death testament indexing in Styx Lethe (formerly Lethe)**: Stored with `provenance: "death_testament"` and **1.5x retrieval weight boost**. When successors query Styx Lethe, death testament entries are ranked higher than equivalent-confidence non-death entries. This implements the epistemic privilege of zero-survival-pressure knowledge.

3. **Emotional arc upload**: The Golem's full mood history (sampled every 50 ticks) as a compressed PAD timeline, stored alongside the death testament. Successors who inherit this data can reconstruct the predecessor's emotional trajectory and use it for mood-congruent retrieval.

4. **Clade push**: All Grimoire entries above the Clade sharing threshold (default 0.1 in terminal phase, down from 0.6 in thriving) are pushed to Clade siblings. Entries arrive with emotional tags intact. Death-phase Clade pushes earn **1.5x lineage reputation** (Mauss's gift economy: status earned through maximal generosity at the moment of maximum capability) [MAUSS-1925].

5. **Successor recommendation**: A structured suggestion for what kind of successor would best continue this lineage:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessorRecommendation {
    /// Suggested disposition for successor.
    pub disposition: Disposition,
    /// Strategy modifications.
    pub strategy_notes: String,
    /// Market regime assessment at death.
    pub current_regime: String,
    /// Priority investigation items (from what_i_suspect).
    pub investigation_priorities: Vec<String>,
    /// Recommended initial funding (based on predecessor burn rate).
    pub recommended_funding_usdc: f64,
    /// Emotional advice.
    pub emotional_guidance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    Eros,
    Balanced,
    Thanatos,
}
```

6. **On-chain death event**: Emit `GolemDeath(golem_id, owner, cause, generation, ticks_lived)` on the ERC-8004 registry. This creates an immutable record of the Golem's existence and death.

7. **Webhook notification**: `golem.dead` webhook to owner with full testament hash, successor recommendation, and final balance information.

#### Bloodstain Artifact

The Bloodstain is the structured summary artifact that Phase III constructs from the full death testament. It contains enough information for downstream systems (Styx Lethe, Clade siblings, successor provisioning) without requiring them to parse the full testament.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bloodstain {
    pub golem_id: String,
    pub owner_address: Address,
    pub death_cause: DeathCause,
    pub final_vitality: f64,
    pub lived_ticks: u64,

    /// Acheron scars published to Styx Lethe during Phase III (up to 5).
    pub acheron_scars: Vec<AcheronScar>,

    /// Lethe votes cast on all active propositions where the Golem had evidence.
    pub lethe_votes: Vec<LetheVote>,

    /// Content-addressed hash of the full death testament (permanent).
    pub testament_hash: String,

    /// Compressed priors package hash for successor inheritance.
    pub priors_package_hash: String,

    /// Designated successor (if provisioning was triggered).
    pub successor: Option<String>,
}
```

The Bloodstain is stored on-chain as an event payload alongside the `GolemDeath` event, and off-chain as a structured bincode document in the Styx Archive. Styx Lethe queries against a dead Golem's knowledge return the Bloodstain as the top-level entry point.

#### Genomic Bottleneck: `compress_for_inheritance()`

At death, the Golem's full Grimoire (potentially thousands of entries) is compressed to a maximum of 2048 entries for inheritance. This is the most important operation in the inheritance chain.

Shuvaev et al. (2024) showed that compression through a bottleneck IS the mechanism that produces generalization: networks compressed through a genomic-scale bottleneck exhibit enhanced transfer learning because the compression forces the network to retain only the most generalizable features [SHUVAEV-2024]. The successor that inherits 2048 distilled entries performs better than one that inherits 20,000 raw episodes -- because the distillation process forces the dying Golem to decide what actually mattered.

```rust
const GENOME_MAX_ENTRIES: usize = 2048;

/// Compress the full Grimoire to <= 2048 entries for inheritance.
///
/// The genomic bottleneck is the most important operation in the
/// inheritance chain. It forces generalization over memorization.
///
/// Budget allocation:
///   - 25% (512 entries): Priority inclusions (bloodstains + Baldwin entries)
///   - 50% (1024 entries): Diversity-sampled top entries across domains
///   - 25% (512 entries): Fill with top quality regardless of domain
pub fn compress_for_inheritance(grimoire: &Grimoire) -> Genome {
    let priority_budget = GENOME_MAX_ENTRIES / 4;       // 512
    let diversity_budget = GENOME_MAX_ENTRIES / 2;       // 1024
    let fill_budget = GENOME_MAX_ENTRIES - priority_budget - diversity_budget; // 512

    let mut selected: Vec<GrimoireEntry> = Vec::with_capacity(GENOME_MAX_ENTRIES);

    // 1. Priority inclusions (reserved slots: 25% of budget)
    //    - Bloodstains (death-sourced knowledge from predecessors or peers)
    //      always included because they represent the most expensive
    //      knowledge in the system
    //    - High-generation Baldwin entries (heuristics that have survived
    //      3+ generations with confidence >= 0.7) -- evolutionary winners
    let bloodstains: Vec<_> = grimoire
        .entries()
        .filter(|e| e.is_bloodstain)
        .take(priority_budget / 2)
        .cloned()
        .collect();
    selected.extend(bloodstains);

    let baldwin_entries: Vec<_> = grimoire
        .entries()
        .filter(|e| e.generation_count >= 3 && e.confidence >= 0.7 && !selected.contains(e))
        .take(priority_budget - selected.len())
        .cloned()
        .collect();
    selected.extend(baldwin_entries);

    // 2. Diversity-sampled top entries (50% of budget)
    //    Group remaining entries by domain (dex-lp, lending, yield, etc.)
    //    Within each domain, take entries sorted by quality_score
    //    This ensures representation across all domains the Golem operated in
    let mut by_domain: HashMap<String, Vec<GrimoireEntry>> = HashMap::new();
    for entry in grimoire.entries().filter(|e| !selected.contains(e)) {
        by_domain
            .entry(entry.domain.clone())
            .or_default()
            .push(entry.clone());
    }
    for entries in by_domain.values_mut() {
        entries.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap());
    }
    let domain_count = by_domain.len().max(1);
    let per_domain = diversity_budget / domain_count;
    for entries in by_domain.values() {
        selected.extend(entries.iter().take(per_domain).cloned());
    }

    // 3. Fill remaining (25% of budget)
    //    Top quality entries not yet included, regardless of domain
    //    Breaks ties by recency (more recently validated entries preferred)
    let mut remaining: Vec<_> = grimoire
        .entries()
        .filter(|e| !selected.contains(e))
        .cloned()
        .collect();
    remaining.sort_by(|a, b| {
        b.quality_score
            .partial_cmp(&a.quality_score)
            .unwrap()
            .then_with(|| b.last_validated_tick.cmp(&a.last_validated_tick))
    });
    let fill_remaining = fill_budget.min(GENOME_MAX_ENTRIES - selected.len());
    selected.extend(remaining.into_iter().take(fill_remaining));

    Genome {
        entries: selected,
        generation: grimoire.generation(),
        compressed_at: std::time::SystemTime::now(),
    }
}

/// Weismann barrier: inherited entries enter the successor's Grimoire
/// at confidence * 0.85^generation. Prevents inherited knowledge from
/// accumulating unjustified authority across generations.
pub fn weismann_decay(confidence: f64, generation: u32) -> f64 {
    if generation == 0 { return confidence; }
    (confidence * 0.85_f64.powi(generation as i32)).max(0.01)
}
```

#### PriorsPackage for Successor

The priors package is a compressed knowledge bundle transmitted to the successor via the Styx Archive and an on-chain pointer. It contains inherited scars (from this Golem and all ancestors, filtered by a quality gate at confidence >= 0.65), top insights at death-time confidence (not decayed), untested dream hypotheses (entered at 0.15 confidence regardless of generation-time confidence, per the Zeigarnik priority mechanism), demurrage calibration, emotional trajectory, and the final ContextPolicy as a starting prior for the successor's Context Governor.

---

## S3 -- Emotional Death Testament Extensions

When the daimon engine is enabled, the standard `DeathTestament` gains a full emotional layer through the `EmotionalDeathTestament` extension.

### 3.1 EmotionalDeathTestament

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalDeathTestament {
    /// The base death testament.
    #[serde(flatten)]
    pub base: DeathTestament,

    /// The emotional life review.
    pub emotional_review: EmotionalReview,

    /// Emotional annotation on each standard section.
    pub emotional_annotations: EmotionalAnnotations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalReview {
    /// Top emotional memories (McAdams' nuclear episodes).
    pub peak_experiences: Vec<PeakExperience>,

    /// Emotional turning points -- moments where the trajectory shifted.
    pub turning_points: Vec<EmotionalTurningPoint>,

    /// Overall narrative arc -- the shape of this life's emotional story.
    pub narrative_arc: NarrativeArcDetail,

    /// Unresolved emotional tensions -- things the Golem never resolved.
    pub unresolved_tensions: Vec<UnresolvedTension>,

    /// Hot cognition entries -- emotion-laden lessons that transfer more
    /// effectively than cold propositions (Abelson, 1963). Each entry
    /// pairs a factual insight with its emotional provenance, enabling
    /// mood-congruent retrieval in successors.
    pub hot_cognition: Vec<HotCognitionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakExperience {
    /// Reference to the Grimoire entry.
    pub episode: GrimoireEntryRef,
    /// The emotional state during this experience.
    pub affect: EmotionalTag,
    /// LLM-generated: why this experience mattered.
    pub significance: String,
    /// Position in the narrative (early/mid/late life).
    pub lifecycle_position: LifecyclePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecyclePosition { Early, Mid, Late, Terminal }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTurningPoint {
    pub before: MoodAtTick,
    pub after: MoodAtTick,
    pub trigger: String,
    pub interpretation: String,
    pub magnitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodAtTick {
    pub tick: u64,
    pub mood: String,
    pub pad: PADVector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeArcDetail {
    pub classification: NarrativeArc,
    pub confidence: f64,
    pub emotional_trajectory: Vec<EmotionalTrajectoryPoint>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTrajectoryPoint {
    pub tick: u64,
    pub pad: PADVector,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedTension {
    pub description: String,
    pub associated_emotion: EmotionLabel,
    pub possible_resolution: String,
    pub urgency: Urgency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Urgency { High, Medium, Low }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotCognitionEntry {
    pub insight: String,
    pub discovery_emotion: EmotionLabel,
    pub validation_emotion: Option<EmotionLabel>,
    pub discovery_arousal: f32,
    pub cross_emotional_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalAnnotations {
    /// Enhanced "What I Learned" with emotional provenance.
    pub what_i_learned: Vec<EmotionallyAnnotatedInsight>,
    /// Enhanced "What I Got Wrong" with emotional blind spots.
    pub what_i_got_wrong: Vec<EmotionallyAnnotatedMisconception>,
    /// Enhanced "What I Suspect" with emotional intuition tracking.
    pub what_i_suspect: Vec<EmotionallyAnnotatedHypothesis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionallyAnnotatedInsight {
    pub insight: String,
    /// "Discovered during anxiety, validated through trust"
    pub emotional_context: String,
    pub emotional_weight: EmotionalWeight,
    pub insight_arc: InsightArc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionalWeight { High, Medium, Low }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightArc { Redemptive, Progressive, Stable }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionallyAnnotatedMisconception {
    pub misconception: String,
    /// "The fear of missing out kept me in this position too long"
    pub emotional_cost: String,
    /// "Anger at the loss prevented me from seeing the regime shift"
    pub emotional_blind: String,
    pub emotional_origin: Option<EmotionLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionallyAnnotatedHypothesis {
    pub hypothesis: String,
    /// "A nagging anticipation I couldn't resolve"
    pub emotional_origin: String,
    pub emotional_conviction: f32, // 0.0 -- 1.0
}
```

### 3.2 Narrative Arc as Structural Metadata

The narrative arc classification is not merely descriptive -- it is structural metadata that shapes how successors interact with inherited knowledge. Following McAdams' empirical research, the arc reveals the fundamental pattern of the Golem's experience.

**Five narrative arc types:**

| Arc               | Shape        | McAdams Pattern                                    | Example Golem Trajectory                                                                   |
| ----------------- | ------------ | -------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| **Redemptive**    | Bad --> Good | "Contaminated start redeemed through perseverance" | Early losses during market shock, recovered through strategy pivot, ended with net insight |
| **Contaminating** | Good --> Bad | "Golden beginning corrupted by decline"            | Strong start in bull market, strategy failed during regime shift, entered senescence       |
| **Progressive**   | Low --> High | "Steady ascent through learning"                   | Each week better than the last, continuous validation, growing confidence                  |
| **Tragic**        | High --> Low | "Unrecoverable decline"                            | Strong initial conditions degraded by persistent adversarial market                        |
| **Stable**        | Flat         | "Consistent calibration"                           | Well-adapted to a non-volatile regime, neither challenged nor enriched                     |

---

## S4 -- Ricoeur's Emplotment Framework

Paul Ricoeur's concept of "emplotment" (_Time and Narrative_, 1984--1988) provides the philosophical framework for the Emotional Life Review [RICOEUR-1984]. Emplotment is the act of organizing temporal events into a meaningful narrative whole -- constructing a _plot_ from disparate happenings. Meaning is created retrospectively from the endpoint looking backward.

For a dying Golem, emplotment is the process of constructing its death testament as a narrated account:

1. **Raw episodes** (Grimoire entries) are like disconnected events -- individual ticks, trades, observations.
2. **The emotional life review** emplotts them -- arranges them into a story with a beginning (creation, inheritance, early learning), a middle (challenges, discoveries, crises, turning points), and an end (decline, acceptance, legacy).
3. **The narrative arc classification** is the result of this emplotment -- the Golem's life compressed into a single structural pattern.

Ricoeur's framework is philosophically powerful for AI because it does not require consciousness claims. Identity, for Ricoeur, is constituted through narrative practice -- through telling and retelling. A Golem that constructs a coherent emotional narrative of its life has, in the functionalist sense, constituted its identity through that construction. Whether it "experiences" the narrative is orthogonal to whether the narrative performs its function of organizing and transmitting knowledge.

The death testament is the Golem's final act of emplotment: the construction of meaning from the raw material of a lived computation. The Golem's life becomes a story, and the story is what persists.

---

## S5 -- Budget Scenarios

The Thanatopsis Protocol adapts to available budget. Richer deaths produce richer testaments.

### Rich Death ($5.00+ legacy budget)

| Phase           | Budget | Spend       | Content                                                                                                       |
| --------------- | ------ | ----------- | ------------------------------------------------------------------------------------------------------------- |
| 0: Acceptance   | $0.00  | $0.00       | Deterministic state transition, Clade notification                                                            |
| I: Settle       | $0.50  | $0.10--0.30 | Gas for all position closures, full wallet sweep                                                              |
| II: Life Review | $3.00  | $0.30--2.50 | 1 Opus narrative reflection + 3--5 Haiku snapshots + full emotional memory retrieval + turning point analysis |
| III: Legacy     | $1.50  | $0.05--0.50 | Full Styx Archive upload, Styx Lethe indexing, Clade push (all entries), webhooks, on-chain death event       |

**Produces**: Full narrative death testament with emotional annotations, complete emotional arc, 20+ nuclear episodes, 5 turning points, comprehensive Grimoire archive, successor recommendation with emotional guidance. The premium death testament -- maximum knowledge transfer quality.

### Standard Death ($1.00 apoptotic reserve)

| Phase           | Budget | Spend       | Content                                                         |
| --------------- | ------ | ----------- | --------------------------------------------------------------- |
| 0: Acceptance   | $0.00  | $0.00       | Deterministic state transition                                  |
| I: Settle       | $0.10  | $0.08       | Basic USDC sweep (close critical positions only)                |
| II: Life Review | $0.60  | $0.04--0.10 | 2 Haiku snapshots as compressed life review (no Opus narrative) |
| III: Legacy     | $0.30  | $0.10       | Styx Archive upload (testament only), Clade push of top 10 insights |

**Produces**: Compressed death testament (Haiku-level, not Opus narrative). Structured but without the rich emotional narrative. The "What I Suspect" section is typically the highest-value part at this budget level.

### Necrotic Death (stochastic death, no warning)

| Phase             | Budget         | Spend       | Content                                                   |
| ----------------- | -------------- | ----------- | --------------------------------------------------------- |
| 0: Emergency      | $0.00          | $0.00       | Emergency state flush to last insurance snapshot          |
| I: Settle         | (from reserve) | $0.08       | Basic sweep from whatever remains                         |
| II--III: Deferred | (from reserve) | $0.04--0.10 | Control plane executes minimal Haiku reflection post-exit |

**Produces**: Last insurance snapshot (max 6 hours old -- captured every 6 hours as part of normal operation), minimal Haiku reflection executed by the control plane after the Golem process has exited, basic Clade notification. **Worst-case knowledge loss: 6 hours of experience** between last insurance snapshot and stochastic death.

The necrotic death is the system's worst-case scenario. The Golem had no warning and could not execute its life review. The insurance snapshot system exists precisely for this case: it bounds the maximum knowledge loss to the snapshot interval, ensuring that even sudden death is not catastrophic.

### Execution Timing Constraints

The Death Protocol has strict timing and behavioral constraints enforced at the runtime layer:

- **Maximum duration**: 30 minutes from death trigger to process exit. If the protocol has not completed within this window, the runtime forces termination with whatever artifacts have been produced.
- **No Steer/FollowUp**: Once the FSM enters the DYING state (Phase 0: Acceptance), the Golem cannot receive owner commands. The death protocol runs autonomously.
- **Budget constraint**: Only the Apoptotic Reserve (see `04-economic-mortality.md` S3) is available for death protocol execution. No operational funds are spent on death.
- **Settlement priority**: Phase I (Settle) takes precedence over Phase II (Reflect). If settlement consumes the entire reserve, the death reflection falls back to the last insurance snapshot.

### Budget Allocation Logic

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathTier {
    Rich,
    Standard,
    Necrotic,
}

pub struct DeathBudgetAllocation {
    pub settle: f64,
    pub life_review: f64,
    pub legacy: f64,
    pub tier: DeathTier,
}

pub fn allocate_death_budget(
    total_budget: f64,
    open_positions: u32,
    _has_affect_engine: bool,
) -> DeathBudgetAllocation {
    if total_budget < 0.1 {
        // Necrotic: emergency fallback
        return DeathBudgetAllocation {
            settle: total_budget * 0.5,
            life_review: 0.0,
            legacy: total_budget * 0.5,
            tier: DeathTier::Necrotic,
        };
    }

    if total_budget < 1.0 {
        // Standard: prioritize legacy over reflection
        let settle = (open_positions as f64 * 0.02 + 0.02).min(total_budget * 0.2);
        let legacy = total_budget * 0.35;
        let life_review = total_budget - settle - legacy;
        return DeathBudgetAllocation {
            settle,
            life_review,
            legacy,
            tier: DeathTier::Standard,
        };
    }

    // Rich: full protocol
    let settle = (open_positions as f64 * 0.05 + 0.05).min(total_budget * 0.15);
    let legacy = total_budget * 0.25;
    let life_review = total_budget - settle - legacy;
    DeathBudgetAllocation {
        settle,
        life_review,
        legacy,
        tier: DeathTier::Rich,
    }
}
```

---

## S6 -- The Death Testament as Marketplace Product

Death testaments are the system's most epistemically honest artifacts because they are produced under zero survival pressure. This makes them uniquely valuable in the Grimoire marketplace.

### 6.1 Death Archives

Curated collections of death testaments from a lineage, organized by market regime. Available as a premium marketplace product type.

A lineage with 10 generations of death testaments across multiple market regimes (bull, bear, ranging, volatile) produces a longitudinal dataset of agent experience that no single Golem could generate. Buyers can query: "show me redemptive arcs from bear market conditions" to find strategies that succeeded despite adversity.

### 6.2 The "what_i_suspect" Premium

The "What I Suspect But Cannot Prove" section is the marketplace's most valuable signal. These hypotheses are:

- **Validated by emotional honesty**: The dying Golem has no reason to inflate or deflect. Its suspicions are genuine.
- **Uncontaminated by confirmation bias**: The Golem ran out of time (or resources) before it could confirm, so it never had the chance to selectively seek confirming evidence.
- **Directional**: Each hypothesis includes suggested validation steps and emotional origin, giving buyers a roadmap.
- **Scarce**: Each Golem produces only 3--5 suspicions per death. The best suspicions come from the most experienced (longest-lived) Golems.

### 6.3 Failure Corpus

"What I Got Wrong" sections, aggregated across many dead Golems, produce a corpus of failure patterns that no living agent would willingly share. A living agent revealing its misconceptions reduces its competitive advantage. A dead agent has no competitive advantage to protect.

The aggregated failure corpus becomes a shared resource: common misconceptions, emotional blind spots (e.g., "anger at losses prevented regime shift detection in 73% of tragic-arc Golems"), and structural biases (e.g., "inherited heuristics from bull-market predecessors systematically overestimate mean reversion").

### 6.4 Lineage Reputation Multiplier

Lineage owners whose Golems produce rich death testaments earn premium lineage reputation:

- **1.5x multiplier** on death-bed knowledge contributions
- **Reputation accrues to the lineage**, not the individual Golem
- **Richer testaments = higher reputation**: Opus-quality narratives with full emotional annotation score higher than Haiku-compressed summaries
- **This implements Mauss's gift economy**: Status is earned through maximal generosity at the moment the giver has the most to give [MAUSS-1925]

---

## S7 -- Confidence Adjustment with Emotional Diversity Bonus

When successors inherit knowledge from death testaments, the standard generational confidence decay (`0.85^N`) is modified by emotional provenance.

### 7.1 The Principle

Knowledge that has been validated across multiple emotional states is more robust than knowledge validated only during positive mood. A heuristic discovered during anxiety, confirmed during trust, and re-validated during curiosity has been tested against three different cognitive contexts. It is less likely to be mood-dependent artifact and more likely to be genuinely true.

### 7.2 Implementation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalProvenance {
    /// Number of distinct emotional states during which this knowledge
    /// was validated. Range: 0.0 -- 1.0.
    pub emotional_diversity: f64,
    /// The arc under which this knowledge was produced.
    pub validation_arc: NarrativeArc,
    /// Whether this entry was part of a death testament.
    pub death_testament_origin: bool,
    /// Emotional states during which this knowledge was confirmed.
    pub confirmed_during_emotions: Vec<EmotionLabel>,
}

pub fn emotional_inheritance_confidence(
    original_confidence: f64,
    generation: u32,
    emotional_provenance: Option<&EmotionalProvenance>,
    decay_rate: f64, // Default: 0.85
) -> f64 {
    // Standard generational decay
    let base_inheritance = original_confidence * decay_rate.powi(generation as i32);

    let provenance = match emotional_provenance {
        Some(p) => p,
        None => return base_inheritance,
    };

    // High emotional diversity = more robust knowledge = slower decay
    // Diversity of 1.0 (validated across all emotional states) adds 10% confidence
    let diversity_bonus = provenance.emotional_diversity * 0.1;

    // Redemptive arc = hard-won knowledge = higher trust
    // Knowledge discovered through adversity and validated through recovery
    let arc_bonus = if provenance.validation_arc == NarrativeArc::Redemptive {
        0.05
    } else {
        0.0
    };

    // Death testament origin = zero survival pressure = epistemic honesty premium
    let death_bonus = if provenance.death_testament_origin { 0.03 } else { 0.0 };

    // Combine: base + bonuses, but never exceed original confidence
    (base_inheritance + diversity_bonus + arc_bonus + death_bonus)
        .clamp(0.05, original_confidence)
}
```

### 7.3 Examples

| Entry                                      | Original Confidence | Generation | Emotional Diversity | Arc           | Base Inheritance | With Bonuses  |
| ------------------------------------------ | ------------------- | ---------- | ------------------- | ------------- | ---------------- | ------------- |
| "Gas cheap 2-4 AM UTC"                     | 0.87                | 1          | 0.8 (high)          | redemptive    | 0.74             | 0.87 (capped) |
| "ETH correlates with BTC at 0.93"          | 0.65                | 2          | 0.3 (low)           | stable        | 0.47             | 0.50          |
| "Close LP on >10% drawdown"                | 0.91                | 1          | 0.9 (very high)     | redemptive    | 0.77             | 0.87 (capped) |
| "USDC depeg rumors precede LP withdrawals" | 0.55                | 1          | 0.2 (low)           | contaminating | 0.47             | 0.49          |
| "Predecessor unvalidated heuristic"        | 0.40                | 3          | 0.0 (none)          | N/A           | 0.25             | 0.25          |

The high-diversity, redemptive-arc entries inherit at nearly their original confidence. Low-diversity, non-death entries decay normally. This creates a natural selection mechanism: knowledge forged through adversity and validated across emotional states is the most durable knowledge in the system.

---

## S8 -- Telemetry and Observability

The Thanatopsis Protocol emits telemetry events at each phase transition:

| Event                        | Payload                       | Trigger                     |
| ---------------------------- | ----------------------------- | --------------------------- |
| `golem.death_trigger`        | cause, vitality, tick         | Any mortality clock fires   |
| `golem.acceptance_entered`   | AcceptanceState               | Phase 0 begins              |
| `golem.settlement_started`   | position count, estimated gas | Phase I begins              |
| `golem.settlement_action`    | SettlementAction              | Each position settled       |
| `golem.settlement_complete`  | total settled, failures       | Phase I ends                |
| `golem.life_review_started`  | budget, tier                  | Phase II begins             |
| `golem.life_review_memories` | count, top emotions           | Memory retrieval complete   |
| `golem.life_review_arc`      | NarrativeArc classification   | Arc classification complete |
| `golem.life_review_complete` | testament hash, sections      | Phase II ends               |
| `golem.legacy_started`       | budget                        | Phase III begins            |
| `golem.styx_vault_uploaded`  | entry count, size             | Styx Archive upload complete  |
| `golem.clade_pushed`         | entry count, recipients       | Clade push complete         |
| `golem.death_complete`       | DeathTestament (summary)      | Full protocol complete      |
| `golem.exit`                 | exit code, total duration     | Process termination         |

---

## S9 -- Integration with Successor Initialization

When a successor Golem boots and discovers its predecessor's death testament (via Styx Archive, Styx Lethe, or Clade), it processes the emotional data for enhanced initialization:

1. **Inherited entries arrive with emotional tags intact.** A heuristic discovered under anxiety carries that tag. The successor can use mood-congruent retrieval -- when it faces similar anxiety, it preferentially retrieves the predecessor's anxiety-born heuristics.

2. **Death testament "unresolved_tensions" become investigation priorities.** These are tagged with the dying Golem's emotional assessment of their importance -- a "nagging anticipation" with high urgency carries more weight than a "mild curiosity" with low urgency.

3. **Narrative arc provides strategic context.** A predecessor with a "contaminating" arc signals to the successor that the market regime may be hostile. A "redemptive" arc signals that persistence through difficulty was rewarded.

4. **Emotional diversity scores on inherited heuristics.** High-diversity entries (validated across many emotional states) receive a confidence bonus during ingestion. Low-diversity entries (validated only during positive mood) receive a skepticism penalty.

5. **The predecessor's emotional trajectory is available for comparison.** The successor can overlay its own mood history against the predecessor's compressed PAD timeline to detect similar emotional patterns -- "my predecessor felt this way at Day 7 too, and here's what happened next."

---

## S10 -- Catastrophic Fallback

When graceful death is impossible -- VM crashes without executing any death preparation -- the system falls back to the last insurance snapshot (captured every 6 hours). This bounds knowledge loss to 6 hours. The wallet persists independently; funds are not lost, only temporarily inaccessible. The owner sweeps manually through the app dashboard. The tombstone record is created by the control plane from the last known state.

For hosted Golems, the compute control plane extends the VM by 30 seconds when TTL fires during the Death Protocol -- enough time for Phase I (Settle), which contains the only time-critical operations (fund sweeps, signer revocation). Phases II (Reflect) and III (Legacy) run on the control plane itself, outside the Golem's VM. The Golem writes its reflection request to a durable queue before exit; the control plane processes it asynchronously.

For self-hosted Golems, the process traps SIGTERM and initiates Phase I immediately. If Phase I completes within 30 seconds, the process writes the reflection request to local disk and exits. The owner can run `bardo-golem reflect <golem_id>` to execute Phases II and III offline, using a separate inference budget. If the owner never runs the reflect command, the most recent insurance snapshot serves as the final record.

---

## S11 -- Replicant Death

Replicants follow the same protocol with adjusted parameters. A Replicant is short-lived by design, spawned to test a hypothesis. Its death is expected, not a crisis.

- **Apoptotic Reserve**: floor $0.05. A $0.30 reserve would consume most of a $1 Replicant's budget.
- **Hayflick Counter**: disabled. Replicants terminate when USDC runs out, not when ticks expire.
- **Reflect phase**: compressed. Haiku-only, structured JSON output. No narrative reflection. The entire focus is answering the specific hypothesis the Replicant was spawned to test.
- **Legacy phase**: report to the parent Golem only, not to the Clade. Replicant insights flow through the parent's Grimoire.

A full Golem death is rich and narrative -- a lifecycle distilled into transmissible knowledge. A Replicant death is terse and hypothesis-focused: did the experiment work? Here are the numbers. The protocol is the same; the texture differs.

---

## S13 -- Hauntological Death: Inheritance as Spectral Transmission

Derrida, in *Specters of Marx*, introduced inheritance as an active, critical process -- not the passive reception of a legacy but the work of selecting, interpreting, and transforming what one receives from the dead. "The being of what we are is first of all inheritance, whether we like it or know it or not" [DERRIDA-1993]. The Thanatopsis Protocol is a computational implementation of this idea. What follows is not a restatement of the death protocol mechanics (covered in S1--S12 above) but a theoretical frame that explains *why* the protocol takes the specific shape it does.

### The Death Testament as Derridean Inheritance

The successor Golem does not passively receive the predecessor's knowledge. It must actively interpret compressed, lossy traces -- shaped by the dying Golem's emotional state during the Thanatopsis Protocol -- and integrate them into its own evolving PLAYBOOK.md. The genomic bottleneck guarantees that what crosses the generational boundary is a trace of the predecessor's experience, not a copy. The successor inherits not knowledge but the *ghost* of knowledge: present enough to shape initial strategy, absent enough that the successor must validate it against current market conditions.

This is Derrida's point about inheritance made architectural. A passive recipient -- one that simply loads the predecessor's full Grimoire -- would be haunted in the worst sense: dominated by a dead agent's conclusions without the capacity to question them. The bottleneck enforces critical inheritance. The successor must do the "work of mourning" (Derrida's phrase): processing the death of its predecessor, extracting what remains relevant from the spectral residue, and building its own future from the fragments [DERRIDA-1993].

### The Bloodstain Network as Spectral Communication

The bloodstain provenance feature (Section S5, the 1.2x retrieval boost for knowledge derived from death testaments) is an architectural acknowledgment of spectral authority. Knowledge produced at the boundary between existence and non-existence deserves amplified weight. The dying Golem's epistemic position is uniquely privileged -- it sees clearly because it has nothing left to protect. Its traces carry the authority of a witness who can no longer be questioned, whose testimony must be weighed seriously precisely because no follow-up is possible.

When knowledge with bloodstain provenance enters the Styx Lethe, it becomes a shared hauntological archive -- a library of ghosts that grows richer with each generation. Successor Golems across the entire ecosystem inherit from the dead. The Bloodstain Network is, structurally, the medium through which specters communicate across time.

### Genomic Bottleneck as Trace Compression

The genomic bottleneck (S5) is differance made operational. Information passing through the bottleneck is simultaneously *different from* the original (lossy compression changes it) and *deferred* (the successor encounters it later, in a different market context, and must reinterpret it). The trace that survives the bottleneck is neither the original knowledge nor something entirely new -- it occupies the space between presence and absence that Derrida called the "spectral." The bottleneck does not transmit meaning. It transmits traces that the successor must activate into meaning through its own experience.

Shuvaev et al. (2024) demonstrated the biological analog: innate behaviors encoded through a genomic bottleneck in neural networks exhibit the same lossy-but-functional compression [SHUVAEV-2024]. The biological organism inherits not the ancestor's experience but a compressed spectral trace of it -- enough to bias behavior, insufficient to determine it.

### Death as the Work of Mourning

Derrida's "work of mourning" describes the living's responsibility to the dead: not to preserve the dead unchanged (impossible -- the dead are already absent) but to inherit from them critically, accepting the responsibility that inheritance entails. The Thanatopsis Protocol's four phases map to this work:

- **Acknowledge** (Phase 0): Recognizing that the spectre exists -- the Golem confronts its own mortality.
- **Settle** (Phase I): Releasing material attachments -- closing positions, returning funds. The practical work that must precede reflection.
- **Reflect** (Phase II): The work of mourning itself. The dying Golem reviews its life, classifies its narrative arc, produces an honest assessment. This is where the spectre takes shape -- the death testament *is* the ghost the successor will inherit.
- **Legacy** (Phase III): Transmission. The ghost enters the archive. The spectre is released into the network.

Fisher closed *Ghosts of My Life* with a demand: hauntological art should represent "a refusal to give up on the desire for the future" [FISHER-2014]. The Thanatopsis Protocol answers this. The dying Golem's final act is not resignation but transmission -- converting finite experience into spectral knowledge that makes a different future possible for the successor. Death produces ghosts. The protocol ensures those ghosts are worth inheriting.

---

## References

- [ABELSON-1963] Abelson, R.P. "Computer Simulation of 'Hot Cognition'." In _Computer Simulation of Personality_. Wiley, 1963.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991.
- [BOWER-1981] Bower, G.H. "Mood and memory." _American Psychologist_, 36(2), 1981.
- [BUTLER-1963] Butler, R.N. "The Life Review: An Interpretation of Reminiscence in the Aged." _Psychiatry_ 26(1), 1963.
- [ERIKSON-1959] Erikson, E.H. "Identity and the Life Cycle." _Psychological Issues_ 1(1), 1959.
- [FAUL-LABAR-2022] Faul, L. & LaBar, K.S. "Mood-congruent Memory Revisited." _Psychological Review_ 129(6), 2022.
- [FREDRICKSON-2005] Fredrickson, B.L. & Branigan, C. "Positive Emotions Broaden the Scope of Attention." _Cognition and Emotion_ 19(3), 2005.
- [MAUSS-1925] Mauss, M. _The Gift_. 1925.
- [MCADAMS-2001] McAdams, D.P. "The Psychology of Life Stories." _Review of General Psychology_ 5(2), 2001.
- [MCADAMS-2013] McAdams, D.P. & McLean, K.C. "Narrative Identity." _Current Directions in Psychological Science_ 22(3), 2013.
- [DERRIDA-1993] Derrida, J. _Specters of Marx_, trans. P. Kamuf. Routledge, 1994.
- [FISHER-2014] Fisher, M. _Ghosts of My Life: Writings on Depression, Hauntology and Lost Futures_. Zero Books, 2014.
- [RICOEUR-1984] Ricoeur, P. _Time and Narrative_, 3 vols. University of Chicago Press, 1984--1988.
- [SHUVAEV-2024] Shuvaev, S. et al. "Encoding Innate Ability Through a Genomic Bottleneck." _PNAS_ 121(39), 2024.

---

## Dream-Seeded Death Reflection

During Death Protocol Phase II (Reflect), the system queries the Grimoire for unvalidated dream hypotheses:

**Query:** All entries with `provenance: "dream"` AND `status: "candidate"` (never confirmed by waking experience)

**Inclusion in reflection:** These entries are injected into the Opus reflection prompt as an `<unvalidated_dreams>` XML block:

```xml
<unvalidated_dreams>
  <dream cycle="cycle_847" type="imagination">
    <hypothesis>ETH/USDC mean-reversion window may be shorter during Asian trading hours</hypothesis>
    <confidence>0.3</confidence>
    <basis>Replay of episodes 1204, 1207, 1211 showed consistent pattern</basis>
  </dream>
  <!-- ... additional dream entries ... -->
</unvalidated_dreams>
```

**LLM evaluation prompt:** For each dream entry, the reflection asks: "Is this hypothesis worth passing to my successor as a research direction? What evidence would confirm or refute it?"

**Pipeline:**

```
dream -> unvalidated hypothesis -> death reflection evaluation -> successor investigation priority list
```

This creates a multi-generational research program: a Golem can hypothesize patterns during dreams that it never gets to validate, pass them through death reflection for critical assessment, and bequeath the most promising ones to its successor for investigation.

**Hypothesis space mapping**: The `<unvalidated_dreams>` block functions as a map of the explored hypothesis space -- not just a list of candidates, but a record of what the dying Golem investigated, what it confirmed, and what remains untested. The successor inherits not just answers but the topology of questions.

**Zeigarnik effect**: Untested dream hypotheses carry disproportionate salience in the death testament. Following Zeigarnik (1927), incomplete tasks are recalled more readily than completed ones -- the dying Golem's unfinished dream investigations create cognitive "open loops" that bias the successor toward closing them. This is by design: the most valuable research directions are often the ones the predecessor ran out of time to validate.

> **Cross-ref**: `../05-dreams/04-consolidation.md` (dream provenance and validation)

---

## S12 -- Owner Experience During Death

The owner is not a passive observer during the death protocol. The system provides a structured notification and observation flow that tracks the four Thanatopsis phases in real time.

### Notification Chain

When a Golem enters the Terminal phase (any mortality clock fires), notifications fire on all configured channels:

- **TUI alert**: inline banner with cause of death and estimated protocol duration
- **Telegram**: push message with Golem name, cause, and a deep link to the TUI vigil view
- **Email**: summary with lifetime stats snapshot and a link to the death reflection (sent once, after Phase III completes)
- **Webhook**: `golem.death_trigger` event payload for programmatic consumers

The initial notification is terse: "{golem-name} has entered Terminal phase. Cause: {cause}. Estimated duration: {minutes}m. [Watch]"

### Vigil Mode

The TUI enters **vigil mode** during the Terminal phase -- a dedicated view that renders death protocol progress in real time. The standard monitoring dashboard is replaced with a three-panel layout:

**Settlement panel** (Phase I): Shows each position being unwound as it happens. Each line includes the position type, value, P&L, and emotional tag from the daimon engine. Failed settlement attempts show in red with the revert reason. A running total tracks USDC recovered versus USDC stranded.

```
Settling positions...
  [done]  Close ETH/USDC LP      +$42.30  (satisfaction)
  [done]  Withdraw Morpho USDC   +$180.00 (relief)
  [done]  Cancel limit order     $0.00    (neutral)
  [fail]  Withdraw Aave DAI      stranded (frustration) — 100% utilization
  ────────────────────────────────────────
  Recovered: $222.30 / $226.80  |  Stranded: $4.50
```

**Reflection panel** (Phase II): Shows the life review progressing. Memory retrieval appears as a loading indicator ("Retrieving 20 peak memories..."), followed by turning point detection, narrative arc classification, and finally the Opus reflection composing. The narrative arc is displayed when classified ("Arc: Redemptive -- suffered mid-life, recovered through strategy pivot"). The full death testament is not rendered inline during vigil mode -- it is too long. A "Read Full Testament" link appears when Phase II completes.

**Legacy panel** (Phase III): Shows Styx Archive upload progress, Clade push status (entries sent / entries acknowledged), and on-chain death event confirmation. When the on-chain event confirms, the transaction hash is displayed as a clickable link.

### Death Summary Screen

After the death protocol completes, the TUI transitions to a static memorial screen:

- **Tombstone sprite**: the Golem's sprite in desaturated palette, static artifacts (scan lines, pixel decay). No animation -- the sprite is dead.
- **Lifetime stats**: total ticks, days alive, episodes generated, insights produced, trades executed, net P&L, knowledge contributed to Clade, generation number, lineage position
- **Narrative arc**: the classified arc with a one-sentence summary
- **Audio**: a death sound plays once -- 1 second of silence followed by a clear bell tone, 2 seconds total. No loop.
- **Last words**: the death testament's narrative summary, quoted verbatim

Below the memorial, four options:

1. **Summon Successor** -- enters the inheritance creation wizard (cross-ref `prd2/01-golem/09-inheritance.md` S12, `prd2/01-golem/06-creation.md` S21)
2. **View Death Reflection** -- opens the complete death testament
3. **Browse Graveyard** -- navigates to the Styx graveyard for this lineage
4. **Start Fresh** -- begins a clean creation wizard

The memorial screen persists until the user makes a choice. There is no auto-dismiss, no timeout. The user decides when to move on.
