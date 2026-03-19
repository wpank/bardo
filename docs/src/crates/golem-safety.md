# golem-safety

`golem-safety` enforces capability-based security and policy constraints on golem actions. Before a golem can execute a transaction or call an external service, the action must pass through the safety layer.

## Features

- `Capability<T>`: a typed token required to perform action `T`; capabilities are granted at startup and cannot be forged
- `PolicyCage`: evaluates a set of configurable policies against a proposed action — position size limits, gas limits, allowed protocols, blocked addresses
- Audit log: every action attempt is recorded with its capability token, policy decision, and outcome
- Taint propagation: actions derived from tainted inputs (from `golem-core`'s `TaintedString`) require elevated capability to execute

## Architecture

`golem-safety` is Layer 3 — it sits above cognition and below the infrastructure crates. This placement is intentional: safety checks run after the golem has decided what it wants to do but before any external side effect occurs.

The `PolicyCage` is configured at startup from `GolemConfig`. Policies are evaluated in order; the first failing policy blocks the action. The audit log is append-only and written to SQLite.

`golem-chain`'s Warden calls into `golem-safety` as its pre-submission check. Any transaction that the Warden would submit must hold the appropriate `Capability<ExecuteTransaction>` and pass all active policies.
