#!/bin/bash
# Service manager for STS RL infrastructure.
# PID-based process control — no blind port killing.
#
# Usage:
#   ./scripts/services.sh start        # Start WS + Vite
#   ./scripts/services.sh stop         # Graceful shutdown via PID files
#   ./scripts/services.sh restart      # Stop then start
#   ./scripts/services.sh status       # Show what's running
#
# Ports (single source of truth):
#   WS_PORT=8080   (WebSocket server — training coordinator)
#   VIZ_PORT=5174  (Vite dev server — React frontend)

set -e
cd "$(dirname "$0")/.."

# ── Config ──────────────────────────────────────────────
WS_PORT=${STS_WS_PORT:-8080}
VIZ_PORT=${STS_VIZ_PORT:-5174}
PID_DIR=".run"
LOG_DIR="/tmp/sts-rl"

mkdir -p "$PID_DIR" "$LOG_DIR"

# ── Helpers ─────────────────────────────────────────────

pid_alive() {
    [ -f "$PID_DIR/$1.pid" ] && kill -0 "$(cat "$PID_DIR/$1.pid")" 2>/dev/null
}

stop_service() {
    local name=$1
    if [ -f "$PID_DIR/$name.pid" ]; then
        local pid
        pid=$(cat "$PID_DIR/$name.pid")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null
            # Wait up to 3 seconds for graceful shutdown
            for i in 1 2 3; do
                kill -0 "$pid" 2>/dev/null || break
                sleep 1
            done
            # Force kill if still alive
            kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null
            echo "  $name (PID $pid): stopped"
        else
            echo "  $name: not running (stale PID file)"
        fi
        rm -f "$PID_DIR/$name.pid"
    else
        echo "  $name: not running"
    fi
}

claim_port() {
    local port=$1 name=$2
    local stale_pid
    stale_pid=$(lsof -ti:"$port" -sTCP:LISTEN 2>/dev/null || true)
    if [ -n "$stale_pid" ]; then
        echo "  $name: port $port held by PID $stale_pid (not ours) — reclaiming"
        kill "$stale_pid" 2>/dev/null || true
        sleep 1
        # verify it released
        if lsof -i :"$port" -sTCP:LISTEN >/dev/null 2>&1; then
            kill -9 "$stale_pid" 2>/dev/null || true
            sleep 1
        fi
    fi
}

start_ws() {
    if pid_alive ws; then
        echo "  ws: already running (PID $(cat $PID_DIR/ws.pid))"
        return 0
    fi
    claim_port "$WS_PORT" ws
    uv run python -m packages.server --port "$WS_PORT" > "$LOG_DIR/ws.log" 2>&1 &
    echo $! > "$PID_DIR/ws.pid"
    echo "  ws: started (PID $!, port $WS_PORT)"
}

start_viz() {
    if pid_alive viz; then
        echo "  viz: already running (PID $(cat $PID_DIR/viz.pid))"
        return 0
    fi
    claim_port "$VIZ_PORT" viz
    cd packages/viz
    VITE_WS_URL="ws://localhost:$WS_PORT" bun dev --port "$VIZ_PORT" > "$LOG_DIR/viz.log" 2>&1 &
    echo $! > "../../$PID_DIR/viz.pid"
    cd ../..
    echo "  viz: started (PID $(cat $PID_DIR/viz.pid), port $VIZ_PORT)"
}

wait_for_port() {
    local port=$1 name=$2 max=$3
    for i in $(seq 1 "$max"); do
        if lsof -i :"$port" -sTCP:LISTEN >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    echo "  WARNING: $name did not bind to port $port within ${max}s"
    return 1
}

# ── Commands ────────────────────────────────────────────

cmd_start() {
    echo "Starting services..."
    start_ws
    wait_for_port "$WS_PORT" "ws" 5
    start_viz
    wait_for_port "$VIZ_PORT" "viz" 5
    echo ""
    echo "Ready:"
    echo "  Viz: http://localhost:$VIZ_PORT"
    echo "  WS:  ws://localhost:$WS_PORT"
    echo "  Logs: $LOG_DIR/"
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
            local pid
            pid=$(cat "$PID_DIR/$svc.pid")
            local port
            [ "$svc" = "ws" ] && port=$WS_PORT || port=$VIZ_PORT
            echo "  $svc: running (PID $pid, port $port)"
        else
            echo "  $svc: stopped"
            rm -f "$PID_DIR/$svc.pid"
        fi
    done
}

cmd_restart() {
    cmd_stop
    sleep 1
    cmd_start
}

# ── Main ────────────────────────────────────────────────

case "${1:-status}" in
    start)   cmd_start ;;
    stop)    cmd_stop ;;
    restart) cmd_restart ;;
    status)  cmd_status ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac
