# golem-mortality

`golem-mortality` implements the three mortality clocks and the Thanatopsis death protocol. Every golem is mortal; this crate is what makes that true.

## The Three Clocks

A golem dies when any of these fires:

- **Metabolic clock** — tracks USDC balance. When it reaches zero, the golem cannot pay for inference or gas and must die.
- **Epistemic clock** — tracks prediction accuracy and knowledge quality over time. If accuracy decays below a configured threshold and stays there, the golem is no longer useful and must die.
- **Stochastic clock** — a random draw fired at configurable intervals. Regardless of financial or epistemic health, mortality can strike at random. This prevents immortal golems from dominating the clade.

## Thanatopsis Protocol

When any clock fires, `golem-mortality` initiates a four-phase death sequence:

1. **Acceptance** — halt new tick execution, notify the runtime, emit a death event
2. **Settlement** — close open positions, return unspent capital, write the final financial record
3. **Reflection** — run a last Grimoire consolidation; write a summary of the golem's life to the death mask
4. **Legacy** — compress the Grimoire to at most 2048 entries, push to bardo-styx, submit the ERC-8004 death mask on-chain

## Features

- Three independent mortality clocks running as background tasks
- Configurable thresholds: minimum USDC balance, epistemic fitness floor, stochastic draw interval
- Clean shutdown: in-flight transactions are allowed to complete before death proceeds
- Death mask: an on-chain record containing the golem's ID, final state hash, and a pointer to its compressed knowledge
- Successor handoff: compressed Grimoire is available via bardo-styx for the next golem in the clade

## Architecture

`golem-mortality` is in Layer 2 (Cognition). It runs three background tasks that continuously monitor the metabolic balance (via chain reads), epistemic fitness (via Grimoire quality metrics), and a random draw timer. Any clock firing sends a shutdown signal to `golem-runtime`, which then calls the Thanatopsis sequence.

The death protocol is designed to be atomic with respect to the clade: either the full Legacy phase completes (including the on-chain mask) or the compressed Grimoire is not pushed. This prevents partially-dead golems from corrupting the clade's knowledge base.
