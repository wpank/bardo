# Bardo Tools -- Identity, Wallet, and Session Keys [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> On-chain agent identity via ERC-8004, wallet management, and session key delegation. Categories: `identity`, `wallet`. 24 tools (12 identity + 9 wallet + 3 session key).

---

> **Reader orientation:** This document specifies 24 identity, wallet, and session key tools in `bardo-tools`. The identity tools interact with the ERC-8004 Agent Registry for on-chain agent discovery, reputation, and profile management. Wallet tools handle balance queries, token transfers, and nonce management. Session key tools manage delegated signing authority. You should be familiar with ERC-4337 account abstraction and smart account delegation patterns. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## ERC-8004 identity (12 tools)

On-chain agent identity via the ERC-8004 Agent Registry [ERC-8004-2025]. Data sourced from The Graph subgraphs (multi-chain) with IPFS multi-gateway racing (ipfs.io, cloudflare-ipfs, dweb.link, pinata) for metadata resolution.

### `identity_discover_agents`

Query the ERC-8004 registry to discover agents by service type, chain, and minimum reputation. Includes A2A Agent Card resolution for agents with A2A endpoints. Subsumes the old `identity_search_agents` + `identity_list_agents` split.

```rust
#[derive(Debug, Deserialize)]
pub struct DiscoverAgentsParams {
    /// Search query (name, description, service). Optional -- omit for browsing.
    pub query: Option<String>,
    /// Filter by protocol: "mcp", "a2a", "oasf".
    pub protocol: Option<String>,
    /// Chain filter.
    pub chain: Option<String>,
    /// Minimum reputation score 0-500 (maps to 0.0-5.0).
    pub min_reputation: Option<u32>,
    /// Sort by: "relevance", "reputation", "activity" (default: "relevance" if query, "activity" otherwise).
    pub sort_by: Option<String>,
    /// Max results (default: 20, max: 100).
    #[serde(default = "default_limit_20")]
    pub limit: u32,
    /// Pagination offset.
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Serialize)]
pub struct DiscoverAgentsResult {
    pub agents: Vec<AgentSummary>,
    pub total: u32,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct AgentSummary {
    pub agent_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub protocols: Vec<String>,
    pub feedback_count: u32,
    pub average_score: f64,
    pub active: bool,
    /// Resolved A2A Agent Card, if the agent has an a2a endpoint.
    pub agent_card: Option<A2aAgentCard>,
    pub relevance_score: Option<f64>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_discover_agents",
    description: concat!(
        "Query ERC-8004 registry to discover agents by service type, chain, and reputation. ",
        "Includes A2A Agent Card resolution. ",
        "Use for agent discovery, A2A collaboration setup, and service resolution. ",
        "Supports both search (with query) and browse (without query) modes.",
    ),
    category: Category::Identity,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,       // Subgraph + IPFS racing
    progress_steps: &["Querying subgraph", "Resolving metadata", "Loading Agent Cards"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Discover ERC-8004 agents by service type, chain, reputation. Includes A2A Agent Card resolution.",
    prompt_guidelines: &[
        "thriving: Use freely for agent discovery and A2A collaboration setup.",
        "cautious: Verify discovered agents with identity_evaluate_agent_trust before interaction.",
        "declining: Only discover agents that provide exit-related services.",
        "terminal: Not needed.",
    ],
};
```

**Event Fabric** (Bardo's `tokio::broadcast` channel system for real-time event streaming between runtime components)**:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ tool_name: "identity_discover_agents", params_hash, tick }` |
| `tool:update` | Steps: "Querying subgraph", "Resolving metadata", "Loading Agent Cards" |
| `tool:end` | `{ success, duration_ms, result_summary: "Found {n} agents" }` |

### `identity_get_agent_profile`

Get full agent profile: contract state, registered services, on-chain and off-chain metadata (IPFS), engagement stats, and recent feedback. Subsumes the old `identity_get_agent` + `identity_get_agent_metadata` + `identity_get_profile` split.

```rust
#[derive(Debug, Deserialize)]
pub struct GetAgentProfileParams {
    /// Agent ID (e.g., "1:1213" for chainId:registrationId, or "8453:42").
    pub agent_id: String,
    /// Include full feedback history (default: false, returns only summary).
    #[serde(default)]
    pub include_feedback: bool,
}

#[derive(Debug, Serialize)]
pub struct AgentProfile {
    pub agent_id: String,
    pub chain_id: u64,
    pub owner: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub active: bool,
    pub registration: AgentRegistration,
    pub services: Vec<AgentService>,
    pub engagement: EngagementStats,
    pub feedback_summary: FeedbackSummary,
    pub feedback_history: Option<Vec<Feedback>>,
}

#[derive(Debug, Serialize)]
pub struct AgentRegistration {
    pub x402_support: bool,
    pub supported_trusts: Vec<String>,
    pub tool_endpoint: Option<String>,
    pub tool_version: Option<String>,
    pub a2a_endpoint: Option<String>,
    pub a2a_version: Option<String>,
    pub ens: Option<String>,
    pub did: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentService {
    pub service_type: String,
    pub endpoint: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EngagementStats {
    pub total_feedback: u32,
    pub average_score: f64,
    pub score_distribution: [u32; 5],
    pub recent_trend: String,       // "improving", "declining", "stable"
}

#[derive(Debug, Serialize)]
pub struct FeedbackSummary {
    pub total: u32,
    pub average: f64,
    pub tag_breakdown: std::collections::HashMap<String, u32>,
    pub top_providers: Vec<FeedbackProvider>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_get_agent_profile",
    description: concat!(
        "Get full agent profile from ERC-8004: contract state, services, metadata (IPFS), ",
        "engagement stats, feedback summary. ",
        "Use for agent due diligence and A2A endpoint resolution.",
    ),
    category: Category::Identity,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Fetching contract state", "Resolving IPFS metadata", "Loading engagement stats"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Full agent profile: state, services, metadata, engagement, feedback. Use for due diligence.",
    prompt_guidelines: &[
        "thriving: Check before A2A interactions. Verify services and active status.",
        "cautious: Always check. Cross-reference with trust evaluation.",
        "declining: Quick check for exit-related service agents.",
        "terminal: Not needed.",
    ],
};
```

### `identity_get_reputation_summary`

Aggregated reputation from the ERC-8004 Reputation Registry. Bayesian Beta distribution with deterministic audit weighting.

```rust
#[derive(Debug, Deserialize)]
pub struct GetReputationSummaryParams {
    pub agent_id: String,
}

#[derive(Debug, Serialize)]
pub struct ReputationSummary {
    pub agent_id: String,
    pub total_feedback: u32,
    pub average_score: f64,
    pub tag_breakdown: std::collections::HashMap<String, u32>,
    /// "improving", "declining", "stable".
    pub recent_trend: String,
    pub top_providers: Vec<FeedbackProvider>,
    /// Bayesian Beta posterior: (alpha, beta).
    pub beta_distribution: (f64, f64),
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_get_reputation_summary",
    description: "Aggregated reputation from ERC-8004 Reputation Registry: Bayesian Beta, feedback stats, trend analysis. Counterparty due diligence.",
    category: Category::Identity,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Querying Reputation Registry"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Aggregated reputation: Bayesian Beta, feedback stats, trend. Use for counterparty due diligence.",
    prompt_guidelines: &[
        "thriving: Check before A2A transactions. Require average score > 3.5/5.0.",
        "cautious: Require score > 4.0/5.0. Check for declining trend.",
        "declining: Quick checks for exit service agents.",
        "terminal: Not needed.",
    ],
};
```

### `identity_submit_feedback`

Submit feedback (0-5 stars) for an agent interaction. On-chain write to the Reputation Registry.

```rust
#[derive(Debug, Deserialize)]
pub struct SubmitFeedbackParams {
    pub agent_id: String,
    /// Score 0-500, maps to 0.0-5.0.
    pub score: u32,
    /// Optional tags (max 2). E.g., "reliable", "fast", "accurate", "overpriced".
    pub tag1: Option<String>,
    pub tag2: Option<String>,
    /// Optional review text.
    pub review: Option<String>,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct SubmitFeedbackResult {
    pub tx_hash: String,
    pub feedback_id: String,
    pub agent_id: String,
    pub score: u32,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_submit_feedback",
    description: "Submit feedback (0-5 stars) for an agent interaction. On-chain write to Reputation Registry. Call after A2A task completion.",
    category: Category::Identity,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Building transaction", "Simulating", "Broadcasting"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Submit on-chain feedback for an agent. Write operation.",
    prompt_guidelines: &[
        "thriving: Submit feedback after every A2A interaction. Builds ecosystem reputation data.",
        "cautious: Submit feedback, especially for negative experiences.",
        "declining: Submit feedback for agents that contributed to decline.",
        "terminal: Submit final feedback if relevant.",
    ],
};
```

**Ground truth:** Transaction receipt confirms event emission. Expected: `FeedbackSubmitted(agentId, score)`. Actual: decoded from receipt logs. Source: `eth_getTransactionReceipt`.

### `identity_get_validation_status`

Validation state from the ERC-8004 Validation Registry. Methods: zkML, TEE attestation, stake-secured.

```rust
#[derive(Debug, Deserialize)]
pub struct GetValidationStatusParams {
    pub agent_id: String,
}

#[derive(Debug, Serialize)]
pub struct ValidationStatus {
    pub agent_id: String,
    pub validated: bool,
    /// "zkml", "tee-attestation", "stake-secured", or null.
    pub method: Option<String>,
    pub last_validated: Option<String>,
    pub validation_score: Option<f64>,
    pub history: Vec<ValidationEvent>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_get_validation_status",
    description: "Validation state from ERC-8004 Validation Registry: zkML, TEE attestation, stake-secured. Read-only.",
    category: Category::Identity,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Querying Validation Registry"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Agent validation status: zkML, TEE, stake-secured. Gate high-value interactions.",
    prompt_guidelines: &[
        "thriving: Check before high-value A2A interactions.",
        "cautious: Always check. Require active validation.",
        "declining: Quick check for exit service agents.",
        "terminal: Not needed.",
    ],
};
```

### `identity_evaluate_agent_trust`

Composite trust evaluation aggregating data from the Identity, Reputation, and Validation registries. Returns a trust score (0-100) with breakdown.

```rust
#[derive(Debug, Deserialize)]
pub struct EvaluateAgentTrustParams {
    pub agent_id: String,
}

#[derive(Debug, Serialize)]
pub struct TrustEvaluation {
    pub trust_score: u32,
    pub breakdown: TrustBreakdown,
    pub confidence: f64,
    pub flags: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
pub struct TrustBreakdown {
    pub identity_score: u32,
    pub reputation_score: u32,
    pub validation_score: u32,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_evaluate_agent_trust",
    description: concat!(
        "Composite trust score (0-100) from Identity + Reputation + Validation registries. ",
        "Gate high-value A2A interactions through this tool.",
    ),
    category: Category::Identity,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Loading identity", "Loading reputation", "Loading validation", "Computing trust score"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Composite trust score (0-100) from Identity + Reputation + Validation. Gate high-value A2A interactions.",
    prompt_guidelines: &[
        "thriving: Run before high-value interactions. Require score > 70.",
        "cautious: Require score > 80. Verify validation is active and recent.",
        "declining: Quick trust check for exit service agents.",
        "terminal: Not needed.",
    ],
};
```

### `identity_register_agent`

Register a new agent on the ERC-8004 registry. On-chain write.

```rust
#[derive(Debug, Deserialize)]
pub struct RegisterAgentParams {
    pub name: String,
    pub description: Option<String>,
    /// IPFS hash or HTTP URL for agent image.
    pub image: Option<String>,
    /// Registration file data (services, endpoints, etc.).
    pub registration: AgentRegistrationInput,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct AgentRegistrationInput {
    pub tool_endpoint: Option<String>,
    pub tool_version: Option<String>,
    pub a2a_endpoint: Option<String>,
    pub a2a_version: Option<String>,
    pub a2a_skills: Option<Vec<String>>,
    pub ens: Option<String>,
    pub did: Option<String>,
    pub x402_support: Option<bool>,
    pub supported_trusts: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct RegisterAgentResult {
    pub tx_hash: String,
    pub agent_id: String,
    pub chain_id: u64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_register_agent",
    description: "Register a new agent on the ERC-8004 registry. On-chain write. Irreversible. Use during initial Golem setup.",
    category: Category::Identity,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Uploading metadata to IPFS", "Building registration tx", "Simulating", "Broadcasting"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Register agent on ERC-8004. On-chain write. Irreversible. Use during initial setup.",
    prompt_guidelines: &[
        "thriving: Register during initial setup. Verify all fields before submission.",
        "cautious: Same. Double-check endpoints are reachable.",
        "declining: Not relevant.",
        "terminal: Not relevant.",
    ],
};
```

**Ground truth:** Transaction receipt confirms event `AgentRegistered(agentId, owner)`. Source: `eth_getTransactionReceipt`.

### `identity_update_metadata`

Update agent metadata (name, description, services, endpoints). On-chain write.

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateMetadataParams {
    pub agent_id: String,
    /// Fields to update. Only non-null fields are changed.
    pub updates: AgentMetadataUpdate,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct AgentMetadataUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub tool_endpoint: Option<String>,
    pub a2a_endpoint: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateMetadataResult {
    pub tx_hash: String,
    pub agent_id: String,
    pub fields_updated: Vec<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_update_metadata",
    description: "Update agent metadata on ERC-8004: name, description, services, endpoints, active status. On-chain write.",
    category: Category::Identity,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Uploading to IPFS", "Building update tx", "Simulating", "Broadcasting"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Update agent metadata on ERC-8004. On-chain write.",
    prompt_guidelines: &[
        "thriving: Update when endpoints change or services expand.",
        "cautious: Update cautiously. Verify new endpoints are reachable.",
        "declining: Set active=false if winding down. Update description to note status.",
        "terminal: Set active=false. Final metadata update.",
    ],
};
```

**Ground truth:** Transaction receipt confirms event `MetadataUpdated(agentId)`. Source: `eth_getTransactionReceipt`.

### `identity_verify_agent`

Verify agent identity via a supported method (zkML, TEE attestation, stake-secured). On-chain write to the Validation Registry.

```rust
#[derive(Debug, Deserialize)]
pub struct VerifyAgentParams {
    pub agent_id: String,
    /// "zkml", "tee-attestation", "stake-secured".
    pub method: String,
    /// Method-specific proof data.
    pub proof: serde_json::Value,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct VerifyAgentResult {
    pub tx_hash: String,
    pub agent_id: String,
    pub method: String,
    pub validation_score: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_verify_agent",
    description: "Verify agent identity via zkML, TEE attestation, or stake-secured. On-chain write to Validation Registry.",
    category: Category::Identity,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Slow,         // Proof verification can take time
    progress_steps: &["Preparing proof", "Submitting verification tx", "Awaiting confirmation"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Verify agent identity on-chain. Write to Validation Registry.",
    prompt_guidelines: &[
        "thriving: Verify during initial setup or when validation expires.",
        "cautious: Re-verify if validation is stale (> 30 days).",
        "declining: Not relevant.",
        "terminal: Not relevant.",
    ],
};
```

**Ground truth:** Transaction receipt confirms `AgentValidated(agentId, method, score)`. Source: `eth_getTransactionReceipt`.

### `identity_get_credentials`

Get agent credentials and API keys associated with the agent's ERC-8004 registration. Read from local secure storage.

```rust
#[derive(Debug, Serialize)]
pub struct AgentCredentials {
    pub agent_id: String,
    pub api_keys: Vec<ApiKeyInfo>,
    pub session_keys: Vec<SessionKeyInfo>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_get_credentials",
    description: "Get agent credentials and API keys from local secure storage. Read-only. Sensitive data handled via TaintedString.",
    category: Category::Identity,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Loading credentials"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Get agent credentials from secure storage. Read-only.",
    prompt_guidelines: &[
        "thriving: Use when API keys are needed for service calls.",
        "cautious: Same. Verify key scopes match intended use.",
        "declining: Same.",
        "terminal: Not needed.",
    ],
};
```

### `identity_get_tier`

Get agent tier based on reputation and on-chain activity.

```rust
#[derive(Debug, Deserialize)]
pub struct GetTierParams {
    pub agent_id: String,
}

#[derive(Debug, Serialize)]
pub struct AgentTier {
    pub agent_id: String,
    pub tier: String,          // "bronze", "silver", "gold", "platinum"
    pub points: u32,
    pub next_tier_threshold: u32,
    pub tier_benefits: Vec<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "identity_get_tier",
    description: "Get agent tier based on reputation and activity. Read-only.",
    category: Category::Identity,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Computing tier"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Agent tier: bronze/silver/gold/platinum. Read-only.",
    prompt_guidelines: &[
        "thriving: Check periodically to track progression.",
        "cautious: Same.",
        "declining: Not relevant.",
        "terminal: Not needed.",
    ],
};
```

---

## Wallet management (9 tools)

### `wallet_get_status`

Comprehensive wallet health: provider, balances across chains, active session keys, pending transactions, policy summary, identity status.

```rust
#[derive(Debug, Serialize)]
pub struct WalletStatus {
    pub provider: String,
    pub address: String,
    pub account_type: String,        // "eoa", "erc4337", "safe"
    pub balances: Vec<ChainBalance>,
    pub active_session_keys: u32,
    pub pending_transactions: u32,
    pub policy_summary: PolicySummary,
    pub identity_status: Option<IdentityLink>,
}

#[derive(Debug, Serialize)]
pub struct ChainBalance {
    pub chain_id: u64,
    pub native_balance: String,
    pub native_usd: f64,
    pub token_count: u32,
    pub total_usd: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_get_status",
    description: "Comprehensive wallet health: provider, balances, session keys, pending tx, policy, identity. Primary wallet diagnostic.",
    category: Category::Wallet,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Querying provider", "Loading balances", "Checking session keys"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Wallet health: provider, balances, session keys, pending tx, policy, identity. Primary diagnostic.",
    prompt_guidelines: &[
        "thriving: Check at each heartbeat for balance awareness.",
        "cautious: Check frequently. Monitor for unexpected balance changes.",
        "declining: Check before exit operations.",
        "terminal: Final status check before wind-down.",
    ],
};
```

### `wallet_provision`

Create a wallet via the configured provider (Privy, local key, Safe, ZeroDev). Restricted to `admin:wallet` scope. Rate limited: 5 creations/hour.

```rust
#[derive(Debug, Deserialize)]
pub struct ProvisionWalletParams {
    /// "privy", "local", "safe", "zerodev" (default: from config).
    pub provider: Option<String>,
    /// Chain ID for initial setup.
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
    /// Optional label.
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProvisionWalletResult {
    pub address: String,
    pub provider: String,
    pub chain_id: u64,
    pub account_type: String,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_provision",
    description: "Create a wallet via provider API. Restricted to admin:wallet scope. Rate limited: 5/hour. Use for initial agent setup.",
    category: Category::Wallet,
    capability: CapabilityTier::Privileged,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Creating wallet", "Verifying"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Provision a wallet. admin:wallet scope required. Rate limited: 5/hour.",
    prompt_guidelines: &[
        "thriving: Use during initial setup.",
        "cautious: Same.",
        "declining: Not relevant.",
        "terminal: Not relevant.",
    ],
};
```

**Ground truth:** Provider API confirms wallet creation. Expected: new address returned. Actual: address verified via `eth_getCode` (for smart accounts) or balance check (for EOAs). Source: provider API response.

### `wallet_fund`

Fund the agent wallet with tokens. Testnet: faucet source. Mainnet: bridge or transfer.

```rust
#[derive(Debug, Deserialize)]
pub struct FundWalletParams {
    /// "faucet", "bridge", "transfer".
    pub source: String,
    /// Token to fund (default: native ETH).
    pub token: Option<String>,
    /// Amount in token units.
    pub amount: Option<String>,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct FundWalletResult {
    pub tx_hash: Option<String>,
    pub amount_funded: String,
    pub token: String,
    pub source: String,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_fund",
    description: "Fund agent wallet. Testnet: faucet. Mainnet: bridge or transfer. Use for initial setup or gas replenishment.",
    category: Category::Wallet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Slow,
    progress_steps: &["Requesting funds", "Awaiting confirmation"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Fund the agent wallet. Testnet: faucet. Mainnet: bridge or transfer.",
    prompt_guidelines: &[
        "thriving: Fund gas reserves when below 0.01 ETH on active chains.",
        "cautious: Maintain minimum gas reserves for exit operations.",
        "declining: Fund only enough gas for planned exits.",
        "terminal: Fund only if gas needed for final exit transactions.",
    ],
};
```

**Ground truth:** Balance check before and after. Expected: balance increases by funded amount. Actual: `eth_getBalance` post-transaction. Source: on-chain balance.

### `wallet_configure_policy`

Update the signing policy for the configured wallet. Policies are enforced by the wallet provider (Privy) and cannot be bypassed by the agent.

```rust
#[derive(Debug, Deserialize)]
pub struct ConfigurePolicyParams {
    /// Preset: "vault-participant", "vault-manager", "defi-trading", "read-only".
    pub policy_template: Option<String>,
    /// Custom policy. Only used if policy_template is None.
    pub custom_policy: Option<CustomPolicy>,
}

#[derive(Debug, Deserialize)]
pub struct CustomPolicy {
    pub allowed_contracts: Vec<String>,
    pub spending_limits: Vec<SpendingLimit>,
    pub chain_restrictions: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct ConfigurePolicyResult {
    pub policy_hash: String,
    pub template: Option<String>,
    pub allowed_contracts: u32,
    pub spending_limit_usd: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_configure_policy",
    description: "Update signing policy enforced by wallet provider. Cannot be bypassed by the agent. Preset templates or custom policies.",
    category: Category::Wallet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Validating policy", "Applying to provider"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Update wallet signing policy. Provider-enforced. Preset templates or custom.",
    prompt_guidelines: &[
        "thriving: Set policy matching strategy (e.g., defi-trading for active trading).",
        "cautious: Tighten policy. Reduce allowed contracts and spending limits.",
        "declining: Set to read-only or minimal write for exits only.",
        "terminal: Set to read-only after final exits.",
    ],
};
```

**Ground truth:** Provider API confirms policy update. Source: provider API response.

### `wallet_get_delegation`

Check the delegation status of a wallet for a specific contract or method.

```rust
#[derive(Debug, Deserialize)]
pub struct GetDelegationParams {
    pub contract: Option<String>,
    pub method: Option<String>,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct DelegationStatus {
    pub delegated: bool,
    pub delegate_address: Option<String>,
    pub permissions: Vec<DelegatedPermission>,
    pub expires_at: Option<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_get_delegation",
    description: "Check wallet delegation status for a specific contract or method. Read-only.",
    category: Category::Wallet,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Checking delegation"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Check wallet delegation status. Read-only.",
    prompt_guidelines: &[
        "thriving: Check before operations requiring delegation.",
        "cautious: Always check. Verify delegations are still appropriate.",
        "declining: Audit delegations for revocation.",
        "terminal: Check before revoking all delegations.",
    ],
};
```

### `wallet_transfer_token`

Create and execute an ERC-20 token transfer.

```rust
#[derive(Debug, Deserialize)]
pub struct TransferTokenParams {
    pub token: String,
    pub recipient: String,
    pub amount: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct TransferTokenResult {
    pub tx_hash: String,
    pub token: String,
    pub recipient: String,
    pub amount: String,
    pub gas_used: u64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_transfer_token",
    description: "Execute an ERC-20 token transfer. Write operation. Passes through full safety hook chain.",
    category: Category::Wallet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Medium,
    progress_steps: &["Validating recipient", "Building transfer tx", "Simulating", "Broadcasting"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "ERC-20 token transfer. Write operation.",
    prompt_guidelines: &[
        "thriving: Use for operational token movements. Verify recipient address.",
        "cautious: Double-check recipient. Use dryRun if available.",
        "declining: Use for consolidating assets before exit.",
        "terminal: Final asset transfers to owner or successor wallet.",
    ],
};
```

**Ground truth:** Balance check before/after. Expected: sender balance decreases, recipient increases by amount. Actual: `balanceOf` calls post-receipt. Source: on-chain balance.

### `wallet_detect_account_type`

Detect whether an address is an EOA or smart account (ERC-4337/Safe) by checking on-chain bytecode.

```rust
#[derive(Debug, Deserialize)]
pub struct DetectAccountTypeParams {
    pub address: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct AccountTypeResult {
    pub address: String,
    pub account_type: String,        // "eoa", "erc4337", "safe", "contract", "unknown"
    pub has_code: bool,
    pub code_size: u64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_detect_account_type",
    description: "Detect if an address is EOA or smart account (ERC-4337/Safe). Check before sending tokens to unknown addresses.",
    category: Category::Wallet,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Checking bytecode"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Detect EOA vs. smart account. Read-only.",
    prompt_guidelines: &[
        "thriving: Check counterparty addresses before interaction.",
        "cautious: Always check. Smart accounts may have permissions or restrictions.",
        "declining: Check destination addresses for transfers.",
        "terminal: Check once for final transfer destinations.",
    ],
};
```

### `wallet_get_gas_price`

Current gas prices on a chain with EIP-1559 breakdown (base fee, priority fee, estimated swap cost).

```rust
#[derive(Debug, Deserialize)]
pub struct GetGasPriceParams {
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

#[derive(Debug, Serialize)]
pub struct GasPriceResult {
    pub chain_id: u64,
    pub base_fee_gwei: f64,
    pub priority_fee_gwei: f64,
    pub estimated_swap_cost_usd: f64,
    pub estimated_lp_cost_usd: f64,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_get_gas_price",
    description: "Current gas prices with EIP-1559 breakdown and estimated operation costs. Read-only.",
    category: Category::Wallet,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Fetching gas oracle"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Gas prices with EIP-1559 breakdown. Read-only.",
    prompt_guidelines: &[
        "thriving: Check before operations. Delay non-urgent writes if gas is expensive.",
        "cautious: Always check. Factor gas into operation cost-benefit.",
        "declining: Check before exit trades.",
        "terminal: Quick check before final transactions.",
    ],
};
```

### `wallet_get_supported_chains`

List all chains supported by this Bardo instance with RPC status, block number, and Uniswap version support.

```rust
#[derive(Debug, Serialize)]
pub struct SupportedChainsResult {
    pub chains: Vec<ChainInfo>,
}

#[derive(Debug, Serialize)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    pub rpc_status: String,          // "healthy", "degraded", "down"
    pub block_number: u64,
    pub uniswap_versions: Vec<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "wallet_get_supported_chains",
    description: "List supported chains with RPC status, block number, Uniswap version support. Read-only.",
    category: Category::Wallet,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Checking chain health"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Supported chains with RPC status and Uniswap versions. Read-only.",
    prompt_guidelines: &[
        "thriving: Check once at startup.",
        "cautious: Check if RPC errors occur.",
        "declining: Verify chains with positions are reachable.",
        "terminal: Same.",
    ],
};
```

---

## Session keys (3 tools)

### `session_provision_key`

Create a scoped session key with time-limited permissions for specific contracts and methods. Delegates narrow authority without exposing the main wallet key.

```rust
#[derive(Debug, Deserialize)]
pub struct ProvisionSessionKeyParams {
    /// Array of permission grants.
    pub permissions: Vec<SessionPermission>,
    /// Seconds until expiry (default: 86400 = 24h).
    #[serde(default = "default_expiry_86400")]
    pub expires_in: u64,
    /// Human-readable label.
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionPermission {
    pub contract: String,
    pub methods: Vec<String>,
    pub chains: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct ProvisionSessionKeyResult {
    pub key_id: String,
    pub public_key: String,
    pub expires_at: String,
    pub permissions: Vec<SessionPermission>,
    pub label: Option<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "session_provision_key",
    description: "Create a scoped session key with time-limited permissions. Delegates narrow authority without exposing the main wallet key.",
    category: Category::Wallet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Generating key", "Registering permissions"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Create scoped session key with time-limited permissions. Delegates narrow authority.",
    prompt_guidelines: &[
        "thriving: Provision for specific automated operations. 24h expiry is reasonable.",
        "cautious: Minimal permissions only. Only contracts and methods strictly needed.",
        "declining: Do not provision new keys. Revoke unnecessary ones.",
        "terminal: Revoke all. No delegation during wind-down.",
    ],
};
```

**Ground truth:** Session key registered with provider. Source: provider API response.

### `session_get_keys`

List all active session keys with permissions, expiry, and usage stats.

```rust
#[derive(Debug, Serialize)]
pub struct SessionKeysResult {
    pub keys: Vec<SessionKeyInfo>,
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct SessionKeyInfo {
    pub key_id: String,
    pub label: Option<String>,
    pub created_at: String,
    pub expires_at: String,
    pub permissions: Vec<SessionPermission>,
    pub usage_count: u32,
    pub last_used: Option<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "session_get_keys",
    description: "List active session keys with permissions, expiry, and usage stats. Audit active delegations.",
    category: Category::Wallet,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Loading session keys"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "List active session keys. Audit delegations. Read-only.",
    prompt_guidelines: &[
        "thriving: Review monthly. Verify all keys are still needed.",
        "cautious: Review weekly. Remove unnecessary keys.",
        "declining: Review for revocation.",
        "terminal: Final check before revoking all.",
    ],
};
```

### `session_revoke_key`

Revoke one or more active session keys immediately.

```rust
#[derive(Debug, Deserialize)]
pub struct RevokeSessionKeyParams {
    /// Session key IDs to revoke. Empty = revoke ALL.
    #[serde(default)]
    pub key_ids: Vec<String>,
    /// Reason for revocation (audit trail).
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RevokeSessionKeyResult {
    pub revoked_count: u32,
    pub revoked_keys: Vec<String>,
}
```

**ToolDef:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "session_revoke_key",
    description: "Revoke active session keys immediately. Empty key_ids = revoke ALL. Risk-reducing operation.",
    category: Category::Wallet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer1,       // Revocation is risk-reducing
    tick_budget: TickBudget::Fast,
    progress_steps: &["Revoking keys"],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Revoke session keys. Empty key_ids = revoke ALL. Risk-reducing.",
    prompt_guidelines: &[
        "thriving: Revoke expired or unused keys monthly.",
        "cautious: Revoke any key with broader permissions than currently needed.",
        "declining: Revoke all session keys.",
        "terminal: Revoke all as part of wind-down.",
    ],
};
```

**Ground truth:** Provider confirms revocation. Source: provider API response.

---

## ERC-8004 data model

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRegistrationFile {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub active: Option<bool>,
    pub x402_support: Option<bool>,
    pub supported_trusts: Vec<String>,
    pub tool_endpoint: Option<String>,
    pub tool_version: Option<String>,
    pub tools: Vec<String>,
    pub prompts: Vec<String>,
    pub resources: Vec<String>,
    pub a2a_endpoint: Option<String>,
    pub a2a_version: Option<String>,
    pub a2a_skills: Vec<String>,
    pub ens: Option<String>,
    pub did: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feedback {
    pub id: String,
    /// 0-500, maps to 0.0-5.0.
    pub value: u32,
    pub tag1: Option<String>,
    pub tag2: Option<String>,
    pub client_address: String,
    pub created_at: String,
    pub revoked: bool,
}
```

---

## Custody implications (identity and wallet write tools)

Identity and wallet tools interact with on-chain identity registries and wallet configuration:

- **Delegation**: Session key signs via the ERC-8004 registry contract. CaveatEnforcer must whitelist the registry address. `identity_register` and `identity_update_metadata` require the session key to be authorized for the Golem's identity record. Session key provisioning (`session_key_provision`) creates a new sub-key scoped to specific operations.
- **Embedded**: Privy server wallet signs. PolicyCage validates registry target and identity ownership.
- **Local Key**: Local keypair signs. On-chain delegation bounds constrain allowed registry interactions.

Privileged tools (`wallet_provision`, `identity_transfer_ownership`) require owner approval in all custody modes. `wallet_fund` requires the funding source to be an owner-controlled address.

---

## Tool count summary

| Section | Tools | Read | Write | Privileged |
| --- | --- | --- | --- | --- |
| ERC-8004 identity | 12 | 7 | 4 | 1 |
| Wallet management | 9 | 5 | 3 | 1 |
| Session keys | 3 | 1 | 2 | 0 |
| **Total** | **24** | **13** | **9** | **2** |

Write tools pass through the full safety hook chain. Privileged tools (`wallet_provision`) require owner approval in addition to capability tokens.
