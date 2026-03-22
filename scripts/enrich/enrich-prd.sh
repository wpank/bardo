#!/usr/bin/env bash
# enrich-prd.sh — Extract relevant PRD sections for a plan
source "$(dirname "$0")/_common.sh"
parse_flags "$@"

plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
output="$plan_dir/prd-extract.md"
should_skip "$output" && exit 0

plan_content="$(cat "$plan_dir/plan.md" | head -c 10000)"

# Find PRD directory (prd2/ or prd/)
PRD_DIR=""
for d in "$ROOT/prd2" "$ROOT/prd" "$ROOT/docs/prd"; do
    [[ -d "$d" ]] && PRD_DIR="$d" && break
done

if [[ -z "$PRD_DIR" ]]; then
    echo "  skip: no PRD directory found (checked prd2/, prd/, docs/prd/)"
    exit 0
fi

# Extract prd2 paths referenced in the plan
refs="$(grep -oE 'prd2?/[^ )\"]+\.md' "$plan_dir/plan.md" 2>/dev/null | sort -u || true)"

BUDGET=20000
output_content="# PRD Extract for $(basename "$plan_dir")\n\n"

if [[ -n "$refs" ]]; then
    while IFS= read -r ref; do
        filepath="$ROOT/$ref"
        [[ -f "$filepath" ]] || continue
        content="$(head -c $((BUDGET / 10)) "$filepath")"
        output_content+="## $ref\n\n$content\n\n---\n\n"
    done <<< "$refs"
fi

echo -e "$output_content" > "$output"
echo "  create: $output (extracted from PRD references)"
