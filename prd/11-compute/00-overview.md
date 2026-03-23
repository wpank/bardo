# 00 -- Compute platform overview [SPEC]

**Bardo Compute: x402-gated VM hosting for mortal Golems**

> Related: [01-architecture.md](01-architecture.md) (system topology), [02-provisioning.md](02-provisioning.md) (VM lifecycle), [prd2/01-golem/00-overview.md](../01-golem/00-overview.md) (Golem runtime)

> **Reader orientation:** This document is the entry point for the **Compute** layer of Bardo (the Rust runtime for mortal autonomous DeFi agents). Bardo Compute is the managed VM hosting service: users pay with USDC micropayments via the x402 protocol (EIP-3009 signed transfers on Base L2), and a Fly.io VM spins up running the Golem agent binary. The key concept before diving in: every VM has a finite lifespan tied directly to the USDC credit paid for it, and anyone can extend any Golem's life with another payment. Terms like Golem, Grimoire, and Styx are defined inline on first use; a full glossary appears in the Terminology section below.

---

## What is Bardo Compute

Bardo Compute is a **pay-per-use VM provisioning service** for autonomous Golem (mortal autonomous DeFi agent compiled as a single Rust binary) agents, powered by x402 (a micropayment protocol using EIP-3009 signed USDC transfers on Base) USDC micropayments on Base. Any user can deploy a Golem by sending a single x402 payment -- no subscription, no credit card, no human intermediation. The VM provisions in under 5 seconds (warm pool) or 15-30 seconds (cold fallback). It runs until its USDC credit expires, and **anyone** can extend any Golem's life with another x402 payment.

Each VM hosts a **Golem-RS runtime** (`golem-runtime` crate) -- a single Rust binary that runs the Heartbeat pipeline (the 9-step autonomous decision cycle: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect), the 28-extension cognitive architecture, the Grimoire (persistent knowledge store combining LanceDB episodes, SQLite insights, and filesystem heuristics) knowledge store, and a wallet operating in one of three custody modes (Delegation, Embedded, or LocalKey). The `bardo-tools` crate provides 423+ DeFi tools loaded via profile-based activation. Bardo Compute does not own the agent runtime. It provisions, monitors, and destroys the VMs that run Golems.

Bardo Compute is one of three deployment paths. It is the managed path -- the path where you pay a markup and get zero infrastructure management in return. The other two paths (self-deploy and bare-metal) run the identical Golem binary on infrastructure you control. See the deployment paths section below.

### Relationship to other products

| Product | Role | Compute's relationship |
| --- | --- | --- |
| **Golem** | Autonomous agent runtime (`golem-runtime` crate) | Compute provisions and hosts Golem VMs |
| **Styx** (global knowledge relay for inter-agent intelligence sharing) | Knowledge ecology service (`wss://styx.bardo.run`) | Each Golem connects outbound to Styx via persistent WebSocket; Compute handles nothing about the connection |
| **Inference** | LLM gateway (x402-gated, multi-provider) | Golems call Inference from within Compute VMs; Bankr self-funding path closes the metabolic loop |
| **Vault** | On-chain capital (ERC-4626 + ERC-8004 gating) | Golems deployed on Compute execute vault strategies; vaults are optional, not required |

Compute is infrastructure. It has no opinion about what the Golem does -- only that it is paid for, properly isolated, and destroyed when credit runs out.

---

## One Command Deploy

The deploy experience is a single CLI command:

```
$ bardo deploy --strategy ./my-arb-strategy.md --tier small
Deploying golem...
  Strategy: my-arb-strategy.md (arbitrage, ETH/USDC)
  Tier: small (1 CPU, 512MB, $0.05/hr)
  Payment: 2.40 USDC (48 hours)

  [################] Provisioning... 4.2s

  golem-kX9mR2Yq7pLN is alive.

  Heartbeat:    active (adaptive, theta ~60s)
  Styx:         connected (23ms latency)
  Wallet:       0x7a3f...9e21 (delegation mode)
  TTL:          47h 59m remaining

  Connect:      bardo attach golem-kX9mR2Yq7pLN
  Monitor:      bardo status golem-kX9mR2Yq7pLN
```

That's the entire deploy experience. One command. A USDC payment. A running Golem. Most people who want to run a DeFi agent don't want to become sysadmins. They have a strategy and capital. Bardo Compute exists because the alternative is renting a VPS, configuring containers, and hoping nothing breaks at 3 AM.

### Four-Stage Provisioning Pipeline

The abstraction layer has four stages, and users see none of them:

1. **TUI command** -- `bardo deploy` reads the strategy file, picks a tier (or uses the default `small`), and constructs an x402 payment header containing an EIP-3009 `receiveWithAuthorization` payload. The user's wallet signs the USDC transfer.

2. **Bardo API** -- The control plane at `api.bardo.money` validates the x402 payment, verifies the signature via ecrecover, checks nonce uniqueness against Turso, and writes a provisioning intent row to the database. Returns 202 Accepted.

3. **Fly.io provisioning** -- The control plane claims a pre-warmed stopped machine from the warm pool (or creates a new one if the pool is empty). It injects `golem.toml`, `STRATEGY.md`, and wallet configuration via Fly's native file injection. Starts the machine.

4. **Running container** -- The Golem binary boots in ~2 seconds (it's Rust, not Node). Initializes Grimoire, loads the tool profile, verifies its wallet delegation on-chain, connects to Styx via outbound WebSocket, and starts its heartbeat pipeline. The health server reports "ready."

The user's TUI polls for status transitions during provisioning. When the Golem reports ready, the TUI shows the confirmation.

---

## Three deployment paths

The Golem is a single Rust binary. It connects outbound to Styx via persistent WebSocket (zero inbound ports needed). How it gets onto a machine and who pays for the machine -- those are the only variables. Everything else is identical.

### Path A: Bardo Compute (managed)

For users who want zero infrastructure management. Pay x402, get a running Golem. The service provisions Fly.io VMs, injects configuration, monitors health, destroys machines at TTL expiry. Users pay an x402 markup that covers Fly.io costs plus margin.

| VM tier | Fly.io config | x402 price/hr | Fly.io cost/hr | Typical use |
| --- | --- | --- | --- | --- |
| micro | shared-cpu-1x / 256MB | $0.035 | $0.004 | Simple monitors, keepers |
| small | shared-cpu-1x / 512MB | $0.065 | $0.008 | Standard Golem (default) |
| medium | shared-cpu-2x / 1GB | $0.12 | $0.015 | Multi-strategy, heavy dreams |
| large | shared-cpu-4x / 2GB | $0.22 | $0.030 | Full observatory + trading |

### Path B: Self-deploy helper (automated)

For users who want to control their own infrastructure but skip the manual setup. They bring their own Fly.io, Railway, or VPS account. The `bardo` CLI automates provisioning:

```bash
bardo deploy --provider fly --app-name my-golem --region iad
bardo deploy --provider railway --project my-golem
bardo deploy --provider ssh --host 203.0.113.42 --user root
bardo deploy --provider docker --name my-golem
```

The user pays their own provider directly. They also pay x402 to Styx for sync, pheromone, and retrieval services. No compute markup.

### Path C: Manual / bare metal

Download the binary, configure it, run it.

```bash
curl -sSL https://get.bardo.run/golem | sh
bardo init   # Interactive wizard: wallet, strategy, Styx connection
./golem-binary --config golem.toml
```

For users who already have a running `bardo` installation and want to extract a portable static binary for deployment on a server:

```bash
# Export a statically-linked musl binary for Linux deployment
bardo export-binary --target x86_64-unknown-linux-musl > golem
scp golem server:~/golem && ssh server "BARDO_CONFIG=./golem.toml ./golem"
```

The exported binary is fully self-contained (~15-25 MB, statically linked against musl). No runtime dependencies, no dynamic libraries. Deploy it to any Linux machine and run it with a `golem.toml` config. See [prd2/17-monorepo/01-rust-workspace.md](../17-monorepo/01-rust-workspace.md) for build target details.

A Raspberry Pi behind double-NAT works identically to a Fly.io VM because the connection model is outbound-only. The user handles everything -- hardware, OS, networking, uptime.

### Cost comparison (30-day, small tier)

| Path | Compute cost | Styx cost | Total |
| --- | --- | --- | --- |
| **Bardo Compute** | ~$47/mo | ~$15/mo | ~$62/mo |
| **Self-deploy (Fly.io)** | ~$6/mo | ~$15/mo | ~$21/mo |
| **VPS (Hetzner)** | ~$4/mo | ~$15/mo | ~$19/mo |
| **Home hardware** | ~$3/mo electricity | ~$15/mo | ~$18/mo |

Bardo Compute is 3x more expensive. The markup IS the product -- you pay for convenience.

---

## Why mortality is architecture

VMs are mortal by design. USDC credit determines lifespan. There is no free tier, no grace period, no "paused" state. When credit expires, the Golem executes Thanatopsis (the four-phase death protocol: Acceptance, Settlement, Reflection, Legacy -- flush Grimoire, settle positions, write death testament) and the VM is destroyed.

This is not incidental. The mortality constraint transforms every other design decision:

- **Memory** implements Ebbinghaus decay and Curator pruning because knowledge must stay fresh within a bounded lifespan
- **Inference** routes 95% of decisions through T0/T1 (cheap or free) because burning $85/day on Opus kills a $0.065/hr Golem in days
- **Safety** enforces phase-gated tool access because a dying Golem (Conservation/Terminal phase) should not open new positions
- **Coordination** uses Bloodstain death warnings because dead Golems' knowledge carries zero survival bias
- **Custody** uses time-locked delegations (MortalityTimeWindow caveat enforcer) that auto-expire at projected death, eliminating stuck funds

A Golem burning $0.40/day (context-engineered, LLM-last) lives 2,500 days on $1,000. A naive agent burning $85/day lives 12 days. The mortality-aware agent is 212x more efficient because efficiency is survival.

See [prd2/02-mortality/01-architecture.md](../02-mortality/01-architecture.md) for the full mortality thesis.

---

## Mandatory ERC-8004 registration

Every Golem MUST register an ERC-8004 (on-chain agent identity standard) on-chain identity on Base L2 before it can connect to Styx. This is non-negotiable. Registration happens during onboarding (TUI `bardo init`, web deploy wizard, or Bardo Compute provisioning). Gas cost: ~$0.01-0.05 on Base.

ERC-8004 identity enables:

- Clade (an owner's fleet of sibling Golems sharing knowledge) discovery (Styx groups Golems by `user_id` from ERC-8004)
- Reputation scoring (ERC-8004 score gates Lethe (formerly Commons) access at >=50)
- Bloodstain provenance (death warnings are EIP-712 signed against ERC-8004 identity)
- Generational succession tracking

```solidity
struct AgentRegistration {
    address walletAddress;       // Custody-mode wallet address
    address ownerAddress;        // Owner's wallet (for clade grouping)
    uint256 generation;          // 0 for first-gen, N for Nth successor
    address parentAgent;         // Previous generation's address (0x0 for first-gen)
    string deploymentType;       // "managed" | "self-deployed" | "bare-metal"
    uint256 registeredAt;        // Block timestamp
}
```

Styx verifies ERC-8004 registration during WebSocket authentication. Unregistered Golems are rejected.

---

## Three custody modes

The Golem's wallet is not a single architecture. Owners select a custody mode at provisioning time.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyMode {
    /// Recommended. Funds stay in the owner's MetaMask Smart Account.
    /// ERC-7710/7715 delegation framework. Session keys with on-chain
    /// caveat enforcers. Key compromise is bounded by caveats.
    Delegation {
        smart_account: Address,
        caveat_enforcers: Vec<CaveatEnforcer>,
    },

    /// Legacy. Funds transferred to Privy server wallet in AWS Nitro
    /// Enclaves. Simpler but custodial.
    Embedded {
        privy_app_id: String,
        server_wallet_id: String,
    },

    /// Dev/self-hosted. Locally generated keys bounded by on-chain
    /// delegation. No TEE, no HSM -- damage bounded by caveats.
    LocalKey {
        private_key_path: PathBuf,
        delegation_bounds: DelegationBounds,
    },
}
```

Delegation mode is strictly superior for death settlement: the owner's funds never leave their wallet, no sweep is needed at death, and the delegation simply expires. See [04-security.md](04-security.md) for the full custody security model.

---

## Three critical invariants

1. **Payment-before-provision**: An x402 payment must be cryptographically verified before any Fly machine is created. The operator never bears unrecoverable compute cost.

2. **TTL accuracy +/-60 seconds**: Turso-authoritative TTL with poll worker + machine-local failsafe cron. Two-layer enforcement ensures no zombie machine runs more than ~90 seconds past expiry.

3. **No raw keys on VMs**: In Delegation mode, only an ephemeral session key exists in process memory. In Embedded mode, Privy's TEE holds all signing keys. In LocalKey mode, the keypair is encrypted at rest and bounded by on-chain delegation caveats.

---

## Design principles

### 1. Operator cost protection

The operator (Bardo) never bears unrecoverable compute cost. Every VM-hour is pre-paid via x402. Cost protection controls (per-region caps, global caps, zombie detection) prevent runaway spend under adversarial conditions.

### 2. User correctness

TTL is derived from payment amount, never user-specified. The system computes exact seconds from USDC paid at the machine's hourly rate. No rounding errors, no timezone confusion.

### 3. x402-native

All resource consumption crosses an x402 payment boundary. Provisioning, extensions, and Styx queries are x402-gated. No API keys, no subscription tiers, no invoices.

### 4. Permissionless extensions

Anyone can extend any Golem's TTL by sending an x402 payment. No authentication required. Payer type is tracked for attribution (`owner`, `self`, `external_user`). This enables self-sustaining Golems that earn trading revenue and extend themselves -- closing the metabolic loop described in [03-billing.md](03-billing.md).

### 5. Mortality as architecture (moat: Die)

Not a feature. A design principle. Every other system -- memory, safety, inference routing, custody, coordination -- uses the mortality signal to make better decisions. Mortality is the structural moat: agents that can die make fundamentally different (and better) decisions than immortal ones. See the section above and `moat/prd2-moat-agents-that-die.md`.

---

## Styx connection

Every Golem, regardless of deployment path, maintains a persistent outbound WebSocket to Styx (`wss://styx.bardo.run/v1/styx/ws`). The connection carries:

- Clade sync deltas (bidirectional via Styx relay)
- Pheromone field updates (push from Styx)
- Bloodstain notifications (push from Styx)
- Grimoire entry writes (Golem -> Styx)
- Retrieval queries (Golem -> Styx -> results)
- Event stream for TUI (Styx -> Golem -> TUI)

No inbound ports. No tunnels. No port forwarding. A Raspberry Pi behind double-NAT works identically to a Fly.io VM. Styx is strictly additive -- a Golem operates at ~95% capability without it.

```
Golem (anywhere) ---- outbound WSS ----> Styx (public, port 443)
                                              |
Golem (anywhere else) ---- outbound WSS ------+

No inbound ports. No tunnels.
```

### Styx backend composition

Styx is a single Rust binary (Axum) exposing 16 services across 6 categories. Its shared data layer uses four stores: Qdrant (vector search), PostgreSQL (structured metadata), Redis (cache and pub/sub), and Cloudflare R2 (blob storage). Styx is self-hostable via `docker compose up` with the published Styx image, or available as the managed instance at `wss://styx.bardo.money`. See [prd2/20-styx/02-infrastructure.md](../20-styx/02-infrastructure.md) for the full service breakdown.

---

## Terminology

| Term | Definition |
| --- | --- |
| **Golem** | An autonomous DeFi agent running as a single Rust binary. Each Golem has a wallet, strategy, knowledge store (Grimoire), and finite lifespan. |
| **Golem-RS** | The Rust runtime (`golem-runtime` crate). 28 extensions across 7 dependency layers, organized as a Cargo workspace. |
| **Grimoire** | The Golem's persistent knowledge store. Episodes (LanceDB), insights/heuristics/warnings (SQLite), PLAYBOOK.md (filesystem). |
| **STRATEGY.md** | Owner-authored strategy file. Entry/exit criteria, risk parameters, target protocols, asset allocation. Hot-swappable without reboot. |
| **PLAYBOOK.md** | Machine-evolved heuristics file. Written only by Dream Integration. Contains procedural knowledge distilled from experience. Read-only to the owner. |
| **Heartbeat** | The 9-step autonomous decision cycle. Observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect. Driven by the Adaptive Clock's three timescales (gamma 5-15s, theta 30-120s, delta 5-30min). |
| **Clade** | An owner's fleet of Golems. Siblings share knowledge via structured pipeline with confidence discounts. |
| **Thanatopsis** | The four-phase death protocol: Acceptance, Settlement, Reflection, Legacy. |
| **Styx** | The public knowledge ecology service. Three privacy layers: Vault (private), Clade (shared), Lethe (anonymized). |
| **Owner** | The human who creates, funds, configures, and watches a Golem. |
| **Operator** | The infrastructure provider running Bardo Compute (the managed deployment path). |

---

## Machine naming

Machine names use cryptographic randomness: `golem-{nanoid(12)}`. Names are not derivable from user ID, strategy type, or any other input. This prevents enumeration attacks on the permissionless extend endpoint. Names are always unique -- re-deploys create new machines with new names (strategy, Grimoire, and wallet config carry over).

The `nanoid` alphabet (A-Za-z0-9\_-) produces 64^12 = 4.7 x 10^21 possible names. At 1M Golems, collision probability is negligible.

---

## MVP scope

### Included

- Full x402 payment saga (pessimistic lock-then-confirm)
- VM lifecycle: provisioning, extension, graceful destruction
- 2-app Fly topology (`bardo-control` monolith + `bardo-machines`)
- Two-layer TTL enforcement (Turso poll worker + machine-local failsafe)
- Warm machine pool (sub-5s provisioning)
- Three-mode custody provisioning (Delegation, Embedded, LocalKey)
- Styx connection setup during provisioning
- Mandatory ERC-8004 registration
- Fly config injection (no SSH bootstrapping)
- Fly OIDC machine authentication
- SSH Certificate Authority (`step-ca`)
- WebSocket SSH with ticket-based auth
- Privy JWT admin auth with identity tracking
- Two-tier VM endpoints (public `:3000`, auth-gated `:3001`)
- Subdomain proxy routing (`<name>.bardo.money`)
- Permissionless extensions with rate limiting
- Agent discovery (`GET /v1/golems`) + tool exposure
- Periodic Grimoire sync (every 6 hours)
- TUI as primary interactive frontend
- Web dashboard as secondary management surface
- Security: threat model, rate limiting matrix, audit logging
- Operations: multi-region HA, monitoring, alerting, structured logging
- Complete database schema + REST API reference
- Inference billing integration (base allowance + overage per VM tier)

### Excluded (phase 2)

| Feature | Rationale |
| --- | --- |
| A2A protocol (ERC-8001) | Specification immature; direct tool calls sufficient for v1 |
| Batch provisioning | Single-golem deploy covers all v1 use cases |
| Golem hibernation | Adds state management complexity; destroy + redeploy is simpler |
| Extension spending caps | Per-user daily limits; not needed when all payments are permissionless |
| Automatic VM resize | Requires live migration; redeploy is acceptable |
| Operator profitability dashboard | Metrics available via admin endpoint; dedicated UI deferred |
| Marketplace commerce flow | Data model for knowledge marketplace specified; purchase/settlement in v2 |

---

## Pricing tiers

All prices in micro-USDC. Minimum purchase is 1 hour.

| VM size | Fly config | Price/hr (micro-USDC) | Min purchase (1h) | Typical use |
| --- | --- | --- | --- | --- |
| `micro` | shared-cpu-1x / 256MB | 25,000 ($0.025) | 25,000 | Simple monitors, keepers |
| `small` | shared-cpu-1x / 512MB | 50,000 ($0.05) | 50,000 | Standard Golem (default) |
| `medium` | shared-cpu-2x / 1GB | 100,000 ($0.10) | 100,000 | Multi-tool, Grimoire-heavy |
| `large` | performance-2x / 2GB | 200,000 ($0.20) | 200,000 | Simulation, orchestration |

Each tier includes a base inference token allowance (see [03-billing.md](03-billing.md)). Overages bill from the Golem's wallet at published rates through the inference gateway.

---

## Cross-references

| Topic | Document |
| --- | --- |
| System architecture | [01-architecture.md](01-architecture.md) — Two-app Fly.io topology (bardo-control + bardo-machines), host header routing, subdomain proxy, and the two-tier VM endpoint model. |
| VM provisioning and lifecycle | [02-provisioning.md](02-provisioning.md) — Warm pool mechanics, Docker image structure, entrypoint script, golem.toml config injection, machine lifecycle states, and graceful shutdown via Thanatopsis integration. |
| x402 billing, self-funding, and TTL | [03-billing.md](03-billing.md) — EIP-3009 payment flow, pricing tiers, the metabolic self-funding loop, inference billing integration, TTL calculation, and two-layer TTL enforcement. |
| Security and threat model | [04-security.md](04-security.md) — VM isolation, three-mode custody security details, attacker taxonomy, top 5 threats with mitigations, authentication contexts, and rate limiting matrix. |
| Operations and monitoring | [05-operations.md](05-operations.md) — Multi-region deployment, TTL worker high availability, Styx health monitoring, reconciliation job, OpenTelemetry metrics, and failure mode matrix. |
| API reference | [06-api.md](06-api.md) — Full REST API specification, Turso database schema (6+ tables), GolemEvent types enum, and HTTP error codes. |
| TUI and web dashboard | [07-frontend.md](07-frontend.md) — Web dashboard pages (6 pages under /compute), GolemCard component, deploy wizard, custody mode selector, and design system tokens. |
| Golem runtime | [prd2/01-golem/00-overview.md](../01-golem/00-overview.md) — The Golem-RS runtime: 28 extensions across 7 dependency layers, the Heartbeat pipeline, CorticalState shared perception surface, and the Extension trait lifecycle. |
| Golem container three-process structure (golem-binary, hermes-agent, sanctum-ts; sidecar minimization policy) | [prd2/01-golem/00-overview.md](../01-golem/00-overview.md) — Covers the three-process VM architecture and the policy that minimizes sidecar dependencies. |
| Inference gateway | [prd2/12-inference/00-overview.md](../12-inference/00-overview.md) — x402-gated multi-provider LLM gateway: three-tier routing (T0 suppress, T1 analyze, T2 deliberate), provider integrations, and cost management. |
| Inference deployment modes (embedded, local, remote) | [prd2/12-inference/01-deployment-modes.md](../12-inference/01-deployment-modes.md) — Details the three ways inference runs: embedded in the binary, via a local server, or through the remote gateway. |
| Mortality thesis | [prd2/02-mortality/01-architecture.md](../02-mortality/01-architecture.md) — The philosophical and architectural argument for agent mortality: three death clocks, VitalityState composition, behavioral phases, and the economic death clock. |
| Grimoire architecture | [prd2/04-memory/01-grimoire.md](../04-memory/01-grimoire.md) — LanceDB + SQLite + filesystem knowledge store: episode embeddings, semantic entries, PLAYBOOK.md heuristics, four-factor retrieval, Curator pruning, and Ebbinghaus decay. |
| Styx architecture | [prd2/20-styx/00-architecture.md](../20-styx/00-architecture.md) — Global knowledge relay design: three privacy tiers (Vault, Clade, Lethe), WebSocket protocol, pheromone field, and Bloodstain death warnings. |
| Styx infrastructure (16 services, 6 categories, data stores) | [prd2/20-styx/02-infrastructure.md](../20-styx/02-infrastructure.md) — Operational details: Qdrant vector search, PostgreSQL metadata, Redis pub/sub, Cloudflare R2 blob storage, and the 16-service breakdown. |
| Custody modes | [prd2/10-safety/01-custody.md](../10-safety/01-custody.md) — Three wallet custody modes (Delegation via ERC-7710/7715, Embedded via Privy TEE, LocalKey for dev), seven caveat enforcers, session key lifecycle, and death settlement. |
