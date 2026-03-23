# Golem Tools -- Overview, Goals, and Personas [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `golem-tools` | **Workspace**: `crates/tools/`
>
> For market context and ecosystem data, see [shared/chains.md](../shared/chains.md) (supported EVM chains, RPC endpoints, block times, and chain-specific deployment addresses).
> See [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (how runtime extensions are loaded, activated, and lifecycle-managed within a running Golem) for how tools integrate with the Extension-based activation system.

---

> **Reader orientation:** This document is the top-level overview of the `golem-tools` crate, the DeFi tool library that ships with the Bardo runtime. It covers the crate's goals, design philosophy, trust-tier model, user personas, and the `ToolDef` pattern that every tool follows. You should be comfortable with Uniswap V3/V4 mechanics, concentrated liquidity, and common DeFi protocols (Aave, Morpho, Lido, Pendle). Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## What this is

`golem-tools` is a Rust-native tool library shipped as a Golem-RS runtime extension. It gives any Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) safe access to DeFi protocols -- Uniswap (V2/V3/V4/UniswapX), Aave, Morpho, Pendle, Lido, EigenLayer, GMX, and others -- across all deployed EVM chains. Data querying, trade execution, liquidity management, lending, staking, cross-chain intents, and defense-in-depth safety, all in one crate. No server process, no transport layer. Tools are called directly by the Pi runtime (the Golem's cognitive execution engine that orchestrates the heartbeat decision cycle) as in-process function invocations.

The crate contains 423+ DeFi tools organized into three trust tiers:

> **Tool count clarification:** The 423+ total breaks down as ~210 read-only tools, ~150 write tools, ~23 privileged tools across all supported protocols (per `rewrite4/11-tools.md`). The legacy "171+ Pi-native tools" figure in older docs refers to the gotts-monorepo TypeScript MCP tool count — a different codebase.

- **~250 read tools** (pool state, prices, positions, portfolio, health factors) -- no capability token required
- **~150 write tools** (swaps, LP operations, deposits, withdrawals, staking) -- require a `Capability<WriteTool>` token (a single-use authorization object enforced at compile time by Rust's move semantics), consumed on use
- **~23 privileged tools** (PolicyCage (the on-chain smart contract that enforces an owner's safety constraints) parameter changes, strategy modifications, risk limit adjustments) -- require capability token plus owner approval

Every tool is a `ToolDef` (a static Rust struct that declares the tool's name, description, trust tier, and handler function) with an `async fn handle(params, ctx)` handler. External agents access the same tools via the A2A interface (Google's Agent-to-Agent protocol, exposed as JSON-RPC 2.0), a separate binary that imports handlers from this crate.

Write tools are simulated before execution using an in-process Revm fork (Revm is a Rust EVM implementation that can replay transactions against a forked snapshot of live chain state) -- the Golem runs the transaction against current chain state locally, inspects balance deltas and gas costs, and only broadcasts after simulation succeeds. Uniswap SDK math (route optimization, concentrated liquidity calculations) runs in a co-located TypeScript sidecar process via Unix domain socket IPC (~1-5ms latency), because porting 50,000+ lines of SDK code to Rust would be months of work with ongoing maintenance burden. Untrusted third-party tools run in a WASM sandbox (Wasmtime) with fuel metering and a 256MB memory cap.

Tool names follow `<prefix>_<action>_<subject>`. Protocol prefixes: `uniswap_`, `aave_`, `morpho_`, `pendle_`, `lido_`, `eigenlayer_`, `gmx_`, `yearn_`. Subsystem prefixes: `data_`, `safety_`, `intel_`, `memory_`, `identity_`, `wallet_`, `vault_`, `stream_`, `testnet_`, `bootstrap_`. The `ALL_TOOL_DEFS` slice and `profile_registry` provide the full catalog. Profiles control which tools load at boot.

---

## Why it matters

Autonomous agents need to interact with DeFi protocols. Uniswap is the largest decentralized exchange by volume [UNISWAP-VOLUME-2025], but agents managing real capital also need lending (Aave, Morpho), liquid staking (Lido, Rocket Pool), restaking (EigenLayer), yield tokenization (Pendle), and hedging instruments (GMX, Panoptic). Today, agents face fragmented, unsafe options:

- **Existing agent tools** are low quality. Community-built tools require raw private keys in environment variables with no simulation or safety guards. Each covers a single narrow function. None use Uniswap's official Trading API or SDKs.
- **GOAT SDK** has a Uniswap plugin, but it is swap-only -- no data querying, LP management, lending, or safety layers.
- **No existing library** combines data + execution + safety across protocols. Agents wire together 3-4 separate integrations and still lack guardrails.

`golem-tools` eliminates this fragmentation. It is the authoritative tool library for AI agents interacting with DeFi, with safety guarantees -- compile-time capability tokens, Revm pre-flight simulation, token allowlists -- that prevent LLM hallucinations from causing fund loss.

---

## The problem it solves

1. **Fragmentation**: Agents lack a unified, safe tool library for DeFi. They wire together separate integrations for prices, pools, trading, lending, and positions from untrusted sources. `golem-tools` consolidates everything into protocol-prefixed `ToolDef` exports with consistent error handling and safety guarantees.
2. **Safety**: Existing tools have zero safety layers. An LLM that hallucinates a token address or amount can drain a wallet. Every write operation passes through Revm simulation, token allowlists, spending limits, and hallucination detection before execution. Write tools require a `Capability<T>` token that the Rust compiler enforces as single-use -- even a fully compromised LLM cannot reuse an authorization. The ClawHub incident (386 malicious skills) [CLAWHUB-2025] shows that safety must be structural, not optional.
3. **Completeness**: No existing tool supports LP management, V4 hooks, UniswapX, Permit2, cross-chain intents, lending health factor monitoring, or liquid staking. `golem-tools` covers Uniswap's full protocol surface plus Aave, Morpho, Lido, EigenLayer, Pendle, GMX, and Panoptic.
4. **Quality**: Existing tools bypass official SDKs and Trading APIs. This crate uses Alloy's (the standard Rust library for EVM interaction) `sol!` macro for type-safe on-chain bindings (compile-time ABI verification, zero-copy decoding, 60% faster U256 arithmetic than JavaScript BigInt) and delegates route optimization to the TypeScript sidecar running canonical Uniswap SDKs.
5. **Agent capital markets**: No existing tool supports x402 (a micropayment protocol that lets agents pay per-request without API keys) payments, ERC-8004 (an on-chain standard for registering and verifying autonomous agent identities) agent identity, or agent-optimized access patterns. `golem-tools` is designed for agent capital markets from day one.

---

## Design philosophy

- **Full autonomy**: Agents are configured once (wallet, policies, allowlists) and operate independently. No human clicks, no manual approvals, no browser flows.
- **Safety-first**: Every transaction is simulated in a local Revm fork before broadcast. Capability tokens enforce single-use authorization at compile time. Multiple independent safety layers (allowlists, spending limits, rate limiting, circuit breakers, hallucination detection) mean no single failure can result in fund loss.
- **Comprehensive**: One crate replaces five community integrations. Covers Uniswap (V2/V3/V4/UniswapX), Aave, Morpho, Lido, EigenLayer, Pendle, GMX, Panoptic, and more. LP management, cross-chain intents, lending, staking, and safety included.
- **Progressive disclosure via Pi's two-layer tool model**: The Golem's LLM sees 8 Pi-facing tools (`preview_action`, `commit_action`, etc.) that internally route to 423+ tool implementations. Profiles control which tool handlers load. 10 profiles cover the spectrum from read-only observatory to full active trading. Profiles are composable -- `TOOL_PROFILE=trader,vault` activates both. See [01-architecture.md](01-architecture.md) (the tool architecture spec covering ToolDef internals, trust tiers, capability tokens, safety hooks, profiles, and the TypeScript sidecar) for the profile system.
- **Rust-native**: Built on Alloy for all on-chain interaction. The `sol!` macro generates type-safe bindings from Solidity signatures at compile time. No ABI JSON, no codegen step, no runtime type errors.
- **Wallet-agnostic**: Works with any type implementing the Alloy `Signer` trait -- local keys, Privy, Safe, ZeroDev, or custom signers. Three custody modes: Delegation (owner's MetaMask Smart Account), Privy (hosted TEE), and Local (development only).
- **Pi-native by design**: Tools use Pi's `ToolContext` interface (`provider()`, `signer()`, `revm_fork()`), session lifecycle, streaming events, and shared typed memory. No server process, no transport protocol.

---

## Tool discovery

Three paths to find and load tools:

1. **`ALL_TOOL_DEFS` slice**: The barrel export from `golem_tools::tool_defs` contains every `ToolDef`. Filter by `.category` for programmatic discovery.
2. **`profile_registry`**: The `resolve_profile_categories(profile)` function maps profile names to allowed category sets. `ALL_TOOL_DEFS.iter().filter(|t| allowed.contains(&t.category))` gives the filtered list.
3. **`handlers` module**: The `golem_tools::handlers` module exports standalone handler functions for direct invocation without loading the full `ToolDef` metadata.

---

## ToolDef pattern

Every tool is a module in `crates/tools/src/tools/` exporting a `TOOL_DEF: ToolDef` static:

```rust
use golem_tools::{ToolDef, ToolContext, ToolResult, Category, RiskTier, TickBudget, CapabilityTier, SpriteTrigger};
use serde::{Deserialize, Serialize};

/// Input parameters for uniswap_get_pool_info.
#[derive(Debug, Deserialize)]
pub struct GetPoolInfoParams {
    /// Pool contract address (0x...).
    pub pool_address: String,
    /// Chain ID (default: 1 for Ethereum).
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,
}

fn default_chain_id() -> u64 { 1 }

pub static TOOL_DEF: ToolDef = ToolDef {
    name: "uniswap_get_pool_info",
    description: concat!(
        "Get current state of a Uniswap V3 or V4 pool: price, liquidity, TVL, volume, fees. ",
        "Returns tick, sqrtPriceX96, liquidity, 24h volume, and fee tier.",
    ),
    category: Category::Data,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Fetching slot0", "Loading subgraph data", "Computing APY"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Use uniswap_get_pool_info for pool state. Call uniswap_get_pools_by_token_pair first to get the pool address.",
    prompt_guidelines: &[
        "Prefer this over manual slot0 reads -- it normalizes V3/V4 differences.",
        "Cache results for 15s. Don't call twice in the same tick for the same pool.",
    ],
};

/// Handler implementation.
pub async fn handle(params: GetPoolInfoParams, ctx: &ToolContext) -> Result<ToolResult> {
    let provider = ctx.provider(params.chain_id)?;
    let slot0 = read_slot0(&provider, params.pool_address.parse()?).await?;
    // ... build result
    Ok(ToolResult::text(serde_json::to_string(&result)?))
}
```

The `ToolContext` provides chain access (`provider()`, `signer()`, `revm_fork()`) and session state. No global singletons, no side effects at import time.

---

## Goals

| ID  | Goal | Rationale |
| --- | ---- | --------- |
| G1 | Expose 423+ Pi-native tools across 17 categories (~250 read, ~150 write, ~23 privileged) | Complete protocol coverage in one crate |
| G2 | Support all 11 Uniswap-deployed chains | Full chain parity |
| G3 | Support Uniswap V2, V3, V4, UniswapX, and Universal Router | Full protocol version coverage |
| G4 | Defense-in-depth safety: simulation, allowlists, spending limits, rate limiting, circuit breakers, nonce management, hallucination detection | Prevent fund loss from LLM errors |
| G5 | Three-mode custody (Delegation, Privy, Local) via Alloy `Signer` trait, plus Safe, ZeroDev, and generic signer | Cover all agent wallet patterns from custodial to self-custodial |
| G6 | Sub-2-second response for read operations, sub-5-second for quotes | Usable latency for interactive agents |
| G7 | Publish as `golem-tools` on crates.io | Standard distribution channel |
| G8 | Accessible via Pi extensions (Golems) and A2A protocol (external agents) | Golems get in-process tool access; external agents get standards-based interop via A2A |
| G9 | Comprehensive configuration system via env vars and `golem.toml` | Owner-controlled safety policies |
| G10 | Cross-chain intent support via ERC-7683 | Uniswap-native competitive advantage |
| G11 | x402 payment integration for pay-per-use access | Agent-native monetization without API keys |
| G12 | ERC-8004 agent identity awareness | Verified agents get lower fees, priority execution, reputation-gated pool access |
| G13 | Agent treasury management tools | Self-funding agents need to auto-convert fees, rebalance treasuries, manage operating capital |
| G14 | Historical data: trade history, OHLCV candlesticks, price charts | Agents making trading and LP decisions need historical context |
| G15 | Comprehensive token list with metadata (name, symbol, logo, price, pairs, verified status) | Agents need rich token information for discovery, verification, and presentation |
| G16 | Real-time streaming via Pi's event subscription system | Autonomous agents need live data streams, not only request-response |
| G17 | CCA participation tools: bid submission, clearing price monitoring, multi-bid management, token claiming | CCA is live on mainnet and is the standard for fair token launches [CCA-LAUNCH-2026] |
| G18 | Liquidity Launcher integration for atomic token launch | Token launchers need atomic deployment pipelines |
| G19 | am-AMM tools for Bunni v2 pool management bidding | am-AMM creates a direct revenue model for agents managing V4 pools [AMAMM-PARADIGM-2024] |
| G20 | Vault integration: compose with vault crate for ERC-4626 vault operations | The vault protocol depends on core tools for pool data, swaps, and LP |
| G21 | Dual delivery: Pi-native tools (Golems) and A2A interface (external agents) | OpenClaw and other agent frameworks integrate via A2A |
| G22 | x402 outbound client for pay-per-use external data acquisition | Agents need richer data than on-chain sources alone. x402 eliminates API key friction. |
| G23 | Evaluation and quality framework with per-tool unit tests, integration tests, regression detection | No existing tool has quality assurance. Evaluation ensures reliability. |
| G24 | Intelligence tools: MEV risk scoring, IL calculation, venue comparison, token discovery with risk scoring | Agents need analytical capabilities beyond raw data reads |
| G25 | Lending protocol tools: Aave V3, Morpho Blue, Fluid, Moonwell, Seamless | Lending support for yield optimization and leveraged strategies |
| G26 | Leveraged looping with health factor monitoring and auto-deleverage | Autonomous agents need recursive leverage strategies with built-in safety |
| G27 | CDP and stablecoin tools: MakerDAO/Sky, Liquity v2, crvUSD, FraxLend | Permissionless synthetic dollar generation is a core strategy primitive |
| G28 | Liquid staking: Lido, Rocket Pool, Coinbase cbETH, StakeWise | LSTs are the primary yield-bearing collateral asset in DeFi |
| G29 | Restaking (EigenLayer, Symbiotic) and liquid restaking tokens | Points programs and additional AVS yield for capital-efficient agents |
| G30 | Yield aggregators: Yearn V3, Beefy, Convex, Aura | Automated strategy vaults capture complex yield without maintaining individual positions |
| G31 | Managed LP: Arrakis V2, Gamma Strategies, Steer Protocol, Charm Finance | Automated V3 range management lowers operational overhead for LP-focused agents |
| G32 | Curve Finance: pools, gauges, crvUSD lending | Curve dominates stablecoin and LST liquidity |
| G33 | Balancer V3 and Aerodrome/Velodrome | Balancer weighted pools and Aerodrome ve(3,3) represent significant Base-native liquidity |
| G34 | Yield tokenization: Pendle PT/YT/LP and Ethena sUSDe | Pendle enables fixed-rate yield capture; Ethena sUSDe is a high-yield synthetic dollar |
| G35 | Perpetuals: GMX v2, Hyperliquid, Synthetix v3 | Perp markets enable hedging LP delta exposure, carry trades, and directional strategies |
| G36 | Options: Panoptic (V4-native), Lyra v2, Premia | Panoptic's V4-native architecture makes it uniquely relevant; options provide precision hedging for LP IL |
| G37 | DEX aggregator routing: 1inch, CoW Protocol, Paraswap, Odos | Non-Uniswap execution paths offer MEV protection (CoW) and better rates for specific pairs |
| G38 | Cross-chain bridges: Stargate, Across, Hop | Multi-chain agents need capital bridging. Bridge comparison prevents overpaying on fees. |
| G39 | Unified cross-protocol position health monitoring and alert system | A single `get_risk_dashboard` call per heartbeat tick replaces five separate health checks |

---

## Non-goals

| ID | Non-goal | Reason |
| ---- | ---- | ---- |
| NG1 | Building a hosted/managed service | Phase 1 is self-hosted. Hosted offering is a future consideration. |
| NG2 | Non-Uniswap protocols as primary focus | Primary focus is Uniswap (V2/V3/V4/UniswapX). Other protocols supported where relevant to agent capital management, not as a general-purpose DeFi aggregator. |
| NG3 | Building a frontend or UI | This is a headless tool library for programmatic access. |
| NG4 | Implementing a custom routing algorithm | Uses Uniswap's Trading API and smart-order-router for routing. |
| NG5 | Supporting Solana or non-EVM chains | Uniswap is EVM-only. |
| NG6 | Providing financial advice or automated trading strategies | The crate executes what agents request. Strategy is the agent's responsibility. |
| NG7 | KYC/AML compliance enforcement | Out of scope. Agents and owners are responsible for compliance. |
| NG8 | Deep analytics beyond subgraph/RPC capability | Use dedicated analytics services for Dune/Flipside queries. The crate provides subgraph-powered historical data (OHLCV, trade history, volume) per G14. |
| NG9 | Smart contract deployment (V4 hooks, custom contracts) | Covered by companion skills/agents, not the tool library. |
| NG10 | Fiat on/off-ramp | Use a dedicated fiat on/off-ramp service. |

---

## User personas

### Persona 1: The autonomous trading agent

**Who**: An AI agent (built on ElizaOS, GOAT, OpenClaw, LangChain, or custom) that executes trades on Uniswap without human intervention.

**Needs**:

- Get real-time quotes across all chains
- Execute swaps safely with pre-flight simulation
- Check balances and manage approvals
- Operate within configured spending limits
- Recover gracefully from failed transactions

**Example workflow**: "Swap 500 USDC for WETH on Base with max 0.5% slippage" results in quote, simulate, approve (if needed), execute, confirm -- all autonomously.

### Persona 2: The LP management agent

**Who**: An autonomous agent that manages concentrated liquidity positions, rebalancing when prices move out of range.

**Needs**:

- Query pool state (current tick, liquidity distribution, fee APY)
- Add and remove liquidity across V3 and V4 pools
- Collect accrued fees
- Monitor position health (in-range status, IL)
- Discover new high-yield pools

**Example workflow**: Agent detects position is 80% out of range, collects fees, removes liquidity, and opens a new position centered on the current price.

### Persona 3: The agent developer

**Who**: A developer building an AI agent that interacts with Uniswap. Uses Claude Code, Cursor, or similar AI-assisted development tools.

**Needs**:

- Well-documented Pi-native tools with clear parameter types and return schemas
- Predictable error handling (structured error codes, not opaque failures)
- Easy local setup for development and testing
- Configuration options for safety policies during development vs. production

**Example workflow**: Developer adds `golem-tools` as a dependency in their Golem config, sets testnet chain + relaxed spending limits for development, then tightens policies for production.

### Persona 4: The A2A-connected agent

**Who**: An external agent (not a Golem) connected via A2A protocol, answering user questions about Uniswap or executing operations on their behalf.

**Needs**:

- Data tools that return LLM-friendly responses (formatted numbers, context, not raw hex)
- Tool descriptions clear enough for the LLM to select the right tool
- Safety tools the LLM can invoke proactively (validate token before trading it)
- Error messages the LLM can interpret and explain to the user

**Example workflow**: User asks "What's the best pool for WETH/USDC on Ethereum?" -- LLM calls `uniswap_get_pools_by_token_pair`, interprets TVL/volume/fee data, and recommends a pool.

### Persona 5: The self-funding agent (Golem)

**Who**: An autonomous Golem that earns revenue from token trading fees, LP fees, or providing services to other agents via x402 -- and uses Uniswap to manage its own treasury. Golems have finite lifespans and must be economically viable to survive.

**Needs**:

- Auto-convert earned fees (in various tokens) to stablecoins or operating tokens
- DCA strategies for converting volatile earnings to stable operating capital
- LP into pools to generate additional yield on idle treasury funds
- Track portfolio balance across chains and token types
- Operate with strict spending limits to prevent treasury drain
- Pay for tool access via x402 micropayments (no API key provisioning)
- Revenue tracking to measure distance from terminal budget threshold

**Example workflow**: Golem earns 0.5 ETH in token trading fees, auto-swaps 80% to USDC via Uniswap for compute costs, LPs the remaining 20% in ETH/USDC pool for yield. Revenue feeds into the heartbeat (the Golem's periodic 9-step decision cycle, running every 30-120 seconds) survival pressure calculation.

### Persona 6: The agent token deployer

**Who**: An agent platform that deploys tokens for other agents and needs automated pool creation and liquidity management on Uniswap V4.

**Needs**:

- Create Uniswap V4 pools with configurable hooks (anti-snipe, dynamic fees, revenue share)
- Bootstrap initial liquidity for newly created tokens
- Lock LP tokens for configurable durations
- Query pool performance metrics for deployed tokens
- Support vaulting and vesting patterns for agent tokens

**Example workflow**: Agent platform deploys new token, creates Uniswap V4 pool with anti-snipe hook (2-block MEV protection), provides initial liquidity, locks LP for 10 years, reports pool address and trading metrics.

### Persona 7: The protocol fee searcher

**Who**: An autonomous agent that monitors the TokenJar for accumulated protocol fees and executes profitable burn-and-claim transactions via the Firepit contract [UNIFIGATION-2025]. Burns 4,000 UNI to claim assets worth significantly more.

**Needs**:

- Real-time monitoring of TokenJar balances across all fee sources (V2, V3, V4, UniswapX, Unichain native)
- Profitability analysis: compare TokenJar value vs. UNI burn cost
- Optimal asset selection (top 20 assets by value from the jar)
- Full execution pipeline: approve UNI, select assets, call Firepit.release()
- Historical burn event analysis to time entries optimally
- Fee accumulation rate tracking to predict next profitable burn window

**Example workflow**: Agent monitors TokenJar, detects $2.4M accumulated with 4,000 UNI threshold ($40K cost), calculates $2.36M profit, approves UNI, selects top 20 assets, executes burn, receives assets, optionally swaps to stables.

### Persona 8: The cross-chain agent

**Who**: An agent that operates across multiple chains, moving assets and finding optimal execution venues.

**Needs**:

- Query data and execute on any of the 11 supported chains
- Submit ERC-7683 cross-chain intents
- Compare prices and liquidity across chains
- Track cross-chain intent fulfillment status

**Example workflow**: Agent finds WETH is cheaper on Arbitrum than Ethereum, submits an ERC-7683 intent to buy on Arbitrum and deliver to Ethereum.

### Persona 9: The leveraged yield farmer

**Who**: An autonomous Golem that builds and monitors recursive leverage loops to amplify staking or lending yield.

**Needs**:

- Build leverage loops (supply WETH -> borrow USDC -> swap -> supply again) targeting configurable leverage ratios
- Monitor health factor on every heartbeat tick
- Auto-deleverage when health factor drops below threshold
- Unwind positions during Conservation phase (one of five BehavioralPhases -- Thriving, Stable, Conservation, Desperate, Terminal -- that govern a Golem's risk appetite based on its remaining budget)

**Example workflow**: Golem supplies 10 ETH to Aave, builds a 2.5x leveraged stETH loop, monitors health factor every tick. When ETH price drops 15%, auto-deleverage triggers, reducing leverage from 2.5x to 1.5x.

### Persona 10: The restaking optimizer

**Who**: An agent that maximizes points accumulation and yield across EigenLayer AVSs and LRT protocols.

**Needs**:

- Track all restaking points (EigenLayer, Kelp Miles, ether.fi points, Puffer points) in one view
- Compare protocols by points-per-ETH-per-day efficiency
- Monitor withdrawal queues and operator health
- Detect LRT depeg events before they affect position value

**Example workflow**: Agent holds 50 ETH, analyzes `intel_get_points_strategies`, identifies ether.fi + EigenLayer delegation to top AVS operator as optimal. Deposits, delegates, monitors weekly for slashing events.

### Persona 11: The derivatives hedger

**Who**: An LP-focused agent that uses perpetuals and Panoptic options to hedge IL or vault exposure.

**Needs**:

- Open perp short on GMX or Hyperliquid to hedge LP delta exposure
- Buy Panoptic put options below LP range for downside protection
- Monitor delta across combined spot + LP + perp + options positions
- Close hedge when LP position is closed or market conditions change

**Example workflow**: Agent opens WETH/USDC V3 LP position at +-5% range. Simultaneously opens 0.5 ETH GMX short for delta-neutral. Panoptic put at -10% provides tail-risk protection.

### Persona 12: The fixed-rate hunter

**Who**: An agent that buys Pendle PTs to capture predictable yield and ladders maturities for continuous income.

**Needs**:

- Discover PT markets with attractive implied APY vs current variable rate
- Execute PT purchases at target discount levels
- Track PT positions to maturity
- Reinvest redeemed principal into new PT markets (ladder strategy)

**Example workflow**: Agent identifies PT-sUSDe-Mar2026 trading at 8.5% implied APY vs 6.2% sUSDe variable rate. Buys 50K USDC worth of PT. At maturity, reinvests into next available PT market with best rate.

---

### Uniswap API tools (batch 40)

24 tools backed by the Uniswap Trading API (`trade-api.gateway.uniswap.org/v1`). Requires `GOLEM_UNISWAP_API_KEY`. See [14-tools-uniswap-api.md](./14-tools-uniswap-api.md) (tools backed by the Uniswap Trading API for limit orders, batch execution, EIP-7702 delegation, and cross-chain plans) for full specs.

| Category | Tools | New capabilities |
| --------------- | ----- | ---------------------------------------------------- |
| Swap (extended) | 9 | Limit orders, batch exec, EIP-7702, order monitoring |
| LP (API-first) | 8 | Migration, rewards, pool info, approval check |
| Reference | 1 | Bridge discovery across all chains |
| Wallet utility | 3 | Token transfer, delegation check, EIP-7702 encoding |
| Plans | 3 | Multi-step cross-chain execution with proof tracking |

Tool count with batch 40: **423+ tools** across 17 functional categories (~250 read, ~150 write, ~23 privileged).

---

## DeFi Domain: PredictionDomain Reference Implementation

The DeFi domain is the first `PredictionDomain` implementation. It teaches the Oracle how to predict, observe, and evaluate DeFi activity on EVM chains, primarily Base L2. Each protocol is a separate `PredictionDomain` implementation. Adding a new protocol means implementing the trait -- no changes to the Oracle engine.

### Supported Protocols (v1)

| Protocol | Activity | Prediction Categories |
|----------|---------|----------------------|
| **Uniswap V3/V4** | Swap, LP | Slippage, fee accumulation, price impact, IL |
| **Aave V3** | Lend, Borrow | Health factor, supply rate, borrow rate, utilization |
| **Morpho Blue** | Lend | Supply APY, utilization, available liquidity |
| **Aerodrome** | LP | Fee accumulation, emissions APR, IL |
| **Compound** | Lend, Borrow | Supply rate, borrow rate |

### EnvironmentClient: EVM Implementation

```rust
pub struct EvmChainClient {
    provider: Arc<RootProvider<Http<Client>>>,
    chain_id: u64,
    indexer: Option<IndexerClient>,  // The Graph, Envio, or equivalent
}

#[async_trait]
impl EnvironmentClient for EvmChainClient {
    async fn read(&self, query: &EnvironmentQuery) -> Result<EnvironmentValue> {
        // Translates EnvironmentQuery into eth_call via Alloy.
        let call = query.to_alloy_call()?;
        let result = self.provider.call(&call).await?;
        Ok(EnvironmentValue::from_abi_decode(result, &query.return_type))
    }

    async fn read_batch(&self, queries: &[EnvironmentQuery]) -> Result<Vec<EnvironmentValue>> {
        // Multicall3 batching for efficiency.
        let multicall = Multicall3::new(&self.provider);
        for q in queries { multicall.add_call(q.to_alloy_call()?); }
        multicall.call().await
    }

    fn env_timestamp(&self) -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() }
}
```

### Resolution: How Ground Truth Is Read

Every prediction resolves via `eth_call` -- a read-only RPC call that executes a contract function without submitting a transaction:

- **Fee accumulation**: Read `feeGrowthGlobal0X128` and `feeGrowthGlobal1X128` from the Uniswap pool contract.
- **Health factor**: Read `getUserAccountData` from Aave's Pool contract.
- **Pool price**: Read `slot0` from the Uniswap pool.
- **Utilization**: Read `totalSupply` and `totalBorrow` from the lending market.

**Known limitation -- on-chain manipulation**: MEV actors can distort state at the moment of resolution (sandwich attacks, JIT liquidity). Mitigation: predictions use TWAP reads where available, and resolution reads are compared against multiple block heights (current and 3 blocks prior) to detect manipulation. If significant divergence is detected, the resolution is deferred.

**Known limitation -- block reorgs**: On L2s like Base, reorgs are rare (<0.01% of blocks) but possible. Mitigation: wait for finalization (L1 attestation) before resolving high-value predictions. Low-value background predictions resolve immediately, accepting the small reorg risk.

### Prediction Categories (Example: Uniswap V3 LP)

| Category | Claim Type | Resolution | Timescale |
|----------|-----------|------------|-----------|
| `uniswap_v3_fee_rate` | InRange ($/hr) | feeGrowthGlobal delta | Hours |
| `uniswap_v3_price_range` | InRange (tick) | slot0.tick | Hours |
| `uniswap_v3_il` | InRange (%) | Position value delta | Days |
| `uniswap_v3_swap_slippage` | WithinBps | Actual vs quoted price | Seconds |
| `uniswap_v3_position_pnl` | Direction | Position NAV delta | Days |

Each category has its own residual buffer, its own accuracy score, and its own correction parameters.

### Attention Seed: From Strategy to Universe

The owner writes: "I want to earn yield on ETH, moderate risk." The LLM compiles this into an AttentionSeed:

```rust
AttentionSeed {
    tokens: ["ETH", "WETH", "stETH", "rETH", "wstETH", "cbETH"],
    activities: [LP, Lending, Staking, YieldAggregation],
    risk_profile: Moderate,
    excluded: [Derivatives, Bridges, Leverage],
    chains: [Base],
}
```

Each PredictionDomain receives this seed and discovers relevant items:
- UniswapV3LpDomain discovers ETH/USDC, ETH/DAI, WETH/stETH pools across fee tiers
- AaveV3LendingDomain discovers ETH, stETH, rETH supply markets
- MorphoLendingDomain discovers ETH vaults
- AerodromeLpDomain discovers ETH pair gauges

Total: 50-200 items across all domains, entering at SCANNED tier.

### Owner Visibility

The Protocol screen in the TUI lets the owner browse supported protocols, see pool data, and (for ACTIVE items) view the full prediction cascade: what the Golem predicted, what actually happened, and the residual.

The owner can also use the protocol browser to manually promote an item to WATCHED or ACTIVE, overriding the attention forager's automatic allocation. This is a course-correction surface: "I want you to pay attention to this pool specifically."

### PolicyCage and Capability Token Integration

All write tools in the DeFi domain require a `Capability<WriteTool>` token, consumed on use via Rust move semantics. The PolicyCage validates every proposed action against the owner's configured constraints before a capability token is issued. Protocol-specific tools inherit the PolicyCage integration from the base tool architecture -- see [01-architecture.md](01-architecture.md) (tool architecture spec: ToolDef pattern, trust tiers, capability token lifecycle, safety hooks, and profiles) for the capability token lifecycle.
