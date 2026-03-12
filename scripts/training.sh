#!/bin/bash
# Training runner with process management, caffeinate, and monitoring.
#
# Usage:
#   ./scripts/training.sh start [--games N] [--workers N] [--batch N] [--asc N] [--headless]
#   ./scripts/training.sh stop          # Graceful shutdown (SIGTERM → wait → SIGKILL)
#   ./scripts/training.sh status        # Read status.json + tail log
#   ./scripts/training.sh resume        # Resume from latest checkpoint
#
# Process management:
#   - caffeinate -dims prevents Mac sleep during training
#   - Process group tracking (kills entire pgroup on stop)
#   - SIGTERM → 10s wait → SIGKILL
#   - PID file at .run/training.pid

set -e
cd "$(dirname "$0")/.."

PID_DIR=".run"
LOG_DIR="logs/training"
STATUS_FILE="$LOG_DIR/status.json"

mkdir -p "$PID_DIR" "$LOG_DIR"

# ── Helpers ─────────────────────────────────────────────

training_alive() {
    [ -f "$PID_DIR/training.pid" ] && kill -0 "$(cat "$PID_DIR/training.pid")" 2>/dev/null
}

stop_training() {
    if [ ! -f "$PID_DIR/training.pid" ]; then
        echo "Training: not running"
        return 0
    fi

    local pid
    pid=$(cat "$PID_DIR/training.pid")

    if ! kill -0 "$pid" 2>/dev/null; then
        echo "Training: not running (stale PID file)"
        rm -f "$PID_DIR/training.pid" "$PID_DIR/caffeinate.pid"
        return 0
    fi

    echo "Stopping training (PID $pid)..."

    # Kill entire process group
    local pgid
    pgid=$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')

    if [ -n "$pgid" ] && [ "$pgid" != "0" ]; then
        kill -- -"$pgid" 2>/dev/null || kill "$pid" 2>/dev/null
    else
        kill "$pid" 2>/dev/null
    fi

    # Wait up to 10 seconds for graceful shutdown
    for i in $(seq 1 10); do
        kill -0 "$pid" 2>/dev/null || break
        sleep 1
    done

    # Force kill if still alive
    if kill -0 "$pid" 2>/dev/null; then
        echo "  Force killing..."
        if [ -n "$pgid" ] && [ "$pgid" != "0" ]; then
            kill -9 -- -"$pgid" 2>/dev/null || true
        fi
        kill -9 "$pid" 2>/dev/null || true
    fi

    # Stop caffeinate
    if [ -f "$PID_DIR/caffeinate.pid" ]; then
        local caf_pid
        caf_pid=$(cat "$PID_DIR/caffeinate.pid")
        kill "$caf_pid" 2>/dev/null || true
        rm -f "$PID_DIR/caffeinate.pid"
    fi

    rm -f "$PID_DIR/training.pid"
    echo "Training stopped."
}

# ── Commands ────────────────────────────────────────────

cmd_start() {
    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "Training already running (PID $pid). Use 'stop' first."
        exit 1
    fi

    # Parse args
    local games=10000 workers=8 batch=256 asc=0 headless=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --games)   games=$2; shift 2 ;;
            --workers) workers=$2; shift 2 ;;
            --batch)   batch=$2; shift 2 ;;
            --asc)     asc=$2; shift 2 ;;
            --headless) headless="--headless-after 0"; shift ;;
            *) echo "Unknown option: $1"; exit 1 ;;
        esac
    done

    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local run_log="$LOG_DIR/run_${timestamp}.log"

    echo "Starting training..."
    echo "  Games: $games | Workers: $workers | Batch: $batch | Ascension: $asc"
    echo "  Log: $run_log"

    # Start caffeinate to prevent sleep
    caffeinate -dims &
    echo $! > "$PID_DIR/caffeinate.pid"
    echo "  caffeinate: PID $!"

    # Start training (macOS doesn't have setsid, use nohup instead)
    nohup uv run python -m packages.training.overnight \
        --games "$games" \
        --workers "$workers" \
        --batch-size "$batch" \
        --ascension "$asc" \
        $headless \
        > "$run_log" 2>&1 &

    local train_pid=$!
    echo "$train_pid" > "$PID_DIR/training.pid"
    echo "  training: PID $train_pid"
    echo ""
    echo "Monitor with: ./scripts/training.sh status"
    echo "Stop with:    ./scripts/training.sh stop"
}

cmd_stop() {
    stop_training
}

cmd_status() {
    echo "=== Training Status ==="

    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "State: RUNNING (PID $pid)"

        # Show caffeinate
        if [ -f "$PID_DIR/caffeinate.pid" ]; then
            local caf_pid
            caf_pid=$(cat "$PID_DIR/caffeinate.pid")
            if kill -0 "$caf_pid" 2>/dev/null; then
                echo "Sleep prevention: active (caffeinate PID $caf_pid)"
            fi
        fi

        # Worker count
        local worker_count
        worker_count=$(pgrep -P "$pid" 2>/dev/null | wc -l | tr -d ' ')
        echo "Workers: $worker_count"
    else
        echo "State: STOPPED"
        rm -f "$PID_DIR/training.pid"
    fi

    echo ""

    # Read status.json if it exists
    if [ -f "$STATUS_FILE" ]; then
        echo "--- Latest Status ---"
        python3 -c "
import json, sys
with open('$STATUS_FILE') as f:
    s = json.load(f)
print(f'Episodes:    {s.get(\"total_episodes\", \"?\")}')
print(f'Avg Floor:   {s.get(\"avg_floor\", \"?\")}')
print(f'Max Floor:   {s.get(\"max_floor\", \"?\")}')
print(f'Win Rate:    {s.get(\"win_rate\", \"?\")}')
print(f'Train Steps: {s.get(\"train_steps\", \"?\")}')
print(f'Games/min:   {s.get(\"games_per_min\", \"?\")}')
" 2>/dev/null || echo "(status.json parse error)"
    else
        echo "(no status.json yet)"
    fi

    echo ""

    # Tail latest log
    local latest_log
    latest_log=$(ls -t "$LOG_DIR"/run_*.log 2>/dev/null | head -1)
    if [ -n "$latest_log" ]; then
        echo "--- Last 10 log lines ---"
        tail -10 "$latest_log"
    fi
}

cmd_resume() {
    if training_alive; then
        echo "Training already running. Stop first."
        exit 1
    fi

    local latest_checkpoint
    latest_checkpoint=$(ls -t logs/overnight/*/checkpoint_*.pt 2>/dev/null | head -1)
    if [ -z "$latest_checkpoint" ]; then
        echo "No checkpoint found. Use 'start' instead."
        exit 1
    fi

    echo "Resuming from: $latest_checkpoint"
    cmd_start --resume "$latest_checkpoint" "$@"
}

# ── Main ────────────────────────────────────────────────

case "${1:-status}" in
    start)   shift; cmd_start "$@" ;;
    stop)    cmd_stop ;;
    status)  cmd_status ;;
    resume)  shift; cmd_resume "$@" ;;
    *)
        echo "Usage: $0 {start|stop|status|resume} [options]"
        echo ""
        echo "Options for start:"
        echo "  --games N      Total games to play (default: 10000)"
        echo "  --workers N    Parallel workers (default: 8)"
        echo "  --batch N      Batch size for PPO (default: 256)"
        echo "  --asc N        Ascension level (default: 0)"
        echo "  --headless     No dashboard output"
        exit 1
        ;;
esac
