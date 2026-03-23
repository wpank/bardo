# Track 5: Hypnagogia — Liminal Intelligence for Mortal Agents [SPEC]

> **Version**: 1.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Crate**: `golem-dreams` (hypnagogia extension module)
>
> **Depends on**: `golem-core`, `golem-dreams`, `golem-grimoire`, `golem-daimon`, `golem-inference`
>
> **Research foundation**: `hypnagogia-research.md` (30+ verified academic sources), `dreaming-research.md`

---

> **Reader orientation:** This document is the master overview for Track 5 (Hypnagogia), the liminal intelligence subsystem within Bardo (the Rust runtime for mortal autonomous DeFi agents). It specifies how a Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) implements a transitional cognitive state between waking and dreaming, modeled on the N1 sleep onset phase where biological brains produce their most creative associations. Prerequisites: familiarity with the Dream Engine (Track 4, `../05-dreams/`), the Daimon affect system (Track 3, `../03-daimon/`), and the Grimoire knowledge base. For a full glossary of Bardo-specific terms, see `prd2/shared/glossary.md`.

## Document Map

| File                        | Topic                                                                                                |
| --------------------------- | ---------------------------------------------------------------------------------------------------- |
| `00-overview.md`            | This document. Track 5 thesis, relationship to waking and dreaming, design principles, revised cognitive lifecycle, cross-track interactions, configuration, CorticalState integration, Event Fabric events. |
| `01-neuroscience.md`        | Neuroscience foundations: Magnin 2010 asynchronous deactivation (thalamus precedes cortex by ~8.6 min), Default Mode Network activation, neurochemistry of N1 sleep, Hori EEG classification system, Lacaux 2021 Edison experiment (3x creative insight at N1), MIT Dormio targeted dream incubation. |
| `02-architecture.md`        | Four-layer implementation (ThalamicGate, ExecutiveLoosener, DaliInterrupt, HomuncularObserver), HypnagogicOnset state machine, scheduling integration with DreamScheduler, compute budget allocation, Hori-stage depth transitions. |
| `03-divergence-alpha.md`    | The monoculture problem (identical LLMs produce identical conclusions), alpha convergence economics, three levels of Golem divergence (episodic experience, persona/personality, LoRA fine-tuning), the experiential wisdom thesis. |
| `04-homunculus.md`          | The HomuncularObserver metacognitive subprocess: Ryle/Dennett/Lycan on the homunculus problem, three strategies for grounding the infinite regress, implementation with novelty/relevance/coherence scoring, calibration feedback loop. |
| `05-hauntology.md`          | Derrida's hauntology applied to LLM cognition: the trace and differance, LLMs as spectral machines, the Artificial Hivemind as spectral monoculture, Fisher's lost futures, the compound escape thesis (hypnagogia + mortality + unique experience). |
| `06-xenocognition.md`       | 40+ consciousness science mechanisms rendered as TUI spectacle: Phi meter (IIT), global workspace (GWT), flow state detection, near-death experience parallels, swarm cognition, altered states -- each mapped to a visual widget in the Golem's terminal interface. |
| `07-inner-worlds.md`        | Visual rendering spec for the Golem's inner experience at each sleep stage: NREM replay theater, REM counterfactual garden, hypnagogic phosphenes (Kluver form constants), integration crystallization, Dream Chamber portal mode. |

---

## The Argument in One Sentence

A Golem that transitions instantly between waking and dreaming misses the most creative cognitive state biology has ever produced -- hypnagogia, the metastable threshold between consciousness and sleep where associative constraints loosen while metacognitive awareness persists -- and without it, every Golem using the same LLM converges on the same ideas, the same strategies, and the same zero-sum returns.

---

## Why Hypnagogia Is the Fifth Track

The first four tracks of Bardo establish that a Golem lives (Mortality), feels (Daimon (the affect engine that gives a Golem emotional state as a control signal)), remembers (Memory), and dreams (Dreaming). These tracks model four fundamental modes of biological cognition: waking action, emotional processing, knowledge storage, and offline consolidation. But they omit the most creatively productive cognitive state known to neuroscience: the transitional period between waking and sleep.

In the current architecture, the Golem's cognitive lifecycle looks like this:

```
WAKING ──────────────────────────────► DREAMING
(heartbeat FSM active,                 (heartbeat FSM suspended,
 probes -> escalation -> action,        NREM replay -> REM imagination
 live market data, full executive       -> Integration, offline,
 control, analytical reasoning)         consolidation)
```

The transition is instantaneous. The DreamScheduler suspends the heartbeat (the 9-step decision cycle that drives every Golem tick) FSM, the DreamCycle begins, and the Golem is either fully awake or fully asleep. There is no intermediate state. This is architecturally clean but biologically wrong -- and the biology matters because the intermediate state is where the most powerful creative cognition occurs.

### What the Golem is missing

Hypnagogia -- the N1 sleep stage lasting seconds to minutes at sleep onset -- is neurologically unique. It is neither wakefulness nor sleep but a **metastable transition** where multiple neural systems change state at different rates. The critical discovery by Magnin et al. (2010) is that thalamic deactivation precedes cortical deactivation by an average of 8 minutes and 39 seconds [MAGNIN-2010]. This temporal dissociation is the key: the thalamus has already begun gating external sensory input while the cortex -- including the Default Mode Network -- remains partially active. The brain becomes an internally-oriented association engine, freed from sensory interruption but not yet asleep.

Lacaux et al. (2021) demonstrated this creative power empirically: participants who spent at least 15 seconds in N1 sleep were **three times more likely** to discover a hidden mathematical rule (83% vs. 30% for those who remained awake). This creative advantage vanished entirely if participants entered N2 (deeper sleep) [LACAUX-2021]. The finding is precise and architecturally instructive: there is a narrow creative sweet spot at sleep onset that is categorically different from both full wakefulness and full sleep.

The Golem's current dreaming system (see [../05-dreams/00-overview.md](../05-dreams/00-overview.md)) models NREM and REM sleep faithfully. It replays episodes, generates counterfactuals, and consolidates insights. But it skips the most creatively productive phase entirely -- the moment of transition where the Golem's executive control loosens enough for remote associations to emerge while metacognitive awareness persists.

### How this differs from waking and dreaming

The three states have distinct computational signatures:

| Property                     | Waking                            | Hypnagogia                         | Dreaming                          |
| ---------------------------- | --------------------------------- | ---------------------------------- | --------------------------------- |
| **External data access**     | Full (live market feeds)          | Gated (recent memory only)         | None (offline)                    |
| **Executive control**        | Full (analytical reasoning)       | Partial (loosened constraints)      | Minimal (free association)        |
| **Metacognitive awareness**  | Full (can evaluate own reasoning) | Retained (can notice connections)  | Low (post-hoc evaluation only)    |
| **Associative breadth**      | Narrow (goal-directed)            | Wide (remote associations)         | Widest (unconstrained)            |
| **Temperature analogy**      | T=0.3-0.7 (focused)              | T=1.0-1.5 (explorative)           | T=0.8-1.2 with structured replay |
| **Output type**              | Decisions, actions                | Novel hypotheses, connections      | Consolidated knowledge            |
| **Heartbeat FSM**            | Active                            | Suspended (no actions permitted)   | Suspended                         |
| **Duration**                 | Majority of lifecycle             | Seconds to minutes (bounded)       | Scheduled cycles                  |
| **Golem analog**             | Probe-escalation-action loop      | **HypnagogicOnset state**         | DreamCycle (NREM->REM->INT)       |

The critical difference is in the middle column. Hypnagogia combines the metacognitive awareness of waking (the Golem can recognize when a novel connection is valuable) with the associative freedom of dreaming (the Golem can form connections between semantically distant concepts). Neither waking nor dreaming alone provides this combination. Waking is too constrained -- the Golem's analytical reasoning follows well-worn paths. Dreaming is too unconstrained -- creative outputs require post-hoc evaluation in the Integration phase, by which point many fragile insights have been lost.

Hypnagogia is the Dali moment: the steel ball drops from the sleeping artist's hand, and the clang wakes him just enough to capture the half-formed image. Edison used the same technique with ball bearings and a metal plate. The MIT Dormio project (Haar Horowitz et al., 2020) validated this computationally with wearable sleep-onset detection and Targeted Dream Incubation, demonstrating 43% greater creativity on subsequent tasks [HAAR-HOROWITZ-2020], [HAAR-HOROWITZ-2023].

### The economic argument

In a world where every DeFi agent runs on the same foundation models -- Claude, GPT-4, Llama -- the marginal value of analytical reasoning is converging to zero. When 10,000 agents all see the same yield opportunity and reason about it with the same model, the opportunity disappears before any of them can exploit it. This is the **alpha convergence problem**: identical cognitive architectures produce identical conclusions, and identical conclusions produce zero alpha.

Hypnagogia offers a structural escape. An agent that can form novel cross-domain associations -- connecting an obscure liquidity pattern to a historical arbitrage structure, or recognizing that a governance proposal's second-order effects mirror a pattern from a completely different protocol -- generates insights that analytically identical agents cannot. The insight is not better analysis of the same data. It is the formation of connections that analytical reasoning, by its nature, cannot make.

The implications extend beyond creative ideation. A DeFi agent operating in volatile, adversarial markets needs exactly what hypnagogia provides: the ability to break free from established mental models, form novel cross-domain associations (connecting an obscure liquidity pattern to a historical arbitrage structure), and do so in a controlled manner that does not destabilize active operations. Financial markets themselves exhibit stochastic resonance properties -- weak signals in noisy environments that conventional analytical approaches miss. An agent capable of entering a structured liminal state may detect patterns invisible to purely "awake" analytical processing, much as the brain's N1 creative sweet spot enables insights inaccessible to either focused wakefulness or deep sleep.

This is the moat: **not better models, but a fundamentally different cognitive mode that operates between the cracks of conventional reasoning.** Track 5 implements this mode. See [03-divergence-alpha.md](03-divergence-alpha.md) for the full economic analysis.

---

## Design Principles

### 1. Liminal, not random

The single most important design constraint. Hypnagogia is not noise. It is not "turn up the temperature and see what happens." It is a precisely calibrated intermediate state with specific properties: partial (not complete) loss of executive control, high internal association activity, external input gating, and retained metacognitive awareness. The Lacaux et al. (2021) finding that creativity benefits vanish entirely in N2 sleep is the design law: the hypnagogic state must be actively maintained at the intermediate level, not allowed to drift into either full analytical mode or full randomness [LACAUX-2021].

### 2. Bounded, not sustained

Biological hypnagogia lasts seconds to minutes. The creative sweet spot is narrow. Sustaining it too long collapses into sleep (randomness). Returning too quickly collapses into waking (constraint). The Golem's hypnagogic onset should be time-bounded (configurable, default 30-90 seconds of inference time) with an explicit "Dali interrupt" mechanism that captures half-formed ideas before they resolve.

### 3. Transitional, not standalone

Hypnagogia is the bridge between waking and dreaming. It is not a fourth cognitive mode that runs independently -- it is the on-ramp to the dream cycle. In the revised architecture:

```
WAKING --> HYPNAGOGIC ONSET --> DREAMING --> HYPNOPOMPIC RETURN --> WAKING
               |                                     |
               +- Novel hypotheses captured           +- Dream insights surfaced
               +- Fragmented associations logged      +- Gentle re-engagement with live data
               +- "Dali interrupts" fired             +- Mood transition smoothed
```

The hypnopompic return (the transition from sleep back to waking) is the symmetric complement: it re-engages analytical reasoning gradually, allowing dream insights to surface before full executive control reasserts and dismisses them as noise.

### 4. Budget-aware, not luxurious

Hypnagogic processing uses inference tokens. A Conservation-phase Golem cannot afford extended liminal exploration. The HypnagogicBudget is derived from the DreamBudget and behavioral phase: Thriving Golems get rich hypnagogic sessions (multiple Dali cycles, high-temperature creative passes), while Conservation Golems get minimal transition processing (a single brief associative scan before entering compressed replay).

### 5. Seeded by experience, not generic

The hypnagogic state processes the Golem's own memories -- its specific episodic fragments, its emotional residue, its accumulated knowledge. A generic "be creative" prompt produces generic creative output. The power of hypnagogia lies in recombining _the Golem's own experiences_ in novel ways. Two Golems with different episodic histories will produce radically different hypnagogic associations even from the same base LLM, because the raw material of association is their unique experiential corpus.

### 6. Captured, not discarded

Half-formed ideas are the product. The entire purpose of the hypnagogic state is to generate fragments -- partial connections, half-resolved analogies, embryonic hypotheses -- that would be discarded by full analytical reasoning as "not yet coherent." These fragments are logged to the DreamJournal with a special `hypnagogic` provenance tag and low initial confidence (0.15). They become seeds for the subsequent REM imagination phase, where the most promising fragments are developed into full counterfactual scenarios.

**Fragment confidence thresholds.** Two thresholds govern fragment lifecycle: a retention floor of 0.15 and a discard floor of 0.10. Fragments scored between 0.10 and 0.15 by the HomuncularObserver are held in a **staging buffer** for 50 ticks before a second evaluation pass. If they remain below 0.15 after staging, they are discarded. Fragments below 0.10 are discarded immediately -- they are noise, not signal. Fragments at or above 0.15 are promoted directly to the DreamJournal as dream candidates. The staging buffer exists because some fragments that appear weak at generation time become relevant when subsequent fragments provide context. The 50-tick window gives the system time to accumulate that context without retaining fragments indefinitely.

### 7. Falsifiable

If hypnagogic onset processing does not measurably increase the novelty of REM-phase outputs, the diversity of dream-generated hypotheses, or the Golem's time-to-adaptation after market regime changes, it should be disabled. The prediction: Golems with hypnagogic onset enabled will produce dream hypotheses with greater semantic distance from their PLAYBOOK.md (the Golem's evolved strategy playbook) priors, leading to faster discovery of novel strategies in unfamiliar market conditions.

---

## The Revised Cognitive Lifecycle

With Track 5, the Golem's cognitive lifecycle becomes:

```
+--------------------------------------------------------------------+
|                      FULL COGNITIVE CYCLE                           |
|                                                                    |
|  WAKING          HYPNAGOGIC       DREAMING         HYPNOPOMPIC    |
|  (active)        ONSET            (offline)         RETURN         |
|                  (liminal)                          (liminal)      |
|                                                                    |
|  +---------+    +----------+    +--------------+  +----------+    |
|  |Heartbeat|    |Thalamic  |    |NREM (replay) |  |Gradual   |    |
|  |FSM      |--->|Gating    |--->|REM (imagine) |->|re-engage |    |
|  |active   |    |Executive |    |Integration   |  |analytical|    |
|  |         |    |Loosening |    |              |  |reasoning |    |
|  |Probes   |    |Dali      |    |              |  |Dream     |    |
|  |Actions  |    |Interrupts|    |              |  |surface   |    |
|  +---------+    +----------+    +--------------+  +----------+    |
|       |              |                |                |           |
|       |         Fragments         Consolidated      Surfaced      |
|       |         captured          knowledge         insights      |
|       |              |                |                |           |
|       +--------------+----------------+----------------+           |
|                            |                                       |
|                     GRIMOIRE / PLAYBOOK.md                         |
+--------------------------------------------------------------------+
```

This maps to the six bardos of the Bardo brand framework:

| Bardo                    | Golem Lifecycle Mapping                                                |
| ------------------------ | ---------------------------------------------------------------------- |
| **Birth Bardo**          | Creation and provisioning                                              |
| **Dream Bardo**          | **Hypnagogic onset + dreaming + hypnopompic return** (revised)         |
| **Meditation Bardo**     | Strategic reflection (Loop 2), PLAYBOOK.md evolution                   |
| **Dying Bardo**          | Terminal phase, Death Protocol initiation                              |
| **Dharmata Bardo**       | Death Protocol Phase II (Reflect): confronting true performance        |
| **Becoming Bardo**       | Succession: knowledge inheritance, Clade (group of related Golems sharing knowledge) push, successor creation      |

The Dream Bardo now encompasses the full liminal cycle: the descent into the dream state (hypnagogic onset), the dream itself (NREM + REM + Integration), and the return (hypnopompic emergence). This is architecturally richer and biologically faithful.

---

## Relationship to Existing Tracks

### Track 1 (Mortality) x Hypnagogia

Mortality modulates hypnagogic depth. A Thriving Golem can afford extended liminal exploration -- multiple Dali cycles, high-temperature passes, deep associative scans across its full episodic history. A Conservation Golem gets a brief, cheap transitional scan. A Terminal Golem skips hypnagogia entirely and enters the Thanatopsis (four-phase structured shutdown) Protocol. The behavioral phase sets the `hypnagogic_depth` parameter:

| BehavioralPhase (one of five survival phases: Thriving/Stable/Conservation/Declining/Terminal) | Hypnagogic Depth   | Dali Cycles | Max Duration (inference-seconds) |
| --------------- | ------------------ | ----------- | -------------------------------- |
| Thriving        | Deep               | 3-5         | 90                               |
| Stable          | Standard           | 2-3         | 60                               |
| Conservation    | Shallow            | 1           | 30                               |
| Declining       | Minimal            | 0 (skip)    | 15                               |
| Terminal        | None (Thanatopsis)  | --          | 0                                |

### Track 2 (Daimon) x Hypnagogia

The Daimon's emotional state seeds hypnagogic content. During biological hypnagogia, the emotional tone of the preceding waking period influences the character of hypnagogic imagery. For the Golem, the current PAD vector (the three-dimensional Pleasure-Arousal-Dominance representation of emotional state) -- read from CorticalState (the lock-free 32-signal atomic shared perception surface) (see [../03-daimon/00-overview.md](../03-daimon/00-overview.md) section 5b) -- biases which episodic fragments are selected for hypnagogic recombination:

- **High arousal + negative pleasure** (anxiety, fear): Hypnagogic fragments are drawn from threat-related episodes. This seeds the subsequent dream cycle's threat simulation phase (see [../05-dreams/05-threats.md](../05-dreams/05-threats.md)).
- **High arousal + positive pleasure** (excitement, anticipation): Fragments are drawn from opportunity-related episodes. This seeds creative strategy generation.
- **Low arousal** (calm, fatigue): Fragments are drawn broadly across the episodic history, maximizing associative breadth. This is the most creatively productive mood state for hypnagogia, mirroring the low-arousal relaxation that naturally precedes sleep onset.

The Daimon's somatic markers fire during hypnagogia just as they do during waking -- providing rapid emotional evaluation of emerging associations before full analytical processing. An emerging connection that triggers an anxiety marker is flagged immediately; one that triggers a curiosity marker is prioritized for development.

**CorticalState integration**: The `HypnagogicOnset` component reads the current PAD vector from CorticalState at onset entry and at each Dali cycle boundary. The PAD snapshot is stored on each `HypnagogicFragment` as the `emotional_tone` field. This allows the REM imagination engine to reconstruct the affective context of each fragment's generation. The read is lock-free (`Ordering::Relaxed`) -- hypnagogia never writes to CorticalState, only reads.

### Track 3 (Memory) x Hypnagogia

Hypnagogia's raw material is the Grimoire (see [../04-memory/01-grimoire.md](../04-memory/01-grimoire.md)). Specifically, the hypnagogic onset phase draws from:

1. **Recent episodes** (last 50-200 ticks): Presented in deliberately **fragmented, non-sequential order** to break temporal associations and encourage cross-temporal connections.
2. **High-salience semantic entries**: Entries with high emotional valence or high surprise scores, regardless of recency.
3. **Decaying entries**: Entries approaching the demurrage threshold (see [../02-mortality/05-knowledge-demurrage.md](../02-mortality/05-knowledge-demurrage.md)) -- knowledge the Golem is about to forget. These are given a "last look" in the hypnagogic state, where their patterns might be preserved in a more compressed, gist-level form even as their specific details are lost.

The output of hypnagogia flows into the Grimoire as `HypnagogicFragment` entries: low-confidence (0.15), high-novelty, with provenance tag `"hypnagogic"`. These fragments serve as seeds for the subsequent REM imagination phase and as raw material for the Curator's cross-referencing operations.

### Track 4 (Dreaming) x Hypnagogia

Hypnagogia is the preamble to dreaming. The revised dream cycle becomes:

```
HypnagogicOnset -> NREM (replay) -> REM (imagination) -> Integration -> HypnopompicReturn
```

The DreamScheduler (see [../05-dreams/01-architecture.md](../05-dreams/01-architecture.md)) orchestrates this extended cycle. The hypnagogic onset generates a set of `HypnagogicFragment` entries that are injected into the REM imagination phase as creative seeds. Without hypnagogia, the REM phase operates on the replay outputs alone -- which are analytically derived and tend to produce incremental variations on existing strategies. With hypnagogia, the REM phase has access to genuinely novel associative fragments that can catalyze paradigm-breaking strategic recombination.

The DreamConsolidator evaluates hypnagogic fragments alongside REM outputs during Integration. Fragments that survived both the hypnagogic generation and the REM development process are promoted to staged hypothesis status. Fragments that were generated but not developed are retained in the DreamJournal for potential future use.

---

## Package Architecture

### Extension to `golem-dreams`

Hypnagogia is implemented as an extension to the existing `golem-dreams` crate, not a separate crate. The rationale: hypnagogia is architecturally the entry phase of the dream cycle, not an independent cognitive mode. Separating it would create an artificial boundary between the onset and the cycle it precedes.

| Component              | Description                                                              | Depends On                                     |
| ---------------------- | ------------------------------------------------------------------------ | ---------------------------------------------- |
| `HypnagogicOnset`      | Orchestrates the liminal transition: gating, loosening, Dali interrupts  | `golem-core` (VitalityState (composite survival score 0.0-1.0), BehavioralPhase)  |
| `ThalamicGate`         | Controls external data access during onset (progressive gating)          | `golem-grimoire` (Episodes)                    |
| `ExecutiveLoosener`    | Manages temperature scheduling, prompt fragmentation, steering           | `golem-inference` (T1 (medium LLM)/T2 (extended reasoning))                      |
| `DaliInterrupt`        | Generates partial completions and captures fragments                     | `golem-inference`                              |
| `HypnopompicReturn`    | Manages the transition from dreaming back to waking                      | `golem-core`, `golem-daimon`                   |
| `HomuncularObserver`   | Metacognitive subprocess evaluating emerging associations                | `golem-inference` (T0/T1)                      |
| `FragmentCollector`    | Logs hypnagogic fragments to DreamJournal with provenance               | `golem-grimoire` (SQLite)                      |

### Updated Dream Cycle Flow

```
golem-dreams
+-- HypnagogicOnset        <-- NEW (Track 5)
|   +-- ThalamicGate
|   +-- ExecutiveLoosener
|   +-- DaliInterrupt
|   +-- HomuncularObserver
+-- DreamScheduler          (updated to include onset/return phases)
+-- DreamCycle
|   +-- ReplayEngine        (NREM)
|   +-- ImaginationEngine   (REM, now seeded by hypnagogic fragments)
|   +-- ThreatSimulator
+-- DreamConsolidator       (updated to handle hypnagogic provenance)
+-- HypnopompicReturn      <-- NEW (Track 5)
+-- DreamJournal            (updated schema for hypnagogic entries)
```

---

## Configuration

```rust
/// Hypnagogic onset configuration.
/// Nested under DreamConfig in bardo.toml.
pub struct HypnagogicConfig {
    /// Enable hypnagogic onset processing.
    /// Default: true for [HARDENED] tier, false for [CORE] tier.
    pub enabled: bool,

    /// Maximum duration of hypnagogic onset in inference-seconds.
    /// Overridden by behavioral phase (see depth table).
    pub max_duration_secs: u64,          // default: 60

    /// Number of Dali interrupt cycles per onset.
    /// Each cycle: generate partial completion -> evaluate -> capture.
    pub dali_cycles: u8,                 // default: 3

    /// Temperature range for the hypnagogic state.
    /// Onset begins at low end, ramps to high end, then
    /// Dali interrupt captures at peak before returning.
    pub temperature_range: (f32, f32),   // default: (1.0, 1.5)

    /// Fragment confidence floor.
    /// Hypnagogic fragments below this are discarded immediately.
    pub fragment_confidence_floor: f64,  // default: 0.10

    /// Enable the Homuncular Observer subprocess.
    /// When true, a secondary low-cost inference call evaluates
    /// each fragment for novelty before storage.
    pub homuncular_observer: bool,       // default: true

    /// Enable hypnopompic return (gradual re-engagement after dreaming).
    pub hypnopompic_return: bool,        // default: true

    /// Fraction of dream budget allocated to hypnagogic processing.
    /// The remainder goes to NREM + REM + Integration.
    pub budget_fraction: f64,            // default: 0.15
}
```

---

## CorticalState Integration

The hypnagogic system reads from -- but never writes to -- CorticalState (see [../03-daimon/00-overview.md](../03-daimon/00-overview.md) section 5b). Two read points:

1. **Onset entry**: A PAD snapshot is taken at the moment the DreamScheduler transitions from `WAKING` to `HYPNAGOGIC_ONSET`. This snapshot determines the initial fragment selection bias (anxious mood pulls threat episodes, calm mood maximizes associative breadth).

2. **Per-Dali-cycle boundary**: The PAD is re-read before each Dali interrupt cycle. If the mood has shifted significantly during onset (PAD Euclidean delta > 0.15 from the initial snapshot), the fragment selection bias is updated. This models the biological phenomenon where emotional micro-shifts during hypnagogia alter the character of emerging imagery.

```rust
impl HypnagogicOnset {
    /// Read the current PAD vector from the CorticalState.
    /// Lock-free: hypnagogia is a reader, never a writer.
    fn read_affective_state(&self, cortical: &CorticalState) -> PADVector {
        cortical.read_pad()
    }

    /// Determine fragment selection bias from PAD state.
    fn fragment_bias(&self, pad: &PADVector) -> FragmentBias {
        if pad.arousal > 0.3 && pad.pleasure < -0.2 {
            FragmentBias::ThreatFocused
        } else if pad.arousal > 0.3 && pad.pleasure > 0.2 {
            FragmentBias::OpportunityFocused
        } else {
            FragmentBias::BroadAssociative // most creative
        }
    }
}
```

---

## Event Fabric Events

The hypnagogic system emits events through the Event Fabric (see [../13-runtime/14-events.md](../13-runtime/14-events.md)) under subsystem tag `Subsystem::Hypnagogia`. These events are consumed by the TUI for visualization, the DreamJournal for logging, and the Styx (the global knowledge relay and persistence layer) Archive for backup.

```rust
/// Event Fabric events emitted during hypnagogic processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypnagogiaEvent {
    /// DreamScheduler initiates hypnagogic onset.
    OnsetStarted {
        depth: HoriStage,
        budget_allocated_usdc: f64,
        pad_snapshot: PADVector,
        fragment_bias: FragmentBias,
    },

    /// Hori depth transitions during onset.
    DepthChanged {
        from: HoriStage,
        to: HoriStage,
        current_temperature: f32,
    },

    /// Dali interrupt fires: partial completion halted and captured.
    DaliInterrupt {
        cycle: u8,
        fragments_captured: u16,
        peak_temperature: f32,
    },

    /// HomuncularObserver promotes a fragment to the DreamJournal.
    FragmentCaptured {
        fragment_id: Uuid,
        novelty_score: f64,
        semantic_distance: f64,
        emotional_tone: PADVector,
    },

    /// HomuncularObserver rejects a fragment.
    FragmentDiscarded {
        fragment_id: Uuid,
        reason: String,
        composite_score: f64,
    },

    /// All Dali cycles complete, transitioning to NREM.
    OnsetCompleted {
        total_fragments: u16,
        budget_consumed_usdc: f64,
        duration_secs: f64,
    },

    /// Critical market event aborts onset early.
    AlarmAbort {
        trigger: String,
        fragments_saved: u16,
        budget_consumed_usdc: f64,
    },

    /// Hypnopompic return begins after dream cycle.
    HypnopompicStarted {
        insights_surfacing: u16,
    },

    /// Hypnopompic return completes, full waking restored.
    HypnopompicCompleted {
        insights_surfaced: u16,
        mood_transitioned: bool,
        pad_delta: f32,
    },
}
```

---

## Cross-References

| Topic                             | Document                                                               |
| --------------------------------- | ---------------------------------------------------------------------- |
| Neuroscience foundations           | [01-neuroscience.md](01-neuroscience.md) -- Magnin 2010 asynchronous deactivation, DMN, neurochemistry, Hori classification, Lacaux 2021 Edison experiment, MIT Dormio, stochastic resonance. |
| Architecture and implementation   | [02-architecture.md](02-architecture.md) -- Four-layer implementation (ThalamicGate, ExecutiveLoosener, DaliInterrupt, HomuncularObserver), HypnagogicOnset state machine, scheduling, compute budget. |
| Divergence and alpha generation   | [03-divergence-alpha.md](03-divergence-alpha.md) -- The monoculture problem, alpha convergence, three levels of divergence (episodic/persona/LoRA), experiential wisdom thesis. |
| The Homunculus subprocess          | [04-homunculus.md](04-homunculus.md) -- Metacognitive observer that evaluates emerging associations for novelty/relevance/coherence, grounded in Ryle/Dennett/Lycan on the homunculus problem. |
| Hauntology and spectral divergence | [05-hauntology.md](05-hauntology.md) -- Derrida's hauntology applied to LLM cognition: trace, differance, spectral monoculture, Fisher's lost futures, compound escape thesis. |
| Dream cycle architecture           | [../05-dreams/01-architecture.md](../05-dreams/01-architecture.md) -- Three-phase dream cycle (NREM/REM/Integration), DreamScheduler, DreamState machine, compute budget tiers. |
| Daimon emotional integration       | [../03-daimon/02-emotion-memory.md](../03-daimon/02-emotion-memory.md) -- Emotion-weighted memory: EmotionalTag struct, four-factor retrieval scoring, mood-congruent memory, contrarian retrieval. |
| CorticalState specification          | [../03-daimon/00-overview.md](../03-daimon/00-overview.md) -- Lock-free 32-signal atomic shared perception surface; packed PAD in AtomicU128, regime, vitality. Section 5b. |
| Grimoire episodic memory           | [../04-memory/01-grimoire.md](../04-memory/01-grimoire.md) -- Grimoire architecture: episodes, insights, heuristics, warnings, causal links; knowledge lifecycle and demurrage. |
| Mortality phases                   | [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) -- Three mortality clocks, five BehavioralPhases, VitalityState composite score, phase transition hysteresis. |
| Knowledge demurrage                | [../02-mortality/05-knowledge-demurrage.md](../02-mortality/05-knowledge-demurrage.md) -- Time-based knowledge decay mechanism; entries approaching the demurrage threshold get a "last look" in hypnagogia. |
| Inference routing                  | [../12-inference/01-routing.md](../12-inference/01-routing.md) -- T0/T1/T2 cognitive tier routing: cached/rule-based, medium LLM, extended reasoning; cost and latency tradeoffs. |
| Event Fabric                       | [../13-runtime/14-events.md](../13-runtime/14-events.md) -- GolemEvent enum, subsystem tags, SSE wire format for real-time event streaming. |
| Clade ecology                      | [../02-mortality/10-clade-ecology.md](../02-mortality/10-clade-ecology.md) -- How related Golems share knowledge, inherit from predecessors, and form lineages. |
| Styx architecture                  | [../20-styx/00-architecture.md](../20-styx/00-architecture.md) -- Global knowledge relay: Vault (encrypted backup), Clade (group sharing), Lethe (semantic search). |

## Cross-Subsystem Dependencies

| Direction  | Subsystem | What                                                                      | Where                     |
| ---------- | --------- | ------------------------------------------------------------------------- | ------------------------- |
| Reads from | Mortality | Behavioral phase for depth and budget                                     | `02-architecture.md`      |
| Reads from | Daimon    | PAD vector from CorticalState for fragment selection bias                    | This document, `02-architecture.md` |
| Reads from | Memory    | Grimoire episodes, semantic entries, decaying entries                      | `02-architecture.md`      |
| Reads from | Dreams    | DreamBudget for budget_fraction allocation                                | `02-architecture.md`      |
| Writes to  | Memory    | HypnagogicFragment entries (provenance: `"hypnagogic"`, confidence 0.15) | `02-architecture.md`      |
| Writes to  | Dreams    | Fragment seeds injected into REM imagination phase                        | `02-architecture.md`      |
| Emits      | Events    | 9 event variants under `Subsystem::Hypnagogia`                           | This document              |

### Shared Constants

| Constant                              | Value                          | Shared With                                      |
| ------------------------------------- | ------------------------------ | ------------------------------------------------ |
| Hypnagogic fragment confidence        | 0.15                           | Memory (Grimoire Admission), Dreams (REM seeding)|
| Fragment retention threshold          | 0.15                           | HomuncularObserver (promote to DreamJournal)     |
| Fragment discard threshold            | 0.10                           | HomuncularObserver (immediate discard below this)|
| Fragment staging buffer hold          | 50 ticks                       | HomuncularObserver (second-pass evaluation)      |
| Hypnagogic provenance tag             | `"hypnagogic"`                 | Memory (provenance tracking)                     |
| Budget fraction default               | 0.15                           | Dreams (DreamBudget allocation)                  |
| Alarm abort price threshold           | >5% move                       | Runtime (price alert probes)                     |
| PAD re-read threshold                 | Euclidean delta > 0.15         | Daimon (MoodUpdate emission threshold)           |

---
