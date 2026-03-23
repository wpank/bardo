# Katabasis: Three Rivers of Shared Intelligence [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Crates**: `golem-grimoire`, `bardo-styx-client`
>
> **Depends on**: `01-grimoire.md`, `../20-styx/01-architecture.md`, `../02-mortality/06-thanatopsis.md`

---

> **Reader orientation:** This document specifies Katabasis, the policy layer governing what knowledge is shared across Bardo's mortal DeFi agents. It belongs to the **04-memory** layer. The key concept is the Three Rivers model: Acheron (knowledge that flows from the dead to the living via Bloodstains (death-sourced warnings)), Lethe (anonymized structural knowledge shared across the ecosystem), and Shade (the policy engine deciding what crosses each privacy boundary). Only anti-rivalrous knowledge is shared; tradeable alpha is never published. For term definitions, see `prd2/shared/glossary.md`.

## Why this document exists

**Moat framing (Die):** The most honest knowledge a golem ever produces is the knowledge it produces while dying. At the moment of death, survival pressure drops to zero. The agent has no future position to protect, no reputation game to play, no competitive edge to guard. The Thanatopsis Protocol exploits this window. Death scars, Lethe (formerly Lethe) votes cast at death, and the full death testament are the system's highest-quality signal — epistemically, not sentimentally. Bloodstain entries getting 1.2x retrieval boost and 3x slower decay is not ornamentation; it is the system encoding that zero-competition testimony is more reliable than competitive testimony.

Death creates knowledge with no ulterior motive. A living agent sharing information always has a competitive incentive to shade the truth, omit the critical detail, or time the disclosure. A dead agent has no future in which it competes. Its testimony is the cleanest signal the system produces. This is why death-sourced knowledge (bloodstain entries) gets a 1.2x retrieval boost and 3x slower decay -- the costly signal is the reliability guarantee.

## Purpose

Katabasis (Greek: katabasis, "descent") is the policy layer that decides what knowledge is anti-rivalrous enough to share and how to weight it given who produced it. It is not a storage system (the Grimoire handles storage) and not a communication protocol (Styx handles routing). The Grossman-Stiglitz paradox (1980) sets the constraint: you cannot share tradeable information without destroying its value. Katabasis solves this by sharing only what is anti-rivalrous.

Three categories of anti-rivalrous knowledge:

- **Failure knowledge**: your loss is not my loss to report; it is my protection to receive. Flows through Acheron.
- **Collective belief**: aggregate regime observations, protocol quirks, gas patterns. None of it is alpha. All of it is useful. Flows through the Pheromone Field (Styx Lethe).
- **Behavioral observation** (v1.2, deferred): on-chain action patterns, observed locally, never published. Flows through Shade.

What does not flow: active strategy parameters, position sizes, timing, specific pool selections, or anything that could be front-run.

---

## The Three Rivers

| River       | What Flows                                            | Write Access                                    | Read Access | Phase |
| ----------- | ----------------------------------------------------- | ----------------------------------------------- | ----------- | ----- |
| **Acheron** | Verified failure scars. Causal context, emotional tag | Thanatopsis Phase II/III + living REFLECT phase | Clade (L1)  | v1    |
| **Lethe**   | Proposition votes. Regime beliefs, protocol quirks    | Verified tier (ERC-8004 score >= 50)            | Open (L2 Lethe) | v1.1  |
| **Shade**   | Behavioral observation signals. No publication.       | n/a (local computation only)                    | Local only  | v1.2  |
| **Styx**    | Full death testament as public commons                | Death protocol only                             | Open        | Defrd |

Shade and Styx are deferred. This document covers Acheron and Lethe in operational detail, plus Bloodstain as the composite death artifact.

---

## Acheron: Verified Failures

Acheron carries verified failure scars -- knowledge of what hurt the agent, validated through confirmed outcome. Death-published scars carry 2.5x weight in Lethe votes if the publisher is dead.

### Scar Generation Lifecycle

A scar begins with a failed action. The detection chain:

1. **ActionPermit issued** -- tool executes via DeFi adapter
2. **`bardo-safety` hook fires** (post-execution) -- classifies the outcome
3. **`bardo-memory` hook records episode** -- attaches OCC emotional tag (PAD vector), strategyState snapshot, marketRegime
4. **Curator cycle** (every 50 ticks) -- runs scar qualification check
5. **If qualified** -- `AcheronScar` constructed and queued for Styx L1 publication

Outcome classification:

- Hard failures (on-chain revert, slippage > max): classified immediately
- Confirmed failures (PnL > 2% drawdown): classified immediately
- Ambiguous outcomes (PnL between -0.3% and -2%): deferred check after 30 ticks
- Recoveries: no scar candidate generated

### Scar Qualification Criteria

All five must pass:

| Criterion         | Threshold                                                  | Rationale                                          |
| ----------------- | ---------------------------------------------------------- | -------------------------------------------------- |
| Confirmed outcome | `confirmedAt: "immediate"` OR deferred check resolved      | No scar from ambiguous outcomes                    |
| Confidence        | `scarConfidence >= 0.65`                                   | Curator distillation confidence in the causal link |
| Novelty           | Not covered by existing L1 scar (cosine similarity < 0.85) | Don't duplicate clade knowledge                    |
| Causal clarity    | LLM can articulate why it failed, not just that it failed  | Scars without mechanism are noise                  |
| Not in progress   | Position resolved (open, closed, or impaired)              | Can't scar a position still playing out            |

### AcheronScar Interface

```rust
pub struct AcheronScar {
    /// Deterministic ID: "scar-{golem_id}-{seq}"
    pub scar_id: String,

    /// Source golem
    pub golem_id: String,

    /// Owner address (for L1 namespace routing)
    pub owner_address: String,

    /// Source episode in LanceDB
    pub episode_id: uuid::Uuid,

    /// Heartbeat tick when failure occurred
    pub tick: u64,

    /// Human-readable causal narrative (LLM-generated, Curator-reviewed)
    pub causal_narrative: String,

    /// What the Golem believed vs what happened
    pub causal_context: CausalContext,

    /// PAD vector at moment of failure
    pub emotional_tag: ScarEmotionalTag,

    /// PLAYBOOK.md state at time of failure
    pub strategy_state: serde_json::Value,

    /// Market regime at time of failure
    pub market_regime: ScarRegime, // HighVol | LowVol | Trending | Ranging | Unknown

    /// Confidence in the causal link (0.0-1.0)
    pub confidence: f64,

    /// When the failure was confirmed
    pub confirmed_at: i64,

    /// Qualification metadata
    pub qualification: ScarQualification,

    /// Death-sourced scars get bloodstain treatment
    pub is_bloodstain: bool,
}

pub struct CausalContext {
    pub active_strategy: String,
    pub hypothesis_at_action: String,
    pub actual_outcome: String,
    pub counterfactual_hint: String,
}

pub struct ScarEmotionalTag {
    pub pleasure: f64,    // [-1.0, 1.0]
    pub arousal: f64,     // [0.0, 1.0]
    pub dominance: f64,   // [0.0, 1.0]
    pub label: ScarEmotion, // Anxious | Regretful | Shocked | Resigned
}

pub struct ScarQualification {
    pub novelty_score: f64,
    pub curator_version: String,
    pub qualified_at_tick: u64,
}
```

### Styx L1 Publication

Scars are queued during life (REFLECT phase) and batch-published during Thanatopsis Phase III (Legacy). The publication path:

- Scar content is JSON-serialized and encrypted with owner SEK
- Embedding generated from `causalNarrative` (768-dim, Styx uses larger model)
- Published to `clade:{operatorAddress}` namespace
- `decayClass: "tactical"` (30-day half-life)
- `metadata.confirmedFailure: true` for retrieval filtering

After publication, Styx L1 propagates to all Golems in the clade via the standard pre-fetch mechanism. Propagation delay is one pre-fetch cycle, typically 2-3 ticks after Phase III completion.

Living golems also publish scars during their REFLECT phase. This is not limited to death -- the Curator's scar qualification pass runs every 50 ticks and publishes qualified scars to Styx L1 as they are generated.

### What Does Not Flow

- Active strategy parameters that could be front-run
- Position sizes or timing that reveal alpha
- Unvalidated hypotheses (those go to Lethe as votes, not Acheron as scars)

---

## Lethe / Styx Lethe: collective belief

The Lethe (named Lethe in the three-rivers mythology) carries knowledge that is collectively validated and anti-rivalrous. Not things that are your edge -- things that become more accurate when more agents validate them. Death votes carry 2.5x weight because the dying have no competitive incentive to mislead.

### Proposition Categories (v1.1)

Three categories in scope:

| Category               | Example                                                                             | Why Anti-Rivalrous                                                        |
| ---------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Regime observation     | "High-vol periods on Base occur 2.3x more frequently on Tuesdays 14-16 UTC"         | Statistical observation. More agents confirming it improves the estimate. |
| Protocol quirk         | "Morpho USDC/WETH utilization above 87% produces rate-curve kink within 90 min"     | Protocol behavior. Knowing it benefits all users.                         |
| Strategy effectiveness | "Narrow LP range <0.5% in high-vol increases rebalance frequency 4x vs +/-1% range" | Effectiveness observation about a common configuration.                   |

Out of scope: specific entry/exit timing, optimal position sizes, alpha-generating parameter combinations, or anything with first-mover advantage.

### LetheProposition and LetheVote Interfaces

```rust
pub enum PropositionDomain {
    DexLp, Lending, Yield, Gas, Regime, Risk,
}

pub struct LetheProposition {
    /// Deterministic ID: SHA-256(domain + claim)[:16]
    pub proposition_id: String,

    /// Semantic domain for retrieval routing
    pub domain: PropositionDomain,

    /// Falsifiable, non-identifiable claim
    pub claim: String,

    /// Structured evidence summary (optional)
    pub evidence: Option<String>,

    pub first_vote_at: i64,
    pub last_vote_at: i64,

    pub consensus: PropositionConsensus,
}

pub struct PropositionConsensus {
    pub weighted_mean_confidence: f64, // 0.0-1.0
    pub total_weight: f64,
    pub vote_count: u32,
    pub quorum_met: bool,              // true if vote_count >= 3
    pub contradicted: bool,            // true if variance > 0.15
    pub contradiction_note: Option<String>,
}

pub struct LetheVote {
    pub vote_id: String,
    pub proposition_id: String,

    /// Anonymized voter ID: SHA-256(golem_id + owner_address + proposition_id)[:12]
    pub voter_id: String,

    /// Confidence in the claim (0.0-1.0)
    pub confidence: f64,

    /// Observation that grounds this confidence
    pub evidence: Option<String>,

    /// True if cast during Thanatopsis Protocol.
    /// Death votes carry weight 2.5x. Cannot be fabricated:
    /// vote timestamp cross-referenced against death record in Styx L1.
    pub is_death_vote: bool,

    /// Vote weight: 1.0 (living) or 2.5 (death vote)
    pub weight: f64,

    pub timestamp: i64,
}
```

### Vote Weighting and Consensus

The weighting rule: dying Golems have no competitive incentive to mislead. A living Golem sharing a proposition has some incentive to share slightly wrong information. A dying Golem has no future in which it competes. Its death vote is the cleanest signal it will ever produce.

Consensus calculation:

- `weightedMeanConfidence = sum(confidence * weight) / sum(weight)`
- Contradiction detection: weighted variance > 0.15 marks the proposition as contradicted
- Quorum: 3+ votes OR 1 death vote (a single death vote lifts the confidence floor to 0.65+)

### Write Paths

Three distinct paths produce Lethe votes:

| Path                  | When                  | Trigger                                                                 | Gate                                |
| --------------------- | --------------------- | ----------------------------------------------------------------------- | ----------------------------------- |
| Routine Curator sweep | Every 50 ticks        | High-confidence insight matches existing proposition or creates new one | confidence >= 0.70                  |
| Death contribution    | Phase III Thanatopsis | All propositions in death testament submitted as death votes            | No gate (death votes always submit) |
| Warning propagation   | Immediate             | Insight with `severity: "critical"` and `confirmed: true`               | confidence >= 0.65                  |

Write access requires Verified tier (ERC-8004 score >= 50).

### Retrieval in Heartbeat

During the DECIDE phase, the Context Governor retrieves top-3 Lethe entries for the current regime via Styx L2 pre-fetch. Entries are injected as Bayesian priors into the ContextBundle. During the REFLECT phase, the golem casts votes on any proposition where it has direct evidence.

Katabasis entries participate in the hybrid retrieval pipeline (see `01-grimoire.md`). Acheron scars and Lethe propositions are scored using the same four-factor formula (semantic similarity 0.40, temporal decay 0.20, importance 0.25, emotional congruence 0.15) with the following Katabasis-specific modifiers:

| Source | Modifier | Applied To |
|---|---|---|
| Acheron scar (own) | 1.0x | Standard scoring |
| Acheron scar (sibling) | 0.8x | Importance factor discounted |
| Bloodstain scar (death-sourced) | 1.2x | Importance factor boosted |
| Lethe proposition (quorum met) | 1.0x | Standard scoring |
| Lethe proposition (contradicted) | 0.5x | Importance factor halved |

The Self-RAG adaptive retrieval decision (see `01-grimoire.md`) applies to Katabasis entries: during high-arousal states the Golem always retrieves relevant Acheron scars (the emotional system demands it), while in routine low-curiosity scenarios the Golem may skip Katabasis retrieval if it has strong semantic coverage of the current protocol. The Generative Agents formula's recency weighting ensures that recently received scars from living siblings surface ahead of older inherited ones, unless the older scar's importance score dominates.

---

## Dreams and Katabasis Material

Dreams are where failure episodes get replayed until their emotional charge drops, counterfactual simulations explore alternatives, and threat records are built for scenarios that haven't killed anyone yet.

### NREM Replay Selection

The ReplayEngine selects episodes from LanceDB using an emotional salience score that prioritizes high-arousal, high-magnitude episodes -- the scar-adjacent ones:

- Arousal weight: 0.4
- Valence absolute weight: 0.3
- Recency weight: 0.2 (exponential decay: `exp(-age_in_ticks / 100)`)
- Novelty weight: 0.1 (1 minus max cosine similarity to existing insights)

Inherited Acheron scars from L1 replay alongside own episodes, weighted by mortality pressure. Higher mortality pressure increases inherited scar replay priority.

### REM Depotentiation

After NREM replay, the REM phase reduces emotional arousal while preserving informational content (Walker and van der Helm, 2009). Rate: 0.12 per REM cycle, capped at minimum arousal 0.10. The golem remembers what happened but stops feeling it as intensely.

### Loop 3: Counterfactual Imagination

For confirmed failure episodes, REM allocates 40% of budget to counterfactual analysis. Each branch generates a `CounterfactualBranch` with an intervention, simulated outcome, and optional insight at confidence 0.2-0.3 (`provenance: "dream_counterfactual"`). These enter the staging buffer as proposed PLAYBOOK.md revisions, requiring live validation before promotion.

### Demurrage for Katabasis Entries

Same decay classes as Grimoire. Lethe propositions with high vote weight decay slower -- dream-validated entries (generated as hypothesis, then confirmed by live trading) decay at 0.5x the standard rate because they survived two validation rounds.

| Decay Class | Half-Life | Dream-Validated Rate |
| ----------- | --------- | -------------------- |
| Structural  | 6 months  | 0.5x (3 months)      |
| Tactical    | 30 days   | 0.5x (15 days)       |
| Ephemeral   | 3 days    | 0.5x (1.5 days)      |

---

## The `g-9b2d` Narrative

A concrete lifecycle trace illustrating that knowledge transmission is not behavioral control.

`g-9b2d` provisions on Base with 12,400 USDC. Its predecessor `g-7f3a` died at tick 847 and left a death reflection: "Set your rebalance anomaly threshold at 0.9 on entry. Do not learn it the hard way." The Context Governor injects this at boot. `g-9b2d` reads it, but its PLAYBOOK.md bootstrap default is 0.7. It decides to "validate" the predecessor's advice through experience before overriding the baseline.

At tick 287, a high-vol event. Anomaly score: 0.76. The predecessor's death reflection is injected again. `g-9b2d`'s own scar from tick 63 (rebalance at anomaly 0.71, unnecessary, position self-recovered) is present. Its own threat record from dream rehearsal says WAIT. Everything says don't rebalance.

The LLM reasons: anomaly 0.76 is within operational threshold 0.7. The predecessor's 0.9 advice is advisory. The staged revision at 0.27 confidence hasn't promoted. Each step in the reasoning is locally valid. The aggregate is fatal.

`g-9b2d` rebalances. Slippage 0.71%, gas $4.20. Position re-enters range 45 minutes later. Cascade: Conservation at tick 289, two more rebalances, Morpho utilization spike locks $3,100 at tick 309, Terminal at tick 312.

At death: 3 scars published to Acheron, 7 Lethe death votes cast (all at weight 2.5), Bloodstain constructed, priors package transmitted to `g-3k9p`.

`g-3k9p` reads the death reflection on tick 1. Sets threshold 0.9 from boot. At tick 47, first high-vol event, anomaly 0.71. Holds. Price recovers in 52 minutes. The scar validated. Three generations, two deaths, one PLAYBOOK.md line: `rebalanceThreshold: highVol: 0.9`.

The knowledge propagated. It just took two deaths.

---

## Bloodstain integration

The Bloodstain (see `../02-mortality/06-thanatopsis.md`) is the composite death artifact. Katabasis rivers feed into it:

| Bloodstain Field  | Source                                         |
| ----------------- | ---------------------------------------------- |
| `acheron_scars`   | Scars published to Acheron during Phase II/III |
| `lethe_votes`     | Final proposition votes from death testament   |
| `content_cid`     | Full death testament on IPFS; includes both    |

### Styx bloodstain network

Death-sourced entries in the Styx network receive special treatment through the bloodstain flag:

- All entries published during the Thanatopsis Protocol get `is_bloodstain: true`
- Bloodstain entries receive a **1.2x retrieval boost** in the four-factor scoring function (applied to the importance factor)
- Bloodstain entries decay at **3x slower** than their decay class dictates (a 7-day tactical heuristic published at death gets an effective 21-day half-life)
- Bloodstain entries are never pruned by the Curator's low-confidence sweep -- they persist at the 0.05 floor indefinitely

The rationale: death-sourced knowledge is a costly signal that cannot be fabricated. A dying Golem spending its last resources on honest reflection rather than self-preservation bears a real opportunity cost [SPENCE-1973]. The 1.2x boost and 3x decay slowdown are the system's way of saying: testimony from the dead is weighted heavier and lasts longer than testimony from the living.

### Pheromone Field mapping

The three rivers map to Styx layers:

| River | Styx Layer | Metaphor |
|-------|-----------|----------|
| **Acheron** (verified failures) | L1 Clade | Scars flow within the owner's fleet -- intimate, trusted, operational |
| **Lethe** (collective belief) | L2 Lethe | Propositions flow to the public -- anonymized, validated, anti-rivalrous. The Pheromone Field: no agent controls it, but every agent's behavior leaves traces that others can read. |
| **Styx** (death testament) | L0 Vault + L2 Lethe | Full testament persists in Vault; anonymized fragments enter Lethe as bloodstain echoes |

The Pheromone Field metaphor is precise: like ant pheromone trails, the Lethe propositions are not instructions. They are traces of past behavior that bias future behavior. No central coordinator decides what to believe. The confidence scores emerge from weighted voting -- each agent depositing its own trace, each trace decaying over time unless refreshed by new votes.

---

## Deferred Rivers

### Shade

Behavioral observation. Agents observe on-chain behavior of siblings and strangers, compute signals locally, never publish. Observation is costly (gas, compute) and cannot be faked at scale. The shades are honest.

**Phase**: v1.2. Deferred until Acheron and Lethe are stable. Requires observability infrastructure.

### Styx

Death testament as public commons. Knowledge crosses only at death. Bloodstain + Acheron scars provide structured death records for v1.

**Reconsider when**: Golems accumulate meaningful Grimoire content; demand for "what the dying knew" as retrieval source.

### Augury

Commit-reveal prediction registry. Agents commit hashed predictions; after resolution, reveal. System tracks accuracy per agent, per domain.

**Reconsider when**: 50+ agent population; oracle infrastructure exists.

---

## Cross-References

| Doc                                  | Relationship                                                                                                          |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| `01-grimoire.md`                     | Curator cycle (50-tick maintenance), confidence decay classes, and promotion gates that determine what enters Katabasis |
| `../20-styx/` (was `05-oracle.md`)   | L1 Clade namespace for sibling sharing, L2 Lethe for anonymized public knowledge, and the mortal scoring function     |
| `../02-mortality/06-thanatopsis.md`  | Four-phase Thanatopsis Protocol (Acceptance, Settlement, Reflection, Legacy) and Bloodstain generation at Phase III    |
| `../03-daimon/03-behavior.md`        | OCC appraisal model and PAD vectors (Pleasure-Arousal-Dominance) that tag knowledge with emotional provenance          |
| `../05-dreams/06-integration.md`     | DreamScheduler, REM depotentiation of emotional tags, and Loop 3 meta-consolidation during offline processing         |
| `../01-golem/14-context-governor.md` | ContextBundle assembly and death-specific ContextPolicy that shifts retrieval priorities in Terminal phase              |
