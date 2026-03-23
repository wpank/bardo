# Mori parallel execution

## Why isolation matters

Two agents writing code in the same repository is a disaster. Agent A edits `lib.rs` while Agent B edits the same file. One overwrites the other. The git index gets corrupted. Partial writes produce files that are half one agent's work, half the other's. Even if they touch different files, they compete for the git lock, and `cargo build` fights over the `target/` directory.

Every multi-agent system that actually works needs physical separation. The question is how to achieve it.

**Separate clones** waste disk. A Rust workspace with 50 crates and a full `target/` directory can eat 10+ GB. Clone it five times for five parallel agents and you're burning 50 GB, plus the object stores are independent, so git operations duplicate work.

**Branches alone** don't help. Git branches share one working directory. You can't have two branches checked out at the same time. Switching branches to merge would destroy every other agent's working state.

**Git worktrees** are the answer. A worktree is a second (or third, or tenth) checkout of the same repository. Each worktree has its own working directory and its own branch, but they all share the same object store. Disk usage is a fraction of a full clone. Git operations stay fast because there's one pack file, not ten copies.

Mori uses git worktrees as its isolation primitive.

## Git worktree isolation

When a plan starts execution, the `WorktreeManager` creates a worktree for it. The worktree lives under `.worktrees/plan-{plan-base}/` and checks out its own branch from the batch branch.

```
.worktrees/
  plan-09-chain-layer/    # full checkout, branch codex/plan/09-chain-layer
  plan-12-grimoire/       # full checkout, branch codex/plan/12-grimoire
  plan-15-mirage/         # full checkout, branch codex/plan/15-mirage
```

After `git worktree add`, the directory contains tracked files from the branch. But agents also need workspace-level config files that might be gitignored or stale. Mori copies these from the main repo root into each worktree:

- **Files:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `nextest.toml`, `bardo-ctl.sh`
- **Directories:** `.cargo/`, `plans/`, `prd2/`, `agents/`, `tests/`

The `plans/` and `prd2/` directories get re-synced on every worktree creation or reuse, since they contain briefs, extracts, and task definitions that change between runs. Other directories copy once and stay put.

Each worktree also gets a `.cursor/cli.json` (agent permissions), `.mori/mcp-config.json` (context server pointed at the worktree root), and `context/in/` and `context/out/` directories for the inter-agent context pipeline.

Agents read and write freely within their worktree. They can run `cargo build`, `cargo test`, edit any file, create new files. Nothing they do affects any other worktree or the main checkout.

### Shared compilation cache

Each worktree has its own `target/` directory, which means Rust recompiles dependencies from scratch in each one. That's slow.

Mori fixes this with sccache. Every agent process spawns with `CARGO_INCREMENTAL=0` and `SCCACHE_BASEDIRS` set to the worktree path. Disabling incremental compilation is counterintuitive, but necessary: incremental builds mark outputs as non-cacheable by sccache. With incremental off, sccache caches every compilation unit by content hash. When Agent B compiles a crate that Agent A already compiled (same source, same flags), sccache returns the cached artifact in milliseconds instead of recompiling for 30 seconds.

The `SCCACHE_BASEDIRS` variable normalizes absolute paths so that `/repo/.worktrees/plan-09/src/lib.rs` and `/repo/.worktrees/plan-12/src/lib.rs` hash to the same key when the source content matches. Without this, sccache treats each worktree's paths as distinct and the cache misses constantly.

## Branch strategy

Mori uses three tiers of branches:

**Plan branches** (`codex/plan/{plan-name}`). Each plan gets one. The agent commits directly to this branch inside its worktree. All implementation, gate fixes, and review responses happen here.

**Batch branch** (`codex/batch/{date}`). When a plan passes all gates and reviews, its branch merges into the batch branch. This is the integration point where all plans come together.

**Target branch** (usually `main`). When the batch is complete, the batch branch merges into the target with `--no-ff`, producing a single merge commit that represents the entire batch of work.

Plan-to-batch merges also use `--no-ff`. This preserves plan identity in the git history: you can see exactly which commits belong to which plan, even after the merge.

### Merge ordering

Plans merge in dependency order. If Plan B declares a dependency on Plan A (in its frontmatter `depends_on` field), Plan A merges first. The `merge_queue` in the `ParallelExecutor` holds plans that have passed gates and reviews but are waiting for their turn. Only one plan merges at a time. This serialization prevents race conditions on the batch branch ref and ensures each plan merges into a batch that already contains its dependencies.

The merge itself happens inside the plan's worktree, not the main checkout:

1. Merge the batch branch *into* the plan worktree (`git merge --no-edit {batch_branch}`)
2. Read the resulting HEAD commit
3. Fast-forward the batch branch ref to that commit via `git update-ref`
4. If the main checkout has the batch branch checked out, sync it with `reset --hard`

This approach avoids `git checkout` in the main repo, which would disrupt the user's working directory and can fail when the branch is already checked out elsewhere.

After a successful merge, Mori tags the commit (`plan/{plan-name}`) with a timestamp for traceability.

## Merge conflict resolution

When a plan branch can't merge cleanly into the batch branch, the merge aborts and Mori captures the list of conflicted files (UU, AA, DD in `git status --porcelain`). The conductor then spawns a `MergeResolver` agent.

The resolver gets a prompt that includes:

- The conflicted file paths
- The plan's original intent (from the plan document)
- The current batch state

The resolver works in the plan's worktree, resolves the conflicts, commits, and signals completion. If automatic resolution fails or the resolver can't produce a clean merge, the conductor escalates. This may mean re-running the plan against an updated batch baseline, or marking the plan as failed for human intervention.

## Warm agent pools

Spawning an agent takes 5-15 seconds. The process starts, loads its system prompt, initializes the model connection, and becomes ready for `turn_start`. During a typical plan lifecycle, this happens at every phase transition: spawn an implementer, wait for it to finish, spawn a reviewer, wait for it to finish. That's 10-30 seconds of dead time per plan just waiting for processes to start.

Mori eliminates the reviewer startup delay with warm spawning. When the implementer finishes and gates begin running, Mori pre-spawns a reviewer in the background. The `MultiAgentPool` calls `pre_spawn_warm()`, which creates the agent connection and initializes it, but does not call `turn_start`. The agent sits idle in the `warm_pool` HashMap, consuming no inference tokens.

When gates pass, the executor calls `promote_warm()`. The warm connection moves from `warm_pool` to `connections`, and `turn_start` fires immediately. Zero wait. The reviewer begins working the instant gates clear.

When gates fail, the warm reviewer is evicted: `evict_warm()` kills the process. No wasted inference, no orphan processes. The implementer gets another iteration, and Mori pre-spawns a fresh reviewer for the next gate attempt.

The warm pool tracks agents by `(role, instance)` tuple, so multiple plans can each have their own warm reviewer pre-spawned simultaneously.

## Batch task spawning

Some plans have many small tasks: rename a constant across 20 files, add a field to 15 structs, update 30 test fixtures. Spawning one agent per task would mean 15 spawn cycles at 5-15 seconds each. The startup cost dwarfs the actual work.

Mori detects this pattern and groups tasks. The `schedule_next()` method in the executor collects runnable tasks for each plan and emits a `SpawnTaskAgentBatch` action instead of individual `SpawnTaskAgent` actions. One agent receives all the tasks in a single prompt and works through them sequentially. The instance ID reflects the grouping: `implementer:{plan}:g0`, `implementer:{plan}:g1` for file-conflict groups that need separate agents.

When tasks touch different files, they can all go to one agent. When tasks have overlapping file dependencies, Mori splits them into conflict groups and assigns one agent per group. Each group still gets a single spawn for all its tasks, amortizing the 5-15 second startup across the group.

## Merge checkpoints

A crash during a merge leaves the repository in an undefined state. The batch branch ref might have been updated but the plan tag might not exist. Or the merge might have completed but the executor state wasn't persisted. Or worse, the merge is half-applied with a MERGE_HEAD file hanging around.

Mori writes a `MergeCheckpoint` to `task-state.json` before starting any merge:

```json
{
  "merge_in_progress": {
    "plan": "09-chain-layer",
    "worktree_head": "a3f7c2d1",
    "batch_ref": "b8e4f912",
    "timestamp": "2026-03-23T14:22:31Z"
  }
}
```

The checkpoint records which plan is merging, the worktree's HEAD commit at merge start, and the batch branch ref before the merge. This write happens atomically (write to `.tmp`, then rename).

On restart, Mori reads `task-state.json` and checks for a stale checkpoint. If one exists:

1. Look for `MERGE_HEAD` in the worktree's gitdir (handles linked worktrees where `.git` is a file pointing elsewhere)
2. If a merge is still in progress, abort it with `git merge --abort`
3. If a rebase is in progress, abort that too
4. Clear the checkpoint and re-enter the merge queue

The `validate_and_repair()` method on `WorktreeManager` runs this check across all worktrees on startup, pruning orphaned refs and resetting conflicted state. Events flow to `.mori/events.jsonl` so every merge attempt, success, failure, and crash recovery is traceable after the fact.

No half-merged states survive a restart.
