#!/usr/bin/env bash
# Nightly training audit via GPT 5.4 (Codex CLI)
#
# Usage: ./scripts/nightly-audit.sh [run-dir]
#
# Reads status.json and last 50 lines of nohup.log from the run directory,
# sends them to GPT 5.4 for analysis, and writes a report to {run-dir}/audits/.

set -euo pipefail

CODEX="/Applications/Codex.app/Contents/Resources/codex"
RUN_DIR="${1:-logs/weekend-run}"
AUDIT_DIR="$RUN_DIR/audits"
DATE="$(date +%Y-%m-%d-%H%M)"
OUTPUT="$AUDIT_DIR/${DATE}-audit.md"

if [[ ! -x "$CODEX" ]]; then
  echo "ERROR: Codex CLI not found at $CODEX" >&2
  exit 1
fi

# Gather data
STATUS_JSON=""
if [[ -f "$RUN_DIR/status.json" ]]; then
  STATUS_JSON="$(cat "$RUN_DIR/status.json")"
fi

LOG_TAIL=""
if [[ -f "$RUN_DIR/nohup.log" ]]; then
  LOG_TAIL="$(tail -50 "$RUN_DIR/nohup.log")"
elif ls "$RUN_DIR"/*.log 1>/dev/null 2>&1; then
  LATEST_LOG="$(ls -t "$RUN_DIR"/*.log 2>/dev/null | head -1)"
  LOG_TAIL="$(tail -50 "$LATEST_LOG")"
fi

if [[ -z "$STATUS_JSON" && -z "$LOG_TAIL" ]]; then
  echo "ERROR: No status.json or logs found in $RUN_DIR. Nothing to audit." >&2
  exit 1
fi

# Build prompt
PROMPT="You are auditing an overnight RL training run for a Slay the Spire bot (Watcher, Ascension 20 target).

Analyze the following training data and provide a concise audit report covering:
1. **Progress Summary**: Games played, current performance (avg floor, win rate, games/min)
2. **Health Check**: Any red flags (stalled training, NaN losses, crashes, low throughput)
3. **Recommendations**: What to adjust next (hyperparams, ascension level, continue/stop)
4. **Notable Patterns**: Any interesting trends in the log output

## status.json
\`\`\`json
${STATUS_JSON:-"(not available)"}
\`\`\`

## Last 50 log lines
\`\`\`
${LOG_TAIL:-"(not available)"}
\`\`\`

Write the report in markdown format. Be concise and actionable."

# Run via codex exec
mkdir -p "$AUDIT_DIR"
$CODEX exec \
  -m gpt-5.4 \
  -c 'reasoning_effort="high"' \
  --sandbox read-only \
  "$PROMPT" > "$OUTPUT" 2>&1

echo "Audit written to $OUTPUT"
