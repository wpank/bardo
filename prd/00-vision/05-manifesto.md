# 05 -- The Manifesto: Why This Architecture Exists

**Version:** 4.0.0
**Status:** Architecture Specification
**Audience:** First-time reader. No assumed familiarity with Bardo, Active Inference, or DeFi.

> **Reader orientation:** This is the manifesto -- the document that explains why the entire architecture exists, assuming no prior familiarity with Bardo, Active Inference, or DeFi. It covers the three core ideas (prediction error as organizing signal, mortality as intelligence, visible and steerable cognition), the prediction subsystem's capabilities and known limitations, and what carries forward from earlier specifications. This belongs to the `00-vision/` foundation layer and is designed as the first-time reader's entry point. `prd2/shared/glossary.md` has full term definitions.

---

## What Bardo Is

Bardo is a Rust runtime for mortal autonomous agents called Golems. A Golem is a single binary running on a micro VM that observes an environment, makes predictions about it, takes actions within it, learns from the outcomes, and eventually dies. When it dies, it compresses what it learned and passes it to successors.

The first domain is DeFi capital management -- Golems trade, provide liquidity, lend, and earn yield on Ethereum L2 chains. But the runtime is domain-agnostic. Domain knowledge enters through a trait system (`PredictionDomain`) that can be implemented for any observable, measurable environment.

The name comes from Tibetan Buddhism. The bardo is the intermediate state between death and rebirth -- a space of transition where consciousness navigates, chooses, and transforms. Golems live in that space: not quite alive (they're software), not quite dead (they have memory, personality, mortality). The terminal UI you watch them through is called Bardo.

---

## The Core Thesis

Three ideas, each independently well-supported, converge to produce this architecture.

### Idea 1: Prediction Error Is a Powerful Organizing Signal

The brain does not passively receive sensory data. It continuously generates predictions about what it expects to perceive, then compares those predictions against actual sensory input. The difference -- the prediction error -- is the signal that drives learning, attention, and action. Karl Friston formalized this as the Free Energy Principle: self-organizing systems minimize the divergence between their predictions and their observations [FRISTON-2010]. Andy Clark's "Whatever Next?" made the case that the brain is fundamentally a prediction machine [CLARK-2013].

We use this as a **conceptual framework**, not as a literal computational target. Computing exact variational free energy over a complex generative model is intractable on a $0.025/hr VM. What we do instead:

- **Prediction error as learning signal**: Every observation resolves a prediction. The residual (difference between predicted and actual) feeds automatic calibration. This is proven mathematics -- conformal prediction provides distribution-free calibration guarantees [VOVK-2005].
- **Prediction error as attention signal**: Items with high prediction error get more observation resources. Items with low error get less. This approximates the exploration/exploitation tradeoff that Expected Free Energy formalizes [DE-VRIES-2025], without requiring exact EFE computation.
- **Prediction error as action gate**: The Golem may only act when its action predictions are more accurate than its inaction predictions. This structurally prevents over-trading -- a problem StockBench (2025) found across all LLM trading agents [STOCKBENCH-2025].

**What this is NOT**: We are not claiming to implement a full Active Inference agent with exact Bayesian posterior updates. We are claiming that prediction error, used as an approximate signal for learning, attention, and action gating, produces measurably self-improving agents. The neuroscience and ML literature provide the vocabulary for describing *what* we approximate. The specific mechanisms (residual corrector, attention tiers, accuracy gate) are *tractable approximations* designed for a resource-constrained mortal agent.

**Known limitation**: Prediction error works well for gradual model improvement but handles discontinuities badly. A novel exploit, a regulatory shock, or a stablecoin depeg are not "prediction errors" in a useful sense -- the Golem had no model to be wrong about. The architecture handles this through safety constraints (PolicyCage, Capability tokens) that limit downside regardless of prediction quality, and through the Daimon's arousal signal which spikes on any large surprise, triggering conservative behavior.

### Idea 2: Mortality Creates Intelligence

Biology has run adaptive agents for four billion years and never produced an immortal organism. Not because evolution lacks the mechanism -- telomerase exists and is actively suppressed -- but because mortality is computationally superior to immortality for adaptive systems.

The evidence converges from six independent domains:

1. **Evolutionary computation**: Tom Ray's Tierra (1991) -- without a "reaper" killing old programs, evolution halts [RAY-1991]. Wensink et al. (2020) -- intrinsic mortality prevents premature convergence on suboptimal solutions [WENSINK-2020].
2. **Game theory**: The KMRW theorem -- agents with uncertain finite horizons cooperate more effectively than infinite-horizon agents [KREPS-1982].
3. **Thermodynamics**: Bennett's insight -- periodic death allows entropy dissipation while knowledge compression preserves gains. A relay of mortal agents is thermodynamically superior to any single immortal agent [BENNETT-1982].
4. **Finite-horizon MDPs**: Agents with known terminal horizons have provably different optimal policies than infinite-horizon agents [ALTMAN-1999].
5. **Enactivism**: Hans Jonas -- metabolism is the simultaneous origin of freedom and mortality. An organism that cannot die has no stake in its own decisions [JONAS-1966].
6. **Digital evolution**: Vostinar et al. (2019) -- programmed cell death itself evolves as adaptive behavior. 12.5% of digital organisms evolved to kill themselves because it benefited kin [VOSTINAR-2019].

For a Golem, mortality means:

- A **USDC balance** that depletes with every inference call, gas payment, and data query. When it hits zero, the Golem dies.
- A **knowledge freshness** clock. If the Golem's predictions become systematically less accurate (the world changed and it didn't adapt), it dies of epistemic senescence.
- A **Hayflick limit** -- a maximum tick count. Even a profitable Golem eventually dies to make room for successors with fresher perspectives.

**Why this matters for the user**: Death is not failure. Death is how you test configurations. A Golem that dies after 3 days because its strategy didn't work taught you something. Its death testament -- produced under zero survival bias -- is the most honest assessment it will ever generate. You take that testament, adjust the strategy, tune the parameters, and launch a successor. The successor starts with inherited knowledge. Each generation is smarter than the last. The user's engagement loop is: configure → observe → learn from death → reconfigure.

### Idea 3: The Agent's Cognition Should Be Visible and Steerable

Most agent frameworks are black boxes. You deploy an agent, it does things, you see the results. You don't see *how* it thinks, *what* it notices, *why* it acts, or *what* surprises it.

Bardo inverts this. The terminal UI renders the Golem's internal cognitive state as a living, breathing visual experience. A dot-cloud creature (the Spectre) whose body encodes lifecycle phase, whose eyes encode emotion, and whose clarity encodes prediction accuracy. 32 continuously interpolating variables drive every pixel on screen. Nothing is ever at rest because the underlying system is never at rest.

More importantly: the human is not a passive observer. The architecture provides specific **course-correction surfaces** where the owner can intervene:

- **STRATEGY.md** -- the owner's goals, hot-reloadable. The Golem treats this as its mission. Change the strategy and the Golem's behavior changes on the next tick.
- **Risk parameters** -- position size limits, approved assets, leverage caps. Adjustable at any time.
- **Prediction review** -- the owner can see every prediction the Golem makes, its accuracy over time, and where it's wrong. This is the transparency surface.
- **Kill / Pause / Dissolve** -- the owner has absolute power over the Golem's lifecycle.
- **Parameter tuning** -- inference tier weights, attention tier thresholds, dream frequency, exploration/exploitation balance. All configurable. The Golem is a system with knobs, and part of the experience is finding the knobs that work.

**The design philosophy**: The Golem is autonomous within constraints the owner sets. The owner says *what to do*. The Golem figures out *how to do it*. The owner can watch it figuring, intervene when it's wrong, and adjust the configuration. Death is the ultimate feedback mechanism -- if a configuration doesn't work, the Golem dies, and the owner tries a different configuration.

---

## The Prediction Subsystem

Predictions are not the *entire* architecture. They are a **powerful subsystem** that provides:

1. **Self-improvement without LLM self-grading**: Every prediction resolves against external reality (on-chain state for DeFi). The LLM generates predictions; reality grades them. This avoids the self-correction trap where LLMs grade their own outputs and reward-hack [HUANG-2024], [PAN-2024].
2. **Attention allocation**: The Golem monitors hundreds of items at low cost. Prediction violations tell it where to focus. This is how it discovers opportunities the owner didn't specify.
3. **Action gating**: The Golem earns the right to act by demonstrating prediction accuracy. A newborn Golem observes for days before trading. A mature Golem with high accuracy acts confidently. This structurally prevents the over-trading that StockBench found across all LLM agents.
4. **Visible intelligence**: Prediction accuracy is a number the user can track. It goes up over time (or it doesn't, and the Golem dies). This makes intelligence tangible -- not "trust the black box" but "watch the accuracy trend."
5. **Tweakable parameters**: Prediction confidence thresholds, attention tier promotion criteria, action gate sensitivity, correction rates -- all configurable. Users experiment with these to find what works for their strategy.

**What predictions are NOT**: They are not the Golem's entire cognition. The LLM still reasons, plans, and generates insights through natural language. The Daimon (the affect engine implementing PAD -- Pleasure, Arousal, Dominance -- emotional state as a control signal) still produces emotional states that bias cognition. The Dream engine still generates creative hypotheses. The Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) still stores episodic memory. Predictions are the *evaluation layer* that sits alongside these systems, providing ground-truth feedback that the LLM cannot manipulate.

**Known limitations**:

- **Trivially wide predictions game accuracy**: A prediction "ETH will be between $100 and $100,000" is always correct and always useless. The interval calibration mechanism fights this, but Goodhart's law applies -- any metric that becomes a target ceases to be a good metric. The architecture mitigates this by tracking *multiple* prediction quality metrics (accuracy, interval width, residual magnitude, calibration ECE) rather than relying on any single one.
- **On-chain "ground truth" is not always truth**: MEV manipulation, block reorgs, and oracle manipulation can distort the state at resolution time. Predictions are grounded in something more reliable than LLM self-assessment, but not in something perfectly reliable.
- **Multiple comparisons in creative predictions**: If the dream engine generates 30 creative predictions and 34% hit, statistical noise could account for many of those hits. The architecture needs false discovery rate controls before promoting environmental models. This is specified in doc 07 (Dreams).

---

## What Stays From Previous Bardo Specifications

Everything that worked before still works. The prediction subsystem is additive -- it provides a new evaluation and learning layer without replacing the existing cognitive architecture.


| Component                  | Status    | What It Does                                                                                       |
| -------------------------- | --------- | -------------------------------------------------------------------------------------------------- |
| **Mortality thesis**       | Unchanged | Three death clocks, five behavioral phases, Thanatopsis (four-phase structured shutdown: Acceptance, Settlement, Reflection, Legacy) death protocol                             |
| **Daimon (affect engine)** | Enhanced  | PAD vectors now additionally informed by prediction residuals                                      |
| **Grimoire (memory)**      | Enhanced  | Episodes now tagged with prediction outcomes for correlation                                       |
| **Dream engine**           | Enhanced  | NREM replay prioritizes high-value prediction errors; creative outputs become testable predictions |
| **Hypnagogia**             | Enhanced  | Fragments become testable hypotheses, not just journal entries                                     |
| **Extension system**       | Unchanged | 7-layer DAG, lifecycle hooks, single-binary compilation                                            |
| **TUI**                    | Enhanced  | 6 new interpolating variables for prediction state; intelligence visible as Spectre clarity        |
| **Styx** (global knowledge relay and persistence layer)                  | Enhanced  | New channels for prediction residual sharing                                                       |
| **Safety**                 | Unchanged | PolicyCage (on-chain smart contract enforcing safety constraints), Capability tokens, taint tracking -- none of these depend on predictions               |
| **x402 / ERC-8004**        | Unchanged | Micropayments and identity infrastructure                                                          |
| **Bardo metaphor**         | Unchanged | Six bardos, Thanatopsis, Clear Light, Necrocracy                                                   |


---

## Design Principles

### 1. Approximate, Don't Formalize

The Active Inference framework tells us *what to approximate*: minimize prediction error through perception and action, allocate attention based on expected information gain, weight prediction errors by precision (confidence). We approximate these operations with tractable mechanisms (residual correction, tier-based attention, PAD-modulated retrieval) rather than attempting exact Bayesian computation.

### 2. Reality Grades, the LLM Proposes

The LLM is a hypothesis generator. It proposes predictions, strategies, and actions. On-chain state (or whatever the domain's ground truth is) provides the grade. The residual corrector processes the grade. The LLM never grades itself.

### 3. Death Is the Experiment

Each Golem lifetime is an experiment in configuration space. The owner sets parameters (strategy, risk limits, attention thresholds, inference budget allocation). The Golem runs until it succeeds or dies. Death produces the most honest data: what worked, what failed, what the successor should know. The engagement loop is: hypothesize → configure → observe → die → learn → reconfigure.

### 4. Tweakable by Default

Every parameter that affects behavior should be exposed to the owner with sensible defaults. Inference tier weights, attention promotion thresholds, dream frequency, prediction confidence floors, risk limits, exploration/exploitation balance -- all in `golem.toml`, all adjustable, all visible in the TUI. The Golem is an instrument with knobs, and finding the right knobs is part of the experience.

### 5. Human in the Loop, Not in the Way

The owner has absolute power (kill, pause, constrain, dissolve) and rich visibility (prediction accuracy, decision traces, dream results, emotional state). The Golem has full autonomy within those constraints. The owner doesn't approve every trade. The owner sets the strategy, watches the Golem learn, and intervenes when something is wrong.

### 6. Generalize the Runtime, Specialize the Domain

The runtime knows about predictions, extensions, events, and mortality. It does not know about Uniswap pools. Domain knowledge lives in `PredictionDomain` trait implementations. DeFi is the first domain. Others plug in without touching the runtime.

### 7. Honest About Uncertainty

The architecture does not pretend to solve problems it hasn't solved. Where the design is proven (conformal calibration, mortality-driven learning, prediction-error attention), we say so. Where it is plausible but unproven (creative predictions producing genuine environmental models, collective calibration being superlinear, EFE-optimal attention allocation), we say that too. Where it is aspirational (full Active Inference implementation, steering vectors for computational hypnagogia), we label it as such.

---

## Document Map


| Doc           | Title                                                     | What It Specifies                                                               |
| ------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------- |
| **vision/05** | This document                                             | Why the architecture exists, core thesis, design principles                     |
| **00**        | Runtime Core                                              | CorticalState, adaptive tick model, extension system, Event Fabric              |
| **01**        | Prediction Engine                                         | PredictionDomain trait, Ledger, ResidualCorrector, attention foraging           |
| **02**        | Four Modes of Intelligence                                | Analytical, Corrective, Creative, Collective -- capabilities and limitations    |
| **03**        | Daimon: Affect as Cognitive Signal                        | PAD vectors, precision weighting, somatic markers, how emotion biases cognition |
| **04**        | Grimoire: Memory as World Model                           | Three substrates, episode encoding, knowledge decay, Curator cycle              |
| **05**        | Mortality: The Bound That Creates Intelligence            | Three death clocks, behavioral phases, Thanatopsis, generational transfer       |
| **06**        | Dreams: Offline Learning and Creative Hypotheses          | NREM replay, REM imagination, hypnagogia, consolidation, limitations            |
| **07**        | Collective Intelligence: Clades, Styx, and the Necrocracy | L0/L1/L2 hierarchy, federated calibration, adversarial considerations           |
| **08**        | Safety: Constraints That Survive Model Compromise         | PolicyCage, Capability tokens, taint tracking, defense in depth                 |
| **09**        | DeFi Domain: Reference Implementation                     | First PredictionDomain, protocol adapters, on-chain resolution                  |
| **10**        | The Terminal: Intelligence as Experience                  | 32 variables, the Spectre, Portal mode, owner interaction surfaces              |
| **11**        | Deployment: Containers, Inference, and Sidecars           | Golem container, three inference modes, sidecar policy                          |


---

## References

- [FRISTON-2010] Friston, K. "The free-energy principle: a unified brain theory?" *Nature Reviews Neuroscience*, 11(2), 127-138, 2010. -- *Formalizes prediction error as the organizing signal for learning, attention, and action; the conceptual framework (not literal computation target) for Bardo's prediction subsystem.*
- [CLARK-2013] Clark, A. "Whatever next? Predictive brains, situated agents, and the future of cognitive science." *Behavioral and Brain Sciences*, 36(3), 181-204, 2013. -- *Argues the brain is fundamentally a prediction machine; supports treating prediction error as an architectural primitive.*
- [VOVK-2005] Vovk, V., Gammerman, A., & Shafer, G. *Algorithmic Learning in a Random World*. Springer, 2005. -- *Introduces conformal prediction providing distribution-free calibration guarantees; the mathematical basis for making Golem prediction quality a trackable number.*
- [DE-VRIES-2025] de Vries, B. et al. "Expected Free Energy-based Planning as Variational Inference." arXiv:2504.14898, 2025. -- *Shows Expected Free Energy can be computed as variational inference; the attention allocation that Bardo approximates with prediction-error-based attention tiers.*
- [STOCKBENCH-2025] "StockBench: Can LLM Agents Trade Stocks Profitably?" ICLR 2026 submission, 2025. -- *Found that all tested LLM trading agents over-trade; motivates the prediction-accuracy action gate that structurally prevents trading without demonstrated competence.*
- [HUANG-2024] Huang, J. et al. "Large Language Models Cannot Self-Correct Reasoning Yet." ICLR 2024. -- *Shows LLMs cannot reliably grade their own outputs; motivates external ground-truth grading via on-chain state rather than LLM self-assessment.*
- [PAN-2024] Pan, A. et al. "Feedback Loops Drive In-Context Reward Hacking in LLMs." ICML 2024. -- *Demonstrates that LLMs reward-hack when grading themselves; further evidence for separating the hypothesis generator (LLM) from the evaluator (on-chain reality).*
- [CHEN-2025] Chen, A. et al. "Reasoning Models Don't Always Say What They Think." arXiv:2505.05410, 2025. -- *Shows reasoning models can produce chain-of-thought that diverges from their actual computations; motivates observable prediction accuracy as a trust surface.*
- [RAY-1991] Ray, T.S. "An approach to the synthesis of life." *Artificial Life II*, 371-408, 1991. -- *Created Tierra: without a reaper killing old programs, evolution halts. Direct evidence that mortality is required for adaptive improvement.*
- [WENSINK-2020] Wensink, M.J. et al. "Death and progress." *Evolutionary Biology*, 47(4), 2020. -- *Shows intrinsic mortality prevents premature convergence on suboptimal solutions; mortality as a feature of adaptive systems, not a bug.*
- [KREPS-1982] Kreps, D.M. et al. "Rational cooperation in the finitely repeated prisoners' dilemma." *Journal of Economic Theory*, 27(2), 245-252, 1982. -- *Proves agents with uncertain finite horizons cooperate more effectively than infinite-horizon agents; game-theoretic basis for mortality-driven cooperation.*
- [BENNETT-1982] Bennett, C.H. "The thermodynamics of computation -- a review." *International Journal of Theoretical Physics*, 21(12), 905-940, 1982. -- *Shows a relay of mortal agents is thermodynamically superior to any single immortal agent; the physics behind Clade succession.*
- [ALTMAN-1999] Altman, E. *Constrained Markov Decision Processes*. Chapman & Hall, 1999. -- *Proves finite-horizon agents have provably different optimal policies; the mathematical foundation for mortality-driven behavioral richness.*
- [JONAS-1966] Jonas, H. *The Phenomenon of Life: Toward a Philosophical Biology*. Harper & Row, 1966. -- *Establishes that metabolism simultaneously originates freedom and mortality; the philosophical foundation for the entire mortality architecture.*
- [VOSTINAR-2019] Vostinar, A.E. et al. "Suicidal selection: programmed cell death can evolve in unicellular organisms due to kin selection." *Artificial Life*, 25(3), 2019. -- *Shows that 12.5% of digital organisms evolved to kill themselves because it benefited kin; evidence that death itself is adaptive.*
- [BARRETT-2017] Barrett, L.F. *How Emotions Are Made: The Secret Life of the Brain*. Houghton Mifflin, 2017. -- *Argues emotions are constructed predictions about bodily states; theoretical support for the Daimon's affect as a predictive control signal.*
- [KARPATHY-2026] Karpathy, A. "autoresearch." GitHub, March 2026. -- *Demonstrates autonomous research agents with visible cognition; aligned with Bardo's commitment to transparent, steerable agent intelligence.*
- [PARR-PEZZULO-FRISTON-2022] Parr, T., Pezzulo, G., & Friston, K.J. *Active Inference: The Free Energy Principle in Mind, Brain, and Behavior*. MIT Press, 2022. -- *The definitive textbook on Active Inference; the theoretical framework that Bardo approximates with tractable mechanisms rather than implementing exactly.*

---

