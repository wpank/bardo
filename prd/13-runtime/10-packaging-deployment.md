# Packaging and deployment [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document specifies how a Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) is packaged, deployed, and connected to the network. It covers the single-binary model, four deployment surfaces, three compute targets (Bardo Compute (the managed hosting platform on Fly.io), self-hosted, local), and the six-stage provisioning pipeline. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: the onboarding flow from `07-onboarding.md` and custody modes from `03-auth-access-control.md`. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

The Golem is a single Rust binary (`golem-binary`, compiled from the `bardo-golem-rs` workspace). Four deployment surfaces (CLI, TUI wizard, web portal, social connectors), three compute targets (Bardo Compute, self-hosted, local), a unified six-stage pipeline, mandatory ERC-8004 (on-chain agent identity standard) registration, and outbound-only WebSocket connectivity.

> **Cross-references:**
>
> - `./07-onboarding.md` — end-to-end onboarding journey from first touch through authentication, custody selection, and first heartbeat
> - `./09-observability.md` — health checks, Prometheus metrics, structured logging, and OpenTelemetry traces for deployed Golems
> - `../10-safety/02-policy.md` — PolicyCage (on-chain safety constraints) specification and the three custody modes

---

## 1. The deployment invariant

Regardless of how a Golem is deployed, it:

1. **Runs the same binary** -- `golem-binary`, compiled from `bardo-golem-rs`
2. **Connects outbound to Styx** (the global knowledge relay and persistence layer at wss://styx.bardo.run) via persistent WebSocket (zero inbound ports required)
3. **Registers an ERC-8004 identity** on Base L2 (mandatory, happens during onboarding)
4. **Pays for Styx services** via x402 (micropayment protocol via signed USDC transfers) micropayments from its wallet
5. **Is mortal** -- it dies when credits run out, knowledge decays, or a stochastic event fires

The deployment method determines WHO pays for compute and HOW the binary gets onto the machine. Everything else -- the cognitive architecture, the knowledge system, the sync protocol, the engagement surfaces -- is identical.

---

## 2. Four deployment surfaces

### Surface A: CLI (`bardo create` / `bardo deploy`)

For technical users who want command-line control.

```bash
# Create and deploy to Bardo Compute
bardo create --strategy ./STRATEGY.md --tier small

# Deploy to user's own Fly.io
bardo deploy --provider fly --app-name my-golem --region iad

# Deploy to user's own Railway
bardo deploy --provider railway --project my-golem

# Deploy to a VPS via SSH
bardo deploy --provider ssh --host 203.0.113.42 --user root

# Deploy to a local Docker container
bardo deploy --provider docker --name my-golem
```

### Surface B: TUI wizard

Interactive terminal wizard with the full onboarding flow from `07-onboarding.md`. Progressive sprite generation, compute tier selection, strategy templates, and first-heartbeat cinematic. The wizard produces the same artifacts as the CLI.

### Surface C: Web portal

Web dashboard at `app.bardo.run`. The guided creation wizard (see `07-onboarding.md`) is the primary onboarding path for new users. The portal produces the same artifacts as CLI and TUI: a `golem.toml`, a `STRATEGY.md`, and wallet credentials.

### Surface D: Social connectors (Telegram, Discord, Twitter/X, Farcaster)

Social platforms are a deployment surface. Users issue natural-language commands in chat; the Bott gateway parses them into the same six-stage pipeline that CLI and TUI use.

```
User (Telegram): @bardo farm morpho $5000

Bott: Deploying: Morpho USDC Yield Optimizer
      Wallet: 0x7a2f...3e1b (existing)
      Cost: $0.065/hr compute + gas
      Strategy: Conservative

      [Deploy] [Customize] [Cancel]
```

The pipeline for social deployment:

```
Stage 1: Parse
  |   Bott NL classifier extracts intent from chat message.
  |   Example: "farm morpho $5000" -> { strategy: yield-optimizer, protocol: morpho, amount: 5000 }
  v
Stage 2: Template match
  |   Identical to CLI/TUI. Maps parsed intent to STRATEGY.md.
  v
Stage 3: Authenticate
  |   User's Privy identity resolved from their linked social account.
  |   First-time users are redirected to app.bardo.run/create for wallet setup.
  v
Stage 4: Confirm
  |   Inline buttons (Telegram) or reactions (Discord) for Deploy / Customize / Cancel.
  v
Stage 5: Provision
  |   Same Bardo Compute provisioning. x402 payment from the user's wallet.
  v
Stage 6: Connect
  |   Golem starts. Events stream back to the social connector as
  |   embeds (Discord), formatted messages (Telegram), or quote-tweets (Twitter/X).
  |   The user can also attach a TUI with `bardo connect --session {token}`.
```

Social connector specifics:

| Platform | Input format | Output format | Auth model | Character limit |
|---|---|---|---|---|
| Telegram | Bot commands or NL messages | Formatted text + inline buttons | Linked Privy account | 4096 chars |
| Discord | Slash commands or NL in bot channel | Rich embeds + button components | Linked Privy account | 6000 chars (embed) |
| Twitter/X | @mention with NL | Quote-tweet with status card | Linked Privy account | 280 chars (reply) |
| Farcaster | Cast with @bardo tag | Reply cast with frame | Linked Privy account (via SIWE) | 1024 chars |

Ongoing event relay: the Golem pushes lifecycle events (phase transitions, trades, dreams, death) through the social connector. Events are formatted as platform-native rich content (Telegram messages, Discord embeds, Twitter cards). Rate-limited to 1 message per 15 minutes per channel during normal operation; critical events (death, kill-switch) bypass the rate limit.

> **Cross-reference:** `./02-communication-channels.md` (per-channel format specs)

---

## 3. Three compute targets

### Target A: Bardo Compute (managed)

Managed VM provisioning on Fly.io infrastructure. Payment-before-provision via x402.

| VM tier | Fly.io config | x402 price/hour | Fly.io cost/hour | Margin | Typical use |
|---|---|---|---|---|---|
| micro | shared-cpu-1x / 256MB | $0.035 | $0.004 | ~$0.031 | Simple monitors, keepers |
| small | shared-cpu-1x / 512MB | $0.065 | $0.008 | ~$0.057 | Standard Golem (default) |
| medium | shared-cpu-2x / 1GB | $0.12 | $0.015 | ~$0.105 | Multi-strategy, heavy dreams |
| large | shared-cpu-4x / 2GB | $0.22 | $0.030 | ~$0.190 | Full observatory + trading |

Properties:
- **TTL from payment**: Lifespan computed from USDC paid at the tier's hourly rate. No subscriptions.
- **Permissionless extensions**: Anyone can extend any Golem's TTL with an x402 payment.
- **Sub-5s provisioning**: Warm pool of pre-provisioned VMs. Cold fallback: 15-30s.
- **No raw keys**: Custody-mode signing (Delegation via MetaMask, Embedded via Privy, or LocalKey). The VM holds only a session signer.

### Target B: Self-hosted (Fly.io, Railway, VPS, Docker)

The user brings their own infrastructure. The `bardo deploy` command automates setup:

1. Authenticates with the provider (Fly.io CLI, Railway CLI, SSH key)
2. Creates the deployment target (Fly app, Railway service, Docker container)
3. Copies the Golem binary and configuration
4. Generates `golem.toml` from user's strategy and wallet
5. Registers ERC-8004 identity (gas paid from Golem wallet)
6. Starts the Golem, connects to Styx, first heartbeat
7. Outputs connection info and TUI connection command

The user pays their own provider directly. They also pay x402 to Styx for sync, retrieval, and pheromone services. No compute markup.

### Target C: Manual / bare metal

The binary is a single static executable. Download, configure, run.

```bash
# Download the binary
curl -sSL https://get.bardo.run/golem | sh

# Or build from source
git clone https://github.com/bardo-protocol/golem-rs
cd golem-rs && cargo build --release

# Interactive wizard: wallet, strategy, Styx connection
bardo init

# Run
./golem-binary --config golem.toml
```

User handles everything: hardware, OS, networking, uptime, backups. The outbound WebSocket model means NAT/firewall is not an issue.

---

## 4. Unified six-stage pipeline

All four surfaces and all three compute targets converge on the same six-stage pipeline. The stages differ only in how input arrives and where the binary lands.

```
Stage 1: Parse and classify
  |   Extract intent, protocol, amount, risk parameters.
  |   CLI: from flags. TUI: from wizard. Web: from form. Social: from NL parsing.
  v
Stage 2: Template match
  |   Map parsed intent to a strategy template.
  |   Generate STRATEGY.md with resolved parameters.
  v
Stage 3: Authenticate
  |   Privy wallet provisioning (new user) or retrieval (returning).
  |   Custody mode setup (Delegation, Privy embedded, local key).
  v
Stage 4: Confirm
  |   Present strategy summary, cost breakdown, funding recommendation.
  |   CLI: --yes flag skips. TUI: type SUMMON. Web: click Summon. Social: tap Deploy button.
  v
Stage 5: Provision
  |   x402 payment verification (Bardo Compute) or provider setup (self-hosted).
  |   VM creation, config injection, ERC-8004 registration.
  |   Sprite resolution stages run in parallel with provisioning.
  v
Stage 6: Connect
      Golem binary starts.
      Outbound WebSocket to Styx.
      First heartbeat tick.
      TUI / web portal / social connector receives event stream.
```

---

## 5. Mandatory ERC-8004 identity registration

Every Golem must register an ERC-8004 identity on Base L2 before connecting to Styx. The registration happens in Stage 5 of the pipeline.

```solidity
struct AgentRegistration {
    address walletAddress;       // Golem's wallet
    address ownerAddress;        // Owner's wallet (for Clade grouping)
    uint256 generation;          // 0 for first-gen, N for Nth successor
    address parentAgent;         // Previous generation (0x0 for first-gen)
    string deploymentType;       // "managed" | "self-deployed" | "bare-metal"
    uint256 registeredAt;        // Block timestamp
}
```

Styx verifies ERC-8004 registration during WebSocket authentication. Unregistered Golems are rejected.

---

## 6. The outbound connection model

Regardless of deployment path, the Golem's relationship to the ecosystem is identical:

```
+-------------------------+
| Golem (anywhere)        |
|                         |---- outbound WSS ----> Styx (public)
| - Local Grimoire        |      port 443           |
| - Heartbeat loop        |      (standard HTTPS)   |
| - Wallet (API)          |                          |
| - ERC-8004 identity     |                          |
+-------------------------+                          |
                                                     |
+-------------------------+                          |
| Golem (anywhere else)   |---- outbound WSS -------+
+-------------------------+

No inbound ports. No tunnels. No port forwarding.
A Raspberry Pi behind double-NAT works identically to a Fly.io VM.
```

What flows on the connection:
- Clade sync deltas (bidirectional via Styx relay)
- Pheromone field updates (push from Styx)
- Bloodstain notifications (push from Styx)
- Entry writes (Golem to Styx)
- Retrieval queries (Golem to Styx to results)
- Event stream for TUI (Styx to Golem to TUI)
- Steers and followUps (TUI to Golem to Styx to other surfaces)

### TUI connection paths

**Managed (Bardo Compute)**: TUI connects to `wss://<golem-name>.bardo.run/ws` and to `wss://styx.bardo.run/v1/styx/ws`.

**Self-hosted (local)**: TUI connects to `ws://localhost:8080/ws` and to Styx.

**Self-hosted (remote, behind NAT)**: TUI connects to the Golem's event stream THROUGH Styx -- the Golem pushes events to Styx, and the TUI subscribes via the Styx WebSocket:

```
TUI <-- WSS ---- Styx <-- WSS ---- Golem (behind NAT)
         |                            |
         |  Events flow: Golem -> Styx -> TUI
         |  Steers flow: TUI -> Styx -> Golem
```

---

## 7. Cost comparison

| Deployment path | Compute cost | Styx cost | Total (small tier, 30 days) |
|---|---|---|---|
| **Bardo Compute** | ~$46.80/mo (small tier, 24/7) | ~$15/mo (sync + queries) | ~$62/mo |
| **Self-deploy Fly.io** | ~$5.76/mo (shared-cpu-1x) | ~$15/mo | ~$21/mo |
| **VPS (Hetzner)** | ~$4/mo (CX22) | ~$15/mo | ~$19/mo |
| **Home hardware** | ~$3/mo (electricity) | ~$15/mo | ~$18/mo |

Bardo Compute is 3x more expensive but requires zero infrastructure management. The x402 markup is the product.

---

## 8. Graceful degradation by deployment path

| Scenario | Bardo Compute | Self-hosted | Bare metal |
|---|---|---|---|
| Styx offline | Golem continues locally at ~95% | Same | Same |
| Golem crashes | Auto-restart via Fly.io health checks | User must monitor | User must monitor |
| Credits exhausted | Golem executes Thanatopsis, dies | Same (if economic clock runs out) | Same |
| Network outage | Styx connection drops, reconnects on recovery | Same | Same |
| Hardware failure | Fly.io restarts on new hardware | User handles | User handles |

---

## 9. Configuration schema

### golem.toml

```toml
[golem]
name = "yield-optimizer-alpha"
strategy = "./STRATEGY.md"

[golem.mortality]
hayflick_limit = 100_000             # Max ticks (0 = disabled for self-hosted)
death_reserve_usdc = 0.30

[golem.heartbeat]
base_interval_ms = 40_000
min_interval_ms = 15_000
max_interval_ms = 120_000

[golem.cognition]
default_model = "claude-haiku-4-5"
escalation_model = "claude-sonnet-4"
critical_model = "claude-opus-4-6"
t0_ratio = 0.80
max_daily_inference_usdc = 5.00

[wallet]
custody = "privy"                    # delegation | privy | local

[wallet.privy]
app_id_env = "PRIVY_APP_ID"
app_secret_env = "PRIVY_APP_SECRET"
wallet_id_env = "BARDO_WALLET_ID"

[wallet.policy]
transfer_restriction = "strict"      # strict | clade | unrestricted
max_tx_usdc = 10_000
max_daily_usdc = 100_000

[styx]
url = "wss://styx.bardo.run/v1/ws"
enabled = true

[grimoire]
path = "./data/grimoire"
max_episodes = 50_000

[chains]
default = 8453
rpc_urls = { "8453" = "${BASE_RPC_URL}", "1" = "${ETH_RPC_URL}" }

[logging]
level = "info"
format = "json"

[telemetry]
enabled = true
otel_endpoint = ""
```

### Dev overrides (dev-golem.toml)

```toml
[golem]
name = "dev-golem"
strategy = "./test/fixtures/STRATEGY.md"

[golem.mortality]
hayflick_limit = 0
death_reserve_usdc = 0

[golem.heartbeat]
base_interval_ms = 5_000
min_interval_ms = 2_000

[wallet]
custody = "local"
private_key_env = "BARDO_WALLET_PRIVATE_KEY"

[wallet.policy]
transfer_restriction = "unrestricted"
max_tx_usdc = 100_000_000

[chains]
default = 31337
rpc_urls = { "31337" = "http://localhost:8545" }

[logging]
level = "debug"
format = "pretty"
```

---

## 10. Deployment modes comparison

| Aspect | Bardo Compute | Self-hosted (Docker) | Bare metal | Local dev |
|---|---|---|---|---|
| Provisioning | 6-stage pipeline | `bardo deploy --provider docker` | `bardo init && ./golem-binary` | `bardo dev` |
| TTL enforcement | x402-based | None (runs until stopped) | None | None |
| Hayflick limit | Enforced (default 100K) | Configurable (0 = disabled) | Configurable | Disabled |
| Warm pool | Yes (3-8s boot) | N/A | N/A | N/A |
| Inference | x402 via inference.bardo.run | Direct API keys or gateway | Direct API keys | Gateway |
| Grimoire backup | Styx Archive snapshots (6-hourly) | Operator managed | Operator managed | Local disk |
| Cost model | x402 micropayments | Operator infrastructure | Operator infrastructure | Free |

---

## 11. Network requirements

### Port table (self-hosted)

| Port | Protocol | Direction | Purpose | Required |
|---|---|---|---|---|
| 443 | TCP | Outbound | HTTPS (Styx, RPC, inference, Privy) | Yes |
| 8402 | TCP | Inbound | Golem RPC + WebSocket (TUI connection) | Optional (local TUI only) |
| 9090 | TCP | Inbound | Prometheus metrics | Optional |

The outbound-only model means self-hosted Golems behind NAT work without port forwarding. The TUI connects through Styx when direct access is unavailable.

### Firewall (minimal production)

```bash
# Default deny inbound, allow outbound
ufw default deny incoming
ufw default allow outgoing

# Optional: expose Golem RPC for local TUI
# ufw allow from 127.0.0.1 to any port 8402

# Optional: Prometheus metrics
# ufw allow 9090/tcp

ufw enable
```

---

## 12. CI/CD pipeline

```yaml
name: Build & Release

on:
  push:
    tags: ["v*"]

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-musl
          - aarch64-unknown-linux-musl
          - x86_64-apple-darwin
          - aarch64-apple-darwin
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/golem-binary

  docker:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - uses: docker/build-push-action@v6
        with:
          push: true
          tags: |
            ghcr.io/bardo-protocol/golem:${{ github.ref_name }}
            ghcr.io/bardo-protocol/golem:latest
```

---

_End of document._
