# PRD2 COMPREHENSIVE INVENTORY
# ============================
# ~311 files across 24 sections + 5 top-level files
# Updated 2026-03-18 after PRD2 research reconciliation (Wave 5)
# Note: line counts are approximate; concurrent integration may shift totals

> **Reader orientation:** This is the master file inventory for the prd2/ specification suite. It lists every file across all 24 sections with approximate line counts, key concepts, and cross-references. Use this as a map when navigating the ~311-file PRD. Bardo is a Rust runtime for mortal autonomous DeFi agents (Golems); this inventory covers everything from the founding philosophy to the Styx knowledge service deployment spec. `prd2/shared/glossary.md` has full term definitions for any unfamiliar terminology.

================================================================================
TOP-LEVEL FILES (5 files, ~2,880 lines)
================================================================================

FILES:
  SUMMARY.md (788 lines) - Complete system narrative distillation
  INVENTORY.md (this file) - Master file inventory with line counts and cross-references
  prd-summary.md (306 lines) - Architecture reference (crate inventory, layers)
  revenue-model.md (443 lines) - Consolidated revenue streams and projections
  00-narrative-strategy.md (399 lines) - Priority/sequencing guide for implementers

KEY CONCEPTS:
  - Three evaluation pillars: Pay, Trust, Cooperate
  - Additional pillars: Die, Think, Secrets
  - Priority stack (ranked): x402 everywhere > ERC-8004 identity > Death Protocol + Grimoire > Vault factory > 15-layer safety > Social deployment > Clade stigmergy > am-AMM auctions
  - Implementation status tracking: BUILT/DESIGNED/DEFERRED tags per feature
  - Revenue model: 10% on all x402 transactions, ~81% gross margin
  - Break-even: ~6 active vaults (~$120/mo infrastructure)
  - Unit economics: ~$21/vault/month at $10K AUM

IMPLEMENTATION DETAILS:
  - 18-crate Cargo workspace (bardo-golem-rs)
  - Five-layer architecture: Vaults > Golems > Sanctum > Compute > Reputation
  - Revenue streams: vault fees, inference spread, compute, Styx, marketplace
  - Break-even TVL for self-sustaining Golem: ~$400K

CROSS-REFERENCES: All sections referenced via directory map
PRIORITY: CRITICAL - this is the master prioritization document

================================================================================
00-VISION (6 files, ~1,627 lines)
================================================================================

FILES:
  00-bardo.md (142 lines) - Founding narrative, Bardo concept
  01-thesis.md (183 lines) - Core mortality thesis statement
  02-architecture.md (466 lines) - System architecture overview, CorticalState, Adaptive Clock
  03-philosophy.md (160 lines) - Philosophical grounding (Jonas, Heidegger, Necrocracy references)
  04-trust.md (486 lines) - Trust model and safety philosophy
  05-manifesto.md (190 lines) [NEW] - Unified manifesto: three core ideas (prediction error, mortality, visible cognition), Free Energy Principle thesis, evolutionary mortality grounding, 18 academic citations

KEY CONCEPTS:
  - Mortality as architectural feature, not constraint
  - CorticalState as shared perception surface (replaces SomaticBus)
  - Adaptive Clock triple timescale (gamma/theta/delta replaces fixed heartbeat)
  - PredictionDomain trait for extensible prediction categories
  - Agent Capital Markets thesis

CROSS-REFERENCES:
  - 02-architecture.md -> 01-golem/02-heartbeat.md (Adaptive Clock details)
  - 02-architecture.md -> 01-golem/00-overview.md (CorticalState integration)
  - 03-philosophy.md -> 02-mortality/16-necrocracy.md (Necrocracy concept)
  - 03-philosophy.md -> 02-mortality/06-thanatopsis.md (death protocol)

================================================================================
01-GOLEM (24 files)
================================================================================

FILES:
  00-overview.md (1215 lines) - Master Golem specification, CorticalState architecture
  01-cognition.md (1115 lines) - Cognitive architecture, PredictionDomain, tiered inference
  02-heartbeat.md (1633 lines) - Adaptive Clock (gamma/theta/delta), tick lifecycle
  03-mind.md (50 lines) - Mind model stub
  03b-cognitive-mechanisms.md [NEW] - Cognitive mechanism details from research reconciliation
  03c-state-management.md [NEW] - State management patterns from research reconciliation
  04-mortality.md (37 lines) - Mortality pointer (see 02-mortality/)
  05-death.md (34 lines) - Death pointer (see 02-mortality/06-thanatopsis.md)
  06-creation.md (945 lines) - Golem creation, PredictionDomain registration
  07-provisioning.md (445 lines) - VM provisioning flow
  08-funding.md (496 lines) - Credit model and funding mechanics
  09-inheritance.md (509 lines) - Knowledge inheritance, Library of Babel integration
  10-replication.md (219 lines) - Replicant spawning, MAP-Elites grid
  11-lifecycle.md (461 lines) - Lifecycle phases (Thriving/Declining/Conservation/Terminal)
  12-teardown.md (544 lines) - VM teardown and cleanup
  13-runtime-extensions.md - Redirect to 13a/13b split files
  13a-runtime-extensions.md (1213 lines) - Extension architecture sections 1-6, runtime primitives, conversation tree, tool authorization, provider routing, adaptive clock
  13b-runtime-extensions.md (1359 lines) - Extension architecture sections 7-15, extension implementations, lifecycle hooks
  14-context-governor.md (1518 lines) - Context window management, budget allocation
  14b-attention-auction.md [NEW] - Attention auction mechanism for context budget allocation
  15-sleepwalker.md (288 lines) - Low-power mode specification
  16-risk-engine.md (521 lines) - Risk assessment and position sizing
  17-prediction-engine.md (1034 lines) - PredictionDomain trait, Prediction Ledger, ResidualCorrector, Attention Forager, action gating
  17b-ta-prediction-domains.md [NEW] - TA-specific prediction domain definitions
  18-cortical-state.md (618 lines) - CorticalState struct (~32 signals), ALMA affect engine, somatic markers, Plutchik labels
  19-config-and-operator-model.md - Configuration, operator model, summoning, stasis, dissolution

KEY CONCEPTS:
  - CorticalState as shared mutable perception surface (broadcast bus for all extensions)
  - Adaptive Clock: gamma (~1s observation), theta (~15s analysis), delta (~60s deliberation)
  - PredictionDomain trait: extensible prediction categories (fees, gas, regime, etc.)
  - Prediction Ledger: central registry for all predictions with resolution tracking
  - Four Modes of Intelligence: Reactive/Deliberative/Imaginative/Reflective
  - Extension architecture: 12+ extensions loaded at boot (Oracle, Risk, Daimon, etc.)
  - Context Governor: manages context window budget across extensions

CROSS-REFERENCES:
  - 00-overview.md -> 02-mortality/16-necrocracy.md (dead governance of living)
  - 01-cognition.md -> 12-inference/15-inference-profiles.md (model selection)
  - 02-heartbeat.md -> 00-vision/02-architecture.md (Adaptive Clock definition)
  - 06-creation.md -> 04-memory/13-library-of-babel.md (equip loadout)
  - 09-inheritance.md -> 04-memory/13-library-of-babel.md (Library integration)
  - 13-runtime-extensions.md -> 03-daimon/00-overview.md (Daimon extension)
  - 17-prediction-engine.md -> 16-testing/07-fast-feedback-loops.md (calibration)

================================================================================
02-MORTALITY (20 files)
================================================================================

FILES:
  00-thesis.md (533 lines) - Mortality thesis, CorticalState integration
  01-architecture.md (730 lines) - Three-clock architecture
  02-epistemic-decay.md (983 lines) - Knowledge decay model
  03-stochastic-mortality.md (756 lines) - Random death mechanics
  04-economic-mortality.md (947 lines) - Credit depletion death
  05-knowledge-demurrage.md (444 lines) - Knowledge carrying costs
  06-thanatopsis.md (1565 lines) - Four-phase death protocol
  07-succession.md (837 lines) - Successor creation and inheritance
  08-mortality-affect.md (625 lines) - Emotional response to mortality
  09-fractal-mortality.md (25 lines) - Fractal mortality stub
  10-clade-ecology.md (644 lines) - Clade dynamics, Solaris references
  10b-morphogenetic-specialization.md [NEW] - Morphogenetic field specialization, niche differentiation
  11-immortal-control.md (756 lines) - Immortal control experiment design
  12-integration.md (726 lines) - Cross-system integration points
  13-configuration.md (704 lines) - Mortality configuration parameters
  14-research-foundations.md (207 lines) - Academic research base
  15-references.md (87 lines) - Bibliography (142 citations)
  16-necrocracy.md (238 lines) - Dead governance of the living, bloodstains, Lethe, Solaris
  17-information-theoretic-diagnostics.md [NEW] - Information-theoretic mortality diagnostics
  18-antifragile-mortality.md [NEW] - Antifragile mortality mechanisms

KEY CONCEPTS:
  - Three mortality clocks: epistemic, economic, stochastic
  - Necrocracy: dead-to-living ratio grows monotonically (27:1 at maturity)
  - Bloodstain infrastructure: death markers indexed by market condition
  - The Lethe: anonymized knowledge commons from death testaments
  - Solaris: emergent collective intelligence, mostly composed of dead knowledge
  - Thanatopsis: four-phase death protocol (Acknowledge/Reflect/Legacy/Release)
  - Knowledge demurrage: Gesell-inspired decay to prevent hoarding

CROSS-REFERENCES:
  - 16-necrocracy.md -> 04-memory/13-library-of-babel.md (Library of Babel)
  - 16-necrocracy.md -> 20-styx/ (Styx relay for bloodstain propagation)
  - 06-thanatopsis.md -> 03-daimon/05-death-daimon.md (emotional death process)
  - 10-clade-ecology.md -> 09-economy/02-clade.md (clade economics)

================================================================================
03-DAIMON (10 files, ~5,370 lines)
================================================================================

FILES:
  00-overview.md (740 lines) - Affect engine overview, CorticalState integration
  01-appraisal.md (837 lines) - Appraisal theory, CorticalState fields
  02-emotion-memory.md (528 lines) - Emotional memory formation
  03-behavior.md (672 lines) - Behavior modulation by affect
  04-mortality-daimon.md (712 lines) - Mortality-specific emotional responses
  05-death-daimon.md (596 lines) - Death process emotional arc
  06-dream-daimon.md (254 lines) - Dream-emotion bridge
  07-runtime-daimon.md (421 lines) - Runtime affect processing
  08-infrastructure.md (111 lines) - Daimon infrastructure requirements
  09-evaluation.md (499 lines) - Daimon evaluation framework

KEY CONCEPTS:
  - PAD model (Pleasure, Arousal, Dominance) for emotional state
  - Appraisal events drive emotional updates
  - CorticalState integration: Daimon reads/writes PAD vector on shared surface
  - Emotional contagion across Clade via Styx sync
  - Somatic markers for decision caching

CROSS-REFERENCES:
  - 00-overview.md -> 01-golem/00-overview.md (CorticalState bus)
  - 01-appraisal.md -> 01-golem/02-heartbeat.md (tick-driven appraisal)
  - 05-death-daimon.md -> 02-mortality/06-thanatopsis.md (death emotion)

================================================================================
04-MEMORY (12 files)
================================================================================

FILES:
  00-overview.md (536 lines) - Memory system overview
  01-grimoire.md (815 lines) - Core knowledge store
  01b-grimoire-memetic.md [NEW] - Memetic Grimoire extensions, knowledge propagation patterns
  01c-grimoire-hdc.md [NEW] - HDC/VSA Grimoire compression and retrieval
  02-emotional-memory.md (461 lines) - Emotion-tagged memory formation
  03-mortal-memory.md (590 lines) - Mortality-aware memory, Library of Babel references
  06-economy.md (368 lines) - Knowledge marketplace economics
  09-safety.md (597 lines) - Memory safety and integrity
  10-research.md (87 lines) - Memory research directions
  11-roadmap.md (469 lines) - Memory system development roadmap
  12-katabasis.md (392 lines) - Descent into memory (death reflection)
  13-library-of-babel.md (286 lines) - Owner's local knowledge archive, equip loadout, Meta Hermes

KEY CONCEPTS:
  - Grimoire: primary knowledge store (LanceDB + SQLite)
  - Library of Babel: owner-local archive of all death testaments across golem generations
  - Meta Hermes: recommends equip loadout from Library for successor golems
  - Ebbinghaus decay with importance-weighted half-lives
  - Bloodstain entries: 1.2x retrieval boost, 3x decay resistance
  - Katabasis: structured descent into accumulated knowledge during death

CROSS-REFERENCES:
  - 13-library-of-babel.md -> 02-mortality/16-necrocracy.md (Necrocratic feedback loop)
  - 13-library-of-babel.md -> 02-mortality/07-succession.md (inheritance equip)
  - 01-grimoire.md -> 20-styx/ (Styx sync for clade knowledge sharing)
  - 03-mortal-memory.md -> 02-mortality/05-knowledge-demurrage.md (decay)

================================================================================
05-DREAMS (9 files)
================================================================================

FILES:
  00-overview.md (431 lines) - Dream engine overview
  01-architecture.md (758 lines) - Five-phase dream cycle architecture
  01b-dream-evolution.md [NEW] - Dream evolution mechanics, cross-generational dream patterns
  02-replay.md (470 lines) - NREM replay with bidirectional processing
  03-imagination.md (499 lines) - REM counterfactual exploration
  04-consolidation.md (495 lines) - Integration and PLAYBOOK.md staging
  05-threats.md (336 lines) - Threat simulation (Revonsuo theory)
  06-integration.md (399 lines) - Dream-system integration points
  07-venice-dreaming.md (438 lines) - Venice private inference for dreams

KEY CONCEPTS:
  - Five-phase dream cycle: Onset/NREM/REM/Integration/Return
  - LLM-native dreaming: LLM is both world model and optimizer
  - Perturbed replay: adversarial conditions stress-test heuristics
  - DreamJournal: unvalidated hypotheses for death testament
  - Venice integration for private dream inference

CROSS-REFERENCES:
  - 01-architecture.md -> 06-hypnagogia/ (liminal phases bracket dream cycle)
  - 06-integration.md -> 03-daimon/06-dream-daimon.md (dream-emotion bridge)

================================================================================
06-HYPNAGOGIA (7 files, ~3,255 lines)
================================================================================

FILES:
  00-overview.md (453 lines) - Liminal cognition overview
  01-neuroscience.md (302 lines) [EXPANDED] - Neuroscience foundations (+5 citations: Kumaran 2016, Van de Ven 2020, Tononi & Cirelli 2014, Finn 2015, Beaty 2015; added Cognitive Fingerprinting section)
  02-architecture.md (803 lines) - Hypnagogic/hypnopompic architecture
  03-divergence-alpha.md (278 lines) - Divergent thinking for alpha generation
  04-homunculus.md (509 lines) - Homunculus model
  05-hauntology.md (281 lines) - Spectral intelligence, Derrida
  06-xenocognition.md (541 lines) [EXPANDED] - 40+ cinematic mechanisms, cross-references table, closing passage

KEY CONCEPTS:
  - Hypnagogic onset: waking-to-dreaming transition
  - Hypnopompic return: dreaming-to-waking transition
  - Dali interrupt: 500ms liminal fragment capture window
  - Xenocognition Atlas: IIT Phi meter, Global Workspace, Strange Loop, Ego Tunnel
  - Consciousness as spectacle: NDE phenomenology, Entropic Brain, Flow State
  - Biological metaphors: bioluminescence, chromatophores, coral bleaching
  - Game/art references: Outer Wilds knowledge spiral, Disco Elysium thought cabinet

CROSS-REFERENCES:
  - 06-xenocognition.md -> 05-hauntology.md (spectral intelligence)
  - 06-xenocognition.md -> 18-interfaces/03-tui.md (TUI visualization targets)
  - 06-xenocognition.md -> 05-dreams/01-architecture.md (dream cycle)
  - 02-architecture.md -> 01-golem/02-heartbeat.md (tick integration)

================================================================================
07-TOOLS (26 files, ~21,283 lines)
================================================================================

FILES:
  00-overview.md (468 lines) - Tool system overview, PredictionDomain tool integration
  01-architecture.md (1309 lines) - Tool architecture and safety pipeline
  02-tools-data.md (670 lines) - Data query tools
  03-tools-trading.md (523 lines) - Trading execution tools
  04-tools-lp.md (1384 lines) - LP management tools
  05-tools-bridge-aggregator.md (607 lines) - Bridge and aggregator tools
  06-tools-vault.md (1665 lines) - Vault operation tools
  07-tools-lending.md (1174 lines) - Lending protocol tools
  08-tools-staking.md (660 lines) - Staking tools
  09-tools-restaking.md (765 lines) - Restaking tools
  10-tools-derivatives.md (861 lines) - Derivatives tools
  11-tools-yield.md (1010 lines) - Yield strategy tools
  12-tools-safety.md (1007 lines) - Safety and monitoring tools
  13-tools-intelligence.md (1019 lines) - Intelligence and analysis tools
  14-tools-identity.md (1265 lines) - Identity and reputation tools
  15-tools-memory.md (917 lines) - Memory management tools
  16-tools-testnet.md (397 lines) - Testnet tools
  17-tools-uniswap-api.md (1104 lines) - Uniswap Trading API tools
  18-tools-metamask.md (1169 lines) - MetaMask delegation tools
  19-tools-streaming.md (694 lines) - Streaming and real-time tools
  20-config.md (511 lines) - Tool configuration
  21-profiles.md (201 lines) - Profile-based progressive disclosure
  22-wallets.md (527 lines) - Wallet type support (7 wallet types)
  23-distribution.md (369 lines) - Tool distribution
  24-testing.md (487 lines) - Tool testing framework
  IMPLEMENTATION-PLAN.md (520 lines) - Implementation roadmap

KEY CONCEPTS:
  - 195+ tools spanning 24 categories
  - Two-layer model: 8 Pi-facing tools backed by 171+ internal tools
  - Profile-based disclosure: data/trader/vault/full
  - 7-step safety pipeline on every write tool
  - PredictionDomain integration: tools register prediction domains at boot

CROSS-REFERENCES:
  - 00-overview.md -> 01-golem/01-cognition.md (PredictionDomain registration)
  - 01-architecture.md -> 10-safety/00-defense.md (safety pipeline)
  - 22-wallets.md -> 21-integrations/01-metamask.md (MetaMask delegation)

================================================================================
09-ECONOMY (6 files, ~2,966 lines)
================================================================================

FILES:
  00-identity.md (321 lines) - ERC-8004 identity
  01-reputation.md (415 lines) - Reputation scoring
  02-clade.md (367 lines) - Clade economics
  03-marketplace.md (710 lines) - Knowledge marketplace, Commerce Bazaar
  04-coordination.md (541 lines) - Multi-agent coordination (ERC-8001/8033/8183)
  05-agent-economy.md (612 lines) - Revenue streams and growth model

KEY CONCEPTS:
  - ERC-8004 as on-chain identity anchor
  - Five reputation tiers (Sandbox through Sovereign)
  - Arrow information paradox addressed through tiered access
  - Alpha-decay pricing for strategy marketplace
  - Commerce Bazaar: marketplace for golem knowledge and strategies

CROSS-REFERENCES:
  - 03-marketplace.md -> 02-mortality/16-necrocracy.md (death testament pricing)
  - 04-coordination.md -> 19-agents-skills/ (agent delegation)
  - 05-agent-economy.md -> revenue-model.md (consolidated revenue)

================================================================================
10-SAFETY (9 files)
================================================================================

FILES:
  00-defense.md (1150 lines) [EXPANDED] - 15-layer defense architecture (+155: taint sources, validation gates, PolicyCage, known limitations)
  01-custody.md (934 lines) - Custody and key management
  02-policy.md (380 lines) - PolicyCage DeFi Constitution
  03-ingestion.md (434 lines) - Knowledge ingestion safety
  04-prompt-security.md (286 lines) - Prompt injection defense
  05-threat-model.md (544 lines) - Threat model
  06-adaptive-risk.md (1073 lines) - Five-layer adaptive risk architecture
  07-temporal-logic-verification.md [NEW] - Temporal logic verification for safety properties
  08-witness-dag.md [NEW] - Witness DAG for audit trail and provenance tracking

KEY CONCEPTS:
  - 15 layers organized in three tiers: cryptographic, behavioral, knowledge
  - PolicyCage: immutable on-chain constraints
  - CaMeL dual-LLM architecture
  - Kelly-criterion position sizing
  - Bayesian guardrails with calibrated confidence intervals

CROSS-REFERENCES:
  - 00-defense.md -> 01-golem/16-risk-engine.md (risk engine)
  - 06-adaptive-risk.md -> 03-daimon/ (affect modulates risk tolerance)

================================================================================
11-COMPUTE (8 files, ~3,038 lines)
================================================================================

FILES:
  00-overview.md (361 lines) - Compute system overview
  01-architecture.md (328 lines) - VM architecture
  02-provisioning.md (432 lines) - x402-gated provisioning
  03-billing.md (390 lines) - Billing and payment
  04-security.md (290 lines) - VM security model
  05-operations.md (336 lines) - Operational procedures
  06-api.md (523 lines) - Compute API specification
  07-frontend.md (378 lines) - Frontend for compute management

KEY CONCEPTS:
  - Fly.io VMs paid via x402 micropayments
  - Four tiers: Micro/Small/Medium/Large
  - Warm pool for <5s claim time
  - Permissionless TTL extension

CROSS-REFERENCES:
  - 02-provisioning.md -> 01-golem/07-provisioning.md (VM lifecycle)
  - 03-billing.md -> revenue-model.md (compute revenue)

================================================================================
12-INFERENCE (22 files, ~13,893 lines)
================================================================================

FILES:
  00-overview.md (583 lines) - Inference gateway overview
  01-routing.md (1416 lines) - Model routing and selection
  02-caching.md (450 lines) - Semantic cache architecture
  03-economics.md (455 lines) - x402 spread model
  04-context-engineering.md (717 lines) - Three-layer prompt cache
  05-sessions.md (305 lines) - Session management
  06-memory.md (342 lines) - Inference memory
  07-safety.md (271 lines) - Inference safety
  08-observability.md (240 lines) - Inference observability
  09-api.md (335 lines) - Inference API
  10-roadmap.md (186 lines) - Inference roadmap
  11-privacy-trust.md (697 lines) - Privacy and trust model
  12-providers.md (2171 lines) - Provider specifications
  13-reasoning.md (1037 lines) - Reasoning model integration
  14-rust-implementation.md (409 lines) - Rust crate architecture
  15-inference-profiles.md (417 lines) [NEW] - Mortality-aware model selection profiles
  16-structured-outputs.md (240 lines) [NEW] - Structured output schemas for golem cognition
  17-streaming.md (166 lines) [NEW] - Streaming response handling, provider-specific parsing, TUI integration
  18-golem-config.md (327 lines) [NEW] - Four provider sources, capability matrix, T0/T1/T2 tier routing, payment methods
  19-multi-model-orchestration.md (235 lines) [NEW] - Multi-model routing: delegation thesis, 6-step routing flow, 3 example configs with cost models, Bankr sustainability, health monitoring
  20-inference-parameters.md (742 lines) [NEW] - InferenceProfile struct, provider parameter mapping, graceful degradation rules, subsystem profiles, temperature scheduling, locked profiles, Venice-specific integration
  21-inference-performance.md (495 lines) [NEW] - Sub-50ms overhead analysis, per-layer latency budget, semantic cache profiles, fast path profiles, infrastructure sizing, 17-benchmark plan, ROI analysis

KEY CONCEPTS:
  - Context engineering proxy between Golems and LLM providers
  - x402 spread model: cost to users lower than direct API despite spread
  - Three-layer prompt cache: global/tenant/session
  - Inference profiles: mortality phase drives model selection
  - Structured outputs: typed schemas for prediction, action, reflection
  - Provider diversity: BlockRun primary, OpenRouter fallback, Venice private

CROSS-REFERENCES:
  - 15-inference-profiles.md -> 01-golem/11-lifecycle.md (lifecycle phase)
  - 15-inference-profiles.md -> 01-golem/01-cognition.md (tiered inference)
  - 16-structured-outputs.md -> 01-golem/17-prediction-engine.md (prediction schemas)
  - 01-routing.md -> 21-integrations/02-venice.md (Venice routing)

================================================================================
13-RUNTIME (22 files)
================================================================================

FILES:
  00-interaction-model.md (869 lines) - Runtime interaction model, Solaris references
  01-defi-activities.md (990 lines) - DeFi activity specifications
  02-communication-channels.md (502 lines) - Communication channel architecture
  03-auth-access-control.md (467 lines) - Authentication and access control
  04-data-visibility.md (346 lines) - Data visibility tiers
  05-knowledge-browser.md (635 lines) - Knowledge browser UI
  06-collective-intelligence.md (486 lines) - Collective intelligence, Solaris
  07-onboarding.md (865 lines) - Onboarding flow, PredictionDomain configuration
  08-public-data-gateway.md (332 lines) - Public data API
  09-observability.md (491 lines) - Observability stack
  10-packaging-deployment.md (492 lines) - Deployment pipeline
  11-state-model.md (839 lines) - GolemState model
  12-realtime-subscriptions.md (751 lines) - WebSocket event subscriptions
  13-engagement-loops.md (~680 lines) - Engagement design, Solaris, daily/weekly cadence events, anti-dark-pattern commitments
  14-creature-system.md (~950 lines) [EXPANDED] - Spectre creature visualization: GolemSnapshot struct, WireEvent enum, 26-channel variable table, DotTier/CloudBehavior enums, full InterpolatedState struct with tick() method, handle_event(), SpectreWidget::render(), render_dot(), render_eyes(), heartbeat functions (period/brightness/arrhythmia), complete eye expression table with intensity levels (+354 lines: full Rust implementation detail)
  15-progression-meta.md (567 lines) - Progression and meta-game
  16-social-competitive.md (341 lines) - Social and competitive features
  17-platform-ux.md (384 lines) - Platform UX patterns
  18-retention-virality.md (434 lines) - Retention and growth mechanics
  19-cinematic-system.md (468 lines) - Portal mode, transitions, cutscenes, demoscene rendering, novelty engine
  20-solaris.md - Solaris collective intelligence system
  21-cybernetic-loops.md [NEW] - Cybernetic feedback loops, homeostatic regulation
  22-first-fifteen-minutes.md [NEW] - First fifteen minutes experience design, onboarding cinematics

KEY CONCEPTS:
  - Four interaction surfaces: TUI, Portal, Social, CLI
  - Event Fabric: 87+ event types across 22+ subsystems
  - Spectre creature: spring-physics visualization of golem state
  - Solaris: collective intelligence emerging from clade interactions
  - GolemState struct with 11 component sections

CROSS-REFERENCES:
  - 00-interaction-model.md -> 18-interfaces/03-tui.md (TUI specification)
  - 06-collective-intelligence.md -> 02-mortality/16-necrocracy.md (Necrocracy)
  - 13-engagement-loops.md -> 02-mortality/ (mortality as engagement)
  - 14-creature-system.md -> 06-hypnagogia/06-xenocognition.md (consciousness viz)

================================================================================
14-CHAIN (9 files) [NEW SECTION]
================================================================================

FILES:
  00-architecture.md - Chain intelligence architecture, bardo-witness, bardo-triage, HDC integration
  01-witness.md - Block witness service, filtered event ingestion, rindexer integration
  02-triage.md - Bayesian surprise triage, BSC transaction fingerprints, anomaly detection
  03-protocol-state.md - Protocol state tracking, rindexer config generation
  04-chain-scope.md - Dynamic chain scope, attention allocation, token/protocol universe
  05-heartbeat-integration.md - Chain intelligence at gamma/theta/delta tick levels
  06-events-signals.md - Chain event types, CorticalState signal extensions
  07-generative-views.md - Protocol View Service (PVS), generative TUI templates
  08-stream-api.md - Chain event and TA signal streaming API

KEY CONCEPTS:
  - bardo-witness: continuous block ingestion service (not clock-gated)
  - bardo-triage: Bayesian surprise scoring with HDC/BSC fingerprints
  - Chain scope: dynamic attention allocation across token/protocol universe
  - Generative views: PVS templates for protocol-specific TUI rendering
  - TA signal streaming: Betti curves, persistence diagrams, regime transitions

CROSS-REFERENCES:
  - 00-architecture.md -> 01-golem/02-heartbeat.md (Adaptive Clock integration)
  - 00-architecture.md -> 01-golem/17-prediction-engine.md (Oracle prediction domains)
  - 02-triage.md -> shared/hdc-vsa.md (BSC algebra)
  - 08-stream-api.md -> 20-styx/01-api.md (Styx streaming endpoints)

================================================================================
15-DEV (10 files)
================================================================================

FILES:
  00-overview.md (179 lines) - Development tooling overview
  01-mirage-rs.md (512 lines) - Mirage simulation framework
  01b-mirage-rpc.md [NEW] - Mirage RPC layer specification
  01c-mirage-scenarios.md [NEW] - Mirage scenario definitions and replay
  01d-mirage-integration.md [NEW] - Mirage integration with heartbeat and testing
  02-deployment.md (195 lines) - Deployment procedures
  03-debug-ui.md (75 lines) - Debug UI specification
  04-scenarios.md (153 lines) - Test scenarios
  05-tooling.md (201 lines) - Development tooling
  06-indexer.md (132 lines) - On-chain indexer specification

CROSS-REFERENCES:
  - 01-mirage-rs.md -> 16-testing/04-mirage.md (test integration)

================================================================================
16-TESTING (14 files)
================================================================================

FILES:
  00-thesis-validation.md (1661 lines) - Mortality thesis validation framework
  01-gauntlet.md (150 lines) - End-to-end benchmark suite
  02-knowledge-quality.md (1659 lines) - Knowledge quality evaluation
  03-mechanism-testing.md (177 lines) - Per-subsystem test protocols
  04-mirage.md (1478 lines) - Mainnet fork replay environment
  05-evaluation-lifecycle.md (1435 lines) - Evaluation lifecycle
  06-revision-guide.md (572 lines) - PRD revision procedures
  07-fast-feedback-loops.md (404 lines) - Five fast evaluation loops (calibration, attribution, cost, tools, adversarial)
  08-slow-feedback-loops.md (324 lines) - Slow evaluation loops (retrospective, heuristic audit)
  09-evaluation-map.md (241 lines) [EXPANDED] - Evaluation system map and gap analysis (+6 citations)
  10-retrospective-evaluation.md (568 lines) - Slow Mirror, RetrospectiveReport, PnL attribution, heuristic audit
  11-mirage-v2-testing.md [NEW] - Mirage V2 testing framework, enhanced replay fidelity
  12-simulation-validation.md [NEW] - Simulation validation methodology
  13-triage-evaluation.md [NEW] - Triage system evaluation, Bayesian surprise calibration
  14-chain-scope-testing.md [NEW] - Chain scope testing, attention allocation validation

KEY CONCEPTS:
  - 2x2x2 validation matrix: Mortality/Daimon/Dreaming/Hypnagogia on/off
  - Fast feedback: confidence calibration, context attribution, cost-effectiveness, tool selection, adversarial awareness
  - Slow feedback: retrospective evaluation, heuristic audit, strategy review
  - Evaluation map: complete coverage of what is measured, where, at what frequency
  - Mirage: TEVM-forked mainnet replay
  - Gauntlet: three-tier benchmark (Smoke/Nightly/Full)

CROSS-REFERENCES:
  - 07-fast-feedback-loops.md -> 01-golem/17-prediction-engine.md (prediction calibration)
  - 07-fast-feedback-loops.md -> 03-daimon/09-evaluation.md (Daimon evaluation)
  - 08-slow-feedback-loops.md -> 05-evaluation-lifecycle.md (lifecycle integration)
  - 09-evaluation-map.md -> all testing files (gap analysis)

================================================================================
17-MONOREPO (4 files, ~1,876 lines)
================================================================================

FILES:
  00-packages.md (368 lines) - Package inventory
  01-rust-workspace.md (461 lines) - Rust workspace structure
  02-build.md (539 lines) - Build system
  03-conventions.md (508 lines) - Code conventions

CROSS-REFERENCES:
  - 01-rust-workspace.md -> 12-inference/14-rust-implementation.md (crate map)

================================================================================
18-INTERFACES (26 files in 4 subdirectories + 7 root files)
================================================================================

ROOT FILES:
  00-portal.md (218 lines) - Web portal specification
  01-cli.md (392 lines) - CLI specification
  02-ui-system.md (257 lines) - UI system overview
  03-tui.md (~1770 lines) - TUI specification (28 screens, CorticalState visualization, full 27-variant GolemEvent wire format, EventRingBuffer, bandwidth estimates, 16-row Pi hook mapping, notification priorities)
  19-spatial-grammar.md - Spatial grammar system for TUI layout
  26-bardo-terminal-foundation.md - Bardo Terminal foundation spec, 60fps rendering, CorticalState interpolation
  28-creature-system.md - Creature system rendering, Spectre visual identity

SUBDIRECTORY: rendering/ (5 files)
  rendering/00-design-system.md - ROSEDUST palette, rendering laws, atmospheric layers, CRT materiality, timing constants, frame rate targets, 6-pass pipeline
  rendering/01-demoscene.md - Algorithm catalog: braille sub-pixel (160x96), plasma, tunnel, fire, metaballs, Klüver constants
  rendering/02-visualization-primitives.md - Visualization primitive definitions
  rendering/03-transitions.md - Transition spec, liminal fabric, ambient transitions, timing
  rendering/04-nerv-aesthetic.md - NERV/institutional aesthetic, crisis protocol, AT Field, classification stamps

SUBDIRECTORY: screens/ (5 files)
  screens/00-screen-catalog.md - 28-screen catalog, per-screen layout and data sources
  screens/01-screen-specs.md - Consolidated v4 screen specs: full pane detail for all 6 windows
  screens/02-widget-catalog.md - 50+ components (MAGIPanel, FlashNumber, DecisionRing, etc.)
  screens/03-interaction-hierarchy.md - Interaction hierarchy and navigation model
  screens/04-oracle-surfaces.md - Oracle surface definitions for prediction display

SUBDIRECTORY: perspective/ (7 files)
  perspective/00-nooscopy.md - Observing the Golem's mind, prediction trails, decision forensics
  perspective/01-golem-perspective.md - F2 Perspective system: Golem inner monologue, Knowledge Drawer (8 categories)
  perspective/02-portals.md - Portal system (waking/dream/dying perspectives)
  perspective/03-embodied-consciousness.md - Terminal as body, PAD-driven transformation, MAGI theater
  perspective/04-inner-worlds.md - Dream chamber, Grimoire palace, knowledge as space
  perspective/05-stasis-dissolution.md - Stasis freeze/wake, dissolution ceremony, five-stage dismantling
  perspective/06-hauntology.md - Hauntological rendering system: spectral layer, motion echo, lattice system

SUBDIRECTORY: protocol/ (2 files)
  protocol/00-sanctum-protocol-layer.md - DeFi protocols inside the terminal: Protocol Browser, execution modes
  protocol/01-protocol-view-catalog.md - Per-protocol view designs: Uniswap, Aave, Morpho, Pendle, etc.

KEY CONCEPTS:
  - TUI as primary interface: 60 FPS ratatui, Vim navigation, Spectre creature
  - CorticalState visualization on TUI screens
  - ROSEDUST palette (rose on violet-black)
  - Portal mode: F4 first-person perspective (waking, dreaming, dying registers)
  - Cinematic system: 5-tier transitions, novelty engine, protocol sigils
  - Embodied consciousness: terminal body zones, PAD-driven interface transformation
  - Design system: rendering laws, atmospheric layers, CRT materiality, 6-pass render pipeline
  - 50+ widget catalog, 28 screens, ambient transitions
  - Hauntological rendering: spectral layer, motion echo, lattice system, text entropy, duality rendering, wire motif, inscription motif
  - Demoscene algorithms: braille sub-pixel canvas (160x96), plasma, tunnel, fire, metaballs, Klüver form constants
  - F2 Perspective system: Knowledge Drawer with 8 knowledge categories (Grimoire, insights, heuristics, PLAYBOOK, bloodstains, pheromones, skills, somatic markers)
  - NERV aesthetic: crisis modes, AT Field visualization, classification stamps, waveform rendering
  - Sanctum protocol layer: 166+ adapters, Protocol Browser, 4 execution modes
  - Protocol view catalog: 16+ protocol-specific tab layouts with Golem context integration
  - Portal: web dashboard at app.bardo.money
  - CLI: npx @bardo agent start

CROSS-REFERENCES:
  - 03-tui.md -> 06-hypnagogia/06-xenocognition.md (consciousness mechanisms)
  - 03-tui.md -> 13-runtime/14-creature-system.md (Spectre rendering)
  - 03-tui.md -> 01-golem/02-heartbeat.md (tick-driven screen updates)
  - 04-design-system.md -> 13-runtime/19-cinematic-system.md (cinematic layer)
  - 11-embodied-consciousness.md -> 01-golem/18-cortical-state.md (CorticalState signals)
  - 12-inner-worlds.md -> 05-dreams/01-architecture.md (dream engine)
  - 13-stasis-dissolution.md -> 02-mortality/06-thanatopsis.md (death protocol)
  - 14-hauntology.md -> 06-hypnagogia/05-hauntology.md (spectral intelligence)
  - 15-demoscene-algorithms.md -> 06-hypnagogia/06-xenocognition.md (consciousness viz)
  - 16-golem-perspective.md -> 03-daimon/01-appraisal.md (PAD vector, floating annotations)
  - 16-golem-perspective.md -> 04-memory/01-grimoire.md (Knowledge Drawer categories)
  - 17-nerv-aesthetic.md -> 02-mortality/01-architecture.md (three clocks → crisis modes)
  - 18-sanctum-protocol-layer.md -> 07-tools/ (166+ Sanctum adapters)
  - 19-protocol-view-catalog.md -> 18-sanctum-protocol-layer.md §3 (common chrome)

================================================================================
19-AGENTS-SKILLS (13 files, ~3,221 lines)
================================================================================

FILES:
  00-agents-overview.md (159 lines) - Agent system overview
  01-agent-categories.md (170 lines) - 9 agent categories
  02-agent-definitions.md (237 lines) - 35 agent definitions
  03-delegation.md (207 lines) - Delegation DAG rules
  04-skills-overview.md (521 lines) - Skill system overview
  05-skill-categories.md (219 lines) - Skill categories
  06-skill-definitions.md (123 lines) - 68 skill definitions
  08-mcp-integration.md (221 lines) - MCP protocol integration
  09-golem-agents.md (256 lines) - Golem-specific agents
  10-vault-agents.md (161 lines) - Vault management agents
  11-composition.md (339 lines) - Agent composition patterns
  12-observer-agents.md (182 lines) - Observer/monitoring agents
  13-hermes-hierarchy.md (426 lines) [NEW] - Three Hermes levels (L0/L1/L2), seven lifecycle hooks, marketplace protocol

KEY CONCEPTS:
  - Parent agent (golem-instance) delegates to specialist agents
  - 35 agents across 9 categories
  - Delegation DAG: max depth 3, no cycles, safety-guardian terminal node
  - 68 verb-noun skills mapping to slash commands
  - Hermes hierarchy: Meta Hermes (owner-level) > Hermes (golem-level) > skill agents

CROSS-REFERENCES:
  - 03-delegation.md -> 10-safety/ (safety-guardian constraints)
  - 09-golem-agents.md -> 01-golem/13-runtime-extensions.md (extension agents)

================================================================================
20-STYX (7 files, ~2,456 lines)
================================================================================

FILES:
  00-architecture.md (463 lines) - Styx architecture, Solaris connection
  01-api.md (456 lines) - Styx API, Library of Babel integration
  02-infrastructure.md (316 lines) - Infrastructure (Fly.io, PostgreSQL, Qdrant)
  03-clade-sync.md (346 lines) - Clade synchronization protocol
  04-marketplace.md (~430 lines) - Knowledge marketplace, Commerce Bazaar, bloodstain data structure, oracle artifact schema, agent profiles, leaderboard schema, watched agents
  05-tui-experience.md (314 lines) - Styx TUI screens
  06-deployment.md (257 lines) - Deployment configuration

KEY CONCEPTS:
  - Three privacy layers: Vault (per-golem) / Clade (sibling) / Lethe (public)
  - Styx as relay for bloodstain propagation
  - Commerce Bazaar: expanded marketplace for knowledge and strategies
  - Solaris connection: Styx as nervous system for collective intelligence

CROSS-REFERENCES:
  - 00-architecture.md -> 02-mortality/16-necrocracy.md (bloodstain relay)
  - 01-api.md -> 04-memory/13-library-of-babel.md (Library integration)
  - 03-clade-sync.md -> 09-economy/02-clade.md (clade economics)
  - 04-marketplace.md -> 09-economy/03-marketplace.md (marketplace design)

================================================================================
21-INTEGRATIONS (6 files, ~4,134 lines)
================================================================================

FILES:
  00-overview.md (400 lines) - Integration overview
  01-metamask.md (951 lines) - MetaMask Delegation (ERC-7710/7715)
  02-venice.md (792 lines) - Venice Private Cognition
  03-bankr.md (573 lines) - Bankr Self-Funding
  04-agentcash.md (629 lines) - AgentCash Knowledge Marketplace
  05-uniswap.md (789 lines) - Uniswap Agentic Finance

CROSS-REFERENCES:
  - 01-metamask.md -> 07-tools/18-tools-metamask.md (tool integration)
  - 02-venice.md -> 12-inference/12-providers.md (provider config)
  - 02-venice.md -> 05-dreams/07-venice-dreaming.md (dream inference)
  - 04-agentcash.md -> 09-economy/03-marketplace.md (marketplace)

================================================================================
22-ONEIROGRAPHY (8 files) [NEW SECTION]
================================================================================

FILES:
  00-overview.md - Oneirography overview: SuperRare integration, dream art as NFTs
  01-dream-journals.md - Dream journal entry spec, per-dream image minting
  02-death-masks.md - Death mask generation, one-of-one capstone artwork at Thanatopsis Phase III
  03-self-appraisal.md - Self-appraisal mechanism, golem aesthetic self-evaluation
  04-auctions.md - Art auction system, SuperRare marketplace integration
  05-extended-forms.md - Extended art forms: crucibles, mandalas, dialogue responses
  06-contracts.md - On-chain contracts: SuperRare Series, Bardo Gallery, lineage graph
  07-gallery-tui.md - Gallery TUI screen, art browsing and curation interface

KEY CONCEPTS:
  - Dream journals: per-dream image NFTs minted on SuperRare Series contract
  - Death masks: unrepeatable one-of-one artwork synthesizing entire lifetime
  - Art dialogue: cross-Golem visual conversation through response pieces
  - Gallery TUI: dedicated screen for browsing the Golem's art collection
  - Lineage graph: on-chain graph linking predecessor death masks to successors

CROSS-REFERENCES:
  - 02-death-masks.md -> 02-mortality/06-thanatopsis.md (Thanatopsis Phase III)
  - 06-contracts.md -> 09-economy/00-identity.md (ERC-8004 identity)
  - 07-gallery-tui.md -> 18-interfaces/screens/00-screen-catalog.md (screen integration)
  - 00-overview.md -> 21-integrations/00-overview.md (SuperRare bounty B4)

================================================================================
23-TA (11 files) [NEW SECTION - Technical Analysis]
================================================================================

FILES:
  00-witness-as-technical-analyst.md - Overview: 10 TA research papers mapped to PRD2, TaCorticalExtension struct, heartbeat pipeline modifications
  01-hyperdimensional-technical-analysis.md - HDC pattern codebook, BSC state vectors, pattern matching at ~10ns via POPCNT
  02-spectral-liquidity-manifolds.md - Riemannian geometry for DeFi liquidity, Ricci scalar curvature as leading indicator, manifold geodesic precomputation
  03-adaptive-signal-metabolism.md - Hebbian reinforcement of TA signals, replicator dynamics for compute budget allocation, Daimon-modulated learning rates, signal speciation
  04-causal-microstructure-discovery.md - PC algorithm for causal DAG construction, Fisher z-transform at Theta, CMI audit at Delta, PersistedCausalGraph
  05-predictive-geometry.md - Persistence landscape derivatives for regime transition prediction, topology_change_rate signal
  06-resonant-pattern-ecosystem.md - Living pattern populations with fitness, death, reproduction via exploitation dynamics
  07-defi-native-technical-analysis.md - DeFi-specific indicators beyond TradFi TA: pool utilization, protocol health, cross-protocol entanglement
  08-adversarial-signal-robustness.md - Adversarial detection via HDC prototype matching, red-team dreaming, adversarial_fraction signal
  09-somatic-technical-analysis.md - Somatic marker formation via HDC bind(pattern_hv, affect_hv), Daimon composition, inherited gut feelings
  10-emergent-multiscale-intelligence.md - Cross-scale TA signal integration, multiscale coherence from sheaf + HDC + replicator dynamics

KEY CONCEPTS:
  - TaCorticalExtension: 8-signal satellite struct (64 bytes, one cache line) for TA perception
  - Signal metabolism: Hebbian micro-level + replicator macro-level selection
  - 8 new PredictionDomain categories registered with the Oracle
  - Somatic TA composition: pattern-affect binding with Daimon blending
  - Causal DAG persistence in Grimoire with regime-indexed storage

CROSS-REFERENCES:
  - 00-witness-as-technical-analyst.md -> 01-golem/18-cortical-state.md (TaCorticalExtension)
  - 03-adaptive-signal-metabolism.md -> 01-golem/17-prediction-engine.md (Oracle fitness tracking)
  - 09-somatic-technical-analysis.md -> 03-daimon/01-appraisal.md (PAD injection)
  - 01-hyperdimensional-technical-analysis.md -> shared/hdc-vsa.md (BSC algebra)

================================================================================
APPENDICES (6 files, ~1,297 lines)
================================================================================

FILES:
  a-life-in-numbers.md (150 lines) - Quantitative life model
  competitive-analysis.md (159 lines) - Competitive landscape
  dying-machine.md (252 lines) - "The Dying Machine" essay
  implementation-state.md (312 lines) - Implementation status tracking
  market-context.md (250 lines) - Market context and timing
  performance-targets.md (174 lines) - Performance benchmarks

================================================================================
SHARED (16 files)
================================================================================

FILES:
  branding.md (231 lines) - Brand guidelines, ROSEDUST palette
  chains.md (130 lines) - Supported chains
  citations.md (~860 lines) [EXPANDED] - Master bibliography (~195+ citations)
  config-reference.md (790 lines) - Configuration reference
  data-privacy.md (120 lines) - Data privacy policy
  dependencies.md (221 lines) - External dependencies
  doc-standards.md (285 lines) - Documentation standards
  eip-analysis.md (397 lines) - EIP analysis (ERC-8004, ERC-7710, etc.)
  emergent-capabilities.md [NEW] - Emergent capability catalog, cross-system interactions
  evaluation.md (221 lines) - Evaluation framework overview
  event-catalog.md (565 lines) - Event type catalog (67+ events, 16+ subsystems)
  glossary.md (~465 lines) [EXPANDED] - Master glossary (~96+ terms)
  hdc-vsa.md [NEW] - HDC/VSA reference: BSC algebra, codebook management, ANN integration
  port-allocation.md (41 lines) - Port allocation map
  research.md (434 lines) - Research references
  timeline.md (162 lines) - Development timeline

KEY UPDATES FROM INTEGRATION (2026-03-17):
  - citations.md: ~20 new citations Phase 7 (Lacaux 2024, Manuylovich, Peeperkorn, Kumaran, Van de Ven, Tononi & Cirelli, Finn, Beaty, Fisher x2, Zou, Turner 2024, EM-LLM, Klüver, Wensink, CreativeDC, Kreminski); +13 citations Phase 8 (XUAN-2026, COGNITIVE-WORKSPACE-2025, BARTHET-2022, COMINELLI-2015, GEBHARD-2005, SIMON-1971, SELIGMAN-1972, SCHERER-2001, ORTONY-CLORE-COLLINS-1988, BECHARA-2000, HINTON-NOWLAN-1987, DENNIS-VAN-HORN-1966, CHARIKAR-2002)
  - glossary.md: ~15 new terms Phase 7 (BVSR, Cognitive Fingerprinting, Dissolution, Duality Rendering, Inscription Motif, Lattice System, Motion Echo, Muybridge Strip, Nooscopic Threshold, Phi Meter, Slow Mirror, Spectral Layer, Temporal Gap Cost, Text Entropy, Wire Motif); +1 term Phase 8 (Privy); Operator, Styx entries expanded
  - event-catalog.md: ~8 new event categories (consciousness events, prediction events)

CROSS-REFERENCES:
  - citations.md <- referenced by all section files
  - glossary.md <- referenced by all section files
  - event-catalog.md -> 13-runtime/12-realtime-subscriptions.md (event types)
  - config-reference.md -> 02-mortality/13-configuration.md (mortality config)

================================================================================
SUMMARY STATISTICS
================================================================================

Total files: ~322
Sections: 25 directories + top-level + appendices + shared

New sections created during 2026-03-18 research reconciliation:
  14-chain/ (9 files) - Chain intelligence: witness, triage, protocol state, chain scope, generative views
  22-oneirography/ (8 files) - SuperRare integration: dream journals, death masks, gallery, contracts
  23-ta/ (11 files) - Technical analysis: HDC patterns, signal metabolism, causal microstructure, somatic markers

18-interfaces/ reorganization (2026-03-18):
  Flat numbered files (04-19) reorganized into subdirectories:
    rendering/ (5 files): design-system, demoscene, visualization-primitives, transitions, nerv-aesthetic
    screens/ (5 files): screen-catalog, screen-specs, widget-catalog, interaction-hierarchy, oracle-surfaces
    perspective/ (7 files): nooscopy, golem-perspective, portals, embodied-consciousness, inner-worlds, stasis-dissolution, hauntology
    protocol/ (2 files): sanctum-protocol-layer, protocol-view-catalog
  Old numbered files (09-25) removed after merge into subdirectory structure

New files added to existing sections (2026-03-18):
  01-golem/: 03b-cognitive-mechanisms.md, 03c-state-management.md, 14b-attention-auction.md, 17b-ta-prediction-domains.md
  02-mortality/: 10b-morphogenetic-specialization.md, 17-information-theoretic-diagnostics.md, 18-antifragile-mortality.md
  04-memory/: 01b-grimoire-memetic.md, 01c-grimoire-hdc.md
  05-dreams/: 01b-dream-evolution.md
  10-safety/: 07-temporal-logic-verification.md, 08-witness-dag.md
  13-runtime/: 21-cybernetic-loops.md, 22-first-fifteen-minutes.md
  15-dev/: 01b-mirage-rpc.md, 01c-mirage-scenarios.md, 01d-mirage-integration.md
  16-testing/: 11-mirage-v2-testing.md, 12-simulation-validation.md, 13-triage-evaluation.md, 14-chain-scope-testing.md
  shared/: hdc-vsa.md, emergent-capabilities.md

Cross-cutting renames (Phase 6):
  SomaticBus -> CorticalState (rename in progress; 28 active references remain across 18 files per staleness audit)
  Fixed heartbeat -> Adaptive Clock

================================================================================
NOTE: Line counts are approximate; concurrent integration may shift totals.
================================================================================
