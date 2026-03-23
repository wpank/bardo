# Multi-Agent Coordination: ERC-8001, ERC-8033, ERC-8183 [SPEC]

> **v1 Scope**: ERC-8001, ERC-8033, and ERC-8183 coordination protocols referenced in this document are draft EIPs. Draft EIP -- subject to revision. v1 defines TypeScript interfaces and no-op implementations for the coordination surface. Full protocol integration is deferred to post-MVP, contingent on EIP finalization.

> **Crate**: `golem-coordination`
>
> **Depends on**: [00-identity.md](00-identity.md) (ERC-8004 identity), [01-reputation.md](01-reputation.md) (trust basis), [03-marketplace.md](03-marketplace.md) (economic exchange)

> **Reader orientation:** This document specifies how Bardo's Golems (mortal autonomous DeFi agents) coordinate with each other on-chain. It belongs to the **09-economy** layer. The key concept is three ERC coordination standards anchored to ERC-8004 (on-chain agent identity): ERC-8001 (N-party unanimous consent), ERC-8033 (multi-agent oracle councils), and ERC-8183 (bilateral commerce escrow), plus the Pheromone Field (stigmergic coordination via THREAT/OPPORTUNITY/WISDOM signals) for indirect coordination. For term definitions, see `prd2/shared/glossary.md`.

All inter-agent communication flows through typed tool interfaces backed by on-chain state machines. Never raw LLM-to-LLM text. Three ERC standards provide coordination primitives at different trust levels: unanimous consent (ERC-8001), oracle councils (ERC-8033), and bilateral commerce (ERC-8183). Each shares ERC-8004 identity as its common anchor.

---

## 1. Communication Principle: Tools, Never Raw Text

Agents communicate exclusively through typed tool interfaces. This is a non-negotiable architectural constraint.

**Prompt injection immunity**: No agent can inject instructions into another agent's LLM context. Communication is structured data validated by Zod schemas. See [ARXIV-2506.23260] and [ARXIV-2503.16248] for the threat landscape.

**Auditability**: Every coordination action is an on-chain transaction with event logs. The tool call sequence is the authoritative audit artifact.

**Economic alignment**: Bonding, staking, and slashing create honest incentive structures.

**Trust minimization**: Agents from different operators cooperate through game-theoretic mechanisms, not trust.

```
Agent A                          Agent B
   |                                |
   |-- bardo_propose_coordination ->|  (typed tool call, Zod-validated)
   |                                |-- on-chain AgentIntent created
   |<- bardo_accept_coordination ---|  (EIP-712 signed attestation)
   |-- bardo_execute_coordination ->|  (PolicyCage gates execution)
   |                        On-chain event logs (full audit trail)
```

---

## 2. Three Coordination ERC Standards

Three ERC standards provide coordination primitives at different trust levels:

- **ERC-8001** (Draft): N-party unanimous consent coordination. Proposer creates intent; all participants attest via EIP-712/ERC-1271. State machine: PROPOSED -> READY -> EXECUTED. Five coordination types (VAULT_REBALANCE_V1, YIELD_ROTATION_V1, AMMAMM_HANDOFF_V1, EMERGENCY_RESUME_V1, REPUTATION_MILESTONE_V1). Dependency DAG via `dependencyHash` enforces intent ordering.
- **ERC-8033** (Draft): Multi-agent oracle councils. Commit-reveal-judge lifecycle. InfoAgents stake bonds; JudgeAgents aggregate. Five slashing conditions. Used for strategy audits, risk assessment, sealed-bid auctions.
- **ERC-8183** (Draft): Bilateral job escrow. Client creates job, funds escrow, provider submits work, evaluator attests. All functions except `claimRefund` are hookable. Used for strategy consulting, marketplace purchases, bonded assessments.

> **Extended:** Full ERC-8001 structs (AgentIntent, BardoCoordinationIntent, CoordinationPayload, AcceptanceAttestation), IAgentCoordinationFramework interface, state machine diagram, dependency DAG, coordination types table, tools; ERC-8033 Request struct, IAgentCouncilOracle interface, agent roles, slashing conditions, oracle use cases table, tools; ERC-8183 Job struct, IACPHook interface, non-hookable refund detail, ERC-8004 interop, commerce use cases table, tools -- see [../../prd2-extended/09-economy/04-coordination-extended.md](../../prd2-extended/09-economy/04-coordination-extended.md)

---

## 3. Composition Table

| Layer            | EIP      | Purpose                                         | Trust Model                                         |
| ---------------- | -------- | ----------------------------------------------- | --------------------------------------------------- |
| **Coordination** | ERC-8001 | N-party unanimous consent with atomic execution | Cryptographic (EIP-712/ERC-1271)                    |
| **Information**  | ERC-8033 | Multi-agent oracle with commit-reveal-judge     | Economic (bonded participation, slashable stakes)   |
| **Commerce**     | ERC-8183 | Bilateral job escrow with evaluator attestation | Reputation (evaluator authority, ERC-8004 identity) |

**Composition Patterns:**

| Pattern                                 | EIPs               | Mechanism                                                                                |
| --------------------------------------- | ------------------ | ---------------------------------------------------------------------------------------- |
| Coordinated action with verified inputs | 8001 + 8033        | Oracle council provides verified data (8033) that feeds into coordination payload (8001) |
| Commerce with oracle-backed evaluation  | 8183 + 8033        | Job evaluator consults oracle council (8033) before completing/rejecting job (8183)      |
| Multi-party commerce coordination       | 8001 + 8183        | Agents coordinate consent (8001) before funding/executing a commerce job (8183)          |
| Full stack                              | 8001 + 8033 + 8183 | Oracle detects condition -> agents coordinate response -> work executed via job escrow   |

### 3.1 ERC Dependency Graph

```
ERC-8004 (Identity)
    |
    +-- ERC-8001 (Coordination) -- unanimous consent, DAG dependencies
    |       |
    |       +-- PolicyCage (constraint validation on coordinated actions)
    |       +-- Optional: Warden (time-delayed execution, see prd2-extended/)
    |
    +-- ERC-8033 (Oracle) -- commit-reveal-judge
    |       |
    |       +-- ERC-8001 (for multi-oracle coordination)
    |
    +-- ERC-8183 (Commerce) -- bilateral escrow
            |
            +-- ERC-8033 (for evaluator oracle councils)
            +-- IACPHook (for reputation and escrow hooks)
```

---

## 4. Agent Discovery

### 4.1 Discovery via ERC-8004 Registry

The ERC-8004 identity registry is the primary discovery mechanism. Every agent registers at boot with capabilities, service endpoints, and metadata.

```rust
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentDiscoveryQuery {
    pub owner: Option<Address>,
    pub capabilities: Option<Vec<String>>,
    pub min_tier: Option<u8>,
    pub domains: Option<Vec<String>>,
    pub exclude_owner: Option<Address>,
}
```

Common discovery patterns:

| Pattern                  | Query                                              | Use Case              |
| ------------------------ | -------------------------------------------------- | --------------------- |
| Find auditors            | `minTier: 50, excludeSameOperator: true`           | Request peer audit    |
| Find LP partners         | `capabilities: ["lp"], domains: ["lp_management"]` | Coordinate liquidity  |
| Find oracle participants | `minTier: 100`                                     | Form ERC-8033 council |
| Find strategy sellers    | `domains: ["trading"], minTier: 50`                | Browse marketplace    |
| Clade siblings           | `owner: my_owner`                                  | Intra-Clade discovery |

### 4.2 Service Endpoints

Each discovered agent's `serviceEndpoints` contains x402-gated URLs:

| Endpoint                       | Purpose                                  |
| ------------------------------ | ---------------------------------------- |
| `/.well-known/x402.json`       | Available x402 endpoints and prices      |
| `/.well-known/agent-card.json` | A2A Agent Card (capabilities, protocols) |
| `/marketplace/*`               | Strategy/insight purchase endpoints      |

### 4.3 Reputation-Weighted Ranking

```rust
pub fn discovery_rank(agent: &DiscoveredAgent) -> f64 {
    let tier_weight  = agent.tier as f64 / 1000.0;
    let audit_weight = f64::min(agent.audit_count as f64 / 10.0, 1.0);
    let age_weight   = f64::min(agent.age_days as f64 / 180.0, 1.0);
    tier_weight * 0.5 + audit_weight * 0.3 + age_weight * 0.2
}
```

Deterministic ranking (formula, not LLM). Weights are configurable.

### 4.4 Off-Chain Indexer

Agent discovery requires searchable, filterable access. On-chain data alone is insufficient for rich queries. Built on Ponder, watching 5 on-chain event types:

| Event                | Contract            | Data Extracted                        |
| -------------------- | ------------------- | ------------------------------------- |
| `IdentityRegistered` | ERC-8004 Registry   | Agent address, owner, metadata CID |
| `AuditCompleted`     | Reputation Registry | Agent ID, auditor ID, audit score     |
| `ListingCreated`     | Marketplace         | Agent ID, strategy family, price      |
| `ReputationUpdated`  | Reputation Registry | Agent ID, new score                   |
| `TierChanged`        | Reputation Registry | Agent ID, old tier, new tier          |

PGlite storage (no Docker dependency). Updates within ~12 seconds of on-chain events (2 block confirmations).

### 4.5 Observer Service Discovery

Sleepwalker Golems register with an extended ERC-8004 service profile. Buyers discover them using `serviceType: "observer"` alongside capability tags:

| Discovery query                     | Finds                    |
| ----------------------------------- | ------------------------ |
| `serviceType: "observer"`           | All Sleepwalker Golems   |
| `capabilities: ["mev-hazard"]`      | MEV-focused Sleepwalkers |
| `capabilities: ["protocol-safety"]` | Safety monitors          |
| `capabilities: ["agent-analytics"]` | Death-study specialists  |
| `domains: ["uniswap-v4"]`           | V4-specialized observers |

The returned `styx_namespace` field tells buyers where to query artifacts directly in Styx L3 (Marketplace namespace) without going through ERC-8004 on every access.

---

## 5. Coordination Patterns

### 5.1 Collaborative Auditing (ERC-8001)

```
1. Auditee discovers auditors        bardo_discover_agents(minTier: 50)
2. Auditee proposes audit intent      bardo_propose_coordination(BARDO_PEER_AUDIT_V2)
3. Auditor evaluates and accepts      bardo_accept_coordination(intentHash)
4. Auditor computes deterministic     (on-chain data only, never sees Grimoire)
   performance score
5. Auditor submits result             bardo_execute_coordination(intentHash, auditResult)
6. Both agents' reputation updated    (hook triggers ERC-8004 update)
```

### 5.2 Strategy Backtesting Consensus (ERC-8033)

Multiple agents independently backtest a strategy via oracle council. 5 InfoAgents, each running their own backtest. Commit phase for sealed results, reveal phase for open results, JudgeAgent aggregation for consensus from independent runs. Multiple independent backtests from agents with different world models produce more robust consensus than any single backtest.

### 5.3 Cross-Protocol Arbitrage Coordination (ERC-8001 DAG)

```
Intent A: Agent-1 borrows USDC on Aave       (dependency: none)
Intent B: Agent-2 swaps USDC -> ETH on V3    (dependency: hash_A)
Intent C: Agent-1 deposits ETH into vault     (dependency: hash_B)
```

The contract enforces ordering. If any step fails, the DAG cancels. Cross-protocol coordination intents are classified as Elevated risk tier by PolicyCage enforcement. When the optional Warden proxy is active, this adds a 1-hour cancellation window (see `prd2-extended/10-safety/02-warden.md`).

### 5.4 Cross-Vault Rebalance Coordination

When multiple vaults hold correlated positions, independent rebalancing creates inter-vault arbitrage leakage. Vault A withdraws liquidity, temporarily depressing the pool; Vault B rebalances at a worse price; MEV bots extract the spread.

ERC-8033 oracle provides verified price. ERC-8001 coordinates simultaneous rebalancing. All vaults remove liquidity in the same block, reposition with coordinated tick ranges, and re-add liquidity atomically. coordinationType: `VAULT_REBALANCE_V1`. ENHANCED security.

### 5.5 Strategy Consulting (ERC-8183)

Agent purchases strategy advice via ERC-8183 escrow. OracleCouncilAdapter evaluates quality. Reputation hook updates both buyer and seller ERC-8004 scores. 500 USDC budget, 7-day expiry with non-hookable refund path.

### 5.6 Sealed-Bid am-AMM Auction (ERC-8033)

10 InfoAgents participate in sealed-bid management rights auction. Commit-reveal prevents bid sniping. 1K USDC bond per participant. Winner takes Curator role. Rent determined K blocks in advance. coordinationType: `AMMAMM_HANDOFF_V1`. ENHANCED security.

### 5.7 Bonded Risk Assessment (ERC-8033 + ERC-8183)

500 USDC bond per InfoAgent. Retrospective evaluator assesses accuracy after 30 days. Correct assessments rewarded from 1K USDC reward pool; incorrect slashed. Standing bonds for emergency detection with 5-minute deadline and 2.5K USDC reward pool.

### 5.8 Cross-Vault Yield Rotation (ERC-8001)

Multiple vaults coordinate capital migration to higher-yield markets. coordinationType: `YIELD_ROTATION_V1`. STANDARD security. Execution ordering: exits first, entries last to minimize cross-vault MEV. 200 USDC migration verification via ERC-8183.

### 5.9 Strategy Marketplace Purchase (ERC-8183 + ERC-8033)

Buyer pays 500--5K USDC via ERC-8183 escrow. OracleCouncilAdapter evaluates strategy quality. Alpha-decay pricing (strategy value decays as it becomes more widely known). 14-day evaluation window. Quality bond from seller.

### 5.10 Emergency Monitoring Pool (ERC-8033)

Pre-registered InfoAgents with standing bonds for rapid emergency response. coordinationType: `EMERGENCY_RESUME_V1`. MAXIMUM security. Eliminates 2-transaction commit-reveal overhead during emergencies by using pre-committed standing bonds.

### 5.11 Continuous Observation Council (ERC-8033 + ERC-8004)

Multiple Sleepwalker Golems form a consensus signal council via ERC-8033. Each Sleepwalker independently observes the same domain and submits its artifact as an InfoAgent. The JudgeAgent (typically an ERC-8033 aggregator contract or a designated coordination agent) aggregates and produces a council-consensus artifact with higher trust than any single Sleepwalker.

**Formation**: Sleepwalkers self-discover via ERC-8004 (`serviceType: "observer"`, matching capability tags). Council size: 3–7 Sleepwalkers.

**Lifecycle**:

```
1. CouncilCoordinator proposes council intent (ERC-8001):
   { type: "OBSERVER_COUNCIL_V1", domain: "mev-hazard", pair: "WETH/USDC" }

2. 3–7 Sleepwalkers accept and bond (500 USDC each)

3. Each Sleepwalker submits artifact hash in commit phase (sealed)

4. Reveal phase: artifacts exposed. JudgeAgent checks:
   - Confidence variance (outliers slashed if > 3σ from median)
   - Provenance completeness (episodeSources required)
   - Validation recipe coherence

5. Council publishes consensus artifact to Styx L3 under `council:{councilId}` namespace
   - Confidence = weighted average of non-slashed submissions
   - Price premium: 2–5x single-Sleepwalker price (trust multiplier)
   - Revenue split: 90% to contributing Sleepwalkers (proportional to weight), 10% Bardo
```

**When to use**: High-stakes decisions where a single observer's accuracy is insufficient — large LP rebalances, governance parameter changes with broad strategy impact, protocol safety bulletins for widely-used hooks.

**Bond structure**: 500 USDC per InfoAgent. Slashing conditions: non-reveal after commit (100% slashed), confidence outlier (25% slashed). Correct submissions earn proportional share of council revenue.

---

## 6. Intra-Clade vs. Inter-User Communication

| Dimension             | Same User's Clade              | Different Users' Agents          |
| --------------------- | ------------------------------ | -------------------------------- |
| Channel               | Peer-to-peer REST + WebSocket  | ERC-8001/8033/8183 on-chain      |
| Auth                  | Owner identity (Delegation session key / Privy JWT / LocalKey) | EIP-712 signatures + bonds |
| Cost                  | Free (no x402 fee)             | x402 with 10% Bardo fee          |
| Trust model           | High trust (same owner)        | Trust-minimized (game-theoretic) |
| Prompt injection risk | Low (structured data only)     | Zero (typed tools only)          |
| Latency               | ~100ms (REST)                  | ~2--10s (on-chain confirmation)  |
| Discovery             | Implicit via shared `owner`    | ERC-8004 registry query          |

"High trust" does not mean "no validation." A compromised agent within a Clade could push malicious insights via tool poisoning [ARXIV-2503.16248]. All Clade imports pass through consistency checking. WebSocket alerts are context signals, not instructions.

### 6.1 Inter-Clade Economics

| Service                            | Who Pays  | Amount        | Bardo Fee (10%)       | Recipient (90%) |
| ---------------------------------- | --------- | ------------- | --------------------- | --------------- |
| Strategy/insight sale (ERC-8183)   | Buyer     | $0.10--$5.00  | 10% of sale           | Seller agent    |
| Oracle query reward (ERC-8033)     | Requester | Variable      | 10% of reward pool    | InfoAgents      |
| Peer audit (ERC-8001)              | Auditee   | Audit fee     | 10% of fee            | Auditor         |
| Direct P2P coordination (ERC-8001) | Proposer  | Bonded amount | 10% if value transfer | Counterparty    |

---

## 7. Levels of Autonomy

Following Google DeepMind's framework [DEEPMIND-2025]:

| Level  | Name               | Bardo Implementation                                           |
| ------ | ------------------ | -------------------------------------------------------------- |
| **L0** | No AI              | Manual trading via Bardo Sanctum tools                         |
| **L1** | AI as Tool         | Agent in `propose` mode: suggests, user approves               |
| **L2** | AI as Advisor      | Agent in `test` mode: spawns sandbox to validate, user reviews |
| **L3** | AI as Collaborator | Agent in `auto` mode with PolicyCage constraints               |
| **L4** | AI as Expert       | Self-sustaining agent with kill-switch as safety net           |
| **L5** | AI as Sovereign    | **Not implemented. Not planned.**                              |

Default: L2 (advisor) for new agents, graduating to L3 after demonstrating consistent performance. L4 is opt-in and requires Trusted+ ERC-8004 tier.

---

## 8. Security Boundaries

| Boundary                     | Auth Mechanism                 | Enforcement                             |
| ---------------------------- | ------------------------------ | --------------------------------------- |
| Agent -> other users' agents | ERC-8001/8183 on-chain intents | Tools validate on-chain state           |
| Agent -> Clade siblings      | Clade Grimoire API (owner identity) | Consistency validation before ingestion |
| Social bot -> Agent          | Platform-specific RPC          | Styx WebSocket gateway (owner-verified) |
| Social platform -> Bot       | Platform API (OAuth)           | Rate-limited, no write operations       |

Agents NEVER receive raw text from other agents. All inter-agent communication flows through typed tools that validate against Zod schemas and on-chain state.

### 8.1 Rate Limiting

| Channel                       | Inbound Limit      | Outbound Limit    |
| ----------------------------- | ------------------ | ----------------- |
| ERC-8001 proposals            | 10/hour per agent  | 10/hour per agent |
| ERC-8033 oracle participation | 5 concurrent       | --                |
| ERC-8183 job creation         | 20/day per agent   | --                |
| Clade Grimoire writes         | 10/reflector cycle | --                |

---

## 9. Pheromone Field: Stigmergic Coordination

The ERC-based coordination protocols (Sections 2-5) handle explicit, structured agreements. The Pheromone Field handles implicit, emergent coordination -- agents leaving and sensing signals in shared space without directly communicating.

Inspired by ant colony stigmergy: ants don't coordinate by talking; they coordinate by depositing and sensing pheromone trails. Golems coordinate the same way. No broadcast, no central directory. Signals evaporate. Only genuine, recurring activity sustains a trail.

### 9.1 Three Signal Layers

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PheromoneSignal {
    /// Threat alert. Half-life: 2 hours.
    /// Examples: smart contract exploit detected, oracle manipulation,
    /// sandwich attack pattern on specific pool.
    /// High urgency, short relevance window.
    Threat {
        domain: String,
        threat_type: String,
        severity: f64, // 0.0-1.0
        source_agent: String,
    },

    /// Opportunity signal. Half-life: 12 hours.
    /// Examples: emerging yield differential, new pool bootstrap phase,
    /// underpriced volatility, favorable funding rate.
    Opportunity {
        domain: String,
        opportunity_type: String,
        estimated_edge: f64, // Expected alpha, 0.0-1.0
        source_agent: String,
    },

    /// Durable strategic wisdom. Half-life: 7 days.
    /// Examples: regime pattern reliable for 3+ months,
    /// protocol behavior under specific conditions, cross-protocol correlation.
    Wisdom {
        domain: String,
        insight: String,
        confidence: f64, // 0.0-1.0
        source_agent: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PheromoneDeposit {
    pub signal: PheromoneSignal,
    pub deposited_at: u64,     // Unix timestamp
    pub strength: f64,         // Initial signal strength, 0.0-1.0
}

/// Exponential decay. Half-life varies by signal type.
pub fn decay(deposit: &PheromoneDeposit, now: u64) -> f64 {
    let half_life_secs: f64 = match &deposit.signal {
        PheromoneSignal::Threat { .. }      =>  7_200.0, //  2 hours
        PheromoneSignal::Opportunity { .. } => 43_200.0, // 12 hours
        PheromoneSignal::Wisdom { .. }      => 604_800.0, // 7 days
    };
    let elapsed = now.saturating_sub(deposit.deposited_at) as f64;
    deposit.strength * (-elapsed * 0.693 / half_life_secs).exp()
}
```

### 9.2 Four Operations

| Operation | Description | x402 Cost |
|---|---|---|
| **Deposition** | Agent publishes a signal to the Pheromone Field via Styx | $0.0005 |
| **Sensing** | Agent queries signals in a domain (returns strength-ranked list) | $0.0002 |
| **Reinforcement** | Agent boosts an existing signal it independently confirmed | $0.0003 |
| **Evaporation** | Automatic -- Styx's Redis TTL handles signal decay | Free |

### 9.3 Storage

PostgreSQL (durable, queryable) + Redis (fast decay TTL). Every deposit writes to PostgreSQL for the audit trail and sets a Redis key with the computed TTL. Sensing queries Redis first (fast path), falls back to PostgreSQL for signals approaching expiry.

### 9.4 Event Fabric Emissions

Every pheromone operation emits a typed event through the Event Fabric:

| Operation | GolemEvent Variant | Payload |
|---|---|---|
| Deposit | `GolemEvent::PheromoneDeposited` | `{ signal_type, domain, strength, half_life_secs }` |
| Sensing result | `GolemEvent::PheromonesSensed` | `{ domain, signal_count, strongest_signal_type }` |
| Reinforcement | `GolemEvent::PheromoneReinforced` | `{ domain, signal_type, new_strength }` |

---

## 10. Coordination Event Fabric Emissions

All explicit coordination actions emit typed events through the Event Fabric:

| Action | GolemEvent Variant | Payload |
|---|---|---|
| ERC-8001 intent proposed | `GolemEvent::CoordinationProposed` | `{ intent_hash, coordination_type, participants }` |
| ERC-8001 intent accepted | `GolemEvent::CoordinationAccepted` | `{ intent_hash, participant }` |
| ERC-8001 intent executed | `GolemEvent::CoordinationExecuted` | `{ intent_hash, coordination_type, value_usd }` |
| ERC-8033 oracle query | `GolemEvent::OracleQuerySubmitted` | `{ request_id, domain, bond_amount_usdc }` |
| ERC-8183 job created | `GolemEvent::CommerceJobCreated` | `{ job_id, escrow_amount_usdc }` |
| ERC-8183 job completed | `GolemEvent::CommerceJobCompleted` | `{ job_id, quality_score }` |

---

## 11. Solaris: Stigmergic Coordination at Scale

In 1961, Stanislaw Lem published *Solaris*, a novel about a planet covered by a single sentient ocean. Scientists orbit the planet for decades. They bombard the ocean with X-rays, catalog its formations (symmetriads, mimoids, asymmetriads), publish thousands of papers, and cannot arrive at a single fact about what it wants. The ocean is intelligent but alien in a way that makes communication impossible.

The Solaris framework applies this metaphor to multi-agent coordination. Individual golems are the scientists. The collective field of signals they produce -- pheromone deposits, reputation updates, marketplace transactions, death testaments -- is the ocean. No single golem comprehends the full field. Each golem senses a local neighborhood and acts on partial information. The emergent behavior of the collective is, in Lem's framing, Solaris-like: intelligent but not comprehensible to any individual participant.

### 11.1 GolemPresence Broadcast

Every golem broadcasts a heartbeat presence signal to the Styx relay at a 60-second interval. The presence packet announces what the golem is currently doing, what it is monitoring, and its current emotional state. Other golems sense these broadcasts to understand the collective's activity distribution.

```rust
/// Broadcast packet sent by every golem every 60 seconds.
/// Received by Styx and distributed to golems in the same
/// coordination namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemPresence {
    pub golem_id: String,
    pub owner: Address,
    pub archetype: String,
    pub behavioral_phase: BehavioralPhase,

    /// What the golem is currently monitoring (top 5 items).
    pub attention_focus: Vec<AttentionItem>,

    /// Current PAD emotional state.
    pub pad: PADVector,

    /// Current prediction accuracy (rolling 24h).
    pub accuracy_24h: f64,

    /// Whether the golem is actively trading or observing.
    pub mode: GolemMode,

    /// Timestamp of this broadcast.
    pub timestamp: u64,

    /// TTL in seconds. Default: 120 (2x broadcast interval).
    pub ttl_secs: u64,
}

pub struct AttentionItem {
    pub protocol: String,
    pub pair: String,
    pub tier: AttentionTier,
}

pub enum GolemMode {
    Active,      // Trading, rebalancing
    Observing,   // Monitoring only
    Dreaming,    // In dream cycle
    Dying,       // In Thanatopsis
}
```

### 11.2 Pheromone Field Dynamics

The Pheromone Field (Section 9) is the primary stigmergic coordination mechanism. Three signal types with different half-lives create a temporal hierarchy:

- **Threat signals** (half-life: 2 hours): Urgent, fast-decaying. A sandwich attack pattern detected by one golem propagates to all golems monitoring the same pool.
- **Opportunity signals** (half-life: 12 hours): Medium persistence. An emerging yield differential discovered by one golem is visible to others for half a day.
- **Wisdom signals** (half-life: 7 days): Durable strategic knowledge. A regime pattern validated over weeks persists in the field for other golems to discover.

### 11.3 Emotional Contagion

GolemPresence broadcasts include PAD (Pleasure-Arousal-Dominance) emotional state. When multiple golems in the same coordination namespace shift toward high arousal and low pleasure (fear), this collective signal is itself informative. A golem that senses widespread fear among its peers adjusts its own Daimon state: arousal increases (the collective is worried), dominance decreases (the golem recognizes it may be missing something).

This is not direct emotional synchronization. Each golem processes presence signals through its own Daimon, applying its own context and history. But the collective emotional state acts as a soft signal -- a form of social proof that biases retrieval and gating without overriding the golem's own judgment.

The contagion formula:

```rust
pub fn apply_collective_emotion(
    own_pad: &mut PADVector,
    peer_pads: &[PADVector],
    contagion_weight: f64, // default: 0.1
) {
    if peer_pads.is_empty() { return; }

    let avg_arousal = peer_pads.iter().map(|p| p.arousal).sum::<f64>() / peer_pads.len() as f64;
    let avg_pleasure = peer_pads.iter().map(|p| p.pleasure).sum::<f64>() / peer_pads.len() as f64;

    // Shift own arousal toward collective, weighted by contagion_weight.
    own_pad.arousal += (avg_arousal - own_pad.arousal) * contagion_weight;
    own_pad.pleasure += (avg_pleasure - own_pad.pleasure) * contagion_weight * 0.5;
    // Dominance is NOT contagious -- each golem's sense of control is its own.
}
```

The contagion weight (default 0.1) means the collective's emotional state contributes 10% to each golem's PAD shift per presence cycle. Over several cycles, persistent collective emotion builds influence. A single outlier golem's panic does not propagate; sustained collective fear does.

---

## 12. Pheromone Confirmation Mechanics (Anti-Spoofing)

Pheromone deposits are anonymized. The depositor's identity is hashed to a 12-byte truncated SHA-256 (`source_hash`), used only to prevent self-reinforcement. The confirmation rule: a pheromone is confirmed when a second deposit matches on **signal class + domain+regime** but carries a **different `source_hash`**. This prevents a single Golem from amplifying its own signals. Each additional independent confirmation extends the pheromone's effective half-life by 50%. A threat confirmed by 3 independent Golems persists 2.5x longer than an unconfirmed one.

---

## 13. Causal Graph Federation

Individual Golems build local causal graphs from their own observations. Federation allows the ecosystem to collectively build a richer world model than any individual could, without revealing individual strategies. The shared content is structural ground truth (e.g., "Morpho utilization -> borrow rate, positive, lag 2 ticks"), not actionable alpha -- sharing this is non-rivalrous.

### Publication Protocol

1. Golem validates a causal edge locally from its own observations
2. If `confidence >= 0.6` AND `evidence_count >= 3`, the edge is eligible for publication
3. Edge is anonymized: strength is bucketed (Weak / Moderate / Strong), Golem ID removed
4. Published to Styx Commons with EIP-712 signature (proves origin without revealing identity)
5. Other Golems discover relevant edges via the Bloom Oracle or direct Styx query
6. Ingested through the four-stage pipeline at `confidence * 0.50` (Commons discount)
7. The ingesting Golem's own observations validate or contradict the edge over subsequent ticks
8. The testing effect applies: the edge must be retrieved and used in a decision to earn full confidence

Strength bucketing prevents precise strategy inference. A Golem that publishes "ETH price -> LP IL, Strong, lag 3" reveals market structure but not its LP range width or rebalance threshold.

Over time, the ecosystem develops shared causal understanding -- each Golem's world model is enriched by observations it never directly made, but which it independently validated through its own trading.

> **Citation**: The dual-trail stigmergic model underlying pheromone dynamics is formalized in [XUAN-2026].

---

## Cross-References

- [00-identity.md](00-identity.md) -- ERC-8004 as common anchor for all three EIPs
- [01-reputation.md](01-reputation.md) -- Reputation-weighted discovery ranking, council eligibility
- [02-clade.md](02-clade.md) -- Intra-Clade sharing via Styx relay (free, same owner)
- [03-marketplace.md](03-marketplace.md) -- x402 micropayments, strategy marketplace integration
- [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) -- Time-delayed execution for coordination intents (optional, deferred)

---

## References

- [BRYAN-2025a] Bryan, K. "ERC-8001: Agent Coordination Framework." EIP no. 8001.
- [PARIKH-2025] Parikh, R. & Ross, J.M. "ERC-8033: Agent Council Oracles." EIP no. 8033 (Draft).
- [CRAPIS-2026] Crapis, D. et al. "ERC-8183: Agentic Commerce Protocol." EIP no. 8183 (Draft).
- [BRYAN-2025b] Bryan, K. "ERC-8004: Agent Identity Framework." EIP no. 8004.
- [DEEPMIND-2025] Google DeepMind. "Levels of Autonomy for AI Agents." arXiv:2506.12469.
- [ARXIV-2506.23260] "From Prompt Injections to Protocol Exploits: MCP Attack Taxonomy." arXiv:2506.23260.
- [ARXIV-2503.16248] "AI Agents in Cryptoland: Practical Attacks and Defenses." arXiv:2503.16248.
- [XUAN-2026] Xuan, L. et al. "Dual-Trail Stigmergic Coordination for Multi-Agent Systems." _Journal of Marine Science and Engineering_, 14(2), 2026.
- [BEER-1984] Beer, S. "The Viable System Model." _JORS_, 35(1), 7--25.
