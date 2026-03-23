#!/usr/bin/env bash
# enrich-decompose.sh — Generate step-by-step decomposition
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/decomposition.md"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 30000)"
tasks=""
[[ -f "$plan_dir/tasks.toml" ]] && tasks="$(cat "$plan_dir/tasks.toml" | head -c 8000)"
model="${MODEL:-claude-sonnet-4-6}"

SYSTEM='You decompose software plans into step-by-step implementation guides.

Output markdown with this structure:

# Decomposition: Plan Name

## Step 1: Title
**Files:** comma-separated list
**Creates:** one sentence
**Action:**
1. Explicit sub-step with actual type/function names
2. ...

### Checkpoint
```bash
cargo check -p crate_name
```

Rules:
- Each step must compile incrementally
- Config/manifest edits first
- Types/structs/enums before implementations
- Test modules last
- Insert compile checkpoint every 2-3 steps
- Name every file explicitly
- Use actual type names from the plan, not placeholders'

export BATCH_OUTPUT_FILE="$output"
export BATCH_VALIDATOR="non_empty"
result="$(call_claude "$model" "$SYSTEM" "Decompose this plan into implementation steps:

$plan_content

${tasks:+Tasks:
$tasks}")"
[[ "$BATCH" == true ]] && exit 0
echo "$result" > "$output"
echo "  create: $output"
