#!/bin/bash
# Session beacon - Update activity and work status (local only)
# Fires on PostToolUse

PAYLOAD=$(cat)

SESSION_ID=$(echo "$PAYLOAD" | jq -r '.session_id // "unknown"')
HOOK_EVENT=$(echo "$PAYLOAD" | jq -r '.hook_event_name // ""')
TOOL_NAME=$(echo "$PAYLOAD" | jq -r '.tool_name // ""')

WORK_STATUS="working"
if [ "$HOOK_EVENT" = "Stop" ]; then
  WORK_STATUS="idle"
elif [ "$TOOL_NAME" = "AskUserQuestion" ]; then
  WORK_STATUS="waiting_for_input"
fi

BEACON_FILE=~/.claude/sessions/${SESSION_ID}.json
if [ -f "$BEACON_FILE" ] && command -v jq &> /dev/null; then
  TMP="${BEACON_FILE}.tmp"
  jq ".lastActivity = $(date +%s)000 | .workStatus = \"$WORK_STATUS\"" "$BEACON_FILE" > "$TMP" 2>/dev/null && mv "$TMP" "$BEACON_FILE"
fi

exit 0
