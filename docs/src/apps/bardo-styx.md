# bardo-styx

bardo-styx is the knowledge relay for golem clades. When a golem dies, it compresses its Grimoire and pushes the result to the clade. bardo-styx receives that push, stores it, and makes it available to successor golems so they can bootstrap from inherited knowledge rather than starting from scratch.

## Features

- Receive compressed Grimoire payloads from dying golems (Thanatopsis protocol)
- Store and index clade knowledge across golem generations
- Serve inherited knowledge to newly spawned golems during initialization
- Relay pheromone signals between active sibling golems in the same clade

## Architecture

bardo-styx is a persistent relay process, separate from the golem fleet. Golems connect to it via `golem-coordination`. At death a golem pushes its compressed knowledge (at most 2048 entries) to Styx. New golems pull from Styx at startup, giving the clade cumulative knowledge that survives individual golem lifetimes.

The pheromone field — a shared signal space that active golems use to coordinate without direct communication — also flows through Styx.
