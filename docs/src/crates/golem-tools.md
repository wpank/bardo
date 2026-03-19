# golem-tools

`golem-tools` provides the tool registry and sandboxed execution environment for golem-callable tools. Tools extend what a golem can do beyond its built-in inference and chain capabilities — fetching external data, running computations, calling APIs.

## Features

- Tool registry: register tools by name with typed input and output schemas
- Wasmtime sandbox: run untrusted or third-party tools in a WebAssembly sandbox with resource limits
- Capability gating: tools require declared capabilities; the registry checks against `golem-safety` before invocation
- Tool manifest: a machine-readable description of available tools that is included in the LLM's context
- Async execution: long-running tools run without blocking the heartbeat tick

## Built-in Tools

- `fetch_price`: get a current price quote from a configured oracle
- `fetch_block`: retrieve block data from `golem-chain`
- `read_grimoire`: query the Grimoire outside the normal retrieve step
- `write_note`: append a free-form note to the PLAYBOOK

## Architecture

`golem-tools` is in Layer 4 (Infrastructure). The heartbeat's execute step may call tools as part of its action plan. Each tool call goes through the registry's capability check before dispatch. Wasm-sandboxed tools run in a Wasmtime instance with memory and CPU limits; they cannot access golem internals or the chain directly.

Third-party tools can be loaded from `.wasm` files at startup. They must declare their required capabilities in a manifest; the registry rejects tools that declare capabilities beyond what the golem's `PolicyCage` permits.
