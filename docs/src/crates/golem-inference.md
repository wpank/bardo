# golem-inference

`golem-inference` handles all LLM calls for the golem. It routes requests to one of three cost tiers, pays for inference using x402 micropayments, and enforces the per-tick inference budget that keeps the golem's metabolic USDC spend predictable.

## Features

- Three-tier routing: `T0` (fast/cheap), `T1` (balanced), `T2` (powerful/expensive)
- x402 micropayment integration: pay-per-call inference without subscriptions
- Per-tick budget enforcement: gate tier upgrades when the budget is nearly exhausted
- Response caching: avoid redundant calls for identical prompts within a tick
- Streaming support for long-form T2 responses

## Tiers

| Tier | Use | Cost |
|---|---|---|
| `T0` | Quick classifications, binary decisions, pattern matching | Low |
| `T1` | Analysis, reasoning, strategy evaluation | Medium |
| `T2` | Complex multi-step reasoning, PLAYBOOK updates | High |

The heartbeat's gate step uses `CognitiveTier` from `golem-core` to determine which tier is appropriate for each inference call in the current tick.

## Architecture

`golem-inference` is in Layer 4 (Infrastructure). It is called from `golem-heartbeat` at the analyze, gate, simulate, and reflect steps. The x402 payment channel connects to an inference provider that accepts per-call USDC micropayments, keeping inference costs tied directly to the golem's metabolic balance.

The tier router selects an endpoint based on the requested `CognitiveTier` and the current per-tick budget. If the budget is exhausted, requests downgrade to `T0` or are rejected entirely.
