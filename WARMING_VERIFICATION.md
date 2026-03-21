# Agent Pre-Warming & Gate-Review Overlap Verification

## Implementation Summary

This document verifies the agent pre-warming and gate-review overlap feature. All changes compile and are architecturally sound. Manual testing required.

### Files Modified

1. **agent/mod.rs** — `MultiAgentPool`
   - Added `warm_pool: HashMap<(AgentRole, String), AgentConnection>`
   - Added `pre_spawn_warm()` — spawn async, no `turn_start`
   - Added `promote_warm()` — retrieve ready agent or cold-spawn fallback
   - Added `evict_warm()` — kill unused warm agent

2. **orchestrator/artifacts.rs** & **orchestrator/registry.rs**
   - Added `#[derive(Clone)]` for `ArtifactStore` and `Registry`
   - Changed `Registry.write_lock` to `Arc<Mutex<()>>` for cloning support

3. **orchestrator/inject.rs** — Context Injection
   - Added `OwnedContextInjector` struct for async spawning
   - Added `pre_inject_implementer_async()` and `pre_inject_reviewer_async()`
   - Both return `JoinHandle<Result<()>>` for background injection

4. **orchestrator/executor.rs** — Execution Planning
   - Added `active_reviewer_instance: Option<String>` to `PlanState`
   - Added `set_active_reviewer()`, `clear_active_reviewer()`, `get_active_reviewer()` helpers
   - Added `StartReviewerInParallel` and `CancelActiveReviewer` actions
   - Comprehensive gate-review overlap pattern documentation (see code comments)

5. **app/parallel.rs** — Event Loop
   - Added handlers for `StartReviewerInParallel` and `CancelActiveReviewer`
   - Executor-level integration ready (event loop wiring needed)

6. **conductor/mod.rs** — Conductor
   - Added `ConductorAction::PingWarmAgent { instance_id }`
   - Handlers added in both parallel and sequential modes

---

## Verification Plan

### Phase 1: Compilation & Warm Pool Basics

✅ Full `cargo build --release` succeeds (37s)

Test warm pool in isolation (requires event loop integration):
```bash
# Terminal 1: Run a Standard complexity plan
cd /Users/will/dev/uniswap/bardo
cargo run --release -- plan 01-sample --complexity standard

# Observe in TUI:
# 1. After Implementer TurnCompleted → should see QuickReviewer spawn log
# 2. Implementer completion timestamp vs. first token from QuickReviewer
#    Expected: <2s (just prompt transmission, no cold-start)
```

### Phase 2: Gate Failure Cancellation

Test gate failure path:
```bash
# Create a plan that will fail compile gates
# Monitor process cleanup:
ps aux | grep claude

# When gates fail with overlap:
# 1. QuickReviewer should be interrupted (turn_interrupt)
# 2. QuickReviewer process should be killed (no zombie processes)
# 3. AutoFix should start (or re-implementation if not express mode)
# 4. Verify: ps aux should show no stale claude processes
```

### Phase 3: Multi-Iteration Plans

Test plans that loop (AutoFix iterations):
```bash
# Run a plan that needs 2-3 AutoFix iterations
# Measure per-iteration time:

Iteration 1: Impl → Compile → Gates Fail → AutoFix
Iteration 2: AutoFix → Compile → Gates Fail or Pass

# Expected improvement: Pre-warming makes handoffs instant
# Without: each transition = ~5-15s (cold start)
# With: each transition = <2s (warm pool + overlap)
```

### Phase 4: Parallel Mode Wave Testing

Test parallel execution with multiple plans:
```bash
# Run a wave of 3+ plans simultaneously
# Each plan should have independent warm pool entries

# Verify:
# 1. Warm pools per-plan don't cross-contaminate
#    (instance IDs are plan-scoped: e.g., "quick:01-alpha" vs "quick:02-beta")
# 2. No agent reuse between plans (each plan kills its agents on completion)
# 3. Token usage doesn't spike for idle warm agents
```

### Phase 5: Context Injection Overlap

Test async context injection:
```bash
# Monitor context/in/ directory in worktree during implementer execution
# Verify context files appear before reviewer turn_start:

Timing:
- Implementer starts → Implementer.elapsed() = 0s
- Context pre-injection spawns (async) → returns immediately
- Implementer runs → 5-30s
- Implementer TurnCompleted → Gates + Reviewer start
- Reviewer turn_start → context/in/ files should be ready
  (pre-injection should complete during implementer's turn)

# If context not ready by turn_start, turn_start awaits injection JoinHandle
```

### Phase 6: Sequential Mode (Future)

Sequential mode documentation added; requires separate implementation:
- AgentPool (sequential) needs equivalent warm pool methods
- Pattern documented in agent/mod.rs
- Event loop integration in app/sequential.rs would mirror parallel mode

---

## Integration Checklist

- [ ] Build succeeds: `cargo build --release`
- [ ] No new compiler warnings beyond existing codebase
- [ ] Single Standard plan runs without errors (warm pool idle state)
- [ ] Implementer → QuickReviewer handoff happens in <2s
- [ ] Gate failure cancels reviewer mid-turn, no zombie processes
- [ ] Multi-iteration plan (AutoFix) loops without leaks
- [ ] Wave of 3 parallel plans completes with independent warm pools
- [ ] Token usage for idle warm agents = 0 (no tokens until turn_start)
- [ ] Context injection completes before reviewer turn_start

---

## Known Limitations

1. **Sequential Mode**: Pre-warming not yet implemented for AgentPool
   - Pattern documented; requires separate PR

2. **Gate-Review Overlap**: Event loop integration directive added
   - Executor logic ready; event loop still needs:
     - Dispatch both `RunPlanGates` AND `StartReviewerInParallel` simultaneously
     - On gate failure, dispatch `CancelActiveReviewer`

3. **PingWarmAgent**: Placeholder implementation
   - Currently logs only; lightweight keepalive protocol not implemented
   - Suitable for future enhancement if agent idle-timeout becomes an issue

---

## Next Steps

1. **Event Loop Wiring** (app/parallel.rs):
   - When Implementer TurnCompleted fires, call:
     ```rust
     executor.pre_spawn_phase_ahead(&plan_id);
     // Dispatch RunPlanGates + StartReviewerInParallel simultaneously
     ```
   - On gate failure, call:
     ```rust
     executor.handle_plan_gates_failed_with_errors(&plan, errors);
     // Dispatch CancelActiveReviewer if active_reviewer exists
     ```

2. **Field Testing**:
   - Run real plans through standard/complex pipelines
   - Measure phase transition times
   - Monitor for edge cases (rapid cancellations, OOM, etc.)

3. **Sequential Mode**:
   - Implement warm pool in AgentPool (pattern in code)
   - Add pre-warming dispatch in app/sequential.rs
   - Test sequential + parallel mode parity

---

## Performance Expectations

Without pre-warming:
- Implementer → compile gate → reviewer = ~60-90s for Cursor, ~5-15s for Claude
- Per-iteration overhead: gate failure → AutoFix restart adds full cold-start

With pre-warming (parallel mode):
- Implementer → review overlap starts immediately (<2s)
- Gates run in parallel with reviewer
- Gate failure cancels reviewer cleanly, AutoFix restarts warm pool
- Expected savings: 30-60s per plan iteration

With pre-warming (sequential mode):
- Would save ~5-15s per phase transition
- Smaller gains than parallel (single-agent pipeline)

---

## Code References

- Warm pool infrastructure: `agent/mod.rs:195-430`
- Executor overlap helpers: `orchestrator/executor.rs:1017-1058`
- Event loop handlers: `app/parallel.rs:1093-1113`
- Pattern documentation: `orchestrator/executor.rs:989-1016`
