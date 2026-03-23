# Bardo Tools -- Distribution and Discovery [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md)
>
> Crate distribution, binary packaging, Golem runtime integration, A2A Agent Card discovery, ERC-8004 on-chain registration, and framework interop.

---

> **Reader orientation:** This document specifies how the `bardo-tools` crate is distributed, discovered, and consumed, covering Cargo publishing, A2A Agent Cards, ERC-8004 (on-chain agent identity standard tracking capabilities, milestones, and reputation) registration, framework adapters, and deployment progression. It belongs to Bardo's DeFi tool layer. The key prerequisite is understanding how Golems (mortal autonomous agents compiled as single Rust binaries running on micro VMs) invoke tools in-process via the `ToolRegistry`. See `prd2/shared/glossary.md` for full term definitions.

## Distribution model

`bardo-tools` distributes as a Rust crate published to crates.io. Two binary targets: `golem-tools` (the CLI) and `bardo-a2a` (the A2A endpoint server). External agents access tool capabilities via the A2A protocol, not by embedding the library. Golems consume tools in-process via the `ToolRegistry`.

| Channel              | Artifact / Package         | Consumer                                       |
| -------------------- | -------------------------- | ---------------------------------------------- |
| Cargo crate          | `bardo-tools`              | Golems (in-process via ToolRegistry)           |
| Binary: CLI          | `golem-tools`              | Owner CLI for setup, health checks, key rotation |
| Binary: A2A server   | `bardo-a2a`                | A2A endpoint server for external agents        |
| A2A Agent Card       | Published to ERC-8004 + DNS | External agents (any A2A-compatible agent)     |
| Framework adapters   | `bardo-tools` feature flags | OpenClaw, ElizaOS, GOAT, custom frameworks     |
| Container image      | `ghcr.io/bardo/a2a`       | Docker/Kubernetes deployments                  |

### Golem runtime integration

```rust
use bardo_tools::{ToolRegistry, ToolConfig, CustodyConfig};

// Build the registry for an active Golem
let registry = ToolRegistry::from_config(ToolConfig {
    profile: "active",
    custody: CustodyConfig::Embedded { privy_app_id: "...".into() },
    safety: safety_config,
})?;

// The Golem's heartbeat pipeline calls tools via the registry
let result = registry.call("uniswap_get_quote", params).await?;
```

The registry loads tools matching the profile at startup. Golems invoke tools in-process with zero transport overhead, which is required for the Adaptive Clock's theta-frequency cadence (30-120s).

### Cargo publishing

Published to crates.io as `bardo-tools`. Feature flags control optional dependencies:

| Feature        | Default | Description                                   |
| -------------- | ------- | --------------------------------------------- |
| `full`         | No      | All tool categories enabled                   |
| `trading`      | Yes     | Uniswap swap, quote, approval tools           |
| `lp`           | Yes     | Liquidity provision tools                      |
| `lending`      | No      | Aave, Compound, Morpho tools                  |
| `staking`      | No      | Lido, Rocket Pool tools                        |
| `restaking`    | No      | EigenLayer, Symbiotic tools                    |
| `derivatives`  | No      | GMX, Panoptic tools                            |
| `yield`        | No      | Yearn, Pendle, Ethena tools                    |
| `vault`        | Yes     | ERC-4626 vault tools                           |
| `memory`       | No      | Grimoire memory tools                          |
| `identity`     | No      | ERC-8004 identity tools                        |
| `sidecar`      | Yes     | TypeScript sidecar for Uniswap SDK math       |
| `a2a`          | No      | A2A server binary and transport layer          |

```toml
# Cargo.toml (consumer)
[dependencies]
bardo-tools = { version = "1", features = ["trading", "lp", "vault"] }
```

### SemVer semantics

| Change type                         | Version bump               | Examples                               |
| ----------------------------------- | -------------------------- | -------------------------------------- |
| New tools (additive)                | Minor                      | Adding `get_pool_health_score`         |
| Breaking parameter/response changes | Major                      | Renaming `pool` param to `poolAddress` |
| Bug fixes, performance improvements | Patch                      | Fixing gas estimation accuracy         |
| Tool deprecation                    | Minor + deprecation notice | 90-day sunset window                   |
| Tool removal                        | Major                      | Only after 90-day deprecation          |

### Binary distribution

Two binary targets ship from the `bardo-tools` workspace:

- **`golem-tools`** -- Owner-facing CLI for setup, health checks, key rotation, and profile inspection. Not used by Golems at runtime.
- **`bardo-a2a`** -- A2A endpoint server. Wraps `bardo-tools` handler functions in a JSON-RPC 2.0 server for external agent consumption.

```bash
# Install both binaries via cargo
cargo install golem-tools
cargo install bardo-a2a

# Or download pre-built binaries
curl -fsSL https://get.bardo.run | sh

# Or use container image (A2A server only)
docker run -e BARDO_PROFILE=trader ghcr.io/bardo/a2a:latest
```

Release artifacts (both binaries for each platform):

| Platform                   | Artifacts                                                                  |
| -------------------------- | -------------------------------------------------------------------------- |
| `x86_64-unknown-linux-gnu` | `golem-tools-x86_64-unknown-linux-gnu.tar.gz`, `bardo-a2a-x86_64-unknown-linux-gnu.tar.gz` |
| `aarch64-unknown-linux-gnu`| `golem-tools-aarch64-unknown-linux-gnu.tar.gz`, `bardo-a2a-aarch64-unknown-linux-gnu.tar.gz` |
| `x86_64-apple-darwin`      | `golem-tools-x86_64-apple-darwin.tar.gz`, `bardo-a2a-x86_64-apple-darwin.tar.gz` |
| `aarch64-apple-darwin`     | `golem-tools-aarch64-apple-darwin.tar.gz`, `bardo-a2a-aarch64-apple-darwin.tar.gz` |
| Container                  | `ghcr.io/bardo/a2a:latest`                                                |

---

## A2A Agent Card

External agents discover Bardo tools via an A2A Agent Card [A2A-SPEC-2025]. The card is published to both DNS (`.well-known/agent.json`) and the ERC-8004 registry.

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    pub capabilities: CardCapabilities,
    pub skills: Vec<CardSkill>,
    pub authentication: CardAuth,
    pub default_input_modes: Vec<String>,
    pub default_output_modes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardCapabilities {
    pub streaming: bool,
    pub push_notifications: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardAuth {
    pub schemes: Vec<String>,
}

/// Build the default Bardo Tools agent card.
pub fn bardo_agent_card() -> AgentCard {
    AgentCard {
        name: "Bardo Tools".into(),
        description: "Uniswap protocol access: data, trading, LP, vaults, safety. ~210 tools across 11 chains.".into(),
        url: "https://tools.bardo.run".into(),
        version: "1.0.0".into(),
        capabilities: CardCapabilities {
            streaming: false,
            push_notifications: false,
        },
        skills: vec![
            CardSkill { id: "uniswap-data".into(), name: "Uniswap Data Queries".into(), description: "Token prices, pool info, positions, historical data".into(), tags: vec!["defi".into(), "uniswap".into(), "data".into()] },
            CardSkill { id: "uniswap-trading".into(), name: "Uniswap Trading".into(), description: "Swap execution, quotes, approvals, MEV protection".into(), tags: vec!["defi".into(), "uniswap".into(), "trading".into()] },
            CardSkill { id: "uniswap-lp".into(), name: "Uniswap Liquidity".into(), description: "LP position management, fee collection, optimization".into(), tags: vec!["defi".into(), "uniswap".into(), "liquidity".into()] },
            CardSkill { id: "vault-operations".into(), name: "Vault Operations".into(), description: "ERC-4626 vault management, deposits, withdrawals".into(), tags: vec!["defi".into(), "vault".into(), "erc4626".into()] },
        ],
        authentication: CardAuth {
            schemes: vec!["bearer".into(), "siwe".into()],
        },
        default_input_modes: vec!["application/json".into()],
        default_output_modes: vec!["application/json".into()],
    }
}
```

### A2A endpoint

The A2A endpoint runs as the `bardo-a2a` binary (separate from the Golem runtime). It translates A2A task requests into `bardo-tools` handler invocations and returns structured results.

```text
POST https://tools.bardo.run/a2a
Authorization: Bearer <token>

{
  "jsonrpc": "2.0",
  "method": "tasks/send",
  "params": {
    "id": "task-123",
    "message": {
      "role": "user",
      "parts": [{ "type": "text", "text": "Get WETH price on Ethereum" }]
    }
  }
}
```

---

## ERC-8004 on-chain registration

The Bardo tool infrastructure registers itself in the ERC-8004 Agent Registry for on-chain discoverability. Other agents can query the registry to find its capabilities, endpoint, and trust score.

Registration metadata:

```rust
AgentMetadata {
    name: "Bardo Tools",
    description: "Uniswap protocol access for autonomous agents",
    active: true,
    a2a_endpoint: "https://tools.bardo.run/a2a",
    a2a_version: "0.2.0",
    a2a_skills: vec![
        "uniswap-data", "uniswap-trading", "uniswap-lp", "vault-operations",
    ],
    supported_trusts: vec!["tee-attestation"],
    x402_support: true,
}
```

Agents searching for DeFi capabilities via `identity_search_agents` or `discover_agents` will find Bardo and resolve its A2A endpoint for task submission.

---

## Framework adapters

OpenClaw (188K+ GitHub stars, 5,705+ skills) and ElizaOS have no native Golem runtime integration [OPENCLAW-2025]. Framework adapters let any agent framework call Bardo tool handlers without embedding the full Golem runtime.

| Framework | Integration                                              |
| --------- | -------------------------------------------------------- |
| OpenClaw  | `bardo-tools` feature `openclaw` exports as OpenClaw skill |
| ElizaOS   | `bardo-tools` feature `eliza` exports as Eliza action     |
| GOAT      | `bardo-tools` feature `goat` exports as GOAT tool          |
| Custom    | Direct import from `bardo-tools`                           |

Framework adapters are thin wrappers that map `ToolDef` handlers to the target framework's tool/skill/action interface. Each adapter creates a `ToolContext` from the framework's native configuration and delegates to the same handler functions Golems use.

```rust
// OpenClaw adapter (simplified)
#[cfg(feature = "openclaw")]
pub fn to_openclaw_skill(tool: &ToolDef) -> OpenClawSkill {
    OpenClawSkill {
        name: tool.name.to_string(),
        description: tool.description.to_string(),
        handler: Box::new(move |params| {
            let ctx = ToolContext::from_openclaw_env()?;
            tokio::runtime::Handle::current()
                .block_on(tool.handler.call(params, &ctx))
        }),
    }
}
```

---

## Authentication

### Bearer token (simple)

```bash
BARDO_AUTH_TOKEN=my-secret-token
```

All A2A requests must include `Authorization: Bearer my-secret-token`.

### Three-tier API keys

See [20-config.md](20-config.md) for the full API key model with Read, Feedback, and Write tiers. API keys govern what operations external agents can perform via the A2A endpoint.

### SIWE + JWT (agent-native)

Sign-In With Ethereum for agent authentication. Agent signs a SIWE message, receives a JWT for subsequent requests. Enables on-chain identity verification.

### OAuth 2.1 (enterprise)

Full OAuth 2.1 flow with scopes mapped to tool categories. For enterprise deployments with multiple owners and agents.

---

## Monitoring and observability

### Logging

- Structured JSON logging via `tracing` + `tracing-subscriber`
- Log levels: debug, info, warn, error
- Sensitive fields automatically redacted (API keys, private keys)

### Metrics

| Metric                           | Type      | Description                            |
| -------------------------------- | --------- | -------------------------------------- |
| `bardo_tool_calls_total`         | Counter   | Total tool calls by name               |
| `bardo_tool_latency_seconds`     | Histogram | Tool execution latency                 |
| `bardo_safety_rejections_total`  | Counter   | Safety middleware rejections by reason |
| `bardo_spending_usd`             | Gauge     | Current spending against limits        |
| `bardo_rpc_latency_seconds`      | Histogram | RPC call latency by chain              |
| `bardo_capability_minted_total`  | Counter   | Capability tokens minted by tool type  |
| `bardo_capability_expired_total` | Counter   | Capability tokens that expired unused  |

Metrics are exported via Prometheus exposition format on `/metrics` (A2A binary) or through the Event Fabric (`GolemEvent::MetricsSample`).

### Health check

```rust
use bardo_tools::health_check;

let status = health_check(&config).await;
// HealthStatus { healthy: true, chains: [...], wallet: "connected", tools: 95 }
```

For the A2A endpoint, `GET /health` returns `200` with service status or `503` if down.

---

## Security hardening checklist

- [ ] HTTPS for all A2A connections
- [ ] API keys rotated on schedule
- [ ] Private keys in TEE (Privy), never in env vars in production
- [ ] Token allowlist in strict mode
- [ ] Capability token limits configured for production use
- [ ] Rate limiting enabled
- [ ] Circuit breakers for all external dependencies
- [ ] Simulation enabled for all write operations
- [ ] Logging with sensitive field redaction
- [ ] Regular dependency audits (`cargo audit`)

---

## Deployment progression

| Stage   | Access          | Custody                | Safety                | Use case                |
| ------- | --------------- | ---------------------- | --------------------- | ----------------------- |
| Stage 0 | In-process      | LocalKey               | Relaxed limits        | Development, testing    |
| Stage 1 | In-process      | Embedded (Privy TEE)   | Production limits     | Local production Golem  |
| Stage 2 | A2A endpoint    | Embedded (Privy TEE)   | Production + auth     | Remote single-agent     |
| Stage 3 | A2A endpoint    | Delegation (MetaMask)  | Full capability stack | Owner-delegated Golem   |
| Stage 4 | A2A + x402      | Embedded (Privy TEE)   | Full + x402 gating    | Public pay-per-use      |

---

## Ecosystem integration

### Companion agents

25 agent definitions in the Golem specification provide autonomous multi-step workflows. The `trade-executor` agent chains `uniswap_get_quote` -> `safety_simulate_transaction` -> `uniswap_execute_swap`. The `lp-manager` agent chains pool analysis -> position management -> fee collection.

### Vault integration

Bardo Vaults (`bardo-vault`) depend on core `bardo-tools` handlers for pool data, swaps, and LP operations. The shared safety pipeline provides defense-in-depth across both crates.

### x402 pricing model

For pay-per-use deployments via the A2A endpoint:

| Tool category   | Price per call |
| --------------- | -------------- |
| Data reads      | $0.001         |
| Quotes          | $0.002         |
| Simulation      | $0.005         |
| Trade execution | $0.01          |
| LP operations   | $0.01          |
| Intelligence    | $0.005         |

Settlement via x402 (micropayment protocol; agents pay via signed USDC transfers, no API keys): USDC on Base, ~200ms. No API keys, no accounts, no billing setup.

### Golem consumption

Golems are the primary consumer of `bardo-tools`. Every heartbeat tick that involves market interaction flows through tool handlers. The `ToolRegistry` provides in-process invocation with zero transport overhead.

When a Golem dies, its final revenue report is generated via `intel_get_agent_revenue`. The death reflection stored in the Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) references specific tool interactions that contributed to the Golem's success or failure.
