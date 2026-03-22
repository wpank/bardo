#!/usr/bin/env bash
set -euo pipefail

# migrate-to-plan-dirs.sh — Reorganize plans/ from flat files to per-plan directories.
#
# Before:  plans/01-workspace-scaffold.md + plans/context/briefs/01-brief.md + ...
# After:   plans/01-workspace-scaffold/plan.md + plans/01-workspace-scaffold/brief.md + ...
#
# This script COPIES files (never deletes originals). After verifying mori works
# with the new layout, you can remove the old flat files manually.
#
# Usage: ./scripts/migrate-to-plan-dirs.sh [--dry-run]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PLANS_DIR="$REPO_ROOT/plans"
CTX_DIR="$PLANS_DIR/context"
DRY_RUN=false

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "=== DRY RUN (no files will be created) ==="
fi

do_cp() {
    local src="$1" dst="$2"
    if [[ "$DRY_RUN" == true ]]; then
        echo "  cp $src → $dst"
    else
        cp "$src" "$dst"
    fi
}

do_mkdir() {
    local dir="$1"
    if [[ "$DRY_RUN" == true ]]; then
        echo "  mkdir -p $dir"
    else
        mkdir -p "$dir"
    fi
}

copied=0
skipped=0
plans=0

# Iterate over every plan .md file
for plan_file in "$PLANS_DIR"/*.md; do
    filename="$(basename "$plan_file")"

    # Skip non-plan files
    [[ "$filename" == "CONTEXT.md" ]] && continue

    # Extract base name (e.g., "01-workspace-scaffold")
    base="${filename%.md}"

    # Extract plan number prefix (e.g., "01", "08a", "R01")
    num="${base%%-*}"

    # Create plan directory
    plan_dir="$PLANS_DIR/$base"
    do_mkdir "$plan_dir"
    plans=$((plans + 1))

    # Copy plan document
    do_cp "$plan_file" "$plan_dir/plan.md"
    copied=$((copied + 1))

    # Brief
    src="$CTX_DIR/briefs/${num}-brief.md"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/brief.md"
        copied=$((copied + 1))
    fi

    # Implementer tasks
    src="$CTX_DIR/tasks/${num}-tasks.toml"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/tasks.toml"
        copied=$((copied + 1))
    fi

    # Review tasks
    src="$CTX_DIR/tasks/${num}-review-tasks.toml"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/review-tasks.toml"
        copied=$((copied + 1))
    fi

    # Verify tasks
    src="$CTX_DIR/tasks/${num}-verify-tasks.toml"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/verify-tasks.toml"
        copied=$((copied + 1))
    fi

    # Scribe tasks
    src="$CTX_DIR/tasks/${num}-scribe-tasks.toml"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/scribe-tasks.toml"
        copied=$((copied + 1))
    fi

    # Shared rubric
    src="$CTX_DIR/tasks/${num}-shared-rubric.md"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/rubric.md"
        copied=$((copied + 1))
    fi

    # PRD extract
    src="$CTX_DIR/prd2-extracts/${num}-prd2.md"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/prd-extract.md"
        copied=$((copied + 1))
    fi

    # Decomposition
    src="$CTX_DIR/decompositions/${num}-decomposition.md"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/decomposition.md"
        copied=$((copied + 1))
    fi

    # Reviews (copy into reviews/ subdirectory)
    has_reviews=false
    for review_file in "$CTX_DIR/reviews/${num}-"*; do
        if [[ -f "$review_file" ]]; then
            if [[ "$has_reviews" == false ]]; then
                do_mkdir "$plan_dir/reviews"
                has_reviews=true
            fi
            review_name="$(basename "$review_file")"
            # Strip plan number prefix: "01-arch.md" → "arch.md"
            short_name="${review_name#${num}-}"
            do_cp "$review_file" "$plan_dir/reviews/$short_name"
            copied=$((copied + 1))
        fi
    done

    # Testing backlog (if exists)
    src="$CTX_DIR/tasks/${num}-testing-backlog.md"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$plan_dir/testing-backlog.md"
        copied=$((copied + 1))
    fi
done

# Move global files to plans/ root
echo ""
echo "Global context files:"

for global in "workspace-map.md" "preflight-snapshot.md" "ignored-tests.md" "last-gate-output.txt" "last-completed.md"; do
    src="$CTX_DIR/$global"
    if [[ -f "$src" ]]; then
        do_cp "$src" "$PLANS_DIR/$global"
        copied=$((copied + 1))
        echo "  $global → plans/$global"
    fi
done

# Registry stays at plans/registry/ (move from plans/context/registry/)
if [[ -d "$CTX_DIR/registry" ]]; then
    do_mkdir "$PLANS_DIR/registry"
    for f in "$CTX_DIR/registry/"*; do
        if [[ -f "$f" ]]; then
            do_cp "$f" "$PLANS_DIR/registry/$(basename "$f")"
            copied=$((copied + 1))
        fi
    done
    echo "  registry/ → plans/registry/"
fi

echo ""
echo "Migration summary:"
echo "  Plans processed: $plans"
echo "  Files copied: $copied"
echo ""
if [[ "$DRY_RUN" == true ]]; then
    echo "This was a dry run. Run without --dry-run to execute."
else
    echo "Done. Original files are untouched."
    echo "After verifying mori works with the new layout, old files can be removed."
fi
