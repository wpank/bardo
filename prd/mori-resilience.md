# Mori resilience

## Failure is normal

A batch build runs 20+ plans across parallel agents. Each agent writes code in its own worktree, runs compile gates, gets reviewed, and merges. That's a lot of moving parts with a lot of failure surfaces.

Agents produce code that doesn't compile. Tests break because the agent misread a type signature. The process gets killed mid-merge when your laptop sleeps. The network drops during an API call. A model returns empty output three times in a row. The Anthropic API rate-limits you at the worst possible moment.

Mori does not treat any of these as exceptional. Every failure mode has a defined recovery path. The system assumes things will go wrong and builds its control flow around that assumption. When something breaks, the question isn't whether recovery is possible -- it's which recovery mechanism activates.


## Persistent state

Before each state transition, Mori writes a `TaskStateFile` to `.mori/state/task-state.json`. This file captures everything the executor needs to resume:

- **Completed tasks** in `plan:task` format (e.g., `09-chain-layer:T1`)
- **In-flight tasks** mapped to their agent instance IDs
- **Per-plan iteration counts** so retry logic picks up at the right attempt number
- **Active worktree paths** per plan, so crash recovery can find or clean up worktrees
- **Merge queue order** preserving the dependency-sorted merge sequence
- **Merge-in-progress checkpoint** (plan name, worktree HEAD, batch branch ref, timestamp)
- **Review feedback archives** so re-implementation attempts carry forward reviewer comments
- **Cost data** accumulated per plan across sessions

Writes are atomic: content goes to a `.tmp` file first, then gets renamed into place. A kill signal mid-write leaves the previous state intact rather than producing a corrupted file. If the state file is corrupted anyway (disk issue, OOM kill during rename on some filesystems), the persistence manager renames it to `task-state.json.corrupt`, logs a warning, and falls back to reconstructing completed tasks from the append-only event log.

That event log is `.mori/state/events.jsonl`. Every state transition appends a line: task starts, task completions, gate results, merges. It's the audit trail. If `task-state.json` disappears entirely, Mori scans the event log for `task_done` events and rebuilds the completed task set. You lose in-flight task assignments (those agents are dead anyway after a crash) but you don't re-do finished work.

On restart, the executor reads the state file, marks completed tasks as done, and calls `schedule_next()`. Tasks that were in-flight at crash time get rescheduled to fresh agents. The process picks up where it stopped.


## Conductor interventions

The conductor is a meta-orchestrator that ticks on every event loop frame. It watches a set of signals and fires interventions when things look wrong.

**Silence detection.** If an agent produces no output for 300 seconds (600 for Claude backend, which is slower), the conductor sends a nudge message: "You've been silent for too long. Summarize what you've done so far and continue working." This often unsticks agents that entered a reasoning loop without producing tool calls.

**Ghost turn detection.** If an agent completes a turn in under 5 seconds with no output, that's a ghost turn -- the model returned immediately without doing anything. The conductor restarts the agent outright, no nudge. Ghost turns are a known failure mode with API-based agents and there's no point trying to reason with a model that already exited.

**Compile failure escalation.** Graduated response to repeated compilation failures on the same plan:

- 3 failures: nudge with the last 10 lines of compiler output and explicit instructions to change approach
- 5 failures: kill the implementer agent and spawn a fresh one
- 7 failures: force-advance past this plan

**Review loop detection.** If reviewers issue 3+ consecutive REVISE verdicts while compile and test gates are passing, the conductor skips reviews entirely. The code works. The reviewers are bikeshedding. Move on.

**Iteration loop detection.** If a plan reaches iteration 6 and is still cycling between strategist and implementer, the conductor force-advances. The plan isn't converging and spending more tokens won't fix it.

**Test failure budget.** When compile gates pass but some tests fail, the conductor checks the pass rate against a configurable threshold (default 70%). If 8 out of 10 tests pass, the remaining 2 are likely test-spec mismatches or unrealistic assertions, not real bugs. The conductor force-advances and logs the failures as deferred, with their error snippets, for manual triage later.

Every intervention has a 120-second cooldown per plan to prevent the conductor from firing the same intervention repeatedly on consecutive ticks.


## Model escalation

Not implemented as a separate escalation ladder in the current codebase -- instead, it's built into the retry flow. When a plan fails its gates, what happens next depends on the mode.

In express mode, gate failure triggers the AutoFixer agent (see next section). If auto-fix fails after `max_auto_fix_attempts` (default 3), the plan enters full re-implementation with a fresh agent that receives the structured error output from all previous attempts.

In standard mode, gate failure feeds structured error output back to the implementer on the next iteration. The implementer's prompt includes the exact compiler errors, the gate verdicts, and any reviewer feedback from prior iterations. Each iteration carries more context about what went wrong.

The conductor's compile-failure escalation (nudge at 3, restart at 5, abort at 7) acts as the de facto model escalation. A restart spawns a cold agent with no prior conversation context but with the same structured error brief. After 10 consecutive spawn failures on a plan (the agent exits immediately with no output every time), the executor marks the plan as permanently failed.


## Error classification and auto-fix

Mori parses `cargo check --message-format=json` output into structured error classes. The `autofix` module classifies each compiler error by its rustc error code:

- **E0432/E0433** (unresolved import): `ImportNotFound`. The module path and item name are extracted. Auto-fixable -- the right `use` statement can be generated from workspace analysis.
- **E0063** (missing struct field): `MissingField`. Struct name and field extracted. Auto-fixable with struct definition context.
- **E0308** (type mismatch): `TypeMismatch`. Expected and found types extracted from compiler children diagnostics. Often auto-fixable with conversion context.
- **E0277** (trait not implemented): `TraitNotImplemented`. Type and trait names extracted. Sometimes auto-fixable, sometimes architectural.
- Everything else: `Other`. Needs a real agent.

The `is_simple()` method identifies errors that can be fixed cheaply: `ImportNotFound` and `MissingField` qualify. These go to an AutoFixer agent -- a lightweight Haiku-tier agent that costs a fraction of a cent per fix. The AutoFixer also gets `rustc`'s own suggested replacements when available, extracted from the diagnostic spans.

This separation matters because a missing import costs $0.01 to fix with Haiku. A full re-implementation cycle with an Opus-tier agent costs $2+. When 6 out of 8 errors are missing imports and one missing struct field, Mori fixes 7 of them for $0.07 and only escalates the remaining architectural error to a full implementer iteration.


## Merge recovery

Merging a plan's worktree into the batch branch is the most dangerous operation in a batch run. If Mori crashes between the merge commit and the branch pointer update, the repo could be in an inconsistent state.

Before starting any merge, Mori writes a `MergeCheckpoint` to the task state file:

```rust
pub struct MergeCheckpoint {
    pub plan: String,           // which plan is merging
    pub worktree_head: String,  // the commit SHA being merged in
    pub batch_ref: String,      // the batch branch ref before merge
    pub timestamp: String,      // when the merge started
}
```

On restart, if `merge_in_progress` is `Some(checkpoint)`, the executor checks: did the merge commit land on the batch branch? If the worktree HEAD is an ancestor of the current batch branch tip, the merge completed -- clear the checkpoint and continue. If not, roll back to `batch_ref` and retry the merge from scratch. The checkpoint gives Mori the information it needs to make this decision without guessing.

The `merge_in_progress` bool on the executor prevents concurrent merges. Only one plan merges at a time, and plans merge in dependency order from the merge queue. A crash during merge blocks the queue until recovery runs, but it never corrupts the batch branch.


## What "failed" means

When a plan exhausts all recovery mechanisms -- auto-fix attempts, conductor restarts, iteration limits, wall-clock timeout (default 45 minutes) -- the executor sets its phase to `PlanPhase::Failed(reason)`.

This does not stop the build. The executor removes the plan from the merge queue, cancels any in-flight tasks for it, and calls `schedule_next()` to dispatch work for other plans. Parallel plans that don't depend on the failed plan continue without interruption.

Failed plans keep their full history: every event in `events.jsonl`, every gate result, every compiler error, every reviewer verdict. The deferred-failures log captures structured records with the error snippets, the iteration number, and the reason the failure was deferred (non-blocking test, budget threshold met, force-advanced by conductor). An operator can read this log, understand exactly what went wrong, and decide whether to retry manually.

The system also persists per-plan cost data across sessions. If plan 12 cost $5 in the first run and $3 in a retry, the cost summary shows $8 total. This matters for budgeting: you can see which plans are expensive failures versus cheap successes before deciding whether another attempt is worth the spend.
