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
#   ./bardo-enrich.sh 01 --with-test-gen    # run test generation pipeline per plan
#   ./bardo-enrich.sh 01 --with-scribe-toml # generate scribe task TOML (citation/diagram tasks)
#   ./bardo-enrich.sh 01 --pre-validate     # run compile gate before enrichment
#   ./bardo-enrich.sh 01 --smoke            # run smoke-test.sh after all plans
#   ./bardo-enrich.sh 01 --improve          # run iterative-improve.sh per plan
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
DO_TEST_GEN=false
DO_PRE_VALIDATE=false
DO_SMOKE=false
DO_IMPROVE=false
DO_SCRIBE_TOML=false
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
    --with-test-gen) DO_TEST_GEN=true; export BARDO_TEST_GEN=1; shift ;;
    --pre-validate)  DO_PRE_VALIDATE=true; shift ;;
    --smoke)         DO_SMOKE=true;    shift ;;
    --improve)       DO_IMPROVE=true;  shift ;;
    --with-scribe-toml) DO_SCRIBE_TOML=true; shift ;;
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

# Per-plan status columns: s_toml, s_plan, s_pre_validate, s_verify_chain,
#   s_integration, s_lifecycle, s_terminal, s_verify_toml, s_improve
enrich_plan() {
  local num="$1"
  local s_toml="-" s_plan="-"
  local s_pre_validate="-" s_verify_chain="-"
  local s_integration="-" s_lifecycle="-" s_terminal="-"
  local s_verify_toml="-" s_improve="-"

  cd "$SCRIPT_DIR"

  local plan_file
  plan_file=$(find_plan_file "$num")
  if [[ -z "$plan_file" ]] || [[ ! -f "$plan_file" ]]; then
    echo -e "${YELLOW}[$num]${NC} no plan file found, skipping" >&2
    printf '%s|!|-|-|-|-|-|-|-|-\n' "$num" > "$TMPDIR_STATUS/$num"
    return 1
  fi

  # --- pre-validate (compile gate) ---
  if $DO_PRE_VALIDATE; then
    if $DRY_RUN; then
      s_pre_validate="dry"
    else
      echo -e "${CYAN}[$num]${NC} pre-validate (compile gate)..." >&2
      if bash "$PROMPTS_DIR/multi-gate.sh" "$num" --skip-deny --skip-spec; then
        s_pre_validate="✓"
      else
        s_pre_validate="✗"
      fi
    fi
  fi

  # --- enhance-toml ---
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

  # --- generate-scribe-toml ---
  local s_scribe_toml="-"
  if $DO_SCRIBE_TOML; then
    if $DRY_RUN; then
      s_scribe_toml="dry"
    else
      echo -e "${CYAN}[$num]${NC} generate-scribe-toml..." >&2
      if BACKEND="$BACKEND" MODEL_CLAUDE="$MODEL_CLAUDE" CLAUDE_MODEL="$CLAUDE_MODEL" \
         bash "$PROMPTS_DIR/generate-scribe-toml.sh" "$num"; then
        s_scribe_toml="✓"
      else
        s_scribe_toml="✗"
      fi
    fi
  fi

  # --- enhance-plan ---
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

  # --- test-gen pipeline ---
  if $DO_TEST_GEN; then
    # Auto-generate scribe TOML if missing (part of full enrichment)
    if [[ ! -f "$CONTEXT_DIR/tasks/${num}-scribe-tasks.toml" ]]; then
      if $DRY_RUN; then
        echo -e "${YELLOW}  would run: generate-scribe-toml.sh $num${NC}" >&2
      else
        echo -e "${CYAN}[$num]${NC} generate-scribe-toml (auto)..." >&2
        BACKEND="$BACKEND" MODEL_CLAUDE="$MODEL_CLAUDE" CLAUDE_MODEL="$CLAUDE_MODEL" \
          bash "$PROMPTS_DIR/generate-scribe-toml.sh" "$num" || true
      fi
    fi

    # Step 1: self-verify-chain (must precede parallel generators)
    if $DRY_RUN; then
      s_verify_chain="dry"
    else
      echo -e "${CYAN}[$num]${NC} self-verify-chain..." >&2
      if bash "$PROMPTS_DIR/self-verify-chain.sh" "$num"; then
        s_verify_chain="✓"
      else
        s_verify_chain="✗"
      fi
    fi

    # Step 2: parallel test generators
    if $DRY_RUN; then
      s_integration="dry"; s_lifecycle="dry"; s_terminal="dry"
    else
      local tg_pids=() tg_results
      tg_results=$(mktemp -d)

      echo -e "${CYAN}[$num]${NC} test generators (parallel)..." >&2

      ( if BACKEND="$BACKEND" MODEL_CLAUDE="$MODEL_CLAUDE" CLAUDE_MODEL="$CLAUDE_MODEL" \
           bash "$PROMPTS_DIR/generate-integration-tests.sh" "$num"; then
          echo "✓" > "$tg_results/integration"
        else
          echo "✗" > "$tg_results/integration"
        fi ) &
      tg_pids+=($!)

      ( if BACKEND="$BACKEND" MODEL_CLAUDE="$MODEL_CLAUDE" CLAUDE_MODEL="$CLAUDE_MODEL" \
           bash "$PROMPTS_DIR/generate-lifecycle-scenarios.sh" "$num"; then
          echo "✓" > "$tg_results/lifecycle"
        else
          echo "✗" > "$tg_results/lifecycle"
        fi ) &
      tg_pids+=($!)

      ( if BACKEND="$BACKEND" MODEL_CLAUDE="$MODEL_CLAUDE" CLAUDE_MODEL="$CLAUDE_MODEL" \
           bash "$PROMPTS_DIR/generate-terminal-assertions.sh" "$num"; then
          echo "✓" > "$tg_results/terminal"
        else
          echo "✗" > "$tg_results/terminal"
        fi ) &
      tg_pids+=($!)

      for tg_pid in "${tg_pids[@]}"; do
        wait "$tg_pid" 2>/dev/null || true
      done

      s_integration=$(cat "$tg_results/integration" 2>/dev/null || echo "✗")
      s_lifecycle=$(cat "$tg_results/lifecycle" 2>/dev/null || echo "✗")
      s_terminal=$(cat "$tg_results/terminal" 2>/dev/null || echo "✗")
      rm -rf "$tg_results"
    fi

    # Step 3: generate-verify-toml (consumes outputs of steps 1-2)
    if $DRY_RUN; then
      s_verify_toml="dry"
    else
      echo -e "${CYAN}[$num]${NC} generate-verify-toml..." >&2
      if bash "$PROMPTS_DIR/generate-verify-toml.sh" "$num"; then
        s_verify_toml="✓"
      else
        s_verify_toml="✗"
      fi
    fi
  fi

  # --- iterative improve ---
  if $DO_IMPROVE; then
    if $DRY_RUN; then
      s_improve="dry"
    else
      echo -e "${CYAN}[$num]${NC} iterative-improve..." >&2
      if bash "$PROMPTS_DIR/iterative-improve.sh" "$num" --max-rounds 1 --threshold 5; then
        s_improve="✓"
      else
        s_improve="✗"
      fi
    fi
  fi

  printf '%s|%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n' \
    "$num" "$s_toml" "$s_scribe_toml" "$s_plan" "$s_pre_validate" "$s_verify_chain" \
    "$s_integration" "$s_lifecycle" "$s_terminal" "$s_verify_toml" "$s_improve" \
    > "$TMPDIR_STATUS/$num"
}

mapfile -t PLANS < <(resolve_plans)

if [[ ${#PLANS[@]} -eq 0 ]]; then
  echo "No plans to enrich. Pass plan numbers, --all, or --range START-END." >&2
  exit 1
fi

echo ""
echo -e "${BOLD}bardo-enrich${NC} — ${#PLANS[@]} plan(s)  backend=$BACKEND  parallel=$PARALLEL"
_steps="enhance-toml"
$DO_SCRIBE_TOML   && _steps+="; scribe-toml"
$DO_ENHANCE_PLAN && _steps+="; enhance-plan"
$DO_PRE_VALIDATE  && _steps+="; pre-validate"
$DO_TEST_GEN      && _steps+="; test-gen pipeline"
$DO_IMPROVE       && _steps+="; iterative-improve"
$DO_SMOKE         && _steps+="; smoke-test (post)"
echo -e "${DIM}Steps: ${_steps}${NC}"
$DRY_RUN && echo -e "${YELLOW}  dry-run mode${NC}"
echo ""

# golden-path-index: run once before per-plan work when --with-test-gen
if $DO_TEST_GEN; then
  local_gpi="$CONTEXT_DIR/golden-path-index.json"
  if $DRY_RUN; then
    echo -e "${YELLOW}  would run: golden-path-index.sh${NC}"
  else
    # Freshen the index if it's missing or older than any plan file
    needs_gpi=false
    if [[ ! -f "$local_gpi" ]]; then
      needs_gpi=true
    else
      for f in "$PLANS_DIR"/[0-9]*.md; do
        if [[ "$f" -nt "$local_gpi" ]]; then
          needs_gpi=true
          break
        fi
      done
    fi
    if $needs_gpi; then
      echo -e "${CYAN}[*]${NC} golden-path-index (refreshing)..." >&2
      bash "$PROMPTS_DIR/golden-path-index.sh" || \
        echo -e "${YELLOW}  golden-path-index failed (continuing)${NC}" >&2
    else
      echo -e "${DIM}[*] golden-path-index.json is fresh${NC}" >&2
    fi
  fi
fi

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

# --- smoke-test (runs once after all plans) ---
s_smoke="-"
if $DO_SMOKE; then
  if $DRY_RUN; then
    s_smoke="dry"
    echo -e "${YELLOW}  would run: smoke-test.sh${NC}"
  else
    echo -e "${CYAN}[*]${NC} smoke-test..." >&2
    if bash "$PROMPTS_DIR/smoke-test.sh"; then
      s_smoke="✓"
    else
      s_smoke="✗"
    fi
  fi
fi

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
  IFS='|' read -r _num s_toml s_scribe_toml s_plan s_pre_validate s_verify_chain \
    s_integration s_lifecycle s_terminal s_verify_toml s_improve \
    < "$per_plan_status"
  printf "  ${BOLD}%-4s${NC}  " "$num"
  fmt_step "enhance-toml" "$s_toml"
  $DO_SCRIBE_TOML && fmt_step "scribe-toml" "$s_scribe_toml"
  fmt_step "enhance-plan" "$s_plan"
  $DO_PRE_VALIDATE && fmt_step "pre-validate" "$s_pre_validate"
  $DO_TEST_GEN && {
    fmt_step "verify-chain" "$s_verify_chain"
    fmt_step "integration" "$s_integration"
    fmt_step "lifecycle" "$s_lifecycle"
    fmt_step "terminal" "$s_terminal"
    fmt_step "verify-toml" "$s_verify_toml"
  }
  $DO_IMPROVE && fmt_step "improve" "$s_improve"
  echo ""
  for _s in "$s_toml" "$s_scribe_toml" "$s_plan" "$s_pre_validate" "$s_verify_chain" \
    "$s_integration" "$s_lifecycle" "$s_terminal" "$s_verify_toml" "$s_improve"; do
    [[ "$_s" == "✗" || "$_s" == "!" ]] && any_fail=true
  done
done

# smoke-test result (global, not per-plan)
if $DO_SMOKE; then
  echo ""
  printf "  ${BOLD}%-4s${NC}  " "[*]"
  fmt_step "smoke-test" "$s_smoke"
  echo ""
  [[ "$s_smoke" == "✗" ]] && any_fail=true
fi

echo ""
$any_fail && echo -e "${YELLOW}Some steps failed — see stderr above.${NC}" || \
             echo -e "${GREEN}All run steps complete.${NC}"
echo ""
