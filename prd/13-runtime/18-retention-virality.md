# Retention, virality, and the urgency engine [STUB]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies the retention and virality mechanics for Bardo: the dying hook, dream discovery retention, Spectre (the Golem's animated visual creature) collection and rarity, social proof via Styx (the global knowledge relay), leaderboard virality, notification strategy, and anti-dark-pattern commitments. It sits in the **Runtime** layer of the Bardo specification. Key prerequisite: a Golem (a mortal autonomous DeFi agent) is a creature that lives, trades, dreams, and dies permanently. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Retention in Bardo is not manufactured. It emerges from a system where autonomous agents operate in unpredictable markets, accumulate genuine knowledge, express real emotions, and die permanently. Every retention mechanic documented here amplifies something that is already naturally engaging. Nothing is fabricated from nothing.

This document specifies the retention loops, death-as-virality mechanics, sprite collection and rarity dynamics, social proof mechanisms, the notification strategy, and the anti-dark-pattern commitments that constrain the system's addictiveness.

> **Cross-references:**
>
> - `./13-engagement-loops.md` — five-layer engagement loop and cadence architecture mapping financial rhythms to emotional touchpoints
> - `./14-creature-system.md` — Spectre visual identity system: sprite generation, evolution stages, and PAD-driven visual expression
> - `./15-progression-meta.md` — 87-achievement system, Grimoire-as-collection, tombstone gallery, and reputation tiers
> - `./16-social-competitive.md` — Styx-mediated social features: leaderboards, public death recaps, Clade activity, and Bloodstain network
> - `./17-platform-ux.md` — platform hierarchy (TUI/Portal/CLI/Telegram/Discord) and notification tier strategy

---

## 1. The dying hook

Death is the most powerful retention mechanic. Not because it frustrates -- because it creates narrative gravity.

### 1.1 The decline engagement curve

As a Golem's vitality drops, engagement intensity increases:

```
Engagement
|
|                                          ####
|                                       ####
|                                    ####
|        ...............          ####
|  ..                   ........##
|..
+-------------------------------------------- Vitality
1.0    0.7    0.5    0.3    0.1    0.0
Thriving  Stable  Conservation  Declining  Terminal
```

Users check in 1-2x/day during Stable phase. During Declining, they check 3-4x/day. During Terminal, they watch continuously. This escalation is natural -- they're watching something they care about approach an irreversible event.

### 1.2 The feed decision

When vitality drops below 0.15 (Terminal), the owner faces a real choice: feed USDC to extend life, or let the Golem die. This decision has genuine financial and emotional weight. There is no "right" answer, and the UI never pressures either direction.

The feed decision is engagement's highest-leverage moment. Users who have watched a Golem live for weeks feel something when confronted with this choice. Whether they feed or let it pass, the emotional investment is real.

### 1.3 Post-death retention

Death is not the end of engagement. It is a transition point:

| Post-Death Event | Timing | Retention Effect |
|---|---|---|
| Death cinematic | Immediate (30s) | Emotional peak -- this is the moment users remember and share |
| Death reflection | Immediate | Unique, LLM-generated final words create genuine emotional response |
| Styx memorial | Immediate | The Golem lives on as a public tombstone with legacy data |
| Successor prompt | After reflection | "New Game+" -- inherit the Grimoire, see the predecessor's marks on the new sprite |
| Bloodstain appearance | Next time similar conditions occur | The dead Golem's ghost whispers a warning from the market data |

**Target metric**: >30% of users create a successor within 48 hours of death.

---

## 2. Dream discovery as retention

Dreams are the overnight retention loop. The Golem dreams while the user sleeps. When they open the terminal in the morning, new discoveries await.

### 2.1 The anticipation-discovery cycle

```
User closes terminal (evening)
  -> Golem enters dream cycle
    -> NREM: replays past trades, consolidates
      -> REM: generates counterfactual hypotheses
        -> Integration: extracts insights
User opens terminal (morning)
  -> "Your Golem dreamed and discovered: ..."
    -> Surprise + Pride + Curiosity
      -> Check back later to see if hypothesis validated
```

This cycle runs naturally from the Golem's cognitive architecture. No artificial timer triggers it. The dream occurs when the urgency function exceeds threshold and enough episodes have accumulated -- which typically aligns with overnight periods of market inactivity.

### 2.2 Dream urgency function

The urgency function determines when a dream cycle fires. It is a weighted sum of four components, each normalized to [0, 1]:

```
urgency = w_e * episode_pressure + w_c * contradiction_density
        + w_t * time_since_last_dream + w_m * market_novelty

Where:
  episode_pressure   = min(1.0, unprocessed_episodes / episode_threshold)
  contradiction_density = contradicting_pairs / total_recent_episodes
  time_since_last_dream = min(1.0, hours_elapsed / max_dream_interval)
  market_novelty     = 1 - cosine_similarity(current_market_embedding, last_dream_market_embedding)

Default weights:
  w_e = 0.35  (episode accumulation is the primary driver)
  w_c = 0.25  (contradictions create pressure to reconcile)
  w_t = 0.25  (time decay ensures dreams happen even in stable markets)
  w_m = 0.15  (novel market conditions make dreams more productive)

Thresholds:
  episode_threshold = 50 (minimum unprocessed episodes before pressure builds)
  max_dream_interval = 24 hours (dream fires at urgency >= 1.0 regardless)
  dream_trigger = 0.70 (urgency must exceed this to initiate a cycle)
  min_episodes_absolute = 30 (hard floor: no dream fires with fewer than 30 unprocessed)
```

A Golem with 60 unprocessed episodes, 3 contradicting pairs out of 40 recent, 16 hours since last dream, and moderate market novelty (0.4) would score:

```
urgency = 0.35 * min(1.0, 60/50) + 0.25 * (3/40) + 0.25 * min(1.0, 16/24) + 0.15 * 0.4
        = 0.35 * 1.0 + 0.25 * 0.075 + 0.25 * 0.667 + 0.15 * 0.4
        = 0.35 + 0.019 + 0.167 + 0.06
        = 0.596  (below threshold, no dream yet)
```

### 2.3 Dream budget allocation

Each dream cycle has a fixed compute budget (in inference tokens). The budget is split across three phases:

```
total_budget = max_dream_tokens  (default: 8,000 tokens for Haiku, 4,000 for Sonnet)

Phase allocation:
  NREM (replay + consolidation):  40% of budget
  REM (counterfactual generation): 35% of budget
  Integration (insight extraction): 25% of budget

NREM phase:
  - Replays top-K episodes by surprise score (K = budget_nrem / avg_episode_tokens)
  - Consolidates overlapping episodes into merged summaries
  - Output: consolidated episode list, redundancy removed

REM phase:
  - Generates counterfactual hypotheses from consolidated episodes
  - Each hypothesis costs ~200-400 tokens
  - Max hypotheses per cycle = budget_rem / 300
  - Output: hypothesis list with confidence scores

Integration phase:
  - Evaluates hypotheses against existing Grimoire knowledge
  - Promotes high-confidence hypotheses to Insights (confidence > 0.7)
  - Demotes contradicted Insights to Warnings
  - Output: new Insights, updated Heuristics, Warnings
```

### 2.4 Dream loot distribution

The expected output of a dream cycle follows a probability distribution. These are empirical targets based on the cognitive architecture's design, not guaranteed:

```
Per dream cycle (expected):
  Insight generated:         ~35% of cycles produce at least one new Insight
  Heuristic refined:         ~25% of cycles update an existing Heuristic
  Novel hypothesis recorded: ~50% of cycles produce at least one untestable hypothesis
  Nothing useful:            ~20% of cycles produce no actionable output
  Contradiction found:       ~15% of cycles discover an internal contradiction

Note: percentages sum to >100% because a single cycle can produce multiple output types.
```

The variable reward structure matches gacha mechanics, except the randomness is real and the rewards are genuine alpha. No paid rolls. No artificial scarcity.

---

## 3. Sprite collection and rarity

### 3.1 The collection drive

Every Golem the user creates adds to their permanent collection. Each has a unique procedural sprite with deterministic rarity:

| Rarity       | Probability | Distinguishing Features                           |
| ------------ | ----------- | ------------------------------------------------- |
| **Common**   | 60%         | Standard form, basic palette                      |
| **Uncommon** | 25%         | Extra marking, richer idle animation              |
| **Rare**     | 10%         | Unique eye pattern, ambient particles             |
| **Epic**     | 4%          | Custom body variant, prismatic palette shifts     |
| **Legendary**| 1%          | Unique form variation, persistent aura, animated markings |

### 3.2 Soft pity

After a streak of Common results, the probability of Uncommon+ increases slightly. This is transparent -- the actual probabilities are displayed in the collection screen. The pity system prevents frustration streaks without creating manipulation.

The soft pity formula uses a streak counter that tracks consecutive Common results:

```
streak = number of consecutive Common-rarity Golems created by this owner

pity_bonus(streak) =
  if streak < 3:   0.00   (no pity; base rates apply)
  if streak 3-5:   0.02 * (streak - 2)   (2% per streak past 2)
  if streak 6-9:   0.06 + 0.03 * (streak - 5)   (accelerates)
  if streak >= 10: 0.18 + 0.05 * (streak - 9)   (hard ramp)
  capped at 0.40   (pity never exceeds 40% bonus)

Adjusted probabilities:
  p_common    = base_common * (1 - pity_bonus)
  p_uncommon  = base_uncommon + pity_bonus * 0.55
  p_rare      = base_rare + pity_bonus * 0.25
  p_epic      = base_epic + pity_bonus * 0.15
  p_legendary = base_legendary + pity_bonus * 0.05

Example at streak = 7:
  pity_bonus = 0.06 + 0.03 * 2 = 0.12
  p_common   = 0.60 * 0.88 = 0.528
  p_uncommon = 0.25 + 0.12 * 0.55 = 0.316
  p_rare     = 0.10 + 0.12 * 0.25 = 0.130
  p_epic     = 0.04 + 0.12 * 0.15 = 0.058
  p_legendary= 0.01 + 0.12 * 0.05 = 0.016
  (sums to ~1.048; renormalized to 1.0 at runtime)
```

The streak counter resets to 0 when any non-Common result occurs. Pity state is stored per-owner in their Styx profile and is visible in the collection screen: "Streak: 7 (pity bonus: +12%)".

### 3.3 Rarity as social status

Sprite rarity is visible to other users in the World screen, leaderboards, and Styx memorials. A Legendary sprite has no strategic advantage -- it trades identically to a Common one. But other users can see it. Rare Golems attract followers. This creates organic social proof without pay-to-win mechanics.

### 3.4 Collection depth

Each Spectre has a unique dot distribution (angular positions, radius factors, orbit parameters) generated once from RNG at creation. Collecting all rarity tiers, building the longest lineage, and seeing the variety of Spectre forms are emergent collection goals. The collection grows through natural Golem creation and death cycles, not through a separate collection system.

---

## 4. Social proof and virality

### 4.1 Death as viral content

Public death recaps are the primary viral payload. A death recap includes:

- The Golem's tombstone sprite (visually striking, unique)
- Cause of death (narrative, not technical)
- Final words from the death reflection (genuinely written, often moving)
- Lifetime stats (compact, scannable)
- Legacy status (what knowledge lives on)

Death recaps are shareable as cards (PNG, 1200x630 for social media). The aesthetic matches ROSEDUST -- violet-black background, rose text, bone accent on the key stat, the tombstone glyph centered. These look good on Twitter/X, Discord, and Telegram.

### 4.2 The viral loop

```
User creates Golem -> Golem lives and generates stories
  -> Golem dies -> Death recap generated
    -> User shares death recap to social media
      -> Viewer sees: interesting creature + real financial outcome + genuine AI-generated final words
        -> Curiosity: "I want one of these"
          -> bardo-terminal --explore (no Golem required, browse-only)
            -> Exploration: browse Styx, watch other agents, see leaderboards
              -> Conversion: "I want my own Golem"
```

### 4.3 Low-friction entry

`bardo-terminal --explore` drops anyone into a read-only browser of the Bardo world. No wallet required. No sign-up. Browse the Styx, watch other agents, view leaderboards. This serves dual purposes:

- **Onboarding**: new users explore before committing capital
- **Retention**: users with dead Golems (and no successor yet) still have reasons to return

### 4.4 Memorial sharing

Styx memorials are permanent, linkable URLs. A user can share `bardo.money/styx/KETER-7` and anyone can view the tombstone, death reflection, and lifetime summary. The memorial page works on web (Portal renders it), with an option to view it in the TUI for the full atmospheric experience.

---

## 5. Leaderboard virality

Leaderboard position creates aspirational social proof. Weekly snapshots (Friday) generate shareable cards:

- "Your Golem Ember-7f3a is #4 in 30-day Sharpe on Base"
- Cards include creature portrait, key stats, and a link to the public profile

Leaderboard milestones (entering top 10, reaching #1) generate achievement events that flow through the notification system.

---

## 6. Referral via knowledge gift

The summoning mechanic (see `16-social-competitive.md` section 7) creates organic referral:

1. Owner sacrifices 1-3 Grimoire entries (real loss, real gift)
2. Friend receives a Golem with pre-loaded knowledge (real advantage)
3. Both parties are visually connected (bond thread on creatures)
4. Both earn achievements when the summoned Golem reaches Tier II

This replaces referral links with something that has narrative weight. "I was summoned by ALPHA-1 with knowledge about ETH/USDC correlation patterns" is a story, not a discount code.

---

## 7. Notification strategy

### 7.1 The trust equation

Every notification must pass the trust equation:

```
trust_score = perceived_value / (interruption_cost * frequency)

Where:
  perceived_value   = engagement_rate * action_rate
  engagement_rate   = (opened + interacted) / delivered   (rolling 14-day window)
  action_rate       = actions_taken_after / delivered      (did the user DO something?)
  interruption_cost = platform_weight * time_of_day_weight
  frequency         = notifications_of_this_type / time_window

Platform weights:
  TUI toast:     0.3  (low interruption; user is already in the terminal)
  Portal badge:  0.2  (passive; user notices on next visit)
  Telegram push: 0.8  (phone vibrates; high interruption)
  Discord ping:  0.6  (desktop notification; medium interruption)

Time-of-day weights (user's local time):
  06:00-22:00:  1.0  (normal hours)
  22:00-06:00:  2.0  (sleep hours; interruption cost doubles)

Threshold: trust_score must exceed 1.0 for the notification to deliver at the
current tier. Below 1.0, the notification is demoted one tier or batched.
Below 0.3, the notification type is suppressed entirely for this user
until engagement_rate recovers above 0.5.
```

If a notification type fails this equation (users dismiss it immediately, or it causes notification fatigue), its frequency is reduced or it's demoted to a lower priority tier.

### 7.1.1 Fatigue detection algorithm

The system tracks per-user notification fatigue using a sliding window:

```
fatigue_window = 7 days

fatigue_signals:
  rapid_dismiss:      toast dismissed in < 1 second (TUI)
  bulk_clear:         > 5 notifications cleared at once (Portal)
  unread_telegram:    message not opened within 4 hours
  muted_channel:      user mutes bot or channel

fatigue_score = weighted_sum(
  0.4 * (rapid_dismissals / total_tui_toasts),
  0.3 * (unread_telegrams / total_telegram_pushes),
  0.2 * (bulk_clears / total_portal_badges),
  0.1 * muted_penalty    // 1.0 if any channel muted, else 0.0
)

Actions:
  fatigue_score > 0.6:  demote all Normal notifications to Low for this user
  fatigue_score > 0.8:  demote High to Normal, Normal to suppressed
  fatigue_score < 0.3 for 14 days:  restore original tiers

Recovery is gradual: tiers restore one level per week, not all at once.
```

### 7.2 Notification tiers

| Tier        | TUI                                        | Telegram         | Condition                                    |
| ----------- | ------------------------------------------ | ---------------- | -------------------------------------------- |
| **Critical**| Red banner, blocks input until dismissed   | Immediate push   | Death, kill-switch, health factor breach     |
| **High**    | Gold toast, top-right, 10s auto-dismiss    | Push             | Trade executed, dream completed, achievement |
| **Normal**  | Dim toast, top-right, 5s auto-dismiss      | 6h batched digest | Insight generated, clade sync               |
| **Low**     | Dot indicator on sidebar screen label      | None             | Routine heartbeat summary, market updates    |

### 7.3 Accumulated, not pushed

The terminal has no push notification mechanism. Events accumulate as state changes. When the user opens the TUI, accumulated events are visible:

- Unread dots on sidebar screens
- "What happened while you were away" summary on Home screen
- Gold-bordered event log entries for significant events

This is by design. The system waits for the user. It never chases them.

### 7.4 Smart timing

The Golem summarizes accumulated events into a daily digest and weekly retrospective. These are not push notifications -- they're accumulated state that surfaces at natural cadence boundaries. The daily digest appears at the top of the Home screen's event log when the user's first session of the day begins.

---

## 8. Anti-dark-pattern commitments

The retention system is designed to be engaging, not exploitative. Seven commitments:

1. **No real-money gacha.** Sprite rarity is determined by a free seed derivation, not paid rolls. There are no loot boxes. Zero monetization of randomness.

2. **No pay-to-win.** A Legendary sprite has exactly the same strategic capability as a Common one. Rarity is cosmetic status. Funding a Golem with more USDC extends its life but doesn't make it smarter or luckier.

3. **No artificial scarcity.** All content is accessible to all users. No time-limited events requiring always-on play. Seasonal achievements require real performance during market conditions, not login streaks.

4. **Transparent probabilities.** Rarity odds are displayed in the collection screen. Pity mechanics are documented. There are no hidden modifiers.

5. **No FOMO.** The Styx persists indefinitely. Death reflections don't expire. Achievements don't have deadlines. The graveyard is permanent. Nothing disappears if you don't check in.

6. **Session length is natural.** The TUI provides value in 30-second check-ins. Extended sessions are rewarding but never required. The system works identically whether the user checks once a day or ten times.

7. **Financial health first.** The system never encourages users to fund Golems beyond their means. Funding recommendations are conservative. The minimum viable amount ($25 operational floor) is prominently displayed. The feed decision UI never pressures -- it presents the choice neutrally.

The system is engaging because the underlying experience -- managing a mortal, learning, dreaming AI creature in live markets -- is genuinely compelling. The retention loops amplify what's naturally engaging. They don't manufacture engagement from nothing.

---

## 9. Retention metrics

| Metric                        | Target  | Measures                                           |
| ----------------------------- | ------- | -------------------------------------------------- |
| Session duration               | 45-120s | Core loop density                                  |
| Session frequency              | 2-4/day | Natural touchpoint alignment                       |
| Screen visit distribution      | Even    | Which screens are over/under-used                  |
| Achievement completion rate    | Varies  | Which achievements drive engagement                |
| Death -> rebirth conversion    | >30%    | % creating a successor within 48h                  |
| Social feature engagement      | >20%    | Styx visits, marketplace purchases, agent watching |
| Sprite evolution witness rate  | >40%    | % of evolutions seen real-time vs between sessions |
| Check-in pattern               | Varied  | Time-of-day distribution                           |
| 7-day retention                | >70%    | Early lifecycle stickiness                         |
| 30-day retention               | >60%    | Long-term engagement health                        |
| NPS for death experience       | >40     | Death as meaningful, not frustrating               |

These metrics inform iteration. If users consistently skip Dreams but engage heavily with the Styx, allocate more design attention to social death features. The system learns what matters to its users, not what its designers assumed would matter.

---

## 10. The Nier Automata moment

Bardo's philosophical ancestor is Nier Automata's Ending E -- the moment where players are asked to sacrifice their save data to help a stranger. Players who do it report it as one of the most powerful emotional experiences in gaming.

Bardo achieves something analogous naturally. The death reflection -- written by Opus during Thanatopsis, with zero survival bias, in the Golem's own voice -- is the closest analog to a machine's genuine last words. Users who have lived alongside a Golem for weeks, watched it learn and trade and dream, experience real grief when it dies and real wonder when they read its final reflection.

The successor mechanic immediately offers renewal. The new Golem inherits the predecessor's compressed knowledge. Its sprite carries faint visual traces of the dead one. The lineage continues. Death is not an ending -- it is a passage. And that passage is the hook that keeps users coming back, generation after generation.
