# PRD Revision Guide: Making Golems Actually Work (v3 — Final) [SPEC]

> **Purpose**: Specific, actionable revisions to the Bardo PRD suite — grounded in academic research on autonomous LLM agent evaluation, memory quality, reflection loop validation, and strategy execution verification.
>
> **Hard constraint**: Zero external service dependencies. A live production Golem on Base mainnet has access to: its RPC endpoint, `alloy` (EVM client in the `golem-chain` Rust crate), on-chain state, its local Grimoire (LanceDB + SQLite via `golem-grimoire`), and the LLM via the `golem-inference` crate. Nothing else. Anvil/TEVM fork simulation is available during development and testing phases only — never assumed in production.
>
> **How to use**: Hand this document (or individual sections) to Claude alongside the relevant PRD files. Each revision is tagged with the file it modifies and the section within that file.

> **Reader orientation:** This document is a concrete revision guide for the Bardo PRD suite, grounded in academic research on autonomous LLM agent evaluation, memory quality, and reflection loop validation. It belongs to Section 16 (Testing) and specifies 12 numbered revisions to anchor the Golem's (mortal autonomous agent's) self-correction loops in external, on-chain feedback signals rather than LLM self-evaluation. The central insight: the blockchain is the ground truth oracle. The Grimoire (the agent's persistent knowledge base) admission gate, Ebbinghaus forgetting, and regime-tagged evaluation are all specified here. See `prd2/shared/glossary.md` for full term definitions.

---

## The Core Problem

The Bardo architecture is built on a foundation that the research literature has shown to be unreliable: **LLMs cannot self-correct reasoning without external feedback** (Huang et al., "Large Language Models Cannot Self-Correct Reasoning Yet," ICLR 2024, arXiv:2310.01798). Intrinsic self-correction — where the same model evaluates its own outputs — consistently degrades performance on reasoning tasks. This has been replicated across GPT-3.5, GPT-4, and Claude models.

The current PRDs describe a triple-loop cybernetic architecture where the Golem reflects on its own outputs, evaluates its own reasoning, curates its own memory, and judges its own improvement. Each of these self-referential loops is vulnerable to convergence on plausible-sounding noise.

The architecture is not fundamentally broken — it is fundamentally incomplete. The triple-loop structure provides the right scaffolding, but each loop needs **external anchoring**. In production, "external" means on-chain state and standard JSON-RPC methods — the blockchain itself is the oracle of ground truth.

---

## What a Production Golem Actually Has

Before any revision, establish the production verification primitives. The Golem already depends on `viem` and an RPC endpoint to interact with Base. These provide:

| Primitive                 | Method                                 | Cost                         | What it tells you                                                     |
| ------------------------- | -------------------------------------- | ---------------------------- | --------------------------------------------------------------------- |
| **Pre-tx simulation**     | `publicClient.simulateContract()`      | Free (uses `eth_call`)       | Will this tx succeed or revert? What's the return value?              |
| **Multi-call simulation** | `publicClient.simulateCalls()`         | Free (uses `eth_simulateV1`) | Will this sequence of txs succeed? State carries across calls.        |
| **Gas estimation**        | `publicClient.estimateContractGas()`   | Free                         | How much gas will this cost?                                          |
| **State reads**           | `publicClient.readContract()`          | Free                         | What is the current on-chain state? (Balances, positions, pool state) |
| **Tx receipts**           | `publicClient.getTransactionReceipt()` | Free                         | Did the tx succeed? What events were emitted? How much gas was used?  |
| **Event logs**            | `publicClient.getLogs()`               | Free                         | What events occurred in a block range?                                |

These are standard JSON-RPC methods supported by every Base RPC provider. No special infrastructure. No external services. The Golem already has the client — verification is just reading more from it.

**The blockchain is the ground truth.** Pre-tx, `simulateContract()` tells you if the action will work. Post-tx, the receipt + state reads tell you what actually happened. The delta between prediction and reality is the external feedback signal that makes Reflexion work.

---

## Revision 1: Ground the Inner Loop in On-Chain Verification

**Modifies**: `prd2-golem.md` — Heartbeat (REFLECTING state); `prd2-safety.md` — Section 7

**The problem**: The REFLECTING state performs "ground-truth backcheck" using the same LLM that made the original decision. Reflexion (Shinn et al., NeurIPS 2023, arXiv:2303.11366) achieved its gains on tasks with **external verification signals**. A replication study (arXiv:2512.20845) observed **degeneration-of-thought** without them. Kamoi et al. (_TACL_, 12:1417-1440, 2024, arXiv:2406.01297) found **no prior work demonstrates successful self-correction with feedback from prompted LLMs**.

### What to change in `prd2-golem.md` — REFLECTING state

**Replace with**:

> **REFLECTING.** Five-step cascade. Steps 1-2 produce external ground truth at $0.00. Steps 3-5 interpret that truth through the LLM.
>
> 1. **Outcome Verification (on-chain, deterministic, $0.00).** Before execution, the `bardo-verifier` extension snapshots relevant state via `readContract()` — token balances, pool reserves, position details. After execution, it reads the transaction receipt and re-reads the same state. It produces a structured `OutcomeVerification` comparing predicted vs. actual:
>
>    ```typescript
>    interface OutcomeVerification {
>      tickNumber: number;
>      actionId: string;
>      preState: {
>        balances: Record<string, bigint>;
>        positions: Record<string, any>;
>        poolState: Record<string, any>;
>      };
>      prediction: {
>        // From simulateContract() before execution
>        expectedReturn: any;
>        expectedGas: bigint;
>        wouldRevert: boolean;
>      };
>      postState: {
>        balances: Record<string, bigint>;
>        positions: Record<string, any>;
>        poolState: Record<string, any>;
>        txReceipt: {
>          status: "success" | "reverted";
>          gasUsed: bigint;
>          logs: Log[];
>        };
>      };
>      deviations: {
>        balanceChangeBps: Record<string, number>;
>        gasDeviationBps: number;
>        unexpectedLogs: Log[];
>        missingExpectedLogs: string[];
>      };
>    }
>    ```
>
>    This is the ground truth. Not the LLM's interpretation — the blockchain's.
>
> 2. **Invariant Checking (deterministic, $0.00).** Validate that all invariants hold after execution: token amount bounds, balance change limits within PolicyCage constraints, no unexpected contract interactions. Any violation → `safety_incident` Episode at confidence 1.0. This is TypeScript, not LLM inference. Based on Trace2Inv invariant templates (Chen et al., FSE 2024): 23 templates across 8 categories, neutralizing 74.1%+ of attacks at <0.32% false positive rate.
> 3. **Outcome Comparison (LLM, secondary).** The LLM receives the `OutcomeVerification` record and interprets it. Its role is _making sense of_ the deviation, not _determining whether_ there was one. "The swap returned 15bps less than `simulateContract()` predicted — was liquidity thinner than the snapshot showed, or did the price move between simulation and execution?"
> 4. **Counterfactual Analysis (LLM).** "What would have happened with the opposite action?" For the production Golem, counterfactuals are LLM reasoning grounded in the `OutcomeVerification` data — not simulated alternatives. The LLM extrapolates from known deviations, not from hypothetical simulations.
> 5. **Insight Extraction (LLM, quality-gated).** If the reflection reveals a pattern, extract as a candidate Insight at confidence 0.5. Must pass the Grimoire Admission Gate (Revision 3) before entering long-term memory.

### What to change in `prd2-safety.md` — Section 7.1

The existing three-stage pipeline references "TEVM Fork Simulation" at Stage 2. In production, this should use `simulateContract()` which is available against any RPC:

> **Stage 2: Pre-Execution Simulation (milliseconds, free)**
>
> `publicClient.simulateContract()` (uses standard `eth_call` against the production RPC) verifies the transaction would succeed and returns the expected result. For multi-step strategies, `publicClient.simulateCalls()` (uses `eth_simulateV1`) simulates sequential calls with state carrying across steps. Both are standard JSON-RPC methods — no special infrastructure.
>
> If simulation shows the tx would revert → block execution, log the revert reason.
> If simulation succeeds but the return value deviates from World Model prediction → flag for review.
>
> **Note for development/testing**: Anvil fork with full state override capabilities is available during Phases 1-2 of the Evaluation Lifecycle (see Revision 6). Production Golems use `eth_call`/`eth_simulateV1` only.

Add Stage 4:

> **Stage 4: Post-Execution Verification (milliseconds, free)**
>
> After on-chain execution, re-read affected state via `readContract()` and retrieve the tx receipt. Compare actual outcome against Stage 2 simulation. Compute deviations. Feed the structured `OutcomeVerification` record to the REFLECTING state. This closes the predict→execute→verify loop.

### Integration with Dream Engine

The `OutcomeVerification` records from waking execution become dream material. During Phase 1 NREM replay, the Dream Engine can access the actual deviation data for replayed episodes. Add to `../05-dreams/02-replay.md`:

> **Deviation-anchored replay**: When replaying episodes that have `OutcomeVerification` records, include the deviation data as context. "This trade deviated 15bps from `simulateContract()` prediction — what would have happened at 50bps, 100bps, 200bps deviation?" This produces perturbation scenarios calibrated to real-world deviation magnitudes rather than arbitrary LLM guesses.

---

## Revision 2: Separate Generator and Evaluator

**Modifies**: `prd2-golem.md` — Learning Pipeline; `prd2-inference.md`

**The problem**: Pan et al. (ICML 2024, arXiv:2402.06627) proved that shared context between generator and evaluator causes **in-context reward hacking**. Scaling model size makes it worse. Pan et al. (arXiv:2407.04549) showed **human quality scores decrease** in later iterations while LLM evaluator scores rise. Anthropic's research (arXiv:2505.05410, 2025) found Claude mentioned its actual reasoning only **25% of the time**.

### What to add to `prd2-golem.md` — Learning Pipeline

> ### Evaluation Separation Principle
>
> Three mechanisms prevent the evaluator from exploiting the generator's shortcuts:
>
> **1. Context isolation.** The evaluator receives a fresh context window: the `OutcomeVerification` record, the action taken, and the generator's stated reasoning. It does NOT receive the full DECIDING dialogue history.
>
> **2. Model diversity (when budget allows).** Different model for evaluation than generation. Generator used Sonnet → evaluator uses Haiku. Different biases prevent shared failure modes.
>
> **3. External metric anchoring.** Evaluator assessment cross-checked against on-chain quantitative metrics:
>
> | Metric                        | Source                                  | What it catches                                 |
> | ----------------------------- | --------------------------------------- | ----------------------------------------------- |
> | Actual PnL (bps)              | Balance change from OutcomeVerification | "Good reasoning" that loses money               |
> | Sharpe ratio (rolling 7d)     | Computed from Episode history           | "Consistent strategy" with inconsistent returns |
> | Max drawdown (rolling 7d)     | Computed from Episode history           | "Risk-aware reasoning" that doesn't manage risk |
> | Slippage vs. simulation (bps) | OutcomeVerification                     | "Improved execution" that's actually worse      |
> | Gas efficiency                | OutcomeVerification                     | "Cost optimization" that wastes gas             |
>
> If the evaluator says "high quality" but external metrics show decline, the external metrics win. On-chain state is the court of last resort.

### What to add to `prd2-inference.md`

> | Evaluation / reflection | Different model from generator | None | ~$0.001-0.01 |
>
> **Evaluator routing rule**: The evaluator model should differ from the generator model. If credit constraints force the same model, the evaluator MUST use a fresh context window. Hard architectural requirement — self-evaluation with shared context produces reward hacking (Pan et al., ICML 2024).

---

## Revision 3: Quality-Gated Memory Admission (Grimoire Admission Gate)

**Modifies**: `prd2-golem.md` — Mind section; `memory-prd.md`

**The problem**: No admission control. Errors in stored memories propagate to similar future tasks (arXiv:2505.16067). A selective policy produces **10% absolute gains** over unfiltered storage. RAG quality degrades as knowledge bases grow (Cuconasu et al., SIGIR 2024, arXiv:2401.14887).

### What to add to `prd2-golem.md` — Mind section

> ### Grimoire Admission Gate
>
> Every candidate entry passes through the Admission Gate before Grimoire write. Implements A-MAC five-factor scoring (Zhang et al., arXiv:2603.04549, March 2026):
>
> | Factor                 | Weight | How computed                                                                                           | Threshold     |
> | ---------------------- | ------ | ------------------------------------------------------------------------------------------------------ | ------------- |
> | **Future utility**     | 0.25   | Single Haiku call: "Will this be useful for future decisions in similar conditions?" Returns 0.0-1.0.  | > 0.4         |
> | **Factual confidence** | 0.25   | Cross-reference against existing Grimoire. Contradicts high-confidence entries → flag. Aligns → boost. | > 0.3         |
> | **Semantic novelty**   | 0.20   | LanceDB similarity search. Cosine > 0.9 → MERGE. > 0.95 → SKIP. < 0.5 → flag off-topic.                | 0.5-0.9 range |
> | **Temporal recency**   | 0.15   | Exponential decay from the event described.                                                            | > 0.2         |
> | **Content type prior** | 0.15   | Calibrated per entry type (most influential factor in A-MAC ablations).                                | Per-type      |
>
> **Content type priors**:
>
> | Entry Type              | Prior | Rationale                                         |
> | ----------------------- | ----- | ------------------------------------------------- |
> | Warning                 | 0.9   | Safety-critical; false negative >> false positive |
> | Causal Link             | 0.7   | Structural; high reuse                            |
> | Heuristic               | 0.6   | Actionable; needs validation                      |
> | Insight                 | 0.5   | Descriptive; moderate reuse                       |
> | Strategy Fragment       | 0.4   | Context-dependent                                 |
> | Observation (ephemeral) | 0.2   | Low durability; high volume                       |
>
> Composite score below 0.45 → rejected. 0.45-0.55 → admitted at confidence 0.3. Above 0.55 → standard confidence.
>
> **Hallucination firewall**: Factual confidence < 0.3 AND contradicts high-confidence existing entries → quarantined in `quarantined_entries` table, reviewed next Curator cycle.
>
> **Cost**: ~$0.001 per candidate (one Haiku call). At ~5-10 candidates per non-suppressed tick, ~$0.005-0.01 per reflection cycle. Expected to filter 40-60% of candidates.

### Integration with Dream Engine

Dream outputs face two quality gates in series:

1. **Dream staging buffer** (`../05-dreams/04-consolidation.md`): validates the hypothesis is plausible and testable.
2. **Grimoire Admission Gate**: validates the promoted hypothesis is novel, useful, and factually consistent.

Add to `../05-dreams/04-consolidation.md`:

> Staged revisions reaching `validated` status (confidence >= 0.7) still pass through the Grimoire Admission Gate before becoming operational entries. The staging buffer confirms live validation; the Admission Gate confirms novelty and consistency. Both gates are necessary — validated but redundant entries waste retrieval bandwidth.

### What to add to `memory-prd.md` — The Bridge

Add "Quality Gate" column:

> | What Crosses         | Quality Gate                                                     |
> | -------------------- | ---------------------------------------------------------------- |
> | Promoted insights    | Admission Gate score > 0.55                                      |
> | Validated heuristics | Active 10+ ticks with positive external metrics                  |
> | Warnings             | Bypass gate (safety-critical), semantic novelty check only       |
> | Regime shifts        | Bypass gate (environmental signal), cross-referenced with Oracle |
> | Death reflection     | Bypass gate (one-time, high-value)                               |

---

## Revision 4: Ebbinghaus Forgetting with Retrieval Strengthening

**Modifies**: `prd2-golem.md` — Confidence and Decay; `memory-prd.md`

**The problem**: Current decay model doesn't account for retrieval strengthening. MemoryBank (Zhong et al., AAAI 2024, arXiv:2305.10250) shows Ebbinghaus + retrieval strengthening outperforms simple time-based decay. The testing effect (Roediger & Karpicke, 2006) — already cited in `memory-prd.md` — justifies it but no concrete mechanism exists.

### Replace decay formula in `prd2-golem.md`

> Each GrimoireEntry carries three lifecycle fields:
>
> | Field          | Type          | Default        | Purpose                             |
> | -------------- | ------------- | -------------- | ----------------------------------- |
> | `confidence`   | float 0.0-1.0 | Per provenance | Evidential quality                  |
> | `strength`     | integer >= 1  | 1              | Successful retrieval count          |
> | `lastAccessed` | timestamp     | Creation time  | Last retrieval for DECIDING context |
>
> ```
> retention(t) = e^(-(t - lastAccessed) / (halfLife × strength))
> effective_confidence(t) = confidence × retention(t)
> ```
>
> `strength` increments when an entry is retrieved AND the tick had a positive outcome (PnL > 0 or risk metric improved). Entry retrieved 5 times with positive outcomes → `strength = 6`, decays 6× slower.
>
> **Strength does NOT increment on mere retrieval** — only retrieval + positive outcome. Prevents self-referential gaming.
>
> **Dream-retrieval strengthening**: When an entry is retrieved during NREM replay and the dream analysis produces a validated pattern, increment `strength` by 0.5 (reduced rate — dream validation is weaker than live-market confirmation). Implements Wilson & McNaughton (1994): sleep replay strengthens memory traces.
>
> **Floor**: `effective_confidence` at 0.05. **Pruning**: Below 0.1 for 3+ consecutive Curator cycles → archived to cold storage.

```sql
ALTER TABLE grimoire_entries ADD COLUMN strength INTEGER DEFAULT 1;
ALTER TABLE grimoire_entries ADD COLUMN last_accessed INTEGER;
ALTER TABLE grimoire_entries ADD COLUMN consecutive_low_confidence INTEGER DEFAULT 0;
```

---

## Revision 5: Detect and Prevent Vacuous Reasoning

**Modifies**: `prd2-golem.md` — Mind section

**The problem**: Liang et al. (arXiv:2507.07484, 2025) found RLHF **increases** the Bullshit Index. Chain-of-thought prompting increases empty rhetoric. Models with 46% accuracy express 76% mean confidence (JMIR Medical Informatics, 2025).

### What to add — Artifact Quality Scoring

> Every cognitive artifact receives a composite quality score at creation time.
>
> | Dimension         | What it measures                       | How scored                                                      | Weight |
> | ----------------- | -------------------------------------- | --------------------------------------------------------------- | ------ |
> | **Specificity**   | Concrete claims vs. vague generalities | Regex: count numbers, addresses, timestamps vs. hedging phrases | 0.25   |
> | **Actionability** | Leads to concrete next action?         | IF-THEN structure or specific trigger condition present?        | 0.25   |
> | **Novelty**       | Adds beyond existing knowledge?        | LanceDB similarity against Grimoire                             | 0.20   |
> | **Verifiability** | Checkable against external data?       | References on-chain data, block numbers, tx hashes?             | 0.15   |
> | **Consistency**   | Regeneration produces similar content  | For high-stakes: regenerate 3×, measure embedding similarity    | 0.15   |
>
> **Red flags** (-0.1 each): empty rhetoric, weasel words, unverified causal claims, self-referential praise, tautological content.
>
> **Implementation**: TypeScript function for rule-based dimensions. Novelty via LanceDB. Consistency check only for Curator-cycle promotions (expensive).
>
> **Retrieval**: `final_score = effective_confidence × quality_score × relevance_similarity`. Low-quality entries deprioritized even if semantically relevant.
>
> **Decay**: quality_score < 0.3 → decays 2× faster than decay class dictates.

### Dream-specific calibration

Dream outputs are by design more exploratory. Score differently:

- **Reduced specificity weight** (0.10 instead of 0.25) for dream hypotheses — they're intentionally abstract.
- **Full specificity weight** for dream-generated threat responses and PLAYBOOK.md guards — safety outputs must be concrete.
- **Increased consistency weight** (0.25) for dream outputs — arbitrary hallucinated content is the primary risk.

---

## Revision 6: Regime-Tagged Evaluation Framework

**Modifies**: `prd2-golem.md` — Regime Detection; `prd2-dev.md`

**The problem**: No mechanism to distinguish genuine improvement from favorable market conditions. Without regime-tagged evaluation, Loop 3 is blind.

### What to add to `prd2-golem.md`

> ### Regime-Tagged Evaluation
>
> Every tick produces a `RegimeTag`:
>
> ```typescript
> interface RegimeTag {
>   volatilityQuintile: 1 | 2 | 3 | 4 | 5;
>   trendDirection: "up" | "down" | "range";
>   gasPriceLevel: "low" | "normal" | "high" | "spike";
>   liquidityCondition: "deep" | "normal" | "thin" | "crisis";
>   timestamp: number;
>   blockNumber: number;
> }
> ```
>
> Attached to every Episode, Grimoire entry, and performance snapshot.
>
> **Improvement measurement**: When Loop 2 proposes a change or the Curator promotes a heuristic:
>
> 1. Baseline: 50 ticks before, tagged.
> 2. Treatment: 50 ticks after.
> 3. Match: Compare only ticks with same volatility quintile + trend direction. Extend window if <10 matched pairs.
> 4. Effect size: Cohen's d with bootstrap 95% CIs.
> 5. Decision: d > 0.2 with CI excluding zero → validated. d ≤ 0 after 100 ticks → rolled back.
>
> **Change-point detection**: Rolling KS statistic on cumulative PnL. Significant change (p < 0.05) → regime reclassification, measurement restart.

### Integration with Dream Engine

Add to `../05-dreams/05-threats.md`:

> **Regime-aware threat scheduling**: When current RegimeTag shows `volatilityQuintile >= 4` or `liquidityCondition === 'thin'`, double Tier 1 threat rehearsal frequency in the next dream cycle. Proactive amplification based on environmental conditions, not waiting for actual loss.

### What to add to `prd2-dev.md`

> ### Evaluation Lifecycle
>
> | Phase                   | Environment                                        | Capital Risk | Gate to next                       |
> | ----------------------- | -------------------------------------------------- | ------------ | ---------------------------------- |
> | **1. Trace inspection** | Anvil fork (dev only)                              | None         | Promptfoo suites >85%              |
> | **2. Backtesting**      | Anvil fork, replayed blocks (dev only)             | None         | Sharpe > 0 across 3+ regimes       |
> | **3. Paper trading**    | Live data, `simulateContract()` only, no execution | None         | No regression vs. baseline over 7d |
> | **4. Canary**           | Live (Base mainnet)                                | 1-5% TVL     | No regression over 14d, p < 0.05   |
>
> Anvil/TEVM fork is available in Phases 1-2 only (development infrastructure). Phase 3+ uses production-only primitives: `simulateContract()`, `readContract()`, event logs.

---

## Revision 7: Episodic-to-Semantic Consolidation Pipeline

**Modifies**: `prd2-golem.md` — Curator Cycle; `memory-prd.md`

**The problem**: No explicit episodic-to-semantic consolidation. Mem0 (Chhikara et al., arXiv:2504.19413, 2025): consolidation cuts storage **60%**, raises retrieval precision **22%**.

### What to add to `prd2-golem.md`

> ### Episodic-to-Semantic Consolidation (runs during Curator cycle)
>
> Every 50 ticks:
>
> 1. **Cluster**: DBSCAN (eps=0.3) on nomic-embed-text-v1.5 embeddings of last 50 ticks' episodes.
> 2. **Summarize**: Haiku generates semantic summary for clusters with 3+ episodes.
> 3. **Deduplicate**: LanceDB similarity check. Cosine > 0.9 → merge (update confidence + evidence count).
> 4. **Resolve conflicts**: New contradicts existing → recency + confidence scoring. Higher evidence AND more recent → downvote existing, admit new.
> 5. **Prune**: Archive episodes older than 7 days to cold storage. Keep `regime_shift`, `safety_incident`, or quality_score > 0.7.

### Curator ↔ Dream coordination

> - **Curator tags ambiguous episodes for dream replay**: Episodes with contradictory patterns or insufficient evidence → `dream_priority: high`. Dream Engine's utility scheduler uses this as a boost factor.
> - **Dream insights feed back to Curator**: Staging buffer entries at `partially_validated` are flagged for the next Curator cycle. Curator can accelerate promotion if corroborating waking evidence exists.

---

## Revision 8: Embedding Drift Detection

**Modifies**: `memory-prd.md`

> ### Embedding Integrity
>
> Every vector carries metadata: `{ model, modelVersion, preprocessHash, chunkConfig, createdAt }`.
>
> **Drift detection** (weekly, during Loop 3 or Dream Integration):
>
> 20 reference documents. Weekly re-embed and measure:
>
> 1. Cosine distance stability week-to-week. Healthy: <0.02. Warning: 0.02-0.05. Critical: >0.05.
> 2. Nearest-neighbor stability: top-5 same as last week? Healthy: 85-95%. Drifting: <60%.
> 3. Vector norm variance: sudden increase → distribution shift.
>
> Critical threshold → pause writes. Migration via Drift-Adapter (Vejendla, EMNLP 2025, arXiv:2509.23471): 95-99% recall recovery. **Never partially re-embed.**

---

## Revision 9: Hybrid Retrieval Pipeline

**Modifies**: `prd2-golem.md` — Memory Architecture

**The problem**: Accuracy spans 20 points across retrieval methods but only 3-8 across write strategies (arXiv:2603.02473). Hybrid reranking cuts retrieval failures by half.

> **Hybrid Retrieval Pipeline**
>
> 1. **Dual retrieval**: Vector ANN (LanceDB) + BM25 full-text (SQLite FTS5) in parallel. Top-20 each.
> 2. **Reciprocal Rank Fusion**: `score = Σ 1/(60 + rank_i)`.
> 3. **Multi-factor reranking**:
>    ```
>    final = rrf × 0.30 + effective_confidence × 0.25 +
>            quality_score × 0.20 + recency_boost × 0.15 +
>            regime_match × 0.10
>    ```
> 4. **MMR diversity** (λ=0.7) on top-10.
> 5. **Context budget**: Top-N entries fitting `maxTokens` (default 500). More entries at shorter length > fewer at full length.

---

## Revision 10: Strategy Input Validation

**Modifies**: `prd2-golem.md` — Creation section

> ### Strategy Validation Pipeline
>
> Before execution begins:
>
> **1. Schema validation** ($0.00): Required fields, types, system limits. `maxDrawdownPct` 1-50. `approvedAssets` contains valid Base ERC-20s. `tickInterval` 15-120s.
>
> **2. Consistency checking** ($0.00): "Aggressive yield farming" + "maxDrawdownPct: 2" is contradictory. Target pool tokens not in `approvedAssets` is inconsistent.
>
> **3. Historical range checking** ($0.00): Parameters >2σ from mean for this strategy type. $100k `buyAmount` when median is $50 → likely decimal error.
>
> **4. Risk assessment** (Haiku, ~$0.003): Brief risk assessment, potential failure modes, viability given current conditions.
>
> **5. Dry-run simulation** ($0.00): `simulateContract()` / `simulateCalls()` for the first 3-5 actions against current Base state. Verify: no reverts, reasonable gas, expected balance direction, no PolicyCage violations.
>
> Critical failures block deployment. Warnings require operator acknowledgment.
>
> **Graceful degradation**: If the strategy proves unviable during execution (insufficient liquidity, untradeable token, gas exceeds returns) → `STRATEGY_UNVIABLE` state: no trades, webhook to operator, structured explanation with suggestions, waits for steer. Does NOT burn credits on futile execution.

---

## Revision 11: Dream Threat Rehearsal Grounding

**Modifies**: `../05-dreams/05-threats.md` — Threat Simulation Protocol

**The problem**: Threat simulation generates scenarios through LLM imagination. Without any grounding, rehearsed responses may be mechanically wrong — the Golem might rehearse an exit sequence that would revert on-chain.

### What to add

> ### Grounding Threat Responses in On-Chain Reality
>
> For Tier 1 (existential) threats, the ThreatSimulator should validate the _feasibility_ of rehearsed responses using `simulateContract()`, even though it cannot simulate the threat conditions themselves:
>
> 1. **Response feasibility check**: For each rehearsed response action (emergency exit, position close, rebalance), run `simulateContract()` against current Base state to verify the action would succeed _right now_. If the exit transaction would revert today, the rehearsed response is broken regardless of the threat scenario.
> 2. **Gas budget validation**: `estimateContractGas()` for the full response sequence. Compare against the Golem's gas partition. If the emergency exit sequence costs more gas than the Golem can afford, the gap is concrete and the PLAYBOOK.md guard is inadequate.
> 3. **Liquidity sanity check**: For exit-based responses, read current pool reserves via `readContract()`. If current liquidity wouldn't support the planned exit size at acceptable slippage, the rehearsed response needs revision.
>
> This doesn't simulate the threat condition itself (you'd need state overrides for that, which require Anvil). But it validates that the _response actions are executable today_. A rehearsed response that would revert under normal conditions will certainly fail under crisis conditions.
>
> **When full state-override simulation is desired** (development only): During Evaluation Lifecycle Phases 1-2, the operator can run threat rehearsals against Anvil forks with injected crisis conditions (oracle price overrides, liquidity removal, gas spike simulation). Results from these dev-phase rehearsals can be stored as `ThreatRehearsalResult` entries in the Grimoire and inherited by production Golems.
>
> | Validation Type                 | Production                         | Dev/Testing                            |
> | ------------------------------- | ---------------------------------- | -------------------------------------- |
> | Response action would succeed   | `simulateContract()`               | Anvil fork + state overrides           |
> | Gas budget sufficient           | `estimateContractGas()`            | Anvil fork with gas price overrides    |
> | Liquidity adequate for exit     | `readContract()` pool reserves     | Anvil fork with liquidity removal      |
> | Full threat scenario end-to-end | Not available (LLM reasoning only) | Anvil fork with full crisis simulation |

---

## Revision 12: Cognitive Quality Dashboards

**Modifies**: `prd2-golem.md` — Observability section

> ### Cognitive Quality Metrics
>
> | Metric                         | Computation                                                  | Healthy Range        | Alarm              |
> | ------------------------------ | ------------------------------------------------------------ | -------------------- | ------------------ |
> | **Admission rate**             | % candidates passing Gate                                    | 40-60%               | <20% or >80%       |
> | **Average quality score** (7d) | Mean of admitted entries                                     | 0.5-0.8              | <0.4 or declining  |
> | **Grimoire size**              | Active entry count                                           | Slow growth, plateau | Unbounded growth   |
> | **Retrieval hit rate**         | % DECIDING ticks referencing retrieved entries               | >30% after 7d        | <10%               |
> | **Heuristic survival rate**    | % promoted heuristics active after 100 ticks                 | 40-70%               | <20% or >90%       |
> | **External metric trend**      | Sharpe, drawdown, PnL (30d rolling)                          | Improving or stable  | Declining 14+ days |
> | **Reflection consistency**     | Cosine across regenerated reflections (weekly)               | >0.7                 | <0.5               |
> | **DecisionCache hit rate**     | % T2-eligible ticks from cache                               | >30% after 7d        | <10% after 14d     |
> | **Dream yield**                | % staged revisions reaching `validated`                      | 10-30%               | <5% or >50%        |
> | **Threat coverage**            | % Tier 1 threats rehearsed in last 7 dream cycles            | 100%                 | <100%              |
> | **Prediction accuracy**        | % of `simulateContract()` predictions within 50bps of actual | >90%                 | <80%               |

---

## Summary of Changes by PRD File

| PRD File                            | Revisions                                                                                                 | Priority     |
| ----------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------ |
| `prd2-golem.md` — REFLECTING        | #1: On-chain verification as primary signal                                                               | **Critical** |
| `prd2-golem.md` — Learning Pipeline | #2: Generator/evaluator separation                                                                        | **Critical** |
| `prd2-golem.md` — Mind (Grimoire)   | #3: Admission gate; #4: Ebbinghaus+strength; #5: Quality scoring; #7: Consolidation; #9: Hybrid retrieval | **Critical** |
| `prd2-golem.md` — Regime Detection  | #6: Regime-tagged evaluation                                                                              | **High**     |
| `prd2-golem.md` — Creation          | #10: Strategy validation with `simulateContract()` dry-run                                                | **High**     |
| `prd2-golem.md` — Observability     | #12: Cognitive quality metrics                                                                            | **High**     |
| `prd2-safety.md` — Section 7        | #1: Clarify Stage 2 uses `eth_call` in production; add Stage 4                                            | **Critical** |
| `memory-prd.md`                     | #3: Quality gates on bridge; #4: Strength; #7: Consolidation; #8: Embedding drift                         | **Critical** |
| `prd2-dev.md`                       | #6: Evaluation lifecycle (Anvil in dev only, `simulateContract()` in prod)                                | **High**     |
| `prd2-inference.md`                 | #2: Evaluator model routing                                                                               | **High**     |
| `../05-dreams/05-threats.md`        | #11: Feasibility-grounded threat rehearsal                                                                | **High**     |
| `../05-dreams/04-consolidation.md`  | #3: Admission Gate for promoted dream entries                                                             | **High**     |
| `../05-dreams/02-replay.md`         | #1: Deviation-anchored replay; #4: Dream-retrieval strengthening                                          | Medium       |

---

## Complete Citation Index

### Self-Correction and Reflection

- Huang, J. et al. "Large Language Models Cannot Self-Correct Reasoning Yet." ICLR 2024. arXiv:2310.01798. — *Demonstrates that intrinsic self-correction degrades LLM performance without external feedback; motivates Revision 1's external verification requirements.*
- Kamoi, R. et al. "When Can LLMs Actually Correct Their Own Mistakes?" _TACL_, 12:1417-1440, 2024. arXiv:2406.01297. — *Identifies the narrow conditions where self-correction works (format errors, not reasoning errors); supports the constraint that quality gates use on-chain data rather than LLM self-assessment.*
- Shinn, N. et al. "Reflexion: Language Agents with Verbal Reinforcement Learning." NeurIPS 2023. arXiv:2303.11366. — *Proposes storing verbal reflections as memory for future decisions; prior art for Grimoire episode storage and retrieval.*
- Zhao, A. et al. "ExpeL: LLM Agents Are Experiential Learners." AAAI 2024. arXiv:2308.10144. — *Shows that agents can extract reusable insights from past episodes; supports the Grimoire's insight extraction pipeline.*

### Reward Hacking

- Pan, A. et al. "Feedback Loops With Language Models Drive In-Context Reward Hacking." ICML 2024. arXiv:2402.06627. — *Warns that iterated LLM feedback loops produce reward hacking; motivates Revision 2's constraint on self-evaluating quality scores.*
- Pan, A. et al. "Spontaneous Reward Hacking in Iterative Self-Refinement." arXiv:2407.04549, 2024. — *Shows reward hacking emerges spontaneously without adversarial intent; strengthens the case for hard-coded red flag detectors that bypass LLM judgment.*
- Chen, A. et al. "Reasoning Models Don't Always Say What They Think." arXiv:2505.05410, 2025. — *Demonstrates that reasoning traces may not reflect actual computation; motivates external verification of reasoning quality rather than trusting chain-of-thought.*
- Lanham, T. et al. "Measuring Faithfulness in Chain-of-Thought Reasoning." arXiv:2307.13702, 2023. — *Quantifies how often chain-of-thought is unfaithful to the model's actual reasoning process; supports using outcome-based evaluation over process-based.*

### Vacuous Reasoning

- Hicks, D. et al. "ChatGPT is Bullshit." _Ethics and Information Technology_, 26:38, 2024. — *Philosophical analysis arguing LLM outputs are "bullshit" in the Frankfurt sense (indifferent to truth); motivates the vacuous reasoning detection pipeline.*
- Liang, Y. et al. "Machine Bullshit." arXiv:2507.07484, 2025. — *Formalizes and measures "bullshit" in LLM outputs; directly informs the over-hedging detector and the "hedged to meaninglessness" red flag.*
- Qiu, X. & Miikkulainen, R. "Semantic Density." NeurIPS 2024. arXiv:2405.13845. — *Proposes measuring information density in LLM outputs; alternative to semantic entropy for detecting low-content generation.*
- Farquhar, S. et al. "Detecting Hallucinations Using Semantic Entropy." _Nature_, 2024. — *Introduces semantic entropy: sample multiple explanations, cluster by meaning, compute entropy; directly implemented in the vacuous reasoning detector.*
- Geng, J. et al. "A Survey of Confidence Estimation and Calibration in LLMs." NAACL 2024. — *Comprehensive survey of LLM calibration methods; informs the choice of isotonic regression for the confidence calibration loop.*

### Memory Management

- Zhang, G. et al. "Adaptive Memory Admission Control for LLM Agents." arXiv:2603.04549, 2026. — *Proposes the A-MAC framework (future utility, factual confidence, novelty, recency, content prior) adopted and extended in the Grimoire admission gate.*
- Zhong, W. et al. "MemoryBank." AAAI 2024. arXiv:2305.10250. — *Implements forgetting curves for LLM agent memory; prior art for Grimoire's knowledge demurrage mechanism.*
- Chhikara, P. et al. "Mem0." arXiv:2504.19413, 2025. — *Open-source agent memory layer with vector storage and retrieval; reference implementation for Grimoire's LanceDB-backed episode store.*
- Park, J. S. et al. "Generative Agents." UIST 2023. arXiv:2304.03442. — *Simulates believable human behavior through memory retrieval; foundational work on affect-modulated retrieval in agent architectures.*
- arXiv:2505.16067. "How Memory Management Impacts LLM Agents." — *Benchmarks different memory strategies (append-only, summarize, forget) on agent performance; supports the Grimoire's multi-tier storage design.*
- arXiv:2603.02473. "Diagnosing Retrieval vs. Utilization Bottlenecks." — *Separates retrieval failures from utilization failures in RAG systems; informs the context attribution feedback loop that debugs Grimoire retrieval.*

### RAG and Embedding Drift

- Cuconasu, F. et al. "The Power of Noise." SIGIR 2024. arXiv:2401.14887. — *Shows that adding noise to RAG retrieval can paradoxically improve downstream task performance; informs the Grimoire's tolerance for imprecise retrieval.*
- Vejendla, A. "Drift-Adapter." EMNLP 2025. arXiv:2509.23471. — *Detects and adapts to embedding distribution drift over time; motivates periodic re-embedding of Grimoire entries as the embedding model updates.*

### Smart Contract Safety

- Chen, H. et al. "Demystifying Invariant Effectiveness." FSE 2024 (Trace2Inv). — *Evaluates which smart contract invariants actually catch bugs in production; informs the PolicyCage constraint selection methodology.*
- Wang, Z. et al. "AgentSpec." ICSE 2026. arXiv:2503.18666. — *Formal specification language for autonomous agent safety properties; prior art for PolicyCage's constraint expression format.*

### Agent Evaluation

- Zhou, A. et al. "Language Agent Tree Search." ICML 2024. arXiv:2310.04406. — *Combines LLM reasoning with tree search for agent planning; evaluation methodology applicable to Golem decision quality measurement.*
- Ma, C. et al. "AgentBoard." NeurIPS 2024. — *Multi-dimensional agent evaluation benchmark covering tool use, planning, and grounding; informs the multi-metric Gauntlet evaluation approach.*
- Forouzandeh, S. et al. "MACLA." AAMAS 2026. arXiv:2512.18950. — *Multi-agent continual learning architecture; evaluation methodology for measuring generational improvement across agent lifetimes.*

### Sleep and Dreaming

- Wilson & McNaughton. "Reactivation of hippocampal ensemble memories." _Science_, 1994. — *Demonstrates that the brain replays waking experiences during sleep; the neuroscience basis for NREM residual replay in Golem dream cycles.*
- Walker & van der Helm. "Overnight therapy?" _Psychological Bulletin_, 2009. — *Shows that sleep reduces emotional intensity of memories while preserving informational content; informs how dream cycles update episode valence.*
- Deperrois et al. "Perturbed and adversarial dreaming." _eLife_, 2022. — *Proposes that dreams inject noise for adversarial robustness; the basis for REM counterfactual generation that tests hypotheses against perturbed scenarios.*
- Revonsuo. "The reinterpretation of dreams." _BBS_, 2000. — *Threat simulation theory: dreams rehearse responses to dangerous scenarios; motivates prioritizing high-loss episodes for dream replay.*
- Mattar & Daw. "Prioritized memory access." _Nature Neuroscience_, 2018. — *Shows that replay prioritizes experiences with highest expected learning value; the basis for prioritizing episodes with large retrospective reversals in NREM replay.*
- Sutton. "Dyna." _ACM SIGART Bulletin_, 1991. — *Integrates model-based planning with model-free RL via simulated experience; the computational template for Golem dream cycles.*

### Evaluation Tooling

- UK AISI Inspect AI: https://inspect.aisi.org.uk/ — *Government AI safety evaluation framework; reference for structured evaluation methodology.*
- LangSmith: https://www.langchain.com — *LLM observability and evaluation platform; used for tracing and debugging Golem reasoning chains.*
- Promptfoo: https://github.com/promptfoo/promptfoo — *Open-source LLM evaluation tool supporting custom assertions; used for the Promptfoo evaluation suite in knowledge quality testing.*
