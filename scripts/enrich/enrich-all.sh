#!/usr/bin/env bash
# enrich-all.sh — Run all enrichment steps for a plan (or all plans).
#
# Usage:
#   ./scripts/enrich/enrich-all.sh --plan 01 --root .
#   ./scripts/enrich/enrich-all.sh --all --root .
#   ./scripts/enrich/enrich-all.sh --plan 01 --force   # overwrite existing
#   ./scripts/enrich/enrich-all.sh --plan 01 --batch   # 50% cheaper via Batch API

source "$(dirname "$0")/_common.sh"
parse_flags "$@"

SCRIPT_DIR="$(dirname "$0")"

# Steps that must run real-time (downstream deps within the pipeline)
REALTIME_STEPS="prd briefs tasks"
# Steps safe to batch (no downstream deps within a single enrichment run)
BATCHABLE_STEPS="verify review decompose tests invariants scribe"

run_for_plan() {
    local plan_dir="$1"
    local plan_name
    plan_name="$(basename "$plan_dir")"
    echo "Enriching: $plan_name"

    FLAGS=(--root "$ROOT")
    [[ "$FORCE" == true ]] && FLAGS+=(--force)
    [[ -n "$MODEL" ]] && FLAGS+=(--model "$MODEL")

    if [[ "$BATCH" == true ]]; then
        # Phase 1: Run real-time steps (needed by later steps)
        for step in $REALTIME_STEPS; do
            script="$SCRIPT_DIR/enrich-${step}.sh"
            if [[ -x "$script" ]]; then
                bash "$script" --plan "$plan_name" "${FLAGS[@]}" || true
            fi
        done

        # Phase 2: Submit batchable steps
        for step in $BATCHABLE_STEPS; do
            script="$SCRIPT_DIR/enrich-${step}.sh"
            if [[ -x "$script" ]]; then
                bash "$script" --plan "$plan_name" "${FLAGS[@]}" --batch || true
            fi
        done
    else
        for step in briefs tasks verify review prd decompose tests invariants scribe; do
            script="$SCRIPT_DIR/enrich-${step}.sh"
            if [[ -x "$script" ]]; then
                bash "$script" --plan "$plan_name" "${FLAGS[@]}" || true
            fi
        done
    fi
    echo ""
}

# In batch mode, set up the manifest for collection
if [[ "$BATCH" == true ]]; then
    export BATCH_MANIFEST
    BATCH_MANIFEST="$(mktemp /tmp/bardo-batch-manifest.XXXXXX)"
    echo "Batch mode: manifest at $BATCH_MANIFEST"
fi

if [[ "$PLAN" == "__all__" ]]; then
    while IFS= read -r dir; do
        run_for_plan "$dir"
    done < <(all_plan_dirs "$ROOT")
else
    plan_dir="$(find_plan_dir "$ROOT" "$PLAN")"
    run_for_plan "$plan_dir"
fi

# In batch mode, collect results
if [[ "$BATCH" == true && -s "$BATCH_MANIFEST" ]]; then
    echo "Collecting batch results..."
    bash "$SCRIPT_DIR/batch-collect.sh" --manifest "$BATCH_MANIFEST"
    rm -f "$BATCH_MANIFEST"
elif [[ "$BATCH" == true ]]; then
    echo "No batch items submitted (all outputs already exist?)"
    rm -f "$BATCH_MANIFEST"
fi
