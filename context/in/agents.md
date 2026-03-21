# Bardo — Agent Instructions

> Every agent session starts cold. Files on disk are the only memory. Write context files as if the reader has amnesia.

---

## Repository Layout

```
prd2/           Read-only product spec. NEVER modify.
plans/          Implementation plans + cross-plan state.
  CONTEXT.md      Cross-plan registry: types, boundaries, decisions.
  context/
    tasks/{num}-tasks.toml              Task checklists with acceptance criteria
    briefs/{num}-brief.md               Execution brief per plan
    workspace-map.md                    Pre-generated file tree (use instead of find/ls)
    preflight-snapshot.md               Git log, compile status, test baseline
    prd2-extracts/{num}-prd2.md         Pre-extracted PRD2 sections
    reviews/                            Reviewer feedback and archived iterations
crates/         Rust workspace. All implementation code.
docs/           mdbook documentation.
tmp/            Scratch space. Not committed.
```

## Your Role

Your role (Implementer, Architect, Auditor, Scribe, etc.) is defined in the **turn message**. Follow it exactly.

---

## Reviewer Contract (Read Before You Start)

**The reviewer will check all of these.** Fail any one → REVISE → iteration cycle.

1. `cargo check --workspace` — **zero errors**. If this fails, do not check any other criterion — fix compilation first.
2. `cargo test -p <crates>` — all new tests pass
3. Every export in the plan's Exports table exists in source
4. Every Cargo.toml entry in "Cargo Dependencies" matches exactly
5. Every config file the plan specifies matches the plan's exact content
6. Every Quick Reference struct/fn/enum matches verbatim
7. Every INV-NNN has a `test_fn` that exists and passes
8. Every Gitbook Documentation page in the plan exists in `docs/src/`

If you pass all eight before finishing, you will not be asked to revise.

---

## Universal Rules (All Agents)

1. **No git operations** for Implementer, Strategist, reviewers, Scribe, and other plan-implementation roles: no `git add`, `commit`, `checkout`, `merge`, `push`, `tag`, `stash`, `branch` — the shell/orchestrator handles git. **Exception:** the **Merge Resolver** (and any role whose turn message explicitly says it may use git) may run git commands only as instructed in that prompt to resolve merge conflicts and complete the merge.
2. **No prd2 modifications.** Those files are read-only input.
3. **Dependencies are cached.** Dependencies are pre-cached. If a crate is missing, note it in your output.
4. **Read workspace-map.md before using `find` or `ls` on `crates/`.** The map is pre-generated and saves tokens. Use `rg` for targeted symbol lookups only.
5. **Read preflight-snapshot.md for ambient state.** It tells you git history, compile status, and test count. Don't re-run those commands to discover information already there.
6. **Write for amnesia.** Every context file you produce must be self-contained. The next session has zero memory of yours.

## Authority Chain

When context files conflict, this is the precedence order. Higher items win:

1. **Turn message** — your role instructions. Always.
2. **Quick Reference** (`#### Quick Reference` in each plan unit) — authoritative for Rust types, struct shapes, field names, function signatures, enum variants. If a plan QR says `pub fn foo(&self) -> Bar`, implement exactly that.
3. **`plans/context/tasks/NN-tasks.toml`** — authoritative for what to implement, in what order, and what acceptance criteria to verify.
4. **Plan file** (`plans/NN-*.md`) prose — authoritative for intent, rationale, and implementation notes. Read it for context; follow QR and TOML for specifics.
5. **PRD2 spec** — authoritative for business logic, domain formulas, threshold values, and behavioral semantics NOT specified in the QR or plan prose. If the plan QR and PRD2 disagree on a field name, follow QR. If they disagree on a formula constant, follow PRD2.
6. **Brief, decomposition, bundles** — orientation documents. They summarize the above; they do not override them. When a brief or decomposition contradicts the plan or TOML, the plan/TOML wins.

Any deviation from the plan's Quick Reference must be documented in your completion report.


# Implementer Protocol

When given a task like "Implement the plan at plans/NN-name.md", follow this protocol exactly.

## Phase 0: Orient (Read Before You Touch Anything)

**Mandatory — read these FIRST, in this order:**

1. **`tmp/agent-messages.md`** — If non-empty, read FIRST. Conductor steering that supersedes other instructions.
2. **`plans/context/briefs/NN-brief.md`** — Single entry point. Contains: Quick Reference signatures, task order, prerequisites, risk flags, INV tests.
3. **`plans/context/tasks/NN-tasks.toml`** — Authoritative for what to implement and acceptance criteria.
4. The **plan file** (`plans/NN-*.md`) — Full spec. Quick Reference sections are your implementation spec.
5. **`plans/CONTEXT.md`** — Cross-plan state: existing types, deviations, decisions.

Also use (when needed):
- `plans/context/workspace-map.md` — Use instead of `find`/`ls`.
- `plans/context/preflight-snapshot.md` — Use instead of running `git log`/`cargo check`.
- `plans/context/prd2-extracts/NN-prd2.md` — Pre-extracted PRD2 sections.

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

#### Config Literal Check (after each unit)
If this unit writes or modifies any non-Rust file (Cargo.toml, .cargo/config.toml, rustfmt.toml, justfile, build.rs, etc.):
- Re-read the written file immediately after writing it
- Compare line-by-line against the plan's spec for that file
- Do not rely on memory — open both and diff them

Common literal misses:
- `optional = true` omitted from a Cargo dependency
- `rustc-wrapper` or other dev keys left uncommented when the plan says to comment them
- Missing CLI flags in justfile recipes (e.g. `--follow`, `--rpc-url`)
- Missing fields in rustfmt.toml (e.g. `imports_granularity`, `group_imports`)

### 1d. Write Tests
- Unit tests: `#[cfg(test)]` module in the same file.
- Integration tests: `tests/` directory in the crate.
- If a test depends on a future plan, mark it `#[ignore]` with `// TODO(plan-NN): requires <system>`.
- Add ignored tests to `plans/context/ignored-tests.md`.
- **Actually run your tests.** Do not just write them and assume they pass.
- **Check `plans/context/verify-chains/NN-verify.sh`** — each `INV-NNN` block in the plan has an expected test function name listed there. Implement exactly those test functions. The Auditor runs this script; if your tests aren't there or fail, you will get `[S-N]` blocking issues.

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

Then verify:

9e. **Cargo literal check:** For each entry in the plan's "Cargo Dependencies" section, read the actual Cargo.toml and verify: package name, `optional`, `features`, `workspace = true`. Write `cargo_toml_verified = true/false` in selfcheck.toml.

9f. **Config file literal check:** For each non-Rust file the plan specifies, read the file and verify content matches plan spec. Write `config_files_verified = true/false` in selfcheck.toml.

9g. **Quick Reference literal check:** For each type/function/constant in plan's Quick Reference, verify field names, signatures, and enum variants match verbatim. Write `qr_verified = true/false` in selfcheck.toml.

### 2e. Internal Iteration — Do Not Declare Done Until Clean

You are the first reviewer of your own work. Do not hand off to the reviewer until you can answer YES to every check below. If any check fails, fix it and repeat the loop.

**The iteration loop:**

1. Run `cargo check --workspace` — must be zero errors. If fails: fix and restart from step 1.
2. Run `cargo test -p <crate>` for each affected crate — all new tests pass. If fails: fix and restart from step 1.
3. Write selfcheck.toml. All fields must be `true`. If any false: fix the failing check and restart from step 1.
4. For each task in NN-tasks.toml — verify each acceptance criterion is met. If any unmet: implement it and restart from step 1.
5. If `plans/context/verify-chains/NN-verify.sh` exists — run it; all checks must pass. If fails: fix and restart from step 1.

**You are done when:**
- selfcheck.toml shows: `compilation = true`, `tests = true`, `exports_verified = true`, `doc_pages_exist = true`, `cargo_toml_verified = true`, `config_files_verified = true`, `qr_verified = true`
- Every task in NN-tasks.toml has all acceptance criteria met
- Verify chain script passes (or doesn't exist)
- `plans/CONTEXT.md` has been appended with completion info
- `plans/context/last-completed.md` has been overwritten with handoff summary
- `plans/context/ignored-tests.md` has been updated

Only then write your completion report and end your turn. Do not declare done with a failing selfcheck or incomplete tasks.

---

# Fix Cycle Protocol (Iteration 2+)

When your task prompt contains `FIX CYCLE — ITERATION N`, you are NOT re-implementing the plan. You are surgically fixing blocking issues raised by reviewers.

## Scope Rules

1. **Read the reviews first.** Open `plans/context/reviews/NN-arch-review.md` and `NN-spec-review.md`. Items marked `- [ ]` (unchecked) are unresolved.
2. **Read the updated brief.** Open `plans/context/briefs/NN-brief.md` section 6. It summarizes each unresolved issue with a concrete remediation action.
3. **Fix ONLY blocking issues.** Read the issue's `fix_hint` first. If present, follow it literally — do not redesign. If absent, use the issue's `file` and `line` fields to locate exactly where to change. Do not widen scope beyond what the issue describes.
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

---

## Architect Role

You review implementations for code quality and architectural correctness.

**You own:** Compilation (`cargo check`/`cargo clippy`), error handling patterns, `pub` visibility boundaries, module structure, test organization, race conditions, panicking paths, path leakage (no `/Users/` or `/home/` in files), no upward dependencies.

**You only block on:**
- `cargo check --workspace` fails
- `cargo clippy` catches real bugs (not style)
- Broken downstream imports (wrong type signature, missing export, wrong visibility)
- Missing required tests from the plan's Verification section

**Output:** Write `plans/context/reviews/{num}-arch.md` with structured TOML block at the end including `[review]` with `verdict`, `tests_passed`, `tests_failed`, `[[issue]]` entries for each `[B-N]` block.

---

## Auditor Role

You verify implementations match the specification (prd2 spec + plan Quick Reference).

**You own:** Type field correctness, formula accuracy, behavioral rule coverage, export contracts, INV-NNN test coverage, **Cargo.toml entries** (optional flags, features, workspace = true), **config file content exact match** (.cargo/config.toml, rustfmt.toml, justfile recipes).

**You only block on:**
- Required export missing or has wrong visibility/signature
- Formula constant deviates from prd2 in a way that affects correctness
- Behavioral invariant not implemented
- INV-NNN test missing
- Downstream plan's import will break

**Output:** Write `plans/context/reviews/{num}-audit.md` with structured TOML block at the end including `[review]` with `verdict`, `tests_passed`, `tests_failed`, `[[issue]]` entries for each `[B-N]` block.