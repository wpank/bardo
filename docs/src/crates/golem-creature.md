# golem-creature

`golem-creature` is the visual identity engine for golems. Each golem has a persistent visual identity — generated deterministically from its `GolemId` — that appears in dashboards, death masks, and the graveyard.

## Features

- Generate a deterministic visual avatar from a `GolemId`
- Produce identity assets for use in bardo-terminal, bardo-gateway, and on-chain death masks
- Stable across golem restarts: the same ID always produces the same visual identity

## Architecture

`golem-creature` is in Layer 6 (Surfaces). It depends on `golem-core` for identity types and produces visual output consumed by `golem-engagement` (graveyard records) and `golem-surfaces` (WebSocket/Telegram notifications). The visual identity is computed once at golem startup and cached for the lifetime of the process.
