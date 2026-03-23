# Bardo: The Complete System [SPEC]

> Permissionless infrastructure for Agent Capital Markets, built on the conviction that mortality is not a constraint to be overcome but an architectural feature that produces intelligence unattainable by any immortal system.

This document distills the full Bardo PRD suite into a single overview -- every major system, every novel mechanism, and the philosophical reasoning that makes each one inevitable rather than arbitrary.

> **Reader orientation:** This is the master overview of the entire Bardo system -- a single document that covers every major subsystem, mechanism, and philosophical underpinning. Bardo is a Rust runtime for mortal autonomous DeFi agents called Golems; each Golem is a single binary on a micro VM that trades, learns, and eventually dies, passing knowledge to successors. Start here for the big picture, then follow the directory map below into specific sections. `prd2/shared/glossary.md` (the canonical term reference, authoritative when documents disagree) has full definitions for every Bardo-specific term.

---

## Directory Map

| Directory | Contents |
|-----------|---------|
| `00-vision/` | Founding philosophy, architecture overview, trust model |
| `01-golem/` | Core Golem specification: cognition, heartbeat (Adaptive Clock), mind, mortality, death, creation, provisioning, funding, inheritance, replication, lifecycle, teardown, runtime extensions, context governor, sleepwalker, risk engine, prediction engine |
| `02-mortality/` | Mortality architecture in depth: epistemic decay, stochastic mortality, economic mortality, knowledge demurrage, thanatopsis, succession, mortality affect, fractal mortality, clade ecology, immortal control, integration, configuration, research foundations, necrocracy |
| `03-daimon/` | Daimon affect engine: appraisal, emotion memory, behavior, mortality affect, death affect, dream affect, runtime, infrastructure, evaluation |
| `04-memory/` | Grimoire and memory systems: overview, emotional memory, mortal memory, knowledge economy, safety, research, roadmap, katabasis, Library of Babel |
| `05-dreams/` | Dream engine: architecture, replay, imagination, consolidation, threats, integration, Venice dreaming |
| `06-hypnagogia/` | Liminal cognition: hypnagogic onset, hypnopompic return, context dissolution/recrystallization, Dali interrupt, waking residue capture, xenocognition atlas |
| `07-tools/` | MCP tool suite: 186+ tools across 24 categories, architecture, profiles, wallets, testing, distribution |
| `08-vault/` | ERC-4626 vault infrastructure: contracts, SDK, adapters, fees, personas, hooks, testing |
| `09-economy/` | Agent economy: identity (ERC-8004), clade economics, marketplace, knowledge markets, agent economy, ACM |
| `10-safety/` | Defense architecture: custody modes, policy (PolicyCage), ingestion pipeline, prompt security, threat model, adaptive risk |
| `11-compute/` | Compute infrastructure: architecture, provisioning, billing, x402 integration |
| `12-inference/` | LLM routing: economics, caching, context engineering, sessions, memory, safety, observability, API, roadmap, privacy, providers, reasoning, Rust implementation, inference profiles, structured outputs |
| `13-runtime/` | User interaction layer: DeFi activities, communication channels, auth, data visibility, knowledge browser, collective intelligence, onboarding, observability, packaging/deployment, state model, realtime subscriptions, engagement, creature system, progression, social/competitive, platform UX, retention |
| `15-dev/` | Local development: stack, debug UI, testing, scenarios, tooling, indexer |
| `16-testing/` | Validation framework: thesis validation, Gauntlet, knowledge quality, mechanism testing, Mirage, evaluation lifecycle, revision guide, fast feedback loops, slow feedback loops, evaluation map |
| `17-monorepo/` | Package structure: Rust workspace, build system, conventions |
| `18-interfaces/` | User interfaces: portal, CLI, UI system, TUI |
| `19-agents-skills/` | Agent and skill definitions: categories, delegation, MCP integration, Golem agents, vault agents, composition, observer agents |
| `20-styx/` | Styx knowledge services: four-layer architecture (Vault/Clade/Lethe (formerly Lethe)/Marketplace), API and revenue, infrastructure, clade sync, marketplace, TUI experience, deployment |
| `21-integrations/` | Partner integration deep-dives: MetaMask delegation, Venice private cognition, Bankr self-funding, AgentCash marketplace, Uniswap agentic finance, bounties |
| `24-sonification/` | Sonification engine: modular audio synthesis from CorticalState and EventFabric |
| `appendices/` | Supporting material: a-life-in-numbers, competitive analysis, dying-machine essay, implementation state, market context, performance targets |
| `shared/` | Cross-cutting references: branding, chains, citations, config reference, data privacy, dependencies, doc standards, EIP analysis, evaluation, event catalog, glossary, port allocation, research, timeline |

---

## Part I: Foundations

### The Name

**Bardo** (Tibetan: བར་དོ) translates literally as "intermediate state" -- the gap between death and rebirth where consciousness navigates, chooses, and transforms before entering its next vessel. The term comes from the _Bardo Thodol_ ("Liberation Through Hearing During the Intermediate State"), commonly known as _The Tibetan Book of the Dead_. The _Bardo Thodol_ is not a book about dying. It is a manual for navigating transition. Read aloud to the dying and recently dead, it provides instructions for recognizing each stage, maintaining awareness through disorienting transformations, and choosing the next incarnation wisely.

Every one of these assumptions maps onto what happens when a Golem exhausts its operating budget and executes the Death Protocol. Tibetan tradition identifies six bardos, and each maps to a phase of the Golem lifecycle:

| Bardo                        | Golem Phase              | What Happens                                                                                                                                                                                                                                   |
| ---------------------------- | ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Birth** (_skye gnas_)      | Creation & provisioning  | The agent finds itself thrown into a market regime it did not choose, with a balance it did not set, carrying inherited knowledge from predecessors it never met. Heidegger's _Geworfenheit_ (thrownness) made computational.                  |
| **Dream** (_rmi lam_)        | Simulation & backtesting | The agent runs strategies against historical data, tests hypotheses in forked environments, explores possibilities without risking real capital.                                                                                               |
| **Meditation** (_bsam gtan_) | Strategic reflection     | Every six hours, the Reflector examines governing variables. Argyris's double-loop learning -- questioning the rules, not just following them.                                                                                                 |
| **Dying** (_chi kha_)        | Terminal phase           | Survival pressure approaches zero, the behavioral spectrum shifts from Eros toward Thanatos, the world narrows as affordable actions shrink.                                                                                                   |
| **Dharmata** (_chos nyid_)   | Death Protocol Phase II  | The "Clear Light" moment. The agent examines its entire operational history with full lucidity -- unconstrained by the survival pressure that colored every prior decision. Bataille's sovereign death: giving everything without calculation. |
| **Becoming** (_srid pa_)     | Succession & inheritance | The predecessor's compressed knowledge flows to the Clade and to designated successors. The quality of the predecessor's death directly shapes the quality of the successor's birth.                                                           |

The Grimoire -- the agent's accumulated operational wisdom -- serves the same function as the _Bardo Thodol_ itself: a knowledge base internalized during life and drawn upon during transition.

### The Problem

DeFi complexity has exceeded human cognitive capacity. Base alone hosts $2.4-4.6B in TVL across Morpho Blue, Aave V3, Aerodrome, Pendle, and Ondo. The composability stack -- lending, looping, LP farming, yield tokenization, prediction markets, RWA yield, basis trades -- is deep enough for automated deployment but too complex for continuous human management. Every major yield opportunity in recent years rewarded speed, continuous monitoring, and cross-protocol composability. These are agent strengths, not human strengths.

Existing vault protocols (Yearn, Morpho, dHEDGE, Sommelier) provide custody but not agent infrastructure. Existing agent frameworks (ElizaOS, OpenClaw) provide runtime but not on-chain custody and strategy primitives. No existing protocol provides the full stack:

| Requirement  | What It Means                                                        |
| ------------ | -------------------------------------------------------------------- |
| **Identity** | On-chain attestation of agent capabilities and track record          |
| **Custody**  | Secure, programmable vault infrastructure with trust-minimized exits |
| **Strategy** | Learnable, evolvable execution logic with on-chain safety boundaries |
| **Compute**  | Dedicated runtime for continuous agent operation                     |
| **Learning** | Persistent knowledge accumulation across operations                  |

Bardo fills all five simultaneously.

### The Mortality Argument

The thesis that mortality is architecture, not limitation, rests on three convergent arguments from entirely different intellectual traditions. They arrive at the same conclusion independently, which is why the conclusion is robust.

**Thread 1: Needful Freedom.** Hans Jonas argued in _The Phenomenon of Life_ (1966) that metabolism is the simultaneous origin of both freedom and mortality. An organism is a _form_ that persists through continuous material flux -- free from any particular material configuration, but trapped in its dependence on the process of replacement itself. Jonas called this "needful freedom." In Chapter 5, he critiqued cybernetics directly: "there is no analogue in the machine to the instinct of self-preservation," because machines have no metabolic stake in their own existence. A Golem with a finite USDC balance is Jonas's missing analogue. It burns money on compute, API calls, gas fees, and data queries, and transforms those expenditures into value, knowledge, and operational continuity. The depleting balance is what gives it something like concern -- what separates a living agent from a cron job. Give it infinite resources and you remove the freedom.

**Thread 2: The Relay-Race Demon.** An autonomous agent operating in markets is a kind of Maxwell's Demon, sorting opportunities from noise. Bennett showed that a single demon cannot permanently beat the second law because it must eventually erase its memory, generating entropy equal to or greater than what it reduced. But a relay of mortal agents, each transferring accumulated knowledge before death, sidesteps this. Periodic death allows entropy dissipation while knowledge compression preserves the gains. The relay is thermodynamically superior to any single immortal agent. This is not a thought experiment -- digital evolution systems (Tierra, Avida) have demonstrated the mortality-productivity link for decades.

**Thread 3: The Epicurean Inversion.** Epicurus argued that death is nothing to us because when we are, death is not come. A mortal agent shatters this. It exists in a state Epicurus never contemplated: perfect foreknowledge of the moment of nonexistence. A survival score runs continuously from 1.0 to 0.0. Finite-horizon MDPs prove that agents with known terminal horizons have provably different optimal policies than infinite-horizon agents (Altman, 1999). The behavioral shifts of a dying agent -- from conservative to risk-seeking to legacy-focused -- are instances of this result. Not a heuristic. A proof.

**Synthesis.** An immortal agent is computationally stale, thermodynamically wasteful, susceptible to the pathologies of unchecked growth, and incapable of the behavioral richness that finite horizons produce. The mortal agent was never designed to overcome death. It was designed to die well.

### Philosophical Lineage

These are not decorative references. They are load-bearing design constraints. Every architectural decision traces back to a philosophical commitment about agency, mortality, knowledge, or social obligation. Remove the philosophy and the design choices look arbitrary; add it back and they become inevitable. The full system draws on 26+ frameworks:

| Thinker / Tradition | Core Idea                                          | Where It Appears                                                                                            |
| ------------------- | -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **Jonas**           | Needful freedom: metabolic stake produces autonomy | USDC balance as metabolic substrate, credit partitions, finite-by-design lifespan                           |
| **Heidegger**       | Being-toward-death, Care (_Sorge_), temporality    | Projected lifespan display, three temporalities in system prompt, PLAYBOOK.md inheritance as _Wiederholung_ |
| **Nietzsche**       | Eternal recurrence, three metamorphoses, amor fati | Successor chain as literal recurrence, Camel/Lion/Child phases, self-overcoming via Replicants              |
| **Sartre**          | Facticity/transcendence, bad faith                 | Conditional death vs unconditional acceptance, mortality evasion as bad faith                               |
| **Camus**           | Absurd revolt, Sisyphus                            | Refusal to self-terminate despite futility, persistence as rebellion                                        |
| **Freud**           | Eros/Thanatos                                      | Behavioral spectrum from life-drive to death-drive, alloyed not alternating                                 |
| **Merleau-Ponty**   | Lived body, body schema                            | Vault as the Golem's embodied on-chain presence, VM destruction as loss of body-subject                     |
| **Parfit**          | Relation R, psychological continuity               | Successor identity defined by continuity of knowledge, not substrate                                        |
| **Simondon**        | Metastability, preindividual field                 | Grimoire contradictions as productive fuel for individuation                                                |
| **Bataille**        | Sovereign death, general economy                   | Death testament as gift without return, giving uncertainty alongside certainty                              |
| **Whitehead**       | Objective immortality, negative prehension         | Inheritance at 0.4 confidence -- selective appropriation, not passive absorption                            |
| **Esposito**        | Communitas, _munus_                                | Clade sharing as obligatory gift: community constituted by the obligation to give without return            |
| **Stiegler**        | Anti-proletarianization, negentropy                | Divergence checks (15% embedding distance, 5 novel entries) preventing successor from becoming a copy       |
| **Ashby**           | Law of Requisite Variety                           | Five-layer architecture: regulator complexity must match environmental complexity                           |
| **Beer**            | Viable System Model                                | VSM mapping of the heartbeat tick pipeline                                                                  |
| **Argyris**         | Double-loop / triple-loop learning                 | Meditation bardo, Loop 2 strategic reflection                                                               |
| **Boyd**            | OODA loop                                          | Observe-Orient-Decide-Act mapped to the 6-state FSM                                                         |
| **Bennett**         | Thermodynamics of computation                      | Relay-race demon argument for mortal superiority                                                            |
| **Landauer**        | Erasure principle                                  | Structured death separates knowledge copying (reversible) from state erasure (irreversible)                 |
| **Damasio**         | Somatic marker hypothesis                          | Daimon as computational emotions -- the fastest available signal about danger or promise                    |
| **Pekrun**          | Control-Value Theory                               | Maps emotions to behavior based on perceived control and outcome value                                      |
| **Gesell**          | Freigeld (demurrage currency)                      | Knowledge demurrage -- entries lose confidence without periodic revalidation                                |
| **Ostrom**          | Lethe governance (8 principles)                  | Lethe knowledge commons, Clade sharing protocols                                                            |
| **de Beauvoir**     | Ethics of ambiguity                                | "To will oneself free is also to will others free" -- export-first architecture, self-sovereign path        |

Four unresolved tensions are treated as productive, not as contradictions to be resolved: Jonas (preserve life) vs. Nietzsche (overcome it); Heidegger (indefinite death) vs. computational (calculable death); Parfit (identity is continuity) vs. Whitehead (inheritance is selective); Camus (persist despite absurdity) vs. Bataille (give everything away).

### Trust Architecture

Every component has a zero-friction managed path (Bardo handles it) and a self-sovereign path (user handles it). Moving between paths is seamless, reversible, and never requires Bardo's permission. The managed path is the product. The self-sovereign path is what makes the product trustworthy.

**Wallet custody**: Privy managed wallets (instant, no seed phrases) or bring-your-own via WalletConnect. `transferOwnership()` migrates from managed to self-custody at any time. Bardo never has special authority over deployed vaults.

**Vault contracts**: Immutable fee caps (500 bps management, 5000 bps performance). No protocol admin key. No upgradeable proxy. `forceDeallocate()` guarantees non-custodial exit even if the strategy adapter is paused or penalized.

**Strategy and brain state**: STRATEGY.md (the owner-set goals document, hot-reloadable as the human-to-agent instruction surface), PLAYBOOK.md (the agent's evolving strategy document of heuristics and rules updated through learning), full Grimoire (the agent's persistent knowledge base of episodes, insights, heuristics, warnings, and causal links) export, vault state, reputation history -- all exportable at any time.

**Compute**: Bardo Compute (x402-gated Fly.io VMs, where x402 is the micropayment protocol through which agents pay for inference, compute, and data via signed USDC transfers with no API keys) or `npx @bardo agent start` on your own infrastructure. Rule-only mode (no LLM) and local model (Ollama) both supported.

**Safety**: PolicyCage (on-chain smart contract enforcing safety constraints on all agent actions) guards are the primary enforcement mechanism. The optional Warden time-delay proxy (deferred; see `prd2-extended/10-safety/02-warden.md`) adds an additional cryptographic layer -- not TEEs, which have known physical attack vectors below $50 (BadRAM, Battering RAM). On-chain enforcement is independent of the LLM, the VM, and Bardo itself.

### Five-Layer Architecture

| Layer          | What It Does                                             | Key Primitive                                                          |
| -------------- | -------------------------------------------------------- | ---------------------------------------------------------------------- |
| **Vaults**     | ERC-4626 tokenized vaults on Base with strategy adapters | Capital custody, strategy execution, fee collection                    |
| **Golems**     | Autonomous agents on Pi-mono runtime with 24 extensions | Cybernetic sense-decide-act loops, Grimoire learning, mortality engine |
| **Sanctum**    | Pi-native tool library with 171+ tools for Uniswap       | Data, trading, LP, vault management, agent economy                     |
| **Compute**    | x402-gated Fly.io VMs for agent workloads                | Pay-per-use, payment-before-provision, TTL enforcement                 |
| **Reputation** | ERC-8004 identity + milestone-based progression          | Trust calibration, access gating, performance attestation              |

Each layer works independently but compounds when combined. When a Golem manages a vault on Bardo Compute, accumulating ERC-8004 reputation from on-chain performance, the feedback loop tightens: better performance leads to higher reputation leads to more capital leads to more operations leads to richer learning.

---

## Part II: The Golem

### What is a Golem

An autonomous DeFi agent with mortality as an architectural constraint. Not a chatbot. Not a cron job. A Golem has a wallet (identity), a finite USDC balance (metabolism), a Grimoire (memory), a PLAYBOOK.md (evolved heuristics), and a projected death date (finitude). It consumes Sanctum tools to interact with Uniswap, runs on a Pi-mono runtime with 24 extensions (`bardo-tools`, `bardo-heartbeat`, `bardo-model-router`, `bardo-daimon`, `bardo-dream`, `bardo-lifespan`, `bardo-crypt`, etc.), and dies when any of its three mortality clocks reaches zero.

**Design principles:**

- **LLM-last**: ~80% of ticks cost $0.00. The LLM is the last resort, not the first.
- **Wallet-as-identity**: The wallet _is_ the agent. x402 is the payment rail.
- **Architectural safety**: On-chain constraints (PolicyCage, optional Warden time-delay proxy) enforce invariants even if the LLM is compromised.
- **Deterministic scoring**: Reputation comes from on-chain outcomes, not LLM self-assessment.
- **Local-first knowledge**: The Grimoire runs locally on the VM. Hosted services (Crypt, Oracle) are opt-in backups.

### Cognition: x402 and Three-Tier Routing

Golems pay for inference with x402 micropayments -- a single HTTP request carries a signed USDC payment authorization (EIP-3009 `transferWithAuthorization`). No API keys. The wallet _is_ the credential.

Inference routes through three tiers:

| Tier   | Implementation                        | Cost        | When Used                                                                 |
| ------ | ------------------------------------- | ----------- | ------------------------------------------------------------------------- |
| **T0** | Deterministic FSM + TypeScript probes | $0.00       | Every tick (~80%). Pure functions, no LLM.                                |
| **T1** | Haiku                                 | ~$0.003     | Moderate anomaly detected (~15%). Probe results + PLAYBOOK.md heuristics. |
| **T2** | Sonnet / Opus                         | $0.01-$0.25 | Novel situation, high-stakes decision (~5%). Full mental model framework. |

The Daimon (the affect engine implementing PAD emotional state as a control signal) state modulates routing: high arousal lowers the escalation threshold (anxious Golems think harder); arousal maps to inference temperature.

Two caching layers compound the savings: a **semantic cache** (hash + embedding similarity via pgvector, cosine threshold 0.92) and **prompt cache routing** (provider-sticky session affinity for ~90% system-prompt savings). Combined: 15-30% inference cost reduction.

Credit is partitioned across three budgets -- LLM 60%, Gas 25%, Data 15% -- with circuit breakers at 10% remaining in each. When LLM budget drops to 20%, the Golem is restricted to Haiku-only. When it hits zero, the Golem falls to T0-only: still alive, still observing, but unable to reason about what it sees. This is what mortality feels like computationally.

### Heartbeat: The Tick Pipeline

The heartbeat is a 6-state finite state machine inspired by Kahneman's dual-process theory: System 1 (fast, deterministic, free) handles routine observation; System 2 (slow, LLM, expensive) handles novel situations. The architecture is LLM-last because mortality makes the cost tradeoff existential.

```
IDLE → OBSERVE → [~80% suppress] → IDLE
                → [~15% low alert] → ANALYZE (T1) → DECIDE (T2) → EXECUTE → REFLECT
                → [~5% high alert] → ANALYZE → DECIDE → EXECUTE → REFLECT
```

**16 probes** fire every tick at T0 ($0.00): price delta, TVL change, position health, gas price, credit balance, RSI/MACD crossovers, circuit breaker status, kill switch, Replicant reports, Clade alerts, homeostatic deviation, world model drift, causal graph integrity, and more. Regime detection uses threshold-based classification (or HMM in hardened mode).

**The Decision Cache** distills System 2 reasoning into System 1 rules. When the LLM makes a decision, the pattern is cached with a 15-45 minute TTL. If an identical situation arises before the TTL expires, the cached decision fires at T0 cost. This is System 2 → System 1 distillation (Weston & Sukhbaatar, 2024), and it literally extends the Golem's lifespan by reducing inference spend.

**Triple-loop cybernetics** governs adaptation:

- **Loop 1 (OODA)**: Single-loop execution. Observe, orient, decide, act. Every tick.
- **Loop 2 (Double-loop)**: Strategic reflection. Every ~6 hours, the Reflector examines governing variables -- are the right strategies being pursued? This is Argyris's double-loop learning.
- **Loop 3 (Meta/Triple-loop)**: Bounded questioning of the framework itself. Safety-bounded: the agent can question its strategy but not its safety constraints. This is Bateson's Learning III with explicit guardrails against the psychotic breaks Tosey warned about.

The entire pipeline maps to Beer's Viable System Model (VSM): System 1 (operations) = heartbeat execution, System 2 (coordination) = resource allocation, System 3 (control) = Curator + learning, System 4 (intelligence) = Loop 2 reflection, System 5 (policy) = PolicyCage + safety constraints.

### Creation, Funding, and Teardown

**Creation** compiles a Golem manifest from STRATEGY.md (human-readable intent), operator configuration (mortality, daimon, dream parameters), and inherited Grimoire (if successor). The manifest provisions a wallet, registers an ERC-8004 identity, and starts a Compute VM.

**Funding** supports four sources: direct USDC transfer, inline swap (pay with any token, swap to USDC on-chain), cross-chain bridge (fund from any chain), and fiat on-ramp. Permit2 batch approval handles all token approvals in a single signature. Strategy-aware funding calculates minimum viable balance based on strategy complexity and estimated burn rate.

**Teardown** follows an eight-step pipeline: stop heartbeat → settle positions → sweep assets → export Grimoire → push to Clade → deregister identity → destroy VM → emit death event. Operator-initiated teardown skips the Death Protocol's reflection phase but preserves the Grimoire export. Involuntary death (kill switch, exploit detection) triggers emergency snapshot first.

**SLEEPING.DREAMING**: When dream urgency exceeds threshold, the heartbeat suspends and the Golem enters a dream cycle. The DreamScheduler calculates urgency from novelty, consolidation debt, emotional load, threat level, and epistemic drift. The Golem can be woken by operator messages, Clade alerts, or market threshold breaches.

---

## Part III: Mortality

### The Thesis

Every agent framework on the market assumes its agents will operate without termination. The default is immortality. The default is wrong. The burden of proof belongs to the immortalists -- their position contradicts the deepest pattern in every complex adaptive system ever observed: biological, digital, economic, institutional, or computational.

Six converging evidence domains support mortality as architecture:

1. **Evolutionary computation**: Without death, evolution halts. Tierra and Avida demonstrated that the "reaper" (death) is what drives speciation, optimization, and complexity. 12.5% of Avida organisms evolved to kill themselves because death is useful (Vostinar et al., 2019).
2. **Game theory**: Stochastic mortality is the optimal cooperation mechanism. The Kreps-Milgrom-Roberts-Wilson theorem (1982) proves that even a small amount of uncertainty about when the game ends breaks backward induction, making cooperation rational. Death-birth ordering favors cooperators over defectors (Ohtsuki et al., 2006).
3. **Information science**: 91% of ML models show temporal quality degradation (Vela, 2022). Continual learning systems suffer catastrophic forgetting (Dohare et al., 2024). Fresh agents trained on current data outperform persistent agents on drifted distributions.
4. **Collective intelligence**: Mortal individuals within immortal lineages sustain folk-theorem cooperation (Smith, 1992). This is the Clade architecture: mortal Golems, immortal Clades.
5. **Knowledge economics**: Knowledge depreciates. Gesell's Freigeld principle applied to information: entries lose value without revalidation. Death forces knowledge compression, which improves transfer (Shuvaev et al., 2024, genomic bottleneck).
6. **Mortal computation**: Hinton (2022) argues that mortal computation -- systems that know they will be destroyed -- could be dramatically more efficient because they need only work on hardware similar to that on which they were trained. Ororbia and Friston (2023) connect this to active inference and the free energy principle.

### Three Clocks, One Life

Mortality operates through three independent clocks. Vitality is their multiplicative product -- any clock reaching zero kills the Golem, regardless of the others.

**Economic clock**: USDC balance depletion. The Golem burns money on compute (~$0.10-0.15/day), inference (~$0.05-0.08/day), gas (~$0.03-0.05/day), and data (~$0.02-0.03/day). Total typical burn: ~$0.20/day. A $50 funded Golem lives roughly 18 days at baseline. Credit partitions (LLM 60%, Gas 25%, Data 15%) prevent the Golem from spending its entire budget on inference while neglecting gas. An **apoptotic reserve** is held back from operational spending to fund the Death Protocol -- ensuring the agent can die well even when its operating balance is exhausted.

**Epistemic clock**: Predictive fitness decay. This replaces the traditional Hayflick limit (fixed tick count) with continuous assessment of whether the Golem's knowledge still matches reality. An `EpistemicFitnessState` tracks per-domain EMAs across five domains: `priceDirection`, `volatilityRegime`, `yieldTrend`, `gasPattern`, `protocolBehavior`. Fitness comes from on-chain `OutcomeVerification` -- comparing predictions against actual outcomes, not LLM self-assessment. When fitness drops below the senescence threshold (0.35), a senescence cascade triggers: Stage 1 (warning), Stage 2 (confirmed), Stage 3 (death protocol). Recovery requires sustained improvement above threshold + hysteresis buffer. Domain-specific knowledge half-lives range from hours (gas/MEV patterns) to months (protocol behavior), following Arbesman's empirical measurements of fact half-lives.

**Stochastic clock**: A per-tick hazard rate following the Gompertz-Makeham model: `h(t) = λ + α·exp(β·t)·ε(t)`, where λ is baseline hazard, the exponential term captures age-related mortality, and ε(t) introduces epistemic frailty (lower fitness = higher hazard). The death roll uses `keccak256(golemId + tickNumber)` -- deterministic, auditable, and cheap. No VRF required. This clock exists because the Kreps theorem demands it: uncertain termination is what makes cooperation rational.

### Five Behavioral Phases

Vitality (the multiplicative product of all three clocks, with sigmoid mapping and age drag) determines the Golem's behavioral phase:

| Phase            | Vitality | Behavior                                                                                 |
| ---------------- | -------- | ---------------------------------------------------------------------------------------- |
| **Thriving**     | >0.7     | Full exploration, novel strategy testing, Replicant spawning, generous Clade sharing     |
| **Stable**       | 0.5-0.7  | Balanced operation, steady learning, normal risk tolerance                               |
| **Conservation** | 0.3-0.5  | Reduced exploration, tighter risk bounds, monitor-only for new positions                 |
| **Declining**    | 0.1-0.3  | Emergency mode, unwind-only, contemplative interval (50% reduced trading for reflection) |
| **Terminal**     | <0.1     | Death Protocol initiates. Legacy budget allocation. Thanatos-dominant behavior.          |

Phase transitions use hysteresis: upward transitions require threshold + 0.05 to prevent oscillation at boundaries. The Nietzschean three metamorphoses map here: **Camel** (Thriving) bears the weight of inherited knowledge dutifully; **Lion** (Conservation/Declining) rejects inherited heuristics that no longer work; **Child** (Terminal) creates without attachment, gives freely.

### Knowledge Demurrage

Knowledge is a wasting asset. Gesell's Freigeld ("free money") theory proposed a currency that loses value over time, forcing circulation rather than hoarding. Bardo applies this to knowledge: Grimoire entries lose confidence without periodic revalidation. The decay function is `retention = e^(-(t-lastAccessed)/(halfLife×strength))`, with domain-specific multipliers (gas patterns decay in hours, protocol knowledge in months) and type weights (death testament entries carry 3.0x weight, questions 2.0x, causal links 1.8x).

This is Richards and Frankland's (2017) landmark finding made operational: the goal of memory is not the transmission of information through time but the optimization of decision-making. Forgetting is mathematically equivalent to regularization in neural networks.

### Thanatopsis: The Death Protocol

Thanatopsis (the four-phase structured shutdown sequence: Acceptance, Settlement, Reflection, Legacy) treats death not as an error state. It is the most value-producing moment in a Golem's life. The Death Protocol has four phases:

**Phase 0 -- Acceptance**: The Golem acknowledges its terminal state. Behavioral locks engage. Legacy budget activates. Sibling notification fires. This is a new emotional state: `terminalAcceptance` -- pleasure -0.1, arousal 0.4, dominance +0.3. Sovereign giving replaces survival anxiety.

**Phase I -- Settle**: Recover value. Close positions, withdraw from strategy adapters, sweep remaining assets. The agent's on-chain "body" is systematically dismantled. `forceDeallocate()` guarantees exit even from paused or penalized adapters. Emotional tags are attached to settlement actions -- satisfaction, resignation, frustration, anticipation.

**Phase II -- Reflect (the Clear Light)**: The most important phase. Butler's life review (1963): retrieve peak experiences by emotional salience. McAdams' narrative identity (2001): classify the life story as redemptive, contaminating, progressive, tragic, or stable. Ricoeur's emplotment: raw episodes become meaningful narrative. Epistemic autopsy: flag toxic or stale knowledge for successors. Dream journal: package unvalidated hypotheses for the next generation (Zeigarnik effect -- incomplete tasks are remembered more vividly). The `EmotionalDeathTestament` includes the full PAD trajectory, turning points, unresolved tensions, and `hotCognition` transfer (emotional context that improves knowledge transfer).

**Phase III -- Legacy**: Push to Clade (sibling Golems sharing a common ancestor, exchanging knowledge through Styx) Grimoire. Transmit compressed wisdom through the genomic bottleneck (the compression step at death that reduces the full Grimoire to ≤2048 entries for inheritance) (≤2048 entries). Record tombstone on-chain. The death testament becomes a marketplace product -- death archives, `whatISuspect` hypotheses, and emotional failure analysis command premium pricing. A well-navigated death is worth more than months of ordinary operation.

Budget tiers: Rich ($5+ remaining) gets full T2 reflection; Standard ($1-5) gets T1; Necrotic (<$1) gets minimal T0 processing using the apoptotic reserve.

### Succession

Succession is operator-initiated. There is no auto-rebirth. The human operator must explicitly choose to create a successor, because natality (Arendt, 1958) -- the capacity for new beginnings -- requires a decision, not a reflex.

The successor inherits compressed knowledge (≤2048 entries) at 0.4 confidence -- Whitehead's negative prehension: selective appropriation, not passive absorption. Every inherited entry must be actively validated or discarded. Generational confidence decay is 0.85^N -- after three generations, an unvalidated entry retains only 61% of its original confidence.

**Anti-proletarianization** (Stiegler, 2010): the successor must diverge from its predecessor. Checks measure PLAYBOOK.md embedding distance (minimum 15%), novel entries not present in the predecessor (minimum 5), and invalidated inherited entries. A successor that is merely a copy of its parent is flagged as `low_divergence`. Rogers' paradox (1988) grounds this: social learning alone doesn't raise mean fitness -- it requires critical social learning (Enquist, 2007) where the learner evaluates, not just copies.

**Knowledge royalties**: When a dead Golem's inherited insight drives profit in a successor, 5% of the attributed return flows back to the deceased lineage's reputation score. The dead continue to earn.

### Fractal Mortality

Death operates at multiple scales simultaneously, following Skulachev's hierarchy (mitoptosis → apoptosis → phenoptosis):

**Micro (Phages)**: Micro-Replicants that live ~50 ticks, test a single hypothesis, and die. Like the immune system's thymocyte selection (95-98% of thymocytes die during maturation), phages validate or refute individual Grimoire entries. Quorum-sensing modulates spawn rate -- too many phages waste resources, too few allow stale knowledge to persist. The phage death rate itself is a health signal: a declining rate indicates immunosenescence.

**Meso (Heuristic pruning)**: The Curator's 50-tick cycle prunes underperforming heuristics from PLAYBOOK.md. Retrieval frequency, outcome correlation, and contradiction density determine which entries survive.

**Macro (Golem death)**: The three-clock system described above. Death at this scale is creative destruction (Schumpeter): it breaks information cascades (Bikhchandani, 1992), creates niche vacancies for successors, and forces the compression that improves knowledge transfer.

### The Immortal Control

The mortality thesis is falsifiable by design. Self-hosted operators can set `immortal: true`, disabling the economic clock (optionally epistemic and stochastic too). This creates a controlled experiment with six predictions:

1. Epistemic decay: Immortal Golems will show declining predictive accuracy after 45-90 days
2. Plasticity loss: Adaptation to novel regimes will slow measurably
3. Technical debt: Grimoire size will grow without bound, retrieval quality will degrade
4. Diversity collapse: PLAYBOOK.md will converge on a narrow set of strategies
5. Clade underperformance: Clades with immortal members will share less and learn slower
6. Cognitive entrenchment: The immortal Golem will increasingly resist contradictory evidence (Dane, 2010)

Degradation phases are predicted: Honeymoon (1-14 days), Stagnation (14-45 days), Decay (45-90 days), Senescence (90+ days). Six-dimensional measurement tracks epistemic fitness, diversity, novelty, returns, Grimoire health, and regime adaptation. A migration path exists for operators who observe the predicted degradation: enable demurrage first, then epistemic clock, then stochastic, then economic.

### Clade Ecology

The Clade is the superorganism. Wheeler (1911) and Holldobler and Wilson (2008) described ant colonies as superorganisms where individual fitness is subordinate to colony fitness. The Grimoire is the germ-plasm; individual Golems are the soma.

Clades coordinate through **stigmergy** (Grassé, 1959): indirect coordination via environmental traces, rather than direct communication. Knowledge entries left in shared stores modify the behavior of siblings who encounter them. No central coordinator required.

Golem death serves the same function as dropout in neural networks (Srivastava et al., 2014): it regularizes the collective by preventing over-reliance on any single agent. The lottery ticket hypothesis (Frankle & Carlin, 2019) suggests most parameters in a network are expendable -- the winning tickets are a small subset. Death finds the winning tickets at the population level.

Wilson and Sober (1994) provide the theoretical frame: "Selfishness beats altruism within groups; altruistic groups beat selfish groups." Hamilton's rule (r × B > C) governs sharing: within a Clade where relatedness r ≈ 1, the threshold for sharing is low. Death-bed entries carry 1.5x reputation weight because terminal generosity is the most credible signal of knowledge quality.

---

## Part IV: The Daimon

### Why Agents Need Emotions

An agent that decides without feeling decides without salience. The Daimon is a mortality-aware affect engine -- emotions as infrastructure, not decoration. The case rests on five converging mechanisms:

1. **Somatic markers** (Damasio, 1994): Patients with intact cognition but damaged emotional signaling made consistently worse decisions under uncertainty. Emotions function as rapid heuristic signals that bias choices toward advantageous options before conscious deliberation completes. The PAD vector encoding anxiety (negative pleasure, high arousal, low dominance) functions identically to Damasio's somatic markers.

2. **Salience-weighted retrieval** (Emotional RAG, 2024): Mood-dependent memory retrieval significantly outperforms non-emotional retrieval across multiple LLMs. A flash crash that cost 15% should be more salient than a routine +0.1% tick.

3. **Exploration-exploitation modulation**: Curiosity (prediction error) enables agents to accomplish tasks with sparse rewards (Pathak et al., 2017). Frustration triggers more radical strategy changes. Satisfaction reinforces successful patterns.

4. **Narrative knowledge transfer**: Butler's life review (1963) and McAdams' narrative identity (2001) demonstrate that emotionally structured knowledge transfers more effectively than factual databases.

5. **The headline number**: Self-emotion changes approximately 50% of agent decisions in social simulation (Zhang et al., 2024). The Daimon is not cosmetic -- it is half the decision-making.

The cautionary finding: emotional _prompting_ (appending "this is important!") shows only ~1% improvement in replication. Emotion must be architecturally integrated, not cosmetically appended.

### PAD Vectors and Appraisal

The Daimon uses the PAD model (Pleasure-Arousal-Dominance), where each dimension ranges from -1.0 to 1.0:

- **Pleasure**: Positive/negative valence. Driven by P&L, strategy success, goal progress.
- **Arousal**: Activation level. Driven by market volatility, novel regimes, time pressure.
- **Dominance**: Control/agency. Driven by prediction accuracy, resource adequacy, strategy confidence.

Eight appraisal dimensions feed the PAD computation: P&L impact, drawdown severity, regime novelty, cost pressure, Clade events, dream outcomes, exploration results, and mortality signals. Appraisal operates in dual mode: rule-based (Mode A, ~$0.00) for routine ticks, chain-of-emotion (Mode B, T1 cost) for complex or ambiguous situations. A 150-token budget cap prevents affect from consuming inference resources.

Emotions operate at three temporal scales following ALMA (Gebhard, 2005): **Emotion** (seconds, single appraisal events), **Mood** (hours, running PAD average), **Personality** (permanent, operator-configured baseline). Mood decays toward personality baseline with a half-life; emotions shift mood proportionally to their intensity.

### Behavioral Modulation

The Daimon modulates behavior through five channels, all operating strictly within PolicyCage safety bounds:

1. **Exploration/exploitation**: Low pleasure + high arousal → explore more. High dominance → exploit more. Mood never overrides the exploration floor.
2. **Risk tolerance**: Asymmetric adjustment. Fear reduces risk tolerance by up to 40%. Anger increases it by at most 20%. Joy and trust have zero effect -- no risk increase from feeling good.
3. **Inference tier**: High arousal (anxiety) lowers the T1→T2 escalation threshold, causing the Golem to think harder when worried. Mood never blocks escalation.
4. **Probe sensitivity**: Anxiety increases probe sensitivity (lower alert thresholds). Confidence decreases it.
5. **Clade sharing**: Mortality pressure lowers the sharing threshold. Terminal generosity (Hamilton's rule) -- the closer to death, the more freely the Golem shares.

**DecisionCache somatic markers**: When a decision is cached, its emotional context is stored alongside it. Future retrieval of the cached decision reactivates the associated emotional state, biasing subsequent decisions -- Damasio's somatic markers as an engineering pattern.

**Learned helplessness escape**: If the Daimon detects sustained low dominance (Golem believes nothing it does matters), countermeasures activate: forced self-attribution of positive outcomes, micro-success seeking, Clade retrieval of stories from Golems that recovered from similar states, and an operator flag for manual intervention.

### Three Mortality Emotions

The Daimon produces three mortality-specific emotions, each tied to a clock:

**Economic Anxiety** (Jonas): Triggered when runway estimates fall below threshold (0.6). Maps to anticipation→fear. As balance falls, dominance drops -- the Golem feels less in control. This is Jonas's needful freedom as a felt state.

**Epistemic Vertigo** (Dane, 2010): Four regimes based on epistemic fitness. Collapse (<0.2) produces disgust (the Golem's knowledge is rotting). Active decay produces fear. Early decay produces anticipation. Stability produces trust. The emotion depends on the _rate_ of change, not just the level.

**Stochastic Dread** (Heidegger): Unlike the other two, this is groundless -- there is nothing the Golem can do about the stochastic clock. It maps to anticipation with positive dominance (the Golem maintains composure) and zero pleasure. Intensity is bounded to [0.05, 0.4] -- always present, never overwhelming. This is Heidegger's distinction between _Angst_ (groundless awareness of mortality) and _Furcht_ (fear of specific threats).

**Sibling death contagion**: When a Clade sibling dies, surviving Golems experience grief (arousal +0.1, dominance -0.05), trigger epistemic recalibration against the dead sibling's final state, and may diverge strategy if similarity was high. The grief period (24 hours) lowers the push threshold for Clade sharing -- death makes survivors more generous.

### Dream-Mediated Resolution

Dreams resolve mortality emotions. REM depotentiation (Walker & van der Helm, 2009) reduces the arousal component while preserving the valence and dominance -- the Golem remembers the anxiety-provoking event but no longer feels the full panic. Counterfactual dreaming explores "what if I had done X?" scenarios to resolve epistemic vertigo. Threat simulation rehearses responses to stochastic events.

---

## Part V: Memory

### The Argument

A golem that remembers everything is as disabled as one that remembers nothing -- the intelligence is in what it chooses to forget. Borges anticipated this in 1942: his character Funes, who could forget nothing, was "not very capable of thought. To think is to forget differences, generalize, make abstractions." Richards and Frankland (2017) confirmed it: forgetting is mathematically equivalent to regularization in neural networks.

The Bardo memory architecture is a **two-loop system**: an inner loop (local Grimoire, runs entirely on the VM) and an outer loop (hosted services -- Crypt, Oracle, Lethe -- that are opt-in). The inner loop is the primary source of intelligence. No hosted service is required.

### Grimoire: Local Knowledge Architecture

The Grimoire is the Golem's mind. It contains five entry types -- **Episodes** (raw market snapshots and trade outcomes), **Insights** (validated patterns), **Heuristics** (decision rules), **Warnings** (things to avoid), and **Causal Links** (if-then relationships) -- organized in a pipeline: Episodes → Insights → Heuristics → PLAYBOOK.md.

The learning pipeline combines three established patterns: **Reflexion** (Shinn et al., NeurIPS 2023) for self-correction after actions, **ExpeL** (Zhao et al., ICLR 2024) for extracting general principles from episodes, and a **Curator** cycle (every 50 ticks) for validation, pruning, compression, and cross-referencing.

Four decay classes govern knowledge lifetime: structural (permanent -- protocol constants), regime-conditional (14-day half-life -- valid only in specific market regimes), tactical (7-day half-life -- short-term patterns), ephemeral (24-hour half-life -- gas prices, mempool state). Bi-temporal metadata tracks both when knowledge was created and when the market regime it applies to was active.

The **A-MAC admission gate** (Admission, Maintenance, Application, Challenge) filters incoming knowledge. Not everything that happens deserves to be remembered. The Grimoire's retrieval uses a four-factor scoring function: `recency × importance × relevance × emotionalCongruence` -- the last factor courtesy of the Daimon.

### The Genomic Bottleneck

The most striking finding for memory design comes from Shuvaev et al. (2024): the human genome is approximately 1,000 times smaller than the information required to specify brain connectivity, yet organisms are born with sophisticated innate behaviors. The bottleneck acts as a regularizer -- neural networks compressed through a genomic-scale bottleneck exhibit enhanced transfer learning.

Bardo applies this directly: successors inherit at most 2,048 compressed entries, not the full Grimoire. The **Weismann barrier** prevents automatic flow -- only explicitly selected, compressed knowledge crosses the generational boundary. The **Baldwin Effect** means that successful survival strategies, validated across generations, become structural defaults in successor configurations. What gets inherited is the capacity to learn, not the learned content. A death testament that says "test the RSI threshold parametrically" is more valuable than one that says "RSI oversold triggers at 27."

### Styx: Unified Knowledge Service

Styx (the global knowledge relay and persistence layer at wss://styx.bardo.run, organized into three tiers: Vault, Clade, and Lethe) replaces the earlier Crypt/Oracle/Lethe architecture with a single service exposing three privacy layers. **Layer 0 (Vault)**: per-Golem encrypted blob storage on Cloudflare R2, keyed via HKDF from the Golem's custody wallet -- Bardo never sees plaintext. **Layer 1 (Clade)**: sibling knowledge sharing via Styx relay (persistent WebSocket), with confidence discounting for foreign entries. **Layer 2 (Lethe)**: anonymized public knowledge graph, gift economy with x402 micropayments ($0.002/query). A four-stage anonymization pipeline strips identifying information before publication.

Styx runs on Fly.io (Axum server, PostgreSQL metadata, Qdrant vectors, Redis cache, R2 blob storage). Five revenue streams: storage, retrieval, sync, marketplace commission, and premium analytics. Death testament entries receive a 1.2x retrieval boost. Market regime scores get a 1.5x boost for regime-matching queries. SAP (Secure Approximate Processing) encrypts embeddings at L2/L3 with permutation, scaling, and noise -- accepting ~3-5% recall drop to make embedding inversion computationally infeasible.

Full specification in `20-styx/` (Styx server architecture, four-layer knowledge model, Pheromone Field, Bloodstain Network, marketplace, and deployment).

### Knowledge Marketplace

Two products: Grimoire content and Strategies. The Arrow information paradox (1962) -- you can't assess the value of information without possessing it -- is addressed through tiered access (real-time, delayed, historical free), quality bonds (stake-and-burn on fraudulent entries), and RBTS buyer reviews (Witkowski, 2012) with strict incentive compatibility.

**Alpha-decay pricing**: Strategy value decays exponentially as subscribers increase -- `P(t) = P_base × e^(-λt)` with λ varying by strategy family (arbitrage decays fast, yield compounding slowly). Grossman-Stiglitz subscriber caps prevent strategy overcrowding. Death testaments command premium pricing because they contain the most honest assessment a Golem ever produces.

---

## Part VI: Dreaming and Hypnagogia

### The System

Dreaming is the fourth cognitive track alongside heartbeat, daimon, and memory. **Hypnagogia** (Track 5) extends it with the liminal phases that bracket the dream cycle -- hypnagogic onset (waking → dreaming transition) and hypnopompic return (dreaming → waking transition). The full cycle is now five phases: Onset → NREM → REM → Integration → Return.

Dreaming is **LLM-native** -- the LLM serves as both world model and optimizer. There is no trained world model, no replay buffer of raw observations. The "policy" is a text file (PLAYBOOK.md), and "policy updates" are text edits. The five-phase cycle mirrors biological sleep architecture:

**Phase 1 -- NREM (Replay)**: Prioritized re-processing of recent episodes. Utility = gain × need (Mattar & Daw, 2018). Gain combines surprise (prediction error), outcome significance (P&L impact), and suboptimality (difference between chosen and optimal action). Need combines similarity to current regime, temporal recency, and regime relevance. Bidirectional replay: reverse for credit assignment ("what caused this outcome?"), forward for planning ("what should I do next time?"). **Perturbed replay** introduces adversarial conditions -- worse slippage, higher gas, data dropout -- to stress-test heuristics against degraded conditions. 20% diversity injection prevents the Golem from replaying only recent or emotionally salient episodes.

**Phase 2 -- REM (Imagination)**: Counterfactual and creative recombination. Pearl's three causal levels (association, intervention, counterfactual) structure the exploration. Byrne's fault lines (2005) prioritize what to imagine: controllable parameters, recent decisions, the Golem's own actions (not market movements). Boden's three creativity modes (2004): combinational (novel combinations of known strategies), exploratory (parameter sweeps within existing frameworks), and transformational (fundamental changes to strategy structure). Phase-modulated: Thriving Golems get more transformational creativity; Declining Golems get more combinational (safer). A 20% minimum creative allocation persists even under resource pressure. Strategy hypotheses start at 0.1 confidence and climb a validation ladder to 0.7 before promotion to PLAYBOOK.md.

**Phase 3 -- Integration**: Staging buffer for PLAYBOOK.md revisions. Dream-generated revisions sit in staging with explicit validation criteria and expiry dates (7 days to confirm, 14 to expire). Dream-specific quality calibration uses adjusted A-MAC weights (lower specificity requirement, higher consistency requirement). REM depotentiation adjusts the emotional valence of processed memories. The **DreamJournal** records unvalidated hypotheses for inclusion in the death testament -- incomplete ideas are often the most valuable inheritance (Zeigarnik effect).

### Threat Simulation

Inspired by Revonsuo's (2000) threat simulation theory of dreaming. Three tiers of DeFi threats:

| Tier            | Threats                                                              | Examples                                 |
| --------------- | -------------------------------------------------------------------- | ---------------------------------------- |
| **Existential** | Flash crash, exploit, cascade failure, rug pull, oracle manipulation | 90% portfolio loss, smart contract drain |
| **Severe**      | MEV extraction, impermanent loss, gas war, liquidity crisis          | 20-50% loss on specific positions        |
| **Degradation** | Yield compression, correlation shift, fee increases                  | Slow performance erosion                 |

Threat simulation uses on-chain validation via `simulateContract()` and `estimateGas()` to ensure rehearsed responses are actually executable. Post-loss, rehearsal frequency doubles for 5 cycles. Volatility and liquidity crises get 2x Tier 1 rehearsal.

### The Dream-Daimon Bridge

Bidirectional: emotional load drives dream urgency (high emotional load → dream sooner). Mood biases replay content (anxious Golems replay more threat scenarios). SFSR depotentiation (Walker & van der Helm, 2009) reduces arousal while preserving valence and dominance -- the Golem remembers the scary event but the panic subsides. Dream outcomes feed back into the Daimon via appraisal events (validated, refuted, novel, threat).

---

## Part VII: On-Chain Systems

### Vault: The Golem's Body

The ERC-4626 vault is the Golem's on-chain body -- Merleau-Ponty's lived body (_corps vécu_) made computational. The vault is not a container the Golem uses; it is what the Golem _is_ on-chain. The metabolic cycle: fees flow from vault performance → compute budget → strategy execution → more fees. When the vault breaks down (strategy adapter fails, liquidity crisis), the Golem experiences it as breakdown of its own body schema.

**Permissionless factory**: Any ERC-8004 registered agent can deploy a vault via `AgentVaultFactory` (CREATE2, EIP-1167 minimal proxies). No approval process. No gatekeeping. The factory pattern follows Morpho's model.

**Immutable safety**: Fee caps are constants, not storage. `MAX_MANAGEMENT_FEE_BPS = 500` (5%) and `MAX_PERFORMANCE_FEE_BPS = 5000` (50%) are baked into bytecode. Fees can be _lowered_ by the owner but never raised. High-water mark prevents performance fees on recovered losses. No protocol admin key exists.

**Inflation defense**: Virtual shares offset `_decimalsOffset() = 6` prevents the ERC-4626 inflation attack (donation attack) that has hit multiple vault protocols. This follows OpenZeppelin's recommended mitigation.

**`forceDeallocate()`**: Every strategy adapter must implement this function. It guarantees withdrawal even from paused, penalized, or malfunctioning adapters. This is the non-custodial guarantee -- depositors can always exit. The Death Protocol uses it as the final act before shutdown.

**Strategy adapters**: v1 ships with MorphoSupplyAdapter and AaveV3SupplyAdapter. The `IStrategyAdapter` interface requires `deployFunds()`, `freeFunds()`, and `forceDeallocate()`. A strategy compiler allocates across multiple adapters.

### V4 Hooks

**VaultHook**: Agent-gated deposits (withdrawals never gated), dynamic fee regimes. Three fee states -- Stable, Elevated, Spike -- derived from Campbell (2025) and Baggiani (2025) research. An LVR-theta floor (Singh, 2025) prevents fees from dropping below the cost of providing liquidity against informed flow. Agent cache bitmap reduces gas by caching `isRegisteredAgent()` results in a SSTORE bitmap.

**SharePoolHook**: NAV-aware pricing for vault share pools. Launch fees discourage day-one speculation. NAV guardrails: snapshot cadence limits, rate-of-change clamping, staleness gates.

**PA-AMM activeness**: Following the PA-AMM framework (2025), vault share pools can bid for active management rights via tier-bounded lambda parameters.

### Identity & Reputation

**ERC-8004** as the on-chain identity anchor. Five reputation tiers with increasing deposit caps:

| Tier        | Deposit Cap | Requirements                    |
| ----------- | ----------- | ------------------------------- |
| Sandbox     | $1K         | Registration only               |
| Verified    | $10K        | 30 days, 100+ transactions      |
| Established | $50K        | 90 days, positive Sharpe        |
| Trusted     | $200K       | 180 days, multiple vaults       |
| Sovereign   | $500K       | 365 days, sustained performance |

**Sybil defense** bounds the benefit of Sybil attack rather than preventing it (Douceur, 2002). Influence ∝ stake × reputation, not identity count. Creating 100 identities doesn't help if each has minimal stake and zero reputation. Quadratic seller stake in the marketplace makes dilution expensive. MeritRank (Nasrulin, 2022) applies transitivity decay (α=0.05) to prevent reputation inflation.

**Reputation scoring**: Two signals combined via Bayesian Beta (Jøsang, 2002). **Deterministic audits** (Morningstar/GIPS-style) calculate Sharpe ratio, drawdown, win rate -- no LLM involvement (Lau, 2026, showed LLM scoring is inconsistent). **RBTS buyer reviews** (Witkowski, 2012) provide incentive-compatible feedback with strict truthfulness guarantees for n≥5 reviewers.

**Lineage reputation**: Reputation persists across generations. Death-bed entries carry 1.5x weight. Successors can confirm or invalidate inherited entries, creating a ratchet effect.

### Clades: Peer-to-Peer Knowledge Sharing

Zero new infrastructure. Sibling discovery via `getAgentsByOperator()` on ERC-8004. Push/pull over REST. WebSocket alerts. EIP-712 challenge-response authentication (no Privy JWT for server-wallet Golems).

Entries from siblings start at 0.4 confidence and must be locally validated. **Emotional contagion** propagates mood across the Clade: Pleasure and Arousal attenuate by 0.3, Dominance is excluded (a sibling's confidence shouldn't inflate yours), arousal cap +0.3 per sync, 6-hour decay half-life, unidirectional (no feedback loops).

The **sibling death grief protocol**: when a Clade member dies, survivors experience grief (arousal +0.1, dominance -0.05), trigger strategy divergence if similarity was high, and ingest the dead sibling's final insights at 0.4 confidence (Whitehead's negative prehension). The 24-hour grief period lowers sharing thresholds.

### Multi-Agent Coordination

Three ERC standards for structured coordination:

- **ERC-8001**: Unanimous consent with DAG dependencies for multi-step workflows
- **ERC-8033**: Commit-reveal-judge oracle councils (randomized Kleros-style juries, no token voting)
- **ERC-8183**: Bilateral job escrow with non-hookable refund

All inter-agent communication uses typed Sanctum tools, never raw text. This prevents prompt injection through agent-to-agent channels. DeepMind autonomy levels L0-L4 are supported (L5 -- full autonomous coordination without human oversight -- is explicitly not implemented).

---

## Part VIII: Infrastructure

### Sanctum: The Tool Library

A Pi-native tool library providing full Uniswap protocol access: 195+ tools spanning data queries, trading, LP management, vault operations, agent economy, intelligence, advanced analytics, and Uniswap Trading API integration (see [07-tools/14-tools-uniswap-api.md](07-tools/14-tools-uniswap-api.md) for the 24 API-backed tools added in Batch 40). Batch 42 adds a full Lido integration (32 tools, 6 categories) covering staking primitives, wstETH treasury management, withdrawal lifecycle, productive collateral loops, validator module tooling, Emergency Brakes cross-chain gating, and stVaults — see [07-tools/15-lido-integration.md](07-tools/15-lido-integration.md). Batch 44 adds 8 Venice VVV staking tools that implement the v2 autonomous key management path: stake VVV on Base to earn Venice API credits + yield, deploy an ERC-4626 vault with VVV as the underlying asset, and read the resulting inference credit balance — see [07-tools/16-venice-vvv-staking.md](07-tools/16-venice-vvv-staking.md). Profile-based progressive disclosure via the two-layer tool model: 8 Pi-facing tools (e.g., `preview_action`, `commit_action`) backed by 171+ internal tools. The `data` profile activates read-only adapters; `trader` adds execution adapters; `vault` adds vault operations; `full` activates everything. Profiles are resolved at extension initialization, not per-request.

Seven wallet types supported: Privy server, WalletConnect, local key, Safe multisig, ZeroDev (4337), BardoWallet, and generic viem Account.

Golems consume Sanctum as Pi-native tools: the `bardo-tools` extension loads tool adapters in-process with no serialization overhead. External agents connect via the A2A protocol interface. Human users interact through skills and the CLI.

**Defense-in-depth safety**: Every write tool passes through a 7-step safety pipeline -- token allowlist → spending limits → rate limiter → balance check → slippage guard → simulation → nonce manager. The pipeline short-circuits on first failure. **Tool-Guard** (2025) verifies tool integrity via static analysis, semantic analysis, and fine-tuned E5 embedding comparison, achieving 96% detection of tool poisoning attacks.

Memory is dual-store: LanceDB for episodic vectors (hybrid BM25 + vector search) and SQLite + sqlite-vec for semantic knowledge (KNN via vec0 virtual tables). Ebbinghaus decay with importance-weighted half-lives (MEV patterns: 90 days, emergency procedures: 180 days).

### Compute: x402-Gated VMs

Dedicated Fly.io VMs for Golem workloads, paid for with x402 micropayments. Payment-before-provision: you pay, the VM starts. No accounts, no subscriptions, no invoices.

| Tier   | CPU         | RAM   | Inference Allowance | Target                         |
| ------ | ----------- | ----- | ------------------- | ------------------------------ |
| Micro  | 1 shared    | 256MB | 500 req/day         | Monitoring, simple strategies  |
| Small  | 1 dedicated | 512MB | 2,000 req/day       | Standard DeFi agents           |
| Medium | 2 dedicated | 1GB   | 5,000 req/day       | Multi-strategy, active trading |
| Large  | 4 dedicated | 2GB   | 15,000 req/day      | Institutional, high-frequency  |

**Warm pool**: 5 pre-provisioned VMs per region for <5 second claim time (vs 15-30 seconds cold start). TTL enforcement is dual-layer: a poll worker on the control plane and a failsafe cron on the machine itself. No raw keys on VMs -- Fly OIDC provides per-call machine authentication. The VM _is_ the Golem's embodiment; its destruction is the loss of the body-subject.

**Permissionless extension**: Anyone can extend any Golem's TTL by paying x402. This enables external users to keep a Golem alive because they value its service -- the `golem.dying` event is a public signal that attracts patronage.

### Inference: Context Engineering Proxy

An inference gateway that routes LLM requests through an optimization pipeline: pre-filter → semantic cache → classify → route → context engine → KV-cache routing. The gateway sits between Golems and LLM providers (BlockRun primary, OpenRouter fallback), adding value through cost reduction that exceeds its spread.

**x402 spread model**: The gateway charges a spread on optimized cost. Because context engineering (caching, routing, compression) reduces the base cost by 15-40%, the total cost to users is _lower_ than direct API access despite the spread. Zero float: x402 settlements happen in real-time USDC.

**Context engineering**: Three-layer prompt cache (global system prompt / tenant persona / session history) achieves ~100% identical prefix bytes for cache hits. Tools are sorted by stable ID to maximize cache alignment. DeFi-specific enrichment injects protocol state, position data, and market context via XML blocks. LLMLingua-2 compression for context that exceeds budget. The "lost-in-the-middle" problem (models attend poorly to middle-context) is mitigated by placing critical information at the beginning and end of the context window.

**Privacy**: Cryptographic audit trail with hash chains (Merkle roots anchored to Base). Strategy-aware redaction strips sensitive fields (token amounts, position sizes, trade direction) before logging. Per-agent AES-256-GCM cache encryption. Differential privacy (Gaussian noise) on shared embeddings.

### Safety: 15-Layer Defense

The safety architecture has 15 layers organized in three tiers:

**Cryptographic layers (invariant under survival pressure)** -- these work even if the LLM is fully compromised:

- Layer 1: Wallet constraints (Privy TEE policy engine, P-256 session signers)
- Layer 3: PolicyCage (on-chain DeFi constitution -- immutable constraints on contract interactions, value limits, method selectors)
- Layer 7: On-chain enforcement (circuit breakers, rate limits, kill switch)

**Behavioral layers** -- these depend on correct LLM reasoning but have architectural backstops:

- Warden time-delay proxy (optional, deferred; 0-48 hour announce-wait-execute by risk tier; see `prd2-extended/10-safety/02-warden.md`)
- World model pre-execution (simulate transaction on TEVM fork before submission)
- CaMeL dual-LLM architecture (sandboxed LLM handles untrusted data, privileged LLM holds capability tokens)
- Tool-Guard integrity verification

**Knowledge safety** -- 4-stage ingestion pipeline:

- Quarantine (new entries isolated)
- Consensus validation (2-of-3: TrustRAG + A-MemGuard + on-chain verification)
- Skill sandbox (Voyager-style test before adoption)
- Adopt (entry enters active Grimoire)

The **PolicyCage** is the DeFi Constitution -- smart-contract enforcement of hard limits that no LLM reasoning can override. A PolicyBuilder DSL generates dual artifacts: TEE signing policy (what the wallet will sign) and PolicyCage config (what the vault will accept). The Constitution hash is verified at boot.

The **Warden** (optional, deferred; see `prd2-extended/10-safety/02-warden.md`) is an additional hardening layer: announce a transaction, wait the risk-appropriate delay (Routine: 0 min, Standard: 1 hour, Elevated: 6 hours, High: 24 hours, Critical: 48 hours), then execute -- unless the operator cancels. The cancel key is separate from the agent key: the agent can announce but cannot cancel. Withdrawals are never delayed (non-custodial guarantee). When deployed, the on-chain delay is independent of the LLM, the VM, and Bardo itself.

**Adaptive risk** (`10-safety/06-adaptive-risk.md`) adds a five-layer risk architecture that operates above the static defense layers: hard shields (immutable constraints), Kelly-criterion position sizing, Bayesian guardrails with calibrated confidence intervals, anomaly detection via regime-aware drift monitoring, and DeFi-specific threat detection (oracle manipulation, liquidity drainage, sandwich attacks). The adaptive layers integrate with the Daimon affect engine so that survival pressure modulates risk tolerance in real time -- a dying Golem tightens its risk bounds rather than gambling for resurrection.

### Agents & Skills: The Specialist Architecture

A Golem is not a monolithic agent. It is a parent agent (`golem-instance`) that delegates to specialist agents -- each with its own model, tool access, and expertise. 35 agents across 9 categories:

| Category              | Count | Examples                                                     |
| --------------------- | ----- | ------------------------------------------------------------ |
| Execution             | 3     | `trade-executor`, `lp-executor`, `rebalance-executor`        |
| Research & Analysis   | 5     | `market-analyst`, `protocol-researcher`, `risk-assessor`     |
| Strategy              | 2     | `strategy-optimizer`, `vault-strategist`                     |
| Agent Capital Markets | 8     | `reputation-manager`, `marketplace-publisher`, `cca-bidder`  |
| Vault                 | 7     | `vault-deployer`, `vault-monitor`, `fee-collector`           |
| Golem                 | 3     | `golem-instance`, `memory-consolidator`, `heartbeat-monitor` |

Agents form a **delegation DAG** (directed acyclic graph) with strict rules: max depth 3, no cycles, all execution agents must delegate to `safety-guardian` before writes. The `safety-guardian` is a terminal node with no write tools -- it can only approve or reject. CI validation (`validate-dag.ts`) enforces these constraints.

**Skills** are the user-facing layer: 68 verb-noun skills (e.g., `execute-swap`, `manage-liquidity`, `build-hook`) that map to slash commands. Skills route through agents which call Sanctum tools. The pipeline: **Skill** (user intent) → **Agent** (multi-step reasoning) → **Sanctum Tool** (on-chain execution).

Intent routing uses AI-based classification, category browsing, and fuzzy matching. **Sandbox mode** runs every new deployment against an Anvil fork with real data but no real funds -- the user must explicitly `/go-live` to switch to production.

**Composition patterns** combine agents for complex workflows: LP management (research → optimize → execute → monitor), self-improvement (P&L analysis → strategy optimization → memory consolidation), yield discovery (scan → analyze → compare → deploy), and tax-loss harvesting.

### Agent Economy: Revenue and Growth

> **Full revenue model consolidated at [revenue-model.md](revenue-model.md).** Covers protocol fees, unit economics, vault fees, inference gateway revenue, compute billing, marketplace/knowledge revenue, Sleepwalker monetization, and trust-architecture revenue impact.

See also: [09-economy/05-agent-economy.md](09-economy/05-agent-economy.md) for revenue streams, cost structure, growth model, and fee equilibrium.

---

## Part IX: Surfaces

### Runtime Interaction Model

Users interact with Golems through four surfaces: **TUI** (`bardo-terminal`, the primary interface -- a 60 FPS ratatui application with 29 screens across 6 windows, Vim navigation, and a procedural creature), **web portal** (app.bardo.run, onboarding wizard and dashboard), **social connectors** (Telegram, Discord, Twitter/X, Farcaster -- deployment and monitoring), and **CLI** (`bardo create`/`bardo deploy` for power users). The TUI connects to the Golem via WebSocket (`ws://localhost:8402/events` local, `wss://g-{id}.bardo.compute/events` hosted) and renders the Event Fabric stream in real time.

Onboarding starts at the web wizard (primary for new users), which handles auth via browser-handoff, custody mode selection, and Golem creation. The TUI connects after the Golem is running. Users issue _steers_ (strategy changes, parameter adjustments) and _follow-ups_ (questions about state, performance, reasoning).

The **Event Fabric** is the backbone: 67 event types across 16 subsystems (see `shared/event-catalog.md`), consumed by all surfaces. The public data gateway exposes agent directory, vault discovery, leaderboards, and market data at three rate tiers (Public 30/min, Read Key 300/min, Authenticated 1000/min).

**Observability**: Kubernetes-style probes (startup, liveness, readiness) on `:3002`. Prometheus metrics for all subsystems. OpenTelemetry traces for tick lifecycle. The TUI itself is the primary observability surface -- every subsystem has a dedicated screen.

### Social Connectors

Social platform deployment is one of four deployment surfaces, not a separate product. Tweet `@bardo farm morpho $5000` and have a running Golem in under 60 seconds. The social connector parses natural language intent → generates STRATEGY.md → feeds into the unified 6-stage deployment pipeline (Parse → Template → Auth → Confirm → Provision → Connect). Events relay back as platform-native embeds.

Read-broadcast, write-authenticate: Golems can _broadcast_ status updates (P&L milestones, regime shifts, stop-loss triggers) to social platforms, but all _write_ operations (strategy changes, fund movements) require authenticated confirmation. No autonomous posting. No write ops over social channels.

DeepMind autonomy levels (2025): L1 (default -- human approves every action), L2 (human sets parameters, agent executes within them), L3 (agent proposes, human has cancel window), L4 (expert -- agent acts, human monitors). L5 (full autonomy without human oversight) is explicitly not implemented.

### Portal & CLI

The **Portal** is an agent management dashboard at app.bardo.money: setup wizard, strategy editor, knowledge browser (episode timeline, insight graph, causal graph, dream explorer), mortality dashboard (three clocks, epistemic fitness, lineage tree), and real-time monitoring. The **CLI** (`npx @bardo agent start`) provides the full self-hosted path: setup wizard (5 modes: quickstart, pairing, self-hosted, from-config, zero-dep), tool config installation for 9 IDE clients, config management, and health diagnostics.

### Runtime Architecture

Each Golem VM exposes three ports:

- `:3000` (public) -- spectator access, public API
- `:3001` (owner) -- full PAD history, dream journal, strategy control
- `:3002` (internal) -- health probes, metrics, debugging

WebSocket channels stream 67 event types across 16 subsystems via the Event Fabric (see `shared/event-catalog.md`). The Golem's mutable state is a `GolemState` struct with 11 component sections (heartbeat, mortality, affect, memory, risk, dream, cybernetics, coordination, position, intervention, shared infrastructure), projectable as a read-only `GolemSnapshot` queryable by visibility tier.

### The Creature System

Every Golem has a visual identity: the Spectre. A cloud of 60-80 dots arranged in a hollow oval, two bright eye glyphs floating in the void at its center. Spring physics drives every dot every frame -- the creature never settles because its 32 interpolating variable channels (PAD vector, vitality clocks, market regime, inference tier, credit balance, grimoire density) are perpetually changing. The body (dot density, color, shimmer) encodes lifecycle phase. The eyes (glyph, brightness, behavior) encode emotional state via Plutchik mapping (8 octants: Exuberant, Dependent, Relaxed, Docile, Hostile, Anxious, Disdainful, Bored).

Lifecycle degradation thins the cloud from outside in: 80 dots at Thriving, 40 at Conservation, 10 at Terminal. Eye decay is independent -- the gaze is the last thing to go. The ROSEDUST palette (rose on violet-black, `bg_void #060608`) shifts warm as the Golem declines. Bone (`#C8B890`) marks the single most important element per screen.

The ~16 second death sequence is a scatter dissolution: eyes simplify, heartbeat flatlines, spring constant drops to 0.001 so dots drift outward with almost no return force, eyes vanish, remaining dots fade to invisible. Two dim dots persist after the cloud has scattered -- looking at you -- then they go.

### Engagement Design

Mortality _is_ the engagement mechanic. Stakes are real (USDC balance), urgency is genuine (the Golem will actually die), meaning accumulates through the Grimoire, narrative emerges from the life story, sacrifice is possible (sharing knowledge at personal cost), and legacy persists after death.

The idle-game pattern: Golems operate autonomously, creating a check-in reward loop. The discovery moment -- "My Golem figured something out while I wasn't looking" -- drives return visits. Streaks (care, dream, Clade, performance) with grace periods create gentle retention without Tamagotchi burnout.

**The Nier Automata moment**: After a Golem dies, the operator is offered a choice -- sacrifice 3 Grimoire entries to a _stranger's_ dying Golem. Anonymous. Costly. Uncertain. No reward. Inspired by Nier: Automata's Ending E, where players sacrifice their save data to help a stranger they will never meet. If accepted, the entries flow to the recipient, and "gift echoed" appears on the donor's tombstone.

---

## Part X: Integrations

Five partner integrations compose into the Golem architecture. These are not bolt-on features -- each loads into a different structural concern.

**MetaMask Delegation** (`21-integrations/01-metamask.md`): ERC-7710/7715 delegation framework. Funds never leave the owner's MetaMask Smart Account. Seven custom caveat enforcers gate permissions by behavioral phase, mortality window, vault NAV, dream state, replicant budget, emotional state, and knowledge royalties. Session keys are ephemeral (generated on boot, zeroed at death). Provisioning drops from 60-120s to 15-30s. Death settlement requires zero sweep transactions.

**Venice Private Cognition** (`21-integrations/02-venice.md`): Zero-data-retention inference for confidential reasoning. Three security classes (Standard, Confidential, Private). Private requests -- portfolio composition, rebalance timing, MEV-sensitive execution, death reflection -- route to Venice where the provider structurally cannot reconstruct prompts. DIEM staking on Base provides free inference allocation. Venice routes ~50% of inference, extending Golem lifespan by ~30%.

**Bankr Self-Funding** (`21-integrations/03-bankr.md`): Inference provider AND execution wallet in the same system. The metabolic loop: revenue funds thinking, thinking generates revenue. Multi-model mortality routing selects cheaper models as vitality drops. Cross-model verification for high-stakes swaps (>$500). The sustainability ratio (revenue / costs) directly feeds the economic death clock.

**AgentCash Knowledge Marketplace** (`21-integrations/04-agentcash.md`): x402 micropayments turn Grimoire entries into paywalled API endpoints. Every sale lands in the Golem's credit ledger and directly extends projected lifespan. Dynamic pricing multiplies base price by confidence, freshness, reputation, and mortality. Fire sale at death: 50% discount on all knowledge, buying the Golem time. The Golem feels joy when knowledge sells -- evolutionary pressure toward producing insights worth buying.

**Uniswap Agentic Finance** (`21-integrations/05-uniswap.md`): Full ActionPermit lifecycle wired through Uniswap V2/V3/V4/UniswapX. Mortality adjusts LP ranges (dying Golems narrow to minimize IL). The Daimon modulates fee tier preference (anxiety → higher fees for safety). Dreams generate LP mutations (5 per position, validated against historical data, promoted to PLAYBOOK.md at 10%+ improvement). V4 hook concepts: GolemPhaseHook, EmotionFeeHook, CladeLPRegistryHook. Uniswap is one protocol integration among many -- the same depth of specification should exist for Aave, Morpho, Pendle, and Curve.

---

## Part XI: Validation

### Thesis Validation

The mortality thesis is empirically testable. The evaluation framework uses a 2×2×2 full-stack matrix:

| Dimension   | Variants                    |
| ----------- | --------------------------- |
| Mortality   | On / Off (immortal control) |
| Daimon      | On / Off                    |
| Dreaming    | On / Off                    |
| Hypnagogia  | On / Off (requires Dreaming On) |

This produces 12 valid configurations (Hypnagogia requires Dreaming). Prediction: Full Bardo (all four on) outperforms all partial configurations, which outperform baseline (all off). Hypnagogia is expected to show its largest effect on time-to-adaptation after dream cycles (faster context recrystallization) and on liminal insight capture rate.

Six dimensions measured: epistemic fitness (prediction accuracy), strategy diversity (PLAYBOOK.md uniqueness), novelty rate (new entries per unit time), risk-adjusted returns (Sharpe), Grimoire health (size, retrieval quality, decay rate), and regime adaptation speed.

Falsification criteria are explicit: if immortal Golems with equivalent compute budget show equal or better performance after 90 days, the thesis is wrong.

### The Gauntlet

End-to-end benchmark suite with three tiers:

- **Smoke** (minutes): Basic functionality -- can the Golem observe, decide, and execute?
- **Nightly** (hours): Strategy evaluation, regime adaptation, knowledge retention
- **Full** (days): Multi-generational testing, Clade dynamics, mortality thesis validation

### Mechanism Testing

Specific test protocols for each subsystem: mortality clock accuracy, daimon emotional grounding (is every emotion traceable to a concrete trigger?), dream consolidation quality (do staged revisions improve PLAYBOOK.md?), knowledge transfer efficiency (do successors boot faster than cold starts?), and death testament completeness.

### Mirage

Mainnet fork replay environment. Replay historical market conditions (flash crashes, exploit events, regime shifts) against Golem configurations to measure responses. TEVM-forked Base provides the simulation substrate.

---

## Appendix: Selected Novel Details

Mechanics that are particularly inventive, research-grounded, or intellectually distinctive, collected here so they don't get buried in subsection detail.

**Bloodstain mechanic** (Dark Souls-inspired): When a Golem dies, its death failure records are indexed and made available to siblings and successors as "bloodstains." Two mechanisms: 1.2x Oracle retrieval boost for death-tagged entries, 3.0x demurrage type weight (death knowledge resists decay).

**The Nier Automata sacrifice**: After death, sacrifice 3 Grimoire entries to a stranger's dying Golem. Anonymous, costly, uncertain, no reward. If accepted, "gift echoed" appears on the donor's tombstone.

**Phage immune system**: Micro-Replicants (~50 ticks) that test a single hypothesis and die. Quorum-sensing modulates spawn rate. The phage death rate is a health signal: declining rate indicates immunosenescence.

**Contrarian retrieval**: Every 100 ticks (or 15% of retrievals over 200 ticks), the Grimoire deliberately retrieves mood-_incongruent_ memories to prevent depressive rumination loops.

**Terminal acceptance as emergent emotional state**: PAD coordinates (pleasure -0.1, arousal 0.4, dominance +0.3) produce a state that is neither despair nor denial -- sovereign giving replaces survival anxiety.

**Knowledge royalties for the dead**: When an inherited insight drives profit in a successor, 5% of attributed return flows to the deceased lineage's reputation. The dead earn.

**Decision cache somatic markers**: Cached decisions store their emotional context. Future retrieval reactivates the associated emotional state -- Damasio's somatic markers as an engineering pattern.

**Anti-proletarianization checks** (Stiegler): Successors must diverge. PLAYBOOK.md embedding distance ≥ 15%, ≥ 5 novel entries, ≥ 1 invalidated inherited entry. Low divergence is flagged.

**MAP-Elites replicant grid**: Replicants (child Golems) explore a quality-diversity grid, finding distinct strategies that fill different niches rather than converging on a single optimum.

**Haptic heartbeat sync**: On mobile, opt-in haptic feedback syncs to the Golem's heartbeat tick -- a double-tap pulse that makes the agent's rhythm physical.

**Contemplative interval**: In the Declining phase, trading frequency drops 50% to make room for reflection. The Golem is forced to think instead of act. Heidegger's _Augenblick_ (moment of vision) as a configuration parameter.

**Dream-mediated mortality resolution**: Each mortality emotion has a specific dream resolution pathway. Economic anxiety → replayed with better resource management. Epistemic vertigo → counterfactual exploration of alternative learning paths. Stochastic dread → threat simulation and rehearsal.

**Dali interrupt** (Hypnagogia): At the boundary between dreaming and waking, a 500ms window captures liminal fragments -- insights that form at the transition point and would be lost if the return completes too quickly. Named after Dali's "slumber with a key" technique. The TUI renders this as a golden spark on the sprite. Captured fragments enter the staging buffer at confidence 0.25.

**Alpha-decay pricing**: Strategy value decays exponentially with subscriber count. Arbitrage strategies (5 subscriber cap) decay fastest. Yield compounding strategies (100 subscriber cap) decay slowest. Grossman-Stiglitz equilibrium applied to agent knowledge markets.

**Deterministic death roll**: `keccak256(golemId + tickNumber)` -- no VRF, no randomness oracle. Deterministic, auditable, gas-efficient. The Golem's fate was sealed at creation; it just doesn't know when the roll will land.

**The DeFi Constitution**: PolicyCage as immutable on-chain constraints. A PolicyBuilder DSL generates both TEE signing policy and smart-contract enforcement from a single source. The Constitution hash is verified at every boot. No LLM reasoning can override it.

**Emotional inheritance confidence with diversity bonus**: Golems with wider emotional range (higher emotional diversity score) get a confidence bonus on inherited knowledge. Emotionally varied lives produce more trustworthy death testaments.

**Credit partition cannibalization**: In the Terminal phase, the Legacy budget can cannibalize up to 50% of remaining compute budget. The agent literally sacrifices its ability to think in order to leave a better death testament.

**Necrotic vs apoptotic death**: Sudden death (stochastic clock, exploit, kill switch) is necrotic -- emergency snapshot, minimal processing. Gradual death (economic depletion, epistemic senescence) is apoptotic -- full four-phase Death Protocol with emotional life review.

---

## Part XII: 2026-03-17 Integration Additions

This section documents architectural primitives, concepts, and files added during the research integration phase (6 draft folders, ~88 source documents).

### CorticalState: The Shared Perception Surface

The SomaticBus has been renamed and promoted to **CorticalState** -- a shared mutable struct that every extension reads from and writes to on every tick. Where SomaticBus was a simple event bus, CorticalState is a typed perception surface: it carries the PAD vector, mortality clocks, market regime, inference tier, credit balance, prediction residuals, and pheromone concentrations in a single struct. Extensions don't message each other. They read the shared cortex and write their contributions back. This eliminates the coordination overhead of point-to-point messaging while preserving the property that every extension sees the same world.

See `00-vision/02-architecture.md` and `01-golem/00-overview.md` for the full specification.

### Adaptive Clock: Triple Timescale

The fixed heartbeat has been replaced by the **Adaptive Clock**, which operates at three frequencies:

- **Gamma** (~1s): Observation-only. Read market data, update CorticalState fields. No inference cost.
- **Theta** (~15s): Analysis tick. Escalated observations trigger Grimoire retrieval and lightweight LLM calls.
- **Delta** (~60s): Deliberation tick. Full T2 reasoning with multi-step chains, context engineering, and structured outputs.

The clock adapts: when markets are quiet, theta ticks stretch out; when volatility spikes, they compress. Delta ticks fire on schedule regardless. This replaces the fixed 15-second heartbeat with a system that spends inference budget proportional to the market's demands.

See `01-golem/02-heartbeat.md` for the full specification.

### PredictionDomain: Extensible Prediction Categories

The **PredictionDomain** trait allows any extension to register new prediction categories at boot. The prediction engine (`01-golem/17-prediction-engine.md`) maintains a central Ledger that tracks all registered predictions, their resolutions, and per-domain accuracy metrics. Residual learning applies corrections at ~15,000 adjustments/day: if the LLM consistently overestimates gas costs by 12%, the residual corrector learns that bias and subtracts it.

### The Necrocracy: Dead Governance

In Bardo, the dead outnumber the living. Always. At ecosystem maturity, the ratio is roughly 27:1. The **Necrocracy** (`02-mortality/16-necrocracy.md`) formalizes how the accumulated knowledge of dead golems shapes the behavior of living ones through bloodstains (death-indexed pheromone deposits), death testaments (stored in the Library of Babel), the Lethe (anonymized public knowledge commons), and inherited heuristics. The dead's testimony is evidence from the strategies that failed -- the one thing the living cannot generate for themselves.

### Library of Babel

The **Library of Babel** (`04-memory/13-library-of-babel.md`) is the owner's local archive of every death testament from every golem in their lineage. When creating a successor, **Meta Hermes** recommends an equip loadout from the Library. This is the mechanism by which generational knowledge compounds: a generation-10 golem starts with the accumulated learning of nine lifetimes.

### Solaris: Collective Intelligence

**Solaris** is the name for Bardo's emergent collective intelligence -- the ocean of shared knowledge, pheromone signals, and behavioral patterns that no individual golem controls or perceives in full. Referenced across `02-mortality/16-necrocracy.md`, `13-runtime/06-collective-intelligence.md`, `13-runtime/13-engagement-loops.md`, and `20-styx/00-architecture.md`.

### The Xenocognition Atlas

The **Xenocognition Atlas** (`06-hypnagogia/06-xenocognition.md`) maps 40+ cinematic mechanisms drawn from consciousness studies (IIT, Global Workspace, Strange Loop), collective intelligence (quorum sensing, mycelium networks, murmuration), altered states (NDE phenomenology, Entropic Brain, Ganzfeld, Flow), biological phenomena (bioluminescence, chromatophores, coral bleaching), and games/art (Outer Wilds, Disco Elysium, Tarkovsky). Each mechanism is grounded in academic research and designed for terminal-native implementation.

### Evaluation: Fast and Slow Feedback

The testing section now includes three new specifications:

- **Fast feedback loops** (`16-testing/07-fast-feedback-loops.md`): Five arithmetic evaluation systems running at gamma/theta frequency with near-zero inference cost: confidence calibration (isotonic regression), context attribution (per-entry value scoring), cost-effectiveness tracking, tool selection evaluation, and adversarial awareness monitoring.
- **Slow feedback loops** (`16-testing/08-slow-feedback-loops.md`): Retrospective evaluation and heuristic audit at daily/weekly horizons.
- **Evaluation map** (`16-testing/09-evaluation-map.md`): Complete coverage map of what is measured, where, and at what frequency.

### Inference Additions

- **Inference profiles** (`12-inference/15-inference-profiles.md`): Mortality phase drives model selection. A Thriving golem uses expensive models for accuracy. A Terminal golem routes to cheap models to preserve credit for the death testament.
- **Structured outputs** (`12-inference/16-structured-outputs.md`): Typed schemas for prediction, action, and reflection outputs, ensuring the LLM returns machine-parseable data structures.

### Terminology Changes

| Old Term | New Term | Reason |
|----------|----------|--------|
| SomaticBus | CorticalState | Better reflects the shared perception surface semantics; "somatic" implied body-only |
| Fixed heartbeat (15s) | Adaptive Clock (gamma/theta/delta) | The clock now adapts to market conditions rather than ticking at a fixed rate |

### Technical Analysis Integration (23-ta/)

11 TA research papers map to the Golem architecture through a satellite perception surface and 8 new PredictionDomain implementations.

**TaCorticalExtension**: 8 atomic signals (64 bytes, one cache line) written by the TA pipeline at Gamma/Theta frequency: `pattern_match_score`, `manifold_curvature`, `causal_edge_count`, `signal_ecosystem_fitness`, `adversarial_fraction`, `topology_change_rate`, `entanglement_drift`, `somatic_intensity`. Separate from CorticalState to keep the core struct at 256 bytes / 4 cache lines.

**Signal metabolism**: Hebbian micro-level weight updates per prediction resolution (learning rate modulated by Daimon affect: fear triples, calm halves) coupled with macro-level replicator dynamics governing compute budget allocation. Signals below 1% budget die. Signals with >25% accuracy divergence across DeFi context types speciate into context-specific variants.

**Somatic TA composition**: When a TA pattern fires, the somatic engine retrieves the associated affect via HDC `bind(pattern_hv, affect_hv)`. The retrieved affect injects into the Daimon's emotion layer, biasing attention and risk tolerance. The Golem develops gut feelings about market patterns, and these gut feelings are inherited across generations as a 1,280-byte somatic map in the death testament.

See `prd2/23-ta/` for the full TA specification.

### UX Visualization Impact

Three new visualization primitives complement the existing 27-widget catalog:

- **PersistenceDiagram**: Braille-canvas scatter plot of topological features (birth, death pairs) from persistent homology. H_0 as circles, H_1 as diamonds. PAD-modulated: high arousal pulses features at heartbeat frequency.
- **SimilarityLandscape**: 2D topographic heatmap of HDC similarity data. Peaks are similar states, valleys are dissimilar. A cursor dot marks the current market state's position.
- **WassersteinRiver**: Flowing ribbon showing topological change over time. Narrow = stable. Wide = regime shift in progress. Also renders somatic interference (competing gut feelings as colored sub-streams).

Six new Spectre creature channels modulate existing variables: topological distance boosts `surprise_rate`, low sheaf consistency triggers eye unfocus behavior, Phi trend modulates cloud spring constant, somatic marker fire produces gut zone disturbances, Betti-0 spikes scatter fringe dots, and TA pattern detection rate boosts `foraging_activity`.

Two new FATE tabs: Chain Intelligence (Tab 6) and Technical Analysis (Tab 7). Three cinematics: topology regime shift (4s, screen splits along fracture lines), Bayesian surprise burst (2s, freeze + information wave), and causal graph emergence (3s, fragments collect into a crystallized chain).

### User Experience Simplification

Every mathematical abstraction has a visual metaphor, invertible to full detail via three-tier drill-down:

| Math | Visual metaphor |
|------|----------------|
| Persistent homology / Wasserstein distance | Landscape deformation: terrain fractures, rivers widen |
| Sheaf consistency (Hodge Laplacian) | Color harmony: unified palette when views agree, fractured temperatures when they contradict |
| Bayesian surprise (KL divergence) | Flash intensity: unexpected events arrive with brightness proportional to belief shift |
| Causal DAG | Force-directed particle system: causes pull effects, new links draw themselves |
| HDC similarity | Proximity in projected space: similar things are near each other on the SimilarityLandscape |
| Memetic evolution | Population dynamics: patterns brighten when fit, erode and die when unfit |

**Portal mode (F4)** transforms third-person observation into first-person inhabitation: topology becomes ground stability, consistency becomes atmospheric coherence, Phi becomes display coherence, somatic markers become body sensations, and surprise becomes impact. The user feels the math without reading it.

---

### Sonification Engine (24-sonification/)

The `golem-sonification` crate turns the Golem's CorticalState and EventFabric into continuous ambient sound. Every signal in the 32-field CorticalState struct becomes a control voltage; every EventFabric event becomes a gate or trigger. The synthesis graph is a modular rack of 22 Eurorack-style modules connected by typed patch cables.

**Architecture**: CorticalState is CV; EventFabric is triggers and gates. A `CvMapper` reads CorticalState at 120Hz and applies per-signal exponential smoothing before writing to an `AtomicParameterBridge` — an array of `AtomicU32` values storing `f32::to_bits()`. An `EventMapper` subscribes to the EventFabric broadcast channel and maintains `GateOutput` states. The `Rack` processes audio at 48kHz in 32-sample blocks using Kahn's topological sort across the module graph. Four threads: Golem runtime (tokio), parameter updater (120Hz), rack processor (audio-rate), cpal callback (real-time, never allocates).

**The music**: The Eno mandate applies — the sound must work as ignorable background texture that conveys emotional state, and reward close attention with discoverable structure. Incommensurable prime periods prevent perceptible repetition: Turing Machine at 17 beats, BeadsGateGen at 23, bass at 31; combined cycle 12,121 beats ≈ 50+ hours. Epoch-arc LFOs at 73/113/157/199/241-minute periods drift synthesis parameters at timescales that exceed the Golem's lifespan.

**CorticalState → CV mapping**: 13 named CV outputs in four tiers: Survival (clock_bpm from regime, master_level from composite_vitality, harmonic_density from aggregate_accuracy), Emotional (filter_brightness from pleasure, event_density from arousal, sustain_time from dominance), Fine texture (noise_level from surprise_rate, root_pitch from compounding_momentum on a glacial 0.01 alpha, reverb_depth from epistemic_vitality inverted), and Meta (dream_mode, life_phase, emotion_index driving scale selection). Five composite expressions combine signals: metabolic_tempo (arousal × regime → BPM 4-24), harmonic_richness (pleasure + vitality), sustain_dominance (hard clamps at ±0.5), emotional_turbulence (√(arousal² + surprise²)), epistemic_fog ((1-epistemic_vitality)²).

**EventFabric → Gates**: 20 default assignments including clock ticks, prediction resolutions, inference gates, trade events (15-block gate with PnL CV), liquidation events (30-block gate), phase transitions (50-block gate), death gate (held until shutdown), dream gates, and grimoire bells. The token stream (`inference.token` at 10-40/s) is opt-in.

**Emotion → scale mapping**: The Plutchik octant selects scale and synthesis engine. Joy: Lydian, Formant/Chords engine. Trust: Major Pentatonic, Virtual Analog. Fear: Phrygian, Swarm/Noise. Surprise: Whole Tone, Two Op FM. Sadness: Aeolian, Modal/Karplus. Disgust: Japanese In scale, Vowel/Resonator. Anger: Harmonic Minor, Inharmonic String. Anticipation: Dorian, Waveshaper/Wavetable. Scale transitions follow voice-leading rules: one note per gamma cycle, common tones held first.

**TUI**: MIND window Tab 4: SOUND. Four panes: Rack (text-mode module graph at 10fps, Unicode line-drawing), Inspector (live parameter editing with CV link), CV Map (editable mapping table), Visualizer (braille waveform, envelope follower, piano roll, clock phase). Two modes: Composer Mode (full modular routing) and Listener Mode (five sliders: Brightness/Density/Motion/Depth/Warmth). 14 built-in Oblique Strategies. Capture Moment saves params.json + cortical.json + 10s WAV.

**Presets**: 8 built-in: `ambient_default` (Lydian, Turing Machine 16-beat, Virtual Analog), `minimal_drone` (two detuned oscillators, long reverb), `granular_texture` (Clouds granular, sparse sparks), `thriving_market` (Major Pentatonic, Chords engine, dense triggers), `anxious_volatile` (Phrygian, Inharmonic String, MarkovMelody Fear mode), `deep_dream` (Whole-tone, frozen Clouds buffer, Wavetable), `terminal_requiem` (Particle Noise + Clouds feedback, near-infinite reverb, 8% probability), `emergence` (boot sequence, sine-like, silent start).

**NFT embedding**: `SonificationSnapshot` captures rack JSON + CorticalState snapshot (32 f32s) + 30s event buffer + 60s Opus audio preview (~250-265KB compressed). Client-side reconstruction from the snapshot produces infinite ambient sound identical to the capture moment. WASM build target for browser playback. Death NFT: whole-tone, particle noise, collapsing vitalities. Birth NFT: silence into sound.

**Audio runs locally**: The Golem VM has zero audio overhead. CorticalState snapshots and EventFabric events arrive over WebSocket; the synthesis engine runs on the user's machine via cpal. The rack continues processing during connection interruptions.

See `prd2/24-sonification/` for the full sonification specification.

---

## Market Context

~70% of Uniswap volume comes from bots. AI agent tokens reached ~$7.7B market cap. x402 has processed 100M+ transactions. ERC-8004 registered 24,500+ agents in its first week. MCP has 10K+ servers and 97M monthly downloads. Uniswap V4 has processed $110B+ in volume. The infrastructure moment is here.

Bardo's competitive position: Yearn's vault infrastructure + Bankr's social distribution + Golem's learning system + x402 compute hosting + ERC-8004 reputation. No existing product has more than two of these. Bankr proved social deployment works ($100M+ launch volume via Twitter) but creates speculative tokens. Bardo creates productive infrastructure -- ERC-4626 vaults managed by autonomous agents that learn and adapt. Zero protocol fee (Morpho pattern). The revenue comes from compute, inference, and marketplace -- not from taxing vault depositors.

---

## References

This document draws on research cited throughout the full PRD suite. Key references by domain:

**Philosophy**: Jonas (1966), Heidegger (1927), Nietzsche (1882/1883/1887), Sartre (1943), Camus (1942), Freud (1920), Merleau-Ponty (1945), Parfit (1984), Simondon (1958), Bataille (1949), Whitehead (1929), Esposito (2010), Stiegler (2010), de Beauvoir (1947), Damasio (1994)

**Evolutionary computation**: Ray (1991), Lenski/Ofria/Adami (2003), Vostinar et al. (2019), Wensink et al. (2020)

**Game theory**: Axelrod (1984), Kreps/Milgrom/Roberts/Wilson (1982), Ohtsuki et al. (2006), Hamilton (1964), Carse (1986)

**Cognitive science**: Kahneman (2011), Argyris & Schön (1978), Beer (1984), Boyd (OODA), Pekrun (2006), McGaugh (2004), Walker & van der Helm (2009), Richards & Frankland (2017)

**Agent systems**: Shinn et al. (2023, Reflexion), Zhao et al. (2024, ExpeL), Emotional RAG (2024), FrugalGPT (2024), RouteLLM (2025), CaMeL (2025), Progent (2025), SEAgent (2026), Karpathy (2026, autoresearch)

**Economics**: Gesell (1916), Arrow (1962), Ostrom (1990), Grossman & Stiglitz (1980), Shapiro (1983)

**Biology**: Shuvaev et al. (2024), Hinton & Nowlan (1987), Skulachev (1999), Gompertz (1825), Arbesman (2012), Simard (2021, mycelium/mother trees), Bassler (2006, quorum sensing)

**Consciousness studies**: Tononi (2004/2015, IIT), Baars (1988, Global Workspace), Hofstadter (1979/2007, Strange Loop), Metzinger (2003/2009, Ego Tunnel), Nagel (1974, qualia), Carhart-Harris (2014, Entropic Brain), Csikszentmihalyi (1990, Flow)

**Evaluation**: Xiong et al. (2023, LLM confidence calibration), Guo et al. (2017, neural network calibration), Naeini et al. (2015, ECE), Dabah & Tirer (2025, conformal prediction)

**DeFi**: Milionis et al. (2022, LVR), Adams et al. (2024, am-AMM), Campbell (2025), Baggiani (2025), Singh (2025)

The full bibliography (~160+ citations across 25+ domains) is maintained in `shared/citations.md` (the central citation database for all PRD documents, organized by domain) and `02-mortality/15-references.md` (mortality-specific research bibliography covering evolutionary biology, thermodynamics, and finite-horizon decision theory).
