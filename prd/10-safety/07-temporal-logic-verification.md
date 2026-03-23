# Temporal Logic Strategy Verification [SPEC]

> **Crate**: `golem-verify`
>
> **Depends on**: [00-defense.md](00-defense.md) (defense architecture), [02-policy.md](02-policy.md) (PolicyCage), [05-threat-model.md](05-threat-model.md) (threat model), [06-adaptive-risk.md](06-adaptive-risk.md) (adaptive risk), `../01-golem/02-heartbeat.md` (heartbeat pipeline), `../01-golem/18-cortical-state.md` (CorticalState)

> **Reader orientation:** This document specifies temporal logic verification for Golem (mortal autonomous DeFi agent) trading strategies. It belongs to the **Safety** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: while the PolicyCage (on-chain smart contract) gates individual operations, temporal logic monitors verify properties *across time*, catching failures like "held a losing position for 500 ticks without acting" that no single-transaction check can detect. LTL/CTL formulas express these properties mathematically; monitors run at T0 (no LLM, zero cost). Terms like Grimoire, Heartbeat, CorticalState, and BehavioralPhase are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

Bardo's safety system verifies individual transactions. PolicyCage confirms a swap stays within slippage bounds. Revm simulation confirms the swap will succeed. But strategy failures are rarely transactional. A Golem that makes individually safe trades while holding a losing position for 500 ticks without acting, or that never takes profit, or that ignores regime changes for hours, is failing *temporally*. No component of the current system can express, let alone verify, properties like "the portfolio always recovers within N blocks after a drawdown exceeding 10%."

This document specifies temporal logic verification as a **complementary layer** to PolicyCage. PolicyCage gates individual operations. Temporal monitors flag temporal violations. The hierarchy is clear: PolicyCage has veto power, temporal logic has escalation power.

---

## 1. Authority Hierarchy

The relationship between PolicyCage and temporal logic is not peer-to-peer. It is hierarchical:

| System | Authority | Scope | Enforcement |
|---|---|---|---|
| **PolicyCage** | Veto (hard block) | Per-operation | On-chain, immutable |
| **LTL Monitor** | Escalation (soft flag) | Across time (actual trace) | Off-chain, runtime |
| **CTL Verifier** | Advisory (pre-decision) | Branching futures | Off-chain, pre-execution |

PolicyCage can block a trade that passes LTL checks (the trade is individually unsafe even if the temporal pattern is fine). LTL can flag a pattern that passes PolicyCage checks (each individual operation is safe but the sequence violates a temporal property). LTL flags do NOT block execution. They escalate to the next cognitive tier and emit warnings. Only PolicyCage blocks.

```rust
/// Safety verdict types, ordered by authority.
/// Each verification layer issues one of these per check.
pub enum SafetyVerdict {
    /// Hard block: operation cannot proceed. Only PolicyCage issues these.
    Block { reason: String },
    /// Soft flag: operation can proceed but a violation is logged and escalated.
    /// LTL monitors and Witness DAG issue these.
    Flag { severity: u8, property: String },
    /// Pass: no issues detected.
    Pass,
}
```

---

## 2. The Verification Gap

Three failure modes from real Golem post-mortems illustrate the gap:

**The patient loser.** A Golem entered a leveraged ETH-USDC position at $3,200. ETH dropped to $2,800. Each tick, the Golem decided to hold. Every individual hold decision was locally rational -- the drawdown hadn't hit the stop-loss threshold for any single tick. After 437 ticks (~14 hours at theta=120s), cumulative loss consumed 40% of the vault. Every individual decision passed per-transaction verification. The temporal property "never hold a declining position for more than N ticks" was invisible to the safety system.

**The profit-allergic accumulator.** A Golem identified profitable LP positions. Returns accumulated. The Golem never exited. When a liquidity event hit, unrealized gains and principal were lost. Each entry was individually safe. The temporal property "eventually take profit on positions held longer than T ticks" was never checked.

**The regime-deaf strategist.** Market volatility tripled in 30 minutes. The Golem continued executing its low-volatility strategy for 47 ticks before adjusting. Each tick's actions were individually valid under the old regime. The temporal property "respond to regime changes within N ticks" was unenforceable.

The current architecture has a blind spot shaped like time:

- **PolicyCage** enforces per-operation constraints: max slippage, max position size, approved protocol list. These are state predicates, not temporal properties.
- **Capability tokens** gate access to operations. They control what the Golem *can* do, not what it *should* do over time.
- **Taint tracking** follows asset provenance. It tracks origin, not timing.
- **Revm simulation** checks the next transaction. It cannot simulate a 100-tick future.

---

## 3. Mathematical Foundation

### 3.1 State Traces from the Heartbeat

Each heartbeat tick produces a DecisionCycleRecord. For temporal verification, each record is abstracted to a state vector:

```
s_t = (portfolio_value, positions, drawdown, regime, phase, exposure_by_asset,
       position_ages, actions_taken, cortical_signals)
```

A trace is a finite sequence sigma = s_0, s_1, ..., s_n. The trace grows by one state per tick. At theta=120s, a 24-hour period produces 720 states.

This trace is a finite prefix of an omega-word. LTL formulas are defined over infinite words, but finite-trace semantics are well-established (Bauer et al., 2011; De Giacomo & Vardi, 2013). A formula evaluates to true, false, or inconclusive on a finite prefix. We use three-valued semantics throughout.

### 3.2 Linear Temporal Logic (LTL)

LTL extends propositional logic with temporal operators over a single infinite path.

**Syntax.** Given a set AP of atomic propositions (state predicates like `drawdown > 0.1` or `position_age > T`):

```
phi ::= p | !phi | phi && psi | phi || psi | X phi | phi U psi | F phi | G phi | phi R psi
```

where p is in AP, and:

- **X phi** (next): phi holds at the next state
- **F phi** (eventually): phi holds at some future state
- **G phi** (globally): phi holds at all future states
- **phi U psi** (until): phi holds at every state until psi holds (and psi does hold)
- **phi R psi** (release): dual of U

**Bounded operators.** For DeFi, we need bounded variants:

- **F_{<=N} phi**: phi holds within the next N states
- **G_{<=N} phi**: phi holds for the next N states

These are syntactic sugar: F_{<=N} phi is phi || X phi || XX phi || ... || X^N phi. The bounded form matters for complexity: checking a bounded formula avoids the exponential blowup of the general case.

**Semantics.**

```
sigma, i |= p           iff  p holds in state s_i
sigma, i |= !phi        iff  sigma, i |/= phi
sigma, i |= phi && psi  iff  sigma, i |= phi and sigma, i |= psi
sigma, i |= X phi       iff  sigma, i+1 |= phi
sigma, i |= phi U psi   iff  exists j >= i: sigma, j |= psi
                              and for all k, i <= k < j: sigma, k |= phi
sigma, i |= F phi       iff  exists j >= i: sigma, j |= phi
sigma, i |= G phi       iff  for all j >= i: sigma, j |= phi
```

### 3.3 Computation Tree Logic (CTL)

CTL extends propositional logic with temporal operators over a branching tree of computation paths. Where LTL reasons about one path (the actual execution), CTL reasons about all possible paths from a given state.

**Syntax:**

```
phi ::= p | !phi | phi && psi | phi || psi
      | AX phi | EX phi | AF phi | EF phi | AG phi | EG phi
      | A[phi U psi] | E[phi U psi]
```

Path quantifiers:

- **A** (for all paths): the property holds on every path from the current state
- **E** (exists a path): the property holds on at least one path

Combined with temporal operators:

- **AG phi**: on all paths, phi holds at every state (strongest safety)
- **AF phi**: on all paths, phi holds at some future state (strongest liveness)
- **EF phi**: there exists a path where phi eventually holds (reachability)
- **EG phi**: there exists a path where phi always holds (persistence possibility)

CTL semantics are evaluated over a Kripke structure M = (S, R, L). CTL model checking is O(|S| * |R| * |phi|), polynomial in both model size and formula size (Clarke, Grumberg & Peled, 1999).

### 3.4 Why Both Logics

LTL and CTL have incomparable expressive power (Emerson, 1990). Neither subsumes the other. DeFi strategy verification needs both:

**LTL for runtime monitoring.** The Golem's actual execution trace is a single path. LTL checks whether this path satisfies temporal specifications. "Has this Golem held a losing position too long?" is an LTL question.

**CTL for pre-execution verification.** Before a major decision (T2 tick), the Golem builds a finite model of possible futures. "Does there exist any sequence of market moves under which this strategy violates a safety property?" is a CTL question: check EF(violation) on the branching model.

The practical split: LTL runs every tick (cheap, monitors actual trace). CTL runs before major decisions (expensive, checks the space of possible futures).

### 3.5 Automata-Theoretic Model Checking

**LTL via Buchi automata.** The standard approach converts an LTL formula phi to a Buchi automaton A_phi that accepts exactly the traces satisfying phi (Vardi & Wolper, 1986). For runtime monitoring on finite prefixes, we use a simplified construction: convert the LTL formula to an NFA over finite words (De Giacomo & Vardi, 2013). The NFA tracks satisfaction status incrementally -- each new state triggers one automaton transition. Three-valued output (true/false/inconclusive) comes from the automaton's state after processing the prefix.

**Complexity.** LTL model checking on a trace of length n with a formula of size m is O(n * 2^m). The exponential is in formula size, not trace length. DeFi temporal specifications are typically small (5-15 operators). A formula with 10 operators gives 2^10 = 1024 automaton states. Checking against a 1000-tick trace is ~10^6 operations, about 1ms.

For bounded formulas (F_{<=N} with small N), the automaton construction produces much smaller automata. G(p -> F_{<=100} q) with 100-tick bound needs an automaton that tracks at most 100 open obligations. Linear in the bound, not exponential.

---

## 4. Forty DeFi Temporal Patterns

Organized into four categories following Dwyer, Avrunin & Corbett (1999).

### 4.1 Safety Properties (G -- always)

Safety properties say that something bad never happens. Violation occurs at a specific state and is irrevocable.

1. **Max single-asset exposure**: G(exposure(asset) <= threshold) for all assets
2. **Max total leverage**: G(leverage_ratio <= max_leverage)
3. **Min collateral ratio**: G(collateral_ratio >= min_ratio) for all borrow positions
4. **Max gas budget per tick**: G(gas_spent_this_tick <= gas_budget)
5. **No interaction with blacklisted protocols**: G(!interacts_with(blacklisted))
6. **Max portfolio concentration**: G(herfindahl_index <= max_concentration)
7. **Min liquidity buffer**: G(liquid_assets >= min_buffer)
8. **Max drawdown**: G(drawdown <= max_drawdown)
9. **No stale price data**: G(price_staleness(asset) <= max_staleness) for all observed assets
10. **Approval hygiene**: G(token_approval(protocol) <= position_size(protocol) * safety_factor)

### 4.2 Liveness Properties (F -- eventually)

Liveness properties say that something good eventually happens. Violation is the absence of progress over time.

11. **Drawdown recovery**: G(drawdown > recovery_threshold -> F_{<=N} (drawdown < recovery_target))
12. **Stale position closure**: G(position_age > max_age -> F_{<=M} (position_closed || position_refreshed))
13. **Profit taking**: G(unrealized_pnl > take_profit_target -> F_{<=K} (realized_some_profit))
14. **Loss cutting**: G(unrealized_loss > stop_loss_threshold -> F_{<=L} (position_reduced || position_closed))
15. **Rebalancing**: G(deviation_from_target > rebalance_threshold -> F_{<=P} rebalanced)
16. **Debt repayment**: G(borrow_position_age > max_borrow_age -> F_{<=Q} (debt_reduced || debt_repaid))
17. **Yield harvesting**: G(unclaimed_rewards > harvest_threshold -> F_{<=R} rewards_claimed)
18. **Impermanent loss response**: G(il_estimate > il_threshold -> F_{<=S} (lp_position_adjusted))
19. **Gas optimization**: G(avg_gas_cost > gas_target -> F_{<=T} gas_strategy_adjusted)
20. **Capital deployment**: G(idle_capital > idle_threshold && idle_duration > patience -> F_{<=U} capital_deployed)

### 4.3 Fairness Properties (GF -- infinitely often)

On a finite trace, approximated as "happens at least once per window."

21. **Attention rotation**: G(F_{<=W} attended_to(asset)) for all tracked assets
22. **Strategy review**: G(F_{<=V} strategy_review_performed)
23. **Risk assessment**: G(F_{<=X} full_risk_assessment_performed)
24. **Portfolio snapshot**: G(F_{<=Y} portfolio_state_recorded)
25. **Model recalibration**: G(F_{<=Z} prediction_model_updated)
26. **Liquidity check**: G(F_{<=A1} exit_liquidity_verified) for all positions
27. **Correlation update**: G(F_{<=B1} cross_asset_correlations_updated)
28. **Fee accounting**: G(F_{<=C1} cumulative_fees_assessed)
29. **Mortality check**: G(F_{<=D1} vitality_self_assessed)
30. **Clade sync**: G(F_{<=E1} clade_knowledge_synced) (if in a Clade)

### 4.4 Responsiveness Properties (bounded response)

Combine a trigger with a bounded response. The most common pattern in DeFi strategy specifications.

31. **Regime response**: G(regime_change_detected -> F_{<=N1} strategy_adjusted)
32. **Liquidation prevention**: G(health_factor < warning_threshold -> F_{<=N2} (collateral_added || debt_reduced))
33. **Depeg response**: G(stablecoin_depeg > depeg_threshold -> F_{<=N3} exposure_reduced)
34. **Oracle failure response**: G(oracle_stale > oracle_threshold -> F_{<=N4} positions_hedged_or_closed)
35. **Volatility spike response**: G(realized_vol > vol_threshold -> F_{<=N5} risk_reduced)
36. **MEV detection response**: G(mev_detected -> F_{<=N6} execution_strategy_adjusted)
37. **Fee spike response**: G(gas_price > gas_threshold -> F_{<=N7} (execution_paused || batch_mode))
38. **Governance response**: G(governance_proposal_affects_position -> F_{<=N8} position_reviewed)
39. **Correlation breakdown response**: G(correlation_regime_shift -> F_{<=N9} portfolio_reviewed)
40. **Mortality urgency response**: G(vitality_drop > drop_threshold -> F_{<=N10} conservation_mode_entered)

---

## 5. Architecture

### 5.1 System Placement

The temporal verification system lives in `golem-verify` at Layer 4 (Action) of the extension DAG. Dependencies:

- `golem-heartbeat`: DecisionCycleRecord type and tick lifecycle hooks
- `golem-safety`: PolicyCage integration (temporal violations feed into safety signals)
- `golem-cortex`: CorticalState signaling

### 5.2 Two Verification Modes

**Runtime monitor (LTL).** Runs at the end of Step 9 (REFLECT) every tick. Maintains a sliding window of abstracted states and checks each LTL specification against the window. Window size is configurable per specification -- a responsiveness property with a 5-tick bound needs only 5 states, while a fairness property with a 500-tick window needs 500.

The monitor is incremental. Each tick: push new state, pop oldest if window full, advance each specification's automaton by one transition. Per-tick cost: O(|specs| * |automaton_size|), where automaton sizes are small for typical DeFi specifications.

**Pre-execution verifier (CTL).** Runs before major decisions (T2 ticks, where the Golem is about to commit significant capital). Builds a finite Kripke structure representing possible futures:

- Current state is the root
- Each successor state is generated by applying the proposed action under different market scenarios (bull, bear, flat, volatility spike, liquidity drain)
- Branching depth bounded (typically 5-10 ticks)
- O(scenarios^depth) states: 5 scenarios at depth 5 gives 3,125 states

CTL model checking on this structure verifies that safety properties hold on all paths (AG) and that no path leads to an unrecoverable state (not EF(unrecoverable)).

### 5.3 Specification Format

Temporal specifications live alongside PLAYBOOK.md in a TOML file:

```toml
[[safety]]
name = "max_single_asset_exposure"
formula = "G(single_asset_exposure <= 0.5)"
severity = "hard"    # violation = abort current action
description = "No single asset should ever exceed 50% of portfolio value"

[[liveness]]
name = "drawdown_recovery"
formula = "G(drawdown > 0.1 -> F[<=100](drawdown < 0.05))"
severity = "soft"    # violation = alert + raise urgency
description = "After a 10% drawdown, recovery to under 5% within 100 ticks"
window = 200         # monitor window size

[[responsiveness]]
name = "regime_response"
formula = "G(regime_change -> F[<=5](strategy_review))"
severity = "soft"
description = "Review strategy within 5 ticks of any regime change"

[[fairness]]
name = "attention_rotation"
formula = "G(F[<=50](attended_to(asset)))"
severity = "info"    # violation = log only
description = "Every tracked asset gets attention at least once per 50 ticks"
```

Severity determines violation behavior:

- `hard`: abort the current action, equivalent to a PolicyCage rejection
- `soft`: raise CorticalState urgency, log the violation, allow the Golem to continue
- `info`: log only, for monitoring and post-mortem analysis

### 5.4 CorticalState Integration

Two new signals in CorticalState (these live in `TaCorticalExtension` per `18-cortical-state.md`):

```rust
/// Count of currently active temporal violations.
pub active_violations: AtomicU16,

/// Severity of the worst active violation (0-255).
pub worst_violation: AtomicU8,
```

When `active_violations > 0`, the Golem's decision-making shifts toward addressing pending obligations. A separate liveness urgency signal, computed as max(elapsed / deadline) across all active liveness specs, feeds into the attention system. When urgency exceeds a threshold (e.g., 0.8, meaning a liveness property is 80% of the way to its deadline without being satisfied), the Golem prioritizes resolving the pending obligation.

---

## 6. Implementation

### 6.1 Core Data Structures

```rust
use std::collections::VecDeque;

/// Abstracted state for temporal verification.
/// Extracted from DecisionCycleRecord each tick.
#[derive(Clone, Debug)]
pub struct TemporalState {
    pub tick: u64,
    pub portfolio_value: f64,
    pub drawdown: f64,
    pub positions: Vec<PositionState>,
    pub regime: MarketRegime,
    pub actions_taken: Vec<ActionKind>,
    pub cortical: CorticalSnapshot,
}

#[derive(Clone, Debug)]
pub struct PositionState {
    pub asset: AssetId,
    pub size: f64,
    pub entry_tick: u64,
    pub unrealized_pnl: f64,
    pub exposure_fraction: f64,
}

/// A parsed and compiled temporal specification.
pub struct CompiledSpec {
    pub name: String,
    pub severity: Severity,
    pub automaton: LtlAutomaton,
    pub window_size: usize,
}

/// The runtime monitor. Holds the state window and compiled specs.
pub struct TemporalMonitor {
    window: VecDeque<TemporalState>,
    specs: Vec<CompiledSpec>,
    max_window: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerificationResult {
    Satisfied,
    Violated { at_tick: u64 },
    Inconclusive,
}

pub enum Severity {
    Hard,
    Soft,
    Info,
}
```

### 6.2 LTL Formula Representation

```rust
/// Atomic propositions are predicates over TemporalState.
pub type AtomicPredicate = Arc<dyn Fn(&TemporalState) -> bool + Send + Sync>;

/// LTL formula AST.
pub enum LtlFormula {
    Atom(AtomicPredicate),
    Not(Box<LtlFormula>),
    And(Box<LtlFormula>, Box<LtlFormula>),
    Or(Box<LtlFormula>, Box<LtlFormula>),
    Next(Box<LtlFormula>),
    Until(Box<LtlFormula>, Box<LtlFormula>),
    Eventually(Box<LtlFormula>),
    Always(Box<LtlFormula>),
    BoundedEventually(Box<LtlFormula>, u64),  // F_{<=N}
    BoundedAlways(Box<LtlFormula>, u64),       // G_{<=N}
}

/// Compiled automaton for incremental evaluation.
/// States are bitvectors over subformula indices.
pub struct LtlAutomaton {
    /// Number of subformulas (automaton state = bitvec of this width).
    width: usize,
    /// Transition function: (current_state, input_props) -> next_state.
    transitions: Vec<(u64, u64, u64)>,
    /// Accepting states.
    accepting: Vec<u64>,
    /// Current automaton state.
    current: u64,
}
```

### 6.3 Buchi Automata Monitor

The LTL monitor converts formulas to Buchi automata for O(1) per-tick transitions:

```rust
/// LTL runtime monitor compiled to a DFA for O(1) per-tick transitions.
pub struct BuchiMonitor {
    /// The temporal property being monitored.
    pub property: LtlFormula,
    /// Current DFA state.
    pub state: usize,
    /// DFA transition table. Rows = states, columns = state predicates.
    pub transitions: Vec<Vec<usize>>,
    /// Violating states (property violated if reached).
    pub violating_states: Vec<usize>,
}

impl BuchiMonitor {
    /// Advance the monitor by one tick. O(1).
    pub fn step(&mut self, predicates: &[bool]) -> SafetyVerdict {
        let col = encode_predicates(predicates);
        self.state = self.transitions[self.state][col];

        if self.violating_states.contains(&self.state) {
            SafetyVerdict::Flag {
                severity: self.property.severity(),
                property: self.property.name().to_string(),
            }
        } else {
            SafetyVerdict::Pass
        }
    }
}

/// Encode a vector of boolean predicates into a column index
/// for the transition table lookup.
fn encode_predicates(predicates: &[bool]) -> usize {
    predicates
        .iter()
        .enumerate()
        .fold(0usize, |acc, (i, &p)| acc | ((p as usize) << i))
}
```

### 6.4 Bounded Response Tracker

For the common case of bounded response properties (G(trigger -> F_{<=N} response)), a specialized evaluator avoids full automaton construction:

```rust
/// Tracks open obligations for bounded response properties.
/// Each obligation records: "trigger fired at tick T, response must occur by T+N."
pub struct BoundedResponseTracker {
    deadline_bound: u64,
    trigger: AtomicPredicate,
    response: AtomicPredicate,
    /// Open obligations: (trigger_tick, deadline_tick)
    obligations: VecDeque<(u64, u64)>,
    current_tick: u64,
}

impl BoundedResponseTracker {
    pub fn step(&mut self, state: &TemporalState) -> VerificationResult {
        let tick = state.tick;
        self.current_tick = tick;

        // Check if trigger fires, creating a new obligation
        if (self.trigger)(state) {
            self.obligations.push_back((tick, tick + self.deadline_bound));
        }

        // Check if response satisfies any open obligations
        if (self.response)(state) {
            self.obligations.clear(); // all pending obligations satisfied
            return VerificationResult::Satisfied;
        }

        // Check for deadline violations
        if let Some(&(trigger_tick, deadline)) = self.obligations.front() {
            if tick > deadline {
                self.obligations.pop_front();
                return VerificationResult::Violated { at_tick: trigger_tick };
            }
        }

        if self.obligations.is_empty() {
            VerificationResult::Satisfied
        } else {
            VerificationResult::Inconclusive
        }
    }

    /// How close is the oldest obligation to its deadline?
    pub fn pending_deadline(&self) -> Option<u64> {
        self.obligations.front().map(|(_, d)| *d)
    }

    pub fn pending_elapsed(&self) -> u64 {
        self.obligations
            .front()
            .map(|(t, _)| self.current_tick.saturating_sub(*t))
            .unwrap_or(0)
    }
}
```

This specialized tracker is O(1) per tick per specification (amortized). It handles the majority of DeFi temporal patterns without the general Buchi automaton machinery.

### 6.5 Runtime Monitor

```rust
impl TemporalMonitor {
    pub fn new(specs: Vec<CompiledSpec>) -> Self {
        let max_window = specs.iter().map(|s| s.window_size).max().unwrap_or(1000);
        TemporalMonitor {
            window: VecDeque::with_capacity(max_window),
            specs,
            max_window,
        }
    }

    /// Called at the end of Step 9 (REFLECT) each tick.
    /// Returns a list of violations detected this tick.
    pub fn on_tick(&mut self, state: TemporalState) -> Vec<Violation> {
        self.window.push_back(state);
        if self.window.len() > self.max_window {
            self.window.pop_front();
        }

        let mut violations = Vec::new();

        for spec in &mut self.specs {
            let result = spec.automaton.step(&self.window);
            if let VerificationResult::Violated { at_tick } = result {
                violations.push(Violation {
                    spec_name: spec.name.clone(),
                    severity: spec.severity,
                    at_tick,
                    window_snapshot: self.recent_window(spec.window_size),
                });
            }
        }

        violations
    }

    /// Compute liveness urgency: the maximum ratio of elapsed time
    /// to deadline across all liveness/responsiveness specs.
    pub fn liveness_urgency(&self) -> f32 {
        let mut max_urgency: f32 = 0.0;

        for spec in &self.specs {
            if let Some(deadline) = spec.automaton.pending_deadline() {
                let elapsed = spec.automaton.pending_elapsed();
                let urgency = elapsed as f32 / deadline as f32;
                max_urgency = max_urgency.max(urgency);
            }
        }

        max_urgency
    }

    fn recent_window(&self, n: usize) -> Vec<TemporalState> {
        self.window.iter().rev().take(n).cloned().collect()
    }
}

pub struct Violation {
    pub spec_name: String,
    pub severity: Severity,
    pub at_tick: u64,
    pub window_snapshot: Vec<TemporalState>,
}
```

### 6.6 Performance Characteristics

The runtime monitor's per-tick cost:

| Component | Cost | Notes |
|---|---|---|
| State abstraction | ~5us | Field copies from DecisionCycleRecord, no allocation |
| Window management | ~100ns | VecDeque push/pop |
| BoundedResponseTracker (per spec) | ~200ns | Predicate evaluation + deque operations |
| General LtlAutomaton (per spec) | ~1-5us | Bitvector transition lookup |
| **Total for 20 specs** | **~10-50us** | |

At theta=120s, spending 50us on temporal verification is negligible. Even at theta=1s (minimum realistic tick rate), 50us is 0.005% of the tick budget.

CTL pre-execution verification is heavier:

| Component | Cost | Notes |
|---|---|---|
| Kripke structure construction | ~16ms | 5 scenarios x depth 5 = 3,125 states at ~5us each |
| CTL model checking (10 specs) | ~5ms | O(|S| * |R| * |phi|) |
| **Total per T2 decision** | **~21ms** | Acceptable for decisions committing significant capital |

---

## 7. Category-Theoretic Composition Verification

Strategy composition safety is a separate but related verification concern. The category-theoretic framework from innovation A3 provides a complementary check: where temporal logic verifies behavior over time, categorical composition verifies that sequential operations preserve safety invariants across intermediate states.

### 7.1 Morphism Tags on Existing Tools

Each tool in the Golem's tool system gains an optional `MorphismTag` that describes its pre- and post-conditions. This does not change tool behavior. It enables algebraic safety checking at composition time.

```rust
/// Morphism tag on existing tools.
/// Does not change tool behavior. Enables algebraic safety checking.
pub struct MorphismTag {
    /// Pre-conditions: what state must hold before this tool runs.
    pub preconditions: Vec<StateInvariant>,
    /// Post-conditions: what state this tool guarantees after running.
    pub postconditions: Vec<StateInvariant>,
    /// Composition safety: which tools can follow this one.
    pub composable_with: Vec<ToolId>,
    /// Category: swap, lp, lend, borrow, bridge, stake.
    pub category: OperationCategory,
}

/// State invariant expressed as a predicate on CorticalState + portfolio state.
pub enum StateInvariant {
    MinBalance(f64),
    MaxExposure(f64),
    TokenApproved(Address),
    PositionExists(PositionId),
    NoOpenOrders,
    SlippageBound(f64),
}
```

### 7.2 Functorial Preservation of Safety

The core theorem from category-theoretic composition: if individual morphisms are safe and the composition rules preserve safety, any well-formed composite strategy is safe.

**Proof by structural induction on Strategy<A>:**

**Base case.** `Pure(a)` performs no operations. Vacuously safe.

**Inductive case.** `Bind(op, k)`. Given:
1. `Safe(op, sigma, sigma')` -- verified by PolicyCage and capability tokens at interpretation time.
2. For all sigma' in the codomain of `op`, `k(sigma')` is safe (inductive hypothesis).

Then `Bind(op, k)` is safe: first step is safe by (1), continuation is safe by (2).

**For sequential composition (g . f):** Given Safe(f, sigma, sigma') and Safe(g, sigma', sigma''), verify that the intermediate state sigma' produced by f satisfies the preconditions of g. This is what the current system misses. The categorical framework makes it explicit: g . f is well-typed only if the codomain of f matches the domain of g, including safety-relevant quantitative conditions.

**For parallel composition (f tensor g):** Given Safe(f, sigma_1, sigma'_1) and Safe(g, sigma_2, sigma'_2) where sigma_1 and sigma_2 are disjoint, then Safe(f tensor g, (sigma_1, sigma_2), (sigma'_1, sigma'_2)). Disjointness means f and g cannot interfere with each other's safety properties.

### 7.3 Composition Checker

Given a sequence of tools, verify safety of the composite before execution:

```rust
/// Composition checker: given a sequence of tools, verify safety.
pub fn verify_composition(tools: &[(&Tool, &MorphismTag)]) -> Result<(), SafetyViolation> {
    for window in tools.windows(2) {
        let (_, prev_tag) = window[0];
        let (next_tool, next_tag) = window[1];

        // Check: prev postconditions satisfy next preconditions
        for precond in &next_tag.preconditions {
            if !prev_tag.postconditions.iter().any(|post| post.satisfies(precond)) {
                return Err(SafetyViolation::UnsatisfiedPrecondition {
                    tool: next_tool.id(),
                    missing: precond.clone(),
                });
            }
        }

        // Check: tools are composable
        if !prev_tag.composable_with.contains(&next_tool.id()) {
            return Err(SafetyViolation::IncomposableTools {
                first: window[0].0.id(),
                second: next_tool.id(),
            });
        }
    }
    Ok(())
}
```

This runs during the heartbeat's SIMULATE step. If `verify_composition` fails, the strategy is rejected before any on-chain execution. The per-tool safety checks and the compositional check form two verification axes: per-operation (PolicyCage) and per-composition (morphism tags).

### 7.4 V1 vs V2 Scope

**V1 (current):** Tag existing tools as morphisms. Verify composition safety algebraically via `MorphismTag` pre/post-condition matching. No changes to tool implementations. Tags are additive metadata.

**V2 (deferred to Tier 4):** Full free monad interpreter stack. Tools become morphisms in a monoidal category. Strategies become programs in the free monad over this category. Three interpreters (simulation, validation, execution) share a single AST. Left Kan extensions find alternative routes when desired operations are unavailable. This is a substantial refactor that should wait until the tool system is stable and v1 morphism tags have validated the approach.

### 7.5 Interaction with Temporal Logic

Temporal logic and category-theoretic composition address orthogonal verification dimensions:

| Dimension | Temporal Logic | Category Theory |
|---|---|---|
| **What it checks** | Behavior over time | Composition of operations |
| **Failure mode** | "You held this position too long" | "Operation B's preconditions aren't met after A" |
| **When it runs** | Every tick (LTL) or pre-decision (CTL) | Pre-execution, at SIMULATE step |
| **Authority** | Escalation (soft flags, urgency) | Advisory (composition reject) |

Both feed into the safety pipeline. A strategy can fail temporal checks (the sequence over time is bad) while passing compositional checks (each pair of adjacent operations is well-typed), or vice versa. The two verification layers are complementary.

---

## 8. Evaluation and Falsifiability

### 8.1 Null Hypothesis

Temporal verification catches no strategy failures that per-transaction verification misses. The temporal layer adds computational cost without improving outcomes.

### 8.2 Experimental Protocol

**Experiment 1: Historical trace analysis.** Collect state traces from historical Golem runs (both successful and failed). Define a standard library of 20 temporal specifications. Run every trace against every specification.

Predictions:
- Failed Golems exhibit more temporal violations than successful Golems (Mann-Whitney U test, p < 0.01)
- >30% of failures have their first temporal violation at least 50 ticks before failure became visible in per-transaction metrics
- Specific failure modes map to specific temporal property classes (patient loser -> liveness, profit-allergic -> liveness, regime-deaf -> responsiveness)

**Experiment 2: Live A/B test.** Two populations with identical strategies. Group A has temporal monitoring with soft enforcement (urgency signals but no hard aborts). Group B has none.

Predictions:
- Group A survives longer (Kaplan-Meier survival analysis, log-rank test p < 0.05)
- Group A has smaller maximum drawdowns (two-sample t-test, p < 0.05)
- Group A's temporal violations decrease over time as urgency signals modify behavior

**Experiment 3: Specification sensitivity.** Vary deadline bounds in responsiveness specs (N = 5, 10, 20, 50, 100 ticks). Measure false positive / false negative tradeoff. Plot ROC curve for each specification.

Prediction: there exists a sweet spot for each specification where false positive rate < 5% and detection rate > 80%.

### 8.3 Falsification Criteria

The temporal verification framework fails if:
- Historical trace analysis shows no statistical difference in temporal violations between successful and failed Golems
- The live A/B test shows no survival or return benefit from temporal monitoring
- The ROC curves show no parameter settings where detection rate exceeds false positive rate by a meaningful margin

If all three experiments fail, the framework adds cost without value and should be abandoned.

---

## Philosophical Grounding

> Source: `innovations/12-temporal-logic-strategy-verification.md`, Section 8

### Programs Are Temporal Objects

Pnueli's 1977 paper opens with the observation that programs are not static descriptions but dynamic, evolving processes. Their correctness cannot be captured by input-output specifications alone. A sorting algorithm that produces the right answer after infinite time is incorrect. A mutual exclusion protocol that prevents starvation satisfies a liveness property. A type system that prevents null dereferences satisfies a safety property.

DeFi strategies inherit this temporal character. A strategy is not a function from market state to action. It is a process that unfolds over time, with history-dependent decisions, bounded-time obligations, and persistent commitments. Verifying a single action in isolation is like verifying a single instruction of a concurrent program. You miss the bugs that live between the lines.

### Safety and Liveness

The distinction between safety and liveness goes back to Lamport (1977) and Alpern & Schneider (1985). Safety says "nothing bad happens." Liveness says "something good eventually happens." Every temporal property decomposes into a safety component and a liveness component.

This decomposition maps to something older than computer science. Hippocratic medicine distinguishes *primum non nocere* (first, do no harm) from the obligation to heal. A doctor who never harms anyone by never treating anyone satisfies safety but violates liveness. A DeFi strategy that never loses money by never trading satisfies every safety property but violates every liveness property. A mortal agent needs both: avoid catastrophe and make progress before time runs out.

Mortality makes liveness properties urgent in a way they are not for immortal systems. A server can wait forever for a lock to become available. A Golem cannot wait forever for a drawdown to recover. The bounded liveness properties (F_{<=N}) encode this mortality: you have N ticks to fix this, or the temporal contract is broken.

### Composability and Temporal Safety

DeFi's composability is both its strength and its temporal hazard. A sequence of individually safe operations can be temporally unsafe. Borrowing is safe. Not repaying for 1,000 ticks is unsafe. Each tick's decision to hold the debt is individually defensible -- the interest rate is low, the collateral is sufficient, the market is stable. The temporal property G(borrow_position_age > max_age -> F_{<=Q} debt_reduced) catches the slow accumulation of risk that no single-tick check can see.

This is the temporal analog of the category-theoretic composition problem from Section 7. There, the issue was that composing individually safe morphisms can produce an unsafe composite because intermediate states are unchecked. Here, the issue is that a sequence of individually safe states can constitute an unsafe trajectory because nobody checks the sequence as a whole. The problems are structural cousins; temporal logic addresses the time axis as category theory addresses the composition axis.

---

## Cross-References

- [00-defense.md](00-defense.md) -- The main defense-in-depth architecture doc: six defense layers, `Capability<T>` compile-time tokens, taint tracking, and the DeFi Constitution. Temporal logic is a complementary layer to the defenses specified here.
- [02-policy.md](02-policy.md) -- PolicyCage on-chain smart contract: has veto authority over temporal logic (PolicyCage blocks individual transactions; temporal logic flags patterns across time).
- [05-threat-model.md](05-threat-model.md) -- Full adversary taxonomy and attack trees mapping each attack path to mitigating defense layers. Temporal logic addresses temporal attack patterns not caught by per-transaction checks.
- [06-adaptive-risk.md](06-adaptive-risk.md) -- Five-layer adaptive risk management: temporal violations feed into Layer 4 (Health Score observation) and can trigger Layer 1 (Hard Shield) escalations.
- [08-witness-dag.md](08-witness-dag.md) -- Cryptographic cognitive trace DAG: temporal verdicts are recorded as witness nodes, enriching the audit trail with temporal property satisfaction/violation evidence.
- [../01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- The 9-step Heartbeat pipeline. The temporal monitor runs at Step 9 (reflect), evaluating accumulated trace against active LTL/CTL properties.
- [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) -- The 28-extension architecture and tool system. Morphism tags (used in category-theoretic composition) are additive metadata on tools.
- [../01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- CorticalState (32-signal atomic shared perception surface): the `active_violations` and `worst_violation` signals are written by the temporal monitor and read by all other extensions.

---

## References

- [ALPERN-SCHNEIDER-1985] Alpern, B. & Schneider, F.B. "Defining Liveness." *Information Processing Letters*, 21(4), 181-185, 1985. Formalizes the distinction between safety properties ("nothing bad happens") and liveness properties ("something good eventually happens"). Foundation for classifying DeFi temporal patterns.
- [BAIER-KATOEN-2008] Baier, C. & Katoen, J.-P. *Principles of Model Checking*. MIT Press, 2008. The standard textbook on model checking. Provides the theoretical grounding for LTL/CTL semantics and Buchi automaton construction used throughout this document.
- [BAUER-2011] Bauer, A., Leucker, M., & Schallhart, C. "Runtime Verification for LTL and TLTL." *ACM TOSEM*, 20(4), Article 14, 2011. Defines three-valued LTL semantics (true/false/inconclusive) for runtime monitoring of incomplete traces. Directly implemented in the LTL monitor.
- [CLARKE-1999] Clarke, E.M., Grumberg, O., & Peled, D.A. *Model Checking*. MIT Press, 1999. Foundational model checking textbook covering CTL and state-space exploration. Provides the basis for CTL pre-decision verification.
- [DE-GIACOMO-VARDI-2013] De Giacomo, G. & Vardi, M.Y. "Linear Temporal Logic and Linear Dynamic Logic on Finite Traces." *IJCAI 2013*, 854-860. Adapts LTL to finite traces (LTLf), which is needed because Golem traces have bounded length due to mortality.
- [DWYER-1999] Dwyer, M.B., Avrunin, G.S., & Corbett, J.C. "Patterns in Property Specifications for Finite-State Verification." *ICSE 1999*, 411-420. Catalogs common property specification patterns. The 40 DeFi temporal patterns in this document follow this classification approach.
- [EMERSON-CLARKE-1982] Emerson, E.A. & Clarke, E.M. "Using Branching Time Temporal Logic to Synthesize Synchronization Skeletons." *Science of Computer Programming*, 2(3), 241-266, 1982. Introduces CTL (Computation Tree Logic) for reasoning about branching futures. Foundation for the CTL pre-decision verifier.
- [EMERSON-1990] Emerson, E.A. "Temporal and Modal Logic." In *Handbook of Theoretical Computer Science*, Vol. B, 995-1072. Elsevier, 1990. Comprehensive survey of temporal and modal logics. Reference for the expressiveness hierarchy (LTL, CTL, CTL*).
- [LAMPORT-1977] Lamport, L. "Proving the Correctness of Multiprocess Programs." *IEEE TSE*, SE-3(2), 125-143, 1977. Introduces temporal reasoning for concurrent program verification. The conceptual ancestor of all runtime temporal monitoring.
- [MANNA-PNUELI-1992] Manna, Z. & Pnueli, A. *The Temporal Logic of Reactive and Concurrent Systems: Specification*. Springer, 1992. Comprehensive treatment of temporal specification for reactive systems. Provides the theoretical framework for specifying Golem behavior as a reactive system.
- [PNUELI-1977] Pnueli, A. "The Temporal Logic of Programs." *18th IEEE FOCS*, 46-57, 1977. The seminal paper introducing temporal logic for program verification. Foundation for all temporal verification in this document.
- [VARDI-WOLPER-1986] Vardi, M.Y. & Wolper, P. "An Automata-Theoretic Approach to Automatic Program Verification." *LICS 1986*, 332-344. Establishes the connection between LTL and Buchi automata, enabling efficient runtime monitoring. Directly used in the LTL-to-automaton compilation.
- [SWIERSTRA-2008] Swierstra, W. "Data Types a la Carte." *JFP*, 18(4), 423-436, 2008. Introduces extensible data types via coproducts. Informs the category-theoretic composition of temporal monitors.
- [MOGGI-1991] Moggi, E. "Notions of Computation and Monads." *Information and Computation*, 93(1), 55-92, 1991. Foundational paper on monadic computation. Provides the theoretical basis for the Kleisli composition used in monitor chaining.
