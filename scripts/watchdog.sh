#!/bin/bash
# Training watchdog -- polls milestones.jsonl and status.json every 60s.
#
# Usage:
#   bash scripts/watchdog.sh &           # Run in background
#   bash scripts/watchdog.sh --once      # Single check (for testing)
#
# Reads milestones.jsonl for new events and forwards them to alert.sh.
# Also checks PID health and disk space independently.
# Designed to run via caffeinate alongside training.

set -e
cd "$(dirname "$0")/.."

POLL_INTERVAL=60
ACTIVE_DIR="logs/active"
MILESTONES_FILE="$ACTIVE_DIR/milestones.jsonl"
STATUS_FILE="$ACTIVE_DIR/status.json"
PID_FILE=".run/training.pid"
WATCHDOG_STATE=".run/watchdog_last_line"

# Track the last line we processed in milestones.jsonl
mkdir -p .run
LAST_LINE=0
[ -f "$WATCHDOG_STATE" ] && LAST_LINE=$(cat "$WATCHDOG_STATE")

alert() {
    local severity="$1"
    local message="$2"
    bash scripts/alert.sh "$severity" "$message" 2>/dev/null || true
}

check_milestones() {
    [ ! -f "$MILESTONES_FILE" ] && return

    local total_lines
    total_lines=$(wc -l < "$MILESTONES_FILE" | tr -d ' ')

    if [ "$total_lines" -gt "$LAST_LINE" ]; then
        # Process new lines
        tail -n "+$((LAST_LINE + 1))" "$MILESTONES_FILE" | while IFS= read -r line; do
            local type severity detail
            type=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin).get('type','?'))" 2>/dev/null || echo "?")
            severity=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin).get('severity','info'))" 2>/dev/null || echo "info")
            detail=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin).get('detail',''))" 2>/dev/null || echo "")
            value=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin).get('value',''))" 2>/dev/null || echo "")

            echo "[$(date '+%H:%M:%S')] Milestone: $type ($severity) $detail"
            alert "$severity" "$type: $detail"
        done
        LAST_LINE=$total_lines
        echo "$LAST_LINE" > "$WATCHDOG_STATE"
    fi
}

check_pid() {
    if [ -f "$PID_FILE" ]; then
        local pid
        pid=$(cat "$PID_FILE")
        if ! kill -0 "$pid" 2>/dev/null; then
            echo "[$(date '+%H:%M:%S')] Training PID $pid is DEAD"
            alert "critical" "Training process died (PID $pid)"
            # Remove stale PID to avoid repeated alerts
            rm -f "$PID_FILE"
        fi
    fi
}

check_disk() {
    local free_gb
    free_gb=$(df -g . 2>/dev/null | tail -1 | awk '{print $4}')
    if [ -n "$free_gb" ] && [ "$free_gb" -lt 5 ] 2>/dev/null; then
        echo "[$(date '+%H:%M:%S')] Disk low: ${free_gb}GB free"
        alert "critical" "Disk low: ${free_gb}GB free"
    fi
}

check_status() {
    if [ -f "$STATUS_FILE" ]; then
        local games floor wins
        games=$(python3 -c "import json; s=json.load(open('$STATUS_FILE')); print(s.get('total_games',0))" 2>/dev/null || echo "?")
        floor=$(python3 -c "import json; s=json.load(open('$STATUS_FILE')); print(s.get('avg_floor_100',0))" 2>/dev/null || echo "?")
        wins=$(python3 -c "import json; s=json.load(open('$STATUS_FILE')); print(s.get('total_wins',0))" 2>/dev/null || echo "?")
        echo "[$(date '+%H:%M:%S')] Status: ${games} games, floor ${floor}, ${wins} wins"
    fi
}

run_once() {
    echo "=== Watchdog check $(date '+%Y-%m-%d %H:%M:%S') ==="
    check_milestones
    check_pid
    check_disk
    check_status
}

# Single check mode for testing
if [ "${1:-}" = "--once" ]; then
    run_once
    exit 0
fi

echo "Watchdog started (polling every ${POLL_INTERVAL}s). PID $$"
echo "  Monitoring: $MILESTONES_FILE, $STATUS_FILE"
echo "  Alerts via: scripts/alert.sh -> iMessage"

# Main loop
while true; do
    run_once
    sleep "$POLL_INTERVAL"
done
