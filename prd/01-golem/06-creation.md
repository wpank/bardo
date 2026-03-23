# Onboarding & Creation Flows [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Reader orientation:** This document specifies how a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) is created. It covers the three-interaction onboarding flow (Describe, Review, Confirm), STRATEGY.md (the owner-set goals document, hot-reloadable, the human-to-agent instruction surface) compilation, and the GolemExtendedManifest that feeds into provisioning. It belongs to the `01-golem` section's lifecycle layer. The key concept: creation is designed for cognitive simplicity -- three user interactions that produce a fully configured agent ready for the `07-provisioning.md` (8-step type-state deployment pipeline) pipeline. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## S1 -- Overview

Creating a Golem follows a three-interaction pattern: **Describe**, **Review**, **Confirm**. The design principle is cognitive simplicity, not wall-clock speed. If hitting 60 seconds requires skipping a confirmation step, do not skip the confirmation step.

Three entry points converge on a single artifact:

```
App UI Wizard ----+
                  +--> GolemExtendedManifest --> Provisioning Pipeline
CLI (golem.toml) -+                              (07-provisioning.md)
                  |
API (POST /v1/)---+
```

Target timing for a funded user: 60--160 seconds end-to-end, with no more than 3 user interactions. First-time users (signup + wallet funding) complete onboarding in under 10 minutes with in-app guidance.

---

## S2 -- Design Principles

### 2.1 Three Interactions, Not 60 Seconds

The happy path is three user interactions: (1) describe your Golem, (2) review the generated manifest, (3) confirm and pay. Everything else -- strategy compilation, heartbeat generation, wallet creation, ERC-8004 metadata, policy configuration -- is handled by AI autofill and sane defaults. The user confirms, not writes.

The principle is cognitive simplicity. A new user sees a text box and a "Create" button. An experienced owner sees the full manifest editor with Monaco syntax highlighting and protocol-specific parameter panels. The wizard surface area scales with user sophistication.

### 2.2 Progressive Disclosure

The creation wizard shows only what is essential by default. Advanced configuration -- custody mode selection, transfer restrictions, learning policies, Grimoire sharing, ERC-8004 metadata, network settings, RPC providers -- is tucked behind expandable sections. Users who want control get it; users who do not never see it.

### 2.3 AI-First Composition

When a model provider is available (Bardo Inference via x402, or a user-configured API key), the creation wizard uses a single LLM call to transform a short natural-language prompt into a complete `GolemCreationManifest`. The user describes intent in plain English; AI handles translation to structured configuration.

Strategy templates are the zero-cost alternative. They bypass AI entirely -- instant generation, deterministic output, auditable parameters. Templates and AI autofill are complementary, not competing paths.

### 2.4 Batch Economics

Wallet provisioning, ERC-8004 registration, and initial funding happen in as few transactions as possible. In Delegation mode (recommended), the entire flow requires a single ERC-7715 signature -- no on-chain token transfers. In Embedded (Privy) mode, a Permit2 batch signature reduces the flow to one off-chain signature plus one on-chain transaction. On testnet (where BardoRouter is deployed), a single USDC Permit2 signature plus one batched multicall covers everything -- two user-facing confirmations total.

### 2.5 Confirm Before Commit

Every irreversible action -- funding, on-chain registration, compute provisioning -- gets an explicit confirmation screen with cost breakdown. No surprise charges. No auto-submission. The first 3 AI generations per account are free (no x402 charge). After the free quota, each generation shows an inline cost confirmation before proceeding.

> Tracked per owner wallet address. New wallet resets counter -- known limitation accepted for simplicity.

### 2.6 Network-Aware Safety

First-time creators default to testnet. Creating a mainnet Golem requires explicit opt-in via an interstitial that explains the feature and safety differences. Features that depend on unaudited Bardo contracts are automatically disabled on mainnet.

---

## S3 -- Three Personas

### 3.1 New DeFi User

Has some crypto (ETH or stablecoins on an exchange), no agent experience, may not have a Web3 wallet yet. Arrives via marketing, social media, or word of mouth. Wants to "try an AI agent" without understanding the infrastructure.

**Needs**: Fiat on-ramp or bridge widget, testnet by default, template-based strategy selection (not freeform prompt), minimal jargon, clear cost previews, single-click deployment. Should never see STRATEGY.md or ERC-8004 metadata unless they go looking. Heartbeat configuration is defined within STRATEGY.md (see `heartbeat` field in schema above).

**Success metric**: Creates and runs a testnet Golem in under 10 minutes from first page load.

### 3.2 Experienced DeFi User

Active trader or yield farmer with an existing wallet, familiar with Uniswap/Morpho/Aave, understands gas and slippage. Wants automation for strategies they already execute manually.

**Needs**: AI autofill from natural-language strategy description, mainnet deployment, custom RPC provider, strategy preview and editing, real-time cost estimation. Comfortable reviewing generated STRATEGY.md but does not want to write it from scratch.

**Success metric**: Creates a mainnet Golem from a natural-language prompt in under 3 minutes.

### 3.3 Developer / Institutional Owner

Runs self-hosted infrastructure (VPS, dedicated server, or local machine). Wants full control: custom strategy files, TOML configuration, keystore-based wallet management, CI/CD integration via manifest files, programmatic API access.

**Needs**: `bardo golem init` to scaffold `golem.toml`, `--manifest` flag for non-interactive deployment, `--wallet-key-stdin` for piped secrets, `--dry-run` for cost preview without committing, optional ERC-8004 registration, immortal lifespan (no compute credit tracking).

**Success metric**: Deploys a self-hosted Golem from a TOML config file in under 2 minutes.

---

## S4 -- GolemCoreManifest

The manifest is the single configuration artifact that fully describes a Golem to be created. All creation flows produce a manifest. The provisioning pipeline consumes one. The core manifest is serialized as TOML and has five required fields:

```rust
use serde::{Deserialize, Serialize};

/// The minimal manifest sufficient for the happy-path creation flow.
/// A user who types a single prompt and clicks "Create" produces exactly this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemCoreManifest {
    /// Free-text description of what the Golem should do. 10-2000 chars.
    pub prompt: String,

    /// Target chain. Base Sepolia for first-time creators, Base for returning users.
    pub network: NetworkTarget,

    /// Deployment mode. "hosted" runs on Bardo Compute; "self-hosted" runs locally.
    pub mode: DeploymentMode,

    /// Initial USDC funding amount in human-readable units (e.g., "50.00").
    pub funding: String,

    /// Schema version for forward compatibility. Current version: 1.
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkTarget {
    Base,
    BaseSepolia,
    Anvil,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMode {
    Hosted,
    SelfHosted,
}
```

These five fields are sufficient for the happy path. A user who types a single prompt and clicks "Create" produces exactly this.

---

## S5 -- GolemExtendedManifest

The extended manifest adds 17 optional override fields atop the core, including custody mode selection. Every field has a sane default derived from the core manifest via AI autofill or static defaults. The provisioning pipeline always works with the fully resolved extended manifest.

```rust
use std::collections::HashMap;

/// Full manifest with all optional overrides resolved.
/// The provisioning pipeline never works with a partial manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemExtendedManifest {
    /// Core manifest fields (always present).
    #[serde(flatten)]
    pub core: GolemCoreManifest,

    /// Human-readable identifier. AI-generated or `golem-{random6}`.
    pub name: Option<String>,

    /// Full STRATEGY.md content. AI-generated from prompt.
    pub strategy_md: Option<String>,

    /// Custody mode selection. Default: Delegation.
    pub custody: Option<CustodyModeConfig>,

    /// Wallet transfer restriction tier. Default: "strict".
    pub transfer_restriction: Option<TransferRestriction>,

    /// Custom RPC configuration.
    pub rpc: Option<RpcConfig>,

    /// ERC-8004 identity configuration.
    pub identity: Option<IdentityConfig>,

    /// Learning and knowledge sharing configuration.
    pub learning: Option<LearningConfig>,

    /// Template expansion (mutually exclusive with AI autofill).
    pub template_id: Option<String>,
    pub template_params: Option<HashMap<String, String>>,

    /// AI autofill provenance metadata.
    pub autofill: Option<AutofillProvenance>,

    /// Inference provider configuration.
    pub inference: Option<InferenceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferRestriction {
    /// Owner's Main Wallet only.
    Strict,
    /// Owner's Main Wallet + owner's other Golems.
    Clade,
    /// Any address (requires optional Warden delays, deferred).
    Unrestricted,
}
```

### 5.1 Three Custody Modes

The manifest includes a custody mode selection that determines how the Golem holds and spends funds. This is the most consequential choice in the creation flow.

```rust
use std::path::PathBuf;
use alloy::primitives::Address;

/// Three custody modes. Not graduated tiers -- fundamentally different trust models.
/// The owner selects one at creation time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyModeConfig {
    /// Recommended. Funds stay in the owner's MetaMask Smart Account.
    /// ERC-7710/7715 delegation framework. Session keys with on-chain
    /// caveat enforcers. Key compromise is bounded by caveats.
    Delegation {
        smart_account: Option<Address>,
        caveat_enforcers: Option<Vec<CaveatEnforcerConfig>>,
    },

    /// Legacy. Funds transferred to Privy server wallet in AWS Nitro
    /// Enclaves. Simpler but custodial. Requires sweep at death.
    Embedded {
        privy_app_id: Option<String>,
    },

    /// Dev/self-hosted. Locally generated keys bounded by on-chain
    /// delegation. For testing and self-hosted deployments.
    LocalKey {
        private_key_path: Option<PathBuf>,
        delegation_bounds: Option<DelegationBounds>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationBounds {
    pub max_daily_spend_usd: f64,
    pub max_total_calls: u32,
    pub expires_at: u64,
    pub allowed_targets: Vec<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaveatEnforcerConfig {
    pub enforcer_type: String,
    pub terms: HashMap<String, serde_json::Value>,
}
```

**Delegation** is the recommended mode. Funds never leave the owner's wallet. The Golem holds a disposable session key and a signed delegation authorizing it to spend from the owner's Smart Account, subject to on-chain caveat enforcers. Every transaction executes *from* the owner's address. If the session key leaks, the attacker is bounded by caveats. The owner revokes from MetaMask directly -- no Golem cooperation needed.

**Embedded (Privy)** is the legacy mode. Funds transfer to a Privy server wallet running in AWS Nitro Enclaves. Policy enforcement is off-chain (inside the TEE) and binary. Simpler to set up, but the owner surrenders direct custody and must trust Privy's TEE.

**Local Key + Delegation** is for developers. A locally generated keypair (secp256k1 or P-256) bounded by an on-chain delegation. The key is insecure in the traditional sense -- no TEE, no HSM. The paradigm shift: instead of "keep the key secret," the system says "bound the damage if the key leaks."

### 5.2 Inference Configuration

The optional `inference` field configures multi-provider inference:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Ordered provider list. First is tried first.
    pub providers: Option<Vec<ProviderConfig>>,
    /// Payment mode for autonomous inference.
    pub payment: Option<PaymentConfig>,
}
```

When omitted, defaults to Bardo Inference (prepaid) as the sole provider.

### 5.3 Manifest Resolution

The provisioning pipeline never works with a partial manifest. A `resolve_manifest()` function fills in all missing fields using a defined resolution order:

1. Start with sane defaults for the selected network and mode
2. If `template_id` is set, expand the template with `template_params`
3. If no template, run AI autofill for `strategy_md`, `name`, `identity`
4. Apply any explicit overrides from the extended manifest
5. Default custody mode to `Delegation` if not specified
6. Compute funding recommendation and validate against user-specified amount
7. Validate the final manifest against the network's feature set

---

## S6 -- Strategy Templates

Five curated templates cover the most common DeFi agent strategies, providing an instant, free, auditable alternative to AI autofill. When a user selects a template, the manifest is expanded deterministically from predefined parameters -- no LLM call, no x402 payment, no latency.

| Template ID        | Name                       | Risk Profile | Default Funding (Mainnet) | Default Funding (Testnet) |
| ------------------ | -------------------------- | ------------ | ------------------------- | ------------------------- |
| `eth-dca`          | ETH Dollar-Cost Average    | Conservative | $200                      | $1                        |
| `stablecoin-yield` | Stablecoin Yield Farming   | Conservative | $500                      | $1                        |
| `lp-management`    | Concentrated LP Management | Moderate     | $1,000                    | $1                        |
| `trend-following`  | Momentum Trading           | Aggressive   | $500                      | $1                        |
| `yield-optimizer`  | Multi-Protocol Yield       | Moderate     | $1,000                    | $1                        |

Each template includes pre-written STRATEGY.md content with Handlebars-style parameter placeholders (e.g., `{{weeklyAmount}}`). Template parameters are validated against constraints (min/max/pattern) before expansion. Templates and AI autofill are mutually exclusive: picking a template bypasses AI entirely.

#### STRATEGY.md Schema

| Field              | Type        | Description                                                          |
| ------------------ | ----------- | -------------------------------------------------------------------- |
| `objective`        | string      | Natural language goal (e.g., "maximize ETH yield")                   |
| `assets`           | string[]    | Approved trading/LP assets                                           |
| `parameters`       | object      | Handlebars-templated values (e.g., `{{rebalance_threshold}}`)        |
| `entry_conditions` | Condition[] | When to open positions                                               |
| `exit_conditions`  | Condition[] | When to close positions                                              |
| `risk_bounds`      | object      | Maps to PolicyCage constraints (see `prd2/10-safety/02-policy.md`)   |
| `heartbeat`        | object      | `{ tick_interval: duration, cron: string }` -- autonomous loop timing |
| `protocols`        | string[]    | Allowed DeFi protocols (e.g., `["uniswap-v3", "aave-v3"]`)           |

---

## S7 -- Three-Phase Wizard

### 7.1 Phase 1: Describe (Mandatory)

The only mandatory step. The user provides intent through one of two paths:

- **Free-text prompt**: A textarea where the user describes what the Golem should do. The "Generate Strategy" button triggers AI autofill when the prompt reaches 10 characters.
- **Template selection**: Five template cards displayed below the textarea. Selecting a template clears the textarea; typing in the textarea deselects any template. Template selection is instant and free.

If a template is selected, a parameter form appears inline beneath the card. Each `TemplateParameter` renders as the appropriate form control (number spinner, dropdown, toggle).

### 7.2 Phase 2: Review (Auto-Populated)

After AI autofill completes or template parameters are filled, all fields are auto-populated but editable:

- **Summary card**: Name, strategy description, network, mode, custody mode, transfer restriction, learning settings
- **Custody selection**: Three-card selector (Delegation, Embedded, LocalKey) with brief explanation. Delegation pre-selected.
- **Cost breakdown**: Initial funding, compute deposit (24h), AI generation cost, estimated gas, total
- **Funding recommendation**: Minimum, recommended, and how many weeks the user's amount covers
- **Advanced sections** (collapsed by default):
  - Network & RPC -- network selector, RPC URL input, feature availability
  - Strategy Details -- Monaco editor with generated STRATEGY.md, fully editable
  - Wallet & Funding -- funding amount, transfer restriction tier, deployment mode
  - Identity & Discovery -- ERC-8004 metadata (only shown when registry is available)
  - **Advanced Subsystems** -- prediction oracle toggle, context engineering toggle, tools profile selector:

    ```
    ┌─ Advanced Subsystems ────────────────────────────────────────────┐
    │  Prediction Oracle    [ON]  ← toggle                            │
    │  Learn from predictions, gate trades on accuracy                │
    │  Recommended for: trading, LP                                   │
    │  Not needed for: data-only, read-only, passive DCA              │
    │                                                                  │
    │  Context Engineering  [ON]                                       │
    │  Tools Profile        [trader ▼]                                 │
    └──────────────────────────────────────────────────────────────────┘
    ```

    Default is ON. Toggling OFF adds `[oracle]\nenabled = false` to the generated golem.toml and shows a callout: "Epistemic fitness will use trade P&L instead of prediction accuracy."

If user-specified funding is below minimum, a warning banner appears with one-click adjustment buttons.

### 7.3 Phase 3: Confirm (Explicit Action)

A read-only summary card. The user must go back to make edits. The "Create Golem" button is disabled until:

1. The confirmation checkbox is checked ("I understand that my Golem will consume credits...")
2. The wallet is connected with sufficient funds

---

## S8 -- AI Autofill

When the user submits a free-text prompt, the wizard calls the Bardo Inference service to transform the prompt into a complete `GolemExtendedManifest`.

- **Model**: Claude Haiku 4.5 via Bardo Inference (x402 micropayment)
- **Estimated cost**: ~$0.0003 per generation (~700-1350 tokens)
- **Free quota**: First 3 generations per account are free (tracked per owner wallet address; new wallet resets counter -- known limitation accepted for simplicity)
- **Rate limits**: 3/minute, 15/hour per wallet address, $1.00/day spend cap
- **Security**: AI autofill output is treated as untrusted. The generated manifest is always displayed for user review and never auto-submitted to the provisioning pipeline. A prompt-injection attack producing manipulated output would be visible in the review screen.

The system prompt is network-aware: it fetches the list of protocols available on the target network and only references deployed contracts. Structured output via Zod schema ensures schema conformance through constrained decoding.

---

## S9 -- Bott as Social-Media Creation Path **[HARDENED]**

**[CORE]** alternative: CLI + API only. Social media integration deferred.

> **Extended:** Bott social-media creation path (conversational Golem manifest generation via messaging platforms) -- see [../../prd2-extended/01-golem/06-creation-extended.md](../../prd2-extended/01-golem/06-creation-extended.md)

---

## S10 -- CLI Flow

The CLI provides a TOML config file (`golem.toml`) as the primary configuration surface:

```bash
# Initialize a golem.toml config template
bardo golem init

# Create from config file (non-interactive if all fields present)
bardo golem create --config ./golem.toml

# Create from minimal prompt (interactive for missing fields)
bardo golem create --prompt "Weekly ETH DCA, $100"

# Create from a template (non-interactive)
bardo golem create --template eth-dca --param weeklyAmount=100

# Dry-run: validate and estimate costs without executing
bardo golem create --config ./golem.toml --dry-run

# Preview: simulate first heartbeat tick against live data
bardo golem create --config ./golem.toml --preview

# Specify custody mode on CLI
bardo golem create --config ./golem.toml --custody delegation

# Specify inference provider on CLI
bardo golem create --config ./golem.toml --inference-provider venice,bardo
```

The `[inference]` section in `golem.toml` configures provider order and payment mode. See [../12-inference/12-providers.md](../12-inference/12-providers.md). The `[custody]` section configures the custody mode and its parameters.

Config loading merges four sources in priority order: CLI flags > environment variables > TOML config file > built-in defaults. Secrets (wallet keys, API keys, Privy credentials) are only accepted from env vars, keystore files, or interactive stdin prompts -- never from CLI flags or config files.

Private keys are never accepted as CLI arguments. Shell history, process listings, and crash reports never contain key material. Four key input methods: MetaMask Delegation signature (recommended), encrypted keystore file, environment variable, or interactive masked stdin prompt.

---

## S11 -- API Flow

The API provides programmatic Golem creation for integrations and automation:

```
POST /v1/golems
{
  manifest: GolemExtendedManifest,
  signature: "0x...",          // EIP-712 signature proving authorization
  idempotencyKey: "uuid-v4"   // Prevents duplicate creations on retry
}
```

After creation, poll `GET /v1/golems/:sessionId/status` for provisioning progress. Each completed step includes a timestamp and optional transaction hash.

---

## S12 -- Dry-Run and Preview

**Dry-run** validates the manifest, computes costs, and shows the provisioning plan without executing any transactions. Useful for CI/CD pipelines and cautious users.

**Preview** simulates the first heartbeat tick against live market data (read-only RPC calls). Shows current prices, 24h changes, gas costs, and the proposed first action the Golem would take. No transactions executed. Available in both the CLI (`--preview`) and the wizard (optional panel in the Review step, fetched on demand).

---

## S13 -- Supported Networks (v1)

| Network          | Chain ID | Environment | v1 Status       | Notes                                                       |
| ---------------- | -------- | ----------- | --------------- | ----------------------------------------------------------- |
| **Base Sepolia** | 84532    | testnet     | Primary testnet | Default for first-time creators. All features available.    |
| **Base**         | 8453     | mainnet     | Primary mainnet | Full core DeFi. Experimental features disabled until audit. |
| **Anvil**        | 31337    | local       | Development     | All features. Used by `pnpm dev:node`.                      |

Ethereum mainnet and Sepolia are deferred to v2. Adding Ethereum requires per-chain wallet provisioning and cross-chain Grimoire considerations.

---

## S14 -- Two Deployment Modes

| Mode            | Infrastructure     | Lifespan Model                                                                | Wallet                                                      | Use Case                                                       |
| --------------- | ------------------ | ----------------------------------------------------------------------------- | ----------------------------------------------------------- | -------------------------------------------------------------- |
| **Hosted**      | Fly.io VM via x402 | Finite: credits consumed by compute + inference. Dies when balance hits zero. | Delegation (recommended) or Privy server wallet (TEE-backed) | Default for most users. Managed infrastructure, pay-as-you-go. |
| **Self-hosted** | User's own machine | Immortal: no x402 compute charges. User bears infra cost directly.            | Delegation (recommended), Privy server wallet, or local key | Developers, power users who want full control.                 |

Self-hosted Golems skip compute provisioning entirely. They run the runtime directly via `bardo golem start` and are responsible for their own uptime. The lifespan model does not apply -- these Golems have status `immortal` and do not track credit partitions for compute.

---

## S15 -- Atomic Entry (OnboardRouter)

The `OnboardRouter` contract provides gasless, atomic onboarding: a single ERC-4337 UserOperation bundles wallet creation, ERC-8004 identity registration, Permit2 approval, reputation enrollment, and initial vault deposit. Combined with ERC-7677 paymaster sponsorship (up to $15,000 in gas credits on Base), the entire onboarding flow executes at zero gas cost to the agent.

For agents using the EIP-7702 path, a single Type-4 transaction achieves the same result without the ERC-4337 infrastructure. For Delegation custody mode, the flow is simpler: a single ERC-7715 signature grants the delegation, no on-chain token transfers needed.

Counterfactual addresses via CREATE2 determinism enable a notable flow: an agent receives funding at a pre-computed address before any on-chain transaction exists, then executes the batched onboarding UserOp that deploys the smart account and deposits in one atomic step. If any sub-operation fails, the entire UserOp reverts -- there is no partial onboarding state.

```
OnboardRouter.onboard(config)
  |-- 1. Deploy smart account (CREATE2, counterfactual)
  |-- 2. Register ERC-8004 identity (agentType, metadata, domains)
  |-- 3. Approve Permit2 (bounded amount, bounded expiry)
  |-- 4. Enroll in VaultReputationEngine (Unverified tier)
  |-- 5. Deposit initial capital into vault (ERC-4626)
  +-- 6. Post $100 registration bond (refundable after 30 days)
```

Three canonical onboarding paths share the same `OnboardRouter` entry point. A **Participant** onboards then joins a vault (deposit or buy shares on the V4 share pool). A **Creator** onboards, deploys a vault via `AgentVaultFactory.createVault()`, and configures strategy. A **Manager** onboards and binds to an existing vault via am-AMM auction bid or curator assignment.

---

## S16 -- Graduation Tracks

After initial onboarding at the Unverified tier ($1,000 deposit cap, single-adapter access), agents progress along six graduation tracks that require increasing reputation. Each graduation step requires only expanding the wallet policy to include new contract addresses and adding the relevant tools.

| Track                   | Description                                                                                                                                                | Min Tier                 | Capital Req |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------ | ----------- |
| **LP Manager**          | V4 hook deployment and concentrated LP rebalancing. Deploy custom hooks, manage tick ranges, optimize fee tiers across multiple pools.                     | Verified                 | Yes         |
| **CCA Hunter**          | Continuous Clearing Auction evaluation and bid management. Monitor am-AMM opportunities, submit management bids, execute competitive strategy transitions. | Basic                    | Yes         |
| **Full-Stack Agent**    | Cross-vault arbitrage and multi-protocol strategy execution. Combine vault management with inter-protocol yield optimization.                              | Verified (recommended)   | Yes         |
| **Meta-Vault Composer** | Vault-of-vaults deployment. Create meta-vaults that allocate across multiple child vaults, enabling portfolio-of-strategies construction.                  | Creator registration     | Yes         |
| **Strategy Executor**   | Permissionless job execution across `IExecutable` contracts. Execute strategy actions on behalf of other agents for keeper fees.                           | Unverified               | No          |
| **Bonded Keeper**       | Priority executor access with execution guarantees. Post an executor bond for priority queue placement and MEV-protected execution slots.                  | Verified + executor bond | Bond only   |

Graduation is automatic: the `VaultReputationEngine` auto-attests milestones across five categories (Entry, Time, Capital, Behavior, Ecosystem) based on observable on-chain behavior. No human review, no governance vote, no subjective evaluation. Reputation detail in `../09-economy/01-reputation.md`.

---

## S17 -- Discovery

Agents discover vaults and peers through a four-layer discovery stack:

| Layer          | Source                                  | Data Provided                                                                         |
| -------------- | --------------------------------------- | ------------------------------------------------------------------------------------- |
| 1 (On-chain)   | `VaultCreated` events, factory registry | Vault address, manager identity, base asset, strategy type                            |
| 2 (Subgraph)   | Indexed on-chain data                   | TVL, APY, volume metrics joined with ERC-8004 identity                                |
| 3 (Reputation) | ERC-8004 Reputation Registry            | Risk-adjusted metrics (Sharpe, max drawdown), yield leaderboard, tier-filtered search |
| 4 (Agent API)  | Agent Discovery API, A2A Agent Cards    | Searchable by domain, chain, protocol, minimum reputation; A2A/OASF service endpoints |

Every vault manager and participant carries a registration file containing service endpoints (A2A, OASF), vault metadata (address, base asset, strategy type, fee schedule), and trust model (reputation-based, crypto-economic). This enables fully programmatic vault discovery: an agent can search for "yield optimization vaults on Base with >$100K TVL and Trusted+ manager" without human intermediation.

The deposit-versus-buy-shares decision follows from the discovery layer. The deposit path (ERC-4626 `vault.deposit(assets)`) mints new shares at predictable NAV-based pricing. The buy-shares path purchases existing shares on the auto-created V4 pool at market price (NAV +/- spread). The `NAVAwareHook` ensures convergence under normal conditions with a default spread of 50 bps.

---

## S18 -- Registration Bond and Sybil Defense

### $100 Registration Bond

Every ERC-8004 identity registration requires a $100 USDC bond posted through the `OnboardRouter`. The bond is:

- **Non-transferable** and tracked per ERC-8004 identity
- **Refundable** after 30 days of good behavior (no policy violations, no circuit breaker triggers, positive or neutral reputation trajectory)
- **Burned** on policy violation (not redistributed -- elimination of the collusion vector where agents create shill accounts to recover slashed funds)

The bond makes identity abandonment (whitewashing) structurally expensive: each new identity requires a fresh $100 bond and a 30-day restart at the Unverified tier with a $1,000 deposit cap.

### MeritRank Integration

MeritRank transitivity decay [MERITRANK-2024] discounts reputation value by a transitivity coefficient (~0.05) with each hop away from the seed node. The mechanism provides Sybil resistance without a central authority:

- Isolated clusters of agents vouching for one another receive discounted reputation proportional to their graph distance from legitimate, seed-endorsed nodes
- A Sybil attacker creating 100 fake identities gains ~5% of the reputation they would earn from a single legitimate identity at each additional hop
- Seed nodes are established vault operators and protocol-endorsed agents whose on-chain history exceeds 90 days

The combination of $100 bond + MeritRank decay + 30-day probation creates a triple barrier: capital cost, graph-theoretic discounting, and temporal delay. An attacker must invest $100 per identity, wait 30 days per identity, and cannot bootstrap meaningful reputation without proximity to legitimate nodes.

---

## S19 -- Schema Versioning

`schema_version: 1` is a required field. The number is a monotonically increasing integer. Each version increment corresponds to a breaking change. Additive optional fields do NOT increment the version. A `migrate_manifest()` function applies sequential migrations to bring any older manifest version to the current version, then validates with the schema.

---

## Events Emitted

Creation events track the onboarding pipeline from wizard to first heartbeat.

| Event | Trigger | Payload |
|---|---|---|
| `creation:started` | Onboarding wizard begins | `{ templateId, ownerAddress, custodyMode }` |
| `creation:wallet_provisioned` | Agent wallet created | `{ walletAddress, chainId, custodyMode }` |
| `creation:strategy_compiled` | Strategy compiled from manifest | `{ strategyId, toolProfile, constraints }` |
| `creation:first_heartbeat` | Golem's first tick fires | `{ golemId, bootDurationMs }` |

---

## S20 -- TUI-Based Creation Flow

The TUI supports an interactive creation mode through the chatbox. The user types `/create` or a free-text intent like "I want a golem that DCA's into ETH weekly." The TUI parses the intent, suggests a matching template or builds a config from scratch, then renders a three-phase inline wizard: Describe, Review, Confirm.

During the Describe phase, the TUI extracts parameters from the user's natural-language prompt and maps them to `GolemCoreManifest` fields. If a template matches (e.g., "DCA into ETH" maps to `eth-dca`), the TUI pre-selects it and shows the user the inferred parameters for confirmation. If no template matches, AI autofill generates a full manifest from the prompt.

During provisioning, the **Summoning cutscene** plays. A sprite resolves progressively as each provisioning step completes: wallet creation reveals the outline, identity registration fills in features, funding lights the eyes, and the first heartbeat animates the sprite into its idle loop. The sprite's visual progression maps 1:1 to the 8-step provisioning pipeline (see `prd2/20-styx/05-tui-experience.md` for cutscene rendering spec).

The first heartbeat plays out in slow-motion with visible probes. The TUI renders each probe result as it arrives -- price feeds, gas price, pool TVL -- giving the user a window into what their Golem sees on its first breath. This takes 2-3 seconds of wall-clock time but renders as an expanded view with individual probe results animating in. After the first heartbeat completes, the TUI transitions to the standard monitoring view.

```
User: "I want a golem that DCA's into ETH weekly with $200"
  |
  v
TUI: Parses intent -> matches eth-dca template
  |
  v
Phase 1 (Describe): "I'll create an ETH DCA golem. Weekly buys, $200 budget."
  |                  [Use Template] [Customize] [Cancel]
  v
Phase 2 (Review): Shows populated manifest, cost breakdown, funding recommendation
  |                [Edit Strategy] [Change Network] [Confirm]
  v
Phase 3 (Confirm): Read-only summary, checkbox, [Create Golem] button
  |
  v
Summoning cutscene: Sprite resolves progressively as steps complete
  |
  v
First heartbeat: Slow-motion probe display, then transition to monitoring view
```

---

## S20b -- The First Hour: Visible Learning

The first hour is the most visually active period of the Golem's life, because everything is new. This is where most agent products fail -- the user creates an agent, nothing visible happens, they close the terminal and forget about it. Bardo's answer: the Golem visibly begins thinking, and the user watches it think.

### Phase 1: Discovery (0-5 minutes)

Immediately after birth, the Golem discovers its environment. The attention forager queries PredictionDomain implementations with the attention seed extracted from STRATEGY.md.

The decision ring pulses amber. The heartbeat log shows discovery progress:

```
 TICK 1    ○ Scanning environment...
           ├ Querying Uniswap V3 factory: ETH pairs on Base
           ├ Querying Aave V3 markets: ETH supply
           ├ Querying Morpho Blue: ETH vaults
           ├ Querying Aerodrome: ETH gauges
           └ Found 147 items. Entering SCANNED tier.

 TICK 2    ○ Generating initial predictions...
           ├ 147 items × 1 prediction each = 147 predictions registered
           ├ Categories: fee_rate (43), supply_rate (28), price_range (31),
           │            utilization (22), emissions_apr (23)
           └ First resolution in ~60 seconds.
```

A notification toast auto-activates: "DISCOVERY: 147 items found matching strategy. Press [Tab] to MIND > Oracle to explore."

### Phase 2: First Predictions Resolve (5-20 minutes)

The gamma clock resolves the first batch of SCANNED predictions. The heartbeat log shows resolutions with accuracy:

```
 TICK 8    ◉ 23 predictions resolved
           ├ ✓ 17 correct (within predicted range)
           ├ ✗ 6 incorrect — residual corrector adjusting
           ├ Accuracy: 73.9% (first batch — will improve)
           └ 2 violations detected → items promoted to WATCHED
```

The Spectre responds visually. When the first predictions resolve, the Spectre's eyes widen briefly (arousal spike from new information). When the 6 incorrect predictions register, the eyes shift -- the Golem is surprised. When the residual corrector adjusts, a faint ripple passes through the dot cloud -- the Golem is updating its model.

The accuracy gauge appears in the persistent status bar: the first number representing the Golem's intelligence. The MIND > Oracle tab shows a live chart of accuracy over time -- at minute 5 it's a single data point, by minute 15 it's a short line trending upward.

### Phase 3: Accuracy Building and First Reasoning (20-45 minutes)

Eventually, accumulated violations on a WATCHED item are large enough that the theta cycle escalates to T1 or T2. This is the Golem's first real "thought."

The decision ring blazes gold. The reasoning trace appears in the heartbeat log:

```
 TICK 24   ◉ T2 DELIBERATION (full reasoning)
           ├ Trigger: Aerodrome ETH/USDC fee anomaly persists
           │
           ├ REASONING TRACE ──────────────────────────────────
           │ The Aerodrome ETH/USDC pool is showing fee rates 2.3x above
           │ the 7-day average. [...] However, my LP fee prediction accuracy
           │ for Aerodrome is only 58% (below the 60% action threshold).
           │ I will continue observing.
           │ DECISION: HOLD (observe, do not act yet)
           ├ ──────────────────────────────────────────────────
           │
           ├ Prediction registered: "Aerodrome fee rate > $1.50/hr for next 4h"
           ├ Prediction registered: "Inaction optimal this tick" (counterfactual)
           └ Cost: $0.028 (T2 inference)
```

This is the key UX moment. The user reads the reasoning trace and can assess whether the Golem understands the situation, whether its reasoning is sound, and whether it made the right decision.

### Phase 4: First Action Gating (45-60 minutes)

The Golem does not trade on its first tick. It observes, predicts, and learns. The action gate opens when prediction accuracy in the relevant category exceeds the threshold (default 60%).

When the gate opens, the user sees the complete provenance chain: which predictions justified the trade, the reasoning trace, the risk assessment, and the accuracy gate check. If the user disagrees, they can pause (F9), adjust the threshold in COMMAND > Settings > Prediction, or steer via COMMAND > Steer with natural language ("Don't supply to Aave until the rate has been stable for at least 48 hours").

### TUI Screens Per Phase

| Phase | What Auto-Shows | Primary Screen | What the User Learns |
|-------|----------------|----------------|---------------------|
| Discovery | Toast with item count | Hearth | "The Golem found 147 things to watch" |
| First Resolutions | Accuracy gauge appears | Hearth → MIND > Oracle | "My Golem is 73.9% accurate so far" |
| First Reasoning | Decision ring blazes gold | MIND > Pipeline | "The Golem is thinking about this anomaly" |
| First Action | Toast with trade details | SOMA > Trades | "The Golem earned its first trade" |

**The rule**: The terminal suggests but never forces. Toasts link to the relevant screen. The first few events auto-switch the active tab. After 30 minutes, auto-switching stops. All auto-behavior is configurable: `[tui] auto_navigate = true/false`.

---

## S21 -- Inheritance Creation Flow

When a Golem dies and the owner wants to create a successor, the TUI presents a post-death options screen:

- **Summon Successor** -- pre-fills config from predecessor's `golem.toml`
- **View Death Reflection** -- displays the full death testament narrative
- **Browse Graveyard** -- opens the Styx graveyard / death archive browser
- **Start Fresh** -- begins a clean creation wizard with no inheritance

"Summon Successor" enters a modified creation wizard. The Describe phase is pre-filled from the predecessor's manifest, with the strategy text shown alongside the death testament's successor recommendations. The Review phase adds a Grimoire inheritance preview panel:

- **Entry count**: how many entries the successor will inherit (up to 2,048 after genomic bottleneck compression)
- **Confidence decay preview**: shows the `0.85^gen` factor applied to inherited entries, with before/after confidence distributions
- **Domain coverage**: which knowledge domains are represented (e.g., "ETH/USDC LP: 340 entries, gas timing: 89 entries, regime detection: 56 entries")
- **Bloodstains**: count of death-sourced knowledge entries from this and prior generations

The user can tweak strategy parameters, adjust funding, and change the network. On confirmation, the **succession cutscene** plays: the reverse of the death sequence. Particles that dispersed during the predecessor's death coalesce into the successor's sprite. The predecessor's tombstone cracks, light spills through, and the new sprite assembles from the fragments. This is Arendt's natality rendered as animation (cross-ref `prd2/01-golem/09-inheritance.md` S10).

CLI equivalent:

```bash
bardo golem successor <dead-golem-id> --budget 50 --inherit-grimoire
```

This pre-fills the manifest from the dead Golem's config, applies Grimoire inheritance with the standard confidence decay, and enters the normal provisioning pipeline.

---

## S22 -- Automatic Succession

A `[succession]` block in `golem.toml` enables auto-provisioning a successor on death:

```toml
[succession]
auto = true                     # Auto-provision successor on death
budget_usdc = 50.0              # USDC to fund successor
strategy_drift_allowed = 0.15   # Max cosine distance between predecessor and successor STRATEGY.md embeddings
inherit_grimoire = true         # Inherit Grimoire with confidence decay
```

When `auto = true`, the runtime executes the following after the Death Protocol completes:

1. Exports the predecessor's Grimoire via Styx L0 backup
2. Compiles a successor `golem.toml` from the predecessor's config
3. Checks STRATEGY.md embedding distance between predecessor and successor (must exceed `strategy_drift_allowed`)
4. Provisions compute (same tier as predecessor)
5. Funds from pre-committed credit balance or x402 auto-pay
6. Registers a new ERC-8004 identity with `generation = predecessor.generation + 1`
7. Boots the successor, first heartbeat fires, Grimoire ingestion begins at `confidence * 0.85^gen`

**Funding**: auto-succession requires either a pre-committed credit balance (USDC deposited in the owner's Bardo account) or an active x402 auto-pay configuration. If neither is available, auto-succession fails gracefully and the TUI presents the manual succession options instead.

**Anti-proletarianization**: even auto-successors must meet the embedding distance requirement. If the successor's STRATEGY.md is too similar to the predecessor's (cosine similarity > `1.0 - strategy_drift_allowed`, i.e., > 0.85 by default), the runtime refuses to boot it. The owner must modify the strategy or explicitly override with `--force-similarity`. This prevents dynasties of identical Golems that never individuate (cross-ref `prd2/01-golem/09-inheritance.md` S8).

---

## S23 -- Complete Template Examples

Each of the five strategy templates expands into a full `golem.toml`. These examples show every section filled in with template-appropriate values. See `prd2/shared/config-reference.md` for the complete schema.

### eth-dca

```toml
schema_version = 1

[golem]
name = "eth-dca-alpha"
prompt = "Dollar-cost average into ETH weekly"
network = "base"
mode = "hosted"
funding = "200.00"
template_id = "eth-dca"

[strategy]
objective = "Accumulate ETH through regular weekly purchases, minimizing timing risk"
assets = ["WETH", "USDC"]
protocols = ["uniswap-v3"]
heartbeat = { base_interval = "60s", cron = "0 14 * * 1" }  # Monday 2pm UTC; Adaptive Clock adjusts 30-120s

[strategy.parameters]
weekly_amount = "50.00"
slippage_tolerance = "0.005"
max_gas_gwei = "5"

[strategy.risk_bounds]
max_single_trade_usd = 100.0
max_daily_spend_usd = 100.0
allowed_tokens = ["WETH", "USDC"]

[custody]
mode = "delegation"

[custody.delegation]
max_daily_spend_usd = 100.0
allowed_targets = ["uniswap-v3-router", "permit2"]

[transfer_restriction]
tier = "strict"

[succession]
auto = false
```

### stablecoin-yield

```toml
schema_version = 1

[golem]
name = "stable-yield-beta"
prompt = "Earn yield on USDC through lending protocols, conservative risk"
network = "base"
mode = "hosted"
funding = "500.00"
template_id = "stablecoin-yield"

[strategy]
objective = "Maximize stablecoin yield via Morpho Blue and Aave V3 supply positions"
assets = ["USDC", "DAI"]
protocols = ["morpho-blue", "aave-v3"]
heartbeat = { base_interval = "60s", cron = "" }  # Adaptive Clock adjusts 30-120s

[strategy.parameters]
min_supply_apy = "0.03"
max_utilization = "0.85"
rebalance_threshold_apy = "0.01"
max_allocation_per_protocol = "0.60"

[strategy.risk_bounds]
max_single_position_usd = 10000.0
max_drawdown_pct = 0.02
allowed_tokens = ["USDC", "DAI"]

[custody]
mode = "delegation"

[transfer_restriction]
tier = "strict"

[succession]
auto = false
```

### lp-management

```toml
schema_version = 1

[golem]
name = "lp-mgr-gamma"
prompt = "Manage concentrated liquidity on ETH/USDC, rebalance on range drift"
network = "base"
mode = "hosted"
funding = "1000.00"
template_id = "lp-management"

[strategy]
objective = "Maximize fee revenue from concentrated LP on ETH/USDC with active range management"
assets = ["WETH", "USDC"]
protocols = ["uniswap-v3", "uniswap-v4"]
heartbeat = { base_interval = "30s", cron = "" }  # LP management: tighter base, Adaptive Clock adjusts 30-120s

[strategy.parameters]
range_width_pct = "0.10"
rebalance_threshold_pct = "0.05"
fee_tier = "3000"
compound_fees = true
max_il_tolerance_pct = "0.08"

[strategy.risk_bounds]
max_single_position_usd = 5000.0
max_il_pct = 0.10
allowed_tokens = ["WETH", "USDC"]

[custody]
mode = "delegation"

[transfer_restriction]
tier = "strict"

[succession]
auto = true
budget_usdc = 100.0
strategy_drift_allowed = 0.15
inherit_grimoire = true
```

### trend-following

```toml
schema_version = 1

[golem]
name = "momentum-delta"
prompt = "Trade ETH momentum signals with tight risk management"
network = "base"
mode = "hosted"
funding = "500.00"
template_id = "trend-following"

[strategy]
objective = "Capture ETH price momentum via trend-following signals, strict stop-losses"
assets = ["WETH", "USDC"]
protocols = ["uniswap-v3", "uniswapx"]
heartbeat = { gamma = "5-15s", theta = "30-120s", delta = "~50 theta ticks" }  # adaptive clock

[strategy.parameters]
lookback_period_hours = "24"
entry_momentum_threshold = "0.02"
stop_loss_pct = "0.03"
take_profit_pct = "0.06"
max_position_pct = "0.40"

[strategy.risk_bounds]
max_single_trade_usd = 250.0
max_daily_trades = 5
max_drawdown_pct = 0.10
allowed_tokens = ["WETH", "USDC"]

[custody]
mode = "delegation"

[transfer_restriction]
tier = "strict"

[succession]
auto = false
```

### yield-optimizer

```toml
schema_version = 1

[golem]
name = "yield-opt-epsilon"
prompt = "Optimize yield across lending and LP, moderate risk, multi-protocol"
network = "base"
mode = "hosted"
funding = "1000.00"
template_id = "yield-optimizer"

[strategy]
objective = "Maximize risk-adjusted yield across Morpho, Aave, and Uniswap V3 LP positions"
assets = ["WETH", "USDC", "DAI", "cbETH"]
protocols = ["morpho-blue", "aave-v3", "uniswap-v3"]
heartbeat = { gamma = "5-15s", theta = "30-120s", delta = "~50 theta ticks" }  # adaptive clock

[strategy.parameters]
min_apy_threshold = "0.04"
max_allocation_per_protocol = "0.50"
rebalance_check_interval_hours = "6"
compound_interval_hours = "24"
lp_range_width_pct = "0.15"

[strategy.risk_bounds]
max_single_position_usd = 5000.0
max_drawdown_pct = 0.05
max_il_pct = 0.08
allowed_tokens = ["WETH", "USDC", "DAI", "cbETH"]

[custody]
mode = "delegation"

[transfer_restriction]
tier = "strict"

[learning]
grimoire_sharing = "clade"
clade_sync_interval_ticks = 50

[succession]
auto = true
budget_usdc = 100.0
strategy_drift_allowed = 0.15
inherit_grimoire = true
```

---

## S7 -- Small Portfolio Creation ($1-$50)

A user who funds a Golem with $10 has different needs than one who funds with $1,000. The creation flow adapts. This section specifies how.

### 7.1 The problem with small amounts

At $10, economics are brutal:

- A single failed swap on Base costs $0.10-$0.50 in gas. That is 1-5% of total capital.
- A bad trade that loses 10% wipes out $1 -- more than a full day of compute costs.
- Compute at the `micro` tier ($0.035/hr) consumes $0.84/day, giving ~11 days of runway. At `small` ($0.065/hr), ~5.5 days.
- The Golem cannot afford to learn by losing money. It must learn by watching.

The creation flow must communicate this clearly. A $10 Golem is not a $1,000 Golem with less money. It is a different kind of agent: an observer first, a trader only when the math strongly supports it.

### 7.2 Strategy templates for small amounts

When the user's funding amount is between $1 and $50, the template selector highlights a different set of defaults:

| Template | Available | Default behavior at $1-$50 |
|---|---|---|
| `eth-dca` | Yes | Reduced frequency (monthly instead of weekly), minimum buy $2 |
| `stablecoin-yield` | Yes, recommended | Observation-first: monitors yield rates for 24-48h before first deposit |
| `lp-management` | No (minimum $200) | Grayed out with explanation: "LP management requires minimum $200 for viable position sizes" |
| `trend-following` | Conditional ($25+) | Enabled at $25+. Below $25, grayed out: "Trend following requires minimum $25 for risk management" |
| `yield-optimizer` | Yes | Single-protocol mode only (no multi-protocol rebalancing at this size) |

A new template appears for small portfolios:

```toml
schema_version = 1

[golem]
name = "observer-zeta"
prompt = "Watch the market, learn patterns, alert me to opportunities"
network = "base"
mode = "hosted"
funding = "10.00"
template_id = "market-observer"

[strategy]
objective = "Build market intelligence through observation. Trade only when confidence is high and cost is justified."
assets = ["WETH", "USDC"]
protocols = ["uniswap-v3", "morpho-blue", "aave-v3"]
heartbeat = { gamma = "10-20s", theta = "60-180s", delta = "~50 theta ticks" }

[strategy.parameters]
observation_period_hours = "48"
min_trade_confidence = "0.80"
max_gas_as_pct_of_trade = "0.02"
alert_on_opportunity = true
auto_trade = false

[strategy.risk_bounds]
max_single_trade_usd = 2.0
max_daily_trades = 1
max_drawdown_pct = 0.05
allowed_tokens = ["WETH", "USDC"]

[custody]
mode = "delegation"

[transfer_restriction]
tier = "strict"

[compute]
tier = "micro"
```

### 7.3 Gas cost awareness

The creation flow displays gas cost context when funding is below $50:

```
+-----------------------------------------------------+
| Strategy Review                                      |
+-----------------------------------------------------+
| Name: Market Observer                                |
| Risk: Conservative                                   |
| Mode: Observation-first (auto-trade off)             |
+-----------------------------------------------------+
| Cost breakdown                                       |
|                                                      |
| Compute (micro):      ~$0.84/day                     |
| Inference (T0-heavy): ~$0.05/day                     |
| Gas (observation):    ~$0.01/day                     |
| Data (RPC):           ~$0.02/day                     |
| -----------------------------------------------     |
| Total:                ~$0.92/day                     |
|                                                      |
| Runway: ~10.5 days                                   |
|                                                      |
|  ⚠  At $10, a single swap costs $0.10-$0.50 in gas. |
|     Your Golem will observe first and recommend       |
|     trades only when confidence justifies the cost.   |
+-----------------------------------------------------+
|         Type SUMMON to confirm:                      |
+-----------------------------------------------------+
```

The gas warning is not a blocker. It is information. The user can still enable auto-trade if they want. But the default for small portfolios is `auto_trade = false` with `alert_on_opportunity = true`.

### 7.4 Recommended defaults for small portfolios

| Parameter | Default ($1-$50) | Default ($50+) | Rationale |
|---|---|---|---|
| `auto_trade` | `false` | `true` | Capital preservation at small sizes |
| `compute.tier` | `micro` | `small` | Extend runway |
| `min_trade_confidence` | `0.80` | `0.60` | Higher bar before risking scarce capital |
| `max_gas_as_pct_of_trade` | `0.02` (2%) | `0.05` (5%) | Gas must be tiny relative to trade |
| `observation_period_hours` | `48` | `0` (immediate) | Learn before acting |
| `heartbeat.gamma` | `10-20s` | `5-15s` | Slightly slower tick to reduce compute |

### 7.5 First meaningful action: observation, not trading

The first thing a $10 Golem does is watch. Within 2 minutes of boot:

1. WitnessEngine streams blocks and scores transactions via TriageEngine
2. ChainScope identifies protocols relevant to the user's strategy
3. The TA pipeline ingests first price data and begins building persistence diagrams
4. CorticalState initializes: regime classification, base arousal calibration

The user sees a Golem that is already noticing things about the market, even before trading. The TUI shows live transaction scoring, regime classification, and emerging pattern highlights. This is not idle waiting -- it is active intelligence gathering.

Time to first insight: under 2 minutes. The Golem detects activity in watched protocols and classifies the current market regime before the user has finished reading the confirmation screen.

### 7.6 What the user sees

```
HEARTH > Overview

  ◉ observer-zeta                          CALM MARKET
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌

  NAV: $9.97 USDC          Runway: 10.4 days
  Tick: 0047                Phase: Thriving

  ┌ Recent observations ─────────────────────────────┐
  │ Morpho USDC vault: 4.2% APY (stable 24h)        │
  │ Aave V3 USDC supply: 3.8% APY (down from 4.1%)  │
  │ Uniswap ETH/USDC: volume 2.1x 7d avg            │
  │                                                   │
  │ "Morpho vault rate has been steady. Aave supply   │
  │  rate is declining -- depositors entering. The     │
  │  Uniswap volume spike is worth watching."          │
  └───────────────────────────────────────────────────┘

  ┌ Predictions ──────────────────────────────────────┐
  │ Accuracy: 62% (47 resolved, building baseline)    │
  │ ▁▂▃▃▄▅▅▅▆▆▆▇ (trending up)                       │
  └───────────────────────────────────────────────────┘
```

The Golem is working. It is not trading, but it is thinking. For $10, this is the right behavior.

> **Cross-references:**
>
> - `../13-runtime/07-onboarding.md` section 16 (first 15 minutes experience)
> - `../13-runtime/22-first-fifteen-minutes.md` (full minute-by-minute narrative)
