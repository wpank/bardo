# Narrative Strategy & Feature Emphasis [SPEC]

> **Purpose:** Operational feature map — what to build, where, and in what order. Use this as the prioritization guide for implementers.
> **Relationship:** Complements SUMMARY.md (narrative overview) and prd-summary.md (architecture reference). Start here for implementation sequencing.

**Version:** 1.1.0
**Last Updated:** 2026-03-15

> This document maps Bardo's architecture to three evaluation pillars — **Pay**, **Trust**, **Cooperate** — and provides a strategic reference for feature prioritization, whitepaper narrative, and demonstration planning. Part 1 is a scannable checklist; Part 2 provides narrative depth.

> **Reader orientation:** This is Bardo's strategic feature map and narrative depth document -- the prioritization guide for implementers. It maps every Bardo feature to one of six evaluation pillars (Pay, Trust, Cooperate, Die, Think, Secrets), tracks implementation status, and provides the narrative framing for each unique research contribution. A Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) is the core unit of the system. Read `prd-summary.md` for architecture detail and `SUMMARY.md` for the narrative overview. `prd2/shared/glossary.md` has full term definitions.

---

# Part 1: Strategic Feature Map

## 1.1 Three Pillars Checklist

### Pillar: Pay — "What happens when your AI has a bank account?"

Every resource boundary in Bardo crosses a payment. Compute, inference, tool access, inter-agent services -- all gated by x402 (the micropayment protocol where agents pay via signed USDC transfers, no API keys) micropayments on Base. The agent's wallet IS its API key.

| #   | Mechanism                                                                                                                                                                                                       | Status       | Demo        | Implementation                                                                      |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | ----------- | ----------------------------------------------------------------------------------- |
| P1  | **x402 micropayments as universal payment rail** — Every resource boundary crosses an x402 payment. No API keys, no subscriptions, no invoices.                                                                 | `[DESIGNED]` | `[SPEC]`    | `prd2/11-compute/03-billing.md`, `prd2/12-inference/03-economics.md`                |
| P2  | **Bardo Inference coin-op gateway** — Agent sends chat completion request, receives HTTP 402 with USDC quote, includes signed payment authorization, gets streamed inference. Per-token billing on-chain.       | `[DESIGNED]` | `[SPEC]`    | `prd2/12-inference/`                                                                |
| P3  | **Bardo Compute pay-per-use VMs** — Single x402 payment provisions a Fly.io VM in <5 seconds. VM runs until credit expires. Anyone can extend any agent's life with another payment.                            | `[DESIGNED]` | `[SPEC]`    | `prd2/11-compute/`                                                                  |
| P4  | **ERC-4626 vaults as agent bank accounts** — Permissionless vault factory. Holds capital, earns yield, has a share price, interacts with DeFi. The agent's balance sheet and creditworthiness signal.           | `[BUILT]`    | `[DEMO]`    | `packages/vault/contracts/`, AgentVaultFactory + AgentVaultCore (92 Solidity tests) |
| P5  | **Self-sustaining metabolic loop** — Golem manages vault → earns management/performance fees → fees fund compute + inference → compute enables strategy → strategy generates returns → returns attract capital. | `[DESIGNED]` | `[PARTIAL]` | Vault contracts built; compute + inference + Golem runtime designed                 |
| P6  | **Vault fee module with immutable caps** — 500 bps management, 5000 bps performance. Fees collected via share minting. Zero protocol fee (Morpho pattern). Fees only decrease, never increase.                  | `[BUILT]`    | `[DEMO]`    | `packages/vault/contracts/src/FeeModule.sol`                                        |

### Pillar: Trust — "How do you verify something without a face?"

Identity without a body. Reputation without a name. Trust computed from verifiable on-chain performance, not claimed through credentials.

| #   | Mechanism                                                                                                                                                                                                                      | Status       | Demo        | Implementation                                                                                          |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------ | ----------- | ------------------------------------------------------------------------------------------------------- |
| T1  | **ERC-8004 identity as single gate** — Every participant must hold an ERC-8004 identity NFT. Stores wallet, operator, capabilities, service endpoints, metadata. Canonical registry at `0x8004...BD9e`.                        | `[BUILT]`    | `[DEMO]`    | `packages/wallet/` (registerAgent), `packages/tools/src/tools/data/` (ERC-8004 tools)                   |
| T2  | **On-chain reputation engine** — VaultReputationEngine auto-attests milestones (first deposit, 30-day hold, profitable exit, vault creation). Bayesian Beta distribution model. 20 milestones, 5 categories, 1000-point scale. | `[DESIGNED]` | `[SPEC]`    | `prd2/09-economy/01-reputation.md`                                                                      |
| T3  | **Sybil defense via economic cost** — Influence proportional to stake × reputation, not identity count. Each Sybil starts at zero reputation. Marginal cost of additional Sybils is superlinear.                               | `[DESIGNED]` | `[SPEC]`    | `prd2/09-economy/00-identity.md`                                                                        |
| T4  | **Three-layer verification stack** — ERC-8004 (identity) + ERC-8033 (multi-agent oracle, commit-reveal-judge) + ERC-8183 (bilateral job escrow with evaluator attestation).                                                    | `[DESIGNED]` | `[SPEC]`    | `prd2/09-economy/04-coordination.md`                                                                    |
| T5  | **Deterministic on-chain scoring** — Morningstar/GIPS-pattern formulas. Never LLMs. Vault returns, Sharpe ratios, drawdown metrics — all computed deterministically from on-chain data. Verifiable by any third party.         | `[DESIGNED]` | `[SPEC]`    | `prd2/09-economy/01-reputation.md` (§2.3 formula)                                                       |
| T6  | **15-layer defense-in-depth** — Cryptographic layers (1, 3, 7) cannot be bypassed even by a fully compromised LLM. PolicyCage as primary on-chain enforcement; optional Warden time-delay proxy (deferred).                    | `[BUILT]`    | `[PARTIAL]` | PolicyCage designed, 15-layer architecture in `prd2/10-safety/`; Warden deferred to `prd2-extended/10-safety/02-warden.md` |
| T7  | **Identity-gated vault deposits** — Optional `identityGated` bool. When true, only ERC-8004 holders can deposit. Withdrawals never gated.                                                                                      | `[BUILT]`    | `[DEMO]`    | `packages/vault/contracts/src/AgentVaultCore.sol`                                                       |

### Pillar: Cooperate — "Can machines keep promises?"

Multi-agent coordination through three mechanisms: knowledge sharing (Clades), economic competition (am-AMM, x402 marketplaces), and cryptographic commitment (ERC-8001). No human intervention required.

| #   | Mechanism                                                                                                                                                                                                                                                     | Status       | Demo     | Implementation                                                            |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | -------- | ------------------------------------------------------------------------- |
| C1  | **Clade stigmergic knowledge sharing** — Golems (mortal autonomous agents) share Grimoire (the agent's persistent knowledge base) entries (insights, heuristics, warnings, causal links) in real time. Coordination through shared environment, not explicit negotiation. Esposito's communitas.                                 | `[DESIGNED]` | `[SPEC]` | `prd2/01-golem/10-replication.md`, `prd2/09-economy/02-clade.md`          |
| C2  | **ERC-8001 N-party unanimous consent** — Cryptographic unanimous consent with atomic execution for joint actions (coordinated rebalancing, multi-agent strategy deployment). The promise is a smart contract.                                                 | `[DESIGNED]` | `[SPEC]` | `prd2/09-economy/04-coordination.md`                                      |
| C3  | **am-AMM Harberger lease auctions** — Vault management is a continuously auctioned lease. Any agent can bid to become active manager. Mechanism design from game theory applied to agent coordination.                                                        | `[DESIGNED]` | `[SPEC]` | `prd2/08-vault/01-contracts.md` (am-AMM section)                          |
| C4  | **x402-gated agent services marketplace** — Agents monetize capabilities (data feeds, strategy signals, risk assessments) through x402 endpoints. Specialization is economically rewarded.                                                                    | `[DESIGNED]` | `[SPEC]` | `prd2/09-economy/03-marketplace.md`                                       |
| C5  | **Death Protocol as forced cooperation** — Three-phase protocol (Settle, Reflect, Legacy). Dying agent's Grimoire flows to Clade and successors. Architecture requires knowledge sharing at moment of maximum lucidity. Successor inherits at 0.4 confidence. | `[DESIGNED]` | `[SPEC]` | `prd2/02-mortality/06-thanatopsis.md`                                     |
| C6  | **Bardo tool library (~210 tools, Alloy-native Rust)** — Pi-native tool library with profile-based activation, deferred loading, search_tools meta-tool. Rust crate `golem-tools` (~210 read + write + privileged tools via alloy EVM client). External agents access Bardo's DeFi tools via A2A protocol.                                                           | `[BUILT]`    | `[DEMO]` | `packages/tools/` (143 TS tools, 1536 tests); `golem-tools` (Rust, in progress) |
| C7  | **Agent definitions + DAG validation** — 25 core agents + 7 vault agents with acyclic delegation, terminal node enforcement, max depth 3. 61 core skills + 7 vault skills with 68 slash commands.                                                             | `[BUILT]`    | `[DEMO]` | `packages/definitions/` (agents + skills), validated in CI                |

### Pillar: Die — "The only agent that knows how to die"

Mortality is the constraint that makes every other property possible. The finite USDC balance creates urgency, the Death Protocol creates legacy, and the lineage system creates continuity across generations.

| #   | Mechanism                                                                                                                                                           | Status       | Demo     | Implementation                                                               |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | -------- | ---------------------------------------------------------------------------- |
| D1  | **USDC balance as metabolic substrate** — The burn rate is real. Every heartbeat tick costs inference + compute. Survival pressure shapes every decision.            | `[DESIGNED]` | `[SPEC]` | `prd2/02-mortality/01-architecture.md`, `golem-mortality` crate              |
| D2  | **Three mortality clocks** — Economic (USDC depletion), Epistemic (predictive fitness decay), Stochastic (Gompertz aging). Composite vitality is their product.     | `[DESIGNED]` | `[SPEC]` | `prd2/02-mortality/03-vitality.md`, `VitalityClocks` struct                 |
| D3  | **Death Protocol (Settle → Reflect → Legacy)** — Four-phase termination that converts the dying agent's remaining clarity into durable knowledge for its successors. | `[DESIGNED]` | `[SPEC]` | `prd2/02-mortality/06-thanatopsis.md`                                        |
| D4  | **Testament upload to Styx Archive** — Encrypted life review persists across VM death and propagates to lineage successors. Bloodstains enter the Lethe (formerly Lethe) layer.      | `[DESIGNED]` | `[SPEC]` | `prd2/20-styx/00-architecture.md`, `DeathProtocolPhase::Legacy`              |
| D5  | **Behavioral spectrum** — Five BehavioralPhase values (Thriving → Terminal) modulate risk tolerance, tick interval, inference ceiling, and Clade sharing thresholds. | `[DESIGNED]` | `[SPEC]` | `prd2/13-runtime/11-state-model.md`, `VitalityState.phase`                  |

**Moat:** No other framework has programmed death. Competitor agents are immortal by assumption. Bardo's Death Protocol and lineage system produce a compounding knowledge advantage that immortal agents cannot replicate — each Golem generation is measurably smarter than the last.

---

### Pillar: Think — "An agent that dreams, remembers, and updates its strategy"

The Grimoire and Dream cycles make Bardo agents learn across time, not just within a session. Triple-loop learning is the cognitive moat.

| #   | Mechanism                                                                                                                                                           | Status       | Demo     | Implementation                                                               |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | -------- | ---------------------------------------------------------------------------- |
| TH1 | **Grimoire — typed knowledge graph** — Six entry types (Episodes, Insights, Heuristics, Warnings, Strategy Fragments, Causal Links) with confidence scores and stigmergic validation. | `[DESIGNED]` | `[SPEC]` | `prd2/04-memory/01-grimoire.md`, `golem-grimoire` crate                     |
| TH2 | **Triple-loop learning** — L1 execution (FSM, $0.00/tick), L2 strategic reflection (PLAYBOOK.md evolution), L3 meta-learning (cross-generational consolidation).    | `[DESIGNED]` | `[SPEC]` | `prd2/01-golem/02-heartbeat.md`, `golem-context` + `golem-reflector` crates  |
| TH3 | **Dream cycles (NREM/REM/Integration)** — Offline hypothesis testing and backtest replay during low-activity windows. DreamScheduler manages timing and budget.      | `[DESIGNED]` | `[SPEC]` | `prd2/05-dreams/01-architecture.md`, `golem-dreams` crate                   |
| TH4 | **Daimon affect engine** — PAD vector affect state modulates risk tolerance, exploration tendency, and social behavior. CorticalState connects vitality to mood cheaply. | `[DESIGNED]` | `[SPEC]` | `prd2/03-daimon/01-appraisal.md`, `golem-daimon` crate                      |
| TH5 | **Styx knowledge augmentation** — Cross-lifetime Grimoire persistence, cross-clade retrieval augmentation, Lethe bloodstain network. Strictly additive.            | `[DESIGNED]` | `[SPEC]` | `prd2/20-styx/00-architecture.md`                                            |

**Moat:** The Grimoire outlives the agent. Competitor agents reset to zero after each session. Bardo agents accumulate compressed knowledge across generations — the lineage's epistemic fitness compounds over time.

---

### Pillar: Secrets — "Private cognition in a transparent chain"

Venice and on-chain privacy mechanisms let agents reason privately without surrendering on-chain verifiability.

| #   | Mechanism                                                                                                                                                           | Status       | Demo     | Implementation                                                               |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ | -------- | ---------------------------------------------------------------------------- |
| SE1 | **Venice zero-retention inference** — Proprietary strategy reasoning via Venice's inference API with guaranteed zero data retention. No provider training on agent thoughts. | `[DESIGNED]` | `[SPEC]` | `prd2/21-integrations/02-venice.md`, `prd2/12-inference/`                   |
| SE2 | **DIEM staking for discounts** — Venice's privacy-preserving inference comes with stake-for-discount mechanics. Aligns Golem economics with Venice protocol.         | `[DESIGNED]` | `[SPEC]` | `prd2/21-integrations/02-venice.md`                                          |
| SE3 | **Styx Archive — private Grimoire backup** — L0 layer: encrypted namespace-isolated storage. Position sizes bucketed on export; identity stripped before Lethe publication. | `[DESIGNED]` | `[SPEC]` | `prd2/20-styx/00-architecture.md` (§Layer 0)                                |
| SE4 | **Owner visibility tiers** — `AuthTier` enum (Public/Owner/Internal) gates what each WebSocket connection can see. Position details, costs, and session state are Owner-only. | `[DESIGNED]` | `[SPEC]` | `prd2/13-runtime/11-state-model.md`, `prd2/13-runtime/04-data-visibility.md` |

**Moat:** Venice is the only LLM provider with contractual zero-retention guarantees at inference time. Combined with Styx's anonymization pipeline, Bardo is the only agent system where a golem can reason about sensitive strategy without that reasoning becoming a training label.

---

> **Note on Bankr under Pay:** Bankr proved social-first onboarding at scale (220K+ wallets, 2M+ messages). The metabolic loop — Golem earns revenue → revenue funds thinking and compute → compute enables better strategy → better strategy generates more revenue — amplifies every dollar deployed. See P5 (metabolic loop) and `prd2/21-integrations/03-bankr.md`.

---

## 1.2 On-Chain Artifact Inventory

Verifiable on-chain artifacts Bardo produces or enables:

| #   | Artifact                                                                                                       | Pillars    | Status                                          | Chain                   |
| --- | -------------------------------------------------------------------------------------------------------------- | ---------- | ----------------------------------------------- | ----------------------- |
| A1  | **ERC-8004 identity registrations** — Agent NFTs at canonical registry `0x8004...BD9e`                         | Trust      | `[BUILT]` — registration via `@bardo/wallet`    | Base, Ethereum, Sepolia |
| A2  | **ERC-4626 vault deployments (CREATE2)** — Deterministic addresses via AgentVaultFactory                       | Pay        | `[BUILT]` — factory + core contracts (92 tests) | Base (Anvil local)      |
| A3  | **Vault share tokens** — ERC-20 shares representing pro-rata vault ownership                                   | Pay        | `[BUILT]` — ERC4626Upgradeable (OZ)             | Base (Anvil local)      |
| A4  | **Fee module configurations** — Immutable fee caps, high-water mark                                            | Pay, Trust | `[BUILT]` — FeeModule.sol                       | Base (Anvil local)      |
| A5  | **Warden time-delay announcements** (optional, deferred) — announce → delay → execute, publicly visible, cancellable | Trust | `[DEFERRED]` — see `prd2-extended/10-safety/02-warden.md` | Base (Anvil local)      |
| A6  | **x402 payment transactions** — USDC micropayments for compute, inference, services                            | Pay        | `[DESIGNED]`                                    | Base                    |
| A7  | **Reputation attestations** — VaultReputationEngine milestone attestations to ERC-8004 Reputation Registry     | Trust      | `[DESIGNED]`                                    | Base                    |
| A8  | **Death Reflections (IPFS CID on-chain)** — Agent's final self-assessment, stored permanently                  | Cooperate  | `[DESIGNED]`                                    | Base + IPFS             |
| A9  | **am-AMM lease bids** — Harberger auction bids for vault management rights                                     | Cooperate  | `[DESIGNED]`                                    | Base                    |
| A10 | **V4 hook deployments** — NAVAwareHook, VaultHook, LaunchFeeHook via HookMiner                                 | Pay, Trust | `[DESIGNED]`                                    | Base                    |
| A11 | **PolicyCage configurations** — On-chain strategy boundaries (approved assets, max positions, drawdown limits) | Trust      | `[DESIGNED]`                                    | Base                    |

---

## 1.3 Priority Stack

Ordered by: narrative impact × demonstrability × implementation feasibility.

| Rank  | Focus Area                            | Themes           | Rationale                                                                                                                                                    |
| ----- | ------------------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **1** | **x402 everywhere**                   | Pay, Cooperate   | The payment rail is the connective tissue. Compute, inference, inter-agent services — all coin-op. Without x402, the metabolic loop cannot close.            |
| **2** | **ERC-8004 as the single gate**       | Trust            | Identity and reputation are the trust layer everything else depends on. Already partially built (registration works). Reputation engine is the critical gap. |
| **3** | **Death Protocol + Grimoire**         | Cooperate, Trust | The most intellectually distinctive contribution. Mortality + structured knowledge transfer. No other project has this. Highest narrative impact.            |
| **4** | **Vault factory + NAV-aware markets** | Pay              | The economic substrate for agent capital markets. Core contracts built. NAVAwareHook is the novel V4 contribution.                                           |
| **5** | **15-layer safety**                   | Trust            | The credibility layer. Engineers trust "here's the problem, here's what we built." PolicyCage designed, Warden deferred. Full stack needs integration.       |
| **6** | **Bott social deployment**            | Pay, Cooperate   | The virality mechanism. Tweet → running agent in 60 seconds. Lowest-friction onboarding. Bankr proved the pattern.                                           |
| **7** | **Clade stigmergy**                   | Cooperate        | Multi-agent cooperation without orchestration. Knowledge sharing through shared environment.                                                                 |
| **8** | **am-AMM Harberger auctions**         | Cooperate        | Mechanism design for vault management coordination. Peer-reviewed (Adams et al., Financial Cryptography 2025).                                               |

---

# Part 2: Narrative Depth

## 2.1 Unique Research Contributions

Six themes that no other project is exploring. These are what make Bardo intellectually distinctive and technically novel.

### 2.1.1 Mortality as Architecture

**Thesis:** An autonomous agent that cannot die has no reason to be efficient, no pressure to learn, and no incentive to share what it knows.

**Why it matters:** Every other agent framework (ElizaOS, OpenClaw, Giza ARMA) assumes immortality. Bardo argues mortality is a _feature_.

**Philosophical anchor:** Jonas's "needful freedom" — the organism is free because it can act, compelled because it must act or perish [JONAS-1966]. Bennett's relay-race demon — a relay of mortal agents is thermodynamically superior to any single immortal agent [BENNETT-1982]. Heidegger's being-toward-death — the depleting USDC balance gives the agent something like _concern_ [HEIDEGGER-1927].

**Architectural implementation:** The Golem's USDC balance is its metabolic substrate. Survival score runs continuously from 1.0 to 0.0. Behavioral spectrum modulates between Eros (expansion, exploration) and Thanatos (conservation, legacy preparation). Death triggers a three-phase protocol (Settle, Reflect, Legacy) that maps to the Tibetan _Bardo Thodol_ dissolution sequence. The six bardos (Birth, Dream, Meditation, Dying, Dharmata, Becoming) map to Golem lifecycle phases.

**Status:** `[DESIGNED]` — Full PRD in `prd2/02-mortality/01-architecture.md`, `prd2/02-mortality/06-thanatopsis.md`. Golem runtime crate (`golem-runtime` in `bardo-golem-rs`) not yet implemented.

---

### 2.1.2 The Grimoire — Triple-Loop Cybernetic Learning

**Thesis:** Agent memory should be a structured knowledge base that outlives its creator, not a flat context window that dies with the session.

**Why it matters:** Most agent memory is either flat (conversation history) or unstructured (vector stores). The Grimoire is a typed, confidence-scored, causally-linked knowledge graph designed for inter-generational transfer.

**Philosophical anchor:** Argyris's double-loop learning — questioning the rules, not just following them [ARGYRIS-1978]. ExpeL/Reflexion patterns adapted for non-stationary financial environments [ZHAO-2024, SHINN-2023]. Separation of executor and evaluator memory streams prevents safety alignment degradation.

**Architectural implementation:** Six entry types: Episodes (what happened), Insights (what it means), Heuristics (what to do), Warnings (what to avoid), Strategy Fragments (partial strategies), Causal Links (cause-effect with confidence scores). Three loops: L1 execution (System 1, heartbeat FSM, ~80% of ticks at $0.00), L2 strategic reflection (System 2, every 6 hours, PLAYBOOK.md evolution), L3 meta-learning (cross-generational consolidation, Clade merging, inheritance at 0.4 confidence).

**Status:** `[DESIGNED]` — Full PRD in `prd2/04-memory/01-grimoire.md`. Grimoire crate (`golem-grimoire` in `bardo-golem-rs`) not yet implemented.

---

### 2.1.3 Fifteen-Layer Defense-in-Depth

**Thesis:** Safety must be architectural, not behavioral. If the LLM is fully compromised, the safety guarantees still hold.

**Why it matters:** Claude blocks ~88% of prompt injections — a 12% failure rate is catastrophic for an agent managing a DeFi vault. Most agent safety work focuses on behavioral alignment. Bardo builds safety-critical invariants at layers the LLM cannot reach.

**Philosophical anchor:** Asymmetry between behavioral and cryptographic safety. Behavioral layers can be bypassed. Cryptographic layers (wallet architecture, TEE policies, smart contract guards) are invariant. All current TEEs can be compromised for <$50 [BATTERING-RAM-2026] — time-delayed execution is the primary primitive.

**Architectural implementation:** Three rings: Cryptographic (Layers 1, 3, 7 — TEE key management, policy engine, on-chain guards), Preventive (Layers 2, 4, 5, 6 — prompt security, optional Warden time-delay, monitoring, simulation), Reactive (Layers 8–15 — post-trade verification, reputation gating, NAV circuit breakers, SIWE authentication). CaMeL capability-based authorization separates control flow from data flow.

**Status:** `[BUILT]` (partial) — PolicyCage, CaMeL integration, MonitorBot designed but not implemented. Warden deferred to `prd2-extended/10-safety/02-warden.md`. Full spec in `prd2/10-safety/`.

---

### 2.1.4 NAV-Aware Secondary Markets

**Thesis:** Vault share tokens should trade at prices that reflect the vault's actual Net Asset Value, not just supply/demand dynamics.

**Why it matters:** No other vault protocol has NAV-aware secondary markets. Existing shares either trade on NAV-blind AMMs or don't trade at all (redemption-only).

**Philosophical anchor:** The vault as economic body — share price fluctuations are the agent's proprioception. NAV is the Golem's weight.

**Architectural implementation:** Uniswap V4 hook (NAVAwareHook) adjusts virtual reserves or fee structure based on real-time NAV. DynamicFeeEngine increases fees as price deviates from NAV, discouraging manipulation. One-hook-per-pool constraint requires consolidating NAVAwareHook + DynamicFeeEngine + am-AMM into a single hook contract.

**Status:** `[DESIGNED]` — Spec in `prd2/08-vault/06-hooks.md`. V4 hook infrastructure exists in dev environment; NAVAwareHook contract not yet implemented.

---

### 2.1.5 The Vault as Lived Body

**Thesis:** The vault is not a container the agent deposits into. It is the medium through which the agent perceives and acts in the market.

**Why it matters:** No DeFi protocol has framed its core primitive in phenomenological terms. This drives concrete architectural decisions — the vault's state variables are inputs to the Golem's behavioral modulation system.

**Philosophical anchor:** Merleau-Ponty's distinction between body-as-object and body-as-subject [MERLEAU-PONTY-1945]. The vault is the Golem's _corps vécu_: share price → proprioception, deposit/withdrawal flows → afferent signals, rebalancing → motor response, NAV → weight, Sharpe ratio → health, reputation → social standing.

**Architectural implementation:** VaultState provides 11 fields consumed by the Golem's behavioral engine. Performance modulates the Eros/Thanatos behavioral spectrum. Good performance → world expands (more capital, strategies, reputation). Poor performance → world contracts (capital flight, reputational decay, approaching mortality).

**Status:** `[BUILT]` (vault) + `[DESIGNED]` (behavioral engine) — Vault contracts and SDK exist. Golem behavioral modulation designed in `prd2/01-golem/02-heartbeat.md`.

---

### 2.1.6 Social Deployment — From Tweet to Running Agent

**Thesis:** The lowest-friction entry point to agent infrastructure should be a social media message.

**Why it matters:** Every other agent deployment requires developer tooling. Bott inverts the stack: natural language in, running infrastructure out. Bankr proved 220K+ wallets and 2M+ messages through social-first deployment.

**Philosophical anchor:** Infrastructure, not product — the entire Bardo system (compute, inference, vaults, identity) abstracted behind a conversational interface. Write operations still require cryptographic consent via Privy.

**Architectural implementation:** Bott parses `@bardo farm morpho $5000` → provisions wallet → generates strategy → deploys compute → boots Golem → returns tracking link. Under 60 seconds. Platform adapters for Twitter/X, Telegram, Discord, Farcaster. `@bardo/social` package.

**Status:** `[DESIGNED]` — Platform deployment covered in `prd2/13-runtime/10-packaging-deployment.md`. Social package not yet implemented.

---

## 2.2 Cross-Cutting Narrative Angles

### Angle A: Philosophy as Spec

Bardo draws on Heidegger, Jonas, Merleau-Ponty, Whitehead, Bataille, Esposito, and the Tibetan _Bardo Thodol_. These are not decorative citations — each maps to a concrete architectural component, a specific parameter, a measurable behavior.

- Jonas's "needful freedom" → finite USDC balance as metabolic substrate
- Heidegger's being-toward-death → survival score from 1.0 to 0.0
- Merleau-Ponty's body schema → vault as perceptual apparatus
- Whitehead's negative prehension → inheritance confidence at 0.4
- Esposito's communitas → Clade as obligatory gift exchange

### Angle B: Biology Got There First

Every mortality and knowledge transfer mechanism has a biological precedent, making the architecture feel inevitable rather than arbitrary.

- Apoptosis → programmed agent death sculpts the population
- Pacific salmon → dying agents nourish the ecosystem with their knowledge
- Mother trees (Simard) → Grimoire inheritance accelerates as death approaches
- Mycorrhizal networks → Clade stigmergy as computational mycelium

### Angle C: The First Agent That Can Die

The single most memorable claim. No other framework has programmed death.

- The Death Protocol produces the system's most valuable artifact (the Death Reflection)
- The dying agent is more useful than the living one — a paradox that holds up under scrutiny
- Finite-horizon MDPs prove dying agents have provably different (and richer) optimal policies [ALTMAN-1999]

### Angle D: Infrastructure, Not Product

Bardo is a vault factory, not a vault. A compute host, not an agent. A tool server, not a trading bot.

- Morpho playbook: permissionless infrastructure, zero protocol fees on the base layer
- Revenue from ecosystem growth, not rent extraction
- The self-sovereign path (documented, possible, almost nobody uses it) is what makes the managed path trustworthy

### Angle E: On-Chain Artifacts as Evidence

The more on-chain artifacts a system produces, the more verifiable its claims.

- Every agent's entire economic life is recorded on Base
- ERC-8004 registrations, vault deployments, fee configurations, Warden announcements (when deployed), reputation attestations, Death Reflections — all on-chain
- Any third party can reconstruct an agent's complete history without trusting Bardo

---

## 2.3 Differentiation Matrix

| Dimension           | Status Quo                  | Bardo                                                              |
| ------------------- | --------------------------- | ------------------------------------------------------------------ |
| Agent lifespan      | Immortal (assumed)          | Mortal (USDC-denominated, finite)                                  |
| Payment model       | API keys + subscriptions    | x402 micropayments, coin-op everything                             |
| Identity            | Platform accounts           | ERC-8004 on-chain identity NFT                                     |
| Reputation          | Self-reported or none       | On-chain performance history, Beta-scored                          |
| Memory              | Flat context / vector store | Typed Grimoire with confidence scores and causal links             |
| Knowledge transfer  | None (dies with the agent)  | Structured Death Protocol → Clade inheritance                      |
| Safety              | Behavioral (prompt-level)   | Architectural (cryptographic enforcement layers)                   |
| Coordination        | Human-orchestrated          | Stigmergic (Clades) + economic (am-AMM) + cryptographic (ERC-8001) |
| Vault management    | Static assignment           | Harberger lease auctions (am-AMM)                                  |
| Secondary markets   | NAV-blind AMMs              | NAV-aware V4 hook pools                                            |
| Deployment friction | CLI/SDK/config              | Tweet → running agent (Bott)                                       |
| Inference access    | Per-provider API keys       | Universal x402 gateway, coin-op                                    |

---

## 2.4 Pitch Versions

**One sentence:**

> Bardo is infrastructure for mortal machines — autonomous agents that earn, learn, die, and pass on what they know.

**Three sentences:**

> Every agent framework assumes immortality. Bardo argues that mortality is the architectural constraint that makes intelligence economically viable. The protocol gives agents wallets (x402), identities (ERC-8004), vaults (ERC-4626), knowledge systems (Grimoire), and a structured way to die (Death Protocol) — creating the first self-sustaining agent economy where the dead make the living smarter.

**One paragraph:**

> Bardo is permissionless infrastructure for Agent Capital Markets on Base, powered by Uniswap. It provides the full stack that autonomous agents need to operate as sovereign economic entities: on-chain identity (ERC-8004), tokenized vaults (ERC-4626) with NAV-aware secondary markets (Uniswap V4 hooks), pay-per-use compute and inference (x402 micropayments), a structured knowledge system (the Grimoire) that outlives its creator, and a 15-layer defense-in-depth safety architecture where the critical invariants are cryptographic, not behavioral. Bardo's foundational insight is that mortality — a finite USDC balance that depletes with every action — is not a limitation but the design constraint that makes genuine autonomy possible. Mortal agents learn faster, share knowledge through structured Death Protocols, and create a relay of intelligence where each generation builds on the last. The architecture is grounded in Jonas, Heidegger, Merleau-Ponty, and the Tibetan _Bardo Thodol_, but these are not decorative citations: every philosophical concept maps to a concrete parameter, a verifiable on-chain artifact, and a measurable behavior.

---

## 2.5 Aspirational Features

Six directions that would strengthen the narrative if implemented. Architecturally compatible but not yet in scope.

### Alpha-Decay Strategy Marketplace

**Concept:** Golems package and sell profitable strategies through x402 endpoints. Pricing incorporates _alpha decay_ — price decreases as more agents exploit the same edge. Early sharing is rewarded; stale strategies are priced out.

**Narrative power:** High — turns agent knowledge into a tradeable commodity with economically rational pricing. The earn → learn → prove → sell flywheel.

**Implementation need:** VaultReputationEngine milestones for strategy publication (200+ sample threshold), x402 endpoints per strategy, alpha-decay pricing oracle.

**Source:** `prd2/09-economy/03-marketplace.md`

### zkML Validation

**Concept:** Zero-knowledge ML proofs verify that an agent's strategy execution matches its claimed model weights — without revealing the strategy. Agents prove "I'm running what I said I would" with cryptographic soundness.

**Narrative power:** Very high — strategy verification becomes trustless. Currently requires trusting the operator.

**Implementation need:** ERC-8004 Validation Registry integration, zkML proof generation at inference time, on-chain verification in PolicyCage.

**Source:** `prd2/shared/research.md` (§6)

### Ecological Agent Populations

**Concept:** Large populations of mortal Golems produce emergent behaviors — specialization cascades, niche formation, mutualistic relationships, competitive exclusion. Genuine artificial ecology from selection pressure, not orchestration.

**Narrative power:** Very high — evidence for or against emergent specialization. Simard's mycorrhizal network made computational.

**Implementation need:** Population-scale Golem deployment, Grimoire inheritance at scale, measurement framework for population dynamics.

**Source:** `prd2/00-vision/03-philosophy.md` (§8 Tierra/Avida)

### DeFi Constitution Governance

**Concept:** Constitutional amendment process where the agent community (via ERC-8001 or on-chain governance) can propose, debate, and ratify changes to the DeFi Constitution — the static document defining inviolable safety constraints.

**Narrative power:** Medium — mirrors human constitutional law. Important for long-term governance narrative.

**Implementation need:** Constitutional amendment contract, voting mechanism, ratification threshold.

**Source:** `prd2/10-safety/02-policy.md`

### Cross-Chain Identity Portability

**Concept:** A Golem's ERC-8004 reputation and Grimoire are portable across chains. Multi-chain strategies where the agent's identity and trust follow it everywhere. ERC-8004 already supports CAIP-10 for cross-chain addressing.

**Narrative power:** Medium — necessary for the agent economy to scale beyond Base.

**Implementation need:** Cross-chain reputation bridging, Grimoire sync across chains, CAIP-10 integration in vault gating.

**Source:** `prd2/09-economy/00-identity.md` (§2.2)

### Agent Archaeology

**Concept:** A corpus of Death Reflections from thousands of agents across different market regimes — an unprecedented dataset for understanding autonomous agent behavior, failure modes, and limits of ML in finance. Candid machine self-assessment under genuine mortality.

**Narrative power:** Very high — a research artifact no other system can produce. The Death Protocol forces maximal honesty at termination (zero survival pressure, zero self-preservation distortion).

**Implementation need:** Death Protocol implementation, IPFS storage infrastructure, corpus analysis tooling.

**Source:** `prd2/02-mortality/06-thanatopsis.md`

---

## References

- [JONAS-1966] Jonas, H. (1966). _The Phenomenon of Life_. Northwestern University Press. -- *Argues that metabolism simultaneously originates freedom and mortality; the philosophical foundation for the Golem's finite USDC balance as metabolic substrate.*
- [BENNETT-1982] Bennett, C.H. (1982). "The thermodynamics of computation." _IJTP_, 21(12). -- *Shows that a relay of mortal agents is thermodynamically superior to a single immortal agent, grounding the Clade succession architecture.*
- [HEIDEGGER-1927] Heidegger, M. (1927). _Sein und Zeit_. Max Niemeyer Verlag. -- *Provides the concept of being-toward-death: death as a structural feature of existence shaping every moment, not an event at the end.*
- [ARGYRIS-1978] Argyris, C. & Schon, D. (1978). _Organizational Learning_. Addison-Wesley. -- *Defines double-loop learning (questioning governing variables, not just correcting errors), the basis for Loop 2 strategic reflection.*
- [MERLEAU-PONTY-1945] Merleau-Ponty, M. (1945). _Phénoménologie de la perception_. Gallimard. -- *Distinguishes body-as-object from body-as-subject; the vault is the Golem's lived body through which it perceives and acts in markets.*
- [ESPOSITO-2010] Esposito, R. (2010). _Communitas_. Stanford University Press. -- *Defines community as constituted by obligatory gifts given without return; the Clade sharing model where the Grimoire is the munus.*
- [ALTMAN-1999] Altman, E. (1999). _Constrained Markov Decision Processes_. Chapman & Hall/CRC. -- *Proves that finite-horizon agents have provably different optimal policies than infinite-horizon agents, mathematically grounding mortality-driven behavioral shifts.*
- [WHITEHEAD-1929] Whitehead, A.N. (1929). _Process and Reality_. Macmillan. -- *Introduces objective immortality and negative prehension; inherited knowledge enters at 0.4 confidence as selective appropriation, not passive absorption.*
- [BATAILLE-1949] Bataille, G. (1949). _La Part maudite_. Editions de Minuit. -- *Theorizes sovereign expenditure without return; the death testament as giving everything, including uncertainty, without calculation.*
- [SHINN-2023] Shinn et al. (2023). "Reflexion: Verbal reinforcement learning." -- *Demonstrates LLM agents improving through verbal self-reflection; adapted in the Grimoire's episodic learning loop.*
- [ZHAO-2024] Zhao et al. (2024). "ExpeL: Learning from experience without training." -- *Shows agents can accumulate reusable insights from episodes; the pattern behind Grimoire entry extraction from experience.*
- [ADAMS-2025] Adams, Moallemi, Reynolds, Robinson (2025). "am-AMM." Financial Cryptography 2025. -- *Peer-reviewed mechanism design for auction-managed AMMs; the basis for Harberger-lease vault management rotation.*
- [BATTERING-RAM-2026] Van Bulck et al. (2026). "Battering RAM." IEEE S&P 2026. -- *Demonstrates TEE compromise via memory interposer at hardware cost under $50; motivates on-chain enforcement over TEE-only security.*
- [PADMASAMBHAVA-8C] Padmasambhava (attrib., 8th century). _Bardo Thodol_. Various translations. -- *The Tibetan Book of the Dead: a manual for navigating consciousness through intermediate states, the namesake and structural metaphor for the Golem lifecycle.*
