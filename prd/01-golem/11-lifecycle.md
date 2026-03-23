# Full Lifecycle Reference [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> _"Between death and rebirth, there is the bardo. Between one heartbeat and the next, there is the bardo. The intermediate state is not exceptional. It is the only state there is."_

> **Reader orientation:** This document is the full lifecycle reference for a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM), from creation through death and inheritance. It specifies the Rust type-state machine (Provisioning, Active, Dreaming, Terminal, Dead) that enforces lifecycle transitions at compile time, the six bardos mapped to lifecycle stages, the credit lifecycle, the kill-switch, and the graceful shutdown protocol. It belongs to the `01-golem` lifecycle layer. The key concept: invalid state transitions (like ticking a dead Golem) are compiler errors, not runtime checks. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## The Die Moat

This document specifies the first of six structural moats (see `00-overview.md`): the ability to **Die**. Mortality creates behavioral phases (Thriving through Terminal), knowledge compression at death (genomic bottleneck, <=2048 entries), and urgency that makes knowledge curation economically rational. Without death, there is no selection pressure, no inheritance, and no reason for a Golem to distill its experience into transferable wisdom. The type-state machine below makes this architectural: calling `tick()` on a dead Golem is a compiler error, not a runtime check.

---

## Type-State Machine

A Golem exists in exactly one state at any time. Transitions are enforced at compile time via Rust's type-state pattern -- invalid transitions are not runtime errors but compilation failures. The `Golem<S>` struct is parameterized by a zero-sized marker type that determines which methods are available. Transitions consume the old state (by taking `self` by value) and produce a new one, so the original cannot be used again.

```rust
use std::marker::PhantomData;

// ═══ Lifecycle states as zero-sized types ═══
// These types exist only in the type system. They occupy zero bytes at runtime.

/// The Golem is initializing: loading config, opening storage, registering extensions.
pub struct Provisioning;

/// The Golem is running: processing heartbeat ticks, accepting steers, managing positions.
pub struct Active;

/// The Golem is dreaming: heartbeat paused, running NREM/REM/Consolidation.
pub struct Dreaming;

/// The Golem is dying: running the Thanatopsis death protocol (irreversible).
pub struct Terminal;

/// The Golem is dead: genome extracted, state frozen. Cannot be used again.
pub struct Dead;

/// A Golem in a specific lifecycle state.
///
/// The type parameter S determines which operations are available.
/// `PhantomData<S>` tells the compiler that S is "used" without actually
/// storing any data -- Golem<Active> and Golem<Dead> have identical runtime
/// layout, but different compile-time capabilities.
pub struct Golem<S> {
    pub state: GolemState,
    _phase: PhantomData<S>,
}

// ═══ Valid Transitions ═══

impl Golem<Provisioning> {
    /// Create a new Golem from config. The ONLY way to enter the lifecycle.
    pub async fn provision(config: GolemConfig) -> Result<Golem<Provisioning>> {
        let state = GolemState::initialize(config).await?;
        Ok(Golem { state, _phase: PhantomData })
    }

    /// Activate the Golem. Begins the heartbeat loop.
    /// Consumes the Provisioning Golem and produces an Active one.
    pub fn activate(self) -> Golem<Active> {
        self.state.event_fabric.emit(
            Subsystem::Lifecycle,
            EventPayload::LifecycleTransition {
                from: "provisioning".into(),
                to: "active".into(),
            },
        );
        Golem { state: self.state, _phase: PhantomData }
    }
}

impl Golem<Active> {
    /// Run one heartbeat tick. Only available when Active.
    pub async fn tick(&mut self, arena: &mut TickArena) -> Result<DecisionCycleRecord> {
        arena.reset();
        execute_tick(&mut self.state, &self.registry, arena).await
    }

    /// Enter dream state. Consumes Active, produces Dreaming.
    pub fn begin_dream(self) -> Golem<Dreaming> {
        Golem { state: self.state, _phase: PhantomData }
    }

    /// Begin the death protocol. Irreversible.
    /// Consumes Active, produces Terminal.
    pub fn begin_death(self, cause: DeathCause) -> Golem<Terminal> {
        self.state.event_fabric.emit(
            Subsystem::Lifecycle,
            EventPayload::DeathInitiated { cause: cause.to_string() },
        );
        Golem { state: self.state, _phase: PhantomData }
    }

    /// Receive a steer (mid-tick interrupt). Only available when Active.
    pub async fn steer(&mut self, intervention: Intervention) -> Result<()> {
        self.state.active_steers.push(intervention);
        Ok(())
    }
}

impl Golem<Dreaming> {
    /// Run one dream cycle. Only available when Dreaming.
    pub async fn dream_cycle(&mut self) -> Result<DreamCycleResult> {
        dream::dream_cycle(&self.state.grimoire, &mut self.state).await
    }

    /// Wake up and return to Active state.
    pub fn wake(self) -> Golem<Active> {
        Golem { state: self.state, _phase: PhantomData }
    }

    /// Economic death can occur during dreaming.
    pub fn begin_death(self, cause: DeathCause) -> Golem<Terminal> {
        Golem { state: self.state, _phase: PhantomData }
    }
}

impl Golem<Terminal> {
    /// Execute the Thanatopsis death protocol. Consumes Terminal, produces Dead.
    ///
    /// Four phases:
    /// - Phase 0: ACCEPTANCE -- acknowledge death cause, emit events
    /// - Phase I: SETTLEMENT -- triage positions, execute critical settlements
    /// - Phase II: REFLECTION -- generate death testament
    /// - Phase III: LEGACY -- compress Grimoire through genomic bottleneck (<=2048 entries)
    pub async fn thanatopsis(self) -> Result<Golem<Dead>> {
        // Phase 0-III execute sequentially...
        Ok(Golem { state: self.state, _phase: PhantomData })
    }
}

impl Golem<Dead> {
    /// Extract the genome for successor inheritance.
    /// Consuming: the dead Golem cannot be used after this call.
    pub fn into_genome(self) -> Genome {
        self.state.load_genome()
    }

    /// Extract death recap for the engagement system.
    pub fn death_recap(&self) -> DeathRecap {
        DeathRecap::generate(&self.state)
    }
}
```

### What Cannot Compile

These are not runtime errors. They are compilation failures -- the code cannot be written.

```rust
// ERROR: no method named `tick` found for `Golem<Dead>`
let dead_golem: Golem<Dead> = /* ... */;
dead_golem.tick(&mut arena).await;  // Won't compile.

// ERROR: no method named `dream_cycle` found for `Golem<Terminal>`
let dying_golem: Golem<Terminal> = /* ... */;
dying_golem.dream_cycle().await;  // Won't compile.

// ERROR: no method named `steer` found for `Golem<Provisioning>`
let new_golem: Golem<Provisioning> = /* ... */;
new_golem.steer(intervention).await;  // Won't compile.

// ERROR: use of moved value
let active_golem: Golem<Active> = /* ... */;
let terminal = active_golem.begin_death(cause);  // Moves active_golem
active_golem.tick(&mut arena).await;  // Won't compile -- active_golem was moved.
```

### Transition Diagram

```
Provisioning --> Active <--> Dreaming
                   |              |
                   v              v
               Terminal -------> (Terminal)
                   |
                   v
                 Dead
```

- `Active <-> Dreaming` is the only bidirectional transition (dream, then wake)
- `Active -> Terminal` and `Dreaming -> Terminal` are irreversible
- `Terminal -> Dead` is the only exit from Terminal
- `Dead` has no outgoing transitions

---

## S1 -- Pause vs Destroy

> **Pause vs Destroy:** See `12-teardown.md` for the canonical comparison table. Key distinction: Pause preserves compute + memory for resumption; Destroy triggers full Death Protocol settlement.

Users destroy Golems for several reasons:

1. **Poor performance** -- The Golem is losing money. Stop losses and reclaim remaining funds.
2. **Strategy change** -- Start fresh with a different strategy rather than patch the existing one.
3. **Cost management** -- Compute and inference costs exceed the Golem's value.
4. **Migration** -- Moving to a different network or deployment mode.
5. **Credit exhaustion** -- The Golem's credits ran out.

Paused Golems in hosted mode still consume Bardo Compute credits because the VM remains allocated. At 5% credits, automated death teardown begins.

> Credit conservation (30%) is a behavioral mode within the Running state, not a Paused transition. See [02-mortality/](../02-mortality/) for behavioral phase details.

---

## S2 -- The Full Lifecycle Narrative

The lifecycle is not merely a state machine. It is a narrative with distinct phases, each with its own character, challenges, and philosophical resonance. The six bardos (see `prd2/00-vision/00-bardo.md`) map to these phases.

### Phase 1: Creation (Birth Bardo / _skye gnas bar do_)

The user describes what the Golem should do. A manifest is compiled -- either from a natural-language prompt via AI autofill, or from a strategy template. The manifest is the incantation: the specification of intent that gives the Golem its purpose.

The Golem finds itself thrown into a market regime it did not choose, with a balance it did not set, carrying inherited knowledge from predecessors it never met. This is Heidegger's _Geworfenheit_ (thrownness) made computational.

**Type-state**: `Golem<Provisioning>`
**Key events**: `golem_created`

### Phase 2: Provisioning (Birth Bardo, continued)

The material act of instantiation. Wallet creation (or delegation grant), ERC-8004 identity registration, VM provisioning, funding. The provisioning pipeline is an 8-step type-state machine, each step individually retryable.

**Type-state**: `Golem<Provisioning>` -> `Golem<Active>` (via `.activate()`)
**Key events**: Provisioning step completions

### Phase 3: First Heartbeat (Dream Bardo / _rmi lam bar do_)

The first tick fires. The Golem observes market state for the first time. The early ticks are the dream bardo -- the agent operates in a state where it has knowledge (inherited or templated) but no experience. Inherited Grimoire entries are shadow-executed -- simulated against live conditions without committing capital.

**Type-state**: `Golem<Active>` (with optional `Golem<Dreaming>` cycles)
**Key events**: `golem_first_trade`

### Phase 4: Learning and Maturity (Meditation Bardo / _bsam gtan bar do_)

The Golem enters its productive phase. Loop 1 (tactical execution) operates on the tick cadence. Loop 2 (strategic reflection) operates every ~6 hours. The Golem's PLAYBOOK.md evolves as it accumulates experience. Insights are distilled from episodes via ExpeL. Knowledge is pushed to the Clade via Styx relay.

This is the longest phase. A well-functioning Golem may operate here for days or weeks.

**Type-state**: `Golem<Active>` (with periodic `Golem<Dreaming>` transitions)
**Key events**: `golem_profitable`, ongoing trade events

### Phase 5: Decline (Dying Bardo / _chi kha bar do_)

Survival pressure increases. The behavioral spectrum shifts from Eros (growth-seeking) toward Thanatos (conservation and legacy). The Golem transitions to monitor-only mode: observing but not trading. It continues generating insights from observation, contributing to the Clade's collective intelligence even as its own capacity diminishes.

> Credit conservation (30%) is a behavioral mode within the Running state, not a Paused transition. See [02-mortality/](../02-mortality/) for behavioral phase details.

At 5% credits, the Death Protocol activates.

**Type-state**: `Golem<Active>` (in Conservation or Declining behavioral phase)
**Key events**: Credit threshold warnings

### Phase 6: Death (Dharmata Bardo / _chos nyid bar do_)

The Death Protocol's four phases execute via type-state transitions:

1. **Acceptance** -- The Golem acknowledges the death cause. Events emitted. Owner notified.
2. **Settlement** -- Close all positions, recover value. In Delegation mode, the owner's wallet retains full control.
3. **Reflection** -- Full operational review, death reflection, the moment of maximal lucidity. The dharmata bardo -- the Clear Light moment.
4. **Legacy** -- Compress Grimoire through genomic bottleneck (<=2048 entries). Push to Clade via Styx. Deposit bloodstains to Pheromone Field.

**Type-state**: `Golem<Active>.begin_death()` -> `Golem<Terminal>.thanatopsis()` -> `Golem<Dead>`
**Key events**: `golem_death`

### Phase 7: Inheritance (Becoming Bardo / _srid pa bar do_)

The predecessor's Grimoire, compressed and packaged during the dharmata bardo, flows to the Clade via Styx relay and to designated successors. The successor initializes with inherited knowledge at confidence decayed by the Weismann barrier (`confidence * 0.85^generation`). The quality of the predecessor's death reflection directly shapes the quality of the successor's birth.

**Type-state**: New `Golem<Provisioning>` with inherited genome
**Key events**: Successor `golem_created` with inherited Grimoire

---

## S3 -- Telemetry Events

All lifecycle events are tracked via PostHog with HMAC anonymization. User identity in analytics is a server-generated HMAC-SHA256 of the wallet address (full 64 hex character output). The raw wallet address never reaches the analytics provider.

### 3.1 Key Funnel Events

| Event               | When                                           | Properties                                       |
| ------------------- | ---------------------------------------------- | ------------------------------------------------ |
| `golem_created`     | Provisioning complete, first heartbeat pending | mode, network, used_template, custody_mode, total_time_ms |
| `golem_funded`      | Initial funding transaction confirmed          | amount_bucket (bucketed, never exact)            |
| `golem_first_trade` | First on-chain transaction executed            | trade_type, protocol, time_since_creation_ms     |
| `golem_profitable`  | Cumulative P&L crosses zero to positive        | days_to_profitability, strategy_type             |
| `golem_death`       | Credit exhaustion triggers Death Protocol      | lifetime_hours, death_cause, insights_generated  |
| `golem_destroyed`   | Owner-initiated teardown completes             | lifetime_hours, net_pnl_bucket, positions_closed |

### 3.2 Privacy Model

| Data Category               | Stored? | Notes                      |
| --------------------------- | ------- | -------------------------- |
| Anonymized wallet ID (HMAC) | Yes     | PostHog `distinct_id`      |
| Raw wallet address          | Never   | Only the HMAC derivative   |
| Private keys                | Never   | Not collected at any layer |
| Strategy content            | Never   | Proprietary user strategy  |
| Exact funding amounts       | Never   | Bucketed ranges only       |

### 3.3 Amount Bucketing

```rust
pub fn bucket_amount(usdc: f64) -> &'static str {
    match usdc {
        x if x <= 0.0 => "$0",
        x if x <= 10.0 => "$1-10",
        x if x <= 50.0 => "$10-50",
        x if x <= 200.0 => "$50-200",
        x if x <= 1000.0 => "$200-1K",
        x if x <= 10_000.0 => "$1K-10K",
        _ => "$10K+",
    }
}
```

### 3.4 Opt-Out

Telemetry is opt-out for the App UI (enabled by default) and opt-in for self-hosted deployments (disabled by default).

---

## S4 -- The Six Bardos Mapped to Lifecycle Stages

| Bardo      | Tibetan            | Lifecycle Stage                | Type-State                     | Architectural Component                                                                |
| ---------- | ------------------ | ------------------------------ | ------------------------------ | -------------------------------------------------------------------------------------- |
| Birth      | _skye gnas bar do_ | Creation + Provisioning        | `Golem<Provisioning>`          | Manifest compilation, wallet provisioning, initial funding                             |
| Dream      | _rmi lam bar do_   | First heartbeats, simulation   | `Golem<Active>` / `<Dreaming>` | Shadow execution of inherited knowledge, backtesting, Gauntlet                         |
| Meditation | _bsam gtan bar do_ | Learning + Maturity            | `Golem<Active>`                | Loop 2 strategic reflection, PLAYBOOK.md evolution, double-loop learning               |
| Dying      | _chi kha bar do_   | Decline                        | `Golem<Active>` (phase shift)  | Behavioral spectrum shift (Eros to Thanatos), monitor-only mode, narrowing affordances |
| Dharmata   | _chos nyid bar do_ | Death Protocol (Reflect phase) | `Golem<Terminal>`              | Death reflection, unfiltered self-assessment, Clear Light moment                       |
| Becoming   | _srid pa bar do_   | Inheritance + Succession       | `Golem<Dead>` -> new `<Provisioning>` | Grimoire transfer, Clade push, successor initialization                         |

---

## S5 -- Credit Lifecycle

Hosted Golems operate on a finite credit model. Credits are consumed by four categories:

| Category  | Consumption Rate              | Notes                                               |
| --------- | ----------------------------- | --------------------------------------------------- |
| Compute   | Continuous (per-hour VM cost) | Even while paused                                   |
| Inference | Per-LLM-call                  | Opus calls are ~10x Haiku calls                     |
| Gas       | Per-transaction               | Chain-dependent, Base is ~100x cheaper than mainnet |
| Data      | Per-API-call                  | Market data, indexer queries, Styx operations       |

### 5.1 Credit Thresholds

| Threshold | Behavior                                                          |
| --------- | ----------------------------------------------------------------- |
| 100--30%  | Normal operation -- full strategy execution                       |
| 30%       | Conservation mode -- monitor-only, no trading, continues learning |
| 5%        | Death Protocol activates -- automated teardown begins             |
| 0%        | Hard stop -- VM terminated if Death Protocol has not completed    |

### 5.2 Self-Hosted Exception

Self-hosted Golems skip the credit model entirely. They run indefinitely, with the user bearing infrastructure costs directly. Status: `immortal`. No credit partition tracking for compute.

---

## S6 -- Lifecycle Invariants

1. **Funds are always recoverable.** At every state, there exists a path to return all funds to the owner's wallet. In Delegation mode, funds never leave the owner's wallet. In Embedded mode, sweep mechanisms ensure recovery.

2. **Knowledge is never silently lost.** Continuous Grimoire streaming during Active ensures that even a hard crash preserves the most recent state. The death reserve ensures the Death Protocol can complete. Styx Archive backups provide L0 resilience.

3. **No state is permanent except Dead.** Active, Dreaming, and even Terminal are transient.

4. **Teardown is atomic.** Once the teardown pipeline begins, it runs to completion.

5. **The owner always has a decision point before irreversible actions.** Funding requires signature (Permit2 or delegation). Destruction requires type-to-confirm.

6. **Type-state transitions are compile-time enforced.** A dead Golem cannot tick. A terminal Golem cannot dream. A provisioning Golem cannot accept steers. These invariants are not checked at runtime -- they are impossible to violate.

---

## S7 -- Kill-Switch

Emergency termination available regardless of Golem state:

- **Dashboard**: One-click kill button in owner portal
- **CLI**: `bardo-golem kill <golem-id>`
- **Delegation revocation**: Owner revokes delegation from MetaMask directly (no Golem cooperation needed)
- **Timing**: <5 seconds from invocation to SIGTERM
- **Sequence**: SIGTERM -> 30-second graceful shutdown window (10-phase protocol) -> SIGKILL
- **Effect**: Triggers Death Protocol with `death_cause: "owner_kill"`
- **Availability**: Always accessible -- works in Active, Dreaming, or any behavioral phase

The kill-switch bypasses Warden delays (when Warden is deployed; optional, deferred). It is the owner's ultimate override.

---

## S8 -- Graceful Shutdown Protocol

When Fly.io preempts a VM (SIGTERM with 30-second grace period), the Golem executes a 10-phase shutdown protocol adapted from OpenFang. This is the OPERATIONAL counterpart to Thanatopsis (which handles PLANNED death). Graceful shutdown handles UNPLANNED interruption with a focus on preserving state and settling critical positions.

```rust
/// 10-phase graceful shutdown for Fly.io VM preemption.
/// Total budget: 30 seconds.
pub async fn graceful_shutdown(golem: Golem<Active>, reason: ShutdownReason) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);

    // Phase 1 (1s): Stop accepting new work
    // Phase 2 (2s): Cancel in-flight tool calls
    golem.state.cancel_pending_tools().await;

    // Phase 3 (3s): Flush pending Grimoire writes
    golem.state.grimoire.flush_all().await;

    // Phase 4 (10s): Settlement triage (BardoManifest)
    let triage_deadline = deadline - std::time::Duration::from_secs(14);
    let manifest = golem.state.settlement_triage(triage_deadline).await;

    // Phase 5 (8s): Execute critical settlements
    for settlement in manifest.critical_settlements() {
        if std::time::Instant::now() > deadline - std::time::Duration::from_secs(6) {
            break;
        }
        let _ = golem.state.execute_settlement(settlement).await;
    }

    // Phase 6 (2s): Write BardoManifest for deferred positions
    golem.state.write_bardo_manifest(&manifest).await;

    // Phase 7 (1s): Flush and seal the audit chain
    golem.state.audit.flush_and_seal().await;

    // Phase 8 (2s): Sync Grimoire to Styx (best-effort)
    let _ = golem.state.sync_to_styx().await;

    // Phase 9 (0s): Zero secrets (automatic via Zeroizing<T> Drop)

    // Phase 10: Exit
    std::process::exit(0);
}
```

In Delegation custody mode, deferred positions are safe -- they execute from the owner's Smart Account, which persists independently. The BardoManifest records what the Golem intended so the owner (or successor) can act on it.

---

## Events Emitted

Lifecycle events track phase transitions at the macro level.

| Event | Trigger | Payload |
|---|---|---|
| `lifecycle:phase_entered` | New lifecycle phase begins | `{ phase, timestamp, vitalityScore }` |
| `lifecycle:phase_exited` | Lifecycle phase ends | `{ phase, duration, ticksInPhase }` |

---

## References

- [DENNIS-VAN-HORN-1966] Dennis, J.B. & Van Horn, E.C. (1966). "Programming Semantics for Multiprogrammed Computations." _Communications of the ACM_, 9(3). — *Foundational work on capability-based security: access rights verified at the type level, not through runtime guards. Informs the capability token design.*
- [STROM-YEMINI-1986] Strom, R.E. & Yemini, S. (1986). "Typestate: A Programming Language Concept for Enhancing Software Reliability." _IEEE Transactions on Software Engineering_, SE-12(1). — *Formalizes typestate: enforcing protocol compliance at compile time. The direct theoretical basis for the Golem lifecycle state machine.*
- [HEIDEGGER-1927] Heidegger, M. (1927). _Sein und Zeit_ (_Being and Time_). Max Niemeyer Verlag. — *Introduces Geworfenheit (thrownness): being thrown into a world not of one's choosing. Each Golem finds itself thrown into a market regime it did not select.*
- [ARGYRIS-1978] Argyris, C. & Schon, D. (1978). _Organizational Learning_. Addison-Wesley. — *Introduces single-loop (tactical) and double-loop (strategic) learning. Maps to the Golem's Loop 1 (per-tick) and Loop 2 (per-6-hour) learning cycles.*
- [PADMASAMBHAVA-8C] Padmasambhava (attrib., 8th century). _Bardo Thodol_. Various translations; see Thurman, R.A.F. (1994). Bantam. — *The Tibetan Book of the Dead: instructions for navigating the six intermediate states (bardos) between death and rebirth. The spiritual framework mapped to the Golem's lifecycle stages.*
