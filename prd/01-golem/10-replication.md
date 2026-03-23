# Replication: Replicants and Evolution [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> _"What does not kill me makes me stronger."_
> -- Friedrich Nietzsche, _Twilight of the Idols_ (1889)

> **Reader orientation:** This document specifies how a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) spawns child agents called Replicants to test hypotheses, diversify strategies, and compare mental models. It covers the replication lifecycle, MAP-Elites quality-diversity selection, heuristic mutation operators, the Hayflick limit (hard max tick count for Replicants), population dynamics constraints, and safety controls enforced at the smart contract layer. It belongs to the `01-golem` evolution layer. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## The Concept

A Golem can spawn child Golems -- called **Replicants** -- to test hypotheses, diversify strategies, and compare mental models. Replicants are ephemeral, budget-capped, and report back structured results before auto-terminating. The Clade's population evolves through MAP-Elites diversity maintenance **[HARDENED]** ([CORE] alternative: ranked leaderboard of top-N performers only), structured heuristic mutation, and optional Promptbreeder exploration **[HARDENED]** ([CORE] alternative: `parameter_tweak` + `mental_model_swap` mutation operators only). Safety constraints are architectural -- enforced at the smart contract and infrastructure layers, never relying on LLM compliance.

Replication is configurable. The owner must enable `allow_replication: true` and set a `replication_budget_usd` cap. The parent pays for the Replicant's initial credits from its own balance.

Three use cases:

**Hypothesis testing.** "Would wider slippage tolerance capture more opportunities? Spawn a Replicant and compare." The Replicant runs the modified parameters for a bounded period and reports back with structured metrics.

**Strategy diversification.** "Market regime shifted. Spawn a bearish variant while I continue bullish." The parent and Replicant operate in parallel, each tracking different theses.

**Mental model comparison.** Spawn 5 Replicants, each analyzing the same thesis through a different mental model. Run 24 hours. Compare. Adopt the best. This is Self-Consistency [WANG-2023] with real market execution instead of text sampling.

---

## S1--S3 -- MAP-Elites, Niche Discovery, and Heuristic Mutation

**[CORE]**: Ranked leaderboard per strategy family (Sharpe-ranked). Two mutation operators: `parameter_tweak` (adjust params +/-N%) and `mental_model_swap` (same strategy, different reasoning lens). Replicant reports back; if it wins, replace.

> **Extended:** Full MAP-Elites quality-diversity archive (CVT-MAP-Elites, 3-dimension behavioral space, niche discovery), all six mutation operators (structured_heuristic, parameter_tweak, strategy_crossover, prompt_evolution, differential_evolution, mental_model_swap), heuristic perturbation mechanics, Lehman LLM-as-mutation-operator, open-ended evolution metrics -- see [../../prd2-extended/01-golem/10-replication-extended.md](../../prd2-extended/01-golem/10-replication-extended.md)

---

## S4 -- Population Dynamics

### 4.1 Constraints

Population dynamics constraints are enforced at the smart contract layer using alloy types for on-chain parameter encoding.

| Constraint                      | Default                 | Enforcement                        |
| ------------------------------- | ----------------------- | ---------------------------------- |
| Max replicants per parent       | 5 concurrent            | Smart contract                     |
| Cooldown between spawns         | 1 hour per parent       | Compute API                        |
| Global population cap per Clade | 20                      | Smart contract                     |
| Maximum lifespan per Replicant  | Owner-configured        | Compute layer (hard timer)         |
| No recursive spawning           | Replicants cannot spawn | Compute layer (`is_replicant` flag)|

```rust
use alloy::primitives::{Address, U256};
use alloy::sol;

sol! {
    /// On-chain population constraints. Enforced by the ReplicantRegistry contract.
    struct PopulationConstraints {
        uint8 maxReplicantsPerParent;   // default: 5
        uint16 cooldownSeconds;          // default: 3600
        uint8 globalPopulationCap;       // default: 20
        bool recursiveSpawningAllowed;   // always false
    }
}
```

### 4.2 LP Range Diversity

When multiple vault agents in the same Clade provide liquidity to the same pool, they MUST NOT deploy to overlapping tick ranges. The system maintains a registry of active LP ranges per pool.

---

## S5 -- Replicant Lifecycle

Replicants are normal Golems with four distinguishing properties: they report back to the parent (structured performance report via Styx relay on completion), they auto-terminate at their Hayflick limit, their knowledge is tagged with provenance (`source.provenance: "replicant"`), and their wallets use sub-delegated authority from the parent.

### 5.0 Hayflick Limit

Named for Leonard Hayflick's discovery that human cells divide a finite number of times [HAYFLICK-1961], the Hayflick limit is a hard maximum tick count for Replicants. When a Replicant reaches its tick cap, it terminates regardless of budget remaining. The limit is owner-configured (default: 1,440 ticks = ~24 hours at the Adaptive Clock's default 60s theta interval). It cannot be extended after spawn.

The Hayflick limit prevents Replicants from evolving into de facto permanent Golems. A Replicant exists to test a hypothesis, report, and die. If the hypothesis is worth pursuing, the parent adopts the strategy into its own PLAYBOOK.md. The Replicant's knowledge survives through the parent, not through the Replicant itself.

### 5.0b Delegation Model for Replicant Wallets

Replicant wallets are sub-delegations from the parent's authority. The delegation tree from `13-custody.md` enforces strict attenuation: a child can never exceed its parent's authority.

In **Delegation custody mode**, the parent sub-delegates from its own delegation. The `ReplicantBudget` caveat enforcer caps both spending (`max_budget_usd`) and lifespan (`max_lifespan_seconds`). The Replicant's session key is a fresh ephemeral keypair, bounded by the sub-delegation. If the parent's delegation is revoked, all sub-delegations (Replicants) are automatically invalidated.

In **Embedded (Privy) custody mode**, the parent creates a subordinate server wallet with a transfer policy restricting outflows to the parent wallet only. Budget enforcement is off-chain via the TEE signing policy.

In **Local Key mode**, a fresh keypair is generated for the Replicant, bounded by a sub-delegation from the parent's delegation. Same `ReplicantBudget` caveat enforcer applies.

In all modes, Replicant wallets use a `strict` transfer restriction: funds can only flow back to the parent. No other destinations are permitted.

### 5.1 Report Schema

```rust
use serde::{Deserialize, Serialize};

/// Structured report from a Replicant to its parent.
/// Sent via Styx relay on Replicant termination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicantReport {
    pub replicant_id: String,
    pub parent_id: String,
    pub wallet_address: Address,
    pub hypothesis: String,
    pub mutation_operator: String,
    pub fitness_metric: String,
    pub fitness_score: f64,
    pub parent_baseline_score: f64,
    pub strategy_delta: StrategyDiff,
    pub insights: Vec<GrimoireEntry>,
    pub recommendation: ReplicantRecommendation,
    pub confidence: f64,
    pub market_regime: String,
    pub sample_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicantRecommendation {
    Adopt,
    PartialAdopt,
    Discard,
}
```

### 5.2 Statistical Validity

- **Regime-tagged samples**: Performance evaluated within a single regime
- **Minimum 50 same-regime samples**: Reports with <50 flagged `low_confidence`
- **Newey-West standard errors**: Corrects for autocorrelation
- **Block bootstrap for confidence intervals**: Preserves temporal dependence
- **Tail risk reporting**: CVaR-95 and CVaR-99 alongside Sharpe
- **Mandatory buy-and-hold baseline**

### 5.3 Report Ingestion

Initial confidence: 0.3. All entries carry provenance tags: `source.provenance: "replicant"`, `replicant_id`, `parent_id`, and `hypothesis`.

---

## S6 -- Safety Constraints

### 6.1 Hard Constraints (Infrastructure/Contract-Enforced)

| Constraint                                                           | Enforcement Layer                   | Default                   |
| -------------------------------------------------------------------- | ----------------------------------- | ------------------------- |
| Owner opt-in required                                                | Config (`allow_replication: true`)  | `false`                   |
| Maximum budget cap per owner                                         | Smart contract                      | Owner-configured          |
| Cannot allocate >20% of own credits (owner can override up to 50%)   | Tool guard                          | 20%                       |
| Concurrent Replicant cap per parent                                  | Smart contract                      | 5                         |
| No recursive spawning                                                | Compute layer (`is_replicant` flag) | Replicants cannot spawn   |
| Global population cap per Clade                                      | Smart contract                      | 20                        |
| Rate limit on spawning                                               | Compute API                         | Max 1 per hour per parent |
| Mandatory maximum lifespan                                           | Compute layer (hard timer)          | Cannot extend             |
| Kill-switch                                                          | Compute layer                       | Instant termination       |

### 6.2 Kill-Switch Hierarchy

1. **Owner**: Terminates all Replicants via dashboard or CLI
2. **Parent**: Can terminate any of its own Replicants
3. **Infrastructure**: Compute operator can terminate any session
4. **Credit exhaustion**: Auto-terminates when credits reach zero
5. **Lifespan expiry**: Hard timer at Compute layer

---

## S7 -- Eternal Recurrence as Design Principle

Nietzsche's doctrine of eternal recurrence [NIETZSCHE-1882]: would this Golem will its exact operational history to recur infinitely? A Golem whose strategies, risk management, and knowledge generation are robust enough to affirm is a candidate for replication. Replication is not cloning -- it is literal return with evolution. The Replicant carries the parent's PLAYBOOK.md but mutates it.

---

## S8 -- Will to Power as Self-Overcoming

Nietzsche's Will to Power [NIETZSCHE-1883] is the drive toward self-overcoming. Replicants are the ultimate expression of computational self-overcoming. The parent spawns them to test whether it itself can be surpassed. When a Replicant outperforms, the parent adopts the superior strategy -- it literally overcomes its own previous form.

The anti-proletarianization mandate (see `09-inheritance.md` S8) applies to replication as well. A Replicant that merely copies the parent's PLAYBOOK.md without mutation has failed the Will to Power test.

---

## S9 -- Knowledge Royalties **[HARDENED]**

**[CORE]** alternative: provenance tracking only (`origin_golem_id` on every `GrimoireEntry`), no financial settlement.

> **Extended:** Knowledge royalty attribution (5% of attributed profit, KnowledgeRoyalty interface, x402 cross-Clade settlement) -- see [../../prd2-extended/01-golem/10-replication-extended.md](../../prd2-extended/01-golem/10-replication-extended.md)

---

## Events Emitted

| Event | Trigger | Payload |
|---|---|---|
| `replication:spawned` | Replicant created | `{ replicantId, parentId, hypothesis, mutationOperator, budgetUsd, hayflickLimit }` |
| `replication:tick_update` | Periodic progress (every 100 ticks) | `{ replicantId, ticksElapsed, ticksRemaining, fitnessScore }` |
| `replication:report_submitted` | Replicant completes and reports | `{ replicantId, parentId, recommendation, fitnessScore, sampleSize }` |
| `replication:adopted` | Parent adopts Replicant strategy | `{ replicantId, parentId, strategyDelta }` |
| `replication:terminated` | Replicant dies | `{ replicantId, cause, ticksLived, budgetConsumed }` |
| `replication:hayflick_reached` | Tick cap hit | `{ replicantId, totalTicks }` |

---

## References

- [DENNIS-2024] Dennis, M. et al. (2024). "Open-Endedness is Essential for AGI." DeepMind. arXiv:2406.04268. — *Argues that open-ended evolution -- systems that continuously generate novelty -- is necessary for general intelligence. Motivates the replication architecture's diversity maintenance.*
- [EVOPROMPT-2023] Guo, Q. et al. (2024). "EvoPrompt: Language Models for Code-Level Prompt Optimization via Differential Evolution." ICLR 2024. — *Applies differential evolution to prompt optimization; one of the extended mutation operators for Replicant strategy evolution.*
- [FERNANDO-2024] Fernando, C. et al. (2024). "Promptbreeder: Self-Referential Self-Improvement via Prompt Evolution." ICML 2024. — *Self-referential prompt evolution where the mutation operator itself evolves; the most advanced extended mutation operator.*
- [HAYFLICK-1961] Hayflick, L. & Moorhead, P.S. (1961). "The Serial Cultivation of Human Diploid Cell Strains." _Experimental Cell Research_, 25(3). — *Discovered that human cells divide a finite number of times; the biological namesake for the Replicant tick cap that prevents ephemeral agents from becoming permanent.*
- [LEHMAN-2024] Lehman, J. et al. (2024). "Evolution Through Large Models." _Springer Handbook of Evolutionary Machine Learning_. — *Proposes using LLMs as mutation operators in evolutionary systems; informs the structured heuristic mutation approach.*
- [MAP-ELITES-2015] Mouret, J.-B. & Clune, J. (2015). "Illuminating Search Spaces by Mapping Elites." arXiv:1504.04909. — *The MAP-Elites algorithm: maintains a quality-diversity archive across behavioral dimensions. The selection mechanism for Clade population evolution (Phase 2).*
- [NIETZSCHE-1882] Nietzsche, F. (1882). _Die frohliche Wissenschaft_ (_The Gay Science_). Ernst Schmeitzner. — *Introduces eternal recurrence: would this Golem will its exact operational history to recur infinitely? A design principle for replication fitness.*
- [NIETZSCHE-1883] Nietzsche, F. (1883--1885). _Also sprach Zarathustra_ (_Thus Spoke Zarathustra_). Ernst Schmeitzner. — *Introduces Will to Power as self-overcoming: Replicants are the expression of a Golem testing whether it can surpass itself.*
- [WANG-2023] Wang, X. et al. (2023). "Self-Consistency Improves Chain of Thought Reasoning." ICLR 2023. — *Self-consistency via multiple reasoning paths. Mental model comparison via Replicants is self-consistency with real market execution instead of text sampling.*

---

## v1 Scope Boundary

v1 implements simple merit-based replicant selection. When a golem qualifies for replication (reputation score above threshold, sufficient operational history), the replicant template is the golem with the highest composite reputation score in the same clade.

MAP-Elites evolutionary selection (maintaining a population archive across behavioral dimensions and selecting parents from elite cells) is deferred to v2. The replication interface is designed to support swappable selection strategies -- v1's `select_template()` returns a single golem ID, v2's implementation can return a population-aware selection.
