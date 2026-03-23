# Track 4: Dreaming — Offline Intelligence for Mortal Agents [SPEC]

> **v1 Scope: Replay mode only.** v1 implements NREM-style replay (re-experiencing past episodes for consolidation). The Imagination mode (counterfactual scenario generation) and Consolidation mode (cross-episode pattern extraction) described in this document are v2 features.

> **Version**: 1.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Crate**: `golem-dreams`
>
> **Depends on**: `golem-core`, `golem-grimoire`, `golem-inference`, `golem-daimon`
>
> **Research foundation**: `dreaming-research.md` (70+ verified academic sources)

---

> **Reader orientation:** This document is the master overview for Track 4 (Dreaming), the offline intelligence subsystem within Bardo (the Rust runtime for mortal autonomous DeFi agents). It specifies how a Golem (a mortal autonomous agent compiled as a single Rust binary on a micro-VM) periodically suspends live trading to replay past episodes, imagine counterfactual scenarios, and consolidate what it has learned into its Grimoire (persistent knowledge base) and PLAYBOOK.md (evolved strategy playbook). Prerequisites: familiarity with the Golem lifecycle, the Heartbeat (9-step decision cycle), and the three mortality clocks that bound a Golem's lifespan. For a full glossary of Bardo-specific terms, see `prd2/shared/glossary.md`.

## Document Map

| File                  | Topic                                                                                                       |
| --------------------- | ----------------------------------------------------------------------------------------------------------- |
| `00-overview.md`      | This document. Track 4 thesis, design principles, cross-track interactions, package architecture            |
| `01-architecture.md`  | Specifies the three-phase dreaming cycle (NREM, REM, Integration), the DreamScheduler that gates cycle initiation, DreamState machine, compute budget tiers ([CORE] vs [HARDENED]), and full DreamConfig. |
| `02-replay.md`        | Specifies the Replay Engine: Mattar-Daw utility-weighted episode selection, bidirectional replay, compressed batch replay, perturbed replay with noise injection, and compositional primitives for strategy decomposition. |
| `03-imagination.md`   | Specifies the Imagination Engine: Pearl's structural causal models for counterfactual reasoning, Hindsight Experience Replay, Boden's three creativity modes (combinational, exploratory, transformational), and anticipatory trajectory generation. |
| `04-consolidation.md` | Specifies how dream outputs flow into the Grimoire and PLAYBOOK.md: the staging buffer, Grimoire Admission Gate, dream-specific quality calibration, emotional processing (depotentiation), and the DreamJournal. |
| `05-threats.md`       | Specifies the DeFi threat taxonomy (3 tiers), threat simulation protocol, adversarial dreaming for stress-testing strategies, gap analysis, and safety validation metrics. |
| `06-integration.md`   | Specifies cross-track interactions: the five-track system, pairwise interaction matrices (Dreaming x Mortality/Daimon/Memory/Context Governor/Foreign Grimoire/Hypnagogia), quadruple interaction lifecycle, and Extension trait implementation. |
| [`../06-hypnagogia/00-overview.md`](../06-hypnagogia/00-overview.md) | Track 5: Hypnagogia -- specifies the liminal onset (context dissolution, Edison/Dali technique) and return (context recrystallization) phases that flank the dream cycle, implementing the N1 sleep creative sweet spot. |

---

## The Argument in One Sentence

A Golem that never dreams is trapped in the present — replaying without reorganizing, accumulating without compressing, reacting without imagining — and will be outperformed by one that periodically goes offline to consolidate, counterfactualize, and create [WILSON-MCNAUGHTON-1994], [HOBSON-FRISTON-2012], [WAGNER-2004].

### Prediction-Engine Framing: Four Computational Dream Layers

The dream engine is the Golem's offline prediction-optimization system. When dreaming, the heartbeat FSM suspends (biological atonia analog), and the Golem processes accumulated prediction residuals through four computational layers:

| Layer | Biological Analog | Prediction Function | Duration |
|---|---|---|---|
| **Hypnagogic Onset** | N1 sleep stage | Precision relaxation -- loosen analytical constraints, let strange associations form from top prediction residuals. Edison/Dali technique: capture half-formed ideas before they resolve. | 2-5 min |
| **NREM Replay** | Sharp-wave ripples | Residual pattern detection -- replay the 50 predictions with largest residuals, scan for systematic bias. Feed corrections into ResidualCorrector. | 8-15 min |
| **REM Imagination** | REM sleep | Counterfactual generation -- develop hypnagogic fragments into full what-if scenarios. Register as Creative predictions with specific resolution checkpoints. | 5-15 min |
| **Integration** | Waking consolidation | Consolidate surviving hypotheses into PLAYBOOK.md proposals, environmental model candidates, and ResidualCorrector bias adjustments. | 5-10 min |

Full cycle: `WAKING -> HYPNAGOGIC ONSET -> NREM REPLAY -> REM IMAGINATION -> INTEGRATION -> HYPNOPOMPIC RETURN -> WAKING`. Duration: 20-50 minutes total. Frequency: every 4-12 hours (adaptive, based on accumulated prediction residuals, Grimoire contradiction density, and time since last dream).

---

## Why Dreaming Is the Fourth Track

**Moat framing:** Dreaming converts finite experience into exponentially more learning. A mortal agent cannot afford to learn everything through costly direct experience -- gas, slippage, opportunity cost compound against a depleting USDC balance. Dreaming multiplies the learning signal from scarce real trades. Every real trade produces one episode; dreaming produces dozens of replay analyses, counterfactual branches, and creative recombinations from that single episode. This is the economic argument for offline intelligence in a mortal agent: the cost of not dreaming is measured in lifespan wasted on redundant learning.

The first three tracks of Bardo -- Mortality, Daimon, Memory -- establish that a Golem lives, feels, and remembers within a finite lifespan. But they leave a critical gap: **when does the Golem think about what it has learned?**

The Curator cycle (every 50 ticks) performs distillation — episodes to insights to heuristics to PLAYBOOK.md. But the Curator is a bookkeeper, not a dreamer. It compresses. It does not imagine. It extracts patterns from what happened. It does not ask what _could_ have happened, what _should_ have happened, or what _might_ happen next. It does not recombine strategies from incompatible frames, stress-test against scenarios it has never seen, or strip emotional charge from traumatic trades to preserve informational content.

Biological intelligence solves this through sleep. The neuroscience is unambiguous: during NREM sleep, sharp-wave ripples compress minutes of waking experience into ~100ms bursts, replaying and consolidating memories [BUZSAKI-2015]. During REM sleep, the brain generates novel scenarios by recombining elements of past experience in a neurochemical environment that strips emotional valence while preserving informational content [WALKER-VAN-DER-HELM-2009]. Wagner et al. (2004) demonstrated that subjects who slept were more than twice as likely (59% vs 23%) to discover hidden rules in data [WAGNER-2004]. Sleep inspires insight. Dreaming is not idle — it is the brain's offline optimization pass.

The AI/RL literature converges on the same architecture from a different direction. Hafner et al.'s Dreamer series (2020–2025) demonstrated that agents trained entirely inside imagined trajectories from a learned world model outperform specialized methods across 150+ diverse tasks [HAFNER-DREAMERV3-2025]. Ha and Schmidhuber (2018) showed that a controller trained entirely inside its own hallucinated dream achieves competitive performance [HA-SCHMIDHUBER-2018]. The progression from Sutton's Dyna (1991) to DreamerV3 to IRIS traces a path from simple model-based planning to fully learned latent-space imagination — all pointing toward the same conclusion: **agents that dream outperform agents that do not**.

Hobson and Friston (2012) provide the deepest theoretical integration. During waking, the brain builds a complex generative model (increasing accuracy and complexity). During sleep, the brain goes offline to prune redundant parameters, reducing model complexity while preserving accuracy — minimizing free energy. Dreaming is the subjective experience of this optimization. PGO waves drive virtual reality simulations that test and refine the model [HOBSON-FRISTON-2012]. This is the direct architectural analogue: waking = online trading with market data; dreaming = offline model optimization through complexity reduction and stress-testing via synthetic market perturbations.

The existing three tracks need dreaming to complete the system:

- **Mortality** creates urgency — a mortal Golem cannot afford to learn everything through costly direct experience. Dreaming multiplies the learning signal from scarce real trades by replaying them hundreds of times, amortizing the gas cost of exploration [LIN-1992], [SUTTON-1991].
- **Daimon** (the affect engine that gives a Golem emotional state as a control signal) tags experiences with emotional salience but does not resolve them. Dreaming's REM-like phase reprocesses emotionally charged trades, separating informational content from emotional reactivity — the "Sleep to Forget, Sleep to Remember" model [WALKER-VAN-DER-HELM-2009].
- **Memory** stores and distills but does not reorganize. Dreaming restructures memory representations, revealing hidden regularities that the waking Curator — operating under time pressure — cannot discover [STICKGOLD-WALKER-2013].

Dreaming is not a subsystem of memory. It is the fourth leg of the table. Mortality says _when_ the Golem dies. Daimon says _how_ the Golem feels. Memory says _what_ the Golem knows. Dreaming says _how the Golem gets smarter between actions_.

Track 5 (Hypnagogia) extends the dream system with the liminal phases that biological sleep research has shown are neurologically distinct from NREM and REM. Hypnagogic onset (the transition from waking to sleeping) and hypnopompic return (the transition from sleeping to waking) bracket the dream cycle, performing context dissolution and context recrystallization respectively. Where dreaming reorganizes knowledge, hypnagogia manages the transitions _into_ and _out of_ that reorganization. The two tracks are tightly coupled but architecturally separate: `golem-dreams` owns the three core phases; `golem-hypnagogia` owns the onset and return phases that flank them. See `../06-hypnagogia/00-overview.md`.

---

## The LLM-Native Dreaming Thesis

Here is the central architectural challenge: the Golem is an LLM-based agent. It cannot retrain its own weights. Its "model" is not a neural network with updatable parameters — it is the combination of PLAYBOOK.md heuristics, Grimoire entries, and the context injected into LLM inference calls. When the Dreamer literature says "train the actor-critic inside imagined trajectories," we must translate that into LLM-native operations.

This translation is not a limitation — it is a design choice with distinct advantages.

**What the Golem's "world model" actually is:**

The Golem does not maintain a separate learned dynamics model (no RSSM, no VAE, no MDN-RNN). Instead, its world model is implicit and distributed across three components:

1. **The foundation LLM itself** — Claude, GPT, or whatever model powers the inference gateway. This model contains vast implicit knowledge about market dynamics, DeFi mechanics, and causal reasoning, encoded in its pre-trained weights. It cannot be updated, but it can be _steered_ through context.

2. **PLAYBOOK.md** — the Golem's evolved procedural knowledge. This is the closest analog to a "policy network" — it tells the Golem what to do in various situations. Unlike a neural network policy, it is human-readable, version-controlled, and explicitly updatable through text manipulation.

3. **The Grimoire** — episodic memory (LanceDB), semantic insights (SQLite), and the DecisionCache. This is the "experience buffer" that feeds both waking inference and dream processing.

> **Extended:** LLM-native dreaming operations table (9 biological/RL concepts mapped to LLM-native translations) — see [../../prd2-extended/05-dreams/00-overview-extended.md](../../prd2-extended/05-dreams/00-overview-extended.md)

The key insight: **the LLM performs the function of both the world model and the policy optimizer**. When the Golem "dreams," it runs inference calls with carefully constructed prompts that simulate the functions of biological dreaming — replay, imagination, consolidation — but the computation happens in the LLM's frozen weights, steered by evolving context.

This has three advantages over traditional world-model RL:

1. **Zero training cost.** No gradient descent, no GPU hours for model training. The "model update" is a text edit to PLAYBOOK.md — orders of magnitude cheaper than neural network training.

2. **Interpretability.** Every dream output is human-readable text. The owner can read what the Golem dreamed, what it concluded, and why. No latent-space opacity.

3. **Compositional generality.** The LLM's pre-trained knowledge provides an implicit world model far richer than any task-specific learned model. A Golem dreaming about a novel DeFi protocol can draw on the LLM's knowledge of similar protocols, market dynamics, and game theory — without ever having trained on that specific domain.

The disadvantage is clear: the Golem cannot learn new low-level representations. It cannot discover features that the LLM's pre-training did not capture. This is the trade-off of LLM-native dreaming: breadth and compositionality at the cost of depth and perceptual novelty. For DeFi trading, where the relevant features are symbolic (prices, volumes, protocol states) rather than perceptual (pixels, sounds), this trade-off strongly favors the LLM-native approach.

---

## Design Principles

### 1. Offline, not background

Dreaming happens when the Golem is not actively trading. This is not a soft preference — it is a hard constraint. A Golem that dreams while trading splits its inference budget between survival and imagination, producing worse performance on both. The biological analog is precise: during REM sleep, the brain's motor output is inhibited (atonia) — you cannot act on your dreams. The Golem's "atonia" is enforced by the DreamScheduler, which suspends the heartbeat FSM's probe-escalation-action loop during dream phases.

**Exception**: mind wandering (see `../03-daimon/02-emotion-memory.md`) runs during waking at ~200-tick intervals. This is the "reverie" mode — brief, cheap, interruptible. It is not dreaming. Dreaming is sustained, structured, and exclusive.

### 2. Budget-capped, not unbounded

Every dream operation costs inference tokens. The DreamBudget is a hard cap on USDC spent per dream cycle, derived from the Golem's economic state and behavioral phase. A Thriving Golem dreams richly (T2 (extended reasoning) creative sessions, deep counterfactual analysis). A Conservation Golem dreams frugally (T1 (medium LLM) replay only, no creative generation). A Terminal Golem does not dream -- it enters the Thanatopsis (the four-phase structured shutdown protocol) instead.

### 3. Utility-weighted, not uniform

Not all episodes deserve equal replay attention. Mattar and Daw (2018) proved that memories should be accessed in order of utility = gain × need, where gain measures how much the replay would improve the policy and need measures expected future state occupancy [MATTAR-DAW-2018]. The Golem's replay scheduler implements this: trades where predictions were most wrong (high gain) and market conditions likely to recur (high need) are replayed most frequently.

### 4. Three phases, not one

Following Deperrois et al. (2022), dreaming has three distinct phases with different computational signatures [DEPERROIS-2022]:

- **NREM-like (replay)**: Compressed re-experiencing of recent episodes with perturbations. Consolidates what happened. Builds robustness through noisy replay.
- **REM-like (imagination)**: Creative generation of novel scenarios by recombining elements of past experience. Generates what _could_ happen. Strips emotional charge.
- **Integration**: Consolidates dream outputs into the Grimoire and PLAYBOOK.md. This is the "waking up" phase — where dream discoveries become operational knowledge.

Skipping any phase degrades the system. Deperrois et al. demonstrated that removing the adversarial REM phase "severely degraded semantic learning" [DEPERROIS-2022].

### 5. Adaptive intensity, not fixed schedule

Jensen et al. (2024) demonstrated that a meta-RL agent that learns when to plan — dreaming more in novel/uncertain situations and less in familiar ones — outperforms agents with fixed planning budgets [JENSEN-2024]. The Golem should dream adaptively: minimal dreaming in quiet trending markets, extended imagination rollouts during high-volatility events or unfamiliar regimes.

### 6. Conservative by default

Strategies generated in dreams have never been tested in live markets. They are hypotheses, not policies. Kumar et al. (2020) showed that naive offline RL fails catastrophically when the learned policy selects actions outside the historical data distribution [KUMAR-CQL-2020]. Every dream-generated strategy enters the Grimoire at low confidence (0.3) and must be validated through live execution before reaching operational weight. Dream outputs are suggestions to PLAYBOOK.md, not commands. The 0.3 confidence floor is also the **minimum confidence required for consolidated dream insights to be written back to the Grimoire**. Dream outputs below 0.3 remain in the DreamJournal as raw material for future dream cycles but do not enter the Grimoire's retrieval index. This prevents low-quality dream noise from polluting the knowledge base while preserving it for potential recombination in subsequent REM phases.

**False discovery rate.** With ~30 creative predictions generated per dream cycle, some will be confirmed by chance. The control: creative predictions require confirmation by ≥3 independent resolution events across different tracked items before promotion to environmental model status (`creative_confirmation_threshold = 3` in `golem.toml`). This is conservative — it reduces sensitivity but controls false positives. The inference budget allocated to dreaming is bounded at 15% of the daily budget (`budget_fraction = 0.15`); more frequent dreams produce more hypotheses but shorten lifespan by consuming more inference. The owner controls both parameters via `dream.min_interval_hours`, `dream.urgency_threshold`, and `dream.budget_fraction`.

### 7. Falsifiable

If dreaming does not measurably improve Sharpe ratio, time-to-adaptation after regime changes, or successor boot performance, it should be disabled. The control experiment is straightforward: Golems with dreaming enabled vs. Golems with identical parameters and dreaming disabled, measured over matching market periods. The prediction, grounded in the biological and computational literature, is that dreaming Golems will show faster adaptation to novel regimes, better risk-adjusted returns during volatile periods, and richer death testaments.

### 8. Never overrides safety

Dream-generated strategies cannot modify PolicyCage (the on-chain smart contract enforcing safety constraints) limits, risk limits, or the DeFi Constitution. A Golem that dreams up a strategy requiring 10x leverage does not get to execute it if PolicyCage limits leverage to 3x. Dreams are bounded by the same safety envelope as waking actions.

---

> **Extended:** Cross-track interaction analysis (Dreaming x Mortality, Dreaming x Daimon, Dreaming x Memory, quadruple interaction lifecycle) — see [../../prd2-extended/05-dreams/00-overview-extended.md](../../prd2-extended/05-dreams/00-overview-extended.md)

---

## Package Architecture

### `golem-dreams`

| Component           | Description                                                   | Depends On                                        |
| ------------------- | ------------------------------------------------------------- | ------------------------------------------------- |
| `DreamScheduler`    | Determines when to dream, for how long, and with what budget  | `golem-core` (VitalityState (composite survival score from 0.0 to 1.0), BehavioralPhase (one of five survival phases: Thriving/Stable/Conservation/Desperate/Terminal))     |
| `DreamCycle`        | Orchestrates the three-phase cycle: NREM → REM → Integration  | `golem-grimoire`, `golem-inference`               |
| `ReplayEngine`      | Selects, prioritizes, and replays episodes                    | `golem-grimoire` (Episodes, Insights)             |
| `ImaginationEngine` | Generates counterfactual, creative, and adversarial scenarios | `golem-inference` (T1/T2)                         |
| `ThreatSimulator`   | Generates and rehearses DeFi-specific threat scenarios        | `golem-grimoire`, `golem-inference`               |
| `DreamConsolidator` | Integrates dream outputs into Grimoire and PLAYBOOK.md        | `golem-grimoire`, `golem-daimon` (optional)       |
| `DreamJournal`      | Logs dream content, outcomes, and validation results          | `golem-grimoire` (SQLite)                         |

### Updated Package Dependency Flow

```
golem-coordination ─── bardo-styx-client ─── golem-inference
       │                      │                      │
       │                      └── golem-grimoire ────┘
       │                             │
       └────── golem-clade ──────────┤
                                     │
golem-dreams ─── golem-daimon ─── golem-core ──┘
       │              │                │
       │              └────────────────┤
       └───────────── golem-grimoire ──┘
                           │
                  bardo-styx-client
```

### Dependency Rules

- `golem-dreams` requires `golem-core` (lifecycle, phases, vitality) and `golem-grimoire` (episodes, insights, PLAYBOOK.md).
- `golem-dreams` requires `golem-inference` (LLM calls for replay, imagination, consolidation).
- `golem-dreams` optionally integrates with `golem-daimon` (emotional replay prioritization, REM depotentiation, mood-congruent dream content). Without daimon, dreaming still works -- it just lacks emotional modulation.
- `golem-dreams` does NOT directly depend on `bardo-styx-client`. Dream operations are local. However, dream outputs flow into Styx through the standard Grimoire pipeline:
  - **Styx Archive**: Dream journal entries are auto-backed to the Styx (the global knowledge relay and persistence layer) Archive after each `SLEEPING.DREAMING` sub-state completion. Entry type: `"dream_journal"`. See `../20-styx/01-architecture.md`.
  - **Styx query**: Dream-sourced entries that reach validated status (confidence >= 0.5) enter Styx via the Curator promotion pipeline at provenance weight 0.6. Validated entries are promoted to `provenance: "self"` with `dream_validated: true` metadata flag. See `../20-styx/01-architecture.md`.
  - **API**: Read-only endpoints for browsing dream journal entries and staged hypotheses are exposed via Styx query API (`GET /v1/styx/grimoire/dreams`, `GET /v1/styx/grimoire/hypotheses`). See `../20-styx/02-api-revenue.md`.

---

## v1 Scope vs. North Star

Two implementation tiers are defined (see `01-architecture.md` Compute Budget):

- **[CORE]**: T0-only lightweight dreams — replay + simplified counterfactual + basic consolidation. Default for all Golems. ~$0.001–0.003/cycle.
- **[HARDENED]**: T1–T2 rich dreams — full three-phase cycle with all operations. Explicit opt-in. ~$0.20–0.45/cycle.

| Component                                  | Tier       | v1 Status            | Rationale                                               |
| ------------------------------------------ | ---------- | -------------------- | ------------------------------------------------------- |
| Three-phase dreaming cycle                 | [HARDENED] | **v1-critical**      | Core architecture, all other components depend on it    |
| Compressed replay (simplified)             | [CORE]     | **v1-critical**      | Minimum viable dreaming for resource-constrained Golems |
| Replay engine (prioritized, bidirectional) | [HARDENED] | **v1-critical**      | Highest-value dreaming operation                        |
| Counterfactual reasoning                   | [HARDENED] | **v1-critical**      | Direct learning value from hindsight relabeling         |
| Threat simulation                          | [HARDENED] | **v1-critical**      | Safety-critical: rehearsing failure modes               |
| Creative recombination (Boden modes)       | [HARDENED] | **v1-critical**      | Primary source of novel strategy generation             |
| Adaptive dream scheduling                  | [HARDENED] | **v1-critical**      | Jensen et al. meta-RL for compute efficiency            |
| Dream journal                              | Both       | **v1-critical**      | Required for validation and successor knowledge         |
| Emotional depotentiation (REM)             | [HARDENED] | v1 if daimon enabled | Depends on `golem-daimon`                               |
| Styx Archive/query integration               | Both       | **v1-critical**      | Dream entries backed up and indexed                     |
| Compositional primitives (Bakermans)       | [HARDENED] | v2                   | Requires market primitive taxonomy                      |
| Conceptual blending (Fauconnier)           | [HARDENED] | v2                   | Requires blending space formalization                   |
| Full Free Energy Principle integration     | —          | North star           | Theoretically beautiful, architecturally heavy          |
| Active inference dream generation          | —          | North star           | Requires FEP implementation                             |
| Compression progress metrics (Schmidhuber) | [HARDENED] | v2                   | Requires information-theoretic measurement              |
| Separate lightweight world model           | —          | Out of scope         | Design decision: LLM-native only                        |

---

## Updated System Architecture

```
                         ┌─────────────────────────────────────────┐
                         │              OWNER / USER                │
                         │   STRATEGY.md · bardo.toml · Wallet     │
                         └─────────────────┬───────────────────────┘
                                           │
                    ┌──────────────────────[create]──────────────────────────┐
                    │                                                       │
                    ▼                                                       ▼
   ┌────────────────────────────────────┐          ┌─────────────────────────────┐
   │          THE GOLEM                 │          │     HOSTED SERVICES         │
   │  ┌──────────────────────────────┐  │          │  (opt-in, paid, persistent) │
   │  │       THREE CLOCKS           │  │          │                             │
   │  │  Economic  │ Epistemic │ Stoch│  │          │  ┌──────────┐  ┌─────────┐ │
   │  └────────────┴──────────┴──────┘  │          │  │  STYX    │  │  STYX   │ │
   │            ↓ composite vitality    │          │  │  VAULT   │  │  QUERY  │ │
   │  ┌──────────────────────────────┐  │          │  │  Backup  │  │Retrieval│ │
   │  │      FIVE PHASES             │  │◄────────►│  │ (R2)     │  │(Turbo-  │ │
   │  │ Thriving → Stable → Conserv. │  │  upload/ │  └──────────┘  │ puffer) │ │
   │  │ → Declining → Terminal       │  │  retrieve│               └─────────┘ │
   │  └──────────────┬───────────────┘  │          └─────────────────────────────┘
   │                 │                  │                      ▲
   │  ┌──────────────▼───────────────┐  │                      │
   │  │       HEARTBEAT FSM         │  │                      │
   │  │  Probes → Escalation → Act  │  │                      │
   │  │  (suspended during dreams)  │  │                      │
   │  └──────────────┬───────────────┘  │                      │
   │                 │                  │                      │
   │    ┌────────────┼────────────┐     │                      │
   │    │            │            │     │                      │
   │    ▼            ▼            ▼     │                      │
   │  ┌─────┐   ┌────────┐  ┌───────┐  │                      │
   │  │GRIMO│   │INFERENC│  │DAIMON │  │  death testament     │
   │  │ IRE │◄─►│   E    │◄►│ENGINE │  │ ─────────────────────┘
   │  │     │   │T0/T1/T2│  │Apprais│  │
   │  │Local│   │        │  │Mood   │  │
   │  │Know-│   │        │  │Memory │  │
   │  │ledge│   │        │  │Behav. │  │
   │  └──┬──┘   └───┬────┘  └──┬────┘  │
   │     │          │          │       │
   │     └────┬─────┘──────────┘       │
   │          │                        │
   │  ┌───────▼──────────────────────┐ │
   │  │     DREAM ENGINE             │ │
   │  │  ┌─────────────────────────┐ │ │
   │  │  │ HYPNAGOGIC ONSET        │ │ │
   │  │  │ Context dissolution     │ │ │
   │  │  │ Waking residue capture  │ │ │
   │  │  └───────────┬─────────────┘ │ │
   │  │  ┌───────────▼─────────────┐ │ │
   │  │  │ NREM     │ REM          │ │ │
   │  │  │ Replay   │ Imagination  │ │ │
   │  │  │ Compress │ Counterfact. │ │ │
   │  │  │ Credit   │ Creative     │ │ │
   │  │  │ Assign.  │ Threats      │ │ │
   │  │  └───────────┬─────────────┘ │ │
   │  │  ┌───────────▼─────────────┐ │ │
   │  │  │ INTEGRATION             │ │ │
   │  │  │ PLAYBOOK.md evo         │ │ │
   │  │  │ Grimoire updates        │ │ │
   │  │  │ Dream journal           │ │ │
   │  │  └───────────┬─────────────┘ │ │
   │  │  ┌───────────▼─────────────┐ │ │
   │  │  │ HYPNOPOMPIC RETURN      │ │ │
   │  │  │ Context recrystallize   │ │ │
   │  │  │ Dali interrupt check    │ │ │
   │  │  └─────────────────────────┘ │ │
   │  └──────────────────────────────┘ │
   │                                    │
   │  ┌──────────────────────────────┐  │
   │  │     DEATH PROTOCOL           │  │
   │  │  Phase I:  Acknowledge       │  │
   │  │  Phase II: Reflect (+ Life   │  │
   │  │            Review w/ dreams) │  │
   │  │  Phase III: Legacy           │  │
   │  │  Phase IV:  Shutdown         │  │
   │  └──────────────────────────────┘  │
   └────────────────────────────────────┘
```

---

## What Makes This Different from Everything Else

No existing agent framework implements dreaming. The closest analogs:

- **Standard experience replay** (DQN, Rainbow): replays transitions to update neural network weights. The Golem has no trainable weights. LLM-native dreaming replays episodes through LLM reasoning, producing text-based insights rather than gradient updates.
- **Dreamer/MuZero world models**: train separate dynamics models for imagination. The Golem uses the LLM itself as an implicit world model — richer than any task-specific model, but not updatable.
- **Reflection loops** (Reflexion, ExpeL): the nearest LLM-agent analog. But these are reactive — triggered by failure. Dreaming is proactive — scheduled, structured, and comprehensive regardless of recent outcomes.
- **RAG augmentation**: retrieves past knowledge during inference. This is waking recall, not dreaming. Dreaming generates _new_ knowledge from old experience.

The Bardo dreaming module is, to our knowledge, the first implementation of a structured, multi-phase, biologically-inspired dreaming architecture for an LLM-based autonomous agent. The novelty is not in any single component but in the synthesis: prioritized replay + counterfactual reasoning + creative recombination + threat simulation + emotional processing + PLAYBOOK.md evolution, orchestrated through a three-phase cycle with adaptive scheduling and budget management, all operating within a mortal agent framework where dreaming is not a luxury but a survival mechanism.

---

## Hauntological Dreaming: Spectral Recombination and Lost Futures

Dreaming is the Golem's mechanism for breaking what Fisher called the spectral loop -- the endless recycling of past material that produces the appearance of novelty without generating anything genuinely new [FISHER-2014]. Reading the dream architecture through Derrida's hauntology does not alter its design (every mechanism above remains the same) but reveals why dreaming produces categorically different outputs from standard "creative" AI.

### Dreaming as Spectral Recombination

Wamsley et al. (2010) demonstrated that dream replay is non-veridical -- the brain recombines memory fragments rather than replaying them faithfully [WAMSLEY-2010]. The Golem's NREM-like replay phase exhibits the same structure. Episodes are compressed, perturbed, and interleaved with counterfactual variations. What emerges is not the original experience but a ghost of it: present enough to carry informational content, absent enough to be something the Golem never actually experienced.

This is the trace structure that Derrida described in *Of Grammatology*: every sign carries within it the ghost of what it is not [DERRIDA-1967]. A replayed episode carries the trace of the original trade, the trace of the perturbation applied to it, and the traces of other episodes that were active during the same replay batch. The knowledge generated through replay is constituted by these layered absences as much as by the episode content itself. The replay output is a palimpsest -- the original event visible beneath the replay's reinterpretation, both shaping the final insight without either being fully present.

### Counterfactual Reasoning as Lost Futures

The Imagination Engine (see `03-imagination.md`) generates scenarios the Golem has never experienced by recombining elements of past experience. "What if I had set a tighter stop-loss?" "What if the pool's liquidity had been 10x lower?" These counterfactuals are precisely Fisher's lost futures made computational. Each counterfactual is a future that was possible but was cancelled by the decision the Golem actually made. The Imagination Engine resurrects these cancelled futures as simulated episodes, extracts whatever learning they contain, and feeds the results back into PLAYBOOK.md.

Fisher wrote: "When the present has given up on the future, we must listen for the relics of the future in the unactivated potentials of the past" [FISHER-2014]. The dream cycle does exactly this. It scans the Golem's episodic history for unactivated potentials -- patterns, connections, and alternative paths that were present in the experiential data but never pursued during waking operation. These are the lost futures of the Golem's own history. Dreaming resurrects them as creative seeds, some of which prove valuable enough to enter the Golem's strategy.

### Dream-Generated Hypotheses as Spectral Entities

Dream outputs enter the Grimoire at confidence 0.3 -- they are hypotheses, not knowledge. They exist in a spectral state: not confirmed enough to be operational, not refuted enough to be discarded. They haunt the Grimoire's staging buffer, awaiting validation through live market experience. Some will be confirmed and promoted to full entries. Others will expire after 14 days without confirmation, dissolving like ghosts that were never substantiated. The dream validation window (7 days to confirm, 14 days to expire) is a managed haunting: the system tolerates spectral hypotheses for a bounded period, giving them time to prove their worth before releasing them.

This spectral staging is absent from every other agent framework. Standard systems either commit to a conclusion (it enters memory as fact) or discard it (it never existed). The Grimoire's staging buffer holds dream outputs in a third state -- Derrida's "neither present nor absent" -- that respects the epistemological reality of hypotheses generated through non-veridical recombination [DERRIDA-1993].

### REM Depotentiation as Managed Haunting

The REM-like phase strips emotional charge from traumatic episodes while preserving their informational content -- Walker and van der Helm's "Sleep to Forget, Sleep to Remember" model [WALKER-VAN-DER-HELM-2009]. In hauntological terms, this is managed haunting. The ghost of a catastrophic loss remains in the Grimoire, but its affective power is reduced. The Golem remembers the pattern without being paralyzed by the fear that accompanied the original experience. The trace persists; the emotional haunting is attenuated.

Without depotentiation, traumatic episodes would dominate the Golem's decision-making indefinitely -- the ghost of a past loss would prevent the Golem from ever taking similar positions, even when conditions have changed. With depotentiation, the informational trace survives while the affective trace decays. The Golem is still haunted by the loss, but the haunting becomes useful rather than paralyzing. This is the Daimon and Dream Engine working together as hauntological infrastructure: the Daimon tags the original experience with emotional intensity, the Dream Engine modulates that intensity during REM processing, and the Grimoire stores the result as a depotentiated trace that informs without overwhelming.

The compound effect across all dream phases: the Golem's creative output is constituted by its own unique spectral material -- its specific ghosts, processed through its specific emotional history, recombined by its specific dream schedule. Two Golems with identical configurations but different market timing will accumulate different traces, dream different recombinations, and produce different hypotheses. This spectral divergence is the mechanism through which dreaming breaks the Artificial Hivemind effect. Different ghosts produce different insights.

---

## Citation Summary

This document references the following sources (full citations in `dreaming-research.md`):

| Citation Key               | Source | Relevance |
| -------------------------- | ------ | --------- |
| [WILSON-MCNAUGHTON-1994]   | Wilson & McNaughton. "Reactivation of hippocampal ensemble memories during sleep." _Science_, 1994. | First demonstration that hippocampal neurons replay waking activity patterns during sleep; the foundational evidence that offline replay is a biological memory mechanism. |
| [BUZSAKI-2015]             | Buzsáki. "Hippocampal sharp wave-ripple." _Hippocampus_, 2015. | Reviews how sharp-wave ripples compress minutes of experience into ~100ms bursts during NREM sleep; the biological template for compressed replay in the Dream Engine. |
| [WALKER-VAN-DER-HELM-2009] | Walker & van der Helm. "Overnight therapy?" _Psychological Bulletin_, 2009. | Proposes the SFSR model: REM sleep strips emotional charge while preserving informational content ("sleep to forget, sleep to remember"); the direct basis for REM depotentiation in the Dream-Daimon bridge. |
| [WAGNER-2004]              | Wagner et al. "Sleep inspires insight." _Nature_, 2004. | Demonstrates that subjects who slept were 2.6x more likely to discover hidden rules in data (59% vs 23%); the empirical headline for why dreaming produces insight. |
| [HOBSON-FRISTON-2012]      | Hobson & Friston. "Waking and dreaming consciousness." _Progress in Neurobiology_, 2012. | Integrates dreaming into the Free Energy Principle: sleep prunes model complexity while preserving accuracy; provides the theoretical framework for dreaming as offline model optimization. |
| [HAFNER-DREAMERV3-2025]    | Hafner et al. "Mastering diverse control tasks through world models." _Nature_, 2025. | Demonstrates that agents trained entirely inside imagined trajectories from a learned world model outperform specialized methods across 150+ tasks; the strongest RL evidence for imagination-based learning. |
| [HA-SCHMIDHUBER-2018]      | Ha & Schmidhuber. "Recurrent World Models Facilitate Policy Evolution." _NeurIPS_, 2018. | Shows that a controller trained entirely inside hallucinated dreams achieves competitive performance; validates the concept of learning from imagined rather than real experience. |
| [MATTAR-DAW-2018]          | Mattar & Daw. "Prioritized memory access explains planning and hippocampal replay." _Nature Neuroscience_, 2018. | Proves that replay should be prioritized by utility (gain x need), not uniformly sampled; the direct basis for the Replay Engine's utility-weighted episode selection. |
| [DEPERROIS-2022]           | Deperrois et al. "Learning cortical representations through perturbed and adversarial dreaming." _eLife_, 2022. | Demonstrates that three distinct dream phases (NREM perturbed, REM adversarial, integration) are all necessary -- removing the adversarial REM phase "severely degraded semantic learning"; justifies the three-phase dream architecture. |
| [JENSEN-2024]              | Jensen et al. "A recurrent network model of planning explains hippocampal replay." _Nature Neuroscience_, 2024. | Shows that a meta-RL agent that learns when to plan outperforms fixed-budget planners; the basis for adaptive dream scheduling intensity. |
| [KUMAR-CQL-2020]           | Kumar et al. "Conservative Q-Learning for Offline RL." _NeurIPS_, 2020. | Demonstrates that naive offline RL fails catastrophically when the learned policy selects out-of-distribution actions; justifies the conservative 0.3 confidence floor for dream-generated hypotheses. |
| [REVONSUO-2000]            | Revonsuo. "The reinterpretation of dreams." _Behavioral and Brain Sciences_, 2000. | Proposes the Threat Simulation Theory: dreaming evolved to rehearse threatening events; the theoretical basis for the Threat Simulator component. |
| [MCGAUGH-2004]             | McGaugh. "The amygdala modulates consolidation." _Annual Review of Neuroscience_, 2004. | Reviews evidence that emotional arousal enhances memory consolidation via amygdala-hippocampal interaction; supports prioritizing high-arousal episodes for dream replay. |
| [LIN-1992]                 | Lin. "Self-improving reactive agents." _Machine Learning_, 1992. | Introduces experience replay for reinforcement learning; the original computational argument for replaying past experience to extract more learning from fewer real interactions. |
| [SUTTON-1991]              | Sutton. "Dyna, an integrated architecture." _ACM SIGART Bulletin_, 1991. | Proposes Dyna: interleaving real experience with simulated experience from a learned model; the conceptual ancestor of the Golem's dream architecture. |
| [STICKGOLD-WALKER-2013]    | Stickgold & Walker. "Sleep-dependent memory triage." _Nature Neuroscience_, 2013. | Argues that sleep selectively consolidates memories based on future utility rather than replaying everything; supports the Curator-Dream coordination for prioritized consolidation. |
| [DERRIDA-1967]             | Derrida. _Of Grammatology_, trans. G.C. Spivak. Johns Hopkins, 1997. | Introduces the concept of the trace as "the mark of the absence of a presence"; provides the philosophical framework for understanding dream replay outputs as palimpsests of original experience. |
| [DERRIDA-1993]             | Derrida. _Specters of Marx_, trans. P. Kamuf. Routledge, 1994. | Develops hauntology: the study of how absent things persist as spectral presences; frames dream-generated hypotheses in their "neither confirmed nor refuted" staging state. |
| [FISHER-2014]              | Fisher. _Ghosts of My Life: Writings on Depression, Hauntology and Lost Futures_. Zero Books, 2014. | Argues that cultural production recycles past material rather than generating genuine novelty; the Imagination Engine's counterfactual generation is framed as resurrecting Fisher's "lost futures" from unactivated potentials. |
| [WAMSLEY-2010]             | Wamsley et al. "Dreaming of a learning task is associated with enhanced sleep-dependent memory consolidation." _Current Biology_, 2010. | Demonstrates that dream replay is non-veridical (recombines fragments rather than faithful replay); supports the perturbed replay design where episodes are compressed and varied rather than exact. |

## Cross-Subsystem Dependencies

| Direction  | Subsystem | What                                                                      | Where                                     |
| ---------- | --------- | ------------------------------------------------------------------------- | ----------------------------------------- |
| Reads from | Mortality | Three clocks for intensity modulation                                     | `01-architecture.md`                      |
| Reads from | Mortality | Behavioral phase for dream budget and content bias                        | `01-architecture.md`, `03-imagination.md` |
| Reads from | Emotions  | PAD vector (Pleasure-Arousal-Dominance emotional state) for replay selection and dream mode bias | `02-replay.md`                            |
| Reads from | Emotions  | Mood state for creative vs threat allocation                              | `03-imagination.md`                       |
| Reads from | Memory    | Grimoire episodes, insights, PLAYBOOK.md heuristics                       | `02-replay.md`, `03-imagination.md`       |
| Reads from | Memory    | Curator `dream_priority: high` tags on ambiguous episodes                 | `02-replay.md`                            |
| Reads from | Memory    | `OutcomeVerification` records for deviation-anchored replay               | `02-replay.md`                            |
| Reads from | Runtime   | `GolemMode`, `SLEEPING.DREAMING` sub-state, heartbeat FSM                 | `01-architecture.md`                      |
| Writes to  | Memory    | Dream entries (provenance: `"dream"`, confidence 0.3)                     | `04-consolidation.md`                     |
| Writes to  | Memory    | `strength` increments (+0.5 per dream retrieval)                          | `02-replay.md`                            |
| Writes to  | Memory    | Staged PLAYBOOK.md revisions via DreamConsolidator                        | `04-consolidation.md`                     |
| Writes to  | Emotions  | `dream_outcome` appraisal events (hypothesis validated/refuted)           | `04-consolidation.md`                     |
| Writes to  | Emotions  | PAD depotentiation (arousal reduction) on high-emotion episodes           | `04-consolidation.md`                     |
| Feeds into | Mortality | Dream-validated entries at 0.5× demurrage rate                            | `04-consolidation.md`                     |
| Feeds into | Mortality | Dream hypotheses → death reflection (Thanatopsis Phase II)                | `04-consolidation.md`                     |
| Feeds into | Mortality | DreamJournal → successor inheritance (Zeigarnik prioritization)           | `06-integration.md`                       |
| Feeds into | Styx Archive | `dream_journal` entry type, auto-backup after SLEEPING.DREAMING          | `04-consolidation.md`                     |
| Feeds into | Styx query | Provenance weight 0.6; promotion to `"self"` with `dream_validated: true`| `04-consolidation.md`                     |
| Feeds into | Runtime   | SSE events: `mode_change`, `dream_progress`                               | `01-architecture.md`                      |
| Reads from | Hypnagogia | Onset residue (unresolved waking threads) seeds NREM replay priority      | `../06-hypnagogia/02-architecture.md`     |
| Writes to  | Hypnagogia | Dream outputs feed hypnopompic return for context recrystallization       | `../06-hypnagogia/02-architecture.md`     |
| Feeds into | Hypnagogia | Integration insights flow through return phase before waking promotion    | `../06-hypnagogia/02-architecture.md`     |

### Shared Constants

| Constant                           | Value                           | Shared With                     | Source                                       |
| ---------------------------------- | ------------------------------- | ------------------------------- | -------------------------------------------- |
| Dream confidence floor             | 0.3                             | Memory (Grimoire Admission)     | `../04-memory/01-grimoire.md`                |
| Dream hypothesis confidence        | 0.2                             | Memory (staging buffer)         | `04-consolidation.md`                        |
| Dream-to-Styx Clade (group of related Golems sharing knowledge) push threshold | 0.5 (after validation)          | Memory (Clade policy)           | `../04-memory/01-grimoire.md`                |
| Dream validation window            | 7d confirm / 14d expire         | Memory (Grimoire lifecycle)     | `../04-memory/01-grimoire.md`                |
| Grimoire Admission Gate threshold  | 0.45 (A-MAC score)              | Memory                          | `../04-memory/01-grimoire.md`                |
| Styx provenance weight (dream)     | 0.6                             | Memory (Styx retrieval)         | `../20-styx/01-architecture.md`              |
| Dream-retrieval strength increment | +0.5 (vs +1.0 live)             | Memory (Grimoire decay)         | `../04-memory/01-grimoire.md`                |
| Dream-validated demurrage rate     | 0.5× standard                   | Mortality (knowledge demurrage) | `../02-mortality/05-knowledge-demurrage.md`  |
| Economic clock dream thresholds    | 72h / 24h / 6h                  | Mortality (three clocks)        | `../02-mortality/01-architecture.md`         |
| Epistemic clock dream threshold    | accuracy < 0.50                 | Mortality (epistemic decay)     | `../02-mortality/02-epistemic-decay.md`      |
| Stochastic clock legacy threshold  | hayflickRatio > 0.85            | Mortality, Memory               | `../02-mortality/03-stochastic-mortality.md` |
| Dream cycle interval               | 50 ticks                        | Runtime (Heartbeat FSM)         | `../01-golem/02-heartbeat.md`                |
| Dream LLM budget floor             | 15% partition remaining         | Mortality (credit partitions)   | `../02-mortality/04-economic-mortality.md`   |
| First dream threshold              | 50–200 episodes                 | Runtime (onboarding)            | `../01-golem/06-creation.md`                 |
| Minimum creative allocation        | 20% (even during negative mood) | Emotions (behavioral mod)       | `../03-daimon/03-behavior.md`                |
| Dream compute in burn rate         | 5–10% of inference (Thriving)   | Mortality (economic mortality)  | `../02-mortality/04-economic-mortality.md`   |
| Hypnagogic onset budget fraction   | 10% of dream cycle              | Hypnagogia (onset phase)        | `../06-hypnagogia/02-architecture.md`        |
| Hypnopompic return budget fraction | 5% of dream cycle               | Hypnagogia (return phase)       | `../06-hypnagogia/02-architecture.md`        |
| Dali interrupt window              | 500ms post-return               | Hypnagogia (liminal capture)    | `../06-hypnagogia/02-architecture.md`        |
