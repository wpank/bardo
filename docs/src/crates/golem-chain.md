# golem-chain

`golem-chain` handles all Ethereum interaction for golem processes. It wraps Alloy for RPC connectivity, integrates with `revm` for local execution, implements ERC-8004 for on-chain golem identity, and provides the Warden — a pre-flight transaction guard that enforces safety constraints before any transaction leaves the process.

## Features

- Alloy-based HTTP and WebSocket RPC client with retry and backoff
- Local transaction simulation via `revm` before mainnet submission
- ERC-8004 contract interface for on-chain golem registration and death masks
- Warden: pre-submission transaction guard that checks slippage, gas limits, and policy constraints
- ABIs and typed bindings for common DeFi protocols (Uniswap V3, Aave V3)
- Support for EIP-1559, EIP-2930, and EIP-4844 transaction envelopes

## Architecture

`golem-chain` is in Layer 4 (Infrastructure). It depends on `golem-safety` to enforce the Warden's policy checks and on `golem-core` for configuration and error types.

The typical flow for a golem executing a trade: the heartbeat's execute step constructs a transaction, passes it through the Warden for pre-flight checks, simulates it locally via `revm`, and only submits to mainnet if simulation succeeds and the Warden approves. The verify step then confirms the transaction landed and reads back the on-chain result.
