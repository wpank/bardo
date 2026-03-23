#!/usr/bin/env bash
# enrich-scribe.sh — Generate documentation task TOML
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/scribe-tasks.toml"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 20000)"
model="${MODEL:-claude-haiku-4-5-20251001}"

SYSTEM='You generate documentation task TOML files for a scribe/documentation agent.

Output valid TOML:

[meta]
plan = "plan-name"
role = "scribe"
total = N

[[task]]
id = "N1"
type = "narrative"
title = "Plan-level narrative documentation"
sections = ["context", "architecture", "data_flow", "failure_modes"]

Task types:
- narrative (N*): exactly one, holistic plan overview (MUST be first)
- module_doc (D*): one per crate/module
- citation (C*): one per academic reference found in the plan
- formula (F*): one per formula/equation
- interaction (X*): one per cross-module boundary
- diagram (M*): state/sequence/class/flowchart diagrams

Rules:
- Exactly one N-type task (N1, must be first)
- Order: N, D, C, F, X, M
- Every citation in the plan must have a C-type task
- Output ONLY TOML, no markdown fences'

export BATCH_OUTPUT_FILE="$output"
export BATCH_VALIDATOR="toml_has_meta"
result="$(call_claude "$model" "$SYSTEM" "Generate scribe-tasks.toml:

$plan_content")"
[[ "$BATCH" == true ]] && exit 0
echo "$result" > "$output"
echo "  create: $output"
