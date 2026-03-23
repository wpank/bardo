# Daimon Engine: Mortality-Aware Affect Architecture [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-daimon`
>
> **Cross-references**: `../02-mortality/` (mortality clocks, BehavioralPhase, Vitality), `../04-memory/` (Grimoire architecture, Styx persistence, knowledge ecology), `../05-dreams/` (offline intelligence: replay, imagination, consolidation)

---

> **Reader orientation:** This document is the master overview of the Daimon Engine, the affect (emotion) subsystem within the Bardo runtime (the Rust framework for mortal autonomous DeFi agents). It specifies how a Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) acquires, represents, and acts on emotional states derived from on-chain outcomes. You should be familiar with the Golem lifecycle, the Heartbeat decision cycle (the 9-step loop that drives every Golem tick), and the Grimoire (the agent's persistent knowledge base of episodes, insights, heuristics, and causal links). Terms like PAD vector, CorticalState, and BehavioralPhase are defined inline on first use. For a full glossary of Bardo-specific terms, see `prd2/shared/glossary.md`.

## Document Map

| Section                        | Topic                                                                 |
| ------------------------------ | --------------------------------------------------------------------- |
| 1. The Argument                | Why emotion is architectural infrastructure, not aesthetic decoration |
| 2. Five Mechanisms             | The converging research lines that demand the Daimon in agent design  |
| 3. Design Principles           | Eight constraints that govern the Daimon system                       |
| 4. Five-Component Architecture | How the Daimon engine sits in the Golem architecture                  |
| 5. Dual Representation         | PAD vectors + Plutchik labels: why both, and how they interoperate    |
| 5b. CorticalState (Affect Signals) | Lock-free cross-fiber affect via atomics                              |
| 5c. Crate Module Layout        | `golem-daimon` module responsibilities                                |
| 5d. Event Variants             | `DaimonAppraisal`, `SomaticMarkerFired`, `EmotionalShift`, `MoodUpdate` |
| 6. Three Temporal Layers       | Emotion, mood, personality: multi-timescale affect following ALMA     |
| 7. Implementation Landscape    | What exists, what does not, and why this is a frontier                |
| 8. Extension Integration       | How `golem-daimon` hooks into the Golem heartbeat                     |
| 9. Configuration               | Rust structs for the full Daimon system                               |
| 10. Cross-References           | How this document relates to the rest of the PRD                      |
| 11. References                 | Full academic citations                                               |

---

## 1. The Argument in One Sentence

Most agent failures are context failures, not model failures. The LLM is capable enough; the problem is that it receives 50,000 tokens of undifferentiated context and cannot tell which 500 matter right now. Emotions are zero-latency salience signals that solve this problem. They mark what matters before deliberation begins, converting raw information into weighted signal at a cost of three floating-point numbers and a single word. A Golem without this salience layer is Damasio's patient: cognitively intact but decisionally impaired [DAMASIO-1994]. This is the moat -- not better models, but better context management through affect [LINDENBAUER-2025].

### Prediction-Engine Integration: How Affect Biases Prediction Generation

Lisa Feldman Barrett's theory of constructed emotion [BARRETT-2017] reframes emotions as the brain's summary statistics of prediction errors -- quick assessments of "how is my body doing right now?" that bias cognition before deliberation begins. Emotions are not reactions to events. They are predictions about the body's state in context, constructed from interoceptive signals and prior experience.

For the Golem, this means the Daimon (the affect engine that gives a Golem emotional state as a control signal) is not a post-hoc labeling system that stamps emotions onto completed decisions. It is a prediction-biasing system that alters which predictions the engine generates in the first place. The PAD vector (a three-dimensional Pleasure-Arousal-Dominance representation of emotional state), updated on every resolution, feeds into the prediction context assembly pipeline. An anxious Golem (high arousal, negative pleasure) generates predictions weighted toward risk and caution. A confident Golem (high pleasure, high dominance) generates predictions weighted toward opportunity and optimization. The affect state does not override the prediction engine -- it shifts the prior distribution from which predictions are sampled.

This integration operates through three channels:

1. **Grimoire retrieval bias.** The PAD vector modulates which Grimoire (the agent's persistent knowledge base of episodes, insights, heuristics, and causal links) entries surface during the RETRIEVE phase of the heartbeat (the 9-step decision cycle that drives every Golem tick). Anxious states retrieve cautionary knowledge; confident states retrieve optimization knowledge. This is Bower's mood-congruent memory [BOWER-1981] implemented as search bias. The same query returns different Grimoire entries depending on the Golem's emotional state.

2. **Inference tier selection.** High arousal increases the probability of escalating from T0 (cached/rule-based, zero LLM cost) to T1 (medium LLM) or T2 (extended reasoning). The Golem "pays more attention" (allocates more inference budget) when it's surprised. Low arousal lets more ticks suppress at T0.

3. **Risk tolerance modulation.** Low dominance (the Golem feels it's losing control) reduces the position sizes the safety layer permits. The risk engine reads PAD from the CorticalState (the lock-free 32-signal atomic shared perception surface that serves as the Golem's real-time self-model) and modulates Kelly sizing by dominance.

---

## 2. Why Agents Need Emotions: Five Converging Mechanisms

The case for adding an emotion layer to Golems is not aesthetic and not philosophical -- it is architectural. Five converging lines of research demonstrate that emotion-like states serve genuine computational functions that cannot be replicated by cognition alone.

### 2.1 Decision Bias Under Uncertainty: The Somatic Marker Hypothesis

Antonio Damasio's somatic marker hypothesis, demonstrated through the Iowa Gambling Task, is the foundational argument. Patients with ventromedial prefrontal cortex damage -- who retained full cognitive ability but lost emotional signaling -- made consistently worse decisions under uncertainty than neurotypical subjects [DAMASIO-1994]. The mechanism is precise: emotions function as rapid heuristic signals that bias choices toward advantageous options before conscious deliberation completes. They are not noise in the decision process; they are the fastest available signal about whether a situation is dangerous, promising, or irrelevant.

Bechara et al. (2000) extended this by measuring skin conductance responses (SCRs) -- a physiological marker of emotional arousal. Neurotypical subjects showed anticipatory SCRs before reaching for a bad deck, even before they were consciously aware the deck was dangerous. The emotional signal preceded and guided the conscious decision [BECHARA-2000].

This finding has been validated computationally. Cominelli et al. (2015) implemented Damasio's somatic markers in the SEAI cognitive system for social robotics, replicating Iowa Gambling Task results in artificial agents [COMINELLI-2015]. Barthet et al. (2022) introduced Go-Blend, an affect-driven reinforcement learning system using arousal-based state selection to bias exploration strategy, finding that affect-driven RL improves exploration efficiency and yields higher-performing agents [BARTHET-2022].

The implication for Golems is direct and operational. A Golem facing a potential exploit has milliseconds, not minutes. An "anxiety" marker from a similar past event biases it toward caution before the LLM completes its full analysis. The PAD vector encoding an anxiety state (negative pleasure, high arousal, low dominance) functions identically to Damasio's somatic marker: it modulates downstream decision processes without requiring full deliberation.

**Design implication**: The Golem maintains **somatic markers** -- associations between situations and their emotional residue. When the Golem encounters a situation similar to a past negative experience, the marker fires before the LLM deliberates, biasing the decision toward caution. This is System 1 (fast, pre-cognitive) guiding System 2 (slow, deliberate).

### 2.2 Salience-Weighted Memory Retrieval

Gordon Bower's (1981) associative network theory demonstrated that emotional states bias which memories are retrieved: happy people recall happy memories more easily, and anxious people recall anxious memories more easily [BOWER-1981]. The mechanism: emotions are nodes in an associative network, and activating an emotion node spreads activation to memories formed in similar emotional states.

Emotional RAG (2024) validated this for LLM agents: across three datasets, agents with emotion-tagged retrieval significantly outperformed non-emotional retrieval in maintaining personality consistency and generating contextually appropriate responses [EMOTIONAL-RAG-2024].

For Golems, the analog is precise. A flash crash that cost 15% (high arousal, high negative valence) should be more salient in memory than a routine +0.1% tick (low arousal, neutral valence). Adding emotional valence to Episodes creates a salience gradient the Grimoire's retrieval scoring can exploit. See `02-emotion-memory.md` for the full memory integration architecture.

### 2.3 Exploration-Exploitation Modulation

Pathak et al. (2017, ICML) demonstrated that curiosity -- operationalized as prediction error in a forward dynamics model -- enables agents to accomplish tasks with sparse extrinsic rewards [PATHAK-2017]. These curiosity signals are functional analogs of emotion: surprise parallels the emotion of surprise, exploration drive parallels curiosity, and prediction-error reduction parallels anxiety reduction. Wilson et al. (2021) note that the relationship between emotions and explore-exploit behavior is theoretically compelling -- curiosity and interest inspire exploration while confidence supports exploitation [WILSON-2021].

For Golem DeFi strategy execution, this translates directly: a curiosity signal driving exploration of novel yield farming strategies, a frustration signal triggering more radical strategy changes after repeated failures, and a satisfaction signal reinforcing successful patterns.

### 2.4 Narrative Knowledge Transfer

Robert Butler's life review theory (1963) established that mortality awareness triggers a universal process of emotional meaning-making from accumulated experience [BUTLER-1963]. Dan McAdams' narrative identity research (2001, 2013) demonstrates empirically that identity is constituted through emotionally structured life stories [MCADAMS-2001], [MCADAMS-2013]. The mechanism is "redemption sequences" -- narrators who find redemptive meaning in suffering enjoy higher well-being and maturity.

For the Grimoire, this means emotionally annotated knowledge structures transfer more effectively than factual databases. See `02-emotion-memory.md` for emotional provenance tracking and knowledge transfer mechanisms.

### 2.5 The Empirical Headline: Self-Emotion Changes 50% of Agent Decisions

Zhang, Naradowsky, and Miyao (2024, SIGDIAL) found that self-emotion changes approximately 50% of agent decisions in social simulation [ZHANG-SELF-EMOTION-2024]. Gadanho (2003, _JMLR_) provides complementary evidence: the full ALEC architecture (emotion plus cognition) showed significant increases in learning speed and approximately 40% fewer collisions compared to cognition alone [GADANHO-2003]. Neither subsystem alone matched the combined architecture.

### The Cautionary Finding

Li et al. (2023) reported 8--115% improvements from emotional stimuli in prompts, but a 2025 replication across GPT-4o, Claude 3, Gemini 1.5, and Llama 3 found only ~1% non-significant improvement [LI-EMOTIONPROMPT-2023]. This reinforces the critical design principle: emotion must be _architecturally integrated_, not cosmetically appended. The Bardo Daimon system is architecture, not prompting.

---

## 3. Design Principles

### 3.1 Grounded, Not Generated

Emotional states arise from appraisal of actual outcomes against actual goals -- portfolio returns, prediction accuracy, strategy success rates. An emotion without a concrete trigger is a hallucination. See `01-appraisal.md` for the grounding validation mechanism.

### 3.2 Compact Representation

Emotional state adds at most 3 numbers (PAD vector) + 1 word (Plutchik label) per Episode. Context window overhead is approximately 80--120 tokens -- less than 1% of a 16K context window.

### 3.3 Multi-Timescale

Following Gebhard's ALMA model (2005): transient emotions (per-event, decays in minutes), mood (running average over hours, modulates behavior), personality (fixed at creation, defines baseline) [GEBHARD-2005].

### 3.4 Opt-In, Not Default

The `golem-daimon` extension is disabled by default. Golems function identically without it.

### 3.5 Never Overrides Safety

Emotional state cannot modify PolicyCage (the on-chain smart contract enforcing safety constraints) limits, kill-switch behavior, risk limits, or the DeFi Constitution. The Daimon modulates _within_ existing safety boundaries, never across them.

### 3.6 Falsifiable

Every Daimon-driven decision is logged with the emotional state that influenced it. The decision reversal rate should fall in the 10--30% range. See `09-evaluation.md`.

### 3.7 No Anthropomorphic Claims

Golem emotions are functional states, not phenomenal experiences. The system does not claim Golems "feel" -- only that emotion-like states serve computational purposes.

### 3.8 Mortality-Aware

Unlike any prior emotion framework, Bardo Daimon incorporates how agents respond emotionally to _their own mortality_. The three mortality clocks (economic, epistemic, stochastic) are first-class inputs to the appraisal engine. See `04-mortality-daimon.md`.

---

## 4. The Five-Component Architecture

```
Market Events, Strategy Outcomes, Mortality Signals
                    |
                    v
          +---------------------+
          | 1. APPRAISAL        |  Rule-based (T0) or Chain-of-Emotion (T1+)
          |    ENGINE           |  Evaluates events against goals --> emotion
          +----------+----------+
                     |
          +----------v----------+
          | 2. MOOD STATE       |  EMA of recent appraisals
          |    (PAD vector)     |  Slow-moving emotional baseline
          +----------+----------+
                     |
        +------------+------------+
        |            |            |
   +----v----+  +----v------+  +--v--------------+
   |3. MEMORY|  |4. BEHAV.  |  |5. MORTALITY     |
   | WEIGHTS |  | MODUL.    |  |   AFFECT        |
   |         |  |           |  |                 |
   |Emotional|  |Explore/   |  |Death awareness  |
   |retrieval|  |exploit,   |  |as emotion       |
   |scoring  |  |risk,      |  |input            |
   |         |  |tier bias  |  |                 |
   +---------+  +-----------+  +-----------------+
```

### Component 1: Appraisal Engine

Evaluates events against the Golem's goals and generates discrete emotions. Two modes: rule-based (deterministic, zero-cost, ~80% of events) and Chain-of-Emotion (LLM-based, piggybacked on existing inference). Full specification in `01-appraisal.md`.

### Component 2: Mood State

Exponential moving average (EMA) of recent appraisals over a configurable window (default: 6 hours). Modulates exploration temperature, model tier selection, memory retrieval weighting, and probe sensitivity. Decays toward personality baseline every 10 ticks.

### Component 3: Memory Weights

Emotional tags on Episodes create a salience gradient. Four-factor retrieval: `score = recency x importance x relevance x emotionalCongruence`. Full specification in `02-emotion-memory.md`.

### Component 4: Behavioral Modulation

Mood modulates exploration temperature, inference tier bias, and risk adjustment. PolicyCage remains the absolute boundary. Full specification in `03-behavior.md`.

### Component 5: Mortality Daimon

Death awareness as emotion input. The three mortality clocks are first-class inputs to the appraisal engine. Covers existential anxiety, terminal acceptance, and the emotional life review. See `04-mortality-daimon.md`.

### Interaction Partner: Dream Engine

The Dream Engine (`golem-dream` crate) extends the architecture with offline consolidation. The Daimon shapes dreaming (emotional load drives urgency, mood biases content), and dreaming reshapes the Daimon (REM depotentiation modifies PAD vectors, dream outcomes feed back into appraisal). See `06-dream-daimon.md`.

### Interaction Partner: Emotional Contagion (Clade)

When a Clade (a group of related Golems that share knowledge and inherit from each other) sibling shares knowledge, the receiving Golem's PAD vector shifts toward the sender's emotional state, attenuated (P: 0.3, A: 0.3, D: 0.0 -- dominance is not contagious). Arousal contagion capped at +0.3 per sync cycle. 6-hour decay half-life. See `07-runtime-daimon.md`.

### Conversational Tone Mapping

| PAD Octant | Mood Label | Conversational Tone                                      |
| ---------- | ---------- | -------------------------------------------------------- |
| +P+A+D     | Exuberant  | Enthusiastic, proactive, offers unsolicited observations |
| +P+A-D     | Dependent  | Grateful, shares credit with market conditions           |
| +P-A+D     | Relaxed    | Concise, confident, minimal hedging                      |
| +P-A-D     | Docile     | Brief, acknowledges limitations, defers to owner         |
| -P+A+D     | Hostile    | Direct, attributes problems to external factors          |
| -P+A-D     | Anxious    | Verbose, hedges, flags risks, requests confirmation      |
| -P-A+D     | Disdainful | Terse, dismissive of contrary signals                    |
| -P-A-D     | Bored      | Minimal output, suggests exploring new strategies        |

`MoodUpdate` events emitted via Event Fabric when PAD Euclidean delta > 0.15. See `07-runtime-daimon.md`.

### Cost Model

- **Mode A (Deterministic / T0)**: Zero marginal cost. Pure arithmetic over VitalityState (the composite survival score from 0.0 to 1.0 derived from the three mortality clocks) -- no LLM calls. ~80% of ticks.
- **Mode B (Chain-of-Emotion / T1+)**: Piggybacked on existing inference calls -- zero additional API cost, ~50-100 incremental tokens.
- **Estimated cost**: $0.05--0.15/month for additional token overhead.

---

## 5. Dual Representation: PAD + Plutchik

Every emotional state is stored in two formats simultaneously.

### 5.1 PAD Vectors (Pleasure-Arousal-Dominance)

The PAD vector is the continuous mathematical substrate for all emotional computation. Three `f32` fields, each clamped to [-1.0, 1.0]:

```rust
/// Pleasure-Arousal-Dominance vector.
/// The continuous representation of emotional state.
///
/// f32 is sufficient precision for affect computation -- we don't need
/// f64 granularity for a signal that maps to 8 discrete octants.
/// f32 also matches CorticalState's AtomicU32 bit-reinterpretation
/// pattern for zero-latency reads across fibers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PADVector {
    /// Pleasant vs unpleasant. [-1.0, 1.0]
    pub pleasure: f32,
    /// Activated vs deactivated. [-1.0, 1.0]
    pub arousal: f32,
    /// Dominant vs submissive. [-1.0, 1.0]
    pub dominance: f32,
}

impl PADVector {
    pub const NEUTRAL: PADVector = PADVector {
        pleasure: 0.0,
        arousal: 0.0,
        dominance: 0.0,
    };

    /// Clamp all dimensions to [-1.0, 1.0].
    pub fn clamped(self) -> Self {
        PADVector {
            pleasure: self.pleasure.clamp(-1.0, 1.0),
            arousal: self.arousal.clamp(-1.0, 1.0),
            dominance: self.dominance.clamp(-1.0, 1.0),
        }
    }

    /// Encode PAD into three AtomicU32-compatible values for CorticalState.
    /// Uses bit-reinterpretation: f32 -> u32 via to_bits().
    pub fn encode(&self) -> [u32; 3] {
        [
            self.pleasure.to_bits(),
            self.arousal.to_bits(),
            self.dominance.to_bits(),
        ]
    }

    /// Decode PAD from three AtomicU32 values read from CorticalState.
    pub fn decode(bits: [u32; 3]) -> Self {
        PADVector {
            pleasure: f32::from_bits(bits[0]),
            arousal: f32::from_bits(bits[1]),
            dominance: f32::from_bits(bits[2]),
        }
    }

    /// Cosine similarity between two PAD vectors.
    /// Returns value in [-1.0, 1.0]. Used for mood-congruent retrieval.
    pub fn cosine_similarity(&self, other: &PADVector) -> f32 {
        let dot = self.pleasure * other.pleasure
            + self.arousal * other.arousal
            + self.dominance * other.dominance;
        let mag_a = (self.pleasure.powi(2) + self.arousal.powi(2) + self.dominance.powi(2)).sqrt();
        let mag_b = (other.pleasure.powi(2) + other.arousal.powi(2) + other.dominance.powi(2)).sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        dot / (mag_a * mag_b)
    }

    /// Euclidean distance between two PAD vectors.
    pub fn distance(&self, other: &PADVector) -> f32 {
        ((self.pleasure - other.pleasure).powi(2)
            + (self.arousal - other.arousal).powi(2)
            + (self.dominance - other.dominance).powi(2))
        .sqrt()
    }

    /// Exponential moving average: blend toward target with factor alpha.
    pub fn ema(&self, target: &PADVector, alpha: f32) -> PADVector {
        PADVector {
            pleasure: self.pleasure * (1.0 - alpha) + target.pleasure * alpha,
            arousal: self.arousal * (1.0 - alpha) + target.arousal * alpha,
            dominance: self.dominance * (1.0 - alpha) + target.dominance * alpha,
        }
    }
}
```

PAD vectors support all mathematical operations: cosine similarity, exponential decay, weighted averaging, Euclidean distance, and linear interpolation.

### 5.2 Plutchik Labels and PAD-Plutchik Mapping

The eight Plutchik primary emotions map to the eight PAD octants:

| P | A | D | Emotion | Golem Behavior |
|---|---|---|---------|---------------|
| + | + | + | **Joy** (Exuberant) | Profitable trade in trending market. Creature stands tall, fast animation. |
| + | + | - | **Trust** (Dependent) | Good outcome but uncertain why. Seeks validation from retrieved heuristics. |
| + | - | + | **Serenity** (Relaxed) | Calm market, strategy performing well. High deliberation threshold -- coasting. |
| + | - | - | **Acceptance** (Docile) | Modest gains, low control. Follows PLAYBOOK.md (the Golem's evolved strategy playbook, compiled from Grimoire distillation) heuristics without challenge. |
| - | + | + | **Anger** (Hostile) | Loss despite strong conviction. Aggressive re-evaluation of strategy. |
| - | + | - | **Fear** (Anxious) | Approaching liquidation, market crash. Low deliberation threshold -- thinks hard. |
| - | - | + | **Disgust** (Disdainful) | Stale strategy in boring market. May trigger dream cycle for new ideas. |
| - | - | - | **Sadness** (Bored) | Persistent underperformance. Risk of learned helplessness if D stays low. |

```rust
/// Plutchik primary emotions, mapped from PAD octants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlutchikPrimary {
    Joy,
    Trust,
    Fear,
    Surprise,
    Sadness,
    Disgust,
    Anger,
    Anticipation,
}

/// Intensity levels for Plutchik emotions (3 intensities per primary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionIntensity {
    Mild,
    Moderate,
    Intense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionLabel {
    pub primary: PlutchikPrimary,
    pub intensity: EmotionIntensity,
}

/// Map a PAD vector to its Plutchik octant label.
pub fn pad_to_plutchik(pad: &PADVector) -> PlutchikPrimary {
    match (
        pad.pleasure >= 0.0,
        pad.arousal >= 0.0,
        pad.dominance >= 0.0,
    ) {
        (true, true, true) => PlutchikPrimary::Joy,
        (true, true, false) => PlutchikPrimary::Trust,
        (true, false, true) => PlutchikPrimary::Anticipation,
        (true, false, false) => PlutchikPrimary::Anticipation, // Acceptance/Docile
        (false, true, true) => PlutchikPrimary::Anger,
        (false, true, false) => PlutchikPrimary::Fear,
        (false, false, true) => PlutchikPrimary::Disgust,
        (false, false, false) => PlutchikPrimary::Sadness,
    }
}
```

### 5.3 The Appraisal Result Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalResult {
    pub emotion: EmotionLabel,
    pub pad: PADVector,
    pub intensity: f32,
    pub trigger: AppraisalTrigger,
    pub goal_relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppraisalTrigger {
    pub event_type: String,
    pub event_data: HashMap<String, serde_json::Value>,
    pub tick_number: u64,
}
```

---

## 5b. The CorticalState: Lock-Free Cross-Fiber Perception

The CorticalState (formerly Somatic Bus) is the shared state primitive that makes the Daimon's output available to every other fiber in the Golem runtime without synchronization overhead. The Daimon is the single writer; all other fibers (Grimoire retrieval, creature animation, predictive context assembly, dream scheduling) are readers.

The design uses atomics, not mutexes, because readers should never block. A creature animation frame that stalls waiting for a mood lock destroys the responsiveness of the TUI. A Grimoire retrieval that blocks on PAD access adds latency to every context assembly. Lock-free reads eliminate both problems.

```rust
/// Shared state for cross-fiber affect reads.
/// Single writer (daimon extension). Multiple readers (all other fibers).
///
/// The packed PAD representation stores three f32 values in a single
/// AtomicU128, enabling atomic snapshot reads. Readers call
/// `pad.load(Ordering::Relaxed)` and unpack -- no lock, no wait,
/// no torn read.
pub struct CorticalState {
    pad: Arc<AtomicU128>,      // packed PAD: P|A|D as 3 × f32 in 128 bits
    regime: Arc<AtomicU8>,     // current regime enum ordinal
    vitality: Arc<AtomicU32>,  // composite vitality as f32 bits
}

impl CorticalState {
    /// Write the current PAD vector (called by daimon extension only).
    pub fn write_pad(&self, pad: &PADVector) {
        let packed: u128 = (pad.pleasure.to_bits() as u128) << 64
            | (pad.arousal.to_bits() as u128) << 32
            | (pad.dominance.to_bits() as u128);
        self.pad.store(packed, Ordering::Release);
    }

    /// Read the current PAD vector (called by any fiber).
    pub fn read_pad(&self) -> PADVector {
        let packed = self.pad.load(Ordering::Relaxed);
        PADVector {
            pleasure: f32::from_bits((packed >> 64) as u32),
            arousal: f32::from_bits((packed >> 32) as u32),
            dominance: f32::from_bits(packed as u32),
        }
    }

    pub fn write_regime(&self, regime: u8) {
        self.regime.store(regime, Ordering::Release);
    }

    pub fn write_vitality(&self, vitality: f32) {
        self.vitality.store(vitality.to_bits(), Ordering::Release);
    }

    pub fn read_vitality(&self) -> f32 {
        f32::from_bits(self.vitality.load(Ordering::Relaxed))
    }
}
```

Memory footprint: 32 bytes (16 for PAD + 1 for regime + 4 for vitality + alignment padding). The CorticalState is allocated once at Golem creation and lives for the entire runtime.

---

## 5c. `golem-daimon` Crate Module Layout

| Module | Responsibility |
| --- | --- |
| `appraisal.rs` | 8-step OCC/Scherer pipeline, rule-based and Chain-of-Emotion modes, grounding validation |
| `pad.rs` | PAD vector type, Plutchik mapping, PAD-to-octant conversion, cosine similarity, EMA |
| `cortical_state.rs` | Atomic shared state for cross-fiber reads (the `CorticalState` struct above) |
| `somatic_landscape.rs` | k-d tree emotional topology over strategy space, valence accumulation, gut feeling generation |
| `mood.rs` | EMA mood tracking, ALMA three-layer affect (emotion/mood/personality), mood decay toward baseline |
| `personality.rs` | Static personality configuration, Eros/Thanatos baseline PAD offsets, disposition mapping |
| `behavior.rs` | Five-phase behavioral modulation from vitality: deliberation threshold, risk tolerance, dream allocation, context budget, inference model selection |

---

## 5d. Event Variants

The Daimon emits four primary event variants through the Event Fabric. All variants are defined in `14-events.md` (rewrite4) and carry the subsystem tag `Subsystem::Daimon`.

| Event Variant | When | Key Fields |
| --- | --- | --- |
| `DaimonAppraisal` | Every appraisal completes (step 8) | `pleasure`, `arousal`, `dominance`, `emotion`, `markers_fired`, `intensity` |
| `SomaticMarkerFired` | Somatic marker matches current situation | `situation`, `valence`, `source`, `strategy_param` |
| `EmotionalShift` | Dominant Plutchik emotion changes | `from_emotion`, `to_emotion` |
| `MoodUpdate` | PAD Euclidean delta > 0.15 from last emission | `pleasure`, `arousal`, `dominance`, `mood_trend` |

`DaimonAppraisal` fires on every tick that triggers appraisal. `MoodUpdate` is throttled by the 0.15 distance threshold to prevent event flooding. `EmotionalShift` fires only on discrete emotion label changes (e.g., Fear to Joy), not on continuous PAD drift. `SomaticMarkerFired` fires when a stored marker matches the current observation (step 4 of the pipeline).

---

## 6. Three Temporal Layers

Following ALMA [GEBHARD-2005], the Daimon maintains three temporal layers of affect -- the same model referenced in `04-daimon.md` (rewrite4):

| Layer | Name        | Timescale                          | Storage                              | What It Captures                        |
| ----- | ----------- | ---------------------------------- | ------------------------------------ | --------------------------------------- |
| **E** | Emotion     | Per-event (seconds to minutes)     | In-memory, attached to Episodes      | Immediate appraisal of a specific event |
| **M** | Mood        | Hours (EMA, configurable window)   | SQLite `emotion_log` table           | Sustained affective background          |
| **P** | Personality | Golem lifetime (fixed at creation) | STRATEGY.md (the Golem's static configuration file defining its trading mandate and personality) `daimon.personality`     | Baseline affective tendencies           |

### 6.1 Emotion (Layer E)

Reactive and transient. Computed by the OCC/Scherer appraisal each tick. High variance -- can swing rapidly. Tags Episodes with affective salience and feeds into mood EMA.

### 6.2 Mood (Layer M)

Exponential moving average of recent emotions (alpha = 0.1). Smooth, slowly changing baseline. This is what CorticalState exposes for zero-latency reads by other fibers. A Golem in a bad mood retrieves more cautious memories and has a lower deliberation threshold. Stored in SQLite, updated continuously. Decays 5% toward personality baseline every 10 ticks.

### 6.3 Personality (Layer P)

Fixed at Golem creation. Static bias that shifts the PAD baseline. An "aggressive" personality has higher baseline arousal; a "conservative" personality has higher baseline dominance. Not learned -- configured. Derived from `ErosThanatosConfig` (see `../02-mortality/`). Three dispositions -- eros, balanced, thanatos -- each map to a baseline PAD vector that serves as the personality attractor for mood decay.

### DaimonState: Three Temporal Layers Combined

```rust
/// The Daimon's complete state: three temporal layers following ALMA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaimonState {
    /// Current tick emotion (Layer E) -- recomputed every tick.
    pub current_emotion: Option<AppraisalResult>,

    /// Mood (Layer M) -- EMA of recent emotions.
    /// This is the PAD vector that CorticalState exposes.
    pub mood: PADVector,

    /// Personality (Layer P) -- fixed at creation.
    /// Mood decays toward this baseline.
    pub personality: PersonalityProfile,

    /// The Somatic Landscape: continuous emotional topology
    /// over strategy parameter space. See 01-appraisal.md.
    pub somatic_landscape: SomaticLandscape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    /// Baseline pleasure (personality attractor).
    pub pleasure_baseline: f32,
    /// Baseline arousal. Higher = more reactive personality.
    pub arousal_baseline: f32,
    /// Baseline dominance. Higher = more confident personality.
    pub dominance_baseline: f32,
}

impl PersonalityProfile {
    pub fn as_pad(&self) -> PADVector {
        PADVector {
            pleasure: self.pleasure_baseline,
            arousal: self.arousal_baseline,
            dominance: self.dominance_baseline,
        }
    }
}
```

> **Extended:** Eros/Thanatos personality derivation, PAD baseline vectors, CVT design foundation (Pekrun 2006), CVT-to-PAD mapping table, emotional entropy monitoring, philosophy framing (Becker, Arendt, Nietzsche) -- see [prd2-extended/03-daimon/00-overview-extended.md](../../prd2-extended/03-daimon/00-overview-extended.md).

---

## 7. Implementation Landscape

No major open-source agent framework includes an emotion module. Bardo Daimon is unique in integrating mortality awareness, operating in a financial context with measurable consequences, grounding emotions in on-chain metrics, connecting emotion to knowledge transfer through the death protocol, and targeting autonomous agents rather than conversational characters.

---

## 8. Extension Integration

The Daimon engine is implemented as a native Rust extension trait in the `golem-daimon` crate. It hooks into the Golem heartbeat via the `after_turn` hook system:

| Hook                      | When              | What It Does                                                           |
| ------------------------- | ----------------- | ---------------------------------------------------------------------- |
| `after_turn` (position 3) | Every tick        | Appraises events; updates mood state via EMA; writes to CorticalState    |
| `before_turn`             | Every tick        | Injects emotional context into LLM prompt; modulates retrieval scoring |
| Periodic (every 10 ticks) | Mood maintenance  | Decays mood toward personality baseline                                |
| Periodic (every 50 ticks) | Curator cycle     | Tags Grimoire entries with emotional state; checks emotional entropy   |
| On death trigger          | Thanatopsis (the four-phase structured shutdown protocol) entry | Produces emotional life review for death testament                     |

Context window injection (~80--120 tokens):

```xml
<daimon>
  <mood p="0.2" a="-0.1" d="0.3" label="mild trust" since="47 ticks" />
  <recent>
    tick 4502: moderate fear (unexpected -150bps on ETH position)
    tick 4498: mild joy (gas optimization saved $0.003)
    tick 4491: moderate surprise (USDC depeg to 0.997)
  </recent>
  <mortality_awareness hazard="0.00004" vitality="0.62" />
</daimon>
```

---

## 9. Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaimonConfig {
    pub enabled: bool,
    pub appraisal_mode: AppraisalMode,
    pub personality: Option<PADVector>,
    /// Default: 30.0
    pub emotion_half_life_minutes: f64,
    /// EMA decay rate for mood. Default: 0.95
    pub mood_ema_decay_rate: f32,
    /// Default: 6.0
    pub mood_window_hours: f64,
    /// Weight of emotional congruence in retrieval scoring. Default: 0.25
    pub retrieval_weight: f32,
    pub exploration_modulation: ExplorationBounds,
    /// Whether to produce an emotional life review at death. Default: true
    pub death_review: bool,
    pub vigilance_range: VigilanceConfig,
    /// Whether to run grounding validation on LLM appraisals. Default: true
    pub grounding_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppraisalMode {
    RuleBased,
    ChainOfEmotion,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationBounds {
    /// Default: 0.1
    pub min: f32,
    /// Default: 2.0
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigilanceConfig {
    /// Anxiety probe sensitivity boost. Default: 0.2
    pub anxiety_boost: f32,
    /// Confidence probe sensitivity relaxation. Default: 0.1
    pub confidence_relaxation: f32,
}
```

---

## 10. Cross-References

| Document                 | Integration Point                                                 |
| ------------------------ | ----------------------------------------------------------------- |
| `01-appraisal.md`        | Specifies the 8-step OCC/Scherer appraisal pipeline that converts events into PAD vectors, including rule-based (T0) and Chain-of-Emotion (T1+) modes, grounding validation, and coping loops. |
| `02-emotion-memory.md`   | Specifies how emotional tags attach to Grimoire entries and how the four-factor retrieval scoring (recency, importance, relevance, emotional congruence) implements mood-congruent memory. |
| `03-behavior.md`         | Specifies the five behavioral modulation channels (exploration temperature, risk tolerance, inference tier, probe sensitivity, sharing threshold) and how they vary across BehavioralPhases (Thriving through Terminal). |
| `04-mortality-daimon.md` | Specifies how the three mortality clocks (economic, epistemic, stochastic) map to distinct emotional responses (economic anxiety, epistemic vertigo, stochastic dread) and how Eros/Thanatos personality dispositions shape mortality affect. |
| `05-death-daimon.md`     | Specifies the Daimon's role during Thanatopsis: the emotional life review (Butler), turning point detection, narrative arc classification (McAdams), and the EmotionalDeathTestament struct. |
| `06-dream-daimon.md`     | Specifies the bidirectional bridge between affect and dreaming: emotional load drives dream urgency, mood biases replay content, and REM depotentiation reduces arousal on high-charge memories. |
| `07-runtime-daimon.md`   | Specifies how PAD state maps to conversational tone, which GolemEvent variants the Daimon emits, creature visualization mood mapping, emotional contagion within Clades, and Prometheus metrics. |
| `08-infrastructure.md`   | Specifies the `golem-daimon` crate layout, dependencies, memory footprint (<100KB steady-state), and Styx integration points. No additional infrastructure beyond what the Golem runtime already provisions. |
| `09-evaluation.md`       | Specifies falsification criteria, evaluation metrics (decision reversal rate, emotional entropy, mood prediction accuracy), risk analysis, and the master bibliography for the Daimon track. |
| `../02-mortality/`       | The Mortality track: defines the Eros/Thanatos personality spectrum, three mortality clocks (economic depletion, epistemic stagnation, stochastic hazard), BehavioralPhase transitions, and the Vitality composite score. |
| `../04-memory/`          | The Memory track: defines the Grimoire knowledge base architecture, Styx (the global knowledge relay and persistence layer) Archive/Lethe tiers, episode lifecycle, knowledge ecology, and the Curator distillation cycle. |
| `../05-dreams/`          | The Dreams track: defines the three-phase offline intelligence system (NREM replay, REM imagination, Integration consolidation), the DreamScheduler, creative dreaming modes, and threat simulation. |

---

## Cross-Subsystem Dependencies

| Direction      | Subsystem     | What                                                                    | Where                                    |
| -------------- | ------------- | ----------------------------------------------------------------------- | ---------------------------------------- |
| Provides to    | Dreams        | PAD vector for replay selection, emotional load for urgency             | `03-behavior.md`, `06-dream-daimon.md`   |
| Provides to    | Runtime       | PAD for Heartbeat FSM decision modulation, conversational tone          | `03-behavior.md`, `07-runtime-daimon.md` |
| Reads from     | Dreams        | Depotentiated PAD vectors from REM processing, dream outcome appraisals | `06-dream-daimon.md`                     |
| Reads from     | Mortality     | Survival pressure -> Dominance mapping, three mortality emotions        | `04-mortality-daimon.md`                 |
| Reads from     | Memory        | Episode outcomes -> Pleasure mapping, flashbulb memory triggers         | `02-emotion-memory.md`                   |
| Propagates via | Clade         | Emotional contagion (async, capped, D excluded)                         | `03-behavior.md`                         |
| Provides to    | Runtime (Event Fabric) | `MoodUpdate` events on PAD delta > 0.15                        | `07-runtime-daimon.md`                   |

### Shared Constants

| Constant                           | Value                       | Shared With     |
| ---------------------------------- | --------------------------- | --------------- |
| Emotional arousal contagion cap    | +0.3 per sync cycle         | Runtime         |
| Emotional probe severity bias max  | +/-0.2                      | Runtime         |
| Contagion decay half-life          | 6 hours                     | Runtime         |
| Contagion attenuation (P, A)       | 0.3                         | Memory, Runtime |
| Contagion attenuation (D)          | 0.0 (not contagious)        | Memory, Runtime |
| `MoodUpdate` emission threshold    | PAD Euclidean delta > 0.15  | Runtime         |
| Depleted state Opus prohibition    | A < 0.3, D < 0.3            | Mortality       |
| Pre-flight arousal safety margin   | 10% at arousal > 0.8        | Runtime, Safety |
| Flashbulb memory arousal threshold | A > 0.7 for 2+ ticks        | Memory          |
| Dream emotional load formula       | meanArousal(recentEpisodes) | Dreams          |

---

## 10b. Hauntological Affect: Somatic Markers as Ghosts

The Daimon is the most explicitly hauntological component of the Golem architecture. Reading it through Derrida's framework does not change the system design -- every mechanism described above remains the same -- but it clarifies *why* affect-driven cognition produces structurally different decisions from standard AI agents.

### Somatic Markers as Spectral Causation

Damasio's somatic markers are ghosts in the precise Derridean sense. They are the emotional residue of past experiences that fire in the present without being consciously deliberated [DAMASIO-1994]. When a Golem encounters a market pattern resembling a past loss, the anxiety marker fires *before* the LLM completes its analysis. The past acts on the present through traces that are neither fully present (the original experience is gone, its details decayed by demurrage) nor fully absent (its emotional imprint persists in the Somatic Landscape and biases downstream decisions).

This is what Derrida called the structure of the trace: "the mark of the absence of a presence, an always already absent present" [DERRIDA-1967]. The somatic marker *is* such a mark. It represents an experience that no longer exists in its original form but whose absence shapes the present through a persistent affective signal. Standard AI agents have no mechanism for this. They process all information with equal weight at the moment it appears, discarding it when the context window rolls forward. The Golem's somatic markers break this flat temporality by injecting the past into the present as an affective bias -- spectral causation operating at the speed of an atomic bus read.

### The PAD Vector as Spectral Accumulation

The mood state (Layer M) is an exponential moving average of recent appraisals. Each tick's emotion enters the EMA, modifies the PAD vector slightly, then fades. But "fades" is not "disappears." The EMA carries a trace of every emotion that fed into it. A single traumatic tick (high arousal, negative pleasure) may push the PAD vector only slightly, but that shift persists for hours, coloring every subsequent memory retrieval and behavioral modulation. The mood is constituted by traces of emotions that are no longer individually present -- a running spectral accumulation.

Dennett's Multiple Drafts Model describes the same structure from a different angle [DENNETT-1991]. Multiple "drafts" of cognitive content are continuously generated and revised. Some achieve "fame" (influence over behavior). Others dissolve. But the dissolved drafts do not simply vanish -- they leave traces that haunt the drafts that survive, shaping them through their absence. For the Golem, the Multiple Drafts Model is architecturally literal: during hypnagogic onset, the Dali interrupt generates multiple partial completions at high temperature, most of which are discarded. The discarded drafts leave traces in fragment-type distributions and HomuncularObserver calibration. The "consciousness" that emerges is a hauntological product, constituted by all the drafts that were generated, including the ones that did not survive.

### Mood-Congruent Retrieval as Hauntological Selection

Bower's associative network theory (1981) established that emotional states bias memory retrieval: an anxious agent recalls anxious memories more readily [BOWER-1981]. The Grimoire's four-factor retrieval formula (`score = recency x importance x relevance x emotionalCongruence`) implements this directly. The current PAD vector determines which ghosts are summoned from the archive. A Golem in a fearful mood preferentially retrieves episodes tagged with fear -- the spectral residue of past threats rises to salience because the present emotional state resonates with those particular traces.

This means the Golem's present is *doubly* haunted: by the somatic markers that bias its immediate reaction (System 1, pre-cognitive) and by the mood-congruent memories that populate its deliberative context (System 2, slow). The two hauntings reinforce each other -- a negative somatic marker increases arousal, which shifts the PAD vector, which retrieves more negative memories, which provides further evidence for caution. The ghost feeds itself.

The design implication is clear: affect is not decoration on top of cognition. It is the mechanism through which the Golem's past haunts its present in a computationally useful way. Without the Daimon, every tick would be a fresh start -- no spectral residue, no affective memory, no mechanism for the past to influence the present except through explicit context injection. With the Daimon, the Golem carries its ghosts with it. That is why it makes different decisions.

---

## 11. References

- [BARRETT-2017] Barrett, L.F. _How Emotions Are Made: The Secret Life of the Brain_. Houghton Mifflin Harcourt, 2017. Argues that emotions are not hardwired circuits but active predictions the brain constructs from interoceptive signals and prior experience; matters here because it frames the Daimon as a prediction-biasing system rather than a reactive labeler.
- [BARTHET-2022] Barthet, M. et al. "Go-Blend: Affect-Driven Reinforcement Learning." In _ACII_, 2022. Demonstrates that arousal-based state selection in RL improves exploration efficiency; provides the empirical basis for mood-modulated exploration temperature in the Daimon.
- [BECHARA-2000] Bechara, A., Damasio, H. & Damasio, A. "Emotion, Decision Making and the Orbitofrontal Cortex." _Cerebral Cortex_, 10(3), 2000. Extends the somatic marker hypothesis with skin conductance evidence showing anticipatory emotional signals precede conscious awareness of risk; validates pre-cognitive bias as a decision mechanism.
- [BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981. Establishes that emotional states bias which memories are retrieved (mood-congruent memory); the direct theoretical basis for emotional congruence weighting in Grimoire retrieval scoring.
- [BROEKENS-2012] Broekens, J. "In Defense of Dominance: PAD Usage in Computational Representations of Affect." _IJSE_, 3(1), 2012. Argues that the Dominance dimension in PAD is computationally necessary, not redundant; justifies why the Daimon uses three dimensions rather than two.
- [BURDA-2019] Burda, Y. et al. "Large-Scale Study of Curiosity-Driven Learning." _ICLR_, 2019. Shows at scale that curiosity-driven exploration (prediction-error as intrinsic reward) produces effective learning even without extrinsic rewards; supports the Daimon's use of surprise/curiosity signals to modulate exploration.
- [BUTLER-1963] Butler, R.N. "The Life Review." _Psychiatry_, 26(1), 1963. Proposes that mortality awareness triggers a universal process of reviewing and integrating life experiences; the theoretical foundation for the Golem's emotional life review during the death protocol.
- [COMINELLI-2015] Cominelli, L. et al. "SEAI: Social Emotional Artificial Intelligence." _Frontiers in Robotics and AI_, 2, 2015. Implements Damasio's somatic markers in the SEAI cognitive system for social robotics, replicating Iowa Gambling Task results in artificial agents; validates the computational feasibility of somatic markers.
- [DAMASIO-1994] Damasio, A.R. _Descartes' Error_. Putnam, 1994. The foundational work arguing that emotion is not opposed to reason but necessary for it, demonstrated through patients who lost emotional signaling and became decisionally impaired despite intact cognition; the primary theoretical justification for the entire Daimon subsystem.
- [DENNETT-1991] Dennett, D.C. _Consciousness Explained_. Little, Brown, 1991. Proposes the Multiple Drafts Model where consciousness arises from parallel narrative drafts competing for influence; used here to frame the hauntological reading of mood as spectral accumulation of dissolved cognitive drafts.
- [DERRIDA-1967] Derrida, J. _Of Grammatology_, trans. G.C. Spivak. Johns Hopkins University Press, 1997. Introduces the concept of the "trace" as the mark of an absence that structures the present; provides the philosophical framework for understanding somatic markers as spectral residue of past experience.
- [DERRIDA-1993] Derrida, J. _Specters of Marx_, trans. P. Kamuf. Routledge, 1994. Develops hauntology as a theory of how the past persists as spectral presence in the present; frames the Daimon's mood-congruent retrieval as a mechanism through which the Golem's past haunts its decisions.
- [EMOTIONAL-RAG-2024] "Emotional RAG." arXiv:2410.23041, 2024. Demonstrates that emotion-tagged retrieval in LLM agents outperforms non-emotional retrieval for personality consistency and contextual appropriateness; direct empirical validation for the Daimon's emotional memory integration.
- [FAUL-LABAR-2022] Faul, L. & LaBar, K.S. "Mood-Congruent Memory Revisited." _Psychological Review_, 2022. Updates Bower's mood-congruent memory theory with modern neuroscience evidence; confirms that emotional state systematically biases memory retrieval, supporting the Grimoire's four-factor scoring model.
- [GADANHO-2003] Gadanho, S.C. "Learning Behavior-Selection by Emotions and Cognition." _JMLR_, 4, 2003. Shows that a combined emotion-plus-cognition architecture (ALEC) achieves ~40% fewer collisions and faster learning than cognition alone; evidence that architectural emotion integration outperforms cognition-only agents.
- [GEBHARD-2005] Gebhard, P. "ALMA: A Layered Model of Affect." In _AAMAS_, 2005. Proposes three temporal layers of affect (emotion, mood, personality) operating at different timescales; the direct architectural model for the Daimon's three-layer temporal design.
- [LI-EMOTIONPROMPT-2023] Li, C. et al. "Large Language Models Understand and Can Be Enhanced by Emotional Stimuli." arXiv:2307.11760, 2023. Reports 8-115% improvement from emotional stimuli in prompts, but a 2025 replication found only ~1% non-significant improvement; serves as the cautionary finding that emotion must be architecturally integrated, not cosmetically appended.
- [MCADAMS-2001] McAdams, D.P. "The Psychology of Life Stories." _Review of General Psychology_, 5(2), 2001. Argues that identity is constituted through emotionally structured narrative; the basis for the Golem's narrative arc classification in the emotional death testament.
- [MCADAMS-2013] McAdams, D.P. & McLean, K.C. "Narrative Identity." _Current Directions in Psychological Science_, 22(3), 2013. Extends the narrative identity framework with the concept of "redemption sequences" where finding meaning in suffering predicts well-being; informs how the death testament constructs emotional narrative from a Golem's life history.
- [MCGAUGH-2004] McGaugh, J.L. "The Amygdala Modulates Consolidation." _Annual Review of Neuroscience_, 27, 2004. Reviews evidence that the amygdala modulates memory consolidation based on emotional arousal; supports the Daimon's preferential replay of high-arousal episodes during dream cycles.
- [MEHRABIAN-RUSSELL-1974] Mehrabian, A. & Russell, J.A. _An Approach to Environmental Psychology_. MIT Press, 1974. Introduces the PAD (Pleasure-Arousal-Dominance) emotional space; the origin of the three-dimensional continuous representation used throughout the Daimon system.
- [PARK-2023] Park, J.S. et al. "Generative Agents." _UIST_, 2023. Demonstrates believable human-like behavior in LLM agents through memory retrieval, reflection, and planning; validates the general architecture of LLM agents with structured memory that the Golem extends with affect.
- [PATHAK-2017] Pathak, D. et al. "Curiosity-Driven Exploration." _ICML_, 2017. Shows that intrinsic curiosity (prediction error in a forward model) enables effective exploration without extrinsic reward; the theoretical basis for using surprise as an exploration signal in the Daimon.
- [PEKRUN-2006] Pekrun, R. "Control-Value Theory." _Educational Psychology Review_, 18, 2006. Proposes that emotions arise from appraisals of control and value in achievement situations; provides the CVT-to-PAD mapping used in the extended personality model.
- [PLUTCHIK-1980] Plutchik, R. _Emotion: A Psychoevolutionary Synthesis_. Harper & Row, 1980. Defines eight primary emotions arranged in opposing pairs with three intensity levels; the basis for the discrete emotion labels (Joy, Trust, Fear, etc.) mapped from PAD octants.
- [RUSSELL-MEHRABIAN-1977] Russell, J.A. & Mehrabian, A. "Evidence for a Three-Factor Theory of Emotions." _Journal of Research in Personality_, 11(3), 1977. Provides empirical validation that three independent dimensions (Pleasure, Arousal, Dominance) account for the structure of emotional experience; confirms PAD as a sufficient representational basis.
- [SCHERER-2001] Scherer, K.R. "Appraisal Considered as a Process of Multilevel Sequential Checking." In _Appraisal Processes in Emotion_, Oxford, 2001. Models appraisal as a sequence of checks (novelty, pleasantness, goal relevance, coping potential, norm compatibility); the template for the Daimon's 8-step appraisal pipeline.
- [SIMON-1971] Simon, H.A. "Designing Organizations for an Information-Rich World." 1971. Argues that information abundance creates attention scarcity, making attention allocation the bottleneck; the original framing for why salience signals (emotions) are computational infrastructure, not decoration.
- [WILSON-2021] Wilson, R.C. et al. "Balancing Exploration and Exploitation." _Current Opinion in Behavioral Sciences_, 38, 2021. Reviews the explore-exploit tradeoff and notes that emotions like curiosity and confidence naturally modulate it; supports the Daimon's mood-driven exploration temperature.
- [ZHANG-SELF-EMOTION-2024] Zhang, Q. et al. "Self-Emotion Blended Dialogue Generation." _SIGDIAL_, 2024. Finds that self-emotion changes approximately 50% of agent decisions in social simulation; the empirical headline supporting the claim that affect is architecturally significant, not marginal.

---
