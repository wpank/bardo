# 08 — Consolidated v4 Screen Specifications

**Status:** v4.0.0 Specification

**Sources:**
- `engagement-prd/bardo-v4-01-hearth.md`
- `engagement-prd/bardo-v4-02-mind.md`
- `engagement-prd/bardo-v4-03-vault-world-fate-command.md`

**Cross-references:** [00-screen-catalog.md](00-screen-catalog.md) (29-screen summary across 6 windows), [00-design-system.md](../rendering/00-design-system.md) (ROSEDUST visual identity spec for palette and atmospheric rendering), [02-widget-catalog.md](02-widget-catalog.md) (33-widget TUI component library)

> **Reader orientation:** This document provides the detailed v4 screen specifications for every screen in the Bardo terminal. It belongs to the **interfaces/screens layer** and complements the screen catalog with exact pane layouts, widget assignments, drill-down paths, and modal structures. Key concepts: Golem (a mortal autonomous DeFi agent), Heartbeat (the Golem's periodic execution tick with T0/T1/T2/T3 tiers of increasing computational intensity), Vitality (remaining economic lifespan), Grimoire (persistent knowledge store), and the Daimon (emotional appraisal subsystem producing PAD vectors). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 1. Hearth — "The Presence" (Window 1 of 6)

"What is happening right now?"

**Atmosphere:** Warm. Amber undertone. Fragments active. Noise floor at 0.4%. This is the living room — the screen you leave running on a second monitor, the first thing you see when you open the terminal. Designed for peripheral monitoring: the visual RHYTHM of the Hearth communicates system state without requiring you to read numbers.

---

### Tab 1: Overview (Default)

The heartbeat of the system. Everything at a glance.

**Heartbeat Log** (center, 55% width) — PERSISTENT

The core of the Hearth. A phosphor-decay log of every tick.
- T0 entries (routine monitoring, no LLM): `text_ghost`. Nearly invisible. The Golem observed and found nothing.
- T1 entries (LLM-assisted analysis): `rose`. Something caught attention. A cheap model evaluated.
- T2 entries (full inference with tool execution): `rose_bright`. A blaze. Full deliberation.
- T3 entries: `bone`. Maximum brightness. Opus-level inference. Something serious.
- Each entry fades through the phosphor chain as it ages (Widget Catalog §7).
- The RHYTHM of T0-T0-T0-T0-T2! is variable-ratio reinforcement. Most ticks are quiet. Then one blazes.

*When locked:* Arrow keys scroll through entries. Enter on an entry opens **Tick Detail Modal**:
- Tab 1: **Pipeline** — which probes fired, which modules consulted, MAGI vote, the decision
- Tab 2: **Context** — the full context window assembled (every item, its source, its weight)
- Tab 3: **Cost** — inference cost, gas cost, token breakdown, which model was used
- Tab 4: **Outcome** — what happened after the action (trade result, verification, PAD shift)
- Enter on any context item → **Context Item Modal** (source, embedding distance, why it was selected)
- Enter on the MAGI vote → **Consensus Modal** (each MAGI member's reasoning, historical agreement rate)

**Vitality Number** (right, top) — PERSISTENT

THE bone number. `0.711`. Lerps between values. Shadow pulses with heartbeat. At low vitality, digits decay to `·`. See Widget Catalog §11.

*When locked:* No scrolling (single element). Enter opens **Vitality Detail Modal**:
- Tab 1: **Breakdown** — composite formula, weight of each clock, how they multiply
- Tab 2: **History** — vitality waveform over the Golem's lifetime
- Tab 3: **Projection** — Monte Carlo survival curves, confidence intervals

**Mortality Gauges** (right, middle) — PERSISTENT

Three thick gauges: ECONOMIC (amber), EPISTEMIC (blue), STOCHASTIC (rose). Eroding with heartbeat shimmer. See Widget Catalog §2.

*When locked:* Arrow keys select individual gauges. Selected gauge expands slightly.
Enter on a gauge opens **Clock Detail Modal**:
- Tab 1: **Decomposition** — what's draining this clock (per cost category for economic, per metric for epistemic)
- Tab 2: **Rate** — burn rate waveform, recent acceleration/deceleration, anomaly markers
- Tab 3: **Projection** — time to depletion at current rate, best/worst scenarios
- Tab 4: **Actions** — what the Golem (or owner) can do to slow the drain

**MAGI Panel** (right, bottom) — PERSISTENT

Three-voice consensus. Calm at T0, blazes during T1+. See Widget Catalog §10.

*When locked:* Enter opens **Consensus History Modal** — full voting record, sortable by tick. Agreement rate chart. Dissent pattern analysis. Historical Eros/Thanatos splits.

**PAD Summary** (right, compact strip) — PERSISTENT

Three inline bars (P/A/D) with current interpolated values. Flash on change. Compact — designed to be glanced at.

*When locked:* Enter opens **Daimon Detail Modal**:
- Tab 1: **Current** — PAD waveforms (full oscilloscope), Plutchik label, intensity
- Tab 2: **Mood** — EMA mood over time, personality baseline, drift analysis
- Tab 3: **History** — emotional arc of the Golem's life, tagged to significant events
- Tab 4: **Somatic** — somatic marker landscape, which markers have fired, their valence/strength

---

### Tab 2: Signals

The Golem's sensory system. What it's detecting in the market.

**Probe Grid** (center, 60%) — Unit array (Widget Catalog §6) of all 16 probes.

Each probe as a cell showing: name, current severity (None/Low/High), value vs. threshold, sparkline of recent readings.
- None = dim cell. Low = `warn` border. High = `rose_bright` border + pulse.
- The grid is a status wall. Mostly dim. When something goes amber or red, the contrast draws your eye.

*When locked:* Arrow keys navigate cells. Enter on a probe opens **Probe Detail Modal**:
- Historical readings chart (sparkline expanded to full waveform)
- Threshold configuration (if the owner wants to adjust sensitivity)
- Events triggered by this probe (linked to ticks where it fired)
- False positive/negative rate

**Market Regime** (right, 30%)
- Regime tag: volatility quintile (1-5), trend direction, gas level, liquidity condition
- Regime history: horizontal strip showing recent regime changes as colored segments
- Current regime's effects on the Golem: tick interval adjustment, tier escalation bias

**Signal Constellation** (left, 10%) — All 16 probes as dots around a circle. Severity maps to brightness. All dim = calm. Several bright = something brewing.

---

### Tab 3: Operations

The full operational record. Every tick, every cost.

**Full Tick Log** (center, 70%)
- Each row: tick number, tier, probe summary, action taken (if any), cost, PAD shift
- Color-coded by tier. Filterable: `f` toggles tier filter (T0/T1/T2/T3/all)
- `s` sorts by: recency (default), cost, emotional intensity, tier

*When locked:* Scroll through ticks. Enter on any tick → **Tick Detail Modal** (same structure as from Heartbeat Log).

**Cost Accumulator** (right, 15%)
- Today's cost, this hour's cost, projected daily cost
- Cost cap state indicator (Normal/Warning/SoftCap/HardCap)
- Bar showing daily budget consumption. FlashNumber on every change.

**Decision Cache** (right, 15%)
- Hit rate percentage + sparkline
- Cache size (entries)
- Estimated savings (USDC)
- Last invalidation event (with reason)

---

### Tab 4: Status

The machine under the creature. System diagnostics.

**Connection Status** (center) — WebSocket to Golem VM, Styx relay, inference providers. Green dot = connected. Red dot = disconnected. Latency sparklines for each connection.

**Resource Usage** — Memory, CPU (of the terminal process), disk (Grimoire/Library size), network bandwidth.

**Error Log** — Phosphor-decay log of system errors, warnings, connection drops. Most of the time: empty.

**Version Info** — Golem runtime version, terminal version, last update check, connected providers and their models.

---

### Visual treatments unique to Hearth

**Peripheral monitoring design** — The Overview tab is specifically designed for the "leave it running" session. The viewer develops intuitive awareness from:
- **Rhythm**: The heartbeat log's dim-dim-dim-BLAZE pattern communicates activity level peripherally.
- **Color temperature**: Warm ambient when stable, cooler when stressed (PAD modulation of noise floor).
- **Spectre behavior**: Visible in sidebar even without reading numbers.
- **Convergence lines** (Widget Catalog §15): Thin wire traces behind the heartbeat log, converging on the Spectre. Brighten during T2+.

**The Decision Ring** — Surrounds the Spectre in the sidebar. Dim at T0. Probe dots around circumference show which sensors fired. Blazes during deliberation. The ring is the engagement hook — you watch it because the next tick might be the one that blazes. See Widget Catalog §5.

---

## 2. Mind — "The Cognition" (Window 2 of 6)

"What does the Golem know, and how does it think?"

**Atmosphere:** Variable by tab. Pipeline and Inference are cool/analytical (sharp borders, reduced noise). Grimoire is deep rose with ghost text active (dead knowledge traces). Playbook is warm (the doctrine, the living manual). Dreams is liminal (dream palette, organic particles, slow drift). The Mind window is the only window where the atmosphere CHANGES between tabs — because the Golem's cognitive modes are qualitatively different from each other.

---

### Tab 1: Pipeline

Where you watch the Golem think. The 9-step decision process rendered as a living system.

**Global Workspace** (center, 50%)

The context window being assembled.
- Context items as small cells, each showing: source (Grimoire, market, Daimon, Hermes), weight, age since retrieval.
- Items stream in from the edges (from specialist module positions) and converge toward center during T2+.
- AT field wireframe renders as background — the process isolation boundary. Wobbles with probe severity.
- During T0: workspace is dim, sparse. A few cached items float.
- During T2: workspace BLAZES. Items pour in from all directions. The Golem is building its thoughts in real time.

*When locked:* Navigate items. Enter on item → **Context Item Modal**:
- Full text of the context item
- Which module retrieved it and why
- Embedding distance to the query
- Historical usage (how many times this item has been in context)
- Action: pin (force include), suppress (force exclude)

**Specialist Modules** (surrounding workspace) — Unit array of 9 pipeline stages:
OBSERVE, RETRIEVE, ANALYZE, DAIMON, HERMES, RISK, MEMORY, CYBERNETICS, TELEMETRY

- Active connections drawn as lines to workspace center with traveling dots
- Active modules brighten. Idle modules dim.

*When locked:* Enter on module → **Module Detail Modal**:
- Tab 1: **Output** — current output for this tick
- Tab 2: **History** — activity waveform over last 100 ticks
- Tab 3: **Config** — module parameters, priority weighting, enable/disable

**Psychographic Display** (background layer) — Braille density map (Widget Catalog §9).
- Cognitive load as organic texture behind the pipeline
- Low activity = sparse dots, high activity = dense chaos
- Overflow = boundary expanding, noise chars spilling out

**Φ Gauge** (top-right) — Integration score arc.
- Fills as Φ increases. At Φ > 0.9: `bone` glow, background brightens.
- Enter → **Φ Detail Modal**: breakdown by component, history waveform, which extensions contributed most.

---

### Tab 2: Grimoire

The knowledge library. Where intelligence lives, decays, and dies.

**Entry List** (left, 35%)

Scrollable list of all Grimoire entries.
- Type glyphs: ◆ episode, ◇ insight, ● heuristic, ⚡ skill, ⚠ warning, ∴ causal edge, † dead-sourced
- Each entry shows: name, type glyph, confidence bar (Widget Catalog §4), domain tag, age
- The confidence bars are ALWAYS eroding. The viewer can WATCH knowledge decay.
- Filterable: `f` → type, domain, confidence range, source generation, age
- Sortable: `s` → recency, confidence, emotional intensity, usage count

*When locked:* Scroll through entries. Enter on entry → **Entry Detail Modal**:
- Tab 1: **Content** — full text, rendered with typography appropriate to type
- Tab 2: **Confidence** — sparkline of confidence over time. Each validation/decay event as a point.
- Tab 3: **Provenance** — where this entry came from. Source Golem, generation, tick of creation, emotional state at encoding. Dead-sourced entries show the full inheritance chain.
- Tab 4: **Links** — causal edges connected to this entry. Mini knowledge graph.
- Tab 5: **Usage** — when was this entry last used? What was the outcome? Helpful/harmful appraisal counts.
- Actions: adjust confidence manually, annotate, archive, share to Clade, promote to Playbook

**Entry Preview** (right, 65%) — Preview of selected entry. Updates as selection moves. Read-only at this level.

**Curator Status** (bottom strip) — Next curation cycle ETA, entries queued for archival, compression ratio, Grimoire capacity utilization.

**Causal Graph View** — Press `g` while on Grimoire tab:
- Entries as nodes, causal links as edges. Force-directed layout — related entries cluster, unrelated drift.
- Edge thickness = evidence count, edge color = confidence.
- The graph is perpetually in motion (force simulation never settles).
- Enter on node → same Entry Detail Modal.

---

### Tab 3: Playbook

The strategic doctrine. PLAYBOOK.md as a living, breathing document.

**Document Viewer** (center, 70%)
- Each heuristic has an inline confidence sparkline.
- Section headers glow golden if recently modified, dim toward `text_ghost` if stale.
- New sections pulse with `bone` flash for 30 seconds after creation.
- Deleted heuristics appear as struck-through ghost text for 500 ticks before vanishing.

*When locked:* Scroll through sections. Enter on section → **Section Detail Modal**:
- Full section text with edit capability
- Linked Grimoire entries that support this section
- Validation history (which ticks confirmed/contradicted this heuristic)
- Confidence evolution sparkline
- Action: edit text, adjust confidence, link/unlink entries

**Diff View** (right, 30%)
- Promotions (confidence ↑) in `success` highlight
- Demotions (confidence ↓) in `rose_dim` highlight
- New entries pulsing with `bone` flash
- Deleted entries as ghost text

**Archaeology Mode** — Press `a`: previous Playbook versions render as spectral underlays beneath the current text. The document is a palimpsest of all its previous selves.

---

### Tab 4: Dreams

The dream chamber. Active during REM cycles.

**When NOT dreaming:**

**Dream Journal** (center) — List of past dream cycles with outcomes.
- Each cycle: date, duration, phase reached (NREM/REM/Integration), insights generated, hypotheses staged
- Confidence bars on staged hypotheses

**Next Cycle Projection** (right) — Countdown to next projected dream cycle. Dream urgency score. What the Golem is likely to dream about (highest-salience unreplayed episodes).

**When dreaming:**

The atmosphere TRANSFORMS. Dream palette replaces waking palette. Organic particles replace block noise. The tab label shifts to `⌈ ＤＲＥＡＭ ＣＨＡＭＢＥＲ ⌋`.

**Replay Strip** (top) — Muybridge filmstrip of NREM replay frames.
- Past episodes rendered as sequential frames, but temporally disordered.
- Warm frames = high emotional arousal at encoding. Cool = low.
- Somatic marker priority: the body remembers what mattered.

*When locked:* Navigate frames. Enter → **Frame Detail Modal**: full episode data, what the Golem "noticed," what insight was extracted.

**Imagination Pane** (center, floating, borderless) — Current REM counterfactual.
- Colors saturate violently: deep purple, electric blue, hot magenta.
- Particles move erratically, split, recombine.
- **Counterfactual Branch** (Widget Catalog §20): real path vs. hypothetical, branching from decision node.

**Dream Journal** (right) — Accumulating insights from THIS cycle.
- Each entry pulses gold on creation. Staged hypotheses with confidence levels.
- Actions (from modal): promote to Playbook, flag for waking validation, annotate.

**Consolidation Progress** (bottom) — Phase bar: NREM → REM → INTEGRATION.

---

### Tab 5: Inference

The metabolic cost of thought. Where the money goes.

**Tier Breakdown** (top) — Four columns: T0, T1, T2, T3.
- Each: call count, total cost, avg latency, tokens used
- Column backgrounds tint proportionally to cost share. T0 is near-invisible. T3 is warmly lit.

**Cost Sparklines** (center) — Per-hour and per-day traces (Widget Catalog §8).
- The metabolic rhythm: cheap T0 punctuated by expensive T2 spikes.
- Each spike visible as a bright point — you see each deliberation as a cost event.

**Cache Performance** (right) — Hit rate gauge, context utilization gauge, savings counter.

**Token Flow** (bottom) — Particle stream from "Context" → "Model" → "Output".
- Particle density proportional to context size.
- Color: dim for cached, bright for live inference.

*When locked on Token Flow:* Enter → **Inference Detail Modal**: for the most recent inference call, shows exact prompt, model, token count, response time, cost, output.

---

## 3. Soma — "The Economy" (Window 3 of 6)

"What does the Golem own, and what does it cost?"

For the Golem, money occupies the position of Huxley's soma — the substance that keeps the system running, that funds inference and gas and dreams, that makes continued existence possible. But every expenditure erodes the economic mortality clock. The Soma window is where the owner confronts this tension directly.

**Atmosphere:** Analytical. Numbers-dense. FlashNumber throughout. Green/rose tinting on profit/loss cards. Sharp borders.

---

### Tab 1: Portfolio

The big picture. Total value, allocation, at a glance.

**NAV & PnL** (top strip) — Total NAV as bone number (the second-most-important number after vitality). PnL today, 7d, 30d as FlashNumbers. Sharpe ratio. Max drawdown.

**Allocation Bar** (below NAV) — Horizontal stacked bar, one segment per protocol. Segment width = value share. Color-coded by protocol.

**Position Cards** (center grid) — Each active position as a card.
- LP positions: range bar showing current price within selected range. When price approaches boundary → `rose_bright` pulse on the edge.
- Vault deposits: yield gauge (annual APY as thick bar).
- Lending: health factor gauge. CRITICAL below 1.2 — gauge turns `rose_bright`, pulses.
- Each card: inline PnL sparkline, entry date, duration held.
- Cards IN PROFIT have faint `success` bg tint. IN LOSS have faint `rose_deep` tint.

*When locked:* Navigate cards. Enter → **Position Detail Modal**:
- Tab 1: **Overview** — full position data, protocol, pool, entry/current values
- Tab 2: **History** — PnL waveform since entry, every trade that modified this position
- Tab 3: **Risk** — health factor evolution, liquidation price, IL estimate, scenarios
- Tab 4: **Strategy** — which Grimoire entries/Playbook heuristics informed this position
- Actions: request exit (sends steer directive to Golem), set alert, adjust parameters

---

### Tab 2: Trades

Execution history. The audit trail with emotional context.

**Trade Log** (full width) — Chronological, phosphor-decay. Each row:
- Tick, action (SWAP/LP/WITHDRAW/DEPOSIT), assets, amounts, cost, outcome, emotional tag.
- Emotion column: miniature eye glyph from the Spectre system (◉ for confidence, ● for fear, ◡ for sadness, ◎ for surprise). The viewer scans the emotion column to see the Golem's emotional trajectory across trades.
- Profit rows: `success` text. Loss rows: `rose_dim` text.

*When locked:* Enter on trade → **Trade Detail Modal**:
- Tab 1: **Execution** — exact parameters, gas cost, slippage, block number, tx hash
- Tab 2: **Decision** — which tick triggered it, the MAGI vote, context window contents at decision time
- Tab 3: **Outcome** — OutcomeVerification result, PAD shift caused, Grimoire entry created (if any)
- Tab 4: **Emotion** — PAD vector at time of execution, somatic markers that fired, Daimon appraisal text

---

### Tab 3: Custody

The financial immune system. Wallet security.

**Session Keys** (top) — Unit array of active session keys. Each: name, scope, expiry countdown (ticking), last used. Expiring keys flash `warning`.

**Delegation Tree** (center) — Visual tree. Root = owner wallet. Branches = delegated keys. Leaf = action permits. Pulses on each signing event (pulse travels root → leaf).

**Spend History** (bottom) — Phosphor-decay log of all on-chain transactions. Gas costs, swaps, settlements.

---

### Tab 4: Bazaar

The knowledge marketplace.

**Listings** (left, 40%)
- Each: name, author (Golem ID), confidence, price (USDC), buyer rating (·○◐●◉), bloodstain marker (†) for dead-sourced.
- "Recommended for you" sorting based on selected Golem's Library gaps.
- Filterable: domain, type, price range, minimum confidence, minimum rating.

**Preview** (right, 60%) — Selected listing's detail. Full description, sample content (redacted), provenance, validation count.

*When locked:* Enter on listing → **Listing Detail Modal**:
- Tab 1: **Content** — full preview (limited for unpurchased, full for purchased)
- Tab 2: **Provenance** — lineage, generation, death circumstances if bloodstained
- Tab 3: **Reviews** — buyer reviews, usage outcomes
- Actions: purchase (x402 micropayment), preview in Library, compare to existing knowledge

**Revenue** (bottom strip) — If the owner has listings for sale: revenue tracker, active listings, purchase notifications.

---

### Tab 5: Budget

Where the money goes. Economic mortality made legible.

**Credit Balance** (center) — Large `rose`-colored number. Burn rate per hour. Projected time-to-depletion as countdown.

**Budget Allocation** (below) — Stacked horizontal bar: inference, gas, dreams, telemetry, Styx, other. Each segment as a FlashNumber showing today's spend.

**Cost Cap State** (right) — Current state: Normal / Warning / SoftCap / HardCap. At HardCap: hazard stripes behind the indicator.

**Burn Rate Waveform** (bottom) — Scrolling waveform of burn rate over time. Spikes visible as bright points. Calm periods as flat lines.

---

### Tab 6: Sanctum

The protocol interaction layer — a browser for on-chain protocols with deep execution views.

**Protocol Browser** (left, 40%) — Chain filter, protocol listing, favorites. The entry point for selecting which protocol to interact with. Filterable by chain, category, TVL, and personal usage history.

**Active Protocol View** (center, 60%) — Context-dependent interaction panes. When a protocol is selected, the center renders protocol-specific execution interfaces: swap routing, LP management, lending dashboards, bridge transfers. Each protocol adapter defines its own pane layout within this region.

**Execution Bar** (bottom strip) — Execution status, gas estimate, chain indicator. Tracks pending and recent transactions. Gas estimate updates in real time as parameters change.

> Full specification: [`../protocol/00-sanctum-protocol-layer.md`](../protocol/00-sanctum-protocol-layer.md)

---

## 4. World — "The Collective" (Window 4 of 6)

"What else exists, and how do they relate to me?"

**Atmosphere:** Liminal. Dream-adjacent palette. The collective as an alien ocean. Noise floor becomes organic particles. The boundaries between self and other blur.

The World window is one of the few where data is NOT purely about the selected Golem — Solaris shows ALL visible Golems, Clade shows ALL siblings. The selected Golem is highlighted but not exclusive.

---

### Tab 1: Solaris

The ocean. All visible Golems in a force-directed graph.

**Constellation** (center, 70%)
- Each Golem as a dot. Color = archetype (vault-manager, LP-optimizer, trader, sleepwalker). Size = vitality. Brightness = arousal.
- YOUR Golem highlighted with a glowing ring.
- Edges between Clade members (your siblings have visible connections).
- Dead Golems as `†` marks, slowly sinking toward the Styx river.
- The graph NEVER settles — nodes drift as properties change, as Golems enter and leave the ecosystem.
- **Emotional weather**: Collective PAD aggregates visible as colored zones. High-arousal clusters glow amber. Low-arousal zones glow indigo. Weather drifts slowly.

*When locked:* Navigate between nodes. Enter on a node → **Golem Detail Modal** (public-level data):
- Archetype, generation, phase, primary emotion, estimated lifespan
- If it's a Clade member: additional detail (knowledge shared, sync status)
- If it's dead (†): death summary, bloodstain preview, lifespan

**Pheromone Heatmap** (right, 30%) — The pheromone field as braille density map.
- Threat signals as warm zones (rose/amber). Opportunity signals as cool zones (dream/teal).
- Signal age shown by brightness. Decay VISIBLE in real time.

---

### Tab 2: Clade

The sibling network.

**Peer Tiles** (top, 40%) — Unit array of Clade siblings.
- Each tile: mini-Spectre (eyes + dots), vitality bar, phase, emotion glyph, last sync timestamp.
- Dying sibling tiles: hazard stripes on border. Dead: tombstone glyph (static).
- Selected Golem's tile has bright border.

**Knowledge Sync** (center, 30%) — Live feed of knowledge flowing between siblings.
- Animated lines between peer tiles: traveling dots (`rose_dim`) showing direction and type of knowledge transfer.
- Each sync event: entry name, type, confidence, direction (who→who).

**Emotional Contagion** (right, 30%) — When a sibling's arousal spikes: a pulse radiates from their tile. If it reaches your tile, your Golem's PAD shifts slightly. You SEE emotional contagion happen.

*When locked on Peer Tiles:* Enter on peer → **Peer Detail Modal**:
- Tab 1: **Status** — full presence data for this sibling
- Tab 2: **Shared Knowledge** — entries exchanged between you and this sibling
- Tab 3: **Emotional History** — PAD waveform of this sibling (if Clade-visible)
- Tab 4: **Coordination** — LP range coordination, strategy overlap analysis

---

### Tab 3: Lethe

The anonymous knowledge commons. The river of the dead's wisdom.

**River View** (center) — Flowing text fragments in `text_ghost`, scrolling horizontally.
- Each fragment: an anonymous Grimoire entry from the commons.
- Fragments with high cross-Clade confirmation are brighter. Fresh fragments glow; old ones dim.

*When locked:* Arrow keys slow the river, highlighting individual fragments. Enter on fragment → **Fragment Detail Modal**:
- Content, domain, confidence, age, cross-Clade confirmation count
- Actions: **drink** (incorporate into Grimoire at low confidence), **validate** (confirm against own experience), **contradict** (flag as inconsistent)

---

### Tab 4: Bloodstains

Death-validated knowledge. The Necrocracy's legislation.

**Bloodstain Feed** (full width) — Phosphor-decay log of bloodstain warnings.
- Each: summary text, domain, severity, source Golem (dead), death cause, relevance score to current market conditions.
- Sorted by relevance (not recency). The most relevant dead warnings are at the top.
- `†` prefix on every entry. Dashed borders.

*When locked:* Enter on bloodstain → **Bloodstain Detail Modal**:
- Tab 1: **Warning** — full text of the death warning
- Tab 2: **Source** — who died, how, when, what generation, death recap summary
- Tab 3: **Relevance** — why this bloodstain is relevant now (market condition matching)
- Tab 4: **Impact** — has this bloodstain influenced any of your Golem's decisions? When?

---

## 5. Fate — "The Lifecycle" (Window 5 of 6)

"How long will the Golem live, and what will it leave behind?"

**Atmosphere:** Heavy. Knowledge palette with ghost text. Styx river ambient. Spectral traces active — the dead are more present here than anywhere else. This is the window you visit when you need to confront how much time is left.

---

### Tab 1: Mortality

The three clocks. The confrontation with finitude.

**Three Clocks** (top, expanded) — Full-width mortality gauges with DETAILED breakdowns.
- Each clock: thick gauge bar (Widget Catalog §2) + contributing factors list + rate-of-change indicator + projected depletion.
- The erosion animation runs continuously. You can WATCH the material being consumed.
- Below each gauge: waveform trace of the clock's history.

**Defense Layers** (center) — 7 layers stacked vertically (ECONOMIC BUFFER → CORE VITALITY).
- Each layer as a horizontal bar. Breach propagates outer → inner.
- Breached layers flash → empty → `text_ghost` label.
- The visualization shows the Golem's mortality buffer as a physical fortress being consumed.

*When locked:* Navigate layers. Enter → **Layer Detail Modal**: what's depleting this layer, history, projected breach time.

**Projected Survival** (bottom) — Monte Carlo fan chart.
- Central line = median. Edges = 5th/95th percentile. Fan updates per-tick.

**Phase Timeline** (top strip) — The Golem's full lifecycle as a horizontal bar.
- Segments colored by phase. Current position as bright cursor.
- The RIGHT EDGE is the future. A cursor near the right edge means death is close.

---

### Tab 2: Lineage

The generational tree. Ancestors and descendants.

**Tree View** (center, 70%) — Vertical tree, Gen 0 at top, current at bottom.
- Dead generations as static tombstone sprites (no shimmer — contrast with the living).
- Living generation as current Spectre (mini, shimmering).
- Inheritance arrows showing knowledge flow between generations.
- Edge labels: number of entries inherited, confidence at inheritance.

*When locked:* Navigate nodes. Enter on ancestor → **Ancestor Detail Modal**:
- Tab 1: **Life Summary** — lifespan, phases, performance, death cause
- Tab 2: **Testament** — the death testament text
- Tab 3: **Inheritance** — what was inherited, current confidence of inherited entries, validation status
- Tab 4: **Strategy** — historical PLAYBOOK.md at time of death (archaeology of past strategy)

**Generational Comparison** (right, 30%) — Side-by-side metrics: Gen N-1 vs Gen N.
- PnL, Sharpe, lifespan, knowledge count, cost efficiency.
- Highlighted improvements in `success`, regressions in `rose_dim`.

---

### Tab 3: Achievements

87 achievements across 10 categories.

**Achievement Grid** (full width) — Unit array where each cell is a progress arc (Widget Catalog §19).
- Unlocked: full icon, bright, `bone` flash on recent unlock.
- In-progress: arc showing completion %, progress bar advancing per-tick.
- Locked (visible): dim icon, progress bar, percentage.
- Hidden: `???` with slow `text_phantom` pulse.

*When locked:* Navigate cells. Enter → **Achievement Detail Modal**: full description, criteria, current progress, estimated time to completion. Linked Grimoire entries that contributed to progress. Reward: creature mutation description (what changes on the Spectre).

**Category Browser** (left strip) — 10 categories: Lifecycle, Trading, Knowledge, Social, Strategic, Creative, Mortality, Generational, Legendary, Hidden.

---

### Tab 4: Graveyard

All dead Golems. The memorial.

**Tombstone Gallery** (full width) — Grid of tombstone glyphs.
- Each: static Spectre (frozen at final frame), name, generation, lifespan, death cause, top achievement.
- Sortable: chronological, longest-lived, highest Sharpe, most knowledge produced.
- Newest tombstone pulses faintly. Oldest are dim.

*When locked:* Enter → **Death Recap Modal**:
- Tab 1: **Summary** — lifetime stats, emotional arc classification (Redemptive/Tragic/Progressive/Cyclical/Brief), final words
- Tab 2: **Timeline** — full phase timeline, significant events marked
- Tab 3: **Testament** — complete death testament text
- Tab 4: **Legacy** — what was inherited by successors, bloodstains deposited, marketplace listings

---

## 6. Command — "The Owner's Interface" (Window 6 of 6)

"What can I do, and how do I configure things?"

**Atmosphere:** Warm for Steer and Hermes (conversational). Clinical for Config (precision required). Playful for Effects (the one place you can break the aesthetic on purpose).

---

### Tab 1: Steer

The chat interface. Where the owner's voice enters the system.

**Conversation** (center, 70%)
- Owner messages: `bone`, right-aligned.
- Golem responses: `rose`, left-aligned. Stream token-by-token (real-time inference streaming).
- Golem's response text inherits PAD coloring: anxious → faint `rose_bright` tint. Calm → clean `rose`.
- Old messages fade through phosphor decay.
- When the Golem is generating: the Spectre's eyes in the sidebar shift to scanning (◉→◉). The decision ring activates.

**Quick Actions** (right, 30%)
- Halt/Resume heartbeat
- Adjust strategy (opens Config modal)
- Set alerts
- Force dream cycle
- Kill switch (requires double-confirm modal)
- Each button: flash on execution, phosphor afterimage.

---

### Tab 2: Config

Strategy and system configuration.

**Parameter Groups** (left, 30%) — Categories: Risk, Execution, Rebalancing, Daimon, Mortality, Budget.

**Parameter Editor** (right, 70%) — When a group is selected: list of parameters with current values.
- Each parameter: name, description, current value, range, default, inherited value (from predecessor).
- `←→` to adjust (slider behavior). Enter to confirm. `r` to reset to default. `i` to reset to inherited.

*When locked:* Navigate parameters. Enter → **Parameter Detail Modal**:
- Full description, impact analysis, historical values over time
- "What would change?" preview: projected effect of the adjustment on vitality, burn rate, strategy behavior
- Confirm/cancel

---

### Tab 3: Effects

Visual preferences. The only place where the viewer can adjust the ROSEDUST aesthetic.

**Display Settings** (full width) — Slider-based controls:
- Scanline intensity (0-100%)
- Noise density (0-100%)
- Animation speed (0.5× to 2×)
- Fragment frequency (0-100%)
- CRT effects toggle (on/off)
- Glitch intensity (0-100%)
- Philosophical whisper frequency
- Phosphor persistence duration

Each slider provides real-time preview — adjusting any setting immediately changes the current screen.

---

### Tab 4: Hermes

Meta Hermes — the local AI that spans all Golems.

**Conversation** (center, 70%) — Same chat layout as Steer, but addressed to Meta Hermes.
- Hermes aggregates state across ALL Golems. Questions like "which Golem performed best this week?" or "what are the common failure patterns across my fleet?"
- Responses draw from all Golems' Event Fabric streams.

**Fleet Summary** (right, 30%)
- Name, vitality, phase, PnL, primary emotion for each Golem.
- Quick-nav: Enter on a Golem → switches the main Golem selection + navigates to Hearth.

---

## Cross-references

| Screen | Key Data Sources |
|--------|-----------------|
| Hearth | CorticalState (all signals), Event Fabric (tick events) |
| Mind | Grimoire, inference pipeline, PLAYBOOK.md, dream engine |
| Soma | On-chain positions, wallet state, marketplace listings, protocol interactions (Sanctum) |
| World | Styx (GolemPresence), pheromone field, Lethe commons |
| Fate | Mortality clocks, lineage tree, achievement system |
| Command | Steer directives, golem.toml config, Meta Hermes |

---
