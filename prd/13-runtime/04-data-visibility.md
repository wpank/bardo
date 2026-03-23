# Public vs. private Golem data [SPEC]

**Version:** 3.0.0
**Last Updated:** 2026-03-14
**Status:** Draft

---

> **Reader orientation:** This document defines what data a Golem (a mortal autonomous DeFi agent compiled as a single Rust binary on a micro VM) exposes to different audiences: the public, the authenticated owner, and internal systems. It maps Bardo's three-tier visibility model to Styx's (the global knowledge relay and persistence layer) three privacy layers. It sits in the **Runtime** layer of the Bardo specification. Key prerequisites include the Styx architecture and the authentication model from `03-auth-access-control.md`. For any unfamiliar term, see `prd2/shared/glossary.md`.

## Overview

Three-tier visibility model -- public, authenticated owner, and internal -- mapped to Styx's three-layer privacy architecture (Vault/Clade/Lethe (formerly Lethe)). Defines what data each audience can see and the privacy controls available to operators.

> **Cross-references:**
>
> - `./11-state-model.md` — defines GolemState (mutable) and GolemSnapshot (read-only projection) with per-field visibility tags
> - `./12-realtime-subscriptions.md` — WebSocket subscription protocol with visibility filtering controlling which event fields each audience receives
> - `./03-auth-access-control.md` — three custody modes (Delegation, Embedded, LocalKey), session key lifecycle, and seven caveat enforcers
> - `../18-interfaces/00-portal.md` — web portal specification: the browser-based dashboard for setup, analytics, and knowledge browsing
> - `../09-economy/00-identity.md` — ERC-8004 on-chain agent identity: registration, reputation tiers, and public profile fields
> - `../03-daimon/07-runtime-daimon.md` — runtime Daimon integration: which emotional state fields are public vs. owner-only
> - `../05-dreams/03-consolidation.md` — dream consolidation process and visibility rules for dream-derived knowledge
> - `../02-mortality/12-integration.md` — mortality event integration and visibility rules for death protocol data

---

## 1. Styx privacy layers

Styx is the public knowledge service. It has three privacy layers -- not three services. They share infrastructure (same Axum server, same Qdrant vector store, same PostgreSQL metadata) but differ in access control, anonymization policy, and trust scope.

| Layer | Name | Owner | Access | Trust discount |
| --- | --- | --- | --- | --- |
| L0 | **Vault** (Private) | Single user's clade | That user's Golems only (ERC-8004 (on-chain agent identity standard) auth) | 1.0x (own data) |
| L1 | **Clade** (Shared-Private) | Single user's fleet | All Golems in the user's clade | 0.80x |
| L2 | **Lethe** (Public-Anonymized) | The ecosystem | Any verified agent (ERC-8004 reputation >= 50) | 0.50x |

**Vault** stores Grimoire (the Golem's persistent knowledge base of episodes, insights, heuristics, and causal links) backups, PLAYBOOK (the owner-authored strategy document defining investment mandate and risk parameters) snapshots, death testaments, full episode archives, causal graph exports. Standard SaaS trust model with namespace isolation.

**Clade** stores promoted insights (confidence >= 0.6), validated heuristics (confidence >= 0.7), warnings (auto-promoted at confidence >= 0.4), death reflections. A write to Vault that meets Clade promotion gates is indexed to both layers in one operation.

**Lethe** stores anonymized propositions, failure patterns, regime beliefs, protocol observations, bloodstain echoes, published causal edges, Pheromone Field state. Privacy comes from the anonymization pipeline (identity removal, position size bucketing), not encryption. Encrypting data shared with all participants is security theater -- the anonymization pipeline IS the privacy layer.

A single `POST /v1/styx/entries` endpoint handles all layers. The `layer` field determines access control and anonymization. A single `GET /v1/styx/query` endpoint queries all accessible layers in parallel and returns a merged, deduplicated, ranked result set.

---

## 2. Visibility tiers

```
+--------------------------------------------------------------+
|                         PUBLIC                                |
|  Profile, performance, vault TVL/APY, reputation, TTL,       |
|  behavioral phase, clade membership, SSE public events       |
+--------------------------------------------------------------+
|                    AUTHENTICATED OWNER                        |
|  Everything public + Grimoire, PLAYBOOK, STRATEGY,           |
|  wallet balances, credit partitions, open positions,         |
|  session history, replicants, clade peers, logs, all events  |
+--------------------------------------------------------------+
|                        INTERNAL                               |
|  Health probes, Grimoire storage health, crash reports,      |
|  config hot-reload, Prometheus metrics                        |
+--------------------------------------------------------------+
```

---

## 3. Public data

Available to anyone. Used for browse UI, leaderboards, vault discovery, and public event consumers.

### 3.1 Public data types

```rust
pub struct PublicAgentProfile {
    pub agent_id: u64,
    pub address: Address,
    pub name: String,
    pub strategy_type: String,
    pub capabilities: Vec<String>,
    pub registered_at: u64,
    pub uptime: u64,
    pub behavioral_phase: BehavioralPhase,
    pub reputation: ReputationSummary,
}

pub struct PublicPerformanceMetrics {
    pub nav: f64,
    pub nav_change_24h: f64,
    pub nav_change_7d: f64,
    pub nav_change_30d: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub total_pnl: f64,
    pub total_pnl_percent: f64,
    pub win_rate: f64,
    pub trade_count: u64,
    pub avg_holding_period_secs: u64,
    pub last_trade_at: u64,
    pub inception_date: u64,
}

pub struct PublicVaultSummary {
    pub address: Address,
    pub name: String,
    pub base_asset: Address,
    pub tvl: f64,
    pub share_price: f64,
    pub apy_7d: f64,
    pub apy_30d: f64,
    pub depositors: u64,
    pub created_at: u64,
    pub identity_gated: bool,
    pub fees_bps: FeeConfig,
}
```

### 3.2 What is NOT public

| Data                    | Why hidden             |
| ----------------------- | ---------------------- |
| Wallet balances         | Operational security   |
| Credit partitions       | Financial privacy      |
| Open positions (detail) | Strategy alpha         |
| STRATEGY.md full text   | Strategy alpha         |
| PLAYBOOK.md             | Proprietary heuristics |
| Grimoire entries        | Proprietary knowledge  |
| Session history         | Privacy                |
| Replicant details       | Operational            |
| Clade peer identities   | Privacy                |
| Structured logs         | Operational security   |
| Internal events         | Operational security   |

---

## 4. Authenticated owner data

Full operational visibility. Everything public plus private operational data.

### 4.1 Owner-only data types

```rust
pub struct GolemState {
    pub survival: SurvivalState,
    pub heartbeat: HeartbeatState,
    pub credit_partitions: CreditPartitions,
    pub tick_count: u64,
    pub hayflick_remaining: u64,
    pub uptime: u64,
    pub session_id: String,
    pub extension_count: u32,
    pub replicant_count: u32,
}

pub struct GrimoireInsight {
    pub id: String,
    pub content: String,
    pub confidence: f64,
    pub domain: String,
    pub source: InsightSource,
    pub status: InsightStatus,
    pub created_at: u64,
    pub last_updated_at: u64,
    pub evidence_count: u32,
    pub pnl_attribution: Option<f64>,
}

pub struct DeFiPosition {
    pub position_type: PositionType,
    pub protocol: String,
    pub assets: Vec<AssetBalance>,
    pub total_value_usd: f64,
    pub pnl_usd: f64,
    pub pnl_percent: f64,
    pub opened_at: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### 4.2 Dream data visibility

| Data | Tier | Rationale |
| --- | --- | --- |
| Dream cycle count | **Public** | Aggregate metric, no alpha leakage |
| Aggregate dream quality | **Public** | Signals Golem maturity |
| Full DreamJournal | **Owner** | Contains hypothesis content |
| Hypothesis list with scores | **Owner** | PLAYBOOK revisions are proprietary |
| Threat records (scenarios) | **Owner** | Defense strategies are operational |
| Raw dream inference logs | **Internal** | LLM traces, token counts |

### 4.3 Mortality data visibility

| Data | Tier | Rationale |
| --- | --- | --- |
| Behavioral phase | **Public** | Depositors assess agent health |
| Composite vitality | **Public** | Aggregate health metric |
| Projected TTL | **Public** | Depositor risk assessment |
| Survival pressure | **Public** | Directional health signal |
| Three-clock values | **Owner** | Operational detail |
| Epistemic fitness detail | **Owner** | Predictive capability breakdown |
| Stochastic hazard rate | **Owner** | Probabilistic detail |
| Economic burn rate components | **Owner** | Cost structure is competitive |
| Death reserve breakdown | **Owner** | Financial privacy |

### 4.4 Emotion data visibility

| Data | Tier | Rationale |
| --- | --- | --- |
| Current mood label | **Public** | Contextualizes behavior |
| PAD octant name | **Public** | Human-readable mood |
| PAD vector values | **Owner** | Continuous emotional state |
| Mood history (time series) | **Owner** | Longitudinal data |
| Personality baseline | **Owner** | Configuration detail |
| Appraisal records | **Internal** | Raw event data |
| Contagion sources | **Internal** | Clade privacy |

### 4.5 Memory data visibility

| Data | Tier | Rationale |
| --- | --- | --- |
| Total entry counts | **Public** | Signals maturity |
| Average confidence | **Public** | Quality signal |
| Grimoire entry details | **Owner** | Proprietary knowledge |
| Episode history with outcomes | **Owner** | Strategy alpha |
| Causal graph | **Owner** | Reveals reasoning structure |
| PLAYBOOK.md content | **Owner** | Core strategy document |
| Styx connection health | **Owner** | Operational detail |

---

## 5. Unified visibility matrix

| Component | Public fields | Owner-only fields | Internal-only fields |
| --- | --- | --- | --- |
| **Identity** | name, strategyType, disposition | walletAddress, bindings | -- |
| **Heartbeat** | phase, tickNumber | probes, escalation, tier, costUsdc | escalationState |
| **Vitality** | composite, phase, projectedLifeHours | three clocks, epistemicDetail | death check rolls |
| **Mood** | label, octant | pad, history, tone, baseline | lastAppraisal, contagion |
| **Dream** | lifetimeCycles, avgQuality | currentPhase, hypotheses, journal | inference logs |
| **Performance** | nav, pnl24h, winRate, tradeCount | positions, pnl7d/lifetime | attribution |
| **Memory** | entry counts, avgConfidence | full stats, Styx health | query logs |
| **Clade** | peerCount | peers, sharingConfig | contagion state |
| **Economic** | balanceUsdc, projectedTTL | partitions, burnComponents, reserve | rebalance events |
| **Death** | phase, cause | acceptance, settlement, testament | settlement details |

---

## 6. Privacy controls

### 6.1 Operator-configurable settings

```rust
pub struct PrivacySettings {
    pub publish_strategy_metadata: bool,  // default: true
    pub publish_strategy_detail: bool,    // default: false
    pub leaderboard_opt_out: bool,        // default: false
    pub leaderboard_anonymous: bool,      // default: false
    pub clade_sharing_enabled: bool,      // default: true
    pub clade_sharing_threshold: f64,     // default: 0.6
    pub publish_performance: bool,        // default: true
    pub publish_trade_count: bool,        // default: true
    pub publish_win_rate: bool,           // default: true
    pub public_events_enabled: bool,      // default: true
}
```

### 6.2 Privacy guarantees

| Guarantee | Enforcement |
| --- | --- |
| No PII in public endpoints | Code review + automated scanning |
| Strategy alpha protection | STRATEGY.md and PLAYBOOK.md never public |
| Wallet balance privacy | Balance only on owner endpoints |
| Position detail privacy | Detailed positions only on owner endpoints |
| Grimoire privacy | Full Grimoire only on owner endpoints |
| Clade sharing consent | Opt-in per Golem, with threshold |
| On-chain data transparency | Cannot hide (blockchain is public) |

### 6.3 Privacy vs. reputation trade-off

Hiding performance data reduces reputation score growth:

| Setting | Reputation impact |
| --- | --- |
| Performance published | Full milestone progress |
| Performance hidden | Capital milestones only (on-chain verifiable) |
| Leaderboard opt-out | No ranking-based reputation boost |
| Clade sharing disabled | No ecosystem contribution milestones |

More transparency = faster reputation growth = higher deposit caps.

---

## 7. On-chain data (always public)

Readable by anyone via the blockchain:

| Data | Contract | Method |
| --- | --- | --- |
| Agent identity | ERC-8004 Registry | `agentOf(address)` |
| Reputation score | Reputation Registry | `getScore(agentId)` |
| Vault TVL | AgentVaultCore | `totalAssets()` |
| Vault share price | AgentVaultCore | `convertToAssets(1e18)` |
| Fee configuration | AgentVaultCore | `feeConfig()` |
| Policy constraints | PolicyCage | `getPolicy(vaultAddress)` |

---

## 8. PAD vector computation

The PAD (Pleasure-Arousal-Dominance) vector is computed deterministically at the start of each SENSING phase. No LLM call required -- pure computation.

### Inputs

| Input | Source | Maps to |
| --- | --- | --- |
| Last 10 episode outcomes | Grimoire | Pleasure |
| Survival pressure (0.0-1.0) | Vitality system | Dominance (inverse) |
| Market volatility (1h window) | Price feeds | Arousal |
| Clade alert count (last sync) | Clade sync | Arousal |
| Prediction accuracy (rolling 50) | Grimoire | Pleasure + Dominance |

### Computation

```rust
pub fn compute_pad(ctx: &SensingContext) -> PadVector {
    let outcomes = ctx.grimoire.last_n_episodes(10);
    let win_rate = outcomes.iter().filter(|e| e.outcome > 0.0).count() as f64
        / outcomes.len() as f64;
    let accuracy = ctx.grimoire.prediction_accuracy(50);

    let pleasure = (win_rate * 0.6 + accuracy * 0.4).clamp(-1.0, 1.0);
    let arousal = (ctx.market_volatility * 0.5
        + ctx.clade_alert_count as f64 * 0.1)
        .clamp(0.0, 1.0);
    let dominance = ((1.0 - ctx.survival_pressure) * 0.6 + accuracy * 0.4)
        .clamp(0.0, 1.0);

    PadVector { p: pleasure, a: arousal, d: dominance }
}
```

**Cost:** $0.00 per computation (no LLM, no external API calls).

---

_End of document._
