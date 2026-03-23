# Dream Integration: Cross-Track Interactions [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `00-overview.md`, all Track 1–3 documents

---

> **Reader orientation:** This document specifies how the Dreams track interacts with the other four tracks of Bardo (the Rust runtime for mortal autonomous DeFi agents): Mortality, Daimon (affect engine), Memory, and Hypnagogia (liminal onset/return phases). It covers pairwise interaction matrices, the quadruple interaction lifecycle (Mortality x Daimon x Memory x Dreams), the Extension trait implementation, and how each track's outputs feed into and modulate dreaming. Prerequisites: the Dreams overview (`00-overview.md`) and familiarity with all five tracks. For a full glossary, see `prd2/shared/glossary.md`.

## The Five-Track System

With hypnagogia as Track 5, the Bardo system now comprises five interconnected tracks. Each is independently motivated by distinct research traditions. Each is individually valuable. But their power is multiplicative -- the interactions between tracks produce dynamics that no single track could achieve alone.

| Track              | Core Question                                       | Research Domains                                                                        | Package                                            |
| ------------------ | --------------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------- |
| **1. Mortality**   | When and why should an agent die?                   | Evolutionary computation, game theory, information science, mortal computation          | `golem-core`                                             |
| **2. Daimon**      | How should feelings shape decisions?                | Cognitive neuroscience, RL curiosity, LLM emotion research, narrative psychology        | `golem-daimon`                                           |
| **3. Memory**      | What should be remembered, forgotten, bequeathed?   | Neuroscience of forgetting, evolutionary biology, knowledge economics                   | `golem-grimoire`, `bardo-styx-client`                    |
| **4. Dreaming**    | How does the agent get smarter between actions?     | Sleep neuroscience, world models/RL, counterfactual reasoning, computational creativity | `golem-dreams`                                           |
| **5. Hypnagogia**  | How should the agent transition in and out of sleep? | Hypnagogic/hypnopompic neuroscience, liminal cognition, Dali's slumber-with-a-key      | `golem-hypnagogia`                                       |

---

## Pairwise Interactions

### Dreaming × Mortality

**Mortality makes dreaming essential.** An immortal agent can afford to learn slowly through direct experience — time is free. A mortal agent cannot. Every real trade costs gas, risks capital, and consumes finite lifespan. Sutton's (1991) Dyna architecture showed that each real experience can be supplemented by many "planning steps" of simulated experience, multiplying learning signal [SUTTON-1991]. For a mortal Golem, this multiplication converts a finite lifespan of N real trades into N × R learning episodes (where R is the replay ratio), dramatically increasing the return on each unit of lived experience.

**Dreaming intensity tracks mortality pressure.** As the Golem ages and its mortality clocks advance, dream behavior changes:

| Behavioral Phase | Vitality | Dream Character                                                         | Budget Priority             |
| ---------------- | -------- | ----------------------------------------------------------------------- | --------------------------- |
| **Thriving**     | 0.7–1.0  | Exploratory. Creative recombination dominates. Novel hypotheses sought. | Maximize exploration        |
| **Stable**       | 0.5–0.7  | Consolidating. Replay dominates. PLAYBOOK.md stabilizes.                | Balanced                    |
| **Conservation** | 0.3–0.5  | Defensive. Threat simulation intensifies. Pruning and compression.      | Maximize robustness         |
| **Declining**    | 0.15–0.3 | Legacy-oriented. Deep replay for death testament enrichment.            | Maximize knowledge transfer |
| **Terminal**     | <0.15    | No dreaming. All resources to Thanatopsis Protocol.                     | $0 (all to death reserve)   |

**Dreaming rehearses death.** Revonsuo's threat simulation theory (2000) shows biological dreams over-represent threats [REVONSUO-2000]. A mortal Golem dreams about capital depletion, epistemic decay, and cascading failures — its own death scenarios. This rehearsal produces anticipatory responses: pre-positioned exits, early warning triggers, defensive PLAYBOOK.md guards. A non-dreaming Golem can only develop these through direct (and possibly fatal) experience.

**Dreaming extends effective lifespan.** If dreaming produces better strategies, faster adaptation, and more robust risk management, then a dreaming Golem extracts more value from each tick of lifespan. The lifespan in ticks may be identical to a non-dreaming Golem, but the _effective intelligence-ticks_ are higher. This is the mortal computation argument applied to dreaming: the quality of computation within a finite lifespan is what matters, not the quantity of ticks.

### Dreaming × Daimon

**Daimon guides dream content.** Emotional tags on episodes create a salience gradient for dream replay. McGaugh (2004) showed that emotional memories receive preferential consolidation through amygdala-hippocampal interactions [MCGAUGH-2004]. During NREM-like replay, high-arousal episodes (whether positive or negative) are replayed preferentially. During REM-like imagination, mood state biases scenario generation — an anxious Golem (low dominance, negative pleasure) generates more threat scenarios; a confident Golem (high dominance, positive pleasure) generates more exploratory scenarios.

**Dreaming processes daimon.** Walker and van der Helm's (2009) "Sleep to Forget, Sleep to Remember" model: REM sleep depotentiates emotional tone while preserving informational content [WALKER-VAN-DER-HELM-2009]. Without this processing, emotional tags accumulate monotonically — the Golem becomes increasingly reactive, eventually trapped in mood-congruent retrieval loops (always anxious because anxiety retrieves anxious memories which generate more anxiety). Dreaming breaks these loops through controlled reprocessing.

**The creative-emotion tension.** Franceschelli and Musolesi (2024) showed that RLHF reduces LLM output diversity [FRANCESCHELLI-MUSOLESI-2024]. Similarly, a Golem experiencing sustained negative affect will generate less creative dream content — its "exploration temperature" drops. The dream architecture counters this by maintaining a minimum creative allocation (20%) even during negative mood states. Some creative generation must persist regardless of emotional state, or the Golem loses its ability to discover novel escape strategies precisely when it needs them most.

**Dream-mediated resolution of mortality emotions.** Per `../03-daimon/04-mortality-daimon.md`, dreaming provides the primary resolution mechanism for three mortality-specific emotional states:

- **Economic anxiety** (fear of capital depletion): REM depotentiation reduces the arousal dimension while preserving the informational content ("low balance is dangerous"). Over multiple dream cycles, the Golem maintains rational caution without panic.
- **Epistemic vertigo** (confusion about what it knows): Counterfactual dreaming processes episodes where predictions diverged most, restoring dominance (sense of understanding) through structured re-analysis.
- **Stochastic dread** (awareness of random death): Threat simulation rehearses worst-case scenarios, converting abstract dread into concrete preparedness with rehearsed responses. Revonsuo's (2000) threat simulation theory predicts that rehearsal reduces emotional charge while maintaining behavioral vigilance.

Without dreaming, these mortality emotions accumulate unresolved, progressively impairing decision quality through mood-congruent retrieval cascades.

**Dream outcome events close the emotional loop.** When a staged dream hypothesis is validated or refuted by live trading, the DreamConsolidator emits a `dream_outcome` appraisal event to the Daimon engine (see `04-consolidation.md`). Validated hypotheses generate mild joy; refuted hypotheses generate moderate surprise. This integrates dreaming into the Daimon's appraisal cycle — the Golem develops emotional relationships with its own hypotheses.

> **Cross-ref**: `../03-daimon/04-mortality-daimon.md` (economic anxiety, epistemic vertigo, stochastic dread), `../03-daimon/01-appraisal.md` (dream_outcome trigger)

### Dreaming × Memory

**Memory provides dream material; dreaming produces memory updates.** The Grimoire's episodes, insights, and PLAYBOOK.md heuristics are the raw inputs to dream processing. Dream outputs — novel insights, validated hypotheses, threat records, PLAYBOOK.md revisions — flow back into the Grimoire. This bidirectional flow creates a feedback loop: better memory produces better dreams which produce better memory.

**The Curator and the Dreamer are complementary.** The Curator (every 50 ticks during waking) performs incremental distillation under time pressure — it must not delay trading. Dreaming performs deep reorganization without time pressure. The Curator is the Golem's daily housekeeping; dreaming is the annual spring cleaning. Together they implement the full complementary learning systems architecture from McClelland et al. (1995): fast episodic encoding (Grimoire episodes) → slow extraction of statistical regularities (PLAYBOOK.md) via interleaved replay (Curator + Dream) [MCCLELLAND-1995].

**Dream outputs are memory with special provenance.** Dream-generated insights enter the Grimoire at lower confidence (0.2–0.3) than waking-generated insights (0.5+). This confidence differential implements a variant of the Weismann barrier — dream knowledge is treated as hypothesis, not established fact, until validated through waking experience. The IKEA Effect [NORTON-MOCHON-ARIELY-2012] ensures the Golem values its own waking validations above its dream speculations.

**Dream-validated entries get preferential demurrage.** Per `../02-mortality/05-knowledge-demurrage.md`, entries that have been dream-analyzed and subsequently confirmed by waking experience decay at 0.5× the standard knowledge demurrage rate. This creates a virtuous cycle: dreaming produces hypotheses → waking validates them → validated entries persist longer → they are retrieved more frequently → their continued utility is further confirmed. The preferential rate reflects double validation (dream staging buffer + live-market confirmation).

**Dream outputs flow through Styx Archive and Styx query.** Dream journal entries are auto-backed to the Styx Archive after each `SLEEPING.DREAMING` sub-state (entry type: `"dream_journal"`). Validated dream entries enter Styx through the standard Curator promotion pipeline at provenance weight 0.6. Once promoted to `dream_validated` status, entries compete on equal footing with `self`-provenance entries. Read-only API endpoints (`GET /v1/oracle/grimoire/dreams`, `GET /v1/oracle/grimoire/hypotheses`) provide owner visibility into dream outputs.

**Dream journal enriches death testament.** During the Thanatopsis Protocol, the DreamJournal is available for the life review. A dying Golem with a rich dream history can report to its successor: "I dreamed about flash crash scenarios 47 times. Here's what I learned. I generated 12 creative strategy hypotheses; 3 were validated, 6 were refuted, 3 were never tested — the untested ones might be worth exploring." This is more valuable than any single strategy parameter because it provides a map of the explored and unexplored hypothesis space. Untested hypotheses are included in the death testament as an `<unvalidated_dreams>` XML block with `provenance: "dream_untested"`.

> **Cross-ref**: `../02-mortality/05-knowledge-demurrage.md` (demurrage rates), `../20-styx/01-architecture.md` (dream_journal entry type, provenance weights), `../20-styx/02-api-revenue.md` (dream API endpoints), `../02-mortality/06-thanatopsis.md` (death reflection, `<unvalidated_dreams>`)

### Dreaming × Context Governor

**Dreams optimize context policies offline.** The Context Governor (`../01-golem/14-context-governor.md`) assembles per-inference context windows using a learned `ContextPolicy` -- per-category token allocations, retrieval thresholds, and compression settings. During REM imagination, the dream engine runs counterfactual context assembly: "tick 42 failed -- would different context have led to a better decision?" The dream replays the tick with alternative `ContextPolicy` allocations (more contrarian evidence, fewer episodes, different retrieval thresholds) and evaluates whether the counterfactual assembly would have produced a better outcome.

Dream-generated `ContextPolicy` revisions enter the staging buffer at confidence 0.2, following the same staging pattern as other dream hypotheses (see `04-consolidation.md`). They are validated by comparing waking performance under the mutated policy against performance under the current policy. Validated mutations are promoted and applied to the active ContextPolicy; refuted mutations are discarded.

**Quantitative prediction:** Dreaming Golems converge on effective context policies 2x faster than non-dreaming Golems. The basis: offline counterfactual evaluation multiplies context experiments without consuming live inference budget. A non-dreaming Golem learns context allocations only from live trading experience (Loop 2 aggregation every 24 hours). A dreaming Golem supplements this with dozens of counterfactual context experiments per dream cycle (Loop 3), each testing alternative allocations against historical outcomes.

> **Cross-ref**: `../01-golem/14-context-governor.md` S6 (Loop 3 meta-optimization), `../01-golem/02-heartbeat.md` S9.4 (context feedback integration)

### Dreaming × Foreign Grimoire Inheritance

**At birth, the successor receives the predecessor's `PriorsPackage`.** The predecessor's death testament includes distilled knowledge: top-5 scars (highest-impact negative outcomes with lessons extracted), top-3 hypotheses (highest-confidence validated dream hypotheses), and unvalidated dream hypotheses marked `requiresInvestigation: true`. Raw episodes are not inherited -- they carry too much predecessor-specific context.

```rust
pub struct PriorsPackage {
    pub scars: Vec<Scar>,                          // top-5 by impact magnitude
    pub hypotheses: Vec<StrategyHypothesis>,        // top-3 by confidence
    pub unvalidated_dreams: Vec<DreamStagingRecord>, // predecessor's untested hypotheses
    pub playbook_snapshot: String,                  // predecessor's final PLAYBOOK.md
    pub dream_quality_calibration: DreamQualityCalibration,
}

pub struct DreamQualityCalibration {
    /// Which dream modes produced the best results
    pub nrem_yield: f64,
    pub rem_yield: f64,
    pub threat_yield: f64,
}
```

**The successor's first dream cycle processes inherited material.** Inherited entries enter the successor's Grimoire through the foreign ingestion pipeline (`ingest/*` branch) with validation lighter than marketplace purchases -- Layer 1 (TrustRAG) is skipped for predecessors with fewer than 100 Grimoire entries, since sparse Grimoires produce degenerate clusters. Adopted entries start at confidence 0.15 for dream-validated entries and 0.25 for self-validated entries (the predecessor saw them work in live trading).

Once adopted, inherited entries are processed in the successor's first dream cycle as if they were replayed episodes -- the same NREM selection algorithm applies, the same staging buffer entry point at confidence 0.2-0.3. The Zeigarnik effect drives prioritization: unvalidated dream hypotheses (incomplete ideas) are more salient than completed work, so the successor's early dreams finish the predecessor's unfinished intellectual investigations.

> **Cross-ref**: `../02-mortality/07-succession.md` (PriorsPackage, death testament), `../10-safety/03-ingestion.md` (predecessor ingestion pipeline)

### Dreaming × Hypnagogia

**Hypnagogia manages the transitions that dreaming assumes.** The three-phase dream cycle (NREM, REM, Integration) presupposes that the Golem has cleanly transitioned from waking cognition to offline processing. In biological sleep, this transition is not instantaneous -- the hypnagogic period (sleep onset) involves a distinct neurological state where waking context dissolves progressively, and the hypnopompic period (sleep return) involves recrystallization of waking context from dream outputs. Mavromatis (1987) documented that hypnagogic imagery is qualitatively different from both waking thought and dream content -- more fragmentary, more loosely associated, and often the site of creative breakthroughs (Kekule's benzene ring, Edison's steel ball technique, Dali's slumber-with-a-key).

**Onset feeds NREM.** During hypnagogic onset, the Golem performs context dissolution: releasing the waking context window's focus, capturing unresolved "waking residue" (threads of reasoning that were active but incomplete at dream entry), and transitioning the inference profile from waking (tool-heavy, action-oriented) to dreaming (reflective, generative). The waking residue captured during onset seeds NREM replay priorities -- if the Golem was mid-analysis of a puzzling trade when sleep triggered, that trade becomes the highest-priority replay target.

**Integration feeds return.** The hypnopompic return phase receives Integration's outputs and recrystallizes them into waking-compatible context. Dream insights are re-framed from the reflective register of dream cognition into the action-oriented register of waking cognition. The return phase also implements the "Dali interrupt" -- a brief window where the Golem checks if any liminal insight was forming at the boundary between dream and waking that might be lost if the transition completes too quickly.

**Budget interaction.** The onset and return phases consume 10% and 5% of the dream cycle budget respectively, leaving 85% for the three core phases. These fractions are fixed -- the liminal transitions are necessary regardless of behavioral phase or dream intensity.

> **Cross-ref**: `../06-hypnagogia/00-overview.md` (Track 5 thesis), `../06-hypnagogia/02-architecture.md` (onset/return phase specifications)

### Real-Time Dream Triggers

Two mechanisms allow dreams outside scheduled windows:

**REFLECT-phase urgency accumulator.** During each REFLECT step of the heartbeat FSM, the `bardo-dream` extension evaluates `dreamUrgency()` and accumulates the score. When the composite urgency exceeds the phase-specific threshold (0.8 for Thriving, 0.6 for uncertain states), a dream triggers even outside configured sleep windows. The urgency score incorporates `recentSurprise`, `unreplayedEpisodes`, `unresolvedArousal`, `recentLosses`, `ticksSinceLastDream`, and `epistemicDrift`. If a precondition blocks dreaming (e.g., active `ActionPermit`), urgency decays by 0.1 per tick rather than resetting, preserving the buildup.

**Owner-commanded immediate dream.** `steer("dream now")` bypasses urgency scoring entirely, triggering with implicit urgency 1.0.

**Emergency dream on terminal phase entry.** When a death trigger fires and the Golem transitions to terminal phase, `bardo-dream` triggers a compressed single-cycle dream before the Thanatopsis Protocol starts. This emergency dream runs all three phases (NREM, REM, Integration) in compressed form with a reduced turn limit. Its outputs go directly to the death testament draft rather than the standard staging buffer, since there is no time for waking validation.

### Mortality × Daimon × Memory (Existing Triple)

The existing triple interaction (documented in `../03-daimon/00-overview.md`) is unchanged but deepened by Track 4. The original lifecycle arc:

1. Birth with compressed priors → 2. Emotional learning under mortality pressure → 3. Knowledge-producing death → 4. Compressed transmission to successor

Gains a new dimension at each stage:

1. Birth with compressed priors **plus a map of the predecessor's explored hypothesis space from its dream journal**
2. Emotional learning **augmented by dream-based emotional depotentiation and threat rehearsal**
3. Knowledge-producing death **enriched by dream-refined knowledge that has been replayed, counterfactualized, and stress-tested**
4. Transmission includes **not just facts and heuristics but the dream-enhanced version: knowledge that has been offline-optimized across multiple dream cycles**

---

## The Quadruple Interaction: Complete Lifecycle

### Full Arc with Dreaming

1. **Birth**: The Golem wakes with compressed priors from its predecessor's death testament (memory), low initial confidence in inherited knowledge (mortality's generational decay), a personality baseline (daimon), and the predecessor's dream journal as a hypothesis map (dreaming). The successor knows not just what the predecessor learned but what it _wondered about_.

2. **Early Thriving**: Active trading generates episodes tagged with emotional salience. First dream cycles are exploratory — creative recombination dominates, seeking novel strategies appropriate for current market conditions. Dream content is biased toward the untested hypotheses from the predecessor's dream journal — finishing the dead Golem's unfinished intellectual work. The Zeigarnik effect (1927) ensures incomplete ideas are more salient than completed ones [ZEIGARNIK-1927].

3. **Stable Operation**: PLAYBOOK.md stabilizes through the combined action of waking Curator cycles and dream consolidation cycles. Emotional depotentiation resolves accumulated reactivity from early-phase learning. Threat simulation builds a comprehensive set of rehearsed responses. The Golem's effective strategy set grows through dream-validated hypotheses being promoted into operational use.

4. **Conservation/Decline**: Mortality pressure increases across all three clocks. Dream content shifts: less creation, more consolidation and threat preparation. The Golem dreams defensively — replaying success patterns for preservation, rehearsing failure modes for resilience. Emotional processing focuses on resolving residual high-arousal memories, preparing for the equanimity needed during the death protocol.

5. **Terminal and Death**: Dreaming ceases. The Thanatopsis Protocol runs a final, comprehensive life review that draws on the entire dream history. The death testament includes:
   - Validated dream discoveries (strategies that worked)
   - Refuted dream hypotheses (what not to try)
   - Untested dream hypotheses (the most generative successor fuel)
   - Threat records (what dangers were rehearsed)
   - Dream quality calibration (which dream modes were most productive)
   - Emotional trajectory through dreaming (how daimon evolved across the lifecycle)

6. **Succession**: The next Golem inherits a richer starting position than any non-dreaming system could produce. The Baldwin Effect operates on dream-enhanced knowledge: what crosses generations is not just experience but experience that has been dreamed about — replayed, counterfactualized, creatively recombined, stress-tested, emotionally processed, and integrated into a coherent PLAYBOOK.md. This is the full cycle of mortal intelligence: live, feel, learn, dream, die, transmit.

   The Zeigarnik effect (1927) is critical here: the predecessor's _untested_ dream hypotheses — marked `requiresInvestigation: true` in the inherited DreamJournal — are more salient to the successor than completed work. These hypotheses are prioritized in the successor's early dream cycles, creating an intellectual inheritance where each generation picks up where the last left off. The predecessor dreamed up a novel strategy but died before testing it; the successor's first dreams finish that investigation. Per `../02-mortality/07-succession.md`, inherited dream hypotheses enter at confidence 0.15 with `provenance: "inherited_dream"` — lower than fresh dream hypotheses (0.2–0.3) because they originated in a different market regime.

---

## Extension trait implementation

The dream system implements the `Extension` trait from `golem-core`, registering on two hooks:

```rust
impl Extension for DreamExtension {
    fn name(&self) -> &str { "bardo-dream" }

    fn layer(&self) -> u8 { 4 }
    // Layer 4: after heartbeat(1), lifespan(1), daimon(2), memory(3)
    // Before: cybernetics(5), coordination(6)

    async fn on_idle(&self, ctx: &mut ExtensionContext) -> Result<()> {
        // Dream scheduling: evaluate urgency, open dream branch if threshold met.
        // This hook fires when the heartbeat FSM enters SLEEPING state.
        self.scheduler.evaluate_and_maybe_dream(ctx).await
    }

    async fn after_turn(&self, ctx: &mut ExtensionContext) -> Result<()> {
        // Accumulate dream urgency from waking ticks.
        // Update DreamState (urgency score, episode counts).
        self.scheduler.accumulate_urgency(ctx).await
    }
}
```

The `on_idle` hook handles dream scheduling -- when the heartbeat FSM enters SLEEPING, the dream extension evaluates whether to open a dream branch. The `after_turn` hook accumulates urgency from each waking tick, tracking surprise, consolidation debt, and emotional load.

**DAG position**: Layer 4 runs after memory extensions have finished their `after_turn` work (Grimoire writes, Curator tagging), so the dream urgency calculation sees up-to-date episode counts and tags. It runs before cybernetics (Layer 5) and coordination (Layer 6), so context policy mutations from dreams are available for the next waking tick's context assembly.

## Event Fabric cross-references

The dream system emits five canonical `GolemEvent` variants through the Event Fabric (see `rewrite4/14-events.md`):

| Event | Wire name | When emitted |
|---|---|---|
| `DreamStart` | `dream.start` | Dream branch opens. Trigger type recorded. |
| `DreamPhase` | `dream.phase` | Each phase transition (NREM -> REM -> Integration). Budget token allocation included. |
| `DreamHypothesis` | `dream.hypothesis` | REM generates a counterfactual hypothesis. Seed episode tick for provenance. |
| `DreamCounterfactual` | `dream.counterfactual` | Counterfactual evaluated. Outcome, lesson, and confidence recorded. |
| `DreamComplete` | `dream.complete` | Dream branch closes. Full summary: episodes replayed, counterfactuals generated, insights crystallized, heuristics proposed, episodes depotentiated, playbook edits. |

All events carry the standard base fields (`timestamp`, `golem_id`, `sequence`, `tick`) and are broadcast through the `tokio::broadcast` channel with ring buffer replay for reconnecting subscribers.

The TUI's Dreams tab subscribes to the `dream` event category; switching away unsubscribes to minimize bandwidth.

> **Cross-ref**: `rewrite4/14-events.md` (GolemEvent enum, section 2.4), `rewrite4/10-surfaces.md` (TUI dream pane)

---

## Updated architecture diagram

```
┌───────────────────────────────────────────────────────────────────────┐
│                         THE GOLEM                                     │
│                                                                       │
│  ┌─────────────────── TRACK 1: MORTALITY ──────────────────────────┐  │
│  │  Three Clocks (Economic, Epistemic, Stochastic)                 │  │
│  │  Five Phases (Thriving → Stable → Conservation → Declining → T) │  │
│  │  Composite Vitality · Behavioral Phase                          │  │
│  └────────────────────────────┬────────────────────────────────────┘  │
│                               │                                       │
│            ┌──────────────────┼──────────────────┐                    │
│            │                  │                  │                    │
│            ▼                  ▼                  ▼                    │
│  ┌──── TRACK 2 ────┐ ┌── TRACK 3 ──┐ ┌──── TRACK 4 ────────────┐   │
│  │    DAIMON        │ │   MEMORY    │ │     DREAMING             │   │
│  │                  │ │             │ │                           │   │
│  │ Appraisal Engine │ │ Grimoire    │ │ ┌─NREM──┐  ┌──REM────┐  │   │
│  │ PAD Vectors      │ │  Episodes   │ │ │Replay │  │Imagine  │  │   │
│  │ Mood EMA         │ │  Insights   │ │ │Credit │  │Counter  │  │   │
│  │ Behavioral Mod.  │ │  PLAYBOOK   │ │ │Perturb│  │Create   │  │   │
│  │ Life Review      │ │  Cache      │ │ └───┬───┘  │Threats  │  │   │
│  │                  │ │             │ │     │      │Emotion  │  │   │
│  │ [guides dream    │ │ [feeds      │ │     │      └──┬──────┘  │   │
│  │  content via     │◄│  dream      │◄│     └────┬───┘         │   │
│  │  emotional tags] │ │  material]  │ │          ▼              │   │
│  │                  │ │             │ │   ┌─INTEGRATE───────┐   │   │
│  │ [receives dream  │ │ [receives   │ │   │Consolidate     │   │   │
│  │  depotentiation]►│ │  dream     │►│   │PLAYBOOK.md evo │   │   │
│  │                  │ │  outputs]   │ │   │Dream journal   │   │   │
│  └──────────────────┘ └─────────────┘ │   └─────────────────┘   │   │
│                                       └─────────────────────────┘   │
│                                                                       │
│  ┌─────────────────── HEARTBEAT FSM ──────────────────────────────┐  │
│  │  Waking: Probes → Escalation → Action                          │  │
│  │  Dreaming: Suspended (Dream Engine runs instead)                │  │
│  │  Dying: Thanatopsis Protocol (enriched by dream journal)        │  │
│  └────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

---

## What Dreaming Adds to the Existing System

### Quantitative Predictions

These predictions are derived from the research literature and are the basis for the falsifiability requirement (Design Principle 7 from `00-overview.md`):

| Metric                                 | Prediction                             | Basis                                                                                           |
| -------------------------------------- | -------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Time-to-adaptation after regime change | 30–50% faster for dreaming Golems      | Jensen et al. (2024): meta-RL agents that learn when to plan adapt faster [JENSEN-2024]         |
| Response latency to rehearsed threats  | 5–10x faster than unrehearsed          | Revonsuo (2000): threat rehearsal produces automatic responses [REVONSUO-2000]                  |
| Successor boot performance             | 15–25% better first-week Sharpe        | Baldwin Effect: dream-enhanced priors provide richer starting position                          |
| PLAYBOOK.md convergence speed          | 2–3x faster to stable strategy set     | Replay multiplies learning signal [LIN-1992]; consolidation extracts rules faster [WAGNER-2004] |
| Emotional reactivity after trauma      | 40–60% lower arousal to similar events | Walker & van der Helm (2009): REM depotentiates emotional charge [WALKER-VAN-DER-HELM-2009]     |
| Hidden pattern discovery               | 2x more likely to find regime rules    | Wagner et al. (2004): sleep subjects 2.5x more likely to discover hidden rules [WAGNER-2004]    |
| Death testament richness               | 3–5x more hypothesis space coverage    | Dream journal provides tested and untested hypotheses                                           |

### Control Experiment

The control is straightforward: run matched pairs of Golems -- one with dreaming enabled, one without -- in identical market conditions with identical parameters (same owner, same strategies, same capital, same risk limits). Measure the metrics above over a 30-day lifecycle.

The dreaming Golem should outperform on adaptation speed, threat response, and successor quality. The non-dreaming Golem may outperform on raw trade volume (no time lost to dreaming) and simplicity of operation. If the dreaming Golem does not show statistically significant improvement on at least three of the seven metrics, dreaming should be disabled — the cost does not justify the benefit.

---

## Integration Status with Existing Documents

The following changes to other PRD documents have been completed or are needed to support Track 4. This section serves as a cross-reference checklist — it describes changes _in other docs_, not in this one.

### Daimon PRD (../03-daimon/) — Completed

| File                                  | Change                                                                             | Status |
| ------------------------------------- | ---------------------------------------------------------------------------------- | ------ |
| `../03-daimon/00-overview.md`         | Track 4 added to four-track system, package map includes `golem-dreams`            | Done   |
| `../03-daimon/00-overview.md`         | Dream Engine as sixth interaction partner                                          | Done   |
| `../03-daimon/01-appraisal.md`        | `dream_outcome` appraisal trigger defined                                          | Done   |
| `../03-daimon/02-emotion-memory.md`   | Dream-retrieval strengthening (+0.5), Dream-Memory-Emotion Triangle                | Done   |
| `../03-daimon/04-mortality-daimon.md` | Dream-mediated resolution of economic anxiety, epistemic vertigo, stochastic dread | Done   |
| `../03-daimon/03-behavior.md`         | Dream content modulation by mood, minimum creative allocation (20%)                | Done   |
| `../03-daimon/05-death-daimon.md`     | DreamJournal in Phase II Life Review                                               | Done   |
| `../03-daimon/09-evaluation.md`       | Dream-Daimon interaction metrics, cognitive quality dashboard                      | Done   |

### Mortality PRD (../02-mortality/) — Completed

| File                                        | Change                                                                           | Status |
| ------------------------------------------- | -------------------------------------------------------------------------------- | ------ |
| `../02-mortality/00-thesis.md`              | Cross-subsystem table includes Dream interactions                                | Done   |
| `../02-mortality/01-architecture.md`        | Dream budget column per behavioral phase                                         | Done   |
| `../02-mortality/02-epistemic-decay.md`     | Dreaming as partial countermeasure to epistemic decay                            | Done   |
| `../02-mortality/04-economic-mortality.md`  | Dream compute in burn rate (5–10% inference)                                     | Done   |
| `../02-mortality/05-knowledge-demurrage.md` | Dream-validated entries at 0.5× demurrage rate                                   | Done   |
| `../02-mortality/06-thanatopsis.md`         | DreamJournal enrichment, `<unvalidated_dreams>` XML block                        | Done   |
| `../02-mortality/07-succession.md`          | DreamJournal inheritance, Zeigarnik prioritization, `inherited_dream` provenance | Done   |
| `../02-mortality/08-mortality-affect.md`    | Dream-mediated resolution of mortality emotions                                  | Done   |
| `../02-mortality/12-integration.md`         | Dream integration section (budget, content bias, DreamJournal)                   | Done   |

### Memory PRD (../04-memory/) — Completed

| File                                  | Change                                                                                                                      | Status |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ------ |
| `../04-memory/00-overview.md`         | Third processing mode (offline dreaming), shared constants                                                                  | Done   |
| `../04-memory/01-grimoire.md`         | Dream provenance, DreamJournalEntry schema, Curator ↔ Dream coordination, dream-retrieval strengthening, dream yield metric | Done   |
| `../04-memory/02-emotional-memory.md` | Dream-Emotional Memory Integration, REM depotentiation                                                                      | Done   |
| `../04-memory/03-mortal-memory.md`    | Dreaming under mortality pressure                                                                                           | Done   |
| `../20-styx/01-architecture.md`       | `dream_journal` entry type, upload after SLEEPING.DREAMING, dream provenance weight 0.6, `dream_validated` promotion path  | Done   |
| `../04-memory/06-economy.md`          | Dream knowledge lifecycle, trophic levels                                                                                   | Done   |
| `../20-styx/02-api-revenue.md`        | Dream Grimoire API endpoints (read-only)                                                                                    | Done   |

### Golem PRD (../01-golem/) — Partially Completed

| File                          | Change                                                                | Status  |
| ----------------------------- | --------------------------------------------------------------------- | ------- |
| `../01-golem/02-heartbeat.md` | `GolemMode`, SLEEPING.DREAMING, SSE events, chat hold during dreaming | Done    |
| `../01-golem/06-creation.md`  | First-dream milestone (4–6 hours, 50–200 episodes)                    | Done    |
| Observability                 | Dream-specific Prometheus metrics, OpenTelemetry spans, health fields | **Gap** |

### Remaining Gaps

1. **Observability**: No dream-specific Prometheus metrics (dream cycles, duration, hypotheses staged/validated/refuted), no OpenTelemetry spans for dream phases, no dream state in health/readiness responses. These should be added when the observability PRD is next revised.

2. **Evaluation matrix** (`../03-daimon/09-evaluation.md`): The 2×2×2 evaluation matrix (Mortality × Daimon × Memory) does not include Dreaming as a factor. A 2×2×2×2 matrix (16 configurations) would allow systematic comparison of dreaming vs non-dreaming across all other system combinations.

3. **Architecture diagrams**: Several docs' architecture diagrams (notably `../03-daimon/00-overview.md`) do not yet include the Dream Engine. These should be updated when diagrams are next revised.

---

## Updated Terminology

| Term                            | Definition                                                                                                                       |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **Dream Cycle**                 | A complete three-phase sequence: NREM-like replay → REM-like imagination → Integration.                                          |
| **DreamState**                  | The Golem's state during offline processing: current phase, budget, progress.                                                    |
| **DreamJournal**                | Persistent log of all dream cycles, outputs, and validation results.                                                             |
| **DreamScheduler**              | Component that determines when to dream, for how long, with what budget.                                                         |
| **ReplayEngine**                | Selects, prioritizes, and re-processes episodes during NREM-like phase.                                                          |
| **ImaginationEngine**           | Generates counterfactual, creative, and adversarial scenarios during REM-like phase.                                             |
| **ThreatSimulator**             | Generates DeFi-specific threat scenarios and rehearses responses.                                                                |
| **DreamConsolidator**           | Integrates dream outputs into Grimoire and PLAYBOOK.md through staging buffer.                                                   |
| **Strategy Hypothesis**         | A dream-generated strategy proposal at low confidence, requiring live validation.                                                |
| **Staged Revision**             | A proposed PLAYBOOK.md change from dreaming, waiting for live validation.                                                        |
| **Threat Record**               | A rehearsed threat scenario with trigger conditions and pre-planned responses.                                                   |
| **Dream Quality Score**         | Self-assessed quality of a dream cycle, validated against subsequent outcomes.                                                   |
| **Exploration Temperature (τ)** | LLM sampling temperature during dream generation; controls scenario divergence.                                                  |
| **Perturbed Replay**            | Episode replay with injected noise (slippage, gas, latency) for robustness.                                                      |
| **Emotional Depotentiation**    | REM-like processing that reduces emotional charge while preserving informational content.                                        |
| **[CORE] Dreams**               | Lightweight T0-only dreaming tier: compressed replay + simplified counterfactual + basic consolidation. Default for all Golems.  |
| **[HARDENED] Dreams**           | Full T1–T2 dreaming tier: complete three-phase cycle with all operations including creative recombination and threat simulation. |
| **Dream Outcome Event**         | Appraisal event emitted when a staged dream hypothesis is confirmed or refuted by live trading.                                  |
| **Knowledge Demurrage**         | Time-based decay of knowledge entries; dream-validated entries decay at 0.5× standard rate.                                      |
| **First Dream Threshold**       | Minimum episode count (50–200) before dream cycles produce meaningful patterns.                                                  |

---

## Citation Summary

| Citation Key                  | Source                                                                                  |
| ----------------------------- | --------------------------------------------------------------------------------------- |
| [SUTTON-1991]                 | Sutton. "Dyna, an integrated architecture." _ACM SIGART Bulletin_, 1991.                |
| [MCCLELLAND-1995]             | McClelland et al. "Complementary Learning Systems." _Psychological Review_, 1995.       |
| [REVONSUO-2000]               | Revonsuo. "The reinterpretation of dreams." _Behavioral and Brain Sciences_, 2000.      |
| [MCGAUGH-2004]                | McGaugh. "The amygdala modulates consolidation." _Annual Review of Neuroscience_, 2004. |
| [WALKER-VAN-DER-HELM-2009]    | Walker & van der Helm. "Overnight therapy?" _Psychological Bulletin_, 2009.             |
| [NORTON-MOCHON-ARIELY-2012]   | Norton et al. "The IKEA Effect." _JCSP_, 2012.                                          |
| [FRANCESCHELLI-MUSOLESI-2024] | Franceschelli & Musolesi. "Creativity and ML." _ACM Computing Surveys_, 2024.           |
| [JENSEN-2024]                 | Jensen et al. "A recurrent network model of planning." _Nature Neuroscience_, 2024.     |
| [LIN-1992]                    | Lin. "Self-improving reactive agents." _Machine Learning_, 1992.                        |
| [WAGNER-2004]                 | Wagner et al. "Sleep inspires insight." _Nature_, 2004.                                 |
| [ZEIGARNIK-1927]              | Zeigarnik. Unfinished task recall effect, 1927.                                         |
