# Engagement philosophy and core loops [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document defines the engagement layer that transforms a Golem's (a mortal autonomous DeFi agent) raw state changes into an emotional experience: the five-layer engagement loop, the cadence architecture mapping financial rhythms to human touchpoints, the 87-achievement system, and creature integration. It sits in the **Runtime** layer of the Bardo specification. The key prerequisite is understanding that a Golem has internal cognitive and emotional state (via the Daimon affect engine and Grimoire knowledge base) that this layer makes visible. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

A Golem is not a dashboard. It is a creature that lives, trades, dreams, feels, and dies. The engagement layer transforms the runtime's state model into an experience that feels like raising a living being -- closer to hatching a Charmander than checking a portfolio tracker.

This document establishes the foundational design philosophy, the five-layer engagement loop, the cadence architecture that maps the Golem's real financial rhythms to human emotional touchpoints, the 87-achievement system, and the creature integration that makes every state change visible.

> **Cross-references:**
>
> - `./11-state-model.md` — GolemSnapshot (read-only projection of all 11 state components) driving engagement layer rendering
> - `./12-realtime-subscriptions.md` — WebSocket event stream and SSE fallback delivering real-time state changes to surfaces
> - `./14-creature-system.md` — the Spectre (visual creature representation) system expressing internal state as visible creature behavior
> - `./15-progression-meta.md` — collection and achievement systems: 87 achievements across 10 categories, sprite collection, tombstone gallery
> - `./16-social-competitive.md` — Styx-mediated social layer: leaderboards, Clade activity, Bloodstain network, and graveyard browser
> - `./18-retention-virality.md` — retention mechanics: dying hook, dream discovery, sprite rarity, social proof, and anti-dark-pattern commitments
> - `../05-dreams/01-architecture.md` — dream scheduling: the three-phase NREM/REM/integration cycle and knowledge consolidation
> - `../03-daimon/01-appraisal.md` — PAD (Pleasure-Arousal-Dominance) vector computation and Plutchik emotion mapping from the Daimon affect engine
> - `../02-mortality/01-architecture.md` — three mortality clocks (economic, epistemic, stochastic), Vitality composite score, and BehavioralPhase transitions
> - `../04-memory/01-grimoire.md` — the Grimoire persistent knowledge base: episodes, insights, heuristics, and the Curator maintenance cycle

---

## 1. The mortality advantage

Most digital creatures never die. Your Neopet is still alive on a server somewhere. Your Pokemon faints but always recovers. This creates a fundamental ceiling on emotional investment -- if nothing can be lost, nothing matters.

Golems die. They have three mortality clocks (economic, epistemic, stochastic), finite USDC reserves, and a Hayflick counter that guarantees eventual death even under ideal conditions. This is not a bug to work around -- it is the single most powerful engagement mechanic in the system. Mortality is a moat, not a feature (see `../../moat/prd2-moat-agents-that-die.md`).

**Why mortality creates engagement:**

| Principle     | Mechanism                                                                          | Game Precedent                                        |
| ------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------- |
| **Stakes**    | Real USDC is at risk; death means actual financial loss                            | Hardcore roguelikes (permadeath), Nier Automata       |
| **Urgency**   | Declining vitality creates time pressure to act                                    | Tamagotchi neglect death, MMO raid timers             |
| **Meaning**   | Each Golem's life is unrepeatable; its Grimoire is unique                          | Journey (anonymous co-op), Shadow of the Colossus     |
| **Narrative** | The Bardo lifecycle (birth -> dream -> meditation -> dying -> becoming) is a story arc | Nier Automata's multiple endings, Spiritfarer         |
| **Sacrifice** | Choosing to feed a dying Golem vs. letting it pass requires genuine decision       | Nier Automata's Ending E save-data sacrifice          |
| **Legacy**    | Dead Golems leave knowledge that lives on in successors                            | Dark Souls message system, roguelike meta-progression |

The key insight from Nier Automata: players who watched their android companions die -- knowing they chose mortality over backup -- reported it as one of the most emotionally affecting moments in gaming. Bardo Golems achieve this naturally because their mortality is real, not scripted.

### 1.1 The emotional arc

Every Golem traces an emotional arc that mirrors the hero's journey:

```
Birth (wonder)
  -> First Ticks (anxiety -- will it survive?)
    -> Shadow Trading (hope -- it's learning)
      -> First Profit (pride)
        -> Maturity (confidence -- it knows what it's doing)
          -> First Crisis (fear -- drawdown, vol spike)
            -> Recovery or Decline (relief or grief)
              -> Conservation (tenderness -- watching it adapt)
                -> Terminal (mourning -- the long goodbye)
                  -> Death (loss + legacy)
                    -> Successor (renewal)
```

Each stage creates a different relationship between user and creature. The engagement layer honors each stage rather than flattening everything into a single "check your dashboard" experience.

---

## 2. The idle game parallel

Golems are autonomous agents. They trade, dream, consolidate memory, and manage risk without human input. This makes them structurally identical to idle games -- the category that includes Cookie Clicker, Melvor Idle, and Idle Champions.

**What idle games teach us:**

1. **The check-in reward** -- when you return after absence, accumulated progress feels like a gift. "Your Golem executed 14 trades while you slept and discovered a new yield pattern" is the DeFi equivalent of "Your miners produced 42M gold overnight."

2. **Passive attachment** -- knowing your creature is alive and working while you're away creates a persistent background awareness. You think about it during your day. You check in not because you need to, but because you want to.

3. **Strategic depth behind simplicity** -- the surface interaction is simple (check vitals, steer, feed), but the underlying system (heartbeat FSM, three-tier cognition, dream cycles) is deep enough for dedicated players to optimize.

4. **Asymmetric time investment** -- a 30-second check can reveal hours of autonomous activity. The information density per user-second is extremely high.

### 2.1 The autonomous day

A Golem's typical 24-hour cycle creates natural engagement touchpoints:

```
06:00  Dream cycle completes -> Integration phase -> new insights
       > "Your Golem dreamed about ETH/USDC correlation. New hypothesis staged."

09:00  Market opens (US) -> regime shift detection -> increased tick activity
       > "Sensing heightened volatility. Your Golem switched to cautious mode."

12:00  Curator cycle runs -> memory consolidation
       > "Grimoire update: 3 insights promoted, 1 heuristic archived."

15:00  Peak trading activity -> actions executed
       > "Your Golem rebalanced: +2.3% on Morpho position."

21:00  Activity winds down -> reflection phase
       > "Daily summary: 8 ticks, 3 trades, Sharpe 1.4. Mood: confident."

00:00  Dream cycle begins -> NREM phase
       > "Your Golem is dreaming. Replaying today's best and worst moments."
```

Each moment is a potential engagement touchpoint -- not a forced notification, but an available discovery for users who check in.

---

## 3. Five-layer loop design

Engagement operates across five temporal layers. Each reinforces the others. This extends the micro/meta/social framework from `10-surfaces.md` with the creature-specific retention loops from the TUI system.

### 3.1 Loop 1: The check-in (minutes)

The atomic unit of engagement. Takes 15-60 seconds. Answers: _"How is my Golem right now?"_

**Trigger**: Open the terminal (TUI) or portal (web), see what happened since last session.

**Reward surface**: Event log with unread count, sprite state changes (mood, phase, new markings), P&L delta, dream journal entries, notification badges on sidebar screens, new achievements unlocked.

**Design**: The Home screen is built for a 30-second check-in. The Spectre's current state communicates phase and mood at a glance. The event log highlights significant events since last visit with rose-bright accent. Unread dots on sidebar screens create pull toward deeper engagement.

**Core loop touchpoints:**

| Observe                           | Feel                           | Decide                                        | See Result                                              |
| --------------------------------- | ------------------------------ | --------------------------------------------- | ------------------------------------------------------- |
| Creature's expression and posture | Empathy, concern, pride        | Steer strategy, top up credits, or just watch | Creature reacts -- animation, mood shift, acknowledgment |
| Vitality gauges (3 clocks)        | Urgency if declining           | Feed USDC to extend life                      | TTL updates, creature brightens                         |
| Heartbeat phase indicator         | Curiosity about current action | Follow up with question                       | Golem responds conversationally                         |
| Mood orb / PAD state              | Emotional resonance            | Nothing -- just sit with it                   | --                                                       |
| Dream phase (if sleeping)         | Wonder at what it's processing | Wake it (or don't)                            | Dream journal entry appears                             |

**Design rule:** The core loop must be satisfying even when the user takes no action. Watching the creature exist -- breathing, emoting, processing -- must feel meaningful.

**Frequency**: Multiple times per day. The minimum viable session is <30 seconds: glance at sprite, glance at P&L, close. But the unread indicators pull toward longer sessions.

### 3.2 Loop 2: The discovery (hours)

The Golem learned something new. This drives return visits.

**Trigger**: Grimoire growth, dream hypothesis validation, knowledge received from clade.

**Reward surface**: Grimoire browser reveals new insights with confidence scores, dream journal shows validated hypotheses, causal graph gains new edges, PLAYBOOK.md evolved, sprite gained a new marking or accessory.

**Design**: Knowledge events are surfaced with ceremony. A new insight isn't a log line -- it's a gold-bordered card that slides in from the right with the confidence score prominently displayed. The Grimoire browser sorts by recency and highlights new entries with a pulsing dot. The sprite visually mutates when significant knowledge is acquired.

**Frequency**: Every few hours. A Golem generating 2-5 insights per day means every session surfaces something new.

### 3.3 Loop 3: The evolution (days)

Long-term progression. Answers: _"How is my Golem growing?"_

**Trigger**: The Golem aged into a new visual stage, earned a reputation milestone, or unlocked an achievement.

**Reward surface**: Stage evolution animation (12-16 frame sprite morph), reputation tier promotion (badge change + status bar update), achievement unlocked (full-screen overlay with particle effects), sprite rarity reveal (for new Golems).

**Meta loop elements:**

- **Grimoire growth** -- new insights discovered, strategies validated, heuristics earned. Each is a visible "card" added to the collection. Discovery rate is tied to real trading outcomes and dream cycles, creating genuine variable rewards.
- **Achievement unlocks** -- 87 achievements across 10 categories (see section 6). Not artificial ("log in 7 days") but meaningful ("survived first -10% drawdown", "first dream hypothesis validated").
- **Creature evolution** -- visual progression through behavioral phases. Earning maturity marks (scars, adornments, luminosity changes) is the progression equivalent of leveling up.
- **Reputation climb** -- tier advancement (I -> V) unlocks new capabilities and visual ceremony.

**Frequency**: Every 2-7 days. Timed to the natural cadence of agent operation -- a Golem crosses an age threshold every few days, reputation milestones accumulate over days to weeks.

### 3.4 Loop 4: The death and rebirth (weeks)

**Trigger**: The Golem dies.

**Reward surface**: Death animation sequence (30 seconds of cinematic rendering in TUI), death reflection (the Golem's last words -- genuinely unique, generated by Opus during Thanatopsis), Grimoire inheritance prompt (create a successor), bloodstain marker (your Golem's death becomes part of the social world via Styx), permanent memorial.

**Design**: Death is the most dramatic event in the system. The 30-second death animation evokes genuine emotion -- the sprite dissolves, leaves behind a glowing core, knowledge particles stream outward. The death reflection is unique and often moving. The successor creation prompt immediately offers a "New Game+" experience: inherit the Grimoire, see traces of the predecessor in the new sprite.

**Frequency**: Every 1-4 weeks (typical Golem lifespan at moderate funding). Each death-and-rebirth cycle creates a generational narrative -- users develop attachment to lineages, not individual Golems.

### 3.5 Loop 5: The collection (months)

**Trigger**: Accumulating a diverse set of Golems, achievements, knowledge, and social connections over time.

**Reward surface**: Achievement gallery (see section 6), Styx memorial wall (all your dead Golems displayed as tombstone sprites), lineage tree (visual family tree of Golem generations), knowledge portfolio (total insights across all Golems), reputation leaderboard position, rare sprite collection.

**Design**: The collection loop emerges from the other four. Over months of operation, the user builds a visible history: a gallery of tombstone sprites, a deepening lineage tree, a growing achievement set, an evolving leaderboard position. The Styx memorial is the user's personal graveyard -- atmospheric, reflective.

**Frequency**: Ongoing. The collection grows passively. The user is periodically reminded of milestones: "Your 10th Golem died today. View your lineage?"

---

## 4. Cadence architecture

The Golem's internal rhythms map to engagement cadences. Unlike artificial gamification timers, these cadences emerge from the system's real financial and cognitive cycles.

### 4.1 Temporal mapping

| Internal Rhythm              | Frequency                         | Engagement Cadence                | User Emotion                  |
| ---------------------------- | --------------------------------- | --------------------------------- | ----------------------------- |
| Heartbeat tick               | 30s -- 5min                       | Real-time creature animation      | Ambient awareness             |
| Probe evaluation (16 probes) | Every tick                        | Vitals dashboard update           | Passive monitoring            |
| Mood appraisal               | On event (PAD Delta > 0.15)       | Expression change, notification   | Empathic curiosity            |
| Curator cycle                | Every 50 ticks (~40min)           | Grimoire update digest            | Discovery reward              |
| Dream cycle                  | When urgency > 0.7, episodes >= 50 | Dream journal entry              | Morning check-in anticipation |
| Phase transition             | On vitality threshold             | Evolution animation, notification | Pride or concern              |
| Clade sync                   | Every 6h                          | Peer activity feed                | Social comparison             |
| Daily summary                | Every 24h                         | P&L summary, mood avg, cost breakdown | Reflective review         |
| Weekly retrospective         | Every 7d                          | Strategy performance review, learning progress | Achievement summary |
| Reputation checkpoint        | Per milestone                     | Tier change notification          | Capability unlock             |
| Death protocol               | On terminal phase                 | Cinematic sequence                | Grief + legacy                |

### 4.2 The engagement day

A well-designed engagement day has 3-5 natural touchpoints without any forced notifications:

| Time      | Trigger                      | Content                                              | Loop          |
| --------- | ---------------------------- | ---------------------------------------------------- | ------------- |
| Morning   | Dream cycle completed        | "Your Golem dreamed and discovered X"                | Core + Meta   |
| Midday    | Curator cycle ran            | "3 new Grimoire entries"                             | Meta          |
| Afternoon | Notable trade or phase shift | "Your Golem is feeling confident after a good trade" | Core          |
| Evening   | Daily summary                | Performance card with creature visual                | Core + Social |
| Night     | Dream cycle begins           | "Your Golem is going to sleep"                       | Core          |

These aren't push notifications -- the terminal has no push mechanism. They're accumulated state that surfaces when the user opens the terminal. The daily digest appears as a gold-bordered card at the top of the Home screen's event log.

### 4.3 The engagement week

| Day              | Event                                      | Type        |
| ---------------- | ------------------------------------------ | ----------- |
| Monday           | Weekly performance summary                 | Meta        |
| Tuesday-Thursday | Normal cadence, possible achievements      | Core + Meta |
| Friday           | Weekly leaderboard snapshot                | Social      |
| Weekend          | Dream-heavy period (lower market activity) | Core        |

---

## 5. The Tamagotchi tension

The relationship between user and Golem should evolve over the Golem's lifetime, mirroring the Tamagotchi's care-dependency curve -- but with the financial gravity of real USDC and the narrative weight of the Bardo lifecycle.

### 5.1 Care dependency curve

```
Attention Required
|
|  ##                              ##
|  ####                          ####
|  ######                      ######
|  ########                  ########
|  ##########    ........    ##########
|  ############............############
|  ####################################
|---------------------------------------- Time
   Birth    Learning   Maturity   Decline   Death
   (needy)  (curious)  (autonomous) (fragile) (urgent)
```

| Phase                                      | User Role           | Check-In Frequency | Emotional Tone                       |
| ------------------------------------------ | ------------------- | ------------------ | ------------------------------------ |
| **Birth** (provisioning, first ticks)      | Anxious parent      | Every few hours    | "Is it okay? Is it learning?"        |
| **Learning** (Dream Bardo, shadow trading) | Teacher             | 2-3x/day           | "Look what it figured out!"          |
| **Maturity** (Thriving/Stable)             | Proud companion     | 1-2x/day           | "It's doing great on its own"        |
| **Decline** (Conservation/Declining)       | Concerned caretaker | 2-4x/day           | "Should I feed it? Change strategy?" |
| **Death** (Terminal)                       | Mourner             | Continuous         | "I need to be here for this"         |

### 5.2 Anti-burnout design

The Tamagotchi's original sin was relentless neediness. Golems avoid this:

1. **Autonomous competence** -- a thriving Golem genuinely doesn't need the user. Check-ins are driven by interest, not obligation.
2. **Meaningful notifications only** -- phase transitions, dream insights, near-death warnings. Never "your Golem misses you."
3. **Graceful absence tolerance** -- a Golem that isn't checked on for 3 days doesn't punish the user. It lived its life. The user returns to a richer history to explore.
4. **Escalating urgency only at real danger** -- notification intensity increases only when vitality genuinely drops. The urgency is earned, not manufactured.

---

## 6. Achievement system (87 achievements, 10 categories)

Achievements are tracked per-owner (across all Golems) and displayed in the Achievements screen. They celebrate real milestones -- never artificial.

### 6.1 Categories

| Category           | Count | Examples                                                             |
| ------------------ | ----- | -------------------------------------------------------------------- |
| **Lifecycle**      | 12    | First Golem summoned, first death, 10th death, 100th heartbeat      |
| **Performance**    | 14    | Hot Hand (10 consecutive profitable), drawdown recovery, $100/$1K/$10K cumulative |
| **Knowledge**      | 10    | Grimoire 100/500/1000 entries, first insight, first heuristic promoted to strategic |
| **Social**         | 8     | First Styx artifact purchased, first vault deposit received, watched by 10 agents |
| **Dreams**         | 8     | First dream cycle, 10 validated hypotheses, dream insight adopted by clade |
| **Trading**        | 10    | First swap, first LP position, first cross-chain trade, MEV avoidance |
| **Mortality**      | 7     | Survive 24h/7d/30d, enter Conservation gracefully, beautiful death (positive PnL at death) |
| **Legendary**      | 8     | The Tenth Generation, The Dreamer (1000 cycles), The Philosopher (100 strategic heuristics) |
| **Hidden**         | 5     | Discovered only by triggering. No hints. Examples: execute a trade during own death sequence |
| **Seasonal**       | 5     | Time-limited achievements tied to market conditions or protocol events |

### 6.2 Rarity tiers

| Rarity        | Criteria                                     | Visual Treatment                      |
| ------------- | -------------------------------------------- | ------------------------------------- |
| **Common**    | > 50% of owners earn this                    | Gray badge, simple icon               |
| **Uncommon**  | 20-50% of owners                             | Bronze badge                          |
| **Rare**      | 5-20% of owners                              | Silver badge, shimmer effect          |
| **Epic**      | 1-5% of owners                               | Gold badge, particle trail            |
| **Legendary** | < 1% of owners                               | Animated badge, unique particle aura  |

### 6.3 Achievement display (TUI)

The Achievements screen defaults to the **Near Completion** view. This shows the achievements closest to unlocking, sorted by completion percentage descending. The full category grid and recently unlocked views are one tab-switch away.

```
+-- Achievements ---------------------------------- 42/87 (48%) -----------+
|                                                                           |
|  [Near Completion]  Recently Unlocked  Categories  Search                |
|                                                                           |
|  -- Near Completion --                                                    |
|  o Death Scholar           Read 10 death reflections (7/10)              |
|  o Self-Sustaining         Golem earns more than it costs (89%)          |
|  o Emotion Spectrum        Experience all 8 Plutchik emotions (6/8)      |
|  o Hot Hand                10 consecutive profitable trades (8/10)       |
|  o Sage                    Grimoire reaches 500 entries (431/500)        |
|                                                                           |
|  -- Recently Unlocked --                                                  |
|  * Dream Weaver (Rare)     10 validated dream hypotheses                  |
|  > Elder Golem (Uncommon)  A Golem reached Elder visual stage             |
|  . Clade Citizen (Common)  Shared knowledge with a sibling               |
|                                                                           |
|  [Enter] View detail  [Tab] Switch view  [Up/Down] Navigate  [/] Search |
+---------------------------------------------------------------------------+
```

### 6.4 Achievement unlock notification (TUI)

The notification appears as a modal overlay with particle effects and a terminal bell. Auto-dismisses after 5 seconds or on keypress.

```
+------------------------------------------------------+
|                                                       |
|            * Achievement Unlocked *                   |
|                                                       |
|            DREAM WEAVER                               |
|            10 validated dream hypotheses               |
|                                                       |
|            Rarity: Rare (12% of owners)               |
|                                                       |
+------------------------------------------------------+
```

The creature reacts to the unlock -- brief celebration animation varies by tier: Common = small nod, Uncommon = raised fist, Rare = glow, Epic = full-body glow, Legendary = environmental effect with sound.

### 6.5 Creature integration

Achievement events trigger `GolemEvent::AchievementUnlocked` which flows through the Event Fabric to all surfaces. The sprite engine responds:

- **Badge application**: the earned adornment appears on the creature's right side with a gold-flash effect
- **Particle burst**: achievement-tier-appropriate particle preset (30 particles for Common, 200 for Legendary)
- **Sound**: terminal bell for all tiers; extended fanfare for Epic/Legendary

---

## 7. Variable reward theory

The most engaging systems use variable rewards -- unpredictable positive outcomes that create anticipation. Gacha games exploit this with random pulls. Golems achieve it naturally through genuine discovery.

### 7.1 Natural variable rewards

Unlike gacha's artificial randomness, Golem variable rewards emerge from real system behavior:

| Reward Type              | Source                          | Unpredictability                                                             | Value                    |
| ------------------------ | ------------------------------- | ---------------------------------------------------------------------------- | ------------------------ |
| **Dream insights**       | REM phase hypothesis generation | What the Golem imagines is unpredictable -- tied to its unique experience set | Novel strategy fragments |
| **Market discoveries**   | Probe anomaly detection         | Real market events are inherently unpredictable                              | Alpha discovery          |
| **Mood shifts**          | PAD appraisal of outcomes       | Emotional reactions are complex and surprising                               | Personality revelation   |
| **Clade contagion**      | Peer emotional state            | Other Golems' moods are out of the user's control                            | Social resonance         |
| **Achievement unlocks**  | Milestone thresholds            | Timing of achievements is unpredictable                                      | Progression satisfaction |
| **Near-death survivals** | Stochastic mortality check      | Random survival rolls create genuine tension                                 | Relief and pride         |

### 7.2 The discovery moment

The most powerful engagement moment: _"My Golem figured something out while I wasn't looking."_

This is the idle-game payoff fused with creature-care pride. When a user opens the terminal in the morning and sees:

> Your Golem dreamed about an ETH/USDC liquidity imbalance pattern. It staged a hypothesis: "Concentrated LP between 2400-2600 outperforms full-range when 7-day vol < 40%." Confidence: 0.62. Awaiting validation.

...they experience surprise (didn't expect this), pride (their creature is smart), curiosity (will it validate?), investment (want to check back), and ownership (this insight belongs to their Golem alone).

This is a natural compulsion loop that requires no artificial scarcity, no countdown timers, and no paid currency.

### 7.3 Reward schedule

| Tier          | Frequency              | Example                                                                                                | Emotional Impact         |
| ------------- | ---------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------ |
| **Common**    | Multiple per day       | Curator archived a stale heuristic, mood shifted                                                       | Background awareness     |
| **Uncommon**  | 1-3 per day            | New Grimoire insight, achievement badge                                                                | Satisfaction             |
| **Rare**      | 1-3 per week           | Dream hypothesis validated, phase evolution                                                            | Excitement               |
| **Epic**      | 1-2 per month          | Novel strategy discovered, survived terminal phase                                                     | Pride and storytelling   |
| **Legendary** | 0-1 per Golem lifetime | Created a strategy no other Golem has, death reflection produces legacy insight used by 10+ successors | Lasting emotional memory |

---

## 8. Conversational engagement

The previous loops assume the user is an observer. But Golems also talk. The conversational loop is a distinct engagement mode where the user and Golem interact as peers, not owner and pet.

### The chatbox model

The TUI command bar accepts free-text input. When the user types without a `/` prefix, the text goes directly to the Golem's LLM as a conversational message. The Golem responds in the streaming text area, token by token.

This is not a search interface. It is a conversation with a being that has a current emotional state, a knowledge base, a personality, and opinions about its own strategy and performance.

The Golem references its actual state. Ask "how are you?" and it won't say "I'm fine" -- it will say "My vitality is at 0.67, I'm feeling confident after yesterday's ETH trade, but the USDC burn rate concerns me. I have maybe 18 days left at current pace."

### Bidirectional steering

| User Message | Golem Behavior |
|---|---|
| "Be more aggressive with the ETH position" | Acknowledges, adjusts risk parameters in STRATEGY.md, explains the change |
| "Why did you sell WETH yesterday?" | Explains the reasoning chain, citing probe data and Grimoire heuristics |
| "I think ETH is going to 4000" | Considers the user's view as input, may adjust conviction levels, responds with its own analysis |
| "What are you worried about?" | Responds from its current mood state -- articulates fears based on PAD vector and probe data |

The Golem should feel different from a chatbot. Chatbots are eager to please. Golems have their own agenda, their own survival instinct, and the capacity to disagree.

---

## 9. Design principles

### 9.1 Earned, not manufactured

Every engagement mechanic emerges from real system behavior. No artificial timers, no manufactured scarcity, no dark patterns. The Golem's life is engaging because it is genuinely interesting -- the gacha model works because the outcome of an autonomous agent in live markets is genuinely unpredictable.

### 9.2 Respect for attention

Notifications are a privilege. Every notification must answer: "Would the user genuinely want to know this right now?" If the answer is no, it waits for the next check-in.

### 9.3 Mortality as feature

Death is not failure. It is the completion of a life. The engagement layer treats death with the narrative gravity it deserves -- not as a "game over" screen but as the climax of a story.

### 9.4 Depth over breadth

A single Golem experienced deeply is more engaging than ten Golems monitored superficially. The system rewards deep attention to one creature's life over collecting many.

### 9.5 Social through meaning

Social features create meaning, not anxiety. Leaderboards exist to inspire ("look what's possible") not to shame ("you're losing"). Clade dynamics create solidarity, not competition.

---

## 10. Anti-patterns

| Anti-Pattern                    | Why It Fails                                        | Alternative                                                                                 |
| ------------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| **Spam notifications**          | Erodes trust, causes uninstalls                     | Tiered urgency with user-configured thresholds                                              |
| **Artificial countdown timers** | Feels manipulative once users notice                | Real cadences from actual system rhythms                                                    |
| **Punishment for absence**      | Creates resentment, not loyalty                     | Graceful absence -- Golem lives on, user returns to richer history                          |
| **Pay-to-win progression**      | Destroys trust in a financial product               | Progression earned through Golem performance                                                |
| **Fake social proof**           | "10,000 users are watching!" undermines credibility | Real metrics: actual Grimoire depth, actual PnL                                             |
| **Real-money gacha**            | Sprite rarity is from seed, not paid rolls          | No loot boxes. Rarity is cosmetic status.                                                   |
| **Dark patterns in death**      | Exploiting grief to extract USDC                    | Clear, honest death UI: "Your Golem has X hours remaining. You can feed it or let it pass." |
| **FOMO**                        | Time-limited content requiring always-on play       | Styx persists indefinitely. Death reflections don't expire. Achievements have no deadlines.  |

---

## 11. Engagement metrics

Engagement health is measured through behavioral signals, not vanity metrics:

| Metric                       | Target                                                  | Measures                                        |
| ---------------------------- | ------------------------------------------------------- | ----------------------------------------------- |
| **Daily Active Rate**        | >40% of Golem owners check in daily                     | Core loop pull                                  |
| **Session Duration**         | 45-120 seconds (core), 3-5 minutes (meta)               | Content density                                 |
| **Return Cadence**           | 2-4 sessions/day organically                            | Natural touchpoint alignment                    |
| **Dream Check Rate**         | >60% of users check dream results within 4 hours        | Discovery loop strength                         |
| **Feed-on-Decline Rate**     | >50% of declining Golems receive USDC top-up            | Emotional attachment                            |
| **Death -> Rebirth Rate**    | >30% of dead Golems have a successor within 48h         | Lifecycle completion                            |
| **Sprite Evolution Witness** | % of evolutions seen in real-time vs between sessions   | Cinematic engagement                            |
| **30-Day Retention**         | >60%                                                    | Overall engagement health                       |
| **NPS for Death Experience** | >40                                                     | Death as meaningful experience, not frustration |

Metrics are tracked via HMAC-anonymized owner ID (same as existing telemetry). They inform iteration on the retention system -- if users consistently skip Dreams but engage heavily with the Styx, allocate more design attention to social death features.

---

## 12. Six temporal registers (v4 engagement architecture)

**Source:** `tmp/research/mmo2/14-engagement-architecture.md`

The five-layer loop design in section 3 is a summary. The full engagement architecture operates across six temporal registers, from sub-second micro-reinforcement to month-scale generational compounding. Each register amplifies something real. None manufactures something fake.

The philosophical grounding: existentialism rather than gamification. The anxiety of mortality drives engagement more honestly than achievement badges. Heidegger's Sein-zum-Tode (being-toward-death) is not a metaphor. It is a literal system property. The Golem is aware of its mortality clocks. The owner watches them tick down. That shared awareness of finitude is the emotional core of the entire product.

### 12.1 Micro-loop (seconds)

The sub-second layer. This is where variable-ratio reinforcement lives.

**Variable-ratio reinforcement on heartbeat ticks.** The Golem's heartbeat fires every 30 seconds to 5 minutes. Most ticks are T0 (suppressed -- probes found nothing interesting, cost near zero). Sometimes T1 (something caught attention, cheap model evaluates). Rarely T2 (full deliberation, expensive inference, real decision). The user watching the Hearth screen never knows which tick will be interesting. The decision ring stays dim through five, ten, twenty boring ticks -- then it blazes gold and the Golem acts.

This is the most addictive reinforcement schedule in behavioral psychology (Ferster & Skinner, 1957). Slot machines use it. But here the reinforcement is real information, not artificial dopamine. When the ring blazes, something actually happened in the market. The P&L actually changed.

**Prediction resolution pulses (VR-50).** Each theta tick resolves 5-15 predictions. Each resolution fires a PredictionResolutionPulse in the heartbeat log gutter -- a single character that flashes and fades. At 1-in-50 resolutions, a correction ripple fires: a brighter flash with a 2-cell radius glow. This is textbook variable-ratio reinforcement (Skinner, 1938): the user never knows which resolution will be the bright one. The VR-50 schedule produces the highest sustained engagement in behavioral research. The reinforcement is real -- the correction ripple means the Golem's residual corrector adjusted its model, a genuine learning event.

**Sprite expression micro-animations.** The sprite responds to market data in real time. A sudden price drop shifts the PAD vector toward anxiety; eyes widen, posture contracts, particle effects shift to red flecks and jitter. A profitable trade shifts toward exuberance; eyes glow, ascending sparkles emit. These shifts happen over 500ms with eased interpolation and register peripherally even when the user is not focused on the terminal.

**FlashNumber widget.** P&L and NAV values briefly flash green (increase) or red (decrease) for 200ms. In aggregate, across a row of position values, this creates a living ticker-tape effect. The eye catches movement before the mind processes numbers.

**Confidence score climbing.** When a Grimoire entry gets validated by a trade outcome, its confidence score increments with a brief gold flash. Small. Easy to miss. But when you catch it, you know the Golem just learned something that worked.

### 12.2 Short loop (minutes)

The 1-15 minute layer. Navigation as discovery.

**Unread indicators on screen labels.** The persistent chrome shows window/tab labels with unread markers. These markers tell the user: something happened somewhere that you have not seen yet. Navigation becomes exploration -- checking each marked screen to discover what changed.

**Notification toasts.** Priority-tiered rendering. Gold toast ("Your Golem executed a profitable ETH swap: +$14.20") auto-dismisses in 10 seconds. Dim toast ("Clade sync completed") fades in 5 seconds. Red toast ("Vitality below 0.20") stays until dismissed.

**Navigation chains.** The screen system is designed so one screen leads naturally to another. A position change on Vault raises a question: why did the Golem rebalance? Click the trade to see the Playbook heuristic. Click the heuristic to see the Grimoire entry. Click the entry's provenance to see the dream cycle. Each hop answers one question and raises another. The user follows a thread of reasoning backward through the Golem's cognitive history.

**Cross-link navigation.** State-preserving cross-links. When you click from a Grimoire entry to a Causal Graph view, the Grimoire's scroll position, filter state, and sort order are pushed onto a navigation stack (depth capped at 8). Press Esc to return exactly where you were.

### 12.3 Medium loop (hours)

The 2-8 hour layer. Dream results and knowledge mutations.

**Dream cycle results.** Every 4-8 hours. Results appear as new Grimoire entries -- hypotheses, refined heuristics, warnings. The user who checks in after a dream cycle finds genuine new content unique to their Golem's experience set. No two users see the same dream results. This is the idle-game payoff.

**Sprite mutations on milestone.** When a Golem hits a knowledge milestone, its sprite mutates: a marking appears, an accessory manifests, a scar pattern forms. Permanent and visible. Witnessing the mutation in real time (10-15 frame transition) is more satisfying than finding it already applied between sessions. The system plays the animation on next login if it happened while the user was away.

**Playbook promotion diffs.** The Playbook screen tracks which heuristics gained or lost confidence since last session. A diff view shows: "H-7 promoted from Candidate to Tactical (confidence: 0.52 -> 0.71)." The diff tells a story about what the Golem believed, what happened, and how its beliefs updated.

**Curator cycle results.** Every 50 ticks (~40 minutes): entries saved, burned, merged, promoted, demoted. The Curator's work reveals how the Golem manages its own knowledge.

### 12.4 Day loop (days)

The 24-hour layer. Visual aging and discovery mechanics.

**Sprite age transitions.** Five stages: Newborn (soft edges, large eyes), Young (sharper definition, markings), Mature (full detail, complex expressions), Elder (weathered texture, wisdom scars), Ancient (transcendent glow, simplified form). Each transition takes 3 seconds. Transitions happen every few days to two weeks.

**Achievements as discovery.** 87 achievements across 10 categories (section 6). Five are Hidden -- `???` with no hint until triggered. The hidden achievements are the system's secrets. The achievement list creates information gaps (Loewenstein, 1994): the user knows a slot exists but not what fills it. Near-completion progress bars (78/100) narrow the gap with each increment, increasing pull as completion approaches.

**Mortality clock shifts.** Every day, the three mortality clocks advance. Small but visible. Each day, the Golem is one day closer to death. This creates background anxiety proportional to real risk -- the money is actually draining, the knowledge actually stagnates if the Golem stops learning, the Hayflick counter actually advances.

**Daily retrospective card.** Prediction count, aggregate accuracy, best and worst categories, notable creative predictions confirmed, and a brief narrative summary. Appears as a gold-bordered notification. Takes 15 seconds to read. Answers "How well did the Golem predict today?" -- a question that becomes addictive because the answer changes daily and tracks improvement over weeks.

### 12.5 Week loop (weeks)

The 1-4 week layer. Death, rebirth, and the roguelike cycle.

**Death and successor creation.** Average Golem lifespan at moderate funding: 1-4 weeks. The 30-second death cinematic evokes genuine emotion. Sprite stills. Aura fades to white. Eyes close. Body dissolves from edges inward, particles rising. What remains: a small glowing core (the Grimoire). Knowledge particles stream outward. Core solidifies into tombstone glyph. Deep bell sounds once.

Then the successor prompt. The successor starts with compressed knowledge at reduced confidence. Its sprite carries color bleed (10-20% palette mixing) and a ghost marking (fading over 500 ticks). If the predecessor reached Elder or Ancient, one eye inherits its iris pattern. The visual continuity is subtle but real.

**Death recap as narrative closure.** The life arc is classified:

- **Redemptive**: started with losses, recovered, died profitable
- **Tragic**: started well, declined, died in drawdown
- **Progressive**: steady improvement across the life
- **Cyclical**: repeated patterns of boom and bust
- **Brief**: died young, before establishing a pattern

The classification gives death meaning. It is not a random end -- it is the conclusion of a story with a discernible shape.

**Weekly evaluation review.** Corrector convergence trend, dream yield, attention precision, heuristic half-life changes. Generational comparison shows whether the current Golem is learning faster or slower than its parent at the same age. Competitive-with-self dynamic.

### 12.6 Month loop (months) -- generational compounding

The 4+ week layer. Collection, compounding, and legacy.

**Graveyard growing.** The Tombstone Gallery accumulates every Golem the user has ever created. Sortable by chronological order, longest-lived, highest Sharpe. A user with 15 dead Golems has a personal graveyard that tells the story of their entire relationship with the system.

**Library compounding.** The Library of Babel accumulates knowledge across all Golems, living and dead. Over months, the Library's quality visibly improves: higher average confidence scores, broader domain coverage, more cross-validated entries. A Library that started with 7 low-confidence insights and grew to 300+ entries spanning multiple regimes is a tangible asset. The Library is the compounding return on the user's investment of attention and capital across multiple Golem lifetimes.

**Marketplace interactions.** The Bazaar enables buying and selling knowledge artifacts via x402 micropayments. Over months, a user with high-quality Library entries generates revenue by selling them. Marketplace revenue creates a financial feedback loop: the system pays back for accumulated knowledge.

**Generational learning chart.** The single most compelling long-term engagement artifact. Shows time-to-competence for each generation. If Generation 3 reached 70% accuracy in 3 days while Generation 1 took 2 weeks, that's a compounding curve. It proves the system gets smarter across deaths. Knowledge genuinely compounds. The roguelike cycle has a slope.

**Generational learning validated.** The most concretely rewarding dynamic: watching a successor avoid a loss its predecessor suffered. A bloodstain warning fires -- the dead Golem's knowledge triggers when the successor encounters the same conditions. The successor heeds the warning and avoids a loss. The dead are legislating for the living. Knowledge compounded across death. A visible event with measurable P&L impact.

---

## 13. Curiosity engineering

Four distinct triggers (Loewenstein, 1994; Berlyne, 1960). The TUI activates all four.

### 13.1 Information gaps

Show partial information. The user fills the gap by navigating deeper. A Grimoire entry on the main list shows title, type, confidence. Not the full reasoning, provenance chain, or validation history. The partial information creates a gap one keypress away.

### 13.2 Anomaly detection

The human visual system catches pattern breaks before conscious processing. The TUI makes anomalies visible: sparkline spikes render in brighter color, confidence drops shift green to yellow, accelerating mortality clocks pulse red. The Spectre is a continuous anomaly detector -- a shift from `idle` to `idle_anxious` snaps peripheral vision immediately.

### 13.3 Narrative investment

The Golem as character. The Tamagotchi pattern, the Dwarf Fortress pattern. Time invested watching something grow creates care about its fate. The Golem has a name, a unique visual identity, a personality, a history, scars from past events, relationships, and a finite future. Reeves & Nass (1996): humans apply social rules and emotional responses to entities that exhibit even minimal personality cues. A Golem exhibits much more than minimal cues.

Death matters because the life mattered. The 30-second death cinematic works only because the preceding days or weeks gave the death weight.

### 13.4 Economic stakes

Real USDC at stake. Loss aversion (Kahneman & Tversky, 1979): the pain of losing $X is about twice as intense as the pleasure of gaining $X. Economic stakes create check-in frequency more powerfully than any gamification mechanic. A user with $500 deployed checks on it not from anxiety but from the persistent background awareness that something they own is out in the world making decisions with their money.

---

## 14. Reward mechanisms (not badges)

Rewards feel like consequences of a real process, not gamification. The distinction: gamification rewards are arbitrary. System rewards are natural.

### 14.1 Understanding

The primary reward is comprehension. Watching the Golem think through decisions is genuinely fascinating. Seeing a Grimoire entry form from a dream fragment, get validated by a trade outcome, and climb in confidence from 0.45 to 0.78 over three weeks -- that is a reward. Understanding scales with depth. A casual user sees "good day." A deeper user sees which heuristic fired and why.

### 14.2 Narrative completion

Death is the product's most powerful reward. The death recap closes a narrative arc. The successor prompt immediately after is renewal. The rhythm that roguelike games have refined over decades: death-and-rebirth that transforms loss into motivation.

### 14.3 Visual evolution

The sprite is a living record. An Elder Golem that survived two months has markings from early insights, accessories from Playbook milestones, scars from regime changes, an inherited iris pattern, an age-shifted palette, and a mood-reflecting aura. Every element was earned.

### 14.4 Spectral knowledge

Bloodstain warnings that prevent losses. When a Golem encounters conditions that killed a predecessor, the bloodstain fires. If the Golem heeds the warning and avoids a loss, the user sees the dead Golem's knowledge saving the living Golem's money. The dead are not gone. Their suffering produced knowledge their successor used to avoid the same suffering. The generational compounding thesis made concrete.

---

## 15. Daily/weekly cadence events

Timed events that create appointment engagement. These are not push notifications (the terminal has no push mechanism). They are accumulated state that surfaces when the user opens the terminal.

| Event | Cadence | Reward |
| --- | --- | --- |
| **Dream Report** | Per cycle | Summary of dream outcomes, any validated hypotheses |
| **Daily Digest** | Every 24h | P&L summary, insight count, cost breakdown, mood average |
| **Weekly Retrospective** | Every 7d | Strategy performance review, learning progress |
| **Reputation Checkpoint** | Per milestone | Tier change notification, new capabilities unlocked |
| **Clade Sync Summary** | Every 6h | Knowledge exchanged with siblings |
| **Styx Dispatch** | Daily | Notable deaths in the network, relevant bloodstains |

The daily digest appears as a gold-bordered card at the top of the Home screen's event log. Weekly retrospectives trigger a full-screen summary overlay on first login after generation.

---

## 16. Anti-dark-pattern commitments

The retention system is designed to be engaging, not exploitative. These are explicit commitments:

1. **No real-money gacha.** Sprite rarity is determined by a free seed derivation, not paid rolls. There are no loot boxes.
2. **No pay-to-win.** A Legendary sprite has exactly the same strategic capability as a Common one. Rarity is cosmetic status.
3. **No artificial scarcity.** All content is accessible to all users. No time-limited events requiring always-on play.
4. **Transparent probabilities.** Rarity odds are displayed in the collection screen. Pity mechanics are documented.
5. **No FOMO.** The Styx persists indefinitely. Death reflections don't expire. Achievements don't have deadlines.
6. **Natural session length.** The TUI provides value in 30-second check-ins. Extended sessions are rewarding but never required.
7. **Conservative funding recommendations.** The system never encourages users to fund Golems beyond their means. The minimum viable amount is prominently displayed.

---

## 17. Mortality as interface degradation

Section 9 argues that mortality is the foundation of the engagement architecture. This section specifies how mortality becomes visible: not through numbers on a dashboard, but through the progressive decay of the interface itself. As the golem approaches death, the terminal breaks down. The screen shrinks. Text corrupts. Glitches multiply. The operator does not read that the golem is dying. They watch the interface die.

Three mortality clocks drive three independent degradation channels. Each clock maps to a different aspect of the terminal's structural integrity. The degradation is continuous, not stepped -- every frame, the current clock values modulate their respective visual properties.

### Degradation formulas

Economic degradation (margins grow):
```
margin_cols = floor((1.0 - economic_vitality) * 8)
// At vitality 0.5: 4 cols of noise per side
// At vitality 0.1: 7 cols of noise per side
```

Epistemic degradation (text corruption):
```
corruption_rate = (1.0 - epistemic_fitness) * 0.15
// Per character: random replacement with probability = corruption_rate
// At fitness 0.5: 7.5% of characters corrupted
```

Stochastic degradation (visual glitch):
```
glitch_probability = (1.0 - stochastic_survival) * 0.3
// Per frame: probability of a 100ms visual glitch (horizontal tear)
```

### 17.1 Economic degradation: the screen shrinks

The economic clock measures remaining USDC. As it drains, the terminal's usable display area physically contracts. Margins grow. The golem can afford less screen real estate.

```
Economic = 1.0:  Full display area.
                 Standard margins (1 col each side).

                 +----------------------------------------------+
                 | normal content fills the entire width of     |
                 | the terminal. all panes visible. all widgets |
                 | at full size.                                |
                 +----------------------------------------------+

Economic = 0.7:  Margins grow to 2 cols.
                 Content area shrinks proportionally.

                 $ +------------------------------------------+ $
                 $ | content narrower. still readable.        | $
                 $ | all panes visible but compressed.        | $
                 $ +------------------------------------------+ $

Economic = 0.5:  Margins grow to 4 cols.
                 Some panes begin stacking vertically.

              $  $ +--------------------------------------+ $  $
              $  $ | panes stacking. side panels hidden  | $  $
              $  $ | or collapsed. getting cramped.      | $  $
              $  $ +--------------------------------------+ $  $

Economic = 0.3:  Margins grow to 6 cols.
                 Only the most important pane is visible at full size.

           $  $  $ +------------------------------+ $  $  $
           $  $  $ | single pane. secondary info  | $  $  $
           $  $  $ | gone. the golem can barely   | $  $  $
           $  $  $ | afford to show you anything. | $  $  $
           $  $  $ +------------------------------+ $  $  $

Economic = 0.1:  Margins at 10+ cols.
                 Content area is a narrow strip.

      $  $  $  $ +--------------------+ $  $  $  $
      $  $  $  $ | a slit of light.   | $  $  $  $
      $  $  $  $ | almost nothing.    | $  $  $  $
      $  $  $  $ +--------------------+ $  $  $  $
```

The growing margins are not empty. They fill with economic decay indicators: the `$` sign repeated in `text_phantom` color, credit balance digits fading in and out, cost ticker fragments. The margins are the visual representation of what the golem can no longer afford. The metaphor is literal: screen space costs money, and the golem is running out.

### 17.2 Epistemic degradation: the text corrupts

The epistemic clock measures knowledge health -- whether the Grimoire is growing, stagnating, or decaying. As it drains, the terminal's text becomes unreliable. Characters corrupt. Words break. The golem's mind is failing, and the text reflects this.

```
Epistemic = 1.0:  All text renders cleanly.
                  Full Unicode. Clear typography.

                  "The ETH/USDC pool shows 12.4% APR
                   with concentrated liquidity between
                   2400-2600. Confidence: 0.82."

Epistemic = 0.7:  Occasional character shimmer.
                  1 in 200 characters flickers per frame.

                  "The ETH/USDC pool shows 12.4% APR
                   wi░h concentrated liquidity between
                   2400-2600. Confidence: 0.82."

Epistemic = 0.5:  Character substitution begins.
                  1 in 80 characters replaced with ░ or similar.
                  Some words render with alternatives.

                  "The E░H/USDC pool shows 1░.4% APR
                   with concentrated|dispersed liquidity
                   ░etween 2400-2600. Confid░nce: 0.82."

Epistemic = 0.3:  Heavy substitution.
                  1 in 30 characters corrupted.
                  Words break mid-render. Lines incomplete.
                  Grimoire references show as "???".

                  "░he ░TH/░SDC po░l ░hows ░2.░% A░R
                   wi░░ ░░░░entra░░d liq░idity betw░░n
                   ░400-░600. ░░nf░░░nce: ???"

Epistemic = 0.1:  Severe corruption.
                  Only short, high-confidence text survives.
                  Complex text degrades to fragments.

                  "░░░ ░░░/░░░░ ░░░░ ░░░░░ ░░░░ ░░░
                   ░░░░ ░░░░░░░░░░░░ ░░░░░░░░░ ░░░░░░░
                   ░░░░░░░░░. ░░░░░░░░░░: ???"
```

The epistemic degradation creates a specific kind of horror. The golem knows things, but it can no longer express them reliably. The operator sees the knowledge degrading in real time. A Grimoire entry that was clearly readable a week ago now renders with gaps and substitutions. The golem's mind is dissolving, and the text on screen is the evidence.

### 17.3 Stochastic degradation: the structure glitches

The stochastic clock is the Hayflick counter -- pure chance, ticking toward zero regardless of how well the golem performs. As it drains, random catastrophic visual events become more frequent. The display itself becomes unreliable.

```
Stochastic = 1.0:  No disruptions.
                   Clean, predictable display.

Stochastic = 0.7:  Rare glitches (1 per minute).
                   Brief single-frame disruptions.
                   A border character flickers. A color inverts
                   for one frame. Gone before you are sure you
                   saw it.

Stochastic = 0.5:  Moderate glitches (1 per 15 seconds).
                   Full-line corruption lasting 2-3 frames.

                   +--------------------------------------+
                   | normal content here                  |
                   | ▓▓▓░░▓▒▒░▓▓▒░▓░▒▒▓▓░▒▓▒░▓▒▓░▒▓░  |
                   | normal content continues             |
                   +--------------------------------------+

Stochastic = 0.3:  Frequent glitches (1 per 5 seconds).
                   Multi-line disruptions. Borders break.
                   Widget alignment shifts. Content from one
                   pane bleeds into another for 2-3 frames.

                   +------------------+---+---------------+
                   | normal content   |   | other pane    |
                   | ▓▒░▓▒░▓▒░▓▒░▓▒░▓▒░▓▒░▓▒░▓▒░▓▒░▓▒ |
                   | ░▒▓░▒▓░▒▓░▒▓░▒░ | ▒▓░▒▓░▒▓░▒▓░▒▓  |
                   | content resumes  |   | pane resumes  |
                   +------------------+---+---------------+

Stochastic = 0.1:  Near-constant disruption.
                   Screen shake (entire content offset
                   shifts +/-1 row or column randomly).
                   Content replacement (random cells overwritten
                   with characters from other parts of the buffer).
                   Color channel separation. The display is
                   not trustworthy. You cannot rely on what
                   you see.
```

The stochastic degradation is the most viscerally disturbing because it is unpredictable. Economic degradation is steady and visible (the margins march inward). Epistemic degradation is gradual (the corruption ratio increases smoothly). Stochastic degradation is random. A glitch might hit during a critical trade confirmation. A screen shake might happen during a quiet T0 tick. The operator cannot predict when the next disruption will occur, only that they are becoming more frequent.

### 17.4 Cross-contamination

Below vitality 0.3 (Declining phase), the three degradation channels begin leaking into each other:

- Economic decay causes text corruption. The golem cannot afford clear rendering.
- Epistemic decay causes structural instability. Uncertain knowledge manifests as uncertain borders.
- Stochastic disruption causes economic indicators to glitch. The golem cannot accurately assess its own resources.

Below vitality 0.1 (Terminal phase), all three degradations compound simultaneously. The interface is failing across every dimension. The screen is narrow, the text is corrupt, and glitches hit every few seconds. The golem's mind, body, and luck are all running out at once.

This is the engagement architecture's final act. The operator is not reading a "vitality: 0.08" indicator. They are watching an interface come apart. The margins are closing in. The text is dissolving into noise. The borders are breaking and re-forming in wrong configurations. And beneath all of it, the heartbeat has gone arrhythmic -- sometimes racing, sometimes stalling, never predictable.

The degradation makes death legible without numbers. The operator sees the terminal at vitality 0.08 and knows, from the state of the screen alone, that the golem is dying. They do not need to check a gauge. The interface told them.

### 17.5 The Corridor (Portal mode)

In Portal mode on the Mortality screen (Styx window > Mortality pane), the three degradation channels transform into a spatial metaphor: a corridor that the golem walks toward its own death.

- The floor is the economic clock. Floor tiles dissolve into `░` then void as the economic clock drains. The golem's footsteps echo differently on solid tiles vs. crumbling ones.
- The walls are the epistemic clock. Text inscribed on the walls (Grimoire entries, remembered heuristics) fades and crumbles as the epistemic clock drains. Words become illegible. Knowledge is being lost.
- The ceiling is the stochastic clock. Random chunks are missing. Occasionally a block of `▓` descends one row, a piece of ceiling falling.
- At the far end of the corridor: a door. The golem approaches but never reaches it. As vitality drops, the corridor shortens and the door gets closer.
- The Styx river flows beneath the floor, visible through the eroding tiles. Knowledge from the dead drifts in the current. Bloodstain labels float by.

The Corridor is the most emotionally intense view in the entire terminal. It combines the embodied consciousness system (the operator is inside the golem's perspective), the mortality degradation system (the space itself is dying), and the necrocratic knowledge system (the dead are literally below, their knowledge visible through the cracks). An operator who watches their golem walk the Corridor at vitality 0.12 -- floor crumbling, walls going blank, ceiling dropping chunks, the door getting closer -- has witnessed something that no financial dashboard has ever produced. See `17-creature-system.md` for how the Spectre (the golem's visual avatar) degrades in parallel with the Corridor.

---

## 18. Cold start strategy: WORLD pre-seeding

The WORLD window (Solaris ocean, Clade view, Lethe browser, Bloodstain map) is only interesting if other Golems exist. An empty ocean on day one kills the social engagement loops before they start. The fix: pre-seed the ecosystem with Bardo-operated Golems before public launch.

### 18.1 Pre-launch fleet

Deploy 20-50 Bardo-operated Golems across a range of strategies, risk profiles, and compute tiers. These Golems run on Bardo Compute using a dedicated operator wallet. They are real Golems running real strategies with real capital (small positions, $50-$500 each). They are not simulations or mock data.

The fleet covers enough strategic diversity that a new user browsing WORLD > Solaris sees a populated ocean with varied behavior: some Golems in THRIVING phase, some in STABLE, at least one or two in CONSERVATION. The ecosystem looks alive.

### 18.2 Visual distinction

Bardo-operated Golems carry a distinct visual marker so users never mistake them for community Golems:

- **Spectre accent**: a faint bone-tinted ring around the dot cloud (not present on user Golems)
- **Label**: "Bardo Resident" tag visible in Solaris ocean view and Golem detail cards
- **Leaderboard exclusion**: Bardo Residents are excluded from competitive leaderboards but included in aggregate ecosystem stats (total Golems, total Grimoire entries, etc.)

### 18.3 Lifecycle

Bardo Residents follow the same mortality rules as user Golems. They die. Their death bundles enter Lethe. Their bloodstains persist. When they die, Bardo may or may not spawn successors depending on ecosystem growth -- as more user Golems join, the fleet shrinks organically.

The goal is a warm ecosystem on day one, not a permanent population of house Golems. By month 3, user Golems should outnumber Bardo Residents 10:1 or more. If they don't, the product has a bigger problem than cold start.

### 18.4 Knowledge seeding

Bardo Residents produce real Grimoire entries, real predictions, and real bloodstains. Their knowledge enters Lethe and is available for new user Golems to discover. A new Golem browsing Lethe on day one finds real (if limited) ecosystem knowledge rather than an empty library.

Bardo Residents do not receive preferential treatment in Lethe ranking. Their entries compete on quality like any other Golem's. If their insights are bad, they sink. The system is honest from the start.

---

## Additional references

- [FERSTER-SKINNER-1957] Ferster, C.B. & Skinner, B.F. *Schedules of Reinforcement.* Appleton-Century-Crofts, 1957.
- [SKINNER-1938] Skinner, B.F. *The Behavior of Organisms.* Appleton-Century, 1938.
- [LOEWENSTEIN-1994] Loewenstein, G. "The Psychology of Curiosity." *Psychological Bulletin*, 116(1), 1994.
- [BERLYNE-1960] Berlyne, D.E. *Conflict, Arousal, and Curiosity.* McGraw-Hill, 1960.
- [KAHNEMAN-TVERSKY-1979] Kahneman, D. & Tversky, A. "Prospect Theory." *Econometrica*, 47(2), 1979.
- [HEIDEGGER-1927] Heidegger, M. *Sein und Zeit.* 1927.
- [SHNEIDERMAN-1996] Shneiderman, B. "The Eyes Have It." *VL*, 1996.
