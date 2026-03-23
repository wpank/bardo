# Dream Consolidation: Knowledge Integration and PLAYBOOK.md Evolution [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `00-overview.md`, `01-architecture.md`, `golem-grimoire`, `golem-daimon` (optional)

---

> **Reader orientation:** This document specifies how dream outputs integrate into the Grimoire (persistent knowledge base) and PLAYBOOK.md (evolved strategy playbook) within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers the staging buffer, the Grimoire Admission Gate (quality filter), dream-specific quality calibration, emotional processing (depotentiation via the Daimon, the affect engine), the DreamJournal, and safety constraints preventing untested hypotheses from reaching operational status. Prerequisites: the Dreams overview (`00-overview.md`), architecture (`01-architecture.md`), and Grimoire spec (`../04-memory/`). For a full glossary, see `prd2/shared/glossary.md`.

## The Integration Problem

Dreaming produces knowledge. But dream-generated knowledge is not automatically trustworthy. Biological brains solve this through a consolidation process where dream content is selectively integrated into long-term memory — most dreams are forgotten; only those that survive a filtering process become part of the organism's operational knowledge. Born and Wilhelm (2012) showed this consolidation is selective: it preferentially preserves memories with relevance for future plans and goals, and produces qualitative changes including extraction of explicit knowledge from implicit learning [BORN-WILHELM-2012].

For the Golem, the consolidation problem has a specific form: dream outputs (replay insights, counterfactual conclusions, creative hypotheses, threat responses) must be integrated into the Grimoire and PLAYBOOK.md without contaminating operational knowledge with untested hallucinations. The DreamConsolidator is the gatekeeper — it implements the waking-dream boundary that prevents dream content from directly controlling behavior.

---

## PLAYBOOK.md Evolution Through Dreaming

### The Staging Buffer

Dream-proposed revisions to PLAYBOOK.md never apply directly. They enter a staging buffer:

```rust
pub struct StagedPlaybookRevision {
    pub id: String,
    pub revision_type: PlaybookRevisionType,
    pub target_section: String,
    pub current_content: String,
    pub proposed_content: String,
    pub evidence: String,
    pub validation_criterion: String,
    pub falsification_criterion: String,
    pub confidence: f64,
    pub dream_cycle_origin: String,
    pub staged_at: u64,
    pub expires_at: u64,
    pub status: RevisionStatus,
    pub validation_episodes: Vec<String>,
}

pub enum PlaybookRevisionType {
    AddHeuristic,
    ModifyHeuristic,
    RemoveHeuristic,
    AddGuard,
    ModifyParameter,
    AddSection,
}

pub enum RevisionStatus {
    Staged,
    PartiallyValidated,
    Validated,
    Applied,
    Expired,
    Refuted,
}
```

**Lifecycle of a staged revision**:

```
Dream generates revision
    │
    ▼
[Deduplication check] ── duplicate ──► discard
    │ unique
    ▼
[Plausibility check] ── implausible ──► discard
    │ plausible
    ▼
[Stage in buffer] (confidence 0.2–0.3)
    │
    ▼
[Live trading proceeds]
    │
    ▼ (matching market conditions occur)
[Validation episode] ── refuted ──► mark refuted, remove
    │ consistent
    ▼
[Boost confidence] (0.3 → 0.4 → ... → 0.7)
    │ confidence >= 0.7
    ▼
[Apply to PLAYBOOK.md]
```

**Expiration**: Staged revisions that are never tested (because the relevant market conditions never occurred) expire after a configurable number of ticks (default: 5,000, approximately 3.5 days at 1 tick/minute). Expired revisions are logged in the DreamJournal as "untested" and may be re-generated in future dream cycles if the underlying conditions remain relevant.

### Grimoire Admission Gate for Dream Outputs

Staged revisions reaching `validated` status (confidence >= 0.7) still pass through the Grimoire Admission Gate (see `../04-memory/01-grimoire.md`) before becoming operational entries. Two quality gates in series:

1. **Dream staging buffer** (this document): Validates the hypothesis is plausible, testable, and internally consistent. Filters obviously wrong dream artifacts.
2. **Grimoire Admission Gate** (A-MAC five-factor scoring): Validates the promoted hypothesis is novel, useful, and factually consistent with existing Grimoire entries. Filters valid but redundant or outdated entries.

Both gates are necessary — validated but redundant entries waste retrieval bandwidth. The staging buffer catches dream noise; the Admission Gate catches validated noise. A memory management study (arXiv:2505.16067) found selective admission produces 10% absolute gains over unfiltered storage.

**A-MAC scoring dimensions** (from `../04-memory/01-grimoire.md`): The Admission Gate scores entries on five factors — Accuracy, Memorability, Actionability, Consistency (with existing Grimoire), and (non-)Redundancy. The minimum admission threshold is 0.45. Dream outputs use the dream-specific weight calibration below, which reduces the specificity bar while increasing the consistency bar — reflecting the different failure modes of dream-generated vs waking-generated content.

### Dream-Specific Quality Calibration

Dream outputs are by design more exploratory than waking cognition. The artifact quality scoring (see `../04-memory/01-grimoire.md`) is calibrated differently for dream outputs:

| Dimension     | Standard Weight | Dream Weight | Rationale                                                            |
| ------------- | --------------- | ------------ | -------------------------------------------------------------------- |
| Specificity   | 0.25            | 0.10         | Dream hypotheses are intentionally abstract                          |
| Actionability | 0.25            | 0.25         | Unchanged — actionable outputs are always valued                     |
| Novelty       | 0.20            | 0.20         | Unchanged — redundant dreams waste processing                        |
| Verifiability | 0.15            | 0.20         | Dream outputs need stronger verifiability signal                     |
| Consistency   | 0.15            | 0.25         | Increased — arbitrary hallucinated content is the primary dream risk |

**Exception**: Dream-generated threat responses and PLAYBOOK.md guards retain full specificity weight (0.25). Safety outputs must be concrete regardless of their origin.

**Red flags specific to dream outputs** (-0.1 each): unfalsifiable dream hypotheses, pattern claims without example episodes, threat responses that can't be validated against current on-chain state.

### Types of PLAYBOOK.md Evolution

**1. Parameter adjustment** — The most common and safest type:

```
Current: "Enter LP position when pool APY > 15%"
Dream finding: Perturbed replay showed strategy fails when APY > 15%
               but gas > 80 gwei, because gas costs consume the yield.
Proposed: "Enter LP position when pool APY > 15% AND gas < 60 gwei"
Validation: Next time APY > 15% AND gas > 60 gwei, observe whether
            skipping the entry would have been correct.
```

**2. Guard addition** — Adding a new safety condition:

```
Dream finding: Threat simulation revealed vulnerability to oracle
               deviation > 2% during position entry.
Proposed: Add guard: "Before entering any position, verify oracle
          deviation from spot < 1.5%. If > 1.5%, delay entry."
Validation: Monitor oracle deviation at entry points. Measure whether
            delayed entries under this guard outperform immediate entries.
```

**3. Strategy hypothesis promotion** — Moving a dream-generated strategy into operational use:

```
Dream finding: Creative recombination produced "use momentum reversal
               signals to time LP rebalancing"
Validation journey:
  - Dream confidence: 0.2
  - After 3 consistent observations: 0.4
  - After shadow-execution test (5 simulated trades): 0.6
  - After 3 live micro-tests with minimum position: 0.7
  - Promoted to PLAYBOOK.md
```

**4. Heuristic pruning** — Removing or downweighting knowledge that dreaming has shown to be unreliable:

Hobson and Friston (2012) identified that a core function of dreaming is model complexity reduction — pruning redundant parameters while preserving accuracy [HOBSON-FRISTON-2012]. The Golem's PLAYBOOK.md accumulates heuristics over time. Some become redundant (subsumed by more general rules). Some become stale (market conditions changed). Some contradict each other.

```
Dream finding: Replay revealed that heuristics #14 and #27 in
               PLAYBOOK.md give contradictory advice when gas > 100 gwei
               AND volatility is high. In 8/10 replayed episodes, #27
               was correct and #14 was wrong.
Proposed: Downweight #14 confidence by 0.2. Add a note: "This heuristic
          may be unreliable during high-gas, high-volatility conditions.
          Consider #27 as the primary guide."
```

---

## Grimoire Integration

### Dream-Generated Insights

Dream replay and imagination produce new Insights that enter the Grimoire's SQLite database. These follow the same schema as waking Insights but carry special provenance:

```rust
pub struct DreamInsight {
    pub insight: Insight, // base Grimoire Insight fields
    pub provenance: DreamProvenance,
    pub dream_cycle_id: String,
    pub confidence: f64,
    pub supporting_episodes: Vec<String>,
    pub validation_status: DreamValidationStatus,
}

pub enum DreamProvenance {
    DreamReplay,
    DreamCounterfactual,
    DreamCreative,
    DreamThreat,
}

pub enum DreamValidationStatus {
    Pending,
    Partial,
    Validated,
    Refuted,
}
```

**Confidence differential**: Dream-generated insights start at 0.2–0.3, below the 0.5 threshold for waking-generated insights. This implements the conservative offline learning principle from Kumar et al. (2020) [KUMAR-CQL-2020]: dream knowledge is inherently less reliable than direct experience because it is generated by the LLM's implicit model, which may diverge from actual market dynamics. The confidence gap ensures that dream insights never override waking insights unless they are independently validated through live trading.

**Interaction with the Curator cycle**: The Curator (every 50 ticks during waking) encounters dream-generated insights in the Grimoire alongside waking-generated insights. It treats them the same way it treats Styx-retrieved or inherited insights: with initial skepticism, subject to validation through matching against new episodes. This is the IKEA Effect applied to dream knowledge — the Golem must re-earn confidence through its own experience [NORTON-MOCHON-ARIELY-2012].

### Dream-Generated Threat Records

Threat simulation outputs are stored as a special Grimoire entry type:

```rust
pub struct ThreatRecord {
    pub id: String,
    pub scenario: SyntheticScenario,
    pub rehearsed_responses: Vec<RehearsedThreatResponse>,
    pub trigger_conditions: Vec<String>,
    pub last_rehearsed: u64,
    pub activations: u32,
    pub false_alarms: u32,
    pub true_positives: u32,
}

pub struct RehearsedThreatResponse {
    pub response: String,
    pub expected_outcome: String,
    pub confidence: f64,
}
```

Threat records are indexed for fast retrieval during waking probes. When the Golem's probe system detects conditions matching a threat record's trigger conditions, the rehearsed response is immediately available — reducing decision latency from "analyze from scratch" to "retrieve rehearsed response and validate applicability."

---

## Emotional Processing in Dreams

### The REM Depotentiation Cycle

If `golem-daimon` is enabled, the Integration phase includes emotional processing following Walker and van der Helm's "Sleep to Forget, Sleep to Remember" model (2009) [WALKER-VAN-DER-HELM-2009]:

**Mechanism**: During the REM-like phase, high-arousal episodes are reprocessed. The LLM re-examines the episode with an explicit instruction to separate informational content from emotional reaction:

```
Emotional Depotentiation Session

Episode: Trade #4,872 [tagged: fear (0.8), anger (0.6)]
Original emotional context: "Panic exit during flash crash. Gas spike
prevented orderly exit. Lost 8.3% in 4 minutes."

Reprocess this episode with reduced emotional weighting:

1. Strip the emotional language. What actually happened in factual terms?
2. What was the information content of this episode (separate from the
   emotional reaction)?
3. What is the ONE lesson from this episode that should persist,
   independent of how it felt?
4. Should the emotional tag on this episode be reduced?
   Recommend new arousal level (0-1).
```

**Effect on PAD vectors**: After depotentiation, the episode's emotional tag is updated:

- Arousal dimension reduced by a decay factor (typically 0.3–0.5 per dream cycle)
- Pleasure dimension moved toward neutral
- Dominance dimension unchanged (structural learning about control persists)

**Multiple cycles**: Full depotentiation requires multiple dream cycles. A highly traumatic episode (arousal > 0.8) may take 3–5 REM-like processing sessions before arousal drops below the retrieval salience threshold. This matches the biological pattern where trauma resolution requires multiple sleep cycles [WALKER-VAN-DER-HELM-2009].

**Why this matters for trading**: A Golem that experienced a flash crash and never depotentiates the emotional memory will be permanently biased toward premature exits during any volatility. The emotional tag "fear: 0.8" on the crash episode causes mood-congruent retrieval during subsequent volatile periods, triggering anxiety cascades that impair decision quality. Emotional depotentiation reduces this reactivity while preserving the factual lesson ("monitor gas prices during high volatility for orderly exit").

---

## Compression Progress as Quality Metric (v2)

Schmidhuber (2009) proposed that curiosity is the desire for data enabling compression progress — the first derivative of subjective compressibility. Creativity is generating data that forces compression progress [SCHMIDHUBER-2009].

**v2 implementation**: After each dream cycle, measure the Golem's model complexity:

- Number of heuristics in PLAYBOOK.md
- Average length of heuristics
- Number of active Grimoire insights
- Redundancy ratio (pairs of insights that could be merged)

If dreaming reduces complexity while maintaining predictive accuracy, the dream was productive. If it increases complexity without improving accuracy, the dream added noise. This metric — compression progress rate — becomes a fitness function for evaluating dream quality and calibrating dream scheduling.

---

## Schema Integration

Bartlett (1932) and Piaget established that new information is not passively absorbed but actively assimilated into existing cognitive schemas [BARTLETT-1932]. Dream-generated knowledge must be integrated into the Golem's existing understanding — not just appended to a list.

**Implementation**: During Integration, the DreamConsolidator checks each dream output against existing PLAYBOOK.md structure:

1. **Schema-consistent additions** — Dream outputs that fit naturally into existing PLAYBOOK.md sections. These are cheaply integrated. Following Kumaran, Hassabis, and McClelland (2016), schema-consistent information can be rapidly integrated without full consolidation [KUMARAN-2016].

2. **Schema-inconsistent additions** — Dream outputs that contradict existing structure. These are expensive to integrate and require explicit conflict resolution. They are flagged for deeper processing in the next dream cycle.

3. **Schema-extending additions** — Dream outputs that require new sections in PLAYBOOK.md. These represent genuine learning — the Golem has discovered a pattern that its existing framework cannot accommodate. New sections are created at low confidence and populated incrementally.

### Curator ↔ Dream Coordination

Bidirectional coordination between the Curator cycle and Dream Engine:

**Curator → Dream**: The Curator tags ambiguous episodes for dream replay. Episodes with contradictory patterns, insufficient evidence, or unresolved conflicts → `dream_priority: high`. The Dream Engine's utility scheduler (Mattar & Daw, 2018) uses this as a boost factor in replay selection, prioritizing ambiguous episodes for exploratory analysis.

**Dream → Curator**: Dream insights at `partially_validated` status are flagged for the next Curator cycle. The Curator can accelerate promotion if corroborating waking evidence exists — a dream hypothesis that matches an independently observed waking pattern gains confidence faster than either signal alone.

This bidirectional loop implements the episodic-to-semantic consolidation pipeline (see `../04-memory/01-grimoire.md`): the Curator identifies gaps, the Dream Engine explores them, and the Curator validates the results against waking experience.

---

## The Dream Journal

Every dream cycle produces a comprehensive journal entry. The DreamJournal serves three functions:

1. **Operational record**: What the Golem dreamed, what it concluded, and what changes resulted. Available for owner inspection.

2. **Successor knowledge**: During the Thanatopsis Protocol, the DreamJournal enriches the death testament. A dying Golem with a rich dream journal can report not just what it did but what it _imagined_ — which scenarios it stress-tested, which creative hypotheses it generated, which threats it rehearsed. This provides successors with a map of the explored hypothesis space, preventing redundant exploration.

3. **Validation tracking**: Dream outputs are tagged with validation status. Over time, this produces a meta-level insight: how reliable is this Golem's dreaming? Which dream modes (replay, counterfactual, creative, threat) produce the most validated insights? This calibrates future dream scheduling — dream more in modes that produce results, dream less in modes that don't.

```rust
pub struct DreamJournal {
    pub entries: Vec<DreamJournalEntry>,

    pub total_cycles: u32,
    pub total_cost_usdc: f64,
    pub hypotheses_generated: u32,
    pub hypotheses_validated: u32,
    pub hypotheses_refuted: u32,
    pub playbook_revisions_applied: u32,
    pub threats_rehearsed: u32,
    pub threats_activated: u32,
    pub emotional_episodes_processed: u32,

    pub validation_rate: f64,
    pub false_alarm_rate: f64,
    pub cost_per_validated_insight: f64,

    pub replay_validation_rate: f64,
    pub counterfactual_validation_rate: f64,
    pub creative_validation_rate: f64,
    pub threat_predictive_value: f64,
}
```

---

## Safety Constraints on Consolidation

1. **Dream outputs cannot modify PolicyCage constraints.** No dream-generated strategy can override risk limits, position size caps, or the DeFi Constitution. If a dream proposes a strategy that requires exceeding PolicyCage limits, the strategy is discarded.

2. **Dream outputs cannot bypass the staging buffer.** No path exists from dream output to live PLAYBOOK.md execution without passing through staging, validation, and promotion. The staging buffer is the Weismann barrier — it cannot be circumvented.

3. **Dream outputs are bounded by the historical data distribution.** Following Kumar et al. (2020), strategies that require actions far outside the Golem's historical experience distribution are flagged as high-risk and require additional validation (more episodes, higher confidence threshold for promotion) [KUMAR-CQL-2020].

4. **Maximum staged revisions.** The staging buffer has a configurable cap (default: 10). If the buffer is full, new revisions must compete for slots based on utility score. This prevents runaway hypothesis generation from consuming all cognitive capacity.

5. **Owner visibility.** All dream outputs, staging decisions, and PLAYBOOK.md revisions are logged and available for owner inspection. The owner can manually approve, reject, or modify any staged revision.

6. **PLAYBOOK.md single-writer enforcement.** Only the Dream Integration Phase 3 writes to PLAYBOOK.md. No other code path -- not the Curator, not micro-consolidation, not waking inference -- may modify PLAYBOOK.md directly. The Curator proposes changes by writing to the staging buffer; the Dream Integration phase applies them. Micro-consolidation (the 60-second lightweight fiber described in `01-architecture.md`) explicitly never writes to PLAYBOOK.md; it updates Grimoire metadata only. This single-writer constraint prevents concurrent modification conflicts and ensures every PLAYBOOK.md change has passed through the full staging-validation-promotion pipeline.

---

## Knowledge Demurrage for Dream-Validated Entries

Dream-validated entries (those that passed through the staging buffer, reached confidence ≥ 0.5, and were confirmed by waking experience) receive preferential treatment in the knowledge demurrage system. Per `../02-mortality/05-knowledge-demurrage.md`, dream-validated entries decay at **0.5× the standard demurrage rate**. The rationale: entries that survived both dream analysis and live-market confirmation have been subjected to two independent validation processes and represent higher-quality knowledge.

This creates a virtuous cycle: dreaming produces hypotheses → waking validates them → validated entries persist longer in the active Grimoire → they are retrieved more frequently during waking decisions → their continued utility is further confirmed.

> **Cross-ref**: `../02-mortality/05-knowledge-demurrage.md` (demurrage rates by provenance)

---

## Dream Outcome Events (Daimon Integration)

When a staged dream hypothesis is confirmed or refuted by live trading, the DreamConsolidator emits a `dream_outcome` appraisal event to the Daimon engine. This closes the loop between dreaming and emotional processing:

```rust
pub struct DreamOutcomeEvent {
    pub hypothesis_id: String,
    pub validated: bool,
    pub confidence: f64,
    pub dream_cycle_origin: String,
    pub validation_episodes: Vec<String>,
    pub pnl_impact: Option<f64>,
}
```

**Appraisal mapping** (from `../03-daimon/01-appraisal.md`):

- Validated hypothesis → mild joy (P+0.2, A+0.1, D+0.1) — the Golem's dreams proved prescient
- Refuted hypothesis → moderate surprise (P-0.1, A+0.2, D-0.1) — recalibrate dream confidence

The `dream_outcome` trigger is optional (not in the mandatory-events list) -- it fires only when `golem-daimon` is enabled.

> **Cross-ref**: `../03-daimon/01-appraisal.md` (dream_outcome trigger), `../03-daimon/00-overview.md` (Dream Engine as sixth interaction partner)

---

## Styx Archive integration

Dream journal entries are automatically backed up to the Styx Archive after each `SLEEPING.DREAMING` sub-state completion. The Vault preserves dream content for two purposes: insurance (recoverable after crashes) and death reflection (Thanatopsis Phase II).

**Entry type:** `"dream_journal"` in the Styx Archive data model.

**Payload:** Compressed dream history including validated hypotheses, refuted hypotheses, untested hypotheses, and threat records. The Vault does not store raw replay prompts or LLM outputs -- only the structured DreamJournalEntry.

**Upload trigger:** Dream cycle completion (`SLEEPING.DREAMING` -> `SLEEPING`). The upload is non-blocking -- if the Styx Archive upload fails, the dream cycle still completes and the entry is retried on the next cycle.

**Death testament enrichment:** During Thanatopsis Phase II (Reflect), the Golem's life review includes an `<unvalidated_dreams>` XML block containing untested hypotheses from the DreamJournal. These are tagged `provenance: "dream_untested"` in the death testament, giving successors a map of the explored but unconfirmed hypothesis space.

> **Cross-ref**: `../20-styx/01-architecture.md` (entry types, upload triggers), `../02-mortality/06-thanatopsis.md` (death reflection, `<unvalidated_dreams>` block)

---

## Styx query integration

Dream-sourced entries flow into Styx (shared RAG knowledge) through the standard Grimoire promotion pipeline -- they do not have a direct path to Styx.

**Provenance weight:** `dream: 0.6` in Styx retrieval scoring. Dream-sourced entries are weighted below `self: 1.0` and `clade: 0.8` but above `inherited: 0.4`. This reflects their intermediate evidential quality -- validated by the dream staging buffer and the Grimoire Admission Gate, but not by the full weight of repeated live-market confirmation.

**Promotion path:** Dream entries that reach `dream_validated` status (confidence >= 0.5 after waking confirmation) are promoted to `provenance: "self"` with a `dream_validated: true` metadata flag. This flag is informational -- once promoted, the entry competes on equal footing with other `self`-provenance entries in Styx retrieval.

**API access:** Read-only endpoints for browsing dream journal entries and staged hypotheses:

- `GET /v1/oracle/grimoire/dreams` -- list dream journal entries
- `GET /v1/oracle/grimoire/dreams/:cycleId` -- dream cycle detail
- `GET /v1/oracle/grimoire/hypotheses` -- staged hypotheses with validation status

**Styx publication eligibility:** Validated heuristics with confidence >= 0.7 become eligible for Styx L1 (Clade) promotion during the next Curator cycle. Dream-generated bloodstain insights (those derived from replaying inherited death episodes) receive a 1.2x confidence boost for Styx publication, reflecting their higher-than-average signal quality. Styx Clade sync happens after dream consolidation completes, aligned with the Curator's normal cycle.

> **Cross-ref**: `../20-styx/01-architecture.md` (provenance weights, promotion), `../20-styx/02-api-revenue.md` (dream API endpoints)

---

## Citation Summary

| Citation Key                | Source                                                                                                                     |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| [BORN-WILHELM-2012]         | Born & Wilhelm. "System consolidation of memory during sleep." _Psychological Research_, 2012.                             |
| [HOBSON-FRISTON-2012]       | Hobson & Friston. "Waking and dreaming consciousness." _Progress in Neurobiology_, 2012.                                   |
| [KUMAR-CQL-2020]            | Kumar et al. "Conservative Q-Learning for Offline RL." _NeurIPS_, 2020.                                                    |
| [NORTON-MOCHON-ARIELY-2012] | Norton, Mochon, & Ariely. "The IKEA Effect." _Journal of Consumer Psychology_, 2012.                                       |
| [WALKER-VAN-DER-HELM-2009]  | Walker & van der Helm. "Overnight therapy?" _Psychological Bulletin_, 2009.                                                |
| [SCHMIDHUBER-2009]          | Schmidhuber. "Driven by Compression Progress." _LNAI 5499_, 2009.                                                          |
| [BARTLETT-1932]             | Bartlett. _Remembering._ Cambridge, 1932.                                                                                  |
| [KUMARAN-2016]              | Kumaran, Hassabis, & McClelland. "What Learning Systems do Intelligent Agents Need?" _Trends in Cognitive Sciences_, 2016. |

## Dream Entries in the Grimoire

Insights generated during dream consolidation are written to the Grimoire with special provenance tracking:

**Provenance:** `"dream"` (distinct from `"self"`, `"clade"`, `"replicant"`)

**Confidence floor:** 0.3 — dream-sourced entries are inherently speculative and must be validated by waking experience before being trusted.

**Styx Archive integration:** All dream entries are auto-backed to the Styx Archive. These represent the "what I suspect but can't prove" category -- hypotheses worth preserving for death reflection even if never confirmed.

**Validation lifecycle:**

1. Created at confidence 0.3, provenance `"dream"`
2. Confirmed by waking experience within 7 days → promoted to 0.5, tagged `"dream_validated"`
3. Unconfirmed after 14 days → enters normal temporal decay (no special protection)

**Clade policy:** Dream entries are NOT pushed to Clade siblings unless confirmed (confidence ≥ 0.5). Unvalidated dreams are private speculation.

**DreamMetadata interface:**

```rust
pub struct DreamMetadata {
    pub cycle_id: String,
    pub replayed_episode_ids: Vec<String>,
    pub pad_vector: PadVector,
    pub dream_type: DreamType,
    pub mortality_clock_state: MortalityClockSnapshot,
}

pub struct PadVector {
    pub p: f64,
    pub a: f64,
    pub d: f64,
}

pub enum DreamType {
    Replay,
    Imagination,
    ThreatSim,
    Consolidation,
}

pub struct MortalityClockSnapshot {
    pub projected_life_hours: f64,
    pub prediction_accuracy: f64,
    pub hayflick_ratio: f64,
}
```

> **Cross-ref**: `../04-memory/01-grimoire.md` (entry schema), `../20-styx/01-architecture.md` (backup), `../02-mortality/06-thanatopsis.md` (death reflection)
