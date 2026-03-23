# Coding Conventions [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-08

> **Reader orientation:** This document defines coding conventions enforced across the Bardo codebase (TypeScript, Solidity, and Rust). It belongs to the **Monorepo** section. The key concept before diving in: these conventions are designed for DeFi agent code that must be auditable and safe by construction, with specific patterns for the Golem (mortal autonomous DeFi agent) runtime, Solidity vault contracts, and the tool system. Terms like Golem, Grimoire, and PolicyCage are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

---

## Overview

This document defines the coding conventions enforced across the Bardo monorepo. These conventions ensure consistency across 20+ workspace packages, prevent common DeFi-specific bugs, and establish patterns for AI agent code that must be auditable and safe by construction.

---

## TypeScript Conventions

### Strict Mode

All packages use TypeScript 5.8+ with `strict: true` and `noUncheckedIndexedAccess: true`. The shared config at `@bardo/typescript-config/base.json` enforces these flags.

### `erasableSyntaxOnly` (TS 5.8+)

Node.js 22+ supports `--experimental-strip-types`, which strips TypeScript syntax at startup without a transpiler. `erasableSyntaxOnly: true` ensures all TypeScript files can run directly via `tsx` or `node --experimental-strip-types`.

**Allowed:** Type annotations, interfaces, type aliases, `as` casts
**Forbidden:** `enum` (use `as const` objects), `namespace`, constructor parameter properties

```typescript
// WRONG: parameter property
class Vault {
  constructor(private readonly address: Address) {}
}

// RIGHT: explicit field + assignment
class Vault {
  readonly #address: Address;
  constructor(address: Address) {
    this.#address = address;
  }
}
```

### `verbatimModuleSyntax`

All type-only imports must use `import type` syntax. No import elision ambiguity.

```typescript
// WRONG
import { Address, getClient } from "@bardo/chain";

// RIGHT
import type { Address } from "@bardo/chain";
import { getClient } from "@bardo/chain";
```

### `bundler` Module Resolution

All packages use `module: "esnext"` + `moduleResolution: "bundler"`. Extension-free imports throughout. tsup handles dual ESM+CJS output; the `exports` field in `package.json` controls consumer resolution.

### Type Safety

- Prefer `unknown` over `any`; narrow types explicitly
- Always handle promise rejections; use `Promise.allSettled` for parallel independent calls
- Catch specific error types with actionable messages including DeFi context
- Use optional chaining for DOM queries (`?.textContent`)
- Use `vi.fn()` for noop callbacks (not `() => {}`)
- Use type assertions (`as HTMLElement`) instead of non-null assertions (`!`)

### Address and Chain Validation

- Never hardcode addresses inline -- use shared constants with per-chain maps
- Always validate addresses before on-chain calls
- Always validate chain IDs before on-chain calls
- All on-chain calls must handle reverts and gas estimation gracefully

```typescript
// WRONG: hardcoded address
const permit2 = "0x000000000022D473030F116dDEE9F6B43aC78BA3";

// RIGHT: shared constant
import { PERMIT2_ADDRESS } from "@bardo/chain";
```

### Error Handling

Use structured error codes from the error hierarchy. Every error includes:

- Machine-readable error code (e.g., `TOKEN_NOT_FOUND`, `AGENT_NOT_REGISTERED`)
- Human-readable message with DeFi context
- Suggested action when possible

```typescript
import { BardoError } from "@bardo/core";

throw new BardoError("AGENT_NOT_REGISTERED", {
  message: `Agent at ${address} is not registered in the ERC-8004 Identity Registry`,
  suggestion:
    "Register the agent first using vault_register_agent or the portal wizard",
});
```

### Module-Level Side Effects

Never capture `globalThis.fetch` or other globals at module level. MSW patches fetch after module evaluation, so module-level captures break test mocks. Use lazy accessors instead.

```typescript
// WRONG: breaks MSW
const fetch = globalThis.fetch;

// RIGHT: lazy accessor
function getFetch() {
  return globalThis.fetch;
}
```

---

## Solidity Conventions

### General

- Solidity `^0.8.26` for all new contracts
- Checks-Effects-Interactions pattern strictly
- Custom errors (not revert strings) for gas efficiency
- All external-facing functions must have NatSpec
- Never use `tx.origin` for authorization
- Use `SafeCast` for all uint downcasts

### Vault-Specific

- Virtual shares offset (`_decimalsOffset()`) set to 3-6 for all factory-deployed vaults (inflation attack prevention)
- All adapters must implement `forceDeallocate()` for guaranteed non-custodial exits
- Use CREATE2 for deterministic addresses (vault factory, hook deployment)
- Hook address mining via `HookMiner` for V4 hook permission bits

### Fee Enforcement

Immutable caps enforced at the contract level:

| Fee Type    | Cap             | Enforcement                                          |
| ----------- | --------------- | ---------------------------------------------------- |
| Management  | 500 bps (5%/yr) | Factory constructor rejects higher values            |
| Performance | 5,000 bps (50%) | Factory constructor rejects higher values            |
| Protocol    | 0 bps           | No protocol fee extraction; vault creators keep 100% |

### Testing

- `testFuzz_*` functions with meaningful lower bounds (`totalAssets >= 1e18`, `feeBps >= 10`)
- a16z ERC-4626 property test suite with `_delta_ = 1e6` for `decimalsOffset=6`
- Slither + Aderyn static analysis with zero high-severity finding policy
- Formal verification targets via Certora and Halmos

---

## Bardo Tool Conventions

### File Organization

Each tool is a single file exporting a `TOOL_DEF: ToolDef` constant:

```typescript
import { z } from "zod";
import type { ToolDef } from "../types.js";

export const TOOL_DEF: ToolDef = {
  name: "get_pool_info",
  description: "Get detailed information about a Uniswap pool",
  category: "data",
  inputSchema: {
    poolAddress: z.string().describe("Pool contract address"),
    chain: z.string().describe("Chain name or chain ID"),
  },
  handler: async (params, ctx) => {
    // Implementation
    return {
      content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
    };
  },
};
```

### Naming

- Format: `verb_noun` with underscores
- Data tools: `get_*`, `search_*`, `list_*`
- Write tools: `execute_*`, `submit_*`, `vault_deposit`, `vault_withdraw`
- Vault tools: `vault_*`, `create_vault`, `list_vaults`
- Safety tools: `simulate_*`, `check_*`, `validate_*`
- Proxy tools: `proxy_*`

### Injectable Dependencies

- Transaction executor: `setTxExecutor(fn)` / `resetTxExecutor()` via DI pattern
- Chain client: `getClient(chainId)` from `@bardo/chain` (not `getPublicClient`)
- Permit2 tools return typed data only -- not actual signatures

---

## CLI Output Conventions

### No Emojis

**All CLI and TUI output MUST use Unicode symbols/dingbats only** (U+0000-U+FFFF BMP range). No emojis. Icons are defined in `packages/tui/src/icons.ts` as the single source of truth.

```typescript
// WRONG
console.log("✅ Deploy complete"); // emoji

// RIGHT
import { ICONS } from "@bardo/tui";
console.log(`${ICONS.check} Deploy complete`); // Unicode checkmark
```

### TTY Awareness

All TUI output must gracefully degrade for non-TTY environments (CI, piped output). Spinners become static progress lines. Colors are stripped. Box drawing falls back to ASCII.

### JSON Mode

All CLI subcommands support `--json` for machine-readable output:

- Suppress all TUI formatting (no spinner, no banner, no color)
- Output structured JSON to stdout
- Errors output JSON to stderr with `{ "error": "...", "code": "..." }`
- Exit codes: 0 (success), 1 (error), 2 (user cancelled)

---

## Agent and Skill Conventions

### Agents

- **Naming**: noun-role format (`vault-manager`, `trade-executor`, `pool-researcher`)
- **Safety-first**: Every write operation routes through `safety-guardian` before broadcast
- **Acyclic delegation**: Agent A -> B -> A is prohibited; the delegation graph is a DAG
- **No transitive trust**: Agents independently validate all inputs via Bardo tools
- **Terminal nodes**: 8 agents that never delegate to other agents
- **Max delegation depth**: 3 (enforced by DAG validation in CI)

### Skills

- **Naming**: verb-noun format (`create-vault`, `execute-swap`, `analyze-pool`)
- **Directory-per-skill**: Each skill is a directory with a `SKILL.md` file
- **3-field frontmatter**: `description` (string), `allowed-tools` (comma-separated or array), `model` ("opus" or "sonnet")
- **Agent coverage**: Every agent must be invoked by at least one skill

### Definitions Location

Agents and skills live in their owning package, colocated with the code they operate on:

| Content          | Location                       |
| ---------------- | ------------------------------ |
| Core agents (25) | `packages/definitions/agents/` |
| Vault agents (7) | `packages/vault/agents/`       |
| Core skills (61) | `packages/definitions/skills/` |
| Vault skills (7) | `packages/vault/skills/`       |

---

## Git Conventions

### Branch Naming

```text
<batch-number>/<feature-name>
```

Examples: `01/foundation`, `02/devenv`, `13/agents`, `15/contracts`

### Commit Message Format

Conventional Commits enforced by commitlint:

```text
type(scope): description

feat(vault): add ERC-4626 deposit tool
fix(tools): handle missing pool address in get_pool_info
chore(deps): update viem to 2.30.0
refactor(dev): merge devenv + webenv into single package
```

### Allowed Types

`feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `revert`

---

## Golem Conventions

### Mortality-Aware Code

Golem agents have finite lifespans. Code that interacts with Golems must handle the case where an agent may have died (USDC balance zero, Hayflick limit reached, or staleness trigger).

```typescript
// Always check agent liveness before interaction
const isAlive = await golem.isAlive();
if (!isAlive) {
  // Handle graceful degradation, not crash
  return { status: "agent_expired", lastHeartbeat: golem.lastHeartbeat };
}
```

### Grimoire Access Patterns

The Grimoire (unified knowledge store) uses typed knowledge items. All access goes through the Grimoire API -- never raw database queries.

| Item Type   | Purpose                            | Mutability                   |
| ----------- | ---------------------------------- | ---------------------------- |
| Episode     | Concrete past experience           | Append-only                  |
| Insight     | Derived conclusion from episodes   | Updatable (confidence decay) |
| Heuristic   | Action rule with conditional       | Updatable                    |
| Warning     | Negative lesson                    | Append-only                  |
| Causal Link | Observed cause-effect relationship | Append-only                  |

### Clade Protocol

Peer-to-peer sibling sync uses REST APIs with no central service. Golems discover siblings via the ERC-8004 registry and share Grimoire items filtered by relevance.

---

## Environment Variables

### Naming Convention

All Bardo-specific environment variables use the `BARDO_` prefix:

| Variable                 | Purpose                      | Default               |
| ------------------------ | ---------------------------- | --------------------- |
| `BARDO_PROFILE`          | Tool profile for Bardo tools | `data`                |
| `BARDO_TRANSPORT`        | A2A transport mode           | `stdio`               |
| `BARDO_DEFERRED_LOADING` | Enable deferred tool loading | `true` when >20 tools |
| `BARDO_MAX_ACTIVE_TOOLS` | Session cap for active tools | `40`                  |
| `BARDO_CORE_TOOLS`       | Override default core tools  | (see tool docs)       |

### Config File Location

User configuration lives at `~/.bardo/config.json` with `0o600` permissions (owner read/write only).

### Environment File Separation

| File          | Package        | Purpose                                               |
| ------------- | -------------- | ----------------------------------------------------- |
| `.env.devenv` | `packages/dev` | Devenv-specific config (fork URL, block number, mode) |
| `.env`        | Root           | Shared config loaded by `dotenv-cli`                  |

These files MUST NOT be merged. The devenv layer reads `.env.devenv`; the tools/vault layers read `.env`.

---

## Rust Conventions

### Error Handling

- **Library crates** (`golem-chain`, `golem-grimoire`, etc.): Use `thiserror` for structured, typed error enums. Each crate defines its own error type. Callers can match on variants without string parsing.
- **Application binaries** (`golem-cli`, `golem-runtime`): Use `anyhow` for flexible error propagation with `.context("what was happening")` annotations. Binary entry points convert to user-facing messages at the top level.

```rust
// Library crate
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("agent {agent_id} not found in ERC-8004 registry")]
    AgentNotFound { agent_id: Address },
    #[error("RPC call failed: {0}")]
    Rpc(#[from] alloy::transports::TransportError),
}

// Application binary
fn main() -> anyhow::Result<()> {
    let agent = registry.lookup(addr).context("looking up agent identity")?;
    Ok(())
}
```

### Async

`tokio` is the async runtime everywhere. Default to `async fn`. Never call blocking I/O (filesystem, DNS, heavy computation) inside an async context without `tokio::task::spawn_blocking`. Use `tokio::select!` for cancellation-aware concurrent operations.

### Serialization

`serde` with derive macros on all data types that cross a boundary (disk, network, IPC). Use `#[serde(rename_all = "camelCase")]` for JSON API surfaces so Rust structs stay snake_case internally while producing camelCase JSON that matches TypeScript conventions.

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultState {
    pub total_assets: U256,
    pub share_price: U256,
    pub is_paused: bool,
}
```

### EVM Interaction

`alloy` exclusively. Do not use ethers-rs. ABI types via `alloy-sol-types` with the `sol!` macro for compile-time ABI generation. Contract instances via `alloy-contract`.

```rust
use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IIdentityRegistry {
        function getAgent(uint256 agentId) external view returns (address, string memory);
    }
}
```

### Testing

- **Test runner**: `cargo nextest` for parallel execution and better output. CI runs `cargo nextest run --all`.
- **Property tests**: `proptest` for randomized input testing, same role as Forge fuzz tests on the Solidity side.
- **HTTP mocking**: `wiremock` for deterministic HTTP mock servers in integration tests. Replaces MSW on the Rust side.
- **Snapshot tests**: `insta` for snapshot assertions where output format matters (serialized configs, error messages).

### Workspace Dependency Inheritance

All shared dependencies live in the root `Cargo.toml` under `[workspace.dependencies]`. Individual crates reference them with `dep.workspace = true`. This guarantees a single version of each dependency across the workspace.

```toml
# Root Cargo.toml
[workspace.dependencies]
alloy = { version = "1.0", features = ["full"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
anyhow = "1"

# Crate Cargo.toml
[dependencies]
alloy.workspace = true
tokio.workspace = true
```

### Lint Config

Clippy pedantic mode enabled at the workspace level with selective allows for rules that produce more noise than signal:

```toml
# Root Cargo.toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
```

### Naming

| Element     | Convention           | Example                          |
| ----------- | -------------------- | -------------------------------- |
| Functions   | `snake_case`         | `lookup_agent`, `fetch_vault`    |
| Variables   | `snake_case`         | `total_assets`, `agent_id`       |
| Types       | `CamelCase`          | `VaultState`, `RegistryError`    |
| Constants   | `SCREAMING_SNAKE`    | `MAX_FEE_BPS`, `REGISTRY_ADDR`  |
| Crate names | `kebab-case`         | `golem-chain`, `golem-grimoire`  |
| Modules     | `snake_case`         | `registry`, `vault_manager`      |

### Visibility

`pub(crate)` by default. Only mark items `pub` when they are part of the crate's public API surface. Internal helpers, utility functions, and implementation details stay `pub(crate)` or private. This keeps the public API small and auditable.

### Generics vs Trait Objects

Prefer generics with `impl Trait` over `dyn Trait` where the concrete type is known at compile time. Use `dyn Trait` only when you need runtime polymorphism (heterogeneous collections, plugin systems). Boxing trait objects (`Box<dyn Error>`) is acceptable at application boundaries but not in library APIs.

```rust
// Prefer: monomorphized, zero-cost
fn register<T: Transport>(transport: &T) -> Result<AgentId, RegistryError> { ... }

// Acceptable at boundaries: dynamic dispatch
fn handlers() -> Vec<Box<dyn EventHandler>> { ... }
```

---

## Cross-Language Boundaries

The Bardo stack spans three languages. Each owns a distinct layer:

| Layer           | Language   | Packages                             | Build Tool |
| --------------- | ---------- | ------------------------------------ | ---------- |
| Golem runtime   | Rust       | `golem-*` crates                     | Cargo      |
| Tool server     | TypeScript | `@bardo/*` packages                  | tsup       |
| Smart contracts | Solidity   | `packages/vault/contracts/`, etc.    | Foundry    |

### MCP Tool Calls

The Golem runtime (Rust) invokes TypeScript MCP tools via JSON-RPC over stdio. The `golem-mcp` crate serializes tool call requests as JSON-RPC 2.0 messages, pipes them to the Bardo tool server process, and deserializes the response. Tool names, parameter schemas, and response shapes are the contract -- both sides must agree on them.

### Contract Interaction

Both Rust and TypeScript interact with the same on-chain contracts using the same ABIs:

- **Rust**: `alloy-sol-types` with the `sol!` macro generates typed bindings from Solidity interfaces
- **TypeScript**: `viem` reads ABI JSON artifacts and provides typed contract instances

The ABIs are the single source of truth. Foundry compiles contracts to `out/` artifacts; both languages consume these artifacts (TypeScript directly, Rust via codegen or manual `sol!` declarations).

### Configuration

| Config file      | Format | Owner      | Purpose                                     |
| ---------------- | ------ | ---------- | ------------------------------------------- |
| `golem.toml`     | TOML   | Rust       | Golem runtime config (identity, RPC, clade) |
| `package.json`   | JSON   | TypeScript | Package metadata, scripts, dependencies     |
| `tsconfig.json`  | JSON   | TypeScript | TypeScript compiler options                 |
| `foundry.toml`   | TOML   | Solidity   | Forge compiler and test configuration       |
| `Cargo.toml`     | TOML   | Rust       | Crate metadata, dependencies, workspace     |

TOML is the native config format for Rust crates. TypeScript packages continue using JSON. There is no shared config file between the two runtimes -- they communicate exclusively through MCP JSON-RPC messages and shared on-chain state.
