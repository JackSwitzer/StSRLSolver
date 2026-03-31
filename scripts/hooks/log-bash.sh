#!/bin/bash
# PostToolUse hook: log Bash commands for /harvest analysis + audit trail
# Reads JSON from stdin, appends to session log and audit log

input=$(cat)
tool=$(echo "$input" | jq -r '.tool_name // empty')

if [ "$tool" = "Bash" ]; then
  cmd=$(echo "$input" | jq -r '.tool_input.command // empty')
  ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

  # Session log for /harvest
  echo "{\"ts\":\"$ts\",\"cmd\":$(echo "$cmd" | jq -Rs .)}" >> "$HOME/.claude/session-bash-log.jsonl"

  # Audit log (merged from command-monitor.sh)
  {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Tool: $tool"
    echo "Command: $cmd"
    echo "---"
  } >> "$HOME/.claude/command-audit.log"
fi
