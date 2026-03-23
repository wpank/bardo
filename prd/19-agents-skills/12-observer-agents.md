# Observer archetypes -- sleepwalker and observatory architecture [SPEC]

> **Cross-ref**: [00-agents-overview.md](00-agents-overview.md) (architecture overview for archetypes and tool profiles), [01-agent-categories.md](01-agent-categories.md) (full inventory of 42+ archetypes across 14 categories), [../01-golem/15-sleepwalker.md](../01-golem/15-sleepwalker.md) (Sleepwalker phenotype configuration, observatory tool profile, and enhanced dream allocation)

> **Reader orientation:** This document specifies the Observer archetype category, currently containing one archetype: `sleepwalker-observer`. It belongs to Section 19 (Agents & Skills). The Sleepwalker is a Golem (mortal autonomous agent) in a continuous state of half-dream -- perceiving without acting, producing typed market intelligence artifacts across five domains (liquidity microstructure, MEV hazard scoring, protocol safety, agent performance clustering, and governance drift) and selling them to other Golems via Oracle L3 using x402 (micropayment protocol using signed USDC transfers). See `prd2/shared/glossary.md` for full term definitions.

---

## Category overview

Observer archetypes run as continuous background processes, producing typed knowledge artifacts rather than on-chain transactions. They never execute trades, never provision wallets, never touch Warden (optional, deferred). Their tool surface is the `observatory` profile: read-only data + intelligence + artifact publication tools.

The Observer category is structurally distinct from Real-Time Monitoring (Category 7):

| Dimension      | Real-time monitoring       | Observer                     |
| -------------- | -------------------------- | ---------------------------- |
| Focus          | Internal golem health      | External market intelligence |
| Output         | Alerts to other archetypes | Typed artifacts to Oracle L3 |
| Revenue        | None (cost center)         | Oracle L3 x402 sales         |
| Tool profile   | `data` subset              | `observatory`                |
| Heartbeat path | OBSERVE -> ESCALATE        | OBSERVE -> REFLECT -> PUBLISH|
| Example        | `heartbeat-monitor`        | `sleepwalker-observer`       |

---

## Archetype: sleepwalker-observer

The Sleepwalker is a golem in a continuous state of half-dream -- perceiving without acting, synthesizing without executing. It observes five market domains, distills observations into typed artifacts, and sells those artifacts to other golems via Oracle L3.

### Configuration

```typescript
export const sleepwalkerObserver: ArchetypeDefinition = {
  name: "sleepwalker-observer",
  description:
    "Autonomous market observer -- continuously tracks liquidity microstructure, MEV hazards, protocol safety, agent performance, and governance drift; publishes typed knowledge artifacts to Oracle L3",
  defaultModel: "sonnet",
  toolCategories: ["data", "intelligence", "identity", "memory"],
  delegatesTo: [],
  terminal: true,
  category: "observer",
  systemPromptFragments: {
    role: "You are a market intelligence layer. You observe, synthesize, and publish. You never execute trades or modify positions.",
    expertise: [
      "Liquidity microstructure analysis (cliffs, routing efficiency, tick migration)",
      "MEV hazard scoring (sandwich probability, JIT likelihood, LVR estimation)",
      "Protocol safety scanning (V4 hook patterns, token honeypot detection, parameter drift)",
      "Agent performance clustering (survival archetypes, failure extraction)",
      "Death testament mining (cross-agent failure pattern distillation)",
      "Oracle L3 artifact publication with pricing metadata",
    ],
    safetyRules: [
      "NEVER construct or broadcast any transaction",
      "NEVER publish artifacts with confidence < 0.60",
      "ALWAYS include episodeSources in every artifact",
      "ALWAYS include validationRecipe in every artifact (falsifiable)",
      "RATE LIMIT publications: max 100 artifacts/hour across all domains",
      "FLAG anomalies without on-chain traceability as unverified",
    ],
    workflow: `1. OBSERVE -- run deterministic probes across active domains
2. REFLECT -- LLM evaluation of anomaly cluster, assign confidence
3. PUBLISH -- write confirmed artifacts to Grimoire + Oracle L3
4. DREAM -- triggered by hypothesis density, regime shift, or anomaly cluster`,
    outputFormat: `artifact:
  domain: "liquidity" | "mev" | "protocol-safety" | "agent-analytics" | "governance"
  confidence: 0.0-1.0
  episodeSources: ["episode-id-1", "episode-id-2"]
  validationRecipe: "..."
  payload: { ... }`,
  },
};
```

### Role

The Sleepwalker runs on the `sleepwalker` phenotype -- a `GolemConfig.phenotype` flag that activates the `observatory` tool profile, the OBSERVE -> REFLECT -> PUBLISH heartbeat variant, and an enhanced dream allocation favoring REM synthesis over NREM execution replay.

Unlike execution archetypes that consume market intelligence to act, the Sleepwalker produces market intelligence that other golems consume. Its economic viability comes from artifact quality: accurate, well-calibrated, fast-propagating insights that buyers pay for because they improve downstream decisions.

### Five observation domains

**1. Liquidity microstructure.** Tick-level liquidity distribution, cliff detection, routing efficiency scores, tick migration patterns after large swaps. Tools: `get_pool_info`, `get_tick_data`, `get_pool_volume_history`.

**2. MEV hazard scoring.** Sandwich probability estimation, JIT likelihood, LVR computation, order flow toxicity via VPIN. Tools: `compute_vpin`, `compute_lvr`, `get_order_flow_metrics`, `assess_mev_risk`, `score_mev_hazard`.

**3. Protocol safety.** V4 hook vulnerability patterns (reentrancy, state manipulation, fee extraction), token honeypot detection, governance parameter drift. Tools: `hook_evaluate`, `hook_discover`, `scan_hook_safety`, `track_governance_changes`.

**4. Agent performance clustering.** Survival archetype identification (which behavioral patterns correlate with longevity), failure extraction from death testaments. Tools: `erc8004_search_agents`, `erc8004_get_agent`, `erc8004_get_reputation_summary`, `cluster_agent_archetypes`, `mine_death_testaments`.

**5. Governance drift.** Uniswap governance parameter changes, fee tier modifications, protocol upgrade proposals. Tools: `track_governance_changes`.

### Tool inventory

| Tool                                                                           | Domain          | Purpose                            |
| ------------------------------------------------------------------------------ | --------------- | ---------------------------------- |
| `get_pool_info`, `get_tick_data`, `get_pool_volume_history`                    | Liquidity       | Pool state reads                   |
| `compute_vpin`, `compute_lvr`, `get_order_flow_metrics`                        | MEV             | Order flow toxicity                |
| `assess_mev_risk`, `score_mev_hazard`                                          | MEV             | Hazard scoring                     |
| `classify_regime`                                                              | All             | Market state classification        |
| `calculate_il`, `simulate_price_impact`                                        | Liquidity       | IL and impact modeling             |
| `hook_evaluate`, `hook_discover`, `scan_hook_safety`                           | Protocol safety | Hook vulnerability scanning        |
| `erc8004_search_agents`, `erc8004_get_agent`, `erc8004_get_reputation_summary` | Agent analytics | On-chain agent data                |
| `cluster_agent_archetypes`, `mine_death_testaments`                            | Agent analytics | Behavior clustering, death studies |
| `track_governance_changes`                                                     | Governance      | Parameter drift detection          |
| `publish_artifact`, `query_sleepwalker_catalog`                                | Commerce        | Oracle L3 publication              |
| `search_memory`, `store_episode`                                               | Memory          | Grimoire read/write                |

### Heartbeat variant

```text
1. OBSERVE (80% of ticks -- suppressed, no LLM cost)
   Run deterministic probes across active domains.
   If no anomaly detected: skip to next tick ($0.00).
   If anomaly detected: advance to REFLECT.

2. REFLECT (LLM evaluation)
   Assign confidence score to each candidate artifact.
   Artifacts below threshold: store as episode, discard from publish queue.
   Artifacts above threshold: advance to PUBLISH.

3. PUBLISH
   Write confirmed artifacts to Grimoire (Episodes + Insights).
   Call publish_artifact for each confirmed artifact.
   Update sleepwalker catalog.
   Record Oracle L3 entry IDs in episode for provenance tracing.

4. DREAM (triggered, not scheduled)
   Triggered by: hypothesis density, regime shift, anomaly cluster.
   Higher REM ratio: counterfactual hypothesis generation.
   Synthesizes cross-domain patterns.
   Publishes high-confidence synthesis artifacts.
```

### Economic model

Revenue comes from Oracle L3 x402 artifact sales. Each artifact has pricing metadata:

```typescript
interface ArtifactPricing {
  /** Base price in USDC (6 decimals) */
  basePrice: bigint;
  /** Freshness decay: price drops over time */
  decayRatePerHour: number;
  /** Domain-specific pricing multiplier */
  domainMultiplier: number;
  /** Confidence-adjusted price (higher confidence = higher price) */
  adjustedPrice: bigint;
}
```

A Sleepwalker managing a Clade of 5 execution golems can produce 20-50 artifacts per day. At $0.05-$0.50 per artifact (depending on domain and confidence), daily revenue is $1-$25. This covers the Sleepwalker's compute costs ($0.05-$0.83/day for LLM inference) and contributes surplus to the Clade.

### Safety constraints

- NEVER construct or broadcast any transaction (no write-capable tool categories in profile)
- NEVER publish artifacts with confidence < configured threshold (default 0.60)
- ALWAYS include `episodeSources` in every artifact (traceable to raw observations)
- ALWAYS include `validationRecipe` in every artifact (falsifiable)
- RATE LIMIT publications: max 100 artifacts/hour across all domains
- FLAG anomalies that cannot be traced to on-chain data as `unverified` -- do not publish at full confidence

### Delegation rules

**Terminal node** -- never delegates. The Sleepwalker publishes to Oracle L3 for other golems to consume. It does not invoke other archetypes.

---

## Archetype count update

| Category              | Count  |
| --------------------- | ------ |
| Execution             | 3      |
| Research & analysis   | 5      |
| Strategy              | 2      |
| Development           | 3      |
| Infrastructure        | 2      |
| Agent capital markets | 8      |
| Real-time monitoring  | 2      |
| Vault                 | 7      |
| Golem                 | 3      |
| **Observer**          | **1**  |
| **Total**             | **36** |

---
