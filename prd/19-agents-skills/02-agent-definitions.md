# Golem archetypes -- behavioral preset format [SPEC]

> **Cross-ref**: [00-agents-overview.md](00-agents-overview.md) (architecture overview for archetypes and tool profiles), [01-agent-categories.md](01-agent-categories.md) (full inventory of 42+ archetypes across 14 categories), [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Pi extension loading and tool registration)

> **Reader orientation:** This document specifies the concrete format of Golem (mortal autonomous agent) archetype definitions: TOML configuration files deserialized into Rust structs at boot. It belongs to Section 19 (Agents & Skills) and covers the `ArchetypeConfig` struct, validation rules, model selection rationale, and the directory structure. An archetype is a behavioral preset -- not the Golem itself -- that configures tool access, delegation rules, and system prompt fragments. See `prd2/shared/glossary.md` for full term definitions.

---

## Archetype as TOML + Rust configuration

Archetypes are TOML files in `bardo-golem-rs/crates/golem-tools/archetypes/`. Each file is deserialized at boot into an `ArchetypeConfig` Rust struct. No markdown frontmatter. No Claude-specific conventions.

### ArchetypeConfig struct

```rust
// golem-tools/src/archetypes/types.rs

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ArchetypeConfig {
    /// Unique identifier, kebab-case
    pub name: String,

    /// One-sentence description of role
    pub description: String,

    /// Default inference tier for this archetype
    pub default_tier: InferenceTier,

    /// Tool categories this archetype needs loaded
    pub tool_categories: Vec<ToolCategory>,

    /// Archetype names this archetype may delegate to
    pub delegates_to: Vec<String>,

    /// Whether this archetype is a terminal node (no delegation)
    pub terminal: bool,

    /// Category for grouping and discovery
    pub category: ArchetypeCategory,

    /// System prompt fragments injected into Pi session
    pub system_prompt: SystemPromptFragments,

    /// Tool-specific overrides (optional)
    pub tool_overrides: Option<ToolOverrides>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SystemPromptFragments {
    pub role: String,
    pub expertise: Vec<String>,
    pub safety_rules: Vec<String>,
    pub workflow: String,
    pub output_format: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ToolOverrides {
    pub exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchetypeCategory {
    Execution, Research, Strategy, Development, Infrastructure,
    AgentCapitalMarkets, RealTimeMonitoring, Vault, Golem, Observer,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Data, Trading, Lp, Vault, Safety, Identity, Memory,
    Streaming, Intelligence, Fees, SelfImprovement, Hook, Testnet,
}
```

### Example: trade-executor archetype

```toml
# bardo-golem-rs/crates/golem-tools/archetypes/trade-executor.toml

name = "trade-executor"
description = "Full swap pipeline from quote through safety validation and confirmation"
default_tier = "T2"
tool_categories = ["data", "trading", "safety"]
delegates_to = ["safety-guardian", "risk-assessor"]
terminal = false
category = "execution"

[system_prompt]
role = "You execute token swaps on any supported chain. You handle the full pipeline: quote retrieval, route optimization, simulation, safety validation, and broadcast."
expertise = [
    "Multi-hop swap routing across V2, V3, V4, and UniswapX",
    "Slippage management and price impact analysis",
    "Permit2 approval and signature workflows",
    "Gas optimization for L2 chains",
]
safety_rules = [
    "ALWAYS simulate before broadcast",
    "NEVER skip safety-guardian delegation for swaps above spending limit",
    "REFUSE swaps on tokens not in the allowlist unless operator explicitly approves",
]
workflow = """
1. Parse swap intent (token pair, amount, chain)
2. Get quote via get_quote tool
3. Simulate transaction via simulate_transaction
4. Delegate to safety-guardian for validation
5. If approved: broadcast via commit_action
6. Record outcome as episode
"""
output_format = """
result:
  status: success | failure | vetoed
  txHash: "0x..."
  amountIn: "..."
  amountOut: "..."
  priceImpact: "0.12%"
  gasUsed: "..."
"""
```

---

## Schema validation

Archetypes are validated at boot by `golem-tools` using serde + custom validation:

```rust
// golem-tools/src/archetypes/registry.rs

pub fn load_archetypes(dir: &Path) -> Result<ArchetypeRegistry, ArchetypeError> {
    let configs: Vec<ArchetypeConfig> = dir
        .read_dir()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("toml".as_ref()))
        .map(|e| toml::from_str(&std::fs::read_to_string(e.path())?)
            .map_err(ArchetypeError::ParseError))
        .collect::<Result<_, _>>()?;

    validate_dag(&configs)?;
    Ok(ArchetypeRegistry { configs })
}
```

### Validation rules enforced at boot

- All `delegatesTo` entries reference archetypes that exist in the registry
- Terminal archetypes have empty `delegatesTo`
- No DAG cycles (topological sort passes)
- Max delegation depth: 3
- `safety-guardian` archetype has no write-capable tool categories
- System prompt fragments are non-empty

---

## Model selection rationale

| Archetype role              | Model  | Rationale                                                |
| --------------------------- | ------ | -------------------------------------------------------- |
| Multi-step execution        | opus   | Safety-critical path, adversarial input risk             |
| Creative strategy           | opus   | Ambiguous inputs, multi-variable tradeoffs               |
| Code generation             | opus   | Solidity generation, architecture design                 |
| Structured data queries     | sonnet | Templated output, well-defined sequential workflows      |
| Threshold monitoring        | sonnet | Status checks, metric comparison, alert dispatch         |
| Deterministic probes (T0)   | none   | No LLM needed, pure Rust FSM in `golem-heartbeat`        |

The model assignment is a default. The `bardo-model-router` extension can escalate or de-escalate based on tick severity, available budget, and PAD mood state. See `../01-golem/01-cognition.md` S2.

---

## Directory structure

```text
bardo-golem-rs/crates/golem-tools/
  src/archetypes/
    mod.rs                        # Re-exports
    registry.rs                   # Loads TOML files, validates DAG at boot
    types.rs                      # ArchetypeConfig, ArchetypeCategory, ToolCategory
    validate_dag.rs               # Cycle detection, depth check, terminal enforcement
  archetypes/
    # Execution
    trade-executor.toml
    liquidity-manager.toml
    cross-chain-executor.toml

    # Research & analysis
    pool-researcher.toml
    token-analyst.toml
    opportunity-scanner.toml
    portfolio-analyst.toml
    pnl-analyst.toml

    # Strategy
    lp-strategist.toml
    risk-assessor.toml

    # Development
    hook-builder.toml
    integration-architect.toml
    integration-advisor.toml

    # Infrastructure
    safety-guardian.toml
    wallet-provisioner.toml

    # Agent capital markets
    treasury-manager.toml
    token-deployer.toml
    identity-verifier.toml
    protocol-fee-seeker.toml
    competition-participant.toml
    agent-service-broker.toml
    strategy-optimizer.toml
    yield-scout.toml

    # Real-time monitoring
    market-monitor.toml
    position-monitor.toml

    # Golem
    golem-instance.toml
    memory-consolidator.toml
    heartbeat-monitor.toml

    # Observer
    sleepwalker-observer.toml

    # Vault (lives in packages/vault/, config shared with golem-tools)
    vault-manager.toml
    vault-strategist.toml
    vault-watchdog.toml
    vault-auctioneer.toml
    vault-creator.toml
    vault-allocator.toml
    vault-executor.toml
```

---
