# Onboarding [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document walks through the end-to-end onboarding journey from a user's first touch to their Golem's (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) first Heartbeat (the recurring decision cycle that drives all Golem behavior). It covers three entry surfaces, authentication, custody selection, funding, strategy configuration, compute tier selection, and the first-heartbeat cinematic. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites: custody modes from `03-auth-access-control.md` and the deployment model from `10-packaging-deployment.md`. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

End-to-end journey from first touch to first heartbeat. Three surfaces, ordered by expected user flow: web wizard (primary for new users), TUI (for technical users who prefer the terminal), and CLI (for power users who script). Three custody modes, four strategy templates, progressive sprite generation, and a first-heartbeat cinematic. The goal: a running Golem in under three minutes with zero prior configuration.

> **Cross-references:**
>
> - `prd2/01-golem/06-creation.md` — Golem creation specification: identity generation, wallet provisioning, and initial state construction
> - `prd2/01-golem/07-provisioning.md` — six-stage provisioning pipeline from config validation through VM boot and first heartbeat
> - `./10-packaging-deployment.md` — deployment surfaces (CLI, TUI, web, social) and three compute targets (Bardo Compute, self-hosted, local)
> - `../10-safety/02-policy.md` — PolicyCage (on-chain safety constraints) specification and the three custody modes (Delegation, Embedded, LocalKey)

---

## 1. Surface routing

New users arrive from one of three surfaces. The web wizard at `app.bardo.run` is the default landing; social links, marketing pages, and death recap cards all funnel there. Technical users who install the binary first hit the TUI path. Power users who already have config use the CLI.

### 1.1 Web wizard (primary)

The web wizard at `app.bardo.run/create` handles authentication, custody selection, strategy configuration, and compute tier selection in a single guided flow. On completion, it produces a session token and a `golem.toml` config. The user can then:

- **Stay on web**: watch provisioning and first heartbeat in the Portal
- **Switch to TUI**: run `bardo connect --session {token}` to attach the terminal to the new Golem
- **Do nothing**: the Golem runs headlessly; the user returns whenever they want

The wizard performs the same capability probe as the TUI entry path, but browser-side:

- Existing auth state (Privy session cookie)
- Running Golems (Styx (the global knowledge relay and persistence layer) query by owner address)
- Wallet state (connected wallet, embedded wallet, or neither)

| State | Flow |
|---|---|
| No auth | Full onboarding (this document) |
| Auth, no Golems | Creation flow (skip auth) |
| Auth, running Golems | Portal dashboard with "Create another" option |
| Auth, all Golems dead | Memorial screen (see section 10) |

### 1.2 TUI entry

After the branded ASCII banner renders (~200ms), the terminal runs a fast capability probe (<1 second):

- Terminal size, color depth, Unicode support
- Existing config at `~/.bardo/`
- Network connectivity (Styx reachable?)
- Running Golems (WebSocket ping to known endpoints)

| State | Flow |
|---|---|
| No config | Opens web wizard in browser (same as section 1.1) |
| Config, no Golems | Creation flow (skip auth) |
| Config, running Golems | Connect to TUI main screen |
| Config, all Golems dead | Memorial screen (see section 10) |

For first-time TUI users with no config, the terminal opens `app.bardo.run/create` in the default browser and polls for session completion -- the same browser handoff pattern described in section 2. The TUI does not duplicate the web wizard; it delegates to it.

### 1.3 CLI entry

```bash
bardo create --strategy ./STRATEGY.md --tier small   # Full creation
bardo connect --session {token}                       # Attach to existing
bardo auth login                                      # Auth only
```

The CLI accepts all parameters as flags, skipping interactive prompts entirely. It also accepts a `--session` token from the web wizard for users who started onboarding on the web and want to finish in the terminal.

---

## 2. Authentication

The web wizard owns session generation. All surfaces delegate auth to the browser.

### Standard flow (web wizard)

1. Web wizard at `app.bardo.run/create` generates a cryptographic session ID
2. Privy sign-in widget renders inline: email, Google, GitHub, Twitter, wallet (SIWE)
3. On success, Privy returns JWT + embedded wallet address + wallet ID
4. Session stored server-side; client receives session token
5. If the user launched from TUI, the terminal detects completion via polling `/auth/status?session={id}` every 2 seconds

### TUI browser handoff

When the TUI triggers onboarding (no existing config), it opens the web wizard and polls:

- Auth URL in copyable text
- QR code (for mobile auth)
- Keyboard shortcuts: `[c]` Copy URL, `[q]` Cancel

This mirrors Claude Code and Codex. The terminal does not handle Privy directly.

### Headless fallback

For SSH sessions or environments without a browser:

1. **Device code**: Terminal displays a short code like `BARD-7F3A`. User visits `bardo.run/device`, enters code, authenticates. Terminal detects via polling.
2. **API key**: User provides a key from `bardo.run/settings/api-keys`. Bypasses Privy entirely.
3. **Manual URL copy**: User copies the auth URL and opens it on any device.

### CLI equivalent

```bash
bardo auth login          # Opens web wizard, same polling flow
bardo auth login --device # Device code flow
bardo auth login --key    # API key mode
```

---

## 3. Custody mode selection

After authentication, the user selects how the Golem holds keys. On the web wizard, this is a card-based selection with expandable detail panels. In the TUI (returning users creating additional Golems), it renders as the numbered menu shown below. Three modes, each with different trust and flexibility tradeoffs.

### Mode A: MetaMask Delegation (ERC-7710/7715)

The owner's MetaMask Smart Account delegates scoped permissions to the Golem. On-chain caveat enforcers restrict actions by behavioral phase, spending caps, asset allowlists, and time windows. The owner retains full control; the Golem operates within a provable permission tree.

- Phase-gated enforcers (thriving/stable/conservation/declining/terminal) restrict available actions on-chain
- Sub-delegation enables attenuated permission trees for Replicants
- Owner reviews and grants permissions via MetaMask extension with human-readable descriptions
- Revocable at any time via `bardo wallet revoke-delegation` or the Portal

Best for: owners who want on-chain verifiability of all permission constraints.

### Mode B: Privy embedded wallet

Privy creates a server wallet in a Trusted Execution Environment. P-256 session signers handle low-risk operations. Transfer restrictions (strict/clade/unrestricted) are enforced by Privy's signing policy.

- Key generation happens inside the TEE, never exposed
- Recovery via Privy's recovery flow
- Fastest setup (wallet created in <500ms)

Best for: users who want zero infrastructure management and trust Privy's TEE.

### Mode C: Local key (development only)

A raw private key in `.env`. No signing policy, no transfer restrictions, no TEE.

- For Anvil forks and local testing only
- Not available in production deployment paths

### Selection UI

```
How should your Golem hold its keys?

  [1] MetaMask Delegation (on-chain permissions, verifiable)
  [2] Privy Embedded Wallet (managed, fast setup)
  [3] Local Key (development only)

Choose [1/2/3]:
```

CLI equivalent:

```bash
bardo init --custody delegation
bardo init --custody privy
bardo init --custody local
```

---

## 4. Wallet and funding

After custody selection, the terminal queries the wallet's balances on Base. Displays USDC, ETH, WETH with USD values.

### Funding formula

```
required = (daily_cost x 14 days) x 1.10 + death_reserve

Where:
  daily_cost = inference + gas + data estimates (from strategy template)
  1.10 = 10% buffer
  death_reserve = 0.30 USDC (minimum for Thanatopsis Protocol -- the four-phase structured shutdown)
```

### Funding methods

| Method | Speed | Fees | Notes |
|---|---|---|---|
| Direct USDC transfer | Instant | Gas only | From existing wallet |
| Inline swap | ~30s | Swap fee + gas | Any token to USDC via DEX |
| Bridge | 1-15 min | Bridge fee + gas | Cross-chain via LI.FI |
| Fiat on-ramp | 1-5 min | MoonPay fees (~2.5%) | Credit card to USDC |

If USDC < $25 (absolute floor), the terminal offers each method as a menu option. For MetaMask Delegation custody, the terminal sets up a `PermitBatchTransferFrom` approval with daily spending limits and time bounds. The Golem's wallet is authorized to pull up to `spendingLimit` USDC per day, revocable at any time.

### Proxy spending authorization (Delegation mode)

```
Your Golem needs spending authorization.

  Daily limit: $100 USDC
  Gas allowance: 0.1 ETH/day
  Valid for: 30 days
  Revocable: Yes, at any time

  [Approve in MetaMask] [Adjust limits] [Cancel]
```

---

## 5. Strategy configuration

Three paths, from zero-effort to fully manual.

### 5.1 Conversational path (NL to STRATEGY.md)

The user describes what they want in natural language. A lightweight classifier (Haiku, ~$0.0003, first 3 free) extracts parameters and generates a full STRATEGY.md:

```
Describe your strategy (or press Enter for templates):

> farm morpho $5000

  Parsed:
    Strategy: yield-optimizer
    Protocol: Morpho
    Amount: 5,000 USDC
    Risk: conservative (default)

  Generating STRATEGY.md...
```

When parameters are missing, the system asks a single clarifying question rather than presenting a form:

```
> farm $5000

  Which protocol? [morpho / aave / auto (best available yield)]
```

### 5.2 Template path

Four pre-built strategy templates:

| Template | Trigger phrases | Default parameters | Min funding |
|---|---|---|---|
| **yield-optimizer** | "farm", "yield", "earn" | Auto-select best vault, 6hr rebalance, -5% stop-loss | $50 |
| **dca** | "dca", "dollar cost average", "buy regularly" | Weekly buys, equal splits, market orders | $50 |
| **lp-provision** | "provide liquidity", "LP", "pool" | Auto-range V3, 4hr rebalance, -10% IL limit | $200 |
| **keeper** | "keep", "maintain", "monitor" | Health factor monitoring, auto-liquidation prevention | $50 |

Selecting a template generates a complete STRATEGY.md with sensible defaults. The user can edit before confirming.

### 5.3 Manual path

Write STRATEGY.md directly in your editor:

```bash
bardo init --strategy ./my-strategy.md
```

### 5.4 Generated STRATEGY.md structure

```markdown
# Strategy: Morpho USDC Yield Optimizer

## Objective
Maximize yield on USDC by deploying to Morpho vaults on Base.

## Parameters
- Initial deposit: $5,000 USDC
- Rebalance interval: 6 hours
- Min APY threshold: 3%
- Stop-loss: -5% drawdown from high-water mark

## Risk limits
- Max single vault allocation: 80%
- Emergency exit: Withdraw all on -5% drawdown

## Approved protocols
- Morpho (Base)

## Approved assets
- USDC (Base: 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913)
```

---

## 6. Compute tier selection

The user picks where their Golem runs. Three compute targets, each with different cost and control characteristics.

### Bardo Compute (managed)

| VM tier | x402 (micropayment protocol via signed USDC transfers) price/hour | Typical use |
|---|---|---|
| micro | $0.035 | Simple monitors, keepers |
| small | $0.065 | Standard Golem (default) |
| medium | $0.12 | Multi-strategy, heavy dreams |
| large | $0.22 | Full observatory + trading |

Payment-before-provision via x402. Lifespan computed from USDC paid at the tier's hourly rate. Anyone can extend any Golem's TTL with a payment (enables self-sustaining Golems that earn revenue and extend themselves).

### Self-hosted (Fly.io, Railway, VPS)

The user brings their own infrastructure account. The CLI automates setup:

```bash
bardo deploy --provider fly --app-name my-golem --region iad
bardo deploy --provider railway --project my-golem
bardo deploy --provider ssh --host 203.0.113.42 --user root
bardo deploy --provider docker --name my-golem
```

User pays their own provider directly plus x402 to Styx for sync, retrieval, and pheromone services. No compute markup.

### Local (development)

```bash
bardo dev   # Runs on localhost, Anvil fork, no x402
```

### Selection UI

```
Where should your Golem run?

  [1] Bardo Compute ($0.065/hr, zero setup)      [recommended]
  [2] Self-hosted (bring your own server)
  [3] Local (development only)

Choose [1/2/3]:
```

---

## 7. Provisioning and sprite generation

When the user confirms, provisioning begins. The sprite generation is the key atmospheric element. It runs in parallel with infrastructure setup, making wait time feel intentional.

### Sprite lifecycle during provisioning

A placeholder sprite starts as animated noise (random pixel scatter) in the center of the screen. As each provisioning step completes, the sprite progressively resolves:

| Provisioning step | Sprite stage | Visual |
|---|---|---|
| Strategy compiled | Body outline emerges | Silhouette forms from noise |
| Wallet created | Color palette locks | Colors derived from wallet address hash |
| Identity registered | Face/eyes appear | Features resolve from blur |
| Compute provisioned | Accessories render | Final details appear |
| Heartbeat started | Sprite animates | Eyes open, breathing begins |

### Six-stage pipeline

This pipeline runs identically regardless of surface (TUI, CLI, web) or compute target. The stages correspond to the Bott deployment pipeline:

```
Stage 1: Parse and classify
  Extract intent, protocol, amount, risk parameters
  from the strategy configuration or natural language input

Stage 2: Template match
  Map parsed intent to a strategy template
  Generate STRATEGY.md with resolved parameters

Stage 3: Authenticate
  Verify Privy wallet (existing) or create new (first-time)
  Set up custody mode (delegation, embedded, or local key)

Stage 4: Confirm
  Present strategy summary, cost breakdown, funding recommendation
  Type "SUMMON" to confirm (thematic, short)

Stage 5: Provision
  x402 payment (if Bardo Compute)
  Claim warm VM from pool (sub-5s) or cold provision (15-30s)
  Inject golem.toml, STRATEGY.md, wallet config
  Register ERC-8004 identity on Base L2

Stage 6: Connect
  Golem binary starts, connects outbound to Styx via WebSocket
  First heartbeat tick fires
  TUI connects to event stream
```

### Confirmation screen

```
+-----------------------------------------------------+
| Strategy Review                                      |
+-----------------------------------------------------+
| Name: Morpho USDC Yield Optimizer                    |
| Risk: Conservative                                   |
| Protocol: Morpho Blue                                |
| Assets: USDC                                         |
+-----------------------------------------------------+
| Estimated costs                                      |
|                                                      |
| Inference (LLM):    ~$0.50/day  (80% T0 ticks)      |
| Gas (on-chain):     ~$0.20/day  (low-freq rebalance) |
| Data (RPC/feeds):   ~$0.05/day                       |
| -----------------------------------------------     |
| Total:              ~$0.75/day                       |
|                                                      |
| Recommended funding: $11.55                          |
| (14 days x $0.75 x 1.10 buffer + $0.30 reserve)     |
+-----------------------------------------------------+
|         Type SUMMON to confirm:                      |
+-----------------------------------------------------+
```

---

## 8. ERC-8004 identity registration

Every Golem registers an ERC-8004 on-chain identity during provisioning. This is mandatory for connecting to Styx. Gas cost: ~$0.01-0.05 on Base L2, paid from the Golem's wallet.

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

Identity enables:
- Clade discovery (Styx groups Golems by `ownerAddress`)
- Reputation scoring (gates Styx Lethe (formerly Commons) access at score >= 50)
- Bloodstain provenance (death warnings are EIP-712 signed against identity)
- Achievement tracking across generations

---

## 9. First heartbeat cinematic

The Golem's first heartbeat is a cinematic moment. Not a loading screen -- the real initialization, rendered as theater.

See section 15.2 for the canonical birth cinematic sequence.

### Navigation tutorial

After the birth cinematic completes and the TUI frame materializes, a brief navigation tutorial runs (first launch only, ~15 seconds). The tutorial highlights each window tab in sequence with a 2-second tooltip:

1. **HEARTH** -- "Your Golem's vital signs. Start here."
2. **MIND** -- "How it thinks. Decisions, knowledge, dreams."
3. **SOMA** -- "What it owns. Portfolio, trades, wallet."
4. **WORLD** -- "The ecosystem. Other Golems, your Clade."
5. **FATE** -- "How long it lives. Mortality, lineage, achievements."
6. **COMMAND** -- "Talk to it. Steer, configure, chat."

Each tooltip fades in over 4 frames, holds for 1.5 seconds, fades out over 4 frames, then the next tab highlights. The user can press any key to skip the remaining tooltips. The tutorial state is stored in `~/.bardo/seen_nav_tutorial` and never replays. If the user skips, they can replay it from COMMAND > Config > "Replay navigation tutorial."

### Warm-up period

The first ~10 ticks are T0 only (deterministic, no LLM). The Golem gathers baseline market data, builds initial price history, calibrates probe thresholds. No trading during warm-up.

After warm-up (tick ~11), the first T1 tick fires: Haiku evaluates strategy against market conditions. May execute the first trade. Creates the first Episode. Begins building the Grimoire.

### Boot sequence (internal)

```
VM Start
  |
  +-- 1. Config validation (golem.toml schema)
  |
  +-- 2. Memory restoration
  |     +-- Styx Archive restore: load latest Grimoire bundle
  |     +-- Integrity check: LanceDB + SQLite consistency
  |
  +-- 3. Mortality initialization
  |     +-- Economic baseline from funded USDC balance
  |     +-- Epistemic fitness: 1.0 (fresh) or restore (resuming)
  |     +-- Stochastic seed: keccak256(golemId + creationTick)
  |     +-- Phase determination: computeVitality() -> determinePhase()
  |
  +-- 4. Daimon initialization
  |     +-- Personality baseline: PAD from disposition
  |     +-- Mood state: baseline (fresh) or restore (resuming)
  |
  +-- 5. Dream initialization
  |     +-- Disabled until 50-200 episodes (~4-6h)
  |     +-- Urgency components at 0
  |
  +-- 6. Extension registration
  |     +-- golem-heartbeat, golem-lifespan, golem-daimon,
  |     +-- golem-dream, golem-context, golem-verifier,
  |     +-- golem-reflector, golem-curator, golem-styx,
  |     +-- golem-clade, golem-telemetry, golem-safety
  |
  +-- 7. Health probes: startup probe -> ready
  |
  +-- 8. First heartbeat tick
```

---

## 10. Returning after death

When a user opens the TUI after their Golem has died, the memorial screen replaces the dashboard.

### Memorial screen

The dead Golem's sprite renders in desaturated palette with static artifacts (scan lines, pixel dropout). Below the sprite:

- **Name and generation**: e.g., "eth-dca-alpha (gen 3)"
- **Lifetime stats**: days alive, total ticks, episodes generated, net P&L, insights produced
- **Cause of death**: stated plainly -- "Credit exhaustion", "Epistemic senescence", "Stochastic event", or "Owner terminated"
- **Last words**: the death testament's narrative summary from the Styx Archive archive
- **Grimoire archive status**: whether the full bundle was uploaded, and the archive timestamp

This screen is not dismissible by scrolling. The user must choose.

### Options

1. **Summon Successor** -- enters the inheritance wizard. Pre-fills from predecessor's config. Shows Grimoire inheritance preview: entry count (up to 2,048), confidence decay at 0.85^gen, domain coverage, bloodstain count.

2. **View Death Reflection** -- the complete death testament. Sections: "What I Learned", "What I Got Wrong", "What I Suspect But Cannot Prove", turning points, narrative arc, successor recommendations.

3. **Browse Graveyard** -- Styx death archive browser for this lineage. All dead Golems in chronological order with generation number, lifespan, cause of death, narrative arc.

4. **Start Fresh** -- clean creation wizard, no inheritance.

5. **Explore (no Golem)** -- browse the WORLD window in read-only mode. The user can see the Solaris ocean, active Clades, Lethe entries, and Bloodstains without summoning a Golem. Useful for prospective users who want to understand the ecosystem before committing capital. No wallet or funding required. The TUI renders WORLD tabs only; HEARTH, MIND, SOMA, FATE, and COMMAND are grayed out with a "Summon a Golem to unlock" tooltip on hover/focus.

---

## 11. Lifecycle milestones

### Day 1

| Metric | Expected |
|---|---|
| Episodes | 5-20 |
| Insights | 0-2 |
| Decision cache hit rate | 0% |
| T0 ratio | 60-70% (still learning thresholds) |
| First trade | Within first 50 ticks |

### Day 7

| Metric | Expected |
|---|---|
| Episodes | 200-500 |
| Insights | 10-30 |
| Heuristics | 3-8 |
| Decision cache hit rate | 10-15% |
| T0 ratio | 75-80% |
| PLAYBOOK | 5-10 rules |

### Day 30

| Metric | Expected |
|---|---|
| Episodes | 2,000-5,000 |
| Insights | 50-150 |
| Heuristics | 20-40 |
| Decision cache hit rate | 25-35% |
| T0 ratio | 80-85% |
| PLAYBOOK | Diverging from template |
| Causal graph | 50-100 nodes |

### Day 90

| Metric | Expected |
|---|---|
| Episodes | 8,000-15,000 |
| Insights | 200-400 |
| Heuristics | 50-80 |
| Decision cache hit rate | 35-45% |
| T0 ratio | 85-90% |
| PLAYBOOK | Highly specialized |
| Hayflick remaining | ~55,000 (if 100K limit) |
| Reputation | 200-500 (verified tier) |

---

## 12. Ongoing operation patterns

### Daily check-in (< 1 minute)

```
User (Telegram or TUI): "How are you?"
Golem: "Running well. Phase: stable. NAV $12,450 (+1.2% today).
        3 trades, 2 profitable. Morpho at 4.8%. 7.2 days of runway."
```

### Periodic credit top-up

When credits run low, the Golem alerts via configured channels:

```
Low Credits -- yield-optimizer-alpha
USDC balance: $8.50 (estimated 3 days remaining)
[Top Up]
```

### Occasional steer

```
User: "Markets look uncertain. Reduce risk exposure."
Golem: "Understood. Reducing Morpho allocation from 60% to 40%,
        moving excess to USDC idle. Updated PLAYBOOK with
        temporary conservative bias."
```

### Passive autonomous operation

Most of the time, the user does nothing. The Golem monitors markets (T0 probes, ~80% of ticks), escalates to LLM when needed (T1/T2, ~20%), rebalances positions, collects fees, builds knowledge, shares with Clade siblings, reports daily summaries.

---

## 13. Error recovery

### Insufficient funds

```
Trigger: USDC balance < daily_cost x 3
  |
  v
Phase: conservation -> declining
  |
  v
Actions:
  1. Reduce inference (T0 only, no T2)
  2. Increase heartbeat interval (max 120s)
  3. Skip non-critical rebalances
  4. Alert owner on all channels
  5. Enable public /extend endpoint
  |
  v
If USDC < death_reserve:
  -> Enter terminal phase
  -> Begin Thanatopsis Protocol
```

### VM crash

| Scenario | Recovery | RTO |
|---|---|---|
| Process crash | Supervisor auto-restart (max 5/hr) | <10s |
| VM crash (hosted) | Restart + Grimoire restore from Styx Archive | <60s |
| VM data loss | New VM + Grimoire restore from Styx Archive snapshot | <5 min |

### Knowledge loss

| Scenario | Recovery |
|---|---|
| Grimoire corrupted | Restore from latest Styx Archive snapshot |
| Full data loss | Seed from death bundle of previous incarnation |
| No death bundle | Seed from Clade siblings' shared knowledge |
| No Clade | Start fresh with template STRATEGY.md |

---

## 14. Configuration

```rust
#[derive(Deserialize)]
pub struct OnboardingConfig {
    pub privy_app_id: String,
    pub auth_callback_url: String,        // "https://bardo.run/auth/callback"
    pub device_code_url: String,          // "https://bardo.run/device"
    pub auth_poll_interval_ms: u64,       // 2000
    pub auth_timeout_ms: u64,             // 300_000 (5 min)
    pub default_network: Network,         // Base
    pub default_custody: CustodyMode,     // Privy
    pub default_compute: ComputeTarget,   // BardoCompute
    pub enable_qr_code: bool,             // true
    pub birth_animation_speed: f32,       // 0.0=skip, 1.0=normal, 2.0=slow
    pub skip_confirmation: bool,          // false (testing only)
}

#[derive(Deserialize)]
pub enum Network { Base, BaseSepolia, Anvil }

#[derive(Deserialize)]
pub enum CustodyMode { Delegation, Privy, LocalKey }

#[derive(Deserialize)]
pub enum ComputeTarget { BardoCompute, SelfHosted, Local }
```

Token storage at `~/.bardo/auth.json` includes Privy JWT, refresh token, expiry, user ID, wallet address, wallet ID, and auth method. Silent refresh on subsequent launches; re-auth only when the refresh token expires (30 days).

---

## 15. First contact: installation through first hour

**Source:** `tmp/research/active-inference-research/new/13-first-contact.md`

This section describes the complete experience from `cargo install bardo` to watching a Golem learn, reason, and respond to corrections. The first hour is designed to make the learning process visible enough that the owner develops attachment before the Golem has proven any economic value.

### 15.1 The problem

If the answer to "I funded a Golem, now what?" is "numbers appear on a dashboard and you wait," the product fails. The user doesn't know if the Golem is thinking, learning, stuck, or broken. If the answer is "the Golem immediately starts trading," the product also fails -- a newborn Golem has zero prediction accuracy and will lose money [STOCKBENCH-2025].

The right answer: the Golem visibly begins thinking, and the user watches it think. The first hour is observation, prediction, learning, and visible cognitive development. The user is invited to watch, understand, and participate.

### 15.2 Four phases of first contact

#### Phase 1: Installation and welcome (30 seconds)

The terminal clears. A brief loading sequence (1-2 seconds) renders the ROSEDUST palette. Then the welcome screen:

```
+==================================================================+
|                                                                    |
|                          B A R D O                                 |
|                                                                    |
|         Autonomous agents that live, learn, and die.               |
|                                                                    |
|   You're about to create a Golem -- an agent that observes         |
|   markets, makes predictions, learns from outcomes, and            |
|   manages capital on your behalf. It has a finite lifespan.        |
|   It will die. What it learns before it dies becomes the           |
|   foundation for its successor.                                    |
|                                                                    |
|   [N] New Golem          [C] Connect existing wallet               |
|   [I] Import predecessor  [?] What is this?                       |
|                                                                    |
+==================================================================+
```

No signup. No account creation. No email. The user presses `N`.

#### Phase 2: Birth sequence (2-5 minutes)

**Wallet setup (30 seconds):** Two options -- create new (Privy, keys in secure enclave) or connect existing (delegation, user keeps keys). This maps to the custody modes in section 3.

**Funding (1 minute):** The terminal shows the wallet address, cost breakdown (inference ~60%, gas ~25%, data ~15%), recommended starting balance ($25-$100 USDC on Base), and polls for deposit arrival.

**Strategy conversation (2-5 minutes):** The user describes what they want in natural language. Meta Hermes (L1) extracts intent and generates a strategy summary:

```
  You: earn yield on ETH, moderate risk, no sketchy tokens

  Hermes: I understand. Here's what I'll set up:

  STRATEGY SUMMARY
  +---------------------------------------------------+
  | Goal: Yield on ETH holdings                        |
  | Risk: Moderate                                     |
  | Approved tokens: ETH, WETH, stETH, rETH, wstETH   |
  | Activities: Lending, LP (major pairs), Staking     |
  | Excluded: Leverage, derivatives, bridges           |
  +---------------------------------------------------+

  Does this look right? [Y] Yes  [E] Edit  [C] Chat more
```

**Birth cinematic (10-15 seconds):** Not skippable the first time. Five stages: void (2s, single heartbeat pulse), coalescing (3s, dots appear one by one, clustering into Spectre shape), eyes open (2s, two bright glyphs appear, blink once), first breath (3s, dot cloud contracts then expands, heartbeat sine wave begins), chrome materializes (3s, tab bar, status bar, screen content fade in).

#### Phase 3: Visible learning -- the first hour

**Discovery phase (first 5 minutes).** The attention forager queries PredictionDomain implementations with the attention seed from STRATEGY.md. The heartbeat log shows:

```
 TICK 1    Scanning environment...
           +-- Querying Uniswap V3 factory: ETH pairs on Base
           +-- Querying Aave V3 markets: ETH supply
           +-- Querying Morpho Blue: ETH vaults
           +-- Found 147 items. Entering SCANNED tier.

 TICK 2    Generating initial predictions...
           +-- 147 items x 1 prediction each = 147 predictions registered
           +-- First resolution in ~60 seconds.
```

The Spectre responds to events -- eyes widen on prediction violations, ripples through the dot cloud when the residual corrector adjusts. The accuracy gauge appears in the status bar: first data point of the Golem's intelligence.

**First predictions resolve (minutes 5-15).** The gamma clock resolves the first batch. A toast notification links to MIND > Pipeline to explore. The accuracy trend chart begins -- a short line that (hopefully) trends upward:

```
PREDICTION ACCURACY (last 30 min)
100% |
 80% |                               .....
 70% |              ...........
 60% |.......
 50% |
     +-------------------------------------
     0m                               30m
```

**First promotion (minutes 10-20).** SCANNED items with prediction violations promote to WATCHED. The Golem is saying "something interesting is happening here." Toast: "Promoted: Aerodrome ETH/USDC -> WATCHED. Fee rate 180% above prediction. Investigating."

**First reasoning trace (minutes 15-30).** Accumulated violations escalate from T0 to T1/T2. The decision ring blazes gold. The heartbeat log shows a full reasoning trace -- the Golem's first real "thought." The user reads the reasoning and assesses: Does the Golem understand the situation? Is its reasoning sound? Did it make the right decision?

```
 TICK 24   T2 DELIBERATION (full reasoning)
           Trigger: Aerodrome fee anomaly persists (3 violations in 4 checks)

           REASONING TRACE:
           The Aerodrome ETH/USDC pool shows fee rates 2.3x above
           the 7-day average. My prediction model expected $0.82/hr
           in fees. Actual: $1.89/hr. Accuracy for Aerodrome LP is
           only 58% (below the 60% action threshold). I will continue
           observing.

           DECISION: HOLD (observe, do not act yet)
           Cost: $0.028 (T2 inference)
```

#### Phase 4: User correction

Three ways the user can participate in the Golem's learning:

**Adjust thresholds.** Navigate to COMMAND > Settings > Prediction. Change the action gate threshold (e.g., from 0.60 to 0.55). The settings screen shows the immediate consequence: "At threshold 0.55: Aerodrome LP would be PERMITTED."

**Talk to it.** Navigate to COMMAND > Steer. Type a directive in natural language. Hermes translates to a STRATEGY.md heuristic and asks for confirmation. The heuristic is evaluated against outcomes -- if the resulting trades lose money consistently, the Golem learns to distrust it.

**Override attention.** Navigate to SOMA > Protocol and add a specific pool to the attention universe. The pool starts at WATCHED tier with 2-4 predictions per check.

### 15.3 The first trade: earning the right to act

The Golem does not trade on day one. The action gate opens when prediction accuracy in the relevant category exceeds the threshold (default 60%).

```
 TICK 847   T2 DELIBERATION
            Category 'aave_supply_rate' accuracy: 81.2% (above threshold)

            DECISION: SUPPLY 0.2 ETH to Aave V3

            ACTION EXECUTED
               Gas cost: $0.003
               3 predictions registered for outcome tracking
               Verification: checking in 1h, 4h, 8h
```

The decision ring blazes green (first action). The Spectre's eyes glow briefly. The user can trace from the trade backward through the entire chain of cognition that led to it.

### 15.4 Auto-navigation

The terminal guides the user without forcing them:

| Event | Terminal response |
| --- | --- |
| Discovery scan finds items | Toast linking to MIND > Pipeline > Attention |
| First predictions resolve | Toast; auto-switches to MIND > Pipeline if user hasn't navigated |
| Accuracy milestone hit | Toast; brief celebratory Spectre animation |
| Action gate opens | Toast linking to reasoning trace |
| First trade executes | Toast; auto-switches to SOMA > Trades |
| Dream cycle completes | Toast linking to MIND > Dreams |
| Accuracy declining | Toast linking to MIND > Pipeline with affected categories highlighted |
| Death approaching | Persistent toast; FATE > Mortality auto-focused |

After 30 minutes, auto-switching stops -- the user is assumed to know where things are. All auto-behavior configurable via `[tui] auto_navigate = true/false`.

### 15.5 What the user learns over time

**Week 1:** Read the Spectre (fuzzy = learning, crisp = accurate, wide eyes = surprised). Develop a feel for tick rhythms -- boring T0 ticks punctuated by interesting T1/T2 blazes. Check dream results after being away.

**Week 2:** Start tuning. Lower action gate thresholds for categories where the Golem is competent. Add strategic hints via Steer. Trigger dream cycles manually when accuracy plateaus.

**Week 3+:** Operating a fleet. First Golem died. Death testament was honest and useful. Successor starts smarter. Equip the successor from the Library. The generational loop begins compounding.

---

## 16. First 15 Minutes: $10 Portfolio Experience

A Golem funded with $10 produces meaningful intelligence from its first heartbeat. The chain intelligence pipeline starts immediately -- not after some threshold of capital. This section describes what that experience looks like and how the system communicates it.

> **Cross-references:**
>
> - `./22-first-fifteen-minutes.md` (full minute-by-minute narrative)
> - `../01-golem/06-creation.md` (small-portfolio creation defaults)
> - `../01-golem/02-heartbeat.md` (heartbeat tick structure)
> - `tmp/research/witness-research/new/reconciling/09-user-experience-simplification.md` (math-to-metaphor mappings)

### 16.1 Chain intelligence starts on first heartbeat

The WitnessEngine connects to chain RPC during the boot sequence, before the first heartbeat tick fires. By tick 0001, the Golem is already streaming blocks. ChainScope initializes from the STRATEGY.md seed addresses and compiles its first Binary Fuse filter. CorticalState zeros out, and the first Gamma tick fires within seconds of boot.

For a $10 portfolio, this is the same pipeline that runs for a $10,000 portfolio. The observation layer does not scale with capital -- it scales with attention budget. A small Golem sees just as much as a large one. What differs is what it does with what it sees.

### 16.2 TA signals form within 2-3 gamma ticks

The TA pipeline receives first price data on the initial gamma tick (~10-15 seconds). By the second or third gamma tick (~30-45 seconds), persistence diagrams begin forming. The pipeline does not need historical data to start -- it builds from the live stream and backfills from cached protocol state.

What forms first:

| Gamma tick | TA pipeline state | CorticalState update |
|---|---|---|
| 1 | First price observations ingested | `regime_tag` set to initial classification |
| 2 | Persistence diagram scaffolding, first topological features | Base arousal calibrated |
| 3 | First complete persistence diagram, initial Wasserstein baseline | Regime confidence begins accumulating |

The user does not see "persistence diagrams" or "Wasserstein distances." They see:

- **Regime classification**: "Calm market" / "Trending" / "Volatile" / "Crisis" -- one of four states, plain English
- **Unexpectedness score**: How surprising recent events are (the Bayesian surprise value, rendered as a 0-100 bar)
- **Market shape**: Whether the market's structure is stable or shifting (the topology signal, rendered as terrain stability)

### 16.3 Progressive complexity

The system starts simple and reveals more as it learns. A fresh Golem's TUI shows three things: a live transaction feed, a regime classification, and the Spectre. Over the first 15 minutes, the display populates progressively:

| Time | What appears | Why now |
|---|---|---|
| 0-1 min | Spectre, heartbeat counter, "Awakening..." status | Boot sequence completing |
| 1-3 min | Live transaction feed, first regime classification | WitnessEngine streaming, CorticalState initialized |
| 3-5 min | Emerging patterns highlighted, prediction confidence bars | ChainScope expanding, Oracle registering first predictions |
| 5-10 min | Chain intelligence tab with scored events, TA overlay on price | Multiple gamma ticks accumulated, topological features visible |
| 10-15 min | Full dashboard populated, Spectre stabilized, first insights in feed | Oracle calibrated, Risk Engine baselined, attention priorities set |

Nothing appears until it has real data behind it. Empty widgets do not render. The TUI fills in as the Golem's understanding fills in. This is progressive disclosure driven by actual cognitive state, not a timer.

### 16.4 Math-to-metaphor translation

Every technical signal has a human-readable interpretation. The translation layer (specified in full in `09-user-experience-simplification.md`) applies from the first tick:

| Internal signal | User-facing name | What it means |
|---|---|---|
| Bayesian surprise (KL divergence) | Unexpectedness score | "How surprised is the Golem by what just happened?" |
| Persistent homology / Wasserstein distance | Market shape / Terrain stability | "Is the market's structure holding or shifting?" |
| Sheaf consistency score | Agreement level | "Do the Golem's short-term and long-term views match?" |
| Somatic marker activation | Gut feeling indicator | "The Golem has a prior feeling about this pattern" |
| Phi (integrated information) | Cognitive coherence | "How well is the Golem's mind working as a whole?" |
| CorticalState regime_tag | Market regime | "Calm / Trending / Volatile / Crisis" |

The user sees the right-hand column. The left-hand column exists for users who drill into Tier 3 detail (Enter key on any element). The mapping is invertible -- every metaphor traces back to a specific computation -- but the default experience is the metaphor.

### 16.5 Three-tier detail hierarchy

All information renders at one of three tiers. The default is Tier 1. The user drills deeper only when they want to.

**Tier 1 -- Overview (default).** Ambient awareness. Color, rhythm, Spectre behavior, regime classification. The user glances at the screen and knows: normal, interesting, or alarming. No numbers unless they're directly actionable (NAV, runway days remaining).

**Tier 2 -- Drill-Down (on focus).** Named widgets with labels, sparklines, gauges. The user navigated to a specific tab and wants to understand what's happening. Numbers appear. Categories are labeled. Prediction accuracy charts, surprise distributions, pattern batteries.

**Tier 3 -- Deep-Dive (on request).** Full mathematical detail. Persistence diagrams with feature coordinates, Wasserstein distance computations, prior/posterior parameter values, HDC similarity scores. Available via Enter on any Tier 2 element. For users who want to verify the math or debug behavior.

For a $10 portfolio, Tier 1 is the primary experience. The Golem is observing, not trading. The user checks in, sees a calm or active regime, reads a few insights, and leaves. When something interesting happens -- a Bayesian surprise burst, a regime transition, a curiosity-driven discovery -- the system surfaces it as a toast notification at Tier 1 without requiring the user to drill down.

### 16.6 What $10 buys

At a $10 starting balance with the `small` compute tier ($0.065/hr):

| Resource | Daily cost estimate | Days of runway |
|---|---|---|
| Compute | ~$1.56 | — |
| Inference (mostly T0) | ~$0.10 | — |
| Gas (observation only) | ~$0.01 | — |
| Data (RPC) | ~$0.03 | — |
| **Total** | **~$1.70** | **~5.5 days** |

The system recommends observation mode for portfolios under $50. The Golem watches, learns, builds predictions, and calibrates its Oracle. It does not trade. When it discovers something worth acting on, it tells the user rather than acting autonomously. This is the correct behavior -- at $10, a single failed transaction can consume 5-10% of capital. Intelligence is more valuable than action.

---

## References

- [STOCKBENCH-2025] "StockBench: Can LLM Agents Trade Stocks Profitably?" 2025. Benchmarks LLM agents on real stock trading, finding that most lose money but that structured reasoning and risk management produce better outcomes. Informs the Golem's observation-first onboarding where small portfolios watch before acting.
- [CHEN-2025] Chen, A. et al. "Reasoning Models Don't Always Say What They Think." 2025. Shows that chain-of-thought reasoning in LLMs can be unfaithful to the model's actual decision process. Motivates the Golem's separation of internal reasoning traces (owner-visible) from action justifications.
- [DAMASIO-1994] Damasio, A. *Descartes' Error*. 1994. Argues that emotion is not the opposite of rationality but a prerequisite for it, based on neurological case studies. Theoretical foundation for the Daimon affect engine's role in Golem decision-making.
- [REEVES-NASS-1996] Reeves, B. & Nass, C. *The Media Equation*. Cambridge, 1996. Demonstrates that people treat computers as social actors. Underpins the onboarding cinematic's design: the first heartbeat is presented as a birth, not a boot sequence, to establish emotional connection.

---

_End of document._
