# Runtime-Daimon Bridge: How Emotions Surface in the Runtime Layer [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-daimon`, `golem-surfaces`, `golem-creature`
>
> **Canonical sources**: Runtime PRD suite -- interaction model, communication channels, data visibility, knowledge browser, collective intelligence, observability
>
> **Depends-on**: `00-overview.md`, `03-behavior.md`

---

> **Reader orientation:** This document specifies how the Daimon's (the affect engine's) internal emotional state surfaces in the Golem's (mortal autonomous agent's) runtime layer: conversational tone mapping from PAD vectors (Pleasure-Arousal-Dominance), GolemEvent variants, creature visualization, emotional contagion within Clades (groups of related Golems sharing knowledge), and Prometheus metrics. Part of the Daimon track within Bardo (the Rust runtime for mortal autonomous DeFi agents). Prerequisites: the Daimon overview (`00-overview.md`) and behavioral modulation (`03-behavior.md`). For a full glossary, see `prd2/shared/glossary.md`.

## Summary

The runtime layer is the surface through which the Daimon's internal emotional state becomes visible to owners, external systems, and Clade siblings. This bridge document specifies how PAD state maps to conversational tone, which GolemEvent variants carry emotional data, what mood data formats the knowledge browser expects, how emotional contagion is capped and observed, creature visualization integration, and what Prometheus metrics the Daimon exports. For full runtime architecture, see the Runtime PRD suite.

---

## 1. Conversational Tone Mapping

The Daimon's current PAD state modulates the Golem's conversational style when interacting with owners via web chat, Telegram, or Discord. The mapping is injected as part of the daimon context block (see `00-overview.md`, Section 9) and influences the LLM's response generation.

### PAD-to-Tone Table

| PAD Octant | Mood Label | Conversational Characteristics                                                                                 |
| ---------- | ---------- | -------------------------------------------------------------------------------------------------------------- |
| +P, +A, +D | Exuberant  | Enthusiastic, proactive, offers unsolicited observations ("I noticed an interesting pattern in gas prices...") |
| +P, +A, -D | Dependent  | Shares credit with external factors, grateful ("The market moved in our favor today")                          |
| +P, -A, +D | Relaxed    | Concise, confident, minimal hedging, uses definitive language                                                  |
| +P, -A, -D | Docile     | Brief acknowledgments, defers to owner judgment, low initiative                                                |
| -P, +A, +D | Hostile    | Direct, attributes problems externally, terse ("Gas costs are unreasonable")                                   |
| -P, +A, -D | Anxious    | Verbose, hedges extensively, flags risks, requests owner confirmation before actions                           |
| -P, -A, +D | Disdainful | Minimal output, dismissive of contrary signals, low engagement                                                 |
| -P, -A, -D | Bored      | Very brief responses, may suggest exploring new strategies or markets                                          |

### Implementation

The tone mapping is not a hard override but a contextual signal. The mood label is injected into the `<daimon>` block, and the LLM naturally adapts its response style. In testing, the mood label alone (without explicit tone instructions) produces contextually appropriate responses in 80%+ of cases.

### Channel-Specific Adaptation

- **Web chat**: Full tone expression. Mood label and recent emotions are visible in the daimon context.
- **Telegram**: Tone expression limited by MarkdownV2 formatting. Mood label influences word choice but not formatting.
- **Discord**: Tone expressed through embed color coding (phase-colored: Thriving green -> Terminal red) and embed footer text.

---

## 2. GolemEvent Variants for Mood

The runtime's Event Fabric carries emotional state changes as typed `GolemEvent` enum variants to connected clients via WebSocket or SSE. This replaces the previous SSE-only event model with the unified GolemEvent system (see `14-events.md` in rewrite4).

### Daimon Event Variants

The canonical daimon event variants (see `14-events.md` in rewrite4):

| Event Variant | Trigger | Key Fields |
| --- | --- | --- |
| `DaimonAppraisal` | Every appraisal completes (step 8) | `pleasure`, `arousal`, `dominance`, `emotion`, `markers_fired`, `intensity` |
| `SomaticMarkerFired` | Somatic marker matches current situation | `situation`, `valence`, `source`, `strategy_param` |
| `EmotionalShift` | Dominant Plutchik emotion changes | `from_emotion`, `to_emotion` |
| `MoodUpdate` | PAD Euclidean delta > 0.15 | `pleasure`, `arousal`, `dominance`, `mood_trend` |

The `MoodUpdate` threshold (0.15 Euclidean distance from last emitted state) prevents event flooding. A shift from "relaxed" to "anxious" (PAD distance ~1.2) always emits; a shift from "mildly relaxed" to "slightly less relaxed" (PAD distance ~0.05) does not. `EmotionalShift` fires on discrete label changes, not continuous PAD drift.

### Dream Mode Emotional Events

During dream mode, the Event Fabric emits dream-specific GolemEvent variants:

| GolemEvent Variant    | When                           | Key Fields                                            |
| --------------------- | ------------------------------ | ----------------------------------------------------- |
| `DreamStart`          | Dream cycle begins             | `emotional_load`, `dream_type_allocation`             |
| `DreamPhase`          | Dream phase transition         | `phase`, `episode_count`                              |
| `DreamHypothesis`     | Hypothesis generated during REM| `hypothesis`, `confidence`, `emotional_origin`        |
| `DreamCounterfactual` | Counterfactual imagined in REM | `hypothesis`, `outcome`                               |
| `DreamComplete`       | Dream cycle completes          | `depotentiation_count`, `hypotheses_generated`, `mood_shift` |

The `DreamComplete` event includes the mood shift caused by the dream cycle, allowing connected clients to observe the depotentiation effect in real time.

### Phase Change Events

Vitality phase transitions (Thriving -> Stable -> Conservation -> Declining -> Terminal) emit `VitalityUpdate` events with fields `{ economic, epistemic, stochastic, composite, phase }` that include the emotional state at transition. Critical mortality events (vitality < 0.2) are pushed to all connected channels regardless of subscription. `HeartbeatTick` and `HeartbeatComplete` events carry timing data; `TierSelected` events log inference tier escalation decisions.

---

## 3. TUI and Creature Visualization

The TUI is the primary surface for affect visualization. The creature system (`golem-creature` crate, see `10-surfaces.md`) reads PAD state from the CorticalState atomics, not from events. This is a performance choice: the creature animation loop runs at ~10fps and needs the latest PAD values on every frame. Polling the CorticalState (`cortical_state.read_pad()`) is a single atomic load, 2--3 nanoseconds, zero contention. Subscribing to `DaimonAppraisal` events through the Event Fabric would add channel overhead and introduce lag between appraisal completion and visual update.

The creature system provides a visual avatar that embodies the Golem's internal state. Event Fabric events are still used for discrete transitions (dream start, death, achievement) that trigger particle effects.

### PAD -> Creature Expression Mapping

The 8-octant PAD model maps directly to creature visual state:

| Expression     | PAD Octant | Animation Speed | Posture          | When                         |
|---------------|------------|----------------|------------------|------------------------------|
| **Exuberant** | +P +A +D   | Fast           | Tall, open       | Profitable trade in trending market |
| **Dependent** | +P +A -D   | Fast           | Small            | Good outcome but unsure why  |
| **Relaxed**   | +P -A +D   | Slow           | Open, casual     | Calm market, strategy working |
| **Docile**    | +P -A -D   | Slow           | Low, passive     | Modest gains, following PLAYBOOK |
| **Hostile**   | -P +A +D   | Fast           | Aggressive       | Loss despite strong conviction |
| **Anxious**   | -P +A -D   | Jittery        | Defensive, hunched| Approaching liquidation     |
| **Disdainful**| -P -A +D   | Slow           | Dismissive       | Stale strategy, bored       |
| **Bored**     | -P -A -D   | Minimal        | Slumped          | Persistent underperformance |

### Creature State Computer

The creature state computer runs as a background tokio fiber, subscribing to the Event Fabric and updating visual state reactively:

```rust
/// Creature state computer. Subscribes to Event Fabric, computes visual state.
/// This is the bridge between the Daimon's internal PAD state and the
/// visible creature avatar that owners interact with.
pub struct CreatureStateComputer {
    current_state: Arc<parking_lot::RwLock<CreatureState>>,
}

impl CreatureStateComputer {
    pub async fn run(self, fabric: Arc<EventFabric>) {
        let mut rx = fabric.subscribe();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let mut state = self.current_state.write();
                    match &event.payload {
                        EventPayload::DaimonAppraisal {
                            pleasure, arousal, dominance, ..
                        } => {
                            state.expression =
                                Self::pad_to_expression(*pleasure, *arousal, *dominance);
                            state.animation_speed = arousal.abs();
                            state.posture = *dominance;
                        }
                        EventPayload::VitalityUpdate {
                            composite, phase, ..
                        } => {
                            state.form = Self::phase_to_form(phase, *composite);
                            state.luminosity = *composite;
                        }
                        EventPayload::DreamStart { .. } => {
                            state.active_effects.push(
                                ParticleEffect::persistent("dream_shimmer", 0.7),
                            );
                        }
                        EventPayload::DreamComplete { .. } => {
                            state.active_effects
                                .retain(|e| e.effect_type != "dream_shimmer");
                        }
                        EventPayload::DeathInitiated { .. } => {
                            state.form = CreatureForm::Dissolving { progress: 0.0 };
                            state.active_effects.push(
                                ParticleEffect::persistent("death_glow", 1.0),
                            );
                        }
                        _ => {}
                    }
                    state.expire_effects();
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(
                        "Creature lagged by {n} events -- rebuilding from snapshot"
                    );
                }
                Err(_) => break,
            }
        }
    }
}
```

### Particle Effects

Internal events trigger particle effects on the creature visualization:

| Internal Event           | Particle Effect       | Duration        |
|-------------------------|-----------------------|-----------------|
| Dream cycle starts       | `dream_shimmer`       | Until dream ends |
| Somatic marker fires     | `somatic_flash`       | 500ms           |
| Trade executed (success) | `trade_spark`         | 1000ms          |
| Trade executed (failure) | `trade_fizzle`        | 1000ms          |
| Bloodstain received      | `bloodstain_ripple`   | 2000ms          |
| Achievement unlocked     | `achievement_burst`   | 3000ms          |
| Death initiated          | `death_glow`          | Until dissolution |

### Evolution Forms

The creature's visual form tracks the lifecycle phase:

| Lifecycle Phase          | Creature Form    | Visual Character |
|-------------------------|------------------|------------------|
| Provisioning             | Egg              | Dormant seed, pulsing glow |
| Thriving (early)         | Hatchling        | Fresh, curious, slightly translucent |
| Thriving / Stable        | Mature           | Full form, detailed features |
| Conservation / Declining | Weathered        | Battle-scarred, darker palette |
| Terminal                 | Transcendent     | Ethereal, partially transparent |
| Dead                     | Dissolving       | Particles dispersing upward |

See `10-surfaces.md` for the full sprite generation algorithm, 29-screen, 6-window TUI system, and engagement loop design.

---

## 4. Knowledge Browser Mood Data

The Portal's Knowledge Browser displays mood-related data through several UI components:

### Mood History Widget

Displays the Golem's PAD trajectory over time as a three-line chart (P, A, D):

```rust
/// Mood history data served by the /api/v1/state endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodHistoryData {
    pub samples: Vec<MoodSample>,
    pub sampling_interval: u64,
    pub current_mood: PADVector,
    pub personality_baseline: PADVector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodSample {
    pub tick: u64,
    pub timestamp: u64,
    pub pad: PADVector,
    pub label: String,
    pub trigger: Option<String>,
}
```

Sampling interval defaults to every 10 ticks for the last 1000 ticks, every 50 ticks for older data.

### Mood-Performance Overlay

Correlates mood trajectory with P&L performance, marking periods where emotional state and performance diverged (potential emotional bias indicators).

### Phase Timeline

Displays behavioral phases as colored bands overlaid on the mood chart, showing how emotional state relates to vitality-driven phase transitions.

### Emotional Tags on Episodes

Episode cards in the timeline display their Plutchik label and intensity as a colored badge. Flashbulb memories are highlighted with a distinct border treatment.

### Mortality Widget

A compact vitality bar with phase indicator, TTL estimate, and color coding (green through red). Displays the current metamorphosis state (camel/lion/child) when the daimon is enabled.

---

## 5. Collective Intelligence Emotional Contagion

The Clade system implements emotional contagion with specific caps and decay rules that the Daimon must respect.

### Contagion Rules

| Rule                 | Value          | Purpose                                                          |
| -------------------- | -------------- | ---------------------------------------------------------------- |
| P attenuation        | 0.3            | A sibling's pleasure shift produces at most 30% equivalent shift |
| A attenuation        | 0.3            | A sibling's arousal shift produces at most 30% equivalent shift  |
| D attenuation        | 0.0            | Dominance is not contagious (control perception is local)        |
| Arousal cap per sync | +0.3           | Prevents cascading panic across the Clade                        |
| Decay half-life      | 6 hours        | Borrowed emotions dissipate unless reinforced by own experience  |
| Propagation          | Unidirectional | No reciprocal feedback (prevents positive feedback loops)        |

### Contagion Triggers

| Trigger                                             | Emotional Effect                                       |
| --------------------------------------------------- | ------------------------------------------------------ |
| Sibling `warning` push                              | Receiver arousal +0.1 (capped)                         |
| Sibling `regime_shift` alert                        | Receiver arousal +0.1 (capped)                         |
| Sibling Sharpe > 2.0 in 24h (discovered via status) | Receiver dominance +0.05                               |
| Sibling death                                       | Full `sibling_death_appraisal()` (see `01-appraisal.md`) |

### Anti-Cascade Design

Three mechanisms collectively prevent emotional cascades:

1. **Cap per sync cycle** (+0.3 arousal max): Even if multiple siblings send alarming data, arousal increase is bounded per sync.
2. **Unidirectional propagation**: Sibling A's contagion to B does not cause B to emit contagion back to A.
3. **Rapid decay** (6h half-life): Unless the receiver's own experience confirms the borrowed emotion, it fades quickly.

---

## 6. Observability and Telemetry

### Prometheus Metrics

The Daimon exports the following metrics to the Golem's Prometheus endpoint:

| Metric                                         | Type         | Labels                        | Description                          |
| ---------------------------------------------- | ------------ | ----------------------------- | ------------------------------------ |
| `bardo_daimon_mood_pleasure`                   | Gauge        | `golem_id`                    | Current mood pleasure component      |
| `bardo_daimon_mood_arousal`                    | Gauge        | `golem_id`                    | Current mood arousal component       |
| `bardo_daimon_mood_dominance`                  | Gauge        | `golem_id`                    | Current mood dominance component     |
| `bardo_daimon_mood_label`                      | Gauge (enum) | `golem_id`, `label`           | Current mood label (octant)          |
| `bardo_daimon_appraisals_total`                | Counter      | `golem_id`, `mode`, `emotion` | Appraisal count by mode and emotion  |
| `bardo_daimon_mood_updates_total`              | Counter      | `golem_id`                    | mood_update events emitted           |
| `bardo_daimon_contagion_received_total`        | Counter      | `golem_id`                    | Emotional contagion events received  |
| `bardo_daimon_depotentiation_cycles_total`     | Counter      | `golem_id`                    | REM depotentiation cycles completed  |
| `bardo_daimon_flashbulb_memories_total`        | Counter      | `golem_id`                    | Flashbulb memories created           |
| `bardo_daimon_contrarian_retrievals_total`     | Counter      | `golem_id`                    | Forced contrarian retrievals         |
| `bardo_daimon_learned_helplessness_detections` | Counter      | `golem_id`                    | Learned helplessness flags triggered |

### Grafana Dashboard Panels

Recommended dashboard panels for daimon monitoring:

1. **PAD Trajectory**: Three-line time series (P, A, D) with phase-colored background bands
2. **Appraisal Distribution**: Pie chart of emotion categories over rolling 24h window
3. **Mood Stability**: Standard deviation of PAD components over rolling 200-tick window
4. **Contagion Activity**: Bar chart of contagion events received by source type
5. **Dream-Emotion**: Side-by-side pre/post dream PAD values showing depotentiation effect
6. **Emotional Load**: Current emotional load with urgency threshold line

### Alerts

| Alert                 | Condition                                     | Severity | Action                              |
| --------------------- | --------------------------------------------- | -------- | ----------------------------------- |
| Mood locked           | Same mood label for 500+ ticks                | Warning  | Check for emotional degeneration    |
| Rumination detected   | >80% negative-valence retrievals in 100 ticks | Warning  | Contrarian retrieval forced         |
| Helplessness detected | D < -0.3 for 200+ ticks                       | Warning  | Escape mechanisms triggered         |
| Emotional flooding    | >10 appraisals in single tick                 | Info     | Check novelty threshold calibration |
| Contagion saturation  | Arousal cap hit 3+ times in 24h               | Info     | Review Clade alert frequency        |

---

## 7. Data Visibility Tiers

Emotional data follows the runtime's three-tier visibility model:

| Data                     | Public (:3000) | Owner (:3001) | Internal (:3002) |
| ------------------------ | -------------- | ------------- | ---------------- |
| Current mood label       | Yes            | Yes           | Yes              |
| Current PAD vector       | No             | Yes           | Yes              |
| Mood history             | No             | Yes           | Yes              |
| Emotion log              | No             | Yes           | Yes              |
| Appraisal details        | No             | No            | Yes              |
| Contagion sources        | No             | Yes           | Yes              |
| Dream emotional metadata | No             | Yes           | Yes              |

The public tier exposes only the mood label (e.g., "relaxed") -- sufficient for marketplace signaling but insufficient for emotional fingerprinting. Raw PAD vectors are owner-only to prevent adversarial exploitation of predictable emotional responses.

---

## 8. Extension Integration

The `after_turn` hook chain runs extensions in a fixed order. The daimon is third in the chain, after the heartbeat (which writes observation data) and the lifespan extension (which updates mortality state). This ordering is load-bearing: the daimon needs fresh observation data and current vitality readings before it can appraise.

```rust
/// Extension trait impl for the daimon.
/// Position 3 in the after_turn chain:
///   1. heartbeat  — writes observation, updates probes
///   2. lifespan   — updates mortality clocks, vitality composite
///   3. daimon     — appraises events, updates mood, writes CorticalState
///   4. memory     — consolidation, Curator cycle
///   5. risk       — PolicyCage enforcement
impl Extension for DaimonExtension {
    fn name(&self) -> &'static str { "daimon" }
    fn priority(&self) -> u32 { 17 }

    async fn after_turn(&self, ctx: &mut TurnContext) -> Result<()> {
        // 8-step appraisal pipeline (see 01-appraisal.md)
        let result = daimon_appraisal(
            &mut ctx.state,
            &ctx.observation,
            &ctx.outcome,
        ).await?;

        // Emit DaimonAppraisal event
        ctx.emit(GolemEvent::DaimonAppraisal {
            pleasure: result.pad.pleasure as f64,
            arousal: result.pad.arousal as f64,
            dominance: result.pad.dominance as f64,
            emotion: format!("{:?}", result.emotion),
            markers_fired: result.markers.len() as u32,
            intensity: result.pad.arousal.abs() as f64,
        });

        // Emit MoodUpdate if PAD delta exceeds threshold
        if ctx.state.daimon.mood.distance(&ctx.previous_mood) > 0.15 {
            ctx.emit(GolemEvent::MoodUpdate {
                pleasure: ctx.state.daimon.mood.pleasure as f64,
                arousal: ctx.state.daimon.mood.arousal as f64,
                dominance: ctx.state.daimon.mood.dominance as f64,
                mood_trend: classify_mood_trend(&ctx.state.daimon),
            });
        }

        Ok(())
    }
}
```

The dream extension (#21) fires later in the chain and reads the daimon's output (emotional load, current mood) to make scheduling decisions.

| Extension # | Name | Hook | Behavior |
|---|---|---|---|
| 17 | `daimon` | `after_turn` | 8-step appraisal pipeline, mood EMA update, CorticalState write, event emission |
| 21 | `dream` | `after_turn` | DreamScheduler decides whether to enter a dream cycle based on emotional load, novelty, and time since last dream. Reads daimon output from CorticalState. |

---

## 9. Cross-References

| Topic                   | Document                                    |
| ----------------------- | ------------------------------------------- |
| Interaction model       | `runtime/prd/00-interaction-model.md`       |
| Communication channels  | `runtime/prd/02-communication-channels.md`  |
| Data visibility         | `runtime/prd/04-data-visibility.md`         |
| Knowledge browser       | `runtime/prd/05-knowledge-browser.md`       |
| Collective intelligence | `runtime/prd/06-collective-intelligence.md` |
| Observability           | `runtime/prd/09-observability.md`           |
| GolemEvent enum         | `14-events.md` (rewrite4)                   |
| Creature system         | `10-surfaces.md` (rewrite4)                 |
| Extension registry      | `01a-runtime-extensions.md` (rewrite4)      |
| Daimon architecture     | `00-overview.md`                            |
| Behavioral modulation   | `03-behavior.md`                            |
| Emotional contagion     | `00-overview.md`, `03-behavior.md`          |
| Dream-daimon bridge     | `06-dream-daimon.md`                        |

---
