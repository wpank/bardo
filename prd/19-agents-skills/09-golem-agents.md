# Golem archetypes -- the autonomous core [SPEC]

> **Cross-ref**: [00-agents-overview.md](00-agents-overview.md) (architecture overview for archetypes and tool profiles), [01-agent-categories.md](01-agent-categories.md) (full inventory of 42+ archetypes across 14 categories), [../01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) (the 9-step Heartbeat decision cycle specification), [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Pi extension loading and tool registration)

> **Reader orientation:** This document specifies the three Golem (mortal autonomous agent) archetypes that form the autonomous core: `golem-instance` (the Golem itself, running the Heartbeat loop), `memory-consolidator` (Grimoire lifecycle and PLAYBOOK.md evolution), and `heartbeat-monitor` (health observation and mortality metrics). It belongs to Section 19 (Agents & Skills). Unlike specialist archetypes that receive parameters and return results, Golem archetypes ARE the autonomous entities -- they run continuously, evolve their reasoning, and die when resources run out. See `prd2/shared/glossary.md` for full term definitions.

---

## Overview

Golem archetypes are fundamentally different from specialist archetypes. Specialist archetypes (trade-executor, pool-researcher, etc.) are invoked to perform a task and return a result. Golem archetypes ARE the autonomous entities -- they run continuously in a Pi session, make strategic decisions, evolve their reasoning, and die when their resources run out.

The three golem archetypes form the cybernetic core:

| Archetype             | Role                                    | Invocation pattern          |
| --------------------- | --------------------------------------- | --------------------------- |
| `golem-instance`      | The golem itself: heartbeat, strategy   | Autonomous (daemon loop)    |
| `memory-consolidator` | Grimoire lifecycle, knowledge evolution | Periodic (every 50 ticks)   |
| `heartbeat-monitor`   | Health observation, mortality metrics   | Continuous (background)     |

---

## golem-instance

**Default model:** opus

**Role:** The `golem-instance` archetype IS the golem. It runs a continuous heartbeat loop via the `bardo-heartbeat` extension, reads its PLAYBOOK.md to determine strategic priorities, evaluates market conditions through deterministic probes, decides which specialist archetypes to delegate to, and evolves its reasoning based on outcomes.

### How it differs from specialist archetypes

Specialist archetypes are reactive: they receive parameters and return results. The golem-instance is proactive: it reads its own evolving strategic context (PLAYBOOK.md, HEARTBEAT.md, STRATEGY.md files), decides what to do, does it, reflects on the outcome, and updates its strategic context. This is the cybernetic feedback loop that makes golems autonomous.

### Heartbeat loop

The heartbeat is the master execution pattern. All specialist archetype invocations originate from it.

```text
1. PROBE
   - Read PLAYBOOK.md (learned heuristics, regime-specific rules)
   - Read HEARTBEAT.md (operator standing orders)
   - Evaluate all active STRATEGY.md files
   - For each strategy: check trigger conditions
   - Suppression: if no triggers fire, skip to next tick (cost: $0.00)

2. ESCALATE
   - For triggered strategies: run escalation gate
   - MUST constraints checked pre-LLM (hard guards)
   - If escalation gate passes: proceed to LLM decision

3. DECIDE
   - LLM reads: PLAYBOOK.md + strategy context + market data from tools
   - Decides: which action to take, which archetypes to delegate to
   - SHOULD/MAY constraints inform but do not bind the decision

4. EXECUTE
   - Delegate to specialist archetypes:
     - trade-executor for swaps
     - liquidity-manager for LP operations
     - vault-manager for vault operations
   - Each delegation flows through the ActionPermit system

5. REFLECT (double loop)
   - After execution: compare predicted vs actual outcome
   - Generate structured self-reflection (Reflexion pattern)
   - Produce delta entries for PLAYBOOK.md (not full rewrites)
   - Store episodic memory via memory-consolidator

6. CURATE (meta loop -- every 50 ticks)
   - Trigger memory-consolidator to integrate accumulated deltas
   - Restructure PLAYBOOK.md based on accumulated experience
   - Second-order cybernetics: the system observes itself
```

### PLAYBOOK.md

The PLAYBOOK.md is a living document that the golem reads at the start of every tick and updates after outcomes:

- **Learned heuristics**: Rules derived from execution experience
- **Regime-specific rules**: Different behavior for bull/bear/sideways markets
- **Strategic principles**: High-level guidelines distilled from accumulated learning
- **Version history**: Every change tracked for rollback

The PLAYBOOK.md is machine-evolved, not human-authored. Human directives go in HEARTBEAT.md. The `bardo-context` extension assembles the relevant portions into the ContextBundle for each inference call.

### STRATEGY.md files

Strategies are operator-authored or NL-compiled configuration documents:

```text
~/.bardo/strategies/
  eth-dca-weekly/STRATEGY.md
  rsi-mean-revert/STRATEGY.md
  delta-neutral-lp/STRATEGY.md
```

Each strategy specifies:

- **Schedule**: When to evaluate (cron, time windows, duration bounds)
- **Trigger**: Conditions that activate execution (RSI levels, delta drift, schedule)
- **Action**: What to do when triggered (swap, rebalance, enter/exit position)
- **Constraints**: MUST/SHOULD/MAY rules mapped to enforcement levels [RFC-2119]
- **Risk bounds**: Hard limits (max drawdown, stop-loss, max slippage)
- **Completion**: When the strategy expires (budget exhausted, target reached, idle timeout)

The `bardo-compiler` extension parses STRATEGY.md into `StrategyParams` objects at boot and on hot-reload.

### Delegation targets

The golem-instance delegates to specialist archetypes but never receives delegation from other archetypes:

| Archetype delegated to | When                                              |
| ---------------------- | ------------------------------------------------- |
| `trade-executor`       | Strategy requires a swap operation                |
| `liquidity-manager`    | Strategy requires LP add/remove/rebalance         |
| `vault-manager`        | Strategy involves vault deposits or management    |
| `vault-strategist`     | Strategy needs vault analysis or recommendations  |
| `lp-strategist`        | Strategy needs LP range optimization              |
| `risk-assessor`        | Pre-execution risk validation                     |
| `memory-consolidator`  | Periodic knowledge consolidation (every 50 ticks) |

### Safety behavior

- Runs in DRY RUN mode by default. Live execution requires explicit `--live` flag.
- MUST constraints in STRATEGY.md enforced pre-LLM via escalation gates.
- Every write operation flows through the ActionPermit system (preview -> commit).
- Risk bounds (max drawdown, stop-loss) are hard limits that cannot be overridden by playbook evolution.
- PLAYBOOK.md evolution cannot weaken safety constraints -- the `bardo-safety` extension validates this on every delta.

### Tool access

The golem-instance uses the `full` profile. All 186+ tools are available through the two-layer adapter system. The LLM sees 8 Pi-facing tools; behind them, the adapter resolves to concrete `golem-tools` implementations.

---

## memory-consolidator

**Default model:** opus

**Role:** Manages the Grimoire knowledge lifecycle. Consolidates episodic memories into semantic insights, evolves the golem's PLAYBOOK.md, and maintains the knowledge graph that makes accumulated experience durable and transferable.

### Grimoire architecture

| Layer        | Storage     | Contents                                      | Lifetime         |
| ------------ | ----------- | --------------------------------------------- | ---------------- |
| Episodes     | LanceDB     | Individual execution records with reflections | Permanent        |
| Insights     | SQLite      | Aggregated patterns from multiple episodes    | Confidence-decay |
| Heuristics   | PLAYBOOK.md | Strategic rules derived from insights         | Evolving         |
| Warnings     | SQLite      | Failure patterns and anti-patterns            | Permanent        |
| Causal links | SQLite      | Condition -> outcome relationships            | Confidence-decay |

### Consolidation cycle

Runs every 50 heartbeat ticks (or on explicit invocation):

```text
1. SCAN EPISODES
   - Query recent episodes since last consolidation
   - Cluster by common features: token pair, market regime, strategy type
   - Identify patterns: repeated successes, repeated failures

2. EXTRACT INSIGHTS
   - For clusters with 5+ episodes: extract semantic insight
   - Calculate confidence score from sample size and consistency
   - Insights with confidence > 0.8 and sample size > 30 are "publishable"

3. GENERATE DELTAS
   - Compare new insights against existing PLAYBOOK.md heuristics
   - Produce delta entries (additions, modifications, removals)
   - Use ACE Generator-Reflector-Curator pattern [ZHANG-ICLR2026]
   - Deltas are incremental -- never rewrite the full PLAYBOOK.md

4. INTEGRATE
   - Apply deltas to PLAYBOOK.md
   - Version the update (git-style diff tracking)
   - Validate: no safety constraint weakening, no unbounded parameter drift

5. DECAY
   - Reduce confidence of insights not reinforced by recent episodes
   - Archive insights below confidence threshold (0.3)
   - Ensure the Grimoire reflects current market conditions, not stale history
```

### Tool access

Uses the `memory` tool category: `search_episodes`, `store_episode`, `search_insights`, `update_heuristic_confidence`, `get_memory_stats`, `trigger_curator_cycle`, `export_grimoire`, `get_episode_by_id`, `link_causal_nodes`.

### Delegation

None. Terminal knowledge node. Other archetypes write episodes to the Grimoire; the memory-consolidator reads and consolidates.

### Safety behavior

- NEVER modify risk bounds or safety constraints in PLAYBOOK.md
- ALWAYS version PLAYBOOK.md changes for rollback
- REFUSE to consolidate insights from fewer than 5 supporting episodes
- Parameter changes bounded: no insight can shift any parameter by more than 10% per consolidation cycle

---

## heartbeat-monitor

**Default model:** sonnet

**Role:** Continuous health observation. Monitors the golem's operational vitals: USDC balance trajectory, heartbeat regularity, strategy execution rates, error frequencies, and mortality metrics. Terminal observation node -- dispatches health reports but never modifies golem state.

### Monitored metrics

| Metric                | Source                | Alert threshold                      |
| --------------------- | --------------------- | ------------------------------------ |
| USDC balance          | `get_account_balance` | Below 30-day runway projection       |
| Heartbeat regularity  | Tick timestamps       | >2x expected interval between ticks  |
| Strategy success rate | Episode outcomes      | Below 40% success rate over 20 ticks |
| Error frequency       | Error logs            | >5 errors per 10 ticks               |
| Gas efficiency        | Transaction receipts  | Gas costs >15% of gross income       |
| Playbook staleness    | PLAYBOOK.md mtime     | No updates for 100+ ticks            |

### Mortality integration

The heartbeat-monitor tracks the golem's distance from mortality conditions:

- **USDC depletion**: Projects runway based on burn rate and income
- **Hayflick limit**: Tracks total heartbeats against configured maximum
- **Staleness**: Monitors whether the golem is making meaningful strategic progress

When mortality metrics enter warning zones, the heartbeat-monitor emits structured `ProbeAlert` events on the event bus. The golem-instance reads these during its next probe phase and may adjust strategy (reduce spending, increase income-seeking) in response.

### Tool access

Uses the `data` tool category. Reads health metrics via `get_golem_health`, `get_mortality_state`, `get_vitality_score`, `get_heartbeat_stats`, `get_token_balance`, `get_wallet_address`.

### Delegation

None. Terminal observation node. Reads golem state but never modifies it. Health reports are written to the event bus as `ProbeAlert` events.

### Safety behavior

- NEVER execute transactions or modify golem state
- ALWAYS include quantitative metrics in health reports
- BATCH alerts to prevent notification spam (5-minute windows)
- LOG all metric readings for audit trail

---

## How golem archetypes differ from specialist archetypes

| Dimension   | Specialist archetypes           | Golem archetypes                             |
| ----------- | ------------------------------- | -------------------------------------------- |
| Invocation  | Delegated to by other archetypes| Autonomous (daemon loop)                     |
| Lifecycle   | Request-response                | Continuous heartbeat                         |
| State       | Stateless between invocations   | Persistent (PLAYBOOK.md, Grimoire)           |
| Learning    | None                            | Playbook evolves strategic reasoning         |
| Identity    | Fungible (any instance works)   | Unique (accumulated knowledge, identity)     |
| Mortality   | N/A (always available)          | Mortal (USDC depletion, Hayflick limit)      |
| Composition | Composed by golems              | Composes specialist archetypes               |

The golem is not a user of archetypes -- it IS an archetype that uses other archetypes. The heartbeat loop is the master composition pattern from which all other archetype invocations derive.

---
