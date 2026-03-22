#!/usr/bin/env bash
# bardo-supervisor.sh — Self-healing supervisor for bardo-ctl.
#
# Catches crashes, feeds crash reports to an AI agent for auto-fix,
# rebuilds, and restarts. Circuit breaker prevents infinite loops.
#
# Usage: ./bardo-supervisor.sh [--agent claude|cursor] [--claude-model opus|sonnet|haiku]
#                              [--max-attempts N] [--same-error-max N] [bardo-ctl args...]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TUI_DIR="$SCRIPT_DIR/tmp/bardo-ctl"
LOG_DIR="$SCRIPT_DIR/tmp/plan-runs"
CRASH_REPORT="$LOG_DIR/crash-report.json"
SUPERVISOR_LOG="$LOG_DIR/supervisor.log"
FIX_HISTORY="$LOG_DIR/fix-history.jsonl"
STDERR_LOG="$LOG_DIR/stderr.log"

MAX_ATTEMPTS=10
SAME_ERROR_MAX=3
MIN_UPTIME_SECS=30
AGENT="claude"         # claude | cursor
CLAUDE_MODEL="opus"    # opus | sonnet | haiku  (Claude CLI short names)

# --- Flag parsing (strip supervisor flags before passing remainder to bardo-ctl) ---

PASSTHROUGH_ARGS=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --agent)
            AGENT="$2"
            shift 2
            ;;
        --max-attempts)
            MAX_ATTEMPTS="$2"
            shift 2
            ;;
        --same-error-max)
            SAME_ERROR_MAX="$2"
            shift 2
            ;;
        --claude-model)
            CLAUDE_MODEL="$2"
            shift 2
            ;;
        *)
            PASSTHROUGH_ARGS+=("$1")
            shift
            ;;
    esac
done
# (passthrough args used directly as PASSTHROUGH_ARGS below)

consecutive_failures=0
declare -A error_attempts  # signature -> count

# --- Logging ---

log() {
    local ts
    ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "[$ts] $*" | tee -a "$SUPERVISOR_LOG"
}

# --- Signal handling ---

cleanup_and_exit() {
    trap - INT TERM  # Reset to avoid recursion
    log "Supervisor exiting (signal)"
    exit 0
}

trap 'cleanup_and_exit' INT TERM

# --- JSON helpers ---

# JSON-escape a string (requires jq, which is already used on line 115+)
json_escape() {
    printf '%s' "$1" | jq -Rs . 2>/dev/null || echo '"unknown"'
}

# --- Fallback crash report ---

build_fallback_report() {
    local stderr_tail
    stderr_tail="$(tail -50 "$STDERR_LOG" 2>/dev/null || echo 'no stderr captured')"
    local log_tail
    log_tail="$(tail -100 "$LOG_DIR/bardo-ctl.log" 2>/dev/null || echo 'no log')"

    # Compute a rough signature from the first line of stderr
    local first_line
    first_line="$(head -1 "$STDERR_LOG" 2>/dev/null || echo 'unknown')"
    local sig
    sig="$(echo -n "$first_line" | shasum -a 256 | cut -c1-12)"

    cat > "$CRASH_REPORT" <<EOJSON
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "crash_type": "unknown",
  "message": $(json_escape "$first_line"),
  "location": null,
  "backtrace": $(json_escape "$stderr_tail"),
  "error_signature": "$sig",
  "recent_logs": [],
  "app_state": {
    "orchestrator_state": "unknown",
    "current_plan": null,
    "current_phase": "unknown",
    "current_iteration": 0,
    "plan_statuses": [],
    "active_agents": [],
    "recent_in_memory_logs": [],
    "gate_running": [],
    "error": null
  },
  "config_summary": "from supervisor fallback",
  "environment": {
    "rust_version": "unknown",
    "os": "$(uname -s)",
    "terminal_size": null,
    "pid": 0
  }
}
EOJSON
    log "Built fallback crash report (sig=$sig)"
}

# --- Context assembly ---

build_claude_context() {
    local context=""

    # 1. Crash report
    context+="## Crash Report\n\`\`\`json\n$(cat "$CRASH_REPORT")\n\`\`\`\n\n"

    # 2. Recent logs
    local recent_log
    recent_log="$(tail -200 "$LOG_DIR/bardo-ctl.log" 2>/dev/null || echo 'no log')"
    context+="## Recent Logs (last 200 lines)\n\`\`\`\n${recent_log}\n\`\`\`\n\n"

    # 3. Stderr
    local stderr_content
    stderr_content="$(tail -100 "$STDERR_LOG" 2>/dev/null || echo 'no stderr')"
    context+="## Stderr\n\`\`\`\n${stderr_content}\n\`\`\`\n\n"

    # 4. Previous fix attempts for this signature
    local sig
    sig="$(jq -r '.error_signature' "$CRASH_REPORT" 2>/dev/null || echo 'unknown')"
    if [[ -f "$FIX_HISTORY" ]]; then
        local prior_fixes
        prior_fixes="$(grep "\"$sig\"" "$FIX_HISTORY" 2>/dev/null || echo '')"
        if [[ -n "$prior_fixes" ]]; then
            context+="## Prior Fix Attempts (SAME ERROR — try a different approach)\n\`\`\`\n${prior_fixes}\n\`\`\`\n\n"
        fi
    fi

    # 5. Uncommitted diff
    local diff
    diff="$(cd "$TUI_DIR" && git diff -- src/ 2>/dev/null | head -500 || echo 'no diff')"
    if [[ -n "$diff" && "$diff" != "no diff" ]]; then
        context+="## Uncommitted Changes in tmp/bardo-ctl/src/\n\`\`\`diff\n${diff}\n\`\`\`\n\n"
    fi

    echo -e "$context"
}

# --- Agent invocations ---

build_fix_prompt() {
    local context
    context="$(build_claude_context)"
    cat <<PROMPT
You are fixing a crash in bardo-ctl, a Rust TUI application.

${context}

## Rules
- Only modify files under tmp/bardo-ctl/src/
- Fix the ROOT CAUSE, not the symptom
- Keep changes minimal — smallest diff that fixes the crash
- The code must compile after your fix (cargo check must pass)
- If prior fix attempts are shown above, try a DIFFERENT approach

Fix the crash now.
PROMPT
}

invoke_claude_fix() {
    local sig="$1"
    log "Invoking Claude to fix error $sig"

    local prompt
    prompt="$(build_fix_prompt)"

    if claude --print \
        --model "$CLAUDE_MODEL" \
        --max-turns 10 \
        --allowedTools "Read,Edit,Glob,Grep,Bash(cargo:*),Bash(ls:*),Bash(wc:*),Bash(cat:*)" \
        --add-dir "$TUI_DIR" \
        "$prompt" >> "$SUPERVISOR_LOG" 2>&1; then
        log "Claude ($CLAUDE_MODEL) fix attempt completed for $sig"
    else
        log "Claude ($CLAUDE_MODEL) fix invocation failed for $sig"
    fi
}

invoke_cursor_fix() {
    local sig="$1"
    log "Invoking Cursor agent to fix error $sig"

    local prompt
    prompt="$(build_fix_prompt)"

    if agent "$prompt" >> "$SUPERVISOR_LOG" 2>&1; then
        log "Cursor agent fix attempt completed for $sig"
    else
        log "Cursor agent fix invocation failed for $sig"
    fi
}

invoke_fix() {
    local sig
    sig="$(jq -r '.error_signature' "$CRASH_REPORT" 2>/dev/null || echo 'unknown')"
    case "$AGENT" in
        cursor) invoke_cursor_fix "$sig" ;;
        claude) invoke_claude_fix "$sig" ;;
        *)
            log "Unknown agent '$AGENT', falling back to claude"
            invoke_claude_fix "$sig"
            ;;
    esac
}

# --- Build ---

rebuild() {
    log "Rebuilding bardo-ctl..."
    if (cd "$TUI_DIR" && cargo build --release 2>&1 | tee -a "$SUPERVISOR_LOG"); then
        log "Build succeeded"
        return 0
    else
        log "Build FAILED"
        return 1
    fi
}

# --- Fix history ---

record_fix() {
    local sig="$1"
    local success="$2"
    local diff_stat
    diff_stat="$(cd "$TUI_DIR" && git diff --stat -- src/ 2>/dev/null | tail -1 || echo 'unknown')"
    local files_changed
    files_changed="$(cd "$TUI_DIR" && git diff --name-only -- src/ 2>/dev/null | tr '\n' ', ' || echo 'unknown')"

    local entry
    entry="{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"error_signature\":\"$sig\",\"success\":$success,\"files_changed\":\"$files_changed\",\"diff_stat\":\"$diff_stat\"}"
    echo "$entry" >> "$FIX_HISTORY"
}

# --- Circuit breaker ---

alert_and_exit() {
    log "CIRCUIT BREAKER TRIPPED: $consecutive_failures consecutive failures"
    # macOS notification
    osascript -e 'display notification "bardo-ctl supervisor gave up after '"$consecutive_failures"' failures" with title "bardo-supervisor"' 2>/dev/null || true
    exit 1
}

# --- Main loop ---

mkdir -p "$LOG_DIR"

# Resolve binary path like bardo-ctl.sh does
RELEASE_BIN="${TUI_DIR}/target/release/bardo-ctl"
if [[ -x "$RELEASE_BIN" ]]; then
    BARDO_CTL_CMD="$RELEASE_BIN"
else
    BARDO_CTL_CMD="cargo run --release --manifest-path ${TUI_DIR}/Cargo.toml --"
fi

# Match bardo-ctl.sh default flags so the supervisor gets the same behavior.
DEFAULT_FLAGS=(--parallel --pre-plan --repo-root "$SCRIPT_DIR")

log "Supervisor starting (agent=$AGENT claude_model=$CLAUDE_MODEL, MAX_ATTEMPTS=$MAX_ATTEMPTS, SAME_ERROR_MAX=$SAME_ERROR_MAX)"

# Always rebuild on fresh start to pick up source changes
rebuild

while true; do
    rm -f "$CRASH_REPORT"
    start_time="$(date +%s)"

    # Run bardo-ctl in the foreground so it inherits the terminal directly.
    # Backgrounding with & breaks crossterm's EventStream initialization because
    # mio's kqueue-based event source fails when the process is in a background
    # process group. Running in the foreground avoids this entirely — signals
    # (INT/TERM) go to the process group and the child handles them naturally.
    set +e
    RUST_BACKTRACE=1 $BARDO_CTL_CMD "${DEFAULT_FLAGS[@]}" "${PASSTHROUGH_ARGS[@]}" 2>"$STDERR_LOG"
    exit_code=$?
    set -e

    # Clean exit or user quit (Ctrl-C = 130)
    if [[ $exit_code -eq 0 ]] || [[ $exit_code -eq 130 ]]; then
        log "bardo-ctl exited cleanly (code=$exit_code)"
        break
    fi

    log "bardo-ctl crashed (exit_code=$exit_code)"

    # If it ran long enough, this is a new problem — reset counter
    end_time="$(date +%s)"
    elapsed=$((end_time - start_time))
    if [[ $elapsed -ge $MIN_UPTIME_SECS ]]; then
        consecutive_failures=0
    fi
    consecutive_failures=$((consecutive_failures + 1))

    # Build fallback crash report from stderr if bardo-ctl didn't write one
    if [[ ! -f "$CRASH_REPORT" ]]; then
        build_fallback_report
    fi

    # Check circuit breakers
    sig="$(jq -r '.error_signature' "$CRASH_REPORT" 2>/dev/null || echo 'unknown')"
    error_attempts[$sig]=$(( ${error_attempts[$sig]:-0} + 1 ))

    if [[ ${error_attempts[$sig]} -gt $SAME_ERROR_MAX ]]; then
        log "Giving up on error $sig after $SAME_ERROR_MAX fix attempts"
        if [[ $consecutive_failures -ge $MAX_ATTEMPTS ]]; then
            alert_and_exit
        fi
        log "Restarting without fix (hoping for a different crash path)"
        sleep 2
        continue
    fi

    if [[ $consecutive_failures -ge $MAX_ATTEMPTS ]]; then
        alert_and_exit
    fi

    # Invoke agent to fix
    invoke_fix

    # Rebuild
    if rebuild; then
        record_fix "$sig" "true"
    else
        record_fix "$sig" "false"
        consecutive_failures=$((consecutive_failures + 1))
        log "Fix didn't compile, continuing to next attempt"
        sleep 2
        continue
    fi

    # Reset terminal state in case TUI left it in raw/alt-screen mode
    stty sane 2>/dev/null || true
    tput reset 2>/dev/null || true

    log "Restarting bardo-ctl (attempt $consecutive_failures/$MAX_ATTEMPTS)"
    sleep 1
done

log "Supervisor exiting normally"
