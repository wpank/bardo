#!/usr/bin/env bash
# Shared helpers for enrichment scripts.
# Source this at the top of each enrich-*.sh script.

set -euo pipefail

ROOT="${ROOT:-.}"
PLAN=""
FORCE=false
MODEL=""
BATCH=false
GATEWAY_URL="${BARDO_GATEWAY_URL:-http://127.0.0.1:4000}"
GATEWAY_KEY="${BARDO_GATEWAY_API_KEY:-mori-local-gateway}"

parse_flags() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --plan)   PLAN="$2"; shift 2 ;;
            --root)   ROOT="$2"; shift 2 ;;
            --model)  MODEL="$2"; shift 2 ;;
            --force)  FORCE=true; shift ;;
            --batch)  BATCH=true; shift ;;
            --all)    PLAN="__all__"; shift ;;
            *)        echo "unknown flag: $1" >&2; exit 1 ;;
        esac
    done
    if [[ -z "$PLAN" ]]; then
        echo "usage: $0 --plan <plan-name-or-number> [--root .] [--force] [--model <model>]" >&2
        exit 1
    fi
}

# Find the plan directory matching a plan name or number prefix.
find_plan_dir() {
    local root="$1" plan="$2"
    for base in "$root/.mori/plans" "$root/plans"; do
        if [[ -d "$base" ]]; then
            for d in "$base"/${plan}*/; do
                if [[ -f "$d/plan.md" ]]; then
                    echo "$d"
                    return 0
                fi
            done
        fi
    done
    echo "error: plan '$plan' not found in $root" >&2
    return 1
}

# List all plan directories.
all_plan_dirs() {
    local root="$1"
    for base in "$root/.mori/plans" "$root/plans"; do
        if [[ -d "$base" ]]; then
            for d in "$base"/*/; do
                if [[ -f "$d/plan.md" ]]; then
                    echo "$d"
                fi
            done
            return 0
        fi
    done
}

# Call Claude CLI with system prompt and user message.
# In batch mode ($BATCH=true), submits to the gateway batch API instead.
call_claude() {
    local model="${1:-claude-haiku-4-5-20251001}"
    local system="$2"
    local message="$3"

    if [[ "$BATCH" == true ]]; then
        batch_submit "$model" "$system" "$message"
        return 0
    fi

    claude --print --model "$model" --max-turns 1 --output-format text \
        --system-prompt "$system" "$message"
}

# Submit a prompt to the gateway batch API.
# Writes a manifest line for later collection.
batch_submit() {
    local model="$1" system="$2" message="$3"
    local body
    body=$(jq -n \
        --arg model "$model" \
        --arg system "$system" \
        --arg message "$message" \
        --argjson max_tokens "${BATCH_MAX_TOKENS:-16000}" \
        '{
            model: $model,
            max_tokens: $max_tokens,
            system: $system,
            messages: [{role: "user", content: $message}]
        }')

    local resp
    resp=$(curl -sf -X POST "${GATEWAY_URL}/v1/batch/submit" \
        -H "Authorization: Bearer ${GATEWAY_KEY}" \
        -H "Content-Type: application/json" \
        -d "$body" 2>/dev/null) || {
        echo "  warn: batch submit failed, falling back to real-time" >&2
        BATCH=false
        claude --print --model "$model" --max-turns 1 --output-format text \
            --system-prompt "$system" "$message"
        return $?
    }

    local item_id
    item_id=$(echo "$resp" | jq -r '.batch_item_id')

    if [[ -n "$BATCH_MANIFEST" && -n "$item_id" && "$item_id" != "null" ]]; then
        printf '%s\t%s\t%s\n' "$item_id" "${BATCH_OUTPUT_FILE:-/dev/null}" "${BATCH_VALIDATOR:-none}" >> "$BATCH_MANIFEST"
        echo "  queued: ${BATCH_OUTPUT_FILE:-?} (item: $item_id)"
    fi

    # Return empty — caller should check BATCH mode
    echo ""
}

# Check if output file exists and --force not set.
should_skip() {
    local path="$1"
    if [[ -f "$path" ]] && [[ "$FORCE" != true ]]; then
        echo "  skip: $path (exists)"
        return 0
    fi
    return 1
}
