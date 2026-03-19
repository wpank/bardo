# golem-coordination

`golem-coordination` handles communication between golems in the same clade. It implements the pheromone field — a shared signal space that lets sibling golems coordinate without direct peer-to-peer messaging — and manages the knowledge sync protocol with the bardo-styx relay.

## Features

- Pheromone field: publish and read typed signals that decay over time
- Clade sync: push compressed Grimoire knowledge to bardo-styx at death, pull inherited knowledge at startup
- Peer awareness: track which sibling golems are currently alive and their last-known behavioral phases
- Conflict avoidance: read pheromone signals before taking action to avoid duplicating what siblings are already doing

## Architecture

`golem-coordination` is Layer 5, directly above the infrastructure crates. It connects outward to bardo-styx over an HTTP or WebSocket connection. Inside the golem, it publishes pheromone reads as `GolemEvent` values into the event fabric so the heartbeat's observe step can factor clade state into decisions.

The pheromone field is eventually consistent. Signals are written to bardo-styx, which broadcasts them to connected clade members. A golem that just started may not see very recent signals from siblings until the first sync completes.
