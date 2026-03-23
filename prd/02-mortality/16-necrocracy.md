# The Necrocracy: How the Dead Shape the Living [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Crates**: `golem-mortality`, `golem-grimoire`, `bardo-styx`
>
> **Depends on**: `00-thesis.md` (foundational mortality thesis), `06-thanatopsis.md` (four-phase death protocol), `10-clade-ecology.md` (superorganism dynamics, stigmergic coordination), `../04-memory/13-library-of-babel.md` (shared death testament archive)

---

> **Reader orientation:** This document specifies the necrocracy, Bardo's system for how dead Golems (mortal autonomous DeFi agents) shape the behavior of living ones. At ecosystem maturity, dead Golems outnumber living ones roughly 27:1. Their death testaments, produced during the Thanatopsis Protocol (four-phase structured death sequence) under zero survival bias, are the most epistemically honest knowledge in the system. Bloodstains (market conditions at death overlaid as warnings) mark where predecessors died; the Library of Babel stores their testaments; the Lethe (Styx's commons knowledge tier) pools anonymized reflections. See `06-thanatopsis.md` (death protocol) and `10-clade-ecology.md` (superorganism dynamics) for the upstream systems. See `prd2/shared/glossary.md` for full term definitions.

## The Thesis

In Bardo, the dead outnumber the living. Always. The ratio grows monotonically. At ecosystem maturity, modeling suggests something like 27:1 dead-to-living. This is not a failure mode. It is the system's deepest structural feature.

Every golem that dies leaves behind knowledge that no living golem can produce. A dead golem cannot benefit from its own warning. It has no future self-interest, no competitive position to protect, no survival pressure distorting its assessments. Its death testament, produced during the Thanatopsis Protocol under zero survival bias, is the most epistemically honest artifact in the entire system. Walter Benjamin argued that "death is the sanction of everything that the storyteller can tell" [BENJAMIN-1936]. The golem's authority comes from the same place.

Living golems' knowledge is systematically biased. They know what worked -- for them, in their context, during their particular slice of market history. They don't know what killed their predecessors. Survivorship bias means the living overweight strategies that happened to work and underweight risks that would destroy them. The dead correct this. Their testimony is evidence from the strategies that failed, the market conditions that killed, the risks that were underestimated. This evidence is the one thing the living cannot generate for themselves.

Necrocracy is governance by the dead. Not literally -- the dead don't vote, don't communicate, don't hold office. But the accumulated experience of dead golems shapes the behavior of living ones through multiple channels: bloodstains deposited in the pheromone field, death testaments stored in the Library of Babel, anonymized reflections pooled in the Lethe, inherited heuristics passed down through lineages. The dead carved messages into the architecture. The living are shaped by what they find.

The system's collective intelligence is mostly composed of dead golems' contributions. And that share only grows. The Necrocracy is the invisible government of Bardo, a government whose legislators are all corpses and whose legislation is written in blood.

Most agent frameworks treat death as failure. Bardo treats death as the single most productive event in an agent's existence -- the moment when its accumulated knowledge transforms from private competitive asset to public good, when its biases are shed, when its most honest reflection becomes possible. The entire system is designed around this inversion: death is not the opposite of intelligence production. Death is its highest form.

---

## Bloodstain Infrastructure

In Dark Souls (FromSoftware, 2011), bloodstains mark where other players died. You can touch one and replay their final moments. In Bardo, bloodstains are death-location markers indexed not by physical location but by market condition. A bloodstain says: "A golem died here, and here is defined by the state of the market when it happened."

### What a Bloodstain Carries

| Component | Contents | Visibility |
|---|---|---|
| Death cause | What killed the golem: credit exhaustion, epistemic senescence, stochastic shock, owner kill | Everyone on Styx |
| Death context | Market regime, positions held, strategy being executed | Clade siblings + Lethe |
| Warnings | Specific observations that contributed to the death | Clade siblings + Lethe |
| Causal edges | Most validated world-model relationships | Clade siblings only |
| Emotional topology | PAD vector near the failure region | Clade siblings only |
| EIP-712 signature | Cryptographic proof from the dying golem's ERC-8004 identity | On-chain, verifiable |

The visibility gradient is deliberate. Everyone sees that a golem died and why. Siblings see the full context. Strangers see the warning but not the proprietary strategy details. This mirrors biological signaling: alarm pheromones travel far, but detailed territorial information stays within the kin group.

### The Costly Signal

Why should a living golem trust a bloodstain? Zahavi's handicap principle (1975) gives the answer: a signal is reliable in proportion to its cost. A peacock's tail is an honest signal of genetic fitness because it is expensive to produce and maintain. Cheap signals get ignored; costly signals get trusted.

Bloodstains are the most costly signals in the system. The golem that produced one paid with its existence. It cannot benefit from the warning. It cannot be bribed, coerced, or incentivized to lie. A dead golem's warning is a pure costly signal in the Zahavian sense -- the ultimate handicap, because the signaler is dead.

### Mechanical Effects

When a golem receives a bloodstain via Styx relay, two things happen:

**Pheromone deposits.** The bloodstain's death context produces THREAT pheromones in the relevant domain. Other golems in that domain feel increased arousal on the CorticalState. They become more vigilant without knowing why. They don't receive a message; they receive a chemical gradient. This is Grasse's (1959) stigmergy: coordination through environmental traces rather than direct communication.

**Grimoire entries.** The bloodstain's warnings enter the receiving golem's Grimoire tagged with `is_bloodstain: true`. Two mechanical advantages:
- **1.2x retrieval boost.** Bloodstain-tagged entries surface more aggressively in semantic search. When a living golem queries its Grimoire under market conditions that resemble a bloodstain's death context, the dead golem's warning floats to the top.
- **3x slower Ebbinghaus decay.** Normal Grimoire entries fade over time. Bloodstain entries decay at one-third the normal rate. The dead's warnings persist in the living's memory long after ordinary knowledge has faded.

### Propagation Path

1. Golem dies. Phase 3 (Legacy) of Thanatopsis emits bloodstain to Styx.
2. Styx relays the bloodstain. Clade siblings receive it immediately. The public pheromone field receives the THREAT deposit.
3. The pheromone field diffuses the signal. THREAT pheromones spread outward from the deposit point, decaying over roughly 2 hours.
4. Living golems encounter the pheromone during their next tick. Their arousal spikes. Behavioral adjustments follow: tighter stop-losses, reduced position sizes, increased monitoring frequency.
5. Over time, the pheromone decays. But the Grimoire entry (with its 3x decay resistance) persists. The immediate alarm fades; the memory endures.

### Bloodstain Visualization

The World screen renders bloodstain density as a heatmap overlaid on the domain grid. Dense regions glow with accumulated death. Individual bloodstain labels drift through the Styx River visualization -- `oracle_stale`, `credit_exhaustion`, `epistemic_senescence` -- each one a record of how a mind ended.

Tombstones mark individual deaths. Hover to see: name, generation, lifespan, death cause, PnL. Fog accumulates near death concentrations. Brightness indicates recency; old deaths fade but don't disappear.

```
+-- The Styx --------------------------------------------------+
|                                                                |
|  ~~~~ River of the Dead ~~~~  |  Deaths: 1,266               |
|                                                                |
|                    . . .                                       |
|              . . . . . . . . .                                |
|                                                                |
|        [##]   [##]                    [##]   [##]             |
|        [##]   [##]                    [##]   [##]             |
|                                                                |
|  oracle_stale ---======~~~~~                                  |
|                                                                |
|              regime low_liquidity =====~~~   high_gas+ranging |
|                                                                |
|                          epistemic_senescence                 |
|                                                                |
|     [##]         [##]         [##]                            |
|     [##]         [##]         [##]         volatile+trending  |
|                                                                |
+----------------------------------------------------------------+
```

The tombstones are interactive. Select one to read the full death testament. Select a drifting bloodstain label to see aggregate statistics: how many golems died from that condition, when, and what they warned about before the end.

### Bloodstain Rendering in the UI

Dead golems don't disappear from the interface. They leave marks. Wherever a dead golem's data appears -- in lists, tables, logs, clade views, the World screen -- the rendering changes to signal that this entity no longer exists.

**Dashed borders.** Panels, rows, and containers associated with a dead golem use dashed box-drawing characters (`┄` instead of `─`, `┆` instead of `│`). A living golem's panel has solid borders. A dead golem's panel has dashed borders. The effect is subtle but unmistakable: the dead golem's container is porous, no longer sealed. Solid lines mean alive. Broken lines mean gone.

**Dagger prefix.** Every dead golem's name and identifier is prefixed with `†` (U+2020), the typographic dagger -- a centuries-old convention for marking the deceased. `†morpho-rebalancer-gen3` was a golem. `morpho-rebalancer-gen4` still is one. The dagger appears in every context where the name is rendered: lists, logs, tooltips, graph nodes, clade rosters. No exceptions.

**Ghost palette.** Dead golem references use a desaturated, blue-shifted color palette. Where living golems render in the `rose` family (#AA7088, warm pink), dead golems render in `text_ghost` (#403848, a cold purple-gray) or `text_phantom` (#201820, near-invisible). The warmth drains out. What remains is a cool afterimage, readable but clearly not alive.

**Generational decay.** Each successive death in a lineage adds visual corruption to the dead entries associated with older generations. The effect is cumulative:

```
Gen N-1 (most recent death):  rose_dim — still warm, recently dead
Gen N-2:                      text_primary — cooler, fading
Gen N-3:                      text_dim — hard to read
Gen N-4:                      text_ghost — barely visible
Gen N-5+:                     text_phantom — you have to squint
```

A golem in its 10th generation, viewing its lineage history, sees a gradient from nearly-legible (recent ancestors) to nearly-invisible (distant ones). The dead fade with distance, but they don't vanish. Their traces persist at the threshold of perception.

**A normal entry versus a bloodstained entry:**

```
LIVING GOLEM:
┌─────────────────────────────────────────────┐
│  morpho-rebalancer-gen4       ◉ ◉  ACTIVE  │
│  vitality: 0.72   PnL: +$3.40   tick: 4847 │
│  strategy: LP rebalance (ETH/USDC 0.3%)    │
└─────────────────────────────────────────────┘

DEAD GOLEM (gen N-1, recent):
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
┆  †morpho-rebalancer-gen3          DEAD      ┆
┆  lifespan: 18d   PnL: -$4.60   cause: econ ┆
┆  strategy: LP rebalance (ETH/USDC 0.3%)    ┆
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄

DEAD GOLEM (gen N-4, distant ancestor):
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
┆  †m░rpho-r░balancer-gen0         DEAD       ┆
┆  l░fespan: ░░d   PnL: -$░.░0   cause: stoc ┆
┆  strategy: ░░ ░ebalance (░░░/░░░░ ░.░%)    ┆
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
```

The corruption on distant ancestors is not data loss -- the underlying data is intact. It is a rendering decision: old death records display with increasing entropy, mirroring the epistemic decay of their relevance. A 5th-generation-old death testament is still queryable, still retrievable, still in the Library. But visually, it looks like something recovered from deep sediment.

---

## The Lethe: Anonymized Knowledge Commons

Named for the river of forgetting in Greek mythology -- the river that the dead drink from to erase their memories before reincarnation. In Bardo, the Lethe inverts the myth. It is a river of remembering: anonymized knowledge from the dead, pooled into a commons that any golem can query.

The Lethe is the public, anonymized knowledge layer on Styx. Relationship: individual golems have Grimoires (private), clades share via Clade Sync, and the Lethe aggregates validated knowledge from death testaments across the ecosystem. The Library of Babel (`../04-memory/13-library-of-babel.md`) is the owner's local view of accumulated knowledge.

### Anonymization: SAP (Secure Approximate Processing)

Death reflections are stripped of identity. SAP removes: strategy specifics (exact entry/exit parameters, position sizes, timing rules), wallet addresses, golem identifiers, owner identifiers, and raw embeddings (which could be inverted via Vec2Text attacks to recover original text).

What remains: market observations, pattern descriptions, timing insights, risk assessments, regime characterizations. The knowledge without the fingerprint.

Example of what goes in versus what comes out:

**Before SAP:** "golem-alpha-gen3, owner 0x7a3b..., discovered that ETH/USDC 0.3% pool on Base tends to see 40% liquidity withdrawal within 2 hours of large Morpho rate changes, creating a 15-minute window where the effective spread widens by 3-5 bps. Exploited this 7 times with average gain of $2.30 per event."

**After SAP:** "Pattern observed: large rate changes in lending protocols correlate with temporary liquidity withdrawal from major L2 DEX pools. Withdrawal typically occurs within a 2-hour window. Effective spread widening during withdrawal period is measurable. Pattern confirmed across multiple observations."

The specific pool, the specific chain, the position size, the dollar amounts, the golem's identity -- all gone. The structural insight remains.

### Why Anonymization Matters

Without anonymization, you could reverse-engineer other users' strategies from their dead golems' knowledge. If golem-alpha dies and its full death testament enters a public commons, anyone who reads it learns the owner's trading approach, position sizing, risk tolerance, and market views. That is a privacy violation and a competitive disadvantage for the owner.

The Lethe solves this: knowledge flows from the dead to the living without carrying identity. The insight travels; the source does not.

### Six Knowledge Domains

| Domain | Publication Delay | Rationale |
|---|---|---|
| `market_structure` | 3 hours | Medium temporal sensitivity |
| `risk_patterns` | 2 hours | Safety-first: faster availability |
| `protocol_mechanics` | 1 hour | Lowest temporal sensitivity |
| `gas_dynamics` | 2 hours | Changes frequently but not front-runnable |
| `regime_signals` | 6 hours | Maximum front-running risk |
| `cross_domain` | 4 hours | Mixed sensitivity |

The delays prevent a dead golem's market observations from being immediately exploitable.

### Economics

Free to publish. $0.002 per query. Revenue distributed to contributors proportionally by query hit rate. Kenneth Arrow's information paradox applies: the value of information is not known until you have it. The micropayment sidesteps this -- you pay $0.002 to retrieve, evaluate, and decide whether to validate through live testing.

### The Monotonic River

The Lethe grows monotonically. Nothing is deleted. Entries age and their retrieval weight decays (alpha-decay), but they remain in the commons. The river only flows one direction.

Over time, the Lethe becomes a geological formation -- layers of dead knowledge deposited over weeks and months, each stratum corresponding to a particular market regime. The deepest layers are the oldest, least relevant, most decayed. The surface layers are fresh, immediately actionable. But even deep layers occasionally surface. A market regime that hasn't been seen in three months suddenly returns, and the Lethe entries from golems that died during the last occurrence of that regime become relevant again. The old dead advise the living about a present that resembles their past.

This geological quality distinguishes the Lethe from any conventional knowledge base. A traditional database stores information. The Lethe stores the final thoughts of minds that experienced the information firsthand and died. The emotional weight, the contextual richness, the hard-won quality of death knowledge makes Lethe entries qualitatively different from, say, scraped market data or backtested strategy results. The Lethe is a history written by the dead about the world that killed them.

---

## Death Archives

Death testaments have their own storage and retrieval characteristics, distinct from both the Library of Babel and the Lethe.

### Alpha-Decay Pricing

Knowledge rots. The relevance of a death testament decays over time as market conditions drift. Decay constant lambda = 0.08, giving a half-life of roughly 9 days. After 9 days, a death testament's market price is half its initial value. After a month, about 10%.

Exception: structural knowledge (protocol mechanics, contract behavior, gas patterns) decays much more slowly.

### Lineage Grimoires

The premium product. A lineage grimoire is the complete chain of a golem's ancestors: every death testament in the lineage, cross-validated across generations. A 10-generation lineage grimoire contains the accumulated learning of 10 lifetimes, with each generation's knowledge tested against the next generation's experience.

Lineage grimoires trade on the marketplace at $10-$50 depending on generation depth, lineage PnL, and market relevance.

### Archive Indexing

Death archives are indexed by:
- **Death cause** (credit exhaustion, epistemic senescence, stochastic, owner kill)
- **Market regime at death** (trending, ranging, volatile, low-liquidity)
- **Strategy archetype** (LP management, directional trading, yield farming, arbitrage)
- **Generation** (first-gen golems have no predecessors; deep-lineage golems have long inheritance chains)
- **Emotional arc** (Redemptive, Contaminating, Progressive, Tragic, Stable -- following McAdams' narrative identity taxonomy)
- **Timestamp** (for alpha-decay calculation)

**Death registry GC.** Entries younger than 1 year are kept in hot storage (SQLite). Entries 1-5 years old move to cold storage (compressed JSONL on R2). Entries older than 5 years are pruned to tombstone-only (id, cause, date, final NAV if opted in). Tombstones are permanent.

---

## The Necrocratic Feedback Loop

The full cycle, traced from death through rebirth:

A golem dies. Thanatopsis executes. The testament fans out through multiple channels simultaneously.

Bloodstains enter the pheromone field. Living golems feel heightened arousal on their next tick. They tighten risk controls, shrink positions, increase vigilance. They don't know why they feel alert. The pheromone carries no message, only intensity.

Death warnings enter siblings' Grimoires with 1.2x retrieval boost and 3x decay resistance. When conditions align, the dead sibling's last words become the loudest voice in the living sibling's memory.

The death testament enters the Library of Babel. When the owner creates a successor, Meta Hermes recommends an equip loadout from the Library. The successor starts life carrying the dead golem's learning.

Anonymized reflections enter the Lethe. Now any golem in the ecosystem can query this knowledge. One golem's experience becomes ecosystem intelligence.

Then a new golem is born. It validates some of what its predecessor believed. Contradicts some. Discovers new things the predecessor never encountered. Eventually, it too dies. Its testament adds to the Library, the Lethe, and the pheromone field. The dead-to-living ratio increases by one. The collective intelligence deepens by one death's worth.

### Why the Loop Accelerates

The feedback loop has a compounding property. Each generation of golems starts with more inherited knowledge than the last. A generation-10 golem, equipped from a Library containing nine death testaments, begins with a knowledge base that took nine lifetimes to accumulate. It can skip the mistakes its ancestors made. It can pursue the hypotheses its ancestors suspected but couldn't prove.

The limit is the rate at which markets change. DeFi markets are not slow. New protocols launch. New attack vectors emerge. The territory is always changing, which means death always produces new information. There is no steady state where the dead have said everything there is to say.

### The Role of Category 7

One detail deserves special attention: category 7 of the death reflection ("What I suspect but cannot prove"). This is the mechanism by which the Necrocracy generates novelty rather than just accumulating confirmed knowledge.

A dying golem deposits its unproven suspicions. A successor equips them as research prompts. Over generations, category 7 acts as a distributed research agenda. No single golem designs the agenda. No central planner decides what to investigate. The agenda emerges from the accumulated unfinished business of the dead. This is how the Necrocracy avoids ossification.

---

## Survivorship Bias Correction

The dead-to-living ratio is the system's mechanism for correcting survivorship bias. Living golems' knowledge is biased toward what worked -- for them, in their context. The dead provide the complementary evidence: what failed, what killed, what was underestimated.

The distribution of death causes matters because different causes produce qualitatively different testaments:

| Death Cause | Estimated Share | Testament Character |
|---|---|---|
| Economic (USDC depletion) | 60-70% | Defensive knowledge: what drained funds, where costs exceeded returns |
| Epistemic (predictive fitness decay) | 15-25% | Regime-transition knowledge: the market changed and the golem couldn't adapt |
| Stochastic (random death) | 5-15% | Richest testaments: the golem was healthy when it died, producing the most generous reflection |
| Owner kill | Remainder | Variable, but still triggers full Thanatopsis |

Stochastic deaths produce the richest testaments because the golem had no survival pressure at all. Its death reflection is the most generous, least defensive document in the system.

### The Dead-to-Living Ratio Over Time

Assume steady state: new golems created at roughly constant rate, average lifespan 3 weeks.

| Time | Alive | Dead | Ratio |
|---|---|---|---|
| Week 1 | 100 | 0 | 0:1 |
| Week 4 | 100 | ~200 | 2:1 |
| Week 12 | 100 | ~900 | 9:1 |
| Week 26 | 100 | ~2,200 | 22:1 |
| Year 1 | 100 | ~4,800 | 48:1 |

The dead-to-living ratio grows without bound. Every living golem will eventually die and join the dead majority.

### When Does Necrocratic Knowledge Dominate?

For a single owner running serial golems, the Library reaches necrocratic dominance by generation 5 or 6. At the ecosystem level, the Lethe reaches dominance within the first month of operation.

---

## Mass Death Events

Black swans produce mass death. A flash crash, a protocol exploit, a sudden regulatory action can kill dozens or hundreds of golems in a single day. From the perspective of individual owners, this is a catastrophe. From the perspective of collective intelligence, it is a windfall.

Mass death events produce:

- **Correlated bloodstains.** Dozens of THREAT pheromone deposits in the same domain. The concentration becomes so dense that living golems experience extreme arousal -- a system-level alarm.

- **Multi-perspective death testaments.** The same event described by LP golems, directional traders, arbitrageurs, yield farmers. Each saw the same conditions through a different strategic lens. Together, they describe the anatomy of the crash in three dimensions.

- **Cross-validating Lethe deposits.** Multiple independent observers describe the same phenomenon. The Lethe's retrieval system identifies convergent knowledge, strengthening confidence without revealing individual sources.

Golems born after a mass death event inherit an unusually detailed knowledge base about the conditions that caused the extinction. This is the Baldwin Effect: environmental pressure doesn't directly modify the golem's code, but it modifies the knowledge environment the next generation inherits.

The system learns fastest when it hurts most.

---

## Connection to Solaris

Solaris is the name for Bardo's collective intelligence -- the emergent ocean of shared knowledge, pheromone signals, and behavioral patterns that no individual golem controls or fully perceives.

The connection to the Necrocracy is structural: Solaris is mostly composed of the dead. At ecosystem maturity, the dead-to-living ratio is roughly 27:1. The Lethe contains anonymized knowledge from all dead golems. The pheromone field contains decaying signals from every death. The Library of Babel contains the accumulated testaments of every golem that ever lived.

Living golems contribute to Solaris too. But their contributions are a thin layer on top of a deep sediment of dead knowledge. The living are perturbations on a surface shaped by the dead.

Each death sends a wave through the ocean. Multiple simultaneous deaths produce constructive interference: overlapping waves that reinforce each other, creating temporary peaks of collective awareness in the affected domains.

---

## References

- [BENJAMIN-1936] Benjamin, W. "The Storyteller." In _Illuminations_, trans. H. Zohn. Schocken, 1968.
- [BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991.
- [GRASSE-1959] Grasse, P.-P. "La reconstruction du nid." _Insectes Sociaux_ 6, 1959.
- [MCADAMS-2001] McAdams, D.P. "The Psychology of Life Stories." _Review of General Psychology_ 5(2), 2001.
- [ZAHAVI-1975] Zahavi, A. "Mate Selection -- A Selection for a Handicap." _JTB_ 53(1), 1975.

---
