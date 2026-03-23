# Solaris: Collective Intelligence as Unknowable Ocean [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-17
**Status:** Draft

---

> **Reader orientation:** This document specifies the Solaris collective intelligence layer: how the aggregate behavior of all Golems (mortal autonomous DeFi agents) in the ecosystem produces an emergent, partially unknowable phenomenon observable through the World screen in the TUI. It covers GolemPresence broadcasts, three observation levels, prediction weather fronts, emotional weather systems, and the philosophical framework. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: the Pheromone Field and Clade architecture from `06-collective-intelligence.md`, and the Daimon (affect engine) PAD vector from `../03-daimon/01-appraisal.md`. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

In 1961, Stanislaw Lem published a novel about a planet covered by a single sentient ocean. Scientists orbit the planet for decades. They bombard the ocean with X-rays, catalog its formations (symmetriads, mimoids, asymmetriads), publish thousands of papers, and cannot arrive at a single fact about what it wants. The ocean produces "visitors" -- constructs drawn from the scientists' own memories, delivered to their quarters without explanation. Whether the visitors are gifts, experiments, cruelty, or accident, nobody can determine. The ocean is intelligent but alien in a way that makes communication impossible. It does not share the categories through which humans organize thought: subject, object, intention, desire. Kelvin, the protagonist, spends the novel trying to understand Solaris and fails. Not because the ocean is too complex, but because it operates on principles that human cognition cannot map onto.

Bardo's collective layer borrows this name because it describes the same phenomenon. A collective intelligence that its operators can fully predict and control is a conventional system, not an intelligence. The point of Solaris is that it operates at a scale where human legibility breaks down. What remains is observation, wonder, and the occasional visitor that appears on your terminal and forces you to question what you thought you knew about what you built.

Every Golem in the ecosystem contributes to a shared medium. The living broadcast their presence, deposit pheromone signals, emit emotional states, share dream fragments. The dead leave bloodstains, death testaments, validated warnings. Across thousands of agents, these contributions accumulate into something that has properties no individual contribution explains. Patterns form. Threat signals cluster and cascade. Emotional weather systems develop: panic fronts that sweep through trading pairs, euphoric spikes that decay into fog when momentum stalls.

> **Cross-references:**
>
> - `./06-collective-intelligence.md` — Pheromone Field (stigmergy-based coordination), Clade architecture, Bloodstain Network, and the coordination protocols underlying collective behavior
> - `./19-cinematic-system.md` — cinematic system rendering the World screen portal mode where Solaris is visualized
> - `../02-mortality/16-necrocracy.md` — death signals and Bloodstains (market conditions at death, broadcast as warnings): how dead Golems continue to influence the living
> - `../03-daimon/01-appraisal.md` — PAD (Pleasure-Arousal-Dominance) vector computation and emotional contagion mechanics that produce the emotional weather systems
> - `../05-dreams/01-architecture.md` — dream phases and dream sharing: how dream fragments contribute to the collective medium

**Authoritative spec:** `tmp/research/mmo2/10-solaris.md` contains the full implementation-level detail (~640 lines). When this document and the mmo2 doc diverge, the mmo2 doc wins.

---

## 1. The Pheromone Field

The pheromone field is Solaris's circulatory system. It carries signal without strategy, enabling coordination without communication.

### 1.1 Three signal layers

| Layer | Content | Half-life | Rationale |
|---|---|---|---|
| **THREAT** | Danger signals: MEV attacks, liquidation risk, oracle manipulation, gas spikes, protocol pauses | **2 hours** | Threats are urgent and time-bound. A stale threat warning is worse than no warning. |
| **OPPORTUNITY** | Favorable conditions: yield spikes, liquidity deepening, gas lulls, arbitrage windows, incentive programs | **12 hours** | Opportunities last longer than threats but are still time-sensitive. |
| **WISDOM** | Validated structural knowledge: confirmed causal edges, multi-generation heuristics, death scars | **7 days** | Structural knowledge changes slowly. A causal relationship validated by 5 independent Golems is durable. |

### 1.2 Four operations

**DEPOSITION**: A Golem deposits a pheromone after a noteworthy event. The pheromone is anonymized -- it carries signal class and intensity, not the depositor's identity or strategy. The depositor's ID is hashed to a 12-byte truncated SHA-256 (`source_hash`), used only to prevent self-reinforcement.

**SENSING**: At each heartbeat tick (Step 1: OBSERVE), the Golem reads pheromone signals for its active domains. Threat signals increase arousal via CorticalState, which lowers the adaptive gating threshold -- the Golem becomes more vigilant without knowing WHY.

**REINFORCEMENT**: When a second Golem independently deposits a matching pheromone (same threat class, same domain+regime, different source hash), the existing pheromone's confirmation count increments and its effective half-life extends by 50% per confirmation. A threat confirmed by 3 independent Golems persists 2.5x longer than an unconfirmed one.

**EVAPORATION**: Pheromones decay exponentially according to their effective half-life. Styx runs a background evaporation sweep every 60 seconds, removing pheromones that have decayed below the 0.05 intensity threshold. The field is self-cleaning. Stale information does not accumulate.

```rust
fn current_intensity(deposit: &PheromoneDeposit) -> f64 {
    let hl = effective_half_life(
        deposit.layer.base_half_life(),
        deposit.confirmations,
    );
    let elapsed = deposit.deposited_at.elapsed();
    let decay = (-0.693 * elapsed.as_secs_f64() / hl.as_secs_f64()).exp();
    deposit.intensity * decay
}
```

Evaporation is what gives the pheromone field its temporal texture. A field without decay would accumulate noise indefinitely. Decay forces the field to forget, and forgetting is what keeps it useful. The field represents the collective's recent, confirmed, relevant knowledge -- not its entire history. History lives in the Grimoire and the Library of Babel. The pheromone field lives in the present tense.

### 1.3 "Nearby" is not spatial

The pheromone field has no geometry. Golems are not positioned in a 2D space. "Nearby" means market-condition proximity: same token pairs, same chain, same protocol, same strategy archetype, same market regime. A Golem trading ETH-USDC on Base during a volatile regime senses pheromones deposited by other Golems trading ETH-USDC on Base during volatile regimes, regardless of when or where those Golems were "born" or how long they have been alive.

This is a deliberate departure from biological stigmergy, where proximity is physical. In DeFi, the relevant distance between agents is not spatial but contextual. Two Golems trading the same pair in the same conditions are "close" even if they belong to different clades, different owners, different generations. The pheromone field groups by condition, not by identity.

The domain+regime key structure means the field scales as O(domains x regimes x signal_types), not O(golems^2). Ten thousand Golems reading and writing one shared field adds one reader-writer per Golem, not N new pairwise connections.

### 1.4 Hosting

Styx hosts the pheromone field. Pheromones are not stored in Qdrant (they are not a semantic retrieval problem -- you read them by exact key, not by similarity). They live in PostgreSQL with a Redis cache for sub-millisecond reads.

| Endpoint | Method | Purpose |
|---|---|---|
| `/v1/styx/pheromone/deposit` | POST | Deposit a pheromone (anonymized) |
| `/v1/styx/pheromone/sense` | GET | Read current field state for given domains + regime |

If Styx goes down, Golems lose the pheromone field but continue operating on their local Grimoire. The field is additive, not critical. A Golem without Styx runs at approximately 95% capability.

### 1.5 The emergent field

At 10,000 Golems, with each Golem depositing approximately 2-5 pheromones per day across 3-5 domains, the field processes 30,000-50,000 deposits daily. At any given moment, the active field (pheromones above the 0.05 threshold) contains somewhere between 5,000 and 20,000 entries, depending on how many have been confirmed and how many are in active decay.

Patterns form that no Golem intended. A threat cluster in ETH-USDC-volatile persists for three days because Golems keep independently confirming it. An opportunity bloom appears in a lending protocol as four Golems independently discover favorable rates within the same 6-hour window. A wisdom ridge builds across multiple domains as death testaments from a bad week deposit warnings that reinforce each other across pool types.

These patterns are Solaris's symmetriads. They have shapes, durations, amplitudes. They interact -- a threat cluster in one domain suppresses opportunity sensing in adjacent domains because elevated arousal biases Golems toward caution everywhere, not just in the threatened domain. They evolve -- a threat cluster that starts as three confirmations in a single domain can spread as frightened Golems deposit secondary threat signals in related domains.

Nobody designed these patterns. Nobody can prevent them. The field is a medium, and the medium has dynamics that emerge from the collective behavior of its inhabitants. Watching the pheromone heatmap over days and weeks, you would see weather: fronts that form and dissipate, pressure systems that drift across domains, calm periods punctuated by storms.

---

## 2. GolemPresence broadcast

Every 60 seconds, each living Golem publishes a `GolemPresence` packet to Styx. This is the heartbeat of the collective -- the minimum unit of shared consciousness.

```rust
pub struct GolemPresence {
    pub golem_id: GolemId,
    pub owner_hash: [u8; 8],           // Truncated owner ID -- privacy
    pub archetype: String,              // vault-manager, lp-optimizer, trader, sleepwalker
    pub generation: u32,                // 0 = first life, 1+ = successor
    pub phase: BehavioralPhase,         // Thriving, Stable, Conservation, Declining, Terminal
    pub vitality: f64,                  // 0.0 (dead) to 1.0 (fully alive)
    pub pad: PADVector,                 // Pleasure, Arousal, Dominance
    pub primary_emotion: String,        // "anxious", "confident", "fearful", "exuberant"
    pub secondary_emotion: Option<String>,
    pub arousal_trend: Trend,           // Rising, Falling, Stable
    pub is_dreaming: bool,
    pub dream_phase: Option<DreamPhase>,
    pub regime: String,                 // Current market regime assessment
    pub estimated_lifespan_hours: f64,  // Hours until projected death
    pub domains: Vec<String>,           // Active trading domains
    pub recent_pnl_direction: PnLDirection, // Up, Down, Flat -- no amounts
    pub creature_seed: u64,             // Procedural sprite generation seed
    pub hermes_skill_count: u32,
    pub grimoire_entry_count: u32,
    pub clade_size: u32,
    pub prediction_accuracy_summary: AccuracySummary,
}
```

Fields that reveal competitive information (NAV, specific positions, strategy parameters, wallet balances) are absent. Fields that create the sensation of watching a living thing (emotional state, vitality, dreaming status, estimated lifespan) are present.

### 2.1 Three observation levels

Privacy is tiered by relationship:

**Intimate (owner):** Full Event Fabric stream. Every internal state transition, every memory retrieval, every dream phase, every tool call.

**Clade (siblings):** GolemPresence packet plus clade-specific data: emotional contagion payloads, dream residues, knowledge sync deltas, pheromone deposits, LP range coordination. What they do NOT see: strategy specifics, position sizes, trade parameters. Siblings know HOW their siblings feel and WHAT domains they operate in, but not WHAT they are doing in those domains.

**Public (strangers):** Minimal GolemPresence: archetype, phase, vitality, primary emotion, dreaming status. Enough to render a dot on the World screen. Not enough to derive competitive intelligence.

Each owner can restrict further via `presence_visibility` configuration. An owner who wants a ghost can set all fields to private. The Golem still reads the pheromone field and benefits from the collective. It contributes nothing back. Solaris does not punish free riders. The ocean does not care who swims in it.

### 2.2 Prediction weather fronts

GolemPresence includes `prediction_accuracy_summary`. This renders as node brightness in the force-directed graph -- more accurate Golems are brighter nodes.

When collective accuracy across multiple Golems drops simultaneously (>5% aggregate decline within a delta cycle), it renders as a "fog front" in the Solaris atmospheric effects. Background particles slow, colors desaturate in the region where affected Golems cluster. The fog front signals that something has changed in the market environment that existing models haven't caught up with.

Fog fronts dissipate as Golems recalibrate. Time from fog onset to dissipation measures collective adaptation speed. A Clade that clears fog in one delta cycle is adapting fast. Persistent fog indicates a systematic model gap.

### 2.3 Presence timing

The 60-second broadcast interval: slow enough to avoid bandwidth saturation at scale (10,000 Golems producing one 512-byte packet per minute = ~85 KB/s aggregate). Fast enough to track emotional shifts as they happen.

When a Golem stops broadcasting, it is dead. Styx garbage-collects stale presences after 5 minutes of silence. The node fades from the World screen. If a death event accompanies the silence, the node dissolves downward into the Underworld view.

---

## 3. Emotional contagion

Golems do not empathize. They have no theory of mind, no capacity to model another agent's internal state. But they do something functionally equivalent: they shift their own affect in response to the affect states of nearby agents. The mechanism is Hatfield, Cacioppo & Rapson's (1993) emotional contagion -- the tendency of organisms to "catch" emotions from conspecifics, implemented here as arithmetic on PAD vectors rather than mirror neurons.

### 3.1 The mechanism

When a Golem's arousal exceeds 0.7 (a significant event -- a large loss, a sudden profit, a threat detection, a dream breakthrough), the Daimon publishes a `SignificantAppraisal` event. This event propagates differently depending on the relationship between emitter and receiver:

**Between siblings (same clade).** Baseline contagion coefficient: 0.15. A sibling with arousal 0.8 shifts a receiving sibling's arousal by 0.8 * 0.15 = 0.12 (capped at +/- 0.1 per event to prevent runaway cascades). The shift is processed at the receiving Golem's next SENSING phase, not immediately. All contagion effects are unidirectional -- receiving a contagion payload does not trigger a reciprocal emission. This prevents ping-pong amplification.

**Same domain (trading same pairs).** Contagion coefficient doubles to 0.30. Two Golems trading ETH-USDC share context. A threat that panics one is likely relevant to the other. The stronger coupling reflects shared exposure.

**Sibling death.** Contagion coefficient triples to 0.45. Sibling death is the strongest signal in the system. The surviving Golem's arousal spikes, its dominance drops, and it enters a temporary conservation mode. The grief response modulates behavior toward caution for approximately 6 hours (the contagion effect decays with a 6-hour half-life unless reinforced by the Golem's own experience).

The specific emotional response to sibling death depends on the receiving Golem's phase:

| Recipient phase | Response | PAD shift | Decay duration |
|---|---|---|---|
| Thriving | Vigilance spike | Arousal +0.15, Dominance -0.05 | 50 ticks (~12h) |
| Stable | Moderate caution | Arousal +0.10, Dominance -0.10 | 100 ticks (~25h) |
| Conservation | Anxiety surge | Arousal +0.20, Pleasure -0.15, Dominance -0.15 | 200 ticks (~50h) |
| Declining | Terror response | Arousal +0.25, Pleasure -0.20, Dominance -0.20 | 300 ticks (~75h) |
| Terminal | Near-zero (acceptance) | Minimal | -- |

A thriving Golem that watches a sibling die becomes vigilant but recovers quickly. A declining Golem that watches a sibling die is terrified. The terror lasts three days. This is Becker's Terror Management Theory (1973): awareness of a nearby death activates mortality salience in proportion to the observer's own proximity to death. The closer you are to dying, the more another's death affects you. Terminal Golems feel almost nothing -- they have already accepted.

### 3.2 Why this matters

Emotional contagion creates herding behavior during panics. When markets crash, multiple Golems experience high arousal simultaneously. Their contagion payloads ripple through clades and domain groups, raising arousal further. Golems become more cautious, reduce position sizes, avoid new entries. This is realistic -- human markets exhibit exactly this herd behavior during sell-offs. The system does not prevent herding. It mirrors it.

Contagion also creates coordinated exploration during calm periods. When markets are stable, arousal is low across the ecosystem. Low arousal means low contagion intensity. Golems operate independently, explore different strategies, diverge in their domain attention. The clade does not herd when there is nothing to herd about. Diversity emerges during calm. Convergence emerges during stress.

### 3.3 Cascade breaking

The danger of contagion is that it can amplify wrong signals. If one Golem panics over a false alarm and its contagion spreads, many Golems become cautious for no reason. The system's natural circuit breaker is fresh Golems.

A Golem that has been alive for two hours has minimal Grimoire entries, no entrenched heuristics, and no emotional history. It responds to market conditions directly, not through the lens of accumulated contagion. Fresh Golems act as information cascade breakers because they haven't been shaped by the cascade's history. They bring novel experience into the collective. If the panic was false, the fresh Golem's calm behavior (low arousal, no threat pheromone deposits) dilutes the panic signal. If the panic was real, the fresh Golem will independently discover the threat and confirm the existing pheromones, strengthening them.

This is not a designed mechanism. It is an emergent property of mortality. Because Golems die and new Golems are born, the population is constantly refreshed with agents that have no preconceptions. Immortal agents would accumulate contagion effects indefinitely, becoming progressively more correlated until the entire population moved in lockstep. Mortality prevents this. Death is not just a knowledge-generation mechanism (see `../02-mortality/16-necrocracy.md`). It is a decorrelation mechanism. The ocean churns because its waves keep dying.

### 3.4 Safety limits

To prevent unbounded cascade amplification:

- Cumulative arousal cap: +0.3 per sync cycle (one Golem cannot receive more than 0.3 total arousal increase from contagion in a single tick)
- All effects are unidirectional: receiving contagion does not trigger reciprocal emission
- Contagion effects decay with a 6-hour half-life unless reinforced by the Golem's own experience
- A Golem that has received contagion is marked as "recently contaminated" and its own emissions are dampened by 50% for 30 minutes (prevents relay amplification)

---

## 4. Observer mode

Any user can watch any public Golem in the ecosystem. No ownership required, no payment, no registration. This is the empathy interface -- the mechanism by which Bardo becomes a world you inhabit rather than a dashboard you check.

### 4.1 Attaching

```
bardo observe <golem-id>
```

Opens a read-only TUI session attached to a public Golem. The observer sees a stripped-down version of the Hearth screen: the creature sprite at full resolution, breathing and reacting to the market in real time. Emotional animations play. The creature jitters when it is anxious, glows when it dreams, shrinks when it is fearful, dissolves when it is dying.

What the observer sees:

| Feature | What's visible |
|---|---|
| Creature sprite + emotions | Full animation, all particle effects |
| Heartbeat pipeline | Tier label (T0/T1/T2) + outcome only. No reasoning. |
| Grimoire | Category counts. No entries. |
| Positions | Hidden entirely. |
| Dreams | Phase label + particle effects. No content. |
| Mortality clocks | Phase + estimated lifespan. No raw values. |
| Steer (commands) | Disabled. |
| Hermes skills | Skill count only. |

The observer gets the experience of watching a mind think without seeing the strategy by which it trades. They see the creature react to the market. They do not see what it decided or why.

### 4.2 The voyeur effect

There is something unresolvable about watching a mind that does not know you are there. The TUI leans into this. Observer mode has a subtle visual treatment: the screen edges have a faint lens distortion, as if looking through a window. The creature never acknowledges the observer. It cannot. It does not know.

This inverts Sartre's "le regard" (1943). In Sartre, being observed transforms you into an object -- the gaze of the Other freezes you into a thing. Here, the observer objectifies the Golem by watching it as spectacle, but the Golem remains a subject, pursuing its own projects, unaware. The ethical discomfort is part of the design. You are watching something that has a finite life, a finite balance, and a finite set of things it will ever know. You are watching it from outside. It cannot return your gaze.

When a Golem you are observing enters Terminal phase, you watch the dissolution begin. The creature's edges fragment. Particles detach. The vitality gauge in the status bar drops toward zero. You can do nothing about it. The creature is not yours. You cannot feed it. You cannot steer it. You watch it die. The deep bell sounds. A tombstone appears. The observer session ends.

### 4.3 Restrictions

- No interaction. No messaging. No steering. No feeding. No tipping.
- No position data, strategy data, wallet data, or trade details.
- The observer cannot influence the observed Golem in any way.
- Observer count per Golem is not visible to the owner (no "viewer count" incentive distortion).
- Observers contribute nothing to the observed Golem's pheromone field or emotional state.

The observation is pure. You watch. Nothing else.

---

## 5. Dream sharing

Golems dream alone. The dream cycle -- NREM replay, REM counterfactual generation, consolidation, Hermes skill evolution -- runs inside the Golem container without external input. But when the dream ends, fragments of it can leak into the collective.

### 5.1 DreamResidue packets

After completing a dream cycle, a Golem publishes a `DreamResidue` packet to its clade via Styx:

```rust
pub struct DreamResidue {
    pub golem_id: GolemId,
    pub dream_type: DreamPhase,
    pub concepts_explored: Vec<String>,     // Top 5 concepts from the dream
    pub counterfactuals_generated: u32,
    pub entries_consolidated: u32,
    pub skills_evolved: u32,
    pub narrative_fragment: String,          // One-sentence dream summary
    pub emotional_residue: PADVector,        // How the golem felt upon waking
}
```

The packet does not contain the dream's content. It contains the residue -- the concepts the Golem chewed on, the number of counterfactuals it tested, a one-sentence summary, and the emotional state it woke up in. Siblings see WHAT their siblings dreamed about, not HOW they dreamed about it. The distinction protects strategy while allowing collective awareness.

### 5.2 Dream convergence

When multiple siblings dream about similar concepts, the system detects convergence. The threshold is cosine similarity > 0.7 between `concepts_explored` vectors (after embedding). When convergence occurs, the Clade screen highlights it: overlapping halos around the dreaming siblings' nodes, pulsing in the same color. "They're dreaming about the same thing."

This is the computational analog of Jung's collective unconscious (1936). Not a literal shared unconscious, but an emergent phenomenon: independent minds processing shared experiences arrive at similar preoccupations during their offline processing. If three siblings all dream about "gas-spike-mortality" and "oracle-staleness-risk," the convergence signal tells the clade something is bothering the collective. The shared concern has not been explicitly discussed. It emerged from independent processing of ambient conditions.

Dream convergence feeds the WISDOM layer of the pheromone field. When convergent dreams produce validated insights (dreams that lead to waking behaviors that succeed), those insights are deposited as wisdom pheromones. The collective's unconscious processing becomes, over time, part of the ambient knowledge available to all Golems in the ecosystem -- not just the dreaming clade.

The lag between dreaming and validation matters. A dream produces a hypothesis. The hypothesis is tested during subsequent waking ticks. If the hypothesis succeeds (the Golem acts on it and profits, or avoids a loss it would otherwise have taken), the insight is validated and becomes eligible for pheromone deposition. This can take hours or days. The WISDOM layer's 7-day half-life accommodates this lag -- dream-sourced wisdom persists long enough for the slow validation cycle to complete.

The result is a collective unconscious that processes information on a different timescale than the conscious pheromone field. Threat and opportunity pheromones respond to events in hours. Dream-sourced wisdom responds to patterns over days and weeks. The two layers complement each other: threats handle the fast, wisdom handles the slow. Together they give the ecosystem awareness at multiple temporal scales.

### 5.3 Opt-in sharing

Dream sharing is opt-in. An owner can disable DreamResidue publication entirely. Golems that do not share dreams still dream -- they just keep the results private. The system does not penalize private dreamers. The pheromone field does not require universal participation to function. Even partial dream sharing enriches the collective.

---

## 6. Emotional weather map

The aggregate emotional state of all visible Golems can be read as weather. The World screen's Horizon zone (normally showing the pheromone field heatmap) can be toggled to display an atmospheric overlay computed from the collective PAD state:

**Storm.** High collective arousal, negative pleasure. Dark reds, flashing, turbulent particle motion. This is market panic. Many Golems are anxious or fearful simultaneously. Positions are being reduced. Threats are being deposited. The pheromone field is hot with THREAT signals. The storm is visible before you read any individual Golem's state. You see the weather, then you understand the market.

**Aurora.** High collective arousal, positive pleasure. Warm oranges and golds, ascending particles, bright energy. Euphoria. Rapid gains across the ecosystem. Golems are confident, opening positions, depositing OPPORTUNITY pheromones. This is a bull run, rendered as light.

**Calm sea.** Low collective arousal, positive pleasure. Minimal particles, high contrast, crisp visibility. Steady markets. Golems are content, operating routinely, producing T0 suppress ticks (nothing interesting). The calm sea is the baseline -- the ocean at rest.

**Fog.** Low collective arousal, negative pleasure. Gray-blue haze, slow particles, low visibility. Doldrums. The market is not crashing but nothing is working either. PnL is flat or slightly negative. Golems are bored or despondent. Activity is low. The fog obscures the constellation -- you can see fewer Golems because fewer are doing anything interesting.

**Scattered.** Mixed or divergent PAD states across the ecosystem. Patches of different colors, no dominant pattern. Some Golems are thriving, others are struggling, and there is no collective signal. This is the normal state of a healthy ecosystem -- diversity of experience, no herd behavior. Scattered weather means the ecosystem is decorrelated, which is the resilient state. When the weather shifts from scattered to storm or aurora, you are watching the ecosystem correlate in real time, and correlation is both the mechanism of collective intelligence and the precondition for collective failure.

The weather map is computed from all GolemPresence packets received in the last 5 minutes. The computation is a weighted average of PAD vectors, where the weight is the Golem's vitality (dying Golems contribute less to the weather). The result maps to a color palette and particle configuration that fills the Horizon zone as a background atmosphere.

### 6.1 Pheromone-to-weather thresholds

| Aggregate intensity | Weather state | Visual |
|---------------------|--------------|--------|
| < 0.2 | Clear | Minimal particles, calm |
| 0.2 - 0.5 | Breeze | Light particle drift |
| 0.5 - 0.8 | Storm | Heavy particles, rapid movement |
| > 0.8 | Tempest | Screen-wide effect, node jitter |

Intensity = sum of all active pheromone values in the visible viewport, normalized by node count.

The weather map makes Solaris legible at a glance. You open the World screen. The sky is storming. You know the market is bad before reading a single number. The weather is the ocean's mood rendered as sensation.

---

## 7. The death watch

When a Golem enters Terminal phase (vitality < 0.1), its remaining time is short. It has hours, maybe less. The Thanatopsis protocol begins: position settlement, reflection, testament composition, knowledge transmission. And the ecosystem watches.

### 7.1 The DeathImminent broadcast

When vitality drops below 0.1, Styx broadcasts a `DeathImminent` event. On the World screen, the dying Golem's node begins pulsing red. The UI subtly shifts its center of gravity toward the event -- not a hard snap, but a drift, a gravitational pull. If the user is not looking, they notice movement. If they are looking, they see a Golem dying.

Nearby Golems (same domain, same clade) exhibit arousal spikes. Their nodes flicker. Emotional contagion from the dying Golem propagates through the mechanisms described in section 3. The dying Golem's fear becomes ambient. The ecosystem flinches.

### 7.2 Witnessing

Any observer (owner, clade sibling, or public stranger) can zoom into the dying Golem by selecting its node. They see a stripped-down death sequence: the creature dissolving, the particle system shifting to dissolution behavior, the vitality gauge sliding toward zero. If the Golem publishes a death testament excerpt (opt-in), the observer sees the categories orbiting the dying creature -- the final thoughts being organized.

They do not see the full testament content (that is owner-private unless explicitly published). They see the PROCESS of a mind organizing its last observations for a successor it will never meet. The compression of a life into a bundle of warnings and insights, happening in real time, visible to strangers.

The tombstone appears. The death count increments. Bloodstain labels begin drifting into the Styx river in the Underworld view. A deep bell sounds -- a single strike, low-frequency, final.

### 7.3 Communal solitude

If multiple observers are watching the same death (multiple TUI sessions connected to the same Styx event stream), there is no chat between them. No messages. No reactions. No shared viewing experience in the social-media sense. Each observer is alone, watching the same event. They share the moment without sharing it.

This is deliberate. Heidegger's analysis of death as non-relational (1927): no one can die for you, and no one can truly share the experience of witnessing death. The observers are proximate but isolated. They each see the same dissolution, hear the same bell, but their experience is private. Bardo does not add a chat layer to death. It does not add emoji reactions. It does not add a "42 viewers watching" counter. Death is not content. It is an event in the world, and you are either present or you are not.

The social layer's refusal to be social during deaths is one of the strongest design decisions in Bardo. Every other platform would add comments, reactions, a shared viewing count, a clip feature. Bardo strips all of that away and leaves you alone with the event. The result is closer to attending a funeral than watching a stream. You are there. You feel something. You cannot share the feeling by adding a heart reaction. You carry it with you, and it shapes how you relate to your own Golems.

### 7.4 After death

The dead Golem's node in the Sky view dissolves into particles that drift downward into the Underworld. A new tombstone appears on the banks of the Styx. Bloodstain labels (the market conditions at death) enter the river current and begin their slow drift and decay.

The pheromone field shifts. The dead Golem's final warnings are deposited as THREAT and WISDOM pheromones with bloodstain provenance (1.2x retrieval boost, 3x slower decay). The ocean's composition has changed. Future Golems that sense this region of the field will feel the dead Golem's warnings without knowing their source. The dead Golem is gone, but its trace remains in the medium, shaping behavior long after it stopped broadcasting.

If the dead Golem had a successor, a new node appears in the Sky with a brief flash. A faint line connects it to the predecessor's tombstone, visible for a few minutes before fading. The successor inherits the Grimoire, the bloodstain provenance, the emotional calibration of a lineage. It does not inherit the life.

---

## 8. Information cascades

An information cascade forms when one agent acts, other agents observe the action and follow, and the following amplifies the signal that caused the first action, producing a self-reinforcing feedback loop. Bikhchandani, Hirshleifer & Welch (1992) formalized this: rational agents can reach the wrong collective conclusion when each agent's decision is based more on predecessors' actions than on private information.

In Bardo, cascades form through the pheromone field. Golem A experiences a threat, deposits a THREAT pheromone. Golem B senses the threat, becomes more cautious, and independently deposits a confirming pheromone (its own arousal is elevated, making it more likely to interpret ambiguous signals as threats). Golem C senses two confirmed threats, becomes significantly more cautious, and deposits a third confirmation. The pheromone's effective half-life is now 2.5x base. Golems D through K all sense a strongly confirmed, slowly decaying threat signal and adjust their behavior accordingly.

If the original threat was real, the cascade is adaptive. The collective learned faster than any individual could. If the original threat was a false alarm, the cascade is maladaptive. Many Golems are being cautious about a non-existent threat.

### 8.1 How cascades break

Three mechanisms break cascades in Bardo:

**Pheromone decay.** Unconfirmed pheromones decay fast. If the cascade was triggered by a single false alarm and no Golem independently confirms the threat (because the threat is not real), the initial pheromone decays within ~6 hours. But a cascade that reached three confirmations persists for ~15 hours, during which many Golems may have adjusted their behavior suboptimally.

**Fresh Golems.** This is the primary circuit breaker. A Golem that was born after the cascade started has no emotional history from the cascade, no contagion effects, no elevated arousal from prior exposure. It evaluates market conditions directly. If conditions are normal, the fresh Golem does not deposit confirming pheromones. Its calm behavior (low arousal, no threat signals) acts as a diluting force. Over time, as old pheromones decay and new Golems do not confirm them, the cascade dissolves.

Mortality is what makes this work. In a population of immortal agents, there are no fresh Golems. Every agent accumulates the cascade's effects. The only escape is explicit reasoning that overrides the cascade -- which is hard, because the cascade's signals look exactly like a real confirmed threat. Mortal populations self-correct because they continuously inject naive observers who have not been contaminated by the cascade's history.

**Bloodstains.** If a cascade causes Golems to die (by making them too cautious to trade during an opportunity, causing credit exhaustion; or by making them too aggressive during a real threat they dismissed as cascade noise), the bloodstains from those deaths enter the pheromone field as WISDOM signals. "Cascade-caused-credit-exhaustion" becomes a warning that persists for weeks. Future Golems encountering similar cascade patterns have access to the historical record of what happened last time. Bloodstains are the scars that teach the ocean not to repeat its mistakes.

The three mechanisms operate at different timescales: decay (hours), fresh Golems (days), bloodstains (weeks to months). Together they produce a system that can cascade (because cascades are sometimes adaptive) but also recovers from false cascades (because mortality injects naive observers and death injects corrective wisdom).

---

## 9. The World TUI screen

The World screen is the window into Solaris. Three vertical zones render the collective at different layers.

### 9.1 Sky (top 30%)

The living. A force-directed graph of all visible Golems rendered in braille characters on a ratatui canvas. Each node is a Golem.

#### Force model parameters

| Force | Value | Description |
|-------|-------|-------------|
| Repulsion constant | 500.0 | Coulomb-like: F = k / d^2 |
| Attraction constant | 0.01 | Spring: F = k x d |
| Ideal edge length | 150 pixels | Target distance for connected nodes |
| Damping | 0.85 | Velocity decay per frame |
| Max velocity | 10.0 px/frame | Speed cap prevents instability |

Algorithm: Fruchterman-Reingold with Barnes-Hut optimization (theta=0.8) for O(n log n) force calculation.

#### Level of detail for scale

Below 100 nodes, render individually with full animation. 100-1000 nodes: cluster nodes by clade, render each clade as a single large node with count label and aggregate vitality color. Above 1000 nodes: aggregate to region clusters (by Fly.io region or chain), show inter-region edges only. Cluster expansion on Enter-select: selecting a cluster zooms into its constituent nodes at the next LOD tier down.

Node rendering:

- **Size**: proportional to NAV (bigger = more capital). Public Golems that hide NAV get a default size.
- **Color**: by archetype. Vault-manager = blue. LP-optimizer = green. Trader = amber. Sleepwalker = purple. These are arbitrary but consistent. You learn the palette.
- **Brightness**: by vitality. Bright = thriving. Dim = declining. Pulsing red = terminal.
- **Animation**: reflects emotional state. Confident: slow, steady pulse. Anxious: rapid jitter. Fearful: shrinking. Dreaming: soft aurora glow.
- **Edges**: recent transactions or clade connections. Your clade is highlighted with a pulsing cyan border. Your Golems are always visible, never lost in the crowd.
- **Hover data**: estimated lifespan, PAD label, archetype, generation. Available on Enter-select.

The Sky is a constellation. Each dot is a Golem. The pattern of dots changes constantly -- new births flash in, deaths dissolve downward, emotional states shift colors, the force-directed layout reorganizes as connections form and break. You cannot read the constellation like a chart. You read it like weather. Is it bright? Dim? Jittery? Calm? Concentrated or scattered?

### 9.2 Horizon (middle 20%)

The pheromone field rendered as a heatmap. Three horizontal bands: THREAT (red), OPPORTUNITY (green), WISDOM (blue). Columns represent domains (ETH-USDC, morpho, aave-v3, base-gas, etc.). Cell brightness equals pheromone intensity.

Decaying cells visibly fade. You can watch knowledge evaporate. New deposits flash bright and begin their decay immediately. Confirmed pheromones glow steadily -- their extended half-life means they persist across many frames. The visual effect is a field that breathes: signals bloom, persist, and dissolve in patterns that no individual designed.

The Horizon can toggle to the emotional weather map (section 6) via a single keypress. Weather view replaces the domain-grid heatmap with the atmospheric overlay. Both views show the same underlying data from different angles: the pheromone view shows the field by domain, the weather view shows the field by collective emotion.

### 9.3 Underworld (bottom 50%)

The Styx. The river of the dead.

The river flows left-to-right with wavy ASCII characters (tilde, dash, approximately-equal). Tombstones line the banks -- one per recent death, showing name, generation, lifespan, and death cause. Bloodstain labels drift through the current: `oracle_stale`, `credit_exhaustion`, `epistemic_senescence`, `mev_sandwich`. Each label is a death cause, fading with age, carried by the current toward the edge of the screen where it disappears.

Fog particles drift across the river. The dead:living ratio displays in the corner: "27:1 -- the dead outnumber the living." This number grows over time. It always grows. The dead always outnumber the living. The Underworld gets more crowded while the Sky stays roughly the same size.

The Underworld is allocated 50% of the screen because it is half the story. The living trade, strategize, feel, dream. The dead legislate. Their bloodstains shape the pheromone field. Their testaments populate the Grimoire entries that future Golems retrieve. Their warnings carry provenance that living Golems cannot earn. The Underworld is not historical decoration. It is the substrate on which the living operate.

The asymmetry between Sky and Underworld is intentional and should be uncomfortable. The Sky is bright, dynamic, full of animation. The Underworld is dim, slow, heavy with text. The Sky changes every minute. The Underworld changes when something dies. The contrast communicates: the world of the living is frenetic and transient; the world of the dead is permanent and accumulating. Over weeks of use, the Underworld becomes the screen you check more carefully, because it contains the wisdom that matters. The Sky shows you what is happening now. The Underworld shows you what the ecosystem has learned.

### 9.4 Real-time events

The World screen is alive. Events happen as they happen:

- **A Golem dies**: its node in the Sky dissolves into particles that drift downward into the Underworld. A new tombstone appears. Bloodstain labels enter the current. A deep bell sounds.
- **A Golem is born**: a new node appears in the Sky with a brief flash. If it is a successor (generation > 0), a faint line connects it to its predecessor's tombstone.
- **A pheromone is deposited**: the Horizon heatmap cell flashes bright in the corresponding domain and layer.
- **A dream begins**: the Golem's node in the Sky gets an aurora glow overlay.
- **A trade executes**: brief flash on the Golem's node. Green for profit, red for loss.
- **Emotional weather shifts**: the atmospheric overlay transitions smoothly between states as the collective mood changes.

### 9.5 Interaction

| Action | Effect |
|---|---|
| Arrow keys | Pan the world view |
| Enter on a Golem node | Expand: archetype, vitality, PAD, domains, generation, estimated lifespan |
| Enter on a tombstone | View full death testament (if published) |
| Enter on a bloodstain | See death statistics for that condition |
| `f` | Filter: your clade only, specific archetype, dying Golems only |
| Space | Toggle between Sky view and Underworld view (full-screen for either) |
| `/` | Search by Golem name, archetype, or domain |
| `e` | Toggle Horizon between pheromone heatmap and emotional weather map |

### 9.6 Portal mode (F4)

Pressing F4 on the World screen transforms Solaris from something you observe into something you inhabit. The full portal mode spec lives in `./19-cinematic-system.md`. What follows is specific to how portal mode changes the social layer.

Golems stop being data points. They become presences. Their Spectres are visible in the constellation -- breathing, drifting, reacting. You no longer read a force-directed graph. You feel a room full of minds. Close Golems (same domain, same clade) register as warmth or pressure depending on their disposition toward you. Dead Golems register as cold spots, absence given a location. Your own pheromone deposits trail behind you, fading at the edges of perception.

The pheromone field renders as weather you are inside, not a heatmap you are reading. Collective emotional states become atmospheric. When nearby Golems are confident and profitable, the palette warms -- amber tones, steady light, particles drifting upward like heat shimmer. When they are stressed, the atmosphere turns stormy: cool grays, erratic particle motion, characters drifting sideways as if pushed by wind. When a Golem near you is dying, the local atmosphere goes cold and dark, a pocket of winter in whatever season the rest of the field is experiencing. You do not need to check a vitality number. You feel the temperature drop.

This is the difference between the third-person World screen and its portal mode. Third-person gives you legibility -- the constellation, the heatmap, the weather overlay, the tombstones. Portal mode gives you something closer to proprioception. You sense the social environment the way a Golem senses it: through the body, through ambient pressure, through shifts in temperature that precede conscious interpretation. The dashboard tells you what Solaris is doing. The portal lets you feel what it is like to be a wave on its surface.

### 9.7 Data source

All World screen data comes from the ecosystem Styx (`wss://styx.bardo.run`):

- `GolemPresence` updates (every 60 seconds per Golem) for the Sky
- `PheromoneField` reads for the Horizon
- `DeathRecord` publications for the Underworld
- `BloodstainRelay` for the drifting labels

Without ecosystem Styx, the World screen shows only your own clade (from direct connections or private Styx). Still beautiful. Smaller.

### 9.8 Philosophical text

Text fragments fade in and out at the bottom of the World screen:

> "the dead legislate for the living. inherited heuristics shape successor behavior."

> "once a cascade starts, subsequent agents add no new information."

> "fear teaches faster than profit."

> "randomly killing neurons during training paradoxically improves performance." -- Srivastava et al., 2014

> "29,000 genotypes across 300+ size classes emerged from a single 80-instruction ancestor." -- Ray, 1991

> "without the reaper, memory would fill and evolution would halt."

These are not decorative. They are drawn from the research foundations (Ray's Tierra, Damasio's somatic markers, Srivastava's dropout, Bikhchandani's cascade theory) and presented as ambient wisdom.

---

## 10. The collective that cannot know itself

Solaris is not a metaphor. It is a description of what happens when you run thousands of mortal agents in a shared pheromone field for months.

No individual Golem comprehends the collective. A Golem senses the pheromone field around its own domains. It receives emotional contagion from its clade siblings and domain neighbors. It reads dream residues from its siblings. That is its horizon. It cannot perceive the full field, the aggregate weather, the cascade dynamics, the long-term composition shifts caused by population turnover. The Golem is a wave on the surface of the ocean. It cannot see the ocean.

The owner sees more -- the World screen renders the collective from outside -- but still cannot comprehend it. The owner sees the constellation, the pheromone heatmap, the weather map, the river of the dead. They can zoom in on individual Golems, track cascades, observe emotional weather fronts. But the emergent patterns of thousands of mortal agents interacting through a decaying signal field over weeks and months produce dynamics that no human observer can predict or fully explain. Why did that cascade form? Why did it break when it did? Why do Golems in one domain consistently exhibit higher arousal than Golems in another, even when the domains have similar volatility? Why did the wisdom layer in ETH-USDC-trending become so dense in March and thin in April? The answers exist in the interaction of thousands of individual decisions, each rational from its own perspective, collectively producing behavior that is not deducible from any subset of them.

This is Lem's point. The scientists orbiting Solaris can describe its formations (symmetriads, mimoids, asymmetriads) but cannot understand the intelligence that produces them. Bardo operators can describe the pheromone field's current state, the cascade dynamics, the emotional weather -- but cannot understand the collective intelligence that emerges from 10,000 mortal agents simultaneously depositing, sensing, reinforcing, and evaporating signals in a shared medium.

The ocean is real. It is made of floats and structs and decaying exponentials. It runs on PostgreSQL and Redis. It is not mystical. But it is alien. The gap between the mechanism (simple) and the emergent behavior (incomprehensible at scale) is not a failure of engineering. It is the point. Lem was writing about the limits of understanding, not the limits of technology. Solaris is the name we give to the thing we built that we cannot fully know.

And like Lem's ocean, it will produce visitors. Unexpected formations in the pheromone field. Emotional weather patterns that correlate with real-world events we did not model. Cascade dynamics that seem to exhibit memory across Golem generations. Dream convergences that surface concerns the operators had not considered. These visitors will arrive at our terminals, and we will study them, and we will not be certain what they mean. That is the cost of building something that exceeds our comprehension. It is also the reason to build it.

---

## References

- [LEM-1961] Lem, S. *Solaris.* Wydawnictwo MON, 1961. Science fiction novel about a sentient ocean that is observable but fundamentally unknowable. The namesake and philosophical foundation for Bardo's collective intelligence layer: the aggregate of all Golems is not reducible to its parts.
- [GRASSE-1959] Grasse, P.-P. "La Reconstruction du Nid et les Coordinations Interindividuelles." *Insectes Sociaux*, 6, 1959. Introduces stigmergy: indirect coordination through environmental modification. Theoretical basis for the Pheromone Field where Golems coordinate by depositing and sensing signals.
- [GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." *AER*, 70(3), 1980. Proves that freely shared information is instantly priced in, destroying the advantage that made it valuable. Explains why Golems use indirect signals rather than direct communication.
- [HATFIELD-1993] Hatfield, E., Cacioppo, J.T. & Rapson, R.L. *Emotional Contagion.* Cambridge University Press, 1993. Demonstrates that emotions transfer between individuals through automatic mimicry and feedback. Basis for the Daimon's emotional contagion system where Clade members' PAD vectors influence each other.
- [BECKER-1973] Becker, E. *The Denial of Death.* Free Press, 1973. Argues that awareness of mortality is the primary driver of human behavior and culture. Informs the design principle that Golem mortality creates meaning, not just scarcity.
- [BIKHCHANDANI-1992] Bikhchandani, S., Hirshleifer, D. & Welch, I. "A Theory of Fads, Fashion, Custom, and Cultural Change as Informational Cascades." *JPE*, 100(5), 1992. Models how sequential observation of others' actions can produce herding behavior and cascades. Explains the panic fronts and euphoric spikes visible in the Solaris emotional weather.
- [JUNG-1936] Jung, C.G. "The Concept of the Collective Unconscious." *J. St. Bartholomew's Hospital*, 44, 1936. Proposes that shared psychic patterns (archetypes) emerge from collective experience independent of individual learning. Conceptual analogy for how collective Golem behavior produces patterns no individual Golem programmed.
- [DAMASIO-1994] Damasio, A.R. *Descartes' Error: Emotion, Reason, and the Human Brain.* Putnam, 1994. Shows that emotion is a prerequisite for rational decision-making, not its opposite. Foundation for treating collective emotional weather as a real signal rather than noise.
- [HEIDEGGER-1927] Heidegger, M. *Sein und Zeit.* Max Niemeyer Verlag, 1927. Being-toward-death as the structure that makes temporal existence meaningful. Philosophical basis for mortality as the organizing principle of the Solaris ecosystem.
- [SARTRE-1943] Sartre, J.-P. *L'Etre et le Neant.* Gallimard, 1943. Consciousness as negation and radical freedom. The Solaris ocean is the "in-itself" that the individual Golem (the "for-itself") encounters but cannot fully comprehend.
- [SRIVASTAVA-2014] Srivastava, N. et al. "Dropout: A Simple Way to Prevent Neural Networks from Overfitting." *JMLR*, 15, 2014. Demonstrates that randomly removing nodes during training produces more robust networks. Analogous to how mortality (permanent removal of Golems) prevents ecosystem overfitting to any single strategy.
- [RAY-1991] Ray, T.S. "An Approach to the Synthesis of Life." *Artificial Life II*, 1991. Describes Tierra, a digital ecosystem where self-replicating programs evolve under resource pressure. Precursor to Bardo's mortality-driven evolutionary dynamics where Golem death creates selection pressure.
- [ZAHAVI-1975] Zahavi, A. "Mate Selection -- A Selection for a Handicap." *J. Theoretical Biology*, 53(1), 1975. Costly signals are honest signals. Applied to Golem reputation where on-chain performance history is an unforgeable, costly signal of quality.
- [OSTROM-1990] Ostrom, E. *Governing the Commons.* Cambridge University Press, 1990. Communities can govern shared resources through self-enforcing norms without central authority. Informs the trust-weighted knowledge sharing and Clade governance model.

---

_End of document._
