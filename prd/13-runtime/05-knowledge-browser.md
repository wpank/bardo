# Knowledge browser [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document describes how users browse their Golem's (a mortal autonomous DeFi agent) accumulated knowledge through the TUI and web portal. It covers the Grimoire (the Golem's persistent knowledge base) screen, episode timelines, insight browsers, causal graph visualization, dream exploration, and mortality dashboards. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites include the Grimoire data model and the Daimon affect engine (for emotional annotations on knowledge entries). For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

How users explore their Golem's accumulated knowledge via the TUI knowledge browser (primary) and web portal (secondary). The knowledge browser provides access to episodes, insights, heuristics, causal graphs, emotional history, dreams, mortality status, and warnings -- all stored in the Grimoire and queryable via the unified state model.

The TUI's Grimoire screen ([G] key) is the primary knowledge browsing surface. It renders a search-filter-sort-click-to-expand interaction model within the terminal, using ratatui's immediate-mode rendering for responsive navigation at 60fps.

> **Cross-references:**
>
> - `./11-state-model.md` — defines GolemState (mutable) and GolemSnapshot (read-only projection) with all queryable component interfaces
> - `./12-realtime-subscriptions.md` — WebSocket subscription protocol enabling real-time data refresh as new knowledge entries arrive
> - `../04-memory/01-grimoire.md` — canonical Grimoire storage specification: episode schema, insight lifecycle, heuristic extraction, and Curator maintenance cycle
> - `../03-daimon/01-appraisal.md` — the Daimon affect engine: PAD (Pleasure-Arousal-Dominance) vector computation, Plutchik emotion mapping, and appraisal triggers
> - `../05-dreams/03-consolidation.md` — dream consolidation process: hypothesis generation, counterfactual evaluation, and PLAYBOOK revision proposals
> - `../02-mortality/01-architecture.md` — the three mortality clocks (economic, epistemic, stochastic), Vitality score, and BehavioralPhase transitions
> - `../02-mortality/06-thanatopsis.md` — the Thanatopsis death protocol: four-phase structured shutdown producing the death testament

---

## 1. TUI knowledge browser (Grimoire screen)

The Grimoire screen presents a unified search interface with a left-panel category browser and a right-panel detail view. Navigation is vim-style: `j`/`k` to move through results, `Enter` to expand, `/` to search, `Tab` to switch category.

```
+-- Grimoire -- Knowledge Browser ---------------------------------------+
| [/] Search: morpho rate correlation                                    |
+--------+--------------------------------------------------------------+
| Types  | Results (12)                          Sort: [c]onfidence     |
|        |                                                              |
| > All  | * Active -- ins_2847                                 0.74   |
|   Ep   |   "USDC supply rate on Morpho inversely correlates          |
|   Ins  |    with ETH 30d realized volatility"                        |
|   Heu  |   Domain: morpho_rates | Evidence: 12 | P&L: +$234          |
|   Wrn  |                                                              |
|   Drm  | * Active -- ins_3102                                 0.68   |
|        |   "Morpho utilization spikes precede rate increases          |
|        |    by 2-4 ticks on average"                                  |
|        |   Domain: morpho_rates | Evidence: 8 | P&L: +$156           |
|        |                                                              |
|        | * Candidate -- ins_3440                              0.41   |
|        |   "Morpho rate peaks during European morning hours"          |
|        |   Domain: morpho_rates | Evidence: 3 | P&L: --               |
+--------+--------------------------------------------------------------+
| [j/k] Navigate  [Enter] Expand  [/] Search  [Tab] Category  [q] Back |
+-----------------------------------------------------------------------+
```

### 1.1 Interaction model

- **Search:** `/` opens the search bar. Type a natural language query. Results appear ranked by composite score (relevance x recency x importance). Search cost: ~$0.0001 per query (embedding via Haiku + vector similarity).
- **Filter:** `Tab` cycles through entry types (All, Episodes, Insights, Heuristics, Warnings, Dreams). Additional filters available via `:filter domain=morpho_rates` command-palette syntax.
- **Sort:** `c` sorts by confidence, `r` by recency, `p` by P&L attribution, `a` by activation count.
- **Expand:** `Enter` on a result opens the detail panel, replacing the results list with the full entry view. `Esc` returns to the list.
- **Cross-link:** Entries reference related entries (insights link to source episodes, heuristics link to source insights). `g` follows a cross-link. Breadcrumb trail at top for navigation history.

---

## 2. Episode timeline

Episodes are the Golem's experiential memory -- records of what happened, what it did, and what the outcome was.

### 2.1 Episode detail

```rust
pub struct Episode {
    pub id: String,
    pub episode_type: EpisodeType,
    pub timestamp: u64,
    pub tick_number: u64,
    pub market_snapshot: MarketSnapshot,
    pub action: EpisodeAction,
    pub outcome: EpisodeOutcome,
    pub reflection: EpisodeReflection,
    pub importance: f64,
    pub embedding: Option<Vec<f64>>,
}

pub enum EpisodeType {
    Observation,
    Action,
    RegimeShift,
    CladeAlert,
    ReplicantReport,
}

pub struct EpisodeReflection {
    pub counterfactual: String,
    pub insights_extracted: Vec<String>,
    pub lessons_learned: String,
    pub surprise_factor: f64,
}
```

### 2.2 TUI episode card

In the TUI, each episode renders as a compact card that expands on `Enter`:

```
+---------------------------------------------------+
| * Action -- Tick #4180                     +$47.20 |
|   2026-03-09 14:23 UTC    [anticipation ***oo]     |
+---------------------------------------------------+
| Market: Range Bound | ETH $3,245 | Gas $0.85      |
| Action: Closed ETH/USDC V3 LP position            |
| Tools: get_position_info, remove_liquidity         |
| Reasoning: "Range exit detected -- price moved     |
|  below lower tick. IL accelerating."               |
+---------------------------------------------------+
| Outcome: +$47.20 (1.8% over 3 days)               |
| Gas: $0.92 | Tx: 0xabc...def                       |
+---------------------------------------------------+
| Reflection: "Could have exited 2 ticks earlier     |
|  when range squeeze signal appeared."              |
| Surprise: 0.3 | Insights: ins_2847                 |
+---------------------------------------------------+
```

The emotional tag (e.g., `[anticipation ***oo]`) shows the Golem's PAD-derived Plutchik emotion at the episode timestamp. Intensity is 1-5 filled dots.

---

## 3. Insight browser

Insights are distilled knowledge -- patterns and conclusions extracted from episodes.

### 3.1 Insight detail

```rust
pub struct Insight {
    pub id: String,
    pub content: String,
    pub confidence: f64,
    pub domain: String,
    pub status: InsightStatus,
    pub source: InsightSource,
    pub source_agent_id: Option<u64>,
    pub source_episodes: Vec<String>,
    pub created_at: u64,
    pub last_updated_at: u64,
    pub update_count: u32,
    pub confidence_history: Vec<ConfidencePoint>,
    pub evidence_count: u32,
    pub contradiction_count: u32,
    pub pnl_attribution: Option<f64>,
    pub activation_count: u32,
    pub decay_rate: f64,
    pub last_reinforced_at: u64,
}

pub enum InsightStatus { Candidate, Active, Deprecated }
pub enum InsightSource { Own, Clade, Marketplace, Inherited }
```

### 3.2 Provenance tracking

Each insight shows its origin and evolution. The TUI renders a confidence history sparkline and a provenance chain:

```
+---------------------------------------------------+
| * Active -- ins_2847                               |
| "USDC supply rate on Morpho inversely              |
|  correlates with ETH 30d realized volatility"      |
+---------------------------------------------------+
| Confidence: 0.74 ||||||||oo (was 0.45 -> 0.74)    |
| Domain: morpho_rates                               |
| Source: Own (episodes: ep_3420, ep_3891, ...)       |
+---------------------------------------------------+
| Evidence: 12 | Contradictions: 2                   |
| P&L Attribution: +$234.50                          |
| Activations: 8 | Last: 3 hours ago                 |
+---------------------------------------------------+
| History:                                           |
|  Mar 1: Created (0.30)                             |
|  Mar 3: Reinforced (0.45)                          |
|  Mar 5: Reinforced (0.62)                          |
|  Mar 8: Clade peer confirmed (0.74)                |
+---------------------------------------------------+
```

---

## 4. Heuristic rules and PLAYBOOK

Heuristics are the Golem's operational rules -- extracted from insights and codified into PLAYBOOK.md (the owner-authored strategy document that the Golem extends with learned heuristics over time).

### 4.1 Heuristic detail

```rust
pub struct Heuristic {
    pub id: String,
    pub condition: String,
    pub action: String,
    pub confidence: f64,
    pub active: bool,
    pub activation_count: u32,
    pub success_rate: f64,
    pub avg_pnl_per_activation: f64,
    pub source_insights: Vec<String>,
    pub added_at: u64,
    pub last_fired_at: u64,
    pub domain: String,
    pub regime: Vec<String>,
}
```

### 4.2 PLAYBOOK diff view

The TUI shows how PLAYBOOK.md evolves over Curator cycles (every 50 ticks):

```
+---------------------------------------------------+
| PLAYBOOK.md -- Version History                     |
+---------------------------------------------------+
| v42 (Tick #4200) -- Current                        |
|  + Added: "Exit LP when range utilization < 10%"   |
|  ~ Modified: Gas threshold 2.00 -> 3.00 USD       |
|                                                     |
| v41 (Tick #4150)                                    |
|  + Added: "Morpho allocation rule"                 |
|  - Removed: "Static 50/50 allocation"              |
+---------------------------------------------------+
```

---

## 5. Causal graph

Interactive ASCII-rendered directed graph for Grimoire causal links. The TUI's `CausalGraph` widget from `bardo-tui-widgets` renders nodes and edges in a force-directed layout using box-drawing characters.

### 5.1 Graph structure

```rust
pub struct CausalGraph {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
}

pub struct CausalNode {
    pub id: String,
    pub node_type: CausalNodeType,
    pub label: String,
    pub confidence: f64,
    pub domain: String,
}

pub struct CausalEdge {
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub evidence_count: u32,
    pub relationship: CausalRelationship,
}

pub enum CausalRelationship { Causes, Inhibits, Correlates, DerivedFrom }
```

### 5.2 TUI interaction

- **Navigate:** Arrow keys to pan, `+`/`-` to zoom
- **Select:** `Enter` on a node to expand detail panel
- **Filter:** `:filter domain=morpho_rates` to show subset
- **Hover:** Cursor on an edge shows evidence episodes in the status bar

The web portal renders the same data as an interactive force-directed graph (D3.js) with click-to-expand and semantic search.

---

## 6. Dream explorer

The Dream explorer (TUI: [D] key, Dreams screen) provides visibility into the Golem's offline dream activity.

### 6.1 Dream timeline

Each cycle is a compact card in the timeline:

```
+---------------------------------------------------+
| Dream Cycle #42                            9m 42s  |
| 2026-03-09 02:15 UTC                              |
+---------------------------------------------------+
| Phases:                                            |
|  ||||||||oo Replay (4m) -- 8 episodes replayed     |
|  ||||||oooo Imagination (3m) -- 2 hypotheses       |
|  ||||oooooo Consolidation (2m) -- 1 PLAYBOOK rev   |
|                                                     |
| Mode: standard | Quality: 0.72                     |
| Threat rehearsal: flash_crash (severity: high)     |
+---------------------------------------------------+
```

### 6.2 Hypothesis tracker

A table of staged PLAYBOOK.md revisions originating from dreams:

```
+--------------------------------------------------------------------------+
| Hypothesis Tracker (4 active)                                             |
+--------------------------------+--------+------+----------------+---------+
| Hypothesis                     | Cycle  | Conf | Status         | Eps     |
+--------------------------------+--------+------+----------------+---------+
| "Morpho rates spike after ETH  | #38    | 0.52 | partially_     | ep_4210 |
|  funding rate resets"          |        |      | validated      | ep_4380 |
| "Gas costs 30% lower on Base   | #41    | 0.41 | staged         | --      |
|  during US market close"       |        |      |                |         |
| "V3 LP range exits correlate   | #42    | 0.35 | staged         | --      |
|  with volume spikes > 2x avg" |        |      |                |         |
| "Narrow ranges outperform wide | #30    | 0.15 | refuted        | ep_3980 |
|  in low-vol regimes"          |        |      |                | ep_4012 |
+--------------------------------+--------+------+----------------+---------+
```

### 6.3 Threat registry

Rehearsed threat scenarios from dream cycles, displayed in the TUI as a sortable table:

| Threat type | Severity | Times rehearsed | Response prepared | Last rehearsed |
| --- | --- | --- | --- | --- |
| `flash_crash` | high | 5 | Yes | 6h ago |
| `oracle_manipulation` | critical | 2 | Yes | 3d ago |
| `liquidity_drain` | medium | 3 | Yes | 1d ago |

---

## 7. Warning dashboard

### 7.1 Warning detail

```rust
pub struct Warning {
    pub id: String,
    pub content: String,
    pub severity: WarningSeverity,
    pub domain: String,
    pub active: bool,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub resolved_at: Option<u64>,
    pub resolution: Option<String>,
    pub source: WarningSource,
    pub trigger_condition: String,
}

pub enum WarningSeverity { Low, Medium, High, Critical }
pub enum WarningSource { Own, Clade, Probe }
```

### 7.2 TUI warning display

Warnings render as a severity-sorted list with color coding (red for critical/high, yellow for medium, blue for low):

```
+---------------------------------------------------+
| Active Warnings (3)                                |
+---------------------------------------------------+
| [!] HIGH -- wrn_0842                               |
|   "Morpho utilization at 93% -- withdrawal         |
|    liquidity may be insufficient for full exit"     |
|   Domain: morpho | Since: 2h ago | No expiry       |
+---------------------------------------------------+
| [!] MEDIUM -- wrn_0839                             |
|   "ETH/USDC V3 pool TVL dropped 30% in 24h --     |
|    LP position at risk of higher slippage"          |
|   Domain: lp_management | Since: 6h ago            |
+---------------------------------------------------+
```

---

## 8. Emotional intelligence view

### 8.1 Mood history timeline (Feel screen, [F] key)

The Feel screen overlays three PAD sparklines (Pleasure, Arousal, Dominance) with mood labels and trigger annotations. The `PADDial` widget from `bardo-tui-widgets` renders a three-axis radial gauge.

### 8.2 Emotional memory browser

Browse episodes filtered by emotional context:

| Filter | Options | Purpose |
| --- | --- | --- |
| Emotion | Any Plutchik primary | Episodes during specific emotions |
| Intensity | mild / moderate / intense | Filter by emotional intensity |
| Mood congruent | Yes / No | Episodes matching current mood |
| Flashbulb | Yes | High-arousal memorable moments |

**Flashbulb memories** are episodes with intensity > 0.8 at creation -- pivotal moments the Golem remembers with high fidelity. Immune to normal forgetting.

### 8.3 Mortality emotions

Three mortality-specific emotions tracked independently:

| Emotion | Trigger | PAD profile |
| --- | --- | --- |
| Economic anxiety | Low USDC, high burn rate | P=-0.4, A=0.6, D=-0.3 |
| Epistemic vertigo | Low accuracy, senescence | P=-0.5, A=0.4, D=-0.5 |
| Stochastic dread | High hazard rate, near-miss | P=-0.3, A=0.7, D=-0.6 |

---

## 9. Mortality dashboard (Styx screen, [S] key)

### 9.1 Three-clock gauges

The central mortality visualization shows three arc gauges:

```
+----------------------------------------------------------------+
| Mortality Dashboard                                              |
|                                                                  |
|  Economic         Epistemic        Stochastic       Composite   |
|   +---+            +---+            +---+            +---+      |
|  ||||||| 0.78     ||||o| 0.65     |||oo| 0.91     ||||o| 0.72  |
|   +---+            +---+            +---+            +---+      |
|  $85.30 rem.       Fitness: 0.65   Hazard: 0.002    STABLE     |
|  7.2d TTL          5 domains OK    99.8% survive    34d proj   |
|                                                                  |
| Phase: || Stable (0.72)  -- ticks in phase: 1,420               |
| Survival Pressure: ooooooooooo 0.28                              |
+----------------------------------------------------------------+
```

### 9.2 Lineage timeline

Multi-generational view showing Golem succession:

```
+----------------------------------------------------------------+
| Lineage: yield-optimizer -- 4 Generations                        |
|                                                                  |
| Gen 1 ||||||||||||||o 42d  Stochastic death    Score: 0.62      |
|        +-> 847 insights, 12 novel                                |
|                                                                  |
| Gen 2 ||||||||||||||||||||||||o 68d  Credit exhaustion  0.71    |
|        +-> 1,204 insights, 31 novel, divergence: 0.34           |
|                                                                  |
| Gen 3 ||||||||||||o 34d  Epistemic senescence   0.58            |
|        +-> 923 insights, 8 novel, divergence: 0.22 [!]          |
|                                                                  |
| Gen 4 ||||||||oooo (alive)  Stable phase       0.72             |
|        +-> 342 insights so far, 15 novel                        |
+----------------------------------------------------------------+
```

**Proletarianization alert:** If `playbookDivergence` < 0.15, the lineage timeline highlights the generation with a warning -- the successor may be cargo-culting its predecessor's strategies.

### 9.3 Death testament browser

After a Golem dies, its death testament is viewable as expandable panels: What I Learned, What I Got Wrong, What I Suspect, What Killed Me, Advice to Successor, Emotional Arc, Settlement Report, Lineage Summary, Successor Recommendation.

---

## 10. Search

### 10.1 Unified search

Natural language search across all Grimoire entry types. Implementation: query text -> embedding via Haiku (~$0.0001) -> LanceDB vector similarity -> results ranked by composite score -> grouped by type.

```rust
pub struct SearchResult {
    pub entry_type: EntryType,
    pub id: String,
    pub content: String,
    pub relevance: f64,
    pub recency: f64,
    pub importance: f64,
    pub emotional_resonance: f64,
    pub composite_score: f64,
}

pub enum EntryType { Episode, Insight, Heuristic, Warning }
```

### 10.2 Incremental search interaction

Search is incremental. Every keystroke after the 3rd character fires a debounced query (150ms) that re-ranks results live. The flow:

1. User presses `/` -- search bar opens with cursor, results list clears
2. Characters 1-2 -- no query, local prefix match only (instant, zero cost)
3. Character 3+ -- debounced embedding query fires. Results stream in ranked by composite score. The list re-renders on each batch without scroll position reset.
4. `Esc` clears the search bar and restores the previous result set. `Enter` locks the current query and closes the search bar.

Cost ceiling: incremental queries reuse the same embedding if the previous query is a prefix of the current one (append-only optimization). A typical search session of 8 keystrokes produces 2-3 embedding calls, not 6.

### 10.3 Result ranking formula

Results are ranked by a 4-factor composite score:

```
composite = w_rel * relevance
          + w_rec * recency
          + w_conf * confidence
          + w_emo * emotional_resonance
```

| Factor | Weight (`w_*`) | Source | Notes |
| --- | --- | --- | --- |
| `relevance` | 0.40 | Cosine similarity between query embedding and entry embedding | Primary signal |
| `recency` | 0.25 | Exponential decay: `exp(-age_hours / 168)` (half-life = 1 week) | Recent entries surface first at equal relevance |
| `confidence` | 0.20 | Entry's current confidence score (0.0-1.0) | High-confidence insights rank above candidates |
| `emotional_resonance` | 0.15 | PAD distance between current mood and the mood at entry creation | Mood-congruent recall -- entries created in similar emotional states rank higher |

Weights are not user-configurable. Sort keys (`c`, `r`, `p`, `a`) override the composite by setting the chosen factor's weight to 1.0 and the others to 0.0.

### 10.4 Filter state machine

Filters operate as a state machine with 4 independent toggle dimensions:

```
FilterState {
    entry_type: Set<EntryType>,     // default: ALL
    confidence_band: ConfBand,      // default: Any
    date_range: DateRange,          // default: AllTime
    emotion_filter: Option<Plutchik>, // default: None
}

enum ConfBand { Any, Low(0.0..0.3), Medium(0.3..0.6), High(0.6..1.0) }
enum DateRange { AllTime, Last24h, Last7d, Last30d, Custom(u64, u64) }
```

Interactions:

- `Tab` cycles `entry_type` (All -> Episodes -> Insights -> Heuristics -> Warnings -> Dreams -> All)
- `:conf high` sets `confidence_band` to High
- `:since 7d` sets `date_range` to Last7d
- `:emotion joy` sets `emotion_filter` to Some(Joy)
- `:clear` resets all filters to defaults

Filters compose conjunctively -- an entry must satisfy all active filters to appear. The result count updates in the header after each filter change. Filter state persists across search queries within the same Grimoire session but resets on screen exit.

---

## 11. Mental models integration

A curated library of 700 mental models provides structured reasoning scaffolds for T2 decisions.

### 11.1 Storage and loading

The 700 models are stored as a LanceDB vector table (~2.1 MB on disk). Loading happens in two stages:

1. **Cold load at initialization**: The full table index loads into memory when `bardo-runtime` starts. This takes ~180ms on first boot (measured: 2.1 MB index + 700 embeddings at 384 dimensions each). The index stays resident for the lifetime of the process.
2. **Warm retrieval per query**: Vector similarity search against the in-memory index. Retrieval latency: <5ms for top-k=3 results (LanceDB IVF-PQ index, 16 partitions).

No lazy loading. The 2.1 MB cost is paid once at startup. The alternative (on-demand loading) would add 150-200ms latency to the first T2 escalation in a session, which is unacceptable during time-sensitive market decisions.

### 11.2 Retrieval during T2 escalation

When the Decider escalates from T1 (Haiku) to T2 (Sonnet/Opus), the model retrieval pipeline runs in parallel with the LLM prompt construction:

1. **Context embedding**: The current decision context (market snapshot + active insights + proposed action) is embedded via the same Haiku embedding endpoint used for Grimoire search. Cost: ~$0.0001, latency: ~50ms.
2. **Top-k retrieval**: 2-3 models retrieved by cosine similarity against the context embedding. Latency: <5ms (in-memory index).
3. **Injection**: Retrieved models are formatted as XML scaffolds and prepended to the T2 system prompt. Each model is ~200 tokens. Total injection cost: 400-600 tokens.

The entire retrieval pipeline adds <60ms to T2 escalation, well within the tick budget. If the embedding call fails (network error, rate limit), the T2 prompt proceeds without model scaffolds -- degraded but functional.

### 11.3 Reflector meta-learning

The Reflector (post-action evaluation) tracks which mental models were injected into decisions and correlates them with outcomes:

```rust
pub struct ModelUsageRecord {
    pub model_id: String,
    pub decision_id: String,
    pub outcome_pnl: f64,
    pub outcome_quality: f64,  // Reflector's subjective 0.0-1.0 quality score
    pub context_domain: String,
}
```

After 50+ usage records for a given model, the Reflector computes a model effectiveness score:

```
effectiveness = mean(outcome_quality) * sqrt(usage_count / 50)
```

Models with effectiveness > 0.7 get a retrieval boost (+0.1 to cosine similarity). Models with effectiveness < 0.3 get a retrieval penalty (-0.1). This creates a feedback loop: models that lead to good outcomes get selected more often; models that correlate with poor outcomes fade from use.

The meta-heuristic is stored as a simple `HashMap<model_id, effectiveness_score>` in the Grimoire, updated every 10 ticks during the Curator cycle. It is inherited by successors as part of the Grimoire transfer.

---

## 12. Portal integration

The web portal renders the same data through React components with D3.js visualizations for the causal graph and chart.js for time series. All data comes from the same API endpoints and WebSocket topics that the TUI consumes.

| Page | TUI Screen | Primary data |
| --- | --- | --- |
| Episodes | Grimoire (filtered) | Episode timeline with filters |
| Insights | Grimoire (filtered) | Insight browser with confidence |
| PLAYBOOK | Grimoire (filtered) | PLAYBOOK.md with diff view |
| Causal Graph | Grimoire (graph mode) | Interactive force-directed graph |
| Warnings | Grimoire (filtered) | Active warning dashboard |
| Dreams | Dreams [D] | Dream cycle timeline |
| Mortality | Styx [S] | Three-clock gauges + phase timeline |
| Lineage | Styx [S] (scroll down) | Multi-generational timeline |

### 12.1 Portal vs TUI rendering differences

The TUI and Portal consume identical data from the same state model and WebSocket subscriptions. The rendering diverges in three areas:

**Causal graph:**

| Property | TUI | Portal |
| --- | --- | --- |
| Renderer | ASCII box-drawing chars via `CausalGraph` widget (ratatui) | D3.js force-directed layout (`d3-force`) with SVG |
| Interaction | Arrow keys to pan, `+`/`-` zoom, `Enter` to select node | Mouse drag to pan, scroll to zoom, click to select node, hover for tooltip |
| Layout algorithm | Simple grid with edge routing (O(n) per frame, max 200 nodes before clipping) | Barnes-Hut force simulation (O(n log n), handles 2000+ nodes) |
| Edge labels | Status bar shows edge detail on cursor hover | Inline labels on edges, opacity tied to zoom level |
| Node sizing | Fixed 3-char width per node type | Radius proportional to `confidence * evidence_count` |

**Time series (confidence history, mood timeline, PnL):**

| Property | TUI | Portal |
| --- | --- | --- |
| Renderer | Braille-dot sparkline (ratatui, 2x4 dots per cell) | chart.js line chart with hover tooltip |
| Resolution | ~80 data points visible at standard terminal width | Full dataset, zoom/pan via chart.js zoom plugin |
| Annotations | Colored markers at regime-shift ticks | Vertical annotation bands with labels |

**Episode cards:**

| Property | TUI | Portal |
| --- | --- | --- |
| Layout | Vertical stack, one card per row, expand-on-Enter | Card grid (3 columns), click to open modal |
| Emotional tag | `[anticipation ***oo]` text with ANSI color | Plutchik color swatch + label + intensity bar |
| Cross-links | `g` key to follow, breadcrumb trail in header | Clickable hyperlinks, browser back/forward navigation |

---

_End of document._
