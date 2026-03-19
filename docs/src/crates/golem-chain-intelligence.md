# golem-chain-intelligence

`golem-chain-intelligence` provides on-chain observation infrastructure for golems. It handles real-time block watching and price validation — the two most latency-sensitive data feeds a golem needs to make informed trading decisions.

## Features

- **bardo-witness**: real-time block and event subscriber that watches the chain and feeds observations into the golem's event fabric
- **Price Validation Service (PVS)**: validates price data from multiple sources, detects manipulation, and provides a confidence-weighted price feed that the golem can trust

## Architecture

`golem-chain-intelligence` sits in Layer 4 (Infrastructure). It consumes chain types from `golem-chain` and publishes events to `golem-core`'s `EventFabric`. The PVS cross-references prices from on-chain oracles, DEX pools, and external feeds to reject outliers before they reach the golem's decision logic.

bardo-witness runs as a background task inside the golem process. It subscribes to new blocks and filtered log streams, then emits typed `GolemEvent` values that the heartbeat's observe step picks up on the next tick.
