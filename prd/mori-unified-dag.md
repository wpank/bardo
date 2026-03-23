# The unified task DAG

## Plan-level scheduling is wasteful

Most build orchestrators treat plans as atoms. Plan 1 runs. Plan 1 finishes. Plan 2 starts. If Plan 1 has five tasks and Plan 2 has three, and none of them touch the same files, you've wasted the time Plan 2 spent waiting. Eight tasks could have run simultaneously.

Mori doesn't schedule plans. It schedules tasks.

The unified task DAG flattens every task from every plan into a single directed acyclic graph. Dependencies are edges. Files are annotations on each node. The scheduler looks at this graph and asks one question: "Which tasks can run right now without conflicting with anything already running?" The answer is often tasks from multiple plans at once.

The difference matters at scale. A build with 12 plans and 40 total tasks might have a theoretical sequential runtime of 3 hours. The unified DAG, by interleaving non-conflicting tasks across plans, can compress that to under an hour. The constraint isn't plan ordering -- it's file contention.

## GlobalTaskId and cross-plan dependencies

Every task in the DAG gets a globally unique identifier in `"plan:task"` format: `"03:T1"`, `"07:T3"`, `"12:T2"`. The struct is minimal:

```rust
pub struct GlobalTaskId {
    pub plan: String,
    pub task: String,
}
```

Display format is `plan:task`. Parsing splits on the colon. That's it.

The DAG itself holds six maps:

```rust
pub struct UnifiedTaskDag {
    nodes: Vec<GlobalTaskId>,
    deps: HashMap<GlobalTaskId, HashSet<GlobalTaskId>>,
    reverse_deps: HashMap<GlobalTaskId, HashSet<GlobalTaskId>>,
    file_sets: HashMap<GlobalTaskId, HashSet<String>>,
    estimates: HashMap<GlobalTaskId, u32>,
    plan_for_task: HashMap<GlobalTaskId, String>,
}
```

`deps` maps each task to the set of tasks it depends on. `reverse_deps` is the inverse -- given a task, which tasks depend on it. `file_sets` records which files each task will modify. `estimates` holds per-task time estimates in minutes.

Dependencies can cross plan boundaries. A task in plan 09 can declare a dependency on `"07:T1"`, meaning it needs task T1 from plan 07 to finish before it starts. The DAG resolves these cross-plan references during construction: it maps plan number prefixes to full plan base names, then wires the edges. Intra-plan dependencies (task T3 depends on T1 within the same plan) work the same way, just without the prefix.

The graph isn't plan-scoped. It's a single structure that spans the entire build.

## File conflict detection

`next_runnable()` is the core scheduling function. It takes two sets -- completed tasks and in-flight tasks -- and returns every task that can start right now.

The algorithm:

```
function next_runnable(completed, in_flight):
    blocked_files = union of all files touched by in_flight tasks

    for each task in the DAG:
        skip if task is completed
        skip if task is in-flight
        skip if any dependency is not in completed
        skip if any of task's files appear in blocked_files
        -> task is runnable

    return all runnable tasks
```

Four conditions, checked in order. The first two are obvious. The third enforces the dependency graph. The fourth is the safety mechanism.

Two tasks editing `src/auth/mod.rs` never run at the same time. Two tasks editing `src/auth/session.rs` and `src/cache/store.rs` run in parallel with no issue. The file sets are the ground truth. If the sets don't intersect, the tasks are independent, regardless of which plans they belong to.

In the implementation, `in_flight_files` is built once per call by flat-mapping over all in-flight tasks' file sets. Each candidate task checks its own file set against this collected set. The check is O(files_per_task) per candidate, which is fast -- task file sets are small, usually 2-8 entries.

## Critical path estimation

Each task carries an `estimated_minutes` value. The DAG uses these to compute the critical path: the longest weighted chain through the dependency graph.

```rust
pub fn critical_path_minutes(&self) -> u32 {
    let topo = self.topological_order();
    let mut dist: HashMap<&GlobalTaskId, u32> = HashMap::new();

    // Initialize each node with its own time
    for gid in &topo {
        dist.insert(gid, self.estimates.get(gid).copied().unwrap_or(0));
    }

    // Relax edges in topological order
    for gid in &topo {
        let current_dist = dist[gid];
        if let Some(dependents) = self.reverse_deps.get(gid) {
            for dep in dependents {
                let dep_time = self.estimates.get(dep).copied().unwrap_or(0);
                let new_dist = current_dist + dep_time;
                let entry = dist.entry(dep).or_insert(0);
                if new_dist > *entry {
                    *entry = new_dist;
                }
            }
        }
    }

    dist.values().copied().max().unwrap_or(0)
}
```

Standard longest-path on a DAG. Process nodes in topological order, relax forward edges, take the maximum. The result feeds two things: ETA calculations displayed in the TUI, and merge queue ordering (plans on the critical path get scheduling priority so they don't become bottlenecks).

Consider a 4-plan build:

```
Plan 01 (T1: 5min, T2: 10min, T3: 3min)  -- no deps
Plan 02 (T1: 8min, T2: 4min)              -- no deps
Plan 03 (T1: 6min, T2: 7min)              -- depends on Plan 01
Plan 04 (T1: 12min)                        -- depends on Plan 03
```

The critical path runs through `01:T2 (10min) -> 03:T2 (7min) -> 04:T1 (12min)` = 29 minutes. Plans 01 and 02 run in parallel. Plan 02's tasks finish in 12 minutes total and sit idle while the critical path continues through Plans 03 and 04. The TUI shows 29 minutes as the ETA, not 55 (the sum of all task estimates).

## Synthetic nodes and atomic plans

Not every plan has a `tasks.toml` with a per-task breakdown. Some plans are small enough to be a single unit of work. The DAG handles this with a synthetic `__whole__` node.

When a plan has no task file, the builder creates one node with the task ID `"__whole__"`. Its file set comes from the plan's `crates_touched` frontmatter. Its time estimate comes from `estimated_minutes` in the same frontmatter.

```rust
const WHOLE_PLAN_TASK: &str = "__whole__";

// In from_plans():
let gid = GlobalTaskId {
    plan: plan.base.clone(),
    task: WHOLE_PLAN_TASK.to_string(),
};
```

If another plan depends on this atomic plan (via frontmatter `depends_on`), all of that plan's tasks get edges pointing to the `__whole__` node. The DAG doesn't distinguish between granular and atomic plans at scheduling time. Nodes are nodes. Edges are edges. A 12-task plan and a single-node plan sit in the same graph and the scheduler treats them identically.

This matters because real builds are mixed. Some plans justify a five-task breakdown with careful file partitioning. Others are "add a config value and write a test" -- one task, done in minutes. Both coexist.

## Wave scheduling

Plans group into waves based on their dependency structure. Wave 1 contains plans with no dependencies. Wave 2 contains plans that depend only on wave 1 plans. Wave 3 depends on wave 1 or wave 2. And so on.

```
Wave 1: [plan-01, plan-02, plan-04]    -- no dependencies
Wave 2: [plan-03]                       -- depends on plan-01
Wave 3: [plan-05, plan-06]             -- depend on plan-03
```

Waves set the coarse-grained execution order. The unified DAG provides the fine-grained parallelism within and across wave boundaries.

While wave N runs its gates (compile checks, test suites, reviews), Mori starts enrichment for wave N+1 -- assembling context briefs, generating workspace maps, pre-spawning agent processes. By the time wave N merges, wave N+1 agents already have context loaded and are ready to write code. This pipelining eliminates the dead time between waves.

Within each wave, the DAG resolves at task granularity. Two plans in the same wave with non-overlapping file sets have their tasks interleaved freely. The wave boundary only constrains when a plan's tasks can *begin* relative to their upstream dependencies, not when they run relative to their wave peers.

## Concrete example

Four plans. Mixed dependencies. File conflicts.

```
Plan 01 (3 tasks):
  T1 edits src/types.rs
  T2 edits src/handler.rs
  T3 edits tests/integration.rs

Plan 02 (2 tasks):
  T1 edits src/cache.rs
  T2 edits src/handler.rs        <-- CONFLICT with 01:T2

Plan 03 (2 tasks, depends on Plan 01):
  T1 edits src/api.rs
  T2 edits src/types.rs          <-- CONFLICT with 01:T1

Plan 04 (1 task, no dependencies):
  T1 edits src/config.rs
```

Here's how the scheduler walks through it.

**Tick 0.** `next_runnable(completed={}, in_flight={})`. Plans 01 and 02 and 04 have no unsatisfied dependencies. Plan 03 depends on Plan 01, so its tasks are blocked. Check file conflicts against in-flight (empty). Result: `01:T1`, `01:T2`, `01:T3`, `02:T1`, `04:T1` are all runnable. Not `02:T2` -- wait. Actually, in-flight is empty, so no file conflicts yet. All five start.

**Tick 1.** Five tasks are in-flight. `in_flight_files = {src/types.rs, src/handler.rs, tests/integration.rs, src/cache.rs, src/config.rs}`. `02:T2` wants `src/handler.rs`, which is held by `01:T2`. Blocked. Nothing else is runnable (Plan 03 tasks still waiting on Plan 01 completion).

**Tick 2.** `01:T3` completes. `04:T1` completes. In-flight files shrink: `{src/types.rs, src/handler.rs, src/cache.rs}`. `02:T2` still blocked (handler.rs). Plan 03 tasks still blocked (01 not fully complete). Nothing new starts.

**Tick 3.** `01:T2` completes. In-flight files: `{src/types.rs, src/cache.rs}`. `02:T2` is now runnable -- `src/handler.rs` is free. It starts.

**Tick 4.** `01:T1` completes. `02:T1` completes. All Plan 01 tasks are done. Plan 03's dependency is satisfied. In-flight files: `{src/handler.rs}` (from `02:T2`). `03:T1` wants `src/api.rs` -- no conflict, starts. `03:T2` wants `src/types.rs` -- no conflict (01:T1 already completed and released it), starts.

**Tick 5.** `02:T2`, `03:T1`, `03:T2` finish. DAG is empty. Build complete.

Without the unified DAG, this build runs Plan 01 (3 tasks), then Plan 02 (2 tasks), then Plan 03 (2 tasks), then Plan 04 (1 task). Sequential plan execution. With the DAG, tasks from all four plans interleave based on actual file contention. The wall clock time drops to roughly the length of the critical path instead of the sum of all plans.

The executor turns these scheduling decisions into `ExecutorAction` variants -- `SpawnTaskAgent` for individual tasks, `SpawnTaskAgentBatch` when multiple tasks from one plan can run together, `RunPlanGates` when all tasks for a plan complete. The event loop dispatches them. The DAG decides what's safe to run. Everything else follows from that.
