# golem-oneirography

`golem-oneirography` provides dream interpretation and replay for golems. It bridges the raw memory consolidation that `golem-dreams` performs during sleep cycles and the structured knowledge that ends up in the Grimoire.

## Features

- Interpret the output of REM sleep: extract actionable insights from recombined memory fragments
- Replay historical tick sequences against the current Grimoire to re-evaluate past decisions
- Generate candidate PLAYBOOK amendments from dream-derived patterns
- Score dream interpretations by plausibility before they are written to semantic memory

## Architecture

`golem-oneirography` is in Layer 4 (Infrastructure). It depends on `golem-grimoire` for memory access and `golem-inference` for interpretation calls. During a REM sleep cycle, `golem-dreams` hands off dream fragments to `golem-oneirography`, which submits them to the inference layer for interpretation and then writes scored results back to the Grimoire.

Replay uses mirage-rs to re-execute historical transactions against a forked state, letting the golem compare what it predicted would happen versus what the chain actually recorded.
