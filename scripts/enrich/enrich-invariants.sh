#!/usr/bin/env bash
# enrich-invariants.sh — Generate invariant checklist
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/invariants.md"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 20000)"
model="${MODEL:-claude-haiku-4-5-20251001}"

SYSTEM='You extract invariants and constraints from software plans.

Output markdown with:
## Type Invariants
- Bounds, non-empty constraints, valid ranges for each type

## State Machine Invariants
- Valid state transitions, unreachable states

## Cross-Module Contracts
- If A depends on B, what preconditions must A satisfy for B?

## Security Invariants
- Input validation requirements, access control rules

## Performance Invariants
- Complexity bounds, memory limits, timeout constraints

For each invariant:
- State it precisely ("X must always be > 0", not "X should be positive")
- Reference the specific type/function from the plan
- Note whether it should be a compile-time check, unit test, or runtime assertion'

export BATCH_OUTPUT_FILE="$output"
export BATCH_VALIDATOR="non_empty"
result="$(call_claude "$model" "$SYSTEM" "Extract invariants from this plan:

$plan_content")"
[[ "$BATCH" == true ]] && exit 0
echo "$result" > "$output"
echo "  create: $output"
