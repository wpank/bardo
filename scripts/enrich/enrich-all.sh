#!/usr/bin/env bash
# enrich-all.sh — Run all enrichment steps for a plan (or all plans).
#
# Usage:
#   ./scripts/enrich/enrich-all.sh --plan 01 --root .
#   ./scripts/enrich/enrich-all.sh --all --root .
#   ./scripts/enrich/enrich-all.sh --plan 01 --force   # overwrite existing

source "$(dirname "$0")/_common.sh"
parse_flags "$@"

SCRIPT_DIR="$(dirname "$0")"

run_for_plan() {
    local plan_dir="$1"
    local plan_name
    plan_name="$(basename "$plan_dir")"
    echo "Enriching: $plan_name"

    FLAGS=(--root "$ROOT")
    [[ "$FORCE" == true ]] && FLAGS+=(--force)
    [[ -n "$MODEL" ]] && FLAGS+=(--model "$MODEL")

    for step in briefs tasks verify review prd decompose tests invariants scribe; do
        script="$SCRIPT_DIR/enrich-${step}.sh"
        if [[ -x "$script" ]]; then
            bash "$script" --plan "$plan_name" "${FLAGS[@]}" || true
        fi
    done
    echo ""
}

if [[ "$PLAN" == "__all__" ]]; then
    while IFS= read -r dir; do
        run_for_plan "$dir"
    done < <(all_plan_dirs "$ROOT")
else
    plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
    run_for_plan "$plan_dir"
fi
