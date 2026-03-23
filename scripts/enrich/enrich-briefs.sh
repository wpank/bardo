#!/usr/bin/env bash
# enrich-briefs.sh — Generate brief.md from plan.md
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/brief.md"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 30000)"
model="${MODEL:-claude-haiku-4-5-20251001}"

SYSTEM="You are a technical writer creating implementation briefs.

Given a plan document, produce a concise brief with these sections:
## Authority Chain — which documents take precedence
## Dependencies — prerequisites (other plans, modules, libraries)
## Task Map — table of implementation units with files and acceptance criteria
## Key Types and Interfaces — important types/traits this plan introduces
## Verification Checklist — what must pass (compile, test, lint)

Keep under 3000 words. Focus on what an implementer needs to start coding."

export BATCH_OUTPUT_FILE="$output"
export BATCH_VALIDATOR="non_empty"
result="$(call_claude "$model" "$SYSTEM" "Generate a brief for this plan:

$plan_content")"
[[ "$BATCH" == true ]] && exit 0
echo "$result" > "$output"
echo "  create: $output"
