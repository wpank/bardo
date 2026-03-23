# Pi tool integration [SPEC]

> **Cross-ref**: [00-agents-overview.md](00-agents-overview.md) (architecture overview for archetypes and tool profiles), [03-delegation.md](03-delegation.md) (tool resolution pipeline and delegation DAG), [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Pi extension loading and tool registration)

> **Reader orientation:** This document specifies how the `bardo-tools` extension bridges 190+ tool implementations in `golem-tools` with the Pi agent session. It belongs to Section 19 (Agents & Skills) and covers the tool resolution pipeline (category filtering, override application, Pi adapter conversion, two-layer registration), the profile system, capability gating by BehavioralPhase (one of five survival phases: Thriving/Stable/Conservation/Desperate/Terminal), spending limits, and PolicyCage (on-chain smart contract enforcing safety constraints) enforcement. See `prd2/shared/glossary.md` for full term definitions.

---

## How the Pi runtime discovers tools

The `bardo-tools` extension is the bridge between `golem-tools` (190+ tool implementations) and the Pi agent session. It loads at boot, resolves which tools the golem needs based on its archetype and profile, converts `ToolDef` objects to Pi-native tool definitions, and registers them with the session.

There is no MCP server. No stdio transport. No HTTP transport. Tools are loaded in-process, registered directly with Pi's tool system, and called through Pi's `tool_call` hook infrastructure.

```text
golem-tools (190+ ToolDef objects)
  |
  v
bardo-tools extension (profile-based filtering)
  |  - Reads archetype.toolCategories
  |  - Filters ALL_TOOL_DEFS by category
  |  - Applies toolOverrides (include/exclude)
  |  - Converts ToolDef -> PiToolDefinition
  |
  v
Pi Session (8 Pi-facing tools registered)
  |  - preview_action, commit_action, cancel_action
  |  - query_state, search_context, query_grimoire
  |  - update_directive, emergency_halt
  |
  v
bardo-safety extension (policy enforcement)
  |  - PolicyCage validation on every tool_call
  |  - Spending limits, simulation gates
  |  - Phase-based action blocking
  |
  v
bardo-permits extension (action lifecycle)
     - ActionPermit: preview -> commit
     - Simulation hash verification
     - TTL enforcement
```

---

## Tool resolution pipeline

### Step 1: Category filtering

Each `ToolDef` in `golem-tools` has a `category` field:

| Category           | Example tools                                            |
| ------------------ | -------------------------------------------------------- |
| `data`             | `get_pool_info`, `get_token_price`, `search_tokens`      |
| `trading`          | `get_quote`, `execute_swap`, `simulate_transaction`      |
| `lp`               | `add_liquidity`, `remove_liquidity`, `collect_fees`      |
| `vault`            | `vault_deposit`, `vault_withdraw`, `vault_rebalance`     |
| `safety`           | `check_safety_status`, `simulate_transaction`            |
| `identity`         | `register_agent`, `query_reputation`                     |
| `memory`           | `search_memory`, `store_episode`, `get_insights`         |
| `streaming`        | `subscribe_price_feed`, `subscribe_trades`               |
| `intelligence`     | `compute_vpin`, `compute_lvr`, `optimize_lp_range`       |
| `fees`             | `get_tokenjar_balances`, `execute_burn`                  |
| `self_improvement`  | `record_execution`, `record_outcome`, `tune_parameters`  |
| `hook`             | `hook_discover`, `hook_evaluate`, `get_hook_template`    |
| `testnet`          | `setup_local_testnet`, `deploy_mock_pool`, `time_travel` |

The archetype declares which categories it needs:

```typescript
// trade-executor archetype
toolCategories: ["data", "trading", "safety"]
```

The extension filters:

```typescript
const tools = ALL_TOOL_DEFS.filter((t) =>
  archetype.toolCategories.includes(t.category),
);
```

### Step 2: Override application

Archetypes can fine-tune their tool set:

```typescript
toolOverrides: {
  exclude: ["execute_burn"],  // Don't need fee tools
  include: ["compute_vpin"],  // Need this one intelligence tool
}
```

### Step 3: Pi adapter conversion

Each `ToolDef` is converted to a Pi-native tool definition. The Pi tool wraps the `ToolDef.handler` with context injection, error handling, and event emission:

```typescript
function toPiTool(toolDef: ToolDef, ctx: GolemToolContext): PiToolDefinition {
  return {
    name: toolDef.name,
    description: toolDef.description,
    inputSchema: toolDef.inputSchema,
    handler: async (params: Record<string, unknown>) => {
      const result = await toolDef.handler(params, ctx);
      ctx.eventBus.emit({
        kind: "runtime",
        name: "ToolCallAttempt",
        payload: { toolName: toolDef.name, params, result },
      });
      return result;
    },
  };
}
```

### Step 4: Two-layer registration

The 190+ concrete tools are not exposed directly to the LLM. Instead, they back 8 Pi-facing tools. The `bardo-tools` extension (in `golem-tools`) maps action kinds to concrete tool calls:

```typescript
const ACTION_KIND_MAP: Record<string, string[]> = {
  swap: ["get_quote", "simulate_transaction", "execute_swap"],
  "add-liquidity": ["get_pool_info", "add_liquidity", "simulate_transaction"],
  "vault-deposit": [
    "vault_simulate_deposit",
    "vault_deposit",
    "simulate_transaction",
  ],
  // ... 40+ action kinds
};
```

When the LLM calls `preview_action({ kind: "swap", params: {...} })`, the extension resolves `kind: "swap"` to the appropriate concrete tools and executes them in sequence.

---

## Profile system

Profiles are named bundles of tool categories. They exist as a convenience -- operators and configs reference profiles instead of listing individual categories.

| Profile        | Categories                                 | Tool count |
| -------------- | ------------------------------------------ | ---------- |
| `data`         | data, identity                             | ~46        |
| `trader`       | data, trading, safety                      | ~57        |
| `lp`           | data, trading, lp, safety                  | ~71        |
| `vault`        | data, vault, safety                        | ~40        |
| `fees`         | data, fees                                 | ~25        |
| `erc8004`      | data, identity                             | ~28        |
| `intelligence` | data, intelligence                         | ~38        |
| `observatory`  | data, intelligence, identity, memory       | ~72        |
| `learning`     | data, memory, self_improvement             | ~30        |
| `dashboard`    | data, intelligence                         | ~38        |
| `full`         | all categories                             | ~186       |
| `dev`          | all + testnet                              | ~190       |

Profile resolution:

```typescript
function resolveProfile(profile: ProfileName): ToolCategory[] {
  return PROFILE_CATEGORY_MAP[profile];
}
```

---

## Capability gating

The `bardo-safety` extension applies capability gating on every `tool_call` hook:

### Phase-based gating

Tools are classified by action class. The golem's current behavioral phase (Thriving, Cautious, Defensive, Survival, Terminal) determines which action classes are allowed:

| Action class       | Thriving | Cautious | Defensive | Survival | Terminal |
| ------------------ | -------- | -------- | --------- | -------- | -------- |
| `new-position`     | yes      | yes      | no        | no       | no       |
| `increase-position`| yes      | yes      | no        | no       | no       |
| `rebalance`        | yes      | yes      | yes       | no       | no       |
| `decrease-position`| yes      | yes      | yes       | yes      | no       |
| `close-position`   | yes      | yes      | yes       | yes      | yes      |
| `read-only`        | yes      | yes      | yes       | yes      | yes      |

### Spending limits

The `bardo-safety` extension enforces per-transaction and per-day spending limits:

```typescript
interface SpendingLimits {
  perTransaction: bigint; // Max USDC per single tx
  perDay: bigint; // Max USDC per 24h rolling window
  hotLane: bigint; // Max USDC without safety-guardian delegation
}
```

Transactions above the hot lane limit must go through the safety-guardian archetype for independent validation.

### PolicyCage enforcement

On-chain PolicyCage constraints (approved assets, max position sizes, max drawdown, rebalance frequency) are checked before every write operation. These constraints cannot be overridden by the LLM, the archetype, or the operator. They are enforced at the smart contract level. The `bardo-safety` extension validates them pre-flight to avoid wasting gas on operations that would revert on-chain.

---

## Injectable transaction executor

The `executeTx()` function uses dependency injection:

- `setTxExecutor(fn)` / `resetTxExecutor()` -- configure the transaction execution function
- Default throws `WALLET_NOT_CONFIGURED`
- Real wallet integration provides the concrete implementation via `bardo-custody` extension
- This allows tools to construct transactions without knowing the wallet implementation

---

## Implementation notes

- **Chain client**: Use `GolemChain::get_client(chain_id)` from the `golem-chain` crate
- **Permit2 tools return typed data only** -- not actual signatures. Signing requires wallet integration through `bardo-custody`.
- **V3/V4 auto-detection**: Uses `UNISWAP_CONTRACTS[chainId].v4PositionManager` presence check
- **No `@uniswap/smart-order-router` dependency**: SDK fallback is injectable but defaults to throwing `SDK_FALLBACK_UNAVAILABLE`
- **Each tool file exports `TOOL_DEF: ToolDef`** -- no registration function. The `bardo-tools` extension reads `ALL_TOOL_DEFS` and handles registration.

---
