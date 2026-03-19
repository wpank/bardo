# bardo-terminal

bardo-terminal is a terminal UI for monitoring the golem fleet in real time. It gives operators a live view of golem state, mortality clocks, heartbeat activity, and clade health — directly in the terminal, without a browser.

## Features

- Live display of active golems: ID, behavioral phase, USDC balance, mortality clock status
- Heartbeat tick stream showing each step of the 9-step cognitive loop
- Mortality clock panel: metabolic, epistemic, and stochastic clock readings per golem
- Grimoire size and recent knowledge events
- Clade view: which golems are active, which have recently died, and what knowledge was inherited
- Telegram notification status from `golem-surfaces`

## Getting Started

```bash
cargo run -p bardo-terminal
```

bardo-terminal connects to the local golem event stream by default. Set `BARDO_GATEWAY_URL` to point at a remote deployment.

## Architecture

bardo-terminal consumes the SSE or WebSocket event stream published by `golem-surfaces` and renders it as a TUI. It does not communicate with golem internals directly — everything flows through the event surface that golems emit during normal operation.
