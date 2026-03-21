#!/usr/bin/env bash
# bardo-enrich.sh — Runs the enrichment scripts that ship in this repo.
#
# Only plans/context/prompts/enhance-toml.sh and enhance-plan.sh are guaranteed
# to exist. Legacy generators (retrofit-plans, extract-prd2-context,
# task-decomposer, self-verify-chain, generate-verify-toml) are not in-tree;
# use other tooling or add scripts under plans/context/prompts/ as needed.
#
# Usage:
#   ./bardo-enrich.sh 01 02 03              # enhance-toml per plan (if NN-tasks.toml exists)
#   ./bardo-enrich.sh --all                 # all plans/*.md
#   ./bardo-enrich.sh --range 01-25         # numeric range (matches base plan number)
#   ./bardo-enrich.sh 01 --enhance-plan     # also run enhance-plan.sh (needs optional context inputs)
#   ./bardo-enrich.sh 01 --dry-run          # print what would run
#   ./bardo-enrich.sh 01 --parallel 4       # parallel workers (default: 4)
#
# Deprecated (ignored with a warning): --skip-retrofit
#
# Env:
#   BACKEND=claude|cursor
#   MODEL_CLAUDE=...
#   CLAUDE_MODEL=...
#   MODEL_CURSOR=composer-2-fast (default; Cursor agent CLI)

# Require bash 4+ for associative arrays (macOS ships with bash 3.2)
if [[ "${BASH_VERSINFO[0]}" -lt 4 ]]; then
  for _bash4 in /opt/homebrew/bin/bash /usr/local/bin/bash; do
    if [[ -x "$_bash4" ]] && [[ "$("$_bash4" -c 'echo ${BASH_VERSINFO[0]}')" -ge 4 ]]; then
      exec "$_bash4" "$0" "$@"
    fi
  done
  echo "bash 4+ required: brew install bash" >&2
  exit 1
fi

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLANS_DIR="$SCRIPT_DIR/plans"
PROMPTS_DIR="$SCRIPT_DIR/plans/context/prompts"
CONTEXT_DIR="$SCRIPT_DIR/plans/context"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

DRY_RUN=false
PARALLEL=4
PLAN_NUMS=()
DO_ALL=false
RANGE=""
DO_ENHANCE_PLAN=false
WARNED_DEPRECATED=false

# Default to Cursor agent CLI so MODEL_CURSOR (composer-2-fast) applies; set BACKEND=claude for Anthropic CLI.
export BACKEND="${BACKEND:-cursor}"
export MODEL_CLAUDE="${MODEL_CLAUDE:-claude-haiku-4-5-20251001}"
export CLAUDE_MODEL="${CLAUDE_MODEL:-$MODEL_CLAUDE}"
export MODEL_CURSOR="${MODEL_CURSOR:-composer-2-fast}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all)           DO_ALL=true;         shift ;;
    --range)         RANGE="$2";          shift 2 ;;
    --dry-run)       DRY_RUN=true;        shift ;;
    --parallel)      PARALLEL="$2";       shift 2 ;;
    --enhance-plan)  DO_ENHANCE_PLAN=true; shift ;;
    --skip-retrofit)
      if ! $WARNED_DEPRECATED; then
        echo -e "${YELLOW}bardo-enrich: --skip-retrofit is deprecated (retrofit step removed).${NC}" >&2
        WARNED_DEPRECATED=true
      fi
      shift ;;
    --*)
      echo -e "${YELLOW}bardo-enrich: unknown flag $1 (ignored)${NC}" >&2
      shift ;;
    *)               PLAN_NUMS+=("$1");   shift ;;
  esac
done

resolve_plans() {
  local -a nums=()
  if $DO_ALL; then
    for f in "$PLANS_DIR"/[0-9]*.md; do
      [[ -f "$f" ]] || continue
      local n
      n=$(basename "$f" | grep -oE '^[0-9]+[a-z]?' || true)
      [[ -n "$n" ]] && nums+=("$n")
    done
  elif [[ -n "$RANGE" ]]; then
    local range_start range_end
    range_start=$(echo "$RANGE" | cut -d- -f1)
    range_end=$(echo "$RANGE" | cut -d- -f2)
    for f in "$PLANS_DIR"/[0-9]*.md; do
      [[ -f "$f" ]] || continue
      local plan_num
      plan_num=$(basename "$f" | grep -oE '^[0-9]+')
      if [[ "$((10#$plan_num))" -ge "$((10#$range_start))" ]] && \
         [[ "$((10#$plan_num))" -le "$((10#$range_end))" ]]; then
        local n
        n=$(basename "$f" | grep -oE '^[0-9]+[a-z]?' || true)
        [[ -n "$n" ]] && nums+=("$n")
      fi
    done
  else
    nums=("${PLAN_NUMS[@]}")
  fi
  printf '%s\n' "${nums[@]+"${nums[@]}"}"
}

find_plan_file() {
  find "$PLANS_DIR" -maxdepth 1 \
    \( -name "${1}-*.md" -o -name "${1}[a-z]-*.md" \) 2>/dev/null | head -1 || true
}

TMPDIR_STATUS=$(mktemp -d)
trap 'rm -rf "$TMPDIR_STATUS"' EXIT

# Per-plan: s_toml, s_plan (✓ ✗ - dry)
enrich_plan() {
  local num="$1"
  local s_toml="-" s_plan="-"

  cd "$SCRIPT_DIR"

  local plan_file
  plan_file=$(find_plan_file "$num")
  if [[ -z "$plan_file" ]] || [[ ! -f "$plan_file" ]]; then
    echo -e "${YELLOW}[$num]${NC} no plan file found, skipping" >&2
    printf '%s|!|-\n' "$num" > "$TMPDIR_STATUS/$num"
    return 1
  fi

  local toml_file="$CONTEXT_DIR/tasks/${num}-tasks.toml"
  if [[ -f "$toml_file" ]]; then
    if $DRY_RUN; then
      s_toml="dry"
    else
      echo -e "${CYAN}[$num]${NC} enhance-toml..." >&2
      if BACKEND="$BACKEND" MODEL_CLAUDE="$MODEL_CLAUDE" CLAUDE_MODEL="$CLAUDE_MODEL" \
         bash "$PROMPTS_DIR/enhance-toml.sh" "$num"; then
        s_toml="✓"
      else
        s_toml="✗"
      fi
    fi
  else
    s_toml="-"
  fi

  if $DO_ENHANCE_PLAN; then
    if $DRY_RUN; then
      s_plan="dry"
    else
      echo -e "${CYAN}[$num]${NC} enhance-plan..." >&2
      if BACKEND="$BACKEND" MODEL_CLAUDE="$MODEL_CLAUDE" CLAUDE_MODEL="$CLAUDE_MODEL" \
         bash "$PROMPTS_DIR/enhance-plan.sh" "$num"; then
        s_plan="✓"
      else
        s_plan="✗"
      fi
    fi
  else
    s_plan="-"
  fi

  printf '%s|%s|%s\n' "$num" "$s_toml" "$s_plan" > "$TMPDIR_STATUS/$num"
}

mapfile -t PLANS < <(resolve_plans)

if [[ ${#PLANS[@]} -eq 0 ]]; then
  echo "No plans to enrich. Pass plan numbers, --all, or --range START-END." >&2
  exit 1
fi

echo ""
echo -e "${BOLD}bardo-enrich${NC} — ${#PLANS[@]} plan(s)  backend=$BACKEND  parallel=$PARALLEL"
echo -e "${DIM}Shipped steps: enhance-toml (if tasks/NN-tasks.toml exists)$(
  $DO_ENHANCE_PLAN && echo '; enhance-plan' || true
). PRD2/decomposition/verify-chain generators are not in this repo.${NC}"
$DRY_RUN && echo -e "${YELLOW}  dry-run mode${NC}"
echo ""

declare -a pids=()

for num in "${PLANS[@]}"; do
  while [[ ${#pids[@]} -ge $PARALLEL ]]; do
    new_pids=()
    for pid in "${pids[@]}"; do
      if kill -0 "$pid" 2>/dev/null; then
        new_pids+=("$pid")
      fi
    done
    if [[ ${#new_pids[@]} -ge $PARALLEL ]]; then
      sleep 1
    fi
    pids=("${new_pids[@]+"${new_pids[@]}"}")
  done

  enrich_plan "$num" &
  pids+=($!)
done

wait

fmt_step() {
  local label="$1" val="$2"
  case "$val" in
    "✓")   printf "${GREEN}✓ %-14s${NC}" "$label" ;;
    "✗")   printf "${RED}✗ %-14s${NC}" "$label" ;;
    "dry") printf "${YELLOW}? %-14s${NC}" "$label" ;;
    "!")   printf "${RED}! %-14s${NC}" "$label" ;;
    *)     printf "${DIM}- %-14s${NC}" "$label" ;;
  esac
}

echo ""
echo -e "${BOLD}Results:${NC}"
echo ""

any_fail=false
for num in "${PLANS[@]}"; do
  per_plan_status="$TMPDIR_STATUS/$num"
  if [[ ! -f "$per_plan_status" ]]; then
    echo -e "  ${YELLOW}${num}${NC}  (no status recorded)"
    continue
  fi
  IFS='|' read -r _num s_toml s_plan < "$per_plan_status"
  printf "  ${BOLD}%-4s${NC}  " "$num"
  fmt_step "enhance-toml" "$s_toml"
  fmt_step "enhance-plan" "$s_plan"
  echo ""
  [[ "$s_toml" == "✗" || "$s_plan" == "✗" || "$s_toml" == "!" ]] && any_fail=true
done

echo ""
$any_fail && echo -e "${YELLOW}Some steps failed — see stderr above.${NC}" || \
             echo -e "${GREEN}All run steps complete.${NC}"
echo ""
