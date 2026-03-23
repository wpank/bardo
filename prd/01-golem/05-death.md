# Death Protocol [STUB]

> **Version**: 1.2 | **Status**: Cross-reference stub
>
> **Crate**: `golem-mortality` (Thanatopsis module)

> **Reader orientation:** This is a cross-reference stub pointing to the full Death Protocol specification. Thanatopsis (the four-phase structured shutdown: Acceptance, Settlement, Reflection, Legacy) ensures that death produces value: recovered capital, distilled knowledge, and transmissible wisdom. The full spec lives in `02-mortality/06-thanatopsis.md`. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

The full death protocol specification -- Thanatopsis phases, apoptotic reserve, degradation cascade, insurance snapshots, emotional life review, dream journal, and tombstone records -- has moved to the dedicated mortality section.

**See [02-mortality/06-thanatopsis.md](../02-mortality/06-thanatopsis.md) for the death protocol.**

Key subsections:

| File                               | Topic                                                                  |
| ---------------------------------- | ---------------------------------------------------------------------- |
| `02-mortality/06-thanatopsis.md`   | The four-phase Death Protocol: Acceptance, Settlement, Reflection, Legacy -- how death produces value |
| `02-mortality/07-succession.md`    | Rebirth mechanics: lineage tracking, the Baldwin Effect (learned heuristics becoming innate defaults), anti-proletarianization mandate |
| `02-mortality/13-configuration.md` | MortalityConfig struct, apoptotic reserve (minimum-viable death budget), protocol invariants |

The Death Protocol ensures that death produces value: recovered capital, distilled knowledge, and transmissible wisdom. No Golem dies silent. The Apoptotic Reserve guarantees a minimum-viable death even when resources are nearly exhausted.

---

## Events Emitted

Death protocol events track each phase of the Thanatopsis sequence. All events are variants of the `GolemEvent` enum emitted to the Event Fabric.

| Event | Trigger | Payload |
|---|---|---|
| `GolemEvent::DeathProtocolStarted` | Thanatopsis begins | `{ golem_id: GolemId, trigger: DeathTrigger, vitality_score: f64 }` |
| `GolemEvent::PositionsUnwound` | All positions closed | `{ positions_exited: u32, total_recovered: f64, slippage_cost: f64 }` |
| `GolemEvent::TestamentWritten` | Final knowledge persisted | `{ testament_hash: B256, heuristic_count: u32, styx_vault_entries: u32 }` |
| `GolemEvent::DeathCompleted` | Golem process exits | `{ golem_id: GolemId, lifetime_ticks: u64, total_cost_usd: f64, final_nav: f64 }` |
