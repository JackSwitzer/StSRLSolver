#!/usr/bin/env bash
# Nightly training audit via GPT 5.4 (Codex CLI)
#
# Usage: ./scripts/nightly-audit.sh [run-dir]
#
# Runs a comprehensive audit covering:
#   1. Training status (progress, health, recommendations)
#   2. Code quality sweep (recently changed files, bugs, dead code)
#   3. Combat performance (floor distribution, death patterns, bottlenecks)
#
# Writes combined report to {run-dir}/audits/YYYY-MM-DD-HHMM-audit.md
# Sends email notification via Mail.app (osascript)

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR"

CODEX="/Applications/Codex.app/Contents/Resources/codex"
RUN_DIR="${1:-logs/weekend-run}"
AUDIT_DIR="$RUN_DIR/audits"
DATE="$(date +%Y-%m-%d-%H%M)"
OUTPUT="$AUDIT_DIR/${DATE}-audit.md"
EMAIL_TO="$(git config user.email 2>/dev/null || echo '')"

# ── Preflight checks ──────────────────────────────────────

if [[ ! -x "$CODEX" ]]; then
  echo "ERROR: Codex CLI not found at $CODEX" >&2
  exit 1
fi

mkdir -p "$AUDIT_DIR"

# ── Helper: run codex review ──────────────────────────────

codex_review() {
  local effort="$1"
  local prompt="$2"
  $CODEX exec \
    -m gpt-5.4 \
    -c "reasoning_effort=\"$effort\"" \
    --sandbox read-only \
    "$prompt" 2>&1 || echo "(Codex review failed with exit code $?)"
}

# ── Gather data ───────────────────────────────────────────

echo "[$(date)] Nightly audit starting for $RUN_DIR"

# Training status
STATUS_JSON=""
if [[ -f "$RUN_DIR/status.json" ]]; then
  STATUS_JSON="$(cat "$RUN_DIR/status.json")"
fi

# Training logs (check multiple locations)
LOG_TAIL=""
for logfile in "$RUN_DIR/nohup.log" "$RUN_DIR/training.log"; do
  if [[ -f "$logfile" ]]; then
    LOG_TAIL="$(tail -80 "$logfile")"
    break
  fi
done
if [[ -z "$LOG_TAIL" ]]; then
  LATEST_LOG="$(ls -t "$RUN_DIR"/*.log 2>/dev/null | head -1 || true)"
  if [[ -n "$LATEST_LOG" ]]; then
    LOG_TAIL="$(tail -80 "$LATEST_LOG")"
  fi
fi

# Episode data (last 200 lines for stats)
EPISODE_DATA=""
if [[ -f "$RUN_DIR/episodes.jsonl" ]]; then
  EPISODE_DATA="$(tail -200 "$RUN_DIR/episodes.jsonl")"
fi

# Recently changed Python files (last 24h)
CHANGED_FILES="$(find packages/ -name '*.py' -mtime -1 2>/dev/null | head -20 || true)"
CHANGED_DIFFS=""
if [[ -n "$CHANGED_FILES" ]]; then
  CHANGED_DIFFS="$(git diff --stat HEAD~5 -- packages/ 2>/dev/null || true)"
  # Get actual diff for recently changed training files
  TRAINING_DIFF="$(git diff HEAD~3 -- packages/training/ 2>/dev/null | head -300 || true)"
fi

# Recent git log
GIT_LOG="$(git log --oneline -10 2>/dev/null || true)"

# Test count (quick check)
TEST_COUNT=""
if command -v uv &>/dev/null; then
  TEST_COUNT="$(uv run pytest tests/ -q --co 2>/dev/null | tail -1 || echo 'unavailable')"
fi

# ── Section 1: Training Status Review (high effort) ──────

echo "[$(date)] Running training status review..."

TRAINING_PROMPT="You are auditing an overnight RL training run for a Slay the Spire bot (Watcher, Ascension 20 target).

Analyze the following training data and provide a concise audit covering:
1. **Progress Summary**: Games played, current performance (avg floor, win rate, games/min)
2. **Health Check**: Red flags (stalled training, NaN losses, crashes, low throughput, memory leaks)
3. **Trend Analysis**: Is performance improving, plateauing, or degrading?
4. **Recommendations**: What to adjust next (hyperparams, ascension level, continue/stop)

## status.json
\`\`\`json
${STATUS_JSON:-"(not available)"}
\`\`\`

## Last 80 log lines
\`\`\`
${LOG_TAIL:-"(not available)"}
\`\`\`

## Recent episodes (last 200 lines of episodes.jsonl)
\`\`\`
${EPISODE_DATA:-"(not available)"}
\`\`\`

Be concise and actionable. Use markdown headers."

SECTION1="$(codex_review "high" "$TRAINING_PROMPT")"

# ── Section 2: Code Quality Sweep (extra-high effort) ────

echo "[$(date)] Running code quality sweep..."

CODE_PROMPT="You are reviewing recent code changes for the Slay the Spire RL project.
PROJECT: Python game engine + RL training (PPO, MLX inference, multiprocessing on M4 Mac Mini).
Test suite: ${TEST_COUNT:-'~6060 tests'}.

## Recent git history
\`\`\`
${GIT_LOG:-"(no recent commits)"}
\`\`\`

## Changed files (last 24h)
\`\`\`
${CHANGED_FILES:-"(no changes detected)"}
\`\`\`

## Git diff summary (last 5 commits)
\`\`\`
${CHANGED_DIFFS:-"(no diff available)"}
\`\`\`

## Training code diff (last 3 commits)
\`\`\`diff
${TRAINING_DIFF:-"(no training changes)"}
\`\`\`

Review for:
1. **Critical bugs**: Anything that could cause incorrect training, data corruption, or crashes
2. **Performance issues**: Unnecessary allocations, GIL contention, suboptimal batching
3. **Dead code**: Unused imports, unreachable branches, orphaned functions
4. **Security**: Hardcoded paths, credential leaks, unsafe eval/exec
5. **Ranked improvements**: Top 3 things to fix, with effort estimates

Be concise. Use markdown headers."

SECTION2="$(codex_review "extra-high" "$CODE_PROMPT")"

# ── Section 3: Combat Performance Analysis (high effort) ─

echo "[$(date)] Running combat performance analysis..."

COMBAT_PROMPT="You are analyzing combat performance data for a Slay the Spire RL bot (Watcher class).

Context:
- Act 1 boss is at floor 16. Current bottleneck: ~6.7% reach boss, 0% beat it.
- Human benchmark: 94% winrate at A20. Best bot: 52% at A0.
- Combat uses heuristic planner, strategic decisions use neural net (PPO).

## Recent episode data
\`\`\`
${EPISODE_DATA:-"(not available)"}
\`\`\`

## Training status
\`\`\`json
${STATUS_JSON:-"(not available)"}
\`\`\`

Analyze:
1. **Floor distribution**: Where are games ending? What's the death floor histogram?
2. **Boss performance**: How many reach Act 1 boss? Win rate against specific bosses?
3. **Common death patterns**: What's killing the bot? (HP attrition, specific enemies, bad pathing)
4. **Bottleneck diagnosis**: What single improvement would most increase avg floor?
5. **Comparison to baselines**: How does current perf compare to random/heuristic baselines?

Be concise. Use markdown headers."

SECTION3="$(codex_review "high" "$COMBAT_PROMPT")"

# ── Assemble report ──────────────────────────────────────

cat > "$OUTPUT" << REPORT_EOF
# Nightly Audit Report
**Date**: $(date '+%Y-%m-%d %H:%M')
**Run Directory**: $RUN_DIR
**Auditor**: GPT 5.4 via Codex CLI

---

## 1. Training Status

$SECTION1

---

## 2. Code Quality Sweep

$SECTION2

---

## 3. Combat Performance Analysis

$SECTION3

---

*Generated by scripts/nightly-audit.sh at $(date '+%Y-%m-%d %H:%M:%S')*
REPORT_EOF

echo "[$(date)] Audit written to $OUTPUT"

# ── Email notification ───────────────────────────────────

send_email() {
  local recipient="$1"
  local subject="$2"
  local body="$3"

  # Use osascript to send via Mail.app
  osascript <<APPLESCRIPT
tell application "Mail"
  set newMessage to make new outgoing message with properties {subject:"${subject}", content:"${body}", visible:false}
  tell newMessage
    make new to recipient at end of to recipients with properties {address:"${recipient}"}
  end tell
  send newMessage
end tell
APPLESCRIPT
}

if [[ -n "$EMAIL_TO" ]]; then
  echo "[$(date)] Sending email notification to $EMAIL_TO..."

  # Build a plain-text summary for the email (first ~100 lines of report)
  EMAIL_SUBJECT="[StS-RL] Nightly Audit $(date '+%Y-%m-%d %H:%M')"
  EMAIL_BODY="$(head -100 "$OUTPUT")

---
Full report: $PROJECT_DIR/$OUTPUT
Run: $RUN_DIR"

  if send_email "$EMAIL_TO" "$EMAIL_SUBJECT" "$EMAIL_BODY" 2>/dev/null; then
    echo "[$(date)] Email sent to $EMAIL_TO"
  else
    echo "[$(date)] WARNING: Email send failed (Mail.app may not be configured)" >&2
    # Fallback: try mail command
    if command -v mail &>/dev/null; then
      echo "$EMAIL_BODY" | mail -s "$EMAIL_SUBJECT" "$EMAIL_TO" 2>/dev/null && \
        echo "[$(date)] Email sent via mail command" || \
        echo "[$(date)] WARNING: mail command also failed" >&2
    fi
  fi
else
  echo "[$(date)] WARNING: No email configured (git config user.email not set)" >&2
fi

echo "[$(date)] Nightly audit complete."
