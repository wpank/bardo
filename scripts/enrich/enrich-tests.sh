#!/usr/bin/env bash
# enrich-tests.sh — Generate integration test suggestions
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/test-suggestions.md"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 20000)"
tasks=""
[[ -f "$plan_dir/tasks.toml" ]] && tasks="$(cat "$plan_dir/tasks.toml" | head -c 6000)"
model="${MODEL:-claude-sonnet-4-6}"

SYSTEM='You generate integration test suggestions for software plans.

Output markdown with:
1. A table of cross-module contracts (provider crate, consumer crate, test type)
2. Compilable test code covering:
   - Type roundtrip tests (serialize/deserialize)
   - API contract tests (public function signatures)
   - Cross-module data flow tests
   - Property-based tests for numeric/financial code

Rules:
- Use REAL type names from the plan
- Self-contained tests (no external fixtures)
- Include serde roundtrip tests for Serialize+Deserialize types
- Tests must catch real breakage (renamed fields, changed enums)
- Mark each test with the crate it belongs in'

result="$(call_claude "$model" "$SYSTEM" "Generate test suggestions:

$plan_content

${tasks:+Tasks:
$tasks}")"
echo "$result" > "$output"
echo "  create: $output"
