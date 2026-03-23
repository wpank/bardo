#!/usr/bin/env bash
# enrich-tasks.sh — Generate tasks.toml from plan.md
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/tasks.toml"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 30000)"
brief=""
[[ -f "$plan_dir/brief.md" ]] && brief="$(cat "$plan_dir/brief.md" | head -c 8000)"
model="${MODEL:-claude-sonnet-4-6}"

SYSTEM='You generate structured task TOML files from software plans.

Output valid TOML with this structure:

[meta]
plan = "plan-name"
total = N
estimated_total_minutes = N

[[task]]
id = "T1"
title = "descriptive title"
status = "pending"
files = ["path/to/file"]
acceptance = ["criterion 1"]
depends_on = []
estimated_seconds = 600

Rules:
- One task per implementation unit/section in the plan
- Extract file paths from the plan (backtick-quoted paths)
- Tasks should be ordered so dependencies flow forward (T1 before T2 if T2 depends on T1)
- Set depends_on when one task requires another to be done first
- Estimate 600s (10min) per task as default
- Output ONLY the TOML, no markdown fences'

export BATCH_OUTPUT_FILE="$output"
export BATCH_VALIDATOR="toml_has_meta"
result="$(call_claude "$model" "$SYSTEM" "Generate tasks.toml for this plan:

$plan_content

${brief:+Brief summary:
$brief}")"
[[ "$BATCH" == true ]] && exit 0
echo "$result" > "$output"
echo "  create: $output"
