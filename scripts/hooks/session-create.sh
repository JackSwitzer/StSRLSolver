#!/bin/bash
# Session Create Hook - Creates local beacon when Claude starts

PAYLOAD=$(cat)

SESSION_ID=$(echo "$PAYLOAD" | jq -r '.session_id // "unknown"')
CWD=$(echo "$PAYLOAD" | jq -r '.cwd // env.PWD')
PROJECT_NAME=$(basename "$CWD")
MACHINE_ID=$(hostname)

mkdir -p ~/.claude/sessions
cat > ~/.claude/sessions/${SESSION_ID}.json <<EOF
{
  "session_id": "$SESSION_ID",
  "project_path": "$CWD",
  "project_name": "$PROJECT_NAME",
  "start_time": $(date +%s)000,
  "machine_id": "$MACHINE_ID"
}
EOF

exit 0
