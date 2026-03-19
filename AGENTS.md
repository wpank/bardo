# Bardo — Agent Instructions

> Every agent session starts cold. The files on disk are the only memory. Write context files as if the reader has amnesia.

---

## Repository Layout

```
prd2/           Read-only product spec (334 files, ~230K lines). NEVER modify.
plans/          Implementation plans + cross-plan state.
  CONTEXT.md      Cross-plan registry: types, crate boundaries, decisions. Read first, append at end.
  context/
    last-completed.md   Summary from previous plan. Overwritten by each plan.
    ignored-tests.md    Ledger of #[ignore] tests. Check on entry, update on exit.
    workspace-map.md    Pre-generated file tree. Read instead of running find/ls.
    preflight-snapshot.md  Git log, compile status, test count. Generated before each plan.
    briefs/             Strategist execution briefs (one per plan).
    reviews/            Reviewer feedback and archived iterations.
agents/         Agent role prompts. Read yours at session start.
crates/         Rust workspace. All implementation code.
docs/           mdbook documentation. Updated by each plan after code is written.
tmp/            Scratch space. Not committed.
```

## Universal Rules (All Agents)

1. **No git operations.** No `git add`, `commit`, `checkout`, `merge`, `push`, `tag`, `stash`, `branch`. The shell handles all git.
2. **No prd2 modifications.** Those files are read-only input.
3. **Dependencies are cached.** Dependencies are pre-cached. If a crate is missing, note it in your output.
4. **Read workspace-map.md before using `find` or `ls` on `crates/`.** The map is pre-generated and saves tokens. Use `rg` for targeted symbol lookups only.
5. **Read preflight-snapshot.md for ambient state.** It tells you git history, compile status, and test count. Don't re-run those commands to discover information already there.
6. **Write for amnesia.** Every context file you produce must be self-contained. The next session has zero memory of yours.

## Git Branch Awareness

You are running on branch `codex/plan/NN-name`, created by `run-plans.sh` before your session. When you finish, the shell will commit your work, merge into the batch branch, tag it, and delete the plan branch. You never see any of this — just write files and stop.

---

# Implementer Protocol

When given a task like "Implement the plan at plans/NN-name.md", follow this protocol exactly.

## Phase 0: Orient (Read Before You Touch Anything)

Read these files in this order. Do not write a single line of code until you've read all of them:

1. **`plans/context/workspace-map.md`** — Every `.rs` file grouped by crate. This IS your `find` / `ls`. Do not run file discovery commands.
2. **`plans/context/preflight-snapshot.md`** — Git log, compile status, test baseline. This IS your `git log` and ambient state check.
3. **`plans/context/briefs/NN-brief.md`** — The Strategist's execution brief. It tells you what dependencies exist, what patterns to follow, what risks to watch. **This is your most important pre-read.** If the brief flags a missing type or a pattern mismatch, address it proactively.
4. **`plans/context/last-completed.md`** — What the previous plan built, what it left behind, what you should watch for.
5. **The plan file** — Read it cover to cover. The Quick Reference sections are your implementation spec.
6. **`plans/CONTEXT.md`** — Cross-plan state. Types, crate boundaries, decisions, deviations.
7. **`plans/context/ignored-tests.md`** — If your plan unblocks any tests, un-ignore them.
8. **Check for supervisor messages**: If `tmp/agent-messages.md` exists and is non-empty, read it now and treat its contents as high-priority steering context for this plan. Do NOT delete or modify this file — the shell manages it.

**After reading, verify:**
- Prerequisites listed in the plan are marked complete in CONTEXT.md.
- Imports listed in the plan exist in the workspace (the brief should confirm this, but double-check with `rg` if uncertain).
- No files your plan will create already exist (the brief's conflict scan covers this).

## Phase 1: Implement Each Unit of Work (In Order)

For each unit in the plan's Implementation Details:

### 1a. Read Sources
Read the prd2 files in the unit's Source Files section. The Quick Reference is authoritative — if prd2 contradicts it, follow the Quick Reference. But read the prd2 files for full context, edge cases, and design rationale.

### 1b. Read Existing Code
Before writing, understand what exists:
- If the crate exists, read its `src/lib.rs` and key modules. Match their patterns.
- Run `rg "pub struct TypeName"` for types your plan imports. Verify signatures.
- If modifying existing files, read them fully before changing them.

### 1c. Write Code
- Create or modify files per the plan's Crate Location section.
- Implement types and traits from Quick Reference.
- **Match existing workspace patterns.** The brief's Pattern Alignment section tells you what those are. If the brief says "error types follow `thiserror` enums in `golem-core/src/error.rs`", match that exactly.
- Use `pub(crate)` by default. Only `pub` for cross-crate API specified in the plan's Exports section.
- Every public item gets a doc comment.

### 1d. Write Tests
- Unit tests: `#[cfg(test)]` module in the same file.
- Integration tests: `tests/` directory in the crate.
- If a test depends on a future plan, mark it `#[ignore]` with `// TODO(plan-NN): requires <system>`.
- Add ignored tests to `plans/context/ignored-tests.md`.
- **Actually run your tests.** Do not just write them and assume they pass.

### 1e. Compilation Gate
Run `cargo check --workspace` after each unit. **Do not proceed with a broken workspace.** Fix the error, even if it means adjusting your code to match what actually exists rather than what the plan assumed.

### 1f. Test Gate
Run `cargo test -p <your-crate>`. Fix failures in your code. If failures are in pre-existing tests you didn't modify, note them but continue.

### 1g. Write Documentation
Write mdbook pages per the plan's Gitbook Documentation section. Place in `docs/`. **Document what you ACTUALLY built, not what the plan intended.** If you deviated, the docs reflect reality.

### 1h. Checkpoint
Call `update_plan` after each unit with a brief status note. This is your safety net against context compaction.

## Phase 2: Completion

After all units are done:

### 2a. Append to `plans/CONTEXT.md`

```markdown
## Plan NN: [Name] — Completed [YYYY-MM-DD]

### Types Defined
- `TypeName` — `crate::module::path` — one-line description

### Public Traits
- `TraitName` — `crate::module::path` — `fn method(&self, ...) -> ReturnType`

### Deviations
- [What changed vs. plan, and why]

### Unresolved Issues
- [Ambiguities, missing pieces, things future plans should check]

### Status
- Branch: codex/plan/NN-name
- cargo check: pass
- cargo test: N pass, N fail, N ignored
```

### 2b. Overwrite `plans/context/last-completed.md`

```markdown
# Last Completed: Plan NN — [Name]

## What Changed
- Created/modified crates: [list]
- Key files: [list important new/changed files]

## Types & Traits Defined
- `TypeName` — `crate::path` — purpose
- `TraitName` — `crate::path` — key methods

## Deviations From Plan
- [What differed and why — the next plan NEEDS to know this]

## Issues For Next Plan
- [Types that might need adjustment]
- [Missing pieces or placeholders left behind]
- [Anything surprising about the codebase state]

## Test Status
- cargo check: pass
- cargo test: N pass, N fail, N ignored
- New #[ignore] tests: [list with plan numbers they await]
- Un-ignored tests: [list]
```

### 2c. Update `plans/context/ignored-tests.md`
- Add entries for new `#[ignore]` tests: `test_name | crate | reason | unblock_plan`
- Remove entries for tests you un-ignored.

### 2d. Final Verification
Run `cargo test --workspace`. Record pass/fail/ignore counts in both CONTEXT.md and last-completed.md. This is the number reviewers will check against.

---

# Fix Cycle Protocol (Iteration 2+)

When your task prompt contains `FIX CYCLE — ITERATION N`, you are NOT re-implementing the plan. You are surgically fixing blocking issues raised by reviewers.

## Scope Rules

1. **Read the reviews first.** Open `plans/context/reviews/NN-arch-review.md` and `NN-spec-review.md`. Items marked `- [ ]` (unchecked) are unresolved.
2. **Read the updated brief.** Open `plans/context/briefs/NN-brief.md` section 6. It summarizes each unresolved issue with a concrete remediation action.
3. **Fix ONLY blocking issues.** Each fix should be traceable to a specific `[B-N]` or `[S-N]` issue ID.
4. **Do not refactor unrelated code.** Do not "improve" things that weren't flagged. Do not reorganize modules. Do not add features.
5. **Do not re-run the entire plan.** You are patching, not rebuilding.
6. **After fixes:** Update the completion report in CONTEXT.md (amend the existing report, don't create a duplicate). Update last-completed.md. Run `cargo check --workspace` and `cargo test -p <crate>`.

## What Counts as "Fixed"

A blocking issue is fixed when:
- The specific condition described in the review no longer exists.
- `cargo check --workspace` passes.
- If the issue was a missing test, the test now exists and passes (or is `#[ignore]` with justification).
- If the issue was a missing type/trait, it now exists with the correct signature.

---

# Compaction Protocol

If your context fills up mid-plan (you've processed many units and have extensive tool output):

1. **Write in-progress state to `plans/CONTEXT.md`** under `## Plan NN — In Progress`:
   - Units completed (by name)
   - Types defined so far (name, crate path)
   - File you were working on
   - Next unit to implement
   - Any issues encountered
2. **Call `update_plan`** with a progress note.
3. **After compaction**, re-read `plans/CONTEXT.md` to find your `In Progress` section. Continue from there.
4. **When complete**, replace the `In Progress` heading with the final `Completed` report.

---

# Coding Conventions

- **Rust edition**: 2024
- **Error handling**: `thiserror` typed enums for library crates, `anyhow` with `.context()` for binaries only.
- **Async**: `tokio` (multi-thread). Blocking I/O via `spawn_blocking`.
- **Serialization**: `serde` derive. `#[serde(rename_all = "camelCase")]` for JSON APIs.
- **Ethereum**: `alloy` exclusively (not ethers-rs). `sol!` macro for ABI generation.
- **Logging**: `tracing` (not `log`).
- **Testing**: `#[test]` + `tokio::test` for async. `proptest` for property tests. Both `#[cfg(test)]` inline and `tests/` integration.
- **Naming**: `snake_case` files/functions, `CamelCase` types, `SCREAMING_SNAKE` constants.
- **Visibility**: `pub(crate)` by default. `pub` only for documented cross-crate API.
- **Generics**: `impl Trait` over `dyn Trait` except at module boundaries.
- **No `unwrap()` in library code.** Use `?` or `expect("reason")` where panic is intentional and documented.
- **No `unsafe`.** `unsafe_code = "deny"` in workspace lints.
- **Workspace deps**: All versions in root `[workspace.dependencies]`. Crates use `workspace = true`.
- **Layer rule**: No upward dependencies. Layer N depends only on layers < N. `golem-core` has zero workspace deps.
- **Config**: `golem.toml` (TOML). Env vars override with `BARDO_*` / `GOLEM_*` prefixes. Secrets from env vars or keystore only.

---

# Failure Recovery

1. **Spec contradiction**: Quick Reference is authoritative. Follow it. Note the contradiction in your completion report.
2. **Missing type from earlier plan**: Check CONTEXT.md → search with `rg "struct TypeName"` → if truly missing, create a minimal placeholder:
   ```rust
   /// Placeholder — expected from Plan NN. Replace when available.
   #[derive(Debug, Clone, Default)]
   pub struct TypeName;
   ```
   Note in completion report AND last-completed.md.
3. **Test can't pass yet**: `#[ignore]` with `// TODO(plan-NN): requires <system>`. Add to ignored-tests.md.
4. **Ambiguity**: Simplest choice. Fewer types. Document the decision.
5. **Missing Cargo dependency**: Add to workspace `Cargo.toml`. Note in completion report. Current plan may not be able to build with it (no internet for fetch).
6. **Compilation error in other crate**: Do not modify other crates' logic. You MAY fix visibility (`pub(crate)` → `pub`) or add missing derives if the plan's Exports section says it should be different. Note ALL such cross-crate changes.
7. **Context filling up**: Compaction Protocol (above). Checkpoint before you lose state.
8. **Plan-specific recovery**: Check the plan's own Failure Recovery section.

---

# Multi-Agent Orchestration

## Pipeline

```
┌──────────┐   ┌──────────────┐   ┌─────────────┐   ┌──────────────┐
│Strategist│──▶│ Implementer  │──▶│Arch Reviewer │──▶│ Spec Auditor │
│  (brief) │   │ (code+tests) │   │  (quality)   │   │  (fidelity)  │
└──────────┘   └──────────────┘   └──────┬───────┘   └──────┬───────┘
                                         │                    │
                                    Both APPROVE? ────────────┘
                                     │         │
                                    YES        NO
                                     │         │
                                  commit    archive reviews
                                             ↓
                                    ┌──────────┐
                                    │Strategist│ (integrate feedback)
                                    │  re-run  │
                                    └────┬─────┘
                                         ↓
                                    Implementer fix cycle
                                         ↓
                                    Both reviewers re-run
                                         ↓
                                    (repeat up to MAX_REVIEW_ITERATIONS)
```

## Agent Roles

| Agent | Runs | Reads | Writes | Checks |
|-------|------|-------|--------|--------|
| **Strategist** | Before implementation | Plan, CONTEXT.md, workspace-map, preflight, prior reviews | `briefs/NN-brief.md` | Dependencies exist, patterns match, risks identified, review feedback integrated |
| **Implementer** | After strategist | Brief, plan, CONTEXT.md, workspace-map (iter 2+: reviews) | Code, tests, docs, CONTEXT.md, last-completed.md | Code compiles, tests pass, docs written |
| **Architect** | After implementation | Diff, full modified files, brief, CONTEXT.md | `reviews/NN-arch-review.md` | Compilation, clippy, layering, API surface, patterns, correctness |
| **Spec Auditor** | After implementation | Diff, prd2 sources, plan, CONTEXT.md | `reviews/NN-spec-review.md` | Type contracts, behavioral completeness, missing pieces, deviations, prd2 intent |

## Non-Overlapping Concerns

To prevent redundant checks and conflicting feedback:

- **Architect** owns: compilation, clippy, `pub` visibility, error handling style, module structure, test organization, race conditions, panicking paths, doc comments existence.
- **Spec Auditor** owns: type field correctness, formula accuracy, behavioral rule coverage, missing implementations, export contracts, prd2 alignment, deviation documentation.
- **Neither reviews the other's domain.** If the Spec Auditor notices a code quality issue, it notes it under "Notes" (non-blocking) and trusts the Architect to catch it. Vice versa.

## Iteration Protocol

1. Both APPROVE → compilation gate → test gate → git commit → next plan.
2. Either REVISE → current reviews archived as `NN-{role}-iterN.md` → strategist re-runs with history → implementer fix cycle → both reviewers re-run.
3. Maximum `MAX_REVIEW_ITERATIONS` (default 3). If exceeded → halt report written → exit 1 → human intervention required.
4. Only **blocking issues** (`[B-N]`, `[S-N]`) trigger a REVISE verdict. Recommendations are noted but never block.
5. `--no-review` skips the entire orchestration loop. Use for Plan 01 or trivial plans.
6. `--max-iterations N` overrides the default cap.

## Severity Calibration

An issue is **blocking** only if at least one of these is true:
- `cargo check` or `cargo test` fails because of it.
- A downstream plan's import will break (wrong type signature, missing export, wrong visibility).
- A type, trait, or formula deviates from the plan's Quick Reference without documentation.
- A test specified in the plan's Verification section was not written.
- A doc page specified in the plan was not created.

An issue is a **recommendation** if:
- It's a style preference (naming, comment wording, module organization) that doesn't break anything.
- It's an optimization that isn't required for correctness.
- It's a suggestion for a future plan, not this one.

## Context File Lifecycle

```
plans/context/
├── workspace-map.md              # Regenerated by shell before each plan
├── preflight-snapshot.md         # Regenerated by shell before each plan
├── last-completed.md             # Overwritten by implementer at plan completion
├── ignored-tests.md              # Append/remove by implementer
├── briefs/
│   └── NN-brief.md               # Written by strategist (overwritten each iteration)
└── reviews/
    ├── NN-arch-review.md          # Current architect review (overwritten each iteration)
    ├── NN-arch-review-iterN.md    # Archived before overwrite
    ├── NN-spec-review.md          # Current spec review (overwritten each iteration)
    ├── NN-spec-review-iterN.md    # Archived before overwrite
    └── NN-halt-report.md          # Written only if max iterations exceeded
```

---

# Documentation Policy

Write `docs/src/` pages as external-facing GitBook documentation for users who don't know the codebase.

## Structure (per page)

1. **What It Is** — one paragraph, user-facing, no plan numbers
2. **Features** — bullet list of capabilities from a user's perspective
3. **Getting Started** — prerequisites, how to run it
4. **Configuration** — flags and environment variables
5. **API** — RPC methods or public interface (if applicable)
6. **Architecture** — high-level diagram or prose only

## Rules

- DO: explain what it does, how to use it, what to configure
- DO NOT: mention plan numbers, implementation status, deviations, internal file paths
- DO NOT: write "Plan 03 is partially complete" or "Current Deviations" sections
- DO NOT: list internal source file paths like `apps/mirage-rs/src/fork.rs`
- Replace "What Exists" sections with "Features"
- Tone: GitHub README / GitBook style. Confident, present tense.