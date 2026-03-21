# Next Steps: Activating Warm Reviewer Turn Start

The infrastructure is in place. Warm reviewers are pre-spawned when gates run. The final step is to actually call `turn_start()` on them instead of cold-spawning.

## Quick Integration (Est. 30 minutes)

### Step 1: Warm Reviewer Turn Start

File: `app/parallel.rs` — in `RunPlanReviews` handler (around line 241)

Currently:
```rust
ExecutorAction::RunPlanReviews { ref plan } => {
    // ... state updates ...
    // Then later in reviews logic:
    let aid = AgentInstanceId::new(role, format!("{}:{}", role, plan));
    pool.spawn_instance(aid.clone(), None, &effort, model).await?;
    pool.turn_start(&aid, &prompt, model).await?;
}
```

Change to:
```rust
ExecutorAction::RunPlanReviews { ref plan } => {
    // ... state updates ...
    // Check for warm reviewer first
    if let Some(warm_instance_id) = executor.get_active_reviewer(plan) {
        let aid = AgentInstanceId::new(role, warm_instance_id.clone());
        if pool.is_spawned(&aid) {
            // Turn start the warm agent
            pool.turn_start(&aid, &prompt, model).await?;
            executor.clear_active_reviewer(plan);
            state.add_log("executor", &format!("Started warm {} for {}", role, plan), LogLevel::Info);
        } else {
            // Warm agent didn't make it; cold spawn fallback
            let aid = AgentInstanceId::new(role, format!("{}:{}", role, plan));
            pool.spawn_instance(aid.clone(), None, &effort, model).await?;
            pool.turn_start(&aid, &prompt, model).await?;
        }
    } else {
        // No warm agent; cold spawn as before
        let aid = AgentInstanceId::new(role, format!("{}:{}", role, plan));
        pool.spawn_instance(aid.clone(), None, &effort, model).await?;
        pool.turn_start(&aid, &prompt, model).await?;
    }
}
```

**Why this order?**
1. First check if warm agent exists (`get_active_reviewer`)
2. Verify it's actually in the pool (`is_spawned`)
3. Start its turn (no spawn, just `turn_start`)
4. Clear the active reviewer flag
5. Fallback to cold spawn if warm isn't ready (graceful degradation)

### Step 2: Test Compilation

```bash
cargo build --release
```

Should compile without new errors (only pre-existing warnings).

### Step 3: Field Test

Run a **single Standard complexity plan**:

```bash
cd /Users/will/dev/uniswap/bardo
cargo run --release -- plan 01-sample --complexity standard
```

**What to watch**:
- After Implementer completes, gates should start
- QuickReviewer should start *while gates are running* (not after)
- Implementer → FirstToken from Reviewer should be <5s (instead of 5-15s or 60-90s)
- No errors in logs about "Agent not spawned" for reviewer

**If it breaks**:
- Check that `PreSpawnWarmReviewer` is being dispatched (search logs for "Pre-spawning warm")
- Check that warm agent is being promoted (search logs for "Warm ... ready for")
- Verify `turn_start` is being called on the right instance ID

---

## Optional: Context Pre-Injection

File: `orchestrator/inject.rs` — wire up async context preparation

**Goal**: Write context/in/ files while implementer runs, not when reviewer starts.

**Where**: When starting implementer, call:

```rust
let injector = executor.artifact_store.to_owned();
let mut context_handles = HashMap::new();
context_handles.insert(
    plan.clone(),
    injector.pre_inject_reviewer_async(
        worktree.clone(),
        plan_num.clone(),
        iter,
        plan_deps.clone(),
    ),
);
// Store context_handles in executor or state for later use
```

**When reviewer is about to start**:
```rust
if let Some(handle) = context_handles.remove(plan) {
    handle.await.ok(); // Wait for injection if not done yet
}
```

**Timing**: If implementer runs for 10-30s, async injection finishes long before reviewer starts.

---

## Optional: Other Gate Failures

Currently, cancellation wrap only applied to Compile gate (line 3500).

To apply to Terminal Render and Golem gates, add same pattern around lines 3544-3560:

```rust
if let Some(reviewer_id) = executor.get_active_reviewer(&plan) {
    actions.push(ExecutorAction::CancelActiveReviewer {
        plan: plan.clone(),
        instance_id: reviewer_id,
    });
}
```

---

## Verification Commands

```bash
# Check warm pool exists
grep -n "warm_pool" src/agent/mod.rs

# Check executor has helpers
grep -n "set_active_reviewer" src/orchestrator/executor.rs

# Check event loop has handlers
grep -n "PreSpawnWarmReviewer" src/app/parallel.rs

# Verify build
cargo build --release 2>&1 | tail -3
```

---

## Performance Benchmarking

Before/after measurement for a Standard plan:

**Baseline** (without warm reviewer turn_start):
```
Implementer TurnCompleted   T=0s
QuickReviewer spawn+start   T=0-15s (Claude CLI)
Review completes            T=120s
```

**After warm reviewer enabled**:
```
Implementer TurnCompleted   T=0s
QuickReviewer already warm  T=0s (no cold-start)
Review runs                 T=120s
→ Saves 15s per iteration
```

For complex plans with multiple iterations:
- Each iteration saves 15s
- 3-iteration plan: 45s total savings

---

## Rollback

If issues arise:

```bash
# Remove the warm reviewer turn_start logic
git diff src/app/parallel.rs  # Review your changes
git checkout src/app/parallel.rs  # Revert
cargo build --release
```

The warm pool infrastructure stays active but unused (agents spawn cold as before). No performance regression.

---

## Code References

- Warm pool implementation: `agent/mod.rs:195-430`
- Executor helpers: `orchestrator/executor.rs:1039-1051`
- Event loop dispatch: `app/parallel.rs:244-283`
- RunPlanReviews handler: `app/parallel.rs:241-460`
- Gate result handling: `app/parallel.rs:3500-3548`

---

**Estimated integration time**: 30–60 minutes
**Complexity**: Low (straightforward fallthrough logic)
**Risk**: Low (graceful fallback to cold spawn)
**Expected impact**: 15-60s savings per plan iteration (depending on backend)
