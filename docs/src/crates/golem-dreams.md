# golem-dreams

`golem-dreams` implements sleep cycles for golems. When a golem is not actively trading — during low-activity windows or after a high-intensity session — it enters a sleep state that consolidates knowledge from episodic memory into more durable semantic and procedural forms.

## Features

- NREM sleep: replay recent episodic memories and extract patterns for semantic storage
- REM sleep: recombine memories in novel ways, generating candidate hypotheses and PLAYBOOK updates
- Consolidation: write distilled knowledge back to the Grimoire's SQLite semantic store and PLAYBOOK.md
- Configurable sleep schedule: trigger on time elapsed, activity thresholds, or explicit signals

## Architecture

`golem-dreams` is in Layer 2 (Cognition). It reads from and writes to `golem-grimoire`. The `golem-runtime` lifecycle FSM transitions the golem into a sleeping state, at which point the heartbeat pauses execution ticks and `golem-dreams` takes over. When consolidation completes, the FSM transitions back to active.

Sleep is not just idle time. The quality of a golem's decisions over its lifetime depends partly on how well it has consolidated past experience into the Grimoire. Golems that sleep regularly carry better-organized knowledge into each tick.
