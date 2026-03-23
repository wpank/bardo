#!/usr/bin/env bash
# enrich-verify.sh — Generate verify-tasks.toml from plan + tasks
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/verify-tasks.toml"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 20000)"
tasks=""
[[ -f "$plan_dir/tasks.toml" ]] && tasks="$(cat "$plan_dir/tasks.toml" | head -c 8000)"
model="${MODEL:-claude-haiku-4-5-20251001}"

SYSTEM='You generate verification task TOML files for testing and validation.

Output valid TOML with this structure:

[meta]
plan = "plan-name"
role = "verifier"
total = N

[[task]]
id = "CG1"
type = "compile"
command = "cargo check -p crate_name"
blocking = true
status = "pending"

Task types: compile, unit_test, integration_test, invariant
- Compile gates (CG*) first, always blocking
- Unit test tasks (UT*) from test functions in the plan
- Integration tests (IT*) for cross-module contracts
- Invariant checks (INV*) from ## Verification section

Rules:
- Commands must be valid (cargo check, cargo test, etc.)
- Extract crate names from the plan
- Output ONLY TOML, no markdown fences'

export BATCH_OUTPUT_FILE="$output"
export BATCH_VALIDATOR="toml_has_meta"
result="$(call_claude "$model" "$SYSTEM" "Generate verify-tasks.toml:

Plan:
$plan_content

${tasks:+Implementation tasks:
$tasks}")"
[[ "$BATCH" == true ]] && exit 0
echo "$result" > "$output"
echo "  create: $output"
