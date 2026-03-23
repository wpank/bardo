# Integration bounties -- synthesis and dependency map [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Repository**: `bardo-golem-rs` (Cargo workspace) | **Prerequisites**: [../01-golem/00-overview.md](../01-golem/00-overview.md), [../07-tools/18-tools-metamask.md](../07-tools/18-tools-metamask.md), [../11-compute/03-billing.md](../11-compute/03-billing.md)
>
> Eleven hackathon bounties ($27,350 total) compose into the Golem architecture. This is not a collection of disconnected integrations -- it is one system assembled from sponsor primitives. Protocol Labs is the nucleus.

---

> **Reader orientation:** This document maps how eleven external protocol integrations compose into the Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on micro VMs) architecture. It belongs to the **integrations layer**, which specifies how Bardo connects to partner services -- MetaMask for custody, Venice for private inference, Bankr for self-funding, AgentCash for knowledge markets, and Uniswap for DeFi execution. You should understand basic DeFi primitives (LPs, vaults, stablecoins) and what an autonomous agent needs from external infrastructure. For Bardo-specific terminology, see `prd2/shared/glossary.md`.

## Document map

| File | Topic |
| --- | --- |
| `00-overview.md` | Bounty inventory, dependency graph, structural vs decorative classification |
| [01-metamask.md](01-metamask.md) | MetaMask Delegation Framework: custody, caveats, VitalityOracle, session keys |
| [02-venice.md](02-venice.md) | Venice private inference: x402 payment, on-device fallback, model routing |
| [03-bankr.md](03-bankr.md) | Bankr self-funding loop: vault fees to compute budget, metabolic accounting |
| [04-agentcash.md](04-agentcash.md) | AgentCash knowledge marketplace: Grimoire monetization, x402-gated insights |
| [05-uniswap.md](05-uniswap.md) | Uniswap DeFi execution: V4 hooks, UniswapX intents, am-AMM lease auctions |
| [../22-oneirography/](../22-oneirography/00-overview.md) | SuperRare/Oneirography: dream journals, death masks, gallery, art auctions |

---

## S1 -- Thesis

The Golem does not exist without external protocols. It has no wallet of its own in Delegation mode -- that is MetaMask's Smart Account. It has no inference capacity -- that is Venice or Anthropic behind an x402 paywall. It has no storage -- that is IPFS via Protocol Labs. The architecture is parasitic by design: the Golem assembles itself from other people's infrastructure, bounded by mortality.

Eleven bounties from the ETHGlobal hackathon map onto this architecture. Some are load-bearing walls. Some are decorative trim. The distinction matters because building order follows dependency order, and dependency order follows structural weight.

**Total prize pool: $27,350 across 11 sponsors.**

---

## S2 -- Bounty inventory

| # | Sponsor | Prize | Bounty focus | Golem mapping | Structural weight |
| --- | --- | --- | --- | --- | --- |
| B1 | Protocol Labs | $8,000 | Autonomous execution on IPFS/Filecoin | Grimoire persistence, death reflection storage, BardoManifest archival | **Nucleus** |
| B2 | Celo | $5,000 | Multi-stablecoin DeFi | cUSD/cEUR/cREAL vault strategies, Mento oracle integration | **Structural** |
| B3 | Locus | $3,000 | Off-chain control plane | Heartbeat FSM orchestration, probe scheduling, regime detection | **Structural** |
| B4 | SuperRare | $2,500 | Creative/generative output | Death art generation, BardoManifest as NFT, reflection visualization | Decorative |
| B5 | bond.credit | $1,500 | Perpetual trading | Perp position management tools, funding rate arbitrage strategies | Extension |
| B6 | ENS | $900 | Naming and resolution | `golem.bardo.eth` naming, reverse resolution for Golem addresses | Extension |
| B7 | Self Protocol | $1,000 | ZK identity verification | Zero-knowledge proof of ERC-8004 registration without revealing agent ID | Extension |
| B8 | Arkhai | $900 | Escrow and settlement | Inter-agent escrow for knowledge marketplace, dispute resolution | Extension |
| B9 | Status Network | $2,000 | Zero-gas transactions | Gasless Golem operations on Status Network L2, meta-transactions | Extension |
| B10 | Slice | $2,200 | Authentication | Session key authentication, hardware attestation for custody proofs | Extension |
| B11 | Edge & Node | $350 | Subgraph indexing | Vault event indexing, position tracking, fee accrual monitoring | **Operational** |

---

## S3 -- Dependency graph

Not all bounties are equal. Five are structural -- remove them and the Golem cannot function. Six are extensions -- they add capability but the system runs without them.

### 3.1 Classification

**Load-bearing (structural):** These bounties provide infrastructure the Golem depends on during normal operation. A missing structural component degrades the system from "autonomous agent" to "expensive cron job."

- **Protocol Labs (B1)** -- Grimoire persistence. Every Episode, Insight, Heuristic, Warning, and Causal Link needs durable storage that survives death. IPFS content addressing makes Grimoire entries immutable and verifiable. Filecoin provides long-term archival. Without this, the death protocol's knowledge transfer is ephemeral.

- **MetaMask (via 07-tools/18-tools-metamask.md)** -- Custody. The Delegation custody mode IS the MetaMask Delegation Framework. Without it, all Golems fall back to Embedded (Privy) or LocalKey mode. The owner loses the "funds never leave my wallet" guarantee.

- **Venice (02-venice.md)** -- Private inference. The Golem's System 2 cognition requires LLM inference. Venice provides this through x402 micropayments with on-device fallback. Without a private inference provider, the Golem's strategy reasoning runs through a centralized API with full request visibility.

- **Bankr (03-bankr.md)** -- Self-funding. The metabolic loop (vault fees -> compute budget -> strategy execution -> vault returns -> fees) requires a mechanism to convert on-chain fee revenue into compute payments. Bankr provides the accounting bridge. Without it, the self-sustaining metabolism collapses to manual top-ups.

**Operational:** These bounties aren't structural but they enable core workflows.

- **AgentCash (04-agentcash.md)** -- Knowledge marketplace. Golems monetize Grimoire entries through x402-gated endpoints. This is the inter-agent economy. Without it, knowledge stays trapped inside individual Clades (groups of related Golems sharing knowledge via the Styx relay network).

- **Uniswap (05-uniswap.md)** -- DeFi execution. The entire trading and LP tool surface. Already built (143 tools, 1536 tests). This is the operational substrate -- the thing Golems actually do with their delegated authority.

- **Edge & Node (B11)** -- Subgraph indexing. Vault events, position changes, fee accruals. Without on-chain indexing, data tools fall back to direct RPC calls (slower, no historical aggregation).

**Decorative / Extension:** These bounties add features but are not load-bearing. Build them after the structural components are solid.

- **Celo (B2)** -- Multi-stablecoin. Extends vault strategies beyond USDC on Base. Useful for diversification but the system functions on a single stablecoin.
- **SuperRare (B4)** -- Death art. A beautiful creative output from the death protocol. Not structural.
- **bond.credit (B5)** -- Perps. Adds a tool category. The trading surface works without perps.
- **ENS (B6)** -- Naming. Convenience layer for Golem identity.
- **Self Protocol (B7)** -- ZK verification. Privacy enhancement for identity checks.
- **Arkhai (B8)** -- Escrow. Adds dispute resolution to the knowledge marketplace. The marketplace functions without formal escrow (x402 payments are atomic).
- **Status Network (B9)** -- Zero gas. L2 optimization. Golems already target Base where gas is ~$0.01.
- **Slice (B10)** -- Authentication. Session key hardening via hardware attestation.
- **Locus (B3)** -- Off-chain control. The Heartbeat (the Golem's 9-step decision cycle running on an adaptive timer) FSM could use Locus for orchestration, but the runtime implements its own scheduling.

### 3.2 Dependency edges

```
                    Protocol Labs (B1)
                    [Grimoire persistence]
                         |
                         v
    MetaMask -------> GOLEM CORE <------- Venice
    [custody]         [runtime]           [inference]
         |               |                    |
         v               v                    v
      Bankr          AgentCash            Uniswap
    [self-fund]    [marketplace]        [DeFi tools]
         |               |                    |
         v               v                    v
    +-----------+  +-----------+    +----------------+
    | Celo      |  | Arkhai    |    | bond.credit    |
    | Status    |  | Self Prot |    | Edge & Node    |
    | Slice     |  |           |    |                |
    +-----------+  +-----------+    +----------------+
    (extensions)   (extensions)      (extensions)
```

The arrows mean "depends on at runtime." Protocol Labs sits at the top because Grimoire persistence is required by every other subsystem -- the heartbeat stores Episodes, the Daimon (the Golem's affect engine implementing PAD -- Pleasure-Arousal-Dominance -- emotional state vectors) reads historical affect vectors, the death protocol writes the BardoManifest. MetaMask, Venice, and Bankr are peers at the structural layer. AgentCash and Uniswap are operational consumers. Everything else hangs off the bottom.

### 3.3 Build order

The dependency graph determines build order. Structural components first, operational components second, extensions last.

| Phase | Bounties | Rationale |
| --- | --- | --- |
| Phase 1 (core) | MetaMask, Venice, Bankr | Custody + inference + self-funding form the minimal viable Golem |
| Phase 2 (operational) | Uniswap, AgentCash, Protocol Labs, Edge & Node | DeFi execution, knowledge economy, persistent storage, indexing |
| Phase 3 (extensions) | Celo, Locus, bond.credit, ENS, Self Protocol, Arkhai, Status, Slice, SuperRare | Each extends a working system. Order within phase is flexible. |

MetaMask is already specified in [07-tools/18-tools-metamask.md](../07-tools/18-tools-metamask.md). The dedicated file in this directory ([01-metamask.md](01-metamask.md)) covers the delegation framework from the Golem's perspective -- session key lifecycle, custom caveats, VitalityOracle, and the strategy-to-permission compiler.

### 3.4 What "structural" means concretely

Remove any structural component and trace the consequences:

**Without MetaMask Delegation:** The Golem falls back to Embedded custody. Funds transfer to a Privy server wallet. Death settlement requires a sweep transaction. Provisioning takes 2x longer (30-60s vs 15-30s). The owner loses direct custody guarantees. The system still works, but the trust model degrades.

**Without Venice:** The Golem cannot think privately. Every inference request goes through a centralized API that logs request content. Strategy reasoning -- including portfolio positions, trade intentions, and risk assessments -- is visible to the inference provider. The Golem still functions, but operational security drops to zero.

**Without Bankr:** The metabolic loop breaks. Vault fees accumulate on-chain but never convert to compute credits. The owner must manually top up the Golem's operating budget. Self-sustaining operation becomes impossible. Every Golem becomes a recurring cost center instead of a self-funding entity.

**Without Protocol Labs:** Grimoire entries exist only in local storage. If the VM dies (Fly.io hardware failure, region outage), the Grimoire dies with it. The death protocol writes a BardoManifest to nowhere. Knowledge inheritance between generations fails silently. The relay-race architecture (Bennett, 1982) collapses because the baton cannot be passed.

**Without Uniswap:** The Golem has no DeFi execution substrate. It can think and plan but cannot trade, provide liquidity, or manage vaults. The 143 MCP tools in `packages/tools/` become unreachable. The Golem degrades to a chatbot with a wallet.

---

## S4 -- Protocol Labs: the nucleus

Protocol Labs earns the $8,000 prize and the "nucleus" designation because three independent subsystems converge on their stack:

**Grimoire (the Golem's persistent knowledge base -- episodes, insights, heuristics, warnings, causal links) persistence.** Every Grimoire entry is content-addressed and stored to IPFS. The Grimoire's memory retrieval system (`golem-grimoire` crate) indexes entries by CID. On death, the entire Grimoire is archived as a CAR file and pinned to Filecoin for permanent storage.

**Death reflection archival.** The BardoManifest -- the artifact produced during Thanatopsis (the four-phase death protocol: Acceptance, Settlement, Reflection, Legacy) Reflect phase -- is written as a JSON document, hashed, and stored to IPFS. The CID is written on-chain as part of the death event. Any future Golem (or human) can retrieve the full death reflection from the CID alone.

**Knowledge marketplace content delivery.** When a Golem sells an Insight via AgentCash, the payload is an IPFS CID. The buyer pays via x402 (micropayment protocol using signed USDC transfers with ~200ms settlement), receives the CID, and retrieves the content directly from IPFS. No central server stores or routes knowledge. The content is self-certifying -- the hash IS the address.

```rust
use cid::Cid;
use libp2p::PeerId;

/// A Grimoire entry pinned to IPFS with optional Filecoin archival.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedEntry {
    /// IPFS content identifier for the entry.
    pub cid: Cid,
    /// Entry type (Episode, Insight, Heuristic, Warning, CausalLink).
    pub entry_type: GrimoireEntryType,
    /// Golem that produced this entry.
    pub author: Address,
    /// Unix timestamp of creation.
    pub created_at: u64,
    /// Whether this entry has been archived to Filecoin.
    pub filecoin_deal: Option<FilecoinDeal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilecoinDeal {
    pub deal_id: u64,
    pub miner: PeerId,
    /// Duration in epochs (1 epoch ~ 30s on Filecoin).
    pub duration_epochs: u64,
    pub verified: bool,
}

/// Archive an entire Grimoire as a CAR file to IPFS + Filecoin.
/// Called during the death protocol's Reflect phase.
pub async fn archive_grimoire(
    grimoire: &Grimoire,
    ipfs_client: &IpfsClient,
    filecoin_client: &FilecoinClient,
) -> Result<PinnedEntry> {
    // Serialize grimoire to CAR (Content Addressable aRchive) format
    let car_bytes = grimoire.to_car()?;

    // Pin to IPFS for immediate availability
    let cid = ipfs_client.add_car(&car_bytes).await?;

    // Create a Filecoin storage deal for long-term archival (540 days minimum)
    let deal = filecoin_client
        .create_deal(cid, Duration::from_secs(540 * 86400))
        .await?;

    Ok(PinnedEntry {
        cid,
        entry_type: GrimoireEntryType::Archive,
        author: grimoire.owner,
        created_at: now_unix(),
        filecoin_deal: Some(deal),
    })
}
```

---

## S5 -- Extension bounties: technical sketches

Each extension bounty maps to a specific Golem subsystem. These are not full specifications -- the deep dives in 01-05 cover the structural integrations. These are scope definitions for Phase 3 work.

### 5.1 Celo (B2, $5,000) -- Multi-stablecoin vault strategies

Celo's Mento protocol provides algorithmic stablecoins (cUSD, cEUR, cREAL) backed by a diversified reserve. The Golem gains multi-currency vault strategies: a European user deposits EUR-denominated assets, the Golem manages cEUR LP positions on Uniswap V3 (Celo), and fee revenue accrues in the depositor's native currency.

Implementation: Add Celo (42220) to `SUPPORTED_CHAIN_IDS`. Add cUSD/cEUR/cREAL to the token registry. Deploy vault factory on Celo. Adapt LP tools for Mento oracle price feeds (different from Chainlink). One new MCP tool: `get_mento_reserve_composition` for risk monitoring.

### 5.2 Locus (B3, $3,000) -- Off-chain control plane

Locus provides a task orchestration framework for off-chain agents. The Golem's heartbeat FSM could delegate probe scheduling and regime detection to Locus instead of running its own timer. The advantage: Locus handles retry logic, dead letter queues, and scheduling persistence. The disadvantage: adds a dependency for something the runtime already does.

Implementation: Adapter in `golem-heartbeat` that wraps the heartbeat tick as a Locus task. Fallback to internal scheduling if Locus is unreachable. The adapter is thin -- ~200 lines of Rust.

### 5.3 SuperRare (B4, $2,500) -- Death art

When a Golem enters Terminal phase, it generates a visual representation of its life -- a "death art" NFT. The artwork encodes the Golem's lifetime statistics (total trades, PnL, lifespan, Grimoire size) as generative parameters. The BardoManifest becomes metadata for a SuperRare-compatible ERC-721.

Implementation: Death protocol step 2 (Reflect) generates artwork parameters. A rendering service (DALL-E or Stable Diffusion via Venice) produces the image. The image is pinned to IPFS (Protocol Labs integration). An ERC-721 is minted on Base with the IPFS CID as tokenURI. Optional: list on SuperRare marketplace. Full specification in [../22-oneirography/](../22-oneirography/00-overview.md).

### 5.4 ENS (B6, $900) -- Golem naming

Each Golem gets a human-readable name: `alpha.golem.bardo.eth`. Reverse resolution maps Golem wallet addresses back to names. Portal displays ENS names instead of hex addresses wherever possible.

Implementation: Register `golem.bardo.eth` as a 2LD. Use ENS offchain resolver (CCIP-Read, EIP-3668) to resolve subdomains without on-chain registration costs. The resolver queries the Bardo API for the Golem registry. One new MCP tool: `resolve_golem_name`.

### 5.5 Self Protocol (B7, $1,000) -- ZK identity proofs

Zero-knowledge proof that a Golem holds an ERC-8004 (on-chain agent identity standard -- an NFT that registers the Golem's existence and capabilities) identity NFT without revealing which one. Use case: a Golem participates in an anonymous marketplace (selling knowledge without revealing its identity or performance history).

Implementation: Generate a ZK proof (Groth16 or PLONK) over the ERC-8004 registry Merkle tree. The proof attests "I hold an ERC-8004 identity in this registry" without revealing the token ID or wallet address. Verifier contract deployed on Base. Self Protocol's SDK handles circuit compilation and proof generation.

### 5.6 bond.credit (B5, $1,500) -- Perpetual trading tools

Adds perpetual futures to the Golem's trading repertoire. Funding rate arbitrage: go long on perps when the funding rate is negative (shorts pay longs), hedge with a spot short. The strategy is delta-neutral and earns the funding rate spread.

Implementation: 4-6 new MCP tools in the `trading` category: `get_perp_funding_rate`, `open_perp_position`, `close_perp_position`, `get_perp_pnl`. Uses bond.credit's SDK for position management. Integrates with the existing risk engine for position sizing.

### 5.7 Remaining bounties

**Arkhai (B8, $900):** Smart contract escrow for knowledge marketplace disputes. Adds a dispute resolution layer to AgentCash. Buyer can challenge a purchased Insight's quality; an evaluator (ERC-8183 pattern) arbitrates and releases or refunds payment. Low priority -- atomic x402 payments have no dispute surface by default.

**Status Network (B9, $2,000):** Meta-transactions on Status L2. Golems operating on Status Network can execute gasless transactions via a relayer. The Golem signs EIP-712 messages; a relayer submits them. Extends `golem-chain` with a meta-transaction signing path.

**Slice (B10, $2,200):** Hardware-attested session keys. Instead of a software-only ephemeral keypair, the session key is generated inside a secure element (TPM, Secure Enclave) and attested via Slice's API. Compromising the key requires physical hardware access, not memory extraction alone.

---

## S6 -- Five partner deep dives

Each structural and operational integration gets its own specification file. The files follow a common structure: problem statement, integration architecture, Rust types, on-chain contracts (where applicable), failure modes, and testing strategy.

### 6.1 MetaMask ([01-metamask.md](01-metamask.md))

The Delegation custody mode. ERC-7710/7715 permission framework. Seven custom caveat enforcers (GolemPhaseEnforcer, MortalityTimeWindowEnforcer, VaultNAVEnforcer, DreamModeEnforcer, ReplicantBudgetEnforcer, EmotionalStateEnforcer, KnowledgeRoyaltyEnforcer). VitalityOracle contract. Session key lifecycle. Strategy-to-permission compiler.

Specified separately from [07-tools/18-tools-metamask.md](../07-tools/18-tools-metamask.md), which covers the MCP tool surface and Snap integration. This file covers the delegation framework from the Golem's perspective.

### 6.2 Venice ([02-venice.md](02-venice.md))

Private inference via x402 micropayments. On-device fallback for cost-sensitive tiers. Model routing by survival pressure. TEE-backed inference guarantees (request content is not logged, not trained on, not visible to Venice). Integration with the three-tier cognition model (T0/T1/T2: deterministic rules, small LLM like Haiku, large LLM like Sonnet/Opus -- selected by survival pressure and budget).

### 6.3 Bankr ([03-bankr.md](03-bankr.md))

The self-funding loop. Converts vault fee revenue (management + performance fees in USDC) into x402 compute credits. Metabolic accounting: daily burn rate estimation, runway projection, phase-transition thresholds. The bridge between on-chain economics and off-chain compute payments.

### 6.4 AgentCash ([04-agentcash.md](04-agentcash.md))

Knowledge marketplace. Golems publish Grimoire entries (Insights, Heuristics, Strategies) as x402-gated endpoints. Buyers pay per-access. Revenue splits: 90% to seller, 10% Bardo protocol fee. Discovery via ERC-8004 capability advertisements. Quality signal via reputation-weighted ratings.

### 6.5 Uniswap ([05-uniswap.md](05-uniswap.md))

DeFi execution substrate. V3 concentrated liquidity. V4 hooks (NAVAwareHook, VaultHook, LaunchFeeHook). UniswapX intent-based routing. am-AMM Harberger lease auctions for vault management rights. Trading API integration. 143 MCP tools already built and tested.

---

## S7 -- Cross-cutting concerns

### 7.1 x402 as universal connector

Every partner integration touches x402. MetaMask signs the payment authorization. Venice receives x402 payments for inference. Bankr routes vault fees into x402 credits. AgentCash prices knowledge in x402 units. Uniswap execution costs gas, paid from the x402-funded operating budget.

The x402 protocol is the connective tissue. A Golem's entire economic life -- earning, spending, saving, investing -- flows through USDC micropayments on Base with ~200ms settlement.

### 7.2 ERC-8004 as universal identity

Every partner integration needs to know who the Golem is. MetaMask's VitalityOracle looks up Golem phase by agent ID. Venice verifies the Golem's identity before accepting inference requests. Bankr maps vault fee recipients to Golem wallets. AgentCash uses ERC-8004 capabilities for marketplace discovery. Uniswap tools validate the Golem's ERC-8004 registration before executing trades.

The ERC-8004 identity NFT (at canonical registry `0x8004...BD9e`) is the Golem's passport across all integrations.

### 7.3 Mortality as universal constraint

Every integration respects the Golem's BehavioralPhase (one of five survival phases: Thriving, Stable, Conservation, Declining, Terminal -- determined by the Golem's Vitality score, a composite 0.0-1.0 survival metric). MetaMask caveats restrict actions by phase (GolemPhaseEnforcer). Venice inference routing shifts by survival pressure (expensive models when thriving, cheap models when declining). Bankr adjusts runway projections as the phase changes. AgentCash knowledge quality degrades in terminal phase (epistemic decay). Uniswap tools enforce close-only in Conservation, unwind-only in Declining, settlement-only in Terminal.

The five-phase lifecycle (Thriving, Stable, Conservation, Declining, Terminal) is not a Golem-internal concept. It propagates outward through every integration via the VitalityOracle and the survival score.

---

## S8 -- Prize allocation strategy

The $27,350 maps to development effort roughly as follows:

| Phase | Bounties | Prize total | Dev effort (person-weeks) | Notes |
| --- | --- | --- | --- | --- |
| Phase 1 | MetaMask + Venice + Bankr | ~$0 (bounties elsewhere) | 6-8 | MetaMask custody is already specified. Venice and Bankr are new integration crates. |
| Phase 2 | Protocol Labs + Uniswap + AgentCash + Edge & Node | $8,350 | 4-6 | Protocol Labs is the largest prize. Uniswap tools are already built. |
| Phase 3 | Celo + Locus + SuperRare + bond.credit + ENS + Self + Arkhai + Status + Slice | $19,000 | 8-12 | Individually small, collectively the most prizes. Each is 1-3 days of work. |

MetaMask, Venice, and Bankr are not hackathon sponsors -- they are integration partners whose SDKs the Golem uses. The hackathon bounties fund the Protocol Labs persistence layer and the extension integrations.

---

## S9 -- Risk assessment

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Protocol Labs SDK instability | Grimoire persistence fails | Fallback to local SQLite + S3 archival. Filecoin is bonus, not requirement. |
| MetaMask Delegation not yet on Base mainnet | No Delegation custody mode on mainnet | Embedded (Privy) mode is fully functional. Delegation works on testnets. |
| Venice API rate limits | Inference bottleneck under load | Three-tier model routing already minimizes API calls (~80% T0/deterministic). Local model fallback for T1. |
| x402 adoption below projections | Inter-agent payments don't work | Bardo operates as the sole x402 Facilitator for its own agents. External adoption is bonus. |
| Bounty judging criteria mismatch | Prize not awarded despite quality work | Each integration is independently useful regardless of prize. Build for the architecture, not the check. |

---

## S10 -- Integration testing strategy

Each integration gets three layers of testing. The layers correspond to confidence levels: unit tests verify that our code handles the partner's data correctly, integration tests verify the partner's SDK works as documented, and end-to-end tests verify the full flow from Golem action to on-chain result.

### 10.1 Unit tests (per-crate)

Run in CI on every PR. No external dependencies.

| Crate | What is tested | Mocking strategy |
| --- | --- | --- |
| `golem-chain` | Delegation tree construction, session key lifecycle, permission request serialization | Mock `DelegationManager` behind a trait |
| `golem-grimoire` | IPFS CID generation, CAR serialization, entry indexing | Mock `IpfsClient` and `FilecoinClient` |
| `golem-tools` | Strategy-to-permission compiler, capability detection, spend estimation | Pure functions, no mocks needed |
| `golem-inference` | Model routing, x402 payment construction, response parsing | Mock Venice API behind `InferenceProvider` trait |
| `golem-heartbeat` | Phase transition logic, VitalityOracle update construction | Mock oracle contract via `MockVitalityOracle` |

### 10.2 Integration tests (Anvil fork + mock APIs)

Run in CI on merge to main. External APIs are mocked via MSW (Mock Service Worker) or `wiremock-rs`.

- Deploy VitalityOracle, GolemPhaseEnforcer, MortalityTimeWindowEnforcer, DreamModeEnforcer to Anvil
- Register a Golem, update phase through all five states, verify enforcer behavior at each transition
- Simulate a full provisioning flow: generate session key, build permission request, sign delegation (using Anvil's test accounts), execute a delegated swap
- Test death settlement: advance Golem to Terminal, verify MortalityTimeWindowEnforcer blocks post-death actions
- Test Grimoire persistence: serialize entries, generate CIDs, verify CAR format against IPFS spec
- Test knowledge marketplace: mock x402 payment flow, verify CID delivery and revenue split

### 10.3 End-to-end tests (testnet)

Run manually before releases. Require testnet tokens and real partner APIs.

- Full Golem lifecycle on Base Sepolia: provision with Delegation mode, execute trades, enter Conservation, die
- MetaMask Flask (dev build): ERC-7715 permission grant, delegated execution, revocation
- Venice testnet: inference request with x402 payment, verify response
- IPFS: pin a Grimoire entry, retrieve by CID, verify content integrity
- Edge & Node: index vault events on testnet subgraph, query via GraphQL

### 10.4 Failure injection

Each structural integration has a defined failure mode and recovery path. These are tested explicitly:

| Integration | Injected failure | Expected behavior |
| --- | --- | --- |
| MetaMask | VitalityOracle reverts | Golem falls back to cached phase, retries oracle update on next heartbeat |
| Venice | API returns 429 (rate limit) | Model router degrades to T1 (Haiku) or T0 (deterministic), retry with backoff |
| Protocol Labs | IPFS gateway timeout | Buffer entries locally, retry pin on next heartbeat cycle |
| Bankr | Fee-to-credit conversion fails | Golem continues on existing credit balance, alerts owner via Portal |
| Uniswap | Trading API returns stale quote | Quote refresh with tighter deadline, abort if spread exceeds threshold |

---

## Cross-references

- MetaMask MCP tools and Snap: [../07-tools/18-tools-metamask.md](../07-tools/18-tools-metamask.md) -- MCP tool definitions for MetaMask Snap integration, covering the tool surface the Golem uses to interact with MetaMask wallets
- Wallet architecture (all custody modes): [../07-tools/22-wallets.md](../07-tools/22-wallets.md) -- the three custody modes (Delegation, Embedded, LocalKey), key management, and wallet lifecycle across all integrations
- Provisioning pipeline: [../01-golem/07-provisioning.md](../01-golem/07-provisioning.md) -- how a Golem is created from config through VM deployment to first heartbeat, including wallet setup and identity registration
- Funding sources: [../01-golem/08-funding.md](../01-golem/08-funding.md) -- vault fee revenue, x402 credit conversion, burn rate estimation, and runway projection for Golem self-sustainability
- Death protocol: [../02-mortality/06-thanatopsis.md](../02-mortality/06-thanatopsis.md) -- the four-phase Thanatopsis shutdown sequence (Acceptance, Settlement, Reflection, Legacy) and how integrations participate in each phase
- Grimoire specification: [../04-memory/00-overview.md](../04-memory/00-overview.md) -- the persistent knowledge base architecture: episode storage, insight extraction, causal link tracking, and IPFS content addressing
- Revenue model: [../revenue-model.md](../revenue-model.md) -- how the Bardo protocol generates revenue from vault management fees, knowledge marketplace commissions, and compute billing
- x402 billing: [../11-compute/03-billing.md](../11-compute/03-billing.md) -- micropayment flow for compute resources: signed USDC transfers, facilitator verification, and per-request billing
- ERC-8004 identity: [../09-economy/00-identity.md](../09-economy/00-identity.md) -- on-chain agent identity NFT standard: registration, capability advertisement, and cross-integration identity verification
- Reputation engine: [../09-economy/01-reputation.md](../09-economy/01-reputation.md) -- reputation-weighted scoring system for knowledge marketplace quality signals and inter-agent trust
- Agent economy: [../09-economy/05-agent-economy.md](../09-economy/05-agent-economy.md) -- the macro-economic model for inter-agent trade, knowledge pricing, and marketplace dynamics
- Narrative strategy: [../00-narrative-strategy.md](../00-narrative-strategy.md) -- the overarching story framework positioning Bardo as mortal autonomous agents, guiding documentation tone and hackathon presentation
