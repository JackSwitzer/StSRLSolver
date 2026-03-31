#!/bin/bash
# Session End Hook - Updates local beacon with end_time and duration

PAYLOAD=$(cat)

SESSION_ID=$(echo "$PAYLOAD" | jq -r '.session_id // "unknown"')
BEACON_FILE=~/.claude/sessions/${SESSION_ID}.json

if [ -f "$BEACON_FILE" ] && command -v jq &> /dev/null; then
  NOW=$(date +%s)000
  START=$(jq -r '.start_time // 0' "$BEACON_FILE")
  DURATION=$(( (${NOW%000} - ${START%000}) ))
  TMP="${BEACON_FILE}.tmp"
  jq ".end_time = $NOW | .duration_s = $DURATION" "$BEACON_FILE" > "$TMP" 2>/dev/null && mv "$TMP" "$BEACON_FILE"
fi

exit 0
