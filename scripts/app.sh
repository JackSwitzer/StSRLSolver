#!/bin/bash
# Spire Monitor — build, run, and hot-reload the native macOS app.
#
# Usage:
#   ./scripts/app.sh              # Clean build + run (or reload if already running)
#   ./scripts/app.sh --build      # Build release only
#   ./scripts/app.sh --stop       # Kill running instance
#   ./scripts/app.sh --status     # Check if running

set -e
cd "$(dirname "$0")/.."

APP_DIR="packages/app"
APP_NAME="SpireMonitor"
PID_FILE=".run/spire-monitor.pid"

mkdir -p .run

is_running() {
    [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null
}

stop_app() {
    if is_running; then
        local pid=$(cat "$PID_FILE")
        kill "$pid" 2>/dev/null || true
        sleep 0.3
        kill -9 "$pid" 2>/dev/null || true
        rm -f "$PID_FILE"
        echo "  Stopped (PID $pid)"
    fi
    # Also kill any orphaned instances
    pkill -9 -f ".build/.*/$APP_NAME" 2>/dev/null || true
}

launch_app() {
    open "$APP_DIR/.build/debug/$APP_NAME"
    sleep 0.5
    local pid=$(pgrep -n "$APP_NAME" 2>/dev/null || echo "")
    if [ -n "$pid" ]; then
        echo "$pid" > "$PID_FILE"
        echo "  Running (PID $pid)"
    else
        echo "  Launched (PID unknown)"
    fi
}

case "${1:-}" in
    --build)
        echo "Building $APP_NAME (release)..."
        cd "$APP_DIR"
        swift build -c release 2>&1 | tail -5
        echo "Built: $APP_DIR/.build/release/$APP_NAME"
        exit 0
        ;;
    --stop)
        stop_app
        exit 0
        ;;
    --status)
        if is_running; then
            echo "$APP_NAME: running (PID $(cat "$PID_FILE"))"
        else
            echo "$APP_NAME: not running"
            rm -f "$PID_FILE"
        fi
        exit 0
        ;;
esac

# Default: stop -> clean build -> launch
stop_app

echo "Building $APP_NAME..."
cd "$APP_DIR"
# Touch a source file to force recompile (SPM caching workaround)
touch SpireMonitor/App/SpireMonitorApp.swift
swift build 2>&1 | tail -3
cd ../..

launch_app
