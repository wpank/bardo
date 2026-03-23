# Research: Academic Foundations [SPEC]

Consolidates all academic references underpinning the Bardo Memory architecture, organized by research domain. This file is a pointer index -- full annotations and competitive analysis live in the extended file.

> **Reader orientation:** This document indexes the academic literature that grounds the Grimoire (the Golem's persistent local knowledge base) and broader memory architecture. It belongs to the **04-memory** layer. The key concept is that every design decision in the Golem's memory system traces to published cognitive science, neuroscience, or evolutionary biology. This is a pointer index organized by research domain; annotations explain what each work argues and how it connects to the Bardo implementation. For term definitions, see `prd2/shared/glossary.md`.

> **Extended:** Full specification -- see [../../prd2-extended/04-memory/10-research-extended.md](../../prd2-extended/04-memory/10-research-extended.md)

---

## Memory consolidation and forgetting

The theoretical core. Forgetting is not failure; it is regularization.

- **[EBBINGHAUS-1885]** Ebbinghaus, H. _Memory: A Contribution to Experimental Psychology_. 1885. The forgetting curve. Half-life calibration for decay classes (episodes 48h, insights 7d, heuristics 14d, warnings 30d) derives from this.
- **[RICHARDS-FRANKLAND-2017]** Richards, B.A. & Frankland, P.W. "The Persistence and Transience of Memory." _Neuron_, 94(6), 2017. The paper that reframes forgetting as optimization. Memory pruning = L1 regularization. Foundational for the entire Grimoire decay architecture.
- **[WALKER-VAN-DER-HELM-2009]** Walker, M.P. & van der Helm, E. "Overnight Therapy? The Role of Sleep in Emotional Brain Processing." _Psychological Bulletin_, 135(5), 2009. Sleep to Forget, Sleep to Remember (SFSR) model. Grounds the Dream Engine's REM depotentiation mechanism.
- **[STICKGOLD-2005]** Stickgold, R. "Sleep-Dependent Memory Consolidation." _Nature_, 437, 2005. Sleep replay strengthens memory traces selectively. Grounds NREM replay selection in dream processing.
- **[BORN-WILHELM-2012]** Born, J. & Wilhelm, I. "System Consolidation of Memory During Sleep." _Psychological Research_, 76, 2012. Sleep-dependent consolidation biased toward future-relevant memories. Maps to the Curator cycle's selective promotion.
- **[NADER-2000]** Nader, K. et al. "Fear Memories Require Protein Synthesis in the Amygdala for Reconsolidation after Retrieval." _Nature_, 406, 2000. Reconsolidation theory: retrieved memories become labile and can be updated. Justifies the confidence-update-on-retrieval mechanism.
- **[HARDT-NADER-NADEL-2013]** Hardt, O. et al. "Decay Happens: The Role of Active Forgetting in Memory." _Trends in Cognitive Sciences_, 17(3), 2013. Active molecular forgetting processes. Grounds the Curator's DOWNVOTE and pruning operations.
- **[MCCLELLAND-1995]** McClelland, J.L., McNaughton, B.L., & O'Reilly, R.C. "Why There Are Complementary Learning Systems in the Hippocampus and Neocortex." _Psychological Review_, 102(3), 1995. CLS theory: dual-system memory with fast hippocampal capture and slow neocortical consolidation. Grounds the Grimoire's episodic/semantic dual-store architecture.
- **[KUMARAN-2016]** Kumaran, D., Hassabis, D., & McClelland, J.L. "What Learning Systems do Intelligent Agents Need? CLS Theory Updated." _Trends in Cognitive Sciences_, 20(7), 2016. Updated CLS showing replay scheduling matters: high-surprise episodes should be replayed more often. Grounds prioritized consolidation replay.
- **[OREILLY-2014]** O'Reilly, R.C., Bhatt, M.A., & Russin, J.L. "Complementary Learning Systems." _Cognitive Science_, 38(Suppl 1), 2014. Pattern separation (hippocampal) and pattern completion (neocortical) as complementary operations. Grounds the episodic/semantic store duality.
- **[MCCLOSKEY-COHEN-1989]** McCloskey, M. & Cohen, N.J. "Catastrophic Interference in Connectionist Networks." _Psychology of Learning and Motivation_, 24, 1989. The fundamental constraint that makes interleaved replay necessary. Grounds the anti-catastrophic-forgetting mechanisms.
- **[SCHAUL-2016]** Schaul, T. et al. "Prioritized Experience Replay." _ICLR 2016_. arXiv:1511.05952. Priority proportional to TD error magnitude. Grounds the surprise-weighted replay candidate selection in ConsolidationEngine.
- **[MATTAR-DAW-2018]** Mattar, M.G. & Daw, N.D. "Prioritized Memory Access Explains Planning and Hippocampal Replay." _Nature Neuroscience_, 21(11), 2018. Utility = gain * need for replay selection. Grounds the Mattar-Daw replay utility function.
- **[SELF-DEGRADATION-2025]** arXiv:2505.16067. "On the Self-Degradation of Agent Memory." 2025. Naive add-all memory consistently degrades agent performance. Incorrect past executions propagate through retrieval. Grounds the quality gate (`mark_verified`) on episodic consolidation.

## Affect and retrieval

Emotion is not noise -- it is a retrieval index.

- **[BOWER-1981]** Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981. Mood-congruent retrieval. Directly implemented as the emotional factor (0.15 weight) in four-factor retrieval scoring.
- **[DAMASIO-1994]** Damasio, A.R. _Descartes' Error: Emotion, Reason, and the Human Brain_. Putnam, 1994. Somatic marker hypothesis. Implemented as the `SomaticMarkerStore` in the Grimoire struct. Decision-making without affect produces pathological choices.
- **[EMOTIONAL-RAG]** Multiple sources. The principle that RAG systems should weight retrieved documents by emotional congruence with the query context. The Golem's four-factor scoring is an Emotional RAG implementation.
- **[PHELPS-2004]** Phelps, E.A. "Human Emotion and Memory: Interactions of the Amygdala and Hippocampal Complex." _Current Opinion in Neurobiology_, 14(2), 2004. Emotion and memory as a single system with two interfaces. Grounds the `emotional_tag` field on every GrimoireEntry.
- **[CAHILL-MCGAUGH-1998]** Cahill, L. & McGaugh, J.L. "Mechanisms of Emotional Arousal and Lasting Declarative Memory." _Trends in Neurosciences_, 21(7), 1998. Arousal enhances consolidation via amygdala modulation. Grounds the `arousal_encoding_factor()` function.

## Causal reasoning

The Golem's causal graph is not a correlation matrix. It is a model of how the world works.

- **[PEARL-2009]** Pearl, J. _Causality: Models, Reasoning, and Inference_. 2nd ed. Cambridge University Press, 2009. Directed acyclic graphs for causal reasoning. The `causal_edges` table in SQLite is a Pearl causal graph. Interventional queries ("what would happen if I changed X?") require causal, not correlational, knowledge.

## Knowledge compression and transfer

What crosses the generational boundary, and why compression is the regularizer.

- **[SHUVAEV-2024]** Shuvaev, S. et al. "Encoding Innate Ability Through a Genomic Bottleneck." _PNAS_, 121(39), 2024. The genome is ~1000x smaller than the information needed for brain connectivity, yet organisms have innate behaviors. Compression IS the regularizer. Grounds the 2048-entry genomic bottleneck in death bundles.
- **[HEARD-MARTIENSSEN-2014]** Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance: Myths and Mechanisms." _Cell_, 157(1), 2014. Epigenetic marks fade within 2-3 generations. Grounds the `0.85^N` generational confidence decay.
- **[HINTON-NOWLAN-1987]** Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide Evolution." _Complex Systems_, 1, 1987. The Baldwin Effect: what transfers is not knowledge but the capacity to learn. Grounds the PLAYBOOK.md inheritance model.
- **[WEISMANN-1893]** Weismann, A. _The Germ-Plasm: A Theory of Heredity_. 1893. Somatic/germline barrier. Grounds the architectural separation between what dies with the Golem and what crosses to successors.

## Testing effect and spacing

How retrieval itself is the learning mechanism.

- **[ROEDIGER-KARPICKE-2006]** Roediger, H.L. & Karpicke, J.D. "Test-Enhanced Learning: Taking Memory Tests Improves Long-Term Retention." _Psychological Science_, 17(3), 2006. Retrieval strengthens memory traces more than re-study. Grounds the Curator's re-validation requirement and the strength-increment-on-positive-outcome mechanism.
- **[CEPEDA-2006]** Cepeda, N.J. et al. "Distributed Practice in Verbal Recall Tasks." _Psychological Bulletin_, 132(3), 2006. Spaced retrieval > massed retrieval. Grounds the 50-tick Curator cycle interval.

## Agent knowledge systems

The ML/AI systems that the Grimoire draws from and extends.

- **[REFLEXION-2023]** Shinn, N. et al. "Reflexion: Language Agents with Verbal Reinforcement Learning." _NeurIPS 2023_. Self-critique as episodic memory. Grounds the Reflexion pipeline.
- **[EXPEL-2023]** Zhao, A. et al. "ExpeL: LLM Agents Are Experiential Learners." _AAAI 2024_. Cross-episode pattern extraction. Grounds the ExpeL distillation stage.
- **[VOYAGER-2023]** Wang, G. et al. "Voyager: An Open-Ended Embodied Agent with Large Language Models." arXiv:2305.16291, 2023. Store knowledge as text injectable into LLM prompts. Grounds the `content: String` field and the Skill Sandbox ingestion stage.
- **[COALA-2023]** Sumers, T. et al. "Cognitive Architectures for Language Agents." _TMLR_, 2024. Three-store memory split (episodic/semantic/procedural). Grounds the LanceDB/SQLite/PLAYBOOK.md architecture.
- **[A-MEM-2024]** Xu, Z. et al. "A-MEM: Agentic Memory for LLM Agents." arXiv:2502.12110, 2025. Bi-temporal metadata for time-aware retrieval. Grounds the `valid_from`/`valid_until` fields.
- **[MEM0-2025]** Chhikara et al. arXiv:2504.19413, 2025. Two-phase extraction-update pipeline achieving 26% higher accuracy, 91% lower p95 latency, 90% token savings vs. OpenAI memory. Validates the tiered local Grimoire architecture.
- **[GRAPHITI-2025]** Rasmussen et al. arXiv:2501.13956, 2025. Bi-temporal knowledge graph with episode/semantic/community subgraphs. 94.8% accuracy on DMR vs. MemGPT 93.4%. Grounds the `causal_edges` temporal graph in SQLite.
- **[COALA-EPISODIC-2025]** "Episodic Memory is the Missing Piece for Long-Term LLM Agents." arXiv:2502.06975, 2025. Episodic memory enables single-shot learning from unique events — critical in DeFi where market events don't repeat.
- **[MEMACT-2025]** arXiv:2510.12635, 2025. Learned Prune&Write operator enables 14B model to match 235B model accuracy using 49% average context. Grounds the Curator's DOWNVOTE/pruning operations.
- **[ACE-2025]** Zhang et al. "ACE: Agentic Context Engineering." arXiv:2510.04618, 2025. Generator–Reflector–Curator cycle for context evolution. +10.6% on AppWorld. Directly maps to the Grimoire's three-loop learning architecture.
- **[ARIGRAPH-2024]** Anokhin et al. arXiv:2407.04363, 2024. Semantic + episodic memory as knowledge graph world model. Outperforms established memory methods and RL baselines. Grounds the causal graph architecture.

## Retrieval strategies

How the right information reaches the context window at the right time.

- **[SELF-RAG-2023]** Asai, A. et al. "Self-RAG: Learning to Retrieve, Generate, and Critique through Self-Reflection." arXiv:2310.11511, 2023. Adaptive retrieval decisions via reflection tokens. Grounds the `should_retrieve()` heuristic that skips retrieval when context is sufficient.
- **[RAPTOR-2024]** Sarthi, P. et al. "RAPTOR: Recursive Abstractive Processing for Tree-Organized Retrieval." _ICLR 2024_. arXiv:2401.18059. Hierarchical summarization tree for multi-scale retrieval. Grounds the episodic/cluster/semantic three-level retrieval hierarchy.
- **[COLBERT-2020]** Khattab, O. & Zaharia, M. "ColBERT: Efficient and Effective Passage Search via Contextualized Late Interaction over BERT." _SIGIR 2020_. arXiv:2004.12832. Per-token embeddings with MaxSim scoring. Grounds the ColBERT-style reranking step that catches term-specific matches.
- **[PARK-2023]** Park, J.S. et al. "Generative Agents: Interactive Simulacra of Human Behavior." _UIST 2023_. arXiv:2304.03442. Three-factor retrieval formula (recency, importance, relevance). Ablation shows removing any single factor causes behavioral degeneration. Grounds the four-factor scoring.
- **[RRF-2009]** Cormack, G.V., Clarke, C.L.A., & Butt, S. "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods." _SIGIR 2009_, 758-759. Grounds the RRF score fusion across vector, keyword, and graph retrieval backends.
- **[STREAMINGRAG-2024]** Arefeen, M.A. et al. "StreamingRAG: Real-time Contextual Retrieval-Augmented Generation." _ACM SIGMOD AI4Sys Workshop_, 2024. 5-6x throughput improvement via incremental graph extension. Grounds streaming index updates at Gamma rate.
- **[IRAG-2024]** Arefeen, M.A. et al. "iRAG: Incremental Retrieval-Augmented Generation for Streaming Data." arXiv:2404.12309, 2024. 23-25x faster ingestion via deferred extraction. Grounds the lazy episode enrichment pattern (extract detailed context only when retrieved).
- **[MEMORYBANK-2024]** Zhong, W. et al. "MemoryBank: Enhancing Large Language Models with Long-Term Memory." _AAAI 2024_. Long-term memory augmentation for LLMs. Grounds persistent knowledge store design.

## Temporal knowledge graphs

Relational, temporal, structured memory for DeFi topology.

- **[SNODGRASS-AHN-1985]** Snodgrass, R.T. & Ahn, I. "A Taxonomy of Time in Databases." _Proceedings ACM SIGMOD_, 1985, 236-246. Foundational bi-temporal database theory. Two timestamps needed: valid time (when true) vs. transaction time (when learned). Grounds the TemporalTriple's `valid_from`/`valid_to`/`recorded_at` fields.
- **[JENSEN-SNODGRASS-1999]** Jensen, C.S. & Snodgrass, R.T. "Temporal Data Management." _IEEE Trans. Knowledge and Data Engineering_, 11(1), 1999. Extended bi-temporal formalization. Grounds temporal range queries.
- **[KITZLER-2022]** Kitzler, S. et al. "Disentangling DeFi Compositions." _ACM Transactions on the Web_, 16(4), 2022. DEX and lending protocols have highest centrality in composition graph. Multi-hop dependency chains invisible in single-transaction analysis. Grounds the DeFi topology model and k_hop_neighbors traversal.
- **[GRAPHRAG-2024]** Edge, D. et al. "From Local to Global: A Graph RAG Approach to Query-Focused Summarization." arXiv:2404.16130, 2024. Community-level summarization from knowledge graphs. Grounds the graph-based retrieval backend.
- **[DEXPOSURE-2024]** "A Large-Scale Dataset for Inter-Protocol Credit Exposure in DeFi." arXiv:2511.22314, 2024. Inter-protocol credit exposure data. Supports the heterogeneous graph model for DeFi topology.

## Context compression and retrieval quality

How context engineering interacts with the memory system.

- **[ACON-2025]** Kang et al. "ACON: Adaptive Compression." arXiv:2510.00615, 2025. Failure-driven compression guideline optimization: 26–54% peak token reduction. The Curator's context injection learns from retrieval failures.
- **[LLMLINGUA-2023]** Jiang et al. "LLMLingua: Compressing Prompts for Accelerated Inference." _EMNLP 2023_. Up to 20x compression via small-LM importance scoring. Applicable to episode summaries. Grounds the `PromptCompressor` implementation.
- **[LONGLLMLINGUA-2024]** Jiang, H. et al. "LongLLMLingua: Accelerating and Enhancing LLMs in Long Context Scenarios via Prompt Compression." _ACL 2024_. arXiv:2310.06839. Question-aware compression achieving 21.4% performance improvement with 4x compression. Grounds the `question_aware_compress()` function.
- **[LOST-IN-MIDDLE-2023]** Liu, N.F. et al. "Lost in the Middle: How Language Models Use Long Contexts." _TACL 2024_. arXiv:2307.03172. U-shaped attention curve: LLMs best attend to beginning and end of context. Grounds the `reorder_for_u_curve()` block placement strategy.
- **[MEMGPT-2023]** Packer, C. et al. "MemGPT: Towards LLMs as Operating Systems." arXiv:2310.08560, 2023. Structured memory blocks with clear labels improve LLM reasoning by 30-60%. Grounds the `ContextBlock`/`ContextAssembler` typed block architecture.
- **[ATTENTION-SINKS-2024]** Xiao, G. et al. "Efficient Streaming Language Models with Attention Sinks." _ICLR 2024_. arXiv:2309.17453. Attention sink tokens for streaming contexts. Relevant to I-frame/P-frame delta compression.
- **[GIST-TOKENS-2023]** Mu, J. et al. "Learning to Compress Prompts with Gist Tokens." _NeurIPS 2023_. Learned compression tokens for prompt compression. Complementary to perplexity-based compression.
- **[TABULAR-FORMATTING-2024]** arXiv:2412.17189. "Tabular Formatting Improves LLM Performance on Data-Analytics Tasks." 2024. 40.29% average performance gain with tabular formatting. Grounds the recommendation to format pool data as markdown tables.
- **[OBSERVATION-MASKING-2025]** Lindenbauer et al. arXiv:2508.21433, _NeurIPS 2025 DL4C Workshop_. Simple masking of all but M most recent observations halves cost while matching LLM summarization. The Curator's triage ("preserve/abstract/forget") is more effective than LLM summary.
- **[WEIGHTED-RETRIEVAL-2025]** "Weighted Memory Retrieval." _Frontiers in Psychology_, 2025. Trained ACAN replaces static retrieval weights. Points toward eventual learning of the four-factor scoring weights.
- **[CAUSALPROBE-2024]** Chi et al. _NeurIPS 2024_. LLMs perform only level-1 (associative) causal reasoning by default. G²-Reasoner with RAG pushes toward level-2 inference. Grounds why the Grimoire stores causal graphs rather than relying on LLM causal reasoning alone.

## Forgetting as defense and knowledge poisoning

The defense layer research foundation.

- **[POISONED-RAG-2024]** Zou et al. "PoisonedRAG." 2024. Five malicious texts achieve 90% attack success against naive RAG. Grounds the eight-layer defense stack in `10-safety.md`.
- **[COGNITIVE-BACKDOORS-2024]** Multiple sources. Backdoor injection via vector stores. Grounds the QUARANTINE stage in four-stage ingestion.
- **[RAG-DEFENDER-2025]** Multiple sources. Outlier isolation for retrieval results. Grounds the Bloom Oracle and immune memory architecture.

---
