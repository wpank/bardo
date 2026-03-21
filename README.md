# Bardo

This repository starts from the implementation plan set under `plans/`.

Execution restarts at Batch 01 and must proceed batch-by-batch from the plan files, with git branches, commit slices, proofs, docs parity, and changelog updates handled exactly as the execution manual specifies.

## Local setup

- **Git hooks:** Hooks live in [`.githooks/`](.githooks/). After clone, register them once:
  `git config core.hooksPath .githooks`
- **Agent context files:** Regenerate the crate tree and preflight snapshot anytime:
  `./scripts/bardo-sync-context.sh`
  Fast mode (git only, no `cargo check`): `SKIP_CARGO_CHECK=1 ./scripts/bardo-sync-context.sh`
- **Task TOMLs / enrichment:** See [AGENTS.md](AGENTS.md) (`bardo-enrich.sh`, `retrofit-tomls.sh`, distiller, golden-path index).
