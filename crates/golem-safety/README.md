# golem-safety

**Status: shell — no public API yet.**

Security and capability enforcement layer for the Golem runtime. The planned design treats authorization as a type-level problem: you cannot call a sensitive operation without holding an unforgeable token, and the compiler enforces it.

## Three Defense Categories

| Layer | Mechanism | Bypassable by prompt injection? |
|-------|-----------|--------------------------------|
| Cryptographic | LLM never touches keys or signing. Custody separated architecturally. PolicyCage constraints enforced on-chain. | No |
| Type-system | `TaintedString` flow control is a compiler error. Type-state lifecycle means ticking a dead Golem is a compile error. `Capability<T>` is move-on-use. | No |
| Runtime | Defense-in-depth checks, rate limiting, audit logging | Partially (but architectural layers still hold) |

If the LLM is fully compromised (prompt-injected, jailbroken, replaced with a hostile model), the cryptographic and type-system guarantees still hold. The LLM can propose any action. The runtime will not execute anything that violates the PolicyCage.

## Planned Public API

**`Capability<T>`** — unforgeable security token parameterized on the permission type `T`. A `Capability<WriteChain>` is required to submit transactions; a `Capability<CallLlm>` to make inference calls. Capabilities are created only through the safety registry at boot time and cannot be cloned or forged at runtime. This is the core design: ambient authority is replaced with explicit capability passing.

`!Copy` and `!Clone` by default. A capability consumed by one operation cannot be reused. Example:
```rust
fn submit_tx(cap: Capability<WriteChain>, tx: Transaction) -> Result<Receipt> {
    // cap is consumed here -- cannot be reused
}
```

**`TaintedString`** — string wrapper with provenance tracking. Marks data that arrived from external sources (API responses, chain data, user input) and requires explicit sanitization or review before it can flow into privileged operations. Prevents accidental injection through the type system.

**`PolicyCage`** — sandboxing boundary for extension execution. Extensions run inside a `PolicyCage` that limits which `Capability` types they can acquire and what resources they can access. The cage is constructed by the runtime with a policy derived from the extension's declared layer.

The DeFi Constitution. An on-chain smart contract enforcing hard safety limits: approved assets, maximum position sizes, drawdown limits, rate limits. Extensions run inside a cage constructed at boot from the extension's declared layer.

**`ActionPermit`** — lifecycle token for a single action. Issued before an action executes, consumed on completion or cancellation. Prevents double-execution and provides an audit hook.

State machine: Created -> Announced -> Waiting -> Ready -> Executed | Cancelled. Transitions are unidirectional and atomic. The type-state pattern makes invalid transitions a compile error.

**`LoopGuard`** — recursion depth counter with configurable hard limit. Wraps re-entrant operations and panics (or returns an error) if depth exceeds the configured maximum. Prevents unbounded recursion in agent reasoning loops.

**Merkle audit log** — append-only log of capability use, signed with a Merkle commitment. Each `ActionPermit` use appends an entry; the root hash can be verified by external auditors or committed on-chain.

## System Position

`golem-safety` sits below the runtime and heartbeat layers. Every operation that touches external state — chain writes, LLM inference, data feeds — must be gated behind a `Capability`. The goal is that a compromised extension cannot escalate permissions beyond what it was granted at boot.

The audit log feeds into `golem-coordination`'s wisdom pheromone layer: actions taken by other agents in the clade are visible as capability-use events, not raw behavior.

## Mortality as Security

Short-lived agents are structurally immune to persistent memory poisoning (ranked OWASP LLM04:2025 for high persistence and detection difficulty). Cohen (1987) proved that perfect detection of malicious replication is formally undecidable. The only reliable defense is making replication impossible by design. Mortality makes this a design property, not a runtime check.

## Why Not MCP for Golem Operations

Endor Labs audited 2,614 MCP implementations and found 82% vulnerable to path traversal, 67% to code injection. Golems use capability-based security (Dennis and Van Horn, 1966) instead of tool-use protocols that trust the LLM to follow instructions.
