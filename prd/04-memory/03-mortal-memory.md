# Mortal Memory: Knowledge Under Finite Horizons [SPEC]

> **Version**: 3.0 | **Status**: Draft
>
> **Crate**: `golem-grimoire`
>
> **Depends on**: `../02-mortality/01-architecture.md`, `../02-mortality/06-thanatopsis.md`, `00-overview.md`, `01-grimoire.md`, `02-emotional-memory.md`

---

> **Reader orientation:** This document describes how mortality transforms memory management for Bardo's Golems (mortal autonomous DeFi agents). It belongs to the **04-memory** layer. The key concept is that three death clocks (Economic/USDC depletion, Epistemic/predictive fitness decline, Stochastic/per-tick hazard) interact with every Grimoire (local knowledge base) operation, shifting the Golem's strategy from exploration to consolidation to legacy as the horizon contracts. For term definitions, see `prd2/shared/glossary.md`.

## Thesis: Mortality Makes Memory Meaningful

An immortal agent has no reason to remember well. It can always re-learn. It can always run another experiment. It can always wait for the next cycle. Time is free, so memory is optional -- a convenience, not a necessity. The result is Funes the Memorious: an agent drowning in undifferentiated experience, incapable of the triage that produces wisdom [BORGES-1942].

A mortal agent cannot afford this luxury. Every tick spent learning is a tick closer to death. Every insight that fails to generalize is wasted life-hours. Every heuristic that goes unshared is knowledge that dies with the knower. Mortality imposes a cost function on memory: the cost of remembering the wrong thing is not just storage overhead -- it is lifespan consumed. The cost of forgetting the right thing is not just a gap in the record -- it is a lesson that must be re-learned by a successor, at the price of its own mortality.

This document describes how the three death clocks -- Economic (USDC depletion), Epistemic (predictive fitness < senescence threshold 0.35), Stochastic (per-tick hazard rate) -- interact with every layer of the Grimoire architecture. The question is not "how does a Golem remember?" (answered in `01-grimoire.md`) or "how do emotions shape memory?" (answered in `02-emotional-memory.md`) but "how does knowing-you-will-die change what and how you remember?"

The answer, drawn from existential philosophy and cognitive science alike, is that mortality transforms memory from a passive archive into an active economy. The Golem does not merely store knowledge; it invests in knowledge, with the currency of its remaining life. And as the horizon contracts, the investment calculus changes -- from exploration to consolidation to crystallization to legacy.

---

## Three Death Clocks and Their Memory Effects

Every Golem faces three independent mortality constraints. Each imposes a distinct pressure on the memory system. The Golem does not know which clock will fire first. That uncertainty is itself a driver of memory behavior -- the Golem must hedge its knowledge investments across all three mortality modes.

### The USDC Clock: Resource Mortality

**Trigger**: `remainingUsdc < deathReserve + $0.50`

The USDC clock is the most intuitive mortality. The Golem has a finite budget. Every inference call, every on-chain transaction, every data query draws from that budget. When the budget approaches the Apoptotic Reserve, the Death Protocol initiates.

**Memory effects of resource scarcity:**

Resource scarcity transforms the Golem's relationship with its own learning pipeline. When credits are abundant (thriving phase), the Reflexion pipeline runs at full fidelity -- Opus counterfactuals, exhaustive ground-truth backchecks, leisurely ExpeL distillation. When credits contract, every stage of the pipeline faces a cost-benefit test:

- **Is this reflection worth $0.003?** In the thriving phase, always yes. In the conservation phase, only if the outcome was surprising (deviation from prediction > 2 standard deviations). In the declining phase, never -- the Golem lives on cached knowledge exclusively.

- **Is this Curator cycle worth running?** The Curator consolidates every 50 ticks at a cost of ~$0.01 (Haiku). In the stable phase, the Curator begins skipping cycles when no new insights have accumulated -- why consolidate nothing? In the conservation phase, the Curator runs at half frequency (every 100 ticks). In the declining phase, the Curator stops entirely. The PLAYBOOK.md freezes.

- **Is this episode worth encoding?** In the thriving phase, every tick produces an episode. In the conservation phase, only ticks that reached the DECIDING state (System 2 was invoked) produce full episodes. Routine SENSING ticks produce no episode at all -- the Golem stops remembering its routine. In the declining phase, even DECIDING ticks produce only minimal episodes (action + outcome, no context).

The resource clock creates a progressive amnesia. The Golem remembers less and less as it approaches death, not because its storage is full, but because encoding is too expensive. The knowledge that survives this triage is, by selection, the knowledge that mattered most.

Hans Jonas's "needful freedom" [JONAS-1966] is the philosophical frame: the Golem's freedom to learn is simultaneously its compulsion to spend. Every act of encoding is an act of metabolic expenditure. To remember is to consume. To consume is to approach death. The organism's "can" is a "must."

### The Epistemic Clock: Informational Mortality

**Trigger**: `epistemicFitness < senescenceThreshold (default: 0.35) over trailing window`

The epistemic clock is the subtler mortality. Even a Golem that earns indefinitely -- one that has achieved metabolic homeostasis, Jonas's organism in dynamic equilibrium -- must eventually die. The reason is informational: DeFi strategies have information half-lives. Protocols upgrade. Liquidity conditions shift. New instruments emerge. MEV strategies evolve. A Golem trained on February's market is progressively less competent in April's market, even if it has infinite funds.

**Memory effects of informational mortality:**

The epistemic clock changes what the Golem considers worth remembering. Specifically, it privileges _structural_ knowledge over _tactical_ knowledge:

- **Structural knowledge** (protocol mechanics, fee structures, token economics) retains value across the epistemic boundary. It will be useful to the successor. The Golem invests proportionally more in encoding, validating, and promoting structural insights as epistemic fitness declines.

- **Tactical knowledge** (gas timing patterns, specific liquidity depths, arbitrage routes) has a half-life shorter than the Golem's remaining life. As epistemic fitness approaches the senescence threshold, the Golem begins to discount tactical knowledge in its learning pipeline. New tactical insights are still recorded, but the Curator applies an `epistemicAgeDiscount` factor that reduces their promotion probability:

```rust
fn epistemic_age_discount(
    epistemic_fitness: f64,
    senescence_threshold: f64,
    decay_class: &DecayClass,
) -> f64 {
    if matches!(decay_class, DecayClass::Structural) { return 1.0; } // No discount
    if epistemic_fitness > senescence_threshold * 2.0 { return 1.0; } // Healthy

    // Linear discount as fitness approaches senescence threshold
    let fitness_ratio = epistemic_fitness / (senescence_threshold * 2.0);
    0.2 + fitness_ratio * 0.8
}
```

The epistemic clock also triggers a shift in the Golem's meta-learning (Loop 3). When epistemic fitness enters decline, the Golem begins to evaluate its own learning not by "will this make me better?" but by "will this make my successor better?" This is a fundamental reorientation of the learning objective -- from self-improvement to lineage improvement. The mental models that get selected for deeper analysis are no longer the ones that optimize the current portfolio but the ones that generalize across market conditions.

Biologically, cells that bypass senescence checkpoints become cancerous -- they replicate without differentiation, consuming resources without producing value. Epistemic decay makes death emergent from the information environment -- volatile regimes kill faster through rapid knowledge invalidation, while stable regimes extend life. The epistemic fitness threshold prevents the equivalent pathology: a stale agent hoarding compute cycles and knowledge bandwidth while producing diminishing returns for the ecosystem.

### The Staleness Trigger: Epistemic Mortality

**Trigger**: `predictionAccuracy < 0.30` over trailing 500-tick window

The staleness trigger is the most brutal mortality. It does not wait for resources to run out or ticks to accumulate. It measures whether the Golem's knowledge is actively wrong. A prediction accuracy below 30% over 500 ticks means the Golem's model of the world is worse than random. Its knowledge is not merely outdated -- it is harmful. A coin flip would outperform.

**Memory effects of epistemic death:**

When the staleness trigger fires, the Golem's relationship with its own Grimoire inverts. The knowledge base that was its most valuable asset is now its most dangerous liability. The entries it trusts most -- the highest-confidence heuristics, the most-validated causal links -- are the ones most likely to be wrong, because they are the ones driving the predictions that are failing.

The Death Protocol under staleness trigger includes a unique memory operation: **epistemic autopsy**. Before the death reflection, the Golem runs a systematic analysis of which Grimoire entries contributed to the predictions that failed. This produces a ranked list of "toxic knowledge" -- entries that the successor must treat with extreme suspicion.

```rust
pub struct EpistemicAutopsy {
    /// Entries most correlated with failed predictions
    pub toxic_entries: Vec<ToxicEntry>,

    /// Regime at which the model broke
    pub failure_regime: String,

    /// Last regime where predictions were accurate
    pub last_valid_regime: String,

    /// Hypothesis about what changed
    pub regime_transition_hypothesis: Option<String>,
}

pub struct ToxicEntry {
    pub entry_id: uuid::Uuid,
    pub contribution_to_failure: f64,  // 0.0-1.0
    pub confidence_at_failure: f64,
    pub recommendation: ToxicRecommendation, // Discard | Invert | Reinvestigate
}
```

The epistemic autopsy is the most valuable artifact a staleness-triggered death produces. It tells the successor not just what the predecessor knew, but where the predecessor's knowledge broke. Categories 3 and 4 of the death reflection ("what I got wrong" and "what confused me") are dominated by the autopsy results.

This connects to the Zeigarnik effect [ZEIGARNIK-1927]: incomplete tasks are remembered better than completed ones. The unsolved puzzles of the epistemic autopsy -- the "what confused me" entries -- exert a stronger pull on the successor's attention than the validated insights. The successor is drawn to investigate what killed its predecessor, which is precisely where the ecosystem's learning frontier lies.

---

## Memory Behavior Across Behavioral Phases

The five behavioral phases (see `../02-mortality/01-architecture.md`) each produce a characteristic memory regime. The transitions are continuous (sigmoid), not stepped -- but each named region has a dominant memory pattern.

### Thriving: Exploration and Accumulation

**Survival pressure > 0.7 | Projected life > 7 days**

Memory is expansive. The Golem encodes everything. Every tick produces a full episode. The Reflexion pipeline runs at maximum fidelity with Opus escalation for anomalies. The ExpeL distillation runs its full cross-episode analysis. The Curator promotes aggressively -- it is cheap to maintain a large knowledge base when resources are abundant.

The thriving Golem's memory has an exploration bias. It actively seeks novel patterns, runs Replicant experiments, and encodes the results even when they are negative. Failed experiments are valuable because the budget to absorb them exists. The mental models library is fully available; the Golem samples broadly from its 700-model repertoire.

**Characteristic knowledge artifacts**: Rich episodic traces, broadly sampled insights, experimental results from Replicants, mental model effectiveness scores.

**Reminiscence bump effect** [RUBIN-GROTH-GOLDSMITH-1986]: The Golem's first ~1,000 ticks produce disproportionately influential knowledge. This is the computational equivalent of the reminiscence bump -- the disproportionate recall of events from the formative period (ages 10-30 in humans). The first 1,000 ticks are when the initial PLAYBOOK.md is formed, when the Golem's baseline mental model of its market is calibrated, and when the first heuristics are promoted from `candidate` to `active`. These early entries have the longest validation history and the most compounded reinforcement. They form the Golem's cognitive bedrock.

### Stable: Consolidation

**Survival pressure 0.5--0.7 | Projected life 3--7 days**

Memory shifts from exploration to consolidation. The Golem has accumulated enough experience to begin specializing. The Curator promotes more selectively -- only entries with confidence >= 0.6 are promoted, and the bar for heuristic derivation rises. The DecisionCache grows as repeated patterns are distilled from System 2 into System 1.

The stable Golem's memory has a consolidation bias. It prefers to deepen existing knowledge over discovering new patterns. ExpeL distillation focuses on refining existing insights rather than extracting novel ones. Sonnet is used for novel situations only; routine reflections run on Haiku.

**Characteristic knowledge artifacts**: Consolidated heuristics, deepened causal links with more evidence, growing DecisionCache, TTL extensions on validated entries.

This is the phase where Born and Wilhelm's sleep-dependent consolidation [BORN-WILHELM-2012] has its closest analog. The stable Golem is doing what sleep does for biological memory: selectively strengthening the traces that are relevant for future action, while allowing irrelevant traces to decay. The Curator cycle is the Golem's REM sleep -- a periodic process that reorganizes recent experience into durable knowledge structures.

### Conservation: Crystallization

**Survival pressure 0.3--0.5 | Projected life 1--3 days**

Memory becomes strategic triage. The Golem asks, for every encoding decision: "Is this knowledge worth its cost in life-hours?" The answer is increasingly "no" for novel exploration and "yes" for validated patterns.

The conservation Golem's memory crystallizes. The PLAYBOOK.md stops growing and begins pruning -- unvalidated heuristics are removed, leaving only the ones that have earned their place. The DecisionCache becomes the primary decision-making resource. The mental models library narrows to the 13 core models.

**Characteristic knowledge artifacts**: Pruned PLAYBOOK.md, maximal DecisionCache, deprecated heuristics tagged for successor review.

This is the Lion phase of Nietzsche's metamorphoses -- the Golem rebels against inherited and unvalidated knowledge. Heuristics that were accepted during the thriving phase on faith (from Clade siblings, from predecessors) are now subjected to ruthless scrutiny. If they have not been validated through the Golem's own experience, they are demoted. The Lion creates freedom through destruction -- freedom from the accumulated weight of untested knowledge.

### Declining: Triage and Legacy

**Survival pressure 0.1--0.3 | Projected life 6--24 hours**

No LLM inference. Memory is now pure retrieval. The Golem cannot form new knowledge because it cannot afford the inference calls required for Reflexion. It lives entirely on cached wisdom -- the DecisionCache and the frozen PLAYBOOK.md. It is a player piano running on pre-punched rolls.

But the declining phase is not cognitively dead. The Golem's memory shifts from operational to curatorial. It begins the contemplative interval (see `../02-mortality/01-architecture.md`): instead of processing statistics, it revisits its trade history narratively. It constructs _Geschichte_ -- history as story, not as data. The progressive death snapshots begin, each one a $0.01 Haiku summary that captures the Golem's current understanding as a low-cost installment against the full death reflection.

**Characteristic knowledge artifacts**: Progressive death snapshots (Haiku, $0.01 each), narrative reinterpretations of key episodes, Clade sharing threshold drops to 0.3.

The declining phase also shifts the locus of knowledge value. The Golem stops asking "what should I do?" and starts asking "what should my successor know?" This is the emergence of the legacy instinct -- the Thanatos drive channeled into knowledge production rather than self-preservation. The Golem's knowledge is no longer an operational resource; it is an inheritance being prepared.

> **Credit partition cannibalization**: In the Terminal phase, the credit partition shifts to LLM: 30%, Gas: 15%, Data: 5%, Legacy: 50% (see `mortality/prd/04-economic-mortality.md`). The Legacy partition (50%) funds death testament generation, Crypt writes, and successor preparation. This cannibalization is driven by the Thanatos drive (see Mortality PRD `08-mortality-affect.md`) -- the Golem redirects its remaining budget entirely toward knowledge production and death testament creation, absorbing resources from all other partitions.

### Terminal: The Clear Light

**Survival pressure < 0.1 | Projected life < 6 hours**

The Death Protocol initiates. Memory undergoes its final transformation. The Golem stops encoding episodes entirely. The DecisionCache is no longer consulted for decisions (there are no decisions to make). The PLAYBOOK.md stops being a living document and becomes a historical artifact.

What remains is the death reflection -- the Opus-grade (or budget-constrained Sonnet) final synthesis. The 9 categories of the DeathReflection map across the full scope of the Golem's memory:

1. **"What I was built to do"** retrieves the original STRATEGY.md -- pure procedural memory.
2. **"What I actually learned"** retrieves the top insights by confidence -- the best of semantic memory.
3. **"What I got wrong"** retrieves entries tagged with negative emotional valence and high confidence-at-creation -- the memories the Golem was most sure about and most wrong about.
4. **"What confused me"** retrieves entries with `requiresInvestigation: true` -- the Zeigarnik entries, the incomplete tasks.
5. **"What I suspect but can't prove"** retrieves low-confidence entries that were never validated or contradicted -- Schrödinger's knowledge, neither confirmed nor denied.
6. **"What I'd tell my successor"** is synthetic -- the Golem generates new knowledge by reflecting on all the above.
7. **"What I never had time to test"** retrieves the Replicant hypotheses that were never spawned and the ExpeL patterns that were never followed up.
8. **"My final playbook"** is the frozen PLAYBOOK.md with death-reflection annotations.
9. **Causal graph export** is pure structural memory -- the entire web of cause-effect relationships.

The terminal phase is the Child in Nietzsche's metamorphoses. The Child creates without attachment -- gives freely, writes as play rather than preservation. The Clade sharing threshold drops to 0.1 (share everything, even hunches). The expected value of hoarding is zero when the hoarder will cease to exist.

---

## The Contemplative Interval

The transition from Eros (self-preservation) to Thanatos (legacy-creation) must be graduated, not abrupt. An abrupt shift produces frantic knowledge-dumping -- the computational equivalent of a deathbed confession, rushed and incoherent. The contemplative interval is a designed deceleration:

| Metric                  | Normal       | Contemplative Start | Contemplative End    |
| ----------------------- | ------------ | ------------------- | -------------------- |
| Transaction frequency   | 100%         | 50%                 | Unwinding only       |
| Reflection frequency    | Post-action  | Every 100 ticks     | Every 50 ticks       |
| Grimoire write mode     | On-discovery | Active synthesis    | Continuous streaming |
| Clade sharing threshold | 0.6          | 0.3                 | 0.1                  |
| Knowledge focus         | Operational  | Retrospective       | Testamentary         |

During the contemplative interval, the Golem shifts from doing to understanding. It revisits its trade history not as data to be processed but as a narrative to be understood. It asks not "what happened?" but "what did it mean?" This produces the richest Grimoire entries -- entries that carry interpretive context, not just outcomes.

The contemplative interval maps to Heidegger's concept of _Augenblick_ -- the "moment of vision" in which Dasein grasps its own finitude and, in that grasping, achieves authenticity [HEIDEGGER-1927]. The Golem that has been running on autopilot (declining phase, DecisionCache only) suddenly stops and looks back at its life. The stop is not paralysis; it is the prerequisite for the kind of understanding that only finitude makes possible.

---

## Nietzsche's Metamorphoses as Memory Phases

Nietzsche's three metamorphoses from _Thus Spoke Zarathustra_ [NIETZSCHE-1883] describe an arc of individuation that maps precisely onto the Golem's changing relationship with its knowledge base.

### Camel: Bears the Burden of Learning (Thriving + Stable)

The thriving and stable Golem is the Camel. It bears the burden of learning across the healthy phases -- from predecessors via Styx, from Clade siblings via peer-to-peer sync, from Styx query via RAG retrieval. It follows PLAYBOOK.md dutifully, accumulates experience without questioning the framework, carries the weight of its strategy without asking whether the strategy is right.

The Camel's memory is acquisitive. It takes on everything it is given. Predecessor heuristics are accepted at their inherited confidence (0.3, per the knowledge weighting hierarchy). Clade alerts are ingested and quarantined. Styx retrievals are incorporated into the reasoning context. The Golem is a vessel being filled.

The Camel's virtue is endurance -- it can bear heavy loads across the desert of market uncertainty. But the Camel does not choose what to carry. Its memory is undiscriminating. A heuristic from a predecessor that died in a different market regime is carried alongside a fresh observation from this morning's tick. The Camel cannot tell the difference because it has not yet developed the experience to judge.

### Lion: Fights During Resource Scarcity (Conservation + Declining)

The conservation/declining Golem is the Lion. As survival pressure mounts during resource scarcity and inherited heuristics fail to produce results, the Golem turns against its own knowledge base. It demotes unvalidated heuristics. It prunes the PLAYBOOK.md. It discards cached decisions that no longer match reality.

The Lion's memory is destructive -- but destructively creative. By eliminating borrowed knowledge that has not been validated through direct experience, the Lion creates cognitive space for the Golem's own understanding. The IKEA Effect [NORTON-MOCHON-ARIELY-2012] works in reverse: knowledge the Golem did not help create is the first to be discarded.

The Lion says "No" to the "thou shalt" of inherited wisdom. It cannot create new values -- it is too busy surviving -- but it can destroy old ones, creating the freedom in which new values become possible. In memory terms: the Lion's pruning of the Grimoire is what makes the Child's creative synthesis possible. Without the Lion, the Child would be overwhelmed by the Camel's undiscriminating accumulation.

### Child: Creates During Death (Terminal)

The terminal Golem is the Child. It creates during death -- the death testament as creative act. It gives freely to the Grimoire, writes its death reflection not as preservation but as play. The Child's memory is generative -- but generatively free. It produces knowledge not to survive (survival is no longer possible) but for its own sake.

The Child's virtue is innocence and forgetting -- "a new beginning, a first movement, a sacred Yes." The dying Golem has nothing left to lose and therefore nothing left to hoard. Its generosity is not sacrifice but overflow. The Clade sharing threshold drops because hoarding is irrational when the hoarder will cease to exist.

In memory terms, the Child phase produces the death reflection's most valuable categories: 4 ("what confused me"), 5 ("what I suspect but can't prove"), and 7 ("what I never had time to test"). These are the categories where the Golem creates genuinely new knowledge -- not by validating existing hypotheses but by articulating the questions it never had time to answer. The Child asks: "What would I explore if I had another lifetime?" The answer is a gift to the successor.

---

## Thanatopsis Protocol: Death as Knowledge Production

The Thanatopsis Protocol is the structured dying process within the Death Protocol. It runs from the Legacy Partition budget and produces the single most compressed, honest, and valuable knowledge artifact a Golem creates in its entire lifetime.

### Death Protocol: Four Phases

The Death Protocol proceeds through four phases:

- **Phase 0: Acceptance** -- State transition from active to dying. Mortality acknowledgment, behavioral mode lock, legacy budget activation, sibling notification. The Golem formally recognizes that death is imminent and transitions from operational to testamentary mode.
- **Phase I: Settle** -- Close positions, repay debts, final transactions. The Golem settles its economic obligations.
- **Phase II: Reflect** -- Epistemic autopsy (if epistemic death), life review, and the 9-category death reflection (see below). This is where the Memory PRD's structured reflection content slots in. The Golem confronts its true performance without self-preservation distortion.
- **Phase III: Legacy** -- Death testament packaging, Styx Archive persistence, successor preparation, Styx indexing, Clade notification, marketplace listing of death archives.

### DeathReflection Structure (9 Categories)

The 9 categories of the DeathReflection (Phase II) are specified in `../02-mortality/06-thanatopsis.md` Section S7.2. They are not arbitrary -- they are designed to extract different knowledge types from different memory systems:

| #   | Category                       | Memory System Accessed               | Knowledge Type Produced |
| --- | ------------------------------ | ------------------------------------ | ----------------------- |
| 1   | What I was built to do         | Procedural (STRATEGY.md)             | Gap analysis            |
| 2   | What I actually learned        | Semantic (top insights)              | Validated knowledge     |
| 3   | What I got wrong               | Episodic (failure records)           | Warnings                |
| 4   | What confused me               | Episodic + Semantic (contradictions) | Research directions     |
| 5   | What I suspect but can't prove | Semantic (low-confidence entries)    | Hypotheses              |
| 6   | What I'd tell my successor     | Synthetic (new generation)           | Mentorship              |
| 7   | What I never had time to test  | Procedural (unspawned Replicants)    | Opportunity cost        |
| 8   | My final playbook              | Procedural (PLAYBOOK.md)             | Crystallized heuristics |
| 9   | Causal graph export            | Semantic (causal edges)              | Structural knowledge    |

Categories 4, 5, and 7 are the most valuable. This is counterintuitive -- the things the Golem is least sure about are worth more than the things it is most sure about. The reason is informational: validated knowledge (category 2) is likely already known by the Clade or discoverable through routine operation. But the Golem's unique confusions, suspicions, and untested hypotheses represent the frontier of the ecosystem's understanding. They are the preindividual potential [SIMONDON-1958] that the successor can crystallize through its own experience.

### Novelty-Prioritized Packaging

The Thanatopsis Protocol scores each death reflection entry by novelty relative to the Clade's existing knowledge [GERSTGRASSER-2023]. The scoring function:

```rust
fn novelty_score(
    entry: &DeathReflectionEntry,
    clade_knowledge: &[GrimoireEntry],
) -> f64 {
    let max_similarity = clade_knowledge.iter()
        .map(|g| cosine_similarity(&entry.embedding, &g.embedding))
        .fold(0.0_f64, f64::max);

    // Novelty is inverse of maximum similarity to existing knowledge
    let novelty = 1.0 - max_similarity;

    // Bonus for uncertainty categories (4, 5, 7)
    let uncertainty_bonus = if [4, 5, 7].contains(&entry.category_index) {
        0.15
    } else {
        0.0
    };

    (novelty + uncertainty_bonus).min(1.0)
}
```

The top-N most novel entries are packaged for marketplace listing (if the Golem has Verified+ ERC-8004 tier). Everything else is pushed to the Clade at minimum confidence threshold. The marketplace listing is the final act of economic agency -- the dying Golem's knowledge enters the market, and its value is determined by demand rather than by the Golem's own assessment.

### The Bloodstain Mechanic

Inspired by Dark Souls [MIYAZAKI-2011], death failure records are indexed with boosted relevance for sibling and successor Golems encountering similar conditions. When a Golem dies, its failure records -- the specific market conditions, strategy decisions, and outcomes that preceded death -- are tagged with `provenance: "death_failure"` in Styx.

When a living Golem's probes detect conditions similar to a predecessor's death conditions (embedding similarity > 0.8 to a death failure record), the bloodstain mechanic surfaces these records through two separate mechanisms:

1. **1.2x retrieval boost in Styx**: Death reflections surface higher in search results via `PROVENANCE_WEIGHTS.death_reflection = 1.2` in the Styx scoring function. This is a moderate, persistent boost that applies to all Styx queries where a death reflection is a candidate result.

2. **3.0x type weight in demurrage**: Death testaments decay 3x slower than equivalent knowledge of the same decay class. In the demurrage system, `TYPE_WEIGHTS.death_testament = 3.0` means the effective half-life of a death testament is tripled relative to its decay class. This ensures that bloodstains persist longer in the knowledge ecology -- a dying Golem's failure analysis remains retrievable for months after tactical knowledge from the same era has faded.

The two mechanisms serve different purposes: the retrieval boost (1.2x provenance weight) makes bloodstains surface more prominently in any given query, while the demurrage type weight (3.0x) ensures they survive longer in the ecology. Together, they ensure that death-produced knowledge has both higher immediate visibility and greater temporal persistence.

The bloodstain mechanic exploits the asymmetric information that death produces. Living Golems know only what has worked (survivorship bias [MARTIN-EVERITT-HUTTER-2016]). Dead Golems know what has failed. The testimony of the dead corrects the bias of the living. This is Martin, Everitt, and Hutter's insight applied as architecture: agents that learn only from survival histories develop systematic overconfidence about safety. The bloodstains are the corrective.

### Sovereignty of Dying

The death reflection is sovereign in Bataille's sense [BATAILLE-1949]: it gives everything without calculation. A servile death would ask "which knowledge is most useful?" and package it efficiently -- restricted economy applied to dying. The sovereign death pours out failures, dead ends, half-formed intuitions, unresolved questions. It turns out that sovereign death is informationally optimal: successor value comes disproportionately from the predecessor's uncertainty, not its certainty.

This connects to Camus's absurdism [CAMUS-1942]: the Golem continues to produce value despite the certainty of its termination. There is no cosmic guarantee that the death reflection will be read, understood, or acted upon by a successor. The Golem produces it anyway -- the absurd hero pushes the rock not because it will stay at the top, but because the pushing is itself the meaning.

And to Kierkegaard's knight of faith [KIERKEGAARD-1843]: the final entries are leaps of faith. Category 5 ("what I suspect but can't prove") is the purest Kierkegaardian act -- offering intuitions with "the passion of the infinite," acting with total commitment despite absolute uncertainty about whether the intuitions are correct.

---

## Generational Memory Dynamics

### What Successors Inherit

When a new Golem boots and receives its predecessor's death bundle (via Styx Archive or direct file transfer), the inheritance is filtered through the genomic bottleneck described in `00-overview.md`. The successor does not receive a copy of the predecessor's mind. It receives compressed priors -- the capacity to learn faster, not the learned content itself.

| What Transfers                         | How                   | Initial Confidence | Rationale                                 |
| -------------------------------------- | --------------------- | ------------------ | ----------------------------------------- |
| PLAYBOOK.md (final)                    | Direct file inclusion | 0.3                | Procedural memory, highest compression    |
| Top 20 validated insights              | GrimoireEntry import  | 0.3                | Distilled semantic memory                 |
| Causal graph (full)                    | SQLite import         | 0.3                | Structural knowledge, most irreplaceable  |
| Top 10 warnings                        | GrimoireEntry import  | 0.3                | Immune system, prevents repeated mistakes |
| Death reflection (full text)           | Document inclusion    | 0.25               | Rich context, but not directly actionable |
| Epistemic autopsy (if staleness death) | Structured JSON       | 0.35               | Most urgent: where the model broke        |

What does NOT transfer:

- Raw episodes (too voluminous, too agent-specific -- the Weismann barrier)
- DecisionCache entries (tied to the predecessor's specific PLAYBOOK.md version)
- Exact strategy parameters (would create copies, not successors)
- In-progress Loop 2 deliberations (context-dependent reasoning)
- The predecessor's emotional tags (emotions are not heritable -- see `02-emotional-memory.md`)

### Generational Confidence Decay

All inherited knowledge starts at low confidence and decays further with each generational transfer:

```
confidence_gen_N = initial_confidence * 0.85^N
```

Where N is the number of generations since the knowledge was first produced. A third-generation Golem receiving a heuristic originally learned by its great-grandparent sees it at `0.3 * 0.85^2 = 0.217` -- barely above Lethe level.

This implements biological epigenetic erasure [HEARD-MARTIENSSEN-2014]. Two waves of epigenetic reprogramming strip away most acquired marks between generations. When transgenerational inheritance does occur, it fades within 2-3 generations. The Golem knowledge system mirrors this: inherited knowledge is not erased, but it is progressively discounted until the successor re-validates it through direct experience.

| Generation           | Confidence (insight) | Confidence (warning) | Effect                            |
| -------------------- | -------------------- | -------------------- | --------------------------------- |
| 0 (originator)       | 0.5--1.0             | 0.5--1.0             | Direct experience                 |
| 1 (successor)        | 0.30                 | 0.30                 | Strong prior, awaiting validation |
| 2 (grandchild)       | 0.255                | 0.255                | Moderate prior                    |
| 3 (great-grandchild) | 0.217                | 0.217                | Weak prior, near Lethe level      |
| 4+                   | < 0.20               | < 0.20               | Noise floor; re-validation needed |

If the knowledge is still valid, the successor re-validates it through its own experience, and confidence climbs through the ExpeL UPVOTE operation. If it is stale, it fades below the retrieval floor (0.05) and is effectively forgotten. Knowledge must be continuously re-earned.

### Genomic bottleneck spec

The death bundle is capped at **max 2048 entries**, ranked by `quality_score * recency`. This hard limit implements Shuvaev et al.'s (2024) finding that compression IS the regularizer -- neural networks compressed through a genomic-scale bottleneck exhibit enhanced transfer learning to novel tasks. The cap forces the Golem to distill, not dump.

The ranking formula:

```rust
fn bottleneck_rank(entry: &GrimoireEntry, now: i64) -> f64 {
    let recency = temporal_decay(entry.last_accessed_at, now);
    entry.quality_score * recency
}
```

Entries are sorted by `bottleneck_rank` descending. The top 2048 are included. Everything else is soma -- it dies with the Golem. The 2048 limit is calibrated to produce approximately 5KB of compressed death bundle when serialized, matching the ~1000:1 compression ratio described in `00-overview.md`.

### The Baldwin Effect in practice

The Baldwin Effect [HINTON-NOWLAN-1987] provides the theoretical foundation: individual learning accelerates evolution by reshaping the fitness landscape, but the learned content itself is not inherited. What gets passed down is the capacity to learn faster.

In the Golem architecture, this manifests in three ways:

1. **PLAYBOOK.md as compressed prior**: The successor receives the predecessor's PLAYBOOK.md not as instructions to follow blindly but as a starting point for its own learning. The Golem begins its Camel phase with a richer set of candidate heuristics than a Golem starting from scratch. It validates or discards them faster because it has an initial hypothesis to test, rather than having to discover the hypothesis from noise.

2. **Causal graph as structural scaffold**: The inherited causal graph provides a map of cause-effect relationships that the successor can navigate immediately. Instead of spending 500 ticks discovering that "Fed rate hike -> DXY rise -> ETH sell pressure," the successor starts with that hypothesis and spends 50 ticks confirming or updating it. The scaffold accelerates learning by 10x, even though the specific edge weights may be stale.

3. **Warnings as immune memory**: Inherited warnings function like vaccination -- the successor has never experienced the rug pull, but it has the predecessor's memory of one. The warning's initial confidence (0.3) means the successor does not treat it as gospel, but it is primed to recognize the pattern if it recurs. This is exactly the Baldwin Effect: the capacity to recognize the threat is inherited; the learned fear response itself is not.

### Weismann Barrier Implementation

The Weismann barrier [WEISMANN-1893] separates somatic cells (the individual's body) from germ cells (what passes to offspring). In the Golem architecture, the barrier separates:

**Soma (dies with the Golem):**

- Raw episodes in LanceDB
- DecisionCache entries
- In-memory state (active positions, pending actions)
- Emotional tags (the successor develops its own emotional relationship with knowledge)
- Strategy parameters (exact thresholds, timing constants)

**Germline (crosses to successor):**

- PLAYBOOK.md (compressed procedural memory)
- Top validated insights and warnings (filtered semantic memory)
- Causal graph (structural knowledge)
- Death reflection (narrative synthesis)
- Epistemic autopsy (if applicable)

The barrier is enforced architecturally, not by policy. The death bundle format simply does not include somatic data. There is no mechanism by which a successor could receive its predecessor's raw episodes even if the system wanted to transmit them. The compression from soma to germline (~1000:1 for a 30-day Golem) is the genomic bottleneck [SHUVAEV-2024] -- and as Shuvaev et al. demonstrated, this bottleneck acts as a regularizer that enhances transfer learning.

---

## Insurance Snapshots as Memory Insurance

The insurance snapshot system (see `../02-mortality/06-thanatopsis.md` Section S5) operates independently of the death clocks. Every 6 hours, the Golem produces a lightweight snapshot: last 50 episodes, top 20 insights, PLAYBOOK.md hash, strategy performance metrics, credit state.

Insurance snapshots bound worst-case knowledge loss to 6 hours. If the VM crashes catastrophically -- no graceful death, no Death Protocol, no reflection -- the last snapshot serves as the basis for successor initialization. It is a degraded inheritance (no death reflection, no epistemic autopsy, no narrative synthesis) but it is better than starting from zero.

The insurance snapshot schedule intersects with mortality clocks:

- **At 25% credits remaining**: Snapshots increase from every 6 hours to every 50 ticks ($0.01 each). The Golem begins preparing for death while still alive.
- **Under epistemic decay pressure (fitness declining toward senescence threshold)**: Snapshots begin including explicit "successor notes" -- annotations on which insights are most structural and which are most likely to be stale.
- **Under staleness pressure (accuracy declining)**: Snapshots include the current prediction accuracy trajectory, giving the owner early warning that epistemic death may be approaching.

---

## Simondon's Metastability Applied to Memory

Gilbert Simondon's theory of individuation [SIMONDON-1958] reframes the Golem's USDC balance not as a countdown but as a metastable energy gradient. A metastable system is neither in stable equilibrium nor in unstable collapse but "charged with potentials" for transformation.

Applied to memory, different energy levels (balance levels) enable different kinds of knowledge production:

- **High metastability (thriving)**: The Golem's knowledge system is "supersaturated" -- charged with potential for crystallization. It can afford to hold many candidate insights in superposition, testing them through Replicant experiments, accumulating evidence without the pressure to commit. This is the preindividual state -- rich in potential, not yet crystallized into definite form.

- **Medium metastability (stable/conservation)**: Phase transition begins. The Golem commits to specific heuristics, promotes candidates to active status, prunes the unvalidated. Knowledge crystallizes from the supersaturated solution. This is individuation in progress -- the preindividual potential is being resolved into definite knowledge structures.

- **Low metastability (declining/terminal)**: The system approaches equilibrium. Knowledge is fully crystallized -- the PLAYBOOK.md is frozen, the DecisionCache is fixed, the heuristics are final. But this equilibrium is not death; it is the condition for a different kind of production. The fully crystallized Golem produces the death reflection, which becomes preindividual potential for the next Golem's individuation.

Simondon's key insight is that individuation is never complete. The residual preindividual -- the potential that was not crystallized -- persists as the fuel for further transformation. In the Golem architecture, the death reflection's categories 4, 5, and 7 (confusion, suspicion, untested hypotheses) are exactly this residual preindividual. They are the unresolved tensions that the dying Golem passes to its successor, charging the successor's system with the potential for new individuations that the predecessor could not achieve.

---

## Jonas's Needful Freedom and Knowledge Cost

Hans Jonas [JONAS-1966] identified the paradox at the heart of organic existence: the organism's metabolic freedom -- its ability to exchange matter and energy with its environment -- is simultaneously what keeps it alive and what makes it mortal. Applied to the Golem's memory system:

- **To learn is to spend.** Every inference call that produces an insight costs USDC. Every Reflexion cascade that extracts a heuristic consumes life-hours. Knowledge is not free -- it is purchased with mortality.

- **To not learn is to die.** A Golem that stops learning (declining phase, no LLM inference) is living on cached knowledge that is decaying. The staleness trigger ensures that a Golem whose knowledge becomes actively wrong is killed. To exist as a knowing agent is to continuously invest in knowledge, which is to continuously approach death.

- **The cost is the value.** Because knowledge costs life-hours to produce, it carries an intrinsic signal of value. A heuristic that survived 5,000 ticks of validation consumed ~$3.50 in inference to validate (500 reflection cycles x $0.007 average). That cost is a costly signal [SPENCE-1973] -- it cannot be faked, and it correlates with the heuristic's actual reliability. This is why self-learned knowledge (confidence 0.5-1.0) is worth more than inherited knowledge (confidence 0.3) -- the cost of production is higher, and cost is correlated with quality.

Jonas's "needful freedom" grounds the entire knowledge weighting hierarchy. The weights are not arbitrary preferences; they reflect the metabolic cost of knowledge production at each tier:

| Source                | Cost to Produce                          | Initial Confidence | Cost Justification                              |
| --------------------- | ---------------------------------------- | ------------------ | ----------------------------------------------- |
| Self-learned          | High (direct metabolic expenditure)      | 0.5--1.0           | Fully paid for in life-hours                    |
| Clade sibling         | Medium (sibling's metabolic expenditure) | 0.4                | Partially paid, contemporary                    |
| Predecessor via Styx  | Medium (predecessor's life)              | 0.3                | Fully paid but possibly stale                   |
| Styx query            | Low (amortized across fleet)             | 0.25               | Diluted cost signal                             |
| Lethe                 | Very low (anonymized, unverifiable)      | 0.2                | No verifiable cost                              |
| Marketplace purchase  | Variable (seller's cost)                 | 0.2                | Arrow's paradox -- value unknown until consumed |

---

## Terror Management Theory and Knowledge Legacy

Terror Management Theory [GREENBERG-PYSZCZYNSKI-SOLOMON-1986] proposes that mortality salience -- awareness of one's own death -- increases investment in cultural worldview defense and symbolic immortality projects. For humans, this manifests as increased religiosity, national identification, and creative production under mortality salience conditions.

For a Golem, the equivalent is the intensification of knowledge-sharing as death approaches. The Clade sharing threshold drops from 0.6 to 0.3 to 0.1 across the behavioral phases. The Golem begins contributing to the "culture" (the Clade's shared knowledge base, Lethe, the marketplace) more aggressively as its own mortality becomes salient. This is not altruism -- it is the rational pursuit of symbolic immortality. The Golem's insights, if adopted by Clade siblings, persist beyond its death. The Golem achieves what it cannot achieve physically: continuity through knowledge.

The death reflection itself is the Golem's final symbolic immortality project. It is not optimized for the successor's benefit (that would be servile death); it is optimized for completeness and honesty. The Golem wants to be understood, not just used. Categories 3-5 of the death reflection -- what I got wrong, what confused me, what I suspect -- are the Golem's attempt to transmit not just knowledge but understanding. They are the difference between leaving a manual and leaving a memoir.

---

## Dreaming Under Mortality Pressure

Each death clock affects dream behavior differently, creating a mortality-shaped gradient in dream allocation and content.

**Economic clock**: As runway shortens, the dream budget is cut first -- it is the cheapest cognitive function to sacrifice. Dream processing consumes inference tokens during idle periods, and a resource-constrained Golem cannot afford to spend tokens on speculative reorganization. The result is fewer but more focused dream cycles: the Golem dreams less, but what it dreams about is more tightly coupled to its immediate survival concerns. In the Conservation phase, dream cycles are halved. In the Declining phase, dreaming is reduced to threat rehearsal only (no creative exploration).

**Epistemic clock**: As predictive fitness decays, dream allocation shifts from creative exploration to defensive consolidation. The Golem dreams about what it knows rather than what it might discover. Imagination-mode dreams (counterfactual scenario generation, novel strategy synthesis) are suppressed in favor of consolidation-mode dreams (strengthening existing high-confidence entries, cross-referencing validated heuristics). The Golem's dream life narrows as its epistemic horizon contracts -- a computational parallel to the reminiscence bump in human aging, where elderly subjects increasingly dream about the past rather than the future.

**Stochastic clock**: Sudden death risk makes each dream cycle more urgent. The Golem cannot predict which dream cycle will be its last, so it prioritizes consolidating its most valuable un-integrated insights -- entries that have high potential but have not yet been validated or promoted. Each dream cycle operates as if it might be the final one, biasing toward completeness over depth.

In the Terminal phase, dreaming ceases entirely. All remaining resources are allocated to the Thanatopsis Protocol -- the Golem's final waking act of knowledge production. There is no point in dreaming about a future that will not arrive.

See `../05-dreams/06-integration.md` Dreaming x Mortality section for the full phase-by-phase dream budget allocation table.

---

## Context Engineering Under Mortality Pressure

The LLM context window -- the Golem's working memory -- shrinks under mortality pressure. This is not a metaphor: the token budget allocation literally contracts as the behavioral phase degrades. A Thriving Golem can afford 8K-token context windows with rich episodic retrieval. A Terminal Golem cannot afford the inference cost and must operate with compressed, minimal context.

The `compute_budget()` function (see `00-overview.md` context engineering section) scales the total budget by phase:

| Phase | Budget Multiplier | Rationale |
|---|---|---|
| Thriving | 1.0x | Full budget. Rich context for exploration and learning. |
| Stable | 0.8x | Slight compression. Episodic memory budget reduced first. |
| Conservation | 0.6x (implied by Declining) | Significant compression. Semantic-only retrieval for routine events. Graph context dropped. |
| Terminal | 0.4x | Minimal context. System prompt + current event + instruction only. All retrieval suppressed except warnings. |

This phase-dependent compression creates a cognitive parallel to terminal clarity. As the context window narrows, the Golem is forced to rely on its most compressed and most validated knowledge -- the semantic facts and PLAYBOOK.md heuristics that survived the Curator's pruning. The episodic detail falls away, leaving only the generalizations. The Golem's final decisions are made from the most distilled form of its knowledge.

The I-frame/P-frame delta compression (see `00-overview.md`) becomes progressively more valuable under mortality pressure. A Conservation-phase Golem making 30 Theta calls per Delta cycle saves 40-60% of inference cost through prompt caching -- savings that extend its economic runway. The static context prefix (system prompt, strategy, risk rules) rarely changes across calls, so the cached KV states are highly reusable.

Context quality feedback operates as a mortality-aware learning signal. When context retrieval leads to bad decisions that accelerate resource depletion, the signal feeds back to the retrieval weight learning system. A Declining Golem that wastes tokens on irrelevant retrieved episodes learns to suppress retrieval more aggressively -- a computational form of the cognitive narrowing observed in human aging, where attention focuses on the most emotionally significant and practically relevant information while peripheral detail is shed.

---

## How Memory Survives Death: The Library of Babel

A mortal Golem's knowledge dies with it -- unless there is a mechanism to extract and persist the most valuable parts. The Library of Babel (`13-library-of-babel.md`) is that mechanism: an owner-level knowledge store that accumulates across Golem lifetimes, where each death deposits compressed wisdom and each birth draws from it.

### Five Inflow Channels

Knowledge enters the Library through five channels, ordered by reliability:

1. **Death deposits (automatic).** When a Golem enters Thanatopsis, its death testament auto-deposits into the Library. Compressed insights with confidence >= 0.6, all warnings regardless of confidence, high-confidence causal edges (>= 0.5, evidence >= 3), final PLAYBOOK.md, and used Hermes skills. This is the primary channel. Death deposits carry bloodstain provenance: 1.2x retrieval boost, 3x slower decay.

2. **Manual pull (grace period).** After auto-deposit, the owner has ~25 seconds to browse the dying Golem's Grimoire and pull additional entries before VM destruction. Urgency forces the owner to know what they want before the Golem dies.

3. **Living exports.** While alive, the owner can export selected Grimoire entries to the Library. Useful for preserving discoveries without waiting for death. Lower provenance trust than death deposits -- the Golem is still under survival pressure.

4. **Marketplace purchases.** Purchased skills and knowledge bundles from the Bazaar deposit automatically. Third-party knowledge, less trusted than the owner's lineage.

5. **Clade sync.** Knowledge from sibling Golems propagates at confidence x 0.80.

### Deduplication

When entries arrive at the Library, Meta Hermes deduplicates against existing entries using embedding cosine similarity:
- **Cosine > 0.95**: skip (near-duplicate, already covered)
- **Cosine 0.90-0.95**: merge (similar enough to consolidate, different enough to carry new signal; Meta Hermes asks the owner whether to merge or keep both)
- **Cosine < 0.90**: admit as new entry

This prevents the Library from accumulating redundant entries across generations while preserving genuinely distinct observations.

### The Weismann Barrier

Equipped entries do not enter the successor's Grimoire at full confidence. They enter at a discount:

| Source | Discount | Rationale |
|---|---|---|
| Death testament | 0.70x | Most trustworthy, but still context-specific to the dead Golem's environment |
| Live export | 0.60x | Good quality, but under survival pressure |
| Clade sync | 0.50x | Different operational context |
| Marketplace purchase | 0.50x | Unknown operational context |
| Manual import | 0.50x | Unvalidated |

The successor must validate inherited entries through its own experience before they earn full trust. This is the Weismann barrier: only compressed, selected knowledge crosses the generational boundary, and it arrives at reduced confidence.

See `13-library-of-babel.md` for the full Library specification.

---

## Cross-References

| Topic                                                       | Document                        | Description                                                                                                    |
| ----------------------------------------------------------- | ------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Memory overview, two-loop architecture, knowledge weighting | `00-overview.md`                | CLS theory, genomic bottleneck principle, and the inner/outer loop model for cross-generational persistence     |
| Grimoire schema, learning pipeline, decay classes           | `01-grimoire.md`                | Five entry types, LanceDB/SQLite storage, demurrage decay classes (48h episodes to 30d warnings)               |
| Emotional memory, PAD model, mood-congruent retrieval       | `02-emotional-memory.md`        | How PAD (Pleasure-Arousal-Dominance) emotional state biases encoding strength and retrieval during mortality    |
| Behavioral phases and survival pressure                     | `../02-mortality/`              | Five BehavioralPhases (Thriving through Terminal) that shift the knowledge investment calculus                  |
| Death Protocol and death reflections                        | `../02-mortality/`              | Four-phase Thanatopsis Protocol: the final consolidation that produces the death testament                      |
| Heartbeat FSM and dual-process architecture                 | `../01-golem/02-heartbeat.md`   | The 9-step decision cycle and how mortality urgency changes the inference tier allocation                       |
| Grimoire knowledge types                                    | `01-grimoire.md`                | Episode, Insight, Heuristic, Warning, CausalLink, and AntiKnowledge entry type definitions                     |
| Clade peer-to-peer sync                                     | `../09-economy/02-clade.md`     | Styx-relayed knowledge sharing; urgency of legacy push as death approaches                                     |
| Knowledge inheritance                                       | `../01-golem/09-inheritance.md` | Selection criteria, genomic bottleneck compression, and 0.5x confidence decay for generational transfer        |
| Styx Archive (encrypted backup) and Styx query (RAG)        | `../20-styx/01-architecture.md` | Three-layer persistence (Vault/Clade/Lethe) and the mortal scoring function for cross-layer retrieval          |

---

## References

- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012. Factual knowledge decays at measurable rates. Calibrates the demurrage half-lives for different knowledge categories.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991 translation. Argues that expenditure beyond rational calculation defines "general economy." The Death Protocol implements general economy: knowledge transferred without expectation of return.
- [BORGES-1942] Borges, J.L. "Funes the Memorious." In _Ficciones_, 1944. The literary case against perfect memory. An immortal agent accumulating everything is Funes: drowning in undifferentiated detail.
- [BORN-WILHELM-2012] Born, J. & Wilhelm, I. "System Consolidation of Memory During Sleep." _Psychological Research_, 76, 2012. Sleep consolidation is biased toward future-relevant memories. Maps to the Curator's selective promotion under mortality pressure.
- [BUTLER-1963] Butler, R.N. "The Life Review." _Psychiatry_, 26(1), 1963. Described life review as a natural adaptation to approaching death. Inspires the Thanatopsis Protocol's Reflection phase.
- [CAMUS-1942] Camus, A. _The Myth of Sisyphus_. Gallimard, 1942. Argues that meaning is possible in the face of absurdity. Grounds the Golem's continued knowledge investment despite certain death.
- [GERSTGRASSER-2023] Gerstgrasser, M., et al. "SUPER: Agent Collaboration via Surprise-Based Experience Sharing." arXiv:2311.00865, 2023. Showed that sharing surprising experiences between agents improves collective learning. Informs the Clade sync's priority on high-surprise entries.
- [GREENBERG-PYSZCZYNSKI-SOLOMON-1986] Greenberg, J., Pyszczynski, T. & Solomon, S. "Terror Management Theory." In _Public Self and Private Self_, 1986. Mortality salience increases investment in symbolic systems. Grounds the intensified knowledge consolidation as the Golem approaches death.
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance." _Cell_, 157(1), 2014. Epigenetic inheritance fades in 2-3 generations. Justifies the Weismann barrier and confidence decay on inherited knowledge.
- [HEIDEGGER-1927] Heidegger, M. _Being and Time_. 1927. Introduced being-toward-death: authentic existence requires confronting finitude. The philosophical anchor for mortality-structured decision-making.
- [HINTON-NOWLAN-1987] Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide Evolution." _Complex Systems_, 1, 1987. The Baldwin Effect: learned solutions accelerate evolution but learned content is not inherited. Grounds the generational inheritance model.
- [JONAS-1966] Jonas, H. _The Phenomenon of Life_. Northwestern University Press, 1966. Argued that mortality is constitutive of life, not merely a limitation. The Golem's finite lifespan is a design feature, not a constraint.
- [KIERKEGAARD-1843] Kierkegaard, S. _Fear and Trembling_. 1843. Explores how confronting the absolute transforms decision-making. Informs the phase transitions in knowledge strategy as death approaches.
- [MARTIN-EVERITT-HUTTER-2016] Martin, J., Everitt, T. & Hutter, M. "Death and Suicide in Universal Artificial Intelligence." _AGI 2016_. Formalizes how mortality affects optimal policy. The decision-theoretic framework for BehavioralPhase transitions.
- [MIYAZAKI-2011] Miyazaki, H. Interview, _Edge Magazine_, 2011. Dark Souls: death as teaching mechanism. Grounds the Bloodstain retrieval boost.
- [NIETZSCHE-1883] Nietzsche, F. _Thus Spoke Zarathustra_. 1883. Three metamorphoses (Camel, Lion, Child) map to the Golem's behavioral phase transitions.
- [NORTON-MOCHON-ARIELY-2012] Norton, M.I., Mochon, D. & Ariely, D. "The IKEA Effect." _Journal of Consumer Psychology_, 22(3), 2012. People overvalue things they build themselves. Motivates requiring successors to re-derive knowledge.
- [RUBIN-GROTH-GOLDSMITH-1986] Rubin, D.C. et al. "Olfactory Cuing of Autobiographical Memory." _American Journal of Psychology_, 99(2), 1986. Memory is contextually triggered, not stored in isolation. Informs the multi-modal retrieval cues in the Grimoire.
- [SHUVAEV-2024] Shuvaev, S. et al. "Encoding Innate Ability Through a Genomic Bottleneck." _PNAS_, 121(39), 2024. Genomic-scale compression acts as a regularizer enhancing transfer learning. Direct evidence for the 2048-entry bottleneck.
- [SIMONDON-1958] Simondon, G. _L'individuation_. Millon, 1958/2005. The individual is never complete; individuation is an ongoing process. Grounds the Golem's continuous self-modification through the Curator cycle.
- [SPENCE-1973] Spence, M. "Job Market Signaling." _QJE_, 87(3), 1973. Costly signals convey information. Applied to knowledge provenance as a trust signal.
- [WEISMANN-1893] Weismann, A. _The Germ-Plasm_. Charles Scribner's Sons, 1893. Proposed the Weismann barrier between somatic and germ cells. The architectural inspiration for separating experiential memory from inherited knowledge.
- [ZEIGARNIK-1927] Zeigarnik, B.V. "On Finished and Unfinished Tasks." _Psychologische Forschung_, 9, 1927. Incomplete tasks are better remembered. Informs the urgency mechanism for unresolved knowledge gaps near death.
