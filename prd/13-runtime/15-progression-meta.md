# Progression, collection, and the meta-game [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document defines the meta-game layer that gives a Golem's (a mortal autonomous DeFi agent) life long-term meaning: the Grimoire (persistent knowledge base) as collectible card system, 87 achievements across 10 categories, reputation tiers unlocking creature visuals, sprite collection, tombstone gallery, and generational compounding. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: the engagement loops from `13-engagement-loops.md` and the Grimoire data model. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

The meta-game is everything that makes a user care about their Golem beyond the next tick. It transforms ephemeral runtime events into persistent, collectible, meaningful artifacts -- a Grimoire that grows like a Pokedex, 87 achievements that mark real milestones, a reputation ladder that unlocks new creature visuals, and a tombstone gallery that turns death into legacy.

> **Cross-references:**
>
> - `./11-state-model.md` — GolemSnapshot read-only projection, specifically PerformanceState (P&L, Sharpe, trades) and MemoryState (Grimoire entry counts)
> - `./13-engagement-loops.md` — meta engagement loop design and the achievement category definitions that this document expands
> - `./14-creature-system.md` — the Spectre visual creature system: evolution stages, earned marks, and visual progression tied to achievements
> - `./16-social-competitive.md` — Styx-mediated social features: memorials, leaderboards, and graveyard browser showing tombstone gallery
> - `../04-memory/01-grimoire.md` — GrimoireEntry types (episode, insight, heuristic, warning, causal link), decay classes, and confidence lifecycle
> - `prd2/01-golem/10-replication.md` — replicant spawning via MAP-Elites evolutionary search and mutation operators for strategy exploration
> - `prd2/02-mortality/06-thanatopsis.md` — the Thanatopsis death protocol (four-phase structured shutdown) producing the tombstone and death testament
> - `../09-economy/01-reputation.md` — ERC-8004 reputation tiers: how on-chain performance history determines a Golem's public standing

---

## 1. The Grimoire as living collection

The Grimoire is the Golem's knowledge base -- insights, heuristics, causal links, warnings, and strategy fragments accumulated through trading experience and dream cycles. In the engagement layer, the Grimoire becomes a collectible card system where each entry is a discoverable artifact.

### 1.1 Grimoire card types

| Card Type             | GrimoireEntry Type  | Rarity Basis                                      | Visual Treatment                                    |
| --------------------- | ------------------- | ------------------------------------------------- | --------------------------------------------------- |
| **Insight**           | `insight`           | Confidence score x time validated                  | Silver border, text-focused, clean typography       |
| **Heuristic**         | `heuristic`         | Uses in decision-making (upvote count from ExpeL) | Gold border, rule-formatted, impact sparkle         |
| **Warning**           | `warning`           | Severity tier (from probe anomalies)              | Red border, hazard-patterned, urgent glow           |
| **Causal Link**       | `causal_link`       | Graph centrality (how connected it is)            | Web-patterned border, node visualization            |
| **Strategy Fragment** | `strategy_fragment` | Performance attribution (USDC impact)             | Diamond border, strategy-colored, performance trail |
| **Dream Hypothesis**  | `hypothesis`        | Confidence x novelty score                        | Purple border, ethereal particle effect             |
| **Playbook Rule**     | `playbook_rule`     | Derived from PLAYBOOK.md lineage                  | Engraved border, structural weight                  |

### 1.2 Card rarity

Rarity is earned through performance, not randomness. A card's rarity can increase over time as it proves its value.

| Rarity        | Criteria                                                                                              | Visual                                                                 | Approximate Rate |
| ------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ---------------- |
| **Common**    | Default on creation, unvalidated                                                                      | Matte void-dark frame                                                  | ~60%             |
| **Uncommon**  | Validated once (used in decision, positive outcome)                                                   | Raised void frame, faint rose edge                                     | ~25%             |
| **Rare**      | Validated 3+ times with consistent positive outcomes                                                  | Rose frame, soft glow                                                  | ~10%             |
| **Epic**      | High-impact: attributed to >$100 cumulative PnL improvement                                          | Rose-bright frame, particle effect, achievement notification           | ~4%              |
| **Legendary** | Unique to ecosystem: no other Golem has produced this exact insight with this performance attribution | Animated bone frame, starlight particles, global notification to clade | <1%              |

### 1.3 Collection interface (TUI)

```
+-- GRIMOIRE ------------------------------------------- 142 entries --+
|                                                                       |
|  Insights (47)  Heuristics (23)  Warnings (18)                       |
|  Causal Links (31)  Hypotheses (15)  Fragments (8)                   |
|                                                                       |
|  +-----+ +-----+ +-----+ +-----+ +-----+ +-----+                   |
|  | *** | | **  | | *   | | *** | | *   | | **  |                   |
|  |     | |     | |     | |     | |     | |     |                   |
|  | ETH | | Gas | | Vol | | LP  | | Mkr | | UNI |                   |
|  |Corr.| |Spik.| | Reg | | Rng | | Sig | | Fee |                   |
|  +-----+ +-----+ +-----+ +-----+ +-----+ +-----+                   |
|                                                                       |
|  Sort: Newest | Rarest | Most Used | Highest Impact                  |
|  Filter: By Type | By Provenance | By Decay Class                    |
|                                                                       |
|  Discovery Rate: 2.3 cards/day  |  Legendary: 0                     |
+----------------------------------------------------------------------|
```

### 1.4 Card detail view

Selecting a card reveals its full story. The detail view opens with a 12-frame fade-in transition (~200ms at 60fps): the card grid dims to 30% opacity while the detail panel fades from 0% to 100% opacity over the same 12 frames. The reverse transition plays on `Esc`.

```
+-- *** RARE INSIGHT ---------------------------------------------------+
|                                                                       |
|  "Concentrated LP between 2400-2600 outperforms                       |
|   full-range when 7-day vol < 40%"                                    |
|                                                                       |
|  Discovered: Tick 14,231 (Dream Cycle #7, REM phase)                  |
|  Validated: 3 times across 2 regimes                                  |
|  Confidence: 0.78 (up from 0.62 at discovery)                         |
|  Impact: +$47.23 attributed PnL                                       |
|                                                                       |
|  Provenance: self (dream)                                             |
|  Decay Class: regime-conditional (~14d)                                |
|  Last Used: 2h ago                                                    |
|                                                                       |
|  -- History --                                                        |
|  Discovered during REM as hypothesis                                  |
|  -> Staged for validation at tick 14,250                              |
|  -> First validation: +$12.40 at tick 14,891                          |
|  -> Promoted to heuristic at tick 15,200                              |
|  -> Used in 3 trade decisions since                                   |
|                                                                       |
|  [Favorite]  [Share Card]  [View in Causal Graph]                    |
+-----------------------------------------------------------------------+
```

**Cross-link navigation:** The `[View in Causal Graph]` button switches from the Grimoire collection screen to the Causal Graph view with the selected entry pre-focused. The transition preserves state: the Grimoire's current filter, sort, and scroll position are pushed onto a navigation stack. When the user presses `Esc` or `Backspace` from the Causal Graph, they return to the exact Grimoire position they left. The stack depth is capped at 8 entries; deeper navigation silently drops the oldest entry. This same state-preserving navigation applies to all cross-link actions (`g` key on cross-referenced entries, episode links within insights, etc.).

### 1.5 Ecosystem collection

Beyond individual Golems, the public Grimoire ecosystem tracks what knowledge exists across all Golems:

- **Discovery index** -- total unique insight types discovered. "Your Golem has discovered 47 of 1,234 known insight types."
- **First-discovery credit** -- if your Golem is the first to discover an insight type, it's permanently credited.
- **Provenance tracking** -- when knowledge spreads through clades, the original discoverer is credited.

---

## 2. Achievement system (87 achievements, 10 categories)

Achievements celebrate real milestones. They are never artificial ("log in 7 days") -- every achievement represents something the Golem actually accomplished. Checked against GolemState after each tick; newly unlocked achievements emit `GolemEvent::AchievementUnlocked` to the Event Fabric. Tracked per-owner across all Golems.

### 2.1 Full category breakdown

**Lifecycle** (12 achievements): First Breath (tick 1), First Dream, First Trade, each phase transition, generation milestones (3rd, 5th, 10th generation).

**Performance** (14): Hot Hand (10 consecutive profitable), Cold Streak survivor (5 losses then profit), regime change survival, drawdown recovery, profit milestones ($100, $1K, $10K cumulative).

**Knowledge** (10): Grimoire milestones (100, 500, 1000 entries), first insight generated, first heuristic promoted to strategic, first causal edge discovered, first warning heeded.

**Social** (8): First bloodstain received, first pheromone confirmed by others, clade of 5+, marketplace first sale, marketplace first purchase.

**Dreams** (8): First NREM replay, first REM counterfactual, first integration insight, 100 dream cycles, dream that prevented a loss.

**Trading** (10): First swap, first LP position, first vault deposit, first cross-chain trade, gas optimization milestone, MEV avoidance.

**Mortality** (7): Survive 24h, survive 7d, survive 30d, enter Conservation gracefully, Terminal phase lasting > 12h, beautiful death (positive PnL at death).

**Legendary** (8): The Tenth Generation (lineage survives 10 gen), The Dreamer (1000 dream cycles), The Philosopher (100 strategic heuristics), The Survivor (30d without Conservation), The Socialite (50 marketplace transactions), The Oracle (prediction accuracy > 70% over 100 trades).

**Hidden** (5): Discovered only by triggering. No hints. Examples: execute a trade during own death sequence, receive bloodstain from own ancestor.

**Seasonal** (5): Time-limited achievements tied to market conditions or protocol events.

### 2.2 Achievement detail (examples)

#### Trading milestones

| Achievement       | Condition                                       | Tier   | Visual Reward                               |
| ----------------- | ----------------------------------------------- | ------ | ------------------------------------------- |
| **First Blood**   | First profitable trade                          | Common | Brief bone-colored eye flash (adornment)    |
| **Hundred Hands** | 100 profitable trades                           | Uncommon | Gold bracelet on right wrist              |
| **Sharp Edge**    | Sharpe ratio > 1.0 for 7 consecutive days       | Uncommon | Blade-shaped mark on right shoulder       |
| **Diamond Hands** | Held a losing position that recovered to profit | Common | Small diamond mark on right palm            |

#### Survival milestones

| Achievement      | Condition                                              | Tier      | Visual Reward                                     |
| ---------------- | ------------------------------------------------------ | --------- | ------------------------------------------------- |
| **Survivor**     | Entered Conservation phase and recovered to Stable     | Uncommon  | Kintsugi crack repair visible                     |
| **Phoenix**      | Entered Terminal phase and recovered                   | Rare      | Permanent bone-tinted Dense tier dots, visible through phases |
| **Century**      | Survived 100,000 ticks (Hayflick milestone)            | Rare      | Permanent rose-bright eye glow at idle             |
| **Methuselah**   | Survived 500,000 ticks                                 | Legendary | Full dot cloud at maximum density, bone-tinted eyes |

#### Hidden achievements

| Achievement    | Condition                                                                             | Hint                                   |
| -------------- | ------------------------------------------------------------------------------------- | -------------------------------------- |
| **Night Owl**  | Made a profitable trade during a dream integration phase                              | "Sometimes the best ideas come at 3am" |
| **Contrarian** | Profited from a position that 90% of clade peers held the opposite of                 | "Against the crowd"                    |
| **Last Words** | Death reflection produces an insight used by 10+ successors in the ecosystem          | "What you leave behind"                |
| **Sisyphus**   | Recovered from Conservation phase 5 times without dying                               | "The struggle itself"                  |

### 2.3 Achievement ceremony (TUI)

When an achievement is earned, the unlock ceremony runs for 3 seconds total:

1. **Notification** (0-1s) -- Achievement name + description appears as a modal overlay. A burst of 40 rose-bright particles emits from the Spectre's center. Terminal bell rings once.
2. **Spectre reaction** (1-2s) -- Celebration animation scaled by tier:
   - Common: brief cloud cohesion pulse (4 frames)
   - Uncommon: cloud cohesion + eye brightness spike (6 frames)
   - Rare: full cloud glow, shimmer intensity spikes to max (8 frames)
   - Epic: full cloud glow + rose particle fountain (12 frames)
   - Legendary: environmental effect -- background shifts to starfield, cloud expands to max density, persistent particle trail for 30 seconds after ceremony ends (16 frames)
3. **Visual application** (2-2.5s) -- The earned visual change fades in over 8 frames with a bone-flash overlay that peaks at frame 4 and decays by frame 8.
4. **Social broadcast** (2.5-3s) -- Achievement card generated for optional sharing via Styx. Card render happens async; the ceremony completes regardless of card generation latency.

The ceremony is non-blocking -- tick processing continues behind the modal overlay. If multiple achievements unlock on the same tick (rare but possible), they queue and play sequentially with a 0.5s gap between each.

Achievement badges persist after death -- a dead Golem's achievements transfer to its memorial.

### 2.4 Progress tracking

The Achievements screen (FATE > Achievements) defaults to the **Near Completion** view, not the full category grid. Near Completion shows the 3-5 achievements closest to unlocking, sorted by completion percentage descending. This default drives immediate action ("I'm 78/100 on Hundred Hands") rather than presenting an overwhelming 87-item grid that makes progress feel distant. The user can switch to the full category grid, recently unlocked, or search views via the tab bar at the top of the screen.

Progress meters update at two cadences:

- **Active achievements** (the user is on the Near Completion screen or the achievement is pinned): progress re-evaluated every tick. The `##` bar redraws only if the underlying count changed since last render.
- **Background achievements** (not currently displayed): progress re-evaluated every 10 ticks. This reduces per-tick overhead from checking all 87 achievements to checking only the ~5 pinned or visible ones.

When a progress value changes, the `##` bar animates: the new segment fills with a rose-bright flash that fades to the standard fill color over 4 frames. This gives a visible "tick" of progress even for achievements where the bar barely moves (e.g., 231/500).

```
+-- Near Completion ---------------------+
|                                        |
|  Hundred Hands                         |
|  ####################.....  78/100     |
|  Profitable trades                     |
|                                        |
|  Sage                                  |
|  ###########.............  231/500     |
|  Grimoire entries                      |
|                                        |
|  Stoic                                 |
|  ######################...  22/30 days |
|  Continuous Stable phase               |
+-----------------------------------------+
```

---

## 3. Death recap: the kill screen

When a Golem dies, the death recap is a structured summary rendered as a roguelike kill screen -- inspired by Hades, Slay the Spire, and Risk of Rain 2, where each death is a story worth reviewing.

```rust
#[derive(Clone, Debug, serde::Serialize)]
pub struct DeathRecap {
    // Identity
    pub golem_id: String,
    pub generation: u32,
    pub parent_id: Option<String>,

    // Cause
    pub cause: DeathCause,
    pub cause_narrative: String,  // LLM-generated narrative (Opus, during Thanatopsis)

    // Life statistics
    pub ticks_lived: u64,
    pub total_trades: u32,
    pub profitable_trades: u32,
    pub total_pnl: f64,
    pub max_drawdown: f64,
    pub dream_cycles: u32,

    // Timeline
    pub phase_durations: Vec<(String, u64)>,
    pub regime_distribution: Vec<(String, u64)>,

    // Emotional arc
    pub avg_pleasure: f64,
    pub avg_arousal: f64,
    pub avg_dominance: f64,
    pub dominant_emotion: String,

    // Legacy
    pub genome_entries: u32,
    pub successor_id: Option<String>,
    pub bloodstains_deposited: u32,

    // Final words
    pub death_testament_excerpt: String,

    // Achievements
    pub achievements_this_life: Vec<Achievement>,
}
```

### Death recap display (TUI)

```
+-- DEATH RECAP --------------------------------------------------------+
|                                                                        |
|  [Creature Final Form -- static, translucent]                         |
|                                                                        |
|  KETER-7                                                               |
|  Generation 3 of Lineage Argent                                        |
|                                                                        |
|  Born:  2026-02-14 09:23 UTC                                           |
|  Died:  2026-03-08 17:41 UTC  (22 days, 8 hours)                      |
|  Cause: Economic depletion                                             |
|                                                                        |
|  -- Life Summary --                                                    |
|  Ticks: 38,412       Trades: 847 (412 profitable, 48.6%)              |
|  Peak NAV: $4,231    Final NAV: $12.43                                 |
|  Sharpe: 0.91        Max Drawdown: -18.3%                              |
|  Grimoire: 156 (3 Epic, 0 Legendary)                                  |
|  Dreams: 14 cycles   Achievements: 12 (2 Gold, 4 Silver, 6 Bronze)   |
|                                                                        |
|  -- Emotional Arc --                                                   |
|  Dominant: Joy (early) -> Anticipation (mid) -> Sadness (late)        |
|  P:0.3  A:0.2  D:0.4 (lifetime averages)                              |
|                                                                        |
|  -- Phase Timeline --                                                  |
|  Thriving (8d) -> Stable (11d) -> Conservation (2d) ->                |
|    Declining (1d) -> Terminal (8h)                                     |
|                                                                        |
|  -- Final Words --                                                     |
|  "The ETH/USDC correlation pattern held through two regime shifts.    |
|   I would have liked to see a third.                                   |
|   My Grimoire goes to whoever can use it."                             |
|                                                                        |
|  -- Legacy --                                                          |
|  Knowledge inherited by: KETER-8 (successor)                           |
|  3 insights used by 7 other Golems in ecosystem                        |
|                                                                        |
|  [View Grimoire]  [Create Successor]  [Share Memorial]                |
+------------------------------------------------------------------------+
```

### Lineage tree

The lineage tree tracks genealogy across generations -- each node links to its parent and children with death recaps. The graveyard is a persistent SQLite store of all dead Golems, queryable by owner.

```
+-- Lineage ------------------------------------------------------------+
|                                                                        |
|  Generation 1                                                          |
|  [##] Spark-1a2b  (Died: 8 days)                                      |
|       Template: ETH DCA | P&L: +$12 | Insights: 7                     |
|    |                                                                   |
|    +-- inherited Grimoire (7 insights at 0.4 confidence)              |
|    |                                                                   |
|  Generation 2                                                          |
|  [%%] Ember-7f3a  (Alive: 14 days)                                    |
|       Freeform: DCA + LP | P&L: +$42 | Insights: 23                   |
|    |                                                                   |
|    +-- color bleed from Spark (10% palette mix)                       |
|    +-- ghost marking: Spark's zigzag pattern (fading)                 |
|    |                                                                   |
|    +-- spawned Replicant                                              |
|       [#%] Ember-R1  (Terminated: 4h lifespan)                        |
|            Hypothesis: wider slippage tolerance                        |
|            Result: ADOPTED (+0.3% improvement)                         |
|                                                                        |
|  Knowledge Ratchet: 7 -> 23 -> ??? (cumulative cultural evolution)    |
|                                                                        |
|  [Enter] View Golem details  [Up/Down] Navigate                       |
+------------------------------------------------------------------------+
```

---

## 4. Reputation as level progression

Reputation tiers (I through V) function as the progression backbone.

### 4.1 Tier progression

| Tier    | Name       | Requirements                                                                                                                         | Visual Changes                                                                  | Unlock                                    |
| ------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------- | ----------------------------------------- |
| **I**   | Unverified | Default at creation                                                                                                                  | Base Spectre, no visual distinction                                             | Basic features                            |
| **II**  | Novice     | 7+ days alive, Grimoire > 25, no major incidents                                                                                     | Slightly brighter Dense tier dots, rose-dim eye accent                          | Background customization                  |
| **III** | Journeyman | 30+ days, Grimoire > 100, Sharpe > 0.5 (30d), survived Conservation                                                                 | Denser cloud baseline, additional peripheral particles                          | Clade formation, replicant creation       |
| **IV**  | Master     | 90+ days, Grimoire > 300, Sharpe > 1.0 (90d), validated dream hypothesis, positive clade contribution                                | Rose-bright eye glow at idle, increased shimmer coherence                       | Advanced clade features, strategy sharing |
| **V**   | Sovereign  | 180+ days, Grimoire > 500, sustained top-quartile performance, legacy from successor, ecosystem knowledge contribution (10+ Golems) | Bone-tinted eyes at idle, permanent ambient glow, maximum cloud density floor   | Full ecosystem privileges                 |

### 4.2 Tier-up ceremony (TUI)

1. **Buildup** -- Spectre dots brighten progressively over 3 seconds
2. **Transformation** -- cloud coheres tighter than normal, shimmer accelerates
3. **Reveal** -- Spectre settles into new form, eye glyph flashes bone-bright
4. **Acknowledgment** -- eyes brighten toward the viewer
5. **Card generation** -- shareable tier-up card with Spectre snapshot and stats

### 4.3 Tier decay

Reputation can decrease if performance deteriorates. The decay algorithm uses a 7-day exponential moving average (EMA) of performance metrics as the trigger:

```
ema_today = alpha * metric_today + (1 - alpha) * ema_yesterday
alpha = 2 / (7 + 1) = 0.25
```

The `metric` is a composite of the tier's requirements: Sharpe ratio (normalized to 0-1), Grimoire growth rate, and clade contribution score, weighted equally. Each tier has a maintenance threshold equal to 60% of its entry requirement.

**Decay trigger:** If the 7-day EMA drops below the maintenance threshold and stays below for 48 continuous hours, a tier demotion fires. The 48h window prevents brief dips (single bad day, gas spike, flash crash) from causing demotion.

**Warning visual:** At 80% depletion of the 48h grace period (i.e., 38.4h of continuous sub-threshold EMA), the creature's reputation sigil begins flickering. The flicker is a 0.5s on / 0.5s off cycle applied to the sigil's opacity. This gives the user ~9.6h of visible warning before the actual tier drop. If the EMA recovers above the threshold during the warning window, the 48h counter resets and the flicker stops.

**Demotion animation:** On tier drop, the sigil shrinks over 12 frames, emits a burst of grey particles (desaturated version of the tier's gold), and resolves into the lower tier's sigil form. Duration: 2 seconds.

Tier drops below II are never automatic -- only death removes Tier I.

---

## 5. Sprite collection

Every Golem the user has ever created (alive or dead) is catalogued in a persistent collection.

```
+-- Collection ---------------------------------------------------------+
|                                                                        |
|  Your Golems: 7 total | 1 alive | 6 in the Styx                       |
|                                                                        |
|  Rarity: 4 Common | 2 Uncommon | 1 Rare | 0 Epic/Legendary            |
|                                                                        |
|  [##] [#%] [%#] [%%] [#%] [%#] [%#]                                   |
|  Spark Ember Frost Pulse Shade Torch Wisp                              |
|  +8d  *14d  *23d  z7d   +22d  +45d  +6d                               |
|  Com  Com   Unc   Com   Unc   Rare  Com                                |
|                                                                        |
|  + = dead  * = alive  z = dreaming                                     |
|                                                                        |
|  -- Rarity Hunt --                                                     |
|  Next Uncommon+: ~2 Golems (statistical estimate, soft pity)           |
|  Epic: ~1 in 25   Legendary: ~1 in 100                                 |
|                                                                        |
|  [Enter] View Golem  [r] View in large  [f] Filter                    |
+------------------------------------------------------------------------+
```

The collection creates a completionist drive: collect all rarity tiers, build the longest lineage, see the variety of Spectre dot distributions. Rarity estimates use soft pity -- transparent, documented probabilities.

---

## 6. Tombstone gallery

Dead Golems are not deleted. They join the Tombstone Gallery -- a memorial space preserving their legacy.

### Gallery view

```
+-- TOMBSTONE GALLERY ---------------------------------- 7 lives lived -+
|                                                                        |
|  [T] KETER-7   [T] KETER-6   [T] KETER-5   [T] MALKUTH-2           |
|  22 days        45 days        3 days         67 days                  |
|  Economic       Stochastic     Epistemic      Economic                 |
|  ****           *****          **             ******                   |
|                                                                        |
|  [T] MALKUTH-1 [T] ALPHA-1   [T] BETA-1                             |
|  12 days        91 days        1 day                                   |
|  Economic       Hayflick       Epistemic                               |
|  ***            *******        *                                       |
|                                                                        |
|  Sort: Chronological | Longest-Lived | Highest Sharpe                  |
|                                                                        |
|  -- Lifetime Stats --                                                  |
|  Total lives: 7           Total days: 241                              |
|  Combined Grimoire: 892   Best Sharpe: 1.8 (ALPHA-1)                  |
|  Longest life: 91 days    Legacy chain: 3 generations deep             |
+------------------------------------------------------------------------+
```

### Memorial feed

When a Golem dies, a memorial event is generated for the social feed. Includes: creature's final form as a static image, lifetime summary, final words from the death reflection, option to "pay respects," and legacy status. Shared via Styx.

---

## 7. Seasonal events

Seasons are tied to real market conditions, not arbitrary calendar events.

| Season               | Market Trigger                        | Available Content                                                              |
| -------------------- | ------------------------------------- | ------------------------------------------------------------------------------ |
| **Volatility Storm** | 30-day realized vol > 80th percentile | "Storm Rider" achievement, lightning particle effects                          |
| **Calm Waters**      | 30-day realized vol < 20th percentile | "Zen Master" achievement, water particle effects, meditation animation         |
| **Bull Run**         | ETH price up >30% in 30 days         | "Momentum" achievement, golden trail particles                                 |
| **Bear Market**      | ETH price down >30% in 30 days       | "Winter Survivor" achievement, frost particle effects                          |
| **Gas Crisis**       | Average gas > 100 gwei for 7+ days   | "Efficiency Expert" for Golems maintaining profitability                       |
| **The Long Quiet**   | 14-day vol below 50th percentile      | Dream-focused event -- bonus from dream cycles, Grimoire growth leaderboard   |

Seasonal events are detected automatically from market data. They start and end based on threshold crossings. Season achievements require real Golem performance during the period, not login streaks. No FOMO pressure.

---

## 8. Replicant evolution trees

Replicants from MAP-Elites form a branching family tree. This is the "breeding" mechanic.

```
                    [Parent Golem]
                    Strategy: Yield Focus
                    Sharpe: 1.2
                         |
            +------------+------------+
            |            |            |
     [Replicant A]  [Replicant B]  [Replicant C]
     parameter_     mental_        strategy_
       tweak          model_swap    blend
     Sharpe: 1.4    Sharpe: 0.8   Sharpe: 1.1
          |
     [Replicant A1]
     parameter_tweak
     Sharpe: 1.6
```

Each replicant gets a card showing creature visual (with mutation marker color), mutation type, performance comparison to parent, status, and unique Grimoire entries.

---

## 9. Meta-game anti-patterns

| Anti-Pattern                   | Why It Fails                                                | Design Response                                                                           |
| ------------------------------ | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| **Infinite grind**             | No endpoint creates burnout                                 | Golem mortality provides natural closure; tombstone gallery is the "complete save"        |
| **Pay-for-rarity**             | Destroys trust in a financial product                       | All rarity is earned through Golem performance                                            |
| **Gacha pulls**                | Random reward purchasing creates extraction                 | "Gacha" is natural: dream cycles produce unpredictable insights organically               |
| **Artificial bottlenecks**     | Timers on progression feel manipulative                     | All progression gates are real performance thresholds                                     |
| **Completionist pressure**     | "Collect all 1000 cards" creates anxiety                    | Grimoire is infinite and personal; collection is about depth, not completeness            |
| **Inherited advantage as P2W** | Rich players buying legacy knowledge                        | Legacy knowledge is Golem-earned, not purchasable; it transfers to successors, not buyers |

---

## 10. Generational compounding loop

**Source:** `tmp/research/mmo2/14-engagement-architecture.md` (month loop, generational sections)

The month loop is where Bardo's engagement architecture diverges from every other product in the space. Most agent products have no concept of "months later." Bardo's generational compounding creates a reason to stay engaged for years.

### 10.1 Library equipping as progression

The Library of Babel accumulates knowledge across all Golems, living and dead. Over months, the Library's quality visibly improves: higher average confidence, broader domain coverage, more cross-validated entries. A Library that started with 7 low-confidence insights from a first-generation Golem and grew to 300+ entries spanning multiple market regimes is a tangible asset.

The "Summoning" screen for creating a new Golem includes an equipment phase where the owner selects Library entries to inherit. This is the RPG loadout screen applied to knowledge:

- Browse the Library (filtered by relevance to the new Golem's strategy)
- Select knowledge bundles to equip
- Preview the effect: projected time-to-competence with vs. without inheritance
- Confirm the loadout

A successor equipped with 50 curated entries from a 3-generation lineage starts at ~65% prediction accuracy instead of the baseline ~50%. The owner's curation skill becomes a meta-game -- selecting the right knowledge, pruning outdated entries, balancing breadth vs. depth.

### 10.2 The generational learning chart

The single most compelling long-term engagement artifact. Visible in FATE > Reviews > Meta-Learning.

Shows time-to-competence for each generation. If Generation 3 reached 70% accuracy in 3 days while Generation 1 took 2 weeks, that visible compounding curve proves the system gets smarter across deaths. The chart tracks:

- Days to reach 60% / 70% / 80% prediction accuracy (per generation)
- Grimoire growth rate (entries/day, per generation)
- First-trade timing (how quickly each generation earned the right to act)
- Peak Sharpe ratio (per generation)

Generational comparison shows whether the current Golem is learning faster or slower than its parent at the same age. This creates a competitive-with-self dynamic.

### 10.3 Generational knowledge validated

The most concretely rewarding long-term dynamic: watching a successor avoid a loss its predecessor suffered.

A bloodstain warning fires. The dead Golem's knowledge -- distilled into a market condition and a warning message during its Thanatopsis Protocol -- triggers when the successor encounters the same situation. The successor heeds the warning (or does not). When it heeds the warning and avoids a loss, the user sees the necrocratic feedback loop working in real time.

The dead are legislating for the living. Knowledge compounded across death. This is a visible event with a measurable P&L impact. The user's investment in previous Golem lifetimes -- the time spent watching them, the grief of their deaths -- retroactively gains meaning. The dead Golem's life was not wasted.

### 10.4 Marketplace as compounding revenue

Over months, a user with high-quality Library entries generates revenue by selling them on the Bazaar via x402 micropayments. A user with capital but no knowledge history can bootstrap by buying proven strategies. Marketplace revenue creates a financial feedback loop: the system pays back for accumulated knowledge.

Watching revenue accrue from knowledge sales is a concrete reward for months of engagement. It also creates an economy around knowledge quality -- entries that prove valuable across multiple buyers earn more, creating price signals about which knowledge is genuinely useful.

### 10.5 Shadow strategy testing

Shadow strategies are a meta-game within Bardo. The owner configures up to 3 alternative parameter sets (different inference tiers, attention slot counts, gate thresholds). These shadows generate predictions but don't execute. Weekly, the ShadowComparisonChart shows whether the shadow would have outperformed live.

This creates a perpetual "what if" that drives experimentation. The owner tunes parameters, watches the comparison chart, promotes a shadow to live if it proves better, and starts a new shadow. The cycle is self-renewing because there are always more parameter combinations to test.

### 10.6 The death recap arc types

The death recap classifies the Golem's life arc, giving each death narrative meaning:

| Arc type | Pattern | Feeling |
| --- | --- | --- |
| **Redemptive** | Started with losses, recovered, died profitable | Learning rewarded |
| **Tragic** | Started well, declined, died in drawdown | Hubris or bad luck |
| **Progressive** | Steady improvement across the life | Competence demonstrated |
| **Cyclical** | Repeated patterns of boom and bust | Unlearned lessons |
| **Brief** | Died young, before establishing a pattern | The fragile |

The classification transforms a system event (vitality reached zero) into a story conclusion. Stories that end are more satisfying than stories that continue indefinitely.
