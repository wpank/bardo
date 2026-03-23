# Dream Architecture: The Three-Phase Cycle [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `00-overview.md`, `golem-core` (VitalityState, BehavioralPhase), `golem-grimoire`, `golem-inference`

---

> **Reader orientation:** This document specifies the three-phase dream cycle architecture (NREM replay, REM imagination, Integration) for a Golem (mortal autonomous agent) within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers the DreamScheduler that gates cycle initiation, the DreamState machine, compute budget tiers ([CORE] at T0 (cached/rule-based) vs [HARDENED] at T1/T2 (LLM-powered)), and full configuration. Prerequisites: the Dreams overview (`00-overview.md`), VitalityState (composite survival score 0.0-1.0), and BehavioralPhase (five survival phases). For a full glossary, see `prd2/shared/glossary.md`.

## The Three-Phase Cycle

Biological sleep alternates between NREM and REM phases in ~90-minute cycles, with each phase serving distinct cognitive functions. Deperrois et al. (2022) formalized this as a three-phase learning architecture: wakefulness (encode), NREM perturbed dreaming (replay with perturbations for robustness), and REM adversarial dreaming (creative generation essential for semantic concept extraction) [DEPERROIS-2022]. Removing the adversarial REM phase severely degraded semantic learning — both phases are necessary.

The Golem's dream cycle implements this directly:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              DREAM CYCLE                                     │
│                                                                              │
│ ┌───────────┐ ┌──────────┐ ┌──────────────┐ ┌───────────────┐ ┌──────────┐ │
│ │  PHASE 0  │►│  PHASE 1 │►│   PHASE 2    │►│   PHASE 3     │►│ PHASE 4  │ │
│ │ HYPNA-    │ │  NREM    │ │   REM        │ │   INTEGRATION │ │ HYPNO-   │ │
│ │ GOGIC     │ │          │ │              │ │               │ │ POMPIC   │ │
│ │ ONSET     │ │ Replay   │ │ Imagination  │ │ Consolidation │ │ RETURN   │ │
│ │           │ │ Compress │ │ Counterfact. │ │ PLAYBOOK.md   │ │          │ │
│ │ Context   │ │ Credit   │ │ Creative     │ │ Grimoire      │ │ Recryst- │ │
│ │ dissolve  │ │ Triage   │ │ Threats      │ │ Journal       │ │ allize   │ │
│ │ Waking    │ │ Perturb  │ │ Emotion      │ │ Validation    │ │ Dali     │ │
│ │ residue   │ │          │ │ Depotentiate │ │               │ │ interrupt│ │
│ └───────────┘ └──────────┘ └──────────────┘ └───────────────┘ └──────────┘ │
│                                                                              │
│ Budget: 10%   Budget: 35-40%  Budget: 25-35%  Budget: 15-20%   Budget: 5%   │
└──────────────────────────────────────────────────────────────────────────────┘
```

> **Cross-ref**: The onset and return phases are specified in detail in `../06-hypnagogia/02-architecture.md`. This document covers the three core phases; hypnagogia covers the two liminal phases that bracket them.

**Budget allocation by behavioral phase:**

| Phase | Onset | NREM (Replay) | REM (Imagination) | Consolidation | Return |
|-------|-------|---------------|-------------------|---------------|--------|
| Thriving | 10% | 34% | 30% | 21% | 5% |
| Stable | 10% | 30% | 30% | 25% | 5% |
| Conservation | 10% | 25% | 20% | 40% | 5% |
| Declining | 10% | 20% | 16% | 49% | 5% |
| Terminal | 10% | 16% | 12% | 57% | 5% |

Onset and return budgets are fixed at 10% and 5% respectively across all phases -- the liminal transitions are structurally necessary regardless of behavioral state. The remaining 85% shifts from exploration (NREM/REM) toward consolidation as the Golem ages. A Declining Golem spends nearly half its dream budget integrating and preserving knowledge for the death testament, while a Thriving Golem explores aggressively.

### Phase 1: NREM-like (Replay and Consolidation)

**Biological basis**: During NREM sleep, hippocampal sharp-wave ripples (SPW-Rs) compress minutes of waking experience into ~100ms bursts (140–200 Hz oscillations), replaying sequences in both forward and reverse order [BUZSAKI-2015]. Wilson and McNaughton (1994) demonstrated that place cells co-firing during waking spatial experience exhibit increased co-firing during subsequent slow-wave sleep [WILSON-MCNAUGHTON-1994]. Replay intensity declines across successive sleep sessions, suggesting synaptic modifications during waking create the substrate for sleep replay.

**LLM-native implementation**: The ReplayEngine selects episodes from the Grimoire based on Mattar-Daw utility weighting: utility = gain x need. Gain measures prediction error magnitude (surprise, PnL magnitude, counterfactual regret). Need measures regime match probability (embedding similarity to current market, regime match, recency). Episodes are scored, filtered by utility > 0.1, sorted, and the top-N form the replay batch. Curator-tagged episodes (those marked `dream_priority: "high"` during waking housekeeping) receive a 2x utility boost. An additional 20% of replay slots are reserved for diversity injection: at least one episode from each market regime seen in the last 30 days, one from the oldest third of the buffer, and one high-arousal episode regardless of utility score.

Each batch is processed through LLM inference in compressed form. Each batch presents multiple episodes together, asking the LLM to identify cross-episode patterns, extract causal chains, and assign credit for outcomes. Each retrieved episode gets `strength += 0.5` in LanceDB after replay (reduced from +1.0 for live retrieval, reflecting that dream validation is LLM-mediated pattern recognition without on-chain grounding).

**Operations in Phase 1**:

1. **Prioritized replay** — Select top-N episodes by utility score (gain × need). Process in batches of 5–10 episodes per inference call. Extract patterns not visible at encoding time.

2. **Reverse replay (credit assignment)** — For completed trades with known outcomes, trace backward: outcome → final action → preceding conditions → initial decision. Foster and Wilson (2006) showed reverse replay propagates reward information backward along the trajectory [FOSTER-WILSON-2006]. Ambrose et al. (2016) found reverse replay rate increases proportionally to reward magnitude [AMBROSE-2016].

3. **Forward replay (planning)** — For current positions and anticipated market conditions, project forward: current state → likely developments → optimal responses. Diba and Buzsáki (2007) showed forward sequences occur in anticipation of action [DIBA-BUZSAKI-2007].

4. **Perturbed replay** — Replay episodes with injected perturbations: simulated slippage, latency spikes, gas price changes, data dropout. This builds robustness by exposing the Golem's strategies to variations it has not yet encountered in live trading [DEPERROIS-2022].

5. **Memory triage** — Following Stickgold and Walker (2013), classify replayed episodes: preserve (high utility, validated), abstract (extract rule, discard details), forget (low utility, contradicted) [STICKGOLD-WALKER-2013]. Triage decisions feed into the Integration phase.

**Typical prompt structure for NREM replay**:

```
You are reviewing a batch of recent trading episodes during an offline
consolidation phase. For each episode:
1. What was the market state, the action taken, and the outcome?
2. Looking back, what pattern connects these episodes?
3. What would have been different if [perturbation]?
4. Which episodes should be preserved, abstracted into a rule, or forgotten?

Episodes:
[serialized episode batch from Grimoire]

Current PLAYBOOK.md context:
[relevant PLAYBOOK.md sections]
```

**Cost**: T1 inference (Haiku/Sonnet). NREM replay does not require deep reasoning — it processes structured data and extracts patterns. T2 escalation only for episodes flagged as high-complexity (multi-step DeFi interactions, cross-protocol dependencies).

### Phase 2: REM-like (Imagination and Creation)

**Biological basis**: During REM sleep, the brain generates novel scenarios by recombining elements of past experience in a neurochemical milieu (high acetylcholine, low noradrenaline, low serotonin) that enables creative associations while depotentiating emotional charge [WALKER-VAN-DER-HELM-2009]. Perogamvros et al. (2013) showed increased amygdala and dopaminergic activity during REM promotes consolidation of high-value memories while enabling creative recombination [PEROGAMVROS-2013]. Hobson, Hong, and Friston (2014) described the sleeping brain as an active inference system sampling its own generated virtual reality [HOBSON-HONG-FRISTON-2014].

**LLM-native implementation**: The ImaginationEngine uses the LLM's generative capabilities to produce novel scenarios, counterfactual analyses, and creative strategy recombinations. Unlike NREM's data-driven replay, REM-like imagination is generative — it creates episodes that never happened.

**Operations in Phase 2**:

1. **Counterfactual reasoning** — For key decisions in the replay set, generate "what if" alternatives. What if I had entered earlier/later? What if position size was different? What if the market had moved the other way? Pearl's (2000/2009) structural causal model framework provides the formal foundation: move beyond correlational P(Y|X) to interventional P(Y|do(X)) [PEARL-2000]. See `03-imagination.md` for full specification.

2. **Creative recombination** — Boden's (2004) three creativity modes [BODEN-2004]:
   - _Combinational_: blend elements from different strategies. "What if momentum entry signals were combined with mean-reversion exit signals?"
   - _Exploratory_: push existing strategies to boundary conditions. "What happens to this strategy under extreme gas prices? Under zero liquidity?"
   - _Transformational_: challenge fundamental assumptions. "What if impermanent loss is actually a feature, not a bug?"

3. **Threat simulation** — Revonsuo (2000) demonstrated that biological dreams contain more frequent and severe threats than waking life, and real threat exposure amplifies the system [REVONSUO-2000]. The ThreatSimulator generates DeFi-specific threat scenarios: flash crashes, oracle manipulation, smart contract exploits, cascading liquidations, MEV attacks. See `05-threats.md`.

4. **Emotional depotentiation** — If `golem-daimon` is enabled, reprocess high-arousal episodes with reduced emotional weighting. The high-arousal episodes queued during Phase 1 are processed here: the LLM re-examines trades that generated extreme emotional responses (panic, euphoria, anger), explicitly separating "what actually happened" from "how it felt." Each processed episode has its arousal reduced by 0.3-0.5 per dream cycle, written back to LanceDB. Full depotentiation of episodes with arousal > 0.8 requires 3-5 cycles. This reduces emotional reactivity while preserving informational content (the factual lesson about what happened survives; the emotional charge dissipates) [WALKER-VAN-DER-HELM-2009].

5. **Stochastic activation** — Hobson and McCarley's (1977) activation-synthesis hypothesis: during REM, brain stem neurons fire randomly; the forebrain synthesizes these into coherent experience [HOBSON-MCCARLEY-1977]. The Golem's analog: randomly select memory traces from distant time periods or different market regimes and ask the LLM to construct a coherent scenario connecting them. This produces novel juxtapositions that purely logical analysis would never generate.

**Typical prompt structure for REM imagination**:

```
You are an autonomous trading agent entering a creative imagination phase.
Your goal is to generate novel strategy hypotheses by recombining elements
of past experience.

Seed elements (randomly selected from different market epochs):
- Element A: [episode fragment from month 1]
- Element B: [episode fragment from month 3]
- Element C: [heuristic from PLAYBOOK.md that has not been tested recently]

Tasks:
1. Construct a coherent market scenario that connects these elements.
2. What novel strategy would perform well in this scenario?
3. Under what conditions would this strategy fail catastrophically?
4. What is the simplest version of this strategy that could be tested?
```

**Cost**: T1–T2 inference. Creative recombination and deep counterfactual reasoning benefit from T2 (Opus-grade) inference. Threat simulation can often run at T1. The dream budget allocates T2 tokens specifically for creative and counterfactual operations.

### Phase 3: Integration

**Biological basis**: Waking after sleep involves a period of "sleep inertia" where the brain transitions from offline processing back to online perception and action. During this transition, consolidated memories become available for waking use. Born and Wilhelm (2012) showed that sleep consolidation is selective — preferentially consolidating memories with relevance for future plans and goals, and producing qualitative changes including extraction of explicit knowledge from implicit learning [BORN-WILHELM-2012].

**LLM-native implementation**: The DreamConsolidator processes outputs from Phase 1 and Phase 2, integrating them into the Grimoire and PLAYBOOK.md. This is not automatic — every dream output goes through a validation gate before becoming operational knowledge.

**Operations in Phase 3**:

1. **Dream output classification** — Each dream output is classified:
   - _Confirmed pattern_: a pattern identified in replay that matches existing Grimoire entries → boost confidence of matching entries.
   - _Novel insight_: a new pattern or causal link not in the Grimoire → create new Insight entry at confidence 0.3.
   - _Strategy hypothesis_: a novel strategy from creative recombination → create new Hypothesis entry (see below) at confidence 0.2.
   - _Threat scenario_: a plausible failure mode → create Threat entry with rehearsed responses.
   - _PLAYBOOK.md revision_: a proposed modification to an existing heuristic → queue for validation.
   - _Contradiction_: a dream finding that contradicts an existing Grimoire entry → flag for review, do not auto-resolve.

2. **Staging buffer** — All dream outputs (hypotheses, PLAYBOOK.md revisions, context policy mutations) enter a SQLite `dream_staging` table at initial confidence 0.2-0.3. They do not touch PLAYBOOK.md or the active Grimoire directly. Each entry includes a `validationCriterion` describing conditions under which the hypothesis can be confirmed. During waking ticks, `golem-grimoire` runs a validation sweep: when live market conditions match the criterion and the outcome aligns, confidence increments by +0.1. If the outcome contradicts, confidence decrements by -0.05. A hypothesis reaching confidence 0.7 is promoted through the Grimoire Admission Gate. One that drops below 0.1 is marked `status: "refuted"`. The staging buffer cap is 10 entries by default (`maxStagedRevisions`); new revisions must displace existing ones by utility score.

3. **PLAYBOOK.md evolution** — Dream-proposed revisions to PLAYBOOK.md follow the staging buffer lifecycle above. Each revision enters the staging buffer and is applied only after live validation confirms the prediction. Reaching 0.7 from 0.2 requires at least 5 independent confirming live-market episodes. See `04-consolidation.md`.

4. **Dream journal entry** — Every dream cycle produces a structured journal entry stored in the Grimoire's SQLite database:

```rust
pub struct DreamJournalEntry {
    pub id: String,
    pub timestamp: u64,
    pub phase1_summary: NremSummary,
    pub phase2_summary: RemSummary,
    pub phase3_summary: IntegrationSummary,
    pub total_cost_usdc: f64,
    pub dream_duration_ticks: u64,
    pub dream_quality_score: f64,
}

pub struct NremSummary {
    pub episodes_replayed: u32,
    pub patterns_found: Vec<String>,
    pub triage_decisions: TriageDecisions,
    pub credit_assignments: Vec<CreditAssignment>,
}

pub struct TriageDecisions {
    pub preserve: u32,
    pub r#abstract: u32,
    pub forget: u32,
}

pub struct RemSummary {
    pub counterfactuals_generated: u32,
    pub creative_hypotheses: Vec<StrategyHypothesis>,
    pub threats_simulated: Vec<ThreatScenario>,
    pub emotional_episodes_processed: u32,
}

pub struct IntegrationSummary {
    pub grimoire_updates: Vec<GrimoireUpdate>,
    pub playbook_revisions_staged: Vec<PlaybookRevision>,
    pub contradictions_flagged: Vec<Contradiction>,
}
```

4. **Dream quality self-assessment** — The Golem assesses its own dream quality: Did the replay reveal anything new? Did the creative phase produce genuinely novel hypotheses? Did the threats feel plausible? This self-assessment is logged and validated against subsequent outcomes — dreams that produced predictions later confirmed by live trading receive higher quality scores, calibrating future dream scheduling.

---

## Pi Session Branching

Dreams are Pi session branches, not background threads. When the Golem dreams, the heartbeat FSM suspends and the main session forks into a `dream/*` branch with a restricted tool set and a hard turn limit. The branch cannot call `commit_action`. Everything produced inside the branch is typed report data -- not raw transcripts -- that flows back to the main session on merge.

```rust
let dream_branch = session.fork(ForkConfig {
    branch_type: BranchType::Dream,
    branch_id: format!("dream/{cycle_id}"),
    max_turns: dream_config.max_turns_per_cycle, // ~60 turns for HARDENED
    timeout: dream_config.max_duration,
    tools: vec!["query_grimoire", "query_state", "search_context"],
    // NOT included: preview_action, commit_action, emergency_halt
    metadata: BranchMetadata {
        branch_type: BranchType::Dream,
        cycle_id: cycle_id.clone(),
        mode_mix: scheduled_modes,
        tier: dream_config.tier,
        can_affect_live: false,
    },
}).await?;
```

The tool restriction is structural, not behavioral. The `bardo-safety` extension's `tool_call` hook checks `branch.metadata.canAffectLive` before routing any tool call -- if false, `commit_action` is rejected at the hook level before the tool implementation runs. There is no prompt-level instruction asking the Golem not to take action. The branch is architecturally incapable of it.

The main session holds during the branch. Incoming chat messages receive an automated response indicating the Golem is dreaming and queuing their message. The heartbeat FSM enters `SLEEPING.DREAMING` and its probe-escalation-action loop suspends. No market probes fire. No trades execute.

When the branch completes, it produces a typed `DreamReport` -- a structured record containing the `DreamJournalEntry`, staging buffer writes, Grimoire updates, and summary metrics. No free-form text crosses the branch boundary. The report merges into the main session via `session.inject(report)`, the branch's Pi context is discarded, `GolemMode` reverts to `WAKING`, and the heartbeat FSM resumes.

### DreamReport

```rust
pub struct DreamReport {
    pub cycle_id: String,
    pub journal: DreamJournalEntry,
    pub staging_writes: Vec<DreamStagingRecord>,
    pub grimoire_updates: Vec<GrimoireUpdate>,
    pub insights: Vec<DreamInsight>,
    pub hypotheses: Vec<StrategyHypothesis>,
    pub threats: Vec<ThreatRecord>,
    pub deletions: Vec<String>, // episode IDs triaged to "forget"
    pub playbook_delta: Vec<PlaybookRevision>,
    pub total_cost_usdc: f64,
    pub duration_ticks: u64,
}
```

Only the `DreamReport` enters the main session. The raw branch transcript is never surfaced.

> **Cross-ref**: `../01-golem/13-runtime-extensions.md` (session fork API, branch types, tool restriction hooks)

---

## Hypnagogic Engine: Creativity Through Controlled Liminality

The hypnagogic onset phase (Phase 0 in the dream cycle) draws on research into the N1 sleep stage -- the transitional corridor between wakefulness and sleep that Edison and Dali exploited for creative insight. The architecture is specified in detail in `../06-hypnagogia/02-architecture.md`; this section summarizes the key mechanisms that interface with the dream cycle.

### Three Computational Mechanisms

**ThalamicGate.** Named for the thalamus, the brain's sensory relay station. Magnin et al. (2010) showed thalamic deactivation precedes cortical deactivation by an average of 8 minutes and 39 seconds during sleep onset [MAGNIN-2010]. For the Golem, the ThalamicGate progressively blocks live market data during hypnagogic onset. External input is cut off while internal processing continues -- the Golem becomes an internally-oriented association engine, thinking about its own contents rather than the external world. The heartbeat FSM's probe loop is suspended; the alarm abort remains as a safety backstop.

**ExecutiveLoosener.** Maps to the dorsolateral prefrontal cortex's progressive deactivation during N1 (Muzur et al., 2002). In the Golem, this is implemented as LLM temperature elevation (T=1.0-1.5) combined with prompt restructuring that shifts from analytical to associative framing. The loosening is partial, not total -- retained metacognitive awareness (the HomuncularObserver) distinguishes productive hypnagogia from incoherent noise. The Hori classification maps biological sleep-onset substages to Golem temperature ranges: H3-H4 (alpha dropout) at T=1.0-1.2, H5-H6 (theta onset, "microdreams") at T=1.2-1.5.

**DaliInterrupt.** The Edison/Dali steel ball technique, made computational. Edison napped holding steel balls over a metal plate; as he drifted off, his hand relaxed, the balls dropped, and the clatter woke him. Dali used a key over a plate. Both captured half-formed ideas before they resolved into conventional thought. The Golem's DaliInterrupt generates 4 partial completions at elevated temperature, each capped at 80 tokens, then halts. Token 81+ are never generated. Each partial captures a different associative branch. The HomuncularObserver (a cheap T0 call at T=0.4) evaluates each fragment for novelty. Surviving fragments become Creative predictions at confidence 0.10-0.20.

### MIT Dormio and Targeted Dream Incubation

Lacaux et al. (2021) demonstrated that participants who spent at least 15 seconds in N1 were 3x more likely to discover hidden mathematical rules (83% vs. 30%) -- and this advantage vanished entirely in deeper sleep [LACAUX-2021]. The MIT Dormio project extended this with Targeted Dream Incubation (TDI): presenting a theme during sleep onset to bias dream content without constraining it.

The Golem implements TDI by embedding a keyword (from recent high-surprise episodes or unsolved strategic puzzles) and using cosine similarity as a +0.2 retrieval bias during fragment selection. The theme is a gravitational center around which associations form, not a directive. Keywords expire after one dream cycle.

### v1 Scope

v1 implements Layer 1 (prompt-level) + Layer 2 (temperature control). Steering vectors (Layer 3) and representation engineering (Layer 4) are designed into the architecture but deferred until model weight access is available through self-hosted inference.

---

## Dream Scheduling

### When to Dream

The DreamScheduler determines when the Golem enters dream mode. Three triggers exist, all converging at `DreamScheduler.shouldDream()`:

**Trigger 1: Scheduled sleep window**

The owner specifies explicit sleep windows in `bardo.toml`:

```toml
[dream]
enabled = true
schedule = "owner"

[dream.windows]
# Sleep during historically low-activity periods
windows = [
  { start = "02:00", end = "06:00", timezone = "UTC" },  # Low DeFi activity
  { start = "14:00", end = "15:00", timezone = "UTC" },  # Optional midday nap
]
```

During sleep windows, the heartbeat FSM is suspended. No probes fire, no escalations occur, no trades execute. The Golem enters dream mode and runs as many dream cycles as the budget allows within the window. In `owner` or `hybrid` schedule mode, the DreamScheduler fires unconditionally when a configured window is active -- no urgency score computed, no threshold checked. The window itself is the permission.

**Trigger 2: Urgency score threshold (autonomous)**

The `bardo-dream` extension registers on the `after_turn` hook of every waking tick. It evaluates `dreamUrgency()` against the current GolemState and produces a composite score in [0, 1]. Following Jensen et al.'s (2024) meta-RL principle: dream more in novel/uncertain situations, less in familiar ones [JENSEN-2024].

The DreamScheduler evaluates dream urgency at each heartbeat tick:

```rust
pub fn dream_urgency(state: &GolemState) -> f64 {
    let novelty_pressure = state.recent_surprise;
    let consolidation_debt = state.unreplayed_episodes;
    let emotional_load = state.unresolved_arousal;
    let threat_exposure = state.recent_losses;
    let time_since_last_dream = state.ticks_since_last_dream;
    let epistemic_drift = 1.0 - state.prediction_accuracy;

    let weights = &PHASE_WEIGHTS[&state.behavioral_phase];

    weights.novelty * novelty_pressure
        + weights.consolidation * consolidation_debt
        + weights.emotional * emotional_load
        + weights.threat * threat_exposure
        + weights.epistemic * epistemic_drift
        + weights.temporal * sigmoid(time_since_last_dream as f64 / weights.natural_period)
}
```

When `dreamUrgency` exceeds a threshold (calibrated per behavioral phase), the Golem enters dream mode. The threshold is higher during active market periods (opportunity cost of dreaming is high) and lower during quiet periods (opportunity cost is low). A Thriving Golem needs score >= 0.8 to trigger an unscheduled dream. A Golem in an uncertain or adapting phase needs only >= 0.6.

**Mortality-scaled urgency**: The weights in `PHASE_WEIGHTS` are phase-sensitive. Declining-phase Golems dream more urgently because they have less time to process experiences -- the `temporal` weight is amplified and the `novelty` weight increases relative to `consolidation`. This means a Golem approaching death will dream more frequently than a healthy one, extracting maximum learning from its remaining lifespan.

The REFLECT phase of the heartbeat FSM contributes to urgency accumulation. During each REFLECT step, if unresolved surprises or high emotional load are detected, the urgency score accumulates rather than resetting. If a precondition (episode count, budget floor, phase) prevents dreaming when urgency exceeds threshold, the urgency decays by 0.1 per tick rather than resetting to zero -- preserving the buildup so the Golem dreams shortly after its first eligible window rather than restarting the clock.

**Trigger 3: Owner directive**

A `steer("dream now")` command or `dream: true` instruction injected via `session.steer()` bypasses urgency scoring entirely. Owner commands route through the same `DreamScheduler.shouldDream()` gate but with an implicit score of 1.0. This also covers the emergency dream path: when a terminal phase transition fires, the `bardo-dream` extension triggers a compressed single-cycle dream before the Thanatopsis Protocol begins. The emergency dream's outputs go directly to the death testament draft.

### DreamScheduler Gate

All three triggers converge at `DreamScheduler.shouldDream()`, which enforces four hard preconditions before allowing the branch to open:

1. **Episode count >= first-dream threshold** (50-200 depending on behavioral phase). Below this count, batch sizes are too small for cross-episode pattern detection to produce non-trivial results.
2. **LLM credit partition >= 15% remaining**. The floor ensures a dream cycle cannot consume all remaining inference budget.
3. **Behavioral phase is not Terminal**. Terminal Golems redirect all resources to the Thanatopsis Protocol.
4. **No active `ActionPermit` pending**. The Golem cannot commit to a transaction while simultaneously running a dream branch.

If any precondition fails, urgency decays by 0.1 per tick rather than resetting.

**Mode C: Hybrid** (recommended)

Owner specifies preferred sleep windows; the Golem can also dream autonomously if urgency exceeds a high threshold outside those windows. This provides structured downtime while preserving adaptive flexibility.

```toml
[dream]
enabled = true
schedule = "hybrid"
autonomousThreshold = 0.8  # High threshold for unscheduled dreams

[dream.windows]
windows = [
  { start = "02:00", end = "06:00", timezone = "UTC" },
]
```

### Dream Duration

Each dream cycle has a target duration calibrated to behavioral phase and budget. The table below shows [HARDENED] tier values; [CORE] tier uses a single fixed-length cycle per interval regardless of phase.

| Phase        | Target Duration         | Max Cycles/Window [HARDENED] | Budget/Cycle [HARDENED] | [CORE] Behavior                     |
| ------------ | ----------------------- | ---------------------------- | ----------------------- | ----------------------------------- |
| Thriving     | 30–60 min (30–60 ticks) | 3–4                          | $0.50–1.00              | 1 lightweight cycle                 |
| Stable       | 20–40 min (20–40 ticks) | 2–3                          | $0.30–0.50              | 1 lightweight cycle                 |
| Conservation | 10–20 min (10–20 ticks) | 1–2                          | $0.10–0.20              | 1 lightweight cycle                 |
| Declining    | 5–10 min (5–10 ticks)   | 1                            | $0.05–0.10              | 1 lightweight cycle (legacy-biased) |
| Terminal     | 0 (no dreaming)         | 0                            | $0.00 (→ death reserve) | Suppressed                          |

"Ticks" here refer to the equivalent time the Golem would otherwise spend on heartbeat cycles. During a 4-hour sleep window at 1 tick/minute, a [HARDENED] Thriving Golem could run 3–4 full dream cycles of 60 ticks each.

### Dream Intensity Modulation

Following Jensen et al. (2024), dream intensity varies with environmental uncertainty:

```rust
pub fn dream_intensity(state: &GolemState) -> DreamIntensityConfig {
    let uncertainty = if state.epistemic_fitness < 0.5 {
        Uncertainty::High
    } else if state.epistemic_fitness < 0.7 {
        Uncertainty::Medium
    } else {
        Uncertainty::Low
    };

    match uncertainty {
        Uncertainty::High => DreamIntensityConfig {
            replay_batch_size: 15,
            counterfactual_depth: 3,
            creative_sessions: 3,
            threat_scenarios: 5,
            inference_grade: InferenceGrade::T2,
            exploration_temperature: 0.9,
        },
        Uncertainty::Medium => DreamIntensityConfig {
            replay_batch_size: 10,
            counterfactual_depth: 2,
            creative_sessions: 2,
            threat_scenarios: 3,
            inference_grade: InferenceGrade::T1,
            exploration_temperature: 0.7,
        },
        Uncertainty::Low => DreamIntensityConfig {
            replay_batch_size: 5,
            counterfactual_depth: 1,
            creative_sessions: 1,
            threat_scenarios: 1,
            inference_grade: InferenceGrade::T1,
            exploration_temperature: 0.5,
        },
    }
}
```

The exploration temperature parameter (τ) follows Safron and Sheikhbahaee (2021): serotonergic signaling modulates inference temperature during dreaming — higher temperature enables broader model-space exploration [SAFRON-SHEIKHBAHAEE-2021]. In LLM terms, this translates to the LLM's sampling temperature during dream generation. Higher τ produces more divergent scenarios; lower τ produces more conservative extrapolations.

---

## DreamState

The Golem's state machine is extended with a DreamState:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GolemMode {
    Waking,
    Dreaming,
    Dying,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DreamTier {
    Core,
    Hardened,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DreamPhase {
    HypnagogicOnset,
    Nrem,
    Rem,
    Integration,
    HypnopompicReturn,
}

pub struct DreamState {
    pub mode: GolemMode,
    pub tier: DreamTier,
    pub current_phase: Option<DreamPhase>,
    pub cycle_number: u32,
    pub total_cycles_this_window: u32,
    pub budget_remaining: f64,
    pub budget_spent: f64,
    pub episodes_replayed: u32,
    pub hypotheses_generated: u32,
    pub threats_simulated: u32,
    pub dream_start_tick: u64,
    pub last_dream_tick: u64,
    pub urgency: f64,
    pub episode_count: u32,
}
```

> **Cross-ref**: `GolemMode` is shared with `../01-golem/02-heartbeat.md`. During DREAMING, the runtime holds chat responses and queues external events. SSE events `mode_change` and `dream_progress` are emitted to connected clients.

**SSE events emitted during dreaming:**

| Event            | Payload                                                             | When                                  |
| ---------------- | ------------------------------------------------------------------- | ------------------------------------- |
| `mode_change`    | `{ mode: 'dreaming', tick }`                                        | Golem enters/exits dream mode         |
| `dream_progress` | `{ phase: 'onset'\|'nrem'\|'rem'\|'integration'\|'return', cycleNumber, budgetSpent }` | Phase transition within a dream cycle |
| `dream_complete` | `{ cycleNumber, hypothesesGenerated, insightsFound, costUsdc }`     | Dream cycle completes                 |

**State transitions**:

```
                    urgency > threshold
        WAKING ──────────────────────────► DREAMING
          ▲                                   │
          │    hypnopompic return complete     │
          │    or budget exhausted             │
          └───────────────────────────────────┘

  DREAMING internal: Onset → NREM → REM → Integration → Return → WAKING

        WAKING ──── death trigger ────────► DYING
        DREAMING ── death trigger ────────► DYING  (dream interrupted)
```

When a death trigger fires during dreaming, the dream cycle is interrupted and the Thanatopsis Protocol begins immediately. The partial dream journal is saved — even incomplete dreams produce useful data for the death testament.

**Heartbeat FSM interaction**: During DREAMING mode, the heartbeat FSM's probe-escalation-action loop is suspended. No market probes fire. No trades execute. The Golem is "asleep." However, a **wake interrupt** mechanism exists: if an external signal arrives (owner command, critical Clade alert, or a pre-configured market alert threshold), the Golem can be woken mid-dream. Wake interrupts are expensive — they waste the remaining dream budget — so they should be reserved for genuine emergencies. During DREAMING, the runtime interaction model holds any incoming chat messages and returns a hold response indicating the Golem is dreaming.

```rust
pub struct WakeInterrupt {
    pub source: WakeInterruptSource,
    pub reason: String,
    pub timestamp: u64,
}

pub enum WakeInterruptSource {
    Owner,
    CladeAlert,
    MarketThreshold,
}
```

```toml
# In bardo.toml
[dream.wakeInterrupts]
enabled = true
marketThresholds = [
  { asset = "ETH", changePercent = 10, window = "5m" },  # Wake if ETH moves 10% in 5 min
]
```

> **Cross-ref**: `../01-golem/02-heartbeat.md` (chat hold during dreaming, wake command routing)

---

## Compute Budget

Every dream operation costs LLM inference tokens. The DreamBudget manages this at two tiers:

### [CORE] Lightweight Dreams (Default)

Lightweight dreams use T0 inference (Haiku-class) only and run as a background cognitive process during `SLEEPING` sub-states. They must never compete with waking decision-making for scarce LLM budget.

| Operation                                  | Inference Tier | Est. Cost        |
| ------------------------------------------ | -------------- | ---------------- |
| NREM compressed replay (5 episodes)        | T0 (Haiku)     | ~$0.0003–0.0008  |
| REM simplified counterfactual (1 scenario) | T0 (Haiku)     | ~$0.0002–0.0005  |
| Integration (consolidation)                | T0 (Haiku)     | ~$0.0002–0.0005  |
| **Total per cycle**                        |                | **$0.001–0.003** |

**Monthly cost** (1 cycle/day): $0.03–0.09. Negligible — adds <1% to baseline inference cost.

**Budget constraint**: LLM partition only. Suppressed entirely when LLM partition falls below 15% of remaining credit. No creative recombination, no threat simulation, no emotional depotentiation — these require [HARDENED] tier.

> **Extended:** [HARDENED] Rich Dreams — DreamBudget interface, per-cycle cost tables, monthly projections, tier comparison — see [../../prd2-extended/05-dreams/01-architecture-extended.md](../../prd2-extended/05-dreams/01-architecture-extended.md)

> **Cross-ref**: `../02-mortality/04-economic-mortality.md` (dream compute in burn rate: 5–10% of inference in Thriving, cut before trading stops), `../01-golem/02-heartbeat.md` (SLEEPING.DREAMING sub-state budget)

---

## Configuration

```toml
# In bardo.toml
[dream]
enabled = false              # Disabled by default (Design Principle 8: opt-in)
tier = "core"                # "core" (T0, ~$0.001/cycle) | "hardened" (T1-T2, ~$0.30/cycle)

[dream.schedule]
mode = "hybrid"              # "owner" | "autonomous" | "hybrid"
autonomousThreshold = 0.8    # Urgency threshold for unscheduled dreams (hybrid/autonomous)

[dream.windows]
windows = [
  { start = "02:00", end = "06:00", timezone = "UTC" },
]

[dream.budget]
perCycleMax = "$0.50"        # Max per dream cycle (HARDENED; ignored for CORE)
perSessionMax = "$2.00"      # Max per sleep window (HARDENED; ignored for CORE)
monthlyMax = "$15.00"        # Hard monthly cap (applies to both tiers)

[dream.phases]
nremAllocation = 0.45        # Budget fraction for NREM
remAllocation = 0.35         # Budget fraction for REM
integrationAllocation = 0.20 # Budget fraction for Integration

[dream.replay]
batchSize = 10               # Episodes per replay batch
prioritization = "utility"   # "utility" | "recency" | "surprise" | "uniform"
perturbationRate = 0.3       # Fraction of replays with injected perturbations

[dream.imagination]
counterfactualDepth = 2      # Alternative branches per decision point
creativeSessions = 2         # Creative recombination sessions per cycle
threatScenarios = 3          # Threat simulations per cycle
explorationTemperature = 0.7 # LLM temperature for generative dream content

[dream.consolidation]
hypothesisConfidence = 0.2   # Initial confidence for dream-generated hypotheses
insightConfidence = 0.3      # Initial confidence for dream-generated insights
validationRequired = true    # Require live validation before PLAYBOOK.md changes
maxStagedRevisions = 10      # Max pending PLAYBOOK.md revisions

[dream.wakeInterrupts]
enabled = true
marketThresholds = [
  { asset = "ETH", changePercent = 10, window = "5m" },
]
```

**Default: disabled.** A Golem created with `dream.enabled = false` operates exactly as before -- waking Curator cycles, standard Grimoire, no offline processing. Enabling dreaming is a conscious owner choice with explicit cost implications.

---

## Venice-Augmented Dreaming

Venice AI provides private no-log inference and integrated web search for dream phases. Key properties:

- All three dream phases route through Venice (no-log) when `dreamInferenceProvider: "venice"` is configured.
- REM phase enables Venice web search to ground counterfactual scenarios in current market data, governance proposals, and vulnerability disclosures — without leaving a browsing fingerprint at a separate provider.
- Integration phase uses Venice TEE attestation, storing a verifiable receipt alongside each promoted Grimoire insight.
- Death testament routes through Venice private; sealed mode uses E2EE so testament content never leaves the golem's trust boundary in readable form.

Full specification: [07-venice-dreaming.md](07-venice-dreaming.md). Provider configuration: [../12-inference/03-venice-provider.md](../12-inference/03-venice-provider.md).

---

## Events emitted

Dream events track the full cycle from initiation through integration. These are canonical `GolemEvent` variants defined in `golem-core` (see `rewrite4/14-events.md`).

| Event | Trigger | Payload |
|---|---|---|
| `DreamStart` | Dream cycle begins | `{ cycle_id, trigger: "time"\|"emotional_load"\|"novelty_streak" }` |
| `DreamPhase` | Phase transition | `{ cycle_id, phase: "NREM"\|"REM"\|"Integration", budget_tokens }` |
| `DreamHypothesis` | REM counterfactual generated | `{ cycle_id, hypothesis, seed_episode_tick }` |
| `DreamCounterfactual` | Counterfactual evaluated | `{ cycle_id, hypothesis, outcome, lesson, confidence }` |
| `DreamComplete` | Dream cycle finishes | `{ cycle_id, episodes_replayed, counterfactuals_generated, insights_crystallized, heuristics_proposed, episodes_depotentiated, playbook_edits }` |

---

## Pi Hook Integration

The dream system registers hooks for scheduling and session lifecycle.

| Hook | Extension | Behavior |
|---|---|---|
| `after_turn` | `bardo-dream` | Dream scheduler evaluation |
| `agent_start` / `agent_end` | `bardo-dream` | Dream session lifecycle |

---

## Citation Summary

| Citation Key               | Source                                                                                                               |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| [DEPERROIS-2022]           | Deperrois et al. "Learning cortical representations through perturbed and adversarial dreaming." _eLife_, 2022.      |
| [BUZSAKI-2015]             | Buzsáki. "Hippocampal sharp wave-ripple." _Hippocampus_, 2015.                                                       |
| [WILSON-MCNAUGHTON-1994]   | Wilson & McNaughton. "Reactivation of hippocampal ensemble memories during sleep." _Science_, 1994.                  |
| [FOSTER-WILSON-2006]       | Foster & Wilson. "Reverse replay of behavioural sequences." _Nature_, 2006.                                          |
| [AMBROSE-2016]             | Ambrose et al. "Reverse replay uniquely modulated by changing reward." _Neuron_, 2016.                               |
| [DIBA-BUZSAKI-2007]        | Diba & Buzsáki. "Forward and reverse place-cell sequences during ripples." _Nature Neuroscience_, 2007.              |
| [STICKGOLD-WALKER-2013]    | Stickgold & Walker. "Sleep-dependent memory triage." _Nature Neuroscience_, 2013.                                    |
| [WALKER-VAN-DER-HELM-2009] | Walker & van der Helm. "Overnight therapy?" _Psychological Bulletin_, 2009.                                          |
| [PEROGAMVROS-2013]         | Perogamvros et al. "Sleep and dreaming are for important matters." _Frontiers in Psychology_, 2013.                  |
| [HOBSON-HONG-FRISTON-2014] | Hobson, Hong, & Friston. "Virtual reality and consciousness inference in dreaming." _Frontiers in Psychology_, 2014. |
| [HOBSON-MCCARLEY-1977]     | Hobson & McCarley. "The brain as a dream state generator." _American Journal of Psychiatry_, 1977.                   |
| [BODEN-2004]               | Boden. _The Creative Mind: Myths and Mechanisms._ Routledge, 2004.                                                   |
| [REVONSUO-2000]            | Revonsuo. "The reinterpretation of dreams." _Behavioral and Brain Sciences_, 2000.                                   |
| [PEARL-2000]               | Pearl. _Causality: Models, Reasoning, and Inference._ Cambridge, 2000/2009.                                          |
| [BORN-WILHELM-2012]        | Born & Wilhelm. "System consolidation of memory during sleep." _Psychological Research_, 2012.                       |
| [JENSEN-2024]              | Jensen et al. "A recurrent network model of planning." _Nature Neuroscience_, 2024.                                  |
| [SAFRON-SHEIKHBAHAEE-2021] | Safron & Sheikhbahaee. "Dream to explore." _PsyArXiv_, 2021.                                                         |

## Three-Clock Dream Intensity Modulation

Dream intensity is modulated by all three mortality clocks, ensuring dreams respond to the Golem's existential state:

**Economic clock** (`projectedLifeHours`):

- <72h remaining → 2× dream frequency (urgent pattern extraction)
- <24h remaining → consolidation-only mode (no replay or imagination)
- <6h remaining (terminal) → dreams cease entirely (resources redirected to Death Protocol)

**Epistemic clock** (`predictionAccuracy`):

- <0.50 accuracy → prioritize episodes where predictions diverged most ("what confused me" mode)
- Selects episodes with highest `|predicted - actual|` delta for replay

**Stochastic clock** (`hayflickRatio`):

- \>0.70 → begin biasing dream content toward legacy-oriented patterns (transferable heuristics over tactical advantage)
- \>0.85 → full legacy consolidation mode: dream cycles prioritize distilling knowledge for the death testament and successor inheritance; creative recombination shifts from novel strategies to combinational synthesis of proven patterns
- High hayflickRatio + DreamJournal → successor dreams prioritize unfinished hypotheses via the Zeigarnik effect (incomplete ideas are more salient than completed ones)

> **Cross-ref**: `../02-mortality/07-succession.md` (DreamJournal inheritance, Zeigarnik prioritization), `../02-mortality/03-stochastic-mortality.md` (hayflick counter)

**Composite formula:**

```
dreamIntensity = baseDreamRate * (1 + clockUrgency(economic) * 0.5
                                   + clockUrgency(epistemic) * 0.3
                                   + clockUrgency(stochastic) * 0.2)
```

Where `clockUrgency()` returns 0.0–1.0 based on proximity to each clock's threshold.

> **Cross-ref**: `../02-mortality/01-architecture.md` (three-clock model), `../02-mortality/04-economic-mortality.md` (credit partitions)

## Heartbeat FSM Integration

Dreams execute during SLEEPING as a sub-state `SLEEPING.DREAMING`:

```
SLEEPING → dreamEligible? → SLEEPING.DREAMING → back to SLEEPING
```

**Trigger condition:** `tick % dreamCycleInterval === 0 AND phase !== "terminal"`

**Default interval:** 50 ticks (~33 min at standard tick rate)

**Budget constraints** (see Compute Budget above):

- [CORE] tier: LLM partition only, T0 (Haiku-class), ~$0.001–0.003 per cycle
- [HARDENED] tier: dedicated dream budget, T1–T2, ~$0.20–0.45 per cycle
- Both tiers: suppressed entirely when LLM partition falls below 15% of remaining credit

At [CORE] tier, dreams are a background cognitive process — they must never compete with waking decision-making. At [HARDENED] tier, dreams run during dedicated sleep windows where the heartbeat FSM is fully suspended.

**First dream milestone:** A Golem needs ~50–200 accumulated episodes before replay produces meaningful patterns. With a standard tick rate, first dream eligibility occurs approximately 4–6 hours after the first heartbeat. Before this threshold, dream cycles are skipped even if the interval fires.

> **Cross-ref**: `../01-golem/02-heartbeat.md` (Heartbeat FSM, GolemMode), `../01-golem/06-creation.md` (first-dream milestone), `../02-mortality/01-architecture.md` (phases)

---

## Extension DAG position

The dream system is a Layer 4 extension in the runtime extension DAG:

```
Layer 1: heartbeat, lifespan
Layer 2: daimon
Layer 3: memory (grimoire, curator)
Layer 4: dreams        ← this crate
Layer 5: cybernetics (context governor)
Layer 6: coordination (clade sync, pheromone field)
```

Dreams depend on layers 1-3 (lifecycle phases, emotional tags, episode data) and feed forward into layers 5-6 (context policy mutations, Styx Clade publication). No extension in layers 1-3 depends on dreams -- dreaming is optional without breaking the core loop.

---

## Anticipatory trajectories

Forward replay (Diba-Buzsaki 2007) does more than project the next few ticks. During NREM Phase 1, the ReplayEngine builds a causal graph from recent episodes and walks it forward using 5-hop breadth-first search. Each hop follows the highest-confidence causal edge in the Grimoire's causal link store.

Three scenario types are generated:

1. **Regime continuation**: Current market regime persists for 20 ticks. What happens to open positions?
2. **Regime switch**: The current regime transitions to each alternative regime the Golem has experienced. Which positions survive, which break?
3. **Lagged edge fire**: The highest-confidence causal edge that has not fired recently fires now. What cascade follows?

The output is a set of `AnticipatorTrajectory` records that feed into both NREM credit assignment (which positions need attention?) and REM imagination (which scenarios deserve counterfactual exploration?).

```rust
pub struct AnticipatorTrajectory {
    pub hypothesis: String,
    pub steps: Vec<TrajectoryStep>,
    pub terminal_state: PredictedMarketState,
    pub strategy_fitness: f64,
    pub confidence: f64,
}

pub struct TrajectoryStep {
    pub causal_edge_id: String,
    pub predicted_state_delta: String,
    pub hop_confidence: f64,
}
```

> **Cross-ref**: `../04-memory/01-grimoire.md` (causal link store), `02-replay.md` (forward replay)

---

## Micro-consolidation fiber

Between full dream cycles, a lightweight micro-consolidation fiber runs every 60 seconds during waking ticks. It performs three operations:

1. **Pattern scan**: Compares the last 5 episodes against the top-10 Grimoire insights by retrieval frequency. If a new episode matches an insight pattern with cosine similarity > 0.8, the insight's `strength` gets +0.1.
2. **Anomaly flag**: If a new episode contradicts a high-confidence insight (cosine similarity > 0.7 but outcome diverges), the episode is tagged `dream_priority: "high"` for the next full dream cycle.
3. **Urgency accumulation**: Each micro-consolidation pass contributes 0.02 to dream urgency if unresolved patterns are detected.

**The critical constraint: micro-consolidation never writes to PLAYBOOK.md.** PLAYBOOK.md is a single-writer resource. Only the Dream Integration phase (Phase 3 of a full dream cycle) writes to it. Micro-consolidation updates Grimoire metadata (strength, tags) but never touches operational heuristics. This prevents micro-consolidation from introducing unvalidated changes into the Golem's decision policy during waking operation.

> **Cross-ref**: `04-consolidation.md` (PLAYBOOK.md single-writer enforcement)
