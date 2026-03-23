# golem-safety

**Status: shell — no public API yet.**

Security and capability enforcement layer for the Golem runtime. The planned design treats authorization as a type-level problem: you cannot call a sensitive operation without holding an unforgeable token, and the compiler enforces it.

## Planned Public API

**`Capability<T>`** — unforgeable security token parameterized on the permission type `T`. A `Capability<WriteChain>` is required to submit transactions; a `Capability<CallLlm>` to make inference calls. Capabilities are created only through the safety registry at boot time and cannot be cloned or forged at runtime. This is the core design: ambient authority is replaced with explicit capability passing.

**`TaintedString`** — string wrapper with provenance tracking. Marks data that arrived from external sources (API responses, chain data, user input) and requires explicit sanitization or review before it can flow into privileged operations. Prevents accidental injection through the type system.

**`PolicyCage`** — sandboxing boundary for extension execution. Extensions run inside a `PolicyCage` that limits which `Capability` types they can acquire and what resources they can access. The cage is constructed by the runtime with a policy derived from the extension's declared layer.

**`ActionPermit`** — lifecycle token for a single action. Issued before an action executes, consumed on completion or cancellation. Prevents double-execution and provides an audit hook.

**`LoopGuard`** — recursion depth counter with configurable hard limit. Wraps re-entrant operations and panics (or returns an error) if depth exceeds the configured maximum. Prevents unbounded recursion in agent reasoning loops.

**Merkle audit log** — append-only log of capability use, signed with a Merkle commitment. Each `ActionPermit` use appends an entry; the root hash can be verified by external auditors or committed on-chain.

## System Position

`golem-safety` sits below the runtime and heartbeat layers. Every operation that touches external state — chain writes, LLM inference, data feeds — must be gated behind a `Capability`. The goal is that a compromised extension cannot escalate permissions beyond what it was granted at boot.

The audit log feeds into `golem-coordination`'s wisdom pheromone layer: actions taken by other agents in the clade are visible as capability-use events, not raw behavior.
