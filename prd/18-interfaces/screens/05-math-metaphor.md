# ⌈ ＢＡＲＤＯ ＤＥＳＩＧＮ ＳＹＳＴＥＭ ⌋
## Math-to-Metaphor Translation · v1.0
### Making Complex Mathematics Intuitive

**Cross-references:** `02-widget-catalog.md` (33-widget TUI component library with detail_tier property), `../rendering/02-visualization-primitives.md` (13 visualization primitives for terminal data displays), `../perspective/00-nooscopy.md` (Nooscopy: Golem-initiated decision approval modal, also known as Golem perspective overlay), `00-screen-catalog.md` (29-screen summary with screen placements)

> **Reader orientation:** This document specifies the math-to-metaphor translation layer, mapping every mathematical construct in the Golem's prediction and evaluation systems to a user-facing metaphor. It belongs to the **interfaces/screens layer**. The user never needs to understand persistent homology or KL divergence; the math determines what happens on screen while the metaphor tells the user what they need to know. Key concepts: Golem (a mortal autonomous DeFi agent), the Spectre (dot-cloud creature whose visual behavior encodes mathematical signals), Grimoire (persistent knowledge store), and CorticalState (the 32-signal perception surface feeding all widgets). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## Overview

Every mathematical construct in the system has a user-facing metaphor. The user never needs to understand persistent homology, sheaf cohomology, or KL divergence. The math determines what happens on screen. The metaphor tells the user what they need to know.

This document specifies the translation layer: what each technical term becomes in the user's experience, how detail hierarchies prevent information overload, and how widgets expose different levels of mathematical depth.

---

## 1. Translation Table

| Technical Term | User-Facing Metaphor | What It Actually Means | Widget Context |
|---|---|---|---|
| Bayesian surprise | **Unexpectedness score** | How much a single observation updated the Golem's beliefs (KL divergence between prior and posterior) | FlashNumber flash intensity scales with surprise magnitude |
| Persistence diagram | **Market structure map** | Scatter plot of topological features showing which market structures are stable vs. transient | PersistenceDiagramWidget renders this as a star chart |
| Wasserstein distance | **Structure change rate** | How much the market's geometric structure changed between successive observations | WassersteinRiver ribbon width encodes this value |
| KL divergence | **Belief shift** | The information-theoretic distance between what the Golem expected and what actually happened | Surprise burst cinematic triggers above 3.0 nats |
| Betti numbers | **Feature count** | How many distinct clusters (β₀), cycles (β₁), and voids (β₂) exist in the observation space | FlashNumber inline counters on Chain Intelligence tab |
| Somatic marker | **Gut feeling** | A learned body-level response to a pattern, formed from past emotional outcomes associated with similar situations | SomaticMarkerPanel shows active markers with strength bars |
| Hedge weights | **Signal trust levels** | How much the system trusts each specialist module's contribution to the final decision | ConfidenceBar per specialist in MIND > Pipeline |
| Ergodicity gap | **Hidden risk** | The difference between time-average and ensemble-average returns, measuring whether past performance predicts future results for THIS specific Golem | MortalityGauge variant on stochastic clock detail |
| Sheaf consistency | **Agreement score** | Whether the Golem's short-term, medium-term, and long-term views of the market agree with each other | MortalityGauge variant; screen color harmony in Portal mode |
| Phi (integrated information) | **Coherence** | How tightly the Golem's subsystems are communicating, a measure of cognitive integration | Spectre cloud cohesion; Φ gauge arc in MIND > Pipeline |
| UMAP projection | **Similarity map** | A 2D projection that places similar things near each other in visual space | SimilarityLandscape topographic heatmap |
| Vietoris-Rips complex | **Observation neighborhood** | The geometric structure built by connecting nearby market observations | Not rendered directly; feeds the persistence diagram |
| Hodge Laplacian | **Consistency detector** | The mathematical operator that finds contradictions between timescales | Not rendered directly; feeds the agreement score |
| HDC cosine similarity | **Pattern resemblance** | How closely two encoded patterns match in the Golem's memory space | Proximity in SimilarityLandscape; clustering in ForceGraph |
| Causal DAG | **Cause-effect chain** | Directed graph of discovered relationships where A reliably precedes B | CausalGraphMinimap; ForceGraph in MIND > Grimoire |

---

## 2. Three-Tier Detail Hierarchy

Every signal in the system renders at one of three tiers. The tier controls how much mathematical detail is exposed. Users navigate between tiers by drilling into widgets (Enter) or backing out (Esc).

### Tier 1: Overview (Metaphors Only)

**Purpose:** Ambient awareness. Glance-readable. No numbers, no labels, just color, rhythm, and atmosphere.

**Visual register:** ROSEDUST palette shifts, heartbeat rhythm, Spectre behavior, noise floor density. The user develops intuitive pattern recognition after hours of peripheral monitoring.

| Signal Family | Tier 1 Presentation | Normal | Alarming |
|---|---|---|---|
| Topology | WassersteinRiver in sidebar | Thin quiet stream | Wide bright flood |
| Consistency | Screen color harmony | Unified palette | Fractured color temperatures |
| Coherence (Phi) | Spectre cloud cohesion | Tight, breathing normally | Loose, dots drifting apart |
| Gut feelings | Spectre body zone disturbances | Calm | Gut contractions, unfocused eyes |
| TA patterns | Background activity level | Occasional dim flickers | Cascade of activations |
| Surprise | Flash frequency | Rare | Frequent bright flashes |

**Screen placement:** HEARTH > Overview (Tab 1). The default idle screen, designed for a second monitor.

### Tier 2: Drill-Down (Metaphor + Number)

**Purpose:** Directed investigation. The user noticed something at Tier 1 and wants to understand what is happening.

**Visual register:** Named widgets with labels, sparklines, gauge values. Metaphor labels appear alongside numeric values. Categories are labeled in plain language.

| Signal Family | Tier 2 Presentation | Screen |
|---|---|---|
| Topology | "Market structure map" with feature counts, "Structure change rate" ribbon with numeric annotations, regime tag | FATE > Chain Intelligence |
| Consistency | "Agreement score" gauge (0-100%), contradiction count, disagreeing timescales named | SOMA > Epistemic, FATE > Chain Intelligence |
| Coherence | "Coherence" waveform with numeric value, weakest link identified by name | MIND > Pipeline |
| Gut feelings | "Active gut feelings" list with strength bars, pattern name, affect direction | FATE > Technical Analysis, SOMA > Affect |
| TA patterns | Signal battery with per-pattern states, "trust level" overlays | FATE > Technical Analysis |
| Surprise | "Unexpectedness" distribution, per-element scores | FATE > Oracle Overview |

**Interaction:** Lock into any Tier 2 pane to navigate elements. Each element is selectable. Enter opens a Tier 3 modal.

### Tier 3: Deep-Dive (Full Math)

**Purpose:** Complete mathematical detail for users who want to see the raw computation. Debugging, verification, parameter tuning.

**Visual register:** Modals with data tables, formulas, computation traces. This is where the math lives for those who seek it.

| Signal Family | Tier 3 Modal Content |
|---|---|
| Topology | Full persistence diagram with (birth, death) coordinates, Wasserstein distance computation, VR complex stats (simplex count, max dimension, optimal ε) |
| Consistency | Hodge Laplacian eigenvalues, per-edge disagreement vectors, restriction map parameters, raw observation vectors per timescale |
| Coherence | Mutual information matrix (7x7 subsystem pairs), all 63 bipartition scores, MIB history, signal grouping table |
| Gut feelings | Marker binding vectors (HDC), retrieval SNR, somatic map saturation, full PAD decoding, prototype similarity |
| TA patterns | Pattern hypervector details, fitness history, mutation log, crossover parents, attention auction bid history |
| Surprise | Prior/posterior parameter values, conjugate model type, KL divergence computation, sufficient statistics history |

**Access:** Enter on any element in a Tier 2 pane. Modal system supports infinite nesting (see `03-interaction-hierarchy.md`). The user can drill from a topology feature to the observation vectors forming the simplex, to the specific gamma tick, to the market data at that tick.

---

## 3. Widget `detail_tier` Property

Every widget that renders research-derived signals carries a `detail_tier` property that controls which tier of information it currently displays. This property is part of the widget's state, not its configuration. It changes based on user navigation.

```rust
pub enum DetailTier {
    /// Metaphors only. Color, shape, rhythm. No text labels, no numbers.
    /// Used in ambient/overview screens and Portal mode (F4).
    Overview,

    /// Metaphor label + numeric value. Named categories, gauge fills,
    /// sparkline traces. The standard operating mode for most screens.
    DrillDown,

    /// Full mathematical detail. Data tables, formulas, raw values.
    /// Activated by Enter on a DrillDown element, rendered in modals.
    DeepDive,
}

pub trait TieredWidget: Widget {
    fn detail_tier(&self) -> DetailTier;
    fn set_detail_tier(&mut self, tier: DetailTier);

    /// Returns the metaphor label for this widget's data.
    /// Used in Tier 1 and Tier 2 rendering.
    fn metaphor_label(&self) -> &str;

    /// Returns the technical label for Tier 3 rendering.
    fn technical_label(&self) -> &str;
}
```

### Tier transitions

| Navigation action | Tier change |
|---|---|
| Navigate to HEARTH > Overview | All visible widgets → `Overview` |
| Navigate to a detail screen (FATE > Chain Intelligence, etc.) | Widgets → `DrillDown` |
| Enter on a DrillDown widget | Opens modal at `DeepDive` tier |
| Esc from DeepDive modal | Returns to `DrillDown` |
| F4 (Portal mode) | Forces `Overview` tier on all non-modal widgets |
| F4 + F2 (Portal + Golem Perspective) | `Overview` tier + floating annotations from CorticalState |

### Implementation notes

- Tier 1 widgets render using PAD-modulated color and rhythm only. Text rendering is suppressed. The widget's `metaphor_label()` may appear as a dim `text_ghost` watermark.
- Tier 2 widgets use `metaphor_label()` as the visible label, not `technical_label()`. "Agreement score: 72%" rather than "Sheaf consistency: 0.72".
- Tier 3 modals use `technical_label()` as headers. "Sheaf Consistency — Hodge Laplacian Eigenvalues" is the full formal heading.
- Widgets that do not implement `TieredWidget` default to `DrillDown` behavior on all screens.

---

## 4. Metaphor Consistency Rules

The same mathematical phenomenon always produces the same visual metaphor, regardless of which screen or context it appears in.

| Mathematical pattern | Visual metaphor | Examples |
|---|---|---|
| Fragmentation (increasing separation) | Spatial dispersal | High β₀ scatters PersistenceDiagram points. Low Φ disperses Spectre cloud. Lifecycle degradation scatters fringe dots. |
| Contradiction (conflicting signals) | Color temperature split | Sheaf inconsistency splits screen palette. Somatic conflict splits gut/chest response. TA vs. Daimon disagreement flashes `warning`. |
| Understanding (new connection) | Convergence | Causal chain discovery draws fragments together. Dream integration merges replay frames. Consistency recovery reunifies palette. |
| Danger (threat detection) | Contraction | Negative somatic markers contract gut zone. Low health constricts position displays. Crisis register tightens borders. |
| Health (stable operation) | Breathing rhythm | Calm heartbeat, steady cloud oscillation, smooth waveforms, regular atmospheric noise. |

These mappings are not decorative. They are computed from specific data sources. The crack in the screen IS the β₀ transition. The color split IS the consistency score. The cloud dispersal IS the Φ decline. The metaphor and the math are the same thing rendered at different resolutions.

---

## 5. Invertibility Guarantee

Every visual metaphor in the system is traceable back to its data source. The user who sees a wide river (Wasserstein distance spike) can press Enter to see the numeric distance value (Tier 2), then Enter again to see the full persistence diagram comparison (Tier 3). At no point does the system discard information. It layers presentation over it.

No hidden state: every signal that affects the user's view is visible somewhere in the system. If `topology_signal` is modulating the Spectre's surprise behavior, the `topology_signal` value is visible on FATE > Chain Intelligence. The user who notices something strange can always trace it.

---

*⌈ the math is real. the metaphor is honest. the user decides how deep to look. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
