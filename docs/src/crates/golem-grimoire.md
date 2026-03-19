# golem-grimoire

`golem-grimoire` is the persistent knowledge store for golems. It holds everything a golem has learned across its lifetime and provides the retrieval interface that the heartbeat's retrieve step uses each tick.

## Features

- **Episodic memory** (LanceDB): vector-indexed records of past observations and outcomes, searchable by semantic similarity
- **Semantic memory** (SQLite): structured facts, entity relationships, and extracted patterns from episodic consolidation
- **Procedural memory** (PLAYBOOK.md): a human-readable markdown file containing step-by-step strategies the golem has developed and refined
- Similarity search across episodic memory using vector embeddings
- Configurable retrieval: control how many episodic, semantic, and procedural entries are loaded per tick
- Compression at death: the Thanatopsis protocol calls `grimoire.compress(2048)` to reduce the store to its most important entries before pushing to bardo-styx

## Getting Started

The Grimoire is initialized automatically when a golem starts. It reads from persistent storage if a previous generation's compressed Grimoire was inherited from bardo-styx.

```rust
use golem_grimoire::Grimoire;

let grimoire = Grimoire::open(&config).await?;

// Retrieve entries relevant to current context
let entries = grimoire.retrieve(&query_embedding, limit).await?;

// Store a new episodic memory after a completed trade
grimoire.record_episode(&episode).await?;
```

## Architecture

`golem-grimoire` is in Layer 2 (Cognition). It is read every tick by `golem-context` during workspace assembly, and written by the heartbeat's reflect step after a tick completes. `golem-dreams` reads and writes the Grimoire during sleep consolidation. At death, `golem-mortality` calls the compression routine to prepare the Grimoire for clade inheritance.

LanceDB handles the vector similarity search for episodic retrieval. SQLite handles structured queries for semantic facts. PLAYBOOK.md is read as plain text and included in the `CognitiveWorkspace` for the LLM's procedural context.
