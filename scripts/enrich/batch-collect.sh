#!/usr/bin/env bash
# batch-collect.sh — Poll gateway for batch results and write output files.
#
# Usage: ./scripts/enrich/batch-collect.sh [--manifest path] [--timeout 3600]
#
# Reads a TSV manifest (batch_item_id \t output_file \t validator) and polls
# the gateway batch API until all items complete or timeout is reached.

set -euo pipefail

GATEWAY_URL="${BARDO_GATEWAY_URL:-http://127.0.0.1:4000}"
GATEWAY_KEY="${BARDO_GATEWAY_API_KEY:-mori-local-gateway}"
MANIFEST="${BATCH_MANIFEST:-}"
TIMEOUT="${BATCH_TIMEOUT:-3600}"
POLL_INTERVAL=30

while [[ $# -gt 0 ]]; do
    case "$1" in
        --manifest)  MANIFEST="$2"; shift 2 ;;
        --timeout)   TIMEOUT="$2"; shift 2 ;;
        *)           echo "unknown flag: $1" >&2; exit 1 ;;
    esac
done

if [[ -z "$MANIFEST" || ! -f "$MANIFEST" ]]; then
    echo "error: manifest file not found: $MANIFEST" >&2
    exit 1
fi

# Flush any remaining queued items
echo "Flushing batch queue..."
curl -sf -X POST "${GATEWAY_URL}/v1/batch/flush" \
    -H "Authorization: Bearer ${GATEWAY_KEY}" > /dev/null 2>&1 || true

deadline=$(($(date +%s) + TIMEOUT))
total=$(wc -l < "$MANIFEST" | tr -d ' ')
completed=0
failed=0

# Track which items are already collected
declare -A collected

echo "Collecting $total batch results (timeout: ${TIMEOUT}s)..."

while [[ $((completed + failed)) -lt $total && $(date +%s) -lt $deadline ]]; do
    while IFS=$'\t' read -r item_id output_file validator; do
        # Skip already collected
        [[ -n "${collected[$item_id]:-}" ]] && continue

        resp=$(curl -sf "${GATEWAY_URL}/v1/batch/result/${item_id}" \
            -H "Authorization: Bearer ${GATEWAY_KEY}" 2>/dev/null) || continue

        status=$(echo "$resp" | jq -r '.status')

        if [[ "$status" == "completed" ]]; then
            # Extract text from Anthropic Messages response
            text=$(echo "$resp" | jq -r '
                .result.result.message.content[0].text //
                .result.result.content[0].text //
                .result.content[0].text //
                empty
            ' 2>/dev/null)

            if [[ -z "$text" ]]; then
                echo "  FAIL: $output_file (empty response)"
                failed=$((failed + 1))
                collected[$item_id]=1
                continue
            fi

            # Run validator
            valid=true
            case "$validator" in
                toml_has_meta)
                    echo "$text" | grep -q '^\[meta\]' || valid=false ;;
                has_frontmatter)
                    echo "$text" | head -1 | grep -q '^---' || valid=false ;;
                non_empty)
                    [[ -n "$text" ]] || valid=false ;;
                none) ;;
            esac

            if $valid; then
                mkdir -p "$(dirname "$output_file")"
                echo "$text" > "$output_file"
                completed=$((completed + 1))
                collected[$item_id]=1
                echo "  OK: $output_file"
            else
                echo "  FAIL: $output_file (validation: $validator)"
                failed=$((failed + 1))
                collected[$item_id]=1
            fi
        fi
    done < "$MANIFEST"

    [[ $((completed + failed)) -ge $total ]] && break
    remaining=$((total - completed - failed))
    echo "  waiting... ($remaining pending, $(( deadline - $(date +%s) ))s remaining)"
    sleep $POLL_INTERVAL
done

echo ""
echo "Batch complete: $completed/$total succeeded, $failed failed"

if [[ $((completed + failed)) -lt $total ]]; then
    echo "WARNING: $((total - completed - failed)) items still pending (timeout reached)"
    exit 1
fi
