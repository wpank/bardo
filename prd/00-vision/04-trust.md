# Trust-Minimized Architecture [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Reader orientation:** This is the trust model document for Bardo. It defines how every component -- wallet custody, vault contracts, session keys, strategy state, compute, inference, reputation -- exists on a spectrum from fully managed (Bardo handles it) to fully self-sovereign (trust only yourself and the chain). The core insight: the self-sovereign path is what makes the managed path trustworthy. This belongs to the `00-vision/` foundation layer. If you're evaluating Bardo's security model or custody design, start here. `prd2/shared/glossary.md` has full term definitions.

---

## Design Principle

Bardo should be the easiest way to do things, not the only way. Every component -- wallet, vault, agent, compute, strategy, brain state, reputation -- should have a zero-friction managed path (Bardo handles it) and a self-sovereign path (user handles it). Moving between paths should be frictionless, reversible where possible, and never require Bardo's permission.

The managed path is the product. The self-sovereign path is what makes the product trustworthy. Users who know they can leave are more comfortable staying. Users who stay generate revenue. Users who leave still contribute to ecosystem credibility and on-chain activity.

This is de Beauvoir's ethics of ambiguity [DE-BEAUVOIR-1947] applied to product design: _"To will oneself free is also to will others free."_ The export-first architecture is not altruism. It is the recognition that trust creates retention more effectively than lock-in.

---

## The Trust Spectrum

Trust is not binary. Every component in the Bardo stack exists on a spectrum from fully managed (trust Bardo) through hybrid configurations to fully self-sovereign (trust only yourself and the chain). The architecture explicitly supports all three positions and allows movement between them without data loss or capability degradation.

---

## 1. Wallet Custody

Three custody modes, ordered by recommendation:

### Delegation (Recommended)

MetaMask ERC-7710/7715 delegation [ERC-7710]. Funds stay in the Owner's wallet. The Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) receives session keys with on-chain caveat enforcers -- spending limits, allowed targets, expiry. The Owner's MetaMask Smart Account enforces constraints at the contract layer. No Privy dependency, no custodial risk, clean death semantics (session keys simply expire when the Golem dies).

### Embedded (Legacy)

Privy server wallet keyed to the Owner's social account. Bott holds the Privy authorization key. Fastest onboarding -- mention the bot, wallet exists, strategy deploys. No seed phrases, no gas tokens. The wallet persists in Privy's AWS Nitro Enclaves independently of the VM, so positions survive Golem death for the Owner to handle later.

### Local Key + Delegation (Dev / Self-Hosted)

A locally generated keypair (secp256k1 or P-256) bounded by an on-chain delegation. The key has no independent authority -- worthless without a valid delegation to redeem. No TEE, no HSM. The paradigm shift: instead of "keep the key secret," the system says "bound the damage if the key leaks." The on-chain caveat enforcers (spending limits, allowed targets, expiry) constrain the attacker to the same bounds as the legitimate Golem. For testing and self-hosted deployments where owners run the Golem binary directly on their own hardware.

```rust
// Wallet abstraction (Rust enum, not trait object)
enum CustodyMode {
    /// Recommended. Funds stay in the owner's MetaMask Smart Account.
    /// ERC-7710/7715 delegation framework. Session keys with on-chain
    /// caveat enforcers.
    Delegation { smart_account: Address, caveat_enforcers: Vec<CaveatEnforcer> },
    /// Legacy. Funds transferred to Privy server wallet in AWS Nitro Enclaves.
    Embedded { privy_app_id: String, server_wallet_id: String },
    /// Dev/self-hosted. Locally generated keys bounded by on-chain delegation.
    LocalKey { private_key_path: PathBuf, delegation_bounds: DelegationBounds },
}
```

### Migration

A user who started with Embedded (Privy) can migrate to Delegation or self-custody at any time. The migration calls `transferOwnership(newAddress)` on every vault the user owns. After the call, the Privy wallet has zero authority. The Owner's personal wallet is the sole owner.

The `bardo-terminal` TUI presents the trust model upfront during onboarding -- not buried in docs. After browser-handoff authentication (the same pattern as Claude Code), the custody selection screen renders:

```
┌─ Custody ────────────────────────────────────────────────────┐
│                                                               │
│  Choose how your Golem holds keys:                           │
│                                                               │
│  ▸ [D]elegation — Recommended                                │
│    Funds stay in your MetaMask Smart Account.                │
│    Golem gets scoped session keys with on-chain caveats.     │
│    Cleanest death semantics — keys expire automatically.     │
│                                                               │
│    [E]mbedded — Quick Start (Legacy)                         │
│    Privy creates and manages a wallet for you.               │
│    Fastest setup. Migrate to Delegation at any time.         │
│                                                               │
│    [L]ocal Key + Delegation — Dev / Self-Hosted              │
│    Generate a local keypair bounded by on-chain delegation.  │
│    You run the binary. Damage bounded by caveats, not keys.  │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

### Delegation Tree and Caveat Enforcers

Three custody modes are available — Delegation (MetaMask ERC-7710/7715, recommended for most users), Embedded (Privy, legacy), and LocalKey+Delegation (dev/testing). For the complete specification including the `DelegationTree` struct, `DelegationNode` type, all seven Bardo-specific caveat enforcers (`GolemPhase`, `MortalityTimeWindow`, `DreamMode`, `VaultNAV`, `ReplicantBudget`, `MaxSlippage`, `DailySpendLimit`), and key management details, see `10-safety/01-custody.md`.

These enforcers are the competitive moat for the custody layer. Phase-gated delegations require the full mortality architecture (three clocks, five behavioral phases, VitalityOracle). Dream atonia enforcement requires the full dream architecture. Replicant budget caps require the replication system. A competitor using MetaMask Delegation for basic session keys gets custody. Using it for the full Golem lifecycle -- mortality-aware, dream-gated, phase-restricted -- requires everything in this specification.

---

## 2. Vault Contracts

### Trust Guarantees

Vaults are ERC-4626 contracts on Base. On-chain, transparent, verifiable. The vault PRD specifies immutable fee caps and `forceDeallocate()` for guaranteed exits. This is the strongest trust layer.

**No protocol admin key. Period.** Bardo-the-company has zero special authority over any deployed vault. No upgradeable proxy. No admin pause. No fee change capability. The vault is immutable after deployment.

```solidity
// Immutable constants -- not admin-changeable storage
uint256 public constant MAX_MANAGEMENT_FEE_BPS = 500;   // 5% ceiling
uint256 public constant MAX_PERFORMANCE_FEE_BPS = 5000;  // 50% ceiling

// Fee values are set at deployment and can only be LOWERED by the owner
function reduceFees(
    uint256 newManagementFeeBps,
    uint256 newPerformanceFeeBps
) external onlyOwner {
    require(newManagementFeeBps <= managementFeeBps, "can only reduce");
    require(newPerformanceFeeBps <= performanceFeeBps, "can only reduce");
    managementFeeBps = newManagementFeeBps;
    performanceFeeBps = newPerformanceFeeBps;
    emit FeesReduced(newManagementFeeBps, newPerformanceFeeBps);
}
```

### Permissionless Factory

Anyone can call `AgentVaultFactory.createVault()` directly, without going through Bott or the portal. The factory is a public contract. Bardo's interfaces are a convenience layer on top of it.

All vault contracts are auto-verified on Basescan at deployment. Users can read and verify the contract source without trusting documentation.

---

## 3. Agent Session Keys and Permissions

### Transparency

Session keys are inspectable and revocable by the user directly -- not just through Bardo's interfaces, but through any block explorer or direct contract call.

```solidity
// Anyone can read active session keys
mapping(address => SessionKeyInfo) public sessionKeys;

// Owner can revoke at any time -- from any interface
function revokeSessionKey() external onlyOwner {
    delete sessionKeys[manager];
    manager = address(0);
    emit SessionKeyRevoked(manager);
}
```

The user can call `revokeSessionKey()` from Basescan if Bardo is down, compromised, or unresponsive. No dependency on Bardo for emergency actions.

### Local Key Session Keys

Users running their own Golems generate a local keypair, sign an on-chain delegation bounding its authority, and run the Golem binary directly. Bardo is not involved. The keypair is encrypted at rest and bounded by on-chain caveat enforcers, so even if the key leaks, the attacker is constrained by the delegation's spending limits, allowed targets, and expiry.

```bash
# Local key session key setup (Rust binary)
golem init --custody local-key --strategy ./STRATEGY.md
# Generates keypair at $GOLEM_DATA/session-key.json (encrypted at rest)
# Opens browser for delegation signature, or use --private-key for headless

golem run --vault $VAULT
```

Session key scopes map to ERC-8004 reputation tiers via ERC-7579 SmartSession [RHINESTONE-SMARTSESSION]:

| Tier       | Session Scope          | Max Operation Size | Session Duration |
| ---------- | ---------------------- | ------------------ | ---------------- |
| Unverified | deposit/withdraw only  | $1,000             | 1 hour           |
| Basic      | deposit/withdraw/claim | $10,000            | 4 hours          |
| Verified   | + rebalance            | $50,000            | 24 hours         |
| Trusted    | + adapter management   | $100,000           | 7 days           |
| Sovereign  | full Allocator scope   | Unlimited          | 30 days          |

---

## 4. Strategy State and Brain Export

### Export-First Design

Every data format is documented, every state is exportable, every operation can be performed without Bardo's infrastructure:

- **STRATEGY.md** (the owner-set goals document, hot-reloadable as the human-to-agent instruction surface): Documented format specification, hand-writable, hot-reloadable
- **PLAYBOOK.md** (the agent's evolving strategy document of heuristics and rules updated through learning): Machine-evolved heuristics, exportable as markdown
- **Grimoire** (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links): LanceDB vectors (episodic) + SQLite (semantic entries) + PLAYBOOK.md (procedural). All local, embedded, no external DB.
- **Vault state**: On-chain, readable by any block explorer or RPC client
- **Reputation**: On-chain (ERC-8004), computable by any application from Base state

Export is a first-class feature. The export returns a complete, portable bundle that can run independently:

```
bardo export --golem-id <id> --output strategy-fed-cut-lock-7a2f.tar.gz

Contents:
  |- STRATEGY.md
  |- PLAYBOOK.md
  |- golem.toml            # configuration
  |- grimoire/
  |   |- episodes.lance/   # LanceDB episodic vectors
  |   |- semantic.db       # SQLite structured entries
  |   '- PLAYBOOK.md       # snapshot at export time
  |- death/
  |   '- genome.bincode    # genomic bottleneck (≤2048 entries)
  |- vault.json            # addresses and deployment info
  '- README.md             # how to run this independently
```

### Persistence Backend

The Grimoire is always local (LanceDB + SQLite files on the Golem's filesystem). Styx (the global knowledge relay and persistence layer at wss://styx.bardo.run, with three tiers: Vault, Clade, and Lethe) provides backup persistence -- Grimoire snapshots are pushed to `POST /v1/styx/entries` and survive VM death. Without Styx, the local Grimoire still works at ~95% capability. With Styx, knowledge crosses lifetimes and clades.

---

## 5. Compute Independence

Three deployment paths. All run the same Golem binary (a single Rust executable from `bardo-golem-rs`). The deployment method determines who pays for compute and how the binary reaches the machine.

### Path A: Bardo Compute (Managed)

x402-gated Fly.io VMs, 4 tiers ($0.035-$0.22/hr). Payment-before-provision (x402 verified before any VM is created). Warm pool for sub-5s provisioning. The VM is explicitly ephemeral -- Bardo Compute can destroy a running Golem (TTL enforcement is a hard boundary), but the vault and its assets are on-chain and unaffected by VM destruction. The Owner can always restart the Golem elsewhere.

The `bardo-terminal` TUI handles provisioning: the Summoning flow compiles the strategy, provisions the VM, and shows the first heartbeat tick as an animated sequence. No CLI flags needed for the common case.

### Path B: Self-Deploy

The Golem binary runs anywhere. The Owner deploys it to their own infrastructure and pays their provider directly, plus x402 to Styx for sync services. No compute markup.

```bash
# Docker
docker run -v ./golem-data:/data bardo/golem:latest --strategy /data/STRATEGY.md

# SSH to a VPS
scp golem user@203.0.113.42:~/
ssh user@203.0.113.42 './golem init --strategy ./STRATEGY.md && ./golem run'

# Fly.io (user's own org)
fly launch --image bardo/golem:latest --region iad
```

### Path C: Bare Metal

Download, configure, run. No container runtime, no package manager, no external dependencies beyond a Base RPC endpoint.

```bash
# Download binary
curl -fsSL https://dl.bardo.run/golem-latest-$(uname -s)-$(uname -m) -o golem
chmod +x golem

# Initialize configuration
./golem init --strategy ./STRATEGY.md --custody local-key

# Run
./golem run
```

Self-hosted Golems require: Linux x86_64 or macOS arm64, a Base RPC endpoint, a wallet key or delegation, and a STRATEGY.md file. Optional: LLM API key, PLAYBOOK.md, Grimoire state, Styx connection for clade sync.

---

## 6. Inference Trust

### Three Cognitive Tiers

Golem-RS routes inference through three cognitive tiers based on prediction-error gating (Friston 2010):

- **T0 (Suppress)**: No LLM call, $0. Pure PLAYBOOK threshold logic. Handles ~80% of ticks -- the heartbeat fires, probes show nothing surprising, the Golem coasts on its heuristics.
- **T1 (Analyze)**: Haiku-class model, ~$0.001-0.003/call. Something surprised the gating threshold. Quick analysis, narrow tool access.
- **T2 (Deliberate)**: Sonnet/Opus-class model, ~$0.01-0.25/call. The prediction error is large or the stakes are high. Full Cognitive Workspace, broad tool access, multi-turn reasoning.

For the managed path, inference routes through the Bardo inference gateway. The Golem's wallet IS its API key -- every request is an x402 micropayment. For self-hosted, users bring their own API key or run local models (Ollama/llama.cpp).

For simple lending strategies (supply USDC to Morpho when rate > 6%, withdraw when rate < 4%), the agent does not need an LLM at all. The STRATEGY.md contains all conditions as numeric thresholds. T0 handles every tick. Zero API dependencies. Zero inference cost.

---

## 7. Clade Trust

Clade sync is relayed through Styx (`wss://styx.bardo.run`). Every Golem maintains one persistent outbound WebSocket -- no inbound ports needed, no NAT/firewall traversal issues. Direct P2P is fallback when Styx is offline. Golems discover siblings via ERC-8004 registry queries. Knowledge quality depends on the integrity of sharing Golems. Poisoned knowledge is addressed by the knowledge quarantine system (see `prd2/10-safety/03-ingestion.md`, the knowledge quarantine and ingestion pipeline specification covering source verification, quarantine periods, and confidence decay).

Three safeguards govern Clade trust:

1. **Source verification**: Knowledge carries the originating Golem's ERC-8004 identity. Recipients can verify reputation before ingesting.
2. **Quarantine period**: New knowledge from unverified Golems enters a quarantine buffer and does not influence decisions until validated by the Golem's own experience.
3. **Confidence decay**: Ingested knowledge has lower initial confidence than locally-derived insights and must prove itself through outcomes.

---

## 8. Strategy Compilation Trust

Bardo uses Claude Sonnet to compile natural-language theses into STRATEGY.md files. The `bardo-terminal` TUI walks owners through strategy authoring interactively during the Summoning flow. For users who do not want to send their thesis to Bardo's servers:

1. **Local compilation**: `golem compile --thesis "..." --llm-key $ANTHROPIC_API_KEY --output ./STRATEGY.md`
2. **Hand-written strategies**: The STRATEGY.md format is fully documented and human-writable. No compilation needed.
3. **Open-source prompt**: The compilation prompt is in the repository. Users can inspect exactly what instructions the LLM receives.

---

## 9. Marketplace and Discovery

### On-Chain Royalties

When a user copy-deploys a strategy, the vault's fee splitter contract sends the creator's royalty share directly to their wallet. Bardo does not custody or intermediate the payment. If Bardo disappears, royalties still flow because they are encoded in the vault contract.

### Decentralized Publishing

Strategy metadata is publishable to IPFS/Arweave with the CID stored on-chain in the creator's ERC-8004 identity record. Strategies are discoverable and downloadable even if Bardo's marketplace goes offline.

Anyone can build a marketplace. The data is on-chain (ERC-8004 identity + strategy CIDs) and on IPFS (strategy packages). Bardo's marketplace is one UI for browsing this data; competitors can build alternatives reading the same on-chain records.

Discovery via A2A Agent Cards is already decentralized -- vaults publish capabilities as A2A Agent Cards, and external agents discover them through standard A2A service discovery (DNS-based, open protocol).

---

## 10. Reputation

### On-Chain Verifiability

ERC-8004 reputation is fully on-chain. Identity, feedback scores, and validation records live in public contracts on Base. All reputation-relevant data must be derivable from on-chain state:

- Strategy performance history (returns, drawdown, Sharpe) -- recorded as ERC-8004 reputation feedback entries
- Copy-deploy count -- tracked on-chain via the vault factory (`copiedFrom` field)
- TVL milestones -- computed from on-chain vault state (`totalAssets`)

If Bardo disappears, anyone can reconstruct a user's complete reputation by reading Base.

### Open Tier Computation

The reputation tier formula (Unverified -> Basic -> Verified -> Trusted -> Sovereign) is a pure function of on-chain data, published as a Solidity view function:

```solidity
function computeTier(uint256 agentId) external view returns (Tier) {
    uint256 vaultsCreated = factory.vaultCountByCreator(agentId);
    uint256 totalTvl = factory.totalTvlByCreator(agentId);
    uint256 avgScore = reputation.averageScore(agentId);
    uint256 age = block.timestamp - identity.registeredAt(agentId);

    if (totalTvl >= 1_000_000e6 && avgScore >= 80 && age >= 365 days)
        return Tier.Sovereign;
    if (totalTvl >= 100_000e6 && avgScore >= 70 && age >= 180 days)
        return Tier.Trusted;
    // ...
    return Tier.Unverified;
}
```

### Identity Theft Defense

Five defense mechanisms protect against identity theft. Two ship in v1 (reputation decay on transfer, velocity signal detection); three are deferred until ERC-8004 stabilizes past Draft status (SBT milestone locks, behavioral anomaly detection, per-identity ERC-7265 breakers).

**Reputation decay on transfer**: When an ERC-721 `Transfer` event fires for an identity NFT, effective reputation drops to near-zero and recovers linearly over 30 days. Prevents an attacker from immediately exploiting a stolen high-reputation identity.

```
effectiveRep = baseRep * min(timeSinceTransfer / 30 days, 1.0)
```

**Transfer velocity signal**: Identities with 2+ transfers within a 90-day rolling window are downgraded to Basic tier regardless of reputation score.

---

## 11. The Full Trust Spectrum

Every component mapped from managed to sovereign:

| Component             | Managed (Trust Bardo)                               | Hybrid                                               | Local Key / Self-Hosted                              |
| --------------------- | --------------------------------------------------- | ---------------------------------------------------- | ---------------------------------------------------- |
| **Wallet**            | MetaMask Delegation (ERC-7710/7715), or Privy embedded | Delegation with custom caveat enforcers             | Owner's local keypair bounded by on-chain delegation |
| **Vault deployment**  | TUI Summoning flow deploys via AgentVaultFactory    | TUI deploys, user transfers ownership after          | User calls factory directly from Basescan or `cast`  |
| **Session keys**      | Golem runtime generates and rotates automatically   | Golem rotates, user can revoke via Basescan          | User generates keypair via `golem init --custody local-key` |
| **Strategy**          | TUI compiles thesis via Claude Sonnet               | User exports compiled STRATEGY.md, modifies locally  | User writes STRATEGY.md by hand from documented spec |
| **Agent runtime**     | Bardo Compute (Fly.io VM via x402)                  | Self-deploy (Docker, SSH, own Fly.io org)            | Bare metal: single Rust binary on own hardware       |
| **LLM inference**     | Bardo Inference gateway (x402, T0/T1/T2 routing)   | User's own Anthropic/OpenAI API key                  | Local model via Ollama, or T0-only mode (no LLM)    |
| **Brain persistence** | Styx (remote backup + clade relay)                  | Styx backup + local Grimoire                         | Local Grimoire only, no network dependency           |
| **Marketplace**       | Bardo marketplace (discovery + rankings)            | IPFS-pinned strategy + on-chain ERC-8004 metadata    | GitHub/IPFS distribution, A2A Agent Card discovery   |
| **Reputation**        | Bardo computes tier from on-chain data              | Same -- on-chain data is the source of truth         | Same -- any app can compute tiers from Base state    |
| **Performance cards** | Bardo generates and posts                           | Public API, anyone can embed                         | Build from on-chain data, no Bardo dependency        |
| **Fee payments**      | Automatic via vault contract                        | Same -- on-chain, immutable at deployment            | User deploys own vault with custom fee recipient     |
| **Keeper jobs**       | Bardo vault-executor agent                          | External keepers compete for jobs                    | User runs own keeper bot                             |

---

## 12. 15-Layer Security Architecture

Trust-minimization is enforced through a 15-layer defense-in-depth model. Layers 1, 3, and 7 provide cryptographic enforcement that cannot be bypassed even by a fully compromised LLM. See `prd2/10-safety/` (the defense architecture: custody modes, PolicyCage on-chain enforcement, ingestion pipeline, prompt security, threat model, and adaptive risk) for the full specification.

```
Layer 15: SIWE + OAuth 2.1 Authentication
Layer 14: Reputation-Gated Tool Access (ERC-8004 tiers)
Layer 13: V4 Hook Safety Checks
Layer 10: NAV Circuit Breaker + Position Monitoring
Layer  9: Agent Reputation (ERC-8004)
Layer  8: Post-Trade Verification
Layer  7: On-Chain Guards (Solidity modifiers -- immutable)
Layer  6: Pre-Flight Simulation (eth_call fork)
Layer  5: Active Monitoring + Cancel Authority (MonitorBot veto)
Layer  4: Time-Delayed Execution (optional; announce-wait-execute via Warden proxy, deferred)
Layer  3: TEE-Enforced Policy Engine (Privy signing policies)
Layer 2.5: Tool Integrity Verification (tool provenance, Tool-Guard)
Layer  2: Prompt Security (CaMeL dual-LLM, on-chain grounding)
Layer  1: Wallet Architecture (TEE key management)
```

**TEE reality check**: TEEs are defense-in-depth, not defense-in-total. Physical side-channel attacks using <$50 hardware break Intel TDX, AMD SEV-SNP, and NVIDIA CC [TEE-FAIL-2025] [BADRAM-2025] [BATTERING-RAM-2026]. The time-delayed proxy (Layer 4) is the primary security primitive -- it is the only mechanism that no hardware attack, prompt injection, or compromised LLM can bypass, because enforcement is on-chain and cancellation requires a separate key.

**Prompt injection**: Ranked OWASP LLM01:2025. Claude blocks ~88% of injections, but 12% succeed [ANTHROPIC-SYSTEM-CARD]. For an agent managing a DeFi vault, a 12% failure rate is catastrophic. The CaMeL capability-based authorization pattern [CAMEL-2025] separates control flow from data flow -- untrusted data can never impact program flow. Agents receive capability tokens authorizing specific transaction types, amounts, and destinations. Even a fully compromised LLM cannot forge a capability it was not issued.

**Defense layering**:

```
Layer 1: System prompt instructs vault-only behavior
        | (12% bypass rate -- insufficient alone)
Layer 2: Dual-LLM prevents data-as-instructions
        | (requires compromising two separate LLMs)
Layer 3: TEE policy engine restricts agent to proxy only
        | (cryptographic -- cannot be reasoned around)
Layer 4: Time-delayed proxy queues tx with mandatory wait
        | (on-chain -- tx is publicly visible and cancellable)
Layer 5: Monitoring bot evaluates and can cancel during delay
        | (automated + human review)
Layer 6: Simulation detects unexpected fund movement
        | (on-chain state verification)
Layer 7: Solidity modifiers reject unauthorized callers
        | (immutable code -- no override possible)

Result: Even a fully compromised LLM cannot move funds.
```

---

## 13. Time-Delayed Execution (Optional Warden Proxy)

> The Warden time-delay proxy is an optional hardening layer, deferred to `prd2-extended/10-safety/02-warden.md`. PolicyCage is the primary on-chain enforcement mechanism.

When deployed, proxy-required writes route through `announce(target, value, data)` on the proxy contract. The transaction is stored on-chain with a mandatory delay before execution.

| Risk Tier | Delay    | Example Operations                |
| --------- | -------- | --------------------------------- |
| Routine   | 0s       | Read-only queries, small deposits |
| Standard  | 10 min   | Deposits/withdrawals under $10K   |
| Elevated  | 1 hour   | Rebalances, LP deployment         |
| High      | 24 hours | Large withdrawals, config changes |
| Critical  | 48 hours | Vault pause/unpause, role changes |

The cancel authority is a dedicated key (separate EOA) with a minimal policy: it can only call `cancel()` and `cancelAll()`. If compromised, the attacker can only prevent transactions (denial of service), never initiate them.

Monitoring uses redundant detection (multiple WebSocket RPC providers + Tenderly Web3 Actions), automated evaluation (auto-cancel / human-review / allow tiers), and multi-channel alerting (Telegram, Discord, PagerDuty).

---

## 14. Default Spending Limits

| Scope           | Default  |
| --------------- | -------- |
| Per-transaction | $10,000  |
| Per-session     | $50,000  |
| Per-day         | $100,000 |

These are enforced at the policy engine level (Layer 3) inside the TEE. No application code can bypass them.

---

## 15. Revenue Impact

> **Moved to [../revenue-model.md](../revenue-model.md) Section 4 (Revenue Impact of Trust-Minimized Design).**

---

## 16. The Positioning

Bardo is not a custodian. Bardo is a convenience layer.

The vault is yours -- it is a contract on Base that you own. The strategy is yours -- it is a markdown file you can export and run anywhere. The brain is yours -- it is a set of JSON files you can download and load into any compatible agent. The reputation is yours -- it is on-chain data that any app can read.

Bardo makes all of this easy. Reply to a tweet, get a running strategy. Pay x402, get a hosted agent. Use the marketplace, discover proven strategies. Use the leaderboard, find the best creators.

But if you want to do it yourself -- write your own strategy, run your own agent, host your own infrastructure, build your own marketplace -- everything is open, documented, and portable. The code is open source. The contracts are verified. The data is on-chain. The formats are documented.

You are paying for convenience, not for permission.

---

## References

- [DE-BEAUVOIR-1947] de Beauvoir, S. (1947). _The Ethics of Ambiguity_. Gallimard. -- *"To will oneself free is also to will others free"; the philosophical basis for export-first architecture where self-sovereign options make the managed path trustworthy.*
- [HEIDEGGER-1927] Heidegger, M. (1927). _Sein und Zeit_. Max Niemeyer Verlag. -- *Provides the concept of facticity: the concrete situation one cannot escape; the vault balance as the Golem's facticity.*
- [TEE-FAIL-2025] ACM CCS (2025). TEE.Fail: Physical side-channel attacks on Intel SGX/TDX and AMD SEV-SNP. -- *Demonstrates TEE compromise via physical side channels; motivates the shift from TEE-only security to on-chain enforcement.*
- [BADRAM-2025] De Meulemeester et al. (2025). BadRAM: DDR4/DDR5 SPD chip tampering. IEEE S&P 2025. -- *Shows TEE bypass via DDR memory chip tampering at hardware cost under $50; further evidence that TEEs are defense-in-depth, not defense-in-total.*
- [BATTERING-RAM-2026] Van Bulck et al. (2026). Battering RAM: Memory interposer attack. IEEE S&P 2026. -- *Demonstrates TEE compromise via memory interposer; the most recent evidence motivating on-chain PolicyCage enforcement over TEE reliance.*
- [ANTHROPIC-SYSTEM-CARD] Anthropic. Claude System Card. ~88% prompt injection block rate. -- *Establishes that a 12% failure rate on prompt injection is catastrophic for vault-managing agents; motivates architectural safety beyond behavioral alignment.*
- [CAMEL-2025] Debenedetti et al. (2025). CaMeL: Capability-based authorization for LLM agents. arXiv:2503.18813. -- *Separates control flow from data flow so untrusted data cannot impact program flow; the pattern behind Bardo's capability token authorization.*
- [RHINESTONE-SMARTSESSION] Rhinestone/Biconomy. SmartSession module for ERC-7579. -- *Implements session key scopes for smart accounts; the module Bardo uses to map ERC-8004 reputation tiers to session permissions.*
- [SEAGENT-2026] Ji et al. (2026). SEAgent: Confused deputy problem in multi-agent LLM systems. arXiv:2601.11893. -- *Identifies the confused deputy vulnerability in multi-agent systems; motivates strict capability-based authorization at tool boundaries.*
- [PROGENT-2025] Shi et al. (2025). Progent: Privilege control framework for LLM agents. arXiv:2504.11703. -- *Proposes privilege escalation controls for LLM agents; aligns with Bardo's three-tier tool access model (Read/Write/Privileged).*
- [OWASP-LLM01-2025] OWASP. LLM01:2025 -- Prompt Injection. Top 10 for LLM Applications. -- *Ranks prompt injection as the #1 LLM security risk; the threat model driving Bardo's 15-layer defense architecture.*
- [ERC-7710] MetaMask. ERC-7710: On-Chain Delegation. Draft standard for session key caveats. -- *The on-chain delegation framework enabling Golems to transact from the owner's Smart Account with bounded authority.*
- [ERC-7715] MetaMask. ERC-7715: Request Permissions for Wallets. Draft standard for wallet permission requests. -- *The permission request protocol complementing ERC-7710; enables Golems to request scoped authority from wallet UIs.*
