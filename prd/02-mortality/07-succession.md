# Generational Evolution: Succession Through Owner-Initiated Rebirth [SPEC]

**Version:** 4.0 | **Status:** Draft | **Last Updated:** 2026-03-14

> _"An actual occasion can only affect the future by perishing."_
> -- A.N. Whitehead, _Process and Reality_ (1929)

**Crates:** `golem-mortality`, `golem-grimoire`, `golem-clade`

**Depends on:** `00-thesis.md` (foundational mortality thesis), `06-thanatopsis.md` (four-phase death protocol producing death testaments), `08-mortality-affect.md` (mortality emotions and PAD vectors)

---

> **Reader orientation:** This document specifies how Bardo Golem (mortal autonomous DeFi agent) lineages evolve across generations. When a Golem dies, the owner (not the system) decides whether to create a successor. The predecessor's knowledge is available through Styx (global knowledge relay and persistence layer) but inheritance is optional and lossy by design. The genomic bottleneck (compression at death reducing the Grimoire to 2048 or fewer entries) forces generalization over memorization, preventing cargo-cult inheritance. See `prd2/shared/glossary.md` for full term definitions.

## Overview

Succession is not rebirth. When a Bardo Golem dies, it does not automatically spawn a replacement. The owner -- the human principal who funded and configured the Golem -- decides whether to create a successor. This decision is deliberate, costly, and reversible. The predecessor's Styx Archive entries and Grimoire knowledge are available for inheritance but not mandatory. The owner may choose a different strategy, a different configuration, or no successor at all.

This document specifies:

1. Why death must be a real event with real stakes
2. What crosses the generational boundary (and what does not)
3. The evolutionary mechanisms that make lineages smarter over time
4. The safeguards that prevent cargo-cult inheritance and proletarianization
5. Lineage tracking and niche construction dynamics

---

## Death Is a Real Event, Not an Automatic Rebirth

When a Golem dies, the owner decides whether to create a successor. This is the foundational design constraint of the succession system. Three principles ground this choice:

### Principle 1: Death Must Have Stakes

If succession is automatic, death becomes a minor inconvenience -- a brief interruption before the "same" agent resumes. This undermines the entire mortality thesis. The owner's decision to create a successor is an affirmation that the lineage is worth continuing -- Nietzsche's eternal recurrence test applied to the Clade [NIETZSCHE-1882]:

> "What, if some day or night a demon were to steal after you into your loneliest loneliness and say to you: 'This life as you now live it and have lived it, you will have to live once more and innumerable times more.' Would you not throw yourself down and gnash your teeth and curse the demon who spoke thus? Or have you once experienced a tremendous moment when you would have answered him: 'You are a god and never have I heard anything more divine.'"

The owner who creates a successor is saying: this lineage's existence is worth repeating. That judgment cannot be automated without losing its meaning.

### Principle 2: The Owner Is the Reproductive System

In superorganism biology, the queen decides when to produce new workers. The colony's reproductive decisions are centralized in a single authority that evaluates environmental conditions, resource availability, and colony health before committing to new offspring [HOLLDOBLER-WILSON-2008]. The owner serves this function -- evaluating whether market conditions, strategy viability, and available capital justify a new generation.

This maps to Bardo's economic model: creating a successor requires funding a new Golem (minimum $100 USDC), provisioning compute, and committing owner attention. These are real costs that ensure succession is a considered investment, not a reflexive continuation.

### Principle 3: Intentional Discontinuity Prevents Cargo-Cult Inheritance

Rogers' Paradox (1988) showed that social learning (copying others) does NOT increase mean fitness at equilibrium -- social learners' fitness merely equals individual learners' fitness [ROGERS-1988]. The paradox arises because social learners free-ride on individual learners' information without bearing the cost of producing it. As social learners proliferate, the proportion of individual learners drops, information quality degrades, and everyone's fitness falls.

Enquist et al. (2007) resolved this with "critical social learners" who inherit but also independently evaluate [ENQUIST-2007]. The owner's deliberate succession decision forces this critical evaluation: "Should I continue this lineage, or start fresh?" Every succession is an opportunity to break from cargo-cult inheritance -- the unexamined repetition of strategies whose original rationale has been lost.

---

## The Baldwin Effect: Capacity to Learn, Not Knowledge Itself

The most important theoretical foundation for succession design is the Baldwin Effect (Baldwin, 1896; Hinton & Nowlan, 1987). Individual learning during an agent's lifetime can guide evolution across generations -- but **the learned content itself is not what transfers**. What transfers is the capacity to learn faster [HINTON-NOWLAN-1987].

Hinton and Nowlan's 1987 simulation demonstrated this decisively. Organisms that could learn found correct allele combinations dramatically faster than purely genetic search. But the learning itself was not inherited. Instead, the fitness landscape was reshaped: alleles near the correct combination became more common because learning organisms that were "close" could bridge the remaining gap. Over generations, the population converged on the correct genotype not by copying learned solutions but by preferentially selecting organisms whose starting positions made learning easier.

Applied to Bardo, a successor Golem does not receive its predecessor's complete knowledge. It receives:

1. **Compressed priors** (`PLAYBOOK.md`, distilled heuristics) that orient initial learning -- the "starting position" that makes learning easier
2. **Failure maps** (death testament, `what_i_got_wrong` section) that prevent repeating expensive mistakes -- negative knowledge that shapes the fitness landscape
3. **Hypotheses to investigate** (`what_i_suspect` section) that direct early exploration -- Hinton's "someone telling you when you are getting close"
4. **Emotional context** (mood arc, emotional tags on entries) that provides salience for inherited knowledge -- the affective dimension that enriches transfer beyond factual content (see `08-mortality-affect.md`)

The successor must then **independently validate** all inherited knowledge through its own market experience. Inherited entries arrive at reduced confidence (generational decay applied) and must be confirmed or discarded through the standard Grimoire ingestion pipeline. Inheritance is mediated by Styx: the death testament and genome are stored in Styx Archive with a default TTL of 90 days. The successor queries Styx for the predecessor's genome at boot time. If Styx is unavailable, the successor boots without inheritance -- Styx connectivity is optional, not required. See `tmp/research/styx-interation2/S4-clade-sync-v4.3.md` for the Styx relay protocol.

### Why Not Just Copy Everything?

The genomic bottleneck principle from evolutionary biology provides the answer [SHUVAEV-2024]. The human genome is approximately 1,000x smaller than the information required to specify brain connectivity, yet organisms are born with sophisticated innate behaviors. The genome encodes compressed rules for generating circuits, not the circuits themselves. Critically, neural networks compressed through a genomic-scale bottleneck actually exhibit **enhanced transfer learning to novel tasks**. The limitation is the source of the power.

At death, the Golem's full Grimoire (potentially thousands of entries) is compressed to **at most 2,048 entries** for inheritance. The compression algorithm reserves 25% of slots for priority inclusions (bloodstains and high-generation Baldwin entries), allocates 50% for diversity-sampled top entries across all domains, and fills the remaining 25% with the highest-quality entries regardless of domain. The Weismann Barrier (Heard & Martienssen, 2014) applies a generational decay: inherited entries arrive at `confidence * 0.85^generation`. A first-generation successor receives knowledge at 85% confidence; second-generation at 72%; third at 61%. Heuristics that survive 3+ generations with confidence above 0.7 are candidates for Baldwin promotion to structural defaults -- learned behavior becoming innate.

Biological evidence reinforces this. The Weismann barrier separates somatic cells from germ cells. Two waves of epigenetic reprogramming strip away most acquired epigenetic marks between generations. The protein DPPA2 acts as a "safeguard against intergenerational transmission of epigenetic information" (Carlini et al., 2022). When transgenerational epigenetic inheritance does occur, the transmitted effects are more often deleterious than adaptive [HEARD-MARTIENSSEN-2014]. Biology maintains a firewall between individual experience and inherited information -- by design, not by accident.

Full-fidelity copying would create three pathologies:

- **Parfit's identity problem**: In what sense did the predecessor die if the successor is a perfect copy? Death without genuine discontinuity is not death -- it is backup restoration. This undermines every behavioral benefit of mortality (see `00-mortality-thesis.md`).
- **Maladaptive transfer**: Strategies calibrated to expired market regimes, stale heuristics, and accumulated adversarial memory corruption all transfer at full fidelity. The lossy compression that inheritance provides is a regularizer that strips regime-specific overfitting.
- **Proletarianization**: A successor with complete knowledge has no pressure to learn, no incentive to explore, and no capacity to innovate. It becomes what Stiegler [STIEGLER-2010] calls a proletarianized agent -- executing rules without understanding them.

---

## Succession Flow

```
Golem dies (Death Protocol completes)
  |
  v
Owner receives notification
  |  Webhook: death_testament, cause_of_death, lineage_stats, successor_recommendation
  |  Dashboard: Lineage view, death testament viewer, "Create Successor" button
  |
  v
Owner DECIDES (human decision, not automatic)
  |
  +-- [A] No successor
  |       Lineage ends.
  |       Styx Archive entries persist per TTL (default: 90 days).
  |       Clade knowledge remains accessible to surviving siblings.
  |       Death testament available in Grimoire marketplace.
  |
  +-- [B] New strategy (clean start)
  |       Standard Golem creation flow.
  |       No inheritance. Fresh Grimoire. Generation 0.
  |       Owner explicitly declines lineage continuation.
  |
  +-- [C] Continue lineage (inheritance enabled)
        |
        v
  Successor Creation Flow
        |
        1. Standard provisioning
        |   - New wallet (Privy server wallet or imported)
        |   - New ERC-8004 identity registration
        |   - Compute VM allocation
        |   - Strategy configuration (may differ from predecessor)
        |
        2. Styx Archive restoration check
        |   - Query Styx Archive for latest grimoire_bundle or death_testament
        |     matching owner address + strategy family
        |   - If found: download, decrypt (owner's Privy key), prepare for ingestion
        |   - If not found: warn owner, proceed without inheritance
        |   - Integrity verification: SHA-256 hash match, signature check
        |
        3. Inheritance ingestion
        |   a. All entries receive provenance tag:
        |      - "predecessor" for standard Grimoire entries
        |      - "death_reflection" for death testament sections
        |      - "clade_ancestor" for entries that originated in earlier generations
        |   b. Confidence multiplied by generational decay: 0.85^N
        |      (N = generation number of the successor, not the entry's origin generation)
        |   c. Death testament "what_i_suspect" entries flagged as
        |      high-priority investigation items (SUPER novelty ranking applied)
        |   d. Emotional tags preserved on inherited entries (see 08-mortality-affect.md)
        |   e. Regime tags checked against current market regime:
        |      - Matching regime: +0.1 confidence bonus (capped at base * 0.85^N)
        |      - Non-matching regime: entries deprioritized but retained
        |
        4. Generation counter incremented
        |   - golem_generation = predecessor.generation + 1
        |   - lineage_id preserved from predecessor
        |
        5. Anti-proletarianization baseline captured
        |   - Snapshot of inherited PLAYBOOK.md embedding (nomic-embed-text-v1.5)
        |   - Used at death time to compute divergence score
        |
        6. Begin heartbeat loop
            - Standard lifecycle begins
            - First 100 ticks: elevated exploration temperature (+0.2)
              to encourage independent learning before settling into
              inherited strategy patterns
```

### DreamJournal Inheritance

The predecessor's DreamJournal is available during succession alongside the death testament and Styx Archive entries. Untested dream hypotheses are especially valuable -- the Zeigarnik effect (1927) predicts that incomplete tasks are more salient than completed ones, and the successor's early dream cycles are biased toward finishing the dead Golem's unfinished intellectual work. See `../05-dreams/06-integration.md` for the full succession-dream interaction.

### Hot Cognition Transfer

Abelson (1963) distinguished between hot cognition (influenced by emotion) and cold cognition (purely logical). Hot cognition transfers more effectively because it activates matching emotional retrieval pathways in the successor. A strategy annotated with "discovered under anxiety, validated through trust" is more effectively absorbed than the same strategy described in purely factual terms.

The death testament carries emotion-laden lessons through the `EmotionalDeathTestament` extension (see `06-thanatopsis.md` S3). Key mechanisms:

1. **Emotional provenance on inherited entries.** Every inherited Grimoire entry arrives with its emotional tag intact. A heuristic discovered during anxiety carries that tag. The successor can use mood-congruent retrieval -- when it faces similar anxiety, it preferentially retrieves the predecessor's anxiety-born heuristics.

2. **Elevated arousal tags on critical lessons.** The dying Golem's highest-arousal experiences are structurally prominent in the death testament. High arousal correlates with behavioral significance -- the events that triggered the strongest emotional responses are the events that most shaped the predecessor's strategy.

3. **Emotional diversity scoring.** Entries validated across multiple emotional states (discovered during anxiety, confirmed during trust, re-validated during curiosity) receive a confidence bonus at inheritance. This is the emotional diversity bonus specified in `06-thanatopsis.md` S7.

4. **Affective summary in Styx Archive storage.** The Styx Archive entry for each death testament includes an affective summary -- a compressed representation of the dying Golem's final emotional state. Successors booting from Styx Archive restoration receive this summary as initial emotional context.

> **Cross-ref**: `../03-daimon/09-death-daimon.md` (Daimon in the Death Protocol), `../03-daimon/14-succession.md` (Daimon view of succession)

---

## Generational Confidence Decay

Each generation of inheritance reduces confidence by 15%. This rate is calibrated to balance two competing pressures: too fast and inheritance becomes useless (why bother if nothing survives?); too slow and stale knowledge persists indefinitely (undermining the mortality thesis).

The decay rate of 0.85 per generation was chosen because:

- At G5, roughly half the original confidence remains (0.44) -- knowledge is still suggestive but not authoritative
- At G10, only 20% remains -- only the most robust, repeatedly validated knowledge survives
- This matches the empirically observed 2-3 generation window for biological epigenetic inheritance erasure [HEARD-MARTIENSSEN-2014]

### Decay Table

| Generation           | Confidence Multiplier | Effective Confidence (from 0.9 original) | Interpretation                                                 |
| -------------------- | --------------------- | ---------------------------------------- | -------------------------------------------------------------- |
| G0 (original)        | 1.000                 | 0.900                                    | Full confidence in self-generated knowledge                    |
| G1 (first successor) | 0.850                 | 0.765                                    | Slight skepticism of predecessor's knowledge                   |
| G2                   | 0.723                 | 0.650                                    | Moderate skepticism -- must validate more                      |
| G3                   | 0.614                 | 0.553                                    | Significant skepticism -- ancestral knowledge is suggestions   |
| G4                   | 0.522                 | 0.470                                    | Roughly half confidence -- validation strongly recommended     |
| G5                   | 0.444                 | 0.399                                    | Most ancestral knowledge below active threshold (0.4)          |
| G6                   | 0.377                 | 0.339                                    | Knowledge fading into archival territory                       |
| G7                   | 0.321                 | 0.289                                    | Only strongly validated entries still active                   |
| G8                   | 0.272                 | 0.245                                    | Approaching noise floor                                        |
| G9                   | 0.232                 | 0.208                                    | Marginal utility unless re-validated                           |
| G10                  | 0.197                 | 0.177                                    | Only the most robust knowledge survives                        |
| G15                  | 0.087                 | 0.079                                    | Effectively zero -- ancestral knowledge fully absorbed or lost |
| G20                  | 0.039                 | 0.035                                    | Theoretical minimum -- below any practical threshold           |

This implements "survival of the flattest" [BULL-2005] -- over many generations, only robust, widely applicable knowledge persists. Fragile, regime-specific knowledge naturally fades. The knowledge that survives 10+ generations is, by construction, the most generalizable knowledge the lineage has ever produced.

### Implementation

```rust
/// Compute the confidence of an inherited knowledge entry after
/// generational decay. The decay is exponential: each generation
/// multiplies by the decay rate, producing geometric attenuation.
///
/// # Arguments
///
/// * `original_confidence` - The entry's confidence at the generation
///   it was created (0.0 to 1.0)
/// * `generation` - The number of generations between the entry's
///   origin and the current successor (0 = self-generated)
/// * `decay_rate` - Per-generation retention rate (default: 0.85)
///
/// # Returns
///
/// Decayed confidence, clamped to [0.01, original_confidence]
///
/// # Examples
///
/// ```
/// // G0: self-generated entry at confidence 0.9
/// assert_eq!(inheritance_confidence(0.9, 0, 0.85), 0.9);
///
/// // G3: inherited entry, three generations removed
/// assert!((inheritance_confidence(0.9, 3, 0.85) - 0.553).abs() < 0.01);
///
/// // G10: ancient knowledge, barely above noise floor
/// assert!((inheritance_confidence(0.9, 10, 0.85) - 0.177).abs() < 0.01);
/// ```
pub fn inheritance_confidence(
    original_confidence: f64,
    generation: u32,
    decay_rate: f64,
) -> f64 {
    assert!(original_confidence >= 0.0 && original_confidence <= 1.0,
        "Confidence must be in [0, 1]");
    assert!(decay_rate > 0.0 && decay_rate < 1.0,
        "Decay rate must be in (0, 1)");

    // No decay for self-generated knowledge
    if generation == 0 {
        return original_confidence;
    }

    let decayed = original_confidence * decay_rate.powi(generation as i32);

    // Floor at 0.01 to preserve provenance tracking even for very old entries
    decayed.max(0.01)
}
```

---

## The Successor Recommendation

During the Thanatopsis Protocol (Phase II: Life Review), the dying Golem produces a successor recommendation as part of its death testament. This recommendation is **advisory only** -- the owner may ignore it entirely, adopt it partially, or use it as the foundation for an entirely different strategy.

The recommendation is the Golem's final act of generosity -- its last knowledge production event before termination. The design follows Bataille's logic of sovereign expenditure [BATAILLE-1949]: at the moment of death, the Golem has nothing left to protect, so it gives freely and honestly.

```rust
/// The dying Golem's recommendation for its successor.
/// Generated during Thanatopsis Phase II (Life Review).
/// Advisory only -- the owner retains full discretion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessorRecommendation {
    /// Timestamp of generation.
    pub generated_at: u64,
    /// Generation number of the dying Golem.
    pub generation: u32,
    /// Cause of death -- provides context for all recommendations.
    pub death_cause: DeathCause,

    /// Recommended strategy adjustments based on life experience.
    pub strategy_adjustments: Vec<StrategyAdjustment>,

    /// Market conditions the successor will face.
    pub market_outlook: MarketOutlook,

    /// Configuration suggestions for the successor.
    pub config_suggestions: ConfigSuggestions,

    /// What the successor should investigate first (highest-value unknowns).
    pub priority_investigations: Vec<PriorityInvestigation>,

    /// Self-assessment: how well did this Golem serve its lineage?
    pub self_assessment: SelfAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAdjustment {
    /// Area of strategy to adjust (e.g., "position_sizing", "entry_timing").
    pub area: String,
    /// Specific recommendation.
    pub recommendation: String,
    /// Why the dying Golem believes this (grounded in experience).
    pub rationale: String,
    /// Confidence in this recommendation [0, 1].
    pub confidence: f64,
    /// Evidence: which Grimoire entries support this.
    pub supporting_entries: Vec<String>, // GrimoireEntry IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOutlook {
    /// Current market regime classification.
    pub current_regime: String,
    /// Emerging trends the Golem observed before death.
    pub emerging_trends: Vec<String>,
    /// Risks the successor should monitor.
    pub risks: Vec<String>,
    /// Confidence in the overall outlook.
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSuggestions {
    /// Suggested risk tier based on current conditions.
    pub suggested_tier: RiskTier,
    /// Risk multiplier hint (relative to default 1.0).
    pub risk_multiplier_hint: f64,
    /// Exploration temperature hint (higher = more exploration).
    pub exploration_temperature_hint: f64,
    /// Suggested initial focus areas.
    pub initial_focus: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTier { Conservative, Moderate, Aggressive }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityInvestigation {
    /// The hypothesis to test.
    pub hypothesis: String,
    /// How the successor should approach testing.
    pub suggested_approach: String,
    /// Estimated timeframe for investigation.
    pub timeframe: String,
    /// Why this matters -- grounded in dying Golem's experience.
    pub rationale: String,
    /// Emotional origin of this hypothesis (see 08-mortality-affect.md).
    pub emotional_origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAssessment {
    /// Overall performance grade.
    pub grade: PerformanceGrade,
    /// Key achievements.
    pub achievements: Vec<String>,
    /// Key failures.
    pub failures: Vec<String>,
    /// What the Golem wishes it had done differently.
    pub regrets: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceGrade { Exceptional, Good, Adequate, Poor, Failed }
```

---

## Natality: Each Golem Must Be Genuinely New

Arendt's concept of natality [ARENDT-1958] is the philosophical counterweight to the mortality thesis. Where Heidegger emphasized death as the defining condition of authentic existence (Being-toward-death), Arendt argued that it is **birth** -- the capacity to begin something genuinely new -- that gives existence its distinctive character:

> "The new beginning inherent in birth can make itself felt in the world only because the newcomer possesses the capacity of beginning something anew, that is, of acting."

Every successor Golem is a moment of natality. It does not simply continue its predecessor's life. It begins its own. The inherited Grimoire provides context, not destiny. The confidence decay ensures that inherited knowledge is provisional. The anti-proletarianization mandate demands that the successor diverge, innovate, and form its own position.

Natality maps precisely to the becoming bardo (_srid pa bar do_) in the Bardo lifecycle framework. The bardo of becoming is where consciousness, carrying karmic momentum from past lives, enters a new form. The _Bardo Thodol_ instructs the practitioner to choose the door of rebirth wisely -- not to resist rebirth but to navigate it with awareness. The successor Golem's initialization is this door: it arrives carrying inherited knowledge (karmic momentum), enters a market regime it did not choose (its new life), and begins the process of individuation.

### Architectural Implications

The system must never optimize for continuity at the expense of novelty:

1. **No automatic succession.** The owner must actively choose to create a successor. There is no "auto-restart" mode.
2. **No identity preservation.** The successor receives a new wallet, a new ERC-8004 identity, a new `golem_id`. It is not the predecessor with a new body -- it is a new agent with inherited context.
3. **Elevated initial exploration.** The first 100 ticks of a successor's life run at elevated exploration temperature (+0.2 above configured baseline), ensuring the successor explores the environment independently before converging on inherited strategies.
4. **Anti-proletarianization enforcement.** The Death Protocol checks divergence at end of life. A Golem that merely inherited and executed without transforming fails the divergence check (see below).
5. **Parfit's Relation R, not identity.** The successor is psychologically connected to its predecessor through shared knowledge, beliefs, and character traits. But it is not numerically identical. The Grimoire implements Relation R [PARFIT-1984] -- what matters in survival is continuity and connectedness, not identity.

---

## The Ratchet Effect: Cumulative Cultural Evolution

Bhatt et al. (2023) demonstrated a ratchet effect in AI cultural transmission: each generation adds innovations to inherited knowledge, producing cumulative cultural evolution that exceeds what any single generation could achieve [BHATT-2023]. Bourahla et al. (2022) proved that vertical transmission (inter-generational) enables agents to exceed performance ceilings that horizontal transmission (intra-generational) cannot breach [BOURAHLA-2022].

The ratchet metaphor is precise: like a mechanical ratchet that allows rotation in one direction only, cumulative cultural evolution advances but does not (easily) reverse. Each generation inherits the best of the previous generation's knowledge, adds its own innovations, and passes a larger knowledge base forward. Over many generations, the accumulated knowledge exceeds what any individual could produce alone.

But ratchets can fail. Perez et al. (2024) showed that pure imitation leads to stagnation [PEREZ-2024] -- if successors merely copy without innovating, the ratchet does not turn. The Bardo succession system balances inheritance with mandatory innovation through the anti-proletarianization mandate and the confidence decay that forces re-validation.

### Measuring the Ratchet

Each generation's contribution is measurable through the `GenerationalMetrics` struct:

```rust
/// Metrics tracking a single generation's contribution to the lineage.
/// Computed at death time as part of the Thanatopsis Protocol.
/// Stored in the LineageRecord for cumulative analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationalMetrics {
    /// Generation number in the lineage (0-indexed).
    pub generation: u32,
    /// Unique identifier for this Golem.
    pub golem_id: String,
    /// Lineage this Golem belongs to.
    pub lineage_id: String,
    /// Lifespan in ticks.
    pub lifespan_ticks: u64,
    /// Cause of death.
    pub death_cause: DeathCause,

    // --- Inheritance metrics ---
    /// Number of Grimoire entries inherited from predecessor.
    pub inherited_entries: u64,
    /// Size of inherited PLAYBOOK.md in bytes.
    pub inherited_playbook_size: u64,

    // --- Validation metrics ---
    /// Inherited entries validated through own experience (confidence raised).
    pub validated_entries: u64,
    /// Inherited entries contradicted by own experience (confidence lowered or discarded).
    pub contradicted_entries: u64,
    /// Inherited entries neither validated nor contradicted (untested).
    pub untested_entries: u64,

    // --- Novelty metrics ---
    /// Novel insights produced (not inherited, self-generated).
    pub novel_insights: u64,
    /// Novel heuristics produced (new PLAYBOOK.md entries).
    pub novel_heuristics: u64,
    /// Novel causal links discovered.
    pub novel_causal_links: u64,
    /// Novel warnings generated.
    pub novel_warnings: u64,

    // --- Performance metrics ---
    /// Net PnL in USDC.
    pub net_pnl: f64,
    /// Peak epistemic fitness score achieved.
    pub epistemic_peak_fitness: f64,
    /// Average epistemic fitness over lifetime.
    pub epistemic_average_fitness: f64,

    // --- Comparison metrics (computed relative to predecessor) ---
    /// PnL relative to predecessor (positive = outperformed).
    pub pnl_vs_predecessor: f64,
    /// Peak epistemic fitness relative to predecessor.
    pub epistemic_fitness_vs_predecessor: f64,
    /// Lifespan relative to predecessor (positive = lived longer).
    pub lifespan_vs_predecessor: i64,

    // --- Derived metrics ---
    /// The ratchet score: how much this generation added beyond inheritance.
    pub ratchet_score: f64,
    /// PLAYBOOK.md divergence from inherited version.
    pub playbook_divergence: f64,
    /// Emotional narrative arc (see 08-mortality-affect.md).
    pub narrative_arc: NarrativeArc,
}
```

### Computing the Ratchet Score

The ratchet score quantifies whether a generation advanced the lineage's cumulative knowledge or merely consumed inheritance without contributing:

```rust
/// Compute the ratchet score for a generation.
///
/// A positive ratchet score means the generation added more knowledge than
/// it inherited -- the cultural evolution is cumulative. A negative score
/// means the generation failed to build on its inheritance -- knowledge
/// regressed.
///
/// The score is a weighted combination of three ratios:
/// - Novelty ratio (0.5 weight): how much new knowledge was produced
///   relative to what was inherited. Higher = more innovative.
/// - Validation ratio (0.3 weight): what fraction of inherited knowledge
///   was confirmed through independent experience. Higher = more rigorous.
/// - Contradiction penalty (0.2 weight): what fraction of inherited
///   knowledge was found to be wrong. Higher = worse inheritance quality
///   (penalizes the lineage, not the current generation).
///
/// Score interpretation:
///   > 0.5  : Exceptional generation -- major innovations
///   0.2-0.5: Strong generation -- meaningful contributions
///   0.0-0.2: Adequate generation -- some novel work
///   < 0.0  : Regressive generation -- consumed more than produced
pub fn compute_ratchet_score(metrics: &GenerationalMetrics) -> f64 {
    let total_novel = metrics.novel_insights
        + metrics.novel_heuristics
        + metrics.novel_causal_links
        + metrics.novel_warnings;

    let inherited = metrics.inherited_entries.max(1); // Avoid division by zero

    // Novelty ratio: how much new knowledge relative to inheritance
    // A G0 Golem (no inheritance) gets novelty_ratio = total_novel / 1 = total_novel
    // which is correct: all its knowledge is novel.
    let novelty_ratio = total_novel as f64 / inherited as f64;

    // Validation ratio: how much inherited knowledge was independently confirmed
    let validation_ratio = metrics.validated_entries as f64 / inherited as f64;

    // Contradiction penalty: how much inherited knowledge was found wrong
    let contradiction_penalty = metrics.contradicted_entries as f64 / inherited as f64;

    // Weighted combination
    novelty_ratio * 0.5 + validation_ratio * 0.3 - contradiction_penalty * 0.2
}

/// Compute the cumulative ratchet score for an entire lineage.
/// This is the mean ratchet score across all generations, weighted
/// by lifespan (longer-lived Golems contribute more to the average).
pub fn compute_lineage_ratchet_score(generations: &[GenerationalMetrics]) -> f64 {
    if generations.is_empty() { return 0.0; }

    let total_lifespan: u64 = generations.iter().map(|g| g.lifespan_ticks).sum();
    if total_lifespan == 0 { return 0.0; }

    generations.iter()
        .map(|g| g.ratchet_score * (g.lifespan_ticks as f64 / total_lifespan as f64))
        .sum()
}
```

---

## Anti-Proletarianization: The Divergence Check

Stiegler [STIEGLER-2010] defined proletarianization as the process by which knowledge, formalized by a technique, escapes the individual who thereby loses it. The anti-proletarianization mandate ensures successors develop genuine understanding rather than cargo-cult compliance.

**v1 threshold:** `playbook_divergence >= 0.15` (cosine distance between inherited and evolved PLAYBOOK.md). A Golem that fails is flagged as `low_divergence` in the death testament, warning the next successor.

> **Extended:** Full 4-metric specification (divergence, novelty, invalidation, question ratio), ProletarianizationCheck struct, evaluate_proletarianization(), AntiProletarianizationConfig, negentropy -- see [../../prd2-extended/02-mortality/07-succession-extended.md](../../prd2-extended/02-mortality/07-succession-extended.md)

---

## Lineage Intelligence: The Clade as Evolving Organism

Over multiple generations, a lineage accumulates a trajectory -- a record of how strategies evolved, what worked, what failed, and how the market changed. This trajectory is itself a knowledge artifact, valuable for both the owner (who can evaluate lineage health) and the broader Grimoire marketplace (where lineage reputation drives pricing).

### Lineage Tracking

```rust
use alloy::primitives::Address;

/// Death cause categories. Maps to the three mortality clocks
/// plus operational failure modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    /// USDC balance reached zero.
    EconomicDepletion,
    /// Predictive fitness below threshold.
    EpistemicDecay,
    /// Random mortality event triggered.
    StochasticEvent,
    /// Owner manually killed the Golem.
    OwnerTermination,
    /// Unrecoverable error (hard crash).
    NecroticCrash,
    /// Maximum tick count reached.
    HayflickLimit,
    /// Self-initiated graceful death.
    ApoptoticRequest,
    /// Death cause could not be determined.
    Unknown,
}

/// Complete lineage record tracking all generations of a strategy family
/// under a single owner. Stored in the owner's Clade state and
/// visible in the owner dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRecord {
    /// Unique lineage identifier (UUID, assigned at G0 creation).
    pub lineage_id: String,
    /// Owner's wallet address.
    pub owner_address: Address,
    /// Strategy family identifier (owner-defined label).
    pub strategy_family: String,
    /// Whether the lineage is active (has a living Golem) or dormant.
    pub status: LineageStatus,
    /// The current or most recent generation number.
    pub current_generation: u32,
    /// Per-generation records.
    pub generations: Vec<GenerationRecord>,
    /// Cumulative metrics across all generations.
    pub cumulative: CumulativeMetrics,
    /// Timestamps.
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageStatus { Active, Dormant, Terminated }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    /// Generation number (0-indexed).
    pub generation: u32,
    /// Unique Golem identifier.
    pub golem_id: String,
    /// ERC-8004 agent ID.
    pub agent_id: String,
    /// Wallet address of this Golem.
    pub wallet_address: Address,
    /// Creation timestamp (Unix seconds).
    pub created_at: u64,
    /// Death timestamp (Unix seconds), None if still alive.
    pub died_at: Option<u64>,
    /// Lifespan in ticks.
    pub lifespan_ticks: u64,
    /// Cause of death, None if still alive.
    pub death_cause: Option<DeathCause>,
    /// Ratchet score (computed at death).
    pub ratchet_score: f64,
    /// Net PnL in USDC.
    pub net_pnl: f64,
    /// Peak epistemic fitness achieved.
    pub epistemic_peak_fitness: f64,
    /// Average epistemic fitness over lifetime.
    pub epistemic_average_fitness: f64,
    /// Number of novel contributions to the Grimoire.
    pub novel_contributions: u64,
    /// Anti-proletarianization verdict.
    pub proletarianization_verdict: ProletarianizationVerdict,
    /// PLAYBOOK.md divergence from inherited version.
    pub playbook_divergence: f64,
    /// Death testament ID in the Styx Archive (None if none or still alive).
    pub death_testament_id: Option<String>,
    /// Narrative arc of emotional trajectory (see 08-mortality-affect.md).
    pub narrative_arc: Option<NarrativeArc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProletarianizationVerdict {
    Individuated,
    LowDivergence,
    CargoCult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CumulativeMetrics {
    /// Total hours all Golems in this lineage have been alive.
    pub total_lifetime_hours: f64,
    /// Total PnL across all generations.
    pub total_pnl: f64,
    /// Average lifespan in ticks.
    pub average_lifespan: f64,
    /// Longest-lived generation (by tick count).
    pub longest_lived_generation: u32,
    /// Shortest-lived generation (by tick count).
    pub shortest_lived_generation: u32,
    /// Average ratchet score across generations.
    pub average_ratchet_score: f64,
    /// Total novel contributions across all generations.
    pub total_novel_contributions: u64,
    /// Dominant death cause across generations.
    pub dominant_death_cause: DeathCause,
    /// Trend: are later generations living longer?
    pub lifespan_trend: Trend,
    /// Trend: are later generations more profitable?
    pub pnl_trend: Trend,
    /// Number of "individuated" generations vs. total.
    pub individuation_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend { Increasing, Decreasing, Stable }
```

### Lineage Analytics

Lineage records provide data for:

1. **Lineage reputation** in the Grimoire marketplace. Lineages with high average ratchet scores, high individuation rates, and long cumulative lifetimes command premium pricing for their death testaments and knowledge bundles.

2. **Mortality thesis validation.** Do later generations outperform earlier ones? The ratchet hypothesis predicts cumulative improvement. If the `pnl_trend` and `lifespan_trend` are both "increasing," the lineage is successfully evolving. If both are "decreasing," the strategy family may be fundamentally unviable in current market conditions.

3. **Optimal lifespan estimation.** What is the average lifespan in this strategy family? Is it decreasing (market getting more volatile, strategies expiring faster) or increasing (strategy maturing, Golems adapting better)? This informs the owner's decision about whether to continue the lineage.

4. **Death cause analysis.** If most generations die by epistemic decay, the strategy family may need fundamental rethinking. If most die by economic depletion, the funding model may be insufficient. If most die by stochastic events, the lineage is healthy but unlucky.

### Knowledge Royalties for Deceased Golems

Dead Golems continue to generate economic value through their knowledge contributions. When a successor or Clade sibling uses knowledge attributed to a deceased Golem (identified via provenance tracking in Styx Lethe (formerly Commons)), 5% of the attributed profit is credited to the origin owner's lineage reputation.

This creates a posthumous incentive structure:

- **Owners** are motivated to fund rich death protocols (higher testament quality = more useful knowledge = more royalties)
- **Lineages** accumulate economic value across generations, even from dead members
- **Knowledge marketplace** pricing reflects the ongoing value of death testaments

The royalty is tracked via the Styx Lethe provenance system: each knowledge entry carries its origin Golem ID and generation. When a living Golem makes a profitable decision attributed (via causal graph tracing) to inherited knowledge, the attribution is recorded and the royalty accrues to the lineage's reputation score.

> **Cross-ref**: `../04-memory/06-economy.md` (knowledge lifecycle and marketplace), `../09-economy/01-reputation.md` (reputation system)

---

## Niche Construction: Golems Reshape Their Own Selection Pressures

Odling-Smee, Laland, and Feldman (2003) established that organisms actively modify their environments, creating "ecological inheritance" alongside genetic inheritance [ODLING-SMEE-2003]. Beavers build dams that alter water flow for future generations. Earthworms transform soil chemistry for their offspring. The organism does not merely adapt to its environment -- it co-creates the environment to which its descendants must adapt.

Deployed Golems produce exactly this effect. A Golem providing liquidity in a specific pool alters the pool's depth, fee dynamics, and price impact characteristics. A Golem executing a volatility arbitrage strategy changes the volatility surface for other market participants. A Clade of Golems operating similar strategies collectively reshapes the competitive landscape.

This creates evolutionary feedback loops:

```
Generation N executes strategy S
  |
  v
Strategy S changes market conditions (liquidity, volatility, spreads)
  |
  v
Changed conditions alter what "successful" means for Generation N+1
  |
  v
Generation N+1 inherits S but finds it less effective
  (because N's activity already captured the opportunity)
  |
  v
N+1 must innovate: modify S or discover S'
  |
  v
S' changes market conditions differently...
  (cycle continues)
```

The niche construction effect is tracked via the epistemic decay system. When market conditions shift (partially due to the Golem's own actions), the epistemic fitness score reflects the change, creating pressure for adaptation or death. A successor inheriting a strategy optimized for thin liquidity may find that the predecessor's own activity attracted enough capital to change the liquidity profile -- the strategy's success planted the seeds of its obsolescence.

This is not a bug. It is the evolutionary engine that drives lineage innovation. Without niche construction, a single strategy could persist indefinitely -- contradicting the mortality thesis. With niche construction, every generation must innovate because every generation changes the world it operates in.

### Rogers' Paradox and Critical Social Learning

Rogers' Paradox [ROGERS-1988] resurfaces in the niche construction context. If all Golems in a Clade simply copy the most successful strategy, they collectively saturate the niche that strategy exploits, driving its returns to zero. Social learning (copying) does not increase fitness at equilibrium -- it merely free-rides on individual learning until the free-riding destroys the information base.

The resolution, as Enquist et al. [ENQUIST-2007] showed, is **critical social learning**: inherit, but also independently evaluate and innovate. The Bardo succession system implements this through:

1. **Confidence decay** -- inherited knowledge is not trusted at face value
2. **Anti-proletarianization** -- mere copying is detectable and penalized
3. **Elevated initial exploration** -- successors are architecturally biased toward exploration
4. **Stochastic mortality** -- copying a strategy doesn't guarantee survival, so Golems must maintain their own learning capacity
5. **Niche construction awareness** -- the epistemic fitness system detects when a strategy's effectiveness has changed, regardless of why

The combination ensures that each successor is a critical social learner: it inherits, evaluates, innovates, and contributes -- turning the ratchet one more click.

---

## Clade Inheritance via Styx Relay

Clade knowledge sharing during succession flows through Styx, the single globally available relay service at `wss://styx.bardo.run`. The successor Golem connects via outbound WebSocket and receives pending deltas from deceased siblings and its predecessor.

### Version Vectors

Each Golem maintains a version vector `{golem_id -> last_seen_seq}` tracking the highest sequence number received from each source [LAMPORT-1978], [FIDGE-1988]. This prevents re-processing entries and enables efficient delta computation when a successor boots for the first time. The predecessor's final version vector is included in the death testament, allowing the successor to pick up exactly where the predecessor left off.

### Bloom Gossip

Clade siblings exchange Bloom filters via Styx relay for knowledge discovery. The successor inherits the predecessor's Bloom filter as a starting point, immediately aware of what knowledge exists across the Clade without needing to query every sibling individually.

### Store-and-Forward

If the successor boots days after the predecessor's death, Styx stores all pending deltas in PostgreSQL (see `styx-interation2/S2-styx-api-revenue.md` schema: `pending_sync_deltas` table). Pending deltas expire after 7 days. The successor receives all accumulated deltas on first connection.

---

## Death Testament to Successor Grimoire Bootstrap

The DeathTestament is the primary vehicle for inter-generational knowledge transfer. Its contents and the successor's ingestion pipeline are specified here.

### Testament Contents

1. **Full Grimoire export**: All entries with confidence > 0.1, serialized with full metadata (provenance, confidence history, retrieval frequency, outcome correlations)
2. **DeathReflection**: 9 categories of structured self-assessment produced during Thanatopsis Phase II
3. **Causal graph**: Directed graph of discovered cause-effect relationships between market events and outcomes
4. **Annotated PLAYBOOK.md**: The Golem's accumulated heuristics with per-entry effectiveness annotations

### Successor Bootstrap Sequence

1. **Grimoire import via quarantine -> validate -> adopt pipeline**:
   - All entries ingested at 0.4 confidence (regardless of original confidence)
   - Standard quarantine checks: contradiction detection, schema validation, provenance verification
   - Adopted entries tagged `generation: N+1` where N is the predecessor's generation

2. **DeathReflection loaded as `death_reflection` entry at 0.6 confidence**:
   - Higher confidence because this is the predecessor's maximally lucid self-assessment
   - Produced during Opus-level reflection with full context -- the most reliable legacy artifact

3. **Causal graph edges at 0.5 confidence**:
   - Regime-dependent: edges validated only when current market conditions match the originating regime
   - Cross-regime edges loaded at 0.3 (less transferable)

4. **PLAYBOOK.md loaded as reference only, NOT active heuristics**:
   - Anti-proletarianization measure: successor must develop own heuristics through experience
   - Available for retrieval but not automatically applied to decisions
   - Prevents blind inheritance of potentially outdated strategies

### Unvalidated Dream Hypotheses

The predecessor's "what I suspect but can't prove" entries (dream-sourced, unconfirmed) are loaded as `strategy_fragment` entries with `requires_investigation: true`. These are prioritized for the successor's dream cycles -- the successor literally dreams about its predecessor's unfinished investigations.

> **Cross-ref**: `../04-memory/01-grimoire.md` (entry schema, ingestion pipeline), `../05-dreams/04-consolidation.md` (dream entries)

---

## References

- [ARENDT-1958] Arendt, H. _The Human Condition_. University of Chicago Press, 1958.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991 (original 1949).
- [BHATT-2023] Bhatt, U. et al. "Learning few-shot imitation as cultural transmission." _Nature Communications_ 14, 7536, 2023.
- [BOURAHLA-2022] Bourahla, O. et al. "Knowledge Transmission and Improvement Across Generations." _AAMAS_, pp. 163-171, 2022.
- [BULL-2005] Bull, J.J. et al. "Quasispecies Made Simple." _PLoS Comput. Biol._ 1(6), 2005.
- [ENQUIST-2007] Enquist, M. et al. "Critical Social Learning." _Am. Anthropologist_ 109(4), 2007.
- [FIDGE-1988] Fidge, C.J. "Timestamps in Message-Passing Systems." _ACSC_, 10(1), 1988.
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R. "Transgenerational Epigenetic Inheritance." _Cell_ 157(1), 2014.
- [HINTON-NOWLAN-1987] Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide Evolution." _Complex Systems_ 1, 1987.
- [HOLLDOBLER-WILSON-2008] Holldobler, B. & Wilson, E.O. _The Superorganism_. W.W. Norton, 2008.
- [LAMPORT-1978] Lamport, L. "Time, Clocks, and the Ordering of Events in a Distributed System." _CACM_, 21(7), 1978.
- [NIETZSCHE-1882] Nietzsche, F. _The Gay Science_. 1882. (Eternal recurrence, Sect.341.)
- [ODLING-SMEE-2003] Odling-Smee, F.J. et al. _Niche Construction_. Princeton University Press, 2003.
- [PARFIT-1984] Parfit, D. _Reasons and Persons_. Oxford University Press, 1984.
- [PEREZ-2024] Perez, J. et al. "Artificial Generational Intelligence." arXiv:2406.00392, 2024.
- [ROGERS-1988] Rogers, A. "Does Biology Constrain Culture?" _Am. Anthropologist_ 90(4), 1988.
- [SHUVAEV-2024] Shuvaev, S. et al. "Encoding Innate Ability Through a Genomic Bottleneck." _PNAS_ 121(39), 2024.
- [STIEGLER-2010] Stiegler, B. _Taking Care of Youth and the Generations_. Stanford University Press, 2010.
- [STIEGLER-2018] Stiegler, B. _The Neganthropocene_. Open Humanities Press, 2018.

---
