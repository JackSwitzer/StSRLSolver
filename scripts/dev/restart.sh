#!/bin/bash
# Restart the STS dashboard with clean cache
set -e

cd "$(dirname "$0")/../.."
PROJECT_ROOT=$(pwd)

echo "=== STS Dashboard Restart ==="

# 1. Stop existing server
echo "Stopping existing server..."
PID_FILE="/tmp/sts_dashboard.pid"
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        kill "$PID" 2>/dev/null || true
        sleep 0.5
    fi
    rm -f "$PID_FILE"
fi

# Also kill any orphaned uvicorn processes on port 8080
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

# 2. Clear cache
echo "Clearing Python cache..."
"$PROJECT_ROOT/scripts/dev/clear_cache.sh"

# 3. Start fresh
echo "Starting dashboard..."
if [ "$1" == "--fg" ] || [ "$1" == "-f" ]; then
    # Foreground mode
    uv run scripts/sts.py dashboard --fg --no-browser
else
    # Background mode (default)
    uv run scripts/sts.py dashboard --no-browser
    echo ""
    echo "Dashboard running at http://127.0.0.1:8080"
    echo "Use './scripts/dev/restart.sh --fg' for foreground mode"
fi
