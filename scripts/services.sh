#!/bin/bash
# Service manager for STS RL infrastructure.
# After the React->SwiftUI migration, only the training pipeline remains.
# This script is kept for backwards compatibility with training.sh.
#
# Usage:
#   ./scripts/services.sh status       # Show what's running
#   ./scripts/services.sh stop         # Stop any lingering services

set -e
cd "$(dirname "$0")/.."

PID_DIR=".run"
LOG_DIR="/tmp/sts-rl"

mkdir -p "$PID_DIR" "$LOG_DIR"

pid_alive() {
    [ -f "$PID_DIR/$1.pid" ] && kill -0 "$(cat "$PID_DIR/$1.pid")" 2>/dev/null
}

stop_service() {
    local name=$1
    if [ -f "$PID_DIR/$name.pid" ]; then
        local pid
        pid=$(cat "$PID_DIR/$name.pid")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            for i in 1 2 3 4 5; do
                kill -0 "$pid" 2>/dev/null || break
                sleep 1
            done
            kill -9 "$pid" 2>/dev/null || true
            echo "  $name (PID $pid): stopped"
        else
            echo "  $name: not running (stale PID file)"
        fi
        rm -f "$PID_DIR/$name.pid"
    else
        echo "  $name: not running"
    fi
}

cmd_stop() {
    echo "Stopping services..."
    stop_service viz
    stop_service ws
}

cmd_status() {
    echo "STS RL Services:"
    for svc in ws viz; do
        if pid_alive "$svc"; then
            echo "  $svc: running (PID $(cat "$PID_DIR/$svc.pid"))"
        else
            echo "  $svc: stopped"
            rm -f "$PID_DIR/$svc.pid"
        fi
    done
    echo ""
    echo "Note: Dashboard is now native (Spire Monitor). Run: ./scripts/app.sh"
}

case "${1:-status}" in
    stop)    cmd_stop ;;
    status)  cmd_status ;;
    start|restart)
        echo "Note: WS + Vite services are no longer needed."
        echo "Dashboard is now native SwiftUI. Run: ./scripts/app.sh"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac
