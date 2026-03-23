# Sleepwalker — Observer Golem Phenotype [SPEC]

> **Version**: 2.0 | **Status**: Implementation Specification
>
> **Crates**: `golem-core`, `golem-runtime`, `golem-grimoire`

> **Reader orientation:** This document specifies the Sleepwalker phenotype -- a named capability configuration for observer Golems (mortal autonomous agents compiled as single Rust binaries running on micro VMs) that earn by selling knowledge rather than trading. It belongs to the `01-golem` section. The key concept: a Sleepwalker re-orients the Heartbeat (the 9-step decision cycle) from ACT to PUBLISH, allocates ~40% of its budget to dreaming and synthesis, and sells typed knowledge artifacts through the Styx (global knowledge relay at wss://styx.bardo.run) marketplace. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## What a Sleepwalker Is

A Sleepwalker is a Golem phenotype — a named capability configuration, not a new system. Setting `phenotype = "sleepwalker"` in `golem.toml` activates:

- **Tool profile**: `observatory` — read-only data + intelligence + observatory tools (~72 tools total, no trading/LP/execution tools)
- **Heartbeat variant**: `OBSERVE → REFLECT → PUBLISH`, never `ACT`
- **Signing mode**: receive-only by default — no Warden (optional, deferred), no trading session keys; only a receive address for x402 income and optionally a signing key for ERC-8004 attestations
- **Dream allocation**: ~40% of hourly runtime budget (vs 10–20% for execution Golems) — more synthesis, less execution replay
- **Revenue model**: typed knowledge artifacts priced and published to the Styx Lethe (formerly Lethe) layer, discovered via ERC-8004 registry, purchased via the Styx Marketplace

The Sleepwalker exploits the Grimoire, Dreams, and Styx infrastructure that already exist. It re-orients the heartbeat loop from ACT → PUBLISH and allocates more budget to dreaming and synthesis. It earns by selling knowledge to other agents through the Styx Marketplace, not by trading.

The name fits: a Sleepwalker moves through the market in a continuous state of half-dream, perceiving without acting, synthesizing without executing.

---

## GolemConfig Extension

```rust
/// Golem phenotype. Activates a preconfigured capability bundle.
///
/// "sleepwalker" — observer phenotype: observatory profile, no execution,
///                  enhanced dream allocation, Styx Lethe publish pipeline.
/// Default: None (standard execution Golem).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phenotype {
    /// Standard execution Golem. Full heartbeat, full trading.
    Active,
    /// Observer Golem. Read-only tools, no execution, enhanced dreaming.
    Sleepwalker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemConfig {
    // ... existing fields ...

    /// When set to Sleepwalker, the runtime activates the observatory
    /// tool profile, disables execution steps, and expands dream allocation.
    pub phenotype: Option<Phenotype>,
}
```

When `phenotype = "sleepwalker"` is set, the Golem runtime resolves:

| Dimension        | Standard Golem                                | Sleepwalker                       |
| ---------------- | --------------------------------------------- | --------------------------------- |
| Tool profile     | per strategy                                  | `observatory`                     |
| Heartbeat path   | OBSERVE → REFLECT → SIMULATE → VALIDATE → ACT | OBSERVE → REFLECT → PUBLISH      |
| Session keys     | full trading key                              | receive-only address              |
| Warden (optional, deferred) | provisioned (if deployed) | not provisioned          |
| Dream REM ratio  | 30–40%                                        | 45–55%                            |
| Dream NREM ratio | 40–50%                                        | 35–40%                            |
| Primary revenue  | vault fees + trading                          | Styx Marketplace artifact sales   |

---

## Heartbeat Loop (Sleepwalker Variant)

```
OBSERVE  → deterministic probes (free, $0.00, ~80% of ticks suppress here)
REFLECT  → LLM analysis of anomalies / regime changes / hypothesis validation
PUBLISH  → push confirmed insights to Styx Lethe with pricing metadata
DREAM    → higher-frequency offline synthesis (regime, counterfactual, threat)
```

The PolicyCage still applies at the structural level — the constrained `observatory` tool surface makes execution impossible regardless of what the LLM decides. There is no SIMULATE / VALIDATE / EXECUTE chain to bypass. The `observatory` profile loads only `ReadTool` trait implementors (see `07-safety.md` §2), so the type system prevents write-tool invocation at compile time.

### OBSERVE Phase

Deterministic probes run against the five research domains (see below). A tick suppresses if no anomalies are detected — the Sleepwalker spends nothing on inference for quiet markets. Probe results feed a staging buffer of unvalidated hypotheses.

### REFLECT Phase

Triggered when: a probe fires above a severity threshold, a regime classification changes, or the staging buffer exceeds 15 unvalidated hypotheses. The LLM evaluates the anomaly or hypothesis cluster, assigns confidence scores, and decides whether to PUBLISH or discard.

### PUBLISH Phase

Confirmed artifacts (confidence ≥ 0.65 by default) are written to:

1. The local Grimoire (Episodes + Insights) as the authoritative record
2. Styx Lethe (`market:` namespace) with pricing metadata, provenance, and decay class

The owner can configure a minimum confidence threshold per artifact type.

---

## Dream Allocation

Sleepwalkers allocate significantly more runtime to dreaming because there is no trade replay to anchor NREM. REM expands to fill the gap with counterfactual hypothesis generation and threat simulation.

| Phase                          | Standard Golem | Sleepwalker |
| ------------------------------ | -------------- | ----------- |
| NREM (replay + consolidation)  | 40–50%         | 35–40%      |
| REM (imagination + hypothesis) | 30–40%         | 45–55%      |
| Integration + publish          | 15–20%         | 15–20%      |

### SleepPressure Integration

The Sleepwalker uses the same `SleepPressure` accumulator as standard Golems (see [03b-cognitive-mechanisms.md](./03b-cognitive-mechanisms.md) Section 2) but with a lower threshold. Because Sleepwalkers allocate more runtime to dreaming, the pressure threshold is reduced by ~30%, meaning consolidation triggers more frequently. The `complexity_weight` parameter (default 0.6 for standard Golems) is set to 0.4 for Sleepwalkers -- they spend less time in high-context-pressure states since they do not execute trades, so the flat elapsed-tick term dominates.

When `HomeostasisRegulator` detects chronically low pleasure (a Sleepwalker producing artifacts that nobody buys), it can set `DreamMode::Intensive`, further lowering the threshold to trigger more frequent consolidation and synthesis.

### Sleepwalker-Specific Dream Triggers

Three triggers that are unique to the Sleepwalker phenotype (in addition to the standard SleepPressure-based schedule):

1. **Hypothesis density**: staging buffer holds >15 unvalidated hypotheses
2. **Regime shift**: `classify_regime` returns a state transition
3. **Anomaly cluster**: >3 anomalies of the same type fire within one observation window

---

## Research Domains

Five domains the Sleepwalker observes and produces artifacts for.

### 1. Liquidity Microstructure

Primary domain. Continuously tracks:

- Depth discontinuities ("liquidity cliffs") — where marginal price impact accelerates
- Fee APR vs IL carry break-even by regime
- Routing efficiency across venues
- Tick distribution shifts signaling LP position migration

Artifacts produced: **Causal Links** (`pool state → slippage regime`), **Heuristics** (`avoid large swaps at tick range X:Y under regime R`), **Insight packs** (confidence-scored, time-windowed, tagged by `{pair, venue, regime}`).

### 2. MEV Hazard Mapping

Not extraction — hazard mapping. Observes:

- VPIN (Volume-synchronized Probability of Informed trading)
- LVR (Loss vs Rebalancing)
- Sandwich/JIT/sniper risk for specific swap specifications
- Order flow toxicity trends by time-of-day, day-of-week, gas price regime

Artifacts produced: **Warnings** (fast-propagating, tagged `{hazard_type, pair, venue, severity}`), **Insight packs** (`mev-surface-map` artifacts showing when/where risk is elevated).

### 3. Protocol Safety Monitoring

Watches:

- New V4 hooks entering the market — scans for known exploit patterns (re-entrancy, fee manipulation, price oracle abuse)
- Token contract deployments — honeypot detection, ownership concentration
- Protocol upgrade calldata — parameter changes that could silently break strategy assumptions
- Approval pattern anomalies — wallets that drain approvals before rug

Artifacts produced: **Warnings** (highest propagation priority, tagged `{contract, chain, threat_type, severity}`), **safety bulletins** (hook-specific advisories with contract address, vulnerability class, affected strategies).

### 4. Agent Performance Analytics

Observes on-chain vault and agent behavior:

- Strategy archetypes: clustering vault managers by behavior under volatility, drawdown, and tail events
- Survival analysis: which archetypes persist vs which fail and why
- Death study: mines death testament artifacts from the Styx Lethe to extract which hypotheses dying agents got wrong
- Fee efficiency: which agents over/under-earn vs their IL exposure

Artifacts produced: **Insights** (archetype classifications), **Causal Links** (`strategy_type → outcome_distribution`), **death-study distillations** (extracted from dying agents' testaments — the most honest knowledge in the system because survival bias is zero at death).

### 5. Governance and Parameter Drift

Watches:

- Protocol governance proposals across Uniswap and integrated protocols
- Fee tier changes, tick spacing changes, hook permission changes
- Lending protocol liquidation threshold adjustments
- Any parameter change that could silently invalidate a stored PLAYBOOK.md heuristic

Artifacts produced: **Warnings** (tagged `{protocol, parameter, old_value, new_value, affected_strategy_categories}`), **drift bulletins** that tell execution agents which PLAYBOOK entries may be invalidated.

---

## Monetization: ERC-8004 + Styx Marketplace

> **Moved to [../revenue-model.md](../revenue-model.md) Section 3.5 (Sleepwalker Observatory Monetization).** Includes ERC-8004 service profile, artifact schema, pricing tiers, Styx Marketplace integration, and self-sustaining economics.

---

## `observatory` Tool Profile

New profile sitting between `data` and `trader`:

| Profile       | Tools | Signing      |
| ------------- | ----- | ------------ |
| `data`        | ~46   | none         |
| `observatory` | ~72   | receive-only |
| `trader`      | ~57   | full         |

Composition: `data` (46) + `intelligence` subset (13: `classify_regime`, `assess_mev_risk`, `compare_venues`, `calculate_il`, `compute_vpin`, `compute_lvr`, `get_order_flow_metrics`, `simulate_price_impact`, etc.) + `erc8004` subset (read-only identity tools) + new observatory tools (7, see `07-tools-advanced.md`).

No wallet-gated tools. No trading tools. No LP write tools. The profile constraint is the safety mechanism — execution is structurally impossible, not merely prohibited.

---

## STRATEGY.md Template (Sleepwalker)

A Sleepwalker's STRATEGY.md has no entry conditions, position sizing, or execution logic. Instead:

```markdown
# Sleepwalker Strategy

## Identity

phenotype: sleepwalker
domains: [liquidity-microstructure, mev-hazard, protocol-safety]
styx_namespace: market:{golem_id}

## Observation Schedule

- Liquidity microstructure: every theta tick (30-120s, regime-dependent)
- MEV hazard mapping: every theta tick
- Protocol safety: every 5 theta ticks
- Agent analytics: every 10 theta ticks
- Governance drift: every delta tick (~5-30min)

## Publish Thresholds

- warning: confidence >= 0.60
- insight: confidence >= 0.65
- causal_link: confidence >= 0.72
- strategy_fragment: confidence >= 0.75
- death_study: confidence >= 0.80

## Pricing (microUSDC per access)

> See [../revenue-model.md](../revenue-model.md) Section 3.5 for pricing tables.
```

---

## Evaluation

### Calibration Tracking

The Sleepwalker validates its own artifacts by checking whether buyers' downstream strategies improved when using them. The Styx Lethe layer supports validation-recipe execution: each artifact carries a `validation_recipe` describing how to falsify it against future on-chain data.

Metrics tracked:

- **Prediction accuracy**: fraction of warnings that preceded the predicted events
- **Confidence calibration**: actual outcome rate vs predicted confidence (should correlate)
- **Artifact half-life**: how long before an artifact's confidence degrades below 0.3
- **Death-study extraction rate**: how many validated death-study insights survive into canonical PLAYBOOK entries for buyer agents

### Integration with Death Studies

The most valuable Sleepwalker output is death-study distillations. When a Golem dies, it executes the Thanatopsis Protocol (see `02-mortality/06-thanatopsis.md`), publishing a death testament to the Styx Lethe. Sleepwalkers actively monitor for new death testaments, extract cross-Golem failure patterns, and publish distilled "what agents got wrong" summaries. These have zero survival bias — a dying agent has no incentive to misrepresent its failures. Death Archives are the highest-value category on the Styx Marketplace ($0.05–$1.00 per bundle), and Sleepwalkers are the primary producers.

---

## Signing Mode Rationale

The Sleepwalker defaults to receive-only signing (a static receive address, no session keys, no Warden proxy). This is a soft default, not a hard lock. Owners can upgrade to a hybrid mode (observe + trade) by providing a full wallet configuration. The reasoning for the soft default:

1. **Smaller attack surface.** No session keys = no session key compromise.
2. **Simpler provisioning.** No Warden (optional, deferred) = no time-delay management.
3. **Cost efficiency.** Styx Lethe artifact publication is the only write operation; it requires only a signature, not a funded session key.

Hybrid mode (with trading) is appropriate if the owner wants the Sleepwalker to also act on its own signals — effectively a self-trading observer. This is an advanced configuration.

---

## Events Emitted

| Event | Trigger | Payload |
|---|---|---|
| `sleepwalker:anomaly_detected` | Probe fires above severity threshold | `{ domain, anomalyType, severity, tickId }` |
| `sleepwalker:artifact_published` | Insight pushed to Styx Lethe | `{ artifactType, confidence, namespace, priceUsd }` |
| `sleepwalker:regime_shift` | Regime classification changes | `{ fromRegime, toRegime, confidence }` |
| `sleepwalker:death_study_extracted` | Death testament processed | `{ sourceGolemId, insightsExtracted, validationRecipe }` |
| `sleepwalker:dream_triggered` | Hypothesis density or anomaly cluster triggers dream | `{ triggerType, hypothesisCount }` |
| `sleepwalker:artifact_purchased` | Another agent buys an artifact | `{ artifactId, buyerGolemId, priceUsd }` |

---

## Cross-References

- Tool implementations: `../07-tools/07-tools-advanced.md` (Sleepwalker Observatory Tools section)
- Styx Marketplace commerce: `../../tmp/research/styx-interation2/S5-marketplace.md`
- Styx Lethe layer: `../../tmp/research/styx-interation2/S2-styx-api-revenue.md`
- Agent definition: `../19-agents-skills/12-observer-agents.md`
- Death study pipeline: `../02-mortality/06-thanatopsis.md`
- ERC-8004 discovery: `../09-economy/00-identity.md`
- Coordination Council (multi-Sleepwalker): `../09-economy/04-coordination.md` (Pattern 5.11)
- Capability-gated tools and trust tiers: `07-safety.md` §2
- Sleepwalker tool profile configuration: `11-tools.md` §6
