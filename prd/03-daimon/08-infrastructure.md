# Daimon Infrastructure [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crate**: `golem-daimon`
>
> **Depends-on**: `00-overview.md`, `../20-styx/`

---

## Summary

The `golem-daimon` crate is a pure Rust library with no external runtime dependencies. It requires no network access and no inference calls for its core emotion loop. All emotional computation -- appraisal, PAD updates, mood EMA, somatic landscape queries -- runs deterministically inside the Golem's process using only CPU and local memory. The Daimon adds no containers, no network endpoints, and no database instances beyond what the Golem runtime already provisions. Emotional metadata piggybacks on existing Styx storage layers with <2% size overhead per entry.

---

> **Reader orientation:** This document specifies the infrastructure footprint of the `golem-daimon` crate within Bardo (the Rust runtime for mortal autonomous DeFi agents). The Daimon (affect engine) is a pure Rust library with no external runtime dependencies -- no network access, no inference calls for its core emotion loop. All emotional computation runs deterministically using CPU and local memory. This document covers the crate layout, dependencies, memory footprint (<100KB steady-state), and Styx (global knowledge relay) integration points. Prerequisites: the Daimon overview (`00-overview.md`). For a full glossary, see `prd2/shared/glossary.md`.

## Root Configuration

All Daimon sub-configs are aggregated under a single `DaimonConfig` struct, loaded from `bardo.toml` at creation time. This is the entry point for all emotional system configuration:

```rust
/// Root configuration for the Daimon subsystem.
/// Loaded from `[daimon]` section of bardo.toml.
/// Each sub-config governs a distinct emotional subsystem.
pub struct DaimonConfig {
    /// Appraisal rules, PAD weights, mood decay rates.
    pub emotion: EmotionConfig,
    /// Dream cycle gating: interval, load threshold, recency.
    pub dream: DreamScheduler,
    /// Somatic marker EWMA alpha, marker strength thresholds.
    pub somatic: SomaticConfig,
    /// Phase boundary hysteresis band, minimum hold ticks.
    pub phase: PhaseConfig,
}
```

---

## `golem-daimon` Crate Layout

```
golem-daimon/
├── src/
│   ├── lib.rs                   # Crate root, re-exports
│   ├── appraisal.rs             # 8-step OCC/Scherer pipeline
│   │                            #   rule_based_appraisal(), daimon_appraisal()
│   │                            #   grounding validation, mortality_appraisal()
│   ├── pad.rs                   # PADVector type, Plutchik mapping
│   │                            #   pad_to_plutchik(), cosine_similarity()
│   │                            #   encode/decode for CorticalState
│   ├── cortical_state.rs        # Lock-free cross-fiber shared perception surface
│   │                            #   AtomicU128 packed PAD, AtomicU8 regime
│   │                            #   AtomicU32 vitality, ~32 signals total
│   ├── somatic_landscape.rs     # k-d tree emotional topology
│   │                            #   ValenceAccumulator, query_valence()
│   │                            #   gut_feeling(), export_for_inheritance()
│   ├── mood.rs                  # EMA mood tracking (ALMA Layer M)
│   │                            #   update_mood(), decay_toward_personality()
│   │                            #   MoodState, MoodSnapshot
│   ├── personality.rs           # Static personality config (ALMA Layer P)
│   │                            #   PersonalityProfile, Eros/Thanatos baselines
│   └── behavior.rs              # Five-phase behavioral modulation
│                                #   mood_modulated_exploration()
│                                #   daimon_adjusted_risk()
│                                #   mood_escalation_bias()
│                                #   adjust_probe_threshold()
│                                #   daimon_adjusted_sharing_threshold()
├── Cargo.toml
└── tests/
    ├── appraisal_tests.rs
    ├── pad_tests.rs
    ├── cortical_state_tests.rs
    ├── mood_tests.rs
    └── behavior_tests.rs
```

---

## Dependencies

| Dependency | Version | Purpose |
| --- | --- | --- |
| `golem-core` | workspace | `GolemEvent`, `EventFabric`, `GolemState`, type definitions |
| `golem-grimoire` | workspace | `emotion_log` table queries, `GrimoireEntry` emotional tag writes |
| `kiddo` | ^4 | **3-D** k-d tree for Somatic Landscape: the three dimensions are P, A, D (Pleasure, Arousal, Dominance) from the PAD emotional space. Nearest-neighbor queries find the most emotionally similar past outcomes for a given affective state. |
| `moka` | ^0.12 | Decision cache with somatic markers (TTL-bounded, concurrent) |
| `serde` | ^1 | Serialization for all Daimon state structs |
| `rusqlite` | ^0.31 | SQLite queries for emotion_log and Grimoire entry updates |

No async runtime dependency. The Daimon extension runs synchronously within the `after_turn` hook. The CorticalState uses `std::sync::atomic` only.

**Mood sampling.** The mood EMA is sampled every 10 ticks. A minimum of 10 samples (100 ticks) is required before the mood classification is considered meaningful. Before 100 ticks, the Golem uses its personality baseline as the mood state. This prevents early-life transient emotions from triggering phase-dependent behavioral changes before the Golem has accumulated enough affective history for the EMA to stabilize.

---

## Memory Footprint

| Component | Size | Growth Rate |
| --- | --- | --- |
| CorticalState | 256 bytes | Static (allocated once at creation) |
| Somatic Landscape (k-d tree) | ~10 KB initial, grows with strategy exploration | ~100 bytes per recorded outcome |
| PADVector + mood state | 48 bytes | Static |
| Personality profile | 12 bytes | Static |
| emotion_log (SQLite) | ~1 MB/day at 1 tick/min | Linear with tick rate |
| Decision cache (moka) | ~50 KB (bounded) | TTL-bounded, evicts automatically |

Total steady-state memory: under 100 KB excluding the emotion_log, which lives on disk in SQLite.

---

## Styx Integration (External Storage)

The Daimon adds no infrastructure beyond what the Styx service already provides. Emotional metadata travels as additional fields on existing storage structures:

- **Styx Archive layer**: Emotional metadata (PAD vectors, Plutchik labels, arousal levels) is part of the encrypted per-Golem payload. No schema changes. Size overhead <2% per entry.

- **Styx Lethe (formerly Commons) layer**: Emotional metadata is stored as additional metadata fields on existing Qdrant vector entries. Lethe queries apply emotional congruence as a secondary ranking factor (15% weight) at query time.

- **Retrieval scoring**: Emotional boost scores are computed at query time and folded into the final score. No cache schema changes needed.

No additional environment variables, API endpoints, or infrastructure components are required for Daimon support.

---

## Cross-References

| Topic | Document |
| --- | --- |
| Styx service architecture | `../20-styx/` -- Global knowledge relay and persistence layer: Vault (encrypted backup), Clade (group sharing), Lethe (semantic search). Emotional metadata piggybacks on existing Styx storage with <2% size overhead. |
| Emotional tagging in memory | `02-emotion-memory.md` -- How PAD vectors attach to Grimoire entries and bias retrieval through the four-factor scoring model. |
| CorticalState spec | `00-overview.md` (section 5b) -- Lock-free AtomicU128-packed PAD for zero-latency cross-fiber reads. |
| Daimon crate module layout | `00-overview.md` (section 5c) -- Module responsibilities: appraisal, pad, cortical_state, somatic_landscape, mood, personality, behavior. |
| Event variants | `00-overview.md` (section 5d), `14-events.md` -- DaimonAppraisal, SomaticMarkerFired, EmotionalShift, MoodUpdate event types and their emission conditions. |
| Behavioral modulation | `03-behavior.md` -- Five modulation channels (exploration, risk, inference tier, probes, sharing) parameterized by mood and BehavioralPhase. |
| Dream-daimon bridge | `06-dream-daimon.md` -- Bidirectional affect-dreaming interaction: emotional load drives urgency, REM depotentiation reduces arousal, dream outcomes feed appraisal. |
