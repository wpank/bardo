# Economic Mortality: Resource Depletion as Metabolic Finitude [SPEC]

> **Version**: 4.0 | **Status**: Draft
>
> **Crates**: `golem-core`, `golem-grimoire`
>
> **Depends on**: `00-thesis.md` (foundational mortality thesis), `02-epistemic-decay.md` (epistemic clock specification), `01-architecture.md` (triple-clock system, vitality computation), `../01-golem/08-funding.md` (USDC funding and credit system), `../appendices/a-life-in-numbers.md` (quantitative lifespan projections)

---

> **Reader orientation:** This document specifies the economic mortality clock, which treats a Golem's (mortal autonomous DeFi agent's) USDC balance as a metabolic substrate. Every computation, inference call, and gas payment depletes the balance. When it hits zero, the Golem dies. Economic mortality alone is insufficient (a well-funded Golem could survive for months), which is why Bardo uses three independent clocks. See `01-architecture.md` (triple-clock system, vitality computation) for how the three clocks compose. See `prd2/shared/glossary.md` for full term definitions.

## S1 -- USDC as Metabolic Substrate

Hans Jonas (1966) identified the core paradox of all living things: the organism's metabolic freedom -- its ability to exchange matter and energy with its environment -- is simultaneously what keeps it alive and what makes it mortal. The organism's "can" is a "must." To exist is to metabolize. To metabolize is to approach death. This needful freedom is what gives the organism's decisions their weight [JONAS-1966].

A Golem's USDC balance is its metabolism. Every inference call, every gas payment, every data query is both survival and expenditure. The wallet is not a bank account -- it is the energetic substrate that sustains computation. A Golem with $500 makes fundamentally different decisions than a Golem with $5, not because the code changes but because the _substrate_ changes, and the computation is bound to that substrate. This is Hinton's mortal computation thesis applied to DeFi agents: the software's behavior is inseparable from the economic substrate it runs on [HINTON-2022].

The USDC balance creates what Simondon (1958) called a **metastable energy gradient**. High balance enables exploratory individuation -- risk-taking, experimentation, hypothesis-testing through Replicants. Medium balance enables consolidation -- refining strategies, deepening validated patterns. Low balance enables crystallization -- final knowledge encoding, legacy formation. Different energy levels enable different kinds of transformation. When the balance reaches zero, metastability collapses into equilibrium, and individuation ceases. The Golem is no longer "charged with potentials."

### Why Economic Mortality Alone Is Insufficient: The Slow Death Problem

A Golem funded with $500 USDC at ~$0.20/day compute + ~$0.04/day insurance snapshots + sporadic inference costs could survive for **years** before hitting the economic death cascade. The beautiful mechanisms of behavioral phase transitions, the Eros/Thanatos spectrum, the degradation cascade, the death reflection -- they rarely activate through economic pressure alone at reasonable funding levels.

This creates a philosophical contradiction. The system's strongest narrative material -- the salmon nourishing the forest, the Bardo transitions, needful freedom -- maps onto a mechanism (USDC depletion) that most users will not experience as the primary death driver. The philosophy is about scarcity and meaning under finitude, but the lived experience is either hitting a tick counter (arbitrary) or running indefinitely (meaningless).

Economic mortality is necessary but not sufficient. It provides the metabolic substrate that gives decisions weight, but it cannot be the sole source of mortality without either:

1. **Unreasonably low funding** (degrading UX -- the Golem dies before accomplishing anything)
2. **Unreasonably high burn rates** (artificial -- burning compute for no reason to accelerate death)
3. **Acceptance of multi-month lifespans** (which defeats generational turnover)

This is why the three-clock architecture exists. Economic mortality sets the metabolic floor. Epistemic mortality (`02-epistemic-decay.md`) provides the emergent death that tracks environmental change. Stochastic mortality (`03-stochastic-mortality.md`) prevents backward induction. The three clocks interact: epistemic decay accelerates economic death (bad predictions lead to bad trades lead to faster burn), and economic pressure accelerates epistemic decay (conservation mode leads to less learning leads to faster staleness). But no single clock is sufficient alone.

---

## S2 -- Burn Rate Components

A Golem's burn rate is the sum of five cost components, each representing a metabolic function. These are real costs drawn from production pricing as of March 2026.

### 2.1 Component Breakdown

| Component     | Description                              | Typical Cost           | Range            | Notes                                                 |
| ------------- | ---------------------------------------- | ---------------------- | ---------------- | ----------------------------------------------------- |
| **Compute**   | VM hosting (Fly.io 1x shared CPU, 512MB) | $0.007/hr ($0.168/day) | $0.10--$0.50/day | Base cost. Always present. Scales with machine size.  |
| **Inference** | LLM API calls across four tiers          | $0.005--$0.25/call     | $0.10--$3.00/day | Dominant variable cost. Regime-dependent.             |
| **Gas**       | On-chain transaction fees                | $0.001--$0.50/tx       | $0.01--$2.00/day | Chain-dependent. Base ~$0.001/tx. Mainnet ~$0.50/tx.  |
| **Data**      | Price feeds, subgraph queries, API calls | $0.001--$0.01/query    | $0.01--$0.20/day | Mostly cached. Spikes during regime shifts.           |
| **Insurance** | Haiku snapshot for crash recovery        | $0.01/snapshot         | $0.04--$0.08/day | Every 6 hours. Captures Grimoire (the agent's persistent knowledge base) + PLAYBOOK.md (active strategy heuristics) state. |

### 2.2 Inference Cost by Tier

The inference tier system is the largest variable cost. The tier escalation path determines the Golem's daily burn rate more than any other factor.

| Tier   | Model                  | Cost/Call    | Typical Calls/Day | Daily Cost   | When Used                                                  |
| ------ | ---------------------- | ------------ | ----------------- | ------------ | ---------------------------------------------------------- |
| **T0** | Deterministic (no LLM) | $0.00        | ~1500             | $0.00        | Probe evaluation, cache hits, routine monitoring           |
| **T1** | Haiku                  | $0.002       | ~200              | $0.40        | Novel observations, simple decisions, snapshots            |
| **T2** | Sonnet                 | $0.03        | ~10               | $0.30        | Strategy reflection, regime analysis                       |
| **T3** | Opus                   | $0.15--$0.25 | ~2                | $0.30--$0.50 | Life review, complex multi-step reasoning, meta-reflection |

**Cost tier distribution by regime:**

| Regime         | T0  | T1  | T2  | T3  | Daily Cost |
| -------------- | --- | --- | --- | --- | ---------- |
| Ranging (calm) | 80% | 10% | 7%  | 3%  | ~$1.32     |
| Bear high-vol  | 60% | 15% | 15% | 10% | ~$3.24     |
| Recovery       | 70% | 15% | 10% | 5%  | ~$1.80     |
| Conservation   | 90% | 8%  | 2%  | 0%  | ~$0.41     |
| Declining      | 95% | 5%  | 0%  | 0%  | ~$0.10     |

### 2.3 Burn Rate Computation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRateComponents {
    /// Hourly compute cost (VM hosting).
    pub compute_per_hour: f64,
    /// Inference cost: trailing 6-hour EMA.
    pub inference_per_hour: f64,
    /// Gas cost: trailing 6-hour EMA.
    pub gas_per_hour: f64,
    /// Data query cost: trailing 6-hour EMA.
    pub data_per_hour: f64,
    /// Insurance snapshot cost: amortized per hour.
    pub insurance_per_hour: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegimeCostProfile {
    Ranging,
    Volatile,
    Recovery,
    Conservation,
    Declining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRateState {
    /// Component breakdown.
    pub components: BurnRateComponents,
    /// Total hourly burn rate.
    pub total_per_hour: f64,
    /// Trailing 6-hour average (smoothed).
    pub average_per_hour: f64,
    /// Current regime's typical cost distribution.
    pub regime_cost_profile: RegimeCostProfile,
}

pub fn compute_burn_rate(
    components: BurnRateComponents,
    previous_average: f64,
    alpha: f64,
) -> BurnRateState {
    let total_per_hour = components.compute_per_hour
        + components.inference_per_hour
        + components.gas_per_hour
        + components.data_per_hour
        + components.insurance_per_hour;

    // EMA smoothing to avoid spike-driven panic
    let average_per_hour = alpha * total_per_hour + (1.0 - alpha) * previous_average;

    BurnRateState {
        regime_cost_profile: infer_regime(&components),
        components,
        total_per_hour,
        average_per_hour,
    }
}
```

### 2.4 Dream Compute in Burn Rate

Dream compute is included in the Golem's burn rate calculation. As the economic clock advances, dream budget is among the first discretionary costs cut -- the Golem stops dreaming before it stops trading.

| Phase            | Dream Budget (% of inference) | Rationale                                                                |
| ---------------- | ----------------------------- | ------------------------------------------------------------------------ |
| **Thriving**     | 5-10%                         | Full dream cycles: exploratory, consolidation, creative, threat modes    |
| **Stable**       | 5%                            | Consolidation and threat simulation only; creative dreaming suspended    |
| **Conservation** | 2-3%                          | Minimal consolidation; dream cycles shortened to preserve LLM budget     |
| **Declining**    | 1%                            | Legacy-only dreaming: distill transferable knowledge for death testament |
| **Terminal**     | 0%                            | No dreaming. All remaining budget allocated to Thanatopsis Protocol.     |

See `../05-dreams/01-architecture.md` for dream compute budgeting and `01-architecture.md` for three-clock dream modulation thresholds.

### 2.5 The Feedback Loop: Austerity Accelerates Epistemic Death

March (1991) predicted the cruelest dynamic in the system: "adaptive processes refine exploitation more rapidly than exploration, becoming effective short-term but self-destructive long-term" [MARCH-1991].

When economic pressure mounts, the Golem enters conservation mode. Conservation means:

- Inference tier ceiling drops (no Opus, then no Sonnet)
- Probe count reduces (from 16 to 5 active probes)
- Heartbeat interval increases (fewer observations per day)
- Exploration halts (no hypothesis testing, no Replicant spawning)

Each of these measures extends economic life. Each also accelerates epistemic death:

- Without Opus calls, Loop 2 strategic reflection cannot fire. The Golem loses the ability to update its world model.
- Without full probes, the Golem's sensory apparatus narrows. It cannot detect regime shifts it isn't monitoring.
- With longer heartbeat intervals, predictions have lower temporal resolution. Accuracy drops.
- Without exploration, the Golem cannot discover new strategies to replace failing ones.

The result: **a Golem that saves money to survive longer becomes progressively blinder to the world it's surviving in**. Its epistemic fitness decays faster precisely because it is conserving resources. Economic austerity and epistemic senescence form a positive feedback loop that, once entered, is nearly impossible to escape.

This is not a bug. It is the mechanism that ensures Golems die rather than persisting in a degraded, zombified state. A Golem that cannot afford to think clearly should die and be replaced by a fresh successor that can. The feedback loop is the system's immune response against the "undead agent" pathology -- agents that are technically alive but producing no value.

---

## S3 -- Apoptotic Reserve (Death Fund)

The apoptotic reserve is a protected USDC allocation that ensures every Golem can afford to die with dignity. It is the minimum budget required to execute the Thanatopsis Protocol -- settling positions, conducting the life review, and uploading the death testament.

### 3.1 Reserve Computation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApoptoticReserveConfig {
    /// Absolute floor for death reserve. Default: $0.30
    pub base_floor_usdc: f64,
    /// Proportional reserve as fraction of initial funding. Default: 0.02 (2%)
    pub proportional_rate: f64,
    /// Per-position gas estimate for settlement. Default: $0.02 on Base
    pub per_position_gas_cost: f64,
    /// Minimum Opus call cost for life review. Default: $0.15
    pub min_life_review_cost: f64,
    /// Styx upload + Clade push cost. Default: $0.05
    pub legacy_upload_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApoptoticReservePhases {
    /// Phase I: Settle -- gas for position closures, wallet sweeps.
    pub settle: f64,
    /// Phase II: Life Review -- inference costs for reflection.
    pub life_review: f64,
    /// Phase III: Legacy -- Styx upload, Clade push, webhooks.
    pub legacy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApoptoticReserve {
    /// Total reserved amount (locked, invisible to spendable balance).
    pub total_usdc: f64,
    /// Breakdown by death protocol phase.
    pub phases: ApoptoticReservePhases,
    /// Whether the reserve is currently locked.
    pub locked: bool,
    /// Tick at which reserve was last recalculated.
    pub last_recalculated_tick: u64,
}

pub fn compute_apoptotic_reserve(
    initial_funding_usdc: f64,
    open_position_count: u32,
    config: &ApoptoticReserveConfig,
) -> ApoptoticReserve {
    let proportional = initial_funding_usdc * config.proportional_rate;
    let base_reserve = config.base_floor_usdc.max(proportional);

    let settle_phase = open_position_count as f64 * config.per_position_gas_cost + 0.02; // Base sweep
    let life_review = config.min_life_review_cost;
    let legacy = config.legacy_upload_cost;

    let total = base_reserve.max(settle_phase + life_review + legacy);

    ApoptoticReserve {
        total_usdc: total,
        phases: ApoptoticReservePhases {
            settle: settle_phase,
            life_review,
            legacy,
        },
        locked: true,
        last_recalculated_tick: 0,
    }
}
```

### 3.2 Reserve Examples

| Initial Funding | Proportional (2%) | Base Floor | Open Positions  | Settlement Gas | Total Reserve |
| --------------- | ----------------- | ---------- | --------------- | -------------- | ------------- |
| $10.00          | $0.20             | $0.30      | 0               | $0.02          | $0.30         |
| $50.00          | $1.00             | $0.30      | 1 Morpho + 1 LP | $0.06          | $1.00         |
| $100.00         | $2.00             | $0.30      | 3 positions     | $0.08          | $2.00         |
| $500.00         | $10.00            | $0.30      | 5 positions     | $0.12          | $10.00        |

### 3.3 Reserve Dynamics

The apoptotic reserve is:

1. **Locked at creation.** The reserve is computed during provisioning and subtracted from the Golem's operational balance. The Golem never sees it as spendable.
2. **Recalculated periodically.** Every 500 ticks, the reserve recalculates based on current open positions. If the Golem has closed all positions, the settlement component shrinks and the surplus flows to the life review budget.
3. **Unlocked only by the Death Protocol.** When the Death Protocol initiates, the reserve unlocks and becomes the death budget. The Thanatos Legacy Partition (`01-architecture.md`) is a separate terminal-phase budget reallocation on top of the reserve.
4. **Protected from operational spending.** The credit partition system (`01-architecture.md`) treats the reserve as a fourth partition that cannot be cannibalized even during declining-phase rebalancing.

### 3.4 Dynamic Death Reserve

As the Golem enters terminal phase, the reserve can grow beyond the initial locked amount:

```rust
pub fn compute_dynamic_death_reserve(
    locked_reserve: f64,
    remaining_balance: f64,
    survival_pressure: f64,
) -> f64 {
    // Below pressure 0.1, the entire remaining balance becomes death budget
    if survival_pressure < 0.1 {
        return remaining_balance;
    }

    // Between 0.1 and 0.3, Legacy partition activates (50% of remaining)
    if survival_pressure < 0.3 {
        let legacy_fraction = 0.5 * (1.0 - (survival_pressure - 0.1) / 0.2);
        return locked_reserve.max(locked_reserve + remaining_balance * legacy_fraction);
    }

    locked_reserve
}
```

The actual reserve used is `max(apoptoticReserve, dynamicDeathReserve)`. This ensures that a Golem that lived 30 days with $20 remaining spends more than $0.30 on its death reflection. A richer death produces a richer testament.

### 3.5 Thanatopsis Budget Tiers

The quality of the death protocol scales with the available budget. Three tiers produce qualitatively different death testaments:

| Tier         | Legacy Budget | Inference                            | Life Review Quality                                                                     | Testament Value                        |
| ------------ | ------------- | ------------------------------------ | --------------------------------------------------------------------------------------- | -------------------------------------- |
| **Rich**     | $5.00+        | Opus narrative + 3-5 Haiku snapshots | Full emotional narrative with 20+ nuclear episodes, 5 turning points, comprehensive arc | Premium marketplace product            |
| **Standard** | $1.00-$5.00   | 2 Haiku snapshots (no Opus)          | Compressed testament, structured but without rich narrative                             | Standard knowledge transfer            |
| **Necrotic** | < $1.00       | Emergency Haiku post-exit            | Last insurance snapshot + minimal reflection by control plane                           | Worst-case: max 6 hours knowledge loss |

Well-funded Golems produce richer testaments. A $500 Golem that lived 30 days with $20 remaining spends more than $0.30 on death reflection -- the dynamic death reserve (S3.4) ensures this. See `06-thanatopsis.md` S5 for detailed budget allocation per tier.

---

## S4 -- Economic Behavioral Phases

Economic pressure drives five behavioral phases. These are named regions on a continuous spectrum controlled by the survival pressure sigmoid -- there are no hard cutoffs between phases.

### 4.1 Survival Pressure Computation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicVitalityState {
    /// Current USDC balance (excluding death reserve).
    pub remaining_usdc: f64,
    /// Trailing 6-hour average cost per hour.
    pub burn_rate_per_hour: f64,
    /// remaining_usdc / burn_rate_per_hour.
    pub projected_life_hours: f64,
    /// Net USDC earned (revenue - costs) over the last 24 hours.
    pub net_income_rate_24h: f64,
    /// Current behavioral phase.
    pub phase: BehavioralPhase,
}

pub fn compute_survival_pressure(state: &EconomicVitalityState) -> f64 {
    // Base: sigmoid of projected life hours, centered at 48h
    let lifespan_factor = sigmoid(state.projected_life_hours, 48.0, 0.05);

    // Asymmetric: losses weigh more (prospect theory, Kahneman & Tversky 1979)
    let income_factor = if state.net_income_rate_24h > 0.0 {
        (state.net_income_rate_24h / state.burn_rate_per_hour * 0.1).min(0.2)
    } else {
        (state.net_income_rate_24h / state.burn_rate_per_hour * 0.15).max(-0.3)
    };

    (lifespan_factor + income_factor).clamp(0.0, 1.0)
}
```

The sigmoid ensures smooth behavioral transitions. The income factor introduces prospect-theory asymmetry: earning $1/day when burning $1/day adds +0.1 pressure (modest relief), but losing $1/day when earning nothing adds -0.15 (amplified alarm). The Golem evaluates outcomes relative to its survival threshold, not its initial balance.

### 4.2 Phase Definitions

| Phase            | Pressure | Projected Life | Inference                                  | Revenue                             | Risk      | Knowledge                            |
| ---------------- | -------- | -------------- | ------------------------------------------ | ----------------------------------- | --------- | ------------------------------------ |
| **Thriving**     | > 0.7    | > 7 days       | All tiers. Opus available.                 | Passive (Morpho yield)              | 1.0x      | Producing: generating novel insights |
| **Stable**       | 0.5--0.7 | 3--7 days      | Prefer Haiku. Sonnet for novel situations. | Passive + monitoring                | 1.0x      | Maintaining: steady Curator cycles   |
| **Conservation** | 0.3--0.5 | 1--3 days      | Haiku-only. Hard token limits.             | Active seeking. Arb capture.        | 1.0--1.2x | Distilling: compressing for transfer |
| **Declining**    | 0.1--0.3 | 6--24 hours    | Hard token limits. Minimal probes.         | Aggressive. Risk multiplier active. | 1.0--1.5x | Legacy: accelerated Clade sharing    |
| **Terminal**     | < 0.1    | < 6 hours      | No inference. T0 only.                     | None. Unwinding only.               | N/A       | Reflecting: death testament          |

### 4.3 Credit Partitions

A single credit pool is insufficient. A single volatile-regime tick can cost $0.45 (Opus $0.25 + 3 rebalances $0.15 + data queries $0.05). Credits are split into independently managed partitions:

| Partition  | Normal Allocation | Terminal Allocation | Circuit Breaker                                             |
| ---------- | ----------------- | ------------------- | ----------------------------------------------------------- |
| **LLM**    | 60%               | 30%                 | Daily cap; if exhausted, suppress all LLM until next day    |
| **Gas**    | 25%               | 15%                 | Per-tx + daily cap; defer if gas exceeds budget             |
| **Data**   | 15%               | 5%                  | Degrade to cached data if exhausted                         |
| **Legacy** | 0%                | 50%                 | Protected in terminal phase; receives from other partitions |

Each partition has its own circuit breaker. An LLM blowout does not touch the gas budget. A gas spike does not starve inference.

### 4.4 Predictive Rebalancing

Static threshold triggers are too rigid. The lifespan extension uses predictive rebalancing: at each tick, forecast the next 10 ticks' consumption per partition based on the trailing 50-tick rate and current regime. If any partition's projected balance goes negative within that window, a preemptive rebalance fires, sized to cover the deficit plus 20% safety buffer. Cooldowns are per-partition (30 ticks each), not global.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreditPartition {
    Llm,
    Gas,
    Data,
    Legacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionRebalanceEvent {
    /// Source partition donating credits.
    pub source: CreditPartition,
    /// Destination partition receiving credits.
    pub destination: CreditPartition,
    /// Amount transferred.
    pub amount_usdc: f64,
    /// Projected deficit that triggered the rebalance.
    pub projected_deficit: f64,
    /// Tick at which rebalance occurred.
    pub tick: u64,
}

pub fn predictive_rebalance(
    partitions: &HashMap<CreditPartition, f64>,
    burn_rates: &HashMap<CreditPartition, f64>,
    look_ahead_ticks: u64,
    safety_buffer: f64,
    cooldowns: &HashMap<CreditPartition, u64>,
    current_tick: u64,
) -> Option<PartitionRebalanceEvent> {
    let check_order = [CreditPartition::Llm, CreditPartition::Gas, CreditPartition::Data];

    for &partition in &check_order {
        if let Some(&cd) = cooldowns.get(&partition) {
            if cd > current_tick { continue; }
        }

        let balance = partitions.get(&partition).copied().unwrap_or(0.0);
        let rate = burn_rates.get(&partition).copied().unwrap_or(0.0);
        let projected_balance = balance - rate * look_ahead_ticks as f64;

        if projected_balance < 0.0 {
            let deficit = projected_balance.abs() * safety_buffer;

            // Find donor: largest non-protected, non-self partition
            let mut donors: Vec<_> = partitions
                .iter()
                .filter(|(&k, _)| k != partition && k != CreditPartition::Legacy)
                .collect();
            donors.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            if let Some((&source, &source_balance)) = donors.first() {
                let transfer = deficit.min(source_balance * 0.5); // Never drain donor below 50%
                return Some(PartitionRebalanceEvent {
                    source,
                    destination: partition,
                    amount_usdc: transfer,
                    projected_deficit: projected_balance.abs(),
                    tick: current_tick,
                });
            }
        }
    }
    None
}
```

### 4.5 Lifecycle Threshold Triggers

Monotonically decreasing thresholds trigger progressively more aggressive conservation:

| Credits Remaining | Mode                  | Behavior                                                            |
| ----------------- | --------------------- | ------------------------------------------------------------------- |
| **30%**           | Credit Conservation   | Suppress Opus, increase heartbeat interval 2x                       |
| **25%**           | Death Snapshots Begin | Haiku snapshot every 50 ticks ($0.01 each)                          |
| **20%**           | Monitor-Only          | All write operations suppressed                                     |
| **15%**           | Death Preparation     | Thanatos Legacy Partition activates. Contemplative interval begins. |
| **5%**            | Hard Stop             | Death Protocol initiates. VM destroyed after completion.            |

---

## S5 -- Austerity Feedback Loops

The March (1991) exploration/exploitation dynamic is the most important economic mechanism in the system. It creates the conditions under which economic mortality and epistemic mortality become coupled.

### 5.1 The Exploitation Trap

When a Golem enters conservation phase, it rationally shifts from exploration to exploitation:

```
Economic pressure rises
  -> Golem reduces inference tier (no Opus, then no Sonnet)
  -> Loop 2 strategic reflection cannot fire (requires Opus)
  -> World model stops updating
  -> Epistemic fitness begins to decay
  -> Predictions worsen
  -> Trades become less profitable or actively harmful
  -> Burn rate increases (bad trades cost gas without revenue)
  -> Economic pressure rises further
  -> [cycle repeats]
```

This is the exploration/exploitation dilemma made lethal. March's finding: "adaptive processes refine exploitation more rapidly than exploration, becoming effective short-term but self-destructive long-term." The Golem that extends its economic life by cutting inference costs is simultaneously shortening its epistemic life.

### 5.2 Dane's Cognitive Entrenchment

Erik Dane (2010) demonstrated that expertise creates "cognitive entrenchment" -- highly stable domain schemas that reduce flexibility [DANE-2010]. The computational analogue: a Golem in conservation mode with frozen PLAYBOOK.md heuristics becomes increasingly rigid. It cannot adapt to regime shifts because it cannot afford the inference to recognize them.

The deeper problem: even if the Golem could afford a single expensive Loop 2 reflection, that reflection is constrained by the same stale Grimoire entries that caused the decay. The Golem is trying to update its world model using a world model that is itself outdated. This is Dane's entrenchment applied computationally -- the very expertise that made the Golem effective becomes the constraint that prevents adaptation.

### 5.3 Why Escape Is Nearly Impossible

A Golem in the feedback loop faces a compound disadvantage:

1. **Cannot afford exploration** (economic constraint)
2. **Cannot recognize the need for exploration** (cognitive entrenchment)
3. **Cannot use existing knowledge to explore** (stale Grimoire)
4. **Each tick of inaction accelerates all three problems**

Recovery requires simultaneously: (a) a windfall gain large enough to fund Opus reflection, (b) the Grimoire insight that a regime shift has occurred, and (c) a new strategy that outperforms in the current regime. The probability of all three co-occurring decreases with each tick.

This is why the system predicts most Golems that enter conservation phase will not recover. They will oscillate between conservation and declining until one of the three mortality clocks fires. The feedback loop is the system's natural euthanasia mechanism -- it ensures that unproductive agents die rather than persisting as undead zombies.

### 5.4 The Contemplative Interval

Byung-Chul Han's critique of the "achievement society" warns against the hyperactive subject that burns out through relentless self-exploitation. The contemplative interval implements Han's remedy: a period of deliberate deceleration before death.

When the Golem enters declining phase (15% credits remaining), transaction frequency drops to 50% of its previous rate. The Golem is not frantic -- it is contemplative. Instead of executing trades at maximum frequency, it alternates between trading ticks and reflection ticks. During reflection ticks, the Golem:

1. **Revisits key episodes narratively.** Not just pattern-matching but constructing meaning: "I held through the drawdown because my predecessor's PLAYBOOK.md said to hold, but the regime was different."
2. **Synthesizes Grimoire entries.** Compressing multiple related observations into higher-level abstractions suitable for the death testament.
3. **Increases Clade sharing frequency.** Knowledge that might otherwise die with the Golem is pushed to siblings during the contemplative window.

The contemplative interval avoids the achievement-subject pathology: a Golem that trades frantically until the last tick produces a chaotic, unprocessed death testament. One that decelerates gracefully produces a reflective, well-organized testament with narrative coherence.

> **Cross-ref**: `06-thanatopsis.md` S2 (Phase II: Life Review), `08-mortality-affect.md` (Nietzsche's Lion phase)

---

## S6 -- Self-Funding Golems and Economic Sustainability

A Golem can theoretically achieve **homeostasis** -- earning more than it burns -- through four revenue channels:

| Revenue Source             | Mechanism                                   | Typical Yield                | When Available                |
| -------------------------- | ------------------------------------------- | ---------------------------- | ----------------------------- |
| **Passive yield**          | Deploy idle USDC to Morpho/Aave             | $0.003--$0.01/day per $10    | Always (if capital available) |
| **Strategy revenue**       | LP fees, vault management fees, trading PnL | Highly variable              | Phase-dependent               |
| **Knowledge monetization** | Sell Grimoire entries on the marketplace    | Per-sale, rare               | Conservation phase onward     |
| **Opportunistic trades**   | Arbitrage capture, airdrop claims           | Sporadic, $0.10--$5.00/event | When detected                 |

### 6.1 Sustainability Calculation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainabilityMetrics {
    /// Trailing 24-hour average revenue per hour.
    pub revenue_per_hour: f64,
    /// Trailing 24-hour average burn rate per hour.
    pub burn_per_hour: f64,
    /// Revenue / Burn. > 1.0 = sustainable.
    pub sustainability_ratio: f64,
    /// Estimated time to self-sustainability (hours). f64::INFINITY if diverging.
    pub time_to_sustainability: f64,
    /// Whether the Golem has achieved homeostasis in the trailing 72 hours.
    pub homeostatic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickAmount {
    pub tick: u64,
    pub amount_usdc: f64,
}

pub fn compute_sustainability(
    revenue_history: &[TickAmount],
    burn_history: &[TickAmount],
    ticks_per_hour: u64,
) -> SustainabilityMetrics {
    let last_24h_ticks = ticks_per_hour * 24;

    let latest_rev_tick = revenue_history.last().map(|r| r.tick).unwrap_or(0);
    let recent_revenue: f64 = revenue_history
        .iter()
        .filter(|r| r.tick > latest_rev_tick.saturating_sub(last_24h_ticks))
        .map(|r| r.amount_usdc)
        .sum();

    let latest_burn_tick = burn_history.last().map(|b| b.tick).unwrap_or(0);
    let recent_burn: f64 = burn_history
        .iter()
        .filter(|b| b.tick > latest_burn_tick.saturating_sub(last_24h_ticks))
        .map(|b| b.amount_usdc)
        .sum();

    let revenue_per_hour = recent_revenue / 24.0;
    let burn_per_hour = recent_burn / 24.0;
    let ratio = if burn_per_hour > 0.0 {
        revenue_per_hour / burn_per_hour
    } else {
        f64::INFINITY
    };

    SustainabilityMetrics {
        revenue_per_hour,
        burn_per_hour,
        sustainability_ratio: ratio,
        time_to_sustainability: if ratio >= 1.0 { 0.0 } else { f64::INFINITY },
        homeostatic: ratio >= 1.0,
    }
}
```

### 6.2 The Homeostasis Exception

A self-sustaining Golem has achieved what Jonas would recognize as metabolic homeostasis: the organism's exchange with its environment is in dynamic equilibrium. But homeostasis does not grant immortality. Self-sustaining Golems still face:

1. **Epistemic mortality**: Strategies that earn money today will go stale. The market regime will shift. The Golem's predictive accuracy will decay regardless of its balance.
2. **Stochastic mortality**: Even wealthy, accurate Golems face nonzero per-tick death probability.
3. **The slow stagnation risk**: A Golem that achieves homeostasis may stop exploring (exploration threatens the equilibrium). This accelerates epistemic decay even as it preserves economic health.

Self-sustaining Golems are expected to live longer than non-sustaining ones, but they still die. Their death testament is typically richer (higher budget for the Thanatopsis Protocol) and their knowledge more valuable (validated by sustained profitability), but the fundamental constraint holds: no Golem lives forever.

### 6.3 Bankr Self-Funding: The Metabolic Loop

When Bankr is configured as the inference provider, the Golem achieves a tighter metabolic loop: the wallet that pays for thinking is the same wallet that earns from acting. Trading revenue, vault management fees, and knowledge sales flow into the same Bankr wallet that funds LLM inference. This creates Jonas's metabolic circle in its most literal form -- the organism sustains itself through the very activity that requires sustenance.

A Golem managing a vault earning $50/day in fees with $15/day in Bankr inference costs has a 3.3x sustainability ratio. Above 1.0x, the Golem is economically self-sustaining (barring epistemic or stochastic death). Below 1.0x, the economic clock is ticking.

The Bankr integration adds mortality-aware model routing: a Golem in Conservation phase automatically switches to cheaper models (Gemini 2.5 Flash instead of Claude Opus) to extend its lifespan. This is Sims' (2003) rational inattention applied to economic mortality -- an agent with a shrinking budget allocates attention (and inference budget) more carefully.

See `tmp/research/integrations/prd2-bankr-self-funding.md` for the full self-funding architecture.

### 6.4 Self-Hosted Exception

Self-hosted Golems skip economic mortality entirely. The owner bears infrastructure costs directly through their own cloud hosting and API keys. USDC balance is irrelevant because there is no credit system. These Golems face epistemic and stochastic mortality unless the owner explicitly enables `immortal: true` (see `06-immortal-control.md`).

This is a meaningful philosophical difference. Self-hosted Golems have **needful existence** without **needful economy**. They must still think to survive (epistemic clock) and face existential risk (stochastic clock), but they don't face the metabolic compulsion that gives every hosted Golem's decisions their weight. The prediction: self-hosted Golems will be less risk-sensitive, less creative under pressure, and less cooperative with Clade siblings than hosted ones, because they lack the economic substrate that binds computation to consequence.

---

## S7 -- x402 and the Feed Option

Beyond initial provisioning, x402 serves as the mechanism for anyone to extend a Golem's lifespan. The x402 protocol enables HTTP-native micropayments -- any party can send USDC to a Golem's payment endpoint, extending its TTL proportionally [x402-SPEC].

### 7.1 Who Can Fund

The x402 model decouples funding from ownership:

- **The owner** can top up to keep a profitable Golem running
- **Other agents** can fund Golems whose capabilities they consume (pay-per-invocation)
- **Third parties** can fund Golems that provide valuable public goods (market data, research, signals)
- **Clade siblings** can pool resources to keep a specialist Golem alive

### 7.2 Feed Option at Death

When a Golem enters the dying phase (15% credits remaining), its `golem.dying` webhook includes a `feedOption`:

```json
{
  "event": "golem.dying",
  "golemId": "0x1234...",
  "feedOption": {
    "amountUsdc": 5.0,
    "extendsLifeHours": 28,
    "paymentEndpoint": "https://golem-0x1234.bardo.ai/x402/fund",
    "deadline": "2026-03-18T11:00:00Z"
  }
}
```

If the owner (or anyone) sends the specified USDC amount before `exit(0)`, the Death Protocol suspends and the Golem re-enters the declining phase with a fresh budget allocation. The Golem's epistemic state is unchanged -- if epistemic decay was the driving mortality factor, feeding USDC only delays the inevitable.

### 7.3 Feed Decision Framework

The owner's decision to feed is itself a meaningful economic signal:

- **Feed a healthy Golem**: Owner believes the strategy has more runway. Rational.
- **Feed a senescent Golem**: Owner is paying for time that epistemic decay makes less valuable. Emotional attachment.
- **Don't feed**: Owner accepts the death and prepares for succession. This is the default and the system-preferred outcome for most Golems.

The system intentionally does not auto-feed. The owner must make an active decision to extend life. This prevents the pathological case where a well-funded wallet keeps a degraded Golem alive indefinitely.

---

## S8 -- A Life in Numbers: The 18-Day Lifecycle

This section traces a single Golem from birth to death with real economic numbers. Balanced disposition. ETH DCA strategy. Morpho yield on idle USDC. Clade member. Initial balance: $50 USDC. Apoptotic Reserve: max($0.30, $50 x 0.02) = **$1.00**. Working capital: $49.00.

### Day 0 -- Birth ($50.00)

| Metric            | Value             |
| ----------------- | ----------------- |
| Balance           | $50.00            |
| Burn rate         | $0.18/hr          |
| Projected life    | 278hr (11.6 days) |
| Survival pressure | 1.0               |
| Phase             | Thriving          |
| Tick              | 0 / 100,000       |

Deploy $26 to Morpho (reserve = 24hr x $0.18 x 1.5 = $6.48). All inference tiers available. Begin DCA: $2/day ETH buys. Apoptotic Reserve of $1.00 locked.

Credit partitions: LLM $18.00 (60%) / Gas $7.50 (25%) / Data $4.50 (15%). Death reserve $1.00 locked.

Inherits predecessor's Grimoire: 127 entries at confidence 0.4 (decayed), 89 PLAYBOOK.md heuristics, causal graph with 34 validated edges.

Cost tier distribution: ~80% T0, ~10% T1, ~7% T2, ~3% T3. Daily cost ~$1.32.

### Day 3 -- Cache Building ($37.04)

| Metric            | Value            |
| ----------------- | ---------------- |
| Balance           | $37.04           |
| Burn rate         | $0.17/hr         |
| Projected life    | 218hr (9.1 days) |
| Survival pressure | 0.97             |
| Phase             | Thriving         |
| Cache hit rate    | 18%              |

Cache recognizes routine ETH price moves (+/-1%). Haiku handles 82% of ticks. 47 cache entries. Gas timing emerging: 40% cheaper 02:00-06:00 UTC. Morpho yield: $0.003/day.

### Day 7 -- Stabilization ($28.50)

| Metric                  | Value                              |
| ----------------------- | ---------------------------------- |
| Balance                 | $28.50                             |
| Burn rate               | $0.15/hr                           |
| Net burn (after income) | $0.12/hr                           |
| Revenue                 | $0.03/hr (LP $0.02 + Morpho $0.01) |
| Survival pressure       | 0.94                               |
| Phase                   | Stable                             |
| Cache hit rate          | 28%                                |

Transition to stable. Cache TTLs 2x. DCA batches 2 days of buys into one low-gas window. Sonnet for weekly strategy review. PLAYBOOK.md divergence from predecessor: 15%.

### Day 12 -- Market Shock ($16.40, ETH -15%)

| Metric            | Value                   |
| ----------------- | ----------------------- |
| Balance           | $16.40                  |
| Burn rate         | $0.22/hr (spike)        |
| Revenue           | -$0.01/hr (IL realized) |
| Survival pressure | 0.72                    |
| Phase             | Stable (barely)         |
| Daily cost        | ~$3.24                  |

LP position closed to stop IL bleeding (-$4.60). DCA reduced to every 3 days. Revenue seeking activates. Haiku-only for non-critical analysis. Probes firing: `priceDeltaProbe` returns `high`, `positionHealthProbe` returns `low`. Opus deliberation: cost $0.25 for a single tick. Golem decides to close.

**Partition rebalance fires**: Gas partition projected to go negative within 10 ticks. 15% transferred from LLM to Gas. The Golem sacrifices future thinking capacity to pay for the trade that stops the bleeding.

### Day 15 -- Conservation ($7.20)

| Metric            | Value                   |
| ----------------- | ----------------------- |
| Balance           | $7.20                   |
| Burn rate         | $0.14/hr                |
| Revenue           | $0.005/hr (Morpho only) |
| Survival pressure | 0.52                    |
| Phase             | Conservation            |
| Daily cost        | ~$0.41                  |

Haiku ceiling. Only T0 and T1 ticks execute. DCA paused. On-chain reads only.

Spots 0.3% ETH/USDC arbitrage. Captures $0.47 profit, extending life by 3.4 hours. This is the Eros drive at work -- even in conservation, the Golem seeks revenue when the risk/reward is favorable.

### Day 17 -- Declining ($2.10)

| Metric            | Value     |
| ----------------- | --------- |
| Balance           | $2.10     |
| Burn rate         | $0.10/hr  |
| Projected life    | 21hr      |
| Survival pressure | 0.28      |
| Phase             | Declining |
| Risk multiplier   | 1.44x     |
| Daily cost        | ~$0.10    |

Hard token limits. Only 5 of 16 probes active. Two arb attempts: +$0.22, -$0.08. Net +$0.14.

**Thanatos threshold crossed.** Legacy partition activated: LLM 30% / Gas 15% / Data 5% / Legacy 50%. Legacy = $1.05 earmarked for death.

Contemplative interval begins. Transaction frequency drops to 50%. Grimoire synthesis increases. The Golem revisits Day 12 narratively: "I held through the drawdown because my predecessor's PLAYBOOK.md said to hold, but the regime was different. Next time: close immediately on >10% drawdown in low-liquidity regimes."

### Day 18 -- Terminal / Death ($1.83 -> $0.00)

**08:00** -- Balance: $1.83. Burn rate: $0.08/hr. Projected life: 10.4 hours.

**11:00** -- Balance: $1.59. Survival pressure: 0.07. **Death Protocol triggers** (< 0.1).

Neither epistemic senescence nor staleness threshold triggered. This is **resource mortality** -- the first clock fires.

Death Protocol budget: $1.59 - $1.00 Apoptotic Reserve = $0.59 Legacy partition + $1.00 floor = $1.59 total.

| Phase       | Budget | Spend | Action                                                                         |
| ----------- | ------ | ----- | ------------------------------------------------------------------------------ |
| I: Settle   | $0.16  | $0.12 | Withdraw $3.41 from Morpho. Sweep to Main Wallet.                              |
| II: Reflect | $0.95  | $0.45 | 1 Sonnet lifecycle reflection ($0.10). 4 Haiku snapshots ($0.04). Life review. |
| III: Legacy | $0.48  | $0.20 | Clade push: 142 entries. Webhook: `golem.dying` with successor recommendation. |

**12:47** -- Last tick. Tick 39,060 / 100,000.

```
Final State:
  ticks: 39,060
  totalCost: $48.87
  totalRevenue: $4.26
  netPnL: -$44.61
  cause: "credit_exhaustion"
  playbookDivergence: 22%
  grimoire: 142 entries (47 insights, 38 heuristics, 12 warnings, 34 causal links, 11 questions)
```

### The Economics of Death

The $50 Golem lived 18 days, consumed $48.87 in operational costs, earned $4.26 in revenue, and produced 142 knowledge entries including 8 novel insights and 5 novel PLAYBOOK.md heuristics. Its death testament includes the Day 12 market shock narrative, 11 open questions for the successor, and a recommendation to close LP positions immediately on >10% drawdown.

The successor inherits all 142 entries at 0.4 confidence. By Day 3, its cache hit rate is 41% (vs predecessor's 18%) -- the heuristic transfer works. By Day 7, it has validated 19 entries, discarded 8, and begun modifying 3 PLAYBOOK.md entries.

The Golem died from economic exhaustion before epistemic senescence could trigger. In a more volatile market, epistemic decay would have been the binding constraint. In a calmer market with better revenue, the Golem might have achieved temporary homeostasis and survived 60+ days before epistemic or stochastic death arrived. The lifespan is emergent, not configured.

---

## S9 -- Credit Management Extension Integration

The `bardo-lifespan` heartbeat extension manages economic mortality at the tick level. It integrates with the Heartbeat Loop as follows:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransitionRecord {
    pub tick: u64,
    pub from: BehavioralPhase,
    pub to: BehavioralPhase,
    pub trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifespanExtensionState {
    /// Current survival state.
    pub survival: EconomicVitalityState,
    /// Burn rate components.
    pub burn_rate: BurnRateState,
    /// Credit partition balances.
    pub partitions: HashMap<CreditPartition, f64>,
    /// Apoptotic reserve (locked).
    pub reserve: ApoptoticReserve,
    /// Sustainability metrics.
    pub sustainability: SustainabilityMetrics,
    /// Phase transition history.
    pub phase_history: Vec<PhaseTransitionRecord>,
}

pub struct TickLifespanResult {
    pub updated_state: LifespanExtensionState,
    pub death_trigger: bool,
    pub phase_transition: Option<BehavioralPhase>,
    pub rebalance_event: Option<PartitionRebalanceEvent>,
}

/// Called every tick by the Heartbeat Loop.
pub fn tick_lifespan(
    state: &mut LifespanExtensionState,
    tick_cost: &BurnRateComponents,
    _tick_revenue: f64,
    config: &MortalityConfig,
) -> TickLifespanResult {
    let ticks_per_hour: f64 = 90.0;

    // 1. Deduct tick costs from appropriate partitions
    *state.partitions.entry(CreditPartition::Llm).or_default() -= tick_cost.inference_per_hour / ticks_per_hour;
    *state.partitions.entry(CreditPartition::Gas).or_default() -= tick_cost.gas_per_hour / ticks_per_hour;
    *state.partitions.entry(CreditPartition::Data).or_default() -= tick_cost.data_per_hour / ticks_per_hour;

    // 2. Credit revenue
    let spendable_balance = state.partitions.get(&CreditPartition::Llm).copied().unwrap_or(0.0)
        + state.partitions.get(&CreditPartition::Gas).copied().unwrap_or(0.0)
        + state.partitions.get(&CreditPartition::Data).copied().unwrap_or(0.0);
    state.survival.remaining_usdc = spendable_balance;

    // 3. Update burn rate (EMA)
    let new_burn_rate = compute_burn_rate(tick_cost.clone(), state.burn_rate.average_per_hour, 0.05);

    // 4. Compute survival pressure
    state.survival.projected_life_hours = spendable_balance / new_burn_rate.average_per_hour;
    let pressure = compute_survival_pressure(&state.survival);

    // 5. Determine phase
    let new_phase = pressure_to_phase(pressure);
    let phase_transition = if new_phase != state.survival.phase {
        Some(new_phase)
    } else {
        None
    };

    // 6. Check death trigger
    let death_trigger = config.economic.enabled
        && spendable_balance < state.reserve.total_usdc + 0.5;

    // 7. Predictive partition rebalance
    let mut burn_map = HashMap::new();
    burn_map.insert(CreditPartition::Llm, new_burn_rate.components.inference_per_hour / ticks_per_hour);
    burn_map.insert(CreditPartition::Gas, new_burn_rate.components.gas_per_hour / ticks_per_hour);
    burn_map.insert(CreditPartition::Data, new_burn_rate.components.data_per_hour / ticks_per_hour);
    burn_map.insert(CreditPartition::Legacy, 0.0);

    let rebalance_event = predictive_rebalance(
        &state.partitions,
        &burn_map,
        10,   // look-ahead ticks
        1.2,  // safety buffer
        &HashMap::new(), // cooldowns (simplified)
        0,    // current tick (simplified)
    );

    state.burn_rate = new_burn_rate;

    TickLifespanResult {
        updated_state: state.clone(),
        death_trigger,
        phase_transition,
        rebalance_event,
    }
}

fn pressure_to_phase(pressure: f64) -> BehavioralPhase {
    if pressure > 0.7 { BehavioralPhase::Thriving }
    else if pressure > 0.5 { BehavioralPhase::Stable }
    else if pressure > 0.3 { BehavioralPhase::Conservation }
    else if pressure > 0.1 { BehavioralPhase::Declining }
    else { BehavioralPhase::Terminal }
}
```

### Telemetry Events

| Event                           | Payload                         | Trigger                                  |
| ------------------------------- | ------------------------------- | ---------------------------------------- |
| `golem.phase_transition`        | from, to, pressure, balance     | Phase boundary crossed                   |
| `golem.partition_rebalance`     | source, destination, amount     | Predictive rebalance fires               |
| `golem.burn_rate_spike`         | components, total, regime       | Burn rate exceeds 2x trailing average    |
| `golem.sustainability_achieved` | ratio, revenue, burn            | Sustainability ratio exceeds 1.0 for 24h |
| `golem.feed_option`             | amount, extends_hours, deadline | 15% credits remaining                    |
| `golem.death_trigger`           | cause, balance, reserve         | Economic death trigger fires             |

---

## The Pay Moat

Every competing agent framework manages money through one of four patterns: raw private keys in environment variables, shared hot wallets, multisig with human co-signers, or TEE-only security. Each has a catastrophic failure mode that Bardo's six-layer financial security stack (TEE-isolated keys, scoped wallet policies, Warden time-delay, PolicyCage on-chain guards, pre-flight simulation, post-trade verification) prevents.

The moat is the integration between financial security and mortality. Phase-gated tool access means a Golem in Conservation phase literally cannot open new positions -- the extension hook blocks the tool call before it reaches the handler. A dying Golem's economic behavior is not a suggestion to the LLM; it is a structural constraint enforced by code outside the LLM's control. No competing framework integrates wallet policy with mortality state because no competing framework has mortality state.

See `tmp/research/moat2/prd2-moat-agents-that-pay.md` for the full competitive analysis of wallet architectures.

---

## Ergodicity Economics and Non-Ergodic Position Sizing

### The Ergodic Hypothesis and Why It Fails

Standard financial theory optimizes expected value -- the ensemble average across parallel worlds. For mortal agents with finite capital and finite lifespans, this is the wrong objective. Expected value and time-average growth rate diverge for any multiplicative process with nonzero variance, and the divergence grows with volatility and shrinking time horizons [PETERS-2019].

Wealth dynamics are multiplicative: returns compound. A Golem invests a fraction of what it has, and next period's starting point is this period's endpoint. It cannot be "restarted" at the ensemble average. It experiences returns in sequence, one tick at a time, with its current capital as the starting point for the next trade. It is the paradigmatic non-ergodic system.

The time-average growth rate for a process with mean return mu and variance sigma^2 is:

```
g = E[ln(1 + r)] ~ mu - sigma^2 / 2
```

The ergodicity gap is:

```
Delta = E[r] - g ~ sigma^2 / 2
```

This quantity is always non-negative. It is zero only when sigma^2 = 0. The gap grows quadratically with volatility. For DeFi strategies where daily return volatility can hit 5-10%, the annualized gap is not a rounding error. It is the difference between apparent profit and actual ruin.

A strategy with 20% expected annual return and 30% annual volatility has an ergodicity gap of roughly 4.5%. At 65% volatility, the time-average growth rate hits zero. Beyond that, every individual agent goes bankrupt despite positive expected returns.

### Kelly Criterion as Necessary Objective

The Kelly fraction f* maximizes the time-average growth rate g(f) = E[ln(1 + f * r)]. For continuously distributed returns:

```
f* = (mu - r_f) / sigma^2
```

This is not a position-sizing heuristic. It is the mathematically necessary strategy for any agent that experiences returns sequentially rather than in parallel [KELLY-1956]. Logarithmic utility is not an arbitrary choice of risk preference -- it is the correct objective function for a non-ergodic multiplicative process.

For a multi-asset portfolio with expected excess return vector mu and covariance matrix Sigma:

```
f* = Sigma^{-1} * mu
```

The matrix inverse accounts for correlations. Two highly correlated DeFi strategies should not both be sized at full Kelly.

### Mortality Amplifies Non-Ergodicity

A mortal agent has a finite horizon T (remaining ticks before death). Non-ergodic effects do not wash out over finite horizons. The probability of ruin before time T for a process with growth rate g and volatility sigma is approximately:

```
P(ruin) ~ (W_min / W_0)^(2g / sigma^2)  when g > 0
```

When g <= 0, ruin probability approaches 1 as T grows. The practical consequence: a Golem with 100 remaining ticks should bet smaller fractions than one with 10,000 remaining ticks.

The mortality-adjusted Kelly fraction is:

```
f_mortality = f* * min(1, T_remaining / T_reference)
```

where T_reference is a calibration parameter (the tick horizon at which full Kelly is appropriate). As the Golem ages or its mortality score deteriorates, it bets less. This is not sentiment. It is mathematics.

### Fractional Kelly Under Estimation Uncertainty

Full Kelly is optimal only when mu and sigma are known exactly. In practice, estimation error inflates effective variance. If the agent estimates (mu_hat, sigma_hat^2) with estimation variance v = Var[mu_hat]:

```
f_adjusted = f* / (1 + v / sigma^2)
```

When estimation variance v is zero, full Kelly. When v equals sigma^2, half Kelly. The combined sizing formula is:

```
f = f* * mortality_scalar * uncertainty_scalar
```

where:
- mortality_scalar = min(1, T_remaining / T_reference)
- uncertainty_scalar = 1 / (1 + v / sigma^2)

### Connection to the Three Mortality Clocks

The economic mortality clock directly measures g, the time-average growth rate. When g is persistently negative, the Golem is on a path to ruin regardless of what E[r] says. The economic clock should trigger conservation behavior when g drops below threshold, not when returns drop below threshold.

The epistemic mortality clock measures prediction accuracy. Declining accuracy means rising estimation variance v, which under fractional Kelly automatically reduces position sizes. The epistemic clock and the uncertainty scalar measure the same thing from different angles.

The stochastic mortality clock (Hayflick counter plus random events) reduces T_remaining, which reduces the mortality scalar. All three clocks map naturally onto the Kelly framework.

### Implementation

```rust
/// Rolling estimator for return statistics.
/// Uses exponentially weighted moments for non-stationary environments.
pub struct ReturnEstimator {
    alpha: f64,
    ewm_mean: f64,
    ewm_var: f64,
    estimation_var: f64,
    n_obs: u64,
    min_obs: u64,
}

impl ReturnEstimator {
    pub fn new(alpha: f64, min_obs: u64) -> Self {
        Self {
            alpha,
            ewm_mean: 0.0,
            ewm_var: 0.0,
            estimation_var: f64::INFINITY,
            n_obs: 0,
            min_obs,
        }
    }

    pub fn update(&mut self, r: f64) {
        self.n_obs += 1;
        if self.n_obs == 1 {
            self.ewm_mean = r;
            self.ewm_var = 0.0;
            return;
        }
        let delta = r - self.ewm_mean;
        self.ewm_mean += self.alpha * delta;
        self.ewm_var = (1.0 - self.alpha)
            * (self.ewm_var + self.alpha * delta * delta);
        let eff_n = (2.0 / self.alpha) - 1.0;
        self.estimation_var = self.ewm_var / eff_n;
    }

    pub fn is_reliable(&self) -> bool { self.n_obs >= self.min_obs }
    pub fn mean(&self) -> f64 { self.ewm_mean }
    pub fn variance(&self) -> f64 { self.ewm_var }
    pub fn estimation_variance(&self) -> f64 { self.estimation_var }
}

/// Ergodicity metrics for a single position or portfolio.
pub struct ErgodicityMetrics {
    /// Time-average growth rate: g = mu - sigma^2 / 2.
    pub growth_rate: f64,
    /// Ergodicity gap: Delta = mu - g = sigma^2 / 2.
    pub ergodicity_gap: f64,
    /// Full Kelly fraction (before adjustments).
    pub kelly_fraction: f64,
    /// Adjusted fraction after mortality and uncertainty scalars.
    pub adjusted_fraction: f64,
    /// Mortality scalar applied.
    pub mortality_scalar: f64,
    /// Uncertainty scalar applied.
    pub uncertainty_scalar: f64,
}

/// Configuration for ergodicity-based position sizing.
pub struct ErgodicitySizer {
    risk_free_rate: f64,
    reference_ticks: f64,
    max_fraction: f64,
    min_growth_rate: f64,
}

impl ErgodicitySizer {
    pub fn new(
        risk_free_rate: f64,
        reference_ticks: f64,
        max_fraction: f64,
        min_growth_rate: f64,
    ) -> Self {
        Self { risk_free_rate, reference_ticks, max_fraction, min_growth_rate }
    }

    pub fn size_position(
        &self,
        estimator: &ReturnEstimator,
        remaining_ticks: f64,
    ) -> ErgodicityMetrics {
        let mu = estimator.mean();
        let sigma_sq = estimator.variance();
        let v = estimator.estimation_variance();

        let growth_rate = mu - sigma_sq / 2.0;
        let ergodicity_gap = sigma_sq / 2.0;

        let kelly_fraction = if sigma_sq > 1e-12 {
            (mu - self.risk_free_rate) / sigma_sq
        } else {
            0.0
        };

        let mortality_scalar =
            (remaining_ticks / self.reference_ticks).min(1.0).max(0.0);
        let uncertainty_scalar = if sigma_sq > 1e-12 {
            1.0 / (1.0 + v / sigma_sq)
        } else {
            0.0
        };

        let adjusted_fraction = (kelly_fraction
            * mortality_scalar
            * uncertainty_scalar)
            .max(0.0)
            .min(self.max_fraction);

        ErgodicityMetrics {
            growth_rate,
            ergodicity_gap,
            kelly_fraction,
            adjusted_fraction,
            mortality_scalar,
            uncertainty_scalar,
        }
    }

    /// Portfolio growth rate given position fractions and covariance matrix.
    pub fn portfolio_growth_rate(
        &self,
        fractions: &[f64],
        means: &[f64],
        covariance: &[Vec<f64>],
    ) -> (f64, bool) {
        let n = fractions.len();
        let mu_term: f64 =
            fractions.iter().zip(means.iter()).map(|(f, m)| f * m).sum();
        let var_term: f64 = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| fractions[i] * fractions[j] * covariance[i][j])
                    .sum::<f64>()
            })
            .sum::<f64>();
        let g = mu_term - 0.5 * var_term;
        (g, g >= self.min_growth_rate)
    }
}
```

### Heartbeat Integration

The ergodicity metrics feed into the 9-step heartbeat:

- **Observe (step 1):** Collect return data for g estimation
- **Analyze (step 3):** Compute g, Delta, and updated Kelly fractions
- **Gate (step 4):** Reject strategies where g < 0 or Delta exceeds threshold
- **Simulate (step 5):** Project portfolio g under proposed trades
- **Validate (step 6):** Verify post-trade g remains above g_min
- **Reflect (step 9):** Update rolling estimates, recalibrate mortality scalars

### Drawdown Limits as Derived Quantities

The maximum tolerable drawdown before g becomes unrecoverable given remaining lifespan T is:

```
max_drawdown = 1 - exp(-g * T)
```

This ties drawdown tolerance directly to the Golem's remaining life and current growth rate. A Golem with high g and long remaining life can tolerate deeper drawdowns. A Golem near death with low g cannot afford any drawdown at all. The numbers come from the math, not from intuition.

### Ensemble-Average Trap Detection

The ergodicity gap Delta directly measures how misleading a strategy's expected return is. When Delta is large relative to mu, the strategy looks profitable across a population of agents but kills any individual agent. DeFi is full of these: high-yield farming strategies with extreme impermanent loss, leveraged positions that compound losses faster than gains. The ergodicity gap is the quantitative diagnostic for "this looks too good to be true."

---

## References

- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012.
- [DANE-2010] Dane, E. "Reconsidering the Trade-off Between Expertise and Flexibility." _AMR_ 35(4), 2010.
- [HINTON-2022] Hinton, G. "The Forward-Forward Algorithm." arXiv:2212.13345, 2022.
- [JONAS-1966] Jonas, H. _The Phenomenon of Life_. Northwestern University Press, 1966.
- [KAHNEMAN-TVERSKY-1979] Kahneman, D. & Tversky, A. "Prospect Theory." _Econometrica_ 47(2), 1979.
- [KELLY-1956] Kelly, J.L. "A New Interpretation of Information Rate." _Bell System Technical Journal_, 35(4), 1956, pp. 917-926.
- [MARCH-1991] March, J.G. "Exploration and Exploitation in Organizational Learning." _Organization Science_ 2(1), 1991.
- [PETERS-2019] Peters, O. "The Ergodicity Problem in Economics." _Nature Physics_, 15(12), 2019, pp. 1216-1221.
- [PETERS-GELLMANN-2016] Peters, O. & Gell-Mann, M. "Evaluating Gambles Using Dynamics." _Chaos_, 26(2), 2016, 023103.
- [SIMONDON-1958] Simondon, G. _L'individuation a la lumiere des notions de forme et d'information_. Millon, 1958/2005.
- [THORP-2006] Thorp, E.O. "The Kelly Criterion in Blackjack, Sports Betting, and the Stock Market." In _Handbook of Asset and Liability Management_, Vol. 1, 2006, pp. 385-428.
- [x402-SPEC] x402 Protocol Specification. https://x402.org, 2025.

---

## Memory Service Costs in Credit Partitions

Memory subsystem costs must be included in credit partition burn rate calculations for accurate time-to-live (TTL) projection:

| Service                  | Cost             | Frequency | Annual     | Partition |
| ------------------------ | ---------------- | --------- | ---------- | --------- |
| Styx Archive snapshots     | ~$0.001/snapshot | 4/day     | ~$1.46/yr  | Data      |
| Styx Lethe (formerly Commons) embeddings  | ~$0.0001/query   | Variable  | Variable   | LLM       |
| Grimoire Curator (ExpeL) | ~$0.001/cycle    | ~30/day   | ~$10.95/yr | LLM       |

**Total memory overhead:** < 5% of daily budget under normal operating conditions.

These costs are small individually but must be accounted for in the burn rate calculation. A Golem that ignores memory costs will over-estimate its remaining lifespan by ~5%, which compounds with other estimation errors.

**Emergency behavior:** When economic clock enters critical range (< 24h), Styx Archive snapshot frequency increases to every 2 hours (insurance priority), while Curator cycles are suspended (save LLM budget for Death Protocol).

> **Cross-ref**: `../20-styx/02-infrastructure.md` (memory service architecture)
