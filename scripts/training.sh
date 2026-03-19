#!/bin/bash
# Training runner with process management, caffeinate, and monitoring.
#
# Usage:
#   ./scripts/training.sh start [--games N] [--workers N] [--batch N] [--asc N] [--headless]
#   ./scripts/training.sh stop          # Graceful shutdown (SIGTERM -> wait -> SIGKILL)
#   ./scripts/training.sh status        # Read logs/active/status.json + tail log
#   ./scripts/training.sh resume        # Resume from latest checkpoint
#
# Run directory convention:
#   - Each run creates logs/runs/run_YYYYMMDD_HHMMSS[_label]/
#   - logs/active is a symlink to the current run directory
#   - All tools read from logs/active/ (single source of truth)
#
# Process management:
#   - caffeinate -dims prevents Mac sleep during training
#   - Process group tracking (kills entire pgroup on stop)
#   - SIGTERM -> 10s wait -> SIGKILL
#   - PID file at .run/training.pid

set -e
cd "$(dirname "$0")/.."

PID_DIR=".run"
ACTIVE_LINK="logs/active"

mkdir -p "$PID_DIR" "logs/runs"

# -- Helpers -------------------------------------------------------

create_run_dir() {
    local label="${1:-}"
    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local name="run_${timestamp}"
    [ -n "$label" ] && name="${name}_${label}"
    local dir="logs/runs/${name}"
    mkdir -p "$dir"
    # Symlink target must be relative to the symlink's parent dir (logs/)
    ln -sfn "runs/${name}" "$ACTIVE_LINK"
    echo "$dir"
}

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

    local pgid
    pgid=$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')

    # Send SIGTERM to main process (triggers graceful shutdown + checkpoint save)
    kill -TERM "$pid" 2>/dev/null

    # Wait up to 30 seconds for graceful shutdown
    echo "  Waiting up to 30s for graceful shutdown..."
    for i in $(seq 1 30); do
        kill -0 "$pid" 2>/dev/null || break
        sleep 1
    done

    # Force kill entire process group if still alive
    if kill -0 "$pid" 2>/dev/null; then
        echo "  Force killing process group..."
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

    # Clean up any orphaned worker processes from this project
    local orphans
    orphans=$(pgrep -f "packages.training.training_runner\|multiprocessing.spawn.*spawn_main" 2>/dev/null | grep -v "^$$" || true)
    if [ -n "$orphans" ]; then
        echo "  Cleaning up $(echo "$orphans" | wc -l | tr -d ' ') orphaned workers..."
        echo "$orphans" | xargs kill 2>/dev/null || true
        sleep 1
        echo "$orphans" | xargs kill -9 2>/dev/null || true
    fi

    # Kill any stale caffeinate processes (keep system ones)
    pkill -f "caffeinate -dims" 2>/dev/null || true

    echo "Training stopped."
}

# -- Commands ------------------------------------------------------

cmd_start() {
    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "Training already running (PID $pid). Use 'stop' first."
        exit 1
    fi

    # Parse args
    local games=10000 workers=8 batch=256 asc=0 headless="" resume=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --games)   games=$2; shift 2 ;;
            --workers) workers=$2; shift 2 ;;
            --batch)   batch=$2; shift 2 ;;
            --asc)     asc=$2; shift 2 ;;
            --headless) headless="--headless-after 0"; shift ;;
            --resume)  resume="--resume $2"; shift 2 ;;
            --weekend) weekend="--weekend"; shift ;;
            *) echo "Unknown option: $1"; exit 1 ;;
        esac
    done

    local run_dir
    run_dir=$(create_run_dir)
    local run_log="$run_dir/nohup.log"

    echo "Starting training..."
    echo "  Games: $games | Workers: $workers | Batch: $batch | Ascension: $asc"
    echo "  Run dir: $run_dir"
    echo "  Log: $run_log"

    # Start caffeinate to prevent sleep
    caffeinate -dims &
    echo $! > "$PID_DIR/caffeinate.pid"
    echo "  caffeinate: PID $!"

    # Start training (macOS doesn't have setsid, use nohup instead)
    nohup uv run python -m packages.training.training_runner \
        --games "$games" \
        --workers "$workers" \
        --batch-size "$batch" \
        --ascension "$asc" \
        --run-dir "$run_dir" \
        $headless \
        $resume \
        $weekend \
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

    # Read status from active run
    local status_file="$ACTIVE_LINK/status.json"
    if [ -f "$status_file" ]; then
        echo "--- Latest Status ($status_file) ---"
        uv run python -c "
import json, sys
with open('$status_file') as f:
    s = json.load(f)
print(f'Games:       {s.get(\"total_games\", \"?\")}')
print(f'Wins:        {s.get(\"total_wins\", \"?\")}')
print(f'Win Rate:    {s.get(\"win_rate_100\", s.get(\"win_rate\", \"?\"))}%')
print(f'Avg Floor:   {s.get(\"avg_floor_100\", s.get(\"avg_floor\", \"?\"))}')
print(f'Train Steps: {s.get(\"train_steps\", \"?\")}')
print(f'Games/min:   {s.get(\"games_per_min\", \"?\")}')
print(f'Elapsed:     {s.get(\"elapsed_hours\", \"?\")}h')
print(f'Config:      {s.get(\"config_name\", \"?\")}')
" 2>/dev/null || echo "(status.json parse error)"
    else
        echo "(no status.json yet)"
    fi

    echo ""

    # Tail log from active run, fallback to latest archived run
    local latest_log="$ACTIVE_LINK/nohup.log"
    if [ ! -f "$latest_log" ]; then
        latest_log=$(ls -t logs/runs/run_*/nohup.log 2>/dev/null | head -1)
    fi

    if [ -n "$latest_log" ] && [ -f "$latest_log" ]; then
        echo "--- Last 10 log lines ($latest_log) ---"
        tail -10 "$latest_log"
    fi
}

cmd_weekend() {
    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "Training already running (PID $pid). Use 'stop' first."
        exit 1
    fi

    # Parse args (weekend has larger defaults)
    local games=500000 workers=12 batch=256 asc=0 extra_args=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --games)   games=$2; shift 2 ;;
            --workers) workers=$2; shift 2 ;;
            --batch)   batch=$2; shift 2 ;;
            --asc)     asc=$2; shift 2 ;;
            *) echo "Unknown option: $1"; exit 1 ;;
        esac
    done

    # Check for checkpoint BEFORE creating new run dir (symlink moves)
    local resume_flag=""
    if [ -L "$ACTIVE_LINK" ] && [ -f "$ACTIVE_LINK/shutdown_checkpoint.pt" ]; then
        resume_flag="--resume $(cd "$ACTIVE_LINK" && pwd)/shutdown_checkpoint.pt"
        echo "  Will resume from shutdown checkpoint"
    fi

    local run_dir
    run_dir=$(create_run_dir "weekend")
    local run_log="$run_dir/nohup.log"

    echo "Starting weekend training run..."
    echo "  Games: $games | Workers: $workers | Batch: $batch | Ascension: $asc"
    echo "  Run dir: $run_dir"
    echo "  Log: $run_log"

    # Start caffeinate to prevent sleep (display + idle + system + disk)
    caffeinate -dims &
    echo $! > "$PID_DIR/caffeinate.pid"
    echo "  caffeinate: PID $!"

    nohup uv run python -m packages.training.training_runner \
        --games "$games" \
        --workers "$workers" \
        --batch 24 \
        --batch-size "$batch" \
        --ascension "$asc" \
        --headless-after 0 \
        --run-dir "$run_dir" \
        --hidden-dim 1024 \
        --num-blocks 6 \
        --max-batch-size 128 \
        $resume_flag \
        > "$run_log" 2>&1 &

    local train_pid=$!
    echo "$train_pid" > "$PID_DIR/training.pid"
    echo "  training: PID $train_pid"
    echo ""
    echo "Monitor with: ./scripts/training.sh status"
    echo "Stop with:    ./scripts/training.sh stop"
    echo ""
    echo "Weekend mode: headless, caffeinated, long-running."
}

cmd_resume() {
    if training_alive; then
        echo "Training already running. Stop first."
        exit 1
    fi

    # Look in active run first, then scan archived runs
    local latest_checkpoint=""
    if [ -L "$ACTIVE_LINK" ]; then
        latest_checkpoint=$(ls -t "$ACTIVE_LINK"/checkpoint_*.pt "$ACTIVE_LINK"/shutdown_checkpoint.pt 2>/dev/null | head -1)
    fi
    if [ -z "$latest_checkpoint" ]; then
        latest_checkpoint=$(ls -t logs/runs/*/checkpoint_*.pt logs/runs/*/shutdown_checkpoint.pt 2>/dev/null | head -1)
    fi
    if [ -z "$latest_checkpoint" ]; then
        echo "No checkpoint found. Use 'start' instead."
        exit 1
    fi

    echo "Resuming from: $latest_checkpoint"
    cmd_start --resume "$latest_checkpoint" "$@"
}

cmd_archive() {
    # Archive the current active run
    local label="${1:-}"

    if [ ! -L "$ACTIVE_LINK" ]; then
        echo "No active run to archive (logs/active symlink not found)."
        return 1
    fi

    # Resolve the symlink to an absolute path
    local run_dir
    run_dir=$(cd "$(dirname "$ACTIVE_LINK")" && cd "$(readlink "$ACTIVE_LINK")" && pwd)

    if [ ! -d "$run_dir" ]; then
        echo "Active run directory does not exist: $run_dir"
        return 1
    fi

    # Stop training if running
    if training_alive; then
        echo "Stopping training before archive..."
        stop_training
    fi

    # If a label was provided, rename the run dir to include it
    if [ -n "$label" ]; then
        local base_name
        base_name=$(basename "$run_dir")
        # Only append label if not already in the name
        if [[ "$base_name" != *"_${label}" ]]; then
            local new_dir
            new_dir="$(dirname "$run_dir")/${base_name}_${label}"
            mv "$run_dir" "$new_dir"
            run_dir="$new_dir"
        fi
    fi

    echo "Archived: $run_dir"
    ls -lh "$run_dir/" 2>/dev/null | tail -20

    # Remove the active symlink
    rm -f "$ACTIVE_LINK"
    echo ""
    echo "Active symlink removed. Ready for fresh start."
}

cmd_quick_restart() {
    echo "Quick restart: stop -> save -> restart with code changes..."
    local was_running=false
    if training_alive; then
        was_running=true
        stop_training
    fi

    # Rename old log in the active run dir
    if [ -f "$ACTIVE_LINK/nohup.log" ]; then
        local n=1
        while [ -f "$ACTIVE_LINK/nohup_prev${n}.log" ]; do n=$((n+1)); done
        mv "$ACTIVE_LINK/nohup.log" "$ACTIVE_LINK/nohup_prev${n}.log"
        echo "  Old log -> nohup_prev${n}.log"
    fi

    echo "  Restarting weekend mode (auto-resumes from checkpoint)..."
    cmd_weekend "$@"
}

# -- Main ----------------------------------------------------------

case "${1:-status}" in
    start)   shift; cmd_start "$@" ;;
    stop)    cmd_stop ;;
    status)  cmd_status ;;
    resume)  shift; cmd_resume "$@" ;;
    weekend) shift; cmd_weekend "$@" ;;
    restart) shift; cmd_quick_restart "$@" ;;
    archive) shift; cmd_archive "$@" ;;
    fresh)   shift; cmd_archive "${1:-fresh}" && cmd_weekend "${@:2}" ;;
    update)  shift; echo "Pulling latest code..."; git pull --ff-only && cmd_quick_restart "$@" ;;
    hotfix)  shift; ./scripts/hotfix.sh "$@" ;;
    prune)   shift; uv run python scripts/utils/prune_data.py "$@" ;;
    *)
        echo "Usage: $0 {start|stop|status|resume|weekend|restart|update|hotfix|prune} [options]"
        echo ""
        echo "Commands:"
        echo "  start      Start training (default 10K games, creates logs/runs/run_TIMESTAMP/)"
        echo "  stop       Graceful shutdown (SIGTERM -> 30s -> SIGKILL)"
        echo "  status     Read logs/active/status.json + tail log"
        echo "  resume     Resume from checkpoint in logs/active/ or logs/runs/*/"
        echo "  weekend    Long-running headless mode (default 500K games)"
        echo "  restart    Quick restart: stop, save checkpoint, resume with code changes"
        echo "  archive    Archive current active run (optional label arg), remove symlink"
        echo "  fresh      Archive + start fresh (cold start with distillation from trajectories)"
        echo "  update     Pull latest code + restart (git pull -> restart)"
        echo "  hotfix     Live parameter tuning via SIGUSR1 + reload.json"
        echo "  prune      Prune episodes.jsonl + consolidate top runs (safe while running)"
        echo ""
        echo "Run directory convention:"
        echo "  logs/runs/run_TIMESTAMP[_label]/  -- each run gets a timestamped directory"
        echo "  logs/active                       -- symlink to current run (single source of truth)"
        echo ""
        echo "Options for start/weekend:"
        echo "  --games N      Total games to play"
        echo "  --workers N    Parallel workers (default: 8, weekend: 12)"
        echo "  --batch N      Batch size for PPO (default: 256)"
        echo "  --asc N        Ascension level (default: 0)"
        echo "  --headless     No dashboard output (start only)"
        echo ""
        echo "Options for prune:"
        echo "  --dry-run          Preview without modifying files"
        echo "  --keep N           Keep last N episodes (default: 10000)"
        echo "  --top N            Top N episodes in top_episodes.json (default: 500)"
        echo "  --skip-compress    Skip episode archiving"
        echo "  --skip-top         Skip top episode consolidation"
        exit 1
        ;;
esac
