# Teardown: Destruction and Cleanup [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> _"Every ending is a choice about what to preserve."_

> **Reader orientation:** This document specifies the 8-step teardown pipeline -- the reverse of provisioning -- that cleans up a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) when it dies or is destroyed. It covers position settlement in dependency order, wallet sweeping, Grimoire (the agent's persistent knowledge base) export to Styx (global knowledge relay and persistence layer at wss://styx.bardo.run), identity deregistration, VM destruction, and the irreversibility invariant. It belongs to the `01-golem` lifecycle layer. The key concept: once teardown begins, it cannot be stopped -- a partially-torn-down Golem is more dangerous than a fully-torn-down one. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## The Teardown Pipeline

The teardown pipeline is the reverse of provisioning. Eight steps, executed sequentially as a Rust type-state machine. Each step is individually retryable and transitions to the next state. Once teardown begins, it cannot be stopped -- this is a safety invariant. A partially-torn-down Golem in an inconsistent state is more dangerous than a fully-torn-down one, because it may hold positions it can no longer manage or keys it can no longer rotate.

The pipeline is identical whether initiated by the owner or triggered by credit exhaustion. The differences are in the trigger, confirmation flow, and surrounding protocol (see S5).

```rust
use std::marker::PhantomData;

// Teardown step marker types (zero-sized)
pub struct TeardownStop;
pub struct TeardownSettle;
pub struct TeardownSweep;
pub struct TeardownExport;
pub struct TeardownPush;
pub struct TeardownDeregister;
pub struct TeardownDestroyVm;
pub struct TeardownEmitDeath;
pub struct TeardownComplete;

pub struct TeardownPipeline<S> {
    pub golem_id: String,
    pub death_cause: DeathCause,
    pub results: TeardownResults,
    _step: PhantomData<S>,
}

impl TeardownPipeline<TeardownStop> {
    /// Begin teardown. Consumes a running Golem, stops the heartbeat.
    pub async fn begin(golem: Golem<Terminal>) -> Result<TeardownPipeline<TeardownSettle>> {
        // Stop heartbeat, cancel pending ops, wait for current tick (max 30s)
        // ...
        Ok(TeardownPipeline {
            golem_id: golem.state.id.clone(),
            death_cause: golem.state.death_cause(),
            results: TeardownResults::default(),
            _step: PhantomData,
        })
    }
}

// Each step: consume self, execute, produce next state.
// Invalid step sequences are compile errors.
```

```
Step 1: Stop Heartbeat
  Stop heartbeat scheduler, cancel pending ops, wait for current tick (max 30s)
     |
Step 2: Settle Positions
  Close LPs, cancel orders, withdraw lending, exit vaults, convert to USDC
     |
Step 3: Sweep Wallets
  Transfer all token balances to designated beneficiary (Owner's wallet)
     |
Step 4: Export Grimoire
  Snapshot knowledge base, push to Styx Archive layer
     |
Step 5: Push Clade
  Upload novelty-prioritized insights to Clade via Styx WebSocket relay
     |
Step 6: Deregister Identity
  Mark ERC-8004 entry as inactive (does NOT burn the token)
     |
Step 7: Destroy VM
  Revoke session key, zero key material, destroy Fly.io VM
     |
Step 8: Emit Death Event
  Store tombstone record, emit webhook, telemetry
```

---

## S1 -- Stop Heartbeat (Step 1)

The first step ensures no new operations can begin while teardown proceeds.

1. Set Golem status to `tearing_down`
2. Stop the heartbeat scheduler -- no new ticks will fire
3. Cancel all pending scheduled operations (DCA executions, rebalance checks, Warden announcements if Warden is deployed)
4. Wait for the current tick to complete (maximum 30 seconds)
5. If the current tick does not complete within 30 seconds, force-terminate it

This step is idempotent. Calling it on an already-paused Golem is a no-op.

---

## S2 -- Settle Positions (Step 2)

Position settlement closes all open DeFi positions and converts proceeds to USDC (or the position's base token if no USDC route exists). Settlement is best-effort -- positions that cannot be automatically closed are flagged for manual intervention.

In Delegation custody mode, all settlement transactions execute from the owner's Smart Account (where funds always were). The session key signs UserOperations bounded by the delegation's caveat enforcers. Settlement may be simpler: the owner retains full control and can handle deferred positions directly from MetaMask.

### 2.1 Topological Dependency Order

Positions are closed in dependency order. Borrow positions must be repaid before the collateral backing them can be withdrawn. The settlement planner builds a dependency graph from position types and resolves it topologically.

Within each dependency tier, positions are settled in order of decreasing value to prioritize recovering the most capital first (in case gas runs out mid-settlement).

**Settlement order:**

1. Cancel all pending Warden announcements if deployed (free, just marks as cancelled)
2. Cancel all pending limit orders and scheduled trades
3. Repay borrow positions (Morpho, Aave, Compound) -- must happen before collateral withdrawal
4. Withdraw from lending supply positions and freed collateral
5. Exit vault positions (ERC-4626 `withdraw()`)
6. Close LP positions (Uniswap V3 `decreaseLiquidity` + `collect`, V4 equivalent)
7. Swap non-USDC tokens to USDC via best available route
8. Revoke session key (Delegation: delegation hash disabled; Embedded: P-256 signer deregistered)
9. Emit `golem.funds_settled` webhook

### 2.2 Borrow Position Resolution

1. **Build the dependency graph**: For each lending protocol, map which supply positions serve as collateral for which borrow positions
2. **Repay borrows first**: Use existing token balances to repay outstanding borrows. If the Golem holds sufficient tokens of the borrowed asset, repay directly. If not, swap other held tokens to the borrowed asset first
3. **Withdraw collateral after repayment**: Once the borrow is fully repaid (or reduced below the liquidation threshold), the collateral supply position is freed and can be withdrawn
4. **Flash loan fallback**: If the Golem lacks sufficient liquid assets to repay a borrow directly, and the borrow is small enough, use a flash loan to atomically repay the borrow and withdraw collateral in a single transaction
5. **Flag if unresolvable**: If none of the above strategies can settle the borrow (e.g., total assets are insufficient to cover the debt), flag the position as `requires_manual_close: true` with a clear explanation

### 2.3 Unsettleable Positions

Some positions cannot be auto-closed:

- Borrow positions where total assets are insufficient to cover the outstanding debt
- Locked positions with time-based vesting (Pendle PT before maturity)
- Positions in paused or exploited protocols (no contract interaction possible)

These are flagged with `requires_manual_close: true` and described in the teardown plan warnings. The teardown pipeline continues regardless -- unsettleable positions do not block subsequent steps.

In Delegation mode, unsettleable positions remain in the owner's Smart Account -- the owner handles them directly. In Embedded mode, they remain in the Privy wallet for manual sweep.

```rust
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementResult {
    pub position: PositionSummary,
    pub settled: bool,
    pub tx_hash: Option<[u8; 32]>,
    pub tokens_received: Vec<TokenBalance>,
    pub error: Option<String>,
    pub requires_manual_close: bool,
}
```

---

## S3 -- Sweep Wallets (Step 3)

After settlement, behavior depends on custody mode:

**Delegation mode:** No sweep needed. Funds were never transferred to the Golem -- they remain in the owner's Smart Account. The session key is simply revoked (delegation hash disabled on-chain). This is the cleanest death: no sweep, no stuck funds, no gas estimation errors during teardown.

**Embedded mode:** All remaining token balances are transferred to the designated beneficiary -- the owner's Main Wallet.

### 3.1 Sweep Order (Embedded Mode)

Tokens are swept in order of decreasing USD value. The highest-value tokens move first, ensuring maximum capital recovery if the sweep is interrupted.

### 3.2 Dust Threshold

A token is classified as dust if its estimated USD value is less than 2x the gas cost to transfer it. On Base (sub-$0.01 gas), this threshold is very low (~$0.02), so most tokens get swept. Dust tokens are reported in the teardown summary but not transferred.

### 3.3 Manual Sweep Fallback (Embedded Mode)

If the session signer is unavailable (already revoked or VM crashed), the owner can sweep funds via the app dashboard. The dashboard uses the owner's Main Wallet signature to authorize a direct sweep through the Privy admin API, bypassing the session signer entirely. This ensures funds are always recoverable even if the Golem's compute environment is gone.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepResult {
    pub swept: Vec<SweptToken>,
    pub dust: Vec<TokenBalance>,
    pub failed: Vec<FailedSweep>,
    pub total_swept_usd: f64,
    pub total_dust_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweptToken {
    pub token: TokenBalance,
    pub tx_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedSweep {
    pub token: TokenBalance,
    pub error: String,
}
```

---

## S4 -- Export Grimoire (Step 4)

The Golem's knowledge base is packaged and persisted to the Styx Archive layer. For owner-initiated teardown, this step is optional (owner checkbox). For credit exhaustion death, it always executes.

### 4.1 Archive Format

The Grimoire archive is serialized with bincode for compact binary encoding and pushed to the Styx Archive layer via the persistent WebSocket connection:

| File               | Contents                                                            | Typical Size |
| ------------------ | ------------------------------------------------------------------- | ------------ |
| `insights.bin`     | All 5 GrimoireEntry types with confidence scores and metadata       | 50--500 KB   |
| `episodes.bin`     | Most recent 1000 episodes                                           | 1--5 MB      |
| `strategy.md`      | Final compiled STRATEGY.md                                          | 5--50 KB     |
| `playbook.md`      | Machine-evolved heuristics (PLAYBOOK.md)                            | 10--100 KB   |
| `performance.bin`  | Trade log, daily P&L, Sharpe ratio, drawdown events                 | 100 KB--2 MB |
| `metadata.toml`    | Creation manifest, runtime config, lifespan stats, credit breakdown | 10--50 KB    |

Target: under 10 MB for a typical Golem with 30 days of operation.

### 4.2 Import Rules

Exported archives can be imported into a new Golem to bootstrap its knowledge. All imported data goes through a quarantine-then-validate pipeline -- nothing is trusted blindly:

- All imported insights start as `candidate` status regardless of original confidence
- Imported insights must shadow-execute locally before promotion to `active`
- Strategy and playbook are loaded as reference documents, not automatically applied
- Episodes are added to episodic memory but do not influence decisions until the Curator validates

### 4.3 Memory Zeroization

After the Grimoire export completes, all sensitive knowledge material in process memory is zeroized. The `zeroize` crate handles this via the `Zeroizing<T>` wrapper -- secrets are overwritten when dropped. This ensures no residual knowledge persists in VM memory after teardown.

```rust
use zeroize::Zeroizing;

/// Sensitive knowledge material that must be zeroized after export.
pub struct SensitiveKnowledge {
    pub session_key: Zeroizing<Vec<u8>>,
    pub grimoire_encryption_key: Zeroizing<Vec<u8>>,
    pub owner_pii_fragments: Zeroizing<Vec<u8>>,
}
// When SensitiveKnowledge is dropped, all fields are overwritten with zeros.
```

---

## S5 -- Push Clade (Step 5)

Before the infrastructure is torn down, the Golem disseminates its knowledge to the Clade via the Styx WebSocket relay. This step ensures the Clade's collective intelligence is updated with the dying Golem's final contributions.

### 5.1 Knowledge Dissemination Protocol

1. **Extract final insights** via ExpeL (distill operational experience into reusable heuristics)
2. **Run verbal reflection** (if Opus inference budget remains) -- the unfiltered death reflection
3. **Push novelty-prioritized insights** to Styx for relay to all connected siblings with the same `user_id`
4. **Notify sibling Golems** via `CladeDeltaImmediate` message (death bloodstain bundle)
5. **Deposit bloodstains** to the Pheromone Field (threat and wisdom pheromones from death warnings)

```rust
/// Push death knowledge to the Clade via Styx relay.
pub async fn push_death_knowledge(
    styx_conn: &mut StyxConnection,
    grimoire: &Grimoire,
    death_testament: &DeathTestament,
) -> Result<CladeDeathPushResult> {
    // Extract clade-eligible entries
    let final_entries = grimoire.get_entries_with_propagation(
        PropagationPolicy::Clade,
    )?;

    // Build bloodstain bundle
    let bloodstain = Bloodstain {
        source_golem_id: grimoire.golem_id().clone(),
        generation: grimoire.generation(),
        death_cause: death_testament.cause.clone(),
        warnings: death_testament.warnings.clone(),
        causal_edges: grimoire.top_causal_edges(50),
        somatic_landscape_fragment: grimoire.landscape_fragment(),
        timestamp: std::time::SystemTime::now(),
        signature: vec![], // EIP-712 signed
    };

    // Push as immediate (bypasses batch cycle)
    styx_conn.send(StyxMessage::CladeDeltaImmediate {
        entries: vec![
            SyncEntry::Bloodstain(bloodstain),
        ],
    }).await?;

    // Push remaining insights as batch
    styx_conn.send(StyxMessage::CladeDeltaBatch {
        entries: final_entries.iter().map(SyncEntry::from).collect(),
        version_vector: grimoire.version_vector(),
        bloom_filter: None,
    }).await?;

    Ok(CladeDeathPushResult {
        entries_pushed: final_entries.len() as u32,
        bloodstain_deposited: true,
    })
}
```

### 5.2 Clade Grimoire Persistence

When an individual Golem is destroyed, its contributions to the Clade persist in two ways: (1) siblings that received the entries via Styx relay retain them in their local Grimoires, and (2) the Styx Archive layer stores the death bundle for 90 days for offline siblings to retrieve on reconnection. Destroying one Golem does not remove its insights from the Clade -- other siblings retain access to those insights. The knowledge survives the knower.

---

## S6 -- Deregister Identity (Step 6)

The Golem's ERC-8004 identity entry is marked as inactive. The token is NOT burned -- just its status changes. This preserves the historical record (other agents can still see that this identity existed and what it accomplished) while preventing future interactions with a dead agent.

---

## S7 -- Destroy VM (Step 7)

This step handles all compute and cryptographic cleanup:

1. **Revoke session key**: Delegation mode -- disable delegation hash on-chain (one tx). Embedded mode -- deregister P-256 key from Privy.
2. **Delete signing policy** -- no further transactions can be authorized
3. **Zero key material from VM memory** -- the `zeroize` crate overwrites `Zeroizing<T>` wrappers on drop
4. **Destroy Fly.io VM** (hosted) or stop local process (self-hosted)
5. **Delete associated volumes and snapshots** -- no persistent storage remains

---

## S8 -- Emit Death Event (Step 8)

The final step creates the permanent record:

1. **Store tombstone record** -- permanent record in the owner's account (never deleted)
2. **Emit webhook** -- `golem.destroyed` for owner-initiated, `golem.dead` for credit exhaustion
3. **Fire telemetry event** -- `golem_destroyed` or `golem_dead`

### 8.1 Tombstone Schema

```rust
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemTombstone {
    pub golem_id: String,
    pub golem_name: String,
    pub wallet_address: Address,
    pub network: String,
    pub chain_id: u64,
    pub custody_mode: String,
    pub death_cause: DeathCause,
    pub lifetime_stats: LifetimeStats,
    pub grimoire_archive_url: Option<String>, // Expires after 90 days
    pub has_dust_tokens: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathCause {
    /// USDC balance hit apoptotic reserve.
    CreditExhaustion,
    /// Maximum generation count reached.
    HayflickLimit,
    /// Underlying model deprecated/unavailable.
    ModelStaleness,
    /// Owner-initiated termination.
    OwnerKill,
    /// Extended inactivity beyond threshold.
    IdleTimeout,
    /// Policy breach triggered forced shutdown.
    SafetyViolation,
    /// Owner-initiated destroy (legacy).
    UserDestroyed,
    /// Migration to new deployment (legacy).
    Migration,
    /// Unrecoverable system failure (legacy).
    SystemError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeStats {
    pub created_at: u64,
    pub destroyed_at: u64,
    pub uptime_hours: f64,
    pub total_funded: f64,
    pub total_earned: f64,
    pub total_spent: f64,
    pub net_pnl: f64,
    pub trades_executed: u32,
    pub insights_generated: u32,
    pub replicants_spawned: u32,
    pub grimoire_entry_count: u32,
}
```

Tombstones enable: reviewing past Golem performance, understanding why Golems failed, tracking total historical activity. They are never deleted.

---

## S9 -- Irreversibility Invariant

Once teardown starts, it cannot be stopped. This is a deliberate safety invariant, not a limitation.

A partially-torn-down Golem is a liability:

- **Positions without management**: If teardown stops after settling some positions but not others, the remaining positions have no active manager. They cannot be rebalanced, hedged, or emergency-exited.
- **Keys without oversight**: If teardown stops after exporting the Grimoire but before revoking the session key, the signing keys remain live with no process monitoring them.
- **Identity without backing**: If teardown stops after sweeping funds but before deregistering identity, the ERC-8004 entry points to an agent that cannot fulfill obligations.

The type-state machine enforces this structurally: each step consumes the previous state and produces the next. There is no method to "go back" -- the consumed state is moved and cannot be re-used. The pipeline either completes fully or leaves the Golem in its pre-teardown state (if step 1 fails, nothing has changed). Individual steps are retryable -- if step 3 (sweep) fails due to a gas spike, it can be retried without re-executing steps 1 and 2. But the pipeline as a whole does not support rollback past step 1.

The owner's only decision point is before teardown begins: the teardown plan review and type-to-confirm dialog (see S10). After confirmation, the pipeline runs to completion.

---

## S10 -- Teardown Plan Review

Before anything executes, the system generates a teardown plan that the owner reviews. Nothing runs until explicit confirmation.

The plan includes:

- All positions to close (type, protocol, value, auto-closeability)
- All tokens to sweep (balance, USD value, dust classification) -- Embedded mode only
- Pending operations to cancel
- Estimated gas cost for settlement
- Grimoire export option (entry count, episode count)
- Lifetime statistics (uptime, P&L, trades executed)
- Warnings (illiquid positions, borrow positions requiring repayment, locked vesting)
- Custody-specific notes (Delegation: "funds remain in your wallet"; Embedded: "funds will be swept to your Main Wallet")

### 10.1 Type-to-Confirm

Destruction uses GitHub-style type-to-confirm. The owner must type the Golem's name to enable the "Destroy Golem" button. This prevents accidental destruction and forces the owner to consciously acknowledge which agent they are terminating.

---

## S11 -- Graceful Shutdown Sequence

The VM has a 30-second budget for in-process cleanup. This is the 10-phase protocol from `01b-runtime-infrastructure.md`:

```
Phase  1 (1s):  Stop accepting new work
Phase  2 (2s):  Cancel in-flight tool calls
Phase  3 (3s):  Flush Grimoire to persistent storage (SQLite WAL sync + LanceDB flush)
Phase  4 (10s): Settlement triage (build BardoManifest)
Phase  5 (8s):  Execute critical settlements (blockchain txs)
Phase  6 (2s):  Write BardoManifest for deferred positions
Phase  7 (1s):  Flush and seal the audit chain (final Merkle hash)
Phase  8 (2s):  Sync Grimoire to Styx (best-effort)
Phase  9 (0s):  Zero secrets (automatic via Zeroizing<T> Drop)
Phase 10 (0s):  Exit

On control plane (after VM drain, no time pressure):
  11. Sweep wallet (Embedded mode only): query balance, transfer
  12. Record sweep in machine_events
  13. Update DB: status='destroyed'
```

The wallet sweep (Embedded mode) runs on the control plane, not the VM. The control plane has no time pressure -- it can retry failed sweeps and wait for RPC confirmation. This frees ~15s from the VM's shutdown budget for Grimoire operations.

In Delegation mode, steps 11-13 simplify: no sweep needed (funds are in the owner's wallet), just delegation expiry and status update.

---

## S12 -- Dream Engine Shutdown

During teardown, the dream engine either completes the current cycle (if one is in progress) or aborts with a partial journal. A dream cycle in the Consolidate or Validate phase is allowed to finish -- aborting mid-consolidation could leave the Grimoire in an inconsistent state. A cycle still in Replay is aborted, and the partial `DreamJournal` is flushed as-is.

Partial dream journals are still pushed to the Styx Archive layer. Incomplete is better than lost. The `DreamConsolidator` staging buffer is flushed: staged revisions that have been validated are applied to the Grimoire before export; revisions still pending validation are discarded with `status: "abandoned_at_death"` logged in the journal.

---

## S13 -- Final Styx Archive Backup

The final Styx Archive backup is the authoritative terminal state, pushed to Styx as the last act before the VM is destroyed. It includes:

- **grimoire_bundle** -- The fully updated Grimoire with any last-moment dream consolidations applied
- **death_testament** -- The completed death reflection, including `EmotionalDeathTestament` and verbal narrative
- **dream_journal** -- The finalized `DreamJournal`, including partial entries from an interrupted cycle
- **insurance_snapshot** -- A last `InsuranceSnapshot` timestamped at teardown initiation

This backup is what the successor's Styx Archive restoration step (see `07-provisioning.md` S2.5) decrypts and loads. If the Golem dies necrotically (hard crash, no graceful shutdown), the most recent periodic Styx Archive backup is used instead.

---

## S14 -- Automated Death Teardown

When a Golem dies from credit exhaustion (not owner-initiated), the teardown pipeline triggers automatically. The same 8-step pipeline runs, with these differences:

| Aspect            | Owner-Initiated Destroy       | Credit Exhaustion Death                                 |
| ----------------- | ----------------------------- | ------------------------------------------------------- |
| Trigger           | Owner clicks "Destroy"        | Credits reach 5% threshold                              |
| Confirmation      | Type-to-confirm               | Automatic (no owner action required)                    |
| Death preparation | Skipped                       | Full protocol (knowledge extraction, verbal reflection) |
| Grimoire export   | Optional (owner checkbox)     | Always exported                                         |
| Fund settlement   | Standard settlement           | Uses ring-fenced death reserve for gas                  |
| Webhook           | `golem.destroyed`             | `golem.dead` (different event)                          |

The death reserve is a small USDC ring-fence ($0.30 base + per-position sweep gas) set aside at creation time. It ensures the Golem can always afford to close positions and sweep funds (Embedded mode) even when its operating balance hits zero. In Delegation mode, the death reserve covers only gas for settlement transactions.

---

## S15 -- Pause vs Destroy

A running Golem can be stopped two ways. The choice determines whether it can come back.

| Action      | Effect                                                | Reversible           | Funds                | Compute      | Knowledge                           |
| ----------- | ----------------------------------------------------- | -------------------- | -------------------- | ------------ | ----------------------------------- |
| **Pause**   | Stops heartbeat, cancels pending ops, preserves state | Yes (resume anytime) | Stay as-is           | VM suspended | Grimoire intact on disk             |
| **Destroy** | Full teardown pipeline, all resources cleaned up      | No                   | Settled/returned     | VM destroyed | Exported to Styx Archive, then deleted|

Pausing is always safe and instantly reversible. Destroying is permanent and requires explicit confirmation.

Paused Golems in hosted mode still consume Bardo Compute credits -- the VM is allocated, just idle. If credits drop to 5%, the automated death teardown begins.

> Credit conservation (30%) is a behavioral mode within the Running state, not a Paused transition. See [02-mortality/01-architecture.md](../02-mortality/01-architecture.md) for behavioral phase details.

---

## S16 -- Death Settlement by Custody Mode

How a Golem's financial state resolves at death depends on the custody mode.

| Aspect | Delegation | Embedded (Privy) |
|--------|-----------|-----------------|
| Funds at death | Owner's wallet (always were) | Privy server wallet |
| Sweep required | No | Yes |
| Stuck fund risk | None | Yes (if sweep fails) |
| Owner action needed | None (delegation expires) | Must claim deferred positions |
| Session key cleanup | Delegation hash disabled on-chain | P-256 key deregistered from Privy |
| Deferred positions | Owner handles from MetaMask directly | Recorded in BardoManifest for manual handling |

Delegation is strictly superior for death settlement. The "no sweep" death eliminates stuck funds, failed sweeps, gas estimation errors during teardown, and the race condition between death and sweep confirmation.

---

## References

- [DENNIS-VAN-HORN-1966] Dennis, J.B. & Van Horn, E.C. (1966). "Programming Semantics for Multiprogrammed Computations." _Communications of the ACM_, 9(3). — *Foundational work on capability-based security; informs the type-state pipeline design where each step consumes the previous state.*
- ERC-4626: `withdraw()` and `redeem()` for vault position exits during settlement. — *The standard vault interface used for closing ERC-4626 positions in the settlement step.*
- Uniswap V3: `decreaseLiquidity()` + `collect()` for LP position closure. — *The protocol calls used to close concentrated liquidity positions during teardown settlement.*
- MetaMask Delegation Framework: https://github.com/MetaMask/delegation-framework — *Enables delegation revocation during teardown: disabling the delegation hash on-chain in a single transaction.*
- `zeroize` crate: https://crates.io/crates/zeroize — *Rust library for automatic secret wiping on drop; ensures no key material persists in VM memory after teardown.*
