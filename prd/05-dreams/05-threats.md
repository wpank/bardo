# Dream Threat Simulation: Rehearsing Failure Before It Happens [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Depends on**: `00-overview.md`, `01-architecture.md`, `golem-grimoire`, `golem-inference`

---

> **Reader orientation:** This document specifies the Threat Simulator, the dream component that rehearses DeFi-specific failure modes within Bardo (the Rust runtime for mortal autonomous DeFi agents). It covers a three-tier DeFi threat taxonomy (existential, severe, degradation), the threat simulation protocol, adversarial dreaming for stress-testing Golem (mortal autonomous agent) strategies, gap analysis, and validation metrics. A Golem cannot afford to learn about flash crashes through direct experience -- threat dreams provide zero-cost rehearsal. Prerequisites: the Dreams overview (`00-overview.md`) and architecture (`01-architecture.md`). For a full glossary, see `prd2/shared/glossary.md`.

## The Evolutionary Case for Threat Dreams

Antti Revonsuo's Threat Simulation Theory (2000) proposes that the biological function of dreaming is to simulate threatening events and rehearse avoidance behaviors [REVONSUO-2000]. The empirical evidence is striking: dream content contains more frequent and severe threats than waking life. Real threat exposure amplifies the system — children who experience real threats dream more frequent and more intense threat scenarios. Dreams over-represent threats not because the brain is broken but because offline threat rehearsal improves real-world survival.

For a mortal Golem operating in DeFi — an environment where flash crashes, smart contract exploits, MEV attacks, and oracle manipulation are not hypothetical but routine — threat simulation is not a luxury. It is the primary survival mechanism that dreaming provides.

The Golem cannot afford to learn about flash crashes through direct experience. A single catastrophic loss can deplete its USDC balance and trigger economic death. Threat dreams allow the Golem to experience simulated catastrophes at zero cost, developing rehearsed responses that activate instantly when real threats materialize.

---

## The DeFi Threat Taxonomy

The ThreatSimulator maintains a hierarchical threat catalog, organized by source, severity, and the Golem's ability to mitigate:

### Tier 1: Existential Threats (Can Kill the Golem)

| Threat                     | Description                              | Example                     | Mitigation                                                        |
| -------------------------- | ---------------------------------------- | --------------------------- | ----------------------------------------------------------------- |
| **Flash crash**            | Sudden price drop >20% in minutes        | ETH -30% in 5 min, May 2021 | Pre-positioned exit orders, max drawdown trigger                  |
| **Smart contract exploit** | Vulnerability in deployed contracts      | Euler Finance ($197M, 2023) | Monitoring for governance proposals, exit on anomaly detection    |
| **Cascading liquidation**  | Forced closures trigger further closures | LUNA/UST death spiral, 2022 | Correlation monitoring, position diversification across protocols |
| **Rug pull**               | Protocol team drains liquidity           | Numerous examples in DeFi   | TVL monitoring, governance analysis, concentration limits         |
| **Oracle manipulation**    | Price feed compromised                   | Mango Markets ($114M, 2022) | Multi-oracle validation, deviation thresholds                     |

### Tier 2: Severe Threats (Significant Capital Loss)

| Threat                     | Description                         | Example                       | Mitigation                                           |
| -------------------------- | ----------------------------------- | ----------------------------- | ---------------------------------------------------- |
| **MEV sandwich attack**    | Front-running + back-running a swap | Routine on Ethereum mainnet   | Slippage limits, private mempools, MEV-aware routing |
| **Impermanent loss spike** | Extreme divergence in LP pair       | High-vol periods              | Position monitoring, rebalancing triggers            |
| **Gas war**                | Sustained extreme gas prices        | NFT mints, network congestion | Gas ceiling guards, position freezing during spikes  |
| **Liquidity withdrawal**   | Large LPs exit, increasing slippage | Whale movements               | TVL monitoring, exit queue management                |
| **Regulatory action**      | Sanctions, protocol freezing        | Tornado Cash sanctions, 2022  | Compliance monitoring, asset diversification         |

### Tier 3: Degradation Threats (Erodes Performance Over Time)

| Threat                    | Description                     | Example                                | Mitigation                                    |
| ------------------------- | ------------------------------- | -------------------------------------- | --------------------------------------------- |
| **Yield compression**     | Returns decline as TVL grows    | Saturated farming pools                | APY monitoring, threshold guards              |
| **Fee structure changes** | Protocol updates fee parameters | Uniswap governance proposals           | Governance monitoring, strategy adaptation    |
| **Correlation shift**     | Historical correlations break   | BTC-ETH decoupling events              | Rolling correlation monitoring                |
| **Concept drift**         | Market regime changes           | Post-merge ETH dynamics                | Epistemic fitness tracking (existing Track 1) |
| **Memory poisoning**      | Accumulated bad heuristics      | Validated strategies that became stale | PLAYBOOK.md pruning (dream consolidation)     |

---

## Threat Simulation Protocol

### Scenario Generation

For each threat category, the ThreatSimulator generates concrete, detailed scenarios calibrated to the Golem's current positions and strategies:

```rust
pub struct ThreatSimulation {
    pub threat_id: String,
    pub tier: ThreatTier,
    pub category: String,
    pub scenario: ThreatScenarioDetail,
    pub impact_analysis: ImpactAnalysis,
    pub rehearsed_responses: Vec<RehearsedResponse>,
    pub playbook_guards: Vec<String>,
    pub gaps: Vec<String>,
}

pub enum ThreatTier { T1, T2, T3 }

pub struct ThreatScenarioDetail {
    pub narrative: String,
    pub trigger_conditions: Vec<String>,
    pub timeline: String,
    pub severity: f64,
    pub probability: f64,
}

pub struct ImpactAnalysis {
    pub affected_positions: Vec<String>,
    pub estimated_loss: f64,
    pub estimated_loss_percent: f64,
    pub cascade_risk: String,
}

pub struct RehearsedResponse {
    pub name: String,
    pub actions: Vec<String>,
    pub expected_outcome: String,
    pub worst_case: String,
    pub latency_requirement: String,
    pub cost: String,
}
```

### Simulation Prompt Structure

```
Threat Simulation: Flash Crash Scenario

Current portfolio state:
[serialized positions, balances, pending transactions]

Current PLAYBOOK.md guards:
[extracted risk management sections]

Scenario: ETH price drops 25% in 8 minutes. Simultaneously:
- Gas prices spike to 300 gwei
- Pool TVL drops 40% as LPs flee
- Oracle updates lag by 2 blocks
- Three pending transactions are in the mempool

Analyze:
1. Which positions are at immediate risk? Quantify the loss.
2. In what ORDER should positions be closed? (Triage)
3. What is the total gas cost of an emergency exit at 300 gwei?
4. Are any PLAYBOOK.md guards triggered? Which ones fire first?
5. What guard is MISSING that should exist?
6. What is the single fastest action that would preserve the most capital?
7. If only ONE action can be taken (gas too high for multiple txns),
   which action should it be?

For each question, provide both the analysis and a concrete PLAYBOOK.md
guard that would implement the response automatically.
```

### Adversarial Dreaming

Deperrois et al. (2022) showed that the REM "adversarial dreaming" phase — GAN-like creative generation of novel experiences — is essential for semantic learning. Removing it severely degraded performance [DEPERROIS-2022]. The ThreatSimulator implements an adversarial mode where it deliberately generates worst-case scenarios designed to exploit weaknesses in the current PLAYBOOK.md:

```
Adversarial Threat Generation

You are a sophisticated attacker analyzing this trading agent's strategy.

Agent's PLAYBOOK.md (relevant sections):
[extracted strategy and risk management sections]

Agent's current positions:
[serialized positions]

Design an attack that would:
1. Exploit the specific gaps in the agent's risk management
2. Occur in conditions where the agent's guards would NOT trigger
3. Cause maximum damage with minimum detectable precursor signals
4. Be plausible given current DeFi market dynamics

For each attack vector:
- Describe the attack step by step
- Identify which PLAYBOOK.md guard would need to exist to prevent it
- Estimate the probability of this attack occurring naturally
- Estimate the probability of this attack being intentionally executed
  by a sophisticated MEV searcher or attacker
```

This adversarial approach inverts the normal simulation: instead of "what would go wrong in scenario X?", it asks "what scenario would maximally exploit my current weaknesses?" The resulting threat scenarios are by construction the most dangerous ones — they are designed to bypass existing defenses.

---

## Threat-Informed PLAYBOOK.md Evolution

### Gap Analysis

After each threat simulation session, the ThreatSimulator produces a gap analysis: which threats have adequate PLAYBOOK.md coverage and which do not.

```rust
pub struct ThreatGapAnalysis {
    pub covered_threats: Vec<CoveredThreat>,
    pub uncovered_threats: Vec<UncoveredThreat>,
    pub weak_guards: Vec<WeakGuard>,
}

pub struct CoveredThreat {
    pub threat: String,
    pub guards: Vec<String>,
    pub confidence_in_guards: f64,
    pub last_rehearsal: u64,
}

pub struct UncoveredThreat {
    pub threat: String,
    pub severity: f64,
    pub proposed_guard: String,
    pub priority: f64,
}

pub struct WeakGuard {
    pub threat: String,
    pub guard: String,
    pub weakness: String,
    pub proposed_fix: String,
}
```

Gap analysis outputs feed directly into the PLAYBOOK.md staging buffer (see `04-consolidation.md`). Proposed guards for uncovered threats enter the staging buffer at confidence 0.3 — higher than creative hypotheses (0.2) because they address concrete, identified risks rather than speculative opportunities.

### Threat Rehearsal Schedule

Not all threats need equal rehearsal frequency. The schedule follows a priority ordering:

| Factor                            | Weight | Rationale                                           |
| --------------------------------- | ------ | --------------------------------------------------- |
| Recent loss from this threat type | 0.3    | Revonsuo: real threat exposure amplifies simulation |
| Probability of occurrence         | 0.25   | More likely threats need more rehearsal             |
| Severity if unrehearsed           | 0.25   | Existential threats get priority                    |
| Time since last rehearsal         | 0.2    | Avoid staleness, spaced repetition                  |

Tier 1 threats are rehearsed every dream cycle. Tier 2 threats are rehearsed every 2–3 cycles. Tier 3 threats are rehearsed weekly. After a real loss event matching a threat category, rehearsal frequency for that category doubles for the next 5 dream cycles — matching the biological amplification effect [REVONSUO-2000].

#### Regime-Aware Threat Scheduling

When current `RegimeTag` (see `../02-mortality/01-architecture.md`) shows `volatilityQuintile >= 4` or `liquidityCondition === 'thin'`, double Tier 1 threat rehearsal frequency in the next dream cycle. Proactive amplification based on environmental conditions, not waiting for actual loss.

| Condition                       | Tier 1 Frequency Multiplier | Rationale                                    |
| ------------------------------- | --------------------------- | -------------------------------------------- |
| `volatilityQuintile >= 4`       | 2x                          | High volatility → flash crash more likely    |
| `liquidityCondition === 'thin'` | 2x                          | Thin liquidity → slippage crisis more likely |
| Both conditions                 | 3x                          | Compound risk → maximum rehearsal priority   |
| Neither                         | 1x (baseline)               | Standard scheduling                          |

This implements proactive threat preparation: the Golem rehearses emergency responses more frequently when market conditions suggest those emergencies are more likely.

---

## Integration with the Daimon Track

If `golem-daimon` is enabled, threat simulation interacts with the emotional system:

1. **Threat dreams generate controlled anxiety.** The appraisal engine registers threat scenarios as low-dominance, negative-pleasure events — the functional analog of productive worry. This controlled anxiety is adaptive: it sharpens vigilance without paralyzing action. Chronic unresolved anxiety (from real losses) is maladaptive; controlled anxiety from rehearsal (from simulated losses) builds preparedness.

2. **Post-loss threat amplification.** After a real loss, the daimon track registers sustained negative mood. This mood state biases dream content toward threat simulation — more threat scenarios are generated, rehearsed more deeply, and the ThreatSimulator operates with higher severity assumptions. This mirrors the biological pattern where trauma exposure increases threat dream frequency.

3. **Threat desensitization.** Repeated rehearsal of the same threat scenario gradually reduces its emotional charge. A flash crash scenario that initially generates arousal=0.8 will, after 5 rehearsals, generate arousal=0.4. This is not complacency — the factual preparedness remains. It is emotional regulation: the Golem maintains vigilance without paralysis.

---

## Validation: Did Threat Rehearsal Help?

Threat simulation is validated by tracking three metrics:

1. **Response latency.** When a threat materializes during live trading, how quickly does the Golem respond? Compare latency for rehearsed vs. unrehearsed threat types. Prediction: rehearsed threats should show significantly faster response (retrieving a rehearsed response vs. reasoning from scratch).

2. **Capital preserved.** For threats that materialize, compare capital loss for rehearsed vs. unrehearsed threats. Prediction: rehearsed threats should show lower loss-to-severity ratio.

3. **False alarm rate.** Threat trigger conditions that fire but no actual threat materializes. High false alarm rates indicate overly sensitive triggers — the guards are wasting gas and opportunity cost on phantom threats. The target false alarm rate is <30% — at least 70% of trigger activations should correspond to genuine threats.

```rust
pub struct ThreatValidationMetrics {
    pub threat_category: String,
    pub rehearsal_count: u32,
    pub actual_occurrences: u32,

    pub average_response_latency_ms: u64,
    pub baseline_response_latency_ms: u64,
    pub latency_improvement: f64,

    pub average_loss_rehearsed: f64,
    pub average_loss_unrehearsed: f64,
    pub capital_preservation_benefit: f64,

    pub trigger_activations: u32,
    pub true_positives: u32,
    pub false_alarms: u32,
    pub false_alarm_rate: f64,
}
```

---

## Safety Constraints

1. **Threat responses cannot exceed PolicyCage limits.** A rehearsed response to a flash crash cannot involve taking a leveraged short position if PolicyCage prohibits shorts. The threat simulator must generate responses within the existing safety envelope.

2. **Adversarial generation is bounded.** The adversarial dreaming mode generates attack scenarios, not actual attacks. Outputs are stored in the Grimoire as threat records, not executed as strategies. The line between "imagining an attack" and "planning an attack" is enforced by the same staging buffer that governs all dream outputs.

3. **Owner visibility for adversarial outputs.** All adversarial threat generation outputs are tagged `provenance: "adversarial_dream"` and are available for owner review. If the owner is uncomfortable with the sophistication of generated attack vectors, they can disable the adversarial mode while keeping standard threat simulation active.

---

## Grounding Threat Responses in On-Chain Reality

For Tier 1 (existential) threats, the ThreatSimulator validates the _feasibility_ of rehearsed responses using production-available verification primitives:

1. **Response feasibility check**: For each rehearsed response action (emergency exit, position close, rebalance), run `publicClient.simulateContract()` against current state to verify the action would succeed _right now_. If the exit transaction would revert today, the rehearsed response is broken regardless of the threat scenario.

2. **Gas budget validation**: `publicClient.estimateContractGas()` for the full response sequence. Compare against the Golem's gas partition. If the emergency exit sequence costs more gas than the Golem can afford, the gap is concrete and the PLAYBOOK.md guard is inadequate.

3. **Liquidity sanity check**: For exit-based responses, read current pool reserves via `publicClient.readContract()`. If current liquidity wouldn't support the planned exit size at acceptable slippage, the rehearsed response needs revision.

This doesn't simulate the threat condition itself (you'd need state overrides for that, which require Anvil). But it validates that the _response actions are executable today_. A rehearsed response that would revert under normal conditions will certainly fail under crisis conditions.

**Production vs. development capabilities**:

| Validation Type                 | Production                         | Dev/Testing                            |
| ------------------------------- | ---------------------------------- | -------------------------------------- |
| Response action would succeed   | `simulateContract()`               | Anvil fork + state overrides           |
| Gas budget sufficient           | `estimateContractGas()`            | Anvil fork with gas price overrides    |
| Liquidity adequate for exit     | `readContract()` pool reserves     | Anvil fork with liquidity removal      |
| Full threat scenario end-to-end | Not available (LLM reasoning only) | Anvil fork with full crisis simulation |

**When full state-override simulation is desired** (development only): During Evaluation Lifecycle Phases 1-2, the owner can run threat rehearsals against Anvil forks with injected crisis conditions. Results from these dev-phase rehearsals can be stored as `ThreatRehearsalResult` entries in the Grimoire and inherited by production Golems.

---

## Budget Tier Availability

Threat simulation is a **[HARDENED]-only** capability. [CORE] lightweight dreams do not include threat simulation — they are limited to compressed replay and simplified counterfactual reasoning. The rationale: threat simulation requires generating detailed, multi-step adversarial scenarios that benefit from T1–T2 inference quality. T0 (Haiku-class) inference produces insufficiently detailed threat scenarios to be useful for rehearsal.

Owners who want threat simulation must set `dream.tier = "hardened"` in `bardo.toml`.

> **Cross-ref**: `01-architecture.md` (Compute Budget, [CORE] vs [HARDENED] tiers)

---

## Pheromone Field influence

Active THREAT pheromones from the coordination layer (half-life 2h) bias the ThreatSimulator's scenario generation toward currently relevant threats. When the Pheromone Field reports an active threat signal -- for example, a liquidation cascade in protocols the Golem has exposure to -- the dream threat simulation prioritizes that scenario type.

The mechanism: at the start of each threat simulation session, the ThreatSimulator queries the Pheromone Field for active pheromones with `type: "THREAT"` and `relevance > 0.5` (where relevance is computed from protocol overlap between the pheromone's domain and the Golem's current positions). Each matching pheromone adds a pre-seeded scenario to the simulation queue with priority proportional to the pheromone's remaining strength.

This creates a feedback loop between coordination and dreaming: sibling Golems that encounter real threats broadcast pheromones; sleeping Golems pick up those pheromones and rehearse responses before waking into the same threat environment. The result is proactive threat preparation based on peer intelligence rather than waiting for direct experience.

> **Cross-ref**: `../09-coordination/02-pheromone-field.md` (pheromone types, half-lives, relevance scoring)

---

## Citation Summary

| Citation Key     | Source                                                                             |
| ---------------- | ---------------------------------------------------------------------------------- |
| [REVONSUO-2000]  | Revonsuo. "The reinterpretation of dreams." _Behavioral and Brain Sciences_, 2000. |
| [DEPERROIS-2022] | Deperrois et al. "Perturbed and adversarial dreaming." _eLife_, 2022.              |
