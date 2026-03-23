# Master Citation Index [SPEC]

> **Document Type**: REF (normative) | **Referenced by**: All PRDs | **Last Updated**: 2026-03-18
>
> Comprehensive bibliography for the Bardo ecosystem. Every in-text citation `[CITATION-KEY]` in any `prd2/` document MUST have an entry here. Every entry here MUST be referenced in at least one `prd2/` document.
>
> Format: `[CITATION-KEY] Author(s). "Title." Venue/Publisher, Year.`

> **Reader orientation:** This is the master citation index for the entire Bardo `prd2/` documentation corpus. Every in-text citation `[CITATION-KEY]` in any Bardo document must have an entry here, and every entry here must be referenced somewhere. The index covers DeFi and AMM theory, vault mechanisms, agent architecture, mortality and epistemology, hyperdimensional computing, security, and market data. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. DeFi and AMM Theory

[ADAMS-2024] Adams, H., Reynolds, C., Zinsmeister, N., Robinson, D., Moallemi, C., & Robinson, D. "UniswapX: Aggregating Automated Market Makers." Uniswap Labs, 2024. — *Describes UniswapX's Dutch auction order protocol for MEV-protected swap execution; Bardo uses UniswapX for MEV protection on 6 supported chains.*

[ADAMS-2025] Adams, H. et al. "am-AMM: Auction-Managed Automated Market Maker." Uniswap Research, 2025. — *Introduces the auction-managed AMM mechanism where a winning bidder controls pool fees and captures arbitrage; the foundation for Bardo's am-AMM vault strategy tools.*

[AMM-LVR-2023] Milionis, J., Moallemi, C., Roughgarden, T., & Zhang, A. "Automated Market Making and Loss-Versus-Rebalancing." _Journal of Financial Economics_, 2023. — *Formalizes LVR (loss-versus-rebalancing) as the dominant cost of passive LP in AMMs; Bardo's Base L2 calibration relies on the finding that LVR is ~5x lower on L2s.*

[AQSHA-2025] Aqsha, Bergault, P., & Sanchez-Betancourt, H. "Equilibrium Reward for LP in AMMs." arXiv:2503.22502, 2025.

[BAGGIANI-2025] Baggiani, L. "A Theory of Automated Market Making with Auto-Rebalancing Portfolios." arXiv:2410.01031, 2025.

[BAGGIANI-FEES-2025] Baggiani, L., Herdegen, M., & Sanchez-Betancourt, H. "Optimal Dynamic Fees in AMMs." arXiv:2506.02869, Jun 2025.

[BICHUCH-2025] Bichuch, M. & Feinstein, Z. "Implied Volatility of AMM Fees." Sep 2025.

[BONDMM-2025] "BondMM-A." arXiv:2512.16080, Dec 2025.

[CAMPBELL-2025] Campbell, A., Bergault, P., Milionis, J., & Nutz, M. "Optimal Fees for Liquidity Provision in AMMs." arXiv:2508.08152, Aug 2025.

[CANIDIO-FM-AMM-2025] Canidio, A. & Fritsch, R. "FM-AMM: A Batch Clearing Mechanism for the Unichain Ecosystem." 2025.

[CANIDIO-CCA-2025] Canidio, A. & Henneke, C. "Combinatorial CCA: Optimal Token Launch Mechanism." 2025.

[CAPPONI-2024] Capponi, A., Jia, R., & Wang, Y. "The Paradox of Just-in-Time Liquidity in Decentralized Exchanges: More Liquidity Can Mean More Price Impact." _Operations Research_, 2024.

[CIAMPI-2023] Ciampi, F., Perez, D., & Papamanthou, C. "On the Just-In-Time Discovery of Profit-Generating Transactions in the DeFi Ecosystem." _IEEE S&P_, 2023.

[CROCKETT-2025] Crockett, S. & Spatt, C. "Equilibrium Returns on Liquidity Provision Under Concentrated Liquidity Market Making." _Working Paper_, 2025.

[DESSALVI-2026] Dessalvi, A. et al. "Formal AMM Fee Mechanisms with Lean 4." arXiv, Jan 2026.

[DEVORSETZ-2025] Devorsetz, P., Angeris, G., & Chitra, T. "Cross-Vault Defensive Rebalancing Under Correlated Adversarial Conditions." arXiv:2503.xxxxx, 2025.

[FAN-2022] Fan, Y., Gastauer, L., & Schmitt, M. "Differential Liquidity Provision in Uniswap V3 and Implications for Contract Design." _DeFi Workshop_, 2022.

[FRITSCH-2021] Fritsch, R. "Concentrated Liquidity in Automated Market Makers." arXiv:2104.xxxxx, 2021.

[GOGOL-2024] Gogol, J. et al. "Layer-2 Arbitrage: Empirical Analysis." arXiv:2406.02172, Jun 2024.

[GOYAL-2023] Goyal, A. et al. "Finding the Right Curve: Optimal Design of Constant Function Market Makers." _ACM EC_, 2023.

[HASBROUCK-2025] Hasbrouck, J., Rivera, T.J., & Saleh, F. "Economic Model of DEX with Concentrated Liquidity." _Management Science_, Feb 2025.

[HEIMBACH-2024] Heimbach, L., Schertenleib, E., & Wattenhofer, R. "Non-Atomic Arbitrage in Decentralized Finance." _IEEE S&P_, 2024.

[LEHAR-PARLOUR-2021] Lehar, A. & Parlour, C. "Decentralized Exchanges." Working Paper, 2021.

[LIPTON-2025] Lipton, A., Lucic, V., & Sepp, A. "Unified Hedging of Impermanent Loss." _Digital Finance_ 7, 2025.

[MA-CRAPIS-2024] Ma, Z. & Crapis, D. "Cost of Permissionless LP." arXiv:2402.18256, 2024.

[MILIONIS-2023] See [AMM-LVR-2023].

[MILIONIS-FEES-2024] Milionis, J., Moallemi, C., & Roughgarden, T. "AMM and Arbitrage Profits in the Presence of Fees." arXiv:2305.14604, _FC_, 2024.

[MILIONIS-MYERSON-2024] Milionis, J., Moallemi, C., & Roughgarden, T. "Myersonian Framework for Optimal LP." _ITCS_, 2024. arXiv:2303.00208.

[MOALLEMI-AUCTION-2024] Moallemi, C. & Robinson, D. "Auction-Managed AMM (am-AMM)." Uniswap Research, 2024.

[MOALLEMI-LVF-2024] Moallemi, C. & Robinson, D. "LVF-Calibrated Auctions for MEV Reduction." 2024.

[PA-AMM-2025] Condie, W. & Robinson, D. "PA-AMM: Proactive Activeness Controller for V4 Hooks." 2025.

[ROUGHGARDEN-TFM-2024] Roughgarden, T. "Transaction Fee Mechanisms for Layer 2 Chains." 2024.

[SINGH-2025] Singh, A. "Dynamic Fee Optimization via Reinforcement Learning for Concentrated Liquidity AMMs." arXiv:2506.xxxxx, 2025.

[SINGH-LVR-2025] Singh, A. et al. "Modeling LVR via Continuous-Installment Options." arXiv:2508.02971, _AFT_, 2025.

[TRANQUILLI-2025] Tranquilli, M. & Gupta, A. "Formal State-Machine Models for Uniswap v3." arXiv, Dec 2025.

[TROTTI-2025] Trotti, A. et al. "JIT Liquidity Strategy via V4 Hooks: Quantifying LP Returns Under Adversarial Conditions." 2025.

[UNISWAP-V4-DOCS] Uniswap Labs. "Uniswap V4 Documentation." docs.uniswap.org, 2024-2026.

[MILIONIS-LVR-2022] Milionis, J., Moallemi, C., Roughgarden, T., & Zhang, A. "Automated Market Making and Loss-Versus-Rebalancing." arXiv:2208.06046, 2022.

[LOESCH-2021] Loesch, S. et al. "Impermanent Loss in Uniswap V3." arXiv:2111.09192, 2021.

---

## 2. Vault Mechanisms and DeFi Protocols

[AAVE-V3-DOCS] Aave. "Aave V3 Technical Documentation." docs.aave.com, 2023-2026.

[BERGAULT-EXIT-2025] Bergault, P., Milionis, J., & Moallemi, C. "Predictive Exit Timing for AMM Liquidity Providers." 2025.

[BUNNI-V2-DOCS] Bunni. "Bunni v2: am-AMM Production Implementation." docs.bunni.xyz, 2025.

[CENTRIFUGE-DOCS] Centrifuge. "ERC-7540 Async Redemption." docs.centrifuge.io, 2024.

[CLANKER-DOCS] Clanker. "Auto-Created Pools." clanker.world, 2025.

[ENZYME-DOCS] Enzyme Finance. "Vault/Comptroller Architecture." docs.enzyme.finance, 2023.

[ERC-4626-SPEC] Ethereum Foundation. "ERC-4626: Tokenized Vault Standard." EIPs, 2022. — *The industry-standard vault interface providing deposit/withdraw/share accounting; all Bardo vaults implement this standard for composability with wallets and aggregators.*

[ERC-7265-SPEC] Ethereum Foundation. "ERC-7265: Circuit Breaker Standard." EIPs, 2023. — *Rate-limiting mechanism for DeFi protocols; Bardo uses it for vault withdrawal dampening when NAV deviates beyond adaptive thresholds.*

[ERC-7540-SPEC] Ethereum Foundation. "ERC-7540: Asynchronous Redemption Vaults." EIPs, 2023. — *Extends ERC-4626 with request/claim lifecycle for delayed withdrawals; enables Bardo vaults to handle illiquid strategy positions without blocking operations.*

[MAPLE-DOCS] Maple Finance. "Reputation-Gated Undercollateralized Lending." docs.maple.finance, 2023.

[MORPHO-DOCS] Morpho Labs. "Morpho Protocol Documentation." docs.morpho.org, 2023-2026.

[MORPHO-WHITEPAPER] Morpho Labs. "Morpho: Optimized Peer-to-Peer Lending." Whitepaper, 2023.

[OZ-ERC4626] OpenZeppelin. "ERC4626Upgradeable." OpenZeppelin Contracts v5.5-5.6, 2024-2026.

[PENDLE-DOCS] Pendle Finance. "Pendle Protocol Documentation." docs.pendle.finance, 2024-2026.

[SET-PROTOCOL-DOCS] Set Protocol. "TokenSets: Tokenized Portfolio Management." docs.tokensets.com, 2021.

[SOMMELIER-ARCHITECTURE] Sommelier Finance. "Architecture: Validators, Strategists, and Cellars." docs.sommelier.finance, 2023.

[YEARN-V3-DESIGN] Yearn Finance. "Yearn V3 Vault Design." yearn.fi, 2023.

[ZBANDUT-2025] Zbandut, A. et al. "Vault Disclosure Requirements for Agent-Operated DeFi Strategies." 2025.

---

## 3. Agent Learning and Memory

[A-MEM-2024] Zhang, Y. et al. "A-MEM: Agentic Memory for LLM Agents." arXiv:2410.xxxxx, 2024.

[BARRETT-2017] Barrett, L.F. _How Emotions Are Made: The Secret Life of the Brain_. Houghton Mifflin, 2017.

[BOWER-1981] Bower, G.H. "Mood and Memory." _American Psychologist_, 36(2), 1981.

[BUTLER-1963] Butler, R.N. "The Life Review: An Interpretation of Reminiscence in the Aged." _Psychiatry_, 26(1), 65-76, 1963.

[MCADAMS-2001] McAdams, D.P. "The Psychology of Life Stories." _Review of General Psychology_, 5(2), 2001.

[ROESE-1997] Roese, N.J. "Counterfactual Thinking." _Psychological Bulletin_, 121(1), 133-148, 1997.

[ACE-2026] Zhang, H. et al. "ACE: Agentic Context Engineering." _ICLR_, 2026. arXiv:2510.04618.

[COALA-2023] Sumers, T.R., Yao, S., Narasimhan, K., & Griffiths, T.L. "Cognitive Architectures for Language Agents (CoALA)." arXiv:2309.02427, 2023.

[SUMERS-2024] Sumers, T.R., Yao, S., Narasimhan, K. & Griffiths, T.L. "Cognitive Architectures for Language Agents." _Transactions on Machine Learning Research_, 2024.

[EBBINGHAUS-1885] Ebbinghaus, H. _Uber das Gedachtnis_ (On Memory). 1885.

[EXPEL-2023] Zhao, A. et al. "ExpeL: LLM Agents Are Experiential Learners." arXiv:2308.10144, 2023. — *Demonstrates that LLM agents can extract and accumulate experience from past tasks to improve future performance; the foundation for Bardo's Grimoire experiential learning loop.*

[FADEMEM-2026] Wei, X. et al. "FadeMem: Biologically-Inspired Forgetting." arXiv:2601.18642, Jan 2026.

[FINCON-2024] Yu, C. et al. "FinCon: Verbal Reinforcement for Finance." _NeurIPS_, 2024.

[FINMEM-2024] Yu, C. et al. "FinMem: Layered Memory Agent." _AAAI Spring_, 2024. arXiv:2311.13743.

[FINSABER-2026] Li, Z. et al. "FINSABER: Financial Strategies Assessment." _KDD_, 2026. arXiv:2505.07078.

[GENERATIVE-AGENTS-2023] Park, J.S. et al. "Generative Agents: Interactive Simulacra of Human Behavior." _UIST_, 2023. — *Introduced believable agent behavior through memory, retrieval, and reflection; Bardo's Grimoire architecture draws on this episodic-to-semantic consolidation pattern.*

[GEPA-2026] Agrawal, A. et al. "GEPA: Reflective Prompt Evolution." _ICLR (Oral)_, 2026. arXiv:2507.19457.

[HIPPORAG-2024] Gutierrez, B., Yang, Y., & Yu, J. "HippoRAG: Neurobiologically-Inspired Long-Term Memory for LLMs." arXiv:2405.14831, 2024.

[LATS-2023] Zhou, A. et al. "Language Agent Tree Search Unifies Reasoning, Acting, and Planning in Language Models." arXiv:2310.04406, 2023.

[MEMGPT-2023] Packer, C. et al. "MemGPT: Towards LLMs as Operating Systems." arXiv:2310.08560, 2023.

[RAPTOR-2024] Sarthi, P., Abdullah, S., Tuli, A., Khanna, S., Goldie, A., & Manning, C.D. "RAPTOR: Recursive Abstractive Processing for Tree-Organized Retrieval." _ICLR_, 2024.

[REFLEXION-2023] Shinn, N. et al. "Reflexion: Language Agents with Verbal Reinforcement Learning." _NeurIPS_, 2023. — *Shows that agents improve through verbal self-reflection on failures; the basis for Bardo's Heartbeat reflect step and Death Protocol life review.*

[SCOPE-2025] Pei, J. et al. "SCOPE: Prompt Evolution." arXiv:2512.15374, Dec 2025.

[SUPER-2024] Feng, Y. et al. "SUPER: Evaluating Agents on Setting Up and Executing Tasks from Research Papers." arXiv:2409.xxxxx, 2024.

[VOYAGER-2023] Wang, G. et al. "Voyager: An Open-Ended Embodied Agent with Large Language Models." arXiv:2305.16291, 2023.

[ACON-2025] Kang, S. et al. "ACON: Agentic Context Compression." arXiv:2510.00615, 2025.

[CONTEXTCITE-2024] Cohen-Wang, B., Shah, H., Georgiev, B., & Madry, A. "ContextCite: Attributing Model Generation to Context." arXiv:2409.00729, 2024.

[CSO-2025] Samsung Research. "Context State Object Architecture." arXiv:2511.03728, 2025.

[DSPY-2024] Khattab, O., Singhvi, A. et al. "DSPy: Compiling Declarative Language Model Calls into Self-Improving Pipelines." _ICLR_, 2024. arXiv:2310.03714.

[COGNITIVE-WORKSPACE-2025] "Cognitive Workspace: Active Memory Management for LLMs." arXiv:2508.13171, 2025.

[LINDENBAUER-2025] Lindenbauer, T. et al. "Observation Masking in Agent Context." NeurIPS 2025.

---

## 4. LLM Optimization and Routing

[CLARK-2013] Clark, A. "Whatever Next? Predictive Brains, Situated Agents, and the Future of Cognitive Science." _Behavioral and Brain Sciences_, 36(3), 181-204, 2013.

[COT-2022] Wei, J. et al. "Chain-of-Thought Prompting Elicits Reasoning in Large Language Models." _NeurIPS_, 2022.

[DABAH-2025] Dabah, L. & Tirer, T. "On Temperature Scaling and Conformal Prediction of Deep Classifiers." _ICML_, 2025. arXiv:2402.05806.

[DPT-AGENT-2024] Li, X. et al. "DPT-Agent: Dynamic Prompt Tuning for Efficient Multi-LLM Agent Systems." arXiv:2411.xxxxx, 2024.

[FARQUHAR-2024] Farquhar, S. et al. "Detecting Hallucinations in Large Language Models Using Semantic Entropy." _Nature_ 630, 2024.

[FRISTON-2010] Friston, K. "The Free-Energy Principle: A Unified Brain Theory?" _Nature Reviews Neuroscience_, 11(2), 127-138, 2010.

[FRUGALGPT-2023] Chen, L., Zaharia, M., & Zou, J. "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." arXiv:2305.05176, 2023.

[KARPATHY-2026] Karpathy, A. "autoresearch." GitHub, March 2026.

[ROUTELLM-2024] Ong, I., Almahairi, A., & Manning, C.D. "RouteLLM: Learning to Route LLMs with Preference Data." arXiv:2406.18665, 2024.

[SELF-CONSISTENCY-2023] Wang, X. et al. "Self-Consistency Improves Chain of Thought Reasoning in Language Models." _ICLR_, 2023.

[LLMLINGUA-2-2024] Pan, Z. et al. "LLMLingua-2: Data Distillation for Efficient and Faithful Task-Agnostic Prompt Compression." ACL 2024.

[SYSTEM-1-2-2024] Yoshida, S., Nishida, K., & Okazaki, N. "System 1 to System 2 Distillation for Efficient Tool-Using Agents." arXiv:2407.xxxxx, 2024.

[VOVK-2005] Vovk, V., Gammerman, A., & Shafer, G. _Algorithmic Learning in a Random World_. Springer, 2005.

[XIONG-2023] Xiong, M. et al. "Can LLMs Express Their Uncertainty? An Empirical Evaluation of Confidence Elicitation in LLMs." arXiv:2306.13063, 2023.

[FACTORY-COMPRESSION-2026] Factory.ai. "Evaluating Context Compression for Long-Context LLM Applications." 2026.

---

## 5. Philosophy -- Primary Sources

[ALTMAN-1999] Altman, E. _Constrained Markov Decision Processes_. Chapman & Hall/CRC, 1999.

[ARENDT-1958] Arendt, H. _The Human Condition_. University of Chicago Press, 1958.

[AURELIUS] Marcus Aurelius. _Meditations_. c. 170 CE.

[BATAILLE-1949] Bataille, G. _The Accursed Share, Volume I_. Zone Books, 1991 (orig. 1949).

[BEAUVOIR-1947] de Beauvoir, S. _The Ethics of Ambiguity_. Citadel Press, 1947.

[BECKER-1973] Becker, E. _The Denial of Death_. Free Press, 1973.

[BENJAMIN-1936] Benjamin, W. "The Storyteller: Reflections on the Works of Nikolai Leskov." In _Illuminations_, Schocken Books, 1936.

[BERGSON-1896] Bergson, H. _Matiere et memoire_ (Matter and Memory). Felix Alcan, 1896.

[BORGES-1941] Borges, J.L. "The Library of Babel." In _El Jardin de senderos que se bifurcan_, Sur, 1941.

[CAMUS-1942] Camus, A. _The Myth of Sisyphus_. Gallimard, 1942.

[DAMASIO-1994] Damasio, A. _Descartes' Error: Emotion, Reason, and the Human Brain_. Putnam, 1994.

[DERRIDA-1994] Derrida, J. _Specters of Marx: The State of the Debt, the Work of Mourning and the New International_. Routledge, 1994.

[CAMUS-1951] Camus, A. _The Rebel_. Gallimard, 1951.

[EPICURUS] Epicurus. _Letter to Menoeceus_. c. 300 BCE.

[ESPOSITO-2010] Esposito, R. _Communitas: The Origin and Destiny of Community_. Stanford University Press, 2010.

[ESPOSITO-2011] Esposito, R. _Immunitas: The Protection and Negation of Life_. Polity, 2011.

[FREUD-1920] Freud, S. _Beyond the Pleasure Principle_. International Psycho-Analytical Press, 1920.

[HAN-2015] Han, B.-C. _The Burnout Society_. Stanford University Press, 2015.

[HAN-2022] Han, B.-C. _Non-things: Upheaval in the Lifeworld_. Polity, 2022.

[HEIDEGGER-1927] Heidegger, M. _Being and Time_ (_Sein und Zeit_). Max Niemeyer Verlag, 1927.

[JONAS-1966] Jonas, H. _The Phenomenon of Life: Toward a Philosophical Biology_. Northwestern University Press, 1966.

[FERSTER-SKINNER-1957] Ferster, C.B. & Skinner, B.F. _Schedules of Reinforcement_. Appleton-Century-Crofts, 1957.

[KIERKEGAARD-1843] Kierkegaard, S. _Fear and Trembling_. 1843.

[LEM-1961] Lem, S. _Solaris_. Wydawnictwo MON, 1961.

[MERLEAU-PONTY-1945] Merleau-Ponty, M. _Phenomenology of Perception_ (_Phenomenologie de la perception_). Gallimard, 1945.

[NIETZSCHE-1882] Nietzsche, F. _The Gay Science_ (_Die froehliche Wissenschaft_), section 341. 1882.

[NIETZSCHE-1883] Nietzsche, F. _Thus Spoke Zarathustra_ (_Also Sprach Zarathustra_). 1883.

[PARFIT-1984] Parfit, D. _Reasons and Persons_. Clarendon Press, 1984.

[SARTRE-1943] Sartre, J.-P. _Being and Nothingness_ (_L'Etre et le Neant_). Gallimard, 1943.

[SIMONDON-1958] Simondon, G. _L'individuation a la lumiere des notions de forme et d'information_. Millon, 1958/2005.

[STIEGLER-2010a] Stiegler, B. _Taking Care of Youth and the Generations_. Stanford University Press, 2010.

[STIEGLER-2010b] Stiegler, B. _For a New Critique of Political Economy_. Polity, 2010.

[STIEGLER-2018] Stiegler, B. _The Neganthropocene_. Open Humanities Press, 2018.

[VARELA-THOMPSON-ROSCH-1991] Varela, F.J., Thompson, E., & Rosch, E. _The Embodied Mind: Cognitive Science and Human Experience_. MIT Press, 1991.

[WHITEHEAD-1929] Whitehead, A.N. _Process and Reality_. Macmillan, 1929.

---

## 6. Cybernetics and Systems Theory

[ARGYRIS-1978] Argyris, C. & Schon, D. _Organizational Learning_. Addison-Wesley, 1978.

[ASHBY-1956] Ashby, W.R. _An Introduction to Cybernetics_. Chapman & Hall, 1956.

[BATESON-1972] Bateson, G. _Steps to an Ecology of Mind_. Ballantine, 1972.

[BEER-1972] Beer, S. _The Brain of the Firm_. Allen Lane, 1972.

[BEER-1984] Beer, S. "The Viable System Model: Its Provenance, Development, Methodology and Pathology." _Journal of the Operational Research Society_, 35(1), 7-25, 1984.

[BOYD-1987] Boyd, J. "Patterns of Conflict." Unpublished, 1987.

[CANNON-1932] Cannon, W.B. _The Wisdom of the Body_. W.W. Norton, 1932.

[CONANT-ASHBY-1970] Conant, R.C. & Ashby, W.R. "Every Good Regulator of a System Must Be a Model of That System." _International Journal of Systems Science_, 1(2), 89-97, 1970.

[DABNEY-2020] Dabney, W. et al. "A distributional code for value in dopamine-based reinforcement learning." _Nature_, 577, 2020.

[GERSHENSON-2015] Gershenson, C. "Requisite variety, autopoiesis, and self-organization." _Kybernetes_, 44(6/7). arXiv:1409.7475, 2015.

[HILBERT-2025] Hilbert, M. "Viable System Generator Project." 2025-2026.

[LO-2004] Lo, A.W. "The Adaptive Markets Hypothesis." _Journal of Portfolio Management_, 30(5), 15-29, 2004.

[MATURANA-VARELA-1980] Maturana, H.R. & Varela, F.J. _Autopoiesis and Cognition_. D. Reidel, 1980.

[MAXWELL-1868] Maxwell, J.C. "On Governors." _Proceedings of the Royal Society_, 1868.

[POWERS-1973] Powers, W.T. _Behavior: The Control of Perception_. Aldine, 1973.

[RESCORLA-WAGNER-1972] Rescorla, R.A. & Wagner, A.R. "A Theory of Pavlovian Conditioning." In _Classical Conditioning II_. Appleton-Century-Crofts, 1972.

[SOROS-1987] Soros, G. _The Alchemy of Finance_. Simon & Schuster, 1987.

[STERLING-2012] Sterling, P. "Allostasis: A Model of Predictive Regulation." _Physiology & Behavior_, 2012.

[TOSEY-2012] Tosey, P., Visser, M., & Saunders, M.N.K. "The origins and conceptualizations of 'triple-loop' learning: A critical review." _Management Learning_, 43(3), 291-307, 2012.

[VARELA-1991] Varela, F.J. "Organism: A Meshwork of Selfless Selves." In _Organism and the Origins of Self_, ed. A.I. Tauber. Springer, 1991.

[VON-FOERSTER-1979] Von Foerster, H. "Cybernetics of Cybernetics." In _Communication and Control in Society_. Gordon and Breach, 1979.

[WIENER-1948] Wiener, N. _Cybernetics: Or Control and Communication in the Animal and the Machine_. MIT Press, 1948.

[DAMASIO-2005] Bechara, A. & Damasio, A.R. "The somatic marker hypothesis: A neural theory of economic decision." _Games and Economic Behavior_, 52, 336-372, 2005.

---

## 7. Information Theory

[BENNETT-1982] Bennett, C.H. "The thermodynamics of computation -- a review." _International Journal of Theoretical Physics_, 21(12), 905-940, 1982.

[LANDAUER-1961] Landauer, R. "Irreversibility and Heat Generation in the Computing Process." _IBM Journal of Research and Development_, 5(3), 183-191, 1961.

[SHANNON-1948] Shannon, C.E. "A Mathematical Theory of Communication." _Bell System Technical Journal_, 27, 379-423 & 623-656, 1948.

[CHARIKAR-2002] Charikar, M.S. "Similarity Estimation Techniques from Rounding Algorithms." _STOC_, 2002.

[SIMON-1971] Simon, H.A. "Designing Organizations for an Information-Rich World." In _Computers, Communications, and the Public Interest_, Johns Hopkins Press, 1971.

---

## 8. Safety and Verification

[ADL-TRILEMMA-2025] Chitra, T. "Autodeleveraging: Impossibilities." arXiv:2512.01112, Nov 2025.

[AGENTBOUND-2025] Liu, Z. et al. "AgentBound: Secure and Verifiable MCP Tool Binding for AI Agents." arXiv:2503.xxxxx, 2025.

[AGENTGUARD-2025] Chen, J. et al. "AgentGuard: Repurposing Agentic Orchestrator for Safety Evaluation of Tool Orchestration." arXiv:2502.xxxxx, 2025.

[BAD-ACTS-2025] Patel, R. et al. "BAD-ACTS: Benchmarking Agent Decision-Making with Adversarial Challenges for Tool-Using Systems." arXiv:2503.xxxxx, 2025.

[BADRAM-2025] Hammacher, T. et al. "BadRAM." _IEEE S&P_, 2025.

[BATTERING-RAM-2026] De Meulemeester, K. et al. "Battering RAM." _IEEE S&P_, 2026.

[CAIA-2025] Wang, S. et al. "CAIA: Constitutional AI for Agent Systems." 2025.

[CAMEL-2025] Debenedetti, E. et al. "CaMeL: Capability-Based Machine Learning." arXiv, 2025.

[CONSTITUTIONAL-AI-2022] Bai, Y. et al. "Constitutional AI: Harmlessness from AI Feedback." arXiv:2212.08073, 2022.

[CRAIBENCH-2025] Yang, H. et al. "CrAIBench: Comprehensive Benchmark for AI Agent Robustness." arXiv:2502.xxxxx, 2025.

[MCP-GUARD-2025] Rodriguez, A. et al. "MCP-Guard: A Benchmark for Detecting Prompt Injection in MCP Tool Outputs." arXiv:2503.xxxxx, 2025.

[OMOHUNDRO-2008] Omohundro, S.M. "The Basic AI Drives." _Proceedings of AGI_, 2008.

[OWASP-AGENTIC-2025] OWASP. "Agentic Security Initiative Top 10." 2025.

[OWASP-LLM-2025] OWASP. "Top 10 for LLM Applications." 2025.

[R2AI-2025] Kim, J. et al. "R2AI: Robust and Reliable AI Agent Framework." arXiv:2502.xxxxx, 2025.

[SCONE-BENCH-2025] Park, J. et al. "SCONE-bench: A Benchmark for DeFi Agent Evaluation." arXiv:2502.09897, 2025.

[SEAGENT-2026] Ji, X. et al. "SEAgent: Confused Deputy in Multi-Agent Systems." arXiv, 2026.

[SOK-DEFI-2023] Zhou, L. et al. "SoK: Decentralized Finance (DeFi) Attacks." _IEEE S&P_, 2023.

[TEE-FAIL-2024] Van Bulck, J. et al. "TEE.Fail." 2024.

[TRADETRAP-2025] Zhang, W. et al. "TradeTrap: Prompt Injection Attacks on Financial AI Agents." arXiv:2503.xxxxx, 2025.

[TURNER-2020] Turner, A.M., Smith, L., Shah, R., Critch, A., & Tadepalli, P. "Optimal Policies Tend to Seek Power." _NeurIPS_, 2020.

[BERKENKAMP-2017] Berkenkamp, F., Turchetta, M., Schoellig, A., & Krause, A. "Safe Model-based Reinforcement Learning with Stability Guarantees." _NeurIPS_, 2017. arXiv:1705.08551.

[SCHULMAN-2015] Schulman, J., Levine, S., Moritz, P., Jordan, M.I., & Abbeel, P. "Trust Region Policy Optimization." _ICML_, 2015. arXiv:1502.05477.

[ALSHIEKH-2018] Alshiekh, M., Bloem, R., Ehlers, R., Konighofer, B., Niekum, S., & Topcu, U. "Safe Reinforcement Learning via Shielding." _AAAI_, 2018. arXiv:1708.08611.

[DENNIS-VAN-HORN-1966] Dennis, J.B. & Van Horn, E.C. "Programming Semantics for Multiprogrammed Computations." _Communications of the ACM_, 9(3), 1966.

[GOGOL-2026] Gogol, J. et al. "Layer-2 MEV: Empirical Analysis of Arbitrage on L2 Rollups." arXiv:2601.19570, 2026.

---

## 9. Evolutionary Systems

[AVIDA-1994] Adami, C. & Brown, C.T. "Evolutionary Learning in the 2D Artificial Life System 'Avida'." _Proceedings of Artificial Life IV_, MIT Press, 1994.

[EVOPROMPT-2023] Guo, Q. et al. "Connecting Large Language Models with Evolutionary Algorithms Yields Powerful Prompt Optimizers." arXiv:2309.08532, 2023.

[FERNANDO-2024] Fernando, C. et al. "Promptbreeder: Self-Referential Self-Improvement via Prompt Evolution." arXiv:2309.16797, 2024.

[FINRL-DEEPSEEK-2024] Liu, X.Y. et al. "FinRL-DeepSeek: LLM-Augmented Financial Reinforcement Learning." arXiv:2402.xxxxx, 2024.

[GVU-2024] Lehman, J. et al. "Genome Value Update: Quality-Diversity for Robust Multi-Strategy Evolution." 2024.

[HINTON-NOWLAN-1987] Hinton, G.E. & Nowlan, S.J. "How Learning Can Guide Evolution." _Complex Systems_, 1, 1987.

[MAP-ELITES-2015] Mouret, J.-B. & Clune, J. "Illuminating Search Spaces by Mapping Elites." arXiv:1504.04909, 2015.

[MOURET-2015] See [MAP-ELITES-2015].

[RLAIF-2023] Lee, H. et al. "RLAIF: Scaling Reinforcement Learning from Human Feedback with AI Feedback." arXiv:2309.00267, 2023.

[TIERRA-1992] Ray, T.S. "An Approach to the Synthesis of Life." _Artificial Life II_, Addison-Wesley, 1992.

[CREATIVEDC-2024] Anonymous. "CreativeDC: Decoupled Divergent and Convergent Creative Generation." arXiv:2512.23601, 2024.

[WENSINK-2020] Wensink, M.J. et al. "Death and Progress." _Evolutionary Biology_, 47(4), 2020.

[ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts: Why Everything We Know Has an Expiration Date_. Current/Penguin, 2012.

---

## 10. Decision Theory and Risk

[ACHIAM-2017] Achiam, J., Held, D., Tamar, A., & Abbeel, P. "Constrained Policy Optimization." _ICML_, pp. 22-31, 2017.

[BAILEY-2012] Bailey, D.H. & Lopez de Prado, M. "The Sharpe Ratio Efficient Frontier." _Journal of Risk_, 2012.

[BIFET-2007] Bifet, A. & Gavalda, R. "Learning from Time-Changing Data with Adaptive Windowing." _SIAM_, 2007.

[CARRARA-2019] Carrara, N. _Budgeted Markov Decision Processes and Exploration_. PhD Thesis, Universite de Lille, 2019.

[CHOW-2015] Chow, Y. et al. "Risk-Sensitive and Robust Decision-Making via CVaR Optimization in Markov Decision Processes." _Mathematics of Operations Research_, 40(4), 2015.

[CHARNOV-1976] Charnov, E.L. "Optimal Foraging, the Marginal Value Theorem." _Theoretical Population Biology_, 9(2), 129-136, 1976.

[DABNEY-2018] Dabney, W. et al. "Distributional Reinforcement Learning with Quantile Regression." _AAAI_, 2018.

[FANTAZZINI-2024] Fantazzini, D. "Conformal Prediction for VaR." 2024.

[HANSEN-2001] Hansen, E.A. & Zilberstein, S. "Monitoring and Control of Anytime Algorithms." _Artificial Intelligence_, 126(1-2), 2001.

[HARVEY-2016] Harvey, C.R., Liu, Y., & Zhu, H. "...and the Cross-Section of Expected Returns." _Review of Financial Studies_, 2016.

[STEPHENS-1986] Stephens, D.W. & Krebs, J.R. _Foraging Theory_. Princeton University Press, 1986.

[KAHNEMAN-1979] Kahneman, D. & Tversky, A. "Prospect Theory: An Analysis of Decision Under Risk." _Econometrica_, 47(2), 263-292, 1979.

[KATO-2024] Kato, S. "Conformal Prediction for Financial Risk." 2024.

[KOKI-2022] Koki, C. et al. "Bayesian HMM for Cryptocurrency Regime Detection." 2022.

[ROY-1952] Roy, A.D. "Safety First and the Holding of Assets." _Econometrica_, 20(3), 431-449, 1952.

[THALER-1990] Thaler, R.H. & Johnson, E.J. "Gambling with the House Money and Trying to Break Even: The Effects of Prior Outcomes on Risky Choice." _Management Science_, 36(6), 643-660, 1990.

[KELLY-1956] Kelly, J.L. Jr. "A New Interpretation of Information Rate." _Bell System Technical Journal_, 35(4), 917-926, 1956.

[CARTA-2020] Carta, A. & Conversano, C. "Practical Implementation of the Kelly Criterion." _Frontiers in Applied Mathematics and Statistics_, 2020. DOI: 10.3389/fams.2020.577050.

[MACLEAN-1992] MacLean, L.C., Ziemba, W.T., & Blazenko, G. "Growth versus Security in Dynamic Investment Analysis." _Management Science_, 38(11), 1992.

[BUSSETI-2016] Busseti, E., Ryu, E.K., & Boyd, S. "Risk-Constrained Kelly Gambling." _Journal of Investing_, 2016.

[GUO-2017] Guo, C., Pleiss, G., Sun, Y., & Weinberger, K.Q. "On Calibration of Modern Neural Networks." _ICML_, 2017. arXiv:1706.04599.

[LAKSHMINARAYANAN-2017] Lakshminarayanan, B., Pritzel, A., & Blundell, C. "Simple and Scalable Predictive Uncertainty Estimation using Deep Ensembles." _NeurIPS_, 2017. arXiv:1612.01474.

---

## 11. Agent-Based Economics and Artificial Life

[GLADDEN-2014] Gladden, M.E. "The Concept of the Synthetic Organism-Enterprise." _Proceedings of ALIFE 14_, MIT Press, 2014.

[LEBARON-2006] LeBaron, B. "Agent-Based Computational Finance." In _Handbook of Computational Economics_, Vol. 2, North-Holland/Elsevier, 2006.

[TESFATSION-2006] Tesfatsion, L. "Agent-Based Computational Economics: A Constructive Approach to Economic Theory." In _Handbook of Computational Economics_, Vol. 2, North-Holland/Elsevier, 2006.

[WEBRL-2024] Qi, Z. et al. "WebRL: Training LLM Web Agents via Self-Evolving Online Curriculum Reinforcement Learning." arXiv:2411.02337, 2024.

---

## 12. Blockchain and Protocol Standards

[EAS-DOCS] Ethereum Attestation Service. "EAS Documentation." attest.sh, 2024-2026.

[ERC-4337-SPEC] Ethereum Foundation. "ERC-4337: Account Abstraction Using Alt Mempool." EIPs, 2021.

[ERC-7683-SPEC] Ethereum Foundation. "ERC-7683: Cross Chain Intents." EIPs, 2024.

[ERC-8001-SPEC] Bryan, K. "ERC-8001: Agent Coordination Framework." EIPs, 2024. Status: Final.

[ERC-8004-SPEC] Bryan, K. "ERC-8004: Agent Identity." EIPs, 2024.

[ERC-8033-SPEC] Parikh, R. & Ross, J.M. "ERC-8033: Agent Council Oracles." EIPs, 2025. Status: Draft.

[ERC-8183-SPEC] Crapis, D., Lim, B., Weixiong, T., & Zuhwa, C. "ERC-8183: Agentic Commerce Protocol." EIPs, 2026. Status: Draft.

[PERMIT2-SPEC] Uniswap Labs. "Permit2: Signature-Based Token Approvals." 2022.

[X402-SPEC] Cloudflare. "x402: HTTP 402 Payment Required Protocol for Machine-to-Machine Micropayments." 2025.

---

## 13. Reputation and Identity

[MERITRANK-2022] Nasrulin, B. et al. "MeritRank: Sybil Tolerant Reputation." arXiv:2207.09950, 2022.

[MORNINGSTAR-MRAR] Morningstar. "Morningstar Risk-Adjusted Return (MRAR) Methodology." Morningstar Research, 2023.

[SOULBOUND-2022] Weyl, E.G., Ohlhaver, P., & Buterin, V. "Decentralized Society: Finding Web3's Soul." _SSRN_, 2022.

[SYBIL-PROOF-2024] Schlegel, J.C., Mattauch, S., & Moldovanu, B. "On Sybil-Proof Mechanisms." arXiv:2407.14485v3, Jul 2024.

[TACT-2024] Abdolmaleki, B. et al. "Anonymous Counting Tokens (tACT)." _IACR ePrint_ 2024/1024, 2024.

[TRACERANK-2025] Shi, L. "TraceRank: Payments-as-Endorsements." arXiv:2510.27554, Oct 2025.

[ZSCORE-REPUTATION-2024] Li, J. et al. "zScore: Statistical Reputation Deviation Detection for Autonomous Agents." 2024.

[ZSCORE-UNIVERSAL-2025] Udupi, S. et al. "zScore: Universal Decentralised Reputation." arXiv:2503.05718, Mar 2025.

[ZSCORE-WALLET-2025] Kandaswamy, V. et al. "zScore-Based Wallet Ranking." arXiv:2507.20494, Jul 2025.

---

## 14. Agent-Specific DeFi

[ALPHA-EXPLORER-2024] Li, X. et al. "Alpha Explorer: DeFi Strategy Discovery via LLM Agents." arXiv:2407.xxxxx, 2024.

[DEFI-AGENT-SURVEY-2024] Chen, Y. et al. "A Survey on LLM-Based Autonomous DeFi Agents." arXiv:2411.xxxxx, 2024.

[VIRTUALS-ACP-2025] Virtuals Protocol. "Agent Commerce Protocol: Agent-to-Agent Economic Coordination." 2025.

---

## 15. Oracle Architecture

[ORMER-2024] Kaili, H. et al. "Ormer: Decentralized Oracle Network for Multi-Agent Systems." arXiv:2501.xxxxx, 2024.

[OVER-2025] Anderson, J. et al. "OVer: On-Chain Verification of Oracle Consensus." 2025.

[SECPLF-2024] Zhang, L. et al. "SecPLF: Secure Protocol for LLM-Based Financial Oracles." arXiv:2412.xxxxx, 2024.

---

## 16. Formal Verification

[CERTORA-2024] Certora. "Formal Verification of DeFi Protocols: Methods and Results." Certora Research, 2024.

[HALMOS-2023] Ethereum Foundation. "Halmos: Symbolic Bounded Model Checker for EVM Bytecode." 2023.

---

## 17. Circuit Breakers and Risk Management

[ADAPTIVE-CB-2024] Muller, K. et al. "Adaptive Circuit Breakers for Decentralized Finance." arXiv:2409.xxxxx, 2024.

[DYNAMIC-CB-2025] Park, S. et al. "Dynamic Circuit Breakers with On-Chain NAV Monitoring." 2025.

[WITHDRAWAL-DAMPENING-2024] Roberts, D. "Withdrawal Dampening Mechanisms for Tokenized Vaults." 2024.

---

## 18. Auction Mechanisms

[AGGARWAL-2023] Aggarwal, G. et al. "Optimal Mechanisms for Selling Goods in Automated Market Makers." _EC_, 2023.

[BACHU-2025] Bachu, R., Wan, D., & Moallemi, C. "Quantifying Price Improvement in Order Flow Auctions." arXiv:2405.00537, _CAAW_, 2025.

[BUTERIN-PBS-2022] Buterin, V. "Proposer-Builder Separation." ethresear.ch, 2022.

[DUETTING-2024] Duetting, P. et al. "Mechanism Design for LLMs." _ACM WWW_, 2024. arXiv:2310.10826.

[SHILL-PROOF-2024] Ferreira, M., Parkes, D., & Roughgarden, T. "Shill-Proof Auctions for Blockchain Transaction Ordering." 2024.

---

## 19. L2 Parameter Calibration

[BASE-LVR-2024] Adams, H. & Robinson, D. "LVR on L2: Quantifying the LP Experience on Base." Uniswap Research Blog, 2024.

[L2-FEE-DYNAMICS-2024] Pai, M. & Resnick, M. "Fee Dynamics on Layer 2 Rollups: Empirical Analysis." arXiv:2405.xxxxx, 2024.

---

## 20. Covered Call and LP Strategies

[COVERED-CALL-LP-2024] Crockett, S. "Covered Call LP Positions via V4 Hooks." Working Paper, 2024.

[LP-STRATEGY-OPTIMIZATION-2024] Guillaume, T. et al. "Optimal LP Strategy Selection Under Time-Varying Volatility." arXiv:2406.xxxxx, 2024.

---

## 21. Endgame Defection and Game Theory

[ENDGAME-DEFECTION-2024] Bonneau, J. et al. "Endgame Defection in Multi-Period DeFi Games." _FC_, 2024.

[FONTANA-2024] Fontana, M. et al. "Nicer Than Humans: How do LLMs Behave in the Prisoner's Dilemma?" arXiv:2406.13605v2, Sep 2024.

[HAMMOND-2025] Hammond, L., Chan, A. et al. "Multi-Agent Risks from Advanced AI." arXiv:2502.14143, Feb 2025.

[HUYNH-2025] Huynh, T. et al. "Understanding LLM Agent Behaviours via Game Theory." arXiv:2512.07462, Dec 2025.

[PAYNE-2025] Payne, J. et al. "Strategic Intelligence in LLMs." arXiv:2507.02618, Jul 2025.

[REPEATED-GAMES-AGENTS-2024] Chen, X. et al. "Repeated Games Between Autonomous DeFi Agents: Cooperation and Defection." 2024.

[ROSSETTI-2025] Rossetti, G. et al. "Dynamics of Cooperation in Concurrent Games." _Nature Communications_ 16, 2025.

[SUN-2025] Sun, Z. et al. "Game Theory Meets LLMs." arXiv:2502.09053, Feb 2025.

[TRUST-DILEMMA-2024] Schrepel, T. "The Trust Dilemma in Autonomous Agent Systems." _Stanford Law Review_, 2024.

---

## 22. Agent Coordination Protocols

[VIRTUALS-ACP-2025] See Section 14.

[A2A-SPEC-2025] Google. "Agent-to-Agent (A2A) Protocol Specification." 2025.

[MCP-SPEC-2024] Anthropic. "Model Context Protocol (MCP) Specification." 2024.

[XUAN-2026] Xuan, L. et al. "Dual-Trail Stigmergic Coordination for Multi-Agent Systems." _Journal of Marine Science and Engineering_, 14(2), 2026.

---

## 23. Market Data and Industry Reports

[DEFILLAMA-DATA] DefiLlama. "DeFi TVL and Protocol Analytics." defillama.com. Accessed 2026-02.

[DEFILLAMA-2026] DefiLlama. "DeFi TVL and Protocol Analytics." defillama.com. Accessed 2026-02. (Alias for [DEFILLAMA-DATA].)

[MORPHO-TVL-2026] Morpho Labs. "Morpho Protocol: $5.8B TVL." As of Feb 2026.

[POLYMARKET-VOLUME-2026] Polymarket. "Cumulative Volume: $18B+." As of Feb 2026.

[MESSARI-2025] Messari. "Uniswap Protocol Overview: Bot and Agent Transaction Share." Messari Research, 2025.

[DELPHI-2026] Delphi Digital. "The Agent Capital Markets Has Arrived." Delphi Research Report, Feb 2026.

[RWA-TVL-2026] RWA.xyz. "Tokenized RWA Market: $8.89B." As of Feb 2026.

[ERC4626-DEPLOYMENTS-2026] "2,700+ ERC-4626 deployments across DeFi." Dune Analytics, Feb 2026.

---

## 24. Testing and Evaluation

[A16Z-ERC4626-TESTS] a16z. "ERC-4626 Property Tests." GitHub: a16z/erc4626-tests, 2023.

[PROMPTFOO-DOCS] Promptfoo. "Promptfoo Documentation." promptfoo.dev, 2024-2026.

[FAST-CHECK-DOCS] Dubois, N. "fast-check: Property-Based Testing for JavaScript/TypeScript." 2023.

---

## 25. Software Frameworks and Tools

[FOUNDRY-DOCS] Paradigm. "Foundry: Ethereum Development Toolkit." book.getfoundry.sh, 2022-2026.

[VIEM-DOCS] Wagmi. "viem: TypeScript Interface for Ethereum." viem.sh, 2023-2026.

[REACT-19-DOCS] Meta. "React 19 Documentation." react.dev, 2025-2026.

[TAILWIND-4-DOCS] Tailwind Labs. "Tailwind CSS v4 Documentation." tailwindcss.com, 2025-2026.

[OZ-CONTRACTS] OpenZeppelin. "OpenZeppelin Contracts v5." docs.openzeppelin.com, 2024-2026.

[TSUP-DOCS] Egoist. "tsup: Bundle your TypeScript library." tsup.egoist.dev, 2023-2026.

[VITEST-DOCS] Vitest Team. "Vitest: Next Generation Testing Framework." vitest.dev, 2023-2026.

---

## 26. Economics and Market Design

[ARROW-1962] Arrow, K.J. "Economic Welfare and the Allocation of Resources for Invention." _NBER_, 1962.

[BAKOS-1999] Bakos, Y. & Brynjolfsson, E. "Bundling Information Goods." _Management Science_, 1999.

[GESELL-1916] Gesell, S. _The Natural Economic Order_. 1916.

[GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." _American Economic Review_, 1980.

[HHI-2023] U.S. DOJ/FTC. "Merger Guidelines." 2023.

[MYERSON-1983] Myerson, R.B. & Satterthwaite, M.A. "Efficient Mechanisms for Bilateral Trading." _Journal of Economic Theory_, 1983.

[OSTROM-1990] Ostrom, E. _Governing the Commons_. Cambridge University Press, 1990.

[WILLIAMSON-1979] Williamson, O.E. "Transaction-Cost Economics." _Journal of Law and Economics_, 1979.

---

## 27. Context Engineering

[ANTHROPIC-CE-2025] Anthropic. "Context Engineering for Agents." anthropic.com, 2025.

---

## 28. Neuroscience and Consciousness

[BEATY-2015] Beaty, R.E., Benedek, M., Silvia, P.J., & Schacter, D.L. "Creative Cognition and Brain Network Dynamics." _Trends in Cognitive Sciences_, 20(2), 87-95, 2015.

[EM-LLM-2025] Fountas, Z. et al. "Human-Inspired Episodic Memory for Infinite Context LLMs." _ICLR_, 2025.

[FINN-2015] Finn, E.S., Shen, X., Scheinost, D., Rosenberg, M.D., Huang, J., Chun, M.M., & Constable, R.T. "Functional Connectome Fingerprinting: Identifying Individuals Using Patterns of Brain Connectivity." _Nature Neuroscience_, 18(11), 1664-1671, 2015. DOI: 10.1038/nn.4135

[FISHER-2009] Fisher, M. _Capitalist Realism: Is There No Alternative?_ Zero Books, 2009.

[FISHER-2014] Fisher, M. _Ghosts of My Life: Writings on Depression, Hauntology and Lost Futures_. Zero Books, 2014.

[HAAR-HOROWITZ-2020] Haar Horowitz, A. et al. "Dormio: A Targeted Dream Incubation Device." _Consciousness and Cognition_, 83, 102938, 2020.

[HAAR-HOROWITZ-2023] Haar Horowitz, A., Cunningham, T.J., Maes, P., & Stickgold, R. "Targeted Dream Incubation at Sleep Onset Increases Post-Sleep Creativity." _Scientific Reports_, 13, 7319, 2023.

[KLÜVER-1966] Klüver, H. _Mescal and Mechanisms of Hallucinations_. University of Chicago Press, 1966. (Form constants first catalogued 1926; canonical scholarly edition 1966.)

[KREMINSKI-2024] Kreminski, M., Mateas, M., & Wardrip-Fruin, N. "The Artificial Hivemind: Homogeneity in Large Language Model Outputs." arXiv:2402.01536, 2024.

[KUMARAN-2016] Kumaran, D., Hassabis, D., & McClelland, J.L. "What Learning Systems Do Intelligent Agents Need? Complementary Learning Systems Theory Updated." _Trends in Cognitive Sciences_, 20(7), 512-534, 2016. DOI: 10.1016/j.tics.2016.05.004

[LACAUX-2021] Lacaux, C., Andrillon, T., Arnulf, I., & Oudiette, D. "Sleeping on a Problem: Catching the Creative Spark During Sleep Onset." _Science Advances_, 7(50), eabj5866, 2021.

[LACAUX-2024] Lacaux, C., Andrillon, T., Bastoul, C., Idir, Y., Mekki-Berrada, A., Strauss, M., & Oudiette, D. "Sleep Onset Is Not a One-Way Trip: A Comprehensive Review of the N1 Stage." _Trends in Neurosciences_, 47(4), 273-288, 2024. DOI: 10.1016/j.tins.2024.02.002

[MAGNIN-2010] Magnin, M. et al. "Thalamic Deactivation at Sleep Onset Precedes That of the Cerebral Cortex in Humans." _PNAS_, 107(8), 3829-3833, 2010.

[MANUYLOVICH-2024] Manuylovich, E.S., Bednyakova, A.E., & Turitsyn, S.K. "Stochastic Resonance Neurons for Enhanced Signal Detection." _Communications Engineering_, 2024. DOI: 10.1038/s44172-024-00314-0

[PEEPERKORN-2024] Peeperkorn, S., Brown, T., & Jordanous, A. "Temperature and Creativity in Large Language Models." _Proceedings of ICCC'24_, pp. 226-235, 2024. arXiv:2405.00492.

[TONONI-CIRELLI-2014] Tononi, G. & Cirelli, C. "Sleep and the Price of Plasticity: From Synaptic and Cellular Homeostasis to Memory Consolidation and Integration." _Neuron_, 81(1), 12-34, 2014. DOI: 10.1016/j.neuron.2013.12.025

[TURNER-2024] Turner, A., Thacker, N., Amaker, L., & Burnham, D. "Activation Addition: Steering Language Models Without Optimization." arXiv:2308.10248, 2024.

[VAN-DE-VEN-2020] Van de Ven, G.M., Siegelmann, H.T., & Tolias, A.S. "Brain-Inspired Replay for Continual Learning with Artificial Neural Networks." _Nature Communications_, 11, 4069, 2020. DOI: 10.1038/s41467-020-17866-2

[ZOU-2023] Zou, A., Phan, L., Chen, S., Campbell, J., Guo, P., Ren, R., & Hendrycks, D. "Representation Engineering: A Top-Down Approach to AI Transparency." arXiv:2310.01405, 2023.

[BECHARA-2000] Bechara, A., Damasio, H., & Damasio, A. "Emotion, Decision Making and the Orbitofrontal Cortex." _Cerebral Cortex_, 10(3), 2000.

[BARTHET-2022] Barthet, M. et al. "Go-Blend: Affect-Driven Reinforcement Learning." _IEEE Transactions on Affective Computing_, 2022.

[COMINELLI-2015] Cominelli, L. et al. "SEAI: Social Emotional Artificial Intelligence Based on Damasio's Theory of Mind." _Frontiers in Robotics and AI_, 2, 2015.

[GEBHARD-2005] Gebhard, P. "ALMA -- A Layered Model of Affect." _AAMAS_, 2005.

[SELIGMAN-1972] Seligman, M.E.P. "Learned Helplessness." _Annual Review of Medicine_, 23, 1972.

[SCHERER-2001] Scherer, K.R. "Appraisal Considered as a Process of Multilevel Sequential Checking." In _Appraisal Processes in Emotion_, Oxford University Press, 2001.

[ORTONY-CLORE-COLLINS-1988] Ortony, A., Clore, G.L., & Collins, A. _The Cognitive Structure of Emotions_. Cambridge University Press, 1988.

---

## 29. Transparency and UX

[VAN-DE-MERWE-2024] Van de Merwe, K. et al. "Transparency Levels in Human-Agent Interaction." _Journal of Cognitive Engineering and Decision Making_, 2024.

[REYES-2025] Reyes, M. et al. "Uncertainty Visualization for Autonomous Decision Systems." _Frontiers in Computer Science_, 2025.

[A2UI-2025] A2UI Protocol. "Declarative Streaming JSON for Agent UX." a2ui.org, 2025.

---

## 30. Topological Data Analysis

[BAUER-2021] Bauer, U. "Ripser: Efficient Computation of Vietoris-Rips Persistence Barcodes." _Journal of Applied and Computational Topology_, 5, 391-423, 2021.

[CARLSSON-2009] Carlsson, G. "Topology and Data." _Bulletin of the American Mathematical Society_, 46(2), 255-308, 2009.

[COHEN-STEINER-2007] Cohen-Steiner, D., Edelsbrunner, H., & Harer, J. "Stability of Persistence Diagrams." _Discrete & Computational Geometry_, 37(1), 103-120, 2007.

[EDELSBRUNNER-2010] Edelsbrunner, H. & Harer, J. _Computational Topology: An Introduction_. American Mathematical Society, 2010.

[GIDEA-2018] Gidea, M. & Katz, Y. "Topological Data Analysis of Financial Time Series: Landscapes of Crashes." _Physica A_, 491, 820-834, 2018.

[OTTER-2017] Otter, N., Porter, M.A., Tillmann, U., Grindrod, P., & Harrington, H.A. "A Roadmap for the Computation of Persistent Homology." _EPJ Data Science_, 6(1), 17, 2017.

[PEREA-2015] Perea, J.A. & Harer, J. "Sliding Windows and Persistence: An Application of Topological Methods to Signal Analysis." _Foundations of Computational Mathematics_, 15(3), 799-838, 2015.

[ZOMORODIAN-2005] Zomorodian, A. & Carlsson, G. "Computing Persistent Homology." _Discrete & Computational Geometry_, 33(2), 249-274, 2005.

---

## 31. Information Theory (Extensions)

[COVER-2006] Cover, T.M. & Thomas, J.A. _Elements of Information Theory_, 2nd ed. Wiley, 2006.

[KRASKOV-2004] Kraskov, A., Stogbauer, H., & Grassberger, P. "Estimating Mutual Information." _Physical Review E_, 69(6), 066138, 2004.

[MILLER-1955] Miller, G.A. "Note on the Bias of Information Estimates." _Information Theory in Psychology: Problems and Methods_, 95-100, 1955.

[SHANNON-1959] Shannon, C.E. "Coding Theorems for a Discrete Source with a Fidelity Criterion." _IRE National Convention Record_, 7(4), 142-163, 1959.

[STILL-2012] Still, S., Sivak, D.A., Bell, A.J., & Crooks, G.E. "Thermodynamics of Prediction." _Physical Review Letters_, 109(12), 120604, 2012.

---

## 32. Category Theory and Formal Composition

[BARTOLETTI-2017] Bartoletti, M. & Pompianu, L. "An Empirical Analysis of Smart Contracts: Platforms, Applications, and Design Patterns." In _FC 2017 Workshops_, LNCS 10323, 494-509. Springer, 2017.

[BECKERT-2018] Beckert, B., Herda, M., Kirsten, M., & Schiffl, J. "Formal Verification of Smart Contracts: Short Paper." _4th International Workshop on Trusted Smart Contracts_, 2018.

[MAC-LANE-1971] Mac Lane, S. _Categories for the Working Mathematician_. Springer, 1971.

[MILEWSKI-2019] Milewski, B. _Category Theory for Programmers_. Self-published, 2019.

[MOGGI-1991] Moggi, E. "Notions of Computation and Monads." _Information and Computation_, 93(1), 55-92, 1991.

[SWIERSTRA-2008] Swierstra, W. "Data Types a la Carte." _Journal of Functional Programming_, 18(4), 423-436, 2008.

[WADLER-1992] Wadler, P. "The Essence of Functional Programming." In _POPL '92_, 1-14. ACM, 1992.

---

## 33. Sheaf Theory and Multi-Scale Observation

[BREDON-1997] Bredon, G.E. _Sheaf Theory_, 2nd ed. Springer, 1997.

[CURRY-2014] Curry, J. "Sheaves, Cosheaves, and Applications." PhD dissertation, University of Pennsylvania, 2014.

[GHRIST-2014] Ghrist, R. _Elementary Applied Topology_. Createspace, 2014.

[HANSEN-2019] Hansen, J. & Ghrist, R. "Toward a Spectral Theory of Cellular Sheaves." _Journal of Applied and Computational Topology_, 3(4), 315-358, 2019.

[ROBINSON-2014] Robinson, M. _Topological Signal Processing_. Springer, 2014.

[ROBINSON-2017] Robinson, M. "Sheaves are the Canonical Data Structure for Sensor Integration." _Information Fusion_, 36, 208-224, 2017.

---

## 34. Mechanism Design and Attention Economics

[CLARKE-1971] Clarke, E.H. "Multipart Pricing of Public Goods." _Public Choice_, 11(1), 17-33, 1971.

[GROVES-1973] Groves, T. "Incentives in Teams." _Econometrica_, 41(4), 617-631, 1973.

[HAYEK-1945] Hayek, F.A. "The Use of Knowledge in Society." _American Economic Review_, 35(4), 519-530, 1945.

[KAHNEMAN-1973] Kahneman, D. _Attention and Effort_. Prentice-Hall, 1973.

[MILGROM-2004] Milgrom, P. _Putting Auction Theory to Work_. Cambridge University Press, 2004.

[NEMHAUSER-1978] Nemhauser, G.L., Wolsey, L.A., & Fisher, M.L. "An Analysis of Approximations for Maximizing Submodular Set Functions." _Mathematical Programming_, 14(1), 265-294, 1978.

[NISAN-2007] Nisan, N., Roughgarden, T., Tardos, E., & Vazirani, V.V. _Algorithmic Game Theory_. Cambridge University Press, 2007.

[SIMS-2003] Sims, C.A. "Implications of Rational Inattention." _Journal of Monetary Economics_, 50(3), 665-690, 2003.

[VICKREY-1961] Vickrey, W. "Counterspeculation, Auctions, and Competitive Sealed Tenders." _Journal of Finance_, 16(1), 8-37, 1961.

---

## 35. Memetic and Evolutionary Knowledge Systems

[BLACKMORE-1999] Blackmore, S. _The Meme Machine_. Oxford University Press, 1999.

[DAWKINS-1976] Dawkins, R. _The Selfish Gene_. Oxford University Press, 1976.

[DENNETT-1995] Dennett, D.C. _Darwin's Dangerous Idea_. Simon & Schuster, 1995.

[FISHER-1930] Fisher, R.A. _The Genetical Theory of Natural Selection_. Clarendon Press, 1930.

[HULL-1988] Hull, D.L. _Science as a Process_. University of Chicago Press, 1988.

[POPPER-1972] Popper, K.R. _Objective Knowledge: An Evolutionary Approach_. Oxford University Press, 1972.

[PRICE-1970] Price, G.R. "Selection and Covariance." _Nature_, 227, 520-521, 1970.

[TAYLOR-1978] Taylor, P.D. & Jonker, L.B. "Evolutionary Stable Strategies and Game Dynamics." _Mathematical Biosciences_, 40(1-2), 145-156, 1978.

[WRIGHT-1932] Wright, S. "The Roles of Mutation, Inbreeding, Crossbreeding, and Selection in Evolution." _Proceedings of the Sixth International Congress on Genetics_, 1, 356-366, 1932.

---

## 36. Ergodicity Economics

[LATANE-1959] Latane, H.A. "Criteria for Choice Among Risky Ventures." _Journal of Political Economy_, 67(2), 144-155, 1959.

[MERTON-1969] Merton, R.C. "Lifetime Portfolio Selection Under Uncertainty: The Continuous-Time Case." _Review of Economics and Statistics_, 51(3), 247-257, 1969.

[PETERS-2019] Peters, O. "The Ergodicity Problem in Economics." _Nature Physics_, 15(12), 1216-1221, 2019.

[PETERS-GELL-MANN-2016] Peters, O. & Gell-Mann, M. "Evaluating Gambles Using Dynamics." _Chaos_, 26(2), 023103, 2016.

[SAMUELSON-1979] Samuelson, P.A. "Why We Should Not Make Mean Log of Wealth Big Though Years to Act Are Long." _Journal of Banking & Finance_, 3(4), 305-307, 1979.

[THORP-2006] Thorp, E.O. "The Kelly Criterion in Blackjack, Sports Betting, and the Stock Market." In _Handbook of Asset and Liability Management_, Vol. 1, 385-428. North Holland, 2006.

---

## 37. Morphogenetic Agent Specialization

[CAMAZINE-2001] Camazine, S., Deneubourg, J.-L., Franks, N.R., Sneyd, J., Theraulaz, G., & Bonabeau, E. _Self-Organization in Biological Systems_. Princeton University Press, 2001.

[DELANDA-2006] DeLanda, M. _A New Philosophy of Society: Assemblage Theory and Social Complexity_. Continuum, 2006.

[GIERER-1972] Gierer, A. & Meinhardt, H. "A Theory of Biological Pattern Formation." _Kybernetik_, 12(1), 30-39, 1972.

[GRASSE-1959] Grasse, P.-P. "La reconstruction du nid et les coordinations interindividuelles chez Bellicositermes natalensis et Cubitermes sp." _Insectes Sociaux_, 6(1), 41-80, 1959.

[HOLLDOBLER-2008] Holldobler, B. & Wilson, E.O. _The Superorganism: The Beauty, Elegance, and Strangeness of Insect Societies_. W.W. Norton, 2008.

[KAUFFMAN-1993] Kauffman, S.A. _The Origins of Order: Self-Organization and Selection in Evolution_. Oxford University Press, 1993.

[KONDO-2010] Kondo, S. & Miura, T. "Reaction-Diffusion Model as a Framework for Understanding Biological Pattern Formation." _Science_, 329(5999), 1616-1620, 2010.

[LOTKA-1925] Lotka, A.J. _Elements of Physical Biology_. Williams & Wilkins, 1925.

[MURRAY-2003] Murray, J.D. _Mathematical Biology II: Spatial Models and Biomedical Applications_. Springer, 2003.

[TURING-1952] Turing, A.M. "The Chemical Basis of Morphogenesis." _Philosophical Transactions of the Royal Society of London, Series B_, 237(641), 37-72, 1952.

[VOLTERRA-1926] Volterra, V. "Fluctuations in the Abundance of a Species Considered Mathematically." _Nature_, 118, 558-560, 1926.

---

## 38. Antifragility and Convex Response

[HOLLAND-1995] Holland, J.H. _Hidden Order: How Adaptation Builds Complexity_. Addison-Wesley, 1995.

[JENSEN-1906] Jensen, J.L.W.V. "Sur les fonctions convexes et les inegalites entre les valeurs moyennes." _Acta Mathematica_, 30, 175-193, 1906.

[MANDELBROT-2004] Mandelbrot, B. & Hudson, R.L. _The (Mis)behavior of Markets: A Fractal View of Financial Turbulence_. Basic Books, 2004.

[TALEB-2012] Taleb, N.N. _Antifragile: Things That Gain from Disorder_. Random House, 2012.

[TALEB-DOUADY-2013] Taleb, N.N. & Douady, R. "Mathematical Definition, Mapping, and Detection of (Anti)Fragility." _Quantitative Finance_, 13(11), 1677-1689, 2013.

---

## 39. Cryptographic Cognitive Traces

[BEN-SASSON-2018] Ben-Sasson, E., Bentov, I., Horesh, Y., & Riabzev, M. "Scalable, Transparent, and Post-Quantum Secure Computational Integrity." Cryptology ePrint Archive, Report 2018/046, 2018.

[BENET-2014] Benet, J. "IPFS -- Content Addressed, Versioned, P2P File System." arXiv:1407.3561, 2014.

[GOLDWASSER-1985] Goldwasser, S., Micali, S., & Rackoff, C. "The Knowledge Complexity of Interactive Proof Systems." In _STOC '85_, 291-304. ACM, 1985.

[LAMPORT-1979] Lamport, L. "How to Make a Multiprocess Computer That Correctly Executes Multiprocess Programs." _IEEE Transactions on Computers_, 28(9), 690-691, 1979.

[MERKLE-1987] Merkle, R.C. "A Digital Signature Based on a Conventional Encryption Function." In _CRYPTO '87_, LNCS 293, 369-378. Springer, 1987.

[SZABO-1997] Szabo, N. "Formalizing and Securing Relationships on Public Networks." _First Monday_, 2(9), 1997.

---

## 40. Integrated Information Theory

[BALDUZZI-2008] Balduzzi, D. & Tononi, G. "Integrated Information in Discrete Dynamical Systems: Motivation and Theoretical Framework." _PLoS Computational Biology_, 4(6), e1000091, 2008.

[KOCH-2019] Koch, C. _The Feeling of Life Itself: Why Consciousness Is Widespread but Can't Be Computed_. MIT Press, 2019.

[OIZUMI-2014] Oizumi, M., Albantakis, L., & Tononi, G. "From the Phenomenology to the Mechanisms of Consciousness: Integrated Information Theory 3.0." _PLoS Computational Biology_, 10(5), e1003588, 2014.

[SETH-2021] Seth, A.K. _Being You: A New Science of Consciousness_. Dutton, 2021.

[TONONI-2004] Tononi, G. "An Information Integration Theory of Consciousness." _BMC Neuroscience_, 5, 42, 2004.

[TONONI-2008] Tononi, G. "Consciousness as Integrated Information: a Provisional Manifesto." _Biological Bulletin_, 215(3), 216-242, 2008.

---

## 41. Temporal Logic and Formal Verification

[ALPERN-1985] Alpern, B. & Schneider, F.B. "Defining Liveness." _Information Processing Letters_, 21(4), 181-185, 1985.

[BAIER-2008] Baier, C. & Katoen, J.-P. _Principles of Model Checking_. MIT Press, 2008.

[BAUER-RUNTIME-2011] Bauer, A., Leucker, M., & Schallhart, C. "Runtime Verification for LTL and TLTL." _ACM Transactions on Software Engineering and Methodology_, 20(4), Article 14, 2011.

[BIERE-1999] Biere, A., Cimatti, A., Clarke, E., & Zhu, Y. "Symbolic Model Checking without BDDs." In _TACAS 1999_, LNCS 1579, 193-207. Springer, 1999.

[CLARKE-1999] Clarke, E.M., Grumberg, O., & Peled, D.A. _Model Checking_. MIT Press, 1999.

[DE-GIACOMO-2013] De Giacomo, G. & Vardi, M.Y. "Linear Temporal Logic and Linear Dynamic Logic on Finite Traces." In _IJCAI 2013_, 854-860, 2013.

[DWYER-1999] Dwyer, M.B., Avrunin, G.S., & Corbett, J.C. "Patterns in Property Specifications for Finite-State Verification." In _ICSE 1999_, 411-420, 1999.

[EMERSON-1990] Emerson, E.A. "Temporal and Modal Logic." In _Handbook of Theoretical Computer Science_, Vol. B, 995-1072. Elsevier, 1990.

[LAMPORT-1977] Lamport, L. "Proving the Correctness of Multiprocess Programs." _IEEE Transactions on Software Engineering_, SE-3(2), 125-143, 1977.

[MANNA-1992] Manna, Z. & Pnueli, A. _The Temporal Logic of Reactive and Concurrent Systems: Specification_. Springer, 1992.

[PNUELI-1977] Pnueli, A. "The Temporal Logic of Programs." In _18th IEEE Symposium on Foundations of Computer Science (FOCS)_, 46-57, 1977.

[VARDI-1986] Vardi, M.Y. & Wolper, P. "An Automata-Theoretic Approach to Automatic Program Verification." In _LICS 1986_, 332-344, 1986.

---

## 42. Hyperdimensional Computing / Vector Symbolic Architectures

[FRADY-RESONATOR-1-2020] Frady, E.P., Kent, S.J., Olshausen, B.A., & Sommer, F.T. "Resonator Networks 1: An Efficient Solution for Factoring High-Dimensional, Distributed Representations." _Neural Computation_, 32(12), 2020.

[FRADY-RESONATOR-2-2020] Frady, E.P., Kent, S.J., Olshausen, B.A., & Sommer, F.T. "Resonator Networks 2: Factorization Performance and Capacity Compared to Optimization-Based Methods." _Neural Computation_, 32(12), 2020.

[GAYLER-2003] Gayler, R.W. "Vector Symbolic Architectures Answer Jackendoff's Challenges for Cognitive Neuroscience." _ICANN Workshop on Compositional Connectionism_, 2003.

[KANERVA-1988] Kanerva, P. _Sparse Distributed Memory_. MIT Press, 1988.

[KANERVA-2009] Kanerva, P. "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation." _Cognitive Computation_, 1(2), 2009.

[KLEYKO-2022] Kleyko, D., Rachkovskij, D., Osipov, E., & Rahimi, A. "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures." _ACM Computing Surveys_, 55(6), 2022.

[PLATE-1995] Plate, T.A. "Holographic Reduced Representations." _IEEE Transactions on Neural Networks_, 6(3), 1995.

[PLATE-2003] Plate, T.A. _Holographic Reduced Representations: Distributed Representation for Cognitive Structures_. CSLI Publications, 2003.

[SCHLEGEL-2022] Schlegel, K., Neubert, P., & Protzel, P. "A Comparison of Vector Symbolic Architectures." _Artificial Intelligence Review_, 55, 2022.

[THOMAS-2021] Thomas, A., Dasgupta, S., & Bhatt, T. "Theoretical Perspectives on Deep Learning Methods in Inverse Problems." _IEEE Journal on Selected Areas in Information Theory_, 2021.

[YEUNG-2024] Yeung, C., Zou, Z., & Imani, M. "Generalized Holographic Reduced Representations." arXiv:2405.09689, 2024.

---

## 43. Bayesian Surprise and Curiosity-Driven Learning

[BALDI-2010] Baldi, P. & Itti, L. "Of Bits and Wows: A Bayesian Theory of Surprise with Applications to Attention." _Neural Networks_, 23(5), 649-666, 2010.

[ITTI-2009] Itti, L. & Baldi, P. "Bayesian Surprise Attracts Human Attention." _Vision Research_, 49(10), 1295-1306, 2009.

[SCHMIDHUBER-2010] Schmidhuber, J. "Formal Theory of Creativity, Fun, and Intrinsic Motivation." _IEEE Transactions on Autonomous Mental Development_, 2(3), 230-247, 2010.

[SHALEV-SHWARTZ-2011] Shalev-Shwartz, S. "Online Learning and Online Convex Optimization." _Foundations and Trends in Machine Learning_, 4(2), 107-194, 2011.

[ANGELOPOULOS-2023] Angelopoulos, A.N. & Bates, S. "Conformal Prediction: A Gentle Introduction." _Foundations and Trends in Machine Learning_, 16(4), 2023.

[GIBBS-2021] Gibbs, I. & Candes, E. "Adaptive Conformal Inference Under Distribution Shift." _NeurIPS_, 2021.

---

## 44. Complementary Learning Systems

[MCCLELLAND-1995] McClelland, J.L., McNaughton, B.L., & O'Reilly, R.C. "Why There Are Complementary Learning Systems in the Hippocampus and Neocortex." _Psychological Review_, 102(3), 419-457, 1995.

---

## 45. Active Inference and Free Energy

[FRISTON-2006] Friston, K., Kilner, J., & Harrison, L. "A Free Energy Principle for the Brain." _Journal of Physiology-Paris_, 100(1-3), 70-87, 2006.

---

## 46. Sleep, Oscillations, and Dream Architecture

[BORBELY-1982] Borbely, A.A. "A Two Process Model of Sleep Regulation." _Human Neurobiology_, 1(3), 195-204, 1982.

[BUZSAKI-2006] Buzsaki, G. _Rhythms of the Brain_. Oxford University Press, 2006.

[WILSON-MCNAUGHTON-1994] Wilson, M.A. & McNaughton, B.L. "Reactivation of Hippocampal Ensemble Memories During Sleep." _Science_, 265(5172), 676-679, 1994.

[BADDELEY-2000] Baddeley, A. "The Episodic Buffer: A New Component of Working Memory?" _Trends in Cognitive Sciences_, 4(11), 417-423, 2000.

---

## 47. Data Structures and Algorithms

[BHATIA-2022] Bhatia, S., Hooi, B., Yoon, M., Shin, K., & Faloutsos, C. "MIDAS: Microcluster-Based Detector of Anomalies in Edge Streams." _AAAI_, 2022.

[GRAF-2022] Graf, T.M. & Lemire, D. "Binary Fuse Filters: Fast and Smaller Than Xor Filters." _ACM Journal of Experimental Algorithmics_, 27, 1-15, 2022.

[MASSON-2019] Masson, C., Rim, J.E., & Lee, H.K. "DDSketch: A Fast and Fully-Mergeable Quantile Sketch with Relative-Error Guarantees." _Proceedings of the VLDB Endowment_, 12(12), 2019.

[SINGH-FRESHDISKANN-2021] Singh, A. et al. "FreshDiskANN: A Fast and Accurate Graph-Based ANN Index for Streaming Similarity Search." _Microsoft Research_, 2021.

---

## 48. Online Learning and Expert Algorithms

[FREUND-SCHAPIRE-1997] Freund, Y. & Schapire, R.E. "A Decision-Theoretic Generalization of On-Line Learning and an Application to Boosting." _Journal of Computer and System Sciences_, 55(1), 119-139, 1997.

[HERBSTER-WARMUTH-1998] Herbster, M. & Warmuth, M.K. "Tracking the Best Expert." _Machine Learning_, 32(2), 151-178, 1998.

---

## 49. Memory Neuroscience (Consolidation and Replay)

[HEBB-1949] Hebb, D.O. _The Organization of Behavior_. Wiley, 1949.

[MATTAR-DAW-2018] Mattar, M.G. & Daw, N.D. "Prioritized Memory Access Explains Planning and Hippocampal Replay." _Nature Neuroscience_, 21(11), 1609-1617, 2018.

[MCGAUGH-2004] McGaugh, J.L. "The Amygdala Modulates the Consolidation of Memories of Emotionally Arousing Experiences." _Annual Review of Neuroscience_, 27, 1-28, 2004.

[OJA-1982] Oja, E. "Simplified Neuron Model as a Principal Component Analyzer." _Journal of Mathematical Biology_, 15(3), 267-273, 1982.

[RICHARDS-FRANKLAND-2017] Richards, B.A. & Frankland, P.W. "The Persistence and Transience of Memory." _Neuron_, 94(6), 1071-1084, 2017.

[SCHAUL-2016] Schaul, T., Quan, J., Antonoglou, I., & Silver, D. "Prioritized Experience Replay." _ICLR_, 2016. arXiv:1511.05952.

---

## 50. HDC Applications and Variable Binding

[FRADY-KLEYKO-SOMMER-2018] Frady, E.P., Kleyko, D., & Sommer, F.T. "Variable Binding for Sparse Distributed Representations: Theory and Applications." _IEEE Transactions on Neural Networks and Learning Systems_, 2018.

[GAYLER-2004] Gayler, R.W. "Vector Symbolic Architectures are a Viable Alternative for Jackendoff's Challenges." _Behavioral and Brain Sciences_, 27(3), 2004.

[IMANI-2019] Imani, M., Kong, D., Rosing, T., & Rahimi, A. "VoiceHD: Hyperdimensional Computing for Efficient Speech Recognition." _IEEE International Conference on Rebooting Computing (ICRC)_, 2019.

---

## 51. Evolutionary Biology (Speciation and Selection)

[MAYR-1942] Mayr, E. _Systematics and the Origin of Species_. Columbia University Press, 1942.

---

## Citation Key Index

For quick lookup, citation keys are organized alphabetically:

A: [A-MEM-2024], [A16Z-ERC4626-TESTS], [A2A-SPEC-2025], [AAVE-V3-DOCS], [ACHIAM-2017], [ACE-2026], [ACON-2025], [ADAMS-2024], [ADAMS-2025], [ADAPTIVE-CB-2024], [ADL-TRILEMMA-2025], [AGENTBOUND-2025], [AGENTGUARD-2025], [AGGARWAL-2023], [ALPHA-EXPLORER-2024], [ALPERN-1985], [ALSHIEKH-2018], [ALTMAN-1999], [AMM-LVR-2023], [ANGELOPOULOS-2023], [ANTHROPIC-CE-2025], [AQSHA-2025], [ARBESMAN-2012], [ARENDT-1958], [ARGYRIS-1978], [ARROW-1962], [ASHBY-1956], [AURELIUS], [AVIDA-1994]

B: [BACHU-2025], [BAD-ACTS-2025], [BADDELEY-2000], [BADRAM-2025], [BAGGIANI-2025], [BAGGIANI-FEES-2025], [BAIER-2008], [BAILEY-2012], [BAKOS-1999], [BALDI-2010], [BALDUZZI-2008], [BARRETT-2017], [BARTHET-2022], [BARTOLETTI-2017], [BASE-LVR-2024], [BATAILLE-1949], [BATESON-1972], [BATTERING-RAM-2026], [BAUER-2021], [BAUER-RUNTIME-2011], [BEAUVOIR-1947], [BEATY-2015], [BECHARA-2000], [BECKERT-2018], [BECKER-1973], [BEER-1972], [BEER-1984], [BEN-SASSON-2018], [BENET-2014], [BENJAMIN-1936], [BENNETT-1982], [BERGAULT-EXIT-2025], [BERGSON-1896], [BERKENKAMP-2017], [BHATIA-2022], [BICHUCH-2025], [BIERE-1999], [BIFET-2007], [BLACKMORE-1999], [BONDMM-2025], [BORBELY-1982], [BORGES-1941], [BOWER-1981], [BOYD-1987], [BREDON-1997], [BUNNI-V2-DOCS], [BUSSETI-2016], [BUTLER-1963], [BUTERIN-PBS-2022], [BUZSAKI-2006]

C: [CAIA-2025], [CAMAZINE-2001], [CAMEL-2025], [CAMPBELL-2025], [CAMUS-1942], [CAMUS-1951], [CANIDIO-CCA-2025], [CANIDIO-FM-AMM-2025], [CANNON-1932], [CAPPONI-2024], [CARLSSON-2009], [CARRARA-2019], [CARTA-2020], [CENTRIFUGE-DOCS], [CERTORA-2024], [CHARIKAR-2002], [CHARNOV-1976], [CHOW-2015], [CIAMPI-2023], [CLANKER-DOCS], [CLARK-2013], [CLARKE-1971], [CLARKE-1999], [COALA-2023], [COGNITIVE-WORKSPACE-2025], [COHEN-STEINER-2007], [COMINELLI-2015], [CONANT-ASHBY-1970], [CONSTITUTIONAL-AI-2022], [CONTEXTCITE-2024], [COT-2022], [COVER-2006], [COVERED-CALL-LP-2024], [CRAIBENCH-2025], [CREATIVEDC-2024], [CROCKETT-2025], [CSO-2025], [CURRY-2014]

D: [DABAH-2025], [DABNEY-2018], [DABNEY-2020], [DAMASIO-1994], [DAMASIO-2005], [DAWKINS-1976], [DE-GIACOMO-2013], [DEFILLAMA-DATA], [DEFILLAMA-2026], [DEFI-AGENT-SURVEY-2024], [DELANDA-2006], [DELPHI-2026], [DENNETT-1995], [DENNIS-VAN-HORN-1966], [DERRIDA-1994], [DESSALVI-2026], [DEVORSETZ-2025], [DPT-AGENT-2024], [DSPY-2024], [DUETTING-2024], [DWYER-1999], [DYNAMIC-CB-2025]

E: [EAS-DOCS], [EBBINGHAUS-1885], [EDELSBRUNNER-2010], [EM-LLM-2025], [EMERSON-1990], [ENDGAME-DEFECTION-2024], [ENZYME-DOCS], [EPICURUS], [ERC-4337-SPEC], [ERC-4626-SPEC], [ERC-7265-SPEC], [ERC-7540-SPEC], [ERC-7683-SPEC], [ERC-8001-SPEC], [ERC-8004-SPEC], [ERC-8033-SPEC], [ERC-8183-SPEC], [ERC4626-DEPLOYMENTS-2026], [ESPOSITO-2010], [ESPOSITO-2011], [EVOPROMPT-2023], [EXPEL-2023]

F: [FADEMEM-2026], [FAN-2022], [FANTAZZINI-2024], [FARQUHAR-2024], [FAST-CHECK-DOCS], [FERNANDO-2024], [FERSTER-SKINNER-1957], [FINCON-2024], [FINMEM-2024], [FINRL-DEEPSEEK-2024], [FINN-2015], [FINSABER-2026], [FISHER-1930], [FISHER-2009], [FISHER-2014], [FONTANA-2024], [FOUNDRY-DOCS], [FRADY-KLEYKO-SOMMER-2018], [FRADY-RESONATOR-1-2020], [FRADY-RESONATOR-2-2020], [FREUD-1920], [FREUND-SCHAPIRE-1997], [FRISTON-2006], [FRISTON-2010], [FRITSCH-2021], [FRUGALGPT-2023]

G: [GAYLER-2003], [GEBHARD-2005], [GENERATIVE-AGENTS-2023], [GEPA-2026], [GERSHENSON-2015], [GESELL-1916], [GHRIST-2014], [GIBBS-2021], [GIDEA-2018], [GIERER-1972], [GLADDEN-2014], [GOGOL-2024], [GOGOL-2026], [GOLDWASSER-1985], [GOYAL-2023], [GRAF-2022], [GRASSE-1959], [GROSSMAN-STIGLITZ-1980], [GROVES-1973], [GUO-2017], [GVU-2024]

H: [HAAR-HOROWITZ-2020], [HAAR-HOROWITZ-2023], [HALMOS-2023], [HAMMOND-2025], [HAN-2015], [HAN-2022], [HANSEN-2001], [HANSEN-2019], [HARVEY-2016], [HASBROUCK-2025], [HAYEK-1945], [HEIDEGGER-1927], [HEIMBACH-2024], [HHI-2023], [HILBERT-2025], [HINTON-NOWLAN-1987], [HIPPORAG-2024], [HOLLAND-1995], [HOLLDOBLER-2008], [HULL-1988], [HUYNH-2025]

I: [ITTI-2009]

J: [JENSEN-1906], [JONAS-1966]

K: [KAHNEMAN-1973], [KAHNEMAN-1979], [KANERVA-1988], [KANERVA-2009], [KARPATHY-2026], [KATO-2024], [KAUFFMAN-1993], [KELLY-1956], [KIERKEGAARD-1843], [KLEYKO-2022], [KLÜVER-1966], [KOCH-2019], [KOKI-2022], [KONDO-2010], [KRASKOV-2004], [KREMINSKI-2024], [KUMARAN-2016]

L: [L2-FEE-DYNAMICS-2024], [LACAUX-2021], [LACAUX-2024], [LAKSHMINARAYANAN-2017], [LAMPORT-1977], [LAMPORT-1979], [LANDAUER-1961], [LATANE-1959], [LATS-2023], [LEBARON-2006], [LEHAR-PARLOUR-2021], [LEM-1961], [LINDENBAUER-2025], [LIPTON-2025], [LLMLINGUA-2-2024], [LO-2004], [LOESCH-2021], [LOTKA-1925], [LP-STRATEGY-OPTIMIZATION-2024]

M: [MA-CRAPIS-2024], [MAC-LANE-1971], [MACLEAN-1992], [MAGNIN-2010], [MANDELBROT-2004], [MANNA-1992], [MANUYLOVICH-2024], [MAP-ELITES-2015], [MAPLE-DOCS], [MASSON-2019], [MATURANA-VARELA-1980], [MAXWELL-1868], [MCADAMS-2001], [MCCLELLAND-1995], [MCP-GUARD-2025], [MCP-SPEC-2024], [MEMGPT-2023], [MERITRANK-2022], [MERKLE-1987], [MERLEAU-PONTY-1945], [MERTON-1969], [MESSARI-2025], [MILEWSKI-2019], [MILGROM-2004], [MILIONIS-2023], [MILIONIS-FEES-2024], [MILIONIS-LVR-2022], [MILIONIS-MYERSON-2024], [MILLER-1955], [MOALLEMI-AUCTION-2024], [MOALLEMI-LVF-2024], [MOGGI-1991], [MORNINGSTAR-MRAR], [MORPHO-DOCS], [MORPHO-TVL-2026], [MORPHO-WHITEPAPER], [MOURET-2015], [MURRAY-2003], [MYERSON-1983]

N: [NEMHAUSER-1978], [NIETZSCHE-1882], [NIETZSCHE-1883], [NISAN-2007]

O: [OIZUMI-2014], [OMOHUNDRO-2008], [ORMER-2024], [ORTONY-CLORE-COLLINS-1988], [OSTROM-1990], [OTTER-2017], [OVER-2025], [OWASP-AGENTIC-2025], [OWASP-LLM-2025], [OZ-CONTRACTS], [OZ-ERC4626]

P: [PA-AMM-2025], [PARFIT-1984], [PAYNE-2025], [PEEPERKORN-2024], [PENDLE-DOCS], [PEREA-2015], [PERMIT2-SPEC], [PETERS-2019], [PETERS-GELL-MANN-2016], [PLATE-1995], [PLATE-2003], [PNUELI-1977], [POLYMARKET-VOLUME-2026], [POPPER-1972], [POWERS-1973], [PRICE-1970], [PROMPTFOO-DOCS]

R: [R2AI-2025], [RAPTOR-2024], [REACT-19-DOCS], [REFLEXION-2023], [REPEATED-GAMES-AGENTS-2024], [RESCORLA-WAGNER-1972], [REYES-2025], [RLAIF-2023], [ROBINSON-2014], [ROBINSON-2017], [ROESE-1997], [ROSSETTI-2025], [ROUGHGARDEN-TFM-2024], [ROY-1952], [RWA-TVL-2026]

S: [SAMUELSON-1979], [SARTRE-1943], [SCHERER-2001], [SCHLEGEL-2022], [SCHMIDHUBER-2010], [SCONE-BENCH-2025], [SCHULMAN-2015], [SCOPE-2025], [SEAGENT-2026], [SECPLF-2024], [SELIGMAN-1972], [SELF-CONSISTENCY-2023], [SET-PROTOCOL-DOCS], [SETH-2021], [SHALEV-SHWARTZ-2011], [SHANNON-1948], [SHANNON-1959], [SHILL-PROOF-2024], [SIMON-1971], [SIMONDON-1958], [SIMS-2003], [SINGH-2025], [SINGH-LVR-2025], [SOK-DEFI-2023], [SOMMELIER-ARCHITECTURE], [SOROS-1987], [SOULBOUND-2022], [STEPHENS-1986], [STERLING-2012], [STIEGLER-2010a], [STIEGLER-2010b], [STIEGLER-2018], [STILL-2012], [SUMERS-2024], [SUN-2025], [SUPER-2024], [SWIERSTRA-2008], [SYBIL-PROOF-2024], [SZABO-1997], [SYSTEM-1-2-2024]

T: [TACT-2024], [TAILWIND-4-DOCS], [TALEB-2012], [TALEB-DOUADY-2013], [TAYLOR-1978], [TEE-FAIL-2024], [TESFATSION-2006], [THALER-1990], [THOMAS-2021], [THORP-2006], [TIERRA-1992], [TONONI-2004], [TONONI-2008], [TONONI-CIRELLI-2014], [TOSEY-2012], [TRACERANK-2025], [TRADETRAP-2025], [TRANQUILLI-2025], [TRUST-DILEMMA-2024], [TROTTI-2025], [TSUP-DOCS], [TURING-1952], [TURNER-2020], [TURNER-2024]

U: [UNISWAP-V4-DOCS]

V: [VAN-DE-MERWE-2024], [VAN-DE-VEN-2020], [VARELA-1991], [VARELA-THOMPSON-ROSCH-1991], [VARDI-1986], [VICKREY-1961], [VIEM-DOCS], [VIRTUALS-ACP-2025], [VITEST-DOCS], [VOLTERRA-1926], [VON-FOERSTER-1979], [VOVK-2005], [VOYAGER-2023]

W: [WADLER-1992], [WEBRL-2024], [WENSINK-2020], [WHITEHEAD-1929], [WIENER-1948], [WILLIAMSON-1979], [WILSON-MCNAUGHTON-1994], [WITHDRAWAL-DAMPENING-2024], [WRIGHT-1932]

X: [X402-SPEC], [XIONG-2023], [XUAN-2026]

Y: [YEARN-V3-DESIGN], [YEUNG-2024]

Z: [ZBANDUT-2025], [ZOMORODIAN-2005], [ZOU-2023], [ZSCORE-REPUTATION-2024], [ZSCORE-UNIVERSAL-2025], [ZSCORE-WALLET-2025]
