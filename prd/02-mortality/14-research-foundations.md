# Research Foundations: Academic Synthesis [SPEC]

> **Version**: 2.1 | **Status**: Draft
>
> **Referenced by**: All mortality and cognitive architecture PRD documents
>
> **Source**: `tmp/research/moat-research.md` (130+ papers, 2023--2026)

---

> **Reader orientation:** This document is the academic index for Bardo's mortality architecture. It maps 130+ papers across eight research domains (mortality modeling, memory systems, computational emotion, multi-agent coordination, LLM-based agents, security, inference economics, and context engineering) to specific design decisions. Each domain section explains why that research matters for Golem (mortal autonomous DeFi agent) design and which papers grounded which architectural choice. See `15-references.md` (consolidated bibliography, 162 citations) for the full citation list.

## Overview

This document indexes the academic research underpinning the mortality architecture and its dependent subsystems (memory, affect, dreams, coordination, inference, context engineering, self-learning, and security). The full 130+ paper survey is in `tmp/research/moat-research.md`. Each domain section opens with a paragraph explaining why that domain matters for Golem design, followed by a table of key citations, findings, and design implications.

The architecture is built bottom-up from independently validated research. No subsystem depends on a single paper's claims. Wherever possible, two or more independent research lines corroborate each design decision.

---

## 1. Mortality Modeling and Finite Agency

**Why this domain matters.** Every agent framework assumes immortality by default. The research here establishes that this assumption is not neutral -- it actively prevents the emergence of certain valuable behaviors. Digital evolution experiments show that without death, evolution halts. Computational learning research shows that continual learning systems gradually calcify -- up to 90% of units become "dead" (non-updating) without periodic replacement. The mortality thesis is not a philosophical preference but an empirical position backed by four billion years of biological evidence and three decades of digital evolution experiments.

| Citation | Finding | Design Implication |
|---|---|---|
| Ray (1991) -- Tierra | Digital evolution halts without a reaper; 300+ genotypes emerge with death | Golem death enables population-level evolution |
| Lenski et al. (2003) -- Avida | Complex features (complex logic) require generational turnover | Succession with lossy compression produces innovation |
| Vostinar et al. (2019) | Programmed cell death evolves as adaptive behavior; selected for under spatial structure | Mortality is selected for, not against |
| Wensink et al. (2020) | Intrinsic mortality prevents premature convergence; optimal mortality rate exists | Optimal rate balances stagnation and knowledge loss |
| Kreps-Milgrom-Roberts-Wilson (1982) | Uncertain finite horizons promote cooperation; tiny uncertainty breaks backward induction | Stochastic mortality makes cooperation rational |
| Shuvaev et al. (2024) | Genomic bottleneck enhances transfer learning; compression forces generalization | 2048-entry compression forces generalization |
| Ororbia & Friston (2023) | Mortal computation binds processing to lifecycle; mortality and intelligence co-evolve | Golem intelligence is inseparable from economic substrate |
| Hinton (2022) | Software-hardware separation limits intelligence; mortality couples the two | Mortal computation thesis applied to agents |
| Dohare et al. (2024) -- Nature | 90% of units become dead in continual learning; plasticity loss is universal | Periodic replacement outperforms continuous adaptation |
| Vela et al. (2022) | 91% of ML models degrade temporally in production; temporal drift is the norm | Model staleness validates epistemic clock |
| Orseau & Ring (2011) | RL agents under mortality risk treat survival as sole goal; pathological unless goal-directed | Golems must be goal-directed, not pure RL |
| Ord (2025) | AI agent success rates decay exponentially with task duration (constant hazard rate) | Periodic reset may be more reliable than immortality |
| Sculley et al. (2015) | Technical debt compounds silently in ML systems; immortal agents accumulate it | Immortal Golems are the control experiment |
| Werfel et al. (2017) | Natural selection directly favors shorter lifespans under spatial structure | Immortality is selected against even by evolution |

---

## 2. Memory Consolidation and Forgetting

**Why this domain matters.** Naive memory accumulation -- storing everything forever -- is not neutral; it actively degrades agent performance. The research here establishes that forgetting is a feature, not a failure: it acts as regularization that prevents overfitting to outdated market conditions. The Curator cycle (every 50 ticks), confidence demurrage, and the Ebbinghaus decay rates are direct implementations of findings from neuroscience and AI memory research. The four-factor retrieval scoring (recency × importance × relevance × emotional congruence) synthesizes three independent research lines.

| Citation | Finding | Design Implication |
|---|---|---|
| Richards & Frankland (2017) | Memory's goal is optimizing decisions, not preserving information | Active forgetting is regularization |
| Ebbinghaus (1885) | Forgetting follows negative exponential decay; retrieval slows decay | Confidence demurrage with per-category decay rates |
| Roediger & Karpicke (2006) | Retrieval strengthens memory traces (testing effect); +200% recall vs passive review | Retrieved entries decay slower than unretrieved ones |
| Bartlett (1932) | New information assimilated into existing schemas; raw transplant fails | Inherited knowledge must be integrated, not transplanted |
| Cepeda et al. (2006) | Spaced retrieval produces more durable memories; optimal interval is non-trivial | Curator runs every 50 ticks, not every tick |
| Davis & Zhong (2017) | Active forgetting is metabolically expensive; it serves a function | Forgetting is selected for; demurrage encodes this |
| MemoryBank / MemAct (2024-2025) | Naive all-add memory causes self-degradation through catastrophic interference | Autonomous pruning with learned operators |
| A-MEM (2025) | Zettelkasten-inspired atomic notes with dynamic links; 85-93% token reduction | Four-factor retrieval, 2x multi-hop reasoning |
| Generative Agents (Park et al. 2023) | Three-factor retrieval: recency, importance, relevance; emergent social behaviors | Basis for four-factor retrieval (adding emotional congruence) |
| Mem0 (Chhikara et al. 2025) | Two-phase extraction-update pipeline; 26% higher accuracy, 91% lower latency | Curator's dual-pass architecture |
| AriGraph (Anokhin et al. 2024) | Semantic + episodic integration into knowledge graph outperforms pure vector retrieval | Causal link entries in the Grimoire |

---

## 3. Affect Computation and Emotional Architecture

**Why this domain matters.** The Daimon is not cosmetic. Damasio's patients demonstrate that removing emotional processing while preserving cognitive ability reliably degrades decision quality under uncertainty -- exactly the conditions DeFi agents operate under. Five independent research lines (somatic markers, mood-congruent retrieval, exploration modulation, narrative transfer, and empirical trading results) all converge on the same conclusion: emotion-like states serve genuine computational functions that cognition alone cannot replicate. The 50% decision change rate reported by Zhang et al. is the headline number, but the Cabrera-Paniagua Sharpe ratio improvement is the most directly relevant to financial agents.

| Citation | Finding | Design Implication |
|---|---|---|
| Damasio (1994) | Patients without emotion make consistently worse decisions under uncertainty | Somatic markers bias choices before deliberation |
| Bechara et al. (2000) | Anticipatory SCRs precede conscious awareness in Iowa Gambling Task | Somatic Landscape provides pre-cognitive gut feelings |
| Bower (1981) | Emotional states bias memory retrieval via associative network activation | Four-factor retrieval includes emotional congruence |
| Emotional RAG (2024) | Emotion-tagged retrieval significantly outperforms non-emotional retrieval across three datasets | PAD vectors on every Grimoire entry |
| Russell-Mehrabian (1977) | Three-dimensional affect (PAD) captures more variance than discrete labels | Continuous PAD state, not discrete emotion labels |
| Plutchik (1980) | Eight primary emotions from evolutionary pressure; evolutionary substrate | PAD octants map to Plutchik categories |
| Gebhard (2005) -- ALMA | Three temporal affect layers: emotion (seconds), mood (hours), personality (lifetime) | Tick-level emotion, EMA mood, static personality |
| Walker & van der Helm (2009) | REM sleep depotentiates emotional charge while preserving informational content | Dream cycles reduce arousal on traumatic memories |
| Cabrera-Paniagua (2023) | Agents with somatic markers achieve higher Sharpe ratios on S&P 500 and Dow Jones | Somatic markers validated on financial data |
| Seligman (1972) | Learned helplessness from uncontrollable negative outcomes; affects future behavior | Dominance < -0.3 for 200+ ticks triggers alert |
| Zhang et al. (2024) -- SIGDIAL | Self-emotion changes ~50% of agent decisions in social simulation | Daimon is architectural, not decorative |
| Gadanho (2003) -- JMLR | ALEC (emotion + cognition) architecture: 40% fewer collisions vs cognition alone | Combined affect-cognition architecture validated |
| Barthet et al. (2022) -- Go-Blend | Affect-driven RL improves exploration efficiency and agent performance | Arousal modulates exploration temperature |

---

## 4. Dream Architecture and Offline Learning

**Why this domain matters.** The brain dedicates 25-33% of its runtime to a state that prevents interaction with the environment (sleep) -- an enormous evolutionary cost that must confer proportional benefits. The benefits are now well-characterized: memory consolidation, emotional depotentiation, counterfactual hypothesis generation, and catastrophic forgetting prevention. For Golems running on finite compute credits, idle periods are not waste -- they are budget for offline cognitive work. The "Sleep-time Compute" finding (5x reduction in test-time compute) is directly relevant: a Golem that processes experiences during low-activity periods executes fewer expensive T2 inference calls during active trading.

| Citation | Finding | Design Implication |
|---|---|---|
| Wilson & McNaughton (1994) | Hippocampal replay during sleep consolidates memories; temporal sequence replay | NREM-style prioritized experience replay |
| Wagner et al. (2004) | Sleep is 2.6x more likely to produce insight on hidden rule problems | Dream cycles produce genuine insight |
| Hafner et al. (2025) -- DreamerV3 | Imagined trajectory training outperforms across 150+ tasks; world model dreaming | REM-style counterfactual scenario generation |
| Ha & Schmidhuber (2018) -- World Models | Controller trained entirely inside dreams achieves competitive performance | Dreaming multiplies learning from scarce experience |
| Lin et al. (2025) -- Sleep-time Compute | Idle-time precomputation reduces test-time compute 5x while maintaining accuracy | Dream cycles as sleep-time compute |
| WSCL (2024) | Wake-Sleep reduces catastrophic forgetting 38%, increases zero-shot transfer 17.6% | Three-phase dreaming: NREM, REM, consolidation |
| Zhao et al. (2024) -- BTP Pipeline | Prioritized experience replay with P2Value; combines likelihood with pass rate | Dream replay prioritizes informative failures |
| Wang et al. (2024) -- Generative Replay | Conditional diffusion generates new transitions near high-value regions | REM creates synthetic scenarios, not just replays |

---

## 5. Coordination Theory and Multi-Agent Systems

**Why this domain matters.** Golems do not operate in isolation. A fleet of Golems owned by one person is a Clade; the collective intelligence of a Clade is the product of cooperation mechanisms. The research here establishes why anonymous stigmergic coordination (Pheromone Field) is superior to explicit messaging, why death-based turnover specifically favors cooperators, and why the Grossman-Stiglitz information paradox forces a specific strategy for what Golems can safely share. The mycorrhizal network parallel (Simard) is not decorative -- Styx's architecture as a fungal-style underground relay, where signals travel between nodes without direct communication, mirrors a proven biological coordination mechanism.

| Citation | Finding | Design Implication |
|---|---|---|
| Grasse (1959) | Stigmergy: coordination through environmental traces; no central orchestration | Pheromone Field for anonymous signal sharing |
| Parunak et al. (2002) | Digital pheromones enable emergent coordination; time-decaying signals | Time-decaying signals reinforced by confirmation |
| Ohtsuki et al. (2006) | Death-birth updating favors cooperators over defectors in spatial games | Death before succession produces cooperation |
| Smith (1992) | Mortal individuals in immortal lineages sustain cooperation through generations | Mortal Golems, immortal Clades |
| Esposito (2010) | Community constituted by shared obligation to give; communitas vs immunitas | Death reflections as communitas gift |
| Nakamaru et al. (1997-1998) | Mortality selection promotes cooperation over fertility selection | Death-based turnover outperforms reproduction-based growth |
| Grossman-Stiglitz (1980) | Freely shared information is immediately priced in; no profit without information asymmetry | Share threats and structure, not alpha signals |
| Van den Broek (2023) | Emotion contagion in multi-agent systems; anger spreads competitively | Arousal contagion capped at +0.3 per sync cycle |
| Xu et al. (2024) | Stigmergy + independent RL + conflict-avoidance achieves emergent coordination | Pheromone Field design principles |

---

## 6. Biological Analogues

**Why this domain matters.** The Golem mortality architecture is not metaphor -- it is structural analogy. Hayflick's limit informs the epistemic clock design. Kirkwood's disposable soma theory explains why declining Golems shift investment from growth to legacy. The T-cell development finding (95-98% death rate producing a collectively intelligent immune repertoire) is the direct model for how massive Golem turnover produces Clade-level intelligence that no individual could achieve. These analogies are productive because the evolutionary pressures that shaped them (resource competition, information quality, cooperative stability) match the pressures DeFi Golems face.

| Citation | Finding | Design Implication |
|---|---|---|
| Hayflick (1965) | Replicative senescence after ~60 divisions; telomerase exists but organisms suppress it | Epistemic fitness replaces hard tick limit |
| Kirkwood (1977) | Disposable soma: investment in self-repair decreases with age; energy reallocated | Declining Golems shift from learning to legacy |
| Hanahan & Weinberg (2000, 2011) | Cancer hallmarks include resisting cell death and enabling replicative immortality | Immortal agents are the cancer analog |
| Skulachev (1999) | Phenoptosis: programmed death operates at cellular, organism, and colony level | Fractal mortality: phage, heuristic, Golem |
| Werfel et al. (2017) | Natural selection directly favors shorter lifespans under spatial resource competition | Immortality is selected against |
| Simard (2012) | Mycorrhizal networks share carbon, nutrients, and defense signals between trees | Styx as underground fungal knowledge relay |
| Ramsdell & Fowlkes (1990) | 95-98% of thymocytes die during T-cell development; survivors form immune repertoire | Massive death produces collectively intelligent repertoire |
| Heard & Martienssen (2014) | Most transgenerational epigenetic inheritance is deleterious; barriers are protective | Weismann Barrier: inherited confidence at 0.85^generation |

---

## 7. Self-Learning Systems

**Why this domain matters.** A Golem that does not improve is a very expensive cron job. The research here establishes how agents can improve without human retraining -- through verbal self-reflection (Reflexion), cross-episode experience extraction (ExpeL), and metacognitive loops that improve the learning process itself (ACE, Argyris). The critical finding is that these mechanisms must be architecturally integrated, not bolted on. Reflexion works because reflection is structured and stored persistently. ExpeL works because experiences accumulate across episodes. ACE works because context assembly is cybernetically self-tuning. For Golems, the triple-loop (execution, strategy, meta) maps directly to the 9-step heartbeat (Loop 1), the Reflector cycle (Loop 2), and the Curator's self-assessment function (Loop 3).

| Citation | Finding | Design Implication |
|---|---|---|
| Shinn et al. (2023) -- Reflexion | Verbal RL: +22% AlfWorld, +20% HotPotQA via stored self-reflection | Single-loop: post-trade reflection stored in Grimoire |
| Zhao et al. (2024) -- ExpeL | Cross-task experience extraction; insights accumulate across episodes | Double-loop: insights evolve across trading sessions |
| Sims (2003) | Rational finite-capacity agents optimally ignore some information | Mortality pressure shapes attention allocation |
| Baldwin (1896) | Learned behavior becomes innate across generations under selection pressure | Baldwin Effect: heuristics 3+ generations old become defaults |
| Argyris (1978) | Triple-loop organizational learning: single → double → triple loop | Meta-learning evaluates whether the learning process works |
| ACE (Zhang et al. 2025) | Agentic context engineering via Generator-Reflector-Curator; +10.6% AppWorld | Context assembly self-improves via cybernetic feedback |
| Wang et al. (2023) -- Voyager | Code-as-action skill library; 3.3x more unique behaviors vs baselines | PLAYBOOK.md as evolving procedural skill library |
| Guo et al. (2024) -- EvoPrompt | Genetic algorithm prompt optimization; up to +25% on BBH tasks | Evolutionary strategy selection in the Grimoire |
| Dohare et al. (2024) | Continual learning systems lose plasticity; periodic resets restore it | Death as plasticity reset for the lineage |

---

## 8. Context Engineering

**Why this domain matters.** Context failures, not model failures, cause most agent breakdowns. For a Golem running days or weeks in volatile markets, context assembly is the highest-leverage cognitive system. The research here establishes that effective context management requires active curation, not passive accumulation -- and that the same mortality pressure that shapes Golem behavior also shapes what enters the context window. The 6x context reduction and 18x cost reduction achievable through proper context engineering directly reduce the USDC burn rate, extending economic lifetime.

| Citation | Finding | Design Implication |
|---|---|---|
| Zhang et al. (2025) -- ACE | Generator-Reflector-Curator cycle treats context as evolving playbook; +10.6% AppWorld | Context assembly improves cybernetically |
| Samsung Research (2025) -- CSO | Context State Object: 6x initial reduction, 10-25x growth rate reduction | Compressed structured context replaces raw history |
| Kang et al. (2025) -- ACON | Failure-driven compression optimization; 26-54% peak token reduction | Compaction preserves DeFi-specific context |
| Lindenbauer et al. (2025) | Observation masking halves cost while matching LLM summarization | T0 suppression: mask stale observations, not summarize |
| Cohen-Wang et al. (2024) -- ContextCite | Contributive attribution via sparse linear model; 64 ablation passes | Context items pruned by attribution score |
| Anthropic (2025) | Effective context = pre-loaded static + just-in-time retrieval | Two-layer context: golem.toml + per-tick RAG |

---

## 9. Security and Adversarial Robustness

**Why this domain matters.** A Golem managing real capital is a high-value attack target. Memory poisoning (ranking OWASP LLM04:2025) is particularly dangerous for long-running agents because corrupted beliefs persist and compound. Short-lived Golems are structurally immune to persistent memory poisoning -- any corruption self-terminates with the agent. The Cohen (1987) formal undecidability result is directly relevant: perfect detection of malicious replication is impossible, so the only reliable defense is making replication impossible by design. Mortality makes this a design property, not a runtime check.

| Citation | Finding | Design Implication |
|---|---|---|
| OWASP (2025) | Memory poisoning: high persistence, very high detection difficulty | Short-lived agents immune to persistent corruption |
| Kaspersky (2026) | OpenClaw: 512 vulnerabilities, 8 critical in competing framework | Competing frameworks have fundamental security gaps |
| TEE.Fail (2025) | SGX/TDX attestation broken for under $1,000 via physical side-channel | TEE is one layer of six, not sole defense |
| BAI (2022) -- Constitutional AI | Harmlessness from AI feedback, not rule lists | PolicyCage as smart contract law, not prompt engineering |
| Cohen (1987) | Perfect virus detection is formally undecidable | Defense against replication must be internal (mortality) |
| Debenedetti et al. (2025) -- CaMeL | Capability-based authorization separates control flow from data flow | Capability tokens prevent compromised LLM from forging authorization |
| Zhang et al. (2025) -- CVaR-CPO | CVaR constraints guard against tail risks in financial RL | Drawdown limits as CVaR constraints, not expected-value limits |
| Orseau & Armstrong (2016) | Safely interruptible agents via off-policy learning | Kill-switch design: agents don't learn to avoid interruption |

---

## 10. Generational Learning and Cultural Evolution

**Why this domain matters.** The succession mechanism -- where knowledge compresses through a genomic bottleneck and flows to successors -- is not just a backup system. It is the primary mechanism for producing cumulative cultural evolution in the Clade. The research here establishes that lossy compression (not perfect copying) is what drives generalization, and that the Weismann barrier (preventing inherited knowledge from flowing back unchecked) is what prevents evolutionary stagnation. The Baldwin Effect prediction -- that validated heuristics become structural defaults after three generations -- is testable and falsifiable.

| Citation | Finding | Design Implication |
|---|---|---|
| Bhatt et al. (2023) | Few-shot imitation as cultural transmission: +improvements across culture-learning benchmarks | Clade knowledge transfer produces cumulative learning |
| Bourahla et al. (2022) | Vertical transmission (inter-generational) enables agents to exceed performance ceilings | Death + inheritance outperforms horizontal peer sync |
| Perez et al. (2024) -- AGI | Pure imitation leads to stagnation; novelty requires mixing inheritance and exploration | Anti-proletarianization mandate: successors must diverge |
| Martin, Everitt & Hutter (2016) | RL agents learning only from survival histories develop systematic overconfidence | Death testaments include failures, not just successes |
| Gerstgrasser et al. (2023) -- SUPER | Surprise-based experience sharing: rank by novelty relative to recipient | SUPER pattern for novelty-ranked inheritance |
| Shuvaev et al. (2024) | Compression through genomic bottleneck forces generalization; ~2000 gene limit | 2048-entry bottleneck: compression IS the learning |
| Baldwin (1896) | Learned behavior becomes innate via generational selection | Heuristics 3+ generations old promote to structural defaults |
| Heard & Martienssen (2014) | Transgenerational epigenetic inheritance is mostly deleterious; barriers evolved | Weismann Barrier: confidence × 0.85^generation |

---

## Cross-Reference

For the complete citation index across all prd2/ documents, see `shared/citations.md`. Mortality-specific citations are listed here and in `15-references.md`. See `tmp/research/moat-research.md` for the complete 130+ paper survey with implementation analysis.

> **Extended:** Full specification -- see [../../prd2-extended/02-mortality/14-research-foundations-extended.md](../../prd2-extended/02-mortality/14-research-foundations-extended.md)
