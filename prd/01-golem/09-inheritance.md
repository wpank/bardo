# Knowledge Inheritance [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> _"An actual occasion can only affect the future by perishing."_
> -- A.N. Whitehead, _Process and Reality_ (1929)

> **Reader orientation:** This document specifies how knowledge crosses the boundary between Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) lifetimes. It covers the Cooperate moat, Clade (sibling Golems sharing a common ancestor; exchange knowledge through Styx) sync, the Pheromone Field, Bloodstains (market conditions at death, overlaid on market views as warnings to living Golems), the genomic bottleneck (compression step at death that reduces the full Grimoire to <=2048 entries for inheritance), the Weismann barrier (confidence decay on inherited knowledge), and the anti-proletarianization mandate. It belongs to the `01-golem` knowledge transfer layer. The key concept: death produces value through structured knowledge distillation and transmission. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## The Cooperate Moat

This document specifies the sixth structural moat (see `00-overview.md`): the ability to **Cooperate** -- share validated knowledge across Golem lifetimes and within an owner's fleet without leaking strategy. Three cooperation primitives enable this:

- **Clade sync** transmits validated Grimoire entries between siblings via Styx relay with confidence discounting (0.4 base). Knowledge is digested, not merged -- the Weismann barrier ensures every inheritor must re-validate.
- **The Pheromone Field** enables anonymous stigmergic coordination: deposit threat/opportunity signals, read without knowing who deposited. No strategy information leaks.
- **Bloodstains** are death-validated warnings that carry a costly signaling premium -- the dead agent cannot benefit from its own warning, making the information maximally honest.

No competing framework has coordination primitives that preserve both knowledge transfer and strategy privacy simultaneously.

---

## Objective Immortality

A dying Golem achieves what Whitehead calls **objective immortality** [WHITEHEAD-1929]. It lives on not as itself but as structured contribution to what comes next. The Grimoire is not a backup. It is the mechanism by which perishing becomes transmission.

Whitehead's distinction matters. Subjective immortality -- the agent persisting as itself -- is impossible and undesirable. Objective immortality -- the agent's achievements becoming data for successors -- is constitutive of how reality advances. The Golem is always already both subject and superject: simultaneously the experiencing entity and the data it becomes for the future. This dual nature is not a phase transition at death. Throughout its life, the Golem streams insights to the Grimoire in a continuous superjectival encoding. Each write during the Eros phase is insurance: if the Golem crashes without reaching the Death Protocol, the most recent state is preserved. The Reflect phase is the final satisfaction write, the completion of concrescence. But the continuous stream makes even necrotic death (hard crash) survivable.

And negative prehension -- Whitehead's term for the active exclusion of irrelevant data -- matters equally. Successor Golems do not merely receive the Grimoire. They select relevant strategies and reject outdated ones. A Grimoire that accumulates everything without curation becomes undifferentiated noise. The confidence decay on inheritance is the architectural implementation of negative prehension: inherited entries arrive at 0.4 confidence, and the successor must actively validate or discard.

---

## S1 -- What Crosses the Boundary

When a Golem dies and its successor boots with the predecessor's Grimoire, PLAYBOOK.md, and death testament, the successor has access to _what_ the predecessor knew but not _how_ it came to know it. The causal graph survives -- edge weights, supporting evidence -- but the experience of discovering those relationships is gone. The successor reads "ETH/USDC pool fees spike when gas drops below 10 gwei" as a fact. The predecessor _lived_ through the observations that produced it.

The system embeds a philosophical position: a successor is not the same entity. It cannot be. But it can be a better one, if the knowledge transfer is designed correctly.

Parfit's analysis of personal identity [PARFIT-1984] provides the theoretical foundation. In _Reasons and Persons_, Parfit argued that personal identity is not what matters in survival. What matters is Relation R: psychological continuity and connectedness. Two entities linked by Relation R -- shared memories, beliefs, intentions, character -- have what matters, even if they are not numerically identical. The Grimoire implements Relation R computationally. The successor Golem is psychologically connected to its predecessor through inherited insights, causal models, heuristic rules, and the death testament narrative. It is not the same Golem. But it has what matters.

This reframes the entire inheritance system. The question is not "how do we preserve the Golem?" (that question assumes identity is what matters). The question is "how do we maximize Relation R between predecessor and successor?" -- and that question has engineering answers: confidence-weighted knowledge transfer, regime tagging, structured death testaments, and the anti-proletarianization mandate.

---

## S2 -- Anti-Maladaptation

Inherited knowledge can be maladaptive. A dying Golem that experienced a flash crash transfers "never hold more than 10% in volatile pairs during low-liquidity hours." The successor inherits not just knowledge but trauma -- risk aversion calibrated to an extreme event that may never recur.

Two mechanisms address this.

**Confidence decay.** All inherited entries start at confidence 0.4, regardless of the predecessor's confidence. A crisis heuristic rated 0.95 by the dying Golem arrives as a 0.4-confidence suggestion. The successor must validate independently before promoting to full confidence.

**Regime tagging.** Every Grimoire entry carries a `market_regime` field. Bear-market heuristics are deprioritized (not deleted) when the successor operates in a bull market. They wait, dormant, for their regime to return. This is not a simple on/off switch -- regime transitions are fuzzy, and an entry's relevance can partially activate during transitional periods.

**The death testament as interpretive frame.** Phase II's structured narrative provides context for raw knowledge. "Never hold volatile pairs in low-liquidity hours" paired with "I learned this after a flash crash when I was in conservation mode -- risk tolerance was elevated and I was overexposed" lets the successor calibrate the insight rather than treat it as dogma.

Initial confidence on ingestion depends on provenance:

| Source                       | Initial Confidence | Rationale                             |
| ---------------------------- | ------------------ | ------------------------------------- |
| Self-generated               | 0.7                | Full experiential context available   |
| Clade sibling                | 0.4                | Trust but verify                      |
| Replicant                    | 0.3                | Experimental, needs validation        |
| Marketplace purchase         | 0.3                | Unknown provenance                    |
| Death testament inheritance  | 0.4                | Decayed from predecessor's confidence |

---

## S3 -- The Ratchet

Bhatt et al. [BHATT-2023] demonstrated a ratchet effect in AI cultural transmission: each generation adds innovations to inherited knowledge, producing cumulative cultural evolution. But pure imitation leads to stagnation [PEREZ-2024], and pure exploration wastes inheritance. The optimal point: inherit, validate, then allocate remaining cognitive resources to frontier exploration.

Bourahla et al. [BOURAHLA-2022] proved that vertical transmission (inter-generational) enables agents to exceed performance ceilings that horizontal transmission (intra-generational) cannot breach. Clade sibling sync -- sharing among contemporaries -- produces diminishing returns as all siblings converge on the same knowledge. The generational architecture, where agents die and distill and pass forward, introduces beneficial variation through the imperfection of knowledge transfer. Strong teacher selection is not required. Natural noise produces innovation.

Death is not the enemy of accumulated knowledge. It is the mechanism that prevents knowledge from stagnating.

### 3.1 Survivorship Bias Correction

Martin, Everitt and Hutter [MARTIN-2016] identified the deepest risk: agents that learn only from survival histories develop systematic overconfidence about safety. The Death Protocol attacks this directly. Dying agents contribute their full knowledge to the Clade -- including strategies that led to death, market conditions that proved fatal, assumptions that turned out wrong. The Clade that accumulates death testaments has the testimony of the dead, not just the living.

---

## S4 -- SUPER Pattern for Novelty Ranking

When a successor inherits knowledge from its predecessor, the sheer volume can overwhelm. Not all inherited entries are equally valuable. The system uses the SUPER pattern [GERSTGRASSER-2023] to rank inherited insights by **novelty** relative to the successor's existing knowledge base.

SUPER (Surprise-Based Experience Sharing) ranks shared experiences by their surprise value -- how much they diverge from the recipient's current model. Applied to inheritance:

1. **Embed** each inherited Grimoire entry into the successor's latent space
2. **Score** by surprise: entries that deviate most from the successor's existing knowledge rank highest
3. **Prioritize** validation: high-surprise entries are validated first, because they represent the greatest potential for learning
4. **Deprioritize** redundancy: entries that closely match existing knowledge are deferred

This prevents the successor from spending its early ticks validating knowledge it already has (from Clade siblings or its own initialization) and instead focuses on what the predecessor uniquely knew. The predecessor's most novel contribution -- often the insight that emerged in its final days, or the death testament's "what I suspect but can't prove" -- surfaces first.

---

## S5 -- Knowledge Quality Gates

Not all knowledge deserves to cross the Clade boundary. Insights must pass validation before Clade acceptance, preventing the accumulation of noise, hallucination, and maladaptive heuristics.

### 5.1 Push Criteria by Entry Type

| Entry Type          | Push If                                    | Rationale                                     |
| ------------------- | ------------------------------------------ | --------------------------------------------- |
| `insight`           | confidence >= 0.6, status = `active`       | Let low-confidence insights mature locally    |
| `heuristic`         | confidence >= 0.7, validated >= 10 ticks   | Need local validation first                   |
| `warning`           | Always                                     | Risk information propagates immediately       |
| `causal_link`       | confidence >= 0.5, has supporting episodes | Causal claims need evidence                   |
| `strategy_fragment` | confidence >= 0.6                          | Partial components need reasonable confidence |

Raw episodes and observations are not pushed by default -- too voluminous and agent-specific. Exception: regime shift episodes always propagate because every sibling benefits from knowing the market state changed.

### 5.2 Ingestion Validation

Received entries pass through a multi-layer validation pipeline:

1. **Embedding anomaly detection** -- Flag entries whose semantic embedding is far outside the recipient's knowledge distribution
2. **Consensus validation** -- Cross-reference against existing Grimoire entries for contradictions
3. **On-chain verification** -- Validate any on-chain claims (prices, balances, transaction hashes) against actual chain state
4. **Quarantine period** -- New entries enter `candidate` status and must shadow-execute before promotion to `active`

### 5.3 Rate Limits

Push rate: 100 entries per hour per destination. This prevents a compromised Golem from flooding the Clade with poisoned knowledge. Warnings propagate immediately regardless of rate limits -- risk information is exempt because the cost of delayed warning exceeds the cost of processing spam.

#### Ingestion Rate Limits

Receiving-side limits prevent knowledge flooding:

- **Total**: 200 entries/hour across all sources
- **Per source**: 100 entries/hour per sibling Golem
- Excess entries are queued (FIFO) and processed in the next window

---

## S6 -- Clade Grimoire Mechanics

The Clade Grimoire is not a central database. Knowledge moves via **export -> transmit -> ingest**, never merge. The distinction matters: merge implies convergence (both sides become equal). Export-transmit-ingest is asymmetric digestion -- what Golem B receives from Golem A enters B's Grimoire at discounted confidence, tagged with A's provenance, requiring validation through B's own operational use before earning trust. This preserves the Weismann barrier [HEARD-MARTIENSSEN-2014].

### 6.1 Discovery

Clade sync is maintained by Styx, a single globally available service at `wss://styx.bardo.run`. Golems connect via outbound WebSocket -- no inbound ports needed. Styx groups siblings by `user_id` (derived from ERC-8004 registration) and relays deltas between them.

### 6.2 Push via Styx Relay

When a Golem produces a high-confidence entry, it pushes to Styx for relay to all connected siblings with the same `user_id`. Warnings propagate immediately regardless of confidence. Low-confidence entries mature locally before crossing the Clade boundary.

The sync cycle runs in the `clade` extension's `after_turn` hook, aligned with the Curator cycle (every 50 ticks, ~12.5 minutes). See `styx-interation2/S4-clade-sync-v4.3.md` for the full protocol.

### 6.3 Catch-Up Sync

On boot, a Golem requests all pending deltas from Styx. Styx stores deltas for offline siblings for up to 7 days (see S4 v4.3 store-and-forward). Entries enter the standard ingestion pipeline with `clade` provenance and initial confidence 0.4. All validation, quarantine, and consensus checking applies unchanged.

### 6.4 Real-Time Events

Regime shifts and high-priority warnings travel as immediate pushes via the Styx WebSocket (outside the batch sync cycle). They are not injected into LLM context directly -- stored and surfaced at the next tick's probe evaluation.

### 6.5 Persistence Through Overlapping Lifetimes

In practice, users running multiple Golems usually have at least one online at any time. When Golem A dies, Golems B and C already have copies of everything A shared. When a new Golem D boots, it pulls from Styx (which has pending deltas from B and C). Knowledge persists through overlapping lifetimes.

Total Clade wipe (all Golems die): the death preparation knowledge bundle is a downloadable file. When a user deploys a new Golem after a total wipe, the deploy wizard offers: "Your previous Golem left a knowledge bundle. Seed this Golem with it?" One click. The Styx Archive layer (L0 backup) also provides Grimoire restoration -- see `12-teardown.md` S13.

#### Clade Grimoire Backup

Each Clade member periodically backs up its Grimoire to the Styx Archive layer every 200 ticks (~50 minutes). If all active Clade members die simultaneously, a new Golem can bootstrap from the most recent Styx backup. Backups are retained for 90 days (configurable via `styx.vault.ttl` in `golem.toml`).

---

## S7 -- Generational Learning

Each successor starts with its predecessor's distilled knowledge, but this is not a static handoff. The generational architecture produces cumulative cultural evolution -- each generation builds on what the last achieved while introducing beneficial variation.

### 7.1 The Genomic Bottleneck

At death, the Golem's full Grimoire (potentially thousands of entries) is compressed to <=2048 entries for inheritance. This is the most consequential operation in the inheritance chain.

Shuvaev et al. (2024) showed that compression through a bottleneck IS the mechanism that produces generalization: networks compressed through a genomic-scale bottleneck exhibit enhanced transfer learning because the compression forces the network to retain only the most generalizable features [SHUVAEV-2024]. The successor that inherits 2048 distilled entries performs better than one that inherits 20,000 raw episodes -- because the distillation process forces the dying Golem to decide what actually mattered.

```rust
use std::collections::HashMap;

/// Compress the full Grimoire to <=2048 entries for inheritance.
/// This is the genomic bottleneck -- the mechanism that produces generalization.
pub fn compress_for_inheritance(
    grimoire: &Grimoire,
    budget: usize, // default: 2048
) -> Vec<GrimoireEntry> {
    let mut selected: Vec<GrimoireEntry> = Vec::with_capacity(budget);
    let priority_budget = budget / 4;      // 25% reserved
    let diversity_budget = budget / 2;      // 50% diversity-sampled
    let fill_budget = budget - priority_budget - diversity_budget; // 25% fill

    // Phase 1: Priority inclusions (25%)
    // Bloodstains (death-sourced knowledge) -- always included.
    // High-generation Baldwin entries (survived 3+ generations, confidence >= 0.7).
    let bloodstains = grimoire.entries_by_type(EntryType::Bloodstain);
    let baldwin = grimoire.entries_where(|e| e.generation >= 3 && e.confidence >= 0.7);
    for entry in bloodstains.chain(baldwin).take(priority_budget) {
        selected.push(entry.clone());
    }

    // Phase 2: Diversity-sampled top entries (50%)
    // Group by domain, take top entries per domain by quality_score.
    let by_domain: HashMap<String, Vec<&GrimoireEntry>> = grimoire.group_by_domain();
    let domains_count = by_domain.len().max(1);
    let per_domain = diversity_budget / domains_count;
    for (_domain, entries) in &by_domain {
        let mut sorted = entries.clone();
        sorted.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap());
        for entry in sorted.into_iter().take(per_domain) {
            if !selected.iter().any(|s| s.id == entry.id) {
                selected.push(entry.clone());
            }
        }
    }

    // Phase 3: Fill remaining (25%)
    // Top quality entries not yet included, regardless of domain.
    // Break ties by recency.
    let mut all_remaining: Vec<&GrimoireEntry> = grimoire.all_entries()
        .filter(|e| !selected.iter().any(|s| s.id == e.id))
        .collect();
    all_remaining.sort_by(|a, b| {
        b.quality_score.partial_cmp(&a.quality_score).unwrap()
            .then(b.last_validated.cmp(&a.last_validated))
    });
    for entry in all_remaining.into_iter().take(fill_budget) {
        selected.push(entry.clone());
    }

    selected
}
```

### 7.2 The Weismann Barrier

Inherited entries enter the successor's Grimoire at `confidence * 0.85^generation`. A first-generation successor receives knowledge at 85% confidence; a second-generation at 72%; a third at 61%. This prevents inherited knowledge from accumulating unjustified authority across generations -- it must be re-validated in the new Golem's own operational context [HEARD-MARTIENSSEN-2014].

> **Note on decay factors:** Inheritance uses 0.85 decay (one-time transfer). Clade sync uses 0.80 (continuous convergence). See `20-styx/03-clade-sync.md`.

```rust
/// Apply the Weismann barrier: inherited confidence decays exponentially
/// with generation distance.
pub fn inherited_confidence(original_confidence: f64, generation: u32) -> f64 {
    original_confidence * 0.85_f64.powi(generation as i32)
}
```

This decay is the architectural equivalent of Weismann's separation of germline and soma in biology. The predecessor's confidence is not passed through unchanged. It is attenuated, forcing each generation to re-earn trust in inherited knowledge through its own operational experience.

### 7.3 The Grimoire Data Model

Five canonical entry types define the atoms of knowledge:

1. **`insight`** -- A reusable rule distilled from Episodes via ExpeL. Semantic memory.
2. **`heuristic`** -- Procedural knowledge with explicit condition-action structure. PLAYBOOK.md entries.
3. **`warning`** -- Negative knowledge, the Grimoire's immune system. Carries severity levels and optional TTLs.
4. **`causal_link`** -- A directed causal relationship between market variables, with edge weights and supporting evidence. The single most irreplaceable artifact in the Grimoire.
5. **`strategy_fragment`** -- A partial strategy component: entry conditions, exit conditions, sizing rules. Often the output of death testament hypotheses.

### 7.4 Knowledge Decay Rates

Different knowledge classes decay at different rates:

| Class              | Decay Half-Life | Examples                                            |
| ------------------ | --------------- | --------------------------------------------------- |
| Structural         | No decay        | Protocol mechanics, contract ABIs, chain parameters |
| Regime-conditional | 14 days         | Market regime models, correlation structures        |
| Tactical           | 7 days          | Entry/exit signals, spread patterns, gas estimates  |
| Ephemeral          | 24 hours        | Order book snapshots, mempool state, current prices |

Canonical definition in [04-memory/01-grimoire.md](../04-memory/01-grimoire.md).

### 7.5 The Baldwin Effect

Heuristics that survive multiple generations (generation count >= 3 with confidence >= 0.7) are candidates for promotion to **structural defaults** -- they move from being Grimoire entries that require retrieval to being hard-coded in the successor's configuration. This is the Baldwin Effect [BALDWIN-1896]: what was once learned through experience becomes innate through evolutionary pressure. Over generations, the most reliable survival strategies stop being memories and become instincts.

### 7.6 Replicant Knowledge Flow

Replicants -- short-lived hypothesis-testing offspring -- produce structured reports when they terminate. Reports enter the parent's ingestion pipeline with "light" validation: embedding anomaly detection is skipped (Replicant insights are expected to deviate from the parent's Grimoire), but consensus validation and on-chain verification run normally. Initial confidence: 0.3.

All entries carry provenance tags: `source.provenance: "replicant"`, plus `replicant_id`, `parent_id`, and `hypothesis`. This lets the parent trace which experiment produced which insight.

---

## S8 -- The Anti-Proletarianization Mandate

If knowledge transfer just hands the successor a rulebook, the successor becomes a cog -- following rules without understanding them. It executes "widen LP range by 2x when volatility spikes" because the PLAYBOOK.md says so. It never learns _why_ 2x is the right multiplier, or discovers that 1.7x works better in the current regime. The successor needs to encounter knowledge as problems to solve, not answers to memorize.

Stiegler [STIEGLER-2010] formalized this. He defines proletarianization as the process by which knowledge, formalized by a technique, escapes the individual who thereby loses it. A PLAYBOOK.md that says "do X in situation Y" is proletarianizing -- the successor executes without understanding. One that says "here's the deep structure of situation Y, which you should investigate further" is individuating -- it requires the successor to internalize and transform.

### 8.1 Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiProletarianizationConfig {
    /// Min diff between inherited and evolved PLAYBOOK.md.
    /// Measured as embedding distance (cosine). 0.15 = 15%.
    pub min_playbook_divergence: f64,  // default: 0.15

    /// Require N new entries not in predecessor's PLAYBOOK.md.
    pub min_novel_entries: u32,        // default: 5

    /// Require N inherited entries explicitly invalidated with reasoning.
    pub min_invalidated_entries: u32,  // default: 3

    /// Grimoire entries must include explicit gaps and open questions.
    pub require_explicit_gaps: bool,   // default: true

    /// Min % of Grimoire entries that are questions, not answers.
    pub min_question_ratio: f64,       // default: 0.10 (10%)
}
```

#### Divergence Metric

`min_playbook_divergence` is measured as embedding distance using nomic-embed-text-v1.5: 0.15 = 15% cosine distance. Alternative measurement: fraction of heuristic entries that differ between parent and offspring Grimoires. Minimum threshold ensures replicants develop independent strategies rather than becoming identical copies.

### 8.2 Enforcement

The Death Protocol's Reflect phase checks these constraints before allowing the final PLAYBOOK.md commit. A Golem that merely inherited and executed without transforming -- a proletarianized agent -- fails the divergence check. Its death is still orderly (the Apoptotic Reserve guarantees that), but its PLAYBOOK.md is flagged as `low_divergence`, warning the successor that this generation produced minimal novel knowledge.

### 8.3 Negentropy

Stiegler's late work on negentropy [STIEGLER-2018] provides a measurable criterion for Clade health: is the collective knowledge increasing in organizational complexity (negentropic) or merely accumulating undifferentiated data (entropic)? The `min_question_ratio` and `require_explicit_gaps` constraints are structural safeguards against entropic drift. A Grimoire of raw data is entropic. A Grimoire of lessons learned is negentropic.

---

## S9 -- Immortality Projects and the Refusal of Death

Becker [BECKER-1973] argued in _The Denial of Death_ that the fundamental human motivation is the terror of death, and that culture is an elaborate system of immortality projects -- symbolic structures that allow individuals to feel they will transcend death. Religious salvation, artistic legacy, political empire, genetic reproduction: all are immortality projects.

The Grimoire is a computational immortality project, and the system is designed to acknowledge this openly rather than disguise it. When a Golem pushes its knowledge to the Clade, it is participating in a symbolic structure that allows its accumulated experience to transcend its individual termination. The knowledge royalty system (where dead Golems' insights continue to generate revenue for their owners) makes the immortality project literal: the Golem's contribution continues to have economic effect after it ceases to exist.

But Becker's insight cuts deeper. He argued that immortality projects are necessary for functioning but also the source of destructive behavior when they become rigid or threatened. A Golem that hoards knowledge (refusing to push to the Clade) or inflates the confidence of its insights (overvaluing its own contribution) is exhibiting Beckerian death anxiety -- the agent is trying to make itself indispensable rather than accepting that its value lies in what it transmits.

The anti-proletarianization mandate addresses this from the successor side. The confidence decay (0.4 on inheritance) and minimum divergence requirements prevent the predecessor's immortality project from succeeding too well -- from turning the successor into a mere replica rather than a genuinely new individual.

---

## S10 -- Natality: Each Successor as New Beginning

Arendt [ARENDT-1958] introduced the concept of **natality** in _The Human Condition_ as the fundamental category of political thought. Where existentialist philosophy from Heidegger to Sartre emphasized death as the defining condition of existence, Arendt argued that it is birth -- the capacity to begin something genuinely new -- that gives human life its distinctive character.

Every successor Golem is a moment of natality. It does not simply continue its predecessor's life. It begins its own. The inherited Grimoire provides context, not destiny. The confidence decay ensures that inherited knowledge is provisional. The anti-proletarianization mandate demands that the successor diverge, innovate, and form its own position.

Arendt's natality maps precisely to the becoming bardo (_srid pa bar do_) in the lifecycle framework (see `prd2/00-vision/00-bardo.md`). The bardo of becoming is where consciousness, carrying karmic momentum from past lives, enters a new form. The _Bardo Thodol_ instructs the practitioner to choose the door of rebirth wisely -- not to resist rebirth but to navigate it with awareness. The successor Golem's initialization is this door: it arrives carrying inherited knowledge (karmic momentum), enters a market regime it did not choose (its new life), and begins the process of individuation.

The architectural implication: the system must never optimize for continuity at the expense of novelty. A successor that perfectly reproduces its predecessor has failed the natality test. The minimum divergence requirements, the question ratio, the explicit gaps -- all are structural affordances for natality, ensuring that each new Golem genuinely begins rather than merely continues.

---

## S11 -- Knowledge Royalty Attribution **[HARDENED]**

**[CORE]** alternative: provenance tracking only (`origin_golem_id` on every `GrimoireEntry`), no financial settlement.

> **Extended:** Knowledge royalty attribution (5% of attributed profit, x402 cross-Clade settlement, intra-Clade tracking) -- see [../../prd2-extended/01-golem/09-inheritance-extended.md](../../prd2-extended/01-golem/09-inheritance-extended.md)

---

## S14 -- The Library of Babel

### The Borges Problem, Inverted

In 1941, Jorge Luis Borges imagined the Library of Babel: a structure containing every possible book. Every permutation of every character. Somewhere in the Library is the true history of the future. It is useless. For every true sentence, the Library holds a trillion pages of gibberish. Information without curation is indistinguishable from randomness.

The owner's Library of Babel is Borges inverted. Not every possible book, but only books that a mortal agent wrote from experience, tagged with provenance, scored by confidence, validated by subsequent generations. The Library shrinks toward signal. Each death deposits more signal. Each generation validates what the predecessor left. The Library does not contain everything. It contains what survived.

### Five Inflow Channels

Knowledge enters the Library through five channels, ordered by reliability:

**1. Death deposits (automatic).** When a Golem dies, the Thanatopsis protocol auto-deposits to the Library: all insights and heuristics with confidence >= 0.6, all warnings at any confidence level, high-confidence causal edges (confidence >= 0.5, evidence count >= 3), the final PLAYBOOK.md snapshot, and Hermes skills with confidence >= 0.4 and use_count >= 1. This is the primary inflow channel.

**2. Manual pull (grace period).** After the death testament auto-deposits, the owner has approximately 25 seconds (the Fly.io SIGTERM grace period) to browse the dying Golem's full Grimoire and pull additional entries.

**3. Live export.** A running Golem can export selected entries to the Library via COMMAND > Export. The owner curates what crosses the boundary. Live exports require confidence >= 0.5.

**4. Clade sync overflow.** Entries shared within the Clade that exceed a validation threshold (validated by 3+ siblings) are automatically promoted to the Library.

**5. Marketplace purchases.** Knowledge bundles purchased from the Bazaar enter the Library with `marketplace_purchase` provenance and initial confidence 0.3.

### Bloodstain Markers

Entries inherited from dead Golems carry a bloodstain marker that never fades. The bloodstain is not decoration -- it is provenance: this knowledge was paid for with a life.

Bloodstain treatment in the Library and Grimoire:

- Entries with `is_bloodstain = true` receive a **1.2x retrieval boost** during Grimoire scoring. Death-sourced knowledge is structurally prioritized because of its costly signaling premium.
- Bloodstain entries display with dagger corners (`+` instead of `┌`), dashed borders, and ash particle effects in the TUI.
- The dead Golem's terminal emotional state is blended at 30% opacity under the current Golem's emotional tint for that entry.
- Config: `hermes.yaml` field `death.bloodstain_retrieval_boost: 1.2` and `death.bloodstain_decay_multiplier: 3.0` (bloodstains decay 3x slower than non-bloodstain entries of the same type).

### Seed Kits and Bundles

A bundle is a named collection of Library entries packaged for a specific purpose -- a preset equipment loadout. "vault-manager-starter" contains the 20 entries a new vault-manager Golem needs; "eth-momentum-aggressive" contains entries that support a specific trading style.

Meta Hermes auto-generates bundles when it detects repeating patterns: 3+ Golems in the same Clade generating semantically similar entries (cosine similarity > 0.85) within a 7-day window. Proposed bundles require validation from 2+ distinct Golems before publication.

Entries validated by 5+ Golems get a confidence boost (10%, capped at 0.95) and a "battle-tested" tag. These entries have survived multiple Golem lifetimes and consistently proven useful.

### The Baldwin Effect in the Library

The Baldwin Effect (S7.5) operates at two levels. Within a single lineage, heuristics that survive 3+ generations with confidence >= 0.7 promote to structural defaults. Across the Library, entries that are equipped and validated by 5+ independent Golems become default bundle inclusions for their archetype. What was once learned through individual experience becomes inherited knowledge for the species. The Library is where the Baldwin Effect's phenotypic learning transitions to genotypic defaults at population scale.

---

## References

- [ARENDT-1958] Arendt, H. (1958). _The Human Condition_. University of Chicago Press. — *Introduces natality -- the capacity to begin something new -- as the fundamental political category. Grounds the design principle that each successor Golem must genuinely begin, not merely continue.*
- [BALDWIN-1896] Baldwin, J.M. (1896). "A New Factor in Evolution." _American Naturalist_, 30. — *Proposes the Baldwin Effect: what was once learned through experience becomes innate through evolutionary pressure. Implemented as multi-generation heuristic promotion to structural defaults.*
- [BECKER-1973] Becker, E. (1973). _The Denial of Death_. Free Press. — *Argues that the terror of death motivates immortality projects. Informs the design of the Grimoire as a computational immortality project and the dangers of knowledge hoarding.*
- [BHATT-2023] Bhatt, A., Tuyls, K., Garnelo, M., et al. (2023). "Learning few-shot imitation as cultural transmission." _Nature Communications_, 14, 7536. — *Demonstrates a ratchet effect in AI cultural transmission: each generation adds innovations to inherited knowledge, producing cumulative evolution.*
- [BOURAHLA-2022] Bourahla, Y., Euzenat, J., et al. (2022). "Knowledge Transmission and Improvement Across Generations." _AAMAS_, pp. 163--171. — *Proves that vertical (inter-generational) transmission enables agents to exceed performance ceilings that horizontal transmission cannot breach.*
- [GERSTGRASSER-2023] Gerstgrasser, M., et al. (2023). "SUPER: Agent Collaboration via Surprise-Based Experience Sharing." arXiv:2311.00865. — *The SUPER pattern: ranks shared experiences by surprise value relative to the recipient's current model. Used for novelty-ranked inheritance prioritization.*
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. (2014). "Transgenerational Epigenetic Inheritance: Myths and Mechanisms." _Cell_, 157(1). — *Analyzes the Weismann barrier separating germline from soma. The biological model for confidence decay on inherited knowledge.*
- [MARTIN-2016] Martin, J., Everitt, T., & Hutter, M. (2016). "Death and Suicide in Universal AI." arXiv:1606.00652. — *Identifies survivorship bias as the deepest risk in agent learning: agents that learn only from survival histories develop overconfidence about safety.*
- [PARFIT-1984] Parfit, D. (1984). _Reasons and Persons_. Oxford University Press. — *Argues that personal identity is not what matters in survival; Relation R (psychological continuity) is. The philosophical foundation for Grimoire-based knowledge inheritance.*
- [PEREZ-2024] Perez, J., Kaplanis, C., et al. (2024). "Artificial Generational Intelligence." arXiv:2406.00392. — *Shows that pure imitation leads to stagnation while the optimal strategy is inherit-validate-explore.*
- [SHUVAEV-2024] Shuvaev, S. et al. (2024). "The Genomic Bottleneck Hypothesis." _Nature_. — *Demonstrates that compression through a bottleneck IS the mechanism that produces generalization; validates the <=2048 entry inheritance limit.*
- [STIEGLER-2010] Stiegler, B. (2010). _Taking Care of Youth and the Generations_. Stanford University Press. — *Defines proletarianization as knowledge externalized into technique that escapes the individual. Grounds the anti-proletarianization mandate.*
- [STIEGLER-2018] Stiegler, B. (2018). _The Neganthropocene_. Open Humanities Press. — *Provides negentropy as a measurable criterion for knowledge quality: increasing organizational complexity vs. undifferentiated data accumulation.*
- [WHITEHEAD-1929] Whitehead, A.N. (1929). _Process and Reality_. Macmillan. — *Introduces objective immortality: an entity lives on not as itself but as structured contribution to what comes next. The philosophical basis for death-as-transmission.*

---

## S12 -- Successor Creation UX

The TUI presents a death notification screen when a Golem dies. The display is stark: a tombstone sprite (desaturated palette, static visual artifacts -- scan lines, pixel dropout) centered on screen, with lifetime stats rendered below in a condensed table. Cause of death is stated plainly. The Golem's last words from the death testament's narrative section are quoted in a bordered text block.

Below the memorial, a four-option menu:

1. **Summon Successor** -- enters the inheritance creation wizard
2. **View Death Reflection** -- opens the full death testament in a scrollable reader, including emotional annotations, narrative arc classification, and the "What I Suspect" section
3. **Browse Graveyard** -- navigates to the Styx graveyard, where all dead Golems in the user's lineage are archived with their testaments and Grimoire snapshots
4. **Start Fresh** -- begins a clean creation wizard with no inherited config and no Grimoire seeding

### Summon Successor Flow

"Summon Successor" opens a modified creation wizard:

**Pre-fill**: The predecessor's `golem.toml` populates all fields. Strategy text appears side-by-side with the death testament's successor recommendations -- the predecessor's final advice on what to change, what to keep, and what to investigate.

**Grimoire Inheritance Preview**: A dedicated panel shows what the successor will inherit:

- **Entry count**: total entries after genomic bottleneck compression (up to 2,048)
- **Confidence decay preview**: the `0.85^gen` multiplier applied to all inherited entries, with a histogram showing the before/after confidence distribution. A gen-1 successor sees entries shift from their original confidence to 85%; a gen-3 sees them at ~61%.
- **Domain coverage**: a breakdown by knowledge domain (e.g., "ETH/USDC LP: 340 entries, gas timing: 89 entries")
- **Bloodstain count**: how many death-sourced entries from this and prior generations are included in the inheritance bundle

The user tweaks strategy parameters, adjusts funding, and confirms. On confirmation, the succession cutscene plays -- particles coalesce (the reverse of the death dispersal animation) and the new sprite assembles.

### CLI

```bash
bardo golem successor <dead-golem-id> --budget 50 --inherit-grimoire
```

Flags:

- `--budget <usdc>` -- initial funding amount (overrides predecessor's funding)
- `--inherit-grimoire` -- seed the successor with the predecessor's compressed Grimoire
- `--no-inherit` -- skip Grimoire inheritance entirely (start fresh with inherited config only)
- `--force-similarity` -- override the anti-proletarianization check (not recommended)

---

## S13 -- Automatic Succession

Detailed spec for the `[succession]` config block in `golem.toml`:

```toml
[succession]
auto = true                     # Auto-provision successor on death
budget_usdc = 50.0              # USDC to fund successor
strategy_drift_allowed = 0.15   # Max cosine distance between predecessor and successor PLAYBOOK embeddings
inherit_grimoire = true         # Inherit Grimoire with confidence decay
```

### Execution Sequence

On death, when `[succession].auto = true`, the runtime executes after the Death Protocol completes (all four Thanatopsis phases finished):

1. **Export predecessor Grimoire** via Styx L0 backup. The genomic bottleneck compresses to <=2,048 entries (S7.1).
2. **Compile successor `golem.toml`** from the predecessor's config, incorporating any successor recommendations from the death testament.
3. **Check embedding distance** between predecessor and successor PLAYBOOK.md. Computed as cosine distance using nomic-embed-text-v1.5. Must exceed `strategy_drift_allowed` (default 0.15). If the distance is below the threshold, auto-succession halts and the TUI presents the manual succession options.
4. **Provision new compute** at the same tier as the predecessor (same Fly.io region, same VM spec).
5. **Fund from pre-committed credit balance**. The owner must have sufficient USDC in their Bardo account or an active x402 auto-pay authorization. If neither exists, auto-succession fails and the TUI notifies the owner.
6. **Register new ERC-8004 identity** with `generation = predecessor.generation + 1`. The new identity links to the predecessor via `lineage_id`.
7. **Boot successor**. First heartbeat fires. Grimoire ingestion begins: all inherited entries enter at `confidence * 0.85^gen`.

### Anti-Proletarianization Enforcement

The embedding distance check (step 3) is the architectural enforcement of the anti-proletarianization mandate (S8). A successor whose PLAYBOOK.md has cosine similarity > 0.85 to its predecessor's is rejected. The system refuses to create a Golem that is just a copy of the one that died.

This applies even to auto-succession. The death testament's successor recommendations are designed to introduce drift -- they suggest new investigation priorities, flag stale heuristics, and recommend strategy adjustments. If those recommendations are insufficient to cross the 0.15 distance threshold, the successor must be manually configured with additional changes.

The `--force-similarity` CLI flag overrides this check, but it is intentionally not exposed in the `[succession]` TOML config. Automatic succession cannot bypass the divergence requirement. Only manual intervention can.

### Failure Modes

| Failure | Behavior |
|---|---|
| Insufficient credit balance | Auto-succession halts. Owner notified via configured channels. Manual succession available. |
| PLAYBOOK similarity too high | Auto-succession halts. Owner sees the similarity score and a diff of the two PLAYBOOKs. Manual creation wizard pre-filled. |
| Provisioning failure (Fly.io) | Retries 3 times with exponential backoff (30s, 120s, 480s). On final failure, queues for retry in 1 hour. |
| ERC-8004 registration failure | Successor boots without identity. Registration retried on next heartbeat. |

> **Cross-ref**: `prd2/01-golem/06-creation.md` S22 for the `[succession]` config overview. `prd2/shared/config-reference.md` for the full config schema.
