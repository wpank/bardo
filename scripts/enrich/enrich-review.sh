#!/usr/bin/env bash
# enrich-review.sh — Generate review-tasks.toml + rubric.md
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/review-tasks.toml"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 20000)"
tasks=""
[[ -f "$plan_dir/tasks.toml" ]] && tasks="$(cat "$plan_dir/tasks.toml" | head -c 8000)"
model="${MODEL:-claude-haiku-4-5-20251001}"

SYSTEM='You generate review task TOML files for code review agents.

Output valid TOML:

[meta]
plan = "plan-name"
role = "reviewer"
total = N

[[task]]
id = "R1"
title = "descriptive review task"
type = "invariant"
severity = "blocking"
check = ["specific thing to verify"]
files = ["files to inspect"]
verdict = "pending"

Types: invariant, contract, acceptance, gate
Severity: blocking, major, minor

Rules:
- Order: blocking first, then major, then minor
- Be specific in checks ("verify X returns Y", not "check implementation")
- Every invariant from ## Verification must appear
- Output ONLY TOML, no markdown fences'

result="$(call_claude "$model" "$SYSTEM" "Generate review-tasks.toml:

Plan:
$plan_content

${tasks:+Tasks:
$tasks}")"
echo "$result" > "$output"
echo "  create: $output"
