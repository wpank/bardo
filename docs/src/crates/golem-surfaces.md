# golem-surfaces

`golem-surfaces` publishes golem activity to external consumers. It is the outward-facing event layer: WebSocket streams, Server-Sent Events, and Telegram notifications.

## Features

- WebSocket server: real-time stream of `GolemEvent` values for dashboards and bardo-terminal
- SSE endpoint: HTTP-based event stream for clients that can't maintain WebSocket connections
- Telegram integration: push notifications for significant events (death, major trade, achievement)
- Configurable event filtering: subscribers can request only specific event types or subsystems
- Replay on reconnect: new subscribers can request events from a sequence number via the `EventFabric` replay ring

## Architecture

`golem-surfaces` is in Layer 6 (Surfaces). It subscribes to the golem's `EventFabric` from `golem-core` and re-publishes events to external transports. It does not modify golem behavior; it only observes and relays.

bardo-gateway routes external traffic to `golem-surfaces` endpoints. bardo-terminal consumes the same WebSocket stream directly when running locally.

Telegram notifications are sent via the Bot API. Configure `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` in `GolemConfig` to enable them.
