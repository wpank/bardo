# Pi tool adapter resolution [SPEC]

> **Cross-ref**: [00-agents-overview.md](00-agents-overview.md) (architecture overview for archetypes and tool profiles), [02-agent-definitions.md](02-agent-definitions.md) (TOML + Rust archetype definition format), [08-mcp-integration.md](08-mcp-integration.md) (bardo-tools extension bridge and in-process tool loading), [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Pi extension loading and tool registration)

> **Reader orientation:** This document specifies how Golems (mortal autonomous agents) discover, resolve, and delegate to tools through a two-layer adapter model. It belongs to Section 19 (Agents & Skills) and covers the three-step adapter resolution pipeline (DECLARE -> RESOLVE -> ADAPT), the delegation DAG (Directed Acyclic Graph) with max depth 3, and terminal node rules. The LLM sees 8 Pi-facing tools; behind them, the adapter routes to 190+ concrete `golem-tools` implementations. See `prd2/shared/glossary.md` for full term definitions.

---

## How golems discover and use tools

Golems do not call tools by name from a static registry. The `bardo-tools` Pi extension resolves tools at session initialization through a three-step adapter resolution pipeline:

```text
1. DECLARE  — Archetype declares tool categories it needs
2. RESOLVE  — bardo-tools filters ALL_TOOL_DEFS by category membership
3. ADAPT    — Tool definitions are converted to Pi-native format and registered
```

The golem's LLM sees a thin set of Pi-facing tools (8 tools). Behind each Pi tool, the adapter layer routes to the concrete `golem-tools` implementation. This two-layer model is described in `../01-golem/13-runtime-extensions.md` S3.

---

## The two-layer tool model

### Layer 1: Pi-facing tools (what the LLM sees)

8 tools registered with the Pi session:

| Pi tool            | Purpose                                                          |
| ------------------ | ---------------------------------------------------------------- |
| `preview_action`   | Simulate an on-chain action, return expected outcome             |
| `commit_action`    | Execute a previously previewed action                            |
| `cancel_action`    | Cancel a pending action                                          |
| `query_state`      | Read on-chain state (balances, positions, pool data)             |
| `search_context`   | Search PLAYBOOK.md, STRATEGY.md, and active state               |
| `query_grimoire`   | Search episodic memory, insights, heuristics                     |
| `update_directive` | Write a delta to PLAYBOOK.md (heuristic update)                  |
| `emergency_halt`   | Trigger emergency shutdown (revoke keys, pause vault, liquidate) |

### Layer 2: Sanctum adapters (what runs behind the scenes)

Each Pi tool resolves to one or more `golem-tools` implementations based on the action parameters:

```text
preview_action({ kind: "swap", params: { tokenIn: "WETH", ... } })
  |
  v
bardo-tools extension resolves:
  -> get_quote (from golem-tools trading module)
  -> simulate_transaction (from golem-tools safety module)
  |
  v
Returns: ActionPermit { permitId, simulationHash, expectedOutcome }
```

The `bardo-permits` extension governs the `preview_action` -> `commit_action` lifecycle. Every write operation requires a valid ActionPermit with a matching simulation hash. The permit expires after a configurable TTL (default 60s).

---

## Adapter resolution rules

### Category-based filtering

When the `bardo-tools` extension loads, it reads the archetype's `toolCategories` array and filters `ALL_TOOL_DEFS`:

```typescript
const allowedTools = ALL_TOOL_DEFS.filter((t) =>
  archetype.toolCategories.includes(t.category),
);
```

Tool overrides are applied after filtering:

```typescript
// Remove excluded tools
const filtered = allowedTools.filter(
  (t) => !archetype.toolOverrides?.exclude?.includes(t.name),
);

// Add individually included tools
const included = ALL_TOOL_DEFS.filter((t) =>
  archetype.toolOverrides?.include?.includes(t.name),
);

const finalTools = [...filtered, ...included];
```

### Profile resolution

Profiles are named bundles of tool categories. The profile system is a convenience layer on top of category-based filtering:

| Profile        | Categories                                          |
| -------------- | --------------------------------------------------- |
| `data`         | data, identity                                      |
| `trader`       | data, trading, safety                               |
| `lp`           | data, trading, lp, safety                           |
| `vault`        | data, vault, safety                                 |
| `fees`         | data, fees                                          |
| `erc8004`      | data, identity                                      |
| `intelligence` | data, intelligence                                  |
| `observatory`  | data, intelligence, identity, memory                |
| `learning`     | data, memory, self_improvement                      |
| `dashboard`    | data, intelligence                                  |
| `full`         | all categories                                      |
| `dev`          | all categories + testnet                            |

An archetype can specify a profile instead of listing categories. The profile is resolved to categories before filtering.

---

## Delegation graph

Archetype-to-archetype delegation forms a Directed Acyclic Graph. These rules are validated at boot by `validate_dag.rs` in the `golem-tools` crate.

### Rules

1. **Acyclic.** No circular delegation. Archetype A delegates to B, but B never delegates back to A (directly or transitively).
2. **Explicit.** Each archetype's `delegatesTo` array lists exactly which archetypes it may invoke.
3. **Safety-first.** Any archetype performing a write operation must delegate safety validation to `safety-guardian` before execution.
4. **Maximum depth: 3.** No delegation chain exceeds 3 hops.

### Delegation hierarchy

```text
Level 0 (Golem heartbeat invokes):
  golem-instance

Level 1 (golem-instance delegates to):
  trade-executor, liquidity-manager, vault-manager,
  vault-strategist, lp-strategist, risk-assessor,
  memory-consolidator

Level 2 (execution archetypes delegate to):
  trade-executor        -> safety-guardian, risk-assessor
  liquidity-manager     -> safety-guardian, risk-assessor, pool-researcher
  cross-chain-executor  -> safety-guardian, risk-assessor, trade-executor
  vault-manager         -> safety-guardian, risk-assessor
  vault-strategist      -> pool-researcher, risk-assessor

Level 3 (terminal nodes -- never delegate):
  safety-guardian, risk-assessor, wallet-provisioner,
  pnl-analyst, pool-researcher, token-analyst,
  identity-verifier, position-monitor,
  heartbeat-monitor, memory-consolidator,
  vault-watchdog, sleepwalker-observer
```

### Terminal nodes (12)

Terminal nodes are leaf archetypes in the delegation DAG. They provide specialized capabilities but never delegate.

| Terminal node        | Category       | Rationale                                                     |
| -------------------- | -------------- | ------------------------------------------------------------- |
| `safety-guardian`    | Infrastructure | Must be independent -- no circular safety dependencies        |
| `risk-assessor`      | Strategy       | Must be independent -- cannot delegate to archetypes it evaluates |
| `wallet-provisioner` | Infrastructure | Sequential setup flow, no need for delegation                 |
| `pnl-analyst`        | Research       | Structured calculation, uses own tools for context            |
| `pool-researcher`    | Research       | Data retrieval only, no execution needed                      |
| `token-analyst`      | Research       | Data retrieval only, no execution needed                      |
| `identity-verifier`  | Economy        | Registry lookup only                                          |
| `position-monitor`   | Real-time      | Alert-only, dispatches notifications                          |
| `heartbeat-monitor`  | Golem          | Observation-only, dispatches health metrics                   |
| `memory-consolidator`| Golem          | Manages Grimoire lifecycle independently                      |
| `vault-watchdog`     | Vault          | Terminal monitoring node, cancel authority                     |
| `sleepwalker-observer`| Observer      | Publishes artifacts, never executes                           |

### Self-contained archetypes (not terminal, but no delegation)

| Archetype               | Category    | Rationale                                   |
| ----------------------- | ----------- | ------------------------------------------- |
| `hook-builder`          | Development | Self-contained code generation              |
| `integration-architect` | Development | Self-contained architecture design          |
| `integration-advisor`   | Development | Advisory only                               |

---

## DAG visualization

```text
                     golem-instance
                    /    |    |    \
                   /     |    |     \
        trade-executor  lm  vault-mgr  memory-consolidator (terminal)
           /    \       |     |    \
          /      \      |     |     \
 safety-guardian  ra   pr    sg     ra
 (terminal)   (terminal)(t)  (t)   (t)

Legend:
  lm = liquidity-manager
  ra = risk-assessor
  pr = pool-researcher
  sg = safety-guardian
  (t) = terminal node
```

---

## Anti-pattern: circular delegation (prohibited)

```text
NEVER:
  archetype-A -> delegates to archetype-B -> delegates to archetype-A

The delegation hierarchy is a DAG.
Topological sort at boot rejects cycles.
```

---
