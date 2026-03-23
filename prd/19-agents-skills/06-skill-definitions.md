# Pi workflow definitions [SPEC]

> **Cross-ref**: [04-skills-overview.md](04-skills-overview.md) (interaction surfaces, operator directives, and Pi Skills overview), [05-skill-categories.md](05-skill-categories.md) (full inventory of 68 interaction surfaces across 13 categories)

> **Reader orientation:** This document specifies the Pi workflow definition format: typed handlers registered with the Golem's (mortal autonomous agent's) RPC interface or triggered by the Heartbeat (9-step decision cycle) FSM. It belongs to Section 19 (Agents & Skills) and covers the `WorkflowDefinition` interface, parameter schemas, trigger types (rpc, heartbeat, event, chat, schedule), and reference documentation loading. See `prd2/shared/glossary.md` for full term definitions.

---

## Workflow structure

Each interaction surface maps to a Pi workflow: a typed handler registered with the golem's RPC interface or triggered by the heartbeat FSM. Workflows are defined in `golem-tools/src/workflows/`.

### WorkflowDefinition interface

```typescript
interface WorkflowDefinition {
  /** Unique identifier, kebab-case */
  name: string;

  /** Human-readable description */
  description: string;

  /** Primary archetype that handles this workflow */
  primaryArchetype: string;

  /** Additional archetypes (for composite workflows) */
  secondaryArchetypes?: string[];

  /** Input parameters */
  params: WorkflowParam[];

  /** Which interaction surfaces can trigger this workflow */
  triggers: WorkflowTrigger[];
}

interface WorkflowParam {
  name: string;
  type: "string" | "number" | "boolean" | "address" | "bigint";
  required: boolean;
  default?: unknown;
  description: string;
}

type WorkflowTrigger =
  | "rpc"        // External RPC call
  | "heartbeat"  // Heartbeat tick (golem decides autonomously)
  | "event"      // Event bus trigger (price alert, health factor)
  | "chat"       // Chat surface (Discord, Telegram, web)
  | "schedule";  // Cron-based trigger
```

### Example: execute-swap workflow

```typescript
export const executeSwap: WorkflowDefinition = {
  name: "execute-swap",
  description: "Execute a token swap on any supported chain",
  primaryArchetype: "trade-executor",
  params: [
    {
      name: "tokenIn",
      type: "address",
      required: true,
      description: "Token to sell",
    },
    {
      name: "tokenOut",
      type: "address",
      required: true,
      description: "Token to buy",
    },
    {
      name: "amount",
      type: "bigint",
      required: true,
      description: "Amount of tokenIn to sell",
    },
    {
      name: "chainId",
      type: "number",
      required: false,
      default: 8453,
      description: "Target chain",
    },
    {
      name: "maxSlippage",
      type: "number",
      required: false,
      default: 0.005,
      description: "Maximum slippage tolerance (0.005 = 0.5%)",
    },
  ],
  triggers: ["rpc", "heartbeat", "chat"],
};
```

---

## Reference documentation

Complex workflows that interact with deep protocol surfaces have reference documentation in `golem-tools/src/workflows/references/`. These documents are loaded into the Pi session context when the workflow is active.

| Workflow             | Reference topics                                                       | File count |
| -------------------- | ---------------------------------------------------------------------- | ---------- |
| `execute-swap`       | Trading API flow, Permit2, WETH on L2s, smart account patterns         | 4          |
| `manage-liquidity`   | V3 concentrated liquidity math, V4 position actions, tick arithmetic   | 3          |
| `build-hook`         | V4 hook lifecycle, permission flags, delta accounting, CREATE2 mining  | 4          |
| `cross-chain-swap`   | ERC-7683 intent structure, bridge monitoring, destination confirmation | 3          |
| `deploy-agent-token` | CCA mechanics, supply schedule math, factory deployment, LP locking    | 4          |

**Total**: 18 reference files across 5 workflows.

---

## Workflow validation

At boot, workflows are validated:

- All `primaryArchetype` references resolve to registered archetypes
- All `secondaryArchetypes` references resolve
- Parameter types match serde schemas
- No duplicate workflow names
- Reference docs exist on disk if referenced

---
