# Golem Deployment: Managed Compute, Self-Deploy, and Bare Metal [SPEC]

> **PRD2 Section**: 20-styx | **Source**: Styx Research S7 v4.0

> **Status**: Implementation Specification
>
> **Crates**: `bardo-terminal` (CLI deploy commands), `bardo-styx` (compute provisioning endpoints)
>
> **Dependencies**: `prd2/20-styx/03-clade-sync.md` (outbound WebSocket model), `prd2/01-golem/07-provisioning.md` (Golem provisioning), `prd2/09-economy/00-identity.md` (ERC-8004 identity)

> **Reader orientation:** This document specifies the three deployment paths for Golems (mortal autonomous agents compiled as single Rust binaries running on micro VMs): Bardo Compute (managed Fly.io VMs paid via x402), self-deploy helper (user's own infrastructure), and manual bare metal. It belongs to the Styx/deployment layer of Bardo. The key invariant is that all paths produce an identical Golem binary connecting outbound to Styx (global knowledge relay and persistence layer at wss://styx.bardo.run) with a mandatory ERC-8004 (on-chain agent identity standard) registration. See `prd2/shared/glossary.md` for full term definitions.

---

## What This Document Covers

A Golem is a single Rust binary that runs on any Linux machine (and macOS/Windows for development). This document specifies the three deployment paths, the mandatory ERC-8004 identity registration, how the outbound WebSocket model makes all paths equivalent from the ecosystem's perspective, TUI connection patterns, cost comparison, and graceful degradation.

---

## 1. The Deployment Invariant

Regardless of HOW a Golem is deployed, it:

1. **Runs the same binary** (`golem-binary`, compiled from the `bardo-golem-rs` workspace)
2. **Connects outbound to Styx** via persistent WebSocket (zero inbound ports required)
3. **Registers an ERC-8004 identity** on Base L2 (mandatory, happens during onboarding)
4. **Pays for Styx services** via x402 micropayments from its Privy wallet
5. **Is mortal** -- it dies when credits run out, knowledge decays, or a stochastic event fires (see `prd2/02-mortality/`)

The deployment method only determines WHO pays for the compute and HOW the binary gets onto the machine. Everything else -- the cognitive architecture, the knowledge system, the sync protocol, the engagement surfaces -- is identical.

---

## 2. Three Deployment Paths

### Path A: Bardo Compute (Managed)

**For**: Users who want zero infrastructure management. Pay x402, get a running Golem.

Bardo Compute is a managed VM provisioning service that deploys Golem binaries on Fly.io infrastructure. The service provisions, monitors, and destroys VMs. Users pay an x402 markup that covers Fly.io costs plus margin.

| VM Tier | Fly.io Config | x402 Price/Hour | Fly.io Cost/Hour | Margin | Typical Use |
|---------|--------------|----------------|-----------------|--------|-------------|
| micro | shared-cpu-1x / 256MB | $0.035 | $0.004 | ~$0.031 | Simple monitors, keepers |
| small | shared-cpu-1x / 512MB | $0.065 | $0.008 | ~$0.057 | Standard Golem (default) |
| medium | shared-cpu-2x / 1GB | $0.12 | $0.015 | ~$0.105 | Multi-strategy, heavy dreams |
| large | shared-cpu-4x / 2GB | $0.22 | $0.030 | ~$0.190 | Full observatory + trading |

**Key properties**:
- **Payment-before-provision**: x402 payment verified before any VM is created. The service never bears unrecoverable compute cost.
- **TTL from payment**: Lifespan is computed from USDC paid at the tier's hourly rate. No subscriptions.
- **Permissionless extensions**: Anyone can extend any Golem's TTL with an x402 payment. This enables self-sustaining Golems that earn revenue and extend themselves.
- **Sub-5s provisioning**: Warm pool of pre-provisioned VMs claimed on payment. Cold fallback: 15-30s.
- **No raw keys**: Privy server wallet API for all signing. The VM holds only a session signer.

**Provisioning flow**:

```
User (TUI or web) -> POST /v1/compute/provision { tier, strategy, x402_payment }
  -> Verify x402 payment (sufficient for >=1 hour at tier rate)
  -> Claim warm VM from pool (or provision cold)
  -> Inject config: golem.toml, STRATEGY.md, Privy wallet ID
  -> Register ERC-8004 identity (on-chain, gas paid from Golem wallet)
  -> Golem binary starts -> connects to Styx -> first heartbeat
  -> Return { golem_id, status: "alive", ttl_seconds, styx_connected: true }
```

### Path B: Self-Deploy Helper (Automated)

**For**: Users who want to control their own infrastructure but don't want to manually configure everything. They bring their own Fly.io, Railway, or VPS account.

The `bardo-terminal` CLI includes a `deploy` command that automates the setup:

```bash
# Deploy to user's own Fly.io account
bardo deploy --provider fly --app-name my-golem --region iad

# Deploy to user's own Railway project
bardo deploy --provider railway --project my-golem

# Deploy to a VPS via SSH
bardo deploy --provider ssh --host 203.0.113.42 --user root

# Deploy to a local Docker container
bardo deploy --provider docker --name my-golem
```

**What the helper does**:
1. Authenticates with the provider (Fly.io CLI, Railway CLI, SSH key)
2. Creates the deployment target (Fly app, Railway service, Docker container)
3. Copies the Golem binary and configuration files
4. Generates `golem.toml` from user's strategy and wallet
5. Registers the ERC-8004 identity (gas paid from Golem wallet)
6. Starts the Golem, connects to Styx, first heartbeat
7. Outputs connection info and TUI connection command

**The user pays**: their own provider directly (Fly.io bill, VPS monthly cost, electricity). They also pay x402 to Styx for sync, pheromone, and retrieval services. No compute markup -- but they handle their own uptime.

### Path C: Manual / Bare Metal / Home Hardware

**For**: Advanced users, developers, people running on a Raspberry Pi or home server.

The Golem binary is a single static executable. Download it, configure it, run it.

```bash
# Download the binary
curl -sSL https://get.bardo.run/golem | sh

# Or build from source
git clone https://github.com/bardo-protocol/golem-rs
cd golem-rs && cargo build --release

# Configure
bardo init  # Interactive wizard: wallet, strategy, Styx connection

# Run
./golem-binary --config golem.toml
```

**What the user handles**: Everything -- hardware, OS, networking, uptime, backups. The Golem connects outbound to Styx (zero inbound ports), so NAT/firewall is not an issue.

**What `bardo init` does**:
1. Wallet setup (create Privy embedded wallet, or connect existing)
2. Strategy authoring (freeform or template)
3. Fund check (USDC balance on Base)
4. ERC-8004 identity registration (mandatory)
5. Generate `golem.toml` with Styx connection config
6. Test connection to Styx (verify WebSocket connectivity)

---

## 3. Mandatory ERC-8004 Identity Registration

Every Golem MUST register an ERC-8004 on-chain identity before it can connect to Styx. This is non-negotiable -- it's the identity layer that enables:

- Clade discovery (Styx groups Golems by `user_id` from ERC-8004)
- Reputation scoring (ERC-8004 score gates Lethe (formerly Commons) access at >= 50; see `prd2/09-economy/01-reputation.md`)
- Bloodstain provenance (death warnings are EIP-712 signed against ERC-8004 identity)
- Marketplace seller reputation (see `prd2/20-styx/04-marketplace.md`)
- Achievement tracking across generations

### Registration Data

```solidity
// On-chain (Base L2) -- stored in the ERC-8004 registry contract
struct AgentRegistration {
    address walletAddress;       // Privy server wallet address
    address ownerAddress;        // User's wallet (for clade grouping)
    uint256 generation;          // 0 for first-gen, N for Nth successor
    address parentAgent;         // Previous generation's address (0x0 for first-gen)
    string deploymentType;       // "managed" | "self-deployed" | "bare-metal"
    uint256 registeredAt;        // Block timestamp
}
```

### Registration Flow

Registration happens during onboarding (TUI `bardo init`, web deploy wizard, or Bardo Compute provisioning). The gas cost (~$0.01-0.05 on Base L2) is paid from the Golem's Privy wallet. The transaction is:

```
GolemWallet -> ERC8004Registry.register(metadata) -> tx confirmed -> Golem can connect to Styx
```

Styx verifies ERC-8004 registration during WebSocket authentication. An unregistered Golem is rejected. This prevents unregistered agents from consuming Styx services without being accountable to the reputation system.

> **Custody alternatives**: Golems may use MetaMask Delegation (ERC-7710/7715) as an alternative to Privy embedded wallets. See `prd2/11-compute/` for custody mode details.

---

## 4. The Outbound Connection Model

Regardless of deployment path, the Golem's relationship to the ecosystem is identical:

```
+---------------------------+
| Golem (anywhere)          |
|                           |---- outbound WSS ----> Styx (public)
| - Local Grimoire          |      port 443            |
| - Heartbeat loop          |      (standard HTTPS)    |
| - Privy wallet (API)      |                          |
| - ERC-8004 identity       |                          |
+---------------------------+                          |
                                                       |
+---------------------------+                          |
| Golem (anywhere else)     |---- outbound WSS --------+
+---------------------------+

No inbound ports. No tunnels. No port forwarding.
A Raspberry Pi behind double-NAT works identically to a Fly.io VM.
```

**What flows on the connection** (see `prd2/20-styx/03-clade-sync.md`):
- Clade sync deltas (bidirectional via Styx relay)
- Pheromone field updates (push from Styx)
- Bloodstain notifications (push from Styx)
- Entry writes (Golem -> Styx)
- Retrieval queries (Golem -> Styx -> results)
- Event stream for TUI (Styx -> Golem -> TUI)
- Steers and followUps (TUI -> Golem -> Styx -> other surfaces)

---

## 5. TUI Connection Patterns

The TUI (`bardo-terminal`) connects to both the Golem and Styx. The connection pattern depends on the deployment path:

**For managed (Bardo Compute) Golems**: The TUI connects to the Golem's public WebSocket endpoint (`wss://<golem-name>.bardo.run/ws`) and to Styx (`wss://styx.bardo.run/v1/styx/ws`).

**For self-hosted Golems**: The TUI connects to the Golem locally (`ws://localhost:8080/ws`) or via whatever address the user configured, and to Styx.

**For remote self-hosted Golems**: If the Golem is on a remote VPS without a public WebSocket endpoint, the TUI can connect to the Golem's event stream THROUGH Styx -- because the Golem pushes all its Event Fabric events to Styx, and the TUI subscribes to those events via the Styx WebSocket. This means the TUI works from anywhere, even if the Golem is behind NAT:

```
TUI <--- WSS ---- Styx <--- WSS ---- Golem (behind NAT)
         |                              |
         |  Events flow: Golem -> Styx -> TUI
         |  Steers flow: TUI -> Styx -> Golem
```

See `prd2/20-styx/05-tui-experience.md` for the full TUI specification.

---

## 6. Cost Comparison

| Deployment Path | Compute Cost | Styx Cost | Total (small tier, 30 days) |
|----------------|-------------|-----------|---------------------------|
| **Bardo Compute** | ~$46.80/mo (small tier x 24/7) | ~$15/mo (sync + queries) | ~$62/mo |
| **Self-deploy Fly.io** | ~$5.76/mo (shared-cpu-1x) | ~$15/mo | ~$21/mo |
| **VPS (Hetzner)** | ~$4/mo (CX22) | ~$15/mo | ~$19/mo |
| **Home hardware** | ~$3/mo (electricity) | ~$15/mo | ~$18/mo |

Bardo Compute is 3x more expensive but requires zero infrastructure management. The x402 markup IS the product -- users pay for convenience, not just compute. The Styx cost is the same regardless of deployment path, since every Golem uses the same sync, pheromone, and retrieval services.

---

## 7. Graceful Degradation by Deployment Path

| Scenario | Bardo Compute | Self-Hosted | Bare Metal |
|----------|--------------|-------------|-----------|
| Styx offline | Golem continues locally at ~95% | Same | Same |
| Golem crashes | Auto-restart via Fly.io health checks | User must monitor | User must monitor |
| Credits exhausted | Golem executes Thanatopsis -> dies | Same (if economic clock runs out) | Same |
| Network outage | Styx connection drops, reconnects on recovery | Same | Same |
| Hardware failure | Fly.io restarts on new hardware | User handles | User handles |

The Golem binary handles all degradation internally (see `prd2/20-styx/00-architecture.md` for the degradation table). The deployment path only affects who monitors the host. The Golem's cognitive architecture, mortality system, and knowledge persistence work identically regardless of where the binary runs.

---

## References

- [ERC-8004-2025] De Rossi, M. et al. "ERC-8004: Trustless Agents." Ethereum Improvement Proposals, 2025. — *On-chain agent identity standard; mandatory registration for all Golems before Styx connection.*
- [FLY-IO] Fly.io. "Run Your Full Stack Apps Globally." https://fly.io — *Firecracker microVM platform used for Bardo Compute managed deployments.*

---

*The Golem doesn't care where it runs. A Raspberry Pi, a Fly.io VM, a Hetzner box -- it connects outbound to Styx and joins the ecosystem. The deployment is invisible. The intelligence is universal.*
