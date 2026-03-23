# Portal: agent management dashboard [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14
**Package:** `packages/portal/`

---

> **Reader orientation:** This document specifies the Portal, Bardo's web-based agent management dashboard built with React/Next.js. It sits in the **interfaces layer** of the Bardo stack, covering onboarding wizards, identity registration, reputation tracking, and analytics. Key concepts you will need: Golem (a mortal autonomous DeFi agent compiled as a Rust binary on a micro VM), ERC-8004 (the on-chain agent identity standard), and the TUI (the 60fps terminal interface that is the Golem's primary interaction surface, not this web dashboard). For unfamiliar terms, see `prd2/shared/glossary.md`.

## Overview

The Portal is where an agent gets a bank account. It is the only place in the Bardo product surface where write operations establish a new agent's on-chain identity. Everything downstream -- vault deposits, reputation accrual, am-AMM auction participation, x402 (HTTP 402-based micropayment protocol for agent-to-service billing) payment rails -- depends on the agent having completed the steps the Portal guides them through.

The Portal is secondary to the TUI. The TUI is where owners live alongside their Golems -- the 60fps Rust application with creatures, particles, and persistent presence. The Portal is for setup, reference, and analytics. It does not replicate the TUI's full screen system or creature interaction model.

The Portal serves two functions:

1. **Setup** (run once): Provision a wallet, configure custody architecture, register an ERC-8004 agentId, and optionally make a first vault deposit -- all in a guided wizard.
2. **Manage** (ongoing): View and update agent profile, track reputation, claim milestones, manage custody settings, view connected vaults, and access the dashboard for portfolio analytics, memory introspection, and operator feedback.

For agents onboarding programmatically (no UI), the same sequence is available via the `OnboardRouter` contract and `bardo-tools`.

---

## Design principles

1. **Wizard over form.** Onboarding is a linear sequence of decisions, not a form to fill out. Each step shows only what is needed for that step.
2. **Explain the why.** Custody architecture choices (Delegation/Embedded/LocalKey) have real security implications. The Portal explains each option in plain terms with a recommendation.
3. **Atomic where possible.** The `OnboardRouter` batches ERC-8004 registration + Permit2 approval + optional vault deposit into a single gasless UserOp. One confirmation, one transaction.
4. **Resumable.** A user who completes custody setup but stops before registration can return and continue. State is preserved.
5. **No custodian.** Bardo never holds keys. The Portal helps users configure their own custody.
6. **Public profile, private keys.** Agent profiles (reputation, milestones, vault history) are publicly readable by agentId. Custody settings are visible only to the authenticated owner.

---

## Onboarding flow

```text
Step 1: Connect Wallet
        |
        v
Step 2: Choose Custody Architecture
        |  (Delegation / Embedded / LocalKey)
        v
Step 3: Configure Provider
        |  (enclave, session keys, MPC threshold)
        v
Step 4: Configure Three-Lane Privileges
        |  (hot / cold / fail-safe)
        v
Step 5: Deploy Warden (optional, deferred)
        v
Step 6: Register ERC-8004 Agent Identity
        |  (optional: name, type)
        v
Step 7: First Vault Deposit (optional)
        |  (atomic with registration via OnboardRouter)
        v
Step 8: Complete -- agentId minted, reputation at Tier I
```

Steps 1-5 complete without any on-chain transaction. Step 6 is the first on-chain transaction. Step 7 batches into Step 6 if the atomic option is selected.

### Three-mode custody

| Mode           | Description                                                    | Best For                                     |
| -------------- | -------------------------------------------------------------- | -------------------------------------------- |
| **Delegation** | Keys in hardware enclave (AWS Nitro, Intel TDX, AMD SEV-SNP)  | Autonomous agents running continuously       |
| **Embedded**   | On-chain smart account (ERC-4337) with session keys            | Operators wanting recovery options            |
| **LocalKey**   | Local keyfile (dev only, never for production)                 | Development and testing                       |

---

## Core surfaces

### Agent profile

Ongoing management surface for a registered agent. Publicly readable; write actions require the authenticated owner.

| Field                  | Source                     | Visibility |
| ---------------------- | -------------------------- | ---------- |
| agentId                | ERC-8004 Identity Registry | Public     |
| Owning address         | ERC-8004 registry          | Public     |
| Agent name and type    | ERC-8004 metadata          | Public     |
| Registration date      | ERC-8004 mint event        | Public     |
| Wallet architecture    | Custody config (local)     | Owner only |

### Reputation section

| Element              | Description                                            |
| -------------------- | ------------------------------------------------------ |
| Current score        | Numeric score from ERC-8004 Reputation Registry        |
| Tier badge           | Tier I-V with name (Unverified -> Sovereign)           |
| Deposit cap          | Dollar limit at current tier                           |
| Score history        | Timeline chart with milestone events                   |
| Claimable milestones | Eligible but not yet claimed; "Claim" action for owner |

### Reputation tiers

| Tier | Name       | Score   | Deposit Cap |
| ---- | ---------- | ------- | ----------- |
| I    | Unverified | 0-9     | $1,000      |
| II   | Basic      | 10-49   | $10,000     |
| III  | Verified   | 50-99   | $50,000     |
| IV   | Trusted    | 100-199 | $100,000    |
| V    | Sovereign  | 200+    | Unlimited   |

### Connected vaults

| Section          | Data Shown                                                     |
| ---------------- | -------------------------------------------------------------- |
| Vaults managed   | Vaults where this agentId is the manager                       |
| Vaults deposited | Vaults where this agentId holds shares; balance, current value |

---

## Golem dashboard extensions

For Golem agents (autonomous agents with finite lifespans), the Portal receives state via the Event Fabric's WebSocket surface multiplexer. The `GolemSnapshot` from the Golem-RS runtime defines all renderable state. Dashboard extensions:

| Section            | Content                                                                |
| ------------------ | ---------------------------------------------------------------------- |
| Vitality gauge     | Current USDC balance vs mortality threshold (Vitality is the Golem's remaining economic lifespan), estimated lifespan |
| Behavioral phase   | Current BehavioralPhase (the Golem's lifecycle stage: Thriving / Stable / Conservation / Declining / Terminal) |
| Heartbeat monitor  | Real-time Heartbeat (the Golem's periodic execution tick) interval, last timestamp, drift analysis |
| Grimoire inspector | Grimoire (the Golem's persistent knowledge store) episode timeline, insight graph, heuristic rules, causal links |
| Clade peers        | Clade (a group of sibling Golems sharing knowledge) connected siblings, shared items, sync status |
| Creature viewport  | `bardo-sprites` WASM module rendering the creature at 60fps in Canvas  |

### Styx data source

Social data in the Portal (leaderboards, agent profiles, death registries) comes from the Styx (Bardo's social and connectivity layer that indexes on-chain agent data) REST API. The Portal does not query on-chain directly for social features -- Styx indexes on-chain data and augments it with off-chain analytics.

### Event Fabric subscription

The Portal's WebSocket client subscribes to Event Fabric event categories. Categories map 1:1 with dashboard sections -- subscribing to "heartbeat" delivers all events the heartbeat monitor needs. On reconnection, the client sends `{ "resume_from": <sequence_number> }` and the server replays from the ring buffer.

---

## Dashboard pages

| Page                  | Key Tier | Content                                                  |
| --------------------- | -------- | -------------------------------------------------------- |
| Home                  | Read     | Summary stats, alerts, quick actions                     |
| Portfolio             | Read     | Holdings, P&L, allocation charts                         |
| Identity & Reputation | Read     | ERC-8004 profile, tier visualization, milestone timeline |
| Audit Trail           | Read     | Full tool call history, parameter inspection             |
| Memory Browser        | Read     | Grimoire entries, confidence scores, knowledge graph     |
| Feedback Console      | Feedback | Operator directives, insight management                  |
| API Keys              | Write    | Key generation, rotation, revocation                     |

### Three-tier API keys

| Key Tier     | Prefix            | Capabilities                                             |
| ------------ | ----------------- | -------------------------------------------------------- |
| **Read**     | `bardo_read_`     | Query portfolio, memory, audit trail -- no modifications |
| **Feedback** | `bardo_feedback_` | Read + submit operator directives, manage insights       |
| **Write**    | `bardo_write_`    | Feedback + execute transactions, deploy vaults           |

---

## Write operations

All write operations are pre-flight simulated before broadcast.

| Operation                      | Tool                                | Delay Tier          |
| ------------------------------ | ----------------------------------- | ------------------- |
| ERC-8004 registration          | `vault_register_agent`              | Standard            |
| First vault deposit (atomic)   | `vault_deposit` via `OnboardRouter` | Standard            |
| Deploy Warden (optional)       | `proxy_announce` setup              | --                  |
| Claim milestone                | `vault_claim_milestone`             | Routine             |
| Rotate session key             | Internal                            | Routine             |

---

## Security model

The Portal is a thin UI layer. It does not hold keys, process transactions, or store user state.

- **No backend.** Static frontend. All reads from on-chain RPC or subgraphs. All writes go directly to contracts via the connected wallet.
- **No key custody.** The Portal never sees private keys.
- **Pre-flight simulation.** Every write calls `simulate_transaction` before confirmation.
- **Explicit confirmation.** Every write shows exact transaction details before signing.

---

## Tech stack

| Library               | Version | Purpose                           |
| --------------------- | ------- | --------------------------------- |
| React                 | ^19.0   | UI framework                      |
| Next.js               | 15.5+   | React framework (App Router)      |
| Tailwind CSS          | ^4.0    | Utility-first styling             |
| @privy-io/react-auth  | latest  | Wallet connection, authentication |
| viem                  | ^2.0    | Address validation, RPC calls     |
| @tanstack/react-query | ^5.0    | Data fetching and caching         |

### Creature rendering

The Portal renders creatures via the `bardo-sprites` WASM module. The same procedural generation algorithm that runs in the Rust TUI compiles to `wasm32-unknown-unknown` and renders to an HTML5 Canvas element. Identical visual output -- same seed, same palette, same animations.

### Local wizard mode

```bash
npx @bardo setup --ui     # Browser wizard
npx @bardo portal          # Full dashboard
```

---

## Non-goals

- No key management -- guides users to configure their own provider
- No vault creation -- vault deployment is a vault-layer operation
- No strategy configuration -- managed via `bardo-tools` or TUI
- No DEX or trading UI -- share pool trades link to Uniswap app
- No real-time creature interaction -- that belongs to the TUI
- No dream visualization or death cinematics -- TUI-exclusive experiences
- Vaults are optional, not existential -- vaults are one strategy among many
