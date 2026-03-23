# Mori quality gates

## The verification gap

Most AI coding tools generate code and hand it to you. They don't check if it compiles. They don't run the tests. They don't review for design problems. The developer is the quality gate, copying error output back into the chat, waiting for the model to try again, then copying the next error. This manual loop is slow, expensive, and breaks the moment you step away.

Mori has automated gates that every piece of agent output must pass before it's accepted. If the code doesn't compile, the build iterates. If tests fail, it iterates again. If an architecture review flags problems, same thing. The agent keeps going until it passes all gates or exhausts its retry budget. No human in the loop.

This is not optional. Every plan, every iteration, every agent -- the gates run. The system does not produce output that hasn't been verified.


## Three-tier gate system

Gates execute in sequence after each implementation iteration. If any gate fails, later gates don't run.

### Compile gate

The compile gate runs `cargo check`, but not on the full workspace. It detects which crates have changes by running `git diff --name-only HEAD` on `.rs` files, mapping changed file paths to their parent crate directories, and reading each crate's `Cargo.toml` to get the package name. The result is a targeted command like:

```
cargo check -p bardo-gateway -p golem-core
```

If a change touches `crates/golem-core/src/session.rs` and `apps/bardo-gateway/src/routes.rs`, only those two crates get checked. Not the 50-crate workspace. This matters because a full workspace check takes 30-90 seconds on a cold build, while a scoped check on two crates takes 5-15 seconds.

If the diff detection fails (no `.rs` files changed, or files outside `crates/` and `apps/`), it falls back to `--workspace`.

### Test gate

Same scoping logic. Runs `cargo test -p crate1 -p crate2` on affected crates only. Captures pass/fail/ignored counts in a `TestCount` struct. A test gate failure includes which tests failed, not just a binary pass/fail.

### Review gate

Reviews are conditional on plan complexity. Complex plans get three parallel reviewers: Architect (design correctness and cross-crate impact), Auditor (code quality and edge cases), and Scribe (documentation accuracy). Standard plans get a single QuickReviewer that does one combined pass. Trivial and simple plans skip reviews entirely -- the compile and test gates are enough for a one-line change.


## Structured error digest

Raw `cargo check` output for a real compilation failure can run 2,000+ lines. Warnings about unused imports, notes about where a trait is defined, suggestions for similar-looking items, duplicate messages from different compilation stages. Dumping all of that into the agent's context wastes tokens and buries the actual errors under noise.

Mori parses the output and extracts a structured error digest. The `extract_error_digest` function walks each line, captures blocks that start with `error[E` or `error:`, attaches their `-->` file:line references, deduplicates by the first line of each error block, and caps the result at 10 unique errors.

Here's what the agent actually sees versus what cargo produced:

**Raw cargo output (truncated from 847 lines):**
```
   Compiling golem-core v0.1.0 (/workspace/crates/golem-core)
warning: unused import: `std::collections::BTreeMap`
  --> crates/golem-core/src/lib.rs:4:5
   |
4  | use std::collections::BTreeMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

error[E0308]: mismatched types
  --> crates/golem-core/src/auth/session.rs:42:24
   |
42 |     let expiry: u64 = Utc::now().timestamp();
   |                ---    ^^^^^^^^^^^^^^^^^^^^^^^ expected `u64`, found `i64`
   |                |
   |                expected due to this
   |
   = note: for more information about this error, try `rustc --explain E0308`

error[E0433]: failed to resolve: use of undeclared crate or module `serde_json`
  --> crates/golem-core/src/auth/session.rs:58:19
   |
58 |     let payload = serde_json::to_string(&claims)?;
   |                   ^^^^^^^^^^ use of undeclared crate or module `serde_json`

... [800+ more lines of warnings, notes, and help suggestions]
```

**Error digest the agent receives:**
```
2 unique error(s):

error[E0308]: mismatched types
  --> crates/golem-core/src/auth/session.rs:42:24
   |
42 |     let expiry: u64 = Utc::now().timestamp();

error[E0433]: failed to resolve: use of undeclared crate or module `serde_json`
  --> crates/golem-core/src/auth/session.rs:58:19
   |
58 |     let payload = serde_json::to_string(&claims)?;
```

Two errors, two file references, two concrete things to fix. The agent can act on this immediately.


## The reflection loop

A bare error digest tells the agent *what* broke but not *why* or *what to try next*. For that, Mori spawns a reflection call.

On gate failure, before the next iteration begins, Mori fires an async request to Claude Haiku. Cost: about $0.01. Latency: 1-2 seconds. The reflection prompt includes the error digest (truncated to 8,000 characters if needed) and asks four questions:

1. What failed?
2. Why did it fail?
3. What should the agent try differently?
4. Which files and functions should it focus on?

The response comes back as structured markdown:

```markdown
## What failed
Compilation failed due to a type mismatch in session expiry calculation
and a missing serde_json dependency.

## Why it failed
`Utc::now().timestamp()` returns i64 (Unix timestamps can be negative for
dates before 1970), but the expiry field is declared as u64. The serde_json
crate is used in session.rs but not listed in golem-core's Cargo.toml
dependencies.

## What to try differently
- Cast the timestamp with `as u64` or change the expiry field type to i64
- Add `serde_json = "1"` to crates/golem-core/Cargo.toml under [dependencies]

## Files/functions to focus on
- crates/golem-core/src/auth/session.rs:42 (type mismatch)
- crates/golem-core/src/auth/session.rs:58 (missing import)
- crates/golem-core/Cargo.toml (missing dependency)
```

This reflection is stored in the plan's iteration memory and injected into the next iteration's context. The implementer doesn't just see "there was a compile error" -- it sees a diagnosis of what went wrong and a concrete plan for fixing it.

Error pattern deduplication prevents redundant analysis. If the same error code appears at the same file location across iterations, Mori reuses the existing reflection instead of calling Haiku again. The `has_error_pattern` check compares the first error line against all stored gate results. Same error, same analysis, no wasted inference.

The reflection call is non-blocking. It runs as a spawned async task while the retry pipeline sets up the next iteration. Because Haiku typically responds in 1-2 seconds and retry setup takes 5-10 seconds, the reflection is almost always ready before the next iteration needs it.


## Iteration memory

Each plan maintains a persistent record of every gate failure it has encountered. The `IterationMemory` struct tracks a list of `IterationEntry` records, each containing:

- **Iteration number** -- which attempt this was
- **Gate results** -- a map of gate name to outcome (the error pattern that triggered the failure)
- **Diagnosis** -- the LLM-generated reflection, if one was produced
- **Files changed** -- which files were modified in that attempt

This history is serialized to JSON at `plans/context/iteration-memory/{plan_num}-memory.json` and loaded at the start of each new iteration.

When the implementer starts iteration 3, it sees context like this:

```markdown
# Prior iteration reflections

These are analyses of previous failed attempts at this plan.
Learn from these failures and avoid repeating the same mistakes.

## Iteration 1 reflection

Compilation failed due to mismatched types at session.rs:42...
[full reflection text]

Files changed in that attempt: crates/golem-core/src/auth/session.rs

---

## Iteration 2 reflection

Tests failed: session_timeout test reported an off-by-one error
in the expiry calculation. The test expects 3600 seconds but the
implementation adds 3599 due to truncating the millisecond component...

Files changed in that attempt: crates/golem-core/src/auth/session.rs,
crates/golem-core/Cargo.toml

---
```

The agent sees what was tried, what broke, and why. It doesn't have to re-discover that `as u64` truncation was already attempted in iteration 1. It can try a different approach.

This history persists across process restarts. If Mori crashes and resumes, the iteration memory is still on disk. No progress is lost.


## Complexity-driven pipeline

Not every plan needs the full verification pipeline. Running three parallel reviewers on a one-line derive attribute change is waste. Mori classifies plans into four complexity tiers based on metadata from the plan frontmatter: task count, estimated file count, and number of crates touched.

**Trivial** -- adding a derive, fixing an import, renaming a constant. No strategist phase, no reviews. One iteration max. If it doesn't compile on the first try, it gets escalated rather than retried. These plans take 30-60 seconds.

**Simple** -- a small feature touching a few files in one crate. No strategist, no reviews. Two iterations allowed. The compile and test gates catch problems; a reviewer would add latency without adding much signal.

**Standard** -- a normal plan. Compile and test gates run with two iterations. Reviews are currently disabled at this tier (self-validation plus gates have proven sufficient), but the QuickReviewer can be re-enabled in config.

**Complex** -- multi-crate changes, state machines, protocol implementations. Full gate pipeline with two iterations. A QuickReviewer runs a single combined design/quality/docs pass. These are the plans where a reviewer catches architectural issues that compile and test gates miss, like a new struct that should implement a trait from a different crate, or a public API that breaks the existing contract.

The classification function checks for risk escalation signals: if a plan touches `golem-core` (the shared kernel) or has three or more plan dependencies, it gets bumped up at least one tier regardless of size. A small change to the core crate can break downstream consumers in ways that only a review catches.


## AutoFixer

Some gate failures are trivial. A missing import, a type mismatch that needs a single cast, a struct literal missing a field that has an obvious default. Re-running the full Implementer agent (Opus, $0.15-0.50 per call, 30-60 seconds) for these is overkill.

Mori's AutoFixer handles these cases. When the compile gate fails, the `autofix` module classifies each error by parsing cargo's JSON diagnostic output. Errors are sorted into categories: `ImportNotFound`, `TypeMismatch`, `MissingField`, `TraitNotImplemented`, and `Other`. The first two categories are flagged as "simple" -- fixable with high confidence by a lightweight model.

When all errors in a failed gate are classified as simple, Mori spawns an AutoFixer agent using Haiku instead of the full Implementer. The AutoFixer sees the error digest and the contents of the affected file, makes the minimal fix, and commits. If the gates pass after the AutoFixer's change, the plan moves forward. If they don't, Mori escalates to the full Implementer for the next iteration.

The cost difference is significant. A Haiku AutoFixer call costs about $0.01 and completes in 2-3 seconds. A full Opus Implementer call costs $0.15-0.50 and takes 30-60 seconds. For a build with 30 plans where 40% hit a simple compile error on the first iteration, that's the difference between $0.12 and $3-6 for the fix pass alone.

The AutoFixer doesn't attempt anything ambitious. It won't refactor a function signature or redesign an API boundary. It fixes the import, adds the cast, fills in the missing field. If the error is an `Other` or `TraitNotImplemented`, the full Implementer handles it. The boundary is intentionally conservative -- a failed AutoFixer attempt wastes an iteration, and iterations are limited.
